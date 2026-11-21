//! `display_symbol`（DisplayCore.dll 简单线条/填充符号族）严格读取器的合成
//! 输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-display-tails.md`：五类符号的字段序来自
//! 10.1 原生 DLL 反编译，读写两侧对称闭合；仓库参考样本无这五类实例
//! （无字节佐证）。本文件全部使用合成输入，仅验证读取器自身契约（正常/
//! 异常/截断/事务性/嵌套深度），不代表 MXD 格式已验证，未在目标软件中
//! 核对。

use mxd2qgz::objects::display_color::{
    CMYK_COLOR_CLASS_ID, ColorSlot10_1, RGB_COLOR_CLASS_ID, read_color_slot_10_1,
};
use mxd2qgz::objects::display_symbol::{
    MAX_SYMBOL_NESTING_DEPTH, SIMPLE_FILL_SYMBOL_CLASS_ID, SIMPLE_LINE_SYMBOL_CLASS_ID,
    SIMPLE_MARKER_SYMBOL_CLASS_ID, SimpleMarkerSymbolStop10_1, SymbolStop10_1,
    read_cartographic_line_symbol_body_10_1, read_display_symbol_slot_10_1,
    read_hash_line_symbol_body_10_1, read_marker_line_symbol_body_10_1,
    read_simple_fill_symbol_body_10_1, read_simple_line_symbol_body_10_1,
};
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
                mxd2qgz::objects::display_color::ColorBody10_1::Rgb(body) => {
                    assert_eq!(body.key, 37)
                }
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
            body: mxd2qgz::objects::display_color::ColorBody10_1::Cmyk(_)
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
            assert_eq!(
                kind,
                mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::Color
            );
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
    bytes.extend(13_i32.to_le_bytes()); // ROP2
    bytes.extend(42_i32.to_le_bytes()); // MapLevel
    bytes.extend(3_i32.to_le_bytes()); // Style

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
        mxd2qgz::objects::display_symbol::SymbolSlotFull10_1::SimpleLineSymbol(body) => {
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
    assert_eq!(result.int32_7c, Some(13));
    assert_eq!(result.int32_74, Some(42));
    assert_eq!(result.style, Some(3));
    assert_eq!(result.int32_c, result.style);
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
        mxd2qgz::objects::display_symbol::SymbolSlotFull10_1::Unsupported { key: 800, .. }
    ));
    assert_eq!(result.color, None);
    assert_eq!(result.style, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::Symbol,
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
        mxd2qgz::objects::display_symbol::SymbolSlotFull10_1::Null
    );
    assert_eq!(
        result.template,
        Some(mxd2qgz::objects::display_symbol::DecorationSlot10_1::Null)
    );
    assert_eq!(
        result.line_decoration,
        Some(mxd2qgz::objects::display_symbol::DecorationSlot10_1::Null)
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
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::Template,
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
        mxd2qgz::objects::display_symbol::SymbolSlotFull10_1::SimpleMarkerSymbol(body) => {
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
    let make = || {
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
        Some(mxd2qgz::objects::display_symbol::DecorationSlot10_1::Null)
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
fn hash_line_reads_multilayer_line_and_respects_nested_depth() {
    use mxd2qgz::objects::display_symbol::MULTI_LAYER_LINE_SYMBOL_CLASS_ID;
    let mut bytes = 1u16.to_le_bytes().to_vec();
    bytes.extend([0; 33]); // 角度、端头、连接、宽度、翻转和偏移
    push_entry(&mut bytes, &MULTI_LAYER_LINE_SYMBOL_CLASS_ID, 22);
    bytes.extend(2u16.to_le_bytes());
    bytes.extend([0; 12]); // ROP2、MapLevel、零层
    for _ in 0..3 {
        push_null(&mut bytes);
    }
    bytes.extend([0; 25]); // 两个整数、一个标志和两个 double
    let mut table = RefTable10_1::new();
    let mut limited = Reader::new(&bytes);
    assert!(read_hash_line_symbol_body_10_1(&mut limited, 1024, &mut table, 1, 0).is_err());
    assert_eq!(limited.position(), 0);
    assert!(table.is_empty());
    let mut reader = Reader::new(&bytes);
    let body =
        read_hash_line_symbol_body_10_1(&mut reader, 1024, &mut table, 1, MAX_SYMBOL_NESTING_DEPTH)
            .unwrap();
    assert!(matches!(
        body.symbol,
        Some(SymbolSlotFull10_1::MultiLayerLineSymbol(_))
    ));
    assert!(body.stop.is_none());
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(22));
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
        Some(mxd2qgz::objects::display_symbol::SymbolSlotFull10_1::SimpleLineSymbol(body)) => {
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
        mxd2qgz::objects::display_symbol::DisplaySymbolSlot10_1::Unsupported { class_id, key } => {
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
        mxd2qgz::objects::display_symbol::DisplaySymbolSlot10_1::SimpleFillSymbol(_)
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
        mxd2qgz::objects::display_symbol::SymbolSlotFull10_1::SimpleMarkerSymbol(body) => {
            // 停止原因携带未知颜色条的类标识与键（该条目键 920）。
            assert!(body.stop.is_some(), "SMS 应在其颜色槽停止");
            match body.stop {
                Some(SimpleMarkerSymbolStop10_1::ColorUnsupported { class_id, key }) => {
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
// ===== 第二轮闭合类（docs/arcmap-10.1-display-symbols-round2.md）=====
//
// 全部为合成输入：样本 0 实例（无字节佐证），只验证读取器自身契约。

use mxd2qgz::objects::display_symbol::{
    DecorationSlot10_1, GRADIENT_FILL_SYMBOL_CLASS_ID, LINE_DECORATION_CLASS_ID,
    SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID, StdFontSlot10_1, StdPictureSlot10_1,
    SymbolSlotFull10_1, TEMPLATE_CLASS_ID, read_arrow_marker_symbol_body_10_1,
    read_character_marker_symbol_body_10_1, read_dot_density_fill_symbol_body_10_1,
    read_gradient_fill_symbol_body_10_1, read_line_decoration_body_10_1,
    read_line_fill_symbol_body_10_1, read_marker_fill_symbol_body_10_1,
    read_multi_layer_fill_symbol_body_10_1, read_multi_layer_line_symbol_body_10_1,
    read_multi_layer_marker_symbol_body_10_1, read_picture_line_symbol_body_10_1,
    read_picture_marker_symbol_body_10_1, read_simple_line_decoration_element_body_10_1,
    read_symbol_slot_full_10_1, read_template_body_10_1,
};

/// 追加 4 字节计数（int32 LE）。
fn push_count(bytes: &mut Vec<u8>, count: i32) {
    bytes.extend(count.to_le_bytes());
}

/// 追加 String（原生 4 字节长度[含终止零] + UTF-16LE 数据 + 终止零）。
fn push_string(bytes: &mut Vec<u8>, text: &str) {
    let raw: Vec<u8> = text.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    bytes.extend(((raw.len() + 2) as u32).to_le_bytes());
    bytes.extend_from_slice(&raw);
    bytes.extend_from_slice(&[0, 0]);
}

#[test]
fn template_body_full_consumption_and_limits() {
    // 版本 1 + double 间隔 + 2 个 16 字节标记：完整消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(2.5_f64.to_le_bytes());
    push_count(&mut bytes, 2);
    bytes.extend(1.0_f64.to_le_bytes());
    bytes.extend(90.0_f64.to_le_bytes());
    bytes.extend(2.0_f64.to_le_bytes());
    bytes.extend(45.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_template_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 71).unwrap();
    assert_eq!(result.key, 71);
    assert_eq!(result.version, 1);
    assert_eq!(result.interval, 2.5);
    assert_eq!(result.mark_count, 2);
    assert_eq!(result.marks, [[1.0, 90.0], [2.0, 45.0]]);
    assert_eq!(reader.position(), bytes.len());

    // 版本 2 拒绝；负计数拒绝；超限计数拒绝；逐字节截断事务性。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(0.0_f64.to_le_bytes());
    push_count(&mut bytes, 0);
    let mut reader = Reader::new(&bytes);
    let error =
        read_template_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0.0_f64.to_le_bytes());
    push_count(&mut bytes, -1);
    let mut reader = Reader::new(&bytes);
    let error =
        read_template_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0.0_f64.to_le_bytes());
    push_count(&mut bytes, 3);
    bytes.extend([0_u8; 16]);
    let mut reader = Reader::new(&bytes);
    let error = read_template_body_10_1(&mut reader, 2, &mut RefTable10_1::new(), 1).unwrap_err();
    assert_eq!(
        error.kind(),
        ErrorKind::InvalidData,
        "计数超 max_bytes 限额"
    );

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1.0_f64.to_le_bytes());
    push_count(&mut bytes, 1);
    bytes.extend([1_u8; 16]);
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let first = read_template_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1);
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn line_decoration_nests_element_and_stops_on_unknown() {
    // 版本 1 + 1 个 SimpleLineDecorationElement（键 81，符号槽为空标记，
    // 2 个位置 double）：完整消费。
    let mut element = 1_u16.to_le_bytes().to_vec();
    element.push(1);
    element.push(0);
    element.push(1);
    element.extend(0_i16.to_le_bytes());
    push_null(&mut element);
    push_count(&mut element, 2);
    element.extend(0.25_f64.to_le_bytes());
    element.extend(0.75_f64.to_le_bytes());

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID, 81);
    bytes.extend(&element);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_line_decoration_body_10_1(&mut reader, 1024, &mut table, 80, 1).unwrap();
    assert_eq!(result.element_count, 1);
    assert_eq!(result.elements.len(), 1);
    match &result.elements[0] {
        mxd2qgz::objects::display_symbol::LineDecorationElement10_1::Element(body) => {
            assert_eq!(body.key, 81);
            assert!(body.rotate());
            assert!(!body.flip_first());
            assert!(body.flip_all());
            assert!(!body.position_as_ratio());
            assert_eq!(body.positions.as_deref(), Some(&[0.25, 0.75][..]));
            assert_eq!(body.stop, None);
        }
        other => panic!("期望嵌套元素体，得到 {other:?}"),
    }
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(81));

    // 将元素内的空槽换成新标记，此时需要第二层限额。
    let mut nested = bytes[..2 + 4 + 20 + 7].to_vec();
    push_entry(&mut nested, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 82);
    nested.extend(sms_body_v1());
    push_count(&mut nested, 0);
    let mut reader = Reader::new(&nested);
    let mut table = RefTable10_1::new();
    let err = read_line_decoration_body_10_1(&mut reader, 1024, &mut table, 80, 1).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
    read_line_decoration_body_10_1(&mut reader, 1024, &mut table, 80, 2).unwrap();
    assert_eq!(reader.position(), nested.len());
    assert!(table.contains(81) && table.contains(82));

    // 元素为未知 ILineDecorationElement 实现类：停止于该条目起点，键不保留。
    let unknown = [0xa7_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_count(&mut bytes, 2);
    push_null(&mut bytes); // 首条目空标记可消费
    push_entry(&mut bytes, &unknown, 901);
    bytes.push(0xFF); // 不得被消费
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result =
        read_line_decoration_body_10_1(&mut reader, 1024, &mut table, 82, MAX_SYMBOL_NESTING_DEPTH)
            .unwrap();
    assert_eq!(result.elements.len(), 2);
    assert_eq!(
        result.elements[0],
        mxd2qgz::objects::display_symbol::LineDecorationElement10_1::Null
    );
    assert!(matches!(
        result.elements[1],
        mxd2qgz::objects::display_symbol::LineDecorationElement10_1::Unsupported { key: 901, .. }
    ));
    match result.stop {
        Some(SymbolStop10_1::Slot {
            kind,
            class_id,
            key,
        }) => {
            assert_eq!(
                kind,
                mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::LineDecorationElement
            );
            assert_eq!(class_id, unknown);
            assert_eq!(key, 901);
        }
        other => panic!("期望 Slot 停止，得到 {other:?}"),
    }
    // 游标停在未知条目起点：版本(2)+计数(4)+空标记(16) 之后。
    assert_eq!(reader.position(), 2 + 4 + 16);
    assert!(table.is_empty());
}

#[test]
fn sldelem_symbol_slot_stop_keeps_prefix() {
    // 符号槽未知类：停止于槽位起点，位置数组不可达。
    let unknown = [0x3c_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.push(0);
    bytes.push(0);
    bytes.extend(2_i16.to_le_bytes());
    push_entry(&mut bytes, &unknown, 910);
    bytes.extend(1.0_f64.to_le_bytes()); // 不得被消费
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_simple_line_decoration_element_body_10_1(
        &mut reader,
        1024,
        &mut table,
        85,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.byte_18, 1);
    assert_eq!(result.short_14, 2);
    assert!(matches!(
        result.symbol,
        Some(SymbolSlotFull10_1::Unsupported { key: 910, .. })
    ));
    assert_eq!(result.positions, None);
    assert!(result.stop.is_some());
    assert_eq!(reader.position(), 2 + 3 + 2);
    assert!(table.is_empty());
}

#[test]
fn multi_layer_line_symbol_layers_arrays_and_strings() {
    // 版本 2：1 个 SimpleLineSymbol 层（键 22）+ int 数组 + 串数组。
    let mut layer = 1_u16.to_le_bytes().to_vec();
    push_null(&mut layer);
    layer.extend(4.0_f64.to_le_bytes());
    layer.extend(0_i32.to_le_bytes());
    layer.extend(0_i32.to_le_bytes());
    layer.extend(0_i32.to_le_bytes());

    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &SIMPLE_LINE_SYMBOL_CLASS_ID, 22);
    bytes.extend(&layer);
    push_count(&mut bytes, 7);
    push_count(&mut bytes, 9);
    push_string(&mut bytes, "层");
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_multi_layer_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        60,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.version, 2);
    assert_eq!(result.int32_c4, 1);
    assert_eq!(result.int32_bc, 2);
    assert_eq!(result.layer_count, 1);
    assert!(matches!(
        &result.layers[0],
        SymbolSlotFull10_1::SimpleLineSymbol(body) if body.width == Some(4.0)
    ));
    assert_eq!(result.ints_60.as_deref(), Some(&[7][..]));
    assert_eq!(result.ints_4c.as_deref(), Some(&[9][..]));
    assert_eq!(result.strings.as_deref(), Some(&["层".to_string()][..]));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(22));

    // 版本 1：无串段（流内不存在），数组仍消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_count(&mut bytes, 0);
    let mut reader = Reader::new(&bytes);
    let result = read_multi_layer_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        61,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.strings, None);
    assert_eq!(result.ints_60.as_deref(), Some(&[][..]));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 未知层符号类：停止于该层条目起点，数组与串不可达。
    let unknown = [0xb1_u8; 16];
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &unknown, 911);
    bytes.push(0);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_multi_layer_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        62,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(matches!(
        &result.layers[0],
        SymbolSlotFull10_1::Unsupported { key: 911, .. }
    ));
    assert_eq!(result.ints_60, None);
    assert_eq!(result.strings, None);
    assert!(result.stop.is_some());
    assert_eq!(reader.position(), 2 + 4 + 4 + 4);
    assert!(table.is_empty());
}

