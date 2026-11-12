use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

#[test]
fn numeric_reads_reject_every_truncated_prefix_without_consuming_input() {
    let bytes = [0xa5, 0x78, 0x56, 0x34, 0x12, 0, 0, 0xf0, 0x3f];
    for width in [2, 4, 8] {
        for available in 0..width {
            let mut reader = Reader::new(&bytes[..1 + available]);
            reader.read_bytes(1).unwrap();
            let error = match width {
                2 => reader.read_u16_le().unwrap_err(),
                4 => reader.read_u32_le().unwrap_err(),
                8 => reader.read_f64_le().unwrap_err(),
                _ => unreachable!(),
            };
            assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
            assert!(error.to_string().contains("offset 0x1"));
            assert_eq!(reader.position(), 1);
            assert_eq!(reader.remaining(), available);
            assert_eq!(
                reader.read_bytes(available).unwrap(),
                &bytes[1..1 + available]
            );
        }
    }
}

#[test]
fn floating_point_reads_preserve_ieee_bits_including_unknown_values() {
    // 输入中的非有限值交给字段语义层处理，通用读取器保留原始位模式。
    for bits in [
        0_u64,
        0x8000_0000_0000_0000,
        0x3ff0_0000_0000_0000,
        1,
        0x7fef_ffff_ffff_ffff,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_1234,
    ] {
        let bytes = bits.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_f64_le().unwrap().to_bits(), bits);
        assert_eq!(reader.position(), 8);
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn utf16_single_units_accept_exactly_non_surrogates() {
    for unit in 0..=u16::MAX {
        let bytes = unit.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        match char::from_u32(u32::from(unit)) {
            Some(expected) => {
                assert_eq!(reader.read_utf16_le(2).unwrap(), expected.to_string());
                assert_eq!(reader.position(), 2);
            }
            None => {
                assert_eq!(
                    reader.read_utf16_le(2).unwrap_err().kind(),
                    ErrorKind::InvalidData
                );
                assert_eq!(reader.position(), 0);
            }
        }
    }
}

#[test]
fn utf16_pairs_cover_supplementary_boundaries_and_failed_reads_are_retryable() {
    for scalar in [0x10000, 0x103ff, 0x10400, 0x1f600, 0x10ffff] {
        let expected = char::from_u32(scalar).unwrap().to_string();
        let bytes: Vec<_> = expected.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let mut reader = Reader::new(&bytes);
        // 高代理项不能单独解码；失败后仍可从原位置读取完整代理对。
        assert_eq!(
            reader.read_utf16_le(2).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.read_utf16_le(4).unwrap(), expected);
    }

    for units in [[0xdc00, 0xd800], [0xd800, 0x0041], [0xd800, 0xd800]] {
        let bytes: Vec<_> = [0x0041_u16]
            .into_iter()
            .chain(units)
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_utf16_le(2).unwrap(), "A");
        let error = reader.read_utf16_le(4).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("offset 0x2"));
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.read_bytes(4).unwrap(), &bytes[2..]);
    }
}

#[test]
fn nested_regions_cannot_consume_siblings_and_report_local_offsets() {
    let mut root = Reader::new(&[0xaa, 0x34, 0x12, 0x78, 0x56, 0xbb]);
    root.read_bytes(1).unwrap();
    let mut region = root.read_region(4).unwrap();
    let mut nested = region.read_region(2).unwrap();
    nested.read_bytes(1).unwrap();
    let error = nested.read_u16_le().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert!(error.to_string().contains("offset 0x1"));
    assert_eq!(nested.position(), 1);
    assert_eq!(nested.read_bytes(1).unwrap(), &[0x12]);
    assert_eq!(region.read_u16_le().unwrap(), 0x5678);
    assert!(region.read_region(usize::MAX).is_err());
    assert_eq!(region.position(), 4);
    assert_eq!(root.position(), 5);
    assert_eq!(root.read_bytes(1).unwrap(), &[0xbb]);
}
