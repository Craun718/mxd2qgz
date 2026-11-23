use mxd2qgz::objects::geodata_xform::RPC_XFORM_CLASS_ID;
use mxd2qgz::objects::geodata_xform::{
    ADJUST_XFORM_CLASS_ID, APPROXIMATION_XFORM_CLASS_ID, COMPOSITE_XFORM_CLASS_ID,
    COORDINATE_XFORM_CLASS_ID, GCS_SHIFT_XFORM_CLASS_ID, GeodataXformBody10_1,
    IDENTITY_XFORM_CLASS_ID, POLYNOMIAL_XFORM_CLASS_ID, read_geodata_xform_slot_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;

const DOUBLE_ARRAY: [u8; 16] = [
    0xa7, 0x6c, 0xc0, 0x60, 0x9e, 0xe0, 0xd2, 0x11, 0x9f, 0x7b, 0, 0xc0, 0x4f, 0x8e, 0xce, 0x27,
];

fn header(bytes: &mut Vec<u8>, class: [u8; 16], key: u32) {
    bytes.extend(class);
    bytes.extend(key.to_le_bytes());
}

#[test]
fn csm_keeps_counted_narrow_states_and_rejects_malformed_values() {
    use mxd2qgz::objects::geodata_xform::CSM_XFORM_CLASS_ID;

    for (version, second) in [(1_u16, &b""[..]), (2, &b"model:2"[..])] {
        let mut bytes = Vec::new();
        header(&mut bytes, CSM_XFORM_CLASS_ID, 119);
        bytes.extend(version.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        let first_length = bytes.len();
        let first = [0x91, 0xe4, b'A'];
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(first);
        bytes.extend((second.len() as u32).to_le_bytes());
        bytes.extend(second);
        let end = bytes.len();
        bytes.push(0xf3);

        let rejects = |input: &[u8], limit| {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(input);
            assert!(read_geodata_xform_slot_10_1(&mut reader, limit, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        };
        for cut in 0..end {
            rejects(&bytes[..cut], 4096);
        }
        for (offset, value) in [(22, 2_u32), (26, 2), (first_length, u32::MAX)] {
            let mut malformed = bytes.clone();
            malformed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            rejects(&malformed, 4096);
        }
        let mut malformed = bytes.clone();
        malformed[first_length + 4] = 0;
        rejects(&malformed, 4096);
        rejects(&bytes, 2 + second.len());

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Csm(body) = *slot.body.unwrap() else {
            panic!("预期 CSMXform");
        };
        assert_eq!((body.version, body.state_version), (i32::from(version), 1));
        assert_eq!(body.flags_f8_fc, [1, 0]);
        assert_eq!(body.states, [first.to_vec(), second.to_vec()]);
        assert_eq!(reader.position(), end);
        assert!(refs.contains(119));
    }
}

#[test]
fn spot5_reads_legacy_header_and_v2_tail_at_their_own_boundaries() {
    use mxd2qgz::objects::geodata_xform::SPOT5_XFORM_CLASS_ID;

    for version in [1_u16, 2] {
        let mut bytes = Vec::new();
        header(&mut bytes, SPOT5_XFORM_CLASS_ID, 117);
        bytes.extend(version.to_le_bytes());
        if version == 1 {
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend([b'X', 0, 0, 0]);
            bytes.extend(21_u16.to_le_bytes());
        }
        bytes.extend([0; 3 * 4 + 3 * 8 + 8 * 4]);
        bytes.extend(22_u16.to_le_bytes());
        bytes.extend((-23_i32).to_le_bytes());
        for value in [24.5_f64, 25.5, 26.5] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend([0; 20 * 16]);
        header(&mut bytes, DOUBLE_ARRAY, 118);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(27.5_f64.to_le_bytes());
        if version == 2 {
            for value in [28.5_f64, 29.5, 30.5] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(31_u16.to_le_bytes());
            bytes.extend(32_u16.to_le_bytes());
            bytes.extend([0; 16]);
        }
        let end = bytes.len();
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        bytes.push(0xf3);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Spot5(body) = *slot.body.unwrap() else {
            panic!("预期 SPOT5Xform");
        };
        assert_eq!(body.legacy, (version == 1).then(|| ("X".into(), 21)));
        assert_eq!((body.short_144, body.long_148), (22, -23));
        assert_eq!(body.doubles_14c_15c_154, [24.5, 25.5, 26.5]);
        assert_eq!(body.double_arrays.len(), 21);
        assert_eq!(body.double_arrays[20].body, Some((1, vec![27.5])));
        if version == 2 {
            let tail = body.tail.unwrap();
            assert_eq!(tail.doubles_1c_24_2c, [28.5, 29.5, 30.5]);
            assert_eq!(tail.shorts_36_34, [31, 32]);
            assert!(tail.raster.is_none());
        } else {
            assert!(body.tail.is_none());
        }
        assert_eq!(refs.len(), 2);
        assert_eq!(reader.position(), end);
    }
}

#[test]
fn frame_reads_versioned_tail_and_shares_point_transform_and_array_refs() {
    use mxd2qgz::objects::geodata_xform::FRAME_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::POINT_CLASS_ID;

    for version in 1_u16..=7 {
        let mut bytes = Vec::new();
        header(&mut bytes, FRAME_XFORM_CLASS_ID, 110);
        bytes.extend(version.to_le_bytes());
        bytes.extend([0; 16]); // 空空间参考。
        for value in [1.5_f64, 2.5, 3.5] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(4_u16.to_le_bytes());
        bytes.extend(5_u16.to_le_bytes());
        bytes.extend([0; 16]); // LoadRaster 的空名称。
        header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 111);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(6_u16.to_le_bytes());
        for value in 7..=11 {
            bytes.extend((f64::from(value) + 0.5).to_le_bytes());
        }
        header(&mut bytes, POINT_CLASS_ID, 112);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes()); // 空点记录。
        header(&mut bytes, POINT_CLASS_ID, 112);
        bytes.extend(12_u16.to_le_bytes());
        bytes.extend(13.5_f64.to_le_bytes());
        bytes.extend(14_u16.to_le_bytes());
        for value in 15..=24 {
            bytes.extend((f64::from(value) + 0.5).to_le_bytes());
        }
        bytes.extend((-25_i32).to_le_bytes());
        if version >= 5 {
            bytes.extend(26.5_f64.to_le_bytes());
            header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 111);
        }
        if version >= 6 {
            bytes.extend((-27_i32).to_le_bytes());
        }
        if version >= 7 {
            header(&mut bytes, DOUBLE_ARRAY, 113);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(28.5_f64.to_le_bytes());
            bytes.extend(29.5_f64.to_le_bytes());
            header(&mut bytes, DOUBLE_ARRAY, 113);
        }
        let end = bytes.len();
        for cut in 0..end {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        bytes.push(0xf3);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Frame(body) = *slot.body.unwrap() else {
            panic!("预期 FrameXform");
        };
        assert_eq!(body.version, i32::from(version));
        assert_eq!(body.doubles_1c_24_2c, [1.5, 2.5, 3.5]);
        assert_eq!(body.shorts_36_34, [4, 5]);
        assert!(body.raster.is_none());
        assert!(body.inner.body.is_some());
        assert_eq!(body.short_a8, 6);
        assert_eq!(body.doubles_ac, [7.5, 8.5, 9.5, 10.5, 11.5]);
        assert!(body.points[0].body.is_some());
        assert!(matches!(
            body.points[1].entry,
            ObjectEntry10_1::BackReference { key: 112, .. }
        ));
        assert_eq!(
            (body.short_e0, body.double_e4, body.short_f4),
            (12, 13.5, 14)
        );
        assert_eq!(
            body.doubles_fc,
            [15.5, 16.5, 17.5, 18.5, 19.5, 20.5, 21.5, 22.5, 23.5]
        );
        assert_eq!((body.double_144, body.long_14c), (24.5, -25));
        assert_eq!(body.double_ec, (version >= 5).then_some(26.5));
        assert_eq!(
            body.secondary_transform.map(|slot| slot.entry),
            (version >= 5).then_some(ObjectEntry10_1::BackReference {
                class_id: IDENTITY_XFORM_CLASS_ID,
                key: 111
            })
        );
        assert_eq!(body.long_d4, (version >= 6).then_some(-27));
        if version == 7 {
            let [first, second] = body.double_arrays.unwrap();
            assert_eq!(first.body, Some((1, vec![28.5, 29.5])));
            assert!(matches!(
                second.entry,
                ObjectEntry10_1::BackReference { key: 113, .. }
            ));
            assert!(second.body.is_none());
        } else {
            assert!(body.double_arrays.is_none());
        }
        assert_eq!(reader.position(), end);
    }
}

#[test]
fn formosat2_reads_fourteen_arrays_then_polynomial_and_raster() {
    use mxd2qgz::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
    use mxd2qgz::objects::geodata_xform::FORMOSAT2_XFORM_CLASS_ID;

    let mut bytes = Vec::new();
    header(&mut bytes, FORMOSAT2_XFORM_CLASS_ID, 114);
    bytes.extend(1_u16.to_le_bytes());
    for value in 0..10_i32 {
        bytes.extend(value.to_le_bytes());
    }
    for value in 10..16 {
        bytes.extend((f64::from(value) + 0.25).to_le_bytes());
    }
    header(&mut bytes, DOUBLE_ARRAY, 115);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(19.25_f64.to_le_bytes());
    bytes.extend([0; 12 * 16]);
    let array_reference = bytes.len();
    header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 115);
    let polynomial_start = bytes.len();
    header(&mut bytes, POLYNOMIAL_XFORM_CLASS_ID, 116);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16 + 12 + 16 + 12]); // 空空间参考、计数与选择子。
    let polynomial_end = bytes.len();
    for value in [20.25_f64, 21.25, 22.25] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(23_u16.to_le_bytes());
    bytes.extend(24_u16.to_le_bytes());
    header(&mut bytes, RASTER_DATASET_NAME_CLASS_ID, 70);
    bytes.extend(0_i32.to_le_bytes());
    header(&mut bytes, POLYNOMIAL_XFORM_CLASS_ID, 116);
    bytes.extend([0; 16]); // 空范围。
    let end = bytes.len();

    for cut in 0..end {
        let mut refs = RefTable10_1::new();
        refs.register(70);
        let mut reader = Reader::new(&bytes[..cut]);
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);
    }
    bytes.push(0xf3);
    let mut refs = RefTable10_1::new();
    refs.register(70);
    let mut reader = Reader::new(&bytes);
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Formosat2(body) = *slot.body.unwrap() else {
        panic!("预期 Formosat2Xform");
    };
    assert_eq!(body.longs_a8_cc, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(
        body.doubles_d4_fc_f4,
        [10.25, 11.25, 12.25, 13.25, 14.25, 15.25]
    );
    assert_eq!(body.double_arrays.len(), 14);
    assert_eq!(body.double_arrays[0].body, Some((1, vec![19.25])));
    assert!(matches!(
        body.double_arrays[13].entry,
        ObjectEntry10_1::BackReference {
            class_id: DOUBLE_ARRAY,
            key: 115
        }
    ));
    assert!(matches!(
        body.polynomial.body.as_deref(),
        Some(GeodataXformBody10_1::Polynomial(_))
    ));
    assert_eq!(body.doubles_1c_24_2c, [20.25, 21.25, 22.25]);
    assert_eq!(body.shorts_36_34, [23, 24]);
    assert!(matches!(
        body.raster.unwrap().transform.entry,
        ObjectEntry10_1::BackReference { key: 116, .. }
    ));
    assert_eq!(refs.len(), 4);
    assert_eq!(reader.position(), end);

    let mut wrong = bytes.clone();
    wrong[array_reference + 16..array_reference + 20].copy_from_slice(&70_u32.to_le_bytes());
    let mut wrong_refs = RefTable10_1::new();
    let mut existing = Vec::new();
    header(&mut existing, RASTER_DATASET_NAME_CLASS_ID, 70);
    mxd2qgz::storage::object_refs::read_object_entry_10_1(
        &mut Reader::new(&existing),
        &mut wrong_refs,
    )
    .unwrap();
    let before = wrong_refs.clone();
    let mut wrong_reader = Reader::new(&wrong);
    assert_eq!(
        read_geodata_xform_slot_10_1(&mut wrong_reader, 4096, &mut wrong_refs)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
    assert_eq!(wrong_reader.position(), 0);
    assert_eq!(wrong_refs, before);

    let mut reference = Vec::new();
    header(&mut reference, IDENTITY_XFORM_CLASS_ID, 116);
    bytes.splice(polynomial_start..polynomial_end, reference);
    for registered in [POLYNOMIAL_XFORM_CLASS_ID, IDENTITY_XFORM_CLASS_ID] {
        let mut existing = Vec::new();
        header(&mut existing, registered, 116);
        let mut refs = RefTable10_1::new();
        refs.register(70);
        mxd2qgz::storage::object_refs::read_object_entry_10_1(
            &mut Reader::new(&existing),
            &mut refs,
        )
        .unwrap();
        let before = refs.clone();
        let mut reader = Reader::new(&bytes);
        let result = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs);
        if registered == POLYNOMIAL_XFORM_CLASS_ID {
            let GeodataXformBody10_1::Formosat2(body) = *result.unwrap().body.unwrap() else {
                panic!("预期 Formosat2Xform");
            };
            assert_eq!(
                body.polynomial.entry,
                ObjectEntry10_1::BackReference {
                    class_id: POLYNOMIAL_XFORM_CLASS_ID,
                    key: 116,
                }
            );
            assert!(body.polynomial.body.is_none());
            assert_eq!(body.shorts_36_34, [23, 24]);
            assert_eq!(reader.read_bytes(1).unwrap(), &[0xf3]);
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn pictometry_consumes_the_string_written_by_save() {
    use mxd2qgz::objects::geodata_xform::PICTOMETRY_XFORM_CLASS_ID;

    for (version, text) in [(1_u16, ""), (2, "影像")] {
        let mut bytes = Vec::new();
        header(&mut bytes, PICTOMETRY_XFORM_CLASS_ID, 95);
        bytes.extend(version.to_le_bytes());
        let encoded: Vec<_> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        bytes.extend((encoded.len() as u32).to_le_bytes());
        bytes.extend(encoded);
        let end = bytes.len();
        bytes.push(0xf3);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes[..end - 1]);
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
        let mut reader = Reader::new(&bytes);
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(
            *slot.body.unwrap(),
            GeodataXformBody10_1::Pictometry {
                version: i32::from(version),
                text: text.into()
            }
        );
        assert_eq!(reader.position(), end);
        assert!(refs.contains(95));
    }
}

#[test]
fn lsr_reads_short_version_and_point_before_doubles() {
    use mxd2qgz::objects::geodata_xform::LSR_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::POINT_CLASS_ID;

    let mut bytes = Vec::new();
    header(&mut bytes, LSR_XFORM_CLASS_ID, 93);
    bytes.extend(1_u16.to_le_bytes());
    header(&mut bytes, POINT_CLASS_ID, 94);
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes()); // 空坐标记录。
    bytes.extend([0; 16]);
    for i in 0..11 {
        bytes.extend((i as f64 + 0.25).to_le_bytes());
    }
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    let end = bytes.len();
    bytes.push(0xf3);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Lsr(body) = *slot.body.unwrap() else {
        panic!("预期 LSRXform");
    };
    assert_eq!(body.version, 1);
    assert!(body.point.is_some());
    assert_eq!(
        body.doubles_ac,
        [0.25, 1.25, 2.25, 3.25, 4.25, 5.25, 6.25, 7.25, 8.25]
    );
    assert_eq!(body.doubles_f4_fc, [9.25, 10.25]);
    assert!(refs.contains(93) && refs.contains(94));
    assert_eq!(reader.position(), end);
}

#[test]
fn lsr_point_reference_uses_registered_class() {
    use mxd2qgz::objects::geodata_xform::LSR_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::{MULTIPOINT_CLASS_ID, POINT_CLASS_ID};
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    for (registered, claimed) in [
        (POINT_CLASS_ID, MULTIPOINT_CLASS_ID),
        (MULTIPOINT_CLASS_ID, POINT_CLASS_ID),
    ] {
        let mut previous = Vec::new();
        header(&mut previous, registered, 94);
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
        let before = refs.clone();

        let mut bytes = Vec::new();
        header(&mut bytes, LSR_XFORM_CLASS_ID, 93);
        bytes.extend(1_u16.to_le_bytes());
        header(&mut bytes, claimed, 94);
        bytes.extend([0; 88]);
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let result = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs);
        if registered == POINT_CLASS_ID {
            let GeodataXformBody10_1::Lsr(body) = *result.unwrap().body.unwrap() else {
                panic!("缺少 LSRXform");
            };
            assert_eq!(
                body.point_entry,
                ObjectEntry10_1::BackReference {
                    class_id: POINT_CLASS_ID,
                    key: 94,
                }
            );
            assert_eq!(reader.position(), end);
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn geometric_reads_collection_and_inline_or_object_transform() {
    use mxd2qgz::objects::geodata_xform::{GEOMETRIC_XFORM_CLASS_ID, GeometricTransformation10_1};
    use mxd2qgz::objects::geometry::{GeometryBody10_1, MULTIPOINT_CLASS_ID};

    for (version, selector, kind) in [(1_i32, 0, 0), (1, 0, 1), (2, 0, 2), (2, 1, 0)] {
        let mut bytes = Vec::new();
        header(&mut bytes, GEOMETRIC_XFORM_CLASS_ID, 91);
        bytes.extend(version.to_le_bytes());
        bytes.extend([0; 16]); // 空空间参考。
        header(&mut bytes, MULTIPOINT_CLASS_ID, 92);
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes()); // 空点集。
        bytes.extend([0; 16]);
        bytes.push(selector);
        let expected = if selector != 0 {
            bytes.extend([0; 16]);
            GeometricTransformation10_1::Object {
                selector,
                entry: ObjectEntry10_1::Null,
                body: None,
            }
        } else {
            bytes.push(kind);
            match kind {
                0 => GeometricTransformation10_1::None,
                1 => {
                    let values = [1.25_f64, 2.25, 3.25, 4.25, 5.25, 6.25];
                    bytes.extend(values.into_iter().flat_map(f64::to_le_bytes));
                    GeometricTransformation10_1::Affine(values)
                }
                2 => {
                    let values = [1.5_f64, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5];
                    bytes.extend(values.into_iter().flat_map(f64::to_le_bytes));
                    GeometricTransformation10_1::Projective(values)
                }
                _ => unreachable!(),
            }
        };
        if version == 2 {
            bytes.extend(6.75_f64.to_le_bytes());
        }
        for cut in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let end = bytes.len();
        bytes.push(0xf3);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Geometric(body) = *slot.body.unwrap() else {
            panic!("预期 GeometricXform");
        };
        assert!(matches!(
            body.geometry.body.as_deref(),
            Some(GeometryBody10_1::Multipoint(_))
        ));
        assert_eq!(body.transformation, expected);
        assert_eq!(body.double_ac, (version == 2).then_some(6.75));
        assert!(refs.contains(91) && refs.contains(92));
        assert_eq!(reader.position(), end);
    }
}

#[test]
fn geometric_transformation_reference_uses_registered_class() {
    use mxd2qgz::objects::geodata_xform::{GEOMETRIC_XFORM_CLASS_ID, GeometricTransformation10_1};
    use mxd2qgz::objects::geometry::POINT_CLASS_ID;
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    for (registered, claimed) in [
        (IDENTITY_XFORM_CLASS_ID, POINT_CLASS_ID),
        (POINT_CLASS_ID, IDENTITY_XFORM_CLASS_ID),
    ] {
        let mut previous = Vec::new();
        header(&mut previous, registered, 71);
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
        let before = refs.clone();

        let mut bytes = Vec::new();
        header(&mut bytes, GEOMETRIC_XFORM_CLASS_ID, 80);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 32]);
        bytes.push(1);
        header(&mut bytes, claimed, 71);
        bytes.extend(6.75_f64.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let result = read_geodata_xform_slot_10_1(&mut reader, 128, &mut refs);
        if registered == IDENTITY_XFORM_CLASS_ID {
            let GeodataXformBody10_1::Geometric(body) = *result.unwrap().body.unwrap() else {
                panic!("缺少 GeometricXform");
            };
            assert!(matches!(
                body.transformation,
                GeometricTransformation10_1::Object {
                    entry: ObjectEntry10_1::BackReference {
                        class_id: IDENTITY_XFORM_CLASS_ID,
                        key: 71,
                    },
                    body: None,
                    ..
                }
            ));
            assert_eq!(body.double_ac, Some(6.75));
            assert_eq!(reader.position(), end);
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn geometric_reads_persisted_transformation_before_its_tail() {
    use mxd2qgz::objects::geo_transformation::HARN_TRANSFORMATION_CLASS_ID;
    use mxd2qgz::objects::geodata_xform::{GEOMETRIC_XFORM_CLASS_ID, GeometricTransformation10_1};

    let mut bytes = Vec::new();
    header(&mut bytes, GEOMETRIC_XFORM_CLASS_ID, 91);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 32]); // 空空间参考及几何集合。
    bytes.push(1);
    header(&mut bytes, HARN_TRANSFORMATION_CLASS_ID, 92);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(3_u32.to_le_bytes());
    bytes.extend_from_slice(b"raw");
    bytes.extend(1.25_f64.to_le_bytes());

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let decoded = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Geometric(body) = *decoded.body.unwrap() else {
        panic!("预期 GeometricXform");
    };
    let GeometricTransformation10_1::Object {
        body: Some(transform),
        ..
    } = body.transformation
    else {
        panic!("预期非空 ITransformation");
    };
    assert_eq!(transform.text, b"raw");
    assert_eq!(body.double_ac, Some(1.25));
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(91) && refs.contains(92));
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn spline_reads_multipatch_as_a_point_collection() {
    use mxd2qgz::objects::geodata_xform::SPLINE_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::{GeometryBody10_1, MULTIPATCH_CLASS_ID};

    let mut bytes = Vec::new();
    header(&mut bytes, SPLINE_XFORM_CLASS_ID, 101);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    header(&mut bytes, MULTIPATCH_CLASS_ID, 102);
    let mut record = 0x1f_u32.to_le_bytes().to_vec();
    record.extend([0; 32]);
    for value in [1_u32, 3, 0, 0x100] {
        record.extend(value.to_le_bytes());
    }
    record.extend([0; 11 * 8]);
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend((record.len() as u32).to_le_bytes());
    bytes.extend(record);
    bytes.extend([0; 16]);
    header(&mut bytes, MULTIPATCH_CLASS_ID, 102);
    let end = bytes.len();
    bytes.push(0xaa);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Spline { points, .. } = *result.body.unwrap() else {
        panic!("缺少 SplineXform");
    };
    assert!(matches!(
        points[0].body.as_deref(),
        Some(GeometryBody10_1::MultiPatch(_))
    ));
    assert!(matches!(
        points[1].entry,
        ObjectEntry10_1::BackReference {
            class_id: MULTIPATCH_CLASS_ID,
            key: 102,
        }
    ));
    assert!(refs.contains(101) && refs.contains(102));
    assert_eq!(reader.position(), end);

    let mut reader = Reader::new(&bytes[..end - 1]);
    let mut refs = RefTable10_1::new();
    assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn spline_reads_both_point_collections_in_the_host_table() {
    use mxd2qgz::objects::geodata_xform::SPLINE_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::{GeometryBody10_1, MULTIPOINT_CLASS_ID};

    let mut bytes = Vec::new();
    header(&mut bytes, SPLINE_XFORM_CLASS_ID, 101);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    header(&mut bytes, MULTIPOINT_CLASS_ID, 102);
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend([0; 16]);
    header(&mut bytes, MULTIPOINT_CLASS_ID, 102);
    let end = bytes.len();
    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    bytes.push(0xf3);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Spline {
        version: 1, points, ..
    } = *slot.body.unwrap()
    else {
        panic!("预期 SplineXform v1");
    };
    assert!(matches!(
        points[0].body.as_deref(),
        Some(GeometryBody10_1::Multipoint(_))
    ));
    assert_eq!(
        points[1].entry,
        ObjectEntry10_1::BackReference {
            class_id: MULTIPOINT_CLASS_ID,
            key: 102,
        }
    );
    assert!(points[1].body.is_none());
    assert!(refs.contains(101) && refs.contains(102));
    assert_eq!(reader.position(), end);
}

#[test]
fn rpc_raster_keeps_reference_keys_and_recursion_budget() {
    use mxd2qgz::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;

    fn rpc_header(bytes: &mut Vec<u8>, key: u32, version: i32) {
        header(bytes, RPC_XFORM_CLASS_ID, key);
        bytes.extend(version.to_le_bytes());
        bytes.extend([0x5a; 0x2d0]);
        bytes.extend([0; 16]);
        for value in [1.5_f64, 2.5, 3.5] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(79_u16.to_le_bytes());
        bytes.extend(80_u16.to_le_bytes());
    }

    fn raster_name(bytes: &mut Vec<u8>) {
        // 名称键预先登记，以便单独检查下一级变换的计数。
        header(bytes, RASTER_DATASET_NAME_CLASS_ID, 70);
        bytes.extend(0_i32.to_le_bytes());
    }

    for version in [1, 2] {
        let mut bytes = Vec::new();
        rpc_header(&mut bytes, 71, version);
        if version == 2 {
            raster_name(&mut bytes);
            header(&mut bytes, RPC_XFORM_CLASS_ID, 71);
            bytes.extend([0; 16]);
        }
        for cut in 0..bytes.len() {
            let mut refs = RefTable10_1::new();
            refs.register(70);
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
        }
        bytes.push(0x95);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(70);
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Rpc(rpc) = *slot.body.unwrap() else {
            panic!("预期 RPCXform");
        };
        assert_eq!(rpc.version, version);
        assert_eq!(*rpc.raw_parameters, [0x5a; 0x2d0]);
        assert_eq!(rpc.doubles_1c_24_2c, [1.5, 2.5, 3.5]);
        assert_eq!(rpc.shorts_36_34, [79, 80]);
        assert_eq!(
            rpc.raster.map(|raster| raster.transform.entry),
            (version == 2).then_some(ObjectEntry10_1::BackReference {
                class_id: RPC_XFORM_CLASS_ID,
                key: 71,
            })
        );
        assert_eq!(reader.read_bytes(1).unwrap(), [0x95]);
    }

    for count in [16, 17] {
        let mut bytes = Vec::new();
        for i in 0..count {
            rpc_header(&mut bytes, 100 + i, 2);
            raster_name(&mut bytes);
        }
        bytes.extend([0; 16]); // 最内层变换为空。
        bytes.extend(vec![0; count as usize * 16]); // 各层范围为空。
        let mut refs = RefTable10_1::new();
        refs.register(70);
        let mut reader = Reader::new(&bytes);
        let result = read_geodata_xform_slot_10_1(&mut reader, bytes.len(), &mut refs);
        if count == 16 {
            result.unwrap();
            assert_eq!(reader.remaining(), 0);
            assert_eq!(refs.len(), 17);
        } else {
            assert!(result.unwrap_err().to_string().contains("嵌套深度"));
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
        }
    }
}

#[test]
fn coordinate_transform_preserves_operation_set_boundary() {
    let mut bytes = Vec::new();
    header(&mut bytes, COORDINATE_XFORM_CLASS_ID, 40);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 32]); // 两个空间参考。
    let operations = bytes.len();
    bytes.extend([0; 16]);
    bytes.extend(87.5_f64.to_le_bytes());
    bytes.extend((-88_i32).to_le_bytes());
    bytes.extend(89_u16.to_le_bytes());
    bytes.extend(90_u16.to_le_bytes());
    let end = bytes.len();
    bytes.push(0x95);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let decoded = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Coordinate {
        operation_set,
        double_b4,
        long_124,
        shorts_ac_ae,
        ..
    } = *decoded.body.unwrap()
    else {
        panic!("预期 CoordinateXform");
    };
    assert_eq!(operation_set, ObjectEntry10_1::Null);
    assert_eq!((double_b4, long_124, shorts_ac_ae), (87.5, -88, [89, 90]));
    assert_eq!(reader.position(), end);
    assert!(refs.contains(40));

    bytes[operations..operations + 16].fill(0x39);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let error = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn adjust_wraps_approximation_and_gcs_shift_in_native_order() {
    for shift_version in [1_i32, 2] {
        let mut bytes = Vec::new();
        header(&mut bytes, ADJUST_XFORM_CLASS_ID, 41);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(71_u16.to_le_bytes());
        bytes.extend((-72_i32).to_le_bytes());
        bytes.extend([0; 16]);
        header(&mut bytes, APPROXIMATION_XFORM_CLASS_ID, 42);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        header(&mut bytes, GCS_SHIFT_XFORM_CLASS_ID, 43);
        bytes.extend(shift_version.to_le_bytes());
        bytes.extend(73.5_f64.to_le_bytes());
        bytes.extend([0; 16]);
        if shift_version == 2 {
            bytes.extend(74_u16.to_le_bytes());
        }
        bytes.extend(75.5_f64.to_le_bytes());
        bytes.extend((-76_i32).to_le_bytes());
        bytes.extend(77_u16.to_le_bytes());
        bytes.extend(78_u16.to_le_bytes());
        for value in [79.5_f64, 80.5, 81.5] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(82_u16.to_le_bytes());
        bytes.extend(83_u16.to_le_bytes());

        for cut in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }

        bytes.push(0x95);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Adjust {
            short_c8,
            long_a8,
            inner,
            ..
        } = *slot.body.unwrap()
        else {
            panic!("预期 AdjustXform");
        };
        assert_eq!((short_c8, long_a8), (71, -72));
        let GeodataXformBody10_1::Approximation {
            inner,
            double_ac,
            long_b4,
            shorts_ba_b8,
            doubles_1c_24_2c,
            shorts_36_34,
            ..
        } = *inner.body.unwrap()
        else {
            panic!("预期 ApproximationXform");
        };
        assert_eq!((double_ac, long_b4), (75.5, -76));
        assert_eq!(shorts_ba_b8, [77, 78]);
        assert_eq!(doubles_1c_24_2c, [79.5, 80.5, 81.5]);
        assert_eq!(shorts_36_34, [82, 83]);
        let GeodataXformBody10_1::GcsShift {
            version,
            double_b4,
            short_bc,
            ..
        } = *inner.body.unwrap()
        else {
            panic!("预期 GCSShiftXform");
        };
        assert_eq!((version, double_b4), (shift_version, 73.5));
        assert_eq!(short_bc, (shift_version == 2).then_some(74));
        assert!([41, 42, 43].into_iter().all(|key| refs.contains(key)));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x95]);
    }
}

