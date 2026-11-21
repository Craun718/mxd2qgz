//! NetworkLayer.dll 10.1 的持久化前缀。
use super::feature_layer_tail::{
    DisplayFilterSlot10_1, RefSlot10_1, TRANSPARENCY_DISPLAY_FILTER_CLASS_ID,
    read_transparency_display_filter_10_1,
};
use super::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use super::na_layer::{NetworkDatasetName10_1, read_na_network_name_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkLayerPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub network: (ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>),
    pub display: Option<NetworkLayerDisplay10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkLayerDisplay10_1 {
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub filter: DisplayFilterSlot10_1,
    pub shorts: [u16; 4],
    pub secondary_name: String,
    pub scalars: [u64; 5],
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub description: String,
    pub renderer_count: i32,
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
    let display = read_display(&mut trial, max, refs).ok();
    *reader = trial;
    Ok(NetworkLayerPrefix10_1 {
        version,
        name,
        network,
        display,
    })
}

fn read_display(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NetworkLayerDisplay10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let spatial_reference = read_spatial_reference_entry_10_1(&mut trial, max, &mut keys)?;
    let filter = match read_object_entry_10_1(&mut trial, &mut keys)? {
        ObjectEntry10_1::Null => DisplayFilterSlot10_1::Reference(RefSlot10_1::Null),
        ObjectEntry10_1::BackReference { class_id, key } => {
            DisplayFilterSlot10_1::Reference(RefSlot10_1::BackReference { class_id, key })
        }
        ObjectEntry10_1::NewObject {
            class_id: TRANSPARENCY_DISPLAY_FILTER_CLASS_ID,
            key,
        } => read_transparency_display_filter_10_1(&mut trial, key)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NetworkLayer 显示过滤器未支持",
            ));
        }
    };
    let first = trial.read_u16_le()?;
    let secondary_name = read_string_10_1(&mut trial, max)?;
    let second = trial.read_u16_le()?;
    let a = trial.read_f64_le()?.to_bits();
    let b = trial.read_f64_le()?.to_bits();
    let shorts = [first, second, trial.read_u16_le()?, trial.read_u16_le()?];
    let c = trial.read_f64_le()?.to_bits();
    let extensions = read_layer_extensions_10_1(&mut trial, max, &mut keys)?;
    let description = read_string_10_1(&mut trial, max)?;
    let scalars = [
        a,
        b,
        c,
        trial.read_f64_le()?.to_bits(),
        trial.read_f64_le()?.to_bits(),
    ];
    let renderer_count = trial.read_u32_le()? as i32;
    *reader = trial;
    *refs = keys;
    Ok(NetworkLayerDisplay10_1 {
        spatial_reference,
        filter,
        shorts,
        secondary_name,
        scalars,
        extensions,
        description,
        renderer_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_fields_stop_at_first_renderer() {
        let mut bytes = vec![0; 32 + 2];
        bytes.extend([2, 0, 0, 0, 0, 0]);
        bytes.extend([0; 2 + 16 + 4 + 8 + 4]);
        bytes.extend([2, 0, 0, 0, 0, 0]);
        bytes.extend([0; 16]);
        bytes.extend(5_i32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_display(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let display = read_display(&mut reader, 128, &mut refs).unwrap();
        assert_eq!(display.renderer_count, 5);
        assert!(display.extensions.is_empty());
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }

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
