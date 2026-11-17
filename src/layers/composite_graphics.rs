//! ArcMap 10.1 CompositeGraphicsLayer（复合图形层）对象体已确认前缀的严格
//! 读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`「CompositeGraphicsLayer::Load
//! （0x6f6b2100）」节（2026-11-14 第四轮）、「未解决事项（第五轮新增）」
//! 与「字段序闭合补充（2026-11-13 第二轮）」。字段序以 FeatureLayer.dll
//! 10.1.0.3035 的 `CompositeGraphicsLayer::Load`（`0x6f6b2100`，RTTI
//! `.?AVCompositeGraphicsLayer@@` → COL `0x6f7b4624` → 虚表 `0x6f7aa318`
//! +0x14）反编译为准（研究产物 `/tmp/r3/annolayer_work/cgl_load_6f6b2100.c`，
//! 见证据文档「证据与复现」节）；本模块实现时逐行复核了该反编译，与文档
//! 节摘要有差异的字段在下方逐处标注。
//!
//! 已确认字段序（编号 = 流内顺序）：
//!
//! 1. int16 版本：原生以有符号 short 比较 `ver < 7` 接受（>6 拒绝）。符
//!    号性未在文档闭合，本读取器按无符号解释，负值位模式一并拒绝，较原
//!    生更严；
//! 2. int32 元素组计数（原生按有符号比较，<=0 不迭代）+ 每组
//!    [int32 元素计数（同样有符号比较）+ 每元素一个嵌套对象（请求 IID
//!    `DAT_6f7aa4fc`={9B001319} IElement，esriCarto 类型库定名）]；
//! 3. AfCore String 名称 → obj+0x20；
//! 4. int16 ×3 → +0x34/+0x36/+0x38；
//! 5. double ×2 → +0x24/+0x2c；
//! 6. int32 ×3 → +0x4c、+0x50、读入局部变量。[文档节摘要有误：反编译为
//!    连续三个 int32，第三个读入局部变量后未见消费；样本 `/Maps:0x17024`
//!    起的 `3,1,1` 与其后对象头逐字节印证这一布局]；
//! 7. 嵌套对象 ×1（请求 IID `DAT_6f7aa58c`={CE8F3971} IElementCollection，
//!    esriCarto 类型库定名，2026-11-15）→ +0x1c。样本中该对象是
//!    ElementCollection `{CE8F3972-E9BE-11D1-A232-080009B6F22B}` coclass
//!    （请求 IID 与 coclass CLSID 同族 ±1）。新对象按类标识分派：与
//!    [`ELEMENT_COLLECTION_CLASS_ID`] 相同时经
//!    [`crate::objects::element_collection::read_element_collection_10_1`]
//!    整体解析其体（体内停止透传为
//!    [`CompositeGraphicsLayerStop10_1::NestedCollectionIncomplete`]，游标
//!    停在内层停止点，键保留登记；体完整消费则继续第 8 步）；其余类维持
//!    未知类停止回退（[`CompositeGraphicsLayerStop10_1::NestedNewObject`]）；
//! 8. 版本 >1：double → +0x54 + int32（读入局部变量）；
//! 9. [反编译此处调用 `FUN_6f6b3100()`，无参数——流对象是本函数局部变量，
//!    按不消费流处理，未验证] int32 子图层计数（有符号比较，<=0 不迭代）
//!    + 每项：AfCore String 条目名 + 嵌套对象（请求 IID
//!    {34C20002-4D3C-11D0-92D8-00805F7C28B0}=ILayer，第五轮候选转确认）
//!    + 条目尾段 short ×3（→ 新记录 +0x60/+0x62/+0x64）、double ×2
//!    （→ +0x68/+0x70）、int32 ×2（→ +0x7c/+0x80）、嵌套对象
//!    （`DAT_6f7aa58c` → +0x94）、版本 >1 再读 double（→ +0x88）。
//!    [条目尾段在文档节摘要中未列出，第五轮「未解决事项」3 已记载
//!    「short×3+double×2+int×2+LoadObject+double，CGL::Load 已有」]；
//! 10. int32 + ILayer 对象槽 + 版本 >2 的对象槽 + 版本 >=4 的 String；
//!     版本 ==4 再读 short，版本 >=5 再读两个 double。未知对象在对象头停
//!     止；各字段读完返回 Completed。写侧 FeatureLayer.dll 0x6f6b2620 与
//!     10.1 样本尾段对位见 docs/mxd-layer-label-cgl-20261117.md。
//!
//! 停止语义（与 [`crate::maps::map_object`] 的设计差异及理由）：map_object 对未
//! 知嵌套类直接报错，因为其后没有同序字段可继续；本模块面对的嵌套对象
//! （元素、条目 ILayer、ElementCollection 等）之后仍有已确认字段，而新对
//! 象体没有外部长度字段、无法跳过——报错会丢弃已解析前缀。因此本读取器
//! 是「已确认前缀」语义：解析到首个不可闭合字段（新对象体 / 尾段）为止，
//! 返回已解析字段 + 剩余字节数 + 完成或停止原因
//! [`CompositeGraphicsLayerStop10_1`]，不因未知嵌套类报错中断。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`COMPOSITE_GRAPHICS_LAYER_CLASS_ID`]，从对象头之后的位置调用
//! [`read_composite_graphics_layer_prefix_10_1`]；本模块不重复读取对象头。
//!
//! 样本核对（辅助观察，非测试依赖；2026-11-15 分派接线后重走）：`samples/
//! 测试底图.mxd` 的 `/Maps` 导出中，对象头 @`0x16fba` 类标识与本常量逐字节
//! 相同、键 1553；体 @`0x16fce` 起逐段吻合：版本 6、元素组计数 11 且组内
//! 计数全 0（44 字节零）、名称 `<默认>`（长度 10，@`0x17000`）、short
//! `0xffff/0xffff/0x0000`（@`0x1700e`）、double `0.0/0.0`（@`0x17014`）、
//! 三个 int32 `3,1,1`（@`0x17024`）、第 7 步对象头 @`0x17030` 为
//! ElementCollection 键 1554（新对象，经分派整体解析：体 = 版本 4 + 计数
//! 0 共 6 字节，@`0x17044..0x1704a`，完整消费、无停止）→ 第 8 步 double+
//! int32（@`0x1704a`）→ 第 9 步子图层计数 @`0x17056` = 10（样本字节支持
//! 第六轮裁决：`0x17052` 的 1 是第 8 步 int32）→ 条目 1 = String `JZD1 注
//! 记`（@`0x1705a`，16 字节声明）+ FeatureLayer 对象头 @`0x1706e` 键 1555
//! 新对象；现按 FeatureLayer 分派。完整对象登记上下文中，10 条子层
//! 均可消费，随后在循环后尾段停止。文档第四节转录的条目名「MZD1 」与当前样本字节不符
//! （实际为 `JZD1 注记`）。本次核对基于 2026-11-14 的样本导出（与第四轮
//! 研究副本在 CGL 区域外有 157 字节字符串差异；CGL 区域内逐字节一致）。
//! 样本吻合为辅助观察，未在目标软件中验证。
//!
//! 未解决问题（接线轮更新）：第 2 步元素循环与第 9 步条目尾段嵌套对象已
//! 接入 ElementCollection 分派（第 7 步同模式，体内停止透传
//! `ElementCollectionIncomplete`/`SubLayerTailCollectionIncomplete`，落位记
//! 录见 docs/arcmap-10.1-map-object.md 文末「接线轮」节）；第 6 步第三个
//! int32、第 8 步 int32、条目尾段字段的语义未定性；`FUN_6f6b3100` 运行时行为未验证；版本 short 符号性未闭合（本读取器按无符号更严处理）；条目尾段
//! 与单元 FeatureLayer 体结束边界的逐字节对位未完成（第五轮「未解决事项」
//! 3）。人工目视验收未做，不声称已验证。

