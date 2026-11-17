use mxd2qgz::layers::composite_graphics::{
    CompositeGraphicsLayerStop10_1, ElementEntry10_1, read_composite_graphics_layer_prefix_10_1,
};
use mxd2qgz::objects::element_body::{
    LINE_ELEMENT_CLASS_ID, MARKER_ELEMENT_CLASS_ID, TEXT_ELEMENT_CLASS_ID,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn string() -> Vec<u8> {
    [2_u32.to_le_bytes().as_slice(), &[0, 0]].concat()
}

fn input(class: [u8; 16], unknown_symbol: bool) -> (Vec<u8>, usize) {
    let mut b = 0_u16.to_le_bytes().to_vec();
    b.extend(1_u32.to_le_bytes());
    b.extend(1_u32.to_le_bytes());
    b.extend(class);
    b.extend(91_u32.to_le_bytes());
    b.extend(0_u16.to_le_bytes()); // 元素版本
    b.extend(4_u16.to_le_bytes()); // ElementProperties 版本
    b.extend(string());
    b.extend(string());
    b.extend(0_u16.to_le_bytes()); // VT_EMPTY
    b.extend(0_u16.to_le_bytes());
    if class == TEXT_ELEMENT_CLASS_ID {
        b.extend(string());
        b.extend(0_u32.to_le_bytes());
    }
    let symbol_offset = b.len();
    if unknown_symbol {
        b.extend([0x55; 16]);
        b.extend(92_u32.to_le_bytes());
        return (b, symbol_offset);
    }
    b.extend([0; 32]); // 空符号和空几何
    b.extend(string()); // CGL 名称
    b.extend([0; 6 + 16 + 12]);
    b.extend([0; 16]); // ElementCollection 槽
    b.extend(0_u32.to_le_bytes()); // 子图层数
    b.extend(0_u32.to_le_bytes());
    b.extend([0; 16]); // 尾段 ILayer
    (b, symbol_offset)
}

#[test]
fn element_group_dispatches_three_classes_and_rolls_back_truncation() {
    for class in [
        TEXT_ELEMENT_CLASS_ID,
        MARKER_ELEMENT_CLASS_ID,
        LINE_ELEMENT_CLASS_ID,
    ] {
        let (bytes, _) = input(class, false);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
        assert_eq!(parsed.stop, CompositeGraphicsLayerStop10_1::Completed);
        assert!(matches!(
            parsed.element_groups[0].elements[0],
            ElementEntry10_1::Element { key: 91, .. }
        ));
        assert_eq!(reader.position(), bytes.len());
        assert!(table.contains(91));
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            assert!(
                read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).is_err(),
                "end={end}"
            );
            assert_eq!(reader.position(), 0);
            assert!(!table.contains(91));
        }
    }
}

#[test]
fn element_inner_stop_keeps_body_and_does_not_register_unknown_symbol() {
    for class in [
        TEXT_ELEMENT_CLASS_ID,
        MARKER_ELEMENT_CLASS_ID,
        LINE_ELEMENT_CLASS_ID,
    ] {
        let (bytes, offset) = input(class, true);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_composite_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
        assert!(matches!(
            parsed.stop,
            CompositeGraphicsLayerStop10_1::ElementBodyIncomplete {
                group: 0,
                index: 0,
                key: 91,
                ..
            }
        ));
        assert!(matches!(
            parsed.element_groups[0].elements[0],
            ElementEntry10_1::Element { key: 91, .. }
        ));
        assert_eq!(reader.position(), offset);
        assert!(table.contains(91));
        assert!(!table.contains(92));
    }
}
