//! Zone configuration and parameter adjustment controls.

use gtk4::prelude::*;
use gtk4::{Adjustment, Box as GtkBox, Button, DropDown, Label, Orientation, Scale, StringList};
use std::cell::RefCell;
use std::rc::Rc;

use crate::domain::effect::{Brightness, Direction, EffectType, Speed};
use crate::domain::zone::ZoneId;

/// Callbacks for parameter changes.
#[derive(Clone)]
pub struct ZoneControlCallbacks {
    pub on_tab_selected: Rc<dyn Fn(Option<ZoneId>)>,
    pub on_brightness_changed: Rc<dyn Fn(Brightness)>,
    pub on_speed_changed: Rc<dyn Fn(Speed)>,
    pub on_direction_changed: Rc<dyn Fn(Direction)>,
}

/// Zone configuration panel.
pub struct ZoneControlPanel {
    container: GtkBox,
    zone_tab_buttons: Vec<(Option<ZoneId>, Button)>,
    brightness_scale: Scale,
    speed_scale: Scale,
    speed_box: GtkBox,
    direction_dropdown: DropDown,
    direction_box: GtkBox,
    is_updating: Rc<RefCell<bool>>,
}

impl ZoneControlPanel {
    /// Creates a new `ZoneControlPanel`.
    pub fn new(callbacks: ZoneControlCallbacks) -> Self {
        let container = GtkBox::new(Orientation::Vertical, 8);
        let is_updating = Rc::new(RefCell::new(false));

        // 1. Zone Selection Tabs
        let tabs_row = GtkBox::new(Orientation::Horizontal, 4);
        tabs_row.set_homogeneous(true);

        let tabs = [
            (None, "⚡ All Zones"),
            (Some(ZoneId::Zone1), "Z1 (WASD)"),
            (Some(ZoneId::Zone2), "Zone 2"),
            (Some(ZoneId::Zone3), "Zone 3"),
            (Some(ZoneId::Zone4), "Zone 4"),
        ];

        let mut zone_tab_buttons = Vec::new();

        for (zone_opt, label) in tabs {
            let btn = Button::with_label(label);
            btn.add_css_class("effect-chip");

            let cb = callbacks.on_tab_selected.clone();
            btn.connect_clicked(move |_| {
                cb(zone_opt);
            });

            tabs_row.append(&btn);
            zone_tab_buttons.push((zone_opt, btn));
        }

        container.append(&tabs_row);

        // 2. Sliders Grid: Brightness & Speed
        let sliders_row = GtkBox::new(Orientation::Horizontal, 10);
        sliders_row.set_homogeneous(true);

        // Brightness
        let brightness_box = GtkBox::new(Orientation::Vertical, 2);
        let brt_label = Label::new(Some("BRIGHTNESS"));
        brt_label.add_css_class("msi-card-header");
        brt_label.set_halign(gtk4::Align::Start);
        brightness_box.append(&brt_label);

        let brt_adj = Adjustment::new(100.0, 0.0, 100.0, 5.0, 10.0, 0.0);
        let brightness_scale = Scale::new(Orientation::Horizontal, Some(&brt_adj));
        brightness_scale.set_digits(0);
        brightness_scale.set_draw_value(true);
        brightness_scale.set_value_pos(gtk4::PositionType::Right);

        let cb_brt = callbacks.on_brightness_changed.clone();
        let updating_brt = is_updating.clone();
        brightness_scale.connect_value_changed(move |scale| {
            if !*updating_brt.borrow() {
                let val = scale.value().round() as u8;
                cb_brt(Brightness::clamped(val));
            }
        });
        brightness_box.append(&brightness_scale);
        sliders_row.append(&brightness_box);

        // Speed
        let speed_box = GtkBox::new(Orientation::Vertical, 2);
        let spd_label = Label::new(Some("ANIMATION SPEED"));
        spd_label.add_css_class("msi-card-header");
        spd_label.set_halign(gtk4::Align::Start);
        speed_box.append(&spd_label);

        let spd_adj = Adjustment::new(50.0, 0.0, 100.0, 5.0, 10.0, 0.0);
        let speed_scale = Scale::new(Orientation::Horizontal, Some(&spd_adj));
        speed_scale.set_digits(0);
        speed_scale.set_draw_value(true);
        speed_scale.set_value_pos(gtk4::PositionType::Right);

        let cb_spd = callbacks.on_speed_changed.clone();
        let updating_spd = is_updating.clone();
        speed_scale.connect_value_changed(move |scale| {
            if !*updating_spd.borrow() {
                let val = scale.value().round() as u8;
                cb_spd(Speed::clamped(val));
            }
        });
        speed_box.append(&speed_scale);
        sliders_row.append(&speed_box);

        container.append(&sliders_row);

        // 3. Wave Direction Row
        let direction_box = GtkBox::new(Orientation::Horizontal, 8);
        direction_box.set_halign(gtk4::Align::Start);

        let dir_label = Label::new(Some("Direction:"));
        dir_label.add_css_class("zone-sublabel");
        direction_box.append(&dir_label);

        let dir_model = StringList::new(&["Leftwards (◄)", "Rightwards (►)"]);
        let direction_dropdown = DropDown::new(Some(dir_model), Option::<gtk4::Expression>::None);

        let cb_dir = callbacks.on_direction_changed.clone();
        let updating_dir = is_updating.clone();
        direction_dropdown.connect_selected_notify(move |dd| {
            if !*updating_dir.borrow() {
                let dir = match dd.selected() {
                    0 => Direction::Left,
                    _ => Direction::Right,
                };
                cb_dir(dir);
            }
        });
        direction_box.append(&direction_dropdown);
        container.append(&direction_box);

        Self {
            container,
            zone_tab_buttons,
            brightness_scale,
            speed_scale,
            speed_box,
            direction_dropdown,
            direction_box,
            is_updating,
        }
    }

    /// Returns the GTK container widget.
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Updates controls based on active effect mode and selection.
    pub fn update(
        &self,
        active_effect: EffectType,
        brightness: Brightness,
        speed: Speed,
        direction: Direction,
        selected_zone: Option<ZoneId>,
    ) {
        *self.is_updating.borrow_mut() = true;

        // Highlight selected tab
        for (tab_opt, btn) in &self.zone_tab_buttons {
            if *tab_opt == selected_zone {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }

        // Update slider values
        self.brightness_scale
            .set_value(brightness.as_percentage() as f64);
        self.speed_scale.set_value(speed.as_percentage() as f64);

        // Update direction dropdown
        self.direction_dropdown.set_selected(match direction {
            Direction::Left => 0,
            Direction::Right => 1,
        });

        // Hide or show contextual controls
        self.speed_box.set_visible(active_effect.supports_speed());
        self.direction_box
            .set_visible(active_effect.supports_direction());

        *self.is_updating.borrow_mut() = false;
    }
}
