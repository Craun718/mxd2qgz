//! Desktop 10.8 PageLayout 分流格式 `/Maps` 的严格前缀读取。
//!
//! 原生读取链确认数量在流起点，首个对象入口在数量之后。本模块不解析对象体，
//! 不能推断地图数量之外的完整图层树，也不能推广到 10.1。

use crate::object_stream::{ObjectHeader, read_header_10_8};
use crate::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapsRootPrefix10_8 {
    /// `/Maps` 起点的原始有符号数量；仅正数时原生代码进入对象循环。
    pub count: i32,
    /// 正数量下的第一个对象头；空头保持后续字段未读取。
    pub first_header: Option<ObjectHeader>,
    /// 首个对象头之后未读取的字节数；对象体边界未知。
    pub remaining_bytes: usize,
}

/// 从 `/Maps` 起点读取 10.8 数量和首对象头，失败时不推进游标。
///
/// `max_count` 限制进入对象循环前申请解析工作的上限。负数量在原生分支中
/// 不进入循环，但这里显式拒绝，避免把未知异常当作空集合。
pub fn read_root_prefix_10_8(
    reader: &mut Reader<'_>,
    max_count: u32,
) -> io::Result<MapsRootPrefix10_8> {
    let mut trial = reader.clone();
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 {
        return Err(io::Error::new(ErrorKind::InvalidData, "/Maps 数量为负数"));
    }
    let limit = i32::try_from(max_count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "/Maps 限额不能由本平台表示"))?;
    if count > limit {
        return Err(io::Error::new(ErrorKind::InvalidData, "/Maps 数量超过限额"));
    }
    let first_header = if count == 0 {
        None
    } else {
        Some(read_header_10_8(&mut trial)?)
    };
    let result = MapsRootPrefix10_8 {
        count,
        first_header,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}
