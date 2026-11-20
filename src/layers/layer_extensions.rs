//! LayerLib 10.1 ExtensionHelper 的旧式图层扩展列表。

use super::scene_layer_properties::{
    SceneLayerProperties10_1, is_scene_layer_properties, read_bounded_scene_layer_properties,
};

use super::hatch_layer_extension::{
    HATCH_LAYER_EXTENSION_CLASS_ID, HatchLayerExtension10_1, read_bounded_hatch_layer_extension,
};
use super::image_server_rules::{RuleSlot10_1, slot, unsupported_class};
use super::route_layer_extension::{
    ROUTE_LAYER_EXTENSION_CLASS_ID, RouteLayerExtension10_1, read_bounded_route_layer_extension,
};
use super::time_layer_track_extension::{
    TIME_LAYER_TRACK_EXTENSION_CLASS_ID, TimeLayerTrackExtension10_1,
    read_bounded_time_layer_track_extension,
};
use super::tracking_filter_layer_extension::{
    TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID, TrackingFilterLayerExtension10_1,
    read_bounded_tracking_filter_layer_extension,
};
use crate::layers::annotate_props::{
    EDIT_TEMPLATE_MANAGER_CLASS_ID, EditTemplateManagerBody10_1,
    read_edit_template_manager_body_10_1,
};
use crate::layers::globe_layer_properties::{
    GLOBE_LAYER_PROPERTIES_CLASS_ID, GlobeLayerProperties10_1, read_bounded_globe_layer_properties,
};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_tabular_name_slot_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::{read_bstr_string_10_1, read_string_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const SERVER_LAYER_EXTENSION_CLASS_ID: [u8; 16] = [
    0xc9, 0x0e, 0xfd, 0xf7, 0x15, 0xf2, 0x7f, 0x4e, 0xb1, 0xf8, 0x94, 0xba, 0x0e, 0xb6, 0x03, 0xb9,
];
pub const DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xd9, 0x35, 0x23, 0x7e, 0x8d, 0x1e, 0x88, 0x44, 0x9e, 0x1e, 0x88, 0x85, 0x57, 0xc3, 0x1b, 0x3d,
];
pub const FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID: [u8; 16] = [
    0x48, 0xda, 0x2b, 0x9a, 0x26, 0xad, 0x06, 0x47, 0xbb, 0x88, 0xf4, 0x7d, 0x7b, 0x0c, 0x8f, 0x32,
];
pub const REPRESENTATION_LAYER_EXTENSION_CLASS_ID: [u8; 16] = [
    0x91, 0x37, 0x7c, 0xde, 0xe4, 0x49, 0xea, 0x48, 0xbf, 0xe3, 0x73, 0x63, 0xdd, 0x47, 0xcd, 0x49,
];
pub const E2_LAYER_EXTENSION_VALUES_CLASS_ID: [u8; 16] = [
    0x6b, 0x46, 0x8c, 0x0e, 0xd5, 0x2e, 0x59, 0x4b, 0x8b, 0x51, 0x7f, 0x75, 0x41, 0xf3, 0xff, 0x52,
];
const E2_LAYER_DESCRIPTION_VALUES_CLASS_ID: [u8; 16] = [
    0xfc, 0x95, 0x7e, 0xfd, 0xf8, 0xf4, 0x6f, 0x48, 0xa3, 0xde, 0xfb, 0xaf, 0x11, 0x14, 0x04, 0xf5,
];

/// NMFDocument.dll 10.1 中 E2LayerExtensionValues 的二进制字段。
#[derive(Debug, Clone, PartialEq)]
pub struct E2LayerExtensionValues10_1 {
    pub version: i16,
    pub description: ObjectEntry10_1,
    pub flag: u8,
}

pub(super) fn read_bounded_e2_layer_extension_values(
    bytes: &[u8],
    refs: &RefTable10_1,
) -> io::Result<E2LayerExtensionValues10_1> {
    let mut body = Reader::new(bytes);
    let version = body.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("E2LayerExtensionValues 未支持版本 {version}"),
        ));
    }
    // 唯一已对位的描述类提供 XML 接口。独立二进制对象体尚无证据。
    let description = read_object_entry_10_1(&mut body, &mut refs.clone())?;
    match description {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id: E2_LAYER_DESCRIPTION_VALUES_CLASS_ID,
            ..
        } => {}
        ObjectEntry10_1::NewObject { class_id, .. }
            if class_id == E2_LAYER_DESCRIPTION_VALUES_CLASS_ID =>
        {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "E2LayerDescriptionValues 尚无可读取的二进制对象体",
            ));
        }
        ObjectEntry10_1::BackReference { class_id, .. }
        | ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported_class(
                "E2LayerExtensionValues 描述对象",
                &class_id,
            ));
        }
    }
    let flag = body.read_bytes(1)?[0];
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "E2LayerExtensionValues 对象体消费与声明长度不一致",
        ));
    }
    Ok(E2LayerExtensionValues10_1 {
        version,
        description,
        flag,
    })
}

