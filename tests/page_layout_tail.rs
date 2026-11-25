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

#[test]
fn nonempty_print_settings_handle_paper_and_each_printer_body() {
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;
    use mxd2qgz::maps::print_settings::{
        ARC_PRESS_PRINTER_CLASS_ID, EMF_PRINTER_CLASS_ID, PS_PRINTER_CLASS_ID, PrinterBody10_1,
    };

    let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
    page.extend(1_u32.to_le_bytes());
    page.extend(15_u16.to_le_bytes());
    page.extend([0; 8]);
    page.extend(layout_tail(15, 0));
    page.extend(1_u16.to_le_bytes());
    let settings_start = page.len();
    page.extend(3_u16.to_le_bytes());
    page.extend(0x804_u16.to_le_bytes());
    page.extend(4_u32.to_le_bytes());
    page.extend("纸\0".encode_utf16().flat_map(u16::to_le_bytes));
    page.extend(8_u32.to_le_bytes());
    page.extend([0; 8]);
    let mode_size_offset = page.len();
    page.extend(223_u32.to_le_bytes());
    let mut dev_mode = [0; 223];
    dev_mode[0x44..0x46].copy_from_slice(&220_u16.to_le_bytes());
    dev_mode[0x46..0x48].copy_from_slice(&3_u16.to_le_bytes());
    dev_mode[220..].copy_from_slice(&[1, 2, 3]);
    page.extend(dev_mode);
    let paper_end = page.len();

    for class in [
        EMF_PRINTER_CLASS_ID,
        PS_PRINTER_CLASS_ID,
        ARC_PRESS_PRINTER_CLASS_ID,
    ] {
        page.truncate(paper_end);
        page.extend(class);
        page.extend(7_u32.to_le_bytes());
        if class == ARC_PRESS_PRINTER_CLASS_ID {
            page.extend(2_u16.to_le_bytes());
            page.extend(4_u32.to_le_bytes());
            page.extend("甲\0".encode_utf16().flat_map(u16::to_le_bytes));
            page.extend(8_u16.to_le_bytes());
            page.push(1);
            page.extend(1_u32.to_le_bytes()); // BSTR 的长度以码元计。
            page.extend("X\0".encode_utf16().flat_map(u16::to_le_bytes));
            page.extend(3_u32.to_le_bytes());
            page.extend([1, 5, 9]);
            for value in [-10_i32, 20, 30, 40, 50] {
                page.extend(value.to_le_bytes());
            }
            page.push(0);
        } else {
            page.extend(1_u16.to_le_bytes());
        }
        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.consumed_end_offset(), page.len());
        assert!(layout.tail_error.is_none());
        let settings = layout.print_settings.unwrap();
        assert_eq!(settings.paper.version, 3);
        assert_eq!(settings.paper.system_language, Some(0x804));
        assert_eq!(settings.paper.printer_description.as_deref(), Some("纸"));
        assert_eq!(settings.paper.dev_names, [0; 8]);
        assert_eq!(settings.paper.dev_mode, dev_mode);
        assert_eq!(
            settings.printer,
            ObjectEntry10_1::NewObject {
                class_id: class,
                key: 7
            }
        );
        match (class, settings.printer_body.unwrap()) {
            (EMF_PRINTER_CLASS_ID, PrinterBody10_1::Emf { version: 1 })
            | (PS_PRINTER_CLASS_ID, PrinterBody10_1::PostScript { version: 1 }) => {}
            (ARC_PRESS_PRINTER_CLASS_ID, PrinterBody10_1::ArcPress(printer)) => {
                assert_eq!(printer.name, "甲");
                assert_eq!(printer.short, 8);
                assert_eq!(printer.first_flag, 1);
                assert_eq!(printer.driver_class, "X");
                assert_eq!(printer.driver_data.as_deref(), Some([1, 5, 9].as_slice()));
                assert_eq!(printer.longs, [-10, 20, 30, 40, 50]);
                assert_eq!(printer.last_flag, 0);
            }
            other => panic!("未读出预期的打印机：{other:?}"),
        }
    }

    for end in settings_start..page.len() {
        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page[..end])).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.consumed_end_offset(), settings_start, "位置 {end}");
        assert!(layout.tail_error.is_some());
    }
    page[mode_size_offset..mode_size_offset + 4].copy_from_slice(&219_u32.to_le_bytes());
    let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_end_offset(), settings_start);
    assert!(layout.tail_error.unwrap().contains("DEVMODE"));
    assert_eq!(report.map.unknown_fields[0].bytes, page[settings_start..]);
}

