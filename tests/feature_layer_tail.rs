//! `feature_layer_tail`（FeatureLayer 尾段链第一环）严格读取器的合成输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-featurelayer-tail.md`：五个类（SimpleRenderer
//! /SimpleMarkerSymbol/RgbColor/LegendGroup/LegendClass）的字段序来自 10.1
//! 原生 DLL 反编译，样本 `0x3ed..0x602` 对位为辅助观察。本文件全部使用合成
//! 输入，仅验证读取器自身契约（正常/异常/截断/事务性），不代表 MXD 格式已
//! 验证，未在目标软件中核对。

use mxd2qgz::layers::feature_layer_tail::{
    LEGEND_CLASS_CLASS_ID, LEGEND_GROUP_CLASS_ID, LegendClassEntry10_1, LegendClassFormatSlot10_1,
    LegendClassStop10_1, LegendGroupSlot10_1, LegendGroupStop10_1, RGB_COLOR_CLASS_ID,
    SIMPLE_MARKER_SYMBOL_CLASS_ID, SIMPLE_RENDERER_CLASS_ID, SimpleRendererStop10_1,
    SymbolSlotFull10_1, read_legend_class_body_10_1, read_legend_group_body_10_1,
    read_simple_renderer_body_10_1,
};
use mxd2qgz::layers::legend_format::LEGEND_CLASS_FORMAT_CLASS_ID;
use mxd2qgz::objects::display_color::{ColorBody10_1, ColorSlot10_1, read_rgb_color_body_10_1};
use mxd2qgz::objects::display_symbol::{
    SIMPLE_LINE_SYMBOL_CLASS_ID, SimpleMarkerSymbolStop10_1,
    SymbolStop10_1 as DisplaySymbolStop10_1, read_simple_marker_symbol_body_10_1,
};
use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
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

/// 预登记键的登记表（模拟「外层对象头已登记」的会话状态）。
fn table_with_keys(keys: &[u32]) -> RefTable10_1 {
    let mut table = RefTable10_1::new();
    for key in keys {
        table.register(*key);
    }
    table
}

/// RgbColor 体（版本 1/1）：flag==0 走 3 个 double 分支，flag!=0 走 4 字节
/// CMYK 分支；尾随两字节原样（补码字节用非常量值验证位模式保留）。
fn rgb_color_body(flag: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.push(flag);
    if flag == 0 {
        for value in [1.0_f64, 2.0, 3.0] {
            bytes.extend(value.to_le_bytes());
        }
    } else {
        bytes.extend(0xAABB_CCDD_u32.to_le_bytes());
    }
    bytes.push(0x00);
    bytes.push(0x37);
    bytes
}

/// SimpleMarkerSymbol 体（版本 2）：主色/轮廓色各为一个 RgbColor 新对象
/// （键 37/38，与样本一致），标量段取样本值，尾随 short 0xFFFF。
fn sms_body_v2() -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 37);
    bytes.extend(rgb_color_body(0));
    bytes.extend(6.0_f64.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(13_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    for value in [0.0_f64; 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(1);
    bytes.extend(1.0_f64.to_le_bytes());
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 38);
    bytes.extend(rgb_color_body(0));
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes
}

/// LegendClass 体（版本 2）：符号槽为空标记（或回引键 `symbol_backref`）、
/// 标签/描述为空串、格式槽空标记、int32 -1。
fn legend_class_body_v2(symbol_backref: Option<u32>) -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    match symbol_backref {
        Some(key) => push_entry_header(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, key),
        None => push_null_marker(&mut bytes),
    }
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    push_null_marker(&mut bytes);
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

/// LegendGroup 体（版本 3）：两个 short -1、空标题、计数 1、一个 LegendClass
/// 新对象（键 40，符号回引键 `class_symbol_backref`）、尾 short 0。
fn legend_group_body_v3(class_symbol_backref: Option<u32>) -> Vec<u8> {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(1_i32.to_le_bytes());
    push_entry_header(&mut bytes, &LEGEND_CLASS_CLASS_ID, 40);
    bytes.extend(legend_class_body_v2(class_symbol_backref));
    bytes.extend(0_i16.to_le_bytes());
    bytes
}

/// RotationRenderer::Load 尾段（版本 1）：样本形态 0/0/""/""/""/0×3/360×3。
fn rotation_tail_bytes() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    for _ in 0..3 {
        bytes.extend(string_bytes(""));
    }
    for value in [0.0_f64, 0.0, 0.0, 360.0, 360.0, 360.0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// SizeRenderer::Load 尾段（版本 1）：样本形态 0/""/0.1/100.0。
fn size_tail_bytes() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(0.1_f64.to_le_bytes());
    bytes.extend(100.0_f64.to_le_bytes());
    bytes
}

/// SimpleRenderer 体（版本 3，样本形态）：SimpleMarkerSymbol 新对象 +
/// LegendGroup 新对象（内嵌 LegendClass 符号回引键 36）+ ILegendItem 空标记
/// + 标量段 + 双尾段。
fn simple_renderer_body_v3() -> Vec<u8> {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 36);
    bytes.extend(sms_body_v2());
    push_entry_header(&mut bytes, &LEGEND_GROUP_CLASS_ID, 39);
    bytes.extend(legend_group_body_v3(Some(36)));
    push_null_marker(&mut bytes);
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(rotation_tail_bytes());
    bytes.extend(size_tail_bytes());
    bytes
}

#[test]
fn class_id_constants_match_documented_clsids() {
    // 常量与 esri.clsid 声明/GetClassID 写出的 CLSID 逐字节对照
    // （内存序：Data1-3 小端、Data4 原序）。
    assert_eq!(
        SIMPLE_RENDERER_CLASS_ID,
        [
            0x01, 0x58, 0x43, 0xf3, 0x79, 0x57, 0xd0, 0x11, 0x98, 0xbf, 0x00, 0x80, 0x5f, 0x7c,
            0xed, 0x21
        ]
    );
    assert_eq!(
        SIMPLE_MARKER_SYMBOL_CLASS_ID,
        [
            0xfe, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee,
            0x4e, 0x41
        ]
    );
    assert_eq!(
        RGB_COLOR_CLASS_ID,
        [
            0x96, 0xc4, 0xe9, 0x7e, 0x23, 0xd1, 0xd0, 0x11, 0x83, 0x83, 0x08, 0x00, 0x09, 0xb9,
            0x96, 0xcc
        ]
    );
    assert_eq!(
        LEGEND_GROUP_CLASS_ID,
        [
            0xa2, 0x5e, 0x7c, 0x16, 0x20, 0xaf, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec,
            0x73, 0x2a
        ]
    );
    assert_eq!(
        LEGEND_CLASS_CLASS_ID,
        [
            0xa3, 0x5e, 0x7c, 0x16, 0x20, 0xaf, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec,
            0x73, 0x2a
        ]
    );
}

#[test]
fn rgb_color_reads_rgb_branch_and_preserves_raw_bits() {
    let mut bytes = rgb_color_body(0);
    // 补码字节用 0x37 验证原样保留（不取补）。
    let last = bytes.len() - 1;
    bytes[last] = 0x37;
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_rgb_color_body_10_1(&mut reader, &mut table, 37).unwrap();
    assert_eq!(body.key, 37);
    assert_eq!(body.version_outer, 1);
    assert_eq!(body.version_inner, 1);
    assert_eq!(body.branch_flag, 0);
    assert_eq!(body.doubles, Some([1.0, 2.0, 3.0]));
    assert_eq!(body.cmyk_raw, None);
    assert_eq!(body.trailing_byte, 0x00);
    assert_eq!(body.complement_byte_raw, 0x37, "末字节保留流内位模式");
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn rgb_color_reads_cmyk_branch() {
    let bytes = rgb_color_body(1);
    let mut reader = Reader::new(&bytes);
    let body = read_rgb_color_body_10_1(&mut reader, &mut RefTable10_1::new(), 38).unwrap();
    assert_eq!(body.branch_flag, 1);
    assert_eq!(body.doubles, None);
    assert_eq!(body.cmyk_raw, Some(0xAABB_CCDD));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn rgb_color_rejects_out_of_range_versions() {
    for version in [2_u16, 0x7fff] {
        let mut bytes = rgb_color_body(0);
        bytes[..2].copy_from_slice(&version.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = table_with_keys(&[9]);
        assert!(read_rgb_color_body_10_1(&mut reader, &mut table, 5).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.contains(9));
    }
    // 内层版本超限同样拒绝。
    let mut bytes = Vec::new();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend([0_u8; 28]);
    let mut reader = Reader::new(&bytes);
    assert!(read_rgb_color_body_10_1(&mut reader, &mut RefTable10_1::new(), 5).is_err());
}

#[test]
fn rgb_color_truncation_scan_keeps_cursor() {
    let bytes = rgb_color_body(0);
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut table = RefTable10_1::new();
        assert!(
            read_rgb_color_body_10_1(&mut reader, &mut table, 37).is_err(),
            "截断到 {cut} 字节应失败"
        );
        assert_eq!(reader.position(), 0, "截断失败不得推进游标（cut={cut}）");
        assert!(table.is_empty(), "截断失败不得修改登记表（cut={cut}）");
    }
    // 同输入重读：完整输入成功，结果一致。
    let mut first_reader = Reader::new(&bytes);
    let first = read_rgb_color_body_10_1(&mut first_reader, &mut RefTable10_1::new(), 37).unwrap();
    let mut second_reader = Reader::new(&bytes);
    let second =
        read_rgb_color_body_10_1(&mut second_reader, &mut RefTable10_1::new(), 37).unwrap();
    assert_eq!(first, second);
}

#[test]
fn simple_marker_symbol_reads_full_body_version_two() {
    let bytes = sms_body_v2();
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let sms = read_simple_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 36).unwrap();
    assert_eq!(sms.key, 36);
    assert_eq!(sms.version, 2);
    assert!(
        matches!(
            &sms.color,
            ColorSlot10_1::Color {
                body: ColorBody10_1::Rgb(body),
                ..
            } if body.key == 37 && body.doubles == Some([1.0, 2.0, 3.0])
        ),
        "主色应为 RgbColor 新对象"
    );
    assert_eq!(sms.size, Some(6.0));
    assert_eq!(sms.int32_a, Some(0));
    assert_eq!(sms.int32_b, Some(13), "marker 样式候选值与样本一致");
    assert_eq!(sms.int32_c, Some(0));
    assert_eq!(sms.doubles, Some([0.0; 3]));
    assert_eq!(sms.byte_flag, Some(1));
    assert_eq!(sms.double_ratio, Some(1.0));
    assert!(
        matches!(
            &sms.outline_color,
            Some(ColorSlot10_1::Color {
                body: ColorBody10_1::Rgb(body),
                ..
            }) if body.key == 38
        ),
        "轮廓色应为 RgbColor 新对象"
    );
    // 0xFFFF 位模式（读者按 `u16 as i16` 保留为 -1）。
    assert_eq!(sms.rotate_short, Some(-1_i16));
    assert_eq!(sms.stop, None);
    assert_eq!(reader.remaining(), 0);
    assert!(
        table.contains(37) && table.contains(38),
        "颜色新键随成功提交"
    );
}

#[test]
fn simple_marker_symbol_version_one_omits_rotate_short() {
    let mut bytes = sms_body_v2();
    bytes[0] = 1; // 版本 1：体更短（少尾 short）
    bytes.truncate(bytes.len() - 2);
    let mut reader = Reader::new(&bytes);
    let sms = read_simple_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 36)
        .unwrap();
    assert_eq!(sms.version, 1);
    assert_eq!(sms.rotate_short, None, "版本 ≤1 不读尾 short");
    assert_eq!(sms.stop, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn simple_marker_symbol_stops_at_unknown_main_color() {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    // 主色槽放一个非 RgbColor 的已知类（LegendClass），其余标量段保留在流内。
    push_entry_header(&mut bytes, &LEGEND_CLASS_CLASS_ID, 44);
    bytes.extend([0xEE_u8; 8]);
    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[36]);
    let sms = read_simple_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 36).unwrap();
    assert_eq!(
        sms.stop,
        Some(SimpleMarkerSymbolStop10_1::ColorUnsupported {
            class_id: LEGEND_CLASS_CLASS_ID,
            key: 44,
        })
    );
    assert_eq!(
        sms.key, 36,
        "停止路径结构体 key 保持 SMS 自身键，不被颜色条目键阴影"
    );
    assert!(
        matches!(sms.color, ColorSlot10_1::Null),
        "未知颜色类条目回退，槽位值不保留"
    );
    assert_eq!(sms.size, None, "停止点后的标量不可达");
    assert!(sms.outline_color.is_none());
    assert_eq!(sms.rotate_short, None);
    // 游标回退到槽位起点（版本 short 之后）；该键不登记、已有键不受影响。
    assert_eq!(reader.position(), 2);
    assert!(!table.contains(44), "停止回退不保留该键");
    assert!(table.contains(36), "调用方已登记的外层键不受影响");
}

#[test]
fn simple_marker_symbol_stops_at_unknown_outline_color() {
    let mut bytes = sms_body_v2();
    // 覆盖轮廓色槽的条目头为非 RgbColor 类（保留主色与标量段）。
    let outline_header = 2 + (20 + 31) + 8 + 12 + 24 + 1 + 8;
    let class_start = outline_header;
    bytes[class_start..class_start + 16].copy_from_slice(&LEGEND_CLASS_CLASS_ID);
    bytes[class_start + 16..class_start + 20].copy_from_slice(&44_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let sms = read_simple_marker_symbol_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 36)
        .unwrap();
    assert_eq!(
        sms.stop,
        Some(SimpleMarkerSymbolStop10_1::OutlineColorUnsupported {
            class_id: LEGEND_CLASS_CLASS_ID,
            key: 44,
        })
    );
    assert_eq!(sms.key, 36, "轮廓停止路径结构体 key 保持 SMS 自身键");
    assert_eq!(sms.size, Some(6.0), "首个颜色与标量段已消费并保留");
    assert_eq!(sms.int32_b, Some(13));
    assert_eq!(sms.rotate_short, None, "前置停止不读版本 short 尾段");
    // 游标停回轮廓槽起点。
    assert_eq!(reader.position(), outline_header);
}

#[test]
fn simple_marker_symbol_truncation_scan_keeps_cursor() {
    let bytes = sms_body_v2();
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut table = RefTable10_1::new();
        assert!(
            read_simple_marker_symbol_body_10_1(&mut reader, 1024, &mut table, 36).is_err(),
            "截断到 {cut} 字节应失败"
        );
        assert_eq!(reader.position(), 0, "截断失败不得推进游标（cut={cut}）");
        assert!(table.is_empty(), "截断失败不得修改登记表（cut={cut}）");
    }
}

#[test]
fn legend_class_reads_full_body_and_back_reference_symbol() {
    // 回引形态（样本形态）：符号槽为对已登记键 36 的回引头。
    let bytes = legend_class_body_v2(Some(36));
    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[36]);
    let lc = read_legend_class_body_10_1(&mut reader, usize::MAX, &mut table, 40).unwrap();
    assert_eq!(lc.key, 40);
    assert_eq!(lc.version, 2);
    assert!(matches!(
        &lc.symbol,
        SymbolSlotFull10_1::BackReference { key: 36, .. }
    ));
    assert_eq!(lc.label, "");
    assert_eq!(lc.description, "");
    assert_eq!(lc.format, LegendClassFormatSlot10_1::Null);
    assert_eq!(lc.int32_1c, Some(-1));
    assert_eq!(lc.stop, None);
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(36));
}

#[test]
fn legend_class_version_one_omits_int32() {
    let mut bytes = legend_class_body_v2(None);
    bytes[0] = 1; // 版本 1：体更短（少 int32）
    bytes.truncate(bytes.len() - 4);
    let mut reader = Reader::new(&bytes);
    let lc =
        read_legend_class_body_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new(), 40).unwrap();
    assert_eq!(lc.version, 1);
    assert_eq!(lc.int32_1c, None, "版本 ≤1 不读 int32");
    assert_eq!(lc.stop, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn legend_class_stops_at_format_new_object() {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    // ILegendClassFormat 槽放一个新对象（coclass 体未闭合）。
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 45);
    bytes.extend((-1_i32).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let lc = read_legend_class_body_10_1(&mut reader, usize::MAX, &mut table, 40).unwrap();
    assert_eq!(
        lc.stop,
        Some(LegendClassStop10_1::FormatNewObject {
            class_id: RGB_COLOR_CLASS_ID,
            key: 45,
        })
    );
    assert_eq!(
        lc.format,
        LegendClassFormatSlot10_1::Unsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 45,
        },
        "未知格式类以 Unsupported 分类保留，游标与登记表回退"
    );
    assert_eq!(lc.int32_1c, None, "停止点后的 int32 不可达");
    assert_eq!(reader.position(), 2 + 16 + 6 + 6, "游标回退到格式槽起点");
    assert!(!table.contains(45), "停止回退不保留该键");
}

#[test]
fn legend_group_reads_full_body_with_nested_legend_class() {
    let bytes = legend_group_body_v3(Some(36));
    let mut reader = Reader::new(&bytes);
    // 外层键 39 由调用方（对象头读取）登记；回引键 36 预登记。
    let mut table = table_with_keys(&[36, 39]);
    let lg = read_legend_group_body_10_1(&mut reader, usize::MAX, &mut table, 39).unwrap();
    assert_eq!(lg.key, 39);
    assert_eq!(lg.version, 3);
    assert_eq!(lg.heading_short, -1);
    assert_eq!(lg.editable_short, Some(-1));
    assert_eq!(lg.heading, "");
    assert_eq!(lg.class_count, 1);
    assert_eq!(lg.classes.len(), 1);
    match &lg.classes[0] {
        LegendClassEntry10_1::NewObject(lc) => {
            assert_eq!(lc.key, 40);
            assert!(matches!(
                &lc.symbol,
                SymbolSlotFull10_1::BackReference { key: 36, .. }
            ));
        }
        other => panic!("应为嵌套 LegendClass，得到 {other:?}"),
    }
    assert_eq!(lg.trailing_short, Some(0));
    assert_eq!(lg.stop, None);
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(39), "调用方登记的外层键不受影响");
    assert!(table.contains(40), "嵌套 LegendClass 新键随成功提交");
}

