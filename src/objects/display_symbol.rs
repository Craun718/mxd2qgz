//! ArcMap 10.1 DisplayCore.dll 简单符号族（SimpleMarkerSymbol/SimpleLineSymbol/
//! SimpleFillSymbol/MarkerLineSymbol/CartographicLineSymbol/HashLineSymbol）
//! 对象体的严格读取子集与符号槽位分派。
//!
//! 证据来源：`docs/arcmap-10.1-display-tails.md`（2026-11-15 本轮研究记录）与
//! `docs/arcmap-10.1-featurelayer-tail.md` 第七轮（SimpleMarkerSymbol）。全部
//! 字段序来自 DisplayCore.dll 10.1.0.3035（SHA-256 `9b3e61f9…a926e4d2`，复用
//! 前轮提取件 temp/agent-fl-tail/）的静态反编译；类身份经 RTTI 类型描述符 →
//! COL → 虚表 → GetClassID 立即数逐类确认，读写两侧对称闭合。
//!
//! **重要边界：仓库参考样本 `samples/测试底图.mxd` 只含 SimpleMarkerSymbol
//! 实例（第七轮闭合，原实现位于 `feature_layer_tail`，接线轮移入本模块），
//! 本模块其余五类符号无样本字节佐证**；只能用合成输入测试，不得声称样本
//! 验证。
//!
//! ## 字段序（Load 实体 / Save 实体逐字段对称）
//!
//! - **SimpleLineSymbol**（IPersistStream 虚表 `0x6ec15498`，Load thunk
//!   `0x6ebbda20` → 实体 `0x6ebbced0`，Save `0x6ebbd9f0` → `0x6ebbcfc0`，
//!   GetClassID `0x6ebbd090`）：
//!   ① short 版本（>1 拒绝 `0x80004005`）→ ② LoadObject（IColor
//!   `{20CD40B0-…}`，`DAT_6ec0b4f4` → `this−0x2c`）→ ③ `FUN_6eb406f0`
//!   （前轮已证不读流，返回值装入 `this−0xc`，无流内字节）→ ④ double
//!   （`this−0x24`，宽度候选）→ ⑤ int32（`this−0x28`，样式候选）→
//!   ⑥⑦ int32 ×2（`this−0x7c`/`this−0x74`，语义未定）。
//! - **SimpleFillSymbol**（虚表 `0x6ec152ac`，Load `0x6ebbc650` →
//!   `0x6ebbb4e0`，Save `0x6ebbc6c0` → `0x6ebbb5e0`，GetClassID
//!   `0x6ebbb6a0`）：
//!   ① short 版本（>1 拒绝）→ ② LoadObject（ISymbol `{F3435802-…}`
//!   `DAT_6ec0ca2c` → Save 槽 `this−0x24`，轮廓线符号候选）→ ③ LoadObject
//!   （IColor `DAT_6ec0b4f4` → Save 槽 `this−0x28`，填充色候选）→
//!   ④ `FUN_6eb406f0`（不读流）→ ⑤⑥⑦ int32 ×3（`this−0x7c` 样式候选/
//!   `this−0x74`/`this−0xc`）。
//! - **MarkerLineSymbol**（虚表 `0x6ec0eca0`，Load `0x6eb636a0` →
//!   `0x6eb62810`，Save `0x6eb63630` → `0x6eb629e0`，GetClassID
//!   `0x6eb61410`，版本常量 `DAT_6ec0b2e0`=2）：
//!   ① short 版本（>2 拒绝）→ ② byte（`this−0x20d8`）→ ③ double
//!   （`this−0x20d4`）→ ④ LoadObject（IMarkerSymbol `{E6BDAA7C-…}`
//!   `DAT_6ec0ca3c`，Save 槽 `this−0x20e0`）→ ⑤ `FUN_6eb68fc0`（本内
//!   释放/重建辅助，反编译无 AfStreamRead，无流内字节）→ ⑥ LoadObject
//!   （`{7914E5F4-…}`，ITemplate 候选，Save 槽 `this−0x20cc`）→
//!   ⑦ LoadObject（`{533D88F0-…}`，ILineDecoration 候选，Save 槽
//!   `this−0x20c8`）→ ⑧⑨ int32 ×2（`this−0x2134`/`this−0x212c`）→
//!   ⑩ byte（`this−0x20ac`）→ ⑪ double（`this−0x20a4`）→ ⑫ 版本 ≥2：
//!   int32 ×2（`this−0x2094`/`this−0x2090`）+ double（`this−0x208c`）。
//!   Save 恒写版本 2（含 ⑫）。
//! - **CartographicLineSymbol**（虚表 `0x6ec11ca8`，Load `0x6eb99e80` →
//!   `0x6eb99140`，Save `0x6eb99e00` → `0x6eb99330`，GetClassID
//!   `0x6eb99100`，版本常量 `DAT_6ec0b27c`=1）：
//!   ① short 版本（>1 拒绝）→ ②③ int32 ×2（`this−0x10a8`/`−0x10a4`）→
//!   ④ double（`this−0x109c`）→ ⑤ byte（`this−0x1094`）→ ⑥ double
//!   （`this−0x108c`）→ ⑦ LoadObject（IColor → Save 槽 `this−0x10ac`）→
//!   ⑧ `FUN_6eb406f0`（不读流 → bool `this−0x104c`）→ ⑨ LoadObject
//!   （`{7914E5F4-…}` ITemplate 候选 → `this−0x1084`）→ ⑩ LoadObject
//!   （`{533D88F0-…}` ILineDecoration 候选 → `this−0x107c`）→
//!   ⑪⑫ int32 ×2（`this−0x10fc`/`−0x10f4`）→ ⑬ byte（`this−0x1060`）→
//!   ⑭⑮ double ×2（`this−0x105c`/`−0x1054`）。
//! - **HashLineSymbol**（虚表 `0x6ec0e0b8`，Load `0x6eb56f30` →
//!   `0x6eb554a0`，Save `0x6eb56e90` → `0x6eb556d0`，GetClassID
//!   `0x6eb53c30`，版本常量 `DAT_6ec0b2f4`=1）：
//!   ① short 版本（>1 拒绝）→ ② double（`this−0x2124`）→ ③④ int32 ×2
//!   （`this−0x2118`/`−0x2114`）→ ⑤ double（`this−0x2104`）→ ⑥ byte
//!   （`this−0x20fc`）→ ⑦ double（`this−0x20f4`）→ ⑧ LoadObject（ISymbol
//!   `{F3435802-…}`，Save 槽 `this−0x20e4`）→ ⑨ LoadObject（IColor，Save
//!   槽 `this−0x211c`）→ ⑩ `FUN_6eb406f0`（不读流 → bool `this−0x2094`）→
//!   ⑪ LoadObject（`{7914E5F4-…}` ITemplate 候选 → `this−0x20ec`）→
//!   ⑫ LoadObject（`{533D88F0-…}` ILineDecoration 候选 → `this−0x20e8`）→
//!   ⑬⑭ int32 ×2（`this−0x217c`/`−0x2174`）→ ⑮ byte（`this−0x20a4`）→
//!   ⑯⑰ double ×2（`this−0x209c`/`−0x210c`）。
//!
//! LoadObject 槽的写入目标多数被 Ghidra 反编译隐藏（`FUN_6eb32c80` 辅助），
//! 槽归属以 Save 侧 `SaveObject(this+槽位)` 序对称印证；请求 IID 直接读自
//! 反编译引用的数据地址。`{7914E5F4}`/`{533D88F0}` 的接口名（ITemplate/
//! ILineDecoration 候选）未经类型库定名；esri.clsid 同族 coclass 为
//! Template `{41093A71}`、LineDecoration `{533D88F5}`——**这两个 coclass
//! 的对象体未闭合**，槽内新对象一律为停止点（空标记与回引可消费）。
//!
//! ## 槽位分派（对象层共享入口）
//!
//! [`read_display_symbol_slot_10_1`] 把一个对象条目按流内类标识分派到
//! SimpleMarkerSymbol（第七轮）与本模块五类符号。这是各宿主（SimpleRenderer
//! 的 ISymbol 槽、LegendClass 的 ISymbol 槽、SimpleFillSymbol 轮廓槽、
//! TextSymbol 的 ISymbol #2 槽等）共用的符号槽分派入口：未知符号类在分派
//! 内部回退到槽位起点并返回 [`DisplaySymbolSlot10_1::Unsupported`]，调用者
//! 据此维持停止语义。符号可嵌套符号（如 SimpleFillSymbol 轮廓槽内常见线符
//! 号），分派带深度参数：超过 [`MAX_SYMBOL_NESTING_DEPTH`] 报
//! `ErrorKind::InvalidInput`（防栈溢出的读取器自身限额，非原生限制；原生对
//! 嵌套深度无静态检查）。颜色槽经 [`crate::objects::display_color::
//! read_color_slot_10_1`] 分派（五类颜色全部闭合）。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头并比对类常量，从对象头之后的位置调用各 `read_*_body_10_1`；
//! 本模块不重复读取对象头。全部读取在 trial 游标与 trial 登记表副本上进行，
//! 完整成功（含停止点返回）才一并提交；任何错误既不推进游标、也不修改登记
//! 表。槽位新对象类未知时该槽位读取器内部回退到槽位起点（不保留该键），
//! 调用者以槽位 `Unsupported` 变体 / 本体 `stop` 字段识别并保留剩余字节。
//! 体内已闭合前缀的嵌套停止传播为本体停止原因（游标停在内层停止点，已
//! 闭合字段保留）。
//!
//! 未解决问题（详见研究记录开放项）：各 int32/double/byte 的业务语义全部
//! 未定名（样式/宽度等按符号学惯例推断）；`FUN_6eb406f0`/`FUN_6eb68fc0`
//! 的副作用仅确认不读流；ITemplate/ILineDecoration 槽的 coclass 体未闭合；
//! 全部结论为静态观察，未经目标软件运行验证，样本无字节佐证。