use crate::layers::feature_layer::{
    FeatureLayerPrefix10_1, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
};
use crate::layers::feature_layer_tail::SuccessorStop10_1;
use crate::layers::layer_list::FEATURE_LAYER_CLASS_ID;
use crate::objects::element_body::{
    ElementBody10_1, ElementBodyStop10_1, LINE_ELEMENT_CLASS_ID, MARKER_ELEMENT_CLASS_ID,
    TEXT_ELEMENT_CLASS_ID, read_element_body_of_class_10_1,
};
use crate::objects::element_collection::{
    ELEMENT_COLLECTION_CLASS_ID, ElementCollection10_1, ElementCollectionStop10_1,
    read_element_collection_10_1,
};
use crate::objects::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use crate::objects::spatial_reference_slots::{
    GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, GeographicCoordinateSystem10_1,
    UNKNOWN_COORDINATE_SYSTEM_CLASS_ID, UnknownCoordinateSystem10_1,
    read_geographic_coordinate_system_10_1, read_unknown_coordinate_system_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// CompositeGraphicsLayer::Load 接受的最大版本：原生 `ver < 7`（有符号），
/// 即 >6 拒绝；样本值 6。
const MAX_ARCHITECTURE_VERSION: u16 = 6;

/// CompositeGraphicsLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、
/// Data4 原序）。
///
/// GUID 字符串形式为 `{9646BB83-9512-11D2-A2F6-080009B6F22B}`。证据：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 CompositeGraphicsLayer
/// （模块 FeatureLayer.dll）；FeatureLayer.dll RTTI 存在
/// `.?AVCompositeGraphicsLayer@@`，其 IPersistStream::Load 即 `0x6f6b2100`；
/// 参考样本 `/Maps:0x16fba` 的对象头类标识与本常量逐字节相同（键 1553）。
pub const COMPOSITE_GRAPHICS_LAYER_CLASS_ID: [u8; 16] = [
    0x83, 0xbb, 0x46, 0x96, 0x12, 0x95, 0xd2, 0x11, 0xa2, 0xf6, 0x08, 0x00, 0x09, 0xb6, 0xf2, 0x2b,
];

/// 已消费的嵌套对象条目；已支持的子图层新对象保留其解析体。
#[derive(Debug, Clone, PartialEq)]
pub enum ConsumedEntry10_1 {
    /// 16 字节全零类标识的空引用：无键、无对象体。
    Null,
    /// 键已在此前登记：回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新投影坐标系对象及已登记键。
    Projected {
        key: u32,
        body: Box<ProjectedCoordinateSystem10_1>,
    },
    /// 新地理坐标系对象及已登记键。
    Geographic {
        key: u32,
        body: Box<GeographicCoordinateSystem10_1>,
    },
    /// 新未知坐标系对象及已登记键。
    UnknownSpatialReference {
        key: u32,
        body: Box<UnknownCoordinateSystem10_1>,
    },
    /// 已完整读取的 FeatureLayer 子图层。
    FeatureLayer {
        key: u32,
        body: Box<FeatureLayerPrefix10_1>,
    },
}
/// 一个元素组（字段序第 2 步循环）。
#[derive(Debug, Clone, PartialEq)]
pub struct ElementGroup10_1 {
    /// 组内元素计数的原始位模式；原生按有符号比较，<=0 不迭代。
    pub declared_count: u32,
    /// 已成功分类的元素条目；触发
    /// [`CompositeGraphicsLayerStop10_1::ElementNewObject`] 停止的元素不
    /// 记录，该组整体不记录。
    pub elements: Vec<ElementEntry10_1>,
}

/// 第 2 步元素组循环单个条目（请求 IID `DAT_6f7aa4fc`={9B001319} IElement）
/// 的分类结果（接线轮起 ElementCollection 新对象按类标识整体解析其体，与
/// 第 7 步同模式；落位记录见 docs/arcmap-10.1-map-object.md 文末「接线轮」
/// 节）。
#[derive(Debug, Clone, PartialEq)]
pub enum ElementEntry10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 ElementCollection 对象（类标识经 [`ELEMENT_COLLECTION_CLASS_ID`]
    /// 校验）：已解析体；体内停止经
    /// [`CompositeGraphicsLayerStop10_1::ElementCollectionIncomplete`] 透
    /// 传（键按「登记先于持久化」语义保留，游标停在内层停止点）。
    Collection(Box<ElementCollection10_1>),
    /// 已解析的 IElement 实现类对象体。
    Element { key: u32, body: ElementBody10_1 },
}

