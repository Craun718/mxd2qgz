//! ArcMap 10.1 IGeodataXform 的已确认持久化对象体。
//! 取证见 docs/mxd-10.1-geodata-xform.md。

use crate::layers::raster_renderer::{
    LoadedRaster10_1, RasterDoubleArraySlot10_1, read_double_array_slot, read_load_raster,
};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::objects::geo_transformation::{
    GeoTransformationBody10_1, read_geo_transformation_body_10_1,
    supports_geo_transformation_body_10_1,
};
use crate::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPOINT_CLASS_ID, POINT_CLASS_ID, POLYGON_CLASS_ID,
    POLYLINE_CLASS_ID, PointBody10_1, read_geometry_body_10_1, read_point_body_10_1,
};
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
pub const ADJUST_XFORM_CLASS_ID: [u8; 16] = [
    0x70, 0xee, 0x10, 0x16, 0x50, 0x8d, 0xe8, 0x41, 0xb2, 0xf3, 0x9b, 0x30, 0x02, 0xbe, 0x7b, 0xaa,
];
pub const APPROXIMATION_XFORM_CLASS_ID: [u8; 16] = [
    0xe6, 0xea, 0x72, 0xc0, 0x5d, 0xf8, 0x0f, 0x47, 0xab, 0xb5, 0x7c, 0xad, 0x96, 0x57, 0xd1, 0x6a,
];
pub const GCS_SHIFT_XFORM_CLASS_ID: [u8; 16] = [
    0x9a, 0xbe, 0x32, 0xcf, 0xc6, 0x58, 0x36, 0x4f, 0xac, 0x65, 0x92, 0x63, 0xfc, 0xfe, 0xde, 0x88,
];
pub const COORDINATE_XFORM_CLASS_ID: [u8; 16] = [
    0xab, 0xf4, 0x1a, 0xa1, 0x61, 0x98, 0x86, 0x4e, 0xac, 0x25, 0x77, 0x56, 0x68, 0x37, 0x21, 0x08,
];
pub const RPC_XFORM_CLASS_ID: [u8; 16] = [
    0xc6, 0xf1, 0x85, 0x74, 0x8f, 0x39, 0x5a, 0x46, 0x91, 0x0e, 0x50, 0xf6, 0x9c, 0x6f, 0x6f, 0x79,
];
pub const GEOMETRIC_XFORM_CLASS_ID: [u8; 16] = [
    0x10, 0x3b, 0x7c, 0x06, 0x91, 0x60, 0x04, 0x48, 0x82, 0xab, 0x1e, 0x06, 0x6f, 0x31, 0x51, 0xa6,
];
pub const LSR_XFORM_CLASS_ID: [u8; 16] = [
    0x49, 0x69, 0x44, 0x15, 0xd6, 0x7f, 0xb6, 0x44, 0x8b, 0xb2, 0x1a, 0x66, 0xff, 0x06, 0x83, 0x7e,
];
pub const PICTOMETRY_XFORM_CLASS_ID: [u8; 16] = [
    0x30, 0xbb, 0xd1, 0x3e, 0xab, 0xec, 0xf2, 0x4c, 0xbe, 0x54, 0xde, 0x2b, 0xd8, 0xd7, 0x13, 0x57,
];
pub const SPLINE_XFORM_CLASS_ID: [u8; 16] = [
    0x30, 0x7d, 0x77, 0x4c, 0xae, 0xc7, 0xbf, 0x45, 0x97, 0x81, 0x19, 0xaf, 0x06, 0x1a, 0x62, 0xe9,
];
pub const FRAME_XFORM_CLASS_ID: [u8; 16] = [
    0xac, 0x47, 0xe3, 0xd3, 0xe5, 0x80, 0x75, 0x4a, 0xb4, 0xa8, 0x1e, 0x7a, 0xc5, 0xd5, 0x07, 0x10,
];
pub const FORMOSAT2_XFORM_CLASS_ID: [u8; 16] = [
    0x6d, 0x51, 0x34, 0xa3, 0x73, 0xde, 0xbd, 0x43, 0x9c, 0xc5, 0x1c, 0xc3, 0x24, 0x70, 0x5e, 0xf8,
];

