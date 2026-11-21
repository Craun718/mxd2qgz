//! TerrainLayer.dll 与 Terrain.dll 10.1 普通对象读取。

use super::feature_layer_tail::{
    DisplayFilterSlot10_1, LegendGroupPrefix10_1, TRANSPARENCY_DISPLAY_FILTER_CLASS_ID,
    read_transparency_display_filter_10_1,
};
use super::image_server_rules::{RuleSlot10_1, slot};
use super::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use super::tin_layer::{TinRenderer10_1, read_legend_group, read_renderer};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_dataset_name_slot_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub(crate) const TERRAIN_LAYER_CLASS_ID: [u8; 16] = [
    0xdd, 0x19, 0x7c, 0x5c, 0xf0, 0xd2, 0x31, 0x4a, 0x9c, 0x70, 0xf8, 0x7e, 0x3a, 0x20, 0x3c, 0x28,
];
const TERRAIN_NAME_CLASS_ID: [u8; 16] = [
    0x11, 0x93, 0xed, 0x4c, 0x66, 0x65, 0xf6, 0x44, 0xb1, 0x35, 0x65, 0x7d, 0x6f, 0x48, 0x14, 0x3e,
];

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainName10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub dataset: DatasetNameMoreSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainLayer10_1 {
    pub version: i16,
    pub name: String,
    pub initial_shorts: [u16; 2],
    pub initial_double: u64,
    pub source: RuleSlot10_1<TerrainName10_1>,
    pub source_shorts: [u16; 2],
    pub extent: Option<TerrainExtent10_1>,
    pub display_shorts: [u16; 3],
    pub display_doubles: [u64; 2],
    pub display_string: String,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub renderers: Vec<RuleSlot10_1<TinRenderer10_1>>,
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub v3: Option<(u16, i32, u64)>,
    pub v4_double: Option<u64>,
    pub v5: Option<TerrainDisplay10_1>,
    pub v7: Option<(String, [u64; 2])>,
    pub legend_group: Option<RuleSlot10_1<LegendGroupPrefix10_1>>,
    pub v9_short: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainExtent10_1 {
    pub flag: u16,
    pub values: [u64; 5],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainDisplay10_1 {
    pub filter: RuleSlot10_1<DisplayFilterSlot10_1>,
    pub shorts: [u16; 3],
    pub expression_string: String,
}

fn unsupported(slot: &str) -> io::Error {
    io::Error::new(
        ErrorKind::Unsupported,
        format!("TerrainLayer 的 {slot} 槽未闭合"),
    )
}

fn read_version(r: &mut Reader<'_>, max: i16) -> io::Result<i16> {
    let version = r.read_u16_le()? as i16;
    if version > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Terrain 对象版本超出 10.1 上限",
        ));
    }
    Ok(version)
}

fn read_double(r: &mut Reader<'_>) -> io::Result<u64> {
    r.read_f64_le().map(f64::to_bits)
}

fn read_shorts(r: &mut Reader<'_>) -> io::Result<[u16; 3]> {
    Ok([r.read_u16_le()?, r.read_u16_le()?, r.read_u16_le()?])
}

