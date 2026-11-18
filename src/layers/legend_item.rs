//! ArcMap 10.1 HorizontalLegendItem（水平图例条目）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-legend-tails.md`（2026-11-15 本轮研究记录）。
//! 二进制证据为 Renderers.dll 10.1.0.3035（本轮从安装介质提取复核，SHA-256
//! `399c80c9…56274`，temp/agent-legend-tails/）与 LayerLib.dll 10.1.0.3035
//! （temp/agent-fl-tail/ 既有基线）的静态反编译。
//!
//! ## 类身份与持久化链路（证据：确定的）
//!
//! - Renderers.dll GetClassID `0x7cad4c30` 写出 CLSID
//!   `{A9401A47-4649-11D1-880B-080009EC732A}`（与 esri.clsid 声明的模块
//!   Renderers.dll、类名 HorizontalLegendItem 三方互证；RTTI TD `0x7caf7970`
//!   名 `.?AVHorizontalLegendItem@@`）。仓库参考样本 /Maps 流 0 命中。
//! - 本类的 IPersistStream 虚表不在 QI 可见的接口表中，而是构造时经静态
//!   配置结构（`DAT_7caec4d4`）动态安装：对象 `this+0x174` 处的接口指针
//!   指向虚表 `0x7caeb244`（COL `0x7caee71c`，TD 同上），+0x0c=
//!   GetClassID 派发器 `0x7cae2350`（读对象内保存的 CLSID，即上述值）、
//!   +0x14=Load thunk `0x7cad5d30`、+0x18=Save thunk `0x7cad5d70`。
//! - Load thunk `0x7cad5d30` → 本体 `0x7cad50e0`（Renderers.dll）：读
//!   int16 条目版本（有符号 `1 < v` 拒绝 `0x80004005`）→ 读 int32（→
//!   对象 -8，写侧从内存回写）→ **尾调** LayerLib.dll 导出
//!   `?Load@BaseLegendItem@@UAGJPAUIStream@@@Z`（`0x786a2b00`，经 IAT
//!   `0x7cae9398` 交叉引用确认）。Save thunk 对称尾调
//!   `?Save@BaseLegendItem@@UAGJPAUIStream@@H@Z`（`0x786a2df0`）。
//! - Save 条目版本恒写静态常量 1（`DAT_7cae9760`）。
//!
//! ## BaseLegendItem::Load 字段序（LayerLib.dll `0x786a2b00`）
//!
//! 跟在条目版本与 int32 之后（体首 int16 为 BaseLegendItem 自己的版本
//! short，无拒绝门，仅做分支；Save 侧经 `AfStreamGetSaveVersion(docVer, 6,
//! DAT_7871ee18)`，版本表 `[1,1,1,1,1,3]` → 10.1（docVer=5）恒写 3）：
//!
//! 1. int16 基类版本（下称 v2；本读取器仅接受 0..=3，>3 与负值无写侧
//!    来源，一并拒绝较原生更严）；
//! 2. LoadObject（请求 IID `{34C20002-4D3C-11D0-92D8-00805F7C28B0}` =
//!    ILayer，类型库 esriCarto）→ this−0x130：条目关联图层（登记先于
//!    持久化，真实流中预期为回引；空标记亦合法）；
//! 3. int16 ×4 → this−0x12c/−0x12a/−0x128/−0x126；
//! 4. LoadObject（`{A80B5E91-7F9C-11D0-9410-080009EEBECB}` = ITextSymbol，
//!    类型库 esriDisplay）→ this−0x124：标签符号候选；
//! 5. int32 → this−0x120；
//! 6. int16 → this−0x11c；
//! 7. LoadObject（ITextSymbol）→ this−0x118：第二标签符号候选；
//! 8. LoadObject（`{52AA5BC2-CB21-11D0-87F9-080009EC732A}` = ILegendClass
//!    Format）→ this−0x114：**新对象且类标识为 LegendClassFormat 时由
//!    [`crate::layers::legend_format`] 整体消费**（本轮闭合；其余类停止）；
//! 9. int16 ×2 → this−0x110/−0x10e；
//! 10. v2 < 2：**不读流**，内存写默认（0、0、" ("、")"、0xFFFF，`Load`
//!     汇编证实；写侧 docVersion 对应版本恒 ≥2，旧版默认值无流内字节）；
//!     v2 ≥ 2：int16 → this−0xce、int16 → this−0xcc、String →
//!     this−0xc8、String → this−0xc4、int16 → this−0xc0；
//! 11. v2 > 2：LoadObject（`{7E4F470D-8E54-11D2-AAD8-000000000000}` =
//!     INumberFormat，类型库 esriSystem）→ this−0xbc；
//! 12. （this−0x10c 处 8 字节内存清零，无流内字节。）
//!
//! Save（LayerLib `0x786a2df0`）逐段对称：版本 short（上表）→
//! SaveObject(this−0x130) → short ×4 → SaveObject(this−0x124) → int32 →
//! short → SaveObject(this−0x118) → SaveObject(this−0x114) → short ×2 →
//! [保存版本 >1] short/String/String/short → [保存版本 >2]
//! SaveObject(this−0xbc)。
//!
//! ## 样本边界
//!
//! 仓库参考样本 `samples/测试底图.mxd` /Maps 流中本类 CLSID 0 命中
//! （样本无图例对象，Legend 条目循环计数为 0）；全部布局结论**无样本
//! 字节佐证**，仅有反编译一手证据，单元测试全部使用合成输入。
//!
//! 调用约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头、比对 [`HORIZONTAL_LEGEND_ITEM_CLASS_ID`] 并取得键，从对象
//! 头之后的位置传入 Reader。对象槽经
//! [`crate::storage::object_refs::read_object_entry_10_1`] 分类：空标记/
//! 回引消费后继续；ITextSymbol/INumberFormat 槽的 TextSymbol/NumericFormat
//! 新对象整体消费（C 轮与 legend_item_family 口径统一），其余新对象（ILayer
//! 槽与其余实现类）为停止点（游标回退到槽位起点、该键不登记，其后字段不
//! 可达）；ILegendClassFormat 槽的新对象为 LegendClassFormat 类时整体消费
//! （体内停止透传）。任何失败不推进调用者游标、不修改登记表。`max_bytes`
//! 约束两个字符串的声明长度，`usize::MAX` 表示不额外设限。
//!
//! C 轮接线备注：BaseLegendItem 字段序读取器
//! [`read_base_legend_item_fields_10_1`] 为本字段序的唯一实现
//! （legend_item_family 的本地副本已消除，三类经它复用）；槽位分派口径与
//! legend_item_family 轮研究记录一致（ITextSymbol/INumberFormat 闭合类整体
//! 消费）。
//!
//! 未解决问题（详见研究记录「开放项」）：条目 int32（对象 -8）、四个
//! int16、int32/int16 组、≥2 组 5 字段与两个 String 的业务语义均未定性
//! （≥2 组默认 " ("/")" 提示名称前后缀候选）；ILayer 槽在真实流中的条目
//! 分布未观察；BaseLegendItem 版本 0/1 分支无字节佐证；全部结论为静态
//! 观察，未经目标软件运行验证。

