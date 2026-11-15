//! ArcMap 10.1 写侧版本表（`AfStreamGetSaveVersion` 查表用的静态表）与原生查表
//! 语义的纯常量封装。
//!
//! 证据来源：`docs/arcmap-10.1-save-side.md`（2026-11-16 E 轮，写侧专项）。全部
//! 表字节经本机 pyghidra 从对应 10.1.0.3035 二进制的 .rdata 逐一读出；查表语义经
//! AfCore.dll 导出函数 `AfStreamGetSaveVersion`（`0x68a847f0`）自身反编译闭合：
//!
//! ```text
//! short AfStreamGetSaveVersion(esriArcGISVersion doc_ver, short n, short const* table)
//!   table == null || n == 0        -> -1
//!   n - 1 < (int)doc_ver           -> table[n - 1]   // 超表尾截断到末项
//!   否则                            -> table[doc_ver]
//! ```
//!
//! `doc_ver` 为保存索引（`esriArcGISVersion` 枚举 0..5，对应 8.3/9.0/9.2/9.3/10.0/
//! 10.1；来源流为 `AfStreamGetDocumentVersion`（`0x68a84760`）：结果初值 5，流携带
//! IDocumentVersion `{ECC43C55-0148-4EC1-BF87-B9A183C5DC98}` 时经其 get 回填）。
//! 表项 `0xFFFF`（本模块以 `i16` 的 `-1` 表示）＝该保存版本**整类不写**：各类
//! Save 在写出任何字段之前检查版本值并直接返回成功（如 EditTemplate::Save
//! `0x6f6ce3c0` 的 `if (ver == 0xffff) return 0;`）。
//!
//! 本模块为纯常量与研究封装，**不接入任何解析/写出行**；各 Save 的完整字段序与
//! 读侧交互见研究文档第二至六节。未经目标软件运行验证。

/// `AfStreamGetSaveVersion` 的 0xFFFF 表项（该保存版本整类不写）。
pub const SAVE_VERSION_SKIP: i16 = -1;

/// `doc_ver` 保存索引常量（`esriArcGISVersion` 枚举）。
pub mod doc_version {
    /// ArcGIS 8.3。
    pub const V83: i32 = 0;
    /// ArcGIS 9.0。
    pub const V90: i32 = 1;
    /// ArcGIS 9.2。
    pub const V92: i32 = 2;
    /// ArcGIS 9.3。
    pub const V93: i32 = 3;
    /// ArcGIS 10.0。
    pub const V10: i32 = 4;
    /// ArcGIS 10.1（也是 `AfStreamGetDocumentVersion` 的缺省值）。
    pub const V10_1: i32 = 5;
}

/// Map（数据框）体版本表。
///
/// Map.dll `DAT_79480dac`（SHA-256 `1c0675cc…`），N=6：
/// docVer 0→22、1→29、2→30、3→31、4→45、≥5→51。仓库样本 /Maps 首对象体版本
/// 51（=0x33）与 docVer=5 列一致（读侧样本对位，辅助观察）。
pub const MAP_BODY: [i16; 6] = [22, 29, 30, 31, 45, 51];

/// FeatureLayer 体版本表。
///
/// FeatureLayer.dll `DAT_6f767d24`（SHA-256 `2baf8706…`），N=6：
/// docVer 0→17、1→22、2→23、3→25、4→34、≥5→34。样本体版本 34 ✓。
/// 无 0xFFFF 档——8.3 起每档都写 FeatureLayer 体；docVer 0/1/2/3 各档按
/// FeatureLayer::Save（`0x6f6eb1b0`）内的降级截断阶梯提前收尾（见研究文档第二节）。
pub const FEATURE_LAYER_BODY: [i16; 6] = [17, 22, 23, 25, 34, 34];

