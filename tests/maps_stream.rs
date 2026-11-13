use mxd2qgz::maps_stream::{
    MapsRootPrefix10_1, MapsRootPrefix10_8, read_root_prefix_10_1, read_root_prefix_10_8,
};
use mxd2qgz::object_stream::ObjectHeader;
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

fn encoded_root(count: i32, class_byte: u8) -> Vec<u8> {
    let mut bytes = count.to_le_bytes().to_vec();
    bytes.extend([class_byte; 16]);
    bytes.extend(7_u32.to_le_bytes());
    bytes.extend(b"opaque");
    bytes
}

#[test]
fn reads_positive_count_and_first_object_header_only() {
    let bytes = encoded_root(1, 0xa5);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_root_prefix_10_8(&mut reader, 16).unwrap(),
        MapsRootPrefix10_8 {
            count: 1,
            first_header: Some(ObjectHeader::Object {
                class_id: [0xa5; 16],
                key: 7,
            }),
            remaining_bytes: 6,
        }
    );
    assert_eq!(reader.position(), 24);
    assert_eq!(reader.read_bytes(6).unwrap(), b"opaque");
}

#[test]
fn zero_count_stops_before_the_object_entry() {
    let mut bytes = 0_i32.to_le_bytes().to_vec();
    bytes.extend(b"unparsed-root-tail");
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_root_prefix_10_8(&mut reader, 16).unwrap(),
        MapsRootPrefix10_8 {
            count: 0,
            first_header: None,
            remaining_bytes: 18,
        }
    );
    assert_eq!(reader.position(), 4);
    assert_eq!(reader.read_bytes(18).unwrap(), b"unparsed-root-tail");
}

#[test]
fn null_first_header_does_not_consume_a_following_field() {
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.extend([0; 16]);
    bytes.extend(0xffff_ffff_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let parsed = read_root_prefix_10_8(&mut reader, 16).unwrap();
    assert_eq!(parsed.first_header, Some(ObjectHeader::Null));
    assert_eq!(reader.position(), 20);
    assert_eq!(reader.read_u32_le().unwrap(), u32::MAX);
}

#[test]
fn rejects_negative_count_and_resource_exhaustion_at_the_start() {
    for count in [-1_i32, i32::MIN] {
        let bytes = count.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        let error = read_root_prefix_10_8(&mut reader, 16).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("数量为负数"));
        assert_eq!(reader.position(), 0);
    }
    let bytes = encoded_root(2, 1);
    let mut reader = Reader::new(&bytes);
    let error = read_root_prefix_10_8(&mut reader, 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("数量超过限额"));
    assert_eq!(reader.position(), 0);
}

#[test]
fn count_or_header_truncation_preserves_the_root_cursor() {
    let bytes = encoded_root(1, 0xa5);
    for available in 0..24 {
        let mut reader = Reader::new(&bytes[..available]);
        let error = read_root_prefix_10_8(&mut reader, 16).unwrap_err();
        assert_eq!(
            error.kind(),
            ErrorKind::UnexpectedEof,
            "available={available}"
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.remaining(), available);
    }
}

#[test]
fn nonzero_start_is_local_to_the_supplied_range() {
    let mut bytes = [0xff].to_vec();
    bytes.extend(encoded_root(1, 0x5a));
    let mut parent = Reader::new(&bytes);
    parent.read_bytes(1).unwrap();
    let parsed = read_root_prefix_10_8(&mut parent, 16).unwrap();
    assert_eq!(parsed.count, 1);
    assert_eq!(
        parsed.first_header,
        Some(ObjectHeader::Object {
            class_id: [0x5a; 16],
            key: 7,
        })
    );
    assert_eq!(parent.position(), bytes.len() - 6);
}

// 以下为显式 10.1 分流 `/Maps` 前缀读取器的用例；全部使用合成字节构造，
// 仓库参考样本不作为测试依赖。

