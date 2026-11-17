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
fn africa_from_map_root_reaches_first_prepare_result_layer() {
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
    assert_eq!(p.feature_keys, vec![42]);
    assert_eq!(
        p.stop,
        BasemapStop10_1::NewObject {
            offset: 0x145d,
            class_id: MAP_CLASS_ID,
            key: 91
        }
    );
    assert_eq!(r.position(), 0x145d);
    assert!(refs.contains(41));
    assert!(refs.contains(42));
    assert!(!refs.contains(91));
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
    let unknown = entry.body.expect("保留未解析的 Basemap 体");
    assert_eq!(unknown.offset, 0x145d);
    assert_eq!(unknown.offset + unknown.len, 0x65fd);
    assert_eq!(
        unknown.basis,
        mxd2qgz::layers::layer_list::BodyBoundBasis::PrefixAInference
    );
    assert!(entry.parse_note.as_ref().unwrap().contains("键 [42]"));
    assert!(refs.contains(42));
    assert!(!refs.contains(91));
    use mxd2qgz::layers::legend_item::{BaseLegendItemStop10_1, LegendItemSlot10_1 as LayerSlot};
    use mxd2qgz::maps::{
        map_object::MapSurroundElement10_1,
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
    assert!(matches!(
        legend.stop,
        LegendStop10_1::LegendItemBodyIncomplete {
            index: 1,
            reason: BaseLegendItemStop10_1::SlotUnsupported {
                class_id: mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID,
                key: 140,
                ..
            },
        }
    ));
    assert_eq!(r.position(), 0xa13d);
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
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(result.results_count, Some(0));
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(
        result.stop,
        BasemapStop10_1::AfterResults {
            offset: bytes.len()
        }
    );
    bytes[0..2].copy_from_slice(&12u16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    assert!(read_basemap_prefix_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}
