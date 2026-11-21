//! TinLayer.dll 与 TinRenderer.dll 10.1 的普通对象体。

use super::feature_layer_tail::{
    DisplayFilterSlot10_1, FIELD_INFO_CLASS_ID, FieldInfoBody10_1, LEGEND_GROUP_CLASS_ID,
    LegendGroupPrefix10_1, TRANSPARENCY_DISPLAY_FILTER_CLASS_ID, read_field_info_body_10_1,
    read_legend_group_body_10_1, read_transparency_display_filter_10_1,
};
use super::image_server_rules::{RuleSlot10_1, slot};
use super::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use super::legend_item::{
    HORIZONTAL_LEGEND_ITEM_CLASS_ID, HorizontalLegendItemBody10_1,
    read_horizontal_legend_item_body_10_1,
};
use super::raster_layer_tail::{RasterLayerEnvelopeSlot10_1, read_envelope_slot};
use super::scene_layer_properties::{SurfaceName10_1, TIN_NAME_CLASS_ID, read_surface_name};
use crate::objects::color_ramp::{ColorRampSlot10_1, read_color_ramp_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub(crate) const TIN_LAYER_CLASS_ID: [u8; 16] = [
    0x38, 0x8f, 0x30, 0xfe, 0xca, 0xbd, 0xd1, 0x11, 0xa5, 0x23, 0x00, 0x00, 0xf8, 0x77, 0x4f, 0x0f,
];
const NODE: [u8; 16] = [
    0xc2, 0x9a, 0xf1, 0x6e, 0xaa, 0x84, 0xd2, 0x11, 0x81, 0xeb, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];
const EDGE: [u8; 16] = [
    0xab, 0x60, 0xcb, 0x13, 0xa0, 0x88, 0xd2, 0x11, 0x81, 0xeb, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];
const FACE: [u8; 16] = [
    0xf0, 0x6d, 0x2e, 0xf1, 0x4c, 0x38, 0xd2, 0x11, 0xb1, 0xf2, 0x00, 0xc0, 0x4f, 0x8e, 0xde, 0xff,
];

const BREAKLINE: [u8; 16] = [
    0x4f, 0x56, 0x20, 0xba, 0xe3, 0xdb, 0xd2, 0x11, 0x9f, 0x39, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0x19,
];

const CONTOUR: [u8; 16] = [
    0xc2, 0x5d, 0xf7, 0xbe, 0xe6, 0x20, 0x65, 0x49, 0x85, 0xe5, 0xa3, 0xbc, 0x79, 0x74, 0x70, 0xbc,
];

const ASPECT: [u8; 16] = [
    0xae, 0x2d, 0xbd, 0xbe, 0xc7, 0x8b, 0xd2, 0x11, 0x81, 0xec, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];
const SLOPE: [u8; 16] = [
    0x62, 0x1b, 0xfb, 0x91, 0x44, 0x89, 0xd2, 0x11, 0x81, 0xec, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];
const ELEVATION: [u8; 16] = [
    0x1b, 0x31, 0xb7, 0xeb, 0xf8, 0x99, 0xd2, 0x11, 0x81, 0xed, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];

const NODE_ELEVATION: [u8; 16] = [
    0x4c, 0x56, 0x20, 0xba, 0xe3, 0xdb, 0xd2, 0x11, 0x9f, 0x39, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0x19,
];

const NODE_VALUE: [u8; 16] = [
    0xb8, 0x34, 0x29, 0x32, 0x24, 0xb2, 0xd2, 0x11, 0x81, 0xfc, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];
const FACE_VALUE: [u8; 16] = [
    0xc8, 0x45, 0x87, 0xaa, 0x66, 0xaa, 0xd2, 0x11, 0x81, 0xfa, 0x00, 0x10, 0x4b, 0xc4, 0xcd, 0x03,
];

pub(super) const TERRAIN_DIRTY_AREA: [u8; 16] = [
    0xa9, 0xc3, 0x62, 0x33, 0x4a, 0xba, 0x39, 0x4f, 0xa7, 0xb1, 0xfc, 0x44, 0x00, 0xcf, 0x21, 0x02,
];

const TERRAIN_POINT_UNIQUE_VALUE: [u8; 16] = [
    0x40, 0xe0, 0x06, 0x8b, 0xb0, 0x89, 0x57, 0x46, 0x98, 0xa5, 0xb9, 0x42, 0x39, 0x87, 0x45, 0xb8,
];

const TERRAIN_POINT_ELEVATION: [u8; 16] = [
    0x3c, 0x3f, 0x47, 0xd8, 0x65, 0x21, 0xaa, 0x44, 0xb8, 0xfd, 0x34, 0x40, 0x3e, 0x1f, 0x93, 0x54,
];

#[derive(Debug, Clone, PartialEq)]
pub enum TinRenderer10_1 {
    Simple(TinSimpleRenderer10_1),
    Breakline(Box<TinBreaklineRenderer10_1>),
    Value(Box<TinBreaklineRenderer10_1>),
    TerrainPointUniqueValue(Box<TerrainPointUniqueValueRenderer10_1>),
    Contour(TinContourRenderer10_1),
    ClassifiedSurface(Box<TinClassifiedSurfaceRenderer10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinClassifiedSurfaceRenderer10_1 {
    pub version: i16,
    pub label: String,
    pub arrays: [Vec<u64>; 2],
    pub v2_double: Option<u64>,
    pub legend_group: RuleSlot10_1<LegendGroupPrefix10_1>,
    pub legend_item: RuleSlot10_1<HorizontalLegendItemBody10_1>,
    pub legacy_string: Option<String>,
    pub guid_flag: Option<u8>,
    pub guid: Option<[u8; 16]>,
    pub first_short: u16,
    pub color_ramp: Option<ColorRampSlot10_1>,
    pub shorts: [u16; 2],
    pub longs: Option<[i32; 2]>,
    pub double: u64,
    pub label_short: Option<u16>,
    pub trailing_label: String,
    pub trailing_short: u16,
    pub v2_tail_short: Option<u16>,
    pub v3_tail_longs: Option<[i32; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinContourRenderer10_1 {
    pub version: i16,
    pub label: String,
    pub shorts: [u16; 2],
    pub legend_group: RuleSlot10_1<LegendGroupPrefix10_1>,
    pub legend_item: RuleSlot10_1<HorizontalLegendItemBody10_1>,
    pub doubles: [u64; 2],
    pub value: i32,
    pub v2_double: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinBreaklineRenderer10_1 {
    pub version: i16,
    pub symbol: SymbolSlotFull10_1,
    pub legend_groups: Vec<RuleSlot10_1<LegendGroupPrefix10_1>>,
    pub legend_item: RuleSlot10_1<HorizontalLegendItemBody10_1>,
    pub records: Vec<TinBreaklineRecord10_1>,
    pub first_short: u16,
    pub strings: [String; 3],
    pub tail_shorts: [u16; 2],
    pub face_middle_short: Option<u16>,
    pub face_last_short: Option<u16>,
    pub final_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinBreaklineRecord10_1 {
    pub text: String,
    pub values: [i32; 2],
    pub string_flag: i32,
    pub extra_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainPointUniqueValueRenderer10_1 {
    pub base: TinBreaklineRenderer10_1,
    pub attribute: String,
    pub values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinSimpleRenderer10_1 {
    pub version: i16,
    pub symbol: SymbolSlotFull10_1,
    pub legend_group: RuleSlot10_1<LegendGroupPrefix10_1>,
    pub legend_item: RuleSlot10_1<HorizontalLegendItemBody10_1>,
    pub first_short: u16,
    pub face_short: Option<u16>,
    pub label: String,
    pub last_short: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinLayer10_1 {
    pub version: i16,
    pub name: String,
    pub source: SurfaceName10_1,
    pub renderers: Vec<RuleSlot10_1<TinRenderer10_1>>,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub base_shorts: [u16; 3],
    pub base_doubles: [u64; 2],
    pub base_string: String,
    pub filter: Option<RuleSlot10_1<DisplayFilterSlot10_1>>,
    pub v2_shorts: Option<[u16; 3]>,
    pub fields: Vec<RuleSlot10_1<FieldInfoBody10_1>>,
    pub v4_short: Option<u16>,
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub v6_double: Option<u64>,
    pub v7_string: Option<String>,
    pub envelope: Option<RasterLayerEnvelopeSlot10_1>,
    pub v9_string: Option<String>,
    pub v9_short: Option<u16>,
    pub v10_doubles: Option<[u64; 2]>,
    pub v11_double: Option<u64>,
}

fn unsupported(slot: &str) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, format!("TIN 的 {slot} 槽未闭合"))
}

fn read_version(r: &mut Reader<'_>, upper: i16) -> io::Result<i16> {
    let version = r.read_u16_le()? as i16;
    if version > upper {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TIN 版本超出 10.1 上限",
        ));
    }
    Ok(version)
}

fn read_shorts(r: &mut Reader<'_>) -> io::Result<[u16; 3]> {
    Ok([r.read_u16_le()?, r.read_u16_le()?, r.read_u16_le()?])
}

fn read_double(r: &mut Reader<'_>) -> io::Result<u64> {
    r.read_f64_le().map(f64::to_bits)
}

fn read_doubles(r: &mut Reader<'_>) -> io::Result<[u64; 2]> {
    Ok([read_double(r)?, read_double(r)?])
}

fn read_count(r: &mut Reader<'_>, max: usize, min_size: usize) -> io::Result<usize> {
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    if count > r.remaining() / min_size {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "TIN 计数超出剩余字节",
        ));
    }
    if count > max / min_size {
        return Err(io::Error::new(ErrorKind::InvalidData, "TIN 计数超出限额"));
    }
    Ok(count)
}

pub(super) fn read_renderer(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<TinRenderer10_1> {
    if matches!(
        class_id,
        ASPECT | SLOPE | ELEVATION | NODE_ELEVATION | TERRAIN_POINT_ELEVATION
    ) {
        return read_classified_surface(r, max, refs, class_id)
            .map(Box::new)
            .map(TinRenderer10_1::ClassifiedSurface);
    }
    if class_id == CONTOUR {
        let version = read_version(r, 2)?;
        return Ok(TinRenderer10_1::Contour(TinContourRenderer10_1 {
            version,
            label: read_string_10_1(r, max)?,
            shorts: [r.read_u16_le()?, r.read_u16_le()?],
            legend_group: read_legend_group(r, max, refs)?,
            legend_item: read_legend_item(r, max, refs)?,
            doubles: read_doubles(r)?,
            value: r.read_u32_le()? as i32,
            v2_double: (version > 1).then(|| read_double(r)).transpose()?,
        }));
    }
    if !matches!(
        class_id,
        NODE | EDGE
            | FACE
            | BREAKLINE
            | NODE_VALUE
            | FACE_VALUE
            | TERRAIN_DIRTY_AREA
            | TERRAIN_POINT_UNIQUE_VALUE
    ) {
        return Err(unsupported("ITinRenderer"));
    }
    let categorical = matches!(
        class_id,
        BREAKLINE | NODE_VALUE | FACE_VALUE | TERRAIN_POINT_UNIQUE_VALUE
    );
    let upper_version = if class_id == TERRAIN_POINT_UNIQUE_VALUE {
        3
    } else if categorical {
        2
    } else {
        1
    };
    let version = read_version(r, upper_version)?;
    let symbol = read_symbol_slot_full_10_1(r, max, refs, MAX_SYMBOL_NESTING_DEPTH)?;
    if slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol).is_some() {
        return Err(unsupported("ISymbol"));
    }
    if categorical {
        let mut legend_groups = Vec::new();
        for _ in 0..read_count(r, max, 16)? {
            legend_groups.push(read_legend_group(r, max, refs)?);
        }
        let legend_item = read_legend_item(r, max, refs)?;
        let mut records = Vec::new();
        // 每项至少有字符串长度前缀和三个 LONG。
        for _ in 0..read_count(r, max, 16)? {
            let text = read_string_10_1(r, max)?;
            let values = [r.read_u32_le()? as i32, r.read_u32_le()? as i32];
            let string_flag = r.read_u32_le()? as i32;
            records.push(TinBreaklineRecord10_1 {
                text,
                values,
                string_flag,
                extra_text: (string_flag != 0)
                    .then(|| read_string_10_1(r, max))
                    .transpose()?,
            });
        }
        let first_short = r.read_u16_le()?;
        let first_string = read_string_10_1(r, max)?;
        let second_string = read_string_10_1(r, max)?;
        let face_middle_short = (class_id == FACE_VALUE)
            .then(|| r.read_u16_le())
            .transpose()?;
        let third_string = read_string_10_1(r, max)?;
        let body = Box::new(TinBreaklineRenderer10_1 {
            version,
            symbol,
            legend_groups,
            legend_item,
            records,
            first_short,
            strings: [first_string, second_string, third_string],
            tail_shorts: [r.read_u16_le()?, r.read_u16_le()?],
            face_middle_short,
            face_last_short: (class_id == FACE_VALUE)
                .then(|| r.read_u16_le())
                .transpose()?,
            // 只有 v1 从本地化资源取值，其他版本从流取值。
            final_string: (version != 1)
                .then(|| read_string_10_1(r, max))
                .transpose()?,
        });
        return Ok(match class_id {
            BREAKLINE => TinRenderer10_1::Breakline(body),
            TERRAIN_POINT_UNIQUE_VALUE => {
                let attribute = read_string_10_1(r, max)?;
                let mut values = Vec::new();
                for _ in 0..read_count(r, max, 4)? {
                    values.push(r.read_u32_le()? as i32);
                }
                TinRenderer10_1::TerrainPointUniqueValue(Box::new(
                    TerrainPointUniqueValueRenderer10_1 {
                        base: *body,
                        attribute,
                        values,
                    },
                ))
            }
            _ => TinRenderer10_1::Value(body),
        });
    }

    // Node 与 Edge 共用 Load；Face 在尾部多一个 short。
    Ok(TinRenderer10_1::Simple(TinSimpleRenderer10_1 {
        version,
        symbol,
        legend_group: read_legend_group(r, max, refs)?,
        legend_item: read_legend_item(r, max, refs)?,
        first_short: r.read_u16_le()?,
        face_short: (class_id == FACE).then(|| r.read_u16_le()).transpose()?,
        label: read_string_10_1(r, max)?,
        last_short: r.read_u16_le()?,
    }))
}

fn read_longs(r: &mut Reader<'_>) -> io::Result<[i32; 2]> {
    Ok([r.read_u32_le()? as i32, r.read_u32_le()? as i32])
}

fn read_classified_surface(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<TinClassifiedSurfaceRenderer10_1> {
    let terrain = class_id == TERRAIN_POINT_ELEVATION;
    let version = read_version(r, if terrain { 3 } else { 2 })?;
    let label = read_string_10_1(r, max)?;
    // 单个计数供两个相邻 double 数组共用。
    let count = read_count(r, max, 16)?;
    let mut arrays = [Vec::new(), Vec::new()];
    for array in &mut arrays {
        for _ in 0..count {
            array.push(read_double(r)?);
        }
    }
    let v2_double = (terrain || version >= 2)
        .then(|| read_double(r))
        .transpose()?;
    let legend_group = read_legend_group(r, max, refs)?;
    let legend_item = read_legend_item(r, max, refs)?;
    let legacy_string = (!terrain && version < 2)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let guid_flag = (terrain || version >= 2)
        .then(|| r.read_bytes(1).map(|bytes| bytes[0]))
        .transpose()?;
    let guid = if guid_flag.is_some_and(|flag| flag != 0) {
        let mut guid = [0; 16];
        guid.copy_from_slice(r.read_bytes(16)?);
        Some(guid)
    } else {
        None
    };
    let first_short = r.read_u16_le()?;
    let offset = r.position();
    let (color_ramp, stop) =
        read_color_ramp_slot_10_1(r, max, refs, MAX_SYMBOL_NESTING_DEPTH, offset)?;
    if stop.is_some() {
        return Err(unsupported("IColorRamp"));
    }
    Ok(TinClassifiedSurfaceRenderer10_1 {
        version,
        label,
        arrays,
        v2_double,
        legend_group,
        legend_item,
        legacy_string,
        guid_flag,
        guid,
        first_short,
        color_ramp,
        shorts: [r.read_u16_le()?, r.read_u16_le()?],
        longs: (!terrain).then(|| read_longs(r)).transpose()?,
        double: read_double(r)?,
        label_short: (!terrain && class_id != NODE_ELEVATION)
            .then(|| r.read_u16_le())
            .transpose()?,
        trailing_label: read_string_10_1(r, max)?,
        trailing_short: r.read_u16_le()?,
        v2_tail_short: (terrain && version >= 2)
            .then(|| r.read_u16_le())
            .transpose()?,
        v3_tail_longs: (terrain && version >= 3)
            .then(|| read_longs(r))
            .transpose()?,
    })
}

pub(super) fn read_legend_group(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<LegendGroupPrefix10_1>> {
    slot(r, refs, |r, refs, class_id, key| {
        if class_id != LEGEND_GROUP_CLASS_ID {
            return Err(unsupported("ILegendGroup"));
        }
        let group = read_legend_group_body_10_1(r, max, refs, key)?;
        if group.stop.is_some() {
            return Err(unsupported("ILegendGroup"));
        }
        Ok(group)
    })
}

fn read_legend_item(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<HorizontalLegendItemBody10_1>> {
    slot(r, refs, |r, refs, class_id, key| {
        if class_id != HORIZONTAL_LEGEND_ITEM_CLASS_ID {
            return Err(unsupported("ILegendItem"));
        }
        let item = read_horizontal_legend_item_body_10_1(r, max, refs, key)?;
        if item.stop.is_some() {
            return Err(unsupported("ILegendItem"));
        }
        Ok(item)
    })
}

pub(crate) fn read_tin_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TinLayer10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let r = &mut trial;
    let version = read_version(r, 12)?;
    let name = read_string_10_1(r, max)?;
    let name_class = r.clone().read_bytes(16)?;
    if name_class != TIN_NAME_CLASS_ID && name_class != [0; 16] {
        return Err(unsupported("IName"));
    }
    let source = read_surface_name(r, max, &mut keys)?;
    let mut renderers = Vec::new();
    for _ in 0..read_count(r, max, 16)? {
        renderers.push(slot(r, &mut keys, |r, refs, class_id, _| {
            read_renderer(r, max, refs, class_id)
        })?);
    }
    let spatial_reference = read_spatial_reference_entry_10_1(r, max, &mut keys)?;
    let base_shorts = read_shorts(r)?;
    let base_doubles = read_doubles(r)?;
    let base_string = read_string_10_1(r, max)?;
    let filter = (version >= 2)
        .then(|| {
            slot(r, &mut keys, |r, _, class_id, key| {
                if class_id != TRANSPARENCY_DISPLAY_FILTER_CLASS_ID {
                    return Err(unsupported("IDisplayFilter"));
                }
                read_transparency_display_filter_10_1(r, key)
            })
        })
        .transpose()?;
    let v2_shorts = (version >= 2).then(|| read_shorts(r)).transpose()?;
    let mut fields = Vec::new();
    if version >= 3 {
        for _ in 0..read_count(r, max, 16)? {
            fields.push(slot(r, &mut keys, |r, refs, class_id, key| {
                if class_id != FIELD_INFO_CLASS_ID {
                    return Err(unsupported("IFieldInfo"));
                }
                let field = read_field_info_body_10_1(r, max, refs, key)?;
                if field.stop.is_some() {
                    return Err(unsupported("IFieldInfo"));
                }
                Ok(field)
            })?);
        }
    }
    let v4_short = (version >= 4).then(|| r.read_u16_le()).transpose()?;
    let extensions = if version >= 5 {
        read_layer_extensions_10_1(r, max, &mut keys)?
    } else {
        Vec::new()
    };
    let layer = TinLayer10_1 {
        version,
        name,
        source,
        renderers,
        spatial_reference,
        base_shorts,
        base_doubles,
        base_string,
        filter,
        v2_shorts,
        fields,
        v4_short,
        extensions,
        v6_double: (version >= 6).then(|| read_double(r)).transpose()?,
        v7_string: (version >= 7)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
        envelope: (version >= 8)
            .then(|| read_envelope_slot(r, max, &mut keys))
            .transpose()?,
        v9_string: (version >= 9)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
        v9_short: (version == 9).then(|| r.read_u16_le()).transpose()?,
        v10_doubles: (version >= 10).then(|| read_doubles(r)).transpose()?,
        v11_double: (version >= 11).then(|| read_double(r)).transpose()?,
    };
    *reader = trial;
    *refs = keys;
    Ok(layer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::layer_list::{LayerBodyPrefix10_1, parse_body_prefix};

    use crate::objects::color_ramp::ALGORITHMIC_COLOR_RAMP_CLASS_ID;
    use crate::objects::display_symbol::SIMPLE_LINE_SYMBOL_CLASS_ID;

    fn string(bytes: &mut Vec<u8>, value: &str) {
        let units: Vec<_> = value.encode_utf16().chain([0]).collect();
        bytes.extend(((units.len() * 2) as u32).to_le_bytes());
        bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
    }

    fn layer(version: i16) -> Vec<u8> {
        let mut bytes = version.to_le_bytes().to_vec();
        string(&mut bytes, "地形");
        bytes.extend(TIN_NAME_CLASS_ID);
        bytes.extend(10_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["tin", "", ""] {
            string(&mut bytes, value);
        }
        bytes.extend([0; 16]);
        bytes.extend(4_i32.to_le_bytes());
        for (class, key) in [(NODE, 11_u32), (EDGE, 12), (FACE, 13)] {
            bytes.extend(class);
            bytes.extend(key.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 16 * 3]);
            bytes.extend(21_u16.to_le_bytes());
            if class == FACE {
                bytes.extend(22_u16.to_le_bytes());
            }
            string(&mut bytes, "图例");
            bytes.extend(23_u16.to_le_bytes());
        }
        bytes.extend(FACE);
        bytes.extend(13_u32.to_le_bytes());
        bytes.extend([0; 16 + 6 + 16]);
        string(&mut bytes, "");
        if version >= 2 {
            bytes.extend([0; 16 + 6]);
        }
        if version >= 3 {
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend([0; 16]);
        }
        if version >= 4 {
            bytes.extend(24_u16.to_le_bytes());
        }
        if version >= 5 {
            bytes.extend(0_u32.to_le_bytes());
        }
        if version >= 6 {
            bytes.extend(6.0_f64.to_le_bytes());
        }
        if version >= 7 {
            string(&mut bytes, "路径");
        }
        if version >= 8 {
            bytes.extend([0; 16]);
        }
        if version >= 9 {
            string(&mut bytes, "说明");
        }
        if version == 9 {
            bytes.extend(25_u16.to_le_bytes());
        }
        if version >= 10 {
            bytes.extend(10.0_f64.to_le_bytes());
            bytes.extend(10.5_f64.to_le_bytes());
        }
        if version >= 11 {
            bytes.extend(11.0_f64.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn dispatch_reads_versions_and_renderer_backrefs_to_the_body_boundary() {
        for version in -1..=12 {
            let mut bytes = layer(version);
            bytes.push(0xee);
            let mut r = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let Some(LayerBodyPrefix10_1::TinLayer(body)) =
                parse_body_prefix(&mut r, TIN_LAYER_CLASS_ID, 4096, &mut refs, 0).unwrap()
            else {
                panic!("TinLayer 未分派");
            };
            assert_eq!(body.version, version);
            assert_eq!(body.name, "地形");
            assert!(refs.contains(10));
            assert!(refs.contains(13));
            assert_eq!(body.renderers.len(), 4);
            let Some(TinRenderer10_1::Simple(node)) = body.renderers[0].body.as_deref() else {
                panic!("Node 体未读取");
            };
            let Some(TinRenderer10_1::Simple(face)) = body.renderers[2].body.as_deref() else {
                panic!("Face 体未读取");
            };
            assert_eq!(node.face_short, None);
            assert_eq!(face.face_short, Some(22));
            assert_eq!(face.label, "图例");
            assert!(body.renderers[3].body.is_none());
            assert_eq!(body.fields.len(), usize::from(version >= 3));
            assert_eq!(body.v9_short, (version == 9).then_some(25));
            assert_eq!(
                body.v11_double,
                (version >= 11).then_some(11.0_f64.to_bits())
            );
            assert_eq!(r.read_bytes(1).unwrap(), &[0xee]);
        }
    }

    #[test]
    fn categorical_renderers_read_nested_groups_and_versioned_strings() {
        for class in [BREAKLINE, NODE_VALUE, FACE_VALUE] {
            for version in [0_i16, 1, 2] {
                let mut renderer = class.to_vec();
                renderer.extend(14_u32.to_le_bytes());
                renderer.extend(version.to_le_bytes());
                renderer.extend([0; 16]);
                renderer.extend(2_i32.to_le_bytes());
                for index in 0..2 {
                    renderer.extend(LEGEND_GROUP_CLASS_ID);
                    renderer.extend(15_u32.to_le_bytes());
                    if index == 0 {
                        renderer.extend([1, 0, 0, 0]);
                        string(&mut renderer, "分组");
                        renderer.extend(0_i32.to_le_bytes());
                    }
                }
                renderer.extend([0; 16]);
                let records_offset = renderer.len();
                renderer.extend(2_i32.to_le_bytes());
                for flag in [0_i32, -1] {
                    string(&mut renderer, "折线");
                    for value in [-2_i32, 3, flag] {
                        renderer.extend(value.to_le_bytes());
                    }
                    if flag != 0 {
                        string(&mut renderer, "补充");
                    }
                }
                renderer.extend(7_u16.to_le_bytes());
                string(&mut renderer, "甲");
                string(&mut renderer, "乙");
                if class == FACE_VALUE {
                    renderer.extend(11_u16.to_le_bytes());
                }
                string(&mut renderer, "丙");
                renderer.extend([8, 0, 9, 0]);
                if class == FACE_VALUE {
                    renderer.extend(12_u16.to_le_bytes());
                }
                if version != 1 {
                    string(&mut renderer, "末尾");
                }

                let mut bytes = layer(12);
                let offset = bytes.windows(16).position(|window| window == NODE).unwrap();
                bytes[offset - 4..offset].copy_from_slice(&5_i32.to_le_bytes());
                bytes.splice(offset..offset, renderer);
                let mut r = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let body = read_tin_layer_10_1(&mut r, 4096, &mut refs).unwrap();
                assert_eq!(r.remaining(), 0);
                assert!(refs.contains(15));
                let breakline = match (class, body.renderers[0].body.as_deref()) {
                    (BREAKLINE, Some(TinRenderer10_1::Breakline(body))) => body,
                    (NODE_VALUE | FACE_VALUE, Some(TinRenderer10_1::Value(body))) => body,
                    _ => panic!("分类渲染器体未读取"),
                };
                assert_eq!(
                    breakline.legend_groups[0].body.as_ref().unwrap().heading,
                    "分组"
                );
                assert!(breakline.legend_groups[1].body.is_none());
                assert_eq!(breakline.records[0].values, [-2, 3]);
                assert_eq!(breakline.records[0].extra_text, None);
                assert_eq!(breakline.records[1].extra_text.as_deref(), Some("补充"));
                assert_eq!(breakline.tail_shorts, [8, 9]);
                assert_eq!(breakline.strings, ["甲", "乙", "丙"]);
                assert_eq!(
                    breakline.face_middle_short,
                    (class == FACE_VALUE).then_some(11)
                );
                assert_eq!(
                    breakline.face_last_short,
                    (class == FACE_VALUE).then_some(12)
                );
                assert_eq!(
                    breakline.final_string.as_deref(),
                    (version != 1).then_some("末尾")
                );

                let bad_count = offset + records_offset;
                bytes[bad_count..bad_count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
                let mut r = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(read_tin_layer_10_1(&mut r, 4096, &mut refs).is_err());
                assert_eq!(r.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn contour_reads_versioned_double_without_crossing_body_boundary() {
        for version in [0_i16, 1, 2] {
            let mut bytes = version.to_le_bytes().to_vec();
            string(&mut bytes, "等高线");
            bytes.extend([3, 0, 4, 0]);
            bytes.extend([0; 32]);
            bytes.extend(5.0_f64.to_le_bytes());
            bytes.extend(6.0_f64.to_le_bytes());
            bytes.extend((-7_i32).to_le_bytes());
            if version == 2 {
                bytes.extend(8.0_f64.to_le_bytes());
            }
            bytes.push(0xee);

            let mut r = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let TinRenderer10_1::Contour(body) =
                read_renderer(&mut r, 4096, &mut refs, CONTOUR).unwrap()
            else {
                panic!("Contour 体未读取");
            };
            assert_eq!(body.label, "等高线");
            assert_eq!(body.shorts, [3, 4]);
            assert_eq!(body.doubles, [5.0_f64.to_bits(), 6.0_f64.to_bits()]);
            assert_eq!(body.value, -7);
            assert_eq!(body.v2_double, (version == 2).then_some(8.0_f64.to_bits()));
            assert_eq!(r.read_bytes(1).unwrap(), &[0xee]);

            for end in 0..bytes.len() - 1 {
                let mut r = Reader::new(&bytes[..end]);
                assert!(read_renderer(&mut r, 4096, &mut refs, CONTOUR).is_err());
            }
        }
    }

    #[test]
    fn edge_consumes_new_symbol_body() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(SIMPLE_LINE_SYMBOL_CLASS_ID);
        bytes.extend(31_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 16 + 8 + 12]);
        bytes.extend([0; 32 + 2]);
        string(&mut bytes, "线");
        bytes.extend(9_u16.to_le_bytes());
        let mut r = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let TinRenderer10_1::Simple(body) = read_renderer(&mut r, 4096, &mut refs, EDGE).unwrap()
        else {
            panic!("Edge 体未读取");
        };
        assert!(matches!(
            body.symbol,
            SymbolSlotFull10_1::SimpleLineSymbol(_)
        ));
        assert!(refs.contains(31));
        assert_eq!(body.last_short, 9);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn classified_surfaces_read_arrays_and_versioned_guid() {
        for class in [ASPECT, SLOPE, ELEVATION, NODE_ELEVATION] {
            for (version, flag) in [(1_i16, None), (2, Some(0_u8)), (2, Some(1))] {
                let mut bytes = version.to_le_bytes().to_vec();
                string(&mut bytes, "分级");
                bytes.extend(2_i32.to_le_bytes());
                for value in [1.0_f64, 2.0, 3.0, 4.0] {
                    bytes.extend(value.to_le_bytes());
                }
                if version == 2 {
                    bytes.extend(5.0_f64.to_le_bytes());
                }
                bytes.extend([0; 32]);
                match flag {
                    None => string(&mut bytes, "旧字段"),
                    Some(flag) => {
                        bytes.push(flag);
                        if flag != 0 {
                            bytes.extend([0xab; 16]);
                        }
                    }
                }
                bytes.extend(6_u16.to_le_bytes());
                bytes.extend(ALGORITHMIC_COLOR_RAMP_CLASS_ID);
                bytes.extend(30_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 4 + 4 + 32]);
                bytes.extend([7, 0, 8, 0]);
                bytes.extend(9_i32.to_le_bytes());
                bytes.extend((-10_i32).to_le_bytes());
                bytes.extend(11.0_f64.to_le_bytes());
                if class != NODE_ELEVATION {
                    bytes.extend(12_u16.to_le_bytes());
                }
                string(&mut bytes, "末尾");
                bytes.extend(13_u16.to_le_bytes());
                bytes.push(0xee);

                let mut refs = RefTable10_1::new();
                let mut r = Reader::new(&bytes);
                let TinRenderer10_1::ClassifiedSurface(body) =
                    read_renderer(&mut r, 4096, &mut refs, class).unwrap()
                else {
                    panic!("分级渲染器未读取");
                };
                assert_eq!(body.arrays[0], [1.0_f64.to_bits(), 2.0_f64.to_bits()]);
                assert_eq!(body.arrays[1], [3.0_f64.to_bits(), 4.0_f64.to_bits()]);
                assert_eq!(body.v2_double, (version == 2).then_some(5.0_f64.to_bits()));
                assert_eq!(
                    body.legacy_string.as_deref(),
                    (version == 1).then_some("旧字段")
                );
                assert_eq!(body.guid_flag, flag);
                assert_eq!(body.guid, (flag == Some(1)).then_some([0xab; 16]));
                assert!(matches!(
                    body.color_ramp,
                    Some(ColorRampSlot10_1::Algorithmic(_))
                ));
                assert!(refs.contains(30));
                assert_eq!(body.longs, Some([9, -10]));
                assert_eq!(body.label_short, (class != NODE_ELEVATION).then_some(12));
                assert_eq!(body.trailing_label, "末尾");
                assert_eq!(body.trailing_short, 13);
                assert_eq!(r.read_bytes(1).unwrap(), &[0xee]);
                for end in 0..bytes.len() - 1 {
                    let mut r = Reader::new(&bytes[..end]);
                    assert!(read_renderer(&mut r, 4096, &mut RefTable10_1::new(), class).is_err());
                }
            }
        }
    }

    #[test]
    fn terrain_point_unique_value_consumes_attribute_and_signed_values() {
        for version in [1_u16, 2, 3] {
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend([0; 16 + 4 + 16 + 4 + 2]);
            for value in ["甲", "乙", "丙"] {
                string(&mut bytes, value);
            }
            bytes.extend([0; 4]);
            if version != 1 {
                string(&mut bytes, "末尾");
            }
            string(&mut bytes, "分类字段");
            let count_offset = bytes.len();
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend((-3_i32).to_le_bytes());
            bytes.extend(8_i32.to_le_bytes());
            let length = bytes.len();
            bytes.push(0xee);
            let mut r = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let TinRenderer10_1::TerrainPointUniqueValue(body) =
                read_renderer(&mut r, 4096, &mut refs, TERRAIN_POINT_UNIQUE_VALUE).unwrap()
            else {
                panic!("TerrainPointUniqueValueRenderer 未分派");
            };
            assert_eq!(body.base.version, version as i16);
            assert_eq!(body.attribute, "分类字段");
            assert_eq!(body.values, [-3, 8]);
            assert_eq!(r.read_bytes(1).unwrap(), &[0xee]);
            for end in 0..length {
                let mut r = Reader::new(&bytes[..end]);
                assert!(
                    read_renderer(&mut r, 4096, &mut refs, TERRAIN_POINT_UNIQUE_VALUE).is_err()
                );
            }
            bytes[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            assert!(
                read_renderer(
                    &mut Reader::new(&bytes),
                    4096,
                    &mut refs,
                    TERRAIN_POINT_UNIQUE_VALUE
                )
                .is_err()
            );
        }
    }

    #[test]
    fn terrain_point_elevation_reads_guid_and_double_in_v1_and_versioned_tail() {
        for version in 1_u16..=3 {
            let mut bytes = version.to_le_bytes().to_vec();
            string(&mut bytes, "点高程");
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend([0; 32]);
            bytes.push(1);
            bytes.extend([3; 16]);
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend([5, 0, 6, 0]);
            bytes.extend(7.0_f64.to_le_bytes());
            string(&mut bytes, "末尾");
            bytes.extend(8_u16.to_le_bytes());
            if version >= 2 {
                bytes.extend(9_u16.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend((-10_i32).to_le_bytes());
                bytes.extend(11_i32.to_le_bytes());
            }
            let length = bytes.len();
            bytes.push(0xee);
            let mut r = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let TinRenderer10_1::ClassifiedSurface(body) =
                read_renderer(&mut r, 4096, &mut refs, TERRAIN_POINT_ELEVATION).unwrap()
            else {
                panic!("TerrainPointElevationRenderer 未分派");
            };
            assert_eq!(body.v2_double, Some(2.0_f64.to_bits()));
            assert_eq!(body.guid, Some([3; 16]));
            assert_eq!(body.longs, None);
            assert_eq!(body.label_short, None);
            assert_eq!(body.trailing_short, 8);
            assert_eq!(body.v2_tail_short, (version >= 2).then_some(9));
            assert_eq!(body.v3_tail_longs, (version >= 3).then_some([-10, 11]));
            assert_eq!(r.read_bytes(1).unwrap(), &[0xee]);
            for end in 0..length {
                let mut r = Reader::new(&bytes[..end]);
                assert!(read_renderer(&mut r, 4096, &mut refs, TERRAIN_POINT_ELEVATION).is_err());
            }
        }
    }

    #[test]
    fn invalid_body_does_not_commit_cursor_or_keys() {
        let bytes = layer(12);
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut r = Reader::new(&bytes[..end]);
            assert!(
                read_tin_layer_10_1(&mut r, 4096, &mut refs).is_err(),
                "{end}"
            );
            assert_eq!(r.position(), 0);
            assert!(refs.is_empty());
        }
        let mut unknown = bytes.clone();
        let offset = unknown
            .windows(16)
            .position(|window| window == NODE)
            .unwrap();
        unknown[offset] ^= 1;
        for (bytes, error) in [
            (unknown, ErrorKind::Unsupported),
            (layer(13), ErrorKind::InvalidData),
        ] {
            let mut r = Reader::new(&bytes);
            assert_eq!(
                read_tin_layer_10_1(&mut r, 4096, &mut refs)
                    .unwrap_err()
                    .kind(),
                error
            );
            assert_eq!(r.position(), 0);
            assert!(refs.is_empty());
        }
    }
}
