//! Domain and Application error definitions.
//!
//! Provides structured, typed error enums for domain operations,
//! backend communication, persistence, and application lifecycle.

use thiserror::Error;

/// Core domain error types.
///
/// Example:
/// ```
/// use msi_rgb::domain::error::DomainError;
///
/// let err = DomainError::InvalidColorHex("#ZZZ".to_string());
/// assert!(err.to_string().contains("Invalid hex color"));
/// ```
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Invalid hexadecimal color string format.
    #[error("Invalid hex color format: '{0}' (expected #RRGGBB or RRGGBB)")]
    InvalidColorHex(String),

    /// Brightness percentage value out of allowable 0-100 range.
    #[error("Invalid brightness percentage: {0} (must be 0..=100)")]
    InvalidBrightness(u8),

    /// Speed percentage value out of allowable 0-100 range.
    #[error("Invalid speed percentage: {0} (must be 0..=100)")]
    InvalidSpeed(u8),

    /// Invalid zone index requested.
    #[error("Zone index {0} is out of bounds (device supports zones 1..=4)")]
    InvalidZoneIndex(usize),

    /// Requested effect is unsupported by the target device or zone.
    #[error("Effect '{0}' is not supported by device/zone")]
    UnsupportedEffect(String),

    /// Requested profile was not found.
    #[error("Profile with name '{0}' was not found")]
    ProfileNotFound(String),

    /// Duplicate profile name conflict.
    #[error("Profile with name '{0}' already exists")]
    ProfileAlreadyExists(String),
}

/// Backend hardware / transport errors.
///
/// Example:
/// ```
/// use msi_rgb::domain::error::BackendError;
///
/// let err = BackendError::Disconnected("Connection refused".to_string());
/// assert!(err.to_string().contains("disconnected"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum BackendError {
    /// Backend service connection failed or is unavailable.
    #[error("Backend service is disconnected: {0}")]
    Disconnected(String),

    /// Communication timeout.
    #[error("Backend communication timed out: {0}")]
    Timeout(String),

    /// Protocol communication error.
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Device not found on the backend.
    #[error("Target MSI RGB device was not found: {0}")]
    DeviceNotFound(String),

    /// Command failed on hardware.
    #[error("Hardware command error: {0}")]
    CommandFailed(String),

    /// General I/O error wrapper.
    #[error("I/O error during backend operation: {0}")]
    Io(String),
}

/// High-level application errors.
#[derive(Debug, Error)]
pub enum AppError {
    /// Domain validation failure.
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    /// Backend communication failure.
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),

    /// Configuration / persistence failure.
    #[error("Persistence error: {0}")]
    Persistence(String),

    /// Initialization failure.
    #[error("Initialization error: {0}")]
    Initialization(String),
}
