use mxd2qgz::maps_stream::{MapsRootPrefix10_8, read_root_prefix_10_8};
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
