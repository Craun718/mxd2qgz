//! ArcMap 10.1 Map（数据框）对象体已确认前缀与后缀序列的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`（2026-11-13/14 各轮）。字段序来自
//! Map.dll 10.1.0.3035 的 `IPersistStream::Load`（RTTI `.?AVMap@@` →
//! IPersistStream 子接口虚表 `0x794874a4` 槽 +0x14 = `0x793ff4d0`）与对称的
//! Save 主体 `0x79401850`，并与仓库参考样本 `samples/测试底图.mxd` 的 `/Maps`
//! 导出逐字节对照吻合：`/Maps:0x18` 版本 short=51、`0x1a` 名称「新建数据框」、
//! `0x2a` 空名称（`02 00 00 00 00 00`）、`0x30` CRS 对象头（键 30/0x1e）、
//! `0x44` CRS 体版本 6 与 455 字节 WKT。
//!
//! ## 已确认前缀（`read_map_prefix_10_1`；Load 第 1-4 步）
//!
//! 1. int16 架构版本（>0x33/51 直接拒绝；样本值 51）；
//! 2. 字符串 → obj+0x160（数据框名称）；
//! 3. 字符串 → obj+0x164（第二名称，样本为空串）；
//! 4. 嵌套对象 ×1（空间参考，类身份由流内 16 字节对象头决定）：样本为
//!    ProjectedCoordinateSystem；16 零字节的空头按"无空间参考"处理；其他
//!    类标识显式拒绝（对象体无外部长度字段，无法跳过未知类）。
//!
//! 字符串布局已由样本字节闭合：AfCore String::Write 普通输出，4 字节长度
//! 前缀 = 含 UTF-16LE 终止零的字节数 n*2+2。样本「新建数据框」前缀 12
//! （5 字 ×2 + 终止零 2），空名称前缀 2（仅终止零，即文档强调的 6 字节
//! `02 00 00 00 00 00` 模式，勿误读为 int32 计数）。与
//! [`crate::stream::native_string::read_string_10_1`] 的布局一致，名称读取据此
//! 接入该严格读取器。
//!
//! ## 后缀序列（`read_map_suffix_10_1`；Save `0x79401850` 与 Load 对称段）
//!
//! 前缀之后依次：**图层列表**（读侧 `FUN_794035f0` ver≥0x17，见
//! [`crate::layers::layer_list`]）→ 基本图形层对象（SaveObject(+0xf8)，请求 IID
//! `{34B2EF81-…}`；样本为 CompositeGraphicsLayer 键 1553 @/Maps:0x16fba）→
//! int32 计数 + 循环（MapSurround，请求 IID `{FE3115E9-…}`；样本计数 0
//! @/Maps:0x24f60）→ **WKSEnvelope 32 字节**（`[XMin,YMin,XMax,YMax]`，写方向
//! 确定；样本 @0x24f64）→ int32、int32（用途未定性；样本 9、9 @0x24f84）→
//! IEnvelope 对象（请求 IID `{1B077914-…}`；样本为空标记）→ int32 计数 + 循环
//! （书签，请求 IID `{EE53528B-…}`；参考样本计数 0 @/Maps:0x24f9c）→
//! 可确认的 18 字节属性前导：一个 `0xffff` short、两个原始成员 short 与
//! 其间的 double，最后为独立表计数。独立表集合为空时还可读取照明属性槽
//! 和随后的定长成员；坐标变换循环位于更后面。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`] 读取
//! Map 对象头并比对 [`MAP_CLASS_ID`]，从对象头之后的位置调用
//! [`read_map_prefix_10_1`]；本模块不重复读取对象头。后缀读取器同样从对象体
//! 起点接收 Reader（内部重读前缀以登记嵌套空间参考键并定位图层列表），
//! 并要求调用者提供跨流共享的键登记表（Map 对象自身的键由调用者在读取
//! Map 对象头时登记）。
//!
//! 样本核对（2026-11-17）：参考文件从图层列表继续读完基本图形层，读到
//! 0x24f64 处的 WKSEnvelope、其后 int32 对 (9, 9)、IEnvelope 空标记和
//! 书签计数 0；属性前导从 0x24fa0 开始，照明属性槽从 0x24fb2 开始。
//! IEnvelope 空标记消费 16 字节，与书签计数
//! 和下一字段的偏移互证。尚未在 ArcMap 中运行验证。
//!
//! 这是仅覆盖上述已确认字段的严格安全子集：要求完整输入、任何失败不推进
//! 调用者游标、不修改共享登记表。停止点（如标志 1 新对象体、未接入读取器的
//! 新对象体）是成功返回：已闭合部分 + 剩余字节 + 停止原因，与错误区分。
//! 因此不代表完整 Map 体行为，未接入完整 MXD 解析，人工目视验收未做。
//!
//! 未解决问题（详见证据文档「未解决事项」）：WKSEnvelope 之后两个 int32 的
//! 用途未定性；MapSurround/书签循环元素的 coclass 语义开放（样本计数均为 0，
//! 非空循环未验证）；版本 short 比较的符号性未在文档闭合（本读取器按无符号
//! 解释，负值位模式一并拒绝，较有符号原生更严，与 spatial_reference 的版本
//! 处理一致）；IEnvelope 槽位空标记的消费长度见
//! [`read_map_suffix_10_1`] 内注释（样本 16 字节 + 紧随计数 0 与文档计数偏移
//! 0x24f9c 互证；任务摘要曾按 20 字节解读，与样本 0x24fa0=0xffff 矛盾）。

use crate::layers::composite_graphics::{
    COMPOSITE_GRAPHICS_LAYER_CLASS_ID, CompositeGraphicsLayerPrefix10_1,
    CompositeGraphicsLayerStop10_1, read_composite_graphics_layer_prefix_10_1,
};
use crate::layers::layer_list::{
    LayerBodyPrefix10_1, LayerEntryObject, LayerList10_1, LayerListStop, class_id_hex,
    read_layer_list_10_1, read_legacy_layer_lists_10_1,
};
use crate::maps::layer_masking::{
    LayerMasking10_1, LayerMaskingSupport10_1, MaskEntry10_1, known_layer_masking_support_10_1,
    read_entries_10_1, read_layer_masking_10_1, read_masking_count_10_1, read_masking_version_10_1,
};
use crate::maps::map_decor::read_map_decor_slot_10_1;
use crate::maps::map_late_tail::{LateMapTail10_1, read_map_late_tail_10_1};
use crate::maps::map_persist_collections::{
    MapPersistCollections10_1, read_map_persist_collections_before_v34_10_1,
};
use crate::maps::map_surround::{
    AOI_BOOKMARK_CLASS_ID, AoiBookmark10_1, FEATURE_BOOKMARK_CLASS_ID, FeatureBookmark10_1,
    LEGEND_CLASS_ID, LegendPrefix10_1, LegendStop10_1, read_aoi_bookmark_10_1,
    read_feature_bookmark_10_1, read_legend_prefix_10_1,
};
use crate::maps::map_surround_family::{
    ALTERNATING_SCALEBAR_CLASS_ID, DOUBLE_ALTERNATING_SCALEBAR_CLASS_ID, FamilyStop10_1,
    HOLLOW_SCALEBAR_CLASS_ID, MAP_TITLE_CLASS_ID, MARKER_NORTH_ARROW_CLASS_ID, MapTitleBody10_1,
    MarkerNorthArrowBody10_1, SCALE_LINE_CLASS_ID, SCALE_TEXT_CLASS_ID, SCALEBAR_CLASS_ID,
    SINGLE_DIVISION_SCALEBAR_CLASS_ID, STEPPED_SCALE_LINE_CLASS_ID, ScaleLineBody10_1,
    ScaleLineStop10_1, ScaleTextBody10_1, ScalebarBody10_1, ScalebarSubclassBody10_1,
    ScalebarSubclassKind10_1, read_map_title_body_10_1, read_marker_north_arrow_body_10_1,
    read_scale_line_10_1, read_scale_text_body_10_1, read_scalebar_body_10_1,
    read_scalebar_subclass_body_10_1, read_stepped_scale_line_10_1,
};
use crate::maps::maplex::{
    MAPLEX_ANNOTATE_MAP_CLASS_ID, MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID, MaplexAnnotateMap10_1,
    MaplexOverposterProperties10_1, read_annotate_map as read_maplex_annotate_map,
    read_overposter_properties as read_maplex_overposter_properties,
};
use crate::maps::standalone_table::{
    STANDALONE_TABLE_CLASS_ID, StandaloneTablePrefix10_1, StandaloneTableStop10_1,
    read_standalone_table_prefix_10_1,
};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_tabular_name_slot_10_1};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::element_decor::{DecorKind10_1, DecorSlot10_1};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID,
    POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID, read_geometry_body_10_1,
};
use crate::objects::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// Map::Load 接受的最大架构版本：>0x33（51）直接拒绝；样本值 51。
const MAX_ARCHITECTURE_VERSION: u16 = 51;

/// 后缀中 MapSurround / 书签循环单个条目的最小字节数（16 零空标记）。
const MIN_LOOP_ENTRY_BYTES: usize = 16;

/// WKSEnvelope 32 字节原始读取结果（非对象；AfCore 写方向确定的字段序）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WksEnvelope10_1 {
    /// 第 1 个 double（AfCore `?XMin@WKSEnvelope` 读结构 +0x00，写方向确定）。
    pub xmin: f64,
    /// 第 2 个 double（结构 +0x08）。
    pub ymin: f64,
    /// 第 3 个 double（结构 +0x10）。
    pub xmax: f64,
    /// 第 4 个 double（结构 +0x18）。
    pub ymax: f64,
}

/// 后缀中一个对象槽位（基本图形层 / MapSurround 条目 / IEnvelope / 书签条目）
/// 的分类结果与流内偏移。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectSlot10_1 {
    /// 对象头起点偏移（相对传入 Reader 的底层字节）。
    pub offset: usize,
    /// 经 [`crate::storage::object_refs`] 分类的对象条目（空标记 / 回引 / 新对象）。
    pub entry: ObjectEntry10_1,
}

impl ObjectSlot10_1 {
    /// 单行诊断描述：分类、类标识十六进制与键。
    pub fn describe(&self) -> String {
        match &self.entry {
            ObjectEntry10_1::Null => "空标记（16 零类标识）".to_string(),
            ObjectEntry10_1::BackReference { class_id, key } => {
                format!("回引 类={} 键 {key}", class_id_hex(class_id))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                format!("新对象 类={} 键 {key}", class_id_hex(class_id))
            }
        }
    }
}

