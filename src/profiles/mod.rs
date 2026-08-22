//! Profile management layer.

pub mod default_profiles;
pub mod manager;

pub use default_profiles::create_default_profiles;
pub use manager::ProfileManager;
