//! ImageServerLayer 四个 10.1 写出版本的后续字段与对象槽。
use super::dep_tail::{
    TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, TimeExtentBody10_1, TimeReferenceBody10_1,
    read_time_extent_body_10_1, read_time_reference_body_10_1,
};
use super::feature_layer_tail::{
    FIELD_INFO_CLASS_ID, FieldInfoBody10_1, read_field_info_body_10_1,
};
use super::image_server_layer::{
    ImageServerLayerPrefix10_1, read_image_server_layer_prefix_10_1, read_object, read_scalar,
    require_vt,
};
use super::image_server_rules::{
    MOSAIC_RULE_CLASS_ID, MosaicRule10_1, RENDERING_RULE_CLASS_ID, RenderingRule10_1,
    read_mosaic_rule_10_1, read_rendering_rule_10_1,
};
use crate::layers::raster_renderer::{RasterRendererPrefix10_1, read_raster_renderer_body_10_1};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::fields::{FIELDS_CLASS_ID, FieldsBody10_1, read_fields_body_10_1};
use crate::objects::geodata_xform::{GeodataXformBody10_1, read_geodata_xform_body_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use crate::stream::variant_stream::{VariantValue10_1, VariantValue10_8};
use std::io::{self, ErrorKind};

/// ImageServer.dll 的 ImageServerName（IPersistStream）。
pub const IMAGE_SERVER_NAME_CLASS_ID: [u8; 16] = [
    0xec, 0x2f, 0x55, 0xbd, 0x0b, 0xff, 0xf1, 0x4b, 0x8c, 0x46, 0xb6, 0x88, 0x6e, 0x71, 0xa4, 0x7c,
];

#[derive(Clone, Debug, PartialEq)]
pub struct ImageServerLayerBody10_1 {
    pub prefix: ImageServerLayerPrefix10_1,
    /// 按保存顺序保留已读取的 Variant 字段；计数也保留。
    pub tail: Vec<ImageServerTailField10_1>,
    /// 无法再读取的第一个字段。未知字段不会进入 tail 或登记其对象键。
    pub stop: Option<ImageServerTailStop10_1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageServerTailStop10_1 {
    pub offset: usize,
    pub error_kind: ErrorKind,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageServerTailField10_1 {
    Scalar(VariantValue10_8),
    Object(VariantValue10_1<Box<ImageServerTailObject10_1>>),
    Color(ColorSlot10_1),
    Symbol(SymbolSlotFull10_1),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageServerTailObject10_1 {
    Name(ImageServerName10_1),
    Envelope(Envelope10_1),
    FieldInfo(FieldInfoBody10_1),
    Fields(FieldsBody10_1),
    TimeReference(TimeReferenceBody10_1),
    TimeExtent(TimeExtentBody10_1),
    RasterRenderer(Box<RasterRendererPrefix10_1>),
    GeodataXform(GeodataXformBody10_1),
    MosaicRule(Box<MosaicRule10_1>),
    RenderingRule(Box<RenderingRule10_1>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageServerName10_1 {
    pub version: u16,
    /// ImageServerName 内保存的 String，保留原值。
    pub text: String,
    /// 普通 LoadObject 条目，无 VARTYPE；复用外层的对象流表。
    pub properties: ObjectEntry10_1,
    /// 仅当 properties 是新 PropertySet 时包含对象体。
    pub new_properties: Option<PropertySetObjectStream10_1>,
}

/// 读取 ImageServerLayer 到对象结束或首个不可消费的尾字段。
/// 前缀错误原样返回并回退；尾字段错误保存在 stop，之前的完整对象键照常提交。
/// 未读尾部须由调用者在既知外层范围内保留，不能把 stop 当成对象终点。
pub fn read_image_server_layer_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ImageServerLayerBody10_1> {
    let prefix = read_image_server_layer_prefix_10_1(reader, max_bytes, table)?;
    let mut tail = TailReader {
        reader,
        table,
        fields: Vec::new(),
        max_bytes,
    };
    let stop = tail
        .read(prefix.version)
        .err()
        .map(|error| ImageServerTailStop10_1 {
            offset: tail.reader.position(),
            error_kind: error.kind(),
            reason: error.to_string(),
        });
    Ok(ImageServerLayerBody10_1 {
        prefix,
        tail: tail.fields,
        stop,
    })
}

/// 字节边界由 Load 与 Save 共同确认；未实现的非空类仍可识别空值和回引。
#[derive(Clone, Copy)]
enum ObjectSlot {
    Name,
    RasterRenderer,
    Envelope,
    Fields,
    FieldInfo,
    TimeReference,
    TimeExtent,
    MosaicRule,
    RenderingRule,
    GeodataXform,
    ArrayEntry,
}

impl ObjectSlot {
    fn label(self) -> &'static str {
        match self {
            Self::Name => "IName",
            Self::RasterRenderer => "IRasterRenderer",
            Self::Envelope => "IEnvelope",
            Self::Fields => "IFields",
            Self::FieldInfo => "IFieldInfo",
            Self::TimeReference => "ITimeReference",
            Self::TimeExtent => "ITimeExtent",
            Self::MosaicRule => "IMosaicRule",
            Self::RenderingRule => "IRenderingRule",
            Self::GeodataXform => "IGeodataXform",
            Self::ArrayEntry => "v21 数组项",
        }
    }
}

struct TailReader<'a, 'data> {
    reader: &'a mut Reader<'data>,
    table: &'a mut RefTable10_1,
    fields: Vec<ImageServerTailField10_1>,
    max_bytes: usize,
}

impl TailReader<'_, '_> {
    fn read(&mut self, version: i32) -> io::Result<()> {
        use ObjectSlot::*;
        self.scalars(&[8])?;
        if version == 8 {
            self.scalars(&[11])?;
        }
        self.scalars(&[8])?;
        self.object(Name)?;
        if version >= 9 {
            self.object(RasterRenderer)?;
            self.object(Envelope)?;
            self.scalars(&[8, 3, 11])?;
        }
        if version >= 11 {
            self.scalars(&[8, 8])?;
        }
        if version >= 12 {
            self.i4_list()?;
        }
        if version >= 13 {
            self.scalars(&[11])?;
        }
        if version >= 14 {
            self.object(Fields)?;
            self.object_list(FieldInfo)?;
            self.scalars(&[8])?;
        }
        if version >= 15 {
            self.scalars(&[8, 11])?;
            self.i4_list()?;
            self.color()?;
            self.symbol()?;
            self.scalars(&[11, 11, 3])?;
            self.symbol()?;
        }
        if version >= 16 {
            self.i4_list()?;
        }
        if version >= 17 {
            self.scalars(&[11])?;
        }
        if version >= 18 {
            self.scalars(&[11])?;
            self.object(TimeReference)?;
            self.scalars(&[8, 8, 8, 8])?;
            self.object(TimeExtent)?;
            self.scalars(&[11, 5, 3, 5, 3])?;
        }
        if version >= 19 {
            self.object(MosaicRule)?;
            self.object(RenderingRule)?;
        }
        if version >= 20 {
            self.object(GeodataXform)?;
        }
        if version >= 21 {
            self.object_list(ArrayEntry)?;
        }
        if version >= 22 {
            self.scalars(&[11, 11, 3])?;
        }
        Ok(())
    }

    fn scalars(&mut self, types: &[u16]) -> io::Result<()> {
        let max_units = (self.max_bytes / 2).min(u32::MAX as usize) as u32;
        for &vt in types {
            let value = read_scalar(self.reader, max_units, vt)?;
            self.fields.push(ImageServerTailField10_1::Scalar(value));
        }
        Ok(())
    }

    fn count(&mut self, min_bytes: usize) -> io::Result<usize> {
        let mut trial = self.reader.clone();
        let value = read_scalar(&mut trial, 0, 3)?;
        let VariantValue10_8::Raw4(raw) = value else {
            unreachable!("已确认 VT_I4")
        };
        let count = usize::try_from(i32::from_le_bytes(raw))
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "ImageServerLayer 尾部计数为负"))?;
        if count > self.max_bytes / min_bytes {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "ImageServerLayer 尾部计数超过资源限额",
            ));
        }
        if count > trial.remaining() / min_bytes {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "ImageServerLayer 尾部计数超出可读范围",
            ));
        }
        *self.reader = trial;
        self.fields.push(ImageServerTailField10_1::Scalar(value));
        Ok(count)
    }

    fn i4_list(&mut self) -> io::Result<()> {
        for _ in 0..self.count(6)? {
            self.scalars(&[3])?;
        }
        Ok(())
    }

    fn object_list(&mut self, slot: ObjectSlot) -> io::Result<()> {
        for _ in 0..self.count(18)? {
            self.object(slot)?;
        }
        Ok(())
    }

    fn object(&mut self, slot: ObjectSlot) -> io::Result<()> {
        let max_bytes = self.max_bytes;
        let value = read_object(self.reader, self.table, |body, header, refs| {
            let key = header.key.expect("ObjectStream 对象有键");
            let decoded = match (slot, header.class_id) {
                (ObjectSlot::Name, IMAGE_SERVER_NAME_CLASS_ID) => {
                    ImageServerTailObject10_1::Name(read_name(body, max_bytes, refs)?)
                }
                (ObjectSlot::Envelope, ENVELOPE_CLASS_ID) => {
                    ImageServerTailObject10_1::Envelope(read_envelope_10_1(body, max_bytes, refs)?)
                }
                (ObjectSlot::Fields, FIELDS_CLASS_ID) => {
                    ImageServerTailObject10_1::Fields(read_fields_body_10_1(body, max_bytes, refs)?)
                }
                (ObjectSlot::MosaicRule, MOSAIC_RULE_CLASS_ID) => {
                    ImageServerTailObject10_1::MosaicRule(Box::new(read_mosaic_rule_10_1(
                        body, max_bytes, refs,
                    )?))
                }
                (ObjectSlot::RenderingRule, RENDERING_RULE_CLASS_ID) => {
                    ImageServerTailObject10_1::RenderingRule(Box::new(read_rendering_rule_10_1(
                        body, max_bytes, refs,
                    )?))
                }
                (ObjectSlot::FieldInfo, FIELD_INFO_CLASS_ID) => {
                    let info = read_field_info_body_10_1(body, max_bytes, refs, key)?;
                    if info.stop.is_some() {
                        return Err(unsupported(
                            "ImageServerLayer IFieldInfo 的嵌套对象体未闭合",
                        ));
                    }
                    ImageServerTailObject10_1::FieldInfo(info)
                }
                (ObjectSlot::TimeReference, TIME_REFERENCE_CLASS_ID) => {
                    let reference = read_time_reference_body_10_1(body, max_bytes, refs, key)?;
                    if reference.short_b.is_none() {
                        return Err(unsupported(
                            "ImageServerLayer ITimeReference 的嵌套对象体未闭合",
                        ));
                    }
                    ImageServerTailObject10_1::TimeReference(reference)
                }
                (ObjectSlot::TimeExtent, TIME_EXTENT_CLASS_ID) => {
                    let extent = read_time_extent_body_10_1(body, max_bytes, refs, key)?;
                    if extent.stop.is_some() {
                        return Err(unsupported(
                            "ImageServerLayer ITimeExtent 的嵌套对象体未闭合",
                        ));
                    }
                    ImageServerTailObject10_1::TimeExtent(extent)
                }
                (ObjectSlot::RasterRenderer, class_id) => {
                    let renderer = read_raster_renderer_body_10_1(body, max_bytes, refs, class_id)?;
                    if !renderer.is_complete() {
                        return Err(unsupported("ImageServerLayer IRasterRenderer 对象体未闭合"));
                    }
                    ImageServerTailObject10_1::RasterRenderer(Box::new(renderer))
                }
                (ObjectSlot::GeodataXform, class_id) => ImageServerTailObject10_1::GeodataXform(
                    read_geodata_xform_body_10_1(body, max_bytes, refs, class_id)?,
                ),
                _ => {
                    return Err(unsupported(format!(
                        "ImageServerLayer {} 类 {}（键 {key}）尚无对象体读取器",
                        slot.label(),
                        crate::storage::class_id_hex(&header.class_id)
                    )));
                }
            };
            Ok(Box::new(decoded))
        })?;
        self.fields.push(ImageServerTailField10_1::Object(value));
        Ok(())
    }

    fn color(&mut self) -> io::Result<()> {
        let mut trial = self.reader.clone();
        require_vt(&trial, 13)?;
        trial.read_u16_le()?;
        let slot = read_color_slot_10_1(&mut trial, self.max_bytes, self.table)?;
        if let ColorSlot10_1::Unsupported { class_id, key } = slot {
            return Err(unsupported(format!(
                "ImageServerLayer IColor 类 {}（键 {key}）尚无对象体读取器",
                crate::storage::class_id_hex(&class_id)
            )));
        }
        *self.reader = trial;
        self.fields.push(ImageServerTailField10_1::Color(slot));
        Ok(())
    }

    fn symbol(&mut self) -> io::Result<()> {
        let mut trial = self.reader.clone();
        require_vt(&trial, 13)?;
        trial.read_u16_le()?;
        let mut refs = self.table.clone();
        let slot = read_symbol_slot_full_10_1(
            &mut trial,
            self.max_bytes,
            &mut refs,
            MAX_SYMBOL_NESTING_DEPTH,
        )?;
        if let Some(stop) = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol) {
            return Err(unsupported(format!(
                "ImageServerLayer ISymbol 对象体未闭合：{stop:?}"
            )));
        }
        *self.reader = trial;
        *self.table = refs;
        self.fields.push(ImageServerTailField10_1::Symbol(slot));
        Ok(())
    }
}

pub(super) fn read_name(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ImageServerName10_1> {
    // 外部 Variant 对象读取器已有完整事务；此处沿用同一张表。
    let version = reader.read_u16_le()?;
    if version > 1 {
        return Err(unsupported(format!(
            "未支持的 ImageServerName 版本 {version}"
        )));
    }
    let text = read_string_10_1(reader, max_bytes)?;
    let properties = read_object_entry_10_1(reader, table)?;
    let new_properties = match properties {
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == PROPERTY_SET_CLASS_ID => Some(
            read_property_set_object_stream_10_1(reader, max_bytes, table)?,
        ),
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(unsupported(format!(
                "ImageServerName IPropertySet 类 {}（键 {key}）尚无对象体读取器",
                crate::storage::class_id_hex(&class_id)
            )));
        }
        _ => None,
    };
    Ok(ImageServerName10_1 {
        version,
        text,
        properties,
        new_properties,
    })
}

fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, message.into())
}