pub const SPOT5_XFORM_CLASS_ID: [u8; 16] = [
    0xe5, 0x3a, 0xc9, 0xbb, 0x9d, 0x45, 0x82, 0x4d, 0xa9, 0x5f, 0x2f, 0xa4, 0x9e, 0x9a, 0xe0, 0xc7,
];
pub const CSM_XFORM_CLASS_ID: [u8; 16] = [
    0x53, 0x1a, 0x80, 0x5f, 0x2a, 0x8d, 0x41, 0x4a, 0xb9, 0x33, 0xfe, 0xff, 0xcf, 0x0c, 0xca, 0xf8,
];
pub const PROJECTIVE_XFORM_CLASS_ID: [u8; 16] = [
    0x26, 0xc2, 0x7d, 0xcb, 0xd2, 0x5a, 0x40, 0x4a, 0x88, 0xf4, 0x3c, 0xed, 0xb5, 0xd6, 0x3b, 0xde,
];
pub const PROJECTIVE_TRANSFORMATION_2D_CLASS_ID: [u8; 16] = [
    0x4a, 0xb0, 0x2e, 0xf8, 0x5a, 0x4e, 0xc7, 0x43, 0x9a, 0x54, 0x45, 0x74, 0x39, 0xaa, 0x8e, 0x9e,
];
pub(crate) const MAX_XFORM_NESTING: usize = 16;

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
    Adjust {
        version: i32,
        short_c8: u16,
        long_a8: i32,
        reference: Box<SpatialReferenceEntry10_1>,
        inner: GeodataXformSlot10_1,
    },
    Approximation {
        version: i32,
        reference: Box<SpatialReferenceEntry10_1>,
        inner: GeodataXformSlot10_1,
        double_ac: f64,
        long_b4: i32,
        shorts_ba_b8: [u16; 2],
        doubles_1c_24_2c: [f64; 3],
        shorts_36_34: [u16; 2],
    },
    GcsShift {
        version: i32,
        double_b4: f64,
        reference: Box<SpatialReferenceEntry10_1>,
        short_bc: Option<u16>,
    },
    Coordinate {
        version: i32,
        references: Box<[SpatialReferenceEntry10_1; 2]>,
        operation_set: ObjectEntry10_1,
        double_b4: f64,
        long_124: i32,
        shorts_ac_ae: [u16; 2],
    },
    Rpc(Box<RpcXform10_1>),
    Geometric(Box<GeometricXform10_1>),
    Projective(Box<ProjectiveXform10_1>),
    Lsr(Box<LsrXform10_1>),
    Pictometry {
        version: i32,
        text: String,
    },
    Spline {
        version: i32,
        reference: Box<SpatialReferenceEntry10_1>,
        points: [GeometryCollectionSlot10_1; 2],
    },
    Frame(Box<FrameXform10_1>),
    Formosat2(Box<Formosat2Xform10_1>),
    Spot5(Box<Spot5Xform10_1>),
    Csm(CsmXform10_1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsmXform10_1 {
    pub version: i32,
    pub state_version: i32,
    pub flags_f8_fc: [i32; 2],
    /// 原生按窄字符字节写出，无结束零。保留编码和模型状态原值。
    pub states: [Vec<u8>; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spot5Xform10_1 {
    pub version: i32,
    pub legacy: Option<(String, u16)>,
    pub longs_fc_104: [i32; 3],
    pub doubles_10c_11c: [f64; 3],
    pub longs_124_140: [i32; 8],
    pub short_144: u16,
    pub long_148: i32,
    /// 末两项按原生写入次序保存。
    pub doubles_14c_15c_154: [f64; 3],
    pub double_arrays: Vec<RasterDoubleArraySlot10_1>,
    pub tail: Option<Spot5XformTail10_1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spot5XformTail10_1 {
    pub doubles_1c_24_2c: [f64; 3],
    pub shorts_36_34: [u16; 2],
    pub raster: Option<LoadedRaster10_1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Formosat2Xform10_1 {
    pub version: i32,
    pub longs_a8_cc: [i32; 10],
    /// 最后两项从成员 +0xfc、+0xf4 依次写入。
    pub doubles_d4_fc_f4: [f64; 6],
    pub double_arrays: Vec<RasterDoubleArraySlot10_1>,
    pub polynomial: GeodataXformSlot10_1,
    pub doubles_1c_24_2c: [f64; 3],
    pub shorts_36_34: [u16; 2],
    pub raster: Option<LoadedRaster10_1>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameXform10_1 {
    pub version: i32,
    pub reference: SpatialReferenceEntry10_1,
    pub doubles_1c_24_2c: [f64; 3],
    pub shorts_36_34: [u16; 2],
    pub raster: Option<LoadedRaster10_1>,
    pub inner: GeodataXformSlot10_1,
    pub short_a8: u16,
    pub doubles_ac: [f64; 5],
    pub points: [XformPointSlot10_1; 2],
    pub short_e0: u16,
    pub double_e4: f64,
    pub short_f4: u16,
    pub doubles_fc: [f64; 9],
    pub double_144: f64,
    pub long_14c: i32,
    pub double_ec: Option<f64>,
    pub secondary_transform: Option<GeodataXformSlot10_1>,
    pub long_d4: Option<i32>,
    pub double_arrays: Option<[RasterDoubleArraySlot10_1; 2]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct XformPointSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<PointBody10_1>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LsrXform10_1 {
    pub version: i32,
    pub point_entry: ObjectEntry10_1,
    pub point: Option<Box<PointBody10_1>>,
    pub doubles_ac: [f64; 9],
    pub doubles_f4_fc: [f64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometricXform10_1 {
    pub version: i32,
    pub reference: SpatialReferenceEntry10_1,
    pub geometry: GeometryCollectionSlot10_1,
    pub transformation: GeometricTransformation10_1,
    pub double_ac: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectiveXform10_1 {
    pub version: i32,
    pub reference: SpatialReferenceEntry10_1,
    pub geometry: GeometryCollectionSlot10_1,
    /// 已确认的投影变换槽，包含有界旧档的空槽。
    pub transformation: Option<ObjectEntry10_1>,
    pub point_count: usize,
    pub points_flag: i32,
    pub point_arrays: Option<[Vec<[f64; 2]>; 2]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCollectionSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<GeometryBody10_1>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeometricTransformation10_1 {
    None,
    Affine([f64; 6]),
    Projective([f64; 8]),
    Object {
        selector: u8,
        entry: ObjectEntry10_1,
        body: Option<Box<GeoTransformationBody10_1>>,
    },
    RasterObject {
        selector: u8,
        entry: ObjectEntry10_1,
        body: Box<GeodataXformBody10_1>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct RpcXform10_1 {
    pub version: i32,
    /// 原生一次读写 0x2d0 字节，保留成员顺序及原始位模式。
    pub raw_parameters: Box<[u8; 0x2d0]>,
    pub inner: GeodataXformSlot10_1,
    pub doubles_1c_24_2c: [f64; 3],
    pub shorts_36_34: [u16; 2],
    pub raster: Option<LoadedRaster10_1>,
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

#[derive(Clone, Copy)]
enum CollectionInterface {
    Geometry,
    Point,
}

fn read_geometry_collection(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    label: &str,
    interface: CollectionInterface,
) -> io::Result<GeometryCollectionSlot10_1> {
    let accepts = |class| {
        matches!(
            class,
            POLYLINE_CLASS_ID | POLYGON_CLASS_ID | MULTIPOINT_CLASS_ID
        ) || (matches!(interface, CollectionInterface::Geometry) && class == GEOMETRY_BAG_CLASS_ID)
    };
    let mut entry = read_object_entry_10_1(r, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if !accepts(registered) {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("{label} 回引键 {key} 的登记类不受支持"),
            ));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject { class_id, .. } if accepts(class_id) => {
            Some(Box::new(read_geometry_body_10_1(&class_id, r, max, table)?))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "{label} 未支持的集合类 {}",
                    crate::storage::class_id_hex(&class_id),
                ),
            ));
        }
    };
    Ok(GeometryCollectionSlot10_1 { entry, body })
}

fn read_geometric_xform(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    version: i32,
    child_depth: usize,
) -> io::Result<GeometricXform10_1> {
    let reference = read_spatial_reference_entry_10_1(r, max, table)?;
    let geometry = read_geometry_collection(
        r,
        max,
        table,
        "GeometricXform IGeometryCollection",
        CollectionInterface::Geometry,
    )?;
    let selector = r.read_bytes(1)?[0];
    let transformation = if selector == 0 {
        match r.read_bytes(1)?[0] {
            0 => GeometricTransformation10_1::None,
            1 => {
                let mut values = [0.0; 6];
                for value in &mut values {
                    *value = r.read_f64_le()?;
                }
                GeometricTransformation10_1::Affine(values)
            }
            2 => {
                let mut values = [0.0; 8];
                for value in &mut values {
                    *value = r.read_f64_le()?;
                }
                GeometricTransformation10_1::Projective(values)
            }
            kind => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("GeometricXform 未支持的内联变换类型 {kind}"),
                ));
            }
        }
    } else {
        let entry = read_object_entry_10_1(r, table)?;
        match entry {
            ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => {
                GeometricTransformation10_1::Object {
                    selector,
                    entry,
                    body: None,
                }
            }
            ObjectEntry10_1::NewObject { class_id, .. }
                if supports_geo_transformation_body_10_1(class_id) =>
            {
                GeometricTransformation10_1::Object {
                    selector,
                    entry,
                    body: Some(Box::new(read_geo_transformation_body_10_1(
                        r, class_id, max,
                    )?)),
                }
            }
            ObjectEntry10_1::NewObject {
                class_id:
                    class_id @ (IDENTITY_XFORM_CLASS_ID
                    | POLYNOMIAL_XFORM_CLASS_ID
                    | COMPOSITE_XFORM_CLASS_ID
                    | ADJUST_XFORM_CLASS_ID
                    | APPROXIMATION_XFORM_CLASS_ID
                    | GCS_SHIFT_XFORM_CLASS_ID
                    | COORDINATE_XFORM_CLASS_ID
                    | RPC_XFORM_CLASS_ID
                    | GEOMETRIC_XFORM_CLASS_ID
                    | LSR_XFORM_CLASS_ID
                    | PICTOMETRY_XFORM_CLASS_ID
                    | SPLINE_XFORM_CLASS_ID
                    | FRAME_XFORM_CLASS_ID
                    | FORMOSAT2_XFORM_CLASS_ID
                    | SPOT5_XFORM_CLASS_ID
                    | CSM_XFORM_CLASS_ID
                    | PROJECTIVE_XFORM_CLASS_ID),
                ..
            } => GeometricTransformation10_1::RasterObject {
                selector,
                entry,
                body: Box::new(read_body(r, max, table, class_id, child_depth)?),
            },
            ObjectEntry10_1::NewObject { class_id, .. } => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "GeometricXform 未支持的 ITransformation 类 {}",
                        crate::storage::class_id_hex(&class_id),
                    ),
                ));
            }
        }
    };
    let double_ac = (version >= 2).then(|| r.read_f64_le()).transpose()?;
    Ok(GeometricXform10_1 {
        version,
        reference,
        geometry,
        transformation,
        double_ac,
    })
}

fn read_projective_xform(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    version: i32,
    has_empty_third_slot: bool,
) -> io::Result<ProjectiveXform10_1> {
    let reference = read_spatial_reference_entry_10_1(r, max, table)?;
    let (geometry, transformation) =
        if !has_empty_third_slot && projective_transformation_follows(r) {
            // Save 写的是投影变换成员，只有 Load 所需的布局另含几何集合。
            let geometry = GeometryCollectionSlot10_1 {
                entry: ObjectEntry10_1::Null,
                body: None,
            };
            (
                geometry,
                Some(read_projective_transformation_reference(r, table)?),
            )
        } else {
            let geometry = read_geometry_collection(
                r,
                max,
                table,
                "ProjectiveXform 第二对象槽",
                CollectionInterface::Geometry,
            )?;
            // 空 GUID 与两个零计数字段可能重合。只按明确的 CLSID 接受额外槽。
            let transformation = if has_empty_third_slot {
                if r.read_bytes(16)? != [0; 16] {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "ProjectiveXform 的第三槽不是空对象",
                    ));
                }
                Some(ObjectEntry10_1::Null)
            } else {
                projective_transformation_follows(r)
                    .then(|| read_projective_transformation_reference(r, table))
                    .transpose()?
            };
            (geometry, transformation)
        };
    let point_count = nonnegative_count(r)?;
    let points_flag = long(r)?;
    let point_arrays = (points_flag != 0)
        .then(|| {
            pair_of_arrays(r, point_count, 16, |r| {
                Ok([r.read_f64_le()?, r.read_f64_le()?])
            })
        })
        .transpose()?;
    Ok(ProjectiveXform10_1 {
        version,
        reference,
        geometry,
        transformation,
        point_count,
        points_flag,
        point_arrays,
    })
}

fn projective_transformation_follows(r: &Reader<'_>) -> bool {
    r.clone()
        .read_bytes(16)
        .is_ok_and(|bytes| bytes == PROJECTIVE_TRANSFORMATION_2D_CLASS_ID)
}

fn read_projective_transformation_reference(
    r: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<ObjectEntry10_1> {
    let entry = read_object_entry_10_1(r, table)?;
    if let ObjectEntry10_1::BackReference { key, .. } = entry {
        if table
            .registered_class(key)
            .is_some_and(|class| class != PROJECTIVE_TRANSFORMATION_2D_CLASS_ID)
        {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("ProjectiveTransformation2D 回引键 {key} 的登记类不受支持"),
            ));
        }
        return Ok(entry);
    }
    // Geometry.dll 的 ProjectiveTransformation2D Load 返回 E_NOTIMPL。
    Err(io::Error::new(
        ErrorKind::Unsupported,
        "ProjectiveTransformation2D 新对象没有可读取的 10.1 二进制体",
    ))
}

