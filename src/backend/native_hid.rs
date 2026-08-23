//! Direct Native Linux HID backend for MSI Mystic Light (MS-1565).
//!
//! Communicates directly with the keyboard microcontroller via `/dev/hidraw*`
//! using HID Feature Reports. Saves lighting configurations directly to onboard
//! flash memory for permanent hardware-level persistence across reboots.

use async_trait::async_trait;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

use crate::backend::trait_def::{BackendStatus, RgbBackend};
use crate::domain::color::RgbColor;
use crate::domain::device::{DeviceInfo, RgbState};
use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::error::BackendError;
use crate::domain::zone::{ZoneConfig, ZoneId};

/// Target MSI Vendor ID.
pub const MSI_VENDOR_ID: u16 = 0x1462;
/// Target MSI Katana 15 HX MysticLight Product ID.
pub const MSI_PRODUCT_ID: u16 = 0x1601;

/// Linux ioctl command for `HIDIOCSFEATURE(64)`.
/// `_IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, 64) = 0xc0404806`
#[cfg(target_os = "linux")]
const HIDIOCSFEATURE_64: libc::c_ulong = 0xc0404806;

/// Direct native Linux `/dev/hidraw` hardware backend for MSI Mystic Light.
///
/// Example:
/// ```no_run
/// use msi_rgb::backend::native_hid::NativeHidBackend;
/// use msi_rgb::backend::RgbBackend;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut backend = NativeHidBackend::new();
///     backend.connect().await?;
///     assert!(backend.status().is_connected());
///     Ok(())
/// }
/// ```
pub struct NativeHidBackend {
    device_path: Option<PathBuf>,
    device_file: Option<File>,
    status: BackendStatus,
    current_zone_mask: Option<u8>,
}

