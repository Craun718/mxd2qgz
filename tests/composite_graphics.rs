//! `composite_graphics` 读取器的合成输入测试（证据 B 字段序）。
//!
//! 仓库参考样本 `samples/测试底图.mxd` 的 /Maps 导出中有一个
//! CompositeGraphicsLayer 对象（头 @0x16fba 键 1553，体 @0x16fce 起）；本文件
//! 不读取样本文件，用与样本逐段对齐的合成字节镜像该布局（值来自
//! docs/arcmap-10.1-map-object.md 第四轮与本次 hex 复核，属辅助观察，不是
//! 目标软件执行验证）。

use mxd2qgz::layers::composite_graphics::{
    COMPOSITE_GRAPHICS_LAYER_CLASS_ID, CompositeGraphicsLayerStop10_1, ConsumedEntry10_1,
    ElementEntry10_1, ElementGroup10_1, SubLayerEntry10_1, SubLayerTailSlot10_1,
    read_composite_graphics_layer_prefix_10_1,
};
use mxd2qgz::objects::element_collection::ElementCollectionStop10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// 样本 ElementCollection 类标识（第 7 步嵌套对象，{CE8F3972-E9BE-11D1-
/// A232-080009B6F22B}，内存序）。
const ELEMENT_COLLECTION_CLASS_ID: [u8; 16] = [
    0x72, 0x39, 0x8f, 0xce, 0xbe, 0xe9, 0xd1, 0x11, 0xa2, 0x32, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];
/// 样本 FeatureLayer 类标识（{E663A651-8AAD-11D0-BEC7-00805F7C4268}，内存序）。
const FEATURE_LAYER_CLASS_ID: [u8; 16] = [
    0x51, 0xa6, 0x63, 0xe6, 0xad, 0x8a, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c, 0x42, 0x68,
];
/// 预置进登记表的既有键：命中即回引。
const SEEN_KEY: u32 = 5;

/// 合成测试用的未知元素类标识（非任何闭合元素类）。
const UNKNOWN_CLASS_ID: [u8; 16] = [0xa5; 16];

/// AfCore String::Write 布局：u32 字节长度前缀 + 含终止零的 UTF-16LE。
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

