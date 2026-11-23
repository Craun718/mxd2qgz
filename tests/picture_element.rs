use mxd2qgz::maps::mxd_maps::parse_mxd_streams_10_1;
use mxd2qgz::maps::page_layout_prefix::PAGE_LAYOUT_CLASS_ID;
use mxd2qgz::maps::picture_element::{
    BMP_PICTURE_ELEMENT_CLASS_ID, EMF_PICTURE_ELEMENT_CLASS_ID, GIF_PICTURE_ELEMENT_CLASS_ID,
    JP2_PICTURE_ELEMENT_CLASS_ID, JPG_PICTURE_ELEMENT_CLASS_ID, PICTURE_ELEMENT_CLASS_ID,
    PNG_PICTURE_ELEMENT_CLASS_ID, PictureElementBase10_1, PictureElementKind10_1,
    TIF_PICTURE_ELEMENT_CLASS_ID,
};
use mxd2qgz::objects::element_body::{ElementBody10_1, read_element_body_of_class_10_1};
use mxd2qgz::objects::ole_font::STDPICTURE_CLASS_ID;
use mxd2qgz::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::stream::reader::Reader;

fn string(out: &mut Vec<u8>, value: &str) {
    let body: Vec<_> = value
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    out.extend((body.len() as u32).to_le_bytes());
    out.extend(body);
}

fn picture(version: u16, flags: u16, name: &str, embedded: bool) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    if version >= 3 {
        bytes.extend(0_u16.to_le_bytes()); // BaseFrameElement v0。
    }
    bytes.extend([0; 2 + 16]); // RectShapeElement v0，空几何。
    bytes.extend(4_u16.to_le_bytes());
    string(&mut bytes, name);
    string(&mut bytes, "");
    bytes.extend([0; 4]); // VT_EMPTY 与 AutoTransform。
    if version >= 3 {
        bytes.extend([0; 16 + 16 + 2]); // 空边框、空背景和 DraftMode。
    }
    if version >= 2 {
        bytes.extend(flags.to_le_bytes());
    }
    if embedded {
        bytes.extend(STDPICTURE_CLASS_ID);
        bytes.extend(9_u32.to_le_bytes());
        bytes.extend([0x6c, 0x74, 0, 0, 3, 0, 0, 0, 1, 2, 3]);
    }
    if version < 4 {
        bytes.extend(0x1234_u16.to_le_bytes());
    }
    if version >= 5 {
        for value in ["图片", "格式", "描述"] {
            string(&mut bytes, value);
        }
    }
    if version >= 6 {
        for value in [0.0_f64, 1.0, 2.0, 3.0] {
            bytes.extend(value.to_le_bytes());
        }
    }
    if version >= 7 {
        string(&mut bytes, "v7");
    }
    if version >= 8 {
        bytes.extend(0x6789_u16.to_le_bytes());
        string(&mut bytes, "v8");
    }
    if version >= 9 {
        string(&mut bytes, "v9");
    }
    bytes
}

#[test]
fn picture_versions_and_embedding_gates_leave_following_field_unread() {
    for (version, flags, name, embedded) in
        (0..=9).map(|version| (version, 0xffff, "", true)).chain([
            (1, 0, "old.bmp", false),
            (4, 1, "", true),
            (4, 0, "old.emf", false),
            (9, 0, "", false),
        ])
    {
        let mut bytes = picture(version, flags, name, embedded);
        let end = bytes.len();
        bytes.extend(0xdeadbeef_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let ElementBody10_1::PictureElement(body) = read_element_body_of_class_10_1(
            &BMP_PICTURE_ELEMENT_CLASS_ID,
            &mut reader,
            1024,
            &mut refs,
        )
        .unwrap() else {
            panic!("需要图片元素体");
        };
        assert_eq!(body.kind, PictureElementKind10_1::Bmp);
        assert_eq!(body.version, version);
        assert_eq!(body.embed_flag_bits, (version >= 2).then_some(flags));
        assert!(matches!(body.base, PictureElementBase10_1::Legacy { .. }) == (version <= 2));
        assert_eq!(body.legacy_short, (version < 4).then_some(0x1234));
        assert_eq!(
            body.picture
                .as_ref()
                .and_then(|(_, b)| b.as_ref())
                .map(|b| b.image_data.as_slice()),
            embedded.then_some([1, 2, 3].as_slice())
        );
        assert_eq!(body.string_19c.as_deref(), (version >= 9).then_some("v9"));
        assert_eq!(refs.contains(9), embedded);
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_u32_le().unwrap(), 0xdeadbeef);
    }
}