use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 线条/填充符号实体版本上限：`1 < ver` 有符号拒绝（四类共用；样本无实例）。
const MAX_SYMBOL_VERSION: i16 = 1;

/// SimpleMarkerSymbol::Load 接受的最大版本：`2 < (short)ver` 有符号拒绝；
/// 样本 2。证据：DisplayCore.dll 实体 Load `0x6ebbefa0`（第七轮记录）。
const MAX_SIMPLE_MARKER_VERSION: i16 = 2;

/// 符号槽位分派的最大嵌套深度（防栈溢出的读取器自身限额，非原生限制）。
pub const MAX_SYMBOL_NESTING_DEPTH: usize = 16;

/// SimpleMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid.ecfg 声明（模块 DisplayCore.dll）；DisplayCore.dll GetClassID
/// `0x6ebbf310` 写出；样本 `/Maps:0x403`（SimpleRenderer 体内 ISymbol 槽，
/// 键 36）与 LegendClass 体内的回引头逐字节相同（辅助观察）。
pub const SIMPLE_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfe, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MarkerLineSymbol 实体版本上限：`2 < ver` 有符号拒绝；Save 恒写 2。
const MAX_MARKER_LINE_VERSION: i16 = 2;

/// SimpleLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5F9-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll，progID esriDisplay.SimpleLineSymbol）；
/// GetClassID `0x6ebbd090` 写出（IPersistStream 虚表 `0x6ec15498` 槽 +0x0c
/// thunk `0x6ebbd9a0`）。样本中无实例，无字节佐证。
pub const SIMPLE_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xf9, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// SimpleFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E603-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6ebbb6a0` 写出（虚表
/// `0x6ec152ac` 槽 +0x0c thunk `0x6ebbc660`）。样本中无实例，无字节佐证。
pub const SIMPLE_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x03, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MarkerLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FD-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb61410` 写出（虚表
/// `0x6ec0eca0` 槽 +0x0c thunk `0x6eb636e0`）。样本中无实例，无字节佐证。
pub const MARKER_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfd, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// CartographicLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FB-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb99100` 写出（虚表
/// `0x6ec11ca8` 槽 +0x0c thunk `0x6eb99e90`）。样本中无实例，无字节佐证。
pub const CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfb, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// HashLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FC-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb53c30` 写出（虚表
/// `0x6ec0e0b8` 槽 +0x0c thunk `0x6eb56f50`）。样本中无实例，无字节佐证。
pub const HASH_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfc, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// 请求 IID `{7914E5F4-C892-11D0-8BB6-080009EE4E41}` 的 16 字节内存序
/// （诊断对照用，不校验流内字节）。
///
/// MarkerLine/CartographicLine/HashLine 三类 Load 的中段 LoadObject 请求该
/// IID（反编译 `DAT_6ec0e0e8`）。接口名未定名；按符号属性面推断为 ITemplate
/// 候选（三类均有模板属性）。未经类型库定名，仅作对照。
pub const TEMPLATE_IID_CANDIDATE: [u8; 16] = [
    0xf4, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// 请求 IID `{533D88F0-0A1A-11D2-B27F-0000F878229E}` 的 16 字节内存序
/// （诊断对照用，不校验流内字节）。
///
/// 同三类 Load 的尾段 LoadObject 请求该 IID（反编译 `DAT_6ec0e0f8`）；esri
/// .clsid 中 LineDecoration coclass 为 `{533D88F5-0A1A-11D2-B27F-0000F878229E}`
/// （同族 Data1 相邻），接口名候选 ILineDecoration，未经类型库定名。
pub const LINE_DECORATION_IID_CANDIDATE: [u8; 16] = [
    0xf0, 0x88, 0x3d, 0x53, 0x1a, 0x0a, 0xd2, 0x11, 0xb2, 0x7f, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];

/// 读取 int16 版本并按原生有符号比较校验上限；负值位模式一并拒绝。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    what: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let version = trial.read_u16_le()?;
    let signed = version as i16;
    if signed > max || signed < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{what} 对象体 offset 0x{entry_offset:x}: 版本 {version} 超出 \
                 {what}::Load 接受范围（有符号比较 >{max} 拒绝；负值位模式一并拒绝）"
            ),
        ));
    }
    Ok(version)
}

