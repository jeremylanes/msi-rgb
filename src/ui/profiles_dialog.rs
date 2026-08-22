//! Presets and user profile management dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, Window,
};
use std::rc::Rc;

use crate::app::AppStateHandle;
use crate::domain::profile::UserProfile;

/// Profile action callback.
pub type ProfileActionCallback = Rc<dyn Fn(&str)>;

/// Displays the profiles management dialog.
pub fn show_profiles_dialog(
    parent: &(impl IsA<Window> + IsA<gtk4::Widget>),
    state: AppStateHandle,
    on_profile_applied: ProfileActionCallback,
) {
    let dialog = Window::new();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_title(Some("Lighting Profiles & Presets"));
    dialog.set_default_size(480, 520);
    dialog.add_css_class("main-window");

    let main_box = GtkBox::new(Orientation::Vertical, 14);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);

    let title_lbl = Label::new(Some("PRESET PROFILES"));
    title_lbl.add_css_class("msi-card-header");
    title_lbl.set_halign(gtk4::Align::Start);
    main_box.append(&title_lbl);

    // Profile list in scrolled window
    let scrolled = ScrolledWindow::new();
    scrolled.set_min_content_height(260);
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

        let info_box = GtkBox::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);

        let is_startup = profile.name == startup_name;
        let name_text = if is_startup {
            format!("★ {} (Startup)", profile.name)
        } else {
            profile.name.clone()
        };

        let name_lbl = Label::new(Some(&name_text));
        name_lbl.set_halign(gtk4::Align::Start);
        name_lbl.add_css_class("zone-label");
        info_box.append(&name_lbl);

        let desc_lbl = Label::new(Some(&profile.description));
        desc_lbl.set_halign(gtk4::Align::Start);
        desc_lbl.add_css_class("zone-sublabel");
        info_box.append(&desc_lbl);

        row_box.append(&info_box);

        // Apply button
        let apply_btn = Button::with_label("Apply");
        apply_btn.add_css_class("btn-primary");
        let cb_apply = on_profile_applied.clone();
        let prof_name = profile.name.clone();
        let dlg_weak = dialog.downgrade();
        apply_btn.connect_clicked(move |_| {
            cb_apply(&prof_name);
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
    save_box.set_margin_top(6);

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("New Profile Name..."));
    name_entry.set_hexpand(true);
    save_box.append(&name_entry);

    let save_btn = Button::with_label("Save Current as Profile");
    save_btn.add_css_class("btn-secondary");

    let state_save = state.clone();
    let name_entry_clone = name_entry.clone();
    let dlg_weak = dialog.downgrade();
    save_btn.connect_clicked(move |_| {
        let new_name = name_entry_clone.text().trim().to_string();
        if !new_name.is_empty() {
            let rt = tokio::runtime::Handle::current();
            let st = state_save.clone();
            let n = new_name.clone();
            rt.block_on(async move {
                let mut data = st.write().await;
                let cur_state = data.current_state.clone();
                let _ = data
                    .profile_manager
                    .create_profile(&n, "Custom user profile", cur_state);
                let _ = data.profile_manager.store().clone();
            });
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
