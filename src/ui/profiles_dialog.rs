//! Presets and user profile management dialog.

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CssProvider, Entry, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, Window,
};
use std::rc::Rc;

use crate::app::AppStateHandle;
use crate::domain::profile::UserProfile;
use crate::persistence::Storage;

/// Profile action callback.
pub type ProfileActionCallback = Rc<dyn Fn(&str)>;

/// Displays the profiles management dialog.
pub fn show_profiles_dialog(
    parent: &(impl IsA<Window> + IsA<gtk4::Widget>),
    state: AppStateHandle,
    storage: Storage,
    on_profile_applied: ProfileActionCallback,
) {
    let dialog = Window::new();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_title(Some("Lighting Profiles Studio"));
    dialog.set_default_size(520, 560);
    dialog.add_css_class("main-window");

    let main_box = GtkBox::new(Orientation::Vertical, 14);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(18);
    main_box.set_margin_end(18);

    let title_lbl = Label::new(Some("CONFIGURED PROFILES & PRESETS"));
    title_lbl.add_css_class("msi-card-header");
    title_lbl.set_halign(Align::Start);
    main_box.append(&title_lbl);

    // Profile list in scrolled window
    let scrolled = ScrolledWindow::new();
    scrolled.set_min_content_height(280);
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);

    let list_box = ListBox::new();
    list_box.add_css_class("msi-card");

    // Fetch profiles from state synchronously
    let profiles: Vec<UserProfile> = {
        let rt = tokio::runtime::Handle::current();
        let state_clone = state.clone();
        rt.block_on(async move {
            let data = state_clone.read().await;
            data.profile_manager.list_profiles().to_vec()
        })
    };

    let startup_name: String = {
        let rt = tokio::runtime::Handle::current();
        let state_clone = state.clone();
        rt.block_on(async move {
            let data = state_clone.read().await;
            data.profile_manager.startup_profile_name().to_string()
        })
    };

    for profile in profiles {
        let row = ListBoxRow::new();
        let row_box = GtkBox::new(Orientation::Horizontal, 10);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(10);
        row_box.set_margin_end(10);
        row_box.set_valign(Align::Center);

        // 4 Mini Color Preview Dots
        let dots_box = GtkBox::new(Orientation::Horizontal, 3);
        dots_box.set_valign(Align::Center);

        for zone in profile.state.all_zones() {
            let col = zone.effective_color();
            let dot = GtkBox::new(Orientation::Horizontal, 0);
            dot.add_css_class("color-swatch-slot");

            let prov = CssProvider::new();
            let css = format!(
                "box.color-swatch-slot {{ background-color: rgb({},{},{}); min-width: 14px; min-height: 14px; border-radius: 4px; }}",
                col.r, col.g, col.b
            );
            prov.load_from_data(&css);
            dot.style_context()
                .add_provider(&prov, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            dots_box.append(&dot);
        }
        row_box.append(&dots_box);

        let info_box = GtkBox::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);

        let is_startup = profile.name == startup_name;
        let name_text = if is_startup {
            format!("★ {} (Default Startup)", profile.name)
        } else {
            profile.name.clone()
        };

        let name_lbl = Label::new(Some(&name_text));
        name_lbl.set_halign(Align::Start);
        name_lbl.add_css_class("zone-label");
        info_box.append(&name_lbl);

        let desc_lbl = Label::new(Some(&profile.description));
        desc_lbl.set_halign(Align::Start);
        desc_lbl.add_css_class("zone-sublabel");
        info_box.append(&desc_lbl);

        row_box.append(&info_box);

        // Set as default button (if not already default)
        if !is_startup {
            let def_btn = Button::with_label("★ Set Default");
            def_btn.add_css_class("btn-pill-small");
            let state_def = state.clone();
            let storage_def = storage.clone();
            let prof_name = profile.name.clone();
            let dlg_weak = dialog.downgrade();
            let cb_app = on_profile_applied.clone();
            def_btn.connect_clicked(move |_| {
                let rt = tokio::runtime::Handle::current();
                let st = state_def.clone();
                let stor = storage_def.clone();
                let name = prof_name.clone();
                rt.block_on(async move {
                    let mut data = st.write().await;
                    let _ = data.profile_manager.set_startup_profile(&name);
                    let _ = stor.save_profiles(data.profile_manager.store());
                });
                cb_app(&prof_name);
                if let Some(dlg) = dlg_weak.upgrade() {
                    dlg.close();
                }
            });
            row_box.append(&def_btn);
        }

        // Apply button
        let apply_btn = Button::with_label("Apply");
        apply_btn.add_css_class("btn-primary");
        let cb_apply = on_profile_applied.clone();
        let prof_name_app = profile.name.clone();
        let dlg_weak = dialog.downgrade();
        apply_btn.connect_clicked(move |_| {
            cb_apply(&prof_name_app);
            if let Some(dlg) = dlg_weak.upgrade() {
                dlg.close();
            }
        });
        row_box.append(&apply_btn);

        row.set_child(Some(&row_box));
        list_box.append(&row);
    }

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    // Create new profile section
    let save_box = GtkBox::new(Orientation::Horizontal, 8);
    save_box.set_margin_top(8);

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("New Profile Name..."));
    name_entry.set_hexpand(true);
    name_entry.add_css_class("hex-entry");
    save_box.append(&name_entry);

    let save_btn = Button::with_label("✚ Save Current Look");
    save_btn.add_css_class("btn-secondary");

    let state_save = state;
    let storage_save = storage;
    let name_entry_clone = name_entry;
    let dlg_weak = dialog.downgrade();
    let cb_save = on_profile_applied;
    save_btn.connect_clicked(move |_| {
        let new_name = name_entry_clone.text().trim().to_string();
        if !new_name.is_empty() {
            let rt = tokio::runtime::Handle::current();
            let st = state_save.clone();
            let stor = storage_save.clone();
            let n = new_name.clone();
            rt.block_on(async move {
                let mut data = st.write().await;
                let cur_state = data.current_state.clone();
                let _ = data.profile_manager.create_profile(
                    &n,
                    "Custom user lighting setup",
                    cur_state,
                );
                let _ = stor.save_profiles(data.profile_manager.store());
            });
            cb_save(&new_name);
            if let Some(dlg) = dlg_weak.upgrade() {
                dlg.close();
            }
        }
    });
    save_box.append(&save_btn);
    main_box.append(&save_box);

    dialog.set_child(Some(&main_box));
    dialog.present();
}
