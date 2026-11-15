//! `legend_format` / `legend_item` / `edit_template`（Legend/EditTemplate
//! 尾段链三类）严格读取器的合成输入测试。
//!
//! 原生证据见 `docs/arcmap-10.1-legend-tails.md`：字段序来自 Layer.dll、
//! Renderers.dll、LayerLib.dll、FeatureLayer.dll 10.1.0.3035 静态反编译。
//! LegendClassFormat 与 HorizontalLegendItem 在仓库参考样本 /Maps 流中
//! 0 命中（无样本字节佐证），测试只用合成输入；EditTemplate 的
//! `sample_edit_template_body_matches_field_order` 用样本键 1605 实例的
//! 字节构造合成等价输入做辅助对位（辅助观察，不代表 MXD 格式已验证）。

use mxd2qgz::layers::edit_template::{
    ClassVariantArray10_1, EDIT_TEMPLATE_CLASS_ID, EditTemplateField10_1, VariantClassValue10_1,
    read_edit_template_body_10_1, read_variant_class_value_10_1,
};
use mxd2qgz::layers::legend_format::{
    LEGEND_CLASS_FORMAT_CLASS_ID, LegendClassFormatSlot10_1, LegendClassFormatStop10_1,
    read_legend_class_format_body_10_1,
};
use mxd2qgz::layers::legend_item::{
    HORIZONTAL_LEGEND_ITEM_CLASS_ID, HorizontalLegendItemStop10_1, LegendItemClassFormatSlot10_1,
    LegendItemSlot10_1, read_horizontal_legend_item_body_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::storage::object_stream::read_header_10_1;
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

/// 10.8 ReadString 布局（n*2+2）：4 字节码元数 + 编码 + 终止零。
fn read_string_layout_bytes(text: &str) -> Vec<u8> {
    let units: Vec<u16> = text.encode_utf16().collect();
    if units.is_empty() {
        // n == 0：无载荷（空串总长 4 字节）。
        return 0_u32.to_le_bytes().to_vec();
    }
    let mut bytes = (units.len() as u32).to_le_bytes().to_vec();
    for unit in units {
        bytes.extend(unit.to_le_bytes());
    }
    bytes.extend(0_u16.to_le_bytes());
    bytes
}

/// 追加对象条目头（16 字节类标识 + 4 字节键）。
fn push_entry_header(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 追加 16 字节空标记。
fn push_null_marker(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&[0_u8; 16]);
}

/// 预登记键的登记表（模拟「宿主对象先于嵌套对象持久化」的会话状态）。
fn table_with_keys(keys: &[u32]) -> RefTable10_1 {
    let mut table = RefTable10_1::new();
    for key in keys {
        table.register(*key);
    }
    table
}

/// 从完整对象（头 + 体）字节读对象头并断言类标识/键，返回（体读取器
/// 输入 Reader、键）。调用后 Reader 位于对象头之后。
fn open_object<'a>(bytes: &'a [u8], class_id: &[u8; 16]) -> (Reader<'a>, u32) {
    let mut reader = Reader::new(bytes);
    match read_header_10_1(&mut reader).expect("对象头应合法") {
        mxd2qgz::storage::object_stream::ObjectHeader::Object {
            class_id: found,
            key,
        } => {
            assert_eq!(found, *class_id, "类标识应一致");
            (reader, key)
        }
        other => panic!("应读到对象头，得到 {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// LegendClassFormat
// ---------------------------------------------------------------------------

/// 构造 LegendClassFormat 体：版本 + 4 槽（各槽形态由 form 给出）+ 版本 >1
/// 的 double ×2。
fn legend_class_format_bytes(version: u16, stop_at_slot: Option<usize>) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    for index in 0..4usize {
        if Some(index) == stop_at_slot {
            // 触发停止的新对象：用一个非本任务闭合类的 CLSID（SimpleMarker
            // Symbol 族候选占位）。
            push_entry_header(&mut bytes, &[1_u8; 16], 0x1234 + index as u32);
            return bytes;
        }
        push_null_marker(&mut bytes);
    }
    if version > 1 {
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.extend((-2.5_f64).to_le_bytes());
    }
    bytes
}

#[test]
fn legend_class_format_reads_full_body_version_2() {
    let mut body = legend_class_format_bytes(2, None);
    body.extend_from_slice(b"tail");
    let bytes = with_header(&LEGEND_CLASS_FORMAT_CLASS_ID, 9, &body);
    let (mut reader, key) = open_object(&bytes, &LEGEND_CLASS_FORMAT_CLASS_ID);
    let mut table = RefTable10_1::new();
    let body =
        read_legend_class_format_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(body.key, key);
    assert_eq!(body.version, 2);
    assert_eq!(
        body.label_symbol,
        LegendClassFormatSlot10_1::Null,
        "四个槽均为空标记"
    );
    assert_eq!(body.line_patch, LegendClassFormatSlot10_1::Null);
    assert_eq!(body.patch_doubles, Some([1.5, -2.5]));
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 4);
    assert!(
        !table.contains(key),
        "体键由调用者经分类登记，读取器不重复登记"
    );
}

#[test]
fn legend_class_format_version_1_has_no_doubles() {
    let bytes = with_header(
        &LEGEND_CLASS_FORMAT_CLASS_ID,
        9,
        &legend_class_format_bytes(1, None),
    );
    let (mut reader, key) = open_object(&bytes, &LEGEND_CLASS_FORMAT_CLASS_ID);
    let mut table = RefTable10_1::new();
    let body =
        read_legend_class_format_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.patch_doubles, None, "版本 1 不读 double 组");
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 0);
}