#[test]
fn multi_layer_marker_symbol_v3_full_body() {
    // 版本 3：颜色空标记 + ISymbol 空标记 + 1 个 SMS 层（键 36）+ 数组 +
    // MarkerBackground 空标记 + 串。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    for value in [1.0_f64, 2.0, 3.0, 4.0] {
        bytes.extend(value.to_le_bytes());
    }
    push_null(&mut bytes); // IColor
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend(6.0_f64.to_le_bytes());
    push_null(&mut bytes); // ISymbol
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 36);
    bytes.extend(&sms_body_v1());
    push_count(&mut bytes, 1);
    push_count(&mut bytes, 2);
    push_null(&mut bytes); // IMarkerBackground
    push_string(&mut bytes, "T");

    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_multi_layer_marker_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        70,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.version, 3);
    assert_eq!(result.color, Some(ColorSlot10_1::Null));
    assert_eq!(result.int32_34, Some(5));
    assert_eq!(result.double_24, Some(6.0));
    assert_eq!(result.symbol, Some(SymbolSlotFull10_1::Null));
    assert_eq!(result.layer_count, 1);
    assert!(matches!(
        &result.layers[0],
        SymbolSlotFull10_1::SimpleMarkerSymbol(_)
    ));
    assert_eq!(result.ints_a8.as_deref(), Some(&[1][..]));
    assert_eq!(result.ints_94.as_deref(), Some(&[2][..]));
    assert_eq!(
        result.marker_background,
        Some(mxd2qgz::objects::callout_background::CalloutSlot10_1::Null)
    );
    assert_eq!(result.strings.as_deref(), Some(&["T".to_string()][..]));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(36));

    // 版本 2：MarkerBackground 槽存在但无串段。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    for _ in 0..2 {
        bytes.extend(0_i32.to_le_bytes());
    }
    for _ in 0..4 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    push_null(&mut bytes);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0.0_f64.to_le_bytes());
    push_null(&mut bytes);
    push_count(&mut bytes, 0);
    push_null(&mut bytes); // IMarkerBackground
    let mut reader = Reader::new(&bytes);
    let result = read_multi_layer_marker_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        71,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.strings, None);
    assert!(result.marker_background.is_some());
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 4 拒绝。
    let bytes = 4_u16.to_le_bytes().to_vec();
    let mut reader = Reader::new(&bytes);
    let error = read_multi_layer_marker_symbol_body_10_1(
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
fn multi_layer_fill_symbol_layers_and_strings() {
    // 版本 2：颜色 + 1 个 SimpleFillSymbol 层（键 21，轮廓与填充色空标记）
    // + 数组 + 串。
    let mut layer = 1_u16.to_le_bytes().to_vec();
    push_null(&mut layer);
    push_null(&mut layer);
    for _ in 0..3 {
        layer.extend(0_i32.to_le_bytes());
    }

    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    push_null(&mut bytes); // IColor
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &SIMPLE_FILL_SYMBOL_CLASS_ID, 21);
    bytes.extend(&layer);
    push_count(&mut bytes, 4);
    push_count(&mut bytes, 5);
    push_string(&mut bytes, "");

    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_multi_layer_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        72,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.color, Some(ColorSlot10_1::Null));
    assert_eq!(result.layer_count, 1);
    assert!(matches!(
        &result.layers[0],
        SymbolSlotFull10_1::SimpleFillSymbol(body) if body.stop.is_none()
    ));
    assert_eq!(result.ints_50.as_deref(), Some(&[4][..]));
    assert_eq!(result.ints_3c.as_deref(), Some(&[5][..]));
    assert_eq!(result.strings.as_deref(), Some(&[String::new()][..]));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(21));
}

