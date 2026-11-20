//! Geometry.dll 10.1.0.3035 的 MultiPatch 材质命令。

use super::multi_patch::MultiPatchMaterials10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiPatchMaterialCommand10_1<'a> {
    Color([u8; 3]),
    TransparencyPercent(u8),
    Shininess(u8),
    CullBackface,
    /// 命令存在时显示边线，载荷为 RGB 三个通道。
    EdgeColor([u8; 3]),
    EdgeWidth(u8),
    /// 指向提供纹理的材质，保留存盘的有符号索引。
    TextureReference {
        opcode: u8,
        material_index: i32,
    },
    Texture {
        opcode: u8,
        bytes_per_pixel: u8,
        width: u16,
        height: u16,
        /// 旧命令的编码由操作码区分，没有独立的存盘字段。
        encoding: Option<u32>,
        bytes: &'a [u8],
    },
}

pub struct MultiPatchMaterialCommands10_1<'a> {
    remaining: Option<Reader<'a>>,
    legacy: bool,
}

impl MultiPatchMaterials10_1 {
    /// 按材质索引借用其命令区间。失败仅终止当前材质的遍历，原始负载仍可访问。
    pub fn commands(&self, index: usize) -> io::Result<MultiPatchMaterialCommands10_1<'_>> {
        if self
            .revision
            .is_some_and(|revision| !(1..=3).contains(&revision))
        {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的 MultiPatch 材质版本",
            ));
        }
        let offsets = self
            .offsets
            .get(index..)
            .filter(|offsets| offsets.len() >= 2)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "MultiPatch 材质索引越界"))?;
        let bytes = self
            .payload
            .get(offsets[0] as usize..offsets[1] as usize)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "MultiPatch 材质区间无效"))?;
        Ok(MultiPatchMaterialCommands10_1 {
            remaining: Some(Reader::new(bytes)),
            legacy: self.revision.is_none(),
        })
    }
}

impl<'a> Iterator for MultiPatchMaterialCommands10_1<'a> {
    type Item = io::Result<MultiPatchMaterialCommand10_1<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.remaining.as_mut()?;
        if r.remaining() == 0 {
            return None;
        }
        let start = r.position();
        let result = read_command(r, self.legacy);
        if result.is_err() {
            self.remaining = None;
        }
        Some(result.map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("MultiPatch 材质命令 +0x{start:x}：{error}"),
            )
        }))
    }
}

/// 旧命令长度由 0x718fdccc 确认。带版本的命令由 0x718fed40 的查找器确认。
/// 旧转换器的复制阶段会多取 3/4 后的一字节，磁盘边界采用旧查找器的两字节宽度。
fn read_command<'a>(
    r: &mut Reader<'a>,
    legacy: bool,
) -> io::Result<MultiPatchMaterialCommand10_1<'a>> {
    use MultiPatchMaterialCommand10_1::*;

    let opcode = r.read_bytes(1)?[0];
    match opcode {
        1 => Ok(Color(r.read_bytes(3)?.try_into().unwrap())),
        2 => read_texture(r, opcode, legacy),
        3 => Ok(TransparencyPercent(r.read_bytes(1)?[0])),
        4 => Ok(Shininess(r.read_bytes(1)?[0])),
        5 | 8 if opcode == 5 || legacy => Ok(TextureReference {
            opcode,
            material_index: r.read_u32_le()? as i32,
        }),
        6 => Ok(CullBackface),
        7 if legacy => read_texture(r, opcode, true),
        9 if !legacy => Ok(EdgeColor(r.read_bytes(3)?.try_into().unwrap())),
        10 if !legacy => Ok(EdgeWidth(r.read_bytes(1)?[0])),
        _ => Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("未知操作码 0x{opcode:02x}"),
        )),
    }
}

fn read_texture<'a>(
    r: &mut Reader<'a>,
    opcode: u8,
    legacy: bool,
) -> io::Result<MultiPatchMaterialCommand10_1<'a>> {
    let bytes_per_pixel = r.read_bytes(1)?[0];
    let width = r.read_u16_le()?;
    let height = r.read_u16_le()?;
    if width > i16::MAX as u16 || height > i16::MAX as u16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "MultiPatch 纹理尺寸为负",
        ));
    }
    let size = if legacy && opcode == 2 {
        usize::from(width)
            .checked_mul(usize::from(height))
            .and_then(|area| area.checked_mul(usize::from(bytes_per_pixel)))
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "MultiPatch 纹理长度溢出"))?
    } else {
        usize::try_from(r.read_u32_le()? as i32)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "MultiPatch 纹理长度为负"))?
    };
    let encoding = if legacy { None } else { Some(r.read_u32_le()?) };
    let bytes = r.read_bytes(size)?;
    Ok(MultiPatchMaterialCommand10_1::Texture {
        opcode,
        bytes_per_pixel,
        width,
        height,
        encoding,
        bytes,
    })
}