#[test]
fn composite_uses_one_table_for_nested_bodies_and_references() {
    let mut bytes = Vec::new();
    header(&mut bytes, COMPOSITE_XFORM_CLASS_ID, 51);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 52);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]); // Identity 的空空间参考。
    header(&mut bytes, IDENTITY_XFORM_CLASS_ID, 52);
    bytes.extend(31_u16.to_le_bytes());
    for cut in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    bytes.push(0x95);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Composite {
        children, short_34, ..
    } = *slot.body.unwrap()
    else {
        panic!("应读为复合变换");
    };
    assert!(matches!(
        children[0].body.as_deref(),
        Some(GeodataXformBody10_1::Identity { .. })
    ));
    assert!(matches!(
        children[1].entry,
        ObjectEntry10_1::BackReference { key: 52, .. }
    ));
    assert_eq!(short_34, Some(31));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x95]);
    assert_eq!(refs.len(), 2);
}

#[test]
fn polynomial_reads_point_and_coefficient_arrays_in_native_order() {
    let mut bytes = Vec::new();
    header(&mut bytes, POLYNOMIAL_XFORM_CLASS_ID, 1);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    for flag in [1_i32, -1, 1] {
        bytes.extend(flag.to_le_bytes());
    }
    for value in 1..=18 {
        bytes.extend((value as f64).to_le_bytes());
    }
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend([b'P', 0, 0, 0]);
    bytes.push(0x87);
    let mut reader = Reader::new(&bytes);
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut RefTable10_1::new()).unwrap();
    let GeodataXformBody10_1::Polynomial(body) = *slot.body.unwrap() else {
        panic!("应读为多项式变换");
    };
    assert_eq!(
        body.point_arrays.unwrap(),
        [vec![[1.0, 2.0]], vec![[3.0, 4.0]]]
    );
    assert_eq!(
        body.coefficients_e4.unwrap(),
        [vec![5.0, 6.0], vec![7.0, 8.0]]
    );
    assert_eq!(
        body.coefficients_e8.unwrap(),
        [vec![9.0, 10.0], vec![11.0, 12.0]]
    );
    assert_eq!(
        body.arrays_c8_cc.unwrap(),
        [vec![13.0, 14.0, 15.0], vec![16.0, 17.0, 18.0]]
    );
    assert_eq!(body.name.as_deref(), Some("P"));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x87]);

    // 同一位置的有符号计数若异常，不能借用后续字节或保留对象键。
    for n in [-1, i32::MAX] {
        let mut bad = bytes.clone();
        bad[44..48].copy_from_slice(&n.to_le_bytes());
        let mut reader = Reader::new(&bad);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn composite_rejects_excessive_nesting_before_committing_keys() {
    let mut bytes = Vec::new();
    for key in 1..=25 {
        header(&mut bytes, COMPOSITE_XFORM_CLASS_ID, key);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
    }
    bytes.extend([0; 16]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap_err();
    assert!(err.to_string().contains("嵌套深度"));
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn projective_geometry_compatibility_layout_preserves_outer_fields() {
    use mxd2qgz::objects::geodata_xform::{
        GEOMETRIC_XFORM_CLASS_ID, GeometricTransformation10_1, PROJECTIVE_XFORM_CLASS_ID,
    };
    use mxd2qgz::objects::geometry::{GeometryBody10_1, MULTIPOINT_CLASS_ID};

    let mut bytes = Vec::new();
    header(&mut bytes, GEOMETRIC_XFORM_CLASS_ID, 120);
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 32]); // 空空间参考和集合。
    bytes.push(1);
    header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 121);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    header(&mut bytes, MULTIPOINT_CLASS_ID, 122);
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend([0; 16]);
    let count_offset = bytes.len();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    let points = [[[1., 2.], [3., 4.]], [[-1., -2.], [-3., -4.]]];
    for value in points.into_iter().flatten().flatten() {
        bytes.extend(f64::to_le_bytes(value));
    }
    bytes.extend(1.25_f64.to_le_bytes());
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    let mut oversized = bytes.clone();
    oversized[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
    let mut reader = Reader::new(&oversized);
    let mut refs = RefTable10_1::new();
    assert!(read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());

    bytes.push(0x5a);
    let mut reader = Reader::new(&bytes);
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Geometric(geometric) = *slot.body.unwrap() else {
        panic!("应读取 GeometricXform");
    };
    let GeometricTransformation10_1::RasterObject { body, .. } = geometric.transformation else {
        panic!("应读取 GeometricXform 的栅格变换对象");
    };
    let GeodataXformBody10_1::Projective(projective) = *body else {
        panic!("应读取 ProjectiveXform");
    };
    assert_eq!(geometric.double_ac, Some(1.25));
    assert_eq!(projective.point_count, 2);
    assert_eq!(projective.point_arrays.unwrap(), points.map(Vec::from));
    assert!(matches!(
        projective.geometry.body.as_deref(),
        Some(GeometryBody10_1::Multipoint(_))
    ));
    assert!(refs.contains(120) && refs.contains(121) && refs.contains(122));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x5a]);
}

