//! `display_symbol`（DisplayCore.dll 简单线条/填充符号族）严格读取器的合成
//! 输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-display-tails.md`：五类符号的字段序来自
//! 10.1 原生 DLL 反编译，读写两侧对称闭合；仓库参考样本无这五类实例
//! （无字节佐证）。本文件全部使用合成输入，仅验证读取器自身契约（正常/
//! 异常/截断/事务性/嵌套深度），不代表 MXD 格式已验证，未在目标软件中
//! 核对。

use mxd2qgz::display_color::{
    CMYK_COLOR_CLASS_ID, ColorSlot10_1, RGB_COLOR_CLASS_ID, read_color_slot_10_1,
};
use mxd2qgz::display_symbol::{
    CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID, HASH_LINE_SYMBOL_CLASS_ID, MARKER_LINE_SYMBOL_CLASS_ID,
    MAX_SYMBOL_NESTING_DEPTH, SIMPLE_FILL_SYMBOL_CLASS_ID, SIMPLE_LINE_SYMBOL_CLASS_ID,
    SymbolStop10_1, read_cartographic_line_symbol_body_10_1, read_display_symbol_slot_10_1,
    read_hash_line_symbol_body_10_1, read_marker_line_symbol_body_10_1,
    read_simple_fill_symbol_body_10_1, read_simple_line_symbol_body_10_1,
};
use mxd2qgz::layers::feature_layer_tail::SIMPLE_MARKER_SYMBOL_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// 追加对象条目头（16 字节类标识 + 4 字节键）。
fn push_entry(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 追加 16 字节空标记。
fn push_null(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&[0_u8; 16]);
}

/// RgbColor 体（版本 1/1，flag==0）。
fn rgb_color_body() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.push(0);
    for value in [1.0_f64, 2.0, 3.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(0);
    bytes.push(0xFF);
    bytes
}

