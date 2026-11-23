//! RasterGraphicElements.dll 10.1.0.3035 的普通图片元素对象体。

use super::map_frame::{
    BaseFrameElement10_1, RectShapeGeometry10_1, base_frame, read_rect_shape_10_1,
};
use crate::objects::element_body::{
    ElementPropertiesPrefix10_1, PrefixOutcome10_1, read_element_properties_prefix_10_1,
};
use crate::objects::ole_font::{
    STDPICTURE_CLASS_ID, StdPictureBody10_1, read_std_picture_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureElementKind10_1 {
    Picture,
    Emf,
    Bmp,
    Jpg,
    Gif,
    Tif,
    Png,
    Jp2,
    Img,
    Sid,
}

pub const PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x92, 0x9a, 0x7b, 0x82, 0x67, 0xc0, 0xd2, 0x11, 0x9f, 0x22, 0x00, 0xc0, 0x4f, 0x6b, 0xc8, 0xdd,
];
pub const EMF_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x90, 0x9a, 0x7b, 0x82, 0x67, 0xc0, 0xd2, 0x11, 0x9f, 0x22, 0x00, 0xc0, 0x4f, 0x6b, 0xc8, 0xdd,
];
pub const BMP_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x91, 0x9a, 0x7b, 0x82, 0x67, 0xc0, 0xd2, 0x11, 0x9f, 0x22, 0x00, 0xc0, 0x4f, 0x6b, 0xc8, 0xdd,
];
pub const JPG_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x80, 0x09, 0xac, 0x06, 0x53, 0x19, 0xd3, 0x11, 0xa3, 0xee, 0x00, 0x04, 0xac, 0x1b, 0x1d, 0x86,
];
pub const GIF_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0xb0, 0x82, 0x7c, 0x4a, 0x53, 0x19, 0xd3, 0x11, 0xa3, 0xee, 0x00, 0x04, 0xac, 0x1b, 0x1d, 0x86,
];
pub const TIF_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x20, 0x09, 0x7c, 0x5e, 0xa5, 0x14, 0xd3, 0x11, 0x80, 0xcf, 0x00, 0x80, 0xc7, 0x59, 0x7e, 0x71,
];
pub const PNG_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x30, 0x63, 0xcd, 0x85, 0x5a, 0xd4, 0xd3, 0x11, 0xa4, 0x14, 0x00, 0x04, 0xac, 0x1b, 0x1d, 0x86,
];
pub const JP2_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x3b, 0x86, 0xbf, 0x8b, 0xc7, 0xd0, 0x5f, 0x4b, 0x88, 0xb0, 0x21, 0xd5, 0xa4, 0xca, 0x06, 0xfd,
];
pub const IMG_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x20, 0x9b, 0x0d, 0x1c, 0x53, 0x19, 0xd3, 0x11, 0xa3, 0xee, 0x00, 0x04, 0xac, 0x1b, 0x1d, 0x86,
];
pub const SID_PICTURE_ELEMENT_CLASS_ID: [u8; 16] = [
    0x60, 0xfc, 0x6e, 0x35, 0x53, 0x19, 0xd3, 0x11, 0xa3, 0xee, 0x00, 0x04, 0xac, 0x1b, 0x1d, 0x86,
];

