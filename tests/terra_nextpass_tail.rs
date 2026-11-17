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
    assert_eq!(position, 0x253be);
    assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 38 });
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
    let mut nonempty = bytes.clone();
    nonempty[0x253b8..0x253bc].copy_from_slice(&1_i32.to_le_bytes());
    let (position, stop, _) = parse(&nonempty);
    assert_eq!(position, 0x253bc);
    assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 37 });
    for version in [3_u16, 0xffff] {
        let mut bad = bytes.clone();
        bad[0x253b6..0x253b8].copy_from_slice(&version.to_le_bytes());
        assert_eq!(parse(&bad).0, 0x253b6);
    }
}

#[test]
fn map_masking_nonempty_back_reference_then_stops_at_layers() {
    let bytes = sample();
    let mut modified = bytes[..0x253b8].to_vec();
    modified.extend(1_i32.to_le_bytes());
    // 使用前面登记过的 Envelope 回引，只检验流结构，不断言接口兼容性。
    modified.extend(&bytes[0x252f2..0x25306]);
    modified.extend(0_i32.to_le_bytes());
    modified.extend(0xffff_u16.to_le_bytes());
    modified.extend(&bytes[0x253be..]);
    let expected = 0x253b8 + 4 + 20 + 4 + 2;
    let (position, stop, _) = parse(&modified);
    assert_eq!(position, expected);
    assert_eq!(stop, MapSuffixStop::AfterBorderTail { stage: 38 });
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