#[test]
fn legend_class_format_rejects_version_out_of_range() {
    for version in [3_u16, 4, 0x8000] {
        let bytes = with_header(
            &LEGEND_CLASS_FORMAT_CLASS_ID,
            9,
            &legend_class_format_bytes(version, None),
        );
        let (mut reader, key) = open_object(&bytes, &LEGEND_CLASS_FORMAT_CLASS_ID);
        let mut table = RefTable10_1::new();
        let err = read_legend_class_format_body_10_1(&mut reader, usize::MAX, &mut table, key)
            .expect_err("版本超限应拒绝");
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 20, "失败后游标必须回到体入口");
        assert!(!table.contains(key), "失败不得登记键");
    }
}

#[test]
fn legend_class_format_back_references_are_consumed() {
    let mut table = table_with_keys(&[7, 8, 9, 10]);
    let mut body = 2_u16.to_le_bytes().to_vec();
    for key in [7_u32, 8, 9, 10] {
        push_entry_header(&mut body, &LEGEND_CLASS_FORMAT_CLASS_ID, key);
    }
    body.extend(0.0_f64.to_le_bytes());
    body.extend(0.0_f64.to_le_bytes());
    let bytes = with_header(&LEGEND_CLASS_FORMAT_CLASS_ID, 5, &body);
    let (mut reader, key) = open_object(&bytes, &LEGEND_CLASS_FORMAT_CLASS_ID);
    let body =
        read_legend_class_format_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert!(matches!(
        body.label_symbol,
        LegendClassFormatSlot10_1::BackReference { key: 7, .. }
    ));
    assert!(matches!(
        body.area_patch,
        LegendClassFormatSlot10_1::BackReference { key: 10, .. }
    ));
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 0);
}

#[test]
fn legend_class_format_stops_at_unsupported_slot_and_rolls_back() {
    let bytes = with_header(
        &LEGEND_CLASS_FORMAT_CLASS_ID,
        9,
        &legend_class_format_bytes(2, Some(2)),
    );
    let (mut reader, key) = open_object(&bytes, &LEGEND_CLASS_FORMAT_CLASS_ID);
    let mut table = table_with_keys(&[0x1234]);
    let body =
        read_legend_class_format_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(
        body.stop,
        Some(LegendClassFormatStop10_1::SlotUnsupported {
            index: 2,
            class_id: [1; 16],
            key: 0x1236
        })
    );
    assert_eq!(
        body.line_patch,
        LegendClassFormatSlot10_1::Unsupported {
            class_id: [1; 16],
            key: 0x1236
        }
    );
    assert_eq!(body.patch_doubles, None, "停止点之后的字段不可达");
    assert!(table.contains(0x1234), "停止前已见键保留");
    assert!(!table.contains(0x1236), "回退停止不登记该键");
    assert_eq!(
        body.remaining_bytes, 20,
        "回退到停止槽起点，槽头 20 字节保留为剩余"
    );
}

#[test]
fn legend_class_format_truncation_is_atomic() {
    let bytes = with_header(
        &LEGEND_CLASS_FORMAT_CLASS_ID,
        9,
        &legend_class_format_bytes(2, None),
    );
    for cut in 20..bytes.len() {
        let (mut reader, key) = open_object(&bytes[..cut], &LEGEND_CLASS_FORMAT_CLASS_ID);
        let mut table = RefTable10_1::new();
        let err = read_legend_class_format_body_10_1(&mut reader, usize::MAX, &mut table, key);
        if let Ok(body) = err {
            assert!(body.stop.is_some(), "截断 {cut}：只允许经停止点提前返回");
            continue;
        }
        assert_eq!(
            reader.position(),
            20,
            "截断 {cut}：失败后游标必须回到体入口"
        );
        assert!(!table.contains(key), "截断 {cut}：失败不得登记键");
        // 同一 Reader 重读得到相同错误类别（clone-trial 事务性）。
        let mut rereader = Reader::new(&bytes[..cut]);
        let _ = read_header_10_1(&mut rereader).unwrap();
        let second = read_legend_class_format_body_10_1(&mut rereader, usize::MAX, &mut table, key);
        assert_eq!(
            second.err().map(|e| e.kind()),
            err.err().map(|e| e.kind()),
            "截断 {cut}：重读错误类别应一致"
        );
    }
}

