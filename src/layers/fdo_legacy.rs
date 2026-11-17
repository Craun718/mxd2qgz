//! ArcMap 10.1 FDOGraphicsLayer（FDO 注记图层）开放点的读取器：版本 <4
//! 旧加载器前缀与 FDO 元素循环类（FDOGraphicsSublayer）体。
//!
//! 证据来源：`docs/arcmap-10.1-fdo-open-points.md`（第七轮，AnnoLayer.dll
//! 10.1.0.3035，cab 条目 `AnnoLayer.dll.6BF03936_…`，SHA-256
//! `b487f3e0…a2d33a5`，pefile FileVersion 10.1.0.3035，ImageBase
//! 0x6b180000）。主 Load `0x6b1955a0` 的版本分发（汇编一手证据）：
//! 有符号 `CMP AX,0xb; JG` **>11 拒绝**；`CMP AX,3; JG` **版本 ≤3 → 旧加载
//! 器 `FUN_6b197dc0`**（调用点 `0x6b1955f2`，参数 `(this-0x54, 流, 版本)`，
//! 直接返回其结果）；**版本 ==4 → 探测函数 `FUN_6b198420`**，探测真 → 旧
//! 加载器 + 一个丢弃的 `String::Read`（IAT `0x6b1aa114`），假 → 主分支
//! （版本 5..=11，见 [`crate::layers::fdo_graphics_layer`]）。
//!
//! ## 旧加载器 `FUN_6b197dc0` 字段序（版本 0..=3）
//!
//! 字节序全部来自反编译一手证据（读流调用逐个核对），**无样本字节佐证**
//! （仓库参考样本 FDO 版本为 11，不进入本分支）：
//!
//! 1. String（AfCore String::Read）——名称，经 BSTR 传入内部接口；
//! 2. int16；3. int16（各自传入内部接口，语义未定性）；
//! 4. double；5. double；
//! 6. int32 → this+0xe8（与主分支步骤 2 同槽）；7. int16 → this+0xe0
//!    （与主分支步骤 3 同槽，非 0 时原生另做接线）；
//! 8. LoadObject（**ILayer** `{34C20002-…}`）→ this+0xec：关联要素图层
//!    （与主分支步骤 4 同请求 IID、同槽）；
//! 9. LoadObject（**ISpatialReference** `{4434D680-…}`，`DAT_6b1abd20`）；
//! 10. LoadObject（**IElementCollection2** `{B2BFC403-…}`）→ this+0x11c
//!     （与主分支步骤 5 同请求 IID、同槽）；
//! 11. `FUN_6b197be0`：LoadObject（**IName** `{4BA33331-…}`，
//!     `DAT_6b1abcf0`）后调 `ConvertToFullPath`（LayerLib.dll 导出）——
//!     除对象条目外不读流；
//! 12. 版本 ≥2：int16；13. String；14. String；15. int32；
//!     16. LoadObject（**ISymbol** `{F3435802-…}`，
//!     `DAT_6b1abd70`，传给 SelectionSymbol；该槽复用已闭合的符号族分派器）；
//!     17. int16；18. int16；19. LoadObject（**IColor** `{20CD40B0-…}`，
//!     该槽复用已闭合的颜色族分派器）；
//! 20. 版本 ≥3（`2 < ver`）：String（收尾）。
//!
//! 旧加载器在收尾 String 后返回——即旧体在最后一个字段处**完整终结**
//! （与主分支不同：主分支步骤 7 起还有大量尾段）。
//!
//! ## 版本 4 探测（仅研究闭合，不实现）
//!
//! `FUN_6b198420` 在**流绝对位置 0** 操作：先以 `IStream::Seek`（虚表
//! +0x14）记录当前位置并回卷到 0，读 16 字节 GUID 与 FDOGraphicsLayer
//! 本类 CLSID（`DAT_6b1ac0e0`）及 **Map coclass CLSID `{E6BDAA76-…}`**
//! （`DAT_6b1ac100`）比对；随后按分支读 `[可选 int32][16 字节 GUID]`、
//! 经辅助函数 `FUN_6b19e860` 对流对象本身 QI **IObjectStream**
//! `{18A45BA7-…}` 并从返回接口读一个 int32，再读 short 判定（分支 A 比对
//! 4、分支 B 比对 9）；分支 A 在判定前把流位置恢复到记录值，分支 B 不恢
//! 复。探测真 → 旧加载器 + 丢弃 String。该行为依赖整个流的绝对起点、
//! IObjectStream 查询的运行时结果与分支 B 的不复位语义，子对象读取器无
//! 从严格复现——本模块对版本 4 显式拒绝（见
//! [`read_fdo_graphics_layer_legacy_prefix_10_1`]），`fdo_graphics_layer`
//! 主读取器维持版本 <5 停止不变。
//!
//! ## FDO 元素循环类 FDOGraphicsSublayer（体读取器实现）
//!
//! 主 Load 步骤 13（版本 ≥9）：int32 计数 + 每元素 `operator new(0x88)` +
//! 构造 `0x6b1a15f0(内存, 属主 this-0x54, 0)` + 经 QI 辅助 `FUN_6b19e210`
//! 查询 **IPersistStream** `{00000109-…}`（对象 +0x10 的 QI 转发到主 QI
//! `0x6b1a1890`，按 IID 表解析到子对象 +0x40）后调其虚表 +0x14 Load。
//! RTTI：TD `0x6b1b5bf4`、主虚表 `0x6b1ac764`、IPersistStream 虚表
//! `0x6b1ac880`。**C++ 类 `.?AVFDOGraphicsSublayer@@`**；GetClassID
//! `0x6b1a1ae0` 写出 `{4ABA3851-BC3A-43C2-8FBE-93A65AD91E83}`（未登记于
//! esri.clsid.ecfg——内部子对象，元素记录**无对象头**，该 CLSID 不出现
//! 在流中）。体字段序（Load `0x6b1a1b10` 反编译一手证据，**无样本字节
//! 佐证**——样本计数 0）：
//!
//! 1. int16 版本：`2 < (short)版本` 拒绝（0x80004005）；
//! 2. int32（→ 对象+0x4c，构造默认 -1）；
//! 3. String；4. int16（→ 对象+0x80，构造默认 0xFFFF）；
//! 5. double（→ 对象+0x78）；6. double（→ 对象+0x70）；
//! 7. String；
//! 8. 版本 ==1：int16；
//! 9. 版本 ≥2：double（→ 对象+0x58）、double（→ 对象+0x60）。
//!
//! Save `0x6b1a1c40` 经 `AfStreamGetSaveVersion(文档版本, 2, 表@0x6b1ab0a8)`
//! 取保存版本后写出同序字段（两个尾随 double 无版本门，版本 1 专属
//! short 无写侧对应——读写不对称，见研究记录）。
//!
//! ## 委派子对象 C++ 类身份（仅研究闭合，无代码）
//!
//! 主 Load 步骤 6 的委派体（this+0xbc 虚表 +0x14 直调、无对象头）经构
//! 造函数（`0x6b18db50` 内 `0x6b18df9d` 调用点）定名：FDO 构造时经
//! AfCore `AfCreateInstance` 以 **FeatureLayer coclass CLSID
//! `{E663A651-8AAD-11D0-BEC7-00805F7C4268}`**（`DAT_6b1ac030`）聚合创建
//! （outer=本对象+0x60，IID_IUnknown，结果存 +0xb8），再 QI 出
//! IPersistStream（`{00000109-…}` → +0xbc）、ILayer（`{34C20002-…}` →
//! +0xc4）、IFeatureLayer（`{40A9E885-…}` → +0xc8）等接口。即委派体 =
//! **聚合 FeatureLayer 实例的 IPersistStream::Load**，其按 FeatureLayer
//! 字段序读取为第六轮样本逐字段吻合的根因。Save 侧委派未逐项展开。
//!
//! 全部结论为静态反编译证据，未运行目标软件；样本对位仅为辅助观察。

