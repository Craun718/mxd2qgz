use mxd2qgz::stream::reader::Reader;
use mxd2qgz::stream::variant_stream::{VariantValue10_1, VariantValue10_8, read_value_10_1};
use std::io::ErrorKind;

#[test]
fn variant_stream_10_1_reads_scalars_and_null_objects() {
    for value_type in [9_u16, 13] {
        let mut bytes = value_type.to_le_bytes().to_vec();
        bytes.extend([0_u8; 16]);
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_value_10_1(&mut reader, 0).unwrap(),
            VariantValue10_1::NullObject { value_type }
        );
        assert_eq!(reader.position(), 18);
        assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
    }
    let mut reader = Reader::new(&[3, 0, 1, 2, 3, 4]);
    assert_eq!(
        read_value_10_1(&mut reader, 0).unwrap(),
        VariantValue10_1::Scalar(VariantValue10_8::Raw4([1, 2, 3, 4]))
    );
}

#[test]
fn variant_stream_10_1_failures_preserve_cursor() {
    let mut bytes = vec![9, 0];
    bytes.extend([0; 16]);
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        assert_eq!(
            read_value_10_1(&mut reader, 0).unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
        assert_eq!(reader.position(), 0);
    }
    bytes[2] = 1;
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_value_10_1(&mut reader, 0).unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(reader.position(), 0);
    // Variant::Read 支持 VT_CY，VariantStream 全局读取器不支持。
    let mut reader = Reader::new(&[6, 0]);
    assert_eq!(
        read_value_10_1(&mut reader, 0).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
}
