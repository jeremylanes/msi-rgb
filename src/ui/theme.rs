//! Modern MSI Mystic Light GTK4 Theme and styling.
//!
//! Provides CSS styles inspired by MSI gaming aesthetics: deep obsidian dark background,
//! subtle card elevation, MSI crimson accents (#FF1744), glowing borders, and rounded surfaces.

use gtk4::CssProvider;
use gtk4::gdk::Display;

/// Full CSS stylesheet for the application.
pub const APP_CSS: &str = r#"
/* ===== Core Dark Theme ===== */
window.main-window {
    background-color: #0d1117;
    color: #e6edf3;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
}

headerbar {
    background-color: #161b22;
    border-bottom: 1px solid #30363d;
    min-height: 48px;
    padding: 0 12px;
}

headerbar .title {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: 0.5px;
    color: #ffffff;
}

headerbar .subtitle {
    font-size: 11px;
    color: #8b949e;
}

/* ===== Card Surfaces ===== */
.msi-card {
    background-color: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 16px;
    margin: 6px;
}

.msi-card-header {
    font-size: 13px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #ff3355;
    margin-bottom: 10px;
}

/* ===== Status Pill Indicator ===== */
.status-pill {
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 600;
}

.status-pill.connected {
    background-color: rgba(35, 134, 54, 0.2);
    border: 1px solid #238636;
    color: #3fb950;
}

.status-pill.connecting {
    background-color: rgba(210, 153, 34, 0.2);
    border: 1px solid #d29922;
    color: #e3b341;
}

.status-pill.disconnected {
    background-color: rgba(248, 81, 73, 0.2);
    border: 1px solid #f85149;
    color: #ff7b72;
}

/* ===== Keyboard Visualizer ===== */
.keyboard-frame {
    background-color: #0b0e14;
    border: 1px solid #21262d;
    border-radius: 14px;
    padding: 14px;
}

.keyboard-zone-box {
    border-radius: 10px;
    padding: 12px 8px;
    margin: 4px;
    border: 2px solid transparent;
    transition: all 200ms ease-in-out;
}

.keyboard-zone-box:hover {
    border-color: rgba(255, 255, 255, 0.4);
}

.keyboard-zone-box.selected {
    border-color: #ffffff;
    box-shadow: 0 0 12px rgba(255, 255, 255, 0.4);
}

.zone-label {
    font-size: 11px;
    font-weight: 700;
    color: #ffffff;
    text-shadow: 0 1px 2px rgba(0,0,0,0.8);
}

.zone-sublabel {
    font-size: 9px;
    color: rgba(255, 255, 255, 0.85);
    text-shadow: 0 1px 2px rgba(0,0,0,0.8);
}

/* Simulated Key Cap */
.keycap {
    background-color: rgba(20, 24, 32, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    min-width: 22px;
    min-height: 22px;
    margin: 2px;
    font-size: 9px;
    font-weight: 600;
    color: #ffffff;
}

/* ===== Effect Chips / Mode Selector ===== */
.effect-chip {
    background-color: #21262d;
    border: 1px solid #30363d;
    color: #c9d1d9;
    border-radius: 8px;
    padding: 8px 16px;
    font-weight: 600;
    font-size: 13px;
    transition: all 150ms ease;
}

.effect-chip:hover {
    background-color: #30363d;
    color: #ffffff;
}

.effect-chip.active {
    background-color: rgba(255, 23, 68, 0.15);
    border: 1px solid #ff1744;
    color: #ff3355;
    box-shadow: 0 0 8px rgba(255, 23, 68, 0.3);
}

/* ===== Color Palette Buttons ===== */
.color-circle {
    min-width: 32px;
    min-height: 32px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.2);
    margin: 4px;
    transition: transform 120ms ease;
}

.color-circle:hover {
    transform: scale(1.15);
    border-color: #ffffff;
}

.color-circle.selected {
    border: 3px solid #ffffff;
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.6);
}

/* ===== Action Buttons ===== */
.btn-primary {
    background: linear-gradient(135deg, #e60026 0%, #ff1744 100%);
    color: #ffffff;
    font-weight: 700;
    border-radius: 8px;
    padding: 8px 20px;
    border: none;
    box-shadow: 0 2px 8px rgba(230, 0, 38, 0.4);
    transition: all 150ms ease;
}

.btn-primary:hover {
    background: linear-gradient(135deg, #ff1744 0%, #ff4569 100%);
    box-shadow: 0 4px 14px rgba(230, 0, 38, 0.6);
}

.btn-secondary {
    background-color: #21262d;
    border: 1px solid #30363d;
    color: #c9d1d9;
    border-radius: 8px;
    padding: 8px 16px;
    font-weight: 600;
}

.btn-secondary:hover {
    background-color: #30363d;
    color: #ffffff;
}

/* ===== Sliders / Scale ===== */
scale highlight {
    background-color: #ff1744;
    border-radius: 4px;
}

scale trough {
    background-color: #21262d;
    border-radius: 4px;
    min-height: 6px;
}

scale slider {
    background-color: #ffffff;
    border: 2px solid #ff1744;
    border-radius: 50%;
    min-width: 18px;
    min-height: 18px;
}
"#;

/// Loads and activates the CSS theme across the application display.
pub fn load_css_theme() {
    let provider = CssProvider::new();
    provider.load_from_data(APP_CSS);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
