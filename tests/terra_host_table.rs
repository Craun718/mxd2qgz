use mxd2qgz::maps::standalone_table::{StandaloneTableStop10_1, read_standalone_table_prefix_10_1};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;

fn string(s: &str) -> Vec<u8> {
    let units: Vec<u16> = s.encode_utf16().chain([0]).collect();
    let mut out = ((units.len() * 2) as u32).to_le_bytes().to_vec();
    out.extend(units.into_iter().flat_map(u16::to_le_bytes));
    out
}

#[test]
fn reads_null_and_reference_prefix_and_stops_before_remaining_body() {
    let mut data = 1u16.to_le_bytes().to_vec();
    data.extend([0; 16]);
    data.extend(1u32.to_le_bytes());
    data.extend(string("字段"));
    data.extend([3; 16]);
    data.extend(9u32.to_le_bytes());
    data.extend(string("表"));
    data.push(0xaa);
    let mut refs = RefTable10_1::new();
    refs.register(9);
    let mut r = Reader::new(&data);
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 2, 100).unwrap();
    assert_eq!(p.dataset_name, Some(ObjectEntry10_1::Null));
    assert!(matches!(
        p.named_objects[0].1,
        ObjectEntry10_1::BackReference { key: 9, .. }
    ));
    assert_eq!(p.trailing_string.as_deref(), Some("表"));
    assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
    assert_eq!(r.read_bytes(1).unwrap(), &[0xaa]);
}

#[test]
fn stops_at_unknown_nested_object_without_registering_it() {
    let mut data = 9u16.to_le_bytes().to_vec();
    data.extend([7; 16]);
    data.extend(42u32.to_le_bytes());
    let mut refs = RefTable10_1::new();
    let mut r = Reader::new(&data);
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 1, 20).unwrap();
    assert!(matches!(
        p.stop,
        StandaloneTableStop10_1::DatasetName { key: 42, .. }
    ));
    assert_eq!(r.position(), 2);
    assert!(refs.is_empty());
}

#[test]
fn errors_are_transactional_for_truncation_version_and_limits() {
    let mut data = 9u16.to_le_bytes().to_vec();
    data.extend([0; 16]);
    data.extend(1u32.to_le_bytes());
    data.extend(string("a"));
    data.extend([0; 15]);
    for bytes in [&data[..], &[15, 0][..]] {
        let mut refs = RefTable10_1::new();
        refs.register(99);
        let mut r = Reader::new(bytes);
        assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 1, 20).is_err());
        assert_eq!(r.position(), 0);
        assert_eq!(refs.len(), 1);
    }
    let mut r = Reader::new(&data);
    assert!(read_standalone_table_prefix_10_1(&mut r, &mut RefTable10_1::new(), 0, 20).is_err());
    assert_eq!(r.position(), 0);
}

#[test]
fn stops_at_named_new_object_after_consuming_its_name() {
    let mut data = 9u16.to_le_bytes().to_vec();
    data.extend([0; 16]);
    data.extend(1u32.to_le_bytes());
    data.extend(string("列"));
    let slot = data.len();
    data.extend([5; 16]);
    data.extend(7u32.to_le_bytes());
    let mut refs = RefTable10_1::new();
    let mut r = Reader::new(&data);
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 1, 20).unwrap();
    assert!(
        matches!(p.stop, StandaloneTableStop10_1::NamedObject { name, key: 7, .. } if name == "列")
    );
    assert_eq!(r.position(), slot);
    assert!(refs.is_empty());
}

#[test]
fn signed_legacy_versions_use_the_common_prefix() {
    for version in [i16::MIN, -1, 0, 1] {
        let mut data = version.to_le_bytes().to_vec();
        data.extend([0; 16]);
        data.extend(0_u32.to_le_bytes());
        data.extend(string("表"));
        let end = data.len();
        data.push(0xaa);
        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let table = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 1, 100).unwrap();
        assert_eq!(table.version, version as u16);
        assert_eq!(table.trailing_string.as_deref(), Some("表"));
        assert_eq!(table.stop, StandaloneTableStop10_1::Complete);
        assert_eq!(reader.position(), end);
        assert!(refs.is_empty());
        for len in 0..end {
            let mut reader = Reader::new(&data[..len]);
            assert!(read_standalone_table_prefix_10_1(&mut reader, &mut refs, 1, 100).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}
