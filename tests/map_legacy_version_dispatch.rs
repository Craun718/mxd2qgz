use mxd2qgz::maps::map_object::MAP_CLASS_ID;
use mxd2qgz::maps::mxd_maps::parse_maps_10_1;

fn map_with_version(version: u16, suffix: &[u8]) -> (Vec<u8>, usize) {
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.extend(MAP_CLASS_ID);
    bytes.extend(7_u32.to_le_bytes());
    bytes.extend(version.to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0_u8; 16]);
    let offset = bytes.len();
    bytes.extend(suffix);
    (bytes, offset)
}

#[test]
fn legacy_versions_stop_at_prefix_and_preserve_all_remaining_bytes() {
    for version in [0, 19, 22] {
        let suffix = [0xff, 0xff, 0xff, 0xff, 1, 2, 3];
        let (bytes, offset) = map_with_version(version, &suffix);
        let report = parse_maps_10_1(&bytes).unwrap();
        assert!(report.layer_list.is_none());
        let frame = &report.map.frames[0];
        assert!(frame.children.is_empty());
        assert!(frame.extent.is_none());
        assert_eq!(frame.unknown_fields.len(), 1);
        assert_eq!(frame.unknown_fields[0].offset, offset as u64);
        assert_eq!(frame.unknown_fields[0].bytes, suffix);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|s| s.contains(&format!("Map 架构版本 {version} 小于 23")))
        );
    }
}

#[test]
fn version_23_enters_new_layout_and_rejects_invalid_count() {
    let (bytes, _) = map_with_version(23, &(-1_i32).to_le_bytes());
    let error = parse_maps_10_1(&bytes).unwrap_err();
    assert!(error.to_string().contains("计数"));
}

#[test]
fn legacy_zero_bytes_are_preserved_as_unknown() {
    // 根读取器还要窥读 4 字节，提供后缀后再检查准确边界。
    let (bytes, offset) = map_with_version(19, &[0; 4]);
    let report = parse_maps_10_1(&bytes).unwrap();
    assert_eq!(report.map.frames[0].unknown_fields[0].offset, offset as u64);
    assert_eq!(report.map.frames[0].unknown_fields[0].bytes, [0; 4]);
}