// ---------------------------------------------------------------------------
// HorizontalLegendItem（BaseLegendItem 体）
// ---------------------------------------------------------------------------

/// 构造 HorizontalLegendItem 体：条目版本 + int32 + 基类版本 + 各段。
///
/// `base_tail` 控制 ≥2 组；`number_format` 决定 >2 组的对象槽形态。
fn legend_item_bytes(
    version: u16,
    base_version: u16,
    class_format_new_object: bool,
    include_tail: bool,
) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(0x5A5A_A5A5_u32.to_le_bytes()); // int32 原样
    bytes.extend(base_version.to_le_bytes());
    // ILayer 槽：回引（键 100 已预登记）。
    push_entry_header(&mut bytes, &LEGEND_CLASS_FORMAT_CLASS_ID, 100);
    for value in [0x1111_u16, 0x2222, 0x3333, 0x4444] {
        bytes.extend(value.to_le_bytes());
    }
    // ITextSymbol 槽 ×2：空标记。
    push_null_marker(&mut bytes);
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend(0x00FF_u16.to_le_bytes());
    push_null_marker(&mut bytes);
    // ILegendClassFormat 槽：新对象（LegendClassFormat 类）或回引。
    if class_format_new_object {
        let inner = legend_class_format_bytes(2, None);
        push_entry_header(&mut bytes, &LEGEND_CLASS_FORMAT_CLASS_ID, 55);
        bytes.extend(&inner);
    } else {
        push_entry_header(&mut bytes, &LEGEND_CLASS_FORMAT_CLASS_ID, 56);
    }
    bytes.extend(0xABCD_u16.to_le_bytes());
    bytes.extend(0xEF01_u16.to_le_bytes());
    if include_tail {
        bytes.extend(0x0102_u16.to_le_bytes());
        bytes.extend(0x0304_u16.to_le_bytes());
        bytes.extend(string_bytes(" ("));
        bytes.extend(string_bytes(")"));
        bytes.extend(0xFFFF_u16.to_le_bytes());
    }
    if base_version > 2 {
        // INumberFormat 槽：空标记。
        push_null_marker(&mut bytes);
    }
    bytes
}

#[test]
fn legend_item_reads_full_body_with_nested_class_format() {
    let mut body = legend_item_bytes(1, 3, true, true);
    body.extend_from_slice(b"tail!");
    let bytes = with_header(&HORIZONTAL_LEGEND_ITEM_CLASS_ID, 77, &body);
    let (mut reader, key) = open_object(&bytes, &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
    let mut table = table_with_keys(&[100]);
    let body =
        read_horizontal_legend_item_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.raw_int32, 0x5A5A_A5A5_u32 as i32);
    assert_eq!(body.base_version, 3);
    assert!(matches!(
        body.layer,
        LegendItemSlot10_1::BackReference { key: 100, .. }
    ));
    assert_eq!(body.shorts_a, [0x1111, 0x2222, 0x3333, 0x4444]);
    assert_eq!(body.raw_int32_b, 7);
    assert_eq!(body.short_b, 0x00FF);
    match &body.class_format {
        LegendItemClassFormatSlot10_1::ClassFormat(inner) => {
            assert_eq!(inner.key, 55);
            assert_eq!(inner.version, 2);
            assert_eq!(inner.patch_doubles, Some([1.5, -2.5]));
            assert_eq!(inner.stop, None);
        }
        other => panic!("应为内嵌 LegendClassFormat 体，得到 {other:?}"),
    }
    assert_eq!(body.shorts_b, [0xABCD, 0xEF01]);
    let tail = body.tail_group.expect("基类版本 3 应读 ≥2 组");
    assert_eq!(
        (tail.short_a, tail.short_b, tail.short_c),
        (0x0102, 0x0304, 0xFFFF)
    );
    assert_eq!(tail.prefix, " (");
    assert_eq!(tail.suffix, ")");
    assert_eq!(body.number_format, Some(LegendItemSlot10_1::Null));
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 5);
    assert!(table.contains(55), "嵌套 LegendClassFormat 键应经分类登记");
    assert!(
        !table.contains(key),
        "本体键由调用者经分类登记，读取器不重复登记"
    );
}

