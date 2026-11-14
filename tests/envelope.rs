use mxd2qgz::envelope::{ENVELOPE_CLASS_ID, SpatialReferenceEntry10_1, read_envelope_10_1};
use mxd2qgz::object_refs::RefTable10_1;
use mxd2qgz::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::reader::Reader;
use mxd2qgz::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use std::io::ErrorKind;

/// 样本 Map PCS 对象键（`/Maps:0x30` 键 30），Envelope 回引的最常见目标。
const MAP_PCS_KEY: u32 = 30;
/// 样本首个图层体内嵌新 PCS 的键（Envelope 键 80 体尾内嵌键 81）。
const NESTED_PCS_KEY: u32 = 81;
/// 合成包络坐标，保持写方向序的次序关系（d0<d2、d1<d3）。
const COORDS: [f64; 4] = [10.0, 20.0, 30.0, 40.0];
/// Z 域候选 double 对（合成值，与 M 域取值区分以钉住流内顺序）。
const Z_PAIR: [f64; 2] = [11.0, 12.0];
/// M 域候选 double 对（合成值）。
const M_PAIR: [f64; 2] = [21.0, 22.0];

/// 版本 short + 32 字节 WKSEnvelope（4 个小端 double，写方向序）。
fn envelope_prefix(version: u16, coords: &[f64; 4]) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    for value in coords {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// 追加版本 2/3 分支的 int32。
fn push_attribute_flags(bytes: &mut Vec<u8>, flags: u32) {
    bytes.extend(flags.to_le_bytes());
}

/// 追加一对 double（Z/M 域候选段）。
fn push_pair(bytes: &mut Vec<u8>, pair: &[f64; 2]) {
    for value in pair {
        bytes.extend(value.to_le_bytes());
    }
}

/// 追加嵌套对象条目头（16 字节类标识 + 4 字节键；回引与新对象同构）。
fn push_entry_header(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 版本 1 布局的合成 PCS 对象体（版本 short + 4 字节长度 + ASCII WKT）。
///
/// 嵌套分派与游标推进不依赖 PCS 体的完整版本 6 样本布局，此处用最小合法体；
/// 样本中实际嵌套的是版本 6 体（键 80 体尾内嵌键 81，455 字节 WKT，辅助
/// 观察，见模块文档）。
fn minimal_pcs_body(wkt: &str) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend((wkt.len() as u32).to_le_bytes());
    bytes.extend(wkt.as_bytes());
    bytes
}

/// 版本 3 + 包络 + int32 0 + 指定嵌套空间参考段的完整对象体，附尾部标记。
fn version_three_body(sr_segment: &[u8], tail: &[u8]) -> Vec<u8> {
    let mut bytes = envelope_prefix(3, &COORDS);
    push_attribute_flags(&mut bytes, 0);
    bytes.extend_from_slice(sr_segment);
    bytes.extend_from_slice(tail);
    bytes
}

#[test]
fn envelope_class_id_constant_matches_documented_guid() {
    // 证据：{30707212-52D5-11D0-A8F2-00608C85EDE5}，Geometry.dll 10.1.0.3035
    // GetClassID 0x71864170 写死（写方向取证轮，证据等级「确定的」）；内存
    // 序 = Data1-3 小端 + Data4 原序，与样本 /Maps:0xedd 起的对象头逐字节
    // 相同（辅助观察）。
    assert_eq!(
        ENVELOPE_CLASS_ID,
        [
            0x12, 0x72, 0x70, 0x30, 0xd5, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85,
            0xed, 0xe5,
        ]
    );
    // 与嵌套空间参考的 PCS 类标识不是同一个标识。
    assert_ne!(ENVELOPE_CLASS_ID, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
}

#[test]
fn reads_version_three_with_back_reference_to_registered_map_pcs() {
    // 样本最常见形态：键 30（Map 的 PCS）已在本会话登记，嵌套段是 20 字节
    // 回引头（16 字节类标识 + 4 字节既有键），其后没有对象体。
    let mut table = RefTable10_1::new();
    table.register(MAP_PCS_KEY);
    table.register(7); // 另一个无关键，用于核对登记表不被意外修改。
    let mut sr_segment = Vec::new();
    push_entry_header(
        &mut sr_segment,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        MAP_PCS_KEY,
    );
    let bytes = version_three_body(&sr_segment, b"tail");
    let body_length = bytes.len() - 4;
    // Envelope 本体是定宽读取，max_bytes = 0 也不约束本体（限额只转发给
    // 嵌套 PCS 体，此处没有 PCS 体）。
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 0, &mut table).unwrap();
    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.coords, COORDS);
    assert_eq!(parsed.attribute_flags, Some(0));
    assert_eq!(parsed.z_domain, None);
    assert_eq!(parsed.m_domain, None);
    assert_eq!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference {
            class_id: PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            key: MAP_PCS_KEY,
        })
    );
    // 回引不登记新键，登记表保持两个原有键。
    assert_eq!(table.len(), 2);
    assert!(table.contains(MAP_PCS_KEY));
    assert!(table.contains(7));
    assert!(!table.contains(MAP_PCS_KEY + 1));
    assert_eq!(reader.position(), body_length);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