/// 后缀读取的停止原因（非错误：已消费内容有效，其后保留为未知尾部）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapSuffixStop {
    /// 原生版本门所需的全部对象体字段已消费，游标位于下一个根流字段。
    Complete,
    /// 图层列表在标志 1 新对象体处停止（体无外部长度、读取器未接入；集成点）。
    /// 明细见 [`LayerList10_1::stop`]。
    LayerListFlag1Body {
        /// 图层列表内的条目序号。
        index: usize,
        /// 条目起点（标志处）偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    LegacyLayerBody {
        index: usize,
        offset: usize,
    },
    LegacyTableName {
        offset: usize,
    },
    /// 基本图形层对象为新对象：该槽位对象体无外部长度，无法跳过。类标识为
    /// CompositeGraphicsLayer 时已解析已确认体前缀
    /// （[`MapSuffix10_1::base_graphics_layer_parsed`]），游标停在其停止点
    /// （嵌套新对象处已回退到该对象头起点）；其余情况停在对象头之后。
    BaseGraphicsLayerBody {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// MapSurround 循环条目为新对象：体无外部长度且读取器未接入。
    MapSurroundBody {
        index: usize,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// IEnvelope 槽位对象为新对象且未被 Envelope 读取器整体消费（非
    /// Envelope 类，或 Envelope 体解析失败）：体无外部长度，无法跳过。
    EnvelopeObjectBody {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// MapSurround 循环条目为族内新对象（Scalebar 族/ScaleText/
    /// MarkerNorthArrow）且其体内出现停止（原因透传）：游标停在其内层
    /// 停止点。
    MapSurroundFamilyBody {
        /// 条目序号（0 起）。
        index: usize,
        /// 条目对象头起点偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
        /// 族读取器带出的体内停止原因。
        reason: FamilyStop10_1,
    },
    /// 比例尺线体内尚未闭合的槽。
    ScaleLineBody {
        index: usize,
        reason: ScaleLineStop10_1,
    },
    /// 书签循环条目为新对象：体无外部长度且读取器未接入。
    BookmarkBody {
        index: usize,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 书签之后未读到完整的属性前导，余下原样保留。
    AfterBookmarks,
    /// 已读属性前导，停止于独立表集合或照明属性槽之前。
    AfterPostBookmarkPrefix {
        standalone_table_count: i32,
    },
    /// 独立表集合尚未完整消费。
    StandaloneTableBody {
        offset: usize,
    },
    /// 独立表集合已消费；下一槽为照明属性。
    AfterStandaloneTables,
    /// 已读照明属性槽，后续 Map 字段仍保留为未知。
    AfterIllumination,
    /// 照明后的定长字段已读，下一段 Map 字段保留为未知。
    AfterIlluminationFields,
    /// 照明后几何槽为空，停止于 IBorder 槽之前。
    AfterNullGeometry,
    /// 照明后非空几何槽已读取，停止于 IBorder 槽之前。
    AfterGeometry,
    /// 照明后几何及边框槽已完整读取。
    AfterNullBorder,
    /// 边框后版本块中已确认字段的停止阶段。
    AfterBorderTail {
        stage: u8,
    },
}

impl MapSuffixStop {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(self) -> String {
        match self {
            Self::Complete => "Map 对象体已完整消费".into(),
            Self::LayerListFlag1Body { index, offset, .. } => format!(
                "图层列表条目 {index}（offset 0x{offset:x}）为标志 1 新对象：体无外部\
                 长度，类分派读取器只覆盖已确认前缀，停止"
            ),
            Self::LegacyLayerBody { index, offset } => {
                format!("旧图层列表条目 {index}（offset 0x{offset:x}）对象体尚未读完")
            }
            Self::LegacyTableName { offset } => {
                format!("旧表 IName 条目未读完（offset 0x{offset:x}）")
            }
            Self::BaseGraphicsLayerBody { offset, .. } => format!(
                "基本图形层对象（offset 0x{offset:x}）为新对象：体无外部长度；\
                 CompositeGraphicsLayer 已解析体前缀时游标停在其停止点，否则停在\
                 对象头之后"
            ),
            Self::MapSurroundBody { index, offset, .. } => format!(
                "MapSurround 条目 {index}（offset 0x{offset:x}）为新对象且未被读取器\
                 整体消费（类没有读取器，或 Legend 体内停止/解析失败），停止"
            ),
            Self::EnvelopeObjectBody { offset, .. } => format!(
                "IEnvelope 槽位对象（offset 0x{offset:x}）为新对象且未被 Envelope 读取\
                 器整体消费（非 Envelope 类或解析失败）：体无外部长度，停止"
            ),
            Self::MapSurroundFamilyBody { index, reason, .. } => format!(
                "MapSurround 条目 {index}（族装饰物）体在停止点停止：{}；游标停在其停止点",
                reason.describe()
            ),
            Self::ScaleLineBody { index, reason } => {
                format!("MapSurround 条目 {index} 的比例尺线体内停止：{reason:?}")
            }
            Self::BookmarkBody { index, offset, .. } => format!(
                "书签条目 {index}（offset 0x{offset:x}）为新对象且未被读取器整体消费\
                （类没有读取器或解析失败），停止"
            ),
            Self::AfterIllumination => "照明属性槽已读；后续 Map 字段保留为未知".to_string(),
            Self::AfterIlluminationFields => {
                "照明后定长字段已读；后续 Map 字段保留为未知".to_string()
            }
            Self::AfterNullGeometry => "IGeometry 槽为空；从 IBorder 槽起保留未知".to_string(),
            Self::AfterGeometry => "IGeometry 槽已读；从 IBorder 槽起保留未知".to_string(),
            Self::AfterNullBorder => {
                "IGeometry 和 IBorder 槽已读取；后续 Map 字段保留为未知".to_string()
            }
            Self::AfterBorderTail { stage } => {
                format!("IBorder 后版本块已读至阶段 {stage}；后续字段保留为未知")
            }
            Self::AfterBookmarks => {
                "书签循环之后未匹配完整的保存侧属性前导，余下原样保留".to_string()
            }
            Self::AfterPostBookmarkPrefix {
                standalone_table_count: 0,
            } => "书签后的属性前导已读，独立表集合为空；从照明属性槽起保留未知".to_string(),
            Self::AfterPostBookmarkPrefix {
                standalone_table_count,
            } => {
                format!("书签后的属性前导已读；从 {standalone_table_count} 条独立表记录起保留未知")
            }
            Self::StandaloneTableBody { offset } => {
                format!("独立表未闭合；从 offset 0x{offset:x} 起保留未知")
            }
            Self::AfterStandaloneTables => "独立表集合已读；从照明槽起保留未知".to_string(),
        }
    }
}

/// MapSurround 循环条目中已整体消费的元素体。
///
/// C 轮接线起 MapSurround 装饰物族（MapTitle/Scalebar/ScaleText/
/// MarkerNorthArrow + 四个比例尺子类）经 map_surround_family 读取器整体
/// 消费；体内停止经 [`MapSuffixStop::MapSurroundFamilyBody`] 透传。
#[derive(Debug, Clone, PartialEq)]
pub enum MapSurroundElement10_1 {
    /// Legend 已确认前缀（Legend 前缀读取器在 ILegendFormat / ILegendItem
    /// 新对象等体内停止点停止时 `stop` 携带原因；此时后缀循环不继续）。
    Legend(Box<LegendPrefix10_1>),
    /// MapTitle（体完全闭合：自身版本 + 共同前缀）。
    MapTitle(Box<MapTitleBody10_1>),
    /// Scalebar（体内停止经其 `stop` 字段携带）。
    Scalebar(Box<ScalebarBody10_1>),
    /// ScaleText（体内停止经其 `stop` 字段携带）。
    ScaleText(Box<ScaleTextBody10_1>),
    /// MarkerNorthArrow（体内停止经其 `stop` 字段携带）。
    MarkerNorthArrow(Box<MarkerNorthArrowBody10_1>),
    /// 四个比例尺子类之一（Hollow/SingleDivision/Alternating/
    /// DoubleAlternating；体内停止经 `body.stop` 携带）。
    ScalebarSubclass(ScalebarSubclassElement10_1),
    /// ScaleLine 或 SteppedScaleLine。
    ScaleLine(Box<ScaleLineBody10_1>),
}

/// 比例尺子类条目的体与类身份（四类共享同一读取器，按流内类标识区分）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScalebarSubclassElement10_1 {
    /// 子类种类（按对象头类标识映射）。
    pub kind: ScalebarSubclassKind10_1,
    /// 已读取体。
    pub body: ScalebarSubclassBody10_1,
}

/// 书签循环条目中已整体消费的元素体（两类书签体字段序完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkElement10_1 {
    /// AOIBookmark 体（位置包络 + 名称）。
    AoiBookmark(Box<AoiBookmark10_1>),
    /// FeatureBookmark 体（int32 + IName 条目 + 名称）。
    FeatureBookmark(Box<FeatureBookmark10_1>),
}

/// Map 对象体后缀序列（图层列表起，书签循环止）的解析结果。
///
/// 各阶段字段仅在读取到达时为 `Some`/非空；中途停止时其后阶段全部保持
/// `None`，停止点之后的字节保留在 [`MapSuffix10_1::remaining_bytes`]。
#[derive(Debug, Clone, PartialEq)]
pub struct MapSuffix10_1 {
    /// Map v21–v27 的旧遮罩附加值。
    pub legacy_masking_short: Option<u16>,
    /// 图层列表（含条目、活动索引与列表自身的停止信息）。
    pub layer_list: LayerList10_1,
    /// 基本图形层对象槽位（到达时）。
    pub base_graphics_layer: Option<ObjectSlot10_1>,
    /// 基本图形层为 CompositeGraphicsLayer 新对象时的已确认体前缀（读取器
    /// 在首个不可闭合字段处停止，体剩余保留在未知尾部中；解析失败时为
    /// `None`，停止原因仍为
    /// [`MapSuffixStop::BaseGraphicsLayerBody`]）。
    pub base_graphics_layer_parsed: Option<CompositeGraphicsLayerPrefix10_1>,
    /// MapSurround 计数原始值（到达时；负数在读取时已拒绝）。
    pub map_surround_count: Option<i32>,
    /// 已消费的 MapSurround 条目（按流内顺序）。
    pub map_surrounds: Vec<ObjectSlot10_1>,
    /// MapSurround 条目中已整体消费的元素体（与 [`Self::map_surrounds`]
    /// 下标对齐；未知类或解析失败为 `None`，后缀在该条目停止）。
    pub map_surround_parsed: Vec<Option<MapSurroundElement10_1>>,
    /// WKSEnvelope 显示范围（到达时）。
    pub extent: Option<WksEnvelope10_1>,
    /// WKSEnvelope 起点偏移（诊断用）。
    pub extent_offset: Option<usize>,
    /// WKSEnvelope 之后的两个 int32 原始位模式（用途未定性，原样保留）。
    pub raw_int32_pair: Option<(i32, i32)>,
    /// IEnvelope 槽位对象（到达时）。
    pub envelope_object: Option<ObjectSlot10_1>,
    /// IEnvelope 槽位为 Envelope 新对象时的完整对象体（Envelope 读取器覆盖
    /// 全部版本分支，读取成功即整体消费，后缀可继续；解析失败时为 `None`，
    /// 停止原因仍为 [`MapSuffixStop::EnvelopeObjectBody`]）。
    pub envelope_object_parsed: Option<Envelope10_1>,
    /// 书签计数原始值（到达时；负数在读取时已拒绝）。
    pub bookmark_count: Option<i32>,
    /// 已消费的书签条目（按流内顺序）。
    pub bookmarks: Vec<ObjectSlot10_1>,
    /// 书签条目中已整体消费的元素体（与 [`Self::bookmarks`] 下标对齐；
    /// 未知类或解析失败为 `None`，后缀在该条目停止）。
    pub bookmark_parsed: Vec<Option<BookmarkElement10_1>>,
    /// 书签后与 10.1 普通写侧对位的局部属性。没有匹配完整前导时为 None。
    pub post_bookmark: Option<PostBookmarkPrefix10_1>,
    /// 已读独立表槽；最后一个体可能携带内部停止原因。
    pub standalone_tables: Vec<(ObjectSlot10_1, Option<StandaloneTablePrefix10_1>)>,
    /// v0–v10 的旧表来源名称。
    pub legacy_table_names: Vec<(usize, DatasetNameMoreSlot10_1)>,
    /// 独立表计数为零时到达的照明属性槽。
    pub illumination: Option<IlluminationSlot10_1>,
    /// v0–v3 内联照明，依次保存三维点和附加 double 的位模式。
    pub legacy_illumination: Option<(usize, [u64; 4])>,
    /// v0–v4 共用的照明后字段，原生版本门跳过的字段为空。
    pub legacy_illumination_fields: Option<LegacyPostIlluminationFields10_1>,
    /// 照明后定长字段，保留流内原始值。
    pub post_illumination: Option<PostIlluminationFields10_1>,
    /// 照明后的 IGeometry 槽；新对象携带解析体。
    pub post_illumination_geometry: Option<(ObjectSlot10_1, Option<MapGeometryBody10_1>)>,
    /// 仅保留已确认的 IGeometry 空引用；未到达、截断或非空时为 None。
    pub post_illumination_null_geometry: Option<ObjectSlot10_1>,
    /// 几何空槽之后的 IBorder 空引用；非空结果见 `post_illumination_border`。
    pub post_illumination_null_border: Option<ObjectSlot10_1>,
    /// 已完整读取的边框槽（含空槽）。
    pub post_illumination_border: Option<(usize, DecorSlot10_1)>,
    /// IBorder 后版本块，按原生顺序保留可达字段。
    pub post_border_tail: PostBorderTail10_1,
    /// 停止原因（恒有：书签循环之后也属停止点）。
    pub stop: MapSuffixStop,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapGeometryBody10_1 {
    Envelope(Envelope10_1),
    Geometry(GeometryBody10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LegacyAnnotateMapBody10_1 {
    AnnotateMap { version: i16 },
    Maplex(Box<MaplexAnnotateMap10_1>),
}

/// Map 边框后字段；未到达或截断的字段为 None。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PostBorderTail10_1 {
    pub short_308: Option<(usize, u16)>,
    pub layer_30c: Option<ObjectSlot10_1>,
    pub short_310: Option<(usize, u16)>,
    pub envelope_314: Option<ObjectSlot10_1>,
    pub envelope_314_parsed: Option<Envelope10_1>,
    pub short_318: Option<(usize, u16)>,
    pub short_31a: Option<(usize, u16)>,
    /// 保留流中 double 的原始位模式，未模拟原生值修正。
    pub double_bits: Option<(usize, u64)>,
    /// 以下成员及临时值的业务语义尚未确定，保留原始位。
    pub int_1a4: Option<(usize, u32)>,
    pub int_1a8: Option<(usize, u32)>,
    pub temporary_short: Option<(usize, u16)>,
    pub temporary_double_bits: Option<(usize, u64)>,
    /// 后续 IGeoTransformation 集合计数。
    pub transformation_count: Option<(usize, i32)>,
    /// 首条方向字段的原值及偏移，完整条目见 `transformations`。
    pub first_transformation_prefix: Option<(usize, u32)>,
    /// 已读取的条目；末项对象为 None 时停在该对象头前。
    pub transformations: Vec<MapTransformation10_1>,
    /// 变换循环后的 double、short 原始位，业务语义尚未确定。
    pub post_transform_double_bits: Option<(usize, u64)>,
    pub post_transform_short: Option<(usize, u16)>,
    /// IBackground 空引用；非空结果见 `post_transform_background`。
    pub post_transform_null_background: Option<ObjectSlot10_1>,
    /// 已完整读取的背景槽（含空槽）。
    pub post_transform_background: Option<(usize, DecorSlot10_1)>,
    /// 版本 19–22 的 IAnnotateMap 槽及新对象体。
    pub legacy_annotate_map: Option<(ObjectSlot10_1, Option<LegacyAnnotateMapBody10_1>)>,
    /// 版本大于 22 的对象槽仅在空引用时消费。
    pub post_transform_null_object: Option<ObjectSlot10_1>,
    pub post_transform_overposter_reference: Option<ObjectSlot10_1>,
    /// 已完整消费的非空 BasicOverposterProperties 对象。
    pub post_transform_overposter: Option<(ObjectSlot10_1, BasicOverposterProperties10_1)>,
    pub post_transform_maplex: Option<(ObjectSlot10_1, Box<MaplexOverposterProperties10_1>)>,
    /// 版本 24 起的三个原始字段（short、double、short）。
    pub short_32a: Option<(usize, u16)>,
    pub double_320_bits: Option<(usize, u64)>,
    pub short_328: Option<(usize, u16)>,
    /// 版本 26 起的四个 IEnvelope 槽和已消费的新对象体。
    pub scale_envelopes: [Option<ObjectSlot10_1>; 4],
    pub scale_envelope_bodies: [Option<Envelope10_1>; 4],
    pub scale_double_2b8_bits: Option<(usize, u64)>,
    pub scale_double_2d8_bits: Option<(usize, u64)>,
    /// 版本 27 起的八个 4 字节原始成员。
    pub scale_ints: [Option<(usize, u32)>; 8],
    /// Map 自身 LayerMasking 的版本和外层计数。
    pub masking_version: Option<(usize, u16)>,
    pub masking_count: Option<(usize, i32)>,
    /// 已完整消费的集合条目；未知新对象使条目整体回退。
    pub masking_entries: Option<Vec<MaskEntry10_1>>,
    /// Map 自身 ILayerMasking::UseMasking 的流内偏移及原始位模式。
    pub masking_empty_short: Option<(usize, u16)>,
    /// 逐图层遮罩记录，顺序与原生图层遍历一致；无接口的条目不产生记录。
    pub layer_masking: Vec<(u32, LayerMasking10_1)>,
    /// 逐图层循环回退前遇到的首个障碍。
    pub layer_masking_stop: Option<String>,
    /// 逐图层遮罩循环完整消费后读取的 Map 字段。
    pub post_masking: Option<PostMaskingMapFields10_1>,
    /// Map 版本 33 起的 IUnknown 集合、LONG 和对象槽。
    pub persist_collections: Option<MapPersistCollections10_1>,
    /// 版本 34 起的已确认字段及未读原因。
    pub late_tail: Option<LateMapTail10_1>,
}

// 只在已确认主接口及其子图层遮罩行为时消费记录。
fn read_map_layer_masking_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    layer_list: &LayerList10_1,
    map_version: u16,
) -> Result<Vec<(u32, LayerMasking10_1)>, String> {
    use crate::maps::layer_masking::{
        LayerMaskingProbe10_1, is_unparsed_layer_without_masking_10_1, layer_masking_probe_10_1,
    };
    if layer_list.stop.is_some()
        || layer_list.active_index.is_none()
        || usize::try_from(layer_list.count) != Ok(layer_list.entries.len())
    {
        return Err("图层列表未完整读取".into());
    }
    let mut traversal = LayerMaskingTraversal {
        reader,
        refs,
        max_bytes,
        skip_top_level_basemap: layer_masking_probe_10_1(map_version as i16, true)
            == LayerMaskingProbe10_1::SkipLegacyBasemapLayer,
        known: Default::default(),
        records: Vec::new(),
    };
    for entry in &layer_list.entries {
        if let LayerEntryObject::Entry(object) = entry.object {
            // 只有已确认 QI 结果且图层列表已定位到有界体末尾，才可登记该层。
            if entry.parsed.is_none()
                && entry.body.is_some()
                && let ObjectEntry10_1::NewObject { class_id, key } = object
                && is_unparsed_layer_without_masking_10_1(class_id)
            {
                traversal.known.insert(key, (class_id, None));
                continue;
            }
            traversal.visit(object, entry.parsed.as_ref(), 0)?;
        }
    }
    Ok(traversal.records)
}

struct LayerMaskingTraversal<'a, 'data, 'body> {
    reader: &'a mut Reader<'data>,
    refs: &'a mut RefTable10_1,
    max_bytes: usize,
    skip_top_level_basemap: bool,
    // FDO、WCS 等专门读取器已经确认身份的嵌入图层可以只登记类。
    known: std::collections::HashMap<u32, ([u8; 16], Option<&'body LayerBodyPrefix10_1>)>,
    records: Vec<(u32, LayerMasking10_1)>,
}

impl<'body> LayerMaskingTraversal<'_, '_, 'body> {
    fn visit(
        &mut self,
        entry: ObjectEntry10_1,
        parsed: Option<&'body LayerBodyPrefix10_1>,
        depth: u32,
    ) -> Result<(), String> {
        use crate::layers::basemap_layer::BasemapStop10_1;
        use crate::layers::composite_graphics::CompositeGraphicsLayerStop10_1;
        use crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
        use crate::layers::group_layer::{GROUP_LAYER_CLASS_ID, GroupLayerStop10_1};

        // 防止异常回引造成无限递归，复用 GroupLayer 读取器的深度限制。
        if depth > crate::layers::group_layer::MAX_RECURSION_DEPTH {
            return Err(format!("遮罩遍历超过深度上限：{entry:?}"));
        }
        let (class_id, key, body) = match entry {
            ObjectEntry10_1::Null => return Ok(()),
            ObjectEntry10_1::NewObject { class_id, key } => {
                let body = parsed.ok_or_else(|| {
                    format!(
                        "图层键 {key} 的类 {} 缺少可用对象体",
                        class_id_hex(&class_id)
                    )
                })?;
                self.known.insert(key, (class_id, Some(body)));
                (class_id, key, Some(body))
            }
            ObjectEntry10_1::BackReference { class_id, key } => {
                let (known_class, body) = self.known.get(&key).copied().ok_or_else(|| {
                    format!(
                        "遮罩图层回引键 {key} 未登记，类 {}",
                        class_id_hex(&class_id)
                    )
                })?;
                if class_id != known_class {
                    return Err(format!(
                        "图层回引键 {key} 与登记类不匹配，请求类 {}",
                        class_id_hex(&class_id)
                    ));
                }
                (class_id, key, body)
            }
        };
        match body {
            Some(LayerBodyPrefix10_1::GroupLayer(group)) => {
                if group.stop != GroupLayerStop10_1::Complete
                    || usize::try_from(group.child_count) != Ok(group.children.len())
                {
                    return Err(format!(
                        "组图层键 {key} 的子层未完整读取，类 {}",
                        class_id_hex(&class_id)
                    ));
                }
                self.read_record(key)?;
                for child in &group.children {
                    self.visit_group_child(child, depth + 1)?;
                }
            }
            Some(LayerBodyPrefix10_1::ProcessLayer(process)) => {
                if usize::try_from(process.group.child_count) != Ok(process.group.children.len()) {
                    return Err(format!(
                        "流程图层键 {key} 的子层未完整读取，类 {}",
                        class_id_hex(&class_id)
                    ));
                }
                self.read_record(key)?;
                // ICompositeLayer 枚举委派组的子层，组本身没有独立遮罩记录。
                for child in &process.group.children {
                    self.visit_group_child(child, depth + 1)?;
                }
            }
            Some(LayerBodyPrefix10_1::BasemapLayer(basemap))
                if basemap.stop == BasemapStop10_1::Complete =>
            {
                for &key in &basemap.feature_keys {
                    self.known
                        .entry(key)
                        .or_insert((FEATURE_LAYER_CLASS_ID, None));
                }
                if depth != 0 || !self.skip_top_level_basemap {
                    // 底图枚举的是集合中的包装层；其 QI 不转发 ILayerMasking。
                    let confirmed = basemap.sublayers.iter().all(|slot| match slot {
                        ObjectEntry10_1::NewObject { class_id, key } => {
                            matches!(*class_id, GROUP_LAYER_CLASS_ID | FEATURE_LAYER_CLASS_ID)
                                && basemap
                                    .sublayer_bodies
                                    .iter()
                                    .any(|(body_key, _)| body_key == key)
                        }
                        ObjectEntry10_1::BackReference {
                            class_id: FEATURE_LAYER_CLASS_ID,
                            key,
                        } => {
                            basemap.feature_keys.contains(key)
                                || basemap.sublayer_bodies.iter().any(|(body_key, body)| {
                                    body_key == key
                                        && matches!(
                                            body.as_ref(),
                                            LayerBodyPrefix10_1::FeatureLayer(_)
                                        )
                                })
                        }
                        ObjectEntry10_1::BackReference {
                            class_id: GROUP_LAYER_CLASS_ID,
                            key,
                        } => basemap.sublayer_bodies.iter().any(|(body_key, body)| {
                            body_key == key
                                && matches!(body.as_ref(), LayerBodyPrefix10_1::GroupLayer(_))
                        }),
                        ObjectEntry10_1::Null => true,
                        _ => false,
                    });
                    if !confirmed {
                        return Err(format!("底图图层键 {key} 的正式子层未全部确认"));
                    }
                    self.read_record(key)?;
                }
            }
            _ if known_layer_masking_support_10_1(&class_id)
                == LayerMaskingSupport10_1::Unsupported =>
            {
                match body {
                    None
                    | Some(
                        LayerBodyPrefix10_1::FeatureLayer(_)
                        | LayerBodyPrefix10_1::CadLayer(_)
                        | LayerBodyPrefix10_1::CadFeatureLayer(_)
                        | LayerBodyPrefix10_1::CadAnnotationLayer(_)
                        | LayerBodyPrefix10_1::CoverageAnnotationLayer(_)
                        | LayerBodyPrefix10_1::RasterLayer(_)
                        | LayerBodyPrefix10_1::TiledLayer(_)
                        | LayerBodyPrefix10_1::Graphics3d(_)
                        | LayerBodyPrefix10_1::NaLayer(_)
                        | LayerBodyPrefix10_1::TerrainLayer(_)
                        | LayerBodyPrefix10_1::TinLayer(_)
                        | LayerBodyPrefix10_1::NetworkLayer(_),
                    ) => {}
                    Some(LayerBodyPrefix10_1::CompositeGraphicsLayer(layer))
                        if layer.stop == CompositeGraphicsLayerStop10_1::Completed => {}
                    Some(LayerBodyPrefix10_1::ImageServerLayer(layer)) if layer.stop.is_none() => {}
                    Some(LayerBodyPrefix10_1::WcsLayer(wcs)) => {
                        if let ObjectEntry10_1::NewObject { key, class_id } = wcs.raster.entry {
                            self.known.entry(key).or_insert((class_id, None));
                        }
                    }
                    Some(LayerBodyPrefix10_1::FdoGraphicsLayer(fdo))
                        if fdo.completion().is_some() =>
                    {
                        if let Some(key) = fdo.associated_feature_layer_key {
                            self.known
                                .entry(key)
                                .or_insert((FEATURE_LAYER_CLASS_ID, None));
                        }
                    }
                    _ => {
                        return Err(format!(
                            "图层键 {key} 的对象体未完整读取，类 {}",
                            class_id_hex(&class_id)
                        ));
                    }
                }
            }
            _ => {
                return Err(format!(
                    "图层键 {key} 的遮罩分派未确认，类 {}",
                    class_id_hex(&class_id)
                ));
            }
        }
        Ok(())
    }

    fn visit_group_child(
        &mut self,
        child: &'body crate::layers::group_layer::GroupChildEntry10_1,
        depth: u32,
    ) -> Result<(), String> {
        use crate::layers::group_layer::GroupChildEntry10_1;

        match child {
            GroupChildEntry10_1::Empty | GroupChildEntry10_1::Null => Ok(()),
            GroupChildEntry10_1::BackReference { class_id, key } => self.visit(
                ObjectEntry10_1::BackReference {
                    class_id: *class_id,
                    key: *key,
                },
                None,
                depth,
            ),
            GroupChildEntry10_1::ParsedNewObject {
                class_id,
                key,
                parsed,
            }
            | GroupChildEntry10_1::SkippedNewObject {
                class_id,
                key,
                body_length: 0,
                parsed: Some(parsed),
                ..
            } => self.visit(
                ObjectEntry10_1::NewObject {
                    class_id: *class_id,
                    key: *key,
                },
                Some(parsed),
                depth,
            ),
            GroupChildEntry10_1::SkippedNewObject {
                class_id,
                key,
                parsed: None,
                ..
            } if crate::maps::layer_masking::is_unparsed_layer_without_masking_10_1(*class_id) => {
                // ExtraSafe 已同步到下一条目，此类的子层查询不消费遮罩。
                self.known.insert(*key, (*class_id, None));
                Ok(())
            }
            GroupChildEntry10_1::SkippedNewObject { class_id, key, .. } => Err(format!(
                "组图层子层键 {key} 的对象体未完整读取，类 {}",
                class_id_hex(class_id)
            )),
        }
    }

    fn read_record(&mut self, key: u32) -> Result<(), String> {
        let record = read_layer_masking_10_1(self.reader, self.max_bytes, self.refs)
            .map_err(|error| format!("图层键 {key} 的遮罩记录读取失败：{error}"))?;
        self.records.push((key, record));
        Ok(())
    }
}

/// Map 版本 29..=51 中 LayerMasking 之后的已确认字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostMaskingMapFields10_1 {
    /// IScreenDisplay.IsFramed 的 16 位 VARIANT_BOOL 原始位。
    pub is_framed_bits: (usize, u16),
    /// Map 成员 +0x398 的字符串；仅版本大于 29 时存在。
    pub string_398: Option<(usize, String)>,
    /// Map 成员 +0x3a0 的 short；仅版本至少为 31 时存在。
    pub short_3a0: Option<(usize, u16)>,
    /// Map 成员 +0x15d 的单字节原值；仅版本至少为 32 时存在。
    pub byte_15d: Option<(usize, u8)>,
}

/// 读取 LayerMasking 后、下一对象集合之前的字段；失败不推进游标。
///
/// 调用方须先确认 Map 版本和本段起点。10.1 Load 在版本 > 0x1c 时读取
/// 通过 Map 子接口虚表 +0x60 设置 IsFramed，在版本 > 0x1d 时读取 Load 视图 +0x398 的
/// String，再按版本读 +0x3a0 short（0x79400908..14）与 +0x15d 字节
/// （0x79400939..45）。未知业务语义保留原始位模式。
pub fn read_post_masking_map_fields_10_1(
    reader: &mut Reader<'_>,
    map_version: u16,
    max_bytes: usize,
) -> io::Result<PostMaskingMapFields10_1> {
    if !(29..=51).contains(&map_version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Map 版本未进入遮罩后 short 分支（需要 29..=51）",
        ));
    }
    let mut trial = reader.clone();
    let short = (trial.position(), trial.read_u16_le()?);
    let string_398 = if map_version > 29 {
        let offset = trial.position();
        Some((offset, read_string_10_1(&mut trial, max_bytes)?))
    } else {
        None
    };
    let short_3a0 = if map_version >= 31 {
        Some((trial.position(), trial.read_u16_le()?))
    } else {
        None
    };
    let byte_15d = if map_version >= 32 {
        Some((trial.position(), trial.read_bytes(1)?[0]))
    } else {
        None
    };
    *reader = trial;
    Ok(PostMaskingMapFields10_1 {
        is_framed_bits: short,
        string_398,
        short_3a0,
        byte_15d,
    })
}

