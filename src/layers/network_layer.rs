//! NetworkLayer.dll 10.1 的持久化前缀。
use super::feature_layer_tail::{
    DisplayFilterSlot10_1, LEGEND_GROUP_CLASS_ID, LegendGroupSlot10_1, RefSlot10_1,
    TRANSPARENCY_DISPLAY_FILTER_CLASS_ID, read_legend_group_body_10_1,
    read_transparency_display_filter_10_1,
};
use super::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use super::na_layer::{NetworkDatasetName10_1, read_na_network_name_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

const NETWORK_DIRTY_AREA_RENDERER_CLASS_ID: [u8; 16] = [
    0x68, 0x61, 0x8a, 0x3d, 0x05, 0x4b, 0xc9, 0x44, 0x92, 0xc0, 0x56, 0x40, 0xb4, 0x0e, 0x6c, 0xbe,
];

const SIMPLE_NETWORK_RENDERER_CLASS_ID: [u8; 16] = [
    0xd1, 0xbf, 0x30, 0x1c, 0x91, 0xdb, 0xf1, 0x48, 0xb3, 0xc1, 0x6a, 0x2f, 0xeb, 0x0c, 0x71, 0x04,
];

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleNetworkRenderer10_1 {
    pub version: u16,
    pub base_version: u16,
    pub name: String,
    pub modes: [i32; 2],
    pub secondary_name: String,
    pub symbols: [SymbolSlotFull10_1; 3],
    pub flags: Option<[u16; 3]>,
    pub sources: Vec<(String, u8, String)>,
    pub legend: LegendGroupSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkRendererEntry10_1 {
    pub selector: u16,
    pub entry: ObjectEntry10_1,
    pub dirty_area: Option<(u16, LegendGroupSlot10_1)>,
    pub simple: Option<Box<SimpleNetworkRenderer10_1>>,
    pub trailer: [u8; 16],
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkLayerPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub network: (ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>),
    pub display: Option<NetworkLayerDisplay10_1>,
    pub renderers: Vec<NetworkRendererEntry10_1>,
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
    let mut renderers = Vec::new();
    if let Some(display) = &display {
        for _ in 0..display.renderer_count.max(0) {
            let Ok(renderer) = read_renderer(&mut trial, max, refs) else {
                break;
            };
            renderers.push(renderer);
        }
    }
    *reader = trial;
    Ok(NetworkLayerPrefix10_1 {
        version,
        name,
        network,
        display,
        renderers,
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

/// 0x7b2903d0 的每项由 short、普通对象槽及原样 16 字节组成。
fn read_renderer(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NetworkRendererEntry10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let selector = trial.read_u16_le()?;
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let dirty_area = match entry {
        ObjectEntry10_1::NewObject {
            class_id: NETWORK_DIRTY_AREA_RENDERER_CLASS_ID,
            ..
        } => {
            let version = trial.read_u16_le()?;
            if version as i16 > 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NetworkDirtyAreaRenderer 版本超过 1",
                ));
            }
            let legend = read_network_legend(&mut trial, max, &mut keys)?;
            Some((version, legend))
        }
        ObjectEntry10_1::NewObject {
            class_id: SIMPLE_NETWORK_RENDERER_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "网络渲染器类未支持",
            ));
        }
        _ => None,
    };
    let simple = matches!(
        entry,
        ObjectEntry10_1::NewObject {
            class_id: SIMPLE_NETWORK_RENDERER_CLASS_ID,
            ..
        }
    )
    .then(|| read_simple_network_renderer(&mut trial, max, &mut keys).map(Box::new))
    .transpose()?;
    let trailer = trial.read_bytes(16)?.try_into().expect("固定 16 字节");
    *reader = trial;
    *refs = keys;
    Ok(NetworkRendererEntry10_1 {
        selector,
        entry,
        dirty_area,
        simple,
        trailer,
    })
}