/// RepresentationLayer.dll 10.1 Load 0x7cb6aca0 只读一个版本 short。
pub(super) fn read_bounded_representation_layer_extension(bytes: &[u8]) -> io::Result<i16> {
    let mut body = Reader::new(bytes);
    let version = body.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("RepresentationLayerExtension 未支持版本 {version}"),
        ));
    }
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RepresentationLayerExtension 对象体消费与声明长度不一致",
        ));
    }
    Ok(version)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerLayerExtension10_1 {
    pub version: i16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicTiledLayerProperties10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub shorts: [u16; 2],
    pub value: f64,
    /// v2 起依次保存 LONG、short、LONG。
    pub version2_fields: Option<(u32, u16, u32)>,
    pub version3_short: Option<u16>,
}

pub(super) fn read_bounded_dynamic_tiled_layer_properties(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<DynamicTiledLayerProperties10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let result = read_dynamic_tiled_layer_properties(&mut body, max, &mut staged)?;
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DynamicTiledLayerProperties 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(result))
}

/// DynamicDisplay.dll 10.1 的 Load 0x6ef878d0。事务由宿主维护。
pub(super) fn read_dynamic_tiled_layer_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DynamicTiledLayerProperties10_1> {
    let version = r.read_u16_le()? as i16;
    if version > 3 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("DynamicTiledLayerProperties 未支持版本 {version}"),
        ));
    }
    let first = read_string_10_1(r, max)?;
    let properties = slot(r, refs, |r, refs, class, _| {
        if class != PROPERTY_SET_CLASS_ID {
            return Err(unsupported_class(
                "DynamicTiledLayerProperties IPropertySet",
                &class,
            ));
        }
        read_property_set_object_stream_10_1(r, max, refs)
    })?;
    Ok(DynamicTiledLayerProperties10_1 {
        version,
        strings: [first, read_string_10_1(r, max)?, read_string_10_1(r, max)?],
        properties,
        shorts: [r.read_u16_le()?, r.read_u16_le()?],
        value: r.read_f64_le()?,
        version2_fields: (version >= 2)
            .then(|| -> io::Result<_> {
                Ok((r.read_u32_le()?, r.read_u16_le()?, r.read_u32_le()?))
            })
            .transpose()?,
        version3_short: (version == 3).then(|| r.read_u16_le()).transpose()?,
    })
}

/// MapServer.dll 10.1 IPersistStream Load 0x79dcb620：版本及一个属性集槽。
pub(super) fn read_server_layer_extension(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<ServerLayerExtension10_1> {
    let version = r.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("ServerLayerExtension 未支持版本 {version}"),
        ));
    }
    let properties = slot(r, refs, |r, refs, class, _| {
        if class != PROPERTY_SET_CLASS_ID {
            return Err(unsupported_class(
                "ServerLayerExtension IPropertySet",
                &class,
            ));
        }
        read_property_set_object_stream_10_1(r, max, refs)
    })?;
    Ok(ServerLayerExtension10_1 {
        version,
        properties,
    })
}

pub(super) fn read_bounded_server_layer_extension(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<ServerLayerExtension10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let parsed = read_server_layer_extension(&mut body, max, &mut staged)?;
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ServerLayerExtension 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(parsed))
}

/// MapServerLayer.dll 10.1 的 FeatureServiceLayerEditExtension 二进制对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureServiceLayerEditExtension10_1 {
    pub version: i16,
    /// 原生依次读取的两个 IName 槽。
    pub names: [DatasetNameMoreSlot10_1; 2],
    pub text: String,
}

pub(super) fn read_bounded_feature_service_layer_edit_extension(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<FeatureServiceLayerEditExtension10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let result = read_feature_service_layer_edit_extension(&mut body, max, &mut staged)?;
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "FeatureServiceLayerEditExtension 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(result))
}

/// MapServerLayer.dll 10.1 的 Load 0x79ea9bd0。事务由宿主维护。
pub(super) fn read_feature_service_layer_edit_extension(
    body: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FeatureServiceLayerEditExtension10_1> {
    let version = body.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("FeatureServiceLayerEditExtension 未支持版本 {version}"),
        ));
    }
    let mut read_name = || {
        let mut stop = None;
        let name = read_tabular_name_slot_10_1(body, max, refs, &mut stop)?;
        if let Some(reason) = stop {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("FeatureServiceLayerEditExtension 的 IName 未读完：{reason:?}"),
            ));
        }
        Ok(name)
    };
    let names = [read_name()?, read_name()?];
    let text = read_bstr_string_10_1(body, max)?;
    Ok(FeatureServiceLayerEditExtension10_1 {
        version,
        names,
        text,
    })
}

