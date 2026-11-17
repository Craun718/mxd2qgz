use mxd2qgz::maps::layer_masking::read_optional_layer_masking_10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn fixture(version: u16) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(version.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0x11; 16]);
    bytes.extend(7_u32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0x22; 16]);
    bytes.extend(8_u32.to_le_bytes());
    if version > 1 {
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend([b'A', 0, 0, 0]);
    }
    bytes.extend(0xffff_u16.to_le_bytes());
    bytes
}

#[test]
fn nonempty_versions_and_transactional_truncation() {
    for version in 0..=2 {
        let bytes = fixture(version);
        for end in 0..bytes.len() {
            let mut refs = RefTable10_1::new();
            refs.register(7);
            refs.register(8);
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_optional_layer_masking_10_1(&mut reader, 1024, &mut refs, true).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 2);
        }
        let mut refs = RefTable10_1::new();
        refs.register(7);
        refs.register(8);
        let mut reader = Reader::new(&bytes);
        let result = read_optional_layer_masking_10_1(&mut reader, 1024, &mut refs, true)
            .unwrap()
            .unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(
            result.entries[0].targets[0].1.as_deref(),
            if version > 1 { Some("A") } else { None }
        );
        assert_eq!(result.tail, 0xffff);
    }
}

#[test]
fn absent_interface_unknown_new_and_bad_counts() {
    let bytes = fixture(2);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    assert!(
        read_optional_layer_masking_10_1(&mut reader, 1024, &mut refs, false)
            .unwrap()
            .is_none()
    );
    assert_eq!(reader.position(), 0);
    assert!(read_optional_layer_masking_10_1(&mut reader, 1024, &mut refs, true).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
    for count in [-1_i32, 1000] {
        let mut bad = bytes.clone();
        bad[2..6].copy_from_slice(&count.to_le_bytes());
        assert!(
            read_optional_layer_masking_10_1(&mut Reader::new(&bad), 1024, &mut refs, true)
                .is_err()
        );
    }
}