/// 嵌套对象条目字节：16 字节类标识 + 4 字节键。
fn entry_bytes(class_id: [u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

/// 按证据 B 字段序构造对象体：版本、组计数+每组[元素计数+条目]、名称、
/// short ×3、double ×2、三个 int32、第 7 步嵌套对象字节、（版本 >1）double+
/// int32、子图层计数+每项[String+ILayer+尾段]、未知尾部。
fn cgl_body(
    version: u16,
    group_region: &[u8],
    name: &str,
    nested_1c: &[u8],
    sublayer_region: &[u8],
    tail: &[u8],
) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(group_region);
    bytes.extend(encoded(name));
    bytes.extend(
        [0xffff_u16, 0xffff, 0x0000]
            .iter()
            .flat_map(|v| v.to_le_bytes()),
    );
    bytes.extend([0.0_f64, 0.0].iter().flat_map(|v| v.to_le_bytes()));
    bytes.extend([3_u32, 1, 1].iter().flat_map(|v| v.to_le_bytes()));
    bytes.extend(nested_1c);
    if version > 1 {
        bytes.extend(0.5_f64.to_le_bytes());
        bytes.extend(9_u32.to_le_bytes());
    }
    bytes.extend(sublayer_region);
    bytes.extend(tail);
    bytes
}

/// 组区字节：组计数 + 每组 [元素计数 + 条目字节]。
fn group_region(groups: &[(u32, &[Vec<u8>])]) -> Vec<u8> {
    let mut bytes = (groups.len() as u32).to_le_bytes().to_vec();
    for (count, entries) in groups {
        bytes.extend(count.to_le_bytes());
        for entry in *entries {
            bytes.extend_from_slice(entry);
        }
    }
    bytes
}

/// 子图层区字节：计数 + 每项 [String + ILayer 条目 + 尾段 short×3+double×2+
/// int32×2 + 嵌套对象 +（版本 >1）double]。
fn sublayer_region(version: u16, items: &[SubLayerItem]) -> Vec<u8> {
    let mut bytes = (items.len() as u32).to_le_bytes().to_vec();
    for item in items {
        bytes.extend(encoded(item.name));
        bytes.extend(item.layer_entry.iter());
        bytes.extend([1_u16, 2, 3].iter().flat_map(|v| v.to_le_bytes()));
        bytes.extend([8.5_f64, -9.5].iter().flat_map(|v| v.to_le_bytes()));
        bytes.extend([4_u32, 5].iter().flat_map(|v| v.to_le_bytes()));
        bytes.extend(item.tail_nested_entry.iter());
        if version > 1 {
            bytes.extend((-1.75_f64).to_le_bytes());
        }
    }
    bytes
}

/// 合成的子图层条目部件。
#[derive(Clone)]
struct SubLayerItem {
    name: &'static str,
    layer_entry: Vec<u8>,
    tail_nested_entry: Vec<u8>,
}

#[test]
fn composite_graphics_layer_class_id_constant_matches_documented_guid() {
    // 证据：{9646BB83-9512-11D2-A2F6-080009B6F22B}（esri.clsid.ecfg 声明 +
    // FeatureLayer.dll RTTI .?AVCompositeGraphicsLayer@@，Load 0x6f6b2100）；
    // 参考样本 /Maps:0x16fba 对象头（键 1553）与本常量逐字节相同。
    assert_eq!(
        COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
        [
            0x83, 0xbb, 0x46, 0x96, 0x12, 0x95, 0xd2, 0x11, 0xa2, 0xf6, 0x08, 0x00, 0x09, 0xb6,
            0xf2, 0x2b,
        ]
    );
}

#[test]
fn dispatches_element_collection_nested_object_and_continues() {
    // 第 7 步对象头为 ElementCollection（{CE8F3972}）键 1554 且体可完整消
    // 费（版本 4 + 计数 0，样本 /Maps:0x17030 形态）：整体解析其体、保留键
    // 登记、继续第 8 步（样本推进到第 9 步子图层条目的 FeatureLayer 停止，
    // 见下方样本走查测试）。
    let group_region = {
        let mut bytes = 11_u32.to_le_bytes().to_vec();
        for _ in 0..11 {
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes
    };
    let mut ec = ELEMENT_COLLECTION_CLASS_ID.to_vec();
    ec.extend(1554_u32.to_le_bytes());
    // ElementCollection 体：版本 4 + 计数 0（样本 @0x17044..0x1704a）。
    ec.extend(4_u16.to_le_bytes());
    ec.extend(0_u32.to_le_bytes());
    let mut sublayers = 1_u32.to_le_bytes().to_vec();
    sublayers.extend(encoded("JZD1 注记"));
    let mut fl_header = FEATURE_LAYER_CLASS_ID.to_vec();
    fl_header.extend(1555_u32.to_le_bytes());
    sublayers.extend(&fl_header);
    let mut bytes = cgl_body(6, &group_region, "<默认>", &ec, &sublayers, b"");
    bytes.extend(b"opaque-feature-layer");
    let header_start = bytes.len() - fl_header.len() - b"opaque-feature-layer".len();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, 6);
    assert_eq!(parsed.group_count, 11);
    assert_eq!(
        parsed.element_groups,
        vec![
            ElementGroup10_1 {
                declared_count: 0,
                elements: Vec::new()
            };
            11
        ]
    );
    assert_eq!(parsed.name, "<默认>", "含中文的样本名");
    assert_eq!(parsed.shorts, [0xffff, 0xffff, 0x0000]);
    assert_eq!(parsed.doubles, [0.0, 0.0]);
    assert_eq!(
        (parsed.int32_4c, parsed.int32_50, parsed.int32_local),
        (3, 1, 1)
    );
    // 第 7 步分派：条目分类为 None（体入专用字段）、体完整消费、键登记。
    assert_eq!(parsed.nested_1c, None);
    let collection = parsed.nested_1c_collection.as_ref().expect("应含解析体");
    assert_eq!(collection.version, 4);
    assert_eq!(collection.count, 0);
    assert_eq!(collection.stop, None, "样本形态体完整消费");
    assert!(table.contains(1554), "新对象键保留登记");
    // 推进到第 8 步之后：版本 >1 的 double+int32 已读，随后子图层条目 1 的
    // ILayer（FeatureLayer 键 1555）为新对象——停止并回退到该头起点。
    assert_eq!(parsed.after_version1, Some((0.5, 9)));
    assert_eq!(parsed.sublayer_count, 1);
    assert!(parsed.sublayers.is_empty());
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::SubLayerNewObject {
            index: 0,
            name: "JZD1 注记".to_string(),
            class_id: FEATURE_LAYER_CLASS_ID,
            key: 1555
        }
    );
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start);
    assert!(!table.contains(1555));
}

