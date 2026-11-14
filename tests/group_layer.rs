//! `group_layer` 读取器的合成输入测试（证据 A 字段序）。
//!
//! 仓库参考样本无组图层（docs/arcmap-10.1-map-object.md「未解决事项」2），
//! GroupLayer 字段序只有 FeatureLayer.dll `0x6f711270` 反编译证据；本文件全
//! 部用合成字节构造，不依赖样本文件。ExtraSafe 前缀的 `[A][B]` 布局与
//! 「A=头 20 字节+体的总体积」有 /Maps 图层列表（Map 图层循环）的逐条目
//! 吻合（第四轮），仅作布局佐证。

use mxd2qgz::group_layer::{
    ConsumedEntry10_1, GROUP_LAYER_CLASS_ID, GroupChildEntry10_1, GroupLayerStop10_1,
    read_group_layer_prefix_10_1,
};
use mxd2qgz::object_refs::RefTable10_1;
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

/// 合成输入固定使用的名称后三个 short（语义未定性，取可辨识值）。
const SHORTS: [u16; 3] = [0x1234, 0x0001, 0x8000];
/// 合成输入固定使用的两个 double。
const DOUBLES: [f64; 2] = [1.5, -0.25];
/// 预置进登记表的既有键：命中即回引。
const SEEN_KEY: u32 = 7;

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

/// ExtraSafe 条目的 8 字节前缀 `[A:int32][B:int32]`。
fn extra_safe_prefix(a: u32, b: u32) -> Vec<u8> {
    let mut bytes = a.to_le_bytes().to_vec();
    bytes.extend(b.to_le_bytes());
    bytes
}

/// 按证据 A 字段序构造 GroupLayer 对象体：版本、名称、short ×3、
/// double ×2、子图层区（调用者提供完整计数与条目字节）、可见性（版本 >=2
/// 才写）、嵌套对象（版本 >2 才写，反编译 `2 < ver`）、double（版本 >=4 才
/// 写）、未知尾部。
fn group_body(
    version: u16,
    name: &str,
    child_region: &[u8],
    visibility: u16,
    nested: &[u8],
    nested_double: f64,
    tail: &[u8],
) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(encoded(name));
    for value in SHORTS {
        bytes.extend(value.to_le_bytes());
    }
    for value in DOUBLES {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(child_region);
    if version >= 2 {
        bytes.extend(visibility.to_le_bytes());
    }
    if version > 2 {
        bytes.extend(nested);
    }
    if version >= 4 {
        bytes.extend(nested_double.to_le_bytes());
    }
    bytes.extend(tail);
    bytes
}

/// 子图层区：计数 + 每条目的原始字节。
fn child_region(count: u32, entries: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = count.to_le_bytes().to_vec();
    for entry in entries {
        bytes.extend_from_slice(entry);
    }
    bytes
}

#[test]
fn group_layer_class_id_constant_matches_documented_guid() {
    // 证据：{EDAD6647-1810-11D1-86AE-0000F8751720}（esri.clsid.ecfg 声明 +
    // FeatureLayer.dll RTTI .?AVGroupLayer@@，Load 0x6f711270）。样本无组
    // 图层，无样本头佐证；Data1-3 小端、Data4 原序。
    assert_eq!(
        GROUP_LAYER_CLASS_ID,
        [
            0x47, 0x66, 0xad, 0xed, 0x10, 0x18, 0xd1, 0x11, 0x86, 0xae, 0x00, 0x00, 0xf8, 0x75,
            0x17, 0x20,
        ]
    );
}

#[test]
fn reads_full_prefix_with_plain_children_and_tail_stop() {
    // 版本 4（<5）：可见性、嵌套、double 全部出现，之后是未展开尾段。
    let children = child_region(
        2,
        &[
            vec![0_u8; 16],                    // 空引用条目
            entry_bytes([0xa5; 16], SEEN_KEY), // 回引条目（键已预置）
        ],
    );
    let bytes = group_body(4, "图层组", &children, 0xffff, &[0_u8; 16], 42.5, b"tail");
    let prefix_length = bytes.len() - 4;
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let parsed = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, 4);
    assert_eq!(parsed.name, "图层组");
    assert_eq!(parsed.shorts_after_name, SHORTS);
    assert_eq!(parsed.doubles, DOUBLES);
    assert_eq!(parsed.child_count, 2);
    assert_eq!(
        parsed.children,
        vec![
            GroupChildEntry10_1::Null,
            GroupChildEntry10_1::BackReference {
                class_id: [0xa5; 16],
                key: SEEN_KEY
            },
        ]
    );
    assert_eq!(parsed.visibility, Some(0xffff));
    assert_eq!(
        parsed.nested_after_visibility,
        Some(ConsumedEntry10_1::Null)
    );
    assert_eq!(parsed.double_after_nested, Some(42.5));
    assert_eq!(parsed.stop, GroupLayerStop10_1::TailNotExpanded);
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), prefix_length);
    // 空引用与回引都不新增登记；提交后登记表仍只有预置键。
    assert_eq!(table.len(), 1);
    assert!(table.contains(SEEN_KEY));
}

