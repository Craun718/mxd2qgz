use mxd2qgz::{
    layers::{
        feature_layer::{FEATURE_LAYER_CLASS_ID, FeatureLayerStop10_1},
        feature_layer_tail::{DisplayFilterSlot10_1, SuccessorStop10_1},
        layer_list::{LayerBodyPrefix10_1, LayerEntryObject},
    },
    maps::map_object::{MAP_CLASS_ID, read_map_suffix_10_1},
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::reader::Reader,
};
use std::io::Read;

#[test]
#[ignore = "需要 10.1 SDK Java 安装介质提取文件"]
fn sdk_java_portland_featurelayer_reaches_all_four_timedata_segments() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/temp/terra-round8-old/sdk-java/portland.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
    );
    let mut doc = cfb::open(path).unwrap();
    let mut version_bytes = Vec::new();
    doc.open_stream("/Version")
        .unwrap()
        .read_to_end(&mut version_bytes)
        .unwrap();
    let version =
        mxd2qgz::version::version_stream::read_layout_10_1(&mut Reader::new(&version_bytes), 4096)
            .unwrap();
    assert_eq!(
        (version.text.as_str(), version.compatibility_short),
        ("10.0", 18)
    );
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(reader.read_u32_le().unwrap(), 1, "首 Map 数量");
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut reader, 16_000_000, &mut refs).unwrap();

    let mut reached = Vec::new();
    let mut transparency_filter = None;
    for entry in &map.layer_list.entries {
        let Some(LayerBodyPrefix10_1::FeatureLayer(layer)) = &entry.parsed else {
            continue;
        };
        let Some(successor) = &layer.successor else {
            continue;
        };
        if layer.version < 0x20 {
            continue;
        }
        reached.push(entry.offset);
        assert_eq!(layer.version, 34);
        assert!(matches!(
            entry.object,
            LayerEntryObject::Entry(ObjectEntry10_1::NewObject {
                class_id: FEATURE_LAYER_CLASS_ID,
                ..
            })
        ));
        assert!(matches!(
            layer.stop,
            FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted)
        ));

        let time_data = successor.time_data.as_ref().expect("TimeData 段应存在");
        assert_eq!(time_data.version, 1);
        assert!(matches!(
            time_data.slot,
            mxd2qgz::layers::dep_tail::TimeReferenceSlot10_1::Null
        ));
        let table = successor
            .time_table_definition
            .as_ref()
            .expect("TimeTableDefinition 段应存在");
        assert_eq!(table.version, 1);
        assert!(matches!(
            table.slot,
            mxd2qgz::layers::dep_tail::TimeExtentSlot10_1::Null
        ));
        assert_eq!(
            successor
                .time_dimension_definition
                .as_ref()
                .unwrap()
                .version,
            1
        );
        let display = successor
            .time_data_display
            .as_ref()
            .expect("TimeDataDisplay 段应存在");
        assert_eq!(display.version, 1);
        assert_eq!((display.int_a, display.int_b), (4, 8));
        assert!(matches!(successor.stop, SuccessorStop10_1::TailCompleted));
        if let Some(DisplayFilterSlot10_1::Transparency {
            key: 938,
            version,
            base_version,
            ..
        }) = successor.display_filter
        {
            transparency_filter = Some((version, base_version));
        }
    }
    assert_eq!(
        reached,
        [
            0x2da, 0x1c450, 0x386ed, 0x39458, 0x3a7b6, 0x3b38c, 0x3c150, 0x4d32a, 0x4e1ac, 0x6a00b,
            0x6af1d, 0x6be57, 0x6cb58, 0x6e402, 0x6f3ed, 0x700d9, 0x713af, 0x720cd
        ],
        "Map 宿主顺序中的 18 个 FeatureLayer 条目"
    );
    assert_eq!(transparency_filter, Some((1, 1)));
}
