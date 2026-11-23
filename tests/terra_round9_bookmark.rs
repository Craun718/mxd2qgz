use mxd2qgz::{
    layers::{
        feature_layer::FGDB_FEATURE_CLASS_NAME_CLASS_ID,
        raster_dataset_name::{RASTER_BAND_NAME_CLASS_ID, RASTER_CATALOG_NAME_CLASS_ID},
    },
    maps::{
        map_surround::{BookmarkNameEntry10_1, read_feature_bookmark_10_1},
        standalone_table::{
            FGDB_TABLE_NAME_CLASS_ID, OBJECT_CLASS_NAME_CLASS_ID, TABLE_NAME_CLASS_ID,
        },
    },
    objects::{
        dataset_name::WorkspaceNameSlot10_1,
        dataset_name_more::{
            DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, FEATURE_QUERY_NAME_CLASS_ID,
            MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID, QUERY_TABLE_NAME_CLASS_ID,
            REL_QUERY_TABLE_NAME_CLASS_ID, RELATIONSHIP_CLASS_NAME_CLASS_ID,
            RelationshipClassTail10_1, SCHEMATIC_DATASET_NAME_CLASS_ID,
            SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID, SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID,
            TABLE_QUERY_NAME_CLASS_ID, UnknownSlot10_1,
        },
        workspace_name::WorkspaceNameEntry10_1,
    },
    storage::object_refs::RefTable10_1,
    stream::reader::Reader,
};

#[test]
fn bookmark_reads_filegdb_catalog_and_query_names_with_transactional_boundaries() {
    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    for class in [
        FGDB_FEATURE_CLASS_NAME_CLASS_ID,
        RASTER_CATALOG_NAME_CLASS_ID,
        FEATURE_QUERY_NAME_CLASS_ID,
        TABLE_QUERY_NAME_CLASS_ID,
    ] {
        let mut data = 1_u16.to_le_bytes().to_vec();
        data.extend(42_i32.to_le_bytes());
        data.extend(class);
        data.extend(17_u32.to_le_bytes());
        let workspace_offset;
        if matches!(
            class,
            FGDB_FEATURE_CLASS_NAME_CLASS_ID | RASTER_CATALOG_NAME_CLASS_ID
        ) {
            if class == FGDB_FEATURE_CLASS_NAME_CLASS_ID {
                data.extend(1_u16.to_le_bytes());
                data.extend(0x1234_u16.to_le_bytes());
            }
            data.extend(2_u16.to_le_bytes()); // 内嵌 FeatureClassName，无第二个对象头。
            for _ in 0..4 {
                data.extend(EMPTY);
            }
            data.extend([0; 8 + 2]);
            workspace_offset = data.len();
            data.extend([0; 16 + 2]);
        } else {
            data.extend(2_u16.to_le_bytes());
            for _ in 0..3 {
                data.extend(EMPTY);
            }
            workspace_offset = data.len();
            data.extend([0; 16]);
            for _ in 0..4 {
                data.extend(EMPTY);
            }
            data.extend(0x1234_u16.to_le_bytes());
            if class == FEATURE_QUERY_NAME_CLASS_ID {
                data.extend(EMPTY);
                data.extend(11_u32.to_le_bytes());
                data.extend(22_u32.to_le_bytes());
            }
        }
        data.extend([4, 0, 0, 0, b'B', 0, 0, 0]);
        let end = data.len();
        data.push(0xa5);
        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        match bookmark.name_object {
            BookmarkNameEntry10_1::FeatureClassName(body) => {
                assert_eq!(class, FGDB_FEATURE_CLASS_NAME_CLASS_ID);
                assert!(body.fgdb_wrapper.is_some());
            }
            BookmarkNameEntry10_1::RasterCatalogName(body) => {
                assert_eq!(class, RASTER_CATALOG_NAME_CLASS_ID);
                assert_eq!(body.key, 17);
                assert!(body.stop.is_none());
            }
            BookmarkNameEntry10_1::FeatureQueryName(body) => {
                assert_eq!(class, FEATURE_QUERY_NAME_CLASS_ID);
                assert_eq!(body.feature_tail, Some((String::new(), 11, 22)));
            }
            BookmarkNameEntry10_1::TableQueryName(body) => {
                assert_eq!(class, TABLE_QUERY_NAME_CLASS_ID);
                assert_eq!(body.version_two_tail, Some((String::new(), 0x1234)));
            }
            other => panic!("非预期的书签名称：{other:?}"),
        }
        assert_eq!(bookmark.name, "B");
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);
        assert!(refs.contains(17));

        let mut unknown = data.clone();
        let mut entry = vec![0x5a; 16];
        entry.extend(18_u32.to_le_bytes());
        unknown.splice(workspace_offset..workspace_offset + 16, entry);
        for input in [&data[..end - 1], &unknown[..]] {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            refs.register(5);
            let before = refs.clone();
            assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn bookmark_reads_relationship_query_and_its_relationship_reference() {
    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(REL_QUERY_TABLE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend(EMPTY.repeat(2));
    let source_offset = bytes.len();
    bytes.extend(RELATIONSHIP_CLASS_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend([0; 32]);
    bytes.extend(EMPTY);
    for flag in [2_u16, 3, 4] {
        bytes.extend(flag.to_le_bytes());
    }
    bytes.extend([4, 0, 0, 0, b'B', 0, 0, 0]);
    let end = bytes.len();
    bytes.push(0x53);

    let mut existing = RefTable10_1::new();
    existing.register(18);
    let mut refs = existing.clone();
    let mut reader = Reader::new(&bytes);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RelQueryTableName(query) = bookmark.name_object else {
        panic!("应读取关系查询名称");
    };
    assert_eq!(query.key, 17);
    assert!(matches!(
        query.source_slot,
        DatasetNameMoreSlot10_1::BackReference {
            class_id: RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key: 18
        }
    ));
    assert_eq!((query.short_c, query.short_d), (Some(3), Some(4)));
    assert_eq!(bookmark.name, "B");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    let mut unknown = bytes.clone();
    unknown[source_offset] ^= 0xff;
    unknown[source_offset + 16..source_offset + 20].copy_from_slice(&19_u32.to_le_bytes());
    for input in [&bytes[..end - 1], &unknown] {
        let mut refs = existing.clone();
        let mut reader = Reader::new(input);
        assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, existing);
    }
}

const BOOKMARK_EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];

fn bookmark_string_10_1(text: &str) -> Vec<u8> {
    let mut bytes = ((text.encode_utf16().count() * 2 + 2) as u32)
        .to_le_bytes()
        .to_vec();
    for unit in text.encode_utf16() {
        bytes.extend(unit.to_le_bytes());
    }
    bytes.extend(0_u16.to_le_bytes());
    bytes
}

fn bookmark_entry_10_1(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

fn bookmark_header_10_1(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(class_id);
    bytes.extend(key.to_le_bytes());
    bytes
}

fn relationship_class_body_10_1(selector: u16, tail: &[u8]) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    for text in ["源类", "目标类", "关系类"] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend((-9_i32).to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(selector.to_le_bytes());
    bytes.extend(tail);
    bytes
}

fn feature_class_endpoint_body_10_1() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(BOOKMARK_EMPTY_STRING.repeat(4));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend([0; 16]);
    bytes
}

fn memory_relationship_prefix_10_1() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    for text in ["内存关系", "源类", "关系类"] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend((-4_i32).to_le_bytes());
    bytes.extend(5_u16.to_le_bytes());
    bytes.extend(6_u16.to_le_bytes());
    bytes
}

fn memory_relationship_class_body_10_1(slot_a: &[u8], slot_b: &[u8]) -> Vec<u8> {
    let mut bytes = memory_relationship_prefix_10_1();
    bytes.extend(slot_a);
    bytes.extend(slot_b);
    for text in ["正向", "反向", "OID", "OriginID"] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes
}

fn assert_bookmark_read_error(
    input: &[u8],
    existing: &RefTable10_1,
    expected_kind: Option<std::io::ErrorKind>,
) {
    let mut reader = Reader::new(input);
    let mut refs = existing.clone();
    let error = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap_err();
    if let Some(expected) = expected_kind {
        assert_eq!(error.kind(), expected);
    }
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, *existing);
}

#[test]
fn bookmark_reads_relationship_class_name_branches_transactionally() {
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    let null = [0; 16].to_vec();
    let back_reference = bookmark_entry_10_1(&RELATIONSHIP_CLASS_NAME_CLASS_ID, 7);
    let new_workspace = {
        let mut bytes = bookmark_entry_10_1(&WORKSPACE_NAME_CLASS_ID, 18);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(BOOKMARK_EMPTY_STRING.repeat(3));
        bytes.extend([0; 16]);
        bytes.push(1);
        bytes.extend([0xa5; 16]);
        bytes.extend(0_i32.to_le_bytes());
        bytes
    };

    for (selector, tail, new_workspace_key) in [
        (0_u16, null.as_slice(), None),
        (0, back_reference.as_slice(), None),
        (0, new_workspace.as_slice(), Some(18)),
        (1, null.as_slice(), None),
        (1, back_reference.as_slice(), None),
    ] {
        let mut data = bookmark_header_10_1(&RELATIONSHIP_CLASS_NAME_CLASS_ID, 17);
        let tail_offset = data.len() + relationship_class_body_10_1(selector, &[]).len();
        data.extend(relationship_class_body_10_1(selector, tail));
        data.extend(bookmark_string_10_1("R"));
        let end = data.len();
        data.push(0xa5);

        let mut existing = RefTable10_1::new();
        existing.register(7);
        let before = existing.clone();
        let mut refs = existing.clone();
        let mut reader = Reader::new(&data);
        let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RelationshipClassName(body) = bookmark.name_object else {
            panic!("RelationshipClassName 不得折叠为其他名称类");
        };
        assert_eq!(body.key, 17);
        assert_eq!(body.version, 1);
        assert_eq!(
            [&body.string_a, &body.string_b, &body.string_c],
            [
                &"源类".to_string(),
                &"目标类".to_string(),
                &"关系类".to_string()
            ]
        );
        assert_eq!((body.int_a, body.int_b), (7, -9));
        assert_eq!((body.short_a, body.short_b), (1, 2));
        assert_eq!(body.selector, selector);
        assert_eq!(body.stop, None);
        match &body.tail {
            RelationshipClassTail10_1::Workspace(workspace) => {
                assert_eq!(selector, 0);
                match workspace {
                    WorkspaceNameSlot10_1::Null => assert!(tail.iter().all(|byte| *byte == 0)),
                    WorkspaceNameSlot10_1::BackReference { key: 7, .. } => {
                        assert_eq!(tail, &back_reference)
                    }
                    WorkspaceNameSlot10_1::WorkspaceName(workspace) => {
                        assert_eq!(new_workspace_key, Some(workspace.key));
                    }
                    other => panic!("未预期的 WorkspaceName 槽：{other:?}"),
                }
            }
            RelationshipClassTail10_1::Dataset(dataset) => {
                assert_eq!(selector, 1);
                match dataset {
                    DatasetNameMoreSlot10_1::Null => assert!(tail.iter().all(|byte| *byte == 0)),
                    DatasetNameMoreSlot10_1::BackReference { key: 7, .. } => {
                        assert_eq!(tail, &back_reference)
                    }
                    other => panic!("未预期的 IDatasetName 槽：{other:?}"),
                }
            }
        }
        assert_eq!(bookmark.name, "R");
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);
        assert!(refs.contains(17));
        assert_eq!(refs.contains(18), new_workspace_key.is_some());

        for cut in 0..end {
            assert_bookmark_read_error(&data[..cut], &before, None);
        }

        let mut unknown = data.clone();
        unknown[tail_offset..tail_offset + 16].fill(0x66);
        unknown[tail_offset + 16..tail_offset + 20].copy_from_slice(&19_u32.to_le_bytes());
        assert_bookmark_read_error(&unknown, &before, Some(std::io::ErrorKind::Unsupported));
    }
}

