//! 旧 FDO 图层中 ElementCollection 槽的合成输入回归。
use mxd2qgz::layers::fdo_legacy::{LegacyFdoStop10_1, read_fdo_graphics_layer_legacy_prefix_10_1};
use mxd2qgz::objects::element_collection::ELEMENT_COLLECTION_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn prefix() -> Vec<u8> {
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0; 4 + 16 + 4 + 2]);
    bytes.extend([0; 16]);
    bytes.extend([0; 16]);
    bytes.extend(ELEMENT_COLLECTION_CLASS_ID);
    bytes.extend(55_u32.to_le_bytes());
    bytes
}

#[test]
fn empty_collection_continues_and_truncation_rolls_back() {
    let mut bytes = prefix();
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend([0; 16]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let value = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(value.stop.is_none());
    assert!(value.element_collection_body.is_some());
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(55));
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let mut table = RefTable10_1::new();
        assert!(read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).is_err());
        assert_eq!(reader.position(), 0);
        assert!(!table.contains(55));
    }
}

#[test]
fn unknown_element_stops_before_its_header() {
    let mut bytes = prefix();
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    let inner_offset = bytes.len();
    bytes.extend([0x42; 16]);
    bytes.extend(56_u32.to_le_bytes());
    bytes.extend([0; 12]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let value = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(matches!(
        value.stop,
        Some(LegacyFdoStop10_1::ElementCollectionStopped(_))
    ));
    assert_eq!(reader.position(), inner_offset);
    assert!(table.contains(55));
    assert!(!table.contains(56));
}
