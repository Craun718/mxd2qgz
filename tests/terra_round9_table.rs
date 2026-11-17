use mxd2qgz::{
    layers::feature_layer_tail::{FIELD_INFO_CLASS_ID, FieldInfoStop10_1},
    maps::{
        map_object::{MAP_CLASS_ID, read_map_suffix_10_1},
        standalone_table::{StandaloneTableStop10_1, read_standalone_table_prefix_10_1},
    },
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::reader::Reader,
};
use std::io::Read;

fn string(s: &str) -> Vec<u8> {
    let units: Vec<_> = s.encode_utf16().chain([0]).collect();
    let mut out = ((units.len() * 2) as u32).to_le_bytes().to_vec();
    out.extend(units.into_iter().flat_map(u16::to_le_bytes));
    out
}

fn fixture(unknown_format: bool) -> (Vec<u8>, usize) {
    let mut data = 1u16.to_le_bytes().to_vec();
    data.extend([0; 16]);
    data.extend(1u32.to_le_bytes());
    data.extend(string("字段"));
    data.extend(FIELD_INFO_CLASS_ID);
    data.extend(334u32.to_le_bytes());
    data.extend(2u16.to_le_bytes());
    data.extend(0xffffu16.to_le_bytes());
    data.extend(0u16.to_le_bytes());
    data.extend(string("别名"));
    let slot = data.len();
    if unknown_format {
        data.extend([0x55; 16]);
        data.extend(335u32.to_le_bytes());
    } else {
        data.extend([0; 16]);
        data.extend(string("表"));
    }
    (data, slot)
}

#[test]
fn field_info_v2_consumed_and_truncation_rolls_back() {
    let (data, _) = fixture(false);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
    assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
    let body = p.named_object_bodies[0].as_ref().unwrap();
    assert_eq!((body.key, body.version), (334, 2));
    assert!(body.short_18.is_none());
    assert_eq!(r.position(), data.len());
    assert!(refs.contains(334));
    for end in 0..data.len() {
        let mut r = Reader::new(&data[..end]);
        let mut refs = RefTable10_1::new();
        refs.register(8);
        assert!(
            read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).is_err(),
            "{end}"
        );
        assert_eq!(r.position(), 0);
        assert_eq!(refs.len(), 1);
    }
}

#[test]
fn field_info_unknown_inner_object_stops_and_keeps_outer_key() {
    let (data, slot) = fixture(true);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
    assert!(matches!(
        p.stop,
        StandaloneTableStop10_1::NamedFieldInfoBody(FieldInfoStop10_1::NumberFormatUnsupported {
            key: 335,
            ..
        })
    ));
    assert_eq!(r.position(), slot);
    assert!(refs.contains(334));
    assert!(!refs.contains(335));
    assert!(p.named_object_bodies[0].as_ref().unwrap().stop.is_some());
}

#[test]
#[ignore = "需要 10.1 SDK Java 安装介质提取文件"]
fn linearref_two_tables_from_map_host() {
    let path = "temp/terra-round8-old/sdk-java/LinearRef.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1";
    let mut doc = cfb::open(path).unwrap();
    let mut data = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    assert!(r.read_u32_le().unwrap() > 0);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut r, 16_000_000, &mut refs).unwrap();
    assert_eq!(map.standalone_tables.len(), 2);
    for (slot, body) in &map.standalone_tables {
        assert!([0x6a50, 0x74ef].contains(&slot.offset));
        let body = body.as_ref().unwrap();
        assert_eq!(body.version, 9);
        assert_eq!(body.stop, StandaloneTableStop10_1::Complete);
    }
    let first = map.standalone_tables[0].1.as_ref().unwrap();
    assert_eq!(first.named_objects.len(), 24);
    assert!(first.named_object_bodies.iter().all(|b| {
        b.as_ref()
            .is_some_and(|b| b.version == 2 && b.stop.is_none())
    }));
    assert_eq!(first.named_object_bodies[0].as_ref().unwrap().key, 334);
    assert_eq!(&data[0x6bed..0x6bfd], &FIELD_INFO_CLASS_ID);
    assert_eq!(&data[0x6bfd..0x6c03], &[0x4e, 1, 0, 0, 2, 0]);
    assert_eq!(r.position(), 0x8407);
}
