//! Modern Apple & DaVinci-grade Color Picker Widget with Interactive HSV Wheel.
//!
//! Provides an ultra-responsive, continuous chromatic color wheel rendered via Cairo,
//! real-time RGB / HSV / HEX controls, curated palettes, and custom user swatches.

use gtk4::cairo::RadialGradient;
use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Align, Box as GtkBox, Button, ColorButton, CssProvider, DrawingArea, Entry,
    GestureClick, GestureDrag, Label, Orientation, Scale, SpinButton,
};
use std::cell::RefCell;
use std::f64::consts::TAU;
use std::rc::Rc;

use crate::domain::color::RgbColor;
use crate::effects::presets::ColorPreset;

/// Callback invoked when the user selects or tweaks a color.
pub type ColorChangedCallback = Rc<dyn Fn(RgbColor)>;

/// Interactive Color Picker component.
pub struct ColorPickerWidget {
    container: GtkBox,
    current_hsv: Rc<RefCell<(f64, f64, f64)>>,
    drawing_area: DrawingArea,
    hex_entry: Entry,
    val_scale: Scale,
    r_spin: SpinButton,
    g_spin: SpinButton,
    b_spin: SpinButton,
    #[allow(dead_code)]
    preview_box: GtkBox,
    preview_label: Label,
    preview_provider: CssProvider,
    preset_buttons: Vec<(RgbColor, Button)>,
    user_swatch_buttons: Vec<Button>,
    user_swatches: Rc<RefCell<Vec<RgbColor>>>,
    is_updating: Rc<RefCell<bool>>,
}

impl ColorPickerWidget {
    /// Builds a new `ColorPickerWidget`.
    pub fn new(on_color_changed: ColorChangedCallback) -> Self {
        let container = GtkBox::new(Orientation::Vertical, 10);
        container.set_hexpand(true);
        container.add_css_class("color-picker-container");

        let initial_color = RgbColor::MSI_RED;
        let (init_h, init_s, init_v) = initial_color.to_hsv();
        let current_hsv = Rc::new(RefCell::new((init_h, init_s, init_v)));
        let is_updating = Rc::new(RefCell::new(false));

        // Default user swatches (8 customizable slots)
        let user_swatches = Rc::new(RefCell::new(vec![
            RgbColor::MSI_RED,
            RgbColor::CYAN,
            RgbColor::MAGENTA,
            RgbColor::GREEN,
            RgbColor::BLUE,
            RgbColor::AMBER,
            RgbColor::PURPLE,
            RgbColor::WHITE,
        ]));

        // --- Main Top Split: Color Wheel (Left) + Fine Controls (Right) ---
        let main_row = GtkBox::new(Orientation::Horizontal, 14);
        main_row.set_halign(Align::Fill);
        main_row.set_valign(Align::Center);
        main_row.set_hexpand(true);

        // 1. Cairo Interactive Color Wheel
        let wheel_box = GtkBox::new(Orientation::Vertical, 0);
        wheel_box.set_valign(Align::Center);

        let drawing_area = DrawingArea::new();
        drawing_area.set_content_width(170);
        drawing_area.set_content_height(170);
        drawing_area.add_css_class("color-wheel-canvas");

        // Cairo draw function
        let hsv_draw = current_hsv.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let (h, s, v) = *hsv_draw.borrow();
            let cx = width as f64 / 2.0;
            let cy = height as f64 / 2.0;
            let radius = (cx.min(cy) - 6.0).max(10.0);

            // Subtle backdrop shadow
            cr.arc(cx, cy, radius + 2.0, 0.0, TAU);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            let _ = cr.fill();

            // Render smooth chromatic color disc using radial gradient sectors
            let segments = 100;
            for i in 0..segments {
                let a1 = (i as f64 / segments as f64) * TAU;
                let a2 = ((i + 1) as f64 / segments as f64) * TAU;
                let mid_angle = (a1 + a2) / 2.0;
                let mid_deg = (mid_angle.to_degrees() + 360.0).rem_euclid(360.0);

                let col_perimeter = RgbColor::from_hsv(mid_deg, 1.0, v);
                let col_center = RgbColor::from_hsv(mid_deg, 0.0, v);

                let (r1, g1, b1) = col_center.to_f64();
                let (r2, g2, b2) = col_perimeter.to_f64();

                let grad = RadialGradient::new(cx, cy, 0.0, cx, cy, radius);
                grad.add_color_stop_rgb(0.0, r1, g1, b1);
                grad.add_color_stop_rgb(1.0, r2, g2, b2);

                cr.move_to(cx, cy);
                cr.arc(cx, cy, radius, a1, a2);
                cr.close_path();
                let _ = cr.set_source(&grad);
                let _ = cr.fill();
            }

            // Outer metallic rim
            cr.arc(cx, cy, radius, 0.0, TAU);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.2);
            cr.set_line_width(1.2);
            let _ = cr.stroke();