#[test]
fn print_settings_legacy_paper_accepts_empty_and_populated_dev_mode() {
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;
    use mxd2qgz::maps::print_settings::{EMF_PRINTER_CLASS_ID, PrinterBody10_1};

    let mut prefix = PAGE_LAYOUT_CLASS_ID.to_vec();
    prefix.extend(1_u32.to_le_bytes());
    prefix.extend(15_u16.to_le_bytes());
    prefix.extend([0; 8]);
    prefix.extend(layout_tail(15, 0));
    prefix.extend(0xffff_u16.to_le_bytes());
    for version in 0..=3_u16 {
        let mut page = prefix.clone();
        page.extend(version.to_le_bytes());
        if version >= 2 {
            page.extend(0x409_u16.to_le_bytes());
            page.extend(2_u32.to_le_bytes());
            page.extend([0; 2]);
        }
        page.extend(0_u32.to_le_bytes());
        let mode_size_offset = page.len();
        page.extend(0_u32.to_le_bytes());
        page.extend([0; 16]);
        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert!(layout.tail_error.is_none());
        assert_eq!(layout.consumed_end_offset(), page.len());
        let settings = layout.print_settings.unwrap();
        assert_eq!(settings.paper.version, version);
        assert_eq!(settings.printer, ObjectEntry10_1::Null);
        assert!(settings.paper.dev_mode.is_empty());
        assert_eq!(
            settings.paper.system_language,
            (version >= 2).then_some(0x409)
        );

        page.truncate(mode_size_offset);
        page.extend(220_u32.to_le_bytes());
        let mut mode = [0; 223];
        mode[0x44..0x46].copy_from_slice(&188_u16.to_le_bytes());
        mode[0x46..0x48].copy_from_slice(&3_u16.to_le_bytes());
        mode[220..].copy_from_slice(&[1, 2, 3]);
        page.extend(mode);
        page.extend(EMF_PRINTER_CLASS_ID);
        page.extend(7_u32.to_le_bytes());
        page.extend(1_u16.to_le_bytes());

        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.consumed_end_offset(), page.len());
        assert!(layout.tail_error.is_none());
        let settings = layout.print_settings.unwrap();
        assert_eq!(settings.paper.version, version);
        assert_eq!(settings.paper.declared_dev_mode_size, 220);
        assert_eq!(settings.paper.dev_mode, mode);
        assert_eq!(
            settings.printer_body,
            Some(PrinterBody10_1::Emf { version: 1 })
        );

        let mode_start = mode_size_offset + 4;
        for end in [mode_start + 219, mode_start + 222] {
            let report = parse_mxd_streams_10_1(&[0; 8], Some(&page[..end])).unwrap();
            let layout = report.page_layout.unwrap();
            assert_eq!(layout.consumed_end_offset(), prefix.len());
            assert!(layout.tail_error.unwrap().contains("IPaper"));
            assert_eq!(report.map.unknown_fields[0].bytes, page[prefix.len()..end]);
        }

        page[mode_start + 0x44..mode_start + 0x46].copy_from_slice(&220_u16.to_le_bytes());
        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.consumed_end_offset(), prefix.len());
        assert!(layout.tail_error.unwrap().contains("DEVMODE"));
        assert_eq!(report.map.unknown_fields[0].bytes, page[prefix.len()..]);
    }
}

