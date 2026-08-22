//! # MSI Mystic Light RGB Controller
//!
//! Native Linux application and domain library for controlling 4-zone RGB keyboards
//! on MSI gaming laptops (specifically MSI Katana 15 HX B14WGK / MS-1565 / `1462:1601`).
//!
//! ## Architecture
//!
//! The application is structured into decoupled layers:
//! - [`domain`]: Core types (`RgbColor`, `EffectType`, `ZoneId`, `RgbState`, `UserProfile`)
//! - [`backend`]: Hardware driver abstraction trait (`RgbBackend`), `OpenRgbBackend`, and `MockBackend`
//! - [`effects`]: Lighting effect models and preset palettes
//! - [`profiles`]: Preset management, default factory presets, and profile storage
//! - [`persistence`]: XDG-compliant atomic JSON file storage
//! - [`settings`]: User application preferences
//! - [`infrastructure`]: Logging, autostart integration, systemd user service
//! - [`app`]: Thread-safe application state and background worker service
//! - [`ui`]: Modern GTK4 native user interface with interactive 4-zone preview
//!
//! ## Usage Example
//!
//! ```no_run
//! use msi_rgb::domain::{RgbColor, EffectType, RgbState};
//! use msi_rgb::backend::{MockBackend, RgbBackend};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let mut backend = MockBackend::new();
//! backend.connect().await.unwrap();
//!
//! let state = RgbState::new_uniform(EffectType::Static, RgbColor::MSI_RED);
//! backend.apply_state(&state).await.unwrap();
//! # }
//! ```

pub mod app;
pub mod backend;
pub mod domain;
pub mod effects;
pub mod infrastructure;
pub mod persistence;
pub mod profiles;
pub mod settings;
pub mod ui;
