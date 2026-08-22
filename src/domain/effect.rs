//! RGB Effects domain model.
//!
//! Provides typed definitions for lighting modes (Static, Breathing, Wave, Color Cycle, Off),
//! animation speed, brightness levels, and movement direction.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::error::DomainError;

/// Core lighting effect modes supported by MSI MysticLight keyboards.
///
/// Example:
/// ```
/// use msi_rgb::domain::effect::EffectType;
///
/// let effect = EffectType::Static;
/// assert_eq!(effect.display_name(), "Static");
/// assert!(effect.supports_color());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EffectType {
    /// Keyboard LEDs turned off.
    Off,
    /// Solid, static color illumination.
    #[default]
    Static,
    /// Gentle pulsation between off and the selected color.
    Breathing,
    /// Continuous smooth rainbow spectrum cycling across all colors.
    ColorCycle,
    /// Dynamic color wave traveling across keyboard zones.
    Wave,
}

impl EffectType {
    /// Returns the human-readable English name of the effect.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Static => "Static",
            Self::Breathing => "Breathing",
            Self::ColorCycle => "Color Cycle",
            Self::Wave => "Wave",
        }
    }

    /// Returns a short descriptive summary of the effect.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Off => "Turn off all keyboard zone lighting",
            Self::Static => "Solid continuous illumination",
            Self::Breathing => "Smooth pulsing breathing glow",
            Self::ColorCycle => "Continuous chromatic spectrum cycle",
            Self::Wave => "Animated moving color wave",
        }
    }

    /// Whether this effect requires/supports a custom target color.
    #[must_use]
    pub const fn supports_color(self) -> bool {
        match self {
            Self::Off => false,
            Self::Static | Self::Breathing | Self::Wave | Self::ColorCycle => true,
        }
    }

    /// Whether this effect supports animation speed configuration.
    #[must_use]
    pub const fn supports_speed(self) -> bool {
        match self {
            Self::Off | Self::Static => false,
            Self::Breathing | Self::ColorCycle | Self::Wave => true,
        }
    }

    /// Whether this effect supports brightness control.
    #[must_use]
    pub const fn supports_brightness(self) -> bool {
        match self {
            Self::Off => false,
            Self::Static | Self::Breathing | Self::ColorCycle | Self::Wave => true,
        }
    }

    /// Whether this effect supports wave direction (Left / Right).
    #[must_use]
    pub const fn supports_direction(self) -> bool {
        matches!(self, Self::Wave)
    }

    /// Returns all available standard effect variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Static,
            Self::Breathing,
            Self::ColorCycle,
            Self::Wave,
            Self::Off,
        ]
    }
}

impl fmt::Display for EffectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for EffectType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "off" => Ok(Self::Off),
            "static" => Ok(Self::Static),
            "breathing" | "breath" => Ok(Self::Breathing),
            "color cycle" | "colorcycle" | "cycle" | "rainbow" => Ok(Self::ColorCycle),
            "wave" => Ok(Self::Wave),
            other => Err(DomainError::UnsupportedEffect(other.to_string())),
        }
    }
}

/// Wave animation traveling direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Direction {
    /// Leftwards wave.
    #[default]
    Left,
    /// Rightwards wave.
    Right,
}

impl Direction {
    /// Returns the display label for direction.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

/// Brightness percentage (0..=100%).
///
/// Example:
/// ```
/// use msi_rgb::domain::effect::Brightness;
///
/// let b = Brightness::new(80).unwrap();
/// assert_eq!(b.as_percentage(), 80);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Brightness(u8);

impl Brightness {
    /// Minimum brightness (0%).
    pub const MIN: Self = Self(0);
    /// Maximum brightness (100%).
    pub const MAX: Self = Self(100);
    /// Default brightness (100%).
    pub const DEFAULT: Self = Self(100);

    /// Constructs a `Brightness` from a percentage value (0..=100).
    pub fn new(val: u8) -> Result<Self, DomainError> {
        if val <= 100 {
            Ok(Self(val))
        } else {
            Err(DomainError::InvalidBrightness(val))
        }
    }

    /// Constructs a clamped `Brightness` ensuring value is within 0..=100.
    #[must_use]
    pub const fn clamped(val: u8) -> Self {
        if val > 100 { Self(100) } else { Self(val) }
    }

    /// Returns the numeric percentage value (0..=100).
    #[must_use]
    pub const fn as_percentage(self) -> u8 {
        self.0
    }

    /// Converts to floating point ratio (0.0..=1.0).
    #[must_use]
    pub fn as_ratio(self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl Default for Brightness {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Animation speed percentage (0..=100%).
///
/// Example:
/// ```
/// use msi_rgb::domain::effect::Speed;
///
/// let s = Speed::new(50).unwrap();
/// assert_eq!(s.as_percentage(), 50);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Speed(u8);

impl Speed {
    /// Minimum speed (0%).
    pub const MIN: Self = Self(0);
    /// Maximum speed (100%).
    pub const MAX: Self = Self(100);
    /// Default medium speed (50%).
    pub const DEFAULT: Self = Self(50);

    /// Constructs a `Speed` from a percentage value (0..=100).
    pub fn new(val: u8) -> Result<Self, DomainError> {
        if val <= 100 {
            Ok(Self(val))
        } else {
            Err(DomainError::InvalidSpeed(val))
        }
    }

    /// Constructs a clamped `Speed` ensuring value is within 0..=100.
    #[must_use]
    pub const fn clamped(val: u8) -> Self {
        if val > 100 { Self(100) } else { Self(val) }
    }

    /// Returns the numeric percentage value (0..=100).
    #[must_use]
    pub const fn as_percentage(self) -> u8 {
        self.0
    }

    /// Converts to floating point ratio (0.0..=1.0).
    #[must_use]
    pub fn as_ratio(self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::DEFAULT
    }
}