/// ILayerExtensions 接受 IUnknown，PropertySet 可以直接作为扩展成员。
pub(super) fn read_bounded_extension_property_set(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<PropertySetObjectStream10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let parsed = read_property_set_object_stream_10_1(&mut body, max, &mut staged)?;
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "PropertySet 扩展消费长度与声明不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(parsed))
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerExtensionEntry10_1 {
    pub class_id: [u8; 16],
    /// 零长度条目没有对象体。
    pub template_manager: Option<EditTemplateManagerBody10_1>,
    pub properties: Option<Box<PropertySetObjectStream10_1>>,
    pub server_layer_extension: Option<Box<ServerLayerExtension10_1>>,
    pub representation_layer_extension_version: Option<i16>,
    pub route_layer_extension: Option<Box<RouteLayerExtension10_1>>,
    pub hatch_layer_extension: Option<Box<HatchLayerExtension10_1>>,
    pub tracking_filter_layer_extension: Option<Box<TrackingFilterLayerExtension10_1>>,
    pub e2_layer_extension_values: Option<E2LayerExtensionValues10_1>,
    pub feature_service_layer_edit_extension: Option<Box<FeatureServiceLayerEditExtension10_1>>,
    pub globe_layer_properties: Option<Box<GlobeLayerProperties10_1>>,
    pub time_layer_track_extension: Option<Box<TimeLayerTrackExtension10_1>>,
    pub dynamic_tiled_layer_properties: Option<Box<DynamicTiledLayerProperties10_1>>,
    pub scene_layer_properties: Option<Box<SceneLayerProperties10_1>>,
}

pub(super) fn read_layer_extensions_10_1(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Vec<LayerExtensionEntry10_1>> {
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    if count > r.remaining() / 24 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "图层扩展计数超出剩余字节",
        ));
    }
    let mut budget = max;
    let mut extensions = Vec::new();
    for _ in 0..count {
        extensions.push(read_layer_extension_entry_10_1(r, max, refs, &mut budget)?);
    }
    Ok(extensions)
}