/// 一个子图层条目（字段序第 9 步循环）。
#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerEntry10_1 {
    /// 条目名称（AfCore String，样本 `JZD1 注记`）。
    pub name: String,
    /// 条目的 ILayer 对象（请求 IID {34C20002-…}）分类。
    pub layer: ConsumedEntry10_1,
    /// 条目尾段 short ×3（→ 新记录 +0x60/+0x62/+0x64），语义未定性。
    pub tail_shorts: [u16; 3],
    /// 条目尾段 double ×2（→ +0x68/+0x70），原样保留。
    pub tail_doubles: [f64; 2],
    /// 条目尾段 int32 ×2（→ +0x7c/+0x80），语义未定性。
    pub tail_int32s: [u32; 2],
    /// 条目尾段嵌套对象（`DAT_6f7aa58c` → +0x94）分类（接线轮起
    /// ElementCollection 新对象整体解析，体入
    /// [`SubLayerEntry10_1::tail_nested_collection`]）。`None` 表示本读取器
    /// 在该对象处停止（见停止原因
    /// [`CompositeGraphicsLayerStop10_1::SubLayerTailNewObject`] 或
    /// [`CompositeGraphicsLayerStop10_1::SubLayerTailCollectionIncomplete`]），
    /// 此时 `tail_double` 亦为 `None`。
    pub tail_nested: Option<SubLayerTailSlot10_1>,
    /// 条目尾段嵌套对象为 ElementCollection（`{CE8F3972}`）新对象时的已解
    /// 析体（键已登记；体内停止经
    /// [`CompositeGraphicsLayerStop10_1::SubLayerTailCollectionIncomplete`]
    /// 上报，本字段仍携带到停止点为止的体）。`None` = 该槽是空标记/回引/
    /// 其他类新对象。
    pub tail_nested_collection: Option<ElementCollection10_1>,
    /// 版本 >1 的尾随 double（→ +0x88）。`None` = 版本 <=1，或本条目在尾
    /// 段嵌套对象处停止。
    pub tail_double: Option<f64>,
}

/// 子图层条目尾段嵌套对象（`DAT_6f7aa58c` → +0x94）的分类结果（接线轮起
/// ElementCollection 新对象按类标识整体解析其体，与第 7 步同模式）。
#[derive(Debug, Clone, PartialEq)]
pub enum SubLayerTailSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 ElementCollection 对象（类标识经 [`ELEMENT_COLLECTION_CLASS_ID`]
    /// 校验）：体解析进
    /// [`SubLayerEntry10_1::tail_nested_collection`]；体内停止经
    /// [`CompositeGraphicsLayerStop10_1::SubLayerTailCollectionIncomplete`]
    /// 透传。本变体在槽位字段记 `None`（与第 7 步 `nested_1c` 同约定）。
    Collection { key: u32 },
}