#[test]
fn reads_version_three_with_embedded_new_pcs_and_registers_key() {
    // 新键 + PCS 类标识：嵌套读完整 PCS 体，键在体读取前登记。
    let mut table = RefTable10_1::new();
    let mut sr_segment = Vec::new();
    push_entry_header(
        &mut sr_segment,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        NESTED_PCS_KEY,
    );
    sr_segment.extend(minimal_pcs_body("PROJCS[\"x\"]"));
    let bytes = version_three_body(&sr_segment, b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap();
    let SpatialReferenceEntry10_1::Projected { key, body } =
        parsed.spatial_reference.as_ref().unwrap()
    else {
        panic!(
            "新键 + PCS 类标识应分类为内嵌 PCS：{:?}",
            parsed.spatial_reference
        );
    };
    assert_eq!(*key, NESTED_PCS_KEY);
    assert_eq!(body.save_version, 1);
    assert_eq!(body.wkt, "PROJCS[\"x\"]");
    // 登记随整体成功提交：后续同键条目将命中回引。
    assert!(table.contains(NESTED_PCS_KEY));
    assert_eq!(table.len(), 1);
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

#[test]
fn reads_version_three_null_marker_without_consuming_key() {
    // 16 字节全零类标识 = 空引用标记：只消耗 16 字节、不读键（S_FALSE 约定）。
    let mut table = RefTable10_1::new();
    let bytes = version_three_body(&[0_u8; 16], b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::Null)
    );
    assert!(table.is_empty());
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

#[test]
fn reads_version_one_and_two_without_attribute_or_reference_segments() {
    // 版本 1：只读 32 字节（无 int32、无空间参考段）；版本 2：再加 int32。
    // 其后的字节不属于本对象体，必须原样留在游标之后。
    let mut v1 = envelope_prefix(1, &COORDS);
    v1.extend(b"tail");
    let mut reader = Reader::new(&v1);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.coords, COORDS);
    assert_eq!(parsed.attribute_flags, None);
    assert_eq!(parsed.z_domain, None);
    assert_eq!(parsed.m_domain, None);
    assert_eq!(parsed.spatial_reference, None);
    assert_eq!(reader.position(), 34);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");

    let mut v2 = envelope_prefix(2, &COORDS);
    push_attribute_flags(&mut v2, 0);
    v2.extend(b"tail");
    let mut reader = Reader::new(&v2);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(parsed.version, 2);
    assert_eq!(parsed.attribute_flags, Some(0));
    assert_eq!(parsed.spatial_reference, None);
    assert_eq!(reader.position(), 38);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

#[test]
fn rejects_versions_outside_native_and_evidenced_branches() {
    // 版本 >3：原生经 FUN_719f1570 记 0x80004005，本读取器拒绝；0x8000..
    // 位模式按无符号解释同样大于 3（符号性未闭合，若原生为有符号比较则
    // 此处更严）。版本 0：证据只覆盖「版本 1」与「版本 2/3」两组分支，
    // 版本 0 的字段布局未闭合，显式拒绝（较原生「仅拒绝 >3」更严）。
    for version in [0_u16, 4, 5, 100, 0x1234, 0x8000, 0xffff] {
        let mut bytes = envelope_prefix(version, &COORDS);
        push_attribute_flags(&mut bytes, 0);
        bytes.extend([0_u8; 16]);
        let mut reader = Reader::new(&bytes);
        let first = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        let message = first.to_string();
        assert!(message.contains("版本"), "{message}");
        assert!(message.contains("offset"), "{message}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
        // 失败可重复：重读得到相同错误类别且游标仍在入口。
        let second = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
    // 仅 2 字节版本即足以触发拒绝，不依赖对象体其余部分。
    let bytes = 4_u16.to_le_bytes();
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn reads_z_and_m_appended_pairs_for_all_bit_combinations() {
    // int32 低字节 bit6(0x40) 追加 Z double 对、bit7(0x80) 再追加 M double
    // 对（Z 在 M 前）；低位其他位（0x3f/0x01）与高位字节（0x4000/
    // 0x80000000）都不触发追加。
    for (flags, has_z, has_m) in [
        (0x40_u32, true, false),
        (0x80, false, true),
        (0xc0, true, true),
        (0x41, true, false),
        (0xff, true, true),
        (0xbf, false, true),
        (0x3f, false, false),
        (0x4000, false, false),
        (0x8000_0000, false, false),
        (0x8000_0040, true, false),
    ] {
        let mut bytes = envelope_prefix(2, &COORDS);
        push_attribute_flags(&mut bytes, flags);
        if has_z {
            push_pair(&mut bytes, &Z_PAIR);
        }
        if has_m {
            push_pair(&mut bytes, &M_PAIR);
        }
        bytes.extend(b"tail");
        let mut reader = Reader::new(&bytes);
        let parsed = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_or_else(|error| panic!("flags={flags:#x}: {error}"));
        assert_eq!(parsed.version, 2, "flags={flags:#x}");
        assert_eq!(parsed.attribute_flags, Some(flags), "flags={flags:#x}");
        // Z 在 M 前：两段同时存在时各自取值不串位。
        assert_eq!(parsed.z_domain, has_z.then_some(Z_PAIR), "flags={flags:#x}");
        assert_eq!(parsed.m_domain, has_m.then_some(M_PAIR), "flags={flags:#x}");
        // 恰好读完追加段，不多读一个字节。
        assert_eq!(reader.position(), bytes.len() - 4, "flags={flags:#x}");
        assert_eq!(reader.read_bytes(4).unwrap(), b"tail", "flags={flags:#x}");
    }

    // 段顺序综合：版本 3 时 Z/M 段在嵌套空间参考条目之前；int32=0xc0 的
    // 完整布局 + 对已登记键 30 的回引。
    let mut table = RefTable10_1::new();
    table.register(MAP_PCS_KEY);
    let mut bytes = envelope_prefix(3, &COORDS);
    push_attribute_flags(&mut bytes, 0xc0);
    push_pair(&mut bytes, &Z_PAIR);
    push_pair(&mut bytes, &M_PAIR);
    push_entry_header(
        &mut bytes,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        MAP_PCS_KEY,
    );
    bytes.extend(b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.z_domain, Some(Z_PAIR));
    assert_eq!(parsed.m_domain, Some(M_PAIR));
    assert!(matches!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference {
            key: MAP_PCS_KEY,
            ..
        })
    ));
    assert_eq!(reader.position(), bytes.len() - 4);
}

#[test]
fn rejects_non_pcs_new_object_as_unsupported() {
    // 新键但类标识不是投影坐标系：嵌套对象体无外部长度字段，无法跳过未知
    // 类，显式拒绝且不推进游标。第一个 GUID 是样本中出现的候选
    // {34B2EF83-F4AC-11D1-A245-080009B6F22B}（FDOGraphicsLayer 候选，作为
    // 空间参考嵌套属于反例），第二个是纯合成标识。
    let sample_candidate = [
        0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6, 0xf2,
        0x2b,
    ];
    let synthetic = [
        0xd8, 0x1c, 0x6c, 0x2e, 0x31, 0xae, 0x44, 0xd8, 0x8e, 0x7a, 0x11, 0x0b, 0x2f, 0x93, 0x7d,
        0xe4,
    ];
    for class_id in [&sample_candidate, &synthetic] {
        assert_ne!(*class_id, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
        let mut table = RefTable10_1::new();
        let mut sr_segment = Vec::new();
        push_entry_header(&mut sr_segment, class_id, 31);
        let bytes = version_three_body(&sr_segment, b"opaque body");
        let mut reader = Reader::new(&bytes);
        let first = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap_err();
        assert_eq!(first.kind(), ErrorKind::Unsupported, "{class_id:?}");
        let message = first.to_string();
        assert!(message.contains("投影坐标系"), "{message}");
        assert!(message.contains("无法跳过"), "{message}");
        assert!(message.contains("键 31"), "{message}");
        assert!(message.contains("offset"), "{message}");
        assert_eq!(reader.position(), 0, "游标必须留在入口");
        // 试读登记不得泄漏：键 31 未进入调用者的登记表。
        assert!(!table.contains(31));
        assert!(table.is_empty());
        let second = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "重读结果不一致");
        assert_eq!(reader.position(), 0, "重读后游标仍在入口");
        assert!(table.is_empty(), "重读失败同样不得修改登记表");
    }
}

#[test]
fn back_reference_classification_depends_on_table_state() {
    // 回引与新对象在流内同构、不可局部区分（object_refs 保留的原生语义）：
    // 同一字节序列在已登记键的表下按回引用（无体），在空表下按新对象读体。
    let mut sr_segment = Vec::new();
    push_entry_header(
        &mut sr_segment,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        MAP_PCS_KEY,
    );
    sr_segment.extend(minimal_pcs_body("PROJCS[\"x\"]"));
    let bytes = version_three_body(&sr_segment, b"");

    let mut table = RefTable10_1::new();
    table.register(MAP_PCS_KEY);
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(matches!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference {
            key: MAP_PCS_KEY,
            ..
        })
    ));
    // 回引停在 20 字节头之后：同键的 PCS 体字节留在游标之后。
    let header_end = 2 + 32 + 4 + 20;
    assert_eq!(reader.position(), header_end);
    assert_eq!(table.len(), 1);

    let mut empty = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut empty).unwrap();
    assert!(matches!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::Projected {
            key: MAP_PCS_KEY,
            ..
        })
    ));
    assert_eq!(reader.position(), bytes.len());
    assert!(empty.contains(MAP_PCS_KEY));
}

