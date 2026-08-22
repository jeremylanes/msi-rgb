//! Device domain model and whole-keyboard RGB state.
//!
//! Provides device metadata, supported capability descriptions,
//! and complete 4-zone state structures.

use serde::{Deserialize, Serialize};

use super::color::RgbColor;
use super::effect::{Brightness, Direction, EffectType, Speed};
use super::zone::{ZoneConfig, ZoneId};

/// Metadata and identification of detected MSI RGB hardware.
///
/// Example:
/// ```
/// use msi_rgb::domain::device::DeviceInfo;
///
/// let dev = DeviceInfo::katana_ms1565();
/// assert_eq!(dev.name, "MSI MysticLight MS-1565");
/// assert_eq!(dev.zone_count, 4);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device human name.
    pub name: String,
    /// Hardware vendor name.
    pub vendor: String,
    /// USB Vendor ID : Product ID (e.g. "1462:1601").
    pub vid_pid: String,
    /// Serial number if provided by hardware.
    pub serial: String,
    /// Physical location or HID interface.
    pub location: String,
    /// Number of independently controllable RGB zones.
    pub zone_count: usize,
    /// List of supported effect modes.
    pub supported_effects: Vec<EffectType>,
    /// Whether independent per-zone control is supported.
    pub supports_per_zone: bool,
}

impl DeviceInfo {
    /// Constructs default metadata for the MSI Katana 15 HX (MS-1565).
    #[must_use]
    pub fn katana_ms1565() -> Self {
        Self {
            name: "MSI MysticLight MS-1565".to_string(),
            vendor: "MSI".to_string(),
            vid_pid: "1462:1601".to_string(),
            serial: "3178696BC000".to_string(),
            location: "HID Keyboard".to_string(),
            zone_count: 4,
            supported_effects: vec![
                EffectType::Static,
                EffectType::Breathing,
                EffectType::ColorCycle,
                EffectType::Wave,
                EffectType::Off,
            ],
            supports_per_zone: true,
        }
    }
}

/// Complete RGB lighting state of the keyboard.
///
/// Holds individual configurations for all 4 zones, as well as
/// global synchronization settings.
///
/// Example:
/// ```
/// use msi_rgb::domain::device::RgbState;
/// use msi_rgb::domain::color::RgbColor;
/// use msi_rgb::domain::effect::EffectType;
///
/// let mut state = RgbState::new_uniform(EffectType::Static, RgbColor::CYAN);
/// assert!(state.sync_all_zones);
/// assert_eq!(state.zone_1.color, RgbColor::CYAN);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbState {
    /// Whether all 4 zones are synchronized together to the same effect/color.
    pub sync_all_zones: bool,
    /// Global effect type when `sync_all_zones` is true.
    pub global_effect: EffectType,
    /// Global primary color when `sync_all_zones` is true.
    pub global_color: RgbColor,
    /// Global brightness percentage when `sync_all_zones` is true.
    pub global_brightness: Brightness,
    /// Global animation speed when `sync_all_zones` is true.
    pub global_speed: Speed,
    /// Global wave animation direction when `sync_all_zones` is true.
    pub global_direction: Direction,

    /// Zone 1 settings (Left / WASD).
    pub zone_1: ZoneConfig,
    /// Zone 2 settings (Center Left).
    pub zone_2: ZoneConfig,
    /// Zone 3 settings (Center Right).
    pub zone_3: ZoneConfig,
    /// Zone 4 settings (Numpad).
    pub zone_4: ZoneConfig,
}

impl RgbState {
    /// Creates a uniform RGB state with all zones set to the same effect and color.
    #[must_use]
    pub fn new_uniform(effect: EffectType, color: RgbColor) -> Self {
        Self {
            sync_all_zones: true,
            global_effect: effect,
            global_color: color,
            global_brightness: Brightness::DEFAULT,
            global_speed: Speed::DEFAULT,
            global_direction: Direction::Left,
            zone_1: ZoneConfig::new(ZoneId::Zone1, effect, color),
            zone_2: ZoneConfig::new(ZoneId::Zone2, effect, color),
            zone_3: ZoneConfig::new(ZoneId::Zone3, effect, color),
            zone_4: ZoneConfig::new(ZoneId::Zone4, effect, color),
        }
    }

    /// Gets a reference to a specific zone configuration.
    #[must_use]
    pub fn zone(&self, id: ZoneId) -> &ZoneConfig {
        match id {
            ZoneId::Zone1 => &self.zone_1,
            ZoneId::Zone2 => &self.zone_2,
            ZoneId::Zone3 => &self.zone_3,
            ZoneId::Zone4 => &self.zone_4,
        }
    }

    /// Gets a mutable reference to a specific zone configuration.
    #[must_use]
    pub fn zone_mut(&mut self, id: ZoneId) -> &mut ZoneConfig {
        match id {
            ZoneId::Zone1 => &mut self.zone_1,
            ZoneId::Zone2 => &mut self.zone_2,
            ZoneId::Zone3 => &mut self.zone_3,
            ZoneId::Zone4 => &mut self.zone_4,
        }
    }

    /// Synchronizes all 4 zones to match current global settings.
    pub fn sync_zones_from_global(&mut self) {
        let eff = self.global_effect;
        let col = self.global_color;
        let brt = self.global_brightness;
        let spd = self.global_speed;
        let dir = self.global_direction;

        for z in [
            &mut self.zone_1,
            &mut self.zone_2,
            &mut self.zone_3,
            &mut self.zone_4,
        ] {
            z.effect = eff;
            z.color = col;
            z.brightness = brt;
            z.speed = spd;
            z.direction = dir;
            z.enabled = eff != EffectType::Off;
        }
    }

    /// Returns an array of all 4 zone configs.
    #[must_use]
    pub fn all_zones(&self) -> [&ZoneConfig; 4] {
        [&self.zone_1, &self.zone_2, &self.zone_3, &self.zone_4]
    }
}

impl Default for RgbState {
    fn default() -> Self {
        Self::new_uniform(EffectType::Static, RgbColor::MSI_RED)
    }
}