#[test]
fn propagates_element_collection_body_stop_keeping_key() {
    // 第 7 步 ElementCollection 体内部停止（版本 4 + 计数 1 + 未闭合元素
    // 类的新对象头）：透传 NestedCollectionIncomplete，游标停在内层停止点
    // （元素对象头起点），嵌套键已登记；其后字段不可达。
    let mut ec = ELEMENT_COLLECTION_CLASS_ID.to_vec();
    ec.extend(1554_u32.to_le_bytes());
    ec.extend(4_u16.to_le_bytes());
    ec.extend(1_u32.to_le_bytes());
    let mut element = UNKNOWN_CLASS_ID.to_vec();
    element.extend(70_u32.to_le_bytes());
    ec.extend(&element);
    ec.extend(b"unreachable-body");
    let bytes = cgl_body(6, &group_region(&[]), "<默认>", &ec, b"", b"tail");
    // 版本 2 + 组区 4 + 名称 14 + short 6 + double 16 + int32 12 + 嵌套头
    // 20 + 体 6 = 80 = 元素对象头起点（内层停止点）。
    let stop_offset = 2 + 4 + encoded("<默认>").len() + 6 + 16 + 12 + 20 + 6;
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let collection = parsed
        .nested_1c_collection
        .as_ref()
        .expect("应携带到停止点为止的体");
    assert_eq!(collection.count, 1);
    let CompositeGraphicsLayerStop10_1::NestedCollectionIncomplete { key, reason } = &parsed.stop
    else {
        panic!("应为 NestedCollectionIncomplete，实际 {:?}", parsed.stop);
    };
    assert_eq!(*key, 1554);
    match reason {
        mxd2qgz::objects::element_collection::ElementCollectionStop10_1::ElementBodyNotClosed {
            index,
            class_id,
            key: element_key,
        } => {
            assert_eq!(*index, 0);
            assert_eq!(*class_id, UNKNOWN_CLASS_ID);
            assert_eq!(*element_key, 70);
        }
        other => panic!("内层应为元素体未闭合停止，实际 {other:?}"),
    }
    assert!(table.contains(1554), "嵌套键已登记（登记先于持久化）");
    assert!(!table.contains(70), "未消费的元素键不登记");
    assert_eq!(parsed.after_version1, None, "其后字段不可达");
    assert_eq!(reader.position(), stop_offset, "游标停在内层停止点");
    assert_eq!(parsed.remaining_bytes, bytes.len() - stop_offset);
}

#[test]
fn element_collection_reader_failure_is_transactional() {
    // 第 7 步 ElementCollection 体非法（版本 5 >4 原生拒绝）：整体报错，
    // 调用者游标与登记表不动（trial 事务性）。
    let mut ec = ELEMENT_COLLECTION_CLASS_ID.to_vec();
    ec.extend(1554_u32.to_le_bytes());
    ec.extend(5_u16.to_le_bytes());
    ec.extend(0_u32.to_le_bytes());
    let mut bytes = cgl_body(6, &group_region(&[]), "<默认>", &ec, b"", b"tail");
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    table.register(42);
    let mut reader = Reader::new(&bytes);
    assert!(read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).is_err());
    assert_eq!(reader.position(), 0, "失败路径不推进调用者游标");
    assert_eq!(table.len(), 1, "失败路径不修改登记表");
    assert!(table.contains(42));
    assert!(!table.contains(1554));
}

