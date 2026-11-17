//! 安装介质局部对象验证；依赖 temp 中已提取的流，默认不运行。
use mxd2qgz::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use mxd2qgz::objects::ole_font::{STDPICTURE_CLASS_ID, read_std_picture_body_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

#[test]
#[ignore = "需要本地安装介质提取流"]
fn africa_symbol_versions_and_nested_boundaries() {
    for (path, start, end, version, key) in [
        (
            "temp/arcmap101-media-streams/Africa.mxd.Maps",
            0x526,
            0x887,
            3,
            47,
        ),
        (
            "temp/arcmap108-media-streams/africa.mxt.2319C344_B117_4BE0_8B98_93D6E4E94502.Maps",
            0x47e,
            0x743,
            2,
            45,
        ),
    ] {
        let bytes = std::fs::read(path).unwrap();
        let mut reader = Reader::new(&bytes[start..end]);
        let mut refs = RefTable10_1::new();
        let slot = read_symbol_slot_full_10_1(&mut reader, 1_000_000, &mut refs, 16).unwrap();
        assert!(slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol).is_none());
        assert_eq!(reader.position(), end - start);
        let SymbolSlotFull10_1::MultiLayerMarkerSymbol(body) = slot else {
            panic!("符号类型不符")
        };
        assert_eq!(
            (body.version, body.key, body.layer_count),
            (version, key, 2)
        );
        assert!(
            body.layers
                .iter()
                .all(|layer| matches!(layer, SymbolSlotFull10_1::CharacterMarkerSymbol(_)))
        );
    }
}

#[test]
#[ignore = "需要本地安装介质提取流"]
fn africa_stdpicture_bmp_and_truncation() {
    let bytes = std::fs::read(
        "temp/arcmap108-media-streams/africa.mxt.2319C344_B117_4BE0_8B98_93D6E4E94502.PageLayout",
    )
    .unwrap();
    let (head, start, end) = (0x922, 0x936, 0x37554);
    assert_eq!(&bytes[head..head + 16], &STDPICTURE_CLASS_ID);
    assert_eq!(
        u32::from_le_bytes(bytes[head + 16..start].try_into().unwrap()),
        537
    );
    let mut reader = Reader::new(&bytes[start..end]);
    let picture = read_std_picture_body_10_1(&mut reader, 224_278).unwrap();
    assert_eq!(reader.position(), end - start);
    assert_eq!((picture.word2, picture.data_size), (0, 224_278));
    assert_eq!(&picture.image_data[..2], b"BM");
    assert_eq!(
        u32::from_le_bytes(picture.image_data[2..6].try_into().unwrap()),
        picture.data_size
    );
    for len in [0, 1, 2, 3, 4, 7, 8, 9, end - start - 1] {
        let mut reader = Reader::new(&bytes[start..start + len]);
        assert_eq!(
            read_std_picture_body_10_1(&mut reader, 224_278)
                .unwrap_err()
                .kind(),
            ErrorKind::UnexpectedEof
        );
        assert_eq!(reader.position(), 0);
    }
    let mut reader = Reader::new(&bytes[start..end]);
    assert_eq!(
        read_std_picture_body_10_1(&mut reader, 224_277)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
}