/// LabelPlacement.dll 的 BasicOverposterProperties 类标识。
pub const BASIC_OVERPOSTER_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x56, 0x3d, 0xc7, 0xe0, 0x88, 0x6c, 0x8c, 0x49, 0xb7, 0x7c, 0x78, 0x60, 0x6e, 0x68, 0x8c, 0x97,
];

/// 原始字段按流顺序保存；版本 0 和 1 无后续字段。
#[derive(Debug, Clone, PartialEq)]
pub struct BasicOverposterProperties10_1 {
    pub version: u16,
    pub short_before_color: Option<u16>,
    pub color: Option<ColorSlot10_1>,
    pub short_after_color: Option<u16>,
    pub double_bits: Option<u64>,
}

/// 读取已确认的对象体；短读、未知颜色或版本超界不推进或登记键。
pub fn read_basic_overposter_properties_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<BasicOverposterProperties10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = trial.read_u16_le()?;
    if version > 2 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BasicOverposterProperties 版本超界",
        ));
    }
    let mut body = BasicOverposterProperties10_1 {
        version,
        short_before_color: None,
        color: None,
        short_after_color: None,
        double_bits: None,
    };
    if version == 2 {
        body.short_before_color = Some(trial.read_u16_le()?);
        let color = read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?;
        if matches!(color, ColorSlot10_1::Unsupported { .. }) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "不支持的 BasicOverposterProperties 颜色类",
            ));
        }
        body.color = Some(color);
        body.short_after_color = Some(trial.read_u16_le()?);
        body.double_bits = Some(u64::from_le_bytes(
            trial.read_bytes(8)?.try_into().expect("固定长度"),
        ));
    }
    *reader = trial;
    *table = staged;
    Ok(body)
}

