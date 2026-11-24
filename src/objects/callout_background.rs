//! ArcMap 10.1 DisplayCore.dll Callout/文字背景族与 IMarkerBackground 实现类
//! 对象体严格读取子集与槽位分派。
//!
//! 证据来源：`docs/arcmap-10.1-display-slots-round3.md`（G 轮研究记录）。四类
//! 宿主经「esri.clsid 声明模块 → RTTI TypeDescriptor → COL → IPersistStream
//! 虚表 → GetClassID 读 `.rdata` CLSID 常量」链在 DisplayCore.dll 10.1.0.3035
//! （SHA-256 `9b3e61f9…6e4d2`，ImageBase `0x6eb30000`）中逐类闭环：
//!
//! - **BalloonCallout** `{C8D09ED2-4FBB-11D1-9A72-0080C7EC5C96}`：虚表
//!   `0x6ec0cdf4`（COL `0x6ec1ab04`），GetClassID `0x6eb41b70`（读常量
//!   `0x6ec0cf1c`）、Load `0x6eb41950`、Save `0x6eb41a70`。
//! - **LineCallout** `{C8D09ED3-…}`：虚表 `0x6ec0e3ec`，GetClassID
//!   `0x6eb59300`（读常量 `0x6ec0d6e4`）、Load `0x6eb58fd0`、Save `0x6eb59170`。
//! - **SimpleLineCallout** `{FA37B822-A959-4ACD-834A-0E114BF420B8}`：虚表
//!   `0x6ec10bc8`，GetClassID `0x6eb82730`（读常量 `0x6ec10c00`）、Load
//!   `0x6eb82580`、Save `0x6eb82670`。
//! - **MarkerTextBackground** `{C5C02D50-7282-11D2-9816-0080C7E04196}`（
//!   IMarkerBackground `{AE41E4F0-…}` 的流内实现类）：虚表 `0x6ec0ee34`，
//!   GetClassID `0x6eb644b0`（读常量 `0x6ec0ee74`）、Load `0x6eb64380`、Save
//!   `0x6eb64440`。
//!
//! ## 字段序（Load 反编译 + Save 对称互证；证据强度：确定的；样本 0 实例）
//!
//! **BalloonCallout::Load `0x6eb41950`**（版本 `1 < (short)ver` 拒绝；Save 恒
//! 写版本 1，常量 `DAT_6ec0b310`=1）：
//!
//! 1. int16 版本；2. int32 → this−0x7c（样式候选 esriBalloonCalloutStyle）；
//! 3. double ×4 → this−0x4c / −0x3c / −0x44 / −0x34（边距候选，Load 顺序）；
//! 4. LoadObject（ISymbol `{F3435802-…}`）→ 填充符号槽（Save 槽 this−0x88，
//!    经 [`crate::objects::display_symbol::read_symbol_slot_full_10_1`] 全闭
//!    合符号类分派）；
//! 5. LoadObject（IPoint `{1B077915-38C5-11D0-92D2-00805F7C28B0}`，常量
//!    `DAT_6ec0ce6c`）→ 锚点槽（Save 槽 this−0x68；Point 类整体解析，其余几
//!    何类停止）；
//! 6. double → this−0x2c（圆角候选）。
//!
//! **LineCallout::Load `0x6eb58fd0`**（版本 ≤1；Save 恒写版本 1，常量
//! `DAT_6ec0b2f0`=1）：
//!
//! 1. int16 版本；2. LoadObject（ISymbol）×3 → Save 槽 this−0xbc（边界符号候
//!    选）/ −0xb8（强调条候选）/ −0xb4（引线符号候选）；
//! 3. LoadObject（IPoint）→ Save 槽 this−0x84（锚点槽）；
//! 4. double → this−0xac；
//! 5. int32 → this−0xa4（样式候选 esriLineCalloutStyle；Save 在 docVer==0 且
//!    值为 6/7 时改写 1 的历史映射，流内值原样保留）；
//! 6. double ×5 → this−0x5c / −0x54 / −0x4c / −0x44 / −0x3c（边距候选）。
//!
//! **SimpleLineCallout::Load `0x6eb82580`**（版本 ≤1；Save 恒写版本 1，常量
//! `DAT_6ec0b294`=1）：
//!
//! 1. int16 版本；2. double → this−0x2c；3. short → this−0x24；
//! 4. LoadObject（IGeometry `{1B07790D-38C5-11D0-92D2-00805F7C28B0}`，常量
//!    `DAT_6ec0ce5c`）→ 锚几何槽（Save 槽 this−0x20；闭合几何类整体解析）；
//! 5. LoadObject（ILineSymbol `{E6BDAA7D-…}`，常量 `DAT_6ec0d334`）→ 引线符
//!    号槽（Save 槽 this−0x1c，符号全分派）；
//! 6. LoadObject（IPoint）→ Save 槽 this−0x18（锚点槽）。
//!
//! **MarkerTextBackground::Load `0x6eb64380`**（版本 ≤1；Save 恒写版本 1，常
//! 量 `DAT_6ec0b2dc`=1）：
//!
//! 1. int16 版本；2. short → this−0x1c；3. LoadObject（ISymbol）→ 标记符号槽
//!    （Save 槽 this−0x2c，符号全分派）。
//!
//! 事务性与限额约定同仓内其他读取器：调用者已读对象头并登记新键，从对象头
//! 之后的位置传入 Reader；全部字段在 trial 游标与 trial 登记表上读取，成功
//! （含停止点）才提交；版本 short 按原生有符号比较设上限，保留原始位模式；
//! 截断与非法长度明确报错；`max_bytes` 约束嵌套体字符串
//! （读取器自身限额，非原生限制）。仓库参考样本 `samples/测试底图.mxd` 中
//! 四类 0 实例，全部结论无样本字节佐证，单元测试仅合成输入，不代表原生已
//! 验证。建议接入点见研究记录（接线归整合轮）。

