//! RasterRenderer.dll 10.1 的已确认栅格渲染器。
//! 原生入口、读写边界及尚未闭合的字段见 docs/mxd-10.1-raster-renderer.md。

use crate::layers::feature_layer_tail::{
    LEGEND_GROUP_CLASS_ID, LegendGroupSlot10_1, read_legend_group_body_10_1,
};
use crate::layers::raster_classify_renderer::{
    RASTER_CLASSIFY_RENDERER_CLASS_ID, RasterClassifyRenderer10_1, read_raster_classify_renderer,
};
pub use crate::layers::raster_dataset_name::RasterDatasetNameEntryBody10_1 as LoadedRasterName10_1;
use crate::layers::raster_dataset_name::{RasterNameSlot10_1, read_name_slot};
use crate::layers::raster_layer_tail::{RasterLayerEnvelopeSlot10_1, read_envelope_slot};
use crate::layers::raster_stretch_renderer::{
    RASTER_STRETCH_RENDERER_CLASS_ID, RasterStretchRenderer10_1, read_raster_stretch_renderer,
};
use crate::layers::raster_unique_value::{
    RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID, RasterUniqueValueRenderer10_1, read_raster_unique_value,
};
use crate::objects::color_ramp::{ColorRampSlot10_1, read_color_ramp_slot_10_1};
use crate::objects::display_color::{
    ColorSlot10_1, RGB_COLOR_CLASS_ID, read_color_slot_10_1, read_rgb_color_body_10_1,
};
use crate::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH;
use crate::objects::geodata_xform::{
    GeodataXformSlot10_1, MAX_XFORM_NESTING, read_geodata_xform_slot_with_depth,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

const NULL_CLASS_ID_10_1: [u8; 16] = [0; 16];
pub const RASTER_COLORMAP_RENDERER_CLASS_ID: [u8; 16] = [
    0xfa, 0x83, 0x82, 0x3b, 0xd4, 0x1b, 0x12, 0x42, 0xb3, 0x85, 0x19, 0xe7, 0x7b, 0x9e, 0x0d, 0x3b,
];
pub const RASTER_RGB_RENDERER_CLASS_ID: [u8; 16] = [
    0x70, 0x18, 0x7f, 0x57, 0x37, 0x70, 0xd2, 0x11, 0x9f, 0x29, 0x00, 0xc0, 0x4f, 0x8e, 0xd1, 0xd7,
];
pub const SIMPLE_RASTER_RENDERER_CLASS_ID: [u8; 16] = [
    0xef, 0xd0, 0x32, 0x32, 0x60, 0x34, 0x79, 0x48, 0xa1, 0xe6, 0x36, 0x65, 0x90, 0xcf, 0x3e, 0xec,
];
pub const SAI_RENDERER_CLASS_ID: [u8; 16] = [
    0x50, 0x00, 0xab, 0x11, 0xf5, 0x74, 0x3e, 0x4c, 0x9e, 0xf9, 0x87, 0x3b, 0x6b, 0xd8, 0x5b, 0x2c,
];
pub const DISCRETE_COLOR_RENDERER_CLASS_ID: [u8; 16] = [
    0x73, 0x45, 0x87, 0xac, 0x78, 0xd7, 0x21, 0x44, 0xb9, 0xc6, 0x14, 0x55, 0x7d, 0x8b, 0xd6, 0x92,
];
pub const PANSHARPENING_FILTER_CLASS_ID: [u8; 16] = [
    0x71, 0x12, 0x8b, 0xb5, 0xfb, 0xe6, 0x27, 0x4b, 0x84, 0x17, 0x2b, 0x8d, 0x62, 0xd6, 0x54, 0xb0,
];
pub(crate) const STATS_HISTOGRAM_CLASS_ID: [u8; 16] = [
    0xc1, 0x27, 0x30, 0xba, 0xca, 0x49, 0x88, 0x47, 0x8b, 0x5a, 0x3a, 0x6b, 0x38, 0x7d, 0xe7, 0x8c,
];
pub(crate) const DOUBLE_ARRAY_CLASS_ID: [u8; 16] = [
    0xa7, 0x6c, 0xc0, 0x60, 0x9e, 0xe0, 0xd2, 0x11, 0x9f, 0x7b, 0x00, 0xc0, 0x4f, 0x8e, 0xce, 0x27,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterRendererSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterRendererPrefix10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterRendererPrefix10_1 {
    Colormap {
        version: i32,
        legend_groups: Vec<LegendGroupSlot10_1>,
        /// 保留记录中的整数原值；单独使用不足以还原 ArcMap 的色表。
        palette_values: Vec<i32>,
        /// 仅在共享体与色表类专属尾字段均完整消费后填入。
        tail: Option<Box<ColormapTail10_1>>,
        stop: Option<String>,
    },
    Simple {
        version: i32,
        /// v2 起在共用体之前读取的原始 short。
        short: Option<u16>,
        shared: RasterRendererShared10_1,
    },
    Sai {
        version: i32,
        short: u16,
        /// 在共用体之前顺序读取的两个原始 double。
        doubles: [f64; 2],
        shared: RasterRendererShared10_1,
    },
    Discrete(Box<DiscreteColorRenderer10_1>),
    UniqueValue(Box<RasterUniqueValueRenderer10_1>),
    Classify(Box<RasterClassifyRenderer10_1>),
    Stretch(Box<RasterStretchRenderer10_1>),
    LegacyRgb(Box<LegacyRgbRenderer10_1>),
    Rgb {
        version: i32,
        /// RGB 使用的波段索引，保留原生零基顺序。
        bands: [i32; 3],
        /// RasterIO tag 4 到 11；保留未定名字段和查找表的原始字节。
        records: Vec<Vec<u8>>,
        tail: Option<Box<RgbTail10_1>>,
        stop: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteColorRenderer10_1 {
    pub version: i32,
    /// 原生以 unsigned long 批量读写，不重排、不截断高位。
    pub colors: Vec<u32>,
    pub text: String,
    /// 类自身的梯度，在共用体之前；共用体 v9 还有独立的梯度槽。
    pub ramp: ColorRampSlot10_1,
    pub shared: RasterRendererShared10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRgbRenderer10_1 {
    pub version: i32,
    pub bands: [i32; 3],
    /// 旧式查找表及后继定长字段，按流内次序存放。
    pub lookup_table: Vec<u8>,
    pub byte_value: u8,
    pub double_value: f64,
    pub longs: [i32; 3],
    /// 原生把第一组四个 double 读到局部变量，仍保留原始双组值。
    pub double_groups: [[f64; 4]; 2],
    pub colors: [ColorSlot10_1; 2],
    pub shared: RasterRendererShared10_1,
    /// 版本 2 和 3 分别增加一个 int32；首项由原生布尔化。
    pub trailing_flag: Option<i32>,
    pub trailing_long: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColormapTail10_1 {
    pub shared: RasterRendererShared10_1,
    /// 颜色表版本 2 增加第一个值，版本 3 增加后两个值。
    pub colormap_values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RgbTail10_1 {
    /// tag 12 的无键颜色对象。
    pub color: Vec<u8>,
    pub tag13: Option<i32>,
    /// tag 14、15、16。显式空对象与省略记录分别保留。
    pub histograms: [Option<RasterHistogramSlot10_1>; 3],
    pub tag17: Option<i32>,
    /// tag 18 可省略、显式为空或内嵌无键锐化过滤器。
    pub pansharpening: Option<PansharpeningSlot10_1>,
    pub tag19: Option<i32>,
    pub tag20: Option<[f64; 3]>,
    pub tag21: Option<f64>,
    pub tag22: Option<f64>,
    pub trailing_short: Option<u16>,
    pub double_arrays: Vec<RasterDoubleArraySlot10_1>,
    pub shared: RasterRendererShared10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PansharpeningSlot10_1 {
    Null,
    Filter(Box<PansharpeningFilter10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PansharpeningFilter10_1 {
    pub version: i32,
    /// 两个栅格按持久化次序保留，空标记无后继字段。
    pub rasters: [Option<LoadedRaster10_1>; 2],
    pub method: i32,
    pub weights: [f64; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedRaster10_1 {
    pub name_entry: ObjectEntry10_1,
    /// 回引仍包含波段列表和后两槽，只省略数据集名称体。
    pub name_body: Option<LoadedRasterName10_1>,
    /// 原生读取有符号数量，非正值不进入波段名称循环。
    pub declared_band_count: i32,
    pub band_names: Vec<String>,
    pub transform: GeodataXformSlot10_1,
    pub extent: RasterLayerEnvelopeSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterHistogramSlot10_1 {
    Null,
    Histogram(Box<RasterHistogram10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterHistogram10_1 {
    pub version: i32,
    pub statistics: [f64; 6],
    pub bins: Vec<f64>,
    pub double_a: Option<f64>,
    pub double_b: Option<f64>,
    pub additional_bins: Option<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterDoubleArraySlot10_1 {
    pub entry: ObjectEntry10_1,
    /// 仅新对象含版本与值；空对象与回引不占正文。
    pub body: Option<(u16, Vec<f64>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterRendererShared10_1 {
    pub version: i32,
    pub layout: RasterRendererSharedLayout10_1,
    /// 共享版本 4、5、6、7、8 依次增加的原始字段。
    pub shared_short_a: Option<u16>,
    pub shared_long_a: Option<i32>,
    pub shared_long_b: Option<i32>,
    pub shared_byte: Option<u8>,
    pub shared_short_b: Option<u16>,
    pub color_ramp: Option<ColorRampSlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterRendererSharedLayout10_1 {
    /// v1 先读四个无标签的 int32，无颜色及 double 记录。
    Legacy([i32; 4]),
    RasterIo {
        /// tag 1–5；tag 2/3 可以省略。
        integers: [Option<i32>; 5],
        /// tag 6 的 CLSID + 无键对象体；空对象为 16 字节零。
        color: Vec<u8>,
        doubles: [f64; 3],
    },
}

impl RasterRendererPrefix10_1 {
    /// 完整体才可以在无长度字段的父对象中继续读取。
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            Self::Colormap { tail: Some(_), .. }
                | Self::Simple { .. }
                | Self::Sai { .. }
                | Self::Discrete(_)
                | Self::UniqueValue(_)
                | Self::Classify(_)
                | Self::Stretch(_)
                | Self::LegacyRgb(_)
                | Self::Rgb { tail: Some(_), .. }
        )
    }
}

fn invalid(reason: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, reason.into())
}

fn signed_long(reader: &mut Reader<'_>) -> io::Result<i32> {
    Ok(i32::from_le_bytes(
        reader.read_bytes(4)?.try_into().unwrap(),
    ))
}

/// 数量先按当前有界区可容纳的条目数校验，再逐项读取。
fn count(reader: &mut Reader<'_>, min_size: usize) -> io::Result<usize> {
    let declared = signed_long(reader)?;
    bounded_count(reader, declared, min_size)
}

fn bounded_count(reader: &Reader<'_>, declared: i32, min_size: usize) -> io::Result<usize> {
    let count = declared.max(0) as usize;
    if count > reader.remaining() / min_size {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "栅格渲染器条目计数超出可读范围",
        ));
    }
    Ok(count)
}

pub(super) fn raster_io_record<'a>(
    reader: &mut Reader<'a>,
    tag: i32,
    len: usize,
) -> io::Result<&'a [u8]> {
    let actual_tag = signed_long(reader)?;
    let actual_len = signed_long(reader)?;
    if actual_tag != tag || usize::try_from(actual_len).ok() != Some(len) {
        return Err(invalid(format!(
            "栅格渲染器 RasterIO 预期 tag={tag} length={len}，实际 tag={actual_tag} length={actual_len}"
        )));
    }
    reader.read_bytes(len)
}

/// 读取对象槽；新对象可能只消费到前缀末端，完整性见 `is_complete`。
/// 错误回退游标和引用表。
pub fn read_raster_renderer_slot_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterRendererSlot10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let mut entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = staged.registered_class(*key)
    {
        if !matches!(
            registered,
            RASTER_COLORMAP_RENDERER_CLASS_ID
                | RASTER_RGB_RENDERER_CLASS_ID
                | SIMPLE_RASTER_RENDERER_CLASS_ID
                | SAI_RENDERER_CLASS_ID
                | DISCRETE_COLOR_RENDERER_CLASS_ID
                | RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID
                | RASTER_CLASSIFY_RENDERER_CLASS_ID
                | RASTER_STRETCH_RENDERER_CLASS_ID
        ) {
            return Err(invalid(format!(
                "IRasterRenderer 回引键 {key} 的登记类不受支持"
            )));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::NewObject { class_id, .. } => Some(read_raster_renderer_body_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            class_id,
        )?),
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
    };
    *reader = trial;
    *table = staged;
    Ok(RasterRendererSlot10_1 { entry, body })
}

/// 调用者已消费外层对象头并登记新键；共享引用表不用于 RasterIO 的无键对象。
pub(crate) fn read_raster_renderer_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<RasterRendererPrefix10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let body = match class_id {
        RASTER_COLORMAP_RENDERER_CLASS_ID => {
            let version = signed_long(&mut trial)?;
            if !(1..=3).contains(&version) {
                return Err(invalid(format!("未支持的色表渲染器版本 {version}")));
            }
            let legend_groups = (0..count(&mut trial, 16)?)
                .map(|_| read_renderer_legend_group(&mut trial, max_bytes, &mut staged))
                .collect::<io::Result<_>>()?;
            let palette_values = (0..count(&mut trial, 4)?)
                .map(|_| signed_long(&mut trial))
                .collect::<io::Result<_>>()?;
            let mut tail_trial = trial.clone();
            let mut tail_table = staged.clone();
            let (tail, stop) =
                match read_colormap_tail(&mut tail_trial, max_bytes, &mut tail_table, version) {
                    Ok(tail) => {
                        trial = tail_trial;
                        staged = tail_table;
                        (Some(Box::new(tail)), None)
                    }
                    Err(error) => (None, Some(error.to_string())),
                };
            RasterRendererPrefix10_1::Colormap {
                version,
                legend_groups,
                palette_values,
                tail,
                stop,
            }
        }
        RASTER_RGB_RENDERER_CLASS_ID => {
            let version = signed_long(&mut trial)?;
            if (1..=3).contains(&version) {
                let legacy = read_legacy_rgb(&mut trial, max_bytes, &mut staged, version)?;
                *reader = trial;
                *table = staged;
                return Ok(RasterRendererPrefix10_1::LegacyRgb(Box::new(legacy)));
            }
            if !(4..=9).contains(&version) {
                return Err(invalid(format!("未支持的 RGB 渲染器版本 {version}")));
            }
            raster_io_record(&mut trial, 0x00ff_00ff, 0)?;
            let mut bands = [0; 3];
            for (tag, band) in (1..=3).zip(&mut bands) {
                *band =
                    i32::from_le_bytes(raster_io_record(&mut trial, tag, 4)?.try_into().unwrap());
            }
            let records = (4..=11)
                .zip([0x930, 1, 4, 8, 4, 4, 32, 2])
                .map(|(tag, len)| raster_io_record(&mut trial, tag, len).map(<[u8]>::to_vec))
                .collect::<io::Result<_>>()?;
            let mut tail_trial = trial.clone();
            let mut tail_table = staged.clone();
            let (tail, stop) =
                match read_rgb_tail(&mut tail_trial, max_bytes, &mut tail_table, version) {
                    Ok(tail) => {
                        trial = tail_trial;
                        staged = tail_table;
                        (Some(Box::new(tail)), None)
                    }
                    Err(error) => (None, Some(error.to_string())),
                };
            RasterRendererPrefix10_1::Rgb {
                version,
                bands,
                records,
                tail,
                stop,
            }
        }
        SIMPLE_RASTER_RENDERER_CLASS_ID => {
            let version = signed_long(&mut trial)?;
            let short = (version >= 2).then(|| trial.read_u16_le()).transpose()?;
            let shared = read_shared_tail(&mut trial, max_bytes, &mut staged)?;
            RasterRendererPrefix10_1::Simple {
                version,
                short,
                shared,
            }
        }
        SAI_RENDERER_CLASS_ID => {
            let version = signed_long(&mut trial)?;
            if version != 1 {
                return Err(invalid(format!("未支持的 SAIRenderer 版本 {version}")));
            }
            // 新对象构造器将共用状态位清零，Load 在此状态下读取两项 double。
            let short = trial.read_u16_le()?;
            let doubles = [trial.read_f64_le()?, trial.read_f64_le()?];
            let shared = read_shared_tail(&mut trial, max_bytes, &mut staged)?;
            RasterRendererPrefix10_1::Sai {
                version,
                short,
                doubles,
                shared,
            }
        }
        DISCRETE_COLOR_RENDERER_CLASS_ID => RasterRendererPrefix10_1::Discrete(Box::new(
            read_discrete_color_renderer(&mut trial, max_bytes, &mut staged)?,
        )),
        RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID => RasterRendererPrefix10_1::UniqueValue(Box::new(
            read_raster_unique_value(&mut trial, max_bytes, &mut staged)?,
        )),
        RASTER_CLASSIFY_RENDERER_CLASS_ID => RasterRendererPrefix10_1::Classify(Box::new(
            read_raster_classify_renderer(&mut trial, max_bytes, &mut staged)?,
        )),
        RASTER_STRETCH_RENDERER_CLASS_ID => RasterRendererPrefix10_1::Stretch(Box::new(
            read_raster_stretch_renderer(&mut trial, max_bytes, &mut staged)?,
        )),
        _ => {
            return Err(invalid(format!(
                "未支持的栅格渲染器类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    *reader = trial;
    *table = staged;
    Ok(body)
}

pub(super) fn read_renderer_legend_group(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LegendGroupSlot10_1> {
    Ok(match read_object_entry_10_1(reader, table)? {
        ObjectEntry10_1::Null => LegendGroupSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            LegendGroupSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        } => {
            let group = read_legend_group_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = group.stop {
                return Err(invalid(format!("栅格渲染器 LegendGroup 未闭合：{stop:?}")));
            }
            LegendGroupSlot10_1::LegendGroup(Box::new(group))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(invalid(format!(
                "未支持的栅格渲染器 LegendGroup 类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    })
}

fn read_discrete_color_renderer(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<DiscreteColorRenderer10_1> {
    let version = signed_long(reader)?;
    // 0x7c967c60 对版本只设上限；写侧的表只产出 v1。
    if version != 1 {
        return Err(invalid(format!(
            "未支持的 RasterDiscreteColorRenderer 版本 {version}"
        )));
    }
    let count = usize::try_from(signed_long(reader)?)
        .map_err(|_| invalid("RasterDiscreteColorRenderer 颜色数量为负"))?;
    let len = count
        .checked_mul(4)
        .ok_or_else(|| invalid("离散色数组字节数溢出"))?;
    if len > max_bytes {
        return Err(invalid("离散色数组超过资源限额"));
    }
    let colors = reader
        .read_bytes(len)?
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    let text = read_string_10_1(reader, max_bytes)?;
    let ramp = read_renderer_color_ramp(reader, max_bytes, table)?;
    let shared = read_shared_tail(reader, max_bytes, table)?;
    Ok(DiscreteColorRenderer10_1 {
        version,
        colors,
        text,
        ramp,
        shared,
    })
}

pub(super) fn read_renderer_color_ramp(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ColorRampSlot10_1> {
    let offset = reader.position();
    let (ramp, stop) =
        read_color_ramp_slot_10_1(reader, max_bytes, table, MAX_SYMBOL_NESTING_DEPTH, offset)?;
    if let Some(stop) = stop {
        return Err(invalid(format!("栅格渲染器颜色梯度未闭合：{stop:?}")));
    }
    ramp.ok_or_else(|| invalid("栅格渲染器颜色梯度槽缺失"))
}

fn read_colormap_tail(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    colormap_version: i32,
) -> io::Result<ColormapTail10_1> {
    let shared = read_shared_tail(reader, max_bytes, table)?;
    let colormap_values = (0..match colormap_version {
        1 => 0,
        2 => 1,
        _ => 3,
    })
        .map(|_| signed_long(reader))
        .collect::<io::Result<_>>()?;
    Ok(ColormapTail10_1 {
        shared,
        colormap_values,
    })
}

fn read_legacy_rgb(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    version: i32,
) -> io::Result<LegacyRgbRenderer10_1> {
    // RasterRenderer.dll 0x7c99aae0：逐个读取定长成员，随后才进入共用体。
    let bands = [
        signed_long(reader)?,
        signed_long(reader)?,
        signed_long(reader)?,
    ];
    let lookup_table = reader.read_bytes(0x930)?.to_vec();
    let byte_value = reader.read_bytes(1)?[0];
    let first_long = signed_long(reader)?;
    let double_value = reader.read_f64_le()?;
    let longs = [first_long, signed_long(reader)?, signed_long(reader)?];
    let mut double_groups = [[0.0; 4]; 2];
    for value in double_groups.iter_mut().flatten() {
        *value = reader.read_f64_le()?;
    }
    let mut color = || match read_color_slot_10_1(reader, max_bytes, table)? {
        ColorSlot10_1::Unsupported { class_id, .. } => Err(invalid(format!(
            "旧式 RGB 渲染器含未支持的 IColor 类 {}",
            crate::storage::class_id_hex(&class_id)
        ))),
        value => Ok(value),
    };
    let colors = [color()?, color()?];
    let shared = read_shared_tail(reader, max_bytes, table)?;
    let trailing_flag = (version >= 2).then(|| signed_long(reader)).transpose()?;
    let trailing_long = (version >= 3).then(|| signed_long(reader)).transpose()?;
    Ok(LegacyRgbRenderer10_1 {
        version,
        bands,
        lookup_table,
        byte_value,
        double_value,
        longs,
        double_groups,
        colors,
        shared,
        trailing_flag,
        trailing_long,
    })
}

pub(super) fn read_shared_tail(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterRendererShared10_1> {
    let shared_version = signed_long(reader)?;
    if shared_version == 1 {
        // 0x7c98c310：唯一受支持的旧共用版本，读取四个 int32 后立即返回。
        return Ok(RasterRendererShared10_1 {
            version: shared_version,
            layout: RasterRendererSharedLayout10_1::Legacy([
                signed_long(reader)?,
                signed_long(reader)?,
                signed_long(reader)?,
                signed_long(reader)?,
            ]),
            shared_short_a: None,
            shared_long_a: None,
            shared_long_b: None,
            shared_byte: None,
            shared_short_b: None,
            color_ramp: None,
        });
    }
    if !(2..=9).contains(&shared_version) {
        return Err(invalid(format!(
            "未支持的栅格渲染器共享版本 {shared_version}"
        )));
    }
    raster_io_record(reader, 0x00ff_00ff, 0)?;
    let mut raster_io_integers = [None; 5];
    raster_io_integers[0] = Some(i32::from_le_bytes(
        raster_io_record(reader, 1, 4)?.try_into().unwrap(),
    ));
    let next = signed_long(&mut reader.clone())?;
    for (index, value) in raster_io_integers.iter_mut().enumerate().skip(1) {
        let tag = index as i32 + 1;
        // 色表类的两个受共同开关控制的字段以相邻 tag 2、3 出现；均缺时原生直接读 tag 4。
        if !matches!(tag, 2 | 3) || next == 2 {
            *value = Some(i32::from_le_bytes(
                raster_io_record(reader, tag, 4)?.try_into().unwrap(),
            ));
        }
    }
    let raster_io_color = read_raster_io_color(reader, 6)?;
    let mut doubles = Reader::new(raster_io_record(reader, 7, 24)?);
    let raster_io_doubles = [
        doubles.read_f64_le()?,
        doubles.read_f64_le()?,
        doubles.read_f64_le()?,
    ];
    read_raster_io_end(reader)?;
    let shared_short_a = (shared_version >= 4)
        .then(|| reader.read_u16_le())
        .transpose()?;
    let shared_long_a = (shared_version >= 5)
        .then(|| signed_long(reader))
        .transpose()?;
    let shared_long_b = (shared_version >= 6)
        .then(|| signed_long(reader))
        .transpose()?;
    let shared_byte = (shared_version >= 7)
        .then(|| reader.read_bytes(1).map(|b| b[0]))
        .transpose()?;
    let shared_short_b = (shared_version >= 8)
        .then(|| reader.read_u16_le())
        .transpose()?;
    let color_ramp = (shared_version >= 9)
        .then(|| read_renderer_color_ramp(reader, max_bytes, table))
        .transpose()?;
    Ok(RasterRendererShared10_1 {
        version: shared_version,
        layout: RasterRendererSharedLayout10_1::RasterIo {
            integers: raster_io_integers,
            color: raster_io_color,
            doubles: raster_io_doubles,
        },
        shared_short_a,
        shared_long_a,
        shared_long_b,
        shared_byte,
        shared_short_b,
        color_ramp,
    })
}

pub(super) fn raster_io_object_class(reader: &mut Reader<'_>, tag: i32) -> io::Result<[u8; 16]> {
    if signed_long(reader)? != tag || signed_long(reader)? != -1 {
        return Err(invalid(format!("栅格渲染器 RasterIO 缺少 tag {tag} 对象")));
    }
    Ok(reader.read_bytes(16)?.try_into().unwrap())
}

fn read_raster_io_color(reader: &mut Reader<'_>, tag: i32) -> io::Result<Vec<u8>> {
    let start = reader.clone();
    let class_id = raster_io_object_class(reader, tag)?;
    match class_id {
        NULL_CLASS_ID_10_1 => (),
        RGB_COLOR_CLASS_ID => {
            // RasterIO 不使用 ObjectStream 键；复用 RGB 布局检查，不保留虚拟键。
            read_rgb_color_body_10_1(reader, &mut RefTable10_1::new(), 0)?;
        }
        _ => {
            return Err(invalid(format!(
                "栅格渲染器 RasterIO tag {tag} 存在未知颜色类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    }
    let mut bytes = start;
    bytes.read_bytes(8)?;
    Ok(bytes
        .read_bytes(reader.position() - bytes.position())?
        .to_vec())
}

pub(super) fn read_raster_io_end(reader: &mut Reader<'_>) -> io::Result<()> {
    if signed_long(reader)? != -1 || signed_long(reader)? != -1 {
        return Err(invalid("栅格渲染器 RasterIO 缺少结束记录"));
    }
    Ok(())
}

pub(super) fn next_is_tag(reader: &Reader<'_>, tag: i32) -> io::Result<bool> {
    Ok(signed_long(&mut reader.clone())? == tag)
}

pub(super) fn optional_record<'a>(
    reader: &mut Reader<'a>,
    tag: i32,
    size: usize,
) -> io::Result<Option<Reader<'a>>> {
    next_is_tag(reader, tag)?
        .then(|| raster_io_record(reader, tag, size).map(Reader::new))
        .transpose()
}

pub(super) fn optional_long_record(reader: &mut Reader<'_>, tag: i32) -> io::Result<Option<i32>> {
    optional_record(reader, tag, 4)?
        .map(|mut r| signed_long(&mut r))
        .transpose()
}

fn read_double_sequence(reader: &mut Reader<'_>, len: usize) -> io::Result<Vec<f64>> {
    if len > reader.remaining() / 8 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "栅格渲染器的 double 数组超出可读范围",
        ));
    }
    (0..len).map(|_| reader.read_f64_le()).collect()
}

pub(super) fn read_raster_histogram(
    reader: &mut Reader<'_>,
    tag: i32,
) -> io::Result<Option<RasterHistogramSlot10_1>> {
    if !next_is_tag(reader, tag)? {
        return Ok(None);
    }
    let class_id = raster_io_object_class(reader, tag)?;
    if class_id == NULL_CLASS_ID_10_1 {
        return Ok(Some(RasterHistogramSlot10_1::Null));
    }
    if class_id != STATS_HISTOGRAM_CLASS_ID {
        return Err(invalid(format!(
            "栅格渲染器 RasterIO tag {tag} 存在未知统计类 {}",
            crate::storage::class_id_hex(&class_id)
        )));
    }

    Ok(Some(RasterHistogramSlot10_1::Histogram(Box::new(
        read_stats_histogram_body_10_1(reader)?,
    ))))
}

/// RasterDB.dll 0x7c101910 / 0x7c101b30；RasterIO 与普通对象流共用该对象体。
/// 调用者已读对象头，并负责对象体失败时与外层头一并回退。
pub(crate) fn read_stats_histogram_body_10_1(
    reader: &mut Reader<'_>,
) -> io::Result<RasterHistogram10_1> {
    let version = signed_long(reader)?;
    if !(1..=5).contains(&version) {
        return Err(invalid(format!("未支持的 StatsHistogram 版本 {version}")));
    }
    let mut statistics = [0.0; 6];
    for value in &mut statistics {
        *value = reader.read_f64_le()?;
    }
    let len = if version < 3 {
        256
    } else {
        usize::try_from(signed_long(reader)?).map_err(|_| invalid("StatsHistogram 桶数量为负"))?
    };
    let bins = read_double_sequence(reader, len)?;
    let double_a = (version >= 2).then(|| reader.read_f64_le()).transpose()?;
    let double_b = (version >= 4).then(|| reader.read_f64_le()).transpose()?;
    let additional_bins = if version >= 5 {
        let len = count(reader, 8)?;
        Some(read_double_sequence(reader, len)?)
    } else {
        None
    };
    Ok(RasterHistogram10_1 {
        version,
        statistics,
        bins,
        double_a,
        double_b,
        additional_bins,
    })
}

pub(crate) fn read_double_array_slot(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<RasterDoubleArraySlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if registered != DOUBLE_ARRAY_CLASS_ID {
            return Err(invalid(format!(
                "IDoubleArray 回引键 {key} 的登记类不受支持"
            )));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == DOUBLE_ARRAY_CLASS_ID => {
            Some(read_double_array_body(reader)?)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(invalid(format!(
                "IDoubleArray 槽存在未知类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterDoubleArraySlot10_1 { entry, body })
}

pub(crate) fn read_double_array_body(reader: &mut Reader<'_>) -> io::Result<(u16, Vec<f64>)> {
    // SystemCore.dll 0x800419b0 -> 0x80040520：short、int32 数量、double 循环。
    let version = reader.read_u16_le()?;
    if (version as i16) > 1 {
        return Err(invalid(format!("未支持的 DoubleArray 版本 {version}")));
    }
    let len = count(reader, 8)?;
    Ok((version, read_double_sequence(reader, len)?))
}

pub(crate) fn read_load_raster(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    xform_depth: usize,
) -> io::Result<Option<LoadedRaster10_1>> {
    // RasterCoreLib.dll 0x7bf2edf0：IName 槽走 AfCore LoadObject，沿用底流表。
    // 名称内也可包含变换，从当前预算进入名称，避免跨两种对象的递归重置限额。
    let RasterNameSlot10_1 {
        entry: name_entry,
        body: name_body,
    } = read_name_slot(
        reader,
        max_bytes,
        table,
        (xform_depth + 1).min(MAX_XFORM_NESTING),
    )?;
    if name_entry == ObjectEntry10_1::Null {
        return Ok(None);
    }
    // 每条编码名称至少为 4 字节长度和 2 字节终止零。
    let declared_band_count = signed_long(reader)?;
    let band_names = (0..bounded_count(reader, declared_band_count, 6)?)
        .map(|_| read_string_10_1(reader, max_bytes))
        .collect::<io::Result<_>>()?;
    let transform = read_geodata_xform_slot_with_depth(reader, max_bytes, table, xform_depth)?;
    let extent = read_envelope_slot(reader, max_bytes, table)?;
    Ok(Some(LoadedRaster10_1 {
        name_entry,
        name_body,
        declared_band_count,
        band_names,
        transform,
        extent,
    }))
}

pub(super) fn read_pansharpening_filter(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PansharpeningFilter10_1> {
    // RasterDB.dll 0x7c02c190 / 0x7c02c090：10.1 写出版本 1。
    let version = signed_long(reader)?;
    if version != 1 {
        return Err(invalid(format!("未支持的全色锐化过滤器版本 {version}")));
    }
    let rasters = [
        read_load_raster(reader, max_bytes, table, MAX_XFORM_NESTING)?,
        read_load_raster(reader, max_bytes, table, MAX_XFORM_NESTING)?,
    ];
    let method = signed_long(reader)?;
    let mut weights = [0.0; 4];
    for weight in &mut weights {
        *weight = reader.read_f64_le()?;
    }
    Ok(PansharpeningFilter10_1 {
        version,
        rasters,
        method,
        weights,
    })
}

fn read_rgb_tail(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    version: i32,
) -> io::Result<RgbTail10_1> {
    let color = read_raster_io_color(reader, 12)?;
    let tag13 = optional_long_record(reader, 13)?;
    let red = read_raster_histogram(reader, 14)?;
    let green = read_raster_histogram(reader, 15)?;
    let blue = read_raster_histogram(reader, 16)?;
    let tag17 = optional_long_record(reader, 17)?;
    let histograms = [red, green, blue];
    let pansharpening = if next_is_tag(reader, 18)? {
        let value = match raster_io_object_class(reader, 18)? {
            NULL_CLASS_ID_10_1 => PansharpeningSlot10_1::Null,
            PANSHARPENING_FILTER_CLASS_ID => PansharpeningSlot10_1::Filter(Box::new(
                read_pansharpening_filter(reader, max_bytes, table)?,
            )),
            class_id => {
                return Err(invalid(format!(
                    "RGB 渲染器 RasterIO tag 18 含未支持的类 {}",
                    crate::storage::class_id_hex(&class_id)
                )));
            }
        };
        Some(value)
    } else {
        None
    };
    let tag19 = optional_long_record(reader, 19)?;
    let tag20 = optional_record(reader, 20, 24)?
        .map(|mut r| -> io::Result<_> {
            Ok([r.read_f64_le()?, r.read_f64_le()?, r.read_f64_le()?])
        })
        .transpose()?;
    let (tag21, tag22) = if version >= 5 {
        let first = optional_record(reader, 21, 8)?
            .map(|mut r| r.read_f64_le())
            .transpose()?;
        let second = optional_record(reader, 22, 8)?
            .map(|mut r| r.read_f64_le())
            .transpose()?;
        (first, second)
    } else {
        (None, None)
    };
    // 该单个 -1 与 IDoubleArray 均在 RasterIO::End 的 (-1,-1) 之前。
    if version >= 6 && signed_long(reader)? != -1 {
        return Err(invalid("RGB 渲染器缺少尾部 -1 标记"));
    }
    let trailing_short = (version >= 7).then(|| reader.read_u16_le()).transpose()?;
    let array_count = if version >= 9 {
        4
    } else if version >= 7 {
        2
    } else {
        0
    };
    let double_arrays = (0..array_count)
        .map(|_| read_double_array_slot(reader, table))
        .collect::<io::Result<_>>()?;
    read_raster_io_end(reader)?;
    let shared = read_shared_tail(reader, max_bytes, table)?;
    Ok(RgbTail10_1 {
        color,
        tag13,
        histograms,
        tag17,
        pansharpening,
        tag19,
        tag20,
        tag21,
        tag22,
        trailing_short,
        double_arrays,
        shared,
    })
}
