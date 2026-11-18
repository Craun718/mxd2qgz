use mxd2qgz::{
    layers::unique_value_renderer::{UniqueValueRendererStop10_1, read_unique_value_renderer_10_1},
    storage::object_refs::RefTable10_1,
    stream::reader::Reader,
};

fn string(bytes: &mut Vec<u8>, value: &str) {
    let units: Vec<_> = value.encode_utf16().chain([0]).collect();
    bytes.extend((units.len() as u32 * 2).to_le_bytes());
    bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
}

fn renderer_v1() -> (Vec<u8>, usize) {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    bytes.extend(1u32.to_le_bytes());
    string(&mut bytes, "类型");
    bytes.extend([1, 0, 0]);
    string(&mut bytes, ", ");
    bytes.extend([0; 16]); // 默认符号。
    bytes.extend(0u32.to_le_bytes());
    let item = bytes.len();
    bytes.extend([0; 16]);
    bytes.extend(2u32.to_le_bytes());
    for (value, label) in [("A", Some("甲")), ("B", None)] {
        string(&mut bytes, value);
        bytes.extend(12u32.to_le_bytes());
        bytes.extend(34u32.to_le_bytes());
        bytes.extend(u32::from(label.is_some()).to_le_bytes());
        if let Some(label) = label {
            string(&mut bytes, label);
        }
    }
    bytes.extend(7u16.to_le_bytes());
    string(&mut bytes, "ANGLE");
    bytes.extend(2u32.to_le_bytes());
    for value in ["first", "second", "third"] {
        string(&mut bytes, value);
    }
    bytes.extend(1u16.to_le_bytes());
    (bytes, item)
}

#[test]
fn v1_unique_values_preserve_order_and_set_no_label_bytes() {
    let (mut bytes, _) = renderer_v1();
    let end = bytes.len();
    bytes.push(0x56);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    refs.register(5);
    let result = read_unique_value_renderer_10_1(&mut reader, 1024, &mut refs, 5).unwrap();
    assert!(result.stop.is_none());
    assert_eq!(result.fields, [("类型".into(), [1, 0, 0])]);
    assert_eq!(result.delimiter, ", ");
    assert_eq!(result.values[0].value, "A");
    assert_eq!(result.values[0].raw_values, [12, 34]);
    assert_eq!(result.values[0].optional_string.as_deref(), Some("甲"));
    assert_eq!(result.values[1].optional_string, None);
    let tail = result.tail.unwrap();
    assert_eq!(tail.rotation_field, "ANGLE");
    assert_eq!(tail.rotation_type, 2);
    assert!(tail.default_label.is_none());
    assert!(tail.rotation.is_none());
    assert!(tail.color_ramp.is_none());
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), &[0x56]);

    for cut in 0..end {
        let mut reader = Reader::new(&bytes[..cut]);
        let before = refs.clone();
        assert!(
            read_unique_value_renderer_10_1(&mut reader, 1024, &mut refs, 5).is_err(),
            "截断位置 {cut}"
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);
    }
}

#[test]
fn unknown_legend_item_is_preserved_without_registering_its_key() {
    let (bytes, offset) = renderer_v1();
    let mut bytes = bytes[..offset].to_vec();
    let class_id = [0x71; 16];
    bytes.extend(class_id);
    bytes.extend(23u32.to_le_bytes());
    bytes.push(0x78);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let body = read_unique_value_renderer_10_1(&mut reader, 1024, &mut refs, 5).unwrap();
    assert_eq!(
        body.stop,
        Some(UniqueValueRendererStop10_1::LegendItem { class_id, key: 23 })
    );
    assert_eq!(reader.position(), offset);
    assert!(refs.is_empty());
}
