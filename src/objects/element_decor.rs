//! ArcMap 10.1 GraphicElements.dll 元素装饰实现类（IBorder/IBackground/
//! IShadow 槽的流内实现类族）对象体严格读取子集与槽位分派。
//!
//! 证据来源：`docs/arcmap-10.1-display-slots-round3.md`（G 轮研究记录）。三
//! 类宿主经「esri.clsid 声明模块 → RTTI TypeDescriptor → COL → IPersistStream
//! 虚表 → GetClassID 读 `.rdata` CLSID 常量」链在 GraphicElements.dll
//! 10.1.0.3035（SHA-256 `f24451c8…f77ae9`，ImageBase `0x75030000`）中逐类闭
//! 环：
//!
//! - **SymbolBorder** `{A5D0F017-62DD-11D2-87BE-0000F8751720}`：IPersistStream
//!   虚表 `0x75072c04`，GetClassID `0x7505ec40`（读常量 `0x750710d8`）、Load
//!   `0x7505ec70`、Save `0x7505ed40`。
//! - **SymbolBackground** `{1BAA33E9-E13B-11D2-B868-00600802E603}`：虚表
//!   `0x75072a64`，GetClassID `0x7505c4e0`（读常量 `0x750710e8`）、Load
//!   `0x7505c510`、Save `0x7505c5e0`。
//! - **SymbolShadow** `{A8861E66-57AA-47D0-AAF8-B288B4FD5240}`：虚表
//!   `0x75072d74`，GetClassID `0x750601b0`（读常量 `0x750710f8`）、Load
//!   `0x750601e0`、Save `0x750602b0`。
//!
//! ## 字段序（Load 反编译 + Save 对称互证；证据强度：确定的；样本 0 实例）
//!
//! **SymbolBorder::Load `0x7505ec70`**（版本 `2 < (short)ver` 拒绝；Save 恒写
//! 版本 2，常量 `DAT_7506e7b8`=2）：
//!
//! 1. int16 版本；2. double → this−0x1c（尺寸候选，装载即复制到 this−0x14）；
//! 3. LoadObject（请求 IID `DAT_7507143c` = ILineSymbol `{E6BDAA7D-…}`）→
//!    边界线符号槽（Save 槽 this−0xc；符号全闭合类分派）；
//! 4. short → this−8；
//! 5. 版本 ≥2：double → this−0x14。
//!
//! **SymbolBackground::Load `0x7505c510`**（版本 ≤2；Save 恒写版本 2，常量
//! `DAT_7506e7bc`=2）：与 SymbolBorder 同构，但符号槽请求 IID 为
//! `DAT_7506ebcc` = **IFillSymbol** `{E6BDAA7E-…}`（背景填充符号槽）。
//!
//! **SymbolShadow::Load `0x750601e0`**（版本 `1 < (short)ver` 拒绝；Save 恒写
//! 版本 1，常量 `DAT_7506e7b4`=1）：
//!
//! 1. int16 版本；2. LoadObject（IFillSymbol）→ 阴影填充符号槽（Save 槽
//!    this−0xc）；3. short → this−8；4. double → this−0x1c；5. double →
//!    this−0x14。
//!
//! 宿主槽位：TextElement 版本 ==7 装饰块（CartoXProvider.dll TextElement::Load
//! `0x6ce70e50`）依次请求 IBorder `{E4240307-DD41-11D2-B868-00600802E603}`、
//! IBackground `{E4240309-…}`、IShadow `{A9258348-1753-4B99-820A-FCF74429ABC1}`
//! （三个请求 IID 常量 `0x6cee3f38`/`0x6cee3f48`/`0x6cee3f58` 逐字节读核），
//! 其后为 double + int32 + double 标量尾段——10.1 Save 恒写版本 8（版本表
//! `DAT_6ced7ef8` = docVer 0→6、≥1→8），==7 精确块是历史文档形态。三类的流
//! 内实现类即本模块三类（esri.clsid 中 IBorder/IBackground/IShadow 的
//! GraphicElements 实现族；MapElements.dll 另有网格边框类，未在本轮闭合范
//! 围）。
//!
//! ## 槽位分派
//!
//! [`read_border_slot_10_1`] / [`read_background_slot_10_1`] /
//! [`read_shadow_slot_10_1`]：分别对应三个装饰槽——空标记与回引可消费，新对
//! 象按类分派对应实现类，其余实现类回退到槽位起点、不保留该键，以停止原因
//! 返回。建议接入点：`element_body.rs` 的 `read_text_element_body_10_1` 版本
//! ==7 块三处分类槽替换为本分派（接线归整合轮）。
//!
//! 事务性与限额约定同仓内其他读取器；样本 0 实例，全部结论无样本字节佐证，
//! 单元测试仅合成输入，不代表原生已验证。

