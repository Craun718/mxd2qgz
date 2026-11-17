//! ArcMap 10.1 Hyperlink（coclass Hyperlink，C++ 实现类 FeatureHyperlink，
//! FeatureLayer.dll）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-hyperlink.md`（本轮研究记录）。原生实现为
//! **FeatureLayer.dll 10.1.0.3035**（SHA-256 `2baf8706…`，ImageBase
//! `0x6f280000`）——修正既有第五轮记录的「3036D35x 家族常量仅存在于
//! Map.dll」：该结论只对 `{3036D35F}` IFeatureBookmark 成立；coclass
//! Hyperlink `{3036D35E}` 与请求 IID IHyperlink `{3036D35D}` 的常量均在
//! FeatureLayer.dll（`DAT_6f7ac2a4`/`DAT_6f7ac2b4` 相邻），Map.dll 数据段
//! 两者皆无。类身份三方互证：`esri.clsid.ecfg` 声明 `{3036D35E}` progID
//! `Hyperlink`（模块 FeatureLayer.dll，esriCarto 前缀）；esriCarto 类型库
//! coclass #506 Hyperlink（接口 IHyperlink `{3036D35D}` = FL 后继段计数循
//! 环的请求 IID）；FeatureLayer.dll GetClassID `0x6f6d9310` 以立即数对
//! `0x11D0EDE53036D35E`/`0x2A73EC090008FE87` 写出该 CLSID，且 IPersist
//! Stream 虚表 `0x6f7ac284` 槽 +0x0c 直接指向它。
//!
//! ## 定位链（RTTI TD→COL→虚表→GetClassID，10.1 静态映像 VA；判据
//! [C]==0 且 [C+0xc]==TD）
//!
//! RTTI `.?AVFeatureHyperlink@@`（TypeDescriptor `0x6f7d2780`；类名与
//! progID 不同——与 FdbFeatureClassName 同款）→ 4 个子接口 COL
//! （`0x6f7b6678`/`0x6f7b668c`/`0x6f7b66a0`/`0x6f7b66b4`）→ 虚表
//! `0x6f7ac284`（首 COL 前置）：槽 +0x0c = GetClassID `0x6f6d9310`（三方
//! 互证闭环）、+0x14 = Load `0x6f6d9170`、+0x18 = Save `0x6f6d9210`（虚表
//! 槽字节直读核对，无调整器）。
//!
//! ## Load `0x6f6d9170` 字段序（版本 short 的有符号比较，>1 拒绝
//! `0x80004005`；Save `0x6f6d9210` 写静态版本常量 `DAT_6f77ba84`=1，对称）
//!
//! | # | 字段 | 版本门 | 对象偏移 | 语义 |
//! | --- | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有 | — | >1 拒绝；负值位模式原生同样通过门（读取器拒绝，较原生更严） |
//! | ② | int32 | 恒有 | this+0x4 | IHyperlink::FeatureId |
//! | ③ | AfCore String::Read | 恒有 | this+0x8 | IHyperlink::Link |
//! | ④ | int32 | 恒有 | this+0xc | IHyperlink::LinkType（原样保留枚举值） |
//!
//! 体 = short + int32 + String + int32，无嵌套对象、无版本分支字段（版本
//! 门只做拒绝）。Save 侧有写向专用规范化：docVersion <3 时若链接非 URL 且
//! 含「?」则截断至「?」前（`FUN_6f6d9350`；读侧无对应分支，10.1 docVersion
//! =3 写满形态）。
//!
//! 样本 `samples/测试底图.mxd` 的 /Maps 流中 FL 后继段 IHyperlink 计数循环
//! 恒 0（`0x5de`=0 等），本体 **无样本字节佐证**，字段序全部来自反编译，
//! 仅合成输入测试覆盖。
//!
//! 这是仅覆盖上述分支的严格安全子集：要求完整输入，任何失败不推进调用者
//! 游标、不修改登记表；已接入 FeatureLayer 后继段读取器，人工目视验收未做。
//!
//! 字段与接口的绑定见 `docs/mxd-round2-dataset-format-20261117.md`。
//! `FUN_6f6d9350` 的截断仅按反编译记
//! 录，未运行验证；版本 0 短体形态无样本；全部结论未运行 ArcMap 验证。

use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// Hyperlink::Load 接受的最大版本：反编译 `1 < (short)ver` 有符号拒绝；写
/// 侧 Save 版本常量 `DAT_6f77ba84` 恒 1。
const MAX_HYPERLINK_VERSION: i16 = 1;

/// Hyperlink 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{3036D35E-EDE5-11D0-87FE-080009EC732A}`。证据：
/// `esri.clsid.ecfg` 声明（模块 FeatureLayer.dll，progID `Hyperlink`）；
/// FeatureLayer.dll GetClassID `0x6f6d9310` 写出该 CLSID 立即数；常量字节
/// 位于 `DAT_6f7ac2b4`（与 FL::Load 计数循环请求 IID
/// `DAT_6f7ac2a4={3036D35D}` IHyperlink 相邻）。esri.clsid 类名 Hyperlink
/// 与 C++ RTTI 类名 FeatureHyperlink 不同（命名差异不影响流内类身份）。
pub const HYPERLINK_CLASS_ID: [u8; 16] = [
    0x5e, 0xd3, 0x36, 0x30, 0xe5, 0xed, 0xd0, 0x11, 0x87, 0xfe, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// Hyperlink（FeatureLayer.dll `0x6f6d9170`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct HyperlinkBody10_1 {
    /// 对象头键（槽位分派传入；样本 IHyperlink 计数恒 0，无实例）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝，写侧恒 1。
    pub version: u16,
    /// int32 → this+0x4，对应 IHyperlink::FeatureId；字段名保留以兼容调用者。
    pub type_04: i32,
    /// AfCore String → this+0x8，对应 IHyperlink::Link。
    pub link: String,
    /// int32 → this+0xc，对应 IHyperlink::LinkType，未知枚举值原样保留。
    pub int32_0c: i32,
}

/// 从对象头之后的位置读取 10.1 Hyperlink 对象体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`HYPERLINK_CLASS_ID`] 并取得键
/// `key`。`max_bytes` 是资源限额，应用于 ③ 的声明字符串长度（含终止零、
/// 不含 4 字节前缀），在任何读取与分配之前检查；`usize::MAX` 表示不额外设
/// 限。`table` 仅在签名上保持一致（无嵌套对象条目）。
pub fn read_hyperlink_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<HyperlinkBody10_1> {
    let _ = table; // 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short：原生有符号比较 `1 < (short)ver` 拒绝；负值位模式一并
    //    拒绝（较原生更严：负值版本无业务意义，写侧恒 1）。
    let version = trial.read_u16_le()?;
    let signed = version as i16;
    if signed > MAX_HYPERLINK_VERSION || signed < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Hyperlink 对象体 offset 0x{entry_offset:x}: 版本 {version} 超出 \
                 Hyperlink::Load 接受范围（有符号比较 >1 拒绝；负值位模式一并拒绝）"
            ),
        ));
    }
    // ② int32 → this+0x4。
    let type_04 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    // ③ AfCore String → this+0x8。
    let link = read_string_10_1(&mut trial, max_bytes)?;
    // ④ int32 → this+0xc。
    let int32_0c = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let result = HyperlinkBody10_1 {
        key,
        version,
        type_04,
        link,
        int32_0c,
    };
    *reader = trial;
    Ok(result)
}
