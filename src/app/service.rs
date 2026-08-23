//! Background RGB service managing hardware communication and event handling.

use std::time::Duration;
use tokio::sync::mpsc;

use super::state::AppStateHandle;
use crate::backend::{BackendStatus, MockBackend, NativeHidBackend, OpenRgbBackend, RgbBackend};
use crate::domain::color::RgbColor;
use crate::domain::device::RgbState;
use crate::domain::effect::EffectType;
use crate::domain::error::BackendError;
use crate::domain::zone::{ZoneConfig, ZoneId};
use crate::persistence::Storage;

/// Commands sent from the UI / CLI to the background RGB service.
#[derive(Debug, Clone)]
pub enum ServiceCommand {
    /// Apply full state to hardware.
    ApplyState(RgbState),
    /// Apply individual zone configuration.
    ApplyZone(ZoneId, ZoneConfig),
    /// Apply a named profile.
    ApplyProfile(String),
    /// Request connection retry.
    Reconnect,
    /// Stop the background worker.
    Shutdown,
}

/// Service controller handling background hardware communication.
pub struct RgbService {
    state: AppStateHandle,
    #[allow(dead_code)]
    storage: Storage,
    cmd_sender: mpsc::Sender<ServiceCommand>,
}

impl RgbService {
    /// Spawns the background RGB service worker task.
    pub fn spawn(
        state: AppStateHandle,
        storage: Storage,
        use_mock: bool,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<ServiceCommand>(32);

        let service = Self {
            state: state.clone(),
            storage,
            cmd_sender: tx,
        };

        let handle = tokio::spawn(async move {
            let backend: Box<dyn RgbBackend> = if use_mock {
                Box::new(MockBackend::new())
            } else {
                let mut native = NativeHidBackend::new();
                if native.connect().await.is_ok() {
                    tracing::info!(
                        "Using Direct Native HID backend (no OpenRGB required, onboard flash persistence)"
                    );
                    Box::new(native)
                } else {
                    tracing::info!("Direct HID not accessible; falling back to OpenRGB SDK daemon");
                    Box::new(OpenRgbBackend::default_local())
                }
            };

            Self::run_worker(state, backend, rx).await;
        });

        (service, handle)
    }

    /// Sends a command to the background service worker.
    pub async fn send_command(&self, cmd: ServiceCommand) -> Result<(), BackendError> {
        self.cmd_sender
            .send(cmd)
            .await
            .map_err(|e| BackendError::Io(format!("Failed to send service command: {e}")))
    }

    /// Helper to apply a new RGB state asynchronously.
    pub async fn apply_state(&self, state: RgbState) -> Result<(), BackendError> {
        // Update local app state
        {
            let mut app_data = self.state.write().await;
            app_data.current_state = state.clone();
        }
        self.send_command(ServiceCommand::ApplyState(state)).await
    }

    /// Helper to apply a profile by name.
    pub async fn apply_profile(&self, name: &str) -> Result<(), BackendError> {
        let profile_state = {
            let mut app_data = self.state.write().await;
            let profile = app_data
                .profile_manager
                .get_profile(name)
                .cloned()
                .ok_or_else(|| {
                    BackendError::CommandFailed(format!("Profile '{name}' not found"))
                })?;
            app_data.active_profile_name = profile.name.clone();
            app_data.current_state = profile.state.clone();
            let _ = app_data.profile_manager.set_startup_profile(name);
            let store = app_data.profile_manager.store().clone();
            let _ = self.storage.save_profiles(&store);
            profile.state
        };

        self.send_command(ServiceCommand::ApplyState(profile_state))
            .await
    }

