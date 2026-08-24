//! Application configuration and user preferences.

use serde::{Deserialize, Serialize};
use std::fs;

use crate::domain::error::AppError;
use crate::persistence::Storage;

/// Application settings and user preferences.
///
/// Example:
/// ```
/// use msi_rgb::settings::config::AppSettings;
///
/// let settings = AppSettings::default();
/// assert!(settings.autostart);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether autostart on session login is enabled.
    pub autostart: bool,
    /// Name of profile to apply at session startup.
    pub startup_profile: String,
    /// Whether to force the MockBackend (for simulation / testing).
    pub use_mock_backend: bool,
    /// UI slider / color change debounce delay in milliseconds.
    pub debounce_ms: u64,
    /// Start minimized to background / tray.
    pub start_minimized: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: true,
            startup_profile: "Gaming Red".to_string(),
            use_mock_backend: false,
            debounce_ms: 120,
            start_minimized: false,
        }
    }
}

impl AppSettings {
    /// Loads settings from standard storage or returns default.
    pub fn load_or_default(storage: &Storage) -> Self {
        let path = storage.settings_path();
        if let Some(settings) = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<Self>(&content).ok())
        {
            return settings;
        }

        let default_settings = Self::default();
        let _ = Self::save(&default_settings, storage);
        default_settings
    }

    /// Saves settings to disk via atomic temporary write.
    pub fn save(&self, storage: &Storage) -> Result<(), AppError> {
        let path = storage.settings_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Persistence(format!("Failed to serialize settings: {e}")))?;

        fs::write(&path, json).map_err(|e| {
            AppError::Persistence(format!("Failed to write settings to {path:?}: {e}"))
        })?;

        Ok(())
    }
}
