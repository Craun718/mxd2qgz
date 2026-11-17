use mxd2qgz::maps::map_object::{
    MAP_CLASS_ID, MapSuffixStop, read_map_suffix_10_1, read_post_illumination_fields_10_1,
};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

fn maps(path: &std::path::Path) -> Vec<u8> {
    let mut doc = cfb::open(path).unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

#[test]
fn reference_map_reaches_post_illumination_fields_from_root() {
    let bytes = maps(std::path::Path::new("samples/测试底图.mxd"));
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    assert_eq!(reader.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(&mut reader, &mut table).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut table).unwrap();
    assert_eq!(map.stop, MapSuffixStop::AfterBorderTail { stage: 35 });
    let fields = map.post_illumination.unwrap();
    assert_eq!(fields.offset, 0x24fe8);
    assert_eq!(fields.short_1a0.to_le_bytes(), bytes[0x24fe8..0x24fea]);
    assert_eq!(
        fields.discarded_short.to_le_bytes(),
        bytes[0x24fea..0x24fec]
    );
    assert_eq!(fields.int_1b0.to_le_bytes(), bytes[0x24fec..0x24ff0]);
    assert_eq!(fields.short_1ac.to_le_bytes(), bytes[0x24ff0..0x24ff2]);
    assert_eq!(fields.short_268.to_le_bytes(), bytes[0x24ff2..0x24ff4]);
    assert_eq!(reader.position(), 0x253b6);
}

#[test]
fn post_illumination_reader_preserves_bits_and_rejects_truncation() {
    let mut bytes = vec![0x5a];
    bytes.extend(0x8123_u16.to_le_bytes());
    bytes.extend(0x4567_u16.to_le_bytes());
    bytes.extend(0x89abcdef_u32.to_le_bytes());
    bytes.extend(0xfedc_u16.to_le_bytes());
    bytes.extend(0xba98_u16.to_le_bytes());
    let end = bytes.len();
    bytes.push(0x5a);
    for version in [23, 34, 35, 51] {
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(1).unwrap();
        let fields = read_post_illumination_fields_10_1(&mut reader, version).unwrap();
        assert_eq!(
            (fields.offset, fields.short_1a0, fields.discarded_short),
            (1, 0x8123, 0x4567)
        );
        assert_eq!(
            (fields.int_1b0, fields.short_1ac, fields.short_268),
            (0x89abcdef, 0xfedc, 0xba98)
        );
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), &[0x5a]);
    }
    for length in 1..end {
        let mut reader = Reader::new(&bytes[..length]);
        reader.read_bytes(1).unwrap();
        assert!(read_post_illumination_fields_10_1(&mut reader, 51).is_err());
        assert_eq!(reader.position(), 1);
    }
    for version in [0, 22, 52, u16::MAX] {
        let mut reader = Reader::new(&bytes);
        assert!(read_post_illumination_fields_10_1(&mut reader, version).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
#[ignore = "需要本地 10.1 与 10.8 安装介质模板"]
fn inventory_media_by_root_parser() {
    for directory in ["temp/arcmap101-media-mxd", "temp/arcmap108-media-mxd"] {
        let mut total = 0;
        let mut roots = 0;
        let mut prefix = 0;
        let mut nonzero = 0;
        let mut after_illumination = 0;
        for item in std::fs::read_dir(directory).unwrap() {
            let path = item.unwrap().path();
            if !path.is_file() {
                continue;
            }
            total += 1;
            let bytes = maps(&path);
            let mut r = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            if r.read_u32_le().ok() != Some(1) {
                continue;
            }
            if !matches!(
                read_object_entry_10_1(&mut r, &mut table),
                Ok(ObjectEntry10_1::NewObject {
                    class_id: MAP_CLASS_ID,
                    ..
                })
            ) {
                continue;
            }
            roots += 1;
            if let Ok(result) = read_map_suffix_10_1(&mut r, 16_000_000, &mut table) {
                if let Some(value) = result.post_bookmark {
                    prefix += 1;
                    nonzero += usize::from(value.standalone_table_count > 0);
                }
                after_illumination += usize::from(result.post_illumination.is_some());
            }
        }
        println!(
            "{directory}: files={total} map_roots={roots} post_bookmark={prefix} nonzero_count={nonzero} post_illumination={after_illumination}"
        );
        assert!(roots > 0);
    }
}
