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
//!    类名候选）、串 3 空，槽数据依次为 TextSymbol 新对象（键 61，接线轮起
//!    经 [`read_text_symbol_body_10_1`] 整体解析，体内停止经
//!    [`LelpStop10_1::TextSymbolBody`] 透传）、其后槽位在样本停止点之后未
//!    达。⑤-⑬/⑳-㉓ 各标量业务语义未定，原样保留。
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
//! - LELP ⑯ ITextSymbol 槽：TextSymbol（`{B65A3E74-…}`，DisplayCore.dll）
//!   体已由元素体轮闭合（`objects::element_body::read_text_symbol_body_
//!   10_1`），接线轮接入本槽整体解析；体内 ITextPath/ISymbol/IFontDisp 槽
//!   未闭合实现类仍为透传停止点（样本停止点在接线后位于 TextSymbol 体内）；
//! - LELP ⑰ BasicOverposterLayerProperties（`{EE535289-…}`）体第十轮闭合
//!   （含 LineLabelPosition/LineLabelPlacementPriorities/PointPlacementPriorities
//!   三个嵌套槽实现类；BOLP 自身 Load 未唯一定位，字段序为样本结构+局部
//!   反编译佐证，见 `BolpBody10_1` 文档）；⑱ AnnotationVBScriptEngine 体
//!   第十轮闭合（`0x785361d0`）；⑲ Envelope 体为对象层已闭合读取器；
//!   ㉔ IGraphicsContainer 实现类体未闭合（样本空标记）；
//! - ALPC/ETM/LELP 各标量与字符串的业务语义为候选（表达式、标注类名、
//!   可见性等），未经目标软件行为验证；
//! - ETM ④ 的 1 字节（样本 0x01）语义未定；
//! - 全部结论为静态反编译观察与样本辅助对位，未运行 ArcMap 10.1，未人工
//!   目视验收。

use crate::layers::edit_template::{EDIT_TEMPLATE_CLASS_ID, read_edit_template_body_10_1};
use crate::objects::element_body::{
    ElementBodyStop10_1, TEXT_SYMBOL_CLASS_ID, TextSymbolBody10_1, read_text_symbol_body_10_1,
};
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
///
/// 接线轮起 TextSymbol 体会经类分派整体解析，其体内停止透传为本枚举新变
/// 体（携带体类型故不再 `Copy`/`Eq`，仅 `Clone`/`PartialEq`）。
#[derive(Debug, Clone, PartialEq)]
pub enum LelpStop10_1 {
    /// ⑯ ITextSymbol 槽的 TextSymbol（`{B65A3E74-…}`）体内部出现停止（原因
    /// 透传，见 [`ElementBodyStop10_1`]）：游标停在内层停止点，槽键已按原
    /// 生「登记先于持久化」语义保留；⑰ 及其后的槽位与标量不可达。
    TextSymbolBody {
        key: u32,
        reason: ElementBodyStop10_1,
    },
    /// ⑯ ITextSymbol 槽的新对象类标识未闭合（TextSymbol 之外的实现类）：
    /// 游标回退到该槽位起点，其后的槽位与标量不可达。
    SymbolSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑰ IBasicOverposterLayerProperties 槽的新对象类标识未闭合。
    OverposterSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑱ IAnnotationExpressionEngine 槽的新对象类标识未闭合。
    ExpressionSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑲ IEnvelope 槽的新对象类标识未闭合。
    EnvelopeSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ㉔ IGraphicsContainer 槽（版本 >2）的新对象类标识未闭合。
    ContainerSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// ⑰ BOLP 体内部停止（原因透传，游标停在其停止点；槽键按「登记先于
    /// 持久化」保留）。
    OverposterBody(BolpStop10_1),
    /// ⑲ Envelope 体内部错误不属于本枚举（嵌套空间参考非投影坐标系为
    /// 硬错误，向上传播）；本变体保留供未来扩展。
    #[allow(dead_code)]
    EnvelopeBody,
}

/// LELP ⑯ ITextSymbol 槽（请求 IID `{A80B5E91-…}`）条目的分类结果（接线轮
/// 起 TextSymbol 新对象整体解析；证据：`docs/arcmap-10.1-display-symbols-
/// round2.md` 第一节虚表归属复核、元素体轮 TextSymbol 字段序闭合）。
#[derive(Debug, Clone, PartialEq)]
pub enum LelpTextSymbolSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TextSymbol 对象（类标识经 [`TEXT_SYMBOL_CLASS_ID`] 校验）：已解析
    /// 体（体内停止经 [`LelpStop10_1::TextSymbolBody`] 透传；键已登记）。
    TextSymbol(Box<TextSymbolBody10_1>),
}

