//! 旧版 Legend 分支的读取器契约；构造输入不代表真实 MXD 样本。

use mxd2qgz::maps::map_surround::{LegendStop10_1, read_legend_prefix_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn empty_string(bytes: &mut Vec<u8>) {
    bytes.extend(2_u32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
}

fn old_legend(version: u16) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(0_u16.to_le_bytes());
    empty_string(&mut bytes);
    bytes.extend([0_u8; 16]);
    bytes.extend([0_u8; 16]);
    empty_string(&mut bytes);
    if version == 1 {
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.extend((-2.25_f64).to_le_bytes());
    }
    for value in [11_u16, 22, 33, 0xffff] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

#[test]
fn old_legend_versions_preserve_field_boundaries() {
    for version in [0, 1] {
        let mut bytes = old_legend(version);
        let end = bytes.len();
        bytes.extend([0xa5, 0x5a]);
        let mut reader = Reader::new(&bytes);
        let body =
            read_legend_prefix_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new()).unwrap();
        assert_eq!(body.stop, LegendStop10_1::Completed);
        assert_eq!(body.shorts, [11, 22, 33]);
        assert_eq!(body.selection_short, 0xffff);
        assert_eq!(body.legacy_doubles, (version == 1).then_some([1.5, -2.25]));
        assert_eq!(reader.position(), end);
        assert_eq!(body.remaining_bytes, 2);
    }
}

#[test]
fn old_legend_truncations_leave_reader_and_references_unchanged() {
    for version in [0, 1] {
        let bytes = old_legend(version);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            table.register(17);
            assert!(read_legend_prefix_10_1(&mut reader, usize::MAX, &mut table).is_err());
            assert_eq!(reader.position(), 0, "版本 {version} 截断位置 {end}");
            assert_eq!(table.len(), 1, "版本 {version} 截断位置 {end}");
            assert!(table.contains(17));
        }
    }
}
