//! ImageServer.dll 10.1 的二进制 MosaicRule / RenderingRule 对象体。

mod class_ids;
mod merge;
mod mosaic;
mod mosaic_arguments;
mod raster_info;
mod template;

pub(crate) use template::read_input_body;

pub use merge::{
    DecodedMergeRasterInput10_1, MergeRasterInput10_1, MergeRastersArguments10_1,
    MergeRastersPayload10_1,
};
pub use mosaic::{MosaicFunction10_1, MosaicLodInfos10_1};
pub use mosaic_arguments::{ItemUri10_1, MosaicArgumentsPayload10_1};
pub(crate) use raster_info::{
    CLASS_ID as RASTER_INFO_CLASS_ID, RASTER_COLORMAP, RASTER_HISTOGRAM, RASTER_INFOS_CLASS_ID,
    RASTER_STATISTICS, read_raster_info_variant_body_10_1, read_raster_variant_body_10_1,
};
pub use raster_info::{
    RasterInfo10_1, RasterInfoColormap10_1, RasterInfoHistogram10_1, RasterInfoVariantBody10_1,
    RasterInfos10_1, RasterStatistics10_1, RasterVariantBody10_1,
};
pub use template::{
    RasterFunctionTemplate10_1, RasterFunctionVariable10_1, RasterFunctionVariableValue10_1,
    RasterTemplateInput10_1, RasterVariableVariant10_1,
};

