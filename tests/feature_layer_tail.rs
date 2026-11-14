//! `feature_layer_tail`（FeatureLayer 尾段链第一环）严格读取器的合成输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-featurelayer-tail.md`：五个类（SimpleRenderer
//! /SimpleMarkerSymbol/RgbColor/LegendGroup/LegendClass）的字段序来自 10.1
//! 原生 DLL 反编译，样本 `0x3ed..0x602` 对位为辅助观察。本文件全部使用合成
//! 输入，仅验证读取器自身契约（正常/异常/截断/事务性），不代表 MXD 格式已
//! 验证，未在目标软件中核对。

use mxd2qgz::layers::feature_layer_tail::{
    ColorSlot10_1, LEGEND_CLASS_CLASS_ID, LEGEND_GROUP_CLASS_ID, LegendClassEntry10_1,
    LegendClassStop10_1, LegendGroupSlot10_1, LegendGroupStop10_1, RGB_COLOR_CLASS_ID,
    SIMPLE_MARKER_SYMBOL_CLASS_ID, SIMPLE_RENDERER_CLASS_ID, SimpleMarkerSymbolStop10_1,
    SimpleRendererStop10_1, SymbolSlot10_1, read_legend_class_body_10_1,
    read_legend_group_body_10_1, read_rgb_color_body_10_1, read_simple_marker_symbol_body_10_1,
    read_simple_renderer_body_10_1,
};
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
    // 外层/内层版本 >1 与负值位模式（有符号比较）都拒绝。
    for version in [2_u16, 0x8000, 0xFFFF] {
        let mut bytes = Vec::new();
        bytes.extend(version.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.push(0);
        bytes.extend(0.0_f64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let position = reader.position();
        let mut table = table_with_keys(&[9]);
        assert!(
            read_rgb_color_body_10_1(&mut reader, &mut table, 5).is_err(),
            "版本 {version:#x} 应拒绝"
        );
        assert_eq!(reader.position(), position, "失败不得推进游标");
        assert!(table.contains(9), "失败不得修改登记表");
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
    let sms = read_simple_marker_symbol_body_10_1(&mut reader, &mut table, 36).unwrap();
    assert_eq!(sms.key, 36);
    assert_eq!(sms.version, 2);
    assert!(
        matches!(
            &sms.color,
            ColorSlot10_1::RgbColor(body) if body.key == 37 && body.doubles == Some([1.0, 2.0, 3.0])
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
            Some(ColorSlot10_1::RgbColor(body)) if body.key == 38
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
    let sms =
        read_simple_marker_symbol_body_10_1(&mut reader, &mut RefTable10_1::new(), 36).unwrap();
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
    let sms = read_simple_marker_symbol_body_10_1(&mut reader, &mut table, 36).unwrap();
    assert_eq!(
        sms.stop,
        Some(SimpleMarkerSymbolStop10_1::ColorUnsupported {
            class_id: LEGEND_CLASS_CLASS_ID,
            key: 44,
        })
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
    let sms =
        read_simple_marker_symbol_body_10_1(&mut reader, &mut RefTable10_1::new(), 36).unwrap();
    assert_eq!(
        sms.stop,
        Some(SimpleMarkerSymbolStop10_1::OutlineColorUnsupported {
            class_id: LEGEND_CLASS_CLASS_ID,
            key: 44,
        })
    );
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
            read_simple_marker_symbol_body_10_1(&mut reader, &mut table, 36).is_err(),
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
        SymbolSlot10_1::BackReference { key: 36, .. }
    ));
    assert_eq!(lc.label, "");
    assert_eq!(lc.description, "");
    assert_eq!(lc.format, ObjectEntry10_1::Null);
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
        ObjectEntry10_1::NewObject {
            class_id: RGB_COLOR_CLASS_ID,
            key: 45,
        },
        "条目分类保留，游标与登记表回退"
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
                SymbolSlot10_1::BackReference { key: 36, .. }
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
        matches!(&body.symbol, SymbolSlot10_1::SimpleMarkerSymbol(sms) if sms.key == 36),
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
    assert!(matches!(body.symbol, SymbolSlot10_1::Null), "未知符号回退");
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