#[test]
fn visibility_and_nested_follow_documented_version_gates() {
    // 可见性：版本 <2 不读（None）、>=2 读（样本同构 VARIANT_BOOL 位模式）。
    // 嵌套对象：反编译为 `2 < ver`（文档表格标注 >=2，以反编译为准）——版本
    // 2 不出现、3 出现。double：>=4。
    for (version, visibility, nested, double) in [
        (1_u16, None, None, None),
        (2, Some(0xffff_u16), None, None),
        (3, Some(0x0000_u16), Some(ConsumedEntry10_1::Null), None),
        (
            4,
            Some(0x0002_u16),
            Some(ConsumedEntry10_1::Null),
            Some(0.5),
        ),
    ] {
        let bytes = group_body(
            version,
            "分组😀图层",
            &child_region(0, &[]),
            visibility.unwrap_or(0xffff),
            &[0_u8; 16],
            double.unwrap_or(0.0),
            b"tail",
        );
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(parsed.version, version, "version={version}");
        assert_eq!(parsed.visibility, visibility, "version={version}");
        assert_eq!(parsed.nested_after_visibility, nested, "version={version}");
        assert_eq!(parsed.double_after_nested, double, "version={version}");
        // 版本 4 起在 ExtensionHelper 处停止，之前在尾段停止。
        let expected_stop = if version >= 5 {
            GroupLayerStop10_1::ExtensionHelperNotExpanded
        } else {
            GroupLayerStop10_1::TailNotExpanded
        };
        assert_eq!(parsed.stop, expected_stop, "version={version}");
        assert_eq!(parsed.remaining_bytes, 4, "version={version}");
        // 名称含非 ASCII 中文与代理对 emoji，按位逐字往返。
        assert_eq!(parsed.name, "分组😀图层", "version={version}");
    }
}

#[test]
fn accepts_version_boundaries_zero_and_twelve_rejects_above() {
    // 接受范围 0..=12（原生 0xc < ver 拒绝）。版本 0 无任何版本条件字段；
    // 版本 12（>=5）在 ExtensionHelper 处停止。
    for (version, expected_stop) in [
        (0_u16, GroupLayerStop10_1::TailNotExpanded),
        (12, GroupLayerStop10_1::ExtensionHelperNotExpanded),
    ] {
        let bytes = group_body(
            version,
            "图层组",
            &child_region(0, &[]),
            0xffff,
            &[0_u8; 16],
            1.0,
            b"tail",
        );
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(parsed.version, version);
        assert_eq!(parsed.stop, expected_stop, "version={version}");
        assert_eq!(parsed.remaining_bytes, 4, "version={version}");
    }
    // >12 拒绝：13 是首个拒绝值；负值位模式在无符号解释下同样拒绝（原生
    // 为有符号 short 比较，负值可能通过原生检查——符号性未闭合，见模块文档）。
    for version in [13_u16, 100, 0x8000, 0xffff] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded(""));
        let mut reader = Reader::new(&bytes);
        let first =
            read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version}");
        assert!(first.to_string().contains("版本"), "version={version}");
        assert_eq!(reader.position(), 0, "version={version}");
        let second =
            read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version}");
        assert_eq!(reader.position(), 0, "version={version}");
    }
}

