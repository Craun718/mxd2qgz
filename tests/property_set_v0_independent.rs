//! PropertySet 版本 0 的合成边界测试，不替代 ArcMap 运行验证。

use mxd2qgz::objects::property_set::{PropertyValue10_1, read_property_set_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn nonempty_v0_body_and_truncated_prefixes() {
    // 两个条目分别覆盖定宽值和非 ASCII 字符串，末尾保留宿主字段。
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    for unit in [4_u32.to_le_bytes().to_vec(), vec![b'A', 0, 0, 0]] {
        bytes.extend(unit);
    }
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend(42_i32.to_le_bytes());
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend([0x2d, 0x4e, 0, 0]);
    bytes.extend(8_u16.to_le_bytes());
    bytes.extend(4_u32.to_le_bytes());
    bytes.extend([0x87, 0x65, 0, 0]);
    let body_end = bytes.len();
    bytes.push(0xa5);

    let mut reader = Reader::new(&bytes);
    let body = read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(body.version, 0);
    assert_eq!(body.entries.len(), 2);
    assert_eq!(body.entries[0].key, "A");
    assert_eq!(
        body.entries[0].value,
        PropertyValue10_1::Raw4(42_i32.to_le_bytes())
    );
    assert_eq!(body.entries[1].key, "中");
    assert_eq!(
        body.entries[1].value,
        PropertyValue10_1::String("文".into())
    );
    assert_eq!(reader.position(), body_end);

    for end in 0..body_end {
        let mut reader = Reader::new(&bytes[..end]);
        assert!(
            read_property_set_10_1(&mut reader, 1024, &mut RefTable10_1::new()).is_err(),
            "截断位置 {end}"
        );
        assert_eq!(reader.position(), 0, "截断位置 {end}");
    }
}
