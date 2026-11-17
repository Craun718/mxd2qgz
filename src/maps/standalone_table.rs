//! ArcMap 10.1 StandaloneTable 类体的保守前缀读取器。
//!
//! 静态证据与未覆盖字段见 `docs/mxd-terra-host-table.md`。调用者须先读取
//! 对象头、确认 CLSID 并登记外层键；未知嵌套对象体前停止，不推断其长度。

use crate::layers::dep_tail::{TimeDataFamily10_1, read_time_data_family_10_1};
use crate::layers::feature_layer::{
    FEATURE_CLASS_NAME_CLASS_ID, FeatureClassNamePrefix10_1, read_feature_class_name_body_10_1,
};
use crate::layers::feature_layer_tail::{
    FIELD_INFO_CLASS_ID, FieldInfoBody10_1, FieldInfoStop10_1, read_field_info_body_10_1,
};
use crate::objects::dataset_name_more::{
    QUERY_TABLE_NAME_CLASS_ID, QueryTableNameBody10_1, read_query_table_name_body_10_1,
};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNameEntry10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// StandaloneTable 的 CLSID，采用 Windows GUID 字节序。
pub const STANDALONE_TABLE_CLASS_ID: [u8; 16] = [
    0x47, 0x50, 0x7c, 0xee, 0xdb, 0xe3, 0xd3, 0x11, 0xa0, 0x96, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0x26,
];

/// TableName 的 CLSID（Windows GUID 字节序）。
pub const TABLE_NAME_CLASS_ID: [u8; 16] = [
    0xb1, 0x3d, 0x78, 0x06, 0xee, 0xe5, 0xd1, 0x11, 0xb0, 0xa2, 0x00, 0x00, 0xf8, 0x78, 0x08, 0x20,
];

/// TableName 对象体；工作空间未知对象处停止，不推断后续长度。
///
/// GdbCoreLib 10.1 的 `FdbDatasetName::Load` 读取版本、三串及
/// IWorkspaceName；证据和原生版本门的差异见 `docs/mxd-terra-round8-tablename.md`。
#[derive(Debug, Clone, PartialEq)]
pub struct TableNameBody10_1 {
    pub version: u16,
    pub strings: [String; 3],
    pub workspace: WorkspaceNameEntry10_1,
}

