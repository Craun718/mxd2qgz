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
    let mut oversized = bytes;
    oversized[2..6].copy_from_slice(&1000_i32.to_le_bytes());
    assert!(
        read_optional_layer_masking_10_1(&mut Reader::new(&oversized), 1024, &mut refs, true)
            .is_err()
    );
}

#[test]
fn reads_new_layers_in_both_mask_slots_and_shares_keys() {
    use mxd2qgz::layers::group_layer::GROUP_LAYER_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    fn group(key: u32) -> Vec<u8> {
        let mut bytes = GROUP_LAYER_CLASS_ID.to_vec();
        bytes.extend(key.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend([0; 2 + 3 * 2 + 2 * 8 + 4]);
        bytes
    }

    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(group(7));
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(group(8));
    bytes.extend([2, 0, 0, 0, 0, 0]);
    bytes.extend(&group(8)[..20]);
    bytes.extend([4, 0, 0, 0, b'A', 0, 0, 0]);
    bytes.extend(0xffff_u16.to_le_bytes());
    for end in 0..bytes.len() {
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_optional_layer_masking_10_1(&mut reader, 1024, &mut refs, true).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_optional_layer_masking_10_1(&mut reader, 1024, &mut refs, true)
        .unwrap()
        .unwrap();
    assert!(matches!(
        result.entries[0].masked_layer(),
        ObjectEntry10_1::NewObject { key: 7, .. }
    ));
    assert!(matches!(
        result.entries[0].masking_layers()[0].0,
        ObjectEntry10_1::NewObject { key: 8, .. }
    ));
    assert!(matches!(
        result.entries[0].masking_layers()[1].0,
        ObjectEntry10_1::BackReference { key: 8, .. }
    ));
    assert_eq!(
        result.entries[0].masking_layers()[1].1.as_deref(),
        Some("A")
    );
    assert_eq!(
        result.entries[0].masking_levels(0),
        Some(mxd2qgz::maps::layer_masking::MaskingLevels10_1::All)
    );
    assert_eq!(result.entries[0].masking_levels(1), None);
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(7) && refs.contains(8));
}

#[test]
fn masking_level_lists_decode_inclusive_ranges_and_preserve_unknown_forms() {
    use mxd2qgz::maps::layer_masking::{MaskingLevels10_1, decode_masking_levels_10_1};

    assert_eq!(
        decode_masking_levels_10_1(" \t"),
        Some(MaskingLevels10_1::All)
    );
    assert_eq!(
        decode_masking_levels_10_1(" 1;3-5,,8; "),
        Some(MaskingLevels10_1::Selected(vec![1..=1, 3..=5, 8..=8]))
    );
    for unknown in ["A", "1-", "-2", "1-2-3", "2147483648"] {
        assert_eq!(decode_masking_levels_10_1(unknown), None);
    }
}