/// 构造 10.1 `/Maps` 前缀的正常合成输入：数量 + 首对象头（仅正数量）+
/// 4 字节尾部值，与写侧顺序（ArcMap `0x466fb0`）一致。
fn encoded_10_1_root(count: i32, class_byte: u8, tail: u32) -> Vec<u8> {
    let mut bytes = count.to_le_bytes().to_vec();
    if count > 0 {
        bytes.extend([class_byte; 16]);
        bytes.extend(7_u32.to_le_bytes());
    }
    bytes.extend(tail.to_le_bytes());
    bytes
}

#[test]
fn reads_10_1_count_first_header_and_raw_tail() {
    // 数量 1 + 首对象头 + 尾部 0/1：尾部是读侧循环结束后的 4 字节值，
    // 0/1 语义（写侧容量预估返回 -1 与否）属证据性解读，这里只验证原始位
    // 模式被保留。
    for tail in [0_u32, 1] {
        let bytes = encoded_10_1_root(1, 0xa5, tail);
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_root_prefix_10_1(&mut reader, 16).unwrap(),
            MapsRootPrefix10_1 {
                count: 1,
                first_header: Some(ObjectHeader::Object {
                    class_id: [0xa5; 16],
                    key: 7,
                }),
                tail_raw: tail,
                remaining_bytes: 0,
            }
        );
        assert_eq!(reader.position(), bytes.len());
    }
    // 尾部取任意其他位模式同样保留；尾部之后的未知字节原样留给调用者。
    let mut bytes = encoded_10_1_root(1, 0xa5, 0xdead_beef);
    bytes.extend(b"rest");
    let mut reader = Reader::new(&bytes);
    let parsed = read_root_prefix_10_1(&mut reader, 16).unwrap();
    assert_eq!(parsed.tail_raw, 0xdead_beef);
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"rest");
}

#[test]
fn zero_10_1_count_skips_the_header_but_still_reads_the_tail() {
    // 读侧在对象循环结束后无条件再读 4 字节尾部值，数量 0（空循环）也一样；
    // 尾部之后的未知字节保留，不解释。
    let bytes = encoded_10_1_root(0, 0, 1);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_root_prefix_10_1(&mut reader, 16).unwrap(),
        MapsRootPrefix10_1 {
            count: 0,
            first_header: None,
            tail_raw: 1,
            remaining_bytes: 0,
        }
    );
    assert_eq!(reader.position(), 8);

    let mut bytes = encoded_10_1_root(0, 0, 0);
    bytes.extend(b"unparsed-root-tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_root_prefix_10_1(&mut reader, 16).unwrap();
    assert_eq!(
        parsed,
        MapsRootPrefix10_1 {
            count: 0,
            first_header: None,
            tail_raw: 0,
            remaining_bytes: 18,
        }
    );
    assert_eq!(reader.position(), 8);
    assert_eq!(reader.read_bytes(18).unwrap(), b"unparsed-root-tail");
}

#[test]
fn rejects_10_1_negative_and_abnormal_counts_without_advancing() {
    // 负数量在原生分支中不进入循环，但语义未知，显式拒绝并在错误信息中
    // 说明原生仅正数进入对象循环。
    for count in [-1_i32, i32::MIN] {
        let bytes = count.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        let error = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("数量为负数"), "{error}");
        assert!(error.to_string().contains("仅正数进入对象循环"), "{error}");
        assert_eq!(reader.position(), 0);
    }
    // 超过限额的异常大值同样被拒且不推进游标。
    let bytes = encoded_10_1_root(2, 1, 0);
    let mut reader = Reader::new(&bytes);
    let error = read_root_prefix_10_1(&mut reader, 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("数量超过限额"), "{error}");
    assert_eq!(reader.position(), 0);
}

