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
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
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

fn read_geometry_def(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<GeometryDef10_1> {
    let version = version(r, 1, "GeometryDef")?;
    let geometry_type = r.read_u32_le()? as i32;
    let count = usize::try_from(r.read_u32_le()? as i32)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "GeometryDef 网格数量为负"))?;
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

fn read_raster_def(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterDef10_1> {
    let version = version(r, 2, "RasterDef")?;
    let string_c = read_string_10_1(r, max_bytes)?;
    let spatial_reference = read_spatial_reference_entry_10_1(r, max_bytes, refs)?;
    let shorts_14_16 = [r.read_u16_le()?, r.read_u16_le()?];
    let short_18 = (version >= 2).then(|| r.read_u16_le()).transpose()?;
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
    fn raster_def_only_reads_the_third_short_from_version_two() {
        for version in [1_u16, 2] {
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
            assert_eq!(body.shorts_14_16, [6, 7]);
            assert_eq!(body.short_18, (version == 2).then_some(8));
            assert_eq!(reader.read_bytes(1).unwrap(), [0xa1]);
            assert!(refs.contains(25));
        }
    }
}
