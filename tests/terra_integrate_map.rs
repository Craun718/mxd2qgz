use mxd2qgz::maps::map_object::{
    MAP_CLASS_ID, MapGeometryBody10_1, MapSuffixStop, MapTransformationDirection10_1,
    NADCON_TRANSFORMATION_CLASS_ID, read_map_suffix_10_1, read_nadcon_transformation_10_1,
    read_post_illumination_null_geometry_10_1,
};
use mxd2qgz::maps::standalone_table::STANDALONE_TABLE_CLASS_ID;
use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
use mxd2qgz::objects::geo_transformation::{
    COMPOSITE_GEO_TRANSFORMATION_CLASS_ID, GEO_TRANSFORMATION_CLASS_ID,
    NTV2_TRANSFORMATION_CLASS_ID,
};
use mxd2qgz::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID, POINT_CLASS_ID,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

#[test]
fn nonempty_standalone_table_stops_at_its_first_unknown_child() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    const TABLE: usize = 0x24fb2;
    bytes.truncate(TABLE);
    bytes[TABLE - 4..TABLE].copy_from_slice(&1_i32.to_le_bytes());
    bytes.extend(STANDALONE_TABLE_CLASS_ID);
    bytes.extend(50000_u32.to_le_bytes());
    bytes.extend(14_u16.to_le_bytes());
    let child = bytes.len();
    bytes.extend([0x5b; 16]);
    bytes.extend(50001_u32.to_le_bytes());
    for (input, expected, table_seen) in [
        (bytes.as_slice(), child, true),
        (&bytes[..child + 19], TABLE, false),
    ] {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1_after_root_prefix(&mut reader, &mut refs);
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        assert_eq!(reader.position(), expected);
        assert_eq!(!map.standalone_tables.is_empty(), table_seen);
        assert_eq!(refs.contains(50000), table_seen);
        assert!(!refs.contains(50001));
        assert!(map.illumination.is_none());
    }
}

#[test]
fn complete_standalone_table_and_back_reference_reach_illumination() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    const TABLE: usize = 0x24fb2;
    let tail = bytes.split_off(TABLE);
    bytes[TABLE - 4..TABLE].copy_from_slice(&2_i32.to_le_bytes());
    bytes.extend(STANDALONE_TABLE_CLASS_ID);
    bytes.extend(50000_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(STANDALONE_TABLE_CLASS_ID);
    bytes.extend(50000_u32.to_le_bytes());
    let illumination_offset = bytes.len();
    bytes.extend(tail);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    read_object_entry_10_1_after_root_prefix(&mut reader, &mut refs);
    let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    assert_eq!(map.standalone_tables.len(), 2);
    assert!(map.standalone_tables[0].1.is_some());
    assert!(matches!(
        map.standalone_tables[1].0.entry,
        ObjectEntry10_1::BackReference { key: 50000, .. }
    ));
    assert_eq!(map.illumination.unwrap().object.offset, illumination_offset);
    assert!(refs.contains(50000));
}

fn read_object_entry_10_1_after_root_prefix(reader: &mut Reader<'_>, refs: &mut RefTable10_1) {
    assert_eq!(reader.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(reader, refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
}

#[test]
fn map_geometry_null_is_reached_from_reference_root() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    assert_eq!(reader.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut table).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut table).unwrap();
    assert!(map.post_illumination.is_some());
    let slot = map.post_illumination_null_geometry.unwrap();
    assert_eq!(slot.offset, 0x24ff4);
    assert_eq!(slot.entry, ObjectEntry10_1::Null);
    let border = map.post_illumination_null_border.unwrap();
    assert_eq!(border.offset, 0x25004);
    assert_eq!(border.entry, ObjectEntry10_1::Null);
    assert_eq!(reader.position(), 0x25569);
    assert_eq!(
        map.post_border_tail.post_transform_double_bits,
        Some((0x2524f, 5000_f64.to_bits()))
    );
    assert_eq!(
        map.post_border_tail.post_transform_short,
        Some((0x25257, 0))
    );
    assert_eq!(
        map.post_border_tail
            .post_transform_null_background
            .unwrap()
            .offset,
        0x25259
    );
    assert!(map.post_border_tail.post_transform_null_object.is_none());
    let (slot, body) = map
        .post_border_tail
        .post_transform_overposter
        .as_ref()
        .unwrap();
    assert_eq!(slot.offset, 0x25269);
    assert_eq!(body.version, 2);
    assert!(table.contains(2180));
    assert!(table.contains(2181));
    let items = &map.post_border_tail.transformations;
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].direction_on_disk(),
        Some(MapTransformationDirection10_1::Forward)
    );
    assert_eq!(
        items[1].direction_on_disk(),
        Some(MapTransformationDirection10_1::Reverse)
    );
    assert_eq!(items[0].object.unwrap().offset, 0x2505e);
    assert_eq!(items[0].body.as_ref().unwrap().version, 1);
    assert_eq!(items[0].body.as_ref().unwrap().text.len(), 447);
    assert!(matches!(
        items[1].object.unwrap().entry,
        ObjectEntry10_1::BackReference { .. }
    ));
    assert_eq!(
        items[0].object.unwrap().entry,
        ObjectEntry10_1::NewObject {
            class_id: NADCON_TRANSFORMATION_CLASS_ID,
            key: 2179
        }
    );
}