#[test]
fn legacy_map_v19_to_v22_handle_both_layer_lists_and_annotate_map() {
    use mxd2qgz::maps::map_object::{
        LegacyAnnotateMapBody10_1, MapSuffixStop, read_map_suffix_10_1,
    };
    use mxd2qgz::maps::maplex::{
        MAPLEX_ANNOTATE_MAP_CLASS_ID, MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID,
    };
    use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
    use mxd2qgz::stream::reader::Reader;

    let annotate_class = [
        2, 0x90, 0x43, 0x8c, 0xec, 0x14, 0xd2, 0x11, 0xa2, 0x7e, 8, 0, 9, 0xb6, 0xf2, 0x2b,
    ];
    for version in 19..=22 {
        let mut suffix = 1_i32.to_le_bytes().to_vec();
        suffix.extend(16_u32.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend([0; 16]); // 第一组的 ExtraSafe 空对象。
        suffix.extend(1_i32.to_le_bytes());
        suffix.extend([0; 16]); // 第二组的普通空对象。
        suffix.extend((-1_i32).to_le_bytes());

        let masking_offset = suffix.len() + if version == 22 { 16 } else { 0 };
        if matches!(version, 20 | 21) {
            suffix.extend(0_u32.to_le_bytes());
            if version == 21 {
                suffix.extend(0xffee_u16.to_le_bytes());
            }
        }
        suffix.extend([0; 16]); // 基本图形层。
        if version == 22 {
            suffix.extend(0_u32.to_le_bytes());
            suffix.extend(0xffee_u16.to_le_bytes());
        }
        suffix.extend([0; 4 + 32 + 8 + 16 + 4]); // 装饰物、范围、两个 LONG、IEnvelope、书签。
        suffix.extend(0xffff_u16.to_le_bytes());
        suffix.extend([0; 2 + 8 + 2 + 4 + 16 + 12]); // 属性前导、照明和后续定长成员。
        suffix.extend([0; 32]); // IGeometry 和 IBorder。
        suffix.extend([0; 2 + 16 + 2 + 16 + 2 + 2 + 8]); // 边框后字段。
        suffix.extend([0; 4 + 4 + 2 + 8 + 4]); // 标量和空变换列表。
        suffix.extend([0; 8 + 2 + 16]); // 变换后的 double、short 和 IBackground。
        let annotate_offset = suffix.len();
        suffix.extend(annotate_class);
        suffix.extend(90_u32.to_le_bytes());
        suffix.extend(1_u16.to_le_bytes());
        let end = suffix.len();
        suffix.extend(0x1234_u32.to_le_bytes());

        let (bytes, offset) = map_with_version(version, &suffix);
        let mut reader = Reader::new(&bytes[24..]);
        let mut refs = RefTable10_1::new();
        let parsed = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.stop, MapSuffixStop::Complete);
        assert_eq!(parsed.layer_list.legacy_counts, Some((1, Some(1))));
        assert_eq!(parsed.layer_list.entries.len(), 2);
        assert_eq!(parsed.layer_list.active_index, Some(-1));
        assert_eq!(
            parsed.legacy_masking_short,
            (version >= 21).then_some(0xffee)
        );
        let (annotate, version_on_disk) = parsed.post_border_tail.legacy_annotate_map.unwrap();
        assert!(matches!(
            annotate.entry,
            ObjectEntry10_1::NewObject { key: 90, .. }
        ));
        assert_eq!(
            version_on_disk,
            Some(LegacyAnnotateMapBody10_1::AnnotateMap { version: 1 })
        );
        assert!(refs.contains(90));
        assert_eq!(reader.read_u32_le().unwrap(), 0x1234);
        assert_eq!(
            parse_maps_10_1(&bytes).unwrap().confirmed_tail,
            Some((offset + end, 0x1234))
        );

        let mut bad = bytes.clone();
        bad[offset + end - 2..offset + end].copy_from_slice(&2_u16.to_le_bytes());
        let mut reader = Reader::new(&bad[24..]);
        let mut refs = RefTable10_1::new();
        let parsed = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_ne!(parsed.stop, MapSuffixStop::Complete);
        assert_eq!(reader.position(), offset + annotate_offset - 24);
        assert!(!refs.contains(90));

        if version >= 20 {
            bad[offset + masking_offset] = 1;
            let mut reader = Reader::new(&bad[24..]);
            assert!(read_map_suffix_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
        }

        if version == 19 {
            let mut legacy = suffix[..annotate_offset].to_vec();
            legacy.extend(MAPLEX_ANNOTATE_MAP_CLASS_ID);
            legacy.extend(91_u32.to_le_bytes());
            legacy.extend(2_i16.to_le_bytes());
            legacy.extend((-12_i32).to_le_bytes());
            legacy.extend(4_u16.to_le_bytes());
            legacy.extend(MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID);
            legacy.extend(92_u32.to_le_bytes());
            legacy.extend(1_i16.to_le_bytes());
            legacy.extend(23_i32.to_le_bytes());
            legacy.extend(5_u16.to_le_bytes());
            let (bytes, _) = map_with_version(version, &legacy);
            let mut reader = Reader::new(&bytes[24..]);
            let mut refs = RefTable10_1::new();
            let parsed = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(parsed.stop, MapSuffixStop::Complete);
            let (_, body) = parsed.post_border_tail.legacy_annotate_map.unwrap();
            let Some(LegacyAnnotateMapBody10_1::Maplex(body)) = body else {
                panic!("缺少 Maplex 标注体");
            };
            assert_eq!(body.version, 2);
            assert_eq!(body.v2_prefix, Some((-12, 4)));
            assert_eq!(body.properties.unwrap().body.unwrap().long_38, 23);
            assert!(refs.contains(91) && refs.contains(92));
            assert_eq!(reader.remaining(), 0);
        }
    }
}

#[test]
fn map_v0_to_v13_read_version_gated_tail_fields() {
    use mxd2qgz::layers::raster_dataset_name::{
        RASTER_BAND_NAME_CLASS_ID, RASTER_CATALOG_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::maps::map_object::read_map_suffix_10_1;
    use mxd2qgz::maps::standalone_table::{
        FGDB_TABLE_NAME_CLASS_ID, OBJECT_CLASS_NAME_CLASS_ID, TABLE_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::ams_dataset_name::AMS_DATASET_NAME_CLASS_ID;
    use mxd2qgz::objects::dataset_name_more::{
        DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, NETCDF_TABLE_NAME_CLASS_ID,
        REFERENCED_TABLE_NAME_CLASS_ID, SCHEMATIC_DATASET_NAME_CLASS_ID,
        SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::geo_transformation::NADCON_TRANSFORMATION_CLASS_ID;
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;
    use mxd2qgz::objects::string_array::STR_ARRAY_CLASS_ID;
    use mxd2qgz::objects::workspace_name::{PropertySetSlot10_1, WORKSPACE_NAME_CLASS_ID};
    use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
    use mxd2qgz::stream::reader::Reader;

    for version in 0..=13 {
        let mut suffix = vec![0; 8]; // 两组空图层列表。
        suffix.extend((-1_i32).to_le_bytes());
        suffix.extend([0; 16 + 4 + 32 + 8 + 16 + 4]); // 基本图形层、装饰物、范围、IEnvelope 和书签。
        suffix.extend(0x1234_u16.to_le_bytes());
        suffix.extend([0; 2 + 8 + 2]);
        let referenced = version == 10;
        let ams = version == 7;
        let fgdb_wrapper = (version % 3 == 1 && !referenced && !ams).then_some((1_u16, 0x4c2a_u16));
        let netcdf = version % 3 == 2;
        let table_offset = (version < 11).then(|| {
            suffix.extend(2_u32.to_le_bytes());
            let offset = suffix.len();
            let class_id = if ams {
                AMS_DATASET_NAME_CLASS_ID
            } else if referenced {
                REFERENCED_TABLE_NAME_CLASS_ID
            } else if netcdf {
                NETCDF_TABLE_NAME_CLASS_ID
            } else if fgdb_wrapper.is_some() {
                FGDB_TABLE_NAME_CLASS_ID
            } else {
                match version {
                    0 => OBJECT_CLASS_NAME_CLASS_ID,
                    3 => RASTER_BAND_NAME_CLASS_ID,
                    6 => RASTER_CATALOG_NAME_CLASS_ID,
                    9 => SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID,
                    _ => TABLE_NAME_CLASS_ID,
                }
            };
            suffix.extend(class_id);
            suffix.extend(90_u32.to_le_bytes());
            if ams {
                // 四个空 ATL BSTR、三个整数，其中首个标记要素类。
                suffix.extend([0; 16]);
                suffix.extend(5_i32.to_le_bytes());
                suffix.extend([0; 8]);
                suffix.extend(13_u16.to_le_bytes());
                suffix.extend(WORKSPACE_NAME_CLASS_ID);
                suffix.extend(1_u16.to_le_bytes());
                suffix.extend([2, 0, 0, 0, 0, 0].repeat(3));
                suffix.extend(PROPERTY_SET_CLASS_ID);
                suffix.extend(101_u32.to_le_bytes());
                suffix.extend(1_u16.to_le_bytes());
                suffix.extend(0_u32.to_le_bytes());
                suffix.extend([0; 5]);
                suffix.extend((-3_i16).to_le_bytes());
                suffix.extend(class_id);
                suffix.extend(90_u32.to_le_bytes());
                return offset;
            }
            if referenced {
                suffix.extend((-1_i16).to_le_bytes());
                suffix.extend(TABLE_NAME_CLASS_ID);
                suffix.extend(101_u32.to_le_bytes());
            }
            if let Some((version, flags)) = fgdb_wrapper {
                suffix.extend(version.to_le_bytes());
                suffix.extend(flags.to_le_bytes());
            }
            if version == 3 {
                suffix.extend(1_u32.to_le_bytes()); // RasterBandName 版本标记。
            } else {
                suffix.extend(1_u16.to_le_bytes());
            }
            for _ in 0..if version == 6 { 4 } else { 3 } {
                suffix.extend(2_u32.to_le_bytes());
                suffix.extend(0_u16.to_le_bytes());
            }
            if version == 6 {
                suffix.extend([0; 8]); // RasterCatalogName 委派给 FeatureClassName。
            }
            if matches!(version, 3 | 6) {
                suffix.extend(0_u16.to_le_bytes());
            }
            if version == 9 {
                suffix.extend(SCHEMATIC_DATASET_NAME_CLASS_ID);
                suffix.extend(101_u32.to_le_bytes());
                suffix.extend(1_u16.to_le_bytes());
                suffix.extend([2, 0, 0, 0, 0, 0].repeat(3));
            }
            suffix.extend([0; 16]); // WorkspaceName 空槽。
            if netcdf {
                suffix.extend(STR_ARRAY_CLASS_ID);
                suffix.extend(101_u32.to_le_bytes());
                suffix.extend(1_u16.to_le_bytes());
                suffix.extend(1_u32.to_le_bytes());
                suffix.extend(4_u32.to_le_bytes());
                suffix.extend([b'x', 0, 0, 0]);
                suffix.extend([0; 16]); // 第二个成员数组为空。
                suffix.extend(STR_ARRAY_CLASS_ID);
                suffix.extend(101_u32.to_le_bytes());
                suffix.extend([
                    0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0, 0xc0, 0x4f,
                    0x8e, 0xce, 0x27,
                ]); // LongArray CLSID。
                suffix.extend(102_u32.to_le_bytes());
                suffix.extend(1_u16.to_le_bytes());
                suffix.extend(1_u32.to_le_bytes());
                suffix.extend(9_i32.to_le_bytes());
            }
            suffix.extend(class_id);
            suffix.extend(90_u32.to_le_bytes());
            offset
        });
        if version >= 11 {
            suffix.extend(0_u32.to_le_bytes());
        }
        if version < 4 {
            for value in [1.0_f64, -2.0, 3.0, 4.0] {
                suffix.extend(value.to_le_bytes());
            }
        } else {
            suffix.extend([0; 16]); // IlluminationProps 空槽。
        }
        if version >= 2 {
            suffix.extend([0; 8]);
        }
        if version >= 3 {
            suffix.extend(0xabcd_u16.to_le_bytes());
        }
        if version >= 5 {
            suffix.extend(0x1234_u16.to_le_bytes());
        }
        if version >= 6 {
            suffix.extend([0; 32]); // IGeometry 和 IBorder。
        }
        if version >= 7 {
            suffix.extend([0; 2 + 16 + 2 + 16 + 2 + 2 + 8]);
        }
        if version >= 8 {
            suffix.extend([0; 8]);
        }
        if version >= 9 {
            suffix.extend([0; 2]);
        }
        if version >= 10 {
            suffix.extend([0; 8]);
        }
        if version >= 12 {
            suffix.extend(2_u32.to_le_bytes());
            suffix.extend(NADCON_TRANSFORMATION_CLASS_ID);
            suffix.extend(91_u32.to_le_bytes());
            suffix.extend(1_u16.to_le_bytes());
            suffix.extend(3_u32.to_le_bytes());
            suffix.extend(b"abc");
            suffix.extend(NADCON_TRANSFORMATION_CLASS_ID);
            suffix.extend(91_u32.to_le_bytes());
        }
        if version >= 13 {
            suffix.extend(4.5_f64.to_le_bytes());
        }
        let end = suffix.len();
        suffix.extend(0x1234_5678_u32.to_le_bytes());

        let (bytes, offset) = map_with_version(version, &suffix);
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(24).unwrap();
        let mut refs = RefTable10_1::new();
        let map = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(map.post_bookmark.unwrap().leading_short, 0x1234);
        assert_eq!(map.illumination.is_some(), version >= 4);
        assert_eq!(
            map.legacy_illumination.map(|(_, fields)| fields[1]),
            (version < 4).then_some((-2.0_f64).to_bits())
        );
        if let Some(fields) = map.legacy_illumination_fields {
            assert_eq!(fields.int_1b0, (version >= 2).then_some(0));
            assert_eq!(fields.short_1ac, (version >= 3).then_some(0xabcd));
        }
        if let Some(table_offset) = table_offset {
            let names = &map.legacy_table_names;
            match &names[0].1 {
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::AmsDatasetName(name))
                    if ams =>
                {
                    assert_eq!(name.scalars, [5, 0, 0]);
                    assert_eq!(name.trailing_short, -3);
                    assert!(matches!(
                        name.workspace_name.property_set,
                        PropertySetSlot10_1::PropertySet(_)
                    ));
                    assert!(refs.contains(101));
                }
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::ReferencedTable(name))
                    if referenced =>
                {
                    assert_eq!(name.version, -1);
                    assert_eq!(name.key, 90);
                    assert!(matches!(
                        name.source,
                        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Table(_))
                    ));
                    assert!(refs.contains(101));
                }
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::SchematicClass(name))
                    if version == 9 =>
                {
                    assert!(matches!(
                        &name.dataset,
                        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::SchematicDataset(dataset))
                            if dataset.key == 101
                    ));
                    assert!(refs.contains(101));
                }
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RasterTable(name)) => {
                    assert!(matches!(
                        (version, name.as_ref()),
                        (3, RasterDatasetNameEntryBody10_1::Band(_))
                            | (6, RasterDatasetNameEntryBody10_1::Catalog(_))
                    ));
                }
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Table(name)) if !netcdf => {
                    assert_eq!(name.fgdb_wrapper, fgdb_wrapper);
                }
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::NetCdfTable(name))
                    if netcdf =>
                {
                    let arrays = name.arrays.as_ref().unwrap();
                    assert_eq!(arrays[0].body, Some((1, vec!["x".to_owned()])));
                    assert!(matches!(
                        arrays[2].entry,
                        ObjectEntry10_1::BackReference { key: 101, .. }
                    ));
                    assert_eq!(
                        name.dimension_indices.as_ref().unwrap().body,
                        Some((1, vec![9]))
                    );
                    assert!(refs.contains(101) && refs.contains(102));
                }
                _ => panic!("缺少旧表名称"),
            }
            assert!(matches!(
                names[1].1,
                DatasetNameMoreSlot10_1::BackReference { key: 90, .. }
            ));
            let mut unknown = bytes.clone();
            let at = offset + table_offset;
            unknown[at..at + 16].fill(0x7d);
            let mut reader = Reader::new(&unknown);
            reader.read_bytes(24).unwrap();
            let mut refs = RefTable10_1::new();
            let stopped = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(stopped.legacy_table_names.is_empty());
            assert_eq!(reader.position(), at);
            assert!(!refs.contains(90));
            if fgdb_wrapper.is_some() || netcdf || referenced {
                let mut bad_version = bytes.clone();
                bad_version[at + 20..at + 22].copy_from_slice(&2_u16.to_le_bytes());
                let mut reader = Reader::new(&bad_version);
                reader.read_bytes(24).unwrap();
                let mut refs = RefTable10_1::new();
                let stopped = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(stopped.legacy_table_names.is_empty());
                assert_eq!(reader.position(), at);
                assert!(!refs.contains(90));
            }
            if ams {
                // 截在旧表名称的尾部 short 前，内嵌属性集此前已遇到。
                let tail = names[1].0 - 2;
                let mut reader = Reader::new(&bytes[..tail]);
                reader.read_bytes(24).unwrap();
                let mut refs = RefTable10_1::new();
                let stopped = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(stopped.legacy_table_names.is_empty());
                assert_eq!(reader.position(), at);
                assert!(!refs.contains(90) && !refs.contains(101));
            }
            if referenced {
                // 未知内层类与已知内层短读均回退到整个包装条目起点。
                let mut unknown_inner = bytes.clone();
                unknown_inner[at + 22..at + 38].fill(0x7d);
                for test in [&unknown_inner[..], &bytes[..at + 43]] {
                    let mut reader = Reader::new(test);
                    reader.read_bytes(24).unwrap();
                    let mut refs = RefTable10_1::new();
                    let stopped = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
                    assert!(stopped.legacy_table_names.is_empty());
                    assert_eq!(reader.position(), at);
                    assert!(!refs.contains(90) && !refs.contains(101));
                }
            }
        }
        if version >= 12 {
            let transformations = &map.post_border_tail.transformations;
            assert!(
                transformations
                    .iter()
                    .all(|item| item.direction_on_disk().is_none())
            );
            assert_eq!(transformations[0].body.as_ref().unwrap().text, b"abc");
            assert!(matches!(
                transformations[1].object.unwrap().entry,
                ObjectEntry10_1::BackReference { key: 91, .. }
            ));
        }
        assert_eq!(reader.position(), offset + end, "Map v{version}");
        assert_eq!(
            parse_maps_10_1(&bytes).unwrap().confirmed_tail,
            Some((offset + end, 0x1234_5678)),
            "Map v{version}"
        );
        assert!(
            parse_maps_10_1(&bytes[..offset + end - 1])
                .unwrap()
                .confirmed_tail
                .is_none()
        );
    }
}