#[test]
fn version_boundaries_and_after_version_one_gate() {
    // 接受范围 0..=6（原生 ver < 7 有符号比较）；版本 >1 才读 double+int32。
    // 版本 1 无 after_version1 字段、条目尾段亦无尾随 double。
    let item = SubLayerItem {
        name: "JZD1 注记",
        layer_entry: vec![0_u8; 16],
        tail_nested_entry: vec![0_u8; 16],
    };
    for (version, after_version1) in [(0_u16, None), (1, None), (2, Some((0.5, 9_u32)))] {
        let bytes = cgl_body(
            version,
            &group_region(&[]),
            "<默认>",
            &[0_u8; 16],
            &sublayer_region(version, &[item.clone()]),
            b"tail",
        );
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap();
        assert_eq!(parsed.version, version, "version={version}");
        assert_eq!(parsed.after_version1, after_version1, "version={version}");
        assert_eq!(parsed.stop, CompositeGraphicsLayerStop10_1::TailNotExpanded);
        assert_eq!(
            parsed.sublayers[0].tail_double,
            if version > 1 { Some(-1.75) } else { None },
            "version={version}"
        );
        assert_eq!(parsed.remaining_bytes, 4, "version={version}");
    }
    // >6 拒绝：7 是首个拒绝值；负值位模式在无符号解释下同样拒绝（符号性未
    // 闭合，见模块文档）。
    for version in [7_u16, 100, 0x8000, 0xffff] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(0_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let first =
            read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version}");
        assert!(first.to_string().contains("版本"), "version={version}");
        assert_eq!(reader.position(), 0, "version={version}");
        let second =
            read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version}");
        assert_eq!(reader.position(), 0, "version={version}");
    }
}

#[test]
fn element_groups_classify_null_and_back_references_and_skip_negative_counts() {
    // 组内元素：空引用继续、回引（键预置）继续；组计数与元素计数为负数位
    // 模式时不迭代（原生按有符号比较）、计数原样保留。
    let mixed = group_region(&[
        (2, &[vec![0_u8; 16], entry_bytes([0xa5; 16], SEEN_KEY)]),
        (0, &[]),
    ]);
    let bytes = cgl_body(
        2,
        &mixed,
        "<默认>",
        &[0; 16],
        &sublayer_region(2, &[]),
        b"tail",
    );
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.group_count, 2);
    assert_eq!(
        parsed.element_groups,
        vec![
            ElementGroup10_1 {
                declared_count: 2,
                elements: vec![
                    ElementEntry10_1::Null,
                    ElementEntry10_1::BackReference {
                        class_id: [0xa5; 16],
                        key: SEEN_KEY,
                    },
                ]
            },
            ElementGroup10_1 {
                declared_count: 0,
                elements: Vec::new()
            }
        ]
    );
    assert_eq!(parsed.stop, CompositeGraphicsLayerStop10_1::TailNotExpanded);
    assert_eq!(table.len(), 1, "空引用与回引不新增登记");

    // 组计数 0xffffffff（有符号 -1）：不迭代，组列表为空，计数位模式保留。
    let mut negative = 2_u16.to_le_bytes().to_vec();
    negative.extend(0xffff_ffff_u32.to_le_bytes());
    negative.extend(encoded("<默认>"));
    negative.extend([0xffff_u16, 0xffff, 0].iter().flat_map(|v| v.to_le_bytes()));
    negative.extend([0.0_f64, 0.0].iter().flat_map(|v| v.to_le_bytes()));
    negative.extend([3_u32, 1, 1].iter().flat_map(|v| v.to_le_bytes()));
    negative.extend([0_u8; 16]);
    negative.extend(0.5_f64.to_le_bytes());
    negative.extend(9_u32.to_le_bytes());
    negative.extend(0_u32.to_le_bytes());
    let mut reader = Reader::new(&negative);
    let parsed =
        read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap();
    assert_eq!(parsed.group_count, 0xffff_ffff);
    assert!(parsed.element_groups.is_empty());
    assert_eq!(parsed.stop, CompositeGraphicsLayerStop10_1::TailNotExpanded);
}

