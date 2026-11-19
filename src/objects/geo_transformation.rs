//! ArcMap 10.1 多个 IGeoTransformation 实现的共同对象体。
//! 静态依据及限制见 `docs/mxd-terra-host-geo.md`；不解析文本语义。

use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// CLSID 的 MXD 流中字节序。
pub const GEO_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x40, 0x4d, 0x3c, 0x84, 0x7e, 0xbd, 0x42, 0x45, 0xa1, 0x26, 0x24, 0xae, 0xba, 0x43, 0xaf, 0x20,
];
pub const NADCON_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x1c, 0x94, 0x61, 0xd6, 0x8a, 0xda, 0xd3, 0x11, 0x9f, 0x60, 0x00, 0xc0, 0x4f, 0x6b, 0xdd, 0x7f,
];
/// Geometry.dll 0x71a2786c 的持久化虚表与前两类共用 Load / Save。
pub const COORDINATE_FRAME_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x31, 0x44, 0x02, 0x0f, 0xf8, 0xc1, 0xd2, 0x11, 0xbd, 0x07, 0x00, 0x00, 0xf8, 0x75, 0xbc, 0xce,
];

pub const GEOCENTRIC_TRANSLATION_CLASS_ID: [u8; 16] = [
    0x30, 0x44, 0x02, 0x0f, 0xf8, 0xc1, 0xd2, 0x11, 0xbd, 0x07, 0x00, 0x00, 0xf8, 0x75, 0xbc, 0xce,
];
pub const POSITION_VECTOR_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0xb1, 0x92, 0xdf, 0x0c, 0xa0, 0xc2, 0xd2, 0x11, 0xbd, 0x08, 0x00, 0x00, 0xf8, 0x75, 0xbc, 0xce,
];
pub const MOLODENSKY_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0xb0, 0x92, 0xdf, 0x0c, 0xa0, 0xc2, 0xd2, 0x11, 0xbd, 0x08, 0x00, 0x00, 0xf8, 0x75, 0xbc, 0xce,
];
pub const MOLODENSKY_BADEKAS_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x29, 0xe6, 0x39, 0xe7, 0x22, 0x0d, 0xf7, 0x48, 0x84, 0x1c, 0x54, 0x11, 0x1c, 0xa6, 0xd6, 0x66,
];
pub const NULL_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x90, 0x7b, 0x5a, 0x1c, 0x8e, 0xe6, 0xde, 0x11, 0x8a, 0x39, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];
pub const UNIT_CHANGE_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x91, 0x7b, 0x5a, 0x1c, 0x8e, 0xe6, 0xde, 0x11, 0x8a, 0x39, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];
pub const GEOGRAPHIC_2D_OFFSET_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x92, 0x7b, 0x5a, 0x1c, 0x8e, 0xe6, 0xde, 0x11, 0x8a, 0x39, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];
pub const ABRIDGED_MOLODENSKY_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0xd0, 0x68, 0x2f, 0xdd, 0xb0, 0xc6, 0xd2, 0x11, 0xbd, 0x09, 0x00, 0x00, 0xf8, 0x75, 0xbc, 0xce,
];
pub const LONGITUDE_ROTATION_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x02, 0x00, 0x3c, 0x6f, 0x7f, 0xda, 0xd3, 0x11, 0x9f, 0x60, 0x00, 0xc0, 0x4f, 0x6b, 0xdd, 0x7f,
];
pub const HARN_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x78, 0x68, 0x14, 0xdf, 0x8a, 0xda, 0xd3, 0x11, 0x9f, 0x60, 0x00, 0xc0, 0x4f, 0x6b, 0xdd, 0x7f,
];
pub const NTV2_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0xe2, 0x71, 0xb9, 0x52, 0xec, 0xeb, 0xd4, 0x11, 0x9f, 0xd6, 0x00, 0xc0, 0x4f, 0x6b, 0xdd, 0x7f,
];
pub const COMPOSITE_GEO_TRANSFORMATION_CLASS_ID: [u8; 16] = [
    0x82, 0xfb, 0x47, 0x58, 0xae, 0xdb, 0xd3, 0x11, 0x9f, 0x60, 0x00, 0xc0, 0x4f, 0x6b, 0xdd, 0x7f,
];

