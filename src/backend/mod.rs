//! Hardware backend abstraction layer.
//!
//! Exposes the `RgbBackend` trait, the production `OpenRgbBackend`,
//! and the testing `MockBackend`.

pub mod mock;
pub mod openrgb;
pub mod trait_def;

pub use mock::MockBackend;
pub use openrgb::OpenRgbBackend;
pub use trait_def::{BackendStatus, RgbBackend};
