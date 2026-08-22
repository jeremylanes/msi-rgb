//! OpenRGB SDK binary network protocol implementation.
//!
//! Handles raw TCP packet framing, version negotiation, and typed message
//! serialization / deserialization according to the OpenRGB SDK specification.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Result as IoResult};

use crate::domain::color::RgbColor;

/// OpenRGB protocol magic identifier bytes: "ORGB".
pub const OPENRGB_MAGIC: [u8; 4] = *b"ORGB";

/// Default local port for OpenRGB SDK server.
pub const OPENRGB_DEFAULT_PORT: u16 = 6742;

/// Default protocol version requested (version 5 is standard in OpenRGB 0.9+/1.0).
pub const OPENRGB_PROTOCOL_VERSION: u32 = 5;

/// OpenRGB Command Packet IDs.
pub mod packet_ids {
    pub const REQUEST_CONTROLLER_COUNT: u32 = 0;
    pub const REQUEST_CONTROLLER_DATA: u32 = 1;
    pub const REQUEST_PROTOCOL_VERSION: u32 = 40;
    pub const SET_CLIENT_NAME: u32 = 50;
    pub const DEVICE_LIST_UPDATED: u32 = 100;
    pub const UPDATE_LEDS: u32 = 1050;
    pub const UPDATE_ZONE_LEDS: u32 = 1051;
    pub const UPDATE_SINGLE_LED: u32 = 1052;
    pub const UPDATE_MODE: u32 = 1101;
}

/// OpenRGB Mode flag bits.
pub mod mode_flags {
    pub const HAS_SPEED: u32 = 1 << 0;
    pub const HAS_DIRECTION_LR: u32 = 1 << 1;
    pub const HAS_DIRECTION_UD: u32 = 1 << 2;
    pub const HAS_DIRECTION_HV: u32 = 1 << 3;
    pub const HAS_BRIGHTNESS: u32 = 1 << 4;
    pub const HAS_PER_LED_COLOR: u32 = 1 << 5;
    pub const HAS_MODE_SPECIFIC_COLOR: u32 = 1 << 6;
    pub const HAS_RANDOM_COLOR: u32 = 1 << 7;
}

/// Mode representation from OpenRGB controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRgbModeData {
    /// Mode index in the controller's modes array.
    pub id: u32,
    /// Name of the mode (e.g. "Static", "Static zone 1", "Breathing").
    pub name: String,
    /// Hardware/driver mode value.
    pub value: i32,
    /// Capability flag bits.
    pub flags: u32,
    /// Minimum animation speed.
    pub speed_min: u32,
    /// Maximum animation speed.
    pub speed_max: u32,
    /// Minimum brightness setting.
    pub brightness_min: u32,
    /// Maximum brightness setting.
    pub brightness_max: u32,
    /// Minimum number of colors required by mode.
    pub colors_min: u32,
    /// Maximum number of colors supported by mode.
    pub colors_max: u32,
    /// Active speed value.
    pub speed: u32,
    /// Active brightness value.
    pub brightness: u32,
    /// Active direction value.
    pub direction: u32,
    /// Color mode.
    pub color_mode: u32,
    /// Associated colors.
    pub colors: Vec<RgbColor>,
}

impl OpenRgbModeData {
    /// Checks if this mode has the speed parameter capability.
    #[must_use]
    pub const fn has_speed(&self) -> bool {
        (self.flags & mode_flags::HAS_SPEED) != 0
    }

    /// Checks if this mode has the brightness parameter capability.
    #[must_use]
    pub const fn has_brightness(&self) -> bool {
        (self.flags & mode_flags::HAS_BRIGHTNESS) != 0
    }

    /// Checks if this mode has left/right direction capability.
    #[must_use]
    pub const fn has_direction(&self) -> bool {
        (self.flags & mode_flags::HAS_DIRECTION_LR) != 0
    }
}

/// Controller description parsed from OpenRGB SDK.
#[derive(Debug, Clone)]
pub struct OpenRgbControllerData {
    /// Index on the OpenRGB server.
    pub index: u32,
    /// Device type id.
    pub device_type: i32,
    /// Hardware name.
    pub name: String,
    /// Vendor name.
    pub vendor: String,
    /// Description.
    pub description: String,
    /// Firmware / version string.
    pub version: String,
    /// Serial number string.
    pub serial: String,
    /// Hardware location (e.g. "HID: /dev/hidraw1").
    pub location: String,
    /// Active mode index.
    pub active_mode_index: i32,
    /// Available modes.
    pub modes: Vec<OpenRgbModeData>,
}

/// Reads a null-terminated string with a 16-bit prefix length from a cursor.
pub fn read_openrgb_string(cursor: &mut Cursor<&[u8]>) -> IoResult<String> {
    let len = cursor.read_u16::<LittleEndian>()? as usize;
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf)?;
    if let Some(&0) = buf.last() {
        buf.pop();
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

/// Writes a string with a 16-bit length prefix and null terminator to a buffer.
pub fn write_openrgb_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let _ = buf.write_u16::<LittleEndian>((bytes.len() + 1) as u16);
    buf.extend_from_slice(bytes);
    buf.push(0);
}