use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, SymbolStop10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::geometry::{POINT_CLASS_ID, PointBody10_1, read_point_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// BalloonCallout 类标识。证据：esri.clsid + GetClassID `0x6eb41b70` 读常量
/// `0x6ec0cf1c` 互证。
pub const BALLOON_CALLOUT_CLASS_ID: [u8; 16] = [
    0xd2, 0x9e, 0xd0, 0xc8, 0xbb, 0x4f, 0xd1, 0x11, 0x9a, 0x72, 0x00, 0x80, 0xc7, 0xec, 0x5c, 0x96,
];

/// LineCallout 类标识。证据：esri.clsid + GetClassID `0x6eb59300` 读常量
/// `0x6ec0d6e4` 互证。
pub const LINE_CALLOUT_CLASS_ID: [u8; 16] = [
    0xd3, 0x9e, 0xd0, 0xc8, 0xbb, 0x4f, 0xd1, 0x11, 0x9a, 0x72, 0x00, 0x80, 0xc7, 0xec, 0x5c, 0x96,
];

/// SimpleLineCallout 类标识。证据：esri.clsid + GetClassID `0x6eb82730` 读常
/// 量 `0x6ec10c00` 互证。
pub const SIMPLE_LINE_CALLOUT_CLASS_ID: [u8; 16] = [
    0x22, 0xb8, 0x37, 0xfa, 0x59, 0xa9, 0xcd, 0x4a, 0x8a, 0x34, 0x0e, 0x11, 0x4b, 0xf4, 0x20, 0xb8,
];

/// MarkerTextBackground 类标识。证据：esri.clsid + GetClassID `0x6eb644b0`
/// 读常量 `0x6ec0ee74` 互证。
pub const MARKER_TEXT_BACKGROUND_CLASS_ID: [u8; 16] = [
    0x50, 0x2d, 0xc0, 0xc5, 0x82, 0x72, 0xd2, 0x11, 0x98, 0x16, 0x00, 0x80, 0xc7, 0xe0, 0x41, 0x96,
];

const MAX_CALLOUT_VERSION: i16 = 1;
const MAX_MARKER_BACKGROUND_VERSION: i16 = 1;

/// 读取流内 int16 版本并按原生有符号上限校验。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    label: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let raw = trial.read_u16_le()?;
    let signed = raw as i16;
    if signed > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{label} 对象体 offset 0x{entry_offset:x}: 版本位模式 {raw}（0x{raw:04x}）\
                 超过原生有符号上限 {max}"
            ),
        ));
    }
    Ok(raw)
}