#[test]
fn legacy_split_print_layouts_follow_the_save_version() {
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1_with_version;
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;
    use mxd2qgz::maps::print_settings::{ARC_PRESS_PRINTER_CLASS_ID, PrinterBody10_1};
    use mxd2qgz::version::{
        mxd_version::identify_version, version_stream::write_arcmap_header_10_1,
    };
    use std::io::Write;

    for (index, text, layout_version) in [(0, "8.3", 7_u16), (1, "9.0", 9)] {
        let version_bytes =
            write_arcmap_header_10_1(index, text, (index != 0).then_some(12)).unwrap();
        let version = identify_version(&version_bytes).unwrap();

        let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
        page.extend(1_u32.to_le_bytes());
        page.extend(layout_version.to_le_bytes());
        page.extend([0; 8]);
        page.extend(layout_tail(layout_version, 0));
        page.extend(1_u16.to_le_bytes());
        let settings_start = page.len();

        let paper_version = if index == 0 { 2_u16 } else { 3_u16 };
        page.extend(paper_version.to_le_bytes());
        page.extend(0x409_u16.to_le_bytes());
        page.extend(2_u32.to_le_bytes());
        page.extend([0; 2]);
        page.extend(0_u32.to_le_bytes());
        let dev_mode = if index == 0 {
            // 原样写盘的分配块保留固定成员之后的额外字节。
            let mut mode = vec![0; 240];
            mode[0x44..0x46].copy_from_slice(&220_u16.to_le_bytes());
            mode[220..].fill(0xa5);
            mode
        } else {
            Vec::new()
        };
        page.extend((dev_mode.len() as u32).to_le_bytes());
        page.extend(&dev_mode);
        page.extend(ARC_PRESS_PRINTER_CLASS_ID);
        page.extend(7_u32.to_le_bytes());
        page.extend(2_u16.to_le_bytes());
        page.extend(4_u32.to_le_bytes());
        page.extend("甲\0".encode_utf16().flat_map(u16::to_le_bytes));
        page.extend(8_u16.to_le_bytes());
        page.push(1);
        page.extend(1_u32.to_le_bytes());
        page.extend("X\0".encode_utf16().flat_map(u16::to_le_bytes));
        for value in [100_i32, -20, 30, 40, 50] {
            page.extend(value.to_le_bytes());
        }
        page.push(0);

        let report =
            parse_mxd_streams_10_1_with_version(&[0; 8], Some(&page), Some(&version)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.consumed_end_offset(), page.len());
        assert!(layout.tail_error.is_none());
        let settings = layout.print_settings.unwrap();
        assert_eq!(settings.paper.version, paper_version);
        assert_eq!(settings.paper.dev_mode, dev_mode);
        let Some(PrinterBody10_1::ArcPress(printer)) = settings.printer_body else {
            panic!("未读出 ArcPress");
        };
        assert_eq!(printer.version, 2);
        assert_eq!(printer.driver_data, None);
        assert_eq!(printer.longs, [100, -20, 30, 40, 50]);

        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert!(layout.print_settings.is_none());
        assert_eq!(layout.consumed_end_offset(), settings_start);

        let truncated = &page[..page.len() - 1];
        let report =
            parse_mxd_streams_10_1_with_version(&[0; 8], Some(truncated), Some(&version)).unwrap();
        let layout = report.page_layout.unwrap();
        assert!(layout.print_settings.is_none());
        assert_eq!(layout.consumed_end_offset(), settings_start);
        assert_eq!(
            report.map.unknown_fields[0].bytes,
            truncated[settings_start..]
        );

        let wrong_version =
            identify_version(&write_arcmap_header_10_1(2, "9.2", Some(12)).unwrap()).unwrap();
        let report =
            parse_mxd_streams_10_1_with_version(&[0; 8], Some(&page), Some(&wrong_version))
                .unwrap();
        assert!(report.page_layout.unwrap().print_settings.is_none());

        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("旧版打印.mxd");
        let mut document = cfb::create(&path).unwrap();
        for (name, bytes) in [
            ("/Maps", &[0; 8][..]),
            ("/PageLayout", page.as_slice()),
            ("/Version", version_bytes.as_slice()),
        ] {
            document
                .create_stream(name)
                .unwrap()
                .write_all(bytes)
                .unwrap();
        }
        drop(document);
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_mxd2qgz"))
            .arg("parse")
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains(&format!(
                "已读 Paper v{paper_version} 及打印机槽至 0x{:x}",
                page.len()
            )),
            "{stdout}"
        );
    }
}

