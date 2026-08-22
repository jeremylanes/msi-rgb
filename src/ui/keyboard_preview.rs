//! Interactive 4-Zone MSI Katana Keyboard visual preview widget.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, CssProvider, Label, Orientation};
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
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.add_css_class("keyboard-frame");
        container.set_homogeneous(true);
        container.set_hexpand(true);

        let zone_configs: [(ZoneId, &str, &str, &[&str]); 4] = [
            (
                ZoneId::Zone1,
                "ZONE 1",
                "WASD & Left",
                &["ESC", "TAB", "W", "A", "S", "D", "SHIFT", "CTRL"],
            ),
            (
                ZoneId::Zone2,
                "ZONE 2",
                "Center Left",
                &["T", "Y", "G", "H", "B", "V", "SPACE", "ALT"],
            ),
            (
                ZoneId::Zone3,
                "ZONE 3",
                "Center Right",
                &["U", "I", "O", "P", "J", "K", "ENTER", "ARROWS"],
            ),
            (
                ZoneId::Zone4,
                "ZONE 4",
                "Numpad",
                &["NUM", "/", "*", "7", "8", "9", "4", "5", "6", "ENTER"],
            ),
        ];

        let mut zone_buttons = Vec::new();
        let mut zone_providers = Vec::new();

        for (zone_id, title, desc, sample_keys) in zone_configs {
            let btn = Button::new();
            btn.add_css_class("keyboard-zone-box");
            btn.set_hexpand(true);

            let provider = CssProvider::new();
            btn.style_context()
                .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            let inner_box = GtkBox::new(Orientation::Vertical, 6);
            inner_box.set_valign(gtk4::Align::Center);
            inner_box.set_halign(gtk4::Align::Fill);

            let title_lbl = Label::new(Some(title));
            title_lbl.add_css_class("zone-label");
            inner_box.append(&title_lbl);

            let desc_lbl = Label::new(Some(desc));
            desc_lbl.add_css_class("zone-sublabel");
            inner_box.append(&desc_lbl);

            // Mini key grid visualization
            let key_grid = GtkBox::new(Orientation::Horizontal, 2);
            key_grid.set_halign(gtk4::Align::Center);
            for key in &sample_keys[0..sample_keys.len().min(4)] {
                let key_lbl = Label::new(Some(*key));
                key_lbl.add_css_class("keycap");
                key_grid.append(&key_lbl);
            }
            inner_box.append(&key_grid);

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
                    "button.keyboard-zone-box {{ background-color: #12161f; border-color: #21262d; {} }}",
                    if is_selected {
                        "border-color: #ff3355; box-shadow: 0 0 10px rgba(255, 51, 85, 0.4);"
                    } else {
                        ""
                    }
                )
            } else {
                let (r, g, b) = (color.r, color.g, color.b);
                let bg_alpha = if is_selected { 0.35 } else { 0.20 };
                let border_alpha = if is_selected { 1.0 } else { 0.6 };
                let glow_blur = if is_selected { 14 } else { 8 };

                format!(
                    "button.keyboard-zone-box {{ background: linear-gradient(180deg, rgba({r},{g},{b},{bg_alpha}) 0%, rgba(18,22,31,0.9) 100%); border: 2px solid rgba({r},{g},{b},{border_alpha}); box-shadow: 0 0 {glow_blur}px rgba({r},{g},{b},0.5); }}"
                )
            };

            self.zone_providers[i].load_from_data(&css);
        }
    }
}
