use mxd2qgz::layers::feature_id_set::read_feature_id_set_body_10_1;
use mxd2qgz::layers::feature_layer_tail::{NumberFormatSlot10_1, read_field_info_body_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

// 这些构造输入只验证读取器契约，不充当历史文件证据。
fn field_info(version: u16) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend((-1_i16).to_le_bytes());
    if version > 1 {
        bytes.extend(17_i16.to_le_bytes());
    }
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes
}

#[test]
fn field_info_v0_to_v2_and_all_truncations() {
    for version in 0..=2 {
        let bytes = field_info(version);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(9);
        let body = read_field_info_body_10_1(&mut reader, 1024, &mut refs, 9).unwrap();
        assert_eq!(body.version, version);
        assert_eq!(body.short_14, (version == 2).then_some(17));
        assert_eq!(body.number_format, NumberFormatSlot10_1::Null);
        assert_eq!(
            (body.short_18, body.int32_1c, body.tail_shorts),
            (None, None, None)
        );
        assert_eq!(reader.remaining(), 0);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = RefTable10_1::new();
            refs.register(9);
            assert!(read_field_info_body_10_1(&mut reader, 1024, &mut refs, 9).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.contains(9));
            assert_eq!(refs.len(), 1);
        }
    }
}

#[test]
fn feature_id_set_checks_count_before_consumption() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend((-7_i32).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    refs.register(9);
    let body = read_feature_id_set_body_10_1(&mut reader, &mut refs, 9).unwrap();
    assert_eq!(body.ids, [42, -7]);
    assert_eq!(reader.remaining(), 0);

    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_feature_id_set_body_10_1(&mut reader, &mut refs, 9).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);
    }
    let mut huge = 1_u16.to_le_bytes().to_vec();
    huge.extend(i32::MAX.to_le_bytes());
    let mut reader = Reader::new(&huge);
    let error = read_feature_id_set_body_10_1(&mut reader, &mut refs, 9).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);

    let mut negative = 1_u16.to_le_bytes().to_vec();
    negative.extend((-1_i32).to_le_bytes());
    let mut reader = Reader::new(&negative);
    let body = read_feature_id_set_body_10_1(&mut reader, &mut refs, 9).unwrap();
    assert_eq!(body.count, -1);
    assert!(body.ids.is_empty());
}
