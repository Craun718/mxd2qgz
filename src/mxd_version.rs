//! OLE `/Version` 数据流的 MXD 版本识别候选。
//!
//! 先按 ArcMap 10.1 严格头部规则解析，再对照已知文本与 short 集合给出候选，
//! 并按静态闭合的「文本↔保存索引↔short」映射给出候选保存索引；10.1 规则
//! 失败时回退显式 10.8 局部布局。所有结果都是候选，未经 ArcMap 或 QGIS 中
//! 的软件验证，不能据此声明文件的实际保存版本或转换能力。

use crate::reader::Reader;
use crate::version_stream::{read_arcmap_header_10_1, read_layout_10_8};
use std::io;

/// 文本读取资源上限（含终止零、不含 4 字节长度前缀），避免对超长流做无界解码。
pub const MAX_TEXT_BYTES: usize = 1024;

/// 静态闭合的「文本 ↔ 保存索引（= `esriArcGISVersion` 枚举值）↔ short」候选表。
/// 注意：文本表缺 9.3（资源 ID `0x11a` 为打印错误文本的异常候选），因此文本
/// 下标与 short 下标不通用，必须按下表三元组配对，不能用数组下标互推。
/// 映射为静态证据，未经运行时或目标软件验证。
const CANDIDATE_TABLE: [(&str, usize, i16); 5] = [
    ("8.3", 0, 10),
    ("9.0", 1, 13),
    ("9.2", 2, 14),
    ("10.0", 4, 18),
    ("10.1", 5, 19),
];

/// ArcMap.exe 英文资源中与保存索引对应的版本文本候选（资源 ID `0x14d`/`0x173`/
/// `0x186`/`0xe804`/`0x193`）。索引 3（esriArcGISVersion93）无文本条目，
/// 不得把未知文本映射为 9.3。
pub const KNOWN_TEXTS: [&str; 5] = ["8.3", "9.0", "9.2", "10.0", "10.1"];

/// ArcMap 10.1 保存入口静态表的六个有符号 short；下标即 `esriArcGISVersion`
/// 枚举值（83/90/92/93/10/101，Current=101=5）。静态闭合见
/// docs/arcmap-10.1-native-baseline.md「10.1 磁盘版本映射、属性来源与回填取值
/// （2026-11-14）」；对应关系未经运行时或目标软件验证。
pub const KNOWN_SHORTS: [i16; 6] = [10, 13, 14, 16, 18, 19];

/// 按「文本与 short 同时命中 [`CANDIDATE_TABLE`] 中同一三元组」给出候选保存
/// 索引（即 `esriArcGISVersion` 枚举值）。文本或 short 任一未知、或组合不属
/// 同一索引时返回 `None`；索引 3（9.3）因资源文本异常无法由文本命中，只能
/// 保持 `None`。这是静态映射候选，不能据此声明文件的实际保存版本。
pub fn candidate_save_index(text: &str, compatibility_short: i16) -> Option<usize> {
    CANDIDATE_TABLE
        .iter()
        .find(|(known_text, _, known_short)| {
            *known_text == text && *known_short == compatibility_short
        })
        .map(|&(_, index, _)| index)
}

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

const KNOWN_CANDIDATE_NOTE: &str = "文本与 short 命中静态候选映射（文本↔保存索引↔short，见 native-baseline 2026-11-14 节）；映射未经运行时确认，识别结果未在目标软件中验证";
const UNRECOGNIZED_NOTE: &str = "未识别文本/short；原生接受任意文本，未知文本不映射为 9.3";
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
    /// 静态闭合映射下的候选保存索引（= `esriArcGISVersion` 枚举值，见
    /// [`candidate_save_index`]）；10.8 布局分支恒为 `None`。
    /// 不能据此声明文件的实际保存版本。
    pub candidate_save_index: Option<usize>,
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
            let candidate_save_index =
                candidate_save_index(&header.prefix.text, header.prefix.compatibility_short);
            return Ok(MxdVersionReport {
                branch,
                text: header.prefix.text,
                compatibility_short: header.prefix.compatibility_short,
                object_count: header.object_count,
                candidate_save_index,
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
                    candidate_save_index: None,
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

    /// 按已闭合的 10.1 头部布局构造合成 `/Version` 字节：4 字节长度（含终止零）、
    /// UTF-16LE 文本、2 字节 short、short > 11 时的 4 字节对象计数。
    fn header_bytes(text: &str, short: i16, count: Option<i32>) -> Vec<u8> {
        let mut encoded: Vec<u8> = text
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect();
        encoded.extend_from_slice(&[0, 0]);
        let mut out = Vec::new();
        out.extend_from_slice(&u32::try_from(encoded.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&encoded);
        out.extend_from_slice(&short.to_le_bytes());
        if let Some(count) = count {
            out.extend_from_slice(&count.to_le_bytes());
        }
        out
    }

    #[test]
    fn known_pairs_map_to_esri_arcgis_version_index() {
        // 样本三要素（10.1/19）→ 索引 5（Current=101）；其余文本/short 对逐一核对。
        let cases = [
            ("8.3", 10, None, Some(0)),
            ("9.0", 13, Some(1), Some(1)),
            ("9.2", 14, Some(2), Some(2)),
            ("10.0", 18, Some(3), Some(4)),
            ("10.1", 19, Some(2267), Some(5)),
        ];
        for (text, short, count, expected) in cases {
            let report = identify_version(&header_bytes(text, short, count)).unwrap();
            assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
            assert_eq!(report.candidate_save_index, expected, "文本 {text}");
        }
    }

    #[test]
    fn mismatched_known_pair_keeps_index_none() {
        // 文本与 short 各自已知但组合不属于同一索引：保留字段，不给候选索引。
        let report = identify_version(&header_bytes("10.1", 13, Some(4))).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1KnownCandidate);
        assert_eq!(report.candidate_save_index, None);
    }

    #[test]
    fn unknown_text_is_never_mapped_to_9_3() {
        // 索引 3 的英文回退文本（0x11a 打印错误文本）+ short 16 不得映射为 9.3。
        let error_text = "An unexpected error occured while printing the document.";
        let report = identify_version(&header_bytes(error_text, 16, Some(7))).unwrap();
        assert_eq!(report.branch, MxdVersionBranch::ArcMap10_1Unrecognized);
        assert_eq!(report.candidate_save_index, None);
    }

    #[test]
    fn candidate_save_index_helper_matches_pair_only() {
        assert_eq!(candidate_save_index("10.1", 19), Some(5));
        assert_eq!(candidate_save_index("10.1", 16), None);
        assert_eq!(candidate_save_index("9.3", 16), None);
        assert_eq!(candidate_save_index("任意文本", 19), None);
    }

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
