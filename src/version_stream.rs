//! Desktop 10.8 PageLayout 打开的 OLE `/Version` 局部布局。
//!
//! 原生入口先调用 AfCore String::Read，再从同一个底层流读取 2 字节。
//! 本模块只实现严格读取子集；文本语义、short 映射、尾部字段和写入规则未知。

use crate::native_string::read_string_10_8;
use crate::reader::Reader;
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionLayout10_8 {
    /// 未经解释的候选文本；不能当作 MXD 保存版本。
    pub text: String,
    /// 保留全部位模式的有符号 short；原生代码仅与 0x12 比较，语义未闭环。
    pub compatibility_short: i16,
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
    let result = VersionLayout10_8 {
        text,
        compatibility_short,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}
