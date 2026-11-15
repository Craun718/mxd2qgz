//! ArcMap 10.1 FeatureLayer 尾段链第一环（渲染器 SimpleRenderer 及其直接
//! 后继）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-featurelayer-tail.md`（2026-11-14 第七轮，
//! 本轮新研究记录）。链头是 `feature_layer` 渲染器槽（FeatureLayer::Load
//! `0x6f6e9940` 第 6 步，请求 IID `{40A9E884-…}`=IFeatureRenderer）处出现的
//! SimpleRenderer 新对象（样本 `temp/r6-maps.bin` 流内偏移 `0x3ed`，键 35）。
//! 本模块闭合以下五类体（字段序全部来自 10.1.0.3035 原生 DLL 反编译）：
//!
//! 1. **SimpleRenderer**（FeatureLayer.dll，IPersistStream 虚表 `0x6f7b0dfc`
//!    槽 +0x14 = 调整器 thunk `0x6f724cf0` → 实体 Load `0x6f722b70`，
//!    Save `0x6f722cf0`；GetClassID `0x6f722e00` 写出 CLSID
//!    `{F3435801-5779-11D0-98BF-00805F7CED21}`，与 esri.clsid 声明、样本
//!    `/Maps:0x3ed` 头三方互证）：
//!    ① int16 版本（`3 < (short)ver` 有符号比较，>3 拒绝；样本 3）→
//!    ② LoadObject(ISymbol `{F3435802-…}`)（样本 SimpleMarkerSymbol 键 36）→
//!    ③ LoadObject(ILegendGroup `{167C5E9F-…}`)（样本 LegendGroup 键 39）→
//!    ④ LoadObject(ILegendItem `{52AA5BC0-…}`)（样本 16 零空标记）→
//!    ⑤ int16 → obj+0x128（样本 0）→
//!    ⑥ LoadLegacyRotationFieldAndType(LayerLib.dll `0x786f93d0`，本类调用
//!    不带版本 short)：String 旋转字段 + int32 旋转类型（样本空串 + 0）→
//!    ⑦ String → obj+0x12c 标签（样本空串）→
//!    ⑧ 版本 >2：RotationRenderer::Load（LayerLib.dll `0x786f7e40`：short
//!    版本 ≤1 + int32 ×2 + String ×3 + double ×6）+ SizeRenderer::Load
//!    （LayerLib.dll `0x7870bbc0`：short 版本 ≤1 + int32 + String +
//!    double ×2）。
//!    Save `0x6f722cf0` 对称（写版本表 docVersion→3，样本 3）；docVersion<4
//!    时省略 ⑧（读侧按版本 >2 门，二者在 10.1 一致）。
//! 2. **SimpleMarkerSymbol**（DisplayCore.dll 10.1.0.3035，SHA-256
//!    `9b3e61f9…a926e4d2`，Setup1.cab 新提取；IPersistStream 虚表
//!    `0x6ec15724` 槽 +0x14 = thunk `0x6ebc0460` → 实体 Load `0x6ebbefa0`，
//!    Save `0x6ebbf190`；GetClassID `0x6ebbf310` 写出
//!    `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`）：
//!    ① short 版本（>2 拒绝；样本 2）→ ② LoadObject(IColor
//!    `{20CD40B0-…}`) → ③ double（样本 6.0）→ ④ int32 ×3（样本 0/13/0，
//!    13 为 marker 样式候选）→ ⑤ double ×3（样本 0）→ ⑥ 1 字节（样本 1）→
//!    ⑦ double（样本 1.0）→ ⑧ LoadObject(IColor)（轮廓色）→
//!    ⑨ 版本 >1：short（样本 0xFFFF）。构造中的 `FUN_6eb406f0` 两处调用
//!    不读流（虚调用返回值），无流内字节。
//! 3. **RgbColor**（DisplayCore.dll，IPersistStream 虚表 `0x6ec0c70c` 槽
//!    +0x14 = thunk `0x6eb3dbf0` → 实体 Load `0x6eb3d8b0` → Color 核心
//!    `0x6eb35310`；Save `0x6eb3d8f0`/`0x6eb35470`；GetClassID `0x6eb3d930`
//!    写出 `{7EE9C496-D123-11D0-8383-080009B996CC}`）：
//!    ① short 版本 ≤1 → ② 第二个 short 版本 ≤1（Color 核心）→ ③ 1 字节
//!    分支标志：==0 → 3 个 double（经 Color 虚表 +0x34 装载，样本三实例
//!    flag 均 0）；!=0 → 4 字节 CMYK（虚表 +0x1c）→ ④ 1 字节 → ⑤ 1 字节
//!    （内存值按 `0xff - 流内字节` 取补）。flag==0 分支体长 31 字节，与样本
//!    三实例（键 37/38/154）逐字节吻合。实现（[`RGB_COLOR_CLASS_ID`]、体
//!    读取器）接线轮移入 [`crate::objects::display_color`]，此处仅重导出。
//! 4. **LegendGroup**（Layer.dll 10.1.0.3035，SHA-256 与既有基线一致，
//!    IPersistStream 虚表 `0x7866fa3c` 槽 +0x14 = Load `0x78665cc0`，Save
//!    `0x78665e20`；GetClassID `0x78665c80` 写出
//!    `{167C5EA2-AF20-11D1-8817-080009EC732A}`）：
//!    ① short 版本（>3 拒绝；样本 3）→ ② short → obj+0xc（样本 -1）→
//!    ③ 版本 ≥2：short → obj+0xe（<2 时内存默认 -1；样本 -1）→
//!    ④ String → obj+0x14（样本空串）→ ⑤ int32 计数 + 循环
//!    LoadObject(ILegendClass `{167C5E9E-…}`)（样本 1 → LegendClass 键 40）→
//!    ⑥ 版本 >2：short → obj+0x12（样本 0）。Save 写版本表 docVersion→3；
//!    docVersion>4 时才写 ⑥（读侧按版本 >2 门）。
//! 5. **LegendClass**（Layer.dll，IPersistStream 虚表 `0x7866f728` 槽 +0x14 =
//!    Load `0x78663560`，Save `0x78663640`；GetClassID `0x78662fe0` 写出
//!    `{167C5EA3-AF20-11D1-8817-080009EC732A}`）：
//!    ① short 版本（**反编译未见上限拒绝**，样本 2；本读取器仅按 ver>1 门
//!    分支，不设额外上限）→ ② LoadObject(ISymbol)（样本为对 SimpleMarker
//!    Symbol 键 36 的回引头，无体）→ ③ String 标签（样本空串）→
//!    ④ String 描述（样本空串）→ ⑤ LoadObject(ILegendClassFormat
//!    `{52AA5BC2-…}`)（样本 16 零空标记；新对象体未闭合 = 停止点）→
//!    ⑥ 版本 >1：int32 → obj+0x1c（样本 -1）。
//!
//! 样本对位（辅助观察，不作测试依赖）：`temp/r6-maps.bin` `0x3ed..0x602`
//! 逐字段吻合（见研究记录字段序表）；SimpleRenderer 体终结于 `0x5b0`，其
//! 后 FeatureLayer 的后继字段段（本模块 [`read_feature_layer_successor_10_1`]）
//! 经第八轮闭合至 FieldInfo 计数循环，样本 `0x5b0..0xafe` 逐字节吻合。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
//! 头并比对类常量，从对象头之后的位置调用各 `read_*_body_10_1`；本模块不
//! 重复读取对象头。嵌套条目用 [`crate::storage::object_refs::read_object_entry_10_1`]
//! 分类并登记（登记先于对象体解析是原生语义；为满足「失败不修改登记表」，
//! 全部登记发生在调用者登记表的副本上，成功后整体提交）。槽位新对象类标识
//! 未知时读取器在该槽位起点停止并回退（不保留该键，与 feature_layer 渲染
//! 器槽语义一致）；体内已闭合前缀的停止（如 SMS 的第二个 IColor 槽）传播
//! 为外层停止原因，游标停在内层停止点（与 feature_layer 的
//! `DataSource` 停止传播语义一致）。
//!
//! 未解决问题（详见研究记录「未解决事项」；接线轮更新）：
//! - ILegendItem 槽（`{52AA5BC0-…}`）的 coclass 体未闭合（样本为空标记，
//! 无字节佐证），新对象即停止；
//! - ILegendClassFormat 槽（`{52AA5BC2-…}`）：LegendClassFormat 体已闭合
//! （`legend_format` 模块，接线轮接入），其余 ILegendClassFormat 实现类新
//! 对象维持停止；样本该槽为空标记；
//! - ISymbol 槽：SimpleMarkerSymbol 与本轮闭合的 DisplayCore 同族
//! （SimpleLineSymbol/Fill、MarkerLine/CartographicLine/HashLine，接线轮
//! 经 [`crate::objects::display_symbol`] 分派）整体解析；MultiLayer 等
//! 其余符号类新对象维持停止；
//! - IColor 槽：五类闭合颜色（RgbColor/HsvColor/HlsColor/GrayColor/
//! CmykColor）经 [`crate::objects::display_color`] 分派整体解析（接线轮
//! 接入）；其余颜色类新对象维持停止；
//! - RotationRenderer/SizeRenderer 的「3D 随机旋转上下限」double ×6 中前
//!   3 个样本为 0、后 3 个为 360.0，语义为候选（对称写侧 getter 未逐个定
//!   名）；各 int32/String 的业务命名（style 候选等）按内容推断；
//! - 第三轮研究文档 IID 表中 `{F3435802-…}` 的 Data2 误记为 5799，经本轮
//!   二进制字节与类型库双重复核应为 **5779**（与 SimpleRenderer CLSID 同
//!   Data2），研究记录已更正。
//!
//! 后继字段段（第八轮闭合，见文末 [`read_feature_layer_successor_10_1`]）
//! 的未解决问题：
//! - IHyperlink 循环（请求 IID `{3036D35D-…}`，第五轮类型库定名）的
//!   Hyperlink 族 coclass 体未闭合（样本计数恒 0，无字节佐证），新对象即
//!   停止；
//! - INumberFormat 槽（`{7E4F470D-…}`）只闭合 NumericFormat 一类；
//!   PercentageFormat/FractionFormat/RateFormat 等同族体未闭合（新对象即
//!   停止）；
//! - 后继段各标量的业务语义未定性：+0x154 字符串（样本「BSM」，字段名样
//!   候选）、+0x158/+0x160 double（样本 0.0）、+0x168 short、+0x1a8/+0x184
//!   字符串（样本空串）、+0x188 int32 均原样保留；FieldInfo 的 -0x16
//!   （可见性 VARIANT_BOOL 候选，样本 OBJECTID 0xFFFF/BSM 0x0000）、-0x14、
//!   -0x18/-0x1c（样本 -1/-1）、-0x8/-0x6（样本 0/-1 或 0/0）与
//!   NumericFormat 的 int32 ×4/short ×3（样本 0/0/0/12 与 0/6/0/12 +
//!   0/0/0）同样未定性；
//! - FieldInfo 循环条目名「样本恒与别名同文」为观察，二者分歧场景未见样
//!   本；
//! - FieldInfo 循环之后的 ALPC 环（第九轮闭合：ALPC/LELP/ETM 体与
//!   FeatureIDSet 体，`annotate_props`/`feature_id_set` 模块）、short
//!   +0x1bc、FeatureIDSet 槽、选择集 ID 数组、图层扩展段（EditTemplate
//!   宿主）与更深的版本门字段已按本轮字段序接入：未闭合槽位为成功停止
//!   （样本停止点 = LELP ⑯ ITextSymbol 槽的 TextSymbol 新对象，键 61，
//!   游标停在其体起点）。样本 0xafe..0xb8e 逐字节吻合；
//! - LELP ⑰ BasicOverposterLayerProperties 等标注族实现类、TextSymbol 体
//!   （DisplayCore.dll，虚表静态不可达）与 TimeData 族槽位实现类未闭合
//!   （新对象即停止）。

