use mxd2qgz::objects::geo_transformation::{
    ABRIDGED_MOLODENSKY_TRANSFORMATION_CLASS_ID, COMPOSITE_GEO_TRANSFORMATION_CLASS_ID,
    COORDINATE_FRAME_TRANSFORMATION_CLASS_ID, GEO_TRANSFORMATION_CLASS_ID,
    GEOCENTRIC_TRANSLATION_CLASS_ID, GEOGRAPHIC_2D_OFFSET_TRANSFORMATION_CLASS_ID,
    HARN_TRANSFORMATION_CLASS_ID, LONGITUDE_ROTATION_TRANSFORMATION_CLASS_ID,
    MOLODENSKY_BADEKAS_TRANSFORMATION_CLASS_ID, MOLODENSKY_TRANSFORMATION_CLASS_ID,
    NADCON_TRANSFORMATION_CLASS_ID, NTV2_TRANSFORMATION_CLASS_ID, NULL_TRANSFORMATION_CLASS_ID,
    POSITION_VECTOR_TRANSFORMATION_CLASS_ID, UNIT_CHANGE_TRANSFORMATION_CLASS_ID,
    read_geo_transformation_body_10_1,
};
use mxd2qgz::stream::reader::Reader;

#[test]
fn supported_classes_share_versions_and_preserve_raw_bytes_and_tail() {
    for class in [
        GEO_TRANSFORMATION_CLASS_ID,
        NADCON_TRANSFORMATION_CLASS_ID,
        COORDINATE_FRAME_TRANSFORMATION_CLASS_ID,
        ABRIDGED_MOLODENSKY_TRANSFORMATION_CLASS_ID,
        GEOCENTRIC_TRANSLATION_CLASS_ID,
        GEOGRAPHIC_2D_OFFSET_TRANSFORMATION_CLASS_ID,
        HARN_TRANSFORMATION_CLASS_ID,
        LONGITUDE_ROTATION_TRANSFORMATION_CLASS_ID,
        MOLODENSKY_BADEKAS_TRANSFORMATION_CLASS_ID,
        MOLODENSKY_TRANSFORMATION_CLASS_ID,
        NTV2_TRANSFORMATION_CLASS_ID,
        NULL_TRANSFORMATION_CLASS_ID,
        POSITION_VECTOR_TRANSFORMATION_CLASS_ID,
        UNIT_CHANGE_TRANSFORMATION_CLASS_ID,
    ] {
        for version in 0u16..=2 {
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend(3u32.to_le_bytes());
            bytes.extend([0, 0xff, 0x80, 0x42]);
            let mut reader = Reader::new(&bytes);
            let body = read_geo_transformation_body_10_1(&mut reader, class, 3).unwrap();
            assert_eq!(body.version, version);
            assert_eq!(body.text, [0, 0xff, 0x80]);
            assert_eq!(reader.position(), 9);
            assert_eq!(reader.read_bytes(1).unwrap(), [0x42]);
        }
    }
}

#[test]
fn composite_truncation_and_depth_limit_restore_the_cursor() {
    let mut nested = vec![1, 0, 0, 0, 0, 0]; // v1、空名称和零成员。
    for depth in 1..=17 {
        let mut reader = Reader::new(&nested);
        let parsed = read_geo_transformation_body_10_1(
            &mut reader,
            COMPOSITE_GEO_TRANSFORMATION_CLASS_ID,
            4096,
        );
        if depth <= 16 {
            assert!(parsed.unwrap().composite.is_some());
            assert_eq!(reader.remaining(), 0);
        } else {
            assert!(parsed.is_err());
            assert_eq!(reader.position(), 0);
        }
        if depth == 2 {
            for cut in 0..nested.len() {
                let mut reader = Reader::new(&nested[..cut]);
                assert!(
                    read_geo_transformation_body_10_1(
                        &mut reader,
                        COMPOSITE_GEO_TRANSFORMATION_CLASS_ID,
                        4096,
                    )
                    .is_err()
                );
                assert_eq!(reader.position(), 0);
            }
        }
        let mut wrapper = vec![1, 0, 0, 0, 1, 0, 1, 0];
        wrapper.extend(COMPOSITE_GEO_TRANSFORMATION_CLASS_ID);
        wrapper.extend(nested);
        nested = wrapper;
    }
}

#[test]
fn errors_leave_the_original_cursor_unchanged() {
    let bytes = [0xaa, 1, 0, 3, 0, 0, 0, 7, 8, 9];
    for class in [
        GEO_TRANSFORMATION_CLASS_ID,
        NADCON_TRANSFORMATION_CLASS_ID,
        COORDINATE_FRAME_TRANSFORMATION_CLASS_ID,
    ] {
        for end in 1..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            reader.read_bytes(1).unwrap();
            assert!(read_geo_transformation_body_10_1(&mut reader, class, 3).is_err());
            assert_eq!(reader.position(), 1);
        }
        for (body, max) in [
            (&[3, 0, 0, 0, 0, 0][..], 10),
            (&[0xff, 0xff, 0, 0, 0, 0][..], 10),
            (&bytes[1..], 2),
            (&[0, 0, 0, 12, 0, 0][..], usize::MAX),
        ] {
            let mut reader = Reader::new(body);
            assert!(read_geo_transformation_body_10_1(&mut reader, class, max).is_err());
            assert_eq!(reader.position(), 0);
        }
    }
    let mut reader = Reader::new(&bytes[1..]);
    assert!(read_geo_transformation_body_10_1(&mut reader, [0; 16], 3).is_err());
    assert_eq!(reader.position(), 0);
}

#[test]
fn empty_and_maximum_allowed_lengths() {
    for length in [0usize, 3071] {
        let mut bytes = 2u16.to_le_bytes().to_vec();
        bytes.extend((length as u32).to_le_bytes());
        bytes.resize(6 + length, 0x5a);
        let mut reader = Reader::new(&bytes);
        let body =
            read_geo_transformation_body_10_1(&mut reader, GEO_TRANSFORMATION_CLASS_ID, length)
                .unwrap();
        assert_eq!(body.text.len(), length);
        assert_eq!(reader.remaining(), 0);
    }
}