#[test]
fn huge_10_1_count_short_reads_without_allocation_or_overflow() {
    // i32::MAX 数量在最大限额之内也在首对象头处立即短读，不做任何分配。
    let count_only = i32::MAX.to_le_bytes();
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, i32::MAX as u32).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    // 限额参数 u32::MAX 无法由 i32 表示，在读取任何对象前被拒。
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, u32::MAX).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
}

#[test]
fn every_10_1_truncation_preserves_the_cursor_and_rereads_identically() {
    // 数量 0-3 字节、首对象头每字节、尾部 0-3 字节的每种前缀截断：失败
    // 不推进游标，同一失败输入重读得到相同错误类别且游标仍在入口。
    let bytes = encoded_10_1_root(1, 0xa5, 1);
    assert_eq!(bytes.len(), 4 + 20 + 4);
    for available in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..available]);
        let first = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
        assert_eq!(
            first.kind(),
            ErrorKind::UnexpectedEof,
            "available={available}"
        );
        assert_eq!(reader.position(), 0, "available={available}");
        assert_eq!(reader.remaining(), available);
        let second = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "available={available}");
        assert_eq!(reader.position(), 0, "available={available}");
    }
    // 数量 0 的输入同样扫描：截断只会落在数量或尾部字节上。
    let empty_root = encoded_10_1_root(0, 0, 1);
    assert_eq!(empty_root.len(), 8);
    for available in 0..empty_root.len() {
        let mut reader = Reader::new(&empty_root[..available]);
        let first = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
        assert_eq!(
            first.kind(),
            ErrorKind::UnexpectedEof,
            "available={available}"
        );
        assert_eq!(reader.position(), 0, "available={available}");
        let second = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "available={available}");
        assert_eq!(reader.position(), 0, "available={available}");
    }
}

#[test]
fn preserves_unknown_10_1_class_id_key_and_tail_bits_verbatim() {
    // 未知类 GUID、键与尾部值的全部位模式原样保留；读取器不校验类注册，
    // 也不把尾部值限制在证据性的 0/1 之上。
    let class_id = [
        0xd8, 0x1c, 0x6c, 0x2e, 0x31, 0xae, 0x44, 0xd8, 0x8e, 0x7a, 0x11, 0x0b, 0x2f, 0x93, 0x7d,
        0xe4,
    ];
    for key in [0_u32, 1, 0x1234_5678, u32::MAX] {
        for tail in [0_u32, 1, 0x8000_0000, u32::MAX] {
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend(class_id);
            bytes.extend(key.to_le_bytes());
            bytes.extend(tail.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            assert_eq!(
                read_root_prefix_10_1(&mut reader, 16).unwrap(),
                MapsRootPrefix10_1 {
                    count: 1,
                    first_header: Some(ObjectHeader::Object { class_id, key }),
                    tail_raw: tail,
                    remaining_bytes: 0,
                }
            );
            assert_eq!(reader.position(), bytes.len());
        }
    }
    // 全零 GUID 是空头：只消耗 16 字节，不吞并紧随的尾部值。
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.extend([0; 16]);
    bytes.extend(0xffff_ffff_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let parsed = read_root_prefix_10_1(&mut reader, 16).unwrap();
    assert_eq!(parsed.first_header, Some(ObjectHeader::Null));
    assert_eq!(parsed.tail_raw, u32::MAX);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn nonzero_10_1_start_is_local_to_the_supplied_range() {
    let mut bytes = [0xff].to_vec();
    bytes.extend(encoded_10_1_root(1, 0x5a, 1));
    let mut parent = Reader::new(&bytes);
    parent.read_bytes(1).unwrap();
    let parsed = read_root_prefix_10_1(&mut parent, 16).unwrap();
    assert_eq!(parsed.count, 1);
    assert_eq!(
        parsed.first_header,
        Some(ObjectHeader::Object {
            class_id: [0x5a; 16],
            key: 7,
        })
    );
    assert_eq!(parsed.tail_raw, 1);
    assert_eq!(parent.position(), bytes.len());
}