#[test]
fn nonempty_or_truncated_border_is_preserved_after_null_geometry() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let border = 0x25004;
    bytes[border] = 1;
    for input in [&bytes[..border + 15], bytes.as_slice()] {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        assert_eq!(reader.read_u32_le().unwrap(), 1);
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        assert_eq!(map.stop, MapSuffixStop::AfterNullGeometry);
        assert!(map.post_illumination_null_geometry.is_some());
        assert!(map.post_illumination_null_border.is_none());
        assert_eq!(reader.position(), border);
        assert_eq!(map.remaining_bytes, input.len() - border);
    }
}

#[test]
fn nonempty_map_geometry_reaches_border_and_preserves_unknown_slots() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    const GEOMETRY: usize = 0x24ff4;
    const KEY: u32 = 60001;
    let header = |class: [u8; 16]| {
        let mut bytes = class.to_vec();
        bytes.extend(KEY.to_le_bytes());
        bytes
    };

    let mut point = header(POINT_CLASS_ID);
    point.extend(1_u16.to_le_bytes());
    point.extend(20_u32.to_le_bytes());
    point.extend(1_u32.to_le_bytes());
    point.extend(2.5_f64.to_le_bytes());
    point.extend(4.0_f64.to_le_bytes());
    let mut envelope = header(ENVELOPE_CLASS_ID);
    envelope.extend(1_u16.to_le_bytes());
    for value in [10.0_f64, 20.0, 30.0, 40.0] {
        envelope.extend(value.to_le_bytes());
    }
    let mut bag = header(GEOMETRY_BAG_CLASS_ID);
    bag.extend(1_u16.to_le_bytes());
    bag.extend([0; 4 + 16 + 16 + 1]);
    let mut patch = header(MULTIPATCH_CLASS_ID);
    patch.extend(2_u16.to_le_bytes());
    patch.extend(4_u32.to_le_bytes());
    patch.extend([0; 4 + 16]);
    let backref = header(POINT_CLASS_ID);
    for (slot_bytes, class_id, is_reference) in [
        (&point, POINT_CLASS_ID, false),
        (&envelope, ENVELOPE_CLASS_ID, false),
        (&backref, POINT_CLASS_ID, true),
        (&bag, GEOMETRY_BAG_CLASS_ID, false),
        (&patch, MULTIPATCH_CLASS_ID, false),
    ] {
        let mut input = bytes.clone();
        input.splice(GEOMETRY..GEOMETRY + 16, slot_bytes.iter().copied());
        let mut reader = Reader::new(&input);
        let mut refs = RefTable10_1::new();
        if is_reference {
            refs.register(KEY);
        }
        assert_eq!(reader.read_u32_le().unwrap(), 1);
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        let (slot, body) = map.post_illumination_geometry.unwrap();
        assert_eq!(slot.offset, GEOMETRY);
        assert!(map.post_illumination_null_geometry.is_none());
        assert_eq!(
            map.post_illumination_null_border.unwrap().offset,
            GEOMETRY + slot_bytes.len()
        );
        assert!(refs.contains(KEY));
        match body {
            Some(MapGeometryBody10_1::Geometry(GeometryBody10_1::Point(value))) => {
                assert_eq!(
                    slot.entry,
                    ObjectEntry10_1::NewObject { class_id, key: KEY }
                );
                assert_eq!((value.record.x, value.record.y), (Some(2.5), Some(4.0)));
            }
            Some(MapGeometryBody10_1::Envelope(_)) => {
                assert_eq!(
                    slot.entry,
                    ObjectEntry10_1::NewObject { class_id, key: KEY }
                );
                assert_eq!(class_id, ENVELOPE_CLASS_ID);
            }
            Some(MapGeometryBody10_1::Geometry(GeometryBody10_1::GeometryBag(value))) => {
                assert_eq!(class_id, GEOMETRY_BAG_CLASS_ID);
                assert!(value.members.is_empty());
            }
            Some(MapGeometryBody10_1::Geometry(GeometryBody10_1::MultiPatch(value))) => {
                assert_eq!(class_id, MULTIPATCH_CLASS_ID);
                assert_eq!(value.record_bytes, [0; 4]);
            }
            None if is_reference => {
                assert_eq!(
                    slot.entry,
                    ObjectEntry10_1::BackReference { class_id, key: KEY }
                );
            }
            other => panic!("非预期几何体：{other:?}"),
        }
        assert_eq!(map.stop, MapSuffixStop::Complete);
        assert_eq!(reader.position(), 0x25569 + slot_bytes.len() - 16);
    }

    let unknown = header([0x41; 16]);
    for invalid in [&unknown, &point[..point.len() - 1]] {
        let mut input = bytes[..GEOMETRY].to_vec();
        input.extend(invalid);
        let mut reader = Reader::new(&input);
        let mut refs = RefTable10_1::new();
        assert_eq!(reader.read_u32_le().unwrap(), 1);
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        assert_eq!(map.stop, MapSuffixStop::AfterIlluminationFields);
        assert!(map.post_illumination_geometry.is_none());
        assert!(map.post_illumination_border.is_none());
        assert_eq!(reader.position(), GEOMETRY);
        assert!(!refs.contains(KEY));
    }
}

