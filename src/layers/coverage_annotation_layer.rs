//! FeatureLayer.dll 10.1 的 CoverageAnnotationLayer 持久化体。

use std::io::{self, ErrorKind};

use super::feature_layer::{
    FeatureLayerPrefix10_1, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
};
use super::feature_layer_tail::{
    DisplayFilterSlot10_1, FIELD_INFO_CLASS_ID, FieldInfoBody10_1, FieldInfoEntry10_1, RefSlot10_1,
    SuccessorStop10_1, TRANSPARENCY_DISPLAY_FILTER_CLASS_ID, read_field_info_body_10_1,
    read_transparency_display_filter_10_1,
};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_tabular_name_slot_10_1};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::ole_font::{STDFONT_CLASS_ID, StdFontBody10_1, read_std_font_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};

pub const COVERAGE_ANNOTATION_LAYER_CLASS_ID: [u8; 16] = [
    0xc9, 0xa4, 0x22, 0x0c, 0xfd, 0xda, 0xd2, 0x11, 0x9f, 0x46, 0, 0xc0, 0x4f, 0x6b, 0xc7, 0x8e,
];

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageAnnotationLegacy10_1 {
    pub name: String,
    pub shorts: [u16; 3],
    pub dataset_name: DatasetNameMoreSlot10_1,
    pub scales: [f64; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageAnnotationLayer10_1 {
    pub version: u16,
    pub legacy: Option<CoverageAnnotationLegacy10_1>,
    pub feature: Option<Box<FeatureLayerPrefix10_1>>,
    pub fields: Vec<FieldInfoEntry10_1>,
    pub longs: [u32; 4],
    pub integer_lists: [Vec<u32>; 2],
    pub fonts: Vec<(ObjectEntry10_1, Option<StdFontBody10_1>)>,
    pub colors: Vec<ColorSlot10_1>,
    pub text_symbols: Vec<SymbolSlotFull10_1>,
    pub short_list: Vec<u16>,
    pub line_symbol: SymbolSlotFull10_1,
    pub legacy_display: Option<([String; 2], u16)>,
    pub legacy_filter: Option<(u32, DisplayFilterSlot10_1)>,
    pub legacy_expression: Option<(String, u16)>,
    pub annotation_short: Option<u16>,
    pub extra_line: Option<SymbolSlotFull10_1>,
    pub extra_marker: Option<SymbolSlotFull10_1>,
}

impl CoverageAnnotationLayer10_1 {
    pub fn name(&self) -> &str {
        self.feature
            .as_ref()
            .map(|feature| feature.name.as_str())
            .or_else(|| self.legacy.as_ref().map(|legacy| legacy.name.as_str()))
            .unwrap_or_default()
    }
}

fn count(r: &mut Reader<'_>, max: usize, item_bytes: usize) -> io::Result<usize> {
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    if count > max / item_bytes {
        return Err(io::Error::new(ErrorKind::InvalidData, "注记层列表超过限额"));
    }
    if count > r.remaining() / item_bytes {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "注记层列表超出剩余字节",
        ));
    }
    Ok(count)
}

fn unsupported(field: &str) -> io::Error {
    io::Error::new(
        ErrorKind::Unsupported,
        format!("CoverageAnnotationLayer 的 {field} 槽未读完"),
    )
}

fn symbol(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    kind: DisplaySlotKind10_1,
) -> io::Result<SymbolSlotFull10_1> {
    let slot = read_symbol_slot_full_10_1(r, max, refs, MAX_SYMBOL_NESTING_DEPTH)?;
    if slot_full_stop_10_1(&slot, kind).is_some() {
        return Err(unsupported("符号"));
    }
    Ok(slot)
}

