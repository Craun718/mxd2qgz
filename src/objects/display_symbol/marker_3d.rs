//! _3DSymbols.dll 的 SimpleMarker3DSymbol 持久化对象。

use super::{read_i32_le, read_signed_symbol_version};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::geometry::{GeometryBody10_1, read_geometry_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io;

pub const SIMPLE_MARKER_3D_SYMBOL_CLASS_ID: [u8; 16] = [
    0x70, 0x72, 0x3f, 0x77, 0xfb, 0xae, 0xd5, 0x11, 0x81, 0x12, 0x00, 0xc0, 0x4f, 0xa0, 0xad, 0xf8,
];

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMarker3dSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub color: ColorSlot10_1,
    pub first_double: f64,
    pub first_integer: i32,
    pub geometry_entry: Option<ObjectEntry10_1>,
    pub geometry: Option<GeometryBody10_1>,
    pub second_double: f64,
    pub integers: [i32; 2],
    pub doubles: [f64; 3],
    pub double_v2: Option<f64>,
    pub short: u16,
    pub vector_v3: Option<[f64; 3]>,
    pub fields_v5: Option<([f64; 4], u16)>,
    pub short_v6: Option<u16>,
}

pub(super) fn read_simple_marker_3d_symbol(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    key: u32,
) -> io::Result<Option<SimpleMarker3dSymbolBody10_1>> {
    let version = read_signed_symbol_version(reader, 6, "SimpleMarker3DSymbol", reader.position())?;
    let signed = version as i16;
    let color = read_color_slot_10_1(reader, max_bytes, table)?;
    if matches!(color, ColorSlot10_1::Unsupported { .. }) {
        return Ok(None);
    }
    let first_double = reader.read_f64_le()?;
    let first_integer = read_i32_le(reader)?;
    let geometry_entry = (signed < 4)
        .then(|| read_object_entry_10_1(reader, table))
        .transpose()?;
    let geometry = match geometry_entry {
        Some(ObjectEntry10_1::NewObject { class_id, .. }) => {
            match read_geometry_body_10_1(&class_id, reader, max_bytes, table) {
                Ok(body) => Some(body),
                Err(error) if error.kind() == io::ErrorKind::Unsupported => return Ok(None),
                Err(error) => return Err(error),
            }
        }
        _ => None,
    };
    Ok(Some(SimpleMarker3dSymbolBody10_1 {
        key,
        version,
        color,
        first_double,
        first_integer,
        geometry_entry,
        geometry,
        second_double: reader.read_f64_le()?,
        integers: [read_i32_le(reader)?, read_i32_le(reader)?],
        doubles: [
            reader.read_f64_le()?,
            reader.read_f64_le()?,
            reader.read_f64_le()?,
        ],
        double_v2: (signed > 1).then(|| reader.read_f64_le()).transpose()?,
        short: reader.read_u16_le()?,
        vector_v3: (signed > 2)
            .then(|| {
                Ok::<_, io::Error>([
                    reader.read_f64_le()?,
                    reader.read_f64_le()?,
                    reader.read_f64_le()?,
                ])
            })
            .transpose()?,
        fields_v5: (signed > 4)
            .then(|| {
                Ok::<_, io::Error>((
                    [
                        reader.read_f64_le()?,
                        reader.read_f64_le()?,
                        reader.read_f64_le()?,
                        reader.read_f64_le()?,
                    ],
                    reader.read_u16_le()?,
                ))
            })
            .transpose()?,
        short_v6: (signed > 5).then(|| reader.read_u16_le()).transpose()?,
    }))
}
