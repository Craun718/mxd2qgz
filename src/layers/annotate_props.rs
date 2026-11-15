//! ArcMap 10.1 FeatureLayer 尾段链 ALPC 环（标注属性集合族）对象体的严格
//! 读取子集：AnnotateLayerPropertiesCollection（ALPC）、
//! LabelEngineLayerProperties（LELP）与 EditTemplateManager（ETM）。
//!
//! 证据来源：`docs/arcmap-10.1-featurelayer-tail.md`（2026-11-15 第九轮研究
//! 记录）。链头是 `feature_layer_tail` 的 `read_feature_layer_successor_10_1`
//! 在 FieldInfo 循环完成处（样本流内 `0xafe`）遇到的 ALPC 新对象（键 59）。
//! 本模块闭合以下三类体（字段序全部来自 10.1.0.3035 原生 DLL 反编译，与
//! 样本 `temp/r6-maps.bin` `0xafe..0xb8e` 逐字节吻合）：
//!
//! 1. **AnnotateLayerPropertiesCollection**（LabelPlacement.dll
//!    10.1.0.3035，SHA-256 `8415f63c…35ea3d`，ImageBase `0x78530000`；
//!    GetClassID `0x78531ba0` 写出 CLSID `{1D5849F3-…}`，请求 IID
//!    `{1D5849F2-…}`=IAnnotateLayerPropertiesCollection）：
//!    ① int16 版本（`2 < (short)ver` 有符号拒绝；样本 2）→ ② int32 计数 →
//!    ③ 循环 [LoadObject（IAnnotateLayerProperties `{01004142-…}`）+
//!    （版本 ≥2）int32 条目键] → ④ 版本 ≥2：int32 尾 int32。样本：版本 2、
//!    计数 1、条目 = LELP 新对象（键 60）+ 条目键 int32 + 尾 int32。
//! 2. **LabelEngineLayerProperties**（LabelPlacement.dll，实现 Load
//!    `0x785468c0`，GetClassID 常量拷贝形、CLSID `{01004145-…}` 数据常量
//!    `0x78579a90` 域；esri.clsid 声明与样本 `/Maps:0xb1c` 头三方互证）：
//!    ① int16 版本（`7 < (short)ver` 拒绝；样本 7）→ ② String → ③ short
//!    （→ -0x74）→ ④ 版本 ≤1：short → ⑤ double（-0x54）→ ⑥ int32（-0x60）
//!    → ⑦ short（-0x5c）→ ⑧ short（-0x4c）→ ⑨ short（-0x4a）→ ⑩ int32
//!    （-0x80）→ ⑪ double（-0x34）→ ⑫ double（-0x3c）→ ⑬ short（-0x48）→
//!    ⑭ String → ⑮ String → ⑯ LoadObject（ITextSymbol `{A80B5E91-…}`）→
//!    ⑰ LoadObject（IBasicOverposterLayerProperties `{3141F2FB-…}`）→
//!    ⑱ LoadObject（IAnnotationExpressionEngine `{AA157206-…}`）→
//!    ⑲ LoadObject（IEnvelope `{1B077914-…}`）→ ⑳ double（-0x24）→
//!    ㉑ double（-0x1c）→ ㉒ int32（-0x14）→ ㉓ int32（-0x2c）→ ㉔ 版本 >2：
//!    LoadObject（IGraphicsContainer `{D09411D1-…}`）→ ㉕ 版本 ≥4：short
//!    （-0x46）→ ㉖ 版本 ≥5：int32（-0x7c）→ ㉗ 版本 ==6：short。
//!    样本：版本 7，串 1 = `[JZDH]`（标注表达式候选）、串 2 = `默认`（标注
//!    类名候选）、串 3 空，槽数据依次为 TextSymbol 新对象（键 61，未闭合 =
//!    停止点）、其后槽位未达。⑤-⑬/⑳-㉓ 各标量业务语义未定，原样保留。
//! 3. **EditTemplateManager**（FeatureLayer.dll，Load `0x6f6d0a80`、Save
//!    `0x6f6d0bc0` 对称，GetClassID `0x6f6d0a50` 写出 CLSID
//!    `{A0162E85-E170-4B10-A370-CEE08D8D4B8C}`；esri.clsid 声明；样本经
//!    FeatureLayer 后继段的图层扩展段装载，声明体长与逐字节消费一致）：
//!    ① int16 版本（`2 < (short)ver` 拒绝；样本 2；Save 版本表
//!    `DAT_6f77bf74` 按文档版本查询，0xFFFF 档不写）→ ② int32 计数 →
//!    ③ 循环 LoadObject（IEditTemplate `{0952F6C1-…}`；样本 = EditTemplate
//!    新对象，体由 [`crate::layers::edit_template`] 闭合读取）→ ④ 版本 ≥2：
//!    1 字节。**EditTemplate 的宿主即本类**（经 FeatureLayer 后继段的
//!    LayerLib.dll `ExtensionHelper::LoadExtensions` 图层扩展段装载），第
//!    八轮此前「宿主 = `SaveObject(+0x218)` 槽」的工作假设撤销：该槽请求
//!    IID 为 `{19F85377-…}`=IDisplayFilter（样本空标记）。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头并比对类常量，从对象头之后的位置调用各 `read_*_body_10_1`；
//! 嵌套条目用 [`crate::storage::object_refs::read_object_entry_10_1`] 分类
//! 并登记（登记先于对象体解析是原生语义；为满足「失败不修改登记表」，全部
//! 登记发生在调用者登记表的副本上，成功后整体提交）。槽位新对象类标识未知
//! 时读取器在该槽位起点停止并回退（不保留该键）；体内已闭合前缀的停止传播
//! 为外层停止原因，游标停在内层停止点。
//!
//! 未解决问题（详见研究记录「未解决事项」）：
//! - LELP ⑯ ITextSymbol 槽：TextSymbol（`{B65A3E74-…}`，DisplayCore.dll，
//!   GetClassID 实体 `0x6ebd08e0` 经调整器 thunk `0x6ebe5bfb` 可达，但其
//!   IPersistStream 虚表在本轮静态检索中无指针/RTTI COL 可达路径）体未闭
//!   合——样本该槽即 TextSymbol 新对象（键 61），是当前样本停止点；
//! - LELP ⑰ BasicOverposterLayerProperties（`{EE535289-…}`，LabelPlacement
//!   .dll，CLSID 数据常量 `0x7857a814`）体未闭合（无 COL、无代码立即数，
//!   实现函数未定位）；⑱ 标注表达式引擎族、⑲/㉔ Envelope 与
//!   IGraphicsContainer 实现类体未闭合（样本对应槽位在停止点之后未达）；
//! - ALPC/ETM/LELP 各标量与字符串的业务语义为候选（表达式、标注类名、
//!   可见性等），未经目标软件行为验证；
//! - ETM ④ 的 1 字节（样本 0x00，布尔候选）语义未定；
//! - 全部结论为静态反编译观察与样本辅助对位，未运行 ArcMap 10.1，未人工
//!   目视验收。