fn read_i32_le(trial: &mut Reader<'_>) -> io::Result<i32> {
    Ok(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
}

/// 读取 8 字节 LE u64（Reader 无 u64 原语，本模块用原样位模式保存 double）。
fn read_u64_le(trial: &mut Reader<'_>) -> io::Result<u64> {
    Ok(u64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap()))
}

/// IPoint 类槽位条目（Callout 锚点槽共用；Point 类整体解析）。
#[derive(Debug, Clone, PartialEq)]
pub enum PointSlotEntry10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 Point 对象（体闭合；`crate::objects::geometry`）。
    Point(Box<PointBody10_1>),
}

/// 读取一个 IPoint 类槽位条目，返回 `(槽位, 停止)`。
///
/// 空标记与回引可消费；新对象类标识为 Point 时解析其体，其余实现类回退到槽
/// 位起点、登记表不保留该键，以停止原因返回（`kind` 固定为 ISymbol 之外自
/// 定义的槽位说明文本见宿主）。
fn read_point_slot_entry_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<PointSlotEntry10_1>, Option<SymbolStop10_1>)> {
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(PointSlotEntry10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(PointSlotEntry10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if class_id == POINT_CLASS_ID {
                let body = read_point_body_10_1(&mut forward, max_bytes, staged)?;
                *trial = forward;
                Ok((Some(PointSlotEntry10_1::Point(Box::new(body))), None))
            } else {
                *trial = origin;
                *staged = saved;
                Ok((
                    None,
                    Some(SymbolStop10_1::Slot {
                        kind,
                        class_id,
                        key,
                    }),
                ))
            }
        }
    }
}

/// 归约符号槽的停止（与装饰槽分派同型）：`Unsupported` → 本槽位停止；嵌套
/// 体停止 → 嵌套未完整。
fn symbol_slot_stop(
    slot: &SymbolSlotFull10_1,
    kind: DisplaySlotKind10_1,
) -> Option<SymbolStop10_1> {
    slot_full_stop_10_1(slot, kind)
}

/// IGeometry 类槽位条目（SimpleLineCallout 锚几何槽；闭合几何类整体解析）。
#[derive(Debug, Clone, PartialEq)]
pub enum GeometrySlotEntry10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新闭合几何类对象（Polyline/Polygon/Point/Multipoint）。
    Body(Box<crate::objects::geometry::GeometryBody10_1>),
}

/// BalloonCallout 体（Load `0x6eb41950`；槽位停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct BalloonCalloutBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// int32 → this−0x7c（样式候选）。
    pub style: i32,
    /// double ×4 → this−0x4c / −0x3c / −0x44 / −0x34（边距候选）。
    pub margins: [u64; 4],
    /// 填充符号槽（ISymbol；Save 槽 this−0x88）。
    pub symbol: SymbolSlotFull10_1,
    /// 锚点槽（IPoint；Save 槽 this−0x68）。
    pub anchor: Option<PointSlotEntry10_1>,
    /// double → this−0x2c（圆角候选）。
    pub rounded_rate: u64,
    /// 嵌套停止（槽位未知实现类/嵌套体停止时非 None；游标停在内层停止点）。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 BalloonCallout 体。
