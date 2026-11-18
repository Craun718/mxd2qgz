//! MapElements.dll 10.1 的 Graticule、MapGrid 和 DMSGridLabel 持久化布局。

use crate::maps::map_frame::unsupported;
use crate::objects::display_symbol::{
    DisplaySlotKind10_1 as Kind, SymbolSlotFull10_1 as Symbol, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::number_format::{
    LATLON_FORMAT_CLASS_ID, LatLonFormatBody10_1, read_latlon_format_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub const GRATICULE_CLASS_ID: [u8; 16] = [
    0x8f, 0x2c, 0x76, 0x03, 0xd0, 0xf4, 0xd1, 0x11, 0xad, 0xe8, 8, 0, 9, 0xec, 0x73, 0x2a,
];

pub const DMS_GRID_LABEL_CLASS_ID: [u8; 16] = [
    6, 0xc5, 0x41, 0xce, 0xf9, 0x9d, 0xd2, 0x11, 0xaa, 0xde, 0, 0, 0, 0, 0, 0,
];

/// SimpleMapGridBorder：MapElements.dll 持久化虚表 0x7959dca0。
pub const SIMPLE_MAP_GRID_BORDER_CLASS_ID: [u8; 16] = [
    0xfb, 0xec, 0x81, 0xac, 0xe4, 0x9e, 0xd2, 0x11, 0xaa, 0xdf, 0, 0, 0, 0, 0, 0,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GridSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub graticule: Option<Box<Graticule10_1>>,
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
    pub line_5c: Symbol,
    pub double_54: u64,
    pub short_4c: u16,
    pub line_48: Symbol,
    pub double_44: u64,
    pub shorts_3c: [u16; 4],
    pub border: ObjectEntry10_1,
    pub simple_border: Option<Box<SimpleMapGridBorder10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMapGridBorder10_1 {
    pub version: u16,
    pub line: Symbol,
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

// 此入口只在具有上层事务的地图框内部调用。
pub(super) fn read_grid_slot_10_1(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<GridSlot10_1> {
    let entry = read_object_entry_10_1(r, refs)?;
    let graticule = match entry {
        ObjectEntry10_1::NewObject {
            class_id: GRATICULE_CLASS_ID,
            ..
        } => Some(Box::new(read_graticule(r, limit, refs)?)),
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IMapGrid", entry)),
        _ => None,
    };
    Ok(GridSlot10_1 { entry, graticule })
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

fn read_base_grid(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MapGrid10_1> {
    let version = read_version(r, 1, "MapGrid")?;
    let name = read_string_10_1(r, limit)?;
    let line_64 = symbol(r, limit, refs, Kind::LineSymbol)?;
    let marker_60 = symbol(r, limit, refs, Kind::MarkerSymbol)?;
    let label = read_object_entry_10_1(r, refs)?;
    let dms_label = match label {
        ObjectEntry10_1::NewObject {
            class_id: DMS_GRID_LABEL_CLASS_ID,
            ..
        } => Some(Box::new(read_dms_label(r, limit, refs)?)),
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IGridLabel", label)),
        _ => None,
    };
    let line_5c = symbol(r, limit, refs, Kind::LineSymbol)?;
    let double_54 = r.read_f64_le()?.to_bits();
    let short_4c = r.read_u16_le()?;
    let line_48 = symbol(r, limit, refs, Kind::LineSymbol)?;
    let double_44 = r.read_f64_le()?.to_bits();
    let shorts_3c = shorts(r)?;
    let border = read_object_entry_10_1(r, refs)?;
    let simple_border = match border {
        ObjectEntry10_1::NewObject {
            class_id: SIMPLE_MAP_GRID_BORDER_CLASS_ID,
            ..
        } => Some(Box::new(SimpleMapGridBorder10_1 {
            version: read_version(r, 1, "SimpleMapGridBorder")?,
            line: symbol(r, limit, refs, Kind::LineSymbol)?,
        })),
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IMapGridBorder", border)),
        _ => None,
    };
    Ok(MapGrid10_1 {
        version,
        name,
        line_64,
        marker_60,
        label,
        dms_label,
        line_5c,
        double_54,
        short_4c,
        line_48,
        double_44,
        shorts_3c,
        border,
        simple_border,
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