/// 读取 4 字节小端 int32。
fn read_i32_le(trial: &mut Reader<'_>) -> io::Result<i32> {
    Ok(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
}

/// 槽位在体字段序中的类别（停止原因与诊断用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySlotKind10_1 {
    /// ISymbol 槽（SimpleFillSymbol 轮廓、HashLineSymbol 首槽）。
    Symbol,
    /// IMarkerSymbol 槽（MarkerLineSymbol 首槽）。
    MarkerSymbol,
    /// IColor 槽（各符号的颜色槽）。
    Color,
    /// `{7914E5F4}` ITemplate 候选槽（coclass 体未闭合，新对象即停止）。
    Template,
    /// `{533D88F0}` ILineDecoration 候选槽（coclass 体未闭合，新对象即停止）。
    LineDecoration,
}

/// 符号体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolStop10_1 {
    /// 本体内某槽位出现未闭合的新对象类（游标已由槽位读取器回退到该槽位
    /// 起点，槽位及其后的本体字段未消费）。
    Slot {
        kind: DisplaySlotKind10_1,
        class_id: [u8; 16],
        key: u32,
    },
    /// 嵌套符号体内出现停止（游标停在内层停止点；内层停止原因见嵌套体
    /// 自身的 `stop` 字段，逐层可查）。
    NestedIncomplete { class_id: [u8; 16], key: u32 },
}

/// 归约符号类槽位的停止：`Unsupported` → 本体槽位停止；嵌套体含 `stop` →
/// 嵌套未完整停止；其余无停止。`kind` 为调用者槽位的类别（进入停止原因）。
/// 公开给宿主（渲染器/图例类/元素体）在分派后透传停止原因。
pub fn slot_stop_10_1(
    slot: &DisplaySymbolSlot10_1,
    kind: DisplaySlotKind10_1,
) -> Option<SymbolStop10_1> {
    match slot {
        DisplaySymbolSlot10_1::Unsupported { class_id, key } => Some(SymbolStop10_1::Slot {
            kind,
            class_id: *class_id,
            key: *key,
        }),
        DisplaySymbolSlot10_1::SimpleMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::SimpleLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::SimpleFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::MarkerLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MARKER_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::CartographicLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::HashLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: HASH_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        _ => None,
    }
}