#[test]
fn geometry_probe_preserves_unknown_and_rejects_truncation() {
    let bytes = [0_u8; 16];
    for version in [23, 34, 51] {
        let mut reader = Reader::new(&bytes);
        assert!(
            read_post_illumination_null_geometry_10_1(&mut reader, version)
                .unwrap()
                .is_some()
        );
        assert_eq!(reader.position(), 16);
    }
    for n in 0..16 {
        let mut reader = Reader::new(&bytes[..n]);
        assert!(read_post_illumination_null_geometry_10_1(&mut reader, 51).is_err());
        assert_eq!(reader.position(), 0);
    }
    let mut non_null = [0_u8; 16];
    non_null[0] = 1;
    let mut reader = Reader::new(&non_null);
    assert!(
        read_post_illumination_null_geometry_10_1(&mut reader, 51)
            .unwrap()
            .is_none()
    );
    assert_eq!(reader.position(), 0);
    for version in [22, 52] {
        let mut reader = Reader::new(&bytes);
        assert!(read_post_illumination_null_geometry_10_1(&mut reader, version).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn border_tail_stops_at_each_incomplete_field_or_unknown_object() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let start = 0x25014;
    let widths = [2, 16, 2, 16, 2, 2, 8];
    let tail = &mut bytes[start..start + 48];
    tail.fill(0);
    tail[0..2].copy_from_slice(&0x8123_u16.to_le_bytes());
    tail[18..20].copy_from_slice(&0x4567_u16.to_le_bytes());
    tail[36..38].copy_from_slice(&0xabcd_u16.to_le_bytes());
    tail[38..40].copy_from_slice(&0xfedc_u16.to_le_bytes());
    tail[40..48].copy_from_slice(&0x7ff8_0123_4567_89ab_u64.to_le_bytes());
    let parse = |input: &[u8]| {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        reader.read_u32_le().unwrap();
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        (map, reader.position())
    };
    let (map, pos) = parse(&bytes[..start + 48]);
    assert_eq!(pos, start + 48);
    let tail = map.post_border_tail;
    assert_eq!(tail.short_308, Some((start, 0x8123)));
    assert_eq!(tail.short_310, Some((start + 18, 0x4567)));
    assert_eq!(tail.short_318, Some((start + 36, 0xabcd)));
    assert_eq!(tail.short_31a, Some((start + 38, 0xfedc)));
    assert_eq!(tail.double_bits, Some((start + 40, 0x7ff8_0123_4567_89ab)));
    let mut offset = start;
    for (stage, width) in widths.into_iter().enumerate() {
        for length in 0..width {
            let (map, pos) = parse(&bytes[..offset + length]);
            assert_eq!(pos, offset, "stage {stage}, length {length}");
            assert_eq!(map.remaining_bytes, length);
        }
        offset += width;
    }
    for offset in [start + 2, start + 20] {
        let mut nonempty = bytes.clone();
        nonempty[offset] = 1;
        let (_, pos) = parse(&nonempty);
        assert_eq!(pos, offset);
    }
}

#[test]
fn transformation_prefix_preserves_each_boundary_and_unknown_objects() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let start = 0x25044;
    bytes.truncate(start);
    bytes.extend(0x81234567_u32.to_le_bytes());
    bytes.extend(0xfedcba98_u32.to_le_bytes());
    bytes.extend(0xabcd_u16.to_le_bytes());
    bytes.extend(0x7ff8_0123_4567_89ab_u64.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(0x87654321_u32.to_le_bytes());
    bytes.extend([0xa5; 20]);
    let parse = |input: &[u8]| {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        reader.read_u32_le().unwrap();
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        (map, reader.position(), refs)
    };
    let (map, pos, _) = parse(&bytes);
    let tail = map.post_border_tail;
    assert_eq!(tail.int_1a4, Some((start, 0x81234567)));
    assert_eq!(tail.int_1a8, Some((start + 4, 0xfedcba98)));
    assert_eq!(tail.temporary_short, Some((start + 8, 0xabcd)));
    assert_eq!(
        tail.temporary_double_bits,
        Some((start + 10, 0x7ff8_0123_4567_89ab))
    );
    assert_eq!(tail.transformation_count, Some((start + 18, 1)));
    assert_eq!(
        tail.first_transformation_prefix,
        Some((start + 22, 0x87654321))
    );
    assert_eq!(
        tail.transformations[0].direction_on_disk(),
        Some(MapTransformationDirection10_1::Unknown(0x87654321))
    );
    assert_eq!(pos, start + 26);
    assert_eq!(map.remaining_bytes, 20);
    let mut offset = start;
    for width in [4, 4, 2, 8, 4, 4] {
        for length in 0..width {
            let (map, pos, _) = parse(&bytes[..offset + length]);
            assert_eq!(pos, offset);
            assert_eq!(map.remaining_bytes, length);
        }
        offset += width;
    }
    for invalid in [-1_i32, i32::MAX] {
        let mut bad = bytes.clone();
        bad[start + 18..start + 22].copy_from_slice(&invalid.to_le_bytes());
        let (map, pos, _) = parse(&bad);
        assert_eq!(pos, start + 18);
        assert!(map.post_border_tail.transformation_count.is_none());
    }
    let mut empty = bytes.clone();
    empty[start + 18..start + 22].copy_from_slice(&0_i32.to_le_bytes());
    empty.truncate(start + 22);
    let (map, pos, _) = parse(&empty);
    assert_eq!(pos, start + 22);
    assert_eq!(
        map.post_border_tail.transformation_count,
        Some((start + 18, 0))
    );
    assert!(map.post_border_tail.first_transformation_prefix.is_none());
}

#[test]
fn nadcon_body_versions_lengths_and_truncation_are_transactional() {
    for version in 0_u16..=2 {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend([0xff, 0, b'A']);
        let mut reader = Reader::new(&bytes);
        let body = read_nadcon_transformation_10_1(&mut reader, 3).unwrap();
        assert_eq!(body.version, version);
        assert_eq!(body.text, [0xff, 0, b'A']);
        assert_eq!(reader.position(), bytes.len());
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_nadcon_transformation_10_1(&mut reader, 3).is_err());
            assert_eq!(reader.position(), 0);
        }
    }
    for (version, length, max) in [
        (3_u16, 0_u32, 9_usize),
        (0xffff, 0, 9),
        (1, 4, 3),
        (1, 3072, 4000),
    ] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(length.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert!(read_nadcon_transformation_10_1(&mut reader, max).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn transformation_loop_handles_null_backref_and_failed_new_body() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes.truncate(0x25056);
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(11_u32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(22_u32.to_le_bytes());
    bytes.extend(NADCON_TRANSFORMATION_CLASS_ID);
    bytes.extend(50000_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(3_u32.to_le_bytes());
    bytes.extend(b"abc");
    bytes.extend(33_u32.to_le_bytes());
    bytes.extend(NADCON_TRANSFORMATION_CLASS_ID);
    bytes.extend(50000_u32.to_le_bytes());
    let end = bytes.len();
    bytes.extend([0xaa; 4]);
    let parse = |input: &[u8]| {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        reader.read_u32_le().unwrap();
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        (map, reader.position(), refs)
    };
    let (map, pos, refs) = parse(&bytes);
    assert_eq!(pos, end);
    assert_eq!(map.stop, MapSuffixStop::AfterBorderTail { stage: 14 });
    let items = &map.post_border_tail.transformations;
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].object.unwrap().entry, ObjectEntry10_1::Null);
    assert_eq!(items[1].body.as_ref().unwrap().text, b"abc");
    assert!(matches!(
        items[2].object.unwrap().entry,
        ObjectEntry10_1::BackReference { key: 50000, .. }
    ));
    assert!(refs.contains(50000));
    let second_header = items[1].object.unwrap().offset;
    let second_end = items[2].offset;
    let third_header = items[2].object.unwrap().offset;
    let mut alternate = bytes.clone();
    for header in [second_header, third_header] {
        alternate[header..header + 16].copy_from_slice(&GEO_TRANSFORMATION_CLASS_ID);
    }
    let (alt, alt_end, _) = parse(&alternate);
    assert_eq!(alt_end, end);
    let items = &alt.post_border_tail.transformations;
    assert_eq!(items[1].body.as_ref().unwrap().text, b"abc");
    assert_eq!(
        items[2].object.unwrap().entry,
        ObjectEntry10_1::BackReference {
            class_id: GEO_TRANSFORMATION_CLASS_ID,
            key: 50000,
        }
    );
    let mut composite = bytes.clone();
    for header in [second_header, third_header] {
        composite[header..header + 16].copy_from_slice(&COMPOSITE_GEO_TRANSFORMATION_CLASS_ID);
    }
    let mut composite_body = 1_u16.to_le_bytes().to_vec();
    composite_body.extend(2_u16.to_le_bytes());
    composite_body.extend(('组' as u16).to_le_bytes());
    composite_body.extend(1_u16.to_le_bytes());
    composite_body.extend((-1_i16).to_le_bytes());
    composite_body.extend(NTV2_TRANSFORMATION_CLASS_ID);
    composite_body.extend(&bytes[second_header + 20..second_end]);
    composite.splice(second_header + 20..second_end, composite_body);
    let (map, pos, refs) = parse(&composite);
    assert_eq!(pos, composite.len() - 4);
    assert_eq!(map.stop, MapSuffixStop::AfterBorderTail { stage: 14 });
    assert!(refs.contains(50000));
    let items = &map.post_border_tail.transformations;
    let body = items[1].body.as_ref().unwrap().composite.as_ref().unwrap();
    assert_eq!(body.name, "组");
    assert_eq!(body.items.len(), 1);
    assert_eq!(body.items[0].direction, -1);
    assert_eq!(body.items[0].class_id, NTV2_TRANSFORMATION_CLASS_ID);
    assert_eq!(body.items[0].body.text, b"abc");
    assert!(matches!(
        items[2].object.unwrap().entry,
        ObjectEntry10_1::BackReference { key: 50000, .. }
    ));
    for cut in second_header..second_end {
        let (map, pos, refs) = parse(&bytes[..cut]);
        assert_eq!(pos, second_header);
        assert!(!refs.contains(50000));
        assert!(map.post_border_tail.transformations[1].object.is_none());
    }
    let mut unknown = bytes.clone();
    unknown[second_header] ^= 0xff;
    let (map, pos, refs) = parse(&unknown);
    assert_eq!(pos, second_header);
    assert!(!refs.contains(50000));
    assert!(map.post_border_tail.transformations[1].object.is_none());
}

#[test]
fn post_transformation_tail_preserves_bits_and_each_boundary() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let start = 0x2524f;
    bytes.truncate(start);
    bytes.extend(0x7ff8_1234_5678_9abc_u64.to_le_bytes());
    bytes.extend(0xabcd_u16.to_le_bytes());
    bytes.extend([0; 32]);
    let parse = |input: &[u8]| {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        reader.read_u32_le().unwrap();
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        (map, reader.position(), refs)
    };
    let (map, pos, _) = parse(&bytes);
    let tail = map.post_border_tail;
    assert_eq!(
        tail.post_transform_double_bits,
        Some((start, 0x7ff8_1234_5678_9abc))
    );
    assert_eq!(tail.post_transform_short, Some((start + 8, 0xabcd)));
    assert_eq!(
        tail.post_transform_null_background.unwrap().offset,
        start + 10
    );
    assert_eq!(tail.post_transform_null_object.unwrap().offset, start + 26);
    assert_eq!(pos, start + 42);
    let mut offset = start;
    for width in [8, 2, 16, 16] {
        for len in 0..width {
            let (map, pos, _) = parse(&bytes[..offset + len]);
            assert_eq!(pos, offset, "offset {offset:x} len {len}");
            assert_eq!(map.remaining_bytes, len);
        }
        offset += width;
    }
    for offset in [start + 10, start + 26] {
        let mut input = bytes.clone();
        input[offset] = 1;
        let (_, pos, refs) = parse(&input);
        let (_, _, baseline_refs) = parse(&bytes[..offset]);
        assert_eq!(pos, offset);
        assert_eq!(refs.len(), baseline_refs.len());
        for key in 0..=50_000 {
            assert_eq!(refs.contains(key), baseline_refs.contains(key));
        }
    }
}

#[test]
fn basic_overposter_body_versions_and_failure_are_transactional() {
    use mxd2qgz::maps::map_object::read_basic_overposter_properties_10_1;
    use mxd2qgz::objects::display_color::ColorSlot10_1;

    for version in [0_u16, 1] {
        let bytes = version.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_basic_overposter_properties_10_1(&mut reader, 100, &mut refs).unwrap();
        assert_eq!(body.version, version);
        assert!(body.color.is_none());
        assert_eq!(reader.position(), 2);
    }
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let body = &bytes[0x2527d..0x252be];
    let mut reader = Reader::new(body);
    let mut refs = RefTable10_1::new();
    refs.register(12345);
    let parsed = read_basic_overposter_properties_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    assert_eq!(parsed.version, 2);
    assert!(matches!(parsed.color, Some(ColorSlot10_1::Color { .. })));
    assert_eq!(reader.position(), body.len());
    assert!(refs.contains(2181));
    for end in 0..body.len() {
        let mut reader = Reader::new(&body[..end]);
        let mut refs = RefTable10_1::new();
        refs.register(12345);
        assert!(read_basic_overposter_properties_10_1(&mut reader, 4_000_000, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(12345));
    }
    let mut unknown = body.to_vec();
    unknown[4] ^= 0x80;
    let mut reader = Reader::new(&unknown);
    let mut refs = RefTable10_1::new();
    assert!(read_basic_overposter_properties_10_1(&mut reader, 4_000_000, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
    for version in [3_u16, 0xffff] {
        let bytes = version.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        assert!(read_basic_overposter_properties_10_1(&mut reader, 100, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn basic_overposter_host_failure_does_not_register_nested_keys() {
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    for end in 0x25269..0x252be {
        let mut reader = Reader::new(&bytes[..end]);
        let mut refs = RefTable10_1::new();
        reader.read_u32_le().unwrap();
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        assert_eq!(reader.position(), 0x25269);
        assert!(map.post_border_tail.post_transform_overposter.is_none());
        assert!(!refs.contains(2180));
        assert!(!refs.contains(2181));
    }
}

#[test]
fn basic_overposter_body_accepts_null_and_known_color_reference() {
    use mxd2qgz::maps::map_object::read_basic_overposter_properties_10_1;
    use mxd2qgz::objects::display_color::{ColorSlot10_1, RGB_COLOR_CLASS_ID};

    for is_reference in [false, true] {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend(0x8123_u16.to_le_bytes());
        if is_reference {
            bytes.extend(RGB_COLOR_CLASS_ID);
            bytes.extend(42_u32.to_le_bytes());
        } else {
            bytes.extend([0; 16]);
        }
        bytes.extend(0xabcd_u16.to_le_bytes());
        bytes.extend(0x7ff8_1234_5678_9abc_u64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        if is_reference {
            refs.register(42);
        }
        let body = read_basic_overposter_properties_10_1(&mut reader, 1000, &mut refs).unwrap();
        assert_eq!(body.short_before_color, Some(0x8123));
        assert_eq!(body.short_after_color, Some(0xabcd));
        assert_eq!(body.double_bits, Some(0x7ff8_1234_5678_9abc));
        assert!(
            matches!(body.color, Some(ColorSlot10_1::Null))
                || matches!(
                    body.color,
                    Some(ColorSlot10_1::BackReference { key: 42, .. })
                )
        );
        assert_eq!(reader.position(), bytes.len());
    }
}

#[test]
fn maplex_overposter_reads_dictionaries_and_number_groups_in_map_object() {
    use mxd2qgz::maps::maplex::{
        MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID, MaplexDictionaryEntries10_1,
    };

    fn header(bytes: &mut Vec<u8>, class: [u8; 16], key: u32) {
        bytes.extend(class);
        bytes.extend(key.to_le_bytes());
    }
    fn string(bytes: &mut Vec<u8>, value: &str) {
        let units: Vec<_> = value.encode_utf16().chain([0]).collect();
        bytes.extend((units.len() as u32 * 2).to_le_bytes());
        bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
    }

    let dictionaries = [
        8, 0x48, 0x66, 0x20, 0xdb, 0x56, 0x28, 0xbd, 0x67, 0xdc, 2, 0xe3, 0x3d, 0xec, 0xf3, 0x21,
    ];
    let dictionary = [
        8, 0x48, 0x66, 0x20, 0xe2, 0x3d, 0xd1, 0x1c, 0x8a, 9, 8, 0xe0, 0xd, 0xec, 0x73, 0x21,
    ];
    let entry = [
        8, 0x48, 0x66, 0x20, 0x45, 0xfe, 0x18, 0x90, 0xbe, 0x1f, 0x66, 0xeb, 0xd, 0xec, 0x73, 0x21,
    ];
    let groups = [
        0xb8, 0x3b, 0x5f, 0x75, 0xfb, 0x63, 0x4b, 0xe5, 0x88, 0x4c, 0x74, 0xf3, 0xdd, 0x2a, 0x8a,
        0x81,
    ];
    let group = [
        0x5d, 0xef, 0xdd, 0x66, 0xb3, 0x1a, 0x31, 0x5f, 0x7b, 0xc1, 0x75, 0x5d, 0xde, 0xf3, 0x3d,
        0x32,
    ];
    let mut maplex = Vec::new();
    header(&mut maplex, MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID, 4001);
    maplex.extend(7_i16.to_le_bytes());
    maplex.extend((-10_i32).to_le_bytes());
    maplex.extend(11_u16.to_le_bytes());
    header(&mut maplex, dictionaries, 4002);
    maplex.extend(1_i16.to_le_bytes());
    let dict_count = maplex.len();
    maplex.extend(2_i32.to_le_bytes());
    header(&mut maplex, dictionary, 4003);
    maplex.extend(1_i16.to_le_bytes());
    maplex.extend(1_i32.to_le_bytes());
    for letter in [b'A', b'B'] {
        maplex.extend(1_u32.to_le_bytes()); // BSTR 长度是码元数。
        maplex.extend([letter, 0, 0, 0]);
    }
    header(&mut maplex, dictionary, 4004);
    maplex.extend(2_i16.to_le_bytes());
    maplex.extend(2_i32.to_le_bytes());
    header(&mut maplex, entry, 4005);
    maplex.extend(1_i16.to_le_bytes());
    string(&mut maplex, "路径");
    string(&mut maplex, "路");
    maplex.extend((-16_i32).to_le_bytes());
    header(&mut maplex, entry, 4005);
    string(&mut maplex, "词典");
    maplex.extend(20_u16.to_le_bytes());
    maplex.extend(21_i32.to_le_bytes());
    maplex.extend(2.5_f64.to_le_bytes());
    maplex.extend([0; 16]);
    maplex.extend([22_u16, 23, 24].into_iter().flat_map(u16::to_le_bytes));
    header(&mut maplex, groups, 4006);
    maplex.extend(1_i16.to_le_bytes());
    maplex.extend(1_i32.to_le_bytes());
    header(&mut maplex, group, 4007);
    maplex.extend(1_i16.to_le_bytes());
    string(&mut maplex, "编号组");
    maplex.extend([25_i32, 26, 27].into_iter().flat_map(i32::to_le_bytes));
    string(&mut maplex, "后缀");
    maplex.extend(28_i32.to_le_bytes());

    let mut file = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    file.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let start = 0x25269;
    let original_end = {
        // BasicOverposterProperties 的边界由正常 Map 解析确认。
        let mut reader = Reader::new(&bytes[24..0x252be]);
        let result =
            read_map_suffix_10_1(&mut reader, 4_000_000, &mut RefTable10_1::new()).unwrap();
        assert!(result.post_border_tail.post_transform_overposter.is_some());
        reader.position() + 24
    };
    let maplex_end = start + maplex.len();
    bytes.splice(start..original_end, maplex);
    let mut reader = Reader::new(&bytes[24..]);
    let mut refs = RefTable10_1::new();
    let parsed = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    let body = parsed.post_border_tail.post_transform_maplex.unwrap().1;
    assert_eq!(body.version, 7);
    assert_eq!(body.v4_v5_long, None);
    assert_eq!(body.v5_shorts, Some([23, 24]));
    assert_eq!(body.v3.unwrap().double_24, 2.5_f64.to_bits());
    let dictionaries = body.dictionaries.unwrap().body.unwrap();
    let items: Vec<_> = dictionaries
        .items
        .into_iter()
        .map(|item| item.body.unwrap().entries)
        .collect();
    assert!(
        matches!(&items[0], MaplexDictionaryEntries10_1::LegacyArrays { first, second } if first == &["A"] && second == &["B"])
    );
    let MaplexDictionaryEntries10_1::Objects { items, name } = &items[1] else {
        panic!("缺少对象式词典");
    };
    assert_eq!(name, "词典");
    assert_eq!(items[0].body.as_ref().unwrap().strings, ["路径", "路"]);
    assert!(matches!(
        items[1].entry,
        ObjectEntry10_1::BackReference { key: 4005, .. }
    ));
    let groups = body.key_number_groups.unwrap().body.unwrap();
    assert_eq!(groups.items[0].body.as_ref().unwrap().longs, [25, 26, 27]);
    assert_eq!(groups.items[0].body.as_ref().unwrap().second_string, "后缀");
    assert_eq!(
        parsed.post_border_tail.short_32a.unwrap().0 + 24,
        maplex_end
    );
    assert!((4001..=4007).all(|key| refs.contains(key)));

    let mut bad_count = bytes.clone();
    bad_count[start + dict_count..start + dict_count + 4].copy_from_slice(&i32::MAX.to_le_bytes());
    for input in [&bytes[..maplex_end - 1], bad_count.as_slice()] {
        let mut reader = Reader::new(&input[24..]);
        let mut refs = RefTable10_1::new();
        let parsed = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        assert!(parsed.post_border_tail.post_transform_maplex.is_none());
        assert_eq!(reader.position() + 24, start);
        assert!((4001..=4007).all(|key| !refs.contains(key)));
    }
}

#[test]
fn border_tail_reads_new_feature_layer_before_following_fields() {
    use mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
    use mxd2qgz::layers::layer_list::LayerBodyPrefix10_1;

    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let start = 0x25016;
    let key = 60000_u32;
    let mut layer = FEATURE_LAYER_CLASS_ID.to_vec();
    layer.extend(key.to_le_bytes());
    layer.extend(1_u16.to_le_bytes());
    let empty_string = [2, 0, 0, 0, 0, 0];
    layer.extend(empty_string);
    layer.extend(empty_string);
    layer.extend([0; 6 + 16 + 16 + 16]);
    layer.extend(empty_string);
    layer.extend([0; 16 + 2 + 4 + 16]);
    layer.extend(empty_string);
    layer.extend(empty_string);
    layer.extend([0; 8 + 16 + 2 + 16 + 4 + 2]);
    let end = start + layer.len();
    bytes.splice(start..start + 16, layer);
    let parse = |bytes: &[u8]| {
        let mut reader = Reader::new(bytes);
        let mut refs = RefTable10_1::new();
        reader.read_u32_le().unwrap();
        read_object_entry_10_1(&mut reader, &mut refs).unwrap();
        let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
        (map, reader.position(), refs.contains(key))
    };
    let (map, _, registered) = parse(&bytes);
    assert!(registered);
    assert!(matches!(
        map.post_border_tail.layer_30c_parsed.as_deref(),
        Some(LayerBodyPrefix10_1::FeatureLayer(_))
    ));
    assert_eq!(map.post_border_tail.short_310.unwrap().0, end);

    let (map, offset, registered) = parse(&bytes[..end - 1]);
    assert!(!registered);
    assert_eq!(offset, start);
    assert!(map.post_border_tail.layer_30c.is_none());
}
