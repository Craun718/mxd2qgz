use mxd2qgz::stream;
use stream::reader::Reader;

#[path = "../src/maps/page_layout_prefix.rs"]
mod page_layout_prefix;
use page_layout_prefix::read_page_layout_prefix_10_1;

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
