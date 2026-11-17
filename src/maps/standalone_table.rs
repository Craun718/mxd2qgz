//! ArcMap 10.1 StandaloneTable 类体的保守前缀读取器。
//!
//! 静态证据与未覆盖字段见 `docs/mxd-terra-host-table.md`。调用者须先读取
//! 对象头、确认 CLSID 并登记外层键；未知嵌套对象体前停止，不推断其长度。

use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// StandaloneTable 的 CLSID，采用 Windows GUID 字节序。
pub const STANDALONE_TABLE_CLASS_ID: [u8; 16] = [
    0x47, 0x50, 0x7c, 0xee, 0xdb, 0xe3, 0xd3, 0x11, 0xa0, 0x96, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0x26,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandaloneTableStop10_1 {
    /// IName 新对象体未知；游标停在其对象头之前。
    DatasetName { class_id: [u8; 16], key: u32 },
    /// 名称配对条目中的新对象体未知；游标停在其对象头之前。
    NamedObject {
        name: String,
        class_id: [u8; 16],
        key: u32,
    },
    /// 已读共同前缀；后续关系类和版本分支尚未实现。
    RemainingBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneTablePrefix10_1 {
    pub version: u16,
    pub dataset_name: Option<ObjectEntry10_1>,
    pub named_objects: Vec<(String, ObjectEntry10_1)>,
    pub trailing_string: Option<String>,
    pub stop: StandaloneTableStop10_1,
}

/// 读取对象体前缀；错误时游标与引用表不变。停止时不登记未知新对象。
/// `max_entries` 和 `max_string_bytes` 是调用者指定的资源限额，非原生限制。
pub fn read_standalone_table_prefix_10_1(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max_entries: usize,
    max_string_bytes: usize,
) -> io::Result<StandaloneTablePrefix10_1> {
    let mut r = reader.clone();
    let mut table = refs.clone();
    let version = r.read_u16_le()?;
    // 原生使用有符号比较；此安全子集拒绝负值位模式。
    if version > 14 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "StandaloneTable 版本不支持",
        ));
    }
    let mut result = StandaloneTablePrefix10_1 {
        version,
        dataset_name: None,
        named_objects: Vec::new(),
        trailing_string: None,
        stop: StandaloneTableStop10_1::RemainingBody,
    };
    let mut slot_reader = r.clone();
    let mut slot_refs = table.clone();
    let entry = read_object_entry_10_1(&mut slot_reader, &mut slot_refs)?;
    if let ObjectEntry10_1::NewObject { class_id, key } = entry {
        result.stop = StandaloneTableStop10_1::DatasetName { class_id, key };
        *reader = r;
        return Ok(result);
    }
    r = slot_reader;
    table = slot_refs;
    result.dataset_name = Some(entry);
    let count = r.read_u32_le()? as i32;
    if count < 0 || count as usize > max_entries {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "StandaloneTable 条目计数无效或超过限额",
        ));
    }
    for _ in 0..count {
        let name = read_string_10_1(&mut r, max_string_bytes)?;
        let mut slot_reader = r.clone();
        let mut slot_refs = table.clone();
        let entry = read_object_entry_10_1(&mut slot_reader, &mut slot_refs)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            result.stop = StandaloneTableStop10_1::NamedObject {
                name,
                class_id,
                key,
            };
            *reader = r;
            *refs = table;
            return Ok(result);
        }
        r = slot_reader;
        table = slot_refs;
        result.named_objects.push((name, entry));
    }
    result.trailing_string = Some(read_string_10_1(&mut r, max_string_bytes)?);
    *reader = r;
    *refs = table;
    Ok(result)
}
