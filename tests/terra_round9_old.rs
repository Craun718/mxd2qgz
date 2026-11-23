//! 旧分支的独立合成边界与 SDK 候选错位核查；不把局部匹配视为宿主解析。
use mxd2qgz::{
    layers::{
        feature_layer_tail::read_field_info_body_10_1,
        group_layer::{GroupChildEntry10_1, read_group_layer_prefix_10_1},
    },
    objects::envelope::{ENVELOPE_CLASS_ID, read_envelope_10_1},
    storage::object_refs::RefTable10_1,
    stream::reader::Reader,
};

#[test]
fn field_info_v0_v1_truncation_and_tail_are_distinct_from_real_evidence() {
    for version in [0_u16, 1] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(0xffff_u16.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend([b'A', 0, 0, 0]);
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.extend([0x5a; 4]);
        let mut table = RefTable10_1::new();
        table.register(17);
        let mut reader = Reader::new(&bytes);
        let result = read_field_info_body_10_1(&mut reader, 1024, &mut table, 17).unwrap();
        assert_eq!(result.version, version);
        assert_eq!(result.alias, "A");
        assert_eq!(result.short_14, None);
        assert_eq!(result.short_18, None);
        assert_eq!(result.tail_shorts, None);
        assert!(result.stop.is_none());
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(4).unwrap(), [0x5a; 4]);
        for n in 0..end {
            let mut reader = Reader::new(&bytes[..n]);
            let mut table = RefTable10_1::new();
            table.register(17);
            assert!(
                read_field_info_body_10_1(&mut reader, 1024, &mut table, 17).is_err(),
                "v{version} 截断 {n}"
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(table.len(), 1);
            assert!(table.contains(17));
        }
    }
}