#[test]
fn stops_at_new_element_object() {
    // 元素循环中的新对象：停止并回退到其头起点；该组与该元素不记录。
    let new_header = entry_bytes([0x66; 16], 40);
    let groups = group_region(&[(2, &[vec![0_u8; 16], new_header.clone()])]);
    let mut bytes = cgl_body(
        2,
        &groups,
        "<默认>",
        &[0; 16],
        &sublayer_region(2, &[]),
        b"",
    );
    bytes.extend(b"opaque-element");
    // 新对象头是组区内的最后 20 字节：版本 2 + 组计数 + 元素计数 + 空引用。
    let header_start = 2 + groups.len() - new_header.len();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::ElementNewObject {
            group: 0,
            index: 1,
            class_id: [0x66; 16],
            key: 40
        }
    );
    assert!(parsed.element_groups.is_empty(), "停止组不记录");
    assert_eq!(parsed.name, "", "停止后的字段保持默认");
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start);
    assert!(!table.contains(40));
    assert!(table.is_empty());
}

#[test]
fn reads_sublayer_items_with_tail_fields() {
    // 版本 2 的完整子图层条目：String + ILayer（回引/空引用）+ 尾段
    // short×3+double×2+int32×2+嵌套对象+尾随 double。
    let items = [
        SubLayerItem {
            name: "注记甲",
            layer_entry: entry_bytes([0xa5; 16], SEEN_KEY),
            tail_nested_entry: vec![0_u8; 16],
        },
        SubLayerItem {
            name: "JZD1 注记",
            layer_entry: vec![0_u8; 16],
            tail_nested_entry: entry_bytes([0xa5; 16], SEEN_KEY),
        },
    ];
    let bytes = cgl_body(
        2,
        &group_region(&[]),
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, &items),
        b"tail",
    );
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.sublayer_count, 2);
    assert_eq!(
        parsed.sublayers,
        vec![
            SubLayerEntry10_1 {
                name: "注记甲".to_string(),
                layer: ConsumedEntry10_1::BackReference {
                    class_id: [0xa5; 16],
                    key: SEEN_KEY
                },
                tail_shorts: [1, 2, 3],
                tail_doubles: [8.5, -9.5],
                tail_int32s: [4, 5],
                tail_nested: Some(SubLayerTailSlot10_1::Null),
                tail_nested_collection: None,
                tail_double: Some(-1.75),
            },
            SubLayerEntry10_1 {
                name: "JZD1 注记".to_string(),
                layer: ConsumedEntry10_1::Null,
                tail_shorts: [1, 2, 3],
                tail_doubles: [8.5, -9.5],
                tail_int32s: [4, 5],
                tail_nested: Some(SubLayerTailSlot10_1::BackReference {
                    class_id: [0xa5; 16],
                    key: SEEN_KEY,
                }),
                tail_nested_collection: None,
                tail_double: Some(-1.75),
            },
        ]
    );
    assert_eq!(parsed.stop, CompositeGraphicsLayerStop10_1::TailNotExpanded);
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(table.len(), 1);
}

#[test]
fn stops_at_sublayer_new_object_keeping_consumed_name() {
    // 子图层条目的 ILayer 新对象（样本链：String 后紧跟 FeatureLayer 键 1555
    // 头 @0x1706e）：停止、条目不入列、已消费的条目名保存在停止原因中。
    let mut new_header = FEATURE_LAYER_CLASS_ID.to_vec();
    new_header.extend(1555_u32.to_le_bytes());
    let mut item_bytes = encoded("JZD1 注记");
    item_bytes.extend(&new_header);
    let mut sublayers = 1_u32.to_le_bytes().to_vec();
    sublayers.extend(&item_bytes);
    let mut bytes = cgl_body(
        6,
        &group_region(&[]),
        "<默认>",
        &[0_u8; 16],
        &sublayers,
        b"",
    );
    bytes.extend(b"opaque-feature-layer");
    let header_start = bytes.len() - new_header.len() - b"opaque-feature-layer".len();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::SubLayerNewObject {
            index: 0,
            name: "JZD1 注记".to_string(),
            class_id: FEATURE_LAYER_CLASS_ID,
            key: 1555
        }
    );
    assert!(parsed.sublayers.is_empty());
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start);
    assert!(!table.contains(1555));
    assert!(table.is_empty());
}

