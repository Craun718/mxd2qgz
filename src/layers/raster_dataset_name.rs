//! ArcMap Desktop 10.1 RasterDatasetName 对象体读取子集。
//!
//! 证据为 RasterDB.dll 10.1 CAB 映像（SHA-256
//! `97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`，PE32/i386，
//! ImageBase `0x7bfb0000`）：`Load 0x7c0ba5b0`、`Save 0x7c0ba790`、
//! `GetClassID 0x7c0ba570`。REA/Ghidra 导出记录为
//! `temp/luna-followup-20261118/raster-native/rasternameread.json`；函数边界与关键
//! IAT 分支由原始指令复核。样本只用于交叉验证，不用样本值命名字段。
//!
//! 调用者先读对象头并登记外层键，再把 RasterDatasetName 对象体的有界 Reader
//! 传入本模块。读取器最多消费四条顺序 String、IWorkspaceName 槽及该对象体
//! 版本门控制的两个 int32；未知嵌套类或旧零门值是成功停止点。

use super::fgdb_raster_dataset_name::{
    FGDB_RASTER_DATASET_NAME_CLASS_ID, FgdbRasterDatasetNamePrefix10_1, FgdbRasterIoPrefix10_1,
    read_fgdb_raster_dataset_name_prefix_10_1, read_fgdb_raster_io_prefix_10_1,
};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, WorkspaceNameStop10_1,
    read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io;

/// RasterDatasetName 类标识 `{75BCE6E2-8AF5-478E-8892-FA45CA50AF4D}` 的
/// 16 字节 Windows GUID 内存序。
///
/// RasterDB.dll `GetClassID 0x7c0ba570` 将 `0x7c12745c..0x7c12746b` 的 16 字节
/// 写入输出；同一映像 RTTI 将 `RasterDatasetName@ESRI` 类型描述符连到 COL
/// `0x7c148bc0` 和虚表 `0x7c133608`。样本键 1545 的父对象头类标识与此值相同。
pub const RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0xe2, 0xe6, 0xbc, 0x75, 0xf5, 0x8a, 0x8e, 0x47, 0x88, 0x92, 0xfa, 0x45, 0xca, 0x50, 0xaf, 0x4d,
];

/// RasterDatasetName 体内 IWorkspaceName 请求槽的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub enum RasterDatasetWorkspaceSlot10_1 {
    /// 空对象头；无对象体。
    Null,
    /// 已在当前对象流登记的键；无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 WorkspaceName 对象及其已确认体字段。
    WorkspaceName(Box<WorkspaceNamePrefix10_1>),
    /// 新对象，但类不是已支持的 WorkspaceName；游标停在该对象头前。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// RasterDatasetName 对象体的有界停止位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterDatasetNameStop10_1 {
    /// 保存版本门为 0；原生把流回退 4 字节后再以 String::Read 解释。
    /// 该零长度 String 分支超出当前严格 String 子集，故停在体起点。
    ZeroVersionGateRewinds,
    /// WorkspaceName 体在其 IPropertySet 新对象处遇到未支持的类。
    NestedWorkspaceName(WorkspaceNameStop10_1),
    /// IWorkspaceName 请求槽的新对象类尚无体读取器。
    UnsupportedWorkspaceClass { class_id: [u8; 16], key: u32 },
}

/// RasterDatasetName 10.1 Load 已确认的体字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RasterDatasetNameBody10_1 {
    /// 外层 RasterDatasetName 对象头中的键；调用者负责登记。
    pub key: u32,
    /// Load 先读取的 4 字节原值。Save 侧由 AfStreamGetSaveVersion 产生；
    /// 这里保留整数，不把它改写成字段或成员名称。
    pub version_gate: i32,
    /// Load 顺序读取并存入对象偏移 +0x10、+0x14、+0x18、+0x4c 的四条 String；
    /// 语义不在本模块命名。
    pub strings: Option<[String; 4]>,
    /// 紧随四条 String 的 LoadObject（请求 IWorkspaceName IID）。
    pub workspace: Option<RasterDatasetWorkspaceSlot10_1>,
    /// `version_gate >= 3` 时读取的 4 字节原值（对象偏移 +0x6c）。
    pub tail_at_6c: Option<i32>,
    /// `version_gate >= 4` 时读取的 4 字节原值（对象偏移 +0x70）。
    pub tail_at_70: Option<i32>,
    /// 成功停止点；`None` 表示已消费本类 Load 当前已闭合的字段序列。
    pub stop: Option<RasterDatasetNameStop10_1>,
}

