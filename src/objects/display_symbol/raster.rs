//! 符号槽中由 RasterLayer.dll 注册的类。原生读写见 `docs/arcmap-10.1-dep-tail.md`。

use super::{SymbolStop10_1, color_slot_stop, read_i32_le, read_signed_symbol_version};
use crate::objects::color_ramp::{ColorRampSlot10_1, read_color_ramp_slot_10_1};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::reader::Reader;
use std::io;

pub const COLOR_SYMBOL_CLASS_ID: [u8; 16] = [
    0xe0, 0x9a, 0x1f, 0xb8, 0x6e, 0x02, 0xd3, 0x11, 0x9c, 0x1f, 0x00, 0xc0, 0x4f, 0x5a, 0xa6, 0xed,
];
pub const COLOR_RAMP_SYMBOL_CLASS_ID: [u8; 16] = [
    0x40, 0x70, 0x98, 0x40, 0x4c, 0x20, 0xd3, 0x11, 0xa3, 0xf2, 0x00, 0x04, 0xac, 0x1b, 0x1d, 0x86,
];
pub const RASTER_RGB_SYMBOL_CLASS_ID: [u8; 16] = [
    0x66, 0xcb, 0xdc, 0x99, 0x09, 0x2e, 0xd3, 0x11, 0xa6, 0x26, 0x00, 0x08, 0xc7, 0xbf, 0x33, 0x47,
];

/// ColorSymbol 的持久化体，RasterRGBSymbol 的 IPersistStream 共用此布局。
#[derive(Debug, Clone, PartialEq)]
pub struct ColorSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub color: ColorSlot10_1,
    /// 流中依次写入的三个整数。末项在 10.1 写侧置零。
    pub integers: Option<[i32; 3]>,
    pub stop: Option<SymbolStop10_1>,
}

impl ColorSymbolBody10_1 {
    /// ISymbol::ROP2 的原始 esriRasterOpCode。
    pub fn rop2(&self) -> Option<i32> {
        self.integers.map(|fields| fields[0])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorRampSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub color: ColorSlot10_1,
    /// IColorRamp 槽之前的三个整数。末项在 10.1 写侧置零。
    pub integers: Option<[i32; 3]>,
    pub ramp: Option<ColorRampSlot10_1>,
    pub tail: Option<ColorRampSymbolTail10_1>,
    pub stop: Option<SymbolStop10_1>,
}

impl ColorRampSymbolBody10_1 {
    /// ISymbol::ROP2 的原始 esriRasterOpCode。
    pub fn rop2(&self) -> Option<i32> {
        self.integers.map(|fields| fields[0])
    }
}

/// 梯度槽之后按版本保存的字段。按流序保留尚无语义定名的值。
#[derive(Debug, Clone, PartialEq)]
pub struct ColorRampSymbolTail10_1 {
    /// IColorRampSymbol::Invert 的原始 VARIANT_BOOL。
    pub first_short: u16,
    /// 有符号版本 <2。
    pub legacy_short: Option<u16>,
    /// IColorRampSymbol::LegendClassIndex。
    pub integer: i32,
    /// 版本 ≥3，依次为透明度、亮度和对比度。
    pub shorts_v3: Option<[u16; 3]>,
    /// 版本 ≥4。
    pub integer_v4: Option<i32>,
    /// 版本 ≥5，依次为 ShowTics 和 Horizontal。
    pub shorts_v5: Option<[u16; 2]>,
}

impl ColorRampSymbolTail10_1 {
    pub fn invert(&self) -> bool {
        self.first_short != 0
    }

    pub fn legend_class_index(&self) -> i32 {
        self.integer
    }

    pub fn show_tics(&self) -> Option<bool> {
        self.shorts_v5.map(|fields| fields[0] != 0)
    }

    pub fn horizontal(&self) -> Option<bool> {
        self.shorts_v5.map(|fields| fields[1] != 0)
    }

    /// 返回原生绘制传给 ApplyEffects 的透明度、亮度和对比度。
    pub fn color_effects(&self) -> Option<(i16, i16, i16)> {
        self.shorts_v3.map(|[transparency, brightness, contrast]| {
            (transparency as i16, brightness as i16, contrast as i16)
        })
    }
}

/// 调用者在外层试读游标和登记表上读取，确保失败时整体回退。
pub(super) fn read_color_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ColorSymbolBody10_1> {
    let version = read_signed_symbol_version(reader, 1, "ColorSymbol", reader.position())?;
    let color = read_color_slot_10_1(reader, max_bytes, table)?;
    let stop = color_slot_stop(&color);
    let integers = if stop.is_none() {
        Some([
            read_i32_le(reader)?,
            read_i32_le(reader)?,
            read_i32_le(reader)?,
        ])
    } else {
        None
    };
    Ok(ColorSymbolBody10_1 {
        key,
        version,
        color,
        integers,
        stop,
    })
}

pub(super) fn read_color_ramp_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<ColorRampSymbolBody10_1> {
    let offset = reader.position();
    let version = read_signed_symbol_version(reader, 5, "ColorRampSymbol", offset)?;
    let signed_version = version as i16;
    let color = read_color_slot_10_1(reader, max_bytes, table)?;
    let mut body = ColorRampSymbolBody10_1 {
        key,
        version,
        stop: color_slot_stop(&color),
        color,
        integers: None,
        ramp: None,
        tail: None,
    };
    if body.stop.is_some() {
        return Ok(body);
    }
    body.integers = Some([
        read_i32_le(reader)?,
        read_i32_le(reader)?,
        read_i32_le(reader)?,
    ]);
    (body.ramp, body.stop) = read_color_ramp_slot_10_1(reader, max_bytes, table, depth, offset)?;
    if body.stop.is_none() {
        body.tail = Some(ColorRampSymbolTail10_1 {
            first_short: reader.read_u16_le()?,
            legacy_short: (signed_version < 2)
                .then(|| reader.read_u16_le())
                .transpose()?,
            integer: read_i32_le(reader)?,
            shorts_v3: (signed_version >= 3)
                .then(|| {
                    Ok::<_, io::Error>([
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                    ])
                })
                .transpose()?,
            integer_v4: (signed_version >= 4)
                .then(|| read_i32_le(reader))
                .transpose()?,
            shorts_v5: (signed_version >= 5)
                .then(|| Ok::<_, io::Error>([reader.read_u16_le()?, reader.read_u16_le()?]))
                .transpose()?,
        });
    }
    Ok(body)
}
