use mxd2qgz::reader::Reader;
use mxd2qgz::version_stream::{CompatibilityBranch10_8, VersionLayout10_8, read_layout_10_8};
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
