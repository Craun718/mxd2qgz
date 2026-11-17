//! ArcMap 10.1 PageLayout 对象体的元素循环前缀。
//!
//! PageLayout.dll 10.1.0.3035 的 Load `0x7b6e4e30` 读取无符号版本、
//! 4 字节成员和有符号元素数；版本小于 7 使用普通对象读取，否则使用
//! ExtraSafeReadObject。调用者须先从对象流消费 PageLayout 对象头并保留
//! 跨流引用表。此处不跳过元素体，也不猜测未知成员的业务含义。

use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLayoutPrefix10_1 {
    pub version: u16,
    pub raw_member: u32,
    pub element_count: u32,
    pub extra_safe: bool,
}

/// 读取元素循环之前的固定前缀；错误时不推进游标。
pub fn read_page_layout_prefix_10_1(
    reader: &mut Reader<'_>,
    max_elements: u32,
) -> io::Result<PageLayoutPrefix10_1> {
    let mut trial = reader.clone();
    let version = u16::from_le_bytes(trial.read_bytes(2)?.try_into().unwrap());
    if version >= 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "PageLayout 版本不受支持",
        ));
    }
    let raw_member = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 || count as u32 > max_elements {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "PageLayout 元素数量无效或超过限额",
        ));
    }
    let prefix = PageLayoutPrefix10_1 {
        version,
        raw_member,
        element_count: count as u32,
        extra_safe: version >= 7,
    };
    *reader = trial;
    Ok(prefix)
}
