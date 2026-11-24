//! ArcMap 10.1 GraphicsLayer3D 与 GlobeGraphicsLayer 对象体。
//! 字段序与版本门见 docs/arcmap-10.1-dep-tail.md「三维图形容器的对象体」。

pub use crate::layers::layer_extensions::LayerExtensionEntry10_1 as Graphics3dExtension10_1;
use crate::layers::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use crate::layers::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
use crate::maps::group_element::{
    GROUP_ELEMENT_CLASS_ID, GroupElement10_1, read_group_element_10_1,
};
use crate::maps::map_frame::MapFrame10_1;
use crate::maps::page_layout_prefix::{MAP_FRAME_CLASS_ID, read_map_frame};
use crate::maps::rectangle_element::{
    CIRCLE_ELEMENT_CLASS_ID, ELLIPSE_ELEMENT_CLASS_ID, RECTANGLE_ELEMENT_CLASS_ID,
    RectShapeElement10_1, RectangleElement10_1, read_rect_shape_element_10_1,
    read_rectangle_element_10_1,
};
use crate::maps::surround_frame::{
    MAP_SURROUND_FRAME_CLASS_ID, MapSurroundFrame10_1, read_surround_frame_10_1,
};
use crate::objects::element_body::{ElementBody10_1, read_element_body_of_class_10_1};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_entry_10_1,
};
use crate::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID,
    POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID, read_keyed_geometry_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const GRAPHICS_LAYER_3D_CLASS_ID: [u8; 16] = [
    0xf0, 0xa8, 0x87, 0x4c, 0xe3, 0x01, 0xd4, 0x11, 0xab, 0x69, 0x00, 0x08, 0xc7, 0x3f, 0xd5, 0x0c,
];
pub const GLOBE_GRAPHICS_LAYER_CLASS_ID: [u8; 16] = [
    0x73, 0xa0, 0x15, 0xa4, 0x06, 0xcf, 0xdc, 0x4c, 0x86, 0x64, 0x43, 0x9d, 0x0d, 0x5e, 0x71, 0x69,
];

