//! Primary Application Window assembling all UI components.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, DropDown, HeaderBar, Label, Orientation,
    StringList, Switch,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use super::color_picker::ColorPickerWidget;
use super::effect_selector::EffectSelector;
use super::keyboard_preview::KeyboardPreview;
use super::profiles_dialog::show_profiles_dialog;
use super::status_banner::StatusBanner;
use super::zone_control::{ZoneControlCallbacks, ZoneControlPanel};
use crate::app::{AppStateHandle, RgbService, ServiceCommand};
use crate::domain::effect::EffectType;
use crate::domain::zone::ZoneId;
use crate::infrastructure::{disable_autostart, enable_autostart, is_autostart_enabled};
use crate::persistence::Storage;

/// Main application window holding all subcomponents.
pub struct MainWindow {
    window: ApplicationWindow,
    keyboard_preview: KeyboardPreview,
    effect_selector: EffectSelector,
    color_picker: ColorPickerWidget,
    zone_control: ZoneControlPanel,
    status_banner: StatusBanner,
    profile_dropdown: DropDown,
    state: AppStateHandle,
    service: Arc<RgbService>,
    #[allow(dead_code)]
    storage: Storage,
    selected_zone: Rc<RefCell<Option<ZoneId>>>,
}

impl MainWindow {
    /// Builds and displays the main application window.
    pub fn build(
        app: &Application,
        state: AppStateHandle,
        service: Arc<RgbService>,
        storage: Storage,
    ) -> Rc<RefCell<Self>> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("MSI Mystic Light Studio Pro")
            .default_width(1040)
            .default_height(660)
            .build();

        window.add_css_class("main-window");

        let selected_zone = Rc::new(RefCell::new(None));

        // 1. HeaderBar
        let header_bar = HeaderBar::new();
        header_bar.add_css_class("headerbar");

        let title_box = GtkBox::new(Orientation::Vertical, 0);
        let title_label = Label::new(Some("MSI MYSTIC LIGHT"));
        title_label.add_css_class("title");
        let subtitle_label = Label::new(Some("Katana 15 HX (MS-1565)"));
        subtitle_label.add_css_class("subtitle");
        title_box.append(&title_label);
        title_box.append(&subtitle_label);
        header_bar.set_title_widget(Some(&title_box));

        // Reconnect callback
        let service_recon = service.clone();
        let status_banner = StatusBanner::new(Rc::new(move || {
            let s = service_recon.clone();
            tokio::spawn(async move {
                let _ = s.send_command(ServiceCommand::Reconnect).await;
            });
        }));
        header_bar.pack_start(status_banner.widget());

        // Profile quick dropdown in HeaderBar
        let profile_names = {
            let rt = tokio::runtime::Handle::current();
            let st = state.clone();
            rt.block_on(async move {
                let data = st.read().await;
                data.profile_manager
                    .list_profiles()
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
            })
        };

        let profile_str_refs: Vec<&str> = profile_names.iter().map(|s| s.as_str()).collect();
        let profile_model = StringList::new(&profile_str_refs);
        let profile_dropdown = DropDown::new(Some(profile_model), Option::<gtk4::Expression>::None);
        header_bar.pack_end(&profile_dropdown);

        // Profile Studio Dialog button
        let profiles_btn = Button::with_label("★ Presets");
        profiles_btn.add_css_class("btn-secondary");
        header_bar.pack_end(&profiles_btn);

        window.set_titlebar(Some(&header_bar));

        // 2. Main Studio Dashboard Box
        let main_box = GtkBox::new(Orientation::Vertical, 10);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(14);
        main_box.set_margin_end(14);
        main_box.set_vexpand(true);
        main_box.set_hexpand(true);

        // ==========================================
        // TOP SECTION: Full-Width Keyboard Deck Hero
        // ==========================================
        let preview_card = GtkBox::new(Orientation::Vertical, 6);
        preview_card.add_css_class("msi-card");
        preview_card.set_hexpand(true);
        preview_card.set_vexpand(false);

        let preview_header = GtkBox::new(Orientation::Horizontal, 6);
        let preview_title = Label::new(Some("4-ZONE KEYBOARD ILLUMINATION"));
        preview_title.add_css_class("msi-card-header");
        preview_header.append(&preview_title);
        preview_card.append(&preview_header);

        let sel_zone_preview = selected_zone.clone();
        let keyboard_preview = KeyboardPreview::new(Rc::new(move |zone_id| {
            *sel_zone_preview.borrow_mut() = Some(zone_id);
        }));
        preview_card.append(keyboard_preview.widget());
        main_box.append(&preview_card);

