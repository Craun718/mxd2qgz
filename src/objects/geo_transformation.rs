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

pub fn supports_geo_transformation_body_10_1(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        GEO_TRANSFORMATION_CLASS_ID
            | NADCON_TRANSFORMATION_CLASS_ID
            | COORDINATE_FRAME_TRANSFORMATION_CLASS_ID
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoTransformationBody10_1 {
    pub version: u16,
    pub text: Vec<u8>,
}

/// 仅接受静态确认共享持久化函数的类；失败时游标不变。
/// max_bytes 和 3071 字节上限是读取器的保守限制，不代表原生校验。
pub fn read_geo_transformation_body_10_1(
    reader: &mut Reader<'_>,
    class_id: [u8; 16],
    max_bytes: usize,
) -> io::Result<GeoTransformationBody10_1> {
    if !supports_geo_transformation_body_10_1(class_id) {
        return Err(io::Error::new(ErrorKind::InvalidData, "未知坐标变换类"));
    }
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    if version > 2 {
        return Err(io::Error::new(ErrorKind::InvalidData, "未知坐标变换版本"));
    }
    let length = usize::try_from(trial.read_u32_le()?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "坐标变换长度溢出"))?;
    if length > max_bytes || length >= 3072 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "坐标变换长度超过限额",
        ));
    }
    let text = trial.read_bytes(length)?.to_vec();
    *reader = trial;
    Ok(GeoTransformationBody10_1 { version, text })
}
