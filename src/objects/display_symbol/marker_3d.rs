//! _3DSymbols.dll 的 3D 标记持久化对象。

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

pub const CHARACTER_MARKER_3D_SYMBOL_CLASS_ID: [u8; 16] = [
    0xf7, 0xc8, 0x8e, 0x6e, 0x0a, 0xe9, 0xd5, 0x11, 0xa1, 0x29, 0x00, 0x50, 0x8b, 0xd6, 0x0c, 0xb9,
];

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterMarker3dSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub color: ColorSlot10_1,
    pub leading_integers: [i32; 2],
    pub doubles: [f64; 9],
    pub legacy_doubles: Option<[f64; 2]>,
    pub font: super::StdFontSlot10_1,
    pub integers: [i32; 2],
    pub short: u16,
    pub character_marker: super::SymbolSlotFull10_1,
    pub byte_v2: Option<u8>,
    pub fields_v3: Option<([f64; 3], u16)>,
    pub short_v4: Option<u16>,
}

pub(super) fn read_character_marker_3d_symbol(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    key: u32,
    depth: usize,
) -> io::Result<Option<CharacterMarker3dSymbolBody10_1>> {
    use super::{
        CHARACTER_MARKER_SYMBOL_CLASS_ID, DisplaySlotKind10_1, SymbolSlotFull10_1,
        read_std_font_slot_10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1,
    };
    use crate::storage::object_stream::{ObjectHeader, read_header_10_1};

    let version =
        read_signed_symbol_version(reader, 4, "CharacterMarker3DSymbol", reader.position())?;
    let signed = version as i16;
    let color = read_color_slot_10_1(reader, max_bytes, table)?;
    if matches!(color, ColorSlot10_1::Unsupported { .. }) {
        return Ok(None);
    }
    let leading_integers = [read_i32_le(reader)?, read_i32_le(reader)?];
    let mut doubles = [0.0; 9];
    for field in &mut doubles {
        *field = reader.read_f64_le()?;
    }
    let legacy_doubles = (signed < 3)
        .then(|| Ok::<_, io::Error>([reader.read_f64_le()?, reader.read_f64_le()?]))
        .transpose()?;
    if let ObjectHeader::Object { class_id, key } = read_header_10_1(&mut reader.clone())?
        && table.registered_class(key).unwrap_or(class_id) != super::STDFONT_CLASS_ID
    {
        return Ok(None);
    }
    let (Some(font), None) =
        read_std_font_slot_10_1(reader, max_bytes, table, DisplaySlotKind10_1::Font)?
    else {
        return Ok(None);
    };
    let integers = [read_i32_le(reader)?, read_i32_le(reader)?];
    let short = reader.read_u16_le()?;
    let character_marker = match read_header_10_1(&mut reader.clone())? {
        ObjectHeader::Object { class_id, key } => {
            let class_id = table.registered_class(key).unwrap_or(class_id);
            if class_id != CHARACTER_MARKER_SYMBOL_CLASS_ID {
                return Ok(None);
            }
            if table.contains(key) {
                read_object_entry_10_1(reader, table)?;
                SymbolSlotFull10_1::BackReference { class_id, key }
            } else {
                read_symbol_slot_full_10_1(reader, max_bytes, table, depth)?
            }
        }
        ObjectHeader::Null => {
            read_object_entry_10_1(reader, table)?;
            SymbolSlotFull10_1::Null
        }
    };
    if slot_full_stop_10_1(&character_marker, DisplaySlotKind10_1::MarkerSymbol).is_some() {
        return Ok(None);
    }
    Ok(Some(CharacterMarker3dSymbolBody10_1 {
        key,
        version,
        color,
        leading_integers,
        doubles,
        legacy_doubles,
        font,
        integers,
        short,
        character_marker,
        byte_v2: (signed > 1)
            .then(|| reader.read_bytes(1).map(|bytes| bytes[0]))
            .transpose()?,
        fields_v3: (signed > 2)
            .then(|| {
                Ok::<_, io::Error>((
                    [
                        reader.read_f64_le()?,
                        reader.read_f64_le()?,
                        reader.read_f64_le()?,
                    ],
                    reader.read_u16_le()?,
                ))
            })
            .transpose()?,
        short_v4: (signed > 3).then(|| reader.read_u16_le()).transpose()?,
    }))
}

pub const MARKER_3D_SYMBOL_CLASS_ID: [u8; 16] = [
    0x74, 0x72, 0x3f, 0x77, 0xfb, 0xae, 0xd5, 0x11, 0x81, 0x12, 0x00, 0xc0, 0x4f, 0xa0, 0xad, 0xf8,
];