use crate::layers::edit_template::{EDIT_TEMPLATE_CLASS_ID, read_edit_template_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// AnnotateLayerPropertiesCollection 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{1D5849F3-0D33-11D2-A26F-080009B6F22B}`。证据：
/// esri.clsid 声明（progID AnnotateLayerPropertiesCollection）与 esriCarto
/// 类型库 coclass #372；LabelPlacement.dll GetClassID `0x78531ba0` 以
/// `0x11d20d331d5849f3`/`0x2bf2b60900086fa2` 两个 8 字节立即数写出；样本
/// `/Maps:0xafe` 头（键 59）逐字节相同。
pub const ANNOTATE_PROPS_COLLECTION_CLASS_ID: [u8; 16] = [
    0xf3, 0x49, 0x58, 0x1d, 0x33, 0x0d, 0xd2, 0x11, 0xa2, 0x6f, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];

/// LabelEngineLayerProperties 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{01004145-0D1C-11D2-A26F-080009B6F22B}`。证据：
/// esri.clsid 声明（progID LabelEngineLayerProperties）与 esriCarto 类型库
/// coclass #378；样本 `/Maps:0xb1c` 头（键 60）逐字节相同（LabelPlacement
/// .dll 内实现，GetClassID 为常量拷贝形，代码无立即数）。
pub const LABEL_ENGINE_PROPS_CLASS_ID: [u8; 16] = [
    0x45, 0x41, 0x00, 0x01, 0x1c, 0x0d, 0xd2, 0x11, 0xa2, 0x6f, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];

/// EditTemplateManager 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{A0162E85-E170-4B10-A370-CEE08D8D4B8C}`。证据：
/// esri.clsid 声明（progID EditTemplateManager）与 esriCarto 类型库 coclass
/// #529；FeatureLayer.dll GetClassID `0x6f6d0a50` 以
/// `0x4b10e170a0162e85`/`0x8c4b8d8de0ce70a3` 两个 8 字节立即数写出；样本
/// `/Maps:0xfd7` 扩展段 CLSID 逐字节相同。
pub const EDIT_TEMPLATE_MANAGER_CLASS_ID: [u8; 16] = [
    0x85, 0x2e, 0x16, 0xa0, 0x70, 0xe1, 0x10, 0x4b, 0xa3, 0x70, 0xce, 0xe0, 0x8d, 0x8d, 0x4b, 0x8c,
];

/// ALPC::Load 接受的最大版本：`2 < (short)ver` 有符号拒绝；样本 2。
/// 证据：LabelPlacement.dll `0x78531be0`。
const MAX_ALPC_VERSION: i16 = 2;

/// LELP::Load 接受的最大版本：`7 < (short)ver` 拒绝；样本 7。
/// 证据：LabelPlacement.dll `0x785468c0`。
const MAX_LELP_VERSION: i16 = 7;

/// EditTemplateManager::Load 接受的最大版本：`2 < (short)ver` 拒绝；样本 2。
/// 证据：FeatureLayer.dll `0x6f6d0a80`。
const MAX_ETM_VERSION: i16 = 2;

/// 把游标回退到入口窗口内的绝对偏移（与 feature_layer_tail 的同名助手语义
/// 一致；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// 读取 int16 并按原生有符号比较校验版本上限；负值位模式一并拒绝
/// （负值版本无业务意义，较原生更严）。
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

/// LELP 体读取的停止原因（全部为「成功停止」：游标停在停止点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LelpStop10_1 {
    /// ⑯ ITextSymbol 槽的新对象类标识未闭合（TextSymbol 族等）：游标回退
    /// 到该槽位起点，其后的槽位与标量不可达。样本停止点（键 61）。
    SymbolSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑰ IBasicOverposterLayerProperties 槽的新对象类标识未闭合。
    OverposterSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑱ IAnnotationExpressionEngine 槽的新对象类标识未闭合。
    ExpressionSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑲ IEnvelope 槽的新对象类标识未闭合。
    EnvelopeSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ㉔ IGraphicsContainer 槽（版本 >2）的新对象类标识未闭合。
    ContainerSlotUnsupported { class_id: [u8; 16], key: u32 },
}