#[test]
fn character_marker_symbol_version_gates() {
    // 版本 4：颜色空标记 + 标量 + 字体描述符（v≥3）+ 第二字体槽空标记
    // （v>3）；无 v<3 字体对象槽。
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    push_null(&mut bytes); // IColor
    bytes.extend(1_i32.to_le_bytes());
    for value in [10.0_f64, 1.0, 2.0, 90.0, 3.0, 4.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend(8_i32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes()); // v>1 short
    push_string(&mut bytes, "Arial"); // v≥3 字体名
    for value in [16_i32, 700, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(204_i16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    push_null(&mut bytes); // v>3 第二 IFontDisp 槽

    let mut reader = Reader::new(&bytes);
    let result =
        read_character_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 75)
            .unwrap();
    assert_eq!(result.version, 4);
    assert_eq!(result.color, Some(ColorSlot10_1::Null));
    assert_eq!(result.double_9c, 90.0);
    assert_eq!(result.font_slot_v3, None, "版本 ≥3 无 v<3 字体对象槽");
    assert_eq!(result.int32_124, Some(7));
    assert_eq!(result.int32_11c, Some(8));
    assert_eq!(result.short_10c, Some(1));
    assert_eq!(result.font_name.as_deref(), Some("Arial"));
    assert_eq!(result.font_ints_a, Some([16, 700, 0, 0]));
    assert_eq!(result.font_shorts, Some([0, 204]));
    assert_eq!(result.font_ints_b, Some([0, 1]));
    assert_eq!(result.font_size_cy(), Some(1));
    assert_eq!(result.font_slot_v5, Some(StdFontSlot10_1::Null));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 2：v<3 字体对象槽存在（空标记）；两个 int32 与 short 恒有；
    // 无字体描述符（≥3 不成立）；无第二槽（v>3 不成立）。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    bytes.extend(0_i32.to_le_bytes());
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    push_null(&mut bytes); // v<3 IFontDisp 槽
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(4_i32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result =
        read_character_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 76)
            .unwrap();
    assert_eq!(result.font_slot_v3, Some(StdFontSlot10_1::Null));
    assert_eq!(result.int32_124, Some(3));
    assert_eq!(result.int32_11c, Some(4));
    assert_eq!(result.short_10c, Some(1));
    assert_eq!(result.font_name, None);
    assert_eq!(result.font_slot_v5, None);
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 1：v<3 字体对象槽存在（实现类未闭合，以未知类停止）。
    let unknown = [0xf0_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    bytes.extend(0_i32.to_le_bytes());
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    push_entry(&mut bytes, &unknown, 920);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_character_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 77).unwrap();
    assert_eq!(result.font_slot_v3, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::Font,
            key: 920,
            ..
        })
    ));
    assert_eq!(reader.position(), 2 + 16 + 4 + 48);
    assert!(table.is_empty());
}

#[test]
fn picture_marker_symbol_v9_type_branches() {
    // 类型 1（路径 String + double）+ 共享尾段。
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_string(&mut bytes, "pic.png");
    bytes.extend(0.5_f64.to_le_bytes());
    for _ in 0..3 {
        push_null(&mut bytes); // IColor ×3
    }
    for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(0_i16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(0_i16.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 90)
            .unwrap();
    match &result.image {
        mxd2qgz::objects::display_symbol::PictureImage10_1::V9 {
            image_type,
            aux_int32,
            payload,
        } => {
            assert_eq!(*image_type, 1);
            assert_eq!(*aux_int32, 0);
            match payload {
                Some(mxd2qgz::objects::display_symbol::PictureImagePayload10_1::Path {
                    path,
                    double_value,
                }) => {
                    assert_eq!(path, "pic.png");
                    assert_eq!(*double_value, 0.5);
                }
                other => panic!("期望 Path 载荷，得到 {other:?}"),
            }
        }
        other => panic!("期望 V9 图像块，得到 {other:?}"),
    }
    assert_eq!(
        result.colors,
        [
            Some(ColorSlot10_1::Null),
            Some(ColorSlot10_1::Null),
            Some(ColorSlot10_1::Null)
        ]
    );
    assert_eq!(result.doubles, Some([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 类型 2（int32 N=5 + 5 字节位图数据）。
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
    push_null(&mut bytes);
    push_null(&mut bytes);
    push_null(&mut bytes);
    for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(0_i16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(0_i16.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 91)
            .unwrap();
    match &result.image {
        mxd2qgz::objects::display_symbol::PictureImage10_1::V9 {
            payload:
                Some(mxd2qgz::objects::display_symbol::PictureImagePayload10_1::Blob { size, data }),
            ..
        } => {
            assert_eq!(*size, 5);
            assert_eq!(data.as_deref(), Some(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE][..]));
        }
        other => panic!("期望 Blob 载荷，得到 {other:?}"),
    }

    // 类型 2 且 N=0：无流内数据字节。
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_null(&mut bytes);
    push_null(&mut bytes);
    push_null(&mut bytes);
    for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(0_i16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(0_i16.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 92)
            .unwrap();
    match &result.image {
        mxd2qgz::objects::display_symbol::PictureImage10_1::V9 {
            payload:
                Some(mxd2qgz::objects::display_symbol::PictureImagePayload10_1::Blob { size, data }),
            ..
        } => {
            assert_eq!(*size, 0);
            assert_eq!(*data, None);
        }
        other => panic!("期望空 Blob 载荷，得到 {other:?}"),
    }

    // 类型 4 拒绝。
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    bytes.extend(4_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 93)
            .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    // 类型 3 槽位为未知实现类：停止于槽位起点（载荷 None）。
    let unknown = [0x77_u8; 16];
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_entry(&mut bytes, &unknown, 930);
    bytes.push(0);
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 94)
            .unwrap();
    match &result.image {
        mxd2qgz::objects::display_symbol::PictureImage10_1::V9 { payload, .. } => {
            assert_eq!(*payload, None)
        }
        other => panic!("期望 V9 图像块，得到 {other:?}"),
    }
    assert!(result.stop.is_some());
    assert_eq!(reader.position(), 2 + 4 + 4);
}

#[test]
fn picture_marker_signed_legacy_version_keeps_the_successor_unread() {
    for version in [i16::MIN, -1, 0, 1] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 5 * 16 + 6 * 8 + 2 * 4]);
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 95)
                .unwrap();
        assert_eq!(parsed.version, version as u16);
        assert!(parsed.stop.is_none());
        assert!(parsed.byte_48.is_none() && parsed.short_134.is_none() && parsed.shorts.is_none());
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);
    }
}

#[test]
fn picture_marker_symbol_legacy_paths() {
    // 版本 8：String（非空 → double）→ 颜色 ×3 → 标量 → short ×3 → blob。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    push_string(&mut bytes, "pic");
    bytes.extend(1.5_f64.to_le_bytes());
    for _ in 0..3 {
        push_null(&mut bytes);
    }
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.push(1); // v>1 byte
    bytes.extend(0_i16.to_le_bytes()); // v>2 short
    for _ in 0..3 {
        bytes.extend(0_i16.to_le_bytes()); // v>4 short ×3
    }
    bytes.extend(3_i32.to_le_bytes()); // v8 blob 尺寸
    bytes.extend_from_slice(&[1, 2, 3]);
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 95)
            .unwrap();
    match &result.image {
        mxd2qgz::objects::display_symbol::PictureImage10_1::Legacy(legacy) => {
            assert_eq!(legacy.path.as_deref(), Some("pic"));
            assert_eq!(legacy.path_double, Some(1.5));
            assert_eq!(legacy.picture_a, None, "版本 ≥4 无第一槽");
            assert_eq!(legacy.picture_b, None, "路径非空无第二槽");
            assert_eq!(legacy.blob_size, Some(3));
            assert_eq!(legacy.blob_data.as_deref(), Some(&[1, 2, 3][..]));
        }
        other => panic!("期望历史图像块，得到 {other:?}"),
    }
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 2：无 String 段（v≥7 才有）；第一槽（v<4）与第二槽（v≤6）为空
    // 标记；无 short ×3（v>4 不成立）；无 blob。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    push_null(&mut bytes);
    for _ in 0..3 {
        push_null(&mut bytes);
    }
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(0_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 96)
            .unwrap();
    match &result.image {
        mxd2qgz::objects::display_symbol::PictureImage10_1::Legacy(legacy) => {
            assert_eq!(legacy.picture_a, Some(StdPictureSlot10_1::Null));
            assert_eq!(legacy.picture_b, Some(StdPictureSlot10_1::Null));
            assert_eq!(legacy.path, None);
            assert_eq!(legacy.blob_size, None);
        }
        other => panic!("期望历史图像块，得到 {other:?}"),
    }
    assert_eq!(result.stop, None);
    // 版本 2 不读 short_134（v>2 门），流止于 141−2。
    assert_eq!(reader.position(), bytes.len() - 2);

    // 版本 10 拒绝。
    let bytes = 10_u16.to_le_bytes().to_vec();
    let mut reader = Reader::new(&bytes);
    let error =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1)
            .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn picture_line_and_arrow_marker_accept_signed_versions() {
    for version in [i16::MIN, -1, 0, 1] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 1 + 2 * 8 + 16]);
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let body =
            read_picture_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 100)
                .unwrap();
        assert_eq!(body.version, version as u16);
        assert!(body.stop.is_none());
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);

        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 16 + 5 * 8 + 3 * 4]);
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let body =
            read_arrow_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 101)
                .unwrap();
        assert_eq!(body.version, version as u16);
        assert!(body.stop.is_none() && body.short_c4.is_none());
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);
    }
}