pub fn read_balloon_callout_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<BalloonCalloutBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_CALLOUT_VERSION,
        "BalloonCallout",
        entry_offset,
    )?;
    let style = read_i32_le(&mut trial)?;
    let mut margins = [0u64; 4];
    for slot in margins.iter_mut() {
        *slot = read_u64_le(&mut trial)?;
    }
    let (symbol, mut stop) =
        match read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)? {
            slot => {
                let stop = symbol_slot_stop(&slot, DisplaySlotKind10_1::Symbol);
                (slot, stop)
            }
        };
    let mut anchor = None;
    if stop.is_none() {
        match read_point_slot_entry_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            DisplaySlotKind10_1::Symbol,
        )? {
            (Some(entry), None) => anchor = Some(entry),
            (None, Some(point_stop)) => stop = Some(point_stop),
            _ => unreachable!(),
        }
    }
    let mut rounded_rate = 0u64;
    if stop.is_none() {
        rounded_rate = read_u64_le(&mut trial)?;
    }
    let result = BalloonCalloutBody10_1 {
        key,
        version,
        style,
        margins,
        symbol,
        anchor,
        rounded_rate,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// LineCallout 体（Load `0x6eb58fd0`；槽位停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct LineCalloutBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// 符号槽 ×3（Save 槽 this−0xbc 边界 / −0xb8 强调条 / −0xb4 引线）。
    pub symbols: [SymbolSlotFull10_1; 3],
    /// 锚点槽（IPoint；Save 槽 this−0x84）。
    pub anchor: Option<PointSlotEntry10_1>,
    /// double → this−0xac。
    pub double_1: u64,
    /// int32 → this−0xa4（样式候选）。
    pub style: i32,
    /// double ×5 → this−0x5c / −0x54 / −0x4c / −0x44 / −0x3c（边距候选）。
    pub margins: [u64; 5],
    /// 嵌套停止。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 LineCallout 体。
pub fn read_line_callout_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<LineCalloutBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, MAX_CALLOUT_VERSION, "LineCallout", entry_offset)?;
    let mut symbols = Vec::with_capacity(3);
    let mut stop = None;
    for _ in 0..3 {
        let slot = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)?;
        if let Some(slot_stop) = symbol_slot_stop(&slot, DisplaySlotKind10_1::Symbol) {
            stop = Some(slot_stop);
            break;
        }
        symbols.push(slot);
    }
    let mut anchor = None;
    let mut double_1 = 0u64;
    let mut style = 0;
    let mut margins = [0u64; 5];
    if stop.is_none() && symbols.len() == 3 {
        match read_point_slot_entry_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            DisplaySlotKind10_1::Symbol,
        )? {
            (Some(entry), None) => anchor = Some(entry),
            (None, Some(point_stop)) => stop = Some(point_stop),
            _ => unreachable!(),
        }
    }
    if stop.is_none() {
        double_1 = read_u64_le(&mut trial)?;
        style = read_i32_le(&mut trial)?;
        for slot in margins.iter_mut() {
            *slot = read_u64_le(&mut trial)?;
        }
    }
    // 未达满 3 符号槽即停止时补空位以保持定长形状。
    while symbols.len() < 3 {
        symbols.push(SymbolSlotFull10_1::Null);
    }
    let symbols = [symbols[0].clone(), symbols[1].clone(), symbols[2].clone()];
    let result = LineCalloutBody10_1 {
        key,
        version,
        symbols,
        anchor,
        double_1,
        style,
        margins,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// SimpleLineCallout 体（Load `0x6eb82580`；槽位停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleLineCalloutBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// double → this−0x2c。
    pub double_1: u64,
    /// short → this−0x24。
    pub short_1: u16,
    /// 锚几何槽（IGeometry；Save 槽 this−0x20；闭合几何类整体解析）。
    pub anchor_geometry: Option<GeometrySlotEntry10_1>,
    /// 引线符号槽（ILineSymbol/ISymbol；Save 槽 this−0x1c）。
    pub line_symbol: Option<SymbolSlotFull10_1>,
    /// 锚点槽（IPoint；Save 槽 this−0x18）。
    pub anchor: Option<PointSlotEntry10_1>,
    /// 嵌套停止。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleLineCallout 体。
pub fn read_simple_line_callout_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SimpleLineCalloutBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_CALLOUT_VERSION,
        "SimpleLineCallout",
        entry_offset,
    )?;
    let double_1 = read_u64_le(&mut trial)?;
    let short_1 = trial.read_u16_le()?;
    // 锚几何槽：闭合几何类整体解析，其余实现类停止回退。
    let mut anchor_geometry = None;
    let mut stop = None;
    {
        let origin = trial.clone();
        let saved = trial_table.clone();
        let mut forward = trial.clone();
        let entry = read_object_entry_10_1(&mut forward, &mut trial_table)?;
        match entry {
            ObjectEntry10_1::Null => {
                anchor_geometry = Some(GeometrySlotEntry10_1::Null);
                trial = forward;
            }
            ObjectEntry10_1::BackReference { class_id, key } => {
                anchor_geometry = Some(GeometrySlotEntry10_1::BackReference { class_id, key });
                trial = forward;
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                match crate::objects::geometry::read_keyed_geometry_body_10_1(
                    &class_id,
                    key,
                    &mut forward,
                    max_bytes,
                    &mut trial_table,
                ) {
                    Ok(body) => {
                        anchor_geometry = Some(GeometrySlotEntry10_1::Body(Box::new(body)));
                        trial = forward;
                    }
                    Err(error) if error.kind() == ErrorKind::Unsupported => {
                        // 非闭合几何类：回退到槽位起点并停止（体自身失败原样
                        // 上抛，整体失败）。
                        trial = origin;
                        trial_table = saved;
                        stop = Some(SymbolStop10_1::Slot {
                            kind: DisplaySlotKind10_1::Symbol,
                            class_id,
                            key,
                        });
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }
    let mut line_symbol = None;
    if stop.is_none() {
        let slot = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)?;
        if let Some(slot_stop) = symbol_slot_stop(&slot, DisplaySlotKind10_1::Symbol) {
            stop = Some(slot_stop);
        } else {
            line_symbol = Some(slot);
        }
    }
    let mut anchor = None;
    if stop.is_none() {
        match read_point_slot_entry_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            DisplaySlotKind10_1::Symbol,
        )? {
            (Some(entry), None) => anchor = Some(entry),
            (None, Some(point_stop)) => stop = Some(point_stop),
            _ => unreachable!(),
        }
    }
    let result = SimpleLineCalloutBody10_1 {
        key,
        version,
        double_1,
        short_1,
        anchor_geometry,
        line_symbol,
        anchor,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// MarkerTextBackground 体（Load `0x6eb64380`；符号槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerTextBackgroundBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// short → this−0x1c。
    pub short_1: u16,
    /// 标记符号槽（ISymbol；Save 槽 this−0x2c）。
    pub symbol: SymbolSlotFull10_1,
    /// 嵌套停止。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MarkerTextBackground 体。
pub fn read_marker_text_background_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MarkerTextBackgroundBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_MARKER_BACKGROUND_VERSION,
        "MarkerTextBackground",
        entry_offset,
    )?;
    let short_1 = trial.read_u16_le()?;
    let slot = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut trial_table, depth)?;
    let stop = symbol_slot_stop(&slot, DisplaySlotKind10_1::MarkerSymbol);
    let result = MarkerTextBackgroundBody10_1 {
        key,
        version,
        short_1,
        symbol: slot,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// Callout/文字背景类槽位条目（三 Callout 与 MarkerTextBackground 共用；宿
/// 主按接口族语义选槽：ITextBackground/ICallout/IMarkerBackground 候选）。
#[derive(Debug, Clone, PartialEq)]
pub enum CalloutSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 BalloonCallout 对象（体闭合；`0x6eb41950`）。
    Balloon(Box<BalloonCalloutBody10_1>),
    /// 新 LineCallout 对象（体闭合；`0x6eb58fd0`）。
    Line(Box<LineCalloutBody10_1>),
    /// 新 SimpleLineCallout 对象（体闭合；`0x6eb82580`）。
    SimpleLine(Box<SimpleLineCalloutBody10_1>),
    /// 新 MarkerTextBackground 对象（`0x6eb64380`，实现 IMarkerTextBackground）。
    MarkerBackground(Box<MarkerTextBackgroundBody10_1>),
}

/// 读取一个 Callout/文字背景类槽位条目，返回 `(槽位, 停止)`。
///
/// `allow_marker_background` 控制 ITextBackground 槽是否接受 MarkerTextBackground。
/// 纯 callout 接口族槽传 `false`。其余实现类回退到槽位起点、
/// 登记表不保留该键，以停止原因返回。
pub fn read_callout_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
    allow_marker_background: bool,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<CalloutSlot10_1>, Option<SymbolStop10_1>)> {
    const MAX_CALLOUT_NESTING_DEPTH: usize = 16;
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(CalloutSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((Some(CalloutSlot10_1::BackReference { class_id, key }), None))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Callout 槽位：嵌套深度超过读取器限额 \
                         {MAX_CALLOUT_NESTING_DEPTH}（防栈溢出，非原生限制）"
                    ),
                ));
            }
            let slot = if class_id == BALLOON_CALLOUT_CLASS_ID {
                Some(CalloutSlot10_1::Balloon(Box::new(
                    read_balloon_callout_body_10_1(
                        &mut forward,
                        max_bytes,
                        staged,
                        key,
                        depth - 1,
                    )?,
                )))
            } else if class_id == LINE_CALLOUT_CLASS_ID {
                Some(CalloutSlot10_1::Line(Box::new(
                    read_line_callout_body_10_1(&mut forward, max_bytes, staged, key, depth - 1)?,
                )))
            } else if class_id == SIMPLE_LINE_CALLOUT_CLASS_ID {
                Some(CalloutSlot10_1::SimpleLine(Box::new(
                    read_simple_line_callout_body_10_1(
                        &mut forward,
                        max_bytes,
                        staged,
                        key,
                        depth - 1,
                    )?,
                )))
            } else if allow_marker_background && class_id == MARKER_TEXT_BACKGROUND_CLASS_ID {
                Some(CalloutSlot10_1::MarkerBackground(Box::new(
                    read_marker_text_background_body_10_1(
                        &mut forward,
                        max_bytes,
                        staged,
                        key,
                        depth - 1,
                    )?,
                )))
            } else {
                None
            };
            match slot {
                Some(slot) => {
                    // 体内容嵌套停止以嵌套未完整原因传播；游标停在内层停止点
                    // （forward 已推进）。
                    let stop = callout_body_stop(&slot);
                    *trial = forward;
                    Ok((Some(slot), stop))
                }
                None => {
                    *trial = origin;
                    *staged = saved;
                    Ok((
                        None,
                        Some(SymbolStop10_1::Slot {
                            kind,
                            class_id,
                            key,
                        }),
                    ))
                }
            }
        }
    }
}