#[test]
fn ordinary_layout_elements_keep_cross_stream_references_and_body_boundaries() {
    use mxd2qgz::maps::map_object::MAP_CLASS_ID;
    use mxd2qgz::maps::page_layout_prefix::{MAP_FRAME_CLASS_ID, PAGE_LAYOUT_CLASS_ID};

    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();

    let v = 6_u16;
    let frame_key = 10_000_u32;
    let map_key = 29_u32;
    let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
    page.extend(9_999_u32.to_le_bytes());
    page.extend(v.to_le_bytes());
    page.extend(0_u32.to_le_bytes());
    page.extend(3_u32.to_le_bytes());

    let mut frame_header = MAP_FRAME_CLASS_ID.to_vec();
    frame_header.extend(frame_key.to_le_bytes());
    page.extend(&frame_header);
    page.extend(4_u16.to_le_bytes()); // MapFrame 无旧范围段及旧版框体。
    let map_offset = page.len();
    page.extend(MAP_CLASS_ID);
    page.extend(map_key.to_le_bytes());
    page.extend([0; 8]); // 两个空集合。
    page.extend([0; 2 + 2 + 16]); // BaseFrameElement、RectShapeElement、空几何。
    page.extend(0_u16.to_le_bytes()); // ElementProperties v0。
    for _ in 0..2 {
        page.extend(2_u32.to_le_bytes());
        page.extend(0_u16.to_le_bytes());
    }
    page.extend([0; 4 + 16 + 16 + 16 + 2]); // 旧属性及框体装饰。
    let backref_offset = page.len();
    page.extend(&frame_header);
    let null_offset = page.len();
    page.extend([0; 16]);
    let tail_offset = page.len();
    page.extend(layout_tail(v, 3));
    page.extend(0_u16.to_le_bytes());

    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    assert!(!layout.prefix.extra_safe);
    assert_eq!(layout.consumed_elements, 3, "{:?}", layout.stop);
    assert_eq!(layout.end_offset, tail_offset);
    assert_eq!(layout.consumed_end_offset(), page.len());
    let frame = &layout.map_frames[0].prefix;
    assert_eq!(frame.map_offset, map_offset);
    assert_eq!(
        frame.map,
        ObjectEntry10_1::BackReference {
            class_id: MAP_CLASS_ID,
            key: map_key
        }
    );
    assert_eq!(
        layout.indirect_elements,
        [
            (
                1,
                backref_offset,
                ObjectEntry10_1::BackReference {
                    class_id: MAP_FRAME_CLASS_ID,
                    key: frame_key
                }
            ),
            (2, null_offset, ObjectEntry10_1::Null)
        ]
    );

    // 回引头仍写 MapFrame 的 CLSID，但键实际指向 /Maps 的 Map。
    let mut wrong_target = page.clone();
    wrong_target[backref_offset + 16..backref_offset + 20].copy_from_slice(&map_key.to_le_bytes());
    let report = parse_mxd_streams_10_1(&maps, Some(&wrong_target)).unwrap();
    let layout = report.page_layout.unwrap();
    assert_eq!(layout.consumed_elements, 3);
    assert_eq!(layout.consumed_end_offset(), wrong_target.len());
    assert!(layout.stop.is_none());
    assert_eq!(
        layout.unresolved_element_references,
        [(
            1,
            backref_offset,
            ObjectEntry10_1::BackReference {
                class_id: MAP_FRAME_CLASS_ID,
                key: map_key,
            }
        )]
    );
    assert_eq!(
        layout.indirect_elements,
        [(2, null_offset, ObjectEntry10_1::Null)]
    );

    // 第一个已完成的体仍然可用，未知体及其后的字节一并保留。
    page[backref_offset..backref_offset + 16].fill(0xa5);
    page[backref_offset + 16..backref_offset + 20].copy_from_slice(&(frame_key + 1).to_le_bytes());
    for bytes in [&page[..backref_offset + 19], page.as_slice()] {
        let report = parse_mxd_streams_10_1(&maps, Some(bytes)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.consumed_elements, 1);
        assert_eq!(layout.consumed_end_offset(), backref_offset);
        assert!(layout.stop.is_some());
        let unknown = report
            .map
            .unknown_fields
            .iter()
            .find(|field| field.stream == "/PageLayout")
            .unwrap();
        assert_eq!(unknown.bytes, bytes[backref_offset..]);
    }
}

