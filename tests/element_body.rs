//! 10.1 元素类对象体（TextElement/MarkerElement/LineElement/TextSymbol）与
//! ElementProperties 共同前缀读取器的单元测试。
//!
//! 合成输入的字段序按 `docs/arcmap-10.1-element-bodies.md` 的反编译证据构造；
//! TextElement 全链（含 TextSymbol 到 ISymbol #2 停止点）另经仓库参考样本
//! /Maps 流首例（键 1613 @`0x18551`）逐字节对位复核（研究文档「样本对位」
//! 节，手工复核，非测试依赖）。仅验证读取器自身契约，不代表 MXD 格式规则已
//! 验证，未经目标软件人工检查。

use mxd2qgz::objects::display_color::{ColorBody10_1, ColorSlot10_1, RGB_COLOR_CLASS_ID};
use mxd2qgz::objects::display_symbol::{SIMPLE_FILL_SYMBOL_CLASS_ID, SymbolSlotFull10_1};
use mxd2qgz::objects::element_body::{
    CustomProperty10_1, FontSlot10_1, GeometrySlot10_1, LINE_ELEMENT_CLASS_ID,
    MARKER_ELEMENT_CLASS_ID, NestedEntry10_1, PointSlot10_1, TEXT_ELEMENT_CLASS_ID,
    TEXT_SYMBOL_CLASS_ID, read_element_body_of_class_10_1, read_line_element_body_10_1,
    read_marker_element_body_10_1, read_text_element_body_10_1, read_text_symbol_body_10_1,
};
use mxd2qgz::objects::geometry::{
    MULTIPOINT_CLASS_ID, POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID,
};
use mxd2qgz::objects::property_set::PropertyValue10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// AfCore String::Write 布局：u32 字节长度前缀（含终止零）+ UTF-16LE。
fn encoded(text: &str) -> Vec<u8> {
    let payload: Vec<_> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes
}

/// Variant 值：u16 VARTYPE + 载荷。
fn value_bytes(value_type: u16, payload: &[u8]) -> Vec<u8> {
    value_type
        .to_le_bytes()
        .into_iter()
        .chain(payload.iter().copied())
        .collect()
}

/// 16 字节对象头：类标识 + 键。
fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

/// 16 字节空标记。
fn null_entry() -> Vec<u8> {
    [0_u8; 16].to_vec()
}

/// RgbColor 体（flag 0 分支，31 字节）：双版本 1 + flag 0 + 3 double + 2 字节。
fn rgb_body(doubles: [f64; 3]) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1_u16.to_le_bytes());
    bytes.push(0);
    for value in doubles {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(0);
    bytes.push(0);
    bytes
}

/// ElementProperties 前缀（版本 6 路径，样本形态）：双 String + VT_EMPTY +
/// int16 + double + int32。
fn ep_prefix(version: u16) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(encoded(""));
    bytes.extend(encoded("文本"));
    if version < 2 {
        bytes.extend(0_i32.to_le_bytes());
    }
    if version < 4 {
        // LoadObject 槽位：本组装器统一附加空标记（回引/新对象场景见
        // element_properties_custom_property_slot_stop）。
        bytes.extend(null_entry());
        return bytes;
    }
    bytes.extend(value_bytes(0, &[]));
    if version > 2 {
        bytes.extend(0xffff_u16.to_le_bytes());
    }
    if version > 4 {
        bytes.extend(5393_f64.to_le_bytes());
    }
    if version > 5 {
        bytes.extend(6_i32.to_le_bytes());
    }
    bytes
}

/// TextSymbol 体前缀（版本 4 路径，样本形态）：字段 1..=13，随后由调用者按
/// 需附加 ISymbol #2 槽位。
fn ts_prefix(version: u16) -> Vec<u8> {
    // 本函数返回不含槽位的版本骨架（字段 1），IColor #1 槽位由组装函数附加。
    version.to_le_bytes().to_vec()
}

#[test]
fn constants_match_esri_clsid_and_memory_order() {
    assert_eq!(
        TEXT_ELEMENT_CLASS_ID,
        [
            0xd3, 0x34, 0x40, 0x20, 0xea, 0xf6, 0xd0, 0x11, 0x83, 0xad, 0x08, 0x00, 0x09, 0xb9,
            0x96, 0xcc
        ]
    );
    assert_eq!(
        MARKER_ELEMENT_CLASS_ID,
        [
            0x12, 0xd7, 0x0f, 0x53, 0x0c, 0xef, 0xd0, 0x11, 0x83, 0xa0, 0x08, 0x00, 0x09, 0xb9,
            0x96, 0xcc
        ]
    );
    assert_eq!(
        LINE_ELEMENT_CLASS_ID,
        [
            0xe1, 0xfb, 0xb7, 0x8a, 0x71, 0xd8, 0xd0, 0x11, 0x83, 0x89, 0x08, 0x00, 0x09, 0xb9,
            0x96, 0xcc
        ]
    );
    assert_eq!(
        TEXT_SYMBOL_CLASS_ID,
        [
            0x74, 0x3e, 0x5a, 0xb6, 0x93, 0x29, 0xd1, 0x11, 0x9a, 0x43, 0x00, 0x80, 0xc7, 0xec,
            0x5c, 0x96
        ]
    );
    assert_eq!(
        RGB_COLOR_CLASS_ID,
        [
            0x96, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9,
            0x96, 0xcc
        ]
    );
}