use crate::layers::fdo_graphics_layer::AssociatedLayer10_1;
use crate::layers::feature_layer::{
    FEATURE_CLASS_NAME_CLASS_ID, FEATURE_LAYER_CLASS_ID, FeatureClassNamePrefix10_1,
    FeatureClassNameStop10_1, FeatureLayerStop10_1, read_feature_class_name_body_10_1,
    read_feature_layer_prefix_10_1,
};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, SymbolStop10_1,
    read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::objects::element_collection::{
    ELEMENT_COLLECTION_CLASS_ID, ElementCollection10_1, ElementCollectionStop10_1,
    read_element_collection_10_1,
};
use crate::objects::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 旧加载器接受的最大版本：主 Load 分发 `CMP AX,3; JG` 有符号比较，≤3 进
/// 旧加载器（负值位模式原生同样进入，本读取器一并拒绝——较原生更严，负值
/// 版本无业务意义）。
const MAX_LEGACY_VERSION: i16 = 3;

/// FDOGraphicsSublayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、
/// Data4 原序）。
///
/// GUID 字符串形式为 `{4ABA3851-BC3A-43C2-8FBE-93A65AD91E83}`。证据：
/// AnnoLayer.dll FDOGraphicsSublayer IPersistStream 虚表 `0x6b1ac880` 槽
/// +0x0c 的 GetClassID `0x6b1a1ae0` 写出（主虚表 `0x6b1ac764` 槽 +0x14 的
/// `0x6b1a1590` 写出同值，两处互证）。该 CLSID 未登记于 esri.clsid.ecfg：
/// 元素记录由 FDO Load 步骤 13 的循环直接构造、无对象头，此常量不用于
/// 流内比对，仅作类身份证据。
pub const FDO_GRAPHICS_SUBLAYER_CLASS_ID: [u8; 16] = [
    0x51, 0x38, 0xba, 0x4a, 0x3a, 0xbc, 0xc2, 0x43, 0x8f, 0xbe, 0x93, 0xa6, 0x5a, 0xd9, 0x1e, 0x83,
];