        // ==========================================
        // BOTTOM SECTION: 50% / 50% Studio Master Grid
        // ==========================================
        let panels_row = GtkBox::new(Orientation::Horizontal, 10);
        panels_row.set_homogeneous(true);
        panels_row.set_hexpand(true);
        panels_row.set_vexpand(true);

        // --- Left Panel: Lighting Effects & Zone Dynamics ---
        let lighting_card = GtkBox::new(Orientation::Vertical, 6);
        lighting_card.add_css_class("msi-card");
        lighting_card.set_hexpand(true);
        lighting_card.set_vexpand(true);

        let eff_header = GtkBox::new(Orientation::Horizontal, 6);
        let eff_title = Label::new(Some("LIGHTING & ZONE ENGINE"));
        eff_title.add_css_class("msi-card-header");
        eff_header.append(&eff_title);
        lighting_card.append(&eff_header);

        let state_eff = state.clone();
        let service_eff = service.clone();
        let storage_eff = storage.clone();
        let sel_zone_eff = selected_zone.clone();
        let effect_selector = EffectSelector::new(Rc::new(move |new_effect| {
            let selected = *sel_zone_eff.borrow();
            let rt = tokio::runtime::Handle::current();
            let st = state_eff.clone();
            let stor = storage_eff.clone();
            let updated_state = rt.block_on(async move {
                let mut data = st.write().await;
                match selected {
                    Some(zid) => {
                        let zone = data.current_state.zone_mut(zid);
                        zone.effect = new_effect;
                        zone.enabled = new_effect != EffectType::Off;
                        data.current_state.sync_all_zones = false;
                    }
                    None => {
                        data.current_state.global_effect = new_effect;
                        data.current_state.sync_zones_from_global();
                    }
                }
                data.auto_save_custom_profile();
                let _ = stor.save_profiles(data.profile_manager.store());
                data.current_state.clone()
            });
            let s = service_eff.clone();
            tokio::spawn(async move {
                let _ = s.apply_state(updated_state).await;
            });
        }));
        lighting_card.append(effect_selector.widget());

        let sel_zone_tab = selected_zone.clone();
        let state_brt = state.clone();
        let service_brt = service.clone();
        let storage_brt = storage.clone();
        let sel_zone_brt = selected_zone.clone();
        let state_spd = state.clone();
        let service_spd = service.clone();
        let storage_spd = storage.clone();
        let sel_zone_spd = selected_zone.clone();
        let state_dir = state.clone();
        let service_dir = service.clone();
        let storage_dir = storage.clone();
        let sel_zone_dir = selected_zone.clone();

        let zone_control = ZoneControlPanel::new(ZoneControlCallbacks {
            on_tab_selected: Rc::new(move |tab_zone| {
                *sel_zone_tab.borrow_mut() = tab_zone;
            }),
            on_brightness_changed: Rc::new(move |new_brt| {
                let selected = *sel_zone_brt.borrow();
                let rt = tokio::runtime::Handle::current();
                let st = state_brt.clone();
                let stor = storage_brt.clone();
                let updated_state = rt.block_on(async move {
                    let mut data = st.write().await;
                    match selected {
                        Some(zid) => {
                            data.current_state.zone_mut(zid).brightness = new_brt;
                            data.current_state.sync_all_zones = false;
                        }
                        None => {
                            data.current_state.global_brightness = new_brt;
                            data.current_state.sync_zones_from_global();
                        }
                    }
                    data.auto_save_custom_profile();
                    let _ = stor.save_profiles(data.profile_manager.store());
                    data.current_state.clone()
                });
                let s = service_brt.clone();
                tokio::spawn(async move {
                    let _ = s.apply_state(updated_state).await;
                });
            }),
            on_speed_changed: Rc::new(move |new_spd| {
                let selected = *sel_zone_spd.borrow();
                let rt = tokio::runtime::Handle::current();
                let st = state_spd.clone();
                let stor = storage_spd.clone();
                let updated_state = rt.block_on(async move {
                    let mut data = st.write().await;
                    match selected {
                        Some(zid) => {
                            data.current_state.zone_mut(zid).speed = new_spd;
                            data.current_state.sync_all_zones = false;
                        }
                        None => {
                            data.current_state.global_speed = new_spd;
                            data.current_state.sync_zones_from_global();
                        }
                    }
                    data.auto_save_custom_profile();
                    let _ = stor.save_profiles(data.profile_manager.store());
                    data.current_state.clone()
                });
                let s = service_spd.clone();
                tokio::spawn(async move {
                    let _ = s.apply_state(updated_state).await;
                });
            }),
            on_direction_changed: Rc::new(move |new_dir| {
                let selected = *sel_zone_dir.borrow();
                let rt = tokio::runtime::Handle::current();
                let st = state_dir.clone();
                let stor = storage_dir.clone();
                let updated_state = rt.block_on(async move {
                    let mut data = st.write().await;
                    match selected {
                        Some(zid) => {
                            data.current_state.zone_mut(zid).direction = new_dir;
                            data.current_state.sync_all_zones = false;
                        }
                        None => {
                            data.current_state.global_direction = new_dir;
                            data.current_state.sync_zones_from_global();
                        }
                    }
                    data.auto_save_custom_profile();
                    let _ = stor.save_profiles(data.profile_manager.store());
                    data.current_state.clone()
                });
                let s = service_dir.clone();
                tokio::spawn(async move {
                    let _ = s.apply_state(updated_state).await;
                });
            }),
        });
        lighting_card.append(zone_control.widget());