#[test]
fn legend_group_version_one_layout_and_negative_count() {
    // 版本 1 + 负计数：无 editable_short、无尾 short（版本门）、负计数不进循环。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(5_i16.to_le_bytes());
    bytes.extend(string_bytes("标题"));
    bytes.extend((-1_i32).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let lg =
        read_legend_group_body_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new(), 39).unwrap();
    assert_eq!(lg.version, 1);
    assert_eq!(lg.editable_short, None);
    assert_eq!(lg.heading, "标题");
    assert_eq!(lg.class_count, -1);
    assert!(lg.classes.is_empty(), "负计数原生不进循环");
    assert_eq!(lg.trailing_short, None, "版本 ≤2 无尾 short");
    assert_eq!(lg.stop, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn legend_group_rejects_out_of_range_versions() {
    for version in [4_u16, 0x8000] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let position = reader.position();
        let mut table = table_with_keys(&[39]);
        assert!(
            read_legend_group_body_10_1(&mut reader, usize::MAX, &mut table, 39).is_err(),
            "版本 {version:#x} 应拒绝"
        );
        assert_eq!(reader.position(), position, "失败不得推进游标");
        assert!(table.contains(39), "失败不得修改登记表");
    }
}

#[test]
fn legend_group_stops_at_unsupported_class_entry() {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(1_i32.to_le_bytes());
    // 循环条目放非 LegendClass 的已知类（RgbColor 新对象）。
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 46);
    bytes.extend(0_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let lg = read_legend_group_body_10_1(&mut reader, usize::MAX, &mut table, 39).unwrap();
    assert_eq!(
        lg.stop,
        Some(LegendGroupStop10_1::ClassUnsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 46,
            index: 0,
        })
    );
    assert!(lg.classes.is_empty(), "不支持的条目不进列表");
    assert_eq!(lg.trailing_short, None, "前置停止不读尾 short");
    // 游标回退到该条目对象头起点（5 字段之后）。
    let item_offset = 2 + 2 + 2 + 6 + 4;
    assert_eq!(reader.position(), item_offset);
    assert!(!table.contains(46), "停止回退不保留该键");
}

#[test]
fn legend_group_propagates_nested_class_stop() {
    // 嵌套 LegendClass 的格式槽放新对象：停止传播为 ClassBody，游标停在内层
    // 停止点（格式槽起点）。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(1_i32.to_le_bytes());
    push_entry_header(&mut bytes, &LEGEND_CLASS_CLASS_ID, 40);
    bytes.extend(2_u16.to_le_bytes());
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 47);
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend(0_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let lg = read_legend_group_body_10_1(&mut reader, usize::MAX, &mut table, 39).unwrap();
    match &lg.stop {
        Some(LegendGroupStop10_1::ClassBody(LegendClassStop10_1::FormatNewObject {
            key, ..
        })) => {
            assert_eq!(*key, 47);
        }
        other => panic!("应为嵌套 LegendClass 停止传播，得到 {other:?}"),
    }
    // 游标停在嵌套 LegendClass 的格式槽起点（版本+符号+两串之后）。
    let format_offset = 2 + 2 + 2 + 6 + 4 + 20 + 2 + 16 + 6 + 6;
    assert_eq!(reader.position(), format_offset);
    assert!(!table.contains(47), "内层停止回退不保留该键");
    assert!(table.contains(40), "已提交的嵌套对象键保留");
}

fn simple_renderer_reads_full_body_sample_shape_setup() -> Vec<u8> {
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &SIMPLE_RENDERER_CLASS_ID, 35);
    bytes.extend(simple_renderer_body_v3());
    bytes
}

#[test]
fn simple_renderer_reads_full_body_sample_shape() {
    let bytes = simple_renderer_reads_full_body_sample_shape_setup();
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    // 模拟调用者：对象头读取 + 类比对 + 键登记（原生 ObjectStream 语义）。
    let entry = read_object_entry_10_1(&mut reader, &mut table).unwrap();
    match &entry {
        ObjectEntry10_1::NewObject { class_id, key: 35 } => {
            assert_eq!(*class_id, SIMPLE_RENDERER_CLASS_ID)
        }
        other => panic!("应为 SimpleRenderer 新对象，得到 {other:?}"),
    }
    let body = read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).unwrap();
    assert_eq!(body.key, 35);
    assert_eq!(body.version, 3);
    assert!(
        matches!(&body.symbol, SymbolSlotFull10_1::SimpleMarkerSymbol(sms) if sms.key == 36),
        "符号槽应为 SimpleMarkerSymbol 新对象"
    );
    match &body.legend_group {
        Some(LegendGroupSlot10_1::LegendGroup(lg)) => {
            assert_eq!(lg.key, 39);
            assert_eq!(lg.classes.len(), 1);
        }
        other => panic!("应为 LegendGroup 新对象，得到 {other:?}"),
    }
    assert_eq!(
        body.legend_item,
        Some(ObjectEntry10_1::Null),
        "样本为空标记"
    );
    assert_eq!(body.short_128, Some(0));
    assert_eq!(body.rotation_field, Some(String::new()));
    assert_eq!(body.rotation_type, Some(0));
    assert_eq!(body.label, Some(String::new()));
    let rotation = body
        .rotation_tail
        .expect("版本 3 应有 RotationRenderer 尾段");
    assert_eq!(rotation.version, 1);
    assert_eq!(rotation.flags, [0, 0]);
    assert_eq!(
        rotation.expressions,
        [String::new(), String::new(), String::new()]
    );
    assert_eq!(rotation.doubles, [0.0, 0.0, 0.0, 360.0, 360.0, 360.0]);
    let size = body.size_tail.expect("版本 3 应有 SizeRenderer 尾段");
    assert_eq!(size.version, 1);
    assert_eq!(size.flags, 0);
    assert_eq!(size.doubles, [0.1, 100.0]);
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 0);
    assert_eq!(reader.remaining(), 0, "体完整消费");
    // 全部新对象键（35 渲染器、36 符号、37/38 颜色、39 组、40 类）已登记。
    for key in [35, 36, 37, 38, 39, 40] {
        assert!(table.contains(key), "键 {key} 应随成功读取登记");
    }
}

#[test]
fn simple_renderer_rejects_out_of_range_versions() {
    for version in [4_u16, 0x8000] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let position = reader.position();
        let mut table = table_with_keys(&[35]);
        assert!(
            read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).is_err(),
            "版本 {version:#x} 应拒绝"
        );
        assert_eq!(reader.position(), position, "失败不得推进游标");
        assert!(table.contains(35), "失败不得修改登记表");
    }
}

#[test]
fn simple_renderer_stops_at_unknown_symbol() {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    // 符号槽放非 SimpleMarkerSymbol 的已知类（RgbColor 新对象）。
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 48);
    bytes.extend([0xEE_u8; 8]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).unwrap();
    assert_eq!(
        body.stop,
        Some(SimpleRendererStop10_1::SymbolUnsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 48,
        })
    );
    assert!(
        matches!(body.symbol, SymbolSlotFull10_1::Null),
        "未知符号回退"
    );
    assert!(body.legend_group.is_none(), "停止点后的槽位不可达");
    assert_eq!(body.rotation_tail, None);
    assert_eq!(reader.position(), 2, "游标回退到符号槽起点");
    assert!(!table.contains(48), "停止回退不保留该键");
}

#[test]
fn simple_renderer_stops_at_legend_item_new_object() {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes); // 符号槽空标记
    push_null_marker(&mut bytes); // 组槽空标记
    // ILegendItem 槽放一个新对象（coclass 体未闭合）。
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 49);
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).unwrap();
    assert_eq!(
        body.stop,
        Some(SimpleRendererStop10_1::LegendItemNewObject {
            class_id: RGB_COLOR_CLASS_ID,
            key: 49,
        })
    );
    assert_eq!(
        body.legend_item,
        Some(ObjectEntry10_1::NewObject {
            class_id: RGB_COLOR_CLASS_ID,
            key: 49,
        }),
        "条目分类保留，游标与登记表回退"
    );
    assert_eq!(body.short_128, None, "停止点后的标量不可达");
    let item_offset = 2 + 16 + 16;
    assert_eq!(
        reader.position(),
        item_offset,
        "游标回退到 ILegendItem 槽起点"
    );
    assert!(!table.contains(49), "停止回退不保留该键");
}

#[test]
fn simple_renderer_truncation_scan_keeps_cursor() {
    // 版本 1 的小体：无双尾段，便于全前缀逐字节扫描。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes);
    push_null_marker(&mut bytes);
    push_null_marker(&mut bytes);
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(string_bytes(""));
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut table = RefTable10_1::new();
        assert!(
            read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).is_err(),
            "截断到 {cut} 字节应失败"
        );
        assert_eq!(reader.position(), 0, "截断失败不得推进游标（cut={cut}）");
        assert!(table.is_empty(), "截断失败不得修改登记表（cut={cut}）");
    }
    // 完整输入成功且体完整消费。
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).unwrap();
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 0);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn simple_renderer_string_limit_is_enforced() {
    // 版本 1 小体 + max_bytes 限额：限额 0 让旋转字段字符串读取失败且游标
    // 与登记表不动。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes);
    push_null_marker(&mut bytes);
    push_null_marker(&mut bytes);
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(string_bytes("标签字段"));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(string_bytes(""));
    let mut reader = Reader::new(&bytes);
    let position = reader.position();
    let mut table = RefTable10_1::new();
    assert!(
        read_simple_renderer_body_10_1(&mut reader, 0, &mut table, 35).is_err(),
        "限额 0 应让字符串读取失败"
    );
    assert_eq!(reader.position(), position, "失败不得推进游标");
    assert!(table.is_empty(), "失败不得修改登记表");
}

#[test]
fn renderer_error_positions_are_body_relative() {
    // 错误位置相对于传入 Reader（无对象头偏移）：从体起点读版本 99 时
    // 错误消息里的偏移应为体起点 0。
    let bytes = [99_u8, 0];
    let mut reader = Reader::new(&bytes);
    let error =
        read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new(), 35)
            .unwrap_err();
    assert!(
        error.to_string().contains("0x0"),
        "错误位置应为体起点：{error}"
    );
    assert_eq!(error.kind(), ErrorKind::InvalidData);
}

// ---------------------------------------------------------------------------
// 第八轮：FeatureLayer 后继字段段（FieldInfo/NumericFormat/后继段读取器）。
// 字段序证据见 docs/arcmap-10.1-featurelayer-tail.md 第八轮（Layer.dll
// FieldInfo::Load 0x7865b520、SystemCore.dll NumericFormat::Load 0x7ffce4b0、
// FeatureLayer.dll FL::Load 0x6f6e9940 后继段）；样本 0x5b0..0xafe 逐字节
// 吻合为辅助观察。全部为合成输入契约测试。
// ---------------------------------------------------------------------------

use mxd2qgz::layers::feature_layer_tail::{
    FIELD_INFO_CLASS_ID, FieldInfoEntry10_1, FieldInfoStop10_1, HyperlinkSlot10_1,
    NUMERIC_FORMAT_CLASS_ID, NumberFormatSlot10_1, SuccessorStop10_1,
    read_feature_layer_successor_10_1, read_field_info_body_10_1, read_numeric_format_body_10_1,
};

/// NumericFormat 体（版本 1）：int32 ×4 + short ×3，无嵌套对象。
fn numeric_format_body(int32s: [i32; 4]) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    for value in int32s {
        bytes.extend(value.to_le_bytes());
    }
    for value in [0_i16; 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// FieldInfo 体（版本 4）：short ×2（-0x16/-0x14）+ 别名串 + INumberFormat
/// 槽（`number_format` 字节：空标记 / 回引头 / NumericFormat 新对象头+体）+
/// 尾段 short/int32/short/short。
fn field_info_body_v4(alias: &str, number_format: &[u8]) -> Vec<u8> {
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    bytes.extend(0xFFFF_u16.to_le_bytes()); // -0x16（可见性候选）
    bytes.extend(0_u16.to_le_bytes()); // -0x14（版本 >1）
    bytes.extend(string_bytes(alias));
    bytes.extend_from_slice(number_format);
    bytes.extend((-1_i16).to_le_bytes()); // -0x18（版本 >2）
    bytes.extend((-1_i32).to_le_bytes()); // -0x1c
    bytes.extend(0_i16.to_le_bytes()); // -0x8（版本 >3）
    bytes.extend(0_i16.to_le_bytes()); // -0x6
    bytes
}

#[test]
fn successor_class_id_constants_match_documented_clsids() {
    // FieldInfo {A2BAAE2D-969B-11D2-AE77-080009EC732A}（esri.clsid 声明 +
    // Layer.dll GetClassID 0x7865b4e0 立即数写出 + 样本 14 个对象头）与
    // NumericFormat {7E4F4719-8E54-11D2-AAD8-000000000000}（esri.clsid 声明
    // + SystemCore.dll GetClassID 0x7ffce470 立即数写出 + 样本嵌套对象头）。
    assert_eq!(
        FIELD_INFO_CLASS_ID,
        [
            0x2d, 0xae, 0xba, 0xa2, 0x9b, 0x96, 0xd2, 0x11, 0xae, 0x77, 0x08, 0x00, 0x09, 0xec,
            0x73, 0x2a
        ]
    );
    assert_eq!(
        NUMERIC_FORMAT_CLASS_ID,
        [
            0x19, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00
        ]
    );
}

#[test]
fn numeric_format_reads_body_and_preserves_values() {
    // 样本形态：OBJECTID 条目 int32 0/0/0/12，XZBZ 条目 0/6/0/12，short 全 0。
    for int32s in [[0, 0, 0, 12], [0, 6, 0, 12]] {
        let mut bytes = numeric_format_body(int32s);
        bytes[0] = 1; // 版本位（numeric_format_body 已写 1，此处保持）
        let mut reader = Reader::new(&bytes);
        let body =
            read_numeric_format_body_10_1(&mut reader, &mut RefTable10_1::new(), 42).unwrap();
        assert_eq!(body.key, 42);
        assert_eq!(body.version, 1);
        assert_eq!(body.int32s, int32s);
        assert_eq!(body.shorts, [0, 0, 0]);
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn numeric_format_rejects_out_of_range_versions_without_advancing() {
    for version in [2_u16, 0x8000, 0xFFFF] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0_u8; 22]);
        let mut reader = Reader::new(&bytes);
        let position = reader.position();
        let mut table = table_with_keys(&[9]);
        assert!(
            read_numeric_format_body_10_1(&mut reader, &mut table, 5).is_err(),
            "版本 {version:#x} 应拒绝"
        );
        assert_eq!(reader.position(), position, "失败不得推进游标");
        assert!(table.contains(9), "失败不得修改登记表");
    }
}

#[test]
fn field_info_reads_version_4_body_with_numeric_format_nested() {
    // 样本 OBJECTID 条目形态：别名与条目名同文、NumericFormat 新对象（键 42）
    // 嵌套、尾段 -1/-1/0/0；FieldInfo 与 NumericFormat 两键登记。
    let mut nested = NUMERIC_FORMAT_CLASS_ID.to_vec();
    nested.extend(42_u32.to_le_bytes());
    nested.extend(numeric_format_body([0, 0, 0, 12]));
    let bytes = field_info_body_v4("OBJECTID", &nested);
    let mut reader = Reader::new(&bytes);
    // 体读取器的键登记发生在调用者的条目分类（read_object_entry_10_1）：
    // 预先登记键 41 模拟该会话状态。
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.key, 41);
    assert_eq!(body.version, 4);
    assert_eq!(body.short_16, -1);
    assert_eq!(body.short_14, Some(0));
    assert_eq!(body.alias, "OBJECTID");
    let NumberFormatSlot10_1::NumericFormat(nf) = &body.number_format else {
        panic!("嵌套应为 NumericFormat：{:?}", body.number_format);
    };
    assert_eq!(nf.key, 42);
    assert_eq!(nf.int32s, [0, 0, 0, 12]);
    assert_eq!(body.short_18, Some(-1));
    assert_eq!(body.int32_1c, Some(-1));
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(body.stop, None);
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(41));
    assert!(table.contains(42));
    assert_eq!(table.len(), 2);
}