/// 旧加载器第 9 步 ISpatialReference 槽位（`{4434D680-…}`）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LegacySpatialReferenceSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 ProjectedCoordinateSystem 对象：体由专门读取器完整消费（其后可
    /// 继续旧体后续字段）。
    ProjectedCoordinateSystem(Box<ProjectedCoordinateSystem10_1>),
    /// 新 GeographicCoordinateSystem 对象（F 轮接线三分派）：体由
    /// `spatial_reference_slots` 读取器完整消费。
    GeographicCoordinateSystem(
        Box<crate::objects::spatial_reference_slots::GeographicCoordinateSystem10_1>,
    ),
    /// 新 UnknownCoordinateSystem 对象（F 轮接线三分派）：体同上完整消费。
    UnknownCoordinateSystem(
        Box<crate::objects::spatial_reference_slots::UnknownCoordinateSystem10_1>,
    ),
}

/// 旧加载器第 11 步 IName 槽位（`{4BA33331-…}`，经 `FUN_6b197be0`）的
/// 分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyDataSource10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 FeatureClassName 对象：已解析体（完整消费时本读取器继续后续字
    /// 段；体内停止时见 [`LegacyFdoStop10_1::DataSourceStopped`]）。
    FeatureClassName(Box<FeatureClassNamePrefix10_1>),
}

/// 旧加载器前缀的停止原因；`None`（无停止原因）= 旧体已完整消费（旧加
/// 载器收尾 String 后返回，其后无 FDO 字段）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyFdoStop10_1 {
    /// ILayer 槽新 FeatureLayer 体在其前缀停止点停止：其体剩余无外部长
    /// 度，旧体后续槽位（空间参考起）不可达。
    AssociatedLayerBodyStopped { layer_stop: FeatureLayerStop10_1 },
    /// ILayer 槽新对象的类标识不是 FeatureLayer：游标回退到该对象头起点。
    AssociatedLayerUnsupported { class_id: [u8; 16], key: u32 },
    /// 关联要素图层前缀读取失败（版本超限、截断以外的内容性失败等）：
    /// 游标与登记表回退到该对象头起点。
    AssociatedLayerFailed { reason: String },
    /// ISpatialReference 槽新对象的类标识不是 ProjectedCoordinateSystem
    /// （如 GCS）：体未知且无外部长度，游标回退到该对象头起点。
    SpatialReferenceUnsupported { class_id: [u8; 16], key: u32 },
    /// IElementCollection2 槽的未知新对象：游标回退到对象头起点。
    ElementCollectionNewObject { class_id: [u8; 16], key: u32 },
    /// ElementCollection 内部停止，后续旧体字段不可达。
    ElementCollectionStopped(ElementCollectionStop10_1),
    /// IName 槽新对象的类标识不是 FeatureClassName：游标回退到该对象头起
    /// 点。
    DataSourceUnsupported { class_id: [u8; 16], key: u32 },
    /// FeatureClassName 体内停止（WorkspaceName 链 / 选择子分支的新对象
    /// 等，见 [`FeatureClassNameStop10_1`]）：其体剩余无外部长度，旧体后
    /// 续字段不可达，游标停在该停止点。
    DataSourceStopped { stop: FeatureClassNameStop10_1 },
    /// FeatureClassName 体读取失败：游标与登记表回退到该对象头起点。
    DataSourceFailed { reason: String },
    /// 版本 ≥2 的 SelectionSymbol 槽新对象类不在符号闭合集：游标回退到对象头。
    RendererNewObject { class_id: [u8; 16], key: u32 },
    /// SelectionSymbol 符号体内部槽未闭合。
    RendererBodyStopped { stop: SymbolStop10_1 },
    /// 版本 ≥2 的 IColor 槽新对象类不在颜色闭合集：游标回退到对象头。
    ColorNewObject { class_id: [u8; 16], key: u32 },
}