#[test]
fn picture_line_respects_the_outer_symbol_budget() {
    let mut line = mxd2qgz::objects::display_symbol::PICTURE_LINE_SYMBOL_CLASS_ID.to_vec();
    line.extend(5_u32.to_le_bytes());
    line.extend(1_u16.to_le_bytes());
    line.extend([0; 1 + 2 * 8]);
    let mut empty = line.clone();
    push_null(&mut empty);
    let mut reader = Reader::new(&empty);
    let mut refs = RefTable10_1::new();
    assert!(matches!(
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 1).unwrap(),
        SymbolSlotFull10_1::PictureLineSymbol(body) if body.stop.is_none()
    ));
    assert_eq!(reader.remaining(), 0);

    line.extend(mxd2qgz::objects::picture_fill_symbol::PICTURE_FILL_SYMBOL_CLASS_ID);
    line.extend(6_u32.to_le_bytes());
    let mut reader = Reader::new(&line);
    let mut refs = RefTable10_1::new();
    let error = read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 1).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn picture_line_symbol_slot_and_fill_family_bodies() {
    // PictureLineSymbol：图像填充槽空标记完整消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend(2.0_f64.to_le_bytes());
    bytes.extend(3.0_f64.to_le_bytes());
    push_null(&mut bytes);
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 100)
            .unwrap();
    assert_eq!(result.byte_30, 1);
    assert_eq!(
        result.picture_fill_symbol,
        Some(mxd2qgz::objects::picture_fill_symbol::PictureFillSymbolSlot10_1::Null)
    );
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 图像填充槽未知类：停止于槽位起点。
    let unknown = [0x19_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.push(0);
    bytes.extend(0.0_f64.to_le_bytes());
    bytes.extend(0.0_f64.to_le_bytes());
    push_entry(&mut bytes, &unknown, 940);
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_line_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 101)
            .unwrap();
    assert_eq!(result.picture_fill_symbol, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::PictureFillSymbol,
            key: 940,
            ..
        })
    ));
    assert_eq!(reader.position(), 2 + 1 + 8 + 8);

    // LineFillSymbol：颜色空标记 + 双 ISymbol 槽（SimpleLineSymbol 键 22 与
    // 空标记）+ 标量段。
    let mut nested = 1_u16.to_le_bytes().to_vec();
    push_null(&mut nested);
    nested.extend(5.0_f64.to_le_bytes());
    nested.extend(0_i32.to_le_bytes());
    nested.extend(0_i32.to_le_bytes());
    nested.extend(0_i32.to_le_bytes());

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    push_entry(&mut bytes, &SIMPLE_LINE_SYMBOL_CLASS_ID, 22);
    bytes.extend(&nested);
    push_null(&mut bytes);
    for value in [1.0_f64, 2.0, 3.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(4_i32.to_le_bytes());
    bytes.extend(5_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_line_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        102,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(matches!(
        result.symbol_a,
        Some(SymbolSlotFull10_1::SimpleLineSymbol(_))
    ));
    assert_eq!(result.symbol_b, Some(SymbolSlotFull10_1::Null));
    assert_eq!(result.doubles, Some([1.0, 2.0, 3.0]));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // MarkerFillSymbol：int32 + double ×4 + 颜色 + 双 ISymbol 槽 + 标量。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    for value in [1.0_f64, 2.0, 3.0, 4.0] {
        bytes.extend(value.to_le_bytes());
    }
    push_null(&mut bytes);
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(45.0_f64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_marker_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        103,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.color, Some(ColorSlot10_1::Null));
    assert_eq!(result.symbol_a, Some(SymbolSlotFull10_1::Null));
    assert_eq!(result.symbol_b, Some(SymbolSlotFull10_1::Null));
    assert_eq!(result.int32s, Some([1, 2]));
    assert_eq!(result.double_c, Some(45.0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // GradientFillSymbol：色带空标记 + 颜色 + 符号 + 标量，完整消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(30.0_f64.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(0.5_f64.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(0_i32.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let result = read_gradient_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        104,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(
        result.color_ramp,
        Some(mxd2qgz::objects::color_ramp::ColorRampSlot10_1::Null)
    );
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 色带未知类：停止于槽位起点。
    let unknown = [0x5f_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &unknown, 950);
    bytes.push(0);
    let mut reader = Reader::new(&bytes);
    let result = read_gradient_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        105,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert_eq!(result.color_ramp, None);
    assert_eq!(result.color, None);
    assert!(matches!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::ColorRamp,
            key: 950,
            ..
        })
    ));
    assert_eq!(reader.position(), 2);
}

#[test]
fn arrow_marker_symbol_short_and_color_stop() {
    // 版本 2：颜色空标记 + 标量 + 尾 short（流内位模式原样保留）。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    for value in [1.0_f64, 2.0, 3.0] {
        bytes.extend(value.to_le_bytes());
    }
    for value in [1_i32, 2, 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(4.0_f64.to_le_bytes());
    bytes.extend(5.0_f64.to_le_bytes());
    bytes.extend(0x0102_u16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result =
        read_arrow_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 110)
            .unwrap();
    assert_eq!(result.color, Some(ColorSlot10_1::Null));
    assert_eq!(result.double_8c, 1.0);
    assert_eq!(result.short_c4, Some(0x0102));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 1：无尾 short。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null(&mut bytes);
    for _ in 0..3 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    for _ in 0..3 {
        bytes.extend(0_i32.to_le_bytes());
    }
    for _ in 0..2 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let result =
        read_arrow_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 111)
            .unwrap();
    assert_eq!(result.short_c4, None);
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());

    // 版本 3 拒绝。
    let bytes = 3_u16.to_le_bytes().to_vec();
    let mut reader = Reader::new(&bytes);
    let error = read_arrow_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    // 颜色槽未知类：停止于槽位起点（标量不可达）。
    let unknown = [0xd3_u8; 16];
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_entry(&mut bytes, &unknown, 960);
    bytes.push(0);
    let mut reader = Reader::new(&bytes);
    let result =
        read_arrow_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 112)
            .unwrap();
    assert!(matches!(
        result.color,
        Some(ColorSlot10_1::Unsupported { key: 960, .. })
    ));
    assert!(result.stop.is_some());
    assert_eq!(reader.position(), 2);
}

#[test]
fn dot_density_fill_symbol_markers_values_and_color() {
    // 版本 1：颜色 + 符号空标记 + 标量 + 2 个 SMS 标记（键 36/37）+ 值数组
    // + 尾颜色。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend(2.0_f64.to_le_bytes());
    bytes.push(0);
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(4.0_f64.to_le_bytes());
    bytes.push(1);
    push_count(&mut bytes, 2);
    push_entry(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 36);
    bytes.extend(&sms_body_v1());
    push_entry(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 37);
    bytes.extend(&sms_body_v1());
    push_count(&mut bytes, 2);
    push_count(&mut bytes, 0);
    push_null(&mut bytes);

    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    // 限额为零仍可越过空轮廓槽，遇到首个新标记时整笔回滚。
    let error =
        read_dot_density_fill_symbol_body_10_1(&mut reader, 1024, &mut table, 120, 0).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
    let result = read_dot_density_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        120,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(!result.use_masking());
    assert_eq!(result.exclude_mask(), Some(true));
    assert_eq!(result.marker_count, 2);
    assert_eq!(result.markers.len(), 2);
    assert_eq!(result.values.as_deref(), Some(&[2, 0][..]));
    assert_eq!(result.color_b, Some(ColorSlot10_1::Null));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(36) && table.contains(37));

    // 标记数组内未知类：停止于该条目起点，值数组与尾颜色不可达。
    let unknown = [0xe4_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.push(0);
    bytes.extend(0.0_f64.to_le_bytes());
    bytes.push(0);
    push_null(&mut bytes);
    push_null(&mut bytes);
    for _ in 0..3 {
        bytes.extend(0_i32.to_le_bytes());
    }
    bytes.extend(0.0_f64.to_le_bytes());
    bytes.push(0);
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &unknown, 970);
    bytes.push(0);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_dot_density_fill_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        121,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(matches!(
        &result.markers[0],
        SymbolSlotFull10_1::Unsupported { key: 970, .. }
    ));
    assert_eq!(result.values, None);
    assert_eq!(result.color_b, None);
    assert!(result.stop.is_some());
    // 游标停在未知标记条目起点：2+1+8+1+16+16+12+8+1+4 之后。
    assert_eq!(reader.position(), 2 + 1 + 8 + 1 + 16 + 16 + 12 + 8 + 1 + 4);
    assert!(table.is_empty());
}

#[test]
fn full_dispatcher_resolves_new_classes_and_stops_unknown() {
    // 全闭合入口直接分派 GradientFillSymbol（对象条目头 + 体版本 + 体）。
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &GRADIENT_FILL_SYMBOL_CLASS_ID, 985);
    bytes.extend(1_u16.to_le_bytes());
    push_null(&mut bytes);
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend(0.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0.0_f64.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(0_i32.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let slot = read_symbol_slot_full_10_1(
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    assert!(matches!(slot, SymbolSlotFull10_1::GradientFillSymbol(_)));
    assert_eq!(reader.position(), bytes.len());

    // 未知类回退 + Unsupported。
    let unknown = [0xcc_u8; 16];
    let mut bytes = Vec::new();
    push_entry(&mut bytes, &unknown, 980);
    bytes.extend([0x22_u8; 4]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    match read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, MAX_SYMBOL_NESTING_DEPTH)
        .unwrap()
    {
        SymbolSlotFull10_1::Unsupported { class_id, key } => {
            assert_eq!(class_id, unknown);
            assert_eq!(key, 980);
        }
        other => panic!("期望 Unsupported，得到 {other:?}"),
    }
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}

#[test]
fn full_dispatcher_nests_template_and_line_decoration() {
    // Template 体（无嵌套对象）与 LineDecoration（内嵌元素）经全闭合入口
    // 相邻消费（对应 MarkerLine 的模板槽 + 装饰槽序）。
    let template_body = {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(1.0_f64.to_le_bytes());
        push_count(&mut bytes, 1);
        bytes.extend(0.5_f64.to_le_bytes());
        bytes.extend(0.0_f64.to_le_bytes());
        bytes
    };
    let mut element = 1_u16.to_le_bytes().to_vec();
    element.push(0);
    element.push(0);
    element.push(0);
    element.extend(0_i16.to_le_bytes());
    push_null(&mut element);
    push_count(&mut element, 0);

    let mut bytes = Vec::new();
    push_entry(&mut bytes, &TEMPLATE_CLASS_ID, 140);
    bytes.extend(&template_body);
    push_entry(&mut bytes, &LINE_DECORATION_CLASS_ID, 141);
    bytes.extend(1_u16.to_le_bytes()); // 体版本
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID, 142);
    bytes.extend(&element);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();

    let template =
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, MAX_SYMBOL_NESTING_DEPTH)
            .unwrap();
    match &template {
        SymbolSlotFull10_1::Template(body) => {
            assert_eq!(body.key, 140);
            assert_eq!(body.marks, [[0.5, 0.0]]);
        }
        other => panic!("期望 Template 体，得到 {other:?}"),
    }

    let decoration =
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, MAX_SYMBOL_NESTING_DEPTH)
            .unwrap();
    match &decoration {
        SymbolSlotFull10_1::LineDecoration(body) => {
            assert_eq!(body.key, 141);
            assert_eq!(body.elements.len(), 1);
            assert!(body.stop.is_none());
        }
        other => panic!("期望 LineDecoration 体，得到 {other:?}"),
    }
    assert_eq!(reader.position(), bytes.len());
    for key in [140_u32, 141, 142] {
        assert!(table.contains(key), "键 {key} 应已登记");
    }
}

