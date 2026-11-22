//! RasterRenderer.dll 10.1 的 RasterClassifyRenderer 持久化对象体。

use super::feature_layer_tail::{
    LEGEND_CLASS_CLASS_ID, LegendClassPrefix10_1, LegendGroupSlot10_1, NumberFormatSlot10_1,
    read_legend_class_body_10_1, read_number_format_body_for_class_10_1,
};
use super::raster_renderer::{
    RasterRendererShared10_1, next_is_tag, optional_long_record, optional_record, raster_io_record,
    read_raster_io_end, read_renderer_legend_group, read_shared_tail,
};
use super::raster_unique_value::{
    UniqueValuesSlot10_1, read_renderer_legend_item, read_unique_values,
};
use crate::maps::map_surround::LegendItemSlot10_1;
use crate::objects::number_format::{
    ANGLE_FORMAT_CLASS_ID, CURRENCY_FORMAT_CLASS_ID, CUSTOM_NUMBER_FORMAT_CLASS_ID,
    FRACTION_FORMAT_CLASS_ID, LATLON_FORMAT_CLASS_ID, NUMERIC_FORMAT_CLASS_ID,
    PERCENTAGE_FORMAT_CLASS_ID, RATE_FORMAT_CLASS_ID, SCIENTIFIC_FORMAT_CLASS_ID,
};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const RASTER_CLASSIFY_RENDERER_CLASS_ID: [u8; 16] = [
    0x44, 0x2f, 0x8b, 0xce, 0x27, 0xa0, 0xd2, 0x11, 0xaa, 0xe7, 0x00, 0xc0, 0x4f, 0xa3, 0x34, 0x16,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterClassifyRenderer10_1 {
    pub version: i32,
    pub legacy_long: Option<i32>,
    pub strings: [String; 3],
    pub classification_method: ClassificationMethod10_1,
    pub legend_groups: Vec<LegendGroupSlot10_1>,
    pub legend_item: LegendItemSlot10_1,
    /// v1 独立整数数组；保留原值，不和后继 double 断点混合。
    pub legacy_values: Option<Vec<i32>>,
    pub breaks: Vec<f64>,
    pub numeric_fields: Option<ClassificationNumericFields10_1>,
    pub value_bounds: Option<ClassificationBounds10_1>,
    pub unique_values: Option<UniqueValuesSlot10_1>,
    pub records: Option<ClassificationIo10_1>,
    pub trailing_short: Option<u16>,
    pub shared: RasterRendererShared10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassificationMethod10_1 {
    LegacyString(String),
    /// byte 为 0 时没有 GUID；非零时保留 16 字节 Windows 内存序。
    Guid {
        flag: u8,
        value: Option<[u8; 16]>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationNumericFields10_1 {
    pub short_a: u16,
    pub format: NumberFormatSlot10_1,
    pub short_b: u16,
    pub double: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationBounds10_1 {
    pub values: [PropertyValueObjectStream10_1; 2],
    pub legend_selector: i16,
    /// 选择子为 −1 时原生从图例组取对象，无独立持久化槽。
    pub legend_class: Option<ClassificationLegendSlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationLegendSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<LegendClassPrefix10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationIo10_1 {
    pub tag1: Option<i32>,
    pub tag2: Option<f64>,
}

fn long(r: &mut Reader<'_>) -> io::Result<i32> {
    Ok(r.read_u32_le()? as i32)
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, msg.into())
}

fn length(r: &mut Reader<'_>, max: usize, width: usize) -> io::Result<usize> {
    let n = usize::try_from(long(r)?).map_err(|_| invalid("栅格分类渲染器计数为负"))?;
    check_length(r, n, max, width)
}

fn check_length(r: &Reader<'_>, n: usize, max: usize, width: usize) -> io::Result<usize> {
    if n > max / width {
        return Err(invalid("栅格分类渲染器计数超过资源限额"));
    }
    if n > r.remaining() / width {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "栅格分类渲染器计数超出可读范围",
        ));
    }
    Ok(n)
}

fn read_format(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<NumberFormatSlot10_1> {
    Ok(match read_object_entry_10_1(r, table)? {
        ObjectEntry10_1::Null => NumberFormatSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = table.registered_class(key);
            if registered.is_some_and(|id| {
                !matches!(
                    id,
                    ANGLE_FORMAT_CLASS_ID
                        | CURRENCY_FORMAT_CLASS_ID
                        | CUSTOM_NUMBER_FORMAT_CLASS_ID
                        | FRACTION_FORMAT_CLASS_ID
                        | LATLON_FORMAT_CLASS_ID
                        | NUMERIC_FORMAT_CLASS_ID
                        | PERCENTAGE_FORMAT_CLASS_ID
                        | RATE_FORMAT_CLASS_ID
                        | SCIENTIFIC_FORMAT_CLASS_ID
                )
            }) {
                return Err(invalid(format!(
                    "INumberFormat 回引键 {key} 的登记类不受支持"
                )));
            }
            NumberFormatSlot10_1::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            }
        }
        ObjectEntry10_1::NewObject { class_id, key } => read_number_format_body_for_class_10_1(
            r, max, table, class_id, key,
        )?
        .ok_or_else(|| {
            invalid(format!(
                "栅格分类渲染器未支持的 INumberFormat 类 {}",
                crate::storage::class_id_hex(&class_id)
            ))
        })?,
    })
}

fn read_legend_class(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<ClassificationLegendSlot10_1> {
    let mut entry = read_object_entry_10_1(r, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if registered != LEGEND_CLASS_CLASS_ID {
            return Err(invalid(format!(
                "ILegendClass 回引键 {key} 的登记类不受支持"
            )));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: LEGEND_CLASS_CLASS_ID,
            key,
        } => {
            let body = read_legend_class_body_10_1(r, max, table, key)?;
            if let Some(stop) = body.stop {
                return Err(invalid(format!(
                    "栅格分类渲染器 LegendClass 体未闭合：{stop:?}"
                )));
            }
            Some(body)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(invalid(format!(
                "栅格分类渲染器未支持的 ILegendClass 类 {}",
                crate::storage::class_id_hex(&class_id),
            )));
        }
    };
    Ok(ClassificationLegendSlot10_1 { entry, body })
}

pub(super) fn read_raster_classify_renderer(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterClassifyRenderer10_1> {
    let version = long(r)?;
    if version < 1 {
        return Err(invalid(format!(
            "未支持的 RasterClassifyRenderer 版本 {version}"
        )));
    }
    let legacy_long = (version == 1).then(|| long(r)).transpose()?;
    let first = read_string_10_1(r, max)?;
    let second = read_string_10_1(r, max)?;
    let classification_method = if version == 1 {
        ClassificationMethod10_1::LegacyString(read_string_10_1(r, max)?)
    } else {
        let flag = r.read_bytes(1)?[0];
        let value = (flag != 0)
            .then(|| r.read_bytes(16).map(|b| b.try_into().unwrap()))
            .transpose()?;
        ClassificationMethod10_1::Guid { flag, value }
    };
    let third = read_string_10_1(r, max)?;
    let legend_count = long(r)?.max(0) as usize;
    let legend_groups = (0..check_length(r, legend_count, max, 16)?)
        .map(|_| read_renderer_legend_group(r, max, table))
        .collect::<io::Result<_>>()?;
    let legend_item = read_renderer_legend_item(r, max, table)?;
    let legacy_values = (version == 1)
        .then(|| {
            (0..length(r, max, 4)?)
                .map(|_| long(r))
                .collect::<io::Result<_>>()
        })
        .transpose()?;
    let breaks = (0..length(r, max, 8)?)
        .map(|_| r.read_f64_le())
        .collect::<io::Result<_>>()?;
    let numeric_fields = (version >= 2)
        .then(|| -> io::Result<_> {
            Ok(ClassificationNumericFields10_1 {
                short_a: r.read_u16_le()?,
                format: read_format(r, max, table)?,
                short_b: r.read_u16_le()?,
                double: r.read_f64_le()?,
            })
        })
        .transpose()?;
    let value_bounds = (version >= 3)
        .then(|| -> io::Result<_> {
            let values = [
                read_padded_variant_object_stream_10_1(r, max, table)?,
                read_padded_variant_object_stream_10_1(r, max, table)?,
            ];
            let legend_selector = r.read_u16_le()? as i16;
            // Load 0x7c971eea 仅在 −1 时走图例数组，不消费独立对象槽。
            let legend_class = (legend_selector != -1)
                .then(|| read_legend_class(r, max, table))
                .transpose()?;
            Ok(ClassificationBounds10_1 {
                values,
                legend_selector,
                legend_class,
            })
        })
        .transpose()?;
    let unique_values = (version >= 4)
        .then(|| read_unique_values(r, max, table))
        .transpose()?;
    // RasterIO::InitCheck 在无 magic 时恢复位置；此时直接进入后续 short 或共用体。
    let records = if next_is_tag(r, 0x00ff_00ff)? {
        raster_io_record(r, 0x00ff_00ff, 0)?;
        let tag1 = optional_long_record(r, 1)?;
        let tag2 = optional_record(r, 2, 8)?
            .map(|mut p| p.read_f64_le())
            .transpose()?;
        read_raster_io_end(r)?;
        Some(ClassificationIo10_1 { tag1, tag2 })
    } else {
        None
    };
    let trailing_short = (version >= 5).then(|| r.read_u16_le()).transpose()?;
    let shared = read_shared_tail(r, max, table)?;
    Ok(RasterClassifyRenderer10_1 {
        version,
        legacy_long,
        strings: [first, second, third],
        classification_method,
        legend_groups,
        legend_item,
        legacy_values,
        breaks,
        numeric_fields,
        value_bounds,
        unique_values,
        records,
        trailing_short,
        shared,
    })
}