/// 坐标变换循环的一项，v12/v13 直接从对象头开始。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTransformation10_1 {
    pub offset: usize,
    /// v14 起保存的 esriTransformDirection 原始位模式。
    pub prefix: Option<u32>,
    pub object: Option<ObjectSlot10_1>,
    pub body: Option<GeoTransformationBody10_1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapTransformationDirection10_1 {
    Forward,
    Reverse,
    Unknown(u32),
}

impl MapTransformation10_1 {
    /// 返回存盘方向。v12/v13 无方向字段，原生装载时传正向给操作集。
    pub fn direction_on_disk(&self) -> Option<MapTransformationDirection10_1> {
        self.prefix.map(|value| match value {
            0 => MapTransformationDirection10_1::Forward,
            1 => MapTransformationDirection10_1::Reverse,
            _ => MapTransformationDirection10_1::Unknown(value),
        })
    }
}

/// 原专用读取入口保留，底层与 GeoTransformation 共用实现。
pub use crate::objects::geo_transformation::{
    GeoTransformationBody10_1 as NadconTransformation10_1, NADCON_TRANSFORMATION_CLASS_ID,
};
use crate::objects::geo_transformation::{
    GeoTransformationBody10_1, read_geo_transformation_body_10_1,
    supports_geo_transformation_body_10_1,
};

pub fn read_nadcon_transformation_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<NadconTransformation10_1> {
    read_geo_transformation_body_10_1(reader, NADCON_TRANSFORMATION_CLASS_ID, max_bytes)
}

/// 普通保存路径中的书签后前导（`Map::Save 0x79401850`）。成员偏移沿用
/// 持久化子接口指针，字段保留原始存盘值。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostBookmarkPrefix10_1 {
    pub offset: usize,
    /// Load 读入临时位置的首个 short。10.1 普通 Save 写 0xffff。
    pub leading_short: u16,
    /// IViewManager.VerboseEvents 的 short 原值。
    pub short_19c: u16,
    /// IMap.ReferenceScale。
    pub double_200: f64,
    /// IMap.UseSymbolLevels 的 short 原值。
    pub short_264: u16,
    /// IStandaloneTable 条目计数。
    pub standalone_table_count: i32,
}

/// Map 版本 5..=51 照明槽后的定长成员（Load 0x793ff4d0）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostIlluminationFields10_1 {
    pub offset: usize,
    /// IViewManager.ConserveMemory 的 short 原值，原生 Load 在版本小于 35 时改为 0xffff。
    pub short_1a0: u16,
    /// Map::Save 写零，Load 只读到临时变量。
    pub discarded_short: u16,
    /// IViewManager.OutputBandSize 的 LONG 原始位模式。
    pub int_1b0: u32,
    /// IViewManager.DelayBackgroundDraw 的 short 原值。
    pub short_1ac: u16,
    /// IMap.Expanded 的 short 原值。
    pub short_268: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyPostIlluminationFields10_1 {
    pub offset: usize,
    pub short_1a0: Option<u16>,
    pub discarded_short: Option<u16>,
    pub int_1b0: Option<u32>,
    pub short_1ac: Option<u16>,
}

/// DisplayCore.dll IlluminationProps 的磁盘类标识（由 GetClassID 确认）。
pub const ILLUMINATION_PROPS_CLASS_ID: [u8; 16] = [
    0x40, 0x2f, 0x35, 0x1c, 0x8e, 0x29, 0xd3, 0x11, 0x9f, 0x4f, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0x19,
];

/// Map 的照明属性槽（新对象体完整消费才登记）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IlluminationSlot10_1 {
    pub object: ObjectSlot10_1,
    pub body: Option<IlluminationProps10_1>,
}

/// 10.1 的 IlluminationProps 体（DisplayCore.dll Load 0x6ebab040）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IlluminationProps10_1 {
    pub version: u16,
    /// 原生 AfStreamRead(_WKSPointZ *) 读入的三个 double。
    pub point: [f64; 3],
    /// 尾随 double，保留原始数值。
    pub value: f64,
}

/// Map 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`。三重印证：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 Map（模块 Map.dll）；
/// 该 CLSID 的字节宿主在 Map.dll `0x79482adc`（GetClassID 引用）；Map.dll
/// RTTI 存在 `.?AVMap@@`。仓库参考样本 `/Maps:0x04..0x13` 首对象头的类
/// 标识与本常量逐字节相同。
pub const MAP_CLASS_ID: [u8; 16] = [
    0x76, 0xaa, 0xbd, 0xe6, 0x35, 0x4d, 0xd0, 0x11, 0x98, 0xbe, 0x00, 0x80, 0x5f, 0x7c, 0xed, 0x21,
];

/// Map 前缀第 4 步嵌套空间参考的三分派结果（F 轮接线；类身份由流内对象
/// 头决定，证据 docs/arcmap-10.1-pcs-geometry-slots.md「接线建议」）。
#[derive(Debug, Clone, PartialEq)]
pub enum MapSpatialReference10_1 {
    /// ProjectedCoordinateSystem（既有读取器）。
    Projected(ProjectedCoordinateSystem10_1),
    /// GeographicCoordinateSystem（F 轮；`spatial_reference_slots` 读取器）。
    Geographic(crate::objects::spatial_reference_slots::GeographicCoordinateSystem10_1),
    /// UnknownCoordinateSystem（F 轮；同上）。
    UnknownCoordinateSystem(crate::objects::spatial_reference_slots::UnknownCoordinateSystem10_1),
}

/// 10.1 Map::Load（`0x793ff4d0`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MapPrefix10_1 {
    /// 2 字节架构版本 short 的无符号位模式，>51 的位模式一律拒绝。
    ///
    /// 证据：Load `0x793ff4d0` 开头 `AfStreamRead(2)` 后对 >0x33（51）直接
    /// 拒绝，样本值 51。反编译比较的符号性未闭合：按无符号解释接受 0..=51；
    /// 若原生为有符号比较，负值位模式（0x8000..）可能通过原生检查，此处
    /// 一并拒绝（较原生更严，与 spatial_reference 的版本处理一致）。
    pub version: u16,
    /// 数据框名称（Load 第 2 步，写入 obj+0x160；样本「新建数据框」）。
    pub name: String,
    /// 第二名称字段（Load 第 3 步，写入 obj+0x164；样本为空串）。
    pub secondary_name: String,
    /// Load 第 4 步的嵌套空间参考对象体。
    ///
    /// 16 零字节空头 → `None`（沿用对象头的 S_FALSE 约定，仅消耗 16 字节、
    /// 不读键）；类标识命中 F 轮三分派（投影/地理/未知坐标系）→ 已解析
    /// 对象体；其他类标识 → [`read_map_prefix_10_1`] 直接报错。对象头键值
    /// 原样丢弃，未保留。
    pub spatial_reference: Option<MapSpatialReference10_1>,
    /// 第 4 步空间参考对象键；供后续严格解析对该对象的回引。
    /// 空标记时为 `None`。
    pub spatial_reference_key: Option<u32>,
    /// 同一 ObjectStream 已登记键的空间参考回引；不在此处重复读对象体。
    pub spatial_reference_back_reference: Option<[u8; 16]>,
    /// 已确认前缀之后剩余的字节数。
    ///
    /// 对应 Load 第 5 步起的字段：其中「图层列表 → … → 书签循环」一段可由
    /// [`read_map_suffix_10_1`] 继续读取，其余（书签循环之后）仍为未闭合字段。
    /// 本前缀读取器不读取、不解释。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 Map 对象体的已确认前缀，失败时不推进游标。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取
/// Map 对象头、比对 [`MAP_CLASS_ID`] 确认类身份，并从对象头之后的位置传
/// 入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：应用于两个名称字符串的声明长度（含终止零、
/// 不含 4 字节前缀），并原样转发给嵌套投影坐标系对象体（约束 WKT 与域字
/// 符串长度）；限额在任何读取与分配之前检查，`usize::MAX` 表示不额外设
/// 限，此时超长流按缺短读失败。
///
/// 嵌套空间参考对象的类身份由流内对象头决定：全零 16 字节为空头
/// （`spatial_reference = None`），类标识命中 F 轮三分派（投影/地理/未知
/// 坐标系）时用对应读取器读取对象体；其他类标识返回
/// [`ErrorKind::Unsupported`]——嵌套对象体没有外部长度字段，本读取器无法
/// 跳过未知类继续解析，调用者不能把该流继续当作 Map 前缀消费。
///
/// 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进调用
/// 者游标。错误位置相对于传入 Reader，调用方需另外记录流和对象身份。这是
/// 严格安全子集，不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_map_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<MapPrefix10_1> {
    // 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进原游标。
    let mut trial = reader.clone();
    let (result, _spatial_reference_key) = read_map_prefix_inner(&mut trial, max_bytes, None)?;
    *reader = trial;
    Ok(result)
}

/// 在已有 ObjectStream 会话中读取 Map 前缀。已登记的空间参考对象仅消费
/// 对象头；本入口不登记新键，完整后缀读取器负责登记并提交。
pub fn read_map_prefix_object_stream_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &RefTable10_1,
) -> io::Result<MapPrefix10_1> {
    let mut trial = reader.clone();
    let (result, _) = read_map_prefix_inner(&mut trial, max_bytes, Some(table))?;
    *reader = trial;
    Ok(result)
}

