//! ArcGIS Desktop 10.8 对象流的局部读取规则。
//!
//! 此模块不定位 MXD 对象，不确定对象体长度，也不将规则推广至 10.1。

use crate::reader::Reader;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectHeader {
    Null,
    /// GUID 保留 Windows 磁盘字节顺序；键是否已存在由后续对象表判断。
    Object {
        class_id: [u8; 16],
        key: u32,
    },
}

/// 从已确定的对象起点读取 10.8 对象头，失败时不推进游标。
///
/// 返回后的位置仅是对象头末尾：已有键对应引用，新键后面仍有未知长度的对象体。
/// 不检查键的唯一性或类标识是否注册，也不加载任何 COM 对象。
pub fn read_header_10_8(reader: &mut Reader<'_>) -> io::Result<ObjectHeader> {
    let mut trial = reader.clone();
    let class_id: [u8; 16] = trial.read_bytes(16)?.try_into().unwrap();
    let header = if class_id == [0; 16] {
        ObjectHeader::Null
    } else {
        ObjectHeader::Object {
            class_id,
            key: trial.read_u32_le()?,
        }
    };
    *reader = trial;
    Ok(header)
}
