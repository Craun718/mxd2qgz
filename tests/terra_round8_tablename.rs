use mxd2qgz::{
    maps::{
        map_object::{MAP_CLASS_ID, read_map_suffix_10_1},
        standalone_table::{
            StandaloneNameBody10_1, StandaloneTableStop10_1, TABLE_NAME_CLASS_ID,
            read_standalone_table_prefix_10_1,
        },
    },
    objects::{
        dataset_name_more::{
            DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, REFERENCED_TABLE_NAME_CLASS_ID,
        },
        workspace_name::WorkspaceNameEntry10_1,
    },
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::reader::Reader,
};
use std::io::Read;

fn fixture(workspace: &[u8]) -> Vec<u8> {
    let mut data = 0u16.to_le_bytes().to_vec();
    data.extend(TABLE_NAME_CLASS_ID);
    data.extend(332u32.to_le_bytes());
    data.extend(1u16.to_le_bytes());
    for _ in 0..3 {
        data.extend(2u32.to_le_bytes());
        data.extend([0, 0]);
    }
    data.extend(workspace);
    data.extend(0u32.to_le_bytes());
    data.extend(2u32.to_le_bytes());
    data.extend([0, 0]);
    data
}

#[test]
fn standalone_table_reads_object_class_alias_and_raster_table_names() {
    use mxd2qgz::layers::raster_dataset_name::{
        RASTER_BAND_NAME_CLASS_ID, RASTER_CATALOG_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::maps::standalone_table::OBJECT_CLASS_NAME_CLASS_ID;

    for id in [
        OBJECT_CLASS_NAME_CLASS_ID,
        RASTER_BAND_NAME_CLASS_ID,
        RASTER_CATALOG_NAME_CLASS_ID,
    ] {
        let mut data = 0_u16.to_le_bytes().to_vec();
        data.extend(id);
        data.extend(71_u32.to_le_bytes());
        if id == RASTER_BAND_NAME_CLASS_ID {
            data.extend(1_u32.to_le_bytes());
        } else {
            data.extend(1_u16.to_le_bytes());
        }
        for _ in 0..if id == RASTER_CATALOG_NAME_CLASS_ID {
            4
        } else {
            3
        } {
            data.extend([2, 0, 0, 0, 0, 0]);
        }
        if id == RASTER_CATALOG_NAME_CLASS_ID {
            data.extend([0; 8]);
        }
        if id != OBJECT_CLASS_NAME_CLASS_ID {
            data.extend([0; 2]);
        }
        data.extend([0; 16]); // 空工作空间。
        data.extend([0; 4]); // 命名对象计数。
        data.extend([2, 0, 0, 0, 0, 0]); // 表名。
        let end = data.len();
        data.push(0xa5);

        let mut r = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let body = read_standalone_table_prefix_10_1(&mut r, &mut refs, 4096, 64).unwrap();
        assert_eq!(body.stop, StandaloneTableStop10_1::Complete);
        assert_eq!(r.position(), end);
        assert!(refs.contains(71));
        match body.dataset_name_body.unwrap() {
            StandaloneNameBody10_1::TableName(name) => {
                assert_eq!(id, OBJECT_CLASS_NAME_CLASS_ID);
                assert_eq!(name.version, 1);
            }
            StandaloneNameBody10_1::OtherTabularName(name) => {
                let DatasetNameMoreBody10_1::RasterTable(name) = name.as_ref() else {
                    panic!("应读取栅格属性表名称");
                };
                assert!(matches!(
                    (id, name.as_ref()),
                    (
                        RASTER_BAND_NAME_CLASS_ID,
                        RasterDatasetNameEntryBody10_1::Band(_)
                    ) | (
                        RASTER_CATALOG_NAME_CLASS_ID,
                        RasterDatasetNameEntryBody10_1::Catalog(_)
                    )
                ));
            }
            other => panic!("{other:?}"),
        }
        for cut in 0..end {
            let mut r = Reader::new(&data[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 4096, 64).is_err());
            assert_eq!(r.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn standalone_table_reads_schematic_names_with_a_shared_dataset_and_workspace() {
    use mxd2qgz::objects::dataset_name::WorkspaceNameSlot10_1;
    use mxd2qgz::objects::dataset_name_more::{
        SCHEMATIC_DATASET_NAME_CLASS_ID, SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID,
        SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    const EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    bytes.extend(SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID);
    bytes.extend(71_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(EMPTY_STRING.repeat(3));
    let dataset_offset = bytes.len();
    bytes.extend(SCHEMATIC_DATASET_NAME_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(EMPTY_STRING.repeat(3));
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(73_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(EMPTY_STRING.repeat(3));
    bytes.extend([0; 16 + 1 + 4]); // 属性集空槽、工厂标志和尾部 int32。
    bytes.extend([0; 4]); // 命名对象。
    bytes.extend(EMPTY_STRING);
    bytes.extend([0; 4 + 16 + 4]); // 关系数、颜色与序列计数。
    bytes.extend(SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID);
    bytes.extend(74_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(EMPTY_STRING.repeat(3));
    bytes.extend(SCHEMATIC_DATASET_NAME_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 4096, 64).unwrap();
    assert_eq!(parsed.stop, StandaloneTableStop10_1::Complete);
    assert_eq!(reader.position(), end);
    assert!((71..=74).all(|key| refs.contains(key)));
    let Some(StandaloneNameBody10_1::OtherTabularName(first)) = parsed.dataset_name_body else {
        panic!("应读取 SchematicDiagramClassName");
    };
    let DatasetNameMoreBody10_1::SchematicClass(first) = first.as_ref() else {
        panic!("应读取 SchematicDiagramClassName");
    };
    let DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::SchematicDataset(dataset)) =
        &first.dataset
    else {
        panic!("应读取 SchematicDatasetName");
    };
    assert!(
        matches!(&dataset.workspace, WorkspaceNameSlot10_1::WorkspaceName(name) if name.key == 73)
    );
    let Some(StandaloneNameBody10_1::OtherTabularName(second)) = parsed.secondary_name_body else {
        panic!("应读取 SchematicElementClassName");
    };
    let DatasetNameMoreBody10_1::SchematicClass(second) = second.as_ref() else {
        panic!("应读取 SchematicElementClassName");
    };
    assert!(matches!(
        second.dataset,
        DatasetNameMoreSlot10_1::BackReference { key: 72, .. }
    ));

    for cut in 0..end {
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes[..cut]);
        assert!(read_standalone_table_prefix_10_1(&mut reader, &mut refs, 4096, 64).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    bytes[dataset_offset..dataset_offset + 16].fill(0x7a);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 4096, 64).unwrap();
    assert_eq!(parsed.stop, StandaloneTableStop10_1::NameBody);
    assert_eq!(reader.position(), dataset_offset);
    assert!(refs.contains(71));
    assert!(!refs.contains(72) && !refs.contains(73) && !refs.contains(74));
}

#[test]
fn standalone_table_reads_attributed_relationship_class_name() {
    use mxd2qgz::objects::dataset_name::WorkspaceNameSlot10_1;
    use mxd2qgz::objects::dataset_name_more::{
        RELATIONSHIP_CLASS_NAME_CLASS_ID, RelationshipClassTail10_1,
    };

    const EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    bytes.extend(RELATIONSHIP_CLASS_NAME_CLASS_ID);
    bytes.extend(75_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(EMPTY_STRING.repeat(3));
    bytes.extend([0; 10]); // 基数、通知模式和复合标志。
    bytes.extend(1_u16.to_le_bytes()); // 有属性标志。
    bytes.extend([0; 2 + 16]); // 工作空间选择子及空槽。
    bytes.extend([0; 4]); // 命名对象数。
    bytes.extend(EMPTY_STRING);
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let parsed = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 4096, 64).unwrap();
    assert_eq!(parsed.stop, StandaloneTableStop10_1::Complete);
    assert_eq!(reader.position(), end);
    assert!(refs.contains(75));
    let Some(StandaloneNameBody10_1::OtherTabularName(name)) = parsed.dataset_name_body else {
        panic!("应读取 RelationshipClassName");
    };
    let DatasetNameMoreBody10_1::RelationshipClass(name) = name.as_ref() else {
        panic!("应读取 RelationshipClassName");
    };
    assert_eq!(name.short_b, 1);
    assert!(matches!(
        name.tail,
        RelationshipClassTail10_1::Workspace(WorkspaceNameSlot10_1::Null)
    ));
}

#[test]
fn tablename_null_workspace_and_truncation() {
    let data = fixture(&[0; 16]);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let body = read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 100).unwrap();
    let Some(StandaloneNameBody10_1::TableName(name)) = body.dataset_name_body else {
        panic!()
    };
    assert_eq!(name.version, 1);
    assert!(matches!(name.workspace, WorkspaceNameEntry10_1::Null));
    assert_eq!(body.stop, StandaloneTableStop10_1::Complete);
    assert_eq!(r.position(), data.len());
    for end in 0..data.len() {
        let mut r = Reader::new(&data[..end]);
        let mut refs = RefTable10_1::new();
        refs.register(9);
        assert!(
            read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 100).is_err(),
            "{end}"
        );
        assert_eq!(r.position(), 0);
        assert_eq!(refs.len(), 1);
    }
}

#[test]
fn standalone_referenced_table_name_reuses_source_key() {
    for version in [0_i16, 1] {
        let mut data = 0_u16.to_le_bytes().to_vec();
        data.extend(REFERENCED_TABLE_NAME_CLASS_ID);
        data.extend(333_u32.to_le_bytes());
        data.extend(version.to_le_bytes());
        data.extend(TABLE_NAME_CLASS_ID);
        data.extend(332_u32.to_le_bytes());
        data.extend(0_u32.to_le_bytes());
        data.extend(2_u32.to_le_bytes());
        data.extend(0_u16.to_le_bytes());

        let mut r = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        refs.register(332);
        let body = read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 100).unwrap();
        let Some(StandaloneNameBody10_1::OtherTabularName(name)) = body.dataset_name_body else {
            panic!("缺少 ReferencedTableName");
        };
        let DatasetNameMoreBody10_1::ReferencedTable(name) = *name else {
            panic!("表名类不符");
        };
        assert_eq!(name.version, version);
        assert_eq!(name.key, 333);
        assert_eq!(
            name.source,
            DatasetNameMoreSlot10_1::BackReference {
                class_id: TABLE_NAME_CLASS_ID,
                key: 332,
            }
        );
        assert_eq!(body.stop, StandaloneTableStop10_1::Complete);
        assert_eq!(r.position(), data.len());
        assert!(refs.contains(333));
    }
}

#[test]
fn tablename_unknown_workspace_stops_before_header() {
    let mut workspace = vec![0x55; 16];
    workspace.extend(333u32.to_le_bytes());
    let data = fixture(&workspace);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 100).unwrap();
    assert_eq!(p.stop, StandaloneTableStop10_1::NameBody);
    assert!(
        matches!(p.dataset_name_body, Some(StandaloneNameBody10_1::TableName(name)) if matches!(name.workspace, WorkspaceNameEntry10_1::Unsupported { key: 333, .. }))
    );
    assert_eq!(r.position(), 42);
    assert!(refs.contains(332));
    assert!(!refs.contains(333));
}

#[test]
fn tablename_version_limits_and_workspace_backreference() {
    for version in [2u16, 0xffff] {
        let mut data = fixture(&[0; 16]);
        data[22..24].copy_from_slice(&version.to_le_bytes());
        let mut r = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 100).is_err());
        assert_eq!(r.position(), 0);
        assert!(refs.len() == 0);
    }
    let mut workspace = vec![0x37; 16];
    workspace.extend(7u32.to_le_bytes());
    let data = fixture(&workspace);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    refs.register(7);
    let p = read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 100).unwrap();
    assert!(
        matches!(p.dataset_name_body, Some(StandaloneNameBody10_1::TableName(body)) if matches!(body.workspace, WorkspaceNameEntry10_1::BackReference { key: 7, .. }))
    );
    assert_eq!(p.stop, StandaloneTableStop10_1::Complete);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    assert!(read_standalone_table_prefix_10_1(&mut r, &mut refs, 100, 1).is_err());
    assert_eq!(r.position(), 0);
    assert!(refs.len() == 0);
}

#[test]
#[ignore = "需要 10.1 SDK Java 安装介质提取文件"]
fn linearref_tablename_from_map_host() {
    let path = "temp/terra-round8-old/sdk-java/LinearRef.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1";
    let mut doc = cfb::open(path).unwrap();
    let mut data = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    assert!(r.read_u32_le().unwrap() > 0);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut r, 16_000_000, &mut refs).unwrap();
    let (slot, prefix) = &map.standalone_tables[0];
    assert_eq!(slot.offset, 0x6a50);
    let prefix = prefix.as_ref().unwrap();
    assert_eq!(prefix.version, 9);
    assert!(matches!(
        prefix.dataset_name,
        Some(ObjectEntry10_1::NewObject {
            class_id: TABLE_NAME_CLASS_ID,
            key: 332
        })
    ));
    let Some(StandaloneNameBody10_1::TableName(name)) = &prefix.dataset_name_body else {
        panic!("未读取 TableName")
    };
    assert_eq!(name.version, 1);
    let WorkspaceNameEntry10_1::NewObject(workspace) = &name.workspace else {
        panic!("未读取 WorkspaceName")
    };
    assert!(workspace.stop.is_none());
    assert_eq!(workspace.key, 333);
    assert_eq!(prefix.named_objects.len(), 24);
    assert_eq!(prefix.named_object_bodies[0].as_ref().unwrap().key, 334);
    assert_eq!(prefix.stop, StandaloneTableStop10_1::Complete);
    assert_eq!(map.standalone_tables.len(), 2);
    let fields = map.post_border_tail.post_masking.as_ref().unwrap();
    assert_eq!(fields.is_framed_bits.0, 0x8407);
    assert_eq!(fields.string_398, None);
    assert_eq!(r.position(), 0x8409);
    assert_eq!(map.stop, mxd2qgz::maps::map_object::MapSuffixStop::Complete);
    let report = mxd2qgz::maps::mxd_maps::parse_maps_10_1(&data).unwrap();
    assert_eq!(report.confirmed_tail, Some((0x8409, 1)));
    assert!(
        report.map.frames[0]
            .unknown_fields
            .iter()
            .all(|field| field.offset < 0x8409)
    );
    assert!(report.map.unknown_fields.is_empty());
}

#[test]
#[ignore = "需要 10.1 SDK Java 安装介质提取文件"]
fn root_loop_reaches_next_slot_only_after_a_verified_complete_map() {
    use mxd2qgz::maps::mxd_maps::parse_maps_10_1;
    let path = "temp/terra-round8-old/sdk-java/LinearRef.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1";
    let mut doc = cfb::open(path).unwrap();
    let mut data = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let end = 0x8409;

    // 复用真实首框及其实际类/键作为第二槽回引；根尾值保留在循环后。
    let mut two = data[..end].to_vec();
    two[..4].copy_from_slice(&2_u32.to_le_bytes());
    two.extend_from_slice(&data[4..24]);
    two.extend_from_slice(&data[end..]);
    let report = parse_maps_10_1(&two).unwrap();
    assert_eq!(report.map.frames.len(), 2);
    assert!(report.map.frames[0].name == report.map.frames[1].name);
    assert_eq!(
        report.map.frames[0].children.len(),
        report.map.frames[1].children.len()
    );
    assert_eq!(report.confirmed_tail, Some((end + 20, 1)));

    // 未知第二个类没有长度，不能把后续字节错认成根尾值。
    two[end] ^= 0x80;
    let report = parse_maps_10_1(&two).unwrap();
    assert_eq!(report.map.frames.len(), 1);
    assert_eq!(report.confirmed_tail, None);
    assert_eq!(report.map.unknown_fields[0].offset as usize, end);
    assert_eq!(report.map.unknown_fields[0].bytes, two[end..]);

    // 根尾值只剩 3 字节，仍应保留已完成的框，不报告完整根尾。
    let report = parse_maps_10_1(&data[..data.len() - 1]).unwrap();
    assert_eq!(report.map.frames.len(), 1);
    assert_eq!(report.confirmed_tail, None);
    assert_eq!(
        report.map.unknown_fields[0].bytes,
        data[end..data.len() - 1]
    );
}