use crate::layers::legend_format::{
    LEGEND_CLASS_FORMAT_CLASS_ID, LegendClassFormatBody10_1, LegendClassFormatStop10_1,
    read_legend_class_format_body_10_1,
};
use crate::objects::element_body::{
    TEXT_SYMBOL_CLASS_ID, TextSymbolBody10_1, read_text_symbol_body_10_1,
};
use crate::objects::number_format::{
    NUMERIC_FORMAT_CLASS_ID, NumericFormatBody10_1, read_numeric_format_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// HorizontalLegendItem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{A9401A47-4649-11D1-880B-080009EC732A}`。证据：
/// esri.clsid.ecfg 声明（模块 Renderers.dll，类名 HorizontalLegendItem）；
/// Renderers.dll 10.1.0.3035 GetClassID `0x7cad4c30` 写出同一 CLSID；
/// RTTI TD `0x7caf7970` 名 `.?AVHorizontalLegendItem@@`。仓库参考样本
/// /Maps 流 0 命中。
pub const HORIZONTAL_LEGEND_ITEM_CLASS_ID: [u8; 16] = [
    0x47, 0x1a, 0x40, 0xa9, 0x49, 0x46, 0xd1, 0x11, 0x88, 0x0b, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// 条目版本（Load thunk 本体 `0x7cad50e0` 有符号 `1 < v` 拒绝）；Save
/// 恒写 1。
const MAX_ITEM_VERSION: i16 = 1;

/// BaseLegendItem 版本 short：Save 侧版本表 `[1,1,1,1,1,3]`（N=6）在
/// 10.1（docVer=5）恒写 3；>3 与负值无写侧来源，读取器一并拒绝。
const MAX_BASE_VERSION: i16 = 3;

/// ILayer / ITextSymbol / INumberFormat 槽位的分类结果。
///
/// C 轮起与 legend_item_family 口径统一：TextSymbol/NumericFormat 新对象
/// 经各自闭合读取器整体消费（本轮接线建议），其余实现类一律停止。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendItemSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TextSymbol 对象（ITextSymbol 槽；体已闭合，整体消费）。
    TextSymbol(Box<TextSymbolBody10_1>),
    /// 新 NumericFormat 对象（INumberFormat 槽；体已闭合，整体消费）。
    NumericFormat(Box<NumericFormatBody10_1>),
    /// 新对象但类标识非本读取器可闭合类：体未知且无外部长度，停止点，
    /// 游标回退到该对象头起点、该键不登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 第 8 步 ILegendClassFormat 槽的分类结果：新对象为 LegendClassFormat
/// 类时由 [`read_legend_class_format_body_10_1`] 整体消费。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendItemClassFormatSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识为 LegendClassFormat：已读取其体（本轮闭合）；
    /// 体内停止时原因透传为 [`HorizontalLegendItemStop10_1::ClassFormat`]。
    ClassFormat(Box<LegendClassFormatBody10_1>),
    /// 新对象且类标识非 LegendClassFormat：停止点，游标回退到槽位起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// BaseLegendItem 版本 ≥2 组的 5 个字段（v2 <2 时原生写默认、不读流）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendItemTailGroup10_1 {
    /// this−0xce 的 int16 原样位模式。
    pub short_a: u16,
    /// this−0xcc 的 int16 原样位模式。
    pub short_b: u16,
    /// this−0xc8 的 String（<2 时默认 " ("，读侧恒读）。
    pub prefix: String,
    /// this−0xc4 的 String（<2 时默认 ")"）。
    pub suffix: String,
    /// this−0xc0 的 int16 原样位模式（<2 时默认 0xFFFF）。
    pub short_c: u16,
}

/// BaseLegendItem 字段序读取的停止原因（非错误：已消费内容有效）。
///
/// C 轮起为本字段序的唯一读取器（legend_item_family 三类复用本类型，本地
/// 副本已删除）。
#[derive(Debug, Clone, PartialEq)]
pub enum BaseLegendItemStop10_1 {
    /// 对象槽（ILayer/ITextSymbol/INumberFormat）新对象类体未闭合；
    /// 游标已回退到该槽位起点。
    SlotUnsupported {
        /// 槽位名（诊断用）。
        slot: &'static str,
        class_id: [u8; 16],
        key: u32,
    },
    /// ILegendClassFormat 槽新对象类标识非 LegendClassFormat；游标已回退
    /// 到槽位起点。
    ClassFormatUnsupported { class_id: [u8; 16], key: u32 },
    /// ILegendClassFormat 槽的 LegendClassFormat 体自身停止，原因透传；
    /// 游标停在其停止点。
    ClassFormat(LegendClassFormatStop10_1),
}

/// BaseLegendItem 字段序（LayerLib.dll `0x786a2b00`）的已确认字段。
///
/// C 轮起为本字段序的唯一读取器：入口为基类版本 short（即条目版本/int32
/// 之后的位置），legend_item_family 三类复用本读取器与类型。
#[derive(Debug, Clone, PartialEq)]
pub struct BaseLegendItemFields10_1 {
    /// BaseLegendItem 版本 short 的无符号位模式（无拒绝门，分支用；10.1
    /// 写侧恒写 3；本读取器仅接受 0..=3）。
    pub base_version: u16,
    /// ILayer 槽。
    pub layer: LegendItemSlot10_1,
    /// int16 ×4 原样位模式。
    pub shorts_a: [u16; 4],
    /// ITextSymbol 槽（标签符号候选）。
    pub label_symbol: LegendItemSlot10_1,
    /// int32 原样位模式。
    pub raw_int32_a: i32,
    /// int16 原样位模式。
    pub short_b: u16,
    /// 第二 ITextSymbol 槽。
    pub second_symbol: LegendItemSlot10_1,
    /// ILegendClassFormat 槽。
    pub class_format: LegendItemClassFormatSlot10_1,
    /// int16 ×2 原样位模式。
    pub shorts_b: [u16; 2],
    /// v2 ≥2 组（v2 <2 时原生写默认、不读流，无对应字段）。
    pub tail_group: Option<LegendItemTailGroup10_1>,
    /// v2 >2 的 INumberFormat 槽。
    pub number_format: Option<LegendItemSlot10_1>,
    /// 停止原因；`None` = 字段序已完整消费。
    pub stop: Option<BaseLegendItemStop10_1>,
}

/// 读取 BaseLegendItem 字段序（LayerLib.dll `0x786a2b00`；入口为基类版本
/// short，即条目版本/int32 之后的位置）。
///
/// 整体事务：入口克隆 trial 游标与暂存登记表，全部成功（含停止点）才提交。
/// HorizontalLegendItem 与 legend_item_family 三类共用本入口（C 轮统一，
/// 消除 legend_item_family 的字段序本地副本）。
pub fn read_base_legend_item_fields_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<BaseLegendItemFields10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("BaseLegendItem 字段序 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 基类版本 short：原生无拒绝门（分支用）；仅接受 0..=3。
    let base_version = trial.read_u16_le()?;
    if (base_version as i16) > MAX_BASE_VERSION || (base_version as i16) < 0 {
        return Err(invalid(format!(
            "BaseLegendItem 版本 {base_version} 超出本读取器接受范围 0..=3（原生无拒绝门、\
             分支用；写侧版本表 DAT_7871ee18 = [1,1,1,1,1,3]，10.1 恒写 3，其余值无字节佐证）"
        )));
    }

    let mut staged_table = table.clone();
    let mut stop = None;

    // 2. ILayer 槽。
    let layer;
    {
        let slot_offset = trial.position();
        let snapshot = staged_table.clone();
        layer = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LegendItemSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BaseLegendItemStop10_1::SlotUnsupported {
                    slot: "layer(ILayer)",
                    class_id,
                    key,
                });
                trial = reposition_to(&reader.clone(), slot_offset);
                staged_table = snapshot;
                LegendItemSlot10_1::Unsupported { class_id, key }
            }
        };
    }

    // 3..7. int16 ×4 + ITextSymbol 槽 + int32 + int16 + 第二 ITextSymbol 槽
    //（停止时不可达）。
    let mut shorts_a = [0_u16; 4];
    let mut label_symbol = LegendItemSlot10_1::Null;
    let mut raw_int32_a = 0;
    let mut short_b = 0;
    if stop.is_none() {
        for slot in &mut shorts_a {
            *slot = trial.read_u16_le()?;
        }
        let symbol_offset = trial.position();
        let snapshot = staged_table.clone();
        label_symbol = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LegendItemSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == TEXT_SYMBOL_CLASS_ID => {
                let body = read_text_symbol_body_10_1(&mut trial, max_bytes, &mut staged_table)?;
                LegendItemSlot10_1::TextSymbol(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BaseLegendItemStop10_1::SlotUnsupported {
                    slot: "label_symbol(ITextSymbol)",
                    class_id,
                    key,
                });
                trial = reposition_to(&reader.clone(), symbol_offset);
                staged_table = snapshot;
                LegendItemSlot10_1::Unsupported { class_id, key }
            }
        };
    }
    if stop.is_none() {
        raw_int32_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        short_b = trial.read_u16_le()?;
    }

    // 7(续). 第二个 ITextSymbol 槽。
    let mut second_symbol = LegendItemSlot10_1::Null;
    if stop.is_none() {
        let symbol_offset = trial.position();
        let snapshot = staged_table.clone();
        second_symbol = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LegendItemSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == TEXT_SYMBOL_CLASS_ID => {
                let body = read_text_symbol_body_10_1(&mut trial, max_bytes, &mut staged_table)?;
                LegendItemSlot10_1::TextSymbol(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BaseLegendItemStop10_1::SlotUnsupported {
                    slot: "second_symbol(ITextSymbol)",
                    class_id,
                    key,
                });
                trial = reposition_to(&reader.clone(), symbol_offset);
                staged_table = snapshot;
                LegendItemSlot10_1::Unsupported { class_id, key }
            }
        };
    }

    // 8. ILegendClassFormat 槽：新对象为 LegendClassFormat 类时整体消费。
    let mut class_format = LegendItemClassFormatSlot10_1::Null;
    if stop.is_none() {
        let format_offset = trial.position();
        let snapshot = staged_table.clone();
        class_format = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => LegendItemClassFormatSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LegendItemClassFormatSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == LEGEND_CLASS_FORMAT_CLASS_ID =>
            {
                let body = read_legend_class_format_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged_table,
                    key,
                )?;
                if let Some(inner) = body.stop.clone() {
                    stop = Some(BaseLegendItemStop10_1::ClassFormat(inner));
                }
                LegendItemClassFormatSlot10_1::ClassFormat(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BaseLegendItemStop10_1::ClassFormatUnsupported { class_id, key });
                trial = reposition_to(&reader.clone(), format_offset);
                staged_table = snapshot;
                LegendItemClassFormatSlot10_1::Unsupported { class_id, key }
            }
        };
    }

    // 9. int16 ×2。
    let mut shorts_b = [0_u16; 2];
    if stop.is_none() {
        for slot in &mut shorts_b {
            *slot = trial.read_u16_le()?;
        }
    }

    // 10. v2 ≥2 组；v2 <2 时原生写默认、不读流。
    let mut tail_group = None;
    if stop.is_none() && base_version >= 2 {
        tail_group = Some(LegendItemTailGroup10_1 {
            short_a: trial.read_u16_le()?,
            short_b: trial.read_u16_le()?,
            prefix: read_string_10_1(&mut trial, max_bytes)?,
            suffix: read_string_10_1(&mut trial, max_bytes)?,
            short_c: trial.read_u16_le()?,
        });
    }

    // 11. v2 >2：INumberFormat 槽（NumericFormat 整体消费，其余类停止回退）。
    let mut number_format = None;
    if stop.is_none() && base_version > 2 {
        let slot_offset = trial.position();
        let snapshot = staged_table.clone();
        let slot = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LegendItemSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == NUMERIC_FORMAT_CLASS_ID => {
                let body = read_numeric_format_body_10_1(&mut trial, &mut staged_table, key)?;
                LegendItemSlot10_1::NumericFormat(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BaseLegendItemStop10_1::SlotUnsupported {
                    slot: "number_format(INumberFormat)",
                    class_id,
                    key,
                });
                trial = reposition_to(&reader.clone(), slot_offset);
                staged_table = snapshot;
                LegendItemSlot10_1::Unsupported { class_id, key }
            }
        };
        number_format = Some(slot);
    }

    let result = BaseLegendItemFields10_1 {
        base_version,
        layer,
        shorts_a,
        label_symbol,
        raw_int32_a,
        short_b,
        second_symbol,
        class_format,
        shorts_b,
        tail_group,
        number_format,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// HorizontalLegendItem 体读取的停止原因（非错误：已消费内容有效）。
///
/// C 轮起为 [`BaseLegendItemStop10_1`] 的别名（字段序读取与停止归约统一
/// 后，条目层不再自有停止类别）。
pub type HorizontalLegendItemStop10_1 = BaseLegendItemStop10_1;

/// esriCarto 10.1 的 esriLegendItemArrangement：图例色块、标签与描述的排列顺序。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendItemArrangement10_1 {
    PatchLabelDescription,
    PatchDescriptionLabel,
    LabelPatchDescription,
    LabelDescriptionPatch,
    DescriptionPatchLabel,
    DescriptionLabelPatch,
}