#[test]
fn marker_line_body_parses_closed_template_and_decoration_slots() {
    // MarkerLineSymbol（第一轮体）的 ITemplate/ILineDecoration 槽在本轮起
    // 按类分派：槽内 Template/LineDecoration 新对象整体解析。
    let template_body = {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(1.0_f64.to_le_bytes());
        push_count(&mut bytes, 1);
        bytes.extend(0.5_f64.to_le_bytes());
        bytes.extend(0.0_f64.to_le_bytes());
        bytes
    };
    let mut element = 1_u16.to_le_bytes().to_vec();
    element.push(0);
    element.push(0);
    element.push(0);
    element.extend(0_i16.to_le_bytes());
    push_null(&mut element);
    push_count(&mut element, 0);

    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.push(0);
    bytes.extend(1.0_f64.to_le_bytes());
    push_null(&mut bytes); // IMarkerSymbol
    push_entry(&mut bytes, &TEMPLATE_CLASS_ID, 150);
    bytes.extend(&template_body);
    push_entry(&mut bytes, &LINE_DECORATION_CLASS_ID, 151);
    bytes.extend(1_u16.to_le_bytes()); // 体版本
    push_count(&mut bytes, 1);
    push_entry(&mut bytes, &SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID, 152);
    bytes.extend(&element);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.push(0);
    bytes.extend(3.0_f64.to_le_bytes());
    bytes.extend(4_i32.to_le_bytes());
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend(6.0_f64.to_le_bytes());

    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_marker_line_symbol_body_10_1(
        &mut reader,
        1024,
        &mut table,
        130,
        MAX_SYMBOL_NESTING_DEPTH,
    )
    .unwrap();
    match &result.template {
        Some(DecorationSlot10_1::Template(body)) => {
            assert_eq!(body.key, 150);
            assert_eq!(body.marks, [[0.5, 0.0]]);
        }
        other => panic!("期望 Template 槽体，得到 {other:?}"),
    }
    match &result.line_decoration {
        Some(DecorationSlot10_1::LineDecoration(body)) => {
            assert_eq!(body.key, 151);
            assert_eq!(body.elements.len(), 1);
        }
        other => panic!("期望 LineDecoration 槽体，得到 {other:?}"),
    }
    assert_eq!(result.int32_2134, Some(1));
    assert_eq!(result.int32_2094, Some(4));
    assert_eq!(result.double_208c, Some(6.0));
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    for key in [150_u32, 151, 152] {
        assert!(table.contains(key), "键 {key} 应已登记");
    }
}

// ---------------------------------------------------------------------------
// 整合接线轮：CharacterMarkerSymbol 字体槽与 PictureMarkerSymbol IPicture 槽
// 接 OLE StdFont/StdPicture 分派（`docs/arcmap-10.1-ole-stdfont.md`「接线建
// 议」1/2；样本 16 处 StdFont 在 CharacterMarker 字体槽语境、StdPicture 样本
// 0 实例——接线契约由合成测试固定）。
// ---------------------------------------------------------------------------

/// StdFont 体（ole_font 轮样本首例形态）：版本 1 + charset 134 + flags 0 +
/// weight 400 + 字号 120000 + 名字 4 字节（GBK「宋体」）。
fn std_font_body() -> Vec<u8> {
    [
        0x01, 0x86, 0x00, 0x00, 0x90, 0x01, 0xc0, 0xd4, 0x01, 0x00, 0x04, 0xcb, 0xce, 0xcc, 0xe5,
    ]
    .to_vec()
}

/// StdPicture 体（空图合法形态）：魔数 0x746c + 第二字 0 + 声明 0。
fn std_picture_body() -> Vec<u8> {
    let mut bytes = 0x746c_u16.to_le_bytes().to_vec();
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes
}

#[test]
fn character_marker_font_slot_v3_parses_stdfont_inline() {
    // 版本 2（<3）：v<3 IFontDisp 槽 = StdFont 新对象；槽位整体消费后宿主
    // 续读字段可达（int32 ×2 + short），体完整消费无停止。
    use mxd2qgz::objects::display_symbol::StdFontSlot10_1;
    use mxd2qgz::objects::ole_font::STDFONT_CLASS_ID;
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null(&mut bytes); // IColor 槽
    bytes.extend(0_i32.to_le_bytes()); // int32 → this−0xc0
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    push_entry(&mut bytes, &STDFONT_CLASS_ID, 76); // v<3 IFontDisp 槽
    bytes.extend(std_font_body());
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(4_i32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_character_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 75).unwrap();
    let StdFontSlot10_1::StdFont(font) = result.font_slot_v3.expect("槽位应存在") else {
        panic!("v<3 字体槽应为 StdFont 体");
    };
    assert_eq!(font.charset, Some(134));
    assert_eq!(result.int32_124, Some(3), "槽位整体消费后续段可达");
    assert_eq!(result.font_slot_v5, None);
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(76), "StdFont 键已登记");
}

#[test]
fn character_marker_font_slot_v5_parses_stdfont_inline() {
    // 版本 4（>3）：字体描述符段之后的第二 IFontDisp 槽 = StdFont 新对象，
    // 体完整消费（样本 16 处 StdFont 的宿主语境为 v≤3/v>3 两类字体槽；本
    // 用例取 v>3 重写门）。
    use mxd2qgz::objects::display_symbol::StdFontSlot10_1;
    use mxd2qgz::objects::ole_font::STDFONT_CLASS_ID;
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    push_null(&mut bytes); // IColor 槽
    bytes.extend(0_i32.to_le_bytes()); // int32 → this−0xc0
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend(8_i32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes()); // v>1 short
    push_string(&mut bytes, "Arial"); // v≥3 字体名
    for value in [16_i32, 700, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(204_i16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    push_entry(&mut bytes, &STDFONT_CLASS_ID, 77); // v>3 第二 IFontDisp 槽
    bytes.extend(std_font_body());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_character_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 75).unwrap();
    let StdFontSlot10_1::StdFont(font) = result.font_slot_v5.expect("槽位应存在") else {
        panic!("v>3 字体槽应为 StdFont 体");
    };
    assert_eq!(font.weight, 400);
    assert_eq!(result.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(77));
}

#[test]
fn character_marker_font_slot_unknown_class_stops() {
    // 字体槽未知实现类：停止回退到槽位起点、键不保留（与 TextSymbol 字体
    // 槽语义一致）。
    let unknown = [0x77_u8; 16];
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null(&mut bytes); // IColor 槽
    bytes.extend(0_i32.to_le_bytes()); // int32 → this−0xc0
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    let slot_offset = bytes.len();
    push_entry(&mut bytes, &unknown, 76); // v<3 IFontDisp 槽未知类
    bytes.extend([0_u8; 8]); // 占位
    let mut reader = Reader::new(&bytes);
    let result =
        read_character_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 75)
            .unwrap();
    assert_eq!(result.font_slot_v3, None, "未知类不出现在字段中");
    assert_eq!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::Font,
            class_id: unknown,
            key: 76,
        })
    );
    assert_eq!(reader.position(), slot_offset, "游标回退到槽位起点");
}

#[test]
fn picture_marker_type3_slot_parses_stdpicture_inline() {
    // 版本 9 图像类型 3：IPicture 槽 = StdPicture 新对象；槽位整体消费后
    // 共享尾段（IColor ×3 等）可达。
    use mxd2qgz::objects::display_symbol::{
        PictureImage10_1, PictureImagePayload10_1, StdPictureSlot10_1,
    };
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes()); // 图像类型 3
    bytes.extend(0_i32.to_le_bytes()); // aux int32
    // 固定流内 GUID 字节，避免与被测常量共用同一个转录错误。
    push_entry(
        &mut bytes,
        &[
            0x04, 0x52, 0xe3, 0x0b, 0x91, 0x8f, 0xce, 0x11, 0x9d, 0xe3, 0x00, 0xaa, 0x00, 0x4b,
            0xb8, 0x51,
        ],
        96,
    ); // IPicture 槽
    bytes.extend(std_picture_body());
    for _ in 0..3 {
        push_null(&mut bytes); // IColor ×3
    }
    for _ in 0..6 {
        bytes.extend(0.0_f64.to_le_bytes());
    }
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.push(0_u8); // byte
    bytes.extend(0_i16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(0_i16.to_le_bytes());
    }
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 95).unwrap();
    let PictureImage10_1::V9 {
        image_type,
        payload,
        ..
    } = &result.image
    else {
        panic!("期望 V9 图像块");
    };
    assert_eq!(*image_type, 3);
    let PictureImagePayload10_1::Picture(StdPictureSlot10_1::StdPicture(picture)) =
        payload.as_ref().expect("类型 3 应有载荷")
    else {
        panic!("IPicture 槽应为 StdPicture 体");
    };
    assert_eq!(picture.data_size, 0, "空图合法形态");
    assert_eq!(result.stop, None, "槽位整体消费后续段可达");
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(96));
}