/// FDOGraphicsLayer（注记图层）体版本表。
///
/// AnnoLayer.dll `DAT_6b1ab0ac`（SHA-256 `b487f3e0…`），N=2：
/// docVer 0→5、≥1→11。与 FDOGraphicsSublayer 表（[`FDO_GRAPHICS_SUBLAYER_BODY`]）
/// 在 .rdata 中背靠背。样本体版本 11 ✓。8.3 写版本 5 体（Save `0x6b195b90` 在委派
/// 写出后 `docVer==0` 提前返回）；**任何档都不产出版本 <5**——版本 4（旧布局 + 收尾
/// String）只来自 8.3 之前的写入器（读侧版本 4 探测的历史对象，见研究文档第六节）。
pub const FDO_GRAPHICS_LAYER_BODY: [i16; 2] = [5, 11];

/// FDOGraphicsSublayer（注记图层元素记录）体版本表。
///
/// AnnoLayer.dll `DAT_6b1ab0a8`（SHA-256 `b487f3e0…`），N=2：
/// docVer 0→**0xFFFF（整类不写）**、≥1→2。Save `0x6b1a1c40` 在版本 short 之外无
/// 版本门；版本 1 从不被 10.1 产出（读侧 ==1 分支仅服务更老写入器的流）。
pub const FDO_GRAPHICS_SUBLAYER_BODY: [i16; 2] = [SAVE_VERSION_SKIP, 2];

/// EditTemplateManager（要素模板管理器，图层扩展段宿主）体版本表。
///
/// FeatureLayer.dll `DAT_6f77bf74`（SHA-256 `2baf8706…`），N=6：
/// docVer ≤3→**0xFFFF（整类不写，连计数都不写）**、4→1、≥5→2。样本 ETM 体版本 2 ✓。
/// docVer ≤3 的 0 字节体使扩展条目尺寸为 0，读侧 `ExtensionHelper::LoadExtensions`
/// 对尺寸 0 的已知类条目不调 Load（见研究文档第五节）。
pub const EDIT_TEMPLATE_MANAGER_BODY: [i16; 6] = [
    SAVE_VERSION_SKIP,
    SAVE_VERSION_SKIP,
    SAVE_VERSION_SKIP,
    SAVE_VERSION_SKIP,
    1,
    2,
];

/// EditTemplate（要素模板）体版本表。
///
/// FeatureLayer.dll `DAT_6f77c02c`（SHA-256 `2baf8706…`），N=5：
/// docVer ≤3→**0xFFFF（整类不写）**、4→2、≥5→2（截断到末项）。样本体版本 2 ✓。
/// FFFF 分支实际不可达：唯一宿主 EditTemplateManager::Save 在 docVer ≤3 时不进入
/// 模板循环（研究文档第五节）。
pub const EDIT_TEMPLATE_BODY: [i16; 5] = [
    SAVE_VERSION_SKIP,
    SAVE_VERSION_SKIP,
    SAVE_VERSION_SKIP,
    SAVE_VERSION_SKIP,
    2,
];