#[test]
fn bookmark_reads_memory_relationship_class_name_endpoints_transactionally() {
    use mxd2qgz::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID;

    let new_feature_class = {
        let mut bytes = bookmark_entry_10_1(&FEATURE_CLASS_NAME_CLASS_ID, 18);
        bytes.extend(feature_class_endpoint_body_10_1());
        bytes
    };
    let back_reference = bookmark_entry_10_1(&TABLE_NAME_CLASS_ID, 7);
    let mut data = bookmark_header_10_1(&MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID, 17);
    let slot_a_offset = data.len() + memory_relationship_prefix_10_1().len();
    let slot_b_offset = slot_a_offset + new_feature_class.len();
    data.extend(memory_relationship_class_body_10_1(
        &new_feature_class,
        &back_reference,
    ));
    data.extend(bookmark_string_10_1("M"));
    let end = data.len();
    data.push(0xc3);

    let mut existing = RefTable10_1::new();
    existing.register(7);
    let before = existing.clone();
    let mut refs = existing.clone();
    let mut reader = Reader::new(&data);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
    let BookmarkNameEntry10_1::MemoryRelationshipClassName(body) = bookmark.name_object else {
        panic!("MemoryRelationshipClassName 不得折叠为其他名称类");
    };
    assert_eq!(body.key, 17);
    assert_eq!(body.version, 1);
    assert_eq!(
        body.strings_a,
        [
            "内存关系".to_string(),
            "源类".to_string(),
            "关系类".to_string()
        ]
    );
    assert_eq!((body.int_a, body.int_b), (3, -4));
    assert_eq!((body.short_a, body.short_b), (5, 6));
    assert!(matches!(
        body.slot_a,
        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::FeatureClassName(ref feature))
            if feature.key == 18
    ));
    assert!(matches!(
        body.slot_b,
        DatasetNameMoreSlot10_1::BackReference {
            class_id: TABLE_NAME_CLASS_ID,
            key: 7
        }
    ));
    assert_eq!(
        body.strings_b,
        [
            "正向".to_string(),
            "反向".to_string(),
            "OID".to_string(),
            "OriginID".to_string()
        ]
    );
    assert_eq!(body.stop, None);
    assert_eq!(bookmark.name, "M");
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0xc3]);
    assert!(refs.contains(17) && refs.contains(18) && refs.contains(7));

    for cut in 0..end {
        assert_bookmark_read_error(&data[..cut], &before, None);
    }

    for offset in [slot_a_offset, slot_b_offset] {
        let mut unknown = data.clone();
        unknown[offset..offset + 16].fill(0x67);
        unknown[offset + 16..offset + 20].copy_from_slice(&19_u32.to_le_bytes());
        assert_bookmark_read_error(&unknown, &before, Some(std::io::ErrorKind::Unsupported));
    }
}

fn fixture(workspace: &[u8]) -> Vec<u8> {
    fixture_for(TABLE_NAME_CLASS_ID, workspace)
}