impl LegendItemArrangement10_1 {
    /// 解释已知排列值；未登记的值由调用者原样保留。
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::PatchLabelDescription),
            1 => Some(Self::PatchDescriptionLabel),
            2 => Some(Self::LabelPatchDescription),
            3 => Some(Self::LabelDescriptionPatch),
            4 => Some(Self::DescriptionPatchLabel),
            5 => Some(Self::DescriptionLabelPatch),
            _ => None,
        }
    }
}

/// HorizontalLegendItem（Renderers.dll thunk `0x7cad5d30` + LayerLib.dll
/// BaseLegendItem::Load `0x786a2b00`）对象的已确认字段。
///
/// 停止点之后的字段不可达，保持 `None`/0/空串。
#[derive(Debug, Clone, PartialEq)]
pub struct HorizontalLegendItemBody10_1 {
    /// 对象头键（由调用者在分类新对象时按「登记先于持久化」登记；本
    /// 读取器不重复登记，仅登记嵌套槽位的新键）。
    pub key: u32,
    /// 条目版本 short 的无符号位模式（原生有符号 `1 < v` 拒绝；Save 恒写
    /// 1；负值一并拒绝较原生更严）。
    pub version: u16,
    /// 第 2 步：IHorizontalLegendItem.Arrangement 的原始 int32（持久化接口 −8）。
    pub raw_int32: i32,
    /// BaseLegendItem 版本 short 的无符号位模式（无拒绝门，分支用；10.1
    /// 写侧恒写 3；本读取器仅接受 0..=3）。
    pub base_version: u16,
    /// 第 2 步（基类）ILayer 槽。
    pub layer: LegendItemSlot10_1,
    /// 基类第 3 步四个 int16 原样位模式。
    pub shorts_a: [u16; 4],
    /// 基类第 4 步 ITextSymbol 槽（标签符号候选）。
    pub label_symbol: LegendItemSlot10_1,
    /// 基类第 5 步 int32 原样位模式。
    pub raw_int32_b: i32,
    /// 基类第 6 步 int16 原样位模式。
    pub short_b: u16,
    /// 基类第 7 步 ITextSymbol 槽（第二标签符号候选）。
    pub second_symbol: LegendItemSlot10_1,
    /// 基类第 8 步 ILegendClassFormat 槽。
    pub class_format: LegendItemClassFormatSlot10_1,
    /// 基类第 9 步两个 int16 原样位模式。
    pub shorts_b: [u16; 2],
    /// v2 ≥2 组（v2 <2 时原生写默认、不读流，无对应字段）。
    pub tail_group: Option<LegendItemTailGroup10_1>,
    /// v2 >2 的 INumberFormat 槽。
    pub number_format: Option<LegendItemSlot10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<HorizontalLegendItemStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

impl HorizontalLegendItemBody10_1 {
    /// 返回该图例项记录的排列顺序；未知值仍可通过 `raw_int32` 读取。
    pub fn arrangement(&self) -> Option<LegendItemArrangement10_1> {
        LegendItemArrangement10_1::from_raw(self.raw_int32)
    }
}

/// 从对象头之后的位置读取 10.1 HorizontalLegendItem 对象体，失败时不
/// 推进游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`HORIZONTAL_LEGEND_ITEM_CLASS_ID`]
/// 并取得键 `key`。字段序与停止规则见模块文档；`max_bytes` 约束两个
/// String 的声明长度（`usize::MAX` 不设限）。对象槽停止为成功返回
/// （游标回退到槽位起点）；截断、版本超限为错误，游标与登记表不动。
///
/// C 轮起条目版本/int32 之后的基类字段序经
/// [`read_base_legend_item_fields_10_1`] 统一读取（与 legend_item_family
/// 三类共用，消除本地副本）。
pub fn read_horizontal_legend_item_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<HorizontalLegendItemBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("HorizontalLegendItem 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 条目版本 short：原生有符号 `1 < v` 拒绝（负值原生通过，一并拒绝
    //    较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ITEM_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "条目版本 {version} 超出 HorizontalLegendItem::Load 接受范围（有符号 >1 拒绝 \
             0x80004005；Save 恒写 1；负值位模式原生通过，此处一并拒绝）"
        )));
    }

    // 2. Arrangement int32（持久化接口 −8），保留未知值。
    let raw_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    // 3..11. 基类字段序（统一读取器）。
    let base = read_base_legend_item_fields_10_1(&mut trial, max_bytes, table)?;
    let result = HorizontalLegendItemBody10_1 {
        key,
        version,
        raw_int32,
        base_version: base.base_version,
        layer: base.layer,
        shorts_a: base.shorts_a,
        label_symbol: base.label_symbol,
        raw_int32_b: base.raw_int32_a,
        short_b: base.short_b,
        second_symbol: base.second_symbol,
        class_format: base.class_format,
        shorts_b: base.shorts_b,
        tail_group: base.tail_group,
        number_format: base.number_format,
        stop: base.stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}

/// 把游标回退到入口窗口内的绝对偏移（Reader 无随机定位原语，用入口克隆
/// 前向读取实现；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}
