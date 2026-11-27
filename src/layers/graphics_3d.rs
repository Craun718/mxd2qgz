//! ArcMap 10.1 GraphicsLayer3D 与 GlobeGraphicsLayer 对象体。
//! 字段序与版本门见 docs/arcmap-10.1-dep-tail.md「三维图形容器的对象体」。

pub use crate::layers::layer_extensions::LayerExtensionEntry10_1 as Graphics3dExtension10_1;
use crate::layers::layer_extensions::{
    LayerExtensionEntry10_1, read_layer_extensions_preserving_unknown_10_1,
};
use crate::layers::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
use crate::maps::group_element::{
    GROUP_ELEMENT_CLASS_ID, GroupElement10_1, read_group_element_10_1,
};
use crate::maps::layer_masking::{LayerMaskingSupport10_1, known_layer_masking_support_10_1};
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
/// 关联图层的新对象须完整解析。未知元素或几何按错误回滚；未知扩展按声明长度保留。
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
    let mut associated = read_object(&mut r, &mut refs, |cls, _, r, refs| {
        read_complete_layer_body_10_1(r, *cls, max, refs)
    })?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut associated.entry
        && let Some(registered) = refs.registered_class(*key)
    {
        // 遮罩能力表的已核对类均为 ILayer，实现遮罩接口并非此槽的条件。
        if known_layer_masking_support_10_1(&registered) == LayerMaskingSupport10_1::Unresolved {
            return Err(unsupported(&registered, "三维图形容器的 ILayer 回引"));
        }
        *class_id = registered;
    }
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
        let extensions = read_layer_extensions_preserving_unknown_10_1(&mut r, max, &mut refs)?;
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
                .then(|| read_layer_extensions_preserving_unknown_10_1(&mut r, max, &mut refs))
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
    use crate::maps::layer_masking::{DUMMY_GRADUATED_MARKER_LAYER_CLASS_ID, DUMMY_LAYER_CLASS_ID};
    use crate::maps::map_object::MAP_CLASS_ID;

    #[test]
    fn graphics_3d_associated_references_check_registered_class() {
        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for registered in [
                GRAPHICS_LAYER_3D_CLASS_ID,
                DUMMY_LAYER_CLASS_ID,
                DUMMY_GRADUATED_MARKER_LAYER_CLASS_ID,
                MAP_CLASS_ID,
            ] {
                let mut prior = registered.to_vec();
                prior.extend(1_u32.to_le_bytes());
                let mut refs = RefTable10_1::new();
                read_object_entry_10_1(&mut Reader::new(&prior), &mut refs).unwrap();
                let before = refs.clone();
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.resize(bytes.len() + short_bytes, 0);
                bytes.extend(MAP_CLASS_ID);
                bytes.extend(1_u32.to_le_bytes());
                bytes.resize(bytes.len() + 16 + 4 + tail_bytes, 0);
                let end = bytes.len();
                bytes.extend([0xa5; 16]);
                let mut reader = Reader::new(&bytes);
                let result = read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs);
                if registered != MAP_CLASS_ID {
                    let layer = result.unwrap();
                    assert_eq!(
                        layer.associated_layer,
                        ObjectEntry10_1::BackReference {
                            class_id: registered,
                            key: 1
                        }
                    );
                    assert!(layer.associated_layer_body.is_none());
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
    fn graphics_3d_reads_wms_map_layer_with_connection_and_versioned_tail() {
        use crate::layers::cache_tile_reader::INTERNET_TILED_LAYER_CLASS_ID;
        use crate::layers::wcs_name::WMS_CONNECTION_NAME_CLASS_ID;
        use crate::maps::layer_masking::WMS_MAP_LAYER_CLASS_ID;

        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            let empty = [2, 0, 0, 0, 0, 0];
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(empty);
            bytes.resize(bytes.len() + short_bytes, 0);
            bytes.extend(WMS_MAP_LAYER_CLASS_ID);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(6_i16.to_le_bytes());
            bytes.extend(WMS_CONNECTION_NAME_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend(empty);
            bytes.extend([0; 8 + 16 + 8 + 8 + 4 + 2]);
            bytes.extend(empty);
            bytes.extend([0; 32 + 4]);
            let tile_offset = bytes.len();
            bytes.extend([0; 16]);
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend(8_i16.to_le_bytes());
            bytes.extend(9.0_f64.to_le_bytes());
            bytes.extend(10_i32.to_le_bytes());
            bytes.extend(11.0_f64.to_le_bytes());
            bytes.extend(12_i32.to_le_bytes());
            let map_end = bytes.len();
            bytes.resize(map_end + 20 + tail_bytes, 0);
            let end = bytes.len();
            bytes.push(0xa5);

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let parsed = read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
            let Some(LayerBodyPrefix10_1::WmsMapLayer(layer)) =
                parsed.associated_layer_body.as_deref()
            else {
                panic!("应读取 WMSMapLayer");
            };
            assert_eq!(layer.connection.as_ref().unwrap().version, 1);
            assert_eq!(layer.integer_v4, Some(7));
            assert_eq!(layer.tail_v5, Some((8, 9.0, 10)));
            assert_eq!(layer.tail_v6, Some((11.0, 12)));
            assert!(refs.contains(1) && refs.contains(2));
            assert_eq!(reader.position(), end);

            let mut header = INTERNET_TILED_LAYER_CLASS_ID.to_vec();
            header.extend(3_u32.to_le_bytes());
            let mut new_tile = header.clone();
            new_tile.extend(1_i16.to_le_bytes());
            new_tile.extend(empty.repeat(2));
            new_tile.extend([0; 6 + 16 + 8 + 16 + 2 + 4 + 4 + 16 + 8 + 16 + 4 + 16]);
            for (is_new, slot) in [(true, new_tile), (false, header.clone())] {
                let mut variant = bytes.clone();
                let variant_end = end + slot.len() - 16;
                variant.splice(tile_offset..tile_offset + 16, slot);
                let mut refs = RefTable10_1::new();
                if !is_new {
                    read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
                }
                let mut reader = Reader::new(&variant);
                let parsed =
                    read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
                let Some(LayerBodyPrefix10_1::WmsMapLayer(layer)) =
                    parsed.associated_layer_body.as_deref()
                else {
                    panic!("应读取 WMSMapLayer");
                };
                let (entry, body) = layer.tile_layer.as_ref().unwrap();
                if is_new {
                    assert!(matches!(entry, ObjectEntry10_1::NewObject { key: 3, .. }));
                    assert!(matches!(body, Some(LayerBodyPrefix10_1::TiledLayer(_))));
                } else {
                    assert!(matches!(
                        entry,
                        ObjectEntry10_1::BackReference { key: 3, .. }
                    ));
                    assert!(body.is_none());
                }
                assert!(refs.contains(3));
                assert_eq!(layer.integer_v4, Some(7));
                assert_eq!(layer.tail_v6, Some((11.0, 12)));
                assert_eq!(reader.position(), variant_end);
            }

            let mut reader = Reader::new(&bytes[..map_end - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn graphics_3d_reads_wms_group_with_child_and_reference() {
        use crate::maps::layer_masking::{WMS_GROUP_LAYER_CLASS_ID, WMS_LAYER_CLASS_ID};

        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            let empty = [2, 0, 0, 0, 0, 0];
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(empty);
            bytes.resize(bytes.len() + short_bytes, 0);
            bytes.extend(WMS_GROUP_LAYER_CLASS_ID);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(empty);
            bytes.extend([0; 6 + 16]);
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(WMS_LAYER_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(empty);
            bytes.extend(empty);
            bytes.extend([0; 6]);
            bytes.extend(WMS_LAYER_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend([0; 2 + 16 + 8]);
            bytes.extend(empty);
            bytes.extend([0; 16]);
            let group_end = bytes.len();
            bytes.resize(group_end + 20 + tail_bytes, 0);
            let end = bytes.len();
            bytes.push(0xa5);

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let parsed = read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
            let Some(LayerBodyPrefix10_1::WmsGroupLayer(group)) =
                parsed.associated_layer_body.as_deref()
            else {
                panic!("应读取 WMSGroupLayer");
            };
            assert!(matches!(
                group.children[0].1.as_ref(),
                Some(LayerBodyPrefix10_1::WmsLayer(_))
            ));
            assert!(matches!(
                group.children[1].0,
                ObjectEntry10_1::BackReference { key: 2, .. }
            ));
            assert!(refs.contains(1) && refs.contains(2));
            assert_eq!(reader.position(), end);

            let mut reader = Reader::new(&bytes[..group_end - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn graphics_3d_reads_versioned_wms_layer_and_following_fields() {
        use crate::maps::layer_masking::WMS_LAYER_CLASS_ID;

        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for version in 1_i16..=5 {
                let empty = [2, 0, 0, 0, 0, 0];
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend(empty);
                bytes.resize(bytes.len() + short_bytes, 0);
                bytes.extend(WMS_LAYER_CLASS_ID);
                bytes.extend(1_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend([4, 0, 0, 0, b'W', 0, 0, 0]);
                bytes.extend(empty);
                bytes.extend([0; 6]);
                if version >= 2 {
                    bytes.extend(empty);
                }
                if version >= 3 {
                    bytes.extend([0; 2]);
                }
                if version >= 4 {
                    bytes.extend(empty);
                }
                if version >= 5 {
                    bytes.extend(2_i16.to_le_bytes());
                    bytes.extend(1.5_f64.to_le_bytes());
                    bytes.extend(7_i32.to_le_bytes());
                }
                let layer_end = bytes.len();
                bytes.resize(layer_end + 20 + tail_bytes, 0);
                let end = bytes.len();
                bytes.push(0xa5);

                let mut refs = RefTable10_1::new();
                let mut reader = Reader::new(&bytes);
                let parsed =
                    read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
                let Some(LayerBodyPrefix10_1::WmsLayer(layer)) =
                    parsed.associated_layer_body.as_deref()
                else {
                    panic!("应读取 WMSLayer");
                };
                assert_eq!(layer.name, "W");
                assert_eq!(layer.tail_v5, (version == 5).then_some((2, 1.5, 7)));
                assert!(refs.contains(1));
                assert_eq!(reader.position(), end);

                let mut reader = Reader::new(&bytes[..layer_end - 1]);
                let mut refs = RefTable10_1::new();
                assert!(read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn graphics_3d_reads_topology_layer_versions_and_successors() {
        use crate::maps::layer_masking::TOPOLOGY_LAYER_CLASS_ID;

        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for version in [-1_i16, 0, 1, 2, 3, 4, 5] {
                let empty_string = [2, 0, 0, 0, 0, 0];
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend(empty_string);
                bytes.resize(bytes.len() + short_bytes, 0);
                bytes.extend(TOPOLOGY_LAYER_CLASS_ID);
                bytes.extend(41_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend(empty_string);
                bytes.extend([0; 6 + 16 + 16]);
                let mut renderer_ref_class_at = None;
                if version != 1 {
                    use crate::layers::feature_layer_tail::SIMPLE_RENDERER_CLASS_ID;
                    use crate::objects::display_symbol::{
                        SIMPLE_FILL_SYMBOL_CLASS_ID, SIMPLE_LINE_SYMBOL_CLASS_ID,
                        SIMPLE_MARKER_SYMBOL_CLASS_ID,
                    };

                    for (key, symbol, body_len) in [
                        (42_u32, SIMPLE_FILL_SYMBOL_CLASS_ID, 44),
                        (46, SIMPLE_LINE_SYMBOL_CLASS_ID, 36),
                        (48, SIMPLE_MARKER_SYMBOL_CLASS_ID, 85),
                        (44, SIMPLE_FILL_SYMBOL_CLASS_ID, 44),
                    ] {
                        if key == 44 {
                            renderer_ref_class_at = Some(bytes.len());
                            bytes.extend(MAP_CLASS_ID);
                            bytes.extend(42_u32.to_le_bytes());
                            bytes.extend([0; 2 * 16]);
                        }
                        bytes.extend(SIMPLE_RENDERER_CLASS_ID);
                        bytes.extend(key.to_le_bytes());
                        bytes.extend(1_u16.to_le_bytes());
                        bytes.extend(symbol);
                        bytes.extend((key + 1).to_le_bytes());
                        bytes.extend(1_u16.to_le_bytes());
                        bytes.resize(bytes.len() + body_len, 0);
                        bytes.extend([0; 16 + 16 + 2]);
                        bytes.extend(empty_string);
                        bytes.extend([0; 4]);
                        bytes.extend(empty_string);
                    }
                }
                if !matches!(version, 1 | 2) {
                    bytes.extend([0; 16]);
                }
                if !matches!(version, 1..=3) {
                    bytes.extend(9_i16.to_le_bytes());
                }
                if !matches!(version, 1..=4) {
                    bytes.extend([0; 4]);
                    bytes.extend(1_i32.to_le_bytes());
                    bytes.extend(123_u32.to_le_bytes());
                    bytes.extend((-4_i16).to_le_bytes());
                }
                bytes.resize(bytes.len() + 16 + 4 + tail_bytes, 0);
                let end = bytes.len();
                bytes.extend([0xa5; 8]);

                let mut refs = RefTable10_1::new();
                for cut in 0..end {
                    let mut reader = Reader::new(&bytes[..cut]);
                    assert!(
                        read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).is_err()
                    );
                    assert_eq!(reader.position(), 0);
                    assert!(refs.is_empty());
                }
                let mut reader = Reader::new(&bytes);
                let layer =
                    read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
                let Some(LayerBodyPrefix10_1::TopologyLayer(body)) =
                    layer.associated_layer_body.as_deref()
                else {
                    panic!("应读取 TopologyLayer");
                };
                if version != 1 {
                    use crate::layers::feature_layer::RendererSlot10_1;
                    use crate::objects::display_symbol::{
                        SIMPLE_FILL_SYMBOL_CLASS_ID, SIMPLE_LINE_SYMBOL_CLASS_ID,
                        SIMPLE_MARKER_SYMBOL_CLASS_ID,
                    };
                    for (index, key, symbol) in [
                        (0, 42, SIMPLE_FILL_SYMBOL_CLASS_ID),
                        (1, 46, SIMPLE_LINE_SYMBOL_CLASS_ID),
                        (2, 48, SIMPLE_MARKER_SYMBOL_CLASS_ID),
                        (6, 44, SIMPLE_FILL_SYMBOL_CLASS_ID),
                    ] {
                        assert!(
                            matches!(&body.renderers[index], RendererSlot10_1::SimpleRenderer(renderer) if renderer.key == key && renderer.stop.is_none())
                        );
                        assert_eq!(refs.registered_class(key + 1), Some(symbol));
                    }
                }
                if let Some(offset) = renderer_ref_class_at {
                    use crate::layers::feature_layer::RendererSlot10_1;
                    use crate::layers::feature_layer_tail::SIMPLE_RENDERER_CLASS_ID;

                    assert_eq!(
                        body.renderers[3],
                        RendererSlot10_1::BackReference {
                            class_id: SIMPLE_RENDERER_CLASS_ID,
                            key: 42,
                        }
                    );
                    let mut bytes = bytes.clone();
                    bytes[offset..offset + 16].copy_from_slice(&SIMPLE_RENDERER_CLASS_ID);
                    bytes[offset + 16..offset + 20].copy_from_slice(&41_u32.to_le_bytes());
                    let mut reader = Reader::new(&bytes);
                    let mut refs = RefTable10_1::new();
                    let error = read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs)
                        .unwrap_err();
                    assert_eq!(error.kind(), ErrorKind::Unsupported);
                    assert_eq!(reader.position(), 0);
                    assert!(refs.is_empty());
                }
                assert_eq!(body.version, version);
                assert_eq!(body.renderers.len(), if version != 1 { 7 } else { 0 });
                assert_eq!(
                    body.legacy_object,
                    (!matches!(version, 1 | 2)).then_some(ObjectEntry10_1::Null)
                );
                assert_eq!(body.raw_count, (!matches!(version, 1..=4)).then_some(1));
                if !matches!(version, 1..=4) {
                    assert_eq!(body.tail_shorts, [9, 0, 0]);
                    assert_eq!(body.rows, [(123, -4)]);
                }
                assert_eq!(refs.registered_class(41), Some(TOPOLOGY_LAYER_CLASS_ID));
                assert_eq!(reader.position(), end);
            }
        }
    }

    #[test]
    fn graphics_3d_reads_dimension_layer_and_preserves_container_boundary() {
        use crate::maps::layer_masking::DIMENSION_LAYER_CLASS_ID;

        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for version in 1_i16..=4 {
                let empty_string = [2, 0, 0, 0, 0, 0];
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend(empty_string);
                bytes.extend(vec![0; short_bytes]);
                bytes.extend(DIMENSION_LAYER_CLASS_ID);
                bytes.extend(41_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(empty_string.repeat(2));
                bytes.extend([0; 6 + 16 + 16 + 16]);
                bytes.extend(empty_string);
                bytes.extend([0; 16 + 2 + 4 + 16]);
                bytes.extend(empty_string.repeat(2));
                bytes.extend([0; 8 + 16 + 2 + 16 + 4 + 2]);
                if version == 2 {
                    bytes.extend(empty_string);
                    bytes.extend((-7_i16).to_le_bytes());
                }
                if version == 4 {
                    bytes.extend((-11_i32).to_le_bytes());
                }
                bytes.resize(bytes.len() + 16 + 4 + tail_bytes, 0);
                let end = bytes.len();
                bytes.extend([0xa5; 8]);

                let mut refs = RefTable10_1::new();
                for cut in 0..end {
                    let mut reader = Reader::new(&bytes[..cut]);
                    assert!(
                        read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).is_err()
                    );
                    assert_eq!(reader.position(), 0);
                    assert!(refs.is_empty());
                }
                let mut reader = Reader::new(&bytes);
                let layer =
                    read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
                let Some(LayerBodyPrefix10_1::DimensionLayer(body)) =
                    layer.associated_layer_body.as_deref()
                else {
                    panic!("应读取 DimensionLayer");
                };
                assert_eq!(body.version, version);
                assert_eq!(body.legacy, (version == 2).then(|| (String::new(), -7)));
                assert_eq!(body.tail_long, (version == 4).then_some(-11));
                assert_eq!(refs.registered_class(41), Some(DIMENSION_LAYER_CLASS_ID));
                assert_eq!(refs.len(), 1);
                assert_eq!(reader.position(), end);
            }
        }
    }

    #[test]
    fn graphics_3d_rejects_new_dummy_layers_without_persisted_bodies() {
        for (class, short_bytes, tail_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 8, 0),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4, 8),
        ] {
            for dummy in [DUMMY_LAYER_CLASS_ID, DUMMY_GRADUATED_MARKER_LAYER_CLASS_ID] {
                let mut bytes = 1_u16.to_le_bytes().to_vec();
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.resize(bytes.len() + short_bytes, 0);
                bytes.extend(dummy);
                bytes.extend(1_u32.to_le_bytes());
                bytes.resize(bytes.len() + 16 + 4 + tail_bytes, 0);

                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let error =
                    read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap_err();
                assert_eq!(error.kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn graphics_3d_preserves_unknown_extensions_and_reads_successors() {
        for (class, version, short_bytes) in [
            (GRAPHICS_LAYER_3D_CLASS_ID, 5_u16, 8),
            (GLOBE_GRAPHICS_LAYER_CLASS_ID, 4_u16, 4),
        ] {
            let globe = class == GLOBE_GRAPHICS_LAYER_CLASS_ID;
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.resize(bytes.len() + short_bytes + 16 + 16 + 4, 0);
            if !globe {
                bytes.extend([0; 2 + 16]);
            }
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(3_u64.to_le_bytes());
            bytes.extend([0xe1; 16]);
            bytes.extend([1, 2, 3]);
            let extension_end = bytes.len();
            bytes.extend(42_u16.to_le_bytes());
            if globe {
                bytes.extend([0; 2 + 32]);
            }
            let end = bytes.len();
            bytes.extend([0xa5; 16]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let layer = read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).unwrap();
            let extensions = match layer.tail {
                Graphics3dTail10_1::Globe {
                    extensions,
                    shorts: [42, 0],
                    ..
                }
                | Graphics3dTail10_1::Scene {
                    extensions: Some(extensions),
                    version_5_short: Some(42),
                    ..
                } => extensions,
                tail => panic!("尾段不匹配：{tail:?}"),
            };
            assert_eq!(extensions[0].class_id, [0xe1; 16]);
            assert_eq!(
                extensions[0].unknown_body.as_deref(),
                Some([1, 2, 3].as_slice())
            );
            assert_eq!(reader.position(), end);
            assert!(refs.is_empty());

            let mut reader = Reader::new(&bytes[..extension_end - 1]);
            assert!(read_graphics_3d_layer_10_1(&class, &mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

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
