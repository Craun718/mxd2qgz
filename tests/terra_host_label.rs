//! 静态接口对位后的构造输入回归，不替代 ArcMap 运行验证。

use mxd2qgz::layers::annotate_props::read_edit_template_manager_body_10_1;
use mxd2qgz::layers::edit_template::read_edit_template_body_10_1;
use mxd2qgz::layers::feature_id_set::read_feature_id_set_body_10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

fn native_string(value: &str) -> Vec<u8> {
    let mut utf16: Vec<u16> = value.encode_utf16().collect();
    utf16.push(0);
    let mut bytes = ((utf16.len() * 2) as u32).to_le_bytes().to_vec();
    for word in utf16 {
        bytes.extend(word.to_le_bytes());
    }
    bytes
}

#[test]
fn template_accessors_follow_stream_and_version_gate() {
    for version in [1_u16, 2] {
        let mut data = version.to_le_bytes().to_vec();
        data.extend(native_string("name"));
        data.extend(native_string("description"));
        data.extend([0x42; 16]);
        data.extend(0_i32.to_le_bytes());
        if version == 2 {
            data.extend(native_string("tags"));
        }
        let mut reader = Reader::new(&data);
        let result =
            read_edit_template_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1).unwrap();
        assert_eq!(result.name, "name");
        assert_eq!(result.description(), "description");
        assert_eq!(result.tool_guid_bytes(), &[0x42; 16]);
        assert_eq!(result.tags(), (version == 2).then_some("tags"));
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn manager_preserves_flag_and_interprets_nonzero() {
    for (byte, expected) in [(0, false), (1, true), (0x80, true)] {
        let mut data = 2_u16.to_le_bytes().to_vec();
        data.extend(0_i32.to_le_bytes());
        data.push(byte);
        let mut reader = Reader::new(&data);
        let result =
            read_edit_template_manager_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1)
                .unwrap();
        assert_eq!(result.tail_byte, Some(byte));
        assert_eq!(result.persisted_flag_is_set(), Some(expected));
        assert_eq!(reader.remaining(), 0);
    }
    let data = [1, 0, 0, 0, 0, 0];
    let result = read_edit_template_manager_body_10_1(
        &mut Reader::new(&data),
        1024,
        &mut RefTable10_1::new(),
        1,
    )
    .unwrap();
    assert_eq!(result.persisted_flag_is_set(), None);
}

#[test]
fn feature_id_set_retains_signed_values_and_duplicate_stream_entries() {
    let mut data = 1_u16.to_le_bytes().to_vec();
    data.extend(3_i32.to_le_bytes());
    for id in [-1_i32, 7, 7] {
        data.extend(id.to_le_bytes());
    }
    let mut reader = Reader::new(&data);
    let result = read_feature_id_set_body_10_1(&mut reader, &mut RefTable10_1::new(), 1).unwrap();
    assert_eq!(result.ids, [-1, 7, 7]);
    assert_eq!(reader.remaining(), 0);
    let mut truncated = Reader::new(&data[..data.len() - 1]);
    assert!(read_feature_id_set_body_10_1(&mut truncated, &mut RefTable10_1::new(), 1).is_err());
    assert_eq!(truncated.position(), 0);
}
