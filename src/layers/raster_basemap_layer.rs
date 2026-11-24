//! ArcMap 10.1 RasterBasemapLayer 的对象体。
use super::basemap_layer::ARRAY_OF_I_SHADER_CLASS_ID;
use super::layer_list::{
    LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_complete_layer_body_10_1,
};
use super::raster_layer::read_bounded_raster_body_10_1;
use super::wcs_layer::{WCS_LAYER_CLASS_ID, read_wcs_layer_body_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub(crate) const RASTER_BASEMAP_LAYER_CLASS_ID: [u8; 16] = [
    0x61, 0x02, 0x52, 0x57, 0x08, 0x26, 0x0b, 0x43, 0x90, 0x4e, 0x7b, 0x0d, 0x48, 0xc5, 0x78, 0xd5,
];
const MOSAIC_LAYER_CLASS_ID: [u8; 16] = [
    0xb8, 0xfc, 0x52, 0x34, 0x0c, 0xd1, 0xe4, 0x44, 0xb9, 0x71, 0x8c, 0x53, 0xdb, 0x39, 0xd5, 0x17,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterBasemapLayer10_1 {
    pub version: u16,
    pub short: i16,
    pub guid: [u8; 16],
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub shaders: ObjectEntry10_1,
    pub shader_array_version: Option<u16>,
    pub shorts: Option<[i16; 2]>,
    pub raster: ObjectEntry10_1,
    pub raster_body: Option<Box<LayerBodyPrefix10_1>>,
    pub name: Option<String>,
    pub mosaic: Option<ObjectEntry10_1>,
    pub mosaic_body: Option<Vec<u8>>,
}

pub(crate) fn read_raster_basemap_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterBasemapLayer10_1> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()?;
    if version > 5 {
        return Err(invalid("RasterBasemapLayer 版本超出 0–5"));
    }
    let short = r.read_u16_le()? as i16;
    let guid = r.read_bytes(16)?.try_into().unwrap();
    let spatial_reference = read_spatial_reference_entry_10_1(&mut r, max, &mut refs)?;
    let shaders = read_slot(&mut r, &mut refs, &[ARRAY_OF_I_SHADER_CLASS_ID])?;
    let shader_array_version = if matches!(shaders, ObjectEntry10_1::NewObject { .. }) {
        let version = r.read_u16_le()?;
        if version > 1 {
            return Err(invalid("RasterBasemapLayer 着色器数组版本超限"));
        }
        if r.read_u32_le()? as i32 > 0 {
            return Err(unsupported("RasterBasemapLayer 非空着色器数组未支持"));
        }
        Some(version)
    } else {
        None
    };
    let shorts = (version != 2)
        .then(|| Ok::<_, io::Error>([r.read_u16_le()? as i16, r.read_u16_le()? as i16]))
        .transpose()?;
    let mut region = read_region(&mut r, max)?;
    let raster = if region.remaining() == 0 {
        ObjectEntry10_1::Null
    } else {
        use super::image_server_layer::IMAGE_SERVER_LAYER_CLASS_ID;
        read_slot(
            &mut region,
            &mut refs,
            &[
                RASTER_LAYER_CLASS_ID,
                WCS_LAYER_CLASS_ID,
                IMAGE_SERVER_LAYER_CLASS_ID,
            ],
        )?
    };
    let raster_body = match raster {
        ObjectEntry10_1::NewObject {
            class_id: RASTER_LAYER_CLASS_ID,
            ..
        } => {
            let body = read_bounded_raster_body_10_1(&mut region, max, &mut refs)?;
            if body.tail.is_none() {
                return Err(unsupported("RasterBasemapLayer 的 RasterLayer 体未读完"));
            }
            Some(Box::new(LayerBodyPrefix10_1::RasterLayer(body)))
        }
        ObjectEntry10_1::NewObject {
            class_id: WCS_LAYER_CLASS_ID,
            ..
        } => {
            let body = read_wcs_layer_body_10_1(&mut region, max, &mut refs, true)?;
            Some(Box::new(LayerBodyPrefix10_1::WcsLayer(Box::new(body))))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => Some(Box::new(
            read_complete_layer_body_10_1(&mut region, class_id, max, &mut refs)?,
        )),
        _ => None,
    };
    if region.remaining() != 0 {
        return Err(invalid("RasterBasemapLayer 的栅格条目消费与声明长度不一致"));
    }
    let name = (version >= 4)
        .then(|| read_string_10_1(&mut r, max))
        .transpose()?;
    let (mosaic, mosaic_body) = if version >= 5 {
        let mut region = read_region(&mut r, max)?;
        let entry = if region.remaining() == 0 {
            ObjectEntry10_1::Null
        } else {
            read_slot(&mut region, &mut refs, &[MOSAIC_LAYER_CLASS_ID])?
        };
        let body = if matches!(entry, ObjectEntry10_1::NewObject { .. }) {
            Some(region.read_bytes(region.remaining())?.to_vec())
        } else {
            if region.remaining() != 0 {
                return Err(invalid("RasterBasemapLayer 的镶嵌图层空槽或回引有多余字节"));
            }
            None
        };
        (Some(entry), body)
    } else {
        (None, None)
    };
    *reader = r;
    *table = refs;
    Ok(RasterBasemapLayer10_1 {
        version,
        short,
        guid,
        spatial_reference,
        shaders,
        shader_array_version,
        shorts,
        raster,
        raster_body,
        name,
        mosaic,
        mosaic_body,
    })
}

fn read_region<'a>(reader: &mut Reader<'a>, max: usize) -> io::Result<Reader<'a>> {
    let a = reader.read_u32_le()?;
    let b = reader.read_u32_le()?;
    let length = if b == 0 { a } else { b } as usize;
    if length > max {
        return Err(invalid("RasterBasemapLayer 的 ExtraSafe 条目超过限额"));
    }
    reader.read_region(length)
}

