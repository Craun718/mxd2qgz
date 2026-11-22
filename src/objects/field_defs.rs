//! GdbCore GeometryDef 与 RasterCatalog RasterDef 的 10.1 对象体。

use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const GEOMETRY_DEF_CLASS_ID: [u8; 16] = [
    0x52, 0x0d, 0x9a, 0x43, 0x15, 0x39, 0xd1, 0x11, 0x9c, 0xa7, 0x00, 0x00, 0xf8, 0x78, 0x06, 0x19,
];
pub const RASTER_DEF_CLASS_ID: [u8; 16] = [
    0x92, 0x61, 0x38, 0xa8, 0x59, 0x36, 0x25, 0x45, 0x98, 0x4f, 0x5d, 0x64, 0x3a, 0x40, 0xee, 0x8c,
];

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinitionSlot10_1<T> {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryDef10_1 {
    pub version: u16,
    pub geometry_type: i32,
    pub grid_sizes: Vec<f64>,
    pub long_14: i32,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub shorts_1c_1e: [u16; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterDef10_1 {
    pub version: u16,
    pub string_c: String,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub shorts_14_16: [u16; 2],
    pub short_18: Option<u16>,
}

pub(crate) fn read_geometry_def_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FieldDefinitionSlot10_1<GeometryDef10_1>> {
    read_definition_slot(
        reader,
        max_bytes,
        refs,
        GEOMETRY_DEF_CLASS_ID,
        "IGeometryDef",
        read_geometry_def,
    )
}

pub(crate) fn read_raster_def_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FieldDefinitionSlot10_1<RasterDef10_1>> {
    read_definition_slot(
        reader,
        max_bytes,
        refs,
        RASTER_DEF_CLASS_ID,
        "IRasterDef",
        read_raster_def,
    )
}

fn read_definition_slot<T>(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    expected_class: [u8; 16],
    interface: &str,
    read_body: impl FnOnce(&mut Reader<'_>, usize, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<FieldDefinitionSlot10_1<T>> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let mut entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = staged.registered_class(*key)
    {
        if registered != expected_class {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("{interface} 回引键 {key} 的登记类不受支持"),
            ));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected_class => {
            Some(Box::new(read_body(&mut trial, max_bytes, &mut staged)?))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "{interface} 未支持的新对象类 {}",
                    crate::storage::class_id_hex(&class_id)
                ),
            ));
        }
    };
    *reader = trial;
    *refs = staged;
    Ok(FieldDefinitionSlot10_1 { entry, body })
}

fn version(r: &mut Reader<'_>, max: u16, label: &str) -> io::Result<u16> {
    let version = r.read_u16_le()?;
    if version > max {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{label} 未支持版本 {version}"),
        ));
    }
    Ok(version)
}

pub(crate) fn read_geometry_def(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<GeometryDef10_1> {
    let version = version(r, 1, "GeometryDef")?;
    let geometry_type = r.read_u32_le()? as i32;
    // 原生网格分配器将非正计数归零。
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    if count > r.remaining() / 8 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "GeometryDef 网格数量超出可读范围",
        ));
    }
    let grid_sizes = (0..count)
        .map(|_| r.read_f64_le())
        .collect::<io::Result<_>>()?;
    let long_14 = r.read_u32_le()? as i32;
    let spatial_reference = read_spatial_reference_entry_10_1(r, max_bytes, refs)?;
    let shorts_1c_1e = [r.read_u16_le()?, r.read_u16_le()?];
    Ok(GeometryDef10_1 {
        version,
        geometry_type,
        grid_sizes,
        long_14,
        spatial_reference,
        shorts_1c_1e,
    })
}

