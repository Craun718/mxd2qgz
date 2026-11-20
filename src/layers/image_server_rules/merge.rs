//! FunctionRasterDB.dll 的合并栅格函数和参数压缩体。

use super::RuleSlot10_1;
use super::raster_info::{self, RasterInfo10_1};
use super::template::{RasterTemplateInput10_1, read_input_body};
use crate::objects::property_set::{PropertyValueObjectStream10_1, read_variant_object_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use crate::stream::zlib::decompress_exact;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub enum MergeRastersPayload10_1 {
    /// 空 IRasterInfo2 时，可以直接定位后继的四 double 项。
    Rectangles(Vec<[f64; 4]>),
    RasterInfo {
        raster_info: RuleSlot10_1<RasterInfo10_1>,
        rectangles: Vec<[f64; 4]>,
    },
    /// 非空 IRasterInfo2 的体和后继数据保留在内层流的确定边界内。
    UnresolvedRasterInfo(MergeRasterInput10_1),
}

#[derive(Debug, Clone)]
pub struct MergeRasterInput10_1 {
    pub entry: ObjectEntry10_1,
    /// 对象头之后的全部内层原字节。
    pub remaining: Vec<u8>,
    /// 完整识别的参数对象体。
    pub decoded: Option<Box<DecodedMergeRasterInput10_1>>,
}

impl PartialEq for MergeRasterInput10_1 {
    fn eq(&self, other: &Self) -> bool {
        // 保持按解压后的原始编码比较，包括浮点值的位模式。
        self.entry == other.entry && self.remaining == other.remaining
    }
}

impl Eq for MergeRasterInput10_1 {}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedMergeRasterInput10_1 {
    Object(PropertyValueObjectStream10_1),
    RasterInput(RasterTemplateInput10_1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeRastersArguments10_1 {
    pub version: u16,
    /// 依次为 Rasters 和 Footprints。
    pub inputs: [MergeRasterInput10_1; 2],
}

pub(super) fn read_payload(
    r: &mut Reader<'_>,
    max: usize,
    outer_refs: &RefTable10_1,
    depth: usize,
) -> io::Result<MergeRastersPayload10_1> {
    let data = read_compressed_variant(r, max)?;
    let mut inner = Reader::new(&data);
    let mut refs = outer_refs.isolated_raster_stream();
    match raster_info::read_slot(&mut inner, max, &mut refs, depth) {
        Ok(info) if info.entry == ObjectEntry10_1::Null => {
            read_rectangles(&mut inner).map(MergeRastersPayload10_1::Rectangles)
        }
        Ok(raster_info) => Ok(MergeRastersPayload10_1::RasterInfo {
            raster_info,
            rectangles: read_rectangles(&mut inner)?,
        }),
        Err(error) if error.kind() == ErrorKind::Unsupported => {
            let mut inner = Reader::new(&data);
            let entry = read_object_entry_10_1(&mut inner, &mut RefTable10_1::new())?;
            Ok(MergeRastersPayload10_1::UnresolvedRasterInfo(
                MergeRasterInput10_1 {
                    entry,
                    remaining: inner.read_bytes(inner.remaining())?.to_vec(),
                    decoded: None,
                },
            ))
        }
        Err(error) => Err(error),
    }
}

fn read_rectangles(inner: &mut Reader<'_>) -> io::Result<Vec<[f64; 4]>> {
    let count = usize::try_from(inner.read_u32_le()? as i32)
        .map_err(|_| invalid("合并栅格矩形数量为负"))?;
    let payload_size = count
        .checked_mul(32)
        .ok_or_else(|| invalid("合并栅格矩形长度溢出"))?;
    if payload_size != inner.remaining() {
        return Err(invalid("合并栅格矩形数量与解压后长度不匹配"));
    }
    let mut rectangles = Vec::with_capacity(count);
    for _ in 0..count {
        rectangles.push([
            inner.read_f64_le()?,
            inner.read_f64_le()?,
            inner.read_f64_le()?,
            inner.read_f64_le()?,
        ]);
    }
    Ok(rectangles)
}

pub(super) fn read_arguments_version(r: &mut Reader<'_>) -> io::Result<u16> {
    let version = r.read_u16_le()?;
    if version as i16 > 2 {
        return Err(super::unsupported(format!(
            "MergeRastersFunctionArguments 未支持版本 {version}"
        )));
    }
    Ok(version)
}

pub(super) fn read_arguments(
    r: &mut Reader<'_>,
    max: usize,
    outer_refs: &RefTable10_1,
    version: u16,
    depth: usize,
) -> io::Result<MergeRastersArguments10_1> {
    let depth = depth
        .checked_sub(1)
        .ok_or_else(|| invalid("合并栅格参数对象嵌套超过限额"))?;
    Ok(MergeRastersArguments10_1 {
        version,
        inputs: [
            read_input(r, max, outer_refs, depth)?,
            read_input(r, max, outer_refs, depth)?,
        ],
    })
}

fn read_input(
    r: &mut Reader<'_>,
    max: usize,
    outer_refs: &RefTable10_1,
    depth: usize,
) -> io::Result<MergeRasterInput10_1> {
    let data = read_compressed_variant(r, max)?;
    let mut inner = Reader::new(&data);
    // 每段解压数据由独立 ObjectStream 读取，键与宿主以及另一段隔离。
    let mut refs = outer_refs.isolated_raster_stream();
    let entry = read_object_entry_10_1(&mut inner, &mut refs)?;
    let remaining = inner.read_bytes(inner.remaining())?.to_vec();
    if entry == ObjectEntry10_1::Null && !remaining.is_empty() {
        return Err(invalid("空合并栅格参数对象之后仍有内层字节"));
    }
    let decoded = decode_input(&remaining, max, &mut refs, entry, depth)?.map(Box::new);
    Ok(MergeRasterInput10_1 {
        entry,
        remaining,
        decoded,
    })
}

fn decode_input(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
    entry: ObjectEntry10_1,
    depth: usize,
) -> io::Result<Option<DecodedMergeRasterInput10_1>> {
    let ObjectEntry10_1::NewObject { class_id, key } = entry else {
        return Ok(None);
    };
    let mut body = Reader::new(bytes);
    let object = read_variant_object_body_10_1(&mut body, max, &mut refs.clone(), 0, entry);
    let decoded = match object {
        Ok(value) => DecodedMergeRasterInput10_1::Object(value),
        Err(error) if error.kind() == ErrorKind::Unsupported => {
            body = Reader::new(bytes);
            match read_input_body(&mut body, max, refs, class_id, key, depth) {
                Ok(value) => DecodedMergeRasterInput10_1::RasterInput(value),
                Err(error) if error.kind() == ErrorKind::Unsupported => return Ok(None),
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    };
    if body.remaining() != 0 {
        return Err(invalid("已识别的合并栅格参数对象体之后仍有内层字节"));
    }
    Ok(Some(decoded))
}

fn read_compressed_variant(r: &mut Reader<'_>, max: usize) -> io::Result<Vec<u8>> {
    let vartype = r.read_u16_le()?;
    if vartype != 0x2011 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("合并栅格压缩 Variant 类型未支持 0x{vartype:04x}"),
        ));
    }
    r.read_bytes(14)?; // Variant::Read(false) 的保留区。
    let array_size = usize::try_from(r.read_u32_le()? as i32)
        .map_err(|_| invalid("合并栅格压缩数组长度为负"))?;
    if !(8..=max).contains(&array_size) {
        return Err(invalid("合并栅格压缩数组长度超限或不足"));
    }
    let mut wrapper = Reader::new(r.read_bytes(array_size)?);
    let raw_size = usize::try_from(wrapper.read_u32_le()? as i32)
        .map_err(|_| invalid("合并栅格解压长度为负"))?;
    let compressed_size = usize::try_from(wrapper.read_u32_le()? as i32)
        .map_err(|_| invalid("合并栅格压缩长度为负"))?;
    if raw_size > max || compressed_size != wrapper.remaining() {
        return Err(invalid("合并栅格压缩头的长度无效"));
    }
    let compressed = wrapper.read_bytes(compressed_size)?;
    decompress_exact(compressed, raw_size)
        .map_err(|error| invalid(format!("合并栅格 zlib 解压失败：{error}")))
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message.into())
}
