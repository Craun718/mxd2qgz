use mxd2qgz::maps::map_object::read_post_masking_map_fields_10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

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

#[test]
fn reads_is_framed_then_versioned_member_string() {
    // 人工构造的连续字段，不来自仓库 MXD：IsFramed short 经 Map 子接口 +0x60，
    // 后续 String 写入 Map 成员 +0x398。
    let mut bytes = 0x1234_u16.to_le_bytes().to_vec();
    let string_offset = bytes.len();
    bytes.extend(encoded("tail-field"));
    let end = bytes.len();
    let mut reader = Reader::new(&bytes);

    let fields = read_post_masking_map_fields_10_1(&mut reader, 30, 1024).unwrap();
    assert_eq!(fields.is_framed_bits, (0, 0x1234));
    assert_eq!(
        fields.string_398,
        Some((string_offset, "tail-field".to_string()))
    );
    assert_eq!(reader.position(), end);
}

#[test]
fn version_29_reads_only_is_framed() {
    let mut bytes = 0xabcd_u16.to_le_bytes().to_vec();
    bytes.extend(b"unparsed");
    let mut reader = Reader::new(&bytes);

    let fields = read_post_masking_map_fields_10_1(&mut reader, 29, 0).unwrap();
    assert_eq!(fields.is_framed_bits, (0, 0xabcd));
    assert_eq!(fields.string_398, None);
    assert_eq!(reader.position(), 2);
}

#[test]
fn rejects_versions_before_the_virtual_setter_branch_without_consuming() {
    for version in [28_u16, 52] {
        let mut reader = Reader::new(&[0x34, 0x12]);
        let error = read_post_masking_map_fields_10_1(&mut reader, version, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn truncation_or_invalid_string_rolls_back_the_complete_field_group() {
    let complete = encoded("完整");
    for length in 0..complete.len() {
        // 前两个 short 字节之后逐段截断字符串；短读不能提交 short 值。
        let mut bytes = 0x5678_u16.to_le_bytes().to_vec();
        bytes.extend_from_slice(&complete[..length]);
        let mut reader = Reader::new(&bytes);
        assert!(read_post_masking_map_fields_10_1(&mut reader, 30, 1024).is_err());
        assert_eq!(reader.position(), 0, "length={length}");
    }

    let mut invalid = 0x5678_u16.to_le_bytes().to_vec();
    invalid.extend(2_u32.to_le_bytes());
    invalid.extend([0x41, 0x00]); // 非终止零 UTF-16LE
    let mut reader = Reader::new(&invalid);
    assert!(read_post_masking_map_fields_10_1(&mut reader, 30, 1024).is_err());
    assert_eq!(reader.position(), 0);
}

#[test]
fn post_masking_short_and_byte_follow_their_version_gates() {
    let mut bytes = 0xffff_u16.to_le_bytes().to_vec();
    bytes.extend(encoded(""));
    let short_offset = bytes.len();
    bytes.extend(0x8765_u16.to_le_bytes());
    let byte_offset = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let fields = read_post_masking_map_fields_10_1(&mut reader, 31, 64).unwrap();
    assert_eq!(fields.short_3a0, Some((short_offset, 0x8765)));
    assert_eq!(fields.byte_15d, None);
    assert_eq!(reader.position(), byte_offset);

    let mut reader = Reader::new(&bytes);
    let fields = read_post_masking_map_fields_10_1(&mut reader, 32, 64).unwrap();
    assert_eq!(fields.byte_15d, Some((byte_offset, 0xa5)));
    assert_eq!(reader.position(), bytes.len());
    let mut truncated = Reader::new(&bytes[..byte_offset]);
    assert!(read_post_masking_map_fields_10_1(&mut truncated, 32, 64).is_err());
    assert_eq!(truncated.position(), 0);
}
