//! `map_surround`（MapSurround/书签非空循环元素体）严格读取器的合成输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-mapsurround-bookmark.md`：全部字段序来自
//! Map.dll 10.1.0.3035 静态反编译。仓库参考样本的两循环计数均为 0，这些
//! 测试只覆盖代码自身契约（合成正常/异常输入），不代表格式已验证，未在
//! 目标软件或样本中核对。

use mxd2qgz::envelope::ENVELOPE_CLASS_ID;
use mxd2qgz::map_surround::{
    AOI_BOOKMARK_CLASS_ID, BookmarkEnvelopeEntry10_1, BookmarkNameEntry10_1,
    FEATURE_BOOKMARK_CLASS_ID, LEGEND_CLASS_ID, LEGEND_FORMAT_CLASS_ID, LegendFormatEntry10_1,
    LegendStop10_1, MapBackReference10_1, read_aoi_bookmark_10_1, read_feature_bookmark_10_1,
    read_legend_prefix_10_1, read_map_surround_common_prefix_10_1,
};
use mxd2qgz::object_refs::RefTable10_1;
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

/// AfCore String 布局：4 字节长度（UTF-16LE 字节数含终止零）+ 编码 + 终止零。
fn string_bytes(text: &str) -> Vec<u8> {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut bytes = ((units.len() * 2 + 2) as u32).to_le_bytes().to_vec();
    for unit in units {
        bytes.extend(unit.to_le_bytes());
    }
    bytes.extend(0_u16.to_le_bytes());
    bytes
}

/// 追加对象条目头（16 字节类标识 + 4 字节键；回引与新对象同构）。
fn push_entry_header(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 追加 16 字节空标记。
fn push_null_marker(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&[0_u8; 16]);
}

/// 最小合法 Envelope 体（版本 3：32 字节坐标 + int32 0 + 空标记空间参考）。
fn minimal_envelope_body(coords: [f64; 4]) -> Vec<u8> {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    for value in coords {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(0_u32.to_le_bytes());
    push_null_marker(&mut bytes);
    bytes
}

/// 构造 MapSurround 共同前缀字节。
fn common_prefix_bytes(version: u16, name: &str, map_key: Option<u32>) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(string_bytes(name));
    match map_key {
        Some(key) => push_entry_header(&mut bytes, &LEGEND_CLASS_ID, key),
        None => push_null_marker(&mut bytes),
    }
    if version >= 2 {
        bytes.extend(0x1234_u16.to_le_bytes());
    }
    if version >= 3 {
        bytes.extend([0xAB_u8; 32]);
    }
    bytes
}

/// 预登记键的登记表（模拟「Map 先于装饰物持久化」的会话状态）。
fn table_with_keys(keys: &[u32]) -> RefTable10_1 {
    let mut table = RefTable10_1::new();
    for key in keys {
        table.register(*key);
    }
    table
}

#[test]
fn aoi_bookmark_reads_nested_envelope_body() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &ENVELOPE_CLASS_ID, 2);
    bytes.extend(minimal_envelope_body([1.0, 2.0, 3.0, 4.0]));
    bytes.extend(string_bytes("书签 1"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let bookmark = read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(bookmark.version, 1);
    assert_eq!(bookmark.name, "书签 1");
    assert_eq!(bookmark.remaining_bytes, 0);
    match &bookmark.envelope {
        BookmarkEnvelopeEntry10_1::Envelope { key, body } => {
            assert_eq!(*key, 2);
            assert_eq!(body.version, 3);
            assert_eq!(body.coords, [1.0, 2.0, 3.0, 4.0]);
        }
        other => panic!("应为内嵌 Envelope，得到 {other:?}"),
    }
    // 新键随成功读取提交（嵌套体先登记、整体成功才提交）。
    assert!(table.contains(2));
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn aoi_bookmark_reads_back_reference_envelope() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &ENVELOPE_CLASS_ID, 5);
    bytes.extend(string_bytes("回引"));
    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[5]);
    let bookmark = read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(
        bookmark.envelope,
        BookmarkEnvelopeEntry10_1::BackReference {
            class_id: ENVELOPE_CLASS_ID,
            key: 5
        }
    );
    assert_eq!(bookmark.name, "回引");
    assert_eq!(bookmark.remaining_bytes, 0);
}

