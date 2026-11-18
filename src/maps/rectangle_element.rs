//! GraphicElements.dll 10.1 RectangleElement::Load（0x7505a400）。

use crate::maps::map_frame::{RectShapeGeometry10_1, read_rect_shape_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::objects::element_body::{
    ElementPropertiesPrefix10_1, PrefixOutcome10_1, read_element_properties_prefix_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const RECTANGLE_ELEMENT_CLASS_ID: [u8; 16] = [
    0xc2, 0x67, 0x97, 0x3a, 0x53, 0xf2, 0xd0, 0x11, 0x83, 0xa4, 8, 0, 9, 0xb9, 0x96, 0xcc,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RectangleElement10_1 {
    pub key: u32,
    pub version: u16,
    pub shape: RectShapeGeometry10_1,
    pub properties: ElementPropertiesPrefix10_1,
    pub fill: SymbolSlotFull10_1,
}

pub(super) fn read_rectangle_element_10_1(
    r: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RectangleElement10_1> {
    let ObjectEntry10_1::NewObject {
        class_id: RECTANGLE_ELEMENT_CLASS_ID,
        key,
    } = read_object_entry_10_1(r, refs)?
    else {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "此槽不是新 RectangleElement 对象",
        ));
    };
    let version = r.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 RectangleElement 版本",
        ));
    }
    let shape = read_rect_shape_10_1(r, limit, refs)?;
    let properties = match read_element_properties_prefix_10_1(r, limit, refs)? {
        PrefixOutcome10_1::Complete(properties) => properties,
        PrefixOutcome10_1::Stopped(reason) => {
            return Err(io::Error::new(ErrorKind::Unsupported, reason.describe()));
        }
    };
    let fill = read_symbol_slot_full_10_1(r, limit, refs, 16)?;
    if let Some(stop) = slot_full_stop_10_1(&fill, DisplaySlotKind10_1::FillSymbol) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("RectangleElement IFillSymbol 未完整：{stop:?}"),
        ));
    }
    use SymbolSlotFull10_1 as Slot;
    if !matches!(
        fill,
        Slot::Null
            | Slot::BackReference { .. }
            | Slot::SimpleFillSymbol(_)
            | Slot::MultiLayerFillSymbol(_)
            | Slot::LineFillSymbol(_)
            | Slot::MarkerFillSymbol(_)
            | Slot::GradientFillSymbol(_)
            | Slot::DotDensityFillSymbol(_)
    ) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RectangleElement 符号不属于已支持的 IFillSymbol 类",
        ));
    }
    Ok(RectangleElement10_1 {
        key,
        version,
        shape,
        properties,
        fill,
    })
}