#[test]
fn legend_item_base_version_below_two_writes_defaults_not_reads() {
    // 基类版本 1：≥2 组与 >2 组原生不读流。
    let mut body = legend_item_bytes(1, 1, false, false);
    body.extend_from_slice(b"rest");
    let bytes = with_header(&HORIZONTAL_LEGEND_ITEM_CLASS_ID, 77, &body);
    let (mut reader, key) = open_object(&bytes, &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
    let mut table = table_with_keys(&[100, 56]);
    let body =
        read_horizontal_legend_item_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(body.base_version, 1);
    assert_eq!(body.tail_group, None, "基类版本 <2 不读 ≥2 组");
    assert_eq!(body.number_format, None, "基类版本 ≤2 无 INumberFormat 槽");
    assert!(matches!(
        body.class_format,
        LegendItemClassFormatSlot10_1::BackReference { key: 56, .. }
    ));
    assert_eq!(body.stop, None);
    assert_eq!(body.remaining_bytes, 4);
}

#[test]
fn legend_item_rejects_versions_out_of_range() {
    for &(item_version, base_version) in &[(2_u16, 1_u16), (0x8000, 1), (1, 4), (1, 0x8004)] {
        let bytes = with_header(
            &HORIZONTAL_LEGEND_ITEM_CLASS_ID,
            77,
            &legend_item_bytes(item_version, base_version, false, false),
        );
        let (mut reader, key) = open_object(&bytes, &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
        let mut table = table_with_keys(&[100, 56]);
        let err = read_horizontal_legend_item_body_10_1(&mut reader, usize::MAX, &mut table, key)
            .expect_err("版本超限应拒绝");
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 20, "失败后游标必须回到体入口");
        assert!(!table.contains(key));
    }
}

#[test]
fn legend_item_stops_at_unsupported_layer_slot() {
    let mut body = 1_u16.to_le_bytes().to_vec();
    body.extend(0_u32.to_le_bytes());
    body.extend(3_u16.to_le_bytes());
    // ILayer 槽：非闭合类新对象。
    push_entry_header(&mut body, &[2_u8; 16], 0x777);
    body.extend_from_slice(b"unreachable");
    let bytes = with_header(&HORIZONTAL_LEGEND_ITEM_CLASS_ID, 77, &body);
    let (mut reader, key) = open_object(&bytes, &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
    let mut table = RefTable10_1::new();
    let body =
        read_horizontal_legend_item_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(
        body.stop,
        Some(HorizontalLegendItemStop10_1::SlotUnsupported {
            slot: "layer(ILayer)",
            class_id: [2; 16],
            key: 0x777
        })
    );
    assert!(matches!(
        body.layer,
        LegendItemSlot10_1::Unsupported { key: 0x777, .. }
    ));
    assert_eq!(body.shorts_a, [0; 4], "停止点之后的字段不可达");
    assert!(!table.contains(0x777));
    assert_eq!(
        body.remaining_bytes,
        bytes.len() - 28,
        "版本+int32+基类版本之后的槽位头与剩余保留（回退到槽位起点）"
    );
}

#[test]
fn legend_item_class_format_unsupported_class_rolls_back() {
    let bytes = with_header(
        &HORIZONTAL_LEGEND_ITEM_CLASS_ID,
        77,
        &legend_item_bytes(1, 3, false, true),
    );
    // 把 ILegendClassFormat 槽替换为非 LegendClassFormat 新对象。
    let header = 20usize;
    let slot_offset = header
        + 2
        + 4
        + 2
        + 20 // ILayer 回引
        + 8 // int16 ×4
        + 16 // ITextSymbol 空标记
        + 4 // int32
        + 2 // int16
        + 16; // ITextSymbol 空标记
    let mut replaced = bytes[..slot_offset].to_vec();
    push_entry_header(&mut replaced, &[3_u8; 16], 0x999);
    replaced.extend_from_slice(&bytes[slot_offset + 20..]);
    let (mut reader, key) = open_object(&replaced, &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
    let mut table = table_with_keys(&[100]);
    let body =
        read_horizontal_legend_item_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(
        body.stop,
        Some(HorizontalLegendItemStop10_1::ClassFormatUnsupported {
            class_id: [3; 16],
            key: 0x999
        })
    );
    assert_eq!(
        body.class_format,
        LegendItemClassFormatSlot10_1::Unsupported {
            class_id: [3; 16],
            key: 0x999
        }
    );
    assert_eq!(body.shorts_b, [0, 0], "停止点之后不可达");
    assert!(!table.contains(0x999));
}

#[test]
fn legend_item_truncation_is_atomic() {
    let bytes = with_header(
        &HORIZONTAL_LEGEND_ITEM_CLASS_ID,
        77,
        &legend_item_bytes(1, 3, true, true),
    );
    for cut in 20..bytes.len() {
        let (mut reader, key) = open_object(&bytes[..cut], &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
        let mut table = table_with_keys(&[100]);
        let result =
            read_horizontal_legend_item_body_10_1(&mut reader, usize::MAX, &mut table, key);
        if let Ok(body) = result {
            assert!(body.stop.is_some(), "截断 {cut}：只允许经停止点提前返回");
            continue;
        }
        assert_eq!(
            reader.position(),
            20,
            "截断 {cut}：失败后游标必须回到体入口"
        );
        assert!(!table.contains(key), "截断 {cut}：失败不得登记键");
    }
}

#[test]
fn legend_item_count_limit_and_overlong_limits() {
    // 嵌套 LegendClassFormat 体合法；但 max_bytes=0 应在首个字符串处拒绝
    // （≥2 组的 " (" 声明长度超限）。
    let bytes = with_header(
        &HORIZONTAL_LEGEND_ITEM_CLASS_ID,
        77,
        &legend_item_bytes(1, 3, true, true),
    );
    let (mut reader, key) = open_object(&bytes, &HORIZONTAL_LEGEND_ITEM_CLASS_ID);
    let mut table = table_with_keys(&[100]);
    let err = read_horizontal_legend_item_body_10_1(&mut reader, 4, &mut table, key)
        .expect_err("限额应拒绝 ≥2 组字符串");
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 20);
}

// ---------------------------------------------------------------------------
// Variant::Read（AfCore 类方法序列化器）
// ---------------------------------------------------------------------------

#[test]
fn variant_reads_all_fixed_width_types() {
    let mut bytes = Vec::new();
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend((-123_i16).to_le_bytes());
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend((-456_789_i32).to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(1.5_f32.to_le_bytes());
    bytes.extend(5_u16.to_le_bytes());
    bytes.extend(2.5_f64.to_le_bytes());
    bytes.extend(6_u16.to_le_bytes());
    bytes.extend(12_345_i64.to_le_bytes());
    bytes.extend(7_u16.to_le_bytes());
    bytes.extend(41986.0_f64.to_le_bytes());
    bytes.extend(10_u16.to_le_bytes());
    bytes.extend(0x8007_0005_u32.to_le_bytes());
    bytes.extend(11_u16.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.push(0x80);
    bytes.extend(17_u16.to_le_bytes());
    bytes.push(0x7F);
    bytes.extend(18_u16.to_le_bytes());
    bytes.extend(65_000_u16.to_le_bytes());
    bytes.extend(19_u16.to_le_bytes());
    bytes.extend(4_000_000_000_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes()); // EMPTY 无载荷
    bytes.extend(1_u16.to_le_bytes()); // NULL 无载荷
    bytes.extend(8_u16.to_le_bytes());
    bytes.extend(string_bytes("值Ⓐ"));

    let mut reader = Reader::new(&bytes);
    let expectations: [(&str, VariantClassValue10_1); 15] = [
        ("I2", VariantClassValue10_1::I2(-123)),
        ("I4", VariantClassValue10_1::I4(-456_789)),
        ("R4", VariantClassValue10_1::R4(1.5)),
        ("R8", VariantClassValue10_1::R8(2.5)),
        ("CY", VariantClassValue10_1::Cy(12_345)),
        ("DATE", VariantClassValue10_1::Date(41986.0)),
        ("ERROR", VariantClassValue10_1::Error(0x8007_0005)),
        ("BOOL", VariantClassValue10_1::Bool(0xFFFF)),
        ("I1", VariantClassValue10_1::I1(-128)),
        ("UI1", VariantClassValue10_1::Ui1(0x7F)),
        ("UI2", VariantClassValue10_1::Ui2(65_000)),
        ("UI4", VariantClassValue10_1::Ui4(4_000_000_000)),
        ("EMPTY", VariantClassValue10_1::Empty),
        ("NULL", VariantClassValue10_1::Null),
        ("BSTR", VariantClassValue10_1::Bstr("值Ⓐ".to_string())),
    ];
    for (label, expected) in expectations {
        let value = read_variant_class_value_10_1(&mut reader, usize::MAX).unwrap();
        assert_eq!(value, expected, "{label} 位模式应一致");
    }
    assert_eq!(reader.position(), bytes.len(), "全部值应恰好消费");
}

#[test]
fn variant_reads_arrays() {
    let mut bytes = Vec::new();
    // VT_I4 | VT_ARRAY：计数 + 整块 8 字节。
    bytes.extend((3_u16 | 0x2000).to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend((-2_i32).to_le_bytes());
    // VT_BSTR | VT_ARRAY：计数 + 每元素 ReadString（n*2+2）。
    bytes.extend((8_u16 | 0x2000).to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(read_string_layout_bytes("甲"));
    bytes.extend(read_string_layout_bytes(""));
    let mut reader = Reader::new(&bytes);
    let first = read_variant_class_value_10_1(&mut reader, usize::MAX).unwrap();
    match first {
        VariantClassValue10_1::Array(ClassVariantArray10_1::Fixed {
            vartype,
            element_bytes,
            raw,
        }) => {
            assert_eq!(vartype, 3);
            assert_eq!(element_bytes, 4);
            let mut expected = 1_i32.to_le_bytes().to_vec();
            expected.extend((-2_i32).to_le_bytes());
            assert_eq!(raw, expected);
        }
        other => panic!("应为定宽数组，得到 {other:?}"),
    }
    let second = read_variant_class_value_10_1(&mut reader, usize::MAX).unwrap();
    match second {
        VariantClassValue10_1::Array(ClassVariantArray10_1::Strings(items)) => {
            assert_eq!(items, vec!["甲".to_string(), String::new()]);
        }
        other => panic!("应为字符串数组，得到 {other:?}"),
    }
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn variant_rejects_object_branch_and_unknown_vartype() {
    for (vt, kind) in [
        (9_u16, ErrorKind::Unsupported),
        (13, ErrorKind::Unsupported),
        (12, ErrorKind::InvalidData), // VT_VARIANT 未实现分支
        (20, ErrorKind::InvalidData),
        (0x2000_u16, ErrorKind::Unsupported), // EMPTY|ARRAY 原生改道未定
        (0x2014, ErrorKind::InvalidData),     // 未实现类型|ARRAY
    ] {
        let mut bytes = vt.to_le_bytes().to_vec();
        bytes.extend([0_u8; 24]); // 余量
        let mut reader = Reader::new(&bytes);
        let err = read_variant_class_value_10_1(&mut reader, usize::MAX)
            .expect_err("对象分支/未知 VARTYPE 应拒绝");
        assert_eq!(err.kind(), kind, "vt 0x{vt:04x} 错误类别");
        assert_eq!(reader.position(), 0, "失败后游标必须回到入口");
    }
}

#[test]
fn variant_truncation_and_limits() {
    let mut bytes = 8_u16.to_le_bytes().to_vec();
    bytes.extend(string_bytes("中文字符串"));
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        if read_variant_class_value_10_1(&mut reader, usize::MAX).is_err() {
            assert_eq!(reader.position(), 0, "截断 {cut}：失败后游标必须回到入口");
        } else {
            assert_eq!(cut, bytes.len(), "截断 {cut}：不应成功");
        }
    }
    // 超大声明长度在限额处拒绝且不分配。
    let mut huge = 8_u16.to_le_bytes().to_vec();
    huge.extend(u32::MAX.to_le_bytes());
    let mut reader = Reader::new(&huge);
    let err = read_variant_class_value_10_1(&mut reader, 1024).expect_err("限额应拒绝");
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

// ---------------------------------------------------------------------------
// EditTemplate（含样本对位）
// ---------------------------------------------------------------------------

/// 构造 EditTemplate 体字节。
fn edit_template_bytes(version: u16, fields: &[(&str, VariantClassValue10_1)]) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(string_bytes("JZD1"));
    bytes.extend(string_bytes(""));
    bytes.extend(0x2A8B3331_u32.to_le_bytes());
    bytes.extend(0x5238_u16.to_le_bytes());
    bytes.extend(0x4025_u16.to_le_bytes());
    bytes.extend_from_slice(&[0x97, 0x2e, 0x45, 0x2a, 0x69, 0x53, 0x5b, 0x06]);
    bytes.extend((fields.len() as u32).to_le_bytes());
    for (key, value) in fields {
        bytes.extend(string_bytes(key));
        match value {
            VariantClassValue10_1::Bstr(text) => {
                bytes.extend(8_u16.to_le_bytes());
                bytes.extend(string_bytes(text));
            }
            VariantClassValue10_1::I4(n) => {
                bytes.extend(3_u16.to_le_bytes());
                bytes.extend(n.to_le_bytes());
            }
            VariantClassValue10_1::R8(x) => {
                bytes.extend(5_u16.to_le_bytes());
                bytes.extend(x.to_le_bytes());
            }
            _ => panic!("测试构造仅覆盖 BSTR/I4/R8"),
        }
    }
    if version >= 2 {
        bytes.extend(string_bytes("点"));
    }
    bytes
}

#[test]
fn edit_template_reads_full_body_version_2() {
    let fields = [
        ("JZDH", VariantClassValue10_1::Bstr(String::new())),
        ("OBJECTID", VariantClassValue10_1::I4(0)),
        ("ZZBZ", VariantClassValue10_1::R8(0.0)),
    ];
    let mut body = edit_template_bytes(2, &fields);
    body.extend_from_slice(b"next");
    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 76, &body);
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let body = read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(body.version, 2);
    assert_eq!(body.name, "JZD1");
    assert_eq!(body.string_b, "");
    assert_eq!(
        body.raw_guid,
        [
            0x31, 0x33, 0x8b, 0x2a, 0x38, 0x52, 0x25, 0x40, 0x97, 0x2e, 0x45, 0x2a, 0x69, 0x53,
            0x5b, 0x06
        ]
    );
    assert_eq!(body.field_count, 3);
    assert_eq!(body.fields.len(), 3);
    assert_eq!(
        body.fields[0],
        EditTemplateField10_1 {
            key: "JZDH".to_string(),
            value: VariantClassValue10_1::Bstr(String::new())
        }
    );
    assert_eq!(body.tail_string, Some("点".to_string()));
    assert_eq!(body.remaining_bytes, 4);
    assert!(
        !table.contains(key),
        "体键由调用者经分类登记，读取器不重复登记"
    );
}

#[test]
fn edit_template_version_below_two_has_no_tail_string() {
    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 76, &edit_template_bytes(1, &[]));
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let body = read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.field_count, 0);
    assert_eq!(body.tail_string, None, "版本 <2 原生不读第三 String");
    assert_eq!(body.remaining_bytes, 0);
}

#[test]
fn edit_template_rejects_version_and_negative_count() {
    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 76, &edit_template_bytes(3, &[]));
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let err = read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key)
        .expect_err("版本 >2 应拒绝");
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 20);

    // 负计数：构造 version 2 + 两串 + GUID + 负计数。
    let mut negative_body = 2_u16.to_le_bytes().to_vec();
    negative_body.extend(string_bytes(""));
    negative_body.extend(string_bytes(""));
    negative_body.extend([0_u8; 16]);
    negative_body.extend((-1_i32).to_le_bytes());
    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 76, &negative_body);
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let err = read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key)
        .expect_err("负计数应拒绝");
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 20);
    assert!(!table.contains(key));
}

