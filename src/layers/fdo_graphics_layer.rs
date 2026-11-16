//! ArcMap 10.1 FDOGraphicsLayer（FDO 注记图层）对象体已确认前缀的严格读取
//! 子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`「FDOGraphicsLayer::Load 闭
//! 合（2026-11-14 第六轮）」节。AnnoLayer.dll 10.1.0.3035（cab 条目
//! `AnnoLayer.dll.6BF03936_…`，SHA-256 `b487f3e0…a2d33a5`，pefile
//! FileVersion 10.1.0.3035，ImageBase 0x6b180000）。定位链：RTTI
//! `.?AVFDOGraphicsLayer@@`（TD `0x6b1b56e0`）→ 36 个 COL → 虚表；其中
//! COL `0x6b1adb30`（this-adjustor 0x54）的虚表 `0x6b1ab984` 槽 +0x0c =
//! GetClassID `0x6b195e60`（写出本类 CLSID `{34B2EF83-…}`，与类表/样本三
//! 方互证），槽 +0x14 = **Load `0x6b1955a0`**（+0x18 = Save
//! `0x6b195b90`）。全部证据为静态反编译 + 样本字节对照，未运行目标软件。
//!
//! ## Load `0x6b1955a0` 字段序（样本版本 11 走主分支）
//!
//! 1. int16 版本：汇编 `CMP AX,0xb; JG` 为**有符号**比较，>11 拒绝；
//!    版本 <4 → 旧加载器 `FUN_6b197dc0`（未展开）；版本 ==4 → 探测函数
//!    `FUN_6b198420`（读取并 Seek 流、比对 CLSID，未展开），探测为真走旧
//!    加载器 + String::Read，为假落入主分支——本读取器对版本 <5 一律停止；
//! 2. 主分支（版本 5..=11）：int32 → this+0xe8（样本 3 @`/Maps:0x290`）；
//! 3. int16 → this+0xe0（样本 0xFFFF @`0x294`；非 0 时对已装图层做关联接
//!    线 `FUN_6b196140`，不读流）；
//! 4. LoadObject（请求 IID `{34C20002-4D3C-11D0-92D8-00805F7C28B0}`=
//!    ILayer，第五轮类型库考证）→ this+0xec：**关联要素图层**。样本为新
//!    FeatureLayer（键 32，头 @`0x296`，体 `0x2aa..0x1622`）；
//! 5. LoadObject（请求 IID `{B2BFC403-6B4C-4490-8878-EA62C4F49720}`=
//!    IElementCollection2，类型库考证）→ this+0x11c。样本为新
//!    ElementCollection（键 86，头 @`0x1608`，体仅版本 short 4 + 元素计数
//!    0，Layer.dll 未反编译、体布局为样本观察）；
//! 6. 委派加载：this+0xbc 接口虚表 +0x14 的 Load(stream)——**无对象头的内
//!    嵌子对象体**。样本 `0x1622..0x2cc7` 按 FeatureLayer 字段序逐字段吻
//!    合（版本 34、名称「龙安镇人民政府工作底图用界址点注记」、第二名称
//!    「文件地理数据库要素类」、可见性 0xFFFF、数据源 FgdbFeatureClassName
//!    键 87 等），即注记图层内嵌的第二个 FeatureLayer 体；C++ 类身份未闭
//!    合（构造函数未逐项展开）；
//! 7. 版本 <6：写默认值并提前返回（主分支尾段不可达）；
//! 8. 版本 ≥6：int32 → this+0x130（样本 1 @`0x2c01`）、int16 → this+0x1a8
//!    （样本 0xFFFF @`0x2c05`）；版本 6/7 另有 int32 计数 + 循环
//!    [String + int16 + double ×2]（`FUN_6b195a80`，样本版本 11 不走）；
//! 9. int32 → this+0x1b0（样本 0 @`0x2c07`）；
//! 10. LoadObject(IColor `{20CD40B0-…}`)、LoadObject(IColor)、
//!     LoadObject(ISymbolCollection `{0E5D8C65-…}`)：样本 RgbColor 154、
//!     RgbColor 155、SymbolCollection 156；
//! 11. 版本 7..10：String（丢弃）+ int16（丢弃）；
//! 12. 版本 ≥8：int16 → this+0x13c（样本 0 @`0x2cc7`）；随后
//!     `FUN_6b199320`/`FUN_6b199480`/`FUN_6b19f840` 不读流（反编译核对）；
//! 13. 版本 ≥9：int32 元素计数 + 每元素 new 0x88 记录并调其虚表 +0x14
//!     Load（图形元素类，样本计数 0 @`0x2cc9`，非空循环未实证）；
//! 14. 版本 ≥10：int16 → this+0x1bc（样本 0 @`0x2ccd`）；
//!     LoadObject(ILineSymbol `{E6BDAA7D-…}`)、LoadObject(IMarkerSymbol
//!     `{E6BDAA7C-…}`)：样本均为 16 零空标记；
//! 15. LoadObject(IColor) 终结：样本 RgbColor 157（头 @`0x2cef`，体 31 字
//!     节恰终结于条目边界 `0x2d22`，与图层列表条目 2 的标志位置逐字节互
//!     证）。
//!
//! 样本对位（`/Maps:0x28e..0x2d22`，辅助观察）：全部字段按上述序吻合——
//! 0x2cc7..0x2cef 的 40 个零 = [short@+0x13c][int32 计数 0][short@+0x1bc]
//! [ILineSymbol 空标记][IMarkerSymbol 空标记]，其前的 SymbolCollection 体
//! 尾随零与 `02` 字节属其体内（Layer.dll 未展开，按未知保留）。
//!
//! ## 本读取器的可达范围
//!
//! 步骤 4/6 的两个 FeatureLayer 体由 [`crate::layers::feature_layer`] 的前缀读取器
//! 分派解析：关联图层体（步骤 4）在其停止点（渲染器新对象 / +0x144 GUID /
//! 数据源体内停止）之后无法继续——本读取器随之停止（步骤 5 起不可达）。
//! 关联体返回 TailCompleted、空标记或回引时可推进到步骤 5/6。委派体
//! （步骤 6）返回后本读取器仍停止：TailCompleted 表示 FeatureLayer 体已
//! 完整消费，步骤 7-15 的 FDO 后续字段尚未接线；其他状态则表示委派体内部
//! 停止。两者的剩余字节均保留，不猜测未知体的长度。
//!
//! 未解决问题：`FUN_6b197dc0`（版本 <4 旧加载器）与 `FUN_6b198420`（版本
//! 4 探测）未展开；委派子对象（this+0xbc）的 C++ 类身份与构造未闭合（其体
//! 按 FeatureLayer 字段序读取为样本佐证）；ElementCollection 体布局
//! （Layer.dll）未反编译，样本观察为版本 short + int32 计数 + 循环；版本
//! ≥9 元素循环的元素类字段序未闭合（样本计数 0）；步骤 8-15 的字段语义
//! （颜色/符号槽位按请求 IID 类型库考证，业务含义候选）未在非样本版本实
//! 证；全部结论未运行 ArcMap 验证。