use crate::layers::annotate_props::{
    ANNOTATE_PROPS_COLLECTION_CLASS_ID, AnnotatePropsBody10_1, AnnotatePropsStop10_1,
    EDIT_TEMPLATE_MANAGER_CLASS_ID, EditTemplateManagerBody10_1, EditTemplateManagerStop10_1,
    read_annotate_props_collection_body_10_1, read_edit_template_manager_body_10_1,
};
use crate::layers::feature_id_set::{
    FEATURE_ID_SET_CLASS_ID, FeatureIdSetBody10_1, read_feature_id_set_body_10_1,
};
use crate::layers::legend_format::{
    LEGEND_CLASS_FORMAT_CLASS_ID, LegendClassFormatBody10_1, LegendClassFormatStop10_1,
    read_legend_class_format_body_10_1,
};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{
    MAX_SYMBOL_NESTING_DEPTH, SimpleMarkerSymbolStop10_1, SymbolStop10_1,
    read_display_symbol_slot_10_1, read_symbol_slot_full_10_1, slot_full_stop_10_1, slot_stop_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// ISymbol 类槽位（请求 IID `{F3435802-…}`）条目的分类结果。
///
/// 接线轮起为对象层 [`crate::objects::display_symbol::DisplaySymbolSlot10_1`]
/// 的等价重导出：SimpleMarkerSymbol（第七轮）与 SimpleLineSymbol/Fill、
/// MarkerLine/CartographicLine/HashLine（本轮闭合的 DisplayCore 同族）新对
/// 象均解析为对应体变体；未知符号类为 `Unsupported` 变体（分派内部回退到
/// 槽位起点、登记表不保留该键），由宿主映射为各自的停止原因。
pub use crate::objects::display_symbol::DisplaySymbolSlot10_1 as SymbolSlot10_1;

/// 全闭合 ISymbol 类槽位（请求 IID `{F3435802-…}`）条目的分类结果。
///
/// 接线轮起 SimpleRenderer/LegendClass 宿主切换到
/// [`crate::objects::display_symbol::read_symbol_slot_full_10_1`]（第二轮闭
/// 合类递归解析；第一轮闭合类停止语义不变），槽位字段类型随切换重导出。
pub use crate::objects::display_symbol::SymbolSlotFull10_1;

/// SimpleRenderer::Load 接受的最大版本：`3 < (short)ver` 有符号比较；样本 3。
const MAX_SIMPLE_RENDERER_VERSION: i16 = 3;

/// LegendGroup::Load 接受的最大版本：`3 < (short)ver`；样本 3。
const MAX_LEGEND_GROUP_VERSION: i16 = 3;

/// RotationRenderer::Load / SizeRenderer::Load 版本上限：各 `1 < ver`；样本 1/1。
const MAX_RENDERER_TAIL_VERSION: i16 = 1;

/// SimpleRenderer 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{F3435801-5779-11D0-98BF-00805F7CED21}`。证据：
/// esri.clsid.ecfg 声明（模块 FeatureLayer.dll）与 esriCarto 类型库
/// coclass #514；FeatureLayer.dll GetClassID `0x6f722e00` 写出同一 CLSID
/// （`0x11d05779f3435801`/`0x21ed7c5f8000bf98` 两个 8 字节立即数）；样本
/// `/Maps:0x3ed` 渲染器槽对象头逐字节相同（键 35）。
pub const SIMPLE_RENDERER_CLASS_ID: [u8; 16] = [
    0x01, 0x58, 0x43, 0xf3, 0x79, 0x57, 0xd0, 0x11, 0x98, 0xbf, 0x00, 0x80, 0x5f, 0x7c, 0xed, 0x21,
];

pub use crate::objects::display_color::RGB_COLOR_CLASS_ID;
/// SimpleMarkerSymbol 与 RgbColor 两类标识（CLSID）现由对象层颜色/符号模块
/// 定义（接线轮移入 [`crate::objects::display_color`] 与
/// [`crate::objects::display_symbol`]，消除 objects→layers 依赖倒置）；此处
/// 重导出保持本模块既有引用路径（feature_layer / 测试）稳定，避免双源。
pub use crate::objects::display_symbol::SIMPLE_MARKER_SYMBOL_CLASS_ID;

/// LegendGroup 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{167C5EA2-AF20-11D1-8817-080009EC732A}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll）；Layer.dll GetClassID `0x78665c80`
/// 写出；样本 `/Maps:0x4b6`（SimpleRenderer 体内 ILegendGroup 槽，键 39）
/// 逐字节相同。
pub const LEGEND_GROUP_CLASS_ID: [u8; 16] = [
    0xa2, 0x5e, 0x7c, 0x16, 0x20, 0xaf, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// LegendClass 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{167C5EA3-AF20-11D1-8817-080009EC732A}`。证据：
/// esri.clsid.ecfg 声明（模块 Layer.dll）；Layer.dll GetClassID `0x78662fe0`
/// 写出；样本 `/Maps:0x4da`（LegendGroup 体内嵌对象头，键 40）逐字节相同。
pub const LEGEND_CLASS_CLASS_ID: [u8; 16] = [
    0xa3, 0x5e, 0x7c, 0x16, 0x20, 0xaf, 0xd1, 0x11, 0x88, 0x17, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// 把游标回退到入口窗口内的绝对偏移（Reader 无随机定位原语，用入口克隆
/// 前向读取实现；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// 读取 int16 并按原生有符号比较校验版本上限；负值位模式一并拒绝
/// （负值版本无业务意义，较原生更严，与 feature_layer 的版本处理一致）。
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

/// IColor 类槽位（请求 IID `{20CD40B0-…}`）条目的分类结果见模块顶部的
/// [`crate::objects::display_color::ColorSlot10_1`] 重导出（五类闭合颜色，
/// 未知类为 `Unsupported` 变体，由宿主映射为停止原因）。

/// RotationRenderer::Load（LayerLib.dll `0x786f7e40`）的尾段字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RotationRendererTail10_1 {
    /// short 版本（>1 拒绝；样本 1）。
    pub version: u16,
    /// int32 ×2（→ obj+8/+0xc；样本 0/0），语义未定（3D 旋转标志候选）。
    pub flags: [i32; 2],
    /// String ×3（→ obj+0x10/+0x14/+0x18；样本全空），语义未定（3D 旋转表
    /// 达式候选）。
    pub expressions: [String; 3],
    /// double ×6（→ obj+0x20..+0x48；样本 0/0/0/360/360/360），语义未定
    /// （3D 随机旋转上下限候选）。
    pub doubles: [f64; 6],
}

/// SizeRenderer::Load（LayerLib.dll `0x7870bbc0`）的尾段字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SizeRendererTail10_1 {
    /// short 版本（>1 拒绝；样本 1）。
    pub version: u16,
    /// int32（→ obj+4；样本 0），语义未定（标志候选）。
    pub flags: i32,
    /// String（→ obj+8；样本空），语义未定（尺寸表达式候选）。
    pub expression: String,
    /// double ×2（→ obj+0x10/+0x18；样本 0.1/100.0），语义未定（尺寸上下
    /// 限候选）。
    pub doubles: [f64; 2],
}

/// SimpleRenderer 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleRendererStop10_1 {
    /// ISymbol 槽的新对象类标识不在闭合符号集合（DisplayCore 同族之外）：
    /// 游标回退到该槽位起点（由符号槽分派内部完成）。
    SymbolUnsupported { class_id: [u8; 16], key: u32 },
    /// ISymbol 槽 SimpleMarkerSymbol 体内的停止（游标停在其停止点，已闭合
    /// 前缀保留）。
    SymbolBody(SimpleMarkerSymbolStop10_1),
    /// ISymbol 槽为本轮闭合的 SimpleLineSymbol/Fill、MarkerLine/
    /// CartographicLine/HashLine 等符号且其体内出现嵌套停止（原因透传，
    /// 游标停在其停止点，已闭合字段保留）。
    SymbolNested(SymbolStop10_1),
    /// ILegendGroup 槽的新对象类标识不是 LegendGroup：游标回退到该槽位起点。
    LegendGroupUnsupported { class_id: [u8; 16], key: u32 },
    /// LegendGroup 体内的停止（游标停在其停止点）。
    LegendGroupBody(LegendGroupStop10_1),
    /// ILegendItem 槽的新对象（coclass 体未闭合，样本为空标记无字节佐证）：
    /// 游标回退到该槽位起点，其后的标量与尾段不可达。
    LegendItemNewObject { class_id: [u8; 16], key: u32 },
}

/// SimpleRenderer（FeatureLayer.dll `0x6f722b70`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleRendererPrefix10_1 {
    /// 对象头键（样本 35 @`/Maps:0x3fd`）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本（样本 3）；有符号比较 >3 拒绝。
    pub version: u16,
    /// ISymbol 槽（请求 IID `{F3435802-…}`）结果。
    pub symbol: SymbolSlotFull10_1,
    /// ILegendGroup 槽（请求 IID `{167C5E9F-…}`）结果。`None` = 前置停止。
    pub legend_group: Option<LegendGroupSlot10_1>,
    /// ILegendItem 槽（请求 IID `{52AA5BC0-…}`）分类；新对象为停止点。
    /// `None` = 未读取。
    pub legend_item: Option<ObjectEntry10_1>,
    /// short → obj+0x128（样本 0），语义未定。`None` = 未读取。
    pub short_128: Option<i16>,
    /// LoadLegacyRotationFieldAndType 的 String 旋转字段（样本空串）。
    /// `None` = 未读取。本类调用不带版本 short（第三参 false）。
    pub rotation_field: Option<String>,
    /// LoadLegacyRotationFieldAndType 的 int32 旋转类型（样本 0）。
    pub rotation_type: Option<i32>,
    /// String → obj+0x12c 标签（样本空串）。`None` = 未读取。
    pub label: Option<String>,
    /// 版本 >2 时 RotationRenderer::Load 尾段（样本有，版本 3）。
    pub rotation_tail: Option<RotationRendererTail10_1>,
    /// 版本 >2 时 SizeRenderer::Load 尾段（样本有）。
    pub size_tail: Option<SizeRendererTail10_1>,
    /// 停止原因；`None` = 体已完整消费（样本形态）。
    pub stop: Option<SimpleRendererStop10_1>,
    /// 停止点之后剩余的字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// ILegendGroup 槽位条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendGroupSlot10_1 {
    /// 空引用标记。
    Null,
    /// 回引条目（无体）。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LegendGroup 对象：已解析体（体内停止经 `stop` 传播）。
    LegendGroup(Box<LegendGroupPrefix10_1>),
}

/// LegendGroup 体内 ILegendClass 循环条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendClassEntry10_1 {
    /// 空引用标记（原生 LoadObject 返回空对象、仍入集合）。
    Null,
    /// 回引条目：无体，类标识原样保留（样本对键 36 的 SimpleMarkerSymbol
    /// 回引发生在 LegendClass 体内，不在本层）。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LegendClass 对象：已解析体。
    NewObject(Box<LegendClassPrefix10_1>),
}

/// LegendGroup 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendGroupStop10_1 {
    /// 第 `index` 个 ILegendClass 循环条目的新对象类标识不是 LegendClass：
    /// 游标回退到该条目对象头起点，版本 >2 尾 short 不可达。
    ClassUnsupported {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
    /// 某个 LegendClass 体内的停止（游标停在其停止点）。
    ClassBody(LegendClassStop10_1),
}

/// LegendGroup（Layer.dll `0x78665cc0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendGroupPrefix10_1 {
    /// 对象头键（样本 39）。
    pub key: u32,
    /// 体首 short 版本（样本 3）；有符号比较 >3 拒绝。
    pub version: u16,
    /// short → obj+0xc（样本 -1），语义未定（可编辑标志候选）。
    pub heading_short: i16,
    /// 版本 ≥2 的 short → obj+0xe（<2 时原生内存默认 -1，不读流；样本 -1）。
    pub editable_short: Option<i16>,
    /// String → obj+0x14（样本空串；标题候选）。
    pub heading: String,
    /// int32 LegendClass 计数（样本 1）。负数原生不进循环，原样保留。
    pub class_count: i32,
    /// LegendClass 循环条目（按流内顺序）。
    pub classes: Vec<LegendClassEntry10_1>,
    /// 版本 >2 的 short → obj+0x12（样本 0）。`None` = 版本 ≤2 或前置停止。
    pub trailing_short: Option<i16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<LegendGroupStop10_1>,
}

/// ISymbol 槽或 ILegendClassFormat 槽的新对象在 LegendClass 体内的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendClassStop10_1 {
    /// ISymbol 槽的新对象类标识不在闭合符号集合：游标回退到该槽位起点
    /// （由符号槽分派内部完成）。
    SymbolUnsupported { class_id: [u8; 16], key: u32 },
    /// ISymbol 槽 SimpleMarkerSymbol 体内的停止（游标停在其停止点）。
    SymbolBody(SimpleMarkerSymbolStop10_1),
    /// ISymbol 槽为本轮闭合的 DisplayCore 同族符号且其体内出现嵌套停止
    /// （原因透传，游标停在其停止点）。
    SymbolNested(SymbolStop10_1),
    /// ILegendClassFormat 槽的新对象类标识非 LegendClassFormat：游标回退到
    /// 该槽位起点，版本 >1 的 int32 不可达。
    FormatNewObject { class_id: [u8; 16], key: u32 },
    /// ILegendClassFormat 槽的 LegendClassFormat 体自身停止（原因透传，游标
    /// 停在其停止点）。
    FormatBody(LegendClassFormatStop10_1),
}

