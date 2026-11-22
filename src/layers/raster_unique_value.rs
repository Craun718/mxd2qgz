//! RasterRenderer.dll 10.1 的栅格唯一值渲染器及 RasterDB 的唯一值集合。

use super::feature_layer_tail::LegendGroupSlot10_1;
use super::legend_item::{HORIZONTAL_LEGEND_ITEM_CLASS_ID, read_horizontal_legend_item_body_10_1};
use super::legend_item_family::{
    HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID, NESTED_LEGEND_ITEM_CLASS_ID,
    VERTICAL_LEGEND_ITEM_CLASS_ID, read_horizontal_bar_legend_item_body_10_1,
    read_nested_legend_item_body_10_1, read_vertical_legend_item_body_10_1,
};
use super::raster_renderer::{
    RasterRendererShared10_1, read_renderer_color_ramp, read_renderer_legend_group,
    read_shared_tail,
};
use crate::maps::map_surround::LegendItemSlot10_1;
use crate::objects::color_ramp::ColorRampSlot10_1;
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID: [u8; 16] = [
    0x95, 0xb5, 0x42, 0x08, 0x2f, 0x4f, 0xd2, 0x11, 0x9f, 0x43, 0x00, 0xc0, 0x4f, 0x8e, 0xce, 0x3d,
];
pub const UNIQUE_VALUES_CLASS_ID: [u8; 16] = [
    0xc7, 0xf1, 0x81, 0x9c, 0x2b, 0x79, 0x7e, 0x46, 0xac, 0x80, 0x06, 0x9e, 0x4f, 0xdf, 0x4d, 0xef,
];
pub const RASTER_UNIQUE_VALUE_COLORS_CLASS_ID: [u8; 16] = [
    0xbe, 0x98, 0x04, 0xd1, 0x16, 0x66, 0xd2, 0x11, 0xaa, 0xbc, 0x00, 0xc0, 0x4f, 0xa3, 0x34, 0x16,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterUniqueValueRenderer10_1 {
    pub version: i32,
    /// 表头字符串按原生顺序保留，旧式体两条、现代体三条。
    pub strings: Vec<String>,
    pub first_short: u16,
    pub symbol: SymbolSlotFull10_1,
    pub label: String,
    pub legend_groups: Vec<LegendGroupSlot10_1>,
    pub classes: Vec<RasterUniqueClass10_1>,
    pub second_short: u16,
    pub layout: RasterUniqueValueLayout10_1,
    pub shared: RasterRendererShared10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterUniqueClass10_1 {
    /// 保留流内原值；v1 在原生加载时会交换高低半字。
    pub raw_key: u32,
    pub values: Vec<PropertyValueObjectStream10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterUniqueValueLayout10_1 {
    Legacy {
        before_groups: i32,
        legend_item: LegendItemSlot10_1,
        colors: RasterUniqueValueColorsSlot10_1,
        samples: Vec<(f64, i32)>,
    },
    Modern {
        unique_values: UniqueValuesSlot10_1,
        ramp: Option<ColorRampSlot10_1>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterUniqueValueColorsSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<RasterUniqueValueColors10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterUniqueValueColors10_1 {
    pub version: i32,
    /// 非零时全部条目用字符串键。
    pub string_flag: i32,
    pub entries: Vec<RasterUniqueValueColorEntry10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterUniqueValueColorEntry10_1 {
    pub value: RasterUniqueValueColorKey10_1,
    /// 原生按 4 字节读写，不重排位模式。
    pub colors: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterUniqueValueColorKey10_1 {
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniqueValuesSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<UniqueValues10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniqueValues10_1 {
    pub version: i32,
    pub declared_count: i32,
    pub values: Vec<(PropertyValueObjectStream10_1, i32)>,
}

fn invalid(label: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, label.into())
}

fn long(r: &mut Reader<'_>) -> io::Result<i32> {
    Ok(r.read_u32_le()? as i32)
}

fn length(r: &mut Reader<'_>, budget: usize, width: usize) -> io::Result<usize> {
    let n = usize::try_from(long(r)?).map_err(|_| invalid("栅格唯一值计数为负"))?;
    check_length(r, n, budget, width)
}

fn check_length(r: &Reader<'_>, n: usize, budget: usize, width: usize) -> io::Result<usize> {
    if n > budget / width {
        return Err(invalid("栅格唯一值计数超过资源限额"));
    }
    if n > r.remaining() / width {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "栅格唯一值计数超出可读范围",
        ));
    }
    Ok(n)
}

pub(super) fn read_raster_unique_value(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterUniqueValueRenderer10_1> {
    let version = long(r)?;
    if version < 1 {
        return Err(invalid(format!("未支持的栅格唯一值渲染器版本 {version}")));
    }
    let legacy = version < 3;
    let strings = (0..if legacy { 2 } else { 3 })
        .map(|_| read_string_10_1(r, max))
        .collect::<io::Result<_>>()?;
    let first_short = r.read_u16_le()?;
    let symbol = read_symbol_slot_full_10_1(r, max, table, MAX_SYMBOL_NESTING_DEPTH)?;
    if let Some(stop) = slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol) {
        return Err(invalid(format!("栅格唯一值渲染器符号未闭合：{stop:?}")));
    }
    let label = read_string_10_1(r, max)?;
    let before_groups = legacy.then(|| long(r)).transpose()?;
    let legend_count = long(r)?.max(0) as usize;
    let legend_groups = (0..check_length(r, legend_count, max, 16)?)
        .map(|_| read_renderer_legend_group(r, max, table))
        .collect::<io::Result<_>>()?;
    let legend_item = legacy
        .then(|| read_renderer_legend_item(r, max, table))
        .transpose()?;
    let mut classes = Vec::new();
    for _ in 0..length(r, max, 8)? {
        let raw_key = r.read_u32_le()?;
        let values = (0..length(r, max, 16)?)
            .map(|_| read_padded_variant_object_stream_10_1(r, max, table))
            .collect::<io::Result<_>>()?;
        classes.push(RasterUniqueClass10_1 { raw_key, values });
    }
    let second_short = r.read_u16_le()?;
    let layout = if let (Some(before_groups), Some(legend_item)) = (before_groups, legend_item) {
        let colors = read_legacy_colors(r, max, table)?;
        let samples = (0..length(r, max, 12)?)
            .map(|_| Ok((r.read_f64_le()?, long(r)?)))
            .collect::<io::Result<_>>()?;
        RasterUniqueValueLayout10_1::Legacy {
            before_groups,
            legend_item,
            colors,
            samples,
        }
    } else {
        let unique_values = read_unique_values(r, max, table)?;
        let ramp = (version >= 4)
            .then(|| read_renderer_color_ramp(r, max, table))
            .transpose()?;
        RasterUniqueValueLayout10_1::Modern {
            unique_values,
            ramp,
        }
    };
    let shared = read_shared_tail(r, max, table)?;
    Ok(RasterUniqueValueRenderer10_1 {
        version,
        strings,
        first_short,
        symbol,
        label,
        legend_groups,
        classes,
        second_short,
        layout,
        shared,
    })
}

fn read_legacy_colors(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterUniqueValueColorsSlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id: RASTER_UNIQUE_VALUE_COLORS_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject {
            class_id: RASTER_UNIQUE_VALUE_COLORS_CLASS_ID,
            ..
        } => {
            let mut r = reader.clone().read_region(reader.remaining().min(max))?;
            let body = read_legacy_colors_body(&mut r, max)?;
            reader.read_bytes(r.position())?;
            Some(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, .. }
        | ObjectEntry10_1::BackReference { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "未支持的 IEnumRasterUniqueValueColor 类 {}",
                    crate::storage::class_id_hex(&class_id)
                ),
            ));
        }
    };
    Ok(RasterUniqueValueColorsSlot10_1 { entry, body })
}

pub(crate) fn read_legacy_colors_body(
    r: &mut Reader<'_>,
    max: usize,
) -> io::Result<RasterUniqueValueColors10_1> {
    let version = long(r)?;
    if version != 1 {
        return Err(invalid(format!(
            "未支持的 RasterUniqueValueColors 版本 {version}"
        )));
    }
    // RasterRendererUI.dll 先读计数和模式，再按同一模式读取各条目。
    let count = length(r, max, 4)?;
    let string_flag = long(r)?;
    let string_keys = string_flag != 0;
    let entry_min = if string_keys { 10 } else { 12 };
    if count > r.remaining() / entry_min {
        return Err(invalid("RasterUniqueValueColors 条目计数超出可读范围"));
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        let value = if string_keys {
            RasterUniqueValueColorKey10_1::Text(read_string_10_1(r, max)?)
        } else {
            RasterUniqueValueColorKey10_1::Number(r.read_f64_le()?)
        };
        // 原生读取器用有符号比较进入颜色循环，负数和零均为空列表。
        let color_count = long(r)?.max(0) as usize;
        let color_count = check_length(r, color_count, max, 4)?;
        let colors = (0..color_count)
            .map(|_| r.read_u32_le())
            .collect::<io::Result<_>>()?;
        entries.push(RasterUniqueValueColorEntry10_1 { value, colors });
    }
    Ok(RasterUniqueValueColors10_1 {
        version,
        string_flag,
        entries,
    })
}

pub(super) fn read_unique_values(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<UniqueValuesSlot10_1> {
    let mut entry = read_object_entry_10_1(r, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id: UNIQUE_VALUES_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject {
            class_id: UNIQUE_VALUES_CLASS_ID,
            ..
        } => Some(read_unique_values_body(r, max, table, 0)?),
        ObjectEntry10_1::NewObject { class_id, .. }
        | ObjectEntry10_1::BackReference { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "未支持的 IUniqueValues 类 {}",
                    crate::storage::class_id_hex(&class_id),
                ),
            ));
        }
    };
    Ok(UniqueValuesSlot10_1 { entry, body })
}

