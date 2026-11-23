use mxd2qgz::stream::{
    native_string::{read_string_10_1, read_string_10_8},
    reader::Reader,
};
use std::io::ErrorKind;

fn encoded(text: &str) -> Vec<u8> {
    let payload: Vec<_> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes
}

#[test]
fn reads_writer_subset_and_stops_before_following_fields() {
    for text in ["", "10.8", "地图", "😀𠀀", "尾部\u{ffff}"] {
        let mut bytes = encoded(text);
        let end = bytes.len();
        bytes.extend([0x13, 0, 0xdb, 8]);
        let mut reader = Reader::new(&bytes);
        assert_eq!(read_string_10_8(&mut reader, end - 4).unwrap(), text);
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_u16_le().unwrap(), 0x13);
    }
}

#[test]
fn rejects_every_truncated_prefix_and_payload_without_advancing() {
    let full = encoded("中文😀");
    for end in 0..full.len() {
        let mut bytes = vec![0xaa, 0xbb];
        bytes.extend(&full[..end]);
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(2).unwrap();
        assert_eq!(
            read_string_10_8(&mut reader, 1024).unwrap_err().kind(),
            ErrorKind::UnexpectedEof,
            "end={end}"
        );
        assert_eq!(reader.position(), 2);
    }
}

#[test]
fn rejects_lengths_and_resource_exhaustion_before_allocation() {
    for length in [0u32, 1, 3, 5, u32::MAX, u32::MAX - 1, 0x8000_0000] {
        let bytes = length.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_string_10_8(&mut reader, 4096).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        assert_eq!(reader.position(), 0);
    }
    let bytes = encoded("地图");
    let mut reader = Reader::new(&bytes);
    assert!(read_string_10_8(&mut reader, 5).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(read_string_10_8(&mut reader, 6).unwrap(), "地图");
    let bytes = (u32::MAX - 1).to_le_bytes();
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_string_10_8(&mut reader, usize::MAX)
            .unwrap_err()
            .kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn rejects_unsafe_native_inputs_with_reader_relative_location() {
    for payload in [
        vec![1, 0],                   // 没有终止零。
        vec![0, 0, 1, 0, 0, 0],       // 提前终止。
        vec![0, 0xd8, 0, 0],          // 孤立高代理项。
        vec![0, 0xdc, 0, 0],          // 孤立低代理项。
        vec![0, 0xd8, 1, 0, 0, 0],    // 代理项后为普通字符。
        vec![0, 0xdc, 0, 0xd8, 0, 0], // 逆序代理对。
    ] {
        let mut bytes = vec![0xff];
        bytes.extend((payload.len() as u32).to_le_bytes());
        bytes.extend(payload);
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(1).unwrap();
        let err = read_string_10_8(&mut reader, 1024).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert!(err.to_string().contains("offset 0x1"));
        assert_eq!(reader.position(), 1);
    }
}

#[test]
fn child_region_cannot_borrow_a_terminator_from_its_sibling() {
    let bytes = encoded("a");
    let mut parent = Reader::new(&bytes);
    let mut child = parent.read_region(bytes.len() - 2).unwrap();
    assert_eq!(
        read_string_10_8(&mut child, 1024).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(child.position(), 0);
    assert_eq!(parent.read_u16_le().unwrap(), 0);
}

#[test]
fn version_10_1_writer_subset_and_invalid_inputs() {
    for text in ["", "10.1", "地图", "😀𠀀"] {
        let bytes = encoded(text);
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_string_10_1(&mut reader, bytes.len() - 4).unwrap(),
            text
        );
        assert_eq!(reader.position(), bytes.len());
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert_eq!(
                read_string_10_1(&mut reader, 1024).unwrap_err().kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(reader.position(), 0);
        }
    }
    for bytes in [
        vec![0, 0, 0, 0],
        vec![3, 0, 0, 0],
        vec![2, 0, 0, 0, 1, 0],
        vec![4, 0, 0, 0, 0, 0xd8, 0, 0],
        u32::MAX.to_le_bytes().to_vec(),
    ] {
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_string_10_1(&mut reader, 1024).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        assert_eq!(reader.position(), 0);
    }
    let bytes = encoded("地图");
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_string_10_1(&mut reader, 5).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn bstr_stream_uses_code_units_and_zero_count_has_no_body() {
    use mxd2qgz::stream::native_string::read_bstr_string_10_1;
    let mut bytes = 0_u32.to_le_bytes().to_vec();
    bytes.extend(3_u32.to_le_bytes());
    bytes.extend("中🗺".encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
    bytes.push(0xa7);
    let mut reader = Reader::new(&bytes);
    assert_eq!(read_bstr_string_10_1(&mut reader, 0).unwrap(), "");
    assert_eq!(reader.position(), 4);
    assert_eq!(read_bstr_string_10_1(&mut reader, 8).unwrap(), "中🗺");
    assert_eq!(reader.read_bytes(1).unwrap(), &[0xa7]);
    for cut in 4..bytes.len() - 1 {
        let mut reader = Reader::new(&bytes[..cut]);
        reader.read_bytes(4).unwrap();
        assert!(read_bstr_string_10_1(&mut reader, 8).is_err());
        assert_eq!(reader.position(), 4);
    }
    let mut reader = Reader::new(&bytes);
    reader.read_bytes(4).unwrap();
    assert!(read_bstr_string_10_1(&mut reader, 7).is_err());
    assert_eq!(reader.position(), 4);
}

#[test]
fn version_10_1_and_10_8_writer_subsets_match() {
    for text in ["", "8.1", "9.3", "10.1", "地图😀"] {
        let bytes = encoded(text);
        let mut reader_10_1 = Reader::new(&bytes);
        let mut reader_10_8 = Reader::new(&bytes);
        assert_eq!(
            read_string_10_1(&mut reader_10_1, bytes.len() - 4).unwrap(),
            read_string_10_8(&mut reader_10_8, bytes.len() - 4).unwrap()
        );
        assert_eq!(reader_10_1.position(), bytes.len());
        assert_eq!(reader_10_8.position(), bytes.len());
    }

    for bytes in [
        vec![0, 0, 0, 0],
        vec![3, 0, 0, 0, 1, 0],
        vec![4, 0, 0, 0, 0, 0xd8, 0, 0],
        u32::MAX.to_le_bytes().to_vec(),
    ] {
        let mut reader_10_1 = Reader::new(&bytes);
        let mut reader_10_8 = Reader::new(&bytes);
        assert_eq!(
            read_string_10_1(&mut reader_10_1, 1024).unwrap_err().kind(),
            read_string_10_8(&mut reader_10_8, 1024).unwrap_err().kind()
        );
        assert_eq!(reader_10_1.position(), 0);
        assert_eq!(reader_10_8.position(), 0);
    }
}