#[test]
fn stops_at_new_child_object_below_version_eleven() {
    // 版本 <11 的普通条目：新对象体无外部长度字段，停止而非报错；游标回退
    // 到该对象头起点，未消费的新键不登记。
    let new_header = entry_bytes([0x33; 16], 21);
    let children = child_region(
        2,
        &[
            entry_bytes([0xa5; 16], SEEN_KEY), // 回引（键已预置）
            new_header.clone(),
        ],
    );
    // 版本 10 时可见性/嵌套/double 也会写入（位于新对象头之后的未知区），
    // 新对象头起点固定在头字段 + 计数 + 回引条目之后。
    let mut bytes = group_body(10, "图层组", &children, 0xffff, &[0; 16], 1.0, b"");
    bytes.extend(b"opaque-body"); // 新对象的未知体之后的未知区
    let header_start = 2 + encoded("图层组").len() + 6 + 16 + 4 + 20;
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let parsed = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        GroupLayerStop10_1::ChildNewObject {
            index: 1,
            class_id: [0x33; 16],
            key: 21
        }
    );
    assert_eq!(parsed.children.len(), 1);
    assert_eq!(parsed.visibility, None, "停止先于可见性字段");
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start, "游标回退到对象头起点");
    assert_eq!(table.len(), 1);
    assert!(table.contains(SEEN_KEY));
    assert!(!table.contains(21), "未消费的新键不得登记");
}

#[test]
fn extra_safe_entries_empty_back_reference_and_bounded_skip() {
    // 版本 >=11：[0][0] 空条目；A=20 回引（无体）；A=25 新对象跳过 5 字节体。
    // A 的体积语义是写侧观察（读侧原生不消费 A），本读取器用作有界跳过。
    let head_len = {
        let mut head = 10_u16.to_le_bytes().to_vec(); // 占位，仅求长度
        head.extend(encoded("图层组"));
        for value in SHORTS {
            head.extend(value.to_le_bytes());
        }
        for value in DOUBLES {
            head.extend(value.to_le_bytes());
        }
        head.len()
    };
    let new_entry = entry_bytes([0x44; 16], 22);
    let children = child_region(
        3,
        &[
            extra_safe_prefix(0, 0),
            {
                let mut bytes = extra_safe_prefix(20, 0);
                bytes.extend(entry_bytes([0xa5; 16], SEEN_KEY));
                bytes
            },
            {
                let mut bytes = extra_safe_prefix(25, 0);
                bytes.extend(new_entry);
                bytes.extend(b"body!"); // A-20 = 5 字节未知体
                bytes
            },
        ],
    );
    let bytes = group_body(11, "图层组", &children, 0xffff, &[0; 16], 7.25, b"tail");
    // 第三个条目未知体的起始偏移：头 + 计数 + 空前缀 +（前缀+头）+（前缀+头）。
    let body_offset = head_len + 4 + 8 + (8 + 20) * 2;
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let parsed = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.children,
        vec![
            GroupChildEntry10_1::Empty,
            GroupChildEntry10_1::BackReference {
                class_id: [0xa5; 16],
                key: SEEN_KEY
            },
            GroupChildEntry10_1::SkippedNewObject {
                class_id: [0x44; 16],
                key: 22,
                declared_total: 25,
                body_offset,
                body_length: 5
            },
        ]
    );
    // 版本 11 >=5：可见性、嵌套、double 已读，随后在 ExtensionHelper 停止。
    assert_eq!(parsed.stop, GroupLayerStop10_1::ExtensionHelperNotExpanded);
    assert_eq!(parsed.remaining_bytes, 4);
    // 成功提交：跳过体的新键 22 已登记。
    assert_eq!(table.len(), 2);
    assert!(table.contains(22));
    // 跳过的 5 字节仍在输入中，游标越过它们。
    assert_eq!(&bytes[body_offset..body_offset + 5], b"body!");
}

#[test]
fn extra_safe_prefix_below_twenty_is_rejected() {
    // A<20 与写侧不变量（A=头 20 字节+体）矛盾，无法用作有界跳过：显式报错，
    // 游标与登记表不动（新键 22 已在 trial 登记后随失败一并丢弃）。
    let mut entry = extra_safe_prefix(8, 0);
    entry.extend(entry_bytes([0x44; 16], 22));
    let bytes = group_body(
        11,
        "图层组",
        &child_region(1, &[entry]),
        0xffff,
        &[0; 16],
        1.0,
        b"",
    );
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let error = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    let message = error.to_string();
    assert!(message.contains("20"), "{message}");
    assert!(message.contains("A=8"), "{message}");
    assert_eq!(reader.position(), 0);
    assert_eq!(table.len(), 1);
    assert!(!table.contains(22));
}