#[test]
fn projective_xform_reads_geometry_collections_before_control_points() {
    use mxd2qgz::objects::geodata_xform::PROJECTIVE_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::{
        GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID,
    };

    for class in [GEOMETRY_BAG_CLASS_ID, MULTIPATCH_CLASS_ID] {
        let mut bytes = Vec::new();
        header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 80);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        header(&mut bytes, class, 81);
        if class == GEOMETRY_BAG_CLASS_ID {
            bytes.extend([0; 2 + 4 + 16 + 16 + 1]);
        } else {
            let mut record = 0x1f_u32.to_le_bytes().to_vec();
            record.extend([0; 32]);
            for value in [1_u32, 3, 0, 0x100] {
                record.extend(value.to_le_bytes());
            }
            record.extend([0; 11 * 8]);
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend((record.len() as u32).to_le_bytes());
            bytes.extend(record);
            bytes.extend([0; 16]);
        }
        bytes.extend([0; 8]);
        let end = bytes.len();
        bytes.push(0xaa);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let slot = read_geodata_xform_slot_10_1(&mut reader, 256, &mut refs).unwrap();
        let GeodataXformBody10_1::Projective(body) = *slot.body.unwrap() else {
            panic!("缺少 ProjectiveXform");
        };
        assert!(matches!(
            (class, body.geometry.body.as_deref()),
            (
                GEOMETRY_BAG_CLASS_ID,
                Some(GeometryBody10_1::GeometryBag(_))
            ) | (MULTIPATCH_CLASS_ID, Some(GeometryBody10_1::MultiPatch(_)))
        ));
        assert_eq!(body.point_count, 0);
        assert_eq!(reader.position(), end);
        assert!(refs.contains(80) && refs.contains(81));

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_geodata_xform_slot_10_1(&mut reader, 256, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn projective_geometry_reference_checks_the_registered_class() {
    use mxd2qgz::objects::geodata_xform::PROJECTIVE_XFORM_CLASS_ID;
    use mxd2qgz::objects::geometry::{MULTIPOINT_CLASS_ID, POINT_CLASS_ID};
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    for class_id in [MULTIPOINT_CLASS_ID, POINT_CLASS_ID] {
        let mut previous = Vec::new();
        header(&mut previous, class_id, 71);
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
        let before = refs.clone();
        let mut bytes = Vec::new();
        header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 80);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        header(&mut bytes, MULTIPOINT_CLASS_ID, 71);
        bytes.extend([0; 8]);
        let mut reader = Reader::new(&bytes);
        let result = read_geodata_xform_slot_10_1(&mut reader, 128, &mut refs);
        if class_id == MULTIPOINT_CLASS_ID {
            let GeodataXformBody10_1::Projective(body) = *result.unwrap().body.unwrap() else {
                panic!("缺少 ProjectiveXform");
            };
            assert!(matches!(
                body.geometry.entry,
                ObjectEntry10_1::BackReference {
                    class_id: MULTIPOINT_CLASS_ID,
                    key: 71,
                }
            ));
            assert_eq!(reader.position(), bytes.len());
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn projective_xform_reads_explicit_projection_references_in_both_positions() {
    use mxd2qgz::objects::geodata_xform::{
        PROJECTIVE_TRANSFORMATION_2D_CLASS_ID, PROJECTIVE_XFORM_CLASS_ID,
    };
    use mxd2qgz::objects::geometry::MULTIPOINT_CLASS_ID;

    for (geometry_present, empty_transform) in [(false, true), (false, false), (true, false)] {
        let mut bytes = Vec::new();
        header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 80);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]); // 空空间参考。
        if geometry_present {
            header(&mut bytes, MULTIPOINT_CLASS_ID, 81);
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend([0; 16]);
        }
        if empty_transform {
            bytes.extend([0; 16]);
        } else {
            header(&mut bytes, PROJECTIVE_TRANSFORMATION_2D_CLASS_ID, 75);
        }
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        for coordinate in [1.25_f64, 2.0, 3.0, -4.0] {
            bytes.extend(coordinate.to_le_bytes());
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut refs = RefTable10_1::new();
        refs.register(75);
        let mut reader = Reader::new(&bytes);
        let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let GeodataXformBody10_1::Projective(body) = *slot.body.unwrap() else {
            panic!("应读取 ProjectiveXform");
        };
        assert_eq!(body.geometry.body.is_some(), geometry_present);
        assert_eq!(
            body.transformation,
            (!empty_transform).then_some(ObjectEntry10_1::BackReference {
                class_id: PROJECTIVE_TRANSFORMATION_2D_CLASS_ID,
                key: 75,
            })
        );
        assert_eq!(
            body.point_arrays.unwrap(),
            [vec![[1.25, 2.0]], vec![[3.0, -4.0]]]
        );
        assert_eq!(reader.position(), end);
        assert_eq!(refs.contains(81), geometry_present);

        if !empty_transform {
            let mut wrong_header = Vec::new();
            header(&mut wrong_header, MULTIPOINT_CLASS_ID, 75);
            let mut wrong_refs = RefTable10_1::new();
            mxd2qgz::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&wrong_header),
                &mut wrong_refs,
            )
            .unwrap();
            let before = wrong_refs.clone();
            let mut wrong_reader = Reader::new(&bytes);
            assert!(
                read_geodata_xform_slot_10_1(&mut wrong_reader, 4096, &mut wrong_refs).is_err()
            );
            assert_eq!(wrong_reader.position(), 0);
            assert_eq!(wrong_refs, before);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let error = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            refs.register(75);
            assert!(
                read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).is_err(),
                "{cut}"
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
        }
    }
}