#[test]
fn aoi_bookmark_accepts_version_zero_null_envelope_and_empty_name() {
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes(""));
    let mut reader = Reader::new(&bytes);
    let bookmark =
        read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new()).unwrap();
    assert_eq!(bookmark.version, 0);
    assert_eq!(bookmark.envelope, BookmarkEnvelopeEntry10_1::Null);
    assert_eq!(bookmark.name, "");
}

#[test]
fn aoi_bookmark_rejects_out_of_range_versions() {
    for version in [2_u16, 0x8000, 0xFFFF] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let position = reader.position();
        let mut table = table_with_keys(&[7]);
        let error = read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "版本 {version:#x}");
        assert_eq!(reader.position(), position, "失败不得推进游标");
        assert!(table.contains(7), "失败不得修改登记表");
    }
}

#[test]
fn aoi_bookmark_rejects_non_envelope_new_object() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &LEGEND_CLASS_ID, 3);
    let mut reader = Reader::new(&bytes);
    let position = reader.position();
    let mut table = RefTable10_1::new();
    let error = read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), position);
    assert!(!table.contains(3), "失败不得提交新键");
}

#[test]
fn aoi_bookmark_truncation_scan_keeps_cursor() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &ENVELOPE_CLASS_ID, 2);
    bytes.extend(minimal_envelope_body([1.0, 2.0, 3.0, 4.0]));
    bytes.extend(string_bytes("截断"));
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut table = RefTable10_1::new();
        assert!(
            read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut table).is_err(),
            "截断到 {cut} 字节应失败"
        );
        assert_eq!(reader.position(), 0, "截断失败不得推进游标（cut={cut}）");
        assert!(table.is_empty(), "截断失败不得修改登记表（cut={cut}）");
    }
    // 同输入重读：完整输入成功，结果一致。
    let mut reader = Reader::new(&bytes);
    let first = read_aoi_bookmark_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new()).unwrap();
    let second = read_aoi_bookmark_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut RefTable10_1::new(),
    )
    .unwrap();
    assert_eq!(first, second);
}

#[test]
fn feature_bookmark_reads_back_referenced_name_object() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend((-1_i32).to_le_bytes());
    push_entry_header(&mut bytes, &FEATURE_BOOKMARK_CLASS_ID, 3);
    bytes.extend(string_bytes("要素书签"));
    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[3]);
    let bookmark = read_feature_bookmark_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(bookmark.version, 1);
    assert_eq!(bookmark.raw_int32, -1);
    assert_eq!(
        bookmark.name_object,
        BookmarkNameEntry10_1::BackReference {
            class_id: FEATURE_BOOKMARK_CLASS_ID,
            key: 3
        }
    );
    assert_eq!(bookmark.name, "要素书签");
    assert_eq!(bookmark.remaining_bytes, 0);
}

#[test]
fn feature_bookmark_accepts_null_name_object() {
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes(""));
    let bookmark = read_feature_bookmark_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut RefTable10_1::new(),
    )
    .unwrap();
    assert_eq!(bookmark.name_object, BookmarkNameEntry10_1::Null);
    assert_eq!(bookmark.name, "");
}