impl PictureElementKind10_1 {
    pub(crate) fn from_class(class_id: &[u8; 16]) -> Option<Self> {
        Some(match *class_id {
            PICTURE_ELEMENT_CLASS_ID => Self::Picture,
            EMF_PICTURE_ELEMENT_CLASS_ID => Self::Emf,
            BMP_PICTURE_ELEMENT_CLASS_ID => Self::Bmp,
            JPG_PICTURE_ELEMENT_CLASS_ID => Self::Jpg,
            GIF_PICTURE_ELEMENT_CLASS_ID => Self::Gif,
            TIF_PICTURE_ELEMENT_CLASS_ID => Self::Tif,
            PNG_PICTURE_ELEMENT_CLASS_ID => Self::Png,
            JP2_PICTURE_ELEMENT_CLASS_ID => Self::Jp2,
            IMG_PICTURE_ELEMENT_CLASS_ID => Self::Img,
            SID_PICTURE_ELEMENT_CLASS_ID => Self::Sid,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PictureElementBase10_1 {
    Legacy {
        geometry: RectShapeGeometry10_1,
        properties: ElementPropertiesPrefix10_1,
    },
    Frame(BaseFrameElement10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PictureElement10_1 {
    /// 类与实际图片编码独立，保留类身份供上层使用。
    pub kind: PictureElementKind10_1,
    pub version: u16,
    pub base: PictureElementBase10_1,
    pub embed_flag_bits: Option<u16>,
    pub picture: Option<(ObjectEntry10_1, Option<StdPictureBody10_1>)>,
    pub legacy_short: Option<u16>,
    /// v5 起的三个 String，保留存盘值。
    pub source_strings: Option<[String; 3]>,
    pub envelope_bits: Option<[u64; 4]>,
    pub string_174: Option<String>,
    pub short_178: Option<u16>,
    pub string_17c: Option<String>,
    pub string_19c: Option<String>,
}

/// 0x7c593450 的持久化分支。调用者已消费对象头；失败时不提交游标和键表。
pub(crate) fn read_picture_element_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    kind: PictureElementKind10_1,
) -> io::Result<PictureElement10_1> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let version = trial.read_u16_le()?;
    if version > 9 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的图片元素版本",
        ));
    }
    let base = if version <= 2 {
        let geometry = read_rect_shape_10_1(&mut trial, max_bytes, &mut refs)?;
        let properties =
            match read_element_properties_prefix_10_1(&mut trial, max_bytes, &mut refs)? {
                PrefixOutcome10_1::Complete(properties) => properties,
                PrefixOutcome10_1::Stopped(reason) => {
                    return Err(io::Error::new(ErrorKind::Unsupported, reason.describe()));
                }
            };
        PictureElementBase10_1::Legacy {
            geometry,
            properties,
        }
    } else {
        PictureElementBase10_1::Frame(base_frame(&mut trial, max_bytes, &mut refs)?)
    };
    let properties = match &base {
        PictureElementBase10_1::Legacy { properties, .. } => properties,
        PictureElementBase10_1::Frame(frame) => &frame.properties,
    };
    let embed_flag_bits = (version >= 2).then(|| trial.read_u16_le()).transpose()?;
    let embeds = embed_flag_bits == Some(0xffff) || (version <= 4 && properties.name.is_empty());
    let picture = if embeds {
        let entry = read_object_entry_10_1(&mut trial, &mut refs)?;
        let body = match entry {
            ObjectEntry10_1::NewObject {
                class_id: STDPICTURE_CLASS_ID,
                ..
            } => Some(read_std_picture_body_10_1(&mut trial, max_bytes)?),
            ObjectEntry10_1::NewObject { .. } => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("尚不支持图片元素的 IPicture 对象 {entry:?}"),
                ));
            }
            _ => None,
        };
        Some((entry, body))
    } else {
        None
    };
    let legacy_short = (version < 4).then(|| trial.read_u16_le()).transpose()?;
    let source_strings = if version >= 5 {
        Some([
            read_string_10_1(&mut trial, max_bytes)?,
            read_string_10_1(&mut trial, max_bytes)?,
            read_string_10_1(&mut trial, max_bytes)?,
        ])
    } else {
        None
    };
    let envelope_bits = if version >= 6 {
        let mut bits = [0; 4];
        for value in &mut bits {
            *value = trial.read_f64_le()?.to_bits();
        }
        Some(bits)
    } else {
        None
    };
    let string_174 = (version >= 7)
        .then(|| read_string_10_1(&mut trial, max_bytes))
        .transpose()?;
    let short_178 = (version >= 8).then(|| trial.read_u16_le()).transpose()?;
    let string_17c = (version >= 8)
        .then(|| read_string_10_1(&mut trial, max_bytes))
        .transpose()?;
    let string_19c = (version >= 9)
        .then(|| read_string_10_1(&mut trial, max_bytes))
        .transpose()?;
    *reader = trial;
    *table = refs;
    Ok(PictureElement10_1 {
        kind,
        version,
        base,
        embed_flag_bits,
        picture,
        legacy_short,
        source_strings,
        envelope_bits,
        string_174,
        short_178,
        string_17c,
        string_19c,
    })
}