use crate::layers::feature_layer::{
    FEATURE_LAYER_CLASS_ID, FeatureLayerPrefix10_1, FeatureLayerStop10_1,
    read_feature_layer_prefix_10_1,
};
use crate::objects::element_collection::{
    ELEMENT_COLLECTION_CLASS_ID, ElementCollection10_1, ElementCollectionStop10_1,
    read_element_collection_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// FDOGraphicsLayer::Load 接受的最大版本：汇编 `CMP AX,0xb; JG` 有符号比
/// 较，>11 拒绝；样本值 11。
const MAX_ARCHITECTURE_VERSION: i16 = 11;

/// FDOGraphicsLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4
/// 原序）。
///
/// GUID 字符串形式为 `{34B2EF83-F4AC-11D1-A245-080009B6F22B}`。证据：
/// `esri.clsid.ecfg` 声明（模块 AnnoLayer.dll）；AnnoLayer.dll GetClassID
/// `0x6b195e60` 写出同一 CLSID（与 Load `0x6b1955a0` 同虚表 `0x6b1ab984`，
/// 第六轮）；仓库参考样本 `/Maps:0x27a` 图层列表条目 1 的对象头与本常量
/// 逐字节相同（键 31）。
pub const FDO_GRAPHICS_LAYER_CLASS_ID: [u8; 16] = [
    0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];

/// Load 第 4 步 ILayer 槽位（关联要素图层）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum AssociatedLayer10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 FeatureLayer 对象：已确认前缀（本读取器在其停止点停止，见
    /// [`FdoGraphicsLayerStop10_1::AssociatedLayerBodyStopped`]）。
    FeatureLayer(Box<FeatureLayerPrefix10_1>),
}