#[test]
fn feature_bookmark_rejects_new_name_object_and_versions() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    push_entry_header(&mut bytes, &LEGEND_CLASS_ID, 4);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error = read_feature_bookmark_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);

    for version in [2_u16, 0x8000] {
        let error = read_feature_bookmark_10_1(
            &mut Reader::new(&version.to_le_bytes()),
            usize::MAX,
            &mut RefTable10_1::new(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }
}

#[test]
fn common_prefix_version_gates() {
    // 版本 0/1：无 short、无 32 字节段。
    for version in [0_u16, 1] {
        let bytes = common_prefix_bytes(version, "装饰物", None);
        let prefix = read_map_surround_common_prefix_10_1(
            &mut Reader::new(&bytes),
            usize::MAX,
            &mut RefTable10_1::new(),
        )
        .unwrap();
        assert_eq!(prefix.version, version);
        assert_eq!(prefix.name, "装饰物");
        assert_eq!(prefix.map_entry, None);
        assert_eq!(prefix.extra_short, None);
        assert_eq!(prefix.raw_envelope, None);
    }
    // 版本 2：仅追加 short。
    let bytes = common_prefix_bytes(2, "比例尺", Some(9));
    let prefix = read_map_surround_common_prefix_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut table_with_keys(&[9]),
    )
    .unwrap();
    assert_eq!(prefix.extra_short, Some(0x1234));
    assert_eq!(prefix.raw_envelope, None);
    assert_eq!(
        prefix.map_entry,
        Some(MapBackReference10_1 {
            class_id: LEGEND_CLASS_ID,
            key: 9
        })
    );
    // 版本 3：short + 32 字节原样。
    let bytes = common_prefix_bytes(3, "图例", Some(9));
    let prefix = read_map_surround_common_prefix_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut table_with_keys(&[9]),
    )
    .unwrap();
    assert_eq!(prefix.extra_short, Some(0x1234));
    assert_eq!(prefix.raw_envelope, Some([0xAB_u8; 32]));
}

#[test]
fn common_prefix_rejects_unverified_versions() {
    for version in [4_u16, 0x8000, 0xFFFF] {
        let bytes = common_prefix_bytes(version, "", None);
        let error = read_map_surround_common_prefix_10_1(
            &mut Reader::new(&bytes),
            usize::MAX,
            &mut RefTable10_1::new(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "版本 {version:#x}");
    }
}

#[test]
fn common_prefix_rejects_new_map_object() {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(string_bytes("孤儿"));
    push_entry_header(&mut bytes, &LEGEND_CLASS_ID, 29);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error =
        read_map_surround_common_prefix_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(!table.contains(29), "失败不得提交新键");
}

#[test]
fn legend_reads_full_version_five_body() {
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", Some(9)));
    push_entry_header(&mut bytes, &LEGEND_FORMAT_CLASS_ID, 10);
    bytes.extend(string_bytes("标题"));
    bytes.extend(0x0102_u16.to_le_bytes());
    bytes.extend(0x0304_u16.to_le_bytes());
    bytes.extend(0x0506_u16.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    // 计数 2：空标记 + 回引条目。
    bytes.extend(2_i32.to_le_bytes());
    push_null_marker(&mut bytes);
    push_entry_header(&mut bytes, &LEGEND_FORMAT_CLASS_ID, 10);
    // 版本 ≥3 / ≥4 的 short 与 ≥5 尾组。
    bytes.extend(0x0708_u16.to_le_bytes());
    bytes.extend(0x090A_u16.to_le_bytes());
    bytes.extend(0x0B0C_u16.to_le_bytes());
    bytes.extend(0x0D0E_u16.to_le_bytes());
    bytes.extend(0x0F10_u16.to_le_bytes());
    bytes.extend(4.5_f64.to_le_bytes());

    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[9, 10]);
    let legend = read_legend_prefix_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(legend.version, 5);
    assert_eq!(legend.stop, LegendStop10_1::Completed);
    assert_eq!(legend.remaining_bytes, 0);
    assert_eq!(legend.common.name, "图例");
    assert_eq!(
        legend.common.map_entry,
        Some(MapBackReference10_1 {
            class_id: LEGEND_CLASS_ID,
            key: 9
        })
    );
    assert_eq!(legend.common.extra_short, Some(0x1234));
    assert_eq!(legend.common.raw_envelope, Some([0xAB_u8; 32]));
    assert_eq!(
        legend.legend_format,
        LegendFormatEntry10_1::BackReference {
            class_id: LEGEND_FORMAT_CLASS_ID,
            key: 10
        }
    );
    assert_eq!(legend.title, "标题");
    assert_eq!(legend.legacy_doubles, None);
    assert_eq!(legend.shorts, [0x0102, 0x0304, 0x0506]);
    assert_eq!(legend.selection_short, 0xFFFF);
    assert_eq!(legend.item_count, 2);
    assert_eq!(legend.items.len(), 2);
    assert_eq!(
        legend.items[0].entry,
        mxd2qgz::object_refs::ObjectEntry10_1::Null
    );
    assert_eq!(legend.tail_short_1, Some(0x0708));
    assert_eq!(legend.tail_short_2, Some(0x090A));
    let group = legend.tail_group.expect("版本 5 应有尾组");
    assert_eq!(group.shorts, [0x0B0C, 0x0D0E, 0x0F10]);
    assert_eq!(group.value, 4.5);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn legend_version_one_reads_legacy_doubles_without_gates() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(0, "旧图例", None));
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes("t"));
    bytes.extend(1.5_f64.to_le_bytes());
    bytes.extend(2.5_f64.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());

    let legend = read_legend_prefix_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut RefTable10_1::new(),
    )
    .unwrap();
    assert_eq!(legend.stop, LegendStop10_1::Completed);
    assert_eq!(legend.legacy_doubles, Some([1.5, 2.5]));
    assert_eq!(legend.items, Vec::new());
    assert_eq!(legend.tail_short_1, None, "版本 1 不读 ≥3 的 short");
    assert_eq!(legend.tail_short_2, None);
    assert_eq!(legend.tail_group, None);
    assert_eq!(legend.remaining_bytes, 0);
}