fn read_point_slot(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    label: &str,
) -> io::Result<XformPointSlot10_1> {
    let entry = read_object_entry_10_1(r, table)?;
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: POINT_CLASS_ID,
            ..
        } => Some(Box::new(read_point_body_10_1(r, max, table)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "{label} 未支持的 IPoint 类 {}",
                    crate::storage::class_id_hex(&class_id)
                ),
            ));
        }
    };
    Ok(XformPointSlot10_1 { entry, body })
}

fn read_doubles<const N: usize>(r: &mut Reader<'_>) -> io::Result<[f64; N]> {
    let mut values = [0.0; N];
    for value in &mut values {
        *value = r.read_f64_le()?;
    }
    Ok(values)
}

fn read_lsr_xform(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    version: i32,
) -> io::Result<LsrXform10_1> {
    let point = read_point_slot(r, max, table, "LSRXform")?;
    Ok(LsrXform10_1 {
        version,
        point_entry: point.entry,
        point: point.body,
        doubles_ac: read_doubles(r)?,
        doubles_f4_fc: read_doubles(r)?,
    })
}

fn read_frame_xform(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    version: i32,
    child_depth: usize,
) -> io::Result<FrameXform10_1> {
    Ok(FrameXform10_1 {
        version,
        reference: read_spatial_reference_entry_10_1(r, max, table)?,
        doubles_1c_24_2c: read_doubles(r)?,
        shorts_36_34: [r.read_u16_le()?, r.read_u16_le()?],
        raster: read_load_raster(r, max, table, child_depth)?,
        inner: read_geodata_xform_slot_with_depth(r, max, table, child_depth)?,
        short_a8: r.read_u16_le()?,
        doubles_ac: read_doubles(r)?,
        points: [
            read_point_slot(r, max, table, "FrameXform")?,
            read_point_slot(r, max, table, "FrameXform")?,
        ],
        short_e0: r.read_u16_le()?,
        double_e4: r.read_f64_le()?,
        short_f4: r.read_u16_le()?,
        doubles_fc: read_doubles(r)?,
        double_144: r.read_f64_le()?,
        long_14c: long(r)?,
        double_ec: (version >= 5).then(|| r.read_f64_le()).transpose()?,
        secondary_transform: (version >= 5)
            .then(|| read_geodata_xform_slot_with_depth(r, max, table, child_depth))
            .transpose()?,
        long_d4: (version >= 6).then(|| long(r)).transpose()?,
        double_arrays: (version >= 7)
            .then(|| -> io::Result<_> {
                Ok([
                    read_double_array_slot(r, table)?,
                    read_double_array_slot(r, table)?,
                ])
            })
            .transpose()?,
    })
}

