//! 10.1 PropertySet 对象体与 Variant 值读取器的单元测试。
//!
//! 仅验证读取器自身契约（字段序、版本边界、限额、游标事务性、VARTYPE 分
//! 支）；样本字节对位以研究文档的辅助观察记录为准，不作为测试依赖。通过
//! 不代表 MXD 格式规则已验证，未经目标软件人工检查。

use mxd2qgz::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PROPERTY_SET_PROPERTY_CLASS_ID, PropertyValue10_1,
    PropertyValueObjectStream10_1, read_property_set_10_1, read_property_set_object_stream_10_1,
    read_variant_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use mxd2qgz::stream::variant_array::ClassVariantArray10_1;
use std::io::ErrorKind;

/// 构造 AfCore String::Write 布局的合法输入：u32 字节长度前缀（含终止零）+
/// UTF-16LE 载荷。
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

/// 构造 Variant 值：u16 VARTYPE + 载荷。
fn value_bytes(value_type: u16, payload: &[u8]) -> Vec<u8> {
    value_type
        .to_le_bytes()
        .into_iter()
        .chain(payload.iter().copied())
        .collect()
}

fn property_set_with_object(key: u32, body: &[u8]) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(encoded("nested"));
    bytes.extend(value_bytes(13, &[]));
    // ObjectStream 非空对象头：Variant 已写 VARTYPE，后随 GUID 和共享键。
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(key.to_le_bytes());
    bytes.extend(body);
    bytes
}

/// 样本实例 1（/Maps `0x147f9`）的体形态：版本 1、计数 1、键 "DATABASE"、
/// VT_BSTR 路径值（样本 cb 0x2a；测试用同构合成值）。返回 (体, 体+宿主尾部)：
/// 尾部为外层 WorkspaceName 的工厂标志 `01` + 工厂 CLSID + int32 0，供「体恰
/// 在宿主下一字段终结」的边界断言。
fn sample_body_with_host_tail() -> (Vec<u8>, usize) {
    let mut body = 1_u16.to_le_bytes().to_vec();
    body.extend(1_i32.to_le_bytes());
    body.extend(encoded("DATABASE"));
    body.extend(value_bytes(8, &encoded("D:\\目录\\数据")));
    let body_end = body.len();
    // 外层宿主（WorkspaceName）的后继字段：工厂标志 + 工厂 CLSID + int32。
    body.push(1);
    let factory: [u8; 16] = PROPERTY_SET_PROPERTY_CLASS_ID; // 任意 16 字节即可
    body.extend(factory);
    body.extend(0_i32.to_le_bytes());
    (body, body_end)
}

#[test]
fn constants_match_esri_clsid_and_memory_order() {
    // {588E5A11-D09B-11D1-AA7C-00C04FA33A15} 内存序：Data1-3 小端、Data4 原序。
    assert_eq!(
        PROPERTY_SET_CLASS_ID,
        [
            0x11, 0x5a, 0x8e, 0x58, 0x9b, 0xd0, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3,
            0x3a, 0x15
        ]
    );
    // {5B6D2B14-F64A-4457-84E0-617AE2130FF9}。
    assert_eq!(
        PROPERTY_SET_PROPERTY_CLASS_ID,
        [
            0x14, 0x2b, 0x6d, 0x5b, 0x4a, 0xf6, 0x57, 0x44, 0x84, 0xe0, 0x61, 0x7a, 0xe2, 0x13,
            0x0f, 0xf9
        ]
    );
}

#[test]
fn reads_sample_shaped_body_and_stops_at_host_tail() {
    let bytes = sample_body_with_host_tail().0;
    let mut reader = Reader::new(&bytes);
    let body = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.count, 1);
    assert_eq!(body.entries.len(), 1);
    assert_eq!(body.entries[0].key, "DATABASE");
    assert_eq!(
        body.entries[0].value,
        PropertyValue10_1::String("D:\\目录\\数据".to_string())
    );
    // 体终结于外层宿主的工厂标志字节：剩余 = 1 + 16 + 4。
    assert_eq!(body.remaining_bytes, 21);
    assert_eq!(reader.position(), bytes.len() - 21);
}