#[test]
fn reads_short_devmode_only_with_matching_version_and_complete_emf() {
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1_with_version;
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;
    use mxd2qgz::maps::print_settings::{EMF_PRINTER_CLASS_ID, PrinterBody10_1};
    use mxd2qgz::version::mxd_version::identify_version;

    let version = identify_version(&[8, 0, 0, 0, 57, 0, 46, 0, 48, 0, 0, 0, 10, 0]).unwrap();
    let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
    page.extend(1_u32.to_le_bytes());
    page.extend(7_u16.to_le_bytes());
    page.extend([0; 8]);
    page.extend(layout_tail(7, 0));
    page.extend(1_u16.to_le_bytes());
    let paper_start = page.len();
    page.extend(2_u16.to_le_bytes());
    page.extend(0x409_u16.to_le_bytes());
    page.extend(2_u32.to_le_bytes());
    page.extend([0; 2]);
    page.extend(0_u32.to_le_bytes());
    page.extend(240_u32.to_le_bytes());
    let mut mode = [0; 232];
    mode[0x44..0x46].copy_from_slice(&220_u16.to_le_bytes());
    mode[0x46..0x48].copy_from_slice(&20_u16.to_le_bytes());
    mode[212..].fill(0x5a);
    page.extend(mode);
    page.extend(EMF_PRINTER_CLASS_ID);
    page.extend(7_u32.to_le_bytes());
    page.extend(1_u16.to_le_bytes());

    let parse = |page: &[u8], version| {
        parse_mxd_streams_10_1_with_version(&[0; 8], Some(page), version)
            .unwrap()
            .page_layout
            .unwrap()
    };
    let layout = parse(&page, Some(&version));
    assert_eq!(layout.consumed_end_offset(), page.len());
    let settings = layout.print_settings.unwrap();
    assert_eq!(settings.paper.dev_mode, mode);
    assert_eq!(
        settings.printer_body,
        Some(PrinterBody10_1::Emf { version: 1 })
    );

    assert!(parse(&page, None).print_settings.is_none());
    for end in paper_start..page.len() {
        let layout = parse(&page[..end], Some(&version));
        assert!(layout.print_settings.is_none());
        assert_eq!(layout.consumed_end_offset(), paper_start);
    }
    page.push(0);
    assert!(parse(&page, Some(&version)).print_settings.is_none());
}

#[test]
#[ignore = "需要本地提取的 ArcGIS 10.1 Java SDK 工程"]
fn reads_yellowstone_print_settings_from_sdk_documents() {
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1_with_version;
    use mxd2qgz::maps::print_settings::{EMF_PRINTER_CLASS_ID, PrinterBody10_1};
    use mxd2qgz::version::mxd_version::identify_version;

    for (file_name, paper_version, layout_end) in [
        ("Yellowstone.mxd", 2, 0x960),
        ("YellowstoneGDB.mxd", 3, 0x1296),
    ] {
        let path = format!(
            "temp/terra-round8-old/sdk-java/{file_name}.4896861F_D89C_46BA_ABEC_17591162B2B1"
        );
        let mut document = cfb::open(path).unwrap();
        let mut read = |name: &str| {
            let mut bytes = Vec::new();
            document
                .open_stream(name)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            bytes
        };
        let version = identify_version(&read("/Version")).unwrap();
        let maps = read("/Maps");
        let page = read("/PageLayout");
        let parsed =
            parse_mxd_streams_10_1_with_version(&maps, Some(&page), Some(&version)).unwrap();
        assert!(
            parsed
                .map
                .unknown_fields
                .iter()
                .all(|field| field.stream != "/PageLayout")
        );
        let layout = parsed.page_layout.unwrap();
        assert_eq!(page.len(), layout_end);
        assert_eq!(layout.consumed_end_offset(), layout_end);
        let settings = layout.print_settings.unwrap();
        assert_eq!(settings.paper.version, paper_version);
        assert_eq!(settings.end_offset, layout_end);
        if paper_version == 2 {
            assert_eq!(settings.paper.declared_dev_mode_size, 1172);
            assert_eq!(settings.paper.dev_mode, page[0x4be..0x94a]);
            assert_eq!(
                settings.printer,
                ObjectEntry10_1::NewObject {
                    class_id: EMF_PRINTER_CLASS_ID,
                    key: 164,
                }
            );
            assert_eq!(
                settings.printer_body,
                Some(PrinterBody10_1::Emf { version: 1 })
            );
        }
    }
}