pub(crate) fn read_raster_def(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterDef10_1> {
    let version = r.read_u16_le()?;
    if (version as i16) > 2 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("RasterDef 未支持版本 {version}"),
        ));
    }
    let string_c = read_string_10_1(r, max_bytes)?;
    let spatial_reference = read_spatial_reference_entry_10_1(r, max_bytes, refs)?;
    let shorts_14_16 = [r.read_u16_le()?, r.read_u16_le()?];
    let short_18 = ((version as i16) >= 2)
        .then(|| r.read_u16_le())
        .transpose()?;
    Ok(RasterDef10_1 {
        version,
        string_c,
        spatial_reference,
        shorts_14_16,
        short_18,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_references_check_the_class_registered_for_each_interface() {
        for expected in [GEOMETRY_DEF_CLASS_ID, RASTER_DEF_CLASS_ID] {
            for registered in [expected, [0x7d; 16]] {
                let mut header = registered.to_vec();
                header.extend(41_u32.to_le_bytes());
                let mut refs = RefTable10_1::new();
                read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
                let before = refs.clone();
                let claimed = if registered == expected {
                    [0x7d; 16]
                } else {
                    expected
                };
                header[..16].copy_from_slice(&claimed);
                header.push(0xaa);
                let mut reader = Reader::new(&header);
                let result = if expected == GEOMETRY_DEF_CLASS_ID {
                    read_geometry_def_slot_10_1(&mut reader, 128, &mut refs).map(|slot| slot.entry)
                } else {
                    read_raster_def_slot_10_1(&mut reader, 128, &mut refs).map(|slot| slot.entry)
                };
                assert_eq!(refs, before);
                if registered == expected {
                    assert_eq!(
                        result.unwrap(),
                        ObjectEntry10_1::BackReference {
                            class_id: expected,
                            key: 41,
                        }
                    );
                    assert_eq!(reader.position(), 20);
                } else {
                    assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                    assert_eq!(reader.position(), 0);
                }
            }
        }
    }

    #[test]
    fn geometry_def_reads_following_fields_with_nonpositive_grid_counts() {
        for count in [i32::MIN, -1, 0] {
            let mut bytes = GEOMETRY_DEF_CLASS_ID.to_vec();
            bytes.extend(21_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(count.to_le_bytes());
            bytes.extend(19_i32.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            let end = bytes.len();
            bytes.push(0xaa);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let body = read_geometry_def_slot_10_1(&mut reader, 128, &mut refs)
                .unwrap()
                .body
                .unwrap();
            assert!(body.grid_sizes.is_empty());
            assert_eq!(body.long_14, 19);
            assert_eq!(body.shorts_1c_1e, [6, 7]);
            assert!(refs.contains(21));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn typed_definition_bodies_can_be_read_from_variant_slots() {
        use crate::objects::property_set::{
            PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
        };

        for class_id in [GEOMETRY_DEF_CLASS_ID, RASTER_DEF_CLASS_ID] {
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            bytes.extend([0; 14]);
            bytes.extend(class_id);
            bytes.extend(25_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            if class_id == GEOMETRY_DEF_CLASS_ID {
                bytes.extend(3_i32.to_le_bytes());
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(16_f64.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
            } else {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 16]);
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            let end = bytes.len();
            bytes.push(0xa1);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value =
                read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
            match value {
                PropertyValueObjectStream10_1::GeometryDef(encoded) => {
                    assert_eq!(encoded.key, 25);
                    let body = encoded.body();
                    assert_eq!(class_id, GEOMETRY_DEF_CLASS_ID);
                    assert_eq!(body.grid_sizes, [16.0]);
                    assert_eq!(body.shorts_1c_1e, [6, 7]);
                }
                PropertyValueObjectStream10_1::RasterDef(encoded) => {
                    assert_eq!(encoded.key, 25);
                    let body = encoded.body();
                    assert_eq!(class_id, RASTER_DEF_CLASS_ID);
                    assert_eq!(body.shorts_14_16, [6, 7]);
                    assert_eq!(body.short_18, None);
                }
                other => panic!("定义对象分派错误：{other:?}"),
            }
            assert!(refs.contains(25));
            assert_eq!(reader.read_bytes(1).unwrap(), [0xa1]);

            let mut reader = Reader::new(&bytes[..end - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn raster_def_only_reads_the_third_short_from_version_two() {
        for version in [i16::MIN as u16, u16::MAX, 0, 1, 2] {
            let mut bytes = RASTER_DEF_CLASS_ID.to_vec();
            bytes.extend(25_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0; 16]); // 空空间参考。
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            if version == 2 {
                bytes.extend(8_u16.to_le_bytes());
            }
            bytes.push(0xa1);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let slot = read_raster_def_slot_10_1(&mut reader, 64, &mut refs).unwrap();
            let body = slot.body.unwrap();
            assert_eq!(body.version, version);
            assert_eq!(body.shorts_14_16, [6, 7]);
            assert_eq!(body.short_18, (version == 2).then_some(8));
            assert_eq!(reader.read_bytes(1).unwrap(), [0xa1]);
            assert!(refs.contains(25));
        }
    }
}
