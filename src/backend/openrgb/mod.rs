//! OpenRGB Backend implementation of `RgbBackend`.
//!
//! Provides the primary hardware driver adapter connecting the application domain
//! to the OpenRGB SDK server with automatic reconnection and capability mapping.

pub mod client;
pub mod protocol;

use async_trait::async_trait;
use tokio::sync::Mutex;

use self::client::OpenRgbClient;
use super::trait_def::{BackendStatus, RgbBackend};
use crate::domain::color::RgbColor;
use crate::domain::device::{DeviceInfo, RgbState};
use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::error::BackendError;
use crate::domain::zone::{ZoneConfig, ZoneId};

/// OpenRGB SDK backend controller.
///
/// Example:
/// ```no_run
/// use msi_rgb::backend::openrgb::OpenRgbBackend;
/// use msi_rgb::backend::RgbBackend;
///
/// # #[tokio::main]
/// # async fn main() {
/// let mut backend = OpenRgbBackend::new("127.0.0.1", 6742);
/// if let Ok(()) = backend.connect().await {
///     println!("Connected: {:?}", backend.status());
/// }
/// # }
/// ```
pub struct OpenRgbBackend {
    client: Mutex<OpenRgbClient>,
    status: BackendStatus,
    host: String,
    port: u16,
}

impl OpenRgbBackend {
    /// Creates a new `OpenRgbBackend` targeting the given host and port.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let host_str = host.into();
        let client = OpenRgbClient::new(&host_str, port, "MSI Mystic Light Linux");
        Self {
            client: Mutex::new(client),
            status: BackendStatus::Disconnected {
                reason: "Not connected".to_string(),
            },
            host: host_str,
            port,
        }
    }

    /// Creates a backend targeting the default local OpenRGB instance.
    #[must_use]
    pub fn default_local() -> Self {
        Self::new("127.0.0.1", protocol::OPENRGB_DEFAULT_PORT)
    }
}

impl Default for OpenRgbBackend {
    fn default() -> Self {
        Self::default_local()
    }
}

#[async_trait]
impl RgbBackend for OpenRgbBackend {
    async fn connect(&mut self) -> Result<(), BackendError> {
        self.status = BackendStatus::Connecting {
            attempt: 1,
            message: format!("Connecting to OpenRGB at {}:{}...", self.host, self.port),
        };

        let mut client = self.client.lock().await;
        match client.connect().await {
            Ok(()) => {
                let dev_name = client
                    .get_active_device_info()
                    .map(|d| d.name)
                    .unwrap_or_else(|| "OpenRGB Controller".to_string());
                let proto_ver = client.protocol_version();

                self.status = BackendStatus::Connected {
                    device_name: dev_name,
                    protocol_version: proto_ver,
                };
                tracing::info!("OpenRgbBackend successfully connected: {:?}", self.status);
                Ok(())
            }
            Err(e) => {
                self.status = BackendStatus::Disconnected {
                    reason: e.to_string(),
                };
                Err(e)
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), BackendError> {
        let mut client = self.client.lock().await;
        let res = client.disconnect().await;
        self.status = BackendStatus::Disconnected {
            reason: "User disconnected".to_string(),
        };
        res
    }

    fn status(&self) -> BackendStatus {
        self.status.clone()
    }

    async fn detect_device(&mut self) -> Result<Option<DeviceInfo>, BackendError> {
        let client = self.client.lock().await;
        if !client.is_connected() {
            return Err(BackendError::Disconnected("OpenRGB is offline".to_string()));
        }
        Ok(client.get_active_device_info())
    }

    async fn apply_state(&mut self, state: &RgbState) -> Result<(), BackendError> {
        let mut client = self.client.lock().await;
        if !client.is_connected() {
            return Err(BackendError::Disconnected("OpenRGB is offline".to_string()));
        }

        if state.sync_all_zones {
            let effect_name = match state.global_effect {
                EffectType::Off => "Off",
                EffectType::Static => "Static",
                EffectType::Breathing => "Breathing",
                EffectType::ColorCycle => "Color Cycle",
                EffectType::Wave => "Wave",
            };
            client
                .apply_mode_by_name(
                    effect_name,
                    state.global_color,
                    state.global_brightness,
                    state.global_speed,
                    state.global_direction,
                )
                .await?;
        } else {
            // Apply per-zone modes
            for zone_config in state.all_zones() {
                client
                    .apply_zone_mode(
                        zone_config.id,
                        zone_config.effect,
                        zone_config.effective_color(),
                        zone_config.brightness,
                        zone_config.speed,
                        zone_config.direction,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn apply_zone(
        &mut self,
        zone_id: ZoneId,
        config: &ZoneConfig,
    ) -> Result<(), BackendError> {
        let mut client = self.client.lock().await;
        if !client.is_connected() {
            return Err(BackendError::Disconnected("OpenRGB is offline".to_string()));
        }

        client
            .apply_zone_mode(
                zone_id,
                config.effect,
                config.effective_color(),
                config.brightness,
                config.speed,
                config.direction,
            )
            .await
    }

    async fn apply_global_mode(
        &mut self,
        effect: EffectType,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError> {
        let mut client = self.client.lock().await;
        if !client.is_connected() {
            return Err(BackendError::Disconnected("OpenRGB is offline".to_string()));
        }

        let effect_name = match effect {
            EffectType::Off => "Off",
            EffectType::Static => "Static",
            EffectType::Breathing => "Breathing",
            EffectType::ColorCycle => "Color Cycle",
            EffectType::Wave => "Wave",
        };

        client
            .apply_mode_by_name(effect_name, color, brightness, speed, direction)
            .await
    }
}
