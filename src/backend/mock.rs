//! Mock RGB hardware backend for unit testing and offline simulation.
//!
//! Provides a full in-memory implementation of `RgbBackend` with state tracking,
//! fault injection, and zero external hardware or network dependencies.

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;

use super::trait_def::{BackendStatus, RgbBackend};
use crate::domain::color::RgbColor;
use crate::domain::device::{DeviceInfo, RgbState};
use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::error::BackendError;
use crate::domain::zone::{ZoneConfig, ZoneId};

/// In-memory mock backend simulating an MSI Katana 15 HX RGB keyboard.
///
/// Example:
/// ```
/// use msi_rgb::backend::mock::MockBackend;
/// use msi_rgb::backend::RgbBackend;
/// use msi_rgb::domain::device::RgbState;
///
/// # #[tokio::main]
/// # async fn main() {
/// let mut mock = MockBackend::new();
/// mock.connect().await.unwrap();
/// assert!(mock.status().is_connected());
///
/// let state = RgbState::default();
/// mock.apply_state(&state).await.unwrap();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MockBackend {
    connected: Arc<AtomicBool>,
    fail_next_command: Arc<AtomicBool>,
    device_info: DeviceInfo,
    current_state: Arc<RwLock<RgbState>>,
}

impl MockBackend {
    /// Creates a new `MockBackend` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(false)),
            fail_next_command: Arc::new(AtomicBool::new(false)),
            device_info: DeviceInfo::katana_ms1565(),
            current_state: Arc::new(RwLock::new(RgbState::default())),
        }
    }

    /// Injects a failure for the next command execution (useful for error handling tests).
    pub fn inject_failure(&self) {
        self.fail_next_command.store(true, Ordering::SeqCst);
    }

    /// Gets a copy of the current in-memory simulated hardware state.
    pub async fn get_simulated_state(&self) -> RgbState {
        self.current_state.read().await.clone()
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RgbBackend for MockBackend {
    async fn connect(&mut self) -> Result<(), BackendError> {
        if self.fail_next_command.swap(false, Ordering::SeqCst) {
            return Err(BackendError::Disconnected(
                "Injected connection failure".to_string(),
            ));
        }
        self.connected.store(true, Ordering::SeqCst);
        tracing::info!("MockBackend connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BackendError> {
        self.connected.store(false, Ordering::SeqCst);
        tracing::info!("MockBackend disconnected");
        Ok(())
    }

    fn status(&self) -> BackendStatus {
        if self.connected.load(Ordering::SeqCst) {
            BackendStatus::Connected {
                device_name: self.device_info.name.clone(),
                protocol_version: 5,
            }
        } else {
            BackendStatus::Disconnected {
                reason: "Mock backend offline".to_string(),
            }
        }
    }

    async fn detect_device(&mut self) -> Result<Option<DeviceInfo>, BackendError> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(BackendError::Disconnected("Not connected".to_string()));
        }
        Ok(Some(self.device_info.clone()))
    }

    async fn apply_state(&mut self, state: &RgbState) -> Result<(), BackendError> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(BackendError::Disconnected("Not connected".to_string()));
        }
        if self.fail_next_command.swap(false, Ordering::SeqCst) {
            return Err(BackendError::CommandFailed(
                "Simulated command failure".to_string(),
            ));
        }

        let mut current = self.current_state.write().await;
        *current = state.clone();
        tracing::debug!("MockBackend applied state: {:?}", state);
        Ok(())
    }

    async fn apply_zone(
        &mut self,
        zone_id: ZoneId,
        config: &ZoneConfig,
    ) -> Result<(), BackendError> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(BackendError::Disconnected("Not connected".to_string()));
        }
        if self.fail_next_command.swap(false, Ordering::SeqCst) {
            return Err(BackendError::CommandFailed(
                "Simulated command failure".to_string(),
            ));
        }

        let mut current = self.current_state.write().await;
        *current.zone_mut(zone_id) = config.clone();
        current.sync_all_zones = false;
        tracing::debug!("MockBackend applied zone {}: {:?}", zone_id, config);
        Ok(())
    }

    async fn apply_global_mode(
        &mut self,
        effect: EffectType,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(BackendError::Disconnected("Not connected".to_string()));
        }
        if self.fail_next_command.swap(false, Ordering::SeqCst) {
            return Err(BackendError::CommandFailed(
                "Simulated command failure".to_string(),
            ));
        }

        let mut current = self.current_state.write().await;
        current.sync_all_zones = true;
        current.global_effect = effect;
        current.global_color = color;
        current.global_brightness = brightness;
        current.global_speed = speed;
        current.global_direction = direction;
        current.sync_zones_from_global();
        tracing::debug!("MockBackend applied global mode: {:?}", effect);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_backend_lifecycle() {
        let mut mock = MockBackend::new();
        assert!(!mock.status().is_connected());

        mock.connect().await.unwrap();
        assert!(mock.status().is_connected());

        let dev = mock.detect_device().await.unwrap();
        assert!(dev.is_some());
        assert_eq!(dev.unwrap().name, "MSI MysticLight MS-1565");

        let state = RgbState::new_uniform(EffectType::Static, RgbColor::CYAN);
        mock.apply_state(&state).await.unwrap();

        let current = mock.get_simulated_state().await;
        assert_eq!(current.zone_1.color, RgbColor::CYAN);

        mock.disconnect().await.unwrap();
        assert!(!mock.status().is_connected());
    }

    #[tokio::test]
    async fn test_mock_backend_failure_injection() {
        let mut mock = MockBackend::new();
        mock.inject_failure();
        assert!(mock.connect().await.is_err());

        // Next attempt succeeds
        assert!(mock.connect().await.is_ok());
    }
}