#[test]
#[ignore = "需要本地提取的 ArcGIS 10.1 SDK 工程"]
fn reads_cross_stream_map_references_with_unmapped_version_headers() {
    use mxd2qgz::maps::map_object::MAP_CLASS_ID;
    use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1_with_version;
    use mxd2qgz::version::mxd_version::identify_version;

    for (path, text, short, layout_version, frame_version, frame_key, map_key) in [
        (
            "sdk-dotnet/GISForXMLSample.mxd.89833ED7_0681_4032_BE76_9AB6C721CEB2",
            "9.4",
            18,
            14,
            4,
            236,
            29,
        ),
        (
            "sdk-java/World.mxd.4896861F_D89C_46BA_ABEC_17591162B2B1",
            "9.3",
            15,
            9,
            2,
            332,
            31,
        ),
    ] {
        let mut document = cfb::open(format!("temp/terra-round8-old/{path}")).unwrap();
        let mut read = |name: &str| {
            let mut bytes = Vec::new();
            document
                .open_stream(name)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            bytes
        };
        let version = identify_version(&read("/Version")).unwrap();
        assert_eq!(
            (version.text.as_str(), version.compatibility_short),
            (text, short)
        );
        assert_eq!(version.candidate_save_index, None);
        let maps = read("/Maps");
        let page = read("/PageLayout");
        let parsed =
            parse_mxd_streams_10_1_with_version(&maps, Some(&page), Some(&version)).unwrap();
        assert_eq!(parsed.confirmed_tail.unwrap().0 + 4, maps.len());
        let mut root = Reader::new(&maps);
        let mut refs = RefTable10_1::new();
        assert_eq!(root.read_u32_le().unwrap(), 1);
        assert_eq!(
            mxd2qgz::storage::object_refs::read_object_entry_10_1(&mut root, &mut refs).unwrap(),
            ObjectEntry10_1::NewObject {
                class_id: MAP_CLASS_ID,
                key: map_key
            }
        );
        let layout = parsed.page_layout.unwrap();
        assert_eq!(layout.prefix.version, layout_version);
        assert_eq!(layout.consumed_end_offset(), page.len());
        let [frame] = layout.map_frames.as_slice() else {
            panic!("预期一个 MapFrame");
        };
        assert_eq!(frame.prefix.version, frame_version);
        assert_eq!(frame.prefix.frame_key, frame_key);
        assert_eq!(frame.prefix.map_offset, 0x3c);
        assert_eq!(
            frame.prefix.map,
            ObjectEntry10_1::BackReference {
                class_id: MAP_CLASS_ID,
                key: map_key,
            }
        );
    }
}

#[test]
fn old_layout_nonpositive_element_count_reaches_the_tail() {
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;

    for count in [i32::MIN, -1, 0] {
        let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
        page.extend(1_u32.to_le_bytes());
        page.extend(6_u16.to_le_bytes());
        page.extend(0_u32.to_le_bytes());
        page.extend(count.to_le_bytes());
        page.extend(layout_tail(6, 0));
        page.extend(0_u16.to_le_bytes());
        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(layout.prefix.element_count_raw, count);
        assert_eq!(layout.prefix.element_count, 0);
        assert_eq!(layout.consumed_elements, 0);
        assert_eq!(layout.print_settings_flag, Some(0));
        assert_eq!(layout.consumed_end_offset(), page.len());
    }
}

#[test]
fn extra_safe_layout_preserves_wrong_map_reference_and_reads_the_next_element() {
    use mxd2qgz::maps::page_layout_prefix::{MAP_FRAME_CLASS_ID, PAGE_LAYOUT_CLASS_ID};

    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    let version = 9_u16;
    let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
    page.extend(9_999_u32.to_le_bytes());
    page.extend(version.to_le_bytes());
    page.extend(0_u32.to_le_bytes());
    page.extend(2_u32.to_le_bytes());
    let wrong_offset = page.len();
    page.extend(20_u64.to_le_bytes());
    page.extend(MAP_FRAME_CLASS_ID);
    page.extend(29_u32.to_le_bytes());
    let next_offset = page.len();
    page.extend(16_u64.to_le_bytes());
    page.extend([0; 16]);
    page.extend(layout_tail(version, 2));
    page.extend(0_u16.to_le_bytes());

    let report = parse_mxd_streams_10_1(&maps, Some(&page)).unwrap();
    let layout = report.page_layout.unwrap();
    let raw = &page[wrong_offset..next_offset];
    assert_eq!(
        layout.placeholder_elements,
        [
            (0, wrong_offset, raw.to_vec()),
            (1, next_offset, page[next_offset..next_offset + 24].to_vec())
        ]
    );
    assert!(layout.indirect_elements.is_empty());
    assert_eq!(layout.consumed_end_offset(), page.len());
    assert!(
        report
            .map
            .unknown_fields
            .iter()
            .any(|field| field.stream == "/PageLayout"
                && field.offset == wrong_offset as u64
                && field.bytes == raw)
    );

    for end in wrong_offset..next_offset {
        let report = parse_mxd_streams_10_1(&maps, Some(&page[..end])).unwrap();
        let layout = report.page_layout.unwrap();
        assert!(layout.placeholder_elements.is_empty());
        assert_eq!(layout.consumed_end_offset(), wrong_offset);
    }
}

