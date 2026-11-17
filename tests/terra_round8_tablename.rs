use mxd2qgz::{
    maps::{
        map_object::{MAP_CLASS_ID, read_map_suffix_10_1},
        standalone_table::{
            StandaloneNameBody10_1, StandaloneTableStop10_1, TABLE_NAME_CLASS_ID,
            read_standalone_table_prefix_10_1,
        },
    },
    objects::workspace_name::WorkspaceNameEntry10_1,
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
    assert_eq!(r.position(), 0x8407);
}
