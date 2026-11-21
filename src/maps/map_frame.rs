//! 10.1 MapFrame 及其委派的 BaseFrameElement 对象体。
//! MapElements.dll 0x79570a20、LayerLib.dll 0x7869b410 及其助手 0x786d00d0。

use crate::maps::page_layout_prefix::PageLayoutMapFramePrefix10_1;
use crate::objects::element_body::{
    ElementPropertiesPrefix10_1, PrefixOutcome10_1, read_element_properties_prefix_10_1,
};
use crate::objects::element_decor::{DecorKind10_1, DecorSlot10_1, read_decor_slot_10_1};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::geometry::{GeometryBody10_1, read_geometry_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// MapElements.dll LocatorRectangle::GetClassID 0x7956ab00。
pub const LOCATOR_RECTANGLE_CLASS_ID: [u8; 16] = [
    0xe2, 0xca, 0xff, 0x83, 0xca, 0xed, 0xd0, 0x11, 0x86, 0x83, 0, 0, 0xf8, 0x75, 0x17, 0x20,
];

#[derive(Debug, Clone, PartialEq)]
pub struct MapFrame10_1 {
    pub prefix: PageLayoutMapFramePrefix10_1,
    /// IMap 槽中首次定义的完整 Map。
    pub embedded_map: Option<
        Box<(
            super::map_object::MapPrefix10_1,
            super::map_object::MapSuffix10_1,
        )>,
    >,
    pub extent_settings: Option<MapFrameExtent10_1>,
    pub legacy_frame: Option<MapFrameLegacy10_1>,
    pub grids: Vec<super::map_grid::GridSlot10_1>,
    pub locator_rectangles: Vec<LocatorRectangleSlot10_1>,
    pub base: BaseFrameElement10_1,
    pub version_three_tail: Option<MapFrameVersionThree10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapFrameExtent10_1 {
    pub mode_bits: u32,
    pub scale_bits: u64,
    pub envelope: ObjectEntry10_1,
    pub envelope_body: Option<Envelope10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapFrameLegacy10_1 {
    pub geometry: FrameGeometrySlot10_1,
    pub flags: [u16; 2],
    pub border: DecorSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapFrameVersionThree10_1 {
    pub map_frame: ObjectEntry10_1,
    pub mode_bits: u32,
    pub scale_bits: u64,
    pub extra_bits: u32,
    pub layer: ObjectEntry10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseFrameElement10_1 {
    pub version: u16,
    pub geometry_version: u16,
    pub geometry: FrameGeometrySlot10_1,
    pub geometry_long: Option<u32>,
    pub geometry_short: Option<u16>,
    pub properties: ElementPropertiesPrefix10_1,
    pub border: DecorSlot10_1,
    pub background: DecorSlot10_1,
    pub draft_mode_bits: u16,
    pub shadow: Option<DecorSlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameGeometrySlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<GeometryBody10_1>>,
}

// 调用者持有试读游标和事务键表；本函数失败时不承诺内部游标。
pub(super) fn read_map_frame_tail(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    prefix: PageLayoutMapFramePrefix10_1,
) -> io::Result<MapFrame10_1> {
    let version = prefix.version;
    let extent_settings = if version < 4 {
        let mode_bits = r.read_u32_le()?;
        let scale_bits = r.read_f64_le()?.to_bits();
        let (envelope, envelope_body) = envelope_slot(r, limit, refs)?;
        Some(MapFrameExtent10_1 {
            mode_bits,
            scale_bits,
            envelope,
            envelope_body,
        })
    } else {
        None
    };
    let legacy_frame = if version < 2 {
        Some(MapFrameLegacy10_1 {
            geometry: geometry_slot(r, limit, refs)?,
            flags: [r.read_u16_le()?, r.read_u16_le()?],
            border: decor(r, limit, refs, DecorKind10_1::Border)?,
        })
    } else {
        None
    };
    let grid_count = collection_count(r, limit, "IMapGrid")?;
    let grids = (0..grid_count)
        .map(|_| super::map_grid::read_grid_slot_10_1(r, limit, refs))
        .collect::<io::Result<_>>()?;
    let locator_count = collection_count(r, limit, "ILocatorRectangle")?;
    let locator_rectangles = (0..locator_count)
        .map(|_| read_locator_rectangle_slot(r, limit, refs))
        .collect::<io::Result<_>>()?;
    let base = base_frame(r, limit, refs)?;
    let version_three_tail = if version == 3 {
        Some(MapFrameVersionThree10_1 {
            map_frame: reference_slot(r, refs, "IMapFrame")?,
            mode_bits: r.read_u32_le()?,
            scale_bits: r.read_f64_le()?.to_bits(),
            extra_bits: r.read_u32_le()?,
            layer: reference_slot(r, refs, "IFeatureLayer")?,
        })
    } else {
        None
    };
    Ok(MapFrame10_1 {
        prefix,
        embedded_map: None,
        extent_settings,
        legacy_frame,
        grids,
        locator_rectangles,
        base,
        version_three_tail,
    })
}

pub(super) fn base_frame(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<BaseFrameElement10_1> {
    let version = r.read_u16_le()?;
    if version > 2 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 BaseFrameElement 版本",
        ));
    }
    let RectShapeGeometry10_1 {
        geometry_version,
        geometry,
        geometry_long,
        geometry_short,
    } = read_rect_shape_10_1(r, limit, refs)?;
    let properties = match read_element_properties_prefix_10_1(r, limit, refs)? {
        PrefixOutcome10_1::Complete(properties) => properties,
        PrefixOutcome10_1::Stopped(reason) => {
            return Err(io::Error::new(ErrorKind::Unsupported, reason.describe()));
        }
    };
    let border = decor(r, limit, refs, DecorKind10_1::Border)?;
    let background = decor(r, limit, refs, DecorKind10_1::Background)?;
    let draft_mode_bits = r.read_u16_le()?;
    let shadow = (version >= 2)
        .then(|| decor(r, limit, refs, DecorKind10_1::Shadow))
        .transpose()?;
    Ok(BaseFrameElement10_1 {
        version,
        geometry_version,
        geometry,
        geometry_long,
        geometry_short,
        properties,
        border,
        background,
        draft_mode_bits,
        shadow,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct RectShapeGeometry10_1 {
    pub geometry_version: u16,
    pub geometry: FrameGeometrySlot10_1,
    pub geometry_long: Option<u32>,
    pub geometry_short: Option<u16>,
}

/// LayerLib RectShapeElement::Load 的可复用几何前段。
pub(super) fn read_rect_shape_10_1(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RectShapeGeometry10_1> {
    let geometry_version = r.read_u16_le()?;
    if geometry_version > 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 RectShapeElement 版本",
        ));
    }
    let geometry = geometry_slot(r, limit, refs)?;
    let geometry_long = (geometry_version >= 2)
        .then(|| r.read_u32_le())
        .transpose()?;
    let geometry_short = (geometry_version >= 3)
        .then(|| r.read_u16_le())
        .transpose()?;
    Ok(RectShapeGeometry10_1 {
        geometry_version,
        geometry,
        geometry_long,
        geometry_short,
    })
}

pub(super) fn envelope_slot(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Envelope10_1>)> {
    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == ENVELOPE_CLASS_ID => {
            Some(read_envelope_10_1(r, limit, refs)?)
        }
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("IEnvelope", entry)),
        _ => None,
    };
    Ok((entry, body))
}

pub(super) fn geometry_slot(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FrameGeometrySlot10_1> {
    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::NewObject { class_id, .. } => Some(Box::new(read_geometry_body_10_1(
            &class_id, r, limit, refs,
        )?)),
        _ => None,
    };
    Ok(FrameGeometrySlot10_1 { entry, body })
}

fn decor(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    kind: DecorKind10_1,
) -> io::Result<DecorSlot10_1> {
    let (slot, stop) = read_decor_slot_10_1(r, limit, refs, 16, kind)?;
    match (slot, stop) {
        (Some(slot), None) => Ok(slot),
        (_, stop) => Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("地图框装饰 {kind:?}：{stop:?}"),
        )),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocatorRectangleSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<LocatorRectangle10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocatorRectangle10_1 {
    pub version: u16,
    pub map_frame: ObjectEntry10_1,
    pub border: DecorSlot10_1,
    pub background: DecorSlot10_1,
    pub symbol: crate::objects::display_symbol::SymbolSlotFull10_1,
    pub short_1c: u16,
    pub shadow: Option<DecorSlot10_1>,
    pub short_24: Option<u16>,
}

// MapElements.dll 0x7956ab40；继承宿主事务，不递归读取新的 IMapFrame。
fn read_locator_rectangle_slot(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LocatorRectangleSlot10_1> {
    use crate::objects::display_symbol::{
        DisplaySlotKind10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1,
    };
    let entry = read_object_entry_10_1(r, refs)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: LOCATOR_RECTANGLE_CLASS_ID,
            ..
        } => {
            let version = r.read_u16_le()?;
            if version > 3 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "不支持的 LocatorRectangle 版本",
                ));
            }
            let map_frame = reference_slot(r, refs, "LocatorRectangle IMapFrame")?;
            if matches!(map_frame, ObjectEntry10_1::BackReference { class_id, .. }
                if class_id != super::page_layout_prefix::MAP_FRAME_CLASS_ID)
            {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "LocatorRectangle 的 IMapFrame 类不受支持",
                ));
            }
            let border = decor(r, limit, refs, DecorKind10_1::Border)?;
            let background = decor(r, limit, refs, DecorKind10_1::Background)?;
            let symbol = read_symbol_slot_full_10_1(r, limit, refs, 16)?;
            if let Some(stop) = slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("LocatorRectangle 符号未完整：{stop:?}"),
                ));
            }
            let short_1c = r.read_u16_le()?;
            let shadow = (version >= 2)
                .then(|| decor(r, limit, refs, DecorKind10_1::Shadow))
                .transpose()?;
            let short_24 = (version >= 3).then(|| r.read_u16_le()).transpose()?;
            Some(Box::new(LocatorRectangle10_1 {
                version,
                map_frame,
                border,
                background,
                symbol,
                short_1c,
                shadow,
                short_24,
            }))
        }
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported("ILocatorRectangle", entry)),
        _ => None,
    };
    Ok(LocatorRectangleSlot10_1 { entry, body })
}

fn collection_count(r: &mut Reader<'_>, limit: usize, name: &str) -> io::Result<usize> {
    let count = r.read_u32_le()? as i32;
    let count = usize::try_from(count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, format!("{name} 计数为负")))?;
    let min = count
        .checked_mul(16)
        .filter(|&n| n <= limit)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("{name} 计数超过限额")))?;
    if min > r.remaining() {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!("{name} 集合超出地图框范围"),
        ));
    }
    Ok(count)
}

fn reference_slot(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    name: &str,
) -> io::Result<ObjectEntry10_1> {
    let entry = read_object_entry_10_1(r, refs)?;
    if matches!(entry, ObjectEntry10_1::NewObject { .. }) {
        return Err(unsupported(name, entry));
    }
    Ok(entry)
}

pub(super) fn unsupported(name: &str, entry: ObjectEntry10_1) -> io::Error {
    io::Error::new(
        ErrorKind::Unsupported,
        format!("{name} 新对象体尚未支持：{entry:?}"),
    )
}
