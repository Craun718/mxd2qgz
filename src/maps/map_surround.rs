//! ArcMap 10.1 MapSurround 与空间书签非空循环元素体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-mapsurround-bookmark.md`（2026-11-14 本轮研究
//! 记录）。核心结论全部来自 Map.dll 10.1.0.3035（SHA-256 `1c0675cc…b1bb8`，
//! temp/agent-map-surround/ 本轮提取复核）与 AfCore.dll 10.1.0.3035
//! （`031858c4…7c17`，temp/arcgis101-bin 既有基线）的静态反编译，接口名称
//! 经 10.1 类型库（wine `LoadTypeLibEx` dump，esriCarto）核对。
//!
//! **重要边界：仓库参考样本 `samples/测试底图.mxd` 的 MapSurround 计数
//! （`/Maps:0x24f60`）与书签计数（`0x24f9c`）均为 0，以下全部结论无样本
//! 字节佐证**；本模块只能用合成输入测试，不得声称样本验证。循环结构本身
//! （int32 计数 + 每元素普通 `LoadObject`，20 字节对象头 + 体，无 ExtraSafe
//! 8 字节前缀）与 `map_object::read_map_suffix_10_1` 现有骨架逐条吻合，
//! 衔接点为 [`crate::maps::map_object::MapSuffixStop::MapSurroundBody`] 与
//! [`crate::maps::map_object::MapSuffixStop::BookmarkBody`] 携带的流内类标识
//! （按 CLSID 分派到本模块各读取器）。
//!
//! ## 循环结构（Map::Load `0x793ff4d0` / Map::Save `0x79401850`，确定的）
//!
//! - 第 5 步（IMapSurround，请求 IID `{FE3115E9-…}`，`DAT_79481a84`）：
//!   `AfStreamRead(4)` int32 计数 + 循环 `LoadObject`；元素经 `(this−0x40)`
//!   子接口虚表 `+0x84` 加入集合。
//! - 第 9 步（ISpatialBookmark，请求 IID `{EE53528B-…}`，`DAT_794815a0`）：
//!   int32 计数 + 循环 `LoadObject`；每元素 QI（`{3036D35F-…}`
//!   IFeatureBookmark，第五轮已闭合）后 `FUN_7941c760` 加入集合。
//! - 写侧对称：MapSurround `int32(*(this+0x130)) + 循环 SaveObject`（数组
//!   `*(this+0x12c)`）；书签 `int32(*(this+0x174)) + 循环 SaveObject`（数组
//!   `*(this+0x170)`）。两循环均无 ExtraSafe 前缀。
//!
//! ## MapSurround 族共同前缀（Load `0x794413d0` / Save `0x79441510`）
//!
//! 装饰物族各类（Legend/Scalebar/ScaleText/MarkerNorthArrow/MapTitle 等）
//! 的 `IPersistStream::Load` 先调共享基类前缀读取（`0x794413d0` 共 9 个
//! 调用者，含 Legend `0x793da2f2`、Scalebar `0x794460fd`），再读各自尾段：
//!
//! 1. int16 版本，按有符号值判断后续字段。写侧版本表 `DAT_79480d58`
//!    为 `[2,2,2,2,3]`，10.1 写 3；
//! 2. String（名称 → `this−0xa0`，AfCore String::Read 布局）；
//! 3. `LoadObject`（请求 IID `{E6BDAA75-…}` IMap）：关联地图对象。流内为
//!    对已持久化 Map 的键回引头（20 字节，无体）；空标记与回引之外的新对象
//!   属未预期形态，本读取器报不支持错误；
//! 4. 版本 ≥2：FixedAspectRatio 的 int16 位模式（`this−0x8c`）；
//! 5. 版本 ≥3：32 字节 WKSEnvelope；原生 Load 读入栈后丢弃，Save 取
//!    FitToBounds 更新的 `this−0x84`。读取器原样保留字节。
//!
//! ## AOIBookmark（空间书签，体完全闭合）
//!
//! CLSID `{BBB1AE73-41E3-11D2-AE1E-080009EC732A}`（esri.clsid 声明 Map.dll；
//! GetClassID `0x793a2600` 写出该 CLSID，虚表 `0x79481550`，adjustor 0x14）。
//! Load `0x793a2cd0`：① int16 版本（有符号比较仅拒绝 >1）→ ② `LoadObject`（请求 IID `{1B077914-…}` IEnvelope，
//! 位置包络）→ ③ String（书签名称）。Save `0x793a2d60` 恒写版本 1
//! （`DAT_79480eac`）+ `SaveObject` + `String::Write`，读写对称。
//!
//! ## FeatureBookmark（要素书签，体完全闭合）
//!
//! CLSID `{EC65B35B-4342-11D2-AE22-080009EC732A}`（GetClassID `0x793b5520`，
//! 虚表 `0x79482be8`）。Load `0x793b5c50`：① int16 版本（有符号 ≤1；本
//! 读取器仅接受 0..=1）→ ② int32
//! （`this+4`，IFeatureBookmark.FeatureId，原样保留）→ ③ `LoadObject`（请求 IID
//! `{4BA33331-…}` IName，名称对象）→ ④ String（书签名称）。Save
//! `0x793b5d70` 恒写版本 1（`DAT_79480e94`）+ int32 + `SaveObject` +
//! `String::Write`，对称。名称对象按已闭合且完整消费的实现分派，包括要素
//! 类、表类、查询表和事件源等；证据与版本门差异见
//! `docs/mxd-terra-round9-bookmark.md`。
//!
//! ## Legend（图例，已确认前缀）
//!
//! CLSID `{7A3F91E4-B9E3-11D1-8756-0000F875172A}`（GetClassID `0x793dede0`
//! → `0x793d1330` 写出，IPersistStream 虚表 `0x794859c0`，COL
//! `0x79493048` disp 0x1f4）。Load（虚表槽 `0x793ded80` → `0x793da250`）：
//!
//! 1. int16 Legend 版本：汇编 `CMP word,5; JLE` 有符号比较，>5 拒绝
//!    `0x80004005`（负值原生通过，本读取器一并拒绝，较原生更严）。写侧
//!    版本表 `DAT_79480dc4` = `[3,4,4,4,4,5]`（`AfStreamGetSaveVersion
//!    (docVer, 6, 表)`），10.1 恒写 5；
//! 2. 共同前缀（上节 5 字段）；
//! 3. `LoadObject`（请求 IID `{52AA5BC1-…}` ILegendFormat）：新对象类标识
//!    为 LegendFormat `{7A3F91E5-…}` 时分派到
//!    [`crate::maps::map_surround_family::read_legend_format_body_10_1`]；其他
//!    未闭合实现类在对象头后停止并保留剩余；
//! 4. String（`this−0xf8`，标题候选）；
//! 5. 版本 ==1：double ×2（读后丢弃的旧版遗留段，原样保留位模式；写侧
//!    无对应写出）；
//! 6. int16 ×3（`this−0xf0/−0xee/−0xec`，原样保留）；
//! 7. int16（读入临时变量，10.1 Load 未使用该值；随后调用虚表 `+0xa8`
//!    清空原有条目；写侧恒写 `0xFFFF`）；
//! 8. int32 计数 + 循环 `LoadObject`（请求 IID `{52AA5BC0-…}` ILegendItem）：
//!    空标记 / 回引条目消费后继续；新对象且类标识为 HorizontalLegendItem 时
//!    整体解析其体（接线轮接入 `legend_item` 模块，体内停止透传），其余
//!    LegendItem 实现类新对象在该条目停止并保留剩余；
//! 9. 版本 ≥3：int16（`this−0xe8`）；版本 ≥4：int16（`this−0xe6`）；
//! 10. 版本 ≥5：int16 ×3 + double（`this−0x9c..−0x94`；写侧 docVersion≥5
//!     才写，与版本 5 恒写一致）；<5 时原生写默认值（不读流）。
//!
//! Save（虚表槽 `0x793df010` → `0x793da650`）逐段对称（含 `0xFFFF` short、
//! int32 计数 + 循环 SaveObject 条目）。
//!
//! 调用约定与事务性：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头、比对相应 `*_CLASS_ID` 常量确认类身份，从对象头之后的位置
//! 传入 Reader；本模块不重复读取对象头。全部字段在 trial 游标与 trial
//! 登记表上读取，完整成功（含停止点返回）才一并提交；任何错误既不推进
//! 调用者游标、也不修改登记表。`max_bytes` 约束各 String 声明长度与循环
//! 「计数 × 最小条目字节数」预检，`usize::MAX` 表示不额外设限。这是仅
//! 覆盖上述字段的严格安全子集：无样本字节佐证，未接入 `parse` 命令与
//! 中间模型，人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见研究记录开放项）：LegendFormat、
//! LegendItem 及装饰物的若干标量字段未定性；此前被列为 ScaleLine 候选的
//! 三个共同前缀调用者分别属于 ImageInset、MapInset、Overview，其对象接口
//! 调用语义仍未闭合（见 `docs/mxd-terra-integrate-page.md`）；共同前缀版本 0/1
//! （docVersion 83..93 写 2）无门字段差异之外的真实流字节佐证。