/// ILegendClassFormat 槽位（请求 IID `{52AA5BC2-…}`）条目的分类结果
/// （接线轮接入 [`read_legend_class_format_body_10_1`]）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendClassFormatSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体（样本形态）。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LegendClassFormat 对象（类标识经 [`LEGEND_CLASS_FORMAT_CLASS_ID`]
    /// 校验）：已读取其闭合体（体内停止经 [`LegendClassStop10_1::FormatBody`]
    /// 传播）。
    Format(Box<LegendClassFormatBody10_1>),
    /// 新对象但类标识非 LegendClassFormat（其余 ILegendClassFormat 实现类体
    /// 未闭合）：停止点，游标已回退到该槽位起点、该键不保留。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// LegendClass（Layer.dll `0x78663560`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendClassPrefix10_1 {
    /// 对象头键（样本 40）。
    pub key: u32,
    /// 体首 short 版本（样本 2）。**反编译未见上限拒绝**，本读取器不设上
    /// 限，仅按 ver>1 门分支。
    pub version: u16,
    /// ISymbol 槽（请求 IID `{F3435802-…}`）结果。
    pub symbol: SymbolSlotFull10_1,
    /// String 标签（样本空串）。
    pub label: String,
    /// String 描述（样本空串）。
    pub description: String,
    /// ILegendClassFormat 槽（请求 IID `{52AA5BC2-…}`）分类；新对象为停止
    /// 点。
    pub format: LegendClassFormatSlot10_1,
    /// 版本 >1 的 int32 → obj+0x1c（样本 -1）。`None` = 版本 ≤1 或前置停止。
    pub int32_1c: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<LegendClassStop10_1>,
}

