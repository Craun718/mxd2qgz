//! RouteEvents.dll 10.1 的路线事件源名称、定位器名称和度量属性。

use super::dataset_name_more::{
    DatasetNameMoreSlot10_1, DatasetNameMoreStop10_1, check_signed_version,
    read_tabular_name_slot_10_1,
};
use crate::storage::class_id_hex;
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const ROUTE_EVENT_SOURCE_NAME_CLASS_ID: [u8; 16] = [
    0x74, 0x91, 0xbe, 0x63, 0xc7, 0xb8, 0xd3, 0x11, 0x9f, 0x7c, 0, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];
pub const ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID: [u8; 16] = [
    0xa6, 0x73, 0xc3, 0x8c, 0x21, 0x21, 0xd4, 0x11, 0x9f, 0xc2, 0, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];
pub const ROUTE_MEASURE_LINE_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xf0, 0xf2, 0xbd, 0x35, 0x21, 0x3b, 0xd4, 0x11, 0x9f, 0xcb, 0, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];
pub const ROUTE_MEASURE_POINT_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x37, 0x65, 0x3d, 0xc1, 0x80, 0x3c, 0xd4, 0x11, 0x9f, 0xcd, 0, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RouteEventSourceNameBody10_1 {
    pub key: u32,
    pub version: u16,
    pub strings: [String; 2],
    pub locator: ObjectEntry10_1,
    pub locator_body: Option<Box<RouteMeasureLocatorName10_1>>,
    pub event_table: Option<DatasetNameMoreSlot10_1>,
    pub properties: Option<ObjectEntry10_1>,
    pub properties_body: Option<RouteMeasureProperties10_1>,
    pub stop: Option<DatasetNameMoreStop10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteMeasureLocatorName10_1 {
    pub version: u16,
    pub base_version: u16,
    /// 位于 IName 槽之前的四个字符串。
    pub strings: [String; 4],
    pub route_name: DatasetNameMoreSlot10_1,
    pub suffix: Option<RouteMeasureLocatorSuffix10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMeasureLocatorSuffix10_1 {
    pub string_40: String,
    pub legacy_short: Option<u16>,
    pub long_44: i32,
    pub legacy_long_v2: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMeasureProperties10_1 {
    pub version: u16,
    pub strings: [String; 4],
    pub flag: u8,
    pub long_1c: i32,
    pub fields_v3: Option<(u16, String)>,
    pub short_v5: Option<u16>,
    /// 点属性在版本 1 之外读取的追加体。
    pub point_fields: Option<RouteMeasurePointFields10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMeasurePointFields10_1 {
    pub shorts: [u16; 2],
    pub string_48: String,
    pub short_4c: u16,
    pub short_v4: Option<u16>,
}

/// 外层事务涵盖名称、路线定位器和事件属性的所有新键。
pub(crate) fn read_route_event_source_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    key: u32,
) -> io::Result<RouteEventSourceNameBody10_1> {
    let mut r = reader.clone();
    let mut staged = refs.clone();
    let version = read_version(&mut r, 1, "RouteEventSourceName")?;
    let strings = read_strings(&mut r, max)?;
    let locator = read_object_entry_10_1(&mut r, &mut staged)?;
    let mut stop = None;
    let locator_body = match locator {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID,
            ..
        } => Some(Box::new(read_locator(&mut r, max, &mut staged, &mut stop)?)),
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(unsupported_object("IRouteLocatorName", &class_id, key));
        }
    };
    let event_table = stop
        .is_none()
        .then(|| read_tabular_name_slot_10_1(&mut r, max, &mut staged, &mut stop))
        .transpose()?;
    let properties = stop
        .is_none()
        .then(|| read_object_entry_10_1(&mut r, &mut staged))
        .transpose()?;
    let properties_body = match properties {
        Some(ObjectEntry10_1::NewObject {
            class_id: ROUTE_MEASURE_LINE_PROPERTIES_CLASS_ID,
            ..
        }) => Some(read_measure_properties(&mut r, max, false)?),
        Some(ObjectEntry10_1::NewObject {
            class_id: ROUTE_MEASURE_POINT_PROPERTIES_CLASS_ID,
            ..
        }) => Some(read_measure_properties(&mut r, max, true)?),
        Some(ObjectEntry10_1::NewObject { class_id, key }) => {
            return Err(unsupported_object("IRouteEventProperties", &class_id, key));
        }
        _ => None,
    };
    *reader = r;
    *refs = staged;
    Ok(RouteEventSourceNameBody10_1 {
        key,
        version,
        strings,
        locator,
        locator_body,
        event_table,
        properties,
        properties_body,
        stop,
    })
}

fn read_locator(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    stop: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<RouteMeasureLocatorName10_1> {
    let version = read_version(r, 1, "RouteMeasureLocatorName")?;
    let base_version = read_version(r, 4, "RouteLocatorNameBase")?;
    let strings = read_strings(r, max)?;
    let route_name = read_tabular_name_slot_10_1(r, max, refs, stop)?;
    let suffix = stop
        .is_none()
        .then(|| -> io::Result<_> {
            Ok(RouteMeasureLocatorSuffix10_1 {
                string_40: read_string_10_1(r, max)?,
                legacy_short: (base_version < 4).then(|| r.read_u16_le()).transpose()?,
                long_44: r.read_u32_le()? as i32,
                legacy_long_v2: (base_version == 2)
                    .then(|| r.read_u32_le().map(|v| v as i32))
                    .transpose()?,
            })
        })
        .transpose()?;
    Ok(RouteMeasureLocatorName10_1 {
        version,
        base_version,
        strings,
        route_name,
        suffix,
    })
}

fn read_measure_properties(
    r: &mut Reader<'_>,
    max: usize,
    point: bool,
) -> io::Result<RouteMeasureProperties10_1> {
    let version = read_version(r, 5, "RouteMeasureProperties")?;
    let strings = read_strings(r, max)?;
    let flag = r.read_bytes(1)?[0];
    let long_1c = r.read_u32_le()? as i32;
    let fields_v3 = (version >= 3)
        .then(|| -> io::Result<_> { Ok((r.read_u16_le()?, read_string_10_1(r, max)?)) })
        .transpose()?;
    let short_v5 = (version >= 5).then(|| r.read_u16_le()).transpose()?;
    let point_fields = (point && version != 1)
        .then(|| -> io::Result<_> {
            Ok(RouteMeasurePointFields10_1 {
                shorts: [r.read_u16_le()?, r.read_u16_le()?],
                string_48: read_string_10_1(r, max)?,
                short_4c: r.read_u16_le()?,
                short_v4: (version >= 4).then(|| r.read_u16_le()).transpose()?,
            })
        })
        .transpose()?;
    Ok(RouteMeasureProperties10_1 {
        version,
        strings,
        flag,
        long_1c,
        fields_v3,
        short_v5,
        point_fields,
    })
}

fn read_strings<const N: usize>(r: &mut Reader<'_>, max: usize) -> io::Result<[String; N]> {
    let mut strings = std::array::from_fn(|_| String::new());
    for value in &mut strings {
        *value = read_string_10_1(r, max)?;
    }
    Ok(strings)
}

fn read_version(r: &mut Reader<'_>, max: i16, class: &str) -> io::Result<u16> {
    let offset = r.position();
    let version = r.read_u16_le()?;
    check_signed_version(version, max, class, offset)?;
    Ok(version)
}

fn unsupported_object(interface: &str, class_id: &[u8; 16], key: u32) -> io::Error {
    io::Error::new(
        ErrorKind::Unsupported,
        format!(
            "{interface} 槽的未知类 {}（键 {key}）",
            class_id_hex(class_id)
        ),
    )
}
