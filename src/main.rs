//! Entry point for MSI Mystic Light RGB Linux controller.

use clap::Parser;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::sync::Arc;
use tokio::runtime::Runtime;

use msi_rgb::app::{RgbService, create_app_state};
use msi_rgb::domain::EffectType;
use msi_rgb::infrastructure::init_logging;
use msi_rgb::persistence::Storage;
use msi_rgb::profiles::ProfileManager;
use msi_rgb::settings::AppSettings;
use msi_rgb::ui::run_gui;

/// CLI Arguments for MSI Mystic Light RGB Controller.
#[derive(Parser, Debug)]
#[command(
    name = "msi-rgb",
    author = "MSI Linux Community",
    version = env!("CARGO_PKG_VERSION"),
    about = "Native Linux RGB keyboard controller for MSI Katana 15 HX (MS-1565 / 1462:1601)"
)]
struct CliArgs {
    /// Run with mock hardware backend (offline simulation without physical hardware).
    #[arg(long)]
    mock: bool,

    /// Run as background daemon without opening graphical interface.
    #[arg(long, short = 'd')]
    daemon: bool,

    /// Apply specific profile by name and exit immediately.
    #[arg(long, short = 'p')]
    profile: Option<String>,

    /// Apply startup profile and exit immediately.
    #[arg(long)]
    apply_default: bool,

    /// List all configured profiles and exit.
    #[arg(long)]
    list_profiles: bool,

    /// Enable verbose debug logging output.
    #[arg(long, short = 'v')]
    debug: bool,
}

fn main() {
    let args = CliArgs::parse();
    init_logging(args.debug);

    tracing::info!(
        "Starting MSI Mystic Light RGB Controller v{}",
        env!("CARGO_PKG_VERSION")
    );

    let storage = Storage::default_xdg();
    let settings = AppSettings::load_or_default(&storage);
    let store = match storage.load_profiles() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Could not load profiles: {e}, using factory defaults");
            msi_rgb::domain::profile::ProfileStore::default()
        }
    };

    let profile_manager = ProfileManager::from_store(store);
    let _ = storage.save_profiles(profile_manager.store());

    // Command line: List profiles
    if args.list_profiles {
        println!("=== Configured RGB Profiles ===");
        for p in profile_manager.list_profiles() {
            let badge = if p.is_builtin {
                "[Built-in]"
            } else {
                "[Custom]"
            };
            let startup = if p.name == profile_manager.startup_profile_name() {
                " ★ (Startup)"
            } else {
                ""
            };
            println!("  • {}{}: {} {}", p.name, startup, p.description, badge);
        }
        return;
    }

    let use_mock = args.mock || settings.use_mock_backend;
    let state = create_app_state(profile_manager.clone(), settings.clone());

    // Build Tokio multithread runtime for background async services
    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize Tokio runtime: {e}");
            std::process::exit(1);
        }
    };

    let _guard = rt.enter();
    let (service, _service_handle) = RgbService::spawn(state.clone(), storage.clone(), use_mock);
    let service_arc = Arc::new(service);

    // Command line: Apply specific profile
    if let Some(ref prof_name) = args.profile {
        println!("Applying profile '{prof_name}'...");
        let s = service_arc.clone();
        let name = prof_name.clone();
        let st = state.clone();
        rt.block_on(async move {
            let _ = s.apply_profile(&name).await;
            // Wait up to 10 seconds for backend connection & application
            for _ in 0..20 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let data = st.read().await;
                if data.backend_status.is_connected() {
                    println!("Successfully applied profile '{name}' to keyboard.");
                    break;
                }
            }
        });

        let is_dynamic = {
            let data = rt.block_on(async { state.read().await });
            data.current_state.global_effect == EffectType::ColorCycle
                && data.current_state.sync_all_zones
        };

        if !args.daemon {
            if is_dynamic {
                spawn_background_daemon();
            } else {
                stop_background_daemon();
            }
            return;
        }
    }

    // Command line: Apply startup default
    if args.apply_default {
        let startup_name = profile_manager.startup_profile_name().to_string();
        println!("Applying startup profile '{startup_name}'...");
        let s = service_arc.clone();
        let st = state.clone();
        let name = startup_name.clone();
        rt.block_on(async move {
            let _ = s.apply_profile(&name).await;
            // Wait up to 20 seconds for backend to connect and apply (handles system boot)
            for _ in 0..40 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let data = st.read().await;
                if data.backend_status.is_connected() {
                    println!("Successfully applied startup profile '{name}' to keyboard.");
                    break;
                }
            }
        });

        let is_dynamic = {
            let data = rt.block_on(async { state.read().await });
            data.current_state.global_effect == EffectType::ColorCycle
                && data.current_state.sync_all_zones
        };

        if !args.daemon {
            if is_dynamic {
                spawn_background_daemon();
            } else {
                stop_background_daemon();
            }
            return;
        }
    }

    // Daemon mode (background service loop)
    if args.daemon {
        let startup_name = profile_manager.startup_profile_name().to_string();
        let s = service_arc.clone();
        rt.block_on(async move {
            let _ = s.apply_profile(&startup_name).await;
            tracing::info!("Running in background daemon mode. Press Ctrl+C to exit.");
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Shutdown signal received.");
        });
        return;
    }

    // Default: Run native GTK4 GUI
    run_gui(state.clone(), service_arc, storage);

    // After GUI closes: if current profile is dynamic, keep it running via background daemon
    let is_dynamic = {
        let data = rt.block_on(async { state.read().await });
        data.current_state.global_effect == EffectType::ColorCycle
            && data.current_state.sync_all_zones
    };

    if is_dynamic {
        spawn_background_daemon();
    } else {
        stop_background_daemon();
    }
}

fn spawn_background_daemon() {
    stop_background_daemon();

    if let Ok(exe) = std::env::current_exe() {
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("--daemon")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        let _ = cmd.spawn();
    }
}

fn stop_background_daemon() {
    let _ = std::process::Command::new("pkill")
        .args(["-f", "msi-rgb.*--daemon"])
        .output();
}
