use mxd2qgz::{
    layers::basemap_layer::{BASEMAP_LAYER_CLASS_ID, BasemapStop10_1, read_basemap_prefix_10_1},
    maps::map_object::MAP_CLASS_ID,
    objects::spatial_reference::{
        PROJECTED_COORDINATE_SYSTEM_CLASS_ID, read_projected_coordinate_system_10_1,
    },
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::{native_string::read_string_10_1, reader::Reader},
};
use std::io::Read;

#[test]
#[ignore = "需要本地 10.1 安装介质 Africa.mxd"]
fn africa_from_map_root_reads_prepare_results_and_sublayers() {
    let mut doc = cfb::open("temp/arcmap101-media-mxd/Africa.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut r = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(r.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    r.read_u16_le().unwrap();
    read_string_10_1(&mut r, 1_000_000).unwrap();
    read_string_10_1(&mut r, 1_000_000).unwrap();
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            key: 39
        }
    ));
    read_projected_coordinate_system_10_1(&mut r, 1_000_000).unwrap();
    assert_eq!(r.read_u32_le().unwrap(), 1);
    assert_eq!(r.read_u16_le().unwrap(), 3);
    r.read_bytes(8).unwrap();
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: BASEMAP_LAYER_CLASS_ID,
            key: 40
        }
    ));
    assert_eq!(r.position(), 0x220);
    let p = read_basemap_prefix_10_1(&mut r, 1_000_000, &mut refs).unwrap();
    assert_eq!(p.name, "Basemap");
    assert_eq!(p.results_count, Some(6));
    assert_eq!(p.feature_keys, [42, 105, 140, 176, 222, 258]);
    assert_eq!(p.sublayers.len(), 6);
    for (&key, sublayer) in p.feature_keys.iter().zip(&p.sublayers) {
        assert_eq!(
            *sublayer,
            ObjectEntry10_1::BackReference {
                class_id: mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID,
                key,
            }
        );
        assert!(refs.contains(key));
    }
    assert_eq!(p.stop, BasemapStop10_1::Complete);
    assert_eq!(r.position(), 0x65fd);
    assert!(refs.contains(41));
    assert!(refs.contains(91));
}

#[test]
fn truncated_prefix_rolls_back_reader_and_references() {
    let mut r = Reader::new(&[11, 0, 2, 0]);
    let mut refs = RefTable10_1::new();
    assert!(read_basemap_prefix_10_1(&mut r, 1024, &mut refs).is_err());
    assert_eq!(r.position(), 0);
    assert!(refs.is_empty());
}