/// 前缀读取的内部实现：额外返回新空间参考对象的键。
///
/// 键不参与前缀的公开行为（保持 [`read_map_prefix_10_1`] 的签名与语义不变），
/// 仅供 [`read_map_suffix_10_1`] 登记进共享键表——流内后续对象对 Map 空间
/// 参考的引用（如 Envelope 内的回引）按键命中，样本中 Map 的 PCS 键 30 被
/// 多个 Envelope 回引（证据文档「Envelope 对象」节）。
fn read_map_prefix_inner(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    table: Option<&RefTable10_1>,
) -> io::Result<(MapPrefix10_1, Option<u32>)> {
    let entry_offset = trial.position();

    // 1. 2 字节架构版本：无符号解释，>51 直接拒绝（原生 >0x33 拒绝）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 对象体 offset 0x{entry_offset:x}: 架构版本 {version} 超出 \
                 Map::Load 接受范围（>0x33/51 直接拒绝）"
            ),
        ));
    }

    // 2/3. 两个名称字符串：AfCore String 布局（样本字节闭合，见模块文档），
    // 各自受 max_bytes 限额约束。
    let name = read_string_10_1(trial, max_bytes)?;
    let secondary_name = read_string_10_1(trial, max_bytes)?;

    // 4. 嵌套空间参考对象：先读对象头（类身份完全由流内对象头决定）。
    let nested_offset = trial.position();
    let mut spatial_reference_key = None;
    let mut spatial_reference_back_reference = None;
    let spatial_reference = match read_header_10_1(trial)? {
        // 全零 16 字节：沿用对象头读取器的 S_FALSE 约定，仅消耗对象头本身、
        // 不读键；语义上表示该 Map 没有空间参考对象。
        ObjectHeader::Null => None,
        ObjectHeader::Object { class_id, key } if table.is_some_and(|table| table.contains(key)) => {
            spatial_reference_key = Some(key);
            spatial_reference_back_reference = Some(class_id);
            None
        }
        ObjectHeader::Object { class_id, key }
            if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // PCS 对象体由专门读取器解析，max_bytes 原样转发；键原样返回
            // 供调用者登记（本函数不修改登记表）。
            spatial_reference_key = Some(key);
            Some(MapSpatialReference10_1::Projected(
                read_projected_coordinate_system_10_1(trial, max_bytes)?,
            ))
        }
        ObjectHeader::Object { class_id, key }
            if class_id
                == crate::objects::spatial_reference_slots::GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // 地理坐标系（F 轮接线三分派）。
            spatial_reference_key = Some(key);
            Some(MapSpatialReference10_1::Geographic(
                crate::objects::spatial_reference_slots::read_geographic_coordinate_system_10_1(
                    trial,
                    max_bytes,
                )?,
            ))
        }
        ObjectHeader::Object { class_id, key }
            if class_id
                == crate::objects::spatial_reference_slots::UNKNOWN_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // 未知坐标系（F 轮接线三分派）。
            spatial_reference_key = Some(key);
            Some(MapSpatialReference10_1::UnknownCoordinateSystem(
                crate::objects::spatial_reference_slots::read_unknown_coordinate_system_10_1(
                    trial,
                    max_bytes,
                )?,
            ))
        }
        ObjectHeader::Object { class_id, key } => {
            // 严格子集只认 F 轮三分派的三个坐标系类。嵌套对象体无外部长度
            // 字段，无法跳过未知类继续读取；显式报错，原游标不推进。
            let id = class_id_hex(&class_id);
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Map 对象体嵌套对象 offset 0x{nested_offset:x}: 类标识 {id}（键 \
                     {key}）不是已闭合的空间参考类；本读取器仅支持投影/地理/未知坐标系 \
                     嵌套（F 轮三分派），且对象体无外部长度字段，无法跳过未知类继续读取"
                ),
            ));
        }
    };

    Ok((
        MapPrefix10_1 {
            version,
            name,
            secondary_name,
            spatial_reference,
            spatial_reference_key,
            spatial_reference_back_reference,
            remaining_bytes: trial.remaining(),
        },
        spatial_reference_key.filter(|_| spatial_reference_back_reference.is_none()),
    ))
}

/// 从 Map 对象体起点读取图层列表起、书签循环止的后缀序列，失败时不推进
/// 游标、不修改共享登记表。
///
/// 起点约定：调用者已读取 Map 对象头并把 Reader 定位在对象体起点（2 字节
/// 架构版本处）——与 [`read_map_prefix_10_1`] 的起点相同；本函数先按同一
/// 规则重读已确认前缀（用于把嵌套空间参考对象的键登记进共享表并定位图层
/// 列表起点，前缀字段不再重复报告），再继续读取后缀序列。`table` 是跨流
/// 共享的会话键登记表：本函数在 trial 副本上登记（含前缀嵌套 PCS 键与后缀
/// 全部新对象键），仅在成功返回（含停止点）时提交；任何错误路径不推进游标、
/// 不改动调用者的登记表。Map 对象自身的键由调用者在读取 Map 对象头时登记，
/// 本函数从对象体起点开始、看不到对象头。
///
/// 各阶段停止规则（均为成功返回 + [`MapSuffixStop`]，剩余字节保留）：
///
/// - 图层列表标志 1 新对象体（无外部长度；集成点）；
/// - 基本图形层、MapSurround 条目、IEnvelope 槽位、书签条目的**新对象**体
///   （普通 `LoadObject` 槽位无 ExtraSafe 前缀，体无外部长度；基本图形层与
///   图层体同样预留集成点）。空标记与回引条目只消耗对象头，继续；
/// - 书签循环后读取定长属性前导；无独立表时读取已确认的
///   照明属性槽。无法确认前导、表条目或照明槽时从相应边界保留。
///
/// IEnvelope 槽位的空标记消费 **16 字节**（沿用 [`crate::storage::object_stream`]
/// S_FALSE 约定）。注意与任务摘要的 20 字节解读不同：样本 `/Maps:0x24f8c` 起
/// 有 20 个零字节，其后 `0x24fa0` 为 `ff ff`；若空标记消费 20 字节，书签计数
/// 将位于 0x24fa0 且读出 0xffff（65535 个书签），与样本无书签及证据文档
/// 「`0x24f9c` int32=0（书签计数）」矛盾。故按 16 字节 + 计数位于 0x24f9c
/// 消费；「SaveObject 空标记是否写 4 字节键 0」仍列为开放问题（前缀嵌套
/// 空头的同项开放问题见模块文档）。
///
/// `max_bytes` 语义与前缀相同（名称字符串与嵌套 PCS 体），并预检图层列表、
/// MapSurround、书签三个循环的「声明计数 × 最小条目字节数」。所有偏移相对
/// 传入 Reader 的底层字节。这是严格安全子集，不代表完整原生 Load 行为。
pub fn read_map_suffix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MapSuffix10_1> {
    read_map_with_suffix_10_1(reader, max_bytes, table).map(|(_, suffix)| suffix)
}