#[test]
fn stops_at_sublayer_tail_new_object_with_partial_entry() {
    // 子图层条目尾段嵌套对象为新对象：条目以部分消费状态入列（tail_nested/
    // tail_double 为 None），游标回退到该对象头起点。
    let new_header = entry_bytes([0x77; 16], 60);
    let item = SubLayerItem {
        name: "注记乙",
        layer_entry: vec![0_u8; 16],
        tail_nested_entry: new_header.clone(),
    };
    let mut bytes = cgl_body(
        2,
        &group_region(&[]),
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, std::slice::from_ref(&item)),
        b"",
    );
    bytes.extend(b"opaque-tail");
    // 新对象头之后仍有版本 >1 的尾随 double（sublayer_region 写在其后），
    // 然后才是追加的未知区。
    let header_start = bytes.len() - new_header.len() - 8 - b"opaque-tail".len();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::SubLayerTailNewObject {
            index: 0,
            class_id: [0x77; 16],
            key: 60
        }
    );
    assert_eq!(parsed.sublayers.len(), 1);
    let entry = &parsed.sublayers[0];
    assert_eq!(entry.name, "注记乙");
    assert_eq!(entry.layer, ConsumedEntry10_1::Null);
    assert_eq!(entry.tail_shorts, [1, 2, 3]);
    assert_eq!(entry.tail_doubles, [8.5, -9.5]);
    assert_eq!(entry.tail_int32s, [4, 5]);
    assert_eq!(entry.tail_nested, None);
    assert_eq!(entry.tail_double, None);
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start);
    assert!(!table.contains(60));
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断扫描：覆盖版本、组区、名称、short/double/int32、嵌套对象、
    // 子图层区各字段内部。任何截断都必须失败、游标归位、登记表不变且重读
    // 得到相同错误类别。
    let item = SubLayerItem {
        name: "JZD1 注记",
        layer_entry: entry_bytes([0xa5; 16], SEEN_KEY),
        tail_nested_entry: vec![0_u8; 16],
    };
    let bytes = cgl_body(
        2,
        &group_region(&[(1, &[vec![0_u8; 16]])]),
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, &[item]),
        b"tail",
    );
    let consumed = bytes.len() - 4;
    // 恰好在停止点截断（未知尾部缺失）是合法完整输入；回引键须预置。
    let mut seeded = RefTable10_1::new();
    seeded.register(SEEN_KEY);
    let mut boundary = Reader::new(&bytes[..consumed]);
    let parsed =
        read_composite_graphics_layer_prefix_10_1(&mut boundary, 1024, &mut seeded).unwrap();
    assert_eq!(parsed.remaining_bytes, 0);
    for end in 0..consumed {
        let mut table = RefTable10_1::new();
        table.register(SEEN_KEY);
        let mut reader = Reader::new(&bytes[..end]);
        let first = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        assert_eq!(table.len(), 1, "end={end}");
        assert!(table.contains(SEEN_KEY), "end={end}");
        let second = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
    let mut seeded = RefTable10_1::new();
    seeded.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    assert!(read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut seeded).is_ok());
    assert_eq!(reader.remaining(), 4);
}