use crate::layers::cache_tile_reader::{
    AGS_SERVER_CONNECTION_NAME, AGS_SERVER_OBJECT_NAME, AgsServerConnectionName10_1,
    AgsServerObjectName10_1, read_ags_server_connection_name_body,
    read_ags_server_object_name_body,
};
use crate::layers::feature_layer::{
    COVERAGE_FEATURE_CLASS_NAME_CLASS_ID, FEATURE_CLASS_NAME_CLASS_ID,
    FGDB_FEATURE_CLASS_NAME_CLASS_ID, FeatureClassNamePrefix10_1,
    read_coverage_feature_class_name_body_10_1, read_feature_class_name_body_10_1,
    read_fgdb_feature_class_name_body_10_1,
};
use crate::layers::fgdb_raster_dataset_name::FGDB_RASTER_DATASET_NAME_CLASS_ID;
use crate::layers::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
use crate::layers::legend_item::{
    HORIZONTAL_LEGEND_ITEM_CLASS_ID, HorizontalLegendItemBody10_1, HorizontalLegendItemStop10_1,
    read_horizontal_legend_item_body_10_1,
};
use crate::layers::legend_item_family::{
    HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID, HorizontalBarLegendItemBody10_1,
    NESTED_LEGEND_ITEM_CLASS_ID, NestedLegendItemBody10_1, VERTICAL_LEGEND_ITEM_CLASS_ID,
    VerticalLegendItemBody10_1, read_horizontal_bar_legend_item_body_10_1,
    read_nested_legend_item_body_10_1, read_vertical_legend_item_body_10_1,
};
use crate::layers::na_layer::{
    NETWORK_DATASET_NAME_CLASS_ID, NetworkDatasetName10_1, read_network_dataset_name,
};
use crate::layers::raster_dataset_name::RASTER_CATALOG_NAME_CLASS_ID;
use crate::layers::raster_dataset_name::RasterDatasetWorkspaceSlot10_1;
use crate::layers::raster_dataset_name::{
    ACCESS_RASTER_DATASET_NAME_CLASS_ID, AIS_IMAGE_SERVICE_NAME_CLASS_ID,
    CACHE_RASTER_DATASET_NAME_CLASS_ID, FUNCTION_RASTER_DATASET_NAME_CLASS_ID,
    INTERNET_TILED_LAYER_NAME_CLASS_ID, IS_RASTER_DATASET_NAME_CLASS_ID,
    MOSAIC_DATASET_NAME_CLASS_ID, NETCDF_RASTER_DATASET_NAME_CLASS_ID,
    RASTER_BAND_COLLECTION_NAME_CLASS_ID, RASTER_BAND_NAME_CLASS_ID, RASTER_DATASET_NAME_CLASS_ID,
    RASTER_DATASET_NAME83_CLASS_ID, RASTER_TYPE_NAME_CLASS_ID, RasterDatasetNameEntryBody10_1,
    SDE_RASTER_DATASET_NAME_CLASS_ID, read_supported_raster_name_body_10_1,
};
use crate::layers::scene_layer_properties::{
    TIN_NAME_CLASS_ID, TinName10_1, read_surface_workspace_name_10_1, read_tin_name_body_10_1,
};
use crate::layers::terrain_layer::{
    TERRAIN_NAME_CLASS_ID, TerrainName10_1, read_terrain_name_body_10_1,
};
use crate::layers::wcs_name::{
    WCS_CONNECTION_NAME_CLASS_ID, WCS_COVERAGE_NAME_CLASS_ID, WMS_CONNECTION_NAME_CLASS_ID,
    WMTS_CONNECTION_NAME_CLASS_ID,
};
use crate::maps::map_surround_family::{
    FamilyStop10_1, LEGEND_FORMAT_CLASS_ID as FAMILY_LEGEND_FORMAT_CLASS_ID, LegendFormatBody10_1,
    read_legend_format_body_10_1,
};
use crate::maps::standalone_table::{
    FGDB_TABLE_NAME_CLASS_ID, OBJECT_CLASS_NAME_CLASS_ID, TABLE_NAME_CLASS_ID,
    read_fgdb_table_name_body_10_1, read_table_name_body_10_1,
};
use crate::objects::ams_dataset_name::{
    AMS_DATASET_NAME_CLASS_ID, AmsDatasetName10_1, read_ams_dataset_name_10_1,
};
use crate::objects::dataset_name::{
    COVERAGE_NAME_CLASS_ID, FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1,
    TOPOLOGY_NAME_CLASS_ID, TopologyNameBody10_1, read_coverage_name_body_10_1,
    read_feature_dataset_name_body_10_1, read_topology_name_body_10_1,
};
use crate::objects::dataset_name_more::{
    CAD_DRAWING_NAME_CLASS_ID, CadDrawingNameBody10_1, DB_TABLE_NAME_CLASS_ID, DbTableNameBody10_1,
    FEATURE_QUERY_NAME_CLASS_ID, FeatureQueryNameBody10_1, GEOMETRIC_NETWORK_NAME_CLASS_ID,
    GeometricNetworkNameBody10_1, MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID,
    MemoryRelationshipClassNameBody10_1, NETCDF_FEATURE_CLASS_NAME_CLASS_ID,
    NETCDF_TABLE_NAME_CLASS_ID, NetCdfFeatureClassNameBody10_1, NetCdfTableNameBody10_1,
    QUERY_TABLE_NAME_CLASS_ID, QueryTableNameBody10_1, REFERENCED_TABLE_NAME_CLASS_ID,
    REL_QUERY_TABLE_NAME_CLASS_ID, RELATIONSHIP_CLASS_NAME_CLASS_ID, ReferencedTableNameBody10_1,
    RelQueryTableNameBody10_1, RelationshipClassNameBody10_1,
    SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID, SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID,
    SchematicClassNameBody10_1, TABLE_QUERY_NAME_CLASS_ID, TableQueryNameBody10_1,
    XY_EVENT_SOURCE_NAME_CLASS_ID, XyEventSourceNameBody10_1, read_cad_drawing_name_body_10_1,
    read_db_table_name_body_10_1, read_feature_query_name_body_10_1,
    read_geometric_network_name_body_10_1, read_memory_relationship_class_name_body_10_1,
    read_netcdf_feature_class_name_body_10_1, read_netcdf_table_name_body_10_1,
    read_query_table_name_body_10_1, read_referenced_table_name_body_10_1,
    read_rel_query_table_name_body_10_1, read_relationship_class_name_body_10_1,
    read_schematic_class_name_body_10_1, read_table_query_name_body_10_1,
    read_xy_event_source_name_body_10_1,
};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_dataset_name_slot_10_1};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::route_event_name::{
    ROUTE_EVENT_SOURCE_NAME_CLASS_ID, RouteEventSourceNameBody10_1,
    read_route_event_source_name_body_10_1,
};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// AOIBookmark（空间书签）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{BBB1AE73-41E3-11D2-AE1E-080009EC732A}`。证据：
/// esri.clsid.xml 声明 Map.dll 模块类名 `AOIBookmark`；Map.dll 10.1.0.3035
/// GetClassID `0x793a2600` 写出该 CLSID（`DAT_79481590`）。样本中无实例
/// （书签计数 0），无字节佐证。
pub const AOI_BOOKMARK_CLASS_ID: [u8; 16] = [
    0x73, 0xae, 0xb1, 0xbb, 0xe3, 0x41, 0xd2, 0x11, 0xae, 0x1e, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// FeatureBookmark（要素书签）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{EC65B35B-4342-11D2-AE22-080009EC732A}`。证据：
/// esri.clsid.xml 声明 Map.dll；Map.dll GetClassID `0x793b5520` 写出
/// （`DAT_79482c18`）。样本中无实例，无字节佐证。
pub const FEATURE_BOOKMARK_CLASS_ID: [u8; 16] = [
    0x5b, 0xb3, 0x65, 0xec, 0x42, 0x43, 0xd2, 0x11, 0xae, 0x22, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// Legend（图例）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91E4-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.xml 声明 Map.dll；Map.dll GetClassID `0x793d1330` 写出
/// （`DAT_79485118`）。样本中无实例，无字节佐证。
pub const LEGEND_CLASS_ID: [u8; 16] = [
    0xe4, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// LegendFormat 类标识（CLSID）的 16 字节内存序，供诊断对照。
///
/// GUID 字符串形式为 `{7A3F91E5-B9E3-11D1-8756-0000F8751720}`（esri.clsid
/// 声明 Map.dll；Load `0x793e0c50` 及字段序见
/// `docs/arcmap-10.1-legend-family.md`）。
pub const LEGEND_FORMAT_CLASS_ID: [u8; 16] = [
    0xe5, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// 共同前缀与书签循环条目的最小字节数（16 字节空标记；回引/新对象头 20 字节）。
const MIN_ENTRY_BYTES: usize = 16;

/// 类标识的十六进制描述（诊断用，与 layer_list 的 class_id_hex 同构）。
fn class_id_hex(class_id: &[u8; 16]) -> String {
    class_id.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// MapSurround 族共同前缀第 3 段（关联地图）的分类结果。
///
/// 真实流中该槽为对已持久化 Map 对象的键回引（登记先于持久化，Map 恒先
/// 于其装饰物写出）；空标记表示无关联地图。新对象（整棵 Map 对象体）未
/// 预期，读取时直接报不支持错误，不产生本枚举值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapBackReference10_1 {
    /// 回引条目头携带的类标识（原样保留；应为 Map 的 CLSID，但不强校验）。
    pub class_id: [u8; 16],
    /// 回引目标键（应为本会话已登记的 Map 对象键）。
    pub key: u32,
}

/// MapSurround 族共同前缀（Load `0x794413d0`）的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MapSurroundCommonPrefix10_1 {
    /// int16 版本的无符号位模式。
    ///
    /// 原生按有符号值判断，≥2 追加 short，≥3 再追加 32 字节。
    pub version: u16,
    /// 装饰物名称（String::Read → `this−0xa0`）。
    pub name: String,
    /// 关联地图条目（请求 esriCarto `IMap`，IID
    /// `{E6BDAA75-4D35-11D0-98BE-00805F7CED21}`）：`None` = 16 字节空标记；
    /// `Some` = 20 字节回引。该成员语义由 Map.dll 10.1 Load 的接口 IID、
    /// 接口 setter 调用及 Save 对应对象指针的写出共同确认；回引目标类标识
    /// 仍原样保留。
    pub map_entry: Option<MapBackReference10_1>,
    /// 版本 ≥2 的 FixedAspectRatio 原始 int16 位模式（持久化接口 `this−0x8c`）。
    pub extra_short: Option<u16>,
    /// 版本 ≥3 的 WKSEnvelope 原始字节。Save 取 FitToBounds 更新的 `this−0x84`，Load 读入
    /// 局部 WKSEnvelope 后丢弃；不作为装饰物的当前范围。
    pub raw_envelope: Option<[u8; 32]>,
}

impl MapSurroundCommonPrefix10_1 {
    /// 返回流中写入的固定纵横比标志。旧版本未存该属性时返回 `None`。
    pub fn fixed_aspect_ratio(&self) -> Option<bool> {
        self.extra_short.map(|raw| raw != 0)
    }
}

/// AOIBookmark 位置包络槽（请求 IID `{1B077914-…}` IEnvelope）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkEnvelopeEntry10_1 {
    /// 16 字节全零类标识：空引用标记，无键、无对象体。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 未见过的新键且类标识为 Envelope：已读取完整 Envelope 对象体
    /// （[`crate::objects::envelope`] 覆盖全部版本分支，整体消费）。
    Envelope { key: u32, body: Envelope10_1 },
}

/// AOIBookmark 对象体（Load `0x793a2cd0` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct AoiBookmark10_1 {
    /// int16 版本位模式（按有符号上限 1 读取，Save 恒写 1）。
    pub version: u16,
    /// IAOIBookmark.Location 位置包络（第 2 字段）。
    pub envelope: BookmarkEnvelopeEntry10_1,
    /// 书签名称（第 3 字段，String::Read → `this+8`）。
    pub name: String,
    /// 体之后的剩余字节数（体无外部长度；正常输入应为 0）。
    pub remaining_bytes: usize,
}

const MAX_SHORTCUT_NAME_NESTING: usize = 16;

pub const SHORTCUT_NAME_CLASS_ID: [u8; 16] = [
    0xe4, 0x09, 0xeb, 0x3b, 0x41, 0x39, 0x07, 0x4a, 0x9d, 0x1a, 0xec, 0x2b, 0x43, 0xba, 0x7d, 0x50,
];

pub const FILE_NAME_CLASS_ID: [u8; 16] = [
    0xdf, 0x75, 0xda, 0x53, 0x1a, 0xd0, 0xd2, 0x11, 0x9f, 0x27, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0x9e,
];

pub const CADASTRAL_FABRIC_NAME_CLASS_ID: [u8; 16] = [
    0x20, 0x72, 0x94, 0x5c, 0xb5, 0xff, 0x71, 0x4e, 0xa0, 0x59, 0xaf, 0x20, 0x1a, 0xe0, 0x81, 0xa6,
];

#[derive(Debug, Clone, PartialEq)]
pub struct CadastralFabricName10_1 {
    pub version: i16,
    pub strings: [String; 2],
    pub dataset: DatasetNameMoreSlot10_1,
    pub workspace: RasterDatasetWorkspaceSlot10_1,
}

pub const REPRESENTATION_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0x22, 0xaa, 0x74, 0x4e, 0xb6, 0xa1, 0xd4, 0x47, 0x99, 0x14, 0x20, 0xf8, 0xc6, 0xc2, 0xb6, 0x99,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RepresentationClassName10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub workspace: RasterDatasetWorkspaceSlot10_1,
}