fn read_name(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<TerrainName10_1> {
    if class_id != TERRAIN_NAME_CLASS_ID {
        return Err(unsupported("IName"));
    }
    let version = read_version(r, 1)?;
    let strings = [
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
    ];
    // TerrainName 在装载后经 IDatasetName 取得工作空间，不另读工作空间槽。
    let mut stop = None;
    let dataset = read_dataset_name_slot_10_1(r, max, refs, &mut stop)?;
    if stop.is_some() {
        return Err(unsupported("IDatasetName"));
    }
    Ok(TerrainName10_1 {
        version,
        strings,
        dataset,
    })
}

pub(crate) fn read_terrain_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<TerrainLayer10_1> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let r = &mut trial;
    let version = read_version(r, 9)?;
    let name = read_string_10_1(r, max)?;
    let initial_shorts = [r.read_u16_le()?, r.read_u16_le()?];
    let initial_double = read_double(r)?;
    let source = slot(r, &mut refs, |r, refs, class_id, _| {
        read_name(r, max, refs, class_id)
    })?;
    let source_shorts = [r.read_u16_le()?, r.read_u16_le()?];
    let extent = if version >= 6 {
        let flag = r.read_u16_le()?;
        let mut values = [0; 5];
        for value in &mut values {
            *value = read_double(r)?;
        }
        Some(TerrainExtent10_1 { flag, values })
    } else {
        None
    };
    let display_shorts = read_shorts(r)?;
    let display_doubles = [read_double(r)?, read_double(r)?];
    let display_string = read_string_10_1(r, max)?;
    let spatial_reference = read_spatial_reference_entry_10_1(r, max, &mut refs)?;
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    if count > r.remaining() / 16 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "TerrainLayer 渲染器计数超出剩余字节",
        ));
    }
    if count > max / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TerrainLayer 渲染器计数超出限额",
        ));
    }
    let mut renderers = Vec::new();
    for _ in 0..count {
        renderers.push(slot(r, &mut refs, |r, refs, class_id, _| {
            read_renderer(r, max, refs, class_id)
        })?);
    }
    let extensions = if version >= 2 {
        read_layer_extensions_10_1(r, max, &mut refs)?
    } else {
        Vec::new()
    };
    let v3 = if version >= 3 {
        Some((r.read_u16_le()?, r.read_u32_le()? as i32, read_double(r)?))
    } else {
        None
    };
    let v4_double = (version >= 4).then(|| read_double(r)).transpose()?;
    let v5 = if version >= 5 {
        let filter = slot(r, &mut refs, |r, _, class_id, key| {
            if class_id != TRANSPARENCY_DISPLAY_FILTER_CLASS_ID {
                return Err(unsupported("IDisplayFilter"));
            }
            read_transparency_display_filter_10_1(r, key)
        })?;
        Some(TerrainDisplay10_1 {
            filter,
            shorts: read_shorts(r)?,
            expression_string: read_string_10_1(r, max)?,
        })
    } else {
        None
    };
    let v7 = if version >= 7 {
        Some((
            read_string_10_1(r, max)?,
            [read_double(r)?, read_double(r)?],
        ))
    } else {
        None
    };
    let legend_group = (version >= 8)
        .then(|| read_legend_group(r, max, &mut refs))
        .transpose()?;
    let v9_short = (version >= 9).then(|| r.read_u16_le()).transpose()?;
    *reader = trial;
    *table = refs;
    Ok(TerrainLayer10_1 {
        version,
        name,
        initial_shorts,
        initial_double,
        source,
        source_shorts,
        extent,
        display_shorts,
        display_doubles,
        display_string,
        spatial_reference,
        renderers,
        extensions,
        v3,
        v4_double,
        v5,
        v7,
        legend_group,
        v9_short,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
    use crate::layers::tin_layer::TERRAIN_DIRTY_AREA;
    use crate::objects::dataset_name::FEATURE_DATASET_NAME_CLASS_ID;

    fn string(bytes: &mut Vec<u8>, value: &str) {
        let raw: Vec<_> = value
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        bytes.extend((raw.len() as u32).to_le_bytes());
        bytes.extend(raw);
    }

    fn fixture(version: i16) -> Vec<u8> {
        let mut bytes = version.to_le_bytes().to_vec();
        string(&mut bytes, "地形");
        bytes.extend([1, 0, 2, 0]);
        bytes.extend(3.0_f64.to_le_bytes());
        bytes.extend(TERRAIN_NAME_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["数据源", "地形集", "类型"] {
            string(&mut bytes, value);
        }
        bytes.extend(FEATURE_DATASET_NAME_CLASS_ID);
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["数据集", "", "", ""] {
            string(&mut bytes, value);
        }
        bytes.extend([0; 1 + 16]);
        bytes.extend([4, 0, 5, 0]);
        if version >= 6 {
            bytes.extend(0_u16.to_le_bytes());
            for value in [6.0_f64, 7.0, 8.0, 9.0, 10.0] {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes.extend([11, 0, 12, 0, 13, 0]);
        bytes.extend(14.0_f64.to_le_bytes());
        bytes.extend(15.0_f64.to_le_bytes());
        string(&mut bytes, "显示");
        bytes.extend([0; 16]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(TERRAIN_DIRTY_AREA);
        bytes.extend(63_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 3 * 16]);
        bytes.extend(23_u16.to_le_bytes());
        string(&mut bytes, "脏区");
        bytes.extend(24_u16.to_le_bytes());
        if version >= 2 {
            bytes.extend(0_i32.to_le_bytes());
        }
        if version >= 3 {
            bytes.extend(16_u16.to_le_bytes());
            bytes.extend((-17_i32).to_le_bytes());
            bytes.extend(18.0_f64.to_le_bytes());
        }
        if version >= 4 {
            bytes.extend(19.0_f64.to_le_bytes());
        }
        if version >= 5 {
            bytes.extend([0; 16 + 6]);
            string(&mut bytes, "表达式");
        }
        if version >= 7 {
            string(&mut bytes, "路径");
            bytes.extend(20.0_f64.to_le_bytes());
            bytes.extend(21.0_f64.to_le_bytes());
        }
        if version >= 8 {
            bytes.extend([0; 16]);
        }
        if version >= 9 {
            bytes.extend(22_u16.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn terrain_dispatch_reads_all_versions_with_dataset_keys() {
        for version in 0..=9 {
            let mut bytes = fixture(version);
            bytes.push(0xee);
            let mut r = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let LayerBodyPrefix10_1::TerrainLayer(body) =
                read_complete_layer_body_10_1(&mut r, TERRAIN_LAYER_CLASS_ID, 4096, &mut refs)
                    .unwrap()
            else {
                panic!("TerrainLayer 未分派");
            };
            assert_eq!(body.version, version);
            assert_eq!(body.source.body.unwrap().strings[0], "数据源");
            assert!(refs.contains(61));
            assert!(refs.contains(62));
            assert_eq!(
                body.extent.map(|extent| extent.values[4]),
                (version >= 6).then_some(10.0_f64.to_bits())
            );
            assert_eq!(body.display_string, "显示");
            let Some(TinRenderer10_1::Simple(renderer)) = body.renderers[0].body.as_deref() else {
                panic!("TerrainDirtyAreaRenderer 未分派");
            };
            assert!(refs.contains(63));
            assert_eq!((renderer.first_short, renderer.last_short), (23, 24));
            assert_eq!(renderer.label, "脏区");
            assert_eq!(
                body.v3,
                (version >= 3).then_some((16, -17, 18.0_f64.to_bits()))
            );
            assert_eq!(
                body.v5.as_ref().map(|v5| v5.expression_string.as_str()),
                (version >= 5).then_some("表达式")
            );
            assert_eq!(
                body.v7.as_ref().map(|v7| v7.0.as_str()),
                (version >= 7).then_some("路径")
            );
            assert_eq!(body.legend_group.is_some(), version >= 8);
            assert_eq!(body.v9_short, (version >= 9).then_some(22));
            assert_eq!(r.read_bytes(1).unwrap(), &[0xee]);
        }
    }

    #[test]
    fn terrain_truncation_and_unknown_dataset_roll_back_keys() {
        let mut bytes = fixture(9);
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut r = Reader::new(&bytes[..end]);
            assert!(
                read_terrain_layer_10_1(&mut r, 4096, &mut refs).is_err(),
                "{end}"
            );
            assert_eq!(r.position(), 0);
            assert!(refs.is_empty());
        }
        let offset = bytes
            .windows(16)
            .position(|part| part == FEATURE_DATASET_NAME_CLASS_ID)
            .unwrap();
        bytes[offset] ^= 0xff;
        let mut r = Reader::new(&bytes);
        assert_eq!(
            read_terrain_layer_10_1(&mut r, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(r.position(), 0);
        assert!(refs.is_empty());
    }
}
