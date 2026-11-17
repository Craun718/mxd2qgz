use mxd2qgz::{
    maps::{
        map_surround::{BookmarkNameEntry10_1, read_feature_bookmark_10_1},
        standalone_table::TABLE_NAME_CLASS_ID,
    },
    objects::workspace_name::WorkspaceNameEntry10_1,
    storage::object_refs::RefTable10_1,
    stream::reader::Reader,
};

fn fixture(workspace: &[u8]) -> Vec<u8> {
    let mut b = 1u16.to_le_bytes().to_vec();
    b.extend(42i32.to_le_bytes());
    b.extend(TABLE_NAME_CLASS_ID);
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
