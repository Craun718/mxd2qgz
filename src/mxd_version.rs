//! OLE `/Version` 数据流的 MXD 版本识别候选。
//!
//! 先按 ArcMap 10.1 严格头部规则解析，再对照已知文本与 short 集合给出候选；
//! 10.1 规则失败时回退显式 10.8 局部布局。所有结果都是候选，未经 ArcMap
//! 或 QGIS 中的软件验证，不能据此声明文件的实际保存版本或转换能力。

use crate::reader::Reader;
use crate::version_stream::{read_arcmap_header_10_1, read_layout_10_8};
use std::io;

/// 文本读取资源上限（含终止零、不含 4 字节长度前缀），避免对超长流做无界解码。
pub const MAX_TEXT_BYTES: usize = 1024;

/// ArcMap.exe 英文资源中与保存索引对应的版本文本候选；资源映射未经运行时确认。
pub const KNOWN_TEXTS: [&str; 5] = ["8.3", "9.0", "9.2", "10.0", "10.1"];

/// ArcMap 10.1 保存入口静态表的六个有符号 short；索引与版本的对应关系未运行时确认。
pub const KNOWN_SHORTS: [i16; 6] = [10, 13, 14, 16, 18, 19];

/// `/Version` 识别命中的分支；这只是候选归类，不代表已验证的版本。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MxdVersionBranch {
    /// 按 ArcMap 10.1 头部规则完整解析，且文本与 short 都在已知集合。
    ArcMap10_1KnownCandidate,
    /// 按 ArcMap 10.1 头部规则完整解析，但文本或 short 不在已知集合。
    ArcMap10_1Unrecognized,
    /// ArcMap 10.1 头部规则失败后的显式 10.8 局部布局候选。
    Layout10_8Fallback,
}

impl MxdVersionBranch {
    /// 分支的固定说明，保留候选与未验证边界。
    pub fn description(self) -> &'static str {
        match self {
            Self::ArcMap10_1KnownCandidate => "ArcMap 10.1 头部规则匹配候选",
            Self::ArcMap10_1Unrecognized => "按 ArcMap 10.1 头部规则解析，但文本或 short 未识别",
            Self::Layout10_8Fallback => "仅 10.8 布局候选（不满足 ArcMap 10.1 头部规则）",
        }
    }
}

const KNOWN_CANDIDATE_NOTE: &str = "文本与 short 都落在 ArcMap 10.1 静态候选集合；资源文本映射未经运行时确认，识别结果未在目标软件中验证";
const UNRECOGNIZED_NOTE: &str = "未识别文本/short；原生接受任意文本，资源映射未运行时确认";
const LAYOUT_10_8_NOTE: &str =
    "仅 10.8 布局候选；不满足 ArcMap 10.1 头部规则，对象计数不由该布局读取，适用版本范围未验证";

/// `/Version` 识别报告；全部字段都是候选观察，未经目标软件验证。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxdVersionReport {
    /// 命中的识别分支。
    pub branch: MxdVersionBranch,
    /// 候选版本文本；原生接受任意文本，不证明文件的实际保存版本。
    pub text: String,
    /// 有符号 short 的完整位模式；到具体版本的映射未运行时确认。
    pub compatibility_short: i16,
    /// short 大于 11 时的对象流表计数；10.8 布局分支不读取该字段，恒为 `None`。
    /// 这是容量预估信息，不能解释为图层数。
    pub object_count: Option<i32>,
    /// 识别读取之后剩余的未知字节数。
    pub remaining_bytes: usize,
    /// 适用范围与未验证声明的固定说明，供调用方直接输出。
    pub note: &'static str,
}

/// 从 `/Version` 数据流的完整字节识别 MXD 版本候选。
///
/// 用 [`MAX_TEXT_BYTES`] 的文本上限尝试 ArcMap 10.1 严格头部规则：成功且
/// 文本与 short 都在已知集合时报告匹配候选；解析成功但集合之外时仍完整
/// 报告字段。10.1 规则失败（含 short 超限、截断、非法 UTF-16）时回退显式
/// 10.8 局部布局并标注候选范围；两者都失败时返回带两个原因的错误。
/// 识别结果是候选，未在 ArcMap 或 QGIS 中验证。
pub fn identify_version(version_stream: &[u8]) -> io::Result<MxdVersionReport> {
    let mut reader = Reader::new(version_stream);
    match read_arcmap_header_10_1(&mut reader, MAX_TEXT_BYTES) {
        Ok(header) => {
            let text_known = KNOWN_TEXTS.contains(&header.prefix.text.as_str());
            let short_known = KNOWN_SHORTS.contains(&header.prefix.compatibility_short);
            let (branch, note) = if text_known && short_known {
                (
                    MxdVersionBranch::ArcMap10_1KnownCandidate,
                    KNOWN_CANDIDATE_NOTE,
                )
            } else {
                (MxdVersionBranch::ArcMap10_1Unrecognized, UNRECOGNIZED_NOTE)
            };
            return Ok(MxdVersionReport {
                branch,
                text: header.prefix.text,
                compatibility_short: header.prefix.compatibility_short,
                object_count: header.object_count,
                remaining_bytes: header.remaining_bytes,
                note,
            });
        }
        Err(error_10_1) => {
            let mut reader = Reader::new(version_stream);
            match read_layout_10_8(&mut reader, MAX_TEXT_BYTES) {
                Ok(layout) => Ok(MxdVersionReport {
                    branch: MxdVersionBranch::Layout10_8Fallback,
                    text: layout.text,
                    compatibility_short: layout.compatibility_short,
                    object_count: None,
                    remaining_bytes: layout.remaining_bytes,
                    note: LAYOUT_10_8_NOTE,
                }),
                Err(error_10_8) => Err(io::Error::new(
                    error_10_1.kind(),
                    format!(
                        "无法按 ArcMap 10.1 头部规则或 10.8 局部布局识别 /Version；10.1 规则：{error_10_1}；10.8 布局：{error_10_8}"
                    ),
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn combined_failure_reports_both_reasons_and_keeps_10_1_kind() {
        // 空流：两个读取器都因缺长度前缀失败，错误保留 10.1 的 UnexpectedEof。
        let error = identify_version(&[]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        assert!(error.to_string().contains("10.1 规则"));
        assert!(error.to_string().contains("10.8 布局"));
        // 奇数长度前缀在两个子集中都是非法数据。
        let error = identify_version(&[3, 0, 0, 0, 0x41, 0]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn garbage_without_terminator_or_short_fails_both_readers() {
        let mut garbage = vec![0x40, 0, 0, 0];
        garbage.extend([0x41; 0x40]);
        let error = identify_version(&garbage).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("终止零") || error.to_string().contains("short"));
    }
}