/// ALPC 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotatePropsStop10_1 {
    /// ③ 循环第 `index` 条（0 起）的新对象类标识不是
    /// LabelEngineLayerProperties：游标回退到该条目对象头起点，条目键与
    /// 尾 int32 不可达。
    EntryUnsupported {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
    /// 某 LELP 体内的停止（游标停在其停止点）。
    LelpBody(LelpStop10_1),
}

/// EditTemplateManager 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTemplateManagerStop10_1 {
    /// ③ 循环第 `index` 条（0 起）的新对象类标识不是 EditTemplate：游标
    /// 回退到该条目对象头起点，其后的条目与尾字节不可达。
    EntryUnsupported {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
}

/// ALPC ③ 循环的单个条目：对象分类 + （版本 ≥2 的）条目键。
#[derive(Debug, Clone, PartialEq)]
pub struct AlpcEntry10_1 {
    /// 对象分类（空引用 / 回引 / 新 LELP 对象）。前置停止时为 `Null`。
    pub slot: AlpcEntrySlot10_1,
    /// 版本 ≥2 的 int32 条目键（样本 0）。`None` = 版本 ≤1 或前置停止。
    pub entry_key: Option<i32>,
}

/// ALPC 条目的对象分类。
#[derive(Debug, Clone, PartialEq)]
pub enum AlpcEntrySlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LabelEngineLayerProperties 对象：已解析体（体内停止经
    /// [`AnnotatePropsStop10_1::LelpBody`] 传播）。
    LabelEngineProps(Box<LelpBody10_1>),
}

