use mxd2qgz::object_stream::{ObjectHeader, read_header_10_8};
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

#[test]
fn null_header_leaves_the_following_field_unread() {
    let mut bytes = vec![0; 16];
    bytes.extend_from_slice(&[1, 2, 3, 4]);
    let mut reader = Reader::new(&bytes);
    assert_eq!(read_header_10_8(&mut reader).unwrap(), ObjectHeader::Null);
    assert_eq!(reader.position(), 16);
    assert_eq!(reader.read_u32_le().unwrap(), 0x04030201);
}

#[test]
fn preserves_class_bytes_and_all_key_bits_without_interpreting_the_body() {
    for nonzero_index in 0..16 {
        let mut class_id = [0; 16];
        class_id[nonzero_index] = 0x80;
        for key in [0, 1, 0x12345678, u32::MAX] {
            let mut bytes = class_id.to_vec();
            bytes.extend_from_slice(&key.to_le_bytes());
            bytes.extend_from_slice(b"opaque");
            let mut reader = Reader::new(&bytes);
            assert_eq!(
                read_header_10_8(&mut reader).unwrap(),
                ObjectHeader::Object { class_id, key }
            );
            assert_eq!(reader.position(), 20);
            assert_eq!(reader.read_bytes(6).unwrap(), b"opaque");
        }
    }
}

#[test]
fn every_truncation_preserves_the_starting_cursor() {
    for is_null in [true, false] {
        let mut bytes = vec![0xa5];
        bytes.extend_from_slice(&[if is_null { 0 } else { 0x80 }; 16]);
        bytes.extend_from_slice(&[0x78, 0x56, 0x34, 0x12]);
        let width = if is_null { 16 } else { 20 };
        for available in 0..width {
            let mut reader = Reader::new(&bytes[..1 + available]);
            reader.read_bytes(1).unwrap();
            let error = read_header_10_8(&mut reader).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
            let offset = if available < 16 { 1 } else { 17 };
            assert!(error.to_string().contains(&format!("offset 0x{offset:x}")));
            assert_eq!(reader.position(), 1);
            assert_eq!(reader.remaining(), available);
            assert_eq!(
                reader.read_bytes(available).unwrap(),
                &bytes[1..1 + available]
            );
        }
    }
}

#[test]
fn child_region_cannot_borrow_a_key_from_the_next_field() {
    let bytes = [1; 20];
    let mut parent = Reader::new(&bytes);
    let mut child = parent.read_region(16).unwrap();
    assert!(read_header_10_8(&mut child).is_err());
    assert_eq!(child.position(), 0);
    assert_eq!(parent.position(), 16);
    assert_eq!(parent.read_u32_le().unwrap(), 0x01010101);
}
