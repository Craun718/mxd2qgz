//! MapElements.dll 10.1 的网格、标签及网格边框持久化布局。

use crate::layers::feature_layer::FeatureClassNamePrefix10_1;
use crate::layers::feature_layer_tail::{
    NumberFormatSlot10_1, read_number_format_body_for_class_10_1,
};
use crate::maps::map_frame::unsupported;
use crate::objects::dataset_name_more::{
    DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, read_feature_source_name_slot_10_1,
};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1 as Kind, SymbolSlotFull10_1 as Symbol, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::objects::number_format::{
    LATLON_FORMAT_CLASS_ID, LatLonFormatBody10_1, read_latlon_format_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub const GRATICULE_CLASS_ID: [u8; 16] = [
    0x8f, 0x2c, 0x76, 0x03, 0xd0, 0xf4, 0xd1, 0x11, 0xad, 0xe8, 8, 0, 9, 0xec, 0x73, 0x2a,
];
pub const MEASURED_GRID_CLASS_ID: [u8; 16] = [
    0x90, 0x2c, 0x76, 0x03, 0xd0, 0xf4, 0xd1, 0x11, 0xad, 0xe8, 8, 0, 9, 0xec, 0x73, 0x2a,
];
pub const INDEX_GRID_CLASS_ID: [u8; 16] = [
    0x91, 0x2c, 0x76, 0x03, 0xd0, 0xf4, 0xd1, 0x11, 0xad, 0xe8, 8, 0, 9, 0xec, 0x73, 0x2a,
];
pub const CUSTOM_OVERLAY_GRID_CLASS_ID: [u8; 16] = [
    0x8a, 0x1c, 0x50, 0xff, 0x4b, 0xd7, 0xd2, 0x11, 0x9f, 0x43, 0, 0xc0, 0x4f, 0x6b, 0xc7, 0x8e,
];
pub const MGRS_GRID_CLASS_ID: [u8; 16] = [
    0x26, 0xd5, 0xf4, 0x35, 0x5d, 0x89, 0x85, 0x4f, 0xb9, 0x7f, 0x1a, 0x3b, 0x84, 0xc6, 0x29, 0xad,
];

pub const DMS_GRID_LABEL_CLASS_ID: [u8; 16] = [
    6, 0xc5, 0x41, 0xce, 0xf9, 0x9d, 0xd2, 0x11, 0xaa, 0xde, 0, 0, 0, 0, 0, 0,
];
pub const FORMATTED_GRID_LABEL_CLASS_ID: [u8; 16] = [
    7, 0xc5, 0x41, 0xce, 0xf9, 0x9d, 0xd2, 0x11, 0xaa, 0xde, 0, 0, 0, 0, 0, 0,
];
pub const MIXED_FONT_GRID_LABEL_CLASS_ID: [u8; 16] = [
    8, 0xc5, 0x41, 0xce, 0xf9, 0x9d, 0xd2, 0x11, 0xaa, 0xde, 0, 0, 0, 0, 0, 0,
];
pub const CORNER_GRID_LABEL_CLASS_ID: [u8; 16] = [
    0x1a, 0xb6, 0x52, 0xfd, 0xcd, 0x71, 0x08, 0x41, 0xa9, 0x16, 0xf8, 0x18, 0x96, 0x94, 0x04, 0xea,
];

/// SimpleMapGridBorder：MapElements.dll 持久化虚表 0x7959dca0。
pub const SIMPLE_MAP_GRID_BORDER_CLASS_ID: [u8; 16] = [
    0xfb, 0xec, 0x81, 0xac, 0xe4, 0x9e, 0xd2, 0x11, 0xaa, 0xdf, 0, 0, 0, 0, 0, 0,
];
pub const CALIBRATED_MAP_GRID_BORDER_CLASS_ID: [u8; 16] = [
    0xb0, 0x16, 0xa4, 0x6c, 0x60, 0xe1, 0xd2, 0x11, 0x9f, 0x4e, 0, 0xc0, 0x4f, 0x6b, 0xc7, 0x8e,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GridSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub graticule: Option<Box<Graticule10_1>>,
    pub measured_grid: Option<Box<MeasuredGrid10_1>>,
    pub index_grid: Option<Box<IndexGrid10_1>>,
    pub custom_overlay_grid: Option<Box<CustomOverlayGrid10_1>>,
    pub mgrs_grid: Option<Box<MgrsGrid10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomOverlayGrid10_1 {
    pub version: i16,
    pub base: MapGrid10_1,
    pub source: ObjectEntry10_1,
    pub feature_class_name: Option<Box<FeatureClassNamePrefix10_1>>,
    pub other_source_name: Option<Box<DatasetNameMoreBody10_1>>,
    pub string_44: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MgrsGrid10_1 {
    pub version: i16,
    pub base: MapGrid10_1,
    pub long_ac: i32,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub short_44: u16,
    pub text_40: Symbol,
    pub line_3c: Symbol,
    pub double_34: u64,
    pub short_2c: u16,
    pub text_28: Symbol,
    pub longs: [i32; 2],
    pub short_1c: Option<u16>,
    pub v3: Option<MgrsGridV3Fields10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MgrsGridV3Fields10_1 {
    pub long_18: i32,
    pub line_14: Symbol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredGrid10_1 {
    pub version: i16,
    pub base: MapGrid10_1,
    pub doubles: [u64; 4],
    pub long_34: i32,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub short_30: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexGrid10_1 {
    pub version: i16,
    pub base: MapGrid10_1,
    pub longs: [i32; 2],
    pub label_counts: [i32; 2],
    pub labels: [Vec<String>; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Graticule10_1 {
    pub version: u16,
    pub base: MapGrid10_1,
    // 下列偏移相对于 Graticule 持久化视图；未知语义保留原值。
    pub short_1d0: u16,
    pub doubles_1cc: [u64; 4],
    pub long_16c: u32,
    pub short_168: Option<u16>,
    pub v3: Option<GraticuleV3Fields10_1>,
    pub double_24: Option<u64>,
    pub v5: Option<(u16, [u64; 2])>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraticuleV3Fields10_1 {
    pub doubles_54: [u64; 2],
    pub line: Symbol,
    pub marker: Symbol,
    pub double_3c: u64,
    pub short_34: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapGrid10_1 {
    pub version: u16,
    pub name: String,
    pub line_64: Symbol,
    pub marker_60: Symbol,
    pub label: ObjectEntry10_1,
    pub dms_label: Option<Box<DmsGridLabel10_1>>,
    pub numeric_label: Option<Box<NumericGridLabel10_1>>,
    pub line_5c: Symbol,
    pub double_54: u64,
    pub short_4c: u16,
    pub line_48: Symbol,
    pub double_44: u64,
    pub shorts_3c: [u16; 4],
    pub border: ObjectEntry10_1,
    pub simple_border: Option<Box<SimpleMapGridBorder10_1>>,
    pub calibrated_border: Option<Box<CalibratedMapGridBorder10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMapGridBorder10_1 {
    pub version: u16,
    pub line: Symbol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibratedMapGridBorder10_1 {
    pub version: i16,
    pub fills: [Symbol; 2],
    pub doubles: [u64; 2],
    pub short_1c: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DmsGridLabel10_1 {
    pub version: u16,
    pub long_08: u32,
    pub text_symbols: [Symbol; 3],
    pub shorts_18: [u16; 2],
    pub format: ObjectEntry10_1,
    pub format_body: Option<Box<LatLonFormatBody10_1>>,
    pub double_24: u64,
    pub shorts_2c: Option<[u16; 5]>,
    pub short_36: Option<u16>,
    pub short_38: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumericGridLabel10_1 {
    pub version: i16,
    pub text_symbol: Symbol,
    pub format: NumberFormatSlot10_1,
    pub double_bits: u64,
    pub tail: NumericGridLabelTail10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericGridLabelTail10_1 {
    Formatted {
        short_18: Option<u16>,
    },
    MixedFont {
        secondary_symbol: Symbol,
        shorts_20: Option<[u16; 5]>,
        short_10: Option<u16>,
    },
    Corner {
        secondary_symbol: Symbol,
        shorts_1c: [u16; 5],
        shorts_26: [u16; 4],
        longs: [i32; 2],
        strings: Option<[String; 3]>,
    },
}

// 此入口只在具有上层事务的地图框内部调用。
pub(super) fn read_grid_slot_10_1(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<GridSlot10_1> {
    let mut entry = read_object_entry_10_1(r, refs)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = refs.registered_class(*key)
    {
        if !matches!(
            registered,
            GRATICULE_CLASS_ID
                | MEASURED_GRID_CLASS_ID
                | INDEX_GRID_CLASS_ID
                | CUSTOM_OVERLAY_GRID_CLASS_ID
                | MGRS_GRID_CLASS_ID
        ) {
            return Err(unsupported("IMapGrid", entry));
        }
        *class_id = registered;
    }
    let mut grid = GridSlot10_1 {
        entry,
        graticule: None,
        measured_grid: None,
        index_grid: None,
        custom_overlay_grid: None,
        mgrs_grid: None,
    };
    match entry {
        ObjectEntry10_1::NewObject {
            class_id: GRATICULE_CLASS_ID,
            ..
        } => grid.graticule = Some(Box::new(read_graticule(r, limit, refs)?)),
        ObjectEntry10_1::NewObject {
            class_id: MEASURED_GRID_CLASS_ID,
            ..
        } => {
            grid.measured_grid = Some(Box::new(read_measured_grid(r, limit, refs)?));
        }
        ObjectEntry10_1::NewObject {
            class_id: INDEX_GRID_CLASS_ID,
            ..
        } => {
            grid.index_grid = Some(Box::new(read_index_grid(r, limit, refs)?));
        }
        ObjectEntry10_1::NewObject {
            class_id: CUSTOM_OVERLAY_GRID_CLASS_ID,
            ..
        } => {
            grid.custom_overlay_grid = Some(Box::new(read_custom_overlay_grid(r, limit, refs)?));
        }
        ObjectEntry10_1::NewObject {
            class_id: MGRS_GRID_CLASS_ID,
            ..
        } => {
            grid.mgrs_grid = Some(Box::new(read_mgrs_grid(r, limit, refs)?));
        }
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IMapGrid", entry)),
        _ => {}
    }
    Ok(grid)
}

fn read_custom_overlay_grid(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<CustomOverlayGrid10_1> {
    let version = read_signed_version(r, 1, "CustomOverlayGrid")?;
    let base = read_base_grid(r, limit, refs)?;
    let header = read_header_10_1(&mut r.clone())?;
    let mut stop = None;
    let name = read_feature_source_name_slot_10_1(r, limit, refs, &mut stop)?;
    if let Some(stop) = stop {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("CustomOverlayGrid IName 未读完: {stop:?}"),
        ));
    }
    let (source, body) = match name {
        DatasetNameMoreSlot10_1::Null => (ObjectEntry10_1::Null, None),
        DatasetNameMoreSlot10_1::BackReference { class_id, key } => {
            (ObjectEntry10_1::BackReference { class_id, key }, None)
        }
        DatasetNameMoreSlot10_1::Body(body) => {
            let ObjectHeader::Object { class_id, key } = header else {
                unreachable!("已读取带键的要素来源名称");
            };
            (ObjectEntry10_1::NewObject { class_id, key }, Some(body))
        }
        DatasetNameMoreSlot10_1::Unsupported { class_id, key } => {
            return Err(unsupported(
                "CustomOverlayGrid IName",
                ObjectEntry10_1::NewObject { class_id, key },
            ));
        }
    };
    let (feature_class_name, other_source_name) = match body {
        Some(DatasetNameMoreBody10_1::FeatureClassName(body)) => (Some(body), None),
        other => (None, other.map(Box::new)),
    };
    Ok(CustomOverlayGrid10_1 {
        version,
        base,
        source,
        feature_class_name,
        other_source_name,
        string_44: read_string_10_1(r, limit)?,
    })
}

fn read_mgrs_grid(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MgrsGrid10_1> {
    let version = read_signed_version(r, 3, "MgrsGrid")?;
    Ok(MgrsGrid10_1 {
        version,
        base: read_base_grid(r, limit, refs)?,
        long_ac: r.read_u32_le()? as i32,
        spatial_reference: read_spatial_reference_entry_10_1(r, limit, refs)?,
        short_44: r.read_u16_le()?,
        text_40: symbol(r, limit, refs, Kind::Symbol)?,
        line_3c: symbol(r, limit, refs, Kind::LineSymbol)?,
        double_34: r.read_f64_le()?.to_bits(),
        short_2c: r.read_u16_le()?,
        text_28: symbol(r, limit, refs, Kind::Symbol)?,
        longs: [r.read_u32_le()? as i32, r.read_u32_le()? as i32],
        short_1c: (version >= 2).then(|| r.read_u16_le()).transpose()?,
        v3: if version >= 3 {
            Some(MgrsGridV3Fields10_1 {
                long_18: r.read_u32_le()? as i32,
                line_14: symbol(r, limit, refs, Kind::LineSymbol)?,
            })
        } else {
            None
        },
    })
}

fn read_measured_grid(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MeasuredGrid10_1> {
    let version = read_signed_version(r, 2, "MeasuredGrid")?;
    Ok(MeasuredGrid10_1 {
        version,
        base: read_base_grid(r, limit, refs)?,
        doubles: doubles(r)?,
        long_34: r.read_u32_le()? as i32,
        spatial_reference: read_spatial_reference_entry_10_1(r, limit, refs)?,
        short_30: (version >= 2).then(|| r.read_u16_le()).transpose()?,
    })
}

fn read_index_grid(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<IndexGrid10_1> {
    let version = read_signed_version(r, 1, "IndexGrid")?;
    let base = read_base_grid(r, limit, refs)?;
    let longs = [r.read_u32_le()? as i32, r.read_u32_le()? as i32];
    let mut label_counts = [0; 2];
    let mut labels = [Vec::new(), Vec::new()];
    for (raw, strings) in label_counts.iter_mut().zip(&mut labels) {
        *raw = r.read_u32_le()? as i32;
        let count = (*raw).max(0) as usize;
        if count > r.remaining().min(limit) / 6 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "IndexGrid 标签数量超出可读范围",
            ));
        }
        for _ in 0..count {
            strings.push(read_string_10_1(r, limit)?);
        }
    }
    Ok(IndexGrid10_1 {
        version,
        base,
        longs,
        label_counts,
        labels,
    })
}

fn read_graticule(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Graticule10_1> {
    let version = read_version(r, 5, "Graticule")?;
    let base = read_base_grid(r, limit, refs)?;
    let short_1d0 = r.read_u16_le()?;
    let doubles_1cc = doubles(r)?;
    let long_16c = r.read_u32_le()?;
    let short_168 = (version >= 2).then(|| r.read_u16_le()).transpose()?;
    let v3 = if version >= 3 {
        Some(GraticuleV3Fields10_1 {
            doubles_54: doubles(r)?,
            line: symbol(r, limit, refs, Kind::LineSymbol)?,
            marker: symbol(r, limit, refs, Kind::MarkerSymbol)?,
            double_3c: r.read_f64_le()?.to_bits(),
            short_34: r.read_u16_le()?,
        })
    } else {
        None
    };
    let double_24 = (version >= 4)
        .then(|| r.read_f64_le().map(f64::to_bits))
        .transpose()?;
    let v5 = if version >= 5 {
        Some((r.read_u16_le()?, doubles(r)?))
    } else {
        None
    };
    Ok(Graticule10_1 {
        version,
        base,
        short_1d0,
        doubles_1cc,
        long_16c,
        short_168,
        v3,
        double_24,
        v5,
    })
}

fn read_grid_member_entry(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    interface: &str,
    supports: impl FnOnce([u8; 16]) -> bool,
) -> io::Result<ObjectEntry10_1> {
    let mut entry = read_object_entry_10_1(r, refs)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = refs.registered_class(*key)
    {
        if !supports(registered) {
            return Err(unsupported(interface, entry));
        }
        *class_id = registered;
    }
    Ok(entry)
}

fn read_base_grid(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MapGrid10_1> {
    let version = read_version(r, 1, "MapGrid")?;
    let name = read_string_10_1(r, limit)?;
    let line_64 = symbol(r, limit, refs, Kind::LineSymbol)?;
    let marker_60 = symbol(r, limit, refs, Kind::MarkerSymbol)?;
    let label = read_grid_member_entry(r, refs, "IGridLabel", |class| {
        matches!(
            class,
            DMS_GRID_LABEL_CLASS_ID
                | FORMATTED_GRID_LABEL_CLASS_ID
                | MIXED_FONT_GRID_LABEL_CLASS_ID
                | CORNER_GRID_LABEL_CLASS_ID
        )
    })?;
    let mut dms_label = None;
    let mut numeric_label = None;
    match label {
        ObjectEntry10_1::NewObject {
            class_id: DMS_GRID_LABEL_CLASS_ID,
            ..
        } => dms_label = Some(Box::new(read_dms_label(r, limit, refs)?)),
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (FORMATTED_GRID_LABEL_CLASS_ID
                | MIXED_FONT_GRID_LABEL_CLASS_ID
                | CORNER_GRID_LABEL_CLASS_ID),
            ..
        } => numeric_label = Some(Box::new(read_numeric_label(r, limit, refs, class_id)?)),
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IGridLabel", label)),
        _ => {}
    };
    let line_5c = symbol(r, limit, refs, Kind::LineSymbol)?;
    let double_54 = r.read_f64_le()?.to_bits();
    let short_4c = r.read_u16_le()?;
    let line_48 = symbol(r, limit, refs, Kind::LineSymbol)?;
    let double_44 = r.read_f64_le()?.to_bits();
    let shorts_3c = shorts(r)?;
    let border = read_grid_member_entry(r, refs, "IMapGridBorder", |class| {
        matches!(
            class,
            SIMPLE_MAP_GRID_BORDER_CLASS_ID | CALIBRATED_MAP_GRID_BORDER_CLASS_ID
        )
    })?;
    let mut simple_border = None;
    let mut calibrated_border = None;
    match border {
        ObjectEntry10_1::NewObject {
            class_id: SIMPLE_MAP_GRID_BORDER_CLASS_ID,
            ..
        } => {
            simple_border = Some(Box::new(SimpleMapGridBorder10_1 {
                version: read_version(r, 1, "SimpleMapGridBorder")?,
                line: symbol(r, limit, refs, Kind::LineSymbol)?,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: CALIBRATED_MAP_GRID_BORDER_CLASS_ID,
            ..
        } => {
            calibrated_border = Some(Box::new(CalibratedMapGridBorder10_1 {
                version: read_signed_version(r, 1, "CalibratedMapGridBorder")?,
                fills: [
                    symbol(r, limit, refs, Kind::FillSymbol)?,
                    symbol(r, limit, refs, Kind::FillSymbol)?,
                ],
                doubles: doubles(r)?,
                short_1c: r.read_u16_le()?,
            }));
        }
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IMapGridBorder", border)),
        _ => {}
    }
    Ok(MapGrid10_1 {
        version,
        name,
        line_64,
        marker_60,
        label,
        dms_label,
        numeric_label,
        line_5c,
        double_54,
        short_4c,
        line_48,
        double_44,
        shorts_3c,
        border,
        simple_border,
        calibrated_border,
    })
}

fn read_numeric_label(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<NumericGridLabel10_1> {
    let (max, name) = match class_id {
        FORMATTED_GRID_LABEL_CLASS_ID => (2, "FormattedGridLabel"),
        MIXED_FONT_GRID_LABEL_CLASS_ID => (3, "MixedFontGridLabel"),
        CORNER_GRID_LABEL_CLASS_ID => (2, "CornerGridLabel"),
        _ => unreachable!("调用方已检查网格标签类"),
    };
    let version = read_signed_version(r, max, name)?;
    let text_symbol = symbol(r, limit, refs, Kind::Symbol)?;
    let format_entry = read_object_entry_10_1(r, refs)?;
    let format = match format_entry {
        ObjectEntry10_1::Null => NumberFormatSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            NumberFormatSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            read_number_format_body_for_class_10_1(r, limit, refs, class_id, key)?
                .ok_or_else(|| unsupported("INumberFormat", format_entry))?
        }
    };
    let double_bits = r.read_f64_le()?.to_bits();
    let tail = match class_id {
        FORMATTED_GRID_LABEL_CLASS_ID => NumericGridLabelTail10_1::Formatted {
            short_18: (version >= 2).then(|| r.read_u16_le()).transpose()?,
        },
        MIXED_FONT_GRID_LABEL_CLASS_ID => NumericGridLabelTail10_1::MixedFont {
            secondary_symbol: symbol(r, limit, refs, Kind::Symbol)?,
            shorts_20: (version >= 2).then(|| shorts(r)).transpose()?,
            short_10: (version >= 3).then(|| r.read_u16_le()).transpose()?,
        },
        CORNER_GRID_LABEL_CLASS_ID => NumericGridLabelTail10_1::Corner {
            secondary_symbol: symbol(r, limit, refs, Kind::Symbol)?,
            shorts_1c: shorts(r)?,
            shorts_26: shorts(r)?,
            longs: [r.read_u32_le()? as i32, r.read_u32_le()? as i32],
            strings: if version >= 2 {
                Some([
                    read_string_10_1(r, limit)?,
                    read_string_10_1(r, limit)?,
                    read_string_10_1(r, limit)?,
                ])
            } else {
                None
            },
        },
        _ => unreachable!("调用方已检查网格标签类"),
    };
    Ok(NumericGridLabel10_1 {
        version,
        text_symbol,
        format,
        double_bits,
        tail,
    })
}

fn read_dms_label(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DmsGridLabel10_1> {
    let version = read_version(r, 4, "DMSGridLabel")?;
    let long_08 = r.read_u32_le()?;
    let text_symbols = [
        symbol(r, limit, refs, Kind::Symbol)?,
        symbol(r, limit, refs, Kind::Symbol)?,
        symbol(r, limit, refs, Kind::Symbol)?,
    ];
    let shorts_18 = shorts(r)?;
    let format = read_object_entry_10_1(r, refs)?;
    let format_body = match format {
        ObjectEntry10_1::NewObject {
            class_id: LATLON_FORMAT_CLASS_ID,
            key,
        } => Some(Box::new(read_latlon_format_body_10_1(r, refs, key)?)),
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("ILatLonFormat", format)),
        _ => None,
    };
    Ok(DmsGridLabel10_1 {
        version,
        long_08,
        text_symbols,
        shorts_18,
        format,
        format_body,
        double_24: r.read_f64_le()?.to_bits(),
        shorts_2c: (version >= 2).then(|| shorts(r)).transpose()?,
        short_36: (version >= 3).then(|| r.read_u16_le()).transpose()?,
        short_38: (version >= 4).then(|| r.read_u16_le()).transpose()?,
    })
}

fn symbol(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    kind: Kind,
) -> io::Result<Symbol> {
    let value = read_symbol_slot_full_10_1(r, limit, refs, 16)?;
    if let Some(stop) = slot_full_stop_10_1(&value, kind) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("网格符号体未完整：{stop:?}"),
        ));
    }
    let valid = match (&value, kind) {
        (Symbol::Null | Symbol::BackReference { .. }, _) => true,
        (Symbol::TextSymbol { .. }, Kind::Symbol) => true,
        (
            Symbol::SimpleFillSymbol(_)
            | Symbol::MultiLayerFillSymbol(_)
            | Symbol::LineFillSymbol(_)
            | Symbol::MarkerFillSymbol(_)
            | Symbol::GradientFillSymbol(_)
            | Symbol::DotDensityFillSymbol(_),
            Kind::FillSymbol,
        ) => true,
        (
            Symbol::SimpleLineSymbol(_)
            | Symbol::MultiLayerLineSymbol(_)
            | Symbol::CartographicLineSymbol(_)
            | Symbol::MarkerLineSymbol(_)
            | Symbol::HashLineSymbol(_)
            | Symbol::PictureLineSymbol(_),
            Kind::LineSymbol,
        ) => true,
        (
            Symbol::SimpleMarkerSymbol(_)
            | Symbol::MultiLayerMarkerSymbol(_)
            | Symbol::CharacterMarkerSymbol(_)
            | Symbol::PictureMarkerSymbol(_)
            | Symbol::ArrowMarkerSymbol(_),
            Kind::MarkerSymbol,
        ) => true,
        _ => false,
    };
    if !valid {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "网格符号类与槽的接口不相符",
        ));
    }
    Ok(value)
}

fn read_version(r: &mut Reader<'_>, max: u16, name: &str) -> io::Result<u16> {
    let value = r.read_u16_le()?;
    if value > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("不支持的 {name} 版本 {value}"),
        ));
    }
    Ok(value)
}

fn read_signed_version(r: &mut Reader<'_>, max: i16, name: &str) -> io::Result<i16> {
    let value = r.read_u16_le()? as i16;
    if value > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("不支持的 {name} 版本 {value}"),
        ));
    }
    Ok(value)
}

fn shorts<const N: usize>(r: &mut Reader<'_>) -> io::Result<[u16; N]> {
    let mut values = [0; N];
    for value in &mut values {
        *value = r.read_u16_le()?;
    }
    Ok(values)
}

fn doubles<const N: usize>(r: &mut Reader<'_>) -> io::Result<[u64; N]> {
    let mut values = [0; N];
    for value in &mut values {
        *value = r.read_f64_le()?.to_bits();
    }
    Ok(values)
}
