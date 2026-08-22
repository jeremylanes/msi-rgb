//! Application service and state layer.

pub mod service;
pub mod state;

pub use service::{RgbService, ServiceCommand};
pub use state::{AppData, AppStateHandle, create_app_state};