/// AnnotateLayerPropertiesCollection 体读取的停止原因。
#[derive(Debug, Clone, PartialEq)]
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
    /// ⑯ ITextSymbol 槽的分类（接线轮起 TextSymbol 新对象整体解析，见
    /// [`LelpTextSymbolSlot10_1`]；体内停止经 `stop` 透传）。
    pub text_symbol: LelpTextSymbolSlot10_1,
    /// ⑰ 槽分类（第十轮起 BOLP 新对象整体解析，见 [`BolpSlot10_1`]；体
    /// 内停止经 `stop` 透传）。
    pub overposter: BolpSlot10_1,
    /// ⑱ 槽分类（第十轮起 AnnotationVBScriptEngine 新对象整体解析，见
    /// [`ExpressionEngineSlot10_1`]）。
    pub engine: ExpressionEngineSlot10_1,
    /// ⑲ 槽分类（第十轮起 Envelope 新对象经对象层读取器整体解析，体内含
    /// 嵌套空间参考，见 [`EnvelopeSlot10_1`]）。
    pub envelope: EnvelopeSlot10_1,
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

    // ⑯ ITextSymbol 槽（接线轮接入 TextSymbol 体分派；证据：
    //    docs/arcmap-10.1-display-symbols-round2.md 第一节 TextSymbol 虚表归
    //    属复核 + docs/arcmap-10.1-element-bodies.md TextSymbol 体字段序闭合，
    //    读取器 read_text_symbol_body_10_1）：新对象且类标识为 TextSymbol
    //    （`{B65A3E74-…}`）时整体解析其体——键已由条目分类按原生「登记先于
    //    持久化」语义登记，体内停止透传为 [`LelpStop10_1::TextSymbolBody`]
    //    （游标停在内层停止点，⑰ 起不可达）；其余 ITextSymbol 实现类维持停
    //    止回退（游标回退到槽位起点、键不保留）。样本槽即 TextSymbol 新对
    //    象（键 61），接线后样本停止点推进至 TextSymbol 体内。
    let mut text_symbol = LelpTextSymbolSlot10_1::Null;
    // ⑰-⑲ 三个 LoadObject 槽(第十轮起按类分派整体解析;新对象类标识未
    //    闭合时仍为停止点,回退到槽位起点):
    //    ⑰ BOLP({EE535289-…})→ read_bolp_body_10_1,体内停止透传;
    //    ⑱ AnnotationVBScriptEngine({AA157207-…})→ 4/6 字节体;
    //    ⑲ Envelope({30707212-…})→ 对象层 read_envelope_10_1(体内嵌套
    //      空间参考,非投影坐标系为硬错误向上传播,无停止点)。
    let mut overposter = BolpSlot10_1::Null;
    let mut engine = ExpressionEngineSlot10_1::Null;
    let mut envelope = EnvelopeSlot10_1::Null;
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre_slot_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                text_symbol = LelpTextSymbolSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == TEXT_SYMBOL_CLASS_ID => {
                let body = read_text_symbol_body_10_1(&mut trial, max_bytes, &mut staged_table)?;
                if let Some(reason) = body.stop.clone() {
                    stop = Some(LelpStop10_1::TextSymbolBody { key, reason });
                }
                text_symbol = LelpTextSymbolSlot10_1::TextSymbol(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(LelpStop10_1::SymbolSlotUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = pre_slot_table;
            }
        }
    }
    // ⑰ BOLP 槽。
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre_slot_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                overposter = BolpSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == BASIC_OVERPOSTER_LAYER_PROPS_CLASS_ID =>
            {
                let body = read_bolp_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(inner) = body.stop.clone() {
                    stop = Some(LelpStop10_1::OverposterBody(inner));
                }
                overposter = BolpSlot10_1::Bolp(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(LelpStop10_1::OverposterSlotUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = pre_slot_table;
            }
        }
    }
    // ⑱ 表达式引擎槽。
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre_slot_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                engine = ExpressionEngineSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID =>
            {
                let body = read_annotation_vbscript_engine_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged_table,
                    key,
                )?;
                engine = ExpressionEngineSlot10_1::VbscriptEngine(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(LelpStop10_1::ExpressionSlotUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = pre_slot_table;
            }
        }
    }
    // ⑲ Envelope 槽(对象层读取器:体内含嵌套空间参考,仅投影坐标系;
    //   其余嵌套类为硬错误向上传播——较原生「无法跳过未知体」语义一致)。
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre_slot_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                envelope = EnvelopeSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key: _ }
                if class_id == crate::objects::envelope::ENVELOPE_CLASS_ID =>
            {
                let body = crate::objects::envelope::read_envelope_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged_table,
                )?;
                envelope = EnvelopeSlot10_1::Envelope(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(LelpStop10_1::EnvelopeSlotUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = pre_slot_table;
            }
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
        text_symbol,
        overposter,
        engine,
        envelope,
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
                if let Some(inner) = body.stop.clone() {
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

// ---------------------------------------------------------------------------
// 第十轮:BOLP 族(BasicOverposterLayerProperties 及其三个嵌套槽实现类)、
// 标注表达式引擎族与表达式属性族。
//
// 证据来源:`docs/arcmap-10.1-featurelayer-tail.md` 第十轮(2026-11-15)。
// 全部为 LabelPlacement.dll 10.1.0.3035(工程副本 /tmp/f-lelp-gh/ghproj_lp,
// 本机 pyghidra)静态反编译 + 样本 `temp/r6-maps.bin` `0xda3..0x155a` 逐字节
// 对位;定位链与证据强度逐类标注于各读取器文档注释。
// ---------------------------------------------------------------------------

/// BasicOverposterLayerProperties 类标识(CLSID)的 16 字节内存序。
///
/// GUID 字符串形式 `{EE535289-41C9-11D1-880A-080009EC732A}`。证据:esri.clsid
/// 声明(progID BasicOverposterLayerProperties,模块 LabelPlacement.dll);
/// GetClassID `0x7853d360`(立即数写出,常量 `0x7857a814`);RTTI TD
/// `.?AVBasicOverposterLayerProperties@@`(0x7858dd20)及 10 个接口 vftable
/// COL 簇(.rdata 0x785810c8..0x78581190);样本 `/Maps:0xda3` 头(键 69)
/// 逐字节相同。
///
/// 别名注记(接线轮 2026-11-15 核对):StdFont 轮(ole-stdfont 文档)曾把同
/// 一流内对象(键 69 @0xda3,LELP ⑰ 槽)的停止诊断解码为
/// `{C9EE5352-D141-11D1-880A-080009EC732A}`。接线轮以主检出
/// `temp/agent-fl-tail/ecfg/esri.clsid.xml` 全表(11348 项)复核:`C9EE5352`
/// 零命中;BasicOverposter 族仅 BasicOverposter `{3141F2FC-38E2-11D1-8809-…}`、
/// BasicOverposterProperties `{E0C73D56-6C88-498C-B77C-…}` 与本类
/// `{EE535289-…}` 三条。同键同偏移的字节与本常量逐位吻合,且现行解析已按本
/// 常量从 0xda3 连续消费 BOLP 体与三槽至 0x151a——E 轮诊断的 GUID 文本为同
/// 一字节序列的错位解码,维持 F 轮命名 BasicOverposterLayerProperties,不再
/// 另立常量。
pub const BASIC_OVERPOSTER_LAYER_PROPS_CLASS_ID: [u8; 16] = [
    0x89, 0x52, 0x53, 0xee, 0xc9, 0x41, 0xd1, 0x11, 0x88, 0x0a, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// LineLabelPosition 类标识(CLSID,与 ILineLabelPosition IID 同值)的 16
/// 字节内存序。
///
/// GUID `{2442958C-D711-11D2-9F41-00C04F6BC6A5}`。证据:esri.clsid 声明;
/// GetClassID `0x7855afb0`(常量拷贝形,常量 `0x7857b02c`);样本
/// `/Maps:0xdcb` 头(键 70)逐字节相同。
pub const LINE_LABEL_POSITION_CLASS_ID: [u8; 16] = [
    0x8c, 0x95, 0x42, 0x24, 0x11, 0xd7, 0xd2, 0x11, 0x9f, 0x41, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0xa5,
];

/// LineLabelPlacementPriorities 类标识(CLSID)的 16 字节内存序。
///
/// GUID `{261A4377-D9D5-11D2-A806-CC9F870BCD5A}`。证据:esri.clsid 声明;
/// GetClassID `0x7855add0`;样本 `/Maps:0xe01` 头(键 71)逐字节相同。
pub const LINE_LABEL_PLACEMENT_PRIORITIES_CLASS_ID: [u8; 16] = [
    0x77, 0x43, 0x1a, 0x26, 0xd5, 0xd9, 0xd2, 0x11, 0xa8, 0x06, 0xcc, 0x9f, 0x87, 0x0b, 0xcd, 0x5a,
];

/// PointPlacementPriorities 类标识(CLSID)的 16 字节内存序。
///
/// GUID `{261A4372-D9D5-11D2-A806-CC9F870BCD5A}`。证据:esri.clsid 声明;
/// GetClassID `0x7856d690`;样本 `/Maps:0xe63` 头(键 72)逐字节相同。
pub const POINT_PLACEMENT_PRIORITIES_CLASS_ID: [u8; 16] = [
    0x72, 0x43, 0x1a, 0x26, 0xd5, 0xd9, 0xd2, 0x11, 0xa8, 0x06, 0xcc, 0x9f, 0x87, 0x0b, 0xcd, 0x5a,
];

/// AnnotationVBScriptEngine 类标识(CLSID)的 16 字节内存序。
///
/// GUID `{AA157207-E079-11D2-9F48-00C04F6BC6A5}`。证据:esri.clsid 声明;
/// GetClassID `0x78535ae0`(常量拷贝形,常量 `0x78579f60`);名字 getter
/// `0x78535520` 返回 `L"AnnotationVBScriptEngine"`;样本 `/Maps:0xec5` 头
/// (键 73)逐字节相同。
pub const ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID: [u8; 16] = [
    0x07, 0x72, 0x15, 0xaa, 0x79, 0xe0, 0xd2, 0x11, 0x9f, 0x48, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0xa5,
];

/// HotLinkExpressionProperties 类标识(CLSID)的 16 字节内存序。
///
/// GUID `{1B848B0F-5E87-4948-841A-86201FACD925}`。证据:esri.clsid 声明;
/// GetClassID `0x78540500`(常量 `0x7857b7e0`);名字 getter `0x785404f0`
/// 返回 `L"HotLinkExpressionProperties"`;样本 `/Maps:0x15d8` 头(键 85)
/// 逐字节相同。
pub const HOT_LINK_EXPRESSION_PROPS_CLASS_ID: [u8; 16] = [
    0x0f, 0x8b, 0x84, 0x1b, 0x87, 0x5e, 0x48, 0x49, 0x84, 0x1a, 0x86, 0x20, 0x1f, 0xac, 0xd9, 0x25,
];

/// DisplayExpressionProperties 类标识(CLSID)的 16 字节内存序。
///
/// GUID 字符串形式 `{D75C6301-C05A-484C-A7BD-AE82E5E1FC75}`。证据:esri.clsid
/// 声明(模块 FeatureLayer.dll);样本 `/Maps:0x151a` 头(键 83)逐字节相同
/// (FeatureLayer.dll 侧 GetClassID/Load 未定位,证据为样本结构)。
///
/// 接线轮更正(2026-11-15):第十轮落位时本常量第 7/8 字节(第三 GUID 字段的
/// LE 对)误写作 `0x48, 0x4c`(即 `{…-C05A-4C48-…}`),与 esri.clsid 声明及样
/// 本头不一致,导致后继段 ㊴ 槽的类比对永不命中、样本停在 DEP 对象头。现按
/// esri.clsid 声明(GUID 第三字段 0x484C → LE 字节 `0x4c, 0x48`)与样本
/// `0x151a` 头逐字节(`01 63 5c d7 5a c0 4c 48 a7 bd ae 82 e5 e1 fc 75`)
/// 更正;此前经本常量构造的合成输入不受影响(输入与常量同源)。
pub const DISPLAY_EXPRESSION_PROPS_CLASS_ID: [u8; 16] = [
    0x01, 0x63, 0x5c, 0xd7, 0x5a, 0xc0, 0x4c, 0x48, 0xa7, 0xbd, 0xae, 0x82, 0xe5, 0xe1, 0xfc, 0x75,
];

/// LineLabelPosition(`0x7855b4e0`)体的已确认字段。
///
/// 证据:确定的。定位链:GetClassID `0x7855afb0`(常量拷贝)→ 名字 getter
/// `0x7855afa0`(`L"LineLabelPosition"`)→ IPersistStream 适配虚表
/// `0x7857d0f0` 槽 +0x14 = 转发调整器 `0x7855b7b0` → Load `0x7855b4e0`;
/// Save `0x7855b650`(槽 +0x18 经 `0x7855b810`)逐字段对称互证。字段序:
/// ① int16 版本(`2 < ver` 有符号拒绝 `0x7fffbffb`;样本 2)→ ② int16 ×11
/// → ③ double → ④ 版本 ≥2:int16。样本 11 个 short =
/// [-1,0,0,0,0,-1,0,0,-1,0,0]、double 0.0、尾 short 0(布尔开关候选)。
#[derive(Debug, Clone, PartialEq)]
pub struct LineLabelPositionBody10_1 {
    /// 对象头键(样本 70)。
    pub key: u32,
    /// 体首 short 版本(样本 2);有符号 >2 拒绝(较原生更严:负值一并拒绝)。
    pub version: u16,
    /// 11 个 int16(位置开关候选;样本 -1/0 混合,原样保留)。
    pub shorts: [i16; 11],
    /// double(样本 0.0),语义未定。
    pub double: f64,
    /// 版本 ≥2 的尾 int16(样本 0)。`None` = 版本 ≤1。
    pub tail_short: Option<i16>,
}

/// LineLabelPlacementPriorities(`0x7855ae10`)体的已确认字段。
///
/// 证据:确定的。定位链:GetClassID `0x7855add0` → Load `0x7855ae10`
/// (GetClassID +0x40 邻位,含 15 次 int32 读取循环)。字段序:① int16
/// 版本(`1 < ver` 拒绝 `0x7fffbffb`;样本 1)→ ② int32 ×15(3×5 优先级
/// 网格候选;样本 [0,3,3,3, 0,0,3,3,3, 0,0,3,3,3,0],原样保留)。
#[derive(Debug, Clone, PartialEq)]
pub struct LineLabelPlacementPrioritiesBody10_1 {
    /// 对象头键(样本 71)。
    pub key: u32,
    /// 体首 short 版本(样本 1);有符号 >1 拒绝。
    pub version: u16,
    /// 15 个 int32,原样保留。
    pub ints: [i32; 15],
}

/// PointPlacementPriorities(`0x7856d920`)体的已确认字段。
///
/// 证据:确定的。定位链:GetClassID `0x7856d690` → Load `0x7856d920`
/// (+0x290 邻位,含 8 次 int32 读取循环)。字段序:① int16 版本(`1 < ver`
/// 拒绝;样本 1)→ ② int32 ×8(方向优先级候选;样本
/// [2,2,1,2,2,3,3,3],原样保留)。
#[derive(Debug, Clone, PartialEq)]
pub struct PointPlacementPrioritiesBody10_1 {
    /// 对象头键(样本 72)。
    pub key: u32,
    /// 体首 short 版本(样本 1);有符号 >1 拒绝。
    pub version: u16,
    /// 8 个 int32,原样保留。
    pub ints: [i32; 8],
}

/// AnnotationVBScriptEngine(`0x785361d0`)体的已确认字段。
///
/// 证据:确定的。定位链:GetClassID `0x78535ae0` → IPersistStream 适配虚表
/// `0x7857a1fc` 槽 +0x18 = Load `0x785361d0`、+0x1c = Save `0x78536240`
/// (对称,写静态常量 2)。字段序:① int16 版本(`2 < ver` → `0x80004005`;
/// 样本 2)→ ② 版本 ≥2:int16 → this+0xc(样本 -1,原样保留)。
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationVbscriptEngineBody10_1 {
    /// 对象头键(样本 73)。
    pub key: u32,
    /// 体首 short 版本(样本 2);有符号 >2 拒绝(较原生更严:负值一并拒绝)。
    pub version: u16,
    /// 版本 ≥2 的 int16(样本 -1),语义未定。`None` = 版本 ≤1。
    pub short: Option<i16>,
}

/// HotLinkExpressionProperties 体(实现函数未定位)的样本结构字段。
///
/// 证据:样本结构(样本 `/Maps:0x15ec..0x1606` 逐字节)+ 构造函数
/// `0x78540560` 成员形状(String、short 0xffff、int)旁证;Load 未定位
/// (QI `0x785406d0` 仅两个接口,无 IPersistStream 形 vtable,负证据见研究
/// 记录)。字段序:① int16 版本(样本 1)→ ② String(样本空串)→
/// ③ int16(样本 -1)→ ④ 16 字节原样块(样本全零;GUID 候选)。
#[derive(Debug, Clone, PartialEq)]
pub struct HotLinkExpressionPropsBody10_1 {
    /// 对象头键(样本 85)。
    pub key: u32,
    /// 体首 short 版本(样本 1);无反编译证据,不设上限。
    pub version: u16,
    /// String(样本空串;热链接表达式候选)。
    pub string: String,
    /// int16(样本 -1),语义未定。
    pub short: i16,
    /// 16 字节原样块(样本全零),语义未定。
    pub guid_block: [u8; 16],
}

/// BOLP 槽间 16 字节标量块(槽 2 与槽 3 之间;`0x7853d3a0` 槽间读序
/// [int16][int32][double][int16] 佐证;样本全零,语义未定)。
#[derive(Debug, Clone, PartialEq)]
pub struct BolpBetweenBlock10_1 {
    /// int16(样本 0)。
    pub short_a: i16,
    /// int32(样本 0)。
    pub int_a: i32,
    /// double(样本 0.0)。
    pub double: f64,
    /// int16(样本 0)。
    pub short_b: i16,
}

/// BOLP 体尾段(样本 `0xe99..0xec5` 共 44 字节;`0x7853d3a0` 尾读序
/// [int32 计数][计数 × double][double][int32][String][int32] 佐证前 24
/// 字节,计数 0 时无 double;其后 20 字节块读序未定名,原样保留)。
#[derive(Debug, Clone, PartialEq)]
pub struct BolpTail10_1 {
    /// int32 计数(样本 0)。负数/零不进循环,原样保留。
    pub count: i32,
    /// 计数 × double(样本计数 0 → 空)。
    pub doubles: Vec<f64>,
    /// double(样本 0.0)。
    pub double: f64,
    /// int32(样本 0)。
    pub int_a: i32,
    /// String(样本 cb=0 空串;BSTR 空语义,无终止零载荷)。
    pub string: String,
    /// int32(样本 0)。
    pub int_b: i32,
    /// 20 字节未定名块:int32 ×2(样本 1、2)+ double(样本 0.0)+
    /// int16 ×2(样本 0、-1),原样保留。
    pub block_int_a: i32,
    pub block_int_b: i32,
    pub block_double: f64,
    pub block_short_a: i16,
    pub block_short_b: i16,
}

/// BOLP 体读取的停止原因(全部为「成功停止」)。
#[derive(Debug, Clone, PartialEq)]
pub enum BolpStop10_1 {
    /// 槽 1 的新对象类标识非 LineLabelPosition:游标回退到该槽位起点。
    LabelPositionSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// 槽 2/槽 3 的新对象类标识非对应 Priorities 类:游标回退到该槽位
    /// 起点;`which` 0 = Line 槽、1 = Point 槽。
    PrioritiesSlotUnsupported {
        class_id: [u8; 16],
        key: u32,
        which: u8,
    },
}

/// BOLP 槽 1(ILineLabelPosition)条目分类。
#[derive(Debug, Clone, PartialEq)]
pub enum LineLabelPositionSlot10_1 {
    /// 16 字节全零类标识:空引用。
    Null,
    /// 键已见:回引条目,无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LineLabelPosition 对象:已解析体。
    Body(Box<LineLabelPositionBody10_1>),
}

/// BOLP 槽 2/槽 3(ILineLabelPlacementPriorities / IPointPlacementPriorities)
/// 条目分类。
#[derive(Debug, Clone, PartialEq)]
pub enum PlacementPrioritiesSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LineLabelPlacementPriorities 对象(槽 2)。
    Line(Box<LineLabelPlacementPrioritiesBody10_1>),
    /// 新 PointPlacementPriorities 对象(槽 3)。
    Point(Box<PointPlacementPrioritiesBody10_1>),
}

/// BasicOverposterLayerProperties 体的已确认字段。
///
/// 证据:样本结构(样本键 69 体 `0xdb7..0xec5` 共 270 字节逐字节对位)+ 部分反编
/// 译佐证(`0x7853d3a0` = LabelPlacement.dll 内唯一经 LoadObject 引用
/// ILineLabelPosition2 `{2442958B}`/ILineLabelPlacementPriorities2
/// `{261A4376}`/IPointPlacementPriorities2 `{261A4371}` 的函数,位于 BOLP
/// GetClassID +0x40 邻位;其标量前缀读序 [int16][int16×3][int32×3]、槽间
/// [int16][int32][double][int16] 与尾段读序与本读取器一致,但函数归属未
/// 证实,且其无条件尾段(计数循环/double/int32/String/int32)与样本版本
/// 行为不吻合——见研究记录「BOLP::Load 未定位」节)。字段序(样本逐字
/// 节对位):① 20 字节标量前缀([int16][int16×3][int32×3],语义候选)→
/// ② 槽 LineLabelPosition → ③ 槽 LineLabelPlacementPriorities → ④ 16 字
/// 节标量块 → ⑤ 槽 PointPlacementPriorities → ⑥ 44 字节尾段。
#[derive(Debug, Clone, PartialEq)]
pub struct BolpBody10_1 {
    /// 对象头键(样本 69)。
    pub key: u32,
    /// 前缀首 int16(样本 0;版本候选,原样保留,无门)。
    pub version: u16,
    /// 前缀 int16 ×3(样本 0、0、0),语义未定。
    pub shorts: [i16; 3],
    /// 前缀 int32 ×3(样本 0、3、1),语义未定。
    pub ints: [i32; 3],
    /// 槽 1 LineLabelPosition 分类(样本键 70 已解析体)。
    pub label_position: LineLabelPositionSlot10_1,
    /// 槽 2 LineLabelPlacementPriorities 分类(样本键 71)。
    pub priorities_line: PlacementPrioritiesSlot10_1,
    /// 槽间 16 字节标量块(样本全零)。
    pub between: BolpBetweenBlock10_1,
    /// 槽 3 PointPlacementPriorities 分类(样本键 72)。
    pub priorities_point: PlacementPrioritiesSlot10_1,
    /// 尾段 44 字节(样本值见 [`BolpTail10_1`])。
    pub tail: BolpTail10_1,
    /// 停止原因;`None` = 体已完整消费(样本形态)。
    pub stop: Option<BolpStop10_1>,
}

/// 读取 AfCore String::Read 布局的字符串,允许 cb=0 的空串。
///
/// 与 [`crate::stream::native_string::read_string_10_1`](写方向 String::
/// Write 布局,恒含终止零,cb≥2)不同:UID/BOLP 尾段/表达式属性等调用
/// 点经 OleAut BSTR 语义写入,空 BSTR 的流内 cb=0、无载荷(样本
/// `/Maps:0x14d0` 与 BOLP 尾段 cb=0 佐证)。cb 奇数为错误;cb≥2 时仍要求
/// 终止零。
pub(crate) fn read_stream_string_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<String> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("流字符串 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let length = usize::try_from(trial.read_u32_le()?).map_err(|_| invalid("长度非法"))?;
    if length == 0 {
        *reader = trial;
        return Ok(String::new());
    }
    if length < 2 || length % 2 != 0 {
        return Err(invalid("非零长度必须为含终止零的偶数字节数"));
    }
    if length > max_bytes {
        return Err(invalid("长度超过调用者资源限额"));
    }
    let bytes = trial.read_bytes(length)?;
    if bytes[length - 2..] != [0, 0] {
        return Err(invalid("缺少末尾终止零"));
    }
    let value = Reader::new(&bytes[..length - 2])
        .read_utf16_le(length - 2)
        .map_err(|_| invalid("非法 UTF-16LE 内容"))?;
    *reader = trial;
    Ok(value)
}

/// 从对象头之后的位置读取 10.1 LineLabelPosition 体(证据:确定的,见
/// 结构体文档)。失败时不推进游标、不修改登记表。
pub fn read_line_label_position_body_10_1(
    reader: &mut Reader<'_>,
    _max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LineLabelPositionBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, 2, "LineLabelPosition", entry_offset)?;
    let mut shorts = [0i16; 11];
    for slot in &mut shorts {
        *slot = trial.read_u16_le()? as i16;
    }
    let double = trial.read_f64_le()?;
    let tail_short = if version >= 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    *reader = trial;
    Ok(LineLabelPositionBody10_1 {
        key,
        version,
        shorts,
        double,
        tail_short,
    })
}

/// 从对象头之后的位置读取 10.1 LineLabelPlacementPriorities 体(证据:
/// 确定的)。失败时不推进游标、不修改登记表。
pub fn read_line_label_placement_priorities_body_10_1(
    reader: &mut Reader<'_>,
    _max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LineLabelPlacementPrioritiesBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, 1, "LineLabelPlacementPriorities", entry_offset)?;
    let mut ints = [0i32; 15];
    for slot in &mut ints {
        *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }
    *reader = trial;
    Ok(LineLabelPlacementPrioritiesBody10_1 { key, version, ints })
}

