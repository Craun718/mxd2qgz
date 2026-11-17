use mxd2qgz::maps::map_object::{
    MAP_CLASS_ID, MapSuffixStop, NADCON_TRANSFORMATION_CLASS_ID, read_map_suffix_10_1,
    read_nadcon_transformation_10_1, read_post_illumination_null_geometry_10_1,
};
use mxd2qgz::maps::standalone_table::STANDALONE_TABLE_CLASS_ID;
use mxd2qgz::objects::geo_transformation::GEO_TRANSFORMATION_CLASS_ID;
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
    assert_eq!(reader.position(), 0x253be);
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
    let second_end = items[2].prefix_offset;
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