/// SimpleMarkerSymbol 体（版本 1 最小形）：颜色双槽空标记 + 标量段。
fn sms_body_v1() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null(&mut bytes); // 主色
    bytes.extend(6.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(13_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    for value in [0.0_f64; 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(1);
    bytes.extend(1.0_f64.to_le_bytes());
    push_null(&mut bytes); // 轮廓色
    bytes
}

/// CmykColor 体（版本 4，无串分支）。
fn cmyk_color_body() -> Vec<u8> {
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    bytes.push(0); // byte → this-0x10
    bytes.push(0); // 标志 0：无串段
    bytes.extend(0x0302_0100_u32.to_le_bytes());
    bytes.push(0x21);
    bytes.push(0x43);
    bytes
}

#[test]
fn simple_line_symbol_full_body_and_nested_colors() {
    // 版本 1 + 空颜色槽 + 标量段：完整消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    bytes.extend(2.0_f64.to_le_bytes());
    bytes.extend(4_i32.to_le_bytes());
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend(6_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result =
        read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 11).unwrap();
    assert_eq!(result.key, 11);
    assert_eq!(result.version, 1);
    assert_eq!(result.color, ColorSlot10_1::Null);
    assert_eq!(result.width, Some(2.0));
    assert_eq!(result.style, Some(4));
    assert_eq!(result.int32_7c, Some(5));
    assert_eq!(result.int32_74, Some(6));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 颜色槽为 RgbColor 新对象（键 37）：嵌套解析，其余完整。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 37);
    bytes.extend(&rgb_color_body());
    bytes.extend(3.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result =
        read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 12).unwrap();
    match &result.color {
        ColorSlot10_1::Color { class_id, body } => {
            assert_eq!(*class_id, RGB_COLOR_CLASS_ID);
            match body {
                mxd2qgz::display_color::ColorBody10_1::Rgb(body) => assert_eq!(body.key, 37),
                other => panic!("期望 Rgb 体，得到 {other:?}"),
            }
        }
        other => panic!("期望颜色体，得到 {other:?}"),
    }
    assert_eq!(result.width, Some(3.0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 颜色槽为 CmykColor 新对象（键 60）：同样闭合。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &CMYK_COLOR_CLASS_ID, 60);
    bytes.extend(&cmyk_color_body());
    bytes.extend(1.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result =
        read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 13).unwrap();
    assert!(matches!(
        result.color,
        ColorSlot10_1::Color {
            class_id: CMYK_COLOR_CLASS_ID,
            body: mxd2qgz::display_color::ColorBody10_1::Cmyk(_)
        }
    ));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn simple_line_symbol_unknown_color_stops_at_slot() {
    // 未知颜色类：停止于槽位起点，标量不可达，键不保留。
    let unknown = [0xcd_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &unknown, 700);
    bytes.extend(9.0_f64.to_le_bytes()); // 不得被消费
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut table, 14).unwrap();
    match result.stop {
        Some(SymbolStop10_1::Slot {
            kind,
            class_id,
            key,
        }) => {
            assert_eq!(kind, mxd2qgz::display_symbol::DisplaySlotKind10_1::Color);
            assert_eq!(class_id, unknown);
            assert_eq!(key, 700);
        }
        other => panic!("期望 Slot 停止，得到 {other:?}"),
    }
    assert_eq!(result.width, None);
    assert_eq!(result.style, None);
    // 游标停在槽位起点（版本 short 之后）。
    assert_eq!(reader.position(), 2);
    assert!(table.is_empty(), "未知键不保留");
}

#[test]
fn simple_line_symbol_rejects_versions_and_truncation() {
    for bad in [2_u16, 0x8000_u16] {
        let mut bytes = bad.to_le_bytes().to_vec();
        push_null(&mut bytes);
        let mut reader = Reader::new(&bytes);
        let error =
            read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1)
                .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "bad={bad}");
        assert_eq!(reader.position(), 0);
    }
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 37);
    bytes.extend(&rgb_color_body());
    bytes.extend(2.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let first =
            read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1);
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        let second =
            read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1);
        assert_eq!(
            first.unwrap_err().kind(),
            second.unwrap_err().kind(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn simple_fill_symbol_nests_line_symbol_outline() {
    // 版本 1 + 轮廓槽为 SimpleLineSymbol 新对象（键 21，内嵌 RgbColor）+
    // 填充色空标记 + 标量段：完整消费。
    let mut outline = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut outline, &RGB_COLOR_CLASS_ID, 41);
    outline.extend(&rgb_color_body());
    outline.extend(4.0_f64.to_le_bytes());
    outline.extend(0_i32.to_le_bytes());
    outline.extend(0_i32.to_le_bytes());
    outline.extend(0_i32.to_le_bytes());

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &SIMPLE_LINE_SYMBOL_CLASS_ID, 21);
    bytes.extend(&outline);
    push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 42); // 填充色
    bytes.extend(&rgb_color_body());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());

    let mut reader = Reader::new(&bytes);
    let result = read_simple_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        20,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    match &result.outline {
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::SimpleLineSymbol(body) => {
            assert_eq!(body.key, 21);
            assert_eq!(body.width, Some(4.0));
            assert_eq!(body.stop, None);
        }
        other => panic!("期望嵌套 SimpleLineSymbol，得到 {other:?}"),
    }
    match &result.color {
        Some(ColorSlot10_1::Color { class_id, .. }) => assert_eq!(*class_id, RGB_COLOR_CLASS_ID),
        other => panic!("期望填充色体，得到 {other:?}"),
    }
    assert_eq!(result.style, Some(0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    // 嵌套键已登记（21 = 轮廓符号，41/42 = 颜色）；外层对象键 20 由调用者
    // 登记，体读取器不代登记。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_simple_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 20, MAX_SYMBOL_NESTING_DEPTH)
        .unwrap();
    for key in [21_u32, 41, 42] {
        assert!(table.contains(key), "键 {key} 应已登记");
    }
    assert_eq!(table.len(), 3);
}

#[test]
fn simple_fill_symbol_unknown_outline_stops() {
    let unknown = [0x5e_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &unknown, 800);
    push_null(&mut bytes); // 填充色槽不得被消费
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_simple_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        15,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(matches!(
        result.outline,
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::Unsupported { key: 800, .. }
    ));
    assert_eq!(result.color, None);
    assert_eq!(result.style, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::display_symbol::DisplaySlotKind10_1::Symbol,
            key: 800,
            ..
        })
    ));
    // 游标停在轮廓槽位起点；键不保留。
    assert_eq!(reader.position(), 2);
    assert!(table.is_empty());
}