#[test]
fn map_v14_to_v18_end_at_their_last_supported_field() {
    for version in [14, 15, 16, 18] {
        let mut suffix = vec![0; 8]; // 两组空图层列表。
        suffix.extend((-1_i32).to_le_bytes());
        suffix.extend([0; 16]); // 基本图形层。
        suffix.extend([0; 4 + 32 + 8 + 16 + 4]); // 装饰物、范围、两个 LONG、IEnvelope、书签。
        suffix.extend(0xffff_u16.to_le_bytes());
        suffix.extend([0; 2 + 8 + 2 + 4 + 16 + 12]); // 属性前导、照明和后续定长成员。
        suffix.extend([0; 32]); // IGeometry 和 IBorder。
        suffix.extend([0; 2 + 16 + 2 + 16 + 2 + 2 + 8]); // 边框后字段。
        suffix.extend([0; 4 + 4 + 2 + 8 + 4]); // 标量和空变换列表。
        suffix.extend(1200_f64.to_le_bytes());
        if version >= 15 {
            suffix.extend(0xabcd_u16.to_le_bytes());
        }
        if version >= 16 {
            suffix.extend([0; 16]); // IBackground。
        }
        let end = suffix.len();
        suffix.extend(0x12_34_56_78_u32.to_le_bytes());
        let (bytes, offset) = map_with_version(version, &suffix);
        assert_eq!(
            parse_maps_10_1(&bytes).unwrap().confirmed_tail,
            Some((offset + end, 0x12_34_56_78)),
            "Map v{version}"
        );

        let truncated = &bytes[..offset + end - 1];
        assert!(parse_maps_10_1(truncated).unwrap().confirmed_tail.is_none());
    }
}