#[derive(Debug, Clone, PartialEq)]
pub struct Graphics3dObject10_1<T> {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Graphics3dElement10_1 {
    pub element: Graphics3dObject10_1<ElementBody10_1>,
    /// 矩形的新对象体；对象头仍保存在 element.entry。
    pub rectangle: Option<Box<RectangleElement10_1>>,
    /// 圆或椭圆的新对象体，类由 element.entry 区分。
    pub circle_or_ellipse: Option<Box<RectShapeElement10_1>>,
    /// 组的新对象体；对象头仍保存在 element.entry。
    pub group: Option<Box<GroupElement10_1>>,
    /// 地图框的新对象体；对象头仍保存在 element.entry。
    pub map_frame: Option<Box<MapFrame10_1>>,
    /// 装饰框的新对象体；对象头仍保存在 element.entry。
    pub surround_frame: Option<Box<MapSurroundFrame10_1>>,
    pub globe_properties: Option<GlobeElementProperties10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobeElementProperties10_1 {
    /// 定宽属性的原始字节，按流中先后顺序保存。字节数由对象版本决定。
    pub raw: Vec<u8>,
    /// 版本 6 有一个几何槽，版本 7 有两个。
    pub geometries: Vec<Graphics3dObject10_1<GeometryBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Graphics3dTail10_1 {
    Scene {
        version_2_short: Option<i16>,
        spatial_reference: Option<SpatialReferenceEntry10_1>,
        extensions: Option<Vec<LayerExtensionEntry10_1>>,
        version_5_short: Option<i16>,
    },
    Globe {
        extensions: Vec<LayerExtensionEntry10_1>,
        shorts: [i16; 2],
        envelopes: Option<[Graphics3dObject10_1<Envelope10_1>; 2]>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Graphics3dLayer10_1 {
    pub version: u16,
    pub name: String,
    pub shorts: Vec<i16>,
    pub associated_layer: ObjectEntry10_1,
    pub associated_layer_body: Option<Box<LayerBodyPrefix10_1>>,
    pub envelope: Graphics3dObject10_1<Envelope10_1>,
    pub element_count: i32,
    pub elements: Vec<Graphics3dElement10_1>,
    pub tail: Graphics3dTail10_1,
}

fn unsupported(cls: &[u8; 16], what: &str) -> io::Error {
    io::Error::new(
        ErrorKind::Unsupported,
        format!("{what}：未支持的类 {}", crate::storage::class_id_hex(cls)),
    )
}

fn read_object<T>(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    read_body: impl FnOnce(&[u8; 16], u32, &mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<Graphics3dObject10_1<T>> {
    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::NewObject { class_id, key } => {
            Some(Box::new(read_body(&class_id, key, r, refs)?))
        }
        _ => None,
    };
    Ok(Graphics3dObject10_1 { entry, body })
}

fn read_envelope(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Graphics3dObject10_1<Envelope10_1>> {
    let mut envelope = read_object(r, refs, |cls, _, r, refs| {
        if *cls != ENVELOPE_CLASS_ID {
            return Err(unsupported(cls, "三维图形容器的 IEnvelope 槽"));
        }
        read_envelope_10_1(r, max, refs)
    })?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut envelope.entry
        && let Some(registered) = refs.registered_class(*key)
    {
        if registered != ENVELOPE_CLASS_ID {
            return Err(unsupported(&registered, "三维图形容器的 IEnvelope 回引"));
        }
        *class_id = registered;
    }
    Ok(envelope)
}

fn read_count(r: &mut Reader<'_>, min_entry_bytes: usize) -> io::Result<(i32, usize)> {
    let count = r.read_u32_le()? as i32;
    let iterations = count.max(0) as usize;
    if iterations > r.remaining() / min_entry_bytes {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!("三维图形容器计数 {count} 超出剩余字节"),
        ));
    }
    Ok((count, iterations))
}

fn read_globe_element_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    version: u16,
) -> io::Result<GlobeElementProperties10_1> {
    // 版本 2 起的七组 double×3 向量和其间的定宽标量。
    let flags_and_scales = 3 + 4 + usize::from(version >= 3) + 2 + 2 * 8;
    let vectors_and_flags = 1 + 24 + 4 + 4 + 1 + 3 * 24 + 2 + 24 + 1 + 2 * 24;
    let version_5_fields = if version >= 5 {
        2 * 8 + 4 * 4 + 3 * 2 + 4
    } else {
        0
    };
    let raw = r
        .read_bytes(flags_and_scales + vectors_and_flags + version_5_fields + 1 + 8)?
        .to_vec();
    let mut geometries = Vec::new();
    for _ in 6..=version {
        let mut geometry = read_object(r, refs, |cls, key, r, refs| {
            read_keyed_geometry_body_10_1(cls, key, r, max, refs)
        })?;
        if let ObjectEntry10_1::BackReference { class_id, key } = &mut geometry.entry
            && let Some(registered) = refs.registered_class(*key)
        {
            if !matches!(
                registered,
                ENVELOPE_CLASS_ID
                    | GEOMETRY_BAG_CLASS_ID
                    | MULTIPATCH_CLASS_ID
                    | MULTIPOINT_CLASS_ID
                    | POINT_CLASS_ID
                    | POLYGON_CLASS_ID
                    | POLYLINE_CLASS_ID
            ) {
                return Err(unsupported(
                    &registered,
                    "Globe 元素附加属性的 IGeometry 回引",
                ));
            }
            *class_id = registered;
        }
        geometries.push(geometry);
    }
    Ok(GlobeElementProperties10_1 { raw, geometries })
}

/// 在对象头后读取三维图形容器，只有完整读出时才提交游标和登记表。
/// 关联图层的新对象须完整解析。未知元素、几何或非空未知扩展按错误回滚。
pub fn read_graphics_3d_layer_10_1(
    class_id: &[u8; 16],
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<Graphics3dLayer10_1> {
    let (globe, max_version, short_count) = match *class_id {
        GRAPHICS_LAYER_3D_CLASS_ID => (false, 5, 4),
        GLOBE_GRAPHICS_LAYER_CLASS_ID => (true, 7, 2),
        _ => return Err(unsupported(class_id, "三维图形容器")),
    };
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()?;
    if !(1..=max_version).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("三维图形容器版本 {version} 超出 1..={max_version}"),
        ));
    }
    let name = read_string_10_1(&mut r, max)?;
    let shorts = (0..short_count)
        .map(|_| r.read_u16_le().map(|n| n as i16))
        .collect::<io::Result<_>>()?;
    let associated = read_object(&mut r, &mut refs, |cls, _, r, refs| {
        read_complete_layer_body_10_1(r, *cls, max, refs)
    })?;
    let envelope = read_envelope(&mut r, max, &mut refs)?;
    let (element_count, iterations) = read_count(&mut r, 16)?;
    let mut elements = Vec::new();
    for _ in 0..iterations {
        let header = read_header_10_1(&mut r.clone())?;
        let mut item = Graphics3dElement10_1 {
            element: Graphics3dObject10_1 {
                entry: ObjectEntry10_1::Null,
                body: None,
            },
            rectangle: None,
            circle_or_ellipse: None,
            group: None,
            map_frame: None,
            surround_frame: None,
            globe_properties: None,
        };
        if let ObjectHeader::Object { class_id, key } = header
            && !refs.contains(key)
        {
            let is_map = |key| table.contains_map(key);
            item.element.entry = ObjectEntry10_1::NewObject { class_id, key };
            match class_id {
                RECTANGLE_ELEMENT_CLASS_ID => {
                    item.rectangle = Some(Box::new(read_rectangle_element_10_1(
                        &mut r, max, &mut refs,
                    )?));
                }
                CIRCLE_ELEMENT_CLASS_ID | ELLIPSE_ELEMENT_CLASS_ID => {
                    item.circle_or_ellipse = Some(Box::new(read_rect_shape_element_10_1(
                        &mut r, max, &mut refs, class_id,
                    )?));
                }
                GROUP_ELEMENT_CLASS_ID => {
                    item.group = Some(Box::new(read_group_element_10_1(
                        &mut r, 0, max, &mut refs, &is_map,
                    )?));
                }
                MAP_FRAME_CLASS_ID => {
                    item.map_frame = Some(Box::new(read_map_frame(
                        &mut r, 0, max, &mut refs, &is_map,
                    )?));
                }
                MAP_SURROUND_FRAME_CLASS_ID => {
                    item.surround_frame = Some(Box::new(read_surround_frame_10_1(
                        &mut r, 0, max, &mut refs, &is_map,
                    )?));
                }
                _ => {
                    read_object_entry_10_1(&mut r, &mut refs)?;
                    let body = read_element_body_of_class_10_1(&class_id, &mut r, max, &mut refs)?;
                    if let Some(stop) = body.stop() {
                        return Err(io::Error::new(ErrorKind::Unsupported, stop.describe()));
                    }
                    item.element.body = Some(Box::new(body));
                }
            }
        } else {
            item.element.entry = read_object_entry_10_1(&mut r, &mut refs)?;
        }
        item.globe_properties = (globe && version >= 2)
            .then(|| read_globe_element_properties(&mut r, max, &mut refs, version))
            .transpose()?;
        elements.push(item);
    }
    let tail = if globe {
        let extensions = read_layer_extensions_10_1(&mut r, max, &mut refs)?;
        let shorts = [r.read_u16_le()? as i16, r.read_u16_le()? as i16];
        let envelopes = (version >= 4)
            .then(|| {
                Ok::<_, io::Error>([
                    read_envelope(&mut r, max, &mut refs)?,
                    read_envelope(&mut r, max, &mut refs)?,
                ])
            })
            .transpose()?;
        Graphics3dTail10_1::Globe {
            extensions,
            shorts,
            envelopes,
        }
    } else {
        Graphics3dTail10_1::Scene {
            version_2_short: (version >= 2)
                .then(|| r.read_u16_le().map(|n| n as i16))
                .transpose()?,
            spatial_reference: (version >= 3)
                .then(|| read_spatial_reference_entry_10_1(&mut r, max, &mut refs))
                .transpose()?,
            extensions: (version >= 4)
                .then(|| read_layer_extensions_10_1(&mut r, max, &mut refs))
                .transpose()?,
            version_5_short: (version >= 5)
                .then(|| r.read_u16_le().map(|n| n as i16))
                .transpose()?,
        }
    };
    *reader = r;
    *table = refs;
    Ok(Graphics3dLayer10_1 {
        version,
        name,
        shorts,
        associated_layer: associated.entry,
        associated_layer_body: associated.body,
        envelope,
        element_count,
        elements,
        tail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps::group_element::GroupChild10_1;
    use crate::maps::map_object::MAP_CLASS_ID;

    #[test]
    fn globe_element_geometry_references_check_registered_class() {
        let mut bytes = 6_u16.to_le_bytes().to_vec();
        bytes.extend([2, 0, 0, 0, 0, 0]);
        bytes.extend([0; 4 + 16 + 16]);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([0; 16 + 258]);
        bytes.extend(MAP_CLASS_ID);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([0; 8 + 32]);
        let end = bytes.len();
        bytes.extend([0xa5; 16]);
        for registered in [POINT_CLASS_ID, MAP_CLASS_ID] {
            let mut prior = registered.to_vec();
            prior.extend(1_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&prior), &mut refs).unwrap();
            let before = refs.clone();
            let mut reader = Reader::new(&bytes);
            let result = read_graphics_3d_layer_10_1(
                &GLOBE_GRAPHICS_LAYER_CLASS_ID,
                &mut reader,
                4096,
                &mut refs,
            );
            if registered == POINT_CLASS_ID {
                let layer = result.unwrap();
                let props = layer.elements[0].globe_properties.as_ref().unwrap();
                assert_eq!(
                    props.geometries[0].entry,
                    ObjectEntry10_1::BackReference {
                        class_id: POINT_CLASS_ID,
                        key: 1
                    }
                );
                assert!(props.geometries[0].body.is_none());
                assert_eq!(reader.position(), end);
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
            }
            assert_eq!(refs, before);
        }
    }

    #[test]
    fn graphics_3d_envelope_references_check_registered_class() {
        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for registered in [ENVELOPE_CLASS_ID, MAP_CLASS_ID] {
                let mut previous = registered.to_vec();
                previous.extend(1_u32.to_le_bytes());
                let mut refs = RefTable10_1::new();
                read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
                let before = refs.clone();
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.resize(bytes.len() + short_bytes + 16, 0);
                bytes.extend(MAP_CLASS_ID);
                bytes.extend(1_u32.to_le_bytes());
                bytes.resize(bytes.len() + 4 + tail_bytes, 0);
                let end = bytes.len();
                bytes.extend([0xa5; 16]);
                let mut reader = Reader::new(&bytes);
                let result = read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs);
                if registered == ENVELOPE_CLASS_ID {
                    let layer = result.unwrap();
                    assert_eq!(
                        layer.envelope.entry,
                        ObjectEntry10_1::BackReference {
                            class_id: ENVELOPE_CLASS_ID,
                            key: 1
                        }
                    );
                    assert!(layer.envelope.body.is_none());
                    assert_eq!(reader.position(), end);
                } else {
                    assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                    assert_eq!(reader.position(), 0);
                }
                assert_eq!(refs, before);
            }
        }
    }

