use mxd2qgz::layers::annotate_props::{
    AlpcEntrySlot10_1, LABEL_ENGINE_PROPS_CLASS_ID, read_annotate_props_collection_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn alpc_v2_null_has_no_id_before_a_back_reference() {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(LABEL_ENGINE_PROPS_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(41_i32.to_le_bytes());
    bytes.extend(42_i32.to_le_bytes());
    let end = bytes.len();
    bytes.push(0xae);
    let mut table = RefTable10_1::new();
    table.register(17);
    let mut reader = Reader::new(&bytes);
    let body = read_annotate_props_collection_body_10_1(&mut reader, 1024, &mut table, 1).unwrap();
    assert!(matches!(body.entries[0].slot, AlpcEntrySlot10_1::Null));
    assert_eq!(body.entries[0].entry_key, None);
    assert!(matches!(
        body.entries[1].slot,
        AlpcEntrySlot10_1::BackReference { key: 17, .. }
    ));
    assert_eq!(body.entries[1].entry_key, Some(41));
    assert_eq!(body.trailing_int32, Some(42));
    assert_eq!(reader.position(), end);
    for cut in 0..end {
        let mut truncated = Reader::new(&bytes[..cut]);
        let mut staged = table.clone();
        assert!(
            read_annotate_props_collection_body_10_1(&mut truncated, 1024, &mut staged, 1).is_err()
        );
        assert_eq!(truncated.position(), 0);
        assert_eq!(staged.len(), 1);
        assert!(staged.contains(17));
    }
}