/// Load 第 5 步 IElementCollection2 槽位的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementCollectionSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 ElementCollection 对象（类标识经 [`ELEMENT_COLLECTION_CLASS_ID`]
    /// 校验）：已读取其闭合体（element_collection 模块）；元素循环在新对象
    /// 元素处停止时经 [`FdoGraphicsLayerStop10_1::ElementCollectionStopped`]
    /// 携带原因，游标停在其停止点。
    ElementCollection(Box<ElementCollection10_1>),
    /// 新对象但类标识不是 ElementCollection（体未知且无外部长度）：停止点，
    /// 游标回退到该对象头起点、该键不登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 已确认前缀的停止原因；本读取器对任何成功输入都返回停止原因（见模块文
/// 档「本读取器的可达范围」）。
#[derive(Debug, Clone, PartialEq)]
pub enum FdoGraphicsLayerStop10_1 {
    /// 版本 <5：旧分支（版本 <4 旧加载器、版本 ==4 探测函数）未展开。游标
    /// 停在版本 short 之后，体全部保留。
    VersionBranchNotExpanded { version: u16 },
    /// ILayer 槽新对象的类标识不是 FeatureLayer（体未知且无外部长度）：游
    /// 标回退到该对象头起点。
    AssociatedLayerUnsupported { class_id: [u8; 16], key: u32 },
    /// 关联要素图层（新 FeatureLayer）体在其前缀停止点停止：其体剩余无外
    /// 部长度，本读取器无法继续（步骤 5 起不可达），游标停在该停止点。
    AssociatedLayerBodyStopped {
        /// 关联图层前缀读取器的停止原因（渲染器新对象 / +0x144 GUID 之后 /
        /// 数据源体内停止）。
        layer_stop: FeatureLayerStop10_1,
    },
    /// 关联图层前缀读取失败（版本超限、数据源类不支持、截断等）：游标与登
    /// 记表回退到该对象头起点，条目保留未知。
    AssociatedLayerFailed { reason: String },
    /// IElementCollection2 槽新对象类标识不是 ElementCollection：游标回退到
    /// 该对象头起点。
    ElementCollectionNewObject { class_id: [u8; 16], key: u32 },
    /// IElementCollection2 槽的 ElementCollection 体在元素循环的新对象元素
    /// 处停止（原因透传，见 [`ElementCollectionStop10_1`]）：游标停在其停
    /// 止点，其后步骤不可达。
    ElementCollectionStopped(ElementCollectionStop10_1),
    /// 委派子对象体（this+0xbc，无对象头）按 FeatureLayer 字段序解析成功：
    /// 内部停止时保留未知体；TailCompleted 时停止于尚未接线的 FDO 后续段。
    DelegatedLayerBodyStopped { layer_stop: FeatureLayerStop10_1 },
    /// 委派子对象体按 FeatureLayer 字段序解析失败（如数据源为
    /// 未支持的新类）：游标回退到委派体起点，体保留未知。
    DelegatedLayerBodyFailed { reason: String },
}

impl FdoGraphicsLayerStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::VersionBranchNotExpanded { version } => format!(
                "版本 {version} < 5：旧加载器与版本 4 探测分支未展开，读取器停在\
                 版本 short 之后，体全部保留"
            ),
            Self::AssociatedLayerUnsupported { class_id, key } => format!(
                "ILayer 槽新对象（键 {key}）类标识 {} 不是 FeatureLayer：体未知且\
                 无外部长度，游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::AssociatedLayerBodyStopped { .. } => {
                "关联要素图层体在其前缀停止点停止：其体剩余无外部长度，本读取器\
                 无法继续（ElementCollection2 槽起不可达）"
                    .to_string()
            }
            Self::AssociatedLayerFailed { reason } => {
                format!("关联要素图层前缀解析失败（{reason}），体保留未知")
            }
            Self::ElementCollectionNewObject { class_id, key } => format!(
                "IElementCollection2 槽新对象（键 {key}，类标识 {}）不是 \
                 ElementCollection：体未知且无外部长度，游标回退到该对象头起点",
                crate::layers::layer_list::class_id_hex(class_id)
            ),
            Self::ElementCollectionStopped(stop) => format!(
                "IElementCollection2 槽的 ElementCollection 体在其元素循环停止点\
                 停止：{}",
                stop.describe()
            ),
            Self::DelegatedLayerBodyStopped {
                layer_stop:
                    FeatureLayerStop10_1::Successor(
                        crate::layers::feature_layer_tail::SuccessorStop10_1::TailCompleted,
                    ),
            } => "委派 FeatureLayer 体已完整消费；FDO 后续字段尚未接线，后续流保留未知".to_string(),
            Self::DelegatedLayerBodyStopped { .. } => {
                "委派子对象体（this+0xbc，无对象头）在其 FeatureLayer 前缀停止点\
                 停止：体剩余无外部长度，无法继续"
                    .to_string()
            }
            Self::DelegatedLayerBodyFailed { reason } => {
                format!("委派子对象体按 FeatureLayer 字段序解析失败（{reason}），体保留未知")
            }
        }
    }
}

