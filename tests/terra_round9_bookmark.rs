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
fn bookmark_rejects_references_to_objects_without_iname() {
    use mxd2qgz::layers::feature_layer_tail::UID_CLASS_ID;
    use mxd2qgz::maps::map_surround::GP_MULTI_VALUE_CLASS_ID;
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    for class_id in [PROPERTY_SET_CLASS_ID, UID_CLASS_ID, GP_MULTI_VALUE_CLASS_ID] {
        let definition = bookmark_entry_10_1(&class_id, 17);
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();
        let saved = refs.clone();

        let mut body = 1_u16.to_le_bytes().to_vec();
        body.extend(42_i32.to_le_bytes());
        body.extend(bookmark_entry_10_1(&TABLE_NAME_CLASS_ID, 17));
        body.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
        let mut reader = Reader::new(&body);
        let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, saved);
    }
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

#[test]
fn feature_bookmark_reads_representation_class_name() {
    use mxd2qgz::layers::raster_dataset_name::RasterDatasetWorkspaceSlot10_1;
    use mxd2qgz::maps::map_surround::REPRESENTATION_CLASS_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&REPRESENTATION_CLASS_NAME_CLASS_ID, 17);
    bytes.extend((-1_i16).to_le_bytes());
    for value in ["表示类", "要素", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RepresentationClassName(name) = result.name_object else {
        panic!("应读取表示类名称");
    };
    assert_eq!(name.version, -1);
    assert_eq!(name.strings, ["表示类", "要素", "名称"]);
    assert!(matches!(
        name.workspace,
        RasterDatasetWorkspaceSlot10_1::Null
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
fn feature_bookmark_reads_direct_schematic_dataset_name() {
    let mut bytes = bookmark_header_10_1(&SCHEMATIC_DATASET_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    for value in ["示意图", "数据集", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    let workspace_offset = bytes.len();
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object,
        BookmarkNameEntry10_1::SchematicDatasetName(body)
            if body.key == 17 && body.version == 1
                && body.strings == ["示意图", "数据集", "名称"]
                && body.workspace == WorkspaceNameSlot10_1::Null
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

    bytes[workspace_offset..workspace_offset + 16].fill(0xaa);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_direct_route_measure_locator_name() {
    use mxd2qgz::objects::route_event_name::ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(2_i16.to_le_bytes());
    for field in ["路线", "标识", "工作区", "定位器"] {
        bytes.extend(bookmark_string_10_1(field));
    }
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("度量"));
    bytes.extend(7_u16.to_le_bytes());
    bytes.extend((-3_i32).to_le_bytes());
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::RouteMeasureLocatorName(body) = result.name_object else {
        panic!("应读取路线度量定位器名称");
    };
    assert_eq!((body.version, body.base_version), (1, 2));
    assert_eq!(body.strings, ["路线", "标识", "工作区", "定位器"]);
    assert!(matches!(body.route_name, DatasetNameMoreSlot10_1::Null));
    let suffix = body.suffix.unwrap();
    assert_eq!(suffix.string_40, "度量");
    assert_eq!(suffix.legacy_short, Some(7));
    assert_eq!(suffix.long_44, -3);
    assert_eq!(suffix.legacy_long_v2, Some(9));
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
fn feature_bookmark_rejects_nonpersistent_unknown_dataset_name() {
    const UNKNOWN_DATASET_NAME: [u8; 16] = [
        0xb1, 0x3d, 0x78, 0x06, 0xee, 0xe5, 0xd1, 0x11, 0xb0, 0xa2, 0x00, 0x00, 0xf8, 0x78, 0x08,
        0x2e,
    ];
    let mut bytes = bookmark_header_10_1(&UNKNOWN_DATASET_NAME, 17);
    bytes.extend(bookmark_string_10_1("伪后继书签"));
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert!(!refs.register(16));

    let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert!(error.to_string().contains("键 17"));
    assert_eq!(reader.position(), 0);
    assert!(refs.contains(16));
    assert!(!refs.contains(17));
}

#[test]
fn feature_bookmark_reads_schematic_folder_name() {
    use mxd2qgz::objects::dataset_name_more::SCHEMATIC_FOLDER_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&SCHEMATIC_FOLDER_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    for value in ["文件夹", "目录", "名称", "路径"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend(SCHEMATIC_DATASET_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    for value in ["示意图", "数据集", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::SchematicFolderName(body) = result.name_object else {
        panic!("应读取示意图文件夹名称");
    };
    assert_eq!(body.version, 1);
    assert_eq!(body.strings, ["文件夹", "目录", "名称", "路径"]);
    assert!(matches!(
        body.dataset,
        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::SchematicDataset(dataset))
            if dataset.key == 18 && dataset.strings == ["示意图", "数据集", "名称"]
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

#[test]
fn feature_bookmark_reads_schematic_diagram_name() {
    use mxd2qgz::objects::dataset_name_more::SCHEMATIC_DIAGRAM_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&SCHEMATIC_DIAGRAM_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    for value in ["示意图", "图表", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    let class_offset = bytes.len();
    bytes.extend(SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    for value in ["示意图类", "数据", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::SchematicDiagramName(body) = result.name_object else {
        panic!("应读取示意图名称");
    };
    assert_eq!(body.version, 1);
    assert_eq!(body.strings, ["示意图", "图表", "名称"]);
    assert!(matches!(
        body.diagram_class,
        DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::SchematicClass(class))
            if class.key == 18 && class.strings == ["示意图类", "数据", "名称"]
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

    bytes[class_offset..class_offset + 16].copy_from_slice(&SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let err = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_schematic_workspace_name() {
    use mxd2qgz::maps::map_surround::SCHEMATIC_WORKSPACE_NAME_CLASS_ID;
    use mxd2qgz::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&SCHEMATIC_WORKSPACE_NAME_CLASS_ID, 17);
    bytes.extend((-1_i16).to_le_bytes());
    for value in ["示意图", "工作区", "名称"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend(WORKSPACE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for value in ["地理数据库", "", ""] {
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
    let BookmarkNameEntry10_1::SchematicWorkspaceName(body) = result.name_object else {
        panic!("应保留示意图工作空间名称类型");
    };
    assert_eq!(body.key, 17);
    assert_eq!(body.version, -1);
    assert_eq!(body.strings, ["示意图", "工作区", "名称"]);
    assert!(matches!(
        body.workspace,
        WorkspaceNameSlot10_1::WorkspaceName(workspace)
            if workspace.key == 18 && workspace.path == "地理数据库"
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

#[test]
fn feature_bookmark_reads_locator_workspace_name_versions() {
    use mxd2qgz::maps::map_surround::{FILE_NAME_CLASS_ID, LOCATOR_WORKSPACE_NAME_CLASS_ID};

    for version in [-1_i16, 0, 1, 2] {
        let mut bytes = bookmark_header_10_1(&LOCATOR_WORKSPACE_NAME_CLASS_ID, 17);
        bytes.extend(version.to_le_bytes());
        bytes.extend(bookmark_string_10_1("定位器"));
        bytes.extend(bookmark_string_10_1("工作空间"));
        if version == 1 {
            bytes.extend(FILE_NAME_CLASS_ID);
            bytes.extend(18_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend([1, 0, 0, 0, b'F', 0, 0, 0]);
        } else {
            bytes.extend([0; 16]);
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::LocatorWorkspaceName {
            version: actual,
            strings,
            source,
        } = result.name_object
        else {
            panic!("应读取定位器工作空间名称");
        };
        assert_eq!(actual, version);
        assert_eq!(strings, ["定位器", "工作空间"]);
        if version == 1 {
            assert!(
                matches!(*source, BookmarkNameEntry10_1::FileName { name_string, .. } if name_string == "F")
            );
        } else {
            assert_eq!(*source, BookmarkNameEntry10_1::Null);
        }
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17));
        assert_eq!(refs.contains(18), version == 1);

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
fn feature_bookmark_reads_locator_workspace_v2_ags_connection_name() {
    use mxd2qgz::layers::cache_tile_reader::AGS_SERVER_CONNECTION_NAME;
    use mxd2qgz::maps::map_surround::LOCATOR_WORKSPACE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&LOCATOR_WORKSPACE_NAME_CLASS_ID, 17);
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("定位器"));
    bytes.extend(bookmark_string_10_1("工作空间"));
    let source_offset = bytes.len();
    bytes.extend(AGS_SERVER_CONNECTION_NAME);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("服务连接"));
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object,
        BookmarkNameEntry10_1::LocatorWorkspaceName { version: 2, source, .. }
            if matches!(source.as_ref(), BookmarkNameEntry10_1::AgsServerConnectionName(connection)
                if connection.extra_string.as_deref() == Some("服务连接"))
    ));
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);
    assert!(refs.contains(17) && refs.contains(18));

    // AGSServerConnection 本身的 CLSID 与可持久化连接名称不同。
    let live_connection_class = [
        0xd0, 0xef, 0x9d, 0xb4, 0xc0, 0x85, 0xf4, 0x46, 0x91, 0x8b, 0x1f, 0xb7, 0xe0, 0x2a, 0x8c,
        0x70,
    ];
    bytes[source_offset..source_offset + 16].copy_from_slice(&live_connection_class);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn locator_workspace_reads_uid_in_iunknown_slot() {
    use mxd2qgz::layers::feature_layer_tail::UID_CLASS_ID;
    use mxd2qgz::maps::map_surround::LOCATOR_WORKSPACE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&LOCATOR_WORKSPACE_NAME_CLASS_ID, 17);
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("定位器"));
    bytes.extend(bookmark_string_10_1("工作空间"));
    bytes.extend(bookmark_entry_10_1(&UID_CLASS_ID, 18));
    bytes.extend(1_u16.to_le_bytes());
    bytes.push(0);
    bytes.extend([0; 16]);
    bytes.extend(29_i32.to_le_bytes());
    let uid_end = bytes.len();
    bytes.extend(bookmark_string_10_1("书签"));

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        bookmark.name_object,
        BookmarkNameEntry10_1::LocatorWorkspaceName { source, .. }
            if matches!(source.as_ref(), BookmarkNameEntry10_1::LocatorWorkspaceUid(uid)
                if uid.key == 18 && uid.value_18 == 29)
    ));
    assert_eq!(bookmark.name, "书签");
    assert_eq!(reader.position(), bytes.len());

    let mut reader = Reader::new(&bytes[..uid_end - 1]);
    let mut refs = RefTable10_1::new();
    assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn locator_workspace_reads_property_set_in_iunknown_slot() {
    use mxd2qgz::maps::map_surround::LOCATOR_WORKSPACE_NAME_CLASS_ID;
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&LOCATOR_WORKSPACE_NAME_CLASS_ID, 17);
    let version_offset = bytes.len();
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("定位器"));
    bytes.extend(bookmark_string_10_1("工作空间"));
    let source_offset = bytes.len();
    bytes.extend(bookmark_entry_10_1(&PROPERTY_SET_CLASS_ID, 18));
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let bookmark = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        bookmark.name_object,
        BookmarkNameEntry10_1::LocatorWorkspaceName { source, .. }
            if matches!(source.as_ref(), BookmarkNameEntry10_1::LocatorWorkspaceProperties(_))
    ));
    assert_eq!(bookmark.name, "书签");
    assert_eq!(reader.position(), bytes.len());
    assert!(refs.contains(18));

    let mut reference = bytes.clone();
    reference[source_offset..source_offset + 16].copy_from_slice(&TABLE_NAME_CLASS_ID);
    reference.drain(source_offset + 20..source_offset + 26);
    let mut prior = RefTable10_1::new();
    let definition = bookmark_entry_10_1(&PROPERTY_SET_CLASS_ID, 18);
    mxd2qgz::storage::object_refs::read_object_entry_10_1(
        &mut Reader::new(&definition),
        &mut prior,
    )
    .unwrap();
    for version in [1_i16, 2] {
        reference[version_offset..version_offset + 2].copy_from_slice(&version.to_le_bytes());
        let mut reader = Reader::new(&reference);
        let mut refs = prior.clone();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs);
        if version == 2 {
            assert!(matches!(
                result.unwrap().name_object,
                BookmarkNameEntry10_1::LocatorWorkspaceName { source, .. }
                    if matches!(*source, BookmarkNameEntry10_1::BackReference {
                        class_id: PROPERTY_SET_CLASS_ID, key: 18,
                    })
            ));
            assert_eq!(reader.position(), reference.len());
        } else {
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, prior);
        }
    }

    bytes[version_offset..version_offset + 2].copy_from_slice(&1_i16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_locator_name_with_workspace() {
    use mxd2qgz::maps::map_surround::{LOCATOR_NAME_CLASS_ID, LOCATOR_WORKSPACE_NAME_CLASS_ID};

    let mut bytes = bookmark_header_10_1(&LOCATOR_NAME_CLASS_ID, 17);
    let version_offset = bytes.len();
    bytes.extend((-1_i16).to_le_bytes());
    for value in ["定位器", "名称", "路径", "说明"] {
        bytes.extend(bookmark_string_10_1(value));
    }
    bytes.extend(LOCATOR_WORKSPACE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("定位器"));
    bytes.extend(bookmark_string_10_1("工作区"));
    bytes.extend([0; 16]);
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::LocatorName {
        version,
        strings,
        source,
    } = result.name_object
    else {
        panic!("应读取定位器名称");
    };
    assert_eq!(version, -1);
    assert_eq!(strings, ["定位器", "名称", "路径", "说明"]);
    assert!(matches!(
        *source,
        BookmarkNameEntry10_1::LocatorWorkspaceName { version: 1, strings, source }
            if strings == ["定位器", "工作区"] && *source == BookmarkNameEntry10_1::Null
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
fn feature_bookmark_reads_gp_data_type_name_versioned_uid_slots() {
    use mxd2qgz::layers::feature_layer_tail::UID_CLASS_ID;
    use mxd2qgz::maps::map_surround::GP_DATA_TYPE_NAME_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for (version, base_version) in [(1_i16, 1_i16), (2, 2), (2, 3), (1, 3)] {
        let mut bytes = bookmark_header_10_1(&GP_DATA_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(version.to_le_bytes());
        bytes.extend(base_version.to_le_bytes());
        for value in [b'a', b'b', b'c', b'd'] {
            bytes.extend([1, 0, 0, 0, value, 0, 0, 0]);
        }
        if base_version < 2 {
            bytes.extend([1, 0, 0, 0, b'e', 0, 0, 0]);
        }
        let mut uid_is_new = true;
        for present in [base_version == 3, version < 2] {
            if present {
                bytes.extend(UID_CLASS_ID);
                bytes.extend(18_u32.to_le_bytes());
                if uid_is_new {
                    bytes.extend(1_u16.to_le_bytes());
                    bytes.extend([0; 17]);
                    bytes.extend(9_i32.to_le_bytes());
                    uid_is_new = false;
                }
            }
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpDataTypeName(body) = result.name_object else {
            panic!("应读取 GPDataTypeName");
        };
        assert_eq!((body.version, body.base_version), (version, base_version));
        assert_eq!(body.strings, ["a", "b", "c", "d"]);
        assert_eq!(
            body.legacy_string.as_deref(),
            (base_version < 2).then_some("e")
        );
        assert_eq!(body.base_uid.is_some(), base_version == 3);
        assert_eq!(body.legacy_uid.is_some(), version < 2);
        if let Some(uid) = body.base_uid.as_ref().or(body.legacy_uid.as_ref()) {
            assert_eq!(uid.body.as_ref().unwrap().value_18, 9);
        }
        if version < 2 && base_version == 3 {
            assert!(matches!(
                body.legacy_uid.unwrap().entry,
                ObjectEntry10_1::BackReference { key: 18, .. }
            ));
        }
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17));
        assert_eq!(refs.contains(18), !uid_is_new);

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
fn feature_bookmark_reads_gp_multi_value_type_name() {
    use mxd2qgz::maps::map_surround::{
        FILE_NAME_CLASS_ID, GP_DATA_TYPE_NAME_CLASS_ID, GP_MULTI_VALUE_TYPE_NAME_CLASS_ID,
    };

    let mut bytes = bookmark_header_10_1(&GP_MULTI_VALUE_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    let child_offset = bytes.len();
    bytes.extend(GP_DATA_TYPE_NAME_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend([1, 0, 0, 0, b'g', 0, 0, 0].repeat(4));
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    assert!(matches!(
        result.name_object,
        BookmarkNameEntry10_1::GpMultiValueTypeName { version: 1, element_type }
            if matches!(element_type.as_ref(), BookmarkNameEntry10_1::GpDataTypeName(body)
                if body.strings == ["g", "g", "g", "g"])
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

    let mut wrong_class = bytes[..child_offset].to_vec();
    wrong_class.extend(FILE_NAME_CLASS_ID);
    wrong_class.extend(18_u32.to_le_bytes());
    wrong_class.extend(1_i16.to_le_bytes());
    wrong_class.extend([1, 0, 0, 0, b'f', 0, 0, 0]);
    wrong_class.extend(bookmark_string_10_1("书签"));
    let mut reader = Reader::new(&wrong_class);
    let mut refs = RefTable10_1::new();
    let err = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_gp_function_name_versions_and_nested_slot() {
    use mxd2qgz::maps::map_surround::{
        GP_FUNCTION_NAME_CLASS_ID, GP_MULTI_VALUE_TYPE_NAME_CLASS_ID,
    };

    for version in [1_i16, 2, 3, 4] {
        let mut bytes = if version == 4 {
            let mut nested = bookmark_header_10_1(&GP_MULTI_VALUE_TYPE_NAME_CLASS_ID, 17);
            nested.extend(1_i16.to_le_bytes());
            nested.extend(GP_FUNCTION_NAME_CLASS_ID);
            nested.extend(18_u32.to_le_bytes());
            nested
        } else {
            bookmark_header_10_1(&GP_FUNCTION_NAME_CLASS_ID, 17)
        };
        bytes.extend(version.to_le_bytes());
        bytes.extend(2_i16.to_le_bytes());
        bytes.extend([1, 0, 0, 0, b'g', 0, 0, 0].repeat(4));
        if version < 2 {
            bytes.extend([0; 16]);
        }
        if version >= 3 {
            bytes.extend(123_u32.to_le_bytes());
            bytes.extend([1, 0, 0, 0, b't', 0, 0, 0]);
            bytes.extend((-7_i32).to_le_bytes());
        }
        if version >= 4 {
            bytes.extend(42_u32.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let name = match result.name_object {
            BookmarkNameEntry10_1::GpMultiValueTypeName { element_type, .. } => {
                assert_eq!(version, 4);
                assert!(refs.contains(18));
                *element_type
            }
            direct => direct,
        };
        let BookmarkNameEntry10_1::GpFunctionName(body) = name else {
            panic!("应读取 GPFunctionName");
        };
        assert_eq!(body.version, version);
        assert_eq!(body.base.strings, ["g", "g", "g", "g"]);
        assert_eq!(body.legacy_uid.is_some(), version < 2);
        assert_eq!(
            body.v3_fields,
            (version >= 3).then_some((123, "t".into(), -7))
        );
        assert_eq!(body.v4_value, (version >= 4).then_some(42));
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
fn feature_bookmark_reads_gp_sa_topo_features_type_name() {
    use mxd2qgz::maps::map_surround::{
        GP_MULTI_VALUE_TYPE_NAME_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
    };

    for version in [2_i16, 3, 4] {
        let mut bytes = bookmark_header_10_1(&GP_MULTI_VALUE_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        if version < 4 {
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend(bookmark_string_10_1("拓扑"));
            bytes.extend((-9_i32).to_le_bytes());
            if version == 3 {
                bytes.extend([0; 16]);
            }
        }
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpMultiValueTypeName { element_type, .. } = result.name_object
        else {
            panic!("应读取多值类型名称");
        };
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = *element_type else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(body.version, version);
        assert_eq!(body.legacy_count, (version < 4).then_some(1));
        assert_eq!(body.legacy_entries.len(), usize::from(version < 4));
        if let Some(entry) = body.legacy_entries.first() {
            assert_eq!(entry.data_type, BookmarkNameEntry10_1::Null);
            assert_eq!(entry.string, "拓扑");
            assert_eq!(entry.value, -9);
            assert_eq!(entry.uid.is_some(), version == 3);
        }
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
fn feature_bookmark_reads_legacy_topo_numeric_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_NUMERIC_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(3_i32.to_le_bytes());
    for index in 0..3 {
        bytes.extend([0; 16]);
        if index == 2 {
            bytes.extend([0; 16]);
        } else {
            bytes.extend(GP_NUMERIC_DOMAIN_CLASS_ID);
            bytes.extend(18_u32.to_le_bytes());
            if index == 0 {
                bytes.extend(1_i16.to_le_bytes());
                bytes.extend(2_u16.to_le_bytes());
                bytes.extend((-0.0_f64).to_le_bytes());
                bytes.extend(3_u16.to_le_bytes());
                bytes.extend(4_u16.to_le_bytes());
                bytes.extend(8.5_f64.to_le_bytes());
            }
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend((-9_i32).to_le_bytes());
    }
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let [first, second, third] = body.legacy_entries.as_slice() else {
        panic!("应读取三条旧版记录");
    };
    let GpTopoDomain10_1::Numeric(domain) = first
        .legacy_domain
        .as_ref()
        .unwrap()
        .body
        .as_deref()
        .unwrap()
    else {
        panic!("应读取数值域");
    };
    assert_eq!(domain.version, 1);
    assert_eq!(domain.first_short, 2);
    assert_eq!(domain.first_double_bits, (-0.0_f64).to_bits());
    assert_eq!(domain.middle_shorts, [3, 4]);
    assert_eq!(domain.second_double_bits, 8.5_f64.to_bits());
    assert!(matches!(
        second.legacy_domain.as_ref().unwrap().entry,
        ObjectEntry10_1::BackReference { key: 18, .. }
    ));
    assert_eq!(
        third.legacy_domain.as_ref().unwrap().entry,
        ObjectEntry10_1::Null
    );
    assert_eq!(third.string, "拓扑");
    assert_eq!(third.value, -9);
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
fn feature_bookmark_reads_legacy_topo_file_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_FILE_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };

    for raw_count in [-1_i32, 2] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_FILE_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        let count_offset = bytes.len();
        bytes.extend(raw_count.to_le_bytes());
        let strings = if raw_count > 0 {
            vec!["文件", ""]
        } else {
            vec![]
        };
        for value in &strings {
            bytes.extend(bookmark_string_10_1(value));
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        let entry = &body.legacy_entries[0];
        assert!(matches!(
            entry.legacy_domain.as_ref().unwrap().body.as_deref(),
            Some(GpTopoDomain10_1::File { version: 1, raw_count: actual, strings: values })
                if *actual == raw_count && *values == strings
        ));
        assert_eq!(entry.string, "拓扑");
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

        bytes[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_legacy_topo_range_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_RANGE_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };
    use mxd2qgz::stream::variant_stream::{VariantValue10_1, VariantValue10_8};

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(GP_RANGE_DOMAIN_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(5_u32.to_le_bytes());
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend((-7_i32).to_le_bytes());
    bytes.extend(5_u16.to_le_bytes());
    bytes.extend(3.5_f64.to_le_bytes());
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let entry = &body.legacy_entries[0];
    assert_eq!(
        entry.legacy_domain.as_ref().unwrap().body.as_deref(),
        Some(&GpTopoDomain10_1::Range {
            version: 1,
            flags: 5,
            bounds: [
                VariantValue10_1::Scalar(VariantValue10_8::Raw4((-7_i32).to_le_bytes())),
                VariantValue10_1::Scalar(VariantValue10_8::Raw8(3.5_f64.to_le_bytes())),
            ],
        })
    );
    assert_eq!(entry.string, "拓扑");
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
fn feature_bookmark_reads_legacy_topo_workspace_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GP_WORKSPACE_DOMAIN_CLASS_ID, GpTopoDomain10_1,
    };

    for raw_count in [-1_i32, 2] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_WORKSPACE_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        let count_offset = bytes.len();
        bytes.extend(raw_count.to_le_bytes());
        let values = if raw_count > 0 {
            vec![-5_i32, 7]
        } else {
            vec![]
        };
        for value in &values {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        let entry = &body.legacy_entries[0];
        assert_eq!(
            entry.legacy_domain.as_ref().unwrap().body.as_deref(),
            Some(&GpTopoDomain10_1::Workspace {
                version: 1,
                raw_count,
                values
            })
        );
        assert_eq!(entry.string, "拓扑");
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

        bytes[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_dataset_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_DATASET_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };

    for raw_count in [-1_i32, 2] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_DATASET_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(raw_count.to_le_bytes());
        let values = if raw_count > 0 {
            vec![-5_i32, 7]
        } else {
            vec![]
        };
        for value in &values {
            bytes.extend(value.to_le_bytes());
        }
        let trailing_shorts = [1_u16, 2, 0xffff, 4];
        for value in trailing_shorts {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        let entry = &body.legacy_entries[0];
        assert_eq!(
            entry.legacy_domain.as_ref().unwrap().body.as_deref(),
            Some(&GpTopoDomain10_1::Dataset {
                version: 1,
                raw_count,
                values,
                trailing_shorts
            })
        );
        assert_eq!(entry.string, "拓扑");
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
fn feature_bookmark_reads_legacy_topo_feature_class_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_FEATURE_CLASS_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };

    for version in [1_i16, 2, 3] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_FEATURE_CLASS_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend((-5_i32).to_le_bytes());
        for value in [1_u16, 2] {
            bytes.extend(value.to_le_bytes());
        }
        if version >= 2 {
            for value in [3_u16, 4, 5, 6] {
                bytes.extend(value.to_le_bytes());
            }
        }
        if version >= 3 {
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(
            body.legacy_entries[0]
                .legacy_domain
                .as_ref()
                .unwrap()
                .body
                .as_deref(),
            Some(&GpTopoDomain10_1::FeatureClass {
                version,
                raw_count: 1,
                values: vec![-5],
                shorts: [1, 2],
                v2_shorts: (version >= 2).then_some([3, 4, 5, 6]),
                v3_list: (version >= 3).then_some((1, vec![7])),
            }),
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_arc_info_item_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_ARC_INFO_ITEM_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };

    for raw_count in [-2_i32, 1] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_ARC_INFO_ITEM_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(raw_count.to_le_bytes());
        let values = if raw_count > 0 { vec![-9_i32] } else { vec![] };
        for value in &values {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend((-7_i32).to_le_bytes());
        for value in [1_u16, 2, 3, 4] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(
            body.legacy_entries[0]
                .legacy_domain
                .as_ref()
                .unwrap()
                .body
                .as_deref(),
            Some(&GpTopoDomain10_1::ArcInfoItem {
                version: 1,
                raw_count,
                values,
                trailing_long: -7,
                trailing_shorts: [1, 2, 3, 4],
            })
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_field_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_FIELD_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpFieldDomain10_1,
        GpTopoDomain10_1,
    };

    for version in 1_i16..=4 {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_FIELD_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend((-5_i32).to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        if version >= 2 {
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(bookmark_string_10_1("字段"));
        }
        if version >= 3 {
            bytes.extend((-1_i32).to_le_bytes());
        }
        if version >= 4 {
            bytes.extend(0xffff_u16.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(
            body.legacy_entries[0]
                .legacy_domain
                .as_ref()
                .unwrap()
                .body
                .as_deref(),
            Some(&GpTopoDomain10_1::Field(GpFieldDomain10_1 {
                version,
                raw_count: 1,
                values: vec![-5],
                shorts: [1, 2],
                v2_strings: (version >= 2).then(|| (1, vec!["字段".into()])),
                v3_strings: (version >= 3).then(|| (-1, Vec::new())),
                v4_short: (version >= 4).then_some(0xffff),
            })),
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_measure_fields_domain() {
    use mxd2qgz::maps::map_surround::{GP_MEASURE_FIELDS_DOMAIN_CLASS_ID, GpTopoDomain10_1};

    assert_legacy_topo_version_only_domain(
        GP_MEASURE_FIELDS_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::MeasureFields { version: -1 },
    );
}

#[test]
fn feature_bookmark_reads_legacy_topo_route_id_fields_domain() {
    use mxd2qgz::maps::map_surround::{GP_ROUTE_ID_FIELDS_DOMAIN_CLASS_ID, GpTopoDomain10_1};

    assert_legacy_topo_version_only_domain(
        GP_ROUTE_ID_FIELDS_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::RouteIdFields { version: -1 },
    );
}

fn assert_legacy_topo_version_only_domain(
    class_id: [u8; 16],
    expected: mxd2qgz::maps::map_surround::GpTopoDomain10_1,
) {
    assert_legacy_topo_version_only_domain_with_max(class_id, expected, 1);
}

fn assert_legacy_topo_version_only_domain_with_max(
    class_id: [u8; 16],
    expected: mxd2qgz::maps::map_surround::GpTopoDomain10_1,
    max_version: i16,
) {
    use mxd2qgz::maps::map_surround::GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID;

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(class_id);
    bytes.extend(18_u32.to_le_bytes());
    let version_offset = bytes.len();
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let entry = &body.legacy_entries[0];
    assert_eq!(
        entry.legacy_domain.as_ref().unwrap().body.as_deref(),
        Some(&expected),
    );
    assert_eq!(entry.string, "拓扑");
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
    bytes[version_offset..version_offset + 2].copy_from_slice(&(max_version + 1).to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_legacy_topo_query_table_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_QUERY_TABLE_FIELDS_DOMAIN_CLASS_ID, GP_QUERY_TABLE_IDS_DOMAIN_CLASS_ID, GpTopoDomain10_1,
    };

    assert_legacy_topo_version_only_domain(
        GP_QUERY_TABLE_IDS_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::QueryTableIds { version: -1 },
    );
    assert_legacy_topo_version_only_domain(
        GP_QUERY_TABLE_FIELDS_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::QueryTableFields { version: -1 },
    );
}

#[test]
fn feature_bookmark_reads_legacy_topo_virtual_table_domain() {
    use mxd2qgz::maps::map_surround::{GP_VIRTUAL_TABLE_DOMAIN_CLASS_ID, GpTopoDomain10_1};

    assert_legacy_topo_version_only_domain(
        GP_VIRTUAL_TABLE_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::VirtualTable { version: -1 },
    );
}

#[test]
fn feature_bookmark_reads_legacy_topo_route_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_ROUTE_DOMAIN_CLASS_ID, GP_ROUTE_MEASURE_EVENT_DOMAIN_CLASS_ID, GpTopoDomain10_1,
    };

    assert_legacy_topo_version_only_domain(
        GP_ROUTE_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::Route { version: -1 },
    );
    assert_legacy_topo_version_only_domain(
        GP_ROUTE_MEASURE_EVENT_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::RouteMeasureEvent { version: -1 },
    );
}

#[test]
fn feature_bookmark_reads_legacy_topo_layers_and_tables_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_LAYERS_AND_TABLES_DOMAIN_CLASS_ID, GP_TABLES_DOMAIN_CLASS_ID, GpTopoDomain10_1,
    };

    assert_legacy_topo_version_only_domain(
        GP_TABLES_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::Tables { version: -1 },
    );
    assert_legacy_topo_version_only_domain(
        GP_LAYERS_AND_TABLES_DOMAIN_CLASS_ID,
        GpTopoDomain10_1::LayersAndTables { version: -1 },
    );
}

#[test]
fn feature_bookmark_reads_legacy_topo_sa_geo_data_domain() {
    use mxd2qgz::maps::map_surround::{
        FILE_NAME_CLASS_ID, GP_SA_GEO_DATA_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
        GpTopoDomain10_1,
    };

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(GP_SA_GEO_DATA_DOMAIN_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(FILE_NAME_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend([1, 0, 0, 0, b'f', 0, 0, 0]);
    bytes.extend(FILE_NAME_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend((-8_i32).to_le_bytes());
    bytes.extend((-2_i32).to_le_bytes());
    bytes.extend(7_u16.to_le_bytes());
    bytes.extend(bookmark_string_10_1("域"));
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let Some(GpTopoDomain10_1::SaGeoData(domain)) = body.legacy_entries[0]
        .legacy_domain
        .as_ref()
        .unwrap()
        .body
        .as_deref()
    else {
        panic!("应读取地理数据域");
    };
    assert_eq!(domain.version, 1);
    assert_eq!(domain.raw_name_count, 2);
    assert!(matches!(&domain.names[..], [
        BookmarkNameEntry10_1::FileName { name_string, .. },
        BookmarkNameEntry10_1::BackReference { class_id: FILE_NAME_CLASS_ID, key: 19 },
    ] if name_string == "f"));
    assert_eq!(domain.first_list, (1, vec![-8]));
    assert_eq!(domain.second_list, (-2, vec![]));
    assert_eq!(domain.trailing_short, 7);
    assert_eq!(domain.trailing_string, "域");
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
fn feature_bookmark_reads_legacy_topo_spatial_analyst_name_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_SA_NAME_DOMAIN_CLASS_ID, GP_SA_TEXT_TABLE_NAME_DOMAIN_CLASS_ID,
        GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };

    for (class_id, expected, payload) in [
        (
            GP_SA_NAME_DOMAIN_CLASS_ID,
            GpTopoDomain10_1::SaName {
                version: -1,
                long: -5,
                short: 7,
            },
            [(-5_i32).to_le_bytes().as_slice(), &7_u16.to_le_bytes()].concat(),
        ),
        (
            GP_SA_TEXT_TABLE_NAME_DOMAIN_CLASS_ID,
            GpTopoDomain10_1::SaTextTableName {
                version: -1,
                string: "文本表".into(),
            },
            bookmark_string_10_1("文本表"),
        ),
    ] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(class_id);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend((-1_i16).to_le_bytes());
        bytes.extend(payload);
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(
            body.legacy_entries[0]
                .legacy_domain
                .as_ref()
                .unwrap()
                .body
                .as_deref(),
            Some(&expected)
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_sa_remap_domain() {
    use mxd2qgz::maps::map_surround::{GP_SA_REMAP_DOMAIN_CLASS_ID, GpTopoDomain10_1};
    assert_legacy_topo_long_list_domain(GP_SA_REMAP_DOMAIN_CLASS_ID, |raw_count, values| {
        GpTopoDomain10_1::SaRemap {
            version: 1,
            raw_count,
            values,
        }
    });
}

#[test]
fn feature_bookmark_reads_legacy_topo_sa_neighborhood_domain() {
    use mxd2qgz::maps::map_surround::{GP_SA_NEIGHBORHOOD_DOMAIN_CLASS_ID, GpTopoDomain10_1};
    assert_legacy_topo_long_list_domain(GP_SA_NEIGHBORHOOD_DOMAIN_CLASS_ID, |raw_count, values| {
        GpTopoDomain10_1::SaNeighborhood {
            version: 1,
            raw_count,
            values,
        }
    });
}

fn assert_legacy_topo_long_list_domain(
    class_id: [u8; 16],
    expected: impl Fn(i32, Vec<i32>) -> mxd2qgz::maps::map_surround::GpTopoDomain10_1,
) {
    use mxd2qgz::maps::map_surround::GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID;

    for count in [-2_i32, 2] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(class_id);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(count.to_le_bytes());
        let values = if count > 0 { vec![-7_i32, 9] } else { vec![] };
        for value in &values {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(
            body.legacy_entries[0]
                .legacy_domain
                .as_ref()
                .unwrap()
                .body
                .as_deref(),
            Some(&expected(count, values)),
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_ra_field_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_RA_FIELD_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpRaFieldDomain10_1,
        GpTopoDomain10_1,
    };

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(GP_RA_FIELD_DOMAIN_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend((-7_i32).to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("字段"));
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let entry = &body.legacy_entries[0];
    assert_eq!(
        entry.legacy_domain.as_ref().unwrap().body.as_deref(),
        Some(&GpTopoDomain10_1::RaField(GpRaFieldDomain10_1 {
            version: 1,
            long_list: (1, vec![-7]),
            shorts: [2, 3],
            string_lists: [(-1, vec![]), (1, vec!["字段".into()])],
        }))
    );
    assert_eq!(entry.string, "拓扑");
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
fn feature_bookmark_reads_legacy_topo_representation_layer_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_REPRESENTATION_LAYER_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
        GpTopoDomain10_1,
    };

    for (version, count) in [(1_i16, 0_i32), (2, -1), (2, 2)] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_REPRESENTATION_LAYER_DOMAIN_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        let values = if count > 0 { vec![-7_i32, 9] } else { vec![] };
        if version == 2 {
            bytes.extend(count.to_le_bytes());
            for value in &values {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        assert_eq!(
            body.legacy_entries[0]
                .legacy_domain
                .as_ref()
                .unwrap()
                .body
                .as_deref(),
            Some(&GpTopoDomain10_1::RepresentationLayer {
                version,
                v2_list: (version == 2).then_some((count, values)),
            })
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
}

#[test]
fn feature_bookmark_reads_legacy_topo_annotation_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_CAD_ANNO_DOMAIN_CLASS_ID, GP_COV_ANNO_DOMAIN_CLASS_ID, GP_GDB_ANNO_DOMAIN_CLASS_ID,
        GpTopoDomain10_1,
    };

    for (class_id, expected) in [
        (
            GP_GDB_ANNO_DOMAIN_CLASS_ID,
            GpTopoDomain10_1::GdbAnnotation { version: -1 },
        ),
        (
            GP_COV_ANNO_DOMAIN_CLASS_ID,
            GpTopoDomain10_1::CoverageAnnotation { version: -1 },
        ),
        (
            GP_CAD_ANNO_DOMAIN_CLASS_ID,
            GpTopoDomain10_1::CadAnnotation { version: -1 },
        ),
    ] {
        assert_legacy_topo_version_only_domain_with_max(class_id, expected, 0);
    }
}

#[test]
fn feature_bookmark_reads_legacy_topo_coverage_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_COVERAGE_DOMAIN_CLASS_ID, GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID,
        GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpCoverageFeatureClassDomain10_1, GpTopoDomain10_1,
    };

    for nested in [false, true] {
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        if nested {
            bytes.extend(GP_COVERAGE_DOMAIN_CLASS_ID);
            bytes.extend(18_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
        }
        bytes.extend(GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID);
        bytes.extend((if nested { 19_u32 } else { 18 }).to_le_bytes());
        let version = if nested { 2_i16 } else { 1 };
        bytes.extend(version.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend((-7_i32).to_le_bytes());
        if nested {
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(9_i32.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        let domain = body.legacy_entries[0]
            .legacy_domain
            .as_ref()
            .unwrap()
            .body
            .as_deref();
        let feature_class = match domain {
            Some(GpTopoDomain10_1::Coverage {
                version: 1,
                feature_class,
            }) if nested => feature_class.body.as_deref().unwrap(),
            Some(GpTopoDomain10_1::CoverageFeatureClass(body)) if !nested => body,
            _ => panic!("应读取对应的 Coverage 域"),
        };
        assert_eq!(
            *feature_class,
            GpCoverageFeatureClassDomain10_1 {
                version,
                long_list: (1, vec![-7]),
                v2_tail: nested.then_some((3, 9)),
            }
        );
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!(refs.contains(17) && refs.contains(18));
        assert_eq!(refs.contains(19), nested);

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
fn feature_bookmark_reads_legacy_topo_composite_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_COMPOSITE_DOMAIN_CLASS_ID, GP_DATA_ELEMENT_DOMAIN_CLASS_ID,
        GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(GP_COMPOSITE_DOMAIN_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(GP_DATA_ELEMENT_DOMAIN_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("数据元素"));
    bytes.extend(GP_DATA_ELEMENT_DOMAIN_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let Some(GpTopoDomain10_1::Composite {
        version: 1,
        raw_count: 2,
        domains,
    }) = body.legacy_entries[0]
        .legacy_domain
        .as_ref()
        .unwrap()
        .body
        .as_deref()
    else {
        panic!("应读取组合域");
    };
    let [first, second] = domains.as_slice() else {
        panic!("应读取两个子域");
    };
    assert_eq!(
        first.body.as_deref(),
        Some(&GpTopoDomain10_1::DataElement {
            version: 1,
            raw_count: 1,
            strings: vec!["数据元素".into()],
        })
    );
    assert!(matches!(
        second.entry,
        ObjectEntry10_1::BackReference { key: 19, .. }
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
fn feature_bookmark_reads_legacy_topo_field_info_and_coded_value_domains() {
    use mxd2qgz::maps::map_surround::{
        GP_AREAL_UNIT_CLASS_ID, GP_BOOLEAN_CLASS_ID, GP_DATE_CLASS_ID, GP_DOUBLE_CLASS_ID,
        GP_ENCRYPTED_STRING_CLASS_ID, GP_LINEAR_UNIT_CLASS_ID, GP_MULTI_VALUE_CLASS_ID,
        GP_MULTI_VALUE_TYPE_NAME_CLASS_ID, GP_RANDOM_NUMBER_GENERATOR_CLASS_ID,
        GP_VALUE_TABLE_CLASS_ID, GP_VARIANT_CLASS_ID,
    };
    use mxd2qgz::maps::map_surround::{
        GP_CODED_VALUE_DOMAIN_CLASS_ID, GP_DATA_TYPE_NAME_CLASS_ID, GP_FIELD_INFO_DOMAIN_CLASS_ID,
        GP_FUNCTION_NAME_CLASS_ID, GP_LONG_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
        GP_STRING_CLASS_ID, GP_VALUE_TABLE_TYPE_NAME_CLASS_ID, GpCodedValue10_1, GpTopoDomain10_1,
    };
    use mxd2qgz::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for type_class in [
        None,
        Some(GP_FUNCTION_NAME_CLASS_ID),
        Some(GP_DATA_TYPE_NAME_CLASS_ID),
        Some(GP_MULTI_VALUE_TYPE_NAME_CLASS_ID),
        Some(GP_VALUE_TABLE_TYPE_NAME_CLASS_ID),
        Some(GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID),
    ] {
        let nested = type_class.is_some();
        let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        if nested {
            bytes.extend(GP_FIELD_INFO_DOMAIN_CLASS_ID);
            bytes.extend(18_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
        }
        bytes.extend(GP_CODED_VALUE_DOMAIN_CLASS_ID);
        bytes.extend((if nested { 19_u32 } else { 18 }).to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("标签"));
        bytes.extend(17_i32.to_le_bytes());
        for (class, key) in [(GP_STRING_CLASS_ID, 20_u32), (GP_LONG_CLASS_ID, 21)] {
            bytes.extend(class);
            bytes.extend(key.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            if class == GP_STRING_CLASS_ID {
                bytes.extend(bookmark_string_10_1("值"));
            } else {
                bytes.extend((-7_i32).to_le_bytes());
            }
        }
        bytes.extend(GP_LONG_CLASS_ID);
        bytes.extend(21_u32.to_le_bytes());
        for (class_id, key, payload) in [
            (
                GP_DOUBLE_CLASS_ID,
                22_u32,
                (-0.0_f64).to_le_bytes().to_vec(),
            ),
            (GP_BOOLEAN_CLASS_ID, 23, 0xffff_u16.to_le_bytes().to_vec()),
            (
                GP_DATE_CLASS_ID,
                24,
                [
                    7_u16.to_le_bytes().as_slice(),
                    &[0; 14],
                    &45000.5_f64.to_le_bytes(),
                ]
                .concat(),
            ),
            (
                GP_LINEAR_UNIT_CLASS_ID,
                25,
                [(-2.5_f64).to_le_bytes().as_slice(), &(-1_i32).to_le_bytes()].concat(),
            ),
            (
                GP_VARIANT_CLASS_ID,
                26,
                [
                    3_u16.to_le_bytes().as_slice(),
                    &[0; 14],
                    &42_i32.to_le_bytes(),
                ]
                .concat(),
            ),
            (
                GP_AREAL_UNIT_CLASS_ID,
                27,
                [(3.25_f64).to_le_bytes().as_slice(), &(-2_i32).to_le_bytes()].concat(),
            ),
        ] {
            bytes.extend(class_id);
            bytes.extend(key.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(payload);
        }
        bytes.extend(GP_VARIANT_CLASS_ID);
        bytes.extend(26_u32.to_le_bytes());
        bytes.extend(GP_AREAL_UNIT_CLASS_ID);
        bytes.extend(27_u32.to_le_bytes());
        bytes.extend(GP_ENCRYPTED_STRING_CLASS_ID);
        bytes.extend(28_u32.to_le_bytes());
        let encrypted_version_offset = bytes.len();
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend(bookmark_string_10_1("synthetic-persisted-value"));
        bytes.extend(GP_ENCRYPTED_STRING_CLASS_ID);
        bytes.extend(28_u32.to_le_bytes());
        bytes.extend(GP_RANDOM_NUMBER_GENERATOR_CLASS_ID);
        bytes.extend(29_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend((-79_i32).to_le_bytes());
        bytes.extend((-9_i32).to_le_bytes());
        bytes.extend(GP_RANDOM_NUMBER_GENERATOR_CLASS_ID);
        bytes.extend(29_u32.to_le_bytes());
        bytes.extend(bookmark_entry_10_1(&GP_MULTI_VALUE_CLASS_ID, 30));
        bytes.extend([1, 0, 1, 0, 7, 0]);
        let type_offset = bytes.len();
        if let Some(class_id) = type_class {
            bytes.extend(bookmark_entry_10_1(&class_id, 32));
            if class_id == GP_MULTI_VALUE_TYPE_NAME_CLASS_ID {
                bytes.extend(1_i16.to_le_bytes());
                bytes.extend(bookmark_entry_10_1(&GP_DATA_TYPE_NAME_CLASS_ID, 34));
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend([0; 16]);
            } else if class_id == GP_FUNCTION_NAME_CLASS_ID {
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend([0; 16]);
            } else if class_id == GP_DATA_TYPE_NAME_CLASS_ID {
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend([0; 16]);
            } else if class_id == GP_VALUE_TABLE_TYPE_NAME_CLASS_ID {
                bytes.extend(3_i16.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
            } else {
                bytes.extend(4_i16.to_le_bytes());
            }
        } else {
            bytes.extend([0; 16]);
        }
        let type_end = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(bookmark_entry_10_1(&GP_LONG_CLASS_ID, 21));
        bytes.extend(bookmark_entry_10_1(&GP_MULTI_VALUE_CLASS_ID, 31));
        bytes.extend([1, 0, 1, 0, 7, 0]);
        if let Some(class_id) = type_class {
            bytes.extend(bookmark_entry_10_1(&class_id, 32));
        } else {
            bytes.extend([0; 16]);
        }
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(bookmark_entry_10_1(&GP_VALUE_TABLE_CLASS_ID, 33));
        bytes.extend([1, 0, 1, 0, 7, 0]);
        bytes.extend(2_i32.to_le_bytes());
        if let Some(class_id) = type_class.filter(|id| *id != GP_FUNCTION_NAME_CLASS_ID) {
            bytes.extend(bookmark_entry_10_1(&class_id, 32));
        } else {
            bytes.extend([0; 16]);
        }
        bytes.extend([0; 16]);
        let row_count_offset = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(bookmark_entry_10_1(&GP_LONG_CLASS_ID, 21));
        bytes.extend([0; 16]);
        bytes.extend(bookmark_entry_10_1(&GP_MULTI_VALUE_CLASS_ID, 31));
        bytes.extend(bookmark_entry_10_1(&GP_STRING_CLASS_ID, 20));
        bytes.extend(bookmark_string_10_1("拓扑"));
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(bookmark_string_10_1("书签"));
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
        let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
            panic!("应读取拓扑要素类型名称");
        };
        let domain = body.legacy_entries[0]
            .legacy_domain
            .as_ref()
            .unwrap()
            .body
            .as_deref();
        let coded = match domain {
            Some(GpTopoDomain10_1::FieldInfo {
                version: 1,
                coded_values,
            }) if nested => coded_values.body.as_deref().unwrap(),
            Some(GpTopoDomain10_1::CodedValue(coded)) if !nested => coded,
            _ => panic!("应读取编码值域"),
        };
        assert_eq!(coded.labels, (1, vec!["标签".into()]));
        assert_eq!(coded.raw_value_count, 17);
        let GpCodedValue10_1::ValueTable {
            raw_column_count,
            column_types,
            raw_row_count,
            cells,
        } = &coded.values[16].body.as_ref().unwrap().value
        else {
            panic!("应读取 GP 值表");
        };
        assert_eq!((*raw_column_count, *raw_row_count), (2, 2));
        assert_eq!(column_types[1], BookmarkNameEntry10_1::Null);
        if nested && type_class != Some(GP_FUNCTION_NAME_CLASS_ID) {
            assert!(matches!(
                column_types[0],
                BookmarkNameEntry10_1::BackReference { key: 32, .. }
            ));
        }
        assert!(matches!(
            cells[0].entry,
            ObjectEntry10_1::BackReference { key: 21, .. }
        ));
        assert_eq!(cells[1].entry, ObjectEntry10_1::Null);
        assert!(matches!(
            cells[2].entry,
            ObjectEntry10_1::BackReference { key: 31, .. }
        ));
        assert!(matches!(
            cells[3].entry,
            ObjectEntry10_1::BackReference { key: 20, .. }
        ));
        let GpCodedValue10_1::MultiValue {
            type_name,
            raw_count,
            values,
        } = &coded.values[15].body.as_ref().unwrap().value
        else {
            panic!("应读取 GP 多值对象");
        };
        match type_class {
            Some(GP_FUNCTION_NAME_CLASS_ID) => assert!(matches!(
                type_name.as_ref(),
                BookmarkNameEntry10_1::GpFunctionName(body) if body.version == 2
            )),
            Some(GP_DATA_TYPE_NAME_CLASS_ID) => assert!(matches!(
                type_name.as_ref(),
                BookmarkNameEntry10_1::GpDataTypeName(body) if body.version == 2
            )),
            Some(GP_MULTI_VALUE_TYPE_NAME_CLASS_ID) => assert!(matches!(
                type_name.as_ref(),
                BookmarkNameEntry10_1::GpMultiValueTypeName { version: 1, element_type }
                    if matches!(element_type.as_ref(), BookmarkNameEntry10_1::GpDataTypeName(body)
                        if body.version == 2)
            )),
            Some(GP_VALUE_TABLE_TYPE_NAME_CLASS_ID) => assert!(matches!(
                type_name.as_ref(),
                BookmarkNameEntry10_1::GpValueTableTypeName { version: 3, columns, .. }
                    if columns.is_empty()
            )),
            Some(GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID) => assert!(matches!(
                type_name.as_ref(),
                BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) if body.version == 4
            )),
            None => assert_eq!(type_name.as_ref(), &BookmarkNameEntry10_1::Null),
            _ => unreachable!(),
        }
        assert_eq!(*raw_count, 2);
        assert_eq!(
            values[0].entry,
            ObjectEntry10_1::BackReference {
                class_id: GP_LONG_CLASS_ID,
                key: 21,
            }
        );
        assert_eq!(
            values[1].body.as_ref().unwrap().value,
            GpCodedValue10_1::MultiValue {
                type_name: Box::new(if let Some(class_id) = type_class {
                    BookmarkNameEntry10_1::BackReference { class_id, key: 32 }
                } else {
                    BookmarkNameEntry10_1::Null
                }),
                raw_count: -1,
                values: Vec::new()
            }
        );
        assert_eq!(
            coded.values[13].body.as_ref().unwrap().value,
            GpCodedValue10_1::RandomNumberGenerator {
                seed: -79,
                raw_generator: -9
            }
        );
        assert_eq!(
            coded.values[14].entry,
            ObjectEntry10_1::BackReference {
                class_id: GP_RANDOM_NUMBER_GENERATOR_CLASS_ID,
                key: 29
            }
        );
        assert!(coded.values[14].body.is_none());
        let encrypted = coded.values[11].body.as_ref().unwrap();
        assert_eq!(encrypted.version, 2);
        assert_eq!(
            encrypted.value,
            GpCodedValue10_1::EncryptedString("synthetic-persisted-value".into())
        );
        assert_eq!(
            coded.values[12].entry,
            ObjectEntry10_1::BackReference {
                class_id: GP_ENCRYPTED_STRING_CLASS_ID,
                key: 28,
            }
        );
        assert!(coded.values[12].body.is_none());
        assert_eq!(
            coded.values[6].body.as_ref().unwrap().value,
            GpCodedValue10_1::LinearUnit {
                value_bits: (-2.5_f64).to_bits(),
                unit: -1
            }
        );
        assert_eq!(
            coded.values[7].body.as_ref().unwrap().value,
            GpCodedValue10_1::Variant(Box::new(PropertyValueObjectStream10_1::Inline(
                PropertyValue10_1::Raw4(42_i32.to_le_bytes())
            )))
        );
        assert_eq!(
            coded.values[9].entry,
            ObjectEntry10_1::BackReference {
                class_id: GP_VARIANT_CLASS_ID,
                key: 26,
            }
        );
        assert!(coded.values[9].body.is_none());
        assert_eq!(
            coded.values[8].body.as_ref().unwrap().value,
            GpCodedValue10_1::ArealUnit {
                value_bits: 3.25_f64.to_bits(),
                unit: -2,
            }
        );
        assert_eq!(
            coded.values[10].entry,
            ObjectEntry10_1::BackReference {
                class_id: GP_AREAL_UNIT_CLASS_ID,
                key: 27
            }
        );
        assert!(coded.values[10].body.is_none());
        assert_eq!(
            coded.values[3].body.as_ref().unwrap().value,
            GpCodedValue10_1::DoubleBits((-0.0_f64).to_bits())
        );
        assert_eq!(
            coded.values[4].body.as_ref().unwrap().value,
            GpCodedValue10_1::Boolean(0xffff)
        );
        assert_eq!(
            coded.values[5].body.as_ref().unwrap().value,
            GpCodedValue10_1::Date(Box::new(PropertyValueObjectStream10_1::Inline(
                PropertyValue10_1::Raw8(45000.5_f64.to_le_bytes())
            )))
        );
        assert_eq!(
            coded.values[0].body.as_ref().unwrap().value,
            GpCodedValue10_1::String("值".into())
        );
        assert_eq!(
            coded.values[1].body.as_ref().unwrap().value,
            GpCodedValue10_1::Long(-7)
        );
        assert!(matches!(
            coded.values[2].entry,
            ObjectEntry10_1::BackReference { key: 21, .. }
        ));
        assert_eq!(result.name, "书签");
        assert_eq!(reader.position(), end);
        assert!([17, 18, 20, 21].into_iter().all(|key| refs.contains(key)));
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut wrong_type = bytes.clone();
        wrong_type.splice(
            type_offset..type_end,
            bookmark_entry_10_1(&GP_DATA_TYPE_NAME_CLASS_ID, 30),
        );
        let mut reader = Reader::new(&wrong_type);
        let mut refs = RefTable10_1::new();
        let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());

        let mut oversized = bytes.clone();
        oversized[row_count_offset..row_count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let mut reader = Reader::new(&oversized);
        let mut refs = RefTable10_1::new();
        assert_eq!(
            read_feature_bookmark_10_1(&mut reader, 1024, &mut refs)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());

        bytes[encrypted_version_offset] = 3;
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn feature_bookmark_reads_object_endpoints_in_legacy_topo_range_domain() {
    use mxd2qgz::maps::map_surround::{
        GP_RANGE_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };
    use mxd2qgz::objects::property_set::{PROPERTY_SET_CLASS_ID, PropertyValueObjectStream10_1};
    use mxd2qgz::stream::variant_stream::{VariantObjectHeader10_1, VariantValue10_1};

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(GP_RANGE_DOMAIN_CLASS_ID);
    bytes.extend(18_u32.to_le_bytes());
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(5_u32.to_le_bytes());
    bytes.extend(13_u16.to_le_bytes());
    let class_offset = bytes.len();
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(13_u16.to_le_bytes());
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(19_u32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let Some(GpTopoDomain10_1::Range { bounds, .. }) = body.legacy_entries[0]
        .legacy_domain
        .as_ref()
        .unwrap()
        .body
        .as_deref()
    else {
        panic!("应读取范围域");
    };
    assert!(matches!(
        &bounds[0],
        VariantValue10_1::Object {
            value_type: 13,
            header: VariantObjectHeader10_1 { class_id: PROPERTY_SET_CLASS_ID, key: Some(19) },
            body,
        } if matches!(body.as_ref(), PropertyValueObjectStream10_1::Object { key: 19, body, .. }
            if body.count == 0)
    ));
    assert!(matches!(
        bounds[1],
        VariantValue10_1::BackReference {
            value_type: 13,
            class_id: PROPERTY_SET_CLASS_ID,
            key: 19
        }
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
    bytes[class_offset..class_offset + 16].fill(0xaa);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let error = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn feature_bookmark_reads_gp_value_table_type_name_columns() {
    use mxd2qgz::layers::feature_layer_tail::UID_CLASS_ID;
    use mxd2qgz::maps::map_surround::{
        GP_DATA_TYPE_NAME_CLASS_ID, GP_VALUE_TABLE_TYPE_NAME_CLASS_ID,
        GP_WORKSPACE_DOMAIN_CLASS_ID, GpTopoDomain10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for version in [1_i16, 2, 3] {
        let mut bytes = bookmark_header_10_1(&GP_VALUE_TABLE_TYPE_NAME_CLASS_ID, 17);
        bytes.extend(version.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(bookmark_entry_10_1(&GP_DATA_TYPE_NAME_CLASS_ID, 18));
        bytes.extend([2, 0, 2, 0]);
        bytes.extend([0; 16]);
        if version < 2 {
            bytes.extend(bookmark_entry_10_1(&GP_WORKSPACE_DOMAIN_CLASS_ID, 19));
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("列"));
        bytes.extend((-5_i32).to_le_bytes());
        if version > 2 {
            bytes.extend(bookmark_entry_10_1(&UID_CLASS_ID, 20));
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 17]);
            bytes.extend(9_i32.to_le_bytes());
        }
        bytes.extend(bookmark_string_10_1("表"));
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut table).unwrap();
        let BookmarkNameEntry10_1::GpValueTableTypeName {
            version: actual_version,
            raw_column_count,
            columns,
        } = result.name_object
        else {
            panic!("应读取 GP 值表类型名称");
        };
        assert_eq!((actual_version, raw_column_count), (version, 1));
        let column = &columns[0];
        assert!(matches!(
            column.data_type,
            BookmarkNameEntry10_1::GpDataTypeName(_)
        ));
        assert_eq!((&*column.name, column.raw_long), ("列", -5));
        assert_eq!(column.legacy_domain.is_some(), version < 2);
        if let Some(domain) = &column.legacy_domain {
            assert!(matches!(
                domain.body.as_deref(),
                Some(GpTopoDomain10_1::Workspace { .. })
            ));
        }
        assert_eq!(column.uid.is_some(), version > 2);
        if let Some(uid) = &column.uid {
            assert_eq!(
                uid.entry,
                ObjectEntry10_1::NewObject {
                    class_id: UID_CLASS_ID,
                    key: 20
                }
            );
            assert!(uid.body.is_some());
        }
        assert_eq!(result.name, "表");
        assert_eq!(reader.position(), end);

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut table = RefTable10_1::new();
            assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut table).is_err());
            assert_eq!(reader.position(), 0);
            assert!(table.is_empty());
        }
    }
}

fn assert_legacy_topo_coded_value(
    class_id: [u8; 16],
    version: u16,
    payload: &[u8],
    expected: mxd2qgz::maps::map_surround::GpCodedValue10_1,
) {
    use mxd2qgz::maps::map_surround::{
        GP_CODED_VALUE_DOMAIN_CLASS_ID, GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, GpTopoDomain10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut bytes = bookmark_header_10_1(&GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID, 17);
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(bookmark_entry_10_1(&GP_CODED_VALUE_DOMAIN_CLASS_ID, 18));
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(bookmark_entry_10_1(&class_id, 19));
    let version_offset = bytes.len();
    bytes.extend(version.to_le_bytes());
    bytes.extend([1, 0, 7, 0]);
    bytes.extend(payload);
    bytes.extend(bookmark_entry_10_1(&class_id, 19));
    bytes.extend(bookmark_string_10_1("拓扑"));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(bookmark_string_10_1("书签"));
    let end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).unwrap();
    let BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(body) = result.name_object else {
        panic!("应读取拓扑要素类型名称");
    };
    let domain = body.legacy_entries[0].legacy_domain.as_ref().unwrap();
    let Some(GpTopoDomain10_1::CodedValue(domain)) = domain.body.as_deref() else {
        panic!("应读取编码值域");
    };
    let value = domain.values[0].body.as_deref().unwrap();
    assert_eq!(
        (value.version, value.base_version, value.base_short),
        (version, 1, 7)
    );
    assert_eq!(value.value, expected);
    assert_eq!(
        domain.values[1].entry,
        ObjectEntry10_1::BackReference { class_id, key: 19 }
    );
    assert!(domain.values[1].body.is_none());
    assert_eq!(result.name, "书签");
    assert_eq!(reader.position(), end);

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(read_feature_bookmark_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    use mxd2qgz::maps::map_surround::GP_NA_HIERARCHY_SETTINGS_CLASS_ID;
    let invalid_version: u16 = if class_id == GP_NA_HIERARCHY_SETTINGS_CLASS_ID {
        0
    } else {
        0x7fff
    };
    bytes[version_offset..version_offset + 2].copy_from_slice(&invalid_version.to_le_bytes());
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
fn feature_bookmark_reads_gp_point_coded_value() {
    use mxd2qgz::maps::map_surround::{GP_POINT_CLASS_ID, GpCodedValue10_1};

    let (x_bits, y_bits) = ((-0.0_f64).to_bits(), 0x7ff8_0000_0000_0042_u64);
    let payload = [x_bits.to_le_bytes(), y_bits.to_le_bytes()].concat();
    assert_legacy_topo_coded_value(
        GP_POINT_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::Point { x_bits, y_bits },
    );
}

#[test]
fn feature_bookmark_reads_gp_expression_coded_values() {
    use mxd2qgz::maps::map_surround::{
        GP_INFO_EXPRESSION_CLASS_ID, GP_SQL_EXPRESSION_CLASS_ID, GpCodedValue10_1,
    };

    let sql = "类别 = '河流'";
    assert_legacy_topo_coded_value(
        GP_SQL_EXPRESSION_CLASS_ID,
        1,
        &bookmark_string_10_1(sql),
        GpCodedValue10_1::SqlExpression(sql.into()),
    );

    let info = "AREA GT 0";
    assert_legacy_topo_coded_value(
        GP_INFO_EXPRESSION_CLASS_ID,
        1,
        &bookmark_string_10_1(info),
        GpCodedValue10_1::InfoExpression(info.into()),
    );
}

#[test]
fn feature_bookmark_reads_gp_values_with_spatial_references() {
    use mxd2qgz::maps::map_surround::{
        GP_COORDINATE_SYSTEM_CLASS_ID, GP_M_DOMAIN_CLASS_ID, GP_SPATIAL_REFERENCE_CLASS_ID,
        GP_XY_DOMAIN_CLASS_ID, GP_Z_DOMAIN_CLASS_ID, GpCodedValue10_1,
    };
    use mxd2qgz::objects::envelope::SpatialReferenceEntry10_1;

    for (class_id, version, legacy_short) in [
        (GP_COORDINATE_SYSTEM_CLASS_ID, 1, None),
        (GP_XY_DOMAIN_CLASS_ID, 1, None),
        (GP_Z_DOMAIN_CLASS_ID, 1, None),
        (GP_M_DOMAIN_CLASS_ID, 1, None),
        (GP_SPATIAL_REFERENCE_CLASS_ID, 1, None),
        (GP_SPATIAL_REFERENCE_CLASS_ID, 2, Some(0xf123_u16)),
        (GP_SPATIAL_REFERENCE_CLASS_ID, 3, None),
    ] {
        let mut payload = vec![0; 16];
        if let Some(raw) = legacy_short {
            payload.extend(raw.to_le_bytes());
        }
        assert_legacy_topo_coded_value(
            class_id,
            version,
            &payload,
            GpCodedValue10_1::SpatialReference {
                reference: Box::new(SpatialReferenceEntry10_1::Null),
                legacy_short,
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_envelope_and_extent_values() {
    use mxd2qgz::maps::map_surround::{
        BookmarkEnvelopeEntry10_1, GP_ENVELOPE_CLASS_ID, GP_EXTENT_CLASS_ID, GpCodedValue10_1,
    };
    use mxd2qgz::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1};

    let coords = [-12.0_f64, -8.0, 11.0, 17.0];
    let mut envelope_bytes = bookmark_entry_10_1(&ENVELOPE_CLASS_ID, 20);
    envelope_bytes.extend(1_u16.to_le_bytes());
    envelope_bytes.extend(coords.into_iter().flat_map(f64::to_le_bytes));

    assert_legacy_topo_coded_value(
        GP_ENVELOPE_CLASS_ID,
        1,
        &envelope_bytes,
        GpCodedValue10_1::Envelope {
            envelope: Box::new(BookmarkEnvelopeEntry10_1::Envelope {
                key: 20,
                body: Envelope10_1 {
                    version: 1,
                    coords,
                    attribute_flags: None,
                    z_domain: None,
                    m_domain: None,
                    spatial_reference: None,
                },
            }),
            raw_mode: None,
        },
    );

    let mode = -3_i32;
    let mut extent_bytes = mode.to_le_bytes().to_vec();
    extent_bytes.extend([0; 16]);
    assert_legacy_topo_coded_value(
        GP_EXTENT_CLASS_ID,
        1,
        &extent_bytes,
        GpCodedValue10_1::Envelope {
            envelope: Box::new(BookmarkEnvelopeEntry10_1::Null),
            raw_mode: Some(mode),
        },
    );
}

#[test]
fn feature_bookmark_reads_gp_cell_size_xy_coded_value() {
    use mxd2qgz::maps::map_surround::{GP_CELL_SIZE_XY_CLASS_ID, GpCodedValue10_1};

    let (x_bits, y_bits) = (4.5_f64.to_bits(), 2.75_f64.to_bits());
    let payload = [x_bits.to_le_bytes(), y_bits.to_le_bytes()].concat();
    assert_legacy_topo_coded_value(
        GP_CELL_SIZE_XY_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::CellSizeXy { x_bits, y_bits },
    );
}

#[test]
fn feature_bookmark_reads_gp_calculator_expression_value() {
    use mxd2qgz::maps::map_surround::{GP_CALCULATOR_EXPRESSION_CLASS_ID, GpCodedValue10_1};

    let expression = "[面积] * 2";
    assert_legacy_topo_coded_value(
        GP_CALCULATOR_EXPRESSION_CLASS_ID,
        1,
        &bookmark_string_10_1(expression),
        GpCodedValue10_1::CalculatorExpression(expression.into()),
    );
}

#[test]
fn feature_bookmark_reads_gp_route_measure_event_properties() {
    use mxd2qgz::maps::map_surround::{
        GP_ROUTE_MEASURE_EVENT_PROPERTIES_CLASS_ID, GpCodedValue10_1,
    };

    let strings = ["路线号", "LINE", "起点", "终点"];
    let payload: Vec<_> = strings.into_iter().flat_map(bookmark_string_10_1).collect();
    assert_legacy_topo_coded_value(
        GP_ROUTE_MEASURE_EVENT_PROPERTIES_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::RouteMeasureEventProperties(strings.map(String::from)),
    );
}

#[test]
fn feature_bookmark_reads_gp_sa_cell_size() {
    use mxd2qgz::maps::map_surround::{GP_SA_CELL_SIZE_CLASS_ID, GpCodedValue10_1};

    let raw_mode = -2_i32;
    let (x_bits, y_bits) = (30.0_f64.to_bits(), (-0.0_f64).to_bits());
    let mut payload = raw_mode.to_le_bytes().to_vec();
    payload.extend(x_bits.to_le_bytes());
    payload.extend(y_bits.to_le_bytes());
    assert_legacy_topo_coded_value(
        GP_SA_CELL_SIZE_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::SaCellSize {
            raw_mode,
            x_bits,
            y_bits,
        },
    );
}

#[test]
fn feature_bookmark_reads_spatial_analyst_values_with_legacy_name_slots() {
    use mxd2qgz::maps::map_surround::{
        GP_EVALUATION_SCALE_CLASS_ID, GP_SA_MAP_ALGEBRA_EXP_CLASS_ID, GP_SA_RADIUS_CLASS_ID,
        GpCodedValue10_1,
    };

    for version in [1_u16, 2] {
        let legacy_name = (version == 1).then(|| Box::new(BookmarkNameEntry10_1::Null));
        let prefix = if version == 1 {
            vec![0; 16]
        } else {
            Vec::new()
        };
        let raw_longs = [-3_i32, 17];
        let distance_bits = 10.75_f64.to_bits();
        let mut payload = prefix.clone();
        payload.extend(raw_longs.into_iter().flat_map(i32::to_le_bytes));
        payload.extend(distance_bits.to_le_bytes());
        assert_legacy_topo_coded_value(
            GP_SA_RADIUS_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::SaRadius {
                legacy_name: legacy_name.clone(),
                raw_longs,
                distance_bits,
            },
        );

        let raw_longs = [-7_i32, 12, 4];
        let mut payload = prefix.clone();
        payload.extend(raw_longs.into_iter().flat_map(i32::to_le_bytes));
        assert_legacy_topo_coded_value(
            GP_EVALUATION_SCALE_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::EvaluationScale {
                legacy_name: legacy_name.clone(),
                raw_longs,
            },
        );

        let expression = "[高程] / 2";
        let mut payload = prefix;
        payload.extend((expression.encode_utf16().count() as u32).to_le_bytes());
        payload.extend(
            expression
                .encode_utf16()
                .chain([0])
                .flat_map(u16::to_le_bytes),
        );
        assert_legacy_topo_coded_value(
            GP_SA_MAP_ALGEBRA_EXP_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::SaMapAlgebraExpression {
                legacy_name,
                expression: expression.into(),
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_sa_fuzzy_function_arguments() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_LONG_CLASS_ID, GP_SA_FUZZY_FUNCTION_CLASS_ID, GpCodedValue10_1, GpCodedValueBody10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut payload = bookmark_entry_10_1(&GP_LONG_CLASS_ID, 20);
    payload.extend([1, 0, 1, 0, 7, 0]);
    payload.extend((-6_i32).to_le_bytes());
    payload.extend(bookmark_entry_10_1(&GP_LONG_CLASS_ID, 20));
    payload.extend([0; 16]);
    let raw_flags = [0, 0xf0];
    payload.extend(raw_flags);
    assert_legacy_topo_coded_value(
        GP_SA_FUZZY_FUNCTION_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::SaFuzzyFunction {
            arguments: [
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::NewObject {
                        class_id: GP_LONG_CLASS_ID,
                        key: 20,
                    },
                    body: Some(Box::new(GpCodedValueBody10_1 {
                        version: 1,
                        base_version: 1,
                        base_short: 7,
                        value: GpCodedValue10_1::Long(-6),
                    })),
                },
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::BackReference {
                        class_id: GP_LONG_CLASS_ID,
                        key: 20,
                    },
                    body: None,
                },
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::Null,
                    body: None,
                },
            ],
            raw_flags,
        },
    );
}

#[test]
fn feature_bookmark_reads_numeric_and_string_remap_rows() {
    use mxd2qgz::maps::map_surround::{
        GP_SA_NUMBER_REMAP_CLASS_ID, GP_SA_STRING_REMAP_CLASS_ID, GpCodedValue10_1, GpSaRemap10_1,
        GpSaRemapRow10_1,
    };

    let range_bits = [(-0.0_f64).to_bits(), 12.5_f64.to_bits()];
    for version in [1_u16, 2, 3, 4] {
        for (class_id, row) in [
            (
                GP_SA_NUMBER_REMAP_CLASS_ID,
                GpSaRemapRow10_1::Number {
                    range_bits,
                    raw_long: 7,
                    text_bytes: vec![0xb2, 0xe2],
                },
            ),
            (
                GP_SA_STRING_REMAP_CLASS_ID,
                GpSaRemapRow10_1::String {
                    source_bytes: vec![0xd6, 0xb5],
                    raw_long: -4,
                    text_bytes: vec![],
                },
            ),
        ] {
            let legacy_name = (version == 1).then(|| Box::new(BookmarkNameEntry10_1::Null));
            let version_three_fields = (version >= 3).then_some((0xffff_u16, -9_i32));
            let mut payload = Vec::new();
            if legacy_name.is_some() {
                payload.extend([0; 16]);
            }
            payload.extend((-2_i32).to_le_bytes());
            payload.extend(1_i32.to_le_bytes());
            if let Some((short, long)) = version_three_fields {
                payload.extend(short.to_le_bytes());
                payload.extend(long.to_le_bytes());
            }
            let push_bytes = |payload: &mut Vec<u8>, bytes: &[u8]| {
                payload.extend((bytes.len() as u32).to_le_bytes());
                payload.extend(bytes);
            };
            match &row {
                GpSaRemapRow10_1::Number {
                    range_bits,
                    raw_long,
                    text_bytes,
                } => {
                    payload.extend(range_bits.iter().flat_map(|bits| bits.to_le_bytes()));
                    payload.extend(raw_long.to_le_bytes());
                    push_bytes(&mut payload, text_bytes);
                }
                GpSaRemapRow10_1::String {
                    source_bytes,
                    raw_long,
                    text_bytes,
                } => {
                    push_bytes(&mut payload, source_bytes);
                    payload.extend(raw_long.to_le_bytes());
                    push_bytes(&mut payload, text_bytes);
                }
            }
            assert_legacy_topo_coded_value(
                class_id,
                version,
                &payload,
                GpCodedValue10_1::SaRemap(Box::new(GpSaRemap10_1 {
                    legacy_name,
                    raw_long: -2,
                    raw_count: 1,
                    version_three_fields,
                    version_four_fields: None,
                    rows: vec![row],
                })),
            );
        }
    }
}

#[test]
fn feature_bookmark_reads_all_spatial_analyst_neighborhood_branches() {
    use mxd2qgz::maps::map_surround::{
        GP_SA_NEIGHBORHOOD_CLASS_ID, GpCodedValue10_1, GpSaNeighborhood10_1,
        GpSaNeighborhoodParameters10_1 as Parameters,
    };
    use mxd2qgz::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};

    for (version, shape_kind, storage_kind) in [
        (1_u16, 1_i32, 0_i32),
        (2, 2, 1),
        (2, 3, 0),
        (2, 4, 0),
        (2, 5, 1),
        (2, 5, 0),
        (2, 6, 1),
        (2, 6, 0),
        (2, -7, 1),
    ] {
        let mut payload = Vec::new();
        let legacy_name = (version == 1).then(|| Box::new(BookmarkNameEntry10_1::Null));
        if legacy_name.is_some() {
            payload.extend([0; 16]);
        }
        payload.extend(storage_kind.to_le_bytes());
        payload.extend(shape_kind.to_le_bytes());
        payload.extend((-3_i32).to_le_bytes());
        let parameters = match shape_kind {
            1..=4 => {
                let count = [2, 1, 2, 3][shape_kind as usize - 1];
                let bits = [(-0.0_f64).to_bits(), 12.0_f64.to_bits(), 17.0_f64.to_bits()];
                payload.extend(bits[..count].iter().flat_map(|value| value.to_le_bytes()));
                Parameters::Dimensions(bits[..count].to_vec())
            }
            5 | 6 if storage_kind == 1 => {
                let raw_longs = [-4_i32, 5];
                payload.extend(raw_longs.into_iter().flat_map(i32::to_le_bytes));
                payload.extend(3_u16.to_le_bytes());
                payload.extend([0; 14]);
                payload.extend(11_i32.to_le_bytes());
                Parameters::Embedded {
                    raw_longs,
                    values: Box::new(PropertyValueObjectStream10_1::Inline(
                        PropertyValue10_1::Raw4(11_i32.to_le_bytes()),
                    )),
                }
            }
            5 | 6 => {
                payload.extend(bookmark_string_10_1("邻域.txt"));
                Parameters::Path("邻域.txt".into())
            }
            _ => Parameters::None,
        };
        assert_legacy_topo_coded_value(
            GP_SA_NEIGHBORHOOD_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::SaNeighborhood(Box::new(GpSaNeighborhood10_1 {
                legacy_name,
                storage_kind,
                shape_kind,
                raw_long: -3,
                parameters,
            })),
        );
    }
}

#[test]
fn feature_bookmark_reads_horizontal_and_vertical_distance_factors() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_SA_HORIZONTAL_FACTOR_CLASS_ID, GP_SA_VERTICAL_FACTOR_CLASS_ID, GpCodedValue10_1,
        PATH_DISTANCE_HORIZONTAL_FACTOR_CLASS_ID, PATH_DISTANCE_VERTICAL_FACTOR_CLASS_ID,
        PathDistanceFactor10_1, PathDistanceFactorParameters10_1 as Parameters,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let numbers = [1.5_f64.to_bits(), (-0.0_f64).to_bits(), u64::MAX, 0];
    for (wrapper, nested, counts) in [
        (
            GP_SA_HORIZONTAL_FACTOR_CLASS_ID,
            PATH_DISTANCE_HORIZONTAL_FACTOR_CLASS_ID,
            &[2, 2, 3, 3, 0][..],
        ),
        (
            GP_SA_VERTICAL_FACTOR_CLASS_ID,
            PATH_DISTANCE_VERTICAL_FACTOR_CLASS_ID,
            &[3, 4, 4, 0, 4, 4, 3, 3, 4, 4][..],
        ),
    ] {
        assert_legacy_topo_coded_value(
            wrapper,
            1,
            &[0; 16],
            GpCodedValue10_1::SaDistanceFactor(RuleSlot10_1 {
                entry: ObjectEntry10_1::Null,
                body: None,
            }),
        );
        for (kind, count) in (1_i32..).zip(counts.iter().copied()).chain([(-7, 0)]) {
            for version in [0_u16, 1] {
                let mut payload = bookmark_entry_10_1(&nested, 20);
                payload.extend(version.to_le_bytes());
                payload.extend(kind.to_le_bytes());
                let parameters = match (count, kind) {
                    (_, -7) => Parameters::None,
                    (0, _) => {
                        let name = "距离因子.txt";
                        payload.extend(bookmark_string_10_1(name));
                        Parameters::Path(name.to_owned())
                    }
                    _ => {
                        let numbers = numbers[..count].to_vec();
                        payload.extend(numbers.iter().flat_map(|bits| bits.to_le_bytes()));
                        Parameters::Numbers(numbers)
                    }
                };
                assert_legacy_topo_coded_value(
                    wrapper,
                    version,
                    &payload,
                    GpCodedValue10_1::SaDistanceFactor(RuleSlot10_1 {
                        entry: ObjectEntry10_1::NewObject {
                            class_id: nested,
                            key: 20,
                        },
                        body: Some(Box::new(PathDistanceFactor10_1 {
                            version,
                            kind,
                            parameters,
                        })),
                    }),
                );
            }
        }
    }
}

#[test]
fn feature_bookmark_reads_all_semivariogram_versions() {
    use mxd2qgz::maps::map_surround::{
        GP_SA_SEMI_VARIOGRAM_CLASS_ID, GpCodedValue10_1, GpSaSemiVariogram10_1,
        GpSaSemiVariogramLag10_1 as Lag,
    };

    let [range_bits, partial_sill_bits, nugget_bits] =
        [15.0_f64.to_bits(), (-0.0_f64).to_bits(), u64::MAX];
    for version in 0_u16..=3 {
        let mut payload = Vec::new();
        let legacy_name = (version == 1).then(|| {
            payload.extend([0; 16]);
            Box::new(BookmarkNameEntry10_1::Null)
        });
        payload.extend((-3_i32).to_le_bytes());
        let lag = if version == 3 {
            let bits = 2.5_f64.to_bits();
            payload.extend(bits.to_le_bytes());
            Lag::Number(bits)
        } else {
            let bytes: &[u8] = match version {
                0 => b"#\0",
                1 => b"12.5\0",
                _ => &[0xe9, 0],
            };
            payload.extend((bytes.len() as u32).to_le_bytes());
            payload.extend(bytes);
            Lag::LegacyText(bytes.to_vec())
        };
        payload.extend(
            [range_bits, partial_sill_bits, nugget_bits]
                .iter()
                .flat_map(|bits| bits.to_le_bytes()),
        );
        assert_legacy_topo_coded_value(
            GP_SA_SEMI_VARIOGRAM_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::SaSemiVariogram(Box::new(GpSaSemiVariogram10_1 {
                legacy_name,
                variogram_type: -3,
                lag,
                range_bits,
                partial_sill_bits,
                nugget_bits,
            })),
        );
    }
}

#[test]
fn feature_bookmark_reads_raster_calculator_expression_and_input_refs() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_LONG_CLASS_ID, GP_RASTER_CALCULATOR_EXPRESSION_CLASS_ID, GpCodedValue10_1,
        GpCodedValueBody10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let expression = "\"高程\" + 2";
    for (version, raw_count) in [(0_u16, -1_i32), (1, 3)] {
        let mut payload = bookmark_string_10_1(expression);
        payload.extend(raw_count.to_le_bytes());
        let mut inputs = Vec::new();
        if raw_count > 0 {
            let input = bookmark_entry_10_1(&GP_LONG_CLASS_ID, 20);
            payload.extend(&input);
            payload.extend([1, 0, 1, 0, 0, 0]);
            payload.extend(2_i32.to_le_bytes());
            payload.extend(&input);
            payload.extend([0; 16]);
            inputs.extend([
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::NewObject {
                        class_id: GP_LONG_CLASS_ID,
                        key: 20,
                    },
                    body: Some(Box::new(GpCodedValueBody10_1 {
                        version: 1,
                        base_version: 1,
                        base_short: 0,
                        value: GpCodedValue10_1::Long(2),
                    })),
                },
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::BackReference {
                        class_id: GP_LONG_CLASS_ID,
                        key: 20,
                    },
                    body: None,
                },
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::Null,
                    body: None,
                },
            ]);
        }
        assert_legacy_topo_coded_value(
            GP_RASTER_CALCULATOR_EXPRESSION_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::RasterCalculatorExpression {
                expression: expression.to_owned(),
                raw_count,
                inputs,
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_extract_values_in_pairs() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_SA_EXTRACT_VALUES_CLASS_ID, GP_STRING_CLASS_ID, GpCodedValue10_1, GpCodedValueBody10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut payload = 2_i32.to_le_bytes().to_vec();
    let string_object = bookmark_entry_10_1(&GP_STRING_CLASS_ID, 20);
    payload.extend(&string_object);
    payload.extend([1, 0, 1, 0, 0, 0]);
    payload.extend(bookmark_string_10_1("栅格"));
    payload.extend([0; 16]);
    payload.extend([0; 16]);
    payload.extend(&string_object);
    let new_object = RuleSlot10_1 {
        entry: ObjectEntry10_1::NewObject {
            class_id: GP_STRING_CLASS_ID,
            key: 20,
        },
        body: Some(Box::new(GpCodedValueBody10_1 {
            version: 1,
            base_version: 1,
            base_short: 0,
            value: GpCodedValue10_1::String("栅格".to_owned()),
        })),
    };
    let reference = RuleSlot10_1 {
        entry: ObjectEntry10_1::BackReference {
            class_id: GP_STRING_CLASS_ID,
            key: 20,
        },
        body: None,
    };
    let null = RuleSlot10_1 {
        entry: ObjectEntry10_1::Null,
        body: None,
    };
    assert_legacy_topo_coded_value(
        GP_SA_EXTRACT_VALUES_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::SaExtractValues {
            raw_count: 2,
            rows: vec![[new_object, null.clone()], [null, reference]],
        },
    );
    assert_legacy_topo_coded_value(
        GP_SA_EXTRACT_VALUES_CLASS_ID,
        0,
        &(-1_i32).to_le_bytes(),
        GpCodedValue10_1::SaExtractValues {
            raw_count: -1,
            rows: Vec::new(),
        },
    );
}

#[test]
fn feature_bookmark_reads_solar_time_configuration_records() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_SA_TIME_CONFIGURATION_CLASS_ID, GpCodedValue10_1, TIME_CONFIGURATION_CLASS_ID,
        TimeConfiguration10_1, TimeConfigurationParameters10_1 as Parameters,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for version in 0_u16..=1 {
        for (kind, parameters) in [
            (-1_i32, Parameters::None),
            (
                2,
                Parameters::LongAndDoubles {
                    raw_long: 87,
                    double_bits: [10.0_f64.to_bits(), 14.5_f64.to_bits()],
                },
            ),
            (3, Parameters::Longs([2026, 15, 50])),
            (4, Parameters::Long(2026)),
        ] {
            let mut payload = bookmark_entry_10_1(&TIME_CONFIGURATION_CLASS_ID, 20);
            payload.extend(version.to_le_bytes());
            payload.extend(kind.to_le_bytes());
            match &parameters {
                Parameters::None => {}
                Parameters::LongAndDoubles {
                    raw_long,
                    double_bits,
                } => {
                    payload.extend(raw_long.to_le_bytes());
                    payload.extend(double_bits.iter().flat_map(|bits| bits.to_le_bytes()));
                }
                Parameters::Longs(longs) => {
                    payload.extend(longs.iter().flat_map(|long| long.to_le_bytes()));
                }
                Parameters::Long(long) => payload.extend(long.to_le_bytes()),
            }
            assert_legacy_topo_coded_value(
                GP_SA_TIME_CONFIGURATION_CLASS_ID,
                version,
                &payload,
                GpCodedValue10_1::SaTimeConfiguration(RuleSlot10_1 {
                    entry: ObjectEntry10_1::NewObject {
                        class_id: TIME_CONFIGURATION_CLASS_ID,
                        key: 20,
                    },
                    body: Some(Box::new(TimeConfiguration10_1 {
                        version,
                        kind,
                        parameters,
                    })),
                }),
            );
        }
    }
}

#[test]
fn feature_bookmark_reads_weighted_sum_in_triples() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_DOUBLE_CLASS_ID, GP_SA_WEIGHTED_SUM_CLASS_ID, GpCodedValue10_1, GpCodedValueBody10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let weight_bits = 0.25_f64.to_bits();
    let mut payload = 2_i32.to_le_bytes().to_vec();
    let double_object = bookmark_entry_10_1(&GP_DOUBLE_CLASS_ID, 20);
    payload.extend([0; 32]);
    payload.extend(&double_object);
    payload.extend([1, 0, 1, 0, 0, 0]);
    payload.extend(weight_bits.to_le_bytes());
    payload.extend([0; 32]);
    payload.extend(&double_object);
    let weight = RuleSlot10_1 {
        entry: ObjectEntry10_1::NewObject {
            class_id: GP_DOUBLE_CLASS_ID,
            key: 20,
        },
        body: Some(Box::new(GpCodedValueBody10_1 {
            version: 1,
            base_version: 1,
            base_short: 0,
            value: GpCodedValue10_1::DoubleBits(weight_bits),
        })),
    };
    let reference = RuleSlot10_1 {
        entry: ObjectEntry10_1::BackReference {
            class_id: GP_DOUBLE_CLASS_ID,
            key: 20,
        },
        body: None,
    };
    let null = RuleSlot10_1 {
        entry: ObjectEntry10_1::Null,
        body: None,
    };
    assert_legacy_topo_coded_value(
        GP_SA_WEIGHTED_SUM_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::SaWeightedSum {
            raw_count: 2,
            rows: vec![
                [null.clone(), null.clone(), weight],
                [null.clone(), null, reference],
            ],
        },
    );
    assert_legacy_topo_coded_value(
        GP_SA_WEIGHTED_SUM_CLASS_ID,
        0,
        &(-1_i32).to_le_bytes(),
        GpCodedValue10_1::SaWeightedSum {
            raw_count: -1,
            rows: Vec::new(),
        },
    );
}

#[test]
fn feature_bookmark_reads_generic_remap_modes_and_versioned_tail() {
    use mxd2qgz::maps::map_surround::{
        GP_SA_REMAP_CLASS_ID, GpCodedValue10_1, GpSaRemap10_1, GpSaRemapRow10_1,
    };

    for version in 0_u16..=5 {
        for kind in [1_i32, 2] {
            let mut payload = Vec::new();
            let legacy_name = (version == 1).then(|| {
                payload.extend([0; 16]);
                Box::new(BookmarkNameEntry10_1::Null)
            });
            payload.extend(kind.to_le_bytes());
            payload.extend(1_i32.to_le_bytes());
            let version_three_fields = (version >= 3).then(|| {
                payload.extend(0xffff_u16.to_le_bytes());
                payload.extend((-9_i32).to_le_bytes());
                (0xffff, -9)
            });
            let row = if kind == 1 {
                let range_bits = [(-0.0_f64).to_bits(), 2.5_f64.to_bits()];
                payload.extend(range_bits.iter().flat_map(|bits| bits.to_le_bytes()));
                payload.extend(3_i32.to_le_bytes());
                payload.extend(1_u32.to_le_bytes());
                payload.push(0xe9);
                GpSaRemapRow10_1::Number {
                    range_bits,
                    raw_long: 3,
                    text_bytes: vec![0xe9],
                }
            } else {
                payload.extend(1_u32.to_le_bytes());
                payload.push(0xe9);
                payload.extend((-3_i32).to_le_bytes());
                payload.extend(0_u32.to_le_bytes());
                GpSaRemapRow10_1::String {
                    source_bytes: vec![0xe9],
                    raw_long: -3,
                    text_bytes: Vec::new(),
                }
            };
            let version_four_fields = (version >= 4).then(|| {
                payload.extend(7_u16.to_le_bytes());
                payload.extend((-8_i32).to_le_bytes());
                payload.extend(9_i32.to_le_bytes());
                (7, [-8, 9])
            });
            assert_legacy_topo_coded_value(
                GP_SA_REMAP_CLASS_ID,
                version,
                &payload,
                GpCodedValue10_1::SaRemap(Box::new(GpSaRemap10_1 {
                    legacy_name,
                    raw_long: kind,
                    raw_count: 1,
                    version_three_fields,
                    version_four_fields,
                    rows: vec![row],
                })),
            );
        }
    }
}

#[test]
fn feature_bookmark_reads_weighted_overlay_legacy_rows() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID, GpCodedValue10_1, GpSaOverlayRemap10_1,
        GpSaOverlayRow10_1, GpSaOverlayScale10_1, GpSaRemapRow10_1, GpSaWeightedOverlayTable10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let limits_bits = [1.5_f64.to_bits(), 9.0_f64.to_bits()];
    let mut payload = vec![0; 16];
    payload.extend(limits_bits.iter().flat_map(|bits| bits.to_le_bytes()));
    payload.extend(3_i32.to_le_bytes());
    payload.extend(2_i32.to_le_bytes());
    payload.extend([0; 48]);
    payload.extend(1_i32.to_le_bytes());
    payload.extend(2_u32.to_le_bytes());
    payload.extend([0xe9, b'A']);
    payload.extend((-5_i32).to_le_bytes());
    payload.extend(7_u32.to_le_bytes());
    payload.extend(b"ValueTo");
    payload.extend([0; 48]);
    payload.extend((-1_i32).to_le_bytes());
    let null = || RuleSlot10_1 {
        entry: ObjectEntry10_1::Null,
        body: None,
    };
    assert_legacy_topo_coded_value(
        GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::SaWeightedOverlayTable(Box::new(GpSaWeightedOverlayTable10_1 {
            scale: GpSaOverlayScale10_1::Legacy {
                name: Box::new(BookmarkNameEntry10_1::Null),
                limits_bits,
                raw_long: 3,
            },
            raw_count: 2,
            rows: vec![
                GpSaOverlayRow10_1 {
                    values: std::array::from_fn(|_| null()),
                    remap: GpSaOverlayRemap10_1::Legacy {
                        raw_count: 1,
                        rows: vec![GpSaRemapRow10_1::String {
                            source_bytes: vec![0xe9, b'A'],
                            raw_long: -5,
                            text_bytes: b"ValueTo".to_vec(),
                        }],
                    },
                },
                GpSaOverlayRow10_1 {
                    values: std::array::from_fn(|_| null()),
                    remap: GpSaOverlayRemap10_1::Legacy {
                        raw_count: -1,
                        rows: Vec::new(),
                    },
                },
            ],
        })),
    );
}

#[test]
fn feature_bookmark_reads_weighted_overlay_object_rows() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_EVALUATION_SCALE_CLASS_ID, GP_SA_REMAP_CLASS_ID, GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID,
        GpCodedValue10_1, GpCodedValueBody10_1, GpSaOverlayRemap10_1, GpSaOverlayRow10_1,
        GpSaOverlayScale10_1, GpSaRemap10_1, GpSaWeightedOverlayTable10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let scale_values = [1_i32, 9, 2];
    let mut payload = bookmark_entry_10_1(&GP_EVALUATION_SCALE_CLASS_ID, 20);
    payload.extend([2, 0, 1, 0, 0, 0]);
    payload.extend(scale_values.iter().flat_map(|value| value.to_le_bytes()));
    payload.extend(2_i32.to_le_bytes());
    let remap_header = bookmark_entry_10_1(&GP_SA_REMAP_CLASS_ID, 21);
    payload.extend([0; 48]);
    payload.extend(&remap_header);
    payload.extend([2, 0, 1, 0, 0, 0]);
    payload.extend(1_i32.to_le_bytes());
    payload.extend((-1_i32).to_le_bytes());
    payload.extend([0; 48]);
    payload.extend(&remap_header);

    let scale = RuleSlot10_1 {
        entry: ObjectEntry10_1::NewObject {
            class_id: GP_EVALUATION_SCALE_CLASS_ID,
            key: 20,
        },
        body: Some(Box::new(GpCodedValueBody10_1 {
            version: 2,
            base_version: 1,
            base_short: 0,
            value: GpCodedValue10_1::EvaluationScale {
                legacy_name: None,
                raw_longs: scale_values,
            },
        })),
    };
    let remap = RuleSlot10_1 {
        entry: ObjectEntry10_1::NewObject {
            class_id: GP_SA_REMAP_CLASS_ID,
            key: 21,
        },
        body: Some(Box::new(GpCodedValueBody10_1 {
            version: 2,
            base_version: 1,
            base_short: 0,
            value: GpCodedValue10_1::SaRemap(Box::new(GpSaRemap10_1 {
                legacy_name: None,
                raw_long: 1,
                raw_count: -1,
                version_three_fields: None,
                version_four_fields: None,
                rows: Vec::new(),
            })),
        })),
    };
    let reference = RuleSlot10_1 {
        entry: ObjectEntry10_1::BackReference {
            class_id: GP_SA_REMAP_CLASS_ID,
            key: 21,
        },
        body: None,
    };
    let null = || RuleSlot10_1 {
        entry: ObjectEntry10_1::Null,
        body: None,
    };
    for version in [0, 2] {
        assert_legacy_topo_coded_value(
            GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::SaWeightedOverlayTable(Box::new(GpSaWeightedOverlayTable10_1 {
                scale: GpSaOverlayScale10_1::Object(scale.clone()),
                raw_count: 2,
                rows: vec![
                    GpSaOverlayRow10_1 {
                        values: std::array::from_fn(|_| null()),
                        remap: GpSaOverlayRemap10_1::Object(remap.clone()),
                    },
                    GpSaOverlayRow10_1 {
                        values: std::array::from_fn(|_| null()),
                        remap: GpSaOverlayRemap10_1::Object(reference.clone()),
                    },
                ],
            })),
        );
    }
}

#[test]
fn feature_bookmark_reads_topo_features_value_table() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_DATA_TYPE_NAME_CLASS_ID, GP_LONG_CLASS_ID, GP_SA_TOPO_FEATURES_CLASS_ID,
        GpCodedValue10_1, GpCodedValueBody10_1, GpDataTypeName10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let column = bookmark_entry_10_1(&GP_DATA_TYPE_NAME_CLASS_ID, 20);
    let value = bookmark_entry_10_1(&GP_LONG_CLASS_ID, 21);
    let mut payload = 2_i32.to_le_bytes().to_vec();
    payload.extend(&column);
    payload.extend([2, 0, 2, 0]);
    payload.extend([0; 16]);
    payload.extend(&column);
    payload.extend(1_i32.to_le_bytes());
    payload.extend(&value);
    payload.extend([1, 0, 1, 0, 0, 0]);
    payload.extend(42_i32.to_le_bytes());
    payload.extend(&value);

    let column_type = Box::new(GpDataTypeName10_1 {
        version: 2,
        base_version: 2,
        strings: std::array::from_fn(|_| String::new()),
        legacy_string: None,
        base_uid: None,
        legacy_uid: None,
    });
    assert_legacy_topo_coded_value(
        GP_SA_TOPO_FEATURES_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::ValueTable {
            raw_column_count: 2,
            column_types: vec![
                BookmarkNameEntry10_1::GpDataTypeName(column_type),
                BookmarkNameEntry10_1::BackReference {
                    class_id: GP_DATA_TYPE_NAME_CLASS_ID,
                    key: 20,
                },
            ],
            raw_row_count: 1,
            cells: vec![
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::NewObject {
                        class_id: GP_LONG_CLASS_ID,
                        key: 21,
                    },
                    body: Some(Box::new(GpCodedValueBody10_1 {
                        version: 1,
                        base_version: 1,
                        base_short: 0,
                        value: GpCodedValue10_1::Long(42),
                    })),
                },
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::BackReference {
                        class_id: GP_LONG_CLASS_ID,
                        key: 21,
                    },
                    body: None,
                },
            ],
        },
    );
}

#[test]
fn feature_bookmark_reads_raster_environment_options() {
    use mxd2qgz::maps::map_surround::{
        GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID, GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID,
        GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID, GpCodedValue10_1, GpRasterEnvironment10_1,
        GpRasterEnvironmentParameters10_1 as Parameters,
    };

    for (class_id, max_version) in [
        (GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID, 2_u16),
        (GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID, 2),
        (GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID, 4),
    ] {
        for version in 0..=max_version {
            let mut payload = Vec::new();
            let legacy_name = (version == 1).then(|| {
                payload.extend([0; 16]);
                Box::new(BookmarkNameEntry10_1::Null)
            });
            let raw_longs = [128_i32, -9];
            let raw_short = 0xffff_u16;
            if class_id == GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID {
                payload.extend(raw_short.to_le_bytes());
            }
            payload.extend(raw_longs.iter().flat_map(|value| value.to_le_bytes()));
            let version_three_longs = (version >= 3).then(|| {
                let longs = [7_i32, -8];
                payload.extend(longs.iter().flat_map(|value| value.to_le_bytes()));
                longs
            });
            let version_four_short = (version >= 4).then(|| {
                payload.extend(0x8001_u16.to_le_bytes());
                0x8001
            });
            let parameters = match class_id {
                GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID => Parameters::Compression(raw_longs),
                GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID => Parameters::TileSize(raw_longs),
                _ => Parameters::Pyramid {
                    raw_short,
                    raw_longs,
                    version_three_longs,
                    version_four_short,
                },
            };
            assert_legacy_topo_coded_value(
                class_id,
                version,
                &payload,
                GpCodedValue10_1::RasterEnvironment(Box::new(GpRasterEnvironment10_1 {
                    legacy_name,
                    parameters,
                })),
            );
        }
    }
}

#[test]
fn feature_bookmark_reads_raster_statistics_versioned_ignore_values() {
    use mxd2qgz::maps::map_surround::{
        GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID, GpCodedValue10_1, GpRasterEnvironment10_1,
        GpRasterEnvironmentParameters10_1, GpRasterStatisticsIgnoreValues10_1 as IgnoreValues,
    };

    for version in 0_u16..=3 {
        let mut payload = Vec::new();
        let legacy_name = (version == 2).then(|| {
            payload.extend([0; 16]);
            Box::new(BookmarkNameEntry10_1::Null)
        });
        let raw_short = 0xffff_u16;
        let raw_longs = [7_i32, -8];
        payload.extend(raw_short.to_le_bytes());
        payload.extend(raw_longs.iter().flat_map(|value| value.to_le_bytes()));
        let ignore_values = if version <= 1 {
            let bytes = [1, 0, 0xfd, 0xff];
            payload.extend(bytes);
            IgnoreValues::LegacyBytes(bytes)
        } else {
            let text = "-1 0";
            payload.extend(4_u32.to_le_bytes());
            payload.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
            payload.extend([0, 0]);
            IgnoreValues::Text(text.to_owned())
        };
        assert_legacy_topo_coded_value(
            GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::RasterEnvironment(Box::new(GpRasterEnvironment10_1 {
                legacy_name,
                parameters: GpRasterEnvironmentParameters10_1::Statistics {
                    raw_short,
                    raw_longs,
                    ignore_values,
                },
            })),
        );
    }
}

#[test]
fn feature_bookmark_reads_analysis_cell_size_and_member_string() {
    use mxd2qgz::maps::map_surround::{GP_ANALYSIS_CELL_SIZE_CLASS_ID, GpCodedValue10_1};

    for (version, size_bits, raw_text) in [
        (0_u16, (-0.0_f64).to_bits(), ""),
        (1, 12.5_f64.to_bits(), "栅格"),
    ] {
        let mut payload = size_bits.to_le_bytes().to_vec();
        payload.extend(bookmark_string_10_1(raw_text));
        assert_legacy_topo_coded_value(
            GP_ANALYSIS_CELL_SIZE_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::AnalysisCellSize {
                size_bits,
                raw_text: raw_text.to_owned(),
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_arc_info_item_fields_in_stream_order() {
    use mxd2qgz::maps::map_surround::{GP_ARC_INFO_ITEM_CLASS_ID, GpCodedValue10_1};

    let strings = ["编号", "CODE"];
    let raw_longs = [11_i32, -2, 3, -4, 5];
    let raw_shorts = [0x8001_u16, 2, 0xffff];
    let mut payload = bookmark_string_10_1(strings[0]);
    payload.extend(bookmark_string_10_1(strings[1]));
    payload.extend(raw_longs.iter().flat_map(|raw| raw.to_le_bytes()));
    payload.extend(raw_shorts.iter().flat_map(|raw| raw.to_le_bytes()));

    for version in [0, 1] {
        assert_legacy_topo_coded_value(
            GP_ARC_INFO_ITEM_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::ArcInfoItem {
                strings: strings.map(str::to_owned),
                raw_longs,
                raw_shorts,
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_data_file_transport_and_properties() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GDS_DATA_CLASS_ID, GP_DATA_FILE_CLASS_ID, GdsData10_1, GpCodedValue10_1,
    };
    use mxd2qgz::objects::property_set::{PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let strings = ["数据文件", "输出", ""].map(str::to_owned);
    let trailer = strings
        .iter()
        .flat_map(|value| bookmark_string_10_1(value))
        .collect::<Vec<_>>();
    for version in [-1_i16, 0, 1, 2, 3] {
        let data = [0, 0xff, 0x81, 1, 0];
        let mut payload = bookmark_entry_10_1(&GDS_DATA_CLASS_ID, 20);
        payload.extend(version.to_le_bytes());
        payload.extend((data.len() as u32).to_le_bytes());
        payload.extend(data);
        payload.extend(bookmark_string_10_1("表"));
        payload.extend((-12_i32).to_le_bytes());
        payload.extend(0xffff_u16.to_le_bytes());
        let extension = (version == 3).then(|| "扩展".to_owned());
        let extension_bytes = extension
            .as_deref()
            .map(bookmark_string_10_1)
            .unwrap_or_default();
        let properties = (version >= 2).then(|| {
            payload.extend(bookmark_entry_10_1(&PROPERTY_SET_CLASS_ID, 21));
            payload.extend(1_u16.to_le_bytes());
            payload.extend(0_u32.to_le_bytes());
            let following_fields = 20
                + bookmark_string_10_1("拓扑").len()
                + 4
                + bookmark_string_10_1("书签").len()
                + 1;
            RuleSlot10_1 {
                entry: ObjectEntry10_1::NewObject {
                    class_id: PROPERTY_SET_CLASS_ID,
                    key: 21,
                },
                body: Some(Box::new(PropertySetObjectStream10_1 {
                    version: 1,
                    count: 0,
                    entries: Vec::new(),
                    remaining_bytes: extension_bytes.len() + trailer.len() + following_fields,
                })),
            }
        });
        payload.extend(extension_bytes);
        payload.extend(&trailer);

        assert_legacy_topo_coded_value(
            GP_DATA_FILE_CLASS_ID,
            1,
            &payload,
            GpCodedValue10_1::DataFile {
                data: RuleSlot10_1 {
                    entry: ObjectEntry10_1::NewObject {
                        class_id: GDS_DATA_CLASS_ID,
                        key: 20,
                    },
                    body: Some(Box::new(GdsData10_1 {
                        version,
                        bytes: data.to_vec(),
                        name: "表".to_owned(),
                        raw_long: -12,
                        raw_short: 0xffff,
                        properties,
                        extra_string: extension,
                    })),
                },
                strings: strings.clone(),
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_workspace_extension_value_guid() {
    use mxd2qgz::maps::map_surround::{GP_WORKSPACE_EXTENSION_CLASS_ID, GpCodedValue10_1};

    let extension_guid = [
        1, 0, 0xff, 0x81, 2, 0, 3, 0, 0xa0, 0x16, 0xef, 0x02, 9, 0, 0x98, 7,
    ];
    for (version, name) in [(0, ""), (1, "工作空间扩展")] {
        let mut payload = bookmark_string_10_1(name);
        payload.extend(extension_guid);
        assert_legacy_topo_coded_value(
            GP_WORKSPACE_EXTENSION_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::WorkspaceExtension {
                name: name.to_owned(),
                extension_guid,
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_field_info_with_versioned_rows() {
    use mxd2qgz::maps::map_surround::{
        GP_FIELD_INFO_CLASS_ID, GpCodedValue10_1, GpFieldInfoRow10_1, GpFieldInfoRowV2Fields10_1,
    };

    let append_bstr = |buffer: &mut Vec<u8>, text: &str| {
        let count = text.encode_utf16().count();
        buffer.extend((count as u32).to_le_bytes());
        if count > 0 {
            buffer.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
            buffer.extend([0, 0]);
        }
    };
    for version in 0_u16..=2 {
        let mut payload = 2_i32.to_le_bytes().to_vec();
        let mut rows = Vec::new();
        for (leading_long, name) in [(2_i32, "字段"), (-3, "ID")] {
            payload.extend(leading_long.to_le_bytes());
            append_bstr(&mut payload, name);
            append_bstr(&mut payload, "");
            payload.extend((-5_i32).to_le_bytes());
            payload.extend(0x8001_u16.to_le_bytes());
            let extra = (version == 2).then(|| {
                append_bstr(&mut payload, "别名");
                payload.extend(6_i32.to_le_bytes());
                append_bstr(&mut payload, "说明");
                payload.extend((-7_i32).to_le_bytes());
                payload.extend(8_i32.to_le_bytes());
                GpFieldInfoRowV2Fields10_1 {
                    first_text: "别名".to_owned(),
                    first_long: 6,
                    second_text: "说明".to_owned(),
                    remaining_longs: [-7, 8],
                }
            });
            rows.push(GpFieldInfoRow10_1 {
                leading_long,
                strings: [name.to_owned(), String::new()],
                trailing_long: -5,
                raw_short: 0x8001,
                extra,
            });
        }
        assert_legacy_topo_coded_value(
            GP_FIELD_INFO_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::FieldInfo { raw_count: 2, rows },
        );
    }
    assert_legacy_topo_coded_value(
        GP_FIELD_INFO_CLASS_ID,
        2,
        &(-1_i32).to_le_bytes(),
        GpCodedValue10_1::FieldInfo {
            raw_count: -1,
            rows: Vec::new(),
        },
    );
}

#[test]
fn feature_bookmark_reads_na_class_field_map_coded_value() {
    use mxd2qgz::maps::map_surround::{
        GpCodedValue10_1, NA_CLASS_FIELD_MAP_CLASS_ID, NaClassFieldMapRow10_1,
    };
    use mxd2qgz::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};

    let mut payload = 1_i32.to_le_bytes().to_vec();
    for name in [b'A', b'B'] {
        payload.extend(1_u32.to_le_bytes());
        payload.extend([name, 0, 0, 0]);
    }
    payload.extend(3_u16.to_le_bytes());
    payload.extend([0; 14]);
    payload.extend(42_i32.to_le_bytes());
    assert_legacy_topo_coded_value(
        NA_CLASS_FIELD_MAP_CLASS_ID,
        1,
        &payload,
        GpCodedValue10_1::NaClassFieldMap {
            raw_count: 1,
            rows: vec![NaClassFieldMapRow10_1 {
                strings: ["A".into(), "B".into()],
                value: PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(
                    42_i32.to_le_bytes(),
                )),
            }],
        },
    );
    assert_legacy_topo_coded_value(
        NA_CLASS_FIELD_MAP_CLASS_ID,
        0xffff,
        &(-1_i32).to_le_bytes(),
        GpCodedValue10_1::NaClassFieldMap {
            raw_count: -1,
            rows: Vec::new(),
        },
    );
}

#[test]
fn feature_bookmark_reads_gp_na_hierarchy_settings_coded_value() {
    use mxd2qgz::maps::map_surround::{GP_NA_HIERARCHY_SETTINGS_CLASS_ID, GpCodedValue10_1};

    let raw_longs = [-3_i32, 25];
    let mut payload = raw_longs[0].to_le_bytes().to_vec();
    payload.extend(raw_longs[1].to_le_bytes());
    for version in [1, u16::MAX] {
        assert_legacy_topo_coded_value(
            GP_NA_HIERARCHY_SETTINGS_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::NaHierarchySettings { raw_longs },
        );
    }
}

#[test]
fn feature_bookmark_reads_3da_settings_coded_values() {
    use mxd2qgz::maps::map_surround::{
        GP_3DA_DECIMATE_CLASS_ID, GP_3DA_INTERPOLATE_CLASS_ID, GpCodedValue10_1,
    };

    let value_bits = (-0.0_f64).to_bits();
    let mut decimate = (-3_i32).to_le_bytes().to_vec();
    decimate.extend(value_bits.to_le_bytes());
    decimate.extend(25_i32.to_le_bytes());

    let raw_longs = [-1_i32, 2, 3, 4, 5, 6];
    let mut interpolate: Vec<_> = raw_longs.into_iter().flat_map(i32::to_le_bytes).collect();
    interpolate.extend(value_bits.to_le_bytes());
    for version in [0, 1] {
        assert_legacy_topo_coded_value(
            GP_3DA_DECIMATE_CLASS_ID,
            version,
            &decimate,
            GpCodedValue10_1::Decimate3da {
                raw_leading_long: -3,
                value_bits,
                raw_trailing_long: 25,
            },
        );
        assert_legacy_topo_coded_value(
            GP_3DA_INTERPOLATE_CLASS_ID,
            version,
            &interpolate,
            GpCodedValue10_1::Interpolate3da {
                raw_longs,
                value_bits,
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_field_info_ex_coded_value() {
    use mxd2qgz::maps::map_surround::{GP_FIELD_INFO_EX_CLASS_ID, GpCodedValue10_1};

    for version in [0, 1] {
        assert_legacy_topo_coded_value(
            GP_FIELD_INFO_EX_CLASS_ID,
            version,
            &bookmark_string_10_1("字段说明"),
            GpCodedValue10_1::FieldInfoEx("字段说明".into()),
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_address_locator_style_without_configuration() {
    use mxd2qgz::maps::map_surround::{GP_ADDRESS_LOCATOR_STYLE_CLASS_ID, GpCodedValue10_1};

    let mut payload = bookmark_string_10_1("地址定位器");
    payload.extend(bookmark_string_10_1("样式"));
    payload.extend(0_u16.to_le_bytes());
    for version in 0..=4 {
        assert_legacy_topo_coded_value(
            GP_ADDRESS_LOCATOR_STYLE_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::AddressLocatorStyle {
                strings: ["地址定位器".into(), "样式".into()],
                configuration: None,
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_address_locator_style_configuration_without_rows() {
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        GP_ADDRESS_LOCATOR_STYLE_CLASS_ID, GpAddressLocatorStyleConfig10_1, GpCodedValue10_1,
    };
    use mxd2qgz::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for version in 0..=4 {
        let mut payload = bookmark_string_10_1("地址定位器");
        payload.extend(bookmark_string_10_1("样式"));
        payload.extend(0xffff_u16.to_le_bytes());
        payload.extend([0; 48]);
        payload.extend((-1_i32).to_le_bytes());
        if version == 1 {
            payload.push(1);
        }
        if version >= 2 {
            payload.extend([0; 16]);
        }
        let variant_flag = (version >= 3).then_some(u16::from(version == 4));
        if let Some(flag) = variant_flag {
            payload.extend(flag.to_le_bytes());
        }
        if version == 4 {
            payload.extend(3_u16.to_le_bytes());
            payload.extend([0; 14]);
            payload.extend(42_i32.to_le_bytes());
            payload.extend([0; 16]);
        }
        assert_legacy_topo_coded_value(
            GP_ADDRESS_LOCATOR_STYLE_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::AddressLocatorStyle {
                strings: ["地址定位器".into(), "样式".into()],
                configuration: Some(Box::new(GpAddressLocatorStyleConfig10_1 {
                    raw_flag: 0xffff,
                    fields: std::array::from_fn(|_| RuleSlot10_1 {
                        entry: ObjectEntry10_1::Null,
                        body: None,
                    }),
                    raw_row_count: -1,
                    rows: Vec::new(),
                    legacy_byte: (version == 1).then_some(1),
                    properties: (version >= 2).then_some(RuleSlot10_1 {
                        entry: ObjectEntry10_1::Null,
                        body: None,
                    }),
                    variant_flag,
                    variant: (version == 4).then_some(PropertyValueObjectStream10_1::Inline(
                        PropertyValue10_1::Raw4(42_i32.to_le_bytes()),
                    )),
                    table: (version == 4).then_some(RuleSlot10_1 {
                        entry: ObjectEntry10_1::Null,
                        body: None,
                    }),
                })),
            },
        );
    }
}

#[test]
fn feature_bookmark_reads_gp_address_locator_style_reference_rows() {
    use mxd2qgz::layers::feature_layer_tail::{UID_CLASS_ID, UidBody10_1};
    use mxd2qgz::layers::image_server_rules::RuleSlot10_1;
    use mxd2qgz::maps::map_surround::{
        BookmarkEnvelopeEntry10_1, ConnectivityRule10_1, DE_FEATURE_CLASS_CLASS_ID,
        DE_FEATURE_DATASET_CLASS_ID, DE_GEOMETRIC_NETWORK_CLASS_ID, DE_RASTER_CATALOG_CLASS_ID,
        DE_RELATIONSHIP_CLASS_CLASS_ID, DE_TABLE_CLASS_ID, DE_TOPOLOGY_CLASS_ID, DataElement10_1,
        DeChild10_1, DeDataset10_1, DeFeatureClassTable10_1, DeGeometricNetwork10_1,
        DeRelationshipBase10_1, DeRelationshipClass10_1, DeRelationshipMetadata10_1, DeTable10_1,
        DeTableIndex10_1, DeTableIndexes10_1, EDGE_CONNECTIVITY_RULE_CLASS_ID,
        GP_ADDRESS_LOCATOR_STYLE_CLASS_ID, GP_GEOMETRIC_NETWORK_MEMBERSHIP_CLASS_ID,
        GP_RELATIONSHIP_CLASS_KEY_CLASS_ID, GP_SUBTYPE_CLASS_ID, GP_TOPOLOGY_MEMBERSHIP_CLASS_ID,
        GpAddressLocatorStyleConfig10_1, GpAddressLocatorStyleRow10_1,
        GpAddressLocatorStyleRowValues10_1, GpCodedValue10_1, GpControllerMembership10_1,
        GpRelationshipClassKey10_1, GpSubtype10_1, GpSubtypeField10_1, INDEX_CLASS_ID,
        INDEXES_CLASS_ID, JUNCTION_CONNECTIVITY_RULE_CLASS_ID, NET_WEIGHT_ASSOCIATION_CLASS_ID,
        NET_WEIGHT_CLASS_ID, NetWeight10_1, RELATIONSHIP_RULE_CLASS_ID, RelationshipRule10_1,
        TOPOLOGY_RULE_CLASS_ID, TopologyRule10_1,
    };
    use mxd2qgz::objects::envelope::SpatialReferenceEntry10_1;
    use mxd2qgz::objects::fields::{FIELDS_CLASS_ID, FieldsBody10_1};
    use mxd2qgz::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};
    use mxd2qgz::objects::string_array::StringArraySlot10_1;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for version in 0_u16..=4 {
        let mut payload = bookmark_string_10_1("");
        payload.extend(bookmark_string_10_1(""));
        payload.extend(1_u16.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&FIELDS_CLASS_ID, 21));
        payload.extend(2_u16.to_le_bytes());
        payload.extend(0_i32.to_le_bytes());
        payload.extend([0; 32]);
        payload.extend(1_i32.to_le_bytes());
        payload.extend(bookmark_string_10_1("参考数据"));
        if version == 4 {
            payload.extend(0xffff_u16.to_le_bytes());
        }
        let raw_object_count = if version >= 2 { 2_i32 } else { 0 };
        payload.extend(raw_object_count.to_le_bytes());
        let mut uids = Vec::new();
        if version >= 2 {
            let guid = [0x47; 16];
            payload.extend(bookmark_entry_10_1(&UID_CLASS_ID, 20));
            payload.extend(1_u16.to_le_bytes());
            payload.push(1);
            payload.extend(guid);
            payload.extend(7_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&UID_CLASS_ID, 20));
            uids.extend([
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::NewObject {
                        class_id: UID_CLASS_ID,
                        key: 20,
                    },
                    body: Some(Box::new(UidBody10_1 {
                        key: 20,
                        version: 1,
                        flag: 1,
                        guid,
                        value_18: 7,
                    })),
                },
                RuleSlot10_1 {
                    entry: ObjectEntry10_1::BackReference {
                        class_id: UID_CLASS_ID,
                        key: 20,
                    },
                    body: None,
                },
            ]);
        }
        payload.extend(1_i32.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&FIELDS_CLASS_ID, 21));
        let values = if version < 2 {
            payload.extend(1_i32.to_le_bytes());
            payload.extend(bookmark_string_10_1("字段"));
            GpAddressLocatorStyleRowValues10_1::Strings(vec![vec!["字段".into()]])
        } else {
            payload.extend(3_u16.to_le_bytes());
            payload.extend([0; 14]);
            payload.extend(5_i32.to_le_bytes());
            GpAddressLocatorStyleRowValues10_1::Variants(vec![
                PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(5_i32.to_le_bytes())),
            ])
        };
        payload.extend(bookmark_entry_10_1(&DE_TABLE_CLASS_ID, 22));
        payload.extend(version.to_le_bytes());
        payload.extend(2_u16.to_le_bytes());
        payload.extend(2_u16.to_le_bytes());
        payload.extend(1_u16.to_le_bytes());
        payload.extend(7_u16.to_le_bytes());
        for name in ["目录", "名称", "标题"] {
            payload.extend(bookmark_string_10_1(name));
        }
        payload.extend([0; 4]);
        payload.extend(u16::MAX.to_le_bytes());
        payload.extend(4_u32.to_le_bytes());
        for unit in "<x/>".encode_utf16().chain([0]) {
            payload.extend(unit.to_le_bytes());
        }
        payload.extend(6_i32.to_le_bytes());
        let catalog_version = (version >= 3).then_some(1_u16);
        let feature_class_id = if catalog_version.is_some() {
            DE_RASTER_CATALOG_CLASS_ID
        } else {
            DE_FEATURE_CLASS_CLASS_ID
        };
        payload.extend(bookmark_entry_10_1(&feature_class_id, 28));
        if let Some(catalog_version) = catalog_version {
            payload.extend(catalog_version.to_le_bytes());
        }
        payload.extend(1_u16.to_le_bytes());
        payload.extend([0; 10]);
        for _ in 0..3 {
            payload.extend(bookmark_string_10_1(""));
        }
        payload.extend([0; 24]);
        for _ in 0..6 {
            payload.extend(bookmark_string_10_1(""));
        }
        payload.extend([0; 76]);
        payload.extend(3_i32.to_le_bytes());
        payload.extend(4_i32.to_le_bytes());
        payload.extend(bookmark_string_10_1("形状"));
        payload.extend([1, 0, 2, 0, 3, 0]);
        payload.extend([0; 32]);
        payload.extend(bookmark_string_10_1("面积"));
        payload.extend(bookmark_string_10_1("长度"));
        payload.extend(bookmark_entry_10_1(&feature_class_id, 28));
        payload.extend(bookmark_entry_10_1(&DE_FEATURE_DATASET_CLASS_ID, 29));
        payload.extend([1, 0, 1, 0]);
        payload.extend([0; 8]);
        for _ in 0..3 {
            payload.extend(bookmark_string_10_1(""));
        }
        payload.extend([0; 54]);
        payload.extend(bookmark_entry_10_1(&DE_GEOMETRIC_NETWORK_CLASS_ID, 32));
        let network_version = version + 2;
        payload.extend(network_version.to_le_bytes());
        payload.extend([1, 0]);
        payload.extend([0; 8]);
        for _ in 0..3 {
            payload.extend(bookmark_string_10_1(""));
        }
        payload.extend([0; 54]);
        payload.extend(15_i32.to_le_bytes());
        payload.extend(bookmark_string_10_1("网络"));
        payload.extend([0; 16]);
        if network_version >= 3 {
            payload.extend(2_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&EDGE_CONNECTIVITY_RULE_CLASS_ID, 35));
            payload.extend(1_u16.to_le_bytes());
            payload.extend(bookmark_string_10_1("边"));
            payload.extend([0; 32]);
            payload.extend(1_i32.to_le_bytes());
            payload.extend(2_i32.to_le_bytes());
            payload.extend(3_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(
                &JUNCTION_CONNECTIVITY_RULE_CLASS_ID,
                36,
            ));
            payload.extend(0_u16.to_le_bytes());
            payload.extend(bookmark_string_10_1("交汇点"));
            payload.extend([0; 40]);
            payload.extend(4_u16.to_le_bytes());
            payload.extend(1_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&NET_WEIGHT_CLASS_ID, 33));
            payload.extend(1_u16.to_le_bytes());
            payload.extend(7_i32.to_le_bytes());
            payload.extend([1, 0, 0, 0, b'W', 0, 0, 0]);
            payload.extend(8_i32.to_le_bytes());
            payload.extend(9_i32.to_le_bytes());
            payload.extend(1_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&NET_WEIGHT_ASSOCIATION_CLASS_ID, 34));
            payload.extend(0_u16.to_le_bytes());
            payload.extend(7_i32.to_le_bytes());
            for name in [b'A', b'B'] {
                payload.extend([1, 0, 0, 0, name, 0, 0, 0]);
            }
        }
        if network_version >= 4 {
            payload.extend(5_u16.to_le_bytes());
            payload.extend(1.5_f64.to_le_bytes());
            payload.extend(2.5_f64.to_le_bytes());
        }
        if network_version >= 5 {
            payload.extend(6_i32.to_le_bytes());
            payload.extend(7_i32.to_le_bytes());
        }
        if network_version >= 6 {
            payload.extend(8_u16.to_le_bytes());
        }
        payload.extend(bookmark_entry_10_1(&DE_TOPOLOGY_CLASS_ID, 30));
        payload.extend([2, 0, 1, 0]);
        payload.extend([0; 8]);
        for _ in 0..3 {
            payload.extend(bookmark_string_10_1(""));
        }
        payload.extend([0; 54]);
        payload.extend(0.25_f64.to_le_bytes());
        payload.extend(0.5_f64.to_le_bytes());
        payload.extend(42_i32.to_le_bytes());
        payload.extend([0; 16]);
        payload.extend(2_i32.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&TOPOLOGY_RULE_CLASS_ID, 31));
        payload.extend([1, 0, 0, 0]);
        payload.extend(bookmark_string_10_1("规则"));
        for number in [1_i32, 2] {
            payload.extend(number.to_le_bytes());
        }
        for name in ["源", "目标"] {
            payload.extend(bookmark_string_10_1(name));
        }
        for number in [3_i32, 4, 5, 6] {
            payload.extend(number.to_le_bytes());
        }
        for number in [7_u16, 8, 9, 10, 11] {
            payload.extend(number.to_le_bytes());
        }
        payload.extend(12_i32.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&TOPOLOGY_RULE_CLASS_ID, 31));
        payload.extend(bookmark_entry_10_1(&DE_RELATIONSHIP_CLASS_CLASS_ID, 37));
        payload.extend(version.to_le_bytes());
        payload.extend([0; 8]);
        for _ in 0..3 {
            payload.extend(bookmark_string_10_1(""));
        }
        payload.extend([0; 22]);
        if version >= 2 {
            payload.extend([0; 40]);
            payload.extend(bookmark_string_10_1("正向"));
            payload.extend(bookmark_string_10_1("反向"));
            payload.extend([0; 8]);
            payload.extend([0, 0, 1, 0, 0, 0]);
            payload.extend(1_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&GP_RELATIONSHIP_CLASS_KEY_CLASS_ID, 38));
            payload.extend(1_u16.to_le_bytes());
            payload.extend(3_i32.to_le_bytes());
            payload.extend(bookmark_string_10_1("源键"));
            payload.extend(bookmark_string_10_1("目标键"));
            payload.extend(1_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&GP_RELATIONSHIP_CLASS_KEY_CLASS_ID, 38));
        }
        if version >= 3 {
            payload.extend(2_i32.to_le_bytes());
            payload.extend(bookmark_entry_10_1(&RELATIONSHIP_RULE_CLASS_ID, 39));
            payload.extend(1_u16.to_le_bytes());
            payload.extend(bookmark_string_10_1("关系规则"));
            for value in 1_i32..=10 {
                payload.extend(value.to_le_bytes());
            }
            payload.extend(bookmark_entry_10_1(&RELATIONSHIP_RULE_CLASS_ID, 39));
        }
        if version >= 2 {
            payload.extend(bookmark_entry_10_1(&FIELDS_CLASS_ID, 21));
        }
        payload.extend(11_i32.to_le_bytes());
        payload.extend(12_u16.to_le_bytes());
        payload.extend(13_u16.to_le_bytes());
        payload.extend(14_i32.to_le_bytes());
        payload.extend(bookmark_string_10_1("数据集"));
        payload.extend(15_u16.to_le_bytes());
        for name in ["A", "B", "C", "D", "E", "F"] {
            payload.extend(bookmark_string_10_1(name));
        }
        payload.extend(16_i32.to_le_bytes());
        payload.extend([0; 32]);
        payload.extend(1_i32.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&GP_SUBTYPE_CLASS_ID, 25));
        payload.extend(1_u16.to_le_bytes());
        payload.extend(bookmark_string_10_1("子类型"));
        payload.extend(8_i32.to_le_bytes());
        payload.extend(1_i32.to_le_bytes());
        payload.extend(bookmark_string_10_1("字段"));
        payload.extend(bookmark_string_10_1("域"));
        payload.extend(3_u16.to_le_bytes());
        payload.extend(6_i32.to_le_bytes());
        payload.extend(2_i32.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&GP_TOPOLOGY_MEMBERSHIP_CLASS_ID, 26));
        payload.extend(1_u16.to_le_bytes());
        payload.extend(bookmark_string_10_1("拓扑"));
        payload.extend((-0.0_f64).to_bits().to_le_bytes());
        payload.extend(3_i32.to_le_bytes());
        payload.extend((-4_i32).to_le_bytes());
        payload.extend(7_u16.to_le_bytes());
        payload.extend(bookmark_entry_10_1(
            &GP_GEOMETRIC_NETWORK_MEMBERSHIP_CLASS_ID,
            27,
        ));
        payload.extend(0_u16.to_le_bytes());
        for name in ["A", "B", "C"] {
            payload.extend(bookmark_string_10_1(name));
        }
        payload.extend(16_i32.to_le_bytes());
        if version >= 2 {
            payload.extend(17_u16.to_le_bytes());
            for _ in 0..2 {
                payload.extend(bookmark_string_10_1("v2"));
            }
        }
        if version >= 3 {
            payload.extend(18_u16.to_le_bytes());
            for _ in 0..4 {
                payload.extend(bookmark_string_10_1("v3"));
            }
        }
        if version >= 4 {
            payload.extend(19_u16.to_le_bytes());
        }
        payload.extend(bookmark_entry_10_1(&FIELDS_CLASS_ID, 21));
        payload.extend(bookmark_entry_10_1(&INDEXES_CLASS_ID, 23));
        payload.extend(1_u16.to_le_bytes());
        payload.extend(1_i32.to_le_bytes());
        payload.extend(bookmark_entry_10_1(&INDEX_CLASS_ID, 24));
        payload.extend([1, 0, 1, 0, 7, 0]);
        payload.extend(bookmark_string_10_1("索引"));
        payload.extend([1, 0, 1]);
        payload.extend(bookmark_entry_10_1(&FIELDS_CLASS_ID, 21));
        if version == 1 {
            payload.push(0);
        }
        if version >= 2 {
            payload.extend([0; 16]);
        }
        if version >= 3 {
            payload.extend(0_u16.to_le_bytes());
        }
        assert_legacy_topo_coded_value(
            GP_ADDRESS_LOCATOR_STYLE_CLASS_ID,
            version,
            &payload,
            GpCodedValue10_1::AddressLocatorStyle {
                strings: [String::new(), String::new()],
                configuration: Some(Box::new(GpAddressLocatorStyleConfig10_1 {
                    raw_flag: 1,
                    fields: [
                        RuleSlot10_1 {
                            entry: ObjectEntry10_1::NewObject {
                                class_id: FIELDS_CLASS_ID,
                                key: 21,
                            },
                            body: Some(Box::new(FieldsBody10_1 {
                                version: 2,
                                fields: Vec::new(),
                                longs_v2: Some(Vec::new()),
                            })),
                        },
                        RuleSlot10_1 {
                            entry: ObjectEntry10_1::Null,
                            body: None,
                        },
                        RuleSlot10_1 {
                            entry: ObjectEntry10_1::Null,
                            body: None,
                        },
                    ],
                    raw_row_count: 1,
                    rows: vec![GpAddressLocatorStyleRow10_1 {
                        name: "参考数据".into(),
                        raw_short: (version == 4).then_some(0xffff),
                        raw_object_count,
                        uids,
                        raw_value_count: 1,
                        fields: RuleSlot10_1 {
                            entry: ObjectEntry10_1::BackReference {
                                class_id: FIELDS_CLASS_ID,
                                key: 21,
                            },
                            body: None,
                        },
                        values,
                        table: RuleSlot10_1 {
                            entry: ObjectEntry10_1::NewObject {
                                class_id: DE_TABLE_CLASS_ID,
                                key: 22,
                            },
                            body: Some(Box::new(DeTable10_1 {
                                version,
                                dataset: DeDataset10_1 {
                                    dataset_version: 2,
                                    element: DataElement10_1 {
                                    element_version: 2,
                                    gp_value_version: 1,
                                    gp_value_short: 7,
                                    element_strings: ["目录", "名称", "标题"].map(String::from),
                                    element_shorts: [0, 0, u16::MAX],
                                    element_legacy_xml: Some("<x/>".into()),
                                    raw_child_count: 6,
                                    children: vec![
                                        RuleSlot10_1 {
                                            entry: ObjectEntry10_1::NewObject {
                                                class_id: feature_class_id,
                                                key: 28,
                                            },
                                            body: Some(Box::new(DeChild10_1::FeatureClass {
                                                derived_version: catalog_version,
                                                coverage: None,
                                                version: 1,
                                                raw_longs: [3, 4],
                                                name: "形状".into(),
                                                raw_shorts: [1, 2, 3],
                                                extent: BookmarkEnvelopeEntry10_1::Null,
                                                spatial_reference: SpatialReferenceEntry10_1::Null,
                                                strings: Some(["面积".into(), "长度".into()]),
                                                table: DeFeatureClassTable10_1::Extended(DeTable10_1 {
                                                    version: 0,
                                                    dataset: DeDataset10_1 {
                                                        dataset_version: 0,
                                                        element: DataElement10_1 {
                                                        element_version: 0,
                                                        gp_value_version: 0,
                                                        gp_value_short: 0,
                                                        element_strings: Default::default(),
                                                        element_shorts: [0; 3],
                                                        element_legacy_xml: None,
                                                        raw_child_count: 0,
                                                        children: Vec::new(),
                                                        },
                                                        dataset_longs: [0; 2],
                                                        dataset_shorts: [0; 2],
                                                        dataset_string: None,
                                                    },
                                                    raw_short: 0,
                                                    strings: Default::default(),
                                                    raw_long: 0,
                                                    string_array: StringArraySlot10_1 {
                                                        entry: ObjectEntry10_1::Null,
                                                        body: None,
                                                    },
                                                    properties: RuleSlot10_1 {
                                                        entry: ObjectEntry10_1::Null,
                                                        body: None,
                                                    },
                                                    raw_subtype_count: 0,
                                                    subtypes: Vec::new(),
                                                    raw_membership_count: 0,
                                                    memberships: Vec::new(),
                                                    v2: None,
                                                    v3: None,
                                                    v4: None,
                                                    fields: RuleSlot10_1 {
                                                        entry: ObjectEntry10_1::Null,
                                                        body: None,
                                                    },
                                                    indexes: RuleSlot10_1 {
                                                        entry: ObjectEntry10_1::Null,
                                                        body: None,
                                                    },
                                                }),
                                            })),
                                        },
                                        RuleSlot10_1 {
                                            entry: ObjectEntry10_1::BackReference {
                                                class_id: feature_class_id,
                                                key: 28,
                                            },
                                            body: None,
                                        },
                                        RuleSlot10_1 {
                                            entry: ObjectEntry10_1::NewObject {
                                                class_id: DE_FEATURE_DATASET_CLASS_ID,
                                                key: 29,
                                            },
                                            body: Some(Box::new(DeChild10_1::FeatureDataset {
            vpf_base_version: None,
                                                version: 1,
                                                geo_dataset_version: 1,
                                                dataset: DeDataset10_1 {
                                                    dataset_version: 0,
                                                    element: DataElement10_1 {
                                                    element_version: 0,
                                                    gp_value_version: 0,
                                                    gp_value_short: 0,
                                                    element_strings: Default::default(),
                                                    element_shorts: [0; 3],
                                                    element_legacy_xml: None,
                                                    raw_child_count: 0,
                                                    children: Vec::new(),
                                                    },
                                                    dataset_longs: [0; 2],
                                                    dataset_shorts: [0; 2],
                                                    dataset_string: None,
                                                },
                                                extent: BookmarkEnvelopeEntry10_1::Null,
                                                spatial_reference: SpatialReferenceEntry10_1::Null,
                                            })),
                                        },
                                        RuleSlot10_1 {
                                            entry: ObjectEntry10_1::NewObject {
                                                class_id: DE_GEOMETRIC_NETWORK_CLASS_ID,
                                                key: 32,
                                            },
                                            body: Some(Box::new(DeChild10_1::GeometricNetwork {
                                                version: network_version,
                                                geo_dataset_version: 1,
                                                dataset: DeDataset10_1 {
                                                    dataset_version: 0,
                                                    element: DataElement10_1 {
                                                    element_version: 0,
                                                    gp_value_version: 0,
                                                    gp_value_short: 0,
                                                    element_strings: Default::default(),
                                                    element_shorts: [0; 3],
                                                    element_legacy_xml: None,
                                                    raw_child_count: 0,
                                                    children: Vec::new(),
                                                    },
                                                    dataset_longs: [0; 2],
                                                    dataset_shorts: [0; 2],
                                                    dataset_string: None,
                                                },
                                                extent: BookmarkEnvelopeEntry10_1::Null,
                                                spatial_reference: SpatialReferenceEntry10_1::Null,
                                                network: DeGeometricNetwork10_1 {
                                                    metadata: Some((
                                                        15,
                                                        "网络".into(),
                                                        StringArraySlot10_1 {
                                                            entry: ObjectEntry10_1::Null,
                                                            body: None,
                                                        },
                                                    )),
                                                    raw_counts: (network_version >= 3)
                                                        .then_some([2, 1, 1]),
                                                    rules: if network_version >= 3 {
                                                        vec![
                                                            RuleSlot10_1 {
                                                                entry: ObjectEntry10_1::NewObject { class_id: EDGE_CONNECTIVITY_RULE_CLASS_ID, key: 35 },
                                                                body: Some(Box::new(ConnectivityRule10_1::Edge {
                                                                    version: 1, name: "边".into(), longs: [0; 8], raw_count: 1, pairs: vec![[2, 3]],
                                                                })),
                                                            },
                                                            RuleSlot10_1 {
                                                                entry: ObjectEntry10_1::NewObject { class_id: JUNCTION_CONNECTIVITY_RULE_CLASS_ID, key: 36 },
                                                                body: Some(Box::new(ConnectivityRule10_1::Junction {
                                                                    version: 0, name: "交汇点".into(), longs: [0; 10], short: 4,
                                                                })),
                                                            },
                                                        ]
                                                    } else {
                                                        Vec::new()
                                                    },
                                                    weights: (network_version >= 3)
                                                        .then(|| RuleSlot10_1 {
                                                            entry: ObjectEntry10_1::NewObject {
                                                                class_id: NET_WEIGHT_CLASS_ID,
                                                                key: 33,
                                                            },
                                                            body: Some(Box::new(
                                                                NetWeight10_1::Weight {
                                                                    version: 1,
                                                                    raw_long: 7,
                                                                    name: "W".into(),
                                                                    trailing_longs: [8, 9],
                                                                },
                                                            )),
                                                        })
                                                        .into_iter()
                                                        .collect(),
                                                    associations: (network_version >= 3)
                                                        .then(|| RuleSlot10_1 {
                                                            entry: ObjectEntry10_1::NewObject {
                                                                class_id:
                                                                    NET_WEIGHT_ASSOCIATION_CLASS_ID,
                                                                key: 34,
                                                            },
                                                            body: Some(Box::new(
                                                                NetWeight10_1::Association {
                                                                    version: 0,
                                                                    raw_long: 7,
                                                                    strings: [
                                                                        "A".into(),
                                                                        "B".into(),
                                                                    ],
                                                                },
                                                            )),
                                                        })
                                                        .into_iter()
                                                        .collect(),

                                                    v4: (network_version >= 4).then_some((
                                                        5,
                                                        [1.5_f64.to_bits(), 2.5_f64.to_bits()],
                                                    )),
                                                    v5: (network_version >= 5).then_some([6, 7]),
                                                    v6: (network_version >= 6).then_some(8),
                                                },
                                            })),
                                        },
                                        RuleSlot10_1 {
                                            entry: ObjectEntry10_1::NewObject {
                                                class_id: DE_TOPOLOGY_CLASS_ID,
                                                key: 30,
                                            },
                                            body: Some(Box::new(DeChild10_1::Topology {
                                                version: 2,
                                                geo_dataset_version: 1,
                                                dataset: DeDataset10_1 {
                                                    dataset_version: 0,
                                                    element: DataElement10_1 {
                                                    element_version: 0,
                                                    gp_value_version: 0,
                                                    gp_value_short: 0,
                                                    element_strings: Default::default(),
                                                    element_shorts: [0; 3],
                                                    element_legacy_xml: None,
                                                    raw_child_count: 0,
                                                    children: Vec::new(),
                                                    },
                                                    dataset_longs: [0; 2],
                                                    dataset_shorts: [0; 2],
                                                    dataset_string: None,
                                                },
                                                extent: BookmarkEnvelopeEntry10_1::Null,
                                                spatial_reference: SpatialReferenceEntry10_1::Null,
                                                value_bits: [0.25_f64.to_bits(), 0.5_f64.to_bits()],
                                                raw_long: 42,
                                                names: StringArraySlot10_1 {
                                                    entry: ObjectEntry10_1::Null,
                                                    body: None,
                                                },
                                                raw_rule_count: Some(2),
                                                rules: vec![
                                                    RuleSlot10_1 {
                                                        entry: ObjectEntry10_1::NewObject {
                                                            class_id: TOPOLOGY_RULE_CLASS_ID,
                                                            key: 31,
                                                        },
                                                        body: Some(Box::new(TopologyRule10_1 {
                                                            version: 1,
                                                            base_version: 0,
                                                            base_name: "规则".into(),
                                                            base_longs: [1, 2],
                                                            strings: ["源".into(), "目标".into()],
                                                            longs: [3, 4, 5, 6],
                                                            shorts: [7, 8, 9, 10, 11],
                                                            trailing_long: 12,
                                                        })),
                                                    },
                                                    RuleSlot10_1 {
                                                        entry: ObjectEntry10_1::BackReference {
                                                            class_id: TOPOLOGY_RULE_CLASS_ID,
                                                            key: 31,
                                                        },
                                                        body: None,
                                                    },
                                                ],
                                            })),
                                        },
                                        RuleSlot10_1 {
                                            entry: ObjectEntry10_1::NewObject {
                                                class_id: DE_RELATIONSHIP_CLASS_CLASS_ID,
                                                key: 37,
                                            },
                                            body: Some(Box::new(DeChild10_1::RelationshipClass(
                                                Box::new(DeRelationshipClass10_1 {
                                                    version,
                                                    base: DeRelationshipBase10_1::Dataset(
                                                        DeDataset10_1 {
                                                            dataset_version: 0,
                                                            element: DataElement10_1 {
                                                            element_version: 0,
                                                            gp_value_version: 0,
                                                            gp_value_short: 0,
                                                            element_strings: Default::default(),
                                                            element_shorts: [0; 3],
                                                            element_legacy_xml: None,
                                                            raw_child_count: 0,
                                                            children: Vec::new(),
                                                            },
                                                            dataset_longs: [0; 2],
                                                            dataset_shorts: [0; 2],
                                                            dataset_string: None,
                                                        },
                                                    ),
                                                    metadata: (version >= 2).then(|| {
                                                        DeRelationshipMetadata10_1 {
                                                            names: std::array::from_fn(|_| {
                                                                StringArraySlot10_1 {
                                                                    entry: ObjectEntry10_1::Null,
                                                                    body: None,
                                                                }
                                                            }),
                                                            longs: [0; 4],
                                                            strings: ["正向".into(), "反向".into()],
                                                            shorts: [0, 1, 0],
                                                            raw_key_counts: [1; 2],
                                                            keys: [
                                                                vec![RuleSlot10_1 {
                                                                    entry: ObjectEntry10_1::NewObject { class_id: GP_RELATIONSHIP_CLASS_KEY_CLASS_ID, key: 38 },
                                                                    body: Some(Box::new(GpRelationshipClassKey10_1 {
                                                                        version: 1, raw_long: 3,
                                                                        strings: ["源键".into(), "目标键".into()],
                                                                    })),
                                                                }],
                                                                vec![RuleSlot10_1 {
                                                                    entry: ObjectEntry10_1::BackReference { class_id: GP_RELATIONSHIP_CLASS_KEY_CLASS_ID, key: 38 },
                                                                    body: None,
                                                                }],
                                                            ],
                                                        }
                                                    }),
                                                    raw_rule_count: (version >= 3).then_some(2),
                                                    rules: if version >= 3 {
                                                        vec![
                                                            RuleSlot10_1 {
                                                                entry: ObjectEntry10_1::NewObject { class_id: RELATIONSHIP_RULE_CLASS_ID, key: 39 },
                                                                body: Some(Box::new(RelationshipRule10_1 {
                                                                    version: 1, name: "关系规则".into(),
                                                                    raw_longs: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                                                                })),
                                                            },
                                                            RuleSlot10_1 {
                                                                entry: ObjectEntry10_1::BackReference { class_id: RELATIONSHIP_RULE_CLASS_ID, key: 39 },
                                                                body: None,
                                                            },
                                                        ]
                                                    } else { Vec::new() },
                                                    fields: (version >= 2).then_some(
                                                        RuleSlot10_1 {
                                                            entry: ObjectEntry10_1::BackReference {
                                                                class_id: FIELDS_CLASS_ID,
                                                                key: 21,
                                                            },
                                                            body: None,
                                                        },
                                                    ),
                                                    v6: None,
                                                }),
                                            ))),
                                        },
                                    ],
                                    },
                                    dataset_longs: [11, 14],
                                    dataset_shorts: [12, 13],
                                    dataset_string: Some("数据集".into()),
                                },
                                raw_short: 15,
                                strings: ["A", "B", "C", "D", "E", "F"].map(String::from),
                                raw_long: 16,
                                string_array: StringArraySlot10_1 {
                                    entry: ObjectEntry10_1::Null,
                                    body: None,
                                },
                                properties: RuleSlot10_1 {
                                    entry: ObjectEntry10_1::Null,
                                    body: None,
                                },
                                raw_subtype_count: 1,
                                subtypes: vec![RuleSlot10_1 {
                                    entry: ObjectEntry10_1::NewObject {
                                        class_id: GP_SUBTYPE_CLASS_ID,
                                        key: 25,
                                    },
                                    body: Some(Box::new(GpSubtype10_1 {
                                        version: 1,
                                        name: "子类型".into(),
                                        raw_code: 8,
                                        raw_count: 1,
                                        fields: vec![GpSubtypeField10_1 {
                                            strings: ["字段".into(), "域".into()],
                                            default: PropertyValueObjectStream10_1::Inline(
                                                PropertyValue10_1::Raw4(6_i32.to_le_bytes()),
                                            ),
                                        }],
                                    })),
                                }],
                                raw_membership_count: 2,
                                memberships: vec![
                                    RuleSlot10_1 {
                                        entry: ObjectEntry10_1::NewObject {
                                            class_id: GP_TOPOLOGY_MEMBERSHIP_CLASS_ID,
                                            key: 26,
                                        },
                                        body: Some(Box::new(
                                            GpControllerMembership10_1::Topology {
                                                version: 1,
                                                name: "拓扑".into(),
                                                value_bits: (-0.0_f64).to_bits(),
                                                raw_longs: [3, -4],
                                                raw_short: 7,
                                            },
                                        )),
                                    },
                                    RuleSlot10_1 {
                                        entry: ObjectEntry10_1::NewObject {
                                            class_id: GP_GEOMETRIC_NETWORK_MEMBERSHIP_CLASS_ID,
                                            key: 27,
                                        },
                                        body: Some(Box::new(
                                            GpControllerMembership10_1::GeometricNetwork {
                                                version: 0,
                                                strings: ["A", "B", "C"].map(String::from),
                                                raw_long: 16,
                                            },
                                        )),
                                    },
                                ],
                                v2: (version >= 2)
                                    .then(|| (17, std::array::from_fn(|_| "v2".into()))),
                                v3: (version >= 3)
                                    .then(|| (18, std::array::from_fn(|_| "v3".into()))),
                                v4: (version >= 4).then_some(19),
                                fields: RuleSlot10_1 {
                                    entry: ObjectEntry10_1::BackReference {
                                        class_id: FIELDS_CLASS_ID,
                                        key: 21,
                                    },
                                    body: None,
                                },
                                indexes: RuleSlot10_1 {
                                    entry: ObjectEntry10_1::NewObject {
                                        class_id: INDEXES_CLASS_ID,
                                        key: 23,
                                    },
                                    body: Some(Box::new(DeTableIndexes10_1 {
                                        version: 1,
                                        raw_count: 1,
                                        entries: vec![RuleSlot10_1 {
                                            entry: ObjectEntry10_1::NewObject {
                                                class_id: INDEX_CLASS_ID,
                                                key: 24,
                                            },
                                            body: Some(Box::new(DeTableIndex10_1 {
                                                version: 1,
                                                gp_value_version: 1,
                                                gp_value_short: 7,
                                                name: "索引".into(),
                                                flags: [1, 0, 1],
                                                fields: RuleSlot10_1 {
                                                    entry: ObjectEntry10_1::BackReference {
                                                        class_id: FIELDS_CLASS_ID,
                                                        key: 21,
                                                    },
                                                    body: None,
                                                },
                                            })),
                                        }],
                                    })),
                                },
                            })),
                        },
                    }],
                    legacy_byte: (version == 1).then_some(0),
                    properties: (version >= 2).then_some(RuleSlot10_1 {
                        entry: ObjectEntry10_1::Null,
                        body: None,
                    }),
                    variant_flag: (version >= 3).then_some(0),
                    variant: None,
                    table: None,
                })),
            },
        );
    }
}
