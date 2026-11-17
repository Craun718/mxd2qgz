//! 合成输入验证旧版几何体的游标与事务性，不代表目标软件运行验证。
use mxd2qgz::objects::geometry::{
    MULTIPOINT_CLASS_ID, POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID,
    read_geometry_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn legacy_versions_leave_following_bytes_and_reference_table_untouched() {
    for class in [
        POLYLINE_CLASS_ID,
        POLYGON_CLASS_ID,
        POINT_CLASS_ID,
        MULTIPOINT_CLASS_ID,
    ] {
        for version in [0_u16, 1] {
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend_from_slice(&4_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u32.to_le_bytes());
            bytes.extend_from_slice(b"next");
            let mut reader = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            table.register(42);
            read_geometry_body_10_1(&class, &mut reader, 4, &mut table).unwrap();
            assert_eq!(reader.position(), 10);
            assert_eq!(reader.read_bytes(4).unwrap(), b"next");
            assert_eq!(table.len(), 1);
            assert!(table.contains(42));
            for end in 0..10 {
                let mut reader = Reader::new(&bytes[..end]);
                let mut table = RefTable10_1::new();
                table.register(42);
                assert!(read_geometry_body_10_1(&class, &mut reader, 4, &mut table).is_err());
                assert_eq!(reader.position(), 0);
                assert_eq!(table.len(), 1);
                assert!(table.contains(42));
            }
        }
    }
}
