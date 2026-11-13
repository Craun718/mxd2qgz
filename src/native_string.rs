//! Desktop 10.1 与 10.8 AfCore String::Write 普通输出的严格读取子集。
//!
//! 调用点、版本及字段归属必须由调用者确认。

use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// 读取包含终止零的字节长度前缀字符串，资源上限包含终止零、不包含前缀。
///
/// 拒绝原生代码未严格检查的短读、非法编码及提前零；失败时保持游标不变。
/// 错误位置相对于传入 Reader，调用方需另外记录流和对象身份。
pub fn read_string_10_8(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<String> {
    read_writer_subset(reader, max_bytes)
}

/// 读取 10.1 AfCore String::Write 的普通输出子集。
///
/// 这是严格安全子集，不能模拟原生短读行为，也不证明 MXD 字段归属。
pub fn read_string_10_1(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<String> {
    read_writer_subset(reader, max_bytes)
}

fn read_writer_subset(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<String> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("AfCore 字符串 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let length =
        usize::try_from(trial.read_u32_le()?).map_err(|_| invalid("长度不能由本平台表示"))?;
    if length < 2 || !length.is_multiple_of(2) {
        return Err(invalid("长度必须为包含终止零的偶数字节数，且至少为 2"));
    }
    if length > max_bytes {
        return Err(invalid("长度超过调用者资源限额"));
    }
    // 先借用完整内容并检查终止，再分配解码空间；下标由 length>=2 保证。
    let bytes = trial.read_bytes(length)?;
    if bytes[length - 2..] != [0, 0] {
        return Err(invalid("缺少末尾终止零"));
    }
    if bytes[..length - 2].chunks_exact(2).any(|u| u == [0, 0]) {
        return Err(invalid("含提前终止零，其后内容尚未支持"));
    }
    let value = Reader::new(&bytes[..length - 2])
        .read_utf16_le(length - 2)
        .map_err(|_| invalid("非法 UTF-16LE 内容"))?;
    *reader = trial;
    Ok(value)
}
