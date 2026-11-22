use mxd2qgz::layers::dep_tail::{TIME_CLASS_ID, TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID};
use mxd2qgz::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID;
use mxd2qgz::maps::standalone_table::{
    StandaloneNameBody10_1, StandaloneTableStop10_1, StandaloneTableTailBody10_1,
    read_standalone_table_prefix_10_1,
};
use mxd2qgz::objects::dataset_name_more::QUERY_TABLE_NAME_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn string(out: &mut Vec<u8>) {
    out.extend(2u32.to_le_bytes());
    out.extend([0, 0]);
}
fn name(class: [u8; 16], nested_unknown: bool) -> Vec<u8> {
    let mut b = class.to_vec();
    b.extend(7u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    let feature = class == FEATURE_CLASS_NAME_CLASS_ID;
    for _ in 0..if feature { 4 } else { 3 } {
        string(&mut b);
    }
    if feature {
        b.extend([0; 8]);
        b.extend(0u16.to_le_bytes());
    }
    if nested_unknown {
        b.extend([5; 16]);
        b.extend(8u32.to_le_bytes());
    } else {
        b.extend([0; 16]);
    }
    if !feature && !nested_unknown {
        b.extend([0; 16]);
    }
    b
}
fn prefix(class: [u8; 16]) -> Vec<u8> {
    let mut b = 3u16.to_le_bytes().to_vec();
    b.extend(name(class, false));
    b.extend(0u32.to_le_bytes());
    string(&mut b);
    b.extend(0u32.to_le_bytes());
    b.extend([0; 16]);
    b.extend(1u32.to_le_bytes());
    b.extend(0x12345678u32.to_le_bytes());
    b
}
#[test]
fn name_classes_reach_end_after_sequence() {
    for class in [FEATURE_CLASS_NAME_CLASS_ID, QUERY_TABLE_NAME_CLASS_ID] {
        let data = prefix(class);
        let mut r = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
        assert!(matches!(
            p.dataset_name_body,
            Some(StandaloneNameBody10_1::FeatureClassName(_))
                | Some(StandaloneNameBody10_1::QueryTableName(_))
        ));
        assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
        assert_eq!(p.sequence_count, Some(1));
        assert_eq!(p.sequence_values, [0x12345678]);
        assert_eq!(r.position(), data.len());
        assert!(refs.contains(7));
    }
}
#[test]
fn nested_unknown_stops_and_truncation_rolls_back() {
    for class in [FEATURE_CLASS_NAME_CLASS_ID, QUERY_TABLE_NAME_CLASS_ID] {
        let mut data = 3u16.to_le_bytes().to_vec();
        data.extend(name(class, true));
        let mut r = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
        assert_eq!(p.stop, StandaloneTableStop10_1::NameBody);
        assert_eq!(r.position(), data.len() - 20);
        assert!(refs.contains(7));
        assert!(!refs.contains(8));
        let data = prefix(class);
        for end in 0..data.len() {
            let mut r = Reader::new(&data[..end]);
            let mut refs = RefTable10_1::new();
            refs.register(99);
            assert!(
                read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).is_err(),
                "{end}"
            );
            assert_eq!(r.position(), 0);
            assert_eq!(refs.len(), 1);
        }
    }
}

#[test]
fn nonpositive_counts_skip_fields_relationships_and_sequence_tokens() {
    for count in [i32::MIN, -1, 0] {
        let mut data = 5_u16.to_le_bytes().to_vec();
        data.extend([0; 16]);
        data.extend(count.to_le_bytes());
        string(&mut data);
        data.extend(count.to_le_bytes());
        data.extend([0; 16]);
        data.extend(count.to_le_bytes());
        data.extend(0x12345678_i32.to_le_bytes());
        data.extend([0; 16]);
        let end = data.len();
        data.push(0xaa);
        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let table = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 0, 100).unwrap();
        assert_eq!(table.stop, StandaloneTableStop10_1::Complete);
        assert!(table.named_objects.is_empty());
        assert!(table.relationships.is_empty());
        assert_eq!(table.sequence_count, Some(count));
        assert!(table.sequence_values.is_empty());
        assert_eq!(table.raw_int32, Some(0x12345678));
        assert_eq!(reader.position(), end);
        assert!(refs.is_empty());
    }
}

