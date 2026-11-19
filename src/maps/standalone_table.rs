//! ArcMap 10.1 StandaloneTable 类体的保守前缀读取器。
//!
//! 静态证据与未覆盖字段见 `docs/mxd-terra-host-table.md`。调用者须先读取
//! 对象头、确认 CLSID 并登记外层键；未知嵌套对象体前停止，不推断其长度。

use crate::layers::annotate_props::{
    ANNOTATE_PROPS_COLLECTION_CLASS_ID, AnnotatePropsBody10_1, DISPLAY_EXPRESSION_PROPS_CLASS_ID,
    DisplayExpressionPropsPrefix10_1, ExpressionEngineSlot10_1,
    read_annotate_props_collection_body_10_1, read_display_expression_props_prefix_10_1,
};
use crate::layers::dep_tail::{TimeDataFamily10_1, read_time_data_family_10_1};
use crate::layers::feature_layer::FeatureClassNamePrefix10_1;
use crate::layers::feature_layer_tail::{
    FIELD_INFO_CLASS_ID, FieldInfoBody10_1, FieldInfoStop10_1, UID_CLASS_ID, UidBody10_1,
    read_field_info_body_10_1, read_uid_body_10_1,
};
pub use crate::layers::layer_extensions::LayerExtensionEntry10_1 as StandaloneTableExtension10_1;
use crate::layers::layer_extensions::read_layer_extension_entry_10_1;
use crate::objects::dataset_name_more::{
    DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, NetCdfTableNameBody10_1,
    QueryTableNameBody10_1, read_relationship_name_slot_10_1, read_tabular_name_slot_10_1,
};
use crate::objects::display_color::{ColorBody10_1, ColorSlot10_1, read_color_slot_10_1};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNameEntry10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
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
/// FgdbTableName 的 CLSID（Windows GUID 字节序）。
pub const FGDB_TABLE_NAME_CLASS_ID: [u8; 16] = [
    0xc5, 0x88, 0x6c, 0x84, 0xc5, 0x6c, 0xc7, 0x49, 0xa6, 0xfe, 0x8f, 0xa0, 0x3d, 0x79, 0xbd, 0x07,
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
    /// FileGDB 外层的版本和持久化 short，普通 TableName 没有外层。
    pub fgdb_wrapper: Option<(u16, u16)>,
}

/// FileGDB.dll 0x6fa33290 读取两个 short 后把同一流交给内联 TableName。
pub(crate) fn read_fgdb_table_name_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
) -> io::Result<(TableNameBody10_1, bool)> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "FgdbTableName 版本不支持",
        ));
    }
    let flags = trial.read_u16_le()?;
    let (mut body, stopped) = read_table_name_body_10_1(&mut trial, table, max_bytes)?;
    body.fgdb_wrapper = Some((version, flags));
    *reader = trial;
    Ok((body, stopped))
}