#[test]
fn group_v10_plain_null_child_has_no_extrasafe_prefix() {
    let mut bytes = 10_u16.to_le_bytes().to_vec();
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend([0; 2]);
    bytes.extend([0; 6 + 16]);
    bytes.extend(1_u32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(0xffff_u16.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(0_f64.to_le_bytes());
    let end = bytes.len();
    bytes.extend([0x5a; 4]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(result.version, 10);
    assert_eq!(result.children, [GroupChildEntry10_1::Null]);
    assert_eq!(reader.position(), end);
    for n in 0..end {
        let mut reader = Reader::new(&bytes[..n]);
        let mut table = RefTable10_1::new();
        table.register(17);
        assert!(
            read_group_layer_prefix_10_1(&mut reader, 1024, &mut table).is_err(),
            "截断 {n}"
        );
        assert_eq!(reader.position(), 0);
        assert!(table.contains(17));
        assert_eq!(table.len(), 1);
    }
}

#[test]
fn envelope_v0_flags_and_complete_boundary() {
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    for n in [1., 2., 3., 4.] {
        bytes.extend(f64::to_le_bytes(n));
    }
    bytes.extend(0_u32.to_le_bytes());
    let end = bytes.len();
    bytes.extend([0x5a; 4]);
    let mut reader = Reader::new(&bytes);
    let result = read_envelope_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(result.version, 0);
    assert_eq!(result.coords, [1., 2., 3., 4.]);
    assert_eq!(reader.position(), end);
}

#[test]
#[ignore = "需要 10.1 SDK 介质提取的 Continents.mxd"]
fn sdk_toolbox_candidate_may_be_shifted_version_three() {
    use std::io::Read;
    let path =
        "temp/terra-round8-old/sdk-dotnet/Continents.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2";
    let mut file = cfb::open(path).unwrap();
    let mut bytes = Vec::new();
    file.open_stream("/ESRI ArcToolbox")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    for (offset, finite) in [(0x306a6, true), (0x5b73c, false)] {
        assert_eq!(&bytes[offset..offset + 16], &ENVELOPE_CLASS_ID);
        assert_eq!(
            u16::from_le_bytes(bytes[offset + 20..offset + 22].try_into().unwrap()),
            0
        );
        assert_eq!(
            u16::from_le_bytes(bytes[offset + 16..offset + 18].try_into().unwrap()),
            3
        );
        let coords: Vec<_> = (0..4)
            .map(|i| {
                f64::from_le_bytes(
                    bytes[offset + 18 + i * 8..offset + 26 + i * 8]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect();
        if finite {
            assert_eq!(coords[0], -180.0);
            assert!((coords[1] + 89.9).abs() < 0.001);
            assert!((coords[2] - 180.0).abs() < 0.001);
            assert!((coords[3] - 83.6236).abs() < 0.001);
        } else {
            assert!(coords.iter().all(|v| v.is_nan()));
        }
        assert_eq!(&bytes[offset + 50..offset + 54], &[0; 4]);
        // 其后连续的类标识与地理坐标系类标识相同；这里不假设对象头格式。
        assert_eq!(
            &bytes[offset + 54..offset + 70],
            &[
                0x80, 0x7a, 0xa8, 0xa6, 0xd1, 0x1d, 0xb2, 0x11, 0xbf, 0x51, 0x08, 0x00, 0x20, 0x22,
                0xf5, 0x73
            ]
        );
    }
}

#[test]
#[ignore = "需要已提取的 8.1 calibrate.mxd"]
fn old_map_v19_reads_calibrate_across_maps_and_page_layout() {
    use mxd2qgz::{
        layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
        maps::map_object::{MAP_CLASS_ID, read_map_prefix_10_1},
        storage::object_refs::{ObjectEntry10_1, read_object_entry_10_1},
    };
    use std::io::Read;
    let path = "temp/arcmap108-media-mxd/calibrate.mxd.73A84E22_FF16_48CA_BD86_B36A72D04488";
    let mut file = cfb::open(path).unwrap();
    let mut bytes = Vec::new();
    file.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(reader.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let prefix = read_map_prefix_10_1(&mut reader, 16_000_000).unwrap();
    assert_eq!(prefix.version, 19);
    assert_eq!(reader.position(), 0x52);
    // Map.dll 10.1 的 794038a0：旧列表一、列表二、活动索引。
    assert_eq!(reader.read_u32_le().unwrap(), 0);
    assert_eq!(reader.read_u32_le().unwrap(), 0);
    assert_eq!(reader.read_u32_le().unwrap(), u32::MAX);
    assert_eq!(reader.position(), 0x5e);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
            key: 32
        }
    ));

    let mut page = Vec::new();
    file.open_stream("/PageLayout")
        .unwrap()
        .read_to_end(&mut page)
        .unwrap();
    let report = mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1(&bytes, Some(&page)).unwrap();
    assert_eq!(report.confirmed_tail, Some((0x4db, 0)));
    assert_eq!(report.layer_list.unwrap().legacy_counts, Some((0, Some(0))));
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 13);
    assert_eq!(layout.consumed_end_offset(), page.len());
    assert_eq!(layout.map_frames[0].prefix.frame_key, 2047);
    assert_eq!(
        layout.map_frames[0].prefix.map,
        ObjectEntry10_1::BackReference {
            class_id: MAP_CLASS_ID,
            key: 31
        }
    );
    assert!(report.map.unknown_fields.is_empty());
}

#[test]
fn old_field_info_number_format_uses_the_registered_class() {
    use mxd2qgz::layers::feature_layer_tail::{
        FieldInfoStop10_1, NUMERIC_FORMAT_CLASS_ID, NumberFormatSlot10_1,
    };
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    for version in [0_u16, 1] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 2]);
        bytes.extend([2, 0, 0, 0, 0, 0]);
        let slot_start = bytes.len();
        bytes.extend(NUMERIC_FORMAT_CLASS_ID);
        bytes.extend(17_u32.to_le_bytes());
        bytes.push(0x42);
        for class_id in [NUMERIC_FORMAT_CLASS_ID, ENVELOPE_CLASS_ID] {
            let mut header = class_id.to_vec();
            header.extend(17_u32.to_le_bytes());
            let mut table = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&header), &mut table).unwrap();
            let saved = table.clone();
            let mut payload = bytes.clone();
            if class_id == NUMERIC_FORMAT_CLASS_ID {
                payload[slot_start..slot_start + 16].copy_from_slice(&ENVELOPE_CLASS_ID);
            }
            let mut reader = Reader::new(&payload);
            let field = read_field_info_body_10_1(&mut reader, 1024, &mut table, 18).unwrap();
            if class_id == NUMERIC_FORMAT_CLASS_ID {
                assert!(field.stop.is_none());
                assert!(matches!(
                    field.number_format,
                    NumberFormatSlot10_1::BackReference {
                        class_id: NUMERIC_FORMAT_CLASS_ID,
                        key: 17,
                    }
                ));
                assert_eq!(reader.read_bytes(1).unwrap(), &[0x42]);
            } else {
                assert!(matches!(
                    field.stop,
                    Some(FieldInfoStop10_1::NumberFormatUnsupported {
                        class_id: ENVELOPE_CLASS_ID,
                        key: 17,
                    })
                ));
                assert_eq!(reader.position(), slot_start);
            }
            assert_eq!(table, saved);
        }
    }
}
