//! Backend trait definition and connection status models.
//!
//! Provides the abstract `RgbBackend` trait allowing the application and domain
//! layers to interact with RGB hardware without depending directly on OpenRGB or HID.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::color::RgbColor;
use crate::domain::device::{DeviceInfo, RgbState};
use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::error::BackendError;
use crate::domain::zone::{ZoneConfig, ZoneId};

/// Current operational and connectivity status of an RGB backend.
///
/// Example:
/// ```
/// use msi_rgb::backend::BackendStatus;
///
/// let status = BackendStatus::Connected {
///     device_name: "MSI MysticLight MS-1565".to_string(),
///     protocol_version: 5,
/// };
/// assert!(status.is_connected());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendStatus {
    /// Fully connected and operational.
    Connected {
        /// Name of the active RGB hardware device.
        device_name: String,
        /// SDK / transport protocol version in use.
        protocol_version: u32,
    },
    /// Attempting to establish or re-establish a connection.
    Connecting {
        /// Number of reconnection attempts made so far.
        attempt: u32,
        /// Informational message.
        message: String,
    },
    /// Currently disconnected / standby.
    Disconnected {
        /// Reason for disconnection.
        reason: String,
    },
    /// Error state.
    Error {
        /// Error description.
        message: String,
    },
}

impl BackendStatus {
    /// Returns true if the backend is currently connected and ready.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. })
    }

    /// Returns human-readable status text.
    #[must_use]
    pub fn display_text(&self) -> String {
        match self {
            Self::Connected { device_name, .. } => format!("Connected: {device_name}"),
            Self::Connecting { attempt, message } => {
                format!("Connecting (attempt {attempt}): {message}")
            }
            Self::Disconnected { reason } => format!("Disconnected: {reason}"),
            Self::Error { message } => format!("Error: {message}"),
        }
    }
}

/// Abstract asynchronous interface for RGB hardware control.
///
/// All hardware backends (OpenRGB SDK, Mock backend for testing, or future direct HID)
/// implement this trait.
#[async_trait]
pub trait RgbBackend: Send + Sync {
    /// Establishes connection to the underlying RGB service or hardware.
    async fn connect(&mut self) -> Result<(), BackendError>;

    /// Closes connection and cleans up transport resources.
    async fn disconnect(&mut self) -> Result<(), BackendError>;

    /// Returns the current connection and operational status.
    fn status(&self) -> BackendStatus;

    /// Detects and returns metadata for the target MSI RGB device if present.
    async fn detect_device(&mut self) -> Result<Option<DeviceInfo>, BackendError>;

    /// Applies complete RGB lighting state across all 4 zones.
    async fn apply_state(&mut self, state: &RgbState) -> Result<(), BackendError>;

    /// Applies configuration to a single zone.
    async fn apply_zone(
        &mut self,
        zone_id: ZoneId,
        config: &ZoneConfig,
    ) -> Result<(), BackendError>;

    /// Applies a uniform global mode to all zones simultaneously.
    async fn apply_global_mode(
        &mut self,
        effect: EffectType,
        color: RgbColor,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
    ) -> Result<(), BackendError>;

    /// Commits the active lighting configuration to persistent hardware flash memory.
    async fn save_to_flash(&mut self) -> Result<(), BackendError> {
        Ok(())
    }
}