/// 10.1 FDOGraphicsLayer::Load（`0x6b1955a0`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct FdoGraphicsLayerPrefix10_1 {
    /// 2 字节版本 short 的无符号位模式。原生有符号比较 >11 拒绝（汇编证
    /// 实）；负值位模式一并拒绝（较原生更严，负值版本无业务意义）。
    pub version: u16,
    /// int32 → this+0xe8（样本 3），语义未定，原样保留。
    pub int32_field: i32,
    /// int16 → this+0xe0（样本 0xFFFF，非 0 时原生对关联图层做接线），
    /// 原样保留。
    pub short_field: u16,
    /// Load 第 4 步 ILayer 槽位（关联要素图层）分类结果。
    pub associated_layer: AssociatedLayer10_1,
    /// Load 第 5 步 IElementCollection2 槽位分类（到达时）。`None` = 在关
    /// 联图层槽即停止。
    pub element_collection: Option<ElementCollectionSlot10_1>,
    /// Load 第 6 步委派子对象体按 FeatureLayer 字段序解析出的前缀（到达
    /// 时）。`None` = 未到达或解析失败（见停止原因）。
    pub delegated_layer: Option<Box<FeatureLayerPrefix10_1>>,
    /// 停止原因（恒有）。
    pub stop: FdoGraphicsLayerStop10_1,
    /// 停止点之后剩余的字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 FDOGraphicsLayer 对象体的已确认前缀。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`FDO_GRAPHICS_LAYER_CLASS_ID`] 确认类身份，并从对象头之后的位
