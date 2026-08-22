//! User Interface module.

pub mod app_window;
pub mod color_picker;
pub mod effect_selector;
pub mod keyboard_preview;
pub mod profiles_dialog;
pub mod status_banner;
pub mod theme;
pub mod zone_control;

use gtk4::Application;
use gtk4::prelude::*;
use std::sync::Arc;

use crate::app::{AppStateHandle, RgbService};
use crate::persistence::Storage;
use app_window::MainWindow;
use theme::load_css_theme;

/// Application ID registered with Wayland / GTK.
pub const APP_ID: &str = "com.msi.mysticlight.rgb";

/// Launches the native GTK4 graphical user interface.
pub fn run_gui(state: AppStateHandle, service: Arc<RgbService>, storage: Storage) {
    let app = Application::builder().application_id(APP_ID).build();

    let state_clone = state.clone();
    let service_clone = service.clone();
    let storage_clone = storage.clone();

    app.connect_startup(|_| {
        load_css_theme();
    });

    app.connect_activate(move |app| {
        let _ = MainWindow::build(
            app,
            state_clone.clone(),
            service_clone.clone(),
            storage_clone.clone(),
        );
    });

    let empty_args: [&str; 0] = [];
    app.run_with_args(&empty_args);
}
