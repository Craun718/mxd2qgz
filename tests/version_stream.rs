use mxd2qgz::reader::Reader;
use mxd2qgz::version_stream::{
    CompatibilityBranch10_1, CompatibilityBranch10_8, VersionLayout10_8, read_arcmap_header_10_1,
    read_layout_10_1, read_layout_10_8, write_arcmap_header_10_1,
};
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
fn arcmap_10_1_six_save_indices_have_expected_bytes_and_count() {
    for (index, short) in [10_i16, 13, 14, 16, 18, 19].into_iter().enumerate() {
        let count = (index != 0).then_some(123_i32);
        let bytes = write_arcmap_header_10_1(index as i32, "10.1", count).unwrap();
        let mut expected = encoded("10.1");
        expected.extend(short.to_le_bytes());
        if let Some(value) = count {
            expected.extend(value.to_le_bytes());
        }
        assert_eq!(bytes, expected);
        let mut reader = Reader::new(&bytes);
        let parsed = read_arcmap_header_10_1(&mut reader, 1024).unwrap();
        assert_eq!(parsed.prefix.compatibility_short, short);
        assert_eq!(parsed.object_count, count);
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn arcmap_10_1_count_boundary_unknown_tail_and_failures() {
    for short in [11_i16, 12] {
        let mut bytes = encoded("版本");
        bytes.extend(short.to_le_bytes());
        if short > 11 {
            bytes.extend((-1_i32).to_le_bytes());
        }
        bytes.push(0xab);
        let mut reader = Reader::new(&bytes);
        let parsed = read_arcmap_header_10_1(&mut reader, 1024).unwrap();
        assert_eq!(parsed.object_count, (short > 11).then_some(-1));
        assert_eq!(parsed.remaining_bytes, 1);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xab]);
    }
    let bytes = write_arcmap_header_10_1(5, "10.1", Some(7)).unwrap();
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_arcmap_header_10_1(&mut reader, 1024).is_err());
        assert_eq!(reader.position(), 0);
    }
    let mut unknown = encoded("x");
    unknown.extend(20_i16.to_le_bytes());
    let mut reader = Reader::new(&unknown);
    assert_eq!(
        read_arcmap_header_10_1(&mut reader, 1024)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    assert!(write_arcmap_header_10_1(-1, "10.1", Some(0)).is_err());
    assert!(write_arcmap_header_10_1(0, "10.1", Some(0)).is_err());
    assert!(write_arcmap_header_10_1(1, "10.1", None).is_err());
    assert!(write_arcmap_header_10_1(1, "bad\0text", Some(0)).is_err());
}

#[test]
fn arcmap_10_1_save_indices_above_table_use_last_entry() {
    for index in [6, 255, i32::MAX] {
        let bytes = write_arcmap_header_10_1(index, "10.1", Some(42)).unwrap();
        let parsed = read_arcmap_header_10_1(&mut Reader::new(&bytes), 1024).unwrap();
        assert_eq!(parsed.prefix.compatibility_short, 19);
        assert_eq!(parsed.object_count, Some(42));
    }
}

#[test]
fn reads_10_1_prefix_with_signed_branches_and_preserves_tail() {
    for (short, expected) in [
        (-1_i16, CompatibilityBranch10_1::Three),
        (0x11, CompatibilityBranch10_1::Three),
        (0x12, CompatibilityBranch10_1::Five),
        (0x7fff, CompatibilityBranch10_1::Five),
    ] {
        let mut bytes = encoded("10.1");
        bytes.extend(short.to_le_bytes());
        bytes.extend([0xab, 0xcd]);
        let mut reader = Reader::new(&bytes);
        let parsed = read_layout_10_1(&mut reader, 1024).unwrap();
        assert_eq!(parsed.text, "10.1");
        assert_eq!(parsed.compatibility_short, short);
        assert_eq!(parsed.compatibility_branch, expected);
        assert_eq!(parsed.remaining_bytes, 2);
        assert_eq!(reader.read_bytes(2).unwrap(), [0xab, 0xcd]);
    }
}

#[test]
fn rejects_all_10_1_prefix_truncations_without_advancing() {
    let mut bytes = encoded("地图");
    bytes.extend(0x12_i16.to_le_bytes());
    for length in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..length]);
        assert!(read_layout_10_1(&mut reader, 1024).is_err());
        assert_eq!(reader.position(), 0);
    }
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_layout_10_1(&mut reader, 2).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn reads_confirmed_prefix_and_reports_unknown_tail() {
    let mut bytes = encoded("10.1");
    bytes.extend(0x13_i16.to_le_bytes());
    let tail = [0xdb, 0x08];
    bytes.extend(tail);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_layout_10_8(&mut reader, 1024).unwrap(),
        VersionLayout10_8 {
            text: "10.1".into(),
            compatibility_short: 0x13,
            compatibility_branch: CompatibilityBranch10_8::Eleven,
            remaining_bytes: 2,
        }
    );
    assert_eq!(reader.position(), bytes.len() - 2);
    assert_eq!(reader.read_bytes(2).unwrap(), tail);
}

#[test]
fn preserves_every_short_bit_pattern_and_optional_empty_tail() {
    for value in [0_i16, 1, 0x11, 0x12, 0x7fff, -0x8000, -1] {
        let mut bytes = encoded("x");
        bytes.extend(value.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let parsed = read_layout_10_8(&mut reader, 1024).unwrap();
        assert_eq!(parsed.compatibility_short, value);
        let expected = if value < 0x12 {
            CompatibilityBranch10_8::Three
        } else {
            CompatibilityBranch10_8::Eleven
        };
        assert_eq!(parsed.compatibility_branch, expected);
        assert_eq!(parsed.remaining_bytes, 0);
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn maps_the_confirmed_signed_comparison_boundaries() {
    for (value, expected) in [
        (0x11_i16, CompatibilityBranch10_8::Three),
        (0x12, CompatibilityBranch10_8::Eleven),
        (0x13, CompatibilityBranch10_8::Eleven),
    ] {
        let mut bytes = encoded("10.1");
        bytes.extend(value.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let parsed = read_layout_10_8(&mut reader, 1024).unwrap();
        assert_eq!(parsed.compatibility_short, value);
        assert_eq!(parsed.compatibility_branch, expected);
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn rejects_short_truncation_without_advancing_past_text() {
    let mut bytes = encoded("10.8");
    bytes.extend([0x13]);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_layout_10_8(&mut reader, 1024).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);
    assert_eq!(reader.remaining(), bytes.len());
}

#[test]
fn invalid_string_leaves_the_caller_at_the_version_start() {
    let payload = [0x00_u8, 0xd8, 0x00, 0x00];
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes.extend([0x13, 0]);
    let mut reader = Reader::new(&bytes);
    let error = read_layout_10_8(&mut reader, 1024).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("offset 0x0"));
    assert_eq!(reader.position(), 0);
    assert_eq!(reader.remaining(), bytes.len());
}

#[test]
fn nonzero_start_uses_a_local_cursor_and_keeps_the_parent_unchanged() {
    let mut bytes = [0xff].to_vec();
    bytes.extend(encoded("版本"));
    bytes.extend([0x12, 0]);
    let mut parent = Reader::new(&bytes);
    parent.read_bytes(1).unwrap();
    let parsed = read_layout_10_8(&mut parent, 1024).unwrap();
    assert_eq!(parsed.text, "版本");
    assert_eq!(parsed.compatibility_short, 0x12);
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(parent.position(), bytes.len());
}
