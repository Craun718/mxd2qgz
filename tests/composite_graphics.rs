//! `composite_graphics` 读取器的合成输入测试（证据 B 字段序）。
//!
//! 仓库参考样本 `samples/测试底图.mxd` 的 /Maps 导出中有一个
//! CompositeGraphicsLayer 对象（头 @0x16fba 键 1553，体 @0x16fce 起）；本文件
//! 不读取样本文件，用与样本逐段对齐的合成字节镜像该布局（值来自
//! docs/arcmap-10.1-map-object.md 第四轮与本次 hex 复核，属辅助观察，不是
//! 目标软件执行验证）。

use mxd2qgz::composite_graphics::{
    COMPOSITE_GRAPHICS_LAYER_CLASS_ID, CompositeGraphicsLayerStop10_1, ConsumedEntry10_1,
    ElementGroup10_1, SubLayerEntry10_1, read_composite_graphics_layer_prefix_10_1,
};
use mxd2qgz::object_refs::RefTable10_1;
use mxd2qgz::reader::Reader;
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
fn reads_sample_layout_prefix_and_stops_at_element_collection() {
    // 镜像样本 /Maps:0x16fce..0x17030 的已确认段：版本 6、11 组全零、名称
    // `<默认>`、short 0xffff/0xffff/0、double 0/0、int32 3/1/1，随后第 7 步
    // 对象头是 ElementCollection 键 1554——新对象，本读取器在此停止。其后
    // 字节（样本中另有子图层计数二解区，见模块文档）只作未知区参与断言。
    let group_region = {
        let mut bytes = 11_u32.to_le_bytes().to_vec();
        for _ in 0..11 {
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes
    };
    let mut ec = ELEMENT_COLLECTION_CLASS_ID.to_vec();
    ec.extend(1554_u32.to_le_bytes());
    ec.extend(b"opaque-collection-body");
    let bytes = cgl_body(6, &group_region, "<默认>", &ec, b"sublayer-junk", b"tail");
    let header_start = 2 + group_region.len() + encoded("<默认>").len() + 6 + 16 + 12;
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
    // 新对象停止：第 7 步字段为 None，其后字段保持默认。
    assert_eq!(parsed.nested_1c, None);
    assert_eq!(parsed.after_version1, None);
    assert_eq!(parsed.sublayer_count, 0);
    assert!(parsed.sublayers.is_empty());
    assert_eq!(
        parsed.stop,
        CompositeGraphicsLayerStop10_1::NestedNewObject {
            class_id: ELEMENT_COLLECTION_CLASS_ID,
            key: 1554
        }
    );
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start, "游标回退到对象头起点");
    assert!(!table.contains(1554), "未消费的新键不得登记");
    assert!(table.is_empty());
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
                    ConsumedEntry10_1::Null,
                    ConsumedEntry10_1::BackReference {
                        class_id: [0xa5; 16],
                        key: SEEN_KEY
                    }
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
                tail_nested: Some(ConsumedEntry10_1::Null),
                tail_double: Some(-1.75),
            },
            SubLayerEntry10_1 {
                name: "JZD1 注记".to_string(),
                layer: ConsumedEntry10_1::Null,
                tail_shorts: [1, 2, 3],
                tail_doubles: [8.5, -9.5],
                tail_int32s: [4, 5],
                tail_nested: Some(ConsumedEntry10_1::BackReference {
                    class_id: [0xa5; 16],
                    key: SEEN_KEY
                }),
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