use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, SymbolStop10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// SymbolBorder 类标识。证据：esri.clsid + GetClassID `0x7505ec40` 读常量
/// `0x750710d8` 互证。
pub const SYMBOL_BORDER_CLASS_ID: [u8; 16] = [
    0x17, 0xf0, 0xd0, 0xa5, 0xdd, 0x62, 0xd2, 0x11, 0x87, 0xbe, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// SymbolBackground 类标识。证据：esri.clsid + GetClassID `0x7505c4e0` 读常
/// 量 `0x750710e8` 互证。
pub const SYMBOL_BACKGROUND_CLASS_ID: [u8; 16] = [
    0xe9, 0x33, 0xaa, 0x1b, 0x3b, 0xe1, 0xd2, 0x11, 0xb8, 0x68, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// SymbolShadow 类标识。证据：esri.clsid + GetClassID `0x750601b0` 读常量
/// `0x750710f8` 互证。
pub const SYMBOL_SHADOW_CLASS_ID: [u8; 16] = [
    0x66, 0x1e, 0x86, 0xa8, 0xaa, 0x57, 0xd0, 0x47, 0xaa, 0xf8, 0xb2, 0x88, 0xb4, 0xfd, 0x52, 0x40,
];

const MAX_BORDER_VERSION: i16 = 2;
const MAX_BACKGROUND_VERSION: i16 = 2;
const MAX_SHADOW_VERSION: i16 = 1;

/// 读取流内 int16 版本并按原生有符号上限校验（负值位模式拒绝，较原生更严）。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    label: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let raw = trial.read_u16_le()?;
    let signed = raw as i16;
    if signed < 0 || signed > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{label} 对象体 offset 0x{entry_offset:x}: 版本位模式 {raw}（0x{raw:04x}）\
                 超出原生接受范围 0..={max}"
            ),
        ));
    }
    Ok(raw)
}

/// 读取 8 字节 LE u64（Reader 无 u64 原语，本模块用原样位模式保存 double）。
fn read_u64_le(trial: &mut Reader<'_>) -> io::Result<u64> {
    Ok(u64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap()))
}

/// 符号槽读取的公共尾段：short +（版本 ≥2 时）double。
fn read_decor_tail_10_1(trial: &mut Reader<'_>, version: u16) -> io::Result<(u16, Option<u64>)> {
    let short_1 = trial.read_u16_le()?;
    let double_2 = if version >= 2 {
        Some(read_u64_le(trial)?)
    } else {
        None
    };
    Ok((short_1, double_2))
}

/// SymbolBorder 体（Load `0x7505ec70`；符号槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolBorderBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝。
    pub version: u16,
    /// double → this−0x1c（尺寸候选）。
    pub double_1: u64,
    /// 边界线符号槽（Save 槽 this−0xc）。
    pub symbol: SymbolSlotFull10_1,
    /// short → this−8。
    pub short_1: u16,
    /// 版本 ≥2：double → this−0x14。
    pub double_2: Option<u64>,
    /// 嵌套停止。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SymbolBorder 体。
