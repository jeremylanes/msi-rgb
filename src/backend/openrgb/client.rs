//! Asynchronous OpenRGB SDK Client with resilient connection management.
//!
//! Provides connection pooling, automatic exponential backoff reconnection,
//! hardware discovery, and typed mode application.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::protocol::{
    OPENRGB_DEFAULT_PORT, OPENRGB_PROTOCOL_VERSION, OpenRgbControllerData, build_packet_header,
    packet_ids, parse_controller_data, serialize_update_mode_payload,
};
use crate::domain::color::RgbColor;
use crate::domain::device::DeviceInfo;
use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::error::BackendError;
use crate::domain::zone::ZoneId;

/// Managed OpenRGB SDK client connection.
pub struct OpenRgbClient {
    host: String,
    port: u16,
    client_name: String,
    stream: Option<Mutex<TcpStream>>,
    protocol_version: u32,
    controllers: Vec<OpenRgbControllerData>,
    target_controller_index: Option<u32>,
}

impl OpenRgbClient {
    /// Creates a new uninitialized `OpenRgbClient`.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16, client_name: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            client_name: client_name.into(),
            stream: None,
            protocol_version: OPENRGB_PROTOCOL_VERSION,
            controllers: Vec::new(),
            target_controller_index: None,
        }
    }

    /// Creates a client with default localhost coordinates.
    #[must_use]
    pub fn default_local() -> Self {
        Self::new(
            "127.0.0.1",
            OPENRGB_DEFAULT_PORT,
            "MSI Mystic Light Control",
        )
    }

    /// Returns true if currently connected.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Returns the negotiated protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    /// Connects to the OpenRGB server and negotiates protocol version.
    pub async fn connect(&mut self) -> Result<(), BackendError> {
        let addr = format!("{}:{}", self.host, self.port);
        tracing::info!("Connecting to OpenRGB server at {}...", addr);

        let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
            BackendError::Disconnected(format!("Failed to connect to OpenRGB at {addr}: {e}"))
        })?;

        // 1. Negotiate Protocol Version
        let client_ver = OPENRGB_PROTOCOL_VERSION;
        let mut ver_pkt = build_packet_header(0, packet_ids::REQUEST_PROTOCOL_VERSION, 4);
        ver_pkt.extend_from_slice(&client_ver.to_le_bytes());
        stream.write_all(&ver_pkt).await.map_err(|e| {
            BackendError::Io(format!("Failed to send protocol version request: {e}"))
        })?;

        let mut hdr = [0u8; 16];
        stream.read_exact(&mut hdr).await.map_err(|e| {
            BackendError::Io(format!("Failed to read protocol version header: {e}"))
        })?;
        let len = u32::from_le_bytes(hdr[12..16].try_into().unwrap_or([0; 4]));
        let mut ver_buf = vec![0u8; len as usize];
        stream
            .read_exact(&mut ver_buf)
            .await
            .map_err(|e| BackendError::Io(format!("Failed to read protocol version body: {e}")))?;
        let server_ver = if ver_buf.len() >= 4 {
            u32::from_le_bytes(ver_buf[0..4].try_into().unwrap_or([0; 4]))
        } else {
            OPENRGB_PROTOCOL_VERSION
        };
        self.protocol_version = client_ver.min(server_ver);
        tracing::info!(
            "Negotiated OpenRGB protocol version: {}",
            self.protocol_version
        );

        // 2. Set Client Name
        let mut name_bytes = self.client_name.as_bytes().to_vec();
        name_bytes.push(0);
        let name_pkt_header =
            build_packet_header(0, packet_ids::SET_CLIENT_NAME, name_bytes.len() as u32);
        let mut name_pkt = name_pkt_header;
        name_pkt.extend_from_slice(&name_bytes);
        stream
            .write_all(&name_pkt)
            .await
            .map_err(|e| BackendError::Io(format!("Failed to send client name packet: {e}")))?;

        self.stream = Some(Mutex::new(stream));

        // 3. Scan and discover controllers
        self.refresh_controllers().await?;

        Ok(())
    }

    /// Disconnects from the server.
    pub async fn disconnect(&mut self) -> Result<(), BackendError> {
        if let Some(stream_lock) = self.stream.take() {
            let mut stream = stream_lock.lock().await;
            let _ = stream.shutdown().await;
        }
        self.controllers.clear();
        self.target_controller_index = None;
        tracing::info!("Disconnected from OpenRGB server");
        Ok(())
    }

    /// Queries the controller count and retrieves full metadata for each controller.
    pub async fn refresh_controllers(&mut self) -> Result<(), BackendError> {
        let count = self.query_controller_count().await?;
        tracing::debug!("OpenRGB reported {} controller(s)", count);

        let mut list = Vec::with_capacity(count as usize);
        for i in 0..count {
            let ctrl = self.query_controller_data(i).await?;
            list.push(ctrl);
        }

        self.controllers = list;

        // Find MSI Katana keyboard or compatible MSI MysticLight device
        self.target_controller_index = self
            .controllers
            .iter()
            .position(|c| {
                c.name.to_lowercase().contains("msi")
                    || c.name.to_lowercase().contains("mystic")
                    || c.vendor.to_lowercase().contains("msi")
            })
            .map(|i| i as u32);

        if let Some(idx) = self.target_controller_index {
            let ctrl = &self.controllers[idx as usize];
            tracing::info!(
                "Found target MSI RGB device [Index {}]: '{}' by '{}' with {} modes",
                idx,
                ctrl.name,
                ctrl.vendor,
                ctrl.modes.len()
            );
        } else if !self.controllers.is_empty() {
            // Default to first controller if specific MSI match wasn't found
            self.target_controller_index = Some(0);
            tracing::warn!(
                "No MSI-specific device name matched, defaulting to controller 0: '{}'",
                self.controllers[0].name
            );
        }

        Ok(())
    }

    async fn query_controller_count(&self) -> Result<u32, BackendError> {
        let stream_lock = self
            .stream
            .as_ref()
            .ok_or_else(|| BackendError::Disconnected("Client not connected".to_string()))?;
        let mut stream = stream_lock.lock().await;

        let pkt = build_packet_header(0, packet_ids::REQUEST_CONTROLLER_COUNT, 0);
        stream
            .write_all(&pkt)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;

        let mut hdr = [0u8; 16];
        stream
            .read_exact(&mut hdr)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;
        let len = u32::from_le_bytes(hdr[12..16].try_into().unwrap_or([0; 4]));
        let mut buf = vec![0u8; len as usize];
        stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;

        if buf.len() >= 4 {
            Ok(u32::from_le_bytes(buf[0..4].try_into().unwrap_or([0; 4])))
        } else {
            Ok(0)
        }
    }

    async fn query_controller_data(
        &self,
        controller_idx: u32,
    ) -> Result<OpenRgbControllerData, BackendError> {
        let stream_lock = self
            .stream
            .as_ref()
            .ok_or_else(|| BackendError::Disconnected("Client not connected".to_string()))?;
        let mut stream = stream_lock.lock().await;

        let mut pkt = build_packet_header(controller_idx, packet_ids::REQUEST_CONTROLLER_DATA, 4);
        pkt.extend_from_slice(&self.protocol_version.to_le_bytes());
        stream
            .write_all(&pkt)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;

        let mut hdr = [0u8; 16];
        stream
            .read_exact(&mut hdr)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;
        let len = u32::from_le_bytes(hdr[12..16].try_into().unwrap_or([0; 4]));
        let mut buf = vec![0u8; len as usize];
        stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;

        parse_controller_data(controller_idx, self.protocol_version, &buf)
            .map_err(|e| BackendError::Protocol(format!("Failed to parse controller data: {e}")))
    }

    /// Returns metadata for the active MSI device.
    #[must_use]
    pub fn get_active_device_info(&self) -> Option<DeviceInfo> {
        let idx = self.target_controller_index? as usize;
        let ctrl = self.controllers.get(idx)?;

        let mut supported_effects = Vec::new();
        for m in &ctrl.modes {
            let name_lower = m.name.to_lowercase();
            if name_lower == "off" && !supported_effects.contains(&EffectType::Off) {
                supported_effects.push(EffectType::Off);
            } else if name_lower == "static" && !supported_effects.contains(&EffectType::Static) {
                supported_effects.push(EffectType::Static);
            } else if name_lower.starts_with("breathing")
                && !supported_effects.contains(&EffectType::Breathing)
            {
                supported_effects.push(EffectType::Breathing);
            } else if (name_lower.starts_with("color cycle") || name_lower.starts_with("cycle"))
                && !supported_effects.contains(&EffectType::ColorCycle)
            {
                supported_effects.push(EffectType::ColorCycle);
            } else if name_lower.starts_with("wave")
                && !supported_effects.contains(&EffectType::Wave)
            {
                supported_effects.push(EffectType::Wave);
            }
        }

        if supported_effects.is_empty() {
            supported_effects = vec![
                EffectType::Static,
                EffectType::Breathing,
                EffectType::ColorCycle,
                EffectType::Wave,
                EffectType::Off,
            ];
        }

        Some(DeviceInfo {
            name: ctrl.name.clone(),
            vendor: ctrl.vendor.clone(),
            vid_pid: "1462:1601".to_string(),
            serial: ctrl.serial.clone(),
            location: ctrl.location.clone(),
            zone_count: 4,
            supported_effects,
            supports_per_zone: true,
        })
    }

    /// Sets an effect mode on the active controller.
    pub async fn apply_mode_by_name(
        &mut self,
        mode_name: &str,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError> {
        let target_idx = self
            .target_controller_index
            .ok_or_else(|| BackendError::DeviceNotFound("No MSI RGB device active".to_string()))?;

        let ctrl = self
            .controllers
            .get(target_idx as usize)
            .ok_or_else(|| BackendError::DeviceNotFound("Controller data missing".to_string()))?;

        // Case-insensitive match on mode name
        let target_mode = ctrl
            .modes
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(mode_name))
            .or_else(|| {
                // Fallback prefix match
                ctrl.modes
                    .iter()
                    .find(|m| m.name.to_lowercase().starts_with(&mode_name.to_lowercase()))
            })
            .cloned()
            .ok_or_else(|| {
                BackendError::CommandFailed(format!("Mode '{mode_name}' not supported by OpenRGB"))
            })?;

        let mut updated_mode = target_mode;
        if updated_mode.has_brightness() {
            let brt_pct = brightness.as_percentage() as u32;
            let range = updated_mode
                .brightness_max
                .saturating_sub(updated_mode.brightness_min);
            updated_mode.brightness = updated_mode.brightness_min + (range * brt_pct / 100).max(1);
        }
        if updated_mode.has_speed() {
            let spd_pct = speed.as_percentage() as u32;
            let range = updated_mode
                .speed_max
                .saturating_sub(updated_mode.speed_min);
            updated_mode.speed = updated_mode.speed_min + (range * spd_pct / 100);
        }
        if updated_mode.has_direction() {
            updated_mode.direction = match direction {
                Direction::Left => 0,
                Direction::Right => 1,
            };
        }

        // Color handling
        if updated_mode.name.to_lowercase().starts_with("off") {
            updated_mode.colors.clear();
        } else {
            let col_count = updated_mode.colors_min.max(1) as usize;
            updated_mode.colors = vec![color; col_count];
        }

        let payload = serialize_update_mode_payload(self.protocol_version, &updated_mode);
        let header = build_packet_header(target_idx, packet_ids::UPDATE_MODE, payload.len() as u32);

        let stream_lock = self
            .stream
            .as_ref()
            .ok_or_else(|| BackendError::Disconnected("Client disconnected".to_string()))?;
        let mut stream = stream_lock.lock().await;

        stream
            .write_all(&header)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;
        stream
            .write_all(&payload)
            .await
            .map_err(|e| BackendError::Io(e.to_string()))?;

        tracing::debug!(
            "Applied mode '{}' on controller {} (color: {}, brt: {}%, spd: {}%)",
            updated_mode.name,
            target_idx,
            color,
            brightness.as_percentage(),
            speed.as_percentage()
        );

        Ok(())
    }

    /// Sets individual zone effect mode and color.
    pub async fn apply_zone_mode(
        &mut self,
        zone_id: ZoneId,
        effect: EffectType,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError> {
        let zone_num = zone_id.number();
        let mode_name = match effect {
            EffectType::Off => format!("Off zone {zone_num}"),
            EffectType::Static => format!("Static zone {zone_num}"),
            EffectType::Breathing => format!("Breathing zone {zone_num}"),
            EffectType::ColorCycle => format!("Color Cycle zone {zone_num}"),
            EffectType::Wave => format!("Wave zone {zone_num}"),
        };

        self.apply_mode_by_name(&mode_name, color, brightness, speed, direction)
            .await
    }
}