#[test]
fn edit_template_rejects_variant_object_branch_atomically() {
    // 循环内出现 VT_DISPATCH（9）应整体失败，游标与登记表不动。
    let mut body = 2_u16.to_le_bytes().to_vec();
    body.extend(string_bytes("模板"));
    body.extend(string_bytes(""));
    body.extend([0_u8; 16]);
    body.extend(1_u32.to_le_bytes());
    body.extend(string_bytes("字段"));
    body.extend(9_u16.to_le_bytes()); // 对象分支
    body.extend([0_u8; 20]);
    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 76, &body);
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let err = read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key)
        .expect_err("Variant 对象分支应拒绝");
    assert_eq!(err.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), 20, "失败后游标必须回到体入口");
    assert!(!table.contains(key), "失败不得登记键");
}

#[test]
fn edit_template_truncation_is_atomic() {
    let fields = [
        ("A", VariantClassValue10_1::Bstr(String::new())),
        ("B", VariantClassValue10_1::I4(3)),
    ];
    let bytes = with_header(
        &EDIT_TEMPLATE_CLASS_ID,
        76,
        &edit_template_bytes(2, &fields),
    );
    for cut in 20..bytes.len() {
        let (mut reader, key) = open_object(&bytes[..cut], &EDIT_TEMPLATE_CLASS_ID);
        let mut table = RefTable10_1::new();
        if read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key).is_err() {
            assert_eq!(
                reader.position(),
                20,
                "截断 {cut}：失败后游标必须回到体入口"
            );
            assert!(!table.contains(key), "截断 {cut}：失败不得登记键");
        }
    }
}