#[test]
fn rejects_every_truncated_prefix_without_advancing_or_touching_table() {
    // 逐字节截断扫描覆盖每个字段内部：版本（2 字节）、32 字节包络的每个
    // double、int32、Z 段 double、嵌套对象头的 GUID 与键、PCS 体的版本/
    // 长度/WKT。任何截断都必须失败、不推进游标、不修改登记表，且重读得到
    // 相同错误类别。
    let mut sr_segment = Vec::new();
    push_entry_header(
        &mut sr_segment,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        NESTED_PCS_KEY,
    );
    sr_segment.extend(minimal_pcs_body("PROJCS[\"x\"]"));
    let mut full = envelope_prefix(3, &COORDS);
    push_attribute_flags(&mut full, 0x40);
    push_pair(&mut full, &Z_PAIR);
    full.extend_from_slice(&sr_segment);
    full.extend(b"tail");
    let body_end = full.len() - 4;

    // 边界情形：恰好在对象体末尾截断（尾部标记缺失）是合法完整输入。
    let mut boundary = Reader::new(&full[..body_end]);
    let parsed = read_envelope_10_1(&mut boundary, 1024, &mut RefTable10_1::new()).unwrap();
    assert!(matches!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::Projected {
            key: NESTED_PCS_KEY,
            ..
        })
    ));
    assert_eq!(boundary.remaining(), 0);

    for end in 0..body_end {
        let mut table = RefTable10_1::new();
        table.register(MAP_PCS_KEY);
        let mut reader = Reader::new(&full[..end]);
        let first = read_envelope_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_envelope_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        // 失败路径不得修改登记表：原有键保留，试读新键不泄漏。
        assert_eq!(table.len(), 1, "end={end}");
        assert!(table.contains(MAP_PCS_KEY), "end={end}");
        assert!(!table.contains(NESTED_PCS_KEY), "end={end}");
    }

    // 完整输入本身必须成功，尾部标记保留在游标之后。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&full);
    assert!(read_envelope_10_1(&mut reader, 1024, &mut table).is_ok());
    assert_eq!(reader.remaining(), 4);
    assert!(table.contains(NESTED_PCS_KEY));
}