#[test]
fn geometric_object_branch_reuses_other_module_xform_and_limits_nesting() {
    use mxd2qgz::objects::geodata_xform::{
        GEOMETRIC_XFORM_CLASS_ID, GeometricTransformation10_1, SPLINE_XFORM_CLASS_ID,
    };

    fn geometric_header(bytes: &mut Vec<u8>, key: u32) {
        header(bytes, GEOMETRIC_XFORM_CLASS_ID, key);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 32]);
        bytes.push(1);
    }

    let mut fixture = Vec::new();
    geometric_header(&mut fixture, 50);
    let spline_offset = fixture.len();
    header(&mut fixture, SPLINE_XFORM_CLASS_ID, 90);
    fixture.extend(1_i32.to_le_bytes());
    fixture.extend([0; 48]);
    let mut reader = Reader::new(&fixture);
    let mut refs = RefTable10_1::new();
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Geometric(geometric) = *slot.body.unwrap() else {
        panic!("应读取 GeometricXform");
    };
    assert!(matches!(
        geometric.transformation,
        GeometricTransformation10_1::RasterObject { body, .. }
            if matches!(*body, GeodataXformBody10_1::Spline { version: 1, .. })
    ));
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(50) && refs.contains(90));

    let mut bytes = Vec::new();
    for key in 50..66 {
        geometric_header(&mut bytes, key);
    }
    bytes.extend(&fixture[spline_offset..]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn projective_preserves_negative_count_when_arrays_are_absent() {
    use mxd2qgz::objects::geodata_xform::PROJECTIVE_XFORM_CLASS_ID;

    for flag in [0_i32, 1] {
        let mut bytes = Vec::new();
        header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 80);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 32]);
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(flag.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_geodata_xform_slot_10_1(&mut reader, 256, &mut refs);
        if flag == 0 {
            let GeodataXformBody10_1::Projective(body) = *result.unwrap().body.unwrap() else {
                panic!("缺少 ProjectiveXform");
            };
            assert_eq!(body.point_count, -1);
            assert!(body.point_arrays.is_none());
            assert_eq!(reader.position(), end);
            assert!(refs.contains(80));
        } else {
            assert!(result.is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn unbounded_projective_empty_third_slot_is_rejected_as_ambiguous() {
    use mxd2qgz::objects::geodata_xform::PROJECTIVE_XFORM_CLASS_ID;

    let mut bytes = Vec::new();
    header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 80);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]); // 空空间参考。
    bytes.extend([0; 16]); // 空几何集合。
    bytes.extend([0; 16]); // Load 布局的空投影变换第三槽。
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    for coordinate in [1.25_f64, 2.0, 3.0, -4.0] {
        bytes.extend(coordinate.to_le_bytes());
    }

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn unbounded_projective_two_slot_zero_tail_remains_readable() {
    use mxd2qgz::objects::geodata_xform::PROJECTIVE_XFORM_CLASS_ID;

    let mut bytes = Vec::new();
    header(&mut bytes, PROJECTIVE_XFORM_CLASS_ID, 80);
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 32]); // 空空间参考与空第二个对象槽。
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.push(0x5a);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let slot = read_geodata_xform_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
    let GeodataXformBody10_1::Projective(body) = *slot.body.unwrap() else {
        panic!("缺少 ProjectiveXform");
    };
    assert_eq!(body.point_count, 0);
    assert_eq!(body.points_flag, 0);
    assert_eq!(reader.read_bytes(1).unwrap(), [0x5a]);
    assert!(refs.contains(80));
}
