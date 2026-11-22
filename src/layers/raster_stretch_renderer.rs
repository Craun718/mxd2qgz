//! RasterRenderer.dll 10.1 的 RasterStretchRenderer。

use super::feature_layer_tail::{
    LEGEND_GROUP_CLASS_ID, LegendGroupSlot10_1, read_inline_legend_group_body_10_1,
};
use super::raster_renderer::{
    RasterHistogramSlot10_1, RasterRendererShared10_1, next_is_tag, optional_long_record,
    optional_record, raster_io_object_class, raster_io_record, read_raster_histogram,
    read_raster_io_end, read_renderer_color_ramp, read_renderer_legend_group, read_shared_tail,
};
use super::raster_unique_value::read_renderer_legend_item;
use crate::maps::map_surround::LegendItemSlot10_1;
use crate::objects::color_ramp::{
    ColorRampSlot10_1, nested_ramp_stop, read_color_ramp_body_for_class_10_1,
};
use crate::objects::display_color::{
    ColorSlot10_1, read_color_body_for_class_10_1, read_color_slot_10_1,
};
use crate::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH;
use crate::storage::{class_id_hex, object_refs::RefTable10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub const RASTER_STRETCH_RENDERER_CLASS_ID: [u8; 16] = [
    0xb2, 0xa3, 0x01, 0xa3, 0xd7, 0x74, 0xd2, 0x11, 0x9f, 0x29, 0x00, 0xc0, 0x4f, 0x8e, 0xd1, 0xd7,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterStretchRenderer10_1 {
    pub version: i32,
    pub band: i32,
    pub label: String,
    pub lookup: Vec<u8>,
    pub stretch_type: i32,
    pub double_a: f64,
    pub long_a: i32,
    pub double_b: f64,
    pub layout: RasterStretchLayout10_1,
    pub shared: RasterRendererShared10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterStretchLayout10_1 {
    Legacy {
        extra_long: i32,
        ramp: ColorRampSlot10_1,
        colors: [ColorSlot10_1; 2],
        legend_group: LegendGroupSlot10_1,
        legend_item: LegendItemSlot10_1,
        trailing_long: Option<i32>,
    },
    RasterIo(Box<RasterStretchIo10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterStretchIo10_1 {
    /// CLSID + 已对位的无键对象体，显式空对象为 16 字节零；记录不存在时为 None。
    pub ramp: Option<Vec<u8>>,
    pub color: Option<Vec<u8>>,
    /// tag 9 必须有非空图例组；仍为无键对象体。
    pub legend_group: Vec<u8>,
    pub tag10: Option<i32>,
    pub tag11: Option<i32>,
    pub tag12: Option<u16>,
    pub tag13: Option<[f64; 2]>,
    pub tag14: Option<i32>,
    pub tag15: Option<i32>,
    pub tag16: Option<f64>,
    pub histogram: Option<RasterHistogramSlot10_1>,
    pub extra: Option<RasterStretchExtra10_1>,
    pub pair_v6: Option<[f64; 2]>,
    pub long_v7: Option<i32>,
    pub pair_v9: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterStretchExtra10_1 {
    pub tag18: Option<i32>,
    pub tag19: Option<f64>,
    pub tag20: Option<i32>,
    pub tag22: Option<i32>,
    pub values: Vec<(f64, Option<String>)>,
}

fn long(r: &mut Reader<'_>) -> io::Result<i32> {
    Ok(r.read_u32_le()? as i32)
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, msg.into())
}

fn read_color(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<ColorSlot10_1> {
    match read_color_slot_10_1(r, max, table)? {
        ColorSlot10_1::Unsupported { class_id, .. } => Err(invalid(format!(
            "栅格拉伸渲染器颜色类未支持：{}",
            class_id_hex(&class_id)
        ))),
        body => Ok(body),
    }
}

fn read_no_key_object(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    tag: i32,
) -> io::Result<Option<Vec<u8>>> {
    if !next_is_tag(r, tag)? {
        return Ok(None);
    }
    let mut start = r.clone();
    let class = raster_io_object_class(r, tag)?;
    match (tag, class) {
        (_, id) if id == [0; 16] => {}
        (7, class_id) => {
            // 根对象无键，数字 0 只传给已知体读取器；结果保留流内字节，
            // 不登记虚拟键。体内 LoadObject 仍在同一底流共享键表。
            let ramp = read_color_ramp_body_for_class_10_1(
                r,
                max,
                table,
                MAX_SYMBOL_NESTING_DEPTH,
                class_id,
                0,
            )?
            .ok_or_else(|| {
                invalid(format!(
                    "栅格拉伸渲染器梯度类未支持：{}",
                    class_id_hex(&class_id)
                ))
            })?;
            if let Some(stop) = nested_ramp_stop(&ramp) {
                return Err(invalid(format!("栅格拉伸渲染器梯度未闭合：{stop:?}")));
            }
        }
        (8, class_id) => {
            read_color_body_for_class_10_1(class_id, r, max, table, 0)?.ok_or_else(|| {
                invalid(format!(
                    "栅格拉伸渲染器 tag 8 的类 {} 未支持",
                    class_id_hex(&class_id)
                ))
            })?;
        }
        (9, LEGEND_GROUP_CLASS_ID) => {
            let group = read_inline_legend_group_body_10_1(r, max, table, 0)?;
            if let Some(stop) = group.stop {
                return Err(invalid(format!("栅格拉伸渲染器图例组未闭合：{stop:?}")));
            }
        }
        _ => {
            return Err(invalid(format!(
                "栅格拉伸渲染器 tag {tag} 的类 {} 未支持",
                class_id_hex(&class)
            )));
        }
    }
    start.read_bytes(8)?;
    Ok(Some(
        start.read_bytes(r.position() - start.position())?.to_vec(),
    ))
}

fn double_record(r: &mut Reader<'_>, tag: i32) -> io::Result<Option<f64>> {
    optional_record(r, tag, 8)?
        .map(|mut field| field.read_f64_le())
        .transpose()
}

fn pair(r: &mut Reader<'_>) -> io::Result<[f64; 2]> {
    Ok([r.read_f64_le()?, r.read_f64_le()?])
}

fn tagged_long(r: &mut Reader<'_>, tag: i32) -> io::Result<i32> {
    long(&mut Reader::new(raster_io_record(r, tag, 4)?))
}

fn read_extra(r: &mut Reader<'_>, max: usize) -> io::Result<RasterStretchExtra10_1> {
    let tag18 = optional_long_record(r, 18)?;
    let tag19 = double_record(r, 19)?;
    let tag20 = optional_long_record(r, 20)?;
    let count = usize::try_from(tagged_long(r, 21)?).map_err(|_| invalid("栅格拉伸样本数为负"))?;
    let tag22 = optional_long_record(r, 22)?;
    let named = tag22.is_some_and(|flag| flag != 0);
    let min_bytes = if named { 14 } else { 8 };
    if count > max / min_bytes || count > r.remaining() / min_bytes {
        return Err(invalid("栅格拉伸样本数量超出限额或剩余长度"));
    }
    let values = (0..count)
        .map(|_| -> io::Result<_> {
            let value = r.read_f64_le()?;
            let label = named.then(|| read_string_10_1(r, max)).transpose()?;
            Ok((value, label))
        })
        .collect::<io::Result<_>>()?;
    Ok(RasterStretchExtra10_1 {
        tag18,
        tag19,
        tag20,
        tag22,
        values,
    })
}

pub(super) fn read_raster_stretch_renderer(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterStretchRenderer10_1> {
    let version = long(r)?;
    if version < 1 {
        return Err(invalid(format!(
            "未支持的 RasterStretchRenderer 版本 {version}"
        )));
    }
    let (band, label, lookup, stretch_type, double_a, long_a, double_b, layout) = if version < 3 {
        // 旧式体 0x7c97fde0 无 RasterIO，所有色彩及图例槽均使用宿主键表。
        let band = long(r)?;
        let label = read_string_10_1(r, max)?;
        let lookup = r.read_bytes(0x930)?.to_vec();
        let stretch_type = long(r)?;
        let double_a = r.read_f64_le()?;
        let long_a = long(r)?;
        let extra_long = long(r)?;
        let double_b = r.read_f64_le()?;
        let ramp = read_renderer_color_ramp(r, max, table)?;
        let colors = [read_color(r, max, table)?, read_color(r, max, table)?];
        let legend_group = read_renderer_legend_group(r, max, table)?;
        let legend_item = read_renderer_legend_item(r, max, table)?;
        let trailing_long = (version >= 2).then(|| long(r)).transpose()?;
        (
            band,
            label,
            lookup,
            stretch_type,
            double_a,
            long_a,
            double_b,
            RasterStretchLayout10_1::Legacy {
                extra_long,
                ramp,
                colors,
                legend_group,
                legend_item,
                trailing_long,
            },
        )
    } else {
        let label = read_string_10_1(r, max)?;
        raster_io_record(r, 0x00ff_00ff, 0)?;
        let band = tagged_long(r, 1)?;
        let lookup = raster_io_record(r, 2, 0x930)?.to_vec();
        let stretch_type = tagged_long(r, 3)?;
        let double_a = Reader::new(raster_io_record(r, 4, 8)?).read_f64_le()?;
        let long_a = tagged_long(r, 5)?;
        let double_b = Reader::new(raster_io_record(r, 6, 8)?).read_f64_le()?;
        let ramp = read_no_key_object(r, max, table, 7)?;
        let color = read_no_key_object(r, max, table, 8)?;
        // 原生在 0x7c97f2d9 对缺省或空 tag 9 直接走失败出口。
        let legend_group = read_no_key_object(r, max, table, 9)?
            .filter(|bytes| bytes[..16] != [0; 16])
            .ok_or_else(|| invalid("栅格拉伸渲染器缺少非空 tag 9 图例组"))?;
        let tag10 = optional_long_record(r, 10)?;
        let tag11 = optional_long_record(r, 11)?;
        let tag12 = optional_record(r, 12, 2)?
            .map(|mut p| p.read_u16_le())
            .transpose()?;
        let tag13 = optional_record(r, 13, 16)?
            .map(|mut p| pair(&mut p))
            .transpose()?;
        let tag14 = optional_long_record(r, 14)?;
        let tag15 = optional_long_record(r, 15)?;
        let tag16 = double_record(r, 16)?;
        let histogram = read_raster_histogram(r, 17)?;
        let extra = (version >= 5).then(|| read_extra(r, max)).transpose()?;
        let pair_v6 = (version >= 6).then(|| pair(r)).transpose()?;
        let long_v7 = (version >= 7).then(|| long(r)).transpose()?;
        let pair_v9 = (version >= 9).then(|| pair(r)).transpose()?;
        read_raster_io_end(r)?;
        (
            band,
            label,
            lookup,
            stretch_type,
            double_a,
            long_a,
            double_b,
            RasterStretchLayout10_1::RasterIo(Box::new(RasterStretchIo10_1 {
                ramp,
                color,
                legend_group,
                tag10,
                tag11,
                tag12,
                tag13,
                tag14,
                tag15,
                tag16,
                histogram,
                extra,
                pair_v6,
                long_v7,
                pair_v9,
            })),
        )
    };
    let shared = read_shared_tail(r, max, table)?;
    Ok(RasterStretchRenderer10_1 {
        version,
        band,
        label,
        lookup,
        stretch_type,
        double_a,
        long_a,
        double_b,
        layout,
        shared,
    })
}