fn read_formosat2_xform(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    version: i32,
    child_depth: usize,
) -> io::Result<Formosat2Xform10_1> {
    let longs_a8_cc = read_longs(r)?;
    let doubles_d4_fc_f4 = read_doubles(r)?;
    let double_arrays = (0..14)
        .map(|_| read_double_array_slot(r, table))
        .collect::<io::Result<_>>()?;
    let entry = read_object_entry_10_1(r, table)?;
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: POLYNOMIAL_XFORM_CLASS_ID,
            ..
        } => Some(Box::new(read_body(
            r,
            max,
            table,
            POLYNOMIAL_XFORM_CLASS_ID,
            child_depth,
        )?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Formosat2Xform 未支持的 IPolynomialXform 类 {}",
                    crate::storage::class_id_hex(&class_id)
                ),
            ));
        }
    };
    Ok(Formosat2Xform10_1 {
        version,
        longs_a8_cc,
        doubles_d4_fc_f4,
        double_arrays,
        polynomial: GeodataXformSlot10_1 { entry, body },
        doubles_1c_24_2c: read_doubles(r)?,
        shorts_36_34: [r.read_u16_le()?, r.read_u16_le()?],
        raster: read_load_raster(r, max, table, child_depth)?,
    })
}

fn read_longs<const N: usize>(r: &mut Reader<'_>) -> io::Result<[i32; N]> {
    let mut values = [0; N];
    for value in &mut values {
        *value = long(r)?;
    }
    Ok(values)
}

