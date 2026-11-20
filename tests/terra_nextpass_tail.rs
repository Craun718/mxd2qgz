use mxd2qgz::maps::map_object::{MapSuffixStop, read_map_suffix_10_1};
use mxd2qgz::storage::object_refs::{RefTable10_1, read_object_entry_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

fn sample() -> Vec<u8> {
    let mut doc = cfb::open("samples/测试底图.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

fn parse(bytes: &[u8]) -> (usize, MapSuffixStop, RefTable10_1) {
    let mut reader = Reader::new(bytes);
    let mut refs = RefTable10_1::new();
    reader.read_u32_le().unwrap();
    read_object_entry_10_1(&mut reader, &mut refs).unwrap();
    let result = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    (reader.position(), result.stop, refs)
}

#[test]
fn tail_fields_stop_at_each_incomplete_field_and_unknown_envelope() {
    let bytes = sample();
    let boundaries = [
        0x252be, 0x252c0, 0x252c8, 0x252ca, 0x252da, 0x252ea, 0x252f2,
    ];
    for pair in boundaries.windows(2) {
        for end in pair[0]..pair[1] {
            let (position, _, refs) = parse(&bytes[..end]);
            assert_eq!(position, pair[0], "截断于 {end:#x}");
            assert!(!refs.contains(2182));
        }
    }
    for (start, end, key) in [(0x252f2, 0x25340, 2182), (0x25340, 0x2538e, 2183)] {
        for cut in start..end {
            let (position, _, refs) = parse(&bytes[..cut]);
            assert_eq!(position, start, "对象截断于 {cut:#x}");
            assert!(!refs.contains(key));
        }
    }
    for cut in 0x25396..0x253b6 {
        let (position, _, _) = parse(&bytes[..cut]);
        assert_eq!(position, 0x25396 + ((cut - 0x25396) / 4) * 4);
    }
    let mut unknown = bytes.clone();
    unknown[0x252f2] ^= 0x80;
    let (position, stop, refs) = parse(&unknown);
    assert_eq!(position, 0x252f2);
    assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 24 });
    assert!(!refs.contains(2182));
    let (position, stop, refs) = parse(&bytes);
    assert_eq!(position, 0x25569);
    assert_eq!(stop, MapSuffixStop::Complete);
    assert!(refs.contains(2182));
    assert!(refs.contains(2183));
}

#[test]
fn map_masking_empty_prefix_and_nonempty_boundary() {
    let bytes = sample();
    for pair in [0x253b6, 0x253b8, 0x253bc, 0x253be].windows(2) {
        for cut in pair[0]..pair[1] {
            assert_eq!(parse(&bytes[..cut]).0, pair[0]);
        }
    }
    for cut in 0x253be..0x253c9 {
        assert_eq!(parse(&bytes[..cut]).0, 0x253be);
    }
    let mut nonempty = bytes.clone();
    nonempty[0x253b8..0x253bc].copy_from_slice(&1_i32.to_le_bytes());
    let (position, stop, _) = parse(&nonempty);
    assert_eq!(position, 0x253bc);
    assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 37 });
    let mut modified = bytes.clone();
    modified[0x253b6..0x253b8].copy_from_slice(&3_u16.to_le_bytes());
    assert_eq!(parse(&modified).0, 0x253b6);
    modified[0x253b6..0x253b8].copy_from_slice(&0xffff_u16.to_le_bytes());
    assert_eq!(parse(&modified).1, MapSuffixStop::Complete);
}

#[test]
fn map_masking_nonempty_back_reference_reaches_post_masking_fields() {
    let bytes = sample();
    let mut modified = bytes[..0x253b8].to_vec();
    modified.extend(1_i32.to_le_bytes());
    // 使用前面登记过的 Envelope 回引，只检验流结构，不断言接口兼容性。
    modified.extend(&bytes[0x252f2..0x25306]);
    modified.extend(0_i32.to_le_bytes());
    modified.extend(0xffff_u16.to_le_bytes());
    modified.extend(&bytes[0x253be..]);
    let expected = 0x25569 + 24;
    let (position, stop, _) = parse(&modified);
    assert_eq!(position, expected);
    assert_eq!(stop, MapSuffixStop::Complete);
}

