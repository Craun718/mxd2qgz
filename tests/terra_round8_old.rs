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

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_feature_layers_consume_server_extensions_and_their_tails() {
    use mxd2qgz::layers::feature_layer_tail::{LayerExtension10_1, SuccessorStop10_1};
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;

    for (file, expected) in [
        ("MontgomerySimple", [(0x4121, 232), (0x4fcb, 265)]),
        ("airportgolf", [(0x2cf, 59), (0x1120, 88)]),
    ] {
        let map = read_sdk_map(&format!(
            "temp/terra-round8-old/sdk-dotnet/{file}.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2"
        ));
        let mut found = Vec::new();
        for entry in map.layer_list.entries {
            let Some(LayerBodyPrefix10_1::FeatureLayer(layer)) = entry.parsed else {
                continue;
            };
            let Some(tail) = layer.successor else {
                continue;
            };
            for extension in tail.extensions {
                let LayerExtension10_1::ServerLayerExtension(Some(extension)) = extension else {
                    continue;
                };
                let ObjectEntry10_1::NewObject {
                    class_id: PROPERTY_SET_CLASS_ID,
                    key,
                } = extension.properties.entry
                else {
                    panic!("预期服务扩展定义一个属性集");
                };
                assert_eq!(extension.version, 1);
                assert_eq!(extension.properties.body.unwrap().count, 1);
                assert_eq!(tail.stop, SuccessorStop10_1::TailCompleted);
                found.push((entry.offset, key));
            }
        }
        assert_eq!(found, expected);
    }
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_maps_cross_the_masking_stage_with_opaque_native_layers() {
    for (path, offset, trailer) in [
        (
            "sdk-dotnet/Montgomery.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
            0x1d7c4,
            1,
        ),
        (
            "sdk-dotnet/MontgomerySimple.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
            0x9484,
            0,
        ),
        (
            "sdk-java/portland.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
            0x77373,
            1,
        ),
    ] {
        let mut doc = cfb::open(format!("temp/terra-round8-old/{path}")).unwrap();
        let mut maps = Vec::new();
        doc.open_stream("/Maps")
            .unwrap()
            .read_to_end(&mut maps)
            .unwrap();
        let report = mxd2qgz::maps::mxd_maps::parse_maps_10_1(&maps).unwrap();
        assert_eq!(report.confirmed_tail, Some((offset, trailer)), "{path}");
        assert_eq!(offset + 4, maps.len(), "{path}");
    }
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_process_layer_reads_its_delegate_and_map_mask() {
    use mxd2qgz::layers::group_layer::GroupChildEntry10_1;

    let map = read_sdk_map(
        "temp/terra-round8-old/sdk-dotnet/USA_Data_ClipTools.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
    );
    let entry = &map.layer_list.entries[2];
    assert_eq!(entry.offset, 0x1a68);
    assert_eq!(entry.body.unwrap().len, 0);
    let Some(LayerBodyPrefix10_1::ProcessLayer(body)) = &entry.parsed else {
        panic!("预期完整 ProcessLayer");
    };
    assert_eq!(
        (body.version, body.group_key, body.group.version),
        (1, 114, 11)
    );
    assert_eq!(body.process_stream.len(), 14_513);
    assert_eq!(body.trailing_short, 0);
    assert!(matches!(
        &body.group.children[..],
        [GroupChildEntry10_1::SkippedNewObject {
            key: 115,
            body_length: 0,
            parsed: Some(parsed),
            ..
        }] if matches!(parsed.as_ref(), LayerBodyPrefix10_1::FeatureLayer(_))
    ));
    let [(key, masking)] = &map.post_border_tail.layer_masking[..] else {
        panic!("预期恰好读取 ProcessLayer 自身的遮罩记录");
    };
    assert_eq!((*key, masking.version, masking.tail), (113, 1, 0));
    assert!(masking.entries.is_empty());
    assert_eq!(map.stop, MapSuffixStop::Complete);
    assert_eq!(map.remaining_bytes, 4);
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_map_time_slots_read_three_extents_and_reach_the_stream_tail() {
    use mxd2qgz::layers::dep_tail::TimeSlot10_1;
    use mxd2qgz::maps::map_late_tail::LateMapValue10_1;
    use mxd2qgz::maps::mxd_maps::parse_maps_10_1;

    for (name, offsets, first_key, end, trailer) in [
        ("PrintMultiPage", [0x7f8, 0x873, 0x8fe], 41, 0x9a3, 0),
        ("airportgolf", [0x2422, 0x249d, 0x2528], 101, 0x25e5, 1),
    ] {
        let path = format!(
            "temp/terra-round8-old/sdk-dotnet/{name}.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2"
        );
        let map = read_sdk_map(&path);
        assert_eq!(map.stop, MapSuffixStop::Complete);
        let tail = map.post_border_tail.late_tail.unwrap();
        assert!(tail.stop.is_none());
        let mut found = Vec::new();
        for field in tail.fields {
            if let LateMapValue10_1::TimeExtent(body) = field.value {
                let (TimeSlot10_1::Body(start), TimeSlot10_1::Body(end)) = (body.start, body.end)
                else {
                    panic!("预期两个新 Time 对象");
                };
                assert_eq!((start.key, end.key), (body.key + 1, body.key + 2));
                assert_eq!(body.version, 2);
                found.push((field.offset, body.key));
            }
        }
        assert_eq!(
            found,
            offsets
                .into_iter()
                .zip([first_key, first_key + 3, first_key + 6])
                .collect::<Vec<_>>()
        );

        let mut doc = cfb::open(&path).unwrap();
        let mut maps = Vec::new();
        doc.open_stream("/Maps")
            .unwrap()
            .read_to_end(&mut maps)
            .unwrap();
        assert_eq!(
            parse_maps_10_1(&maps).unwrap().confirmed_tail,
            Some((end, trailer))
        );
        assert_eq!(maps.len(), end + 4);
    }
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_map_base_graphics_reads_polygon_elements() {
    use mxd2qgz::layers::composite_graphics::ElementEntry10_1;
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1_with_version;
    use mxd2qgz::objects::element_body::{ElementBody10_1, GeometrySlot10_1};
    use mxd2qgz::version::mxd_version::identify_version;

    for (path, key, end) in [
        (
            "sdk-dotnet/Gulf_of_St._Lawrence.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
            373,
            0x1e7fc,
        ),
        (
            "sdk-java/gulf_of_st._lawrence.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
            388,
            0x1d9d1,
        ),
    ] {
        let path = format!("temp/terra-round8-old/{path}");
        let map = read_sdk_map(&path);
        let graphics = map.base_graphics_layer_parsed.unwrap();
        let elements = &graphics.element_groups[0].elements;
        assert_eq!(elements.len(), 5);
        let ElementEntry10_1::Element {
            key: found,
            body: ElementBody10_1::PolygonElement(polygon),
        } = &elements[0]
        else {
            panic!("预期基本图形层中的 PolygonElement");
        };
        assert_eq!(*found, key);
        assert_eq!(polygon.version, 3);
        assert!(polygon.stop.is_none());
        assert!(matches!(polygon.geometry, GeometrySlot10_1::Polygon(_)));
        assert_eq!(map.stop, MapSuffixStop::Complete);

        let mut doc = cfb::open(&path).unwrap();
        let mut read_stream = |name: &str| {
            let mut bytes = Vec::new();
            doc.open_stream(name)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            bytes
        };
        let maps = read_stream("/Maps");
        let layout_bytes = read_stream("/PageLayout");
        let version = identify_version(&read_stream("/Version")).unwrap();
        let report =
            parse_mxd_streams_10_1_with_version(&maps, Some(&layout_bytes), Some(&version))
                .unwrap();
        assert_eq!(report.confirmed_tail, Some((end, 1)));
        assert_eq!(maps.len(), end + 4);
        let layout = report.page_layout.unwrap();
        assert!(layout.tail.is_some() && layout.tail_error.is_none());
        assert_eq!(layout.consumed_end_offset(), layout_bytes.len());
    }
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_hurricanes_reads_time_zone_rules_and_both_stream_ends() {
    use mxd2qgz::layers::dep_tail::{TimeReferenceSlot10_1, TimeZoneInfoSlot10_1};
    use mxd2qgz::layers::feature_layer_tail::SuccessorStop10_1;
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1_with_version;
    use mxd2qgz::version::mxd_version::identify_version;

    let path = "temp/terra-round8-old/sdk-dotnet/TimeAwareHurricanes.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2";
    let map = read_sdk_map(path);
    let entry = &map.layer_list.entries[0];
    assert_eq!(entry.offset, 0x154);
    assert_eq!(entry.body.unwrap().len, 0);
    let Some(LayerBodyPrefix10_1::FeatureLayer(layer)) = &entry.parsed else {
        panic!("预期 FeatureLayer");
    };
    let tail = layer.successor.as_ref().unwrap();
    assert_eq!(tail.stop, SuccessorStop10_1::TailCompleted);
    let data = tail.time_data.as_ref().unwrap();
    assert_eq!(data.version, 1);
    let TimeReferenceSlot10_1::Body(reference) = &data.slot else {
        panic!("预期 TimeReference 新对象");
    };
    let TimeZoneInfoSlot10_1::Body(zone) = &reference.zone_info else {
        panic!("预期 TimeZoneInfo 新对象");
    };
    assert_eq!((reference.key, zone.key, zone.version), (73, 74, 1));
    assert_eq!(zone.count, 2);
    assert_eq!(
        zone.rules.iter().map(|rule| rule.int_a).collect::<Vec<_>>(),
        [2006, 2007]
    );
    assert_eq!((reference.short_a, reference.short_b), (Some(0), Some(-1)));
    assert_eq!(tail.time_table_definition.as_ref().unwrap().version, 1);
    assert_eq!(map.stop, MapSuffixStop::Complete);

    let mut doc = cfb::open(path).unwrap();
    let mut read_stream = |name: &str| {
        let mut bytes = Vec::new();
        doc.open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = read_stream("/Maps");
    let layout_bytes = read_stream("/PageLayout");
    let version = identify_version(&read_stream("/Version")).unwrap();
    let result =
        parse_mxd_streams_10_1_with_version(&maps, Some(&layout_bytes), Some(&version)).unwrap();
    assert_eq!(result.confirmed_tail, Some((0x2507, 0)));
    assert_eq!(maps.len(), 0x250b);
    let layout = result.page_layout.unwrap();
    assert!(layout.tail.is_some() && layout.tail_error.is_none());
    assert_eq!(layout.consumed_end_offset(), layout_bytes.len());
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_print_multi_page_reads_a_nested_map_frame_and_cross_stream_reference() {
    use mxd2qgz::maps::group_element::GroupChild10_1;
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1;

    let path =
        "temp/terra-round8-old/sdk-dotnet/PrintMultiPage.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2";
    let mut doc = cfb::open(path).unwrap();
    let mut read = |name: &str| {
        let mut bytes = Vec::new();
        doc.open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = read("/Maps");
    let mut page = read("/PageLayout");
    let result = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    assert_eq!(result.confirmed_tail, Some((0x9a3, 0)));
    let layout = result.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 1);
    let group = &layout.groups[0];
    assert_eq!((group.key, group.children.len()), (51, 7));
    let frame = group
        .children
        .iter()
        .find_map(|child| match child {
            GroupChild10_1::MapFrame(frame) => Some(frame),
            _ => None,
        })
        .unwrap();
    assert_eq!((frame.prefix.frame_key, frame.prefix.version), (71, 4));
    assert_eq!(frame.prefix.map_offset, 0x1fc3a);
    assert_eq!(
        frame.prefix.map,
        ObjectEntry10_1::BackReference {
            class_id: MAP_CLASS_ID,
            key: 29
        }
    );
    assert!(layout.tail.is_some() && layout.tail_error.is_none());
    assert_eq!(layout.consumed_end_offset(), page.len());

    // 30 是已见的基本图形层键，不能当成根 Map 回引。
    page[0x1fc4a..0x1fc4e].copy_from_slice(&30_u32.to_le_bytes());
    let layout = parse_mxd_streams_10_1(&maps, Some(&page))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(layout.end_offset, 0x1e);
    assert!(layout.groups.is_empty());
    assert!(layout.stop.unwrap().contains("尚未确认的根 Map 键"));
}

#[test]
#[ignore = "需要 10.1 SDK 安装介质提取文件"]
fn sdk_schematics_reuses_nested_map_in_the_root_list() {
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1;

    let path = "temp/terra-round8-old/sdk-dotnet/SchematicsForServerSample.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2";
    let map = read_sdk_map(path);
    let slot = map
        .post_border_tail
        .persist_collections
        .unwrap()
        .map_interface
        .unwrap();
    assert_eq!(slot.offset, 0xcbe5);
    assert_eq!(
        slot.entry,
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            key: 533
        }
    );
    let (prefix, suffix) = *slot.body.unwrap();
    assert_eq!(prefix.version, 45);
    assert_eq!(suffix.stop, MapSuffixStop::Complete);
    assert_eq!(suffix.layer_list.count, 1);

    let mut doc = cfb::open(path).unwrap();
    let mut read = |name: &str| {
        let mut bytes = Vec::new();
        doc.open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = read("/Maps");
    let page = read("/PageLayout");
    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    assert_eq!(report.confirmed_tail, Some((0x2c4a7, 1)));
    assert_eq!(maps.len(), 0x2c4ab);
    assert_eq!(report.map.frames.len(), 4);
    assert_eq!(report.map.frames[3].name, prefix.name);
    let layout = report.page_layout.unwrap();
    assert!(layout.tail.is_some() && layout.tail_error.is_none());
    assert_eq!(layout.consumed_end_offset(), page.len());
}

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