pub const LAS_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x94, 0xa9, 0x29, 0x20, 0xc0, 0x9b, 0x94, 0x40, 0xa1, 0xc7, 0xe5, 0x38, 0x18, 0x69, 0xe4, 0xe3,
];

#[derive(Debug, Clone, PartialEq)]
pub struct LasDatasetName10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub workspace: crate::layers::raster_dataset_name::RasterDatasetWorkspaceSlot10_1,
}

/// FeatureBookmark 名称对象槽（请求 IID `{4BA33331-…}` IName）的分类结果。
///
/// 支持已取证且完整消费的名称体。
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkNameEntry10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference {
        class_id: [u8; 16],
        key: u32,
    },
    /// 内联要素类名称对象。
    FeatureClassName(Box<FeatureClassNamePrefix10_1>),
    RasterCatalogName(Box<FeatureClassNamePrefix10_1>),
    /// 内联表名称对象；不代表能够打开为要素类。
    TableName(Box<crate::maps::standalone_table::TableNameBody10_1>),
    /// FileGDB 表名称包装；内联 TableName 体沿用同一读取器。
    FgdbTableName(Box<crate::maps::standalone_table::TableNameBody10_1>),
    /// 栅格波段名称。
    RasterBandName(Box<RasterDatasetNameEntryBody10_1>),
    RasterDatasetName(Box<RasterDatasetNameEntryBody10_1>),
    /// Schematic 图表类或元素类名称；内层数据源名称共享对象表。
    SchematicClassName(Box<SchematicClassNameBody10_1>),
    /// ObjectClassName 工厂实际创建的 FdbTableName；持久化体与 TableName
    /// 相同，但保留流内身份。
    ObjectClassName(Box<crate::maps::standalone_table::TableNameBody10_1>),
    /// 查询表名称；查询描述新对象未闭合时作为停止点。
    QueryTableName(Box<QueryTableNameBody10_1>),
    FeatureQueryName(Box<FeatureQueryNameBody10_1>),
    NetCdfFeatureClassName(Box<NetCdfFeatureClassNameBody10_1>),
    NetCdfTableName(Box<NetCdfTableNameBody10_1>),
    DbTableName(Box<DbTableNameBody10_1>),
    GeometricNetworkName(Box<GeometricNetworkNameBody10_1>),
    NetworkDatasetName(Box<NetworkDatasetName10_1>),
    TinName(Box<TinName10_1>),
    LasDatasetName(Box<LasDatasetName10_1>),
    RepresentationClassName(Box<RepresentationClassName10_1>),
    ShortcutName {
        version: i16,
        name_string: String,
        target: Box<BookmarkNameEntry10_1>,
    },
    FileName {
        version: i16,
        name_string: String,
    },
    AgsServerObjectName(Box<AgsServerObjectName10_1>),
    AgsServerConnectionName(Box<AgsServerConnectionName10_1>),
    CadastralFabricName(Box<CadastralFabricName10_1>),
    TerrainName(Box<TerrainName10_1>),
    TopologyName(Box<TopologyNameBody10_1>),
    FeatureDatasetName(Box<FeatureDatasetNameBody10_1>),
    CoverageName(Box<FeatureDatasetNameBody10_1>),
    WorkspaceName(Box<WorkspaceNamePrefix10_1>),
    CadDrawingName(Box<CadDrawingNameBody10_1>),
    ReferencedTableName(Box<ReferencedTableNameBody10_1>),
    XyEventSourceName(Box<XyEventSourceNameBody10_1>),
    RouteEventSourceName(Box<RouteEventSourceNameBody10_1>),
    TableQueryName(Box<TableQueryNameBody10_1>),
    RelQueryTableName(Box<RelQueryTableNameBody10_1>),
    /// 关系类名称；可完整读取持久化体，不表示原生 IName::Open 后能取得
    /// IFeatureClass。
    RelationshipClassName(Box<RelationshipClassNameBody10_1>),
    /// 内存关系类名称；边界同 RelationshipClassName。
    MemoryRelationshipClassName(Box<MemoryRelationshipClassNameBody10_1>),
    AmsDatasetName(Box<AmsDatasetName10_1>),
}

