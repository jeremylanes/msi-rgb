//! Interactive 4-Zone MSI Katana Keyboard visual preview widget.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, CssProvider, Label, Orientation};
use std::rc::Rc;

use crate::domain::device::RgbState;
use crate::domain::zone::ZoneId;

/// Callback for when the user clicks a zone on the visual preview.
pub type ZoneClickCallback = Rc<dyn Fn(ZoneId)>;

/// Widget representing the 4-zone MSI laptop keyboard.
pub struct KeyboardPreview {
    container: GtkBox,
    #[allow(dead_code)]
    zone_buttons: [Button; 4],
    zone_providers: [CssProvider; 4],
}

impl KeyboardPreview {
    /// Builds a new `KeyboardPreview` widget.
    pub fn new(on_zone_clicked: ZoneClickCallback) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 10);
        container.add_css_class("keyboard-frame");
        container.set_homogeneous(true);
        container.set_hexpand(true);

        let zone_configs: [(ZoneId, &str, &str, &[&[&str]]); 4] = [
            (
                ZoneId::Zone1,
                "ZONE 1",
                "WASD & Gaming Left",
                &[
                    &["ESC", "1", "2", "3"],
                    &["TAB", "Q", "W", "E"],
                    &["CAPS", "A", "S", "D"],
                    &["SHIFT", "Z", "X", "C"],
                ],
            ),
            (
                ZoneId::Zone2,
                "ZONE 2",
                "Center Alpha",
                &[
                    &["4", "5", "6", "7"],
                    &["R", "T", "Y", "U"],
                    &["F", "G", "H", "J"],
                    &["V", "B", "N", "M"],
                ],
            ),
            (
                ZoneId::Zone3,
                "ZONE 3",
                "Navigation & Return",
                &[
                    &["8", "9", "0", "-"],
                    &["I", "O", "P", "["],
                    &["K", "L", ";", "ENTER"],
                    &[",", ".", "/", "▲"],
                ],
            ),
            (
                ZoneId::Zone4,
                "ZONE 4",
                "Number Pad",
                &[
                    &["NUM", "/", "*", "-"],
                    &["7", "8", "9", "+"],
                    &["4", "5", "6", "ENT"],
                    &["1", "2", "3", "0"],
                ],
            ),
        ];

        let mut zone_buttons = Vec::new();
        let mut zone_providers = Vec::new();

        for (zone_id, title, desc, key_rows) in zone_configs {
            let btn = Button::new();
            btn.add_css_class("keyboard-zone-box");
            btn.set_hexpand(true);

            let provider = CssProvider::new();
            btn.style_context()
                .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            let inner_box = GtkBox::new(Orientation::Vertical, 4);
            inner_box.set_valign(Align::Center);
            inner_box.set_halign(Align::Fill);

            let header_row = GtkBox::new(Orientation::Horizontal, 4);
            header_row.set_halign(Align::Center);

            let title_lbl = Label::new(Some(title));
            title_lbl.add_css_class("zone-label");
            header_row.append(&title_lbl);
            inner_box.append(&header_row);

            let desc_lbl = Label::new(Some(desc));
            desc_lbl.add_css_class("zone-sublabel");
            desc_lbl.set_halign(Align::Center);
            inner_box.append(&desc_lbl);

            // Realistic 4x4 Mini Key Matrix
            let matrix_box = GtkBox::new(Orientation::Vertical, 3);
            matrix_box.set_halign(Align::Fill);
            matrix_box.set_hexpand(true);
            matrix_box.set_margin_top(4);
            matrix_box.set_margin_start(6);
            matrix_box.set_margin_end(6);

            for row in key_rows {
                let row_box = GtkBox::new(Orientation::Horizontal, 3);
                row_box.set_homogeneous(true);
                row_box.set_hexpand(true);
                for key in *row {
                    let key_lbl = Label::new(Some(*key));
                    key_lbl.add_css_class("keycap");
                    key_lbl.set_hexpand(true);
                    if *key == "W" || *key == "A" || *key == "S" || *key == "D" {
                        key_lbl.add_css_class("keycap-wasd");
                    }
                    row_box.append(&key_lbl);
                }
                matrix_box.append(&row_box);
            }

            inner_box.append(&matrix_box);
            btn.set_child(Some(&inner_box));

            let cb = on_zone_clicked.clone();
            btn.connect_clicked(move |_| {
                cb(zone_id);
            });

            container.append(&btn);
            zone_buttons.push(btn);
            zone_providers.push(provider);
        }

        Self {
            container,
            zone_buttons: zone_buttons
                .try_into()
                .unwrap_or_else(|_| panic!("4 buttons")),
            zone_providers: zone_providers
                .try_into()
                .unwrap_or_else(|_| panic!("4 providers")),
        }
    }

    /// Returns the GTK container widget.
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Updates the dynamic glow styling for all 4 zones.
    pub fn update_state(&self, state: &RgbState, selected_zone: Option<ZoneId>) {
        for (i, zone_id) in ZoneId::ALL.iter().enumerate() {
            let zone_config = state.zone(*zone_id);
            let color = zone_config.effective_color();
            let is_selected = selected_zone == Some(*zone_id)
                || (selected_zone.is_none() && state.sync_all_zones);

            let css = if color.is_off() {
                format!(
                    "button.keyboard-zone-box {{ background: #0E121B; border: 1.5px solid #1E2433; {} }}",
                    if is_selected {
                        "border-color: #FF2E55; box-shadow: 0 0 16px rgba(255, 46, 85, 0.45);"
                    } else {
                        ""
                    }
                )
            } else {
                let (r, g, b) = (color.r, color.g, color.b);
                let bg_alpha = if is_selected { 0.38 } else { 0.18 };
                let border_alpha = if is_selected { 0.95 } else { 0.45 };
                let glow_blur = if is_selected { 18 } else { 10 };
                let spread = if is_selected { 2 } else { 0 };

                format!(
                    "button.keyboard-zone-box {{ background: linear-gradient(180deg, rgba({r},{g},{b},{bg_alpha}) 0%, rgba(14,18,27,0.92) 100%); border: 1.5px solid rgba({r},{g},{b},{border_alpha}); box-shadow: 0 0 {glow_blur}px {spread}px rgba({r},{g},{b},0.45), inset 0 1px 0 rgba(255,255,255,0.15); }}"
                )
            };

            self.zone_providers[i].load_from_data(&css);
        }
    }
}