pub(super) fn read_table_name_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
) -> io::Result<(TableNameBody10_1, bool)> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TableName 版本不支持",
        ));
    }
    let strings = [
        read_string_10_1(&mut r, max_bytes)?,
        read_string_10_1(&mut r, max_bytes)?,
        read_string_10_1(&mut r, max_bytes)?,
    ];
    let slot_start = r.clone();
    let before = refs.clone();
    let (workspace, stopped) = match read_object_entry_10_1(&mut r, &mut refs)? {
        ObjectEntry10_1::Null => (WorkspaceNameEntry10_1::Null, false),
        ObjectEntry10_1::BackReference { class_id, key } => (
            WorkspaceNameEntry10_1::BackReference { class_id, key },
            false,
        ),
        ObjectEntry10_1::NewObject { class_id, key } if class_id == WORKSPACE_NAME_CLASS_ID => {
            let body = read_workspace_name_body_10_1(&mut r, max_bytes, &mut refs, key)?;
            let stopped = body.stop.is_some();
            (WorkspaceNameEntry10_1::NewObject(Box::new(body)), stopped)
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            r = slot_start;
            refs = before;
            (WorkspaceNameEntry10_1::Unsupported { class_id, key }, true)
        }
    };
    *reader = r;
    *table = refs;
    Ok((
        TableNameBody10_1 {
            version,
            strings,
            workspace,
        },
        stopped,
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub enum StandaloneNameBody10_1 {
    FeatureClassName(Box<FeatureClassNamePrefix10_1>),
    QueryTableName(Box<QueryTableNameBody10_1>),
    TableName(Box<TableNameBody10_1>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandaloneTableStop10_1 {
    /// IName 新对象体未知；游标停在其对象头之前。
    DatasetName {
        class_id: [u8; 16],
        key: u32,
    },
    /// 名称配对条目中的新对象体未知；游标停在其对象头之前。
    NamedObject {
        name: String,
        class_id: [u8; 16],
        key: u32,
    },
    /// FieldInfo 内部的未知数字格式对象。
    NamedFieldInfoBody(FieldInfoStop10_1),
    /// 名称对象内部的未支持子对象。
    NameBody,
    /// 关系对象循环中的未知新对象。
    RelationshipObject {
        class_id: [u8; 16],
        key: u32,
    },
    /// 版本 3 起的对象槽中的未知新对象。
    SequenceObject {
        class_id: [u8; 16],
        key: u32,
    },
    /// 后续版本对象槽中的未知新对象。
    TailObject {
        slot: u8,
        class_id: [u8; 16],
        key: u32,
    },
    /// 时间族内部的未知对象。
    TimeFamily,
    /// 非空扩展的类体未分派；游标停在尺寸字段之前。
    Extension {
        class_id: [u8; 16],
        size: i64,
    },
    Complete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StandaloneTablePrefix10_1 {
    pub version: u16,
    pub dataset_name: Option<ObjectEntry10_1>,
    pub dataset_name_body: Option<StandaloneNameBody10_1>,
    pub named_objects: Vec<(String, ObjectEntry10_1)>,
    /// 与 named_objects 一一对应；空槽或回引没有新对象体。
    pub named_object_bodies: Vec<Option<FieldInfoBody10_1>>,
    pub trailing_string: Option<String>,
    pub relationships: Vec<ObjectEntry10_1>,
    pub sequence_object: Option<ObjectEntry10_1>,
    pub sequence_count: Option<i32>,
    pub sequence_tokens: Vec<i32>,
    pub sequence_values: Vec<i32>,
    pub raw_int32: Option<i32>,
    pub secondary_name: Option<ObjectEntry10_1>,
    pub secondary_name_body: Option<StandaloneNameBody10_1>,
    pub version_strings: Vec<String>,
    pub pre_join: Option<Vec<(String, String)>>,
    pub tail_objects: Vec<ObjectEntry10_1>,
    pub legacy_short: Option<u16>,
    pub time_family: Option<TimeDataFamily10_1>,
    pub extension_count: Option<i32>,
    pub stop: StandaloneTableStop10_1,
}

fn read_name(
    r: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
) -> io::Result<(
    Option<ObjectEntry10_1>,
    Option<StandaloneNameBody10_1>,
    Option<StandaloneTableStop10_1>,
)> {
    let mut trial = r.clone();
    let mut staged = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let (body, stopped) = match entry {
        ObjectEntry10_1::NewObject { class_id, key } if class_id == FEATURE_CLASS_NAME_CLASS_ID => {
            let body = read_feature_class_name_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stopped = body.stop.is_some();
            (
                Some(StandaloneNameBody10_1::FeatureClassName(Box::new(body))),
                stopped,
            )
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == QUERY_TABLE_NAME_CLASS_ID => {
            let body = read_query_table_name_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stopped = body.stop.is_some();
            (
                Some(StandaloneNameBody10_1::QueryTableName(Box::new(body))),
                stopped,
            )
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == TABLE_NAME_CLASS_ID => {
            let (body, stopped) = read_table_name_body_10_1(&mut trial, &mut staged, max_bytes)?;
            (
                Some(StandaloneNameBody10_1::TableName(Box::new(body))),
                stopped,
            )
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Ok((
                None,
                None,
                Some(StandaloneTableStop10_1::DatasetName { class_id, key }),
            ));
        }
        _ => (None, false),
    };
    *r = trial;
    *table = staged;
    Ok((
        Some(entry),
        body,
        stopped.then_some(StandaloneTableStop10_1::NameBody),
    ))
}

fn count(r: &mut Reader<'_>, max_entries: usize) -> io::Result<i32> {
    let n = r.read_u32_le()? as i32;
    if n < 0 || n as usize > max_entries {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "StandaloneTable 条目计数无效或超过限额",
        ));
    }
    Ok(n)
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
        dataset_name_body: None,
        named_objects: Vec::new(),
        named_object_bodies: Vec::new(),
        trailing_string: None,
        relationships: Vec::new(),
        sequence_object: None,
        sequence_count: None,
        sequence_tokens: Vec::new(),
        sequence_values: Vec::new(),
        raw_int32: None,
        secondary_name: None,
        secondary_name_body: None,
        version_strings: Vec::new(),
        pre_join: None,
        tail_objects: Vec::new(),
        legacy_short: None,
        time_family: None,
        extension_count: None,
        stop: StandaloneTableStop10_1::Complete,
    };
    let (entry, body, stop) = read_name(&mut r, &mut table, max_string_bytes)?;
    result.dataset_name = entry;
    result.dataset_name_body = body;
    if let Some(stop) = stop {
        result.stop = stop;
        *reader = r;
        *refs = table;
        return Ok(result);
    }
    for _ in 0..count(&mut r, max_entries)? {
        let name = read_string_10_1(&mut r, max_string_bytes)?;
        let mut slot_reader = r.clone();
        let mut slot_refs = table.clone();
        let entry = read_object_entry_10_1(&mut slot_reader, &mut slot_refs)?;
        let body = match entry {
            ObjectEntry10_1::NewObject { class_id, key } if class_id == FIELD_INFO_CLASS_ID => {
                Some(read_field_info_body_10_1(
                    &mut slot_reader,
                    max_string_bytes,
                    &mut slot_refs,
                    key,
                )?)
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                result.stop = StandaloneTableStop10_1::NamedObject {
                    name,
                    class_id,
                    key,
                };
                *reader = r;
                *refs = table;
                return Ok(result);
            }
            _ => None,
        };
        let stop = body.as_ref().and_then(|body| body.stop);
        r = slot_reader;
        table = slot_refs;
        result.named_objects.push((name, entry));
        result.named_object_bodies.push(body);
        if let Some(stop) = stop {
            result.stop = StandaloneTableStop10_1::NamedFieldInfoBody(stop);
            *reader = r;
            *refs = table;
            return Ok(result);
        }
    }
    result.trailing_string = Some(read_string_10_1(&mut r, max_string_bytes)?);
    if version >= 2 {
        for _ in 0..count(&mut r, max_entries)? {
            let mut trial = r.clone();
            let mut staged = table.clone();
            let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
            if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                result.stop = StandaloneTableStop10_1::RelationshipObject { class_id, key };
                *reader = r;
                *refs = table;
                return Ok(result);
            }
            r = trial;
            table = staged;
            result.relationships.push(entry);
        }
    }
    if version >= 3 {
        let mut trial = r.clone();
        let mut staged = table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            result.stop = StandaloneTableStop10_1::SequenceObject { class_id, key };
            *reader = r;
            *refs = table;
            return Ok(result);
        }
        r = trial;
        table = staged;
        result.sequence_object = Some(entry);
        let n = count(&mut r, max_entries)?;
        result.sequence_count = Some(n);
        let mut previous = -2i32;
        while result.sequence_values.len() < n as usize {
            let token = r.read_u32_le()? as i32;
            result.sequence_tokens.push(token);
            if token >= 0 {
                result.sequence_values.push(token);
                previous = token;
            } else {
                let end = token.checked_neg().ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "StandaloneTable 压缩序列值溢出")
                })?;
                let span = i64::from(end) - i64::from(previous);
                let remaining = n as usize - result.sequence_values.len();
                if span <= 0 || span as u64 > remaining as u64 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "StandaloneTable 压缩序列展开长度无效",
                    ));
                }
                for value in (i64::from(previous) + 1)..=i64::from(end) {
                    result.sequence_values.push(value as i32);
                }
                previous = end;
            }
        }
    }
    if version >= 5 {
        result.raw_int32 = Some(r.read_u32_le()? as i32);
    }
    if version >= 4 {
        let (entry, body, stop) = read_name(&mut r, &mut table, max_string_bytes)?;
        result.secondary_name = entry;
        result.secondary_name_body = body;
        if let Some(stop) = stop {
            result.stop = stop;
            *reader = r;
            *refs = table;
            return Ok(result);
        }
    }
    if version >= 6 {
        result
            .version_strings
            .push(read_string_10_1(&mut r, max_string_bytes)?);
    }
    if version == 7 {
        result
            .version_strings
            .push(read_string_10_1(&mut r, max_string_bytes)?);
    }
    if version >= 9 {
        result
            .version_strings
            .push(read_string_10_1(&mut r, max_string_bytes)?);
    }
    if version >= 10 {
        let mut pairs = Vec::new();
        for _ in 0..count(&mut r, max_entries)? {
            pairs.push((
                read_string_10_1(&mut r, max_string_bytes)?,
                read_string_10_1(&mut r, max_string_bytes)?,
            ));
        }
        result.pre_join = Some(pairs);
    }
    if version >= 11 {
        result
            .version_strings
            .push(read_string_10_1(&mut r, max_string_bytes)?);
        for slot in 0..if version == 11 || version == 12 { 3 } else { 1 } {
            if slot == 1 {
                result.legacy_short = Some(r.read_u16_le()?);
            }
            let mut trial = r.clone();
            let mut staged = table.clone();
            let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
            if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                result.stop = StandaloneTableStop10_1::TailObject {
                    slot,
                    class_id,
                    key,
                };
                *reader = r;
                *refs = table;
                return Ok(result);
            }
            r = trial;
            table = staged;
            result.tail_objects.push(entry);
        }
    }
    if version >= 12 {
        let family = read_time_data_family_10_1(&mut r, max_string_bytes, &mut table)?;
        let stopped = family.stop.is_some();
        result.time_family = Some(family);
        if stopped {
            result.stop = StandaloneTableStop10_1::TimeFamily;
            *reader = r;
            *refs = table;
            return Ok(result);
        }
    }
    if version >= 14 {
        let n = count(&mut r, max_entries)?;
        result.extension_count = Some(n);
        if n > 0 {
            let mut trial = r.clone();
            let size = i64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap());
            if size < 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "StandaloneTable 扩展尺寸无效",
                ));
            }
            let mut class_id = [0; 16];
            class_id.copy_from_slice(trial.read_bytes(16)?);
            result.stop = StandaloneTableStop10_1::Extension { class_id, size };
        }
    }
    *reader = r;
    *refs = table;
    Ok(result)
}