fn read_slot(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    classes: &[[u8; 16]],
) -> io::Result<ObjectEntry10_1> {
    let mut entry = read_object_entry_10_1(reader, refs)?;
    match &mut entry {
        ObjectEntry10_1::BackReference { class_id, key } => {
            if let Some(registered) = refs.registered_class(*key) {
                *class_id = registered;
            }
            if !classes.contains(class_id) {
                return Err(unsupported("RasterBasemapLayer 回引的登记类未支持"));
            }
        }
        ObjectEntry10_1::NewObject { class_id, .. } if !classes.contains(class_id) => {
            return Err(unsupported("RasterBasemapLayer 对象类未支持"));
        }
        _ => {}
    }
    Ok(entry)
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

    #[test]
    fn scene_reads_raster_basemap_versions_with_bounded_slots() {
        for version in 0_u16..=5 {
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 8]);
            bytes.extend(RASTER_BASEMAP_LAYER_CLASS_ID);
            bytes.extend(40_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend(17_u16.to_le_bytes());
            bytes.extend([0x45; 16]);
            bytes.extend([0; 16]); // 空空间参考。
            bytes.extend(ARRAY_OF_I_SHADER_CLASS_ID);
            bytes.extend(41_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            if version != 2 {
                bytes.extend([7, 0, 8, 0]);
            }
            bytes.extend(20_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(RASTER_LAYER_CLASS_ID);
            bytes.extend(11_u32.to_le_bytes());
            if version >= 4 {
                bytes.extend([4, 0, 0, 0, b'R', 0, 0, 0]);
            }
            if version >= 5 {
                bytes.extend(23_u32.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
                bytes.extend(MOSAIC_LAYER_CLASS_ID);
                bytes.extend(42_u32.to_le_bytes());
                bytes.extend([1, 2, 3]);
            }
            let layer_end = bytes.len();
            bytes.extend([0; 16 + 4]); // 容器的空包络和元素计数。
            let end = bytes.len();
            bytes.extend([0xa5; 16]);
            let mut previous = RASTER_LAYER_CLASS_ID.to_vec();
            previous.extend(11_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
            let before = refs.clone();
            let mut reader = Reader::new(&bytes);
            let scene = read_graphics_3d_layer_10_1(
                &GRAPHICS_LAYER_3D_CLASS_ID,
                &mut reader,
                4096,
                &mut refs,
            )
            .unwrap();
            let Some(LayerBodyPrefix10_1::RasterBasemapLayer(layer)) =
                scene.associated_layer_body.as_deref()
            else {
                panic!("缺少栅格底图关联体");
            };
            assert_eq!(layer.version, version);
            assert_eq!(layer.shader_array_version, Some(1));
            assert_eq!(layer.shorts, (version != 2).then_some([7, 8]));
            assert_eq!(layer.name.as_deref(), (version >= 4).then_some("R"));
            assert_eq!(
                layer.mosaic_body.as_deref(),
                (version >= 5).then_some([1, 2, 3].as_slice())
            );
            assert!(matches!(
                layer.raster,
                ObjectEntry10_1::BackReference { key: 11, .. }
            ));
            assert_eq!(reader.position(), end);

            let mut refs = before.clone();
            let mut reader = Reader::new(&bytes[..layer_end - 1]);
            assert!(
                read_graphics_3d_layer_10_1(
                    &GRAPHICS_LAYER_3D_CLASS_ID,
                    &mut reader,
                    4096,
                    &mut refs
                )
                .is_err()
            );
            assert_eq!(refs, before);
            assert_eq!(reader.position(), 0);
        }
    }
}
