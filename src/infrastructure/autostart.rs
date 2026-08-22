//! Desktop autostart and user systemd integration.
//!
//! Manages automatic startup of the RGB lighting controller on session login
//! without requiring root privileges or sudo.

use std::fs;
use std::path::PathBuf;

use crate::domain::error::AppError;

/// Returns the path to the user's autostart desktop entry.
#[must_use]
pub fn autostart_desktop_path() -> PathBuf {
    let home = std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into());
    PathBuf::from(home)
        .join(".config")
        .join("autostart")
        .join("msi-rgb.desktop")
}

/// Returns the path to the user's systemd unit file.
#[must_use]
pub fn systemd_user_service_path() -> PathBuf {
    let home = std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into());
    PathBuf::from(home)
        .join(".config")
        .join("systemd")
        .join("user")
        .join("msi-rgb.service")
}

/// Checks whether autostart is currently enabled.
#[must_use]
pub fn is_autostart_enabled() -> bool {
    autostart_desktop_path().exists() || systemd_user_service_path().exists()
}

/// Enables session autostart by writing both `~/.config/autostart/msi-rgb.desktop`
/// and `~/.config/systemd/user/msi-rgb.service`.
pub fn enable_autostart() -> Result<(), AppError> {
    let desktop_path = autostart_desktop_path();
    if let Some(parent) = desktop_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Persistence(format!(
                "Failed to create autostart directory {parent:?}: {e}"
            ))
        })?;
    }

    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "msi-rgb".to_string());

    let desktop_content = format!(
        r#"[Desktop Entry]
Type=Application
Name=MSI Mystic Light RGB
Comment=MSI Katana RGB Keyboard Lighting Controller
Exec={exe_path} --daemon --apply-default
Icon=input-keyboard
Terminal=false
Categories=Utility;Settings;
X-GNOME-Autostart-enabled=true
StartupNotify=false
"#
    );

    fs::write(&desktop_path, desktop_content).map_err(|e| {
        AppError::Persistence(format!("Failed to write autostart desktop file: {e}"))
    })?;

    // Also write systemd user unit if directory can be created
    let systemd_path = systemd_user_service_path();
    if let Some(parent) = systemd_path.parent()
        && fs::create_dir_all(parent).is_ok()
    {
        let systemd_content = format!(
            r#"[Unit]
Description=MSI Mystic Light RGB Keyboard Lighting Controller
After=graphical-session.target openrgb.service
Wants=openrgb.service

[Service]
Type=simple
ExecStart={exe_path} --daemon --apply-default
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
"#
        );
        let _ = fs::write(&systemd_path, systemd_content);
    }

    tracing::info!("Enabled autostart at {:?}", desktop_path);
    Ok(())
}

/// Disables session autostart by removing desktop entry and systemd unit.
pub fn disable_autostart() -> Result<(), AppError> {
    let desktop_path = autostart_desktop_path();
    if desktop_path.exists() {
        fs::remove_file(&desktop_path)
            .map_err(|e| AppError::Persistence(format!("Failed to remove autostart file: {e}")))?;
        tracing::info!("Disabled autostart at {:?}", desktop_path);
    }

    let systemd_path = systemd_user_service_path();
    if systemd_path.exists() {
        let _ = fs::remove_file(&systemd_path);
    }

    Ok(())
}