/// 从对象头之后的位置读取 10.1 PointPlacementPriorities 体(证据:确定
/// 的)。失败时不推进游标、不修改登记表。
pub fn read_point_placement_priorities_body_10_1(
    reader: &mut Reader<'_>,
    _max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<PointPlacementPrioritiesBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, 1, "PointPlacementPriorities", entry_offset)?;
    let mut ints = [0i32; 8];
    for slot in &mut ints {
        *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }
    *reader = trial;
    Ok(PointPlacementPrioritiesBody10_1 { key, version, ints })
}

/// 从对象头之后的位置读取 10.1 AnnotationVBScriptEngine 体(证据:确定
/// 的)。失败时不推进游标、不修改登记表。
pub fn read_annotation_vbscript_engine_body_10_1(
    reader: &mut Reader<'_>,
    _max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<AnnotationVbscriptEngineBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let version = read_version_short(&mut trial, 2, "AnnotationVBScriptEngine", entry_offset)?;
    let short = if version >= 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    *reader = trial;
    Ok(AnnotationVbscriptEngineBody10_1 {
        key,
        version,
        short,
    })
}

/// 从对象头之后的位置读取 10.1 HotLinkExpressionProperties 体(证据:样
/// 本结构,见结构体文档)。失败时不推进游标、不修改登记表。
pub fn read_hot_link_expression_props_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<HotLinkExpressionPropsBody10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    let string = read_stream_string_10_1(&mut trial, max_bytes)?;
    let short = trial.read_u16_le()? as i16;
    let mut guid_block = [0u8; 16];
    guid_block.copy_from_slice(trial.read_bytes(16)?);
    *reader = trial;
    Ok(HotLinkExpressionPropsBody10_1 {
        key,
        version,
        string,
        short,
        guid_block,
    })
}