#[test]
fn extra_safe_empty_wrapper_and_null_header_are_placeholders() {
    use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;

    for version in 7_u16..=15 {
        let mut page = PAGE_LAYOUT_CLASS_ID.to_vec();
        page.extend(1_u32.to_le_bytes());
        page.extend(version.to_le_bytes());
        page.extend(0_u32.to_le_bytes());
        page.extend(2_u32.to_le_bytes());
        let empty_offset = page.len();
        page.extend(0_u64.to_le_bytes());
        let next_offset = page.len();
        // 原生占位器复制完整声明范围，空对象头后也可带未解释字节。
        let suffix = if version % 2 == 0 {
            &[0xca, 0xfe][..]
        } else {
            &[]
        };
        page.extend((16 + suffix.len() as u64).to_le_bytes());
        page.extend([0; 16]);
        page.extend(suffix);
        let tail_offset = page.len();
        page.extend(layout_tail(version, 2));
        page.extend(0_u16.to_le_bytes());

        let report = parse_mxd_streams_10_1(&[0; 8], Some(&page)).unwrap();
        let layout = report.page_layout.unwrap();
        assert_eq!(
            layout.placeholder_elements,
            [
                (0, empty_offset, vec![0; 8]),
                (1, next_offset, page[next_offset..tail_offset].to_vec())
            ]
        );
        assert!(layout.indirect_elements.is_empty());
        assert_eq!(layout.consumed_end_offset(), page.len());

        for end in next_offset..tail_offset {
            let report = parse_mxd_streams_10_1(&[0; 8], Some(&page[..end])).unwrap();
            let layout = report.page_layout.unwrap();
            assert_eq!(layout.placeholder_elements.len(), 1);
            assert_eq!(layout.consumed_end_offset(), next_offset);
        }
        for end in empty_offset..next_offset {
            let report = parse_mxd_streams_10_1(&[0; 8], Some(&page[..end])).unwrap();
            let layout = report.page_layout.unwrap();
            assert!(layout.placeholder_elements.is_empty());
            assert_eq!(layout.consumed_end_offset(), empty_offset);
        }
    }
}

