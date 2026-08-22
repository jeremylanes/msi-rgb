//! 4-Zone keyboard domain model.
//!
//! Provides structures and types for managing the 4 discrete RGB zones
//! present on the MSI Katana 15 HX keyboard layout.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::color::RgbColor;
use super::effect::{Brightness, Direction, EffectType, Speed};
use super::error::DomainError;

/// Identifier for one of the 4 discrete keyboard RGB zones.
///
/// Example:
/// ```
/// use msi_rgb::domain::zone::ZoneId;
///
/// let z1 = ZoneId::Zone1;
/// assert_eq!(z1.number(), 1);
/// assert_eq!(z1.index(), 0);
/// assert_eq!(z1.display_name(), "Zone 1 (Left / WASD)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ZoneId {
    /// Zone 1: Left section (Esc, Tab, Caps, Shift, Ctrl, WASD keys).
    Zone1,
    /// Zone 2: Middle-left section (T, Y, G, H, B, Space left).
    Zone2,
    /// Zone 3: Middle-right section (U, I, O, P, J, K, L, Enter, Arrows).
    Zone3,
    /// Zone 4: Right section (Numeric Keypad).
    Zone4,
}

impl ZoneId {
    /// All 4 keyboard zones in order.
    pub const ALL: [Self; 4] = [Self::Zone1, Self::Zone2, Self::Zone3, Self::Zone4];

    /// Returns 1-based zone number (1..=4).
    #[must_use]
    pub const fn number(self) -> usize {
        match self {
            Self::Zone1 => 1,
            Self::Zone2 => 2,
            Self::Zone3 => 3,
            Self::Zone4 => 4,
        }
    }

    /// Returns 0-based zone index (0..=3).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Zone1 => 0,
            Self::Zone2 => 1,
            Self::Zone3 => 2,
            Self::Zone4 => 3,
        }
    }

    /// Creates a `ZoneId` from a 1-based number (1..=4).
    pub fn from_number(num: usize) -> Result<Self, DomainError> {
        match num {
            1 => Ok(Self::Zone1),
            2 => Ok(Self::Zone2),
            3 => Ok(Self::Zone3),
            4 => Ok(Self::Zone4),
            other => Err(DomainError::InvalidZoneIndex(other)),
        }
    }

    /// Creates a `ZoneId` from a 0-based index (0..=3).
    pub fn from_index(idx: usize) -> Result<Self, DomainError> {
        match idx {
            0 => Ok(Self::Zone1),
            1 => Ok(Self::Zone2),
            2 => Ok(Self::Zone3),
            3 => Ok(Self::Zone4),
            other => Err(DomainError::InvalidZoneIndex(other + 1)),
        }
    }

    /// Human-friendly display label with physical keyboard region description.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Zone1 => "Zone 1 (Left / WASD)",
            Self::Zone2 => "Zone 2 (Center Left)",
            Self::Zone3 => "Zone 3 (Center Right)",
            Self::Zone4 => "Zone 4 (Numpad)",
        }
    }

    /// Short label (e.g., "Zone 1").
    #[must_use]
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::Zone1 => "Zone 1",
            Self::Zone2 => "Zone 2",
            Self::Zone3 => "Zone 3",
            Self::Zone4 => "Zone 4",
        }
    }
}

impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

/// Configuration settings for a single RGB zone.
///
/// Example:
/// ```
/// use msi_rgb::domain::zone::{ZoneConfig, ZoneId};
/// use msi_rgb::domain::color::RgbColor;
/// use msi_rgb::domain::effect::EffectType;
///
/// let config = ZoneConfig::new(ZoneId::Zone1, EffectType::Static, RgbColor::MSI_RED);
/// assert!(config.enabled);
/// assert_eq!(config.color, RgbColor::MSI_RED);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneConfig {
    /// Targeted zone identifier.
    pub id: ZoneId,
    /// Whether illumination is enabled for this zone.
    pub enabled: bool,
    /// Active effect type.
    pub effect: EffectType,
    /// Primary color.
    pub color: RgbColor,
    /// Brightness level (0..=100%).
    pub brightness: Brightness,
    /// Speed level (0..=100%).
    pub speed: Speed,
    /// Direction (for Wave effect).
    pub direction: Direction,
}

impl ZoneConfig {
    /// Creates a new `ZoneConfig` with default brightness (100%) and speed (50%).
    #[must_use]
    pub fn new(id: ZoneId, effect: EffectType, color: RgbColor) -> Self {
        Self {
            id,
            enabled: effect != EffectType::Off,
            effect,
            color,
            brightness: Brightness::DEFAULT,
            speed: Speed::DEFAULT,
            direction: Direction::Left,
        }
    }

    /// Returns the effective color for UI rendering (black if disabled or effect is Off).
    #[must_use]
    pub fn effective_color(&self) -> RgbColor {
        if !self.enabled || self.effect == EffectType::Off {
            RgbColor::BLACK
        } else {
            self.color
        }
    }
}

impl Default for ZoneConfig {
    fn default() -> Self {
        Self::new(ZoneId::Zone1, EffectType::Static, RgbColor::MSI_RED)
    }
}