/// 归约 Callout 体内容嵌套停止（与装饰槽分派同型）。
fn callout_body_stop(slot: &CalloutSlot10_1) -> Option<SymbolStop10_1> {
    let inner = match slot {
        CalloutSlot10_1::Balloon(body) => (BALLOON_CALLOUT_CLASS_ID, body.key, body.stop),
        CalloutSlot10_1::Line(body) => (LINE_CALLOUT_CLASS_ID, body.key, body.stop),
        CalloutSlot10_1::SimpleLine(body) => (SIMPLE_LINE_CALLOUT_CLASS_ID, body.key, body.stop),
        CalloutSlot10_1::MarkerBackground(body) => {
            (MARKER_TEXT_BACKGROUND_CLASS_ID, body.key, body.stop)
        }
        // 空标记与回引无体，无嵌套停止。
        CalloutSlot10_1::Null | CalloutSlot10_1::BackReference { .. } => return None,
    };
    inner.2.map(|_| SymbolStop10_1::NestedIncomplete {
        class_id: inner.0,
        key: inner.1,
    })
}

/// 读取 IMarkerBackground 槽。DisplayCore 的 LineCallout::QueryInterface
/// （`0x6eb577f0`）直接返回该接口；未知新对象在头起点停止。
pub fn read_marker_background_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(Option<CalloutSlot10_1>, Option<SymbolStop10_1>)> {
    let kind = DisplaySlotKind10_1::MarkerBackground;
    if let ObjectHeader::Object { class_id, key } = read_header_10_1(&mut trial.clone())?
        && !staged.contains(key)
        && class_id != LINE_CALLOUT_CLASS_ID
    {
        return Ok((
            None,
            Some(SymbolStop10_1::Slot {
                kind,
                class_id,
                key,
            }),
        ));
    }
    read_callout_slot_10_1(trial, max_bytes, staged, depth, false, kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造对象条目字节：16 字节类标识 + u32 键。
    fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
        let mut bytes = class_id.to_vec();
        bytes.extend(key.to_le_bytes());
        bytes
    }

    /// Point 体（Point::Load：版本 + 声明长度 + 类型字 + X + Y + 嵌套空间
    /// 参考空标记；与 tests/geometry.rs 的合成形态一致）。
    fn point_body_bytes() -> Vec<u8> {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend(20_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.extend(2.5_f64.to_le_bytes());
        bytes.extend([0u8; 16]);
        bytes
    }

    #[test]
    fn balloon_callout_with_point_anchor_full_body() {
        let mut bytes = 0xffff_u16.to_le_bytes().to_vec();
        bytes.extend(2_i32.to_le_bytes());
        for v in [1.0f64, 2.0, 3.0, 4.0] {
            bytes.extend(v.to_le_bytes());
        }
        // 填充符号槽：空标记。
        bytes.extend([0u8; 16]);
        // 锚点槽：Point 新对象。
        bytes.extend(entry(&POINT_CLASS_ID, 21));
        bytes.extend(point_body_bytes());
        bytes.extend(0.5_f64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_balloon_callout_body_10_1(&mut reader, 1024, &mut table, 20, 8)
            .expect("BalloonCallout 体应整体消费");
        assert_eq!(body.version, 0xffff);
        assert_eq!(body.style, 2);
        assert_eq!(body.symbol, SymbolSlotFull10_1::Null);
        assert!(matches!(body.anchor, Some(PointSlotEntry10_1::Point(_))));
        assert_eq!(body.rounded_rate, 0.5_f64.to_bits());
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn balloon_callout_unknown_symbol_stops() {
        let unknown = [0x33_u8; 16];
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0u8; 32]);
        bytes.extend(entry(&unknown, 22));
        bytes.extend([0xCC, 0xDD, 0xEE, 0xFF]); // 哨兵
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_balloon_callout_body_10_1(&mut reader, 1024, &mut table, 20, 8)
            .expect("嵌套停止属于成功返回");
        assert!(body.anchor.is_none());
        assert!(body.stop.is_some());
        // 游标停在未知符号槽对象头起点（20 字节条目 + 4 字节哨兵未消费）。
        assert_eq!(reader.remaining(), 24);
        assert!(table.is_empty());
    }

    #[test]
    fn line_callout_three_symbol_slots_then_scalars() {
        let mut bytes = 0xffff_u16.to_le_bytes().to_vec();
        for _ in 0..3 {
            bytes.extend([0u8; 16]); // 三个符号槽全空标记
        }
        bytes.extend([0u8; 16]); // 锚点槽空标记
        bytes.extend(9.0_f64.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        for v in [1.0f64, 2.0, 3.0, 4.0, 5.0] {
            bytes.extend(v.to_le_bytes());
        }
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_line_callout_body_10_1(&mut reader, 1024, &mut table, 30, 8)
            .expect("LineCallout 体应整体消费");
        assert_eq!(body.version, 0xffff);
        assert!(body.symbols.iter().all(|s| *s == SymbolSlotFull10_1::Null));
        assert!(body.anchor.is_some());
        assert_eq!(body.style, 1);
        assert_eq!(body.margins.len(), 5);
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn simple_line_callout_geometry_slot_dispatch() {
        // 锚几何槽：Polyline 类不在 callout 几何闭集合时按 Unsupported 停止。
        // 这里用 Point（闭合）验证通过路径。
        let mut bytes = 0xffff_u16.to_le_bytes().to_vec();
        bytes.extend(1.0_f64.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(entry(&POINT_CLASS_ID, 41));
        bytes.extend(point_body_bytes());
        bytes.extend([0u8; 16]); // 引线符号槽空标记
        bytes.extend([0u8; 16]); // 锚点槽空标记
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_simple_line_callout_body_10_1(&mut reader, 1024, &mut table, 40, 8)
            .expect("SimpleLineCallout 体应整体消费");
        assert_eq!(body.version, 0xffff);
        assert!(matches!(
            body.anchor_geometry,
            Some(GeometrySlotEntry10_1::Body(_))
        ));
        assert!(body.line_symbol.is_some());
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn marker_text_background_body() {
        let mut bytes = 0xffff_u16.to_le_bytes().to_vec();
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(entry(&POINT_CLASS_ID, 51)); // 符号槽用 Point 类不会命中符号闭合集 → 停止
        bytes.extend(point_body_bytes());
        let total = bytes.len();
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_marker_text_background_body_10_1(&mut reader, 1024, &mut table, 50, 8)
            .expect("MarkerTextBackground 体读取应成功（含停止）");
        assert_eq!(body.version, 0xffff);
        assert_eq!(body.short_1, 3);
        // Point 类标识不在符号闭合集：符号槽停止回退。
        assert!(body.stop.is_some());
        assert_eq!(reader.remaining(), total - 4);
        assert!(table.is_empty());
    }

    #[test]
    fn marker_background_slot_accepts_line_callout() {
        let mut bytes = entry(&LINE_CALLOUT_CLASS_ID, 55);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0u8; 4 * 16]); // 三个符号槽与锚点槽
        bytes.extend([0u8; 6 * 8 + 4]); // 标量段
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) = read_marker_background_slot_10_1(&mut reader, 1024, &mut table, 8)
            .expect("槽读取应成功");
        assert!(matches!(slot, Some(CalloutSlot10_1::Line(_))));
        assert!(stop.is_none());
        assert_eq!(reader.remaining(), 0);
        // IMarkerTextBackground 是另一接口，必须在对象头起点停止。
        let unknown = MARKER_TEXT_BACKGROUND_CLASS_ID;
        let bytes = entry(&unknown, 60);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) =
            read_marker_background_slot_10_1(&mut reader, 1024, &mut table, 8).unwrap();
        assert!(slot.is_none());
        match stop.expect("未知实现类应停止") {
            SymbolStop10_1::Slot {
                kind,
                class_id,
                key,
            } => {
                assert_eq!(kind, DisplaySlotKind10_1::MarkerBackground);
                assert_eq!(class_id, unknown);
                assert_eq!(key, 60);
            }
            other => panic!("停止原因形态错误 {other:?}"),
        }
        assert_eq!(reader.remaining(), bytes.len());
        assert!(table.is_empty());
    }

    #[test]
    fn callout_version_gates_and_truncation() {
        // 版本 2 拒绝（各 Callout 上限 1）。
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend([0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_balloon_callout_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
        let mut reader = Reader::new(&bytes);
        assert!(read_line_callout_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
        let mut reader = Reader::new(&bytes);
        assert!(read_simple_line_callout_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
        // BalloonCallout 全体逐前缀截断。
        let mut full = 1_u16.to_le_bytes().to_vec();
        full.extend(0_i32.to_le_bytes());
        full.extend([0u8; 32]);
        full.extend([0u8; 16]); // 符号槽
        full.extend([0u8; 16]); // 锚点槽
        full.extend(0.0_f64.to_le_bytes());
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let mut table = RefTable10_1::new();
            assert!(
                read_balloon_callout_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err(),
                "截断到 {cut} 字节应失败"
            );
            assert!(table.is_empty());
        }
    }
}