#[test]
fn forwards_max_bytes_limit_to_the_nested_pcs_body() {
    // 调用者限额原样转发给嵌套 PCS 体（约束 WKT 长度）：等于 WKT 长度可读，
    // 少一字节即被 PCS 读取器以限额错误拒绝，游标与登记表都不提交。
    let wkt = "PROJCS[\"x\"]";
    let mut sr_segment = Vec::new();
    push_entry_header(
        &mut sr_segment,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        NESTED_PCS_KEY,
    );
    sr_segment.extend(minimal_pcs_body(wkt));
    let bytes = version_three_body(&sr_segment, b"");
    let mut reader = Reader::new(&bytes);
    assert!(read_envelope_10_1(&mut reader, wkt.len(), &mut RefTable10_1::new()).is_ok());
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error = read_envelope_10_1(&mut reader, wkt.len() - 1, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    let message = error.to_string();
    assert!(message.contains("限额"), "{message}");
    assert!(message.contains("WKT"), "{message}");
    assert_eq!(reader.position(), 0);
    // 嵌套体读取失败时，试读登记的新键一并回滚。
    assert!(!table.contains(NESTED_PCS_KEY));
    assert!(table.is_empty());
}

#[test]
fn preserves_coordinate_and_domain_double_bit_patterns() {
    // 4 个坐标与 Z/M double 仅原样保留小端位模式：NaN 载荷、次正规数、
    // -0.0 都按位保留，不断言数值语义，也不校验 d0<d2、d1<d3 之类次序。
    let coords = [
        f64::from_bits(0x7ff8_0000_1234_5678), // 带载荷的安静 NaN。
        f64::INFINITY,
        -0.0,
        5.0e-324, // 最小次正规数。
    ];
    let z_pair = [f64::NAN, f64::NEG_INFINITY];
    let m_pair = [-0.0, f64::from_bits(1)];
    let mut bytes = envelope_prefix(3, &coords);
    push_attribute_flags(&mut bytes, 0xc0);
    push_pair(&mut bytes, &z_pair);
    push_pair(&mut bytes, &m_pair);
    bytes.extend([0_u8; 16]);
    let mut reader = Reader::new(&bytes);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    for (read, expected) in parsed.coords.iter().zip(coords.iter()) {
        assert_eq!(read.to_bits(), expected.to_bits());
    }
    for (read, expected) in parsed
        .z_domain
        .unwrap()
        .iter()
        .zip(z_pair.iter())
        .chain(parsed.m_domain.unwrap().iter().zip(m_pair.iter()))
    {
        assert_eq!(read.to_bits(), expected.to_bits());
    }
    assert_eq!(parsed.attribute_flags, Some(0xc0));
    assert_eq!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::Null)
    );
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reads_envelope_header_then_body_in_full_chain() {
    // 组合链路（样本键 74 字节镜像，辅助观察）：read_header_10_1 确认
    // Envelope 头（类标识 + 键 74，@/Maps:0xedd）→ read_envelope_10_1 读体
    // （`03 00` + 样本 32 字节包络 + int32 0 + 对键 30 的回引头）→ 尾部留给
    // 调用者。坐标取自样本 0xef3 起的 32 字节原字节。
    let sample_coords = [
        f64::from_le_bytes([0xe7, 0xae, 0x25, 0x81, 0x5a, 0xd0, 0x81, 0x41]),
        f64::from_le_bytes([0x24, 0x0d, 0xe0, 0x4d, 0x18, 0x06, 0x43, 0x41]),
        f64::from_le_bytes([0xb3, 0x7b, 0xf2, 0x85, 0x52, 0xd8, 0x81, 0x41]),
        f64::from_le_bytes([0x8c, 0x73, 0x46, 0x54, 0x57, 0x76, 0x43, 0x41]),
    ];
    let mut bytes = ENVELOPE_CLASS_ID.to_vec();
    bytes.extend(74_u32.to_le_bytes());
    bytes.extend(envelope_prefix(3, &sample_coords));
    push_attribute_flags(&mut bytes, 0);
    push_entry_header(
        &mut bytes,
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        MAP_PCS_KEY,
    );
    bytes.extend(b"tail");
    let mut table = RefTable10_1::new();
    table.register(MAP_PCS_KEY);
    let mut reader = Reader::new(&bytes);
    let header = read_header_10_1(&mut reader).unwrap();
    let ObjectHeader::Object { class_id, key } = header else {
        panic!("Envelope 对象头不应为 Null：{header:?}");
    };
    assert_eq!(class_id, ENVELOPE_CLASS_ID);
    assert_eq!(key, 74);
    let parsed = read_envelope_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.coords, sample_coords);
    assert_eq!(parsed.attribute_flags, Some(0));
    assert!(matches!(
        parsed.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference {
            key: MAP_PCS_KEY,
            ..
        })
    ));
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");

    // 反例：顶部对象头是 PCS 标识时类身份不匹配 Envelope 常量，调用者须
    // 拒绝而不能进入 Envelope 体读取（类身份确认是调用者的职责）。
    let mut other = PROJECTED_COORDINATE_SYSTEM_CLASS_ID.to_vec();
    other.extend(MAP_PCS_KEY.to_le_bytes());
    let mut reader = Reader::new(&other);
    match read_header_10_1(&mut reader).unwrap() {
        ObjectHeader::Object { class_id, key } => {
            assert_ne!(class_id, ENVELOPE_CLASS_ID);
            assert_eq!(key, MAP_PCS_KEY);
        }
        other_header => panic!("PCS 对象头不应为 Null：{other_header:?}"),
    }
}