pub fn read_symbol_border_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SymbolBorderBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, MAX_BORDER_VERSION, "SymbolBorder", entry_offset)?;
    let double_1 = read_u64_le(&mut trial)?;
    let slot = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)?;
    let stop = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol);
    let mut short_1 = 0;
    let mut double_2 = None;
    if stop.is_none() {
        let (s, d) = read_decor_tail_10_1(&mut trial, version)?;
        short_1 = s;
        double_2 = d;
    }
    let result = SymbolBorderBody10_1 {
        key,
        version,
        double_1,
        symbol: slot,
        short_1,
        double_2,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// SymbolBackground 体（Load `0x7505c510`；符号槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolBackgroundBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝。
    pub version: u16,
    /// double → this−0x1c（尺寸候选）。
    pub double_1: u64,
    /// 背景填充符号槽（Save 槽 this−0xc）。
    pub symbol: SymbolSlotFull10_1,
    /// short → this−8。
    pub short_1: u16,
    /// 版本 ≥2：double → this−0x14。
    pub double_2: Option<u64>,
    /// 嵌套停止。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SymbolBackground 体。
pub fn read_symbol_background_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SymbolBackgroundBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_BACKGROUND_VERSION,
        "SymbolBackground",
        entry_offset,
    )?;
    let double_1 = read_u64_le(&mut trial)?;
    let slot = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)?;
    let stop = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::FillSymbol);
    let mut short_1 = 0;
    let mut double_2 = None;
    if stop.is_none() {
        let (s, d) = read_decor_tail_10_1(&mut trial, version)?;
        short_1 = s;
        double_2 = d;
    }
    let result = SymbolBackgroundBody10_1 {
        key,
        version,
        double_1,
        symbol: slot,
        short_1,
        double_2,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// SymbolShadow 体（Load `0x750601e0`；符号槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolShadowBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// 阴影填充符号槽（Save 槽 this−0xc）。
    pub symbol: SymbolSlotFull10_1,
    /// short → this−8。
    pub short_1: u16,
    /// double → this−0x1c。
    pub double_1: u64,
    /// double → this−0x14。
    pub double_2: u64,
    /// 嵌套停止。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SymbolShadow 体。
pub fn read_symbol_shadow_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SymbolShadowBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, MAX_SHADOW_VERSION, "SymbolShadow", entry_offset)?;
    let slot = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)?;
    let stop = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::FillSymbol);
    let mut short_1 = 0;
    let mut double_1 = 0;
    let mut double_2 = 0;
    if stop.is_none() {
        short_1 = trial.read_u16_le()?;
        double_1 = read_u64_le(&mut trial)?;
        double_2 = read_u64_le(&mut trial)?;
    }
    let result = SymbolShadowBody10_1 {
        key,
        version,
        symbol: slot,
        short_1,
        double_1,
        double_2,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 装饰槽位条目（IBorder/IBackground/IShadow 三槽共用形状）。
#[derive(Debug, Clone, PartialEq)]
pub enum DecorSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 SymbolBorder 对象（体闭合；`0x7505ec70`）。
    Border(Box<SymbolBorderBody10_1>),
    /// 新 SymbolBackground 对象（体闭合；`0x7505c510`）。
    Background(Box<SymbolBackgroundBody10_1>),
    /// 新 SymbolShadow 对象（体闭合；`0x750601e0`）。
    Shadow(Box<SymbolShadowBody10_1>),
}

/// 读取一个装饰类槽位条目（`expect` 指定该槽的接口族：Border/Background/
/// Shadow 之一，仅接受对应实现类），返回 `(槽位, 停止)`。
///
/// 空标记与回引可消费；其余实现类回退到槽位起点、登记表不保留该键，以停止
/// 原因返回。
pub fn read_decor_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
    expect: DecorKind10_1,
) -> io::Result<(Option<DecorSlot10_1>, Option<SymbolStop10_1>)> {
    const MAX_DECOR_NESTING_DEPTH: usize = 16;
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(DecorSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((Some(DecorSlot10_1::BackReference { class_id, key }), None))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "元素装饰槽位：嵌套深度超过读取器限额 \
                         {MAX_DECOR_NESTING_DEPTH}（防栈溢出，非原生限制）"
                    ),
                ));
            }
            let slot = match expect {
                DecorKind10_1::Border if class_id == SYMBOL_BORDER_CLASS_ID => Some(
                    DecorSlot10_1::Border(Box::new(read_symbol_border_body_10_1(
                        &mut forward,
                        max_bytes,
                        staged,
                        key,
                        depth - 1,
                    )?)),
                ),
                DecorKind10_1::Background if class_id == SYMBOL_BACKGROUND_CLASS_ID => Some(
                    DecorSlot10_1::Background(Box::new(read_symbol_background_body_10_1(
                        &mut forward,
                        max_bytes,
                        staged,
                        key,
                        depth - 1,
                    )?)),
                ),
                DecorKind10_1::Shadow if class_id == SYMBOL_SHADOW_CLASS_ID => Some(
                    DecorSlot10_1::Shadow(Box::new(read_symbol_shadow_body_10_1(
                        &mut forward,
                        max_bytes,
                        staged,
                        key,
                        depth - 1,
                    )?)),
                ),
                _ => None,
            };
            match slot {
                Some(slot) => {
                    let stop = decor_body_stop(&slot);
                    *trial = forward;
                    Ok((Some(slot), stop))
                }
                None => {
                    *trial = origin;
                    *staged = saved;
                    Ok((
                        None,
                        Some(SymbolStop10_1::Slot {
                            kind: match expect {
                                DecorKind10_1::Border => DisplaySlotKind10_1::Symbol,
                                DecorKind10_1::Background => DisplaySlotKind10_1::FillSymbol,
                                DecorKind10_1::Shadow => DisplaySlotKind10_1::FillSymbol,
                            },
                            class_id,
                            key,
                        }),
                    ))
                }
            }
        }
    }
}