#[test]
fn map_v28_skips_masking_for_basemap_and_its_back_reference() {
    use mxd2qgz::layers::basemap_layer::BASEMAP_LAYER_CLASS_ID;

    let mut basemap = BASEMAP_LAYER_CLASS_ID.to_vec();
    basemap.extend(91_u32.to_le_bytes());
    basemap.extend(6_u16.to_le_bytes());
    basemap.extend(2_u32.to_le_bytes());
    basemap.extend([0; 2 + 6 + 18 + 16 + 16 + 16 + 2 + 4 + 4]);

    let mut suffix = map_v28_masking_suffix(&[&basemap, &basemap[..20]]);
    let end = suffix.len();
    suffix.extend(0x1717_u32.to_le_bytes());

    let (bytes, offset) = map_with_version(28, &suffix);
    let report = parse_maps_10_1(&bytes).unwrap();
    assert_eq!(
        report.confirmed_tail,
        Some((offset + end, 0x1717)),
        "{:?}",
        report.diagnostics
    );
    assert!(report.map.unknown_fields.is_empty());

    let (bytes, _) = map_with_version(40, &suffix);
    let report = parse_maps_10_1(&bytes).unwrap();
    assert!(report.confirmed_tail.is_none());
    assert_eq!(
        report.map.frames[0].unknown_fields[0].offset as usize,
        offset + end
    );
}

