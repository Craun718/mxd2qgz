//! PageLayout.dll 10.1.0.3035 的元素列表后字段（Load 0x7b6e4e30）。
use super::map_decor::read_map_decor_slot_10_1;
use super::map_object::{
    MAP_CLASS_ID, MapPrefix10_1, MapSuffix10_1, MapSuffixStop, read_map_with_suffix_10_1,
};
use crate::layers::cache_tile_reader::{
    AGS_SERVER_CONNECTION_NAME, AgsServerConnectionName10_1, read_ags_server_connection_name_body,
};
use crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
use crate::layers::layer_list::read_complete_layer_body_10_1;
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::element_decor::{DecorKind10_1, DecorSlot10_1};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

const PAGE: [u8; 16] = [
    0x6f, 0xd7, 0x94, 0xdd, 0x6d, 0x83, 0xd0, 0x11, 0x87, 0xec, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];
const RULER: [u8; 16] = [
    0xac, 0x81, 0xe0, 0x31, 0x02, 0xcb, 0xd1, 0x11, 0x87, 0x6c, 0, 0, 0xf8, 0x75, 0x17, 0x20,
];
const GRID: [u8; 16] = [
    0xad, 0x81, 0xe0, 0x31, 0x02, 0xcb, 0xd1, 0x11, 0x87, 0x6c, 0, 0, 0xf8, 0x75, 0x17, 0x20,
];
const GUIDES: [u8; 16] = [
    0xae, 0x81, 0xe0, 0x31, 0x02, 0xcb, 0xd1, 0x11, 0x87, 0x6c, 0, 0, 0xf8, 0x75, 0x17, 0x20,
];
const OPTIMIZATION_EXTENSION: [u8; 16] = [
    0x9f, 0xd8, 0x29, 0x6c, 0xba, 0xb7, 0xb0, 0x4f, 0xa8, 0x8d, 0xfe, 0xa7, 0x7a, 0x03, 0xba, 0x0e,
];

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutSlot10_1<T> {
    pub entry: ObjectEntry10_1,
    pub body: Option<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page10_1 {
    pub version: u16,
    pub paper_id: u32,
    pub raw_8c: u32,
    pub orientation: u16,
    pub raw_84: u32,
    pub size: [f64; 2],
    pub stored_printable_extent: [f64; 4],
    pub raw_shorts: [u16; 2],
    pub color: ColorSlot10_1,
    pub border: DecorSlot10_1,
    pub background: DecorSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulerSettings10_1 {
    pub version: u16,
    pub interval: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapGrid10_1 {
    pub version: u16,
    pub raw_short: u16,
    pub spacing: [f64; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapGuides10_1 {
    pub version: u16,
    pub positions: Vec<f64>,
    pub raw_short: u16,
    pub raw_long: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicSnapEnvironment10_1 {
    pub version: u16,
    pub tolerance: f64,
    pub agent_classes: Vec<[u8; 16]>,
    pub extra_class_ids: Vec<[u8; 16]>,
}

pub(crate) const CONVERTER_OPTION: [u8; 16] = [
    0x24, 0x42, 0x31, 0x97, 0xa2, 0x2d, 0x39, 0x49, 0xac, 0xc5, 0xdd, 0x7c, 0x0a, 0x7b, 0xfc, 0x8b,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ConverterOption10_1 {
    pub version: i16,
    pub raw_long: u32,
    pub properties: LayoutSlot10_1<ConverterProperties10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConverterProperties10_1 {
    PropertySet(Box<PropertySetObjectStream10_1>),
    RasterTemplate(Box<crate::layers::image_server_rules::RasterFunctionArgumentsBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutPrepareResult10_1 {
    pub version: i16,
    pub raw_longs: [u32; 2],
    pub converter: LayoutSlot10_1<ConverterOption10_1>,
    pub objects: Vec<ObjectEntry10_1>,
    pub legacy_map: Option<LayoutSlot10_1<Box<(MapPrefix10_1, MapSuffix10_1)>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapOptimizationExtension10_1 {
    pub version: u16,
    pub prepare_results: Vec<LayoutPrepareResult10_1>,
    pub string_2: Option<String>,
    pub values_3: Option<[u32; 2]>,
    pub server_connection: Option<LayoutSlot10_1<AgsServerConnectionName10_1>>,
    pub flag_4: Option<u8>,
    pub flag_5: Option<u8>,
    pub values_6: Option<[u32; 2]>,
    pub string_6: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueLayoutExtension10_1 {
    /// 在原扩展序列中的零基索引。
    pub index: usize,
    pub class_id: [u8; 16],
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutElementAnchor10_1 {
    pub envelope: LayoutSlot10_1<Envelope10_1>,
    pub raw_double: f64,
}

const PAGE_INDEX: [u8; 16] = [
    0x02, 0xb3, 0x84, 0x93, 0xe4, 0x16, 0x27, 0x4f, 0xa6, 0xa8, 0x8c, 0x0c, 0x99, 0xf1, 0x25, 0x79,
];

#[derive(Debug, Clone, PartialEq)]
pub struct PageIndex10_1 {
    pub version: u16,
    pub frame: ObjectEntry10_1,
    pub frame_body: Option<Box<super::map_frame::MapFrame10_1>>,
    pub layer: LayoutSlot10_1<Box<crate::layers::layer_list::LayerBodyPrefix10_1>>,
    pub string_count: i32,
    pub strings: Vec<String>,
    pub first_long: u32,
    pub raw_fields: [u8; 32],
    pub objects: Vec<LayoutSlot10_1<Box<crate::layers::layer_list::LayerBodyPrefix10_1>>>,
    pub second_long: u32,
    pub decor: [DecorSlot10_1; 3],
    pub version_2_long: Option<u32>,
}

impl PageIndex10_1 {
    /// 按原生版本分支换算完整对象 +0x68、+0x78，缺省值按新建实例计算。
    pub fn loaded_index_modes(&self) -> [u32; 2] {
        let stored = u32::from_le_bytes(self.raw_fields[12..16].try_into().unwrap());
        match self.version_2_long {
            Some(value) => [stored, value],
            None => match stored {
                0 => [0, 2],
                1 => [0, 0],
                2 => [0, 1],
                3 => [1, 2],
                4 => [2, 2],
                other => [other, 2],
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageLayoutTail10_1 {
    pub page: LayoutSlot10_1<Page10_1>,
    pub ruler: LayoutSlot10_1<RulerSettings10_1>,
    pub grid: LayoutSlot10_1<SnapGrid10_1>,
    pub guides: [LayoutSlot10_1<SnapGuides10_1>; 2],
    pub snap_environment: GraphicSnapEnvironment10_1,
    pub map: ObjectEntry10_1,
    pub map_body: Option<Box<(MapPrefix10_1, MapSuffix10_1)>>,
    pub raw_shorts: [u16; 4],
    /// 视图范围的四个序列化坐标。
    pub view_extent: [f64; 4],
    /// 版面范围的 XMin、YMin、Width、Height。
    pub layout_origin_size: [f64; 4],
    pub version_2: Option<(u16, u16, u32)>,
    pub version_3_short: Option<u16>,
    pub version_4_double: Option<f64>,
    pub version_5_short: Option<u16>,
    pub version_6_long: Option<u32>,
    pub version_8_short: Option<u16>,
    pub version_9_shorts: Option<[u16; 2]>,
    pub optimization_extensions: Vec<MapOptimizationExtension10_1>,
    pub opaque_extensions: Vec<OpaqueLayoutExtension10_1>,
    pub version_11_slot: Option<ObjectEntry10_1>,
    pub page_index: Option<Box<PageIndex10_1>>,
    /// 已登记 PageIndex 回引头之后的包装字节。
    pub page_index_reference_suffix: Vec<u8>,
    pub legacy_version_12_or_13_doubles: Option<[f64; 2]>,
    pub version_13_long: Option<u32>,
    pub element_values: Vec<u32>,
    pub anchors: Vec<LayoutElementAnchor10_1>,
    pub end_offset: usize,
}

fn invalid(reason: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, reason)
}

fn unsupported(reason: &str) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, reason)
}

fn read_version(r: &mut Reader<'_>, max: u16, name: &str) -> io::Result<u16> {
    let value = r.read_u16_le()?;
    if value > max {
        return Err(invalid(&format!("{name} 版本 {value} 超出范围")));
    }
    Ok(value)
}

fn count(r: &mut Reader<'_>, bytes: usize, max: usize) -> io::Result<usize> {
    let n = r.read_u32_le()?;
    if n > i32::MAX as u32 || n as usize > max.min(r.remaining()) / bytes {
        return Err(invalid("布局集合计数超过声明范围或资源限额"));
    }
    Ok(n as usize)
}

fn doubles<const N: usize>(r: &mut Reader<'_>) -> io::Result<[f64; N]> {
    let mut items = [0.0; N];
    for item in &mut items {
        *item = r.read_f64_le()?;
    }
    Ok(items)
}

fn read_slot<T>(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    expected: [u8; 16],
    name: &str,
    load: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<LayoutSlot10_1<T>> {
    let offset = r.position();
    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        // SystemCore 对键命中的实例查询接口，回引头中的 CLSID 不参与查询。
        ObjectEntry10_1::BackReference { key, .. }
            if refs.registered_class(key) == Some(expected) =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected => Some(load(r, refs)?),
        _ => {
            return Err(unsupported(&format!(
                "{name} @0x{offset:x} 的实现类尚未支持：{entry:?}"
            )));
        }
    };
    Ok(LayoutSlot10_1 { entry, body })
}

fn read_page(r: &mut Reader<'_>, refs: &mut RefTable10_1, max: usize) -> io::Result<Page10_1> {
    let version = read_version(r, 1, "Page")?;
    let paper_id = r.read_u32_le()?;
    let raw_8c = r.read_u32_le()?;
    let orientation = r.read_u16_le()?;
    let raw_84 = r.read_u32_le()?;
    let size = doubles(r)?;
    let stored_printable_extent = doubles(r)?;
    let raw_shorts = [r.read_u16_le()?, r.read_u16_le()?];
    let color = read_color_slot_10_1(r, max, refs)?;
    if matches!(color, ColorSlot10_1::Unsupported { .. }) {
        return Err(unsupported("Page 颜色的实现类尚未支持"));
    }
    let border = read_map_decor_slot_10_1(r, refs, max, DecorKind10_1::Border)?
        .ok_or_else(|| unsupported("Page 边框的实现类尚未支持"))?;
    let background = read_map_decor_slot_10_1(r, refs, max, DecorKind10_1::Background)?
        .ok_or_else(|| unsupported("Page 背景的实现类尚未支持"))?;
    Ok(Page10_1 {
        version,
        paper_id,
        raw_8c,
        orientation,
        raw_84,
        size,
        stored_printable_extent,
        raw_shorts,
        color,
        border,
        background,
    })
}

fn read_guides(r: &mut Reader<'_>, max: usize) -> io::Result<SnapGuides10_1> {
    let version = read_version(r, 1, "SnapGuides")?;
    let n = count(r, 8, max)?;
    let mut positions = Vec::with_capacity(n);
    for _ in 0..n {
        positions.push(r.read_f64_le()?);
    }
    Ok(SnapGuides10_1 {
        version,
        positions,
        raw_short: r.read_u16_le()?,
        raw_long: r.read_u32_le()?,
    })
}

fn read_class_ids(r: &mut Reader<'_>, max: usize) -> io::Result<Vec<[u8; 16]>> {
    let n = count(r, 16, max)?;
    Ok(r.read_bytes(n * 16)?
        .chunks_exact(16)
        .map(|bytes| bytes.try_into().unwrap())
        .collect())
}

fn read_layout_map_body(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    key: u32,
) -> io::Result<(MapPrefix10_1, MapSuffix10_1)> {
    refs.with_nested_graphics(|refs| {
        refs.mark_map(key);
        let body = read_map_with_suffix_10_1(r, max, refs)?;
        if body.1.stop != MapSuffixStop::Complete {
            return Err(unsupported(&body.1.stop.describe()));
        }
        Ok(body)
    })
}

fn read_converter_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LayoutSlot10_1<ConverterProperties10_1>> {
    use crate::layers::image_server_rules::{RASTER_FUNCTION_TEMPLATE_ARGUMENTS, read_arguments};

    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { key, .. }
            if matches!(
                refs.registered_class(key),
                Some(PROPERTY_SET_CLASS_ID | RASTER_FUNCTION_TEMPLATE_ARGUMENTS)
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: PROPERTY_SET_CLASS_ID,
            ..
        } => Some(ConverterProperties10_1::PropertySet(Box::new(
            read_property_set_object_stream_10_1(r, max, refs)?,
        ))),
        ObjectEntry10_1::NewObject {
            class_id: RASTER_FUNCTION_TEMPLATE_ARGUMENTS,
            ..
        } => Some(ConverterProperties10_1::RasterTemplate(Box::new(
            read_arguments(r, max, refs, RASTER_FUNCTION_TEMPLATE_ARGUMENTS, 0)?,
        ))),
        _ => return Err(unsupported("ConverterOption IPropertySet 的实现类尚未支持")),
    };
    Ok(LayoutSlot10_1 { entry, body })
}

pub(crate) fn read_converter_option(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LayoutSlot10_1<ConverterOption10_1>> {
    read_slot(r, refs, CONVERTER_OPTION, "ConverterOption", |r, refs| {
        let version = r.read_u16_le()? as i16;
        if version > 1 {
            return Err(invalid("ConverterOption 版本超限"));
        }
        let raw_long = r.read_u32_le()?;
        let properties = read_converter_properties(r, max, refs)?;
        Ok(ConverterOption10_1 {
            version,
            raw_long,
            properties,
        })
    })
}

fn read_prepare_results(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Vec<LayoutPrepareResult10_1>> {
    // 最短记录包含版本、两个 LONG、空转换选项和零对象数。
    let n = (r.read_u32_le()? as i32).max(0) as usize;
    if n > max.min(r.remaining()) / 30 {
        return Err(invalid("布局准备结果计数超过剩余字节或资源限额"));
    }
    (0..n)
        .map(|_| {
            let version = r.read_u16_le()? as i16;
            if version > 2 {
                return Err(invalid("布局准备结果版本超限"));
            }
            let raw_longs = [r.read_u32_le()?, r.read_u32_le()?];
            let converter = read_converter_option(r, max, refs)?;
            let count = if version == 1 {
                1
            } else {
                let n = (r.read_u32_le()? as i32).max(0) as usize;
                if n > max.min(r.remaining()) / 16 {
                    return Err(invalid("布局准备结果对象计数超过剩余字节或资源限额"));
                }
                n
            };
            let objects = (0..count)
                .map(|_| {
                    let entry = read_object_entry_10_1(r, refs)?;
                    if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                        match class_id {
                            MAP_CLASS_ID => {
                                read_layout_map_body(r, max, refs, key)?;
                            }
                            FEATURE_LAYER_CLASS_ID => {
                                read_complete_layer_body_10_1(r, class_id, max, refs)?;
                            }
                            _ => {
                                return Err(unsupported(&format!(
                                    "布局准备结果对象类尚未支持：{entry:?}"
                                )));
                            }
                        }
                    }
                    Ok(entry)
                })
                .collect::<io::Result<_>>()?;
            let legacy_map = (version == 1)
                .then(|| {
                    let entry = read_object_entry_10_1(r, refs)?;
                    let body = match entry {
                        ObjectEntry10_1::Null => None,
                        ObjectEntry10_1::BackReference { key, .. }
                            if refs.contains_map(key)
                                || refs.registered_class(key) == Some(MAP_CLASS_ID) =>
                        {
                            None
                        }
                        ObjectEntry10_1::NewObject {
                            class_id: MAP_CLASS_ID,
                            key,
                        } => Some(Box::new(read_layout_map_body(r, max, refs, key)?)),
                        _ => return Err(unsupported(&format!("布局准备结果 IMap 槽：{entry:?}"))),
                    };
                    Ok(LayoutSlot10_1 { entry, body })
                })
                .transpose()?;
            Ok(LayoutPrepareResult10_1 {
                version,
                raw_longs,
                converter,
                objects,
                legacy_map,
            })
        })
        .collect()
}

fn read_extension(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MapOptimizationExtension10_1> {
    let version = r.read_u16_le()? as i16;
    if version > 6 {
        return Err(invalid("MapOptimizationLayoutExtension 版本超限"));
    }
    // CartoConverter 0x6ccc0e2a 委派 MappingCoreLib::LoadResults。
    let prepare_results = read_prepare_results(r, max, refs)?;
    let string_2 = (version >= 2)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let (values_3, server_connection) = if version >= 3 {
        let values = [r.read_u32_le()?, r.read_u32_le()?];
        let connection = read_slot(
            r,
            refs,
            AGS_SERVER_CONNECTION_NAME,
            "IAGSServerConnectionName",
            |r, refs| read_ags_server_connection_name_body(r, max, refs),
        )?;
        (Some(values), Some(connection))
    } else {
        (None, None)
    };
    let flag_4 = (version >= 4)
        .then(|| r.read_bytes(1).map(|bytes| bytes[0]))
        .transpose()?;
    let flag_5 = (version >= 5)
        .then(|| r.read_bytes(1).map(|bytes| bytes[0]))
        .transpose()?;
    let (values_6, string_6) = if version >= 6 {
        (
            Some([r.read_u32_le()?, r.read_u32_le()?]),
            Some(read_string_10_1(r, max)?),
        )
    } else {
        (None, None)
    };
    Ok(MapOptimizationExtension10_1 {
        version: version as u16,
        prepare_results,
        string_2,
        values_3,
        server_connection,
        flag_4,
        flag_5,
        values_6,
        string_6,
    })
}

fn read_extensions(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(
    Vec<MapOptimizationExtension10_1>,
    Vec<OpaqueLayoutExtension10_1>,
)> {
    const HEADER_SIZE: usize = 24;
    let n = (r.read_u32_le()? as i32).max(0) as usize;
    if n > max.min(r.remaining()) / HEADER_SIZE {
        return Err(invalid("布局扩展计数超过剩余字节或资源限额"));
    }
    let mut budget = max - n * HEADER_SIZE;
    let mut optimization = Vec::new();
    let mut opaque = Vec::new();
    for index in 0..n {
        let size = u64::from_le_bytes(r.read_bytes(8)?.try_into().unwrap());
        let size = usize::try_from(size)
            .ok()
            .filter(|n| *n <= budget)
            .ok_or_else(|| invalid("PageLayout 扩展体累计长度超过限额"))?;
        budget -= size;
        let class_id: [u8; 16] = r.read_bytes(16)?.try_into().unwrap();
        let mut body = r.read_region(size)?;
        if class_id != OPTIMIZATION_EXTENSION || size == 0 {
            // LayerLib 在类不可实例化时按长度跳过，零长度也不调用 Load。
            opaque.push(OpaqueLayoutExtension10_1 {
                index,
                class_id,
                body: body.read_bytes(size)?.to_vec(),
            });
            continue;
        }
        optimization.push(read_extension(&mut body, max, refs)?);
        if body.remaining() != 0 {
            return Err(invalid("PageLayout 扩展体未完整消费声明范围"));
        }
    }
    Ok((optimization, opaque))
}

fn read_page_index(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max: usize,
    region_offset: usize,
    is_root_map: &impl Fn(u32) -> bool,
) -> io::Result<PageIndex10_1> {
    let version = read_version(r, 2, "PageIndex")?;
    use super::page_layout_prefix::{MAP_FRAME_CLASS_ID, read_map_frame};
    use crate::storage::object_stream::{ObjectHeader, read_header_10_1};

    let mut probe = r.clone();
    let (frame, frame_body) = match read_header_10_1(&mut probe)? {
        ObjectHeader::Null => (ObjectEntry10_1::Null, None),
        ObjectHeader::Object { class_id, key } if refs.contains(key) => {
            if refs.registered_class(key) != Some(MAP_FRAME_CLASS_ID) {
                return Err(unsupported("PageIndex 的回引未指向已确认的 MapFrame"));
            }
            (ObjectEntry10_1::BackReference { class_id, key }, None)
        }
        ObjectHeader::Object {
            class_id: MAP_FRAME_CLASS_ID,
            key,
        } => {
            let body = refs.with_nested_graphics(|refs| {
                read_map_frame(r, region_offset, max, refs, is_root_map)
            })?;
            (
                ObjectEntry10_1::NewObject {
                    class_id: MAP_FRAME_CLASS_ID,
                    key,
                },
                Some(Box::new(body)),
            )
        }
        _ => return Err(unsupported("PageIndex IMapFrame 槽的实现类尚未支持")),
    };
    if frame_body.is_none() {
        *r = probe;
    }
    let layer = read_slot(
        r,
        refs,
        FEATURE_LAYER_CLASS_ID,
        "PageIndex IFeatureLayer",
        |r, refs| read_complete_layer_body_10_1(r, FEATURE_LAYER_CLASS_ID, max, refs).map(Box::new),
    )?;
    let string_count = r.read_u32_le()? as i32;
    let n = string_count.max(0) as usize;
    if n > max.min(r.remaining()) / 4 {
        return Err(invalid("PageIndex 字符串数量超过剩余字节或限额"));
    }
    let strings = (0..n)
        .map(|_| read_string_10_1(r, max))
        .collect::<io::Result<_>>()?;
    let first_long = r.read_u32_le()?;
    let raw_fields = r.read_bytes(32)?.try_into().unwrap();
    let n = (r.read_u32_le()? as i32).max(0) as usize;
    if n > max.min(r.remaining()) / 16 {
        return Err(invalid("PageIndex 对象数量超过剩余字节或限额"));
    }
    let objects = (0..n)
        .map(|_| {
            let entry = read_object_entry_10_1(r, refs)?;
            let body = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: FEATURE_LAYER_CLASS_ID,
                    ..
                } => Some(Box::new(read_complete_layer_body_10_1(
                    r,
                    FEATURE_LAYER_CLASS_ID,
                    max,
                    refs,
                )?)),
                ObjectEntry10_1::NewObject { .. } => {
                    return Err(unsupported("PageIndex 的 IUnknown 列表包含尚未核实的新类"));
                }
                _ => None,
            };
            Ok(LayoutSlot10_1 { entry, body })
        })
        .collect::<io::Result<_>>()?;
    let second_long = r.read_u32_le()?;
    let decor = [
        DecorKind10_1::Border,
        DecorKind10_1::Shadow,
        DecorKind10_1::Background,
    ]
    .map(|kind| {
        read_map_decor_slot_10_1(r, refs, max, kind)?
            .ok_or_else(|| unsupported("PageIndex 装饰对象尚未支持"))
    });
    let [border, shadow, background] = decor;
    let decor = [border?, shadow?, background?];
    let version_2_long = (version >= 2).then(|| r.read_u32_le()).transpose()?;
    Ok(PageIndex10_1 {
        version,
        frame,
        frame_body,
        layer,
        string_count,
        strings,
        first_long,
        raw_fields,
        objects,
        second_long,
        decor,
        version_2_long,
    })
}

fn read_version_11_slot(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max: usize,
    is_root_map: &impl Fn(u32) -> bool,
) -> io::Result<(LayoutSlot10_1<PageIndex10_1>, Vec<u8>)> {
    let size = u64::from_le_bytes(r.read_bytes(8)?.try_into().unwrap());
    if size == 0 {
        return Ok((
            LayoutSlot10_1 {
                entry: ObjectEntry10_1::Null,
                body: None,
            },
            Vec::new(),
        ));
    }
    if size > max as u64 {
        return Err(invalid("PageIndex 声明体长超过限额"));
    }
    let region_offset = r.position();
    let mut bounded = r.read_region(size as usize)?;
    let slot = read_slot(&mut bounded, refs, PAGE_INDEX, "PageIndex", |r, refs| {
        read_page_index(r, refs, max, region_offset, is_root_map)
    })?;
    let suffix = if matches!(slot.entry, ObjectEntry10_1::BackReference { .. }) {
        bounded.read_bytes(bounded.remaining())?.to_vec()
    } else {
        Vec::new()
    };
    if bounded.remaining() != 0 {
        return Err(invalid("PageIndex 未完整读取声明范围"));
    }
    Ok((slot, suffix))
}

/// 从已确认的元素列表末端读取布局体；失败时整个尾段及其引用登记回退。
pub fn read_page_layout_tail_10_1(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    version: u16,
    element_count: u32,
    max: usize,
    is_root_map: impl Fn(u32) -> bool,
) -> io::Result<PageLayoutTail10_1> {
    if version > 15 {
        return Err(invalid("不支持的 PageLayout 版本"));
    }
    let mut r = reader.clone();
    let mut t = refs.clone();
    let page = read_slot(&mut r, &mut t, PAGE, "Page", |r, t| read_page(r, t, max))?;
    let ruler = read_slot(&mut r, &mut t, RULER, "RulerSettings", |r, _| {
        Ok(RulerSettings10_1 {
            version: read_version(r, 1, "RulerSettings")?,
            interval: r.read_f64_le()?,
        })
    })?;
    let grid = read_slot(&mut r, &mut t, GRID, "SnapGrid", |r, _| {
        Ok(SnapGrid10_1 {
            version: read_version(r, 1, "SnapGrid")?,
            raw_short: r.read_u16_le()?,
            spacing: doubles(r)?,
        })
    })?;
    let guides = [
        read_slot(&mut r, &mut t, GUIDES, "SnapGuides", |r, _| {
            read_guides(r, max)
        })?,
        read_slot(&mut r, &mut t, GUIDES, "SnapGuides", |r, _| {
            read_guides(r, max)
        })?,
    ];
    // GraphicSnapEnvironment 的虚表 Load 直接委派，流中没有对象头。
    let snap_environment = GraphicSnapEnvironment10_1 {
        version: read_version(&mut r, 1, "GraphicSnapEnvironment")?,
        tolerance: r.read_f64_le()?,
        agent_classes: read_class_ids(&mut r, max)?,
        extra_class_ids: read_class_ids(&mut r, max)?,
    };
    let map = read_object_entry_10_1(&mut r, &mut t)?;
    let map_body = match map {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { key, .. }
            if is_root_map(key)
                || t.contains_map(key)
                || t.registered_class(key) == Some(MAP_CLASS_ID) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            key,
        } => Some(Box::new(read_layout_map_body(&mut r, max, &mut t, key)?)),
        _ => return Err(unsupported("PageLayout 的 IMap 槽没有已确认的实现类")),
    };
    let raw_shorts = [
        r.read_u16_le()?,
        r.read_u16_le()?,
        r.read_u16_le()?,
        r.read_u16_le()?,
    ];
    let view_extent = doubles(&mut r)?;
    let layout_origin_size = doubles(&mut r)?;
    let version_2 = if version >= 2 {
        Some((r.read_u16_le()?, r.read_u16_le()?, r.read_u32_le()?))
    } else {
        None
    };
    let version_3_short = (version >= 3).then(|| r.read_u16_le()).transpose()?;
    let version_4_double = (version >= 4).then(|| r.read_f64_le()).transpose()?;
    let version_5_short = (version >= 5).then(|| r.read_u16_le()).transpose()?;
    let version_6_long = (version >= 6).then(|| r.read_u32_le()).transpose()?;
    let version_8_short = (version >= 8).then(|| r.read_u16_le()).transpose()?;
    let version_9_shorts = if version >= 9 {
        Some([r.read_u16_le()?, r.read_u16_le()?])
    } else {
        None
    };
    let (optimization_extensions, opaque_extensions) = if version >= 10 {
        read_extensions(&mut r, max, &mut t)?
    } else {
        (Vec::new(), Vec::new())
    };
    let (version_11_slot, page_index_reference_suffix) = if version >= 11 {
        let (slot, suffix) = read_version_11_slot(&mut r, &mut t, max, &is_root_map)?;
        (Some(slot), suffix)
    } else {
        (None, Vec::new())
    };
    let legacy_version_12_or_13_doubles = matches!(version, 12 | 13)
        .then(|| doubles(&mut r))
        .transpose()?;
    let mut version_13_long = None;
    let mut element_values = Vec::new();
    let mut anchors = Vec::new();
    if version >= 13 {
        version_13_long = Some(r.read_u32_le()?);
        let n = count(&mut r, if version >= 15 { 28 } else { 4 }, max)?;
        if n != element_count as usize {
            return Err(invalid("布局尾段计数与元素列表不一致"));
        }
        for _ in 0..n {
            element_values.push(r.read_u32_le()?);
        }
        if version >= 15 {
            for _ in 0..n {
                let envelope = read_slot(
                    &mut r,
                    &mut t,
                    ENVELOPE_CLASS_ID,
                    "布局锚定范围",
                    |r, t| read_envelope_10_1(r, max, t),
                )?;
                anchors.push(LayoutElementAnchor10_1 {
                    envelope,
                    raw_double: r.read_f64_le()?,
                });
            }
        }
    }
    let end_offset = r.position();
    *reader = r;
    *refs = t;
    Ok(PageLayoutTail10_1 {
        page,
        ruler,
        grid,
        guides,
        snap_environment,
        map,
        map_body,
        raw_shorts,
        view_extent,
        layout_origin_size,
        version_2,
        version_3_short,
        version_4_double,
        version_5_short,
        version_6_long,
        version_8_short,
        version_9_shorts,
        optimization_extensions,
        opaque_extensions,
        version_11_slot: version_11_slot.as_ref().map(|slot| slot.entry),
        page_index: version_11_slot.and_then(|slot| slot.body).map(Box::new),
        page_index_reference_suffix,
        legacy_version_12_or_13_doubles,
        version_13_long,
        element_values,
        anchors,
        end_offset,
    })
}
