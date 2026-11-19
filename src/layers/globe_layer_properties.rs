//! GlobeLayers.dll 10.1 的 GlobeLayerProperties 和 GlobeHeightProperties。

use super::feature_id_set::{
    FEATURE_ID_SET_CLASS_ID, FeatureIdSetBody10_1, read_feature_id_set_body_10_1,
};
use super::image_server_rules::{RuleSlot10_1, slot, unsupported_class};
use super::layer_list::{
    LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_complete_layer_body_10_1,
};
use super::raster_layer::read_complete_bounded_raster_body_10_1;
use super::wcs_layer::{WCS_LAYER_CLASS_ID, read_wcs_layer_body_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const GLOBE_LAYER_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xea, 0x96, 0x0d, 0x2c, 0xaf, 0xca, 0xde, 0x4b, 0x84, 0x8c, 0x3f, 0xf8, 0x6b, 0xd9, 0xf5, 0x25,
];
const GLOBE_HEIGHT_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xdf, 0xcd, 0xfb, 0x69, 0xde, 0x43, 0x27, 0x42, 0x8b, 0x7c, 0xa1, 0x9c, 0xc6, 0x88, 0xb9, 0x13,
];

// 各项为（最低版本，固定字节数）。顺序对应 Load，版本 26 和 27 在较早字段之前插入。
const BEFORE_STRING_V28: &[(u16, usize)] = &[
    (1, 4),
    (2, 2),
    (3, 2),
    (4, 4),
    (5, 2),
    (6, 2),
    (7, 12),
    (8, 8),
    (9, 10),
    (10, 2),
    (11, 2),
    (13, 4),
    (14, 4),
    (15, 4),
    (17, 4),
    (18, 16),
    (20, 4),
    (21, 12),
    (22, 2),
    (26, 2),
    (23, 8),
    (27, 4),
    (24, 4),
];
const BEFORE_STRING_V35: &[(u16, usize)] = &[(29, 4), (30, 2), (31, 4), (32, 8), (33, 4), (34, 8)];
const BEFORE_STRING_V40: &[(u16, usize)] = &[(36, 4), (37, 2), (38, 8), (39, 2), (40, 4)];
const BEFORE_PROPERTY_SET: &[(u16, usize)] = &[(40, 40), (41, 4)];
const BEFORE_FEATURE_IDS: &[(u16, usize)] =
    &[(43, 28), (44, 2), (45, 2), (46, 12), (47, 12), (48, 192)];