/// Builds a complete 16-byte header for an OpenRGB network packet.
pub fn build_packet_header(device_idx: u32, command_id: u32, data_len: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(16);
    header.extend_from_slice(&OPENRGB_MAGIC);
    header.extend_from_slice(&device_idx.to_le_bytes());
    header.extend_from_slice(&command_id.to_le_bytes());
    header.extend_from_slice(&data_len.to_le_bytes());
    header
}

/// Serializes an `UPDATE_MODE` payload into bytes.
pub fn serialize_update_mode_payload(proto_ver: u32, mode: &OpenRgbModeData) -> Vec<u8> {
    let mut body = Vec::new();
    write_openrgb_string(&mut body, &mode.name);
    let _ = body.write_i32::<LittleEndian>(mode.value);
    let _ = body.write_u32::<LittleEndian>(mode.flags);
    let _ = body.write_u32::<LittleEndian>(mode.speed_min);
    let _ = body.write_u32::<LittleEndian>(mode.speed_max);
    if proto_ver >= 3 {
        let _ = body.write_u32::<LittleEndian>(mode.brightness_min);
        let _ = body.write_u32::<LittleEndian>(mode.brightness_max);
    }
    let _ = body.write_u32::<LittleEndian>(mode.colors_min);
    let _ = body.write_u32::<LittleEndian>(mode.colors_max);
    let _ = body.write_u32::<LittleEndian>(mode.speed);
    if proto_ver >= 3 {
        let _ = body.write_u32::<LittleEndian>(mode.brightness);
    }
    let _ = body.write_u32::<LittleEndian>(mode.direction);
    let _ = body.write_u32::<LittleEndian>(mode.color_mode);
    let _ = body.write_u16::<LittleEndian>(mode.colors.len() as u16);
    for col in &mode.colors {
        body.push(col.r);
        body.push(col.g);
        body.push(col.b);
        body.push(0);
    }

    let mut payload = Vec::new();
    let full_packet_size = (body.len() + 4 + 4) as u32;
    let _ = payload.write_u32::<LittleEndian>(full_packet_size);
    let _ = payload.write_i32::<LittleEndian>(mode.id as i32);
    payload.extend_from_slice(&body);
    payload
}

/// Parses an OpenRGB controller data buffer into `OpenRgbControllerData`.
pub fn parse_controller_data(
    controller_idx: u32,
    proto_ver: u32,
    data: &[u8],
) -> IoResult<OpenRgbControllerData> {
    let mut cur = Cursor::new(data);
    let _data_size = cur.read_u32::<LittleEndian>()?;
    let device_type = cur.read_i32::<LittleEndian>()?;
    let name = read_openrgb_string(&mut cur)?;
    let vendor = read_openrgb_string(&mut cur)?;
    let description = read_openrgb_string(&mut cur)?;
    let version = read_openrgb_string(&mut cur)?;
    let serial = read_openrgb_string(&mut cur)?;
    let location = read_openrgb_string(&mut cur)?;
    let num_modes = cur.read_u16::<LittleEndian>()?;
    let active_mode_index = cur.read_i32::<LittleEndian>()?;

    let mut modes = Vec::with_capacity(num_modes as usize);
    for i in 0..num_modes {
        let m_name = read_openrgb_string(&mut cur)?;
        let m_val = cur.read_i32::<LittleEndian>()?;
        let m_flags = cur.read_u32::<LittleEndian>()?;
        let m_spd_min = cur.read_u32::<LittleEndian>()?;
        let m_spd_max = cur.read_u32::<LittleEndian>()?;
        let (m_brt_min, m_brt_max) = if proto_ver >= 3 {
            (
                cur.read_u32::<LittleEndian>()?,
                cur.read_u32::<LittleEndian>()?,
            )
        } else {
            (0, 100)
        };
        let m_col_min = cur.read_u32::<LittleEndian>()?;
        let m_col_max = cur.read_u32::<LittleEndian>()?;
        let m_spd = cur.read_u32::<LittleEndian>()?;
        let m_brt = if proto_ver >= 3 {
            cur.read_u32::<LittleEndian>()?
        } else {
            100
        };
        let m_dir = cur.read_u32::<LittleEndian>()?;
        let m_cmode = cur.read_u32::<LittleEndian>()?;
        let m_num_cols = cur.read_u16::<LittleEndian>()?;
        let mut cols = Vec::with_capacity(m_num_cols as usize);
        for _ in 0..m_num_cols {
            let r = cur.read_u8()?;
            let g = cur.read_u8()?;
            let b = cur.read_u8()?;
            let _pad = cur.read_u8()?;
            cols.push(RgbColor::new(r, g, b));
        }

        modes.push(OpenRgbModeData {
            id: i as u32,
            name: m_name,
            value: m_val,
            flags: m_flags,
            speed_min: m_spd_min,
            speed_max: m_spd_max,
            brightness_min: m_brt_min,
            brightness_max: m_brt_max,
            colors_min: m_col_min,
            colors_max: m_col_max,
            speed: m_spd,
            brightness: m_brt,
            direction: m_dir,
            color_mode: m_cmode,
            colors: cols,
        });
    }

    Ok(OpenRgbControllerData {
        index: controller_idx,
        device_type,
        name,
        vendor,
        description,
        version,
        serial,
        location,
        active_mode_index,
        modes,
    })
}
