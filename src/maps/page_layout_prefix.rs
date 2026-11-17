//! ArcMap 10.1 PageLayout 对象体的元素循环前缀。
//!
//! PageLayout.dll 10.1.0.3035 的 Load `0x7b6e4e30` 读取无符号版本、
//! 4 字节成员和有符号元素数；版本小于 7 使用普通对象读取，否则使用
//! ExtraSafeReadObject。调用者须先从对象流消费 PageLayout 对象头并保留
//! 跨流引用表。此处不跳过元素体，也不猜测未知成员的业务含义。

use crate::objects::element_body::{ElementBody10_1, read_element_body_of_class_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// PageLayout 类标识（Windows 字节序）。
pub const PAGE_LAYOUT_CLASS_ID: [u8; 16] = [
    0x6e, 0xd7, 0x94, 0xdd, 0x6d, 0x83, 0xd0, 0x11, 0x87, 0xec, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

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

/// 读取一个页面元素的 ExtraSafe 包装和已支持的元素体。
///
/// 仅支持 B 为零且 A 等于对象头加对象体长度的已观察形态；该长度解释
/// 尚属样本推定。遇到不支持的类、内层停止或范围不匹配时整体回滚，绝不
/// 跳过对象体或虚构引用登记。调用者负责按原文档的跨流顺序提供引用表。
pub fn read_page_layout_element_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, ElementBody10_1)> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let size = trial.read_u32_le()? as usize;
    let seek = trial.read_u32_le()?;
    if seek != 0 || size < 20 || size > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "不支持的页面元素范围",
        ));
    }
    let mut region = trial.read_region(size)?;
    let entry = read_object_entry_10_1(&mut region, &mut refs)?;
    let ObjectEntry10_1::NewObject { class_id, .. } = entry else {
        return Err(io::Error::new(ErrorKind::Unsupported, "页面元素并非新对象"));
    };
    let body = read_element_body_of_class_10_1(&class_id, &mut region, max_bytes, &mut refs)?;
    let stopped = match &body {
        ElementBody10_1::TextElement(body) => body.stop.is_some(),
        ElementBody10_1::MarkerElement(body) => body.stop.is_some(),
        ElementBody10_1::LineElement(body) => body.stop.is_some(),
    };
    if stopped || region.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "页面元素未完整消费声明范围",
        ));
    }
    *reader = trial;
    *table = refs;
    Ok((entry, body))
}
