//! ImageServer.dll 10.1 的二进制 MosaicRule / RenderingRule 对象体。

use crate::layers::dep_tail::{
    TIME_CLASS_ID, TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, TimeBody10_1, TimeExtentBody10_1,
    TimeReferenceBody10_1, read_time_body_10_1, read_time_extent_body_10_1,
    read_time_reference_body_10_1,
};
use crate::layers::feature_id_set::{
    FEATURE_ID_SET_CLASS_ID, FeatureIdSetBody10_1, read_feature_id_set_body_10_1,
};
use crate::layers::raster_renderer::{
    RasterRendererSlot10_1, read_raster_renderer_slot_prefix_10_1,
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
    pub function: RuleSlot10_1<IdentityFunction10_1>,
    pub arguments: RuleSlot10_1<CachedRasterFunctionArguments10_1>,
    pub renderer: RasterRendererSlot10_1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityFunction10_1 {
    pub base_version: u16,
    pub version: u16,
    /// 基础版本 ≥2 和 ≥3 分别出现的两条字符串。
    pub base_strings: [Option<String>; 2],
    pub base_long: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CachedRasterFunctionArguments10_1 {
    pub version: u16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
}

fn unsupported(reason: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, reason.into())
}

fn unsupported_class(label: &str, id: &[u8; 16]) -> io::Error {
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
fn slot<'a, T>(
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
            slot(r, refs, |r, refs, class_id, key| {
                if class_id != FEATURE_ID_SET_CLASS_ID {
                    return Err(unsupported_class("IFIDSet", &class_id));
                }
                read_feature_id_set_body_10_1(r, refs, key)
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

fn read_function(
    r: &mut Reader<'_>,
    max: usize,
    class_id: [u8; 16],
) -> io::Result<IdentityFunction10_1> {
    if class_id != IDENTITY_FUNCTION_CLASS_ID {
        return Err(unsupported_class("IRasterFunction", &class_id));
    }
    let base_version = version(r, 4, "BasicFunction")?;
    // RasterCoreLib::BasicFunction::Load 的 int32 在 v0–2 的两串之前、
    // v3–4 的两串之后；v0/v1 无字符串。
    let first_long = (base_version < 3).then(|| r.read_u32_le()).transpose()?;
    let base_strings = [
        (base_version >= 2)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
        (base_version >= 3)
            .then(|| read_string_10_1(r, max))
            .transpose()?,
    ];
    let base_long = match first_long {
        Some(value) => value,
        None => r.read_u32_le()?,
    } as i32;
    let version = version(r, 1, "IdentityFunction")?;
    Ok(IdentityFunction10_1 {
        base_version,
        version,
        base_strings,
        base_long,
    })
}

fn read_arguments(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<CachedRasterFunctionArguments10_1> {
    if class_id != CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID {
        return Err(unsupported_class("IRasterFunctionArguments", &class_id));
    }
    let version = version(r, 1, "BasicFunctionArguments")?;
    let properties = slot(r, refs, |r, refs, class_id, _| {
        if class_id != PROPERTY_SET_CLASS_ID {
            return Err(unsupported_class("IPropertySet", &class_id));
        }
        read_property_set_object_stream_10_1(r, max, refs)
    })?;
    Ok(CachedRasterFunctionArguments10_1 {
        version,
        properties,
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
    let function = slot(r, refs, |r, _, class_id, _| read_function(r, max, class_id))?;
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