#[test]
fn field_info_number_format_slot_dispatches_four_variants() {
    // 接线轮：INumberFormat 槽四类分派（NumericFormat 既有 +
    // Percentage/Fraction/Rate 三变体；证据与迁移建议见
    // docs/arcmap-10.1-number-format.md「实现」节）。各变体取满形态，
    // FieldInfo 尾段必须可达（槽位整体消费）。
    use mxd2qgz::layers::feature_layer_tail::{
        FRACTION_FORMAT_CLASS_ID, PERCENTAGE_FORMAT_CLASS_ID, RATE_FORMAT_CLASS_ID,
    };

    // Percentage 版本 2 满形态：short ② + 内嵌 NumericFormat 核心。
    let mut percentage = PERCENTAGE_FORMAT_CLASS_ID.to_vec();
    percentage.extend(80_u32.to_le_bytes());
    percentage.extend(2_u16.to_le_bytes());
    percentage.extend(3_i16.to_le_bytes());
    percentage.extend(numeric_format_body([0, 0, 0, 12]));
    let body_bytes = field_info_body_v4("P", &percentage);
    let mut reader = Reader::new(&body_bytes);
    // 键 41 由调用者的条目分类预先登记（与既有 NumericFormat 用例同约定）。
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::PercentageFormat(p) = &body.number_format else {
        panic!("Percentage 分类不符：{:?}", body.number_format);
    };
    assert_eq!(p.key, 80);
    assert_eq!(p.version, 2);
    assert_eq!(p.numeric.as_ref().unwrap().int32s, [0, 0, 0, 12]);
    assert_eq!(body.tail_shorts, Some([0, 0]), "槽位整体消费后续段可达");
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(41) && table.contains(80));

    // Fraction 版本 1 满形态：int32 ×2。
    let mut fraction = FRACTION_FORMAT_CLASS_ID.to_vec();
    fraction.extend(81_u32.to_le_bytes());
    fraction.extend(1_u16.to_le_bytes());
    fraction.extend(7_i32.to_le_bytes());
    fraction.extend((-2_i32).to_le_bytes());
    let body_bytes = field_info_body_v4("F", &fraction);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::FractionFormat(f) = &body.number_format else {
        panic!("Fraction 分类不符：{:?}", body.number_format);
    };
    assert_eq!(f.key, 81);
    assert_eq!((f.int32_10, f.int32_0c), (7, -2));
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(reader.remaining(), 0);

    // Rate 版本 2 满形态：double + String + 内嵌 NumericFormat 核心。
    let mut rate = RATE_FORMAT_CLASS_ID.to_vec();
    rate.extend(82_u32.to_le_bytes());
    rate.extend(2_u16.to_le_bytes());
    rate.extend(1.5_f64.to_le_bytes());
    rate.extend(string_bytes("米每秒"));
    rate.extend(numeric_format_body([1, 2, 3, 4]));
    let body_bytes = field_info_body_v4("R", &rate);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::RateFormat(r) = &body.number_format else {
        panic!("Rate 分类不符：{:?}", body.number_format);
    };
    assert_eq!(r.key, 82);
    assert_eq!(r.double_14, 1.5);
    assert_eq!(r.string_0c, "米每秒");
    assert_eq!(r.numeric.as_ref().unwrap().int32s, [1, 2, 3, 4]);
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn field_info_number_format_slot_dispatches_five_more_variants() {
    // 接线轮：INumberFormat 槽扩为 esriSystem 同族九类全分派——Currency/
    // LatLon/Angle/Scientific/CustomNumber 五类新臂（证据与落位步骤见
    // docs/arcmap-10.1-number-format.md「九类分派接入建议」；五类样本
    // 0 实例，合成输入）。各变体取合法形态，FieldInfo 尾段必须可达。
    use mxd2qgz::layers::feature_layer_tail::{
        ANGLE_FORMAT_CLASS_ID, CURRENCY_FORMAT_CLASS_ID, CUSTOM_NUMBER_FORMAT_CLASS_ID,
        LATLON_FORMAT_CLASS_ID, SCIENTIFIC_FORMAT_CLASS_ID,
    };

    // Currency：恒 2 字节体（short 版本）。
    let mut currency = CURRENCY_FORMAT_CLASS_ID.to_vec();
    currency.extend(83_u32.to_le_bytes());
    currency.extend(1_u16.to_le_bytes());
    let body_bytes = field_info_body_v4("C", &currency);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::CurrencyFormat(c) = &body.number_format else {
        panic!("Currency 分类不符：{:?}", body.number_format);
    };
    assert_eq!(c.key, 83);
    assert_eq!(body.tail_shorts, Some([0, 0]), "槽位整体消费后续段可达");
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(41) && table.contains(83));

    // LatLon 版本 1 短体：short ×4（版本 ≤1 无 ④⑤⑥ 段）。
    let mut latlon = LATLON_FORMAT_CLASS_ID.to_vec();
    latlon.extend(84_u32.to_le_bytes());
    latlon.extend(1_u16.to_le_bytes());
    latlon.extend(0_i16.to_le_bytes());
    latlon.extend(0_i16.to_le_bytes());
    let body_bytes = field_info_body_v4("L", &latlon);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::LatLonFormat(l) = &body.number_format else {
        panic!("LatLon 分类不符：{:?}", body.number_format);
    };
    assert_eq!(l.key, 84);
    assert_eq!(l.numeric, None);
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(reader.remaining(), 0);

    // Angle 版本 1 短体：short ×2（版本 ≤1 无核心与 ④）。
    let mut angle = ANGLE_FORMAT_CLASS_ID.to_vec();
    angle.extend(85_u32.to_le_bytes());
    angle.extend(1_u16.to_le_bytes());
    angle.extend(0_i16.to_le_bytes());
    let body_bytes = field_info_body_v4("A", &angle);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::AngleFormat(a) = &body.number_format else {
        panic!("Angle 分类不符：{:?}", body.number_format);
    };
    assert_eq!(a.key, 85);
    assert_eq!(a.numeric, None);
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(reader.remaining(), 0);

    // Scientific 版本 2 满形态：int32。
    let mut scientific = SCIENTIFIC_FORMAT_CLASS_ID.to_vec();
    scientific.extend(86_u32.to_le_bytes());
    scientific.extend(2_u16.to_le_bytes());
    scientific.extend(6_i32.to_le_bytes());
    let body_bytes = field_info_body_v4("S", &scientific);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::ScientificFormat(s) = &body.number_format else {
        panic!("Scientific 分类不符：{:?}", body.number_format);
    };
    assert_eq!(s.key, 86);
    assert_eq!(s.int32_08, Some(6));
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(reader.remaining(), 0);

    // CustomNumber 版本 1：String 格式串（max_bytes 限额沿用 Rate 臂传参）。
    let mut custom = CUSTOM_NUMBER_FORMAT_CLASS_ID.to_vec();
    custom.extend(87_u32.to_le_bytes());
    custom.extend(1_u16.to_le_bytes());
    custom.extend(string_bytes("0.000"));
    let body_bytes = field_info_body_v4("U", &custom);
    let mut reader = Reader::new(&body_bytes);
    let mut table = table_with_keys(&[41]);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(body.stop, None);
    let NumberFormatSlot10_1::CustomNumberFormat(u) = &body.number_format else {
        panic!("CustomNumber 分类不符：{:?}", body.number_format);
    };
    assert_eq!(u.key, 87);
    assert_eq!(body.tail_shorts, Some([0, 0]));
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(41) && table.contains(87));
}

#[test]
fn field_info_reads_null_number_format_and_version_branches() {
    // 版本 3：-0x14 读取、别名、空标记槽、-0x18/-0x1c 读取、-0x8/-0x6 缺席
    //（`3 < ver` 为假）；版本 1：-0x14 缺席、尾段全部缺席。
    let mut v3 = 3_u16.to_le_bytes().to_vec();
    v3.extend(0_u16.to_le_bytes()); // -0x16
    v3.extend(7_u16.to_le_bytes()); // -0x14（版本 >1）
    v3.extend(string_bytes("别名"));
    push_null_marker(&mut v3); // INumberFormat：空标记
    v3.extend(2_i16.to_le_bytes()); // -0x18
    v3.extend(3_i32.to_le_bytes()); // -0x1c
    let mut reader = Reader::new(&v3);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 9).unwrap();
    assert_eq!(body.version, 3);
    assert_eq!(body.short_14, Some(7));
    assert_eq!(body.number_format, NumberFormatSlot10_1::Null);
    assert_eq!((body.short_18, body.int32_1c), (Some(2), Some(3)));
    assert_eq!(body.tail_shorts, None, "版本 3 无 -0x8/-0x6 段");
    assert_eq!(body.stop, None);
    assert_eq!(reader.remaining(), 0);

    let mut v1 = 1_u16.to_le_bytes().to_vec();
    v1.extend(0xFFFF_u16.to_le_bytes()); // -0x16
    v1.extend(string_bytes(""));
    push_null_marker(&mut v1);
    let mut reader = Reader::new(&v1);
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 10).unwrap();
    assert_eq!(body.short_14, None, "版本 1 无 -0x14 段");
    assert_eq!(body.tail_shorts, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn field_info_rejects_out_of_range_versions_without_advancing() {
    // 原生 `4 < (short)ver` 拒绝；负值位模式一并拒绝（较原生更严）。
    for version in [5_u16, 0x8000, 0xFFFF] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0_u8; 8]);
        let mut reader = Reader::new(&bytes);
        let position = reader.position();
        let mut table = table_with_keys(&[9]);
        assert!(
            read_field_info_body_10_1(&mut reader, 1024, &mut table, 5).is_err(),
            "版本 {version:#x} 应拒绝"
        );
        assert_eq!(reader.position(), position, "失败不得推进游标");
        assert!(table.contains(9), "失败不得修改登记表");
    }
}

#[test]
fn field_info_truncation_scan_keeps_cursor_and_table() {
    let mut nested = NUMERIC_FORMAT_CLASS_ID.to_vec();
    nested.extend(42_u32.to_le_bytes());
    nested.extend(numeric_format_body([0, 0, 0, 12]));
    let bytes = field_info_body_v4("BSM", &nested);
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let position = reader.position();
        let mut table = table_with_keys(&[7]);
        assert!(
            read_field_info_body_10_1(&mut reader, 1024, &mut table, 8).is_err(),
            "截断 {cut} 应失败"
        );
        assert_eq!(reader.position(), position, "截断 {cut} 不得推进游标");
        assert!(
            table.contains(7) && table.len() == 1,
            "截断 {cut} 不得修改登记表"
        );
    }
    // 完整输入成功且完整消费。
    let mut reader = Reader::new(&bytes);
    assert!(read_field_info_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 8).is_ok());
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn field_info_stops_at_unknown_number_format_class() {
    // 嵌套槽新对象类标识非 NumericFormat（PercentageFormat 等同族未闭合）：
    // 游标回退到槽位起点，键不保留，版本尾段不可达（槽位占位字节随剩余
    // 保留——停止点之后无真实尾段，占位即未知对象体开头）。
    let mut slot = RGB_COLOR_CLASS_ID.to_vec();
    slot.extend(55_u32.to_le_bytes());
    slot.extend([0_u8; 8]); // 未知体的占位
    let bytes = field_info_body_v4("X", &slot);
    // 槽位起点 = 版本 2 + -0x16 2 + -0x14 2 + 别名字符串。
    let slot_offset = 2 + 2 + 2 + string_bytes("X").len();
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_field_info_body_10_1(&mut reader, 1024, &mut table, 41).unwrap();
    assert_eq!(
        body.stop,
        Some(FieldInfoStop10_1::NumberFormatUnsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 55,
        })
    );
    assert_eq!(body.number_format, NumberFormatSlot10_1::Null);
    assert_eq!(body.short_18, None, "停止点后的尾段不可达");
    assert_eq!(reader.position(), slot_offset, "游标回退到槽位起点");
    assert!(!table.contains(55), "停止回退不保留该键");
}

/// 后继段 +0x154..+0x188 前缀：标量段 + IHyperlink 计数 `hyperlink_count`
/// 及其条目字节 `hyperlink_entries` + IColor 槽字节 `color_slot` + 字符串段。
fn successor_prefix(hyperlink_count: i32, hyperlink_entries: &[u8], color_slot: &[u8]) -> Vec<u8> {
    let mut bytes = string_bytes("BSM"); // +0x154
    bytes.extend(0_f64.to_le_bytes()); // +0x158
    bytes.extend(0_f64.to_le_bytes()); // +0x160
    bytes.extend(0_i16.to_le_bytes()); // +0x168
    bytes.extend(hyperlink_count.to_le_bytes());
    bytes.extend_from_slice(hyperlink_entries);
    bytes.extend_from_slice(color_slot);
    bytes.extend(string_bytes("")); // +0x1a8
    bytes.extend(string_bytes("")); // +0x184
    bytes.extend(0_i32.to_le_bytes()); // +0x188
    bytes
}

/// 最小完整后继段（两计数 0 形态），循环完整消费即停止。
fn successor_minimal() -> Vec<u8> {
    let mut bytes = successor_prefix(0, b"", &[0_u8; 16]);
    bytes.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
    bytes
}

#[test]
fn successor_loads_explicit_aoi_and_preserves_following_range_ids() {
    let mut bytes = successor_minimal();
    push_null_marker(&mut bytes); // 标注集合。
    bytes.extend(0_i16.to_le_bytes());
    push_null_marker(&mut bytes); // FeatureIDSet。
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i16.to_le_bytes());
    push_null_marker(&mut bytes); // IDisplayFilter。
    bytes.extend([0; 6]); // 三个 short。
    push_null_marker(&mut bytes); // IName。
    bytes.extend(0_i32.to_le_bytes()); // 关系类条数。
    bytes.extend(string_bytes(""));
    let aoi_at = bytes.len();
    push_entry_header(&mut bytes, &ENVELOPE_CLASS_ID, 72);
    bytes.extend(1_u16.to_le_bytes());
    let coords = [1.0_f64, 2.0, 3.0, 4.0];
    for coord in coords {
        bytes.extend(coord.to_le_bytes());
    }
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(42_i32.to_le_bytes());
    let end = bytes.len();
    bytes.push(0xa5);

    let mut table = RefTable10_1::new();
    for cut in aoi_at..end {
        let mut truncated = Reader::new(&bytes[..cut]);
        assert!(read_feature_layer_successor_10_1(&mut truncated, 10, 1024, &mut table).is_err());
        assert_eq!(truncated.position(), 0);
        assert!(table.is_empty());
    }
    let mut reader = Reader::new(&bytes);
    let tail = read_feature_layer_successor_10_1(&mut reader, 10, 1024, &mut table).unwrap();
    assert_eq!(tail.stop, SuccessorStop10_1::TailCompleted);
    assert_eq!(tail.explicit_area_of_interest.unwrap().coords, coords);
    assert_eq!(tail.range_ids, [42]);
    assert!(table.contains(72));
    assert_eq!(reader.position(), end);
}

#[test]
fn successor_reads_minimal_segment_and_completes_field_info_loop() {
    let bytes = successor_minimal();
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(result.string_154, "BSM");
    assert_eq!((result.double_158, result.double_160), (0.0, 0.0));
    assert_eq!(result.short_168, 0);
    assert_eq!(result.hyperlink_count, 0);
    assert!(result.hyperlinks.is_empty());
    assert_eq!(result.color, ColorSlot10_1::Null);
    assert_eq!(result.string_1a8, "");
    assert_eq!(result.string_184, "");
    assert_eq!(result.int32_188, 0);
    assert_eq!(result.field_info_count, 0);
    assert!(result.field_infos.is_empty());
    assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
    assert_eq!(result.remaining_bytes, 0);
    assert_eq!(reader.remaining(), 0);
    assert!(table.is_empty(), "计数 0 无登记");
}

#[test]
fn successor_reads_sample_shaped_field_info_loop() {
    // 样本形态两条目：条目名 + FieldInfo 新对象（一条嵌 NumericFormat 新对
    // 象，一条空标记）；名字与 FieldInfo/NumericFormat 键登记。
    let mut nested = NUMERIC_FORMAT_CLASS_ID.to_vec();
    nested.extend(42_u32.to_le_bytes());
    nested.extend(numeric_format_body([0, 0, 0, 12]));
    let mut bytes = successor_prefix(0, b"", &[0_u8; 16]);
    bytes.extend(2_i32.to_le_bytes()); // FieldInfo 计数 2
    bytes.extend(string_bytes("OBJECTID"));
    push_entry_header(&mut bytes, &FIELD_INFO_CLASS_ID, 41);
    bytes.extend(field_info_body_v4("OBJECTID", &nested));
    bytes.extend(string_bytes("BSM"));
    push_entry_header(&mut bytes, &FIELD_INFO_CLASS_ID, 43);
    bytes.extend(field_info_body_v4("BSM", &[0_u8; 16])); // 空标记嵌套
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(result.field_info_count, 2);
    assert_eq!(result.field_infos.len(), 2);
    assert_eq!(result.field_infos[0].name.as_deref(), Some("OBJECTID"));
    let FieldInfoEntry10_1::FieldInfo(first) = &result.field_infos[0].slot else {
        panic!("条目 0 应为 FieldInfo：{:?}", result.field_infos[0].slot);
    };
    assert_eq!(first.key, 41);
    assert!(matches!(
        first.number_format,
        NumberFormatSlot10_1::NumericFormat(_)
    ));
    assert_eq!(result.field_infos[1].name.as_deref(), Some("BSM"));
    let FieldInfoEntry10_1::FieldInfo(second) = &result.field_infos[1].slot else {
        panic!("条目 1 应为 FieldInfo：{:?}", result.field_infos[1].slot);
    };
    assert_eq!(second.number_format, NumberFormatSlot10_1::Null);
    assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
    assert_eq!(result.remaining_bytes, 0);
    assert_eq!(table.len(), 3, "FieldInfo ×2 + NumericFormat ×1 登记");
    assert!(table.contains(41) && table.contains(42) && table.contains(43));
}

#[test]
fn successor_skips_entry_names_at_architecture_version_five_and_below() {
    // 架构版本 ≤5：条目名由原生从要素类模式接口推导，流内无名字段；读取器
    // 跳过名字直接读条目头（汇编门 0x6f6e9c91）。
    let mut bytes = successor_prefix(0, b"", &[0_u8; 16]);
    bytes.extend(1_i32.to_le_bytes()); // FieldInfo 计数 1
    push_entry_header(&mut bytes, &FIELD_INFO_CLASS_ID, 41);
    bytes.extend(field_info_body_v4("别名", &[0_u8; 16]));
    for version in [0_u16, 5] {
        let mut probe = Reader::new(&bytes);
        // 每次迭代用独立登记表（键登记会让第二次读分类为回引、跳过体）。
        let mut table = RefTable10_1::new();
        let result =
            read_feature_layer_successor_10_1(&mut probe, version, 1024, &mut table).unwrap();
        assert_eq!(result.field_infos.len(), 1, "version={version}");
        assert_eq!(result.field_infos[0].name, None, "version={version}");
        assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
        assert_eq!(probe.remaining(), 0, "version={version}");
    }
    // 版本 6 起名字在流内：同一字节流会把「别名」串误读为名字并把对象头错
    // 位——用带名字的完整形态验证版本 6 读取名字。
    let mut with_name = successor_prefix(0, b"", &[0_u8; 16]);
    with_name.extend(1_i32.to_le_bytes()); // FieldInfo 计数 1
    with_name.extend(string_bytes("名字"));
    push_entry_header(&mut with_name, &FIELD_INFO_CLASS_ID, 41);
    with_name.extend(field_info_body_v4("别名", &[0_u8; 16]));
    let mut probe = Reader::new(&with_name);
    let mut table6 = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut probe, 6, 1024, &mut table6).unwrap();
    assert_eq!(result.field_infos[0].name.as_deref(), Some("名字"));
    assert_eq!(probe.remaining(), 0);
}

#[test]
fn successor_stops_at_hyperlink_new_object() {
    // IHyperlink 循环新对象（Hyperlink 族体未闭合）：游标回退到条目头起点、
    // 键不保留，其后 IColor 槽与字符串段不可达。
    let mut entries = Vec::new();
    push_entry_header(&mut entries, &RGB_COLOR_CLASS_ID, 61);
    let mut bytes = successor_prefix(1, &entries, &[0_u8; 16]);
    // 条目头起点 = 标量段（+0x154 串 + double ×2 + short）+ 计数 4。
    let header_offset = string_bytes("BSM").len() + 8 + 8 + 2 + 4;
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(
        result.stop,
        SuccessorStop10_1::HyperlinkNewObject {
            class_id: RGB_COLOR_CLASS_ID,
            key: 61,
            index: 0,
        }
    );
    assert_eq!(result.hyperlinks, Vec::<HyperlinkSlot10_1>::new());
    assert_eq!(result.field_infos, Vec::new(), "停止点后循环不可达");
    assert!(!table.contains(61), "停止回退不保留该键");
    // 游标停在条目对象头起点（剩余 = 头 20 + 不可达尾部）。
    assert_eq!(
        result.remaining_bytes,
        bytes.len() - header_offset,
        "游标回退到条目头起点"
    );
}

