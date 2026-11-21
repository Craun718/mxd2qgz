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
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1,
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

#[derive(Debug, Clone, PartialEq)]
pub struct TinRenderer10_1 {
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

fn read_count(r: &mut Reader<'_>, max: usize) -> io::Result<usize> {
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    // 空槽是最短的普通对象条目，仅含 16 字节 CLSID。
    if count > r.remaining() / 16 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "TIN 对象计数超出剩余字节",
        ));
    }
    if count > max / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TIN 对象计数超出限额",
        ));
    }
    Ok(count)
}

fn read_renderer(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<TinRenderer10_1> {
    // Node 与 Edge 共用 Load；Face 仅在尾部多一个 short。
    if !matches!(class_id, NODE | EDGE | FACE) {
        return Err(unsupported("ITinRenderer"));
    }
    let version = read_version(r, 1)?;
    let symbol = read_symbol_slot_full_10_1(r, max, refs, 0)?;
    if slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol).is_some() {
        return Err(unsupported("ISymbol"));
    }
    let legend_group = slot(r, refs, |r, refs, class_id, key| {
        if class_id != LEGEND_GROUP_CLASS_ID {
            return Err(unsupported("ILegendGroup"));
        }
        let group = read_legend_group_body_10_1(r, max, refs, key)?;
        if group.stop.is_some() {
            return Err(unsupported("ILegendGroup"));
        }
        Ok(group)
    })?;
    let legend_item = slot(r, refs, |r, refs, class_id, key| {
        if class_id != HORIZONTAL_LEGEND_ITEM_CLASS_ID {
            return Err(unsupported("ILegendItem"));
        }
        let item = read_horizontal_legend_item_body_10_1(r, max, refs, key)?;
        if item.stop.is_some() {
            return Err(unsupported("ILegendItem"));
        }
        Ok(item)
    })?;
    Ok(TinRenderer10_1 {
        version,
        symbol,
        legend_group,
        legend_item,
        first_short: r.read_u16_le()?,
        face_short: (class_id == FACE).then(|| r.read_u16_le()).transpose()?,
        label: read_string_10_1(r, max)?,
        last_short: r.read_u16_le()?,
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
    for _ in 0..read_count(r, max)? {
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
        for _ in 0..read_count(r, max)? {
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
            assert_eq!(body.renderers[0].body.as_ref().unwrap().face_short, None);
            assert_eq!(
                body.renderers[2].body.as_ref().unwrap().face_short,
                Some(22)
            );
            assert_eq!(body.renderers[2].body.as_ref().unwrap().label, "图例");
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
