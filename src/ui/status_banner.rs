//! Status indicator and device connection banner widget.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use std::rc::Rc;

use crate::backend::BackendStatus;
use crate::domain::device::DeviceInfo;

/// Callback for reconnect button.
pub type ReconnectCallback = Rc<dyn Fn()>;

/// Hardware status pill component.
pub struct StatusBanner {
    container: GtkBox,
    status_label: Label,
    reconnect_button: Button,
}

impl StatusBanner {
    /// Creates a new `StatusBanner` widget.
    pub fn new(on_reconnect: ReconnectCallback) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_valign(gtk4::Align::Center);

        let status_label = Label::new(Some("Connecting..."));
        status_label.add_css_class("status-pill");
        status_label.add_css_class("connecting");
        container.append(&status_label);

        let reconnect_button = Button::with_label("↻ Reconnect");
        reconnect_button.add_css_class("btn-secondary");
        reconnect_button.set_visible(false);

        reconnect_button.connect_clicked(move |_| {
            on_reconnect();
        });
        container.append(&reconnect_button);

        Self {
            container,
            status_label,
            reconnect_button,
        }
    }

    /// Returns the GTK container widget.
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Updates the status banner text and CSS class.
    pub fn update_status(&self, status: &BackendStatus, device_info: Option<&DeviceInfo>) {
        self.status_label.remove_css_class("connected");
        self.status_label.remove_css_class("connecting");
        self.status_label.remove_css_class("disconnected");

        match status {
            BackendStatus::Connected { device_name, .. } => {
                let name = device_info
                    .map(|d| d.name.as_str())
                    .unwrap_or(device_name.as_str());
                self.status_label.set_text(&format!("● Connected ({name})"));
                self.status_label.add_css_class("connected");
                self.reconnect_button.set_visible(false);
            }
            BackendStatus::Connecting { attempt, message } => {
                self.status_label
                    .set_text(&format!("◌ Connecting (#{attempt})..."));
                self.status_label.add_css_class("connecting");
                self.status_label.set_tooltip_text(Some(message));
                self.reconnect_button.set_visible(false);
            }
            BackendStatus::Disconnected { reason } => {
                self.status_label.set_text("○ Hardware Offline");
                self.status_label.add_css_class("disconnected");
                self.status_label.set_tooltip_text(Some(reason));
                self.reconnect_button.set_visible(true);
            }
            BackendStatus::Error { message } => {
                self.status_label.set_text("✕ Connection Error");
                self.status_label.add_css_class("disconnected");
                self.status_label.set_tooltip_text(Some(message));
                self.reconnect_button.set_visible(true);
            }
        }
    }
}