/// 装饰槽的接口族（该槽接受的实现类）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorKind10_1 {
    /// IBorder 槽（SymbolBorder）。
    Border,
    /// IBackground 槽（SymbolBackground）。
    Background,
    /// IShadow 槽（SymbolShadow）。
    Shadow,
}

/// 归约装饰体内容嵌套停止（与装饰槽分派同型）。
fn decor_body_stop(slot: &DecorSlot10_1) -> Option<SymbolStop10_1> {
    let inner = match slot {
        DecorSlot10_1::Border(body) => (SYMBOL_BORDER_CLASS_ID, body.key, body.stop.as_ref()),
        DecorSlot10_1::Background(body) => {
            (SYMBOL_BACKGROUND_CLASS_ID, body.key, body.stop.as_ref())
        }
        DecorSlot10_1::Shadow(body) => (SYMBOL_SHADOW_CLASS_ID, body.key, body.stop.as_ref()),
        _ => return None,
    };
    inner.2.map(|_| SymbolStop10_1::NestedIncomplete {
        class_id: inner.0,
        key: inner.1,
    })
}

/// IBorder 槽分派入口（TextElement 版本 ==7 装饰块第 1 槽）。
pub fn read_border_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(Option<DecorSlot10_1>, Option<SymbolStop10_1>)> {
    read_decor_slot_10_1(trial, max_bytes, staged, depth, DecorKind10_1::Border)
}

/// IBackground 槽分派入口（TextElement 版本 ==7 装饰块第 2 槽）。
pub fn read_background_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(Option<DecorSlot10_1>, Option<SymbolStop10_1>)> {
    read_decor_slot_10_1(trial, max_bytes, staged, depth, DecorKind10_1::Background)
}

