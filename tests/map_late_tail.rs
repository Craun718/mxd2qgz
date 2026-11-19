use mxd2qgz::maps::map_late_tail::{LateMapValue10_1, read_map_late_tail_10_1};
use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

#[test]
fn reference_map_reads_scale_list_and_leaves_root_trailer() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut reader = Reader::new(&bytes);
    reader.read_bytes(0x253e1).unwrap();
    let result = read_map_late_tail_10_1(&mut reader, &mut RefTable10_1::new(), 4096, 51).unwrap();
    assert!(result.stop.is_none(), "{:?}", result.stop);
    let list = result.fields.iter().find(|f| f.offset == 0x2542b).unwrap();
    let LateMapValue10_1::DoubleListBits(values) = &list.value else {
        panic!("此处应为比例尺列表");
    };
    assert_eq!(values.len(), 17);
    assert_eq!(f64::from_bits(values[0]), 1000.0);
    assert_eq!(f64::from_bits(values[16]), 3_000_000.0);
    assert_eq!(reader.position(), 0x25569);
    assert_eq!(reader.read_u32_le().unwrap(), 1);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reads_version_gates_without_consuming_following_bytes() {
    // 对象槽为空，所有字符串使用 10.1 普通空串编码。
    // 分别构造几个易混淆的版本分支，长度含各自的空字符串与空对象。
    for (version, size, strings) in [
        (34, 74, &[][..]),
        (35, 74, &[][..]),
        (36, 78, &[][..]),
        (38, 150, &[][..]),
        (39, 188, &[170][..]),
        (42, 212, &[170][..]),
        (43, 216, &[170, 210][..]),
        (45, 232, &[170, 208][..]),
        (49, 242, &[170, 208][..]),
        (50, 238, &[170, 208][..]),
        (51, 256, &[170, 208][..]),
    ] {
        let mut fixture = vec![0; size];
        for &offset in strings {
            fixture[offset] = 2;
        }
        fixture.extend(0xdeadbeef_u32.to_le_bytes());
        let mut reader = Reader::new(&fixture);
        let result =
            read_map_late_tail_10_1(&mut reader, &mut RefTable10_1::new(), 4096, version).unwrap();
        assert!(result.stop.is_none(), "v{version}: {:?}", result.stop);
        assert_eq!(reader.position(), size, "v{version}");
        assert_eq!(reader.read_u32_le().unwrap(), 0xdeadbeef);
    }
}

#[test]
fn unknown_object_and_incomplete_envelope_stop_before_the_header() {
    let mut bytes = vec![0; 12];
    bytes.extend([0x42; 16]);
    bytes.extend(91_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_map_late_tail_10_1(&mut reader, &mut refs, 1024, 34).unwrap();
    assert!(result.stop.unwrap().contains("新对象"));
    assert_eq!(reader.position(), 12);
    assert!(!refs.contains(91));
    bytes[12..28].copy_from_slice(&ENVELOPE_CLASS_ID);
    bytes.extend(2_u16.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let result = read_map_late_tail_10_1(&mut reader, &mut refs, 1024, 34).unwrap();
    assert!(result.stop.is_some());
    assert_eq!(reader.position(), 12);
    assert!(!refs.contains(91));

    // v2 Envelope 含版本、四个 double 及属性标志。
    bytes.extend([0; 36]);
    bytes.extend([0; 46]);
    let mut reader = Reader::new(&bytes);
    let result = read_map_late_tail_10_1(&mut reader, &mut refs, 1024, 34).unwrap();
    assert!(result.stop.is_none(), "{:?}", result.stop);
    assert!(matches!(
        result.fields[2].value,
        LateMapValue10_1::Object {
            entry: ObjectEntry10_1::NewObject { key: 91, .. },
            envelope: Some(_)
        }
    ));
    assert!(refs.contains(91));
}

#[test]
fn count_is_bounded_and_truncation_does_not_consume_the_array() {
    for (count, available) in [(-1_i32, 8), (i32::MAX, 8), (1, 7)] {
        let mut bytes = vec![0; 74];
        bytes.extend(count.to_le_bytes());
        bytes.extend(vec![0; available]);
        let mut reader = Reader::new(&bytes);
        let result =
            read_map_late_tail_10_1(&mut reader, &mut RefTable10_1::new(), 32, 36).unwrap();
        assert!(result.stop.unwrap().contains("集合"));
        assert_eq!(reader.position(), 74);
    }
}

#[test]
fn map_v51_reads_paragraph_element_and_keeps_next_root_field() {
    use mxd2qgz::objects::element_body::PARAGRAPH_TEXT_ELEMENT_CLASS_ID;

    let mut fixture = vec![0; 240];
    fixture[170] = 2;
    fixture[208] = 2;
    let slot_at = fixture.len();
    fixture.extend(PARAGRAPH_TEXT_ELEMENT_CLASS_ID);
    fixture.extend(81_u32.to_le_bytes());
    fixture.extend(1_u16.to_le_bytes());
    fixture.extend([0; 3 * 16 + 8 + 4 + 8]); // 空装饰槽及其后标量。
    fixture.extend(0_u16.to_le_bytes()); // TextElement v0。
    fixture.extend(4_u16.to_le_bytes()); // ElementProperties v4。
    fixture.extend([2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0]);
    fixture.extend([0; 4]); // VT_EMPTY 与 short。
    fixture.extend([4, 0, 0, 0, b'A', 0, 0, 0]);
    fixture.extend([0; 4 + 16 + 16]); // 标量、空符号及空几何槽。
    let end = fixture.len();
    fixture.extend(0xdeadbeef_u32.to_le_bytes());

    let mut reader = Reader::new(&fixture);
    let mut refs = RefTable10_1::new();
    let result = read_map_late_tail_10_1(&mut reader, &mut refs, 4096, 51).unwrap();
    assert!(result.stop.is_none(), "{:?}", result.stop);
    let Some(LateMapValue10_1::ParagraphTextElement { entry, body }) =
        result.fields.last().map(|field| &field.value)
    else {
        panic!("缺少 Map v51 段落文本元素");
    };
    assert!(matches!(entry, ObjectEntry10_1::NewObject { key: 81, .. }));
    assert_eq!(body.text_element.as_ref().unwrap().text, "A");
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_u32_le().unwrap(), 0xdeadbeef);
    assert!(refs.contains(81));

    for cut in slot_at..end {
        let mut reader = Reader::new(&fixture[..cut]);
        let mut refs = RefTable10_1::new();
        let result = read_map_late_tail_10_1(&mut reader, &mut refs, 4096, 51).unwrap();
        assert!(result.stop.is_some());
        assert_eq!(reader.position(), slot_at);
        assert!(refs.is_empty());
    }
}
