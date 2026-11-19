//! ImageServer.dll 10.1 的二进制 MosaicRule / RenderingRule 对象体。

mod class_ids;
mod merge;
mod mosaic;
mod raster_info;
mod template;

pub use merge::{MergeRasterInput10_1, MergeRastersArguments10_1, MergeRastersPayload10_1};
pub use mosaic::{MosaicFunction10_1, MosaicLodInfos10_1};
pub use raster_info::{
    RasterInfo10_1, RasterInfoColormap10_1, RasterInfoHistogram10_1, RasterStatistics10_1,
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
    /// RGB32Function 没有基类体，此时基类版本与 int32 为 0，字符串为空槽。
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
    /// LocalFunctionImpl 在派生版本后保存的 int32。
    pub local_function_value: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionArgumentsBody10_1 {
    pub version: u16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub merge: Option<MergeRastersArguments10_1>,
    /// LocalFunctionArgumentsImpl 在基类属性集之后的 short。
    pub local_arguments_version: Option<u16>,
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
            let version = version(r, 1, "TimeInstant")?;
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
    let version = version(r, 5, "MosaicRule")?;
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
    let fids = (version >= 2)
        .then(|| {
            slot(r, refs, |r, refs, class_id, key| match class_id {
                FID_SET_CLASS_ID => read_fid_set_body_10_1(r, refs, key),
                FEATURE_ID_SET_CLASS_ID => read_feature_id_set_body_10_1(r, refs, key),
                _ => Err(unsupported_class("IFIDSet", &class_id)),
            })
        })
        .transpose()?;
    let time_value = (version >= 4)
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
    let max_version = match class_id {
        class_ids::MOSAIC_FUNCTION => 9,
        class_ids::RASTER_FUNCTION_TEMPLATE => 3,
        STRETCH_FUNCTION_CLASS_ID
        | class_ids::GEOMETRIC_FUNCTION
        | class_ids::MERGE_RASTERS_FUNCTION => 2,
        class_ids::CACHED_RASTER_FUNCTION
        | class_ids::RGB32_FUNCTION
        | class_ids::RASTER_INFO_FUNCTION
        | class_ids::LOCAL_FUNCTION => 1,
        _ if class_ids::FUNCTIONS.contains(&class_id) => 1,
        _ => return Err(unsupported_class("IRasterFunction", &class_id)),
    };
    let has_basic_function = class_id != class_ids::RGB32_FUNCTION;
    let (base_version, base_strings, base_long) = has_basic_function
        .then(|| read_function_base(r, max))
        .transpose()?
        .unwrap_or_default();
    let version = version(r, max_version, "栅格函数派生体")?;
    let cache_strings = (class_id == class_ids::CACHED_RASTER_FUNCTION)
        .then(|| Ok::<_, io::Error>([read_string_10_1(r, max)?, read_string_10_1(r, max)?]))
        .transpose()?;
    let merge = (class_id == class_ids::MERGE_RASTERS_FUNCTION && version == 2)
        .then(|| merge::read_payload(r, max))
        .transpose()?;
    let raster_info = (class_id == class_ids::RASTER_INFO_FUNCTION)
        .then(|| raster_info::read_slot(r, max, refs))
        .transpose()?;
    let mosaic = (class_id == class_ids::MOSAIC_FUNCTION)
        .then(|| mosaic::read_payload(r, max, refs, version).map(Box::new))
        .transpose()?;
    let template = (class_id == class_ids::RASTER_FUNCTION_TEMPLATE)
        .then(|| template::read_template(r, max, refs, version, child_depth).map(Box::new))
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
        local_function_value,
    })
}

fn read_function_base(
    r: &mut Reader<'_>,
    max: usize,
) -> io::Result<(u16, [Option<String>; 2], i32)> {
    let version = version(r, 4, "BasicFunction")?;
    // v0/v1 只有 int32，v2 在 int32 后读一条字符串，v3/v4 先读两条字符串。
    let first_long = (version < 3).then(|| r.read_u32_le()).transpose()?;
    let strings = [
        (version >= 2)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
        (version >= 3)
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
    ) || class_ids::ARGUMENTS.contains(&class_id)
}