fn map_v28_masking_suffix(layers: &[&[u8]]) -> Vec<u8> {
    let mut suffix = (layers.len() as i32).to_le_bytes().to_vec();
    for object in layers {
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend((object.len() as u32).to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(*object);
    }
    suffix.extend((-1_i32).to_le_bytes());
    suffix.extend([0; 16 + 4 + 32 + 8 + 16 + 4]); // 基本图形层至书签列表。
    suffix.extend(0xffff_u16.to_le_bytes());
    suffix.extend([0; 2 + 8 + 2 + 4 + 16 + 12 + 32]); // 属性、照明、几何及边框。
    suffix.extend([0; 2 + 16 + 2 + 16 + 2 + 2 + 8]);
    suffix.extend([0; 4 + 4 + 2 + 8 + 4]);
    suffix.extend([0; 8 + 2 + 16 + 16]); // 变换后背景和标注属性槽。
    suffix.extend([0; 2 + 8 + 2]); // 地图显示比例有关的标量。
    suffix.extend([0; 4 * 16 + 2 * 8 + 8 * 4]); // 四个包络及其后字段。
    suffix.extend([0; 2 + 4 + 2]); // Map 自身的空遮罩。
    suffix
}

#[test]
fn map_v28_skips_bounded_native_layers_without_masking_and_their_back_references() {
    for class_id in [
        [
            0x4b, 0xe9, 0xf8, 0xd4, 0xf5, 0x5c, 0x8f, 0x4f, 0x8b, 0x4d, 0x5b, 0x25, 0xae, 0x4c,
            0x0a, 0xf9,
        ],
        [
            0x40, 0xce, 0x34, 0x33, 0x1d, 0x1a, 0xd5, 0x11, 0xb7, 0xcf, 0x00, 0x01, 0x02, 0x65,
            0xad, 0xc5,
        ],
        [
            0xf6, 0xa8, 0x0d, 0x5b, 0x43, 0x5e, 0xae, 0x40, 0x98, 0x71, 0x56, 0xba, 0x33, 0x93,
            0x6f, 0x30,
        ],
        [
            0x62, 0xba, 0x77, 0xf6, 0xa7, 0x7c, 0x0a, 0x40, 0x9c, 0x59, 0x62, 0x93, 0x0a, 0x28,
            0x2c, 0xeb,
        ],
        [
            0xc0, 0x56, 0x8a, 0xe3, 0xbd, 0xd5, 0x99, 0x48, 0xb0, 0x89, 0xc8, 0xed, 0x4e, 0x38,
            0xb7, 0x7f,
        ],
        [
            0x71, 0x85, 0xbd, 0x5f, 0x60, 0xa5, 0x4f, 0x49, 0x9f, 0xaf, 0x3e, 0xdc, 0xed, 0x5a,
            0x78, 0x90,
        ],
    ] {
        let mut layer = class_id.to_vec();
        layer.extend(90_u32.to_le_bytes());
        layer.extend([0x12, 0x34]);
        let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
        let end = suffix.len();
        suffix.extend(0x1717_u32.to_le_bytes());
        let (bytes, offset) = map_with_version(28, &suffix);
        let report = parse_maps_10_1(&bytes).unwrap();
        assert_eq!(report.confirmed_tail, Some((offset + end, 0x1717)));

        // 未知类即使具备完整的有界体，也不能推定为不消费遮罩。
        layer[0] ^= 1;
        let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
        suffix.extend(0x1717_u32.to_le_bytes());
        let (bytes, _) = map_with_version(28, &suffix);
        assert!(parse_maps_10_1(&bytes).unwrap().confirmed_tail.is_none());
    }
}

#[test]
fn map_v28_reads_nested_group_masks_in_layer_order_and_rolls_back_on_failure() {
    use mxd2qgz::layers::basemap_layer::BASEMAP_LAYER_CLASS_ID;
    use mxd2qgz::layers::group_layer::GROUP_LAYER_CLASS_ID;
    use mxd2qgz::maps::map_object::{MapSuffixStop, read_map_suffix_10_1};
    use mxd2qgz::storage::object_refs::RefTable10_1;
    use mxd2qgz::stream::reader::Reader;

    fn group(key: u32, children: &[&[u8]]) -> Vec<u8> {
        let mut object = GROUP_LAYER_CLASS_ID.to_vec();
        object.extend(key.to_le_bytes());
        object.extend(1_u16.to_le_bytes());
        object.extend(2_u32.to_le_bytes());
        object.extend([0; 2 + 3 * 2 + 2 * 8]);
        object.extend((children.len() as u32).to_le_bytes());
        for child in children {
            object.extend(*child);
        }
        object
    }

    let mut basemap = BASEMAP_LAYER_CLASS_ID.to_vec();
    basemap.extend(92_u32.to_le_bytes());
    basemap.extend(6_u16.to_le_bytes());
    basemap.extend(2_u32.to_le_bytes());
    basemap.extend([0; 2 + 6 + 18 + 16 + 16 + 16 + 2 + 4 + 4]);
    let child = group(91, &[&basemap]);
    let parent = group(90, &[&child, &[0; 16], &child[..20]]);
    let mut suffix = map_v28_masking_suffix(&[&parent, &basemap[..20], &parent[..20]]);
    let masks_at = suffix.len();
    let keys = [90, 91, 92, 91, 92, 90, 91, 92, 91, 92];
    for index in 0..keys.len() {
        suffix.extend([0; 6]);
        suffix.extend((index as u16).to_le_bytes());
    }
    let end = suffix.len();
    suffix.extend(0x1717_u32.to_le_bytes());
    let (bytes, offset) = map_with_version(28, &suffix);
    let mut reader = Reader::new(&bytes[24..]);
    let mut refs = RefTable10_1::new();
    let parsed = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert_eq!(parsed.stop, MapSuffixStop::Complete);
    let records: Vec<_> = parsed
        .post_border_tail
        .layer_masking
        .iter()
        .map(|(key, record)| (*key, record.use_masking_raw()))
        .collect();
    assert_eq!(records, keys.into_iter().zip(0..).collect::<Vec<_>>());
    assert_eq!(reader.read_u32_le().unwrap(), 0x1717);
    let report = parse_maps_10_1(&bytes).unwrap();
    assert_eq!(report.confirmed_tail, Some((offset + end, 0x1717)));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|line| line.contains("图层键 92 的 UseMasking 原始位为 0x0002"))
    );

    let mut bad_class = bytes.clone();
    // 保留回引键，改成不匹配的类；无效回引不得掩盖下一个根字段。
    let child_ref_offset = offset + 4 + 10 + parent.len() - 20;
    bad_class[child_ref_offset..child_ref_offset + 16].copy_from_slice(&BASEMAP_LAYER_CLASS_ID);
    for (invalid, reason) in [
        (&bytes[..offset + end - 1], "遮罩记录读取失败"),
        (&bad_class[..], "与登记类不匹配"),
    ] {
        let mut reader = Reader::new(&invalid[24..]);
        let mut refs = RefTable10_1::new();
        let parsed = read_map_suffix_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.stop, MapSuffixStop::AfterBorderTail { stage: 38 });
        assert_eq!(reader.position(), offset + masks_at - 24);
        assert!(
            parsed
                .post_border_tail
                .layer_masking_stop
                .as_deref()
                .unwrap()
                .contains(reason)
        );
        assert!(parsed.post_border_tail.layer_masking.is_empty());
    }
}