#[test]
fn picture_marker_legacy_picture_slot_unknown_class_stops() {
    // 版本 <4 的第一个 IPicture 槽未知实现类：停止回退到槽位起点。
    let unknown = [0x88_u8; 16];
    let mut bytes = 3_u16.to_le_bytes().to_vec(); // 版本 3（<4：第一个槽）
    let slot_offset = bytes.len();
    push_entry(&mut bytes, &unknown, 96);
    bytes.extend([0_u8; 8]); // 占位
    let mut reader = Reader::new(&bytes);
    let result =
        read_picture_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 95)
            .unwrap();
    assert_eq!(
        result.stop,
        Some(SymbolStop10_1::Slot {
            kind: mxd2qgz::objects::display_symbol::DisplaySlotKind10_1::Picture,
            class_id: unknown,
            key: 96,
        })
    );
    assert_eq!(reader.position(), slot_offset);
}

fn chart_symbol_prefix(class_id: &[u8; 16], fields: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_entry(&mut bytes, class_id, 10);
    bytes.extend(1_u16.to_le_bytes());
    for value in [1.25_f64, 2.5, 3.75, 4.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(1);
    bytes.extend(5.5_f64.to_le_bytes());
    bytes.extend(fields);
    bytes
}

#[test]
fn chart_symbols_read_variant_fields_nested_objects_and_following_slot() {
    use mxd2qgz::objects::callout_background::{CalloutSlot10_1, LINE_CALLOUT_CLASS_ID};
    use mxd2qgz::objects::display_symbol::{
        BAR_CHART_SYMBOL_CLASS_ID, ChartSymbolFields10_1, PIE_CHART_SYMBOL_CLASS_ID,
        STACKED_CHART_SYMBOL_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };
    let mut bar_fields = vec![1, 0];
    bar_fields.extend(6.0_f64.to_le_bytes());
    bar_fields.extend(7.0_f64.to_le_bytes());
    let mut pie_fields = 17_i32.to_le_bytes().to_vec();
    pie_fields.extend([0, 1]);
    let mut stacked_fields = vec![1, 0, 1];
    stacked_fields.extend(8.0_f64.to_le_bytes());

    for (class, fields, expected, common_version) in [
        (
            BAR_CHART_SYMBOL_CLASS_ID,
            bar_fields,
            ChartSymbolFields10_1::Bar {
                flags: [1, 0],
                values: [6.0, 7.0],
            },
            2_u16,
        ),
        (
            PIE_CHART_SYMBOL_CLASS_ID,
            pie_fields,
            ChartSymbolFields10_1::Pie {
                value: 17,
                flags: [0, 1],
            },
            1_u16,
        ),
        (
            STACKED_CHART_SYMBOL_CLASS_ID,
            stacked_fields,
            ChartSymbolFields10_1::Stacked {
                flags: [1, 0, 1],
                value: 8.0,
            },
            2_u16,
        ),
    ] {
        let mut bytes = chart_symbol_prefix(&class, &fields);
        push_null(&mut bytes); // 轮廓
        bytes.extend(common_version.to_le_bytes());
        bytes.extend(19_i32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(14.0_f64.to_le_bytes());
        if common_version == 1 {
            push_entry(&mut bytes, &LINE_CALLOUT_CLASS_ID, 11);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 4 * 16]); // 三个符号槽和锚点槽
            bytes.extend([0; 6 * 8 + 4]);
        }
        bytes.extend(2_i32.to_le_bytes());
        push_entry(&mut bytes, &SIMPLE_FILL_SYMBOL_CLASS_ID, 12);
        bytes.extend(1_u16.to_le_bytes());
        push_null(&mut bytes); // 填充符号的轮廓
        push_null(&mut bytes); // 颜色
        bytes.extend([0; 12]);
        bytes.extend(101.0_f64.to_le_bytes());
        push_entry(&mut bytes, &SIMPLE_FILL_SYMBOL_CLASS_ID, 12);
        bytes.extend(51.0_f64.to_le_bytes());
        if common_version == 1 {
            push_entry(&mut bytes, &LINE_CALLOUT_CLASS_ID, 11);
        } else {
            push_null(&mut bytes);
        }
        let end = bytes.len();
        push_entry(&mut bytes, &class, 10);

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let slot = read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        let SymbolSlotFull10_1::ChartSymbol(body) = slot else {
            panic!("预期统计图符号，实际为 {slot:?}");
        };
        assert_eq!(reader.position(), end);
        assert_eq!(body.fields, expected);
        assert_eq!(body.initial_values, [1.25, 2.5, 3.75, 4.0]);
        assert_eq!(body.flag, 1);
        assert_eq!(body.value, 5.5);
        assert_eq!(body.common_integers, [19, 2]);
        assert_eq!(body.common_value, 14.0);
        assert!(matches!(
            body.entries[0].0,
            SymbolSlotFull10_1::SimpleFillSymbol(_)
        ));
        assert_eq!(body.entries[0].1, 101.0);
        assert_eq!(
            body.entries[1],
            (
                SymbolSlotFull10_1::BackReference {
                    class_id: SIMPLE_FILL_SYMBOL_CLASS_ID,
                    key: 12
                },
                51.0
            )
        );
        if common_version == 1 {
            assert!(matches!(
                body.legacy_callout,
                Some(CalloutSlot10_1::Line(_))
            ));
            assert!(matches!(
                body.marker_background,
                CalloutSlot10_1::BackReference { key: 11, .. }
            ));
        } else {
            assert_eq!(body.legacy_callout, None);
            assert_eq!(body.marker_background, CalloutSlot10_1::Null);
        }
        assert_eq!(
            read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).unwrap(),
            SymbolSlotFull10_1::BackReference {
                class_id: class,
                key: 10
            }
        );
        assert_eq!(reader.remaining(), 0);

        for length in 0..end {
            let mut reader = Reader::new(&bytes[..length]);
            let mut table = RefTable10_1::new();
            assert!(
                read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).is_err(),
                "截断位置 {length}"
            );
            assert_eq!(reader.position(), 0);
            assert!(table.is_empty());
        }
    }
}

#[test]
fn chart_symbol_rejects_negative_count_and_rewinds_unknown_background() {
    use mxd2qgz::objects::display_symbol::{
        PIE_CHART_SYMBOL_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };
    let mut bytes = chart_symbol_prefix(&PIE_CHART_SYMBOL_CLASS_ID, &[0; 6]);
    push_null(&mut bytes); // 轮廓
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend([0; 16]); // 公共标量段
    let count_offset = bytes.len();
    bytes.extend((-1_i32).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    assert_eq!(
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());

    bytes[count_offset..].copy_from_slice(&1_i32.to_le_bytes());
    push_entry(&mut bytes, &SIMPLE_FILL_SYMBOL_CLASS_ID, 12);
    bytes.extend(1_u16.to_le_bytes());
    push_null(&mut bytes);
    push_null(&mut bytes);
    bytes.extend([0; 12]);
    bytes.extend(10.0_f64.to_le_bytes());
    push_entry(&mut bytes, &[0xaa; 16], 90);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).unwrap(),
        SymbolSlotFull10_1::Unsupported {
            class_id: PIE_CHART_SYMBOL_CLASS_ID,
            key: 10,
        }
    );
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}

#[test]
fn raster_color_symbols_preserve_the_color_and_all_integer_fields() {
    use mxd2qgz::objects::display_symbol::{
        COLOR_SYMBOL_CLASS_ID, RASTER_RGB_SYMBOL_CLASS_ID, SymbolSlotFull10_1,
        read_symbol_slot_full_10_1,
    };

    for class in [COLOR_SYMBOL_CLASS_ID, RASTER_RGB_SYMBOL_CLASS_ID] {
        let mut bytes = Vec::new();
        push_entry(&mut bytes, &class, 41);
        bytes.extend(1_u16.to_le_bytes());
        push_entry(&mut bytes, &RGB_COLOR_CLASS_ID, 42);
        bytes.extend(rgb_color_body());
        for value in [13_i32, -5, 17] {
            bytes.extend(value.to_le_bytes());
        }
        let end = bytes.len();
        push_entry(&mut bytes, &class, 41);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = match read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).unwrap() {
            SymbolSlotFull10_1::ColorSymbol(body) if class == COLOR_SYMBOL_CLASS_ID => body,
            SymbolSlotFull10_1::RasterRgbSymbol(body) if class == RASTER_RGB_SYMBOL_CLASS_ID => {
                body
            }
            slot => panic!("符号类不符: {slot:?}"),
        };
        assert_eq!(
            (body.key, body.version, body.integers, body.stop),
            (41, 1, Some([13, -5, 17]), None)
        );
        assert!(matches!(body.color, ColorSlot10_1::Color { .. }));
        assert!(table.contains(42));
        assert_eq!(reader.position(), end);
        assert_eq!(
            read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).unwrap(),
            SymbolSlotFull10_1::BackReference {
                class_id: class,
                key: 41
            }
        );
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn raster_color_ramp_symbol_consumes_only_fields_for_its_version() {
    use mxd2qgz::objects::color_ramp::ColorRampSlot10_1;
    use mxd2qgz::objects::display_symbol::{
        COLOR_RAMP_SYMBOL_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };

    for version in 0_u16..=5 {
        let mut bytes = Vec::new();
        push_entry(&mut bytes, &COLOR_RAMP_SYMBOL_CLASS_ID, 41);
        bytes.extend(version.to_le_bytes());
        push_null(&mut bytes); // 颜色
        for value in [13_i32, -5, 17] {
            bytes.extend(value.to_le_bytes());
        }
        push_null(&mut bytes); // 渐变
        bytes.extend(30_u16.to_le_bytes());
        if version < 2 {
            bytes.extend(31_u16.to_le_bytes());
        }
        bytes.extend(32_i32.to_le_bytes());
        if version >= 3 {
            for value in [33_u16, 34, 35] {
                bytes.extend(value.to_le_bytes());
            }
        }
        if version >= 4 {
            bytes.extend((-36_i32).to_le_bytes());
        }
        if version >= 5 {
            bytes.extend(37_u16.to_le_bytes());
            bytes.extend(0xffff_u16.to_le_bytes());
        }
        let end = bytes.len();
        bytes.push(0xb1);
        let mut reader = Reader::new(&bytes);
        let SymbolSlotFull10_1::ColorRampSymbol(body) =
            read_symbol_slot_full_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 8).unwrap()
        else {
            panic!("预期颜色渐变符号");
        };
        assert_eq!((body.key, body.version, body.stop), (41, version, None));
        assert_eq!(body.integers, Some([13, -5, 17]));
        assert_eq!(body.ramp, Some(ColorRampSlot10_1::Null));
        let tail = body.tail.unwrap();
        assert_eq!(tail.first_short, 30);
        assert_eq!(tail.legacy_short, (version < 2).then_some(31));
        assert_eq!(tail.integer, 32);
        assert_eq!(tail.shorts_v3, (version >= 3).then_some([33, 34, 35]));
        assert_eq!(tail.integer_v4, (version >= 4).then_some(-36));
        assert_eq!(tail.shorts_v5, (version >= 5).then_some([37, 0xffff]));
        assert_eq!(reader.read_bytes(1).unwrap(), &[0xb1]);

        for length in 0..end {
            let mut reader = Reader::new(&bytes[..length]);
            let mut table = RefTable10_1::new();
            assert!(
                read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).is_err(),
                "版本 {version} 截断于 {length}"
            );
            assert_eq!(reader.position(), 0);
            assert!(table.is_empty());
        }
    }
}

