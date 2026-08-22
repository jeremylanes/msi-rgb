//! Unit tests for domain models (Color, Effect, Zone, Device, Profile).

use msi_rgb::domain::color::RgbColor;
use msi_rgb::domain::device::RgbState;
use msi_rgb::domain::effect::{Brightness, EffectType, Speed};
use msi_rgb::domain::profile::{ProfileStore, UserProfile};
use msi_rgb::domain::zone::{ZoneConfig, ZoneId};

#[test]
fn test_rgb_color_hex_parsing() {
    let red: RgbColor = "#FF0000".parse().unwrap();
    assert_eq!(red, RgbColor::new(255, 0, 0));
    assert_eq!(red.to_hex(), "#FF0000");

    let green: RgbColor = "00FF00".parse().unwrap();
    assert_eq!(green, RgbColor::new(0, 255, 0));

    let cyan = RgbColor::CYAN;
    assert_eq!(cyan.to_hex(), "#00F0FF");

    assert!("#ZZZZZZ".parse::<RgbColor>().is_err());
    assert!("123".parse::<RgbColor>().is_err());
}

#[test]
fn test_rgb_color_f64_conversion() {
    let col = RgbColor::from_f64(1.0, 0.5, 0.0);
    assert_eq!(col.r, 255);
    assert_eq!(col.g, 128);
    assert_eq!(col.b, 0);

    let (r, g, b) = col.to_f64();
    assert!((r - 1.0).abs() < 0.01);
    assert!((g - 0.501).abs() < 0.01);
    assert_eq!(b, 0.0);
}

#[test]
fn test_effect_type_properties() {
    assert!(!EffectType::Off.supports_color());
    assert!(!EffectType::Off.supports_brightness());
    assert!(!EffectType::Off.supports_speed());

    assert!(EffectType::Static.supports_color());
    assert!(EffectType::Static.supports_brightness());
    assert!(!EffectType::Static.supports_speed());

    assert!(EffectType::Breathing.supports_color());
    assert!(EffectType::Breathing.supports_speed());

    assert!(EffectType::Wave.supports_direction());
    assert!(!EffectType::Static.supports_direction());

    assert_eq!("wave".parse::<EffectType>().unwrap(), EffectType::Wave);
    assert_eq!("static".parse::<EffectType>().unwrap(), EffectType::Static);
    assert!("unknown".parse::<EffectType>().is_err());
}

#[test]
fn test_brightness_and_speed_clamping() {
    let b_valid = Brightness::new(75).unwrap();
    assert_eq!(b_valid.as_percentage(), 75);

    assert!(Brightness::new(101).is_err());
    assert_eq!(Brightness::clamped(150).as_percentage(), 100);

    let s_valid = Speed::new(40).unwrap();
    assert_eq!(s_valid.as_percentage(), 40);

    assert!(Speed::new(105).is_err());
    assert_eq!(Speed::clamped(200).as_percentage(), 100);
}

#[test]
fn test_zone_id_and_configs() {
    assert_eq!(ZoneId::Zone1.number(), 1);
    assert_eq!(ZoneId::Zone4.number(), 4);
    assert_eq!(ZoneId::from_number(1).unwrap(), ZoneId::Zone1);
    assert_eq!(ZoneId::from_index(2).unwrap(), ZoneId::Zone3);
    assert!(ZoneId::from_number(5).is_err());

    let mut config = ZoneConfig::new(ZoneId::Zone1, EffectType::Static, RgbColor::MSI_RED);
    assert_eq!(config.effective_color(), RgbColor::MSI_RED);

    config.enabled = false;
    assert_eq!(config.effective_color(), RgbColor::BLACK);
}

#[test]
fn test_rgb_state_sync() {
    let mut state = RgbState::new_uniform(EffectType::Static, RgbColor::MSI_RED);
    assert!(state.sync_all_zones);
    assert_eq!(state.zone_1.color, RgbColor::MSI_RED);
    assert_eq!(state.zone_4.color, RgbColor::MSI_RED);

    state.global_color = RgbColor::CYAN;
    state.global_effect = EffectType::Breathing;
    state.sync_zones_from_global();

    assert_eq!(state.zone_2.color, RgbColor::CYAN);
    assert_eq!(state.zone_3.effect, EffectType::Breathing);
}

#[test]
fn test_profile_store_operations() {
    let mut store = ProfileStore::default();
    let profile = UserProfile::new("Cyberpunk", "Neon split", RgbState::default());

    assert!(store.add_profile(profile.clone()).is_ok());
    assert!(store.add_profile(profile).is_err()); // Duplicate name

    assert!(store.get_profile("Cyberpunk").is_some());
    assert!(store.get_profile("cyberpunk").is_some()); // Case-insensitive

    assert!(store.set_startup_profile("Cyberpunk").is_ok());
    assert_eq!(store.startup_profile, "Cyberpunk");

    assert!(store.delete_profile("Cyberpunk").is_ok());
    assert!(store.get_profile("Cyberpunk").is_none());
}