/// FeatureBookmark 对象体（Load `0x793b5c50` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureBookmark10_1 {
    /// int16 版本位模式（按有符号上限 1 读取）。
    pub version: u16,
    /// 第 2 字段原值（`this+4`，IFeatureBookmark.FeatureId）；不检查要素是否存在。
    pub raw_int32: i32,
    /// 名称对象条目（第 3 字段）。
    pub name_object: BookmarkNameEntry10_1,
    /// 书签名称（第 4 字段，String::Read → `this+0xc`）。
    pub name: String,
    /// 体之后的剩余字节数（正常输入应为 0）。
    pub remaining_bytes: usize,
}

/// Legend 第 3 字段（请求 IID `{52AA5BC1-…}` ILegendFormat）的分类结果。
///
/// C 轮接线起新对象命中 LegendFormat 类时整体消费其闭合体
/// （`map_surround_family` 模块；体内停止经
/// [`LegendStop10_1::LegendFormatBodyIncomplete`] 透传），其余实现类维持
/// 停止点语义。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendFormatEntry10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 LegendFormat 对象：已读取其闭合体（C 轮接线；IPS 虚表/9 段字段
    /// 序证据见 docs/arcmap-10.1-legend-family.md）。
    Body(Box<LegendFormatBody10_1>),
}

/// Legend 条目循环内一个对象条目（请求 IID `{52AA5BC0-…}` ILegendItem）的
/// 分类结果（接线轮起 HorizontalLegendItem 体闭合接入分派；C 轮接线起
/// HorizontalBar/Nested/Vertical 三类接入）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendItemSlot10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 HorizontalLegendItem 对象（类标识经
    /// [`HORIZONTAL_LEGEND_ITEM_CLASS_ID`] 校验）：已读取其闭合体（Renderers
    /// 包装 + LayerLib BaseLegendItem 字段序；体内停止经
    /// [`LegendStop10_1::LegendItemBodyIncomplete`] 上抛）。
    HorizontalLegendItem(Box<HorizontalLegendItemBody10_1>),
    /// 新 HorizontalBarLegendItem 对象（C 轮接线；流内布局 = BaseLegendItem
    /// 字段序 + 尾段）。
    HorizontalBarLegendItem(Box<HorizontalBarLegendItemBody10_1>),
    /// 新 NestedLegendItem 对象（C 轮接线；BaseLegendItem 字段序 + 尾段）。
    NestedLegendItem(Box<NestedLegendItemBody10_1>),
    /// 新 VerticalLegendItem 对象（C 轮接线；与 HorizontalLegendItem 逐字节
    /// 同序，区别仅在对象头类身份）。
    VerticalLegendItem(Box<VerticalLegendItemBody10_1>),
}

/// Legend 条目循环中一个已消费的对象条目（含触发停止的新对象头）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendItemEntry10_1 {
    /// 条目头起点偏移（相对传入 Reader 的底层字节）。
    pub offset: usize,
    /// 条目分类；未知 LegendItem 实现类的新对象不出现在本枚举（触发停止）。
    pub entry: LegendItemSlot10_1,
}

/// Legend 版本 ≥5 尾组的 4 个原样标量（`this−0x9c..−0x94`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegendTailGroup10_1 {
    /// FixedFrame、AutoColumns、AutoFit 的 int16 原样位模式。
    pub shorts: [u16; 3],
    /// MinFontSize（<5 时原生写默认 4.0，不读流）。
    pub value: f64,
}

