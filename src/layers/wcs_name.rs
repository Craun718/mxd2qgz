//! OGCClient.dll 10.1 的 WCS 覆盖名称及 OGC 连接名称。

use super::image_server_rules::{RuleSlot10_1, slot, unsupported, unsupported_class};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::{native_string::read_bstr_string_10_1, reader::Reader};
use std::io;

pub const WCS_COVERAGE_NAME_CLASS_ID: [u8; 16] = [
    0xcf, 0x67, 0xa4, 0xc6, 0x2d, 0x26, 0xf0, 0x47, 0xbf, 0x56, 0xdd, 0x3a, 0x29, 0xc5, 0x6b, 0x5f,
];

pub const WCS_CONNECTION_NAME_CLASS_ID: [u8; 16] = [
    0x9e, 0xfe, 0x6c, 0xb9, 0xc7, 0x44, 0x16, 0x46, 0x8a, 0x2a, 0xf1, 0xfb, 0xa8, 0x64, 0x2a, 0xf6,
];

pub const WMS_CONNECTION_NAME_CLASS_ID: [u8; 16] = [
    0x97, 0xe9, 0xb0, 0x98, 0x1d, 0xf2, 0x95, 0x41, 0x8e, 0x06, 0xf9, 0xcd, 0x2a, 0xd9, 0x71, 0x65,
];

pub const WMTS_CONNECTION_NAME_CLASS_ID: [u8; 16] = [
    0xbb, 0x61, 0xb0, 0x93, 0x92, 0xdb, 0x90, 0x4e, 0x99, 0x27, 0x48, 0xc0, 0x31, 0xb3, 0x3f, 0xe8,
];

#[derive(Debug, Clone, PartialEq)]
pub struct WcsCoverageName10_1 {
    pub version: i16,
    pub identifier: String,
    pub connection: RuleSlot10_1<WcsConnectionName10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WcsConnectionName10_1 {
    pub version: i16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub extra_properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OgcConnectionName10_1 {
    pub class_id: [u8; 16],
    pub version: i16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub extra_properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
}

/// 原生从 IName 读短版本、覆盖标识及连接槽。调用者暂存宿主游标和对象表。
pub(super) fn read_wcs_coverage_name(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<WcsCoverageName10_1> {
    let version = r.read_u16_le()? as i16;
    if version > 1 {
        return Err(unsupported(format!("WCSCoverageName 未支持版本 {version}")));
    }
    let identifier = read_bstr_string_10_1(r, max)?;
    let connection = slot(r, refs, |r, refs, class_id, _| {
        if class_id != WCS_CONNECTION_NAME_CLASS_ID {
            return Err(unsupported_class("IWCSConnectionName", &class_id));
        }
        read_wcs_connection_name(r, max, refs)
    })?;
    Ok(WcsCoverageName10_1 {
        version,
        identifier,
        connection,
    })
}

fn read_wcs_connection_name(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<WcsConnectionName10_1> {
    let (version, properties, extra_properties) =
        read_connection_name_body(r, max, refs, "WCSConnectionName")?;
    Ok(WcsConnectionName10_1 {
        version,
        properties,
        extra_properties,
    })
}

/// WCS、WMS 与 WMTS 连接名共用 OGCClient.dll `0x7b420ee0`。宿主根据对象头
/// 类标识选择端类型，连接体本身只保存版本及一个或两个 IPropertySet 槽。
pub(super) fn read_ogc_connection_name(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<OgcConnectionName10_1> {
    let label = match class_id {
        WMS_CONNECTION_NAME_CLASS_ID => "WMSConnectionName",
        WMTS_CONNECTION_NAME_CLASS_ID => "WMTSConnectionName",
        _ => {
            return Err(unsupported_class("OGC 连接名称", &class_id));
        }
    };
    let (version, properties, extra_properties) = read_connection_name_body(r, max, refs, label)?;
    Ok(OgcConnectionName10_1 {
        class_id,
        version,
        properties,
        extra_properties,
    })
}

fn read_connection_name_body(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    label: &str,
) -> io::Result<(
    i16,
    RuleSlot10_1<PropertySetObjectStream10_1>,
    Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
)> {
    let version = r.read_u16_le()? as i16;
    if version > 3 {
        return Err(unsupported(format!("{label} 未支持版本 {version}")));
    }
    let mut read_properties = || {
        slot(r, refs, |r, refs, class_id, _| {
            if class_id != PROPERTY_SET_CLASS_ID {
                return Err(unsupported_class(
                    &format!("{label} IPropertySet"),
                    &class_id,
                ));
            }
            read_property_set_object_stream_10_1(r, max, refs)
        })
    };
    let properties = read_properties()?;
    let extra_properties = (version >= 2).then(read_properties).transpose()?;
    Ok((version, properties, extra_properties))
}