pub fn supports_geo_transformation_body_10_1(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        GEO_TRANSFORMATION_CLASS_ID
            | NADCON_TRANSFORMATION_CLASS_ID
            | COORDINATE_FRAME_TRANSFORMATION_CLASS_ID
            | GEOCENTRIC_TRANSLATION_CLASS_ID
            | POSITION_VECTOR_TRANSFORMATION_CLASS_ID
            | MOLODENSKY_TRANSFORMATION_CLASS_ID
            | MOLODENSKY_BADEKAS_TRANSFORMATION_CLASS_ID
            | NULL_TRANSFORMATION_CLASS_ID
            | UNIT_CHANGE_TRANSFORMATION_CLASS_ID
            | GEOGRAPHIC_2D_OFFSET_TRANSFORMATION_CLASS_ID
            | ABRIDGED_MOLODENSKY_TRANSFORMATION_CLASS_ID
            | LONGITUDE_ROTATION_TRANSFORMATION_CLASS_ID
            | HARN_TRANSFORMATION_CLASS_ID
            | NTV2_TRANSFORMATION_CLASS_ID
            | COMPOSITE_GEO_TRANSFORMATION_CLASS_ID
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoTransformationBody10_1 {
    pub version: u16,
    /// 共同文本体中的原始字节。组合变换的成员另见 composite。
    pub text: Vec<u8>,
    pub composite: Option<CompositeGeoTransformation10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeGeoTransformation10_1 {
    pub name: String,
    pub items: Vec<GeoTransformationComponent10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoTransformationComponent10_1 {
    pub direction: i16,
    pub class_id: [u8; 16],
    pub body: GeoTransformationBody10_1,
}

/// 读取已核对的坐标变换类，失败时游标不变。
/// max_bytes 和 3071 字节上限是读取器的保守限制，不代表原生校验。
pub fn read_geo_transformation_body_10_1(
    reader: &mut Reader<'_>,
    class_id: [u8; 16],
    max_bytes: usize,
) -> io::Result<GeoTransformationBody10_1> {
    let mut trial = reader.clone();
    let body = read_body(&mut trial, class_id, max_bytes, 16)?;
    *reader = trial;
    Ok(body)
}

fn read_body(
    reader: &mut Reader<'_>,
    class_id: [u8; 16],
    max_bytes: usize,
    depth: usize,
) -> io::Result<GeoTransformationBody10_1> {
    if !supports_geo_transformation_body_10_1(class_id) {
        return Err(io::Error::new(ErrorKind::InvalidData, "未知坐标变换类"));
    }
    let version = reader.read_u16_le()?;
    if class_id == COMPOSITE_GEO_TRANSFORMATION_CLASS_ID {
        if version > 1 || depth == 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "组合坐标变换版本或嵌套层数超限",
            ));
        }
        let length = usize::from(reader.read_u16_le()?);
        // 原生以有符号 short 保存字节数，长度不含终止零。
        if length > max_bytes || length > i16::MAX as usize || !length.is_multiple_of(2) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "组合坐标变换名称长度不合法",
            ));
        }
        let name = reader.read_utf16_le(length)?;
        let count = (reader.read_u16_le()? as i16).max(0) as usize;
        // 每项至少包含方向、CLSID 和六字节对象体。
        if count > reader.remaining().min(max_bytes) / 24 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "组合坐标变换成员数量超限",
            ));
        }
        let mut items = Vec::new();
        for _ in 0..count {
            let direction = reader.read_u16_le()? as i16;
            let class_id = reader.read_bytes(16)?.try_into().unwrap();
            // 原生直接委派 IPersistStream，成员没有对象键。
            let body = read_body(reader, class_id, max_bytes, depth - 1)?;
            items.push(GeoTransformationComponent10_1 {
                direction,
                class_id,
                body,
            });
        }
        return Ok(GeoTransformationBody10_1 {
            version,
            text: Vec::new(),
            composite: Some(CompositeGeoTransformation10_1 { name, items }),
        });
    }
    if version > 2 {
        return Err(io::Error::new(ErrorKind::InvalidData, "未知坐标变换版本"));
    }
    let length = usize::try_from(reader.read_u32_le()?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "坐标变换长度溢出"))?;
    if length > max_bytes || length >= 3072 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "坐标变换长度超过限额",
        ));
    }
    let text = reader.read_bytes(length)?.to_vec();
    Ok(GeoTransformationBody10_1 {
        version,
        text,
        composite: None,
    })
}