#[test]
fn sequence_color_reference_uses_registered_class() {
    use mxd2qgz::objects::display_color::RGB_COLOR_CLASS_ID;
    use mxd2qgz::storage::object_refs::{ObjectEntry10_1, read_object_entry_10_1};

    let mut prefix = 3_u16.to_le_bytes().to_vec();
    prefix.extend([0; 16 + 4]);
    string(&mut prefix);
    prefix.extend(0_u32.to_le_bytes());
    let slot = prefix.len();

    for (registered, claimed) in [
        (RGB_COLOR_CLASS_ID, [0x7d; 16]),
        ([0x7d; 16], RGB_COLOR_CLASS_ID),
    ] {
        let mut refs = RefTable10_1::new();
        let mut header = registered.to_vec();
        header.extend(90_u32.to_le_bytes());
        read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
        let before = refs.clone();
        let mut bytes = prefix.clone();
        bytes.extend(claimed);
        bytes.extend(90_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(7_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let table = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 10, 100).unwrap();
        assert_eq!(refs, before);
        if registered == RGB_COLOR_CLASS_ID {
            assert_eq!(table.stop, StandaloneTableStop10_1::Complete);
            assert_eq!(
                table.sequence_object,
                Some(ObjectEntry10_1::BackReference {
                    class_id: registered,
                    key: 90,
                })
            );
            assert_eq!(table.sequence_values, [7]);
            assert_eq!(reader.position(), bytes.len());
        } else {
            assert_eq!(
                table.stop,
                StandaloneTableStop10_1::SequenceObject {
                    class_id: registered,
                    key: 90,
                }
            );
            assert_eq!(reader.position(), slot);
        }
    }
}

#[test]
fn compressed_sequence_checks_expansion_and_overflow() {
    let mut b = 3u16.to_le_bytes().to_vec();
    b.extend([0; 16]);
    b.extend(0u32.to_le_bytes());
    string(&mut b);
    b.extend(0u32.to_le_bytes());
    b.extend([0; 16]);
    b.extend(4u32.to_le_bytes());
    let tokens = b.len();
    b.extend((-1i32).to_le_bytes());
    b.extend(9i32.to_le_bytes());
    let mut r = Reader::new(&b);
    let mut refs = RefTable10_1::new();
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
    assert_eq!(p.sequence_tokens, [-1, 9]);
    assert_eq!(p.sequence_values, [-1, 0, 1, 9]);
    for token in [-5i32, i32::MIN] {
        let mut bad = b.clone();
        bad[tokens..tokens + 4].copy_from_slice(&token.to_le_bytes());
        let mut r = Reader::new(&bad);
        let mut refs = RefTable10_1::new();
        refs.register(77);
        assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).is_err());
        assert_eq!(r.position(), 0);
        assert_eq!(refs.len(), 1);
    }
}
#[test]
fn relationship_unknown_preserves_header_and_prior_keys() {
    let mut b = 2u16.to_le_bytes().to_vec();
    b.extend(name(QUERY_TABLE_NAME_CLASS_ID, false));
    b.extend(0u32.to_le_bytes());
    string(&mut b);
    b.extend(1u32.to_le_bytes());
    let offset = b.len();
    b.extend([6; 16]);
    b.extend(9u32.to_le_bytes());
    let mut r = Reader::new(&b);
    let mut refs = RefTable10_1::new();
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
    assert!(matches!(
        p.stop,
        StandaloneTableStop10_1::RelationshipObject { key: 9, .. }
    ));
    assert_eq!(r.position(), offset);
    assert!(refs.contains(7));
    assert!(!refs.contains(9));
}