/// 归约颜色槽位的停止：`Unsupported` → 本体槽位停止；其余无停止。
fn color_slot_stop(color: &ColorSlot10_1) -> Option<SymbolStop10_1> {
    match color {
        ColorSlot10_1::Unsupported { class_id, key } => Some(SymbolStop10_1::Slot {
            kind: DisplaySlotKind10_1::Color,
            class_id: *class_id,
            key: *key,
        }),
        _ => None,
    }
}

/// SimpleMarkerSymbol 体的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleMarkerSymbolStop10_1 {
    /// 首个 IColor 槽的新对象类标识不在五类闭合颜色集合（同族未知颜色类）：
    /// 游标回退到该槽位起点，其后的标量字段不可达。
    ColorUnsupported { class_id: [u8; 16], key: u32 },
    /// 第二个 IColor 槽（轮廓色）同类停止；首个颜色与标量段已消费。
    OutlineColorUnsupported { class_id: [u8; 16], key: u32 },
}

/// SimpleMarkerSymbol（DisplayCore.dll `0x6ebbefa0`，第七轮闭合）体的已确认
/// 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMarkerSymbolPrefix10_1 {
    /// 对象头键（样本 36）。
    pub key: u32,
    /// 体首 short 版本（样本 2）；有符号比较 >2 拒绝。
    pub version: u16,
    /// IColor 槽（主色）：空标记 / 回引 / 五类闭合颜色体 / 不支持停止。
    pub color: ColorSlot10_1,
    /// double → obj-0x6c（样本 6.0；尺寸候选）。`None` = 首色槽停止。
    pub size: Option<f64>,
    /// int32 → obj-0x70（样本 0），语义未定。
    pub int32_a: Option<i32>,
    /// int32 → obj-0xcc（样本 13；marker 样式候选），语义未定。
    pub int32_b: Option<i32>,
    /// int32 → obj-0xc4（样本 0），语义未定。
    pub int32_c: Option<i32>,
    /// double ×3 → obj-100/-0x5c/-0x54（样本 0/0/0；X/Y 偏移候选）。
    pub doubles: Option<[f64; 3]>,
    /// 1 字节 → obj-0x44（样本 1），语义未定。
    pub byte_flag: Option<u8>,
    /// double → obj-0x4c（样本 1.0；比例候选）。
    pub double_ratio: Option<f64>,
    /// 第二个 IColor 槽（轮廓色）。`None` = 在其之前已停止。
    pub outline_color: Option<ColorSlot10_1>,
    /// 版本 >1 的 short → obj-0xb4（样本 0xFFFF）。`None` = 版本 ≤1 或前置
    /// 停止未读取。
    pub rotate_short: Option<i16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SimpleMarkerSymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleMarkerSymbol 对象体，失败时不推进游