/// 置传入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额，原样转发给两个 FeatureLayer 前缀读取器（约束其
/// 字符串声明长度），在任何读取与分配之前检查；`usize::MAX` 表示不额外设
/// 限。`table` 是跨流共享的键登记表：新对象键在登记后若无法消费其体，则停
/// 止时不保留该登记（回退语义，与 GroupLayer/CGL 读取器一致）。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回（含停止）
/// 才提交；任何失败都不推进调用者游标、不修改登记表。停止返回是成功：游标
/// 推进到停止点（部分停止原因带回退，见各变体文档）。这是严格安全子集，
/// 不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_fdo_graphics_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<FdoGraphicsLayerPrefix10_1> {
    let entry_offset = reader.position();
    // 入口克隆：停止回退用「入口克隆 + 前向读取」重定位；目标都在入口之后。
    let origin = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FDOGraphicsLayer 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 版本 short：原生有符号比较（汇编 CMP/JG），>11 拒绝；负值位模式一
    //    并拒绝（较原生更严：负值版本无业务意义，且原生负值会落入 <5 旧分
    //    支，本读取器将其按数据错误处理）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ARCHITECTURE_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 FDOGraphicsLayer::Load 接受范围（>11 拒绝；负值位\
             模式一并拒绝）"
        )));
    }
    if version < 5 {
        // 版本 <4 旧加载器 / 版本 ==4 探测分支未展开：停在版本 short 之后。
        let result = FdoGraphicsLayerPrefix10_1 {
            version,
            int32_field: 0,
            short_field: 0,
            associated_layer: AssociatedLayer10_1::Null,
            element_collection: None,
            delegated_layer: None,
            stop: FdoGraphicsLayerStop10_1::VersionBranchNotExpanded { version },
            remaining_bytes: trial.remaining(),
        };
        *reader = trial;
        return Ok(result);
    }

    // 2/3. int32 → this+0xe8 与 int16 → this+0xe0。
    let int32_field = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let short_field = trial.read_u16_le()?;

    // 4. ILayer 槽位（关联要素图层）。
    let slot_offset = trial.position();
    let saved_table = trial_table.clone();
    let mut stop;
    let associated_layer = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => {
            stop = None;
            AssociatedLayer10_1::Null
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            stop = None;
            AssociatedLayer10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == FEATURE_LAYER_CLASS_ID => {
            // 新 FeatureLayer：解析已确认前缀。体在其读取器停止点处停止时，
            // 关联图层体剩余无外部长度，本读取器随之停止（步骤 5 起不可达）。
            // A 轮起后继段可完整消费（TailCompleted = FL 体语义终点，无剩余
            // 字段），此时按 FDO::Load 字段序继续步骤 5（IElementCollection2
            // 槽，样本 0x1608 起的 ElementCollection 头即该槽对象）。
            // 截断（UnexpectedEof）向上传播为错误（保持「截断输入必须失败」
            // 的读取器契约）；内容性失败（版本超限、类不支持、限额）作为
            // AssociatedLayerFailed 停止返回。
            match read_feature_layer_prefix_10_1(&mut trial, max_bytes, &mut trial_table) {
                Ok(prefix) => {
                    if prefix.stop
                        == crate::layers::feature_layer::FeatureLayerStop10_1::Successor(
                            crate::layers::feature_layer_tail::SuccessorStop10_1::TailCompleted,
                        )
                    {
                        // FL 体完整消费：步骤 5 可达。
                        stop = None;
                    } else {
                        stop = Some(FdoGraphicsLayerStop10_1::AssociatedLayerBodyStopped {
                            layer_stop: prefix.stop.clone(),
                        });
                    }
                    AssociatedLayer10_1::FeatureLayer(Box::new(prefix))
                }
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                    return Err(error);
                }
                Err(error) => {
                    // 内容性失败：回退游标与登记表到该对象头起点，体保留未知。
                    trial_table = saved_table;
                    trial = reposition(&origin, entry_offset, slot_offset);
                    stop = Some(FdoGraphicsLayerStop10_1::AssociatedLayerFailed {
                        reason: error.to_string(),
                    });
                    AssociatedLayer10_1::Null
                }
            }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 非 FeatureLayer 类的新对象：体未知且无外部长度，回退并停止。
            trial_table = saved_table;
            trial = reposition(&origin, entry_offset, slot_offset);
            stop = Some(FdoGraphicsLayerStop10_1::AssociatedLayerUnsupported { class_id, key });
            AssociatedLayer10_1::Null
        }
    };

    // 5. IElementCollection2 槽位（仅当关联图层槽无新对象体时到达）。新
    //    ElementCollection 对象读取其闭合体（element_collection 模块）；其他
    //    类停止并回退到槽位起点。
    let mut element_collection = None;
    if stop.is_none() {
        let slot_offset = trial.position();
        let saved_table = trial_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        let mut slot_stop = None;
        let slot = match entry {
            ObjectEntry10_1::Null => ElementCollectionSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                ElementCollectionSlot10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key: _ }
                if class_id == ELEMENT_COLLECTION_CLASS_ID =>
            {
                let body = read_element_collection_10_1(&mut trial, max_bytes, &mut trial_table)?;
                slot_stop = body
                    .stop
                    .clone()
                    .map(FdoGraphicsLayerStop10_1::ElementCollectionStopped);
                ElementCollectionSlot10_1::ElementCollection(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, slot_offset);
                slot_stop =
                    Some(FdoGraphicsLayerStop10_1::ElementCollectionNewObject { class_id, key });
                ElementCollectionSlot10_1::Unsupported { class_id, key }
            }
        };
        element_collection = Some(slot);
        if let Some(reason) = slot_stop {
            stop = Some(reason);
        }
    }

    // 6. 委派子对象体（this+0xbc 虚表 +0x14 的 Load，无对象头）：按
    //    FeatureLayer 字段序解析（样本佐证，见模块文档）。截断（UnexpectedEof）
    //    同样向上传播为错误。
    let mut delegated_layer = None;
    if stop.is_none() {
        let body_offset = trial.position();
        match read_feature_layer_prefix_10_1(&mut trial, max_bytes, &mut trial_table) {
            Ok(prefix) => {
                stop = Some(FdoGraphicsLayerStop10_1::DelegatedLayerBodyStopped {
                    layer_stop: prefix.stop.clone(),
                });
                delegated_layer = Some(Box::new(prefix));
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                return Err(error);
            }
            Err(error) => {
                // 内容性失败（如数据源为未支持的新类）：回退到
                // 委派体起点，体保留未知。FeatureLayer 前缀读取器失败时不改登
                // 记表，无需回退表。
                trial = reposition(&origin, entry_offset, body_offset);
                stop = Some(FdoGraphicsLayerStop10_1::DelegatedLayerBodyFailed {
                    reason: error.to_string(),
                });
            }
        }
    }

    // 委派体完整消费后也停在尚未接线的步骤 7；体内停止时则不能跳过未知体。
    let stop = stop.expect("主分支各路径必设置停止原因");

    let result = FdoGraphicsLayerPrefix10_1 {
        version,
        int32_field,
        short_field,
        associated_layer,
        element_collection,
        delegated_layer,
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
