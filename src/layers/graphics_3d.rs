//! ArcMap 10.1 GraphicsLayer3D 与 GlobeGraphicsLayer 对象体。
//! 字段序与版本门见 docs/arcmap-10.1-dep-tail.md「三维图形容器的对象体」。

pub use crate::layers::layer_extensions::LayerExtensionEntry10_1 as Graphics3dExtension10_1;
use crate::layers::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use crate::layers::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
use crate::maps::group_element::{
    GROUP_ELEMENT_CLASS_ID, GroupElement10_1, read_group_element_10_1,
};
use crate::maps::rectangle_element::{
    RECTANGLE_ELEMENT_CLASS_ID, RectangleElement10_1, read_rectangle_element_10_1,
};
use crate::objects::element_body::{ElementBody10_1, read_element_body_of_class_10_1};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_entry_10_1,
};
use crate::objects::geometry::{GeometryBody10_1, read_geometry_body_10_1};
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
    /// 组的新对象体；对象头仍保存在 element.entry。
    pub group: Option<Box<GroupElement10_1>>,
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
    read_body: impl FnOnce(&[u8; 16], &mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<Graphics3dObject10_1<T>> {
    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::NewObject { class_id, .. } => {
            Some(Box::new(read_body(&class_id, r, refs)?))
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
    read_object(r, refs, |cls, r, refs| {
        if *cls != ENVELOPE_CLASS_ID {
            return Err(unsupported(cls, "三维图形容器的 IEnvelope 槽"));
        }
        read_envelope_10_1(r, max, refs)
    })
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
        geometries.push(read_object(r, refs, |cls, r, refs| {
            read_geometry_body_10_1(cls, r, max, refs)
        })?);
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
    let associated = read_object(&mut r, &mut refs, |cls, r, refs| {
        read_complete_layer_body_10_1(r, *cls, max, refs)
    })?;
    let envelope = read_envelope(&mut r, max, &mut refs)?;
    let (element_count, iterations) = read_count(&mut r, 16)?;
    let mut elements = Vec::new();
    for _ in 0..iterations {
        let (element, rectangle, group) = match read_header_10_1(&mut r.clone())? {
            ObjectHeader::Object {
                class_id: RECTANGLE_ELEMENT_CLASS_ID,
                key,
            } if !refs.contains(key) => {
                let rectangle = read_rectangle_element_10_1(&mut r, max, &mut refs)?;
                (
                    Graphics3dObject10_1 {
                        entry: ObjectEntry10_1::NewObject {
                            class_id: RECTANGLE_ELEMENT_CLASS_ID,
                            key,
                        },
                        body: None,
                    },
                    Some(Box::new(rectangle)),
                    None,
                )
            }
            ObjectHeader::Object {
                class_id: GROUP_ELEMENT_CLASS_ID,
                key,
            } if !refs.contains(key) => {
                let group = read_group_element_10_1(&mut r, 0, max, &mut refs, &|_| false)?;
                (
                    Graphics3dObject10_1 {
                        entry: ObjectEntry10_1::NewObject {
                            class_id: GROUP_ELEMENT_CLASS_ID,
                            key,
                        },
                        body: None,
                    },
                    None,
                    Some(Box::new(group)),
                )
            }
            _ => (
                read_object(&mut r, &mut refs, |cls, r, refs| {
                    let body = read_element_body_of_class_10_1(cls, r, max, refs)?;
                    if let Some(stop) = body.stop() {
                        return Err(io::Error::new(ErrorKind::Unsupported, stop.describe()));
                    }
                    Ok(body)
                })?,
                None,
                None,
            ),
        };
        let globe_properties = (globe && version >= 2)
            .then(|| read_globe_element_properties(&mut r, max, &mut refs, version))
            .transpose()?;
        elements.push(Graphics3dElement10_1 {
            element,
            rectangle,
            group,
            globe_properties,
        });
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
