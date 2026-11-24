//! GdbRasterCatalogLayer 的 ArcMap 10.1 IPersistStream 读取。
use super::feature_layer::{
    FeatureLayerPrefix10_1, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
};
use super::feature_layer_tail::SuccessorStop10_1;
use super::raster_layer_tail::{RasterLayerExtraSafeExtension10_1, read_extra_safe_extensions};
use super::raster_renderer::{RasterRendererPrefix10_1, read_raster_renderer_body_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub(crate) const GDB_RASTER_CATALOG_LAYER_CLASS_ID: [u8; 16] = [
    0x6d, 0x79, 0x7d, 0x29, 0x4e, 0x38, 0xf8, 0x48, 0xa1, 0x89, 0xe0, 0x34, 0x72, 0x81, 0xac, 0x7d,
];

#[derive(Debug, Clone, PartialEq)]
pub enum RasterCatalogRecord10_1 {
    Long(i32),
    Short(i16),
    Double(f64),
    String(String),
    SpatialReference(SpatialReferenceEntry10_1),
    Renderer {
        class_id: [u8; 16],
        body: Box<RasterRendererPrefix10_1>,
    },
    NullObject,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterCatalogLayer10_1 {
    pub version: i32,
    pub records: Vec<(i32, RasterCatalogRecord10_1)>,
    pub tail_short: Option<i16>,
    pub tail_long: Option<i32>,
    pub extensions: Vec<RasterLayerExtraSafeExtension10_1>,
    pub feature: FeatureLayerPrefix10_1,
}

pub(crate) fn read_raster_catalog_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterCatalogLayer10_1> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u32_le()? as i32;
    if !(0..=4).contains(&version) {
        return Err(invalid("栅格目录图层版本超出 0–4"));
    }
    let start = r.position();
    if r.read_u32_le()? != 0xff00ff || r.read_u32_le()? != 0 {
        return Err(invalid("栅格目录缺少 RasterIO 记录头"));
    }
    let mut records = Vec::new();
    let mut last_tag = 0;
    let mut renderer_end = 30;
    let mut selector = 0;
    loop {
        let tag = r.read_u32_le()? as i32;
        let length = r.read_u32_le()? as i32;
        if r.position() - start > max {
            return Err(invalid("栅格目录 RasterIO 长度超过限额"));
        }
        if (tag, length) == (-1, -1) {
            break;
        }
        if tag <= last_tag {
            return Err(invalid("栅格目录 RasterIO 标签顺序错误"));
        }
        last_tag = tag;
        let selector_tag = renderer_end + 1;
        let value = match (tag, length) {
            (2, -1) => {
                let class_id = r.read_bytes(16)?.try_into().unwrap();
                if class_id == [0; 16] {
                    RasterCatalogRecord10_1::NullObject
                } else {
                    // RasterIO 外层不登记对象键；空间参考体不使用该临时键。
                    let entry = ObjectEntry10_1::NewObject { class_id, key: 0 };
                    let body = read_spatial_reference_body_10_1(&mut r, max, &entry)?
                        .ok_or_else(|| unsupported("栅格目录 RasterIO 空间参考类未支持"))?;
                    RasterCatalogRecord10_1::SpatialReference(body)
                }
            }
            (31.., -1) if tag <= renderer_end => {
                let class_id = r.read_bytes(16)?.try_into().unwrap();
                if class_id == [0; 16] {
                    RasterCatalogRecord10_1::NullObject
                } else {
                    let body = read_raster_renderer_body_10_1(&mut r, max, &mut refs, class_id)?;
                    if !body.is_complete() {
                        return Err(unsupported("栅格目录 RasterIO 渲染器体未读完"));
                    }
                    RasterCatalogRecord10_1::Renderer {
                        class_id,
                        body: Box::new(body),
                    }
                }
            }
            (1 | 3..=5 | 7..=9 | 30, 4) => {
                let value = r.read_u32_le()? as i32;
                if tag == 30 {
                    if !(0..=i32::MAX - 40).contains(&value) {
                        return Err(invalid("栅格目录渲染器计数导致标签溢出"));
                    }
                    renderer_end = 30 + value;
                }
                RasterCatalogRecord10_1::Long(value)
            }
            (6, 8) => RasterCatalogRecord10_1::Double(r.read_f64_le()?),
            (10, 2) => RasterCatalogRecord10_1::Short(r.read_u16_le()? as i16),
            (11, 0..) => RasterCatalogRecord10_1::String(read_raster_string(&mut r, length, max)?),
            (_, 4) if version >= 2 && tag == selector_tag => {
                selector = r.read_u32_le()? as i32;
                RasterCatalogRecord10_1::Long(selector)
            }
            (_, 4)
                if selector == 1
                    && (1..=if version >= 3 { 8 } else { 4 }).contains(&(tag - selector_tag)) =>
            {
                RasterCatalogRecord10_1::Long(r.read_u32_le()? as i32)
            }
            (_, 0..) if selector == 1 && version >= 3 && tag == selector_tag + 9 => {
                RasterCatalogRecord10_1::String(read_raster_string(&mut r, length, max)?)
            }
            // 保存端也可能给旧版写出固定的关闭选择值。
            (_, 4) if version < 2 && tag == selector_tag => {
                RasterCatalogRecord10_1::Long(r.read_u32_le()? as i32)
            }
            _ => {
                return Err(unsupported(&format!(
                    "栅格目录 RasterIO 未支持的标签 {tag} 或长度 {length}"
                )));
            }
        };
        records.push((tag, value));
    }
    for tag in [1, 3, 8, 30] {
        if records
            .binary_search_by_key(&tag, |record| record.0)
            .is_err()
        {
            return Err(invalid("栅格目录 RasterIO 缺少必要标量记录"));
        }
    }
    let tail_short = (version >= 3)
        .then(|| r.read_u16_le().map(|v| v as i16))
        .transpose()?;
    let tail_long = (version >= 4)
        .then(|| r.read_u32_le().map(|v| v as i32))
        .transpose()?;
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    let extensions = read_extra_safe_extensions(&mut r, &mut refs, count, max)?;
    let feature = read_feature_layer_prefix_10_1(&mut r, max, &mut refs)?;
    if !matches!(
        feature.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted)
    ) {
        return Err(unsupported("栅格目录聚合 FeatureLayer 体未读完"));
    }
    *reader = r;
    *table = refs;
    Ok(RasterCatalogLayer10_1 {
        version,
        records,
        tail_short,
        tail_long,
        extensions,
        feature,
    })
}

