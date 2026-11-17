//! StdPicture 外层读取契约；图像字节为合成数据，不代表有效图片。

use mxd2qgz::objects::ole_font::read_std_picture_body_10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

fn body(word2: u16, data: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x6c, 0x74];
    bytes.extend(word2.to_le_bytes());
    bytes.extend((data.len() as u32).to_le_bytes());
    bytes.extend(data);
    bytes
}

#[test]
fn preserves_all_dispatch_prefixes_and_unknown_data() {
    // 这里只检查外层字节保留，不宣称这些前缀是有效图像。
    for data in [
        &[0x42, 0x4d, 0x12][..],
        &[0x47, 0x49, 0x12],
        &[0x67, 0x69, 0x12],
        &[0xff, 0xd8, 0x12],
        &[1, 0, 0x12],
        &[0xd7, 0xcd, 0x12],
        &[0, 0, 0x12],
        &[0x12, 0x34, 0x12],
        &[0x6c, 0x74, 0x12],
        &[0x42],
        &[],
    ] {
        for word2 in [0, 1, 0xffff] {
            let mut bytes = body(word2, data);
            let end = bytes.len();
            bytes.extend([0xaa, 0xbb]);
            let mut reader = Reader::new(&bytes);
            let parsed = read_std_picture_body_10_1(&mut reader, data.len()).unwrap();
            assert_eq!(parsed.word2, word2);
            assert_eq!(parsed.data_size as usize, data.len());
            assert_eq!(parsed.image_data, data);
            assert_eq!(reader.position(), end);
        }
    }
}

#[test]
fn truncation_and_limit_preserve_nonzero_cursor() {
    let complete = body(0x1234, &[0xff, 0xd8, 1, 2]);
    for end in 0..complete.len() {
        let mut bytes = vec![0x77];
        bytes.extend(&complete[..end]);
        let mut reader = Reader::new(&bytes);
        reader.read_bytes(1).unwrap();
        assert_eq!(
            read_std_picture_body_10_1(&mut reader, 4)
                .unwrap_err()
                .kind(),
            ErrorKind::UnexpectedEof
        );
        assert_eq!(reader.position(), 1);
    }
    let mut bytes = vec![0x77];
    bytes.extend(&complete);
    let mut reader = Reader::new(&bytes);
    reader.read_bytes(1).unwrap();
    assert_eq!(
        read_std_picture_body_10_1(&mut reader, 3)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 1);
}