#[test]
fn legend_stops_at_new_format_object_and_commits_key() {
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", Some(9)));
    push_entry_header(&mut bytes, &LEGEND_FORMAT_CLASS_ID, 11);
    bytes.extend(string_bytes("停止点之后保留为剩余"));

    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[9]);
    let legend = read_legend_prefix_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(
        legend.stop,
        LegendStop10_1::LegendFormatBody {
            offset: 2 + common_prefix_bytes(3, "图例", Some(9)).len(),
            class_id: LEGEND_FORMAT_CLASS_ID,
            key: 11
        }
    );
    assert_eq!(legend.items, Vec::new());
    // 已消费 20 字节对象头；字符串保留在剩余中。
    assert_eq!(
        legend.remaining_bytes,
        string_bytes("停止点之后保留为剩余").len()
    );
    assert_eq!(reader.position(), bytes.len() - legend.remaining_bytes);
    // 停止是成功返回：新键提交。
    assert!(table.contains(11));
    assert!(legend.stop.describe().contains("LegendFormat"));
}

#[test]
fn legend_stops_at_new_item_object() {
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", Some(9)));
    push_entry_header(&mut bytes, &LEGEND_FORMAT_CLASS_ID, 10);
    bytes.extend(string_bytes("标题"));
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    push_entry_header(&mut bytes, &LEGEND_CLASS_ID, 12);
    bytes.extend(string_bytes("条目体之后"));

    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[9, 10]);
    let legend = read_legend_prefix_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    let item_offset = bytes.len() - 20 - string_bytes("条目体之后").len();
    assert_eq!(
        legend.stop,
        LegendStop10_1::LegendItemBody {
            index: 0,
            offset: item_offset,
            class_id: LEGEND_CLASS_ID,
            key: 12
        }
    );
    assert_eq!(legend.items.len(), 1);
    assert_eq!(legend.items[0].offset, item_offset);
    assert_eq!(legend.tail_short_1, None, "停止后不读版本门尾段");
    assert!(table.contains(12), "停止是成功返回，条目新键提交");
}

