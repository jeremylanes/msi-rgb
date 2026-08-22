//! Domain model layer.
//!
//! Exposes device, zone, color, effect, profile, and error domain types.
//!
//! Example:
//! ```
//! use msi_rgb::domain::{RgbColor, EffectType, ZoneId, RgbState};
//!
//! let state = RgbState::new_uniform(EffectType::Static, RgbColor::MSI_RED);
//! assert_eq!(state.zone_1.color, RgbColor::MSI_RED);
//! ```

pub mod color;
pub mod device;
pub mod effect;
pub mod error;
pub mod profile;
pub mod zone;

pub use color::RgbColor;
pub use device::{DeviceInfo, RgbState};
pub use effect::{Brightness, Direction, EffectType, Speed};
pub use error::{AppError, BackendError, DomainError};
pub use profile::{CURRENT_PROFILE_SCHEMA_VERSION, ProfileStore, UserProfile};
pub use zone::{ZoneConfig, ZoneId};