    /// Worker event loop with resilient exponential backoff.
    async fn run_worker(
        state: AppStateHandle,
        mut backend: Box<dyn RgbBackend>,
        mut rx: mpsc::Receiver<ServiceCommand>,
    ) {
        let mut backoff_secs = 1u64;
        const MAX_BACKOFF_SECS: u64 = 30;

        let mut active_state: Option<RgbState> = None;
        let mut current_hue = 0.0f64;

        // Initial connection
        Self::try_connect(&state, backend.as_mut()).await;

        // Apply initial state if connected
        if backend.status().is_connected() {
            let initial_state = {
                let data = state.read().await;
                data.current_state.clone()
            };
            active_state = Some(initial_state.clone());
            let _ = backend.apply_state(&initial_state).await;
        }

        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(3));
        heartbeat_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut anim_interval = tokio::time::interval(Duration::from_millis(30));
        anim_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                cmd = rx.recv() => {
                    match cmd {
                        Some(ServiceCommand::ApplyState(new_state)) => {
                            let is_anim = new_state.sync_all_zones && new_state.global_effect == EffectType::ColorCycle;
                            active_state = Some(new_state.clone());
                            if !backend.status().is_connected() {
                                Self::try_connect(&state, backend.as_mut()).await;
                            }
                            if backend.status().is_connected() {
                                if is_anim {
                                    let col = RgbColor::from_hsv(current_hue, 1.0, 1.0);
                                    let _ = backend.apply_global_mode(
                                        EffectType::Static,
                                        col,
                                        new_state.global_brightness,
                                        new_state.global_speed,
                                        new_state.global_direction,
                                    ).await;
                                } else {
                                    let res = backend.apply_state(&new_state).await;
                                    if let Err(e) = res {
                                        tracing::warn!("Failed to apply state: {e}");
                                        Self::update_status(&state, backend.status()).await;
                                    }
                                }
                            }
                        }
                        Some(ServiceCommand::ApplyZone(zid, zconf)) => {
                            if let Some(ref mut st) = active_state {
                                st.sync_all_zones = false;
                                match zid {
                                    ZoneId::Zone1 => st.zone_1 = zconf.clone(),
                                    ZoneId::Zone2 => st.zone_2 = zconf.clone(),
                                    ZoneId::Zone3 => st.zone_3 = zconf.clone(),
                                    ZoneId::Zone4 => st.zone_4 = zconf.clone(),
                                }
                            }
                            if !backend.status().is_connected() {
                                Self::try_connect(&state, backend.as_mut()).await;
                            }
                            if backend.status().is_connected() {
                                let res = backend.apply_zone(zid, &zconf).await;
                                if let Err(e) = res {
                                    tracing::warn!("Failed to apply zone {zid}: {e}");
                                    Self::update_status(&state, backend.status()).await;
                                }
                            }
                        }
                        Some(ServiceCommand::ApplyProfile(name)) => {
                            let maybe_state = {
                                let data = state.read().await;
                                data.profile_manager.get_profile(&name).map(|p| p.state.clone())
                            };
                            if let Some(st) = maybe_state {
                                let is_anim = st.sync_all_zones && st.global_effect == EffectType::ColorCycle;
                                active_state = Some(st.clone());
                                if !backend.status().is_connected() {
                                    Self::try_connect(&state, backend.as_mut()).await;
                                }
                                if backend.status().is_connected() {
                                    if is_anim {
                                        let col = RgbColor::from_hsv(current_hue, 1.0, 1.0);
                                        let _ = backend.apply_global_mode(
                                            EffectType::Static,
                                            col,
                                            st.global_brightness,
                                            st.global_speed,
                                            st.global_direction,
                                        ).await;
                                    } else {
                                        let _ = backend.apply_state(&st).await;
                                    }
                                }
                            }
                        }
                        Some(ServiceCommand::Reconnect) => {
                            backoff_secs = 1;
                            Self::try_connect(&state, backend.as_mut()).await;
                        }
                        Some(ServiceCommand::Shutdown) | None => {
                            if backend.status().is_connected() {
                                let _ = backend.disconnect().await;
                            }
                            tracing::info!("Service worker terminated");
                            break;
                        }
                    }
                }
                _ = anim_interval.tick() => {
                    // Synchronous whole-keyboard color cycle ticker (0 spatial phase difference across zones)
                    if let Some(ref st) = active_state
                        && st.sync_all_zones
                        && st.global_effect == EffectType::ColorCycle
                        && backend.status().is_connected()
                    {
                        let period_secs = match st.global_speed.as_percentage() {
                            0..=15 => 50.0,  // 50s imperceptible drift (Chameleon)
                            16..=35 => 24.0, // 24s slow drift (Spectrum Harmony at 16)
                            36..=55 => 10.0, // 10s smooth wave
                            56..=75 => 6.0,  // 6s brisk wave
                            76..=90 => 3.5,  // 3.5s fast
                            _ => 2.0,        // 2s ultra fast
                        };
                        let ticks_per_cycle = period_secs / 0.03;
                        let deg_step = 360.0 / ticks_per_cycle;
                        current_hue = (current_hue + deg_step).rem_euclid(360.0);

                        let col = RgbColor::from_hsv(current_hue, 1.0, 1.0);
                        let _ = backend
                            .apply_global_mode(
                                EffectType::Static,
                                col,
                                st.global_brightness,
                                st.global_speed,
                                st.global_direction,
                            )
                            .await;
                    }
                }
                _ = heartbeat_interval.tick() => {
                    // Periodic health check & auto-reconnect
                    if !backend.status().is_connected() {
                        tracing::debug!("Backend offline, attempting reconnect in background...");
                        if Self::try_connect(&state, backend.as_mut()).await {
                            backoff_secs = 1;
                            let current_state = {
                                let data = state.read().await;
                                data.current_state.clone()
                            };
                            let _ = backend.apply_state(&current_state).await;
                        } else {
                            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                        }
                    }
                }
            }
        }
    }

    async fn try_connect(state: &AppStateHandle, backend: &mut dyn RgbBackend) -> bool {
        match backend.connect().await {
            Ok(()) => {
                let dev_info = backend.detect_device().await.ok().flatten();
                let status = backend.status();
                {
                    let mut data = state.write().await;
                    data.backend_status = status;
                    data.device_info = dev_info;
                }
                true
            }
            Err(e) => {
                let status = backend.status();
                {
                    let mut data = state.write().await;
                    data.backend_status = status;
                }
                tracing::debug!("Connection attempt failed: {e}");
                false
            }
        }
    }

    async fn update_status(state: &AppStateHandle, status: BackendStatus) {
        let mut data = state.write().await;
        data.backend_status = status;
    }
}