/// ⑰ IBasicOverposterLayerProperties 槽条目分类。
#[derive(Debug, Clone, PartialEq)]
pub enum BolpSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 BasicOverposterLayerProperties 对象:已解析体(体内停止经
    /// [] 透传)。
    Bolp(Box<BolpBody10_1>),
}

/// ⑲ IEnvelope 槽条目分类。
#[derive(Debug, Clone, PartialEq)]
pub enum EnvelopeSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 Envelope 对象:已解析体(对象层读取器,体内含嵌套空间参考;
    /// 嵌套非投影坐标系为硬错误向上传播,无停止点)。
    Envelope(Box<crate::objects::envelope::Envelope10_1>),
}

/// IAnnotationExpressionEngine 槽条目分类(⑱ 与 DisplayExpressionProperties
/// 体内共用)。
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionEngineSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 AnnotationVBScriptEngine 对象:已解析体。
    VbscriptEngine(Box<AnnotationVbscriptEngineBody10_1>),
    /// 新对象但类标识未闭合:停止点,游标已回退到槽位起点、键不保留。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// DisplayExpressionProperties 体前缀(样本 `0x152e..0x155a`;实现函数未
/// 定位,证据为样本结构)的已确认字段。字段序:① int16 版本(样本 1)→
/// ② String(样本 `[BSM]`,显示表达式候选)→ ③ int16(样本 -1)→
/// ④ IAnnotationExpressionEngine 槽(样本 AnnotationVBScriptEngine 键 84)。
/// 其后尾段(样本 `0x155a..0x15d8` 共 126 字节)字段序未闭合,读取器在该
/// 处成功停止(由宿主传播为停止原因)。
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayExpressionPropsPrefix10_1 {
    /// 对象头键(样本 83)。
    pub key: u32,
    /// 体首 short 版本(样本 1);无反编译证据,不设上限。
    pub version: u16,
    /// String(样本 `[BSM]`)。
    pub string: String,
    /// int16(样本 -1),语义未定。
    pub short: i16,
    /// 引擎槽分类。
    pub engine: ExpressionEngineSlot10_1,
}

