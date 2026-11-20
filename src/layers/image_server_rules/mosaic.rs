//! FunctionRasterDB.dll 10.1 的 MosaicFunction 派生体。

use super::raster_info::{self, RasterInfo10_1, read_pnt_slot};
use super::{RuleSlot10_1, slot, unsupported, unsupported_class, version};
use crate::layers::feature_id_set::{
    FID_SET_CLASS_ID, FeatureIdSetBody10_1, read_fid_set_body_10_1,
};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::geometry::{
    GeometryBody10_1, POINT_CLASS_ID, PointBody10_1, read_geometry_body_10_1, read_point_body_10_1,
};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

const MOSAIC_LOD_INFOS: [u8; 16] = [
    0x14, 0x5e, 0xf1, 0xe3, 0x2a, 0x51, 0xaf, 0x49, 0x80, 0x78, 0xef, 0x31, 0x81, 0x3d, 0x6e, 0xab,
];

#[derive(Debug, Clone, PartialEq)]
pub struct MosaicLodInfos10_1 {
    pub version: u16,
    pub pairs: Vec<[f64; 2]>,
    pub trailing_doubles: [f64; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct MosaicFunction10_1 {
    pub raster_info: RuleSlot10_1<RasterInfo10_1>,
    pub strings: [String; 10],
    pub lod_infos: RuleSlot10_1<MosaicLodInfos10_1>,
    pub doubles: [f64; 2],
    pub shorts: [Option<u16>; 6],
    pub symbol: SymbolSlotFull10_1,
    pub longs: [i32; 2],
    pub sort_value: PropertyValueObjectStream10_1,
    pub point: RuleSlot10_1<PointBody10_1>,
    pub legacy_pnt: Option<ObjectEntry10_1>,
    pub geometry: Option<RuleSlot10_1<GeometryBody10_1>>,
    pub fids: Option<RuleSlot10_1<FeatureIdSetBody10_1>>,
}

pub(super) fn read_payload(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    version: i16,
    depth: usize,
) -> io::Result<MosaicFunction10_1> {
    let raster_info = raster_info::read_slot(r, max, refs, depth)?;
    let mut strings = std::array::from_fn(|_| String::new());
    for text in &mut strings[..5] {
        *text = read_string_10_1(r, max)?;
    }
    let lod_infos = slot(r, refs, |r, _, class_id, _| {
        if class_id != MOSAIC_LOD_INFOS {
            return Err(unsupported_class("IMosaicLODInfos", &class_id));
        }
        read_lod_infos(r, max)
    })?;
    let doubles = [r.read_f64_le()?, r.read_f64_le()?];
    let first_short = r.read_u16_le()?;
    let symbol = read_symbol_slot_full_10_1(r, max, refs, MAX_SYMBOL_NESTING_DEPTH)?;
    if slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol).is_some() {
        return Err(unsupported("MosaicFunction 的 ISymbol 对象体未闭合"));
    }
    let first_long = r.read_u32_le()? as i32;
    strings[5] = read_string_10_1(r, max)?;
    let sort_value = read_padded_variant_object_stream_10_1(r, max, refs)?;
    strings[6] = read_string_10_1(r, max)?;
    strings[7] = read_string_10_1(r, max)?;
    let point = slot(r, refs, |r, refs, class_id, _| {
        if class_id != POINT_CLASS_ID {
            return Err(unsupported_class("MosaicFunction IPoint", &class_id));
        }
        read_point_body_10_1(r, max, refs)
    })?;
    let second_short = r.read_u16_le()?;
    let second_long = r.read_u32_le()? as i32;
    strings[8] = read_string_10_1(r, max)?;
    strings[9] = read_string_10_1(r, max)?;
    let third_short = (version >= 2).then(|| r.read_u16_le()).transpose()?;
    let legacy_pnt = matches!(version, 3 | 4)
        .then(|| read_pnt_slot(r, refs))
        .transpose()?;
    let geometry = (version >= 4)
        .then(|| {
            slot(r, refs, |r, refs, id, _| {
                read_geometry_body_10_1(&id, r, max, refs)
            })
        })
        .transpose()?;
    let fids = (version >= 6)
        .then(|| {
            slot(r, refs, |r, refs, class_id, key| {
                if class_id != FID_SET_CLASS_ID {
                    return Err(unsupported_class("MosaicFunction IFIDSet", &class_id));
                }
                read_fid_set_body_10_1(r, refs, key)
            })
        })
        .transpose()?;
    let shorts = [
        Some(first_short),
        Some(second_short),
        third_short,
        (version >= 7).then(|| r.read_u16_le()).transpose()?,
        (version >= 8).then(|| r.read_u16_le()).transpose()?,
        (version >= 9).then(|| r.read_u16_le()).transpose()?,
    ];
    Ok(MosaicFunction10_1 {
        raster_info,
        strings,
        lod_infos,
        doubles,
        shorts,
        symbol,
        longs: [first_long, second_long],
        sort_value,
        point,
        legacy_pnt,
        geometry,
        fids,
    })
}

fn read_lod_infos(r: &mut Reader<'_>, max: usize) -> io::Result<MosaicLodInfos10_1> {
    let version = version(r, 1, "MosaicLODInfos")?;
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    if count > r.remaining().saturating_sub(16).min(max) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "MosaicLODInfos 项数超限",
        ));
    }
    let pairs = (0..count)
        .map(|_| Ok([r.read_f64_le()?, r.read_f64_le()?]))
        .collect::<io::Result<_>>()?;
    Ok(MosaicLodInfos10_1 {
        version,
        pairs,
        trailing_doubles: [r.read_f64_le()?, r.read_f64_le()?],
    })
}