/// 从已确定且已由调用者限定范围的 RasterDatasetName 对象体起点读取 10.1 字段。
///
/// `max_bytes` 限制每条 AfCore String。嵌套 IWorkspaceName 新对象按已确认类
/// 标识调用现有 WorkspaceName 读取器；其他新类在对象头处停止。读取以临时游标
/// 和登记表副本进行，格式错误不推进调用者状态；成功停止点提交已确认前缀。
pub fn read_raster_dataset_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RasterDatasetNameBody10_1> {
    let mut trial = reader.clone();
    let mut staged_table = table.clone();
    let version_gate = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    // Load 0x7c0ba5b0 对零门值执行 IStream::Seek(-4, CUR)，随后进入首个
    // String::Read。严格 String 读取器不支持 AfCore 的零长度旧串，因此在体首停。
    if version_gate == 0 {
        return Ok(RasterDatasetNameBody10_1 {
            key,
            version_gate,
            strings: None,
            workspace: None,
            tail_at_6c: None,
            tail_at_70: None,
            stop: Some(RasterDatasetNameStop10_1::ZeroVersionGateRewinds),
        });
    }

    let strings = [
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
    ];
    let before_nested = trial.clone();
    let mut stop = None;
    let workspace = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => RasterDatasetWorkspaceSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            RasterDatasetWorkspaceSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject {
            class_id,
            key: nested_key,
        } if class_id == WORKSPACE_NAME_CLASS_ID => {
            let body = read_workspace_name_body_10_1(
                &mut trial,
                max_bytes,
                &mut staged_table,
                nested_key,
            )?;
            if let Some(nested_stop) = body.stop {
                stop = Some(RasterDatasetNameStop10_1::NestedWorkspaceName(nested_stop));
            }
            RasterDatasetWorkspaceSlot10_1::WorkspaceName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id,
            key: nested_key,
        } => {
            stop = Some(RasterDatasetNameStop10_1::UnsupportedWorkspaceClass {
                class_id,
                key: nested_key,
            });
            trial = before_nested;
            staged_table = table.clone();
            RasterDatasetWorkspaceSlot10_1::Unsupported {
                class_id,
                key: nested_key,
            }
        }
    };

    let (tail_at_6c, tail_at_70) = if stop.is_none() {
        let at_6c = if version_gate >= 3 {
            Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
        } else {
            None
        };
        let at_70 = if version_gate >= 4 {
            Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
        } else {
            None
        };
        (at_6c, at_70)
    } else {
        (None, None)
    };

    *reader = trial;
    *table = staged_table;
    Ok(RasterDatasetNameBody10_1 {
        key,
        version_gate,
        strings: Some(strings),
        workspace: Some(workspace),
        tail_at_6c,
        tail_at_70,
        stop,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterDatasetNameEntryBody10_1 {
    Dataset(Box<RasterDatasetNameBody10_1>),
    FileGeodatabase {
        name: Box<FgdbRasterDatasetNamePrefix10_1>,
        records: Box<FgdbRasterIoPrefix10_1>,
    },
}

/// 在宿主已建立的 ObjectStream 表中读取名称条目；调用者须整体暂存游标与键表。
pub(super) fn read_supported_raster_name_entry_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<RasterDatasetNameEntryBody10_1>)> {
    let entry = read_object_entry_10_1(reader, table)?;
    let incomplete = |reason: String| io::Error::new(io::ErrorKind::Unsupported, reason);
    let body = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id: RASTER_DATASET_NAME_CLASS_ID | FGDB_RASTER_DATASET_NAME_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject {
            class_id: RASTER_DATASET_NAME_CLASS_ID,
            key,
        } => {
            let name = read_raster_dataset_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = name.stop {
                return Err(incomplete(format!("RasterDatasetName 未读完：{stop:?}")));
            }
            Some(RasterDatasetNameEntryBody10_1::Dataset(Box::new(name)))
        }
        ObjectEntry10_1::NewObject {
            class_id: FGDB_RASTER_DATASET_NAME_CLASS_ID,
            key,
        } => {
            let name = read_fgdb_raster_dataset_name_prefix_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = name.delegated_name.stop {
                return Err(incomplete(format!("FileGDB 委派名称未读完：{stop:?}")));
            }
            let records = read_fgdb_raster_io_prefix_10_1(reader, max_bytes)?;
            if let Some(stop) = records.stop {
                return Err(incomplete(format!("FileGDB 名称记录未读完：{stop:?}")));
            }
            Some(RasterDatasetNameEntryBody10_1::FileGeodatabase {
                name: Box::new(name),
                records: Box::new(records),
            })
        }
        ObjectEntry10_1::NewObject { class_id, .. }
        | ObjectEntry10_1::BackReference { class_id, .. } => {
            return Err(incomplete(format!(
                "未支持的栅格名称类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok((entry, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_string(text: &str) -> Vec<u8> {
        let payload: Vec<_> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
        bytes.extend(payload);
        bytes
    }

    fn body_with_null_workspace(gate: i32) -> Vec<u8> {
        let mut bytes = gate.to_le_bytes().to_vec();
        for value in ["one", "two", "three", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 16]);
        if gate >= 3 {
            bytes.extend(0x1122_3344_i32.to_le_bytes());
        }
        if gate >= 4 {
            bytes.extend(0x5566_7788_i32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn parses_modern_gate_strings_null_slot_and_versioned_tail() {
        let bytes = body_with_null_workspace(4);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();

        let parsed =
            read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut table, 1545).unwrap();

        assert_eq!(parsed.version_gate, 4);
        assert_eq!(
            parsed.strings.as_ref().unwrap(),
            &["one", "two", "three", ""]
        );
        assert!(matches!(
            parsed.workspace,
            Some(RasterDatasetWorkspaceSlot10_1::Null)
        ));
        assert_eq!(parsed.tail_at_6c, Some(0x1122_3344));
        assert_eq!(parsed.tail_at_70, Some(0x5566_7788));
        assert_eq!(reader.position(), bytes.len());
        assert!(table.is_empty());
    }

    #[test]
    fn gate_three_reads_only_the_first_trailing_integer() {
        let mut bytes = body_with_null_workspace(3);
        bytes.extend([0xaa; 4]);
        let expected_end = bytes.len() - 4;
        let mut reader = Reader::new(&bytes);

        let parsed =
            read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut RefTable10_1::new(), 9)
                .unwrap();

        assert_eq!(parsed.tail_at_6c, Some(0x1122_3344));
        assert_eq!(parsed.tail_at_70, None);
        assert_eq!(reader.position(), expected_end);
    }

    #[test]
    fn zero_gate_stops_at_body_start_after_native_rewind_branch() {
        let bytes = 0_i32.to_le_bytes();
        let mut reader = Reader::new(&bytes);

        let parsed =
            read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut RefTable10_1::new(), 9)
                .unwrap();

        assert_eq!(parsed.version_gate, 0);
        assert_eq!(
            parsed.stop,
            Some(RasterDatasetNameStop10_1::ZeroVersionGateRewinds)
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn truncation_rolls_back_cursor_and_reference_table() {
        let bytes = body_with_null_workspace(4);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            assert!(
                read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut table, 9).is_err()
            );
            assert_eq!(reader.position(), 0, "截断位置 {end} 应回滚");
            assert!(table.is_empty());
        }
    }
}
