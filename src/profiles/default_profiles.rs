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
        // 2. Spectrum Harmony (Synchronized whole-keyboard color wave across all zones)
        UserProfile::new_builtin(
            "Spectrum Harmony",
            "Synchronized whole-keyboard color wave cycling through all colors together (no difference between zones, 100% lit)",
            RgbState {
                sync_all_zones: true,
                global_effect: EffectType::ColorCycle,
                global_color: RgbColor::MSI_RED,
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::clamped(16), // Smooth ~24s synchronized full-keyboard wave
                global_direction: Direction::Left,
                zone_1: {
                    let mut z =
                        ZoneConfig::new(ZoneId::Zone1, EffectType::ColorCycle, RgbColor::MSI_RED);
                    z.speed = Speed::clamped(16);
                    z
                },
                zone_2: {
                    let mut z =
                        ZoneConfig::new(ZoneId::Zone2, EffectType::ColorCycle, RgbColor::MSI_RED);
                    z.speed = Speed::clamped(16);
                    z
                },
                zone_3: {
                    let mut z =
                        ZoneConfig::new(ZoneId::Zone3, EffectType::ColorCycle, RgbColor::MSI_RED);
                    z.speed = Speed::clamped(16);
                    z
                },
                zone_4: {
                    let mut z =
                        ZoneConfig::new(ZoneId::Zone4, EffectType::ColorCycle, RgbColor::MSI_RED);
                    z.speed = Speed::clamped(16);
                    z
                },
            },
        ),
        // 3. Chameleon Flow (Imperceptible slow continuous color morph across all zones)
        UserProfile::new_builtin(
            "Chameleon Flow",
            "Seamless ultra-slow continuous spectrum drift across all zones (never turns off, 100% lit)",
            RgbState {
                sync_all_zones: true,
                global_effect: EffectType::ColorCycle,
                global_color: RgbColor::MSI_RED,
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::clamped(10), // 50s ultra-slow imperceptible morph
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(ZoneId::Zone1, EffectType::ColorCycle, RgbColor::MSI_RED),
                zone_2: ZoneConfig::new(ZoneId::Zone2, EffectType::ColorCycle, RgbColor::MSI_RED),
                zone_3: ZoneConfig::new(ZoneId::Zone3, EffectType::ColorCycle, RgbColor::MSI_RED),
                zone_4: ZoneConfig::new(ZoneId::Zone4, EffectType::ColorCycle, RgbColor::MSI_RED),
            },
        ),
        // 3. Cyberpunk Neon (Per-zone vibrant split)
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
        // 4. Synthwave 84 (Outrun retro aesthetic)
        UserProfile::new_builtin(
            "Synthwave 84",
            "Retro-synth neon palette: Neon Magenta, Deep Violet, Electric Cyan & Sunset Gold",
            RgbState {
                sync_all_zones: false,
                global_effect: EffectType::Static,
                global_color: RgbColor::new(255, 0, 128),
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::DEFAULT,
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(
                    ZoneId::Zone1,
                    EffectType::Static,
                    RgbColor::new(255, 0, 128),
                ),
                zone_2: ZoneConfig::new(
                    ZoneId::Zone2,
                    EffectType::Static,
                    RgbColor::new(138, 43, 226),
                ),
                zone_3: ZoneConfig::new(
                    ZoneId::Zone3,
                    EffectType::Static,
                    RgbColor::new(0, 229, 255),
                ),
                zone_4: ZoneConfig::new(
                    ZoneId::Zone4,
                    EffectType::Static,
                    RgbColor::new(255, 179, 0),
                ),
            },
        ),
        // 5. Tokyo Night (Refined cyberpunk dusk)
        UserProfile::new_builtin(
            "Tokyo Night",
            "Modern Japanese dusk aesthetic: Midnight Indigo, Sakura Pink, Cyan & Violet",
            RgbState {
                sync_all_zones: false,
                global_effect: EffectType::Static,
                global_color: RgbColor::new(61, 89, 161),
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::DEFAULT,
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(
                    ZoneId::Zone1,
                    EffectType::Static,
                    RgbColor::new(61, 89, 161),
                ),
                zone_2: ZoneConfig::new(
                    ZoneId::Zone2,
                    EffectType::Static,
                    RgbColor::new(255, 117, 160),
                ),
                zone_3: ZoneConfig::new(
                    ZoneId::Zone3,
                    EffectType::Static,
                    RgbColor::new(122, 162, 247),
                ),
                zone_4: ZoneConfig::new(
                    ZoneId::Zone4,
                    EffectType::Static,
                    RgbColor::new(187, 154, 247),
                ),
            },
        ),
        // 6. Matrix Code (Emerald Green Breathing)
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
        // 7. Arctic Ice (Clean cold blue)
        UserProfile::new_builtin(
            "Arctic Ice",
            "Crisp glacial cyan-blue backlight",
            RgbState::new_uniform(EffectType::Static, RgbColor::new(0, 180, 255)),
        ),
        // 8. Sunset Horizon (Warm Dusk Gradient)
        UserProfile::new_builtin(
            "Sunset Horizon",
            "Warm dusk gradient: Dragon Crimson, Solar Coral, Amber Glow & Pure Gold",
            RgbState {
                sync_all_zones: false,
                global_effect: EffectType::Static,
                global_color: RgbColor::new(255, 23, 68),
                global_brightness: Brightness::DEFAULT,
                global_speed: Speed::DEFAULT,
                global_direction: Direction::Left,
                zone_1: ZoneConfig::new(
                    ZoneId::Zone1,
                    EffectType::Static,
                    RgbColor::new(255, 23, 68),
                ),
                zone_2: ZoneConfig::new(
                    ZoneId::Zone2,
                    EffectType::Static,
                    RgbColor::new(255, 110, 64),
                ),
                zone_3: ZoneConfig::new(
                    ZoneId::Zone3,
                    EffectType::Static,
                    RgbColor::new(255, 171, 0),
                ),
                zone_4: ZoneConfig::new(
                    ZoneId::Zone4,
                    EffectType::Static,
                    RgbColor::new(255, 214, 0),
                ),
            },
        ),
        // 9. Night Shift / Warm Amber (Reduced eye strain)
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
        // 10. Rainbow Wave
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
        // 11. Stealth Off
        UserProfile::new_builtin(
            "Stealth Off",
            "Completely disable all keyboard RGB backlights",
            RgbState::new_uniform(EffectType::Off, RgbColor::BLACK),
        ),
    ]
}