#[test]
fn edit_template_count_limit_rejected_before_reads() {
    let mut body = 2_u16.to_le_bytes().to_vec();
    body.extend(string_bytes(""));
    body.extend(string_bytes(""));
    body.extend([0_u8; 16]);
    body.extend(u32::MAX.to_le_bytes()); // 声明计数
    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 76, &body);
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let err =
        read_edit_template_body_10_1(&mut reader, 1024, &mut table, key).expect_err("限额应拒绝");
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 20);
    assert!(!table.contains(key));
}

/// 样本对位（辅助观察）：仓库参考样本 /Maps 流键 1605 实例
/// （0x17f62 起的对象头 + 体）的合成等价字节逐字段吻合。
#[test]
fn sample_edit_template_body_matches_field_order() {
    // 样本体字节（去掉 20 字节对象头；来源见 docs/arcmap-10.1-legend-tails.md
    // 「样本对位」）：版本 2、「JZD1」、空串、GUID、计数 12、[键|值] ×12、「点」。
    let mut body = 2_u16.to_le_bytes().to_vec();
    body.extend(string_bytes("JZD1"));
    body.extend(string_bytes(""));
    body.extend(0x2A8B3331_u32.to_le_bytes());
    body.extend(0x5238_u16.to_le_bytes());
    body.extend(0x4025_u16.to_le_bytes());
    body.extend_from_slice(&[0x97, 0x2e, 0x45, 0x2a, 0x69, 0x53, 0x5b, 0x06]);
    body.extend(12_u32.to_le_bytes());
    for (key, value) in [
        ("JZDH", VariantClassValue10_1::Bstr(String::new())),
        ("YSDM", VariantClassValue10_1::Bstr(String::new())),
        ("ZDZHDM", VariantClassValue10_1::Bstr(String::new())),
        ("JBLX", VariantClassValue10_1::Bstr(String::new())),
        ("OBJECTID", VariantClassValue10_1::I4(0)),
        ("ZZBZ", VariantClassValue10_1::R8(0.0)),
        ("YZBZ", VariantClassValue10_1::R8(0.0)),
        ("BSM", VariantClassValue10_1::Bstr(String::new())),
        ("XZBZ", VariantClassValue10_1::R8(0.0)),
        ("JZDLX", VariantClassValue10_1::Bstr(String::new())),
        ("BDCDYBSM", VariantClassValue10_1::Bstr(String::new())),
        ("Ring", VariantClassValue10_1::Bstr(String::new())),
    ] {
        body.extend(string_bytes(key));
        match value {
            VariantClassValue10_1::Bstr(_) => body.extend(8_u16.to_le_bytes()),
            VariantClassValue10_1::I4(_) => body.extend(3_u16.to_le_bytes()),
            VariantClassValue10_1::R8(_) => body.extend(5_u16.to_le_bytes()),
            _ => unreachable!(),
        }
        match value {
            VariantClassValue10_1::Bstr(text) => body.extend(string_bytes(&text)),
            VariantClassValue10_1::I4(n) => body.extend(n.to_le_bytes()),
            VariantClassValue10_1::R8(x) => body.extend(x.to_le_bytes()),
            _ => unreachable!(),
        }
    }
    body.extend(string_bytes("点"));

    let bytes = with_header(&EDIT_TEMPLATE_CLASS_ID, 1605, &body);
    let (mut reader, key) = open_object(&bytes, &EDIT_TEMPLATE_CLASS_ID);
    let mut table = RefTable10_1::new();
    let parsed = read_edit_template_body_10_1(&mut reader, usize::MAX, &mut table, key).unwrap();
    assert_eq!(parsed.name, "JZD1");
    assert_eq!(parsed.field_count, 12);
    let keys: Vec<_> = parsed.fields.iter().map(|f| f.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "JZDH", "YSDM", "ZDZHDM", "JBLX", "OBJECTID", "ZZBZ", "YZBZ", "BSM", "XZBZ", "JZDLX",
            "BDCDYBSM", "Ring"
        ]
    );
    assert_eq!(parsed.fields[4].value, VariantClassValue10_1::I4(0));
    assert_eq!(parsed.fields[5].value, VariantClassValue10_1::R8(0.0));
    assert_eq!(parsed.tail_string, Some("点".to_string()));
    assert_eq!(parsed.remaining_bytes, 0, "样本体应恰好消费");
}

