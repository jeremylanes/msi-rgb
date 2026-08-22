//! Tests for MockBackend simulating hardware behavior.

use msi_rgb::backend::{MockBackend, RgbBackend};
use msi_rgb::domain::color::RgbColor;
use msi_rgb::domain::effect::{Brightness, Direction, EffectType, Speed};
use msi_rgb::domain::zone::{ZoneConfig, ZoneId};

#[tokio::test]
async fn test_mock_backend_full_flow() {
    let mut backend = MockBackend::new();
    assert!(!backend.status().is_connected());

    // Connect
    backend.connect().await.expect("Connect should succeed");
    assert!(backend.status().is_connected());

    // Device detection
    let dev = backend
        .detect_device()
        .await
        .unwrap()
        .expect("Device must be present");
    assert_eq!(dev.zone_count, 4);
    assert!(dev.supports_per_zone);

    // Apply global mode
    backend
        .apply_global_mode(
            EffectType::Static,
            RgbColor::CYAN,
            Brightness::DEFAULT,
            Speed::DEFAULT,
            Direction::Left,
        )
        .await
        .unwrap();

    let state = backend.get_simulated_state().await;
    assert!(state.sync_all_zones);
    assert_eq!(state.zone_1.color, RgbColor::CYAN);

    // Apply individual zone
    let z2_config = ZoneConfig::new(ZoneId::Zone2, EffectType::Static, RgbColor::MAGENTA);
    backend.apply_zone(ZoneId::Zone2, &z2_config).await.unwrap();

    let updated = backend.get_simulated_state().await;
    assert!(!updated.sync_all_zones);
    assert_eq!(updated.zone_2.color, RgbColor::MAGENTA);

    // Disconnect
    backend.disconnect().await.unwrap();
    assert!(!backend.status().is_connected());
}
