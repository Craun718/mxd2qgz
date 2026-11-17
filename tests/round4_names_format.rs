//! IName 对象槽的构造输入；不代替书签实样或目标软件验证。
use mxd2qgz::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID;
use mxd2qgz::maps::map_surround::{BookmarkNameEntry10_1, read_feature_bookmark_10_1};
use mxd2qgz::objects::dataset_name_more::{BookmarkNameSlot10_1, read_bookmark_name_slot_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut data = class_id.to_vec();
    data.extend(key.to_le_bytes());
    data
}

fn feature_name() -> Vec<u8> {
    let mut data = entry(&FEATURE_CLASS_NAME_CLASS_ID, 12);
    data.extend(2_u16.to_le_bytes());
    for _ in 0..4 {
        data.extend(2_u32.to_le_bytes());
        data.extend(0_u16.to_le_bytes());
    }
    data.extend([0; 8]);
    data.extend(0_u16.to_le_bytes());
    data.extend([0; 16]);
    data.extend(0_u16.to_le_bytes());
    data
}

#[test]
fn feature_class_slot_and_back_reference() {
    let data = feature_name();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&data);
    let slot = read_bookmark_name_slot_10_1(&mut reader, 128, &mut table).unwrap();
    let BookmarkNameSlot10_1::FeatureClassName(body) = slot else {
        panic!("应读取要素类名称")
    };
    assert!(body.stop.is_none());
    assert_eq!(reader.position(), data.len());
    assert!(table.contains(12));
    let back = entry(&FEATURE_CLASS_NAME_CLASS_ID, 12);
    let mut reader = Reader::new(&back);
    assert!(matches!(
        read_bookmark_name_slot_10_1(&mut reader, 128, &mut table).unwrap(),
        BookmarkNameSlot10_1::BackReference { key: 12, .. }
    ));
    assert_eq!(reader.position(), 20);
}

#[test]
fn truncation_and_unknown_roll_back_slot() {
    let data = feature_name();
    for n in 0..data.len() {
        let mut reader = Reader::new(&data[..n]);
        let mut table = RefTable10_1::new();
        table.register(44);
        assert!(
            read_bookmark_name_slot_10_1(&mut reader, 128, &mut table).is_err(),
            "截断位置 {n}"
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(table.len(), 1);
        assert!(!table.contains(12));
    }
    let unknown = entry(&[0xaa; 16], 13);
    let mut reader = Reader::new(&unknown);
    let mut table = RefTable10_1::new();
    assert!(matches!(
        read_bookmark_name_slot_10_1(&mut reader, 128, &mut table).unwrap(),
        BookmarkNameSlot10_1::Unsupported { key: 13, .. }
    ));
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
    let null = [0; 16];
    let mut reader = Reader::new(&null);
    assert!(matches!(
        read_bookmark_name_slot_10_1(&mut reader, 128, &mut table).unwrap(),
        BookmarkNameSlot10_1::Null
    ));
    assert_eq!(reader.position(), 16);
}

#[test]
fn nested_unknown_stops_before_its_header() {
    let mut data = feature_name();
    let nested_offset = data.len() - 18;
    data.truncate(nested_offset);
    data.extend(entry(&[0xbb; 16], 21));
    let mut reader = Reader::new(&data);
    let mut table = RefTable10_1::new();
    let BookmarkNameSlot10_1::FeatureClassName(body) =
        read_bookmark_name_slot_10_1(&mut reader, 128, &mut table).unwrap()
    else {
        panic!("应读取要素类名称前缀");
    };
    assert!(body.stop.is_some());
    assert_eq!(reader.position(), nested_offset);
    assert!(table.contains(12));
    assert!(!table.contains(21));
}

#[test]
fn bookmark_reads_name_only_after_complete_feature_class_name() {
    let mut prefix = 1_u16.to_le_bytes().to_vec();
    prefix.extend(0_i32.to_le_bytes());
    let object = feature_name();
    let mut bytes = prefix.clone();
    bytes.extend(&object);
    bytes.extend([4, 0, 0, 0, b'B', 0, 0, 0, 0xaa]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let bookmark = read_feature_bookmark_10_1(&mut reader, 128, &mut table).unwrap();
    assert!(matches!(
        bookmark.name_object,
        BookmarkNameEntry10_1::FeatureClassName(_)
    ));
    assert_eq!(bookmark.name, "B");
    assert!(table.contains(12));
    assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);

    prefix.extend(&object[..object.len() - 18]);
    prefix.extend(entry(&[0xbb; 16], 21));
    prefix.extend([4, 0, 0, 0, b'B', 0, 0, 0]);
    let mut reader = Reader::new(&prefix);
    let mut table = RefTable10_1::new();
    assert_eq!(
        read_feature_bookmark_10_1(&mut reader, 128, &mut table)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::Unsupported
    );
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}
