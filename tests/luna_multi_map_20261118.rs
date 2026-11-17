use mxd2qgz::maps::map_object::{MAP_CLASS_ID, read_map_prefix_10_1};
use mxd2qgz::maps::mxd_maps::parse_maps_10_1;
use mxd2qgz::storage::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::stream::reader::Reader;
use std::{collections::HashSet, io::Read, path::Path};

#[test]
#[ignore = "需要 10.1 安装介质提取文件"]
fn media_maps_consume_distinct_frames_before_the_root_tail() {
    let media = Path::new(env!("CARGO_MANIFEST_DIR")).join("temp/arcmap101-media-mxd");
    for (file, starts, root_tail) in [
        ("MilitaryPortrait.mxt", &[4, 0x49e][..], 0x18dc),
        ("Utilities.mxt", &[4, 0x11d7, 0x1dcb][..], 0x29c1),
    ] {
        let mut doc = cfb::open(media.join(file)).unwrap();
        let mut maps = Vec::new();
        doc.open_stream("/Maps")
            .unwrap()
            .read_to_end(&mut maps)
            .unwrap();
        let report = parse_maps_10_1(&maps).unwrap();
        assert_eq!(report.count as usize, starts.len(), "{file}");
        assert_eq!(report.map.frames.len(), starts.len(), "{file}");
        assert_eq!(report.confirmed_tail, Some((root_tail, 1)), "{file}");
        assert_eq!(maps.len(), root_tail + 4, "{file}");
        assert!(report.map.unknown_fields.is_empty(), "{file}");

        // 独立校验已由根循环定位的每个头都是不同 Map 对象键。
        let mut keys = HashSet::new();
        for (&offset, frame) in starts.iter().zip(&report.map.frames) {
            let mut reader = Reader::new(&maps);
            reader.read_bytes(offset).unwrap();
            let ObjectHeader::Object { class_id, key } = read_header_10_1(&mut reader).unwrap()
            else {
                panic!("{}:{offset:#x} 应为 Map 对象头", file);
            };
            assert_eq!(class_id, MAP_CLASS_ID);
            assert!(keys.insert(key));
            let prefix = read_map_prefix_10_1(&mut reader, 1 << 20).unwrap();
            assert_eq!(prefix.version, 30);
            assert!(frame.name == prefix.name);
        }
    }
}
