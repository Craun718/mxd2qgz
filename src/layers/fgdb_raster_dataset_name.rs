//! ArcMap Desktop 10.1 FgdbRasterDatasetName 的委派名称前缀。
//!
//! `FgdbRasterDB.dll`（SHA-256
//! `571be820c79b217571ea8f902c091e44f6268566bd35880488e8c4ef74008fef`，PE32/i386，
//! ImageBase `0x6f920000`）构造函数 `0x6f9339c0` 在 `0x6f933ade` 将普通
//! `RasterDatasetName` CLSID（`0x6f947648`）传给 `AfCreateInstance`
//! （IAT `0x6f9450ac`），输出槽为该构造对象 `+0x34`。类 CLSID 由
//! `GetClassID 0x6f933960` 从 `0x6f947668` 返回。对应 `Load 0x6f934910`
//! 先对内部名称接口请求 `IPersistStream` 并调用 `Load`，随后才开始 RasterIO。
//!
//! 普通 `RasterDatasetName::Load` 的已确认前缀由
//! [`crate::layers::raster_dataset_name`] 复用。仓库样本 `/Maps:0x501e` 的四条
//! String、WorkspaceName 新对象及版本尾字段解析至 `0x513b`；该位置开始出现
//! RasterIO magic `0x00ff00fa`。调用者可在此边界后独立解析已证实的 tag 1–7
//! 安全子集。
//! ctor `0x6f933a6e` 将 `0x6f947518` 安装到完整对象 `+0x14` 的接口槽；该表
//! `+0x14` 的原始 DWORD 指向 `Load 0x6f934910`。Load 所见接口 `+0x20` 因而对应
//! 完整对象 `+0x34`，与 `AfCreateInstance` 输出槽一致。

use crate::layers::raster_dataset_name::{
    RasterDatasetNameBody10_1, read_raster_dataset_name_body_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::reader::Reader;
use std::io;

/// FgdbRasterDatasetName 类标识 `{F2137C6F-9075-4BAF-9B38-E2F36083FBF4}` 的
/// 16 字节 Windows GUID 内存序；由 GetClassID `0x6f933960` 写出。
pub const FGDB_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x6f, 0x7c, 0x13, 0xf2, 0x75, 0x90, 0xaf, 0x4b, 0x9b, 0x38, 0xe2, 0xf3, 0x60, 0x83, 0xfb, 0xf4,
];

/// FgdbRasterDatasetName 对象体中已证实的委派前缀。
#[derive(Debug, Clone, PartialEq)]
pub struct FgdbRasterDatasetNamePrefix10_1 {
    /// 复用普通 RasterDatasetName 读取器的结果。这里的 `key` 是外层已登记的
    /// FgdbRasterDatasetName 键；内联名称没有自己的对象头，不新登记此键。
    /// 只有其中的 WorkspaceName 新对象按其流内键登记。
    pub delegated_name: RasterDatasetNameBody10_1,
    /// 完整读过委派前缀时的 RasterIO 起点，位置相对传入对象体 Reader。
    /// 调用者可在此位置继续调用独立的 RasterIO 前缀读取器。
    pub rasterio_candidate_offset: Option<usize>,
}

/// 读取 FgdbRasterDatasetName 内部委派给普通 RasterDatasetName 的 10.1 前缀。
///
/// 调用者须从 FgdbRasterDatasetName 对象体起点传入 Reader，并用父对象确定的
/// 有界范围限制读取。普通名称体按自身已确认规则读四条 String、WorkspaceName
/// 槽和版本尾字段；只有该完整前缀成功闭合时才返回 RasterIO 候选起点。错误不
/// 推进输入 Reader 或引用表。
pub fn read_fgdb_raster_dataset_name_prefix_10_1(
    reader: &mut Reader<'_>,
    max_string_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FgdbRasterDatasetNamePrefix10_1> {
    let mut trial = reader.clone();
    let mut staged_table = table.clone();
    let delegated =
        read_raster_dataset_name_body_10_1(&mut trial, max_string_bytes, &mut staged_table, key)?;

    let rasterio_candidate_offset = delegated.stop.is_none().then_some(trial.position());
    let result = FgdbRasterDatasetNamePrefix10_1 {
        delegated_name: delegated,
        rasterio_candidate_offset,
    };

    *reader = trial;
    *table = staged_table;
    Ok(result)
}

/// tag 6 的对象槽；当前只闭合了样本及原生代码确认的空对象分支。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FgdbRasterIoObject10_1 {
    /// `ReadObj` 读取到 16 字节全零类标识，返回空接口。
    Null,
    /// 非空类对象尚未解析；游标停在类标识前。
    Unsupported { class_id: [u8; 16] },
}