#[derive(Debug, Clone, PartialEq)]
pub struct Marker3dSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub color: ColorSlot10_1,
    pub first_double: f64,
    pub geometry_entry: ObjectEntry10_1,
    pub geometry: Option<GeometryBody10_1>,
    pub first_integer: i32,
    pub vector: [f64; 3],
    pub first_short: u16,
    pub integers: [i32; 2],
    pub doubles: [f64; 3],
    pub double_v2: Option<f64>,
    pub second_short: u16,
    pub picture: Option<super::StdPictureSlot10_1>,
    pub fields_v4: Option<([f64; 4], u16)>,
    pub legacy_fields: Option<([f64; 2], u16)>,
    pub short_v6: Option<u16>,
    pub short_v8: Option<u16>,
    pub shorts_v9: Option<[u16; 3]>,
    /// v9 从 MultiPatch 点 ID 恢复的内嵌字符串。
    pub string_v9: Option<String>,
    pub string_v10: Option<String>,
}

pub(super) fn read_marker_3d_symbol(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    key: u32,
) -> io::Result<Option<Marker3dSymbolBody10_1>> {
    use super::{DisplaySlotKind10_1, read_std_picture_slot_10_1};

    let version = read_signed_symbol_version(reader, 10, "Marker3DSymbol", reader.position())?;
    let signed = version as i16;
    let color = read_color_slot_10_1(reader, max_bytes, table)?;
    if matches!(color, ColorSlot10_1::Unsupported { .. }) {
        return Ok(None);
    }
    let first_double = reader.read_f64_le()?;
    let geometry_entry = read_object_entry_10_1(reader, table)?;
    let geometry = match geometry_entry {
        ObjectEntry10_1::NewObject { class_id, .. } => {
            match read_geometry_body_10_1(&class_id, reader, max_bytes, table) {
                Ok(body) => Some(body),
                Err(error) if error.kind() == io::ErrorKind::Unsupported => return Ok(None),
                Err(error) => return Err(error),
            }
        }
        _ => None,
    };
    let string_v9 = (signed == 9)
        .then(|| marker_v9_string(geometry.as_ref()))
        .flatten();
    let first_integer = read_i32_le(reader)?;
    let vector = [
        reader.read_f64_le()?,
        reader.read_f64_le()?,
        reader.read_f64_le()?,
    ];
    let first_short = reader.read_u16_le()?;
    let integers = [read_i32_le(reader)?, read_i32_le(reader)?];
    let doubles = [
        reader.read_f64_le()?,
        reader.read_f64_le()?,
        reader.read_f64_le()?,
    ];
    let double_v2 = (signed > 1).then(|| reader.read_f64_le()).transpose()?;
    let second_short = reader.read_u16_le()?;
    let picture = if signed >= 3 {
        let (picture, stop) =
            read_std_picture_slot_10_1(reader, max_bytes, table, DisplaySlotKind10_1::Picture)?;
        if stop.is_some() {
            return Ok(None);
        }
        picture
    } else {
        None
    };
    Ok(Some(Marker3dSymbolBody10_1 {
        key,
        version,
        color,
        first_double,
        geometry_entry,
        geometry,
        first_integer,
        vector,
        first_short,
        integers,
        doubles,
        double_v2,
        second_short,
        picture,
        fields_v4: (signed >= 4)
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
        legacy_fields: matches!(signed, 5 | 6)
            .then(|| {
                Ok::<_, io::Error>((
                    [reader.read_f64_le()?, reader.read_f64_le()?],
                    reader.read_u16_le()?,
                ))
            })
            .transpose()?,
        short_v6: (signed >= 6).then(|| reader.read_u16_le()).transpose()?,
        short_v8: (signed >= 8).then(|| reader.read_u16_le()).transpose()?,
        shorts_v9: (signed >= 9)
            .then(|| {
                Ok::<_, io::Error>([
                    reader.read_u16_le()?,
                    reader.read_u16_le()?,
                    reader.read_u16_le()?,
                ])
            })
            .transpose()?,
        string_v9,
        string_v10: (signed == 10)
            .then(|| crate::stream::native_string::read_string_10_1(reader, max_bytes))
            .transpose()?,
    }))
}

// v9 把点 ID 1–8 的四个字节依次扩展为 UTF-16 码元。
fn marker_v9_string(geometry: Option<&GeometryBody10_1>) -> Option<String> {
    use crate::objects::multi_patch::MultiPatchRecord10_1;

    let Some(GeometryBody10_1::MultiPatch(body)) = geometry else {
        return None;
    };
    let Some(MultiPatchRecord10_1::Mesh(mesh)) = &body.record else {
        return None;
    };
    if mesh.xy.len() != 10 || mesh.point_ids.first() != Some(&0x00b8_342d) {
        return None;
    }
    let bytes = mesh
        .point_ids
        .get(1..9)?
        .iter()
        .flat_map(|id| id.to_be_bytes())
        .take_while(|byte| *byte != 0)
        .map(char::from);
    Some(bytes.collect())
}
