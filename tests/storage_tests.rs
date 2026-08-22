//! Unit tests for persistence and storage manager.

use msi_rgb::domain::device::RgbState;
use msi_rgb::domain::profile::{ProfileStore, UserProfile};
use msi_rgb::persistence::Storage;
use std::fs;

#[test]
fn test_storage_atomic_write_and_reload() {
    let temp_dir =
        std::env::temp_dir().join(format!("msi_rgb_storage_test_{}", std::process::id()));
    let storage = Storage::with_custom_dir(&temp_dir);

    let mut store = ProfileStore {
        startup_profile: "Night Shift".to_string(),
        ..Default::default()
    };
    store
        .add_profile(UserProfile::new(
            "Night Shift",
            "Amber warm",
            RgbState::default(),
        ))
        .unwrap();

    storage
        .save_profiles(&store)
        .expect("Should save profiles cleanly");

    let loaded = storage
        .load_profiles()
        .expect("Should load profiles cleanly");
    assert_eq!(loaded.startup_profile, "Night Shift");
    assert!(loaded.get_profile("Night Shift").is_some());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