/// 从对象头之后的位置读取 10.1 DisplayExpressionProperties 体前缀,读到
/// 尾段起点即成功返回(尾段未闭合,由调用者停止)。失败时不推进游标、
/// 不修改登记表。
pub fn read_display_expression_props_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<DisplayExpressionPropsPrefix10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    let string = read_stream_string_10_1(&mut trial, max_bytes)?;
    let short = trial.read_u16_le()? as i16;
    let mut staged = table.clone();
    let slot_offset = trial.position();
    let mut engine = ExpressionEngineSlot10_1::Null;
    match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => {}
        ObjectEntry10_1::BackReference { class_id, key } => {
            engine = ExpressionEngineSlot10_1::BackReference { class_id, key };
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID =>
        {
            let body =
                read_annotation_vbscript_engine_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            engine = ExpressionEngineSlot10_1::VbscriptEngine(Box::new(body));
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            engine = ExpressionEngineSlot10_1::Unsupported { class_id, key };
            trial = reposition_to(reader, slot_offset);
            staged = table.clone();
        }
    }
    *table = staged;
    *reader = trial;
    Ok(DisplayExpressionPropsPrefix10_1 {
        key,
        version,
        string,
        short,
        engine,
    })
}

/// 从对象头之后的位置读取 10.1 BasicOverposterLayerProperties 体,失败时
/// 不推进游标、不修改登记表。
///
/// 起点约定:调用者已读取对象头、比对
/// [`BASIC_OVERPOSTER_LAYER_PROPS_CLASS_ID`] 并取得键 `key`。三个子槽的
/// 新对象按类标识分派(LineLabelPosition / LineLabelPlacementPriorities /
/// PointPlacementPriorities),其余类为成功停止(游标回退到该槽位起点、
/// 键不保留),停止经 [`BolpStop10_1`] 传播。
pub fn read_bolp_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<BolpBody10_1> {
    let mut trial = reader.clone();
    // ① 20 字节标量前缀(读序按 0x7853d3a0:[int16][int16 ×3][int32 ×3];
    //   语义候选,原样保留,无版本门——函数归属未证实)。
    let version = trial.read_u16_le()? as i16;
    let mut shorts = [0i16; 3];
    for slot in &mut shorts {
        *slot = trial.read_u16_le()? as i16;
    }
    let mut ints = [0i32; 3];
    for slot in &mut ints {
        *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }

    let mut staged = table.clone();
    let mut stop = None;

    // ② 槽 1 LineLabelPosition。
    let mut label_position = LineLabelPositionSlot10_1::Null;
    {
        let slot_offset = trial.position();
        let pre = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                label_position = LineLabelPositionSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == LINE_LABEL_POSITION_CLASS_ID =>
            {
                let body =
                    read_line_label_position_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
                label_position = LineLabelPositionSlot10_1::Body(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BolpStop10_1::LabelPositionSlotUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged = pre;
            }
        }
    }

    // ③ 槽 2 LineLabelPlacementPriorities。
    let mut priorities_line = PlacementPrioritiesSlot10_1::Null;
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                priorities_line = PlacementPrioritiesSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == LINE_LABEL_PLACEMENT_PRIORITIES_CLASS_ID =>
            {
                let body = read_line_label_placement_priorities_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?;
                priorities_line = PlacementPrioritiesSlot10_1::Line(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BolpStop10_1::PrioritiesSlotUnsupported {
                    class_id,
                    key,
                    which: 0,
                });
                trial = reposition_to(reader, slot_offset);
                staged = pre;
            }
        }
    }

    // ④ 槽间 16 字节标量块。
    let mut between = BolpBetweenBlock10_1 {
        short_a: 0,
        int_a: 0,
        double: 0.0,
        short_b: 0,
    };
    if stop.is_none() {
        between = BolpBetweenBlock10_1 {
            short_a: trial.read_u16_le()? as i16,
            int_a: i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()),
            double: trial.read_f64_le()?,
            short_b: trial.read_u16_le()? as i16,
        };
    }

    // ⑤ 槽 3 PointPlacementPriorities。
    let mut priorities_point = PlacementPrioritiesSlot10_1::Null;
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                priorities_point = PlacementPrioritiesSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == POINT_PLACEMENT_PRIORITIES_CLASS_ID =>
            {
                let body = read_point_placement_priorities_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?;
                priorities_point = PlacementPrioritiesSlot10_1::Point(Box::new(body));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(BolpStop10_1::PrioritiesSlotUnsupported {
                    class_id,
                    key,
                    which: 1,
                });
                trial = reposition_to(reader, slot_offset);
                staged = pre;
            }
        }
    }

    // ⑥ 尾段 44 字节([int32 计数][计数 × double][double][int32][String]
    //   [int32] + 20 字节未定名块;String 经 BSTR 空语义,cb=0 无载荷)。
    let mut tail = BolpTail10_1 {
        count: 0,
        doubles: Vec::new(),
        double: 0.0,
        int_a: 0,
        string: String::new(),
        int_b: 0,
        block_int_a: 0,
        block_int_b: 0,
        block_double: 0.0,
        block_short_a: 0,
        block_short_b: 0,
    };
    if stop.is_none() {
        let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let mut doubles = Vec::new();
        for _ in 0..count.max(0) {
            doubles.push(trial.read_f64_le()?);
        }
        let double = trial.read_f64_le()?;
        let int_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let string = read_stream_string_10_1(&mut trial, max_bytes)?;
        let int_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let block_int_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let block_int_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let block_double = trial.read_f64_le()?;
        let block_short_a = trial.read_u16_le()? as i16;
        let block_short_b = trial.read_u16_le()? as i16;
        tail = BolpTail10_1 {
            count,
            doubles,
            double,
            int_a,
            string,
            int_b,
            block_int_a,
            block_int_b,
            block_double,
            block_short_a,
            block_short_b,
        };
    }

    let result = BolpBody10_1 {
        key,
        version: version as u16,
        shorts,
        ints,
        label_position,
        priorities_line,
        between,
        priorities_point,
        tail,
        stop,
    };
    *table = staged;
    *reader = trial;
    Ok(result)
}
