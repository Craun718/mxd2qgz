use mxd2qgz::maps::map_persist_collections::{
    MapPersistCollectionsStop10_1, read_map_persist_collections_before_v34_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::{ErrorKind, Read};

fn object_header(class_id: [u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

#[test]
fn reference_sample_advances_from_253c9_to_before_v34_block() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    assert_eq!(u16::from_le_bytes(maps[0x18..0x1a].try_into().unwrap()), 51);

    let mut reader = Reader::new(&maps);
    reader.read_bytes(0x253c9).unwrap();
    let mut refs = RefTable10_1::new();
    let parsed =
        read_map_persist_collections_before_v34_10_1(&mut reader, 4_000_000, &mut refs, 51)
            .unwrap();

    assert_eq!(parsed.start_offset, 0x253c9);
    assert_eq!(parsed.count, 0);
    assert!(parsed.collection.is_empty());
    assert_eq!(parsed.long_value, Some(0));
    assert_eq!(parsed.map_interface.unwrap().entry, ObjectEntry10_1::Null);
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.position(), 0x253e1);
    assert!(refs.is_empty());
}

#[test]
fn empty_and_back_reference_collection_slots_then_long_and_map_slot() {
    let class_id = [0x6d; 16];
    let mut bytes = 2_i32.to_le_bytes().to_vec();
    bytes.extend([0; 16]);
    bytes.extend(object_header(class_id, 42));
    bytes.extend((-17_i32).to_le_bytes());
    bytes.extend([0; 16]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    refs.register(42);

    let parsed =
        read_map_persist_collections_before_v34_10_1(&mut reader, 256, &mut refs, 33).unwrap();

    assert_eq!(parsed.count, 2);
    assert_eq!(parsed.collection.len(), 2);
    assert_eq!(parsed.collection[0].entry, ObjectEntry10_1::Null);
    assert_eq!(
        parsed.collection[1].entry,
        ObjectEntry10_1::BackReference { class_id, key: 42 }
    );
    assert_eq!(parsed.long_value, Some(-17));
    assert_eq!(parsed.map_interface.unwrap().entry, ObjectEntry10_1::Null);
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(refs.len(), 1);
}

#[test]
fn unknown_collection_object_stops_before_header_and_rolls_back_its_key() {
    let class_id = [0xa5; 16];
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.extend(object_header(class_id, 77));
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();

    let parsed =
        read_map_persist_collections_before_v34_10_1(&mut reader, 256, &mut refs, 33).unwrap();

    assert_eq!(reader.position(), 4);
    assert!(!refs.contains(77));
    assert_eq!(
        parsed.stop,
        Some(MapPersistCollectionsStop10_1::CollectionObjectBody {
            index: 0,
            offset: 4,
            class_id,
            key: 77,
        })
    );
    assert_eq!(parsed.long_value, None);
}

#[test]
fn unknown_map_interface_object_stops_before_header_and_rolls_back_key() {
    let class_id = [0x35; 16];
    let mut bytes = 0_i32.to_le_bytes().to_vec();
    bytes.extend(123_i32.to_le_bytes());
    bytes.extend(object_header(class_id, 88));
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();

    let parsed =
        read_map_persist_collections_before_v34_10_1(&mut reader, 256, &mut refs, 33).unwrap();

    assert_eq!(reader.position(), 8);
    assert!(!refs.contains(88));
    assert_eq!(parsed.long_value, Some(123));
    assert_eq!(
        parsed.stop,
        Some(MapPersistCollectionsStop10_1::MapInterfaceObjectBody {
            offset: 8,
            class_id,
            key: 88,
        })
    );
}

#[test]
fn malformed_count_or_truncation_rolls_back_cursor_and_reference_table() {
    for (bytes, expected_kind) in [
        (vec![0xff; 4], ErrorKind::InvalidData),
        (1_i32.to_le_bytes().to_vec(), ErrorKind::UnexpectedEof),
        (i32::MAX.to_le_bytes().to_vec(), ErrorKind::InvalidData),
    ] {
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(5);

        let error = read_map_persist_collections_before_v34_10_1(&mut reader, 64, &mut refs, 33)
            .unwrap_err();

        assert_eq!(error.kind(), expected_kind);
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(5));
    }
}

#[test]
fn versions_below_33_are_rejected_without_advancing() {
    let bytes = [0; 16];
    let mut reader = Reader::new(&bytes);
    reader.read_bytes(3).unwrap();
    let mut refs = RefTable10_1::new();

    let error =
        read_map_persist_collections_before_v34_10_1(&mut reader, 64, &mut refs, 32).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 3);
    assert!(refs.is_empty());
}
