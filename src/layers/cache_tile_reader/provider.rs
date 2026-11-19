//! MapServerLayer.dll 10.1 的 Internet 瓦片提供器。

use super::{
    ColorSlot10_1, RasterLayerEnvelopeSlot10_1, Reader, RefTable10_1, RuleSlot10_1,
    SpatialReferenceEntry10_1, TileCacheInfo10_1, TileImageInfo10_1, read_envelope_slot,
    read_spatial_reference_entry_10_1, read_string_10_1, read_tile_cache_slot, read_tile_color,
    read_tile_image_slot, slot, unsupported, unsupported_class, version,
};
use crate::layers::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use std::io;

const ARC_GIS_ONLINE: [u8; 16] = [
    0xc8, 0xd6, 0x95, 0xd7, 0x80, 0xab, 0xe2, 0x48, 0xb5, 0xa2, 0xc1, 0xf3, 0xd9, 0x6c, 0x59, 0x57,
];
const GOOGLE_MAPS: [u8; 16] = [
    0x2a, 0x77, 0x89, 0x32, 0xed, 0xfa, 0x58, 0x4a, 0xa7, 0xbb, 0x37, 0x1f, 0x0b, 0x52, 0xde, 0x99,
];
const IMS: [u8; 16] = [
    0xdd, 0x7d, 0x8e, 0x31, 0x30, 0xec, 0x1c, 0x42, 0xac, 0xa9, 0x24, 0x4e, 0x77, 0xab, 0x4c, 0x89,
];
const VIRTUAL_EARTH: [u8; 16] = [
    0x83, 0xac, 0xe3, 0x1c, 0xe0, 0x26, 0xcd, 0x42, 0xaf, 0x4d, 0x21, 0x3b, 0x72, 0xce, 0xb8, 0x64,
];
const OPEN_STREET_MAP: [u8; 16] = [
    0x0c, 0xeb, 0x1e, 0x35, 0xb9, 0xbc, 0x64, 0x40, 0x9e, 0x61, 0x12, 0x19, 0x16, 0x9a, 0xf6, 0x33,
];
const WMS: [u8; 16] = [
    0x45, 0x28, 0x9f, 0x82, 0xc1, 0xc1, 0x5d, 0x48, 0x91, 0xfe, 0xd9, 0x46, 0x29, 0xff, 0x54, 0xff,
];
const WMTS: [u8; 16] = [
    0x19, 0x70, 0x3b, 0xf9, 0x14, 0x9a, 0xa7, 0x4d, 0x9a, 0x84, 0x62, 0x8c, 0x40, 0x84, 0x12, 0xb9,
];
pub const WMTS_LAYER_CLASS_ID: [u8; 16] = [
    0xa1, 0x43, 0xc7, 0x61, 0x17, 0x83, 0x6a, 0x41, 0x83, 0x17, 0x10, 0x96, 0x4d, 0xad, 0xc6, 0xad,
];
pub const INTERNET_TILED_LAYER_CLASS_ID: [u8; 16] = [
    0x1b, 0xdc, 0xba, 0xa4, 0xed, 0xeb, 0x29, 0x4a, 0x99, 0xdc, 0xc6, 0x33, 0x4d, 0xe3, 0x52, 0xad,
];
const CACHE_INFO: [u8; 16] = [
    0x5e, 0x34, 0x2e, 0x26, 0xcb, 0xb6, 0xcd, 0x41, 0x9f, 0x88, 0x54, 0x53, 0x74, 0x1e, 0x63, 0xf4,
];