/// Legend 前缀读取的停止原因（非错误：已消费内容有效，其后保留为剩余）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendStop10_1 {
    /// ILegendFormat 槽位为新对象：类体无读取器，已消费 20 字节对象头，
    /// 游标停在头之后。
    LegendFormatBody {
        /// 对象头起点偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// ILegendItem 条目为新对象且类标识非 HorizontalLegendItem：实现类体无
    /// 读取器，已消费该条目 20 字节对象头，游标停在头之后。
    LegendItemBody {
        /// 条目序号（0 起，含触发停止的条目）。
        index: usize,
        /// 条目对象头起点偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 接线轮新增：ILegendItem 条目为 HorizontalLegendItem 且其体内出现停
    /// 止（原因透传）。游标停在其内层停止点；该条目对象键已登记，条目以
    /// 部分消费状态记录。
    LegendItemBodyIncomplete {
        /// 条目序号（0 起）。
        index: usize,
        /// HorizontalLegendItem 体内的停止原因。
        reason: HorizontalLegendItemStop10_1,
    },
    /// NestedLegendItem 的装饰符号未读完。
    LegendItemSymbol {
        index: usize,
        reason: crate::objects::display_symbol::SymbolStop10_1,
    },
    /// C 轮接线新增：ILegendFormat 槽为 LegendFormat 新对象且其体内出现
    /// 停止（原因透传）。游标停在其内层停止点。
    LegendFormatBodyIncomplete {
        /// LegendFormat 体内的停止原因。
        reason: FamilyStop10_1,
    },
    /// 版本门尾段读毕（Load `0x793da250` 自然返回点）。
    Completed,
}

impl LegendStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(self) -> String {
        match self {
            Self::LegendFormatBody {
                offset,
                class_id,
                key,
            } => format!(
                "Legend 的 ILegendFormat 槽位（offset 0x{offset:x}）为新对象（类 {}，键 \
                 {key}）：该类无闭合读取器，停止并保留剩余",
                class_id_hex(&class_id)
            ),
            Self::LegendFormatBodyIncomplete { reason } => format!(
                "Legend 的 ILegendFormat 槽位（LegendFormat 体）在停止点停止：{reason:?}；\
                 游标停在内层停止点，其后字段不可达"
            ),
            Self::LegendItemBody {
                index,
                offset,
                class_id,
                key,
            } => format!(
                "Legend 条目 {index}（offset 0x{offset:x}）为新对象（类 {}，键 {key}）：\
                 ILegendItem 实现类体未闭合，停止并保留剩余",
                class_id_hex(&class_id)
            ),
            Self::LegendItemSymbol { index, reason } => {
                format!("图例项 {index} 的符号未读完：{reason:?}")
            }
            Self::LegendItemBodyIncomplete { index, reason } => format!(
                "Legend 条目 {index}（HorizontalLegendItem）体在停止点停止：{reason:?}；\
                 游标停在内层停止点，其后条目不可达"
            ),
            Self::Completed => {
                "Legend 已确认前缀读毕（版本门尾段之后为该对象体自然终点)".to_string()
            }
        }
    }
}

/// Legend 对象体已确认前缀（Load `0x793da250`，至版本门尾段止）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendPrefix10_1 {
    /// 原始版本位模式，读取时按有符号值判断。
    pub version: u16,
    /// MapSurround 族共同前缀（第 2 字段起）。
    pub common: MapSurroundCommonPrefix10_1,
    /// ILegendFormat 槽位条目（第 3 字段；到达即有值）。
    pub legend_format: LegendFormatEntry10_1,
    /// 标题候选字符串（第 4 字段，`this−0xf8`）。
    pub title: String,
    /// 版本 ==1 的两个 double 原样位模式（读后丢弃的旧版遗留段；写侧无
    /// 对应写出，仅旧文档出现）。
    pub legacy_doubles: Option<[f64; 2]>,
    /// AutoAdd、AutoReorder、AutoVisibility 的 int16 原样位模式（第 6 字段）。
    pub shorts: [u16; 3],
    /// 第 7 字段 int16 原样位模式；10.1 Load 不使用该值，Save 恒写 `0xFFFF`。
    pub selection_short: u16,
    /// 条目循环的 int32 计数原始值。非正值不读取条目。
    pub item_count: i32,
    /// 已消费的条目（按流内顺序；末位可能是触发停止的新对象头）。
    pub items: Vec<LegendItemEntry10_1>,
    /// 版本 ≥3 的流向内部标志原样位模式（`this−0xe8`）；FlowRight getter 取反。
    pub tail_short_1: Option<u16>,
    /// 版本 ≥4 的 ScaleSymbols int16 原样位模式（`this−0xe6`）。
    pub tail_short_2: Option<u16>,
    /// 版本 ≥5 的尾组（int16 ×3 + double）；<5 时原生写默认值不读流。
    pub tail_group: Option<LegendTailGroup10_1>,
    /// 停止原因。
    pub stop: LegendStop10_1,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 MapSurround 族共同前缀，失败时不推进游标。
///
/// 起点约定：调用者已确认这是某个 MapSurround 族对象体的共同前缀起点
/// （各类 Load 先调 `0x794413d0`；独立暴露供研究性读取与测试）。字段序、
/// 版本门与证据见模块文档。`max_bytes` 约束名称字符串；`table` 为跨流共享
/// 键登记表（第 3 段 IMap 槽的分类依赖它；新对象在本读取器中报不支持
/// 错误，因此本函数自身不引入新键，trial/commit 语义仍完整保留）。
pub fn read_map_surround_common_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MapSurroundCommonPrefix10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 原生用有符号 short 进行两处版本比较。
    let version = trial.read_u16_le()?;
    let signed_version = version as i16;

    // 2. 名称字符串（String::Read 布局 → this−0xa0）。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    // 3. LoadObject(IMap {E6BDAA75-4D35-11D0-98BE-00805F7CED21})：关联地图。
    //    Load 通过关联 MapSurround 接口子对象的 +0x10 虚函数设置该 IMap；
    //    Save（Map.dll 0x79441510）从其对象指针成员调用 SaveObject。两侧
    //    的 this 调整方式不同，不把伪代码参数偏移误当成相同绝对偏移。
    //    登记先于持久化使该槽在真实流中恒为回引；空标记与回引之外的新对象
    //    （整棵 Map 体，无外部长度）不可跳过，报不支持。
    let map_offset = trial.position();
    let map_entry = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key } => {
            Some(MapBackReference10_1 { class_id, key })
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "MapSurround 共同前缀关联地图槽 offset 0x{map_offset:x}: 类标识 {}（键 \
                     {key}）为新对象；该槽应为对已持久化 Map 的键回引（或空标记），新对象\
                     体为整棵 Map 树且无外部长度字段，无法读取或跳过",
                    class_id_hex(&class_id)
                ),
            ));
        }
    };

    // 4. 版本 ≥2：保留 FixedAspectRatio 的 int16 位模式（this−0x8c）。
    let extra_short = if signed_version >= 2 {
        Some(trial.read_u16_le()?)
    } else {
        None
    };

    // 5. 版本 ≥3：WKSEnvelope 原样（原生读入后丢弃，写侧取
    //    this−0x84 恒写）。
    let raw_envelope = if signed_version >= 3 {
        let bytes = trial.read_bytes(32)?;
        Some(bytes.try_into().expect("固定 32 字节"))
    } else {
        None
    };

    let result = MapSurroundCommonPrefix10_1 {
        version,
        name,
        map_entry,
        extra_short,
        raw_envelope,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 AOIBookmark（空间书签）对象体，失败时不推进
/// 游标、不修改共享登记表。
///
/// 起点约定：调用者已读取对象头并比对 [`AOI_BOOKMARK_CLASS_ID`]。字段序
/// 与证据见模块文档（Load `0x793a2cd0` / Save `0x793a2d60`）。嵌套包络为
/// 新对象且类标识是 Envelope 时用其读取器整体消费；其他新对象类报
/// [`ErrorKind::Unsupported`]（体无外部长度字段，无法跳过）。
pub fn read_aoi_bookmark_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<AoiBookmark10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("AOIBookmark 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    let version = trial.read_u16_le()?;
    if version as i16 > 1 {
        return Err(invalid(format!("不支持的版本 {version}")));
    }

    // 2. LoadObject(IEnvelope {1B077914-…})：位置包络。
    let envelope_offset = trial.position();
    let envelope = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => BookmarkEnvelopeEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = trial_table.registered_class(key);
            if registered.is_some_and(|class| class != ENVELOPE_CLASS_ID) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "AOIBookmark 位置包络槽 offset 0x{envelope_offset:x}: 回引键 {key} 的登记类不受支持"
                    ),
                ));
            }
            BookmarkEnvelopeEntry10_1::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == ENVELOPE_CLASS_ID => {
            // Envelope 全部版本分支已闭合：整体消费对象体（含嵌套空间参考）。
            let body = read_envelope_10_1(&mut trial, max_bytes, &mut trial_table)?;
            BookmarkEnvelopeEntry10_1::Envelope { key, body }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "AOIBookmark 位置包络槽 offset 0x{envelope_offset:x}: 类标识 {}（键 \
                     {key}）不是 Envelope；本读取器仅支持 Envelope 新对象嵌套，且对象体\
                     无外部长度字段，无法跳过未知类继续读取",
                    class_id_hex(&class_id)
                ),
            ));
        }
    };

    // 3. 书签名称（String::Read → this+8）。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    let result = AoiBookmark10_1 {
        version,
        envelope,
        name,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后读取 FeatureBookmark；错误时游标及登记表不变。