/// 读取普通对象槽中的 Map，并保留前缀字段和后缀解析结果。
pub(crate) fn read_map_with_suffix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<(MapPrefix10_1, MapSuffix10_1)> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 0. 重读已确认前缀：登记嵌套 PCS 键（新键，样本键 30 首现于此），
    //    前缀字段本身不重复报告。
    let (map_prefix, spatial_reference_key) =
        read_map_prefix_inner(&mut trial, max_bytes, Some(&trial_table))?;
    if let Some(key) = spatial_reference_key {
        trial_table.register(key);
    }

    let version = map_prefix.version;
    let layer_list = if version < 23 {
        read_legacy_layer_lists_10_1(&mut trial, max_bytes, &mut trial_table, version)?
    } else {
        read_layer_list_10_1(&mut trial, max_bytes, &mut trial_table)?
    };
    let mut stop = layer_list.stop.map(|stop| match stop {
        LayerListStop::Flag1NewObjectBody {
            index,
            offset,
            class_id,
            key,
        } => MapSuffixStop::LayerListFlag1Body {
            index,
            offset,
            class_id,
            key,
        },
        LayerListStop::LegacyObjectBody { index, offset, .. } => {
            MapSuffixStop::LegacyLayerBody { index, offset }
        }
    });
    let mut legacy_masking_short = if stop.is_none() && matches!(version, 20 | 21) {
        read_legacy_masking(&mut trial, version)?
    } else {
        None
    };

    let mut base_graphics_layer = None;
    let mut base_graphics_layer_parsed = None;
    let mut map_surround_count = None;
    let mut map_surrounds = Vec::new();
    let mut map_surround_parsed = Vec::new();
    let mut extent = None;
    let mut extent_offset = None;
    let mut raw_int32_pair = None;
    let mut envelope_object = None;
    let mut envelope_object_parsed = None;
    let mut bookmark_count = None;
    let mut bookmarks = Vec::new();
    let mut bookmark_parsed = Vec::new();

    if stop.is_none() {
        // 2. 基本图形层对象（SaveObject(+0xf8)，请求 IID {34B2EF81-…}）。
        //    CompositeGraphicsLayer 对象体完整消费后才继续。该槽位没有外部
        //    长度；未知类或体内停止时将余下字节保留为未知尾部。
        let offset = trial.position();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        if let ObjectEntry10_1::NewObject { class_id, .. } = entry {
            if class_id == COMPOSITE_GRAPHICS_LAYER_CLASS_ID {
                base_graphics_layer_parsed = read_composite_graphics_layer_prefix_10_1(
                    &mut trial,
                    max_bytes,
                    &mut trial_table,
                )
                .ok();
            }
        }
        base_graphics_layer = Some(ObjectSlot10_1 { offset, entry });
        if let ObjectEntry10_1::NewObject { class_id, key } = entry
            && !base_graphics_layer_parsed
                .as_ref()
                .is_some_and(|body| body.stop == CompositeGraphicsLayerStop10_1::Completed)
        {
            stop = Some(MapSuffixStop::BaseGraphicsLayerBody {
                offset,
                class_id,
                key,
            });
        }
    }

    if stop.is_none() && (22..=27).contains(&version) {
        legacy_masking_short = read_legacy_masking(&mut trial, version)?;
    }

    if stop.is_none() {
        // 3. MapSurround 计数 + 循环（请求 IID {FE3115E9-…}；样本计数 0）。
        //    Legend 新对象经 map_surround 读取器解析其已确认前缀；体完整消费
        //    时循环继续，其余类或体内停止在该条目处停止。
        let count = read_loop_count(&mut trial, "MapSurround", max_bytes)?;
        map_surround_count = Some(count);
        for index in 0..usize::try_from(count).expect("计数已确认为非负") {
            let offset = trial.position();
            let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
            let mut parsed_element = None;
            let mut fully_consumed = false;
            let mut family_stop = None;
            if let ObjectEntry10_1::NewObject { class_id, key } = entry
                && let Ok((body, body_stop)) = read_map_surround_body_10_1(
                    class_id,
                    key,
                    &mut trial,
                    max_bytes,
                    &mut trial_table,
                )
            {
                fully_consumed = body_stop.is_none();
                match body_stop {
                    Some(SurroundBodyStop10_1::Family(reason)) => family_stop = Some(reason),
                    Some(SurroundBodyStop10_1::ScaleLine(reason)) => {
                        stop = Some(MapSuffixStop::ScaleLineBody { index, reason })
                    }
                    _ => {}
                }
                parsed_element = Some(body);
            }
            let new_object = match &entry {
                ObjectEntry10_1::NewObject { class_id, key } => Some((*class_id, *key)),
                _ => None,
            };
            map_surround_parsed.push(parsed_element);
            map_surrounds.push(ObjectSlot10_1 { offset, entry });
            if stop.is_some() {
                break;
            }
            if !fully_consumed {
                if let Some((class_id, key)) = new_object {
                    stop = Some(match family_stop {
                        // 族内读取器已定位的体内停止：透传原因（游标停在
                        // 其停止点）。
                        Some(reason) => MapSuffixStop::MapSurroundFamilyBody {
                            index,
                            offset,
                            class_id,
                            key,
                            reason,
                        },
                        None => MapSuffixStop::MapSurroundBody {
                            index,
                            offset,
                            class_id,
                            key,
                        },
                    });
                    break;
                }
            }
        }
    }

    if stop.is_none() {
        // 4. WKSEnvelope 32 字节（非对象；[XMin,YMin,XMax,YMax]，写方向确定）。
        let offset = trial.position();
        let mut coords = [0_f64; 4];
        for slot in &mut coords {
            *slot = trial.read_f64_le()?;
        }
        extent_offset = Some(offset);
        extent = Some(WksEnvelope10_1 {
            xmin: coords[0],
            ymin: coords[1],
            xmax: coords[2],
            ymax: coords[3],
        });

        // 5. 两个 int32 原样（Load 第 7 步写入子接口 +0x38/+0x40，用途未定性）。
        let first = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let second = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        raw_int32_pair = Some((first, second));
    }

    if stop.is_none() {
        // 6. IEnvelope 槽位对象（请求 IID {1B077914-…}；样本为空标记，消费
        //    长度见函数文档）。类标识为 Envelope 时用其读取器整体消费对象
        //    体（Envelope 全部版本分支已闭合）：读取成功则后缀继续到书签
        //    循环；其余类或解析失败在该槽位停止。
        let offset = trial.position();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        let mut envelope_consumed = false;
        if let ObjectEntry10_1::NewObject { class_id, .. } = entry {
            if class_id == ENVELOPE_CLASS_ID {
                match read_envelope_10_1(&mut trial, max_bytes, &mut trial_table) {
                    Ok(parsed) => {
                        envelope_object_parsed = Some(parsed);
                        envelope_consumed = true;
                    }
                    Err(_) => {
                        // 解析失败（截断、嵌套 SR 非 PCS 等）：在该槽位停止，
                        // 失败细节由调用者用 Envelope 读取器重放诊断。
                    }
                }
            }
        }
        envelope_object = Some(ObjectSlot10_1 { offset, entry });
        if !envelope_consumed {
            if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                stop = Some(MapSuffixStop::EnvelopeObjectBody {
                    offset,
                    class_id,
                    key,
                });
            }
        }
    }

    if stop.is_none() {
        // 7. 书签计数 + 循环（请求 IID {EE53528B-…}；样本计数 0）。AOI 与
        //    Feature 书签的新对象体字段序完全闭合，读取成功即循环继续；其余
        //    类或解析失败在该条目处停止。
        let count = read_loop_count(&mut trial, "书签", max_bytes)?;
        bookmark_count = Some(count);
        for index in 0..usize::try_from(count).expect("计数已确认为非负") {
            let offset = trial.position();
            let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
            let mut parsed_element = None;
            if let ObjectEntry10_1::NewObject { class_id, .. } = &entry {
                if *class_id == AOI_BOOKMARK_CLASS_ID {
                    if let Ok(body) =
                        read_aoi_bookmark_10_1(&mut trial, max_bytes, &mut trial_table)
                    {
                        parsed_element = Some(BookmarkElement10_1::AoiBookmark(Box::new(body)));
                    }
                } else if *class_id == FEATURE_BOOKMARK_CLASS_ID {
                    if let Ok(body) =
                        read_feature_bookmark_10_1(&mut trial, max_bytes, &mut trial_table)
                    {
                        parsed_element = Some(BookmarkElement10_1::FeatureBookmark(Box::new(body)));
                    }
                }
            }
            let new_object = match &entry {
                ObjectEntry10_1::NewObject { class_id, key } => Some((*class_id, *key)),
                _ => None,
            };
            let consumed = parsed_element.is_some();
            bookmark_parsed.push(parsed_element);
            bookmarks.push(ObjectSlot10_1 { offset, entry });
            if !consumed {
                if let Some((class_id, key)) = new_object {
                    stop = Some(MapSuffixStop::BookmarkBody {
                        index,
                        offset,
                        class_id,
                        key,
                    });
                    break;
                }
            }
        }
    }

    // 8. Load 在书签后读取临时 short、三个成员和表计数。普通 Save 的临时值为 0xffff。
    let mut post_bookmark = None;
    if stop.is_none() {
        let mut candidate = trial.clone();
        if let Ok(prefix) = read_post_bookmark_prefix(&mut candidate, max_bytes) {
            trial = candidate;
            stop = Some(MapSuffixStop::AfterPostBookmarkPrefix {
                standalone_table_count: prefix.standalone_table_count,
            });
            post_bookmark = Some(prefix);
        }
    }
    let mut standalone_tables = Vec::new();
    if let Some(prefix) = post_bookmark.filter(|p| p.standalone_table_count > 0)
        && map_prefix.version >= 11
    {
        for _ in 0..prefix.standalone_table_count {
            let mut candidate = trial.clone();
            let mut staged = trial_table.clone();
            let offset = candidate.position();
            stop = Some(MapSuffixStop::StandaloneTableBody { offset });
            let Ok(entry) = read_object_entry_10_1(&mut candidate, &mut staged) else {
                break;
            };
            let body = match entry {
                ObjectEntry10_1::Null => None,
                ObjectEntry10_1::BackReference {
                    class_id: STANDALONE_TABLE_CLASS_ID,
                    ..
                } => None,
                ObjectEntry10_1::NewObject {
                    class_id: STANDALONE_TABLE_CLASS_ID,
                    ..
                } => {
                    let Ok(body) = read_standalone_table_prefix_10_1(
                        &mut candidate,
                        &mut staged,
                        max_bytes,
                        max_bytes,
                    ) else {
                        break;
                    };
                    Some(body)
                }
                _ => break,
            };
            let partial = body
                .as_ref()
                .is_some_and(|body| body.stop != StandaloneTableStop10_1::Complete);
            standalone_tables.push((ObjectSlot10_1 { offset, entry }, body));
            trial = candidate;
            trial_table = staged;
            if partial {
                stop = Some(MapSuffixStop::StandaloneTableBody {
                    offset: trial.position(),
                });
                break;
            }
            if standalone_tables.len() == prefix.standalone_table_count as usize {
                stop = Some(MapSuffixStop::AfterStandaloneTables);
            }
        }
    }
    let mut legacy_table_names = Vec::new();
    if let Some(post) = post_bookmark.filter(|post| version < 11 && post.standalone_table_count > 0)
    {
        for _ in 0..post.standalone_table_count {
            let offset = trial.position();
            stop = Some(MapSuffixStop::LegacyTableName { offset });
            let mut candidate = trial.clone();
            let mut staged = trial_table.clone();
            let mut name_stop = None;
            let Ok(name) =
                read_tabular_name_slot_10_1(&mut candidate, max_bytes, &mut staged, &mut name_stop)
            else {
                break;
            };
            if name_stop.is_some() || matches!(name, DatasetNameMoreSlot10_1::Null) {
                break;
            }
            legacy_table_names.push((offset, name));
            trial = candidate;
            trial_table = staged;
        }
        if legacy_table_names.len() == post.standalone_table_count as usize {
            stop = Some(MapSuffixStop::AfterStandaloneTables);
        }
    }
    let at_illumination = matches!(
        stop,
        Some(
            MapSuffixStop::AfterPostBookmarkPrefix {
                standalone_table_count: 0
            } | MapSuffixStop::AfterStandaloneTables
        )
    );
    let legacy_illumination = if at_illumination && version < 4 {
        let offset = trial.position();
        trial.read_bytes(32).ok().map(|bytes| {
            stop = Some(MapSuffixStop::AfterIllumination);
            let values = std::array::from_fn(|i| {
                u64::from_le_bytes(bytes[i * 8..][..8].try_into().expect("固定长度"))
            });
            (offset, values)
        })
    } else {
        None
    };
    let illumination = if at_illumination && version >= 4 {
        let slot = read_illumination_slot(&mut trial, &mut trial_table).ok();
        if slot.is_some() {
            stop = Some(MapSuffixStop::AfterIllumination);
        }
        slot
    } else {
        None
    };
    let legacy_illumination_fields =
        if version < 5 && (illumination.is_some() || legacy_illumination.is_some()) {
            read_legacy_post_illumination_fields(&mut trial, version).ok()
        } else {
            None
        };
    let post_illumination = if illumination.is_some() && version >= 5 {
        read_post_illumination_fields_10_1(&mut trial, version).ok()
    } else {
        None
    };
    if post_illumination.is_some() {
        stop = Some(MapSuffixStop::AfterIlluminationFields);
    }
    if legacy_illumination_fields.is_some() || (version == 5 && post_illumination.is_some()) {
        stop = Some(MapSuffixStop::Complete);
    }
    let post_illumination_geometry = if version >= 6 && post_illumination.is_some() {
        read_post_illumination_geometry_10_1(
            &mut trial,
            &mut trial_table,
            max_bytes,
            map_prefix.version,
        )
        .ok()
        .flatten()
    } else {
        None
    };
    let post_illumination_null_geometry = post_illumination_geometry
        .as_ref()
        .and_then(|(slot, _)| (slot.entry == ObjectEntry10_1::Null).then_some(*slot));
    if post_illumination_geometry.is_some() {
        stop = Some(if post_illumination_null_geometry.is_some() {
            MapSuffixStop::AfterNullGeometry
        } else {
            MapSuffixStop::AfterGeometry
        });
    }
    let post_illumination_border = if post_illumination_geometry.is_some() {
        let offset = trial.position();
        read_map_decor_slot_10_1(
            &mut trial,
            &mut trial_table,
            max_bytes,
            DecorKind10_1::Border,
        )
        .ok()
        .flatten()
        .map(|slot| (offset, slot))
    } else {
        None
    };
    let post_illumination_null_border =
        post_illumination_border
            .as_ref()
            .and_then(|(offset, slot)| {
                matches!(slot, DecorSlot10_1::Null).then_some(ObjectSlot10_1 {
                    offset: *offset,
                    entry: ObjectEntry10_1::Null,
                })
            });
    if version == 6 && post_illumination_border.is_some() {
        stop = Some(MapSuffixStop::Complete);
    } else if post_illumination_border.is_some() {
        stop = Some(MapSuffixStop::AfterNullBorder);
    }
    let mut post_border_tail = PostBorderTail10_1::default();
    if version >= 7 && post_illumination_border.is_some() {
        // 此处已到达原生 version > 6 的分支；每项独立提交，
        // 未知对象或短读时停在该项起点。
        macro_rules! next_short {
            ($field:ident, $stage:expr) => {
                if ($stage == 0 && matches!(stop, Some(MapSuffixStop::AfterNullBorder)))
                    || matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: $stage }))
                {
                    let mut candidate = trial.clone();
                    let offset = candidate.position();
                    if let Ok(value) = candidate.read_u16_le() {
                        trial = candidate;
                        post_border_tail.$field = Some((offset, value));
                        stop = Some(MapSuffixStop::AfterBorderTail { stage: $stage + 1 });
                    }
                }
            };
        }
        next_short!(short_308, 0);
        if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 1 })) {
            let mut candidate = trial.clone();
            let offset = candidate.position();
            if let Ok(header) = read_header_10_1(&mut candidate)
                && let Some(entry) = match header {
                    ObjectHeader::Null => Some(ObjectEntry10_1::Null),
                    ObjectHeader::Object { class_id, key }
                        if class_id == crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID
                            && trial_table.contains(key) =>
                    {
                        Some(ObjectEntry10_1::BackReference { class_id, key })
                    }
                    _ => None,
                }
            {
                trial = candidate;
                post_border_tail.layer_30c = Some(ObjectSlot10_1 { offset, entry });
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 2 });
            }
        }
        next_short!(short_310, 2);
        if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 3 })) {
            let mut candidate = trial.clone();
            let mut refs = trial_table.clone();
            let offset = candidate.position();
            if let Ok(entry) = read_object_entry_10_1(&mut candidate, &mut refs)
                && let Ok(parsed) = match entry {
                    ObjectEntry10_1::Null
                    | ObjectEntry10_1::BackReference {
                        class_id: ENVELOPE_CLASS_ID,
                        ..
                    } => Ok(None),
                    ObjectEntry10_1::NewObject {
                        class_id: ENVELOPE_CLASS_ID,
                        ..
                    } => read_envelope_10_1(&mut candidate, max_bytes, &mut refs).map(Some),
                    _ => Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "Map 边框后的 Envelope 对象类尚未支持",
                    )),
                }
            {
                trial = candidate;
                trial_table = refs;
                post_border_tail.envelope_314 = Some(ObjectSlot10_1 { offset, entry });
                post_border_tail.envelope_314_parsed = parsed;
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 4 });
            }
        }
        next_short!(short_318, 4);
        next_short!(short_31a, 5);
        if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 6 })) {
            let mut candidate = trial.clone();
            let offset = candidate.position();
            if let Ok(bytes) = candidate.read_bytes(8) {
                let bits = u64::from_le_bytes(bytes.try_into().expect("固定长度"));
                trial = candidate;
                post_border_tail.double_bits = Some((offset, bits));
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 7 });
            }
        }
        macro_rules! next_raw {
            ($field:ident, $stage:expr, $width:expr, $ty:ty) => {
                if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: $stage })) {
                    let mut candidate = trial.clone();
                    let offset = candidate.position();
                    if let Ok(bytes) = candidate.read_bytes($width) {
                        let value = <$ty>::from_le_bytes(bytes.try_into().expect("固定长度"));
                        trial = candidate;
                        post_border_tail.$field = Some((offset, value));
                        stop = Some(MapSuffixStop::AfterBorderTail { stage: $stage + 1 });
                    }
                }
            };
        }
        if version >= 8 {
            next_raw!(int_1a4, 7, 4, u32);
            next_raw!(int_1a8, 8, 4, u32);
        }
        if version >= 9 {
            next_raw!(temporary_short, 9, 2, u16);
        }
        if version >= 10 {
            next_raw!(temporary_double_bits, 10, 8, u64);
        }
        if version >= 12 && matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 11 })) {
            let mut candidate = trial.clone();
            let offset = candidate.position();
            if let Ok(count) = read_loop_count(&mut candidate, "坐标变换", max_bytes) {
                trial = candidate;
                post_border_tail.transformation_count = Some((offset, count));
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 12 });
            }
        }
        if let Some((_, count)) = post_border_tail.transformation_count {
            for index in 0..count {
                let mut candidate = trial.clone();
                let offset = candidate.position();
                let Ok(prefix) = (version >= 14).then(|| candidate.read_u32_le()).transpose()
                else {
                    break;
                };
                trial = candidate;
                if index == 0 {
                    post_border_tail.first_transformation_prefix =
                        prefix.map(|value| (offset, value));
                }
                post_border_tail
                    .transformations
                    .push(MapTransformation10_1 {
                        offset,
                        prefix,
                        object: None,
                        body: None,
                    });
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 13 });
                let mut candidate = trial.clone();
                let mut candidate_table = trial_table.clone();
                let object_offset = candidate.position();
                let Ok(entry) = read_object_entry_10_1(&mut candidate, &mut candidate_table) else {
                    break;
                };
                let body = match entry {
                    ObjectEntry10_1::Null => None,
                    ObjectEntry10_1::BackReference { class_id, .. }
                        if supports_geo_transformation_body_10_1(class_id) =>
                    {
                        None
                    }
                    ObjectEntry10_1::NewObject { class_id, .. }
                        if supports_geo_transformation_body_10_1(class_id) =>
                    {
                        let Ok(body) =
                            read_geo_transformation_body_10_1(&mut candidate, class_id, max_bytes)
                        else {
                            break;
                        };
                        Some(body)
                    }
                    _ => break,
                };
                let item = post_border_tail
                    .transformations
                    .last_mut()
                    .expect("已添加条目");
                item.object = Some(ObjectSlot10_1 {
                    offset: object_offset,
                    entry,
                });
                item.body = body;
                trial = candidate;
                trial_table = candidate_table;
            }
            if post_border_tail.transformations.len() == count as usize
                && post_border_tail
                    .transformations
                    .iter()
                    .all(|item| item.object.is_some())
            {
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 14 });
            }
        }
        if version >= 13 {
            next_raw!(post_transform_double_bits, 14, 8, u64);
        }
        // 原生 0x79400301 和 0x7940032b 分别检查 short 与 IBackground 的版本门。
        if version >= 15 {
            next_short!(post_transform_short, 15);
        }
        if version >= 16 && matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 16 })) {
            let offset = trial.position();
            post_border_tail.post_transform_background = read_map_decor_slot_10_1(
                &mut trial,
                &mut trial_table,
                max_bytes,
                DecorKind10_1::Background,
            )
            .ok()
            .flatten()
            .map(|slot| (offset, slot));
            post_border_tail.post_transform_null_background = post_border_tail
                .post_transform_background
                .as_ref()
                .and_then(|(offset, slot)| {
                    matches!(slot, DecorSlot10_1::Null).then_some(ObjectSlot10_1 {
                        offset: *offset,
                        entry: ObjectEntry10_1::Null,
                    })
                });
            if post_border_tail.post_transform_background.is_some() {
                stop = Some(MapSuffixStop::AfterBorderTail { stage: 17 });
            }
        }
        if matches!(
            (version, stop),
            (7, Some(MapSuffixStop::AfterBorderTail { stage: 7 }))
                | (8, Some(MapSuffixStop::AfterBorderTail { stage: 9 }))
                | (9, Some(MapSuffixStop::AfterBorderTail { stage: 10 }))
                | (10 | 11, Some(MapSuffixStop::AfterBorderTail { stage: 11 }))
                | (12, Some(MapSuffixStop::AfterBorderTail { stage: 14 }))
                | (13 | 14, Some(MapSuffixStop::AfterBorderTail { stage: 15 }))
                | (15, Some(MapSuffixStop::AfterBorderTail { stage: 16 }))
                | (16 | 18, Some(MapSuffixStop::AfterBorderTail { stage: 17 }))
        ) {
            stop = Some(MapSuffixStop::Complete);
        }
        if (19..=22).contains(&version)
            && matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 17 }))
        {
            let mut candidate = trial.clone();
            let mut staged = trial_table.clone();
            if let Ok(annotate_map) =
                read_legacy_annotate_map(&mut candidate, max_bytes, &mut staged)
            {
                trial = candidate;
                trial_table = staged;
                post_border_tail.legacy_annotate_map = Some(annotate_map);
                stop = Some(MapSuffixStop::Complete);
            }
        }
        if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 17 })) && version >= 23 {
            let mut candidate = trial.clone();
            let mut candidate_table = trial_table.clone();
            let offset = candidate.position();
            if let Ok(entry) = read_object_entry_10_1(&mut candidate, &mut candidate_table) {
                let slot = ObjectSlot10_1 { offset, entry };
                match entry {
                    ObjectEntry10_1::Null => {
                        post_border_tail.post_transform_null_object = Some(slot);
                        trial = candidate;
                        trial_table = candidate_table;
                        stop = Some(MapSuffixStop::AfterBorderTail { stage: 18 });
                    }
                    ObjectEntry10_1::BackReference {
                        class_id:
                            BASIC_OVERPOSTER_PROPERTIES_CLASS_ID | MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID,
                        ..
                    } => {
                        post_border_tail.post_transform_overposter_reference = Some(slot);
                        trial = candidate;
                        trial_table = candidate_table;
                        stop = Some(MapSuffixStop::AfterBorderTail { stage: 18 });
                    }
                    ObjectEntry10_1::NewObject {
                        class_id: MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID,
                        ..
                    } => {
                        if let Ok(body) = read_maplex_overposter_properties(
                            &mut candidate,
                            max_bytes,
                            &mut candidate_table,
                        ) {
                            post_border_tail.post_transform_maplex = Some((slot, Box::new(body)));
                            trial = candidate;
                            trial_table = candidate_table;
                            stop = Some(MapSuffixStop::AfterBorderTail { stage: 18 });
                        }
                    }
                    ObjectEntry10_1::NewObject { class_id, .. }
                        if class_id == BASIC_OVERPOSTER_PROPERTIES_CLASS_ID =>
                    {
                        if let Ok(body) = read_basic_overposter_properties_10_1(
                            &mut candidate,
                            max_bytes,
                            &mut candidate_table,
                        ) {
                            post_border_tail.post_transform_overposter = Some((slot, body));
                            trial = candidate;
                            trial_table = candidate_table;
                            stop = Some(MapSuffixStop::AfterBorderTail { stage: 18 });
                        }
                    }
                    _ => {}
                }
            }
        }
        if map_prefix.version >= 24 {
            next_short!(short_32a, 18);
            next_raw!(double_320_bits, 19, 8, u64);
            next_short!(short_328, 20);
        }
        if map_prefix.version >= 26 {
            for (index, stage) in [(0, 21), (1, 22), (2, 24), (3, 25)] {
                if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: s }) if s == stage) {
                    let mut candidate = trial.clone();
                    let mut candidate_table = trial_table.clone();
                    let offset = candidate.position();
                    if let Ok(entry) = read_object_entry_10_1(&mut candidate, &mut candidate_table)
                    {
                        let body = match entry {
                            ObjectEntry10_1::Null => Some(None),
                            ObjectEntry10_1::BackReference { class_id, .. }
                                if class_id == ENVELOPE_CLASS_ID =>
                            {
                                Some(None)
                            }
                            ObjectEntry10_1::NewObject { class_id, .. }
                                if class_id == ENVELOPE_CLASS_ID =>
                            {
                                read_envelope_10_1(&mut candidate, max_bytes, &mut candidate_table)
                                    .ok()
                                    .map(Some)
                            }
                            _ => None,
                        };
                        if let Some(body) = body {
                            post_border_tail.scale_envelopes[index] =
                                Some(ObjectSlot10_1 { offset, entry });
                            post_border_tail.scale_envelope_bodies[index] = body;
                            trial = candidate;
                            trial_table = candidate_table;
                            stop = Some(MapSuffixStop::AfterBorderTail { stage: stage + 1 });
                        }
                    }
                }
                if index == 1 {
                    next_raw!(scale_double_2b8_bits, 23, 8, u64);
                }
            }
            next_raw!(scale_double_2d8_bits, 26, 8, u64);
        }
        if map_prefix.version >= 27 {
            for index in 0..8 {
                let stage = 27 + index as u8;
                if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: s }) if s == stage) {
                    let mut candidate = trial.clone();
                    let offset = candidate.position();
                    if let Ok(value) = candidate.read_u32_le() {
                        post_border_tail.scale_ints[index] = Some((offset, value));
                        trial = candidate;
                        stop = Some(MapSuffixStop::AfterBorderTail { stage: stage + 1 });
                    }
                }
            }
        }
        // 0x7940078c 之后的持久化字段按版本启用；未进入下一版本门即读完。
        if matches!(
            (version, stop),
            (23, Some(MapSuffixStop::AfterBorderTail { stage: 18 }))
                | (24 | 25, Some(MapSuffixStop::AfterBorderTail { stage: 21 }))
                | (26, Some(MapSuffixStop::AfterBorderTail { stage: 27 }))
                | (27, Some(MapSuffixStop::AfterBorderTail { stage: 35 }))
        ) {
            stop = Some(MapSuffixStop::Complete);
        }
        if map_prefix.version >= 28 {
            if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 35 })) {
                let mut candidate = trial.clone();
                let offset = candidate.position();
                if let Ok(version) = read_masking_version_10_1(&mut candidate) {
                    post_border_tail.masking_version = Some((offset, version));
                    trial = candidate;
                    stop = Some(MapSuffixStop::AfterBorderTail { stage: 36 });
                }
            }
            if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 36 })) {
                let mut candidate = trial.clone();
                let offset = candidate.position();
                if let Ok(count) = read_masking_count_10_1(&mut candidate, max_bytes) {
                    post_border_tail.masking_count = Some((offset, count));
                    trial = candidate;
                    stop = Some(MapSuffixStop::AfterBorderTail { stage: 37 });
                }
            }
            if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 37 })) {
                let mut candidate = trial.clone();
                let mut candidate_table = trial_table.clone();
                let version = post_border_tail.masking_version.unwrap().1;
                let count = post_border_tail.masking_count.unwrap().1.max(0) as usize;
                if let Ok(entries) = read_entries_10_1(
                    &mut candidate,
                    max_bytes,
                    &mut candidate_table,
                    version,
                    count,
                ) {
                    post_border_tail.masking_entries = Some(entries);
                    trial = candidate;
                    trial_table = candidate_table;
                }
            }
            if post_border_tail.masking_entries.is_some() {
                next_short!(masking_empty_short, 37);
            }
            // 原生 Map 0x79403d50 遍历与写图层列表相同的数组。
            if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 38 })) {
                let mut candidate = trial.clone();
                let mut candidate_refs = trial_table.clone();
                let records = read_map_layer_masking_10_1(
                    &mut candidate,
                    max_bytes,
                    &mut candidate_refs,
                    &layer_list,
                    map_prefix.version,
                );
                match records {
                    Err(reason) => post_border_tail.layer_masking_stop = Some(reason),
                    Ok(records) => {
                        let fields = (version >= 29)
                            .then(|| {
                                read_post_masking_map_fields_10_1(
                                    &mut candidate,
                                    version,
                                    max_bytes,
                                )
                            })
                            .transpose();
                        if let Ok(fields) = fields {
                            trial = candidate;
                            trial_table = candidate_refs;
                            post_border_tail.layer_masking = records;
                            post_border_tail.post_masking = fields;
                            // 版本 28..=32 的后续分支不消费持久化字段。
                            stop = Some(if map_prefix.version <= 32 {
                                MapSuffixStop::Complete
                            } else {
                                MapSuffixStop::AfterBorderTail { stage: 39 }
                            });
                        }
                    }
                }
            }
            if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 39 }))
                && let Ok(collections) = read_map_persist_collections_before_v34_10_1(
                    &mut trial,
                    max_bytes,
                    &mut trial_table,
                    map_prefix.version,
                )
            {
                if collections.stop.is_none() {
                    stop = Some(if map_prefix.version == 33 {
                        MapSuffixStop::Complete
                    } else {
                        MapSuffixStop::AfterBorderTail { stage: 40 }
                    });
                }
                post_border_tail.persist_collections = Some(collections);
            }
            if matches!(stop, Some(MapSuffixStop::AfterBorderTail { stage: 40 }))
                && let Ok(fields) = read_map_late_tail_10_1(
                    &mut trial,
                    &mut trial_table,
                    max_bytes,
                    map_prefix.version,
                )
            {
                if fields.stop.is_none() {
                    stop = Some(MapSuffixStop::Complete);
                }
                post_border_tail.late_tail = Some(fields);
            }
        }
    }
    let stop = stop.unwrap_or(MapSuffixStop::AfterBookmarks);

    let result = MapSuffix10_1 {
        legacy_masking_short,
        layer_list,
        base_graphics_layer,
        base_graphics_layer_parsed,
        map_surround_count,
        map_surrounds,
        map_surround_parsed,
        extent,
        extent_offset,
        raw_int32_pair,
        envelope_object,
        envelope_object_parsed,
        bookmark_count,
        bookmarks,
        bookmark_parsed,
        post_bookmark,
        standalone_tables,
        legacy_table_names,
        illumination,
        legacy_illumination,
        legacy_illumination_fields,
        post_illumination,
        post_illumination_geometry,
        post_illumination_null_geometry,
        post_illumination_null_border,
        post_illumination_border,
        post_border_tail,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok((map_prefix, result))
}