pub(super) fn read_arguments(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<RasterFunctionArgumentsBody10_1> {
    if !supports_function_arguments(class_id) {
        return Err(unsupported_class("IRasterFunctionArguments", &class_id));
    }
    let merge_version = (class_id == class_ids::MERGE_RASTERS_ARGUMENTS)
        .then(|| version(r, 2, "MergeRastersFunctionArguments"))
        .transpose()?;
    let base_version = version(r, 1, "BasicFunctionArguments")?;
    let properties = slot(r, refs, |r, refs, class_id, _| {
        if class_id != PROPERTY_SET_CLASS_ID {
            return Err(unsupported_class("IPropertySet", &class_id));
        }
        read_property_set_object_stream_10_1(r, max, refs)
    })?;
    let merge = merge_version
        .map(|version| merge::read_arguments(r, max, version))
        .transpose()?;
    let local_arguments_version = (class_id == class_ids::LOCAL_FUNCTION_ARGUMENTS)
        .then(|| version(r, 1, "LocalFunctionArgumentsImpl"))
        .transpose()?;
    Ok(RasterFunctionArgumentsBody10_1 {
        version: base_version,
        properties,
        merge,
        local_arguments_version,
    })
}

pub(crate) fn read_rendering_rule_10_1(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RenderingRule10_1> {
    let version = version(r, 1, "RenderingRule")?;
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
        read_arguments(r, max, refs, class_id)
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
            merge::read_payload(&mut reader, 4096).unwrap(),
            MergeRastersPayload10_1::UnresolvedRasterInfo(MergeRasterInput10_1 {
                entry: ObjectEntry10_1::NewObject { class_id, key: 12 },
                remaining: vec![8, 0, 0xff],
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
        info.extend([0; 2 * 8 + 2 * 4 + 16 + 4 * 4 + 2]);
        info.extend([0; 16]); // v2 的点。
        info.extend(EMPTY_STRING);
        info.extend(2_i32.to_le_bytes());
        let statistics = [
            0x83, 0xeb, 0x46, 0x84, 0x51, 0x32, 0x20, 0x4f, 0x8b, 0x79, 0x1d, 0x54, 0x21, 0xe3,
            0x9d, 0x4a,
        ];
        info.extend(statistics);
        info.extend(15_u32.to_le_bytes());
        info.extend(2_u16.to_le_bytes());
        info.extend([0; 6 * 8 + 2 * 4 + 2 * 16]);
        info.extend(statistics);
        info.extend(15_u32.to_le_bytes());
        info.extend(1_i32.to_le_bytes());
        info.extend([
            0xea, 0x93, 0x12, 0xb0, 0xd0, 0x88, 0x7d, 0x4e, 0xaf, 0x82, 0xa0, 0xb1, 0x0b, 0xac,
            0x9b, 0x33,
        ]);
        info.extend(16_u32.to_le_bytes());
        info.extend(1_u16.to_le_bytes());
        info.extend([0; 16 + 3 * 8 + 4]);
        info.extend(1_i32.to_le_bytes());
        info.extend(8.25_f64.to_le_bytes());
        info.extend(raster_info::RASTER_COLORMAP);
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
        assert_eq!(body.statistics[0].body.as_ref().unwrap().version, 2);
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
        assert!((13..=17).all(|key| refs.contains(key)));
        assert_eq!(reader.remaining(), 0);

        info.extend(1_i32.to_le_bytes());
        let rectangle = [1.0_f64, 2.0, 3.0, 4.0];
        info.extend(rectangle.into_iter().flat_map(f64::to_le_bytes));
        let mut compressed = Vec::new();
        push_compressed_variant(&mut compressed, &info);
        let mut reader = Reader::new(&compressed);
        let MergeRastersPayload10_1::RasterInfo {
            raster_info,
            rectangles,
        } = merge::read_payload(&mut reader, 4096).unwrap()
        else {
            panic!("应读取非空 RasterInfo");
        };
        assert_eq!(raster_info.body, Some(body));
        assert_eq!(rectangles, [rectangle]);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn mosaic_function_reads_lods_fids_and_versioned_tail() {
        let lod_class = [
            0x14, 0x5e, 0xf1, 0xe3, 0x2a, 0x51, 0xaf, 0x49, 0x80, 0x78, 0xef, 0x31, 0x81, 0x3d,
            0x6e, 0xab,
        ];
        for version in [1_u16, 2, 3, 4, 5, 6, 7, 8, 9] {
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
            let mosaic = rule.function.body.unwrap().mosaic.unwrap();
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
        for (function, arguments, local) in [
            (las, las_arguments, false),
            (
                class_ids::LOCAL_FUNCTION,
                class_ids::LOCAL_FUNCTION_ARGUMENTS,
                true,
            ),
        ] {
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