#[test]
fn raster_color_ramp_symbol_stops_at_unrecognized_ramp() {
    use mxd2qgz::objects::display_symbol::{
        COLOR_RAMP_SYMBOL_CLASS_ID, DisplaySlotKind10_1, SymbolSlotFull10_1,
        read_symbol_slot_full_10_1, slot_full_stop_10_1,
    };

    let mut bytes = Vec::new();
    push_entry(&mut bytes, &COLOR_RAMP_SYMBOL_CLASS_ID, 41);
    bytes.extend(5_u16.to_le_bytes());
    push_null(&mut bytes);
    bytes.extend([0; 12]);
    let ramp_offset = bytes.len();
    push_entry(&mut bytes, &[0xe3; 16], 42);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let slot = read_symbol_slot_full_10_1(&mut reader, 1024, &mut table, 8).unwrap();
    assert_eq!(
        slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol),
        Some(SymbolStop10_1::NestedIncomplete {
            class_id: COLOR_RAMP_SYMBOL_CLASS_ID,
            key: 41
        })
    );
    let SymbolSlotFull10_1::ColorRampSymbol(body) = slot else {
        panic!("预期保留渐变符号前缀");
    };
    assert_eq!(
        body.stop,
        Some(SymbolStop10_1::Slot {
            kind: DisplaySlotKind10_1::ColorRamp,
            class_id: [0xe3; 16],
            key: 42
        })
    );
    assert!(body.tail.is_none());
    assert_eq!(reader.position(), ramp_offset);
    assert!(table.contains(41));
    assert!(!table.contains(42));
}

#[test]
fn url_picture_marker_preserves_strings_and_reads_the_embedded_marker() {
    use mxd2qgz::objects::display_symbol::{
        PICTURE_MARKER_SYMBOL_CLASS_ID, SymbolSlotFull10_1, URL_PICTURE_MARKER_SYMBOL_CLASS_ID,
        read_symbol_slot_full_10_1,
    };

    let mut prefix = Vec::new();
    push_entry(&mut prefix, &URL_PICTURE_MARKER_SYMBOL_CLASS_ID, 61);
    prefix.extend(1_u16.to_le_bytes());
    push_string(&mut prefix, "https://example.invalid/图标.png");
    push_string(&mut prefix, "image/png");
    for embedded in [false, true] {
        let mut bytes = prefix.clone();
        if embedded {
            push_entry(&mut bytes, &PICTURE_MARKER_SYMBOL_CLASS_ID, 62);
            bytes.extend(9_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes()); // 路径图像分支
            bytes.extend(0_i32.to_le_bytes());
            push_string(&mut bytes, "图.png");
            bytes.extend(0.5_f64.to_le_bytes());
            bytes.extend([0; 3 * 16 + 6 * 8 + 2 * 4 + 1 + 4 * 2]);
        } else {
            push_null(&mut bytes);
        }
        let end = bytes.len();
        bytes.push(0xd5);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let SymbolSlotFull10_1::UrlPictureMarkerSymbol(body) =
            read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 8).unwrap()
        else {
            panic!("预期 URL 图像标记符号");
        };
        assert_eq!((body.key, body.version, body.stop), (61, 1, None));
        assert_eq!(
            body.strings,
            ["https://example.invalid/图标.png", "image/png"]
        );
        assert!(if embedded {
            matches!(body.picture_marker, SymbolSlotFull10_1::PictureMarkerSymbol(nested) if nested.key == 62)
        } else {
            body.picture_marker == SymbolSlotFull10_1::Null
        });
        assert_eq!(refs.contains(62), embedded);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xd5]);

        for length in 0..end {
            let mut reader = Reader::new(&bytes[..length]);
            let mut refs = RefTable10_1::new();
            assert!(
                read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 8).is_err(),
                "截断于 {length}"
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    let slot_start = prefix.len();
    push_entry(&mut prefix, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 70);
    let mut reader = Reader::new(&prefix);
    let mut refs = RefTable10_1::new();
    let SymbolSlotFull10_1::UrlPictureMarkerSymbol(body) =
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 8).unwrap()
    else {
        panic!("预期保留 URL 图像标记符号前缀");
    };
    assert!(body.stop.is_some());
    assert_eq!(reader.position(), slot_start);
    assert!(!refs.contains(70));
}

#[test]
fn symbol_mimic_reads_empty_rule_and_rolls_back_on_unknown_rule() {
    use mxd2qgz::objects::display_symbol::{
        SYMBOL_MIMIC_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };

    let mut bytes = Vec::new();
    push_entry(&mut bytes, &SYMBOL_MIMIC_CLASS_ID, 11);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend((-23_i32).to_le_bytes());
    let rule_start = bytes.len();
    bytes.extend([0; 16]);
    bytes.extend(19_i32.to_le_bytes());

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).unwrap(),
        SymbolSlotFull10_1::SymbolMimic {
            key: 11,
            version: 1,
            leading_value: -23,
            rule: mxd2qgz::storage::object_refs::ObjectEntry10_1::Null,
            rule_body: None,
            trailing_value: 19,
        }
    );
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(11));
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let mut refs = RefTable10_1::new();
        assert!(read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).is_err());
        assert_eq!(reader.position(), 0);
        assert!(!refs.contains(11));
    }

    bytes.truncate(rule_start);
    push_entry(&mut bytes, &[0xff; 16], 12);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).unwrap(),
        SymbolSlotFull10_1::Unsupported {
            class_id: SYMBOL_MIMIC_CLASS_ID,
            key: 11,
        }
    );
    assert_eq!(reader.position(), 0);
    assert!(!refs.contains(11));
    assert!(!refs.contains(12));
}

#[test]
fn symbol_mimic_preserves_rule_body_and_reuses_rule_key() {
    use mxd2qgz::objects::display_symbol::{
        REPRESENTATION_RULE_CLASS_ID, RepresentationRuleBody10_1, RepresentationRuleOverride10_1,
        SYMBOL_MIMIC_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut bytes = Vec::new();
    push_entry(&mut bytes, &SYMBOL_MIMIC_CLASS_ID, 21);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_entry(&mut bytes, &REPRESENTATION_RULE_CLASS_ID, 22);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    let counts_start = bytes.len();
    for value in [-1_i32, 81, 0, 1, -2, 3] {
        bytes.extend(value.to_le_bytes());
    }
    push_string(&mut bytes, "覆盖值");
    for value in [2_i32, -2, 3, 82, 9] {
        bytes.extend(value.to_le_bytes());
    }
    let first_end = bytes.len();
    push_entry(&mut bytes, &SYMBOL_MIMIC_CLASS_ID, 23);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    push_entry(&mut bytes, &REPRESENTATION_RULE_CLASS_ID, 22);
    bytes.extend(10_i32.to_le_bytes());

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let SymbolSlotFull10_1::SymbolMimic {
        rule_body: Some(body),
        trailing_value: 9,
        ..
    } = read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).unwrap()
    else {
        panic!("未读出 RepresentationRule");
    };
    assert_eq!(
        body,
        RepresentationRuleBody10_1 {
            version: 1,
            effect_list_version: 2,
            effect_count: -1,
            effects: Vec::new(),
            leading_value: 81,
            layer_count: 0,
            layers: Vec::new(),
            override_count: 1,
            overrides: vec![RepresentationRuleOverride10_1 {
                ids: [-2, 3],
                value: "覆盖值".into(),
            }],
            order_count: 2,
            order_ids: vec![-2, 3],
            trailing_value: 82,
        }
    );
    assert_eq!(reader.position(), first_end);
    assert!(refs.contains(22));
    assert!(matches!(
        read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).unwrap(),
        SymbolSlotFull10_1::SymbolMimic {
            rule: ObjectEntry10_1::BackReference { key: 22, .. },
            rule_body: None,
            trailing_value: 10,
            ..
        }
    ));
    assert_eq!(reader.remaining(), 0);
    for end in counts_start..first_end {
        let mut reader = Reader::new(&bytes[..end]);
        let mut refs = RefTable10_1::new();
        assert!(read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).is_err());
        assert_eq!(reader.position(), 0);
        assert!(!refs.contains(22));
    }
    for count_index in [2] {
        let mut unsupported = bytes[..first_end].to_vec();
        let start = counts_start + 4 * count_index;
        unsupported[start..start + 4].copy_from_slice(&1_i32.to_le_bytes());
        let mut reader = Reader::new(&unsupported);
        let mut refs = RefTable10_1::new();
        assert!(matches!(
            read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 2).unwrap(),
            SymbolSlotFull10_1::Unsupported { key: 21, .. }
        ));
        assert_eq!(reader.position(), 0);
        assert!(!refs.contains(21));
        assert!(!refs.contains(22));
    }
}

