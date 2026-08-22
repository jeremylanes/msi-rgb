//! Infrastructure layer (logging, autostart, OS integration).

pub mod autostart;
pub mod logging;

pub use autostart::{disable_autostart, enable_autostart, is_autostart_enabled};
pub use logging::init_logging;