/// 按 `AfStreamGetSaveVersion`（AfCore `0x68a847f0`）的原生语义查表。
///
/// `doc_ver >= table.len()` 时返回末项（原生截断），否则返回 `table[doc_ver]`。
/// 与原生的已知偏离（函数文档注明，较原生更严）：`doc_ver` 为负时原生会以负索引
/// 越界读表，此处返回 [`None`]；空表原生返回 −1，此处返回 [`None`]（[`None`] 与
/// 「表项 0xFFFF＝整类不写」是不同语义，后者为 `Some([`SAVE_VERSION_SKIP`])`）。
pub fn save_version_for(table: &[i16], doc_ver: i32) -> Option<i16> {
    if table.is_empty() || doc_ver < 0 {
        return None;
    }
    let index = if doc_ver as usize >= table.len() {
        table.len() - 1
    } else {
        doc_ver as usize
    };
    Some(table[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_matches_native_semantics() {
        // 末项截断。
        assert_eq!(save_version_for(&MAP_BODY, 5), Some(51));
        assert_eq!(save_version_for(&MAP_BODY, 9), Some(51));
        assert_eq!(save_version_for(&FEATURE_LAYER_BODY, 5), Some(34));
        assert_eq!(save_version_for(&FEATURE_LAYER_BODY, 6), Some(34));
        // 表内直取。
        assert_eq!(
            (0..6)
                .map(|d| save_version_for(&MAP_BODY, d))
                .collect::<Vec<_>>(),
            vec![Some(22), Some(29), Some(30), Some(31), Some(45), Some(51)]
        );
        assert_eq!(
            (0..6)
                .map(|d| save_version_for(&FEATURE_LAYER_BODY, d))
                .collect::<Vec<_>>(),
            vec![Some(17), Some(22), Some(23), Some(25), Some(34), Some(34)]
        );
        // 较原生更严的拒绝（原生为越界读/−1）。
        assert_eq!(save_version_for(&MAP_BODY, -1), None);
        assert_eq!(save_version_for(&[], 0), None);
    }

    #[test]
    fn skip_entries_mean_write_nothing() {
        // EditTemplate：前四档（8.3/9.0/9.2/9.3）整类不写；10.0/10.1 恒写 2。
        assert_eq!(
            (0..=5)
                .map(|d| save_version_for(&EDIT_TEMPLATE_BODY, d))
                .collect::<Vec<_>>(),
            vec![
                Some(SAVE_VERSION_SKIP),
                Some(SAVE_VERSION_SKIP),
                Some(SAVE_VERSION_SKIP),
                Some(SAVE_VERSION_SKIP),
                Some(2),
                Some(2)
            ]
        );
        // EditTemplateManager：前四档不写；10.0 写 1；10.1 写 2。
        assert_eq!(
            (0..=5)
                .map(|d| save_version_for(&EDIT_TEMPLATE_MANAGER_BODY, d))
                .collect::<Vec<_>>(),
            vec![
                Some(SAVE_VERSION_SKIP),
                Some(SAVE_VERSION_SKIP),
                Some(SAVE_VERSION_SKIP),
                Some(SAVE_VERSION_SKIP),
                Some(1),
                Some(2)
            ]
        );
        // FDOGraphicsSublayer：8.3 不写；9.0 起恒写 2。
        assert_eq!(
            (0..=5)
                .map(|d| save_version_for(&FDO_GRAPHICS_SUBLAYER_BODY, d))
                .collect::<Vec<_>>(),
            vec![
                Some(SAVE_VERSION_SKIP),
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                Some(2)
            ]
        );
        // FDOGraphicsLayer：8.3 写 5（体止于委段），9.0 起恒写 11。
        assert_eq!(
            (0..=5)
                .map(|d| save_version_for(&FDO_GRAPHICS_LAYER_BODY, d))
                .collect::<Vec<_>>(),
            vec![Some(5), Some(11), Some(11), Some(11), Some(11), Some(11)]
        );
    }

    #[test]
    fn skip_constant_is_ffff_bit_pattern() {
        // 表字节为小端 0xFFFF（与二进制 .rdata 读出一致）。
        assert_eq!(SAVE_VERSION_SKIP.to_le_bytes(), [0xff, 0xff]);
        assert_eq!(
            EDIT_TEMPLATE_BODY
                .iter()
                .map(|v| v.to_le_bytes())
                .fold(Vec::new(), |mut acc, b| {
                    acc.extend_from_slice(&b);
                    acc
                }),
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02, 0x00]
        );
    }

    #[test]
    fn sample_anchors_doc_version_5() {
        // 仓库样本（10.1 保存，docVer=5）各对象体首 short 与查表值一致。
        assert_eq!(save_version_for(&MAP_BODY, 5), Some(51));
        assert_eq!(save_version_for(&FEATURE_LAYER_BODY, 5), Some(34));
        assert_eq!(save_version_for(&FDO_GRAPHICS_LAYER_BODY, 5), Some(11));
        assert_eq!(save_version_for(&EDIT_TEMPLATE_BODY, 5), Some(2));
        assert_eq!(save_version_for(&EDIT_TEMPLATE_MANAGER_BODY, 5), Some(2));
        assert_eq!(save_version_for(&FDO_GRAPHICS_SUBLAYER_BODY, 5), Some(2));
    }
}