#[derive(Debug, Clone, PartialEq)]
pub enum InternetTileProvider10_1 {
    ArcGisOnline {
        version: i16,
        strings: [String; 2],
        cache: RuleSlot10_1<CacheInfo10_1>,
    },
    GoogleMaps(WebTileProvider10_1),
    OpenStreetMap(WebTileProvider10_1),
    VirtualEarth(WebTileProvider10_1),
    Ims(Box<ImsProvider10_1>),
    Wms(Box<WmsProvider10_1>),
    Wmts(Box<WmtsProvider10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CacheInfo10_1 {
    pub tiles: RuleSlot10_1<TileCacheInfo10_1>,
    pub image: RuleSlot10_1<TileImageInfo10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebTileProvider10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub short: i16,
    pub cache_path: String,
    pub cache: RuleSlot10_1<CacheInfo10_1>,
    pub extra_strings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImsProvider10_1 {
    pub version: i16,
    pub cache: CacheInfo10_1,
    pub strings: [String; 5],
    pub extents: [RasterLayerEnvelopeSlot10_1; 2],
    pub color: ColorSlot10_1,
    pub string_80: String,
    pub short_84: i16,
    pub strings_88_78_7c: [String; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct WmsProvider10_1 {
    pub version: i16,
    pub cache: CacheInfo10_1,
    pub strings: [String; 4],
    pub extent: RasterLayerEnvelopeSlot10_1,
    pub color: ColorSlot10_1,
    pub strings_80_84: [String; 2],
    pub short_94: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WmtsProvider10_1 {
    pub version: i16,
    pub short_24: i16,
    pub cache: RuleSlot10_1<CacheInfo10_1>,
    pub strings: [String; 8],
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub extent: RasterLayerEnvelopeSlot10_1,
    pub pairs: Vec<[String; 2]>,
    pub extents: Vec<[f64; 4]>,
    pub names: Vec<String>,
    pub string_48: String,
    pub short_9c: i16,
    pub layer: RuleSlot10_1<WmtsLayer10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WmtsLayer10_1 {
    pub version: i16,
    pub base: InternetTiledLayer10_1,
    pub provider: RuleSlot10_1<InternetTileProvider10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternetTiledLayer10_1 {
    pub version: i16,
    pub strings: [String; 2],
    pub shorts_5c_60_62: [i16; 3],
    pub doubles_6c_74: [f64; 2],
    pub shorts_8e_92_90_94: [i16; 4],
    pub color: ColorSlot10_1,
    pub short_8c: i16,
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub long_e0: i32,
    pub extent: RasterLayerEnvelopeSlot10_1,
    pub double_84: f64,
    pub provider: RuleSlot10_1<InternetTileProvider10_1>,
    pub long_e4: i32,
    pub doubles_fc_104: [f64; 2],
    pub v2_fields: Option<InternetTiledLayerV2Fields10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TiledLayer10_1 {
    Internet(InternetTiledLayer10_1),
    Wmts(WmtsLayer10_1),
}

impl TiledLayer10_1 {
    pub fn name(&self) -> &str {
        // ILayer::get_Name 读取完整对象 +0x94，与基体 Load 的第一串对位。
        match self {
            Self::Internet(body) => &body.strings[0],
            Self::Wmts(body) => &body.base.strings[0],
        }
    }
}

pub(crate) fn is_tiled_layer(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        INTERNET_TILED_LAYER_CLASS_ID | WMTS_LAYER_CLASS_ID
    )
}

pub(crate) fn read_tiled_layer_10_1(
    class_id: [u8; 16],
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<TiledLayer10_1> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let depth = 16;
    let result = match class_id {
        INTERNET_TILED_LAYER_CLASS_ID => TiledLayer10_1::Internet(read_internet_tiled_layer(
            &mut trial, max_bytes, &mut refs, depth,
        )?),
        WMTS_LAYER_CLASS_ID => {
            TiledLayer10_1::Wmts(read_wmts_layer(&mut trial, max_bytes, &mut refs, depth)?)
        }
        _ => return Err(unsupported_class("瓦片图层", &class_id)),
    };
    *reader = trial;
    *table = refs;
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternetTiledLayerV2Fields10_1 {
    pub strings: [String; 2],
    pub short_f0: i16,
    pub long_10c: i32,
    pub flag_111: u8,
}

fn read_strings<const N: usize>(r: &mut Reader<'_>, max: usize) -> io::Result<[String; N]> {
    let mut result = std::array::from_fn(|_| String::new());
    for value in &mut result {
        *value = read_string_10_1(r, max)?;
    }
    Ok(result)
}

fn read_cache_info(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<CacheInfo10_1> {
    Ok(CacheInfo10_1 {
        tiles: read_tile_cache_slot(r, max, refs)?,
        image: read_tile_image_slot(r, max, refs)?,
    })
}

fn read_cache_info_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<CacheInfo10_1>> {
    slot(r, refs, |r, refs, class, _| {
        if class != CACHE_INFO {
            return Err(unsupported_class("ICacheInfo", &class));
        }
        read_cache_info(r, max, refs)
    })
}

fn exact_v1(r: &mut Reader<'_>, name: &str) -> io::Result<i16> {
    let v = version(r, 1, name)?;
    if v != 1 {
        return Err(unsupported(format!("{name} 未支持版本 {v}")));
    }
    Ok(v)
}

pub(super) fn read_provider(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class: [u8; 16],
    depth: usize,
) -> io::Result<InternetTileProvider10_1> {
    if depth == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "瓦片提供器嵌套超过限额",
        ));
    }
    use InternetTileProvider10_1::*;
    Ok(match class {
        ARC_GIS_ONLINE => ArcGisOnline {
            version: exact_v1(r, "ArcGISOnlineProvider")?,
            strings: read_strings(r, max)?,
            cache: read_cache_info_slot(r, max, refs)?,
        },
        GOOGLE_MAPS | OPEN_STREET_MAP | VIRTUAL_EARTH => {
            let version = match class {
                GOOGLE_MAPS => version(r, 1, "GoogleMapsLayerProvider")?,
                OPEN_STREET_MAP => {
                    let v = version(r, 2, "OpenStreetMapProvider")?;
                    if v < 1 {
                        return Err(unsupported(format!("OpenStreetMapProvider 未支持版本 {v}")));
                    }
                    v
                }
                _ => version(r, 4, "MSVirtualEarthLayerProvider")?,
            };
            let strings = read_strings(r, max)?;
            let short = r.read_u16_le()? as i16;
            let cache_path = read_string_10_1(r, max)?;
            let cache = read_cache_info_slot(r, max, refs)?;
            let count = match class {
                OPEN_STREET_MAP if version == 2 => 1,
                VIRTUAL_EARTH => version.saturating_sub(1).max(0),
                _ => 0,
            };
            let extra_strings = (0..count)
                .map(|_| read_string_10_1(r, max))
                .collect::<io::Result<_>>()?;
            let body = WebTileProvider10_1 {
                version,
                strings,
                short,
                cache_path,
                cache,
                extra_strings,
            };
            match class {
                GOOGLE_MAPS => GoogleMaps(body),
                OPEN_STREET_MAP => OpenStreetMap(body),
                _ => VirtualEarth(body),
            }
        }
        IMS => Ims(Box::new(ImsProvider10_1 {
            version: version(r, 2, "IMSLayerProvider")?,
            cache: read_cache_info(r, max, refs)?,
            strings: read_strings(r, max)?,
            extents: [
                read_envelope_slot(r, max, refs)?,
                read_envelope_slot(r, max, refs)?,
            ],
            color: read_tile_color(r, max, refs)?,
            string_80: read_string_10_1(r, max)?,
            short_84: r.read_u16_le()? as i16,
            strings_88_78_7c: read_strings(r, max)?,
        })),
        WMS => Wms(Box::new(WmsProvider10_1 {
            version: exact_v1(r, "WMSLayerProvider")?,
            cache: read_cache_info(r, max, refs)?,
            strings: read_strings(r, max)?,
            extent: read_envelope_slot(r, max, refs)?,
            color: read_tile_color(r, max, refs)?,
            strings_80_84: read_strings(r, max)?,
            short_94: r.read_u16_le()? as i16,
        })),
        WMTS => Wmts(Box::new(read_wmts(r, max, refs, depth - 1)?)),
        _ => return Err(unsupported_class("InternetTileReader 提供器", &class)),
    })
}

fn read_items<T>(
    r: &mut Reader<'_>,
    item_min: usize,
    mut read: impl FnMut(&mut Reader<'_>) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let count = r.read_u32_le()? as usize;
    if count > r.remaining() / item_min {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "WMTS 集合计数超出剩余字节范围",
        ));
    }
    (0..count).map(|_| read(r)).collect()
}

fn read_wmts(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<WmtsProvider10_1> {
    Ok(WmtsProvider10_1 {
        version: version(r, 1, "WMTSLayerProvider")?,
        short_24: r.read_u16_le()? as i16,
        cache: read_cache_info_slot(r, max, refs)?,
        strings: read_strings(r, max)?,
        spatial_reference: read_spatial_reference_entry_10_1(r, max, refs)?,
        extent: read_envelope_slot(r, max, refs)?,
        pairs: read_items(r, 12, |r| read_strings(r, max))?,
        extents: read_items(r, 32, |r| {
            Ok([
                r.read_f64_le()?,
                r.read_f64_le()?,
                r.read_f64_le()?,
                r.read_f64_le()?,
            ])
        })?,
        names: read_items(r, 6, |r| read_string_10_1(r, max))?,
        string_48: read_string_10_1(r, max)?,
        short_9c: r.read_u16_le()? as i16,
        layer: slot(r, refs, |r, refs, class, _| {
            if class != WMTS_LAYER_CLASS_ID {
                return Err(unsupported_class("IWMTSLayer", &class));
            }
            read_wmts_layer(r, max, refs, depth)
        })?,
    })
}

fn read_wmts_layer(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<WmtsLayer10_1> {
    let version = version(r, 1, "WMTSLayer")?;
    let base = read_internet_tiled_layer(r, max, refs, depth)?;
    let provider = slot(r, refs, |r, refs, class, _| {
        read_provider(r, max, refs, class, depth)
    })?;
    Ok(WmtsLayer10_1 {
        version,
        base,
        provider,
    })
}

fn read_internet_tiled_layer(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<InternetTiledLayer10_1> {
    let version = version(r, 2, "InternetTiledLayer")?;
    Ok(InternetTiledLayer10_1 {
        version,
        strings: read_strings(r, max)?,
        shorts_5c_60_62: [
            r.read_u16_le()? as i16,
            r.read_u16_le()? as i16,
            r.read_u16_le()? as i16,
        ],
        doubles_6c_74: [r.read_f64_le()?, r.read_f64_le()?],
        shorts_8e_92_90_94: [
            r.read_u16_le()? as i16,
            r.read_u16_le()? as i16,
            r.read_u16_le()? as i16,
            r.read_u16_le()? as i16,
        ],
        color: read_tile_color(r, max, refs)?,
        short_8c: r.read_u16_le()? as i16,
        extensions: read_layer_extensions_10_1(r, max, refs)?,
        long_e0: r.read_u32_le()? as i32,
        extent: read_envelope_slot(r, max, refs)?,
        double_84: r.read_f64_le()?,
        provider: slot(r, refs, |r, refs, class, _| {
            read_provider(r, max, refs, class, depth)
        })?,
        long_e4: r.read_u32_le()? as i32,
        doubles_fc_104: [r.read_f64_le()?, r.read_f64_le()?],
        v2_fields: (version >= 2)
            .then(|| {
                Ok::<_, io::Error>(InternetTiledLayerV2Fields10_1 {
                    strings: read_strings(r, max)?,
                    short_f0: r.read_u16_le()? as i16,
                    long_10c: r.read_u32_le()? as i32,
                    flag_111: r.read_bytes(1)?[0],
                })
            })
            .transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::cache_tile_reader::{
        CacheTileReaderBody10_1, INTERNET_TILE_READER, TILE_IMAGE_INFO, read_tile_reader,
    };
    use crate::storage::object_refs::ObjectEntry10_1;

    const EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];

    #[test]
    fn tiled_layer_bodies_are_shared_by_map_and_scene_hosts() {
        use crate::layers::graphics_3d::GRAPHICS_LAYER_3D_CLASS_ID;
        use crate::layers::group_layer::{GroupChildEntry10_1, read_group_layer_prefix_10_1};
        use crate::layers::layer_list::{LayerBodyPrefix10_1, read_layer_list_10_1};
        use crate::maps::layer_masking::{
            LayerMaskingSupport10_1, known_layer_masking_support_10_1,
        };

        let mut base = 2_i16.to_le_bytes().to_vec();
        base.extend(4_u32.to_le_bytes());
        base.extend("图\0".encode_utf16().flat_map(u16::to_le_bytes));
        base.extend(EMPTY_STRING);
        base.extend([0; 6 + 16 + 8 + 16 + 2 + 4 + 4 + 16 + 8]);
        base.extend(GOOGLE_MAPS);
        base.extend(73_u32.to_le_bytes());
        base.extend(1_i16.to_le_bytes());
        base.extend(EMPTY_STRING.repeat(3));
        base.extend([0; 2]);
        base.extend(EMPTY_STRING);
        base.extend([0; 16]);
        base.extend([0; 4 + 16]);
        base.extend(EMPTY_STRING.repeat(2));
        base.extend([0; 2 + 4]);
        base.push(1);

        let mut wmts = 1_i16.to_le_bytes().to_vec();
        wmts.extend(&base);
        wmts.extend([0; 16]);

        let mut scene = 1_i16.to_le_bytes().to_vec();
        scene.extend(EMPTY_STRING);
        scene.extend([0; 8]);
        scene.extend(WMTS_LAYER_CLASS_ID);
        scene.extend(74_u32.to_le_bytes());
        scene.extend(&wmts);
        scene.extend([0; 16 + 4]);

        let wrap = |flag: u16, class_id: [u8; 16], body: &[u8]| {
            let mut bytes = flag.to_le_bytes().to_vec();
            if flag == 2 {
                bytes.extend((20 + body.len() as u32).to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
            }
            bytes.extend(class_id);
            bytes.extend(71_u32.to_le_bytes());
            bytes.extend(body);
            bytes
        };

        for (flag, class_id, body) in [
            (1, INTERNET_TILED_LAYER_CLASS_ID, &base),
            (2, WMTS_LAYER_CLASS_ID, &wmts),
            (1, GRAPHICS_LAYER_3D_CLASS_ID, &scene),
        ] {
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend(wrap(flag, class_id, body));
            bytes.extend(0_i32.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(list.active_index, Some(0));
            assert_eq!(reader.remaining(), 0);
            assert!(refs.contains(73));
            let body = list.entries[0].parsed.as_ref().unwrap();
            let tiled = match body {
                LayerBodyPrefix10_1::TiledLayer(tiled) => tiled,
                LayerBodyPrefix10_1::Graphics3d(scene) => {
                    assert!(refs.contains(74));
                    let Some(LayerBodyPrefix10_1::TiledLayer(tiled)) =
                        scene.associated_layer_body.as_deref()
                    else {
                        panic!("应读出 Scene 关联的瓦片图层");
                    };
                    tiled
                }
                _ => panic!("应读出瓦片图层"),
            };
            assert_eq!(tiled.name(), "图");
        }

        let mut bytes = 2_i32.to_le_bytes().to_vec();
        bytes.extend(wrap(
            2,
            INTERNET_TILED_LAYER_CLASS_ID,
            &base[..base.len() - 1],
        ));
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(1_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(list.entries[0].parsed.is_none());
        assert_eq!(list.active_index, Some(1));
        assert_eq!(reader.remaining(), 0);
        assert!(!refs.contains(73));

        let mut group = 11_u16.to_le_bytes().to_vec();
        group.extend(EMPTY_STRING);
        group.extend([0; 6 + 16]);
        group.extend(1_u32.to_le_bytes());
        group.extend(20_u32.to_le_bytes());
        group.extend((20 + wmts.len() as u32).to_le_bytes());
        group.extend(WMTS_LAYER_CLASS_ID);
        group.extend(75_u32.to_le_bytes());
        group.extend(&wmts);
        group.extend([0; 2 + 16 + 8]);
        let mut reader = Reader::new(&group);
        let mut refs = RefTable10_1::new();
        let group = read_group_layer_prefix_10_1(&mut reader, 4096, &mut refs).unwrap();
        match &group.children[0] {
            GroupChildEntry10_1::SkippedNewObject {
                parsed: Some(parsed),
                body_length: 0,
                ..
            } => {
                assert_eq!(parsed.layer_name(), "图");
            }
            other => panic!("应按 B 边界读出子图层：{other:?}"),
        }
        assert!(refs.contains(73));
        assert_eq!(reader.remaining(), 0);
        for id in [INTERNET_TILED_LAYER_CLASS_ID, WMTS_LAYER_CLASS_ID] {
            assert_eq!(
                known_layer_masking_support_10_1(&id),
                LayerMaskingSupport10_1::Unsupported
            );
        }
    }

    #[test]
    fn provider_bodies_end_before_the_next_field() {
        for (class, version) in [
            (ARC_GIS_ONLINE, 1_i16),
            (GOOGLE_MAPS, 1),
            (OPEN_STREET_MAP, 1),
            (OPEN_STREET_MAP, 2),
            (VIRTUAL_EARTH, i16::MIN),
            (VIRTUAL_EARTH, 4),
            (IMS, 2),
            (WMS, 1),
            (WMTS, 1),
        ] {
            let mut bytes = vec![1, 0];
            bytes.extend(EMPTY_STRING.repeat(2));
            bytes.extend([0; 13]);
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 48]);
            bytes.extend(class);
            bytes.extend(70_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            match class {
                ARC_GIS_ONLINE => {
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend(CACHE_INFO);
                    bytes.extend(71_u32.to_le_bytes());
                    bytes.extend([0; 16]);
                    bytes.extend(TILE_IMAGE_INFO);
                    bytes.extend(72_u32.to_le_bytes());
                    bytes.extend(EMPTY_STRING);
                    bytes.extend(75_i32.to_le_bytes());
                }
                GOOGLE_MAPS | OPEN_STREET_MAP | VIRTUAL_EARTH => {
                    bytes.extend(EMPTY_STRING.repeat(3));
                    bytes.extend([5, 0]);
                    bytes.extend(EMPTY_STRING);
                    bytes.extend([0; 16]);
                    if class == OPEN_STREET_MAP && version == 2 {
                        bytes.extend(EMPTY_STRING);
                    }
                    if class == VIRTUAL_EARTH && version == 4 {
                        bytes.extend(EMPTY_STRING.repeat(3));
                    }
                }
                IMS => {
                    bytes.extend([0; 32]);
                    bytes.extend(EMPTY_STRING.repeat(5));
                    bytes.extend([0; 48]);
                    bytes.extend(EMPTY_STRING);
                    bytes.extend([0; 2]);
                    bytes.extend(EMPTY_STRING.repeat(3));
                }
                WMS => {
                    bytes.extend([0; 32]);
                    bytes.extend(EMPTY_STRING.repeat(4));
                    bytes.extend([0; 32]);
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend([0; 2]);
                }
                WMTS => {
                    bytes.extend([0; 2 + 16]);
                    bytes.extend(EMPTY_STRING.repeat(8));
                    bytes.extend([0; 32]);
                    bytes.extend(1_u32.to_le_bytes());
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend(1_u32.to_le_bytes());
                    for value in [1., 2., 3., 4.] {
                        bytes.extend(f64::to_le_bytes(value));
                    }
                    bytes.extend(1_u32.to_le_bytes());
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend([0; 2]);
                    bytes.extend(WMTS_LAYER_CLASS_ID);
                    bytes.extend(71_u32.to_le_bytes());
                    bytes.extend(1_u16.to_le_bytes());
                    bytes.extend(2_u16.to_le_bytes());
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend([0; 6 + 16 + 8 + 16 + 2 + 4 + 4 + 16 + 8]);
                    bytes.extend(WMTS);
                    bytes.extend(70_u32.to_le_bytes());
                    bytes.extend([0; 4 + 16]);
                    bytes.extend(EMPTY_STRING.repeat(2));
                    bytes.extend([0; 2 + 4]);
                    bytes.push(1);
                    bytes.extend([0; 16]);
                }
                _ => unreachable!(),
            }
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(
                    read_tile_reader(
                        &mut reader,
                        4096,
                        &mut RefTable10_1::new(),
                        INTERNET_TILE_READER,
                        16
                    )
                    .is_err(),
                    "错误接受截断的提供器 {class:?} v{version}，长度 {end}"
                );
            }
            bytes.extend(0x5a3e_u16.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let CacheTileReaderBody10_1::Internet {
                provider_body: Some(decoded),
                ..
            } = read_tile_reader(&mut reader, 4096, &mut refs, INTERNET_TILE_READER, 16).unwrap()
            else {
                panic!("应读取首次定义的 InternetTileReader 提供器");
            };
            assert!(refs.contains(70));
            assert_eq!(reader.read_u16_le().unwrap(), 0x5a3e);
            match *decoded {
                InternetTileProvider10_1::ArcGisOnline { cache, .. } => {
                    assert_eq!(cache.body.unwrap().image.body.unwrap().value, 75);
                    assert!(refs.contains(71) && refs.contains(72));
                }
                InternetTileProvider10_1::Wmts(body) => {
                    assert_eq!(body.pairs.len(), 1);
                    assert_eq!(body.extents, [[1., 2., 3., 4.]]);
                    assert_eq!(body.names.len(), 1);
                    let layer = body.layer.body.unwrap();
                    assert_eq!(layer.base.v2_fields.unwrap().flag_111, 1);
                    assert!(matches!(
                        layer.base.provider.entry,
                        ObjectEntry10_1::BackReference { key: 70, .. }
                    ));
                    assert!(refs.contains(71));
                }
                _ => {}
            }
        }
    }
}
