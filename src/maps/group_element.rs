//! GraphicElements.dll 10.1 GroupElement 的普通对象流子元素。
use super::map_frame::{BaseFrameElement10_1, base_frame};
use super::rectangle_element::{
    RECTANGLE_ELEMENT_CLASS_ID, RectangleElement10_1, read_rectangle_element_10_1,
};
use super::surround_frame::{
    MAP_SURROUND_FRAME_CLASS_ID, MapSurroundFrame10_1, read_surround_frame_10_1,
};
use crate::objects::element_body::{
    ElementBody10_1, ElementPropertiesPrefix10_1, PrefixOutcome10_1,
    read_element_body_of_class_10_1, read_element_properties_prefix_10_1,
};
use crate::objects::geometry::{GeometryBody10_1, read_geometry_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const GROUP_ELEMENT_CLASS_ID: [u8; 16] = [
    0xd2, 0x77, 0x35, 0x80, 0xa3, 0xf8, 0xd0, 0x11, 0x83, 0xaf, 8, 0, 9, 0xb9, 0x96, 0xcc,
];

#[derive(Debug, Clone, PartialEq)]
pub enum GroupChild10_1 {
    Indirect(ObjectEntry10_1),
    Group(Box<GroupElement10_1>),
    Rectangle(Box<RectangleElement10_1>),
    SurroundFrame(Box<MapSurroundFrame10_1>),
    Element(u32, Box<ElementBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupElement10_1 {
    pub key: u32,
    pub version: u16,
    pub properties: ElementPropertiesPrefix10_1,
    pub children: Vec<GroupChild10_1>,
    pub geometry: Option<(ObjectEntry10_1, Option<GeometryBody10_1>)>,
    pub base: BaseFrameElement10_1,
}

/// 从已知边界读取组，子元素继续使用同一键表。未知体或超限时回滚整组。
pub(super) fn read_group_element_10_1(
    r: &mut Reader<'_>,
    region_offset: usize,
    limit: usize,
    table: &mut RefTable10_1,
    is_root_map: &impl Fn(u32) -> bool,
) -> io::Result<GroupElement10_1> {
    let mut candidate = r.clone();
    let mut refs = table.clone();
    let result = read_group(
        &mut candidate,
        region_offset,
        limit,
        &mut refs,
        is_root_map,
        16,
    )?;
    *r = candidate;
    *table = refs;
    Ok(result)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, message.into())
}

fn read_group(
    r: &mut Reader<'_>,
    region_offset: usize,
    limit: usize,
    refs: &mut RefTable10_1,
    is_root_map: &impl Fn(u32) -> bool,
    depth: usize,
) -> io::Result<GroupElement10_1> {
    if depth == 0 {
        return Err(invalid("GroupElement 嵌套超过限额"));
    }
    let ObjectEntry10_1::NewObject {
        class_id: GROUP_ELEMENT_CLASS_ID,
        key,
    } = read_object_entry_10_1(r, refs)?
    else {
        return Err(unsupported("此槽不是新 GroupElement"));
    };
    let version = r.read_u16_le()?;
    if version > 3 {
        return Err(invalid("不支持的 GroupElement 版本"));
    }
    let properties = match read_element_properties_prefix_10_1(r, limit, refs)? {
        PrefixOutcome10_1::Complete(body) => body,
        PrefixOutcome10_1::Stopped(reason) => return Err(unsupported(reason.describe())),
    };
    let n = r.read_u32_le()?;
    if n > i32::MAX as u32 || n as usize > limit.min(r.remaining()) / 16 {
        return Err(invalid("GroupElement 子元素数量超过限额"));
    }
    let mut children = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let header = read_header_10_1(&mut r.clone())?;
        let child = match header {
            ObjectHeader::Object {
                class_id: GROUP_ELEMENT_CLASS_ID,
                key,
            } if !refs.contains(key) => GroupChild10_1::Group(Box::new(read_group(
                r,
                region_offset,
                limit,
                refs,
                is_root_map,
                depth - 1,
            )?)),
            ObjectHeader::Object {
                class_id: RECTANGLE_ELEMENT_CLASS_ID,
                key,
            } if !refs.contains(key) => {
                GroupChild10_1::Rectangle(Box::new(read_rectangle_element_10_1(r, limit, refs)?))
            }
            ObjectHeader::Object {
                class_id: MAP_SURROUND_FRAME_CLASS_ID,
                key,
            } if !refs.contains(key) => GroupChild10_1::SurroundFrame(Box::new(
                read_surround_frame_10_1(r, region_offset, limit, refs, is_root_map)?,
            )),
            _ => {
                let entry = read_object_entry_10_1(r, refs)?;
                if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                    let body = read_element_body_of_class_10_1(&class_id, r, limit, refs)?;
                    if let Some(stop) = body.stop() {
                        return Err(unsupported(stop.describe()));
                    }
                    GroupChild10_1::Element(key, Box::new(body))
                } else {
                    GroupChild10_1::Indirect(entry)
                }
            }
        };
        children.push(child);
    }
    let geometry = if version >= 3 {
        let offset = r.position();
        let entry = read_object_entry_10_1(r, refs)?;
        let body = match entry {
            ObjectEntry10_1::NewObject { class_id, .. } => Some(
                read_geometry_body_10_1(&class_id, r, limit, refs).map_err(|error| {
                    io::Error::new(
                        error.kind(),
                        format!("GroupElement IGeometry @0x{offset:x}：{error}"),
                    )
                })?,
            ),
            _ => None,
        };
        Some((entry, body))
    } else {
        None
    };
    let base = base_frame(r, limit, refs)?;
    Ok(GroupElement10_1 {
        key,
        version,
        properties,
        children,
        geometry,
        base,
    })
}