#[test]
fn extra_safe_b_sync_overrides_a_skip() {
    // A=30 跳过 10 字节体后落在前缀后 30 字节处；B=40 要求同步到前缀后 40
    // 字节处——冲突时以反编译证实的 B 同步为准，多出的 10 字节被越过。
    let mut item = extra_safe_prefix(30, 40);
    item.extend(entry_bytes([0x44; 16], 22));
    item.extend(b"0123456789"); // A-20 = 10 字节未知体
    item.extend(b"ABCDEFGHIJ"); // B 同步越过的 10 字节
    let children = child_region(1, &[item]);
    let bytes = group_body(11, "图层组", &children, 0xffff, &[0; 16], 7.25, b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(
        parsed.children,
        vec![GroupChildEntry10_1::SkippedNewObject {
            class_id: [0x44; 16],
            key: 22,
            declared_total: 30,
            body_offset: 2 + encoded("图层组").len() + 6 + 16 + 4 + 8 + 20,
            body_length: 10
        }]
    );
    // 同步后可见性/嵌套/double 正常读取、ExtensionHelper 停止：证明 B 同步
    // 恰好越过填充字节。
    assert_eq!(parsed.visibility, Some(0xffff));
    assert_eq!(parsed.stop, GroupLayerStop10_1::ExtensionHelperNotExpanded);
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);

    // 反向冲突：A=40 声明 20 字节体（越过其后 group_body 写入的可见性/
    // 嵌套区），B=30 把游标拉回到前缀后 30 字节处——恰好是可见性 short 的
    // 位置。以 B 同步为准，后续字段从那里重新读取；A 记录的未知体范围
    // （含重叠）仅是写侧观察值的保留。
    let mut item = extra_safe_prefix(40, 30);
    item.extend(entry_bytes([0x44; 16], 23));
    item.extend([0xAA; 10]); // A 声明体的前半；后半与可见性/嵌套区重叠
    let children = child_region(1, &[item]);
    let bytes = group_body(11, "图层组", &children, 0x0001, &[0; 16], 7.25, b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(
        parsed.children[0],
        GroupChildEntry10_1::SkippedNewObject {
            class_id: [0x44; 16],
            key: 23,
            declared_total: 40,
            body_offset: 2 + encoded("图层组").len() + 6 + 16 + 4 + 8 + 20,
            body_length: 20
        }
    );
    // B=30 把游标拉回后，可见性/嵌套从 B 目标处的字节重新读取。
    assert_eq!(parsed.visibility, Some(0x0001));
    assert_eq!(
        parsed.nested_after_visibility,
        Some(ConsumedEntry10_1::Null)
    );
    assert_eq!(parsed.stop, GroupLayerStop10_1::ExtensionHelperNotExpanded);
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
}

#[test]
fn extra_safe_b_negative_or_beyond_stream_is_rejected() {
    // B 为负值位模式：原生回退 Seek 行为未证实，直接拒绝。
    let mut negative = extra_safe_prefix(20, 0x8000_0000);
    negative.extend(entry_bytes([0xa5; 16], SEEN_KEY));
    let bytes = group_body(
        11,
        "图层组",
        &child_region(1, &[negative]),
        0xffff,
        &[0; 16],
        1.0,
        b"",
    );
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let error = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("负值"), "{error}");
    assert_eq!(reader.position(), 0);
    assert_eq!(table.len(), 1);

    // B 同步目标超出输入末尾：显式拒绝（回引条目本身合法，B 才是问题）。
    let mut beyond = extra_safe_prefix(20, 1000);
    beyond.extend(entry_bytes([0xa5; 16], SEEN_KEY));
    let bytes = group_body(
        11,
        "图层组",
        &child_region(1, &[beyond]),
        0xffff,
        &[0; 16],
        1.0,
        b"",
    );
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    let error = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("超出输入末尾"), "{error}");
    assert_eq!(reader.position(), 0);
    assert_eq!(table.len(), 1);
}