fn read_raster_string(r: &mut Reader<'_>, length: i32, max: usize) -> io::Result<String> {
    let length = length as usize;
    if length > max {
        return Err(invalid("栅格目录 RasterIO 字符串超过限额"));
    }
    // RasterIO 的四字节长度已由记录头读取；负载是带终止零的宽字符串。
    let raw = r.read_bytes(length)?;
    let mut bytes = (length as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(raw);
    read_string_10_1(&mut Reader::new(&bytes), max)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

fn unsupported(message: &str) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use crate::layers::layer_list::LayerBodyPrefix10_1;
    use crate::objects::spatial_reference_slots::UNKNOWN_COORDINATE_SYSTEM_CLASS_ID;

    fn long_record(bytes: &mut Vec<u8>, tag: i32, value: i32) {
        bytes.extend(tag.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        bytes.extend(value.to_le_bytes());
    }

    #[test]
    fn scene_raster_catalog_reads_sparse_records_and_aggregate() {
        for version in 0_i32..=4 {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 8]);
            bytes.extend(GDB_RASTER_CATALOG_LAYER_CLASS_ID);
            bytes.extend(41_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend(0xff00ff_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            long_record(&mut bytes, 1, 11);
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend(UNKNOWN_COORDINATE_SYSTEM_CLASS_ID);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(4));
            long_record(&mut bytes, 3, 33);
            long_record(&mut bytes, 8, 88);
            long_record(&mut bytes, 30, 2);
            bytes.extend(32_i32.to_le_bytes()); // 第一条渲染器记录可省略。
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend([0; 16]);
            long_record(&mut bytes, 33, i32::from(version >= 3));
            if version >= 3 {
                long_record(&mut bytes, 41, 123);
                bytes.extend(42_i32.to_le_bytes());
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0xff; 8]);
            if version >= 3 {
                bytes.extend(9_u16.to_le_bytes());
            }
            if version >= 4 {
                bytes.extend(10_u32.to_le_bytes());
            }
            bytes.extend(0_i32.to_le_bytes());
            let aggregate_at = bytes.len();
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(2));
            bytes.extend([0; 6 + 16 + 16 + 16]);
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 16 + 2 + 4 + 16]);
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(2));
            bytes.extend([0; 8 + 16 + 2 + 16 + 4 + 2]);
            bytes.extend([0; 16 + 4]); // 容器的空包络和元素计数。
            let end = bytes.len();
            bytes.extend([0xa5; 16]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let scene = read_graphics_3d_layer_10_1(
                &GRAPHICS_LAYER_3D_CLASS_ID,
                &mut reader,
                4096,
                &mut refs,
            )
            .unwrap();
            let Some(LayerBodyPrefix10_1::RasterCatalogLayer(body)) =
                scene.associated_layer_body.as_deref()
            else {
                panic!("缺少栅格目录关联图层");
            };
            assert_eq!(body.version, version);
            assert_eq!(body.tail_short, (version >= 3).then_some(9));
            assert_eq!(body.tail_long, (version >= 4).then_some(10));
            assert!(matches!(
                body.records[1],
                (
                    2,
                    RasterCatalogRecord10_1::SpatialReference(
                        SpatialReferenceEntry10_1::UnknownCoordinateSystem { .. }
                    )
                )
            ));
            assert_eq!(body.records[5], (32, RasterCatalogRecord10_1::NullObject));
            if version >= 3 {
                assert_eq!(
                    body.records.last(),
                    Some(&(42, RasterCatalogRecord10_1::String(String::new())))
                );
            }
            assert_eq!(refs.len(), 1);
            assert_eq!(reader.position(), end);

            let mut reader = Reader::new(&bytes[..aggregate_at + 1]);
            let mut refs = RefTable10_1::new();
            assert!(
                read_graphics_3d_layer_10_1(
                    &GRAPHICS_LAYER_3D_CLASS_ID,
                    &mut reader,
                    4096,
                    &mut refs
                )
                .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}
