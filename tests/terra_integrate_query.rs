//! 合成数据只验证严格读取策略，不代表 ArcMap 实机验证。
use mxd2qgz::objects::dataset_name_more::{
    DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, DatasetNameMoreStop10_1,
    FEATURE_QUERY_NAME_CLASS_ID, TABLE_QUERY_NAME_CLASS_ID, read_dataset_name_slot_10_1,
    read_feature_query_name_body_10_1, read_table_query_name_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn string(s: &str, out: &mut Vec<u8>) {
    let bytes: Vec<u8> = s
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    out.extend((bytes.len() as u32).to_le_bytes());
    out.extend(bytes);
}
fn body(version: u16, feature: bool) -> Vec<u8> {
    let mut b = version.to_le_bytes().to_vec();
    for s in ["a", "b", "c"] {
        string(s, &mut b);
    }
    b.extend([0; 16]);
    for s in ["d", "e", "f"] {
        string(s, &mut b);
    }
    if version >= 2 {
        string("g", &mut b);
        b.extend(0x1234u16.to_le_bytes());
    }
    if feature {
        string("h", &mut b);
        b.extend(0x12345678u32.to_le_bytes());
        b.extend(0x90abcdefu32.to_le_bytes());
    }
    b
}
#[test]
fn versions_and_dispatch_consume_exact_bodies() {
    for version in 0..=2 {
        for (class, feature) in [
            (TABLE_QUERY_NAME_CLASS_ID, false),
            (FEATURE_QUERY_NAME_CLASS_ID, true),
        ] {
            let mut bytes = class.to_vec();
            bytes.extend(7u32.to_le_bytes());
            bytes.extend(body(version, feature));
            let end = bytes.len();
            bytes.extend([0xaa; 4]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let mut stop = None;
            let result =
                read_dataset_name_slot_10_1(&mut reader, 1024, &mut refs, &mut stop).unwrap();
            match result {
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::TableQuery(b)) => {
                    assert!(!feature);
                    assert_eq!(b.version_two_tail.is_some(), version == 2);
                    assert_eq!(b.strings_after_workspace.unwrap()[1], "e");
                }
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::FeatureQuery(b)) => {
                    assert!(feature);
                    assert_eq!(
                        b.feature_tail.unwrap(),
                        ("h".into(), 0x12345678, 0x90abcdef)
                    );
                }
                _ => panic!("错误的类分派"),
            }
            assert_eq!(reader.position(), end);
            assert!(refs.contains(7));
            assert!(stop.is_none());
        }
    }
}
#[test]
fn all_truncations_and_bad_versions_roll_back() {
    for feature in [false, true] {
        let bytes = body(2, feature);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = RefTable10_1::new();
            refs.register(42);
            let ok = if feature {
                read_feature_query_name_body_10_1(&mut reader, 1024, &mut refs, 1).is_ok()
            } else {
                read_table_query_name_body_10_1(&mut reader, 1024, &mut refs, 1).is_ok()
            };
            assert!(!ok, "截断位置 {end}");
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
            assert!(refs.contains(42));
        }
    }
    for v in [3u16, 0xffff] {
        let bytes = v.to_le_bytes();
        let mut r = Reader::new(&bytes);
        let mut t = RefTable10_1::new();
        assert!(read_table_query_name_body_10_1(&mut r, 1024, &mut t, 1).is_err());
        assert_eq!(r.position(), 0);
    }
}
#[test]
fn unknown_workspace_key_stops_before_entry_without_registering() {
    let mut bytes = 2u16.to_le_bytes().to_vec();
    for s in ["a", "b", "c"] {
        string(s, &mut bytes);
    }
    let slot = bytes.len();
    bytes.extend([0x71; 16]);
    bytes.extend(99u32.to_le_bytes());
    bytes.extend([0xff; 30]);
    let mut r = Reader::new(&bytes);
    let mut t = RefTable10_1::new();
    t.register(1);
    let result = read_feature_query_name_body_10_1(&mut r, 1024, &mut t, 1).unwrap();
    assert_eq!(r.position(), slot);
    assert!(!t.contains(99));
    assert!(result.feature_tail.is_none());
    assert!(result.table.strings_after_workspace.is_none());
    assert!(matches!(
        result.table.stop,
        Some(DatasetNameMoreStop10_1::SlotUnsupported { key: 99, .. })
    ));
}
#[test]
fn known_workspace_reference_is_consumed() {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for s in ["a", "b", "c"] {
        string(s, &mut bytes);
    }
    bytes.extend([0x71; 16]);
    bytes.extend(99u32.to_le_bytes());
    for s in ["d", "e", "f"] {
        string(s, &mut bytes);
    }
    let mut r = Reader::new(&bytes);
    let mut t = RefTable10_1::new();
    t.register(99);
    let result = read_table_query_name_body_10_1(&mut r, 1024, &mut t, 1).unwrap();
    assert!(result.stop.is_none());
    assert_eq!(r.position(), bytes.len());
}