/// AnnotateLayerPropertiesCollection（LabelPlacement.dll `0x78531be0`）体的
/// 已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotatePropsBody10_1 {
    /// 对象头键（样本 59）。解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本（样本 2）；有符号比较 >2 拒绝。
    pub version: u16,
    /// int32 条目计数（样本 1）。负数原生不进循环，原样保留。
    pub count: i32,
    /// 已消费的循环条目（按流内顺序）。
    pub entries: Vec<AlpcEntry10_1>,
    /// 版本 ≥2 的尾 int32（→ -0x10 下一个条目键计数候选；样本 0）。
    /// `None` = 版本 ≤1 或前置停止。
    pub trailing_int32: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<AnnotatePropsStop10_1>,
}

/// LabelEngineLayerProperties（LabelPlacement.dll `0x785468c0`）体的已确认
/// 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LelpBody10_1 {
    /// 对象头键（样本 60）。
    pub key: u32,
    /// 体首 short 版本（样本 7）；有符号比较 >7 拒绝。
    pub version: u16,
    /// ② String（样本 `[JZDH]`；标注表达式候选）。
    pub expression: String,
    /// ③ short → -0x74（样本 -1），语义未定。
    pub short_74: i16,
    /// ④ 版本 ≤1 的 short。`None` = 版本 ≥2（原生不读流；样本即此形态）。
    pub short_ver1: Option<i16>,
    /// ⑤ double → -0x54（样本 0.0），语义未定（比例上限候选）。
    pub double_54: f64,
    /// ⑥ int32 → -0x60（样本 0），语义未定。
    pub int32_60: i32,
    /// ⑦ short → -0x5c（样本 -1），语义未定。
    pub short_5c: i16,
    /// ⑧ short → -0x4c（样本 0），语义未定。
    pub short_4c: i16,
    /// ⑨ short → -0x4a（样本 -1），语义未定。
    pub short_4a: i16,
    /// ⑩ int32 → -0x80（样本 0），语义未定。
    pub int32_80: i32,
    /// ⑪ double → -0x34（样本 0.0），语义未定。
    pub double_34: f64,
    /// ⑫ double → -0x3c（样本 0.0），语义未定。
    pub double_3c: f64,
    /// ⑬ short → -0x48（样本 -1），语义未定。
    pub short_48: i16,
    /// ⑭ String（样本 `默认`；标注类名候选）。
    pub name: String,
    /// ⑮ String（样本空串）。
    pub string_b: String,
    /// ⑯-⑲ 四个 LoadObject 槽的分类；新对象仅空标记/回引可消费，未闭合类
    /// 为停止点（对应槽字段记 `Null`，原因见 `stop`）。
    pub slots: [ObjectEntry10_1; 4],
    /// ⑳ double → -0x24（样本未达），语义未定。前置停止时 `None`。
    pub double_24: Option<f64>,
    /// ㉑ double → -0x1c。前置停止时 `None`。
    pub double_1c: Option<f64>,
    /// ㉒ int32 → -0x14。前置停止时 `None`。
    pub int32_14: Option<i32>,
    /// ㉓ int32 → -0x2c。前置停止时 `None`。
    pub int32_2c: Option<i32>,
    /// ㉔ 版本 >2 的 IGraphicsContainer 槽分类。`None` = 版本 ≤2 或前置停止。
    pub container_slot: Option<ObjectEntry10_1>,
    /// ㉕ 版本 ≥4 的 short → -0x46。`None` = 版本 ≤3 或前置停止。
    pub short_46: Option<i16>,
    /// ㉖ 版本 ≥5 的 int32 → -0x7c。`None` = 版本 ≤4 或前置停止。
    pub int32_7c: Option<i32>,
    /// ㉗ 版本 ==6 的 short。`None` = 版本 ≠6 或前置停止。
    pub short_ver6: Option<i16>,
    /// 停止原因；`None` = 体已完整消费（10.1 写侧版本 ≤7 全部字段）。
    pub stop: Option<LelpStop10_1>,
}

