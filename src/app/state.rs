//! Shared Application State model.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::backend::BackendStatus;
use crate::domain::device::{DeviceInfo, RgbState};
use crate::domain::profile::UserProfile;
use crate::domain::zone::ZoneId;
use crate::profiles::ProfileManager;
use crate::settings::AppSettings;

/// Core application state shared between the background RGB service and the UI.
#[derive(Debug, Clone)]
pub struct AppData {
    /// Active RGB lighting configuration.
    pub current_state: RgbState,
    /// Currently selected zone in the UI for editing (None = Sync All).
    pub selected_zone: Option<ZoneId>,
    /// Active profile name.
    pub active_profile_name: String,
    /// Profile manager.
    pub profile_manager: ProfileManager,
    /// Connected device information.
    pub device_info: Option<DeviceInfo>,
    /// Hardware backend connection status.
    pub backend_status: BackendStatus,
    /// User settings.
    pub settings: AppSettings,
}

impl AppData {
    /// Constructs a new `AppData` instance initialized with defaults.
    #[must_use]
    pub fn new(profile_manager: ProfileManager, settings: AppSettings) -> Self {
        let initial_profile = profile_manager
            .startup_profile()
            .cloned()
            .unwrap_or_else(|| UserProfile::new("Default", "Default profile", RgbState::default()));

        Self {
            current_state: initial_profile.state.clone(),
            selected_zone: None,
            active_profile_name: initial_profile.name,
            profile_manager,
            device_info: None,
            backend_status: BackendStatus::Disconnected {
                reason: "Initializing...".to_string(),
            },
            settings,
        }
    }

    /// Saves or updates the current state to the active profile if it is a custom profile.
    pub fn auto_save_custom_profile(&mut self) {
        let act_name = self.active_profile_name.clone();
        let cur = self.current_state.clone();
        if let Some(prof) = self.profile_manager.get_profile(&act_name)
            && !prof.is_builtin
        {
            self.profile_manager
                .save_or_update_current_state(&act_name, cur);
        }
    }
}

/// Thread-safe handle to application state.
pub type AppStateHandle = Arc<RwLock<AppData>>;

/// Helper to construct a new thread-safe state handle.
#[must_use]
pub fn create_app_state(profile_manager: ProfileManager, settings: AppSettings) -> AppStateHandle {
    Arc::new(RwLock::new(AppData::new(profile_manager, settings)))
}