/// IShadow 槽分派入口（TextElement 版本 ==7 装饰块第 3 槽）。
pub fn read_shadow_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(Option<DecorSlot10_1>, Option<SymbolStop10_1>)> {
    read_decor_slot_10_1(trial, max_bytes, staged, depth, DecorKind10_1::Shadow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
        let mut bytes = class_id.to_vec();
        bytes.extend(key.to_le_bytes());
        bytes
    }

    #[test]
    fn symbol_border_full_body() {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend(1.0_f64.to_le_bytes());
        bytes.extend([0u8; 16]); // 符号槽空标记
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend(3.0_f64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_symbol_border_body_10_1(&mut reader, 1024, &mut table, 10, 8)
            .expect("SymbolBorder 体应整体消费");
        assert_eq!(body.version, 2);
        assert_eq!(body.double_1, 1.0_f64.to_bits());
        assert_eq!(body.symbol, SymbolSlotFull10_1::Null);
        assert_eq!(body.short_1, 7);
        assert_eq!(body.double_2, Some(3.0_f64.to_bits()));
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn symbol_border_version_1_has_no_tail_double() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(1.0_f64.to_le_bytes());
        bytes.extend([0u8; 16]);
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend([0xAA]); // 版本 1 无尾 double：哨兵不应被消费
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_symbol_border_body_10_1(&mut reader, 1024, &mut table, 11, 8).unwrap();
        assert_eq!(body.double_2, None);
        assert_eq!(reader.remaining(), 1);
    }

    #[test]
    fn symbol_background_and_shadow_full_bodies() {
        let mut bg = 2_u16.to_le_bytes().to_vec();
        bg.extend(1.0_f64.to_le_bytes());
        bg.extend([0u8; 16]);
        bg.extend(7_u16.to_le_bytes());
        bg.extend(3.0_f64.to_le_bytes());
        let mut reader = Reader::new(&bg);
        let mut table = RefTable10_1::new();
        let body = read_symbol_background_body_10_1(&mut reader, 1024, &mut table, 12, 8)
            .expect("SymbolBackground 体应整体消费");
        assert_eq!(body.version, 2);
        assert_eq!(reader.remaining(), 0);

        let mut sh = 1_u16.to_le_bytes().to_vec();
        sh.extend([0u8; 16]); // 符号槽空标记
        sh.extend(7_u16.to_le_bytes());
        sh.extend(1.0_f64.to_le_bytes());
        sh.extend(2.0_f64.to_le_bytes());
        let mut reader = Reader::new(&sh);
        let mut table = RefTable10_1::new();
        let body = read_symbol_shadow_body_10_1(&mut reader, 1024, &mut table, 13, 8)
            .expect("SymbolShadow 体应整体消费");
        assert_eq!(body.version, 1);
        assert_eq!(body.double_1, 1.0_f64.to_bits());
        assert_eq!(body.double_2, 2.0_f64.to_bits());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn decor_slot_dispatch_matrix() {
        // Shadow 槽接受 SymbolShadow；Border/Background 实现类在 Shadow 槽停
        // 止回退。
        let mut bytes = entry(&SYMBOL_SHADOW_CLASS_ID, 44);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0u8; 16]);
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend(1.0_f64.to_le_bytes());
        bytes.extend(2.0_f64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) = read_shadow_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(matches!(slot, Some(DecorSlot10_1::Shadow(_))) && stop.is_none());
        assert_eq!(reader.remaining(), 0);
        // 回引。
        let mut table = RefTable10_1::new();
        table.register(45);
        let bytes = entry(&SYMBOL_BORDER_CLASS_ID, 45);
        let mut reader = Reader::new(&bytes);
        let (slot, stop) = read_border_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(
            matches!(slot, Some(DecorSlot10_1::BackReference { key: 45, .. })) && stop.is_none()
        );
        // 错位实现类：Border 类出现在 Background 槽 → 停止回退。
        let bytes = entry(&SYMBOL_BORDER_CLASS_ID, 46);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) = read_background_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(slot.is_none());
        assert!(stop.is_some());
        assert_eq!(reader.remaining(), bytes.len());
        assert!(table.is_empty());
        // 未知实现类停止。
        let unknown = [0x55_u8; 16];
        let bytes = entry(&unknown, 47);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) = read_border_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(slot.is_none());
        match stop.expect("未知类应停止") {
            SymbolStop10_1::Slot { class_id, key, .. } => {
                assert_eq!(class_id, unknown);
                assert_eq!(key, 47);
            }
            other => panic!("停止原因形态错误 {other:?}"),
        }
        assert!(table.is_empty());
        // 版本 3 拒绝（Border/Background 上限 2）。
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend([0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_symbol_border_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
        // 逐前缀截断。
        let mut full = 2_u16.to_le_bytes().to_vec();
        full.extend(1.0_f64.to_le_bytes());
        full.extend([0u8; 16]);
        full.extend(7_u16.to_le_bytes());
        full.extend(3.0_f64.to_le_bytes());
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let mut table = RefTable10_1::new();
            assert!(
                read_symbol_border_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err(),
                "截断到 {cut} 字节应失败"
            );
            assert!(table.is_empty());
        }
    }

    #[test]
    fn class_id_constants_match_guid_strings() {
        // {A5D0F017-62DD-11D2-87BE-0000F8751720} 的 LE 内存序。
        let expect_border: [u8; 16] = [
            0x17, 0xf0, 0xd0, 0xa5, 0xdd, 0x62, 0xd2, 0x11, 0x87, 0xbe, 0x00, 0x00, 0xf8, 0x75,
            0x17, 0x20,
        ];
        assert_eq!(SYMBOL_BORDER_CLASS_ID, expect_border);
        // {1BAA33E9-E13B-11D2-B868-00600802E603} 的 LE 内存序。
        let expect_background: [u8; 16] = [
            0xe9, 0x33, 0xaa, 0x1b, 0x3b, 0xe1, 0xd2, 0x11, 0xb8, 0x68, 0x00, 0x60, 0x08, 0x02,
            0xe6, 0x03,
        ];
        assert_eq!(SYMBOL_BACKGROUND_CLASS_ID, expect_background);
        // {A8861E66-57AA-47D0-AAF8-B288B4FD5240} 的 LE 内存序。
        let expect_shadow: [u8; 16] = [
            0x66, 0x1e, 0x86, 0xa8, 0xaa, 0x57, 0xd0, 0x47, 0xaa, 0xf8, 0xb2, 0x88, 0xb4, 0xfd,
            0x52, 0x40,
        ];
        assert_eq!(SYMBOL_SHADOW_CLASS_ID, expect_shadow);
    }
}
