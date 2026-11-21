//! 已确认的 NetworkAnalyst.dll 10.1 NALayer 持久化前缀。

use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

pub(crate) const NA_LAYER_CLASS_ID: [u8; 16] = [
    0x71, 0x85, 0xbd, 0x5f, 0x60, 0xa5, 0x4f, 0x49, 0x9f, 0xaf, 0x3e, 0xdc, 0xed, 0x5a, 0x78, 0x90,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaLayerPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub shorts: [u16; 3],
    pub double_bits: [u64; 2],
}

pub(super) fn read_na_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<NaLayerPrefix10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    if version as i16 > 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NALayer 版本超过 3",
        ));
    }
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let shorts = [
        trial.read_u16_le()?,
        trial.read_u16_le()?,
        trial.read_u16_le()?,
    ];
    let double_bits = [
        trial.read_f64_le()?.to_bits(),
        trial.read_f64_le()?.to_bits(),
    ];
    *reader = trial;
    Ok(NaLayerPrefix10_1 {
        version,
        name,
        shorts,
        double_bits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_prefix_and_leaves_nested_objects_for_the_host() {
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend([b'R', 0, 0, 0]);
        bytes.extend([1_u16, 2, 3].into_iter().flat_map(u16::to_le_bytes));
        bytes.extend([4_u64, 5].into_iter().flat_map(u64::to_le_bytes));
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_layer_prefix_10_1(&mut reader, 64).is_err());
            assert_eq!(reader.position(), 0);
        }

        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let result = read_na_layer_prefix_10_1(&mut reader, 64).unwrap();
        assert_eq!((result.version, result.name.as_str()), (3, "R"));
        assert_eq!(result.shorts, [1, 2, 3]);
        assert_eq!(result.double_bits, [4, 5]);
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }
}