#[test]
fn standalone_reads_query_name_relationship_and_color_before_sequence() {
    use mxd2qgz::objects::dataset_name_more::{
        DatasetNameMoreBody10_1, RELATIONSHIP_CLASS_NAME_CLASS_ID, TABLE_QUERY_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::display_color::{ColorBody10_1, RGB_COLOR_CLASS_ID};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut b = 3u16.to_le_bytes().to_vec();
    b.extend(TABLE_QUERY_NAME_CLASS_ID);
    b.extend(9u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    for _ in 0..3 {
        string(&mut b);
    }
    b.extend([0; 16]);
    for _ in 0..3 {
        string(&mut b);
    }
    b.extend(0u32.to_le_bytes());
    string(&mut b);
    b.extend(2u32.to_le_bytes());
    b.extend(RELATIONSHIP_CLASS_NAME_CLASS_ID);
    b.extend(10u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    for _ in 0..3 {
        string(&mut b);
    }
    b.extend([0; 8 + 6 + 16]);
    b.extend(RELATIONSHIP_CLASS_NAME_CLASS_ID);
    b.extend(10u32.to_le_bytes());
    b.extend(RGB_COLOR_CLASS_ID);
    b.extend(11u32.to_le_bytes());
    b.extend([1, 0, 1, 0, 1, 1, 2, 3, 4, 0, 0]);
    b.extend(1u32.to_le_bytes());
    b.extend(17u32.to_le_bytes());

    let mut r = Reader::new(&b);
    let mut refs = RefTable10_1::new();
    let body = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
    assert_eq!(body.stop, StandaloneTableStop10_1::Complete);
    assert!(matches!(
        body.dataset_name_body,
        Some(StandaloneNameBody10_1::OtherTabularName(name))
            if matches!(*name, DatasetNameMoreBody10_1::TableQuery(_))
    ));
    assert!(matches!(
        body.relationship_bodies[0],
        Some(DatasetNameMoreBody10_1::RelationshipClass(_))
    ));
    assert!(matches!(
        body.relationships[1],
        ObjectEntry10_1::BackReference { key: 10, .. }
    ));
    assert!(matches!(
        body.sequence_color,
        Some(ColorBody10_1::Rgb(color)) if color.key == 11
    ));
    assert_eq!(body.sequence_values, [17]);
    assert_eq!(r.remaining(), 0);
    assert!((9..=11).all(|key| refs.contains(key)));

    let mut r = Reader::new(&b[..b.len() - 1]);
    let mut refs = RefTable10_1::new();
    assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).is_err());
    assert_eq!(r.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn tail_references_resolve_against_the_registered_class() {
    use mxd2qgz::layers::annotate_props::{
        ANNOTATE_PROPS_COLLECTION_CLASS_ID, DISPLAY_EXPRESSION_PROPS_CLASS_ID,
    };
    use mxd2qgz::layers::feature_layer_tail::UID_CLASS_ID;
    use mxd2qgz::storage::object_refs::{ObjectEntry10_1, read_object_entry_10_1};

    let mut prefix = 11_u16.to_le_bytes().to_vec();
    prefix.extend([0; 16 + 4]);
    string(&mut prefix);
    prefix.extend([0; 4 + 16 + 4 + 4 + 16]);
    string(&mut prefix);
    string(&mut prefix);
    prefix.extend(0_u32.to_le_bytes());
    string(&mut prefix);
    for (slot, expected) in [
        DISPLAY_EXPRESSION_PROPS_CLASS_ID,
        UID_CLASS_ID,
        ANNOTATE_PROPS_COLLECTION_CLASS_ID,
    ]
    .into_iter()
    .enumerate()
    {
        for registered in [expected, [0x7d; 16]] {
            let mut header = registered.to_vec();
            header.extend(90_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
            let before = refs.clone();
            let mut bytes = prefix.clone();
            let mut offset = 0;
            for index in 0..3 {
                if index == 1 {
                    bytes.extend(0_u16.to_le_bytes());
                }
                if index == slot {
                    offset = bytes.len();
                    let claimed = if registered == expected {
                        [0x7d; 16]
                    } else {
                        expected
                    };
                    bytes.extend(claimed);
                    bytes.extend(90_u32.to_le_bytes());
                } else {
                    bytes.extend([0; 16]);
                }
            }
            let mut reader = Reader::new(&bytes);
            let result =
                read_standalone_table_prefix_10_1(&mut reader, &mut refs, 10, 100).unwrap();
            assert_eq!(refs, before);
            if registered == expected {
                assert_eq!(result.stop, StandaloneTableStop10_1::Complete);
                assert_eq!(
                    result.tail_objects[slot],
                    ObjectEntry10_1::BackReference {
                        class_id: expected,
                        key: 90,
                    }
                );
                assert_eq!(reader.position(), bytes.len());
            } else {
                assert_eq!(
                    result.stop,
                    StandaloneTableStop10_1::TailObject {
                        slot: slot as u8,
                        class_id: registered,
                        key: 90,
                    }
                );
                assert_eq!(reader.position(), offset);
            }
        }
    }
}

#[test]
fn pre_join_negative_count_leaves_display_expression_aligned() {
    let mut data = 11_u16.to_le_bytes().to_vec();
    data.extend([0; 16 + 4]);
    string(&mut data);
    data.extend([0; 4 + 16 + 4 + 4 + 16]);
    string(&mut data);
    string(&mut data);
    let count = data.len();
    data.extend((-1_i32).to_le_bytes());
    let text = "表达式";
    let units: Vec<_> = text.encode_utf16().chain([0]).collect();
    data.extend(((units.len() * 2) as u32).to_le_bytes());
    data.extend(units.into_iter().flat_map(u16::to_le_bytes));
    data.extend([0; 16 + 2 + 16 + 16]);
    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let result = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 0, 100).unwrap();
    assert_eq!(result.pre_join, Some(Vec::new()));
    assert_eq!(
        result.version_strings.last().map(String::as_str),
        Some(text)
    );
    assert_eq!(result.stop, StandaloneTableStop10_1::Complete);
    assert_eq!(reader.position(), data.len());

    for len in count..data.len() {
        let mut reader = Reader::new(&data[..len]);
        assert!(read_standalone_table_prefix_10_1(&mut reader, &mut refs, 0, 100).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn empty_sequences_reach_version_boundaries_and_secondary_name() {
    use mxd2qgz::layers::annotate_props::{
        ANNOTATE_PROPS_COLLECTION_CLASS_ID, DISPLAY_EXPRESSION_PROPS_CLASS_ID,
    };
    use mxd2qgz::layers::feature_layer_tail::{TIME_LAYER_TRACK_EXTENSION_CLASS_ID, UID_CLASS_ID};

    for version in 0u16..=14 {
        let mut b = version.to_le_bytes().to_vec();
        b.extend([0; 16]);
        b.extend(0u32.to_le_bytes());
        string(&mut b);
        if version >= 2 {
            b.extend(0u32.to_le_bytes());
        }
        if version >= 3 {
            b.extend([0; 16]);
            b.extend(0u32.to_le_bytes());
        }
        if version >= 5 {
            b.extend(123u32.to_le_bytes());
        }
        if version >= 4 {
            b.extend(name(QUERY_TABLE_NAME_CLASS_ID, false));
        }
        if version >= 6 {
            string(&mut b);
        }
        if version == 7 {
            string(&mut b);
        }
        if version >= 9 {
            string(&mut b);
        }
        if version >= 10 {
            b.extend(1u32.to_le_bytes());
            string(&mut b);
            string(&mut b);
        }
        if version >= 11 {
            string(&mut b);
            b.extend(DISPLAY_EXPRESSION_PROPS_CLASS_ID);
            b.extend(41u32.to_le_bytes());
            b.extend(1u16.to_le_bytes());
            string(&mut b);
            b.extend(0xffffu16.to_le_bytes());
            b.extend([0; 16]);
            if version <= 12 {
                b.extend(0u16.to_le_bytes());
                b.extend(UID_CLASS_ID);
                b.extend(42u32.to_le_bytes());
                b.extend([0; 2 + 1 + 16 + 4]);
                b.extend(ANNOTATE_PROPS_COLLECTION_CLASS_ID);
                b.extend(43u32.to_le_bytes());
                b.extend(2u16.to_le_bytes());
                b.extend([0; 8]);
            }
        }
        let mut extent_offset = None;
        if version >= 12 {
            b.extend(2u16.to_le_bytes());
            b.extend(0u16.to_le_bytes());
            b.extend([0; 16]);
            b.extend(0u16.to_le_bytes());
            b.extend(1u16.to_le_bytes());
            for _ in 0..4 {
                string(&mut b);
            }
            extent_offset = Some(b.len());
            b.extend([0; 16]);
            b.extend(1u16.to_le_bytes());
            string(&mut b);
            string(&mut b);
            b.extend(1u16.to_le_bytes());
            b.extend(0u16.to_le_bytes());
            b.extend([0; 24]);
        }
        if version >= 14 {
            b.extend(0u32.to_le_bytes());
        }
        let expected = b.len();
        if let Some(offset) = extent_offset {
            let mut extent = TIME_EXTENT_CLASS_ID.to_vec();
            extent.extend(51u32.to_le_bytes());
            extent.extend(2u16.to_le_bytes());
            extent.extend(TIME_CLASS_ID);
            extent.extend(52u32.to_le_bytes());
            extent.extend(1u16.to_le_bytes());
            extent.extend(1i32.to_le_bytes());
            extent.extend(45.0_f64.to_le_bytes());
            extent.extend([0; 16]);
            extent.extend(TIME_REFERENCE_CLASS_ID);
            extent.extend(53u32.to_le_bytes());
            extent.extend(1u16.to_le_bytes());
            let zone_offset = extent.len();
            extent.extend([0; 16]);
            extent.extend([0; 4]);
            extent.push(1);
            let mut timed = b.clone();
            timed.splice(offset..offset + 16, extent);
            let mut r = Reader::new(&timed);
            let mut refs = RefTable10_1::new();
            let table = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
            assert_eq!(table.stop, StandaloneTableStop10_1::Complete);
            assert_eq!(r.remaining(), 0);
            assert!((51..=53).all(|key| refs.contains(key)));

            let zone = offset + zone_offset;
            timed[zone..zone + 16].fill(0x7d);
            timed[zone + 16..zone + 20].copy_from_slice(&54u32.to_le_bytes());
            let mut r = Reader::new(&timed);
            let mut refs = RefTable10_1::new();
            let table = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
            assert_eq!(table.stop, StandaloneTableStop10_1::TimeFamily);
            assert_eq!(r.position(), offset);
            assert!(refs.contains(41));
            assert!((51..=54).all(|key| !refs.contains(key)));
        }
        if version == 14 {
            for count in [i32::MIN, -1] {
                let mut negative = b.clone();
                negative[expected - 4..expected].copy_from_slice(&count.to_le_bytes());
                negative.push(0xaa);
                let mut reader = Reader::new(&negative);
                let mut refs = RefTable10_1::new();
                let table =
                    read_standalone_table_prefix_10_1(&mut reader, &mut refs, 10, 100).unwrap();
                assert_eq!(table.stop, StandaloneTableStop10_1::Complete);
                assert_eq!(table.extension_count, Some(count));
                assert!(table.extensions.is_empty());
                assert_eq!(reader.position(), expected);
                assert!(refs.contains(41));
            }
            let mut with_extension = b.clone();
            with_extension[expected - 4..].copy_from_slice(&2u32.to_le_bytes());
            let mut track = 1u16.to_le_bytes().to_vec();
            track.extend([
                0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0, 0xc0, 0x4f, 0x8e,
                0xce, 0x27,
            ]); // LongArray CLSID。
            track.extend(44u32.to_le_bytes());
            track.extend(1u16.to_le_bytes());
            track.extend(1u32.to_le_bytes());
            track.extend(12i32.to_le_bytes());
            track.extend(0u32.to_le_bytes());
            with_extension.extend((track.len() as u64).to_le_bytes());
            with_extension.extend(TIME_LAYER_TRACK_EXTENSION_CLASS_ID);
            with_extension.extend(track);
            let mut complete = with_extension.clone();
            complete[expected - 4..expected].copy_from_slice(&1u32.to_le_bytes());
            let mut r = Reader::new(&complete);
            let mut refs = RefTable10_1::new();
            let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
            assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
            let track = p.extensions[0].time_layer_track_extension.as_ref().unwrap();
            assert_eq!(track.long_array.body, Some((1, vec![12])));
            assert!(refs.contains(44));
            assert_eq!(r.remaining(), 0);

            with_extension[expected - 4..expected].copy_from_slice(&3_u32.to_le_bytes());
            let unknown_offset = with_extension.len();
            with_extension.extend(3_i64.to_le_bytes());
            with_extension.extend([4; 16]);
            with_extension.extend([9; 3]);
            with_extension.extend(0_i64.to_le_bytes());
            with_extension.extend([5; 16]);
            let end = with_extension.len();
            with_extension.push(0xaa);
            let mut r = Reader::new(&with_extension);
            let mut refs = RefTable10_1::new();
            let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 128).unwrap();
            assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
            assert_eq!(p.extensions.len(), 3);
            assert_eq!(p.extensions[1].class_id, [4; 16]);
            assert_eq!(p.extensions[1].unknown_body.as_deref(), Some(&[9; 3][..]));
            assert_eq!(p.extensions[2].class_id, [5; 16]);
            assert!(refs.contains(44));
            assert_eq!(r.position(), end);

            let extension_bytes = end - expected;
            let mut r = Reader::new(&with_extension);
            let mut refs = RefTable10_1::new();
            let result =
                read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, extension_bytes).unwrap();
            assert_eq!(result.stop, StandaloneTableStop10_1::Complete);
            assert_eq!(r.position(), end);

            let mut overflow = with_extension.clone();
            overflow[unknown_offset..unknown_offset + 8].copy_from_slice(&u64::MAX.to_le_bytes());
            for (input, limit) in [
                (&with_extension[..], extension_bytes - 1),
                (&overflow[..], extension_bytes),
            ] {
                let mut r = Reader::new(input);
                let mut refs = RefTable10_1::new();
                refs.register(99);
                let before = refs.clone();
                let error =
                    read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, limit).unwrap_err();
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
                assert_eq!(r.position(), 0);
                assert_eq!(refs, before);
            }

            for len in [end - 25, end - 1] {
                let mut r = Reader::new(&with_extension[..len]);
                let mut refs = RefTable10_1::new();
                assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 128).is_err());
                assert_eq!(r.position(), 0);
                assert!(refs.is_empty());
            }
        }
        b.push(0xaa);
        let mut r = Reader::new(&b);
        let mut refs = RefTable10_1::new();
        let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
        assert_eq!(r.position(), expected, "版本 {version}");
        assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
        assert_eq!(refs.contains(7), version >= 4);
        assert_eq!(refs.contains(41), version >= 11);
        if (11..=12).contains(&version) {
            assert!(matches!(
                p.tail_object_bodies[1],
                Some(StandaloneTableTailBody10_1::Uid(_))
            ));
            assert!(matches!(
                p.tail_object_bodies[2],
                Some(StandaloneTableTailBody10_1::AnnotateProps(_))
            ));
            assert!(refs.contains(42) && refs.contains(43));
        }
        assert_eq!(
            p.pre_join.as_ref().map(Vec::len),
            (version >= 10).then_some(1)
        );
    }
}