/// RasterIO tag 1–7 的已确认顺序字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FgdbRasterIoPrefix10_1 {
    /// tag 1，int32 原值。
    pub tag1_value: i32,
    /// tag 2，UTF-16LE 字符串。
    pub tag2_string: String,
    /// tag 3，UTF-16LE 字符串。
    pub tag3_string: String,
    /// tag 4，int32 原值。
    pub tag4_value: i32,
    /// tag 5，UTF-16LE 字符串。
    pub tag5_string: String,
    /// tag 6，当前仅支持空对象。
    pub tag6_object: FgdbRasterIoObject10_1,
    /// tag 7，int32 原值；原生允许该字段读取失败后仍调用 End。
    pub tag7_value: Option<i32>,
    /// End 可定位的结束位置；相对传入 Reader。
    pub end_offset: Option<usize>,
    /// 已知但未解析的对象/尾记录停止点。
    pub stop: Option<FgdbRasterIoStop10_1>,
}

/// RasterIO 安全前缀的有界停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FgdbRasterIoStop10_1 {
    /// tag 6 是非空类对象。
    UnsupportedObjectClass { class_id: [u8; 16] },
    /// tag 7 后有额外记录；原生 End 会扫描，当前子集停在该记录头前。
    TrailingRecord { tag: i32, length: i32 },
}

const RASTER_IO_MAGIC_10_1: i32 = 0x00ff_00fa;
const NULL_CLASS_ID_10_1: [u8; 16] = [0; 16];

pub(super) fn read_raster_io_record_header_10_1(reader: &mut Reader<'_>) -> io::Result<(i32, i32)> {
    let tag = i32::from_le_bytes(reader.read_bytes(4)?.try_into().unwrap());
    let length = i32::from_le_bytes(reader.read_bytes(4)?.try_into().unwrap());
    Ok((tag, length))
}

pub(super) fn read_raster_io_i32_10_1(
    reader: &mut Reader<'_>,
    expected_tag: i32,
) -> io::Result<i32> {
    let (tag, length) = read_raster_io_record_header_10_1(reader)?;
    if tag != expected_tag || length != 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "RasterIO 定长记录不匹配：期望 tag={expected_tag} length=4，实际 tag={tag} length={length}"
            ),
        ));
    }
    Ok(i32::from_le_bytes(
        reader.read_bytes(4)?.try_into().unwrap(),
    ))
}

pub(super) fn read_raster_io_string_10_1(
    reader: &mut Reader<'_>,
    expected_tag: i32,
    max_string_bytes: usize,
) -> io::Result<String> {
    let (tag, length) = read_raster_io_record_header_10_1(reader)?;
    if tag != expected_tag || length < 2 || length % 2 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "RasterIO 字符串记录不匹配：期望 tag={expected_tag}，实际 tag={tag} length={length}"
            ),
        ));
    }
    let length = usize::try_from(length).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "RasterIO 字符串长度不能由本平台表示",
        )
    })?;
    if length > max_string_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "RasterIO 字符串长度超过调用者资源限额",
        ));
    }
    let bytes = reader.read_bytes(length)?;
    if bytes[length - 2..] != [0, 0] || bytes[..length - 2].chunks_exact(2).any(|u| u == [0, 0]) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "RasterIO UTF-16LE 字符串缺少唯一尾部零",
        ));
    }
    Reader::new(&bytes[..length - 2]).read_utf16_le(length - 2)
}