/// 从对象头之后的位置读取 10.1 LegendClass 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LEGEND_CLASS_CLASS_ID`] 并取得键。
/// ISymbol 槽经 [`read_symbol_slot_full_10_1`] 全闭合集分派（接线轮切换，
/// 证据：docs/arcmap-10.1-display-symbols-round2.md 第五节宿主切换——第一
/// 轮闭合类停止语义不变、第二轮闭合类获得递归解析，未知符号类由分派内部
/// 回退到槽位起点）；ILegendClassFormat 槽新对象且类标识为 LegendClassFormat
/// 时读取其闭合体（[`read_legend_class_format_body_10_1`]），其余类为停止点
/// （回退到槽位起点）；体内停止成功返回。
pub fn read_legend_class_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LegendClassPrefix10_1> {
    let mut trial = reader.clone();

    // 1. 版本 short：反编译未见上限检查，原样保留（负值同样原样；门判断按
    //    有符号语义 ver > 1）。
    let version = trial.read_u16_le()?;

    // 2. ISymbol 槽（请求 IID {F3435802-…}）：对象层全闭合符号槽位分派。
    //    SimpleMarkerSymbol 体的停止沿用第一轮 `SymbolBody` 变体；其余闭合
    //    符号（第一轮 SimpleLine/Fill、MarkerLine/CartographicLine/HashLine
    //    与第二轮 MultiLayer/装饰族）体的嵌套停止统一经 `slot_full_stop_
    //    10_1` 归约为 `SymbolNested`（第一轮两入口归约同型，行为不变）。
    let mut staged_table = table.clone();
    let mut stop = None;
    let symbol = match read_symbol_slot_full_10_1(
        &mut trial,
        max_bytes,
        &mut staged_table,
        MAX_SYMBOL_NESTING_DEPTH,
    )? {
        SymbolSlotFull10_1::Null => SymbolSlotFull10_1::Null,
        SymbolSlotFull10_1::BackReference { class_id, key } => {
            SymbolSlotFull10_1::BackReference { class_id, key }
        }
        SymbolSlotFull10_1::SimpleMarkerSymbol(sms) => {
            // 第一轮闭合类的停止沿用原 `SymbolBody` 变体（行为保真）。
            if let Some(inner) = sms.stop {
                stop = Some(LegendClassStop10_1::SymbolBody(inner));
            }
            SymbolSlotFull10_1::SimpleMarkerSymbol(sms)
        }
        SymbolSlotFull10_1::Unsupported { class_id, key } => {
            // 未知符号类：分派内部已回退到槽位起点、登记表不保留该键。
            stop = Some(LegendClassStop10_1::SymbolUnsupported { class_id, key });
            SymbolSlotFull10_1::Null
        }
        other_closed => {
            // 其余闭合符号：体内嵌套停止透传（游标停在其停止点）；第一轮闭
            // 合类的归约结果与原 [`slot_stop_10_1`] 逐值一致。
            if let Some(inner) = slot_full_stop_10_1(
                &other_closed,
                crate::objects::display_symbol::DisplaySlotKind10_1::Symbol,
            ) {
                stop = Some(LegendClassStop10_1::SymbolNested(inner));
            }
            other_closed
        }
    };

    // 3-5. 标签、描述与 ILegendClassFormat 槽（前置停止时不读取）。
    let mut label = String::new();
    let mut description = String::new();
    let mut format = LegendClassFormatSlot10_1::Null;
    let mut int32_1c = None;
    if stop.is_none() {
        label = read_string_10_1(&mut trial, max_bytes)?;
        description = read_string_10_1(&mut trial, max_bytes)?;
        let format_offset = trial.position();
        let pre_format_table = staged_table.clone();
        format = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => LegendClassFormatSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LegendClassFormatSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == LEGEND_CLASS_FORMAT_CLASS_ID =>
            {
                // 接线：LegendClassFormat 体闭合，整体消费（体内停止透传）。
                let body = read_legend_class_format_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged_table,
                    key,
                )?;
                if let Some(inner) = body.stop {
                    stop = Some(LegendClassStop10_1::FormatBody(inner));
                }
                LegendClassFormatSlot10_1::Format(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 其余 ILegendClassFormat 实现类体未闭合：回退到槽位起点并停
                // 止（该键不保留），版本 >1 的 int32 不可达。
                stop = Some(LegendClassStop10_1::FormatNewObject { class_id, key });
                trial = reposition_to(reader, format_offset);
                staged_table = pre_format_table;
                LegendClassFormatSlot10_1::Unsupported { class_id, key }
            }
        };
        // 6. 版本 >1 的 int32。
        if stop.is_none() && (version as i16) > 1 {
            int32_1c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
    }

    let result = LegendClassPrefix10_1 {
        key,
        version,
        symbol,
        label,
        description,
        format,
        int32_1c,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 LegendGroup 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LEGEND_GROUP_CLASS_ID`] 并取得键。
/// 循环内非 LegendClass 新对象为停止点（回退到该条目对象头起点）。
pub fn read_legend_group_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LegendGroupPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 版本 short（有符号 >3 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_LEGEND_GROUP_VERSION,
        "LegendGroup",
        entry_offset,
    )?;
    // 2. short → obj+0xc。
    let heading_short = trial.read_u16_le()? as i16;
    // 3. 版本 ≥2：short → obj+0xe（<2 时内存默认 -1，不读流）。
    let editable_short = if (version as i16) >= 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    // 4. String → obj+0x14。
    let heading = read_string_10_1(&mut trial, max_bytes)?;

    // 登记表副本：循环内条目登记先发生在副本上，完整成功（含停止）才随
    // 外层提交；任何失败路径直接向上传播，调用者的登记表不动。
    let mut staged_table = table.clone();

    // 5. int32 计数 + 循环 LoadObject(ILegendClass)。全部登记发生在
    //    staged_table 副本上，整体成功才随外层提交。
    let class_count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let mut classes = Vec::new();
    let mut stop = None;
    for index in 0..class_count.max(0) as usize {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        match entry {
            ObjectEntry10_1::Null => classes.push(LegendClassEntry10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                classes.push(LegendClassEntry10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == LEGEND_CLASS_CLASS_ID => {
                let lc =
                    read_legend_class_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(inner) = lc.stop.clone() {
                    stop = Some(LegendGroupStop10_1::ClassBody(inner));
                }
                classes.push(LegendClassEntry10_1::NewObject(Box::new(lc)));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(LegendGroupStop10_1::ClassUnsupported {
                    class_id,
                    key,
                    index,
                });
                trial = reposition_to(reader, item_offset);
                // 回退本条目已做的登记（其后的条目尚未登记）。
                staged_table = pre_item_table;
            }
        }
        if stop.is_some() {
            break;
        }
    }

    // 6. 版本 >2 的 short → obj+0x12（前置停止时不读取）。
    let trailing_short = if stop.is_none() && (version as i16) > 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };

    let result = LegendGroupPrefix10_1 {
        key,
        version,
        heading_short,
        editable_short,
        heading,
        class_count,
        classes,
        trailing_short,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 SimpleRenderer 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者（如 feature_layer 渲染器槽分派的后续接线）已读取对象
/// 头、比对 [`SIMPLE_RENDERER_CLASS_ID`] 并取得键 `key`。`max_bytes` 应用于
/// 全部字符串（含嵌套 LegendGroup/LegendClass 体内字符串）的声明长度。
///
/// ISymbol 槽经 [`read_display_symbol_slot_10_1`] 分派（SimpleMarkerSymbol
/// 与本轮闭合的 DisplayCore 同族符号整体解析，未知符号类由分派内部回退到
/// 槽位起点），ILegendGroup 槽仅支持 LegendGroup，ILegendItem 槽新对象为停
/// 止点（后两者均回退到槽位起点、不保留该键）；嵌套体内的停止传播为本读取
/// 器停止原因，游标停在内层停止点。截断、版本超限、字符串超限为错误（游
/// 标与登记表不动）。
pub fn read_simple_renderer_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleRendererPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 版本 short（有符号 >3 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SIMPLE_RENDERER_VERSION,
        "SimpleRenderer",
        entry_offset,
    )?;

    // 2. ISymbol 槽（请求 IID {F3435802-…}）：对象层符号槽位分派。
    let mut staged_table = table.clone();
    let mut stop = None;
    let symbol = match read_symbol_slot_full_10_1(
        &mut trial,
        max_bytes,
        &mut staged_table,
        MAX_SYMBOL_NESTING_DEPTH,
    )? {
        SymbolSlotFull10_1::Null => SymbolSlotFull10_1::Null,
        SymbolSlotFull10_1::BackReference { class_id, key } => {
            SymbolSlotFull10_1::BackReference { class_id, key }
        }
        SymbolSlotFull10_1::SimpleMarkerSymbol(sms) => {
            // 第一轮闭合类的停止沿用原 `SymbolBody` 变体（行为保真）。
            if let Some(inner) = sms.stop {
                stop = Some(SimpleRendererStop10_1::SymbolBody(inner));
            }
            SymbolSlotFull10_1::SimpleMarkerSymbol(sms)
        }
        SymbolSlotFull10_1::Unsupported { class_id, key } => {
            // 未知符号类：分派内部已回退到槽位起点、登记表未保留该键。
            stop = Some(SimpleRendererStop10_1::SymbolUnsupported { class_id, key });
            SymbolSlotFull10_1::Null
        }
        other_closed => {
            // 其余闭合符号（第一轮 SimpleLine/Fill、MarkerLine/
            // CartographicLine/HashLine 与第二轮 MultiLayer/装饰族）：体内嵌
            // 套停止透传（游标停在其停止点）；第一轮闭合类的归约结果与原
            // [`slot_stop_10_1`] 逐值一致。
            if let Some(inner) = slot_full_stop_10_1(
                &other_closed,
                crate::objects::display_symbol::DisplaySlotKind10_1::Symbol,
            ) {
                stop = Some(SimpleRendererStop10_1::SymbolNested(inner));
            }
            other_closed
        }
    };

    // 3. ILegendGroup 槽（请求 IID {167C5E9F-…}）。
    let mut legend_group = None;
    if stop.is_none() {
        let group_offset = trial.position();
        let pre_group_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => legend_group = Some(LegendGroupSlot10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                legend_group = Some(LegendGroupSlot10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == LEGEND_GROUP_CLASS_ID => {
                let lg =
                    read_legend_group_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(inner) = lg.stop.clone() {
                    stop = Some(SimpleRendererStop10_1::LegendGroupBody(inner));
                }
                legend_group = Some(LegendGroupSlot10_1::LegendGroup(Box::new(lg)));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(SimpleRendererStop10_1::LegendGroupUnsupported { class_id, key });
                trial = reposition_to(reader, group_offset);
                staged_table = pre_group_table;
                legend_group = Some(LegendGroupSlot10_1::Null);
            }
        }
    }

    // 4. ILegendItem 槽（请求 IID {52AA5BC0-…}）：coclass 体未闭合，新对象
    //    即停止（回退到槽位起点）。
    let mut legend_item = None;
    if stop.is_none() {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            stop = Some(SimpleRendererStop10_1::LegendItemNewObject { class_id, key });
            trial = reposition_to(reader, item_offset);
            staged_table = pre_item_table;
        }
        legend_item = Some(entry);
    }

    // 5-7. short +0x128、LoadLegacyRotationFieldAndType（无版本 short）与
    //      String +0x12c。
    let mut short_128 = None;
    let mut rotation_field = None;
    let mut rotation_type = None;
    let mut label = None;
    if stop.is_none() {
        short_128 = Some(trial.read_u16_le()? as i16);
        rotation_field = Some(read_string_10_1(&mut trial, max_bytes)?);
        rotation_type = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        label = Some(read_string_10_1(&mut trial, max_bytes)?);
    }

    // 8. 版本 >2：RotationRenderer::Load + SizeRenderer::Load 尾段。
    let mut rotation_tail = None;
    let mut size_tail = None;
    if stop.is_none() && (version as i16) > 2 {
        let tail_offset = trial.position();
        let rv = read_version_short(
            &mut trial,
            MAX_RENDERER_TAIL_VERSION,
            "RotationRenderer",
            tail_offset,
        )?;
        let mut flags = [0_i32; 2];
        for slot in flags.iter_mut() {
            *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        let mut expressions: [String; 3] = Default::default();
        for slot in expressions.iter_mut() {
            *slot = read_string_10_1(&mut trial, max_bytes)?;
        }
        let mut doubles = [0_f64; 6];
        for slot in doubles.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        rotation_tail = Some(RotationRendererTail10_1 {
            version: rv,
            flags,
            expressions,
            doubles,
        });

        let size_offset = trial.position();
        let sv = read_version_short(
            &mut trial,
            MAX_RENDERER_TAIL_VERSION,
            "SizeRenderer",
            size_offset,
        )?;
        let size_flags = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let expression = read_string_10_1(&mut trial, max_bytes)?;
        let mut size_doubles = [0_f64; 2];
        for slot in size_doubles.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        size_tail = Some(SizeRendererTail10_1 {
            version: sv,
            flags: size_flags,
            expression,
            doubles: size_doubles,
        });
    }

    let remaining_bytes = trial.remaining();
    let result = SimpleRendererPrefix10_1 {
        key,
        version,
        symbol,
        legend_group,
        legend_item,
        short_128,
        rotation_field,
        rotation_type,
        label,
        rotation_tail,
        size_tail,
        stop,
        remaining_bytes,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// FieldInfo 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{A2BAAE2D-969B-11D2-AE77-080009EC732A}`。证据：
/// esri.clsid 声明（模块 Layer.dll，progID FieldInfo）；Layer.dll GetClassID
/// `0x7865b4e0` 经 0x11D2969BA2BAAE2D / 0x2A73EC09000877AE 两个 8 字节立即
/// 数写出（IPersistStream 虚表 `0x7866dd58` 槽 +0x0c 经调整器 thunk
/// `0x7865b8f0` 指向）；样本 `/Maps:0x61c` 起 14 个 FieldInfo 对象头逐字节
/// 相同（键 41..58 中的 FieldInfo 部分）。
pub const FIELD_INFO_CLASS_ID: [u8; 16] = [
    0x2d, 0xae, 0xba, 0xa2, 0x9b, 0x96, 0xd2, 0x11, 0xae, 0x77, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// NumericFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F4719-8E54-11D2-AAD8-000000000000}`。证据：
/// esri.clsid 声明（模块 SystemCore.dll）与 esriSystem 类型库 coclass #212；
/// SystemCore.dll GetClassID `0x7ffce470` 写出（0x11D28E547E4F4719 /
/// 0x000000000000D8AA 立即数，IPersistStream 虚表 `0x80070ec4` 槽 +0x0c 经
/// thunk `0x7ffcea60` 指向）；样本 `/Maps:0x64c` 起的嵌套对象头逐字节相同。
pub const NUMERIC_FORMAT_CLASS_ID: [u8; 16] = [
    0x19, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// FieldInfo::Load 接受的最大版本：`4 < (short)ver` 有符号比较；样本 14 个
/// 实例全为 4。证据：Layer.dll `0x7865b520`。
const MAX_FIELD_INFO_VERSION: i16 = 4;

/// NumericFormat::Load 接受的最大版本：`1 < (short)ver`；样本 4 个实例全为
/// 1。证据：SystemCore.dll `0x7ffce4b0`。
const MAX_NUMERIC_FORMAT_VERSION: i16 = 1;

/// NumericFormat（SystemCore.dll `0x7ffce4b0`）体的已确认字段。
///
/// 体长恒 24 字节（short 版本 + int32 ×4 + short ×3），无嵌套对象、无字符
/// 串，无停止点：要么完整消费，要么失败（截断/版本超限）。各 int32/short
/// 语义未定（样本形态 0/0/0/12 与 0/6/0/12 + 0/0/0；数值格式化对齐/位数
/// 候选）。
#[derive(Debug, Clone, PartialEq)]
pub struct NumericFormatBody10_1 {
    /// 对象头键（样本 42/53/55/57）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本（样本 1）；>1 原生拒绝。
    pub version: u16,
    /// int32 ×4 → obj-0x1c/-0x18/-0x14/-0x10（样本 OBJECTID 条目 0/0/0/12，
    /// XZBZ/YZBZ/ZZBZ 条目 0/6/0/12），语义未定，原样保留。
    pub int32s: [i32; 4],
    /// short ×3 → obj-0xc/-0xa/-0x8（样本全 0），语义未定，原样保留。
    pub shorts: [i16; 3],
}

/// 从对象头之后的位置读取 10.1 NumericFormat 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`NUMERIC_FORMAT_CLASS_ID`] 并取得键
/// `key`。证据：SystemCore.dll 10.1.0.3035 `0x7ffce4b0`（Load）与
/// `0x7ffce590`（Save，对称：写版本表恒 1 + 同序字段）。
pub fn read_numeric_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<NumericFormatBody10_1> {
    let _ = table; // 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // ① short 版本（`1 < ver` 有符号拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_NUMERIC_FORMAT_VERSION,
        "NumericFormat",
        entry_offset,
    )?;
    // ② int32 ×4。
    let mut int32s = [0_i32; 4];
    for slot in int32s.iter_mut() {
        *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }
    // ③ short ×3。
    let mut shorts = [0_i16; 3];
    for slot in shorts.iter_mut() {
        *slot = trial.read_u16_le()? as i16;
    }

    let result = NumericFormatBody10_1 {
        key,
        version,
        int32s,
        shorts,
    };
    *reader = trial;
    Ok(result)
}

/// FieldInfo 体内 INumberFormat 槽位（请求 IID `{7E4F470D-…}`）条目的分类
/// 结果。
#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormatSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体（样本 10/14 条目）。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 NumericFormat 对象（类标识经 [`NUMERIC_FORMAT_CLASS_ID`] 校验）：
    /// 已解析体。
    NumericFormat(Box<NumericFormatBody10_1>),
}

/// FieldInfo 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldInfoStop10_1 {
    /// INumberFormat 槽的新对象类标识不是 NumericFormat（PercentageFormat/
    /// FractionFormat/RateFormat 等同族体未闭合）：游标回退到该槽位起点，
    /// 版本 >2/版本 >3 的尾段不可达。
    NumberFormatUnsupported { class_id: [u8; 16], key: u32 },
}

/// FieldInfo（Layer.dll `0x7865b520`）体的已确认字段。
///
/// 字段序：① short 版本（`4 < ver` 有符号拒绝；样本 4）→ ② short →
/// obj-0x16（无条件；样本 OBJECTID 0xFFFF、BSM 0x0000，可见性候选）→
/// ③ 版本 >1：short → obj-0x14（样本 0）→ ④ String 别名 → obj-0x10（样本
/// 与循环条目名同文）→ ⑤ LoadObject(INumberFormat `{7E4F470D-…}`) →
/// obj-0xc → ⑥ 版本 >2：short → obj-0x18 + int32 → obj-0x1c（样本 -1/-1）
/// → ⑦ 版本 >3：short → obj-0x8 + short → obj-0x6（样本 0/-1 或 0/0）。
/// Save `0x7865b660` 对称（写版本表 docVersion→4；docVersion≥1 才写 ③、
/// >2 写 ⑥、>3 写 ⑦——读侧门在 10.1 docVersion 下逐段一致）。
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfoBody10_1 {
    /// 对象头键（样本 41..58 中的 FieldInfo 部分）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本（样本 4）；有符号比较 >4 拒绝。
    pub version: u16,
    /// short → obj-0x16 的原始位模式（可见性 VARIANT_BOOL 候选；样本
    /// OBJECTID 0xFFFF=可见、BSM 0x0000=隐藏）。语义未定，原样保留。
    pub short_16: i16,
    /// 版本 >1 的 short → obj-0x14（样本 0），语义未定。`None` = 版本 ≤1。
    pub short_14: Option<i16>,
    /// String 别名 → obj-0x10（样本与条目名同文；别名候选）。
    pub alias: String,
    /// INumberFormat 槽（请求 IID `{7E4F470D-…}`=INumberFormat）分类。
    pub number_format: NumberFormatSlot10_1,
    /// 版本 >2 的 short → obj-0x18（样本 -1），语义未定。`None` = 版本 ≤2
    /// 或前置停止未读取。
    pub short_18: Option<i16>,
    /// 版本 >2 的 int32 → obj-0x1c（样本 -1），语义未定。`None` = 版本 ≤2
    /// 或前置停止未读取。
    pub int32_1c: Option<i32>,
    /// 版本 >3 的 short ×2 → obj-0x8/obj-0x6（样本 0/-1 或 0/0），语义未
    /// 定。`None` = 版本 ≤3 或前置停止未读取。
    pub tail_shorts: Option<[i16; 2]>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FieldInfoStop10_1>,
}

/// 从对象头之后的位置读取 10.1 FieldInfo 对象体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`FIELD_INFO_CLASS_ID`] 并取得键
/// `key`。INumberFormat 槽的新对象类标识非 NumericFormat 时是成功停止（停
/// 止原因携带类标识与键，游标回退到该槽位起点、该键不保留）；截断与版本
/// 超限为错误。
pub fn read_field_info_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FieldInfoBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // ① 版本 short（有符号 >4 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_FIELD_INFO_VERSION,
        "FieldInfo",
        entry_offset,
    )?;
    // ② short → obj-0x16。
    let short_16 = trial.read_u16_le()? as i16;
    // ③ 版本 >1：short → obj-0x14。
    let short_14 = if (version as i16) > 1 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    // ④ String 别名 → obj-0x10。
    let alias = read_string_10_1(&mut trial, max_bytes)?;

    // ⑤ INumberFormat 槽：新对象仅支持 NumericFormat，其余为停止点（回退
    //    到槽位起点、不保留该键）。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let (number_format, stop) = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => (NumberFormatSlot10_1::Null, None),
        ObjectEntry10_1::BackReference { class_id, key } => {
            (NumberFormatSlot10_1::BackReference { class_id, key }, None)
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == NUMERIC_FORMAT_CLASS_ID => {
            let body = read_numeric_format_body_10_1(&mut trial, &mut staged_table, key)?;
            (NumberFormatSlot10_1::NumericFormat(Box::new(body)), None)
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            let stop = FieldInfoStop10_1::NumberFormatUnsupported { class_id, key };
            trial = reposition_to(reader, slot_offset);
            staged_table = saved_table;
            (NumberFormatSlot10_1::Null, Some(stop))
        }
    };

    // ⑥⑦ 版本 >2 / 版本 >3 的尾段（前置停止时不读取）。
    let (mut short_18, mut int32_1c, mut tail_shorts) = (None, None, None);
    if stop.is_none() && (version as i16) > 2 {
        short_18 = Some(trial.read_u16_le()? as i16);
        int32_1c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        if (version as i16) > 3 {
            let a = trial.read_u16_le()? as i16;
            let b = trial.read_u16_le()? as i16;
            tail_shorts = Some([a, b]);
        }
    }

    let result = FieldInfoBody10_1 {
        key,
        version,
        short_16,
        short_14,
        alias,
        number_format,
        short_18,
        int32_1c,
        tail_shorts,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// FieldInfo 计数循环的单个条目。
///
/// FeatureLayer::Load（`0x6f6e9940`，汇编 `0x6f6e9c91`: `CMP word ptr
/// [ESP+0x20],5`/`JLE`）在架构版本 >5 时先读条目名字符串、≤5 时从要素类模
/// 式接口推导名字（流内无名字段，读取器保留 `None`）；随后加载 FieldInfo
/// 对象并在「名字与对象都非空」时经 this+0x1d0 接口虚表 +0x18 登记。
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfoLoopEntry10_1 {
    /// 条目名（样本为字段名同文串）。`None` = FeatureLayer 版本 ≤5（流内
    /// 无名字段）或前置停止未读取。
    pub name: Option<String>,
    /// FieldInfo 对象条目分类。
    pub slot: FieldInfoEntry10_1,
}

/// FieldInfo 计数循环条目的对象分类。
#[derive(Debug, Clone, PartialEq)]
pub enum FieldInfoEntry10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 FieldInfo 对象（类标识经 [`FIELD_INFO_CLASS_ID`] 校验）：已解析体
    /// （体内停止经 `FieldInfoBody10_1::stop` 传播）。
    FieldInfo(Box<FieldInfoBody10_1>),
}