pub(super) fn read_renderer_legend_item(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<LegendItemSlot10_1> {
    macro_rules! confirmed {
        ($read:ident, $variant:ident, $key:expr, $($stop:ident).+) => {{
            let body = $read(r, max, table, $key)?;
            if let Some(stop) = &body.$($stop).+ {
                return Err(invalid(format!("栅格渲染器 ILegendItem 体未闭合：{stop:?}")));
            }
            LegendItemSlot10_1::$variant(Box::new(body))
        }};
    }
    Ok(match read_object_entry_10_1(r, table)? {
        ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = table.registered_class(key);
            if registered.is_some_and(|id| {
                !matches!(
                    id,
                    HORIZONTAL_LEGEND_ITEM_CLASS_ID
                        | HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID
                        | NESTED_LEGEND_ITEM_CLASS_ID
                        | VERTICAL_LEGEND_ITEM_CLASS_ID
                )
            }) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("ILegendItem 回引键 {key} 的登记类不受支持"),
                ));
            }
            LegendItemSlot10_1::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: HORIZONTAL_LEGEND_ITEM_CLASS_ID,
            key,
        } => confirmed!(
            read_horizontal_legend_item_body_10_1,
            HorizontalLegendItem,
            key,
            stop
        ),
        ObjectEntry10_1::NewObject {
            class_id: HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
            key,
        } => confirmed!(
            read_horizontal_bar_legend_item_body_10_1,
            HorizontalBarLegendItem,
            key,
            base.stop
        ),
        ObjectEntry10_1::NewObject {
            class_id: NESTED_LEGEND_ITEM_CLASS_ID,
            key,
        } => {
            let body = read_nested_legend_item_body_10_1(r, max, table, key)?;
            if let Some(stop) = body.symbol_stop {
                return Err(invalid(format!("栅格渲染器图例项符号未闭合：{stop:?}")));
            }
            if let Some(stop) = &body.base.stop {
                return Err(invalid(format!(
                    "栅格渲染器 ILegendItem 体未闭合：{stop:?}"
                )));
            }
            LegendItemSlot10_1::NestedLegendItem(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: VERTICAL_LEGEND_ITEM_CLASS_ID,
            key,
        } => confirmed!(
            read_vertical_legend_item_body_10_1,
            VerticalLegendItem,
            key,
            base.stop
        ),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "栅格渲染器未支持的 ILegendItem 类 {}",
                    crate::storage::class_id_hex(&class_id)
                ),
            ));
        }
    })
}