impl LegacyFdoStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::AssociatedLayerBodyStopped { .. } => {
                "旧加载器关联要素图层（ILayer 槽）体在其前缀停止点停止：体剩\
                 余无外部长度，空间参考槽起不可达"
                    .to_string()
            }
            Self::AssociatedLayerUnsupported { class_id, key } => format!(
                "旧加载器 ILayer 槽新对象（键 {key}）类标识 {} 不是 \
                 FeatureLayer：体未知且无外部长度，游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::AssociatedLayerFailed { reason } => {
                format!("旧加载器关联要素图层前缀解析失败（{reason}），体保留未知")
            }
            Self::SpatialReferenceUnsupported { class_id, key } => format!(
                "旧加载器 ISpatialReference 槽新对象（键 {key}）类标识 {} 不是投影\
                 坐标系：体未知且无外部长度，游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::ElementCollectionNewObject { class_id, key } => format!(
                "旧加载器 IElementCollection2 槽新对象（键 {key}，类标识 {}）：\
                 类体未知，游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::ElementCollectionStopped(stop) => {
                format!("旧加载器 ElementCollection 体内停止：{}", stop.describe())
            }
            Self::DataSourceUnsupported { class_id, key } => format!(
                "旧加载器 IName 槽新对象（键 {key}）类标识 {} 不是 \
                 FeatureClassName：体未知且无外部长度，游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::DataSourceStopped { stop } => {
                format!(
                    "旧加载器数据源 FeatureClassName 体内停止（{stop:?}）：体剩\
                         余无外部长度，后续字段不可达"
                )
            }
            Self::DataSourceFailed { reason } => {
                format!("旧加载器数据源 FeatureClassName 体解析失败（{reason}），体保留未知")
            }
            Self::RendererNewObject { class_id, key } => format!(
                "旧加载器 SelectionSymbol 槽新对象（键 {key}，类标识 {}）不受支持：\
                 游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::RendererBodyStopped { stop } => {
                format!("旧加载器 SelectionSymbol 符号体内停止：{stop:?}")
            }
            Self::ColorNewObject { class_id, key } => format!(
                "旧加载器 IColor 槽新对象（键 {key}，类标识 {}）不受支持：\
                 游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
        }
    }
}

/// 10.1 FDOGraphicsLayer 旧加载器（`FUN_6b197dc0`，版本 0..=3）已确认字段的
/// 解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFdoGraphicsLayerPrefix10_1 {
    /// 2 字节版本 short 的无符号位模式（主 Load 分发有符号比较 ≤3 进旧加
    /// 载器；负值位模式本读取器一并拒绝，较原生更严）。
    pub version: u16,
    /// 第 1 步 ILayer.Name。
    pub name: String,
    /// 第 2 步 ILayer.Visible 的原始 short。
    pub short_a: u16,
    /// 第 3 步 ILayer.ShowTips 的原始 short。
    pub short_b: u16,
    /// 第 4 步 ILayer.MinimumScale 的原始 double。
    pub double_a: f64,
    /// 第 5 步 ILayer.MaximumScale 的原始 double。
    pub double_b: f64,
    /// 第 6 步 int32 → this+0xe8（与主分支步骤 2 同槽，样本主分支值 3），
    /// 语义未定，原样保留。
    pub int32_field: i32,
    /// 第 7 步 int16 → this+0xe0（与主分支步骤 3 同槽，样本主分支值
    /// 0xFFFF），原样保留。
    pub short_field: u16,
    /// 第 8 步 ILayer 槽位（关联要素图层）分类结果。
    pub associated_layer: AssociatedLayer10_1,
    /// 第 9 步 ISpatialReference 槽位分类（到达时）。
    pub spatial_reference: LegacySpatialReferenceSlot10_1,
    /// 第 10 步 IElementCollection2 槽位分类（到达时）。`None` = 未到达。
    pub element_collection: Option<ObjectEntry10_1>,
    /// 新 ElementCollection 对象的已读取体；空值或其他类时为空。
    pub element_collection_body: Option<Box<ElementCollection10_1>>,
    /// 第 11 步 IName 槽位（数据源）分类（到达时）。
    pub data_source: LegacyDataSource10_1,
    /// 第 12 步 IFeatureLayer.Selectable 原始 short（版本 ≥2）。
    pub v2_short: Option<u16>,
    /// 第 13 步 IFeatureLayer.DataSourceType（版本 ≥2）。
    pub v2_string_a: Option<String>,
    /// 第 14 步 IFeatureLayer.DisplayField（版本 ≥2）。
    pub v2_string_b: Option<String>,
    /// 第 15 步 IFeatureSelection.CombinationMethod 原始 int32（版本 ≥2）。
    pub v2_int32: Option<i32>,
    /// 第 16 步 SelectionSymbol 的原始对象条目分类，保持既有公开类型和值。
    pub renderer_slot: Option<ObjectEntry10_1>,
    /// 第 17 步 IFeatureLayer.ScaleSymbols 原始 short（版本 ≥2）。
    pub v2_short_x: Option<u16>,
    /// 第 18 步 IFeatureSelection.SetSelectionSymbol 原始 short（版本 ≥2）。
    pub v2_short_y: Option<u16>,
    /// 第 19 步 SelectionColor 的原始对象条目分类，保持既有公开类型和值。
    pub color_slot: Option<ObjectEntry10_1>,
    /// 第 20 步 IDataLayer.RelativeBase（版本 ≥3）。
    pub tail_string: Option<String>,
    /// 停止原因；`None` = 旧体已完整消费（旧加载器在收尾 String 后返回）。
    pub stop: Option<LegacyFdoStop10_1>,
    /// 停止点或终点之后剩余的字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从版本 short 起读取 10.1 FDOGraphicsLayer 旧加载器（`FUN_6b197dc0`）