/// 标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_MARKER_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。两个 IColor 槽经 [`read_color_slot_10_1`] 分派（五类闭合
/// 颜色全部解析；接线轮起对象层共享，原实现位于 `feature_layer_tail`），
/// 新对象类标识不在闭合集合时是成功停止（停止原因携带类标识与键，游标回
/// 退到该槽位起点）；截断与版本超限为错误。
pub fn read_simple_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleMarkerSymbolPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged_table = table.clone();

    // 1. 版本 short（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SIMPLE_MARKER_VERSION,
        "SimpleMarkerSymbol",
        entry_offset,
    )?;

    // 2. 首个 IColor 槽（主色）：五类闭合颜色经分派整体解析；未知颜色类
    //    由分派内部回退到槽位起点、登记表不保留该键，此处映射为本体停止。
    let (color, stop) = match read_color_slot_10_1(&mut trial, max_bytes, &mut staged_table)? {
        ColorSlot10_1::Unsupported { class_id, key } => (
            ColorSlot10_1::Null,
            Some(SimpleMarkerSymbolStop10_1::ColorUnsupported { class_id, key }),
        ),
        other => (other, None),
    };

    // 3-8. 标量段：double、int32 ×3、double ×3、byte、double（前置停止时不
    //       读取）。
    let mut size = None;
    let mut int32_a = None;
    let mut int32_b = None;
    let mut int32_c = None;
    let mut doubles = None;
    let mut byte_flag = None;
    let mut double_ratio = None;
    let mut outline_color = None;
    let mut rotate_short = None;
    let mut stop = stop;
    if stop.is_none() {
        size = Some(trial.read_f64_le()?);
        int32_a = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        int32_b = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        int32_c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        let mut values = [0_f64; 3];
        for slot in values.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        doubles = Some(values);
        byte_flag = Some(trial.read_bytes(1)?[0]);
        double_ratio = Some(trial.read_f64_le()?);

        // 9. 第二个 IColor 槽（轮廓色）：同一分派；停止时首个颜色与标量段
        //    已消费并保留，游标停在该槽位起点。
        outline_color = Some(
            match read_color_slot_10_1(&mut trial, max_bytes, &mut staged_table)? {
                ColorSlot10_1::Unsupported { class_id, key } => {
                    stop =
                        Some(SimpleMarkerSymbolStop10_1::OutlineColorUnsupported { class_id, key });
                    ColorSlot10_1::Null
                }
                other => other,
            },
        );

        // 10. 版本 >1 的 short（前置停止时不读取）。
        if stop.is_none() && (version as i16) > 1 {
            rotate_short = Some(trial.read_u16_le()? as i16);
        }
    }

    let result = SimpleMarkerSymbolPrefix10_1 {
        key,
        version,
        color,
        size,
        int32_a,
        int32_b,
        int32_c,
        doubles,
        byte_flag,
        double_ratio,
        outline_color,
        rotate_short,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// SimpleLineSymbol（DisplayCore.dll `0x6ebbced0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// IColor 槽（请求 IID `{20CD40B0-…}`）：空标记 / 回引 / 五类闭合颜色体 /
    /// 不支持停止。
    pub color: ColorSlot10_1,
    /// double → this−0x24（宽度候选）。`None` = 颜色槽停止。
    pub width: Option<f64>,
    /// int32 → this−0x28（样式候选），语义未定。`None` = 前置停止。
    pub style: Option<i32>,
    /// int32 → this−0x7c，语义未定。`None` = 前置停止。
    pub int32_7c: Option<i32>,
    /// int32 → this−0x74，语义未定。`None` = 前置停止。
    pub int32_74: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_LINE_SYMBOL_CLASS_ID`] 并取
/// 得键 `key`。颜色槽为未闭合新对象时是成功停止（游标已回退到该槽位起点，
/// 其后标量字段不可达）；截断与版本超限为错误。
pub fn read_simple_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "SimpleLineSymbol",
        entry_offset,
    )?;
    // ② IColor 槽（读取器内部对未知类回退）。
    let color = read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?;
    let stop = color_slot_stop(&color);
    // ③ `FUN_6eb406f0` 无流内字节（前轮证据），跳过。
    // ④-⑦ 标量段（前置停止时不读取）。
    let (width, style, int32_7c, int32_74) = if stop.is_none() {
        let width = trial.read_f64_le()?;
        let style = read_i32_le(&mut trial)?;
        let a = read_i32_le(&mut trial)?;
        let b = read_i32_le(&mut trial)?;
        (Some(width), Some(style), Some(a), Some(b))
    } else {
        (None, None, None, None)
    };

    *table = staged;
    *reader = trial;
    Ok(SimpleLineSymbolBody10_1 {
        key,
        version,
        color,
        width,
        style,
        int32_7c,
        int32_74,
        stop,
    })
}

/// SimpleFillSymbol（DisplayCore.dll `0x6ebbb4e0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// ISymbol 槽（请求 IID `{F3435802-…}`，轮廓线符号候选）。
    pub outline: DisplaySymbolSlot10_1,
    /// IColor 槽（填充色候选，Save 侧槽 `this−0x28`）。`None` = 轮廓槽停止。
    pub color: Option<ColorSlot10_1>,
    /// int32 → this−0x7c（样式候选）。`None` = 前置停止。
    pub style: Option<i32>,
    /// int32 → this−0x74，语义未定。`None` = 前置停止。
    pub int32_74: Option<i32>,
    /// int32 → this−0xc，语义未定。`None` = 前置停止。
    pub int32_c: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleFillSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_FILL_SYMBOL_CLASS_ID`] 并取
/// 得键 `key`。轮廓槽（ISymbol）经 [`read_display_symbol_slot_10_1`] 分派，
/// 未闭合新对象或嵌套体停止时其后的颜色槽与标量不可达（成功停止）。
pub fn read_simple_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SimpleFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "SimpleFillSymbol",
        entry_offset,
    )?;
    // ② ISymbol 槽（轮廓线符号候选）。
    let outline = read_display_symbol_slot_10_1(&mut trial, max_bytes, &mut staged, depth - 1)?;
    let mut stop = slot_stop_10_1(&outline, DisplaySlotKind10_1::Symbol);
    // ③ IColor 槽 + ④-⑦ 标量段（前置停止时不读取）。
    let mut color = None;
    let mut style = None;
    let mut int32_74 = None;
    let mut int32_c = None;
    if stop.is_none() {
        color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
        if let Some(inner) = color_slot_stop(color.as_ref().unwrap()) {
            stop = Some(inner);
        } else {
            style = Some(read_i32_le(&mut trial)?);
            int32_74 = Some(read_i32_le(&mut trial)?);
            int32_c = Some(read_i32_le(&mut trial)?);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(SimpleFillSymbolBody10_1 {
        key,
        version,
        outline,
        color,
        style,
        int32_74,
        int32_c,
        stop,
    })
}

/// MarkerLineSymbol（DisplayCore.dll `0x6eb62810`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝（Save 恒写 2）。
    pub version: u16,
    /// byte → this−0x20d8，语义未定。
    pub byte_20d8: u8,
    /// double → this−0x20d4，语义未定。
    pub double_20d4: f64,
    /// IMarkerSymbol 槽（请求 IID `{E6BDAA7C-…}`）。
    pub marker_symbol: DisplaySymbolSlot10_1,
    /// `{7914E5F4}` ITemplate 候选槽；新对象 coclass 体未闭合 = `None` 停止。
    pub template: Option<DisplaySymbolSlot10_1>,
    /// `{533D88F0}` ILineDecoration 候选槽；新对象 coclass 体未闭合 = `None`
    /// 停止。`None` 亦表示前置停止未读取。
    pub line_decoration: Option<DisplaySymbolSlot10_1>,
    /// int32 → this−0x2134，语义未定。`None` = 前置停止。
    pub int32_2134: Option<i32>,
    /// int32 → this−0x212c，语义未定。`None` = 前置停止。
    pub int32_212c: Option<i32>,
    /// byte → this−0x20ac，语义未定。`None` = 前置停止。
    pub byte_20ac: Option<u8>,
    /// double → this−0x20a4，语义未定。`None` = 前置停止。
    pub double_20a4: Option<f64>,
    /// 版本 ≥2 的 int32 → this−0x2094。`None` = 版本 ≤1 或前置停止。
    pub int32_2094: Option<i32>,
    /// 版本 ≥2 的 int32 → this−0x2090。`None` = 版本 ≤1 或前置停止。
    pub int32_2090: Option<i32>,
    /// 版本 ≥2 的 double → this−0x208c。`None` = 版本 ≤1 或前置停止。
    pub double_208c: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MarkerLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`MARKER_LINE_SYMBOL_CLASS_ID`] 并取
/// 得键 `key`。ITemplate/ILineDecoration 候选槽的新对象（coclass 体未闭合）
/// 为成功停止（游标已回退到该槽位起点）；版本 ≥2 的尾段在前置停止时不读取。
pub fn read_marker_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MarkerLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_MARKER_LINE_VERSION,
        "MarkerLineSymbol",
        entry_offset,
    )?;
    // ②③ byte + double。
    let byte_20d8 = trial.read_bytes(1)?[0];
    let double_20d4 = trial.read_f64_le()?;
    // ④ IMarkerSymbol 槽（⑤ `FUN_6eb68fc0` 无流内字节，跳过）。
    let marker_symbol =
        read_display_symbol_slot_10_1(&mut trial, max_bytes, &mut staged, depth - 1)?;
    let mut stop = slot_stop_10_1(&marker_symbol, DisplaySlotKind10_1::MarkerSymbol);
    // ⑥⑦ ITemplate / ILineDecoration 候选槽（coclass 体未闭合）。
    let mut template = None;
    let mut line_decoration = None;
    if stop.is_none() {
        let (slot, inner) =
            read_unclosed_object_slot_10_1(&mut trial, &mut staged, DisplaySlotKind10_1::Template)?;
        template = slot;
        if let Some(inner) = inner {
            stop = Some(inner);
        } else {
            let (slot, inner) = read_unclosed_object_slot_10_1(
                &mut trial,
                &mut staged,
                DisplaySlotKind10_1::LineDecoration,
            )?;
            line_decoration = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    // ⑧-⑪ 标量段。
    let mut int32_2134 = None;
    let mut int32_212c = None;
    let mut byte_20ac = None;
    let mut double_20a4 = None;
    // ⑫ 版本 ≥2 尾段。
    let mut int32_2094 = None;
    let mut int32_2090 = None;
    let mut double_208c = None;
    if stop.is_none() {
        int32_2134 = Some(read_i32_le(&mut trial)?);
        int32_212c = Some(read_i32_le(&mut trial)?);
        byte_20ac = Some(trial.read_bytes(1)?[0]);
        double_20a4 = Some(trial.read_f64_le()?);
        if version as i16 >= 2 {
            int32_2094 = Some(read_i32_le(&mut trial)?);
            int32_2090 = Some(read_i32_le(&mut trial)?);
            double_208c = Some(trial.read_f64_le()?);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(MarkerLineSymbolBody10_1 {
        key,
        version,
        byte_20d8,
        double_20d4,
        marker_symbol,
        template,
        line_decoration,
        int32_2134,
        int32_212c,
        byte_20ac,
        double_20a4,
        int32_2094,
        int32_2090,
        double_208c,
        stop,
    })
}

/// CartographicLineSymbol（DisplayCore.dll `0x6eb99140`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct CartographicLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// int32 → this−0x10a8，语义未定。
    pub int32_10a8: i32,
    /// int32 → this−0x10a4，语义未定。
    pub int32_10a4: i32,
    /// double → this−0x109c，语义未定。
    pub double_109c: f64,
    /// byte → this−0x1094，语义未定。
    pub byte_1094: u8,
    /// double → this−0x108c，语义未定。
    pub double_108c: f64,
    /// IColor 槽（Save 槽 this−0x10ac）。`None` = 前置停止（正常流不可能，
    /// 标量段先于颜色槽，保留 defensive 语义）。
    pub color: Option<ColorSlot10_1>,
    /// `{7914E5F4}` ITemplate 候选槽；新对象 = `None` 停止。
    pub template: Option<DisplaySymbolSlot10_1>,
    /// `{533D88F0}` ILineDecoration 候选槽；新对象 = `None` 停止。
    pub line_decoration: Option<DisplaySymbolSlot10_1>,
    /// int32 → this−0x10fc，语义未定。`None` = 前置停止。
    pub int32_10fc: Option<i32>,
    /// int32 → this−0x10f4，语义未定。`None` = 前置停止。
    pub int32_10f4: Option<i32>,
    /// byte → this−0x1060，语义未定。`None` = 前置停止。
    pub byte_1060: Option<u8>,
    /// double → this−0x105c，语义未定。`None` = 前置停止。
    pub double_105c: Option<f64>,
    /// double → this−0x1054，语义未定。`None` = 前置停止。
    pub double_1054: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 CartographicLineSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID`]