/// 从对象头之后的位置读取 10.1 LabelEngineLayerProperties 对象体，失败时
/// 不推进游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LABEL_ENGINE_PROPS_CLASS_ID`] 并
/// 取得键 `key`。四个固定槽与版本 >2 的容器槽的新对象仅空标记/回引可消费，
/// 其余类为成功停止（游标回退到该槽位起点、该键不保留）。
pub fn read_label_engine_props_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LelpBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // ① 版本 short（有符号 >7 拒绝）。
    let version = read_version_short(&mut trial, MAX_LELP_VERSION, "LELP", entry_offset)?;
    // ②③ String + short。
    let expression = read_string_10_1(&mut trial, max_bytes)?;
    let short_74 = trial.read_u16_le()? as i16;
    // ④ 版本 ≤1：short。
    let short_ver1 = if (version as i16) <= 1 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    // ⑤-⑬ 标量段。
    let double_54 = trial.read_f64_le()?;
    let int32_60 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let short_5c = trial.read_u16_le()? as i16;
    let short_4c = trial.read_u16_le()? as i16;
    let short_4a = trial.read_u16_le()? as i16;
    let int32_80 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let double_34 = trial.read_f64_le()?;
    let double_3c = trial.read_f64_le()?;
    let short_48 = trial.read_u16_le()? as i16;
    // ⑭⑮ String ×2。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;

    let mut staged_table = table.clone();
    let mut stop = None;

    // ⑯-⑲ 四个 LoadObject 槽（顺序：TextSymbol/Overposter/ExpressionEngine/
    //    Envelope；新对象非空标记类 = 停止点，回退到槽位起点）。
    let mut slots = [
        ObjectEntry10_1::Null,
        ObjectEntry10_1::Null,
        ObjectEntry10_1::Null,
        ObjectEntry10_1::Null,
    ];
    let slot_stops: [fn([u8; 16], u32) -> LelpStop10_1; 4] = [
        |c, k| LelpStop10_1::SymbolSlotUnsupported {
            class_id: c,
            key: k,
        },
        |c, k| LelpStop10_1::OverposterSlotUnsupported {
            class_id: c,
            key: k,
        },
        |c, k| LelpStop10_1::ExpressionSlotUnsupported {
            class_id: c,
            key: k,
        },
        |c, k| LelpStop10_1::EnvelopeSlotUnsupported {
            class_id: c,
            key: k,
        },
    ];
    for i in 0..4 {
        if stop.is_some() {
            break;
        }
        let slot_offset = trial.position();
        let pre_slot_table = staged_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        match entry {
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(slot_stops[i](class_id, key));
                trial = reposition_to(reader, slot_offset);
                staged_table = pre_slot_table;
            }
            other => slots[i] = other,
        }
    }

    // ⑳-㉓ 标量段（前置停止时不读取）。
    let mut double_24 = None;
    let mut double_1c = None;
    let mut int32_14 = None;
    let mut int32_2c = None;
    let mut container_slot = None;
    let mut short_46 = None;
    let mut int32_7c = None;
    let mut short_ver6 = None;
    if stop.is_none() {
        double_24 = Some(trial.read_f64_le()?);
        double_1c = Some(trial.read_f64_le()?);
        int32_14 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        int32_2c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        // ㉔ 版本 >2：IGraphicsContainer 槽。
        if (version as i16) > 2 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(LelpStop10_1::ContainerSlotUnsupported { class_id, key });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                }
                other => container_slot = Some(other),
            }
        }
        // ㉕㉖㉗ 版本门标量。
        if stop.is_none() && (version as i16) >= 4 {
            short_46 = Some(trial.read_u16_le()? as i16);
        }
        if stop.is_none() && (version as i16) >= 5 {
            int32_7c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        if stop.is_none() && (version as i16) == 6 {
            short_ver6 = Some(trial.read_u16_le()? as i16);
        }
    }

    let result = LelpBody10_1 {
        key,
        version,
        expression,
        short_74,
        short_ver1,
        double_54,
        int32_60,
        short_5c,
        short_4c,
        short_4a,
        int32_80,
        double_34,
        double_3c,
        short_48,
        name,
        string_b,
        slots,
        double_24,
        double_1c,
        int32_14,
        int32_2c,
        container_slot,
        short_46,
        int32_7c,
        short_ver6,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 AnnotateLayerPropertiesCollection 对象体，
/// 失败时不推进游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`ANNOTATE_PROPS_COLLECTION_CLASS_ID`] 并取得键 `key`。循环条目的新对象
/// 仅支持 LabelEngineLayerProperties（其余为成功停止，回退到该条目对象头
/// 起点、不保留该键）；LELP 体内停止传播为
/// [`AnnotatePropsStop10_1::LelpBody`]。截断与版本超限为错误。
pub fn read_annotate_props_collection_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<AnnotatePropsBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // ① 版本 short（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_ALPC_VERSION,
        "AnnotateLayerPropertiesCollection",
        entry_offset,
    )?;
    // ② int32 计数（负数原生不进循环，原样保留）。
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    let mut staged_table = table.clone();
    let mut entries = Vec::new();
    let mut stop = None;
    for index in 0..count.max(0) as usize {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        let mut entry = AlpcEntry10_1 {
            slot: AlpcEntrySlot10_1::Null,
            entry_key: None,
        };
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                entry.slot = AlpcEntrySlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == LABEL_ENGINE_PROPS_CLASS_ID =>
            {
                let body = read_label_engine_props_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged_table,
                    key,
                )?;
                if let Some(inner) = body.stop {
                    stop = Some(AnnotatePropsStop10_1::LelpBody(inner));
                }
                entry.slot = AlpcEntrySlot10_1::LabelEngineProps(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(AnnotatePropsStop10_1::EntryUnsupported {
                    class_id,
                    key,
                    index,
                });
                trial = reposition_to(reader, item_offset);
                staged_table = pre_item_table;
            }
        }
        // 版本 ≥2 的条目键 int32（前置停止时不读取）。
        if stop.is_none() && (version as i16) >= 2 {
            entry.entry_key = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        entries.push(entry);
        if stop.is_some() {
            break;
        }
    }
    // ④ 版本 ≥2：尾 int32。
    let trailing_int32 = if stop.is_none() && (version as i16) >= 2 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };

    let result = AnnotatePropsBody10_1 {
        key,
        version,
        count,
        entries,
        trailing_int32,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// EditTemplateManager ③ 循环的单个条目。
#[derive(Debug, Clone, PartialEq)]
pub enum EtmEntry10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 EditTemplate 对象：已解析体（[`crate::layers::edit_template`] 的
    /// 读取器无停止点：要么完整消费，要么整体失败）。
    EditTemplate(Box<crate::layers::edit_template::EditTemplateBody10_1>),
}

