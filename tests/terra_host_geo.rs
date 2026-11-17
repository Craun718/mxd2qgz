use mxd2qgz::objects::geo_transformation::{
    GEO_TRANSFORMATION_CLASS_ID, NADCON_TRANSFORMATION_CLASS_ID, read_geo_transformation_body_10_1,
};
use mxd2qgz::stream::reader::Reader;

#[test]
fn both_classes_share_versions_and_preserve_raw_bytes_and_tail() {
    for class in [GEO_TRANSFORMATION_CLASS_ID, NADCON_TRANSFORMATION_CLASS_ID] {
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
fn errors_leave_the_original_cursor_unchanged() {
    let bytes = [0xaa, 1, 0, 3, 0, 0, 0, 7, 8, 9];
    for class in [GEO_TRANSFORMATION_CLASS_ID, NADCON_TRANSFORMATION_CLASS_ID] {
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