/// IHyperlink 计数循环（请求 IID `{3036D35D-…}`）条目的分类结果。
///
/// Hyperlink 族 coclass 体未闭合（样本计数恒 0，无字节佐证），新对象即停
/// 止，故无「已解析」变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyperlinkSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// FeatureLayer 后继字段段（渲染器槽 +0x144 GUID 之后）读取的停止原因。
///
/// 全部为「成功停止」：游标停在最深可闭合点，剩余字节原样保留。接线轮起
/// 标注环体内 TextSymbol 体停止透传携带体类型（`AnnotatePropsBody` 变体不
/// 再 `Copy`/`Eq`，仅 `Clone`/`PartialEq`）。
#[derive(Debug, Clone, PartialEq)]
pub enum SuccessorStop10_1 {
    /// 后继段完整消费（读取器当前位置即 FeatureLayer 对象体语义终点）。
    /// 10.1 写侧对称序列的最后一个字段为 `short(+0x2ca)`（读侧版本 >0x21
    /// 门），其后无流内字段。
    TailCompleted,
    /// FieldInfo 计数循环后的延续段因流耗尽无法开始（ALPC 槽对象头 16 字
    /// 节都读不满），或 ALPC 槽头类标识不是
    /// AnnotateLayerPropertiesCollection（游标均回退/停在循环终点；后者与
    /// 既有调用方断言兼容，停止原因不再细分）。
    FieldInfoLoopCompleted,
    /// ALPC 体内的停止（游标停在其停止点）。
    AnnotatePropsBody(AnnotatePropsStop10_1),
    /// FeatureIDSet 槽的新对象类标识不是 FeatureIDSet：游标回退到该槽位
    /// 起点。
    FeatureIdSetUnsupported { class_id: [u8; 16], key: u32 },
    /// IDisplayFilter 槽（`{19F85377-…}`，版本 ≥2）的新对象类标识未闭合
    /// （样本空标记）：游标回退到该槽位起点，其后的 3 short 段不可达。
    /// 此前「该槽为 EditTemplate 宿主」的假设已撤销（宿主 = 图层扩展段的
    /// EditTemplateManager）。
    DisplayFilterUnsupported { class_id: [u8; 16], key: u32 },
    /// IName 槽（`{4BA33331-…}`，版本 ≥4）的新对象类标识未闭合。
    NameUnsupported { class_id: [u8; 16], key: u32 },
    /// 关系类循环第 `index` 条（`{311C2EE2-…}`，版本 >6）的新对象类标识
    /// 未闭合。
    RelationshipClassUnsupported {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
    /// IEnvelope 槽（`{1B077914-…}`）的新对象类标识未闭合；`which` 为槽位
    /// 序（0 = 版本 >8 槽，1 = 版本 >0xe 槽）。Envelope 体本身未在 src/
    /// 闭合（样本两槽均空标记）。
    EnvelopeUnsupported {
        class_id: [u8; 16],
        key: u32,
        which: u8,
    },
    /// 图层扩展段（版本 ≥0xb）某条目的类标识未闭合：游标回退到该条目
    /// `[int64 尺寸]` 字段起点（该条目整体不消费），其后的 double +0x1e8
    /// 等不可达。
    LayerExtensionUnsupported { class_id: [u8; 16], size: i64 },
    /// 扩展条目为 EditTemplateManager 但体内停止（游标停在其停止点）。
    EditTemplateManagerBody(EditTemplateManagerStop10_1),
    /// ISymbol 槽（`{F3435802-…}`，版本 >0xd）经符号族分派后的未闭合类。
    SymbolSlotUnsupported { class_id: [u8; 16], key: u32 },
    /// IUID 槽（`{1714D59B-…}`，版本 >0x10）的新对象类标识未闭合。
    UidUnsupported { class_id: [u8; 16], key: u32 },
    /// `{A62B9EC2-…}`（版本 >0x19）的新对象类标识未闭合。
    ExpressionPropsUnsupported { class_id: [u8; 16], key: u32 },
    /// 版本 0x1c..0x1f 分支（10.1 不写）的槽位新对象类标识未闭合；`which`
    /// 为槽序（0 = `{F6CBA4A3-…}`，1 = `{BD724B95-…}`，2 = `{E720618B-…}`）。
    LegacyTimeSlotUnsupported {
        class_id: [u8; 16],
        key: u32,
        which: u8,
    },
    /// TimeData 族（版本 ≥0x20）槽位的新对象类标识未闭合；`which` 为槽序
    /// （0 = TimeData，1 = TimeTable）。
    TimeSlotUnsupported {
        class_id: [u8; 16],
        key: u32,
        which: u8,
    },
    /// IHyperlink 循环第 `index` 条（0 起）为新对象（Hyperlink 族 coclass
    /// 体未闭合）：游标回退到该条目对象头起点，其后条目不可达。
    HyperlinkNewObject {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
    /// IColor 槽（请求 IID `{20CD40B0-…}`）的新对象类标识不是 RgbColor：
    /// 游标回退到该槽位起点，其后的字符串段不可达。
    ColorUnsupported { class_id: [u8; 16], key: u32 },
    /// FieldInfo 循环第 `index` 条（0 起）的新对象类标识不是 FieldInfo：
    /// 游标回退到该条目对象头起点，其后条目不可达。
    FieldInfoEntryUnsupported {
        class_id: [u8; 16],
        key: u32,
        index: usize,
    },
    /// 某 FieldInfo 体内的停止（游标停在其 INumberFormat 槽位起点）。
    FieldInfoBody(FieldInfoStop10_1),
}

/// FeatureLayer::Load（`0x6f6e9940`）渲染器槽 +0x144 GUID 之后的后继字段段
/// 已确认前缀。
///
/// 字段序（全部来自 10.1.0.3035 FeatureLayer.dll 反编译，样本
/// `0x5b0..0xb8e` 逐字节吻合；对象内存偏移为反编译槽位）：
/// ① String → obj+0x154 → ② double → obj+0x158 → ③ double → obj+0x160 →
/// ④ short → obj+0x168 → ⑤ int32 计数 + 循环 LoadObject(IHyperlink
/// `{3036D35D-…}`)（每项仅 LoadObject + 数组入集辅助 `0x6f701ad0`，无其他
/// 流读取）→ ⑥ LoadObject(IColor `{20CD40B0-…}`) → ⑦ String → obj+0x1a8 →
/// ⑧ String → obj+0x184 → ⑨ int32 → obj+0x188 → ⑩ int32 FieldInfo 计数 +
/// 循环 [（版本 >5）String 条目名 + LoadObject(IFieldInfo
/// `{A2BAAE2C-…}`)] → ⑪ LoadObject(ALPC `{1D5849F2-…}`，第九轮闭合体) →
/// ⑫ short → obj+0x1bc → ⑬ LoadObject(FeatureIDSet `{D5BB4B87-…}`) →
/// ⑭ int32 选择集计数 + count × int32（原生单次整块读）→ ⑮ short →
/// obj+0x1fe → ⑯ LoadObject(IDisplayFilter `{19F85377-…}`，版本 ≥2) →
/// ⑰ short ×3 → obj+0x21c/+0x220/+0x21e（版本 ≥3）→ ⑱ int32 → obj+0x22c
/// （版本 ≥0xc）→ ⑲ LoadObject(IName `{4BA33331-…}`，版本 ≥4) → ⑳ int32
/// 关系类计数 + 循环 LoadObject({311C2EE2-…})（版本 >6）→ ㉑ String →
/// obj+0x18c（版本 ≥8）→ ㉒ LoadObject(IEnvelope `{1B077914-…}`，版本 >8)
/// → ㉓ int32 范围计数 + count × int32（版本 >9，逐元素读）→ ㉔ 图层扩展段
/// （版本 ≥0xb；LayerLib.dll `ExtensionHelper::LoadExtensions` 格式：
/// [int32 计数][每条 int64 尺寸 + 16 字节 CLSID + 体]；EditTemplateManager
/// 体在本轮闭合，EditTemplate 宿主即此）→ ㉕ double → obj+0x1e8（版本
/// ≥0xd）→ ㉖ LoadObject(ISymbol `{F3435802-…}`，版本 >0xd) → ㉗ short →
/// obj+0x12e + int32 → obj+0x1f0（版本 >0xe）→ ㉘ LoadObject(IEnvelope，
/// 版本 >0xe) → ㉙ LoadObject(IUID `{1714D59B-…}`，版本 >0x10) → ㉚ String
/// → obj+0x28c（版本 ≥0x12）→ ㉛ short → obj+0x1d8（版本 ≥0x13）→ ㉜
/// double ×2 → obj+0x290/+0x298（版本 ≥0x14）→ ㉝ short → obj+0x1da（版本
/// ≥0x15）→ ㉞ int32 → obj+0x244（版本 ≥0x16）→ ㉟ short → obj+0x1be（版本
/// ≥0x17）→ ㊱ CInternalLayerBase::ReadPreJoinDefExpressions（LayerLib.dll
/// `0x786c5d30`：int32 计数 + count × [String + String]，版本 >0x17）→
/// ㊲ int32 → obj+0x2c0 + short ×3（版本 ≥0x19）→ ㊳ String ×4 →
/// obj+0x2cc/+0x2d0/+0x2d4/+0x2d8（版本 ≥0x19）→ ㊴ LoadObject({A62B9EC2-…}
/// ，版本 >0x19) → ㊵ short → obj+0x300 + String ×3（版本 >0x1a）→ ㊶
/// TimeData 族四段（版本 ≥0x20；LayerLib.dll `0x7870d8b0`/`0x7870df80`/
/// `0x7870e250`/`0x7870f6c0`，本轮闭合）→ ㊷ LoadObject({E720618B-…}，版本
/// >0x20) → ㊸ short → obj+0x2ca（版本 >0x21）。
/// 中间的 `FUN_6f6e0c00`（+0x144 GUID 后）、`FUN_6f6c8100`、`FUN_6f6acc10`、
/// `FUN_6f701ad0`、`FUN_6f6dd210`、`FUN_6f6fd550`、`FUN_6f7025b0`（选择集
/// 刷新路径）经反编译核对均不读流，无流内字节。
///
/// 版本 0x1c..0x1f 分支（`{F6CBA4A3}/{BD724B95}/{E720618B}` 槽 + 标量块，
/// 10.1 docVersion 不写）：字段序已静态闭合但无样本字节佐证，实现按门
/// `0x1c ≤ ver ≤ 0x1f` 接入、槽位新对象为停止点（[`SuccessorStop10_1::
/// LegacyTimeSlotUnsupported`]），并注明较原生更严的负值版本拒绝。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureLayerSuccessor10_1 {
    /// String → obj+0x154（样本「BSM」；语义未定，字段名样字符串候选）。
    pub string_154: String,
    /// double → obj+0x158（样本 0.0），语义未定。
    pub double_158: f64,
    /// double → obj+0x160（样本 0.0），语义未定。
    pub double_160: f64,
    /// short → obj+0x168（样本 0），语义未定。
    pub short_168: i16,
    /// IHyperlink 循环声明计数（样本 0）。负数原生不进循环，原样保留。
    pub hyperlink_count: i32,
    /// IHyperlink 循环已消费条目（按流内顺序）。
    pub hyperlinks: Vec<HyperlinkSlot10_1>,
    /// IColor 槽（请求 IID `{20CD40B0-…}`）分类；新对象仅支持 RgbColor。
    pub color: ColorSlot10_1,
    /// String → obj+0x1a8（样本空串），语义未定。
    pub string_1a8: String,
    /// String → obj+0x184（样本空串），语义未定。
    pub string_184: String,
    /// int32 → obj+0x188（样本 0），语义未定。
    pub int32_188: i32,
    /// FieldInfo 循环声明计数（样本 14）。负数原生不进循环，原样保留。
    pub field_info_count: i32,
    /// FieldInfo 循环已消费条目（按流内顺序）。
    pub field_infos: Vec<FieldInfoLoopEntry10_1>,
    /// ⑪ ALPC 槽（请求 IID `{1D5849F2-…}`）分类；新对象仅支持
    /// AnnotateLayerPropertiesCollection。前置停止时 `None`。
    pub annotate_props: Option<AnnotatePropsSlot10_1>,
    /// ⑫ short → obj+0x1bc（样本 0xFFFF），语义未定。前置停止时 `None`。
    pub short_1bc: Option<i16>,
    /// ⑬ FeatureIDSet 槽分类。前置停止时 `None`。
    pub feature_id_set: Option<FeatureIdSetSlot10_1>,
    /// ⑭ 选择集 ID 计数（样本 0）。负数原生不进循环，原样保留。前置停止
    /// 时 `None`。
    pub selection_id_count: Option<i32>,
    /// ⑭ 已消费的选择集 ID（原生单次整块读 count × int32）。
    pub selection_ids: Vec<i32>,
    /// ⑮ short → obj+0x1fe（样本 -1），语义未定。前置停止时 `None`。
    pub short_1fe: Option<i16>,
    /// ⑯ IDisplayFilter 槽分类（版本 ≥2）。前置停止/版本不足时 `None`。
    pub display_filter: Option<RefSlot10_1>,
    /// ⑰ short ×3 → obj+0x21c/+0x220/+0x21e（版本 ≥3；样本 0/0/0），语义
    /// 未定。前置停止/版本不足时 `None`。
    pub shorts_21c_220_21e: Option<[i16; 3]>,
    /// ⑱ int32 → obj+0x22c（版本 ≥0xc；样本 0），语义未定。
    pub int32_22c: Option<i32>,
    /// ⑲ IName 槽分类（版本 ≥4）。前置停止/版本不足时 `None`。
    pub name: Option<RefSlot10_1>,
    /// ⑳ 关系类循环声明计数（版本 >6；样本 0）。负数原生不进循环。
    pub relationship_class_count: Option<i32>,
    /// ⑳ 已消费的关系类槽位条目。
    pub relationship_classes: Vec<RefSlot10_1>,
    /// ㉑ String → obj+0x18c（版本 ≥8；样本 cb=2 空串），语义未定。
    pub string_18c: Option<String>,
    /// ㉒ IEnvelope 槽分类（版本 >8；样本空标记）。
    pub envelope_1: Option<RefSlot10_1>,
    /// ㉓ 范围计数（版本 >9；样本 0）。负数原生不进循环。
    pub range_count: Option<i32>,
    /// ㉓ 已消费的范围 int32（逐元素读）。
    pub range_ids: Vec<i32>,
    /// ㉔ 图层扩展段声明计数（版本 ≥0xb；样本 1）。负数原生不进循环。
    pub extension_count: Option<i32>,
    /// ㉔ 已消费的扩展条目（按流内顺序）。
    pub extensions: Vec<LayerExtension10_1>,
    /// ㉕ double → obj+0x1e8（版本 ≥0xd；样本未达），语义未定。
    pub double_1e8: Option<f64>,
    /// ㉖ ISymbol 槽（请求 IID `{F3435802-…}`）分类（版本 >0xd）。
    pub symbol: Option<SymbolSlot10_1>,
    /// ㉗ short → obj+0x12e（版本 >0xe），语义未定。
    pub short_12e: Option<i16>,
    /// ㉗ int32 → obj+0x1f0（版本 >0xe），语义未定。
    pub int32_1f0: Option<i32>,
    /// ㉘ IEnvelope 槽 2 分类（版本 >0xe）。
    pub envelope_2: Option<RefSlot10_1>,
    /// ㉙ IUID 槽分类（版本 >0x10）。
    pub uid: Option<RefSlot10_1>,
    /// ㉚ String → obj+0x28c（版本 ≥0x12），语义未定。
    pub string_28c: Option<String>,
    /// ㉛ short → obj+0x1d8（版本 ≥0x13），语义未定。
    pub short_1d8: Option<i16>,
    /// ㉜ double → obj+0x290（版本 ≥0x14），语义未定。
    pub double_290: Option<f64>,
    /// ㉜ double → obj+0x298（版本 ≥0x14），语义未定。
    pub double_298: Option<f64>,
    /// ㉝ short → obj+0x1da（版本 ≥0x15），语义未定。
    pub short_1da: Option<i16>,
    /// ㉞ int32 → obj+0x244（版本 ≥0x16），语义未定。
    pub int32_244: Option<i32>,
    /// ㉟ short → obj+0x1be（版本 ≥0x17；样本未达），语义未定。
    pub short_1be: Option<i16>,
    /// ㊱ PreJoinDefExpressions 对（样本未达）。
    pub pre_join_pairs: Vec<(String, String)>,
    /// ㊲ int32 → obj+0x2c0（版本 ≥0x19），语义未定。
    pub int32_2c0: Option<i32>,
    /// ㊲ short ×3 → obj+0x2c4/+0x2c6/+0x2c8（版本 ≥0x19），语义未定。
    pub shorts_2c4_2c6_2c8: Option<[i16; 3]>,
    /// ㊳ String ×4 → obj+0x2cc/+0x2d0/+0x2d4/+0x2d8（版本 ≥0x19），语义
    /// 未定。
    pub strings_2cc_2d0_2d4_2d8: Option<[String; 4]>,
    /// ㊴ {A62B9EC2} 槽分类（版本 >0x19）。
    pub expression_props: Option<RefSlot10_1>,
    /// ㊵ short → obj+0x300（版本 >0x1a），语义未定。
    pub short_300: Option<i16>,
    /// ㊵ String ×3 → obj+0x304/+0x308/+0x30c（版本 >0x1a），语义未定。
    pub strings_304_308_30c: Option<[String; 3]>,
    /// ㊶ 版本 0x1c..0x1f 分支标量块（short + String ×4 + short + double +
    /// int32；10.1 不写，无样本佐证；本轮静态闭合）。仅该版本区间读取。
    pub legacy_scalars: Option<LegacyTailScalars10_1>,
    /// ㊶ TimeData 段（版本 ≥0x20；样本未达），本轮闭合。
    pub time_data: Option<TimeDataBody10_1>,
    /// ㊶ TimeTableDefinition 段（版本 ≥0x20）。
    pub time_table_definition: Option<TimeTableDefinitionBody10_1>,
    /// ㊶ TimeDimensionDefinition 段（版本 ≥0x20）。
    pub time_dimension_definition: Option<TimeDimensionDefinitionBody10_1>,
    /// ㊶ TimeDataDisplay 段（版本 ≥0x20）。
    pub time_data_display: Option<TimeDataDisplayBody10_1>,
    /// ㊷ {E720618B} 槽分类（版本 >0x20；10.1 恒写 34 → 不写）。
    pub hot_link_expression_props: Option<RefSlot10_1>,
    /// ㊸ short → obj+0x2ca（版本 >0x21；样本未达），语义未定。
    pub short_2ca: Option<i16>,
    /// 停止原因（恒有：本读取器始终在最深可闭合点停止）。
    pub stop: SuccessorStop10_1,
    /// 停止点之后剩余的字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 通用「仅空标记/回引可消费」槽位的分类结果（第九轮延续段的标量段之间的
/// LoadObject 槽）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// ⑪ ALPC 槽（请求 IID `{1D5849F2-…}`）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotatePropsSlot10_1 {
    /// 空标记（原生 LoadObject 对空引用返回空集合候选）。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 AnnotateLayerPropertiesCollection 对象：已解析体（体内停止经
    /// `body.stop` 传播为 [`SuccessorStop10_1::AnnotatePropsBody`]）。
    Collection(Box<AnnotatePropsBody10_1>),
}

/// ⑬ FeatureIDSet 槽（请求 IID `{D5BB4B87-…}`）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureIdSetSlot10_1 {
    /// 空标记。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 FeatureIDSet 对象：已解析体（无停止点）。
    FeatureIdSet(Box<FeatureIdSetBody10_1>),
}