fn read_csm_xform(r: &mut Reader<'_>, max: usize, version: i32) -> io::Result<CsmXform10_1> {
    let state_version = long(r)?;
    if state_version != 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("CSMXform 未支持的内部版本 {state_version}"),
        ));
    }
    let flags_f8_fc = read_longs(r)?;
    if flags_f8_fc.iter().any(|&flag| !matches!(flag, 0 | 1)) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "CSMXform 标志必须为 0 或 1",
        ));
    }
    let mut states = [Vec::new(), Vec::new()];
    let mut budget = max;
    for state in &mut states {
        let size = usize::try_from(r.read_u32_le()?)
            .ok()
            .filter(|&size| size <= budget)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "CSMXform 状态长度超过限额"))?;
        let bytes = r.read_bytes(size)?;
        if bytes.contains(&0) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "CSMXform 状态含提前结束的零字节",
            ));
        }
        state.extend_from_slice(bytes);
        budget -= size;
    }
    Ok(CsmXform10_1 {
        version,
        state_version,
        flags_f8_fc,
        states,
    })
}

fn read_spot5_xform(
    r: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    version: i32,
    child_depth: usize,
) -> io::Result<Spot5Xform10_1> {
    Ok(Spot5Xform10_1 {
        version,
        legacy: (version == 1)
            .then(|| -> io::Result<_> { Ok((read_string_10_1(r, max)?, r.read_u16_le()?)) })
            .transpose()?,
        longs_fc_104: read_longs(r)?,
        doubles_10c_11c: read_doubles(r)?,
        longs_124_140: read_longs(r)?,
        short_144: r.read_u16_le()?,
        long_148: long(r)?,
        doubles_14c_15c_154: read_doubles(r)?,
        double_arrays: (0..21)
            .map(|_| read_double_array_slot(r, table))
            .collect::<io::Result<_>>()?,
        tail: (version >= 2)
            .then(|| -> io::Result<_> {
                Ok(Spot5XformTail10_1 {
                    doubles_1c_24_2c: read_doubles(r)?,
                    shorts_36_34: [r.read_u16_le()?, r.read_u16_le()?],
                    raster: read_load_raster(r, max, table, child_depth)?,
                })
            })
            .transpose()?,
    })
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

/// 调用者负责提交游标及键，包括在 LoadRaster 中跨对象重新进入时。
pub(crate) fn read_geodata_xform_slot_with_depth(
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
    let slot = read_geodata_xform_slot_with_depth(&mut trial, max, &mut staged, MAX_XFORM_NESTING)?;
    *reader = trial;
    *table = staged;
    Ok(slot)
}

/// 旧式投影变换多一个空槽，仅供能验证后继字段与结束位置的宿主试读。
pub(crate) fn read_projective_with_empty_third_slot(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<GeodataXformSlot10_1> {
    let entry = read_object_entry_10_1(reader, table)?;
    if !matches!(
        entry,
        ObjectEntry10_1::NewObject {
            class_id: PROJECTIVE_XFORM_CLASS_ID,
            ..
        }
    ) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "需要新 ProjectiveXform 对象",
        ));
    }
    let version = long(reader)?;
    if version != 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("ProjectiveXform 未支持的版本 {version}"),
        ));
    }
    let body = read_projective_xform(reader, max, table, version, true)?;
    Ok(GeodataXformSlot10_1 {
        entry,
        body: Some(Box::new(GeodataXformBody10_1::Projective(Box::new(body)))),
    })
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
    let (label, versions) = match class_id {
        IDENTITY_XFORM_CLASS_ID => ("IdentityXform", 0..=2),
        POLYNOMIAL_XFORM_CLASS_ID => ("PolynomialXform", 0..=2),
        COMPOSITE_XFORM_CLASS_ID => ("CompositeXform", 0..=2),
        ADJUST_XFORM_CLASS_ID => ("AdjustXform", 1..=1),
        APPROXIMATION_XFORM_CLASS_ID => ("ApproximationXform", 1..=1),
        GCS_SHIFT_XFORM_CLASS_ID => ("GCSShiftXform", 1..=2),
        COORDINATE_XFORM_CLASS_ID => ("CoordinateXform", 1..=1),
        RPC_XFORM_CLASS_ID => ("RPCXform", 1..=2),
        GEOMETRIC_XFORM_CLASS_ID => ("GeometricXform", 1..=2),
        PROJECTIVE_XFORM_CLASS_ID => ("ProjectiveXform", 1..=1),
        LSR_XFORM_CLASS_ID => ("LSRXform", 1..=1),
        PICTOMETRY_XFORM_CLASS_ID => ("PictometryXform", 1..=2),
        SPLINE_XFORM_CLASS_ID => ("SplineXform", 1..=1),
        FRAME_XFORM_CLASS_ID => ("FrameXform", 1..=7),
        FORMOSAT2_XFORM_CLASS_ID => ("Formosat2Xform", 1..=1),
        SPOT5_XFORM_CLASS_ID => ("SPOT5Xform", 1..=2),
        CSM_XFORM_CLASS_ID => ("CSMXform", 1..=2),
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
    let version = if matches!(
        class_id,
        LSR_XFORM_CLASS_ID
            | PICTOMETRY_XFORM_CLASS_ID
            | FRAME_XFORM_CLASS_ID
            | FORMOSAT2_XFORM_CLASS_ID
            | SPOT5_XFORM_CLASS_ID
            | CSM_XFORM_CLASS_ID
    ) {
        i32::from(r.read_u16_le()?)
    } else {
        long(r)?
    };
    if !versions.contains(&version) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{label} 未支持的版本 {version}"),
        ));
    }
    Ok(match class_id {
        CSM_XFORM_CLASS_ID => GeodataXformBody10_1::Csm(read_csm_xform(r, max, version)?),
        GEOMETRIC_XFORM_CLASS_ID => GeodataXformBody10_1::Geometric(Box::new(
            read_geometric_xform(r, max, table, version, depth - 1)?,
        )),
        PROJECTIVE_XFORM_CLASS_ID => GeodataXformBody10_1::Projective(Box::new(
            read_projective_xform(r, max, table, version, false)?,
        )),
        LSR_XFORM_CLASS_ID => {
            GeodataXformBody10_1::Lsr(Box::new(read_lsr_xform(r, max, table, version)?))
        }
        FRAME_XFORM_CLASS_ID => GeodataXformBody10_1::Frame(Box::new(read_frame_xform(
            r,
            max,
            table,
            version,
            depth - 1,
        )?)),
        FORMOSAT2_XFORM_CLASS_ID => GeodataXformBody10_1::Formosat2(Box::new(
            read_formosat2_xform(r, max, table, version, depth - 1)?,
        )),
        SPOT5_XFORM_CLASS_ID => GeodataXformBody10_1::Spot5(Box::new(read_spot5_xform(
            r,
            max,
            table,
            version,
            depth - 1,
        )?)),
        PICTOMETRY_XFORM_CLASS_ID => GeodataXformBody10_1::Pictometry {
            version,
            // Save 在版本后始终写一条 String。原生 Load 的内存状态门不改变写侧布局。
            text: read_string_10_1(r, max)?,
        },
        SPLINE_XFORM_CLASS_ID => GeodataXformBody10_1::Spline {
            version,
            reference: Box::new(read_spatial_reference_entry_10_1(r, max, table)?),
            points: [
                read_geometry_collection(
                    r,
                    max,
                    table,
                    "SplineXform IPointCollection",
                    CollectionInterface::Point,
                )?,
                read_geometry_collection(
                    r,
                    max,
                    table,
                    "SplineXform IPointCollection",
                    CollectionInterface::Point,
                )?,
            ],
        },
        IDENTITY_XFORM_CLASS_ID => GeodataXformBody10_1::Identity {
            version,
            reference: (version >= 2)
                .then(|| read_spatial_reference_entry_10_1(r, max, table).map(Box::new))
                .transpose()?,
        },
        ADJUST_XFORM_CLASS_ID => GeodataXformBody10_1::Adjust {
            version,
            short_c8: r.read_u16_le()?,
            long_a8: long(r)?,
            reference: Box::new(read_spatial_reference_entry_10_1(r, max, table)?),
            inner: read_geodata_xform_slot_with_depth(r, max, table, depth - 1)?,
        },
        APPROXIMATION_XFORM_CLASS_ID => GeodataXformBody10_1::Approximation {
            version,
            reference: Box::new(read_spatial_reference_entry_10_1(r, max, table)?),
            inner: read_geodata_xform_slot_with_depth(r, max, table, depth - 1)?,
            double_ac: r.read_f64_le()?,
            long_b4: long(r)?,
            shorts_ba_b8: [r.read_u16_le()?, r.read_u16_le()?],
            doubles_1c_24_2c: [r.read_f64_le()?, r.read_f64_le()?, r.read_f64_le()?],
            shorts_36_34: [r.read_u16_le()?, r.read_u16_le()?],
        },
        GCS_SHIFT_XFORM_CLASS_ID => GeodataXformBody10_1::GcsShift {
            version,
            double_b4: r.read_f64_le()?,
            reference: Box::new(read_spatial_reference_entry_10_1(r, max, table)?),
            short_bc: (version >= 2).then(|| r.read_u16_le()).transpose()?,
        },
        COORDINATE_XFORM_CLASS_ID => {
            let references = Box::new([
                read_spatial_reference_entry_10_1(r, max, table)?,
                read_spatial_reference_entry_10_1(r, max, table)?,
            ]);
            let operation_set = read_object_entry_10_1(r, table)?;
            if let ObjectEntry10_1::NewObject { class_id, .. } = operation_set {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "CoordinateXform 未支持的 IGeoTransformationOperationSet 类 {}",
                        crate::storage::class_id_hex(&class_id),
                    ),
                ));
            }
            GeodataXformBody10_1::Coordinate {
                version,
                references,
                operation_set,
                double_b4: r.read_f64_le()?,
                long_124: long(r)?,
                shorts_ac_ae: [r.read_u16_le()?, r.read_u16_le()?],
            }
        }
        RPC_XFORM_CLASS_ID => {
            let raw_parameters = Box::new(r.read_bytes(0x2d0)?.try_into().unwrap());
            let inner = read_geodata_xform_slot_with_depth(r, max, table, depth - 1)?;
            let doubles_1c_24_2c = [r.read_f64_le()?, r.read_f64_le()?, r.read_f64_le()?];
            let shorts_36_34 = [r.read_u16_le()?, r.read_u16_le()?];
            let raster = if version >= 2 {
                read_load_raster(r, max, table, depth - 1)?
            } else {
                None
            };
            GeodataXformBody10_1::Rpc(Box::new(RpcXform10_1 {
                version,
                raw_parameters,
                inner,
                doubles_1c_24_2c,
                shorts_36_34,
                raster,
            }))
        }
        COMPOSITE_XFORM_CLASS_ID => {
            let count = long(r)?.max(0) as usize;
            bounded(r, count, 16)?;
            let children = (0..count)
                .map(|_| read_geodata_xform_slot_with_depth(r, max, table, depth - 1))
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
