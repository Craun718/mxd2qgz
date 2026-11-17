use mxd2qgz::layers::legend_format::{
    LegendClassFormatSlot10_1, LegendClassFormatStop10_1, read_legend_class_format_body_10_1,
};
use mxd2qgz::objects::element_body::TEXT_SYMBOL_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn text_symbol() -> Vec<u8> {
    let mut b = 0_u16.to_le_bytes().to_vec();
    b.extend([0; 16 + 12 + 2 + 24 + 8 + 16 + 16 + 8 + 24 + 1 + 8 + 4 + 8 + 1 + 16 + 16 + 16]);
    b.extend(2_u32.to_le_bytes());
    b.extend([0_u8; 2]);
    b.extend([0; 8 + 4 + 8 + 16 + 16]);
    b
}

fn input() -> Vec<u8> {
    let mut b = 2_u16.to_le_bytes().to_vec();
    b.extend(TEXT_SYMBOL_CLASS_ID);
    b.extend(42_u32.to_le_bytes());
    b.extend(text_symbol());
    b.extend([0; 16 * 3 + 16]);
    b
}

#[test]
fn text_symbol_is_consumed_and_remaining_fields_are_read() {
    let b = input();
    let mut reader = Reader::new(&b);
    let mut table = RefTable10_1::new();
    let body = read_legend_class_format_body_10_1(&mut reader, 1024, &mut table, 1).unwrap();
    assert!(matches!(
        body.label_symbol,
        LegendClassFormatSlot10_1::TextSymbol(_)
    ));
    assert!(body.stop.is_none());
    assert_eq!(body.patch_doubles, Some([0.0, 0.0]));
    assert!(table.contains(42));
    assert_eq!(reader.position(), b.len());
}

#[test]
fn truncated_symbol_rolls_back_and_nested_unknown_stops() {
    let b = input();
    for end in 0..b.len() {
        let mut reader = Reader::new(&b[..end]);
        let mut table = RefTable10_1::new();
        assert!(
            read_legend_class_format_body_10_1(&mut reader, 1024, &mut table, 1).is_err(),
            "{end}"
        );
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
    let mut b = 2_u16.to_le_bytes().to_vec();
    b.extend(TEXT_SYMBOL_CLASS_ID);
    b.extend(42_u32.to_le_bytes());
    b.extend(0_u16.to_le_bytes());
    b.extend([0x11; 16]);
    b.extend(99_u32.to_le_bytes());
    let mut reader = Reader::new(&b);
    let mut table = RefTable10_1::new();
    let body = read_legend_class_format_body_10_1(&mut reader, 1024, &mut table, 1).unwrap();
    assert_eq!(body.stop, Some(LegendClassFormatStop10_1::TextSymbolBody));
    assert_eq!(reader.position(), 24);
    assert!(table.contains(42));
    assert!(!table.contains(99));
}