/// ㉔ 图层扩展段（LayerLib.dll `ExtensionHelper::LoadExtensions`）的条目。
#[derive(Debug, Clone, PartialEq)]
pub enum LayerExtension10_1 {
    /// EditTemplateManager（`{A0162E85-…}`）：已解析体（EditTemplate 宿主；
    /// 体内停止经 `body.stop` 以
    /// [`SuccessorStop10_1::EditTemplateManagerBody`] 传播）。
    EditTemplateManager(Box<EditTemplateManagerBody10_1>),
}

/// 版本 0x1c..0x1f 分支的标量块（10.1 docVersion 不写；本轮静态闭合，无样
/// 本字节佐证）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTailScalars10_1 {
    /// short（→ -0x62），语义未定。
    pub short_62: i16,
    /// String ×4（→ -0x78/-0x74/-0x70/-0x6c），语义未定。
    pub strings: [String; 4],
    /// short（→ -0x44），语义未定。
    pub short_44: i16,
    /// double（→ -0x40），语义未定。
    pub double_40: f64,
    /// int32（→ -0x38），语义未定。
    pub int32_38: i32,
}

/// TimeData 段（LayerLib.dll `0x7870d8b0`，版本 ≤2）的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDataBody10_1 {
    /// 体首 short 版本（有符号 >2 拒绝；负值位模式一并拒绝，较原生更严）。
    pub version: u16,
    /// short（→ this+6），语义未定。
    pub short_a: i16,
    /// LoadObject 槽（请求 IID `DAT_78721e70`；新对象 = 停止点）。
    pub slot: RefSlot10_1,
    /// 版本 ≥2 的 short（→ this+8），语义未定。
    pub tail_short: Option<i16>,
}

/// TimeTableDefinition 段（LayerLib.dll `0x7870df80`，版本 ≤1）的已确认
/// 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeTableDefinitionBody10_1 {
    /// 体首 short 版本（有符号 >1 拒绝；负值位模式一并拒绝）。
    pub version: u16,
    /// String ×4（→ this+8/+0xc/+0x10/+0x14；时间字段名候选），语义未定。
    pub strings: [String; 4],
    /// LoadObject 槽（请求 IID `DAT_78721e80`；新对象 = 停止点）。
    pub slot: RefSlot10_1,
}

/// TimeDimensionDefinition 段（LayerLib.dll `0x7870e250`，版本 ≤1）的已
/// 确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDimensionDefinitionBody10_1 {
    /// 体首 short 版本（有符号 >1 拒绝；负值位模式一并拒绝）。
    pub version: u16,
    /// String ×2（→ this+4/+8；时间维度名/格式候选），语义未定。
    pub strings: [String; 2],
}

/// TimeDataDisplay 段（LayerLib.dll `0x7870f6c0`，版本 ≤1）的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDataDisplayBody10_1 {
    /// 体首 short 版本（有符号 >1 拒绝；负值位模式一并拒绝）。
    pub version: u16,
    /// short（→ this+4），语义未定。
    pub short_a: i16,
    /// double（→ this+8），语义未定。
    pub double_a: f64,
    /// int32（→ this+0x10），语义未定。
    pub int_a: i32,
    /// double（→ this+0x18），语义未定。
    pub double_b: f64,
    /// int32（→ this+0x20），语义未定。
    pub int_b: i32,
}

