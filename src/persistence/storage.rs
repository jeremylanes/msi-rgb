//! Robust XDG storage manager with atomic writes and schema migrations.
//!
//! Stores user profiles and application settings in standard XDG configuration
//! directories (`$XDG_CONFIG_HOME/msi-rgb/` or `~/.config/msi-rgb/`).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::domain::error::AppError;
use crate::domain::profile::ProfileStore;
use crate::profiles::default_profiles::create_default_profiles;

/// Primary storage coordinator for persisting application data.
#[derive(Debug, Clone)]
pub struct Storage {
    config_dir: PathBuf,
}

impl Storage {
    /// Creates a new `Storage` instance using standard XDG directories.
    #[must_use]
    pub fn default_xdg() -> Self {
        let config_dir = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(dir) => PathBuf::from(dir).join("msi-rgb"),
            None => {
                let home = std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into());
                PathBuf::from(home).join(".config").join("msi-rgb")
            }
        };
        Self { config_dir }
    }

    /// Creates a `Storage` instance targeting a custom directory (ideal for testing).
    #[must_use]
    pub fn with_custom_dir(path: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: path.into(),
        }
    }

    /// Returns the configuration directory path.
    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Returns the path to `profiles.json`.
    #[must_use]
    pub fn profiles_path(&self) -> PathBuf {
        self.config_dir.join("profiles.json")
    }

    /// Returns the path to `settings.json`.
    #[must_use]
    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    /// Loads the `ProfileStore` from disk, initializing with default factory profiles if missing.
    pub fn load_profiles(&self) -> Result<ProfileStore, AppError> {
        let path = self.profiles_path();
        if !path.exists() {
            tracing::info!(
                "No existing profiles file at {:?}, generating defaults...",
                path
            );
            let mut store = ProfileStore::default();
            for p in create_default_profiles() {
                let _ = store.add_profile(p);
            }
            let _ = self.save_profiles(&store);
            return Ok(store);
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            AppError::Persistence(format!("Failed to read profiles file at {path:?}: {e}"))
        })?;

        let mut store: ProfileStore = serde_json::from_str(&content).map_err(|e| {
            AppError::Persistence(format!("Failed to parse profiles JSON from {path:?}: {e}"))
        })?;

        // Migration / validation: ensure default factory presets exist
        let defaults = create_default_profiles();
        for default_p in defaults {
            if store.get_profile(&default_p.name).is_none() {
                let _ = store.add_profile(default_p);
            }
        }

        Ok(store)
    }

    /// Saves the `ProfileStore` to disk using an atomic temporary file write.
    pub fn save_profiles(&self, store: &ProfileStore) -> Result<(), AppError> {
        let path = self.profiles_path();
        self.atomic_save_json(&path, store)
    }

    /// Atomically writes any serializable data to target JSON file.
    fn atomic_save_json<T: serde::Serialize>(&self, path: &Path, data: &T) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Persistence(format!("Failed to create config directory {parent:?}: {e}"))
            })?;
        }

        let json = serde_json::to_string_pretty(data)
            .map_err(|e| AppError::Persistence(format!("JSON serialization error: {e}")))?;

        // Write to temporary file in the same directory first
        let tmp_path = path.with_extension("tmp");
        let mut file = File::create(&tmp_path).map_err(|e| {
            AppError::Persistence(format!("Failed to create temp file {tmp_path:?}: {e}"))
        })?;

        file.write_all(json.as_bytes()).map_err(|e| {
            AppError::Persistence(format!(
                "Failed to write data to temp file {tmp_path:?}: {e}"
            ))
        })?;

        file.sync_all().map_err(|e| {
            AppError::Persistence(format!("Failed to sync temp file {tmp_path:?}: {e}"))
        })?;

        drop(file);

        // Atomic rename
        fs::rename(&tmp_path, path).map_err(|e| {
            AppError::Persistence(format!("Failed to rename temp file to {path:?}: {e}"))
        })?;

        tracing::debug!("Persisted data successfully to {:?}", path);
        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::default_xdg()
    }
}
