//! NetworkLayer.dll 10.1 的持久化前缀。
use super::na_layer::{NetworkDatasetName10_1, read_na_network_name_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkLayerPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub network: (ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>),
}

/// Load 0x7b2903d0 读取版本及名称，经 0x7b291140 读取 IName。
pub(super) fn read_network_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NetworkLayerPrefix10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    if version as i16 > 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NetworkLayer 版本超过 0",
        ));
    }
    let name = read_string_10_1(&mut trial, max)?;
    let network = read_na_network_name_10_1(&mut trial, max, refs)?;
    *reader = trial;
    Ok(NetworkLayerPrefix10_1 {
        version,
        name,
        network,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_network_name_reference_and_rejects_truncation() {
        let mut bytes = vec![0, 0, 4, 0, 0, 0, b'N', 0, 0, 0];
        bytes.extend([0x42; 16]);
        bytes.extend(5054_u32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        refs.register(5054);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_network_layer_prefix_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
        }
        let mut reader = Reader::new(&bytes);
        let body = read_network_layer_prefix_10_1(&mut reader, 128, &mut refs).unwrap();
        assert_eq!(body.name, "N");
        assert!(matches!(
            body.network.0,
            ObjectEntry10_1::BackReference { key: 5054, .. }
        ));
        assert_eq!(reader.remaining(), 0);
    }
}