/// 已确认前缀的停止原因；本读取器对任何成功输入都返回停止原因（见模块文档）。
#[derive(Debug, Clone, PartialEq)]
pub enum CompositeGraphicsLayerStop10_1 {
    /// 元素体内部无法继续；元素键已登记，游标停在内层停止点。
    ElementBodyIncomplete {
        group: usize,
        index: usize,
        key: u32,
        reason: ElementBodyStop10_1,
    },
    /// 元素组循环（第 2 步）中的新元素对象：其体没有外部长度字段，本读取
    /// 器无法消费。`group`/`index` 为组序号与组内序号（0 起）；游标回退到
    /// 该对象头起点。
    ElementNewObject {
        group: usize,
        index: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 第 7 步嵌套对象（请求 IID `DAT_6f7aa58c`={CE8F3971} IElementCollection，
    /// 类型库定名，2026-11-15）为 ElementCollection（`{CE8F3972}` coclass）
    /// 以外的类：其体没有外部长度字段，本读取器无法消费。停止并回退到其对
    /// 象头起点（该键不保留登记）。
    NestedNewObject { class_id: [u8; 16], key: u32 },
    /// 第 7 步嵌套对象是 ElementCollection 且其体内出现停止（原因透传，见
    /// [`ElementCollectionStop10_1`]）。游标停在内层停止点；该嵌套对象的键
    /// 已按原生语义登记（登记先于持久化）；第 8 步起的后续字段不可达。
    NestedCollectionIncomplete {
        key: u32,
        reason: ElementCollectionStop10_1,
    },
    /// 子图层条目的 ILayer 新对象：停止并回退到其头起点；`name` 为已消费的
    /// 条目字符串（该条目不进入 [`CompositeGraphicsLayerPrefix10_1::sublayers`]）。
    SubLayerNewObject {
        index: usize,
        name: String,
        class_id: [u8; 16],
        key: u32,
    },
    /// FeatureLayer 体内部停止；对象键保留，条目尾段尚不可达。
    SubLayerFeatureIncomplete {
        index: usize,
        name: String,
        key: u32,
        body: Box<FeatureLayerPrefix10_1>,
    },
    /// 子图层条目尾段嵌套对象（`DAT_6f7aa58c`）是新对象且类标识非
    /// ElementCollection：该条目以部分消费状态记录（`tail_nested`/
    /// `tail_double` 为 `None`），游标回退到其头起点。
    SubLayerTailNewObject {
        index: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 子图层条目尾段 ElementCollection 体（接线轮起类分派解析）内部出现
    /// 停止（原因透传，见 [`ElementCollectionStop10_1`]）：游标停在内层停
    /// 止点，条目键已登记，版本 >1 的尾随 double 不可达。
    SubLayerTailCollectionIncomplete {
        index: usize,
        key: u32,
        reason: ElementCollectionStop10_1,
    },
    /// 第 2 步元素循环中的 ElementCollection 体（接线轮起类分派解析）内部
    /// 出现停止（原因透传）：游标停在内层停止点，键已登记；`group`/`index`
    /// 为组序号与组内序号（0 起），其后条目与组不可达。
    ElementCollectionIncomplete {
        group: usize,
        index: usize,
        key: u32,
        reason: ElementCollectionStop10_1,
    },
    /// 尾段对象槽为未知新对象；游标回退至该对象头。
    TailNewObject {
        slot: u8,
        class_id: [u8; 16],
        key: u32,
    },
    /// 对象体的全部已证字段均已消费。
    Completed,
}

impl CompositeGraphicsLayerStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::ElementBodyIncomplete {
                group,
                index,
                key,
                reason,
            } => format!("元素组 {group} 条目 {index}（键 {key}）体内部停止：{reason:?}"),
            Self::ElementNewObject { group, index, .. } => format!(
                "元素组 {group} 的元素条目 {index} 为新对象：其体没有外部长度字段，\
                 无法消费，游标回退到该对象头起点"
            ),
            Self::NestedNewObject { .. } => {
                "第 7 步嵌套对象（DAT_6f7aa58c=IElementCollection；样本为 ElementCollection \
                 coclass）为其他类的新对象：停止并回退到其头起点"
                    .to_string()
            }
            Self::NestedCollectionIncomplete { key, reason } => format!(
                "第 7 步嵌套 ElementCollection（键 {key}）体在内部停止点停止：{reason:?}；\
                 游标停在内层停止点，其后字段不可达"
            ),
            Self::SubLayerNewObject { index, .. } => format!(
                "子图层条目 {index} 的 ILayer 为新对象：停止并回退到其头起点（该条目\
                 的名称字符串已消费但不入列表）"
            ),
            Self::SubLayerFeatureIncomplete {
                index, key, body, ..
            } => format!(
                "子图层条目 {index} 的 FeatureLayer（键 {key}）体内部停止：{:?}",
                body.stop
            ),
            Self::SubLayerTailNewObject { index, .. } => format!(
                "子图层条目 {index} 的尾段嵌套对象为新对象（类标识非 \
                 ElementCollection）：条目以部分消费状态记录，\
                 游标回退到其对象头起点"
            ),
            Self::SubLayerTailCollectionIncomplete { index, key, reason } => format!(
                "子图层条目 {index} 的尾段 ElementCollection（键 {key}）体在内部停止点停止：\
                 {reason:?}；游标停在内层停止点，版本 >1 的尾随 double 与其后条目不可达"
            ),
            Self::ElementCollectionIncomplete {
                group,
                index,
                key,
                reason,
            } => format!(
                "元素组 {group} 条目 {index} 的 ElementCollection（键 {key}）体在内部停止点停止：\
                 {reason:?}；游标停在内层停止点，其后条目与组不可达"
            ),
            Self::TailNewObject { slot, .. } => {
                format!("尾段对象槽 {slot} 为未知新对象，游标回退到对象头")
            }
            Self::Completed => "CompositeGraphicsLayer 对象体已完整消费".to_string(),
        }
    }
}

/// 子图层循环后的尾段；未到达的字段为 None。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompositeGraphicsTail10_1 {
    /// 尾部 int32 的原始位模式。
    pub int32_50: Option<u32>,
    /// ILayer 对象槽。
    pub layer: Option<ConsumedEntry10_1>,
    /// 版本大于 2 的对象槽。
    pub object_3c: Option<ConsumedEntry10_1>,
    /// 版本不低于 4 的字符串。
    pub string_b4: Option<String>,
    /// 版本等于 4 的 short。
    pub short_v4: Option<u16>,
    /// 版本不低于 5 的两个 double。
    pub doubles: Option<[f64; 2]>,
}

