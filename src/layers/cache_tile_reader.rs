//! ArcMap 10.1 瓦片读取器及缓存配置。原生入口见栅格层尾段研究记录。

use super::image_server_rules::{RuleSlot10_1, slot, unsupported, unsupported_class};
use super::raster_dataset_name::{RasterNameSlot10_1, read_name_slot};
use super::raster_layer_tail::{RasterLayerEnvelopeSlot10_1, read_envelope_slot};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::objects::geometry::{POINT_CLASS_ID, PointBody10_1, read_point_body_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

mod ags_name;
mod provider;
pub use ags_name::{AgsServerConnectionName10_1, AgsServerObjectName10_1};
pub use provider::{
    CacheInfo10_1, INTERNET_TILED_LAYER_CLASS_ID, ImsProvider10_1, InternetTileProvider10_1,
    InternetTiledLayer10_1, InternetTiledLayerV2Fields10_1, TiledLayer10_1, WMTS_LAYER_CLASS_ID,
    WebTileProvider10_1, WmsProvider10_1, WmtsLayer10_1, WmtsProvider10_1,
};
pub(crate) use provider::{is_tiled_layer, read_tiled_layer_10_1};

const INTERNET_TILE_READER: [u8; 16] = [
    0x01, 0xa2, 0x4d, 0x03, 0xd6, 0x5c, 0x4a, 0x47, 0xbf, 0xa8, 0x55, 0xba, 0x06, 0xeb, 0x3c, 0x4d,
];
const MAP_SERVER_REST_TILE_READER: [u8; 16] = [
    0x60, 0x85, 0xd8, 0x27, 0x8e, 0x86, 0x69, 0x48, 0xbf, 0x54, 0x7e, 0xec, 0xa5, 0x22, 0x59, 0x4e,
];
const MAP_SERVER_TILE_READER: [u8; 16] = [
    0x79, 0x83, 0x00, 0xdf, 0x82, 0x1c, 0x42, 0x4e, 0xb3, 0xa9, 0xf1, 0x2a, 0x56, 0x48, 0xd7, 0xf0,
];
const MAP_TILE_READER: [u8; 16] = [
    0x16, 0xec, 0xb2, 0x48, 0xdb, 0xb0, 0x85, 0x4d, 0x9a, 0x86, 0x1d, 0xdb, 0x04, 0x04, 0x85, 0x79,
];
const TILE_CACHE_INFO: [u8; 16] = [
    0x17, 0x20, 0x31, 0x01, 0x38, 0x7d, 0x2b, 0x4d, 0x91, 0xa0, 0x05, 0xc5, 0x48, 0xad, 0xe7, 0xf3,
];
const TILE_IMAGE_INFO: [u8; 16] = [
    0xd5, 0x87, 0xa7, 0x0c, 0xae, 0xfc, 0x2d, 0x41, 0x85, 0xd8, 0x61, 0xaa, 0x30, 0x4b, 0x1a, 0xe1,
];
const LOD_INFOS: [u8; 16] = [
    0x51, 0x9a, 0xd1, 0x8c, 0x4b, 0x33, 0x0a, 0x4c, 0xbc, 0x97, 0xa3, 0xfd, 0xea, 0x0a, 0x1c, 0xe0,
];
const LOD_INFO: [u8; 16] = [
    0x76, 0x0f, 0xbd, 0x35, 0xf3, 0x3c, 0x0c, 0x4d, 0x9c, 0x4a, 0xd7, 0x28, 0xcf, 0xe5, 0x93, 0xa5,
];

#[derive(Debug, Clone, PartialEq)]
pub enum CacheTileReaderBody10_1 {
    Internet {
        version: i16,
        strings: [String; 2],
        flag: u8,
        longs: [i32; 3],
        config_name: String,
        config: CacheTileConfiguration10_1,
        provider: ObjectEntry10_1,
        provider_body: Option<Box<InternetTileProvider10_1>>,
    },
    MapServer {
        rest: bool,
        version: i16,
        flag: u8,
        color: ColorSlot10_1,
        config_name: String,
        config: CacheTileConfiguration10_1,
        server_name: ObjectEntry10_1,
        server_name_body: Option<Box<AgsServerObjectName10_1>>,
        rest_selector: Option<i16>,
        inline_rest: Option<Box<MapServerRestBody10_1>>,
    },
    MapTile {
        version: i16,
        strings: [String; 2],
        shorts: [i16; 2],
        path: String,
        name: Box<RasterNameSlot10_1>,
        selector: i16,
        config: Option<CacheTileConfiguration10_1>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapServerRestBody10_1 {
    pub version: i16,
    pub flag: i16,
    pub strings: [String; 4],
    pub json: [Vec<u8>; 3],
    pub extra_string: Option<String>,
    pub extra_json: Option<[Vec<u8>; 3]>,
    pub name: Option<RasterNameSlot10_1>,
    pub properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CacheTileConfiguration10_1 {
    pub extent: RasterLayerEnvelopeSlot10_1,
    pub cache: RuleSlot10_1<TileCacheInfo10_1>,
    pub image: RuleSlot10_1<TileImageInfo10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileCacheInfo10_1 {
    pub version: i16,
    pub origin: RuleSlot10_1<PointBody10_1>,
    pub dimensions: [i32; 2],
    pub levels: RuleSlot10_1<TileLodInfos10_1>,
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
    pub dpi: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileImageInfo10_1 {
    pub format: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileLodInfos10_1 {
    pub version: i16,
    pub count: i32,
    pub items: Vec<RuleSlot10_1<TileLodInfo10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileLodInfo10_1 {
    pub version: i16,
    pub level: i32,
    pub doubles: [f64; 2],
}

fn version(r: &mut Reader<'_>, max: i16, class: &str) -> io::Result<i16> {
    let version = r.read_u16_le()? as i16;
    if version > max {
        return Err(unsupported(format!("{class} 未支持版本 {version}")));
    }
    Ok(version)
}

/// 父级 CacheRasterDatasetName 统一提交游标和对象表。
pub(super) fn read_tile_reader(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class: [u8; 16],
    depth: usize,
) -> io::Result<CacheTileReaderBody10_1> {
    match class {
        INTERNET_TILE_READER => {
            let version = version(r, 1, "InternetTileReader")?;
            let strings = [read_string_10_1(r, max)?, read_string_10_1(r, max)?];
            let flag = r.read_bytes(1)?[0];
            let longs = [
                r.read_u32_le()? as i32,
                r.read_u32_le()? as i32,
                r.read_u32_le()? as i32,
            ];
            let config_name = read_string_10_1(r, max)?;
            let config = read_config(r, max, refs)?;
            let provider = slot(r, refs, |r, refs, class, _| {
                provider::read_provider(r, max, refs, class, depth)
            })?;
            Ok(CacheTileReaderBody10_1::Internet {
                version,
                strings,
                flag,
                longs,
                config_name,
                config,
                provider: provider.entry,
                provider_body: provider.body,
            })
        }
        MAP_SERVER_REST_TILE_READER | MAP_SERVER_TILE_READER => {
            let rest = class == MAP_SERVER_REST_TILE_READER;
            let version = if rest {
                version(r, 1, "MapServerRESTTileReader")?
            } else {
                version(r, 2, "MapServerTileReader")?
            };
            let flag = r.read_bytes(1)?[0];
            let color = read_tile_color(r, max, refs)?;
            let config_name = read_string_10_1(r, max)?;
            let config = read_config(r, max, refs)?;
            let server_name = ags_name::read_server_object_name_slot(r, max, refs)?;
            let rest_selector = (rest || version >= 2)
                .then(|| r.read_u16_le().map(|value| value as i16))
                .transpose()?;
            let inline_rest = rest_selector
                .filter(|&selector| selector != 0)
                .map(|_| read_map_server_rest(r, max, refs, depth).map(Box::new))
                .transpose()?;
            Ok(CacheTileReaderBody10_1::MapServer {
                rest,
                version,
                flag,
                color,
                config_name,
                config,
                server_name: server_name.entry,
                server_name_body: server_name.body,
                rest_selector,
                inline_rest,
            })
        }
        MAP_TILE_READER => {
            let version = version(r, 2, "MapTileReader")?;
            Ok(CacheTileReaderBody10_1::MapTile {
                version,
                strings: [read_string_10_1(r, max)?, read_string_10_1(r, max)?],
                shorts: [r.read_u16_le()? as i16, r.read_u16_le()? as i16],
                path: read_string_10_1(r, max)?,
                name: Box::new(read_name_slot(r, max, refs, depth)?),
                selector: r.read_u16_le()? as i16,
                config: (version >= 2)
                    .then(|| read_config(r, max, refs))
                    .transpose()?,
            })
        }
        _ => Err(unsupported_class("ITileReader", &class)),
    }
}

fn read_map_server_rest(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<MapServerRestBody10_1> {
    let version = version(r, 5, "MapServerREST")?;
    let flag = r.read_u16_le()? as i16;
    let first = read_string_10_1(r, max)?;
    let second = read_string_10_1(r, max)?;
    let first_json = read_rest_json(r, max)?;
    let third = read_string_10_1(r, max)?;
    let second_json = read_rest_json(r, max)?;
    let fourth = read_string_10_1(r, max)?;
    let third_json = read_rest_json(r, max)?;
    let extra_string = (version >= 2)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let extra_json = (version >= 3)
        .then(|| {
            Ok::<_, io::Error>([
                read_rest_json(r, max)?,
                read_rest_json(r, max)?,
                read_rest_json(r, max)?,
            ])
        })
        .transpose()?;
    let name = (version >= 4)
        .then(|| read_name_slot(r, max, refs, depth))
        .transpose()?;
    let properties = (version >= 5)
        .then(|| read_properties_slot(r, max, refs))
        .transpose()?;
    Ok(MapServerRestBody10_1 {
        version,
        flag,
        strings: [first, second, third, fourth],
        json: [first_json, second_json, third_json],
        extra_string,
        extra_json,
        name,
        properties,
    })
}

fn read_properties_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<PropertySetObjectStream10_1>> {
    slot(r, refs, |r, refs, class, _| {
        if class != PROPERTY_SET_CLASS_ID {
            return Err(unsupported_class("IPropertySet", &class));
        }
        read_property_set_object_stream_10_1(r, max, refs)
    })
}

/// MapLib JSONSerializerCarto 保存字节数及正文。正文保留原字节。
fn read_rest_json(r: &mut Reader<'_>, max: usize) -> io::Result<Vec<u8>> {
    let length = r.read_u32_le()? as usize;
    if length > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "MapServerREST JSON 长度超过限额",
        ));
    }
    Ok(r.read_bytes(length)?.to_vec())
}

fn read_config(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<CacheTileConfiguration10_1> {
    Ok(CacheTileConfiguration10_1 {
        extent: read_envelope_slot(r, max, refs)?,
        cache: read_tile_cache_slot(r, max, refs)?,
        image: read_tile_image_slot(r, max, refs)?,
    })
}

fn read_tile_cache_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<TileCacheInfo10_1>> {
    slot(r, refs, |r, refs, class, _| {
        if class != TILE_CACHE_INFO {
            return Err(unsupported_class("ITileCacheInfo", &class));
        }
        read_cache_info(r, max, refs)
    })
}

fn read_tile_image_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<TileImageInfo10_1>> {
    slot(r, refs, |r, _, class, _| {
        if class != TILE_IMAGE_INFO {
            return Err(unsupported_class("ITileImageInfo", &class));
        }
        Ok(TileImageInfo10_1 {
            format: read_string_10_1(r, max)?,
            value: r.read_u32_le()? as i32,
        })
    })
}

fn read_tile_color(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<ColorSlot10_1> {
    match read_color_slot_10_1(r, max, refs)? {
        ColorSlot10_1::Unsupported { class_id, .. } => Err(unsupported_class("IColor", &class_id)),
        color => Ok(color),
    }
}

fn read_cache_info(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TileCacheInfo10_1> {
    let version = version(r, 3, "TileCacheInfo")?;
    let origin = slot(r, refs, |r, refs, class, _| {
        if class != POINT_CLASS_ID {
            return Err(unsupported_class("TileCacheInfo IPoint", &class));
        }
        read_point_body_10_1(r, max, refs)
    })?;
    let dimensions = [r.read_u32_le()? as i32, r.read_u32_le()? as i32];
    let levels = slot(r, refs, |r, refs, class, _| {
        if class != LOD_INFOS {
            return Err(unsupported_class("ILODInfos", &class));
        }
        read_lod_infos(r, refs)
    })?;
    let spatial_reference = (version >= 2)
        .then(|| read_spatial_reference_entry_10_1(r, max, refs))
        .transpose()?;
    let dpi = match version {
        2 => Some(f64::from(r.read_u32_le()? as i32)),
        3 => Some(r.read_f64_le()?),
        _ => None,
    };
    Ok(TileCacheInfo10_1 {
        version,
        origin,
        dimensions,
        levels,
        spatial_reference,
        dpi,
    })
}

fn read_lod_infos(r: &mut Reader<'_>, refs: &mut RefTable10_1) -> io::Result<TileLodInfos10_1> {
    let list_version = version(r, 1, "LODInfos")?;
    let count = r.read_u32_le()? as i32;
    let slots = count.max(0) as usize;
    if slots > r.remaining() / 16 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "LODInfos 项数超出剩余对象头范围",
        ));
    }
    let mut items = Vec::new();
    for _ in 0..slots {
        items.push(slot(r, refs, |r, _, class, _| {
            if class != LOD_INFO {
                return Err(unsupported_class("ILODInfo", &class));
            }
            Ok(TileLodInfo10_1 {
                version: version(r, 1, "LODInfo")?,
                level: r.read_u32_le()? as i32,
                doubles: [r.read_f64_le()?, r.read_f64_le()?],
            })
        })?);
    }
    Ok(TileLodInfos10_1 {
        version: list_version,
        count,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];

    #[test]
    fn reader_versions_end_before_the_next_field() {
        for (class, version) in [
            (INTERNET_TILE_READER, 1_i16),
            (MAP_SERVER_REST_TILE_READER, 1),
            (MAP_SERVER_TILE_READER, 1),
            (MAP_SERVER_TILE_READER, 2),
            (MAP_TILE_READER, 1),
            (MAP_TILE_READER, 2),
        ] {
            let mut bytes = version.to_le_bytes().to_vec();
            match class {
                INTERNET_TILE_READER => {
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend([1; 13]);
                    bytes.extend(EMPTY_STRING);
                    bytes.extend([0; 16 * 4]);
                }
                MAP_TILE_READER => {
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend([1; 4]);
                    bytes.extend(EMPTY_STRING);
                    bytes.extend([0; 16]);
                    bytes.extend(1_i16.to_le_bytes());
                    if version == 2 {
                        bytes.extend([0; 16 * 3]);
                    }
                }
                _ => {
                    bytes.push(1);
                    bytes.extend([0; 16]);
                    bytes.extend(EMPTY_STRING);
                    bytes.extend([0; 16 * 4]);
                    if class == MAP_SERVER_REST_TILE_READER || version == 2 {
                        bytes.extend(0_i16.to_le_bytes());
                    }
                }
            }
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(
                    read_tile_reader(&mut reader, 4096, &mut RefTable10_1::new(), class, 16)
                        .is_err()
                );
            }
            bytes.extend(0x5a3e_u16.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            read_tile_reader(&mut reader, 4096, &mut RefTable10_1::new(), class, 16).unwrap();
            assert_eq!(reader.read_u16_le().unwrap(), 0x5a3e);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn rest_inline_payload_consumes_versioned_json_and_slots() {
        for inline_version in 1_i16..=5 {
            let mut bytes = vec![1, 0, 0];
            bytes.extend([0; 16]);
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16 * 4]);
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(inline_version.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(EMPTY_STRING);
            for json in [b"{}".as_slice(), b"[]", b""] {
                bytes.extend(EMPTY_STRING);
                bytes.extend((json.len() as u32).to_le_bytes());
                bytes.extend(json);
            }
            if inline_version >= 2 {
                bytes.extend(EMPTY_STRING);
            }
            if inline_version >= 3 {
                bytes.extend([0; 4 * 3]);
            }
            if inline_version >= 4 {
                bytes.extend([0; 16]);
            }
            if inline_version == 5 {
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(90_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
            }
            let body_end = bytes.len();
            bytes.extend(0x5a3e_u16.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let CacheTileReaderBody10_1::MapServer {
                inline_rest: Some(rest),
                ..
            } = read_tile_reader(
                &mut reader,
                4096,
                &mut refs,
                MAP_SERVER_REST_TILE_READER,
                16,
            )
            .unwrap()
            else {
                panic!("应读取内联 MapServerREST");
            };
            assert_eq!(rest.json, [b"{}".to_vec(), b"[]".to_vec(), Vec::new()]);
            assert_eq!(rest.extra_json.is_some(), inline_version >= 3);
            assert_eq!(rest.name.is_some(), inline_version >= 4);
            assert_eq!(rest.properties.is_some(), inline_version == 5);
            assert_eq!(refs.contains(90), inline_version == 5);
            assert_eq!(reader.position(), body_end);
            assert_eq!(reader.read_u16_le().unwrap(), 0x5a3e);
            let mut truncated = Reader::new(&bytes[..body_end - 1]);
            assert!(
                read_tile_reader(
                    &mut truncated,
                    4096,
                    &mut RefTable10_1::new(),
                    MAP_SERVER_REST_TILE_READER,
                    16
                )
                .is_err()
            );
        }
    }

    #[test]
    fn config_reads_tile_cache_levels_and_image_info() {
        for cache_version in 1_i16..=3 {
            let mut bytes = vec![0; 16];
            let add_header = |bytes: &mut Vec<u8>, class: [u8; 16], key: u32| {
                bytes.extend(class);
                bytes.extend(key.to_le_bytes());
            };
            add_header(&mut bytes, TILE_CACHE_INFO, 8);
            bytes.extend(cache_version.to_le_bytes());
            bytes.extend([0; 16]); // 原点
            bytes.extend(512_i32.to_le_bytes());
            bytes.extend(256_i32.to_le_bytes());
            add_header(&mut bytes, LOD_INFOS, 9);
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            add_header(&mut bytes, LOD_INFO, 10);
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(4_i32.to_le_bytes());
            bytes.extend(150.0_f64.to_le_bytes());
            bytes.extend(2.5_f64.to_le_bytes());
            add_header(&mut bytes, LOD_INFO, 10);
            if cache_version >= 2 {
                bytes.extend([0; 16]); // 空空间参考
            }
            match cache_version {
                2 => bytes.extend(96_i32.to_le_bytes()),
                3 => bytes.extend(96.5_f64.to_le_bytes()),
                _ => {}
            }
            add_header(&mut bytes, TILE_IMAGE_INFO, 11);
            bytes.extend([8, 0, 0, 0, b'P', 0, b'N', 0, b'G', 0, 0, 0]);
            bytes.extend(75_i32.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_config(&mut reader, 4096, &mut refs).unwrap();
            let cache = result.cache.body.unwrap();
            let levels = cache.levels.body.unwrap();
            assert_eq!(cache.dimensions, [512, 256]);
            assert_eq!(
                cache.dpi,
                [None, Some(96.0), Some(96.5)][cache_version as usize - 1]
            );
            assert_eq!(levels.items[0].body.as_ref().unwrap().doubles, [150.0, 2.5]);
            assert!(matches!(
                levels.items[1].entry,
                ObjectEntry10_1::BackReference { key: 10, .. }
            ));
            let image = result.image.body.unwrap();
            assert_eq!(image.format, "PNG");
            assert_eq!(image.value, 75);
            assert_eq!(refs.len(), 4);
            assert_eq!(reader.remaining(), 0);
        }
    }
}
