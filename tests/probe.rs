//! Integration test probe for OpenRGB hardware communication.

use msi_rgb::backend::RgbBackend;
use msi_rgb::backend::openrgb::OpenRgbBackend;
use msi_rgb::backend::openrgb::protocol::OPENRGB_DEFAULT_PORT;
use msi_rgb::domain::color::RgbColor;
use msi_rgb::domain::device::RgbState;
use msi_rgb::domain::effect::EffectType;

#[tokio::test]
async fn test_live_openrgb_integration_if_available() {
    let mut backend = OpenRgbBackend::new("127.0.0.1", OPENRGB_DEFAULT_PORT);
    if let Ok(()) = backend.connect().await {
        println!(
            "Live OpenRGB Server detected! Status: {:?}",
            backend.status()
        );
        let dev = backend.detect_device().await.unwrap();
        if let Some(d) = dev {
            println!("Detected MSI device: {} (VID:PID {})", d.name, d.vid_pid);
            let state = RgbState::new_uniform(EffectType::Static, RgbColor::MSI_RED);
            let _ = backend.apply_state(&state).await;
        }
        let _ = backend.disconnect().await;
    } else {
        println!("OpenRGB server offline, live integration test skipped gracefully.");
    }
}
