//! Modern Apple & DaVinci-grade GTK4 Theme and styling.
//!
//! Provides ultra-refined glassmorphism, frosted translucent cards,
//! Apple-style segmented controls, luminescent neon accents (#FF2E55, #00F0FF),
//! and high-DPI crisp typography.

use gtk4::CssProvider;
use gtk4::gdk::Display;

/// Full CSS stylesheet for the application.
pub const APP_CSS: &str = r#"
/* ===== Global Window & Palette ===== */
window.main-window {
    background: radial-gradient(circle at 50% 0%, #151A26 0%, #0A0D14 100%);
    color: #F0F4F8;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Segoe UI", Roboto, sans-serif;
    letter-spacing: -0.2px;
}

headerbar {
    background: rgba(14, 18, 28, 0.85);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    min-height: 52px;
    padding: 0 16px;
}

headerbar .title {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: 0.8px;
    color: #FFFFFF;
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
}

headerbar .subtitle {
    font-size: 11px;
    font-weight: 500;
    color: #8E9BAE;
    letter-spacing: 0.2px;
}

/* ===== Frosted Glass Cards ===== */
.msi-card {
    background: rgba(18, 24, 38, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 14px;
    padding: 12px 16px;
    margin: 2px 0;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
}

.msi-card-header {
    font-size: 11.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    color: #FF3355;
    margin-bottom: 6px;
}

.palette-section-title {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.9px;
    color: #8E9BAE;
    margin-top: 3px;
    margin-bottom: 3px;
}

/* ===== Status Pill Indicator ===== */
.status-pill {
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.2px;
}

.status-pill.connected {
    background: rgba(46, 204, 113, 0.15);
    border: 1px solid rgba(46, 204, 113, 0.6);
    color: #2ECC71;
    box-shadow: 0 0 10px rgba(46, 204, 113, 0.25);
}

.status-pill.connecting {
    background: rgba(241, 196, 15, 0.15);
    border: 1px solid rgba(241, 196, 15, 0.6);
    color: #F1C40F;
    box-shadow: 0 0 10px rgba(241, 196, 15, 0.25);
}

.status-pill.disconnected {
    background: rgba(231, 76, 60, 0.15);
    border: 1px solid rgba(231, 76, 60, 0.6);
    color: #FF5E57;
    box-shadow: 0 0 10px rgba(231, 76, 60, 0.25);
}

/* ===== Keyboard Visualizer Deck ===== */
.keyboard-frame {
    background: #090C12;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 10px 12px;
    box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.8);
}

.keyboard-zone-box {
    border-radius: 10px;
    padding: 8px 6px;
    margin: 3px;
    transition: all 180ms cubic-bezier(0.4, 0, 0.2, 1);
}

.keyboard-zone-box:hover {
    border-color: rgba(255, 255, 255, 0.6);
    transform: translateY(-1px);
}

.zone-label {
    font-size: 11px;
    font-weight: 700;
    color: #FFFFFF;
    letter-spacing: 0.5px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.9);
}

.zone-sublabel {
    font-size: 10px;
    font-weight: 500;
    color: rgba(220, 230, 245, 0.75);
    letter-spacing: 0.2px;
}

/* Key Cap Grid Simulation */
.keycap {
    background: rgba(24, 30, 44, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    min-width: 26px;
    min-height: 20px;
    padding: 2px 4px;
    margin: 1px;
    font-size: 8.5px;
    font-weight: 700;
    color: #E2E8F0;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.keycap-wasd {
    background: rgba(255, 30, 70, 0.35);
    border: 1px solid rgba(255, 60, 95, 0.85);
    color: #FFFFFF;
    box-shadow: 0 0 6px rgba(255, 30, 70, 0.5);
}

/* ===== Effect Chips & Mode Buttons ===== */
.effect-chip {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #D1D9E6;
    border-radius: 8px;
    padding: 6px 12px;
    font-weight: 600;
    font-size: 11.5px;
    transition: all 140ms ease;
}

.effect-chip:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #FFFFFF;
}

.effect-chip.active {
    background: rgba(255, 23, 68, 0.22);
    border: 1px solid #FF1744;
    color: #FFFFFF;
    box-shadow: 0 0 12px rgba(255, 23, 68, 0.45);
}

/* ===== Color Wheel Canvas & Controls ===== */
.color-wheel-canvas {
    border-radius: 50%;
    margin: 2px;
}

.color-preview-pill {
    border-radius: 8px;
    min-height: 32px;
    padding: 4px 12px;
    border: 1.5px solid rgba(255, 255, 255, 0.25);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    transition: all 120ms ease;
}

.color-preview-text {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.9px;
}

.hex-entry {
    background: rgba(14, 18, 28, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    color: #FFFFFF;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, monospace;
    font-weight: 700;
    padding: 3px 8px;
    font-size: 11.5px;
}

.hex-entry:focus {
    border-color: #FF1744;
    box-shadow: 0 0 6px rgba(255, 23, 68, 0.35);
}

.spin-channel {
    background: rgba(14, 18, 28, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: #FFFFFF;
    font-weight: 600;
    font-size: 10.5px;
}

/* Color Circles / Swatches */
.color-circle {
    min-width: 26px;
    min-height: 26px;
    border-radius: 50%;
    border: 1.5px solid rgba(255, 255, 255, 0.2);
    margin: 2px;
    transition: all 120ms cubic-bezier(0.34, 1.56, 0.64, 1);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
}

.color-circle:hover {
    transform: scale(1.2);
    border-color: #FFFFFF;
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.6);
}

.color-circle.selected {
    border: 2px solid #FFFFFF;
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.8);
    transform: scale(1.15);
}

.color-swatch-slot {
    min-width: 24px;
    min-height: 24px;
    border-radius: 6px;
    border: 1.5px solid rgba(255, 255, 255, 0.2);
    margin: 2px;
    transition: all 120ms ease;
}

.color-swatch-slot:hover {
    transform: scale(1.18);
    border-color: #FFFFFF;
}

/* ===== Primary & Secondary Action Buttons ===== */
.btn-primary {
    background: linear-gradient(135deg, #FF1744 0%, #D50000 100%);
    color: #FFFFFF;
    font-weight: 700;
    font-size: 12.5px;
    border-radius: 8px;
    padding: 8px 20px;
    border: none;
    box-shadow: 0 3px 12px rgba(255, 23, 68, 0.45);
    transition: all 150ms ease;
    letter-spacing: 0.2px;
}

.btn-primary:hover {
    background: linear-gradient(135deg, #FF4569 0%, #FF1744 100%);
    box-shadow: 0 6px 22px rgba(255, 23, 68, 0.65);
    transform: translateY(-1px);
}

.btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #E2E8F0;
    border-radius: 10px;
    padding: 8px 18px;
    font-weight: 600;
    font-size: 12.5px;
    transition: all 140ms ease;
}

.btn-secondary:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #FFFFFF;
}

.btn-pill-small {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #CAD5E2;
    border-radius: 16px;
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 600;
    transition: all 120ms ease;
}

.btn-pill-small:hover {
    background: rgba(255, 23, 68, 0.25);
    border-color: #FF1744;
    color: #FFFFFF;
}

/* ===== Sliders (Scale) ===== */
scale highlight {
    background: linear-gradient(90deg, #FF1744 0%, #FF4569 100%);
    border-radius: 6px;
}

scale trough {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    min-height: 6px;
}

scale slider {
    background: #FFFFFF;
    border: 2px solid #FF1744;
    border-radius: 50%;
    min-width: 18px;
    min-height: 18px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
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