/// 并取得键 `key`。ITemplate/ILineDecoration 候选槽新对象为成功停止。
pub fn read_cartographic_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<CartographicLineSymbolBody10_1> {
    let _ = depth; // 颜色槽无嵌套下钻；符号类分派不需要深度
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "CartographicLineSymbol",
        entry_offset,
    )?;
    // ②-⑥ 标量段。
    let int32_10a8 = read_i32_le(&mut trial)?;
    let int32_10a4 = read_i32_le(&mut trial)?;
    let double_109c = trial.read_f64_le()?;
    let byte_1094 = trial.read_bytes(1)?[0];
    let double_108c = trial.read_f64_le()?;
    // ⑦ IColor 槽（⑧ `FUN_6eb406f0` 无流内字节，跳过）。
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    // ⑨⑩ ITemplate / ILineDecoration 候选槽。
    let mut template = None;
    let mut line_decoration = None;
    if stop.is_none() {
        let (slot, inner) =
            read_unclosed_object_slot_10_1(&mut trial, &mut staged, DisplaySlotKind10_1::Template)?;
        template = slot;
        if let Some(inner) = inner {
            stop = Some(inner);
        } else {
            let (slot, inner) = read_unclosed_object_slot_10_1(
                &mut trial,
                &mut staged,
                DisplaySlotKind10_1::LineDecoration,
            )?;
            line_decoration = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    // ⑪-⑮ 标量段（前置停止时不读取）。
    let mut int32_10fc = None;
    let mut int32_10f4 = None;
    let mut byte_1060 = None;
    let mut double_105c = None;
    let mut double_1054 = None;
    if stop.is_none() {
        int32_10fc = Some(read_i32_le(&mut trial)?);
        int32_10f4 = Some(read_i32_le(&mut trial)?);
        byte_1060 = Some(trial.read_bytes(1)?[0]);
        double_105c = Some(trial.read_f64_le()?);
        double_1054 = Some(trial.read_f64_le()?);
    }

    *table = staged;
    *reader = trial;
    Ok(CartographicLineSymbolBody10_1 {
        key,
        version,
        int32_10a8,
        int32_10a4,
        double_109c,
        byte_1094,
        double_108c,
        color,
        template,
        line_decoration,
        int32_10fc,
        int32_10f4,
        byte_1060,
        double_105c,
        double_1054,
        stop,
    })
}

/// HashLineSymbol（DisplayCore.dll `0x6eb554a0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct HashLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// double → this−0x2124，语义未定。
    pub double_2124: f64,
    /// int32 → this−0x2118，语义未定。
    pub int32_2118: i32,
    /// int32 → this−0x2114，语义未定。
    pub int32_2114: i32,
    /// double → this−0x2104，语义未定。
    pub double_2104: f64,
    /// byte → this−0x20fc，语义未定。
    pub byte_20fc: u8,
    /// double → this−0x20f4，语义未定。
    pub double_20f4: f64,
    /// ISymbol 槽（Save 槽 this−0x20e4，语义候选：被刻线的线符号）。
    pub symbol: Option<DisplaySymbolSlot10_1>,
    /// IColor 槽（Save 槽 this−0x211c）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// `{7914E5F4}` ITemplate 候选槽；新对象 = `None` 停止。
    pub template: Option<DisplaySymbolSlot10_1>,
    /// `{533D88F0}` ILineDecoration 候选槽；新对象 = `None` 停止。
    pub line_decoration: Option<DisplaySymbolSlot10_1>,
    /// int32 → this−0x217c，语义未定。`None` = 前置停止。
    pub int32_217c: Option<i32>,
    /// int32 → this−0x2174，语义未定。`None` = 前置停止。
    pub int32_2174: Option<i32>,
    /// byte → this−0x20a4，语义未定。`None` = 前置停止。
    pub byte_20a4: Option<u8>,
    /// double → this−0x209c，语义未定。`None` = 前置停止。
    pub double_209c: Option<f64>,
    /// double → this−0x210c，语义未定。`None` = 前置停止。
    pub double_210c: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 HashLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`HASH_LINE_SYMBOL_CLASS_ID`] 并取得