#[test]
fn representation_rule_reads_fill_and_line_attribute_lists_and_keys() {
    use mxd2qgz::objects::display_symbol::{
        BASIC_FILL_SYMBOL_CLASS_ID, BASIC_LINE_SYMBOL_CLASS_ID, GRADIENT_PATTERN_CLASS_ID,
        LINE_PATTERN_CLASS_ID, LINE_STROKE_CLASS_ID, REPRESENTATION_RULE_CLASS_ID,
        SOLID_COLOR_PATTERN_CLASS_ID, SYMBOL_MIMIC_CLASS_ID, SymbolSlotFull10_1,
        read_symbol_slot_full_10_1,
    };
    use mxd2qgz::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for (symbol_class, style_class) in [
        (BASIC_FILL_SYMBOL_CLASS_ID, SOLID_COLOR_PATTERN_CLASS_ID),
        (BASIC_FILL_SYMBOL_CLASS_ID, LINE_PATTERN_CLASS_ID),
        (BASIC_FILL_SYMBOL_CLASS_ID, GRADIENT_PATTERN_CLASS_ID),
        (BASIC_LINE_SYMBOL_CLASS_ID, LINE_STROKE_CLASS_ID),
    ] {
        let mut bytes = Vec::new();
        push_entry(&mut bytes, &SYMBOL_MIMIC_CLASS_ID, 11);
        bytes.extend([1, 0, 0, 0, 0, 0]);
        push_entry(&mut bytes, &REPRESENTATION_RULE_CLASS_ID, 12);
        bytes.extend([1, 0, 2, 0]);
        for value in [0_i32, 0, 3] {
            bytes.extend(value.to_le_bytes());
        }

        push_entry(&mut bytes, &symbol_class, 13);
        bytes.extend([1, 0, 2, 0, 0, 0, 0, 0]);
        push_entry(&mut bytes, &style_class, 14);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend([0; 14]);
        bytes.extend(19_i32.to_le_bytes());
        bytes.extend(20_i32.to_le_bytes());

        push_entry(&mut bytes, &symbol_class, 15);
        bytes.extend([1, 0, 2, 0, 0, 0, 0, 0]);
        push_entry(&mut bytes, &style_class, 14);
        bytes.extend(21_i32.to_le_bytes());
        push_entry(&mut bytes, &symbol_class, 13);
        for value in [0_i32, 0, 0, 22] {
            bytes.extend(value.to_le_bytes());
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let SymbolSlotFull10_1::SymbolMimic {
            rule_body: Some(rule),
            trailing_value: 22,
            ..
        } = read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 4).unwrap()
        else {
            panic!("未读出符号层");
        };
        let attributes = if symbol_class == BASIC_FILL_SYMBOL_CLASS_ID {
            let first = rule.layers[0].fill.as_ref().unwrap();
            let second = rule.layers[1].fill.as_ref().unwrap();
            assert_eq!(first.trailing_value, 20);
            assert_eq!(second.trailing_value, 21);
            assert!(matches!(
                second.pattern,
                ObjectEntry10_1::BackReference { key: 14, .. }
            ));
            &first.pattern_body.as_ref().unwrap().attributes
        } else {
            let first = rule.layers[0].line.as_ref().unwrap();
            let second = rule.layers[1].line.as_ref().unwrap();
            assert_eq!(first.trailing_value, 20);
            assert_eq!(second.trailing_value, 21);
            assert!(matches!(
                second.stroke,
                ObjectEntry10_1::BackReference { key: 14, .. }
            ));
            &first.stroke_body.as_ref().unwrap().attributes
        };
        assert_eq!(
            attributes,
            &vec![(
                7,
                PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(
                    19_i32.to_le_bytes()
                ))
            )]
        );
        assert!(matches!(
            rule.layers[2].entry,
            ObjectEntry10_1::BackReference { key: 13, .. }
        ));
        assert_eq!(reader.remaining(), 0);

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = RefTable10_1::new();
            assert!(read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 4).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(14));
        }
    }
}

#[test]
fn representation_rule_reads_basic_marker_and_bounded_placement() {
    use mxd2qgz::objects::display_symbol::{
        BASIC_MARKER_SYMBOL_CLASS_ID, MARKER_PLACEMENT_CLASSES_10_1, REPRESENTATION_RULE_CLASS_ID,
        SYMBOL_MIMIC_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for placement_class in MARKER_PLACEMENT_CLASSES_10_1 {
        let mut bytes = Vec::new();
        push_entry(&mut bytes, &SYMBOL_MIMIC_CLASS_ID, 11);
        bytes.extend([1, 0, 0, 0, 0, 0]);
        push_entry(&mut bytes, &REPRESENTATION_RULE_CLASS_ID, 12);
        bytes.extend([1, 0, 2, 0]);
        for value in [0_i32, 0, 3] {
            bytes.extend(value.to_le_bytes());
        }
        let mut placement_size_offset = 0;
        for (key, version) in [(13, 1_u16), (14, 2), (15, 2)] {
            push_entry(&mut bytes, &BASIC_MARKER_SYMBOL_CLASS_ID, key);
            bytes.extend(version.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([1, 0, 0, 0, 0, 0]);
            if version == 2 {
                push_string(&mut bytes, "点上");
            }
            if key == 13 {
                bytes.extend([0; 8]);
            } else {
                let size = if key == 14 {
                    placement_size_offset = bytes.len();
                    26_u32
                } else {
                    20
                };
                bytes.extend(size.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
                push_entry(&mut bytes, &placement_class, 16);
                if key == 14 {
                    bytes.extend([1, 0, 0, 0, 0, 0]);
                }
            }
            bytes.extend(33_i32.to_le_bytes());
        }
        for value in [0_i32, 0, 0, 44] {
            bytes.extend(value.to_le_bytes());
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let SymbolSlotFull10_1::SymbolMimic {
            rule_body: Some(rule),
            trailing_value: 44,
            ..
        } = read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 4).unwrap()
        else {
            panic!("未读出标记符号层");
        };
        let markers: Vec<_> = rule
            .layers
            .iter()
            .map(|entry| entry.marker.as_ref().unwrap())
            .collect();
        assert_eq!(markers[0].placement_name, None);
        assert_eq!(markers[0].placement, ObjectEntry10_1::Null);
        assert_eq!(markers[1].placement_name.as_deref(), Some("点上"));
        assert_eq!(markers[1].placement_body.as_ref().unwrap().version, 1);
        assert!(matches!(
            markers[2].placement,
            ObjectEntry10_1::BackReference { key: 16, .. }
        ));
        assert_eq!(markers[2].trailing_value, 33);
        assert_eq!(reader.remaining(), 0);

        for bad_length in [25_u32, 27] {
            bytes[placement_size_offset..placement_size_offset + 4]
                .copy_from_slice(&bad_length.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            assert!(read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 4).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(16));
        }
    }
}

#[test]
fn representation_rule_reads_known_effects_and_shares_keys_with_marker_layers() {
    use mxd2qgz::objects::display_symbol::{
        BASIC_MARKER_SYMBOL_CLASS_ID, GEOMETRIC_EFFECT_CLASSES_10_1, REPRESENTATION_RULE_CLASS_ID,
        SYMBOL_MIMIC_CLASS_ID, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for effect_class in GEOMETRIC_EFFECT_CLASSES_10_1 {
        let mut bytes = Vec::new();
        push_entry(&mut bytes, &SYMBOL_MIMIC_CLASS_ID, 11);
        bytes.extend([1, 0, 0, 0, 0, 0]);
        push_entry(&mut bytes, &REPRESENTATION_RULE_CLASS_ID, 12);
        bytes.extend([1, 0, 2, 0]);
        bytes.extend(1_i32.to_le_bytes());
        push_string(&mut bytes, "效果");
        let size_offset = bytes.len();
        bytes.extend(26_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        push_entry(&mut bytes, &effect_class, 13);
        bytes.extend([1, 0, 0, 0, 0, 0]);
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        push_entry(&mut bytes, &BASIC_MARKER_SYMBOL_CLASS_ID, 14);
        bytes.extend([1, 0, 1, 0]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(20_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        push_entry(&mut bytes, &effect_class, 13);
        bytes.extend([1, 0, 0, 0, 0, 0]);
        bytes.extend([0; 8]);
        for value in [25_i32, 0, 0, 0, 44] {
            bytes.extend(value.to_le_bytes());
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let SymbolSlotFull10_1::SymbolMimic {
            rule_body: Some(rule),
            trailing_value: 44,
            ..
        } = read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 4).unwrap()
        else {
            panic!("未读出几何效果");
        };
        assert_eq!(rule.effects[0].name.as_deref(), Some("效果"));
        assert_eq!(rule.effects[0].body.as_ref().unwrap().declared_count, 0);
        let marker = rule.layers[0].marker.as_ref().unwrap();
        assert_eq!(marker.effects[0].name, None);
        assert_eq!(
            marker.effects[0].entry,
            ObjectEntry10_1::BackReference {
                class_id: effect_class,
                key: 13
            }
        );
        assert_eq!(marker.trailing_value, 25);
        assert_eq!(reader.remaining(), 0);

        bytes[size_offset..size_offset + 4].copy_from_slice(&27_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        assert!(read_symbol_slot_full_10_1(&mut reader, 1024, &mut refs, 4).is_err());
        assert_eq!(reader.position(), 0);
        assert!(!refs.contains(13));
    }
}
