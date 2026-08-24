//! Tracing logging infrastructure setup.

use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initializes the `tracing` logging subsystem.
///
/// Example:
/// ```
/// use msi_rgb::infrastructure::logging::init_logging;
///
/// init_logging(false);
/// ```
pub fn init_logging(debug: bool) {
    let filter = if debug {
        EnvFilter::new("msi_rgb=debug,info")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("msi_rgb=info,warn"))
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_ansi(true);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();
}