#[test]
fn legend_rejects_version_overflow_negative_count_and_limit() {
    // 版本 6：原生 >5 拒绝（有符号），负值位模式一并拒绝。
    for version in [6_u16, 0x8000] {
        let bytes = version.to_le_bytes().to_vec();
        let error = read_legend_prefix_10_1(
            &mut Reader::new(&bytes),
            usize::MAX,
            &mut RefTable10_1::new(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "版本 {version:#x}");
    }

    // 负计数拒绝（原生负数跳过循环，此处拒绝避免静默错位）。
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", None));
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes("t"));
    bytes.extend([0_u8; 8]);
    bytes.extend((-1_i32).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let position = reader.position();
    let mut table = RefTable10_1::new();
    let error = read_legend_prefix_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), position, "失败不得推进游标");
    assert_eq!(table.len(), 0, "失败不得修改登记表");

    // 计数 × 最小条目字节数超过限额：读取条目之前拒绝。
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", None));
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes("t"));
    bytes.extend([0_u8; 8]);
    bytes.extend(2_i32.to_le_bytes());
    let error = read_legend_prefix_10_1(&mut Reader::new(&bytes), 31, &mut RefTable10_1::new())
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
}

#[test]
fn legend_string_limit_is_enforced() {
    // 标题字符串声明长度超过 max_bytes：字符串读取器拒绝且游标不变。
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", None));
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes("超长标题内容"));
    let mut reader = Reader::new(&bytes);
    let position = reader.position();
    let error = read_legend_prefix_10_1(&mut reader, 4, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), position);
}

#[test]
fn legend_truncation_scan_keeps_cursor_and_table() {
    let mut bytes = 5_u16.to_le_bytes().to_vec();
    bytes.extend(common_prefix_bytes(3, "图例", Some(9)));
    push_entry_header(&mut bytes, &LEGEND_FORMAT_CLASS_ID, 10);
    bytes.extend(string_bytes("标题"));
    bytes.extend([0_u8; 8]);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0_u8; 14]);

    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut table = table_with_keys(&[9, 10]);
        assert!(
            read_legend_prefix_10_1(&mut reader, usize::MAX, &mut table).is_err(),
            "截断到 {cut} 字节应失败"
        );
        assert_eq!(reader.position(), 0, "截断失败不得推进游标（cut={cut}）");
        assert_eq!(table.len(), 2, "截断失败不得修改登记表（cut={cut}）");
    }
    // 完整输入成功且与重读一致。
    let first = read_legend_prefix_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut table_with_keys(&[9, 10]),
    )
    .unwrap();
    let second = read_legend_prefix_10_1(
        &mut Reader::new(&bytes),
        usize::MAX,
        &mut table_with_keys(&[9, 10]),
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.stop, LegendStop10_1::Completed);
    assert_eq!(first.remaining_bytes, 0);
}

#[test]
fn class_id_constants_match_documented_clsids() {
    // 常量与 esri.clsid 声明/GetClassID 写出的 CLSID 逐字节对照
    // （内存序：Data1-3 小端、Data4 原序）。
    assert_eq!(
        AOI_BOOKMARK_CLASS_ID,
        [
            0x73, 0xae, 0xb1, 0xbb, 0x41, 0xe3, 0x11, 0xd2, 0xae, 0x1e, 0x08, 0x00, 0x09, 0xec,
            0x73, 0x2a
        ]
    );
    assert_eq!(
        FEATURE_BOOKMARK_CLASS_ID,
        [
            0x5b, 0xb3, 0x65, 0xec, 0x42, 0x43, 0x11, 0xd2, 0xae, 0x22, 0x08, 0x00, 0x09, 0xec,
            0x73, 0x2a
        ]
    );
    assert_eq!(
        LEGEND_CLASS_ID,
        [
            0xe4, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75,
            0x17, 0x20
        ]
    );
}