        // Fill remaining space in left card
        let spacer_l = GtkBox::new(Orientation::Vertical, 0);
        spacer_l.set_vexpand(true);
        lighting_card.append(&spacer_l);

        // Footer Actions
        let footer_box = GtkBox::new(Orientation::Horizontal, 10);
        footer_box.set_margin_top(4);

        let autostart_switch = Switch::new();
        autostart_switch.set_active(is_autostart_enabled());
        let storage_auto = storage.clone();
        let state_auto = state.clone();
        autostart_switch.connect_state_set(move |_, active| {
            if active {
                let _ = enable_autostart();
            } else {
                let _ = disable_autostart();
            }
            let rt = tokio::runtime::Handle::current();
            let st = state_auto.clone();
            let stor = storage_auto.clone();
            rt.block_on(async move {
                let mut data = st.write().await;
                data.settings.autostart = active;
                let _ = data.settings.save(&stor);
            });
            glib::Propagation::Proceed
        });

        let autostart_lbl = Label::new(Some("Auto-restore on Login"));
        autostart_lbl.add_css_class("zone-sublabel");

        footer_box.append(&autostart_switch);
        footer_box.append(&autostart_lbl);

        let footer_spacer = GtkBox::new(Orientation::Horizontal, 0);
        footer_spacer.set_hexpand(true);
        footer_box.append(&footer_spacer);

        let apply_button = Button::with_label("✔ Save & Apply");
        apply_button.add_css_class("btn-primary");
        footer_box.append(&apply_button);

        lighting_card.append(&footer_box);
        panels_row.append(&lighting_card);

        // --- Right Panel: Chromatic Color Studio ---
        let color_card = GtkBox::new(Orientation::Vertical, 6);
        color_card.add_css_class("msi-card");
        color_card.set_hexpand(true);
        color_card.set_vexpand(true);

        let color_title = Label::new(Some("CHROMATIC COLOR STUDIO"));
        color_title.add_css_class("msi-card-header");
        color_card.append(&color_title);

        let state_col = state.clone();
        let service_col = service.clone();
        let storage_col = storage.clone();
        let sel_zone_col = selected_zone.clone();
        let color_picker = ColorPickerWidget::new(Rc::new(move |new_color| {
            let selected = *sel_zone_col.borrow();
            let rt = tokio::runtime::Handle::current();
            let st = state_col.clone();
            let stor = storage_col.clone();
            let updated_state = rt.block_on(async move {
                let mut data = st.write().await;
                match selected {
                    Some(zid) => {
                        let zone = data.current_state.zone_mut(zid);
                        zone.color = new_color;
                        data.current_state.sync_all_zones = false;
                    }
                    None => {
                        data.current_state.global_color = new_color;
                        data.current_state.sync_zones_from_global();
                    }
                }
                data.auto_save_custom_profile();
                let _ = stor.save_profiles(data.profile_manager.store());
                data.current_state.clone()
            });
            let s = service_col.clone();
            tokio::spawn(async move {
                let _ = s.apply_state(updated_state).await;
            });
        }));
        color_card.append(color_picker.widget());
        panels_row.append(&color_card);

        main_box.append(&panels_row);
        window.set_child(Some(&main_box));

        let main_window = Rc::new(RefCell::new(Self {
            window,
            keyboard_preview,
            effect_selector,
            color_picker,
            zone_control,
            status_banner,
            profile_dropdown,
            state,
            service,
            storage,
            selected_zone,
        }));

        MainWindow::setup_event_handlers(&main_window, &apply_button, &profiles_btn);
        MainWindow::start_state_polling(&main_window);
        main_window.borrow().refresh_ui();