pub(crate) fn read_table_name_body_10_1(
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
            fgdb_wrapper: None,
        },
        stopped,
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub enum StandaloneNameBody10_1 {
    FeatureClassName(Box<FeatureClassNamePrefix10_1>),
    QueryTableName(Box<QueryTableNameBody10_1>),
    TableName(Box<TableNameBody10_1>),
    NetCdfTableName(Box<NetCdfTableNameBody10_1>),
    OtherTabularName(Box<DatasetNameMoreBody10_1>),
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
pub enum StandaloneTableTailBody10_1 {
    DisplayExpression(Box<DisplayExpressionPropsPrefix10_1>),
    Uid(UidBody10_1),
    AnnotateProps(Box<AnnotatePropsBody10_1>),
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
    /// 与 relationships 一一对应，包含已读取的关系名称新对象体。
    pub relationship_bodies: Vec<Option<DatasetNameMoreBody10_1>>,
    pub sequence_object: Option<ObjectEntry10_1>,
    pub sequence_color: Option<ColorBody10_1>,
    pub sequence_count: Option<i32>,
    pub sequence_tokens: Vec<i32>,
    pub sequence_values: Vec<i32>,
    pub raw_int32: Option<i32>,
    pub secondary_name: Option<ObjectEntry10_1>,
    pub secondary_name_body: Option<StandaloneNameBody10_1>,
    pub version_strings: Vec<String>,
    pub pre_join: Option<Vec<(String, String)>>,
    pub tail_objects: Vec<ObjectEntry10_1>,
    pub tail_object_bodies: Vec<Option<StandaloneTableTailBody10_1>>,
    pub legacy_short: Option<u16>,
    pub time_family: Option<TimeDataFamily10_1>,
    pub extension_count: Option<i32>,
    pub extensions: Vec<StandaloneTableExtension10_1>,
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
    let header = read_header_10_1(&mut r.clone())?;
    let mut stop = None;
    let name = read_tabular_name_slot_10_1(r, max_bytes, table, &mut stop)?;
    let (entry, body) = match name {
        DatasetNameMoreSlot10_1::Null => (ObjectEntry10_1::Null, None),
        DatasetNameMoreSlot10_1::BackReference { class_id, key } => {
            (ObjectEntry10_1::BackReference { class_id, key }, None)
        }
        DatasetNameMoreSlot10_1::Unsupported { class_id, key } => {
            return Ok((
                None,
                None,
                Some(StandaloneTableStop10_1::DatasetName { class_id, key }),
            ));
        }
        DatasetNameMoreSlot10_1::Body(name) => {
            let body = match name {
                DatasetNameMoreBody10_1::FeatureClassName(body) => {
                    StandaloneNameBody10_1::FeatureClassName(body)
                }
                DatasetNameMoreBody10_1::QueryTable(body) => {
                    StandaloneNameBody10_1::QueryTableName(body)
                }
                DatasetNameMoreBody10_1::Table(body) => StandaloneNameBody10_1::TableName(body),
                DatasetNameMoreBody10_1::NetCdfTable(body) => {
                    StandaloneNameBody10_1::NetCdfTableName(body)
                }
                body => StandaloneNameBody10_1::OtherTabularName(Box::new(body)),
            };
            let ObjectHeader::Object { class_id, key } = header else {
                unreachable!("已读取表名新对象体");
            };
            (ObjectEntry10_1::NewObject { class_id, key }, Some(body))
        }
    };
    Ok((
        Some(entry),
        body,
        stop.is_some().then_some(StandaloneTableStop10_1::NameBody),
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
        relationship_bodies: Vec::new(),
        sequence_object: None,
        sequence_color: None,
        sequence_count: None,
        sequence_tokens: Vec::new(),
        sequence_values: Vec::new(),
        raw_int32: None,
        secondary_name: None,
        secondary_name_body: None,
        version_strings: Vec::new(),
        pre_join: None,
        tail_objects: Vec::new(),
        tail_object_bodies: Vec::new(),
        legacy_short: None,
        time_family: None,
        extension_count: None,
        extensions: Vec::new(),
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
            let header = read_header_10_1(&mut r.clone())?;
            let mut trial = r.clone();
            let mut staged = table.clone();
            let mut stop = None;
            let relationship = read_relationship_name_slot_10_1(
                &mut trial,
                max_string_bytes,
                &mut staged,
                &mut stop,
            )?;
            if stop.is_some() {
                let ObjectHeader::Object { class_id, key } = header else {
                    unreachable!("关系名称的停止点位于新对象中");
                };
                result.stop = StandaloneTableStop10_1::RelationshipObject { class_id, key };
                *reader = r;
                *refs = table;
                return Ok(result);
            }
            let (entry, body) = match relationship {
                DatasetNameMoreSlot10_1::Null => (ObjectEntry10_1::Null, None),
                DatasetNameMoreSlot10_1::BackReference { class_id, key } => {
                    (ObjectEntry10_1::BackReference { class_id, key }, None)
                }
                DatasetNameMoreSlot10_1::Body(body) => {
                    let ObjectHeader::Object { class_id, key } = header else {
                        unreachable!("已读取关系名称新对象体");
                    };
                    (ObjectEntry10_1::NewObject { class_id, key }, Some(body))
                }
                DatasetNameMoreSlot10_1::Unsupported { .. } => unreachable!("未知关系名称应已停止"),
            };
            r = trial;
            table = staged;
            result.relationships.push(entry);
            result.relationship_bodies.push(body);
        }
    }
    if version >= 3 {
        let color = read_color_slot_10_1(&mut r, max_string_bytes, &mut table)?;
        result.sequence_object = Some(match color {
            ColorSlot10_1::Null => ObjectEntry10_1::Null,
            ColorSlot10_1::BackReference { class_id, key } => {
                ObjectEntry10_1::BackReference { class_id, key }
            }
            ColorSlot10_1::Color { class_id, body } => {
                let key = match &body {
                    ColorBody10_1::Rgb(body) => body.key,
                    ColorBody10_1::Core { body, .. } => body.key,
                    ColorBody10_1::Cmyk(body) => body.key,
                };
                result.sequence_color = Some(body);
                ObjectEntry10_1::NewObject { class_id, key }
            }
            ColorSlot10_1::Unsupported { class_id, key } => {
                result.stop = StandaloneTableStop10_1::SequenceObject { class_id, key };
                *reader = r;
                *refs = table;
                return Ok(result);
            }
        });
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
            let body = if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                let body = match (slot, class_id) {
                    (0, DISPLAY_EXPRESSION_PROPS_CLASS_ID) => {
                        let body = read_display_expression_props_prefix_10_1(
                            &mut trial,
                            max_string_bytes,
                            &mut staged,
                            key,
                        )?;
                        (!matches!(body.engine, ExpressionEngineSlot10_1::Unsupported { .. }))
                            .then(|| StandaloneTableTailBody10_1::DisplayExpression(Box::new(body)))
                    }
                    (1, UID_CLASS_ID) => Some(StandaloneTableTailBody10_1::Uid(
                        read_uid_body_10_1(&mut trial, key)?,
                    )),
                    (2, ANNOTATE_PROPS_COLLECTION_CLASS_ID) => {
                        let body = read_annotate_props_collection_body_10_1(
                            &mut trial,
                            max_string_bytes,
                            &mut staged,
                            key,
                        )?;
                        body.stop
                            .is_none()
                            .then(|| StandaloneTableTailBody10_1::AnnotateProps(Box::new(body)))
                    }
                    _ => None,
                };
                if body.is_none() {
                    result.stop = StandaloneTableStop10_1::TailObject {
                        slot,
                        class_id,
                        key,
                    };
                    *reader = r;
                    *refs = table;
                    return Ok(result);
                }
                body
            } else {
                None
            };
            r = trial;
            table = staged;
            result.tail_objects.push(entry);
            result.tail_object_bodies.push(body);
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
        let mut budget = max_string_bytes;
        for _ in 0..n {
            let mut trial = r.clone();
            let mut staged = table.clone();
            match read_layer_extension_entry_10_1(
                &mut trial,
                max_string_bytes,
                &mut staged,
                &mut budget,
            ) {
                Ok(extension) => {
                    r = trial;
                    table = staged;
                    result.extensions.push(extension);
                }
                Err(error) if error.kind() == ErrorKind::Unsupported => {
                    let mut header = r.clone();
                    let size = i64::from_le_bytes(header.read_bytes(8)?.try_into().unwrap());
                    let class_id = header.read_bytes(16)?.try_into().unwrap();
                    result.stop = StandaloneTableStop10_1::Extension { class_id, size };
                    break;
                }
                Err(error) => return Err(error),
            }
        }
    }
    *reader = r;
    *refs = table;
    Ok(result)
}