#[test]
fn stops_at_new_nested_object_above_version_two() {
    // 版本 >2 的嵌套对象（IEnvelope 候选）为新对象：停止并回退到其头起点。
    let new_header = entry_bytes([0x55; 16], 30);
    let mut bytes = group_body(
        3,
        "图层组",
        &child_region(0, &[]),
        0xffff,
        &new_header,
        1.0,
        b"",
    );
    bytes.extend(b"opaque");
    let header_start = bytes.len() - new_header.len() - b"opaque".len();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        GroupLayerStop10_1::NestedNewObject {
            class_id: [0x55; 16],
            key: 30
        }
    );
    assert_eq!(parsed.nested_after_visibility, None);
    assert_eq!(parsed.double_after_nested, None, "版本 3 无 >=4 double");
    assert_eq!(parsed.remaining_bytes, bytes.len() - header_start);
    assert_eq!(reader.position(), header_start);
    assert!(!table.contains(30));
    assert!(table.is_empty());
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断扫描：普通条目（版本 4）与 ExtraSafe 条目（版本 11）各一遍。
    // 任何截断都必须失败、游标归位、登记表不变且重读得到相同错误类别。
    let plain_children = child_region(2, &[vec![0_u8; 16], entry_bytes([0xa5; 16], SEEN_KEY)]);
    let plain = group_body(
        4,
        "图层组",
        &plain_children,
        0xffff,
        &[0; 16],
        42.5,
        b"tail",
    );
    let mut skipped_entry = extra_safe_prefix(25, 0);
    skipped_entry.extend(entry_bytes([0x44; 16], 22));
    skipped_entry.extend(b"body!");
    let extra_safe = group_body(
        11,
        "图层组",
        &child_region(1, &[skipped_entry]),
        0xffff,
        &[0; 16],
        7.25,
        b"tail",
    );
    for bytes in [&plain, &extra_safe] {
        let consumed = bytes.len() - 4;
        // 恰好在停止点截断（未知尾部缺失）是合法完整输入；回引键须预置才能
        // 按回引分类（空表会把同一字节分类为新对象——原生语义）。
        let mut seeded = RefTable10_1::new();
        seeded.register(SEEN_KEY);
        let mut boundary = Reader::new(&bytes[..consumed]);
        let parsed = read_group_layer_prefix_10_1(&mut boundary, 1024, &mut seeded).unwrap();
        assert_eq!(parsed.remaining_bytes, 0, "len={consumed:#x}");
        for end in 0..consumed {
            let mut table = RefTable10_1::new();
            table.register(SEEN_KEY);
            let mut reader = Reader::new(&bytes[..end]);
            let first = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table)
                .err()
                .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
            assert_eq!(reader.position(), 0, "end={end}");
            assert_eq!(table.len(), 1, "end={end}");
            assert!(table.contains(SEEN_KEY), "end={end}");
            assert!(!table.contains(22), "end={end}");
            let second = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table)
                .err()
                .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
            assert_eq!(first.kind(), second.kind(), "end={end}");
            assert_eq!(reader.position(), 0, "end={end}");
        }
        // 完整输入本身必须成功（预置回引键），未知尾部保留在游标之后。
        let mut seeded = RefTable10_1::new();
        seeded.register(SEEN_KEY);
        let mut reader = Reader::new(bytes);
        assert!(read_group_layer_prefix_10_1(&mut reader, 1024, &mut seeded).is_ok());
        assert_eq!(reader.remaining(), 4);
    }
}

#[test]
fn failed_parse_after_skipped_object_leaves_table_unchanged() {
    // 版本 11：第 1 条目（A=25）的新键已按跳过登记，第 2 条目前缀截断导致
    // 整体失败——trial 登记必须整体丢弃，调用者登记表不变。
    let mut first = extra_safe_prefix(25, 0);
    first.extend(entry_bytes([0x44; 16], 22));
    first.extend(b"body!");
    let mut children = child_region(2, &[first]);
    children.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // 第 2 条目仅 4 字节前缀
    let bytes = group_body(11, "图层组", &children, 0xffff, &[0; 16], 1.0, b"");
    let mut table = RefTable10_1::new();
    table.register(SEEN_KEY);
    let mut reader = Reader::new(&bytes);
    assert!(read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(table.len(), 1);
    assert!(!table.contains(22), "失败路径不得保留 trial 登记");
}

#[test]
fn rejects_over_limit_name_lengths_before_reading_or_allocating() {
    // 名称限额边界：声明长度（含终止零）等于限额可读；少一字节在读取与分配
    // 之前拒绝。
    let children = child_region(0, &[]);
    let bytes = group_body(4, "新建图层组", &children, 0xffff, &[0; 16], 1.0, b"");
    let name_declared = 12_usize; // 5 字 ×2 + 终止零 2。
    let mut reader = Reader::new(&bytes);
    assert!(
        read_group_layer_prefix_10_1(&mut reader, name_declared, &mut RefTable10_1::new()).is_ok()
    );
    let mut reader = Reader::new(&bytes);
    let error =
        read_group_layer_prefix_10_1(&mut reader, name_declared - 1, &mut RefTable10_1::new())
            .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // u32::MAX-1（最大偶数声明长度）：有限额时按限额拒绝；放开限额时因流过
    // 短缺读拒绝；两者都不推进游标、不做无界分配。
    let mut huge = 4_u16.to_le_bytes().to_vec();
    huge.extend((u32::MAX - 1).to_le_bytes());
    huge.extend(b"new");
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_group_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_group_layer_prefix_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
            .unwrap_err()
            .kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);
}
