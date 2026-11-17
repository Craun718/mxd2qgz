use mxd2qgz::maps::map_decor::read_map_decor_slot_10_1;
use mxd2qgz::maps::map_object::read_map_suffix_10_1;
use mxd2qgz::objects::element_decor::{
    DecorKind10_1, DecorSlot10_1, SYMBOL_BACKGROUND_CLASS_ID, SYMBOL_BORDER_CLASS_ID,
};
use mxd2qgz::storage::object_refs::{RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

fn object(class_id: [u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(1_f64.to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend(4_f64.to_le_bytes());
    bytes
}

#[test]
fn slots_commit_only_complete_known_objects() {
    for (kind, id) in [
        (DecorKind10_1::Border, SYMBOL_BORDER_CLASS_ID),
        (DecorKind10_1::Background, SYMBOL_BACKGROUND_CLASS_ID),
    ] {
        let bytes = object(id, 50000);
        for cut in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            let result = read_map_decor_slot_10_1(&mut reader, &mut refs, 4096, kind);
            assert!(result.is_err() || result.unwrap().is_none(), "cut={cut}");
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        assert!(
            read_map_decor_slot_10_1(&mut reader, &mut refs, 4096, kind)
                .unwrap()
                .is_some()
        );
        assert_eq!(reader.position(), bytes.len());
        assert!(refs.contains(50000));
        let mut backref = Reader::new(&bytes[..20]);
        assert!(matches!(
            read_map_decor_slot_10_1(&mut backref, &mut refs, 4096, kind).unwrap(),
            Some(DecorSlot10_1::BackReference { key: 50000, .. })
        ));
        let mut wrong = bytes.clone();
        wrong[0] ^= 0x80;
        let mut reader = Reader::new(&wrong);
        assert!(
            read_map_decor_slot_10_1(&mut reader, &mut refs, 4096, kind)
                .unwrap()
                .is_none()
        );
        assert_eq!(reader.position(), 0);
        let mut nested = bytes.clone();
        nested[30..46].fill(0x55);
        nested.splice(46..46, 50001_u32.to_le_bytes());
        let mut reader = Reader::new(&nested);
        let mut refs = RefTable10_1::new();
        assert!(
            read_map_decor_slot_10_1(&mut reader, &mut refs, 4096, kind)
                .unwrap()
                .is_none()
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn map_reaches_later_fields_after_both_nonempty_slots() {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let border = object(SYMBOL_BORDER_CLASS_ID, 50000);
    let background = object(SYMBOL_BACKGROUND_CLASS_ID, 50001);
    bytes.splice(0x25259..0x25269, background.clone());
    bytes.splice(0x25004..0x25014, border.clone());
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    reader.read_u32_le().unwrap();
    read_object_entry_10_1(&mut reader, &mut refs).unwrap();
    let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    assert!(matches!(
        map.post_illumination_border,
        Some((0x25004, DecorSlot10_1::Border(_)))
    ));
    assert!(matches!(
        map.post_border_tail.post_transform_background,
        Some((_, DecorSlot10_1::Background(_)))
    ));
    assert!(map.post_border_tail.post_transform_overposter.is_some());
    assert_eq!(
        reader.position(),
        0x253be + border.len() + background.len() - 32
    );
    assert!(refs.contains(50000) && refs.contains(50001));
}
