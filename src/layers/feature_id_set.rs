//! ArcMap 10.1 FeatureLayer 尾段链 FeatureIDSet 对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-featurelayer-tail.md`（2026-11-15 第九轮研究
//! 记录）。FeatureLayer::Load（`0x6f6e9940`）在 ALPC 槽与 `short(+0x1bc)`
//! 之后以请求 IID `{D5BB4B87-…}`=IFeatureIDSet 装载本类（样本键 75，
//! `/Maps:0xf63`）。字段序来自 FeatureLayer.dll 10.1.0.3035 反编译（实现与
//! 宿主同 DLL：CLSID `{D5BB4B88-…}` 数据常量 `0x6f7ac390`、IID
//! `0x6f7ac380`，esri.clsid 声明模块 FeatureLayer.dll、ecfg 归属与字节
//! 扫描互证）：
//!
//! - **FeatureIDSet::Load**（`0x6f6d97d0`，由类描述块 `{fns…, IID, CLSID}`
//!   与 Save `0x6f6d9880` 对称互证）：
//!   ① int16 版本（`1 < (short)ver` 有符号拒绝；样本 1）→ ② int32 计数 →
//!   ③ 循环 count × int32（逐元素读；要素 OID 仅为候选，样本计数 0）。Save 对称
//!   （写版本常量 1 + `this+8` 计数 + 迭代器逐元素 int32）。
//! - 类型库 `IFeatureIDSet::Add` 对应虚表 `0x6f7ac338` 的槽 5，
//!   `0x6f6d96b0` 与 Load 均调用 `0x6f6d9940` 插入同一集合；该插入器遇到
//!   已存在的值返回 1 而不增加计数，Save `0x6f6d9880` 遍历集合写出成员。
//!   因此重复流值在对象语义中只保留一个成员；接口名仍不足以证明成员必为
//!   数据源 OID。
//!
//! 调用约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头、比对 [`FEATURE_ID_SET_CLASS_ID`] 并取得键，从对象头之后的
//! 位置传入；本读取器无嵌套对象、无停止点：要么完整消费，要么失败（截断/
//! 版本超限）。负数计数原生跳过循环（`0 < count` 门），原样保留。
//!
//! 未解决问题：版本边界仅有反编译证据（样本全为版本 1）；int32 元素即要素
//! OID 为结构推断（写侧经集合迭代器写出，语义候选）。读取器 `ids` 按输入
//! 首次出现顺序呈现去重成员；原生 Save 的哈希桶遍历顺序未与该向量顺序对位。
//! 全部为静态观察，未经目标软件运行验证。

use crate::storage::object_refs::RefTable10_1;
use crate::stream::reader::Reader;
use std::collections::HashSet;
use std::io::{self, ErrorKind};

/// FeatureIDSet 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{D5BB4B88-E0A1-11D2-9F4D-00C04F6BC78E}`。证据：
/// esri.clsid 声明（progID FeatureIDSet，模块 FeatureLayer.dll）与
/// esriCarto 类型库 coclass #513；FeatureLayer.dll 数据常量 `0x6f7ac390`
/// （其前置 `0x6f7ac380` = 请求 IID `{D5BB4B87}`=IFeatureIDSet）；样本
/// `/Maps:0xf63` 头（键 75）逐字节相同。
pub const FEATURE_ID_SET_CLASS_ID: [u8; 16] = [
    0x88, 0x4b, 0xbb, 0xd5, 0xa1, 0xe0, 0xd2, 0x11, 0x9f, 0x4d, 0x00, 0xc0, 0x4f, 0x6b, 0xc7, 0x8e,
];

/// FeatureIDSet::Load 接受的最大版本：`1 < (short)ver` 有符号拒绝；样本 1。
/// 证据：FeatureLayer.dll `0x6f6d97d0`。
const MAX_FEATURE_ID_SET_VERSION: i16 = 1;

/// FeatureIDSet（FeatureLayer.dll `0x6f6d97d0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureIdSetBody10_1 {
    /// 对象头键（样本 75）。解析成功时该键已按原生语义登记进调用者的
    /// 登记表（由调用者的分派层完成；本读取器自身无嵌套对象）。
    pub key: u32,
    /// 体首 short 版本（样本 1）；有符号比较 >1 拒绝。
    pub version: u16,
    /// int32 计数（样本 0）。负数原生不进循环，原样保留。
    pub count: i32,
    /// IFeatureIDSet 的去重成员，按流内首次出现顺序呈现；原生集合不保留
    /// 重复项。成员仍原样保留有符号 int32 值，不推断它们一定是数据源 OID。
    pub ids: Vec<i32>,
}

/// 从对象头之后的位置读取 10.1 FeatureIDSet 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`FEATURE_ID_SET_CLASS_ID`] 并取得
/// 键 `key`。负数计数按原生语义跳过循环（不报错），原样保留；截断与版本
/// 超限为错误。`table` 仅在签名上保持与同族读取器一致（无嵌套对象，不
/// 触及登记表）。
pub fn read_feature_id_set_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FeatureIdSetBody10_1> {
    let _ = table;
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // ① 版本 short（`1 < (short)ver` 有符号拒绝；负值位模式一并拒绝，
    //    较原生更严）。
    let version = trial.read_u16_le()?;
    let signed = version as i16;
    if signed > MAX_FEATURE_ID_SET_VERSION || signed < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "FeatureIDSet 对象体 offset 0x{entry_offset:x}: 版本 {version} 超出 \
                 FeatureIDSet::Load 接受范围（有符号比较 >{MAX_FEATURE_ID_SET_VERSION} 拒绝；\
                 负值位模式一并拒绝）"
            ),
        ));
    }
    // ② int32 计数（负数原生不进循环，原样保留）。
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    // 在分配或逐项读取之前检查声明的元素是否全部存在。
    let item_count = count.max(0) as usize;
    let byte_count = item_count
        .checked_mul(4)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "FeatureIDSet 元素长度溢出"))?;
    if byte_count > trial.remaining() {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!("FeatureIDSet 声明 {item_count} 个元素，但剩余字节不足"),
        ));
    }
    // ③ 逐元素插入集合；原生插入器会忽略已存在的值。
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for _ in 0..item_count {
        let id = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        if seen.insert(id) {
            ids.push(id);
        }
    }

    let result = FeatureIdSetBody10_1 {
        key,
        version,
        count,
        ids,
    };
    *reader = trial;
    Ok(result)
}