/// 组装完整对象字节：20 字节对象头（16 字节类标识 + 4 字节键）+ 体。
fn with_header(class_id: &[u8; 16], key: u32, body: &[u8]) -> Vec<u8> {
    let mut bytes = push_entry_header_bytes(class_id, key);
    bytes.extend_from_slice(body);
    bytes
}

/// 组装对象头字节（16 字节类标识 + 4 字节键）。
fn push_entry_header_bytes(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

#[test]
fn object_entry_classification_uses_registered_keys() {
    // 交叉验证：体读取器的停止/回引判定依赖 object_refs 的键分类。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(string_bytes(""));
    bytes.extend(string_bytes(""));
    bytes.extend([0_u8; 16]);
    // 槽内放一个已见键的回引（非本读取器场景，仅验证分类一致性）。
    let mut table = table_with_keys(&[42]);
    let entry = with_header(&EDIT_TEMPLATE_CLASS_ID, 42, &bytes);
    let mut reader = Reader::new(&entry);
    let classified =
        mxd2qgz::storage::object_refs::read_object_entry_10_1(&mut reader, &mut table).unwrap();
    assert_eq!(
        classified,
        ObjectEntry10_1::BackReference {
            class_id: EDIT_TEMPLATE_CLASS_ID,
            key: 42
        }
    );
}