#[test]
fn map_v28_skips_bounded_network_child_and_its_back_references() {
    use mxd2qgz::layers::group_layer::GROUP_LAYER_CLASS_ID;

    let mut layer = [
        0x4b, 0xe9, 0xf8, 0xd4, 0xf5, 0x5c, 0x8f, 0x4f, 0x8b, 0x4d, 0x5b, 0x25, 0xae, 0x4c, 0x0a,
        0xf9,
    ]
    .to_vec();
    layer.extend(91_u32.to_le_bytes());
    layer.extend([0x12, 0x34]);

    let mut group = GROUP_LAYER_CLASS_ID.to_vec();
    group.extend(90_u32.to_le_bytes());
    group.extend(11_u16.to_le_bytes());
    group.extend(2_u32.to_le_bytes());
    group.extend([0; 2 + 3 * 2 + 2 * 8]);
    group.extend(2_u32.to_le_bytes());
    for child in [&layer[..], &layer[..20]] {
        group.extend((child.len() as u32).to_le_bytes());
        group.extend(0_u32.to_le_bytes());
        group.extend(child);
    }
    group.extend([0; 2 + 16 + 8 + 4]);
    group.extend(2_u32.to_le_bytes());
    group.extend([0; 2 + 2 + 16 + 2 + 6]);

    let mut suffix = map_v28_masking_suffix(&[&group, &layer[..20]]);
    suffix.extend([0; 8]); // 只有组图层自身消费遮罩。
    let end = suffix.len();
    suffix.extend(0x1717_u32.to_le_bytes());
    let (bytes, offset) = map_with_version(28, &suffix);
    assert_eq!(
        parse_maps_10_1(&bytes).unwrap().confirmed_tail,
        Some((offset + end, 0x1717))
    );

    // 同时改写子图层和两处回引，未知类仍停在逐图层遮罩之前。
    let network_class = &layer[..16];
    let mut unknown = bytes;
    for pos in 0..=unknown.len() - network_class.len() {
        if &unknown[pos..pos + network_class.len()] == network_class {
            unknown[pos] ^= 1;
        }
    }
    assert!(parse_maps_10_1(&unknown).unwrap().confirmed_tail.is_none());
}