impl NativeHidBackend {
    /// Creates a new uninitialized `NativeHidBackend`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            device_path: None,
            device_file: None,
            status: BackendStatus::Disconnected {
                reason: "Not connected".to_string(),
            },
            current_zone_mask: None,
        }
    }

    /// Finds the `/dev/hidraw*` device node corresponding to VID:PID 1462:1601.
    fn find_hidraw_device() -> Option<PathBuf> {
        let sys_hidraw = Path::new("/sys/class/hidraw");
        if !sys_hidraw.exists() {
            return None;
        }

        if let Ok(entries) = std::fs::read_dir(sys_hidraw) {
            for entry in entries.flatten() {
                let uevent_path = entry.path().join("device").join("uevent");
                if let Ok(content) = std::fs::read_to_string(uevent_path) {
                    // Look for HID_ID=0003:00001462:00001601 or similar
                    let upper = content.to_uppercase();
                    if upper.contains("1462") && upper.contains("1601") {
                        let dev_name = entry.file_name();
                        let dev_path = Path::new("/dev").join(dev_name);
                        if dev_path.exists() {
                            info!("Discovered MSI MS-1565 keyboard at {}", dev_path.display());
                            return Some(dev_path);
                        }
                    }
                }
            }
        }

        // Direct fallback checks
        for i in 0..8 {
            let p = PathBuf::from(format!("/dev/hidraw{i}"));
            if p.exists() {
                let uevent_p = PathBuf::from(format!("/sys/class/hidraw/hidraw{i}/device/uevent"));
                if let Ok(content) = std::fs::read_to_string(uevent_p) {
                    let upper = content.to_uppercase();
                    if upper.contains("1462") && upper.contains("1601") {
                        return Some(p);
                    }
                }
            }
        }

        None
    }

    /// Sends a 64-byte feature report using the Linux `HIDIOCSFEATURE` ioctl.
    fn send_feature_report(&self, packet: &[u8; 64]) -> Result<(), BackendError> {
        let file = self.device_file.as_ref().ok_or_else(|| {
            BackendError::Disconnected("Device file handle is not open".to_string())
        })?;

        let fd = file.as_raw_fd();
        let res = unsafe { libc::ioctl(fd, HIDIOCSFEATURE_64, packet.as_ptr()) };

        if res < 0 {
            let err = std::io::Error::last_os_error();
            error!("HIDIOCSFEATURE ioctl failed: {}", err);
            return Err(BackendError::Protocol(format!(
                "HID feature report ioctl failed: {err}"
            )));
        }

        Ok(())
    }

    /// Sends a zone selection packet (Packet ID 1) only if the zone mask has changed.
    ///
    /// Bit 0: Zone 1 (WASD)
    /// Bit 1: Zone 2 (Alpha)
    /// Bit 2: Zone 3 (Nav)
    /// Bit 3: Zone 4 (Numpad)
    fn send_zone_selection(&mut self, zone_mask: u8) -> Result<(), BackendError> {
        if self.current_zone_mask == Some(zone_mask) {
            return Ok(());
        }

        let mut pkt = [0u8; 64];
        pkt[0] = 0x02; // Report ID
        pkt[1] = 0x01; // Packet ID: Zone selection
        pkt[2] = zone_mask;
        self.send_feature_report(&pkt)?;
        self.current_zone_mask = Some(zone_mask);
        Ok(())
    }

    /// Sends the effect configuration packet (Packet ID 2).
    fn send_effect_packet(
        &self,
        effect: EffectType,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError> {
        let mut pkt = [0u8; 64];
        pkt[0] = 0x02; // Report ID
        pkt[1] = 0x02; // Packet ID: Effect config

        // Scale color components by brightness factor
        let b_factor = brightness.as_percentage() as f32 / 100.0;
        let r = ((color.r as f32) * b_factor).round() as u8;
        let g = ((color.g as f32) * b_factor).round() as u8;
        let b = ((color.b as f32) * b_factor).round() as u8;

        // Speed conversion: mapped to centiseconds (300..2400 cs)
        // Lower percentage = slower, more subtle and imperceptible drift
        let speed_val: u16 = match speed.as_percentage() {
            0..=12 => 2400,  // 24 seconds ultra-slow imperceptible morph
            13..=25 => 1800, // 18 seconds slow drift
            26..=40 => 1200, // 12 seconds
            41..=60 => 800,  // 8 seconds (balanced default)
            61..=80 => 500,  // 5 seconds
            _ => 300,        // 3 seconds fast
        };

        let dir_byte: u8 = match direction {
            Direction::Right => 0,
            Direction::Left => 1,
        };

        match effect {
            EffectType::Off => {
                pkt[2] = 0x00; // Disable
                pkt[3] = (speed_val & 0xFF) as u8;
                pkt[4] = ((speed_val >> 8) & 0xFF) as u8;
                pkt[7] = 15;
                pkt[8] = 1;
                pkt[9] = dir_byte;
                // Keyframe 0: [0, 0, 0, 0]
                pkt[10] = 0;
                pkt[11] = 0;
                pkt[12] = 0;
                pkt[13] = 0;
                // Keyframe 1: [100, 0, 0, 0]
                pkt[14] = 100;
                pkt[15] = 0;
                pkt[16] = 0;
                pkt[17] = 0;
            }
            EffectType::Static => {
                pkt[2] = 0x01; // Steady
                pkt[3] = (speed_val & 0xFF) as u8;
                pkt[4] = ((speed_val >> 8) & 0xFF) as u8;
                pkt[7] = 15;
                pkt[8] = 1;
                pkt[9] = dir_byte;
                // Keyframe 0
                pkt[10] = 0;
                pkt[11] = r;
                pkt[12] = g;
                pkt[13] = b;
                // Keyframe 1
                pkt[14] = 100;
                pkt[15] = r;
                pkt[16] = g;
                pkt[17] = b;
            }
            EffectType::Breathing => {
                pkt[2] = 0x02; // Breath
                pkt[3] = (speed_val & 0xFF) as u8;
                pkt[4] = ((speed_val >> 8) & 0xFF) as u8;
                pkt[7] = 15;
                pkt[8] = 1;
                pkt[9] = dir_byte;
                // Keyframe 0: Full color
                pkt[10] = 0;
                pkt[11] = r;
                pkt[12] = g;
                pkt[13] = b;
                // Keyframe 1: Dip to black
                pkt[14] = 50;
                pkt[15] = 0;
                pkt[16] = 0;
                pkt[17] = 0;
                // Keyframe 2: Back to full color
                pkt[18] = 100;
                pkt[19] = r;
                pkt[20] = g;
                pkt[21] = b;
            }
            EffectType::ColorCycle => {
                pkt[2] = 0x03; // Color Cycle
                pkt[3] = (speed_val & 0xFF) as u8;
                pkt[4] = ((speed_val >> 8) & 0xFF) as u8;
                pkt[7] = 15;
                pkt[8] = 1;
                pkt[9] = dir_byte;
                // Seamless 8-point 360° chromatic loop scaled by brightness
                // Loops back from Red to Red with zero jump and zero blackout
                let spectrum = [
                    (0, 255, 0, 0),    // Red
                    (14, 255, 120, 0), // Warm Amber
                    (28, 255, 230, 0), // Yellow
                    (42, 0, 255, 40),  // Emerald Green
                    (57, 0, 210, 255), // Glacial Cyan
                    (71, 10, 80, 255), // Royal Blue
                    (85, 210, 0, 255), // Neon Magenta
                    (100, 255, 0, 0),  // Red (Seamless loop point)
                ];
                let mut offset = 10;
                for (t, cr, cg, cb) in spectrum {
                    pkt[offset] = t;
                    pkt[offset + 1] = ((cr as f32) * b_factor).round() as u8;
                    pkt[offset + 2] = ((cg as f32) * b_factor).round() as u8;
                    pkt[offset + 3] = ((cb as f32) * b_factor).round() as u8;
                    offset += 4;
                }
            }
            EffectType::Wave => {
                pkt[2] = 0x04; // Color Wave
                pkt[3] = (speed_val & 0xFF) as u8;
                pkt[4] = ((speed_val >> 8) & 0xFF) as u8;
                pkt[7] = 15;
                pkt[8] = 1;
                pkt[9] = dir_byte;
                let spectrum = [
                    (0, 255, 0, 0),
                    (17, 255, 255, 0),
                    (33, 0, 255, 0),
                    (50, 0, 255, 255),
                    (67, 0, 0, 255),
                    (83, 255, 0, 255),
                    (100, 255, 0, 0),
                ];
                let mut offset = 10;
                for (t, cr, cg, cb) in spectrum {
                    pkt[offset] = t;
                    pkt[offset + 1] = ((cr as f32) * b_factor).round() as u8;
                    pkt[offset + 2] = ((cg as f32) * b_factor).round() as u8;
                    pkt[offset + 3] = ((cb as f32) * b_factor).round() as u8;
                    offset += 4;
                }
            }
        }

        self.send_feature_report(&pkt)
    }
}