// 0x79403ec0 对 v20–v27 先读零 LONG，v21 起读 short，v25–v27 再读零 LONG。
fn read_legacy_masking(reader: &mut Reader<'_>, version: u16) -> io::Result<Option<u16>> {
    if reader.read_u32_le()? != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "旧 Map 遮罩 LONG 非零",
        ));
    }
    let short = (version >= 21).then(|| reader.read_u16_le()).transpose()?;
    if (25..=27).contains(&version) && reader.read_u32_le()? != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "旧 Map 遮罩第二个 LONG 非零",
        ));
    }
    Ok(short)
}

fn read_legacy_annotate_map(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectSlot10_1, Option<LegacyAnnotateMapBody10_1>)> {
    const ANNOTATE_MAP: [u8; 16] = [
        0x02, 0x90, 0x43, 0x8c, 0xec, 0x14, 0xd2, 0x11, 0xa2, 0x7e, 8, 0, 9, 0xb6, 0xf2, 0x2b,
    ];
    let offset = reader.position();
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id: ANNOTATE_MAP | MAPLEX_ANNOTATE_MAP_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject {
            class_id: ANNOTATE_MAP,
            ..
        } => {
            // Annotation.dll 0x6b1c6910 只读有符号的类版本。
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "AnnotateMap 版本超过 1",
                ));
            }
            Some(LegacyAnnotateMapBody10_1::AnnotateMap { version })
        }
        ObjectEntry10_1::NewObject {
            class_id: MAPLEX_ANNOTATE_MAP_CLASS_ID,
            ..
        } => {
            let body = read_maplex_annotate_map(reader, limit, refs)?;
            Some(LegacyAnnotateMapBody10_1::Maplex(Box::new(body)))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的 IAnnotateMap 类",
            ));
        }
    };
    Ok((ObjectSlot10_1 { offset, entry }, body))
}

