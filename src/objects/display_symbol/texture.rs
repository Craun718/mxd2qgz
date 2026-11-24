//! _3DSymbols.dll 的纹理填充符号与内嵌材质。

use super::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, is_supported_symbol_class, read_i32_le,
    read_signed_symbol_version, read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::objects::display_color::{ColorSlot10_1, RGB_COLOR_CLASS_ID, read_color_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io;

pub const TEXTURE_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x80, 0x87, 0x73, 0x8d, 0x69, 0xc0, 0xe0, 0x42, 0x9d, 0xfa, 0x2b, 0x7b, 0x61, 0x70, 0x7b, 0xa9,
];
pub const GEOMETRY_MATERIAL_CLASS_ID: [u8; 16] = [
    0x27, 0x4b, 0x6f, 0x0e, 0xd0, 0x2b, 0xd6, 0x11, 0xa4, 0xcc, 0x44, 0x45, 0x53, 0x54, 0x77, 0x77,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryMaterialBody10_1 {
    pub version: u16,
    pub dimensions: [i32; 3],
    pub doubles: [f64; 4],
    pub colors: [ColorSlot10_1; 2],
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureFillSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub material_entry: ObjectEntry10_1,
    pub material: Option<GeometryMaterialBody10_1>,
    pub colors: [ColorSlot10_1; 2],
    pub outline: SymbolSlotFull10_1,
    pub doubles: [f64; 2],
    pub legacy_double: Option<f64>,
    pub integers: [i32; 2],
}

fn supported_header(
    reader: &Reader<'_>,
    table: &RefTable10_1,
    supports: impl FnOnce([u8; 16]) -> bool,
) -> io::Result<bool> {
    Ok(match read_header_10_1(&mut reader.clone())? {
        ObjectHeader::Null => true,
        ObjectHeader::Object { class_id, key } => {
            supports(table.registered_class(key).unwrap_or(class_id))
        }
    })
}

// 外层符号负责暂存游标和键表；未知子槽使整个填充体回滚。
pub(super) fn read_texture_fill_body(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    key: u32,
    depth: usize,
) -> io::Result<Option<TextureFillSymbolBody10_1>> {
    let version = read_signed_symbol_version(reader, 2, "TextureFillSymbol", reader.position())?;
    if !supported_header(reader, table, |class| class == GEOMETRY_MATERIAL_CLASS_ID)? {
        return Ok(None);
    }
    let material_entry = read_object_entry_10_1(reader, table)?;
    let material = if matches!(material_entry, ObjectEntry10_1::NewObject { .. }) {
        let Some(material) = read_geometry_material(reader, table, max_bytes)? else {
            return Ok(None);
        };
        Some(material)
    } else {
        None
    };
    let mut colors = [ColorSlot10_1::Null, ColorSlot10_1::Null];
    for color in &mut colors {
        *color = read_color_slot_10_1(reader, max_bytes, table)?;
        if matches!(color, ColorSlot10_1::Unsupported { .. }) {
            return Ok(None);
        }
    }
    if !supported_header(reader, table, is_supported_symbol_class)? {
        return Ok(None);
    }
    let outline = read_symbol_slot_full_10_1(reader, max_bytes, table, depth)?;
    if slot_full_stop_10_1(&outline, DisplaySlotKind10_1::Symbol).is_some() {
        return Ok(None);
    }
    Ok(Some(TextureFillSymbolBody10_1 {
        key,
        version,
        material_entry,
        material,
        colors,
        outline,
        doubles: [reader.read_f64_le()?, reader.read_f64_le()?],
        legacy_double: ((version as i16) < 2)
            .then(|| reader.read_f64_le())
            .transpose()?,
        integers: [read_i32_le(reader)?, read_i32_le(reader)?],
    }))
}

fn read_geometry_material(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
) -> io::Result<Option<GeometryMaterialBody10_1>> {
    let version = reader.read_u16_le()?;
    if !matches!(version, 1 | 2) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "GeometryMaterial 版本必须为 1 或 2",
        ));
    }
    let dimensions = [
        read_i32_le(reader)?,
        read_i32_le(reader)?,
        read_i32_le(reader)?,
    ];
    let doubles = [
        reader.read_f64_le()?,
        reader.read_f64_le()?,
        reader.read_f64_le()?,
        reader.read_f64_le()?,
    ];
    let mut colors = [ColorSlot10_1::Null, ColorSlot10_1::Null];
    for color in &mut colors {
        if !supported_header(reader, table, |class| class == RGB_COLOR_CLASS_ID)? {
            return Ok(None);
        }
        *color = read_color_slot_10_1(reader, max_bytes, table)?;
    }
    let pixels = if dimensions.iter().all(|&dimension| dimension > 0) {
        let length = if version == 2 {
            usize::try_from(read_i32_le(reader)?).ok()
        } else {
            dimensions.iter().try_fold(1_usize, |length, &dimension| {
                length.checked_mul(dimension as usize)
            })
        };
        let length = length
            .filter(|&length| length <= max_bytes)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "GeometryMaterial 像元长度超限")
            })?;
        reader.read_bytes(length)?.to_vec()
    } else {
        Vec::new()
    };
    Ok(Some(GeometryMaterialBody10_1 {
        version,
        dimensions,
        doubles,
        colors,
        pixels,
    }))
}