#[derive(Debug, Clone, PartialEq)]
pub struct GlobeLayerProperties10_1 {
    pub version: u16,
    pub long_10: i32,
    pub path: String,
    pub height: RuleSlot10_1<GlobeHeightProperties10_1>,
    pub string_pair_flag: u16,
    pub string_pair: Option<[String; 2]>,
    pub string_v28: Option<String>,
    pub string_v35: Option<String>,
    pub string_v40: Option<String>,
    /// 其余标量按持久化顺序保存，包括原生 Load 会改写的旧版本位模式。
    pub scalar_bytes: Vec<u8>,
    pub property_set: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
    pub feature_ids: Option<RuleSlot10_1<FeatureIdSetBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobeHeightProperties10_1 {
    pub version: u16,
    pub mode: i32,
    pub string_0c: Option<String>,
    pub legacy_layer_name: Option<String>,
    pub layer: Option<RuleSlot10_1<LayerBodyPrefix10_1>>,
    pub double_24: f64,
    pub legacy_string: Option<String>,
    pub double_14: Option<f64>,
    pub short_2e: Option<u16>,
    pub short_2c: Option<u16>,
    pub fields_v4: Option<(i32, String)>,
    pub short_38: Option<u16>,
}

pub(super) fn read_bounded_globe_layer_properties(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<GlobeLayerProperties10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let version = read_version(&mut body, 48, "GlobeLayerProperties")?;
    let long_10 = body.read_u32_le()? as i32;
    let path = read_string_10_1(&mut body, max)?;
    let height = slot(&mut body, &mut staged, |r, refs, class_id, _| {
        if class_id != GLOBE_HEIGHT_PROPERTIES_CLASS_ID {
            return Err(unsupported_class("IGlobeHeightProperties", &class_id));
        }
        read_height_properties(r, max, refs)
    })?;
    let string_pair_flag = body.read_u16_le()?;
    let string_pair = (string_pair_flag != 0)
        .then(|| {
            Ok::<_, io::Error>([
                read_string_10_1(&mut body, max)?,
                read_string_10_1(&mut body, max)?,
            ])
        })
        .transpose()?;

    let mut scalar_bytes = Vec::new();
    let mut append_scalars = |body: &mut Reader<'_>, fields: &[(u16, usize)]| -> io::Result<()> {
        let len = fields
            .iter()
            .filter(|&&(min, _)| version >= min)
            .map(|&(_, len)| len)
            .sum();
        scalar_bytes.extend_from_slice(body.read_bytes(len)?);
        Ok(())
    };
    append_scalars(&mut body, BEFORE_STRING_V28)?;
    let string_v28 = (version >= 28)
        .then(|| read_string_10_1(&mut body, max))
        .transpose()?;
    append_scalars(&mut body, BEFORE_STRING_V35)?;
    let string_v35 = (version >= 35)
        .then(|| read_string_10_1(&mut body, max))
        .transpose()?;
    append_scalars(&mut body, BEFORE_STRING_V40)?;
    let string_v40 = (version >= 40)
        .then(|| read_string_10_1(&mut body, max))
        .transpose()?;
    append_scalars(&mut body, BEFORE_PROPERTY_SET)?;
    let property_set = (version >= 42)
        .then(|| {
            slot(&mut body, &mut staged, |r, refs, class_id, _| {
                if class_id != PROPERTY_SET_CLASS_ID {
                    return Err(unsupported_class("IPropertySet", &class_id));
                }
                read_property_set_object_stream_10_1(r, max, refs)
            })
        })
        .transpose()?;
    append_scalars(&mut body, BEFORE_FEATURE_IDS)?;
    let feature_ids = (version >= 48)
        .then(|| {
            slot(&mut body, &mut staged, |r, refs, class_id, key| {
                if class_id != FEATURE_ID_SET_CLASS_ID {
                    return Err(unsupported_class("IFeatureIDSet", &class_id));
                }
                read_feature_id_set_body_10_1(r, refs, key)
            })
        })
        .transpose()?;
    finish_body(&body, "GlobeLayerProperties")?;
    *refs = staged;
    Ok(Box::new(GlobeLayerProperties10_1 {
        version,
        long_10,
        path,
        height,
        string_pair_flag,
        string_pair,
        string_v28,
        string_v35,
        string_v40,
        scalar_bytes,
        property_set,
        feature_ids,
    }))
}

fn read_height_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<GlobeHeightProperties10_1> {
    let version = read_version(r, 7, "GlobeHeightProperties")?;
    let mode = r.read_u32_le()? as i32;
    let string_0c = (version >= 4 || mode == 1)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let legacy_layer_name = (mode == 2 && version < 5)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let layer = (mode == 2 && version >= 5)
        .then(|| read_height_layer(r, max, refs))
        .transpose()?;
    let double_24 = r.read_f64_le()?;
    let legacy_string = (version < 3)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let double_14 = (version >= 3).then(|| r.read_f64_le()).transpose()?;
    let short_2e = (version >= 2).then(|| r.read_u16_le()).transpose()?;
    let short_2c = (version >= 3).then(|| r.read_u16_le()).transpose()?;
    let fields_v4 = (version >= 4)
        .then(|| Ok::<_, io::Error>((r.read_u32_le()? as i32, read_string_10_1(r, max)?)))
        .transpose()?;
    let short_38 = (version >= 6).then(|| r.read_u16_le()).transpose()?;
    Ok(GlobeHeightProperties10_1 {
        version,
        mode,
        string_0c,
        legacy_layer_name,
        layer,
        double_24,
        legacy_string,
        double_14,
        short_2e,
        short_2c,
        fields_v4,
        short_38,
    })
}

