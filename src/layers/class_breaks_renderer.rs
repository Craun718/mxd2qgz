use super::feature_layer_tail::{
    LEGEND_GROUP_CLASS_ID, LegendGroupSlot10_1, LegendGroupStop10_1, NumberFormatSlot10_1,
    RotationRendererTail10_1, SizeRendererTail10_1, read_legend_group_body_10_1,
    read_number_format_body_for_class_10_1, read_rotation_renderer_tail_10_1,
    read_size_renderer_tail_10_1,
};
use super::legend_item::{
    BaseLegendItemStop10_1, HORIZONTAL_LEGEND_ITEM_CLASS_ID, HorizontalLegendItemBody10_1,
    read_horizontal_legend_item_body_10_1,
};
use super::legend_item_family::{
    HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID, HorizontalBarLegendItemBody10_1,
    NESTED_LEGEND_ITEM_CLASS_ID, NestedLegendItemBody10_1, VERTICAL_LEGEND_ITEM_CLASS_ID,
    VerticalLegendItemBody10_1, read_horizontal_bar_legend_item_body_10_1,
    read_nested_legend_item_body_10_1, read_vertical_legend_item_body_10_1,
};
use crate::objects::color_ramp::{ColorRampSlot10_1, read_color_ramp_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, FILL_SYMBOL_CLASSES, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1,
    SymbolStop10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub(super) const CLASS_BREAKS_RENDERER_CLASS_ID: [u8; 16] = [
    0xa2, 0x7e, 0x5f, 0xae, 0x48, 0x8b, 0xd0, 0x11, 0x83, 0x56, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

#[derive(Debug, Clone, PartialEq)]
pub enum ClassBreaksLegendItemSlot10_1 {
    Null,
    BackReference { class_id: [u8; 16], key: u32 },
    Horizontal(Box<HorizontalLegendItemBody10_1>),
    HorizontalBar(Box<HorizontalBarLegendItemBody10_1>),
    Nested(Box<NestedLegendItemBody10_1>),
    Vertical(Box<VerticalLegendItemBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassBreaksLegendItemStop10_1 {
    Horizontal(BaseLegendItemStop10_1),
    HorizontalBar(BaseLegendItemStop10_1),
    NestedBase(BaseLegendItemStop10_1),
    NestedSymbol(SymbolStop10_1),
    Vertical(BaseLegendItemStop10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRotationField10_1 {
    pub version: Option<u16>,
    pub field: String,
    pub field_type: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExclusion10_1 {
    pub version: u16,
    pub legend_group: Option<LegendGroupSlot10_1>,
    pub description: Option<String>,
    pub show_exclusion_class: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassBreaksRendererStop10_1 {
    LegendGroupUnsupported { class_id: [u8; 16], key: u32 },
    LegendGroupBody(LegendGroupStop10_1),
    LegendItemUnsupported { class_id: [u8; 16], key: u32 },
    LegendItemBody(ClassBreaksLegendItemStop10_1),
    NumberFormatUnsupported { class_id: [u8; 16], key: u32 },
    FillSymbol(SymbolStop10_1),
    DataExclusionLegendGroupUnsupported { class_id: [u8; 16], key: u32 },
    DataExclusionLegendGroupBody(LegendGroupStop10_1),
    ColorRamp(SymbolStop10_1),
    Symbol(SymbolStop10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassBreaksRenderer10_1 {
    pub key: u32,
    pub version: u16,
    pub field: String,
    pub secondary_field: String,
    pub breaks: Vec<f64>,
    pub secondary_breaks: Vec<f64>,
    pub minimum: f64,
    pub legend_group: Option<LegendGroupSlot10_1>,
    pub legend_item: Option<ClassBreaksLegendItemSlot10_1>,
    pub legacy_text_before_guid: Option<String>,
    pub optional_guid: Option<[u8; 16]>,
    pub raw_short_after_guid: Option<u16>,
    pub number_format: Option<NumberFormatSlot10_1>,
    pub legacy_number_format_text: Option<String>,
    pub label: Option<String>,
    pub legacy_short_after_label: Option<u16>,
    pub raw_shorts: Option<[u16; 2]>,
    pub raw_int32s: Option<[i32; 2]>,
    pub legacy_rotation: Option<LegacyRotationField10_1>,
    pub raw_double: Option<f64>,
    pub fill_symbol: Option<SymbolSlotFull10_1>,
    pub raw_int32_after_fill: Option<i32>,
    pub raw_double_after_fill: Option<f64>,
    pub raw_int32_version_four: Option<i32>,
    pub data_exclusion: Option<DataExclusion10_1>,
    pub rotation: Option<RotationRendererTail10_1>,
    pub size: Option<SizeRendererTail10_1>,
    pub color_ramp: Option<ColorRampSlot10_1>,
    pub symbol: Option<SymbolSlotFull10_1>,
    pub trailing_short: Option<u16>,
    pub stop: Option<ClassBreaksRendererStop10_1>,
}

const LEGACY_MINIMUM: f64 = f64::from_le_bytes([0x76, 0x3b, 0x77, 0x30, 0xd1, 0x42, 0xee, 0xff]);

fn read_count(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<usize> {
    let raw = i32::from_le_bytes(reader.read_bytes(4)?.try_into().unwrap());
    if raw < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("分级渲染器断点数量 {raw} 为负值"),
        ));
    }
    let count = raw as usize;
    let total = count
        .checked_mul(16)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "分级断点数组长度溢出"))?;
    if total > reader.remaining().min(max_bytes) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "分级断点数组超出可读范围",
        ));
    }
    Ok(count)
}

fn read_legend_group_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(
    Option<LegendGroupSlot10_1>,
    Option<ClassBreaksRendererStop10_1>,
)> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => {
            *reader = trial;
            *refs = staged;
            Ok((Some(LegendGroupSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *reader = trial;
            *refs = staged;
            Ok((
                Some(LegendGroupSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        } => {
            let body = read_legend_group_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stop = body
                .stop
                .clone()
                .map(ClassBreaksRendererStop10_1::LegendGroupBody);
            *reader = trial;
            *refs = staged;
            Ok((Some(LegendGroupSlot10_1::LegendGroup(Box::new(body))), stop))
        }
        ObjectEntry10_1::NewObject { class_id, key } => Ok((
            None,
            Some(ClassBreaksRendererStop10_1::LegendGroupUnsupported { class_id, key }),
        )),
    }
}

fn read_legend_item_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(
    Option<ClassBreaksLegendItemSlot10_1>,
    Option<ClassBreaksRendererStop10_1>,
)> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let (slot, stop) = match entry {
        ObjectEntry10_1::Null => (ClassBreaksLegendItemSlot10_1::Null, None),
        ObjectEntry10_1::BackReference { class_id, key } => (
            ClassBreaksLegendItemSlot10_1::BackReference { class_id, key },
            None,
        ),
        ObjectEntry10_1::NewObject {
            class_id: HORIZONTAL_LEGEND_ITEM_CLASS_ID,
            key,
        } => {
            let body =
                read_horizontal_legend_item_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stop = body.stop.clone().map(|stop| {
                ClassBreaksRendererStop10_1::LegendItemBody(
                    ClassBreaksLegendItemStop10_1::Horizontal(stop),
                )
            });
            (
                ClassBreaksLegendItemSlot10_1::Horizontal(Box::new(body)),
                stop,
            )
        }
        ObjectEntry10_1::NewObject {
            class_id: HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
            key,
        } => {
            let body =
                read_horizontal_bar_legend_item_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stop = body.base.stop.clone().map(|stop| {
                ClassBreaksRendererStop10_1::LegendItemBody(
                    ClassBreaksLegendItemStop10_1::HorizontalBar(stop),
                )
            });
            (
                ClassBreaksLegendItemSlot10_1::HorizontalBar(Box::new(body)),
                stop,
            )
        }
        ObjectEntry10_1::NewObject {
            class_id: NESTED_LEGEND_ITEM_CLASS_ID,
            key,
        } => {
            let body = read_nested_legend_item_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stop = body
                .base
                .stop
                .clone()
                .map(|stop| {
                    ClassBreaksRendererStop10_1::LegendItemBody(
                        ClassBreaksLegendItemStop10_1::NestedBase(stop),
                    )
                })
                .or_else(|| {
                    body.symbol_stop.clone().map(|stop| {
                        ClassBreaksRendererStop10_1::LegendItemBody(
                            ClassBreaksLegendItemStop10_1::NestedSymbol(stop),
                        )
                    })
                });
            (ClassBreaksLegendItemSlot10_1::Nested(Box::new(body)), stop)
        }
        ObjectEntry10_1::NewObject {
            class_id: VERTICAL_LEGEND_ITEM_CLASS_ID,
            key,
        } => {
            let body =
                read_vertical_legend_item_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            let stop = body.base.stop.clone().map(|stop| {
                ClassBreaksRendererStop10_1::LegendItemBody(
                    ClassBreaksLegendItemStop10_1::Vertical(stop),
                )
            });
            (
                ClassBreaksLegendItemSlot10_1::Vertical(Box::new(body)),
                stop,
            )
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Ok((
                None,
                Some(ClassBreaksRendererStop10_1::LegendItemUnsupported { class_id, key }),
            ));
        }
    };
    *reader = trial;
    *refs = staged;
    Ok((Some(slot), stop))
}

fn read_number_format_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(
    Option<NumberFormatSlot10_1>,
    Option<ClassBreaksRendererStop10_1>,
)> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => {
            *reader = trial;
            *refs = staged;
            Ok((Some(NumberFormatSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *reader = trial;
            *refs = staged;
            Ok((
                Some(NumberFormatSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            let Some(slot) = read_number_format_body_for_class_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                class_id,
                key,
            )?
            else {
                return Ok((
                    None,
                    Some(ClassBreaksRendererStop10_1::NumberFormatUnsupported { class_id, key }),
                ));
            };
            *reader = trial;
            *refs = staged;
            Ok((Some(slot), None))
        }
    }
}

fn read_fill_symbol_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(
    Option<SymbolSlotFull10_1>,
    Option<ClassBreaksRendererStop10_1>,
)> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::NewObject { class_id, key }
            if !FILL_SYMBOL_CLASSES.contains(&class_id) =>
        {
            Ok((
                None,
                Some(ClassBreaksRendererStop10_1::FillSymbol(
                    SymbolStop10_1::Slot {
                        kind: DisplaySlotKind10_1::FillSymbol,
                        class_id,
                        key,
                    },
                )),
            ))
        }
        _ => {
            let slot =
                read_symbol_slot_full_10_1(reader, max_bytes, refs, MAX_SYMBOL_NESTING_DEPTH)?;
            let stop = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::FillSymbol)
                .map(ClassBreaksRendererStop10_1::FillSymbol);
            Ok((Some(slot), stop))
        }
    }
}

fn read_data_exclusion(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(
    Option<DataExclusion10_1>,
    Option<ClassBreaksRendererStop10_1>,
)> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let version = trial.read_u16_le()?;
    if (version as i16) > 1 || (version as i16) < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("DataExclusion 版本 {version} 超出 0 至 1"),
        ));
    }
    let (legend_group, stop) = read_legend_group_slot(&mut trial, max_bytes, &mut staged)?;
    let stop = match stop {
        Some(ClassBreaksRendererStop10_1::LegendGroupUnsupported { class_id, key }) => {
            Some(ClassBreaksRendererStop10_1::DataExclusionLegendGroupUnsupported { class_id, key })
        }
        Some(ClassBreaksRendererStop10_1::LegendGroupBody(stop)) => Some(
            ClassBreaksRendererStop10_1::DataExclusionLegendGroupBody(stop),
        ),
        Some(other) => {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("DataExclusion 图例组返回意外停止原因 {other:?}"),
            ));
        }
        None => None,
    };
    let result = if stop.is_some() {
        DataExclusion10_1 {
            version,
            legend_group,
            description: None,
            show_exclusion_class: None,
        }
    } else {
        DataExclusion10_1 {
            version,
            legend_group,
            description: Some(read_string_10_1(&mut trial, max_bytes)?),
            show_exclusion_class: Some(trial.read_u16_le()?),
        }
    };
    *reader = trial;
    *refs = staged;
    Ok((Some(result), stop))
}

/// 读取 FeatureLayer.dll 10.1 ClassBreaksRenderer::Load（0x6f6a7190）的完整
/// 已确认字段序。未知嵌套实现是成功停止点；截断和非法字段整体回退。
pub fn read_class_breaks_renderer_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    key: u32,
) -> io::Result<ClassBreaksRenderer10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let version = trial.read_u16_le()?;
    if !(1..=8).contains(&version) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "分级渲染器版本超出 1 至 8",
        ));
    }
    let field = read_string_10_1(&mut trial, max_bytes)?;
    let secondary_field = read_string_10_1(&mut trial, max_bytes)?;
    let count = read_count(&mut trial, max_bytes)?;
    let mut breaks = Vec::with_capacity(count);
    let mut secondary_breaks = Vec::with_capacity(count);
    for values in [&mut breaks, &mut secondary_breaks] {
        for _ in 0..count {
            values.push(trial.read_f64_le()?);
        }
    }
    let minimum = if version >= 2 {
        trial.read_f64_le()?
    } else {
        LEGACY_MINIMUM
    };

    let (legend_group, mut stop) = read_legend_group_slot(&mut trial, max_bytes, &mut keys)?;
    let mut legend_item = None;
    let mut legacy_text_before_guid = None;
    let mut optional_guid = None;
    let mut raw_short_after_guid = None;
    let mut number_format = None;
    let mut legacy_number_format_text = None;
    let mut label = None;
    let mut legacy_short_after_label = None;
    let mut raw_shorts = None;
    let mut raw_int32s = None;
    let mut legacy_rotation = None;
    let mut raw_double = None;
    let mut fill_symbol = None;
    let mut raw_int32_after_fill = None;
    let mut raw_double_after_fill = None;
    let mut raw_int32_version_four = None;
    let mut data_exclusion = None;
    let mut rotation = None;
    let mut size = None;
    let mut color_ramp = None;
    let mut symbol = None;
    let mut trailing_short = None;

    if stop.is_none() {
        (legend_item, stop) = read_legend_item_slot(&mut trial, max_bytes, &mut keys)?;
    }
    if stop.is_none() {
        if version < 2 {
            legacy_text_before_guid = Some(read_string_10_1(&mut trial, max_bytes)?);
        } else {
            let present = trial.read_bytes(1)?[0];
            if present != 0 {
                optional_guid = Some(trial.read_bytes(16)?.try_into().expect("固定 16 字节"));
            }
        }
        raw_short_after_guid = Some(trial.read_u16_le()?);
    }
    if stop.is_none() {
        (number_format, stop) = read_number_format_slot(&mut trial, max_bytes, &mut keys)?;
    }
    if stop.is_none() {
        if version < 5 {
            legacy_number_format_text = Some(read_string_10_1(&mut trial, max_bytes)?);
        }
        label = Some(read_string_10_1(&mut trial, max_bytes)?);
        if version < 5 {
            legacy_short_after_label = Some(trial.read_u16_le()?);
        }
    }
    if stop.is_none() {
        raw_shorts = Some([trial.read_u16_le()?, trial.read_u16_le()?]);
        raw_int32s = Some([
            i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()),
            i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()),
        ]);
    }
    if stop.is_none() {
        let rotation_version = if version > 4 {
            let value = trial.read_u16_le()?;
            if (value as i16) > 1 || (value as i16) < 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("LegacyRotation 版本 {value} 超出 0 至 1"),
                ));
            }
            Some(value)
        } else {
            None
        };
        legacy_rotation = Some(LegacyRotationField10_1 {
            version: rotation_version,
            field: read_string_10_1(&mut trial, max_bytes)?,
            field_type: i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()),
        });
        raw_double = Some(trial.read_f64_le()?);
    }
    if stop.is_none() {
        (fill_symbol, stop) = read_fill_symbol_slot(&mut trial, max_bytes, &mut keys)?;
    }
    if stop.is_none() && version > 2 {
        raw_int32_after_fill = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        raw_double_after_fill = Some(trial.read_f64_le()?);
    }
    if stop.is_none() && version > 3 {
        raw_int32_version_four = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
    }
    if stop.is_none() && version >= 5 {
        (data_exclusion, stop) = read_data_exclusion(&mut trial, max_bytes, &mut keys)?;
    }
    if stop.is_none() && version >= 6 {
        rotation = Some(read_rotation_renderer_tail_10_1(&mut trial, max_bytes)?);
        size = Some(read_size_renderer_tail_10_1(&mut trial, max_bytes)?);
    }
    if stop.is_none() && version > 6 {
        let offset = trial.position();
        let (slot, ramp_stop) = read_color_ramp_slot_10_1(
            &mut trial,
            max_bytes,
            &mut keys,
            MAX_SYMBOL_NESTING_DEPTH,
            offset,
        )?;
        color_ramp = slot;
        stop = ramp_stop.map(ClassBreaksRendererStop10_1::ColorRamp);
    }
    if stop.is_none() && version > 7 {
        let slot =
            read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut keys, MAX_SYMBOL_NESTING_DEPTH)?;
        stop = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol)
            .map(ClassBreaksRendererStop10_1::Symbol);
        symbol = Some(slot);
        if stop.is_none() {
            trailing_short = Some(trial.read_u16_le()?);
        }
    }

    let result = ClassBreaksRenderer10_1 {
        key,
        version,
        field,
        secondary_field,
        breaks,
        secondary_breaks,
        minimum,
        legend_group,
        legend_item,
        legacy_text_before_guid,
        optional_guid,
        raw_short_after_guid,
        number_format,
        legacy_number_format_text,
        label,
        legacy_short_after_label,
        raw_shorts,
        raw_int32s,
        legacy_rotation,
        raw_double,
        fill_symbol,
        raw_int32_after_fill,
        raw_double_after_fill,
        raw_int32_version_four,
        data_exclusion,
        rotation,
        size,
        color_ramp,
        symbol,
        trailing_short,
        stop,
    };
    *reader = trial;
    *refs = keys;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend(value.to_le_bytes());
    }

    fn push_i32(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend(value.to_le_bytes());
    }

    fn push_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend(value.to_le_bytes());
    }

    fn push_f64(bytes: &mut Vec<u8>, value: f64) {
        bytes.extend(value.to_le_bytes());
    }

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        let mut units: Vec<u16> = value.encode_utf16().collect();
        units.push(0);
        push_u32(bytes, (units.len() * 2) as u32);
        for unit in units {
            push_u16(bytes, unit);
        }
    }

    fn push_null(bytes: &mut Vec<u8>) {
        bytes.extend([0; 16]);
    }

    fn push_entry(bytes: &mut Vec<u8>, class_id: [u8; 16], key: u32) {
        bytes.extend(class_id);
        push_u32(bytes, key);
    }

    fn push_base_legend_item_fields(bytes: &mut Vec<u8>, layer: Option<([u8; 16], u32)>) {
        push_u16(bytes, 0); // BaseLegendItem 版本；0 不写 ≥2 尾组
        if let Some((class_id, key)) = layer {
            push_entry(bytes, class_id, key);
        } else {
            push_null(bytes);
        }
        for value in [1, 2, 3, 4] {
            push_u16(bytes, value);
        }
        push_null(bytes); // ITextSymbol
        push_i32(bytes, 5);
        push_u16(bytes, 6);
        push_null(bytes); // 第二 ITextSymbol
        push_null(bytes); // ILegendClassFormat
        push_u16(bytes, 7);
        push_u16(bytes, 8);
    }

    fn legend_item_body(class_id: [u8; 16], layer: Option<([u8; 16], u32)>) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_u16(&mut bytes, 1);
        if class_id == HORIZONTAL_LEGEND_ITEM_CLASS_ID {
            push_i32(&mut bytes, 10);
            push_base_legend_item_fields(&mut bytes, layer);
        } else if class_id == HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID {
            push_base_legend_item_fields(&mut bytes, layer);
            push_i32(&mut bytes, 11);
            push_f64(&mut bytes, 12.0);
            push_f64(&mut bytes, 13.0);
        } else if class_id == NESTED_LEGEND_ITEM_CLASS_ID {
            push_base_legend_item_fields(&mut bytes, layer);
            push_i32(&mut bytes, 14);
            push_f64(&mut bytes, 15.0);
            push_null(&mut bytes);
            push_null(&mut bytes);
            for value in [16, 17, 18] {
                push_u16(&mut bytes, value);
            }
            push_i32(&mut bytes, 19);
        } else if class_id == VERTICAL_LEGEND_ITEM_CLASS_ID {
            push_i32(&mut bytes, 20);
            push_base_legend_item_fields(&mut bytes, layer);
        } else {
            panic!("test helper only builds the four supported legend item classes");
        }
        bytes
    }

    #[derive(Default)]
    struct Slots<'a> {
        legend_group: Option<&'a [u8]>,
        legend_item: Option<&'a [u8]>,
        number_format: Option<&'a [u8]>,
        fill_symbol: Option<&'a [u8]>,
        data_exclusion_group: Option<&'a [u8]>,
        color_ramp: Option<&'a [u8]>,
        symbol: Option<&'a [u8]>,
    }

    fn append_slot(bytes: &mut Vec<u8>, slot: Option<&[u8]>) {
        match slot {
            Some(slot) => bytes.extend_from_slice(slot),
            None => push_null(bytes),
        }
    }

    fn complete_body(version: u16, slots: Slots<'_>) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_u16(&mut bytes, version);
        push_string(&mut bytes, "FIELD");
        push_string(&mut bytes, "SECOND");
        push_i32(&mut bytes, 2);
        for value in [1.0, 2.0] {
            push_f64(&mut bytes, value);
        }
        for value in [3.0, 4.0] {
            push_f64(&mut bytes, value);
        }
        if version >= 2 {
            push_f64(&mut bytes, 5.5);
        }
        append_slot(&mut bytes, slots.legend_group);
        append_slot(&mut bytes, slots.legend_item);
        if version < 2 {
            push_string(&mut bytes, "legacy-before-guid");
        } else {
            bytes.push(0);
        }
        push_u16(&mut bytes, 21);
        append_slot(&mut bytes, slots.number_format);
        if version < 5 {
            push_string(&mut bytes, "legacy-number-format");
        }
        push_string(&mut bytes, "label");
        if version < 5 {
            push_u16(&mut bytes, 22);
        }
        push_u16(&mut bytes, 23);
        push_u16(&mut bytes, 24);
        push_i32(&mut bytes, 25);
        push_i32(&mut bytes, 26);
        if version > 4 {
            push_u16(&mut bytes, 1);
        }
        push_string(&mut bytes, "rotation-field");
        push_i32(&mut bytes, 27);
        push_f64(&mut bytes, 28.0);
        append_slot(&mut bytes, slots.fill_symbol);
        if version > 2 {
            push_i32(&mut bytes, 29);
            push_f64(&mut bytes, 30.0);
        }
        if version > 3 {
            push_i32(&mut bytes, 31);
        }
        if version >= 5 {
            push_u16(&mut bytes, 1);
            append_slot(&mut bytes, slots.data_exclusion_group);
            push_string(&mut bytes, "excluded");
            push_u16(&mut bytes, 32);
        }
        if version >= 6 {
            push_u16(&mut bytes, 1); // RotationRenderer 版本
            push_i32(&mut bytes, 33);
            push_i32(&mut bytes, 34);
            for value in ["rotation-a", "rotation-b", "rotation-c"] {
                push_string(&mut bytes, value);
            }
            for value in 35..=40 {
                push_f64(&mut bytes, value as f64);
            }
            push_u16(&mut bytes, 1); // SizeRenderer 版本
            push_i32(&mut bytes, 41);
            push_string(&mut bytes, "size");
            push_f64(&mut bytes, 42.0);
            push_f64(&mut bytes, 43.0);
        }
        if version > 6 {
            append_slot(&mut bytes, slots.color_ramp);
        }
        if version > 7 {
            append_slot(&mut bytes, slots.symbol);
            push_u16(&mut bytes, 44);
        }
        bytes
    }

    fn read_body(
        bytes: &[u8],
        root_key: u32,
    ) -> io::Result<(ClassBreaksRenderer10_1, Reader<'_>, RefTable10_1)> {
        let mut reader = Reader::new(bytes);
        let mut refs = RefTable10_1::new();
        refs.register(root_key);
        let body = read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, root_key)?;
        Ok((body, reader, refs))
    }

    fn entry_position(bytes: &[u8], class_id: [u8; 16], key: u32) -> usize {
        bytes
            .windows(20)
            .position(|window| window[..16] == class_id && window[16..] == key.to_le_bytes())
            .expect("test body contains the requested object entry")
    }

    #[test]
    fn reads_complete_v8_body_in_native_field_order() {
        let bytes = complete_body(8, Slots::default());
        let (body, reader, refs) = read_body(&bytes, 77).unwrap();

        assert_eq!(body.key, 77);
        assert_eq!(body.version, 8);
        assert_eq!(body.field, "FIELD");
        assert_eq!(body.secondary_field, "SECOND");
        assert_eq!(body.breaks, [1.0, 2.0]);
        assert_eq!(body.secondary_breaks, [3.0, 4.0]);
        assert_eq!(body.minimum.to_bits(), 5.5_f64.to_bits());
        assert_eq!(body.legend_group, Some(LegendGroupSlot10_1::Null));
        assert_eq!(body.legend_item, Some(ClassBreaksLegendItemSlot10_1::Null));
        assert_eq!(body.raw_short_after_guid, Some(21));
        assert_eq!(body.number_format, Some(NumberFormatSlot10_1::Null));
        assert_eq!(body.label.as_deref(), Some("label"));
        assert_eq!(body.raw_shorts, Some([23, 24]));
        assert_eq!(body.raw_int32s, Some([25, 26]));
        let rotation = body.legacy_rotation.as_ref().unwrap();
        assert_eq!(rotation.version, Some(1));
        assert_eq!(rotation.field, "rotation-field");
        assert_eq!(rotation.field_type, 27);
        assert_eq!(body.raw_double.unwrap().to_bits(), 28.0_f64.to_bits());
        assert_eq!(body.fill_symbol, Some(SymbolSlotFull10_1::Null));
        assert_eq!(body.raw_int32_after_fill, Some(29));
        assert_eq!(
            body.raw_double_after_fill.unwrap().to_bits(),
            30.0_f64.to_bits()
        );
        assert_eq!(body.raw_int32_version_four, Some(31));
        let exclusion = body.data_exclusion.as_ref().unwrap();
        assert_eq!(exclusion.version, 1);
        assert_eq!(exclusion.legend_group, Some(LegendGroupSlot10_1::Null));
        assert_eq!(exclusion.description.as_deref(), Some("excluded"));
        assert_eq!(exclusion.show_exclusion_class, Some(32));
        assert_eq!(body.rotation.as_ref().unwrap().version, 1);
        assert_eq!(body.size.as_ref().unwrap().version, 1);
        assert_eq!(body.color_ramp, Some(ColorRampSlot10_1::Null));
        assert_eq!(body.symbol, Some(SymbolSlotFull10_1::Null));
        assert_eq!(body.trailing_short, Some(44));
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(77));
    }

    #[test]
    fn v1_uses_native_legacy_minimum_bit_pattern() {
        let bytes = complete_body(1, Slots::default());
        let (body, reader, _) = read_body(&bytes, 78).unwrap();

        assert_eq!(
            LEGACY_MINIMUM.to_bits(),
            0xffee42d130773b76_u64,
            "v1 minimum must retain the native 0xffffee42d130773b76 bit pattern"
        );
        assert_eq!(body.minimum.to_bits(), LEGACY_MINIMUM.to_bits());
        assert_eq!(
            body.legacy_text_before_guid.as_deref(),
            Some("legacy-before-guid")
        );
        assert_eq!(
            body.legacy_number_format_text.as_deref(),
            Some("legacy-number-format")
        );
        assert_eq!(body.legacy_short_after_label, Some(22));
        assert_eq!(body.legacy_rotation.as_ref().unwrap().version, None);
        assert_eq!(body.raw_int32_after_fill, None);
        assert_eq!(body.data_exclusion, None);
        assert_eq!(body.rotation, None);
        assert_eq!(body.color_ramp, None);
        assert_eq!(body.symbol, None);
        assert_eq!(body.trailing_short, None);
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn accepts_versions_one_through_eight_with_expected_gates() {
        for version in 1..=8 {
            let bytes = complete_body(version, Slots::default());
            let (body, reader, _) = read_body(&bytes, version as u32 + 100).unwrap();
            assert_eq!(body.version, version);
            assert_eq!(body.stop, None, "version {version}");
            assert_eq!(reader.position(), bytes.len(), "version {version}");
            assert_eq!(body.optional_guid, None, "presence byte is zero");
            assert_eq!(body.raw_int32_after_fill.is_some(), version > 2);
            assert_eq!(
                body.raw_double_after_fill.is_some(),
                version > 2,
                "version {version}"
            );
            assert_eq!(
                body.raw_int32_version_four.is_some(),
                version > 3,
                "version {version}"
            );
            assert_eq!(
                body.data_exclusion.is_some(),
                version >= 5,
                "version {version}"
            );
            assert_eq!(body.rotation.is_some(), version >= 6, "version {version}");
            assert_eq!(body.size.is_some(), version >= 6, "version {version}");
            assert_eq!(body.color_ramp.is_some(), version > 6, "version {version}");
            assert_eq!(body.symbol.is_some(), version > 7, "version {version}");
            assert_eq!(
                body.trailing_short.is_some(),
                version > 7,
                "version {version}"
            );
        }
    }

    #[test]
    fn rejects_out_of_range_versions_and_invalid_break_counts_transactionally() {
        let mut invalid_version = complete_body(1, Slots::default());
        invalid_version[..2].copy_from_slice(&9_u16.to_le_bytes());
        let mut refs = RefTable10_1::new();
        refs.register(90);
        let before = refs.clone();
        let mut reader = Reader::new(&invalid_version);
        assert!(read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 90).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);

        for count in [-1, i32::MAX] {
            let mut prefix = Vec::new();
            push_u16(&mut prefix, 8);
            push_string(&mut prefix, "F");
            push_string(&mut prefix, "S");
            push_i32(&mut prefix, count);
            push_f64(&mut prefix, 1.0);
            push_u16(&mut prefix, 8); // 第二数组之后不可达的哨兵
            let mut refs = RefTable10_1::new();
            refs.register(91);
            let before = refs.clone();
            let mut reader = Reader::new(&prefix);
            let error =
                read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 91).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData, "count {count}");
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);
        }
    }

    #[test]
    fn every_v8_prefix_truncation_rolls_back_cursor_and_reference_table() {
        let bytes = complete_body(8, Slots::default());
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = RefTable10_1::new();
            refs.register(92);
            refs.register(93);
            let before = refs.clone();
            assert!(
                read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 92).is_err(),
                "prefix ending at {end} unexpectedly parsed"
            );
            assert_eq!(reader.position(), 0, "truncation at {end}");
            assert_eq!(refs, before, "truncation at {end}");
        }
        let (_, reader, _) = read_body(&bytes, 92).unwrap();
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn null_and_back_reference_slots_are_consumed_without_bodies() {
        let mut legend_group = Vec::new();
        push_entry(&mut legend_group, LEGEND_GROUP_CLASS_ID, 101);
        let mut legend_item = Vec::new();
        push_entry(&mut legend_item, HORIZONTAL_LEGEND_ITEM_CLASS_ID, 102);
        let mut number_format = Vec::new();
        push_entry(
            &mut number_format,
            crate::objects::number_format::NUMERIC_FORMAT_CLASS_ID,
            103,
        );
        let mut fill_symbol = Vec::new();
        push_entry(
            &mut fill_symbol,
            crate::objects::display_symbol::SIMPLE_FILL_SYMBOL_CLASS_ID,
            104,
        );
        let mut data_group = Vec::new();
        push_entry(&mut data_group, LEGEND_GROUP_CLASS_ID, 105);
        let mut color_ramp = Vec::new();
        push_entry(
            &mut color_ramp,
            crate::objects::color_ramp::RANDOM_COLOR_RAMP_CLASS_ID,
            106,
        );
        let mut symbol = Vec::new();
        push_entry(
            &mut symbol,
            crate::objects::display_symbol::SIMPLE_FILL_SYMBOL_CLASS_ID,
            107,
        );
        let bytes = complete_body(
            8,
            Slots {
                legend_group: Some(&legend_group),
                legend_item: Some(&legend_item),
                number_format: Some(&number_format),
                fill_symbol: Some(&fill_symbol),
                data_exclusion_group: Some(&data_group),
                color_ramp: Some(&color_ramp),
                symbol: Some(&symbol),
            },
        );

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        for key in [100, 101, 102, 103, 104, 105, 106, 107] {
            refs.register(key);
        }
        let before = refs.clone();
        let body = read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 100).unwrap();

        assert!(matches!(
            body.legend_group,
            Some(LegendGroupSlot10_1::BackReference { key: 101, .. })
        ));
        assert!(matches!(
            body.legend_item,
            Some(ClassBreaksLegendItemSlot10_1::BackReference { key: 102, .. })
        ));
        assert!(matches!(
            body.number_format,
            Some(NumberFormatSlot10_1::BackReference { key: 103, .. })
        ));
        assert!(matches!(
            body.fill_symbol,
            Some(SymbolSlotFull10_1::BackReference { key: 104, .. })
        ));
        assert!(matches!(
            body.data_exclusion.unwrap().legend_group,
            Some(LegendGroupSlot10_1::BackReference { key: 105, .. })
        ));
        assert!(matches!(
            body.color_ramp,
            Some(ColorRampSlot10_1::BackReference { key: 106, .. })
        ));
        assert!(matches!(
            body.symbol,
            Some(SymbolSlotFull10_1::BackReference { key: 107, .. })
        ));
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(refs, before);
    }

    #[test]
    fn unsupported_nested_classes_stop_at_each_object_entry() {
        let unsupported = [0x77; 16];
        let mut entry = Vec::new();
        push_entry(&mut entry, unsupported, 201);

        let cases = [
            (
                "legend group",
                complete_body(
                    8,
                    Slots {
                        legend_group: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
            (
                "legend item",
                complete_body(
                    8,
                    Slots {
                        legend_item: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
            (
                "number format",
                complete_body(
                    8,
                    Slots {
                        number_format: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
            (
                "fill symbol",
                complete_body(
                    8,
                    Slots {
                        fill_symbol: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
            (
                "data exclusion group",
                complete_body(
                    8,
                    Slots {
                        data_exclusion_group: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
            (
                "color ramp",
                complete_body(
                    8,
                    Slots {
                        color_ramp: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
            (
                "symbol",
                complete_body(
                    8,
                    Slots {
                        symbol: Some(&entry),
                        ..Slots::default()
                    },
                ),
            ),
        ];

        for (index, (name, bytes)) in cases.iter().enumerate() {
            let key = 201 + index as u32;
            let mut entry_bytes = Vec::new();
            push_entry(&mut entry_bytes, unsupported, key);
            let base = (bytes.len() - entry_bytes.len()) as u32;
            let _ = base;
            let mut adjusted = bytes.clone();
            let old_key = 201_u32.to_le_bytes();
            let offset = adjusted
                .windows(4)
                .position(|window| window == old_key)
                .expect("unsupported entry key");
            adjusted[offset..offset + 4].copy_from_slice(&key.to_le_bytes());

            let mut reader = Reader::new(&adjusted);
            let mut refs = RefTable10_1::new();
            refs.register(200);
            let body =
                read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 200).unwrap();
            assert!(body.stop.is_some(), "case {name}");
            assert_eq!(
                reader.position(),
                entry_position(&adjusted, unsupported, key),
                "case {name}"
            );
            assert!(!refs.contains(key), "case {name}");
            assert_eq!(refs.len(), 1, "case {name}");

            match (*name, body.stop.unwrap()) {
                (
                    "legend group",
                    ClassBreaksRendererStop10_1::LegendGroupUnsupported { key: k, .. },
                )
                | (
                    "legend item",
                    ClassBreaksRendererStop10_1::LegendItemUnsupported { key: k, .. },
                )
                | (
                    "number format",
                    ClassBreaksRendererStop10_1::NumberFormatUnsupported { key: k, .. },
                )
                | (
                    "data exclusion group",
                    ClassBreaksRendererStop10_1::DataExclusionLegendGroupUnsupported {
                        key: k, ..
                    },
                ) => assert_eq!(k, key, "case {name}"),
                (
                    "fill symbol",
                    ClassBreaksRendererStop10_1::FillSymbol(SymbolStop10_1::Slot {
                        key: k, ..
                    }),
                )
                | (
                    "color ramp",
                    ClassBreaksRendererStop10_1::ColorRamp(SymbolStop10_1::Slot { key: k, .. }),
                )
                | (
                    "symbol",
                    ClassBreaksRendererStop10_1::Symbol(SymbolStop10_1::Slot { key: k, .. }),
                ) => {
                    assert_eq!(k, key, "case {name}")
                }
                (_, other) => panic!("case {name}: unexpected stop {other:?}"),
            }
        }
    }

    #[test]
    fn dispatches_all_supported_legend_item_classes() {
        for (index, class_id) in [
            HORIZONTAL_LEGEND_ITEM_CLASS_ID,
            HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
            NESTED_LEGEND_ITEM_CLASS_ID,
            VERTICAL_LEGEND_ITEM_CLASS_ID,
        ]
        .into_iter()
        .enumerate()
        {
            let key = 301 + index as u32;
            let mut item = Vec::new();
            push_entry(&mut item, class_id, key);
            item.extend(legend_item_body(class_id, None));
            let bytes = complete_body(
                8,
                Slots {
                    legend_item: Some(&item),
                    ..Slots::default()
                },
            );
            let (body, reader, refs) = read_body(&bytes, 300).unwrap();
            assert_eq!(body.stop, None, "class {class_id:?}");
            assert_eq!(reader.position(), bytes.len(), "class {class_id:?}");
            assert!(refs.contains(key));
            match (class_id, body.legend_item.unwrap()) {
                (
                    HORIZONTAL_LEGEND_ITEM_CLASS_ID,
                    ClassBreaksLegendItemSlot10_1::Horizontal(body),
                ) => {
                    assert_eq!(body.key, key)
                }
                (
                    HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
                    ClassBreaksLegendItemSlot10_1::HorizontalBar(body),
                ) => assert_eq!(body.key, key),
                (NESTED_LEGEND_ITEM_CLASS_ID, ClassBreaksLegendItemSlot10_1::Nested(body)) => {
                    assert_eq!(body.key, key)
                }
                (VERTICAL_LEGEND_ITEM_CLASS_ID, ClassBreaksLegendItemSlot10_1::Vertical(body)) => {
                    assert_eq!(body.key, key)
                }
                (_, other) => panic!("class {class_id:?} returned {other:?}"),
            }
        }
    }

    #[test]
    fn legend_group_inner_stop_is_propagated_at_the_nested_entry() {
        let unsupported = [0x81; 16];
        let mut group = Vec::new();
        push_entry(&mut group, LEGEND_GROUP_CLASS_ID, 401);
        push_u16(&mut group, 3);
        push_u16(&mut group, 1);
        push_u16(&mut group, 2);
        push_string(&mut group, "group");
        push_i32(&mut group, 1);
        push_entry(&mut group, unsupported, 402);
        push_u16(&mut group, 3);
        let bytes = complete_body(
            8,
            Slots {
                legend_group: Some(&group),
                ..Slots::default()
            },
        );
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(400);
        let body = read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 400).unwrap();

        assert!(matches!(
            body.stop,
            Some(ClassBreaksRendererStop10_1::LegendGroupBody(
                LegendGroupStop10_1::ClassUnsupported { key: 402, .. }
            ))
        ));
        assert_eq!(reader.position(), entry_position(&bytes, unsupported, 402));
        assert!(refs.contains(401));
        assert!(!refs.contains(402));
    }

    #[test]
    fn supported_legend_item_base_stops_are_propagated_for_all_classes() {
        let unsupported = [0x91; 16];
        for (index, class_id) in [
            HORIZONTAL_LEGEND_ITEM_CLASS_ID,
            HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
            NESTED_LEGEND_ITEM_CLASS_ID,
            VERTICAL_LEGEND_ITEM_CLASS_ID,
        ]
        .into_iter()
        .enumerate()
        {
            let item_key = 501 + index as u32;
            let layer_key = 601 + index as u32;
            let mut item = Vec::new();
            push_entry(&mut item, class_id, item_key);
            item.extend(legend_item_body(class_id, Some((unsupported, layer_key))));
            let bytes = complete_body(
                8,
                Slots {
                    legend_item: Some(&item),
                    ..Slots::default()
                },
            );
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            refs.register(500);
            let body =
                read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 500).unwrap();
            let ClassBreaksRendererStop10_1::LegendItemBody(inner) = body.stop.unwrap() else {
                panic!("class {class_id:?}: expected legend-item body stop");
            };
            assert_eq!(
                reader.position(),
                entry_position(&bytes, unsupported, layer_key)
            );
            assert!(refs.contains(item_key));
            assert!(!refs.contains(layer_key));
            match (class_id, inner) {
                (
                    HORIZONTAL_LEGEND_ITEM_CLASS_ID,
                    ClassBreaksLegendItemStop10_1::Horizontal(inner),
                )
                | (
                    HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
                    ClassBreaksLegendItemStop10_1::HorizontalBar(inner),
                )
                | (NESTED_LEGEND_ITEM_CLASS_ID, ClassBreaksLegendItemStop10_1::NestedBase(inner))
                | (VERTICAL_LEGEND_ITEM_CLASS_ID, ClassBreaksLegendItemStop10_1::Vertical(inner)) =>
                {
                    assert!(matches!(
                        inner,
                        BaseLegendItemStop10_1::SlotUnsupported { key, .. } if key == layer_key
                    ))
                }
                (_, other) => panic!("class {class_id:?}: unexpected nested stop {other:?}"),
            }
        }
    }

    #[test]
    fn nested_legend_item_symbol_stop_is_propagated() {
        let unsupported = [0xA1; 16];
        let mut item = Vec::new();
        push_entry(&mut item, NESTED_LEGEND_ITEM_CLASS_ID, 701);
        push_u16(&mut item, 1);
        push_base_legend_item_fields(&mut item, None);
        push_i32(&mut item, 1);
        push_f64(&mut item, 2.0);
        push_entry(&mut item, unsupported, 702);
        push_null(&mut item);
        push_u16(&mut item, 3);
        push_u16(&mut item, 4);
        push_u16(&mut item, 5);
        push_i32(&mut item, 6);
        let bytes = complete_body(
            8,
            Slots {
                legend_item: Some(&item),
                ..Slots::default()
            },
        );
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(700);
        let body = read_class_breaks_renderer_10_1(&mut reader, 1 << 20, &mut refs, 700).unwrap();

        assert!(matches!(
            body.stop,
            Some(ClassBreaksRendererStop10_1::LegendItemBody(
                ClassBreaksLegendItemStop10_1::NestedSymbol(SymbolStop10_1::Slot { key: 702, .. })
            ))
        ));
        assert_eq!(reader.position(), entry_position(&bytes, unsupported, 702));
        assert!(!refs.contains(702));
    }
}