#[test]
fn map_v28_graphics_layers_3d_and_back_references_consume_no_masks() {
    use mxd2qgz::layers::graphics_3d::{GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID};
    use mxd2qgz::maps::layer_masking::{LayerMaskingSupport10_1, known_layer_masking_support_10_1};

    for (class_id, short_count, tail_bytes) in [
        (GRAPHICS_LAYER_3D_CLASS_ID, 4, 0),
        (GLOBE_GRAPHICS_LAYER_CLASS_ID, 2, 8),
    ] {
        assert_eq!(
            known_layer_masking_support_10_1(&class_id),
            LayerMaskingSupport10_1::Unsupported
        );

        let mut layer = class_id.to_vec();
        layer.extend(90_u32.to_le_bytes());
        layer.extend(1_u16.to_le_bytes());
        layer.extend(2_u32.to_le_bytes());
        // 空名称、短字段、空关联层和范围、空元素列表，以及 Globe 的空扩展和两个 short。
        layer.resize(
            layer.len() + 2 + 2 * short_count + 16 + 16 + 4 + tail_bytes,
            0,
        );
        let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
        let end = suffix.len();
        suffix.extend(0x1717_u32.to_le_bytes());
        let (bytes, offset) = map_with_version(28, &suffix);
        assert_eq!(
            parse_maps_10_1(&bytes).unwrap().confirmed_tail,
            Some((offset + end, 0x1717))
        );
    }
}