fn fixture_for(class_id: [u8; 16], workspace: &[u8]) -> Vec<u8> {
    let mut b = 1u16.to_le_bytes().to_vec();
    b.extend(42i32.to_le_bytes());
    b.extend(class_id);
    b.extend(17u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    for _ in 0..3 {
        b.extend(2u32.to_le_bytes());
        b.extend([0, 0]);
    }
    b.extend(workspace);
    b.extend(2u32.to_le_bytes());
    b.extend([0, 0]);
    b
}

#[test]
fn object_class_name_keeps_distinct_identity_and_transaction_boundary() {
    let mut new_workspace = {
        use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;
        let mut bytes = WORKSPACE_NAME_CLASS_ID.to_vec();
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
        bytes.extend([0; 16]);
        bytes.push(0);
        bytes.extend(0_u32.to_le_bytes());
        bytes
    };
    let null = vec![0; 16];
    let back_reference = {
        let mut bytes = vec![0x31; 16];
        bytes.extend(7_u32.to_le_bytes());
        bytes
    };
    let fixtures = [
        (null, false),
        (back_reference, false),
        (std::mem::take(&mut new_workspace), true),
    ];

    for (workspace, is_new) in fixtures {
        let data = fixture_for(OBJECT_CLASS_NAME_CLASS_ID, &workspace);
        let mut refs = RefTable10_1::new();
        refs.register(7);
        let before = refs.clone();
        let mut reader = Reader::new(&data);
        let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        let BookmarkNameEntry10_1::ObjectClassName(body) = bookmark.name_object else {
            panic!("ObjectClassName 不得折叠为 TableName");
        };
        assert_eq!(body.version, 1);
        assert_eq!(body.strings, [String::new(), String::new(), String::new()]);
        assert_eq!(body.fgdb_wrapper, None);
        assert!(match body.workspace {
            WorkspaceNameEntry10_1::Null => !is_new,
            WorkspaceNameEntry10_1::BackReference { key: 7, .. } => !is_new,
            WorkspaceNameEntry10_1::BackReference { .. } => false,
            WorkspaceNameEntry10_1::NewObject(workspace) => is_new && workspace.key == 18,
            WorkspaceNameEntry10_1::Unsupported { .. } => false,
        });
        assert_eq!(bookmark.name, "");
        assert_eq!(reader.position(), data.len());
        assert!(refs.contains(17));
        assert_eq!(refs.contains(18), is_new);

        for end in 0..data.len() {
            let mut reader = Reader::new(&data[..end]);
            let mut refs = before.clone();
            assert!(
                read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err(),
                "ObjectClassName 前缀 {end} 应失败"
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }

    let mut unknown_workspace = vec![0x45; 16];
    unknown_workspace.extend(19_u32.to_le_bytes());
    let data = fixture_for(OBJECT_CLASS_NAME_CLASS_ID, &unknown_workspace);
    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    assert_eq!(
        read_feature_bookmark_10_1(&mut reader, 4096, &mut refs)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::Unsupported
    );
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, RefTable10_1::new());
}

#[test]
fn bookmark_reads_fgdb_table_name_wrapper() {
    let mut data = bookmark_header_10_1(&FGDB_TABLE_NAME_CLASS_ID, 17);
    let wrapper_offset = data.len();
    data.extend(1_u16.to_le_bytes());
    data.extend(0x1234_u16.to_le_bytes());
    data.extend(1_u16.to_le_bytes());
    data.extend(BOOKMARK_EMPTY_STRING.repeat(3));
    data.extend([0; 16]);
    data.extend(bookmark_string_10_1("X"));
    let end = data.len();
    data.push(0x6d);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::FgdbTableName(body) = bookmark.name_object else {
        panic!("FgdbTableName 不得折叠为普通 TableName");
    };
    assert_eq!(body.version, 1);
    assert_eq!(body.fgdb_wrapper, Some((1, 0x1234)));
    assert_eq!(body.strings, [String::new(), String::new(), String::new()]);
    assert!(matches!(body.workspace, WorkspaceNameEntry10_1::Null));
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0x6d]);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&data[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(
            read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err(),
            "FgdbTableName 前缀 {cut} 应失败"
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    let mut bad_version = data.clone();
    bad_version[wrapper_offset..wrapper_offset + 2].copy_from_slice(&2_u16.to_le_bytes());
    let mut reader = Reader::new(&bad_version);
    let mut refs = RefTable10_1::new();
    assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn bookmark_reads_raster_band_and_schematic_names() {
    let mut raster = bookmark_header_10_1(&RASTER_BAND_NAME_CLASS_ID, 17);
    raster.extend(1_u32.to_le_bytes());
    for text in ["image\\Band_1", "", ""] {
        raster.extend(bookmark_string_10_1(text));
    }
    raster.extend(0_u16.to_le_bytes());
    raster.extend([0; 16]);
    raster.extend(bookmark_string_10_1("R"));
    let raster_end = raster.len();

    let mut reader = Reader::new(&raster);
    let mut refs = RefTable10_1::new();
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterBandName(body) = bookmark.name_object else {
        panic!("RasterBandName 未进入书签分派");
    };
    assert!(matches!(
        *body,
        mxd2qgz::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1::Band(_)
    ));
    assert_eq!(bookmark.name, "R");
    assert_eq!(reader.position(), raster_end);
    assert!(refs.contains(17));

    for class_id in [
        SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID,
        SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID,
    ] {
        let mut data = bookmark_header_10_1(&class_id, 17);
        data.extend(1_i16.to_le_bytes());
        for text in ["图表", "元素", "名称"] {
            data.extend(bookmark_string_10_1(text));
        }
        data.extend(SCHEMATIC_DATASET_NAME_CLASS_ID);
        data.extend(18_u32.to_le_bytes());
        data.extend(1_i16.to_le_bytes());
        for text in ["数据集", "", "类别"] {
            data.extend(bookmark_string_10_1(text));
        }
        data.extend([0; 16]);
        data.extend(bookmark_string_10_1("S"));
        let end = data.len();

        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::SchematicClassName(body) = bookmark.name_object else {
            panic!("Schematic 名称未进入书签分派");
        };
        assert_eq!(body.version, 1);
        assert_eq!(
            body.strings,
            ["图表".to_string(), "元素".to_string(), "名称".to_string()]
        );
        assert!(matches!(
            body.dataset,
            DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::SchematicDataset(_))
        ));
        assert_eq!(bookmark.name, "S");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17) && refs.contains(18));

        for cut in 0..end {
            let mut reader = Reader::new(&data[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(
                read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err(),
                "Schematic 名称前缀 {cut} 应失败"
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    for cut in 0..raster_end {
        let mut reader = Reader::new(&raster[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(
            read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err(),
            "RasterBandName 前缀 {cut} 应失败"
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn query_table_name_reads_versions_and_rolls_back_unknown_query_description() {
    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    const NAME: [u8; 8] = [4, 0, 0, 0, b'Q', 0, 0, 0];

    for version in [1_u16, 2] {
        let mut data = 1_u16.to_le_bytes().to_vec();
        data.extend(42_i32.to_le_bytes());
        data.extend(QUERY_TABLE_NAME_CLASS_ID);
        data.extend(17_u32.to_le_bytes());
        data.extend(version.to_le_bytes());
        data.extend(EMPTY.repeat(3));
        data.extend([0; 16]);
        let query_offset = data.len();
        if version == 1 {
            data.extend([0; 16]);
        } else {
            data.extend([0x44; 16]);
            data.extend(8_u32.to_le_bytes());
        }
        data.extend(NAME);
        let end = data.len();
        data.push(0x5a);

        let mut refs = RefTable10_1::new();
        refs.register(8);
        let before = refs.clone();
        let mut reader = Reader::new(&data);
        let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        let BookmarkNameEntry10_1::QueryTableName(body) = bookmark.name_object else {
            panic!("应读取 QueryTableName");
        };
        assert_eq!(body.version, version);
        assert_eq!(
            [&body.string_a, &body.string_b, &body.string_c],
            [&String::new(), &String::new(), &String::new()]
        );
        assert_eq!(body.workspace, WorkspaceNameSlot10_1::Null);
        assert!(match body.query_slot {
            UnknownSlot10_1::Null => version == 1,
            UnknownSlot10_1::BackReference { class_id, key } => {
                version == 2 && class_id == [0x44; 16] && key == 8
            }
            UnknownSlot10_1::Unsupported { .. } => false,
        });
        assert_eq!(bookmark.name, "Q");
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0x5a]);
        assert!(refs.contains(17));

        for cut in 0..data.len() - 1 {
            let mut reader = Reader::new(&data[..cut]);
            let mut refs = before.clone();
            assert!(
                read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err(),
                "QueryTableName v{version} 前缀 {cut} 应失败"
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }

        let mut unknown = data.clone();
        unknown[query_offset..query_offset + 16].fill(0x66);
        unknown[query_offset + 16..query_offset + 20].copy_from_slice(&19_u32.to_le_bytes());
        let mut reader = Reader::new(&unknown);
        let mut refs = before.clone();
        assert_eq!(
            read_feature_bookmark_10_1(&mut reader, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::Unsupported
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);
    }
}

#[test]
fn table_name_null_and_backreference() {
    for workspace in [vec![0; 16], {
        let mut v = vec![0x31; 16];
        v.extend(7u32.to_le_bytes());
        v
    }] {
        let data = fixture(&workspace);
        let mut r = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        refs.register(7);
        let b = read_feature_bookmark_10_1(&mut r, 100, &mut refs).unwrap();
        let BookmarkNameEntry10_1::TableName(name) = b.name_object else {
            panic!()
        };
        assert!(matches!(
            name.workspace,
            WorkspaceNameEntry10_1::Null | WorkspaceNameEntry10_1::BackReference { key: 7, .. }
        ));
        assert_eq!(b.raw_int32, 42);
        assert_eq!(r.position(), data.len());
        assert!(refs.contains(17));
    }
}

#[test]
fn failures_rollback_bookmark_and_references() {
    let data = fixture(&[0; 16]);
    for end in 0..data.len() {
        let mut r = Reader::new(&data[..end]);
        let mut refs = RefTable10_1::new();
        refs.register(7);
        assert!(
            read_feature_bookmark_10_1(&mut r, 100, &mut refs).is_err(),
            "{end}"
        );
        assert_eq!(r.position(), 0);
        assert_eq!(refs.len(), 1);
    }
    let mut unknown = vec![0x45; 16];
    unknown.extend(18u32.to_le_bytes());
    let bad = fixture(&unknown);
    let mut r = Reader::new(&bad);
    let mut refs = RefTable10_1::new();
    assert_eq!(
        read_feature_bookmark_10_1(&mut r, 100, &mut refs)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::Unsupported
    );
    assert_eq!(r.position(), 0);
    assert_eq!(refs.len(), 0);
    let mut bad_version = data.clone();
    bad_version[26..28].copy_from_slice(&2u16.to_le_bytes());
    let mut r = Reader::new(&bad_version);
    let mut refs = RefTable10_1::new();
    assert!(read_feature_bookmark_10_1(&mut r, 100, &mut refs).is_err());
    assert_eq!(r.position(), 0);
    assert_eq!(refs.len(), 0);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    assert!(read_feature_bookmark_10_1(&mut r, 1, &mut refs).is_err());
    assert_eq!(r.position(), 0);
    assert_eq!(refs.len(), 0);
}

#[test]
fn new_workspace_and_incomplete_nested_object() {
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;
    let mut workspace = WORKSPACE_NAME_CLASS_ID.to_vec();
    workspace.extend(18u32.to_le_bytes());
    workspace.extend(1u16.to_le_bytes());
    for _ in 0..3 {
        workspace.extend(2u32.to_le_bytes());
        workspace.extend([0, 0]);
    }
    let prefix = workspace.clone();
    workspace.extend([0; 16]);
    workspace.push(0);
    workspace.extend(0u32.to_le_bytes());
    let data = fixture(&workspace);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let b = read_feature_bookmark_10_1(&mut r, 100, &mut refs).unwrap();
    assert!(
        matches!(b.name_object, BookmarkNameEntry10_1::TableName(body) if matches!(body.workspace, WorkspaceNameEntry10_1::NewObject(ref w) if w.key == 18 && w.stop.is_none()))
    );
    assert_eq!(r.position(), data.len());
    assert!(refs.contains(17) && refs.contains(18));
    let mut incomplete = prefix;
    incomplete.extend([0x49; 16]);
    incomplete.extend(19u32.to_le_bytes());
    let data = fixture(&incomplete);
    let mut r = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    refs.register(7);
    assert_eq!(
        read_feature_bookmark_10_1(&mut r, 100, &mut refs)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::Unsupported
    );
    assert_eq!(r.position(), 0);
    assert_eq!(refs.len(), 1);
}

#[test]
fn netcdf_feature_name_preserves_keys_and_boundaries_across_hosts() {
    use mxd2qgz::layers::feature_layer::{
        DataSource10_1, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
    };
    use mxd2qgz::maps::standalone_table::{
        StandaloneNameBody10_1, StandaloneTableStop10_1, read_standalone_table_prefix_10_1,
    };
    use mxd2qgz::objects::dataset_name_more::{
        DatasetNameMoreBody10_1, NETCDF_FEATURE_CLASS_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::string_array::STR_ARRAY_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    const NAMED: [u8; 8] = [4, 0, 0, 0, b'x', 0, 0, 0];
    const LONG_ARRAY: [u8; 16] = [
        0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0, 0xc0, 0x4f, 0x8e, 0xce, 0x27,
    ];
    for version in 0_u16..=2 {
        let mut name = NETCDF_FEATURE_CLASS_NAME_CLASS_ID.to_vec();
        name.extend(17_u32.to_le_bytes());
        name.extend(version.to_le_bytes());
        name.extend(EMPTY.repeat(4));
        name.extend((-1_i32).to_le_bytes());
        name.extend(24_i32.to_le_bytes());
        name.extend([0; 16]);

        name.extend(STR_ARRAY_CLASS_ID);
        name.extend(18_u32.to_le_bytes());
        name.extend(1_u16.to_le_bytes());
        name.extend(1_u32.to_le_bytes());
        name.extend(NAMED);
        name.extend([0; 16]);
        name.extend(STR_ARRAY_CLASS_ID);
        name.extend(18_u32.to_le_bytes());
        let indices_offset = name.len();
        name.extend(LONG_ARRAY);
        name.extend(19_u32.to_le_bytes());
        name.extend(1_u16.to_le_bytes());
        name.extend(1_u32.to_le_bytes());
        name.extend(3_i32.to_le_bytes());
        if version == 2 {
            name.extend(NAMED.repeat(4));
        }

        let mut bookmark = 1_u16.to_le_bytes().to_vec();
        bookmark.extend(42_i32.to_le_bytes());
        bookmark.extend(&name);
        bookmark.extend(NAMED);
        let mut reader = Reader::new(&bookmark);
        let mut refs = RefTable10_1::new();
        let parsed = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        let BookmarkNameEntry10_1::NetCdfFeatureClassName(body) = parsed.name_object else {
            panic!("缺少 NetCDF 要素类名称");
        };
        assert_eq!(
            body.arrays.as_ref().unwrap()[2].body,
            Some((1, vec!["x".to_owned()])),
        );
        let mut no_indices = bookmark.clone();
        let offset = 6 + indices_offset;
        no_indices.splice(offset..offset + 30, [0; 16]);
        let mut invalid = Reader::new(&no_indices);
        let mut invalid_refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut invalid, 4096, &mut invalid_refs).is_err());
        assert_eq!(invalid.position(), 0);
        assert!(invalid_refs.is_empty());

        assert_eq!(parsed.name, "x");
        assert_eq!(body.version, version);
        assert_eq!(body.longs, [-1, 24]);
        assert_eq!(
            body.strings_v2,
            (version == 2).then(|| [
                "x".to_string(),
                "x".to_string(),
                "x".to_string(),
                "x".to_string()
            ])
        );
        assert!(matches!(
            body.arrays.unwrap()[2].entry,
            ObjectEntry10_1::BackReference { key: 18, .. }
        ));
        assert_eq!(body.dimension_indices.unwrap().body, Some((1, vec![3])));
        assert!((17..=19).all(|key| refs.contains(key)));
        assert_eq!(reader.remaining(), 0);

        for end in [bookmark.len() - 1, 6 + name.len() - 1] {
            let mut reader = Reader::new(&bookmark[..end]);
            let mut refs = RefTable10_1::new();
            refs.register(11);
            let before = refs.clone();
            assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }

        let mut layer = 0_u16.to_le_bytes().to_vec();
        layer.extend(EMPTY.repeat(2));
        layer.extend([0; 6]);
        layer.extend(&name);
        let renderer_offset = layer.len();
        layer.extend([0x55; 16]);
        layer.extend(20_u32.to_le_bytes());
        let mut reader = Reader::new(&layer);
        let mut refs = RefTable10_1::new();
        let parsed = read_feature_layer_prefix_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(matches!(
            parsed.data_source,
            DataSource10_1::NetCdfFeatureClassName(body) if body.key == 17
        ));
        assert!(matches!(
            parsed.stop,
            FeatureLayerStop10_1::RendererNewObject { key: 20, .. }
        ));
        assert_eq!(reader.position(), renderer_offset);
        assert!(refs.contains(19) && !refs.contains(20));

        let mut table = 1_u16.to_le_bytes().to_vec();
        table.extend(&name);
        table.extend(0_u32.to_le_bytes());
        table.extend(EMPTY);
        let mut reader = Reader::new(&table);
        let mut refs = RefTable10_1::new();
        let parsed = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 64, 4096).unwrap();
        assert_eq!(parsed.stop, StandaloneTableStop10_1::Complete);
        assert!(matches!(
            parsed.dataset_name_body,
            Some(StandaloneNameBody10_1::OtherTabularName(body))
                if matches!(*body, DatasetNameMoreBody10_1::NetCdfFeatureClass(_))
        ));
        assert!(refs.contains(19));
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn xy_event_bookmark_reads_nested_table_fields_and_spatial_reference() {
    use mxd2qgz::objects::dataset_name_more::{
        DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, XY_EVENT_2_FIELDS_PROPERTIES_CLASS_ID,
        XY_EVENT_SOURCE_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::envelope::SpatialReferenceEntry10_1;
    use mxd2qgz::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let encoded_char = |c: u8| [4, 0, 0, 0, c, 0, 0, 0];
    let empty = [2, 0, 0, 0, 0, 0];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(11_i32.to_le_bytes());
    bytes.extend(XY_EVENT_SOURCE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(encoded_char(b'A'));
    bytes.extend(empty);
    let source_offset = bytes.len();
    bytes.extend(TABLE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(empty.repeat(3));
    bytes.extend([0; 16]);
    bytes.extend(XY_EVENT_2_FIELDS_PROPERTIES_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for field in [b'x', b'y', b'z'] {
        bytes.extend(encoded_char(field));
    }
    bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    bytes.extend(7_u32.to_le_bytes());
    bytes.extend(encoded_char(b'B'));
    let mut refs = RefTable10_1::new();
    refs.register(7);
    let before = refs.clone();
    let mut reader = Reader::new(&bytes);
    let parsed = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
    let BookmarkNameEntry10_1::XyEventSourceName(source) = parsed.name_object else {
        panic!("缺少 XY 事件源");
    };
    assert_eq!(parsed.name, "B");
    assert!(matches!(
        source.source,
        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Table(_))
    ));
    let (entry, fields) = source.properties.unwrap();
    let fields = fields.unwrap();
    assert_eq!(
        [fields.x_field, fields.y_field, fields.z_field],
        ["x", "y", "z"]
    );
    assert!(matches!(entry, ObjectEntry10_1::NewObject { key: 19, .. }));
    assert!(matches!(
        source.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference { key: 7, .. })
    ));
    assert_eq!(reader.remaining(), 0);
    assert!((17..=19).all(|key| refs.contains(key)));

    let mut unknown = bytes.clone();
    unknown[source_offset..source_offset + 16].fill(0x69);
    for malformed in [&unknown[..], &bytes[..bytes.len() - 1]] {
        let mut reader = Reader::new(malformed);
        let mut refs = before.clone();
        assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);
    }
}

#[test]
fn route_event_name_keeps_locator_and_property_versions_aligned() {
    use mxd2qgz::layers::feature_layer::{
        DataSource10_1, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
    };
    use mxd2qgz::maps::standalone_table::{
        StandaloneNameBody10_1, StandaloneTableStop10_1, read_standalone_table_prefix_10_1,
    };
    use mxd2qgz::objects::dataset_name_more::{DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1};
    use mxd2qgz::objects::route_event_name::{
        ROUTE_EVENT_SOURCE_NAME_CLASS_ID, ROUTE_MEASURE_LINE_PROPERTIES_CLASS_ID,
        ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID, ROUTE_MEASURE_POINT_PROPERTIES_CLASS_ID,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    const NAME: [u8; 8] = [4, 0, 0, 0, b'r', 0, 0, 0];
    for (base, props_version, point) in [
        (0_u16, 0_u16, true),
        (1, 1, true),
        (2, 2, true),
        (3, 3, false),
        (4, 4, true),
        (4, 5, true),
        (4, 5, false),
    ] {
        let mut name = ROUTE_EVENT_SOURCE_NAME_CLASS_ID.to_vec();
        name.extend(17_u32.to_le_bytes());
        name.extend(1_u16.to_le_bytes());
        name.extend(EMPTY.repeat(2));

        name.extend(ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID);
        name.extend(18_u32.to_le_bytes());
        name.extend(1_u16.to_le_bytes());
        name.extend(base.to_le_bytes());
        name.extend(EMPTY.repeat(4));
        let route_offset = name.len();
        name.extend(TABLE_NAME_CLASS_ID);
        name.extend(19_u32.to_le_bytes());
        name.extend(1_u16.to_le_bytes());
        name.extend(EMPTY.repeat(3));
        name.extend([0; 16]);
        name.extend(NAME);
        if base < 4 {
            name.extend(0x1234_u16.to_le_bytes());
        }
        name.extend(17_i32.to_le_bytes());
        if base == 2 {
            name.extend((-92_i32).to_le_bytes());
        }

        name.extend(TABLE_NAME_CLASS_ID);
        name.extend(19_u32.to_le_bytes());
        name.extend(if point {
            ROUTE_MEASURE_POINT_PROPERTIES_CLASS_ID
        } else {
            ROUTE_MEASURE_LINE_PROPERTIES_CLASS_ID
        });
        name.extend(20_u32.to_le_bytes());
        name.extend(props_version.to_le_bytes());
        name.extend(EMPTY.repeat(4));
        name.push(1);
        name.extend(23_i32.to_le_bytes());
        if props_version >= 3 {
            name.extend(0x4321_u16.to_le_bytes());
            name.extend(NAME);
        }
        if props_version >= 5 {
            name.extend(0x2468_u16.to_le_bytes());
        }
        if point && props_version != 1 {
            name.extend(4_u16.to_le_bytes());
            name.extend(5_u16.to_le_bytes());
            name.extend(NAME);
            name.extend(6_u16.to_le_bytes());
            if props_version >= 4 {
                name.extend(7_u16.to_le_bytes());
            }
        }

        let mut bookmark = 1_u16.to_le_bytes().to_vec();
        bookmark.extend(12_i32.to_le_bytes());
        bookmark.extend(&name);
        bookmark.extend(NAME);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bookmark);
        let parsed = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RouteEventSourceName(source) = parsed.name_object else {
            panic!("缺少路线事件源");
        };
        let locator = source.locator_body.unwrap();
        assert_eq!(locator.base_version, base);
        let suffix = locator.suffix.unwrap();
        assert_eq!(suffix.string_40, "r");
        assert_eq!(suffix.legacy_short, (base < 4).then_some(0x1234));
        assert_eq!(suffix.long_44, 17);
        assert_eq!(suffix.legacy_long_v2, (base == 2).then_some(-92));
        assert!(matches!(
            source.event_table,
            Some(DatasetNameMoreSlot10_1::BackReference { key: 19, .. })
        ));
        assert!(matches!(
            source.properties,
            Some(ObjectEntry10_1::NewObject { key: 20, .. })
        ));
        let properties = source.properties_body.unwrap();
        assert_eq!(properties.long_1c, 23);
        assert_eq!(
            properties.fields_v3.as_ref().map(|(s, _)| *s),
            (props_version >= 3).then_some(0x4321)
        );
        assert_eq!(properties.short_v5, (props_version >= 5).then_some(0x2468));
        assert_eq!(
            properties.point_fields.is_some(),
            point && props_version != 1
        );
        if let Some(fields) = properties.point_fields {
            assert_eq!(fields.shorts, [4, 5]);
            assert_eq!(fields.short_4c, 6);
            assert_eq!(fields.short_v4, (props_version >= 4).then_some(7));
        }
        assert_eq!(parsed.name, "r");
        assert_eq!(reader.remaining(), 0);
        assert!((17..=20).all(|key| refs.contains(key)));

        let mut unknown_route = bookmark.clone();
        unknown_route[6 + route_offset..6 + route_offset + 16].fill(0x74);
        for bad in [&bookmark[..6 + name.len() - 1], &unknown_route] {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(bad);
            assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
            assert!(refs.is_empty());
            assert_eq!(reader.position(), 0);
        }

        let mut layer = 0_u16.to_le_bytes().to_vec();
        layer.extend(EMPTY.repeat(2));
        layer.extend([0; 6]);
        layer.extend(&name);
        let end = layer.len();
        layer.extend([0x55; 16]);
        layer.extend(21_u32.to_le_bytes());
        let mut reader = Reader::new(&layer);
        let mut refs = RefTable10_1::new();
        let parsed = read_feature_layer_prefix_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(
            matches!(parsed.data_source, DataSource10_1::RouteEventSourceName(body) if body.key == 17)
        );
        assert!(matches!(
            parsed.stop,
            FeatureLayerStop10_1::RendererNewObject { key: 21, .. }
        ));
        assert_eq!(reader.position(), end);
        assert!(refs.contains(20) && !refs.contains(21));

        let mut table = 1_u16.to_le_bytes().to_vec();
        table.extend(&name);
        table.extend(0_u32.to_le_bytes());
        table.extend(EMPTY);
        let mut reader = Reader::new(&table);
        let mut refs = RefTable10_1::new();
        let parsed = read_standalone_table_prefix_10_1(&mut reader, &mut refs, 64, 4096).unwrap();
        assert_eq!(parsed.stop, StandaloneTableStop10_1::Complete);
        assert!(matches!(
            parsed.dataset_name_body,
            Some(StandaloneNameBody10_1::OtherTabularName(body))
                if matches!(*body, DatasetNameMoreBody10_1::RouteEventSource(_))
        ));
        assert!(refs.contains(20));
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn coverage_wrappers_share_inner_object_keys() {
    use mxd2qgz::layers::feature_layer::{
        COVERAGE_FEATURE_CLASS_NAME_CLASS_ID, DatasetNameEntry10_1, FeatureClassNameNested10_1,
    };
    use mxd2qgz::objects::dataset_name::{COVERAGE_NAME_CLASS_ID, WorkspaceNameSlot10_1};
    use mxd2qgz::objects::dataset_name_more::DatasetNameMoreBody10_1;
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    for version in [0_u16, 1] {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(11_i32.to_le_bytes());
        bytes.extend(COVERAGE_FEATURE_CLASS_NAME_CLASS_ID);
        bytes.extend(17_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        bytes.extend((-23_i32).to_le_bytes());
        bytes.extend(24_i32.to_le_bytes());
        bytes.push(0xff);
        bytes.extend(2_u16.to_le_bytes()); // 内联 FeatureClassName。
        bytes.extend(EMPTY.repeat(4));
        bytes.extend([0; 8]);
        bytes.extend(1_u16.to_le_bytes()); // 按 IDatasetName 分派 CoverageName。
        bytes.extend(COVERAGE_NAME_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        bytes.extend(25_i32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes()); // 内联 FeatureDatasetName。
        bytes.extend(EMPTY.repeat(4));
        bytes.push(1);
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(7_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes()); // 内联 FeatureClassName 的第二选择子。
        bytes.extend([4, 0, 0, 0, b'C', 0, 0, 0]);

        let mut refs = RefTable10_1::new();
        refs.register(7);
        let before = refs.clone();
        let mut reader = Reader::new(&bytes);
        let mark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
        let BookmarkNameEntry10_1::FeatureClassName(feature) = mark.name_object else {
            panic!("缺少 Coverage 要素类名称");
        };
        let header = feature.coverage_wrapper.unwrap();
        assert_eq!(header.version, version);
        assert_eq!(header.longs, [-23, 24]);
        assert_eq!(header.flag, 0xff);
        assert_eq!(feature.key, 17);
        assert_eq!(feature.version1_selector, Some(0));
        let FeatureClassNameNested10_1::DatasetName(DatasetNameEntry10_1::NewObject(dataset)) =
            feature.nested
        else {
            panic!("缺少 Coverage 数据集名称")
        };
        let DatasetNameMoreBody10_1::FeatureDataset(dataset) = *dataset else {
            panic!("缺少内联 FeatureDatasetName");
        };
        assert_eq!(dataset.key, 18);
        assert_eq!(dataset.coverage_wrapper, Some((version, 25)));
        assert!(matches!(
            dataset.workspace,
            WorkspaceNameSlot10_1::BackReference { key: 7, .. }
        ));
        assert_eq!(mark.name, "C");
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(17) && refs.contains(18));
        assert_eq!(refs.len(), 3);

        for end in [28, bytes.len() - 9, bytes.len() - 1] {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = before.clone();
            assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn cad_drawing_name_reads_workspace_reference_before_feature_class_suffix() {
    use mxd2qgz::layers::feature_layer::{
        DatasetNameEntry10_1, FEATURE_CLASS_NAME_CLASS_ID, FeatureClassNameNested10_1,
    };
    use mxd2qgz::objects::dataset_name::WorkspaceNameSlot10_1;
    use mxd2qgz::objects::dataset_name_more::{CAD_DRAWING_NAME_CLASS_ID, DatasetNameMoreBody10_1};
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];
    const NAME: [u8; 8] = [4, 0, 0, 0, b'D', 0, 0, 0];
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(11_i32.to_le_bytes());
    bytes.extend(FEATURE_CLASS_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(EMPTY.repeat(4));
    bytes.extend([0; 8]);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(CAD_DRAWING_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    let drawing_version = bytes.len();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(NAME);
    bytes.extend(EMPTY.repeat(2));
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(7_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(NAME);

    let mut refs = RefTable10_1::new();
    refs.register(7);
    let before = refs.clone();
    let mut reader = Reader::new(&bytes);
    let mark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
    let BookmarkNameEntry10_1::FeatureClassName(feature) = mark.name_object else {
        panic!("缺少要素类名称");
    };
    let FeatureClassNameNested10_1::DatasetName(DatasetNameEntry10_1::NewObject(drawing)) =
        feature.nested
    else {
        panic!("缺少 CAD 图名")
    };
    let DatasetNameMoreBody10_1::CadDrawing(drawing) = *drawing else {
        panic!("缺少 CadDrawingName 对象体");
    };
    assert_eq!(drawing.version, 1);
    assert_eq!(drawing.strings, ["D", "", ""]);
    assert!(matches!(
        drawing.workspace,
        WorkspaceNameSlot10_1::BackReference { key: 7, .. }
    ));
    assert_eq!(feature.version1_selector, Some(0));
    assert_eq!(mark.name, "D");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(17) && refs.contains(18));

    for version in [0_u16, 2] {
        let mut malformed = bytes.clone();
        malformed[drawing_version..drawing_version + 2].copy_from_slice(&version.to_le_bytes());
        for bad in [&malformed[..], &bytes[..bytes.len() - 12]] {
            let mut reader = Reader::new(bad);
            let mut refs = before.clone();
            assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }
}

#[test]
fn ams_bookmark_reads_ole_workspace_and_preserves_object_stream_keys() {
    use mxd2qgz::objects::{
        ams_dataset_name::AMS_DATASET_NAME_CLASS_ID,
        property_set::PROPERTY_SET_CLASS_ID,
        workspace_name::{PropertySetSlot10_1, WORKSPACE_NAME_CLASS_ID},
    };

    let mut data = 1_u16.to_le_bytes().to_vec();
    data.extend(42_i32.to_le_bytes());
    data.extend(AMS_DATASET_NAME_CLASS_ID);
    data.extend(17_u32.to_le_bytes());
    let bstr_offset = data.len();
    data.extend(5_u32.to_le_bytes());
    data.extend([0xff, 0, 0x20, 0, 0]);
    data.extend(0_u32.to_le_bytes());
    data.extend(2_u32.to_le_bytes());
    data.extend([0, 0]);
    data.extend(0_u32.to_le_bytes());
    for value in [5_i32, -2, 99] {
        data.extend(value.to_le_bytes());
    }

    let variant_offset = data.len();
    data.extend(13_u16.to_le_bytes());
    let workspace_offset = data.len();
    data.extend(WORKSPACE_NAME_CLASS_ID);
    data.extend(1_u16.to_le_bytes());
    for _ in 0..3 {
        data.extend([2, 0, 0, 0, 0, 0]);
    }
    data.extend(PROPERTY_SET_CLASS_ID);
    data.extend(18_u32.to_le_bytes());
    data.extend(1_u16.to_le_bytes());
    data.extend(0_i32.to_le_bytes());
    data.extend([0; 5]); // 工厂标志和尾随整数。
    data.extend((-12_i16).to_le_bytes());
    data.extend([4, 0, 0, 0, b'B', 0, 0, 0]);
    let end = data.len();
    data.push(0xa5);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let bookmark = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
    let BookmarkNameEntry10_1::AmsDatasetName(body) = bookmark.name_object else {
        panic!("预期 AMSDatasetName");
    };
    assert_eq!(
        body.strings,
        [Some(vec![0xff, 0, 0x20]), None, Some(vec![]), None]
    );
    assert_eq!(body.scalars, [5, -2, 99]);
    assert!(matches!(
        body.workspace_name.property_set,
        PropertySetSlot10_1::PropertySet(_)
    ));
    assert_eq!(body.trailing_short, -12);
    assert_eq!(bookmark.name, "B");
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0xa5]);
    let mut expected_refs = RefTable10_1::new();
    for (class_id, key) in [
        (AMS_DATASET_NAME_CLASS_ID, 17_u32),
        (PROPERTY_SET_CLASS_ID, 18),
    ] {
        let mut header = class_id.to_vec();
        header.extend(key.to_le_bytes());
        mxd2qgz::storage::object_refs::read_object_entry_10_1(
            &mut Reader::new(&header),
            &mut expected_refs,
        )
        .unwrap();
    }
    assert_eq!(refs, expected_refs);

    let verify_failure = |input: &[u8], max_bytes| {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        refs.register(3);
        let before = refs.clone();
        let error = read_feature_bookmark_10_1(&mut reader, max_bytes, &mut refs).unwrap_err();
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);
        error.kind()
    };
    for prefix in 1..end {
        verify_failure(&data[..prefix], 4096);
    }
    use std::io::ErrorKind::{InvalidData, Unsupported};
    let mut bad_length = data.clone();
    bad_length[bstr_offset..bstr_offset + 4].copy_from_slice(&1_u32.to_le_bytes());
    assert_eq!(verify_failure(&bad_length, 4096), InvalidData);
    assert_eq!(verify_failure(&data, 4), InvalidData);

    let mut wrong_variant = data.clone();
    wrong_variant[variant_offset] = 9;
    assert_eq!(verify_failure(&wrong_variant, 4096), InvalidData);
    let mut unknown_workspace = data.clone();
    unknown_workspace[workspace_offset..workspace_offset + 16].fill(0x5a);
    assert_eq!(verify_failure(&unknown_workspace, 4096), Unsupported);
    let mut null_workspace = data;
    null_workspace[workspace_offset..workspace_offset + 16].fill(0);
    assert_eq!(verify_failure(&null_workspace, 4096), InvalidData);
}

#[test]
fn bookmark_name_reference_retains_the_registered_class() {
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    let mut definition = FGDB_FEATURE_CLASS_NAME_CLASS_ID.to_vec();
    definition.extend(17_u32.to_le_bytes());
    let mut refs = RefTable10_1::new();
    read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();
    let saved = refs.clone();

    let mut body = 1_u16.to_le_bytes().to_vec();
    body.extend(42_i32.to_le_bytes());
    body.extend(TABLE_NAME_CLASS_ID);
    body.extend(17_u32.to_le_bytes());
    body.extend([4, 0, 0, 0, b'X', 0, 0, 0, 0x42]);
    let mut reader = Reader::new(&body);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(
        bookmark.name_object,
        BookmarkNameEntry10_1::BackReference {
            class_id: FGDB_FEATURE_CLASS_NAME_CLASS_ID,
            key: 17,
        }
    );
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.read_bytes(1).unwrap(), &[0x42]);
    assert_eq!(refs, saved);
}

#[test]
fn bookmark_reads_a_netcdf_table_name() {
    use mxd2qgz::objects::dataset_name_more::NETCDF_TABLE_NAME_CLASS_ID;

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(NETCDF_TABLE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend([2, 0, 0, 0, 0, 0]);
    }
    bytes.extend([0; 16 * 5]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(
        matches!(bookmark.name_object, BookmarkNameEntry10_1::NetCdfTableName(body)
        if body.version == 1 && body.arrays.is_some())
    );
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(17));

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes[..bytes.len() - 9]);
    assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn bookmark_reads_db_table_name_without_a_version() {
    use mxd2qgz::objects::dataset_name_more::DB_TABLE_NAME_CLASS_ID;

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(DB_TABLE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    for value in [b'D', b'T', b'C'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'G', 0, 0, 0]);
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(
        matches!(bookmark.name_object, BookmarkNameEntry10_1::DbTableName(body)
        if body.strings_a == ["D", "T", "C"] && body.int_a == 7)
    );
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(17));

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes[..bytes.len() - 9]);
    assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn bookmark_reads_cad_drawing_name() {
    use mxd2qgz::objects::dataset_name_more::CAD_DRAWING_NAME_CLASS_ID;

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(CAD_DRAWING_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    let name_start = bytes.len();
    bytes.extend(1_u16.to_le_bytes());
    for value in [b'D', b'N', b'C'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(
        matches!(bookmark.name_object, BookmarkNameEntry10_1::CadDrawingName(body)
        if body.key == 17 && body.strings == ["D", "N", "C"])
    );
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(17));

    for end in name_start..bytes.len() {
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes[..end]);
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn bookmark_reads_a_referenced_table_and_its_child_key() {
    use mxd2qgz::objects::dataset_name_more::REFERENCED_TABLE_NAME_CLASS_ID;

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(REFERENCED_TABLE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    let version_offset = bytes.len();
    bytes.extend(i16::MIN.to_le_bytes());
    bytes.extend(TABLE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend([2, 0, 0, 0, 0, 0]);
    }
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(
        matches!(bookmark.name_object, BookmarkNameEntry10_1::ReferencedTableName(body)
        if body.version == i16::MIN && matches!(body.source,
            DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Table(_))))
    );
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(17) && refs.contains(18));

    let mut newer = bytes.clone();
    newer[version_offset..version_offset + 2].copy_from_slice(&2_u16.to_le_bytes());
    for invalid in [&bytes[..bytes.len() - 9], newer.as_slice()] {
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(invalid);
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn bookmark_reads_geometric_network_name_before_its_label() {
    use mxd2qgz::objects::dataset_name_more::GEOMETRIC_NETWORK_NAME_CLASS_ID;

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(GEOMETRIC_NETWORK_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for value in [b'N', b'D', b'C'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(
        matches!(bookmark.name_object, BookmarkNameEntry10_1::GeometricNetworkName(body)
        if body.key == 17 && body.string_a == "N" && body.dataset == DatasetNameMoreSlot10_1::Null)
    );
    assert_eq!(bookmark.name, "X");
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(17));

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes[..bytes.len() - 9]);
    assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn bookmark_uses_signed_version_comparison() {
    for version in [i16::MIN, -1, 0, 1, 2] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(42_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0, 0x42]);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs);
        if version <= 1 {
            let bookmark = result.unwrap();
            assert_eq!(bookmark.version, version as u16);
            assert_eq!(bookmark.raw_int32, 42);
            assert_eq!(bookmark.name, "X");
            assert_eq!(reader.read_bytes(1).unwrap(), &[0x42]);
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
            assert_eq!(reader.position(), 0);
        }
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_topology_name_and_leaves_host_suffix() {
    use mxd2qgz::objects::dataset_name::{DatasetNameSlot10_1, TOPOLOGY_NAME_CLASS_ID};

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(TOPOLOGY_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    for value in [b'T', b'C'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object, BookmarkNameEntry10_1::TopologyName(body)
            if body.key == 17 && body.version == u16::MAX && body.string_a == "T"
                && body.string_b == "C" && body.dataset == DatasetNameSlot10_1::Null
    ));
    assert_eq!(result.name, "X");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_feature_dataset_name_and_leaves_host_suffix() {
    use mxd2qgz::objects::dataset_name::{FEATURE_DATASET_NAME_CLASS_ID, WorkspaceNameSlot10_1};

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(FEATURE_DATASET_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    for value in [b'T', b'C', b'D', b'E'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.push(1);
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object, BookmarkNameEntry10_1::FeatureDatasetName(body)
            if body.key == 17 && body.version == u16::MAX && body.string_a == "T"
                && body.string_b == "C" && body.flag == 1 && body.workspace == WorkspaceNameSlot10_1::Null
    ));
    assert_eq!(result.name, "X");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_coverage_name_and_leaves_host_suffix() {
    use mxd2qgz::objects::dataset_name::{COVERAGE_NAME_CLASS_ID, WorkspaceNameSlot10_1};

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(COVERAGE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend(12_i32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for value in [b'T', b'C', b'D', b'E'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.push(1);
    bytes.extend([0; 16]);
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object, BookmarkNameEntry10_1::CoverageName(body)
            if body.key == 17 && body.coverage_wrapper == Some((u16::MAX, 12)) && body.version == 1 && body.string_a == "T"
                && body.string_b == "C" && body.flag == 1 && body.workspace == WorkspaceNameSlot10_1::Null
    ));
    assert_eq!(result.name, "X");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));
    assert_eq!(refs.len(), 1);

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_workspace_name_without_opening_it() {
    use mxd2qgz::objects::workspace_name::{PropertySetSlot10_1, WORKSPACE_NAME_CLASS_ID};

    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    for value in [b'W', b'P', b'T'] {
        bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
    }
    bytes.extend([0; 16]);
    bytes.push(0);
    bytes.extend(13_i32.to_le_bytes());
    bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object, BookmarkNameEntry10_1::WorkspaceName(body)
            if body.key == 17 && body.version == u16::MAX && body.path == "W"
                && body.property_set == PropertySetSlot10_1::Null
    ));
    assert_eq!(result.name, "X");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_raster_dataset_name_before_bookmark_text() {
    use mxd2qgz::layers::raster_dataset_name::{
        RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };

    let mut bytes = bookmark_header_10_1(&RASTER_DATASET_NAME_CLASS_ID, 17);
    bytes.extend(4_i32.to_le_bytes());
    for text in ["image.tif", "", "Raster Dataset", ""] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes.extend([0; 16]);
    bytes.extend(10_i32.to_le_bytes());
    bytes.extend(20_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object, BookmarkNameEntry10_1::RasterDatasetName(body)
            if matches!(*body, RasterDatasetNameEntryBody10_1::Dataset(_))
    ));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_raster_dataset_name83_preserves_optional_record_boundary() {
    use mxd2qgz::layers::raster_dataset_name::{
        RASTER_DATASET_NAME83_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };

    for has_records in [false, true] {
        let mut bytes = bookmark_header_10_1(&RASTER_DATASET_NAME83_CLASS_ID, 17);
        bytes.extend(2_i32.to_le_bytes());
        for text in ["image", "", "Raster Dataset", ""] {
            bytes.extend(bookmark_string_10_1(text));
        }
        bytes.extend([0; 16]);
        if has_records {
            for value in [0x00ff_00fa_i32, 0, 1, 4, 27, 2] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(bookmark_string_10_1("栅格记录"));
            bytes.extend([0xff; 8]);
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
            panic!("应读取旧版栅格名称");
        };
        let RasterDatasetNameEntryBody10_1::Dataset83 { records, .. } = *body else {
            panic!("应读取 RasterDatasetName83");
        };
        assert_eq!(
            records.map(|r| (r.first_long, r.text)),
            has_records.then(|| (27, "栅格记录".into()))
        );
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17));

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_raster_band_collection_and_its_array() {
    use mxd2qgz::layers::raster_dataset_name::{
        RASTER_BAND_COLLECTION_NAME_CLASS_ID, RASTER_DATASET_NAME_CLASS_ID,
        RasterDatasetNameEntryBody10_1,
    };

    let mut bytes = bookmark_header_10_1(&RASTER_BAND_COLLECTION_NAME_CLASS_ID, 17);
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("波段集合"));
    bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    for text in ["image.tif", "", "", ""] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes.extend([0; 16]);
    bytes.extend([
        0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0, 0xc0, 0x4f, 0x8e, 0xce, 0x27,
    ]); // LongArray
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for value in [3_i32, 2, 0, 2] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::BandCollection(collection) = *body else {
        panic!("应读取波段集合");
    };
    assert_eq!(collection.array, Some((1, vec![2, 0, 2])));
    assert_eq!(collection.version, 2);
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!([17, 18, 19].into_iter().all(|key| refs.contains(key)));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_netcdf_raster_name_with_versioned_dimensions() {
    use mxd2qgz::layers::raster_dataset_name::{
        NETCDF_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::objects::string_array::STR_ARRAY_CLASS_ID;

    for version in [-1_i32, 1, 2] {
        let mut bytes = bookmark_header_10_1(&NETCDF_RASTER_DATASET_NAME_CLASS_ID, 17);
        bytes.extend(version.to_le_bytes());
        for text in ["name", "", "", "", "temperature", "", "", ""] {
            bytes.extend(bookmark_string_10_1(text));
        }
        if version == 1 {
            bytes.extend(bookmark_string_10_1("旧版字段"));
            bytes.extend([0; 16]);
        }
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("time"));
        bytes.extend([0; 32]); // 空维度值数组和工作空间。
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
            panic!("应读取栅格名称");
        };
        let RasterDatasetNameEntryBody10_1::NetCdf(name) = *body else {
            panic!("应读取 NetCDF 栅格名称");
        };
        assert_eq!(name.version, version);
        assert_eq!(name.variable(), "temperature");
        assert_eq!(name.legacy.is_some(), version == 1);
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17) && refs.contains(18));
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_cache_raster_name_and_envelope() {
    use mxd2qgz::layers::raster_dataset_name::{
        CACHE_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&CACHE_RASTER_DATASET_NAME_CLASS_ID, 17);
    bytes.extend(0xffff_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("缓存"));
    bytes.extend(bookmark_string_10_1("图层"));
    bytes.extend([0; 32]); // 空工作空间和瓦片读取器。
    bytes.extend(ENVELOPE_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend([0; 32]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::Cache(name) = *body else {
        panic!("应读取缓存名称");
    };
    assert_eq!(name.version, u16::MAX);
    assert_eq!(name.strings, ["缓存", "图层"]);
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));
    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_mosaic_name_and_versioned_array() {
    use mxd2qgz::layers::raster_dataset_name::{
        MOSAIC_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::objects::string_array::STR_ARRAY_CLASS_ID;

    for version in [-1_i16, 4] {
        let mut bytes = bookmark_header_10_1(&MOSAIC_DATASET_NAME_CLASS_ID, 17);
        bytes.extend(version.to_le_bytes());
        bytes.extend(bookmark_string_10_1("镶嵌"));
        bytes.extend(bookmark_string_10_1("名称"));
        bytes.extend([0; 16]);
        if version == 4 {
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(33_i32.to_le_bytes());
            bytes.extend(STR_ARRAY_CLASS_ID);
            bytes.extend(18_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(bookmark_string_10_1("项"));
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
            panic!("应读取栅格名称");
        };
        let RasterDatasetNameEntryBody10_1::Mosaic(name) = *body else {
            panic!("应读取镶嵌数据集名称");
        };
        assert_eq!(name.version, version as u16);
        assert_eq!(name.strings, ["镶嵌", "名称"]);
        assert_eq!(name.long_48, (version == 4).then_some(33));
        assert_eq!(name.string_array.is_some(), version == 4);
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17));
        assert_eq!(refs.contains(18), version == 4);
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_sample_filegdb_raster_name_to_rasterio_end() {
    use mxd2qgz::layers::fgdb_raster_dataset_name::FGDB_RASTER_DATASET_NAME_CLASS_ID;
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1;
    use std::io::Read;

    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    assert_eq!(maps[0x500a..0x501a], FGDB_RASTER_DATASET_NAME_CLASS_ID);
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(&maps[0x500a..0x51d9]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::FileGeodatabase { name, records } = *body else {
        panic!("应读取 FileGDB 栅格名称");
    };
    assert_eq!(name.delegated_name.key, 260);
    assert!(records.stop.is_none());
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(260) && refs.contains(261));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_function_raster_name_through_v7_tail() {
    use mxd2qgz::layers::image_server_rules::IDENTITY_FUNCTION_CLASS_ID;
    use mxd2qgz::layers::raster_dataset_name::{
        FUNCTION_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };

    let mut bytes = bookmark_header_10_1(&FUNCTION_RASTER_DATASET_NAME_CLASS_ID, 17);
    bytes.extend(7_u16.to_le_bytes());
    for text in ["函数栅格", "", ""] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("image.afr"));
    bytes.extend(IDENTITY_FUNCTION_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 32]); // 空输入和属性集。
    for text in ["v6", "v7", "书签"] {
        bytes.extend(bookmark_string_10_1(text));
    }
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::Function(name) = *body else {
        panic!("应读取函数栅格名称");
    };
    assert_eq!(name.version, 7);
    assert_eq!(name.path, "image.afr");
    assert_eq!(name.string_48.as_deref(), Some("v7"));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_image_service_raster_name_and_transform() {
    use mxd2qgz::layers::raster_dataset_name::{
        IS_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };
    use mxd2qgz::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&IS_RASTER_DATASET_NAME_CLASS_ID, 17);
    bytes.extend(4_u16.to_le_bytes());
    for text in ["影像服务", "", "", "", "", ""] {
        bytes.extend(bookmark_string_10_1(text));
    }
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend([0; 32]); // 空工作空间和内部名称。
    bytes.extend(bookmark_string_10_1("v2"));
    bytes.extend(bookmark_string_10_1("v3"));
    bytes.extend(IDENTITY_XFORM_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::ImageService(name) = *body else {
        panic!("应读取影像服务栅格名称");
    };
    assert_eq!(name.version, 4);
    assert_eq!(name.long_44, 7);
    assert_eq!(name.string_54.as_deref(), Some("v3"));
    assert!(name.transform.is_some());
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_direct_image_server_name() {
    use mxd2qgz::layers::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1;
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&IMAGE_SERVER_NAME_CLASS_ID, 17);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("影像名称"));
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::ImageServerReference(name) = *body else {
        panic!("应读取 ImageServerName");
    };
    assert_eq!(name.text, "影像名称");
    assert!(name.new_properties.is_some());
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));
    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_access_and_sde_raster_records() {
    use mxd2qgz::layers::raster_dataset_name::{
        ACCESS_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
        SDE_RASTER_DATASET_NAME_CLASS_ID,
    };

    for class in [
        ACCESS_RASTER_DATASET_NAME_CLASS_ID,
        SDE_RASTER_DATASET_NAME_CLASS_ID,
    ] {
        for records_present in [false, true] {
            let mut bytes = bookmark_header_10_1(&class, 17);
            bytes.extend(2_i32.to_le_bytes());
            for text in ["栅格", "", "", ""] {
                bytes.extend(bookmark_string_10_1(text));
            }
            bytes.extend([0; 16]);
            let has_tag7 = records_present && class == ACCESS_RASTER_DATASET_NAME_CLASS_ID;
            if records_present {
                for value in [0x00ff_00fa_i32, 0, 1, 4, 33] {
                    bytes.extend(value.to_le_bytes());
                }
                for tag in [2_i32, 3] {
                    bytes.extend(tag.to_le_bytes());
                    bytes.extend(bookmark_string_10_1(""));
                }
                for value in [4_i32, 4, 12, 5] {
                    bytes.extend(value.to_le_bytes());
                }
                bytes.extend(bookmark_string_10_1(""));
                bytes.extend(6_i32.to_le_bytes());
                bytes.extend((-1_i32).to_le_bytes());
                bytes.extend([0; 16]);
                if has_tag7 {
                    for value in [7_i32, 4, 71] {
                        bytes.extend(value.to_le_bytes());
                    }
                }
                bytes.extend([0xff; 8]);
            }
            bytes.extend(bookmark_string_10_1("书签"));
            let end = bytes.len();
            bytes.push(0xa5);

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
            let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
                panic!("应读取栅格名称");
            };
            let RasterDatasetNameEntryBody10_1::Database(name) = *body else {
                panic!("应读取数据库栅格名称");
            };
            assert_eq!(name.class_id, class);
            assert_eq!(name.records.is_some(), records_present);
            assert_eq!(
                name.records.and_then(|records| records.tag7_long),
                has_tag7.then_some(71)
            );
            assert_eq!(result.name, "书签");
            assert_eq!(reader.position(), end);
            assert!(refs.contains(17));

            for cut in 0..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }
}

#[test]
fn feature_bookmark_reads_tiled_layer_and_ais_image_service_names() {
    use mxd2qgz::layers::raster_dataset_name::{
        AIS_IMAGE_SERVICE_NAME_CLASS_ID, INTERNET_TILED_LAYER_NAME_CLASS_ID,
        RasterDatasetNameEntryBody10_1,
    };

    for class in [
        INTERNET_TILED_LAYER_NAME_CLASS_ID,
        AIS_IMAGE_SERVICE_NAME_CLASS_ID,
    ] {
        let mut bytes = bookmark_header_10_1(&class, 17);
        if class == INTERNET_TILED_LAYER_NAME_CLASS_ID {
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(bookmark_string_10_1("瓦片图层"));
            bytes.extend(9_i32.to_le_bytes());
            bytes.extend([0; 16]); // 空 ITileReader。
        } else {
            bytes.extend(1_i32.to_le_bytes());
            for value in ["影像服务", "名称", ""] {
                bytes.extend(bookmark_string_10_1(value));
            }
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
            panic!("应读取栅格名称");
        };
        match (class, *body) {
            (
                INTERNET_TILED_LAYER_NAME_CLASS_ID,
                RasterDatasetNameEntryBody10_1::InternetTiledLayer(name),
            ) => {
                assert_eq!(name.name_string, "瓦片图层");
                assert_eq!(name.integer, 9);
            }
            (
                AIS_IMAGE_SERVICE_NAME_CLASS_ID,
                RasterDatasetNameEntryBody10_1::AisImageService(name),
            ) => {
                assert_eq!(name.strings, ["影像服务", "名称", ""]);
            }
            _ => panic!("书签名称类型与对象头不符"),
        }
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17));
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_wcs_coverage_and_connection_keys() {
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1;
    use mxd2qgz::layers::wcs_name::{WCS_CONNECTION_NAME_CLASS_ID, WCS_COVERAGE_NAME_CLASS_ID};
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&WCS_COVERAGE_NAME_CLASS_ID, 17);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend([1, 0, 0, 0, b'c', 0, 0, 0]);
    bytes.extend(WCS_CONNECTION_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::WcsCoverage(name) = *body else {
        panic!("应读取 WCS 覆盖名称");
    };
    assert_eq!(name.identifier, "c");
    let connection = name.connection.body.unwrap();
    assert_eq!(connection.version, 3);
    assert!(matches!(
        connection.extra_properties.unwrap().entry,
        mxd2qgz::storage::object_refs::ObjectEntry10_1::BackReference { key: 19, .. }
    ));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!([17, 18, 19].into_iter().all(|key| refs.contains(key)));
    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_preserves_ogc_connection_types() {
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1;
    use mxd2qgz::layers::wcs_name::{
        WCS_CONNECTION_NAME_CLASS_ID, WMS_CONNECTION_NAME_CLASS_ID, WMTS_CONNECTION_NAME_CLASS_ID,
    };
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for class in [
        WCS_CONNECTION_NAME_CLASS_ID,
        WMS_CONNECTION_NAME_CLASS_ID,
        WMTS_CONNECTION_NAME_CLASS_ID,
    ] {
        let mut bytes = bookmark_header_10_1(&class, 17);
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
            panic!("应读取 OGC 名称");
        };
        let RasterDatasetNameEntryBody10_1::OgcConnection(name) = *body else {
            panic!("应读取 OGC 连接");
        };
        assert_eq!(name.class_id, class);
        assert_eq!(name.version, 3);
        assert!(matches!(
            name.extra_properties.unwrap().entry,
            ObjectEntry10_1::BackReference { key: 18, .. }
        ));
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17) && refs.contains(18));
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_raster_type_name_with_mosaic_dataset() {
    use mxd2qgz::layers::raster_dataset_name::{
        MOSAIC_DATASET_NAME_CLASS_ID, RASTER_TYPE_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    };

    let mut bytes = bookmark_header_10_1(&RASTER_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("名称字符串"));
    bytes.extend(bookmark_string_10_1("栅格类型"));
    bytes.extend((-7_i32).to_le_bytes());
    bytes.extend(MOSAIC_DATASET_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("镶嵌"));
    bytes.extend(bookmark_string_10_1("数据集"));
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RasterDatasetName(body) = result.name_object else {
        panic!("应读取栅格名称");
    };
    let RasterDatasetNameEntryBody10_1::RasterType(name) = *body else {
        panic!("应读取栅格类型名称");
    };
    assert_eq!(name.name_string, "名称字符串");
    assert_eq!(name.name, "栅格类型");
    assert_eq!(name.instance_id, -7);
    assert_eq!(
        name.mosaic_dataset.body.unwrap().strings,
        ["镶嵌", "数据集"]
    );
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));
    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_network_dataset_name() {
    use mxd2qgz::layers::na_layer::NETWORK_DATASET_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&NETWORK_DATASET_NAME_CLASS_ID, 17);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("路网"));
    bytes.extend(bookmark_string_10_1("数据集"));
    bytes.extend([0; 32]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::NetworkDatasetName(name) = result.name_object else {
        panic!("应读取网络数据集名称");
    };
    assert_eq!(name.version, 1);
    assert_eq!(name.name, "路网");
    assert_eq!(name.second_string, "数据集");
    assert!(name.dataset.1.is_none() && name.workspace.1.is_none());
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_tin_and_terrain_names() {
    use mxd2qgz::layers::scene_layer_properties::TIN_NAME_CLASS_ID;
    use mxd2qgz::layers::terrain_layer::TERRAIN_NAME_CLASS_ID;

    for class in [TIN_NAME_CLASS_ID, TERRAIN_NAME_CLASS_ID] {
        let mut bytes = bookmark_header_10_1(&class, 17);
        bytes.extend(1_u16.to_le_bytes());
        for value in ["地表", "数据", "名称"] {
            bytes.extend(bookmark_string_10_1(value));
        }
        bytes.extend([0; 16]);
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let strings = match (class, result.name_object) {
            (TIN_NAME_CLASS_ID, BookmarkNameEntry10_1::TinName(name)) => name.strings,
            (TERRAIN_NAME_CLASS_ID, BookmarkNameEntry10_1::TerrainName(name)) => name.strings,
            _ => panic!("应保留地表名称的原始类型"),
        };
        assert_eq!(strings, ["地表", "数据", "名称"]);
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17));

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_las_dataset_name_and_workspace() {
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetWorkspaceSlot10_1;
    use mxd2qgz::maps::map_surround::LAS_DATASET_NAME_CLASS_ID;
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&LAS_DATASET_NAME_CLASS_ID, 17);
    let version_offset = bytes.len();
    bytes.extend((-1_i16).to_le_bytes());
    for value in ["点云", "路径", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for value in ["工作空间", "", ""] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend([0; 16]);
    bytes.push(0);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::LasDatasetName(name) = result.name_object else {
        panic!("应读取 LAS 数据集名称");
    };
    assert_eq!(name.version, -1);
    assert_eq!(name.strings, ["点云", "路径", "名称"]);
    assert!(matches!(
        name.workspace,
        RasterDatasetWorkspaceSlot10_1::WorkspaceName(workspace)
            if workspace.path == "工作空间" && workspace.key == 18
    ));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    bytes[version_offset..version_offset + 2].copy_from_slice(&2_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_cadastral_fabric_name() {
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetWorkspaceSlot10_1;
    use mxd2qgz::maps::map_surround::CADASTRAL_FABRIC_NAME_CLASS_ID;
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&CADASTRAL_FABRIC_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("地籍"));
    bytes.extend(bookmark_string_10_1("名称"));
    let dataset_offset = bytes.len();
    bytes.extend([0; 16]);
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for value in ["工作空间", "", ""] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend([0; 16]);
    bytes.push(0);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::CadastralFabricName(name) = result.name_object else {
        panic!("应读取地籍数据集名称");
    };
    assert_eq!(name.version, 1);
    assert_eq!(name.strings, ["地籍", "名称"]);
    assert!(matches!(name.dataset, DatasetNameMoreSlot10_1::Null));
    assert!(matches!(
        name.workspace,
        RasterDatasetWorkspaceSlot10_1::WorkspaceName(workspace)
            if workspace.path == "工作空间" && workspace.key == 18
    ));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    bytes[dataset_offset..dataset_offset + 16].fill(0xaa);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_ags_server_names() {
    use mxd2qgz::layers::cache_tile_reader::{AGS_SERVER_CONNECTION_NAME, AGS_SERVER_OBJECT_NAME};
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;

    let bstr = [1, 0, 0, 0, b's', 0, 0, 0];
    for class in [AGS_SERVER_CONNECTION_NAME, AGS_SERVER_OBJECT_NAME] {
        let mut bytes = bookmark_header_10_1(&class, 17);
        if class == AGS_SERVER_OBJECT_NAME {
            bytes.extend(3_i16.to_le_bytes());
            bytes.extend(bstr.repeat(3));
            bytes.extend(AGS_SERVER_CONNECTION_NAME);
            bytes.extend(18_u32.to_le_bytes());
        }
        bytes.extend(2_i16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(19_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("连接"));
        if class == AGS_SERVER_OBJECT_NAME {
            bytes.extend(bstr.repeat(3));
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let connection = match (class, result.name_object) {
            (AGS_SERVER_OBJECT_NAME, BookmarkNameEntry10_1::AgsServerObjectName(name)) => {
                assert_eq!(name.strings, ["s", "s", "s"]);
                assert_eq!(name.extra_strings.unwrap(), ["s", "s"]);
                assert_eq!(name.last_string.as_deref(), Some("s"));
                assert!(refs.contains(18));
                name.connection.body.unwrap()
            }
            (AGS_SERVER_CONNECTION_NAME, BookmarkNameEntry10_1::AgsServerConnectionName(name)) => {
                name
            }
            _ => panic!("应保留 AGS 名称的原始类型"),
        };
        assert_eq!(connection.version, 2);
        assert_eq!(connection.extra_string.as_deref(), Some("连接"));
        assert_eq!(connection.properties.body.unwrap().count, 0);
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17) && refs.contains(19));

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn feature_bookmark_reads_file_name_bstr() {
    use mxd2qgz::maps::map_surround::FILE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&FILE_NAME_CLASS_ID, 17);
    let version_offset = bytes.len();
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend(2_u32.to_le_bytes());
    for code in "文件".encode_utf16() {
        bytes.extend(code.to_le_bytes());
    }
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert_eq!(
        result.name_object,
        BookmarkNameEntry10_1::FileName {
            version: -1,
            name_string: "文件".into(),
        }
    );
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    bytes[version_offset..version_offset + 2].copy_from_slice(&2_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(
        read_feature_bookmark_10_1(&mut reader, 1024, &mut refs)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_shortcut_chain_and_rolls_back_invalid_targets() {
    use mxd2qgz::maps::map_surround::{FILE_NAME_CLASS_ID, SHORTCUT_NAME_CLASS_ID};

    let mut bytes = bookmark_header_10_1(&SHORTCUT_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend([1, 0, 0, 0, b'S', 0, 0, 0]);
    let target_offset = bytes.len();
    bytes.extend(FILE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend([1, 0, 0, 0, b'F', 0, 0, 0]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object,
        BookmarkNameEntry10_1::ShortcutName { version: 1, name_string, target }
            if name_string == "S" && *target == (BookmarkNameEntry10_1::FileName {
                version: 1, name_string: "F".into()
            })
    ));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    let mut back_ref = bytes[..target_offset].to_vec();
    back_ref.extend(FILE_NAME_CLASS_ID);
    back_ref.extend(17_u32.to_le_bytes());
    back_ref.extend(bookmark_string_10_1("书签"));
    let mut reader = Reader::new(&back_ref);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object,
        BookmarkNameEntry10_1::ShortcutName { target, .. }
            if matches!(*target, BookmarkNameEntry10_1::BackReference {
                class_id: SHORTCUT_NAME_CLASS_ID, key: 17
            })
    ));

    let mut nested = bytes[..target_offset].to_vec();
    for key in 18_u32..=33 {
        nested.extend(SHORTCUT_NAME_CLASS_ID);
        nested.extend(key.to_le_bytes());
        nested.extend(1_i16.to_le_bytes());
        nested.extend([1, 0, 0, 0, b'S', 0, 0, 0]);
    }
    nested.extend([0; 16]);
    nested.extend(bookmark_string_10_1("书签"));
    let mut reader = Reader::new(&nested);
    let mut refs = RefTable10_1::new();
    let err = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert!(err.to_string().contains("ShortcutName 嵌套层数超限"));
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}