/// 简单网络渲染器 Load 0x7b29e770 委派共用体 0x7b298990。
fn read_simple_network_renderer(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<SimpleNetworkRenderer10_1> {
    let version = reader.read_u16_le()?;
    let base_version = reader.read_u16_le()?;
    if version as i16 > 2 || base_version as i16 > 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "SimpleNetworkRenderer 版本超限",
        ));
    }
    let name = read_string_10_1(reader, max)?;
    let modes = [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32];
    let secondary_name = read_string_10_1(reader, max)?;
    let mut read_symbol = || {
        let body = read_symbol_slot_full_10_1(reader, max, refs, MAX_SYMBOL_NESTING_DEPTH)?;
        if slot_full_stop_10_1(&body, DisplaySlotKind10_1::Symbol).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "网络渲染器符号未完全读取",
            ));
        }
        Ok(body)
    };
    let symbols = [read_symbol()?, read_symbol()?, read_symbol()?];
    let flags = (base_version as i16 >= 2)
        .then(|| {
            Ok::<_, io::Error>([
                reader.read_u16_le()?,
                reader.read_u16_le()?,
                reader.read_u16_le()?,
            ])
        })
        .transpose()?;
    let mut sources = Vec::new();
    if base_version as i16 >= 3 {
        let count = (reader.read_u32_le()? as i32).max(0) as usize;
        if count > reader.remaining().min(max) / 13 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "网络渲染器来源计数超出范围",
            ));
        }
        for _ in 0..count {
            sources.push((
                read_string_10_1(reader, max)?,
                reader.read_bytes(1)?[0],
                read_string_10_1(reader, max)?,
            ));
        }
    }
    let legend = read_network_legend(reader, max, refs)?;
    Ok(SimpleNetworkRenderer10_1 {
        version,
        base_version,
        name,
        modes,
        secondary_name,
        symbols,
        flags,
        sources,
        legend,
    })
}

fn read_network_legend(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LegendGroupSlot10_1> {
    Ok(match read_object_entry_10_1(reader, refs)? {
        ObjectEntry10_1::Null => LegendGroupSlot10_1::Null,
        ObjectEntry10_1::BackReference {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        } => LegendGroupSlot10_1::BackReference {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        },
        ObjectEntry10_1::NewObject {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        } => {
            let body = read_legend_group_body_10_1(reader, max, refs, key)?;
            if body.stop.is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "网络渲染器图例未完全读取",
                ));
            }
            LegendGroupSlot10_1::LegendGroup(Box::new(body))
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "网络渲染器图例类未支持",
            ));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_renderer_reads_version_three_sources_with_item_rollback() {
        let mut bytes = 8_u16.to_le_bytes().to_vec();
        bytes.extend(SIMPLE_NETWORK_RENDERER_CLASS_ID);
        bytes.extend(5064_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        let empty = [2, 0, 0, 0, 0, 0];
        bytes.extend(empty);
        bytes.extend([0; 8]);
        bytes.extend(empty);
        bytes.extend([0; 48 + 6]);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(empty);
        bytes.push(1);
        bytes.extend(empty);
        bytes.extend([0; 16]);
        bytes.extend([0x33; 16]);
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_renderer(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let renderer = read_renderer(&mut reader, 128, &mut refs).unwrap();
        assert_eq!(
            renderer.simple.unwrap().sources,
            [(String::new(), 1, String::new())]
        );
        assert_eq!(renderer.trailer, [0x33; 16]);
        assert!(refs.contains(5064));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn dirty_area_renderer_commits_key_after_trailer() {
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend(NETWORK_DIRTY_AREA_RENDERER_CLASS_ID);
        bytes.extend(5057_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend([0x71; 16]);
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_renderer(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let body = read_renderer(&mut reader, 128, &mut refs).unwrap();
        assert_eq!(body.selector, 3);
        assert_eq!(body.dirty_area, Some((1, LegendGroupSlot10_1::Null)));
        assert_eq!(body.trailer, [0x71; 16]);
        assert!(refs.contains(5057));
        assert_eq!(reader.remaining(), 0);
    }

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