/// 构造样本形态的完整 TextElement 体（到 TextSymbol 的 ISymbol #2 新对象为
/// 止）——字段序与仓库样本键 1613 @`0x18551` 逐字节同构（值可不同）。
fn sample_shaped_te_body() -> Vec<u8> {
    let mut bytes = 8_u16.to_le_bytes().to_vec(); // TE 版本 8
    bytes.extend(ep_prefix(6)); // EP 版本 6：名称空、类型「文本」、VT_EMPTY、FFFF、5393.0、6
    bytes.extend(encoded("T35")); // 文本
    bytes.extend(1_i32.to_le_bytes()); // 布尔 int32
    // ITextSymbol 槽：TextSymbol 新对象（键 1614 形态）
    bytes.extend(entry(&TEXT_SYMBOL_CLASS_ID, 1614));
    // TextSymbol 体（版本 4）
    bytes.extend(ts_prefix(4));
    // 字段 2：IColor #1 = RgbColor 新对象
    bytes.extend(entry(&RGB_COLOR_CLASS_ID, 1615));
    bytes.extend(rgb_body([
        56.547_017_615_341_03,
        76.899_433_471_346_26,
        68.103_444_271_380_82,
    ]));
    // 字段 3..6：int32 ×3、byte ×2、double ×3、int32 ×2
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.push(0);
    bytes.push(0);
    for value in [0.0_f64; 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(13_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    // 字段 7：IColor #2 = RgbColor 新对象（全 0）
    bytes.extend(entry(&RGB_COLOR_CLASS_ID, 1616));
    bytes.extend(rgb_body([0.0; 3]));
    // 字段 8：double ×2、int32 ×2、double ×3、byte、double、int32、double、byte
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(100_f64.to_le_bytes());
    bytes.extend(100_f64.to_le_bytes());
    bytes.push(1);
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(90_f64.to_le_bytes());
    bytes.push(1);
    // 字段 9：16 字节原样 GUID（样本 = TextSymbol 自身 CLSID）
    bytes.extend(TEXT_SYMBOL_CLASS_ID);
    // 字段 10：ITextPath 空标记；字段 11：ISymbol #1 空标记
    bytes.extend(null_entry());
    bytes.extend(null_entry());
    // 字段 12：String（与外层文本相同）；字段 13：double、int32、double
    bytes.extend(encoded("T35"));
    bytes.extend(8_f64.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(1.5_f64.to_le_bytes());
    // 字段 14：ISymbol #2 = 未知类新对象（停止点）。接线轮起样本真实类
    // {7914E603}（SimpleFillSymbol）已闭合并整体解析——见
    // text_element_parses_sample_shaped_symbol2_simple_fill_symbol；本函数
    // 用合成未知类固定「体恰在符号槽 #2 头起点停止」的既有分段语义。
    bytes.extend(entry(&UNCLOSED_SYMBOL_CLASS_ID, 1593));
    bytes
}

/// 合成的未闭合符号类（区别于样本真实类 {7914E603}）。
const UNCLOSED_SYMBOL_CLASS_ID: [u8; 16] = [0x61; 16];

/// SimpleFillSymbol 体（版本 1，样本真实类 {7914E603} 的接线形态）：
/// 轮廓槽空标记 + 填充色 RgbColor（键 1600）+ int32 ×3。
fn simple_fill_symbol_body() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(null_entry()); // ISymbol 轮廓槽
    bytes.extend(entry(&RGB_COLOR_CLASS_ID, 1600));
    bytes.extend(rgb_body([1.0, 2.0, 3.0]));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

/// 样本形态 TextElement 体 + ISymbol #2 = SimpleFillSymbol 新对象（接线轮
/// 闭合后的推进形态）：符号 #2 体整体解析后 TextSymbol 继续到 IFontDisp
/// 空标记与版本门尾段，TextElement 继续到几何槽与尾段。
fn sample_shaped_te_body_with_fill_symbol() -> Vec<u8> {
    let mut bytes = sample_shaped_te_body_with_symbol2_header();
    // ISymbol #2：SimpleFillSymbol 新对象（键 1593，样本键形态）
    bytes.extend(entry(&SIMPLE_FILL_SYMBOL_CLASS_ID, 1593));
    bytes.extend(simple_fill_symbol_body());
    // TextSymbol 字段 15：IFontDisp 空标记；字段 16：版本门尾段（4 = 2+16+2）
    bytes.extend(null_entry());
    bytes.extend(0x8601_u16.to_le_bytes());
    bytes.extend(TEXT_SYMBOL_CLASS_ID);
    bytes.extend(0xf264_u16.to_le_bytes());
    // TextElement 字段 6：IGeometry 空标记；字段 7：版本 >2/>3 的双 int32
    bytes.extend(null_entry());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

/// sample_shaped_te_body 去掉 ISymbol #2 头之后的公共前缀（用于接线形态）。
fn sample_shaped_te_body_with_symbol2_header() -> Vec<u8> {
    let body = sample_shaped_te_body();
    // 末尾 20 字节为 ISymbol #2 对象头（未知类合成形态），截去。
    body[..body.len() - 20].to_vec()
}

#[test]
fn text_element_sample_shaped_body_stops_at_unclosed_symbol2() {
    let body = sample_shaped_te_body();
    let mut reader = Reader::new(&body);
    let parsed = read_element_body_of_class_10_1(
        &TEXT_ELEMENT_CLASS_ID,
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
    )
    .unwrap();
    let mxd2qgz::objects::element_body::ElementBody10_1::TextElement(te) = parsed else {
        panic!("应为 TextElement 体");
    };
    assert_eq!(te.version, 8);
    let props = te.properties.as_ref().unwrap();
    assert_eq!(props.version, 6);
    assert_eq!(props.name, "");
    assert_eq!(props.type_description, "文本");
    assert_eq!(
        props.custom_property,
        CustomProperty10_1::Variant(PropertyValue10_1::Empty)
    );
    assert_eq!(props.extra_short, Some(0xffff));
    assert_eq!(props.reference_scale, Some(5393.0));
    assert_eq!(props.tail_int32, Some(6));
    assert_eq!(te.text, "T35");
    assert_eq!(te.raw_bool_int32, 1);
    let symbol = te.symbol.as_ref().expect("TextSymbol 体应已消费");
    assert_eq!(symbol.version, 4);
    let ColorSlot10_1::Color {
        body: ColorBody10_1::Rgb(color_body),
        ..
    } = &symbol.color1
    else {
        panic!("颜色槽 #1 应为 RgbColor");
    };
    assert_eq!(color_body.key, 1615);
    let ColorSlot10_1::Color {
        body: ColorBody10_1::Rgb(color_body),
        ..
    } = &symbol.color2
    else {
        panic!("颜色槽 #2 应为 RgbColor");
    };
    assert_eq!(color_body.key, 1616);
    assert_eq!(symbol.ints_1, [-1, 3, 1]);
    assert_eq!(symbol.doubles_2, [0.0, 100.0, 100.0]);
    assert_eq!(symbol.double_4, 90.0);
    assert_eq!(symbol.raw_path_clsid, TEXT_SYMBOL_CLASS_ID);
    assert_eq!(symbol.text_path, NestedEntry10_1::Null);
    assert_eq!(symbol.symbol1, NestedEntry10_1::Null);
    assert_eq!(symbol.name_string, "T35");
    assert_eq!(symbol.double_5, 8.0);
    assert_eq!(symbol.int_4, 1);
    assert_eq!(symbol.double_6, 1.5);
    // 停止点 = ISymbol #2 新对象：游标恰在其对象头起点，键未登记。
    let stop = symbol.stop.as_ref().expect("应停止于 ISymbol #2");
    assert!(stop.describe().contains("ISymbol"));
    assert_eq!(symbol.mask_style(), None);
    assert_eq!(symbol.mask_size(), None);
    assert_eq!(symbol.mask_symbol(), None);
    let consumed = body.len() - symbol.remaining_bytes;
    assert_eq!(
        consumed,
        body.len() - 20,
        "停止点恰在末尾 20 字节对象头起点"
    );
    assert_eq!(reader.position(), consumed);
}

#[test]
fn text_element_registry_covers_consumed_keys_only() {
    let body = sample_shaped_te_body();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&body);
    read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table).unwrap();
    // 已消费键：TextSymbol 1614 + RgbColor 1615/1616；停止点键 1593 不登记。
    assert!(table.contains(1614));
    assert!(table.contains(1615));
    assert!(table.contains(1616));
    assert!(!table.contains(1593));
}

#[test]
fn text_element_versions_follow_native_gate() {
    // 版本 0..=9 接受（==9 按 8+尾加块，无独立字段差异直至版本门）；10 与负
    // 值位模式拒绝。
    for version in [0_u16, 1, 7, 8, 9] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(ep_prefix(version.min(6)));
        bytes.extend(encoded("x"));
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(null_entry()); // ITextSymbol 空标记
        bytes.extend(null_entry()); // IGeometry 空标记
        if version > 2 {
            bytes.extend(0_i32.to_le_bytes());
        }
        if version > 3 {
            bytes.extend(0_i32.to_le_bytes());
        }
        if version == 7 {
            // 三槽空标记 + double + int32 + double。
            bytes.extend(null_entry());
            bytes.extend(null_entry());
            bytes.extend(null_entry());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
        }
        if version == 9 {
            bytes.extend(null_entry());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend([0_u8; 32]);
        }
        let mut reader = Reader::new(&bytes);
        let parsed = read_element_body_of_class_10_1(
            &TEXT_ELEMENT_CLASS_ID,
            &mut reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .unwrap_or_else(|error| panic!("版本 {version} 应接受: {error}"));
        let mxd2qgz::objects::element_body::ElementBody10_1::TextElement(te) = parsed else {
            panic!("应为 TextElement");
        };
        assert!(te.stop.is_none(), "版本 {version} 应完整消费");
        assert_eq!(reader.position(), bytes.len());
    }
    for version in [10_u16, 0x8000, 0xffff] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let error = read_element_body_of_class_10_1(
            &TEXT_ELEMENT_CLASS_ID,
            &mut reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn element_properties_prefix_version_gates() {
    // 版本 <2：双 String + 遗留 int32 + LoadObject 槽（空标记）；无 >2/>4/>5
    // 字段。TE 版本恒 8（EP 之前的字段序见 text_element_versions 测试）。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(1_u16.to_le_bytes()); // EP 版本 1
    bytes.extend(encoded("a"));
    bytes.extend(encoded("b"));
    bytes.extend((-7_i32).to_le_bytes());
    bytes.extend(null_entry());
    bytes.extend(encoded("t"));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(null_entry());
    bytes.extend(null_entry());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let body = read_text_element_body_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    let props = body.properties.as_ref().unwrap();
    assert_eq!(props.legacy_int32, Some(-7));
    assert_eq!(
        props.custom_property,
        CustomProperty10_1::ObjectSlot(NestedEntry10_1::Null)
    );
    assert_eq!(props.extra_short, None);
    assert_eq!(body.remaining_bytes, 0);

    // 版本 4：Variant 路径 + int16（无 >4/>5）。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(4_u16.to_le_bytes()); // EP 版本 4
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend(value_bytes(3, &9_i32.to_le_bytes()));
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(encoded("t"));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(null_entry());
    bytes.extend(null_entry());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let body = read_text_element_body_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    let props = body.properties.as_ref().unwrap();
    assert_eq!(
        props.custom_property,
        CustomProperty10_1::Variant(PropertyValue10_1::Raw4(9_i32.to_le_bytes()))
    );
    assert_eq!(props.extra_short, Some(2));
    assert_eq!(props.reference_scale, None);

    // EP 版本 7 拒绝（原生有符号 >6）；负值一并拒绝。
    for version in [7_u16, 0x8000] {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(version.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let error = read_text_element_body_10_1(&mut reader, 1024, &mut RefTable10_1::new());
        assert!(error.is_err(), "version={version}");
    }
}

#[test]
fn element_properties_custom_property_slot_stop() {
    // 版本 3：槽位 CustomProperty 为新对象 → 停止，游标回退到该槽起点，键不
    // 登记；之后字段（int16）结构性未消费。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(3_u16.to_le_bytes()); // EP 版本 3（槽位路径）
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend(entry(&TEXT_SYMBOL_CLASS_ID, 99)); // CustomProperty 新对象
    bytes.extend(0xaaaa_u16.to_le_bytes()); // 未消费
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_text_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(body.properties.is_none());
    let stop = body.stop.as_ref().expect("应停止于 CustomProperty 槽");
    assert!(matches!(
        stop,
        mxd2qgz::objects::element_body::ElementBodyStop10_1::PropertiesCustomProperty {
            key: 99,
            ..
        }
    ));
    // 游标在槽对象头起点（TE 版本 + EP 版本 + 双 String）：2 + 2 + 6 + 6 = 16。
    assert_eq!(reader.position(), 16);
    assert!(!table.contains(99));
}

#[test]
fn text_symbol_color_slot_unknown_class_stops() {
    // 颜色槽 #1 为非 RgbColor 新对象：停止于该槽起点，键不登记。
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    bytes.extend(entry(&[0xa5; 16], 77));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_text_symbol_body_10_1(&mut reader, 1024, &mut table).unwrap();
    let stop = body.stop.as_ref().expect("应停止于颜色槽");
    assert!(stop.describe().contains("IColor"));
    assert_eq!(reader.position(), 2);
    assert!(!table.contains(77));
}

#[test]
fn marker_and_line_element_bodies_consume_fully() {
    // MarkerElement 版本 3：EP + 符号/几何空标记 + int32 + 32 字节。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend(null_entry());
    bytes.extend(null_entry());
    bytes.extend(5_i32.to_le_bytes());
    bytes.extend([0xa5_u8; 32]);
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_body_of_class_10_1(
        &MARKER_ELEMENT_CLASS_ID,
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
    )
    .unwrap();
    let mxd2qgz::objects::element_body::ElementBody10_1::MarkerElement(me) = parsed else {
        panic!("应为 MarkerElement");
    };
    assert!(me.stop.is_none());
    assert_eq!(me.tail_int32, Some(5));
    assert_eq!(me.raw_envelope, Some([0xa5; 32]));
    assert_eq!(reader.position(), bytes.len());

    // LineElement 版本 3：EP + 符号/几何空标记 + int32 + int16。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend(null_entry());
    bytes.extend(null_entry());
    bytes.extend(6_i32.to_le_bytes());
    bytes.extend(7_u16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let parsed = read_element_body_of_class_10_1(
        &LINE_ELEMENT_CLASS_ID,
        &mut reader,
        1024,
        &mut RefTable10_1::new(),
    )
    .unwrap();
    let mxd2qgz::objects::element_body::ElementBody10_1::LineElement(le) = parsed else {
        panic!("应为 LineElement");
    };
    assert!(le.stop.is_none());
    assert_eq!(le.tail_int32, Some(6));
    assert_eq!(le.tail_short, Some(7));
    assert_eq!(reader.position(), bytes.len());

    // 版本 2：无 32 字节/int16 尾段。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend(null_entry());
    bytes.extend(null_entry());
    bytes.extend(5_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let parsed = read_line_element_body_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(parsed.tail_short, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn marker_line_version_gate_rejects() {
    for class in [MARKER_ELEMENT_CLASS_ID, LINE_ELEMENT_CLASS_ID] {
        for version in [4_u16, 0xfffe] {
            let bytes = version.to_le_bytes().to_vec();
            let mut reader = Reader::new(&bytes);
            let error = read_element_body_of_class_10_1(
                &class,
                &mut reader,
                1024,
                &mut RefTable10_1::new(),
            )
            .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData, "version={version:#x}");
            assert_eq!(reader.position(), 0);
        }
    }
}

#[test]
fn dispatcher_rejects_unknown_element_class() {
    let bytes = vec![0xa5_u8; 32];
    let mut reader = Reader::new(&bytes);
    let error =
        read_element_body_of_class_10_1(&[0xaa; 16], &mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert_eq!(reader.remaining(), bytes.len());
}

#[test]
fn truncation_leaves_cursor_and_registry_unchanged() {
    // TextElement 样本形态全前缀截断：失败、游标不推进、重读一致、登记表不
    // 变（含已部分消费路径）。
    let body = sample_shaped_te_body();
    let baseline = {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&body);
        read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table)
            .unwrap();
        table.len()
    };
    assert_eq!(baseline, 3);
    for end in 0..body.len() {
        let mut table = RefTable10_1::new();
        table.register(1);
        let mut reader = Reader::new(&body[..end]);
        let first =
            read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table);
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        let second =
            read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table);
        assert_eq!(
            first.unwrap_err().kind(),
            second.unwrap_err().kind(),
            "end={end}"
        );
        // 失败路径只保留预登记键。
        assert_eq!(table.len(), 1, "end={end}");
        assert!(table.contains(1), "end={end}");
    }
}

#[test]
fn marker_line_truncation_is_transactional() {
    let mut body = 3_u16.to_le_bytes().to_vec();
    body.extend(ep_prefix(6));
    body.extend(null_entry());
    body.extend(null_entry());
    body.extend(5_i32.to_le_bytes());
    body.extend(7_u16.to_le_bytes());
    for end in 0..body.len() {
        let mut reader = Reader::new(&body[..end]);
        let first = read_line_element_body_10_1(&mut reader, 1024, &mut RefTable10_1::new());
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_line_element_body_10_1(&mut reader, 1024, &mut RefTable10_1::new());
        assert_eq!(first.unwrap_err().kind(), second.unwrap_err().kind());
    }
}

#[test]
fn text_symbol_tail_version_fields() {
    // 版本 4：v>1 short + v≥3 GUID + v≥4 short 全读；版本 3：无 v≥4 short；
    // 版本 1：无 short/GUID；版本 0：同 1。
    for (version, tail_len) in [(4_usize, 20), (3, 18), (1, 0), (0, 0)] {
        let mut bytes = (version as u16).to_le_bytes().to_vec();
        bytes.extend(null_entry()); // IColor #1 空标记
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.push(0);
        bytes.push(0);
        for value in [0.0_f64; 3] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(13_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(null_entry()); // IColor #2 空标记
        bytes.extend(0_f64.to_le_bytes());
        bytes.extend(0_f64.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        for value in [0.0_f64; 3] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.push(1);
        bytes.extend(0_f64.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(0_f64.to_le_bytes());
        bytes.push(1);
        bytes.extend([0_u8; 16]); // 原样 GUID（零 = 默认路径）
        bytes.extend(null_entry()); // ITextPath 空标记
        bytes.extend(null_entry()); // ISymbol #1 空标记
        bytes.extend(encoded("x"));
        bytes.extend(0_f64.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(0_f64.to_le_bytes());
        bytes.extend(null_entry()); // ISymbol #2 空标记
        bytes.extend(null_entry()); // IFontDisp 空标记
        bytes.extend(vec![0xa5_u8; tail_len]);
        let mut reader = Reader::new(&bytes);
        let body = read_text_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_or_else(|error| panic!("版本 {version} 应接受: {error}"));
        assert!(body.stop.is_none(), "版本 {version} 应完整消费");
        assert_eq!(reader.position(), bytes.len(), "版本 {version}");
    }
    // 版本 5 拒绝。
    let bytes = 5_u16.to_le_bytes().to_vec();
    let mut reader = Reader::new(&bytes);
    assert!(read_text_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new()).is_err());
}

#[test]
fn back_reference_slots_are_classified_without_bodies() {
    // 回引槽位：20 字节无体，读取器继续消费后续字段。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend(encoded("x"));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(entry(&TEXT_SYMBOL_CLASS_ID, 10)); // 符号槽回引
    bytes.extend(null_entry()); // IGeometry 空标记
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    let mut table = RefTable10_1::new();
    table.register(10);
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table)
            .unwrap();
    let mxd2qgz::objects::element_body::ElementBody10_1::TextElement(te) = parsed else {
        panic!("应为 TextElement");
    };
    assert!(te.symbol.is_none(), "回引槽无 TextSymbol 体");
    assert!(te.stop.is_none());
    assert_eq!(te.geometry, GeometrySlot10_1::Null);
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(table.len(), 1);
}

#[test]
fn text_element_geometry_slot_parses_polyline_body_inline() {
    // 接线轮：IGeometry 槽 Polyline 新对象经几何类分派整体解析（样本 16 实
    // 例原全部停在该槽；证据：docs/arcmap-10.1-geometry.md「接线建议」）。
    // 体取版本 1 最小形态（4 字节类型 0 空几何记录、无空间参考段）。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend(encoded("x"));
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(entry(&TEXT_SYMBOL_CLASS_ID, 10)); // 符号槽回引
    bytes.extend(entry(&POLYLINE_CLASS_ID, 11)); // IGeometry Polyline 新对象
    bytes.extend(1_u16.to_le_bytes()); // Polyline 版本 1
    bytes.extend(4_u32.to_le_bytes()); // 记录声明长度 4
    bytes.extend(0_u32.to_le_bytes()); // 记录：类型 0 空几何
    bytes.extend(0_i32.to_le_bytes()); // 版本 >2 尾 int32
    bytes.extend(0_i32.to_le_bytes()); // 版本 >3 尾 int32
    let mut table = RefTable10_1::new();
    table.register(10);
    let mut reader = Reader::new(&bytes);
    let parsed = read_text_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(reader.position(), bytes.len());
    match parsed.geometry {
        GeometrySlot10_1::Polyline(body) => {
            assert_eq!(body.version, 1);
            assert_eq!(body.record.kind, 0);
            assert!(body.record.points.is_empty());
            assert!(body.spatial_reference.is_none());
        }
        other => panic!("IGeometry 槽应为 Polyline 体：{other:?}"),
    }
    assert_eq!(table.len(), 2, "符号回引 10 + Polyline 新键 11 登记");
}

#[test]
fn text_element_geometry_slot_dispatches_four_geometry_classes() {
    // 整合接线轮：IGeometry 槽自 Polyline 专用扩为四类类型化分派（证据：
    // docs/arcmap-10.1-geometry.md「第二轮落位」接线建议 1）。Polyline 见上
    // 一用例；此处逐类构造独立元素体（TextElement 仅有一个 IGeometry 槽）：
    // Polygon 同 Polyline 最小形态、Point 取类型字 1（XY）、Multipoint 取
    // 低字节 0 空点集。三类样本 0 实例，合成输入固定契约。
    for (class_id, key, body, label) in [
        (
            &POLYGON_CLASS_ID,
            11_u32,
            {
                let mut b = 1_u16.to_le_bytes().to_vec(); // 版本 1
                b.extend(4_u32.to_le_bytes()); // 记录声明长度 4
                b.extend(0_u32.to_le_bytes()); // 类型 0 空几何
                b
            },
            "Polygon",
        ),
        (
            &POINT_CLASS_ID,
            12,
            {
                let mut b = 1_u16.to_le_bytes().to_vec(); // 版本 1
                b.extend(20_u32.to_le_bytes()); // 记录声明长度 20
                b.extend(1_u32.to_le_bytes()); // 类型字 1（XY）
                b.extend(1.5_f64.to_le_bytes()); // X
                b.extend((-2.5_f64).to_le_bytes()); // Y
                b
            },
            "Point",
        ),
        (
            &MULTIPOINT_CLASS_ID,
            13,
            {
                let mut b = 1_u16.to_le_bytes().to_vec(); // 版本 1
                b.extend(4_u32.to_le_bytes()); // 记录声明长度 4
                b.extend(0_u32.to_le_bytes()); // 低字节 0 空点集
                b
            },
            "Multipoint",
        ),
    ] {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(ep_prefix(6));
        bytes.extend(encoded("x"));
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(entry(&TEXT_SYMBOL_CLASS_ID, 10)); // 符号槽回引
        bytes.extend(entry(class_id, key)); // IGeometry 新对象
        bytes.extend(body);
        bytes.extend(0_i32.to_le_bytes()); // 版本 >2 尾 int32
        bytes.extend(0_i32.to_le_bytes()); // 版本 >3 尾 int32
        let mut table = RefTable10_1::new();
        table.register(10);
        let mut reader = Reader::new(&bytes);
        let parsed = read_text_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
        assert!(parsed.stop.is_none(), "{label}");
        assert_eq!(parsed.remaining_bytes, 0, "{label}");
        match (&parsed.geometry, label) {
            (GeometrySlot10_1::Polygon(body), "Polygon") => {
                assert_eq!(body.version, 1);
                assert!(body.record.points.is_empty());
                assert!(body.spatial_reference.is_none());
            }
            (GeometrySlot10_1::Point(body), "Point") => {
                assert_eq!(body.version, 1);
                assert_eq!((body.record.x, body.record.y), (Some(1.5), Some(-2.5)));
            }
            (GeometrySlot10_1::Multipoint(body), "Multipoint") => {
                assert_eq!(body.version, 1);
                assert!(matches!(
                    body.record,
                    mxd2qgz::objects::geometry::MultipointRecord10_1::Empty { kind: 0 }
                ));
            }
            other => panic!("{label} 槽分类不符：{other:?}"),
        }
        assert_eq!(table.len(), 2, "{label}: 回引 10 + 新键登记");
    }
}

#[test]
fn text_element_geometry_slot_unknown_class_still_stops() {
    // 整合接线轮：闭合四类之外的几何类（合成未知标识）维持既有停止语义
    // ——游标回退到槽对象头起点、键不登记、版本门尾段不可达。
    let unknown = [0x44_u8; 16];
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend(encoded("x"));
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(entry(&TEXT_SYMBOL_CLASS_ID, 10)); // 符号槽回引
    let slot_offset = bytes.len();
    bytes.extend(entry(&unknown, 11)); // IGeometry 未知几何类新对象
    bytes.extend([0_u8; 8]); // 未知体占位
    let mut table = RefTable10_1::new();
    table.register(10);
    let mut reader = Reader::new(&bytes);
    let parsed = read_text_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
    match parsed.stop {
        Some(mxd2qgz::objects::element_body::ElementBodyStop10_1::NestedObjectNotClosed {
            class_id,
            key,
            ..
        }) => {
            assert_eq!(class_id, unknown);
            assert_eq!(key, 11);
        }
        other => panic!("应停在未知几何类槽位：{other:?}"),
    }
    assert!(!table.contains(11), "停止回退不保留该键");
    assert_eq!(reader.position(), slot_offset, "游标回退到槽对象头起点");
}

#[test]
fn marker_element_point_slot_parses_point_body_inline() {
    // 整合接线轮：MarkerElement 的 IPoint 槽接 Point 分派（证据：
    // docs/arcmap-10.1-geometry.md「第二轮落位」接线建议 2；样本 0 实例）。
    // Point 取类型字 1（XY），槽后版本门尾段可达 = 槽位整体消费。
    let mut bytes = 2_u16.to_le_bytes().to_vec(); // 版本 2（>1：尾 int32；>2：32 字节块）
    bytes.extend(ep_prefix(6));
    bytes.extend(entry(&MARKER_ELEMENT_CLASS_ID, 20)); // IMarkerSymbol 槽回引
    bytes.extend(entry(&POINT_CLASS_ID, 21)); // IPoint Point 新对象
    bytes.extend(1_u16.to_le_bytes()); // Point 版本 1
    bytes.extend(20_u32.to_le_bytes()); // 记录声明长度 20
    bytes.extend(1_u32.to_le_bytes()); // 类型字 1（XY）
    bytes.extend(3.0_f64.to_le_bytes()); // X
    bytes.extend(4.0_f64.to_le_bytes()); // Y
    bytes.extend(7_i32.to_le_bytes()); // 版本 >1 尾 int32
    let mut table = RefTable10_1::new();
    table.register(20);
    let mut reader = Reader::new(&bytes);
    let parsed = read_marker_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(parsed.remaining_bytes, 0);
    match parsed.geometry {
        PointSlot10_1::Point(body) => {
            assert_eq!(body.version, 1);
            assert_eq!((body.record.x, body.record.y), (Some(3.0), Some(4.0)));
            assert!(body.spatial_reference.is_none());
        }
        other => panic!("IPoint 槽应为 Point 体：{other:?}"),
    }
    assert_eq!(parsed.tail_int32, Some(7), "槽位整体消费后续段可达");
    assert_eq!(table.len(), 2, "符号回引 20 + Point 新键 21");
}

#[test]
fn marker_element_point_slot_unknown_class_stops() {
    // IPoint 槽未知实现类：停止回退到槽起点、键不保留。
    let unknown = [0x55_u8; 16];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    let slot_offset = bytes.len();
    bytes.extend(entry(&unknown, 21)); // IPoint 未知类新对象
    bytes.extend([0_u8; 8]); // 占位
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let parsed = read_marker_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
    match parsed.stop {
        Some(mxd2qgz::objects::element_body::ElementBodyStop10_1::NestedObjectNotClosed {
            class_id,
            key,
            ..
        }) => {
            assert_eq!(class_id, unknown);
            assert_eq!(key, 21);
        }
        other => panic!("应停在未知 IPoint 实现类槽位：{other:?}"),
    }
    assert!(!table.contains(21));
    assert_eq!(reader.position(), slot_offset);
}

#[test]
fn line_element_geometry_slot_parses_polyline_body_inline() {
    // 整合接线轮：LineElement 的 IGeometry 槽与 TextElement 同经四类类型化
    // 分派（样本 0 实例；Polyline 最小形态）。
    let mut bytes = 1_u16.to_le_bytes().to_vec(); // 版本 1（无版本门尾段）
    bytes.extend(ep_prefix(6));
    bytes.extend(entry(&LINE_ELEMENT_CLASS_ID, 30)); // ILineSymbol 槽回引
    bytes.extend(entry(&POLYLINE_CLASS_ID, 31)); // IGeometry Polyline 新对象
    bytes.extend(1_u16.to_le_bytes()); // Polyline 版本 1
    bytes.extend(4_u32.to_le_bytes()); // 记录声明长度 4
    bytes.extend(0_u32.to_le_bytes()); // 类型 0 空几何
    let mut table = RefTable10_1::new();
    table.register(30);
    let mut reader = Reader::new(&bytes);
    let parsed = read_line_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(parsed.remaining_bytes, 0);
    match parsed.geometry {
        GeometrySlot10_1::Polyline(body) => {
            assert_eq!(body.version, 1);
            assert!(body.record.points.is_empty());
        }
        other => panic!("IGeometry 槽应为 Polyline 体：{other:?}"),
    }
    assert_eq!(table.len(), 2, "符号回引 30 + Polyline 新键 31");
}

#[test]
fn string_limit_rejects_without_advancing() {
    // 文本 String 声明超限额：在分配前拒绝。
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(ep_prefix(6));
    bytes.extend((u32::MAX - 1).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error = read_text_element_body_10_1(&mut reader, 16, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn io_errors_propagate_on_empty_input() {
    let mut reader = Reader::new(&[]);
    for class in [
        TEXT_ELEMENT_CLASS_ID,
        MARKER_ELEMENT_CLASS_ID,
        LINE_ELEMENT_CLASS_ID,
    ] {
        let error =
            read_element_body_of_class_10_1(&class, &mut reader, 1024, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    }
}

// ---------------------------------------------------------------------------
// 接线轮：TextSymbol 的 ISymbol #2 槽接入对象层符号槽位分派的增量测试
// （`docs/arcmap-10.1-display-tails.md` 建议接入点；样本真实类 {7914E603} =
// SimpleFillSymbol 体已闭合）。正常路径 + 未知类停止 + 嵌套停止透传。
// ---------------------------------------------------------------------------

#[test]
fn text_element_parses_sample_shaped_symbol2_simple_fill_symbol() {
    // 样本键 1613 形态 + ISymbol #2 = SimpleFillSymbol 新对象（样本真实类
    // {7914E603}）：符号 #2 体整体解析（嵌套 RgbColor 键登记），TextSymbol
    // 推进到 IFontDisp 空标记与版本门尾段，TextElement 推进到几何槽与尾段
    // ——此前「停止于符号 #2 头起点」的分段由本测试的完整消费证明已推进。
    let body = sample_shaped_te_body_with_fill_symbol();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&body);
    let parsed =
        read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table)
            .unwrap();
    let mxd2qgz::objects::element_body::ElementBody10_1::TextElement(te) = parsed else {
        panic!("应为 TextElement 体");
    };
    assert_eq!(te.stop, None, "全部字段应完整消费");
    assert_eq!(reader.position(), body.len());
    let symbol = te.symbol.as_ref().expect("TextSymbol 体应已消费");
    assert!(symbol.stop.is_none(), "TextSymbol 体应完整消费");
    assert_eq!(symbol.mask_style(), Some(1));
    assert_eq!(symbol.mask_size(), Some(1.5));
    assert_eq!(symbol.mask_symbol(), Some(&symbol.symbol2));
    // 符号 #2 为 SimpleFillSymbol 体（轮廓空标记 + 填充色 + 标量段）。
    let SymbolSlotFull10_1::SimpleFillSymbol(fill) = &symbol.symbol2 else {
        panic!("符号 #2 应为 SimpleFillSymbol：{:?}", symbol.symbol2);
    };
    assert_eq!(fill.key, 1593);
    assert!(fill.stop.is_none());
    assert!(
        matches!(
            fill.color,
            Some(ColorSlot10_1::Color {
                body: ColorBody10_1::Rgb(_),
                ..
            })
        ),
        "填充色应为 RgbColor 体"
    );
    // 字段 15/16（接线前未达）：IFontDisp 空标记 + 版本门尾段。
    assert_eq!(symbol.font, FontSlot10_1::Null);
    assert_eq!(symbol.tail_short_1, Some(0x8601));
    assert_eq!(symbol.tail_path_clsid, Some(TEXT_SYMBOL_CLASS_ID));
    assert_eq!(symbol.tail_short_2, Some(0xf264));
    // 已消费键：TextSymbol 1614 + RgbColor ×3（1615/1616/1600）+
    // SimpleFillSymbol 1593。
    for key in [1614_u32, 1615, 1616, 1600, 1593] {
        assert!(table.contains(key), "键 {key} 应登记");
    }
    assert_eq!(table.len(), 5);
}

#[test]
fn text_symbol_symbol2_nested_stop_propagates() {
    // 嵌套停止：符号 #2 为 SimpleFillSymbol 且其轮廓槽为未知类 → 符号体在
    // 内层槽位停止 → TextSymbol 以 SymbolSlotNested 透传，后续字段不可达。
    let unknown = [0x67_u8; 16];
    let mut bytes = sample_shaped_te_body_with_symbol2_header();
    bytes.extend(entry(&SIMPLE_FILL_SYMBOL_CLASS_ID, 1593));
    bytes.extend(1_u16.to_le_bytes()); // SFS 版本
    bytes.extend(entry(&unknown, 1601)); // 轮廓槽：未知类
    bytes.extend(encoded("不可达"));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_element_body_of_class_10_1(&TEXT_ELEMENT_CLASS_ID, &mut reader, 1024, &mut table)
            .unwrap();
    let mxd2qgz::objects::element_body::ElementBody10_1::TextElement(te) = parsed else {
        panic!("应为 TextElement 体");
    };
    let symbol = te.symbol.as_ref().unwrap();
    match &symbol.stop {
        Some(mxd2qgz::objects::element_body::ElementBodyStop10_1::SymbolSlotNested {
            slot,
            reason: _,
        }) => {
            assert_eq!(*slot, "ISymbol 符号槽 #2");
            // 分派把嵌套体内停止归约为 NestedIncomplete（携带 SFS 自身的类
            // 标识与键）；未知轮廓条目的键见 SFS 体自身的 stop 字段。
            let SymbolSlotFull10_1::SimpleFillSymbol(fill) = &symbol.symbol2 else {
                panic!("符号 #2 应为 SimpleFillSymbol");
            };
            assert!(fill.stop.is_some(), "SFS 体应带内部停止");
        }
        other => panic!("应为 SymbolSlotNested 透传，得到 {other:?}"),
    }
    // 游标停在内层轮廓槽起点：SFS 头 20 + 版本 2。
    assert_eq!(
        reader.position(),
        bytes.len() - 20 - encoded("不可达").len()
    );
    assert!(table.contains(1593), "SFS 键已登记（对象头已消费）");
    assert!(!table.contains(1601), "嵌套未知键不保留");
}

#[test]
fn shape_symbols_read_known_types_and_reject_wrong_interface() {
    use mxd2qgz::objects::display_symbol::{
        SIMPLE_LINE_SYMBOL_CLASS_ID, SIMPLE_MARKER_SYMBOL_CLASS_ID,
    };
    for marker in [true, false] {
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend(ep_prefix(6));
        let slot_offset = bytes.len();
        let class = if marker {
            SIMPLE_MARKER_SYMBOL_CLASS_ID
        } else {
            SIMPLE_LINE_SYMBOL_CLASS_ID
        };
        bytes.extend(entry(&class, 71));
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(null_entry());
        if marker {
            bytes.extend([0_u8; 8 + 12 + 24 + 1 + 8]);
            bytes.extend(null_entry());
        } else {
            bytes.extend([0_u8; 8 + 12]);
        }
        bytes.extend(null_entry());
        bytes.extend(0_i32.to_le_bytes());
        if marker {
            bytes.extend([0_u8; 32]);
        } else {
            bytes.extend(0_u16.to_le_bytes());
        }
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        if marker {
            let body = read_marker_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
            assert!(matches!(
                body.symbol,
                SymbolSlotFull10_1::SimpleMarkerSymbol(_)
            ));
            assert!(body.stop.is_none());
        } else {
            let body = read_line_element_body_10_1(&mut reader, 1024, &mut table).unwrap();
            assert!(matches!(
                body.symbol,
                SymbolSlotFull10_1::SimpleLineSymbol(_)
            ));
            assert!(body.stop.is_none());
        }
        assert_eq!(reader.position(), bytes.len());
        assert!(table.contains(71));
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            table.register(4);
            let failed = if marker {
                read_marker_element_body_10_1(&mut reader, 1024, &mut table).is_err()
            } else {
                read_line_element_body_10_1(&mut reader, 1024, &mut table).is_err()
            };
            assert!(failed, "{marker} {end}");
            assert_eq!(reader.position(), 0);
            assert_eq!(table.len(), 1);
            assert!(table.contains(4));
        }
        let mut wrong = bytes[..slot_offset].to_vec();
        wrong.extend(entry(&SIMPLE_FILL_SYMBOL_CLASS_ID, 72));
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&wrong);
        let stop = if marker {
            read_marker_element_body_10_1(&mut reader, 1024, &mut table)
                .unwrap()
                .stop
        } else {
            read_line_element_body_10_1(&mut reader, 1024, &mut table)
                .unwrap()
                .stop
        };
        assert!(matches!(
            stop,
            Some(
                mxd2qgz::objects::element_body::ElementBodyStop10_1::NestedObjectNotClosed {
                    key: 72,
                    ..
                }
            )
        ));
        assert_eq!(reader.position(), slot_offset);
        assert!(!table.contains(72));
    }
}

#[test]
fn shape_symbol_nested_unknown_stops_without_registering_inner_key() {
    use mxd2qgz::objects::display_symbol::{
        SIMPLE_LINE_SYMBOL_CLASS_ID, SIMPLE_MARKER_SYMBOL_CLASS_ID,
    };
    use mxd2qgz::objects::element_body::ElementBodyStop10_1;
    for marker in [true, false] {
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend(ep_prefix(6));
        bytes.extend(entry(
            if marker {
                &SIMPLE_MARKER_SYMBOL_CLASS_ID
            } else {
                &SIMPLE_LINE_SYMBOL_CLASS_ID
            },
            71,
        ));
        bytes.extend(1_u16.to_le_bytes());
        let inner_offset = bytes.len();
        bytes.extend(entry(&[0x55; 16], 72));
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let stop = if marker {
            read_marker_element_body_10_1(&mut reader, 1024, &mut table)
                .unwrap()
                .stop
        } else {
            read_line_element_body_10_1(&mut reader, 1024, &mut table)
                .unwrap()
                .stop
        };
        assert!(matches!(
            stop,
            Some(ElementBodyStop10_1::SymbolSlotNested { .. })
        ));
        assert_eq!(reader.position(), inner_offset);
        assert!(table.contains(71));
        assert!(!table.contains(72));
    }
}

#[test]
#[ignore = "requires extracted ArcMap 10.1 and 10.8 media streams under temp"]
fn media_shape_candidates_reach_geometry_and_tail() {
    use mxd2qgz::objects::display_symbol::SymbolSlotFull10_1;
    let cases = [
        (
            "temp/arcmap101-media-streams/Africa.mxd.Maps",
            0xb53b,
            0xb6c3,
            true,
        ),
        (
            "temp/arcmap101-media-streams/Utilities.mxd.PageLayout",
            0x20cf,
            0x21ec,
            false,
        ),
        (
            "temp/arcmap108-media-streams/asia.mxd.2319C344_B117_4BE0_8B98_93D6E4E94502.Maps",
            0xae25,
            0xafad,
            true,
        ),
        (
            "temp/arcmap108-media-streams/centralusa.mxd.756AEF33_7F48_48CE_804C_228E110C42CC.PageLayout",
            0xa0c,
            0xb29,
            false,
        ),
    ];
    for (path, offset, end, marker) in cases {
        let bytes =
            std::fs::read(path).unwrap_or_else(|error| panic!("无法读取介质流 {path}: {error}"));
        let class = if marker {
            MARKER_ELEMENT_CLASS_ID
        } else {
            LINE_ELEMENT_CLASS_ID
        };
        assert_eq!(&bytes[offset..offset + 16], &class);
        let key = u32::from_le_bytes(bytes[offset + 16..offset + 20].try_into().unwrap());
        let mut table = RefTable10_1::new();
        // 局部探针只是假定此前键已经登记，不代表整流引用关系已经确认。
        for previous in 0..=key {
            table.register(previous);
        }
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(offset + 20).unwrap();
        if marker {
            let body = read_marker_element_body_10_1(&mut reader, 1_000_000, &mut table).unwrap();
            assert!(matches!(
                body.symbol,
                SymbolSlotFull10_1::CharacterMarkerSymbol(_)
            ));
            assert!(matches!(body.geometry, PointSlot10_1::Point(_)));
            assert!(body.stop.is_none());
            assert!(body.raw_envelope.is_some());
        } else {
            let body = read_line_element_body_10_1(&mut reader, 1_000_000, &mut table).unwrap();
            assert!(matches!(
                body.symbol,
                SymbolSlotFull10_1::SimpleLineSymbol(_)
            ));
            assert!(matches!(body.geometry, GeometrySlot10_1::Polyline(_)));
            assert!(body.stop.is_none());
            assert_eq!(body.tail_short, Some(0));
        }
        assert_eq!(reader.position(), end, "{path}");
    }
}
