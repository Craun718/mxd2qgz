//! Desktop 10.1 和 10.8 的 OLE `/Version` 已核实字段。
//!
//! 原生入口先调用 AfCore String::Read，再从同一个底层流读取 2 字节。
//! 读取器是严格安全子集，保留无法确认的后续数据，不模拟原生短读。

use crate::native_string::{read_string_10_1, read_string_10_8};
use crate::reader::Reader;
use std::io;

/// 10.1 局部读取分支的输出值，不代表完整版本映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityBranch10_1 {
    Three,
    Five,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionLayout10_1 {
    /// 原样保留文本；其完整版本语义仍未闭合。
    pub text: String,
    /// 按原生有符号 short 比较，保留全部位模式。
    pub compatibility_short: i16,
    pub compatibility_branch: CompatibilityBranch10_1,
    /// 不解释，也不消费后续未知数据。
    pub remaining_bytes: usize,
}

/// 严格读取 10.1 Version 的已证实前缀；失败时不推进游标。
///
/// 限制与原生短读行为不同；不能据此推断完整 Version 格式或保存版本。
pub fn read_layout_10_1(
    reader: &mut Reader<'_>,
    max_text_bytes: usize,
) -> io::Result<VersionLayout10_1> {
    let mut trial = reader.clone();
    let text = read_string_10_1(&mut trial, max_text_bytes)?;
    let compatibility_short = i16::from_le_bytes(trial.read_bytes(2)?.try_into().unwrap());
    let compatibility_branch = if compatibility_short < 0x12 {
        CompatibilityBranch10_1::Three
    } else {
        CompatibilityBranch10_1::Five
    };
    let result = VersionLayout10_1 {
        text,
        compatibility_short,
        compatibility_branch,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}

/// ArcMap 10.1 Version 流中已核实的头部字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcMapVersionHeader10_1 {
    pub prefix: VersionLayout10_1,
    /// short 大于 11 时使用的对象流计数；不把它解释为图层数。
    pub object_count: Option<i32>,
    /// 头部之后的未知字节数量。
    pub remaining_bytes: usize,
}

/// 严格读取 ArcMap 10.1 的 Version 头部；不模拟原生短读。
pub fn read_arcmap_header_10_1(
    reader: &mut Reader<'_>,
    max_text_bytes: usize,
) -> io::Result<ArcMapVersionHeader10_1> {
    let mut trial = reader.clone();
    let prefix = read_layout_10_1(&mut trial, max_text_bytes)?;
    if prefix.compatibility_short > 0x13 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ArcMap 10.1 不支持大于 0x13 的 Version short",
        ));
    }
    let object_count = if prefix.compatibility_short > 0x0b {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    let remaining_bytes = trial.remaining();
    *reader = trial;
    Ok(ArcMapVersionHeader10_1 {
        prefix,
        object_count,
        remaining_bytes,
    })
}

/// 构造 10.1 写入路径的 Version 头部；文本由调用者提供资源解析结果。
///
/// 非负保存版本索引超过表末尾时使用最后一项；对象计数由实际序列化过程提供。
/// 不接受原生实现中会越界访问表的负索引。
pub fn write_arcmap_header_10_1(
    save_index: i32,
    text: &str,
    object_count: Option<i32>,
) -> io::Result<Vec<u8>> {
    const SHORTS: [i16; 6] = [10, 13, 14, 16, 18, 19];
    let index = usize::try_from(save_index)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "保存版本索引不能为负数"))?;
    let short = SHORTS[index.min(SHORTS.len() - 1)];
    if text.contains('\0') || (save_index == 0) != object_count.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "文本包含零字符，或对象计数与保存版本索引不匹配",
        ));
    }
    let units = text.encode_utf16().count();
    let length = units
        .checked_add(1)
        .and_then(|n| n.checked_mul(2))
        .filter(|&n| n <= 0x7fff_ffff)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "版本文本过长"))?;
    let mut out = Vec::new();
    out.extend_from_slice(&(length as u32).to_le_bytes());
    for unit in text.encode_utf16().chain([0]) {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out.extend_from_slice(&short.to_le_bytes());
    if let Some(count) = object_count {
        out.extend_from_slice(&count.to_le_bytes());
    }
    Ok(out)
}

/// `/Version` short 在 10.8 原生读取器中产生的两个输出分支。
///
/// 变体名只表示原生函数的输出值；不声称其跨版本兼容语义已经闭环。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityBranch10_8 {
    Three,
    Eleven,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionLayout10_8 {
    /// 未经解释的候选文本；不能当作 MXD 保存版本。
    pub text: String,
    /// 保留全部位模式的有符号 short；映射语义仍待验证。
    pub compatibility_short: i16,
    /// 按原生有符号比较得到的输出分支：小于 `0x12` 为 3，否则为 `0x0b`。
    pub compatibility_branch: CompatibilityBranch10_8,
    /// short 之后未读取的字节数，保持位置供后续诊断。
    pub remaining_bytes: usize,
}

/// 从 `/Version` 起点读取 10.8 已确认的局部布局，失败时不推进游标。
///
/// `max_text_bytes` 是 AfCore 字符串资源上限，包含长度前缀后的终止零。
/// 调用者必须先确认输入确实来自 `/Version` 且适用 10.8 规则。
pub fn read_layout_10_8(
    reader: &mut Reader<'_>,
    max_text_bytes: usize,
) -> io::Result<VersionLayout10_8> {
    let mut trial = reader.clone();
    let text = read_string_10_8(&mut trial, max_text_bytes)?;
    let compatibility_short = i16::from_le_bytes(trial.read_bytes(2)?.try_into().unwrap());
    let compatibility_branch = if compatibility_short < 0x12 {
        CompatibilityBranch10_8::Three
    } else {
        CompatibilityBranch10_8::Eleven
    };
    let result = VersionLayout10_8 {
        text,
        compatibility_short,
        compatibility_branch,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}
