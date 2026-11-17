use mxd2qgz::{
    layers::{feature_layer_tail::FIELD_INFO_CLASS_ID, group_layer::GROUP_LAYER_CLASS_ID},
    maps::standalone_table::STANDALONE_TABLE_CLASS_ID,
    objects::{
        envelope::ENVELOPE_CLASS_ID,
        geometry::{MULTIPOINT_CLASS_ID, POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID},
    },
};
use mxd2qgz::{
    layers::{feature_layer_tail::FieldInfoEntry10_1, layer_list::LayerBodyPrefix10_1},
    maps::{
        map_object::{MAP_CLASS_ID, MapSuffix10_1, MapSuffixStop, read_map_suffix_10_1},
        standalone_table::{StandaloneNameBody10_1, StandaloneTableStop10_1},
    },
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::reader::Reader,
};
use std::{collections::BTreeMap, io::Read, path::PathBuf};

fn read_sdk_map(path: &str) -> MapSuffix10_1 {
    let mut doc = cfb::open(path).unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut r = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert!(r.read_u32_le().unwrap() > 0);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    read_map_suffix_10_1(&mut r, 16_000_000, &mut refs).unwrap()
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_old_field_group_and_standalone_host_paths() {
    let continent = read_sdk_map(
        "temp/terra-round8-old/sdk-dotnet/Continents.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
    );
    assert_eq!(continent.layer_list.count, 1);
    let entry = &continent.layer_list.entries[0];
    assert_eq!(entry.offset, 0x154);
    let Some(LayerBodyPrefix10_1::FeatureLayer(feature)) = &entry.parsed else {
        panic!("未到达 FeatureLayer")
    };
    let successor = feature.successor.as_ref().unwrap();
    assert_eq!(successor.field_info_count, 5);
    let fields: Vec<_> = successor
        .field_infos
        .iter()
        .map(|item| match &item.slot {
            FieldInfoEntry10_1::FieldInfo(body) if body.stop.is_none() => (body.key, body.version),
            _ => panic!("FieldInfo 未完整消费"),
        })
        .collect();
    assert_eq!(fields, [(43, 2), (44, 2), (45, 2), (46, 2), (48, 2)]);

    let brazil = read_sdk_map(
        "temp/terra-round8-old/sdk-java/brazil.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
    );
    let groups: Vec<_> = brazil
        .layer_list
        .entries
        .iter()
        .filter_map(|entry| match &entry.parsed {
            Some(LayerBodyPrefix10_1::GroupLayer(group)) => Some((
                entry.offset,
                group.version,
                group.child_count,
                group.children.len(),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        groups,
        [(0x15a, 11, 0, 0), (0x1f6, 11, 7, 7), (0x8d70, 11, 13, 13)]
    );

    let modern = read_sdk_map(
        "temp/terra-round8-old/sdk-dotnet/SchematicsForServerSample.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
    );
    let versions: Vec<_> = modern
        .layer_list
        .entries
        .iter()
        .filter_map(|entry| match &entry.parsed {
            Some(LayerBodyPrefix10_1::GroupLayer(group)) => {
                Some((entry.offset, group.version, group.child_count))
            }
            _ => None,
        })
        .collect();
    assert_eq!(versions, [(0x162, 12, 7), (0xb234, 12, 1)]);

    let linear = read_sdk_map(
        "temp/terra-round8-old/sdk-java/LinearRef.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
    );
    assert_eq!(linear.post_bookmark.unwrap().standalone_table_count, 2);
    assert_eq!(linear.standalone_tables.len(), 2);
    let (slot, body) = &linear.standalone_tables[0];
    assert_eq!(slot.offset, 0x6a50);
    assert!(matches!(
        slot.entry,
        ObjectEntry10_1::NewObject {
            class_id: STANDALONE_TABLE_CLASS_ID,
            key: 331
        }
    ));
    let body = body.as_ref().unwrap();
    assert_eq!(body.version, 9);
    assert!(matches!(
        &body.dataset_name_body,
        Some(StandaloneNameBody10_1::TableName(name)) if name.version == 1
    ));
    assert_eq!(body.named_object_bodies[0].as_ref().unwrap().key, 334);
    assert_eq!(body.stop, StandaloneTableStop10_1::Complete);
    assert!(matches!(linear.stop, MapSuffixStop::Complete));
    assert!(linear.post_border_tail.post_masking.is_some());
}

// 扫描仅生成候选，后两字节未必是版本，也不能判断新对象与回引。
fn candidates(data: &[u8], id: &[u8; 16]) -> Vec<(usize, Option<u16>)> {
    data.windows(16)
        .enumerate()
        .filter(|(_, bytes)| *bytes == id)
        .map(|(offset, _)| {
            let version = data
                .get(offset + 20..offset + 22)
                .map(|b| u16::from_le_bytes([b[0], b[1]]));
            (offset, version)
        })
        .collect()
}

#[test]
fn candidate_scanner_preserves_truncated_and_overlapping_hits() {
    let id = [7; 16];
    let mut data = vec![7; 17];
    assert_eq!(candidates(&data, &id), [(0, None), (1, None)]);
    data.extend([0, 0, 0, 2, 0]);
    assert_eq!(candidates(&data, &id), [(0, Some(2)), (1, None)]);
    assert!(candidates(&data[..15], &id).is_empty());
}

#[test]
#[ignore = "需要两套已提取安装介质模板；输出仅供候选定位"]
fn inventory_all_compound_streams_including_later_maps() {
    let classes = [
        ("group", GROUP_LAYER_CLASS_ID),
        ("field", FIELD_INFO_CLASS_ID),
        ("polyline", POLYLINE_CLASS_ID),
        ("polygon", POLYGON_CLASS_ID),
        ("point", POINT_CLASS_ID),
        ("multipoint", MULTIPOINT_CLASS_ID),
        ("envelope", ENVELOPE_CLASS_ID),
        ("table", STANDALONE_TABLE_CLASS_ID),
    ];
    for (media, dir, expected) in [
        ("101", "temp/arcmap101-media-mxd", 66),
        ("101-extra", "temp/terra-round8-old/desktop-extra", 10),
        ("108", "temp/arcmap108-media-mxd", 68),
        ("sdk-dotnet", "temp/terra-round8-old/sdk-dotnet", 20),
        ("sdk-java", "temp/terra-round8-old/sdk-java", 17),
    ] {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_file() && !p.to_string_lossy().contains(".mxd.xml."))
            .collect();
        paths.sort();
        assert_eq!(paths.len(), expected);
        let mut total = BTreeMap::<(String, String, Option<u16>), usize>::new();
        let mut streams = BTreeMap::<String, usize>::new();
        let mut multi = 0;
        for path in paths {
            let mut doc = cfb::open(&path).unwrap();
            let entries: Vec<_> = doc
                .walk()
                .filter(|e| e.is_stream())
                .map(|e| e.path().to_owned())
                .collect();
            let mut version = Vec::new();
            doc.open_stream("/Version")
                .unwrap()
                .read_to_end(&mut version)
                .unwrap();
            if media.starts_with("sdk-") {
                println!(
                    "FILE media={media} file={} version={version:02x?}",
                    path.file_name().unwrap().to_string_lossy()
                );
            }
            for stream in entries {
                let mut bytes = Vec::new();
                doc.open_stream(&stream)
                    .unwrap()
                    .read_to_end(&mut bytes)
                    .unwrap();
                let kind = if stream == std::path::Path::new("/Maps") {
                    "Maps"
                } else if stream == std::path::Path::new("/PageLayout") {
                    "PageLayout"
                } else {
                    "other"
                };
                *streams.entry(kind.to_string()).or_default() += 1;
                if kind == "Maps"
                    && bytes
                        .get(..4)
                        .is_some_and(|b| u32::from_le_bytes(b.try_into().unwrap()) > 1)
                {
                    multi += 1;
                }
                for (name, id) in classes {
                    for (offset, candidate) in candidates(&bytes, &id) {
                        *total
                            .entry((kind.to_string(), name.to_string(), candidate))
                            .or_default() += 1;
                        let interesting = match name {
                            "group" | "table" => true,
                            "field" => candidate.is_some_and(|v| v <= 2),
                            "envelope" => candidate == Some(0),
                            _ => candidate.is_some_and(|v| v <= 1),
                        };
                        if interesting {
                            println!(
                                "CANDIDATE media={media} file={} stream={} off={offset:#x} class={name} next={candidate:?} version={version:02x?}",
                                path.file_name().unwrap().to_string_lossy(),
                                stream.display()
                            );
                        }
                    }
                }
            }
        }
        println!(
            "SUMMARY media={media} streams={streams:?} multiple_maps={multi} counts={total:?}"
        );
    }
}