/// 从已确认的委派体之后读取 RasterIO 的 tag 1–7 子集。
///
/// RasterCoreLib.dll 10.1 的导出映射与指令确认 tag/length 是两个连续 int32：
/// `read_tag 0x7bf13c80` 分别读取 4 字节；`ReadBase 0x7bf14360` 用记录长度读值；
/// `skip 0x7bf13eb0` 以该长度作为流偏移。`ReadObj` `0x7bf143c0` 转入
/// `get_obj 0x7bf13d90`，读 16 字节类标识；零 CLSID 返回空对象。`End 0x7bf13bd0`
/// 在读取模式下调用 `seek_tag(-1)`，随后清状态并返回成功。当前子集只支持 tag 6
/// 空对象和 tag 7 后紧接的 `(-1,-1)` 结束对；未知对象/尾记录作为停止点保留。
/// 任一格式错误不推进调用者 Reader。
pub fn read_fgdb_raster_io_prefix_10_1(
    reader: &mut Reader<'_>,
    max_string_bytes: usize,
) -> io::Result<FgdbRasterIoPrefix10_1> {
    let mut trial = reader.clone();
    let (magic, magic_length) = read_raster_io_record_header_10_1(&mut trial)?;
    if magic != RASTER_IO_MAGIC_10_1 || magic_length != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("RasterIO InitCheck 记录不匹配：tag={magic:#x} length={magic_length}"),
        ));
    }

    let tag1_value = read_raster_io_i32_10_1(&mut trial, 1)?;
    let tag2_string = read_raster_io_string_10_1(&mut trial, 2, max_string_bytes)?;
    let tag3_string = read_raster_io_string_10_1(&mut trial, 3, max_string_bytes)?;
    let tag4_value = read_raster_io_i32_10_1(&mut trial, 4)?;
    let tag5_string = read_raster_io_string_10_1(&mut trial, 5, max_string_bytes)?;

    let (tag6, tag6_length) = read_raster_io_record_header_10_1(&mut trial)?;
    if tag6 != 6 || tag6_length != -1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("RasterIO tag 6 对象记录不匹配：tag={tag6} length={tag6_length}"),
        ));
    }
    let object_data_start = trial.clone();
    let class_id: [u8; 16] = trial.read_bytes(16)?.try_into().unwrap();
    if class_id != NULL_CLASS_ID_10_1 {
        *reader = object_data_start;
        return Ok(FgdbRasterIoPrefix10_1 {
            tag1_value,
            tag2_string,
            tag3_string,
            tag4_value,
            tag5_string,
            tag6_object: FgdbRasterIoObject10_1::Unsupported { class_id },
            tag7_value: None,
            end_offset: None,
            stop: Some(FgdbRasterIoStop10_1::UnsupportedObjectClass { class_id }),
        });
    }

    let (tag7, tag7_length) = read_raster_io_record_header_10_1(&mut trial)?;
    let tag7_value = if tag7 == -1 && tag7_length == -1 {
        None
    } else if tag7 == 7 && tag7_length == 4 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("RasterIO tag 7/End 记录不匹配：tag={tag7} length={tag7_length}"),
        ));
    };

    let mut stop = None;
    if tag7_value.is_some() {
        let before_end = trial.clone();
        let (end_tag, end_length) = read_raster_io_record_header_10_1(&mut trial)?;
        if end_tag != -1 || end_length != -1 {
            trial = before_end;
            stop = Some(FgdbRasterIoStop10_1::TrailingRecord {
                tag: end_tag,
                length: end_length,
            });
        }
    }
    let end_offset = stop.is_none().then_some(trial.position());
    let result = FgdbRasterIoPrefix10_1 {
        tag1_value,
        tag2_string,
        tag3_string,
        tag4_value,
        tag5_string,
        tag6_object: FgdbRasterIoObject10_1::Null,
        tag7_value,
        end_offset,
        stop,
    };
    *reader = trial;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::raster_dataset_name::RasterDatasetWorkspaceSlot10_1;

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

    fn body_with_null_workspace() -> Vec<u8> {
        let mut bytes = 4_i32.to_le_bytes().to_vec();
        for text in ["one", "two", "three", ""] {
            bytes.extend(encoded_string(text));
        }
        bytes.extend([0; 16]);
        bytes.extend(0x1122_3344_i32.to_le_bytes());
        bytes.extend(0x5566_7788_i32.to_le_bytes());
        bytes
    }

    #[test]
    fn exposes_rasterio_candidate_after_complete_delegated_name_body() {
        let mut bytes = body_with_null_workspace();
        let rasterio_start = bytes.len();
        // RasterIO InitCheck tag/length pair；用于标记委派体结束边界。
        bytes.extend(0x00ff_00fa_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();

        let parsed =
            read_fgdb_raster_dataset_name_prefix_10_1(&mut reader, 0x1000, &mut table, 260)
                .unwrap();

        assert_eq!(parsed.delegated_name.key, 260);
        assert_eq!(parsed.delegated_name.version_gate, 4);
        assert_eq!(
            parsed.delegated_name.strings.as_ref().unwrap(),
            &["one", "two", "three", ""]
        );
        assert!(matches!(
            parsed.delegated_name.workspace,
            Some(RasterDatasetWorkspaceSlot10_1::Null)
        ));
        assert_eq!(parsed.rasterio_candidate_offset, Some(rasterio_start));
        assert_eq!(reader.position(), rasterio_start);
        assert_eq!(reader.remaining(), 8);
        assert!(table.is_empty());
    }

    #[test]
    fn failed_delegated_body_does_not_advance_reader_or_reference_table() {
        let mut bytes = body_with_null_workspace();
        bytes.truncate(bytes.len() - 1);
        let mut reader = Reader::new(&bytes);
        let before = reader.clone();
        let mut table = RefTable10_1::new();

        assert!(
            read_fgdb_raster_dataset_name_prefix_10_1(&mut reader, 0x1000, &mut table, 260)
                .is_err()
        );
        assert_eq!(reader.position(), before.position());
        assert!(table.is_empty());
    }

    #[test]
    fn class_id_matches_the_10_1_get_class_id_memory_bytes() {
        assert_eq!(
            FGDB_RASTER_DATASET_NAME_CLASS_ID,
            [
                0x6f, 0x7c, 0x13, 0xf2, 0x75, 0x90, 0xaf, 0x4b, 0x9b, 0x38, 0xe2, 0xf3, 0x60, 0x83,
                0xfb, 0xf4,
            ]
        );
    }

    fn push_record(bytes: &mut Vec<u8>, tag: i32, payload: &[u8]) {
        bytes.extend(tag.to_le_bytes());
        bytes.extend((payload.len() as i32).to_le_bytes());
        bytes.extend(payload);
    }

    fn raster_io_null_object_records() -> Vec<u8> {
        let mut bytes = Vec::new();
        push_record(&mut bytes, RASTER_IO_MAGIC_10_1, &[]);
        push_record(&mut bytes, 1, &17_i32.to_le_bytes());
        push_record(&mut bytes, 2, &[b'a', 0, 0, 0]);
        push_record(&mut bytes, 3, &[b'b', 0, 0, 0]);
        push_record(&mut bytes, 4, &23_i32.to_le_bytes());
        push_record(&mut bytes, 5, &[b'c', 0, 0, 0]);
        bytes.extend(6_i32.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(NULL_CLASS_ID_10_1);
        push_record(&mut bytes, 7, &29_i32.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        bytes
    }

    #[test]
    fn reads_raster_io_records_through_null_object_and_end_pair() {
        let bytes = raster_io_null_object_records();
        let mut reader = Reader::new(&bytes);

        let parsed = read_fgdb_raster_io_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.tag1_value, 17);
        assert_eq!(parsed.tag2_string, "a");
        assert_eq!(parsed.tag3_string, "b");
        assert_eq!(parsed.tag4_value, 23);
        assert_eq!(parsed.tag5_string, "c");
        assert_eq!(parsed.tag6_object, FgdbRasterIoObject10_1::Null);
        assert_eq!(parsed.tag7_value, Some(29));
        assert_eq!(parsed.end_offset, Some(bytes.len()));
        assert_eq!(parsed.stop, None);
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn unsupported_raster_io_object_class_stops_before_class_id() {
        let mut bytes = raster_io_null_object_records();
        let class_id_at = bytes
            .windows(16)
            .position(|window| window == NULL_CLASS_ID_10_1)
            .unwrap();
        bytes[class_id_at] = 1;
        let mut reader = Reader::new(&bytes);

        let parsed = read_fgdb_raster_io_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert!(matches!(
            parsed.tag6_object,
            FgdbRasterIoObject10_1::Unsupported { .. }
        ));
        assert!(matches!(
            parsed.stop,
            Some(FgdbRasterIoStop10_1::UnsupportedObjectClass { .. })
        ));
        assert_eq!(reader.position(), class_id_at);
    }

    #[test]
    fn malformed_raster_io_record_is_transactional() {
        let mut bytes = raster_io_null_object_records();
        bytes[8..12].copy_from_slice(&3_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let before = reader.clone();

        assert!(read_fgdb_raster_io_prefix_10_1(&mut reader, 0x1000).is_err());
        assert_eq!(reader.position(), before.position());
    }
}