///
/// 调用者已读取对象头并比对 [`FEATURE_BOOKMARK_CLASS_ID`]。名称槽支持空值、
/// 回引及 [`BookmarkNameEntry10_1`] 中可完整读取的名称体；未知类或体内停止返回
/// [`ErrorKind::Unsupported`]，避免错读随后的书签名称。
pub fn read_feature_bookmark_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<FeatureBookmark10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FeatureBookmark 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 原生按有符号 short 比较，保留流内版本位模式。
    let version = trial.read_u16_le()?;
    if version as i16 > 1 {
        return Err(invalid(format!("不支持的版本 {version}")));
    }

    // 2. IFeatureBookmark.FeatureId（this+4）。
    let raw_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    // 3. LoadObject(IName {4BA33331-…})。
    let name_object = read_bookmark_name_slot_10_1(&mut trial, max_bytes, &mut trial_table, 0)?;

    // 4. 书签名称（String::Read → this+0xc）。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    let result = FeatureBookmark10_1 {
        version,
        raw_int32,
        name_object,
        name,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

// 调用者负责将名称槽及其宿主的读取作为同一事务提交。
fn read_bookmark_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    shortcut_depth: usize,
) -> io::Result<BookmarkNameEntry10_1> {
    let name_offset = reader.position();
    Ok(match read_object_entry_10_1(reader, table)? {
        ObjectEntry10_1::Null => BookmarkNameEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => BookmarkNameEntry10_1::BackReference {
            class_id: table.registered_class(key).unwrap_or(class_id),
            key,
        },
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (FEATURE_CLASS_NAME_CLASS_ID
                | FGDB_FEATURE_CLASS_NAME_CLASS_ID
                | COVERAGE_FEATURE_CLASS_NAME_CLASS_ID
                | RASTER_CATALOG_NAME_CLASS_ID),
            key,
        } => {
            let body = match class_id {
                FGDB_FEATURE_CLASS_NAME_CLASS_ID => {
                    read_fgdb_feature_class_name_body_10_1(reader, max_bytes, table, key)?
                }
                COVERAGE_FEATURE_CLASS_NAME_CLASS_ID => {
                    read_coverage_feature_class_name_body_10_1(reader, max_bytes, table, key)?
                }
                _ => read_feature_class_name_body_10_1(reader, max_bytes, table, key)?,
            };
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: FeatureClassName 未读完（{stop:?}）"
                    ),
                ));
            }
            if class_id == RASTER_CATALOG_NAME_CLASS_ID {
                BookmarkNameEntry10_1::RasterCatalogName(Box::new(body))
            } else {
                BookmarkNameEntry10_1::FeatureClassName(Box::new(body))
            }
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (TABLE_NAME_CLASS_ID | FGDB_TABLE_NAME_CLASS_ID | OBJECT_CLASS_NAME_CLASS_ID),
            ..
        } => {
            let (body, stopped) = if class_id == FGDB_TABLE_NAME_CLASS_ID {
                read_fgdb_table_name_body_10_1(reader, table, max_bytes)?
            } else {
                read_table_name_body_10_1(reader, table, max_bytes)?
            };
            if stopped {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: {} 未读完",
                        match class_id {
                            OBJECT_CLASS_NAME_CLASS_ID => "ObjectClassName",
                            FGDB_TABLE_NAME_CLASS_ID => "FgdbTableName",
                            _ => "TableName",
                        }
                    ),
                ));
            }
            match class_id {
                OBJECT_CLASS_NAME_CLASS_ID => {
                    BookmarkNameEntry10_1::ObjectClassName(Box::new(body))
                }
                FGDB_TABLE_NAME_CLASS_ID => BookmarkNameEntry10_1::FgdbTableName(Box::new(body)),
                _ => BookmarkNameEntry10_1::TableName(Box::new(body)),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: QUERY_TABLE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_query_table_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: QueryTableName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::QueryTableName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: REFERENCED_TABLE_NAME_CLASS_ID,
            key,
        } => {
            let mut stop = None;
            let body =
                read_referenced_table_name_body_10_1(reader, max_bytes, table, key, &mut stop)?;
            if let Some(stop) = stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: ReferencedTableName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::ReferencedTableName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: CAD_DRAWING_NAME_CLASS_ID,
            key,
        } => {
            let body = read_cad_drawing_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: CadDrawingName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::CadDrawingName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: TOPOLOGY_NAME_CLASS_ID,
            key,
        } => {
            let body = read_topology_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: TopologyName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::TopologyName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: WORKSPACE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_workspace_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: WorkspaceName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::WorkspaceName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (FEATURE_DATASET_NAME_CLASS_ID | COVERAGE_NAME_CLASS_ID),
            key,
        } => {
            let body = if class_id == COVERAGE_NAME_CLASS_ID {
                read_coverage_name_body_10_1(reader, max_bytes, table, key)?
            } else {
                read_feature_dataset_name_body_10_1(reader, max_bytes, table, key)?
            };
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: FeatureDatasetName 未读完（{stop:?}）"
                    ),
                ));
            }
            if class_id == COVERAGE_NAME_CLASS_ID {
                BookmarkNameEntry10_1::CoverageName(Box::new(body))
            } else {
                BookmarkNameEntry10_1::FeatureDatasetName(Box::new(body))
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: GEOMETRIC_NETWORK_NAME_CLASS_ID,
            key,
        } => {
            let body = read_geometric_network_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: GeometricNetworkName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::GeometricNetworkName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: DB_TABLE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_db_table_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: DBTableName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::DbTableName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: NETCDF_TABLE_NAME_CLASS_ID,
            ..
        } => {
            let body = read_netcdf_table_name_body_10_1(reader, max_bytes, table)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: NetCDFTableName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::NetCdfTableName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: NETCDF_FEATURE_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body = read_netcdf_feature_class_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: NetCDFFeatureClassName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::NetCdfFeatureClassName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: XY_EVENT_SOURCE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_xy_event_source_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: XYEventSourceName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::XyEventSourceName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: ROUTE_EVENT_SOURCE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_route_event_source_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: RouteEventSourceName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::RouteEventSourceName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: FEATURE_QUERY_NAME_CLASS_ID,
            key,
        } => {
            let body = read_feature_query_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.table.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: FeatureQueryName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::FeatureQueryName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: TABLE_QUERY_NAME_CLASS_ID,
            key,
        } => {
            let body = read_table_query_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: TableQueryName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::TableQueryName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: REL_QUERY_TABLE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_rel_query_table_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: RelQueryTableName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::RelQueryTableName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body = read_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: RelationshipClassName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::RelationshipClassName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body =
                read_memory_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = &body.stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: MemoryRelationshipClassName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::MemoryRelationshipClassName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: TERRAIN_NAME_CLASS_ID,
            ..
        } => BookmarkNameEntry10_1::TerrainName(Box::new(read_terrain_name_body_10_1(
            reader, max_bytes, table,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id: SHORTCUT_NAME_CLASS_ID,
            ..
        } => {
            if shortcut_depth == MAX_SHORTCUT_NAME_NESTING {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "ShortcutName 嵌套层数超限",
                ));
            }
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("ShortcutName 版本 {version} 超出 10.1 上限"),
                ));
            }
            let name_string =
                crate::stream::native_string::read_bstr_string_10_1(reader, max_bytes)?;
            let target =
                read_bookmark_name_slot_10_1(reader, max_bytes, table, shortcut_depth + 1)?;
            BookmarkNameEntry10_1::ShortcutName {
                version,
                name_string,
                target: Box::new(target),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: FILE_NAME_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("FileName 版本 {version} 超出 10.1 上限"),
                ));
            }
            BookmarkNameEntry10_1::FileName {
                version,
                name_string: crate::stream::native_string::read_bstr_string_10_1(
                    reader, max_bytes,
                )?,
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: AGS_SERVER_OBJECT_NAME,
            ..
        } => BookmarkNameEntry10_1::AgsServerObjectName(Box::new(
            read_ags_server_object_name_body(reader, max_bytes, table)?,
        )),
        ObjectEntry10_1::NewObject {
            class_id: AGS_SERVER_CONNECTION_NAME,
            ..
        } => BookmarkNameEntry10_1::AgsServerConnectionName(Box::new(
            read_ags_server_connection_name_body(reader, max_bytes, table)?,
        )),
        ObjectEntry10_1::NewObject {
            class_id: CADASTRAL_FABRIC_NAME_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("CadastralFabricName 版本 {version} 超出 10.1 上限"),
                ));
            }
            let strings = [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ];
            let mut stop = None;
            let dataset = read_dataset_name_slot_10_1(reader, max_bytes, table, &mut stop)?;
            if let Some(stop) = stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("CadastralFabricName 数据集名称槽未读完（{stop:?}）"),
                ));
            }
            let workspace =
                read_surface_workspace_name_10_1(reader, max_bytes, table, "CadastralFabricName")?;
            BookmarkNameEntry10_1::CadastralFabricName(Box::new(CadastralFabricName10_1 {
                version,
                strings,
                dataset,
                workspace,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (LAS_DATASET_NAME_CLASS_ID | REPRESENTATION_CLASS_NAME_CLASS_ID),
            ..
        } => {
            let class_name = if class_id == LAS_DATASET_NAME_CLASS_ID {
                "LasDatasetName"
            } else {
                "RepresentationClassName"
            };
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("{class_name} 版本 {version} 超出 10.1 上限"),
                ));
            }
            let strings = [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ];
            let workspace = read_surface_workspace_name_10_1(reader, max_bytes, table, class_name)?;
            if class_id == LAS_DATASET_NAME_CLASS_ID {
                BookmarkNameEntry10_1::LasDatasetName(Box::new(LasDatasetName10_1 {
                    version,
                    strings,
                    workspace,
                }))
            } else {
                BookmarkNameEntry10_1::RepresentationClassName(Box::new(
                    RepresentationClassName10_1 {
                        version,
                        strings,
                        workspace,
                    },
                ))
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: TIN_NAME_CLASS_ID,
            ..
        } => BookmarkNameEntry10_1::TinName(Box::new(read_tin_name_body_10_1(
            reader, max_bytes, table,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id: NETWORK_DATASET_NAME_CLASS_ID,
            ..
        } => BookmarkNameEntry10_1::NetworkDatasetName(Box::new(read_network_dataset_name(
            reader, max_bytes, table,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id: AMS_DATASET_NAME_CLASS_ID,
            key: _,
        } => BookmarkNameEntry10_1::AmsDatasetName(Box::new(read_ams_dataset_name_10_1(
            reader, max_bytes, table,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (RASTER_BAND_NAME_CLASS_ID
                | RASTER_BAND_COLLECTION_NAME_CLASS_ID
                | RASTER_DATASET_NAME_CLASS_ID
                | RASTER_DATASET_NAME83_CLASS_ID
                | NETCDF_RASTER_DATASET_NAME_CLASS_ID
                | CACHE_RASTER_DATASET_NAME_CLASS_ID
                | MOSAIC_DATASET_NAME_CLASS_ID
                | RASTER_TYPE_NAME_CLASS_ID
                | FGDB_RASTER_DATASET_NAME_CLASS_ID
                | FUNCTION_RASTER_DATASET_NAME_CLASS_ID
                | IS_RASTER_DATASET_NAME_CLASS_ID
                | IMAGE_SERVER_NAME_CLASS_ID
                | ACCESS_RASTER_DATASET_NAME_CLASS_ID
                | SDE_RASTER_DATASET_NAME_CLASS_ID
                | INTERNET_TILED_LAYER_NAME_CLASS_ID
                | AIS_IMAGE_SERVICE_NAME_CLASS_ID
                | WCS_COVERAGE_NAME_CLASS_ID
                | WCS_CONNECTION_NAME_CLASS_ID
                | WMS_CONNECTION_NAME_CLASS_ID
                | WMTS_CONNECTION_NAME_CLASS_ID),
            key,
        } => {
            let body =
                read_supported_raster_name_body_10_1(reader, max_bytes, table, class_id, key)?;
            if class_id == RASTER_BAND_NAME_CLASS_ID {
                BookmarkNameEntry10_1::RasterBandName(Box::new(body))
            } else {
                BookmarkNameEntry10_1::RasterDatasetName(Box::new(body))
            }
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID
                | SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID),
            key,
        } => {
            let (body, stopped) =
                read_schematic_class_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = stopped {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: {} 未读完（{stop:?}）",
                        if class_id == SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID {
                            "SchematicDiagramClassName"
                        } else {
                            "SchematicElementClassName"
                        }
                    ),
                ));
            }
            BookmarkNameEntry10_1::SchematicClassName(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: 类标识 {}（键 {key}）为不支持的新对象",
                    class_id_hex(&class_id)
                ),
            ));
        }
    })
}