#[test]
fn bounded_layout_extensions_preserve_unknown_and_leave_following_slot_aligned() {
    let optimization_class = [
        0x9f, 0xd8, 0x29, 0x6c, 0xba, 0xb7, 0xb0, 0x4f, 0xa8, 0x8d, 0xfe, 0xa7, 0x7a, 0x03, 0xba,
        0x0e,
    ];
    let unknown_class = [0x93; 16];
    let baseline = layout_tail(11, 0);
    let extensions_offset = baseline.len() - 12;
    let mut bytes = baseline[..extensions_offset].to_vec();
    bytes.extend(3_u32.to_le_bytes());
    for (class_id, body) in [
        (unknown_class, &[0xaa, 0xbb, 0xcc][..]),
        (optimization_class, &[][..]),
        (optimization_class, &[1, 0, 0, 0, 0, 0][..]),
    ] {
        bytes.extend((body.len() as u64).to_le_bytes());
        bytes.extend(class_id);
        bytes.extend(body);
    }
    bytes.extend(&baseline[extensions_offset + 4..]);
    let end = bytes.len();
    bytes.extend(0x1234_u16.to_le_bytes());

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let tail = read_page_layout_tail_10_1(&mut reader, &mut refs, 11, 0, 8192, |_| false).unwrap();
    assert_eq!(tail.opaque_extensions.len(), 2);
    assert_eq!(tail.opaque_extensions[0].index, 0);
    assert_eq!(tail.opaque_extensions[0].class_id, unknown_class);
    assert_eq!(tail.opaque_extensions[0].body, [0xaa, 0xbb, 0xcc]);
    assert_eq!(tail.opaque_extensions[1].index, 1);
    assert!(tail.opaque_extensions[1].body.is_empty());
    assert_eq!(tail.optimization_extensions.len(), 1);
    assert_eq!(tail.optimization_extensions[0].version, 1);
    assert!(tail.version_11_slot.is_some());
    assert_eq!(reader.position(), end);

    bytes[extensions_offset + 4..extensions_offset + 12].copy_from_slice(&8192_u64.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert!(read_page_layout_tail_10_1(&mut reader, &mut refs, 11, 0, 8192, |_| false).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs.len(), 0);
}

#[test]
fn layout_optimization_extension_reads_connection_and_shared_reference() {
    use mxd2qgz::layers::cache_tile_reader::AGS_SERVER_CONNECTION_NAME;

    let optimization_class = [
        0x9f, 0xd8, 0x29, 0x6c, 0xba, 0xb7, 0xb0, 0x4f, 0xa8, 0x8d, 0xfe, 0xa7, 0x7a, 0x03, 0xba,
        0x0e,
    ];
    let baseline = layout_tail(11, 0);
    let extensions_offset = baseline.len() - 12;
    let mut bytes = baseline[..extensions_offset].to_vec();
    bytes.extend(2_u32.to_le_bytes());
    for is_new in [true, false] {
        let mut body = 3_u16.to_le_bytes().to_vec();
        body.extend([0; 4]); // 空准备结果。
        body.extend([2, 0, 0, 0, 0, 0]); // 空字符串。
        body.extend([0; 8]);
        body.extend(AGS_SERVER_CONNECTION_NAME);
        body.extend(73_u32.to_le_bytes());
        if is_new {
            body.extend(1_u16.to_le_bytes());
            body.extend([0; 16]); // 空 PropertySet。
        }
        bytes.extend((body.len() as u64).to_le_bytes());
        bytes.extend(optimization_class);
        bytes.extend(body);
    }
    bytes.extend(&baseline[extensions_offset + 4..]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let tail = read_page_layout_tail_10_1(&mut reader, &mut refs, 11, 0, 8192, |_| false).unwrap();
    let connection = tail.optimization_extensions[0]
        .server_connection
        .as_ref()
        .unwrap();
    assert_eq!(connection.body.as_ref().unwrap().version, 1);
    let reference = tail.optimization_extensions[1]
        .server_connection
        .as_ref()
        .unwrap();
    assert!(matches!(
        reference.entry,
        ObjectEntry10_1::BackReference { key: 73, .. }
    ));
    assert!(refs.contains(73));
    assert_eq!(reader.position(), bytes.len());

    let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
    let mut refs = RefTable10_1::new();
    assert!(read_page_layout_tail_10_1(&mut reader, &mut refs, 11, 0, 8192, |_| false).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs.len(), 0);
}

#[test]
fn layout_optimization_extension_reads_prepare_result_versions_and_options() {
    let optimization_class = [
        0x9f, 0xd8, 0x29, 0x6c, 0xba, 0xb7, 0xb0, 0x4f, 0xa8, 0x8d, 0xfe, 0xa7, 0x7a, 0x03, 0xba,
        0x0e,
    ];
    let converter_class = [
        0x24, 0x42, 0x31, 0x97, 0xa2, 0x2d, 0x39, 0x49, 0xac, 0xc5, 0xdd, 0x7c, 0x0a, 0x7b, 0xfc,
        0x8b,
    ];
    let mut body = 1_u16.to_le_bytes().to_vec();
    body.extend(2_i32.to_le_bytes());
    for version in [1_i16, 2] {
        body.extend(version.to_le_bytes());
        body.extend(17_u32.to_le_bytes());
        body.extend(19_u32.to_le_bytes());
        body.extend(converter_class);
        body.extend(73_u32.to_le_bytes());
        if version == 1 {
            body.extend(1_i16.to_le_bytes());
            body.extend(23_u32.to_le_bytes());
            body.extend([0; 16]); // 空属性集。
            body.extend([0; 32]); // 空 IUnknown 和 IMap。
        } else {
            body.extend(1_i32.to_le_bytes());
            body.extend([0; 16]);
        }
    }
    let baseline = layout_tail(11, 0);
    let extensions_offset = baseline.len() - 12;
    let mut bytes = baseline[..extensions_offset].to_vec();
    bytes.extend(1_u32.to_le_bytes());
    bytes.extend((body.len() as u64).to_le_bytes());
    bytes.extend(optimization_class);
    bytes.extend(body);
    bytes.extend(&baseline[extensions_offset + 4..]);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let tail = read_page_layout_tail_10_1(&mut reader, &mut refs, 11, 0, 8192, |_| false).unwrap();
    let results = &tail.optimization_extensions[0].prepare_results;
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].raw_longs, [17, 19]);
    assert_eq!(results[0].converter.body.as_ref().unwrap().raw_long, 23);
    assert_eq!(results[0].legacy_map, Some(ObjectEntry10_1::Null));
    assert_eq!(results[1].objects, [ObjectEntry10_1::Null]);
    assert!(matches!(
        results[1].converter.entry,
        ObjectEntry10_1::BackReference { key: 73, .. }
    ));
    assert!(refs.contains(73));
    assert_eq!(reader.position(), bytes.len());

    let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
    let mut refs = RefTable10_1::new();
    assert!(read_page_layout_tail_10_1(&mut reader, &mut refs, 11, 0, 8192, |_| false).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(refs.len(), 0);
}