        main_window.borrow().window.present();
        main_window
    }

    /// Connects interactive signals and callbacks across all widgets.
    fn setup_event_handlers(this: &Rc<RefCell<Self>>, apply_btn: &Button, profiles_btn: &Button) {
        let (state_prof, service_prof, storage_prof, win_weak) = {
            let m = this.borrow();
            (
                m.state.clone(),
                m.service.clone(),
                m.storage.clone(),
                m.window.downgrade(),
            )
        };
        let this_for_dialog = this.clone();
        profiles_btn.connect_clicked(move |_| {
            if let Some(w) = win_weak.upgrade() {
                let s_prof = service_prof.clone();
                let this_diag = this_for_dialog.clone();
                show_profiles_dialog(
                    &w,
                    state_prof.clone(),
                    storage_prof.clone(),
                    Rc::new(move |name| {
                        let s = s_prof.clone();
                        let n = name.to_string();
                        let this_up = this_diag.clone();
                        tokio::spawn(async move {
                            let _ = s.apply_profile(&n).await;
                        });
                        this_up.borrow().refresh_ui();
                    }),
                );
            }
        });

        // 2. Profile quick dropdown
        let this_drop = this.clone();
        let (profile_dropdown, service_drop, state_drop) = {
            let m = this.borrow();
            (
                m.profile_dropdown.clone(),
                m.service.clone(),
                m.state.clone(),
            )
        };

        profile_dropdown.connect_selected_notify(move |dd| {
            let sel_idx = dd.selected() as usize;
            let rt = tokio::runtime::Handle::current();
            let st = state_drop.clone();
            let maybe_name = rt.block_on(async move {
                let data = st.read().await;
                data.profile_manager
                    .list_profiles()
                    .get(sel_idx)
                    .map(|p| p.name.clone())
            });

            if let Some(name) = maybe_name {
                let s = service_drop.clone();
                let this_refresh = this_drop.clone();
                tokio::spawn(async move {
                    let _ = s.apply_profile(&name).await;
                });
                this_refresh.borrow().refresh_ui();
            }
        });

        // 3. Apply Button
        let (state_apply, service_apply, storage_apply) = {
            let m = this.borrow();
            (m.state.clone(), m.service.clone(), m.storage.clone())
        };
        apply_btn.connect_clicked(move |_| {
            let rt = tokio::runtime::Handle::current();
            let st = state_apply.clone();
            let stor = storage_apply.clone();
            let current = rt.block_on(async move {
                let mut data = st.write().await;
                data.auto_save_custom_profile();
                let _ = stor.save_profiles(data.profile_manager.store());
                let _ = data.settings.save(&stor);
                data.current_state.clone()
            });

            let s = service_apply.clone();
            tokio::spawn(async move {
                let _ = s.apply_state(current).await;
            });
        });
    }

    /// Starts a background GLib timer to sync UI with state updates without polling OpenRGB directly.
    fn start_state_polling(this: &Rc<RefCell<Self>>) {
        let this_clone = this.clone();
        glib::timeout_add_local(Duration::from_millis(250), move || {
            this_clone.borrow().refresh_ui();
            glib::ControlFlow::Continue
        });
    }

    /// Synchronizes the GUI widgets with current application state.
    pub fn refresh_ui(&self) {
        let rt = tokio::runtime::Handle::current();
        let st = self.state.clone();
        let (current_state, backend_status, device_info) = rt.block_on(async move {
            let data = st.read().await;
            (
                data.current_state.clone(),
                data.backend_status.clone(),
                data.device_info.clone(),
            )
        });

        let selected = *self.selected_zone.borrow();

        // 1. Update status banner
        self.status_banner
            .update_status(&backend_status, device_info.as_ref());

        // 2. Update keyboard visualizer
        self.keyboard_preview.update_state(&current_state, selected);

        // 3. Update active effect & color
        let active_effect = match selected {
            Some(zid) => current_state.zone(zid).effect,
            None => current_state.global_effect,
        };
        let active_color = match selected {
            Some(zid) => current_state.zone(zid).effective_color(),
            None => current_state.global_color,
        };
        let active_brightness = match selected {
            Some(zid) => current_state.zone(zid).brightness,
            None => current_state.global_brightness,
        };
        let active_speed = match selected {
            Some(zid) => current_state.zone(zid).speed,
            None => current_state.global_speed,
        };
        let active_direction = match selected {
            Some(zid) => current_state.zone(zid).direction,
            None => current_state.global_direction,
        };

        self.effect_selector.set_active_effect(active_effect);
        self.color_picker.set_color(active_color);
        self.zone_control.update(
            active_effect,
            active_brightness,
            active_speed,
            active_direction,
            selected,
        );
    }
}
