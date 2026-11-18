//! FeatureLayer.dll 10.1 UniqueValueRenderer 的存盘字段，见原生 Load 0x6f72fce0。
use crate::layers::feature_layer_tail::{
    LEGEND_GROUP_CLASS_ID, LegendGroupPrefix10_1, LegendGroupSlot10_1, LegendGroupStop10_1,
    RotationRendererTail10_1, SizeRendererTail10_1, read_legend_group_body_10_1,
    read_rotation_renderer_tail_10_1, read_size_renderer_tail_10_1,
};
use crate::objects::color_ramp::{ColorRampSlot10_1, read_color_ramp_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, SymbolStop10_1,
    read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub const UNIQUE_VALUE_RENDERER_CLASS_ID: [u8; 16] = [
    0x29, 0x6d, 0x34, 0xc3, 0xbc, 0xb2, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

#[derive(Debug, Clone, PartialEq)]
pub enum UniqueValueRendererStop10_1 {
    DefaultSymbol(SymbolStop10_1),
    LegendGroup { class_id: [u8; 16], key: u32 },
    LegendGroupBody(LegendGroupStop10_1),
    LegendItem { class_id: [u8; 16], key: u32 },
    ColorRamp(SymbolStop10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniqueValue10_1 {
    pub value: String,
    pub raw_values: [u32; 2],
    pub optional_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniqueValueRendererTail10_1 {
    pub raw_short: u16,
    pub rotation_field: String,
    pub rotation_type: u32,
    pub strings: [String; 3],
    pub raw_short_after_strings: u16,
    pub default_label: Option<String>,
    pub rotation: Option<RotationRendererTail10_1>,
    pub size: Option<SizeRendererTail10_1>,
    pub color_ramp: Option<ColorRampSlot10_1>,
    pub version_five_shorts: Option<[u16; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniqueValueRenderer10_1 {
    pub key: u32,
    pub version: u16,
    /// 字段名和该项随存的三个布尔字节，保留位模式。
    pub fields: Vec<(String, [u8; 3])>,
    pub delimiter: String,
    pub default_symbol: SymbolSlotFull10_1,
    pub legend_groups: Vec<LegendGroupSlot10_1>,
    pub legend_item: Option<ObjectEntry10_1>,
    pub values: Vec<UniqueValue10_1>,
    pub tail: Option<UniqueValueRendererTail10_1>,
    pub stop: Option<UniqueValueRendererStop10_1>,
}

fn read_count(reader: &mut Reader<'_>, max_bytes: usize, item_min: usize) -> io::Result<usize> {
    let count = reader.read_u32_le()?;
    let limit = max_bytes.min(reader.remaining()) / item_min;
    if count > i32::MAX as u32 || count as usize > limit {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "唯一值渲染器计数越界",
        ));
    }
    Ok(count as usize)
}

/// 读取已登记的新对象体。错误不提交；不支持的嵌套对象保留停止位置。
pub fn read_unique_value_renderer_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    key: u32,
) -> io::Result<UniqueValueRenderer10_1> {
    use UniqueValueRendererStop10_1 as Stop;
    let mut r = reader.clone();
    let mut t = refs.clone();
    let version = r.read_u16_le()?;
    if version > 5 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的 UniqueValueRenderer 版本",
        ));
    }
    // 原生以连续三个 String 成员存储字段名，第四项将覆盖其他成员。
    let count = read_count(&mut r, max_bytes, 9)?;
    if count > 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "UniqueValueRenderer 字段数超过 3",
        ));
    }
    let mut fields = Vec::with_capacity(count);
    for _ in 0..count {
        let name = read_string_10_1(&mut r, max_bytes)?;
        fields.push((name, r.read_bytes(3)?.try_into().unwrap()));
    }
    let delimiter = read_string_10_1(&mut r, max_bytes)?;
    let default_symbol =
        read_symbol_slot_full_10_1(&mut r, max_bytes, &mut t, MAX_SYMBOL_NESTING_DEPTH)?;
    let mut result = UniqueValueRenderer10_1 {
        key,
        version,
        fields,
        delimiter,
        default_symbol,
        legend_groups: Vec::new(),
        legend_item: None,
        values: Vec::new(),
        tail: None,
        stop: None,
    };
    result.stop = 'body: {
        if let Some(stop) = slot_full_stop_10_1(&result.default_symbol, DisplaySlotKind10_1::Symbol)
        {
            break 'body Some(Stop::DefaultSymbol(stop));
        }
        let n = read_count(&mut r, max_bytes, 16)?;
        for _ in 0..n {
            let mut candidate = r.clone();
            let mut staged = t.clone();
            let slot = match read_object_entry_10_1(&mut candidate, &mut staged)? {
                ObjectEntry10_1::Null => LegendGroupSlot10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    LegendGroupSlot10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject {
                    class_id: LEGEND_GROUP_CLASS_ID,
                    key,
                } => {
                    let group: LegendGroupPrefix10_1 =
                        read_legend_group_body_10_1(&mut candidate, max_bytes, &mut staged, key)?;
                    if let Some(stop) = group.stop.clone() {
                        r = candidate;
                        t = staged;
                        result
                            .legend_groups
                            .push(LegendGroupSlot10_1::LegendGroup(Box::new(group)));
                        break 'body Some(Stop::LegendGroupBody(stop));
                    }
                    LegendGroupSlot10_1::LegendGroup(Box::new(group))
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    break 'body Some(Stop::LegendGroup { class_id, key });
                }
            };
            r = candidate;
            t = staged;
            result.legend_groups.push(slot);
        }
        let mut candidate = r.clone();
        let mut staged = t.clone();
        let item = read_object_entry_10_1(&mut candidate, &mut staged)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = item {
            break 'body Some(Stop::LegendItem { class_id, key });
        }
        r = candidate;
        t = staged;
        result.legend_item = Some(item);
        let n = read_count(&mut r, max_bytes, 18)?;
        for _ in 0..n {
            let value = read_string_10_1(&mut r, max_bytes)?;
            let raw_values = [r.read_u32_le()?, r.read_u32_le()?];
            let optional_string = (r.read_u32_le()? != 0)
                .then(|| read_string_10_1(&mut r, max_bytes))
                .transpose()?;
            result.values.push(UniqueValue10_1 {
                value,
                raw_values,
                optional_string,
            });
        }
        let raw_short = r.read_u16_le()?;
        let rotation_field = read_string_10_1(&mut r, max_bytes)?;
        let rotation_type = r.read_u32_le()?;
        let strings = [
            read_string_10_1(&mut r, max_bytes)?,
            read_string_10_1(&mut r, max_bytes)?,
            read_string_10_1(&mut r, max_bytes)?,
        ];
        let raw_short_after_strings = r.read_u16_le()?;
        let default_label = (version != 1)
            .then(|| read_string_10_1(&mut r, max_bytes))
            .transpose()?;
        let mut tail = UniqueValueRendererTail10_1 {
            raw_short,
            rotation_field,
            rotation_type,
            strings,
            raw_short_after_strings,
            default_label,
            rotation: None,
            size: None,
            color_ramp: None,
            version_five_shorts: None,
        };
        if version >= 3 {
            tail.rotation = Some(read_rotation_renderer_tail_10_1(&mut r, max_bytes)?);
            tail.size = Some(read_size_renderer_tail_10_1(&mut r, max_bytes)?);
        }
        if version >= 4 {
            let offset = r.position();
            let (ramp, stop) = read_color_ramp_slot_10_1(
                &mut r,
                max_bytes,
                &mut t,
                MAX_SYMBOL_NESTING_DEPTH,
                offset,
            )?;
            tail.color_ramp = ramp;
            if let Some(stop) = stop {
                result.tail = Some(tail);
                break 'body Some(Stop::ColorRamp(stop));
            }
        }
        if version >= 5 {
            tail.version_five_shorts = Some([r.read_u16_le()?, r.read_u16_le()?]);
        }
        result.tail = Some(tail);
        None
    };
    *reader = r;
    *refs = t;
    Ok(result)
}
