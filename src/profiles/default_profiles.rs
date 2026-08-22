//! Built-in factory presets for MSI MysticLight keyboards.
//!
//! Provides ready-to-use profiles tailored for gaming, night work, coding,
//! and aesthetics.

use crate::domain::color::RgbColor;
use crate::domain::device::RgbState;
use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::profile::UserProfile;
use crate::domain::zone::{ZoneConfig, ZoneId};

/// Generates the default set of factory preset profiles.
///
/// Example:
/// ```
/// use msi_rgb::profiles::default_profiles::create_default_profiles;
///
/// let profiles = create_default_profiles();
/// assert!(!profiles.is_empty());
/// assert!(profiles.iter().any(|p| p.name == "Gaming Red"));
/// ```
#[must_use]
pub fn create_default_profiles() -> Vec<UserProfile> {
    vec![
        // 1. MSI Signature Gaming Red
        UserProfile::new_builtin(
            "Gaming Red",
            "MSI Signature Dragon Red solid illumination",
            RgbState::new_uniform(EffectType::Static, RgbColor::MSI_RED),
        ),
        // 2. Cyberpunk Neon (Per-zone vibrant split)
        UserProfile::new_builtin(
            "Cyberpunk Neon",
            "High-contrast Cyberpunk split: Cyan, Magenta, Yellow, Cyan",
            RgbState {
                sync_all_zones: false,
                global_effect: EffectType::Static,
                global_color: RgbColor::CYAN,
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::DEFAULT,
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(ZoneId::Zone1, EffectType::Static, RgbColor::CYAN),
                zone_2: ZoneConfig::new(ZoneId::Zone2, EffectType::Static, RgbColor::MAGENTA),
                zone_3: ZoneConfig::new(ZoneId::Zone3, EffectType::Static, RgbColor::YELLOW),
                zone_4: ZoneConfig::new(ZoneId::Zone4, EffectType::Static, RgbColor::CYAN),
            },
        ),
        // 3. Matrix Code (Emerald Green Breathing)
        UserProfile::new_builtin(
            "Matrix Code",
            "Subtle emerald green breathing pulse",
            RgbState {
                sync_all_zones: true,
                global_effect: EffectType::Breathing,
                global_color: RgbColor::GREEN,
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::clamped(60),
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(ZoneId::Zone1, EffectType::Breathing, RgbColor::GREEN),
                zone_2: ZoneConfig::new(ZoneId::Zone2, EffectType::Breathing, RgbColor::GREEN),
                zone_3: ZoneConfig::new(ZoneId::Zone3, EffectType::Breathing, RgbColor::GREEN),
                zone_4: ZoneConfig::new(ZoneId::Zone4, EffectType::Breathing, RgbColor::GREEN),
            },
        ),
        // 4. Arctic Ice (Clean cold blue)
        UserProfile::new_builtin(
            "Arctic Ice",
            "Crisp glacial cyan-blue backlight",
            RgbState::new_uniform(EffectType::Static, RgbColor::new(0, 180, 255)),
        ),
        // 5. Night Shift / Warm Amber (Reduced eye strain)
        UserProfile::new_builtin(
            "Night Shift",
            "Warm amber glow at comfortable low brightness",
            RgbState {
                sync_all_zones: true,
                global_effect: EffectType::Static,
                global_color: RgbColor::AMBER,
                global_brightness: Brightness::clamped(40),
                global_speed: Speed::DEFAULT,
                global_direction: Direction::Left,
                zone_1: {
                    let mut z = ZoneConfig::new(ZoneId::Zone1, EffectType::Static, RgbColor::AMBER);
                    z.brightness = Brightness::clamped(40);
                    z
                },
                zone_2: {
                    let mut z = ZoneConfig::new(ZoneId::Zone2, EffectType::Static, RgbColor::AMBER);
                    z.brightness = Brightness::clamped(40);
                    z
                },
                zone_3: {
                    let mut z = ZoneConfig::new(ZoneId::Zone3, EffectType::Static, RgbColor::AMBER);
                    z.brightness = Brightness::clamped(40);
                    z
                },
                zone_4: {
                    let mut z = ZoneConfig::new(ZoneId::Zone4, EffectType::Static, RgbColor::AMBER);
                    z.brightness = Brightness::clamped(40);
                    z
                },
            },
        ),
        // 6. Rainbow Wave
        UserProfile::new_builtin(
            "Rainbow Wave",
            "Dynamic animated spectrum wave flowing across zones",
            RgbState {
                sync_all_zones: true,
                global_effect: EffectType::Wave,
                global_color: RgbColor::CYAN,
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::clamped(65),
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(ZoneId::Zone1, EffectType::Wave, RgbColor::CYAN),
                zone_2: ZoneConfig::new(ZoneId::Zone2, EffectType::Wave, RgbColor::CYAN),
                zone_3: ZoneConfig::new(ZoneId::Zone3, EffectType::Wave, RgbColor::CYAN),
                zone_4: ZoneConfig::new(ZoneId::Zone4, EffectType::Wave, RgbColor::CYAN),
            },
        ),
        // 7. Stealth Off
        UserProfile::new_builtin(
            "Stealth Off",
            "Completely disable all keyboard RGB backlights",
            RgbState::new_uniform(EffectType::Off, RgbColor::BLACK),
        ),
    ]
}
