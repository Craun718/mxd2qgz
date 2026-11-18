//! ArcMap 10.1 IGeodataXform 的 Identity、Polynomial 与 Composite 体。
//! 取证见 docs/mxd-10.1-geodata-xform.md。

use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const IDENTITY_XFORM_CLASS_ID: [u8; 16] = [
    0x2e, 0x9a, 0x70, 0xc4, 0x9e, 0x29, 0x09, 0x46, 0x99, 0x04, 0x6c, 0x59, 0x53, 0x19, 0xb3, 0x0f,
];
pub const POLYNOMIAL_XFORM_CLASS_ID: [u8; 16] = [
    0x49, 0xa8, 0xca, 0xb3, 0x23, 0x55, 0x90, 0x44, 0xaa, 0x2d, 0x78, 0x0a, 0xca, 0x41, 0xe1, 0xdd,
];
pub const COMPOSITE_XFORM_CLASS_ID: [u8; 16] = [
    0xbb, 0x3e, 0x92, 0x44, 0x88, 0xd9, 0x47, 0x48, 0x9b, 0x29, 0x11, 0xaa, 0x8e, 0x6e, 0x13, 0x2c,
];

const MAX_XFORM_NESTING: usize = 16;

#[derive(Clone, Debug, PartialEq)]
pub struct GeodataXformSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<GeodataXformBody10_1>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeodataXformBody10_1 {
    Identity {
        version: i32,
        reference: Option<Box<SpatialReferenceEntry10_1>>,
    },
    Polynomial(Box<PolynomialXform10_1>),
    Composite {
        version: i32,
        children: Vec<GeodataXformSlot10_1>,
        short_34: Option<u16>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolynomialXform10_1 {
    pub version: i32,
    pub reference: SpatialReferenceEntry10_1,
    pub long_a4: i32,
    pub coefficient_count: usize,
    pub point_count: usize,
    pub doubles_d4_dc: [f64; 2],
    pub flags_e4_e8_ec: [i32; 3],
    pub point_arrays: Option<[Vec<[f64; 2]>; 2]>,
    pub coefficients_e4: Option<[Vec<f64>; 2]>,
    pub coefficients_e8: Option<[Vec<f64>; 2]>,
    pub arrays_c8_cc: Option<[Vec<f64>; 2]>,
    pub name: Option<String>,
}

fn long(reader: &mut Reader<'_>) -> io::Result<i32> {
    Ok(reader.read_u32_le()? as i32)
}

fn bounded(reader: &Reader<'_>, count: usize, bytes_per_item: usize) -> io::Result<()> {
    if count > reader.remaining() / bytes_per_item {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "变换对象数组超出可读范围",
        ));
    }
    Ok(())
}

fn pair_of_arrays<'a, T>(
    r: &mut Reader<'a>,
    len: usize,
    size: usize,
    item: impl Fn(&mut Reader<'a>) -> io::Result<T>,
) -> io::Result<[Vec<T>; 2]> {
    bounded(r, len, size * 2)?;
    let a = (0..len).map(|_| item(r)).collect::<io::Result<_>>()?;
    let b = (0..len).map(|_| item(r)).collect::<io::Result<_>>()?;
    Ok([a, b])
}

fn nonnegative_count(reader: &mut Reader<'_>) -> io::Result<usize> {
    usize::try_from(long(reader)?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "变换对象数组数量为负"))
}

fn read_slot(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<GeodataXformSlot10_1> {
    let entry = read_object_entry_10_1(r, table)?;
    let body = if let ObjectEntry10_1::NewObject { class_id, .. } = entry {
        Some(Box::new(read_body(r, max, table, class_id, depth)?))
    } else {
        None
    };
    Ok(GeodataXformSlot10_1 { entry, body })
}

/// 普通对象槽。嵌套新类读取失败时，所有对象键及游标一并回退。
pub fn read_geodata_xform_slot_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<GeodataXformSlot10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let slot = read_slot(&mut trial, max, &mut staged, MAX_XFORM_NESTING)?;
    *reader = trial;
    *table = staged;
    Ok(slot)
}

/// 已消费对象头的 Variant 宿主入口。宿主负责提交根对象键。
pub(crate) fn read_geodata_xform_body_10_1(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<GeodataXformBody10_1> {
    read_body(r, max, table, class_id, MAX_XFORM_NESTING)
}

fn read_body(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    depth: usize,
) -> io::Result<GeodataXformBody10_1> {
    if depth == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "变换对象嵌套深度超过限额",
        ));
    }
    let label = match class_id {
        IDENTITY_XFORM_CLASS_ID => "IdentityXform",
        POLYNOMIAL_XFORM_CLASS_ID => "PolynomialXform",
        COMPOSITE_XFORM_CLASS_ID => "CompositeXform",
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "IGeodataXform 未支持的类 {}",
                    crate::storage::class_id_hex(&class_id),
                ),
            ));
        }
    };
    let version = long(r)?;
    if !(0..=2).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{label} 未支持的版本 {version}"),
        ));
    }
    Ok(match class_id {
        IDENTITY_XFORM_CLASS_ID => GeodataXformBody10_1::Identity {
            version,
            reference: (version >= 2)
                .then(|| read_spatial_reference_entry_10_1(r, max, table).map(Box::new))
                .transpose()?,
        },
        COMPOSITE_XFORM_CLASS_ID => {
            let count = long(r)?.max(0) as usize;
            bounded(r, count, 16)?;
            let children = (0..count)
                .map(|_| read_slot(r, max, table, depth - 1))
                .collect::<io::Result<_>>()?;
            let short_34 = (version >= 2).then(|| r.read_u16_le()).transpose()?;
            GeodataXformBody10_1::Composite {
                version,
                children,
                short_34,
            }
        }
        POLYNOMIAL_XFORM_CLASS_ID => {
            let reference = read_spatial_reference_entry_10_1(r, max, table)?;
            let long_a4 = long(r)?;
            let coefficient_count = nonnegative_count(r)?;
            let point_count = nonnegative_count(r)?;
            let doubles_d4_dc = [r.read_f64_le()?, r.read_f64_le()?];
            let flags_e4_e8_ec = [long(r)?, long(r)?, long(r)?];
            let point_arrays = (flags_e4_e8_ec[2] != 0)
                .then(|| {
                    pair_of_arrays(r, point_count, 16, |r| {
                        Ok([r.read_f64_le()?, r.read_f64_le()?])
                    })
                })
                .transpose()?;
            let coefficients_e4 = (flags_e4_e8_ec[0] != 0)
                .then(|| pair_of_arrays(r, coefficient_count, 8, Reader::read_f64_le))
                .transpose()?;
            let coefficients_e8 = (flags_e4_e8_ec[1] != 0)
                .then(|| pair_of_arrays(r, coefficient_count, 8, Reader::read_f64_le))
                .transpose()?;
            let arrays_c8_cc = if flags_e4_e8_ec[2] != 0 {
                let count = point_count.checked_mul(3).ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "变换对象数组数量溢出")
                })?;
                Some(pair_of_arrays(r, count, 8, Reader::read_f64_le)?)
            } else {
                None
            };
            let name = (version >= 2)
                .then(|| read_string_10_1(r, max))
                .transpose()?;
            GeodataXformBody10_1::Polynomial(Box::new(PolynomialXform10_1 {
                version,
                reference,
                long_a4,
                coefficient_count,
                point_count,
                doubles_d4_dc,
                flags_e4_e8_ec,
                point_arrays,
                coefficients_e4,
                coefficients_e8,
                arrays_c8_cc,
                name,
            }))
        }
        _ => unreachable!(),
    })
}