#[test]
fn rejects_over_limit_string_lengths_before_reading_or_allocating() {
    // 图形层名称限额：声明长度（含终止零）等于限额可读，少一字节在读取与
    // 分配之前拒绝。
    let bytes = cgl_body(
        2,
        &group_region(&[]),
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, &[]),
        b"",
    );
    let name_declared = 10_usize; // 4 字 ×2 + 终止零 2。
    let mut reader = Reader::new(&bytes);
    assert!(
        read_composite_graphics_layer_prefix_10_1(
            &mut reader,
            name_declared,
            &mut RefTable10_1::new()
        )
        .is_ok()
    );
    let mut reader = Reader::new(&bytes);
    let error = read_composite_graphics_layer_prefix_10_1(
        &mut reader,
        name_declared - 1,
        &mut RefTable10_1::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // 子图层条目名称同样受限额约束：第 1 项条目名声明 12 超过限额 10，在
    // 读取与分配之前拒绝、游标归位。
    let item = SubLayerItem {
        name: "JZD1 注记",
        layer_entry: vec![0_u8; 16],
        tail_nested_entry: vec![0_u8; 16],
    };
    let bytes = cgl_body(
        2,
        &group_region(&[]),
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, &[item]),
        b"",
    );
    let mut reader = Reader::new(&bytes);
    let error = read_composite_graphics_layer_prefix_10_1(
        &mut reader,
        name_declared,
        &mut RefTable10_1::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // u32::MAX-1（最大偶数声明长度）：有限额时按限额拒绝；放开限额时因流过
    // 短缺读拒绝；两者都不推进游标、不做无界分配。
    let mut huge = 2_u16.to_le_bytes().to_vec();
    huge.extend(0_u32.to_le_bytes());
    huge.extend((u32::MAX - 1).to_le_bytes());
    huge.extend(b"new");
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_composite_graphics_layer_prefix_10_1(
            &mut reader,
            usize::MAX,
            &mut RefTable10_1::new()
        )
        .unwrap_err()
        .kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn sample_basemap_cgl_body_advances_to_sublayer_feature_layer_stop() {
    // 仓库参考样本的辅助观察回归对照（非目标软件执行验证）：/Maps 导出中
    // CGL 对象头 @0x16fba 键 1553，体 @0x16fce 起。分派接线后的预期推进：
    // 第 7 步 ElementCollection 键 1554 体（版本 4 + 计数 0）完整消费 →
    // 第 8 步 double+int32 → 第 9 步子图层计数 10 @0x17056（第六轮裁决）→
    // 条目 1 = String `JZD1 注记` + FeatureLayer 键 1555 头 @0x1706e 新对
    // 象停止。偏移与键来自 docs/arcmap-10.1-map-object.md 第四/六轮与本轮
    // hex 复核。
    use std::io::Read as _;

    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    let body_offset = 0x16fce_usize;
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&maps[body_offset..]);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 0x10000, &mut table)
        .expect("样本 CGL 体应可解析到停止点");
    assert_eq!(parsed.version, 6);
    assert_eq!(parsed.group_count, 11);
    assert_eq!(
        parsed.element_groups,
        vec![
            ElementGroup10_1 {
                declared_count: 0,
                elements: Vec::new()
            };
            11
        ]
    );
    assert_eq!(parsed.name, "<默认>");
    assert_eq!(parsed.shorts, [0xffff, 0xffff, 0x0000]);
    assert_eq!(parsed.doubles, [0.0, 0.0]);
    assert_eq!(
        (parsed.int32_4c, parsed.int32_50, parsed.int32_local),
        (3, 1, 1)
    );
    // 第 7 步分派：ElementCollection 键 1554，体完整消费（版本 4、计数 0）。
    assert_eq!(parsed.nested_1c, None);
    let collection = parsed
        .nested_1c_collection
        .as_ref()
        .expect("样本第 7 步为 ElementCollection");
    assert_eq!(collection.version, 4);
    assert_eq!(collection.count, 0);
    assert_eq!(collection.stop, None);
    assert!(table.contains(1554));
    // 第 8 步 + 第 9 步计数（第六轮裁决：0x17052=1 是第 8 步 int32，计数在
    // 0x17056=10）。
    assert_eq!(parsed.after_version1, Some((0.0, 1)));
    assert_eq!(parsed.sublayer_count, 10);
    assert!(parsed.sublayers.is_empty());
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::SubLayerNewObject {
            index: 0,
            name: "JZD1 注记".to_string(),
            class_id: FEATURE_LAYER_CLASS_ID,
            key: 1555
        }
    );
    // 停止点 = 0x1706e（FeatureLayer 头起点）。
    assert_eq!(reader.position(), 0x1706e - body_offset);
    assert_eq!(parsed.remaining_bytes, maps.len() - 0x1706e);
    assert!(!table.contains(1555));
}