#[test]
fn four_empty_envelopes_and_raw_values_are_consumed_in_order() {
    let mut bytes = sample();
    bytes.truncate(0x252be);
    bytes.extend(0x8123_u16.to_le_bytes());
    bytes.extend(0x7ff8_0000_0000_1234_u64.to_le_bytes());
    bytes.extend(0xfedc_u16.to_le_bytes());
    bytes.extend([0; 32]);
    bytes.extend(0x8000_0000_0000_0000_u64.to_le_bytes());
    bytes.extend([0; 32]);
    bytes.extend(0x7ff0_0000_0000_0001_u64.to_le_bytes());
    let end = bytes.len();
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    reader.read_u32_le().unwrap();
    read_object_entry_10_1(&mut reader, &mut refs).unwrap();
    let map = read_map_suffix_10_1(&mut reader, 4_000_000, &mut refs).unwrap();
    assert_eq!(reader.position(), end);
    assert_eq!(map.stop, MapSuffixStop::AfterBorderTail { stage: 27 });
    let tail = map.post_border_tail;
    assert_eq!(tail.short_32a, Some((0x252be, 0x8123)));
    assert_eq!(tail.double_320_bits, Some((0x252c0, 0x7ff8_0000_0000_1234)));
    assert_eq!(tail.short_328, Some((0x252c8, 0xfedc)));
    assert!(tail.scale_envelopes.iter().all(Option::is_some));
    assert_eq!(tail.scale_double_2b8_bits.unwrap().1, 0x8000_0000_0000_0000);
    assert_eq!(tail.scale_double_2d8_bits.unwrap().1, 0x7ff0_0000_0000_0001);
}

#[test]
fn a_layer_with_unverified_class_or_back_reference_keeps_masking_bytes() {
    let bytes = sample();
    // 流中第二条是对首条 FDO 已登记关联 FeatureLayer 的回引。
    // 改其 GUID 但保留键和长度，仍能读完图层列表，不能以错误类跨过遮罩。
    let mut modified = bytes.clone();
    modified[0x2d2c] ^= 0x80;
    let (position, stop, _) = parse(&modified);
    assert_eq!(position, 0x253be);
    assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 38 });
}

#[test]
fn map_versions_23_to_28_finish_at_their_last_native_field() {
    use mxd2qgz::maps::mxd_maps::parse_maps_10_1;

    let bytes = sample();
    for (version, end) in [
        (23_u16, 0x252be),
        (24, 0x252ca),
        (25, 0x252ca),
        (26, 0x25396),
        (27, 0x253b6),
        (28, 0x253be),
    ] {
        let mut maps = bytes[..end].to_vec();
        maps[24..26].copy_from_slice(&version.to_le_bytes());
        let mut mask_bytes = Vec::new();
        if version <= 27 {
            mask_bytes.extend(0_u32.to_le_bytes());
            mask_bytes.extend(0x1732_u16.to_le_bytes());
            if version >= 25 {
                mask_bytes.extend(0_u32.to_le_bytes());
            }
        }
        let mask_offset = 0x24f60;
        let end = end + mask_bytes.len();
        maps.splice(mask_offset..mask_offset, mask_bytes);
        assert_eq!(parse(&maps).0, end, "Map v{version}");
        assert_eq!(parse(&maps).1, MapSuffixStop::Complete, "Map v{version}");
        assert_ne!(parse(&maps[..end - 1]).1, MapSuffixStop::Complete);

        maps.extend(0x1234_u32.to_le_bytes());
        let report = parse_maps_10_1(&maps).unwrap();
        assert_eq!(report.confirmed_tail, Some((end, 0x1234)));
        assert!(report.map.unknown_fields.is_empty());

        if (25..=27).contains(&version) {
            maps[mask_offset + 6] = 1;
            let err = parse_maps_10_1(&maps).unwrap_err();
            assert!(err.to_string().contains("第二个 LONG 非零"));
        }

        if version == 28 {
            // 修改前面已登记的图层回引类标识，遮罩遍历不再能确认这条图层。
            maps[0x2d2c] ^= 0x80;
            let (position, stop, _) = parse(&maps);
            assert_eq!(position, end);
            assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 38 });
        }
    }
}
