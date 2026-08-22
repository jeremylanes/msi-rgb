//! Tests for OpenRGB binary protocol serialization and framing.

use msi_rgb::backend::openrgb::protocol::{
    OPENRGB_MAGIC, OpenRgbModeData, build_packet_header, read_openrgb_string,
    serialize_update_mode_payload, write_openrgb_string,
};
use msi_rgb::domain::color::RgbColor;
use std::io::Cursor;

#[test]
fn test_openrgb_packet_header_format() {
    let header = build_packet_header(0, 1101, 64);
    assert_eq!(header.len(), 16);
    assert_eq!(&header[0..4], &OPENRGB_MAGIC);
    assert_eq!(u32::from_le_bytes(header[4..8].try_into().unwrap()), 0); // device index
    assert_eq!(u32::from_le_bytes(header[8..12].try_into().unwrap()), 1101); // command ID
    assert_eq!(u32::from_le_bytes(header[12..16].try_into().unwrap()), 64); // payload length
}

#[test]
fn test_openrgb_string_roundtrip() {
    let mut buf = Vec::new();
    write_openrgb_string(&mut buf, "MSI Katana 15 HX");

    let mut cursor = Cursor::new(buf.as_slice());
    let read_back = read_openrgb_string(&mut cursor).unwrap();
    assert_eq!(read_back, "MSI Katana 15 HX");
}

#[test]
fn test_serialize_update_mode_payload() {
    let mode = OpenRgbModeData {
        id: 5,
        name: "Static".to_string(),
        value: 1,
        flags: 1 << 4, // HasBrightness
        speed_min: 0,
        speed_max: 100,
        brightness_min: 0,
        brightness_max: 100,
        colors_min: 1,
        colors_max: 1,
        speed: 50,
        brightness: 100,
        direction: 0,
        color_mode: 2,
        colors: vec![RgbColor::new(255, 0, 50)],
    };

    let payload = serialize_update_mode_payload(5, &mode);
    assert!(payload.len() > 20);
    // First 4 bytes is size, next 4 bytes is mode.id (5)
    let mode_id = i32::from_le_bytes(payload[4..8].try_into().unwrap());
    assert_eq!(mode_id, 5);
}
