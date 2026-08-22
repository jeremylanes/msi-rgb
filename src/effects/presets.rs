//! Color and effect presets for fast application.
//!
//! Provides curated color pairings and preset templates matching
//! MSI gaming hardware aesthetics.

use crate::domain::color::RgbColor;

/// A quick color preset entry with name and color value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPreset {
    /// Human-friendly preset label.
    pub name: &'static str,
    /// Associated RGB color.
    pub color: RgbColor,
}

impl ColorPreset {
    /// Standard palette presets.
    pub const PRESETS: &'static [Self] = &[
        Self {
            name: "MSI Dragon Red",
            color: RgbColor::MSI_RED,
        },
        Self {
            name: "Cyber Cyan",
            color: RgbColor::CYAN,
        },
        Self {
            name: "Neon Magenta",
            color: RgbColor::MAGENTA,
        },
        Self {
            name: "Emerald Matrix",
            color: RgbColor::GREEN,
        },
        Self {
            name: "Ocean Deep Blue",
            color: RgbColor::BLUE,
        },
        Self {
            name: "Warm Sunset Amber",
            color: RgbColor::AMBER,
        },
        Self {
            name: "Royal Purple",
            color: RgbColor::PURPLE,
        },
        Self {
            name: "Golden Yellow",
            color: RgbColor::YELLOW,
        },
        Self {
            name: "Pure White",
            color: RgbColor::WHITE,
        },
    ];
}