/// 10.1 CompositeGraphicsLayer::Load（`0x6f6b2100`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeGraphicsLayerPrefix10_1 {
    /// 2 字节版本 short 的无符号位模式，>6 一律拒绝；样本值 6。
    pub version: u16,
    /// 元素组计数的原始位模式；原生按有符号比较，<=0 不迭代。
    pub group_count: u32,
    /// 已处理的元素组；有类型的元素体内部停止时，保留该组已读取的条目。
    pub element_groups: Vec<ElementGroup10_1>,
    /// 名称（→ obj+0x20；样本 `<默认>`）。
    pub name: String,
    /// 三个 short（→ +0x34/+0x36/+0x38），语义未定性、原样保留；样本
    /// `0xffff/0xffff/0x0000`。
    pub shorts: [u16; 3],
    /// 两个 double（→ +0x24/+0x2c），原样保留；样本 `0.0/0.0`。
    pub doubles: [f64; 2],
    /// 第 6 步三个 int32 之一（→ +0x4c）；样本 3。语义未定性。
    pub int32_4c: u32,
    /// 第 6 步三个 int32 之一（→ +0x50）；样本 1。语义未定性。
    pub int32_50: u32,
    /// 第 6 步第三个 int32：反编译读入局部变量后未见消费；样本 1。
    pub int32_local: u32,
    /// 第 7 步嵌套对象（`DAT_6f7aa58c` → +0x1c）分类。当该对象是
    /// ElementCollection 新对象时本字段为 `None`（体解析进
    /// [`CompositeGraphicsLayerPrefix10_1::nested_1c_collection`]）；其余分
    /// 类照旧；新对象（非 ElementCollection）是停止原因
    /// [`CompositeGraphicsLayerStop10_1::NestedNewObject`]。
    pub nested_1c: Option<ConsumedEntry10_1>,
    /// 第 7 步嵌套对象为 ElementCollection（`{CE8F3972}`）新对象时的已解析
    /// 体（对象键已登记；体内停止经停止原因
    /// [`CompositeGraphicsLayerStop10_1::NestedCollectionIncomplete`] 上
    /// 报，本字段仍携带到停止点为止的体）。`None` = 该槽是空标记/回引/其
    /// 他类新对象（后者触发停止）。
    pub nested_1c_collection: Option<ElementCollection10_1>,
    /// 版本 >1 的 double（→ +0x54）与 int32（局部变量）。
    pub after_version1: Option<(f64, u32)>,
    /// 子图层计数的原始位模式；原生按有符号比较，<=0 不迭代。样本中存在
    /// 两种读法（模块文档「样本核对」），本读取器按反编译字段序消费、不裁
    /// 决。停止发生在读取该计数之前时保持 0（默认值）。
    pub sublayer_count: u32,
    /// 已成功处理的子图层条目。
    pub sublayers: Vec<SubLayerEntry10_1>,
    /// 子图层循环后的字段。
    pub tail: CompositeGraphicsTail10_1,
    /// 完成或停止原因。
    pub stop: CompositeGraphicsLayerStop10_1,
    /// 停止位置之后的剩余字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 CompositeGraphicsLayer 对象体的已确认前缀。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`COMPOSITE_GRAPHICS_LAYER_CLASS_ID`] 确认类身份，并从对象头之