#[test]
fn reads_empty_set_and_multiple_entries() {
    // 计数 0：体恰 6 字节，无条目（样本计数 0 的原生语义 = 空集合成功）。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let body = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(body.count, 0);
    assert!(body.entries.is_empty());
    assert_eq!(body.remaining_bytes, 0);

    // 计数 2：键名与值类型各异，全部按序消费。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(encoded("A"));
    bytes.extend(value_bytes(3, &123_i32.to_le_bytes()));
    bytes.extend(encoded(""));
    bytes.extend(value_bytes(0, &[]));
    let mut reader = Reader::new(&bytes);
    let body = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(body.entries.len(), 2);
    assert_eq!(body.entries[0].key, "A");
    assert_eq!(
        body.entries[0].value,
        PropertyValue10_1::Raw4(123_i32.to_le_bytes())
    );
    assert_eq!(body.entries[1].key, "");
    assert_eq!(body.entries[1].value, PropertyValue10_1::Empty);
    assert_eq!(body.remaining_bytes, 0);
}

#[test]
fn nonempty_variant_reads_nested_property_set_body_and_registers_shared_key() {
    let nested_body = [
        1_u16.to_le_bytes().as_slice(),
        0_i32.to_le_bytes().as_slice(),
    ]
    .concat();
    let bytes = property_set_with_object(41, &nested_body);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let value = read_property_set_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap();

    let PropertyValueObjectStream10_1::Object {
        class_id,
        key,
        body,
    } = &value.entries[0].value
    else {
        panic!("应按已证 PropertySet CLSID 分派新对象体");
    };
    assert_eq!(*class_id, PROPERTY_SET_CLASS_ID);
    assert_eq!(*key, 41);
    assert_eq!(body.version, 1);
    assert_eq!(body.count, 0);
    assert!(refs.contains(41));
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn ordinary_property_set_entry_does_not_assume_object_stream_key() {
    let bytes = property_set_with_object(41, &[1, 0, 0, 0, 0, 0]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_property_set_10_1(&mut reader, 1024, &mut refs).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn property_set_variants_share_refs_across_separate_stream_readers() {
    let nested_body = [
        1_u16.to_le_bytes().as_slice(),
        0_i32.to_le_bytes().as_slice(),
    ]
    .concat();
    let first_bytes = property_set_with_object(77, &nested_body);
    let mut refs = RefTable10_1::new();
    let mut first_reader = Reader::new(&first_bytes);
    let first = read_property_set_object_stream_10_1(&mut first_reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        first.entries[0].value,
        PropertyValueObjectStream10_1::Object { key: 77, .. }
    ));

    // 新流只带同一 GUID+键；命中会话表后，不跟随 PropertySet 对象体。
    let second_bytes = property_set_with_object(77, &[]);
    let mut second_reader = Reader::new(&second_bytes);
    let second = read_property_set_object_stream_10_1(&mut second_reader, 1024, &mut refs).unwrap();
    assert_eq!(
        second.entries[0].value,
        PropertyValueObjectStream10_1::BackReference {
            class_id: PROPERTY_SET_CLASS_ID,
            key: 77,
        }
    );
    assert_eq!(second_reader.position(), second_bytes.len());
    assert_eq!(refs.len(), 1);
}

#[test]
fn deepest_back_reference_is_read_even_at_nested_object_limit() {
    let mut deepest = 1_u16.to_le_bytes().to_vec();
    deepest.extend(1_i32.to_le_bytes());
    deepest.extend(encoded("ancestor"));
    deepest.extend(13_u16.to_le_bytes());
    deepest.extend(PROPERTY_SET_CLASS_ID);
    deepest.extend(1_u32.to_le_bytes());

    let mut bytes = deepest;
    for key in (1..=64).rev() {
        bytes = property_set_with_object(key, &bytes);
    }
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let body = read_property_set_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap();

    assert!(matches!(
        body.entries[0].value,
        PropertyValueObjectStream10_1::Object { key: 1, .. }
    ));
    assert_eq!(refs.len(), 64);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn unsupported_new_object_rolls_back_reader_and_shared_refs() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(encoded("unknown"));
    bytes.extend(13_u16.to_le_bytes());
    bytes.extend([0xA5; 16]);
    bytes.extend(5_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_property_set_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("无已证对象体分派"));
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn version_boundaries_follow_signed_native_compare() {
    // 版本 0 与 1 接受（原生有符号 ≤1；Load 无版本分支）。
    for version in [0_u16, 1] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(0_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let body = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(body.version, version);
    }
    // 版本 2 拒绝（原生 0x80004005）；负值位模式一并拒绝（较原生更严）。
    for version in [2_u16, 0x8000_u16, 0xffff] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let error =
            read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn negative_count_is_rejected_but_zero_succeeds() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend((-1_i32).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn count_times_min_entry_is_prechecked_against_limit() {
    // 计数 3 × 最小条目 6 字节 = 18 > 限额 17：在读取任何条目之前拒绝。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes());
    // 补足 17 字节使「声明成立但限额不足」与截断区分开。
    bytes.extend(vec![0xa5; 15]);
    let mut reader = Reader::new(&bytes);
    let error = read_property_set_10_1(&mut reader, 17, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 限额 18 时进入条目循环，键名 cb（0xa5a5a5a5）超过限额/非法 →
    // InvalidData（字符串限额在条目内拒绝）。
    let mut reader = Reader::new(&bytes);
    let error = read_property_set_10_1(&mut reader, 18, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // usize::MAX 限额与 i32::MAX 计数：乘法不溢出，预检按超限拒绝。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(i32::MAX.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error =
        read_property_set_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn failures_are_transactional_and_repeatable() {
    let (bytes, body_end) = sample_body_with_host_tail();
    // 体完整长度即截断边界：0..body_end 全前缀截断必须失败、游标不推进、重
    // 读一致。
    for end in 0..body_end {
        let mut reader = Reader::new(&bytes[..end]);
        let first = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new());
        assert!(first.is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new());
        assert_eq!(
            first.unwrap_err().kind(),
            second.unwrap_err().kind(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
    }
    // end=body_end：体完整消费、无剩余。
    let mut reader = Reader::new(&bytes[..body_end]);
    let body = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(body.remaining_bytes, 0);
    // 合法输入在失败限额下失败后，同 Reader 以正确限额重试成功。
    let mut reader = Reader::new(&bytes);
    assert!(
        read_property_set_10_1(&mut reader, 8, &mut RefTable10_1::new()).is_err(),
        "限额 8 小于体长，应拒绝"
    );
    assert_eq!(reader.position(), 0);
    read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(reader.position(), bytes.len() - 21);
}

#[test]
fn variant_branches_cover_confirmed_vartypes_and_keep_bit_patterns() {
    let cases: [(u16, &[u8], PropertyValue10_1); 12] = [
        (0, &[], PropertyValue10_1::Empty),
        (1, &[], PropertyValue10_1::Null),
        (2, &[0x34, 0x12], PropertyValue10_1::Raw2([0x34, 0x12])),
        (11, &[0xff, 0xff], PropertyValue10_1::Raw2([0xff, 0xff])),
        (
            3,
            &[0x78, 0x56, 0x34, 0x12],
            PropertyValue10_1::Raw4([0x78, 0x56, 0x34, 0x12]),
        ),
        (
            10,
            &[0, 0, 0, 0x80],
            PropertyValue10_1::Raw4([0, 0, 0, 0x80]),
        ),
        (4, &[0xa5; 4], PropertyValue10_1::Raw4([0xa5; 4])),
        (5, &[0xa5; 8], PropertyValue10_1::Raw8([0xa5; 8])),
        (7, &[0xa5; 8], PropertyValue10_1::Raw8([0xa5; 8])),
        (6, &[0xa5; 8], PropertyValue10_1::Raw8([0xa5; 8])),
        (0x10, &[0x9c], PropertyValue10_1::Raw1([0x9c])),
        (
            0x13,
            &[0x01, 0x02, 0x03, 0x04],
            PropertyValue10_1::Ui4([1, 2, 3, 4]),
        ),
    ];
    for (value_type, payload, expected) in cases {
        let mut bytes = value_bytes(value_type, payload);
        bytes.extend_from_slice(b"TAIL");
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_variant_10_1(&mut reader, 1024).unwrap(),
            expected,
            "VT={value_type}"
        );
        assert_eq!(reader.read_bytes(4).unwrap(), b"TAIL", "VT={value_type}");
        assert_eq!(reader.remaining(), 0);
    }
    // VT_UI1 / VT_UI2 补充分支。
    let mut bytes = value_bytes(0x11, &[0x7f]);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_variant_10_1(&mut reader, 1024).unwrap(),
        PropertyValue10_1::Raw1([0x7f])
    );
    bytes = value_bytes(0x12, &[0x34, 0x12]);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_variant_10_1(&mut reader, 1024).unwrap(),
        PropertyValue10_1::Ui2([0x34, 0x12])
    );
}

#[test]
fn variant_string_uses_afcore_layout_not_readstring() {
    // AfCore String::Read 布局：cb 含终止零。n*2+2（VariantStream ReadString）
    // 布局会把这个输入当作 2 码元读 6 字节载荷——这里必须按 4 字节 cb + 4 字
    // 节载荷消费，并保留尾部。
    let mut bytes = value_bytes(8, &encoded("地图"));
    bytes.extend_from_slice(b"TAIL");
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_variant_10_1(&mut reader, 1024).unwrap(),
        PropertyValue10_1::String("地图".to_string())
    );
    assert_eq!(reader.read_bytes(4).unwrap(), b"TAIL");
    // 空字符串：VT(2) + cb=2（仅终止零；cb=0 的原生行为未验证，本读取器经
    // read_string_10_1 拒绝 cb<2）。
    let mut empty_value = value_bytes(8, &2_u32.to_le_bytes());
    empty_value.extend([0, 0]);
    let mut reader = Reader::new(&empty_value);
    assert_eq!(
        read_variant_10_1(&mut reader, 1024).unwrap(),
        PropertyValue10_1::String(String::new())
    );
    // 截断与终止零缺失：失败且游标不推进。
    let bytes = value_bytes(8, &encoded("x"));
    for end in 2..bytes.len() - 2 {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_variant_10_1(&mut reader, 1024).is_err(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
}

#[test]
fn variant_object_branch_null_guid_consumes_and_nonzero_rejects() {
    // 9/13 + 全零类 GUID：空对象子集，恰消费 2+16 字节并映射 NullObject。
    for value_type in [9_u16, 13] {
        let mut bytes = value_bytes(value_type, &[0u8; 16]);
        bytes.extend_from_slice(b"TAIL");
        let mut reader = Reader::new(&bytes);
        let value = read_variant_10_1(&mut reader, 1024).unwrap();
        assert_eq!(value, PropertyValue10_1::NullObject, "VT={value_type}");
        assert_eq!(reader.position(), 18);
        assert_eq!(reader.remaining(), b"TAIL".len());
    }
    // 9/13 + 非零类 GUID：AfCreateInstance 分支边界未闭合，显式拒绝且
    // 游标不动、失败可重复。
    for value_type in [9_u16, 13] {
        let mut bytes = value_bytes(
            value_type,
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        );
        bytes.extend_from_slice(b"TAIL");
        let mut reader = Reader::new(&bytes);
        let error = read_variant_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported, "VT={value_type}");
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.remaining(), bytes.len());
        assert!(read_variant_10_1(&mut reader, 1024).is_err());
        assert_eq!(reader.position(), 0);
    }
    // 数组标志（0x2000）下无已证布局的元素仍拒绝。
    for base in [0_u16, 1, 9, 13] {
        let bytes = value_bytes(base | 0x2000, &[1, 2, 3, 4, 5, 6, 7, 8]);
        let mut reader = Reader::new(&bytes);
        let error = read_variant_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported, "VT={base:#x}");
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn string_variant_array_checks_aggregate_budget_and_preserves_tail() {
    let mut payload = 3_i32.to_le_bytes().to_vec();
    payload.extend(0_u32.to_le_bytes()); // 空元素只有长度字段。
    payload.extend(1_u32.to_le_bytes());
    payload.extend(('甲' as u16).to_le_bytes());
    payload.extend(0_u16.to_le_bytes());
    payload.extend(0_u32.to_le_bytes());
    let mut bytes = value_bytes(0x2008, &payload);
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        assert!(read_variant_10_1(&mut reader, 16).is_err());
        assert_eq!(reader.position(), 0);
    }
    bytes.push(0x7b);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_variant_10_1(&mut reader, 15).unwrap_err().kind(),
        ErrorKind::InvalidData,
    );
    assert_eq!(reader.position(), 0);
    assert_eq!(
        read_variant_10_1(&mut reader, 16).unwrap(),
        PropertyValue10_1::Array(ClassVariantArray10_1::Strings(vec![
            String::new(),
            "甲".into(),
            String::new(),
        ])),
    );
    assert_eq!(reader.read_bytes(1).unwrap(), [0x7b]);

    for count in [-1_i32, i32::MAX] {
        let bytes = value_bytes(0x2013, &count.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_variant_10_1(&mut reader, 128).unwrap_err().kind(),
            ErrorKind::InvalidData,
        );
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn variant_unknown_vartypes_reject_like_native_default() {
    // 0x2000 之外整个 u16 空间中未确认的分支：0x14..0x1f、0x16、0x1a 等。
    for value_type in [12_u16, 14, 15, 0x14, 0x1f, 0x40, 0x4f] {
        let bytes = value_bytes(value_type, &[0xa5; 8]);
        let mut reader = Reader::new(&bytes);
        let error = read_variant_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "VT={value_type}");
        assert_eq!(reader.position(), 0);
    }
    // 每个定宽分支的截断长度都在声明载荷完成前失败。
    for (value_type, complete) in [(2_u16, 2), (3, 4), (5, 8), (0x10, 1), (0x12, 2), (0x13, 4)] {
        for length in 0..complete {
            let bytes = value_bytes(value_type, &vec![0xa5; length]);
            let mut reader = Reader::new(&bytes);
            assert!(read_variant_10_1(&mut reader, 1024).is_err());
            assert_eq!(reader.position(), 0);
        }
    }
}

#[test]
fn variant_string_limit_rejects_without_allocating() {
    // 声明码元/字节长度超过限额：在分配前拒绝。
    let huge = (u32::MAX - 1).to_le_bytes();
    let mut bytes = vec![8, 0];
    bytes.extend(huge);
    let mut reader = Reader::new(&bytes);
    let error = read_variant_10_1(&mut reader, 16).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 奇数声明长度（非法 UTF-16 字节数）被 String 读取器拒绝。
    let mut bytes = vec![8, 0];
    bytes.extend(3_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error = read_variant_10_1(&mut reader, usize::MAX).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn reader_registration_table_is_untouched() {
    // 体内没有 ObjectStream 编码条目：登记表不被读取也不被修改。
    let (bytes, _) = sample_body_with_host_tail();
    let mut table = RefTable10_1::new();
    table.register(7);
    let mut reader = Reader::new(&bytes);
    read_property_set_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(table.len(), 1);
    assert!(table.contains(7));
    // 失败路径同样不改登记表。
    let mut table = RefTable10_1::new();
    table.register(7);
    let mut reader = Reader::new(&bytes[..5]);
    assert!(read_property_set_10_1(&mut reader, 1024, &mut table).is_err());
    assert_eq!(table.len(), 1);
}

#[test]
fn io_errors_propagate_as_unexpected_eof() {
    // 空输入：版本 short 短读。
    let mut reader = Reader::new(&[]);
    let error = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
}