/// 的已确认字段。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、确认类标识为 FDOGraphicsLayer，并把 Reader 定位在对象体起点（版本
/// short 处）。版本分发由调用者按主 Load 语义决定：本函数只接受 0..=3；
/// 版本 4 的探测分支（`FUN_6b198420`，见模块文档）未展开，显式拒绝；版
/// 本 ≥5 属主分支（[`crate::layers::fdo_graphics_layer`]）。负值位模式一并拒绝
/// （原生有符号分发会进旧加载器，负值版本无业务意义，较原生更严）。
///
/// `max_bytes` 是资源限额，原样转发给全部 String 与嵌套体读取器（约束其
/// 声明长度），在任何读取与分配之前检查；`usize::MAX` 表示不额外设限。
/// `table` 是跨流共享的键登记表：新对象键在登记后若无法消费其体，则停止
/// 时不保留该登记（回退语义，与 `fdo_graphics_layer` 读取器一致）。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回（含停止
/// 与完整消费）才提交；任何失败都不推进调用者游标、不修改登记表。停止返
/// 回是成功：部分停止原因带回退（见各变体文档）。截断（UnexpectedEof）
/// 一律按错误向上传播。这是严格安全子集，不代表完整原生行为；字段序证据
/// 为反编译一手证据，无样本字节佐证（样本版本 11 不进入本分支）。
pub fn read_fdo_graphics_layer_legacy_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LegacyFdoGraphicsLayerPrefix10_1> {
    let entry_offset = reader.position();
    // 入口克隆：停止回退用「入口克隆 + 前向读取」重定位；目标都在入口之后。
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FDOGraphicsLayer 旧加载器体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 版本 short：主 Load 分发 CMP AX,3; JG（有符号）≤3 进旧加载器；
    //    负值位模式一并拒绝（较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 为负值位模式：原生有符号分发会进入旧加载器，本读\
             取器按数据错误拒绝（较原生更严）"
        )));
    }
    if version > MAX_LEGACY_VERSION as u16 {
        return Err(invalid(format!(
            "架构版本 {version} 不属于旧加载器分支（0..=3）：版本 4 的探测函数\
             FUN_6b198420 未展开（依赖流绝对起点与 IObjectStream 查询），版本 \
             5..=11 由主分支读取器负责"
        )));
    }

    // 2-7. String、short ×2、double ×2、int32（+0xe8）、short（+0xe0）。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let short_a = trial.read_u16_le()?;
    let short_b = trial.read_u16_le()?;
    let double_a = trial.read_f64_le()?;
    let double_b = trial.read_f64_le()?;
    let int32_field = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let short_field = trial.read_u16_le()?;

    // 8. ILayer 槽位（关联要素图层；与主分支步骤 4 同请求 IID）。空标记/
    //    回引继续；新 FeatureLayer 在其前缀停止点停止（体剩余无外部长
    //    度）；其他类回退停止；内容性失败回退停止。截断（UnexpectedEof）
    //    向上传播为错误（保持「截断输入必须失败」契约）。
    let mut stop = None;
    let associated_layer = {
        let slot_offset = trial.position();
        let saved_table = trial_table.clone();
        match read_object_entry_10_1(&mut trial, &mut trial_table)? {
            ObjectEntry10_1::Null => AssociatedLayer10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                AssociatedLayer10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key: _ }
                if class_id == FEATURE_LAYER_CLASS_ID =>
            {
                match read_feature_layer_prefix_10_1(&mut trial, max_bytes, &mut trial_table) {
                    Ok(prefix) => {
                        stop = Some(LegacyFdoStop10_1::AssociatedLayerBodyStopped {
                            layer_stop: prefix.stop.clone(),
                        });
                        AssociatedLayer10_1::FeatureLayer(Box::new(prefix))
                    }
                    Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                        return Err(error);
                    }
                    Err(error) => {
                        trial_table = saved_table;
                        trial = reposition(&origin, entry_offset, slot_offset);
                        stop = Some(LegacyFdoStop10_1::AssociatedLayerFailed {
                            reason: error.to_string(),
                        });
                        AssociatedLayer10_1::Null
                    }
                }
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, slot_offset);
                stop = Some(LegacyFdoStop10_1::AssociatedLayerUnsupported { class_id, key });
                AssociatedLayer10_1::Null
            }
        }
    };

    // 9. ISpatialReference 槽位（DAT_6b1abd20={4434D680-…}）。
    let mut spatial_reference = LegacySpatialReferenceSlot10_1::Null;
    if stop.is_none() {
        let slot_offset = trial.position();
        let saved_table = trial_table.clone();
        match read_object_entry_10_1(&mut trial, &mut trial_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                spatial_reference = LegacySpatialReferenceSlot10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key: _ }
                if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
            {
                // PCS 体由专门读取器完整消费，max_bytes 原样转发，成功即继
                // 续旧体后续字段。
                let pcs = read_projected_coordinate_system_10_1(&mut trial, max_bytes)?;
                spatial_reference =
                    LegacySpatialReferenceSlot10_1::ProjectedCoordinateSystem(Box::new(pcs));
            }
            ObjectEntry10_1::NewObject { class_id, key: _ }
                if class_id
                    == crate::objects::spatial_reference_slots::GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID =>
            {
                // GCS 体（F 轮接线三分派）。
                let gcs = crate::objects::spatial_reference_slots::read_geographic_coordinate_system_10_1(
                    &mut trial,
                    max_bytes,
                )?;
                spatial_reference =
                    LegacySpatialReferenceSlot10_1::GeographicCoordinateSystem(Box::new(gcs));
            }
            ObjectEntry10_1::NewObject { class_id, key: _ }
                if class_id
                    == crate::objects::spatial_reference_slots::UNKNOWN_COORDINATE_SYSTEM_CLASS_ID =>
            {
                // Unknown 体（F 轮接线三分派）。
                let unk = crate::objects::spatial_reference_slots::read_unknown_coordinate_system_10_1(
                    &mut trial,
                    max_bytes,
                )?;
                spatial_reference =
                    LegacySpatialReferenceSlot10_1::UnknownCoordinateSystem(Box::new(unk));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 其他空间参考类体未知且无外部长度：回退并停止。
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, slot_offset);
                stop = Some(LegacyFdoStop10_1::SpatialReferenceUnsupported { class_id, key });
            }
        }
    }

    // 10. IElementCollection2 槽位（与主分支步骤 5 同请求 IID）。
    let mut element_collection = None;
    let mut element_collection_body = None;
    if stop.is_none() {
        let slot_offset = trial.position();
        let saved_table = trial_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        element_collection = Some(entry);
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            if class_id == ELEMENT_COLLECTION_CLASS_ID {
                let body = read_element_collection_10_1(&mut trial, max_bytes, &mut trial_table)?;
                stop = body
                    .stop
                    .clone()
                    .map(LegacyFdoStop10_1::ElementCollectionStopped);
                element_collection_body = Some(Box::new(body));
            } else {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, slot_offset);
                stop = Some(LegacyFdoStop10_1::ElementCollectionNewObject { class_id, key });
            }
        }
    }

    // 11. IName 槽位（FUN_6b197be0：LoadObject(IName) + ConvertToFullPath，
    //     除对象条目外不读流）。
    let mut data_source = LegacyDataSource10_1::Null;
    if stop.is_none() {
        let slot_offset = trial.position();
        let saved_table = trial_table.clone();
        match read_object_entry_10_1(&mut trial, &mut trial_table)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                data_source = LegacyDataSource10_1::BackReference { class_id, key };
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == FEATURE_CLASS_NAME_CLASS_ID =>
            {
                match read_feature_class_name_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut trial_table,
                    key,
                ) {
                    Ok(prefix) => {
                        if let Some(fcn_stop) = prefix.stop.clone() {
                            // FCN 体内停止：其体剩余无外部长度，旧体后续字段
                            // 不可达（游标停在 FCN 的停止点，登记表按其结果保
                            // 留）。
                            stop = Some(LegacyFdoStop10_1::DataSourceStopped { stop: fcn_stop });
                        }
                        data_source = LegacyDataSource10_1::FeatureClassName(Box::new(prefix));
                    }
                    Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                        return Err(error);
                    }
                    Err(error) => {
                        trial_table = saved_table;
                        trial = reposition(&origin, entry_offset, slot_offset);
                        stop = Some(LegacyFdoStop10_1::DataSourceFailed {
                            reason: error.to_string(),
                        });
                    }
                }
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, slot_offset);
                stop = Some(LegacyFdoStop10_1::DataSourceUnsupported { class_id, key });
            }
        }
    }

    // 12-19. 版本 ≥2 块。
    let mut v2_short = None;
    let mut v2_string_a = None;
    let mut v2_string_b = None;
    let mut v2_int32 = None;
    let mut renderer_slot = None;
    let mut v2_short_x = None;
    let mut v2_short_y = None;
    let mut color_slot = None;
    if stop.is_none() && version >= 2 {
        v2_short = Some(trial.read_u16_le()?);
        v2_string_a = Some(read_string_10_1(&mut trial, max_bytes)?);
        v2_string_b = Some(read_string_10_1(&mut trial, max_bytes)?);
        v2_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));

        // 16. SelectionSymbol 的 ISymbol 槽（DAT_6b1abd70={F3435802-…}）：
        //     复用已闭合的符号族分派器。未知类回退至槽头，嵌套体停止则
        //     保留停止原因，后续字段不可达。
        let slot_offset = trial.position();
        let saved_table = trial_table.clone();
        let mut entry_reader = trial.clone();
        let mut entry_table = trial_table.clone();
        renderer_slot = Some(read_object_entry_10_1(&mut entry_reader, &mut entry_table)?);
        let slot = read_symbol_slot_full_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            MAX_SYMBOL_NESTING_DEPTH,
        )?;
        if let SymbolSlotFull10_1::Unsupported { class_id, key } = &slot {
            trial_table = saved_table;
            trial = reposition(&origin, entry_offset, slot_offset);
            stop = Some(LegacyFdoStop10_1::RendererNewObject {
                class_id: *class_id,
                key: *key,
            });
        } else {
            if let Some(symbol_stop) = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol) {
                stop = Some(LegacyFdoStop10_1::RendererBodyStopped { stop: symbol_stop });
            }
        }
        if stop.is_none() {
            v2_short_x = Some(trial.read_u16_le()?);
            v2_short_y = Some(trial.read_u16_le()?);

            // 19. IColor 槽位（DAT_6b1abd50={20CD40B0-…}）：解析已闭合颜色族。
            let slot_offset = trial.position();
            let saved_table = trial_table.clone();
            let mut entry_reader = trial.clone();
            let mut entry_table = trial_table.clone();
            color_slot = Some(read_object_entry_10_1(&mut entry_reader, &mut entry_table)?);
            let slot = read_color_slot_10_1(&mut trial, max_bytes, &mut trial_table)?;
            if let ColorSlot10_1::Unsupported { class_id, key } = &slot {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, slot_offset);
                stop = Some(LegacyFdoStop10_1::ColorNewObject {
                    class_id: *class_id,
                    key: *key,
                });
            }
        }
    }

    // 20. 版本 ≥3（2 < ver）收尾 String。
    let mut tail_string = None;
    if stop.is_none() && version > 2 {
        tail_string = Some(read_string_10_1(&mut trial, max_bytes)?);
    }

    let result = LegacyFdoGraphicsLayerPrefix10_1 {
        version,
        name,
        short_a,
        short_b,
        double_a,
        double_b,
        int32_field,
        short_field,
        associated_layer,
        spatial_reference,
        element_collection,
        element_collection_body,
        data_source,
        v2_short,
        v2_string_a,
        v2_string_b,
        v2_int32,
        renderer_slot,
        v2_short_x,
        v2_short_y,
        color_slot,
        tail_string,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 把游标重定位到「传入 Reader 窗口」的绝对偏移 `target`（入口之后的前向
/// 读取实现；目标必在窗口内，越界按内部错误处理）。
fn reposition<'a>(origin: &Reader<'a>, entry_offset: usize, target: usize) -> Reader<'a> {
    debug_assert!(target >= entry_offset);
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - entry_offset)
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// 10.1 FDOGraphicsSublayer（FDO 元素循环类，Load `0x6b1a1b10`）体的解析
/// 结果。
///
/// 元素记录无对象头（FDO Load 步骤 13 的循环直接构造），因此本结构不含
/// 键；体内也无 LoadObject 调用，不涉及登记表。
#[derive(Debug, Clone, PartialEq)]
pub struct FdoGraphicsSublayer10_1 {
    /// 2 字节版本 short 的无符号位模式。原生 `2 < (short)版本` 拒绝（汇编
    /// 语义经反编译核对）；负值位模式一并拒绝（较原生更严）。
    pub version: u16,
    /// int32（→ 对象+0x4c，构造默认 -1），语义未定，原样保留。
    pub int32_field: i32,
    /// 第一个 String（成员槽位未从反编译恢复，语义未定性）。
    pub name: String,
    /// int16（→ 对象+0x80，构造默认 0xFFFF），语义未定，原样保留。
    pub short_field: u16,
    /// double（→ 对象+0x78），原样保留。
    pub double_a: f64,
    /// double（→ 对象+0x70），原样保留。
    pub double_b: f64,
    /// 第二个 String（成员槽位未从反编译恢复）。
    pub secondary_name: String,
    /// 版本 ==1 追加的 int16（丢弃式局部存储，语义未定）。
    pub version1_short: Option<u16>,
    /// 版本 ≥2 追加的两个 double（→ 对象+0x58/+0x60）。
    pub version2_doubles: Option<[f64; 2]>,
    /// 体终点之后剩余的字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从记录体起点读取 10.1 FDOGraphicsSublayer 体（无对象头、无键）。
///
/// 起点约定：FDO Load 步骤 13 循环内每元素 `new 0x88` + 构造后由 IPersist
/// Stream::Load 读取的体起点。字段序证据为 AnnoLayer.dll Load `0x6b1a1b10`
/// 反编译一手证据，**无样本字节佐证**（样本元素计数 0，无实例）；Save
/// `0x6b1a1c40` 与读侧序对称（尾随两个 double 无版本门——写侧总是随保存
/// 版本 ≥2 写出）。版本 >2 拒绝、负值位模式一并拒绝（较原生更严）；
/// `max_bytes` 约束两个 String 的声明长度，在任何读取与分配之前检查。事
/// 务性：全部字段在 trial 游标上读取，成功才提交；截断（UnexpectedEof）
/// 一律按错误向上传播。体在最后一个版本门字段后完整终结（原生 Load 返回
/// 即体终点）。
pub fn read_fdo_graphics_sublayer_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<FdoGraphicsSublayer10_1> {
    let entry_offset = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FDOGraphicsSublayer 体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();

    // 1. 版本 short：原生 2 < (short)版本 拒绝（0x80004005）；负值位模式一
    //    并拒绝（较原生更严，负值版本无业务意义）。
    let version = trial.read_u16_le()?;
    if (version as i16) > 2 || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 FDOGraphicsSublayer::Load 接受范围（>2 拒绝；负值\
             位模式一并拒绝）"
        )));
    }

    // 2-7. int32、String、int16、double ×2、String。
    let int32_field = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let short_field = trial.read_u16_le()?;
    let double_a = trial.read_f64_le()?;
    let double_b = trial.read_f64_le()?;
    let secondary_name = read_string_10_1(&mut trial, max_bytes)?;

    // 8. 版本 ==1：追加 int16。
    let version1_short = if version == 1 {
        Some(trial.read_u16_le()?)
    } else {
        None
    };

    // 9. 版本 ≥2：追加 double ×2。
    let version2_doubles = if version >= 2 {
        Some([trial.read_f64_le()?, trial.read_f64_le()?])
    } else {
        None
    };

    let result = FdoGraphicsSublayer10_1 {
        version,
        int32_field,
        name,
        short_field,
        double_a,
        double_b,
        secondary_name,
        version1_short,
        version2_doubles,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}