/// 后的位置传入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额，应用于图形层名称与每个子图层条目名称的声明长
/// 度（含终止零、不含 4 字节前缀），在任何读取与分配之前检查；
/// `usize::MAX` 表示不额外设限。`table` 是跨流共享的键登记表：新对象键在
/// 登记后若本读取器无法消费其体，则停止时不保留该登记（见各停止原因）。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回才提交；
/// 任何失败都不推进调用者游标、不修改登记表。触发停止的返回是成功：游标
/// 推进到停止点（新对象停止时回退到该对象头起点，调用者可把剩余字节交给
/// 对应的对象体读取器），已消费条目登记进调用者登记表。错误位置相对于传
/// 入 Reader，调用方需另外记录流和对象身份。这是严格安全子集，不代表完
/// 整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_composite_graphics_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<CompositeGraphicsLayerPrefix10_1> {
    let entry_offset = reader.position();
    // 入口克隆：Reader 没有随机定位原语，停止回退都用「入口克隆 + 前向读
    // 取」把游标重定位到窗口内绝对偏移；所有目标都在入口之后。
    let origin = reader.clone();
    let limit = entry_offset + origin.remaining();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("CompositeGraphicsLayer 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    // 全部字段在 trial 游标与 trial 登记表上读取，成功才提交；任何失败路径
    // 都不推进原游标、不改原登记表。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let mut stop: Option<CompositeGraphicsLayerStop10_1> = None;

    // 1. 2 字节版本：无符号解释，>6 拒绝（原生 ver < 7 有符号比较）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(invalid(format!(
            "架构版本 {version} 超出 CompositeGraphicsLayer::Load 接受范围（>6 拒绝）"
        )));
    }

    // 2. 元素组计数 + 每组 [元素计数 + 每元素一个嵌套对象]。
    let group_count = trial.read_u32_le()?;
    let mut element_groups = Vec::new();
    if (group_count as i32) > 0 {
        'groups: for group in 0..group_count as i32 as usize {
            let declared_count = trial.read_u32_le()?;
            let mut elements = Vec::new();
            if (declared_count as i32) > 0 {
                for index in 0..declared_count as i32 as usize {
                    let header_offset = trial.position();
                    let saved_table = trial_table.clone();
                    let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                    match entry {
                        ObjectEntry10_1::Null => elements.push(ElementEntry10_1::Null),
                        ObjectEntry10_1::BackReference { class_id, key } => {
                            elements.push(ElementEntry10_1::BackReference { class_id, key });
                        }
                        ObjectEntry10_1::NewObject { class_id, key }
                            if [
                                TEXT_ELEMENT_CLASS_ID,
                                MARKER_ELEMENT_CLASS_ID,
                                LINE_ELEMENT_CLASS_ID,
                            ]
                            .contains(&class_id) =>
                        {
                            let body = read_element_body_of_class_10_1(
                                &class_id,
                                &mut trial,
                                max_bytes,
                                &mut trial_table,
                            )?;
                            let reason = match &body {
                                ElementBody10_1::TextElement(body) => &body.stop,
                                ElementBody10_1::MarkerElement(body) => &body.stop,
                                ElementBody10_1::LineElement(body) => &body.stop,
                            }
                            .clone();
                            elements.push(ElementEntry10_1::Element { key, body });
                            if let Some(reason) = reason {
                                stop =
                                    Some(CompositeGraphicsLayerStop10_1::ElementBodyIncomplete {
                                        group,
                                        index,
                                        key,
                                        reason,
                                    });
                                element_groups.push(ElementGroup10_1 {
                                    declared_count,
                                    elements,
                                });
                                break 'groups;
                            }
                        }
                        ObjectEntry10_1::NewObject { class_id, key }
                            if class_id == ELEMENT_COLLECTION_CLASS_ID =>
                        {
                            // 接线轮：ElementCollection 新对象整体解析其体
                            // （与第 7 步同模式；键已由条目分类按「登记先于
                            // 持久化」语义登记，体内停止透传为新停止变体，
                            // 游标停在内层停止点，其后条目不可达）。
                            let collection = read_element_collection_10_1(
                                &mut trial,
                                max_bytes,
                                &mut trial_table,
                            )?;
                            if let Some(reason) = collection.stop.clone() {
                                stop = Some(
                                    CompositeGraphicsLayerStop10_1::ElementCollectionIncomplete {
                                        group,
                                        index,
                                        key,
                                        reason,
                                    },
                                );
                            }
                            elements.push(ElementEntry10_1::Collection(Box::new(collection)));
                            if stop.is_some() {
                                break 'groups;
                            }
                        }
                        ObjectEntry10_1::NewObject { class_id, key } => {
                            // 新元素体没有外部长度字段，本读取器无法消费：回退
                            // 登记与游标到该对象头起点，在此停止，不报错。
                            trial_table = saved_table;
                            trial = reposition(&origin, entry_offset, header_offset, limit)
                                .expect("回退目标在停止前的已读位置，必在窗口内");
                            stop = Some(CompositeGraphicsLayerStop10_1::ElementNewObject {
                                group,
                                index,
                                class_id,
                                key,
                            });
                            break 'groups;
                        }
                    }
                }
            }
            element_groups.push(ElementGroup10_1 {
                declared_count,
                elements,
            });
        }
    }

    // 元素循环若已停止，以下字段全部不属于已确认前缀（游标已回退到停止点），
    // 一律保持默认值。
    let mut name = String::new();
    let mut shorts = [0_u16; 3];
    let mut doubles = [0.0; 2];
    let mut int32_4c = 0;
    let mut int32_50 = 0;
    let mut int32_local = 0;
    let mut nested_1c = None;
    let mut nested_1c_collection = None;
    let mut after_version1 = None;
    let mut sublayer_count = 0;
    let mut sublayers = Vec::new();
    if stop.is_none() {
        // 3. 名称：AfCore String 布局，受 max_bytes 限额约束。
        name = read_string_10_1(&mut trial, max_bytes)?;

        // 4. 三个 short：语义未定性，原样保留。
        for slot in &mut shorts {
            *slot = trial.read_u16_le()?;
        }

        // 5. 两个 double：原样保留。
        for slot in &mut doubles {
            *slot = trial.read_f64_le()?;
        }

        // 6. 连续三个 int32（文档节摘要记为一个，反编译与样本均为三个）。
        int32_4c = trial.read_u32_le()?;
        int32_50 = trial.read_u32_le()?;
        int32_local = trial.read_u32_le()?;

        // 7. 嵌套对象（DAT_6f7aa58c={CE8F3971} IElementCollection → +0x1c；
        //    样本为 ElementCollection {CE8F3972} coclass）。新对象按类标识
        //    分派：ElementCollection 整体解析其体并透传体内停止；其余类维
        //    持未知类停止回退语义。
        let header_offset = trial.position();
        let saved_table = trial_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        match entry {
            ObjectEntry10_1::Null => nested_1c = Some(ConsumedEntry10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                nested_1c = Some(ConsumedEntry10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == ELEMENT_COLLECTION_CLASS_ID =>
            {
                // ElementCollection 体读取器是严格事务子集（失败不推进其入
                // 参游标/登记表）；键已由条目分类登记（登记先于持久化），
                // 解析成功（含体内停止）即保留登记。体内停止透传为新停止变
                // 体，游标停在内层停止点；体完整消费则继续第 8 步。
                let collection =
                    read_element_collection_10_1(&mut trial, max_bytes, &mut trial_table)?;
                if let Some(reason) = collection.stop.clone() {
                    stop = Some(CompositeGraphicsLayerStop10_1::NestedCollectionIncomplete {
                        key,
                        reason,
                    });
                }
                nested_1c_collection = Some(collection);
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, header_offset, limit)
                    .expect("回退目标在停止前的已读位置，必在窗口内");
                stop = Some(CompositeGraphicsLayerStop10_1::NestedNewObject { class_id, key });
            }
        }

        // 8. 版本 >1：double（→ +0x54）+ int32（局部变量）。
        if stop.is_none() && version > 1 {
            let double = trial.read_f64_le()?;
            let int32 = trial.read_u32_le()?;
            after_version1 = Some((double, int32));
        }

        // 9. [反编译此处调用无参数的 FUN_6f6b3100，按不消费流处理（见模块
        //    文档）] 子图层计数 + 每项 [String + ILayer + 条目尾段]。
        if stop.is_none() {
            sublayer_count = trial.read_u32_le()?;
            if (sublayer_count as i32) > 0 {
                'sublayers: for index in 0..sublayer_count as i32 as usize {
                    let item_name = read_string_10_1(&mut trial, max_bytes)?;
                    // 条目的 ILayer 对象。
                    let layer = {
                        let header_offset = trial.position();
                        let saved_table = trial_table.clone();
                        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                        match entry {
                            ObjectEntry10_1::Null => ConsumedEntry10_1::Null,
                            ObjectEntry10_1::BackReference { class_id, key } => {
                                ConsumedEntry10_1::BackReference { class_id, key }
                            }
                            ObjectEntry10_1::NewObject { class_id, key }
                                if class_id == FEATURE_LAYER_CLASS_ID =>
                            {
                                let body = read_feature_layer_prefix_10_1(
                                    &mut trial,
                                    max_bytes,
                                    &mut trial_table,
                                )?;
                                if body.stop
                                    != FeatureLayerStop10_1::Successor(
                                        SuccessorStop10_1::TailCompleted,
                                    )
                                {
                                    stop = Some(
                                        CompositeGraphicsLayerStop10_1::SubLayerFeatureIncomplete {
                                            index,
                                            name: item_name,
                                            key,
                                            body: Box::new(body),
                                        },
                                    );
                                    break 'sublayers;
                                }
                                ConsumedEntry10_1::FeatureLayer {
                                    key,
                                    body: Box::new(body),
                                }
                            }
                            ObjectEntry10_1::NewObject { class_id, key } => {
                                // 新对象体不可消费：回退并停止；已消费的条目字
                                // 符串保存在停止原因里，不丢弃。
                                trial_table = saved_table;
                                trial = reposition(&origin, entry_offset, header_offset, limit)
                                    .expect("回退目标在停止前的已读位置，必在窗口内");
                                stop = Some(CompositeGraphicsLayerStop10_1::SubLayerNewObject {
                                    index,
                                    name: item_name,
                                    class_id,
                                    key,
                                });
                                break 'sublayers;
                            }
                        }
                    };
                    // 条目尾段 short ×3 + double ×2 + int32 ×2。
                    let mut tail_shorts = [0_u16; 3];
                    for slot in &mut tail_shorts {
                        *slot = trial.read_u16_le()?;
                    }
                    let mut tail_doubles = [0.0; 2];
                    for slot in &mut tail_doubles {
                        *slot = trial.read_f64_le()?;
                    }
                    let mut tail_int32s = [0_u32; 2];
                    for slot in &mut tail_int32s {
                        *slot = trial.read_u32_le()?;
                    }
                    // 条目尾段嵌套对象（DAT_6f7aa58c → +0x94）：接线轮起
                    // ElementCollection 新对象按类分派整体解析（与第 7 步同
                    // 模式；键登记先于持久化，体内停止透传）。
                    let tail_header_offset = trial.position();
                    let saved_table = trial_table.clone();
                    let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                    let (tail_nested, tail_nested_collection, tail_double) = match entry {
                        ObjectEntry10_1::Null => {
                            let tail_double = read_tail_double(&mut trial, version)?;
                            (Some(SubLayerTailSlot10_1::Null), None, tail_double)
                        }
                        ObjectEntry10_1::BackReference { class_id, key } => {
                            let tail_double = read_tail_double(&mut trial, version)?;
                            (
                                Some(SubLayerTailSlot10_1::BackReference { class_id, key }),
                                None,
                                tail_double,
                            )
                        }
                        ObjectEntry10_1::NewObject { class_id, key }
                            if class_id == ELEMENT_COLLECTION_CLASS_ID =>
                        {
                            // 键已由条目分类登记；体完整消费则继续条目循环
                            // （含版本 >1 尾随 double），体内停止透传为新停
                            // 止变体（游标停在内层停止点，尾随 double 不可
                            // 达，条目由下方公共块记录）。
                            let collection = read_element_collection_10_1(
                                &mut trial,
                                max_bytes,
                                &mut trial_table,
                            )?;
                            match collection.stop.clone() {
                                Some(reason) => {
                                    stop = Some(
                                        CompositeGraphicsLayerStop10_1::SubLayerTailCollectionIncomplete {
                                            index,
                                            key,
                                            reason,
                                        },
                                    );
                                    (
                                        Some(SubLayerTailSlot10_1::Collection { key }),
                                        Some(collection),
                                        None,
                                    )
                                }
                                None => {
                                    let tail_double = read_tail_double(&mut trial, version)?;
                                    (
                                        Some(SubLayerTailSlot10_1::Collection { key }),
                                        Some(collection),
                                        tail_double,
                                    )
                                }
                            }
                        }
                        ObjectEntry10_1::NewObject { class_id, key } => {
                            // 其余新对象体不可消费：条目以部分消费状态记录，
                            // 回退并停止。
                            trial_table = saved_table;
                            trial = reposition(&origin, entry_offset, tail_header_offset, limit)
                                .expect("回退目标在停止前的已读位置，必在窗口内");
                            stop = Some(CompositeGraphicsLayerStop10_1::SubLayerTailNewObject {
                                index,
                                class_id,
                                key,
                            });
                            sublayers.push(SubLayerEntry10_1 {
                                name: item_name,
                                layer,
                                tail_shorts,
                                tail_doubles,
                                tail_int32s,
                                tail_nested: None,
                                tail_nested_collection: None,
                                tail_double: None,
                            });
                            break 'sublayers;
                        }
                    };
                    if stop.is_some() {
                        // ElementCollection 体内停止：条目以「体解析到停止点
                        // 为止」的状态记录（尾随 double 不可达），其后条目不
                        // 可达。
                        sublayers.push(SubLayerEntry10_1 {
                            name: item_name,
                            layer,
                            tail_shorts,
                            tail_doubles,
                            tail_int32s,
                            tail_nested,
                            tail_nested_collection,
                            tail_double: None,
                        });
                        break 'sublayers;
                    }
                    sublayers.push(SubLayerEntry10_1 {
                        name: item_name,
                        layer,
                        tail_shorts,
                        tail_doubles,
                        tail_int32s,
                        tail_nested,
                        tail_nested_collection,
                        tail_double,
                    });
                }
            }
        }
    }

    // 10. 未知对象体无长度，保留头起点；截断则回滚整个读取事务。
    let mut tail = CompositeGraphicsTail10_1::default();
    if stop.is_none() {
        tail.int32_50 = Some(trial.read_u32_le()?);
        for slot in 0..if version > 2 { 2 } else { 1 } {
            let offset = trial.position();
            let saved_table = trial_table.clone();
            let entry = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
                ObjectEntry10_1::Null => ConsumedEntry10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    ConsumedEntry10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if slot == 1 && class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
                {
                    ConsumedEntry10_1::Projected {
                        key,
                        body: Box::new(read_projected_coordinate_system_10_1(
                            &mut trial, max_bytes,
                        )?),
                    }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if slot == 1 && class_id == GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID =>
                {
                    ConsumedEntry10_1::Geographic {
                        key,
                        body: Box::new(read_geographic_coordinate_system_10_1(
                            &mut trial, max_bytes,
                        )?),
                    }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if slot == 1 && class_id == UNKNOWN_COORDINATE_SYSTEM_CLASS_ID =>
                {
                    ConsumedEntry10_1::UnknownSpatialReference {
                        key,
                        body: Box::new(read_unknown_coordinate_system_10_1(&mut trial, max_bytes)?),
                    }
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    trial_table = saved_table;
                    trial = reposition(&origin, entry_offset, offset, limit)?;
                    stop = Some(CompositeGraphicsLayerStop10_1::TailNewObject {
                        slot,
                        class_id,
                        key,
                    });
                    break;
                }
            };
            if slot == 0 {
                tail.layer = Some(entry);
            } else {
                tail.object_3c = Some(entry);
            }
        }
        if stop.is_none() && version >= 4 {
            tail.string_b4 = Some(read_string_10_1(&mut trial, max_bytes)?);
            if version == 4 {
                tail.short_v4 = Some(trial.read_u16_le()?);
            }
        }
        if stop.is_none() && version >= 5 {
            tail.doubles = Some([trial.read_f64_le()?, trial.read_f64_le()?]);
        }
    }
    let stop = stop.unwrap_or(CompositeGraphicsLayerStop10_1::Completed);

    let result = CompositeGraphicsLayerPrefix10_1 {
        version,
        group_count,
        element_groups,
        name,
        shorts,
        doubles,
        int32_4c,
        int32_50,
        int32_local,
        nested_1c,
        nested_1c_collection,
        after_version1,
        sublayer_count,
        sublayers,
        tail,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 读取子图层条目版本 >1 的尾随 double（→ 新记录 +0x88）。
fn read_tail_double(trial: &mut Reader<'_>, version: u16) -> io::Result<Option<f64>> {
    if version > 1 {
        Ok(Some(trial.read_f64_le()?))
    } else {
        Ok(None)
    }
}

/// 把游标重定位到「传入 Reader 窗口」的绝对偏移 `target`。
///
/// Reader 没有随机定位原语；`origin` 是函数入口时的游标克隆（位置 =
/// `entry_offset`），这里用入口克隆前向读取实现重定位。`target` 必须在
/// `entry_offset` 与 `limit`（窗口末尾）之间，越界返回错误。
fn reposition<'a>(
    origin: &Reader<'a>,
    entry_offset: usize,
    target: usize,
    limit: usize,
) -> io::Result<Reader<'a>> {
    if !(entry_offset..=limit).contains(&target) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("重定位目标 0x{target:x} 超出输入窗口 [0x{entry_offset:x}, 0x{limit:x}]"),
        ));
    }
    let mut positioned = origin.clone();
    // 长度已由上界校验限制在窗口内，前向读取不会失败。
    positioned
        .read_bytes(target - entry_offset)
        .expect("目标位置已校验在窗口内");
    Ok(positioned)
}
