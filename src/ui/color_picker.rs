//! Fast color picker widget with preset palette, hex entry, and GTK color chooser button.

use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, ColorButton, CssProvider, Entry, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

use crate::domain::color::RgbColor;
use crate::effects::ColorPreset;

/// Callback for color selection.
pub type ColorChangedCallback = Rc<dyn Fn(RgbColor)>;

/// Color selector component.
pub struct ColorPickerWidget {
    container: GtkBox,
    hex_entry: Entry,
    current_color: Rc<RefCell<RgbColor>>,
    preset_buttons: Vec<(RgbColor, Button, CssProvider)>,
}

impl ColorPickerWidget {
    /// Builds a new `ColorPickerWidget`.
    pub fn new(on_color_changed: ColorChangedCallback) -> Self {
        let container = GtkBox::new(Orientation::Vertical, 10);
        container.set_hexpand(true);

        let current_color = Rc::new(RefCell::new(RgbColor::MSI_RED));

        // 1. Preset circular buttons row
        let presets_row = GtkBox::new(Orientation::Horizontal, 6);
        presets_row.set_halign(gtk4::Align::Center);

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

            let cb = on_color_changed.clone();
            let current = current_color.clone();
            btn.connect_clicked(move |_| {
                *current.borrow_mut() = col;
                cb(col);
            });

            presets_row.append(&btn);
            preset_buttons.push((col, btn, provider));
        }

        container.append(&presets_row);

        // 2. Custom Hex & GTK Color Dialog row
        let custom_row = GtkBox::new(Orientation::Horizontal, 10);
        custom_row.set_halign(gtk4::Align::Center);

        let hex_lbl = Label::new(Some("HEX:"));
        hex_lbl.add_css_class("zone-sublabel");
        custom_row.append(&hex_lbl);

        let hex_entry = Entry::new();
        hex_entry.set_max_length(7);
        hex_entry.set_width_chars(8);
        hex_entry.set_text(&RgbColor::MSI_RED.to_hex());
        hex_entry.set_placeholder_text(Some("#RRGGBB"));

        let cb_hex = on_color_changed.clone();
        let current_hex = current_color.clone();
        hex_entry.connect_changed(move |entry| {
            let text = entry.text();
            if let Ok(parsed) = text.parse::<RgbColor>() {
                *current_hex.borrow_mut() = parsed;
                cb_hex(parsed);
            }
        });
        custom_row.append(&hex_entry);

        // GTK ColorButton
        let color_btn = ColorButton::new();
        color_btn.set_title("Choose Custom RGB Color");
        color_btn.set_rgba(&RGBA::new(1.0, 0.08, 0.15, 1.0));

        let cb_dialog = on_color_changed.clone();
        let current_dialog = current_color.clone();
        color_btn.connect_color_set(move |btn| {
            let rgba = btn.rgba();
            let col =
                RgbColor::from_f64(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64);
            *current_dialog.borrow_mut() = col;
            cb_dialog(col);
        });
        custom_row.append(&color_btn);

        container.append(&custom_row);

        Self {
            container,
            hex_entry,
            current_color,
            preset_buttons,
        }
    }

    /// Returns the GTK container widget.
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Updates the active color displayed in the picker.
    pub fn set_color(&self, color: RgbColor) {
        *self.current_color.borrow_mut() = color;

        let hex = color.to_hex();
        if self.hex_entry.text() != hex {
            self.hex_entry.set_text(&hex);
        }

        // Highlight matching preset button
        for (preset_col, btn, _) in &self.preset_buttons {
            if *preset_col == color {
                btn.add_css_class("selected");
            } else {
                btn.remove_css_class("selected");
            }
        }
    }
}