fn read_height_layer(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<LayerBodyPrefix10_1>> {
    let a = r.read_u32_le()?;
    let b = r.read_u32_le()?;
    let len = if b == 0 { a } else { b } as usize;
    if len == 0 {
        return Ok(RuleSlot10_1 {
            entry: ObjectEntry10_1::Null,
            body: None,
        });
    }
    if len > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GlobeHeightProperties 图层范围超过限额",
        ));
    }
    let mut region = r.read_region(len)?;
    let layer = slot(&mut region, refs, |r, refs, class_id, _| match class_id {
        RASTER_LAYER_CLASS_ID => refs.with_nested_graphics(|refs| {
            read_complete_bounded_raster_body_10_1(r, max, refs)
                .map(LayerBodyPrefix10_1::RasterLayer)
        }),
        WCS_LAYER_CLASS_ID => refs.with_nested_graphics(|refs| {
            read_wcs_layer_body_10_1(r, max, refs, true)
                .map(|body| LayerBodyPrefix10_1::WcsLayer(Box::new(body)))
        }),
        _ => read_complete_layer_body_10_1(r, class_id, max, refs),
    })?;
    finish_body(&region, "GlobeHeightProperties 的 ExtraSafe ILayer")?;
    Ok(layer)
}

fn read_version(r: &mut Reader<'_>, max: u16, name: &str) -> io::Result<u16> {
    let version = r.read_u16_le()?;
    if !(1..=max).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("不支持的 {name} 版本 {version}"),
        ));
    }
    Ok(version)
}