#[test]
fn marker_line_symbol_versions_and_unclosed_slots() {
    // 版本 2（含尾段）：三个槽位全空标记，完整消费。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.push(1); // byte → this-0x20d8
    bytes.extend(10.0_f64.to_le_bytes());
    push_null(&mut bytes); // IMarkerSymbol
    push_null(&mut bytes); // ITemplate 候选
    push_null(&mut bytes); // ILineDecoration 候选
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.push(0);
    bytes.extend(3.0_f64.to_le_bytes());
    bytes.extend(4_i32.to_le_bytes());
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend(6.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        30,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.version, 2);
    assert_eq!(result.byte_20d8, 1);
    assert_eq!(result.double_20d4, 10.0);
    assert_eq!(
        result.marker_symbol,
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::Null
    );
    assert_eq!(
        result.template,
        Some(mxd2qgz::display_symbol::DisplaySymbolSlot10_1::Null)
    );
    assert_eq!(
        result.line_decoration,
        Some(mxd2qgz::display_symbol::DisplaySymbolSlot10_1::Null)
    );
    assert_eq!(result.int32_2094, Some(4));
    assert_eq!(result.double_208c, Some(6.0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 1：无尾段，完整消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend(10.0_f64.to_le_bytes());
    push_null(&mut bytes);
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.push(0);
    bytes.extend(3.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        31,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.int32_2094, None);
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // ITemplate 候选槽出现新对象（合成未知类，代表未闭合 coclass）：
    // 停止于槽位起点，装饰槽与标量不可达，键不保留。
    let unclosed = [0xe1_u8; 16];
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend(10.0_f64.to_le_bytes());
    push_null(&mut bytes);
    push_entry(&mut bytes, &unclosed, 900);
    bytes.extend(1_i32.to_le_bytes()); // 不得被消费
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        32,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.template, None);
    assert_eq!(result.line_decoration, None);
    assert_eq!(result.int32_2134, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::display_symbol::DisplaySlotKind10_1::Template,
            key: 900,
            ..
        })
    ));
    // 游标停在模板槽位起点：版本(2)+byte(1)+double(8)+空标记(16) 之后。
    assert_eq!(reader.position(), 2 + 1 + 8 + 16);
    assert!(table.is_empty());

    // 版本 3 被拒。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    let mut reader = Reader::new(&bytes);
    let error = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        1,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn marker_line_symbol_nests_marker_symbol() {
    // IMarkerSymbol 槽为 SimpleMarkerSymbol 新对象（键 36，样本同款键值）。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.push(0);
    bytes.extend(1.0_f64.to_le_bytes());
    push_entry(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 36);
    bytes.extend(&sms_body_v1());
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.push(0);
    bytes.extend(0.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        33,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    match &result.marker_symbol {
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::SimpleMarkerSymbol(body) => {
            assert_eq!(body.key, 36);
            assert_eq!(body.stop, None);
        }
        other => panic!("期望嵌套 SimpleMarkerSymbol，得到 {other:?}"),
    }
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn cartographic_line_symbol_full_body_and_truncation() {
    let mut make = || {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.push(1);
        bytes.extend(2.5_f64.to_le_bytes());
        push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 43);
        bytes.extend(&rgb_color_body());
        push_null(&mut bytes); // ITemplate 候选
        push_null(&mut bytes); // ILineDecoration 候选
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        bytes.push(1);
        bytes.extend(0.5_f64.to_le_bytes());
        bytes.extend(1.0_f64.to_le_bytes());
        bytes
    };
    let bytes = make();
    let mut reader = Reader::new(&bytes);
    let result = read_cartographic_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        40,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.int32_10a8, 1);
    assert_eq!(result.double_109c, 1.5);
    assert_eq!(result.byte_1094, 1);
    assert_eq!(result.double_108c, 2.5);
    assert!(matches!(&result.color, Some(ColorSlot10_1::Color { .. })));
    assert_eq!(
        result.template,
        Some(mxd2qgz::display_symbol::DisplaySymbolSlot10_1::Null)
    );
    assert_eq!(result.int32_10fc, Some(3));
    assert_eq!(result.double_1054, Some(1.0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 逐字节截断扫描。
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let first = read_cartographic_line_symbol_body_10_1(
            &mut reader,
            1024,
            &mut RefTable10_1::new(),
            40,
            MAX_SYMBOL_NESTING_DEPTH,
        );
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_cartographic_line_symbol_body_10_1(
            &mut reader,
            1024,
            &mut RefTable10_1::new(),
            40,
            MAX_SYMBOL_NESTING_DEPTH,
        );
        assert_eq!(
            first.unwrap_err().kind(),
            second.unwrap_err().kind(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn hash_line_symbol_full_body_and_nested_symbol() {
    // ISymbol 槽为 SimpleLineSymbol 新对象（键 22），颜色槽空标记，
    // 模板/装饰槽空标记：完整消费。
    let mut nested = 1_u16.to_le_bytes().to_vec();
    push_null(&mut nested);
    nested.extend(5.0_f64.to_le_bytes());
    nested.extend(0_i32.to_le_bytes());
    nested.extend(0_i32.to_le_bytes());
    nested.extend(0_i32.to_le_bytes());

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(7.0_f64.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(3.0_f64.to_le_bytes());
    bytes.push(0);
    bytes.extend(4.0_f64.to_le_bytes());
    push_entry(&mut bytes, &SIMPLE_LINE_SYMBOL_CLASS_ID, 22);
    bytes.extend(&nested);
    push_null(&mut bytes); // IColor
    push_null(&mut bytes); // ITemplate 候选
    push_null(&mut bytes); // ILineDecoration 候选
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend(6_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(1.0_f64.to_le_bytes());
    bytes.extend(2.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_hash_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        50,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    match &result.symbol {
        Some(mxd2qgz::display_symbol::DisplaySymbolSlot10_1::SimpleLineSymbol(body)) => {
            assert_eq!(body.key, 22);
            assert_eq!(body.width, Some(5.0));
        }
        other => panic!("期望嵌套 SimpleLineSymbol，得到 {other:?}"),
    }
    assert_eq!(result.color, Some(ColorSlot10_1::Null));
    assert_eq!(result.int32_217c, Some(5));
    assert_eq!(result.double_210c, Some(2.0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 逐字节截断扫描。
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let first = read_hash_line_symbol_body_10_1(
            &mut reader,
            1024,
            &mut RefTable10_1::new(),
            50,
            MAX_SYMBOL_NESTING_DEPTH,
        );
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn slot_dispatcher_reports_unsupported_and_depth_limit() {
    // 未知符号类：Unsupported + 回退 + 键不保留。
    let unknown = [0x99_u8; 16];
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &unknown, 910);
    bytes.extend([0x11; 8]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    match read_display_symbol_slot_10_1(&mut reader, 1024, &mut table, MAX_SYMBOL_NESTING_DEPTH)
        .unwrap()
    {
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::Unsupported { class_id, key } => {
            assert_eq!(class_id, unknown);
            assert_eq!(key, 910);
        }
        other => panic!("期望 Unsupported，得到 {other:?}"),
    }
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());

    // 深度限额：连续嵌套 SimpleFillSymbol 超过 16 层报 InvalidInput。
    // 构造深度 17 的嵌套（每层轮廓槽为下一层 SimpleFillSymbol；最内层轮廓
    // 槽为空标记，其后每层补填充色槽与标量段）。
    let mut bytes = Vec::new();
    let mut level_key = 1_000_u32;
    for _ in 0..17 {
        push_entry(&mut bytes, &SIMPLE_FILL_SYMBOL_CLASS_ID, level_key);
        level_key += 1;
        bytes.extend(1_u16.to_le_bytes()); // 版本 1
    }
    // 最内层轮廓槽：空标记 + 颜色空标记 + 标量段（合法最小体）。
    push_null(&mut bytes);
    push_null(&mut bytes);
    for _ in 0..3 {
        bytes.extend(0_i32.to_le_bytes());
    }
    // 16 个外层：轮廓体结束后回到外层的填充色槽与标量段。
    for _ in 0..16 {
        push_null(&mut bytes);
        for _ in 0..3 {
            bytes.extend(0_i32.to_le_bytes());
        }
    }
    let mut reader = Reader::new(&bytes);
    let error = read_display_symbol_slot_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);

    // 恰好 16 层（最内层为合法体）应当成功：16 层头 + 最内层体 + 15 个
    // 外层尾段。
    let mut bytes = Vec::new();
    let mut level_key = 2_000_u32;
    for _ in 0..16 {
        push_entry(&mut bytes, &SIMPLE_FILL_SYMBOL_CLASS_ID, level_key);
        level_key += 1;
        bytes.extend(1_u16.to_le_bytes());
    }
    push_null(&mut bytes);
    push_null(&mut bytes);
    for _ in 0..3 {
        bytes.extend(0_i32.to_le_bytes());
    }
    for _ in 0..15 {
        push_null(&mut bytes);
        for _ in 0..3 {
            bytes.extend(0_i32.to_le_bytes());
        }
    }
    let mut reader = Reader::new(&bytes);
    let slot = read_display_symbol_slot_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(matches!(
        slot,
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::SimpleFillSymbol(_)
    ));
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn sms_stop_propagates_as_nested_incomplete() {
    // 嵌套 SimpleMarkerSymbol 体的颜色槽为未知类 → SMS 停止 →
    // MarkerLineSymbol 记录 NestedIncomplete，标量不可达。
    let unknown = [0x71_u8; 16];
    let mut sms = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut sms, &unknown, 920);
    sms.extend(6.0_f64.to_le_bytes()); // SMS 停止后不可达

    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.push(0);
    bytes.extend(1.0_f64.to_le_bytes());
    push_entry(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 36);
    bytes.extend(&sms); // SMS 完整体：版本 1 + 未知颜色条目 + 尾随 double
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.push(0);
    bytes.extend(0.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        60,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    match &result.marker_symbol {
        mxd2qgz::display_symbol::DisplaySymbolSlot10_1::SimpleMarkerSymbol(body) => {
            // 停止原因携带未知颜色条的类标识与键（该条目键 920）。注意：前轮
            // feature_layer_tail 的 SMS 首色槽停止路径中，体结构 `key` 字段被
            // 阴影为颜色条键（已知问题，见研究记录），本测试不断言该字段。
            assert!(body.stop.is_some(), "SMS 应在其颜色槽停止");
            match body.stop {
                Some(
                    mxd2qgz::layers::feature_layer_tail::SimpleMarkerSymbolStop10_1::ColorUnsupported {
                        class_id, key,
                    },
                ) => {
                    assert_eq!(class_id, unknown);
                    assert_eq!(key, 920);
                }
                other => panic!("期望 ColorUnsupported 停止，得到 {other:?}"),
            }
        }
        other => panic!("期望嵌套 SMS，得到 {other:?}"),
    }
    assert_eq!(result.template, None);
    assert_eq!(result.int32_2134, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::NestedIncomplete {
            class_id: SIMPLE_MARKER_SYMBOL_CLASS_ID,
            ..
        })
    ));
    // 游标停在内层停止点：头(2+1+8) + SMS 头(20) + SMS 版本(2) = 33。
    assert_eq!(reader.position(), 33);
    // 登记表保留到停止点为止的键（36，原生登记先于持久化）；外层键 60 由
    // 调用者登记，体读取器不代登记。
    assert!(table.contains(36));
    assert_eq!(table.len(), 1);
}

#[test]
fn direct_reader_calls_stay_transactional_on_the_table() {
    // 直接调用各体读取器的截断输入：错误路径不得修改登记表。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 41);
    let body = rgb_color_body();
    bytes.extend(&body[..body.len() - 2]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error = read_simple_line_symbol_body_10_1(&mut reader, 1024, &mut table, 9).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());

    // 对照：颜色槽直读同样事务性（键 41 未被登记）。
    let mut reader = Reader::new(&bytes[2..]);
    let error = read_color_slot_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}
