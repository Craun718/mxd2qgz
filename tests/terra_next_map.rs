//! 构造输入验证 Map 后缀与非空书签循环的接线，不代表真实工程取证。
use mxd2qgz::maps::map_object::{BookmarkElement10_1, MapSuffixStop, read_map_suffix_10_1};
use mxd2qgz::maps::map_surround::{AOI_BOOKMARK_CLASS_ID, FEATURE_BOOKMARK_CLASS_ID};
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;

fn string(value: &str) -> Vec<u8> {
    let payload: Vec<u8> = value
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut result = (payload.len() as u32).to_le_bytes().to_vec();
    result.extend(payload);
    result
}

fn header(bytes: &mut Vec<u8>, class: &[u8; 16], key: u32) {
    bytes.extend(class);
    bytes.extend(key.to_le_bytes());
}

#[test]
fn map_reads_two_bookmark_bodies_and_a_back_reference() {
    let mut bytes = 51_u16.to_le_bytes().to_vec();
    bytes.extend(string("地图"));
    bytes.extend(string(""));
    bytes.extend([0; 16]); // 空空间参考
    bytes.extend(0_i32.to_le_bytes()); // 图层数
    bytes.extend((-1_i32).to_le_bytes()); // 活动图层
    bytes.extend([0; 16]); // 基本图形层
    bytes.extend(0_i32.to_le_bytes()); // 地图装饰物数
    for x in [1.0_f64, 2.0, 3.0, 4.0] {
        bytes.extend(x.to_le_bytes());
    }
    bytes.extend([0; 8]);
    bytes.extend([0; 16]); // IEnvelope
    bytes.extend(3_i32.to_le_bytes());
    header(&mut bytes, &AOI_BOOKMARK_CLASS_ID, 10);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend([0; 16]); // 书签 IEnvelope
    bytes.extend(string("范围"));
    header(&mut bytes, &FEATURE_BOOKMARK_CLASS_ID, 11);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend((-7_i32).to_le_bytes());
    bytes.extend([0; 16]); // IName
    bytes.extend(string("要素"));
    header(&mut bytes, &AOI_BOOKMARK_CLASS_ID, 10);
    let end = bytes.len();
    bytes.push(0xaa);

    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let result = read_map_suffix_10_1(&mut reader, 4096, &mut table).unwrap();
    assert_eq!(result.stop, MapSuffixStop::AfterBookmarks);
    assert_eq!(result.bookmark_count, Some(3));
    assert!(
        matches!(&result.bookmark_parsed[0], Some(BookmarkElement10_1::AoiBookmark(b)) if b.name == "范围")
    );
    assert!(
        matches!(&result.bookmark_parsed[1], Some(BookmarkElement10_1::FeatureBookmark(b)) if b.name == "要素" && b.raw_int32 == -7)
    );
    assert!(matches!(
        result.bookmarks[2].entry,
        ObjectEntry10_1::BackReference { key: 10, .. }
    ));
    assert!(table.contains(10) && table.contains(11));
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
}
