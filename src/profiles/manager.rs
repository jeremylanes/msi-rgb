//! Profile manager for creating, updating, duplicating, and querying profiles.

use super::default_profiles::create_default_profiles;
use crate::domain::device::RgbState;
use crate::domain::error::DomainError;
use crate::domain::profile::{ProfileStore, UserProfile};

/// In-memory manager for profile operations.
///
/// Example:
/// ```
/// use msi_rgb::profiles::manager::ProfileManager;
/// use msi_rgb::domain::device::RgbState;
///
/// let mut manager = ProfileManager::with_defaults();
/// assert!(manager.list_profiles().len() >= 6);
///
/// manager.create_profile("Custom Blue", "My profile", RgbState::default()).unwrap();
/// assert!(manager.get_profile("Custom Blue").is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ProfileManager {
    store: ProfileStore,
}

impl ProfileManager {
    /// Creates a new `ProfileManager` populated with built-in default profiles.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut store = ProfileStore::default();
        for p in create_default_profiles() {
            let _ = store.add_profile(p);
        }
        Self { store }
    }

    /// Wraps an existing `ProfileStore`.
    #[must_use]
    pub fn from_store(mut store: ProfileStore) -> Self {
        // Ensure default factory profiles exist if empty
        if store.profiles.is_empty() {
            for p in create_default_profiles() {
                let _ = store.add_profile(p);
            }
        }
        Self { store }
    }

    /// Returns a reference to the inner `ProfileStore`.
    #[must_use]
    pub const fn store(&self) -> &ProfileStore {
        &self.store
    }

    /// Returns all configured profiles.
    #[must_use]
    pub fn list_profiles(&self) -> &[UserProfile] {
        &self.store.profiles
    }

    /// Finds a profile by name.
    #[must_use]
    pub fn get_profile(&self, name: &str) -> Option<&UserProfile> {
        self.store.get_profile(name)
    }

    /// Creates and registers a new custom profile.
    pub fn create_profile(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        state: RgbState,
    ) -> Result<UserProfile, DomainError> {
        let name_str = name.into();
        if name_str.trim().is_empty() {
            return Err(DomainError::ProfileAlreadyExists(
                "Profile name cannot be empty".to_string(),
            ));
        }

        let profile = UserProfile::new(name_str, description, state);
        self.store.add_profile(profile.clone())?;
        Ok(profile)
    }

    /// Duplicates an existing profile with a new name.
    pub fn duplicate_profile(
        &mut self,
        source_name: &str,
        new_name: impl Into<String>,
    ) -> Result<UserProfile, DomainError> {
        let source = self
            .get_profile(source_name)
            .ok_or_else(|| DomainError::ProfileNotFound(source_name.to_string()))?
            .clone();

        let new_name_str = new_name.into();
        let new_profile = UserProfile::new(
            new_name_str,
            format!("Copy of {}", source.name),
            source.state,
        );

        self.store.add_profile(new_profile.clone())?;
        Ok(new_profile)
    }

    /// Deletes a profile by name (built-in profiles cannot be deleted).
    pub fn delete_profile(&mut self, name: &str) -> Result<UserProfile, DomainError> {
        if self.get_profile(name).is_some_and(|p| p.is_builtin) {
            return Err(DomainError::ProfileAlreadyExists(format!(
                "Cannot delete built-in factory preset '{name}'"
            )));
        }
        self.store.delete_profile(name)
    }

    /// Sets the startup profile.
    pub fn set_startup_profile(&mut self, name: &str) -> Result<(), DomainError> {
        self.store.set_startup_profile(name)
    }

    /// Gets the startup profile name.
    #[must_use]
    pub fn startup_profile_name(&self) -> &str {
        &self.store.startup_profile
    }

    /// Retrieves the startup profile object if found.
    #[must_use]
    pub fn startup_profile(&self) -> Option<&UserProfile> {
        self.get_profile(&self.store.startup_profile)
            .or_else(|| self.store.profiles.first())
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}
