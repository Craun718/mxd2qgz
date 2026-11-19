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
        if version >= 12 {
            b.extend(2u16.to_le_bytes());
            b.extend(0u16.to_le_bytes());
            b.extend([0; 16]);
            b.extend(0u16.to_le_bytes());
            b.extend(1u16.to_le_bytes());
            for _ in 0..4 {
                string(&mut b);
            }
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
        if version == 14 {
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
            let unknown_start = with_extension.len();
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

            with_extension.extend(3i64.to_le_bytes());
            with_extension.extend([4; 16]);
            with_extension.extend([9; 3]);
            let mut r = Reader::new(&with_extension);
            let mut refs = RefTable10_1::new();
            let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 10, 100).unwrap();
            assert!(matches!(
                p.stop,
                StandaloneTableStop10_1::Extension { size: 3, .. }
            ));
            assert_eq!(p.extensions.len(), 1);
            assert!(refs.contains(44));
            assert_eq!(r.position(), unknown_start);
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
