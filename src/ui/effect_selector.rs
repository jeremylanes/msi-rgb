//! Effect mode selector widget.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Orientation};
use std::rc::Rc;

use crate::domain::effect::EffectType;

/// Callback type for effect mode selection.
pub type EffectSelectedCallback = Rc<dyn Fn(EffectType)>;

/// Effect mode chip selection row widget.
pub struct EffectSelector {
    container: GtkBox,
    buttons: Vec<(EffectType, Button)>,
}

impl EffectSelector {
    /// Creates a new `EffectSelector` widget.
    pub fn new(on_effect_selected: EffectSelectedCallback) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_homogeneous(true);
        container.set_hexpand(true);

        let effects = [
            (EffectType::Static, "💡 Static"),
            (EffectType::Breathing, "🌊 Breathing"),
            (EffectType::ColorCycle, "🔄 Color Cycle"),
            (EffectType::Wave, "⚡ Dynamic Wave"),
            (EffectType::Off, "🛑 Stealth Off"),
        ];

        let mut buttons = Vec::new();

        for (effect, label) in effects {
            let btn = Button::with_label(label);
            btn.add_css_class("effect-chip");
            btn.set_hexpand(true);

            let cb = on_effect_selected.clone();
            btn.connect_clicked(move |_| {
                cb(effect);
            });

            container.append(&btn);
            buttons.push((effect, btn));
        }

        Self { container, buttons }
    }

    /// Returns the GTK container widget.
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Updates active effect highlight.
    pub fn set_active_effect(&self, active: EffectType) {
        for (eff, btn) in &self.buttons {
            if *eff == active {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }
    }
}