fn field(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FieldInfoEntry10_1> {
    Ok(match read_object_entry_10_1(r, refs)? {
        ObjectEntry10_1::Null => FieldInfoEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            FieldInfoEntry10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject {
            class_id: FIELD_INFO_CLASS_ID,
            key,
        } => {
            let body: FieldInfoBody10_1 = read_field_info_body_10_1(r, max, refs, key)?;
            if body.stop.is_some() {
                return Err(unsupported("IFieldInfo"));
            }
            FieldInfoEntry10_1::FieldInfo(Box::new(body))
        }
        _ => return Err(unsupported("IFieldInfo")),
    })
}

pub(crate) fn read_coverage_annotation_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<CoverageAnnotationLayer10_1> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()?;
    if version > 5 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("CoverageAnnotationLayer 不支持版本 {version}"),
        ));
    }
    let (legacy, feature) = if version == 5 {
        // 构造在 +0x70 保存 FeatureLayer 的 IPersistStream，直接调用 Load。
        let feature = read_feature_layer_prefix_10_1(&mut r, max, &mut refs)?;
        if feature.stop != FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted) {
            return Err(unsupported("内联 FeatureLayer"));
        }
        (None, Some(Box::new(feature)))
    } else {
        let name = read_string_10_1(&mut r, max)?;
        let shorts = [r.read_u16_le()?, r.read_u16_le()?, r.read_u16_le()?];
        let mut stop = None;
        let dataset_name = read_tabular_name_slot_10_1(&mut r, max, &mut refs, &mut stop)?;
        if stop.is_some() {
            return Err(unsupported("IName"));
        }
        let scales = [r.read_f64_le()?, r.read_f64_le()?];
        (
            Some(CoverageAnnotationLegacy10_1 {
                name,
                shorts,
                dataset_name,
                scales,
            }),
            None,
        )
    };

    // 0x6f6bf2e0 从两种前缀后读取同一组集合和标量。
    let fields = (0..count(&mut r, max, 16)?)
        .map(|_| field(&mut r, max, &mut refs))
        .collect::<io::Result<_>>()?;
    let first_long = r.read_u32_le()?;
    let first_ids = (0..count(&mut r, max, 4)?)
        .map(|_| r.read_u32_le())
        .collect::<io::Result<_>>()?;
    let mut fonts = Vec::new();
    for _ in 0..count(&mut r, max, 16)? {
        let entry = read_object_entry_10_1(&mut r, &mut refs)?;
        let body = match entry {
            ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
            ObjectEntry10_1::NewObject {
                class_id: STDFONT_CLASS_ID,
                ..
            } => Some(read_std_font_body_10_1(&mut r, max)?),
            _ => return Err(unsupported("OLE 字体")),
        };
        fonts.push((entry, body));
    }
    let mut colors = Vec::new();
    for _ in 0..count(&mut r, max, 16)? {
        let slot = read_color_slot_10_1(&mut r, max, &mut refs)?;
        if matches!(slot, ColorSlot10_1::Unsupported { .. }) {
            return Err(unsupported("IColor"));
        }
        colors.push(slot);
    }
    let text_symbols = (0..count(&mut r, max, 16)?)
        .map(|_| symbol(&mut r, max, &mut refs, DisplaySlotKind10_1::Symbol))
        .collect::<io::Result<_>>()?;
    let second_long = r.read_u32_le()?;
    let second_ids = (0..count(&mut r, max, 4)?)
        .map(|_| r.read_u32_le())
        .collect::<io::Result<_>>()?;
    let short_list = (0..count(&mut r, max, 2)?)
        .map(|_| r.read_u16_le())
        .collect::<io::Result<_>>()?;
    let longs = [first_long, second_long, r.read_u32_le()?, r.read_u32_le()?];
    let line_symbol = symbol(&mut r, max, &mut refs, DisplaySlotKind10_1::LineSymbol)?;

    let mut body = CoverageAnnotationLayer10_1 {
        version,
        legacy,
        feature,
        fields,
        longs,
        integer_lists: [first_ids, second_ids],
        fonts,
        colors,
        text_symbols,
        short_list,
        line_symbol,
        legacy_display: None,
        legacy_filter: None,
        legacy_expression: None,
        annotation_short: None,
        extra_line: None,
        extra_marker: None,
    };
    if version == 5 {
        body.annotation_short = Some(r.read_u16_le()?);
        body.extra_line = Some(symbol(
            &mut r,
            max,
            &mut refs,
            DisplaySlotKind10_1::LineSymbol,
        )?);
        body.extra_marker = Some(symbol(
            &mut r,
            max,
            &mut refs,
            DisplaySlotKind10_1::MarkerSymbol,
        )?);
    } else {
        if version >= 2 {
            body.legacy_display = Some((
                [
                    read_string_10_1(&mut r, max)?,
                    read_string_10_1(&mut r, max)?,
                ],
                r.read_u16_le()?,
            ));
        }
        if version >= 3 {
            let value = r.read_u32_le()?;
            let filter = match read_object_entry_10_1(&mut r, &mut refs)? {
                ObjectEntry10_1::Null => DisplayFilterSlot10_1::Reference(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    DisplayFilterSlot10_1::Reference(RefSlot10_1::BackReference { class_id, key })
                }
                ObjectEntry10_1::NewObject {
                    class_id: TRANSPARENCY_DISPLAY_FILTER_CLASS_ID,
                    key,
                } => read_transparency_display_filter_10_1(&mut r, key)?,
                _ => return Err(unsupported("IDisplayFilter")),
            };
            body.legacy_filter = Some((value, filter));
        }
        if version == 4 {
            body.legacy_expression = Some((read_string_10_1(&mut r, max)?, r.read_u16_le()?));
        }
    }
    *reader = r;
    *table = refs;
    Ok(body)
}