/// 从对象头之后的位置读取 Legend 对象体的已确认前缀，失败时不推进游标、
/// 不修改共享登记表；停止点为成功返回（已消费部分 + 剩余字节 + 原因）。
///
/// 起点约定：调用者已读取对象头并比对 [`LEGEND_CLASS_ID`]。字段序与停止
/// 规则见模块文档（Load `0x793da250`）：ILegendFormat 槽与 ILegendItem 条目
/// 的新对象体无读取器，消费 20 字节对象头后停止；版本门尾段读毕为
/// [`LegendStop10_1::Completed`]。`max_bytes` 约束两个字符串与条目计数
/// 预检（计数 × 16 最小条目字节数）。
pub fn read_legend_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LegendPrefix10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Legend 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    let version = trial.read_u16_le()? as i16;
    if version > 5 {
        return Err(invalid(format!("不支持的版本 {version}")));
    }

    // 2. MapSurround 族共同前缀（含 IMap 槽新对象的Unsupported 转发）。
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, &mut trial_table)?;

    // 3. LoadObject(ILegendFormat {52AA5BC1-…})：空标记/回引消费后继续。
    //    C 轮接线：新对象命中 LegendFormat 类时经 map_surround_family 读取
    //    器整体消费其闭合体（体内停止透传）；其余实现类维持既有停止语义
    //    （消费 20 字节对象头后停止）。
    let format_offset = trial.position();
    let legend_format;
    let mut stop = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => {
            legend_format = LegendFormatEntry10_1::Null;
            None
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = trial_table.registered_class(key);
            if registered.is_some_and(|class| class != FAMILY_LEGEND_FORMAT_CLASS_ID) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Legend 格式槽 offset 0x{format_offset:x}: 回引键 {key} 的登记类不受支持"
                    ),
                ));
            }
            legend_format = LegendFormatEntry10_1::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            };
            None
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == FAMILY_LEGEND_FORMAT_CLASS_ID =>
        {
            let body = read_legend_format_body_10_1(&mut trial, max_bytes, &mut trial_table, key)?;
            let stop_reason = body
                .stop
                .clone()
                .map(|reason| LegendStop10_1::LegendFormatBodyIncomplete { reason });
            legend_format = LegendFormatEntry10_1::Body(Box::new(body));
            stop_reason
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            legend_format = LegendFormatEntry10_1::Null;
            Some(LegendStop10_1::LegendFormatBody {
                offset: format_offset,
                class_id,
                key,
            })
        }
    };

    let mut title = String::new();
    let mut legacy_doubles = None;
    let mut shorts = [0_u16; 3];
    let mut selection_short = 0;
    let mut item_count = 0;
    let mut items = Vec::new();
    let mut tail_short_1 = None;
    let mut tail_short_2 = None;
    let mut tail_group = None;

    if stop.is_none() {
        // 4. 标题候选字符串（this−0xf8）。
        title = read_string_10_1(&mut trial, max_bytes)?;

        // 5. 版本 ==1：double ×2（读后丢弃的旧版遗留段，原样保留）。
        if version == 1 {
            let mut pair = [0_f64; 2];
            for slot in &mut pair {
                *slot = trial.read_f64_le()?;
            }
            legacy_doubles = Some(pair);
        }

        // 6/7. int16 ×3 + 保留位模式的 int16（原生读后不用，写侧恒写 0xFFFF）。
        for slot in &mut shorts {
            *slot = trial.read_u16_le()?;
        }
        selection_short = trial.read_u16_le()?;
    }

    if stop.is_none() {
        // 8. int32 条目计数 + 循环 LoadObject(ILegendItem {52AA5BC0-…})。
        let count_offset = trial.position();
        let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        item_count = count;
        // 原生在计数非正时直接进入版本化尾段。
        let count_usize = usize::try_from(count.max(0)).expect("计数已确认为非负");
        let min_total = count_usize
            .checked_mul(MIN_ENTRY_BYTES)
            .ok_or_else(|| invalid("条目计数 × 最小条目字节数溢出".to_string()))?;
        if min_total > max_bytes {
            return Err(invalid(format!(
                "条目计数 offset 0x{count_offset:x}: 声明计数 {count} × 最小条目字节数 \
                 {MIN_ENTRY_BYTES} 超过资源限额 {max_bytes}，在条目读取之前拒绝"
            )));
        }
        for index in 0..count_usize {
            let offset = trial.position();
            let raw = read_object_entry_10_1(&mut trial, &mut trial_table)?;
            let entry = match raw {
                ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    let registered = trial_table.registered_class(key);
                    if registered.is_some_and(|class| {
                        !matches!(
                            class,
                            HORIZONTAL_LEGEND_ITEM_CLASS_ID
                                | HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID
                                | NESTED_LEGEND_ITEM_CLASS_ID
                                | VERTICAL_LEGEND_ITEM_CLASS_ID
                        )
                    }) {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            format!(
                                "Legend 条目槽 offset 0x{offset:x}: 回引键 {key} 的登记类不受支持"
                            ),
                        ));
                    }
                    LegendItemSlot10_1::BackReference {
                        class_id: registered.unwrap_or(class_id),
                        key,
                    }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == HORIZONTAL_LEGEND_ITEM_CLASS_ID =>
                {
                    // 接线：HorizontalLegendItem 体闭合（Renderers + LayerLib
                    // BaseLegendItem 字段序），整体消费；体内停止透传。
                    let body = read_horizontal_legend_item_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut trial_table,
                        key,
                    )?;
                    if let Some(reason) = body.stop.clone() {
                        stop = Some(LegendStop10_1::LegendItemBodyIncomplete { index, reason });
                    }
                    LegendItemSlot10_1::HorizontalLegendItem(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID =>
                {
                    // C 轮接线：HorizontalBarLegendItem 体闭合（BaseLegendItem
                    // 字段序 + int32/double ×2 尾段），整体消费；体内停止透传。
                    let body = read_horizontal_bar_legend_item_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut trial_table,
                        key,
                    )?;
                    if let Some(reason) = body.base.stop.clone() {
                        stop = Some(LegendStop10_1::LegendItemBodyIncomplete { index, reason });
                    }
                    LegendItemSlot10_1::HorizontalBarLegendItem(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == NESTED_LEGEND_ITEM_CLASS_ID =>
                {
                    // C 轮接线：NestedLegendItem 体闭合（BaseLegendItem 字段序
                    // + 尾段），整体消费；体内停止透传。
                    let body = read_nested_legend_item_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut trial_table,
                        key,
                    )?;
                    if let Some(reason) = body.base.stop.clone() {
                        stop = Some(LegendStop10_1::LegendItemBodyIncomplete { index, reason });
                    } else if let Some(reason) = body.symbol_stop {
                        stop = Some(LegendStop10_1::LegendItemSymbol { index, reason });
                    }
                    LegendItemSlot10_1::NestedLegendItem(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == VERTICAL_LEGEND_ITEM_CLASS_ID =>
                {
                    // C 轮接线：VerticalLegendItem 体闭合（与 HorizontalLegendItem
                    // 逐字节同序），整体消费；体内停止透传。
                    let body = read_vertical_legend_item_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut trial_table,
                        key,
                    )?;
                    if let Some(reason) = body.base.stop.clone() {
                        stop = Some(LegendStop10_1::LegendItemBodyIncomplete { index, reason });
                    }
                    LegendItemSlot10_1::VerticalLegendItem(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    // 其余 LegendItem 实现类体无读取器：已消费 20 字节对象头，
                    // 游标停在头之后（键已按「登记先于持久化」登记）。
                    stop = Some(LegendStop10_1::LegendItemBody {
                        index,
                        offset,
                        class_id,
                        key,
                    });
                    LegendItemSlot10_1::Null
                }
            };
            items.push(LegendItemEntry10_1 { offset, entry });
            if stop.is_some() {
                break;
            }
        }
    }

    if stop.is_none() {
        // 9. 版本 ≥3：int16（this−0xe8）。
        if version >= 3 {
            tail_short_1 = Some(trial.read_u16_le()?);
        }
        // 10. 版本 ≥4：int16（this−0xe6）。
        if version >= 4 {
            tail_short_2 = Some(trial.read_u16_le()?);
        }
        // 11. 版本 ≥5：int16 ×3 + double（<5 时原生写默认 4.0，不读流）。
        if version >= 5 {
            let mut group = LegendTailGroup10_1 {
                shorts: [0; 3],
                value: 0.0,
            };
            for slot in &mut group.shorts {
                *slot = trial.read_u16_le()?;
            }
            group.value = trial.read_f64_le()?;
            tail_group = Some(group);
        }
    }

    let stop = stop.unwrap_or(LegendStop10_1::Completed);
    let result = LegendPrefix10_1 {
        version: version as u16,
        common,
        legend_format,
        title,
        legacy_doubles,
        shorts,
        selection_short,
        item_count,
        items,
        tail_short_1,
        tail_short_2,
        tail_group,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}
