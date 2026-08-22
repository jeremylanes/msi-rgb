//! RGB Color domain model and utilities.
//!
//! Provides a type-safe `RgbColor` structure supporting RGB, HSV, Hex string
//! serialization, and predefined MSI gaming color presets.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::error::DomainError;

/// Represents a 24-bit sRGB color (8 bits per channel).
///
/// Example:
/// ```
/// use msi_rgb::domain::color::RgbColor;
///
/// let red = RgbColor::new(255, 0, 0);
/// assert_eq!(red.to_hex(), "#FF0000");
///
/// let parsed: RgbColor = "#00FF88".parse().unwrap();
/// assert_eq!(parsed, RgbColor::new(0, 255, 136));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RgbColor {
    /// Red channel (0..=255).
    pub r: u8,
    /// Green channel (0..=255).
    pub g: u8,
    /// Blue channel (0..=255).
    pub b: u8,
}

impl RgbColor {
    /// Creates a new `RgbColor` with given red, green, and blue components.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Pure Black / Off.
    pub const BLACK: Self = Self::new(0, 0, 0);
    /// Pure White.
    pub const WHITE: Self = Self::new(255, 255, 255);
    /// MSI Signature Dragon Red.
    pub const MSI_RED: Self = Self::new(255, 20, 40);
    /// Cyberpunk Neon Cyan.
    pub const CYAN: Self = Self::new(0, 240, 255);
    /// Electric Magenta.
    pub const MAGENTA: Self = Self::new(255, 0, 180);
    /// Emerald Green.
    pub const GREEN: Self = Self::new(0, 255, 100);
    /// Deep Ocean Blue.
    pub const BLUE: Self = Self::new(0, 120, 255);
    /// Warm Amber / Sunset Orange.
    pub const AMBER: Self = Self::new(255, 140, 0);
    /// Royal Purple.
    pub const PURPLE: Self = Self::new(160, 32, 240);
    /// Golden Yellow.
    pub const YELLOW: Self = Self::new(255, 220, 0);

    /// Formats the color as a standard hexadecimal color string `#RRGGBB`.
    #[must_use]
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Converts normalized RGBA floats (0.0..=1.0) into `RgbColor`.
    #[must_use]
    pub fn from_f64(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            g: (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            b: (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }

    /// Converts into normalized RGB floats (0.0..=1.0) for GTK / Cairo rendering.
    #[must_use]
    pub fn to_f64(self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    /// Converts perceived relative luminance (0.0..=1.0).
    #[must_use]
    pub fn relative_luminance(self) -> f64 {
        0.2126 * (self.r as f64 / 255.0)
            + 0.7152 * (self.g as f64 / 255.0)
            + 0.0722 * (self.b as f64 / 255.0)
    }

    /// Returns true if this color is completely black (off).
    #[must_use]
    pub const fn is_off(self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    /// Converts RGB (0..=255) to HSV representation:
    /// Hue in degrees `[0.0, 360.0)`, Saturation `[0.0, 1.0]`, Value `[0.0, 1.0]`.
    ///
    /// Example:
    /// ```
    /// use msi_rgb::domain::color::RgbColor;
    ///
    /// let red = RgbColor::new(255, 0, 0);
    /// let (h, s, v) = red.to_hsv();
    /// assert!((h - 0.0).abs() < 1e-4);
    /// assert!((s - 1.0).abs() < 1e-4);
    /// assert!((v - 1.0).abs() < 1e-4);
    /// ```
    #[must_use]
    pub fn to_hsv(self) -> (f64, f64, f64) {
        let (r, g, b) = self.to_f64();
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let v = max;
        let s = if max > 0.0 { delta / max } else { 0.0 };

        let h = if delta == 0.0 {
            0.0
        } else if (max - r).abs() < f64::EPSILON {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if (max - g).abs() < f64::EPSILON {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        (h.rem_euclid(360.0), s.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }

    /// Converts HSV representation to `RgbColor`.
    /// `h` in degrees `[0.0, 360.0)`, `s` in `[0.0, 1.0]`, `v` in `[0.0, 1.0]`.
    ///
    /// Example:
    /// ```
    /// use msi_rgb::domain::color::RgbColor;
    ///
    /// let color = RgbColor::from_hsv(180.0, 1.0, 1.0);
    /// assert_eq!(color, RgbColor::new(0, 255, 255));
    /// ```
    #[must_use]
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let h_norm = h.rem_euclid(360.0);
        let s_clamped = s.clamp(0.0, 1.0);
        let v_clamped = v.clamp(0.0, 1.0);

        let c = v_clamped * s_clamped;
        let x = c * (1.0 - ((h_norm / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = v_clamped - c;

        let (r_prime, g_prime, b_prime) = if h_norm < 60.0 {
            (c, x, 0.0)
        } else if h_norm < 120.0 {
            (x, c, 0.0)
        } else if h_norm < 180.0 {
            (0.0, c, x)
        } else if h_norm < 240.0 {
            (0.0, x, c)
        } else if h_norm < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            g: ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            b: ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        }
    }
}

impl Default for RgbColor {
    fn default() -> Self {
        Self::MSI_RED
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl FromStr for RgbColor {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);

        if hex.len() != 6 {
            return Err(DomainError::InvalidColorHex(s.to_string()));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| DomainError::InvalidColorHex(s.to_string()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| DomainError::InvalidColorHex(s.to_string()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| DomainError::InvalidColorHex(s.to_string()))?;

        Ok(Self::new(r, g, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_hex_roundtrip() {
        let color = RgbColor::new(18, 52, 86);
        let hex = color.to_hex();
        assert_eq!(hex, "#123456");
        let parsed: RgbColor = hex.parse().unwrap();
        assert_eq!(parsed, color);
    }

    #[test]
    fn test_color_parse_without_hash() {
        let parsed: RgbColor = "FF8000".parse().unwrap();
        assert_eq!(parsed, RgbColor::new(255, 128, 0));
    }

    #[test]
    fn test_invalid_hex() {
        assert!("invalid".parse::<RgbColor>().is_err());
        assert!("#12345".parse::<RgbColor>().is_err());
        assert!("#1234567".parse::<RgbColor>().is_err());
    }

    #[test]
    fn test_hsv_conversions() {
        let red = RgbColor::new(255, 0, 0);
        let (h, s, v) = red.to_hsv();
        assert!((h - 0.0).abs() < 1e-4);
        assert!((s - 1.0).abs() < 1e-4);
        assert!((v - 1.0).abs() < 1e-4);
        assert_eq!(RgbColor::from_hsv(h, s, v), red);

        let green = RgbColor::new(0, 255, 0);
        let (hg, sg, vg) = green.to_hsv();
        assert!((hg - 120.0).abs() < 1e-4);
        assert_eq!(RgbColor::from_hsv(hg, sg, vg), green);

        let cyan = RgbColor::new(0, 255, 255);
        let (hc, sc, vc) = cyan.to_hsv();
        assert!((hc - 180.0).abs() < 1e-4);
        assert_eq!(RgbColor::from_hsv(hc, sc, vc), cyan);

        let white = RgbColor::WHITE;
        let (_hw, sw, vw) = white.to_hsv();
        assert!((sw - 0.0).abs() < 1e-4);
        assert!((vw - 1.0).abs() < 1e-4);
        assert_eq!(RgbColor::from_hsv(0.0, 0.0, 1.0), white);
    }
}
