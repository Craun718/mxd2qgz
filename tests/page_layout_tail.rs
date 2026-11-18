use mxd2qgz::{
    maps::{mxd_maps::parse_mxd_streams_10_1, page_layout_tail::read_page_layout_tail_10_1},
    storage::object_refs::{ObjectEntry10_1, RefTable10_1},
    stream::reader::Reader,
};
use std::io::Read;

fn layout_tail(version: u16, n: u32) -> Vec<u8> {
    let page_class = [
        0x6f, 0xd7, 0x94, 0xdd, 0x6d, 0x83, 0xd0, 0x11, 0x87, 0xec, 0x08, 0x00, 0x09, 0xec, 0x73,
        0x2a,
    ];
    let mut bytes = page_class.to_vec();
    bytes.extend(9u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(12u32.to_le_bytes());
    bytes.extend([0; 4]);
    bytes.extend(1u16.to_le_bytes());
    bytes.extend([0; 4]);
    bytes.extend(8.5f64.to_le_bytes());
    bytes.extend(11.0f64.to_le_bytes());
    bytes.extend([0; 32 + 4 + 3 * 16]); // 可打印范围、两个 short 和 Page 的三个对象槽。
    bytes.extend([0; 4 * 16]); // 标尺、网格和两组辅助线。
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(0.2f64.to_le_bytes());
    bytes.extend([0; 8]); // 两组 GUID 数量。
    bytes.extend([0; 16 + 8 + 64]); // Map、short×4、视图和版面范围。
    for (first, size) in [
        (2, 8),
        (3, 2),
        (4, 8),
        (5, 2),
        (6, 4),
        (8, 2),
        (9, 4),
        (10, 4),
        (11, 8),
    ] {
        if version >= first {
            bytes.resize(bytes.len() + size, 0);
        }
    }
    if matches!(version, 12 | 13) {
        bytes.extend([0; 16]);
    }
    if version >= 13 {
        bytes.extend([0; 4]);
        bytes.extend(n.to_le_bytes());
        for value in 0..n {
            bytes.extend(value.to_le_bytes());
        }
    }
    if version >= 15 {
        for _ in 0..n {
            bytes.extend([0; 16]); // 空锚定范围。
            bytes.extend(1.0f64.to_le_bytes());
        }
    }
    bytes
}

#[test]
fn page_layout_tail_version_gates_leave_outer_flag_unread() {
    for version in 0..=15 {
        let mut bytes = layout_tail(version, 2);
        let end = bytes.len();
        bytes.extend(0x1234u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let tail = read_page_layout_tail_10_1(&mut reader, &mut refs, version, 2, 8192, |_| false)
            .unwrap();
        let page = tail.page.body.unwrap();
        assert_eq!(
            (page.paper_id, page.orientation, page.size),
            (12, 1, [8.5, 11.0])
        );
        assert_eq!(tail.version_9_shorts.is_some(), version >= 9);
        assert_eq!(
            tail.version_11_slot,
            (version >= 11).then_some(ObjectEntry10_1::Null)
        );
        assert_eq!(
            tail.legacy_version_12_or_13_doubles.is_some(),
            matches!(version, 12 | 13)
        );
        assert_eq!(tail.element_values.len(), if version >= 13 { 2 } else { 0 });
        assert_eq!(tail.anchors.len(), if version >= 15 { 2 } else { 0 });
        assert_eq!(tail.end_offset, end);
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
        assert!(refs.contains(9));
    }
}

#[test]
fn truncated_or_inconsistent_layout_tail_rolls_back_page_registration() {
    let bytes = layout_tail(15, 2);
    let mut refs = RefTable10_1::new();
    refs.register(23);
    let saved = refs.clone();
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(
            read_page_layout_tail_10_1(&mut reader, &mut refs, 15, 2, 8192, |_| false).is_err(),
            "位置 {end}"
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, saved);
    }
    let mut reader = Reader::new(&bytes);
    assert!(read_page_layout_tail_10_1(&mut reader, &mut refs, 15, 1, 8192, |_| false).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, saved);
}

#[test]
fn reference_document_reads_group_and_leaves_nonempty_print_record_at_exact_boundary() {
    use mxd2qgz::maps::group_element::GroupChild10_1;
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut read = |name: &str| {
        let mut bytes = Vec::new();
        document
            .open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = read("/Maps");
    let mut page = read("/PageLayout");
    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 7, "{:?}", layout.stop);
    let group = layout
        .groups
        .iter()
        .find(|group| group.key == 2208)
        .unwrap();
    assert_eq!(group.version, 3);
    assert_eq!(group.children.len(), 2);
    assert!(matches!(group.children[0], GroupChild10_1::Rectangle(_)));
    assert!(group.geometry.as_ref().unwrap().1.is_some());
    assert_eq!(layout.tail.as_ref().unwrap().end_offset, 0x1c62);
    assert_eq!(layout.print_settings_flag, Some(0));
    assert_eq!(layout.consumed_end_offset(), page.len());
    assert!(
        !report
            .map
            .unknown_fields
            .iter()
            .any(|field| field.stream == "/PageLayout")
    );

    page[0x1c62..0x1c64].copy_from_slice(&1u16.to_le_bytes());
    page.extend([0x30, 0x4f, 0x25]);
    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.print_settings_flag, Some(1));
    assert!(layout.tail_error.as_deref().unwrap().contains("IPaper"));
    let unknown = report
        .map
        .unknown_fields
        .iter()
        .find(|field| field.stream == "/PageLayout")
        .unwrap();
    assert_eq!(unknown.offset, 0x1c64);
    assert_eq!(unknown.bytes, [0x30, 0x4f, 0x25]);
}
