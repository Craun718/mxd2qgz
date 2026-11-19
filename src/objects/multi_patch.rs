//! Geometry.dll 10.1.0.3035 的面片记录，导入入口 0x718e0ee0。

use crate::stream::reader::Reader;
use crate::stream::zlib::decompress_exact;
use std::io::{self, ErrorKind};

const GENERAL_MULTIPATCH: u32 = 0x36;
const GENERAL_MODIFIERS: u32 = 0xdd00_0000;
const COMPRESSED: u32 = 0x0080_0000;

#[derive(Debug, Clone, PartialEq)]
pub enum MultiPatchRecord10_1 {
    Empty,
    Mesh(Box<MultiPatchMesh10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiPatchMesh10_1 {
    pub shape_type: u32,
    /// XMin、YMin、XMax、YMax。
    pub envelope: [f64; 4],
    pub part_starts: Vec<u32>,
    /// 保留低位形态标识及高位属性。
    pub part_types: Vec<u32>,
    pub xy: Vec<[f64; 2]>,
    pub z: MultiPatchOrdinate10_1,
    pub m: Option<MultiPatchOrdinate10_1>,
    pub point_ids: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub texture_coordinates: Option<MultiPatchTextureCoordinates10_1>,
    pub materials: Option<MultiPatchMaterials10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiPatchOrdinate10_1 {
    pub range: [f64; 2],
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiPatchTextureCoordinates10_1 {
    pub dimension: u32,
    pub vertex_count: u32,
    pub part_starts: Vec<u32>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiPatchMaterials10_1 {
    /// v1 的 0x36 记录省略此字段。原生兼容转换读取后补入。
    pub revision: Option<u32>,
    /// 相对于 payload 起点的偏移。最后一项为字节长度。
    pub offsets: Vec<u32>,
    pub payload: Vec<u8>,
}

/// 未确认的记录类型由宿主保留原始字节，坐标系仍从声明终点读取。
pub(super) fn decode_multi_patch_record(
    version: u16,
    bytes: &[u8],
    max_bytes: usize,
) -> io::Result<Option<MultiPatchRecord10_1>> {
    let mut header = Reader::new(bytes);
    let shape_type = header.read_u32_le()?;
    let mut packed_header = header.clone();
    let v1_wrapper = version == 1
        && packed_header.read_u32_le().is_ok()
        && packed_header
            .read_u32_le()
            .is_ok_and(|size| size as usize == packed_header.remaining());
    if v1_wrapper || shape_type & 0xff == 0x38 || shape_type & COMPRESSED != 0 {
        let unpacked = count(&mut header)?;
        let packed = count(&mut header)?;
        if unpacked > max_bytes || unpacked < 4 {
            return Err(invalid("MultiPatch 解压声明长度超限或过短"));
        }
        let input = header.read_bytes(packed)?;
        let data = decompress_exact(input, unpacked).map_err(|error| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("MultiPatch zlib 解压失败：{error}"),
            )
        })?;
        return decode_uncompressed(version, &data);
    }
    decode_uncompressed(version, bytes)
}

fn decode_uncompressed(version: u16, bytes: &[u8]) -> io::Result<Option<MultiPatchRecord10_1>> {
    let mut r = Reader::new(bytes);
    let shape_type = r.read_u32_le()?;
    if shape_type == 0 {
        return Ok(Some(MultiPatchRecord10_1::Empty));
    }
    let kind = shape_type & !GENERAL_MODIFIERS;
    let general = matches!(kind, 0x29 | 0x2a | GENERAL_MULTIPATCH);
    if !general && !matches!(shape_type, 0x1f | 0x20) {
        return Ok(None);
    }

    let envelope = doubles(&mut r)?;
    let part_count = count(&mut r)?;
    let point_count = count(&mut r)?;
    let part_starts = array(&mut r, part_count, 4, Reader::read_u32_le)?;
    if part_starts.first().is_some_and(|&start| start != 0)
        || part_starts
            .last()
            .is_some_and(|&start| start as usize >= point_count)
        || part_starts.windows(2).any(|pair| pair[0] >= pair[1])
        || (part_count == 0) != (point_count == 0)
    {
        return Err(invalid("MultiPatch 面片起始索引无效"));
    }
    let part_types = array(&mut r, part_count, 4, Reader::read_u32_le)?;
    let xy = array(&mut r, point_count, 16, doubles)?;
    let z = ordinate(&mut r, point_count)?;
    let mut mesh = MultiPatchMesh10_1 {
        shape_type,
        envelope,
        part_starts,
        part_types,
        xy,
        z,
        m: None,
        point_ids: Vec::new(),
        normals: Vec::new(),
        texture_coordinates: None,
        materials: None,
    };
    if general {
        // v1 转换对 0x36 补入材质版本，两个旧标识直接交给通用导入器。
        let legacy_materials = version == 1 && kind == GENERAL_MULTIPATCH;
        read_general_attributes(&mut r, &mut mesh, part_count, legacy_materials)?;
    } else if r.remaining() != 0 {
        let m = ordinate(&mut r, point_count)?;
        read_legacy_rendering(&mut mesh, &m);
        mesh.m = Some(m);
    }
    Ok(Some(MultiPatchRecord10_1::Mesh(Box::new(mesh))))
}

// 0x718e1835..0x718e188d 用 M 范围两端探测旧法线和纹理标记。
fn read_legacy_rendering(mesh: &mut MultiPatchMesh10_1, m: &MultiPatchOrdinate10_1) {
    const LEGACY_NORMAL: u32 = 0x0010_0000;
    const LEGACY_TEXTURE: u32 = 0x0020_0000;
    let marked = |bits: u64, flag| {
        let high = (bits >> 32) as u32;
        high & flag != 0 && high & 0x7fc0_0000 == 0
    };
    if m.range.iter().any(|m| marked(m.to_bits(), LEGACY_NORMAL)) {
        mesh.normals = m
            .values
            .iter()
            .map(|value| {
                let bits = value.to_bits();
                if !marked(bits, LEGACY_NORMAL) {
                    return [0.0; 3];
                }
                let high = (bits >> 32) as u32;
                let x = f64::from((high >> 1) & 0x1ff) / 255.0 - 1.0;
                let y = f64::from((high >> 10) & 0x1ff) / 255.0 - 1.0;
                let xy_squared = x * x + y * y;
                if xy_squared >= 1.0 {
                    let length = xy_squared.sqrt();
                    return [(x / length) as f32, (y / length) as f32, 0.0];
                }
                let z = (1.0 - xy_squared).sqrt();
                [
                    x as f32,
                    y as f32,
                    if high & 1 == 0 { z as f32 } else { -z as f32 },
                ]
            })
            .collect();
    }
    if m.range.iter().any(|m| marked(m.to_bits(), LEGACY_TEXTURE)) {
        let mut values = Vec::new();
        for m in &m.values {
            let bits = m.to_bits();
            if marked(bits, LEGACY_TEXTURE) {
                let coordinates = bits as u32;
                values.extend([
                    (f64::from(coordinates & 0xffff) / 21000.0 - 1.0) as f32,
                    (f64::from(coordinates >> 16) / 21000.0 - 1.0) as f32,
                ]);
            } else {
                values.extend([0.0, 0.0]);
            }
        }
        mesh.texture_coordinates = Some(MultiPatchTextureCoordinates10_1 {
            dimension: 2,
            vertex_count: m.values.len() as u32,
            part_starts: mesh.part_starts.clone(),
            values,
        });
    }
}

fn read_general_attributes(
    r: &mut Reader<'_>,
    mesh: &mut MultiPatchMesh10_1,
    part_count: usize,
    legacy_materials: bool,
) -> io::Result<()> {
    let measure_count = count(r)?;
    if measure_count != 0 {
        mesh.m = Some(ordinate(r, measure_count)?);
    }
    let id_count = count(r)?;
    mesh.point_ids = array(r, id_count, 4, Reader::read_u32_le)?;
    let normal_count = count(r)?;
    mesh.normals = array(r, normal_count, 12, |r| {
        Ok([float(r)?, float(r)?, float(r)?])
    })?;
    let texture_count = count(r)?;
    if texture_count != 0 {
        let dimension = count(r)?;
        if dimension == 0 {
            return Err(invalid("MultiPatch 纹理坐标维数为零"));
        }
        let scalar_count = texture_count
            .checked_mul(dimension)
            .ok_or_else(|| invalid("MultiPatch 纹理坐标数量溢出"))?;
        let part_starts = array(r, part_count, 4, Reader::read_u32_le)?;
        let values = array(r, scalar_count, 4, float)?;
        mesh.texture_coordinates = Some(MultiPatchTextureCoordinates10_1 {
            dimension: dimension as u32,
            vertex_count: texture_count as u32,
            part_starts,
            values,
        });
    }
    let material_count = count(r)?;
    if material_count != 0 {
        let revision = (!legacy_materials).then(|| r.read_u32_le()).transpose()?;
        let offsets = array(r, material_count + 1, 4, |r| Ok(count(r)? as u32))?;
        if offsets.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err(invalid("MultiPatch 材质偏移未顺序排列"));
        }
        let payload = r.read_bytes(offsets[material_count] as usize)?.to_vec();
        mesh.materials = Some(MultiPatchMaterials10_1 {
            revision,
            offsets,
            payload,
        });
    }
    Ok(())
}

fn ordinate(r: &mut Reader<'_>, n: usize) -> io::Result<MultiPatchOrdinate10_1> {
    Ok(MultiPatchOrdinate10_1 {
        range: doubles(r)?,
        values: array(r, n, 8, Reader::read_f64_le)?,
    })
}

fn doubles<const N: usize>(r: &mut Reader<'_>) -> io::Result<[f64; N]> {
    let mut result = [0.0; N];
    for value in &mut result {
        *value = r.read_f64_le()?;
    }
    Ok(result)
}

fn float(r: &mut Reader<'_>) -> io::Result<f32> {
    Ok(f32::from_bits(r.read_u32_le()?))
}

fn count(r: &mut Reader<'_>) -> io::Result<usize> {
    let value = r.read_u32_le()? as i32;
    usize::try_from(value).map_err(|_| invalid("MultiPatch 数量或偏移为负"))
}

fn array<'a, T>(
    r: &mut Reader<'a>,
    count: usize,
    width: usize,
    read: impl FnMut(&mut Reader<'a>) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let bytes = count
        .checked_mul(width)
        .ok_or_else(|| invalid("MultiPatch 数组字节数溢出"))?;
    let mut region = r.read_region(bytes)?;
    let mut read = read;
    (0..count).map(|_| read(&mut region)).collect()
}

fn invalid(reason: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, reason)
}