#[test]
fn successor_hyperlink_loop_parses_new_null_and_repeated_key_in_order() {
    // 构造新对象、空标记和同一键回引，验证循环之后仍能读取 IColor 与 FieldInfo。
    use mxd2qgz::objects::hyperlink::HYPERLINK_CLASS_ID;

    let mut entries = Vec::new();
    push_entry_header(&mut entries, &HYPERLINK_CLASS_ID, 80);
    entries.extend(1_u16.to_le_bytes()); // Hyperlink 版本 1
    entries.extend(17_i32.to_le_bytes()); // FeatureId
    entries.extend(string_bytes("https://example.invalid/a"));
    entries.extend(0_i32.to_le_bytes()); // LinkType
    push_null_marker(&mut entries);
    push_entry_header(&mut entries, &HYPERLINK_CLASS_ID, 80);

    let mut bytes = successor_prefix(3, &entries, &[0_u8; 16]);
    // 之后只有 FieldInfo 计数 0、无延续段字节：循环完成、ALPC 槽头不可读。
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
    assert_eq!(result.hyperlink_count, 3);
    assert_eq!(result.hyperlinks.len(), 3);
    match &result.hyperlinks[0] {
        HyperlinkSlot10_1::Hyperlink(body) => {
            assert_eq!(body.key, 80);
            assert_eq!(body.version, 1);
            assert_eq!(body.type_04, 17);
            assert_eq!(body.link, "https://example.invalid/a");
        }
        other => panic!("条目 0 应为 Hyperlink 体：{other:?}"),
    }
    assert!(matches!(result.hyperlinks[1], HyperlinkSlot10_1::Null));
    assert_eq!(
        result.hyperlinks[2],
        HyperlinkSlot10_1::BackReference {
            class_id: HYPERLINK_CLASS_ID,
            key: 80,
        }
    );
    assert!(table.contains(80));
    assert_eq!(table.len(), 1);
    assert!(matches!(result.color, ColorSlot10_1::Null));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn successor_stops_at_unsupported_color() {
    // IColor 槽新对象非 RgbColor：游标回退到槽位起点，字符串段不可达。
    let mut slot = Vec::new();
    push_entry_header(&mut slot, &LEGEND_GROUP_CLASS_ID, 62);
    let mut bytes = successor_prefix(0, b"", &slot);
    // 槽位起点 = 标量段（+0x154 串 + double ×2 + short）+ IHyperlink 计数 4。
    let slot_offset = string_bytes("BSM").len() + 8 + 8 + 2 + 4;
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(
        result.stop,
        SuccessorStop10_1::ColorUnsupported {
            class_id: LEGEND_GROUP_CLASS_ID,
            key: 62,
        }
    );
    assert!(matches!(result.color, ColorSlot10_1::Null), "未知颜色回退");
    assert_eq!(result.field_infos, Vec::new(), "停止点后字符串段不可达");
    assert_eq!(result.remaining_bytes, bytes.len() - slot_offset);
    assert!(!table.contains(62));
}

#[test]
fn successor_stops_at_unsupported_field_info_entry() {
    // FieldInfo 循环条目新对象类标识非 FieldInfo：游标回退到条目头起点、
    // 键不保留；已读条目名保留在该条目记录中。
    let mut bytes = successor_prefix(0, b"", &[0_u8; 16]);
    bytes.extend(1_i32.to_le_bytes()); // FieldInfo 计数 1
    bytes.extend(string_bytes("名字"));
    let header_offset = bytes.len();
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 63);
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(
        result.stop,
        SuccessorStop10_1::FieldInfoEntryUnsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 63,
            index: 0,
        }
    );
    assert_eq!(result.field_infos.len(), 1);
    assert_eq!(result.field_infos[0].name.as_deref(), Some("名字"));
    assert!(matches!(
        result.field_infos[0].slot,
        FieldInfoEntry10_1::Null
    ));
    assert_eq!(result.remaining_bytes, bytes.len() - header_offset);
    assert!(!table.contains(63));
}

#[test]
fn successor_stops_inside_field_info_body_at_unknown_number_format() {
    // FieldInfo 体内嵌套停止传播为 Successor(FieldInfoBody)，游标停在内层
    // 槽位起点；FieldInfo 本体键保留（对象头已消费）。
    let mut bytes = successor_prefix(0, b"", &[0_u8; 16]);
    bytes.extend(1_i32.to_le_bytes()); // FieldInfo 计数 1
    bytes.extend(string_bytes("OBJECTID"));
    push_entry_header(&mut bytes, &FIELD_INFO_CLASS_ID, 41);
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(string_bytes("OBJECTID"));
    let slot_offset = bytes.len();
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 64);
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(
        result.stop,
        SuccessorStop10_1::FieldInfoBody(FieldInfoStop10_1::NumberFormatUnsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 64,
        })
    );
    let FieldInfoEntry10_1::FieldInfo(body) = &result.field_infos[0].slot else {
        panic!("条目应为 FieldInfo");
    };
    assert_eq!(
        body.stop,
        Some(FieldInfoStop10_1::NumberFormatUnsupported {
            class_id: RGB_COLOR_CLASS_ID,
            key: 64,
        })
    );
    assert_eq!(result.remaining_bytes, bytes.len() - slot_offset);
    assert!(table.contains(41), "FieldInfo 键保留（对象头已消费）");
    assert!(!table.contains(64), "嵌套未知键不保留");
}

#[test]
fn successor_truncation_scan_keeps_cursor_and_table() {
    // 样本形态整链逐字节截断：全部失败、不 panic、游标不动、登记表不变。
    let mut nested = NUMERIC_FORMAT_CLASS_ID.to_vec();
    nested.extend(42_u32.to_le_bytes());
    nested.extend(numeric_format_body([0, 0, 0, 12]));
    let mut bytes = successor_prefix(0, b"", &[0_u8; 16]);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(string_bytes("OBJECTID"));
    push_entry_header(&mut bytes, &FIELD_INFO_CLASS_ID, 41);
    bytes.extend(field_info_body_v4("OBJECTID", &nested));
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let position = reader.position();
        let mut table = table_with_keys(&[7]);
        assert!(
            read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).is_err(),
            "截断 {cut} 应失败"
        );
        assert_eq!(reader.position(), position, "截断 {cut} 不得推进游标");
        assert!(
            table.contains(7) && table.len() == 1,
            "截断 {cut} 不得修改登记表"
        );
    }
}

#[test]
fn successor_negative_counts_skip_loops_and_are_preserved() {
    // 负数计数原生不进循环（有符号比较），原样保留；随后字段照常读取。
    let mut bytes = string_bytes("");
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(0_f64.to_le_bytes());
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend((-3_i32).to_le_bytes()); // IHyperlink 计数 -3
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    bytes.extend((-7_i32).to_le_bytes()); // +0x188
    bytes.extend((-2_i32).to_le_bytes()); // FieldInfo 计数 -2
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(result.hyperlink_count, -3);
    assert_eq!(result.int32_188, -7);
    assert_eq!(result.field_info_count, -2);
    assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn successor_back_references_pass_through_loops() {
    // 预登记键后，IHyperlink 循环与 FieldInfo 循环的条目分类为回引（无体），
    // 不新增登记。
    let mut entries = Vec::new();
    push_entry_header(&mut entries, &FIELD_INFO_CLASS_ID, 80); // 回引
    let mut bytes = successor_prefix(1, &entries, &[0_u8; 16]);
    bytes.extend(1_i32.to_le_bytes()); // FieldInfo 计数 1
    bytes.extend(string_bytes("字段"));
    push_entry_header(&mut bytes, &FIELD_INFO_CLASS_ID, 80); // 回引
    let mut reader = Reader::new(&bytes);
    let mut table = table_with_keys(&[80]);
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert_eq!(
        result.hyperlinks,
        vec![HyperlinkSlot10_1::BackReference {
            class_id: FIELD_INFO_CLASS_ID,
            key: 80,
        }]
    );
    assert_eq!(result.field_infos.len(), 1);
    assert_eq!(
        result.field_infos[0].slot,
        FieldInfoEntry10_1::BackReference {
            class_id: FIELD_INFO_CLASS_ID,
            key: 80,
        }
    );
    assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
    assert_eq!(table.len(), 1, "回引不新增登记");
}

// ---------------------------------------------------------------------------
// 接线轮：渲染器/图例类/后继段接入对象层颜色与符号槽位分派的增量测试。
// ISymbol 槽与 IColor 槽按 `docs/arcmap-10.1-display-tails.md` 的建议接入点
// 改调 `objects::display_symbol::read_display_symbol_slot_10_1` 与
// `objects::display_color::read_color_slot_10_1`；ILegendClassFormat 槽按
// `docs/arcmap-10.1-legend-tails.md` 接 `legend_format` 体读取器。全部为合
// 成输入契约测试（正常/未知类停止/嵌套停止透传/截断）。
// ---------------------------------------------------------------------------

/// SimpleLineSymbol 体（版本 1）：颜色槽为 RgbColor 新对象（键 61）+ 标量段。
fn simple_line_symbol_body_with_rgb() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &RGB_COLOR_CLASS_ID, 61);
    bytes.extend(rgb_color_body(0));
    bytes.extend(2.5_f64.to_le_bytes()); // 宽度候选
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

#[test]
fn simple_renderer_parses_closed_line_symbol_slot() {
    // 正常路径：ISymbol 槽为本轮闭合的 SimpleLineSymbol 新对象 → 整体解析
    // （嵌套 RgbColor 键登记），渲染器继续消费 LegendGroup 槽与标量段。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &SIMPLE_LINE_SYMBOL_CLASS_ID, 60);
    bytes.extend(simple_line_symbol_body_with_rgb());
    push_null_marker(&mut bytes); // ILegendGroup 槽
    push_null_marker(&mut bytes); // ILegendItem 槽
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(string_bytes(""));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(string_bytes(""));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).unwrap();
    assert!(matches!(
        &body.symbol,
        SymbolSlotFull10_1::SimpleLineSymbol(line) if line.key == 60 && line.width == Some(2.5)
    ));
    assert!(body.legend_group.is_some(), "符号体完整后组槽可达");
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 0);
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(60) && table.contains(61), "符号与颜色键登记");
}

#[test]
fn simple_renderer_nested_symbol_stop_propagates() {
    // 嵌套停止：SimpleLineSymbol 体内颜色槽为未知类 → 符号体在其槽位停止 →
    // 渲染器以 SymbolNested 透传，游标停在内层槽位起点，组槽不可达。
    let unknown = [0x3d_u8; 16];
    let mut line = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut line, &unknown, 70);
    line.extend(2.5_f64.to_le_bytes()); // 停止后不可达
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    push_entry_header(&mut bytes, &SIMPLE_LINE_SYMBOL_CLASS_ID, 60);
    bytes.extend(&line);
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let body = read_simple_renderer_body_10_1(&mut reader, usize::MAX, &mut table, 35).unwrap();
    match body.stop {
        Some(SimpleRendererStop10_1::SymbolNested(DisplaySymbolStop10_1::NestedIncomplete {
            class_id,
            key,
        })) => {
            // 分派把嵌套体内停止归约为 NestedIncomplete（携带闭合符号自身的
            // 类标识与键）；未知颜色条目键 70 见嵌套体自身的 stop 字段。
            assert_eq!(class_id, SIMPLE_LINE_SYMBOL_CLASS_ID);
            assert_eq!(key, 60);
        }
        other => panic!("应为 SymbolNested 透传，得到 {other:?}"),
    }
    assert!(matches!(
        &body.symbol,
        SymbolSlotFull10_1::SimpleLineSymbol(line) if line.stop.is_some()
    ));
    assert!(body.legend_group.is_none(), "停止点后的槽位不可达");
    // 游标停在内层颜色槽起点：渲染器版本 2 + 符号头 20 + 线符号版本 2。
    assert_eq!(reader.position(), 2 + 20 + 2);
    assert!(table.contains(60), "符号键已登记（对象头已消费）");
    assert!(!table.contains(70), "未知颜色键不保留");
}

#[test]
fn legend_class_parses_legend_class_format_body() {
    // 正常路径：ILegendClassFormat 槽为 LegendClassFormat 新对象（键 50，版
    // 本 2 + 四空槽 + double ×2）→ 整体解析，版本 >1 的 int32 可达。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes); // ISymbol 槽
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    push_entry_header(&mut bytes, &LEGEND_CLASS_FORMAT_CLASS_ID, 50);
    bytes.extend(2_u16.to_le_bytes()); // LCF 体版本 2
    for _ in 0..4 {
        push_null_marker(&mut bytes);
    }
    bytes.extend(1.5_f64.to_le_bytes());
    bytes.extend((-2.5_f64).to_le_bytes());
    bytes.extend((-7_i32).to_le_bytes()); // 版本 >1 的 int32
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let lc = read_legend_class_body_10_1(&mut reader, usize::MAX, &mut table, 40).unwrap();
    let LegendClassFormatSlot10_1::Format(format) = &lc.format else {
        panic!("格式槽应为 LegendClassFormat 体：{:?}", lc.format);
    };
    assert_eq!(format.key, 50);
    assert_eq!(format.version, 2);
    assert_eq!(format.patch_doubles, Some([1.5, -2.5]));
    assert_eq!(lc.int32_1c, Some(-7), "格式体完整消费后 int32 可达");
    assert_eq!(lc.stop, None);
    assert_eq!(reader.remaining(), 0);
    assert!(table.contains(50), "格式对象键登记");
}

