//! Profile domain model and serialization store.
//!
//! Provides user profile data structures, versioned serialization envelopes,
//! and profile management methods.

use serde::{Deserialize, Serialize};

use super::device::RgbState;
use super::error::DomainError;

/// Schema version for persisted profiles to guarantee future migration compatibility.
pub const CURRENT_PROFILE_SCHEMA_VERSION: u32 = 1;

/// A named user configuration profile storing complete RGB lighting state.
///
/// Example:
/// ```
/// use msi_rgb::domain::profile::UserProfile;
/// use msi_rgb::domain::device::RgbState;
///
/// let profile = UserProfile::new("Gaming Red", "MSI signature gaming look", RgbState::default());
/// assert_eq!(profile.name, "Gaming Red");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique identifier / profile name.
    pub name: String,
    /// Optional user description.
    pub description: String,
    /// Whether this profile is a built-in read-only preset.
    pub is_builtin: bool,
    /// Associated RGB keyboard state.
    pub state: RgbState,
}

impl UserProfile {
    /// Creates a new custom `UserProfile`.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>, state: RgbState) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            is_builtin: false,
            state,
        }
    }

    /// Creates a built-in factory preset `UserProfile`.
    #[must_use]
    pub fn new_builtin(
        name: impl Into<String>,
        description: impl Into<String>,
        state: RgbState,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            is_builtin: true,
            state,
        }
    }
}

/// Versioned root store structure for saving profiles to disk.
///
/// Example:
/// ```
/// use msi_rgb::domain::profile::{ProfileStore, UserProfile};
/// use msi_rgb::domain::device::RgbState;
///
/// let mut store = ProfileStore::default();
/// let profile = UserProfile::new("Custom", "My profile", RgbState::default());
/// store.add_profile(profile).unwrap();
/// assert!(store.get_profile("Custom").is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileStore {
    /// Schema format version number.
    pub schema_version: u32,
    /// Name of the active profile to apply on session startup.
    pub startup_profile: String,
    /// List of configured user profiles.
    pub profiles: Vec<UserProfile>,
}

impl ProfileStore {
    /// Finds a profile by case-insensitive name match.
    #[must_use]
    pub fn get_profile(&self, name: &str) -> Option<&UserProfile> {
        let name_lower = name.trim().to_lowercase();
        self.profiles
            .iter()
            .find(|p| p.name.trim().to_lowercase() == name_lower)
    }

    /// Finds a mutable profile by case-insensitive name match.
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut UserProfile> {
        let name_lower = name.trim().to_lowercase();
        self.profiles
            .iter_mut()
            .find(|p| p.name.trim().to_lowercase() == name_lower)
    }

    /// Adds a new profile to the store.
    pub fn add_profile(&mut self, profile: UserProfile) -> Result<(), DomainError> {
        if self.get_profile(&profile.name).is_some() {
            return Err(DomainError::ProfileAlreadyExists(profile.name));
        }
        self.profiles.push(profile);
        Ok(())
    }

    /// Updates or replaces an existing profile.
    pub fn save_or_update(&mut self, profile: UserProfile) {
        if let Some(existing) = self.get_profile_mut(&profile.name) {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    /// Removes a profile by name.
    pub fn delete_profile(&mut self, name: &str) -> Result<UserProfile, DomainError> {
        let name_lower = name.trim().to_lowercase();
        let pos = self
            .profiles
            .iter()
            .position(|p| p.name.trim().to_lowercase() == name_lower)
            .ok_or_else(|| DomainError::ProfileNotFound(name.to_string()))?;

        Ok(self.profiles.remove(pos))
    }

    /// Sets the startup profile name after verifying it exists.
    pub fn set_startup_profile(&mut self, name: &str) -> Result<(), DomainError> {
        if self.get_profile(name).is_some() {
            self.startup_profile = name.to_string();
            Ok(())
        } else {
            Err(DomainError::ProfileNotFound(name.to_string()))
        }
    }
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_PROFILE_SCHEMA_VERSION,
            startup_profile: "Gaming Red".to_string(),
            profiles: Vec::new(),
        }
    }
}