use crate::layers::dep_tail::{
    TIME_CLASS_ID, TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, TimeBody10_1, TimeExtentBody10_1,
    TimeReferenceBody10_1, read_time_body_10_1, read_time_extent_body_10_1,
    read_time_reference_body_10_1,
};
use crate::layers::feature_id_set::{
    FEATURE_ID_SET_CLASS_ID, FID_SET_CLASS_ID, FeatureIdSetBody10_1, read_feature_id_set_body_10_1,
    read_fid_set_body_10_1,
};
use crate::layers::raster_renderer::{
    RasterHistogram10_1, RasterRendererSlot10_1, STATS_HISTOGRAM_CLASS_ID,
    read_raster_renderer_slot_prefix_10_1, read_stats_histogram_body_10_1,
};
use crate::objects::geometry::{POINT_CLASS_ID, PointBody10_1, read_point_body_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, PropertyValueObjectStream10_1,
    read_padded_variant_object_stream_10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const MOSAIC_RULE_CLASS_ID: [u8; 16] = [
    0x00, 0x87, 0x63, 0x33, 0x26, 0xd9, 0xcd, 0x4e, 0x9c, 0xe9, 0xbc, 0xaa, 0x96, 0xe8, 0xe8, 0x9c,
];
pub const RENDERING_RULE_CLASS_ID: [u8; 16] = [
    0xf2, 0xc6, 0x2f, 0x91, 0x5f, 0x4b, 0xa3, 0x4a, 0xaf, 0x24, 0x18, 0x70, 0x4a, 0xb5, 0x8f, 0x6e,
];
pub const TIME_INSTANT_CLASS_ID: [u8; 16] = [
    0x87, 0x72, 0xbd, 0x06, 0x85, 0x07, 0x94, 0x42, 0xbd, 0x72, 0xf2, 0x93, 0x3b, 0x7f, 0xd0, 0x0d,
];
pub const IDENTITY_FUNCTION_CLASS_ID: [u8; 16] = [
    0x32, 0xd1, 0x71, 0xdc, 0x2e, 0xe0, 0xff, 0x4d, 0x85, 0x2b, 0x7e, 0x1f, 0x10, 0x02, 0x4d, 0x9b,
];
pub const CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID: [u8; 16] = [
    0x57, 0x39, 0x46, 0x4d, 0x81, 0x3b, 0xba, 0x46, 0xaa, 0x8c, 0x46, 0x54, 0xde, 0x7d, 0x94, 0xe5,
];
pub const STRETCH_FUNCTION_CLASS_ID: [u8; 16] = [
    0xd6, 0xa6, 0xc3, 0xe0, 0x92, 0x88, 0x76, 0x40, 0xb6, 0x14, 0x25, 0x57, 0x17, 0xa5, 0xcf, 0xdf,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSlot10_1<T> {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MosaicRule10_1 {
    pub version: u16,
    pub strings: [String; 3],
    pub sort_value: PropertyValueObjectStream10_1,
    pub short_44: u16,
    pub longs_48_4c: [i32; 2],
    pub point: RuleSlot10_1<PointBody10_1>,
    pub fids: Option<RuleSlot10_1<FeatureIdSetBody10_1>>,
    pub time_value: Option<RuleSlot10_1<MosaicTimeValue10_1>>,
    pub legacy_short: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MosaicTimeValue10_1 {
    Instant {
        version: u16,
        time: RuleSlot10_1<TimeBody10_1>,
        reference: RuleSlot10_1<TimeReferenceBody10_1>,
    },
    Extent(TimeExtentBody10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderingRule10_1 {
    pub version: u16,
    pub strings: [String; 3],
    pub function: RuleSlot10_1<RasterFunctionBody10_1>,
    pub arguments: RuleSlot10_1<RasterFunctionArgumentsBody10_1>,
    pub renderer: RasterRendererSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionBody10_1 {
    /// RGB32Function 与 RasterFunctionCollection 没有基类体。
    pub has_basic_function: bool,
    pub base_version: u16,
    pub version: u16,
    /// 基础版本 ≥2 和 ≥3 分别出现的两条字符串。
    pub base_strings: [Option<String>; 2],
    pub base_long: i32,
    /// StretchFunction v2 的直方图对象队列。
    pub histograms: Option<Vec<RuleSlot10_1<RasterHistogram10_1>>>,
    /// CachedRasterFunction 在派生体中保存的两条字符串。
    pub cache_strings: Option<[String; 2]>,
    pub merge: Option<MergeRastersPayload10_1>,
    pub raster_info: Option<RuleSlot10_1<RasterInfo10_1>>,
    pub mosaic: Option<Box<MosaicFunction10_1>>,
    pub template: Option<Box<RasterFunctionTemplate10_1>>,
    pub collection: Option<Box<RasterFunctionCollection10_1>>,
    /// LocalFunctionImpl 在派生版本后保存的 int32。
    pub local_function_value: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionCollection10_1 {
    /// 有符号原始计数。原生对非正计数不读子对象。
    pub count: i32,
    pub functions: Vec<RuleSlot10_1<RasterFunctionBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionArgumentsBody10_1 {
    /// URIProductNameFilter 的旧版直接保存字段，不含基类体。
    pub has_basic_arguments: bool,
    pub version: u16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub merge: Option<MergeRastersArguments10_1>,
    /// LocalFunctionArgumentsImpl 在基类属性集之后的 short。
    pub local_arguments_version: Option<u16>,
    /// MosaicDB 派生类在基类体之前或之后保存的 short。
    pub mosaic_version: Option<u16>,
    pub mosaic_payload: Option<Box<MosaicArgumentsPayload10_1>>,
}

pub type IdentityFunction10_1 = RasterFunctionBody10_1;
pub type CachedRasterFunctionArguments10_1 = RasterFunctionArgumentsBody10_1;

pub(super) fn unsupported(reason: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, reason.into())
}

pub(super) fn unsupported_class(label: &str, id: &[u8; 16]) -> io::Error {
    unsupported(format!(
        "{label} 未支持的新对象类 {}",
        crate::storage::class_id_hex(id)
    ))
}

fn version(r: &mut Reader<'_>, max: u16, label: &str) -> io::Result<u16> {
    let value = r.read_u16_le()?;
    if value > max {
        return Err(unsupported(format!("{label} 未支持版本 {value}")));
    }
    Ok(value)
}

/// 保留流中的 short 位模式，按原生读取器的有符号上限判断。
fn signed_version(r: &mut Reader<'_>, max: i16, label: &str) -> io::Result<u16> {
    let value = r.read_u16_le()?;
    if (value as i16) > max {
        return Err(unsupported(format!("{label} 未支持版本 {value}")));
    }
    Ok(value)
}

/// 各槽都位于已确认的父对象体中，父级事务统一负责回退。
pub(super) fn slot<'a, T>(
    r: &mut Reader<'a>,
    refs: &mut RefTable10_1,
    f: impl FnOnce(&mut Reader<'a>, &mut RefTable10_1, [u8; 16], u32) -> io::Result<T>,
) -> io::Result<RuleSlot10_1<T>> {
    let entry = read_object_entry_10_1(r, refs)?;
    let body = if let ObjectEntry10_1::NewObject { class_id, key } = entry {
        Some(Box::new(f(r, refs, class_id, key)?))
    } else {
        None
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_time_value(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<MosaicTimeValue10_1> {
    match class_id {
        TIME_EXTENT_CLASS_ID => {
            let extent = read_time_extent_body_10_1(r, max, refs, key)?;
            if extent.stop.is_some() {
                return Err(unsupported("MosaicRule TimeExtent 的嵌套对象未闭合"));
            }
            Ok(MosaicTimeValue10_1::Extent(extent))
        }
        TIME_INSTANT_CLASS_ID => {
            let version = signed_version(r, 1, "TimeInstant")?;
            let time = slot(r, refs, |r, refs, class_id, key| {
                if class_id != TIME_CLASS_ID {
                    return Err(unsupported_class("ITime", &class_id));
                }
                read_time_body_10_1(r, max, refs, key)
            })?;
            let reference = slot(r, refs, |r, refs, class_id, key| {
                if class_id != TIME_REFERENCE_CLASS_ID {
                    return Err(unsupported_class("ITimeReference", &class_id));
                }
                let reference = read_time_reference_body_10_1(r, max, refs, key)?;
                if reference.short_b.is_none() {
                    return Err(unsupported("TimeInstant TimeReference 的嵌套对象未闭合"));
                }
                Ok(reference)
            })?;
            Ok(MosaicTimeValue10_1::Instant {
                version,
                time,
                reference,
            })
        }
        _ => Err(unsupported_class("ITimeValue", &class_id)),
    }
}

pub(crate) fn read_mosaic_rule_10_1(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MosaicRule10_1> {
    let version = signed_version(r, 5, "MosaicRule")?;
    let layout_version = version as i16;
    let first = read_string_10_1(r, max)?;
    let second = read_string_10_1(r, max)?;
    let sort_value = read_padded_variant_object_stream_10_1(r, max, refs)?;
    let third = read_string_10_1(r, max)?;
    let short_44 = r.read_u16_le()?;
    let longs_48_4c = [r.read_u32_le()? as i32, r.read_u32_le()? as i32];
    let point = slot(r, refs, |r, refs, class_id, _| {
        if class_id != POINT_CLASS_ID {
            return Err(unsupported_class("IPoint", &class_id));
        }
        read_point_body_10_1(r, max, refs)
    })?;
    let fids = (layout_version >= 2)
        .then(|| {
            slot(r, refs, |r, refs, class_id, key| match class_id {
                FID_SET_CLASS_ID => read_fid_set_body_10_1(r, refs, key),
                FEATURE_ID_SET_CLASS_ID => read_feature_id_set_body_10_1(r, refs, key),
                _ => Err(unsupported_class("IFIDSet", &class_id)),
            })
        })
        .transpose()?;
    let time_value = (layout_version >= 4)
        .then(|| {
            slot(r, refs, |r, refs, class_id, key| {
                read_time_value(r, refs, max, class_id, key)
            })
        })
        .transpose()?;
    let legacy_short = matches!(version, 3 | 4)
        .then(|| r.read_u16_le())
        .transpose()?;
    Ok(MosaicRule10_1 {
        version,
        strings: [first, second, third],
        sort_value,
        short_44,
        longs_48_4c,
        point,
        fids,
        time_value,
        legacy_short,
    })
}

pub(super) fn read_function(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
    depth: usize,
) -> io::Result<RasterFunctionBody10_1> {
    let child_depth = depth
        .checked_sub(1)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "栅格函数嵌套超过限额"))?;
    let class_id = if class_ids::LOCAL_FUNCTION_ALIASES.contains(&class_id) {
        class_ids::LOCAL_FUNCTION
    } else {
        class_id
    };
    let max_version = match class_id {
        class_ids::MOSAIC_FUNCTION => 9,
        class_ids::RASTER_FUNCTION_TEMPLATE => 3,
        STRETCH_FUNCTION_CLASS_ID
        | class_ids::GEOMETRIC_FUNCTION
        | class_ids::MERGE_RASTERS_FUNCTION => 2,
        class_ids::CACHED_RASTER_FUNCTION
        | class_ids::RGB32_FUNCTION
        | class_ids::RASTER_INFO_FUNCTION
        | class_ids::RASTER_ITEM_FUNCTION
        | class_ids::RASTER_FUNCTION_COLLECTION
        | class_ids::LOCAL_FUNCTION => 1,
        _ if class_ids::FUNCTIONS.contains(&class_id) => 1,
        _ => return Err(unsupported_class("IRasterFunction", &class_id)),
    };
    let has_basic_function = !matches!(
        class_id,
        class_ids::RGB32_FUNCTION | class_ids::RASTER_FUNCTION_COLLECTION
    );
    let (base_version, base_strings, base_long) = has_basic_function
        .then(|| read_function_base(r, max))
        .transpose()?
        .unwrap_or_default();
    let version = signed_version(r, max_version, "栅格函数派生体")?;
    let cache_strings = (class_id == class_ids::CACHED_RASTER_FUNCTION)
        .then(|| Ok::<_, io::Error>([read_string_10_1(r, max)?, read_string_10_1(r, max)?]))
        .transpose()?;
    let merge = (class_id == class_ids::MERGE_RASTERS_FUNCTION && version == 2)
        .then(|| merge::read_payload(r, max, refs, child_depth))
        .transpose()?;
    let raster_info = matches!(
        class_id,
        class_ids::RASTER_INFO_FUNCTION | class_ids::RASTER_ITEM_FUNCTION
    )
    .then(|| raster_info::read_slot(r, max, refs, child_depth))
    .transpose()?;
    let mosaic = (class_id == class_ids::MOSAIC_FUNCTION)
        .then(|| mosaic::read_payload(r, max, refs, version as i16, child_depth).map(Box::new))
        .transpose()?;
    let template = (class_id == class_ids::RASTER_FUNCTION_TEMPLATE)
        .then(|| template::read_template(r, max, refs, version as i16, child_depth).map(Box::new))
        .transpose()?;
    let collection = (class_id == class_ids::RASTER_FUNCTION_COLLECTION)
        .then(|| read_function_collection(r, max, refs, child_depth).map(Box::new))
        .transpose()?;
    let local_function_value = (class_id == class_ids::LOCAL_FUNCTION)
        .then(|| r.read_u32_le().map(|value| value as i32))
        .transpose()?;
    let histograms = if class_id == STRETCH_FUNCTION_CLASS_ID && version == 2 {
        let count = (r.read_u32_le()? as i32).max(0) as usize;
        if count > r.remaining().min(max) / 16 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "StretchFunction 直方图数量超限",
            ));
        }
        let items = (0..count)
            .map(|_| {
                slot(r, refs, |r, _, id, _| {
                    if id != STATS_HISTOGRAM_CLASS_ID {
                        return Err(unsupported_class("StretchFunction 直方图", &id));
                    }
                    read_stats_histogram_body_10_1(r)
                })
            })
            .collect::<io::Result<_>>()?;
        Some(items)
    } else {
        None
    };
    Ok(RasterFunctionBody10_1 {
        has_basic_function,
        base_version,
        version,
        base_strings,
        base_long,
        histograms,
        cache_strings,
        merge,
        raster_info,
        mosaic,
        template,
        collection,
        local_function_value,
    })
}

fn read_function_collection(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<RasterFunctionCollection10_1> {
    let count = r.read_u32_le()? as i32;
    let items = count.max(0) as usize;
    if items > r.remaining().min(max) / 20 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterFunctionCollection 子函数数量超限",
        ));
    }
    let mut functions = Vec::new();
    for _ in 0..items {
        let function = slot(r, refs, |r, refs, class_id, _| {
            read_function(r, max, refs, class_id, child_depth)
        })?;
        // 原生 Load 成功后直接对条目调用 AddRef。
        if function.entry == ObjectEntry10_1::Null {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "RasterFunctionCollection 子函数为空",
            ));
        }
        functions.push(function);
    }
    Ok(RasterFunctionCollection10_1 { count, functions })
}

fn read_function_base(
    r: &mut Reader<'_>,
    max: usize,
) -> io::Result<(u16, [Option<String>; 2], i32)> {
    let version = signed_version(r, 4, "BasicFunction")?;
    let layout_version = version as i16;
    // v0/v1 只有 int32，v2 在 int32 后读一条字符串，v3/v4 先读两条字符串。
    let first_long = (layout_version < 3).then(|| r.read_u32_le()).transpose()?;
    let strings = [
        (layout_version >= 2)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
        (layout_version >= 3)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
    ];
    let long = match first_long {
        Some(value) => value,
        None => r.read_u32_le()?,
    } as i32;
    Ok((version, strings, long))
}

pub(super) fn supports_function_arguments(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        class_ids::MERGE_RASTERS_ARGUMENTS | class_ids::LOCAL_FUNCTION_ARGUMENTS
    ) || class_ids::LOCAL_ARGUMENT_ALIASES.contains(&class_id)
        || class_ids::ARGUMENTS.contains(&class_id)
        || mosaic_arguments::layout(class_id).is_some()
}

fn read_basic_arguments(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(u16, RuleSlot10_1<PropertySetObjectStream10_1>)> {
    let base_version = signed_version(r, 1, "BasicFunctionArguments")?;
    Ok((base_version, read_arguments_properties(r, max, refs)?))
}

fn read_arguments_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<PropertySetObjectStream10_1>> {
    slot(r, refs, |r, refs, class_id, _| {
        if class_id != PROPERTY_SET_CLASS_ID {
            return Err(unsupported_class("IPropertySet", &class_id));
        }
        read_property_set_object_stream_10_1(r, max, refs)
    })
}

pub(super) fn read_arguments(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
    depth: usize,
) -> io::Result<RasterFunctionArgumentsBody10_1> {
    if let Some(layout) = mosaic_arguments::layout(class_id) {
        return mosaic_arguments::read(r, max, refs, layout, depth);
    }
    if !supports_function_arguments(class_id) {
        return Err(unsupported_class("IRasterFunctionArguments", &class_id));
    }
    let merge_version = (class_id == class_ids::MERGE_RASTERS_ARGUMENTS)
        .then(|| merge::read_arguments_version(r))
        .transpose()?;
    let (base_version, properties) = read_basic_arguments(r, max, refs)?;
    let merge = merge_version
        .map(|version| merge::read_arguments(r, max, refs, version, depth))
        .transpose()?;
    let local_arguments_version = (class_id == class_ids::LOCAL_FUNCTION_ARGUMENTS
        || class_ids::LOCAL_ARGUMENT_ALIASES.contains(&class_id))
    .then(|| signed_version(r, 1, "LocalFunctionArgumentsImpl"))
    .transpose()?;
    Ok(RasterFunctionArgumentsBody10_1 {
        has_basic_arguments: true,
        version: base_version,
        properties,
        merge,
        local_arguments_version,
        mosaic_version: None,
        mosaic_payload: None,
    })
}

pub(crate) fn read_rendering_rule_10_1(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RenderingRule10_1> {
    let version = signed_version(r, 1, "RenderingRule")?;
    let first = read_string_10_1(r, max)?;
    let second = read_string_10_1(r, max)?;
    let function = slot(r, refs, |r, refs, class_id, _| {
        read_function(
            r,
            max,
            refs,
            class_id,
            super::raster_dataset_name::MAX_NAME_NESTING,
        )
    })?;
    let arguments = slot(r, refs, |r, refs, class_id, _| {
        read_arguments(
            r,
            max,
            refs,
            class_id,
            super::raster_dataset_name::MAX_NAME_NESTING,
        )
    })?;
    let third = read_string_10_1(r, max)?;
    let renderer = read_raster_renderer_slot_prefix_10_1(r, max, refs)?;
    if renderer
        .body
        .as_ref()
        .is_some_and(|body| !body.is_complete())
    {
        return Err(unsupported("RenderingRule 的 IRasterRenderer 对象体未闭合"));
    }
    Ok(RenderingRule10_1 {
        version,
        strings: [first, second, third],
        function,
        arguments,
        renderer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::ZlibEncoder};
    use std::io::Write;

    const EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];

    fn rendering_rule_with_arguments(id: [u8; 16], arguments: &[u8]) -> Vec<u8> {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING.repeat(2));
        bytes.extend([0; 16]);
        bytes.extend(id);
        bytes.extend(12_u32.to_le_bytes());
        bytes.extend(arguments);
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]);
        bytes
    }

    fn check_mosaic_argument(
        id: [u8; 16],
        arguments: &[u8],
        derived_version: Option<u16>,
    ) -> RasterFunctionArgumentsBody10_1 {
        let mut bytes = rendering_rule_with_arguments(id, arguments);
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut RefTable10_1::new()).unwrap();
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);
        for cut in 50..end {
            assert!(
                read_rendering_rule_10_1(
                    &mut Reader::new(&bytes[..cut]),
                    4096,
                    &mut RefTable10_1::new(),
                )
                .is_err(),
                "截断偏移 {cut}",
            );
        }
        let arguments = rule.arguments.body.unwrap();
        assert_eq!(arguments.mosaic_version, derived_version);
        *arguments
    }

    #[test]
    fn mosaic_crawlers_follow_distinct_legacy_name_orders() {
        use crate::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        use crate::maps::standalone_table::TABLE_NAME_CLASS_ID;
        use MosaicArgumentsPayload10_1 as Payload;

        let raster = class_ids::class_id(0x6a5bb65a, 0x9cf2, 0x4aa4, 0x9eb1d7e8de9a6dd6);
        let table = class_ids::class_id(0x8b77c68d, 0x71ba, 0x4707, 0xaa032fb6e4a2c52b);
        for version in 1_u16..=3 {
            let mut raster_bytes = vec![0; 18]; // 基类 v0 与空属性集。
            raster_bytes.extend(version.to_le_bytes());
            if version == 3 {
                raster_bytes.extend((-170_i32).to_le_bytes());
                raster_bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
                raster_bytes.extend(13_u32.to_le_bytes());
                raster_bytes.extend(1_i32.to_le_bytes());
                raster_bytes.extend(EMPTY_STRING.repeat(4));
                raster_bytes.extend([0; 16]);
            }
            let body = check_mosaic_argument(raster, &raster_bytes, Some(version));
            match (version, body.mosaic_payload.as_deref()) {
                (3, Some(Payload::RasterDatasetCrawler { long: -170, name })) => {
                    assert!(name.body.is_some());
                }
                (1 | 2, None) => {}
                _ => panic!("{body:?}"),
            }

            let mut table_bytes = vec![0; 18];
            table_bytes.extend(version.to_le_bytes());
            if version < 3 {
                table_bytes.extend(TABLE_NAME_CLASS_ID);
                table_bytes.extend(13_u32.to_le_bytes());
                table_bytes.extend(1_u16.to_le_bytes());
                table_bytes.extend(EMPTY_STRING.repeat(3));
                table_bytes.extend([0; 16]);
            }
            if version == 2 {
                table_bytes.extend((-912_i32).to_le_bytes());
            }
            let body = check_mosaic_argument(table, &table_bytes, Some(version));
            match (version, body.mosaic_payload.as_deref()) {
                (1 | 2, Some(Payload::TableCrawler { name, long })) => {
                    assert!(matches!(
                        name,
                        crate::objects::dataset_name_more::DatasetNameMoreSlot10_1::Body(_)
                    ));
                    assert_eq!(*long, (version == 2).then_some(-912));
                }
                (3, None) => {}
                _ => panic!("{body:?}"),
            }
        }
    }

    #[test]
    fn mosaic_inline_builders_and_add_rasters_use_the_host_reference_table() {
        use MosaicArgumentsPayload10_1 as Payload;
        use class_ids::class_id;

        let mut basic = 1_u16.to_le_bytes().to_vec();
        basic.extend(PROPERTY_SET_CLASS_ID);
        basic.extend(13_u32.to_le_bytes());
        basic.extend(1_u16.to_le_bytes());
        basic.extend(0_i32.to_le_bytes());
        let builders = [
            (
                class_id(0x1e076602, 0xa69e, 0x4ff6, 0x9c2ce23f1ff2cd80),
                2_u16,
            ),
            (class_id(0x2c2351f9, 0x9b7c, 0x4208, 0x8d1734d7496007ee), 2),
            (class_id(0xc055b1ff, 0xf26c, 0x488c, 0xb8111f871b00f4a6), 1),
            (
                class_id(0xa9965bfe, 0x7862, 0x4077, 0xa76d83c3a943d258),
                940,
            ),
        ];
        for (id, outer_version) in builders {
            let mut bytes = basic.clone();
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(outer_version.to_le_bytes());
            let body = check_mosaic_argument(id, &bytes, Some(outer_version));
            assert_eq!(body.properties.body.unwrap().version, 1);
            assert!(matches!(
                body.mosaic_payload.as_deref(),
                Some(Payload::InlineRasterDatasetBuilder { version: 3 })
            ));

            let bad_version = basic.len() + 2;
            bytes[bad_version..].copy_from_slice(&(outer_version + 1).to_le_bytes());
            let bytes = rendering_rule_with_arguments(id, &bytes);
            assert_eq!(
                read_rendering_rule_10_1(&mut Reader::new(&bytes), 4096, &mut RefTable10_1::new())
                    .unwrap_err()
                    .kind(),
                ErrorKind::Unsupported,
            );
        }

        let mut bytes = basic;
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(13_u32.to_le_bytes());
        let id = class_id(0x50088aff, 0x736a, 0x4448, 0xb00a18eb0c4971ea);
        let body = check_mosaic_argument(id, &bytes, None);
        let Some(Payload::AddRasters {
            version: 0,
            properties,
        }) = body.mosaic_payload.as_deref()
        else {
            panic!("{body:?}");
        };
        assert!(matches!(
            properties.entry,
            ObjectEntry10_1::BackReference { key: 13, .. }
        ));
    }

    #[test]
    fn mosaic_uri_product_name_filter_reads_both_legacy_arrays() {
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;
        use MosaicArgumentsPayload10_1 as Payload;

        let id = class_ids::class_id(0x227397d1, 0xc518, 0x4edb, 0xaddfa07160aad029);
        let mut bytes = (-1_i16).to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING);
        bytes.extend(77_u16.to_le_bytes());
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(13_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(13_u32.to_le_bytes());
        let body = check_mosaic_argument(id, &bytes, Some(u16::MAX));
        assert!(!body.has_basic_arguments);
        let Some(Payload::UriProductNames {
            short: 77, arrays, ..
        }) = body.mosaic_payload.as_deref()
        else {
            panic!("{body:?}");
        };
        assert_eq!(arrays[0].body.as_ref().unwrap().1, [""]);
        assert!(matches!(
            arrays[1].entry,
            ObjectEntry10_1::BackReference { key: 13, .. }
        ));

        let mut bytes = 940_u16.to_le_bytes().to_vec();
        bytes.extend([0; 18]);
        let body = check_mosaic_argument(id, &bytes, Some(940));
        assert!(body.has_basic_arguments);
        assert!(body.mosaic_payload.is_none());
    }

    #[test]
    fn mosaic_raster_type_v943_reads_two_parameter_slots() {
        use crate::layers::raster_dataset_name::RASTER_TYPE_NAME_CLASS_ID;
        use MosaicArgumentsPayload10_1 as Payload;

        let id = class_ids::class_id(0xc7ee2957, 0xe230, 0x4be0, 0xae55a459b9647e21);
        let child_id = class_ids::class_id(0x50088aff, 0x736a, 0x4448, 0xb00a18eb0c4971ea);
        let mut base = vec![0; 18];
        base.extend(RASTER_TYPE_NAME_CLASS_ID);
        base.extend(13_u32.to_le_bytes());
        base.extend(1_u16.to_le_bytes());
        base.extend(EMPTY_STRING.repeat(2));
        base.extend(56_i32.to_le_bytes());
        base.extend([0; 16]);

        for version in [2_u16, 942, 943] {
            let mut bytes = base.clone();
            bytes.extend(version.to_le_bytes());
            if version == 943 {
                bytes.extend(child_id);
                bytes.extend(14_u32.to_le_bytes());
                bytes.extend([0; 36]); // AddRastersParameters 与 SynchronizeParameters 的基类。
                bytes.extend(child_id);
                bytes.extend(14_u32.to_le_bytes());
            }
            let body = check_mosaic_argument(id, &bytes, Some(version));
            let Some(Payload::RasterType {
                name,
                add_rasters,
                synchronize,
            }) = body.mosaic_payload.as_deref()
            else {
                panic!("{body:?}");
            };
            assert!(name.body.is_some());
            if version == 943 {
                assert!(matches!(
                    add_rasters
                        .as_ref()
                        .unwrap()
                        .body
                        .as_ref()
                        .unwrap()
                        .mosaic_payload
                        .as_deref(),
                    Some(Payload::AddRasters { .. })
                ));
                assert!(matches!(
                    synchronize.as_ref().unwrap().entry,
                    ObjectEntry10_1::BackReference { key: 14, .. }
                ));
            } else {
                assert!(add_rasters.is_none() && synchronize.is_none());
            }
        }
    }

    #[test]
    fn mosaic_overview_uri_reads_three_inline_bodies() {
        use crate::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1;
        use MosaicArgumentsPayload10_1 as Payload;

        let id = class_ids::class_id(0x550753de, 0x479a, 0x46ba, 0xa1c76eb86070d89c);
        for version in [1_u16, 940, 941] {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend(EMPTY_STRING.repeat(5));
            bytes.extend(63.5_f64.to_le_bytes());
            if version > 1 {
                bytes.extend([0; 16]); // ItemURI 的 IStringArray。
            }
            if version == 941 {
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(13_u32.to_le_bytes());
            }

            bytes.extend(0_u16.to_le_bytes()); // CellSizeLevel 的基类。
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(4_u16.to_le_bytes()); // FunctionRasterDatasetName。
            bytes.extend(EMPTY_STRING.repeat(3));
            bytes.extend([0; 16]); // 工作空间。
            bytes.extend(EMPTY_STRING); // 路径。
            bytes.extend([0; 16]); // 函数。
            bytes.extend(0_i32.to_le_bytes()); // 零计数时仍有一个输入槽。
            bytes.extend([0; 48]); // 输入、属性集、几何。

            let body = check_mosaic_argument(id, &bytes, None);
            let Some(Payload::OverviewItemUri {
                item_uri,
                cell_size_version: 0,
                cell_size_properties,
                function_name,
            }) = body.mosaic_payload.as_deref()
            else {
                panic!("{body:?}");
            };
            assert_eq!(item_uri.version, version);
            assert_eq!(item_uri.value, 63.5);
            assert_eq!(item_uri.strings_array.is_some(), version > 1);
            assert_eq!(item_uri.properties.is_some(), version == 941);
            assert!(matches!(
                cell_size_properties.entry,
                ObjectEntry10_1::BackReference { key: 13, .. }
            ));
            assert!(matches!(
                function_name,
                RasterDatasetNameEntryBody10_1::Function(_)
            ));
        }
    }

    #[test]
    fn mosaic_argument_versions_follow_the_persisted_class_layout() {
        use class_ids::class_id;

        let cases = [
            // AnalyzeParameters
            (
                class_id(0xa411f679, 0x9803, 0x479d, 0x885a2318a4b373e9),
                None,
                None,
                None,
            ),
            // URIPropertiesFilter
            (
                class_id(0xaa3e2f92, 0xb23b, 0x427b, 0x8e2a68c0eeca4dfa),
                Some(0_u16),
                None,
                Some(2_u16),
            ),
            // FileCrawler
            (
                class_id(0x1b13f960, 0x8914, 0x4440, 0xa2d508b1b16c916d),
                None,
                Some(1_u16),
                Some(0),
            ),
            // RPDefBuilder
            (
                class_id(0xbc7c6260, 0xc8fb, 0x40ac, 0x98458067f1185f92),
                None,
                Some(2),
                Some(1),
            ),
            // LasBuilder
            (
                class_id(0xda19b110, 0x389b, 0x4300, 0x9c8a49687e41e4bd),
                None,
                Some(0),
                Some(3),
            ),
            // LandsatBuilder
            (
                class_id(0x661cf6c7, 0xcd79, 0x4e72, 0xa0237df0a413041f),
                None,
                Some(1),
                Some(2),
            ),
            // TableBuilder
            (
                class_id(0xea44519b, 0x5080, 0x4001, 0x8054cd5a93b761cb),
                None,
                Some(3),
                Some(2),
            ),
        ];
        for (id, prefix, suffix, invalid_version) in cases {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(EMPTY_STRING.repeat(2));
            bytes.extend([0; 16]); // 空 IRasterFunction。
            bytes.extend(id);
            bytes.extend(12_u32.to_le_bytes());
            let prefix_offset = bytes.len();
            if let Some(prefix) = prefix {
                bytes.extend(prefix.to_le_bytes());
            }
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            let suffix_offset = bytes.len();
            if let Some(suffix) = suffix {
                bytes.extend(suffix.to_le_bytes());
            }
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16]); // 空 IRasterRenderer。
            let end = bytes.len();
            bytes.push(0xa5);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
            let body = rule.arguments.body.unwrap();
            assert_eq!(body.version, 1);
            assert_eq!(body.mosaic_version, prefix.or(suffix));
            assert_eq!(body.properties.body.unwrap().version, 1);
            assert!((12..=13).all(|key| refs.contains(key)));
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);

            for cut in prefix_offset..end {
                assert!(
                    read_rendering_rule_10_1(
                        &mut Reader::new(&bytes[..cut]),
                        4096,
                        &mut RefTable10_1::new(),
                    )
                    .is_err()
                );
            }
            if let Some(invalid) = invalid_version {
                let offset = if prefix.is_some() {
                    prefix_offset
                } else {
                    suffix_offset
                };
                bytes[offset..offset + 2].copy_from_slice(&invalid.to_le_bytes());
                assert_eq!(
                    read_rendering_rule_10_1(
                        &mut Reader::new(&bytes),
                        4096,
                        &mut RefTable10_1::new(),
                    )
                    .unwrap_err()
                    .kind(),
                    ErrorKind::Unsupported
                );
            }
        }
    }

    #[test]
    fn function_collection_reads_raster_item_info_and_reuses_keys() {
        for arguments in [
            class_ids::RASTER_ITEM_FUNCTION_ARGUMENTS,
            class_ids::RASTER_FUNCTION_TEMPLATE_ARGUMENTS,
            class_ids::RASTER_INFO_FUNCTION_ARGUMENTS,
        ] {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(EMPTY_STRING.repeat(2));
            bytes.extend(class_ids::RASTER_FUNCTION_COLLECTION);
            bytes.extend(12_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            let count_offset = bytes.len();
            bytes.extend(3_i32.to_le_bytes());
            let first_child = bytes.len();
            bytes.extend(class_ids::RASTER_ITEM_FUNCTION);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes()); // BasicFunction v1。
            bytes.extend(8_i32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes()); // RasterItemFunction v1。
            bytes.extend(raster_info::CLASS_ID);
            bytes.extend(14_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 4 * 16 + 2 * 8 + 2 * 4 + 16 + 4 * 4]);
            bytes.extend(17_u16.to_le_bytes()); // RasterInfo 标志位。
            bytes.extend(class_ids::RASTER_ITEM_FUNCTION);
            bytes.extend(13_u32.to_le_bytes()); // 子函数回引。
            bytes.extend(class_ids::RASTER_FUNCTION_COLLECTION);
            bytes.extend(15_u32.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend(arguments);
            bytes.extend(16_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(17_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16]);
            let end = bytes.len();
            bytes.push(0xa5);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
            let function = rule.function.body.unwrap();
            assert!(!function.has_basic_function);
            let collection = function.collection.unwrap();
            assert_eq!(collection.count, 3);
            let item = collection.functions[0].body.as_ref().unwrap();
            assert!(item.has_basic_function);
            let info = item.raster_info.as_ref().unwrap().body.as_ref().unwrap();
            assert_eq!((item.base_long, info.flags), (8, 17));
            assert!(matches!(
                collection.functions[1].entry,
                ObjectEntry10_1::BackReference { key: 13, .. }
            ));
            let empty = collection.functions[2].body.as_ref().unwrap();
            assert!(!empty.has_basic_function);
            assert_eq!(empty.collection.as_ref().unwrap().count, -1);
            assert!(empty.collection.as_ref().unwrap().functions.is_empty());
            assert!(rule.arguments.body.unwrap().properties.body.is_some());
            assert_eq!(refs.len(), 6);
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);

            for cut in 0..end {
                assert!(
                    read_rendering_rule_10_1(
                        &mut Reader::new(&bytes[..cut]),
                        4096,
                        &mut RefTable10_1::new()
                    )
                    .is_err()
                );
            }
            let mut invalid = bytes.clone();
            invalid[count_offset..first_child].copy_from_slice(&i32::MAX.to_le_bytes());
            assert_eq!(
                read_rendering_rule_10_1(
                    &mut Reader::new(&invalid),
                    4096,
                    &mut RefTable10_1::new()
                )
                .unwrap_err()
                .kind(),
                ErrorKind::InvalidData
            );
            invalid[count_offset..first_child].copy_from_slice(&3_i32.to_le_bytes());
            invalid[first_child..first_child + 16].fill(0);
            assert_eq!(
                read_rendering_rule_10_1(
                    &mut Reader::new(&invalid),
                    4096,
                    &mut RefTable10_1::new()
                )
                .unwrap_err()
                .kind(),
                ErrorKind::InvalidData
            );
        }
    }

    fn push_compressed_variant(bytes: &mut Vec<u8>, inner: &[u8]) {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(inner).unwrap();
        let compressed = encoder.finish().unwrap();
        bytes.extend(0x2011_u16.to_le_bytes());
        bytes.extend([0; 14]);
        bytes.extend(((8 + compressed.len()) as u32).to_le_bytes());
        bytes.extend((inner.len() as u32).to_le_bytes());
        bytes.extend((compressed.len() as u32).to_le_bytes());
        bytes.extend(compressed);
    }

    #[test]
    fn merge_rasters_rule_consumes_three_independent_compressed_streams() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING);
        bytes.extend(EMPTY_STRING);
        bytes.extend(class_ids::MERGE_RASTERS_FUNCTION);
        bytes.extend(12_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        let compressed_offset = bytes.len();
        let mut payload = vec![0; 16]; // 空 IRasterInfo2。
        payload.extend(1_u32.to_le_bytes());
        let rectangle = [1.0_f64, 2.0, 30.0, 40.0];
        payload.extend(rectangle.into_iter().flat_map(f64::to_le_bytes));
        push_compressed_variant(&mut bytes, &payload);

        bytes.extend(class_ids::MERGE_RASTERS_ARGUMENTS);
        bytes.extend(13_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes()); // 派生体版本在基类体之前。
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 16]); // 空 IPropertySet。
        let mut first_input = [0xf1; 16].to_vec();
        first_input.extend(12_u32.to_le_bytes());
        first_input.push(0x79);
        let mut second_input = [0xf2; 16].to_vec();
        second_input.extend(12_u32.to_le_bytes());
        second_input.push(0x86);
        push_compressed_variant(&mut bytes, &first_input);
        push_compressed_variant(&mut bytes, &second_input);
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 空 IRasterRenderer。

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(
            result.function.body.unwrap().merge,
            Some(MergeRastersPayload10_1::Rectangles(vec![rectangle]))
        );
        let args = result.arguments.body.unwrap().merge.unwrap();
        assert_eq!(args.version, 2);
        for (input, (class_id, value)) in args.inputs.into_iter().zip([(0xf1, 0x79), (0xf2, 0x86)])
        {
            assert_eq!(
                input.entry,
                ObjectEntry10_1::NewObject {
                    class_id: [class_id; 16],
                    key: 12,
                }
            );
            assert_eq!(input.remaining, [value]);
        }
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(12) && refs.contains(13));
        assert_eq!(reader.remaining(), 0);

        let mut corrupted = bytes.clone();
        let raw_len = 4 + compressed_offset + 20;
        let compressed_len =
            u32::from_le_bytes(corrupted[raw_len..raw_len + 4].try_into().unwrap()) as usize;
        corrupted[raw_len + 4 + compressed_len - 1] ^= 1;
        assert!(
            read_rendering_rule_10_1(&mut Reader::new(&corrupted), 4096, &mut RefTable10_1::new(),)
                .is_err()
        );

        for cut in 0..bytes.len() {
            assert!(
                read_rendering_rule_10_1(
                    &mut Reader::new(&bytes[..cut]),
                    4096,
                    &mut RefTable10_1::new(),
                )
                .is_err(),
                "截断位置 {cut}"
            );
        }
        for (relative_offset, invalid) in [(20, 1_u32), (20, u32::MAX), (24, u32::MAX)] {
            let mut broken = bytes.clone();
            let offset = compressed_offset + relative_offset;
            broken[offset..offset + 4].copy_from_slice(&invalid.to_le_bytes());
            assert!(read_rendering_rule_10_1(
                &mut Reader::new(&broken),
                4096,
                &mut RefTable10_1::new(),
            )
            .is_err());
        }
    }

    #[test]
    fn merge_rasters_arguments_decode_rasters_and_footprints_in_separate_streams() {
        use crate::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::geometry::{GEOMETRY_BAG_CLASS_ID, GeometryBody10_1};

        let mut rasters = RASTER_DATASET_NAME_CLASS_ID.to_vec();
        rasters.extend(12_u32.to_le_bytes());
        rasters.extend(2_i32.to_le_bytes());
        rasters.extend(EMPTY_STRING.repeat(4));
        rasters.extend([0; 16]); // 空工作空间。

        let mut footprints = GEOMETRY_BAG_CLASS_ID.to_vec();
        footprints.extend(12_u32.to_le_bytes());
        footprints.extend(1_u16.to_le_bytes());
        footprints.extend(2_i32.to_le_bytes());
        footprints.extend([0; 16]); // 缓存几何。
        footprints.extend(POINT_CLASS_ID);
        footprints.extend(13_u32.to_le_bytes());
        footprints.extend(1_u16.to_le_bytes());
        footprints.extend(4_i32.to_le_bytes());
        footprints.extend(0_i32.to_le_bytes()); // 空 Point 的记录类型。
        footprints.extend(POINT_CLASS_ID);
        footprints.extend(13_u32.to_le_bytes());
        footprints.extend([0; 17]); // 空空间参考和标志。

        let make_rule = |first: &[u8], second: &[u8]| {
            let mut argument_body = (-1_i16).to_le_bytes().to_vec();
            argument_body.extend(1_u16.to_le_bytes());
            argument_body.extend([0; 16]);
            push_compressed_variant(&mut argument_body, first);
            push_compressed_variant(&mut argument_body, second);
            rendering_rule_with_arguments(class_ids::MERGE_RASTERS_ARGUMENTS, &argument_body)
        };
        let bytes = make_rule(&rasters, &footprints);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        let args = rule.arguments.body.unwrap().merge.unwrap();
        assert_eq!(args.version, u16::MAX);
        assert!(matches!(
            args.inputs[0].decoded.as_deref(),
            Some(DecodedMergeRasterInput10_1::RasterInput(
                RasterTemplateInput10_1::RasterName(_)
            ))
        ));
        let Some(DecodedMergeRasterInput10_1::Object(PropertyValueObjectStream10_1::Geometry(
            geometry,
        ))) = args.inputs[1].decoded.as_deref()
        else {
            panic!("应读取 Footprints 的几何集合");
        };
        let GeometryBody10_1::GeometryBag(bag) = geometry.body() else {
            panic!("应读取 GeometryBag");
        };
        assert!(matches!(
            &bag.members[0].entry,
            ObjectEntry10_1::NewObject { key: 13, .. }
        ));
        assert!(matches!(
            &bag.members[1].entry,
            ObjectEntry10_1::BackReference { key: 13, .. }
        ));
        assert_eq!(args.inputs[0].remaining, &rasters[20..]);
        assert_eq!(args.inputs[1].remaining, &footprints[20..]);
        assert!(refs.contains(12));
        assert!(!refs.contains(13));
        assert_eq!(reader.remaining(), 0);

        let mut extra = footprints.clone();
        extra.push(0xa5);
        for invalid in [&footprints[..footprints.len() - 1], &extra[..]] {
            let bytes = make_rule(&rasters, invalid);
            assert!(
                read_rendering_rule_10_1(&mut Reader::new(&bytes), 4096, &mut RefTable10_1::new(),)
                    .is_err()
            );
        }
    }

    #[test]
    fn merge_rasters_retains_unresolved_raster_info_inside_its_bound() {
        let class_id = [
            0xca, 0x5f, 0x1e, 0x4f, 0x34, 0x23, 0x80, 0x42, 0xb1, 0x09, 0x17, 0xb3, 0xca, 0xf9,
            0x29, 0x0e,
        ];
        let mut inner = class_id.to_vec();
        inner.extend(12_u32.to_le_bytes());
        inner.extend([8, 0, 0xff]);
        let mut bytes = Vec::new();
        push_compressed_variant(&mut bytes, &inner);
        bytes.push(0xa8);
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            merge::read_payload(&mut reader, 4096, &RefTable10_1::new(), 16).unwrap(),
            MergeRastersPayload10_1::UnresolvedRasterInfo(MergeRasterInput10_1 {
                entry: ObjectEntry10_1::NewObject { class_id, key: 12 },
                remaining: vec![8, 0, 0xff],
                decoded: None,
            })
        );
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa8]);
    }

    #[test]
    fn raster_info_function_and_merge_read_statistics_and_colormap_in_order() {
        let mut info = raster_info::CLASS_ID.to_vec();
        info.extend(14_u32.to_le_bytes());
        info.extend(7_u16.to_le_bytes());
        info.extend([0; 4 * 16]); // 变换、包络、空间参考、包络。
        info.extend([0; 2 * 8 + 2 * 4]);
        for (class_id, key, prefix) in [
            (RASTER_STATISTICS, 18_u32, &([0; 6 * 8 + 2 * 4])[..]),
            (RASTER_HISTOGRAM, 19, &[][..]),
            (RASTER_COLORMAP, 20, &[][..]),
        ] {
            info.extend(13_u16.to_le_bytes());
            info.extend([0; 14]);
            info.extend(class_id);
            info.extend(key.to_le_bytes());
            info.extend(1_u16.to_le_bytes());
            info.extend(prefix);
        }
        info.extend(13_u16.to_le_bytes());
        info.extend([0; 14]);
        info.extend(STATS_HISTOGRAM_CLASS_ID);
        info.extend(21_u32.to_le_bytes());
        info.extend(3_i32.to_le_bytes());
        info.extend([0; 6 * 8]);
        info.extend(1_i32.to_le_bytes());
        info.extend(7.25_f64.to_le_bytes());
        let nan_bits = 0x7ff8_0000_0000_0007_u64;
        info.extend(nan_bits.to_le_bytes());
        info.extend([0; 3 * 8 + 4]); // RasterHistogram 的 double 和 long。
        info.extend(1_i32.to_le_bytes());
        info.extend(9.25_f64.to_le_bytes());
        info.extend([0; 4 * 4 + 2]); // RasterInfo 的后继字段。
        info.extend([0; 16]); // v2 的点。
        info.extend(EMPTY_STRING);
        info.extend(2_i32.to_le_bytes());
        info.extend(RASTER_STATISTICS);
        info.extend(15_u32.to_le_bytes());
        info.extend(2_u16.to_le_bytes());
        info.extend([0; 6 * 8 + 2 * 4 + 16]);
        info.extend(13_u16.to_le_bytes());
        info.extend([0; 14]);
        info.extend(RASTER_COLORMAP);
        info.extend(20_u32.to_le_bytes()); // 回引前面 Variant 内的色表。
        info.extend(RASTER_STATISTICS);
        info.extend(15_u32.to_le_bytes());
        info.extend(1_i32.to_le_bytes());
        info.extend(RASTER_HISTOGRAM);
        info.extend(16_u32.to_le_bytes());
        info.extend(1_u16.to_le_bytes());
        info.extend([0; 16 + 3 * 8 + 4]);
        info.extend(1_i32.to_le_bytes());
        info.extend(8.25_f64.to_le_bytes());
        info.extend(RASTER_COLORMAP);
        info.extend(17_u32.to_le_bytes());
        info.extend(2_u16.to_le_bytes());
        info.extend([0; 2 * 16]);
        info.extend([0; 16]); // v6 槽为空。
        info.extend(170_i32.to_le_bytes());

        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING);
        bytes.extend(EMPTY_STRING);
        bytes.extend(class_ids::RASTER_INFO_FUNCTION);
        bytes.extend(13_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(5_i32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(&info);
        bytes.extend([0; 16]);
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        let body = rule
            .function
            .body
            .unwrap()
            .raster_info
            .unwrap()
            .body
            .unwrap();
        assert_eq!(body.v7_long, Some(170));
        let stats = body.statistics[0].body.as_ref().unwrap();
        assert_eq!(stats.version, 2);
        assert_eq!(
            stats.v2_value,
            Some(PropertyValueObjectStream10_1::BackReference {
                class_id: RASTER_COLORMAP,
                key: 20,
            })
        );
        let PropertyValueObjectStream10_1::Raster {
            body: statistics, ..
        } = &body.value
        else {
            panic!("应读到 Variant 栅格统计");
        };
        let RasterVariantBody10_1::Statistics(statistics) = statistics.as_ref() else {
            panic!("应读到 RasterStatistics");
        };
        let PropertyValueObjectStream10_1::Raster {
            body: histogram, ..
        } = &statistics.value
        else {
            panic!("应读到 Variant 直方图");
        };
        let RasterVariantBody10_1::Histogram(histogram) = histogram.as_ref() else {
            panic!("应读到 RasterHistogram");
        };
        assert_eq!(histogram.bins, [9.25]);
        let PropertyValueObjectStream10_1::Raster { body: colormap, .. } = &histogram.value else {
            panic!("应读到 Variant 色表");
        };
        let RasterVariantBody10_1::Colormap(colormap) = colormap.as_ref() else {
            panic!("应读到 RasterColormap");
        };
        let PropertyValueObjectStream10_1::Raster {
            body: histogram, ..
        } = &colormap.value
        else {
            panic!("应读到 Variant 统计直方图");
        };
        let RasterVariantBody10_1::StatsHistogram(histogram) = histogram.as_ref() else {
            panic!("应读到 StatsHistogram");
        };
        assert_eq!(histogram.bins, [7.25]);
        assert_eq!(histogram.double_a.map(f64::to_bits), Some(nan_bits));
        assert!(matches!(
            body.statistics[1].entry,
            ObjectEntry10_1::BackReference { key: 15, .. }
        ));
        assert_eq!(body.histograms[0].body.as_ref().unwrap().bins, [8.25]);
        assert_eq!(
            body.colormap
                .as_ref()
                .unwrap()
                .body
                .as_ref()
                .unwrap()
                .version,
            2
        );
        assert!((13..=21).all(|key| refs.contains(key)));
        assert_eq!(reader.remaining(), 0);

        info.extend(1_i32.to_le_bytes());
        let rectangle = [1.0_f64, 2.0, 3.0, 4.0];
        info.extend(rectangle.into_iter().flat_map(f64::to_le_bytes));
        let mut compressed = Vec::new();
        push_compressed_variant(&mut compressed, &info);
        // 压缩流中的键来自独立会话，其新栅格信息仍计入外层递归。
        let mut outer_refs = RefTable10_1::new();
        outer_refs.register(14);
        let mut reader = Reader::new(&compressed);
        let MergeRastersPayload10_1::RasterInfo {
            raster_info,
            rectangles,
        } = merge::read_payload(&mut reader, 4096, &outer_refs, 16).unwrap()
        else {
            panic!("应读取非空 RasterInfo");
        };
        assert_eq!(raster_info.body, Some(body));
        assert_eq!(rectangles, [rectangle]);
        assert_eq!(reader.remaining(), 0);
        assert_eq!(outer_refs.len(), 1);

        fn read_at_depth_limit(
            refs: &mut RefTable10_1,
            compressed: &[u8],
            remaining: usize,
        ) -> io::Result<MergeRastersPayload10_1> {
            if remaining == 0 {
                return merge::read_payload(&mut Reader::new(compressed), 4096, refs, 16);
            }
            refs.with_nested_raster_info(|refs| {
                read_at_depth_limit(refs, compressed, remaining - 1)
            })
        }
        let original = outer_refs.clone();
        let error = read_at_depth_limit(&mut outer_refs, &compressed, 16).unwrap_err();
        assert!(error.to_string().contains("跨流嵌套"));
        assert_eq!(outer_refs, original);
    }

    #[test]
    fn mosaic_function_reads_lods_fids_and_versioned_tail() {
        let lod_class = [
            0x14, 0x5e, 0xf1, 0xe3, 0x2a, 0x51, 0xaf, 0x49, 0x80, 0x78, 0xef, 0x31, 0x81, 0x3d,
            0x6e, 0xab,
        ];
        for version in [-1_i16, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(EMPTY_STRING);
            bytes.extend(EMPTY_STRING);
            bytes.extend(class_ids::MOSAIC_FUNCTION);
            bytes.extend(12_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 16]); // IRasterInfo。
            for _ in 0..5 {
                bytes.extend(EMPTY_STRING);
            }
            bytes.extend(lod_class);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            let count_offset = bytes.len();
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(
                [10.0_f64, 20.0, 30.0, 40.0]
                    .into_iter()
                    .flat_map(f64::to_le_bytes),
            );
            bytes.extend([0; 16 + 2 + 16 + 4]); // 两个 double、short、ISymbol、int32。
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16]); // Variant(false)。
            bytes.extend(EMPTY_STRING);
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16 + 2 + 4]); // IPoint、short、int32。
            bytes.extend(EMPTY_STRING);
            bytes.extend(EMPTY_STRING);
            if version >= 2 {
                bytes.extend(22_u16.to_le_bytes());
            }
            if matches!(version, 3 | 4) {
                bytes.extend([0; 16]);
            }
            if version >= 4 {
                bytes.extend([0; 16]);
            }
            if version >= 6 {
                bytes.extend(FID_SET_CLASS_ID);
                bytes.extend(14_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(3_i32.to_le_bytes());
                bytes.extend(44_i32.to_le_bytes());
                bytes.extend(44_i32.to_le_bytes());
                bytes.extend((-1_i32).to_le_bytes());
            }
            for tail_version in 7..=version {
                bytes.extend(tail_version.to_le_bytes());
            }
            bytes.extend([0; 16]); // 外层参数。
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16]); // 外层渲染器。

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
            let function = rule.function.body.unwrap();
            assert_eq!(function.version, version as u16);
            let mosaic = function.mosaic.unwrap();
            let lod = mosaic.lod_infos.body.unwrap();
            assert_eq!(lod.pairs, [[10.0, 20.0]]);
            assert_eq!(lod.trailing_doubles, [30.0, 40.0]);
            assert_eq!(mosaic.legacy_pnt.is_some(), matches!(version, 3 | 4));
            assert_eq!(mosaic.geometry.is_some(), version >= 4);
            assert_eq!(
                mosaic.fids.map(|slot| slot.body.unwrap().ids),
                (version >= 6).then(|| vec![44])
            );
            assert_eq!(mosaic.shorts[5], (version == 9).then_some(9));
            assert!(refs.contains(12) && refs.contains(13));
            assert_eq!(refs.contains(14), version >= 6);
            assert_eq!(reader.remaining(), 0);

            if version == 9 {
                let mut broken = bytes.clone();
                broken[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
                for input in [&broken[..], &bytes[..bytes.len() - 1]] {
                    assert!(
                        read_rendering_rule_10_1(
                            &mut Reader::new(input),
                            4096,
                            &mut RefTable10_1::new(),
                        )
                        .is_err()
                    );
                }
            }
        }
    }

    #[test]
    fn template_raster_infos_and_direct_raster_info_share_member_keys() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING.repeat(2));
        bytes.extend(class_ids::RASTER_FUNCTION_TEMPLATE);
        bytes.extend(20_u32.to_le_bytes());
        let template_body = bytes.len();
        bytes.extend(1_u16.to_le_bytes()); // 基类版本。
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes()); // 模板版本。
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 子函数。
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(raster_info::RASTER_INFOS_CLASS_ID);
        bytes.extend(21_u32.to_le_bytes());
        bytes.extend((-1_i16).to_le_bytes());
        let count_offset = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        let first_member = bytes.len();
        bytes.extend(raster_info::CLASS_ID);
        bytes.extend(22_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend([0; 4 * 16 + 2 * 8 + 2 * 4 + 16 + 4 * 4]);
        bytes.extend(9_u16.to_le_bytes()); // RasterInfo 标志。
        bytes.extend(POINT_CLASS_ID);
        bytes.extend(23_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes()); // 空点记录。
        bytes.extend(raster_info::CLASS_ID);
        bytes.extend(22_u32.to_le_bytes());
        bytes.extend(raster_info::CLASS_ID);
        bytes.extend(24_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend([0; 4 * 16 + 2 * 8 + 2 * 4 + 16 + 4 * 4 + 2]);
        bytes.extend(POINT_CLASS_ID);
        bytes.extend(23_u32.to_le_bytes());
        bytes.extend(class_ids::RASTER_FUNCTION_VARIABLE);
        bytes.extend(25_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        bytes.extend(raster_info::RASTER_INFOS_CLASS_ID);
        bytes.extend(21_u32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 2 + 16]); // 变量标志和 IStringArray 空槽。
        bytes.extend(raster_info::CLASS_ID);
        bytes.extend(26_u32.to_le_bytes());
        bytes.extend((-2_i16).to_le_bytes());
        bytes.extend([0; 4 * 16 + 2 * 8 + 2 * 4 + 16 + 4 * 4 + 2]);
        bytes.extend([0; 16]); // 外层参数。
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 外层渲染器。
        let end = bytes.len();
        bytes.push(0x7e);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        let template = rule.function.body.unwrap().template.unwrap();
        let Some(RasterTemplateInput10_1::RasterInfos(collection)) =
            template.inputs[0].body.as_deref()
        else {
            panic!("应读取 RasterInfos");
        };
        assert_eq!(collection.version, u16::MAX);
        assert_eq!(collection.declared_count, 2);
        assert_eq!(collection.entries[0].body.as_ref().unwrap().flags, 9);
        assert_eq!(
            collection.entries[1].entry,
            ObjectEntry10_1::BackReference {
                class_id: raster_info::CLASS_ID,
                key: 22
            }
        );
        let Some(RasterTemplateInput10_1::RasterInfo(info)) = template.inputs[1].body.as_deref()
        else {
            panic!("应读取 RasterInfo");
        };
        assert_eq!(
            info.point.as_ref().unwrap().entry,
            ObjectEntry10_1::BackReference {
                class_id: POINT_CLASS_ID,
                key: 23
            }
        );
        let Some(RasterTemplateInput10_1::Variable(variable)) = template.inputs[2].body.as_deref()
        else {
            panic!("应读取模板变量");
        };
        assert!(matches!(
            &variable.value.as_ref().unwrap().value,
            RasterVariableVariant10_1::Object { object, .. } if object.entry ==
                (ObjectEntry10_1::BackReference { class_id: raster_info::RASTER_INFOS_CLASS_ID, key: 21 })
        ));
        let Some(RasterTemplateInput10_1::RasterInfo(legacy)) = template.inputs[3].body.as_deref()
        else {
            panic!("应读取旧版本 RasterInfo");
        };
        assert_eq!(legacy.version, (-2_i16) as u16);
        assert!(legacy.point.is_none());
        assert_eq!(reader.position(), end);
        assert!((20..=26).all(|key| refs.contains(key)));

        let mut invalid_count = bytes.clone();
        invalid_count[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let mut null_member = bytes.clone();
        null_member[first_member..first_member + 16].fill(0);
        for data in (0..end)
            .map(|cut| &bytes[..cut])
            .chain([&invalid_count[..], &null_member[..]])
        {
            assert!(
                read_rendering_rule_10_1(&mut Reader::new(data), 4096, &mut RefTable10_1::new(),)
                    .is_err()
            );
        }
        let mut reader = Reader::new(&bytes[template_body..]);
        let error = read_function(
            &mut reader,
            4096,
            &mut RefTable10_1::new(),
            class_ids::RASTER_FUNCTION_TEMPLATE,
            2,
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("栅格信息对象嵌套"));
    }

    #[test]
    fn rendering_rule_reads_native_scalar_and_argument_array_with_shared_keys() {
        use crate::objects::property_set::{
            PropertyValue10_1, RASTER_ARGUMENT_ARRAY_CLASS_ID, RASTER_SCALAR_CLASS_ID,
        };

        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING.repeat(2));
        bytes.extend(class_ids::RASTER_FUNCTION_TEMPLATE);
        bytes.extend(20_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes()); // 基类版本。
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes()); // 模板版本。
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 子函数槽。
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(RASTER_ARGUMENT_ARRAY_CLASS_ID);
        bytes.extend(21_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(EMPTY_STRING.repeat(2));
        bytes.extend(5_i32.to_le_bytes());
        bytes.extend(RASTER_SCALAR_CLASS_ID);
        bytes.extend(22_u32.to_le_bytes());
        bytes.extend((-1_i16).to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend([0xa5; 14]);
        bytes.extend(42_i32.to_le_bytes());
        bytes.extend(RASTER_SCALAR_CLASS_ID);
        bytes.extend(22_u32.to_le_bytes());
        bytes.extend([0; 16]); // 保留数组的空对象项。
        bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
        bytes.extend(27_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
        bytes.extend(27_u32.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(26_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(26_u32.to_le_bytes());
        bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
        bytes.extend(23_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(24_u32.to_le_bytes());
        let property_body_start = bytes.len();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(13_u16.to_le_bytes()); // 属性集 Variant 无保留区。
        bytes.extend(RASTER_ARGUMENT_ARRAY_CLASS_ID);
        bytes.extend(25_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(EMPTY_STRING.repeat(2));
        let count_offset = bytes.len();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(RASTER_SCALAR_CLASS_ID);
        bytes.extend(22_u32.to_le_bytes());
        let property_body_end = bytes.len();
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 渲染器槽。
        let end = bytes.len();
        bytes.push(0x7e);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        let template = rule.function.body.unwrap().template.unwrap();
        let Some(RasterTemplateInput10_1::NativeArgument(
            PropertyValueObjectStream10_1::RasterArgumentArray(array),
        )) = template.inputs[0].body.as_deref()
        else {
            panic!("模板应读取 ArgumentArray");
        };
        assert!(matches!(
            template.inputs[1].body.as_deref(),
            Some(RasterTemplateInput10_1::NativeArgument(
                PropertyValueObjectStream10_1::Object { key: 26, .. }
            ))
        ));
        assert_eq!(
            template.inputs[2].entry,
            ObjectEntry10_1::BackReference {
                class_id: PROPERTY_SET_CLASS_ID,
                key: 26,
            }
        );
        let reference = PropertyValueObjectStream10_1::BackReference {
            class_id: RASTER_SCALAR_CLASS_ID,
            key: 22,
        };
        assert_eq!(array.declared_count, 5);
        assert_eq!(
            array.objects[0],
            PropertyValueObjectStream10_1::RasterScalar {
                key: 22,
                version: u16::MAX,
                value: Box::new(PropertyValueObjectStream10_1::Inline(
                    PropertyValue10_1::Raw4(42_i32.to_le_bytes())
                )),
            }
        );
        assert_eq!(array.objects[1], reference);
        assert_eq!(
            array.objects[2],
            PropertyValueObjectStream10_1::Inline(PropertyValue10_1::NullObject)
        );
        let PropertyValueObjectStream10_1::RasterArgumentInput(item) = &array.objects[3] else {
            panic!("应读取数组中的栅格参数对象");
        };
        assert_eq!(item.key, 27);
        assert!(matches!(item.body(), RasterTemplateInput10_1::Arguments(_)));
        assert_eq!(
            array.objects[4],
            PropertyValueObjectStream10_1::BackReference {
                class_id: CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID,
                key: 27,
            }
        );
        let properties = rule.arguments.body.unwrap().properties.body.unwrap();
        let PropertyValueObjectStream10_1::RasterArgumentArray(array) =
            &properties.entries[0].value
        else {
            panic!("参数属性集应读取 ArgumentArray");
        };
        assert_eq!(array.key, 25);
        assert_eq!(array.objects, [reference]);
        assert_eq!(reader.position(), end);
        assert!((20..=25).all(|key| refs.contains(key)));

        let mut overflow = bytes[property_body_start..property_body_end].to_vec();
        let count_offset = count_offset - property_body_start;
        overflow[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let mut prior_refs = RefTable10_1::new();
        prior_refs.register(22);
        for input in [
            &overflow[..],
            &bytes[property_body_start..property_body_end - 1],
        ] {
            let mut reader = Reader::new(input);
            let mut refs = prior_refs.clone();
            assert!(read_property_set_object_stream_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, prior_refs);
        }
    }

    #[test]
    fn template_reads_nested_function_variable_name_and_shared_string_array() {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;

        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING);
        bytes.extend(EMPTY_STRING);
        bytes.extend(class_ids::RASTER_FUNCTION_TEMPLATE);
        bytes.extend(20_u32.to_le_bytes());
        let template_body = bytes.len();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(IDENTITY_FUNCTION_CLASS_ID);
        bytes.extend(21_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(5_i32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        let count_offset = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(class_ids::RASTER_FUNCTION_VARIABLE);
        bytes.extend(22_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(23_u32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        for _ in 0..4 {
            bytes.extend(EMPTY_STRING);
        }
        bytes.extend([0; 16]); // 名称的工作空间槽。
        bytes.extend(EMPTY_STRING);
        bytes.extend(0x1234_u16.to_le_bytes());
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(24_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        let second_input = bytes.len();
        bytes.extend(class_ids::RASTER_FUNCTION_VARIABLE);
        bytes.extend(22_u32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(24_u32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 外层参数槽。
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]); // 外层渲染器槽。

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        let template = rule.function.body.unwrap().template.unwrap();
        assert_eq!(template.function.body.unwrap().base_long, 5);
        let RasterTemplateInput10_1::Variable(variable) =
            template.inputs[0].body.as_deref().unwrap()
        else {
            panic!("应读取栅格函数变量");
        };
        let value = variable.value.as_ref().unwrap();
        let RasterVariableVariant10_1::Object { vartype, object } = &value.value else {
            panic!("应读取非空变量对象值");
        };
        assert_eq!(*vartype, 13);
        assert!(matches!(
            object.body.as_deref(),
            Some(RasterTemplateInput10_1::RasterName(_))
        ));
        assert_eq!(value.flags, 0x1234);
        assert_eq!(value.string_array.body.as_ref().unwrap().1, [""]);
        assert!(matches!(
            template.inputs[1].entry,
            ObjectEntry10_1::BackReference { key: 22, .. }
        ));
        assert!(matches!(
            template.string_array.unwrap().entry,
            ObjectEntry10_1::BackReference { key: 24, .. }
        ));
        assert_eq!(reader.remaining(), 0);
        assert_eq!(refs.len(), 5);

        let mut legacy = bytes.clone();
        legacy.drain(second_input..second_input + 20);
        legacy[count_offset..count_offset + 4].copy_from_slice(&(-1_i32).to_le_bytes());
        let mut reader = Reader::new(&legacy);
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut RefTable10_1::new()).unwrap();
        assert_eq!(
            rule.function.body.unwrap().template.unwrap().inputs.len(),
            1
        );
        assert_eq!(reader.remaining(), 0);

        let mut invalid = bytes.clone();
        invalid[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        for data in [&invalid[..], &bytes[..bytes.len() - 1]] {
            assert!(
                read_rendering_rule_10_1(&mut Reader::new(data), 4096, &mut RefTable10_1::new(),)
                    .is_err()
            );
        }
        let mut reader = Reader::new(&bytes[template_body..]);
        let error = read_function(
            &mut reader,
            4096,
            &mut RefTable10_1::new(),
            class_ids::RASTER_FUNCTION_TEMPLATE,
            1,
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("嵌套"));
    }

    #[test]
    fn external_function_pairs_read_base_and_local_suffixes() {
        let las = [
            0x67, 0x8a, 0xa7, 0x06, 0x8a, 0xba, 0x92, 0x45, 0x83, 0xe5, 0x5d, 0xd4, 0x36, 0x2f,
            0x66, 0x36,
        ];
        let las_arguments = [
            0xa0, 0x6a, 0x87, 0x67, 0xeb, 0x40, 0x0a, 0x47, 0xab, 0xc9, 0x03, 0x60, 0x85, 0x75,
            0x6c, 0x53,
        ];
        let base_cases = [
            (las, las_arguments, false),
            (
                class_ids::LOCAL_FUNCTION,
                class_ids::LOCAL_FUNCTION_ARGUMENTS,
                true,
            ),
        ];
        let public_cases = class_ids::LOCAL_FUNCTION_ALIASES
            .into_iter()
            .zip(class_ids::LOCAL_ARGUMENT_ALIASES)
            .map(|(function, arguments)| (function, arguments, true));
        for (function, arguments, local) in base_cases.into_iter().chain(public_cases) {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(EMPTY_STRING);
            bytes.extend(EMPTY_STRING);
            bytes.extend(function);
            bytes.extend(12_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(8_i32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            if local {
                bytes.extend(41_i32.to_le_bytes());
            }
            bytes.extend(arguments);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(14_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            let suffix_offset = bytes.len();
            if local {
                bytes.extend(1_u16.to_le_bytes());
            }
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(
                rule.function.entry,
                ObjectEntry10_1::NewObject {
                    class_id: function,
                    key: 12
                }
            );
            assert_eq!(
                rule.arguments.entry,
                ObjectEntry10_1::NewObject {
                    class_id: arguments,
                    key: 13
                }
            );
            assert_eq!(
                rule.function.body.unwrap().local_function_value,
                local.then_some(41)
            );
            let arguments = rule.arguments.body.unwrap();
            assert_eq!(arguments.local_arguments_version, local.then_some(1));
            assert!(arguments.properties.body.is_some());
            assert_eq!(reader.remaining(), 0);
            assert!((12..=14).all(|key| refs.contains(key)));

            if local {
                bytes[suffix_offset..suffix_offset + 2].copy_from_slice(&2_u16.to_le_bytes());
                let error = read_rendering_rule_10_1(
                    &mut Reader::new(&bytes),
                    4096,
                    &mut RefTable10_1::new(),
                )
                .unwrap_err();
                assert_eq!(error.kind(), ErrorKind::Unsupported);
                assert!(error.to_string().contains("LocalFunctionArgumentsImpl"));
            }
        }
    }

    #[test]
    fn mosaic_rule_uses_signed_versions_for_optional_objects() {
        for version in [-1_i16, 4] {
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend(EMPTY_STRING.repeat(2));
            bytes.extend([0; 16]); // 空 Variant。
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 2 + 8 + 16]); // short、两个 int32、IPoint。
            if version == 4 {
                bytes.extend([0; 16]); // IFIDSet。
                bytes.extend(TIME_INSTANT_CLASS_ID);
                bytes.extend(12_u32.to_le_bytes());
                bytes.extend((-1_i16).to_le_bytes());
                bytes.extend([0; 32]); // ITime 和 ITimeReference。
                bytes.extend(0x1234_u16.to_le_bytes());
            }
            let end = bytes.len();
            bytes.push(0x53);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let rule = read_mosaic_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(rule.version, version as u16);
            if version == 4 {
                assert!(rule.fids.is_some());
                assert!(matches!(
                    rule.time_value.unwrap().body.as_deref(),
                    Some(MosaicTimeValue10_1::Instant {
                        version: u16::MAX,
                        ..
                    })
                ));
                assert_eq!(rule.legacy_short, Some(0x1234));
                assert!(refs.contains(12));
            } else {
                assert!(rule.fids.is_none());
                assert!(rule.time_value.is_none());
                assert!(rule.legacy_short.is_none());
            }
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);
        }
    }

    #[test]
    fn rendering_rule_uses_signed_versions_for_template_and_parameters() {
        let mut bytes = (-1_i16).to_le_bytes().to_vec();
        bytes.extend(EMPTY_STRING.repeat(2));
        bytes.extend(class_ids::RASTER_FUNCTION_TEMPLATE);
        bytes.extend(12_u32.to_le_bytes());
        bytes.extend(i16::MIN.to_le_bytes());
        bytes.extend(21_i32.to_le_bytes());
        let derived_offset = bytes.len();
        bytes.extend((-1_i16).to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(class_ids::RASTER_FUNCTION_VARIABLE);
        bytes.extend(13_u32.to_le_bytes());
        bytes.extend((-1_i16).to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(class_ids::LOCAL_FUNCTION_ARGUMENTS);
        bytes.extend(14_u32.to_le_bytes());
        bytes.extend((-1_i16).to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(i16::MIN.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0x53);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(rule.version, u16::MAX);
        let function = rule.function.body.unwrap();
        assert_eq!(function.base_version, 0x8000);
        assert_eq!(function.base_long, 21);
        assert_eq!(function.base_strings, [None, None]);
        assert_eq!(function.version, u16::MAX);
        let template = function.template.unwrap();
        assert!(template.second_string.is_none());
        assert!(template.string_array.is_none());
        assert!(template.third_string.is_none());
        let Some(RasterTemplateInput10_1::Variable(input)) = template.inputs[0].body.as_deref()
        else {
            panic!("应读取模板的旧版变量");
        };
        assert_eq!(input.version, u16::MAX);
        assert!(input.value.is_none());
        let arguments = rule.arguments.body.unwrap();
        assert_eq!(arguments.version, u16::MAX);
        assert_eq!(arguments.local_arguments_version, Some(0x8000));
        assert!((12..=14).all(|key| refs.contains(key)));
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);

        let mut invalid = bytes.clone();
        invalid[derived_offset..derived_offset + 2].copy_from_slice(&4_i16.to_le_bytes());
        assert_eq!(
            read_rendering_rule_10_1(&mut Reader::new(&invalid), 4096, &mut RefTable10_1::new(),)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported,
        );
    }

    #[test]
    fn rendering_rule_reads_native_function_variants_and_reflectance_arguments() {
        for (class_id, derived_version, has_base, cache_strings) in [
            (class_ids::FUNCTIONS[0], 1_u16, true, None),
            (class_ids::GEOMETRIC_FUNCTION, 2, true, None),
            (class_ids::RGB32_FUNCTION, 1, false, None),
            (
                class_ids::CACHED_RASTER_FUNCTION,
                1,
                true,
                Some(["甲", "乙"]),
            ),
        ] {
            let mut bytes = vec![1, 0];
            bytes.extend(EMPTY_STRING);
            bytes.extend(EMPTY_STRING);
            bytes.extend(class_id);
            bytes.extend(12_u32.to_le_bytes());
            if has_base {
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(4_i32.to_le_bytes());
            }
            let version_offset = bytes.len();
            bytes.extend(derived_version.to_le_bytes());
            if let Some(strings) = cache_strings {
                for value in strings {
                    let unit = value.encode_utf16().next().unwrap();
                    bytes.extend(4_u32.to_le_bytes());
                    bytes.extend(unit.to_le_bytes());
                    bytes.extend(0_u16.to_le_bytes());
                }
            }
            // ReflectanceFunctionArguments 的参数槽，后接规则的输出字符串与渲染器。
            bytes.extend([
                0x74, 0xc8, 0x2d, 0x08, 0xcc, 0x8a, 0xd8, 0x4f, 0xa1, 0xd1, 0xae, 0xaf, 0x73, 0xca,
                0x89, 0x9a,
            ]);
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend(EMPTY_STRING);
            bytes.extend([0; 16]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let rule = read_rendering_rule_10_1(&mut reader, 4096, &mut refs).unwrap();
            let function = rule.function.body.unwrap();
            assert_eq!(function.version, derived_version);
            assert_eq!(function.has_basic_function, has_base);
            assert_eq!(function.base_long, if has_base { 4 } else { 0 });
            assert_eq!(
                function.cache_strings,
                cache_strings.map(|pair| pair.map(String::from))
            );
            assert_eq!(rule.arguments.body.unwrap().version, 1);
            assert_eq!(reader.remaining(), 0);
            assert!(refs.contains(12) && refs.contains(13));

            let mut invalid = bytes;
            invalid[version_offset..version_offset + 2]
                .copy_from_slice(&(derived_version + 1).to_le_bytes());
            assert_eq!(
                read_rendering_rule_10_1(
                    &mut Reader::new(&invalid),
                    4096,
                    &mut RefTable10_1::new(),
                )
                .unwrap_err()
                .kind(),
                ErrorKind::Unsupported
            );
        }
    }
}