pub(crate) fn read_unique_values_body(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<UniqueValues10_1> {
    use crate::objects::property_set::read_padded_variant_object_stream_nested_10_1;

    let version = long(r)?;
    let declared_count = long(r)?;
    let count = check_length(r, declared_count.max(0) as usize, max, 20)?;
    let values = (0..count)
        .map(|_| {
            Ok((
                read_padded_variant_object_stream_nested_10_1(r, max, table, depth)?,
                long(r)?,
            ))
        })
        .collect::<io::Result<_>>()?;
    Ok(UniqueValues10_1 {
        version,
        declared_count,
        values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legend_item_reference_checks_registered_class() {
        use crate::objects::display_color::RGB_COLOR_CLASS_ID;

        for registered in [HORIZONTAL_LEGEND_ITEM_CLASS_ID, RGB_COLOR_CLASS_ID] {
            let mut previous = registered.to_vec();
            previous.extend(72_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
            let before = refs.clone();

            let mut bytes = VERTICAL_LEGEND_ITEM_CLASS_ID.to_vec();
            bytes.extend(72_u32.to_le_bytes());
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let result = read_renderer_legend_item(&mut reader, 128, &mut refs);
            if registered == HORIZONTAL_LEGEND_ITEM_CLASS_ID {
                assert_eq!(
                    result.unwrap(),
                    LegendItemSlot10_1::BackReference {
                        class_id: registered,
                        key: 72,
                    }
                );
                assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
            }
            assert_eq!(refs, before);
        }
    }

    #[test]
    fn legacy_colors_slot_checks_the_registered_class() {
        for (registered, claimed) in [
            (RASTER_UNIQUE_VALUE_COLORS_CLASS_ID, UNIQUE_VALUES_CLASS_ID),
            (UNIQUE_VALUES_CLASS_ID, RASTER_UNIQUE_VALUE_COLORS_CLASS_ID),
        ] {
            let mut bytes = registered.to_vec();
            bytes.extend(12_u32.to_le_bytes());
            let mut table = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&bytes), &mut table).unwrap();
            let before = table.clone();

            bytes[..16].copy_from_slice(&claimed);
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let result = read_legacy_colors(&mut reader, 128, &mut table);
            if registered == RASTER_UNIQUE_VALUE_COLORS_CLASS_ID {
                let slot = result.unwrap();
                assert_eq!(
                    slot.entry,
                    ObjectEntry10_1::BackReference {
                        class_id: registered,
                        key: 12
                    }
                );
                assert!(slot.body.is_none());
                assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
            }
            assert_eq!(table, before);
        }
    }

    #[test]
    fn legacy_color_counts_follow_signed_inner_loop() {
        for empty_count in [0_i32, -1, i32::MIN] {
            let mut bytes = Vec::new();
            for field in [1_i32, 2, 0] {
                bytes.extend(field.to_le_bytes());
            }
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(empty_count.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(7_u32.to_le_bytes());
            let end = bytes.len();
            bytes.push(0xaa);

            let mut reader = Reader::new(&bytes);
            let body = read_legacy_colors_body(&mut reader, 128).unwrap();
            assert!(body.entries[0].colors.is_empty());
            assert_eq!(body.entries[1].colors, [7]);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn legacy_color_variant_reads_numeric_key_and_rolls_back_on_short_color() {
        use crate::objects::property_set::PropertyValueObjectStream10_1;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        bytes.extend(RASTER_UNIQUE_VALUE_COLORS_CLASS_ID);
        bytes.extend(12_u32.to_le_bytes());
        for field in [1_i32, 1, 0] {
            bytes.extend(field.to_le_bytes());
        }
        bytes.extend(2.5_f64.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(0x11223344_u32.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::RasterUniqueValueColors(value) = value else {
            panic!("缺少唯一值颜色枚举");
        };
        assert_eq!(value.key, 12);
        assert_eq!(
            value.body().entries[0].value,
            RasterUniqueValueColorKey10_1::Number(2.5)
        );
        assert_eq!(value.body().entries[0].colors, [0x11223344]);
        assert_eq!(reader.position(), end);
        assert!(refs.contains(12));

        for invalid in [i32::MIN, 0, 2] {
            let mut input = bytes.clone();
            input[36..40].copy_from_slice(&invalid.to_le_bytes());
            let mut reader = Reader::new(&input);
            let mut keys = RefTable10_1::new();
            keys.register(42);
            let before = keys.clone();
            assert_eq!(
                read_padded_variant_object_stream_10_1(&mut reader, 128, &mut keys,)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData,
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(keys, before);
        }

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn unique_values_slot_checks_the_registered_class() {
        let other = [0x7d; 16];
        for (registered, claimed) in [
            (UNIQUE_VALUES_CLASS_ID, other),
            (other, UNIQUE_VALUES_CLASS_ID),
        ] {
            let mut bytes = registered.to_vec();
            bytes.extend(12_u32.to_le_bytes());
            let mut table = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&bytes), &mut table).unwrap();
            let before = table.clone();

            bytes[..16].copy_from_slice(&claimed);
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let result = read_unique_values(&mut reader, 128, &mut table);
            if registered == UNIQUE_VALUES_CLASS_ID {
                let slot = result.unwrap();
                assert_eq!(
                    slot.entry,
                    ObjectEntry10_1::BackReference {
                        class_id: UNIQUE_VALUES_CLASS_ID,
                        key: 12,
                    }
                );
                assert!(slot.body.is_none());
                assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
            }
            assert_eq!(table, before);
        }
    }
}
