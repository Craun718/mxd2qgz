//! 图形容器已知实现类在未闭合体时的保守停止契约；输入为构造数据。

use mxd2qgz::layers::annotate_props::{LelpStop10_1, read_label_engine_props_body_10_1};
use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn lelp_prefix() -> Vec<u8> {
    let mut data = 3_u16.to_le_bytes().to_vec();
    data.extend(2_u32.to_le_bytes());
    data.extend([0; 2]);
    data.extend([0; 2 + 8 + 4 + 2 * 3 + 4 + 8 * 2 + 2]);
    for _ in 0..2 {
        data.extend(2_u32.to_le_bytes());
        data.extend([0; 2]);
    }
    data.extend([0; 16 * 4]);
    data.extend([0; 8 * 2 + 4 * 2]);
    data
}

#[test]
fn page_layout_container_stops_before_unclosed_body() {
    let mut data = lelp_prefix();
    let slot_offset = data.len();
    data.extend(PAGE_LAYOUT_CLASS_ID);
    data.extend(123_u32.to_le_bytes());
    data.extend(7_u16.to_le_bytes());
    data.extend(0_u32.to_le_bytes());
    data.extend(1_u32.to_le_bytes());
    data.extend(20_u32.to_le_bytes());
    data.extend(0_u32.to_le_bytes());
    data.extend([0x5a; 16]);
    data.extend(124_u32.to_le_bytes());
    let mut reader = Reader::new(&data);
    let mut table = RefTable10_1::new();
    let body = read_label_engine_props_body_10_1(&mut reader, 4096, &mut table, 1).unwrap();
    assert_eq!(reader.position(), slot_offset);
    assert!(!table.contains(123));
    assert!(!table.contains(124));
    assert!(matches!(
        body.stop,
        Some(LelpStop10_1::ContainerBodyUnsupported {
            class_id: PAGE_LAYOUT_CLASS_ID,
            key: 123,
            ..
        })
    ));
}

#[test]
fn truncated_container_header_rolls_back_body_and_references() {
    let mut data = lelp_prefix();
    let slot_offset = data.len();
    data.extend(PAGE_LAYOUT_CLASS_ID);
    data.extend(123_u32.to_le_bytes());
    for end in slot_offset..data.len() {
        let mut reader = Reader::new(&data[..end]);
        let mut table = RefTable10_1::new();
        assert!(read_label_engine_props_body_10_1(&mut reader, 4096, &mut table, 1).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
}
