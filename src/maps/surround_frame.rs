//! MapElements.dll 10.1.0.3035 MapSurroundFrame::Load（0x7957b870）。

use crate::maps::map_frame::{
    BaseFrameElement10_1, FrameGeometrySlot10_1, MapFrame10_1, base_frame, envelope_slot,
    geometry_slot, read_map_frame_tail,
};
use crate::maps::map_object::{MapSurroundElement10_1, read_map_surround_body_10_1};
use crate::maps::page_layout_prefix::{
    MAP_FRAME_CLASS_ID, check_root_map_link, read_map_frame_prefix,
};
use crate::objects::envelope::Envelope10_1;
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// GetClassID 0x7957b840 从 0x7959bebc 复制的流内类标识。
pub const MAP_SURROUND_FRAME_CLASS_ID: [u8; 16] = [
    0xe1, 0xca, 0xff, 0x83, 0xca, 0xed, 0xd0, 0x11, 0x86, 0x83, 0, 0, 0xf8, 0x75, 0x17, 0x20,
];

#[derive(Debug, Clone, PartialEq)]
pub struct MapSurroundFrame10_1 {
    pub key: u32,
    pub version: u16,
    pub surround: ObjectEntry10_1,
    pub surround_body: Option<MapSurroundElement10_1>,
    pub map_frame: ObjectEntry10_1,
    pub map_frame_body: Option<Box<MapFrame10_1>>,
    pub bounds: SurroundFrameBounds10_1,
    pub base: BaseFrameElement10_1,
    pub version_three_double_bits: Option<[u64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurroundFrameBounds10_1 {
    Geometry(FrameGeometrySlot10_1),
    Envelope {
        entry: ObjectEntry10_1,
        body: Option<Box<Envelope10_1>>,
    },
}

// 调用者持有试读游标及临时键表；这里的失败由宿主回滚整个元素。
pub(crate) fn read_surround_frame_10_1(
    reader: &mut Reader<'_>,
    region_offset: usize,
    limit: usize,
    refs: &mut RefTable10_1,
    is_root_map: &impl Fn(u32) -> bool,
) -> io::Result<MapSurroundFrame10_1> {
    let ObjectEntry10_1::NewObject {
        class_id: MAP_SURROUND_FRAME_CLASS_ID,
        key,
    } = read_object_entry_10_1(reader, refs)?
    else {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "此槽不是新 MapSurroundFrame 对象",
        ));
    };
    let version = reader.read_u16_le()?;
    if version > 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 MapSurroundFrame 版本",
        ));
    }
    let surround = read_object_entry_10_1(reader, refs)?;
    let surround_body = if let ObjectEntry10_1::NewObject { class_id, key } = surround {
        let (body, stop) = read_map_surround_body_10_1(class_id, key, reader, limit, refs)?;
        if let Some(stop) = stop {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("IMapSurround 体未完整：{stop:?}"),
            ));
        }
        Some(body)
    } else {
        None
    };
    let mut probe = reader.clone();
    let map_frame = match read_header_10_1(&mut probe)? {
        ObjectHeader::Null => ObjectEntry10_1::Null,
        ObjectHeader::Object {
            class_id: MAP_FRAME_CLASS_ID,
            key,
        } => {
            if refs.contains(key) {
                ObjectEntry10_1::BackReference {
                    class_id: MAP_FRAME_CLASS_ID,
                    key,
                }
            } else {
                ObjectEntry10_1::NewObject {
                    class_id: MAP_FRAME_CLASS_ID,
                    key,
                }
            }
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的 IMapFrame 类",
            ));
        }
    };
    let map_frame_body = if let ObjectEntry10_1::NewObject { key, .. } = map_frame {
        let prefix = read_map_frame_prefix(reader, region_offset, refs)?;
        check_root_map_link(&prefix, is_root_map)?;
        refs.register(key);
        Some(Box::new(read_map_frame_tail(reader, limit, refs, prefix)?))
    } else {
        *reader = probe;
        None
    };
    let bounds = if version < 2 {
        SurroundFrameBounds10_1::Geometry(geometry_slot(reader, limit, refs)?)
    } else {
        let (entry, body) = envelope_slot(reader, limit, refs)?;
        SurroundFrameBounds10_1::Envelope {
            entry,
            body: body.map(Box::new),
        }
    };
    let base = base_frame(reader, limit, refs)?;
    let version_three_double_bits = if version >= 3 {
        Some([
            reader.read_f64_le()?.to_bits(),
            reader.read_f64_le()?.to_bits(),
        ])
    } else {
        None
    };
    Ok(MapSurroundFrame10_1 {
        key,
        version,
        surround,
        surround_body,
        map_frame,
        map_frame_body,
        bounds,
        base,
        version_three_double_bits,
    })
}
