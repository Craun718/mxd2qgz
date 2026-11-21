use mxd2qgz::maps::page_layout_prefix::{
    PAGE_LAYOUT_CLASS_ID, read_page_layout_element_10_1, read_page_layout_prefix_10_1,
};
use mxd2qgz::{objects, storage, stream};
use objects::element_body::{
    ElementBody10_1, GeometrySlot10_1, LINE_ELEMENT_CLASS_ID, TEXT_ELEMENT_CLASS_ID,
};
use objects::envelope::SpatialReferenceEntry10_1;
use std::io::Read;
use storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use stream::reader::Reader;

#[test]
fn page_layout_prefix_checks_versions_counts_and_truncation() {
    for version in [0_u16, 6, 7, 15] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(0x12345678_u32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_page_layout_prefix_10_1(&mut reader, 2).is_err());
            assert_eq!(reader.position(), 0);
        }
        let mut reader = Reader::new(&bytes);
        let prefix = read_page_layout_prefix_10_1(&mut reader, 2).unwrap();
        assert_eq!(prefix.raw_member, 0x12345678);
        assert_eq!(prefix.element_count, 2);
        assert_eq!(prefix.extra_safe, version >= 7);
        assert_eq!(reader.position(), 10);
        let mut reader = Reader::new(&bytes);
        assert!(read_page_layout_prefix_10_1(&mut reader, 1).is_err());
        assert_eq!(reader.position(), 0);
    }
    for (version, count) in [(16_u16, 0_i32), (1, -1)] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(count.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert!(read_page_layout_prefix_10_1(&mut reader, 100).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
#[ignore = "需要本地提取的 ArcMap 10.1 介质流"]
fn media_page_layout_prefix_matches_element_loop() {
    let page_layout = [
        0x6e, 0xd7, 0x94, 0xdd, 0x6d, 0x83, 0xd0, 0x11, 0x87, 0xec, 0x08, 0x00, 0x09, 0xec, 0x73,
        0x2a,
    ];
    for (name, key, version, count, target_index, target_offset, target_key) in [
        ("Utilities", 163, 14, 27, 11, 0x20cf, 254),
        ("CentralUSA", 736, 15, 9, 1, 0xa0c, 744),
    ] {
        let path = format!("temp/arcmap101-media-streams/{name}.mxd.PageLayout");
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(&bytes[..16], &page_layout);
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), key);
        let mut reader = Reader::new(&bytes[20..]);
        let prefix = read_page_layout_prefix_10_1(&mut reader, 100).unwrap();
        assert_eq!(prefix.version, version);
        assert_eq!(prefix.raw_member, 1);
        assert_eq!(prefix.element_count, count);
        assert!(prefix.extra_safe);
        assert_eq!(reader.position(), 10);
        // 候选范围不登记跳过对象，不能证明其内部引用有效。
        let mut offset = 30_usize;
        for index in 0..count {
            let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let seek = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            assert_eq!(seek, 0);
            assert!(length >= 20);
            let start = offset + 8;
            let end = start.checked_add(length).unwrap();
            assert!(end <= bytes.len());
            if index == target_index {
                assert_eq!(start, target_offset);
                assert_eq!(
                    &bytes[start..start + 16],
                    &mxd2qgz::objects::element_body::LINE_ELEMENT_CLASS_ID
                );
                assert_eq!(
                    u32::from_le_bytes(bytes[start + 16..start + 20].try_into().unwrap()),
                    target_key
                );
                assert_eq!(end, target_offset + 0x11d);
            }
            offset = end;
        }
    }
}

#[test]
#[ignore = "需要本地 CentralUSA 介质文件"]
fn central_usa_first_two_elements_use_real_reference_definition() {
    let mut document = cfb::open("temp/arcmap101-media-mxd/CentralUSA.mxd").unwrap();
    let mut data = Vec::new();
    document
        .open_stream("/PageLayout")
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&data);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut table).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PAGE_LAYOUT_CLASS_ID,
            key: 736
        }
    ));
    let prefix = read_page_layout_prefix_10_1(&mut reader, 100).unwrap();
    assert_eq!(prefix.element_count, 9);
    assert_eq!(reader.position(), 0x1e);
    let (entry, body) = read_page_layout_element_10_1(&mut reader, 1_000_000, &mut table).unwrap();
    assert!(matches!(
        entry,
        ObjectEntry10_1::NewObject {
            class_id: TEXT_ELEMENT_CLASS_ID,
            key: 737
        }
    ));
    let ElementBody10_1::TextElement(text) = body else {
        panic!("应为文本元素")
    };
    let GeometrySlot10_1::Point(point) = text.geometry else {
        panic!("应为点几何")
    };
    let Some(SpatialReferenceEntry10_1::UnknownCoordinateSystem { key: 743, body }) =
        point.spatial_reference
    else {
        panic!("应为新建的 UnknownCoordinateSystem 对象");
    };
    assert_eq!(body.save_version, 3);
    assert_eq!(body.strings, ["Unknown", "", "", ""]);
    assert_eq!(reader.position(), 0xa04);
    assert_eq!(table.len(), 8);
    for key in 736..=743 {
        assert!(table.contains(key));
    }
    for end in 0xa04..0xb29 {
        let mut partial = Reader::new(&data[0xa04..end]);
        let mut prior = table.clone();
        assert!(read_page_layout_element_10_1(&mut partial, 1_000_000, &mut prior).is_err());
        assert_eq!(partial.position(), 0);
        assert_eq!(prior.len(), 8);
        assert!(!prior.contains(744));
    }
    let (entry, body) = read_page_layout_element_10_1(&mut reader, 1_000_000, &mut table).unwrap();
    assert!(matches!(
        entry,
        ObjectEntry10_1::NewObject {
            class_id: LINE_ELEMENT_CLASS_ID,
            key: 744
        }
    ));
    let ElementBody10_1::LineElement(line) = body else {
        panic!("应为线元素")
    };
    let GeometrySlot10_1::Polyline(polyline) = line.geometry else {
        panic!("应为折线几何")
    };
    assert!(matches!(
        polyline.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference { key: 743, .. })
    ));
    assert!(line.stop.is_none());
    assert_eq!(reader.position(), 0xb29);
    assert_eq!(table.len(), 12);
    for key in 736..=747 {
        assert!(table.contains(key));
    }
    // 独立读取第二项时，未见键 743 被当作新对象而失败，且游标和表回滚。
    let mut reader = Reader::new(&data[0xa04..0xb29]);
    let mut table = RefTable10_1::new();
    assert!(read_page_layout_element_10_1(&mut reader, 1_000_000, &mut table).is_err());
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
    assert_eq!(&data[0x96e..0x982], &data[0xb0f..0xb23]);
}

#[test]
fn page_element_rejects_bad_wrapper_without_mutation() {
    for bytes in [
        vec![],
        vec![0; 8],
        [20_u32.to_le_bytes(), 1_u32.to_le_bytes()].concat(),
        [20_u32.to_le_bytes(), 0_u32.to_le_bytes()].concat(),
    ] {
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_page_layout_element_10_1(&mut reader, 100, &mut table).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
}