#[test]
#[ignore = "需要本地 10.1 安装介质 Africa.mxd"]
fn africa_map_root_uses_basemap_refs_in_legend() {
    let mut doc = cfb::open("temp/arcmap101-media-mxd/Africa.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut r = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(r.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let suffix =
        mxd2qgz::maps::map_object::read_map_suffix_10_1(&mut r, 1_000_000, &mut refs).unwrap();
    let entry = &suffix.layer_list.entries[0];
    let unknown = entry.body.expect("保留 Basemap 的 ExtraSafe 边界");
    assert_eq!(unknown.offset, 0x65fd);
    assert_eq!(unknown.len, 0);
    assert!(refs.contains(91));
    assert!(refs.contains(140));
    use mxd2qgz::layers::legend_item::LegendItemSlot10_1 as LayerSlot;
    use mxd2qgz::maps::{
        map_object::{MapSuffixStop, MapSurroundElement10_1},
        map_surround::{LegendItemSlot10_1 as LegendEntry, LegendStop10_1},
    };
    let Some(MapSurroundElement10_1::Legend(legend)) = &suffix.map_surround_parsed[0] else {
        panic!("未读取 Legend")
    };
    let LegendEntry::HorizontalLegendItem(first) = &legend.items[0].entry else {
        panic!("未读取首个图例条目")
    };
    assert!(matches!(
        first.layer,
        LayerSlot::BackReference {
            class_id: mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID,
            key: 42
        }
    ));
    assert!(first.stop.is_none());
    assert_eq!(legend.stop, LegendStop10_1::Completed);
    assert_eq!(legend.items.len(), 3);
    let [(key, masking)] = suffix.post_border_tail.layer_masking.as_slice() else {
        panic!("应读取首个图层的独立遮罩段");
    };
    assert_eq!((*key, masking.version, masking.tail), (40, 2, 0));
    assert!(masking.entries.is_empty());
    assert_eq!(suffix.stop, MapSuffixStop::Complete);
    assert_eq!(r.position(), 0xc510);
    assert_eq!(bytes.len() - r.position(), 4);
    let mut layout_bytes = Vec::new();
    doc.open_stream("/PageLayout")
        .unwrap()
        .read_to_end(&mut layout_bytes)
        .unwrap();
    let report =
        mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1(&bytes, Some(&layout_bytes)).unwrap();
    let layout = report.page_layout.unwrap();
    assert!(layout.stop.is_none(), "{:?}", layout.stop);
    assert_eq!(layout.consumed_elements, 7);
    assert_eq!(layout.map_frames.len(), 1);
    assert_eq!(
        layout.map_frames[0].prefix.map,
        ObjectEntry10_1::BackReference {
            class_id: MAP_CLASS_ID,
            key: 38
        }
    );
}

#[test]
fn empty_results_and_invalid_version_are_bounded() {
    let mut bytes = Vec::new();
    bytes.extend(6u16.to_le_bytes());
    bytes.extend(2u32.to_le_bytes());
    bytes.extend([0u8; 2]);
    bytes.extend([0u8; 6 + 18 + 16]);
    bytes.extend([0u8; 16]);
    bytes.extend([0u8; 16]);
    bytes.extend([0u8; 2]);
    bytes.extend([0u8; 4]);
    bytes.extend(0u32.to_le_bytes()); // 正式子图层计数
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(result.results_count, Some(0));
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(result.stop, BasemapStop10_1::Complete);
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    bytes[0..2].copy_from_slice(&12u16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    assert!(read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn globe_graphics_reads_basemap_through_its_ordinary_ilayer_slot() {
    use mxd2qgz::layers::graphics_3d::{
        GLOBE_GRAPHICS_LAYER_CLASS_ID, read_graphics_3d_layer_10_1,
    };
    use mxd2qgz::layers::layer_list::LayerBodyPrefix10_1;

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend([0; 2 + 4]);
    bytes.extend(BASEMAP_LAYER_CLASS_ID);
    bytes.extend(91_u32.to_le_bytes());
    bytes.extend(6_u16.to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend([0; 2 + 6 + 18 + 16 + 16 + 16 + 2 + 4 + 4]);
    bytes.extend([0; 16 + 4 + 4]);
    bytes.extend([1, 0, 2, 0]);
    let end = bytes.len();
    bytes.extend([0xa5, 0x5a]);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_graphics_3d_layer_10_1(&GLOBE_GRAPHICS_LAYER_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    assert!(matches!(
        parsed.associated_layer_body.as_deref(),
        Some(LayerBodyPrefix10_1::BasemapLayer(layer)) if layer.stop == BasemapStop10_1::Complete
    ));
    assert!(refs.contains(91));
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(2).unwrap(), [0xa5, 0x5a]);
}

#[test]
fn formal_basemap_sublayer_consumes_bounded_group() {
    use mxd2qgz::layers::group_layer::GROUP_LAYER_CLASS_ID;
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend([0; 2 + 6 + 18]);
    bytes.extend(1_u32.to_le_bytes());
    let slot_offset = bytes.len();
    let mut group = 1_u16.to_le_bytes().to_vec();
    group.extend(2_u32.to_le_bytes());
    group.extend([0; 2 + 6 + 16 + 4]);
    let size = (20 + group.len()) as u32;
    bytes.extend(size.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(GROUP_LAYER_CLASS_ID);
    bytes.extend(77_u32.to_le_bytes());
    bytes.extend(group);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(body.stop, BasemapStop10_1::Complete);
    assert_eq!(body.sublayer_bodies[0].0, 77);
    assert!(refs.contains(77));
    assert_eq!(reader.remaining(), 0);

    // 声明区间有一个多余字节时，回退到子层封装起点。
    bytes[slot_offset..slot_offset + 4].copy_from_slice(&(size + 1).to_le_bytes());
    bytes.push(0);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        body.stop,
        BasemapStop10_1::NewObject { key: 77, .. }
    ));
    assert!(refs.is_empty());
    assert_eq!(reader.position(), slot_offset);
}

#[test]
fn basemap_prepare_results_use_native_signed_versions_and_counts() {
    for (count, result) in [(-1_i32, false), (1, true)] {
        let mut bytes = 6_u16.to_le_bytes().to_vec();
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend([0; 2 + 6 + 18 + 16 + 16 + 16 + 2]);
        bytes.extend(count.to_le_bytes());
        if result {
            bytes.extend((-1_i16).to_le_bytes());
            bytes.extend([0; 8 + 16]);
            bytes.extend((-1_i32).to_le_bytes());
        }
        bytes.extend(0_i32.to_le_bytes()); // 正式子层数量。
        let mut reader = Reader::new(&bytes);
        let body = read_basemap_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(body.stop, BasemapStop10_1::Complete);
        assert_eq!(body.results_count, Some(count as u32));
        assert_eq!(reader.position(), bytes.len());
    }
}

#[test]
fn basemap_prepare_results_read_converter_option_and_reuse_its_key() {
    let converter = [
        0x24, 0x42, 0x31, 0x97, 0xa2, 0x2d, 0x39, 0x49, 0xac, 0xc5, 0xdd, 0x7c, 0x0a, 0x7b, 0xfc,
        0x8b,
    ];
    let mut bytes = 6_u16.to_le_bytes().to_vec();
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend([0; 2 + 6 + 18 + 16 + 16 + 16 + 2]);
    bytes.extend(2_i32.to_le_bytes());
    for is_new in [true, false] {
        bytes.extend(2_i16.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend(if is_new { converter } else { [0x93; 16] });
        bytes.extend(73_u32.to_le_bytes());
        if is_new {
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend([0; 4 + 16]);
        }
        bytes.extend(0_i32.to_le_bytes());
    }
    bytes.extend(0_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let body = read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(body.stop, BasemapStop10_1::Complete);
    assert!(refs.contains(73));
    assert_eq!(reader.position(), bytes.len());
}