    #[test]
    fn scene_group_resolves_only_confirmed_map_keys() {
        let properties = |bytes: &mut Vec<u8>| {
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(2)); // 两条空字符串。
            bytes.extend([0; 4 + 16]); // 旧标量和空自定义属性。
        };
        let base_frame = |bytes: &mut Vec<u8>| {
            bytes.extend([0; 2 + 2 + 16]); // 两段版本和空几何。
            properties(bytes);
            bytes.extend([0; 16 + 16 + 2]); // 空边框、背景和草稿模式。
        };
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend([2, 0, 0, 0, 0, 0]); // 容器名称。
        bytes.extend([0; 8 + 16 + 16]); // 定宽头、关联图层、包络。
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(GROUP_ELEMENT_CLASS_ID);
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        properties(&mut bytes);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(MAP_FRAME_CLASS_ID);
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(4_u16.to_le_bytes());
        bytes.extend(MAP_CLASS_ID);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([0; 8]); // 地图框的两个空集合。
        base_frame(&mut bytes);
        base_frame(&mut bytes);

        bytes.extend(MAP_FRAME_CLASS_ID);
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(4_u16.to_le_bytes());
        bytes.extend(MAP_CLASS_ID);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([0; 8]);
        base_frame(&mut bytes);

        let mut refs = RefTable10_1::new();
        refs.register(1);
        let mut reader = Reader::new(&bytes);
        let error =
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);

        refs.mark_map(1);
        let layer =
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .unwrap();
        let [GroupChild10_1::MapFrame(frame)] = layer.elements[0]
            .group
            .as_ref()
            .unwrap()
            .children
            .as_slice()
        else {
            panic!("预期组内地图框");
        };
        assert!(matches!(
            frame.prefix.map,
            ObjectEntry10_1::BackReference { key: 1, .. }
        ));
        let direct = layer.elements[1].map_frame.as_ref().unwrap();
        assert_eq!(direct.prefix.frame_key, 4);
        assert_eq!(direct.prefix.map, frame.prefix.map);
        assert!(refs.contains(2) && refs.contains(3) && refs.contains(4));
        assert_eq!(reader.remaining(), 0);
    }
}