/// 从渲染器槽 +0x144 原样 GUID 之后的位置读取 10.1 FeatureLayer 后继字段
/// 段，失败时不推进游标、不修改登记表。
///
/// 起点约定：调用者已读取 +0x144 的 16 字节 GUID（`fl_version` 为
/// FeatureLayer 体首的架构版本，用于 FieldInfo 条目名的版本 >5 门）。
/// IHyperlink 循环与 FieldInfo 循环的新对象、IColor 槽非 RgbColor 新对象、
/// FieldInfo 体内未知 NumericFormat 族对象均为成功停止（游标回退到该条目/
/// 槽位起点，该键不保留；FieldInfo 体内停止传播为
/// [`SuccessorStop10_1::FieldInfoBody`]，游标停在内层停止点）。截断、版本
/// 超限、字符串超限为错误（游标与登记表不动）。
pub fn read_feature_layer_successor_10_1(
    reader: &mut Reader<'_>,
    fl_version: u16,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<FeatureLayerSuccessor10_1> {
    let mut trial = reader.clone();

    // ①-④ 标量段：String +0x154、double ×2、short +0x168。
    let string_154 = read_string_10_1(&mut trial, max_bytes)?;
    let double_158 = trial.read_f64_le()?;
    let double_160 = trial.read_f64_le()?;
    let short_168 = trial.read_u16_le()? as i16;

    // 登记表副本：循环与槽位登记先发生在副本上，整体成功才随外层提交。
    let mut staged_table = table.clone();

    // ⑤ IHyperlink 计数循环：新对象为停止点（Hyperlink 族体未闭合）。
    let hyperlink_count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let mut hyperlinks = Vec::new();
    let mut stop = None;
    for index in 0..hyperlink_count.max(0) as usize {
        let item_offset = trial.position();
        let pre_item_table = staged_table.clone();
        match read_object_entry_10_1(&mut trial, &mut staged_table)? {
            ObjectEntry10_1::Null => hyperlinks.push(HyperlinkSlot10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                hyperlinks.push(HyperlinkSlot10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                stop = Some(SuccessorStop10_1::HyperlinkNewObject {
                    class_id,
                    key,
                    index,
                });
                trial = reposition_to(reader, item_offset);
                staged_table = pre_item_table;
            }
        }
        if stop.is_some() {
            break;
        }
    }

    // ⑥ IColor 槽：五类闭合颜色经对象层颜色槽位分派整体解析；未知颜色类
    //    为停止点（分派内部回退到槽位起点、登记表不保留该键）。
    let mut color = ColorSlot10_1::Null;
    if stop.is_none() {
        match read_color_slot_10_1(&mut trial, max_bytes, &mut staged_table)? {
            ColorSlot10_1::Unsupported { class_id, key } => {
                stop = Some(SuccessorStop10_1::ColorUnsupported { class_id, key });
                color = ColorSlot10_1::Null;
            }
            other => color = other,
        }
    }

    // ⑦-⑨ 字符串段与 int32 +0x188（前置停止时不读取）。
    let mut string_1a8 = String::new();
    let mut string_184 = String::new();
    let mut int32_188 = 0;
    let mut field_info_count = 0;
    let mut field_infos = Vec::new();
    if stop.is_none() {
        string_1a8 = read_string_10_1(&mut trial, max_bytes)?;
        string_184 = read_string_10_1(&mut trial, max_bytes)?;
        int32_188 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

        // ⑩ FieldInfo 计数循环：条目 = [（版本 >5）String 名 + FieldInfo
        //    对象]。原生架构版本 ≤5 的名字从要素类模式接口推导，流内无名字
        //    段（汇编 `0x6f6e9c91` 门）。
        field_info_count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        for index in 0..field_info_count.max(0) as usize {
            let name = if fl_version > 5 {
                Some(read_string_10_1(&mut trial, max_bytes)?)
            } else {
                None
            };
            let item_offset = trial.position();
            let pre_item_table = staged_table.clone();
            let slot = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => FieldInfoEntry10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    FieldInfoEntry10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { class_id, key } if class_id == FIELD_INFO_CLASS_ID => {
                    let body =
                        read_field_info_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                    if let Some(inner) = body.stop {
                        stop = Some(SuccessorStop10_1::FieldInfoBody(inner));
                    }
                    FieldInfoEntry10_1::FieldInfo(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::FieldInfoEntryUnsupported {
                        class_id,
                        key,
                        index,
                    });
                    trial = reposition_to(reader, item_offset);
                    staged_table = pre_item_table;
                    FieldInfoEntry10_1::Null
                }
            };
            field_infos.push(FieldInfoLoopEntry10_1 { name, slot });
            if stop.is_some() {
                break;
            }
        }
    }
    // ⑪-㊸ FieldInfo 循环之后的延续段（第九轮闭合）。流已耗尽（ALPC 槽
    //    对象头 16 字节无法完整读取）时保留第八轮「循环完成」停止语义；
    //    对象头可读后的中途截断一律报错（整体回退，与同读取器前段语义
    //    一致）。
    let mut annotate_props = None;
    let mut short_1bc = None;
    let mut feature_id_set = None;
    let mut selection_id_count = None;
    let mut selection_ids = Vec::new();
    let mut short_1fe = None;
    let mut display_filter = None;
    let mut shorts_21c_220_21e = None;
    let mut int32_22c = None;
    let mut name = None;
    let mut relationship_class_count = None;
    let mut relationship_classes = Vec::new();
    let mut string_18c = None;
    let mut envelope_1 = None;
    let mut range_count = None;
    let mut range_ids = Vec::new();
    let mut extension_count = None;
    let mut extensions = Vec::new();
    let mut double_1e8 = None;
    let mut symbol = None;
    let mut short_12e = None;
    let mut int32_1f0 = None;
    let mut envelope_2 = None;
    let mut uid = None;
    let mut string_28c = None;
    let mut short_1d8 = None;
    let mut double_290 = None;
    let mut double_298 = None;
    let mut short_1da = None;
    let mut int32_244 = None;
    let mut short_1be = None;
    let mut pre_join_pairs = Vec::new();
    let mut int32_2c0 = None;
    let mut shorts_2c4_2c6_2c8 = None;
    let mut strings_2cc_2d0_2d4_2d8 = None;
    let mut expression_props = None;
    let mut short_300 = None;
    let mut strings_304_308_30c = None;
    let mut legacy_scalars = None;
    let mut time_data = None;
    let mut time_table_definition = None;
    let mut time_dimension_definition = None;
    let mut time_data_display = None;
    let mut hot_link_expression_props = None;
    let mut short_2ca = None;
    if stop.is_none() {
        if trial.remaining() < 16 {
            // 流在此结束：ALPC 槽对象头不可读，保留「循环完成」停止。
            stop = Some(SuccessorStop10_1::FieldInfoLoopCompleted);
        } else {
            // ⑪ ALPC 槽（请求 IID {1D5849F2-…}）：新对象仅支持
            //    AnnotateLayerPropertiesCollection。
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => {
                    annotate_props = Some(AnnotatePropsSlot10_1::Null);
                }
                ObjectEntry10_1::BackReference { class_id, key } => {
                    annotate_props = Some(AnnotatePropsSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == ANNOTATE_PROPS_COLLECTION_CLASS_ID =>
                {
                    let body = read_annotate_props_collection_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged_table,
                        key,
                    )?;
                    if let Some(inner) = body.stop.clone() {
                        stop = Some(SuccessorStop10_1::AnnotatePropsBody(inner));
                    }
                    annotate_props = Some(AnnotatePropsSlot10_1::Collection(Box::new(body)));
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    // ALPC 槽头不是 AnnotateLayerPropertiesCollection：游标
                    // 回退到槽位起点（该键不保留），并按第八轮「FieldInfo
                    // 循环完成、其后对象未闭合」语义停止（停止点相同，仅
                    // 停止原因不区分「对象流后续内容」与「未知标注类」，
                    // 与既有调用方断言兼容）。
                    stop = Some(SuccessorStop10_1::FieldInfoLoopCompleted);
                    let _ = (class_id, key);
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    annotate_props = Some(AnnotatePropsSlot10_1::Null);
                }
            }

            // ⑫ short +0x1bc（前置停止时不读取）。
            if stop.is_none() {
                short_1bc = Some(trial.read_u16_le()? as i16);

                // ⑬ FeatureIDSet 槽（请求 IID {D5BB4B87-…}）。
                let slot_offset = trial.position();
                let pre_slot_table = staged_table.clone();
                match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                    ObjectEntry10_1::Null => {
                        feature_id_set = Some(FeatureIdSetSlot10_1::Null);
                    }
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        feature_id_set =
                            Some(FeatureIdSetSlot10_1::BackReference { class_id, key });
                    }
                    ObjectEntry10_1::NewObject { class_id, key }
                        if class_id == FEATURE_ID_SET_CLASS_ID =>
                    {
                        let body =
                            read_feature_id_set_body_10_1(&mut trial, &mut staged_table, key)?;
                        feature_id_set = Some(FeatureIdSetSlot10_1::FeatureIdSet(Box::new(body)));
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        stop = Some(SuccessorStop10_1::FeatureIdSetUnsupported { class_id, key });
                        trial = reposition_to(reader, slot_offset);
                        staged_table = pre_slot_table;
                        feature_id_set = Some(FeatureIdSetSlot10_1::Null);
                    }
                }
            }
        }
    }

    // ⑭-㊸ 后续段（ALPC 槽完整消费且无前置停止时才继续；此处之后的中途
    //    截断一律报错——对象流一旦进入本段即长度确定）。
    if stop.is_none() && short_1bc.is_some() {
        // ⑭ 选择集 ID：int32 计数 + count × int32（原生单次整块读；
        //    负计数原生跳过，原样保留）。
        let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        selection_id_count = Some(count);
        if count > 0 {
            let total = usize::try_from(count)
                .ok()
                .and_then(|c| c.checked_mul(4))
                .ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "选择集 ID 计数 {count} × 4 溢出（offset 0x{:x}）",
                            trial.position()
                        ),
                    )
                })?;
            let raw = trial.read_bytes(total)?;
            for chunk in raw.chunks_exact(4) {
                selection_ids.push(i32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }

        // ⑮ short +0x1fe。
        short_1fe = Some(trial.read_u16_le()? as i16);

        // ⑯ IDisplayFilter 槽（请求 IID {19F85377-…}，版本 ≥2）。此前
        //    「EditTemplate 宿主」假设撤销：样本该槽为空标记；新对象类为
        //    停止点。
        if (fl_version as i16) >= 2 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => display_filter = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    display_filter = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::DisplayFilterUnsupported { class_id, key });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    display_filter = Some(RefSlot10_1::Null);
                }
            }
        }

        // ⑰-⑲ 版本门标量段与 IName 槽。
        if stop.is_none() && (fl_version as i16) >= 3 {
            let a = trial.read_u16_le()? as i16;
            let b = trial.read_u16_le()? as i16;
            let c = trial.read_u16_le()? as i16;
            shorts_21c_220_21e = Some([a, b, c]);
        }
        if stop.is_none() && (fl_version as i16) >= 0xc {
            int32_22c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        if stop.is_none() && (fl_version as i16) >= 4 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => name = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    name = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::NameUnsupported { class_id, key });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    name = Some(RefSlot10_1::Null);
                }
            }
        }

        // ⑳ 关系类计数 + 循环（版本 >6；新对象 = 停止点）。
        if stop.is_none() && (fl_version as i16) > 6 {
            let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            relationship_class_count = Some(count);
            for index in 0..count.max(0) as usize {
                let slot_offset = trial.position();
                let pre_slot_table = staged_table.clone();
                match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                    ObjectEntry10_1::Null => relationship_classes.push(RefSlot10_1::Null),
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        relationship_classes.push(RefSlot10_1::BackReference { class_id, key });
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        stop = Some(SuccessorStop10_1::RelationshipClassUnsupported {
                            class_id,
                            key,
                            index,
                        });
                        trial = reposition_to(reader, slot_offset);
                        staged_table = pre_slot_table;
                        break;
                    }
                }
                if stop.is_some() {
                    break;
                }
            }
        }

        // ㉑ String +0x18c（版本 ≥8；写侧恒写，10.1 与读侧门一致）。
        if stop.is_none() && (fl_version as i16) >= 8 {
            string_18c = Some(read_string_10_1(&mut trial, max_bytes)?);
        }

        // ㉒ IEnvelope 槽（版本 >8；Envelope 体未在 src/ 闭合，新对象 =
        //    停止点；样本空标记）。
        if stop.is_none() && (fl_version as i16) > 8 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => envelope_1 = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    envelope_1 = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::EnvelopeUnsupported {
                        class_id,
                        key,
                        which: 0,
                    });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    envelope_1 = Some(RefSlot10_1::Null);
                }
            }
        }

        // ㉓ 范围计数 + 逐元素 int32（版本 >9；负计数原生跳过）。
        if stop.is_none() && (fl_version as i16) > 9 {
            let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            range_count = Some(count);
            for _ in 0..count.max(0) as usize {
                range_ids.push(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
            }
        }

        // ㉔ 图层扩展段（版本 ≥0xb；LayerLib.dll ExtensionHelper::
        //    LoadExtensions 格式）。EditTemplateManager 体本轮闭合
        //    （EditTemplate 宿主）；其余类为停止点（回退到该条目 [int64
        //    尺寸] 字段起点，该条目整体不消费）。
        if stop.is_none() && (fl_version as i16) >= 0xb {
            let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            extension_count = Some(count);
            for _ in 0..count.max(0) as usize {
                let entry_offset = trial.position();
                let pre_entry_table = staged_table.clone();
                let size = i64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap());
                let header = read_object_entry_10_1(&mut trial, &mut staged_table)?;
                match header {
                    ObjectEntry10_1::Null => {
                        // 尺寸声明之后紧跟 16 字节全零类标识：原生按
                        // CLSID 语义读，此处不可能出现空标记布局；按未
                        // 闭合停止处理（防静默错位）。
                        stop = Some(SuccessorStop10_1::LayerExtensionUnsupported {
                            class_id: [0u8; 16],
                            size,
                        });
                        trial = reposition_to(reader, entry_offset);
                        staged_table = pre_entry_table;
                    }
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        // 回引条目无体：原生扩展段每条都带完整对象头+
                        // 体，回引布局与 ExtensionHelper 实现不符，按未
                        // 闭合停止。
                        stop =
                            Some(SuccessorStop10_1::LayerExtensionUnsupported { class_id, size });
                        let _ = key;
                        trial = reposition_to(reader, entry_offset);
                        staged_table = pre_entry_table;
                    }
                    ObjectEntry10_1::NewObject { class_id, key }
                        if class_id == EDIT_TEMPLATE_MANAGER_CLASS_ID =>
                    {
                        // 尺寸字段语义（样本扩展条目：声明 387，头+体恰为
                        // 387 字节）：覆盖 [类标识][键][体] 整段，不含其前
                        // 的 [int64 尺寸] 字段本身。object_start 从尺寸字段
                        // 之后（对象头之前）起算。
                        let object_start = entry_offset + 8;
                        let body = read_edit_template_manager_body_10_1(
                            &mut trial,
                            max_bytes,
                            &mut staged_table,
                            key,
                        )?;
                        let consumed = trial.position() - object_start;
                        if consumed != usize::try_from(size).unwrap_or(usize::MAX) {
                            // 尺寸字段与实际消费不一致：原生不校验（已
                            // 知类不使用尺寸），显式报错较严（防静默
                            // 错位）；错误整体回退。
                            return Err(io::Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "EditTemplateManager 扩展条目 offset 0x{entry_offset:x}: \
                                     声明尺寸 {size} 与实际消费 {consumed} 不一致（原生不校验，\
                                     本读取器显式拒绝）"
                                ),
                            ));
                        }
                        extensions.push(LayerExtension10_1::EditTemplateManager(Box::new(body)));
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        stop =
                            Some(SuccessorStop10_1::LayerExtensionUnsupported { class_id, size });
                        let _ = key;
                        trial = reposition_to(reader, entry_offset);
                        staged_table = pre_entry_table;
                    }
                }
                if stop.is_some() {
                    break;
                }
            }
        }

        // ㉕ double +0x1e8（版本 ≥0xd）。
        if stop.is_none() && (fl_version as i16) >= 0xd {
            double_1e8 = Some(trial.read_f64_le()?);
        }

        // ㉖ ISymbol 槽（请求 IID {F3435802-…}，版本 >0xd）：符号族分派，
        //    未知类为停止点（分派内部已回退）。
        if stop.is_none() && (fl_version as i16) > 0xd {
            match read_display_symbol_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged_table,
                MAX_SYMBOL_NESTING_DEPTH,
            )? {
                SymbolSlot10_1::Unsupported { class_id, key } => {
                    stop = Some(SuccessorStop10_1::SymbolSlotUnsupported { class_id, key });
                    symbol = Some(SymbolSlot10_1::Null);
                }
                other => symbol = Some(other),
            }
        }

        // ㉗-㉙ 版本 >0xe：short +0x12e、int32 +0x1f0、IEnvelope 槽 2。
        if stop.is_none() && (fl_version as i16) > 0xe {
            short_12e = Some(trial.read_u16_le()? as i16);
            int32_1f0 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => envelope_2 = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    envelope_2 = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::EnvelopeUnsupported {
                        class_id,
                        key,
                        which: 1,
                    });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    envelope_2 = Some(RefSlot10_1::Null);
                }
            }
        }

        // ㉙ IUID 槽（请求 IID {1714D59B-…}，版本 >0x10）。
        if stop.is_none() && (fl_version as i16) > 0x10 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => uid = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    uid = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::UidUnsupported { class_id, key });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    uid = Some(RefSlot10_1::Null);
                }
            }
        }

        // ㉚-㊱ 版本门标量段（≥0x12 / ==0x12||0x13 / ≥0x13 / ≥0x14 / ≥0x15 /
        //    ≥0x16 / ≥0x17）。
        if stop.is_none() && (fl_version as i16) >= 0x12 {
            string_28c = Some(read_string_10_1(&mut trial, max_bytes)?);
            if (fl_version as i16) == 0x12 || (fl_version as i16) == 0x13 {
                // 原生读 2 字节入临时（丢弃）。
                let _ = trial.read_u16_le()?;
            }
            if (fl_version as i16) >= 0x13 {
                short_1d8 = Some(trial.read_u16_le()? as i16);
            }
            if (fl_version as i16) >= 0x14 {
                double_290 = Some(trial.read_f64_le()?);
                double_298 = Some(trial.read_f64_le()?);
            }
            if (fl_version as i16) >= 0x15 {
                short_1da = Some(trial.read_u16_le()? as i16);
            }
            if (fl_version as i16) >= 0x16 {
                int32_244 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
            }
            if (fl_version as i16) >= 0x17 {
                short_1be = Some(trial.read_u16_le()? as i16);
            }
        }

        // ㊱ PreJoinDefExpressions（版本 >0x17；LayerLib.dll
        //    `0x786c5d30`：int32 计数 + count × [String + String]）。
        if stop.is_none() && (fl_version as i16) > 0x17 {
            let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            if count > 0 {
                for _ in 0..count as usize {
                    let first = read_string_10_1(&mut trial, max_bytes)?;
                    let second = read_string_10_1(&mut trial, max_bytes)?;
                    pre_join_pairs.push((first, second));
                }
            }
        }

        // ㊲㊳ 版本 ≥0x19 标量块。
        if stop.is_none() && (fl_version as i16) >= 0x19 {
            int32_2c0 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
            let a = trial.read_u16_le()? as i16;
            let b = trial.read_u16_le()? as i16;
            let c = trial.read_u16_le()? as i16;
            shorts_2c4_2c6_2c8 = Some([a, b, c]);
            let s1 = read_string_10_1(&mut trial, max_bytes)?;
            let s2 = read_string_10_1(&mut trial, max_bytes)?;
            let s3 = read_string_10_1(&mut trial, max_bytes)?;
            let s4 = read_string_10_1(&mut trial, max_bytes)?;
            strings_2cc_2d0_2d4_2d8 = Some([s1, s2, s3, s4]);
        }

        // ㊴ {A62B9EC2} 槽（版本 >0x19；新对象 = 停止点）。
        if stop.is_none() && (fl_version as i16) > 0x19 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => expression_props = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    expression_props = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::ExpressionPropsUnsupported { class_id, key });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    expression_props = Some(RefSlot10_1::Null);
                }
            }
        }

        // ㊵ 版本 >0x1a：short +0x300 + String ×3。
        if stop.is_none() && (fl_version as i16) > 0x1a {
            short_300 = Some(trial.read_u16_le()? as i16);
            let s1 = read_string_10_1(&mut trial, max_bytes)?;
            let s2 = read_string_10_1(&mut trial, max_bytes)?;
            let s3 = read_string_10_1(&mut trial, max_bytes)?;
            strings_304_308_30c = Some([s1, s2, s3]);
        }

        // ㊶ 版本 0x1c..0x1f 分支（10.1 docVersion 不写；本轮静态闭合，
        //    无样本字节佐证）：标量块 + 三个时间族槽（新对象 = 停止点）。
        if stop.is_none() && (fl_version as i16) >= 0x1c && (fl_version as i16) < 0x20 {
            let legacy_short = trial.read_u16_le()? as i16;
            let l1 = read_string_10_1(&mut trial, max_bytes)?;
            let l2 = read_string_10_1(&mut trial, max_bytes)?;
            let l3 = read_string_10_1(&mut trial, max_bytes)?;
            let l4 = read_string_10_1(&mut trial, max_bytes)?;
            let legacy_short2 = trial.read_u16_le()? as i16;
            let legacy_double = trial.read_f64_le()?;
            let legacy_int = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            legacy_scalars = Some(LegacyTailScalars10_1 {
                short_62: legacy_short,
                strings: [l1, l2, l3, l4],
                short_44: legacy_short2,
                double_40: legacy_double,
                int32_38: legacy_int,
            });
            for which in 0u8..3 {
                let slot_offset = trial.position();
                let pre_slot_table = staged_table.clone();
                match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                    ObjectEntry10_1::Null => {}
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        let _ = (class_id, key);
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        stop = Some(SuccessorStop10_1::LegacyTimeSlotUnsupported {
                            class_id,
                            key,
                            which,
                        });
                        trial = reposition_to(reader, slot_offset);
                        staged_table = pre_slot_table;
                        break;
                    }
                }
                if stop.is_some() {
                    break;
                }
            }
        }

        // ㊶ TimeData 族四段（版本 ≥0x20；LayerLib.dll `0x7870d8b0` 等，
        //    本轮闭合；槽位新对象 = 停止点）。
        if stop.is_none() && (fl_version as i16) >= 0x20 {
            let entry_offset = trial.position();
            let version = trial.read_u16_le()?;
            let signed = version as i16;
            if signed > 2 || signed < 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "TimeData 对象段 offset 0x{entry_offset:x}: 版本 {version} 超出 \
                         TimeData::LoadTimeData 接受范围（有符号比较 >2 拒绝；负值位模式\
                         一并拒绝）"
                    ),
                ));
            }
            let short_a = trial.read_u16_le()? as i16;
            let mut time_slot = RefSlot10_1::Null;
            {
                let slot_offset = trial.position();
                let pre_slot_table = staged_table.clone();
                match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                    ObjectEntry10_1::Null => {}
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        time_slot = RefSlot10_1::BackReference { class_id, key };
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        stop = Some(SuccessorStop10_1::TimeSlotUnsupported {
                            class_id,
                            key,
                            which: 0,
                        });
                        trial = reposition_to(reader, slot_offset);
                        staged_table = pre_slot_table;
                    }
                }
            }
            let mut tail_short = None;
            if stop.is_none() && signed >= 2 {
                tail_short = Some(trial.read_u16_le()? as i16);
            }
            if stop.is_none() {
                time_data = Some(TimeDataBody10_1 {
                    version,
                    short_a,
                    slot: time_slot,
                    tail_short,
                });
            }

            // TimeTableDefinition（版本 ≤1；String ×4 + 槽）。
            if stop.is_none() {
                let entry_offset = trial.position();
                let version = trial.read_u16_le()?;
                let signed = version as i16;
                if signed > 1 || signed < 0 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "TimeTableDefinition 对象段 offset 0x{entry_offset:x}: 版本 {version} \
                             超出 TimeTableDefinition::LoadTimeTableDefinition 接受范围\
                             （有符号比较 >1 拒绝；负值位模式一并拒绝）"
                        ),
                    ));
                }
                let s1 = read_string_10_1(&mut trial, max_bytes)?;
                let s2 = read_string_10_1(&mut trial, max_bytes)?;
                let s3 = read_string_10_1(&mut trial, max_bytes)?;
                let s4 = read_string_10_1(&mut trial, max_bytes)?;
                let mut slot = RefSlot10_1::Null;
                let slot_offset = trial.position();
                let pre_slot_table = staged_table.clone();
                match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                    ObjectEntry10_1::Null => {}
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        slot = RefSlot10_1::BackReference { class_id, key };
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        stop = Some(SuccessorStop10_1::TimeSlotUnsupported {
                            class_id,
                            key,
                            which: 1,
                        });
                        trial = reposition_to(reader, slot_offset);
                        staged_table = pre_slot_table;
                    }
                }
                if stop.is_none() {
                    time_table_definition = Some(TimeTableDefinitionBody10_1 {
                        version,
                        strings: [s1, s2, s3, s4],
                        slot,
                    });
                }
            }

            // TimeDimensionDefinition（版本 ≤1；String ×2）。
            if stop.is_none() {
                let entry_offset = trial.position();
                let version = trial.read_u16_le()?;
                let signed = version as i16;
                if signed > 1 || signed < 0 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "TimeDimensionDefinition 对象段 offset 0x{entry_offset:x}: 版本 \
                             {version} 超出 TimeDimensionDefinition::LoadTimeDimensionDefinition \
                             接受范围（有符号比较 >1 拒绝；负值位模式一并拒绝）"
                        ),
                    ));
                }
                let s1 = read_string_10_1(&mut trial, max_bytes)?;
                let s2 = read_string_10_1(&mut trial, max_bytes)?;
                if stop.is_none() {
                    time_dimension_definition = Some(TimeDimensionDefinitionBody10_1 {
                        version,
                        strings: [s1, s2],
                    });
                }
            }

            // TimeDataDisplay（版本 ≤1；short + double + int32 + double +
            // int32）。
            if stop.is_none() {
                let entry_offset = trial.position();
                let version = trial.read_u16_le()?;
                let signed = version as i16;
                if signed > 1 || signed < 0 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "TimeDataDisplay 对象段 offset 0x{entry_offset:x}: 版本 {version} \
                             超出 TimeDataDisplay::LoadTimeDataDisplay 接受范围（有符号比较 \
                             >1 拒绝；负值位模式一并拒绝）"
                        ),
                    ));
                }
                let short_a = trial.read_u16_le()? as i16;
                let double_a = trial.read_f64_le()?;
                let int_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
                let double_b = trial.read_f64_le()?;
                let int_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
                if stop.is_none() {
                    time_data_display = Some(TimeDataDisplayBody10_1 {
                        version,
                        short_a,
                        double_a,
                        int_a,
                        double_b,
                        int_b,
                    });
                }
            }
        }

        // ㊷ {E720618B} 槽（版本 >0x20；10.1 恒写 34 → 不达；新对象 = 停止
        //    点）。注意：读侧 ==0x1f 的同名槽属 0x1c..0x1f 分支（见上）。
        if stop.is_none() && (fl_version as i16) > 0x20 {
            let slot_offset = trial.position();
            let pre_slot_table = staged_table.clone();
            match read_object_entry_10_1(&mut trial, &mut staged_table)? {
                ObjectEntry10_1::Null => hot_link_expression_props = Some(RefSlot10_1::Null),
                ObjectEntry10_1::BackReference { class_id, key } => {
                    hot_link_expression_props = Some(RefSlot10_1::BackReference { class_id, key });
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    stop = Some(SuccessorStop10_1::ExpressionPropsUnsupported { class_id, key });
                    trial = reposition_to(reader, slot_offset);
                    staged_table = pre_slot_table;
                    hot_link_expression_props = Some(RefSlot10_1::Null);
                }
            }
        }

        // ㊸ short +0x2ca（版本 >0x21；样本 34 → 不达）。
        if stop.is_none() && (fl_version as i16) > 0x21 {
            short_2ca = Some(trial.read_u16_le()? as i16);
        }
    }
    let stop = stop.unwrap_or(SuccessorStop10_1::TailCompleted);

    let remaining_bytes = trial.remaining();
    let result = FeatureLayerSuccessor10_1 {
        string_154,
        double_158,
        double_160,
        short_168,
        hyperlink_count,
        hyperlinks,
        color,
        string_1a8,
        string_184,
        int32_188,
        field_info_count,
        field_infos,
        annotate_props,
        short_1bc,
        feature_id_set,
        selection_id_count,
        selection_ids,
        short_1fe,
        display_filter,
        shorts_21c_220_21e,
        int32_22c,
        name,
        relationship_class_count,
        relationship_classes,
        string_18c,
        envelope_1,
        range_count,
        range_ids,
        extension_count,
        extensions,
        double_1e8,
        symbol,
        short_12e,
        int32_1f0,
        envelope_2,
        uid,
        string_28c,
        short_1d8,
        double_290,
        double_298,
        short_1da,
        int32_244,
        short_1be,
        pre_join_pairs,
        int32_2c0,
        shorts_2c4_2c6_2c8,
        strings_2cc_2d0_2d4_2d8,
        expression_props,
        short_300,
        strings_304_308_30c,
        legacy_scalars,
        time_data,
        time_table_definition,
        time_dimension_definition,
        time_data_display,
        hot_link_expression_props,
        short_2ca,
        stop,
        remaining_bytes,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}