impl Default for NativeHidBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RgbBackend for NativeHidBackend {
    async fn connect(&mut self) -> Result<(), BackendError> {
        let path = match Self::find_hidraw_device() {
            Some(p) => p,
            None => {
                let err_msg = "MSI Mystic Light MS-1565 device not found under /dev/hidraw*";
                self.status = BackendStatus::Error {
                    message: err_msg.to_string(),
                };
                return Err(BackendError::DeviceNotFound(err_msg.to_string()));
            }
        };

        info!("Opening direct HID device node: {}", path.display());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
            .map_err(|e| {
                let msg = format!(
                    "Failed to open {} with read/write permissions: {}",
                    path.display(),
                    e
                );
                warn!("{msg}");
                BackendError::Io(msg)
            })?;

        self.device_path = Some(path);
        self.device_file = Some(file);
        self.status = BackendStatus::Connected {
            device_name: "MSI MysticLight MS-1565 (Native HID)".to_string(),
            protocol_version: 1,
        };

        info!("Native HID direct backend connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BackendError> {
        self.device_file = None;
        self.current_zone_mask = None;
        self.status = BackendStatus::Disconnected {
            reason: "User disconnected".to_string(),
        };
        Ok(())
    }

    fn status(&self) -> BackendStatus {
        self.status.clone()
    }

    async fn detect_device(&mut self) -> Result<Option<DeviceInfo>, BackendError> {
        if self.device_file.is_none() {
            self.connect().await?;
        }

        let mut dev = DeviceInfo::katana_ms1565();
        if let Some(ref p) = self.device_path {
            dev.location = p.display().to_string();
        }
        Ok(Some(dev))
    }

    async fn apply_state(&mut self, state: &RgbState) -> Result<(), BackendError> {
        if self.device_file.is_none() {
            self.connect().await?;
        }

        if state.sync_all_zones {
            // Apply all 4 zones in a single hardware batch
            self.send_zone_selection(0x0F)?;
            self.send_effect_packet(
                state.global_effect,
                state.global_color,
                state.global_brightness,
                state.global_speed,
                state.global_direction,
            )?;
        } else {
            // Apply each zone individually
            for zone_id in [ZoneId::Zone1, ZoneId::Zone2, ZoneId::Zone3, ZoneId::Zone4] {
                let zone_config = state.zone(zone_id);
                let mask = match zone_id {
                    ZoneId::Zone1 => 0x01,
                    ZoneId::Zone2 => 0x02,
                    ZoneId::Zone3 => 0x04,
                    ZoneId::Zone4 => 0x08,
                };
                self.send_zone_selection(mask)?;
                self.send_effect_packet(
                    zone_config.effect,
                    zone_config.color,
                    zone_config.brightness,
                    zone_config.speed,
                    zone_config.direction,
                )?;
            }
        }

        Ok(())
    }

    async fn apply_zone(
        &mut self,
        zone_id: ZoneId,
        config: &ZoneConfig,
    ) -> Result<(), BackendError> {
        if self.device_file.is_none() {
            self.connect().await?;
        }

        let mask = match zone_id {
            ZoneId::Zone1 => 0x01,
            ZoneId::Zone2 => 0x02,
            ZoneId::Zone3 => 0x04,
            ZoneId::Zone4 => 0x08,
        };

        self.send_zone_selection(mask)?;
        self.send_effect_packet(
            config.effect,
            config.color,
            config.brightness,
            config.speed,
            config.direction,
        )?;

        Ok(())
    }

    async fn apply_global_mode(
        &mut self,
        effect: EffectType,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError> {
        if self.device_file.is_none() {
            self.connect().await?;
        }

        self.send_zone_selection(0x0F)?;
        self.send_effect_packet(effect, color, brightness, speed, direction)?;
        Ok(())
    }

    async fn save_to_flash(&mut self) -> Result<(), BackendError> {
        if self.device_file.is_none() {
            self.connect().await?;
        }

        let mut pkt = [0u8; 64];
        pkt[0] = 0x02; // Report ID
        pkt[1] = 0xA0; // Packet ID: Save to Flash
        self.send_feature_report(&pkt)?;
        debug!("MSI keyboard configuration successfully saved to hardware flash");
        Ok(())
    }
}
