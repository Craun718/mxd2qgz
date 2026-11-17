use mxd2qgz::{
    layers::feature_layer_tail::{NumberFormatSlot10_1, read_field_info_body_10_1},
    storage::object_refs::RefTable10_1,
    stream::reader::Reader,
};

fn string(value: &str) -> Vec<u8> {
    let units: Vec<_> = value.encode_utf16().chain([0]).collect();
    let mut bytes = ((units.len() * 2) as u32).to_le_bytes().to_vec();
    bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
    bytes
}

#[test]
fn fieldinfo_v0_v1_keep_the_unconditional_numberformat_slot() {
    // 10.1 Layer.dll FieldInfo::Load 依次读取版本、short、别名和
    // LoadObject(INumberFormat)；版本 0、1 也包含该对象槽。
    for version in [0_u16, 1] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(0xffff_u16.to_le_bytes());
        bytes.extend(string("alias"));
        bytes.extend([0x0d, 0x47, 0x4f, 0x7e, 0xf8, 0x4e, 0xd0, 0x11]);
        bytes.extend([0x9b, 0x5e, 0x00, 0x80, 0x5f, 0x7c, 0x7e, 0x9b]);
        bytes.extend(41_u32.to_le_bytes());
        let end = bytes.len();
        bytes.extend([0x5a; 4]);

        let mut refs = RefTable10_1::new();
        refs.register(41);
        let mut reader = Reader::new(&bytes);
        let body = read_field_info_body_10_1(&mut reader, 1024, &mut refs, 7).unwrap();

        assert_eq!(body.version, version);
        assert_eq!(body.alias, "alias");
        assert_eq!(body.short_14, None);
        assert_eq!(
            body.number_format,
            NumberFormatSlot10_1::BackReference {
                class_id: [
                    0x0d, 0x47, 0x4f, 0x7e, 0xf8, 0x4e, 0xd0, 0x11, 0x9b, 0x5e, 0x00, 0x80, 0x5f,
                    0x7c, 0x7e, 0x9b,
                ],
                key: 41,
            }
        );
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(4).unwrap(), [0x5a; 4]);
        assert_eq!(refs.len(), 1);
        assert!(!refs.contains(7));
        assert!(refs.contains(41));

        for cut in 0..end {
            let mut truncated = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            refs.register(41);
            assert!(
                read_field_info_body_10_1(&mut truncated, 1024, &mut refs, 7).is_err(),
                "v{version} 截断 {cut}"
            );
            assert_eq!(truncated.position(), 0);
            assert_eq!(refs.len(), 1);
            assert!(refs.contains(41));
            assert!(!refs.contains(7));
        }
    }
}
