//! Tests for Direct Native Linux HID backend.

use msi_rgb::backend::RgbBackend;
use msi_rgb::backend::native_hid::{MSI_PRODUCT_ID, MSI_VENDOR_ID, NativeHidBackend};

#[test]
fn test_native_hid_constants() {
    assert_eq!(MSI_VENDOR_ID, 0x1462);
    assert_eq!(MSI_PRODUCT_ID, 0x1601);
}

#[tokio::test]
async fn test_native_hid_backend_initialization() {
    let mut backend = NativeHidBackend::new();
    assert!(!backend.status().is_connected());

    // If running on actual hardware with /dev/hidraw available, connect succeeds
    let _ = backend.connect().await;
    let _ = backend.disconnect().await;
    assert!(!backend.status().is_connected());
}
