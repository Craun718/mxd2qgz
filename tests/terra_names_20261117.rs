//! AngleFormat 核心版本失败路径的合成输入检查。

use mxd2qgz::objects::number_format::read_angle_format_body_10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn angle_consumes_only_rejected_core_version_before_the_outer_tail() {
    for version in [2, 3] {
        let mut bytes = vec![version, 0, 7, 0, 2, 0];
        if version == 3 {
            bytes.extend(9_i16.to_le_bytes());
        }
        let next = bytes.len();
        bytes.push(0xab);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(42);

        let angle = read_angle_format_body_10_1(&mut reader, &mut table, 99).unwrap();
        assert_eq!(angle.rejected_numeric_version, Some(2));
        assert!(angle.numeric.is_none());
        assert_eq!(angle.short_0a, (version == 3).then_some(9));
        assert_eq!(reader.position(), next);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xab]);

        let mut truncated = Reader::new(&bytes[..next - 1]);
        assert!(read_angle_format_body_10_1(&mut truncated, &mut table, 99).is_err());
        assert_eq!(truncated.position(), 0);
        assert_eq!(table.len(), 1);
        assert!(table.contains(42));
    }
}