pub(crate) fn read_layer_extension_entry_10_1(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    budget: &mut usize,
) -> io::Result<LayerExtensionEntry10_1> {
    // LayerLib ExtensionHelper 的长度覆盖对象体，CLSID 后没有键。
    let length = u64::from_le_bytes(r.read_bytes(8)?.try_into().unwrap());
    let class_id = r.read_bytes(16)?.try_into().unwrap();
    let length = usize::try_from(length)
        .ok()
        .filter(|length| *length <= *budget)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "图形容器扩展体长度超过限额"))?;
    let mut template_manager = None;
    let mut properties = None;
    let mut server_layer_extension = None;
    let mut representation_layer_extension_version = None;
    let mut route_layer_extension = None;
    let mut hatch_layer_extension = None;
    let mut tracking_filter_layer_extension = None;
    let mut e2_layer_extension_values = None;
    let mut feature_service_layer_edit_extension = None;
    let mut globe_layer_properties = None;
    let mut time_layer_track_extension = None;
    let mut dynamic_tiled_layer_properties = None;
    let mut scene_layer_properties = None;
    if length != 0 {
        match class_id {
            class if is_scene_layer_properties(class) => {
                scene_layer_properties = Some(read_bounded_scene_layer_properties(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID => {
                dynamic_tiled_layer_properties = Some(read_bounded_dynamic_tiled_layer_properties(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            GLOBE_LAYER_PROPERTIES_CLASS_ID => {
                globe_layer_properties = Some(read_bounded_globe_layer_properties(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            TIME_LAYER_TRACK_EXTENSION_CLASS_ID => {
                time_layer_track_extension = Some(read_bounded_time_layer_track_extension(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID => {
                feature_service_layer_edit_extension =
                    Some(read_bounded_feature_service_layer_edit_extension(
                        r.read_bytes(length)?,
                        max,
                        refs,
                    )?);
            }
            E2_LAYER_EXTENSION_VALUES_CLASS_ID => {
                e2_layer_extension_values = Some(read_bounded_e2_layer_extension_values(
                    r.read_bytes(length)?,
                    refs,
                )?);
            }
            TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID => {
                tracking_filter_layer_extension = Some(
                    read_bounded_tracking_filter_layer_extension(r.read_bytes(length)?, max, refs)?,
                );
            }
            HATCH_LAYER_EXTENSION_CLASS_ID => {
                hatch_layer_extension = Some(read_bounded_hatch_layer_extension(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            ROUTE_LAYER_EXTENSION_CLASS_ID => {
                route_layer_extension = Some(read_bounded_route_layer_extension(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            REPRESENTATION_LAYER_EXTENSION_CLASS_ID => {
                representation_layer_extension_version = Some(
                    read_bounded_representation_layer_extension(r.read_bytes(length)?)?,
                );
            }
            SERVER_LAYER_EXTENSION_CLASS_ID => {
                let bytes = r.read_bytes(length)?;
                server_layer_extension =
                    Some(read_bounded_server_layer_extension(bytes, max, refs)?);
            }
            PROPERTY_SET_CLASS_ID => {
                properties = Some(read_bounded_extension_property_set(
                    r.read_bytes(length)?,
                    max,
                    refs,
                )?);
            }
            EDIT_TEMPLATE_MANAGER_CLASS_ID => {
                let mut body = r.read_region(length)?;
                let parsed = read_edit_template_manager_body_10_1(&mut body, max, refs, 0)?;
                if let Some(reason) = &parsed.stop {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        format!("图形容器扩展体含未支持对象：{reason:?}"),
                    ));
                }
                if body.remaining() != 0 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "图形容器扩展体消费长度与声明不一致",
                    ));
                }
                template_manager = Some(parsed);
            }
            _ => return Err(unsupported_class("图层扩展", &class_id)),
        }
    }
    *budget -= length;
    Ok(LayerExtensionEntry10_1 {
        class_id,
        template_manager,
        properties,
        server_layer_extension,
        representation_layer_extension_version,
        route_layer_extension,
        hatch_layer_extension,
        tracking_filter_layer_extension,
        e2_layer_extension_values,
        feature_service_layer_edit_extension,
        globe_layer_properties,
        time_layer_track_extension,
        dynamic_tiled_layer_properties,
        scene_layer_properties,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::raster_layer_tail::read_raster_layer_tail_10_1;

    #[test]
    fn e2_extension_keeps_description_boundary_in_both_wrappers() {
        let class = E2_LAYER_EXTENSION_VALUES_CLASS_ID;
        let mut empty = 1_i16.to_le_bytes().to_vec();
        empty.extend([0; 16]);
        empty.push(1);
        let mut reference = 0_i16.to_le_bytes().to_vec();
        reference.extend(E2_LAYER_DESCRIPTION_VALUES_CLASS_ID);
        reference.extend(95_u32.to_le_bytes());
        reference.push(0);

        for version in [3, 8] {
            let build = |body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(1_i32.to_le_bytes());
                if version == 3 {
                    bytes.extend((body.len() as u64).to_le_bytes());
                    bytes.extend(class);
                    bytes.extend(body);
                } else {
                    bytes.extend((20 + body.len() as u32).to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                    bytes.extend(class);
                    bytes.extend(96_u32.to_le_bytes());
                    bytes.extend(body);
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            for (body, description, flag) in [
                (&empty, ObjectEntry10_1::Null, 1),
                (
                    &reference,
                    ObjectEntry10_1::BackReference {
                        class_id: E2_LAYER_DESCRIPTION_VALUES_CLASS_ID,
                        key: 95,
                    },
                    0,
                ),
            ] {
                let bytes = build(body);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                refs.register(95);
                let tail =
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
                let parsed = if version == 3 {
                    let mut legacy = Reader::new(&bytes[26..]);
                    let shared = read_layer_extensions_10_1(&mut legacy, 4096, &mut refs).unwrap();
                    assert_eq!(legacy.remaining(), 0);
                    assert_eq!(refs.len(), 1);
                    let entry = &tail.legacy_extensions[0].e2_layer_extension_values;
                    assert_eq!(entry, &shared[0].e2_layer_extension_values);
                    entry
                } else {
                    assert!(refs.contains(96));
                    assert_eq!(refs.len(), 2);
                    &tail.extra_safe_extensions[0].e2_layer_extension_values
                }
                .as_ref()
                .unwrap();
                assert_eq!((parsed.description, parsed.flag), (description, flag));
                assert_eq!(reader.remaining(), 0);
            }
            let mut wrong_version = empty.clone();
            wrong_version[0] = 2;
            let mut padded = empty.clone();
            padded.push(0);
            for body in [
                &empty[..empty.len() - 1],
                &reference,
                &wrong_version,
                &padded,
            ] {
                let bytes = build(body);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new(); // 描述键 95 尚未出现。
                let result = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs);
                assert!(result.is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }
}
