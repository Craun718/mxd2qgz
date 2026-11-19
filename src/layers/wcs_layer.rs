//! RasterLayer.dll 10.1 WCSLayer 外壳与普通 RasterLayer 子对象。

use super::image_server_rules::{RuleSlot10_1, slot, unsupported, unsupported_class};
use super::layer_list::RASTER_LAYER_CLASS_ID;
use super::raster_layer::{
    RasterLayerPrefix10_1, read_complete_bounded_raster_body_10_1, read_inline_raster_body_10_1,
};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

pub const WCS_LAYER_CLASS_ID: [u8; 16] = [
    0xf1, 0x10, 0x9c, 0x23, 0x2a, 0x6e, 0x50, 0x42, 0xb4, 0x3a, 0x79, 0x20, 0xdb, 0x1c, 0xdc, 0x88,
];

#[derive(Debug, Clone, PartialEq)]
pub struct WcsLayer10_1 {
    pub version: i32,
    pub strings: [String; 3],
    pub v14_longs: Option<[i32; 2]>,
    pub v15_string: Option<String>,
    pub v16_fields: Option<(String, PropertyValueObjectStream10_1)>,
    pub v17_string: Option<String>,
    pub raster: RuleSlot10_1<RasterLayerPrefix10_1>,
}

/// 原生在底层 RasterLayer 槽后不再消费字节。外层已定界时，剩余范围可用于 RasterLayer v12。
pub(super) fn read_wcs_layer_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    bounded: bool,
) -> io::Result<WcsLayer10_1> {
    let mut r = reader.clone();
    let mut table = refs.clone();
    let version = r.read_u32_le()? as i32;
    if version > 17 {
        return Err(unsupported(format!("WCSLayer 未支持版本 {version}")));
    }
    let strings = [
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
    ];
    let v14_longs = (version >= 14)
        .then(|| Ok::<_, io::Error>([r.read_u32_le()? as i32, r.read_u32_le()? as i32]))
        .transpose()?;
    let v15_string = (version >= 15)
        .then(|| read_string_10_1(&mut r, max))
        .transpose()?;
    let v16_fields = (version >= 16)
        .then(|| {
            Ok::<_, io::Error>((
                read_string_10_1(&mut r, max)?,
                read_padded_variant_object_stream_10_1(&mut r, max, &mut table)?,
            ))
        })
        .transpose()?;
    let v17_string = (version >= 17)
        .then(|| read_string_10_1(&mut r, max))
        .transpose()?;
    let raster = slot(&mut r, &mut table, |r, refs, cls, _| {
        if cls != RASTER_LAYER_CLASS_ID {
            return Err(unsupported_class("WCSLayer 底层图层", &cls));
        }
        if bounded {
            read_complete_bounded_raster_body_10_1(r, max, refs)
        } else {
            read_inline_raster_body_10_1(r, max, refs)
        }
    })?;
    if bounded && r.remaining() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "WCSLayer 对象体消费与声明长度不一致",
        ));
    }
    *reader = r;
    *refs = table;
    Ok(WcsLayer10_1 {
        version,
        strings,
        v14_longs,
        v15_string,
        v16_fields,
        v17_string,
        raster,
    })
}
