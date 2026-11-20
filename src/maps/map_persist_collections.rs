//! Map 10.1 版本 33 的 IUnknown 集合、LONG 与 IMap 槽位安全读取子集。
//!
//! Map.dll `Map::Load` 原始指令 `0x79400959..0x79400ab0` 显示，版本 `<33`
//! 跳过集合分支；版本 `>=33` 读取计数、IUnknown 条目、LONG 和 IMap 槽位，
//! 随后进入版本 `>=34` 门。本模块在未知新对象体前停止，不猜测其长度。

use super::map_object::{
    MAP_CLASS_ID, MapPrefix10_1, MapSuffix10_1, MapSuffixStop, read_map_with_suffix_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 集合中的单个 IUnknown 请求槽；类标识仍按对象头原样保留。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownSlot10_1 {
    /// 对象头起点，相对 Reader 底层字节。
    pub offset: usize,
    pub entry: ObjectEntry10_1,
}

/// Map+0x1b8 的 IMap 请求槽。
#[derive(Debug, Clone, PartialEq)]
pub struct MapInterfaceSlot10_1 {
    /// 对象头起点，相对 Reader 底层字节。
    pub offset: usize,
    pub entry: ObjectEntry10_1,
    /// 新 Map 的对象体。空槽及回引仅保留对象头。
    pub body: Option<Box<(MapPrefix10_1, MapSuffix10_1)>>,
}

/// 未知新对象体的保守停止点；对象头未提交，等待后续专用读取器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapPersistCollectionsStop10_1 {
    /// IUnknown 集合中的新对象体。
    CollectionObjectBody {
        index: usize,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// Map+0x1b8 IMap 槽中的新对象体。
    MapInterfaceObjectBody {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
}

/// Map 版本 33 分支已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MapPersistCollections10_1 {
    pub start_offset: usize,
    /// 集合声明计数。
    pub count: i32,
    /// 已完整读取的空槽和回引槽；新对象体处停止时不含该新对象头。
    pub collection: Vec<UnknownSlot10_1>,
    /// 原始 LONG 值；集合出现未知新对象体时尚未读取，故为 None。
    pub long_value: Option<i32>,
    /// Map+0x1b8 IMap 槽；若该槽为未知新对象则保持 None 并在槽头停止。
    pub map_interface: Option<MapInterfaceSlot10_1>,
    pub stop: Option<MapPersistCollectionsStop10_1>,
}

/// 读取 Map 版本 `>=33` 的集合和槽位，停在版本 `>=34` 数据块之前。
///
/// 计数是有符号 int32；负数、超过字节上限或截断均报错，且整个调用不推进
/// Reader 或引用表。IMap 新对象完整读取后才提交其键；遇到未知类或
/// 未闭合对象时只提交此前字段，游标停在对象头起点。
pub fn read_map_persist_collections_before_v34_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    architecture_version: u16,
) -> io::Result<MapPersistCollections10_1> {
    if architecture_version < 33 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("Map 持久化集合要求架构版本 >=33，收到 {architecture_version}"),
        ));
    }

    let start_offset = reader.position();
    let mut trial = reader.clone();
    let count_offset = trial.position();
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let item_count = usize::try_from(count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Map IUnknown 集合计数为负"))?;
    let min_bytes = item_count
        .checked_mul(16)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Map IUnknown 集合计数溢出"))?;
    if min_bytes > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map IUnknown 集合计数 {count} 超过字节上限 {max_bytes}（计数偏移 0x{count_offset:x}）"
            ),
        ));
    }
    if min_bytes > trial.remaining() {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!("Map IUnknown 集合至少需要 {min_bytes} 字节"),
        ));
    }

    let mut collection = Vec::with_capacity(item_count);
    for index in 0..item_count {
        let offset = trial.position();
        let mut slot_reader = trial.clone();
        let entry = peek_slot(&mut slot_reader, refs)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            *reader = trial;
            return Ok(MapPersistCollections10_1 {
                start_offset,
                count,
                collection,
                long_value: None,
                map_interface: None,
                stop: Some(MapPersistCollectionsStop10_1::CollectionObjectBody {
                    index,
                    offset,
                    class_id,
                    key,
                }),
            });
        }
        trial = slot_reader;
        collection.push(UnknownSlot10_1 { offset, entry });
    }

    let long_value = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let offset = trial.position();
    let mut slot_reader = trial.clone();
    let entry = peek_slot(&mut slot_reader, refs)?;
    let mut body = None;
    if let ObjectEntry10_1::NewObject { class_id, key } = entry {
        if class_id == MAP_CLASS_ID {
            let mut staged = refs.clone();
            staged.register(key);
            let parsed = staged.with_nested_graphics(|staged| {
                read_map_with_suffix_10_1(&mut slot_reader, max_bytes, staged)
            });
            if let Ok(parsed) = parsed
                && parsed.1.stop == MapSuffixStop::Complete
            {
                *refs = staged;
                body = Some(Box::new(parsed));
            }
        }
        if body.is_none() {
            *reader = trial;
            return Ok(MapPersistCollections10_1 {
                start_offset,
                count,
                collection,
                long_value: Some(long_value),
                map_interface: None,
                stop: Some(MapPersistCollectionsStop10_1::MapInterfaceObjectBody {
                    offset,
                    class_id,
                    key,
                }),
            });
        }
    }

    trial = slot_reader;
    let map_interface = MapInterfaceSlot10_1 {
        offset,
        entry,
        body,
    };
    *reader = trial;
    Ok(MapPersistCollections10_1 {
        start_offset,
        count,
        collection,
        long_value: Some(long_value),
        map_interface: Some(map_interface),
        stop: None,
    })
}

// 此子集从不消费新对象头，因而无需为每个槽复制或修改会话表。
fn peek_slot(reader: &mut Reader<'_>, refs: &RefTable10_1) -> io::Result<ObjectEntry10_1> {
    Ok(match read_header_10_1(reader)? {
        ObjectHeader::Null => ObjectEntry10_1::Null,
        ObjectHeader::Object { class_id, key } if refs.contains(key) => {
            ObjectEntry10_1::BackReference { class_id, key }
        }
        ObjectHeader::Object { class_id, key } => ObjectEntry10_1::NewObject { class_id, key },
    })
}