            // Reticle Target Ring at current (H, S)
            let angle_rad = h.to_radians();
            let reticle_dist = s * radius;
            let rx = cx + reticle_dist * angle_rad.cos();
            let ry = cy + reticle_dist * angle_rad.sin();

            // Outer shadow ring
            cr.arc(rx, ry, 8.0, 0.0, TAU);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.6);
            cr.set_line_width(3.0);
            let _ = cr.stroke();

            // White crisp Apple-style ring
            cr.arc(rx, ry, 7.0, 0.0, TAU);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
            cr.set_line_width(1.8);
            let _ = cr.stroke();

            // Inner color core
            let cur_col = RgbColor::from_hsv(h, s, v);
            let (cr_r, cr_g, cr_b) = cur_col.to_f64();
            cr.arc(rx, ry, 5.0, 0.0, TAU);
            cr.set_source_rgb(cr_r, cr_g, cr_b);
            let _ = cr.fill();
        });

        wheel_box.append(&drawing_area);
        main_row.append(&wheel_box);

        // 2. Right Controls Column
        let controls_col = GtkBox::new(Orientation::Vertical, 8);
        controls_col.set_hexpand(true);
        controls_col.set_valign(Align::Center);

        // --- Preview Pill ---
        let preview_box = GtkBox::new(Orientation::Horizontal, 6);
        preview_box.add_css_class("color-preview-pill");
        preview_box.set_halign(Align::Fill);

        let preview_provider = CssProvider::new();
        preview_box
            .style_context()
            .add_provider(&preview_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let preview_label = Label::new(Some(&initial_color.to_hex()));
        preview_label.add_css_class("color-preview-text");
        preview_label.set_hexpand(true);
        preview_label.set_halign(Align::Center);
        preview_box.append(&preview_label);

        controls_col.append(&preview_box);

        // --- Brightness / Lightness (V) Slider ---
        let val_box = GtkBox::new(Orientation::Horizontal, 6);
        val_box.set_valign(Align::Center);

        let val_icon = Label::new(Some("☀ Light"));
        val_icon.add_css_class("zone-sublabel");
        val_icon.set_width_chars(6);
        val_box.append(&val_icon);

        let val_adj = Adjustment::new(100.0, 0.0, 100.0, 1.0, 5.0, 0.0);
        let val_scale = Scale::new(Orientation::Horizontal, Some(&val_adj));
        val_scale.set_digits(0);
        val_scale.set_draw_value(true);
        val_scale.set_hexpand(true);
        val_scale.set_value_pos(gtk4::PositionType::Right);
        val_box.append(&val_scale);

        controls_col.append(&val_box);

        // --- Hex & Native Chooser Row ---
        let hex_row = GtkBox::new(Orientation::Horizontal, 6);
        hex_row.set_valign(Align::Center);

        let hex_lbl = Label::new(Some("HEX:"));
        hex_lbl.add_css_class("zone-sublabel");
        hex_row.append(&hex_lbl);

        let hex_entry = Entry::new();
        hex_entry.set_max_length(7);
        hex_entry.set_width_chars(8);
        hex_entry.set_text(&initial_color.to_hex());
        hex_entry.set_placeholder_text(Some("#RRGGBB"));
        hex_entry.add_css_class("hex-entry");
        hex_row.append(&hex_entry);

        let color_btn = ColorButton::new();
        color_btn.set_title("System Color Picker");
        color_btn.set_rgba(&RGBA::new(1.0, 0.08, 0.15, 1.0));
        color_btn.set_tooltip_text(Some("OS Palette"));
        hex_row.append(&color_btn);

        controls_col.append(&hex_row);

        // --- RGB Spin Buttons Row ---
        let rgb_row = GtkBox::new(Orientation::Horizontal, 4);
        rgb_row.set_valign(Align::Center);

        let r_adj = Adjustment::new(initial_color.r as f64, 0.0, 255.0, 1.0, 10.0, 0.0);
        let r_spin = SpinButton::new(Some(&r_adj), 1.0, 0);
        r_spin.set_width_chars(3);
        r_spin.add_css_class("spin-channel");

        let g_adj = Adjustment::new(initial_color.g as f64, 0.0, 255.0, 1.0, 10.0, 0.0);
        let g_spin = SpinButton::new(Some(&g_adj), 1.0, 0);
        g_spin.set_width_chars(3);
        g_spin.add_css_class("spin-channel");

        let b_adj = Adjustment::new(initial_color.b as f64, 0.0, 255.0, 1.0, 10.0, 0.0);
        let b_spin = SpinButton::new(Some(&b_adj), 1.0, 0);
        b_spin.set_width_chars(3);
        b_spin.add_css_class("spin-channel");

        let r_lbl = Label::new(Some("R"));
        r_lbl.add_css_class("zone-sublabel");
        let g_lbl = Label::new(Some("G"));
        g_lbl.add_css_class("zone-sublabel");
        let b_lbl = Label::new(Some("B"));
        b_lbl.add_css_class("zone-sublabel");

        rgb_row.append(&r_lbl);
        rgb_row.append(&r_spin);
        rgb_row.append(&g_lbl);
        rgb_row.append(&g_spin);
        rgb_row.append(&b_lbl);
        rgb_row.append(&b_spin);

        controls_col.append(&rgb_row);

        main_row.append(&controls_col);
        container.append(&main_row);

        // --- Section: Curated Aesthetic Palette ---
        let palette_box = GtkBox::new(Orientation::Vertical, 4);
        let pal_title = Label::new(Some("STUDIO PALETTES"));
        pal_title.add_css_class("palette-section-title");
        pal_title.set_halign(Align::Start);
        palette_box.append(&pal_title);

        let presets_row = GtkBox::new(Orientation::Horizontal, 4);
        presets_row.set_halign(Align::Center);

        let mut preset_buttons = Vec::new();
        for preset in ColorPreset::PRESETS {
            let col = preset.color;
            let btn = Button::new();
            btn.add_css_class("color-circle");
            btn.set_tooltip_text(Some(preset.name));

            let provider = CssProvider::new();
            let css = format!(
                "button.color-circle {{ background-color: rgb({},{},{}); }}",
                col.r, col.g, col.b
            );
            provider.load_from_data(&css);
            btn.style_context()
                .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            presets_row.append(&btn);
            preset_buttons.push((col, btn));
        }
        palette_box.append(&presets_row);
        container.append(&palette_box);

        // --- Section: User Favorites Swatches ---
        let user_box = GtkBox::new(Orientation::Vertical, 4);
        let user_header = GtkBox::new(Orientation::Horizontal, 6);

        let user_title = Label::new(Some("MY FAVORITES"));
        user_title.add_css_class("palette-section-title");
        user_title.set_halign(Align::Start);
        user_header.append(&user_title);

        let spacer_u = GtkBox::new(Orientation::Horizontal, 0);
        spacer_u.set_hexpand(true);
        user_header.append(&spacer_u);

        let save_swatch_btn = Button::with_label("✚ Save");
        save_swatch_btn.add_css_class("btn-pill-small");
        user_header.append(&save_swatch_btn);
        user_box.append(&user_header);

        let user_row = GtkBox::new(Orientation::Horizontal, 4);
        user_row.set_halign(Align::Center);

        let mut user_swatch_buttons = Vec::new();
        for col in user_swatches.borrow().iter() {
            let btn = Button::new();
            btn.add_css_class("color-swatch-slot");

            let provider = CssProvider::new();
            let css = format!(
                "button.color-swatch-slot {{ background-color: rgb({},{},{}); }}",
                col.r, col.g, col.b
            );
            provider.load_from_data(&css);
            btn.style_context()
                .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            user_row.append(&btn);
            user_swatch_buttons.push(btn);
        }
        user_box.append(&user_row);
        container.append(&user_box);

        let widget = Self {
            container,
            current_hsv,
            drawing_area,
            hex_entry,
            val_scale,
            r_spin,
            g_spin,
            b_spin,
            preview_box,
            preview_label,
            preview_provider,
            preset_buttons,
            user_swatch_buttons,
            user_swatches,
            is_updating,
        };

        widget.setup_wheel_gestures(on_color_changed.clone());
        widget.setup_controls_signals(on_color_changed, color_btn, save_swatch_btn);
        widget.sync_ui_from_hsv();

        widget
    }

    /// Connects click and drag gesture recognizers to the Cairo Color Wheel.
    fn setup_wheel_gestures(&self, on_color_changed: ColorChangedCallback) {
        let current_hsv = self.current_hsv.clone();
        let is_updating = self.is_updating.clone();
        let da_weak = self.drawing_area.downgrade();
        let hex_weak = self.hex_entry.downgrade();
        let r_weak = self.r_spin.downgrade();
        let g_weak = self.g_spin.downgrade();
        let b_weak = self.b_spin.downgrade();
        let prev_lbl_weak = self.preview_label.downgrade();
        let prev_prov = self.preview_provider.clone();

        let update_from_pos = {
            let cb = on_color_changed;
            move |x: f64, y: f64, width: f64, height: f64| {
                if *is_updating.borrow() {
                    return;
                }
                let cx = width / 2.0;
                let cy = height / 2.0;
                let radius = (cx.min(cy) - 8.0).max(10.0);

                let dx = x - cx;
                let dy = y - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let sat = (dist / radius).clamp(0.0, 1.0);

                let angle_rad = dy.atan2(dx);
                let hue = (angle_rad.to_degrees() + 360.0).rem_euclid(360.0);

                let (_, _, val) = *current_hsv.borrow();
                *current_hsv.borrow_mut() = (hue, sat, val);

                let col = RgbColor::from_hsv(hue, sat, val);

                // Update UI widgets safely without triggering recursive events
                *is_updating.borrow_mut() = true;

                if let Some(h_entry) = hex_weak.upgrade() {
                    h_entry.set_text(&col.to_hex());
                }
                if let Some(r) = r_weak.upgrade() {
                    r.set_value(col.r as f64);
                }
                if let Some(g) = g_weak.upgrade() {
                    g.set_value(col.g as f64);
                }
                if let Some(b) = b_weak.upgrade() {
                    b.set_value(col.b as f64);
                }
                if let Some(p_lbl) = prev_lbl_weak.upgrade() {
                    p_lbl.set_text(&col.to_hex());
                }

                let (fg_r, fg_g, fg_b) = if col.relative_luminance() > 0.45 {
                    (18, 18, 24)
                } else {
                    (255, 255, 255)
                };
                let css = format!(
                    ".color-preview-pill {{ background-color: rgb({},{},{}); color: rgb({},{},{}); }}",
                    col.r, col.g, col.b, fg_r, fg_g, fg_b
                );
                prev_prov.load_from_data(&css);

                if let Some(da) = da_weak.upgrade() {
                    da.queue_draw();
                }

                *is_updating.borrow_mut() = false;
                cb(col);
            }
        };

        // Click Gesture
        let click = GestureClick::new();
        let da_for_click = self.drawing_area.clone();
        let update_click = update_from_pos.clone();
        click.connect_pressed(move |_gesture, _n, x, y| {
            let w = da_for_click.width() as f64;
            let h = da_for_click.height() as f64;
            update_click(x, y, w, h);
        });
        self.drawing_area.add_controller(click);

        // Drag Gesture
        let drag = GestureDrag::new();
        let da_for_drag = self.drawing_area.clone();
        let update_drag = update_from_pos;
        drag.connect_drag_update(move |gesture, offset_x, offset_y| {
            if let Some((start_x, start_y)) = gesture.start_point() {
                let cur_x = start_x + offset_x;
                let cur_y = start_y + offset_y;
                let w = da_for_drag.width() as f64;
                let h = da_for_drag.height() as f64;
                update_drag(cur_x, cur_y, w, h);
            }
        });
        self.drawing_area.add_controller(drag);
    }

    /// Sets up input controls, presets, and custom swatch handlers.
    fn setup_controls_signals(
        &self,
        on_color_changed: ColorChangedCallback,
        color_btn: ColorButton,
        save_swatch_btn: Button,
    ) {
        let current_hsv = self.current_hsv.clone();
        let is_updating = self.is_updating.clone();
        let da_weak = self.drawing_area.downgrade();
        let hex_weak = self.hex_entry.downgrade();
        let r_weak = self.r_spin.downgrade();
        let g_weak = self.g_spin.downgrade();
        let b_weak = self.b_spin.downgrade();
        let prev_lbl_weak = self.preview_label.downgrade();
        let prev_prov = self.preview_provider.clone();

        // Helper to update full UI when a new RgbColor is chosen
        let apply_new_color = {
            let current_hsv = current_hsv.clone();
            let is_updating = is_updating.clone();
            let cb = on_color_changed;
            move |col: RgbColor| {
                let (h, s, v) = col.to_hsv();
                *current_hsv.borrow_mut() = (h, s, v);

                *is_updating.borrow_mut() = true;
                if let Some(h_entry) = hex_weak.upgrade() {
                    h_entry.set_text(&col.to_hex());
                }
                if let Some(r) = r_weak.upgrade() {
                    r.set_value(col.r as f64);
                }
                if let Some(g) = g_weak.upgrade() {
                    g.set_value(col.g as f64);
                }
                if let Some(b) = b_weak.upgrade() {
                    b.set_value(col.b as f64);
                }
                if let Some(p_lbl) = prev_lbl_weak.upgrade() {
                    p_lbl.set_text(&col.to_hex());
                }

                let (fg_r, fg_g, fg_b) = if col.relative_luminance() > 0.45 {
                    (18, 18, 24)
                } else {
                    (255, 255, 255)
                };
                let css = format!(
                    ".color-preview-pill {{ background-color: rgb({},{},{}); color: rgb({},{},{}); }}",
                    col.r, col.g, col.b, fg_r, fg_g, fg_b
                );
                prev_prov.load_from_data(&css);

                if let Some(da) = da_weak.upgrade() {
                    da.queue_draw();
                }
                *is_updating.borrow_mut() = false;
                cb(col);
            }
        };

        // 1. Value / Lightness Slider
        let apply_val = apply_new_color.clone();
        let hsv_val = current_hsv.clone();
        let is_up_val = is_updating.clone();
        self.val_scale.connect_value_changed(move |scale| {
            if *is_up_val.borrow() {
                return;
            }
            let new_v = (scale.value() / 100.0).clamp(0.0, 1.0);
            let (h, s, _) = *hsv_val.borrow();
            *hsv_val.borrow_mut() = (h, s, new_v);
            let col = RgbColor::from_hsv(h, s, new_v);
            apply_val(col);
        });

        // 2. Hex text entry
        let apply_hex = apply_new_color.clone();
        let is_up_hex = is_updating.clone();
        self.hex_entry.connect_changed(move |entry| {
            if *is_up_hex.borrow() {
                return;
            }
            let text = entry.text();
            if let Ok(parsed) = text.parse::<RgbColor>() {
                apply_hex(parsed);
            }
        });

        // 3. RGB Spin Buttons
        let apply_rgb = apply_new_color.clone();
        let is_up_rgb = is_updating.clone();
        let r_sp = self.r_spin.clone();
        let g_sp = self.g_spin.clone();
        let b_sp = self.b_spin.clone();

        let handle_rgb_spin = move || {
            if *is_up_rgb.borrow() {
                return;
            }
            let r = r_sp.value().round() as u8;
            let g = g_sp.value().round() as u8;
            let b = b_sp.value().round() as u8;
            apply_rgb(RgbColor::new(r, g, b));
        };

        let h_r = handle_rgb_spin.clone();
        self.r_spin.connect_value_changed(move |_| h_r());
        let h_g = handle_rgb_spin.clone();
        self.g_spin.connect_value_changed(move |_| h_g());
        let h_b = handle_rgb_spin;
        self.b_spin.connect_value_changed(move |_| h_b());

        // 4. GTK ColorButton dialog
        let apply_dialog = apply_new_color.clone();
        color_btn.connect_color_set(move |btn| {
            let rgba = btn.rgba();
            let col =
                RgbColor::from_f64(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64);
            apply_dialog(col);
        });

        // 5. Curated Presets
        for (preset_col, btn) in &self.preset_buttons {
            let apply_p = apply_new_color.clone();
            let c = *preset_col;
            btn.connect_clicked(move |_| {
                apply_p(c);
            });
        }

        // 6. User Swatch slots
        for (i, btn) in self.user_swatch_buttons.iter().enumerate() {
            let apply_s = apply_new_color.clone();
            let swatches = self.user_swatches.clone();
            btn.connect_clicked(move |_| {
                if let Some(col) = swatches.borrow().get(i).copied() {
                    apply_s(col);
                }
            });
        }

        // 7. Save Current Color to User Swatches
        let swatches_save = self.user_swatches.clone();
        let hsv_save = current_hsv;
        let swatch_btns = self.user_swatch_buttons.clone();
        save_swatch_btn.connect_clicked(move |_| {
            let (h, s, v) = *hsv_save.borrow();
            let current_col = RgbColor::from_hsv(h, s, v);

            let mut sw = swatches_save.borrow_mut();
            if !sw.is_empty() {
                sw.remove(0);
                sw.push(current_col);
            }

            for (idx, btn) in swatch_btns.iter().enumerate() {
                if let Some(col) = sw.get(idx) {
                    let provider = CssProvider::new();
                    let css = format!(
                        "button.color-swatch-slot {{ background-color: rgb({},{},{}); }}",
                        col.r, col.g, col.b
                    );
                    provider.load_from_data(&css);
                    btn.style_context()
                        .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                }
            }
        });
    }

    /// Refreshes all child widgets to reflect internal HSV state.
    fn sync_ui_from_hsv(&self) {
        *self.is_updating.borrow_mut() = true;

        let (h, s, v) = *self.current_hsv.borrow();
        let col = RgbColor::from_hsv(h, s, v);

        self.val_scale.set_value(v * 100.0);
        self.hex_entry.set_text(&col.to_hex());
        self.r_spin.set_value(col.r as f64);
        self.g_spin.set_value(col.g as f64);
        self.b_spin.set_value(col.b as f64);
        self.preview_label.set_text(&col.to_hex());

        let (fg_r, fg_g, fg_b) = if col.relative_luminance() > 0.45 {
            (18, 18, 24)
        } else {
            (255, 255, 255)
        };
        let css = format!(
            ".color-preview-pill {{ background-color: rgb({},{},{}); color: rgb({},{},{}); }}",
            col.r, col.g, col.b, fg_r, fg_g, fg_b
        );
        self.preview_provider.load_from_data(&css);

        self.drawing_area.queue_draw();

        *self.is_updating.borrow_mut() = false;
    }

    /// Returns the GTK container widget.
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Updates the currently displayed color from external state.
    pub fn set_color(&self, color: RgbColor) {
        let (h, s, v) = color.to_hsv();
        *self.current_hsv.borrow_mut() = (h, s, v);
        self.sync_ui_from_hsv();

        // Highlight preset if matching
        for (preset_col, btn) in &self.preset_buttons {
            if *preset_col == color {
                btn.add_css_class("selected");
            } else {
                btn.remove_css_class("selected");
            }
        }
    }
}