#[test]
fn element_loop_and_sublayer_tail_dispatch_element_collections_inline() {
    // 接线轮：第 2 步元素条目与第 9 步条目尾段嵌套对象按类标识分派
    // ElementCollection 体（与第 7 步同模式；落位记录见 map-object.md 文末
    // 「接线轮」节）。体取样本第 7 步形态：版本 4 + 计数 0（6 字节完整消
    // 费，无体内停止）；尾段体完整消费后版本 >1 的尾随 double 可达。
    let collection_body = {
        let mut b = 4_u16.to_le_bytes().to_vec();
        b.extend(0_u32.to_le_bytes());
        b
    };
    let element_entry = {
        let mut e = entry_bytes(ELEMENT_COLLECTION_CLASS_ID, 90);
        e.extend_from_slice(&collection_body);
        e
    };
    let tail_entry = {
        let mut e = entry_bytes(ELEMENT_COLLECTION_CLASS_ID, 91);
        e.extend_from_slice(&collection_body);
        e
    };
    let groups = group_region(&[(1, &[element_entry])]);
    let items = [SubLayerItem {
        name: "JZD1 注记",
        layer_entry: vec![0_u8; 16], // ILayer 空标记
        tail_nested_entry: tail_entry,
    }];
    let bytes = cgl_body(
        2,
        &groups,
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, &items),
        b"",
    );
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    // 两处体整体消费、无体内停止：游标停在第 10 步尾段（TailNotExpanded）。
    assert!(matches!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::TailNotExpanded
    ));
    assert_eq!(parsed.element_groups.len(), 1);
    match &parsed.element_groups[0].elements[0] {
        ElementEntry10_1::Collection(collection) => {
            assert_eq!(collection.stop, None);
            assert_eq!(collection.count, 0);
        }
        other => panic!("元素条目应为 ElementCollection 体：{other:?}"),
    }
    match &parsed.sublayers[0].tail_nested {
        Some(SubLayerTailSlot10_1::Collection { key }) => assert_eq!(*key, 91),
        other => panic!("尾段槽应为 Collection 分类：{other:?}"),
    }
    let body = parsed.sublayers[0].tail_nested_collection.as_ref().unwrap();
    assert_eq!(body.stop, None);
    assert_eq!(
        parsed.sublayers[0].tail_double,
        Some(-1.75),
        "体完整消费后版本 >1 的尾随 double 可达"
    );
    assert!(table.contains(90) && table.contains(91));
    assert_eq!(
        parsed.remaining_bytes, 0,
        "尾段为空，读取器在 TailNotExpanded 停止"
    );
}

#[test]
fn element_collection_inner_stop_propagates_from_element_loop() {
    // 第 2 步 ElementCollection 体在内部未知元素处停止：透传为
    // ElementCollectionIncomplete（键按「登记先于持久化」保留，游标停在内
    // 层停止点，组内后继条目不可达）。
    let collection_body_offset = 2 + 4; // 体 = 版本 + 计数之后
    let mut collection_body = 4_u16.to_le_bytes().to_vec();
    collection_body.extend(1_u32.to_le_bytes()); // 元素计数 1
    let unknown_header_offset = collection_body.len();
    collection_body.extend(entry_bytes(UNKNOWN_CLASS_ID, 99)); // 体内停止点
    let element_entry = {
        let mut e = entry_bytes(ELEMENT_COLLECTION_CLASS_ID, 90);
        e.extend_from_slice(&collection_body);
        e
    };
    let groups = group_region(&[(1, &[element_entry])]);
    let bytes = cgl_body(
        2,
        &groups,
        "<默认>",
        &[0_u8; 16],
        &sublayer_region(2, &[]),
        b"",
    );
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let parsed = read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    // 外层停止原因携带键 90 与内层原因（未知元素键 99，内层未登记）。
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::ElementCollectionIncomplete {
            group: 0,
            index: 0,
            key: 90,
            reason: ElementCollectionStop10_1::ElementBodyNotClosed {
                index: 0,
                class_id: UNKNOWN_CLASS_ID,
                key: 99,
            },
        }
    );
    // 游标停在内层未知元素对象头起点：传入 Reader 自体首起（对象头由调用
    // 者消费）：版本(2) + 组计数(4) + 组内计数(4) + 条目头(20) + 体内偏移。
    let entry_offset = 2 + 4 + 4 + 20;
    assert_eq!(reader.position(), entry_offset + unknown_header_offset);
    // 内层键 99 不登记；外层 ElementCollection 键 90 已登记。
    assert!(table.contains(90));
    assert!(!table.contains(99));
}