/// 从已确认的照明槽末尾读取 Map 版本 5..=51 的定长成员；失败不推进游标。
/// 调用方须先按完整对象流顺序确认版本、照明槽和本段起点。
pub fn read_post_illumination_fields_10_1(
    reader: &mut Reader<'_>,
    map_version: u16,
) -> io::Result<PostIlluminationFields10_1> {
    if !(5..=51).contains(&map_version) {
        return Err(io::Error::new(ErrorKind::InvalidData, "不支持的 Map 版本"));
    }
    let mut trial = reader.clone();
    let fields = PostIlluminationFields10_1 {
        offset: trial.position(),
        short_1a0: trial.read_u16_le()?,
        discarded_short: trial.read_u16_le()?,
        int_1b0: trial.read_u32_le()?,
        short_1ac: trial.read_u16_le()?,
        short_268: trial.read_u16_le()?,
    };
    *reader = trial;
    Ok(fields)
}

fn read_legacy_post_illumination_fields(
    reader: &mut Reader<'_>,
    version: u16,
) -> io::Result<LegacyPostIlluminationFields10_1> {
    let mut trial = reader.clone();
    let fields = LegacyPostIlluminationFields10_1 {
        offset: trial.position(),
        short_1a0: (version >= 2).then(|| trial.read_u16_le()).transpose()?,
        discarded_short: (version >= 2).then(|| trial.read_u16_le()).transpose()?,
        int_1b0: (version >= 2).then(|| trial.read_u32_le()).transpose()?,
        short_1ac: (version >= 3).then(|| trial.read_u16_le()).transpose()?,
    };
    *reader = trial;
    Ok(fields)
}

/// 从照明后定长字段末尾试读 IGeometry 槽的空引用。
///
/// 仅在调用方已确认 Map 版本及本槽起点时使用；非空对象的接口兼容性和
/// 对象体尚未闭合，返回 `None` 并保持游标不变。截断同样不推进游标。
pub fn read_post_illumination_null_geometry_10_1(
    reader: &mut Reader<'_>,
    map_version: u16,
) -> io::Result<Option<ObjectSlot10_1>> {
    if !(23..=51).contains(&map_version) {
        return Err(io::Error::new(ErrorKind::InvalidData, "不支持的 Map 版本"));
    }
    read_null_object_slot_10_1(reader)
}

/// 读取 Map.dll 10.1 在 0x793fffe1 请求的 IGeometry 槽。
/// 非空未知类停在槽起点，已知类只在完整读取后提交引用。
fn read_post_illumination_geometry_10_1(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max_bytes: usize,
    map_version: u16,
) -> io::Result<Option<(ObjectSlot10_1, Option<MapGeometryBody10_1>)>> {
    if !(6..=51).contains(&map_version) {
        return Err(io::Error::new(ErrorKind::InvalidData, "不支持的 Map 版本"));
    }
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let offset = trial.position();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let parsed = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id:
                ENVELOPE_CLASS_ID
                | POINT_CLASS_ID
                | MULTIPOINT_CLASS_ID
                | POLYLINE_CLASS_ID
                | POLYGON_CLASS_ID
                | GEOMETRY_BAG_CLASS_ID
                | MULTIPATCH_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject {
            class_id: ENVELOPE_CLASS_ID,
            ..
        } => Some(MapGeometryBody10_1::Envelope(read_envelope_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (POINT_CLASS_ID
                | MULTIPOINT_CLASS_ID
                | POLYLINE_CLASS_ID
                | POLYGON_CLASS_ID
                | GEOMETRY_BAG_CLASS_ID
                | MULTIPATCH_CLASS_ID),
            ..
        } => Some(MapGeometryBody10_1::Geometry(read_geometry_body_10_1(
            &class_id,
            &mut trial,
            max_bytes,
            &mut staged,
        )?)),
        _ => return Ok(None),
    };
    *reader = trial;
    *refs = staged;
    Ok(Some((ObjectSlot10_1 { offset, entry }, parsed)))
}

fn read_null_object_slot_10_1(reader: &mut Reader<'_>) -> io::Result<Option<ObjectSlot10_1>> {
    let mut trial = reader.clone();
    let offset = trial.position();
    if trial.read_bytes(16)?.iter().any(|&byte| byte != 0) {
        return Ok(None);
    }
    *reader = trial;
    Ok(Some(ObjectSlot10_1 {
        offset,
        entry: ObjectEntry10_1::Null,
    }))
}

fn read_post_bookmark_prefix(
    trial: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<PostBookmarkPrefix10_1> {
    let offset = trial.position();
    let leading_short = trial.read_u16_le()?;
    let short_19c = trial.read_u16_le()?;
    let double_200 = trial.read_f64_le()?;
    let short_264 = trial.read_u16_le()?;
    let standalone_table_count = read_loop_count(trial, "独立表", max_bytes)?;
    Ok(PostBookmarkPrefix10_1 {
        offset,
        leading_short,
        short_19c,
        double_200,
        short_264,
        standalone_table_count,
    })
}

fn read_illumination_slot(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<IlluminationSlot10_1> {
    // 先读头和体，完整识别后才提交游标和新键；未知体不能安全跳过。
    let mut trial = reader.clone();
    let offset = trial.position();
    let (entry, body) = match read_header_10_1(&mut trial)? {
        ObjectHeader::Null => (ObjectEntry10_1::Null, None),
        ObjectHeader::Object { class_id, key }
            if class_id == ILLUMINATION_PROPS_CLASS_ID && table.contains(key) =>
        {
            (ObjectEntry10_1::BackReference { class_id, key }, None)
        }
        ObjectHeader::Object { class_id, key } if class_id == ILLUMINATION_PROPS_CLASS_ID => {
            let version = trial.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "不支持的 IlluminationProps 版本",
                ));
            }
            let point = [
                trial.read_f64_le()?,
                trial.read_f64_le()?,
                trial.read_f64_le()?,
            ];
            let value = trial.read_f64_le()?;
            table.register(key);
            (
                ObjectEntry10_1::NewObject { class_id, key },
                Some(IlluminationProps10_1 {
                    version,
                    point,
                    value,
                }),
            )
        }
        _ => {
            return Err(io::Error::new(ErrorKind::InvalidData, "未知的照明属性对象"));
        }
    };
    *reader = trial;
    Ok(IlluminationSlot10_1 {
        object: ObjectSlot10_1 { offset, entry },
        body,
    })
}

/// 读取后缀循环（MapSurround / 书签）的 int32 计数：拒绝负数并预检限额。
///
/// 原生读取仅按有符号比较决定是否进入循环（写侧计数非负）；负数语义未知，
/// 这里显式拒绝，与 maps_stream 的数量处理保持同一严格程度。
fn read_loop_count(trial: &mut Reader<'_>, label: &str, max_bytes: usize) -> io::Result<i32> {
    let offset = trial.position();
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 后缀 {label} 计数 offset 0x{offset:x}: 计数为负数（{count}）；原生 \
                 读取仅正数进入循环，负数语义未知，本读取器拒绝"
            ),
        ));
    }
    let count_usize = usize::try_from(count).expect("计数已确认为非负");
    let min_total = count_usize
        .checked_mul(MIN_LOOP_ENTRY_BYTES)
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("Map 后缀 {label} 计数 offset 0x{offset:x}: 计数 × 最小条目字节数溢出"),
            )
        })?;
    if min_total > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 后缀 {label} 计数 offset 0x{offset:x}: 声明计数 {count} × 最小条目 \
                 字节数 {MIN_LOOP_ENTRY_BYTES} 超过资源限额 {max_bytes}，在条目读取之前拒绝"
            ),
        ));
    }
    Ok(count)
}

/// MapSurround 独立对象体的停止原因；宿主决定停止时是否提交前缀。
#[derive(Debug, Clone, PartialEq)]
pub(super) enum SurroundBodyStop10_1 {
    Legend(LegendStop10_1),
    Family(FamilyStop10_1),
    ScaleLine(ScaleLineStop10_1),
}

/// 允许 Map 和 PageLayout 使用同一套装饰物类读取器。
pub(super) fn read_map_surround_body_10_1(
    class_id: [u8; 16],
    key: u32,
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(MapSurroundElement10_1, Option<SurroundBodyStop10_1>)> {
    use MapSurroundElement10_1 as Element;
    use SurroundBodyStop10_1 as Stop;
    let (element, stop) = match class_id {
        LEGEND_CLASS_ID => {
            let body = read_legend_prefix_10_1(reader, limit, refs)?;
            let stop =
                (body.stop != LegendStop10_1::Completed).then(|| Stop::Legend(body.stop.clone()));
            (Element::Legend(Box::new(body)), stop)
        }
        MAP_TITLE_CLASS_ID => {
            let body = read_map_title_body_10_1(reader, limit, refs, key)?;
            (Element::MapTitle(Box::new(body)), None)
        }
        SCALEBAR_CLASS_ID => {
            let body = read_scalebar_body_10_1(reader, limit, refs, key)?;
            let stop = body.stop.map(Stop::Family);
            (Element::Scalebar(Box::new(body)), stop)
        }
        SCALE_TEXT_CLASS_ID => {
            let body = read_scale_text_body_10_1(reader, limit, refs, key)?;
            let stop = body.stop.map(Stop::Family);
            (Element::ScaleText(Box::new(body)), stop)
        }
        MARKER_NORTH_ARROW_CLASS_ID => {
            let body = read_marker_north_arrow_body_10_1(reader, limit, refs, key)?;
            let stop = body.stop.map(Stop::Family);
            (Element::MarkerNorthArrow(Box::new(body)), stop)
        }
        SCALE_LINE_CLASS_ID | STEPPED_SCALE_LINE_CLASS_ID => {
            let read = if class_id == SCALE_LINE_CLASS_ID {
                read_scale_line_10_1
            } else {
                read_stepped_scale_line_10_1
            };
            let body = read(reader, limit, refs, key)?;
            let stop = body.stop.map(Stop::ScaleLine);
            (Element::ScaleLine(Box::new(body)), stop)
        }
        _ => {
            let Some(kind) = scalebar_subclass_kind(&class_id) else {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "未支持的 IMapSurround 类 {}，键 {key}",
                        class_id_hex(&class_id)
                    ),
                ));
            };
            let body = read_scalebar_subclass_body_10_1(reader, limit, refs, key, kind)?;
            let stop = body.stop.map(Stop::Family);
            (
                Element::ScalebarSubclass(ScalebarSubclassElement10_1 { kind, body }),
                stop,
            )
        }
    };
    Ok((element, stop))
}

/// 按流内类标识映射比例尺子类种类（C 轮接线；四类常量与映射依据
/// docs/arcmap-10.1-legend-family.md）。
fn scalebar_subclass_kind(class_id: &[u8; 16]) -> Option<ScalebarSubclassKind10_1> {
    if *class_id == HOLLOW_SCALEBAR_CLASS_ID {
        Some(ScalebarSubclassKind10_1::Hollow)
    } else if *class_id == SINGLE_DIVISION_SCALEBAR_CLASS_ID {
        Some(ScalebarSubclassKind10_1::SingleDivision)
    } else if *class_id == ALTERNATING_SCALEBAR_CLASS_ID {
        Some(ScalebarSubclassKind10_1::Alternating)
    } else if *class_id == DOUBLE_ALTERNATING_SCALEBAR_CLASS_ID {
        Some(ScalebarSubclassKind10_1::DoubleAlternating)
    } else {
        None
    }
}
