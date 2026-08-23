//! Hardware backend abstraction layer.
//!
//! Exposes the `RgbBackend` trait, the production `OpenRgbBackend`,
//! and the testing `MockBackend`.

pub mod mock;
pub mod native_hid;
pub mod openrgb;
pub mod trait_def;

pub use mock::MockBackend;
pub use native_hid::NativeHidBackend;
pub use openrgb::OpenRgbBackend;
pub use trait_def::{BackendStatus, RgbBackend};
