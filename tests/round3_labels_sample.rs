//! 参考样本中的既有对象边界回归；不替代 ArcMap 运行验证。

use mxd2qgz::layers::annotate_props::{
    HOT_LINK_EXPRESSION_PROPS_CLASS_ID, read_hot_link_expression_props_body_10_1,
};
use mxd2qgz::layers::feature_id_set::{FEATURE_ID_SET_CLASS_ID, read_feature_id_set_body_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::storage::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

fn maps() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/测试底图.mxd");
    let mut compound = cfb::open(path).unwrap();
    let mut bytes = Vec::new();
    compound
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

#[test]
fn reference_sample_feature_id_set_boundary() {
    let bytes = maps();
    let mut reader = Reader::new(&bytes[0xf63..0xf7d]);
    assert_eq!(
        read_header_10_1(&mut reader).unwrap(),
        ObjectHeader::Object {
            class_id: FEATURE_ID_SET_CLASS_ID,
            key: 75,
        }
    );
    let body = read_feature_id_set_body_10_1(&mut reader, &mut RefTable10_1::new(), 75).unwrap();
    assert_eq!((body.version, body.count, body.ids.len()), (1, 0, 0));
    assert_eq!(reader.position(), 26);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reference_sample_hotlink_prefix_and_truncation() {
    let bytes = maps();
    let mut reader = Reader::new(&bytes[0x15d8..0x1606]);
    assert_eq!(
        read_header_10_1(&mut reader).unwrap(),
        ObjectHeader::Object {
            class_id: HOT_LINK_EXPRESSION_PROPS_CLASS_ID,
            key: 85,
        }
    );
    let start = reader.position();
    let body =
        read_hot_link_expression_props_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 85)
            .unwrap();
    assert_eq!(
        (body.version, body.string.as_str(), body.short),
        (1, "", -1)
    );
    assert_eq!(body.guid_block, [0; 16]);
    assert_eq!(reader.remaining(), 0);
    let data = &bytes[0x15d8 + start..0x1606];
    for cut in 0..data.len() {
        let mut truncated = Reader::new(&data[..cut]);
        assert!(
            read_hot_link_expression_props_body_10_1(
                &mut truncated,
                1024,
                &mut RefTable10_1::new(),
                85
            )
            .is_err()
        );
        assert_eq!(truncated.position(), 0);
    }
}