fn finish_body(r: &Reader<'_>, name: &str) -> io::Result<()> {
    if r.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{name} 对象体消费与声明长度不一致"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::group_layer::GROUP_LAYER_CLASS_ID;
    use crate::layers::raster_layer_tail::read_raster_layer_tail_10_1;

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        let units: Vec<_> = value.encode_utf16().chain([0]).collect();
        bytes.extend(((units.len() * 2) as u32).to_le_bytes());
        bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
    }

    fn header(bytes: &mut Vec<u8>, class_id: [u8; 16], key: u32) {
        bytes.extend(class_id);
        bytes.extend(key.to_le_bytes());
    }

    fn properties(version: u16, height_version: u16, mode: i32, layer_prefix_b: bool) -> Vec<u8> {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(3_i32.to_le_bytes());
        push_string(&mut bytes, "路径");
        header(&mut bytes, GLOBE_HEIGHT_PROPERTIES_CLASS_ID, 91);
        bytes.extend(height_version.to_le_bytes());
        bytes.extend(mode.to_le_bytes());
        if mode == 1 || height_version >= 4 {
            push_string(&mut bytes, "高度");
        }
        if mode == 2 {
            if height_version < 5 {
                push_string(&mut bytes, "地表");
            } else {
                let mut group = Vec::new();
                header(&mut group, GROUP_LAYER_CLASS_ID, 92);
                group.extend(0_u16.to_le_bytes());
                push_string(&mut group, "地表图层");
                group.extend([0; 6 + 16 + 4]);
                let len = group.len() as u32;
                bytes.extend(if layer_prefix_b { 0 } else { len }.to_le_bytes());
                bytes.extend(if layer_prefix_b { len } else { 0 }.to_le_bytes());
                bytes.extend(group);
            }
        }
        bytes.extend(3.5_f64.to_le_bytes());
        if height_version < 3 {
            push_string(&mut bytes, "旧字段");
        } else {
            bytes.extend(7.0_f64.to_le_bytes());
        }
        if height_version >= 2 {
            bytes.extend(9_u16.to_le_bytes());
        }
        if height_version >= 3 {
            bytes.extend(10_u16.to_le_bytes());
        }
        if height_version >= 4 {
            bytes.extend(11_i32.to_le_bytes());
            push_string(&mut bytes, "第二表达式");
        }
        if height_version >= 6 {
            bytes.extend(12_u16.to_le_bytes());
        }

        bytes.extend(0xffff_u16.to_le_bytes());
        push_string(&mut bytes, "甲");
        push_string(&mut bytes, "乙");
        let lengths = match version {
            1 => [4, 0, 0, 0, 0],
            21 => [98, 0, 0, 0, 0],
            36 => [118, 30, 4, 0, 0],
            39 => [118, 30, 16, 0, 0],
            45 => [118, 30, 20, 44, 32],
            48 => [118, 30, 20, 44, 248],
            _ => unreachable!(),
        };
        bytes.extend(std::iter::repeat_n(1, lengths[0]));
        if version >= 28 {
            push_string(&mut bytes, "v28");
        }
        bytes.extend(std::iter::repeat_n(2, lengths[1]));
        if version >= 35 {
            push_string(&mut bytes, "v35");
        }
        bytes.extend(std::iter::repeat_n(3, lengths[2]));
        if version >= 40 {
            push_string(&mut bytes, "v40");
        }
        bytes.extend(std::iter::repeat_n(4, lengths[3]));
        if version >= 42 {
            header(&mut bytes, PROPERTY_SET_CLASS_ID, 93);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes.extend(std::iter::repeat_n(5, lengths[4]));
        if version >= 48 {
            header(&mut bytes, FEATURE_ID_SET_CLASS_ID, 94);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(71_i32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn globe_properties_version_boundaries_and_height_modes() {
        for (version, height_version, mode) in [
            (1, 1, 1),
            (21, 2, 0),
            (36, 3, 2),
            (39, 4, 2),
            (45, 6, 2),
            (48, 7, 2),
        ] {
            let bytes = properties(version, height_version, mode, height_version == 6);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_globe_layer_properties(&bytes, 4096, &mut refs).unwrap();
            assert_eq!(parsed.version, version);
            assert_eq!(parsed.path, "路径");
            assert_eq!(parsed.string_pair.unwrap(), ["甲", "乙"]);
            assert_eq!(
                parsed.string_v28.as_deref(),
                (version >= 28).then_some("v28")
            );
            assert_eq!(
                parsed.string_v35.as_deref(),
                (version >= 35).then_some("v35")
            );
            assert_eq!(
                parsed.string_v40.as_deref(),
                (version >= 40).then_some("v40")
            );
            let height = parsed.height.body.unwrap();
            assert_eq!(height.version, height_version);
            assert_eq!(height.mode, mode);
            assert_eq!(height.double_24, 3.5);
            assert_eq!(height.double_14, (height_version >= 3).then_some(7.0));
            assert_eq!(
                height.legacy_layer_name.as_deref(),
                (mode == 2 && height_version < 5).then_some("地表")
            );
            assert_eq!(height.short_38, (height_version >= 6).then_some(12));
            assert!(refs.contains(91));
            assert_eq!(refs.contains(92), height_version >= 5 && mode == 2);
            if let Some(layer) = height.layer {
                let Some(LayerBodyPrefix10_1::GroupLayer(body)) = layer.body.as_deref() else {
                    panic!("应完整解析有界地表图层");
                };
                assert_eq!(body.name, "地表图层");
            }
            assert_eq!(refs.contains(93), version >= 42);
            assert_eq!(refs.contains(94), version >= 48);
            if let Some(ids) = parsed.feature_ids {
                assert_eq!(ids.body.unwrap().ids, [71]);
            }
        }
    }

    #[test]
    fn globe_properties_raster_wrappers_and_transactional_truncation() {
        let body = properties(48, 7, 2, false);
        for raster_version in [3, 8] {
            let wrap = |body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(1_i32.to_le_bytes());
                let header_len = if raster_version == 8 { 20 } else { 0 };
                bytes.extend(((body.len() + header_len) as u64).to_le_bytes());
                bytes.extend(GLOBE_LAYER_PROPERTIES_CLASS_ID);
                if raster_version == 8 {
                    bytes.extend(90_u32.to_le_bytes());
                }
                bytes.extend(body);
                if raster_version == 8 {
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            let bytes = wrap(&body);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail =
                read_raster_layer_tail_10_1(&mut reader, raster_version, 4096, &mut refs).unwrap();
            let parsed = if raster_version == 3 {
                tail.legacy_extensions[0].globe_layer_properties.as_deref()
            } else {
                tail.extra_safe_extensions[0]
                    .globe_layer_properties
                    .as_deref()
            };
            assert_eq!(parsed.unwrap().version, 48);
            assert_eq!(refs.contains(90), raster_version == 8);
            assert!((91..=94).all(|key| refs.contains(key)));
            assert_eq!(reader.remaining(), 0);

            for truncated in [&body[..1], &body[..body.len() - 1]] {
                let bytes = wrap(truncated);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                refs.register(33);
                let saved = refs.clone();
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, raster_version, 4096, &mut refs)
                        .is_err()
                );
                assert_eq!(reader.position(), 0);
                assert_eq!(refs, saved);
            }
        }
    }
}