#[test]
fn picture_body_errors_restore_reader_and_nested_keys() {
    let bytes = picture(9, 0xffff, "", true);
    let nested_at = bytes
        .windows(16)
        .position(|window| window == STDPICTURE_CLASS_ID)
        .unwrap();
    let mut unknown = bytes.clone();
    unknown[nested_at..nested_at + 16].fill(0x55);
    let mut excess = bytes.clone();
    excess[nested_at + 24..nested_at + 28].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut bad_version = bytes.clone();
    bad_version[..2].copy_from_slice(&10_u16.to_le_bytes());

    for data in
        (0..bytes.len())
            .map(|cut| &bytes[..cut])
            .chain([unknown.as_slice(), &excess, &bad_version])
    {
        let mut reader = Reader::new(data);
        let mut refs = RefTable10_1::new();
        refs.register(6);
        assert!(
            read_element_body_of_class_10_1(
                &EMF_PICTURE_ELEMENT_CLASS_ID,
                &mut reader,
                1024,
                &mut refs
            )
            .is_err()
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(6));
    }
}

#[test]
fn layout_dispatches_all_picture_classes_and_resolves_element_and_picture_refs() {
    let classes = [
        (PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Picture),
        (EMF_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Emf),
        (BMP_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Bmp),
        (JPG_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Jpg),
        (GIF_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Gif),
        (TIF_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Tif),
        (PNG_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Png),
        (JP2_PICTURE_ELEMENT_CLASS_ID, PictureElementKind10_1::Jp2),
    ];
    let mut layout = PAGE_LAYOUT_CLASS_ID.to_vec();
    layout.extend(1_u32.to_le_bytes());
    layout.extend(9_u16.to_le_bytes());
    layout.extend([0; 4]);
    layout.extend(((classes.len() + 1) as u32).to_le_bytes());
    for (index, (class_id, _)) in classes.iter().enumerate() {
        let mut body = picture(4, 0xffff, "", true);
        if index > 0 {
            body.truncate(body.len() - 11); // 同一 StdPicture 的后继槽只写回引头。
        }
        layout.extend(((20 + body.len()) as u32).to_le_bytes());
        layout.extend([0; 4]);
        layout.extend(class_id);
        layout.extend((100 + index as u32).to_le_bytes());
        layout.extend(body);
    }
    layout.extend(22_u64.to_le_bytes());
    layout.extend([0xa5; 16]); // 回引按键识别已读取的元素。
    layout.extend(100_u32.to_le_bytes());
    let suffix_offset = layout.len();
    layout.extend([0x34, 0x12]);

    let parsed = parse_mxd_streams_10_1(&[0; 8], Some(&layout))
        .unwrap()
        .page_layout
        .unwrap();
    assert_eq!(parsed.consumed_elements, 9, "{:?}", parsed.stop);
    assert_eq!(parsed.end_offset, layout.len());
    assert_eq!(
        parsed.element_reference_suffixes,
        [(8, suffix_offset, vec![0x34, 0x12])]
    );
    assert_eq!(
        parsed
            .pictures
            .iter()
            .map(|(_, body)| body.kind)
            .collect::<Vec<_>>(),
        classes.map(|(_, kind)| kind)
    );
    assert!(matches!(
        parsed.pictures[1].1.picture,
        Some((ObjectEntry10_1::BackReference { key: 9, .. }, None))
    ));
    assert!(matches!(
        &parsed.indirect_elements[..],
        [(8, _, ObjectEntry10_1::BackReference { key: 100, .. })]
    ));
}