#[test]
fn map_v28_skips_masking_for_complete_composite_graphics_layer() {
    use mxd2qgz::layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID;

    let mut layer = COMPOSITE_GRAPHICS_LAYER_CLASS_ID.to_vec();
    layer.extend(90_u32.to_le_bytes());
    layer.extend(1_u16.to_le_bytes());
    layer.extend(0_u32.to_le_bytes()); // 空元素组。
    layer.extend(2_u32.to_le_bytes()); // 空名称。
    layer.extend([0; 2 + 6 + 16 + 12]);
    let nested_offset = layer.len();
    layer.extend([0; 16 + 4 + 4 + 16]); // 空嵌套槽、空子层集合和尾字段。
    let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
    let end = suffix.len();
    suffix.extend(0x1717_u32.to_le_bytes());
    let (bytes, offset) = map_with_version(28, &suffix);
    assert_eq!(
        parse_maps_10_1(&bytes).unwrap().confirmed_tail,
        Some((offset + end, 0x1717))
    );

    // 图层外层边界仍有效，内部未知新对象使遮罩遍历停在原处。
    layer[nested_offset..nested_offset + 16].fill(0x7d);
    let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
    suffix.extend(0x1717_u32.to_le_bytes());
    let (bytes, _) = map_with_version(28, &suffix);
    assert!(parse_maps_10_1(&bytes).unwrap().confirmed_tail.is_none());
}

#[test]
fn map_v28_reads_past_complete_image_server_layer_and_back_reference() {
    use mxd2qgz::layers::image_server_layer::IMAGE_SERVER_LAYER_CLASS_ID;

    let mut layer = IMAGE_SERVER_LAYER_CLASS_ID.to_vec();
    layer.extend(90_u32.to_le_bytes());
    layer.extend(3_u16.to_le_bytes()); // VT_I4 版本。
    layer.extend(8_i32.to_le_bytes());
    for (vt, size) in [
        (8_u16, 4),
        (8, 4),
        (11, 2),
        (11, 2),
        (11, 2),
        (11, 2),
        (5, 8),
        (5, 8),
        (5, 8),
        (8, 4),
        (5, 8),
        (5, 8),
        (11, 2),
        (11, 2),
        (2, 2),
        (11, 2),
        (2, 2),
        (11, 2),
        (2, 2),
        (2, 2),
        (13, 16),
        (13, 16),
        (13, 16),
        (8, 4),
        (11, 2),
        (8, 4),
    ] {
        layer.extend(vt.to_le_bytes());
        layer.resize(layer.len() + size, 0);
    }
    let name_slot = layer.len();
    layer.extend(13_u16.to_le_bytes()); // 空 IName。
    layer.extend([0; 16]);
    let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
    let end = suffix.len();
    suffix.extend(0x1717_u32.to_le_bytes());
    let (bytes, offset) = map_with_version(28, &suffix);
    assert_eq!(
        parse_maps_10_1(&bytes).unwrap().confirmed_tail,
        Some((offset + end, 0x1717))
    );

    // 未知 IName 使图层尾部解析停止，即使图层列表有外层边界。
    layer[name_slot + 2..name_slot + 18].fill(0x7d);
    layer.extend(91_u32.to_le_bytes());
    let mut suffix = map_v28_masking_suffix(&[&layer, &layer[..20]]);
    suffix.extend(0x1717_u32.to_le_bytes());
    let (bytes, _) = map_with_version(28, &suffix);
    assert!(parse_maps_10_1(&bytes).unwrap().confirmed_tail.is_none());
}