/// EditTemplateManager（FeatureLayer.dll `0x6f6d0a80`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct EditTemplateManagerBody10_1 {
    /// 对象头键。解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本（样本 2）；有符号比较 >2 拒绝。
    pub version: u16,
    /// int32 计数（样本 1）。负数原生不进循环，原样保留。
    pub count: i32,
    /// 已消费的循环条目（按流内顺序）。
    pub entries: Vec<EtmEntry10_1>,
    /// 版本 ≥2 的 1 字节（样本 0x00；布尔候选，语义未定），原样保留。
    /// `None` = 版本 ≤1 或前置停止。
    pub tail_byte: Option<u8>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<EditTemplateManagerStop10_1>,
}

/// 从对象头之后的位置读取 10.1 EditTemplateManager 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`EDIT_TEMPLATE_MANAGER_CLASS_ID`]
/// 并取得键 `key`。循环条目的新对象仅支持 EditTemplate（其余为成功停止，
/// 回退到该条目对象头起点、不保留该键）。截断与版本超限为错误。
pub fn read_edit_template_manager_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<EditTemplateManagerBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // ① 版本 short（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_ETM_VERSION,
        "EditTemplateManager",
        entry_offset,
    )?;
    // ② int32 计数。
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    let mut staged_table = table.clone();
    let mut entries = Vec::new();
    let mut stop = None;
    for index in 0..count.max(0) as usize {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        let mut entry = EtmEntry10_1::Null;
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                entry = EtmEntry10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == EDIT_TEMPLATE_CLASS_ID => {
                let body =
                    read_edit_template_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                entry = EtmEntry10_1::EditTemplate(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(EditTemplateManagerStop10_1::EntryUnsupported {
                    class_id,
                    key,
                    index,
                });
                trial = reposition_to(reader, item_offset);
                staged_table = pre_item_table;
            }
        }
        entries.push(entry);
        if stop.is_some() {
            break;
        }
    }
    // ④ 版本 ≥2：1 字节。
    let tail_byte = if stop.is_none() && (version as i16) >= 2 {
        Some(trial.read_bytes(1)?[0])
    } else {
        None
    };

    let result = EditTemplateManagerBody10_1 {
        key,
        version,
        count,
        entries,
        tail_byte,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}