#[test]
fn legend_class_format_body_stop_propagates() {
    // 嵌套停止：LegendClassFormat 体首个对象槽为未知类新对象 → 格式体在其
    // 槽位停止 → LegendClass 以 FormatBody 透传，游标停在内层槽位起点。
    let unknown = [0x41_u8; 16];
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    push_null_marker(&mut bytes);
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    let format_offset = bytes.len();
    push_entry_header(&mut bytes, &LEGEND_CLASS_FORMAT_CLASS_ID, 50);
    bytes.extend(2_u16.to_le_bytes()); // LCF 体版本 2
    push_entry_header(&mut bytes, &unknown, 71); // 格式体槽 0：未知类
    bytes.extend(string_bytes("不可达"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let lc = read_legend_class_body_10_1(&mut reader, usize::MAX, &mut table, 40).unwrap();
    match lc.stop {
        Some(LegendClassStop10_1::FormatBody(
            mxd2qgz::layers::legend_format::LegendClassFormatStop10_1::SlotUnsupported {
                index,
                class_id,
                key,
            },
        )) => {
            assert_eq!(index, 0);
            assert_eq!(class_id, unknown);
            assert_eq!(key, 71);
        }
        other => panic!("应为 FormatBody 透传，得到 {other:?}"),
    }
    assert!(matches!(
        &lc.format,
        LegendClassFormatSlot10_1::Format(body) if body.stop.is_some()
    ));
    assert_eq!(lc.int32_1c, None, "嵌套停止后 int32 不可达");
    // 游标停在内层槽位起点：LCF 头 20 + LCF 版本 2 之后。
    assert_eq!(reader.position(), format_offset + 20 + 2);
    assert!(table.contains(50), "格式对象键已登记");
    assert!(!table.contains(71), "嵌套未知键不保留");
}

#[test]
fn successor_parses_cmyk_color_slot() {
    // 正常路径：后继段 IColor 槽为 CmykColor 新对象（键 65，版本 4 无串分
    // 支）→ 整体解析，字符串段可达。
    let mut bytes = string_bytes("BSM"); // +0x154
    bytes.extend(0_f64.to_le_bytes()); // +0x158
    bytes.extend(0_f64.to_le_bytes()); // +0x160
    bytes.extend(0_i16.to_le_bytes()); // +0x168
    bytes.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
    // IColor 槽：CmykColor 新对象（键 65，版本 4 无串分支）。
    bytes.extend(mxd2qgz::objects::display_color::CMYK_COLOR_CLASS_ID);
    bytes.extend(65_u32.to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.push(0);
    bytes.push(0);
    bytes.extend(0x0302_0100_u32.to_le_bytes());
    bytes.push(0x21);
    bytes.push(0x43);
    bytes.extend(string_bytes("")); // +0x1a8
    bytes.extend(string_bytes("")); // +0x184
    bytes.extend(0_i32.to_le_bytes()); // +0x188
    bytes.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
    assert!(matches!(
        &result.color,
        ColorSlot10_1::Color {
            class_id,
            body: ColorBody10_1::Cmyk(_)
        } if *class_id == mxd2qgz::objects::display_color::CMYK_COLOR_CLASS_ID
    ));
    assert_eq!(result.stop, SuccessorStop10_1::FieldInfoLoopCompleted);
    assert!(table.contains(65));
    assert_eq!(reader.remaining(), 0);
}

// ===== 第九轮：FieldInfo 循环之后的延续段（ALPC/LELP/FeatureIDSet/扩展段
// （EditTemplate 宿主）/TimeData 族）。原生证据见
// docs/arcmap-10.1-featurelayer-tail.md「第九轮」。 =====

mod tail_continuation {
    use mxd2qgz::layers::annotate_props::EnvelopeSlot10_1;
    use mxd2qgz::layers::feature_layer_tail::UID_CLASS_ID;

    use super::*;
    use mxd2qgz::layers::annotate_props::{
        ANNOTATE_PROPS_COLLECTION_CLASS_ID, AlpcEntrySlot10_1, AnnotatePropsStop10_1,
        EDIT_TEMPLATE_MANAGER_CLASS_ID, EditTemplateManagerStop10_1, LABEL_ENGINE_PROPS_CLASS_ID,
        LelpStop10_1, LelpTextSymbolSlot10_1, read_annotate_props_collection_body_10_1,
        read_edit_template_manager_body_10_1, read_label_engine_props_body_10_1,
    };
    use mxd2qgz::layers::edit_template::EDIT_TEMPLATE_CLASS_ID;
    use mxd2qgz::layers::feature_id_set::{FEATURE_ID_SET_CLASS_ID, read_feature_id_set_body_10_1};
    use mxd2qgz::layers::feature_layer_tail::{
        AnnotatePropsSlot10_1, FeatureIdSetSlot10_1, FeatureLayerSuccessor10_1, LayerExtension10_1,
        RefSlot10_1,
    };
    use mxd2qgz::objects::element_body::ElementBodyStop10_1;

    /// TextSymbol 类标识（CLSID）字节（样本键 61 对象头，仅用于合成测试）。
    const TEXT_SYMBOL_CLASS_ID: [u8; 16] = [
        0x74, 0x3e, 0x5a, 0xb6, 0x93, 0x29, 0xd1, 0x11, 0x9a, 0x43, 0x00, 0x80, 0xc7, 0xec, 0x5c,
        0x96,
    ];

    /// FeatureLayer 后继段到 FieldInfo 循环为止的前缀（样本形态）。
    fn successor_prefix() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(string_bytes("BSM"));
        b.extend(0_f64.to_le_bytes()); // +0x158
        b.extend(0_f64.to_le_bytes()); // +0x160
        b.extend(0_i16.to_le_bytes()); // +0x168
        b.extend(0_i32.to_le_bytes()); // IHyperlink 计数
        push_null_marker(&mut b); // IColor 槽
        b.extend(string_bytes("")); // +0x1a8
        b.extend(string_bytes("")); // +0x184
        b.extend(0_i32.to_le_bytes()); // +0x188
        b.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
        b
    }

    /// LabelEngineLayerProperties 体（版本 7）⑯ 槽之前的标量段前缀。
    fn lelp_body_v7_prefix() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(7_i16.to_le_bytes()); // 版本 7
        b.extend(string_bytes("[JZDH]")); // ② 串 1（表达式候选）
        b.extend((-1_i16).to_le_bytes()); // ③ short -0x74
        // ④ 版本 ≤1 的 short：版本 7 不写
        b.extend(0_f64.to_le_bytes()); // ⑤ double -0x54
        b.extend(0_i32.to_le_bytes()); // ⑥ int32 -0x60
        b.extend((-1_i16).to_le_bytes()); // ⑦ short -0x5c
        b.extend(0_i16.to_le_bytes()); // ⑧ short -0x4c
        b.extend((-1_i16).to_le_bytes()); // ⑨ short -0x4a
        b.extend(0_i32.to_le_bytes()); // ⑩ int32 -0x80
        b.extend(0_f64.to_le_bytes()); // ⑪ double -0x34
        b.extend(0_f64.to_le_bytes()); // ⑫ double -0x3c
        b.extend((-1_i16).to_le_bytes()); // ⑬ short -0x48
        b.extend(string_bytes("默认")); // ⑭ 串 2（标注类名候选）
        b.extend(string_bytes("")); // ⑮ 串 3
        b
    }

    /// LabelEngineLayerProperties 体（版本 7）：标量段取样本形态；⑯ 槽可注
    /// 入新对象头（停止点形态），⑰ 起默认空标记。
    fn lelp_body_v7(symbol_slot: Option<(&[u8; 16], u32)>) -> Vec<u8> {
        let mut b = lelp_body_v7_prefix();
        match symbol_slot {
            Some((class_id, key)) => push_entry_header(&mut b, class_id, key),
            None => push_null_marker(&mut b),
        }
        push_null_marker(&mut b); // ⑰ 过摄属性槽
        push_null_marker(&mut b); // ⑱ 表达式引擎槽
        push_null_marker(&mut b); // ⑲ Envelope 槽
        b.extend(0_f64.to_le_bytes()); // ⑳ double -0x24
        b.extend(0_f64.to_le_bytes()); // ㉑ double -0x1c
        b.extend(0_i32.to_le_bytes()); // ㉒ int32 -0x14
        b.extend(0_i32.to_le_bytes()); // ㉓ int32 -0x2c
        push_null_marker(&mut b); // ㉔ 容器槽（版本 >2）
        b.extend(0_i16.to_le_bytes()); // ㉕ short -0x46（版本 ≥4）
        b.extend(0_i32.to_le_bytes()); // ㉖ int32 -0x7c（版本 ≥5）
        b
    }

    /// TextSymbol 体（版本 4）：槽位默认空标记 + 标量段样本形态；IFontDisp
    /// 槽可注入新对象头（接线轮停止点形态）。返回（字节，IFontDisp 槽对
    /// 象头在返回字节内的偏移）。
    fn text_symbol_body_v4(font_slot: Option<(&[u8; 16], u32)>) -> (Vec<u8>, usize) {
        let mut b = Vec::new();
        b.extend(4_u16.to_le_bytes()); // 版本 4
        push_null_marker(&mut b); // 颜色槽 #1
        for value in [0_i32; 3] {
            b.extend(value.to_le_bytes()); // int32 ×3
        }
        b.extend([0_u8; 2]); // byte ×2
        for value in [0_f64; 3] {
            b.extend(value.to_le_bytes()); // double ×3
        }
        for value in [0_i32; 2] {
            b.extend(value.to_le_bytes()); // int32 ×2
        }
        push_null_marker(&mut b); // 颜色槽 #2
        for value in [0_f64; 2] {
            b.extend(value.to_le_bytes()); // ⑧ double ×2
        }
        for value in [0_i32; 2] {
            b.extend(value.to_le_bytes()); // ⑧ int32 ×2
        }
        for value in [0_f64; 3] {
            b.extend(value.to_le_bytes()); // ⑧ double ×3
        }
        b.push(0); // ⑧ byte
        b.extend(0_f64.to_le_bytes()); // ⑧ double
        b.extend(0_i32.to_le_bytes()); // ⑧ int32
        b.extend(0_f64.to_le_bytes()); // ⑧ double
        b.push(0); // ⑧ byte
        b.extend([0x11_u8; 16]); // ⑨ 原样 GUID
        push_null_marker(&mut b); // ⑩ ITextPath 槽
        push_null_marker(&mut b); // ⑪ ISymbol 槽 #1
        b.extend(string_bytes("")); // ⑫ String
        b.extend(0_f64.to_le_bytes()); // ⑬ double
        b.extend(0_i32.to_le_bytes()); // ⑬ int32
        b.extend(0_f64.to_le_bytes()); // ⑬ double
        push_null_marker(&mut b); // ⑭ ISymbol 槽 #2
        let font_slot_offset = b.len();
        match font_slot {
            Some((class_id, key)) => push_entry_header(&mut b, class_id, key),
            None => push_null_marker(&mut b),
        }
        // 版本门尾段：>1 short、≥3 GUID、≥4 short。
        b.extend(0_u16.to_le_bytes());
        b.extend([0x22_u8; 16]);
        b.extend(0_u16.to_le_bytes());
        (b, font_slot_offset)
    }

    /// LELP 体中 ITextSymbol 槽的偏移（版本 7 形态的字节长度表）。
    fn lelp_symbol_slot_offset() -> usize {
        let mut n = 2; // 版本
        n += string_bytes("[JZDH]").len() + 2;
        n += 8 + 4 + 2 + 2 + 2 + 4 + 8 + 8 + 2;
        n += string_bytes("默认").len() + string_bytes("").len();
        n
    }

    /// EditTemplate 体（版本 2）：名称 T1、空串 B、16 字节 GUID、1 项
    /// [F1 → BSTR "x"]、尾串 点。
    fn edit_template_body() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(2_i16.to_le_bytes());
        b.extend(string_bytes("T1"));
        b.extend(string_bytes(""));
        b.extend(0x1122_3344_5566_7788_u64.to_le_bytes());
        b.extend(0x9900_1122_3344_5566_u64.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        b.extend(string_bytes("F1"));
        b.extend(8_i16.to_le_bytes()); // VT_BSTR
        b.extend(string_bytes("x"));
        b.extend(string_bytes("点"));
        b
    }

    /// EditTemplateManager 体（版本 2）：1 个 EditTemplate 条目 + 尾字节。
    fn edit_template_manager_body(et_key: u32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(2_i16.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        push_entry_header(&mut b, &EDIT_TEMPLATE_CLASS_ID, et_key);
        b.extend(edit_template_body());
        b.push(0x00); // 版本 ≥2 尾字节
        b
    }

    #[test]
    fn successor_tail_consumes_alpc_lelp_and_stops_at_unknown_symbol_slot() {
        // ⑯ 槽为 TextSymbol 之外的实现类（合成类标识）新对象：维持停止回退
        // 语义（键不保留、游标停在该槽位对象头起点）。
        let unknown = [0x66_u8; 16];
        let mut b = successor_prefix();
        push_entry_header(&mut b, &ANNOTATE_PROPS_COLLECTION_CLASS_ID, 59);
        b.extend(2_i16.to_le_bytes()); // ALPC 版本 2
        b.extend(1_i32.to_le_bytes()); // ALPC 计数 1
        push_entry_header(&mut b, &LABEL_ENGINE_PROPS_CLASS_ID, 60);
        let lelp_body_start = b.len();
        b.extend(lelp_body_v7(Some((&unknown, 61))));
        let text_symbol_slot = lelp_body_start + lelp_symbol_slot_offset();

        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let result = read_feature_layer_successor_10_1(&mut reader, 34, 1024, &mut table).unwrap();
        assert_eq!(
            result.stop,
            SuccessorStop10_1::AnnotatePropsBody(AnnotatePropsStop10_1::LelpBody(
                LelpStop10_1::SymbolSlotUnsupported {
                    class_id: unknown,
                    key: 61,
                }
            ))
        );
        assert!(table.contains(59));
        assert!(table.contains(60));
        assert!(!table.contains(61));
        assert_eq!(reader.remaining(), b.len() - text_symbol_slot);
        match &result.annotate_props {
            Some(AnnotatePropsSlot10_1::Collection(body)) => {
                assert_eq!(body.key, 59);
                assert_eq!(body.version, 2);
                assert_eq!(body.count, 1);
                assert_eq!(body.entries.len(), 1);
                assert!(matches!(
                    body.entries[0].slot,
                    AlpcEntrySlot10_1::LabelEngineProps(_)
                ));
                assert_eq!(body.entries[0].entry_key, None);
            }
            other => panic!("ALPC 槽分类不符：{other:?}"),
        }
    }

    #[test]
    fn successor_tail_parses_text_symbol_body_and_stops_at_font_slot() {
        // 样本形态接线轮推进：ALPC（键 59）→ LELP（键 60）→ ⑯ ITextSymbol
        // 槽 TextSymbol 新对象（键 61，整体解析）→ 体 IFontDisp 槽未知类新
        // 对象（键 62）= 透传停止点（体内键保留、游标停在内层对象头起点）。
        // 证据：display-symbols-round2 第一节（TextSymbol 虚表归属）+ 元素体
        // 轮 TextSymbol 体闭合（element_body::read_text_symbol_body_10_1）。
        let font_class_id = [0x77_u8; 16];
        let (text_symbol_body, font_slot_offset) = text_symbol_body_v4(Some((&font_class_id, 62)));
        let mut b = successor_prefix();
        push_entry_header(&mut b, &ANNOTATE_PROPS_COLLECTION_CLASS_ID, 59);
        b.extend(2_i16.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        push_entry_header(&mut b, &LABEL_ENGINE_PROPS_CLASS_ID, 60);
        b.extend(lelp_body_v7_prefix());
        let text_symbol_header_offset = b.len();
        push_entry_header(&mut b, &TEXT_SYMBOL_CLASS_ID, 61);
        b.extend_from_slice(&text_symbol_body);
        let font_slot = text_symbol_header_offset + 20 + font_slot_offset;

        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let result = read_feature_layer_successor_10_1(&mut reader, 34, 4096, &mut table).unwrap();
        assert_eq!(
            result.stop,
            SuccessorStop10_1::AnnotatePropsBody(AnnotatePropsStop10_1::LelpBody(
                LelpStop10_1::TextSymbolBody {
                    key: 61,
                    reason: ElementBodyStop10_1::NestedObjectNotClosed {
                        slot: "IFontDisp 字体槽",
                        offset: font_slot,
                        class_id: font_class_id,
                        key: 62,
                    },
                }
            ))
        );
        // TextSymbol 键按原生「登记先于持久化」语义保留；字体类键未登记。
        assert!(table.contains(59));
        assert!(table.contains(60));
        assert!(table.contains(61));
        assert!(!table.contains(62));
        assert_eq!(reader.remaining(), b.len() - font_slot);
        // LELP 体的 ⑯ 槽分类携带 TextSymbol 体；ALPC 条目键未读（LELP 体内
        // 停止）。
        match &result.annotate_props {
            Some(AnnotatePropsSlot10_1::Collection(body)) => match &body.entries[0].slot {
                AlpcEntrySlot10_1::LabelEngineProps(lelp) => match &lelp.text_symbol {
                    LelpTextSymbolSlot10_1::TextSymbol(ts) => {
                        assert_eq!(ts.version, 4);
                        assert!(ts.stop.is_some());
                    }
                    other => panic!("TextSymbol 槽分类不符：{other:?}"),
                },
                other => panic!("ALPC 条目分类不符：{other:?}"),
            },
            other => panic!("ALPC 槽分类不符：{other:?}"),
        }
        assert_eq!(body_entry_key_of(&result), None);
    }

    /// ALPC 首条目的条目键（供透传停止断言复用）。
    fn body_entry_key_of(result: &FeatureLayerSuccessor10_1) -> Option<i32> {
        match &result.annotate_props {
            Some(AnnotatePropsSlot10_1::Collection(body)) => body.entries[0].entry_key,
            other => panic!("ALPC 槽分类不符：{other:?}"),
        }
    }

    fn successor_through_short_300() -> Vec<u8> {
        use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;

        // 合成含多种扩展及其嵌套键的后继段前缀。
        let mut b = successor_prefix();
        // ALPC：版本 2、计数 0、尾 int32。
        push_entry_header(&mut b, &ANNOTATE_PROPS_COLLECTION_CLASS_ID, 59);
        b.extend(2_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_i32.to_le_bytes()); // 尾 int32
        b.extend((-1_i16).to_le_bytes()); // short +0x1bc
        // FIDSet（键 75，版本 1，计数 0）。
        push_entry_header(&mut b, &FEATURE_ID_SET_CLASS_ID, 75);
        b.extend(1_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_i32.to_le_bytes()); // 选择集计数 0
        b.extend(2_i16.to_le_bytes()); // short +0x1fe
        push_null_marker(&mut b); // IDisplayFilter 槽
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes()); // +0x22c
        push_null_marker(&mut b); // IName 槽
        b.extend(0_i32.to_le_bytes()); // 关系类计数 0
        b.extend(string_bytes("")); // String +0x18c
        push_null_marker(&mut b); // Envelope 槽 1
        b.extend(0_i32.to_le_bytes()); // 范围计数 0
        // 旧式扩展的体长之后写 CLSID 和对象体，无外层对象键。
        b.extend(11_i32.to_le_bytes());
        let etm_body = edit_template_manager_body(77);
        b.extend((etm_body.len() as i64).to_le_bytes());
        b.extend_from_slice(&EDIT_TEMPLATE_MANAGER_CLASS_ID);
        b.extend_from_slice(&etm_body);
        b.extend(2_i64.to_le_bytes());
        b.extend(mxd2qgz::layers::feature_layer_tail::REPRESENTATION_LAYER_EXTENSION_CLASS_ID);
        b.extend(1_i16.to_le_bytes());
        let mut route = 1_i16.to_le_bytes().to_vec();
        route.extend(string_bytes("路线编号"));
        push_null_marker(&mut route);
        push_null_marker(&mut route);
        b.extend((route.len() as i64).to_le_bytes());
        b.extend(mxd2qgz::layers::feature_layer_tail::ROUTE_LAYER_EXTENSION_CLASS_ID);
        b.extend(route);
        b.extend(7_i64.to_le_bytes());
        b.extend(mxd2qgz::layers::feature_layer_tail::HATCH_LAYER_EXTENSION_CLASS_ID);
        b.extend(1_i16.to_le_bytes());
        b.push(1);
        b.extend(0_i32.to_le_bytes());
        let mut tracking = 1_f64.to_le_bytes().to_vec();
        tracking.extend(0_i32.to_le_bytes());
        for _ in 0..2 {
            tracking.extend(13_u16.to_le_bytes());
            push_null_marker(&mut tracking);
        }
        tracking.extend([0; 8]); // 两个空 ATL BSTR 指针。
        b.extend((tracking.len() as i64).to_le_bytes());
        b.extend(mxd2qgz::layers::feature_layer_tail::TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID);
        b.extend(tracking);
        b.extend(19_i64.to_le_bytes());
        b.extend(mxd2qgz::layers::feature_layer_tail::E2_LAYER_EXTENSION_VALUES_CLASS_ID);
        b.extend(1_i16.to_le_bytes());
        push_null_marker(&mut b);
        b.push(1);
        b.extend(38_i64.to_le_bytes());
        b.extend(
            mxd2qgz::layers::feature_layer_tail::FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID,
        );
        b.extend(1_i16.to_le_bytes());
        push_null_marker(&mut b);
        push_null_marker(&mut b);
        b.extend(0_u32.to_le_bytes()); // 空 BSTR。
        b.extend(22_i64.to_le_bytes());
        b.extend(mxd2qgz::layers::feature_layer_tail::TIME_LAYER_TRACK_EXTENSION_CLASS_ID);
        b.extend(1_u16.to_le_bytes());
        push_null_marker(&mut b);
        b.extend(0_u32.to_le_bytes()); // 空时间窗口列表。
        b.extend(6_i64.to_le_bytes());
        b.extend(PROPERTY_SET_CLASS_ID);
        b.extend(1_u16.to_le_bytes());
        b.extend(0_u32.to_le_bytes());
        let server = [
            0xc9, 0x0e, 0xfd, 0xf7, 0x15, 0xf2, 0x7f, 0x4e, 0xb1, 0xf8, 0x94, 0xba, 0x0e, 0xb6,
            0x03, 0xb9,
        ];
        for is_new in [true, false] {
            b.extend((if is_new { 28_i64 } else { 22 }).to_le_bytes());
            b.extend(server);
            b.extend(1_u16.to_le_bytes());
            push_entry_header(&mut b, &PROPERTY_SET_CLASS_ID, 93);
            if is_new {
                b.extend(1_u16.to_le_bytes());
                b.extend(0_u32.to_le_bytes());
            }
        }
        b.extend(1.5_f64.to_le_bytes()); // double +0x1e8
        // ISymbol 槽：SMS 新对象（键 90，全闭合符号族整体解析）。
        push_entry_header(&mut b, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 90);
        b.extend(sms_body_v2());
        b.extend(0_i16.to_le_bytes()); // short +0x12e
        b.extend(0_i32.to_le_bytes()); // int32 +0x1f0
        // Envelope 槽 2：Envelope 新对象（键 91，版本 1 = 仅 4 个 double）。
        push_entry_header(&mut b, &ENVELOPE_CLASS_ID, 91);
        b.extend(1_i16.to_le_bytes());
        for value in [1.0_f64, 2.0, 3.0, 4.0] {
            b.extend(value.to_le_bytes());
        }
        // IUID 槽：UID 新对象（键 92，体 23 字节）。
        push_entry_header(&mut b, &UID_CLASS_ID, 92);
        b.extend(1_i16.to_le_bytes());
        b.push(1);
        b.extend_from_slice(&LABEL_ENGINE_PROPS_CLASS_ID);
        b.extend(42_i32.to_le_bytes()); // UID GUID 后的 int32
        b.extend(string_bytes("")); // String +0x28c
        b.extend(0_i16.to_le_bytes()); // short +0x1d8
        b.extend(0_f64.to_le_bytes()); // double +0x290
        b.extend(0_f64.to_le_bytes()); // double +0x298
        b.extend(0_i16.to_le_bytes()); // short +0x1da
        b.extend(0_i32.to_le_bytes()); // int32 +0x244
        b.extend(0_i16.to_le_bytes()); // short +0x1be
        b.extend(1_i32.to_le_bytes()); // PreJoin 计数；紧接一对 String
        b.extend(string_bytes("左表达式"));
        b.extend(string_bytes("右表达式"));
        b.extend(0_i32.to_le_bytes()); // int32 +0x2c0
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        for _ in 0..4 {
            b.extend(string_bytes(""));
        }
        push_null_marker(&mut b); // 表达式属性槽
        b.extend(0_i16.to_le_bytes()); // short +0x300
        for _ in 0..3 {
            b.extend(string_bytes(""));
        }
        b
    }

    #[test]
    fn successor_display_expression_stops_at_unknown_engine() {
        use mxd2qgz::layers::annotate_props::{
            DISPLAY_EXPRESSION_PROPS_CLASS_ID, ExpressionEngineSlot10_1,
        };

        let bytes = successor_through_short_300();
        let slot_offset = bytes.len() - (16 + 2 + 3 * 6);
        let mut data = bytes[..slot_offset].to_vec();
        push_entry_header(&mut data, &DISPLAY_EXPRESSION_PROPS_CLASS_ID, 124);
        let version_offset = data.len();
        data.extend(1_u16.to_le_bytes());
        data.extend(string_bytes("[NAME]"));
        data.extend((-1_i16).to_le_bytes());
        let engine_offset = data.len();
        // 首 6 字节也能按 short 与空字符串读取，便于捕捉误读后继字段。
        let mut unknown = [0xa5; 16];
        unknown[..8].copy_from_slice(&[0, 0, 2, 0, 0, 0, 0, 0]);
        push_entry_header(&mut data, &unknown, 125);
        data.extend_from_slice(&bytes[slot_offset + 16..]);

        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let layer = read_feature_layer_successor_10_1(&mut reader, 0x1b, 4096, &mut refs).unwrap();
        assert_eq!(
            layer.stop,
            SuccessorStop10_1::ExpressionPropsUnsupported {
                class_id: unknown,
                key: 125,
            },
        );
        assert!(layer.short_300.is_none());
        assert!(matches!(
            layer.expression_props,
            Some(mxd2qgz::layers::feature_layer_tail::ExpressionPropsSlot10_1::Display(body))
                if matches!(body.engine, ExpressionEngineSlot10_1::Unsupported { key: 125, .. })
        ));
        assert_eq!(reader.position(), engine_offset);
        assert!(refs.contains(124) && !refs.contains(125));

        data[version_offset..version_offset + 2].copy_from_slice(&2_u16.to_le_bytes());
        let mut reader = Reader::new(&data);
        let mut refs = table_with_keys(&[7]);
        let before = refs.clone();
        assert!(read_feature_layer_successor_10_1(&mut reader, 0x1b, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);
    }

    #[test]
    fn successor_legacy_time_slots_follow_each_version_gate() {
        use mxd2qgz::layers::annotate_props::{
            ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID, ExpressionEngineSlot10_1,
            HOT_LINK_EXPRESSION_PROPS_CLASS_ID,
        };
        use mxd2qgz::layers::dep_tail::{
            TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, TimeExtentSlot10_1,
            TimeReferenceSlot10_1,
        };
        use mxd2qgz::layers::feature_layer_tail::HotLinkSlot10_1;

        let mut prefix = successor_through_short_300();
        let legacy_offset = prefix.len();
        prefix.extend(7_i16.to_le_bytes());
        for value in ["起点", "终点", "", "时间"] {
            prefix.extend(string_bytes(value));
        }
        prefix.extend((-2_i16).to_le_bytes());
        prefix.extend(2.5_f64.to_le_bytes());
        prefix.extend(23_i32.to_le_bytes());

        for (version, which, registered, claimed) in [
            (0x1d, 0, TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID),
            (0x1e, 1, TIME_REFERENCE_CLASS_ID, TIME_EXTENT_CLASS_ID),
        ] {
            let mut bytes = prefix.clone();
            if which == 1 {
                push_null_marker(&mut bytes);
            }
            let offset = bytes.len();
            push_entry_header(&mut bytes, &claimed, 120);

            let mut prior = Vec::new();
            push_entry_header(&mut prior, &registered, 120);
            let mut refs = RefTable10_1::new();
            mxd2qgz::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&prior),
                &mut refs,
            )
            .unwrap();
            let mut reader = Reader::new(&bytes);
            let layer =
                read_feature_layer_successor_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            assert_eq!(
                layer.stop,
                SuccessorStop10_1::LegacyTimeSlotUnsupported {
                    class_id: claimed,
                    key: 120,
                    which,
                }
            );
            assert_eq!(reader.position(), offset);
            assert!(refs.contains(120));
        }

        let mut bytes = prefix;
        for version in 0x1c..=0x1f {
            match version {
                0x1d => {
                    push_entry_header(&mut bytes, &TIME_REFERENCE_CLASS_ID, 120);
                    bytes.extend(1_u16.to_le_bytes());
                    push_null_marker(&mut bytes);
                    bytes.extend([1, 0, 2, 0]);
                }
                0x1e => {
                    push_entry_header(&mut bytes, &TIME_EXTENT_CLASS_ID, 121);
                    bytes.extend(2_u16.to_le_bytes());
                    push_null_marker(&mut bytes);
                    push_null_marker(&mut bytes);
                    push_entry_header(&mut bytes, &TIME_REFERENCE_CLASS_ID, 120);
                    bytes.push(1);
                }
                0x1f => {
                    push_entry_header(&mut bytes, &HOT_LINK_EXPRESSION_PROPS_CLASS_ID, 122);
                    bytes.extend(1_u16.to_le_bytes());
                    bytes.extend(string_bytes("[URL]"));
                    bytes.extend((-1_i16).to_le_bytes());
                    push_entry_header(&mut bytes, &ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID, 123);
                    bytes.extend(2_u16.to_le_bytes());
                    bytes.extend((-1_i16).to_le_bytes());
                }
                _ => {}
            }
            let body_end = bytes.len();
            bytes.extend([0xa5; 20]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let layer =
                read_feature_layer_successor_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            assert_eq!(layer.stop, SuccessorStop10_1::TailCompleted);
            assert_eq!(reader.position(), body_end, "版本 {version:#x}");
            let scalars = layer.legacy_scalars.unwrap();
            assert_eq!(scalars.strings, ["起点", "终点", "", "时间"]);
            assert_eq!(
                (
                    scalars.short_62,
                    scalars.short_44,
                    scalars.double_40,
                    scalars.int32_38
                ),
                (7, -2, 2.5, 23)
            );
            assert!(layer.time_data.is_none() && layer.short_2ca.is_none());
            let slots = layer.legacy_time_slots.unwrap();
            assert_eq!(
                matches!(slots.time_reference, Some(TimeReferenceSlot10_1::Body(_))),
                version >= 0x1d
            );
            assert_eq!(
                matches!(slots.time_extent,
                Some(TimeExtentSlot10_1::Body(value)) if matches!(value.reference,
                    TimeReferenceSlot10_1::BackReference { key: 120, .. }) && value.tail_byte == Some(1)),
                version >= 0x1e
            );
            assert_eq!(
                matches!(layer.hot_link_expression_props,
                Some(HotLinkSlot10_1::Body(value)) if value.string == "[URL]" &&
                    matches!(&value.engine, ExpressionEngineSlot10_1::VbscriptEngine(engine) if engine.key == 123)),
                version == 0x1f
            );
            assert_eq!(refs.contains(123), version == 0x1f);
            bytes.truncate(body_end);
            for end in legacy_offset..body_end {
                let mut reader = Reader::new(&bytes[..end]);
                let mut table = table_with_keys(&[7]);
                assert!(
                    read_feature_layer_successor_10_1(&mut reader, version, 4096, &mut table)
                        .is_err()
                );
                assert_eq!(reader.position(), 0);
                assert_eq!(table.len(), 1);
            }
        }

        let mut reader = Reader::new(&bytes[..bytes.len() - 4]);
        let mut table = table_with_keys(&[123]);
        let layer = read_feature_layer_successor_10_1(&mut reader, 0x1f, 4096, &mut table).unwrap();
        assert_eq!(layer.stop, SuccessorStop10_1::TailCompleted);
        assert_eq!(reader.remaining(), 0);
        assert!(matches!(
            layer.hot_link_expression_props,
            Some(HotLinkSlot10_1::Body(value))
                if matches!(value.engine, ExpressionEngineSlot10_1::BackReference { key: 123, .. })
        ));

        // 非空热链接对象内的新未知引擎应保留外层键，并在内层对象头处停止。
        let inner_offset = bytes.len() - 24;
        let unsupported = [0xa5; 16];
        bytes[inner_offset..inner_offset + 16].copy_from_slice(&unsupported);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let layer = read_feature_layer_successor_10_1(&mut reader, 0x1f, 4096, &mut table).unwrap();
        assert_eq!(
            layer.stop,
            SuccessorStop10_1::ExpressionPropsUnsupported {
                class_id: unsupported,
                key: 123,
            }
        );
        assert_eq!(reader.position(), inner_offset);
        assert!(table.contains(122));
        assert!(!table.contains(123));
    }

    #[test]
    fn successor_tail_full_path_through_edit_template_extension_to_completion() {
        let mut b = successor_through_short_300();
        // TimeData（版本 2）：short、空槽、尾 short。
        b.extend(2_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        push_null_marker(&mut b);
        b.extend(0_i16.to_le_bytes());
        // TimeTable（版本 1）：String ×4 + 空槽。
        b.extend(1_i16.to_le_bytes());
        for _ in 0..4 {
            b.extend(string_bytes(""));
        }
        push_null_marker(&mut b);
        // TimeDimension（版本 1）：String ×2。
        b.extend(1_i16.to_le_bytes());
        b.extend(string_bytes(""));
        b.extend(string_bytes(""));
        // TimeDisplay（版本 1）：short、double、int32、double、int32。
        b.extend(1_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        b.extend(0_f64.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_f64.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        push_null_marker(&mut b); // {E720618B} 槽（版本 >0x20）
        b.extend(0_i16.to_le_bytes()); // short +0x2ca（版本 >0x21）

        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let result = read_feature_layer_successor_10_1(&mut reader, 34, 4096, &mut table).unwrap();
        assert_eq!(result.stop, SuccessorStop10_1::TailCompleted);
        assert_eq!(reader.remaining(), 0);
        let left_pair = string_bytes("左表达式");
        let pair_at = b
            .windows(left_pair.len())
            .position(|window| window == left_pair)
            .unwrap();
        let pre_join_at = pair_at - 4;
        for cut in pre_join_at..b.len() {
            let mut truncated = Reader::new(&b[..cut]);
            let mut refs = table_with_keys(&[7]);
            assert!(
                read_feature_layer_successor_10_1(&mut truncated, 34, 4096, &mut refs).is_err(),
                "PreJoin 至后继尾部在偏移 {cut} 截断应失败"
            );
            assert_eq!(truncated.position(), 0, "截断不得推进游标（cut={cut}）");
            assert_eq!(refs.len(), 1, "截断不得提交引用表（cut={cut}）");
            assert!(refs.contains(7));
        }
        assert!(table.contains(59));
        assert!(table.contains(75));
        // 扩展段 ETM 无 ObjectStream 键（第十轮）：仅其 EditTemplate 条目
        // （键 77）登记。
        assert!(!table.contains(76));
        assert!(table.contains(77));
        assert!(matches!(
            &result.feature_id_set,
            Some(FeatureIdSetSlot10_1::FeatureIdSet(body)) if body.count == 0
        ));
        assert!(matches!(
            result.display_filter,
            Some(
                mxd2qgz::layers::feature_layer_tail::DisplayFilterSlot10_1::Reference(
                    RefSlot10_1::Null
                )
            )
        ));
        assert_eq!(result.shorts_21c_220_21e, Some([0, 0, 0]));
        assert_eq!(
            result.pre_join.as_ref().map(|block| block.pairs.as_slice()),
            Some(&[("左表达式".to_owned(), "右表达式".to_owned())][..])
        );
        assert_eq!(result.int32_2c0, Some(0));
        assert_eq!(result.extension_count, Some(11));
        assert!(matches!(
            &result.extensions[8],
            LayerExtension10_1::PropertySet(Some(properties)) if properties.count == 0
        ));
        assert!(matches!(
            &result.extensions[9],
            LayerExtension10_1::ServerLayerExtension(Some(server))
                if server.properties.body.as_ref().is_some_and(|body| body.count == 0)
        ));
        assert!(matches!(
            &result.extensions[10],
            LayerExtension10_1::ServerLayerExtension(Some(server))
                if matches!(server.properties.entry,
                    mxd2qgz::storage::object_refs::ObjectEntry10_1::BackReference { key: 93, .. })
        ));
        assert!(table.contains(93));
        assert_eq!(
            result.extensions[1],
            LayerExtension10_1::RepresentationLayerExtension(Some(1))
        );
        let LayerExtension10_1::RouteLayerExtension(Some(route)) = &result.extensions[2] else {
            panic!("预期 RouteLayerExtension");
        };
        assert_eq!(route.string, "路线编号");
        assert!(route.anomaly.body.is_none());
        assert!(route.identify.body.is_none());
        let LayerExtension10_1::HatchLayerExtension(Some(hatch)) = &result.extensions[3] else {
            panic!("预期 HatchLayerExtension");
        };
        assert_eq!(hatch.flag, 1);
        assert!(hatch.classes.is_empty());
        let LayerExtension10_1::TrackingFilterLayerExtension(Some(tracking)) =
            &result.extensions[4]
        else {
            panic!("预期 TrackingFilterLayerExtension");
        };
        assert_eq!(tracking.properties, [None, None]);
        assert_eq!(tracking.strings, [None, None]);
        let LayerExtension10_1::E2LayerExtensionValues(Some(e2)) = &result.extensions[5] else {
            panic!("预期 E2LayerExtensionValues");
        };
        assert_eq!(e2.flag, 1);
        assert!(
            matches!(&result.extensions[6], LayerExtension10_1::FeatureServiceLayerEditExtension(Some(edit)) if edit.text.is_empty())
        );
        assert!(
            matches!(&result.extensions[7], LayerExtension10_1::TimeLayerTrackExtension(Some(track)) if track.windows.is_empty())
        );
        match &result.extensions[0] {
            LayerExtension10_1::EditTemplateManager(etm) => {
                // 扩展段条目无 ObjectStream 键：body.key 恒 0（第十轮）。
                assert_eq!(etm.key, 0);
                assert_eq!(etm.version, 2);
                assert_eq!(etm.count, 1);
                assert_eq!(etm.tail_byte, Some(0));
                match &etm.entries[0] {
                    mxd2qgz::layers::annotate_props::EtmEntry10_1::EditTemplate(et) => {
                        assert_eq!(et.key, 77);
                        assert_eq!(et.name, "T1");
                        assert_eq!(et.fields.len(), 1);
                        assert_eq!(et.tail_string.as_deref(), Some("点"));
                    }
                    other => panic!("EditTemplate 条目分类不符：{other:?}"),
                }
            }
            other => panic!("扩展条目分类不符：{other:?}"),
        }
        // 第十轮接线断言：㉖ SMS 体、㉘ Envelope 体、㉙ UID 体。
        assert!(matches!(
            result.symbol,
            Some(mxd2qgz::layers::feature_layer_tail::SymbolSlotFull10_1::SimpleMarkerSymbol(_))
        ));
        assert_eq!(result.stop, SuccessorStop10_1::TailCompleted);
        match result.cached_dataset_extent() {
            Some(EnvelopeSlot10_1::Envelope(env)) => {
                assert_eq!(env.version, 1);
                assert_eq!(env.coords, [1.0, 2.0, 3.0, 4.0]);
            }
            other => panic!("Envelope 槽 2 分类不符：{other:?}"),
        }
        match &result.uid {
            Some(mxd2qgz::layers::feature_layer_tail::UidSlot10_1::Uid(uid)) => {
                assert_eq!(uid.key, 92);
                assert_eq!(uid.version, 1);
                assert_eq!(uid.flag, 1);
                assert_eq!(uid.guid, LABEL_ENGINE_PROPS_CLASS_ID);
            }
            other => panic!("IUID 槽分类不符：{other:?}"),
        }
        assert!(result.time_data.is_some());
        assert!(result.time_table_definition.is_some());
        assert!(result.time_dimension_definition.is_some());
        assert!(result.time_data_display.is_some());
        assert_eq!(result.short_2ca, Some(0));

        // 对同一完整后继段替换 ㉖ 为回引，两种头部 CLSID 均故意与登记类不同。
        let mut symbol_header = Vec::new();
        push_entry_header(&mut symbol_header, &SIMPLE_MARKER_SYMBOL_CLASS_ID, 90);
        let slot_start = b
            .windows(symbol_header.len())
            .position(|window| window == symbol_header)
            .unwrap();
        for class_id in [
            mxd2qgz::objects::display_symbol::TEMPLATE_CLASS_ID,
            mxd2qgz::objects::display_symbol::LINE_DECORATION_CLASS_ID,
        ] {
            let mut data = b[..slot_start + symbol_header.len()].to_vec();
            data[slot_start..slot_start + 16].copy_from_slice(&class_id);
            let mut reader = Reader::new(&data);
            let mut refs = RefTable10_1::new();
            let result =
                read_feature_layer_successor_10_1(&mut reader, 34, 4096, &mut refs).unwrap();
            assert_eq!(
                result.stop,
                SuccessorStop10_1::SymbolSlotUnsupported { class_id, key: 90 }
            );
            assert_eq!(reader.position(), slot_start);
            assert!(!refs.contains(90));
        }
        for (registered, header) in [
            (SIMPLE_MARKER_SYMBOL_CLASS_ID, RGB_COLOR_CLASS_ID),
            (RGB_COLOR_CLASS_ID, SIMPLE_MARKER_SYMBOL_CLASS_ID),
        ] {
            let mut data = b.clone();
            data.drain(
                slot_start + symbol_header.len()
                    ..slot_start + symbol_header.len() + sms_body_v2().len(),
            );
            data[slot_start..slot_start + 16].copy_from_slice(&header);
            let mut definition = Vec::new();
            push_entry_header(&mut definition, &registered, 90);
            let mut refs = RefTable10_1::new();
            mxd2qgz::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&definition),
                &mut refs,
            )
            .unwrap();
            let mut reader = Reader::new(&data);
            let result =
                read_feature_layer_successor_10_1(&mut reader, 34, 4096, &mut refs).unwrap();
            assert!(refs.contains(90));
            if registered == SIMPLE_MARKER_SYMBOL_CLASS_ID {
                assert_eq!(result.stop, SuccessorStop10_1::TailCompleted);
                assert_eq!(
                    result.symbol,
                    Some(
                        mxd2qgz::layers::feature_layer_tail::SymbolSlotFull10_1::BackReference {
                            class_id: registered,
                            key: 90,
                        }
                    )
                );
                assert_eq!(reader.remaining(), 0);
            } else {
                assert_eq!(
                    result.stop,
                    SuccessorStop10_1::SymbolSlotUnsupported {
                        class_id: registered,
                        key: 90,
                    }
                );
                assert_eq!(reader.position(), slot_start);
                assert!(!refs.contains(91));
            }
        }
    }

    #[test]
    fn alpc_body_version_gate_and_entry_stop_rollback() {
        // 版本 3 → 拒绝（有符号比较 >2）。
        let mut b = Vec::new();
        b.extend(3_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let err =
            read_annotate_props_collection_body_10_1(&mut reader, 64, &mut table, 59).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.remaining(), b.len());

        // 条目非 LELP 新对象 → 成功停止 + 回退 + 键不保留。
        let mut b = Vec::new();
        b.extend(2_i16.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        push_entry_header(&mut b, &RGB_COLOR_CLASS_ID, 90);
        b.extend(rgb_color_body(0));
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let body =
            read_annotate_props_collection_body_10_1(&mut reader, 256, &mut table, 59).unwrap();
        assert_eq!(
            body.stop,
            Some(AnnotatePropsStop10_1::EntryUnsupported {
                class_id: RGB_COLOR_CLASS_ID,
                key: 90,
                index: 0,
            })
        );
        assert!(!table.contains(90));
        assert_eq!(reader.remaining(), b.len() - 6); // 游标停在条目头起点
    }

    #[test]
    fn lelp_body_version_gate_and_truncation_sweep() {
        // 版本 8 → 拒绝。
        let mut b = Vec::new();
        b.extend(8_i16.to_le_bytes());
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        assert!(read_label_engine_props_body_10_1(&mut reader, 64, &mut table, 60).is_err());
        assert_eq!(reader.remaining(), b.len());

        // 完整体（版本 7，槽全空）逐字节截断扫描：任何截断都失败且不推进。
        let full = lelp_body_v7(None);
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let mut table = RefTable10_1::new();
            assert!(
                read_label_engine_props_body_10_1(&mut reader, 64, &mut table, 60).is_err(),
                "截断到 {cut} 字节应失败"
            );
            assert_eq!(reader.remaining(), cut, "截断扫描 {cut}：游标不得推进");
        }
        // 完整体成功。
        let mut reader = Reader::new(&full);
        let mut table = RefTable10_1::new();
        let body = read_label_engine_props_body_10_1(&mut reader, 64, &mut table, 60).unwrap();
        assert_eq!(body.stop, None);
        assert_eq!(body.version, 7);
        assert_eq!(body.expression, "[JZDH]");
        assert_eq!(body.name, "默认");
        assert_eq!(body.annotation_class_id, Some(0));
        assert!(body.short_ver1.is_none()); // 版本 7 不读 ④
        assert!(body.short_ver6.is_none()); // 版本 ≠6
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn lelp_legacy_short_and_annotation_class_id_have_separate_version_gates() {
        let mut v4 = lelp_body_v7(None);
        v4[..2].copy_from_slice(&4_u16.to_le_bytes());
        v4.truncate(v4.len() - 4);
        let mut v6 = lelp_body_v7(None);
        v6[..2].copy_from_slice(&6_u16.to_le_bytes());
        let id_offset = v6.len() - 4;
        v6[id_offset..].copy_from_slice(&31_i32.to_le_bytes());
        v6.extend(123_i16.to_le_bytes());
        for (mut bytes, id, legacy_short) in [(v4, None, None), (v6, Some(31), Some(123))] {
            bytes.push(0x9a);
            let mut reader = Reader::new(&bytes);
            let body =
                read_label_engine_props_body_10_1(&mut reader, 64, &mut RefTable10_1::new(), 60)
                    .unwrap();
            assert_eq!(body.annotation_class_id, id);
            assert_eq!(body.short_ver6, legacy_short);
            assert_eq!(reader.read_bytes(1).unwrap(), [0x9a]);
        }
    }

    #[test]
    fn feature_id_set_body_contract() {
        // 正常：版本 1、计数 2、两个 ID。
        let mut b = Vec::new();
        b.extend(1_i16.to_le_bytes());
        b.extend(2_i32.to_le_bytes());
        b.extend(10_i32.to_le_bytes());
        b.extend((-3_i32).to_le_bytes());
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let body = read_feature_id_set_body_10_1(&mut reader, &mut table, 75).unwrap();
        assert_eq!(body.version, 1);
        assert_eq!(body.ids, vec![10, -3]);
        assert_eq!(reader.remaining(), 0);

        // 版本 2 → 拒绝；负版本位模式 → 拒绝（较原生更严）。
        for version in [2_i16, -1_i16] {
            let mut b = Vec::new();
            b.extend(version.to_le_bytes());
            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            assert!(read_feature_id_set_body_10_1(&mut reader, &mut table, 75).is_err());
            assert_eq!(reader.remaining(), b.len());
        }
        // 负计数 → 原生语义跳过循环（不报错），原样保留。
        let mut b = Vec::new();
        b.extend(1_i16.to_le_bytes());
        b.extend((-5_i32).to_le_bytes());
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let body = read_feature_id_set_body_10_1(&mut reader, &mut table, 75).unwrap();
        assert_eq!(body.count, -5);
        assert!(body.ids.is_empty());
        assert_eq!(reader.remaining(), 0);
        // 截断扫描。
        let full = {
            let mut b = Vec::new();
            b.extend(1_i16.to_le_bytes());
            b.extend(1_i32.to_le_bytes());
            b.extend(7_i32.to_le_bytes());
            b
        };
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let mut table = RefTable10_1::new();
            assert!(read_feature_id_set_body_10_1(&mut reader, &mut table, 75).is_err());
            assert_eq!(reader.remaining(), cut);
        }
    }

    #[test]
    fn edit_template_manager_contract_and_extension_mismatch() {
        // 正常体 + 版本门。
        let body = edit_template_manager_body(77);
        let mut reader = Reader::new(&body);
        let mut table = RefTable10_1::new();
        let parsed =
            read_edit_template_manager_body_10_1(&mut reader, 256, &mut table, 76).unwrap();
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.stop, None);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(reader.remaining(), 0);

        let mut over = Vec::new();
        over.extend(3_i16.to_le_bytes());
        let mut reader = Reader::new(&over);
        let mut table = RefTable10_1::new();
        assert!(read_edit_template_manager_body_10_1(&mut reader, 256, &mut table, 76).is_err());

        // 循环条目非 EditTemplate → 停止 + 回退。
        let mut b = Vec::new();
        b.extend(2_i16.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        push_entry_header(&mut b, &RGB_COLOR_CLASS_ID, 90);
        b.extend(rgb_color_body(0));
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let parsed =
            read_edit_template_manager_body_10_1(&mut reader, 256, &mut table, 76).unwrap();
        assert_eq!(
            parsed.stop,
            Some(EditTemplateManagerStop10_1::EntryUnsupported {
                class_id: RGB_COLOR_CLASS_ID,
                key: 90,
                index: 0,
            })
        );
        assert!(!table.contains(90));

        // 扩展段尺寸声明与实际消费不一致 → 显式错误（事务性回退）。
        let mut b = successor_prefix();
        push_entry_header(&mut b, &ANNOTATE_PROPS_COLLECTION_CLASS_ID, 59);
        b.extend(2_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        push_entry_header(&mut b, &FEATURE_ID_SET_CLASS_ID, 75);
        b.extend(1_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        push_null_marker(&mut b);
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i16.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        push_null_marker(&mut b);
        b.extend(0_i32.to_le_bytes());
        b.extend(string_bytes(""));
        push_null_marker(&mut b);
        b.extend(0_i32.to_le_bytes());
        b.extend(1_i32.to_le_bytes()); // 扩展计数 1
        let mut etm = Vec::new();
        push_entry_header(&mut etm, &EDIT_TEMPLATE_MANAGER_CLASS_ID, 76);
        etm.extend(edit_template_manager_body(77));
        b.extend((etm.len() as i64 + 5).to_le_bytes()); // 尺寸多声明 5 字节
        b.extend_from_slice(&etm);
        let mut reader = Reader::new(&b);
        let mut table = RefTable10_1::new();
        let err = read_feature_layer_successor_10_1(&mut reader, 34, 4096, &mut table).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        // 事务性：登记表与游标都不动。
        assert!(!table.contains(59));
        assert_eq!(reader.remaining(), b.len());
    }
    // -------------------------------------------------------------------------
    // 第十轮:BOLP 族、标注表达式引擎、UID/表达式属性族、扩展段样本形态。
    // 字段值取自样本 `temp/r6-maps.bin` 层 32 链(`0xda3..0x155a`)的逐字节
    // 对位;样本对位为辅助观察,本模块仍只验证读取器契约。
    // -------------------------------------------------------------------------

    #[cfg(test)]
    mod round10 {
        use super::*;
        use mxd2qgz::layers::annotate_props::{
            BASIC_OVERPOSTER_LAYER_PROPS_CLASS_ID, BolpSlot10_1, BolpStop10_1,
            ExpressionEngineSlot10_1, LINE_LABEL_PLACEMENT_PRIORITIES_CLASS_ID,
            LINE_LABEL_POSITION_CLASS_ID, POINT_PLACEMENT_PRIORITIES_CLASS_ID,
            read_annotation_vbscript_engine_body_10_1, read_bolp_body_10_1,
            read_display_expression_props_prefix_10_1, read_hot_link_expression_props_body_10_1,
            read_line_label_placement_priorities_body_10_1, read_line_label_position_body_10_1,
            read_point_placement_priorities_body_10_1,
        };
        use mxd2qgz::layers::feature_layer_tail::read_uid_body_10_1;

        /// BOLP 体(样本形态,键 69):20B 前缀 + 三槽 + 槽间块 + 44B 尾段。
        fn bolp_body_sample() -> Vec<u8> {
            let mut b = Vec::new();
            // ① 前缀:ver 6、short ×3 = 0、int ×3 = 0/3/1。
            b.extend(6_i16.to_le_bytes());
            for _ in 0..3 {
                b.extend(0_i16.to_le_bytes());
            }
            b.extend(0_i32.to_le_bytes());
            b.extend(3_i32.to_le_bytes());
            b.extend(1_i32.to_le_bytes());
            // ② 槽 1 LineLabelPosition(键 70):ver 2、short ×11、double、尾 short。
            push_entry_header(&mut b, &LINE_LABEL_POSITION_CLASS_ID, 70);
            b.extend(2_i16.to_le_bytes());
            for value in [-1_i16, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0] {
                b.extend(value.to_le_bytes());
            }
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            // ③ 槽 2 LineLabelPlacementPriorities(键 71):ver 1、int ×15。
            push_entry_header(&mut b, &LINE_LABEL_PLACEMENT_PRIORITIES_CLASS_ID, 71);
            b.extend(1_i16.to_le_bytes());
            for value in [0_i32, 3, 3, 3, 0, 0, 3, 3, 3, 0, 0, 3, 3, 3, 0] {
                b.extend(value.to_le_bytes());
            }
            // ④ 槽间块:short 0、int 0、double 0.0、short 0。
            b.extend(0_i16.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            // ⑤ 槽 3 PointPlacementPriorities(键 72):ver 1、int ×8。
            push_entry_header(&mut b, &POINT_PLACEMENT_PRIORITIES_CLASS_ID, 72);
            b.extend(1_i16.to_le_bytes());
            for value in [2_i32, 2, 1, 2, 2, 3, 3, 3] {
                b.extend(value.to_le_bytes());
            }
            // ⑥ 公共尾段和 v3/v4/v5/v6 的附加字段。
            b.extend(0_i32.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(1_i32.to_le_bytes());
            b.extend(string_bytes(""));
            b.extend(0_i16.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b
        }

        #[test]
        fn bolp_body_parses_sample_shape_with_nested_slots() {
            let mut b = bolp_body_sample();
            push_entry_header(&mut b, &LABEL_ENGINE_PROPS_CLASS_ID, 999); // 哨兵
            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let body = read_bolp_body_10_1(&mut reader, 1024, &mut table, 69).unwrap();
            assert_eq!(body.key, 69);
            assert_eq!(body.version, 6);
            assert_eq!(body.shorts, [0, 0, 0]);
            assert_eq!(body.ints, [0, 3, 1]);
            assert_eq!(body.stop, None);
            match &body.label_position {
                mxd2qgz::layers::annotate_props::LineLabelPositionSlot10_1::Body(llp) => {
                    assert_eq!(llp.key, 70);
                    assert_eq!(llp.version, 2);
                    assert_eq!(llp.shorts, [-1, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0]);
                    assert_eq!(llp.double, 0.0);
                    assert_eq!(llp.tail_short, Some(0));
                }
                other => panic!("LLP 槽分类不符：{other:?}"),
            }
            match &body.priorities_line {
                mxd2qgz::layers::annotate_props::PlacementPrioritiesSlot10_1::Line(pp) => {
                    assert_eq!(pp.key, 71);
                    assert_eq!(pp.ints, [0, 3, 3, 3, 0, 0, 3, 3, 3, 0, 0, 3, 3, 3, 0]);
                }
                other => panic!("LLPP 槽分类不符：{other:?}"),
            }
            assert_eq!(body.between.short_a, 0);
            assert_eq!(body.between.double, 0.0);
            match &body.priorities_point {
                mxd2qgz::layers::annotate_props::PlacementPrioritiesSlot10_1::Point(pp) => {
                    assert_eq!(pp.key, 72);
                    assert_eq!(pp.ints, [2, 2, 1, 2, 2, 3, 3, 3]);
                }
                other => panic!("PPP 槽分类不符：{other:?}"),
            }
            assert_eq!(body.tail.count, 0);
            assert!(body.tail.doubles.is_empty());
            assert_eq!(body.tail.double_24, Some(0.0));
            assert_eq!(body.tail.v4_fields, Some((1, String::new(), 0)));
            assert_eq!(body.tail.v5_fields, Some((0, 0)));
            assert_eq!(body.tail.short_5a, Some(-1));
            // 哨兵未消费:BOLP 体完整结束。
            assert_eq!(reader.remaining(), 20);
            // 键已登记(登记先于持久化)。
            assert!(table.contains(70));
            assert!(table.contains(71));
            assert!(table.contains(72));
        }

        #[test]
        fn bolp_tail_follows_native_version_gates() {
            let sample = bolp_body_sample();
            for version in [-1_i16, 0, 1, 2, 3, 4, 5, 6] {
                let mut bytes = sample[..sample.len() - 44].to_vec();
                bytes[..2].copy_from_slice(&version.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend(1.5_f64.to_le_bytes());
                bytes.extend((-2.5_f64).to_le_bytes());
                bytes.extend(8.0_f64.to_le_bytes());
                bytes.extend(9_i32.to_le_bytes());
                if version <= 1 {
                    bytes.extend(string_bytes("旧"));
                    bytes.extend((-10_i32).to_le_bytes());
                }
                if version >= 3 {
                    bytes.extend(11.5_f64.to_le_bytes());
                }
                if version >= 4 {
                    bytes.extend(12_i32.to_le_bytes());
                    bytes.extend(string_bytes("标注"));
                    bytes.extend((-13_i16).to_le_bytes());
                }
                if version >= 5 {
                    bytes.extend(14_i32.to_le_bytes());
                    bytes.extend((-15_i16).to_le_bytes());
                }
                if version >= 6 {
                    bytes.extend((-16_i16).to_le_bytes());
                }

                let end = bytes.len();
                bytes.push(0xa5);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let body = read_bolp_body_10_1(&mut reader, 1024, &mut refs, 69).unwrap();
                assert_eq!(body.version as i16, version);
                assert_eq!(body.tail.doubles, [1.5, -2.5]);
                assert_eq!((body.tail.double_44, body.tail.int_70), (8.0, 9));
                assert_eq!(
                    body.tail.legacy_fields,
                    (version <= 1).then(|| ("旧".into(), -10))
                );
                assert_eq!(body.tail.double_24, (version >= 3).then_some(11.5));
                assert_eq!(
                    body.tail.v4_fields,
                    (version >= 4).then(|| (12, "标注".into(), -13))
                );
                assert_eq!(body.tail.v5_fields, (version >= 5).then_some((14, -15)));
                assert_eq!(body.tail.short_5a, (version >= 6).then_some(-16));
                assert_eq!(reader.position(), end);
                for cut in sample.len() - 44..end {
                    let mut reader = Reader::new(&bytes[..cut]);
                    let mut refs = RefTable10_1::new();
                    assert!(read_bolp_body_10_1(&mut reader, 1024, &mut refs, 69).is_err());
                    assert_eq!(reader.position(), 0);
                    assert!(refs.is_empty());
                }
            }
            let invalid = 7_i16.to_le_bytes();
            let mut reader = Reader::new(&invalid);
            assert!(read_bolp_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 69).is_err());
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn bolp_slot_unknown_class_stops_and_rolls_back() {
            let b = bolp_body_sample();
            // 槽 1 改为未知类新对象:整体停止 + 回退到该槽起点。
            let unknown = [0x66_u8; 16];
            let slot_offset = 20; // 前缀后
            let mut patched = b[..slot_offset].to_vec();
            push_entry_header(&mut patched, &unknown, 70);
            patched.extend_from_slice(&b[slot_offset + 20 + 4..]);
            let mut reader = Reader::new(&patched);
            let mut table = RefTable10_1::new();
            let body = read_bolp_body_10_1(&mut reader, 1024, &mut table, 69).unwrap();
            assert_eq!(
                body.stop,
                Some(BolpStop10_1::LabelPositionSlotUnsupported {
                    class_id: unknown,
                    key: 70,
                })
            );
            assert!(!table.contains(70));
            // 游标停在槽位起点(前缀之后)。
            assert_eq!(reader.remaining(), patched.len() - slot_offset);
        }

        #[test]
        fn label_position_version_gate_rejects_over_two() {
            let b = 3_i16.to_le_bytes().to_vec();
            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            assert!(read_line_label_position_body_10_1(&mut reader, 1024, &mut table, 1).is_err());
            // LLPP / PPP 版本 >1 拒绝。
            let b2 = 2_i16.to_le_bytes().to_vec();
            let mut reader2 = Reader::new(&b2);
            assert!(
                read_line_label_placement_priorities_body_10_1(&mut reader2, 1024, &mut table, 1)
                    .is_err()
            );
            let b3 = 2_i16.to_le_bytes().to_vec();
            let mut reader3 = Reader::new(&b3);
            assert!(
                read_point_placement_priorities_body_10_1(&mut reader3, 1024, &mut table, 1)
                    .is_err()
            );
        }

        #[test]
        fn annotation_vbscript_engine_body_parses_and_gates() {
            let mut b = 2_i16.to_le_bytes().to_vec();
            b.extend((-1_i16).to_le_bytes());
            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let body = read_annotation_vbscript_engine_body_10_1(&mut reader, 1024, &mut table, 73)
                .unwrap();
            assert_eq!(body.version, 2);
            assert_eq!(body.short, Some(-1));
            assert_eq!(reader.remaining(), 0);
            // 版本 >2 拒绝。
            let over = 3_i16.to_le_bytes().to_vec();
            let mut reader2 = Reader::new(&over);
            assert!(
                read_annotation_vbscript_engine_body_10_1(&mut reader2, 1024, &mut table, 73)
                    .is_err()
            );
        }

        #[test]
        fn hot_link_expression_props_parses_sample_shape() {
            let mut b = 1_i16.to_le_bytes().to_vec();
            b.extend(2_u32.to_le_bytes()); // String cb=2(空串含终止零)
            b.extend(0_u16.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend_from_slice(&[0u8; 16]);
            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let body = read_hot_link_expression_props_body_10_1(&mut reader, 1024, &mut table, 85)
                .unwrap();
            assert_eq!(body.key, 85);
            assert_eq!(body.version, 1);
            assert_eq!(body.string, "");
            assert_eq!(body.short, -1);
            assert!(matches!(
                body.engine,
                mxd2qgz::layers::annotate_props::ExpressionEngineSlot10_1::Null
            ));
            assert_eq!(reader.remaining(), 0);

            b[..2].copy_from_slice(&2_u16.to_le_bytes());
            let mut reader = Reader::new(&b);
            assert!(
                read_hot_link_expression_props_body_10_1(&mut reader, 1024, &mut table, 85)
                    .is_err()
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn uid_body_parses_both_sample_shapes() {
            // 形态 A:ver 1、flag 1、GUID = LELP CLSID(样本键 82)。
            let mut a = 1_i16.to_le_bytes().to_vec();
            a.push(1);
            a.extend_from_slice(&LABEL_ENGINE_PROPS_CLASS_ID);
            a.extend(42_i32.to_le_bytes());
            let mut reader = Reader::new(&a);
            let _table = RefTable10_1::new();
            let body = read_uid_body_10_1(&mut reader, 82).unwrap();
            assert_eq!(body.key, 82);
            assert_eq!(body.version, 1);
            assert_eq!(body.flag, 1);
            assert_eq!(body.guid, LABEL_ENGINE_PROPS_CLASS_ID);
            assert_eq!(body.value_18, 42);
            assert_eq!(reader.remaining(), 0);
            // 形态 B:ver 2、flag 0、零 GUID(样本键 82 的零值实例)。
            let mut zero = 2_i16.to_le_bytes().to_vec();
            zero.push(0);
            zero.extend_from_slice(&[0u8; 16]);
            zero.extend((-7_i32).to_le_bytes());
            let mut reader2 = Reader::new(&zero);
            let body2 = read_uid_body_10_1(&mut reader2, 82).unwrap();
            assert_eq!(body2.version, 2);
            assert_eq!(body2.flag, 0);
            assert_eq!(body2.guid, [0u8; 16]);
            assert_eq!(body2.value_18, -7);
            for cut in 0..zero.len() {
                let mut truncated = Reader::new(&zero[..cut]);
                assert!(read_uid_body_10_1(&mut truncated, 82).is_err());
                assert_eq!(truncated.position(), 0);
            }
        }

        #[test]
        fn reference_sample_uid_ends_before_feature_layer_string() {
            use std::io::Read;

            let path =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/测试底图.mxd");
            let mut compound = cfb::open(path).unwrap();
            let mut maps = Vec::new();
            compound
                .open_stream("/Maps")
                .unwrap()
                .read_to_end(&mut maps)
                .unwrap();
            assert_eq!(&maps[0x14a9..0x14b9], &UID_CLASS_ID);
            assert_eq!(&maps[0x14b9..0x14bd], &82_u32.to_le_bytes());
            let mut reader = Reader::new(&maps[0x14bd..0x14da]);
            let body = read_uid_body_10_1(&mut reader, 82).unwrap();
            assert_eq!(body.version, 1);
            assert_eq!(body.flag, 1);
            assert_eq!(body.guid, LABEL_ENGINE_PROPS_CLASS_ID);
            assert_eq!(body.value_18, 0);
            assert_eq!(reader.position(), 0x17);
            assert_eq!(reader.read_bytes(6).unwrap(), [2, 0, 0, 0, 0, 0]);
        }

        #[test]
        fn display_expression_props_prefix_stops_before_unclosed_tail() {
            // 样本形态:ver 1、String "[BSM]"、short -1、引擎槽(键 84)。
            let mut b = 1_i16.to_le_bytes().to_vec();
            b.extend(12_u32.to_le_bytes());
            for unit in "[BSM]".encode_utf16() {
                b.extend_from_slice(&unit.to_le_bytes());
            }
            b.extend_from_slice(&[0u8; 2]); // 终止零
            b.extend((-1_i16).to_le_bytes());
            push_entry_header(
                &mut b,
                &mxd2qgz::layers::annotate_props::ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID,
                84,
            );
            b.extend(2_i16.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            let prefix_len = b.len();
            b.extend(0u8.to_le_bytes()); // 未闭合尾段哨兵
            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let prefix =
                read_display_expression_props_prefix_10_1(&mut reader, 1024, &mut table, 83)
                    .unwrap();
            assert_eq!(prefix.key, 83);
            assert_eq!(prefix.version, 1);
            assert_eq!(prefix.string, "[BSM]");
            assert_eq!(prefix.short, -1);
            match &prefix.engine {
                ExpressionEngineSlot10_1::VbscriptEngine(engine) => {
                    assert_eq!(engine.key, 84);
                    assert_eq!(engine.short, Some(-1));
                }
                other => panic!("引擎槽分类不符：{other:?}"),
            }
            // 游标停在尾段起点(哨兵未消费)。
            assert_eq!(reader.remaining(), 1);
            assert!(table.contains(84));
            let _ = prefix_len;
        }

        #[test]
        fn lelp_slots_dispatch_bolp_engine_envelope() {
            // LELP v7(样本形态):⑯ TextSymbol 槽 = 空标记(简化)、⑰ BOLP 新
            // 对象、⑱ 引擎新对象、⑲ Envelope 空标记 + ⑳-㉖ 标量(样本值)。
            let mut b = 7_i16.to_le_bytes().to_vec();
            b.extend(string_bytes("[JZDH]"));
            b.extend((-1_i16).to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(string_bytes("默认"));
            b.extend(string_bytes(""));
            push_null_marker(&mut b); // ⑯ TextSymbol 槽
            // ⑰ BOLP 新对象(键 69,样本形态)。
            push_entry_header(&mut b, &BASIC_OVERPOSTER_LAYER_PROPS_CLASS_ID, 69);
            b.extend(bolp_body_sample());
            // ⑱ 引擎新对象(键 73)。
            push_entry_header(
                &mut b,
                &mxd2qgz::layers::annotate_props::ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID,
                73,
            );
            b.extend(2_i16.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            push_null_marker(&mut b); // ⑲ Envelope 槽
            b.extend(0.0_f64.to_le_bytes()); // ⑳
            b.extend(0.0_f64.to_le_bytes()); // ㉑
            b.extend(0_i32.to_le_bytes()); // ㉒
            b.extend(1_i32.to_le_bytes()); // ㉓
            push_null_marker(&mut b); // ㉔ 容器槽
            b.extend((-1_i16).to_le_bytes()); // ㉕
            b.extend(2_i32.to_le_bytes()); // ㉖

            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let body =
                read_label_engine_props_body_10_1(&mut reader, 1024, &mut table, 60).unwrap();
            assert_eq!(body.stop, None);
            assert!(matches!(body.text_symbol, LelpTextSymbolSlot10_1::Null));
            match &body.overposter {
                BolpSlot10_1::Bolp(bolp) => {
                    assert_eq!(bolp.key, 69);
                    assert_eq!(bolp.stop, None);
                }
                other => panic!("⑰ 槽分类不符：{other:?}"),
            }
            match &body.engine {
                ExpressionEngineSlot10_1::VbscriptEngine(engine) => {
                    assert_eq!(engine.key, 73);
                }
                other => panic!("⑱ 槽分类不符：{other:?}"),
            }
            assert!(matches!(body.envelope, EnvelopeSlot10_1::Null));
            assert_eq!(body.scale_ratio, Some(0.0));
            assert_eq!(body.int32_2c, Some(1));
            assert_eq!(body.short_46, Some(-1));
            assert_eq!(body.annotation_class_id, Some(2));
            assert_eq!(reader.remaining(), 0);
            assert!(table.contains(69));
            assert!(table.contains(70));
            assert!(table.contains(71));
            assert!(table.contains(72));
            assert!(table.contains(73));
        }

        #[test]
        fn lelp_bolp_inner_stop_propagates() {
            // ⑰ 槽 BOLP 体内槽 1 为未知类:停止透传为 OverposterBody,游标停在
            // 内层停止点。
            let mut b = 7_i16.to_le_bytes().to_vec();
            b.extend(string_bytes("[JZDH]"));
            b.extend((-1_i16).to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(string_bytes("默认"));
            b.extend(string_bytes(""));
            push_null_marker(&mut b); // ⑯
            let bolp_offset = b.len();
            push_entry_header(&mut b, &BASIC_OVERPOSTER_LAYER_PROPS_CLASS_ID, 69);
            // BOLP 前缀 20B + 槽 1 未知类。
            b.extend(0_i16.to_le_bytes());
            for _ in 0..3 {
                b.extend(0_i16.to_le_bytes());
            }
            b.extend(0_i32.to_le_bytes());
            b.extend(3_i32.to_le_bytes());
            b.extend(1_i32.to_le_bytes());
            let unknown = [0x66_u8; 16];
            let inner_stop_offset = b.len();
            push_entry_header(&mut b, &unknown, 70);
            b.extend(0u8.to_le_bytes()); // 哨兵

            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let body =
                read_label_engine_props_body_10_1(&mut reader, 1024, &mut table, 60).unwrap();
            match body.stop {
                Some(LelpStop10_1::OverposterBody(
                    BolpStop10_1::LabelPositionSlotUnsupported { class_id, key },
                )) => {
                    assert_eq!(class_id, unknown);
                    assert_eq!(key, 70);
                }
                other => panic!("停止原因不符：{other:?}"),
            }
            // 游标停在内层停止点(未知对象头起点;哨兵与该头均未消费)。
            assert_eq!(reader.remaining(), b.len() - inner_stop_offset);
            assert_eq!(bolp_offset + 20 + 20, inner_stop_offset);
            assert!(!table.contains(70));
            // BOLP 键已登记(登记先于持久化)。
            assert!(table.contains(69));
        }

        #[test]
        fn extension_section_and_chart_symbol_leave_the_rest_of_the_tail_aligned() {
            // 样本形态扩展段(层 32):计数 1、尺寸 387(仅体)、CLSID、ETM 体
            // (ver 2、count 1、ET 条目、尾字节 0x01)。
            let mut b = successor_prefix();
            // ALPC:空集合(计数 0)→ 快速推进到扩展段之前的字段。
            push_entry_header(&mut b, &ANNOTATE_PROPS_COLLECTION_CLASS_ID, 59);
            b.extend(2_i16.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0_i16.to_le_bytes()); // short +0x1bc
            push_entry_header(&mut b, &FEATURE_ID_SET_CLASS_ID, 75);
            b.extend(1_i16.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0_i32.to_le_bytes()); // 选择集计数
            b.extend(2_i16.to_le_bytes()); // short +0x1fe
            push_null_marker(&mut b); // IDisplayFilter
            for _ in 0..3 {
                b.extend(0_i16.to_le_bytes());
            }
            b.extend(0_i32.to_le_bytes());
            push_null_marker(&mut b); // IName
            b.extend(0_i32.to_le_bytes()); // 关系类计数
            b.extend(string_bytes("")); // String +0x18c
            push_null_marker(&mut b); // Envelope 槽 1
            b.extend(0_i32.to_le_bytes()); // 范围计数
            // 扩展段(样本形态):尺寸 = 体字节数,条目 = [CLSID][体],无键。
            b.extend(1_i32.to_le_bytes());
            let etm_body = edit_template_manager_body(77);
            b.extend((etm_body.len() as i64).to_le_bytes());
            b.extend_from_slice(&EDIT_TEMPLATE_MANAGER_CLASS_ID);
            // ETM 体:样本尾字节为 0x01(修正第九轮 0x00 记录);此处按 0x01。
            let mut etm = 2_i16.to_le_bytes().to_vec();
            etm.extend(1_i32.to_le_bytes());
            push_entry_header(&mut etm, &EDIT_TEMPLATE_CLASS_ID, 77);
            etm.extend(edit_template_body());
            etm.push(0x01);
            assert_eq!(etm.len(), etm_body.len()); // 尺寸一致性(合成体同长)
            b.extend_from_slice(&etm);
            // ㉖ 统计图符号之后继续读取 ㉗–㊸。
            use mxd2qgz::objects::display_symbol::{PIE_CHART_SYMBOL_CLASS_ID, SymbolSlotFull10_1};
            b.extend(98.0_f64.to_le_bytes()); // ㉕ double
            push_entry_header(&mut b, &PIE_CHART_SYMBOL_CLASS_ID, 78);
            b.extend(1_u16.to_le_bytes());
            b.extend([0; 4 * 8 + 1 + 8 + 4 + 2]); // 饼图首段字段
            push_null_marker(&mut b); // ILineSymbol
            b.extend(2_u16.to_le_bytes());
            b.extend([0; 4 + 4 + 8 + 4]); // 公共标量段和空填充数组
            push_null_marker(&mut b); // IMarkerBackground
            b.extend((-1_i16).to_le_bytes());
            b.extend(1_i32.to_le_bytes());
            push_null_marker(&mut b); // Envelope 槽 2
            push_null_marker(&mut b); // IUID 槽
            b.extend(0_u32.to_le_bytes()); // String +0x28c:cb=0(BSTR 空,第十轮)
            b.extend(2_i16.to_le_bytes()); // short +0x1d8
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend((-1_i16).to_le_bytes());
            b.extend(0_i32.to_le_bytes()); // PreJoin 计数；后接 +0x2c0
            b.extend(0_i32.to_le_bytes());
            for _ in 0..3 {
                b.extend(0_i16.to_le_bytes());
            }
            for _ in 0..4 {
                b.extend(string_bytes(""));
            }
            push_null_marker(&mut b); // ㊴ 表达式属性槽
            b.extend(0_i16.to_le_bytes());
            for _ in 0..3 {
                b.extend(string_bytes(""));
            }
            // TimeData 族 + ㊷ + ㊸(同 full-path 测试)。
            b.extend(2_i16.to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            push_null_marker(&mut b);
            b.extend(0_i16.to_le_bytes());
            b.extend(1_i16.to_le_bytes());
            for _ in 0..4 {
                b.extend(string_bytes(""));
            }
            push_null_marker(&mut b);
            b.extend(1_i16.to_le_bytes());
            b.extend(string_bytes(""));
            b.extend(string_bytes(""));
            b.extend(1_i16.to_le_bytes());
            b.extend(0_i16.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            b.extend(0.0_f64.to_le_bytes());
            b.extend(0_i32.to_le_bytes());
            push_null_marker(&mut b); // ㊷ {E720618B} 槽
            b.extend(0_i16.to_le_bytes()); // ㊸

            let mut reader = Reader::new(&b);
            let mut table = RefTable10_1::new();
            let result =
                read_feature_layer_successor_10_1(&mut reader, 34, 4096, &mut table).unwrap();
            assert_eq!(result.stop, SuccessorStop10_1::TailCompleted);
            assert!(
                matches!(result.symbol, Some(SymbolSlotFull10_1::ChartSymbol(body)) if body.key == 78)
            );
            assert!(table.contains(78));
            assert_eq!(reader.remaining(), 0);
            assert_eq!(result.extension_count, Some(1));
            let LayerExtension10_1::EditTemplateManager(etm) = &result.extensions[0] else {
                panic!("预期 EditTemplateManager");
            };
            assert_eq!(etm.key, 0); // 扩展段无键
            assert_eq!(etm.version, 2);
            assert_eq!(etm.tail_byte, Some(1)); // 样本 0x01
            // ㉚ String cb=0 已消费(其后字段可达 → TailCompleted 已证)。
            assert_eq!(result.string_28c, Some(String::new()));
            assert!(!table.contains(76)); // ETM 无键不登记
            assert!(table.contains(77)); // ET 条目键登记
        }
    }
}