/// 键 `key`。ISymbol 槽经 [`read_display_symbol_slot_10_1`] 分派（未知类或
/// 嵌套体停止时其后的槽位与标量不可达）；ITemplate/ILineDecoration 候选槽
/// 新对象为成功停止。
pub fn read_hash_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<HashLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "HashLineSymbol",
        entry_offset,
    )?;
    // ②-⑦ 标量段。
    let double_2124 = trial.read_f64_le()?;
    let int32_2118 = read_i32_le(&mut trial)?;
    let int32_2114 = read_i32_le(&mut trial)?;
    let double_2104 = trial.read_f64_le()?;
    let byte_20fc = trial.read_bytes(1)?[0];
    let double_20f4 = trial.read_f64_le()?;
    // ⑧ ISymbol 槽。
    let symbol = Some(read_display_symbol_slot_10_1(
        &mut trial,
        max_bytes,
        &mut staged,
        depth - 1,
    )?);
    let mut stop = slot_stop_10_1(symbol.as_ref().unwrap(), DisplaySlotKind10_1::Symbol);
    // ⑨ IColor 槽（⑩ `FUN_6eb406f0` 无流内字节，跳过）。
    let mut color = None;
    if stop.is_none() {
        color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
        if let Some(inner) = color_slot_stop(color.as_ref().unwrap()) {
            stop = Some(inner);
        }
    }
    // ⑪⑫ ITemplate / ILineDecoration 候选槽。
    let mut template = None;
    let mut line_decoration = None;
    if stop.is_none() {
        let (slot, inner) =
            read_unclosed_object_slot_10_1(&mut trial, &mut staged, DisplaySlotKind10_1::Template)?;
        template = slot;
        if let Some(inner) = inner {
            stop = Some(inner);
        } else {
            let (slot, inner) = read_unclosed_object_slot_10_1(
                &mut trial,
                &mut staged,
                DisplaySlotKind10_1::LineDecoration,
            )?;
            line_decoration = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    // ⑬-⑰ 标量段（前置停止时不读取）。
    let mut int32_217c = None;
    let mut int32_2174 = None;
    let mut byte_20a4 = None;
    let mut double_209c = None;
    let mut double_210c = None;
    if stop.is_none() {
        int32_217c = Some(read_i32_le(&mut trial)?);
        int32_2174 = Some(read_i32_le(&mut trial)?);
        byte_20a4 = Some(trial.read_bytes(1)?[0]);
        double_209c = Some(trial.read_f64_le()?);
        double_210c = Some(trial.read_f64_le()?);
    }

    *table = staged;
    *reader = trial;
    Ok(HashLineSymbolBody10_1 {
        key,
        version,
        double_2124,
        int32_2118,
        int32_2114,
        double_2104,
        byte_20fc,
        double_20f4,
        symbol,
        color,
        template,
        line_decoration,
        int32_217c,
        int32_2174,
        byte_20a4,
        double_209c,
        double_210c,
        stop,
    })
}

/// ISymbol/IMarkerSymbol 类槽位条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DisplaySymbolSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 SimpleMarkerSymbol 对象（前轮闭合；体内停止经其 `stop` 传播）。
    SimpleMarkerSymbol(Box<SimpleMarkerSymbolPrefix10_1>),
    /// 新 SimpleLineSymbol 对象。
    SimpleLineSymbol(Box<SimpleLineSymbolBody10_1>),
    /// 新 SimpleFillSymbol 对象。
    SimpleFillSymbol(Box<SimpleFillSymbolBody10_1>),
    /// 新 MarkerLineSymbol 对象。
    MarkerLineSymbol(Box<MarkerLineSymbolBody10_1>),
    /// 新 CartographicLineSymbol 对象。
    CartographicLineSymbol(Box<CartographicLineSymbolBody10_1>),
    /// 新 HashLineSymbol 对象。
    HashLineSymbol(Box<HashLineSymbolBody10_1>),
    /// 新对象但类标识不在闭合集合：游标已回退到该槽位起点、登记表未保留
    /// 该键（本读取器内部完成回退，调用者据此停止）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 从当前位置读取一个符号类槽位对象条目并按类标识分派。
///
/// 支持的闭合类：SimpleMarkerSymbol（前轮）与 SimpleLineSymbol/
/// SimpleFillSymbol/MarkerLineSymbol/CartographicLineSymbol/HashLineSymbol
/// （本模块）。新对象类未知时回退到槽位起点并返回
/// [`DisplaySymbolSlot10_1::Unsupported`]。`depth` 为剩余允许嵌套层数：为 0
/// 时报 `ErrorKind::InvalidInput`（防栈溢出的读取器自身限额，非原生限制）。
/// 全部过程在 trial 游标与 trial 登记表上进行，任何失败不推进调用者游标、
/// 不修改登记表。
pub fn read_display_symbol_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DisplaySymbolSlot10_1> {
    let origin = reader.clone();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let result = match entry {
        ObjectEntry10_1::Null => DisplaySymbolSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            DisplaySymbolSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 深度限额在需要解析新对象体时才生效（空标记与回引不消耗深度）。
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "符号槽位 offset 0x{:x}: 嵌套深度超过读取器限额 \
                         {MAX_SYMBOL_NESTING_DEPTH}（防栈溢出，非原生限制）",
                        origin.position()
                    ),
                ));
            }
            let slot = if class_id == SIMPLE_MARKER_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::SimpleMarkerSymbol(Box::new(
                    read_simple_marker_symbol_body_10_1(&mut trial, max_bytes, &mut staged, key)?,
                ))
            } else if class_id == SIMPLE_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::SimpleLineSymbol(Box::new(
                    read_simple_line_symbol_body_10_1(&mut trial, max_bytes, &mut staged, key)?,
                ))
            } else if class_id == SIMPLE_FILL_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::SimpleFillSymbol(Box::new(
                    read_simple_fill_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == MARKER_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::MarkerLineSymbol(Box::new(
                    read_marker_line_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::CartographicLineSymbol(Box::new(
                    read_cartographic_line_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == HASH_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::HashLineSymbol(Box::new(read_hash_line_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else {
                // 未知符号类：回退到槽位起点，不保留该键。
                *reader = origin;
                return Ok(DisplaySymbolSlot10_1::Unsupported { class_id, key });
            };
            slot
        }
    };
    *reader = trial;
    *table = staged;
    Ok(result)
}

/// 读取 ITemplate 候选 / ILineDecoration 候选槽：空标记与回引可消费；新对象
///（coclass 体未闭合）回退到槽位起点、不保留该键，以停止原因返回。
///
/// 返回 `(槽位, 停止)`：停止非 `None` 时槽位为 `None`（新对象的类标识与键
/// 在停止原因中）。`staged` 为调用者（体读取器）的 trial 登记表；回退时
/// 一并恢复，任何失败不修改登记表。
fn read_unclosed_object_slot_10_1(
    trial: &mut Reader<'_>,
    staged: &mut RefTable10_1,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<DisplaySymbolSlot10_1>, Option<SymbolStop10_1>)> {
    let origin = trial.clone();
    let mut forward = trial.clone();
    let saved = staged.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(DisplaySymbolSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(DisplaySymbolSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // coclass 体未闭合：回退游标并丢弃该键登记。
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
