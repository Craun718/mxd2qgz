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
use crate::layers::feature_layer_tail::{UID_CLASS_ID, UidBody10_1, read_uid_body_10_1};
use crate::layers::fgdb_raster_dataset_name::FGDB_RASTER_DATASET_NAME_CLASS_ID;
use crate::layers::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
use crate::layers::image_server_rules::{
    RasterFunctionArgumentsBody10_1, RuleSlot10_1, read_raster_type_slot_10_1,
};
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
use crate::objects::dataset_name_more::{
    SCHEMATIC_DATASET_NAME_CLASS_ID, SCHEMATIC_FOLDER_NAME_CLASS_ID, SchematicDatasetNameBody10_1,
    SchematicFolderNameBody10_1, read_schematic_dataset_name_body_10_1,
    read_schematic_folder_name_body_10_1,
};
use crate::objects::dataset_name_more::{
    SCHEMATIC_DIAGRAM_NAME_CLASS_ID, SchematicDiagramNameBody10_1,
    read_schematic_diagram_name_body_10_1,
};
use crate::objects::domain::{DomainSlot10_1, read_domain_slot_nested_10_1};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_entry_10_1,
};
use crate::objects::fields::{
    FIELD_CLASS_ID, FIELDS_CLASS_ID, FieldBody10_1, FieldsBody10_1, read_field_body_10_1,
    read_fields_body_nested_10_1,
};
use crate::objects::geometry::{POINT_CLASS_ID, PointBody10_1, read_point_body_10_1};
use crate::objects::network_source_directions::{
    DirectionSlot10_1, NetworkAttributeEntryBody10_1, NetworkDirections10_1, NetworkSource10_1,
    TrafficData10_1, read_network_attribute_slot_10_1, read_network_directions_slot_10_1,
    read_network_source_slot_10_1, read_traffic_data_slot_10_1,
};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, PropertyValueObjectStream10_1,
    read_padded_variant_object_stream_nested_10_1, read_property_set_object_stream_10_1,
    read_variant_object_body_10_1, read_variant_object_stream_value_10_1,
};
use crate::objects::route_event_name::{
    ROUTE_EVENT_SOURCE_NAME_CLASS_ID, ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID,
    RouteEventSourceNameBody10_1, RouteMeasureLocatorName10_1,
    read_route_event_source_name_body_10_1, read_route_measure_locator_name_body_10_1,
};
use crate::objects::string_array::{StringArraySlot10_1, read_string_array_slot_10_1};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_bstr_string_10_1;
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use crate::stream::variant_stream::{
    VariantObjectContext10_1, VariantValue10_1, read_value_with_objects_10_1,
};
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

const MAX_BOOKMARK_NAME_NESTING: usize = 16;

pub const GP_GDB_ANNO_DOMAIN_CLASS_ID: [u8; 16] = [
    0x32, 0x98, 0xdb, 0x07, 0x18, 0xc8, 0xd3, 0x41, 0x86, 0xd7, 0x13, 0x8f, 0x84, 0xb0, 0xa3, 0x3f,
];

pub const GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID: [u8; 16] = [
    0x60, 0xb2, 0xe7, 0x3f, 0x52, 0xce, 0xd5, 0x11, 0x93, 0x3a, 0x00, 0x80, 0xc7, 0x1a, 0x32, 0x26,
];

pub const GP_COVERAGE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x90, 0x11, 0x78, 0x56, 0x9b, 0xd4, 0xd5, 0x11, 0x93, 0x41, 0x00, 0x80, 0xc7, 0x1a, 0x32, 0x26,
];

pub const GP_COV_ANNO_DOMAIN_CLASS_ID: [u8; 16] = [
    0xcb, 0xdb, 0xc0, 0xb5, 0x53, 0x99, 0x01, 0x4e, 0xa7, 0x53, 0x88, 0xc7, 0xec, 0x07, 0xaf, 0x4f,
];

pub const GP_CAD_ANNO_DOMAIN_CLASS_ID: [u8; 16] = [
    0xd6, 0x0c, 0x8f, 0xc7, 0xc2, 0xa4, 0xae, 0x4e, 0xa1, 0xa9, 0xbc, 0x80, 0x5d, 0x13, 0x5c, 0x62,
];

pub const GP_CODED_VALUE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x71, 0x08, 0x1e, 0xa1, 0x07, 0x66, 0xd4, 0x4b, 0x87, 0xd0, 0x35, 0xe0, 0x18, 0x9a, 0xee, 0x59,
];

pub const GP_FIELD_INFO_DOMAIN_CLASS_ID: [u8; 16] = [
    0x4f, 0x28, 0x22, 0x2b, 0x4c, 0xa2, 0xbf, 0x4f, 0x81, 0x3b, 0x09, 0xb0, 0x62, 0x7c, 0xdc, 0x19,
];

pub const GP_STRING_CLASS_ID: [u8; 16] = [
    0x33, 0x85, 0x34, 0x6a, 0x67, 0xb6, 0x05, 0x4a, 0xab, 0xb0, 0x00, 0x24, 0x11, 0x0d, 0x17, 0xf0,
];

pub const GP_ENCRYPTED_STRING_CLASS_ID: [u8; 16] = [
    0xdb, 0xd4, 0x6c, 0xb7, 0x3b, 0x84, 0x23, 0x47, 0xb1, 0x74, 0xa0, 0x83, 0x30, 0x95, 0xf8, 0x66,
];

pub const GP_DOUBLE_CLASS_ID: [u8; 16] = [
    0x74, 0x45, 0x4c, 0x37, 0x6e, 0x5d, 0x77, 0x4d, 0x9a, 0x67, 0x54, 0x89, 0xc6, 0xbb, 0xe9, 0x32,
];

pub const XML_INDEX_CLASS_ID: [u8; 16] = [
    0x9b, 0x55, 0xec, 0x18, 0xbe, 0x07, 0x98, 0x4f, 0x89, 0x60, 0x20, 0x66, 0x36, 0xba, 0x25, 0x9a,
];
pub const XML_INDEX_TAGS_CLASS_ID: [u8; 16] = [
    0x70, 0xf1, 0xd0, 0x78, 0x98, 0x0f, 0xcf, 0x41, 0x87, 0x68, 0xd7, 0xc8, 0x88, 0x7c, 0x19, 0x35,
];
pub const XML_INDEX_TAG_CLASS_ID: [u8; 16] = [
    0x91, 0xc1, 0x3e, 0xc8, 0x34, 0xd2, 0x53, 0x43, 0x95, 0x22, 0x72, 0x23, 0x29, 0xd2, 0xbc, 0xbf,
];
pub const FME_DATASET_CLASS_ID: [u8; 16] = [
    0xb8, 0xd8, 0x3a, 0xd6, 0x46, 0xab, 0x03, 0x49, 0xb1, 0x58, 0x26, 0xc3, 0x24, 0x0d, 0xe0, 0xd9,
];
pub const GP_BOOLEAN_CLASS_ID: [u8; 16] = [
    0x59, 0xd5, 0xe9, 0x66, 0x0e, 0x5e, 0xb4, 0x45, 0xad, 0xa8, 0xd7, 0xc2, 0x07, 0xba, 0x6b, 0x49,
];

pub const GP_DATE_CLASS_ID: [u8; 16] = [
    0x2b, 0xf2, 0xd6, 0xbe, 0x8e, 0xad, 0x0e, 0x44, 0x8d, 0xd5, 0xb4, 0x16, 0x39, 0x6b, 0x6f, 0xd4,
];

pub const GP_LINEAR_UNIT_CLASS_ID: [u8; 16] = [
    0xc7, 0xe5, 0x2d, 0x84, 0xfb, 0x80, 0x41, 0x4c, 0x86, 0x21, 0x69, 0xde, 0xad, 0x3c, 0xc0, 0x24,
];

pub const GP_AREAL_UNIT_CLASS_ID: [u8; 16] = [
    0x96, 0x8a, 0xc2, 0x62, 0x4e, 0xb8, 0xf8, 0x49, 0xac, 0xf5, 0x0b, 0x1b, 0x5a, 0x87, 0xef, 0x08,
];

pub const GP_RANDOM_NUMBER_GENERATOR_CLASS_ID: [u8; 16] = [
    0x4b, 0x67, 0xc6, 0xa5, 0xa6, 0xbc, 0xc8, 0x46, 0x82, 0x05, 0xb5, 0xb9, 0x58, 0xee, 0xae, 0xb9,
];

pub const GP_MULTI_VALUE_CLASS_ID: [u8; 16] = [
    0x94, 0xbc, 0x34, 0x6a, 0xb2, 0x06, 0x1b, 0x48, 0xb1, 0x04, 0x66, 0x63, 0xc4, 0xa3, 0x44, 0xef,
];

pub const GP_SA_TOPO_FEATURES_CLASS_ID: [u8; 16] = [
    0x5f, 0x76, 0x71, 0x83, 0x0b, 0x0f, 0x68, 0x48, 0xb7, 0x86, 0x01, 0x5b, 0x2e, 0x2c, 0x1c, 0x10,
];

pub const GP_VALUE_TABLE_CLASS_ID: [u8; 16] = [
    0xcd, 0xa0, 0xa7, 0x98, 0x82, 0x32, 0xcd, 0x44, 0x90, 0xb7, 0x4f, 0x39, 0xcd, 0xb8, 0x5b, 0x84,
];

pub const GP_VARIANT_CLASS_ID: [u8; 16] = [
    0x5e, 0xff, 0x82, 0x5b, 0x15, 0x31, 0xe4, 0x40, 0xba, 0xc8, 0xe6, 0x21, 0x05, 0xca, 0x28, 0x11,
];

pub const GP_LONG_CLASS_ID: [u8; 16] = [
    0x47, 0x7f, 0xed, 0x30, 0xd9, 0x12, 0x9b, 0x4c, 0xbe, 0x8b, 0x7a, 0x0a, 0xe3, 0x86, 0xe5, 0x74,
];

/// GPPoint 的 CLSID，按对象流中的字节序排列。
const GP_REPLICA_DATASETS_CLASS_ID: [u8; 16] = [
    0x62, 0xec, 0x68, 0xfc, 0xa0, 0xcd, 0x97, 0x44, 0x99, 0x77, 0xb4, 0x58, 0x20, 0x26, 0x8a, 0x43,
];
pub const GP_REPLICA_CLASS_ID: [u8; 16] = [
    0xfc, 0x4d, 0x52, 0xe5, 0xe0, 0x66, 0xb0, 0x49, 0x88, 0xe7, 0x09, 0x40, 0x40, 0xc9, 0x8f, 0x74,
];
pub const GP_REPLICA_DESCRIPTION_CLASS_ID: [u8; 16] = [
    0x9a, 0x83, 0x63, 0xf1, 0xb8, 0x58, 0xb9, 0x45, 0x93, 0xad, 0x88, 0x80, 0x3e, 0x0f, 0xa5, 0x0f,
];
pub const GP_REPLICA_DATASET_CLASS_ID: [u8; 16] = [
    0x3c, 0x7e, 0x47, 0x72, 0xce, 0x85, 0xde, 0x49, 0xaa, 0x3e, 0x5d, 0x55, 0x90, 0x9c, 0x22, 0xea,
];
const GP_FIELD_MAPPING_CLASS_ID: [u8; 16] = [
    0xb9, 0x2d, 0x27, 0xd9, 0x3c, 0xf0, 0x84, 0x44, 0x8f, 0xd2, 0x8a, 0x50, 0x47, 0xb2, 0xcb, 0xb6,
];
const GP_FIELD_MAP_CLASS_ID: [u8; 16] = [
    0xa7, 0x43, 0xcf, 0xde, 0x8a, 0x57, 0xca, 0x48, 0xbc, 0xa2, 0x4b, 0x11, 0x14, 0x8e, 0x20, 0x09,
];
const GP_RASTER_BUILDER_CLASS_ID: [u8; 16] = [
    0x0c, 0x62, 0xcc, 0x2f, 0x8c, 0x1c, 0x96, 0x4a, 0xa0, 0xf9, 0x46, 0x5b, 0x4a, 0xa3, 0xea, 0x7a,
];
const GP_RASTER_FORMULATED_CLASS_ID: [u8; 16] = [
    0x93, 0x8e, 0xa5, 0xff, 0x9a, 0x05, 0x5f, 0x43, 0xaf, 0x2b, 0x5a, 0x34, 0x8a, 0x88, 0x2c, 0x29,
];
const GP_RASTER_DATA_LAYER_CLASS_ID: [u8; 16] = [
    0x83, 0x7e, 0x91, 0xff, 0xf9, 0x04, 0x28, 0x43, 0xa3, 0x36, 0x2c, 0xfd, 0x3c, 0x12, 0xd8, 0x89,
];
const FEATURE_LAYER_DRAWING_CLASS_ID: [u8; 16] = [
    0xba, 0xc2, 0x66, 0xaa, 0xbf, 0x55, 0xaa, 0x4e, 0x80, 0xaf, 0x64, 0x8b, 0x80, 0x22, 0x81, 0x03,
];
const LABELING_CLASS_ID: [u8; 16] = [
    0xd0, 0x43, 0x51, 0x9d, 0x37, 0x97, 0xeb, 0x40, 0x9c, 0x9b, 0xac, 0x13, 0x48, 0x31, 0xc6, 0xe0,
];
const LABEL_CLASSES_CLASS_ID: [u8; 16] = [
    0x97, 0x2a, 0x55, 0xf0, 0xc8, 0xf4, 0x65, 0x47, 0xab, 0x22, 0x2f, 0xa4, 0xbf, 0xad, 0x4b, 0x3e,
];
const LABEL_CLASS_CLASS_ID: [u8; 16] = [
    0xa1, 0x31, 0x26, 0x60, 0x11, 0xc1, 0xa1, 0x4a, 0x80, 0x63, 0x76, 0x99, 0xc0, 0x03, 0x79, 0x2c,
];
const POINT_PLACEMENT_CLASS_ID: [u8; 16] = [
    0x37, 0x8b, 0x45, 0x6a, 0x5d, 0x05, 0xda, 0x48, 0xb3, 0xa5, 0x85, 0x6d, 0x9d, 0xc9, 0x52, 0x5f,
];
const LINE_PLACEMENT_CLASS_ID: [u8; 16] = [
    0x00, 0x92, 0x72, 0x17, 0x9f, 0xd1, 0x7f, 0x43, 0x86, 0x07, 0x4a, 0x79, 0x95, 0xbc, 0x09, 0x28,
];
const POLYGON_PLACEMENT_CLASS_ID: [u8; 16] = [
    0xb2, 0x9b, 0x5d, 0x42, 0x95, 0xbd, 0xb9, 0x4d, 0xb9, 0xf2, 0x2c, 0x7b, 0x4d, 0x44, 0xec, 0x58,
];
const RASTER_LAYER_DRAWING_DESCRIPTION_CLASS_ID: [u8; 16] = [
    0x04, 0x6a, 0x6f, 0x42, 0x8c, 0x90, 0x49, 0x47, 0x9a, 0xd8, 0xaf, 0xf1, 0x72, 0x21, 0xc5, 0x81,
];
const GP_RASTER_DATA_CLASS_ID: [u8; 16] = [
    0xbd, 0x0f, 0xd4, 0xa4, 0xd8, 0xcf, 0x8d, 0x4d, 0x94, 0xd1, 0x37, 0x26, 0xc6, 0xff, 0x79, 0x2f,
];
pub const GP_LINE_CLASS_ID: [u8; 16] = [
    0xe4, 0x80, 0x5f, 0x43, 0xb2, 0x6d, 0x6b, 0x47, 0xb9, 0x77, 0xb0, 0x6b, 0xff, 0x52, 0xf9, 0x78,
];
pub const GP_POLYGON_CLASS_ID: [u8; 16] = [
    0x0b, 0x8f, 0x1b, 0xec, 0x38, 0xdf, 0x44, 0x44, 0xbe, 0x03, 0x01, 0x16, 0xe0, 0x95, 0x37, 0xdf,
];
pub const GP_POINT_CLASS_ID: [u8; 16] = [
    0x82, 0x30, 0xe9, 0xa1, 0xbe, 0xba, 0x44, 0x4d, 0x8c, 0xe3, 0x46, 0x37, 0x74, 0x8e, 0xdd, 0xdd,
];

pub const GP_WORKSPACE_EXTENSION_CLASS_ID: [u8; 16] = [
    0xbe, 0x3d, 0x23, 0x8b, 0x91, 0x05, 0x9b, 0x45, 0xbf, 0xd1, 0x46, 0x8d, 0x2d, 0xb5, 0x26, 0x7d,
];

pub const GP_DATA_FILE_CLASS_ID: [u8; 16] = [
    0xe4, 0x35, 0x30, 0x76, 0xc9, 0xe5, 0xaf, 0x4a, 0xa6, 0x0c, 0x65, 0xc3, 0x8a, 0xdd, 0xb5, 0xef,
];

pub const GDS_DATA_CLASS_ID: [u8; 16] = [
    0x9c, 0xe6, 0x77, 0x8b, 0x95, 0x2a, 0xe3, 0x42, 0x88, 0xe3, 0x22, 0xa6, 0x12, 0xb6, 0x97, 0x52,
];

pub const GP_3DA_DECIMATE_CLASS_ID: [u8; 16] = [
    0xb1, 0x79, 0x83, 0x1c, 0x8a, 0x15, 0xc2, 0x49, 0x97, 0x9c, 0xe9, 0x48, 0xe0, 0x7b, 0xa3, 0x03,
];
pub const GP_3DA_INTERPOLATE_CLASS_ID: [u8; 16] = [
    0xfe, 0x0e, 0x85, 0x41, 0xde, 0x51, 0xa4, 0x43, 0x81, 0x47, 0xb8, 0x8c, 0xca, 0x19, 0xcf, 0x3c,
];
pub const GP_NA_HIERARCHY_SETTINGS_CLASS_ID: [u8; 16] = [
    0x22, 0x7d, 0xf2, 0x07, 0x6c, 0xa5, 0x9b, 0x41, 0x99, 0x31, 0xdd, 0x38, 0xc6, 0x08, 0x97, 0xf1,
];
pub const NA_CLASS_FIELD_MAP_CLASS_ID: [u8; 16] = [
    0xf8, 0x9f, 0x1c, 0xe6, 0xbf, 0x06, 0xe5, 0x42, 0xad, 0x66, 0x84, 0x93, 0x64, 0xb4, 0x85, 0x49,
];

pub const INDEXES_CLASS_ID: [u8; 16] = [
    0x13, 0x98, 0x85, 0x03, 0xa5, 0x4d, 0xd1, 0x11, 0x88, 0x24, 0x00, 0x00, 0xf8, 0x77, 0x76, 0x2d,
];
pub const INDEX_CLASS_ID: [u8; 16] = [
    0x01, 0x27, 0x6e, 0x82, 0xa6, 0x4d, 0xd1, 0x11, 0x88, 0x24, 0x00, 0x00, 0xf8, 0x77, 0x76, 0x2d,
];

pub const GP_TOPOLOGY_MEMBERSHIP_CLASS_ID: [u8; 16] = [
    0xf6, 0x8f, 0xa7, 0x5d, 0x6b, 0x54, 0xf0, 0x4a, 0x84, 0xb8, 0x50, 0xb6, 0x84, 0x84, 0xd9, 0xd5,
];
pub const GP_GEOMETRIC_NETWORK_MEMBERSHIP_CLASS_ID: [u8; 16] = [
    0xd6, 0x91, 0xe8, 0x7e, 0xcd, 0x6a, 0x87, 0x46, 0x9b, 0xf4, 0xfc, 0xe4, 0x9f, 0xc4, 0x71, 0x57,
];

pub const GP_SUBTYPE_CLASS_ID: [u8; 16] = [
    0x02, 0x53, 0x1b, 0x74, 0x17, 0x29, 0xab, 0x4a, 0x95, 0x2a, 0xdf, 0x5f, 0x0e, 0x01, 0x9e, 0xb6,
];

pub const TOPOLOGY_RULE_CLASS_ID: [u8; 16] = [
    0x89, 0x38, 0xd4, 0x8a, 0xa4, 0xf2, 0xb2, 0x43, 0x83, 0xdc, 0x6d, 0x1d, 0x38, 0xdd, 0xbc, 0xc8,
];

pub const EDGE_CONNECTIVITY_RULE_CLASS_ID: [u8; 16] = [
    0xe0, 0xd9, 0x90, 0xf2, 0xe5, 0x58, 0xd2, 0x11, 0xab, 0x26, 0, 0, 0, 0, 0, 0,
];
pub const JUNCTION_CONNECTIVITY_RULE_CLASS_ID: [u8; 16] = [
    0xe1, 0xd9, 0x90, 0xf2, 0xe5, 0x58, 0xd2, 0x11, 0xab, 0x26, 0, 0, 0, 0, 0, 0,
];

pub const NET_WEIGHT_CLASS_ID: [u8; 16] = [
    0x31, 0xe0, 0xbb, 0x3e, 0x7d, 0x55, 0xd1, 0x11, 0xb2, 0x54, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];
pub const NET_WEIGHT_ASSOCIATION_CLASS_ID: [u8; 16] = [
    0x91, 0xc4, 0xbd, 0x19, 0x01, 0x62, 0xd1, 0x11, 0xb2, 0x55, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];

pub const RELATIONSHIP_RULE_CLASS_ID: [u8; 16] = [
    0x1e, 0x6c, 0x4c, 0xf8, 0xff, 0x47, 0xd2, 0x11, 0x99, 0x33, 0, 0, 0xf8, 0x03, 0x72, 0xb4,
];

pub const GP_RELATIONSHIP_CLASS_KEY_CLASS_ID: [u8; 16] = [
    0x1f, 0x4e, 0xca, 0x5d, 0xa4, 0x81, 0xc3, 0x41, 0xb1, 0x60, 0x81, 0x8d, 0x22, 0xfc, 0x50, 0xb0,
];

pub const DE_RELATIONSHIP_CLASS_INFO_CLASS_ID: [u8; 16] = [
    0xb6, 0xf0, 0x28, 0x31, 0x7e, 0x51, 0xa3, 0x4a, 0xa2, 0x22, 0x20, 0x44, 0x1f, 0x2c, 0xb3, 0x02,
];

pub const DE_RELATIONSHIP_CLASS_CLASS_ID: [u8; 16] = [
    0x87, 0x54, 0x1f, 0xd4, 0x16, 0x6b, 0xfd, 0x42, 0xab, 0x61, 0x63, 0xa5, 0x5b, 0x51, 0x9b, 0x71,
];

pub const DE_GEOMETRIC_NETWORK_CLASS_ID: [u8; 16] = [
    0x8c, 0xba, 0x68, 0xfe, 0x5d, 0x93, 0x6f, 0x4f, 0x80, 0xd5, 0x79, 0x57, 0xf8, 0xdb, 0xef, 0xdf,
];

pub const DE_TOPOLOGY_CLASS_ID: [u8; 16] = [
    0x7d, 0x63, 0xc4, 0xac, 0xe0, 0xd5, 0xd5, 0x47, 0xab, 0xcd, 0x9f, 0x25, 0xab, 0x32, 0x6e, 0x6f,
];

const DE_SCHEMATIC_DIAGRAM_CLASS_ID: [u8; 16] = [
    0x3f, 0xaf, 0x58, 0x01, 0x10, 0x76, 0x26, 0x42, 0x9f, 0x88, 0xb3, 0x4b, 0x62, 0xa2, 0xe9, 0x7b,
];
const DE_SCHEMATIC_DATASET_CLASS_ID: [u8; 16] = [
    0x3f, 0x8c, 0xd3, 0x06, 0xe0, 0x47, 0x98, 0x4a, 0x9c, 0xb0, 0xa4, 0x40, 0x43, 0xb4, 0xe5, 0xff,
];
const DE_SCHEMATIC_FOLDER_CLASS_ID: [u8; 16] = [
    0xfd, 0x8e, 0x2b, 0x16, 0xc8, 0x1f, 0x0c, 0x4d, 0x98, 0xc7, 0x19, 0x67, 0x89, 0x7f, 0x19, 0x37,
];
pub const DE_CATALOG_ROOT_CLASS_ID: [u8; 16] = [
    0xdd, 0xeb, 0xd6, 0x01, 0x9a, 0xde, 0x17, 0x42, 0x8b, 0xfe, 0x32, 0x72, 0x65, 0x85, 0x5f, 0xa2,
];

pub const DE_REMOTE_DATABASE_FOLDER_CLASS_ID: [u8; 16] = [
    0xdf, 0x6f, 0x3e, 0xdb, 0xf8, 0xd5, 0x94, 0x4c, 0xb7, 0x43, 0x47, 0xab, 0x4d, 0xe7, 0x9a, 0x05,
];

pub const DE_SPATIAL_REFERENCES_FOLDER_CLASS_ID: [u8; 16] = [
    0xfc, 0xf8, 0x89, 0x31, 0xc8, 0xa9, 0x8e, 0x4b, 0x8c, 0x47, 0x43, 0xc7, 0x58, 0xa7, 0xa8, 0xbd,
];

pub const DE_LAYER_CLASS_ID: [u8; 16] = [
    0x08, 0x17, 0xd7, 0xbd, 0x63, 0x34, 0x87, 0x47, 0xa4, 0x91, 0xb9, 0x36, 0x3b, 0x93, 0x7d, 0xbe,
];

const GP_LAYER_CLASS_ID: [u8; 16] = [
    0x1e, 0x59, 0x47, 0x6a, 0x16, 0xaa, 0x3e, 0x44, 0xb5, 0x0d, 0xd8, 0x92, 0x66, 0x59, 0xa4, 0xe1,
];

const GP_FEATURE_RECORD_SET_LAYER_CLASS_ID: [u8; 16] = [
    0x5f, 0x11, 0xe6, 0x1c, 0x72, 0xa4, 0xa1, 0x49, 0xa2, 0x28, 0x88, 0xf6, 0x01, 0xb3, 0x13, 0x4e,
];
const GP_RECORD_SET_CLASS_ID: [u8; 16] = [
    0xc2, 0x67, 0xdb, 0x38, 0xc1, 0xeb, 0x1a, 0x46, 0x87, 0x29, 0x01, 0x1b, 0xb7, 0xdb, 0xaf, 0x0c,
];
const RECORD_SET_CLASS_ID: [u8; 16] = [
    0xc7, 0xe8, 0x17, 0xb5, 0xc0, 0x6e, 0xe0, 0x46, 0x80, 0xbb, 0xd6, 0x1e, 0xd6, 0x91, 0xc5, 0xce,
];
const GP_TABLE_VIEW_CLASS_ID: [u8; 16] = [
    0xed, 0x16, 0x08, 0xcf, 0xa0, 0xa6, 0xc1, 0x4a, 0xb8, 0xf1, 0x19, 0x2b, 0x32, 0x91, 0xa8, 0x53,
];
const GP_RASTER_LAYER_CLASS_ID: [u8; 16] = [
    0x79, 0x22, 0x31, 0x89, 0xe6, 0xb7, 0x87, 0x47, 0xbe, 0x93, 0x9b, 0x25, 0xcd, 0x94, 0xd7, 0x68,
];
const GP_LAS_DATASET_LAYER_CLASS_ID: [u8; 16] = [
    0xee, 0x45, 0x39, 0xf8, 0x37, 0xcb, 0xc7, 0x4a, 0x98, 0xec, 0xf2, 0x54, 0x46, 0xa9, 0x0a, 0x5d,
];
const GP_TOPOLOGY_LAYER_CLASS_ID: [u8; 16] = [
    0x48, 0x71, 0x7d, 0xc1, 0xe8, 0x66, 0x8d, 0x49, 0x85, 0x5b, 0x6c, 0x5d, 0x35, 0x2e, 0xf6, 0x9a,
];
const GP_NETWORK_DATASET_LAYER_CLASS_ID: [u8; 16] = [
    0xdc, 0x73, 0xc8, 0xc9, 0x4e, 0xba, 0xdb, 0x47, 0xae, 0x78, 0x75, 0xad, 0xb2, 0x8f, 0xa9, 0x9e,
];
const GP_TERRAIN_LAYER_CLASS_ID: [u8; 16] = [
    0x2a, 0xcf, 0xef, 0xad, 0x95, 0x96, 0x22, 0x44, 0x82, 0xfc, 0x90, 0x6f, 0x44, 0xfc, 0x49, 0xf6,
];
const GP_TIN_LAYER_CLASS_ID: [u8; 16] = [
    0xb6, 0x3f, 0xf7, 0x1f, 0x74, 0x1d, 0x22, 0x47, 0xa7, 0x85, 0xdf, 0xa1, 0xf8, 0x32, 0x7a, 0xb5,
];
const GP_FEATURE_LAYER_CLASS_ID: [u8; 16] = [
    0xf3, 0x6d, 0x3d, 0x0e, 0x43, 0x8d, 0x0e, 0x4a, 0x82, 0x90, 0x8a, 0xb8, 0x62, 0x97, 0xab, 0x17,
];
const GP_RASTER_CATALOG_LAYER_CLASS_ID: [u8; 16] = [
    0xf1, 0x0f, 0x8f, 0x60, 0xec, 0xdf, 0x81, 0x48, 0xa6, 0x51, 0x2b, 0xc5, 0xce, 0x5a, 0xcd, 0x73,
];
const GP_NA_LAYER_CLASS_ID: [u8; 16] = [
    0x90, 0xde, 0x42, 0xf1, 0x41, 0xb9, 0x92, 0x44, 0x85, 0x11, 0x44, 0x12, 0xb5, 0x1c, 0x2f, 0x43,
];
const GP_GROUP_LAYER_CLASS_ID: [u8; 16] = [
    0x68, 0x72, 0x95, 0x10, 0x3e, 0x80, 0xb4, 0x4c, 0xbd, 0x9c, 0xc2, 0xb6, 0x60, 0xb9, 0x8b, 0xdf,
];
const GP_CADASTRAL_FABRIC_LAYER_CLASS_ID: [u8; 16] = [
    0xfb, 0x8d, 0x5d, 0x6f, 0x4b, 0xa0, 0x41, 0x47, 0x88, 0x23, 0x90, 0xef, 0x1c, 0x56, 0xfd, 0xf1,
];
const GP_MOSAIC_LAYER_CLASS_ID: [u8; 16] = [
    0xea, 0xb6, 0x2d, 0xbf, 0x17, 0x45, 0xcb, 0x44, 0xa7, 0xc4, 0xad, 0xf9, 0xf6, 0x3d, 0x4e, 0x35,
];
const GP_COMPOSITE_LAYER_CLASS_ID: [u8; 16] = [
    0x51, 0x4b, 0x99, 0x22, 0xba, 0xc6, 0xff, 0x41, 0xab, 0xe6, 0x54, 0x96, 0x15, 0x44, 0x1e, 0x3f,
];

pub const DE_COVERAGE_FEATURE_CLASS_ID: [u8; 16] = [
    0x65, 0x38, 0xec, 0xb7, 0x5b, 0x80, 0xa4, 0x41, 0x80, 0x4c, 0x8b, 0x4a, 0x30, 0xaa, 0x41, 0x28,
];

pub const DE_SHAPE_FILE_CLASS_ID: [u8; 16] = [
    0x47, 0xfb, 0xe6, 0x03, 0x38, 0xaa, 0x6c, 0x47, 0x93, 0x4b, 0x1d, 0xf5, 0x76, 0x5c, 0x76, 0xeb,
];

pub const DE_VPF_COVERAGE_CLASS_ID: [u8; 16] = [
    0x0d, 0xf8, 0x4f, 0xca, 0xa9, 0x85, 0x5c, 0x44, 0xad, 0xb2, 0xa5, 0xc7, 0x81, 0x1b, 0x3c, 0x36,
];

pub const DE_LAS_DATASET_CLASS_ID: [u8; 16] = [
    0x09, 0x67, 0x00, 0xea, 0x03, 0xcd, 0x9d, 0x43, 0x80, 0xa1, 0x92, 0x54, 0xe2, 0x54, 0x12, 0xe3,
];

pub const DE_COVERAGE_CLASS_ID: [u8; 16] = [
    0x47, 0x0c, 0xb7, 0x96, 0x7f, 0x2c, 0x5f, 0x40, 0xb9, 0xfd, 0xde, 0xaa, 0x3c, 0xdc, 0x08, 0x4c,
];

const CADASTRAL_FIELD_EDITS_CLASS_ID: [u8; 16] = [
    0xfb, 0x5d, 0xe3, 0xde, 0xa2, 0x81, 0x8e, 0x46, 0xab, 0xa8, 0x24, 0x14, 0xe7, 0x57, 0xa0, 0xd9,
];

const CLASS_ATTRIBUTES_CLASS_ID: [u8; 16] = [
    0x01, 0x64, 0x38, 0x02, 0x23, 0x02, 0xa6, 0x43, 0x9a, 0xb2, 0xbe, 0x7f, 0x92, 0xd6, 0xf8, 0x1a,
];

const REPRESENTATION_RULES_CLASS_ID: [u8; 16] = [
    0x56, 0x53, 0x48, 0xfa, 0xf1, 0x8f, 0x15, 0x45, 0xad, 0xbd, 0x16, 0x55, 0xbe, 0xb5, 0x00, 0xc1,
];

const DE_REPRESENTATION_CLASS_ID: [u8; 16] = [
    0xa6, 0x29, 0xa8, 0xab, 0x55, 0x9c, 0xee, 0x41, 0x8c, 0x15, 0x2d, 0xaa, 0x38, 0x21, 0xd5, 0x65,
];

const DE_TOOLBOX_CLASS_ID: [u8; 16] = [
    0x23, 0x6b, 0xc6, 0xfe, 0x2a, 0xe6, 0x6f, 0x46, 0xb1, 0x1b, 0xa8, 0xa8, 0xf2, 0xc2, 0x50, 0x4c,
];

const DE_TOOL_CLASS_ID: [u8; 16] = [
    0x9a, 0x02, 0xd1, 0x8c, 0x56, 0x0b, 0x6b, 0x43, 0xb4, 0xa6, 0x8f, 0x93, 0x60, 0xcd, 0x63, 0xd9,
];

const DE_CADASTRAL_FABRIC_CLASS_ID: [u8; 16] = [
    0xc0, 0x68, 0xf8, 0xb0, 0xbc, 0xbf, 0x69, 0x46, 0xb2, 0x63, 0xcd, 0x34, 0x8d, 0x65, 0xd7, 0xae,
];

pub const DE_NETWORK_DATASET_CLASS_ID: [u8; 16] = [
    0xb7, 0x08, 0x88, 0xff, 0x0c, 0xe5, 0xfe, 0x45, 0x84, 0xea, 0x1c, 0x06, 0x1d, 0x74, 0x31, 0xf6,
];
pub const DE_TERRAIN_CLASS_ID: [u8; 16] = [
    0xb6, 0x43, 0x65, 0x54, 0x7a, 0x32, 0x3c, 0x4d, 0xac, 0x15, 0x81, 0x22, 0xc8, 0xb6, 0x68, 0x0a,
];

pub const DE_TIN_CLASS_ID: [u8; 16] = [
    0xf1, 0x32, 0xdc, 0x5e, 0xb2, 0xc1, 0x08, 0x48, 0x88, 0xe0, 0x2e, 0xd6, 0x8e, 0x77, 0x26, 0x9d,
];

pub const DE_CAD_DRAWING_DATASET_CLASS_ID: [u8; 16] = [
    0x2d, 0x33, 0x86, 0xf7, 0xb2, 0x20, 0x07, 0x46, 0xbf, 0x34, 0x0a, 0x53, 0xe4, 0xd0, 0xb4, 0x13,
];

pub const DE_RASTER_CATALOG_INFO_CLASS_ID: [u8; 16] = [
    0xe3, 0x5c, 0x44, 0x6b, 0x63, 0xeb, 0x96, 0x42, 0x9a, 0x11, 0xe6, 0xe1, 0xd1, 0xc0, 0x52, 0x00,
];

pub const DE_FEATURE_CLASS_INFO_CLASS_ID: [u8; 16] = [
    0x17, 0xfc, 0x35, 0xa7, 0x0d, 0xd5, 0x63, 0x4d, 0xaa, 0x98, 0x45, 0xe7, 0xc5, 0x98, 0xce, 0xc3,
];

pub const DE_TABLE_INFO_CLASS_ID: [u8; 16] = [
    0x34, 0xc4, 0x44, 0x3d, 0x7c, 0xb6, 0xd0, 0x4f, 0xa7, 0xc9, 0xa2, 0x06, 0x85, 0xab, 0x67, 0x95,
];

pub const DE_ARC_INFO_TABLE_CLASS_ID: [u8; 16] = [
    0xec, 0x86, 0xc7, 0x1c, 0xfe, 0x5e, 0x3c, 0x49, 0x8a, 0x25, 0xb5, 0x46, 0x84, 0x46, 0x65, 0xd7,
];

pub const DE_DBASE_TABLE_CLASS_ID: [u8; 16] = [
    0x76, 0x41, 0xe8, 0x69, 0x98, 0xfb, 0x36, 0x40, 0xa8, 0xff, 0xe6, 0x9c, 0x8e, 0x82, 0xf0, 0x13,
];

pub const DE_VPF_TABLE_CLASS_ID: [u8; 16] = [
    0x32, 0xb2, 0xaa, 0x48, 0x1f, 0x6f, 0x61, 0x42, 0x9d, 0xac, 0xcb, 0xa5, 0x5d, 0x18, 0x36, 0x49,
];

pub const DE_TEXT_FILE_CLASS_ID: [u8; 16] = [
    0x66, 0x07, 0x8d, 0xe9, 0x3e, 0xc1, 0xca, 0x45, 0xba, 0x78, 0x1d, 0xef, 0x8b, 0x60, 0xca, 0xb2,
];

pub const DE_PRJ_FILE_CLASS_ID: [u8; 16] = [
    0xb0, 0x4b, 0x70, 0x7e, 0xbe, 0xcf, 0xd5, 0x11, 0x93, 0x3d, 0x00, 0x80, 0xc7, 0x1a, 0x32, 0x26,
];

pub const DE_FILE_CLASS_ID: [u8; 16] = [
    0xc5, 0xf2, 0x5c, 0x17, 0xbf, 0x12, 0x3a, 0x41, 0x8f, 0xdf, 0x3d, 0xa3, 0x0c, 0xc0, 0x9b, 0xee,
];
pub const DE_FOLDER_CLASS_ID: [u8; 16] = [
    0xbc, 0xce, 0x3c, 0x27, 0xa2, 0x1a, 0x29, 0x44, 0xbb, 0xd5, 0x4c, 0xb7, 0xc6, 0xe2, 0x04, 0x65,
];
pub const DE_MAP_DOCUMENT_CLASS_ID: [u8; 16] = [
    0xd9, 0xd5, 0xbe, 0xc0, 0xe2, 0x13, 0x9b, 0x46, 0xa7, 0xd8, 0x22, 0x70, 0x4b, 0x0c, 0x86, 0x30,
];
pub const DE_DISK_CONNECTION_CLASS_ID: [u8; 16] = [
    0x5a, 0x85, 0x5e, 0x8e, 0x60, 0x7d, 0xe1, 0x4c, 0xaf, 0xd0, 0x45, 0x92, 0x9c, 0x9f, 0x49, 0x2c,
];

pub const DE_WORKSPACE_CLASS_ID: [u8; 16] = [
    0x36, 0xaf, 0x27, 0x21, 0xe1, 0xca, 0x7e, 0x42, 0xb1, 0xa8, 0xbf, 0x0f, 0x1a, 0xd1, 0x9f, 0x6e,
];

pub const DE_GEO_DATA_SERVER_CLASS_ID: [u8; 16] = [
    0xe4, 0x41, 0xe9, 0xd6, 0xf4, 0x9e, 0x8d, 0x43, 0xae, 0x31, 0x2f, 0xb0, 0x82, 0xa2, 0x81, 0x4d,
];
pub const DE_MAP_SERVER_CLASS_ID: [u8; 16] = [
    0x87, 0x43, 0xd7, 0x69, 0x84, 0x76, 0x8e, 0x45, 0x84, 0xa3, 0xbd, 0x9e, 0x01, 0x36, 0x69, 0xc0,
];
pub const DE_GLOBE_SERVER_CLASS_ID: [u8; 16] = [
    0x53, 0x8e, 0xdf, 0x9a, 0xfc, 0xb3, 0x48, 0x43, 0xb0, 0xc1, 0x27, 0xbf, 0x1a, 0x5f, 0x5f, 0x39,
];
const DE_IMAGE_SERVER_CLASS_ID: [u8; 16] = [
    0x5f, 0x81, 0xa8, 0x2d, 0x55, 0x1c, 0x91, 0x47, 0xba, 0x69, 0x78, 0x56, 0xaf, 0x60, 0xf9, 0x95,
];
const DE_WCS_COVERAGE_CLASS_ID: [u8; 16] = [
    0x3e, 0x88, 0xe1, 0x54, 0x44, 0x3f, 0xc7, 0x4f, 0xb5, 0x88, 0xa2, 0x99, 0xb6, 0x8c, 0xa9, 0xd4,
];
const DE_WMS_MAP_CLASS_ID: [u8; 16] = [
    0x4f, 0xb2, 0x57, 0x09, 0x9d, 0x20, 0x72, 0x43, 0xb2, 0x9b, 0x61, 0xa7, 0x08, 0x24, 0x9c, 0xa7,
];
pub const DE_GP_SERVER_CLASS_ID: [u8; 16] = [
    0xda, 0x7a, 0x17, 0xa8, 0x0d, 0x39, 0x0c, 0x42, 0xa4, 0x33, 0xee, 0x67, 0xdd, 0x2e, 0x86, 0x38,
];

pub const DE_SERVER_CONNECTION_CLASS_ID: [u8; 16] = [
    0x40, 0x49, 0x7d, 0x88, 0x43, 0x93, 0xa3, 0x42, 0x8c, 0x8f, 0x5c, 0x41, 0xd4, 0xdc, 0xfa, 0xfd,
];

pub const RASTER_STORAGE_DEF_CLASS_ID: [u8; 16] = [
    0x02, 0xc6, 0xab, 0x82, 0xf9, 0x67, 0x42, 0x40, 0xbe, 0xe1, 0x64, 0x5b, 0x01, 0x39, 0xfd, 0x70,
];

pub const DE_RASTER_BAND_CLASS_ID: [u8; 16] = [
    0x33, 0x3b, 0xc6, 0x37, 0xd6, 0xc7, 0x11, 0x43, 0x91, 0xd5, 0x9e, 0x98, 0xa2, 0xa0, 0xe3, 0x9a,
];

const DE_MOSAIC_DATASET_CLASS_ID: [u8; 16] = [
    0x2d, 0x86, 0xb0, 0xf5, 0x81, 0x3e, 0xf7, 0x45, 0x9c, 0x79, 0x52, 0x23, 0x67, 0x43, 0xc9, 0x5d,
];
pub const DE_RASTER_DATASET_CLASS_ID: [u8; 16] = [
    0x68, 0xbb, 0x95, 0x82, 0x00, 0xf2, 0xa6, 0x4c, 0x80, 0x3f, 0x97, 0x99, 0x52, 0xd7, 0x32, 0xbd,
];

pub const DE_RASTER_CATALOG_CLASS_ID: [u8; 16] = [
    0x92, 0x71, 0x20, 0x80, 0xe8, 0x7c, 0x06, 0x4d, 0x95, 0x42, 0x76, 0x3e, 0x78, 0x51, 0x8f, 0xfb,
];

pub const DE_CATALOG_DATASET_CLASS_ID: [u8; 16] = [
    0xcd, 0x9f, 0x54, 0x74, 0xea, 0x9f, 0x0f, 0x48, 0xa9, 0x74, 0xae, 0x00, 0x36, 0x84, 0x56, 0x4d,
];

pub const DE_FEATURE_DATASET_CLASS_ID: [u8; 16] = [
    0xfe, 0x19, 0xc6, 0x6a, 0x1f, 0x19, 0x2d, 0x43, 0xb1, 0xa3, 0x28, 0xce, 0x85, 0xa5, 0x91, 0xaf,
];

pub const DE_FEATURE_CLASS_CLASS_ID: [u8; 16] = [
    0x40, 0x7f, 0xbb, 0x00, 0x51, 0x3e, 0x64, 0x48, 0xa6, 0x6c, 0x7f, 0x0c, 0x8e, 0xe9, 0xa8, 0x42,
];

pub const DE_TABLE_CLASS_ID: [u8; 16] = [
    0xdb, 0x05, 0x84, 0x9c, 0x87, 0xbb, 0xd6, 0x4b, 0x93, 0x2d, 0x7f, 0xfb, 0x35, 0xb0, 0x4b, 0xa0,
];

pub const GP_ADDRESS_LOCATOR_STYLE_CLASS_ID: [u8; 16] = [
    0x85, 0x06, 0xb1, 0xc0, 0xf8, 0x71, 0xd2, 0x4c, 0xb3, 0x3a, 0x8c, 0x91, 0x90, 0xb8, 0xe8, 0x07,
];

pub const GP_FIELD_INFO_EX_CLASS_ID: [u8; 16] = [
    0xfb, 0xf3, 0xc6, 0x4b, 0x66, 0x46, 0x5d, 0x42, 0xbb, 0xb5, 0xf0, 0x9b, 0xe7, 0x7a, 0x4e, 0x5e,
];

pub const GP_FIELD_INFO_CLASS_ID: [u8; 16] = [
    0xd7, 0x5c, 0xce, 0x2e, 0x67, 0x85, 0xc7, 0x44, 0xa6, 0xa3, 0xc8, 0x6f, 0xaf, 0x2b, 0x55, 0x50,
];

pub const GP_ARC_INFO_ITEM_CLASS_ID: [u8; 16] = [
    0xb2, 0xbb, 0xdd, 0x6d, 0xdc, 0x41, 0x12, 0x4f, 0x91, 0x3e, 0x80, 0x31, 0x96, 0xeb, 0x33, 0xfa,
];

pub const GP_ANALYSIS_CELL_SIZE_CLASS_ID: [u8; 16] = [
    0xf8, 0xdf, 0x42, 0xd0, 0xa2, 0xea, 0x06, 0x47, 0xb3, 0xde, 0x6e, 0x62, 0xcf, 0xed, 0xa7, 0x6a,
];

pub const GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID: [u8; 16] = [
    0x97, 0xa8, 0xaf, 0x21, 0xae, 0x84, 0x1f, 0x41, 0x9b, 0xf9, 0x14, 0x04, 0x48, 0xe3, 0xb0, 0x70,
];

pub const GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID: [u8; 16] = [
    0xca, 0x67, 0xfb, 0x23, 0xad, 0x72, 0x7b, 0x4a, 0xb4, 0xac, 0x37, 0x05, 0xc1, 0xc5, 0xd6, 0xcf,
];

pub const GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID: [u8; 16] = [
    0x26, 0x16, 0xf8, 0x94, 0x2e, 0x00, 0x51, 0x4c, 0x88, 0x29, 0xa0, 0x4c, 0x6c, 0xb5, 0xe8, 0xea,
];

pub const GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID: [u8; 16] = [
    0xf3, 0xff, 0x32, 0xce, 0x5f, 0xf7, 0x15, 0x49, 0xb9, 0xe0, 0x2a, 0x6d, 0x42, 0xe5, 0xfb, 0x20,
];

pub const GP_RASTER_CALCULATOR_EXPRESSION_CLASS_ID: [u8; 16] = [
    0xa1, 0xd0, 0x45, 0xf7, 0xa2, 0x22, 0xee, 0x4a, 0x89, 0x3e, 0x01, 0xea, 0xb8, 0x60, 0x3b, 0xb8,
];

pub const GP_SA_EXTRACT_VALUES_CLASS_ID: [u8; 16] = [
    0x87, 0xd3, 0x79, 0xc0, 0x73, 0x74, 0xd7, 0x41, 0xba, 0x10, 0xac, 0x9a, 0x92, 0xf8, 0x1a, 0x0c,
];

pub const GP_SA_FUZZY_FUNCTION_CLASS_ID: [u8; 16] = [
    0x96, 0xfc, 0x1d, 0xf3, 0xe0, 0x01, 0x01, 0x4f, 0x96, 0x2b, 0x06, 0x4d, 0x48, 0xd5, 0xc2, 0x46,
];

pub const GP_SA_HORIZONTAL_FACTOR_CLASS_ID: [u8; 16] = [
    0x23, 0x70, 0xd2, 0x74, 0xe7, 0x49, 0xee, 0x4c, 0xb7, 0x69, 0x07, 0x46, 0x15, 0xe8, 0x09, 0xbd,
];

pub const GP_SA_VERTICAL_FACTOR_CLASS_ID: [u8; 16] = [
    0x10, 0x7d, 0x12, 0xa2, 0x8d, 0x81, 0x0b, 0x44, 0xbd, 0x0e, 0xfa, 0x76, 0xcf, 0x75, 0xc3, 0x5b,
];

pub const PATH_DISTANCE_HORIZONTAL_FACTOR_CLASS_ID: [u8; 16] = [
    0xbc, 0xfb, 0xda, 0x9f, 0x91, 0xa4, 0xd7, 0x43, 0x9d, 0x41, 0xd2, 0x88, 0xbb, 0xa5, 0x68, 0xe1,
];

pub const PATH_DISTANCE_VERTICAL_FACTOR_CLASS_ID: [u8; 16] = [
    0xcc, 0xfe, 0x4e, 0x1d, 0x5c, 0xb8, 0xb9, 0x4e, 0xb0, 0x9b, 0xce, 0xf8, 0xc2, 0x25, 0xb5, 0xa3,
];

pub const GP_SA_NEIGHBORHOOD_CLASS_ID: [u8; 16] = [
    0xf7, 0x35, 0xa4, 0x6f, 0x46, 0x90, 0x2b, 0x48, 0x86, 0xcc, 0x08, 0x9e, 0x16, 0x82, 0xea, 0x3f,
];

pub const GP_SA_REMAP_CLASS_ID: [u8; 16] = [
    0xa9, 0xd0, 0x02, 0xf4, 0x32, 0xf0, 0x6a, 0x47, 0xbc, 0xbb, 0xd4, 0x8c, 0xf0, 0x41, 0xc2, 0x4e,
];

pub const GP_SA_NUMBER_REMAP_CLASS_ID: [u8; 16] = [
    0xff, 0xc3, 0x56, 0x31, 0xce, 0x80, 0x8f, 0x49, 0xb1, 0x6f, 0xf3, 0xa8, 0x1a, 0x45, 0x90, 0x8f,
];

pub const GP_SA_SEMI_VARIOGRAM_CLASS_ID: [u8; 16] = [
    0x77, 0x23, 0x73, 0x7a, 0xb8, 0xb6, 0xab, 0x41, 0xad, 0x2d, 0xc2, 0x05, 0x1e, 0x82, 0xa4, 0x94,
];

pub const GP_SA_TIME_CONFIGURATION_CLASS_ID: [u8; 16] = [
    0xd0, 0x3f, 0x73, 0x24, 0xde, 0x2c, 0x46, 0x44, 0xb6, 0x5e, 0x8f, 0x23, 0x9d, 0x77, 0xde, 0x7e,
];

pub const TIME_CONFIGURATION_CLASS_ID: [u8; 16] = [
    0x80, 0x19, 0xa3, 0x0a, 0x44, 0x3d, 0x9a, 0x43, 0xa8, 0xbf, 0xc7, 0xd4, 0xc9, 0xd8, 0xf3, 0x0b,
];

pub const GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID: [u8; 16] = [
    0x94, 0x7c, 0x01, 0x96, 0x71, 0xf4, 0xa7, 0x42, 0x86, 0xef, 0x63, 0x3b, 0x40, 0x65, 0x0f, 0x8c,
];

pub const GP_SA_WEIGHTED_SUM_CLASS_ID: [u8; 16] = [
    0x99, 0xb6, 0x1d, 0x1f, 0x5c, 0xd0, 0x50, 0x4c, 0xb9, 0x51, 0xb7, 0x96, 0x9e, 0x03, 0x97, 0xe3,
];

pub const GP_SA_STRING_REMAP_CLASS_ID: [u8; 16] = [
    0x89, 0x20, 0x53, 0xd0, 0x4d, 0x34, 0x23, 0x4d, 0xa6, 0x7c, 0x5d, 0xfc, 0xd1, 0x4d, 0x66, 0x94,
];

pub const GP_SA_RADIUS_CLASS_ID: [u8; 16] = [
    0x7f, 0xc0, 0x3f, 0x2b, 0x3b, 0x10, 0xc4, 0x41, 0xaa, 0x42, 0x34, 0xa2, 0x43, 0xfd, 0x7b, 0x87,
];

pub const GP_EVALUATION_SCALE_CLASS_ID: [u8; 16] = [
    0xb5, 0x71, 0x16, 0xc8, 0x3c, 0xb2, 0xae, 0x4a, 0xbf, 0xa8, 0xb8, 0xa6, 0x6f, 0xc1, 0x73, 0xb7,
];

pub const GP_SA_MAP_ALGEBRA_EXP_CLASS_ID: [u8; 16] = [
    0x44, 0xbe, 0xea, 0xac, 0x4c, 0x32, 0x5b, 0x45, 0xb6, 0x35, 0xc8, 0x76, 0x4f, 0xdb, 0x00, 0x29,
];

pub const GP_SA_CELL_SIZE_CLASS_ID: [u8; 16] = [
    0x04, 0xf4, 0x82, 0x08, 0x1d, 0x55, 0xac, 0x4d, 0x98, 0x42, 0x3d, 0xc2, 0xf2, 0x7f, 0x5b, 0x53,
];

pub const GP_CELL_SIZE_XY_CLASS_ID: [u8; 16] = [
    0x6a, 0x89, 0xb9, 0x4e, 0xb8, 0x27, 0xc2, 0x4c, 0xa0, 0xc2, 0x55, 0xe4, 0x0d, 0x2e, 0x92, 0x8c,
];

pub const GP_SQL_EXPRESSION_CLASS_ID: [u8; 16] = [
    0x74, 0xfb, 0x66, 0xf9, 0x71, 0xea, 0xc7, 0x41, 0xa8, 0x48, 0x8d, 0x26, 0xa7, 0x1d, 0xb9, 0x1a,
];

pub const GP_INFO_EXPRESSION_CLASS_ID: [u8; 16] = [
    0xa6, 0x65, 0xa2, 0x32, 0xd2, 0x78, 0xa5, 0x49, 0xa6, 0x5b, 0x57, 0x29, 0x73, 0xa7, 0xb6, 0xa0,
];

pub const GP_CALCULATOR_EXPRESSION_CLASS_ID: [u8; 16] = [
    0xa0, 0x38, 0x2c, 0x95, 0xef, 0x4c, 0x8a, 0x48, 0x8c, 0x99, 0x7d, 0x19, 0xb6, 0xf7, 0x4f, 0x91,
];

pub const GP_ROUTE_MEASURE_EVENT_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x95, 0x7f, 0x52, 0x35, 0x45, 0xbb, 0xce, 0x4b, 0xa8, 0x13, 0xea, 0x13, 0x69, 0xe7, 0x35, 0xbc,
];

pub const GP_SPATIAL_REFERENCE_CLASS_ID: [u8; 16] = [
    0xea, 0x07, 0x33, 0x50, 0x31, 0x41, 0x6b, 0x45, 0x9b, 0x9a, 0x15, 0x98, 0x95, 0x34, 0xbe, 0x4d,
];

pub const GP_COORDINATE_SYSTEM_CLASS_ID: [u8; 16] = [
    0xed, 0xd7, 0xe1, 0xb3, 0xd6, 0xa8, 0xa7, 0x44, 0xa9, 0x11, 0x81, 0x7c, 0x36, 0x1b, 0x57, 0x4c,
];

pub const GP_XY_DOMAIN_CLASS_ID: [u8; 16] = [
    0x7b, 0x28, 0x76, 0x97, 0x47, 0x3d, 0xb7, 0x4d, 0x80, 0x1c, 0xd6, 0x79, 0xd7, 0xaa, 0x68, 0x72,
];

pub const GP_Z_DOMAIN_CLASS_ID: [u8; 16] = [
    0x86, 0xfe, 0xf4, 0x44, 0x70, 0x88, 0x75, 0x43, 0xa9, 0xc2, 0x96, 0xae, 0xbb, 0x9d, 0xe3, 0xbc,
];

pub const GP_M_DOMAIN_CLASS_ID: [u8; 16] = [
    0x9e, 0x9b, 0x25, 0x84, 0xc9, 0x73, 0x2a, 0x46, 0xbe, 0xb1, 0x3b, 0xb3, 0x05, 0x1e, 0x59, 0x3a,
];

pub const GP_ENVELOPE_CLASS_ID: [u8; 16] = [
    0x6b, 0x67, 0xa9, 0x01, 0x51, 0xe2, 0x40, 0x4a, 0x85, 0x44, 0x5e, 0x5b, 0x65, 0xe2, 0x98, 0x3c,
];

pub const GP_EXTENT_CLASS_ID: [u8; 16] = [
    0xdc, 0x9c, 0x4f, 0x75, 0x51, 0xee, 0x2a, 0x41, 0x97, 0xea, 0xe5, 0x73, 0x07, 0x7c, 0x55, 0x9f,
];

#[derive(Debug, Clone, PartialEq)]
pub enum GpCodedValue10_1 {
    RecordSet(Box<GpRecordSet10_1>),
    FeatureRecordSetLayer(Box<GpFeatureRecordSetLayer10_1>),
    TableView(Box<GpTableView10_1>),
    MosaicLayer(Box<GpLayer10_1>),
    GroupLayer(Box<GpLayer10_1>),
    NaLayer(Box<GpLayer10_1>),
    CadastralFabricLayer(Box<GpLayer10_1>),
    TinLayer(Box<GpLayer10_1>),
    TerrainLayer(Box<GpLayer10_1>),
    TopologyLayer(Box<GpLayer10_1>),
    NetworkDatasetLayer(Box<GpLayer10_1>),
    LasDatasetLayer(Box<GpLayer10_1>),
    RasterLayer(Box<GpLayer10_1>),
    RasterCatalogLayer(Box<GpLayer10_1>),
    FeatureLayer(Box<GpLayer10_1>),
    RasterBuilder(RuleSlot10_1<RasterFunctionArgumentsBody10_1>),
    RasterFormulated(Box<GpRasterFormulated10_1>),
    RasterDataLayer {
        name: Box<BookmarkNameEntry10_1>,
        text: String,
        drawing: RuleSlot10_1<LayerDrawingDescription10_1>,
    },
    RasterData {
        name: Box<BookmarkNameEntry10_1>,
        text: String,
    },
    FieldMapping {
        raw_map_count: i32,
        maps: Vec<RuleSlot10_1<GpFieldMap10_1>>,
        raw_table_count: i32,
        tables: Vec<RuleSlot10_1<DeTable10_1>>,
        tail: Option<RuleSlot10_1<DeTable10_1>>,
    },
    Replica(Box<GpReplica10_1>),
    ReplicaDescription {
        raw_longs: [u32; 2],
        raw_shorts: [u16; 2],
        geometry: RuleSlot10_1<crate::objects::geometry::GeometryBody10_1>,
        datasets: RuleSlot10_1<GpReplicaDatasets10_1>,
    },
    ReplicaDataset {
        strings: [String; 2],
        raw_longs: [u32; 2],
        raw_shorts: [u16; 2],
        type_11: Option<(u32, Option<[String; 2]>)>,
        indices: crate::layers::raster_dataset_name::RasterLongArraySlot10_1,
        v3: Option<(String, u32, [String; 2], u32)>,
    },
    LineOrPolygon {
        raw_count: i32,
        points: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
    },
    Field(Box<FieldBody10_1>),
    FmeDataset(Option<Vec<u8>>),
    XmlIndex {
        first: String,
        fields: RuleSlot10_1<FieldsBody10_1>,
        raw_long: u32,
        tags: Box<[RuleSlot10_1<GpCodedValueBody10_1>; 2]>,
        second: String,
    },
    XmlIndexTags {
        raw_count: i32,
        tags: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
    },
    XmlIndexTag {
        first: String,
        raw_longs: [u32; 2],
        second: String,
    },
    Point {
        x_bits: u64,
        y_bits: u64,
    },
    WorkspaceExtension {
        name: String,
        extension_guid: [u8; 16],
    },
    DataFile {
        data: RuleSlot10_1<GdsData10_1>,
        strings: [String; 3],
    },
    Decimate3da {
        raw_leading_long: i32,
        value_bits: u64,
        raw_trailing_long: i32,
    },
    Interpolate3da {
        raw_longs: [i32; 6],
        value_bits: u64,
    },
    NaHierarchySettings {
        raw_longs: [i32; 2],
    },
    NaClassFieldMap {
        raw_count: i32,
        rows: Vec<NaClassFieldMapRow10_1>,
    },
    AddressLocatorStyle {
        strings: [String; 2],
        configuration: Option<Box<GpAddressLocatorStyleConfig10_1>>,
    },
    FieldInfoEx(String),
    FieldInfo {
        raw_count: i32,
        rows: Vec<GpFieldInfoRow10_1>,
    },
    ArcInfoItem {
        strings: [String; 2],
        raw_longs: [i32; 5],
        raw_shorts: [u16; 3],
    },
    AnalysisCellSize {
        size_bits: u64,
        raw_text: String,
    },
    RasterEnvironment(Box<GpRasterEnvironment10_1>),
    RasterCalculatorExpression {
        expression: String,
        raw_count: i32,
        inputs: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
    },
    SaExtractValues {
        raw_count: i32,
        rows: Vec<[RuleSlot10_1<GpCodedValueBody10_1>; 2]>,
    },
    SaWeightedOverlayTable(Box<GpSaWeightedOverlayTable10_1>),
    SaWeightedSum {
        raw_count: i32,
        rows: Vec<[RuleSlot10_1<GpCodedValueBody10_1>; 3]>,
    },
    SaFuzzyFunction {
        arguments: [RuleSlot10_1<GpCodedValueBody10_1>; 3],
        raw_flags: [u8; 2],
    },
    SaDistanceFactor(RuleSlot10_1<PathDistanceFactor10_1>),
    SaNeighborhood(Box<GpSaNeighborhood10_1>),
    SaRemap(Box<GpSaRemap10_1>),
    SaSemiVariogram(Box<GpSaSemiVariogram10_1>),
    SaTimeConfiguration(RuleSlot10_1<TimeConfiguration10_1>),
    SaRadius {
        legacy_name: Option<Box<BookmarkNameEntry10_1>>,
        raw_longs: [i32; 2],
        distance_bits: u64,
    },
    EvaluationScale {
        legacy_name: Option<Box<BookmarkNameEntry10_1>>,
        raw_longs: [i32; 3],
    },
    SaMapAlgebraExpression {
        legacy_name: Option<Box<BookmarkNameEntry10_1>>,
        expression: String,
    },
    SaCellSize {
        raw_mode: i32,
        x_bits: u64,
        y_bits: u64,
    },
    CellSizeXy {
        x_bits: u64,
        y_bits: u64,
    },
    String(String),
    EncryptedString(String),
    SpatialReference {
        reference: Box<SpatialReferenceEntry10_1>,
        legacy_short: Option<u16>,
    },
    Envelope {
        envelope: Box<BookmarkEnvelopeEntry10_1>,
        raw_mode: Option<i32>,
    },
    SqlExpression(String),
    InfoExpression(String),
    CalculatorExpression(String),
    RouteMeasureEventProperties([String; 4]),
    Long(i32),
    DoubleBits(u64),
    Boolean(u16),
    Date(Box<PropertyValueObjectStream10_1>),
    LinearUnit {
        value_bits: u64,
        unit: i32,
    },
    ArealUnit {
        value_bits: u64,
        unit: i32,
    },
    Variant(Box<PropertyValueObjectStream10_1>),
    MultiValue {
        type_name: Box<BookmarkNameEntry10_1>,
        raw_count: i32,
        values: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
    },
    ValueTable {
        raw_column_count: i32,
        column_types: Vec<BookmarkNameEntry10_1>,
        raw_row_count: i32,
        cells: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
    },
    RandomNumberGenerator {
        seed: i32,
        raw_generator: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFieldInfoRow10_1 {
    pub leading_long: i32,
    pub strings: [String; 2],
    pub trailing_long: i32,
    pub raw_short: u16,
    pub extra: Option<GpFieldInfoRowV2Fields10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpAddressLocatorStyleConfig10_1 {
    pub raw_flag: u16,
    pub fields: [RuleSlot10_1<FieldsBody10_1>; 3],
    pub raw_row_count: i32,
    pub rows: Vec<GpAddressLocatorStyleRow10_1>,
    pub legacy_byte: Option<u8>,
    pub properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
    pub variant_flag: Option<u16>,
    pub variant: Option<PropertyValueObjectStream10_1>,
    pub table: Option<RuleSlot10_1<DeTable10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpAddressLocatorStyleRow10_1 {
    pub name: String,
    pub raw_short: Option<u16>,
    pub raw_object_count: i32,
    pub uids: Vec<RuleSlot10_1<UidBody10_1>>,
    pub raw_value_count: i32,
    pub fields: RuleSlot10_1<FieldsBody10_1>,
    pub values: GpAddressLocatorStyleRowValues10_1,
    pub table: RuleSlot10_1<DeTable10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpAddressLocatorStyleRowValues10_1 {
    Strings(Vec<Vec<String>>),
    Variants(Vec<PropertyValueObjectStream10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTable10_1 {
    pub version: u16,
    pub dataset: DeDataset10_1,
    pub raw_short: u16,
    pub strings: [String; 6],
    pub raw_long: i32,
    pub string_array: StringArraySlot10_1,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub raw_subtype_count: i32,
    pub subtypes: Vec<RuleSlot10_1<GpSubtype10_1>>,
    pub raw_membership_count: i32,
    pub memberships: Vec<RuleSlot10_1<GpControllerMembership10_1>>,
    pub v2: Option<(u16, [String; 2])>,
    pub v3: Option<(u16, [String; 4])>,
    pub v4: Option<u16>,
    pub fields: RuleSlot10_1<FieldsBody10_1>,
    pub indexes: RuleSlot10_1<DeTableIndexes10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeDataset10_1 {
    pub dataset_version: u16,
    pub element: DataElement10_1,
    pub dataset_longs: [i32; 2],
    pub dataset_shorts: [u16; 2],
    pub dataset_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataElement10_1 {
    pub element_version: u16,
    pub gp_value_version: u16,
    pub gp_value_short: u16,
    pub element_strings: [String; 3],
    pub element_shorts: [u16; 3],
    pub element_legacy_xml: Option<String>,
    pub raw_child_count: i32,
    pub children: Vec<RuleSlot10_1<DeChild10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpCompositeLayerMembers10_1 {
    pub version: u16,
    pub base_version: u16,
    pub raw_count: i32,
    pub layers: Vec<RuleSlot10_1<GpLayer10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFeatureLayerFields10_1 {
    pub version: u16,
    pub text: String,
    pub fid_set: RuleSlot10_1<crate::layers::feature_id_set::FeatureIdSetBody10_1>,
    pub table: Option<RuleSlot10_1<DeChild10_1>>,
    pub field_info: Option<RuleSlot10_1<GpCodedValueBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFeatureRecordSetLayer10_1 {
    pub record_set: GpRecordSet10_1,
    pub layer_text: String,
    pub drawing: RuleSlot10_1<LayerDrawingDescription10_1>,
    pub templates: Option<RuleSlot10_1<crate::layers::annotate_props::EditTemplateManagerBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpRecordSet10_1 {
    pub records: RuleSlot10_1<RecordSet10_1>,
    pub v2: Option<(String, u16)>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordSet10_1 {
    pub version: i16,
    pub fields: RuleSlot10_1<FieldsBody10_1>,
    pub raw_count: i32,
    pub rows: Vec<Record10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record10_1 {
    pub version: i16,
    pub raw_count: i32,
    pub values: Vec<PropertyValueObjectStream10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpTableView10_1 {
    pub first_text: String,
    pub table: RuleSlot10_1<DeTable10_1>,
    pub second_text: String,
    pub fid_set: RuleSlot10_1<crate::layers::feature_id_set::FeatureIdSetBody10_1>,
    pub tail: Option<GpTableViewTail10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpTableViewTail10_1 {
    pub table: RuleSlot10_1<DeTable10_1>,
    pub field_info: RuleSlot10_1<GpCodedValueBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpRasterLayerFields10_1 {
    pub text: String,
    pub field: RuleSlot10_1<GpCodedValueBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpNaLayerFields10_1 {
    pub network: RuleSlot10_1<DeChild10_1>,
    pub definitions: Vec<RuleSlot10_1<crate::layers::na_class::NaClassDefinition10_1>>,
    pub linear_unit: RuleSlot10_1<GpCodedValueBody10_1>,
    pub agents: Vec<RuleSlot10_1<crate::layers::na_layer::NaLocatorFeatureAgent10_1>>,
    pub raw_short: u16,
    pub properties: [RuleSlot10_1<PropertySetObjectStream10_1>; 2],
    pub parameters: Option<RuleSlot10_1<crate::layers::na_layer::NaParameterArray10_1>>,
    pub tables: Option<Vec<RuleSlot10_1<GpCodedValueBody10_1>>>,
    pub tail_short: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpLayer10_1 {
    pub na: Option<Box<GpNaLayerFields10_1>>,
    pub raster: Option<GpRasterLayerFields10_1>,
    pub derived_version: Option<u16>,
    pub feature_layer: Option<GpFeatureLayerFields10_1>,
    pub composite: Option<GpCompositeLayerMembers10_1>,
    pub version: u16,
    pub gp_value_version: u16,
    pub gp_value_short: u16,
    pub name: String,
    pub area_of_interest: BookmarkEnvelopeEntry10_1,
    pub element: RuleSlot10_1<DeChild10_1>,
    pub legacy_layer: Option<RuleSlot10_1<crate::layers::layer_list::LayerBodyPrefix10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeLasDatasetTail10_1 {
    pub raw_longs: [i32; 3],
    pub raw_shorts: [u16; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeFeatureClassTable10_1 {
    Compact(DeCompactTable10_1),
    Extended(DeTable10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeCoverageFeatureClassTail10_1 {
    pub longs: [i32; 2],
    pub short: u16,
    pub raw_item_count: i32,
    pub items: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTerrainSource10_1 {
    pub version: u32,
    pub longs: [u32; 5],
    pub shorts: [u16; 3],
    pub values: [u64; 2],
    pub strings: [String; 3],
    pub v2: Option<u32>,
    pub name_count: i32,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTerrainLegacy10_1 {
    pub version: u32,
    pub leading_long: u32,
    pub leading_value: u64,
    pub longs: [u32; 2],
    pub source_count: i32,
    pub sources: Vec<DeTerrainSource10_1>,
    pub level_count: i32,
    pub levels: Vec<([u32; 3], u64)>,
    pub v2: Option<(String, u32, [u16; 5])>,
    pub v3: Option<String>,
    pub v4: Option<(BookmarkEnvelopeEntry10_1, SpatialReferenceEntry10_1)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTerrainV9Tail10_1 {
    pub v11: Option<u16>,
    pub v13: Option<u32>,
    pub longs: [u32; 2],
    pub envelopes: [[u64; 6]; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTerrainGeoDataset10_1 {
    pub version: u32,
    pub geo_dataset_version: u16,
    pub dataset: DeDataset10_1,
    pub extent: BookmarkEnvelopeEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub level_type: u32,
    pub v6: Option<u32>,
    pub v7_bits: Option<u64>,
    pub v8: Option<u32>,
    pub value_bits: u64,
    pub longs: [u32; 2],
    pub name: String,
    pub v9_tail: Option<DeTerrainV9Tail10_1>,
    pub source_count: i32,
    pub sources: Vec<DeTerrainSource10_1>,
    pub level_count: i32,
    pub levels: Vec<([u32; 3], u64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CadastralFieldEdits10_1 {
    pub version: i16,
    pub fields: RuleSlot10_1<FieldsBody10_1>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeCadastralTail10_1 {
    pub longs: [u32; 4],
    pub values: [u64; 2],
    pub name: String,
    pub shorts: [u16; 5],
    pub properties: Option<[RuleSlot10_1<PropertySetObjectStream10_1>; 5]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeCadastralFabric10_1 {
    pub version: i16,
    pub geo_dataset_version: u16,
    pub dataset: DeDataset10_1,
    pub extent: BookmarkEnvelopeEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub longs: [u32; 2],
    pub name: String,
    pub edit_count: i32,
    pub edits: Vec<RuleSlot10_1<CadastralFieldEdits10_1>>,
    pub tail_version: i16,
    pub tail: Option<DeCadastralTail10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeToolboxExtension10_1 {
    pub version: u16,
    pub name: String,
    pub v2: Option<([String; 2], u32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeRepresentationClass10_1 {
    pub version: i16,
    pub dataset: DeDataset10_1,
    pub legacy_longs: Option<[u32; 2]>,
    pub first_name: Option<String>,
    pub names: [String; 2],
    pub rules: RuleSlot10_1<RepresentationRules10_1>,
    pub attributes: RuleSlot10_1<ClassAttributes10_1>,
    pub trailing_short: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepresentationRules10_1 {
    pub version: i16,
    pub value: u32,
    pub raw_count: i32,
    pub items: Vec<RepresentationRuleItem10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepresentationRuleItem10_1 {
    pub id: u32,
    pub name: String,
    pub v1: Option<u32>,
    pub rule: RuleSlot10_1<crate::objects::display_symbol::RepresentationRuleBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassAttributes10_1 {
    pub version: i16,
    pub attributes: crate::objects::display_symbol::GraphicAttributeList10_1,
    pub raw_count: i32,
    pub names: Vec<(u32, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeNetworkDataset10_1 {
    pub version: u16,
    pub geo_dataset_version: u16,
    pub dataset: DeDataset10_1,
    pub extent: BookmarkEnvelopeEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub name: String,
    pub initial_values: [u32; 6],
    pub directions: Option<DirectionSlot10_1<NetworkDirections10_1>>,
    pub properties: [RuleSlot10_1<PropertySetObjectStream10_1>; 2],
    pub sources: (i32, Vec<DirectionSlot10_1<NetworkSource10_1>>),
    pub attributes: (i32, Vec<DirectionSlot10_1<NetworkAttributeEntryBody10_1>>),
    pub evaluator_rows: (i32, Vec<DeNetworkEvaluatorRow10_1>),
    pub extra_name: String,
    pub attribute_flag: u32,
    pub extra_attribute: Option<DirectionSlot10_1<NetworkAttributeEntryBody10_1>>,
    pub values: (i32, Vec<u32>),
    pub extra_value: Option<u32>,
    pub traffic: Option<DeNetworkTraffic10_1>,
    pub final_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeNetworkEvaluatorRow10_1 {
    pub values: [u32; 4],
    pub uid: RuleSlot10_1<UidBody10_1>,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeNetworkTraffic10_1 {
    pub values: [u32; 3],
    pub data: Option<DirectionSlot10_1<TrafficData10_1>>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeMosaicDataset10_1 {
    pub version: i16,
    pub geo_dataset_version: u16,
    pub dataset: DeDataset10_1,
    pub extent: BookmarkEnvelopeEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub value: PropertyValueObjectStream10_1,
    pub raw_short: u16,
    pub raw_long: Option<i32>,
    pub catalog: Option<RuleSlot10_1<DeChild10_1>>,
    pub names: StringArraySlot10_1,
    pub table: Option<RuleSlot10_1<DeTable10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeChild10_1 {
    MosaicDataset(Box<DeMosaicDataset10_1>),
    SchematicDiagram {
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        text: String,
    },
    SchematicDataset {
        dataset: DeDataset10_1,
    },
    RepresentationClass(Box<DeRepresentationClass10_1>),
    Toolbox {
        dataset: DeDataset10_1,
        extension: Option<DeToolboxExtension10_1>,
    },
    Tool {
        version: u16,
        dataset: DeDataset10_1,
    },
    CadastralFabric(Box<DeCadastralFabric10_1>),
    NetworkDataset(Box<DeNetworkDataset10_1>),
    TerrainLegacy(Box<DeTerrainLegacy10_1>),
    TerrainGeoDataset(Box<DeTerrainGeoDataset10_1>),
    TableInfo(DeCompactTable10_1),
    LasDataset {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        tail: Option<DeLasDatasetTail10_1>,
    },
    Coverage {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    },
    Tin {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        value_bits: u64,
        raw_shorts: [u16; 4],
        fields: RuleSlot10_1<FieldsBody10_1>,
    },
    CadDrawingDataset {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        raw_shorts: [u16; 4],
    },
    ArcInfoTable {
        version: u16,
        table: DeCompactTable10_1,
        raw_item_count: i32,
        items: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
    },
    FileTable {
        version: u16,
        table: DeCompactTable10_1,
    },
    Layer {
        version: u16,
        element: DataElement10_1,
        layer: RuleSlot10_1<GpLayer10_1>,
    },
    TextFile {
        version: u16,
        base: DeTextFileBase10_1,
    },
    FileSystem {
        version: u16,
        element: DataElement10_1,
        spatial_reference: Option<Box<SpatialReferenceEntry10_1>>,
    },
    RasterBand(Box<DeRasterBand10_1>),
    Workspace(Box<DeWorkspace10_1>),
    Server {
        version: u16,
        element: DataElement10_1,
        strings: [String; 2],
    },
    RelationshipClass(Box<DeRelationshipClass10_1>),
    Table(DeTable10_1),
    Topology {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        value_bits: [u64; 2],
        raw_long: i32,
        names: StringArraySlot10_1,
        raw_rule_count: Option<i32>,
        rules: Vec<RuleSlot10_1<TopologyRule10_1>>,
    },
    GeometricNetwork {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        network: DeGeometricNetwork10_1,
    },
    RasterDataset {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        strings: [String; 3],
        raw_long: i32,
        raw_short: u16,
        storage: Option<RuleSlot10_1<RasterStorageDef10_1>>,
    },
    CatalogDataset {
        version: u16,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        raw_longs: [i32; 2],
    },
    FeatureDataset {
        version: u16,
        vpf_base_version: Option<u16>,
        geo_dataset_version: u16,
        dataset: DeDataset10_1,
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
    },
    FeatureClass {
        derived_version: Option<u16>,
        version: u16,
        table: DeFeatureClassTable10_1,
        raw_longs: [i32; 2],
        name: String,
        raw_shorts: [u16; 3],
        extent: BookmarkEnvelopeEntry10_1,
        spatial_reference: SpatialReferenceEntry10_1,
        strings: Option<[String; 2]>,
        coverage: Option<DeCoverageFeatureClassTail10_1>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeRasterBand10_1 {
    pub version: u16,
    pub table: DeCompactTable10_1,
    pub raw_short: u16,
    pub double_bits: [u64; 2],
    pub longs: [i32; 2],
    pub v3: Option<i32>,
    pub no_data: RasterBandNoData10_1,
    pub trailing_longs: [i32; 2],
    pub extent: BookmarkEnvelopeEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeTextFileBase10_1 {
    Element(DataElement10_1),
    Table(DeCompactTable10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeCompactTable10_1 {
    pub table_version: u16,
    pub dataset: DeDataset10_1,
    pub table_short: u16,
    pub table_string: String,
    pub fields: RuleSlot10_1<FieldsBody10_1>,
    pub indexes: RuleSlot10_1<DeTableIndexes10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterBandNoData10_1 {
    Legacy([u8; 24]),
    Variant(PropertyValueObjectStream10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterStorageDef10_1 {
    pub version: u16,
    pub longs: [i32; 6],
    pub point_flag: Option<u8>,
    pub xy_bits: Option<[u64; 2]>,
    pub point: Option<RuleSlot10_1<PointBody10_1>>,
    pub short: Option<u16>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeWorkspace10_1 {
    pub version: u16,
    pub element: DataElement10_1,
    pub factory: Option<(String, [String; 2])>,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub raw_domain_count: i32,
    pub domains: Vec<DomainSlot10_1>,
    pub v1: Option<i32>,
    pub v4: Option<[i32; 3]>,
    pub v5: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeRelationshipClass10_1 {
    pub version: u16,
    pub base: DeRelationshipBase10_1,
    pub metadata: Option<DeRelationshipMetadata10_1>,
    pub raw_rule_count: Option<i32>,
    pub rules: Vec<RuleSlot10_1<RelationshipRule10_1>>,
    pub fields: Option<RuleSlot10_1<FieldsBody10_1>>,
    pub v6: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipRule10_1 {
    pub version: u16,
    pub name: String,
    pub raw_longs: [i32; 10],
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeRelationshipBase10_1 {
    Dataset(DeDataset10_1),
    Table(Box<DeTable10_1>),
    CompactTable(Box<DeCompactTable10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeRelationshipMetadata10_1 {
    pub names: [StringArraySlot10_1; 2],
    pub longs: [i32; 4],
    pub strings: [String; 2],
    pub shorts: [u16; 3],
    pub raw_key_counts: [i32; 2],
    pub keys: [Vec<RuleSlot10_1<GpRelationshipClassKey10_1>>; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpRelationshipClassKey10_1 {
    pub version: u16,
    pub raw_long: i32,
    pub strings: [String; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeGeometricNetwork10_1 {
    pub metadata: Option<(i32, String, StringArraySlot10_1)>,
    pub raw_counts: Option<[i32; 3]>,
    pub rules: Vec<RuleSlot10_1<ConnectivityRule10_1>>,
    pub weights: Vec<RuleSlot10_1<NetWeight10_1>>,
    pub associations: Vec<RuleSlot10_1<NetWeight10_1>>,
    pub v4: Option<(u16, [u64; 2])>,
    pub v5: Option<[i32; 2]>,
    pub v6: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectivityRule10_1 {
    Edge {
        version: u16,
        name: String,
        longs: [i32; 8],
        raw_count: i32,
        pairs: Vec<[i32; 2]>,
    },
    Junction {
        version: u16,
        name: String,
        longs: [i32; 10],
        short: u16,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetWeight10_1 {
    Weight {
        version: u16,
        raw_long: i32,
        name: String,
        trailing_longs: [i32; 2],
    },
    Association {
        version: u16,
        raw_long: i32,
        strings: [String; 2],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopologyRule10_1 {
    pub version: u16,
    pub base_version: u16,
    pub base_name: String,
    pub base_longs: [i32; 2],
    pub strings: [String; 2],
    pub longs: [i32; 4],
    pub shorts: [u16; 5],
    pub trailing_long: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpControllerMembership10_1 {
    Topology {
        version: u16,
        name: String,
        value_bits: u64,
        raw_longs: [i32; 2],
        raw_short: u16,
    },
    GeometricNetwork {
        version: u16,
        strings: [String; 3],
        raw_long: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSubtype10_1 {
    pub version: u16,
    pub name: String,
    pub raw_code: i32,
    pub raw_count: i32,
    pub fields: Vec<GpSubtypeField10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSubtypeField10_1 {
    pub strings: [String; 2],
    pub default: PropertyValueObjectStream10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTableIndexes10_1 {
    pub version: u16,
    pub raw_count: i32,
    pub entries: Vec<RuleSlot10_1<DeTableIndex10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeTableIndex10_1 {
    pub version: u16,
    pub gp_value_version: u16,
    pub gp_value_short: u16,
    pub name: String,
    pub flags: [u8; 3],
    pub fields: RuleSlot10_1<FieldsBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaClassFieldMapRow10_1 {
    pub strings: [String; 2],
    pub value: PropertyValueObjectStream10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFieldInfoRowV2Fields10_1 {
    pub first_text: String,
    pub first_long: i32,
    pub second_text: String,
    pub remaining_longs: [i32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct GdsData10_1 {
    pub version: i16,
    pub bytes: Vec<u8>,
    pub name: String,
    pub raw_long: i32,
    pub raw_short: u16,
    pub properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
    pub extra_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpRasterEnvironment10_1 {
    pub legacy_name: Option<Box<BookmarkNameEntry10_1>>,
    pub parameters: GpRasterEnvironmentParameters10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpRasterEnvironmentParameters10_1 {
    Compression([i32; 2]),
    TileSize([i32; 2]),
    Statistics {
        raw_short: u16,
        raw_longs: [i32; 2],
        ignore_values: GpRasterStatisticsIgnoreValues10_1,
    },
    Pyramid {
        raw_short: u16,
        raw_longs: [i32; 2],
        version_three_longs: Option<[i32; 2]>,
        version_four_short: Option<u16>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpRasterStatisticsIgnoreValues10_1 {
    LegacyBytes([u8; 4]),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathDistanceFactor10_1 {
    pub version: u16,
    pub kind: i32,
    pub parameters: PathDistanceFactorParameters10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathDistanceFactorParameters10_1 {
    None,
    Numbers(Vec<u64>),
    Path(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaSemiVariogram10_1 {
    pub legacy_name: Option<Box<BookmarkNameEntry10_1>>,
    pub variogram_type: i32,
    pub lag: GpSaSemiVariogramLag10_1,
    pub range_bits: u64,
    pub partial_sill_bits: u64,
    pub nugget_bits: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpSaSemiVariogramLag10_1 {
    LegacyText(Vec<u8>),
    Number(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeConfiguration10_1 {
    pub version: u16,
    pub kind: i32,
    pub parameters: TimeConfigurationParameters10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeConfigurationParameters10_1 {
    None,
    LongAndDoubles {
        raw_long: i32,
        double_bits: [u64; 2],
    },
    Longs([i32; 3]),
    Long(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaWeightedOverlayTable10_1 {
    pub scale: GpSaOverlayScale10_1,
    pub raw_count: i32,
    pub rows: Vec<GpSaOverlayRow10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpSaOverlayScale10_1 {
    Legacy {
        name: Box<BookmarkNameEntry10_1>,
        limits_bits: [u64; 2],
        raw_long: i32,
    },
    Object(RuleSlot10_1<GpCodedValueBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaOverlayRow10_1 {
    pub values: [RuleSlot10_1<GpCodedValueBody10_1>; 3],
    pub remap: GpSaOverlayRemap10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpSaOverlayRemap10_1 {
    Legacy {
        raw_count: i32,
        rows: Vec<GpSaRemapRow10_1>,
    },
    Object(RuleSlot10_1<GpCodedValueBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaNeighborhood10_1 {
    pub legacy_name: Option<Box<BookmarkNameEntry10_1>>,
    pub storage_kind: i32,
    pub shape_kind: i32,
    pub raw_long: i32,
    pub parameters: GpSaNeighborhoodParameters10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpSaNeighborhoodParameters10_1 {
    None,
    Dimensions(Vec<u64>),
    Path(String),
    Embedded {
        raw_longs: [i32; 2],
        values: Box<PropertyValueObjectStream10_1>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaRemap10_1 {
    pub legacy_name: Option<Box<BookmarkNameEntry10_1>>,
    pub raw_long: i32,
    pub raw_count: i32,
    pub version_three_fields: Option<(u16, i32)>,
    pub version_four_fields: Option<(u16, [i32; 2])>,
    pub rows: Vec<GpSaRemapRow10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpSaRemapRow10_1 {
    Number {
        range_bits: [u64; 2],
        raw_long: i32,
        text_bytes: Vec<u8>,
    },
    String {
        source_bytes: Vec<u8>,
        raw_long: i32,
        text_bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayerDrawingDescription10_1 {
    Raster(RasterLayerDrawingDescription10_1),
    Feature(FeatureLayerDrawingDescription10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeatureLayerDrawingDescription10_1 {
    pub version: i16,
    pub renderer: crate::layers::feature_layer::RendererSlot10_1,
    pub raw_short: Option<u16>,
    pub raw_shorts: Option<[u16; 3]>,
    pub labeling: Option<RuleSlot10_1<LabelingDescription10_1>>,
    pub raw_long: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelingDescription10_1 {
    pub version: i16,
    pub classes: RuleSlot10_1<LabelClassDescriptions10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelClassDescriptions10_1 {
    pub version: i16,
    pub raw_count: i32,
    pub classes: Vec<RuleSlot10_1<LabelClassDescription10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelClassDescription10_1 {
    pub version: i16,
    pub placement: RuleSlot10_1<LabelPlacementDescription10_1>,
    pub text: String,
    pub symbol: crate::objects::display_symbol::SymbolSlotFull10_1,
    pub raw_doubles: [f64; 2],
    pub raw_short: Option<u16>,
    pub tail: Option<(i32, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPlacementDescription10_1 {
    pub version: i16,
    pub raw_long: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerDrawingDescription10_1 {
    pub version: i16,
    pub renderer: crate::layers::raster_renderer::RasterRendererSlot10_1,
    pub raw_shorts: Option<[u16; 3]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpRasterFormulated10_1 {
    pub raster_version: u16,
    pub raster_long: u32,
    pub raster_text: String,
    pub envelope: BookmarkEnvelopeEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub raw_long: u32,
    pub value: GpRasterFormulatedValue10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpRasterFormulatedValue10_1 {
    Legacy([u8; 24]),
    Variant(PropertyValueObjectStream10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFieldMap10_1 {
    pub version: u16,
    pub raw_long: u32,
    pub delimiter: String,
    pub field: RuleSlot10_1<GpCodedValueBody10_1>,
    pub raw_count: i32,
    pub rows: Vec<GpFieldMapRow10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFieldMapRow10_1 {
    pub field: RuleSlot10_1<GpCodedValueBody10_1>,
    pub table: RuleSlot10_1<DeTable10_1>,
    pub raw_longs: [u32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpReplica10_1 {
    pub initial_long: u32,
    pub strings: [String; 4],
    pub variant: PropertyValueObjectStream10_1,
    pub long_before_description: u32,
    pub description: RuleSlot10_1<GpCodedValueBody10_1>,
    pub trailing_longs: [u32; 6],
    pub trailing_short: u16,
    pub v2: Option<GpReplicaV2Tail10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpReplicaV2Tail10_1 {
    pub variants: [PropertyValueObjectStream10_1; 3],
    pub raw_shorts: [u16; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpReplicaDatasets10_1 {
    pub version: i16,
    pub raw_count: i32,
    pub datasets: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpCodedValueBody10_1 {
    pub version: u16,
    pub base_version: u16,
    pub base_short: u16,
    pub value: GpCodedValue10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpCodedValueDomain10_1 {
    pub version: i16,
    pub labels: (i32, Vec<String>),
    pub raw_value_count: i32,
    pub values: Vec<RuleSlot10_1<GpCodedValueBody10_1>>,
}

pub const GP_COMPOSITE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x25, 0x11, 0xae, 0xf9, 0xef, 0x48, 0x2d, 0x41, 0xab, 0x8c, 0xad, 0x91, 0x7f, 0x3d, 0x5f, 0x8e,
];

pub const GP_DATA_ELEMENT_DOMAIN_CLASS_ID: [u8; 16] = [
    0x28, 0x9d, 0xf3, 0xb6, 0xf6, 0xc6, 0xb4, 0x45, 0xa6, 0x0c, 0x26, 0x01, 0xa1, 0x1b, 0xe4, 0xbd,
];

pub const GP_REPRESENTATION_LAYER_DOMAIN_CLASS_ID: [u8; 16] = [
    0x85, 0xcb, 0x1a, 0x82, 0x63, 0x50, 0xa3, 0x48, 0xb2, 0xd5, 0x3a, 0x95, 0xf1, 0xc8, 0xc7, 0x3d,
];

pub const GP_RA_FIELD_DOMAIN_CLASS_ID: [u8; 16] = [
    0x58, 0xa8, 0x6c, 0x4b, 0x16, 0x57, 0xc3, 0x4a, 0xa2, 0xee, 0x70, 0xee, 0x2d, 0x29, 0xc1, 0xbd,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpRaFieldDomain10_1 {
    pub version: i16,
    pub long_list: (i32, Vec<i32>),
    pub shorts: [u16; 2],
    pub string_lists: [(i32, Vec<String>); 2],
}

pub const GP_SA_NEIGHBORHOOD_DOMAIN_CLASS_ID: [u8; 16] = [
    0xfc, 0x70, 0x4c, 0x25, 0xf5, 0xf9, 0xaf, 0x4b, 0xa0, 0x39, 0x0c, 0x5c, 0xd3, 0x73, 0x38, 0x05,
];

pub const GP_SA_REMAP_DOMAIN_CLASS_ID: [u8; 16] = [
    0x6d, 0x51, 0xa8, 0xac, 0xd9, 0x30, 0x02, 0x40, 0x87, 0x44, 0x62, 0x41, 0x35, 0x0e, 0x6f, 0x73,
];

pub const GP_SA_NAME_DOMAIN_CLASS_ID: [u8; 16] = [
    0x4e, 0x8f, 0xa7, 0xc6, 0xf5, 0xa1, 0x1a, 0x42, 0xad, 0x8c, 0xc4, 0xc9, 0xec, 0x85, 0x4a, 0x40,
];

pub const GP_SA_TEXT_TABLE_NAME_DOMAIN_CLASS_ID: [u8; 16] = [
    0x5a, 0x92, 0x1b, 0xa5, 0x1e, 0x78, 0x3b, 0x4a, 0x99, 0x20, 0xb7, 0x6f, 0xc2, 0xc4, 0x32, 0xda,
];

pub const GP_SA_GEO_DATA_DOMAIN_CLASS_ID: [u8; 16] = [
    0x5f, 0x90, 0x6f, 0x61, 0x6d, 0x75, 0xc1, 0x46, 0xa5, 0xe9, 0xa7, 0x70, 0x76, 0xc8, 0xc7, 0x40,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaGeoDataDomain10_1 {
    pub version: i16,
    pub raw_name_count: i32,
    pub names: Vec<BookmarkNameEntry10_1>,
    pub first_list: (i32, Vec<i32>),
    pub second_list: (i32, Vec<i32>),
    pub trailing_short: u16,
    pub trailing_string: String,
}

pub const GP_TABLES_DOMAIN_CLASS_ID: [u8; 16] = [
    0x02, 0x00, 0xda, 0x5b, 0x74, 0x25, 0x2e, 0x49, 0xa3, 0x97, 0x82, 0x55, 0xfd, 0xff, 0x2d, 0x58,
];

pub const GP_LAYERS_AND_TABLES_DOMAIN_CLASS_ID: [u8; 16] = [
    0x0d, 0xa4, 0xe7, 0xec, 0x9c, 0xc0, 0xa1, 0x4d, 0xbe, 0x5d, 0x1a, 0x6d, 0x17, 0xfe, 0x6a, 0x82,
];

pub const GP_ROUTE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x2f, 0xff, 0x3f, 0x19, 0x23, 0xc9, 0x0b, 0x43, 0x87, 0x9f, 0x50, 0xbf, 0x65, 0xf6, 0x17, 0x03,
];

pub const GP_ROUTE_MEASURE_EVENT_DOMAIN_CLASS_ID: [u8; 16] = [
    0xf9, 0x4c, 0x7d, 0x5b, 0xe0, 0x95, 0x90, 0x49, 0xad, 0x6e, 0xd2, 0x5e, 0xf8, 0x9b, 0xae, 0xa8,
];

pub const GP_VIRTUAL_TABLE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x40, 0x53, 0x35, 0xbe, 0xa9, 0xb5, 0x32, 0x44, 0xb2, 0xf4, 0x49, 0x7a, 0x50, 0xe0, 0x51, 0x15,
];

pub const GP_QUERY_TABLE_IDS_DOMAIN_CLASS_ID: [u8; 16] = [
    0x60, 0xb0, 0xf6, 0x74, 0xb6, 0x5e, 0x51, 0x48, 0x8f, 0xfd, 0x8b, 0x18, 0x8a, 0x84, 0x5f, 0x37,
];
pub const GP_QUERY_TABLE_FIELDS_DOMAIN_CLASS_ID: [u8; 16] = [
    0x68, 0x36, 0xc4, 0x9c, 0xd8, 0x43, 0x82, 0x41, 0x98, 0x20, 0xb2, 0xd6, 0x20, 0x30, 0x5b, 0xed,
];

pub const GP_ROUTE_ID_FIELDS_DOMAIN_CLASS_ID: [u8; 16] = [
    0xb0, 0x70, 0x4f, 0x4a, 0x3c, 0x91, 0x82, 0x4a, 0xa3, 0x3f, 0xe1, 0x90, 0xff, 0xa4, 0x09, 0xea,
];

pub const GP_MEASURE_FIELDS_DOMAIN_CLASS_ID: [u8; 16] = [
    0x25, 0x24, 0x6e, 0xc0, 0xd9, 0x30, 0x9d, 0x4c, 0x8c, 0xd3, 0x7f, 0xe2, 0x43, 0xb3, 0xaf, 0xcb,
];

pub const GP_FIELD_DOMAIN_CLASS_ID: [u8; 16] = [
    0xf1, 0xaf, 0xff, 0xd7, 0x90, 0x59, 0x28, 0x48, 0x82, 0x63, 0xaa, 0x93, 0xc1, 0xc0, 0xbd, 0x6a,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpFieldDomain10_1 {
    pub version: i16,
    pub raw_count: i32,
    pub values: Vec<i32>,
    pub shorts: [u16; 2],
    pub v2_strings: Option<(i32, Vec<String>)>,
    pub v3_strings: Option<(i32, Vec<String>)>,
    pub v4_short: Option<u16>,
}

pub const GP_ARC_INFO_ITEM_DOMAIN_CLASS_ID: [u8; 16] = [
    0x8c, 0xff, 0xc5, 0x01, 0x1b, 0xe0, 0x6d, 0x4a, 0xa9, 0x65, 0xe7, 0xe3, 0x02, 0x49, 0xe6, 0x41,
];

pub const GP_FEATURE_CLASS_DOMAIN_CLASS_ID: [u8; 16] = [
    0xff, 0xc5, 0x2e, 0x65, 0x72, 0x17, 0xab, 0x49, 0xb7, 0xc7, 0x43, 0x91, 0x43, 0xd4, 0x4d, 0x56,
];

pub const GP_DATASET_DOMAIN_CLASS_ID: [u8; 16] = [
    0x98, 0xda, 0x7c, 0x50, 0xfd, 0xce, 0xfc, 0x4b, 0xbf, 0x14, 0x21, 0xa4, 0x1b, 0x77, 0x3e, 0x2d,
];

pub const GP_WORKSPACE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x1e, 0x38, 0x38, 0x02, 0x3f, 0x08, 0x29, 0x4d, 0x93, 0x3d, 0x09, 0xb4, 0x6e, 0xcc, 0xcf, 0x73,
];

pub const GP_RANGE_DOMAIN_CLASS_ID: [u8; 16] = [
    0xc0, 0x41, 0x22, 0x5b, 0x1c, 0xa6, 0xd5, 0x11, 0x93, 0x1d, 0x00, 0x80, 0xc7, 0x1a, 0x32, 0x26,
];

pub const GP_FILE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x3a, 0x8e, 0xdc, 0x3e, 0x87, 0xff, 0xe1, 0x47, 0xaf, 0x34, 0x09, 0xbb, 0x4c, 0xb2, 0x2d, 0xa9,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpCoverageFeatureClassDomain10_1 {
    pub version: i16,
    pub long_list: (i32, Vec<i32>),
    pub v2_tail: Option<(u16, i32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpTopoDomain10_1 {
    CodedValue(GpCodedValueDomain10_1),
    FieldInfo {
        version: i16,
        coded_values: RuleSlot10_1<GpCodedValueDomain10_1>,
    },
    Composite {
        version: i16,
        raw_count: i32,
        domains: Vec<RuleSlot10_1<GpTopoDomain10_1>>,
    },
    DataElement {
        version: i16,
        raw_count: i32,
        strings: Vec<String>,
    },
    Coverage {
        version: i16,
        feature_class: RuleSlot10_1<GpCoverageFeatureClassDomain10_1>,
    },
    CoverageFeatureClass(GpCoverageFeatureClassDomain10_1),
    GdbAnnotation {
        version: i16,
    },
    CoverageAnnotation {
        version: i16,
    },
    CadAnnotation {
        version: i16,
    },
    RepresentationLayer {
        version: i16,
        v2_list: Option<(i32, Vec<i32>)>,
    },
    RaField(GpRaFieldDomain10_1),
    SaNeighborhood {
        version: i16,
        raw_count: i32,
        values: Vec<i32>,
    },
    SaRemap {
        version: i16,
        raw_count: i32,
        values: Vec<i32>,
    },
    SaName {
        version: i16,
        long: i32,
        short: u16,
    },
    SaTextTableName {
        version: i16,
        string: String,
    },
    SaGeoData(GpSaGeoDataDomain10_1),
    Numeric(GpNumericDomain10_1),
    Field(GpFieldDomain10_1),
    Tables {
        version: i16,
    },
    LayersAndTables {
        version: i16,
    },
    Route {
        version: i16,
    },
    RouteMeasureEvent {
        version: i16,
    },
    VirtualTable {
        version: i16,
    },
    QueryTableIds {
        version: i16,
    },
    QueryTableFields {
        version: i16,
    },
    RouteIdFields {
        version: i16,
    },
    MeasureFields {
        version: i16,
    },
    ArcInfoItem {
        version: i16,
        raw_count: i32,
        values: Vec<i32>,
        trailing_long: i32,
        trailing_shorts: [u16; 4],
    },
    FeatureClass {
        version: i16,
        raw_count: i32,
        values: Vec<i32>,
        shorts: [u16; 2],
        v2_shorts: Option<[u16; 4]>,
        v3_list: Option<(i32, Vec<i32>)>,
    },
    Dataset {
        version: i16,
        raw_count: i32,
        values: Vec<i32>,
        trailing_shorts: [u16; 4],
    },
    Workspace {
        version: i16,
        raw_count: i32,
        values: Vec<i32>,
    },
    Range {
        version: i16,
        flags: u32,
        bounds: [VariantValue10_1<Box<PropertyValueObjectStream10_1>>; 2],
    },
    File {
        version: i16,
        raw_count: i32,
        strings: Vec<String>,
    },
}

pub const GP_NUMERIC_DOMAIN_CLASS_ID: [u8; 16] = [
    0x80, 0xe1, 0xdd, 0xad, 0x22, 0x39, 0xf7, 0x48, 0x87, 0xc8, 0x34, 0x2f, 0xfa, 0xe4, 0x11, 0x61,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpNumericDomain10_1 {
    pub version: i16,
    pub first_short: u16,
    pub first_double_bits: u64,
    pub middle_shorts: [u16; 2],
    pub second_double_bits: u64,
}

pub const GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID: [u8; 16] = [
    0x7f, 0x55, 0x10, 0xaa, 0x72, 0x0b, 0x35, 0x43, 0x90, 0xad, 0x00, 0x65, 0x5b, 0x0d, 0x8b, 0xbe,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaTopoFeaturesTypeName10_1 {
    pub version: i16,
    pub legacy_count: Option<i32>,
    pub legacy_entries: Vec<GpSaTopoFeaturesTypeEntry10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpSaTopoFeaturesTypeEntry10_1 {
    pub data_type: BookmarkNameEntry10_1,
    pub legacy_domain: Option<RuleSlot10_1<GpTopoDomain10_1>>,
    pub string: String,
    pub value: i32,
    pub uid: Option<RuleSlot10_1<UidBody10_1>>,
}

pub const GP_FUNCTION_NAME_CLASS_ID: [u8; 16] = [
    0x42, 0x2f, 0x33, 0x9a, 0x7f, 0x37, 0x13, 0x42, 0x95, 0x4b, 0xb6, 0x8c, 0xc8, 0xb9, 0xce, 0x46,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpNameBase10_1 {
    pub version: i16,
    pub strings: [String; 4],
    pub legacy_string: Option<String>,
    pub uid: Option<RuleSlot10_1<UidBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpFunctionName10_1 {
    pub version: i16,
    pub base: GpNameBase10_1,
    pub legacy_uid: Option<RuleSlot10_1<UidBody10_1>>,
    pub v3_fields: Option<(i32, String, i32)>,
    pub v4_value: Option<i32>,
}

pub const GP_MULTI_VALUE_TYPE_NAME_CLASS_ID: [u8; 16] = [
    0x70, 0x33, 0xde, 0x35, 0x8e, 0x67, 0xfb, 0x40, 0x9b, 0x7e, 0xff, 0xaf, 0x08, 0x22, 0x4b, 0x99,
];

pub const GP_VALUE_TABLE_TYPE_NAME_CLASS_ID: [u8; 16] = [
    0x5f, 0x1a, 0x52, 0x65, 0xa6, 0x7d, 0x93, 0x48, 0xae, 0x58, 0xf7, 0x0f, 0xd3, 0xcd, 0x1d, 0xee,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpValueTableColumnType10_1 {
    pub data_type: BookmarkNameEntry10_1,
    pub legacy_domain: Option<RuleSlot10_1<GpTopoDomain10_1>>,
    pub name: String,
    pub raw_long: i32,
    pub uid: Option<RuleSlot10_1<UidBody10_1>>,
}

pub const GP_DATA_TYPE_NAME_CLASS_ID: [u8; 16] = [
    0x2a, 0xb0, 0x0c, 0x96, 0xaf, 0x8c, 0x2b, 0x44, 0xac, 0xf7, 0x6b, 0x62, 0x56, 0xfc, 0x19, 0xf9,
];

#[derive(Debug, Clone, PartialEq)]
pub struct GpDataTypeName10_1 {
    pub version: i16,
    pub base_version: i16,
    pub strings: [String; 4],
    pub legacy_string: Option<String>,
    pub base_uid: Option<RuleSlot10_1<UidBody10_1>>,
    pub legacy_uid: Option<RuleSlot10_1<UidBody10_1>>,
}

pub const LOCATOR_NAME_CLASS_ID: [u8; 16] = [
    0x64, 0xfe, 0x4f, 0xc2, 0xbe, 0x88, 0xd3, 0x11, 0x9f, 0x66, 0x00, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];

pub const LOCATOR_WORKSPACE_NAME_CLASS_ID: [u8; 16] = [
    0xf0, 0xca, 0x85, 0x96, 0x13, 0xbf, 0x83, 0x42, 0xb3, 0x88, 0xa5, 0x73, 0x03, 0x52, 0xc7, 0x78,
];

pub const SHORTCUT_NAME_CLASS_ID: [u8; 16] = [
    0xe4, 0x09, 0xeb, 0x3b, 0x41, 0x39, 0x07, 0x4a, 0x9d, 0x1a, 0xec, 0x2b, 0x43, 0xba, 0x7d, 0x50,
];

pub const SCHEMATIC_WORKSPACE_NAME_CLASS_ID: [u8; 16] = [
    0x72, 0xe4, 0x54, 0xaf, 0x8a, 0x0a, 0xda, 0x4c, 0xa9, 0xed, 0xb3, 0x50, 0xa0, 0xba, 0x9e, 0xbb,
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
    SchematicDatasetName(Box<SchematicDatasetNameBody10_1>),
    /// 与 SchematicDatasetName 共用持久化字段布局。
    SchematicWorkspaceName(Box<SchematicDatasetNameBody10_1>),
    SchematicFolderName(Box<SchematicFolderNameBody10_1>),
    SchematicDiagramName(Box<SchematicDiagramNameBody10_1>),
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
    GpDataTypeName(Box<GpDataTypeName10_1>),
    GpFunctionName(Box<GpFunctionName10_1>),
    GpSaTopoFeaturesTypeName(Box<GpSaTopoFeaturesTypeName10_1>),
    GpValueTableTypeName {
        version: i16,
        raw_column_count: i32,
        columns: Vec<GpValueTableColumnType10_1>,
    },
    GpMultiValueTypeName {
        version: i16,
        element_type: Box<BookmarkNameEntry10_1>,
    },
    LocatorName {
        version: i16,
        strings: [String; 4],
        source: Box<BookmarkNameEntry10_1>,
    },
    LocatorWorkspaceProperties(Box<PropertySetObjectStream10_1>),
    LocatorWorkspaceUid(UidBody10_1),
    LocatorWorkspaceName {
        version: i16,
        strings: [String; 2],
        source: Box<BookmarkNameEntry10_1>,
    },
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
    RouteMeasureLocatorName(Box<RouteMeasureLocatorName10_1>),
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
    let envelope = read_surround_envelope_slot(
        &mut trial,
        max_bytes,
        &mut trial_table,
        "AOIBookmark 位置包络槽",
    )?;

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

fn read_surround_envelope_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    slot: &str,
) -> io::Result<BookmarkEnvelopeEntry10_1> {
    let envelope_offset = reader.position();
    Ok(match read_object_entry_10_1(reader, table)? {
        ObjectEntry10_1::Null => BookmarkEnvelopeEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = table.registered_class(key);
            if registered.is_some_and(|class| class != ENVELOPE_CLASS_ID) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("{slot} offset 0x{envelope_offset:x}: 回引键 {key} 的登记类不受支持"),
                ));
            }
            BookmarkEnvelopeEntry10_1::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == ENVELOPE_CLASS_ID => {
            // Envelope 全部版本分支已闭合：整体消费对象体（含嵌套空间参考）。
            let body = read_envelope_10_1(reader, max_bytes, table)?;
            BookmarkEnvelopeEntry10_1::Envelope { key, body }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "{slot} offset 0x{envelope_offset:x}: 类标识 {}（键 \
                     {key}）不是 Envelope；本读取器仅支持 Envelope 新对象嵌套，且对象体\
                     无外部长度字段，无法跳过未知类继续读取",
                    class_id_hex(&class_id)
                ),
            ));
        }
    })
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

fn read_gp_uid_slot(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<UidBody10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::NewObject {
            class_id: UID_CLASS_ID,
            key,
        } => Some(Box::new(read_uid_body_10_1(reader, key)?)),
        ObjectEntry10_1::BackReference { class_id, key }
            if table.registered_class(key).unwrap_or(class_id) == UID_CLASS_ID =>
        {
            None
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GP 对象的 IUID 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_name_base_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GpNameBase10_1> {
    let base_version = read_gp_name_version(reader, 3, "GPName")?;
    let strings = [
        read_bstr_string_10_1(reader, max_bytes)?,
        read_bstr_string_10_1(reader, max_bytes)?,
        read_bstr_string_10_1(reader, max_bytes)?,
        read_bstr_string_10_1(reader, max_bytes)?,
    ];
    let base_uid = (base_version >= 3)
        .then(|| read_gp_uid_slot(reader, table))
        .transpose()?;
    let legacy_string = (base_version < 2)
        .then(|| read_bstr_string_10_1(reader, max_bytes))
        .transpose()?;
    Ok(GpNameBase10_1 {
        version: base_version,
        strings,
        legacy_string,
        uid: base_uid,
    })
}

fn read_gp_name_version(reader: &mut Reader<'_>, max: i16, class: &str) -> io::Result<i16> {
    let version = reader.read_u16_le()? as i16;
    if version > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{class} 版本 {version} 超出 10.1 上限"),
        ));
    }
    Ok(version)
}

fn read_gp_data_type_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GpDataTypeName10_1> {
    let version = read_gp_name_version(reader, 2, "GPDataTypeName")?;
    let base = read_gp_name_base_10_1(reader, max_bytes, table)?;
    let legacy_uid = (version < 2)
        .then(|| read_gp_uid_slot(reader, table))
        .transpose()?;
    Ok(GpDataTypeName10_1 {
        version,
        base_version: base.version,
        strings: base.strings,
        legacy_string: base.legacy_string,
        base_uid: base.uid,
        legacy_uid,
    })
}

fn read_gp_function_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GpFunctionName10_1> {
    let version = read_gp_name_version(reader, 4, "GPFunctionName")?;
    let base = read_gp_name_base_10_1(reader, max_bytes, table)?;
    let legacy_uid = (version < 2)
        .then(|| read_gp_uid_slot(reader, table))
        .transpose()?;
    let v3_fields = (version >= 3)
        .then(|| -> io::Result<_> {
            Ok((
                reader.read_u32_le()? as i32,
                read_bstr_string_10_1(reader, max_bytes)?,
                reader.read_u32_le()? as i32,
            ))
        })
        .transpose()?;
    let v4_value = (version >= 4)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    Ok(GpFunctionName10_1 {
        version,
        base,
        legacy_uid,
        v3_fields,
        v4_value,
    })
}

fn read_gp_domain_longs(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    class: &str,
) -> io::Result<(i32, Vec<i32>)> {
    let raw_count = reader.read_u32_le()? as i32;
    let count = raw_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{class} 条目数量超限"),
        ));
    }
    let values = (0..count)
        .map(|_| reader.read_u32_le().map(|value| value as i32))
        .collect::<io::Result<Vec<_>>>()?;
    Ok((raw_count, values))
}

fn read_gp_domain_strings(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    class: &str,
) -> io::Result<(i32, Vec<String>)> {
    let raw_count = reader.read_u32_le()? as i32;
    let count = raw_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / 6 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{class} 字符串数量超限"),
        ));
    }
    let strings = (0..count)
        .map(|_| read_string_10_1(reader, max_bytes))
        .collect::<io::Result<Vec<_>>>()?;
    Ok((raw_count, strings))
}

fn read_gp_coverage_feature_class_domain(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<GpCoverageFeatureClassDomain10_1> {
    let version = read_gp_name_version(reader, 2, "GPCoverageFeatureClassDomain")?;
    let long_list = read_gp_domain_longs(reader, max_bytes, "GPCoverageFeatureClassDomain")?;
    let v2_tail = (version == 2)
        .then(|| -> io::Result<_> { Ok((reader.read_u16_le()?, reader.read_u32_le()? as i32)) })
        .transpose()?;
    Ok(GpCoverageFeatureClassDomain10_1 {
        version,
        long_list,
        v2_tail,
    })
}

fn read_gp_coded_value_domain(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<GpCodedValueDomain10_1> {
    let version = read_gp_name_version(reader, 1, "GPCodedValueDomain")?;
    let labels = read_gp_domain_strings(reader, max_bytes, "GPCodedValueDomain")?;
    let (raw_value_count, values) = read_gp_coded_value_list(reader, max_bytes, table, depth)?;
    Ok(GpCodedValueDomain10_1 {
        version,
        labels,
        raw_value_count,
        values,
    })
}

fn read_gp_coded_value_list(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(i32, Vec<RuleSlot10_1<GpCodedValueBody10_1>>)> {
    let (raw_value_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut values = Vec::new();
    for _ in 0..count {
        values.push(read_gp_coded_value_slot(reader, max_bytes, table, depth)?);
    }
    Ok((raw_value_count, values))
}

fn read_gp_coded_value_count(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    min_row_bytes: usize,
) -> io::Result<(i32, usize)> {
    let raw_count = reader.read_u32_le()? as i32;
    let count = raw_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / min_row_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "IGPValue 列表数量超限",
        ));
    }
    Ok((raw_count, count))
}

fn read_gp_coded_value_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<GpCodedValueBody10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    read_gp_coded_value_entry(reader, max_bytes, table, depth, entry)
}

fn read_gp_coded_value_entry(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    entry: ObjectEntry10_1,
) -> io::Result<RuleSlot10_1<GpCodedValueBody10_1>> {
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                table.registered_class(key).unwrap_or(class_id),
                GP_STRING_CLASS_ID
                    | GP_TABLE_VIEW_CLASS_ID
                    | GP_RECORD_SET_CLASS_ID
                    | GP_FEATURE_RECORD_SET_LAYER_CLASS_ID
                    | GP_FIELD_MAPPING_CLASS_ID
                    | GP_RASTER_FORMULATED_CLASS_ID
                    | GP_RASTER_BUILDER_CLASS_ID
                    | GP_TERRAIN_LAYER_CLASS_ID
                    | GP_TOPOLOGY_LAYER_CLASS_ID
                    | GP_NETWORK_DATASET_LAYER_CLASS_ID
                    | GP_LAS_DATASET_LAYER_CLASS_ID
                    | GP_RASTER_LAYER_CLASS_ID
                    | GP_TIN_LAYER_CLASS_ID
                    | GP_FEATURE_LAYER_CLASS_ID
                    | GP_RASTER_CATALOG_LAYER_CLASS_ID
                    | GP_MOSAIC_LAYER_CLASS_ID
                    | GP_CADASTRAL_FABRIC_LAYER_CLASS_ID
                    | GP_GROUP_LAYER_CLASS_ID
                    | GP_NA_LAYER_CLASS_ID
                    | GP_RASTER_DATA_LAYER_CLASS_ID
                    | GP_RASTER_DATA_CLASS_ID
                    | GP_POINT_CLASS_ID
                    | GP_REPLICA_CLASS_ID
                    | GP_REPLICA_DESCRIPTION_CLASS_ID
                    | GP_REPLICA_DATASET_CLASS_ID
                    | GP_LINE_CLASS_ID
                    | GP_POLYGON_CLASS_ID
                    | GP_CELL_SIZE_XY_CLASS_ID
                    | GP_SA_CELL_SIZE_CLASS_ID
                    | GP_WORKSPACE_EXTENSION_CLASS_ID
                    | GP_DATA_FILE_CLASS_ID
                    | GP_ADDRESS_LOCATOR_STYLE_CLASS_ID
                    | GP_FIELD_INFO_EX_CLASS_ID
                    | GP_FIELD_INFO_CLASS_ID
                    | GP_3DA_DECIMATE_CLASS_ID
                    | GP_3DA_INTERPOLATE_CLASS_ID
                    | GP_NA_HIERARCHY_SETTINGS_CLASS_ID
                    | NA_CLASS_FIELD_MAP_CLASS_ID
                    | GP_ARC_INFO_ITEM_CLASS_ID
                    | GP_ANALYSIS_CELL_SIZE_CLASS_ID
                    | GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID
                    | GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID
                    | GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID
                    | GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID
                    | GP_RASTER_CALCULATOR_EXPRESSION_CLASS_ID
                    | GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID
                    | GP_SA_WEIGHTED_SUM_CLASS_ID
                    | GP_SA_EXTRACT_VALUES_CLASS_ID
                    | GP_SA_FUZZY_FUNCTION_CLASS_ID
                    | GP_SA_REMAP_CLASS_ID
                    | GP_SA_NUMBER_REMAP_CLASS_ID
                    | GP_SA_HORIZONTAL_FACTOR_CLASS_ID
                    | GP_SA_VERTICAL_FACTOR_CLASS_ID
                    | GP_SA_NEIGHBORHOOD_CLASS_ID
                    | GP_SA_TIME_CONFIGURATION_CLASS_ID
                    | GP_SA_SEMI_VARIOGRAM_CLASS_ID
                    | GP_SA_STRING_REMAP_CLASS_ID
                    | GP_SA_RADIUS_CLASS_ID
                    | GP_EVALUATION_SCALE_CLASS_ID
                    | GP_SA_MAP_ALGEBRA_EXP_CLASS_ID
                    | GP_ENVELOPE_CLASS_ID
                    | GP_EXTENT_CLASS_ID
                    | GP_SPATIAL_REFERENCE_CLASS_ID
                    | GP_COORDINATE_SYSTEM_CLASS_ID
                    | GP_XY_DOMAIN_CLASS_ID
                    | GP_Z_DOMAIN_CLASS_ID
                    | GP_M_DOMAIN_CLASS_ID
                    | GP_SQL_EXPRESSION_CLASS_ID
                    | GP_INFO_EXPRESSION_CLASS_ID
                    | GP_CALCULATOR_EXPRESSION_CLASS_ID
                    | GP_ROUTE_MEASURE_EVENT_PROPERTIES_CLASS_ID
                    | GP_ENCRYPTED_STRING_CLASS_ID
                    | GP_LONG_CLASS_ID
                    | GP_DOUBLE_CLASS_ID
                    | FIELD_CLASS_ID
                    | XML_INDEX_CLASS_ID
                    | XML_INDEX_TAGS_CLASS_ID
                    | XML_INDEX_TAG_CLASS_ID
                    | GP_BOOLEAN_CLASS_ID
                    | FME_DATASET_CLASS_ID
                    | GP_DATE_CLASS_ID
                    | GP_LINEAR_UNIT_CLASS_ID
                    | GP_AREAL_UNIT_CLASS_ID
                    | GP_VARIANT_CLASS_ID
                    | GP_MULTI_VALUE_CLASS_ID
                    | GP_SA_TOPO_FEATURES_CLASS_ID
                    | GP_VALUE_TABLE_CLASS_ID
                    | GP_RANDOM_NUMBER_GENERATOR_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (GP_TIN_LAYER_CLASS_ID
                | GP_TERRAIN_LAYER_CLASS_ID
                | GP_TOPOLOGY_LAYER_CLASS_ID
                | GP_NETWORK_DATASET_LAYER_CLASS_ID
                | GP_LAS_DATASET_LAYER_CLASS_ID
                | GP_RASTER_LAYER_CLASS_ID),
            ..
        } => {
            let layer = read_gp_layer_body(reader, max_bytes, table, depth, class_id)?;
            Some(Box::new(GpCodedValueBody10_1 {
                version: layer.derived_version.expect("GP 图层有派生层版本"),
                base_version: layer.gp_value_version,
                base_short: layer.gp_value_short,
                value: match class_id {
                    GP_TIN_LAYER_CLASS_ID => GpCodedValue10_1::TinLayer(Box::new(layer)),
                    GP_TERRAIN_LAYER_CLASS_ID => GpCodedValue10_1::TerrainLayer(Box::new(layer)),
                    GP_TOPOLOGY_LAYER_CLASS_ID => GpCodedValue10_1::TopologyLayer(Box::new(layer)),
                    GP_NETWORK_DATASET_LAYER_CLASS_ID => {
                        GpCodedValue10_1::NetworkDatasetLayer(Box::new(layer))
                    }
                    GP_LAS_DATASET_LAYER_CLASS_ID => {
                        GpCodedValue10_1::LasDatasetLayer(Box::new(layer))
                    }
                    _ => GpCodedValue10_1::RasterLayer(Box::new(layer)),
                },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (GP_RASTER_CATALOG_LAYER_CLASS_ID | GP_FEATURE_LAYER_CLASS_ID),
            ..
        } => {
            let layer = read_gp_layer_body(reader, max_bytes, table, depth, class_id)?;
            Some(Box::new(GpCodedValueBody10_1 {
                version: layer
                    .feature_layer
                    .as_ref()
                    .expect("GPFeatureLayerBase 有要素图层前缀")
                    .version,
                base_version: layer.gp_value_version,
                base_short: layer.gp_value_short,
                value: if class_id == GP_FEATURE_LAYER_CLASS_ID {
                    GpCodedValue10_1::FeatureLayer(Box::new(layer))
                } else {
                    GpCodedValue10_1::RasterCatalogLayer(Box::new(layer))
                },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (GP_MOSAIC_LAYER_CLASS_ID
                | GP_CADASTRAL_FABRIC_LAYER_CLASS_ID
                | GP_GROUP_LAYER_CLASS_ID
                | GP_NA_LAYER_CLASS_ID),
            ..
        } => {
            let layer = read_gp_layer_body(reader, max_bytes, table, depth, class_id)?;
            Some(Box::new(GpCodedValueBody10_1 {
                version: layer
                    .composite
                    .as_ref()
                    .expect("GP 组合图层有组合层前缀")
                    .version,
                base_version: layer.gp_value_version,
                base_short: layer.gp_value_short,
                value: match class_id {
                    GP_MOSAIC_LAYER_CLASS_ID => GpCodedValue10_1::MosaicLayer(Box::new(layer)),
                    GP_CADASTRAL_FABRIC_LAYER_CLASS_ID => {
                        GpCodedValue10_1::CadastralFabricLayer(Box::new(layer))
                    }
                    GP_GROUP_LAYER_CLASS_ID => GpCodedValue10_1::GroupLayer(Box::new(layer)),
                    _ => GpCodedValue10_1::NaLayer(Box::new(layer)),
                },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_RASTER_FORMULATED_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            let raster_version = reader.read_u16_le()?;
            let base_version = reader.read_u16_le()?;
            if version > 2 || raster_version > 1 || base_version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPRasterFormulated 版本超限",
                ));
            }
            let base_short = reader.read_u16_le()?;
            let raster_long = reader.read_u32_le()?;
            let raster_text = read_string_10_1(reader, max_bytes)?;
            let envelope = read_surround_envelope_slot(
                reader,
                max_bytes,
                table,
                "GPRasterFormulated IEnvelope 槽",
            )?;
            let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, table)?;
            let raw_long = reader.read_u32_le()?;
            let value = if version < 2 {
                let mut raw = [0; 24];
                raw.copy_from_slice(reader.read_bytes(24)?);
                GpRasterFormulatedValue10_1::Legacy(raw)
            } else {
                GpRasterFormulatedValue10_1::Variant(read_variant_object_stream_value_10_1(
                    reader, max_bytes, table, depth,
                )?)
            };
            Some(Box::new(GpCodedValueBody10_1 {
                version,
                base_version,
                base_short,
                value: GpCodedValue10_1::RasterFormulated(Box::new(GpRasterFormulated10_1 {
                    raster_version,
                    raster_long,
                    raster_text,
                    envelope,
                    spatial_reference,
                    raw_long,
                    value,
                })),
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_REPLICA_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 2 {
                return Err(io::Error::new(ErrorKind::InvalidData, "GPReplica 版本超限"));
            }
            let initial_long = reader.read_u32_le()?;
            let strings = [
                read_bstr_string_10_1(reader, max_bytes)?,
                read_bstr_string_10_1(reader, max_bytes)?,
                read_bstr_string_10_1(reader, max_bytes)?,
                read_bstr_string_10_1(reader, max_bytes)?,
            ];
            let variant = read_variant_object_stream_value_10_1(reader, max_bytes, table, depth)?;
            let long_before_description = reader.read_u32_le()?;
            let description = read_gp_coded_value_slot_of_class(
                reader,
                max_bytes,
                table,
                depth,
                GP_REPLICA_DESCRIPTION_CLASS_ID,
            )?;
            let mut trailing_longs = [0; 6];
            for entry in &mut trailing_longs {
                *entry = reader.read_u32_le()?;
            }
            let trailing_short = reader.read_u16_le()?;
            let v2 = if version == 2 {
                let mut read_variant =
                    || read_variant_object_stream_value_10_1(reader, max_bytes, table, depth);
                Some(GpReplicaV2Tail10_1 {
                    variants: [read_variant()?, read_variant()?, read_variant()?],
                    raw_shorts: [
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                    ],
                })
            } else {
                None
            };
            Some(Box::new(GpCodedValueBody10_1 {
                version,
                // GPReplica 直接保存自己的字段，没有 GPValue 基类头。
                base_version: 0,
                base_short: 0,
                value: GpCodedValue10_1::Replica(Box::new(GpReplica10_1 {
                    initial_long,
                    strings,
                    variant,
                    long_before_description,
                    description,
                    trailing_longs,
                    trailing_short,
                    v2,
                })),
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_REPLICA_DESCRIPTION_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPReplicaDescription 版本超限",
                ));
            }
            let first = reader.read_u32_le()?;
            let flag = reader.read_u16_le()?;
            let second = reader.read_u32_le()?;
            let geometry = read_gp_replica_geometry_slot(reader, max_bytes, table)?;
            let datasets = read_gp_replica_datasets_slot(reader, max_bytes, table, depth)?;
            let tail = reader.read_u16_le()?;
            Some(Box::new(GpCodedValueBody10_1 {
                version,
                base_version: 0,
                base_short: 0,
                value: GpCodedValue10_1::ReplicaDescription {
                    raw_longs: [first, second],
                    raw_shorts: [flag, tail],
                    geometry,
                    datasets,
                },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_REPLICA_DATASET_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 3 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPReplicaDataset 版本超限",
                ));
            }
            let first = read_bstr_string_10_1(reader, max_bytes)?;
            let raw_longs = [reader.read_u32_le()?, reader.read_u32_le()?];
            let raw_shorts = [reader.read_u16_le()?, reader.read_u16_le()?];
            let second = read_bstr_string_10_1(reader, max_bytes)?;
            let type_11 = if raw_longs[0] == 11 {
                let raw = reader.read_u32_le()?;
                let strings = if version > 1 {
                    Some([
                        read_bstr_string_10_1(reader, max_bytes)?,
                        read_bstr_string_10_1(reader, max_bytes)?,
                    ])
                } else {
                    None
                };
                Some((raw, strings))
            } else {
                None
            };
            let indices =
                crate::layers::raster_dataset_name::read_long_array_slot(reader, max_bytes, table)?;
            let v3 = if version == 3 {
                Some((
                    read_bstr_string_10_1(reader, max_bytes)?,
                    reader.read_u32_le()?,
                    [
                        read_bstr_string_10_1(reader, max_bytes)?,
                        read_bstr_string_10_1(reader, max_bytes)?,
                    ],
                    reader.read_u32_le()?,
                ))
            } else {
                None
            };
            Some(Box::new(GpCodedValueBody10_1 {
                version,
                // 该类直接保存自己的字段，没有 GPValue 基类头。
                base_version: 0,
                base_short: 0,
                value: GpCodedValue10_1::ReplicaDataset {
                    strings: [first, second],
                    raw_longs,
                    raw_shorts,
                    type_11,
                    indices,
                    v3,
                },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_LINE_CLASS_ID | GP_POLYGON_CLASS_ID,
            ..
        } => {
            // 原生 ObjectStream 写出基类字段的位置与原生 Load 不同。
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPLine/GPPolygon 版本超限",
                ));
            }
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
            let mut points = Vec::new();
            for _ in 0..count {
                points.push(read_gp_coded_value_slot_of_class(
                    reader,
                    max_bytes,
                    table,
                    depth,
                    GP_POINT_CLASS_ID,
                )?);
            }
            let base_version = reader.read_u16_le()?;
            if base_version > 1 {
                return Err(io::Error::new(ErrorKind::InvalidData, "GPValue 版本超限"));
            }
            let base_short = reader.read_u16_le()?;
            Some(Box::new(GpCodedValueBody10_1 {
                version,
                base_version,
                base_short,
                value: GpCodedValue10_1::LineOrPolygon { raw_count, points },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: FIELD_CLASS_ID,
            ..
        } => {
            let field = read_field_body_10_1(reader, max_bytes, table, depth)?;
            Some(Box::new(GpCodedValueBody10_1 {
                version: field.version,
                base_version: field.gp_value_version,
                base_short: field.gp_value_short,
                value: GpCodedValue10_1::Field(Box::new(field)),
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: FME_DATASET_CLASS_ID,
            ..
        } => Some(Box::new(GpCodedValueBody10_1 {
            // DIToolbox 对象体无版本头，三个公共字段没有流上值。
            version: 0,
            base_version: 0,
            base_short: 0,
            value: GpCodedValue10_1::FmeDataset(crate::stream::native_string::read_atl_bstr_10_1(
                reader, max_bytes,
            )?),
        })),
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (GP_STRING_CLASS_ID
                | GP_TABLE_VIEW_CLASS_ID
                | GP_RECORD_SET_CLASS_ID
                | GP_FEATURE_RECORD_SET_LAYER_CLASS_ID
                | GP_FIELD_MAPPING_CLASS_ID
                | GP_RASTER_BUILDER_CLASS_ID
                | GP_RASTER_DATA_LAYER_CLASS_ID
                | GP_RASTER_DATA_CLASS_ID
                | GP_POINT_CLASS_ID
                | GP_CELL_SIZE_XY_CLASS_ID
                | GP_SA_CELL_SIZE_CLASS_ID
                | GP_WORKSPACE_EXTENSION_CLASS_ID
                | GP_DATA_FILE_CLASS_ID
                | GP_ADDRESS_LOCATOR_STYLE_CLASS_ID
                | GP_FIELD_INFO_EX_CLASS_ID
                | GP_FIELD_INFO_CLASS_ID
                | GP_3DA_DECIMATE_CLASS_ID
                | GP_3DA_INTERPOLATE_CLASS_ID
                | GP_NA_HIERARCHY_SETTINGS_CLASS_ID
                | NA_CLASS_FIELD_MAP_CLASS_ID
                | GP_ARC_INFO_ITEM_CLASS_ID
                | GP_ANALYSIS_CELL_SIZE_CLASS_ID
                | GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID
                | GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID
                | GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID
                | GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID
                | GP_RASTER_CALCULATOR_EXPRESSION_CLASS_ID
                | GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID
                | GP_SA_WEIGHTED_SUM_CLASS_ID
                | GP_SA_EXTRACT_VALUES_CLASS_ID
                | GP_SA_FUZZY_FUNCTION_CLASS_ID
                | GP_SA_REMAP_CLASS_ID
                | GP_SA_NUMBER_REMAP_CLASS_ID
                | GP_SA_HORIZONTAL_FACTOR_CLASS_ID
                | GP_SA_VERTICAL_FACTOR_CLASS_ID
                | GP_SA_NEIGHBORHOOD_CLASS_ID
                | GP_SA_TIME_CONFIGURATION_CLASS_ID
                | GP_SA_SEMI_VARIOGRAM_CLASS_ID
                | GP_SA_STRING_REMAP_CLASS_ID
                | GP_SA_RADIUS_CLASS_ID
                | GP_EVALUATION_SCALE_CLASS_ID
                | GP_SA_MAP_ALGEBRA_EXP_CLASS_ID
                | GP_ENVELOPE_CLASS_ID
                | GP_EXTENT_CLASS_ID
                | GP_SPATIAL_REFERENCE_CLASS_ID
                | GP_COORDINATE_SYSTEM_CLASS_ID
                | GP_XY_DOMAIN_CLASS_ID
                | GP_Z_DOMAIN_CLASS_ID
                | GP_M_DOMAIN_CLASS_ID
                | GP_SQL_EXPRESSION_CLASS_ID
                | GP_INFO_EXPRESSION_CLASS_ID
                | GP_CALCULATOR_EXPRESSION_CLASS_ID
                | GP_ROUTE_MEASURE_EVENT_PROPERTIES_CLASS_ID
                | GP_ENCRYPTED_STRING_CLASS_ID
                | GP_LONG_CLASS_ID
                | GP_DOUBLE_CLASS_ID
                | XML_INDEX_CLASS_ID
                | XML_INDEX_TAGS_CLASS_ID
                | XML_INDEX_TAG_CLASS_ID
                | GP_BOOLEAN_CLASS_ID
                | GP_DATE_CLASS_ID
                | GP_LINEAR_UNIT_CLASS_ID
                | GP_AREAL_UNIT_CLASS_ID
                | GP_VARIANT_CLASS_ID
                | GP_MULTI_VALUE_CLASS_ID
                | GP_SA_TOPO_FEATURES_CLASS_ID
                | GP_VALUE_TABLE_CLASS_ID
                | GP_RANDOM_NUMBER_GENERATOR_CLASS_ID),
            ..
        } => {
            let version = reader.read_u16_le()?;
            let base_version = reader.read_u16_le()?;
            let max_version = match class_id {
                GP_FEATURE_RECORD_SET_LAYER_CLASS_ID => 3,
                GP_FIELD_MAPPING_CLASS_ID | GP_TABLE_VIEW_CLASS_ID | GP_RECORD_SET_CLASS_ID => 2,
                GP_SPATIAL_REFERENCE_CLASS_ID
                | GP_SA_SEMI_VARIOGRAM_CLASS_ID
                | GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID => 3,
                GP_SA_REMAP_CLASS_ID => 5,
                GP_SA_NUMBER_REMAP_CLASS_ID
                | GP_SA_STRING_REMAP_CLASS_ID
                | GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID
                | GP_ADDRESS_LOCATOR_STYLE_CLASS_ID => 4,
                GP_ENCRYPTED_STRING_CLASS_ID
                | GP_SA_RADIUS_CLASS_ID
                | GP_EVALUATION_SCALE_CLASS_ID
                | GP_SA_MAP_ALGEBRA_EXP_CLASS_ID
                | GP_SA_NEIGHBORHOOD_CLASS_ID
                | GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID
                | GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID
                | GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID
                | GP_FIELD_INFO_CLASS_ID => 2,
                _ => 1,
            };
            let version_unsupported = if class_id == NA_CLASS_FIELD_MAP_CLASS_ID {
                (version as i16) > 1
            } else if class_id == GP_NA_HIERARCHY_SETTINGS_CLASS_ID {
                version == 0
            } else {
                version > max_version
            };
            if version_unsupported || base_version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GP 编码值对象或 GPValue 版本超出 10.1 上限",
                ));
            }
            let base_short = reader.read_u16_le()?;
            let value = match class_id {
                GP_FEATURE_RECORD_SET_LAYER_CLASS_ID => {
                    GpCodedValue10_1::FeatureRecordSetLayer(Box::new(
                        read_gp_feature_record_set_layer(reader, max_bytes, table, depth, version)?,
                    ))
                }
                GP_RECORD_SET_CLASS_ID => GpCodedValue10_1::RecordSet(Box::new(
                    read_gp_record_set(reader, max_bytes, table, depth, version)?,
                )),
                GP_TABLE_VIEW_CLASS_ID => GpCodedValue10_1::TableView(Box::new(
                    read_gp_table_view(reader, max_bytes, table, depth, version)?,
                )),
                GP_STRING_CLASS_ID => {
                    GpCodedValue10_1::String(read_string_10_1(reader, max_bytes)?)
                }
                GP_ENCRYPTED_STRING_CLASS_ID => {
                    GpCodedValue10_1::EncryptedString(read_string_10_1(reader, max_bytes)?)
                }
                GP_ENVELOPE_CLASS_ID | GP_EXTENT_CLASS_ID => {
                    let raw_mode = (class_id == GP_EXTENT_CLASS_ID)
                        .then(|| reader.read_u32_le().map(|mode| mode as i32))
                        .transpose()?;
                    let envelope =
                        read_surround_envelope_slot(reader, max_bytes, table, "GP IEnvelope 槽")?;
                    GpCodedValue10_1::Envelope {
                        envelope: Box::new(envelope),
                        raw_mode,
                    }
                }
                GP_SPATIAL_REFERENCE_CLASS_ID
                | GP_COORDINATE_SYSTEM_CLASS_ID
                | GP_XY_DOMAIN_CLASS_ID
                | GP_Z_DOMAIN_CLASS_ID
                | GP_M_DOMAIN_CLASS_ID => {
                    let reference = read_spatial_reference_entry_10_1(reader, max_bytes, table)?;
                    let legacy_short = (class_id == GP_SPATIAL_REFERENCE_CLASS_ID && version == 2)
                        .then(|| reader.read_u16_le())
                        .transpose()?;
                    GpCodedValue10_1::SpatialReference {
                        reference: Box::new(reference),
                        legacy_short,
                    }
                }
                GP_SQL_EXPRESSION_CLASS_ID => {
                    GpCodedValue10_1::SqlExpression(read_string_10_1(reader, max_bytes)?)
                }
                GP_INFO_EXPRESSION_CLASS_ID => {
                    GpCodedValue10_1::InfoExpression(read_string_10_1(reader, max_bytes)?)
                }
                GP_CALCULATOR_EXPRESSION_CLASS_ID => {
                    GpCodedValue10_1::CalculatorExpression(read_string_10_1(reader, max_bytes)?)
                }
                GP_ROUTE_MEASURE_EVENT_PROPERTIES_CLASS_ID => {
                    GpCodedValue10_1::RouteMeasureEventProperties([
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ])
                }
                GP_DOUBLE_CLASS_ID => GpCodedValue10_1::DoubleBits(reader.read_f64_le()?.to_bits()),
                GP_FIELD_MAPPING_CLASS_ID => {
                    let (raw_map_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
                    let mut maps = Vec::new();
                    for _ in 0..count {
                        maps.push(read_gp_field_map_slot(reader, max_bytes, table, depth)?);
                    }
                    let (raw_table_count, count) =
                        read_gp_coded_value_count(reader, max_bytes, 16)?;
                    let mut tables = Vec::new();
                    for _ in 0..count {
                        tables.push(read_gp_de_table_slot(reader, max_bytes, table, depth)?);
                    }
                    let tail = (version >= 2)
                        .then(|| read_gp_de_table_slot(reader, max_bytes, table, depth))
                        .transpose()?;
                    GpCodedValue10_1::FieldMapping {
                        raw_map_count,
                        maps,
                        raw_table_count,
                        tables,
                        tail,
                    }
                }
                GP_RASTER_BUILDER_CLASS_ID => GpCodedValue10_1::RasterBuilder(
                    read_raster_type_slot_10_1(reader, max_bytes, table, depth)?,
                ),
                GP_RASTER_DATA_LAYER_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPRasterDataLayer 名称嵌套过深",
                        ));
                    }
                    GpCodedValue10_1::RasterDataLayer {
                        name: Box::new(read_bookmark_name_slot_10_1(
                            reader,
                            max_bytes,
                            table,
                            depth + 1,
                        )?),
                        text: read_string_10_1(reader, max_bytes)?,
                        drawing: read_layer_drawing_description_slot(reader, max_bytes, table)?,
                    }
                }
                GP_RASTER_DATA_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPRasterData 名称嵌套过深",
                        ));
                    }
                    GpCodedValue10_1::RasterData {
                        name: Box::new(read_bookmark_name_slot_10_1(
                            reader,
                            max_bytes,
                            table,
                            depth + 1,
                        )?),
                        text: read_string_10_1(reader, max_bytes)?,
                    }
                }
                GP_POINT_CLASS_ID => GpCodedValue10_1::Point {
                    x_bits: reader.read_f64_le()?.to_bits(),
                    y_bits: reader.read_f64_le()?.to_bits(),
                },
                GP_SA_RADIUS_CLASS_ID
                | GP_EVALUATION_SCALE_CLASS_ID
                | GP_SA_MAP_ALGEBRA_EXP_CLASS_ID => {
                    let legacy_name =
                        read_gp_legacy_name(reader, max_bytes, table, depth, version == 1)?;
                    match class_id {
                        GP_SA_RADIUS_CLASS_ID => GpCodedValue10_1::SaRadius {
                            legacy_name,
                            raw_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
                            distance_bits: reader.read_f64_le()?.to_bits(),
                        },
                        GP_EVALUATION_SCALE_CLASS_ID => GpCodedValue10_1::EvaluationScale {
                            legacy_name,
                            raw_longs: [
                                reader.read_u32_le()? as i32,
                                reader.read_u32_le()? as i32,
                                reader.read_u32_le()? as i32,
                            ],
                        },
                        _ => GpCodedValue10_1::SaMapAlgebraExpression {
                            legacy_name,
                            expression: read_bstr_string_10_1(reader, max_bytes)?,
                        },
                    }
                }
                GP_SA_HORIZONTAL_FACTOR_CLASS_ID | GP_SA_VERTICAL_FACTOR_CLASS_ID => {
                    GpCodedValue10_1::SaDistanceFactor(read_path_distance_factor_slot(
                        reader,
                        max_bytes,
                        table,
                        class_id == GP_SA_VERTICAL_FACTOR_CLASS_ID,
                    )?)
                }
                GP_SA_TIME_CONFIGURATION_CLASS_ID => GpCodedValue10_1::SaTimeConfiguration(
                    read_gp_sa_time_configuration(reader, table)?,
                ),
                GP_SA_SEMI_VARIOGRAM_CLASS_ID => GpCodedValue10_1::SaSemiVariogram(Box::new(
                    read_gp_sa_semi_variogram(reader, max_bytes, table, depth, version)?,
                )),
                GP_SA_NEIGHBORHOOD_CLASS_ID => GpCodedValue10_1::SaNeighborhood(Box::new(
                    read_gp_sa_neighborhood(reader, max_bytes, table, depth, version)?,
                )),
                GP_SA_REMAP_CLASS_ID
                | GP_SA_NUMBER_REMAP_CLASS_ID
                | GP_SA_STRING_REMAP_CLASS_ID => GpCodedValue10_1::SaRemap(Box::new(
                    read_gp_sa_remap(reader, max_bytes, table, depth, version, class_id)?,
                )),
                GP_WORKSPACE_EXTENSION_CLASS_ID => GpCodedValue10_1::WorkspaceExtension {
                    name: read_string_10_1(reader, max_bytes)?,
                    extension_guid: reader.read_bytes(16)?.try_into().unwrap(),
                },
                GP_DATA_FILE_CLASS_ID => GpCodedValue10_1::DataFile {
                    data: read_gp_data_file_transport(reader, max_bytes, table)?,
                    strings: [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ],
                },
                GP_3DA_DECIMATE_CLASS_ID => GpCodedValue10_1::Decimate3da {
                    raw_leading_long: reader.read_u32_le()? as i32,
                    value_bits: reader.read_f64_le()?.to_bits(),
                    raw_trailing_long: reader.read_u32_le()? as i32,
                },
                GP_3DA_INTERPOLATE_CLASS_ID => {
                    let mut raw_longs = [0; 6];
                    for value in &mut raw_longs {
                        *value = reader.read_u32_le()? as i32;
                    }
                    GpCodedValue10_1::Interpolate3da {
                        raw_longs,
                        value_bits: reader.read_f64_le()?.to_bits(),
                    }
                }
                GP_NA_HIERARCHY_SETTINGS_CLASS_ID => GpCodedValue10_1::NaHierarchySettings {
                    raw_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
                },
                NA_CLASS_FIELD_MAP_CLASS_ID => {
                    let raw_count = reader.read_u32_le()? as i32;
                    let count = raw_count.max(0) as usize;
                    if count > max_bytes.min(reader.remaining()) / 24 {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "NAClassFieldMap 条目数量超限",
                        ));
                    }
                    let mut rows = Vec::new();
                    for _ in 0..count {
                        rows.push(NaClassFieldMapRow10_1 {
                            strings: [
                                read_bstr_string_10_1(reader, max_bytes)?,
                                read_bstr_string_10_1(reader, max_bytes)?,
                            ],
                            value: read_padded_variant_object_stream_nested_10_1(
                                reader, max_bytes, table, depth,
                            )?,
                        });
                    }
                    GpCodedValue10_1::NaClassFieldMap { raw_count, rows }
                }
                GP_ADDRESS_LOCATOR_STYLE_CLASS_ID => {
                    let strings = [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ];
                    let raw_flag = reader.read_u16_le()?;
                    let configuration = (raw_flag != 0)
                        .then(|| {
                            read_gp_address_locator_style_config(
                                reader, max_bytes, table, depth, version, raw_flag,
                            )
                            .map(Box::new)
                        })
                        .transpose()?;
                    GpCodedValue10_1::AddressLocatorStyle {
                        strings,
                        configuration,
                    }
                }
                GP_FIELD_INFO_EX_CLASS_ID => {
                    GpCodedValue10_1::FieldInfoEx(read_string_10_1(reader, max_bytes)?)
                }
                GP_FIELD_INFO_CLASS_ID => read_gp_field_info(reader, max_bytes, version)?,
                GP_ARC_INFO_ITEM_CLASS_ID => GpCodedValue10_1::ArcInfoItem {
                    strings: [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ],
                    raw_longs: [
                        reader.read_u32_le()? as i32,
                        reader.read_u32_le()? as i32,
                        reader.read_u32_le()? as i32,
                        reader.read_u32_le()? as i32,
                        reader.read_u32_le()? as i32,
                    ],
                    raw_shorts: [
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                    ],
                },
                GP_ANALYSIS_CELL_SIZE_CLASS_ID => GpCodedValue10_1::AnalysisCellSize {
                    size_bits: reader.read_f64_le()?.to_bits(),
                    raw_text: read_string_10_1(reader, max_bytes)?,
                },
                GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID
                | GP_RASTER_GDB_ENV_PYRAMID_CLASS_ID
                | GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID
                | GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID => {
                    GpCodedValue10_1::RasterEnvironment(Box::new(read_gp_raster_environment(
                        reader, max_bytes, table, depth, version, class_id,
                    )?))
                }
                GP_RASTER_CALCULATOR_EXPRESSION_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPRasterCalculatorExpression 嵌套层数超限",
                        ));
                    }
                    let expression = read_string_10_1(reader, max_bytes)?;
                    let (raw_count, inputs) =
                        read_gp_coded_value_list(reader, max_bytes, table, depth + 1)?;
                    GpCodedValue10_1::RasterCalculatorExpression {
                        expression,
                        raw_count,
                        inputs,
                    }
                }
                GP_SA_EXTRACT_VALUES_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPSAExtractValues 嵌套层数超限",
                        ));
                    }
                    let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 32)?;
                    let mut rows = Vec::new();
                    for _ in 0..count {
                        rows.push([
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                        ]);
                    }
                    GpCodedValue10_1::SaExtractValues { raw_count, rows }
                }
                GP_SA_WEIGHTED_OVERLAY_TABLE_CLASS_ID => {
                    GpCodedValue10_1::SaWeightedOverlayTable(Box::new(read_gp_sa_weighted_overlay(
                        reader, max_bytes, table, depth, version,
                    )?))
                }
                GP_SA_WEIGHTED_SUM_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPSAWeightedSum 嵌套层数超限",
                        ));
                    }
                    let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 48)?;
                    let mut rows = Vec::new();
                    for _ in 0..count {
                        rows.push([
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                        ]);
                    }
                    GpCodedValue10_1::SaWeightedSum { raw_count, rows }
                }
                GP_SA_FUZZY_FUNCTION_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPSAFuzzyFunction 嵌套层数超限",
                        ));
                    }
                    GpCodedValue10_1::SaFuzzyFunction {
                        arguments: [
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                            read_gp_coded_value_slot(reader, max_bytes, table, depth + 1)?,
                        ],
                        raw_flags: reader.read_bytes(2)?.try_into().unwrap(),
                    }
                }
                GP_SA_CELL_SIZE_CLASS_ID => GpCodedValue10_1::SaCellSize {
                    raw_mode: reader.read_u32_le()? as i32,
                    x_bits: reader.read_f64_le()?.to_bits(),
                    y_bits: reader.read_f64_le()?.to_bits(),
                },
                GP_CELL_SIZE_XY_CLASS_ID => GpCodedValue10_1::CellSizeXy {
                    x_bits: reader.read_f64_le()?.to_bits(),
                    y_bits: reader.read_f64_le()?.to_bits(),
                },
                XML_INDEX_CLASS_ID => GpCodedValue10_1::XmlIndex {
                    first: read_bstr_string_10_1(reader, max_bytes)?,
                    fields: read_gp_fields_slot(reader, max_bytes, table, depth)?,
                    raw_long: reader.read_u32_le()?,
                    tags: Box::new([
                        read_gp_coded_value_slot_of_class(
                            reader,
                            max_bytes,
                            table,
                            depth,
                            XML_INDEX_TAGS_CLASS_ID,
                        )?,
                        read_gp_coded_value_slot_of_class(
                            reader,
                            max_bytes,
                            table,
                            depth,
                            XML_INDEX_TAGS_CLASS_ID,
                        )?,
                    ]),
                    second: read_bstr_string_10_1(reader, max_bytes)?,
                },
                XML_INDEX_TAGS_CLASS_ID => {
                    let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
                    let mut tags = Vec::new();
                    for _ in 0..count {
                        tags.push(read_gp_coded_value_slot_of_class(
                            reader,
                            max_bytes,
                            table,
                            depth,
                            XML_INDEX_TAG_CLASS_ID,
                        )?);
                    }
                    GpCodedValue10_1::XmlIndexTags { raw_count, tags }
                }
                XML_INDEX_TAG_CLASS_ID => GpCodedValue10_1::XmlIndexTag {
                    first: read_bstr_string_10_1(reader, max_bytes)?,
                    raw_longs: [reader.read_u32_le()?, reader.read_u32_le()?],
                    second: read_bstr_string_10_1(reader, max_bytes)?,
                },
                GP_BOOLEAN_CLASS_ID => GpCodedValue10_1::Boolean(reader.read_u16_le()?),
                GP_DATE_CLASS_ID => GpCodedValue10_1::Date(Box::new(
                    read_padded_variant_object_stream_nested_10_1(reader, max_bytes, table, depth)?,
                )),
                GP_LINEAR_UNIT_CLASS_ID => GpCodedValue10_1::LinearUnit {
                    value_bits: reader.read_f64_le()?.to_bits(),
                    unit: reader.read_u32_le()? as i32,
                },
                GP_AREAL_UNIT_CLASS_ID => GpCodedValue10_1::ArealUnit {
                    value_bits: reader.read_f64_le()?.to_bits(),
                    unit: reader.read_u32_le()? as i32,
                },
                GP_VARIANT_CLASS_ID => GpCodedValue10_1::Variant(Box::new(
                    read_padded_variant_object_stream_nested_10_1(reader, max_bytes, table, depth)?,
                )),
                GP_RANDOM_NUMBER_GENERATOR_CLASS_ID => GpCodedValue10_1::RandomNumberGenerator {
                    seed: reader.read_u32_le()? as i32,
                    raw_generator: reader.read_u32_le()? as i32,
                },
                GP_MULTI_VALUE_CLASS_ID => {
                    if depth >= MAX_BOOKMARK_NAME_NESTING {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "GPMultiValue 嵌套层数超限",
                        ));
                    }
                    let type_name =
                        read_bookmark_name_slot_10_1(reader, max_bytes, table, depth + 1)?;
                    // Load 要求 IName。Open 返回对象的接口查询不影响后续流读取。
                    let (raw_count, values) =
                        read_gp_coded_value_list(reader, max_bytes, table, depth + 1)?;
                    GpCodedValue10_1::MultiValue {
                        type_name: Box::new(type_name),
                        raw_count,
                        values,
                    }
                }
                GP_SA_TOPO_FEATURES_CLASS_ID | GP_VALUE_TABLE_CLASS_ID => {
                    read_gp_value_table(reader, max_bytes, table, depth)?
                }
                _ => GpCodedValue10_1::Long(reader.read_u32_le()? as i32),
            };
            Some(Box::new(GpCodedValueBody10_1 {
                version,
                base_version,
                base_short,
                value,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPCodedValueDomain 的 IGPValue 类未支持",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_replica_datasets_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<GpReplicaDatasets10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == GP_REPLICA_DATASETS_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_REPLICA_DATASETS_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPReplicaDatasets")?;
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 20)?;
            let mut datasets = Vec::new();
            for _ in 0..count {
                let slot = read_gp_coded_value_slot_of_class(
                    reader,
                    max_bytes,
                    refs,
                    depth,
                    GP_REPLICA_DATASET_CLASS_ID,
                )?;
                if matches!(slot.entry, ObjectEntry10_1::Null) {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "GPReplicaDatasets 包含空元素",
                    ));
                }
                datasets.push(slot);
            }
            Some(Box::new(GpReplicaDatasets10_1 {
                version,
                raw_count,
                datasets,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的 IGPReplicaDatasets 类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_replica_geometry_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<crate::objects::geometry::GeometryBody10_1>> {
    use crate::objects::geometry::{
        GEOMETRY_BAG_CLASS_ID, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID, POINT_CLASS_ID,
        POLYGON_CLASS_ID, POLYLINE_CLASS_ID, read_keyed_geometry_body_10_1,
    };

    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                refs.registered_class(key).unwrap_or(class_id),
                ENVELOPE_CLASS_ID
                    | GEOMETRY_BAG_CLASS_ID
                    | MULTIPATCH_CLASS_ID
                    | MULTIPOINT_CLASS_ID
                    | POINT_CLASS_ID
                    | POLYGON_CLASS_ID
                    | POLYLINE_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, key } => Some(Box::new(
            read_keyed_geometry_body_10_1(&class_id, key, reader, max_bytes, refs)?,
        )),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的复制描述几何回引",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_layer_drawing_description_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<LayerDrawingDescription10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                refs.registered_class(key).unwrap_or(class_id),
                RASTER_LAYER_DRAWING_DESCRIPTION_CLASS_ID | FEATURE_LAYER_DRAWING_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: RASTER_LAYER_DRAWING_DESCRIPTION_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()? as i16;
            if version > 2 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "RasterLayerDrawingDescription 版本超限",
                ));
            }
            let renderer = crate::layers::raster_renderer::read_raster_renderer_slot_prefix_10_1(
                reader, max_bytes, refs,
            )?;
            if renderer
                .body
                .as_ref()
                .is_some_and(|body| !body.is_complete())
            {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "绘制描述中的栅格渲染器尚未读完",
                ));
            }
            let raw_shorts = if version == 2 {
                Some([
                    reader.read_u16_le()?,
                    reader.read_u16_le()?,
                    reader.read_u16_le()?,
                ])
            } else {
                None
            };
            Some(Box::new(LayerDrawingDescription10_1::Raster(
                RasterLayerDrawingDescription10_1 {
                    version,
                    renderer,
                    raw_shorts,
                },
            )))
        }
        ObjectEntry10_1::NewObject {
            class_id: FEATURE_LAYER_DRAWING_CLASS_ID,
            ..
        } => Some(Box::new(LayerDrawingDescription10_1::Feature(
            read_feature_layer_drawing_description(reader, max_bytes, refs)?,
        ))),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的 ILayerDrawingDescription 类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_feature_layer_drawing_description(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FeatureLayerDrawingDescription10_1> {
    let version = read_drawing_version(reader, 5)?;
    let renderer =
        crate::layers::feature_layer::read_drawing_feature_renderer(reader, max_bytes, refs)?;
    let raw_short = (version >= 2).then(|| reader.read_u16_le()).transpose()?;
    let raw_shorts = if version >= 3 {
        Some([
            reader.read_u16_le()?,
            reader.read_u16_le()?,
            reader.read_u16_le()?,
        ])
    } else {
        None
    };
    let labeling = if version >= 4 {
        Some(read_typed_rule_slot(
            reader,
            refs,
            &[LABELING_CLASS_ID],
            |reader, refs| {
                let version = read_drawing_version(reader, 1)?;
                let classes = read_typed_rule_slot(
                    reader,
                    refs,
                    &[LABEL_CLASSES_CLASS_ID],
                    |reader, refs| read_label_class_descriptions(reader, max_bytes, refs),
                )?;
                Ok(LabelingDescription10_1 { version, classes })
            },
        )?)
    } else {
        None
    };
    let raw_long = (version >= 5)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    Ok(FeatureLayerDrawingDescription10_1 {
        version,
        renderer,
        raw_short,
        raw_shorts,
        labeling,
        raw_long,
    })
}

fn read_typed_rule_slot<T>(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    classes: &[[u8; 16]],
    read_body: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<RuleSlot10_1<T>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if classes.contains(&refs.registered_class(key).unwrap_or(class_id)) =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if classes.contains(&class_id) => {
            Some(Box::new(read_body(reader, refs)?))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "对象槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_drawing_version(reader: &mut Reader<'_>, max_version: i16) -> io::Result<i16> {
    let version = reader.read_u16_le()? as i16;
    if version > max_version {
        return Err(io::Error::new(ErrorKind::InvalidData, "绘制描述版本超限"));
    }
    Ok(version)
}

fn read_label_class_descriptions(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LabelClassDescriptions10_1> {
    let version = read_drawing_version(reader, 1)?;
    let raw_count = reader.read_u32_le()? as i32;
    // 每个成员至少包含 16 字节对象头。
    let count = raw_count.max(0) as usize;
    if count > reader.remaining() / 16 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "标注类描述集合截断",
        ));
    }
    let mut classes = Vec::new();
    for _ in 0..count {
        classes.push(read_typed_rule_slot(
            reader,
            refs,
            &[LABEL_CLASS_CLASS_ID],
            |reader, refs| read_label_class_description(reader, max_bytes, refs),
        )?);
    }
    Ok(LabelClassDescriptions10_1 {
        version,
        raw_count,
        classes,
    })
}

fn read_label_class_description(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LabelClassDescription10_1> {
    use crate::objects::display_symbol::{
        DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1,
        read_symbol_slot_full_10_1, slot_full_stop_10_1,
    };
    use crate::objects::element_body::TEXT_SYMBOL_CLASS_ID;
    let version = read_drawing_version(reader, 3)?;
    let placement = read_typed_rule_slot(
        reader,
        refs,
        &[
            POINT_PLACEMENT_CLASS_ID,
            LINE_PLACEMENT_CLASS_ID,
            POLYGON_PLACEMENT_CLASS_ID,
        ],
        |reader, _| {
            Ok(LabelPlacementDescription10_1 {
                version: read_drawing_version(reader, 1)?,
                raw_long: reader.read_u32_le()? as i32,
            })
        },
    )?;
    let text = read_string_10_1(reader, max_bytes)?;
    let symbol = read_symbol_slot_full_10_1(reader, max_bytes, refs, MAX_SYMBOL_NESTING_DEPTH)?;
    let is_text_symbol = match &symbol {
        SymbolSlotFull10_1::Null | SymbolSlotFull10_1::TextSymbol { .. } => true,
        SymbolSlotFull10_1::BackReference { class_id, key } => {
            refs.registered_class(*key).unwrap_or(*class_id) == TEXT_SYMBOL_CLASS_ID
        }
        _ => false,
    };
    if !is_text_symbol || slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol).is_some() {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "标注描述中的文本符号尚未读完",
        ));
    }
    let raw_doubles = [reader.read_f64_le()?, reader.read_f64_le()?];
    let raw_short = (version >= 2).then(|| reader.read_u16_le()).transpose()?;
    let tail = if version >= 3 {
        Some((
            reader.read_u32_le()? as i32,
            read_string_10_1(reader, max_bytes)?,
        ))
    } else {
        None
    };
    Ok(LabelClassDescription10_1 {
        version,
        placement,
        text,
        symbol,
        raw_doubles,
        raw_short,
        tail,
    })
}

fn read_gp_field_map_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<GpFieldMap10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == GP_FIELD_MAP_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_FIELD_MAP_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 2 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPFieldMap 版本超限",
                ));
            }
            let raw_long = reader.read_u32_le()?;
            let delimiter = read_string_10_1(reader, max_bytes)?;
            let field =
                read_gp_coded_value_slot_of_class(reader, max_bytes, refs, depth, FIELD_CLASS_ID)?;
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 40)?;
            let mut rows = Vec::new();
            for _ in 0..count {
                rows.push(GpFieldMapRow10_1 {
                    field: read_gp_coded_value_slot_of_class(
                        reader,
                        max_bytes,
                        refs,
                        depth,
                        FIELD_CLASS_ID,
                    )?,
                    table: read_gp_de_table_slot(reader, max_bytes, refs, depth)?,
                    raw_longs: [reader.read_u32_le()?, reader.read_u32_le()?],
                });
            }
            Some(Box::new(GpFieldMap10_1 {
                version,
                raw_long,
                delimiter,
                field,
                raw_count,
                rows,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "不支持的 IGPFieldMap 类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_coded_value_slot_of_class(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    expected_class: [u8; 16],
) -> io::Result<RuleSlot10_1<GpCodedValueBody10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let class = match entry {
        ObjectEntry10_1::Null => expected_class,
        ObjectEntry10_1::NewObject { class_id, .. } => class_id,
        ObjectEntry10_1::BackReference { class_id, key } => {
            table.registered_class(key).unwrap_or(class_id)
        }
    };
    if class != expected_class {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "IGPValue 子槽包含不支持的类",
        ));
    }
    read_gp_coded_value_entry(reader, max_bytes, table, depth, entry)
}

fn read_gp_field_info(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    version: u16,
) -> io::Result<GpCodedValue10_1> {
    let min_row_bytes = if version >= 2 { 38 } else { 18 };
    let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, min_row_bytes)?;
    let mut rows = Vec::new();
    for _ in 0..count {
        rows.push(GpFieldInfoRow10_1 {
            leading_long: reader.read_u32_le()? as i32,
            strings: [
                read_bstr_string_10_1(reader, max_bytes)?,
                read_bstr_string_10_1(reader, max_bytes)?,
            ],
            trailing_long: reader.read_u32_le()? as i32,
            raw_short: reader.read_u16_le()?,
            extra: (version >= 2)
                .then(|| -> io::Result<_> {
                    Ok(GpFieldInfoRowV2Fields10_1 {
                        first_text: read_bstr_string_10_1(reader, max_bytes)?,
                        first_long: reader.read_u32_le()? as i32,
                        second_text: read_bstr_string_10_1(reader, max_bytes)?,
                        remaining_longs: [
                            reader.read_u32_le()? as i32,
                            reader.read_u32_le()? as i32,
                        ],
                    })
                })
                .transpose()?,
        });
    }
    Ok(GpCodedValue10_1::FieldInfo { raw_count, rows })
}

fn read_gp_data_file_transport(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<GdsData10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if table.registered_class(key).unwrap_or(class_id) == GDS_DATA_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: GDS_DATA_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 3, "GDSData")?;
            let length = reader.read_u32_le()? as i32;
            if length < 0 || length as usize > max_bytes {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GDSData 数据长度超限",
                ));
            }
            let bytes = reader.read_bytes(length as usize)?.to_vec();
            let name = read_string_10_1(reader, max_bytes)?;
            let raw_long = reader.read_u32_le()? as i32;
            let raw_short = reader.read_u16_le()?;
            let properties = (version >= 2)
                .then(|| read_gp_property_set_slot(reader, max_bytes, table))
                .transpose()?;
            let extra_string = (version >= 3)
                .then(|| read_string_10_1(reader, max_bytes))
                .transpose()?;
            Some(Box::new(GdsData10_1 {
                version,
                bytes,
                name,
                raw_long,
                raw_short,
                properties,
                extra_string,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPDataFile 的 IGDSData 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_address_locator_style_config(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
    raw_flag: u16,
) -> io::Result<GpAddressLocatorStyleConfig10_1> {
    let fields = [
        read_gp_fields_slot(reader, max_bytes, table, depth)?,
        read_gp_fields_slot(reader, max_bytes, table, depth)?,
        read_gp_fields_slot(reader, max_bytes, table, depth)?,
    ];
    let (raw_row_count, count) = read_gp_coded_value_count(reader, max_bytes, 40)?;
    let mut rows = Vec::new();
    for _ in 0..count {
        rows.push(read_gp_address_locator_style_row(
            reader, max_bytes, table, depth, version,
        )?);
    }
    let legacy_byte = (version == 1)
        .then(|| reader.read_bytes(1).map(|bytes| bytes[0]))
        .transpose()?;
    let properties = (version >= 2)
        .then(|| read_gp_property_set_slot(reader, max_bytes, table))
        .transpose()?;
    let variant_flag = (version >= 3).then(|| reader.read_u16_le()).transpose()?;
    let variant_and_table = variant_flag
        .filter(|flag| *flag != 0)
        .map(|_| {
            let value =
                read_padded_variant_object_stream_nested_10_1(reader, max_bytes, table, depth)?;
            let entry = read_gp_de_table_slot(reader, max_bytes, table, depth)?;
            Ok::<_, io::Error>((value, entry))
        })
        .transpose()?;
    let (variant, de_table) = variant_and_table.unzip();
    Ok(GpAddressLocatorStyleConfig10_1 {
        raw_flag,
        fields,
        raw_row_count,
        rows,
        legacy_byte,
        properties,
        variant_flag,
        variant,
        table: de_table,
    })
}

fn read_gp_address_locator_style_row(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpAddressLocatorStyleRow10_1> {
    let name = read_string_10_1(reader, max_bytes)?;
    let raw_short = (version >= 4).then(|| reader.read_u16_le()).transpose()?;
    let mut uids = Vec::new();
    let raw_object_count = if version >= 2 {
        let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        for _ in 0..count {
            uids.push(read_gp_uid_slot(reader, table)?);
        }
        raw_count
    } else {
        reader.read_u32_le()? as i32
    };
    let (raw_value_count, count) =
        read_gp_coded_value_count(reader, max_bytes, if version >= 2 { 16 } else { 4 })?;
    if raw_value_count < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GPAddressLocatorStyle 的参考数据行值数为负",
        ));
    }
    let fields = read_gp_fields_slot(reader, max_bytes, table, depth)?;
    let values = if version >= 2 {
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(read_padded_variant_object_stream_nested_10_1(
                reader, max_bytes, table, depth,
            )?);
        }
        GpAddressLocatorStyleRowValues10_1::Variants(values)
    } else {
        let mut values = Vec::new();
        for _ in 0..count {
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 4)?;
            if raw_count < 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPAddressLocatorStyle 的参考数据行字符串数为负",
                ));
            }
            let mut strings = Vec::new();
            for _ in 0..count {
                strings.push(read_string_10_1(reader, max_bytes)?);
            }
            values.push(strings);
        }
        GpAddressLocatorStyleRowValues10_1::Strings(values)
    };
    let de_table = read_gp_de_table_slot(reader, max_bytes, table, depth)?;
    Ok(GpAddressLocatorStyleRow10_1 {
        name,
        raw_short,
        raw_object_count,
        uids,
        raw_value_count,
        fields,
        values,
        table: de_table,
    })
}

fn read_gp_fields_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<FieldsBody10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if table.registered_class(key).unwrap_or(class_id) == FIELDS_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: FIELDS_CLASS_ID,
            ..
        } => Some(Box::new(read_fields_body_nested_10_1(
            reader, max_bytes, table, depth,
        )?)),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GP 对象的 IFields 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_de_table_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<DeTable10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == DE_TABLE_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: DE_TABLE_CLASS_ID,
            ..
        } => {
            if depth >= MAX_BOOKMARK_NAME_NESTING {
                return Err(io::Error::new(ErrorKind::InvalidData, "DETable 嵌套过深"));
            }
            Some(Box::new(read_gp_de_table(
                reader,
                max_bytes,
                refs,
                depth + 1,
            )?))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "DETable 对象槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_layer_version(reader: &mut Reader<'_>, max: u16, class: &str) -> io::Result<u16> {
    let version = reader.read_u16_le()?;
    if version > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{class} 版本超限"),
        ));
    }
    Ok(version)
}

fn read_gp_layer_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<GpLayer10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                refs.registered_class(key).unwrap_or(class_id),
                GP_LAYER_CLASS_ID
                    | GP_COMPOSITE_LAYER_CLASS_ID
                    | GP_MOSAIC_LAYER_CLASS_ID
                    | GP_CADASTRAL_FABRIC_LAYER_CLASS_ID
                    | GP_GROUP_LAYER_CLASS_ID
                    | GP_NA_LAYER_CLASS_ID
                    | GP_TERRAIN_LAYER_CLASS_ID
                    | GP_TOPOLOGY_LAYER_CLASS_ID
                    | GP_NETWORK_DATASET_LAYER_CLASS_ID
                    | GP_LAS_DATASET_LAYER_CLASS_ID
                    | GP_RASTER_LAYER_CLASS_ID
                    | GP_TIN_LAYER_CLASS_ID
                    | GP_FEATURE_LAYER_CLASS_ID
                    | GP_RASTER_CATALOG_LAYER_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (GP_LAYER_CLASS_ID
                | GP_COMPOSITE_LAYER_CLASS_ID
                | GP_MOSAIC_LAYER_CLASS_ID
                | GP_CADASTRAL_FABRIC_LAYER_CLASS_ID
                | GP_GROUP_LAYER_CLASS_ID
                | GP_NA_LAYER_CLASS_ID
                | GP_TERRAIN_LAYER_CLASS_ID
                | GP_TOPOLOGY_LAYER_CLASS_ID
                | GP_NETWORK_DATASET_LAYER_CLASS_ID
                | GP_LAS_DATASET_LAYER_CLASS_ID
                | GP_RASTER_LAYER_CLASS_ID
                | GP_TIN_LAYER_CLASS_ID
                | GP_FEATURE_LAYER_CLASS_ID
                | GP_RASTER_CATALOG_LAYER_CLASS_ID),
            ..
        } => Some(Box::new(read_gp_layer_body(
            reader, max_bytes, refs, depth, class_id,
        )?)),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("DELayer 的 IGPLayer 槽尚未支持：{entry:?}"),
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_layer_body(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
) -> io::Result<GpLayer10_1> {
    if depth >= MAX_BOOKMARK_NAME_NESTING {
        return Err(io::Error::new(ErrorKind::InvalidData, "IGPLayer 嵌套过深"));
    }
    let derived_version = matches!(
        class_id,
        GP_TIN_LAYER_CLASS_ID
            | GP_TERRAIN_LAYER_CLASS_ID
            | GP_TOPOLOGY_LAYER_CLASS_ID
            | GP_NETWORK_DATASET_LAYER_CLASS_ID
            | GP_LAS_DATASET_LAYER_CLASS_ID
            | GP_RASTER_LAYER_CLASS_ID
    )
    .then(|| {
        let version = read_de_layer_version(reader, 1, "GP 图层派生类")?;
        if class_id == GP_LAS_DATASET_LAYER_CLASS_ID && version != 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "GPLasDatasetLayer 只接受版本 1",
            ));
        }
        Ok(version)
    })
    .transpose()?;
    let feature_layer_version = matches!(
        class_id,
        GP_FEATURE_LAYER_CLASS_ID | GP_RASTER_CATALOG_LAYER_CLASS_ID
    )
    .then(|| read_de_layer_version(reader, 2, "GPFeatureLayerBase"))
    .transpose()?;
    let composite_versions = matches!(
        class_id,
        GP_COMPOSITE_LAYER_CLASS_ID
            | GP_MOSAIC_LAYER_CLASS_ID
            | GP_CADASTRAL_FABRIC_LAYER_CLASS_ID
            | GP_GROUP_LAYER_CLASS_ID
            | GP_NA_LAYER_CLASS_ID
    )
    .then(|| {
        Ok::<_, io::Error>((
            if class_id == GP_NA_LAYER_CLASS_ID {
                {
                    let version = reader.read_u16_le()?;
                    if version as i16 > 3 {
                        return Err(io::Error::new(ErrorKind::InvalidData, "GPNALayer 版本超限"));
                    }
                    version
                }
            } else {
                read_de_layer_version(reader, 1, "GPCompositeLayer/GPMosaicLayer")?
            },
            read_de_layer_version(reader, 1, "GPCompositeLayerBase")?,
        ))
    })
    .transpose()?;
    let version = read_de_layer_version(reader, 2, "GPLayerBase")?;
    let gp_value_version = read_de_layer_version(reader, 1, "GPValue")?;
    let gp_value_short = reader.read_u16_le()?;
    let name = read_string_10_1(reader, max_bytes)?;
    let area_of_interest =
        read_surround_envelope_slot(reader, max_bytes, refs, "GPLayer IEnvelope 槽")?;
    let element = read_gp_data_element_child(reader, max_bytes, refs, depth)?;
    let legacy_layer = if version < 2 {
        // GP 层旧版在读取后释放这个 ILayer 槽，流中仍须完整消费。
        let entry = read_object_entry_10_1(reader, refs)?;
        let body = match entry {
            ObjectEntry10_1::NewObject { class_id, .. } => Some(Box::new(
                crate::layers::layer_list::read_complete_layer_body_10_1(
                    reader, class_id, max_bytes, refs,
                )?,
            )),
            _ => None,
        };
        Some(RuleSlot10_1 { entry, body })
    } else {
        None
    };
    let composite = if let Some((version, base_version)) = composite_versions {
        let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        let mut layers = Vec::new();
        for _ in 0..count {
            layers.push(read_gp_layer_slot(reader, max_bytes, refs, depth + 1)?);
        }
        Some(GpCompositeLayerMembers10_1 {
            version,
            base_version,
            raw_count,
            layers,
        })
    } else {
        None
    };
    let feature_layer = feature_layer_version
        .map(|version| read_gp_feature_layer_fields(reader, max_bytes, refs, depth, version))
        .transpose()?;
    let raster = if class_id == GP_RASTER_LAYER_CLASS_ID {
        Some(GpRasterLayerFields10_1 {
            text: read_string_10_1(reader, max_bytes)?,
            field: read_gp_coded_value_slot_of_class(
                reader,
                max_bytes,
                refs,
                depth,
                FIELD_CLASS_ID,
            )?,
        })
    } else {
        None
    };
    let na = if class_id == GP_NA_LAYER_CLASS_ID {
        let version = composite.as_ref().expect("GPNALayer 有组合层前缀").version as i16;
        Some(Box::new(read_gp_na_layer_fields(
            reader, max_bytes, refs, depth, version,
        )?))
    } else {
        None
    };
    Ok(GpLayer10_1 {
        na,
        raster,
        derived_version,
        feature_layer,
        composite,
        version,
        gp_value_version,
        gp_value_short,
        name,
        area_of_interest,
        element,
        legacy_layer,
    })
}

fn read_gp_na_layer_fields(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    version: i16,
) -> io::Result<GpNaLayerFields10_1> {
    use crate::layers::{
        na_class::read_na_class_definition_slot,
        na_layer::{
            NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID, NA_LOCATOR_FEATURE_AGENT_CLASS_ID,
            read_na_locator_feature_agent_body, read_na_parameter_array,
        },
    };

    let network = read_gp_data_element_child(reader, max_bytes, refs, depth)?;
    if let ObjectEntry10_1::NewObject { class_id, .. }
    | ObjectEntry10_1::BackReference { class_id, .. } = network.entry
    {
        let class_id = match network.entry {
            ObjectEntry10_1::BackReference { key, .. } => {
                refs.registered_class(key).unwrap_or(class_id)
            }
            _ => class_id,
        };
        if class_id != DE_NETWORK_DATASET_CLASS_ID {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPNALayer 的 IDENetworkDataset 类尚未支持",
            ));
        }
    }
    let (_, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut definitions = Vec::new();
    for _ in 0..count {
        let (entry, body) = read_na_class_definition_slot(reader, max_bytes, refs)?;
        definitions.push(RuleSlot10_1 { entry, body });
    }
    let linear_unit =
        read_gp_coded_value_slot_of_class(reader, max_bytes, refs, depth, GP_LINEAR_UNIT_CLASS_ID)?;
    let (_, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut agents = Vec::new();
    for _ in 0..count {
        agents.push(read_typed_rule_slot(
            reader,
            refs,
            &[NA_LOCATOR_FEATURE_AGENT_CLASS_ID],
            |reader, _| read_na_locator_feature_agent_body(reader, max_bytes),
        )?);
    }
    let raw_short = reader.read_u16_le()?;
    let properties = [
        read_gp_property_set_slot(reader, max_bytes, refs)?,
        read_gp_property_set_slot(reader, max_bytes, refs)?,
    ];
    let parameters = (version >= 1)
        .then(|| {
            read_typed_rule_slot(
                reader,
                refs,
                &[NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID],
                |reader, refs| read_na_parameter_array(reader, max_bytes, refs),
            )
        })
        .transpose()?;
    let tables = if version >= 2 {
        let (_, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        let mut tables = Vec::new();
        for _ in 0..count {
            tables.push(read_gp_coded_value_slot_of_class(
                reader,
                max_bytes,
                refs,
                depth,
                GP_TABLE_VIEW_CLASS_ID,
            )?);
        }
        Some(tables)
    } else {
        None
    };
    let tail_short = (version >= 3).then(|| reader.read_u16_le()).transpose()?;
    Ok(GpNaLayerFields10_1 {
        network,
        definitions,
        linear_unit,
        agents,
        raw_short,
        properties,
        parameters,
        tables,
        tail_short,
    })
}

fn read_gp_feature_layer_fields(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpFeatureLayerFields10_1> {
    let text = read_string_10_1(reader, max_bytes)?;
    let fid_set = read_gp_fid_set_slot(reader, refs)?;
    let (table, field_info) = if version >= 2 {
        (
            Some(read_gp_data_element_child(reader, max_bytes, refs, depth)?),
            Some(read_gp_coded_value_slot_of_class(
                reader,
                max_bytes,
                refs,
                depth,
                GP_FIELD_INFO_CLASS_ID,
            )?),
        )
    } else {
        (None, None)
    };
    Ok(GpFeatureLayerFields10_1 {
        version,
        text,
        fid_set,
        table,
        field_info,
    })
}

fn read_gp_fid_set_slot(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<crate::layers::feature_id_set::FeatureIdSetBody10_1>> {
    use crate::layers::feature_id_set::{FID_SET_CLASS_ID, read_fid_set_body_10_1};

    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == FID_SET_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: FID_SET_CLASS_ID,
            key,
        } => Some(Box::new(read_fid_set_body_10_1(reader, refs, key)?)),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GP 对象的 IFIDSet 类尚未支持",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_feature_record_set_layer(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpFeatureRecordSetLayer10_1> {
    use crate::layers::annotate_props::{
        EDIT_TEMPLATE_MANAGER_CLASS_ID, read_edit_template_manager_body_10_1,
    };
    let record_set = read_gp_record_set(reader, max_bytes, refs, depth, version)?;
    let layer_text = read_string_10_1(reader, max_bytes)?;
    let drawing = read_layer_drawing_description_slot(reader, max_bytes, refs)?;
    let templates = if version >= 3 {
        let entry = read_object_entry_10_1(reader, refs)?;
        let body = match entry {
            ObjectEntry10_1::Null => None,
            ObjectEntry10_1::BackReference { class_id, key }
                if refs.registered_class(key).unwrap_or(class_id)
                    == EDIT_TEMPLATE_MANAGER_CLASS_ID =>
            {
                None
            }
            ObjectEntry10_1::NewObject {
                class_id: EDIT_TEMPLATE_MANAGER_CLASS_ID,
                key,
            } => {
                let parsed = read_edit_template_manager_body_10_1(reader, max_bytes, refs, key)?;
                if parsed.stop.is_some() {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "GPFeatureRecordSetLayer 的编辑模板尚未读完",
                    ));
                }
                Some(Box::new(parsed))
            }
            _ => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "GPFeatureRecordSetLayer 的编辑模板管理器类尚未支持",
                ));
            }
        };
        Some(RuleSlot10_1 { entry, body })
    } else {
        None
    };
    Ok(GpFeatureRecordSetLayer10_1 {
        record_set,
        layer_text,
        drawing,
        templates,
    })
}

fn read_gp_record_set(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpRecordSet10_1> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == RECORD_SET_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: RECORD_SET_CLASS_ID,
            ..
        } => Some(Box::new(read_record_set(reader, max_bytes, refs, depth)?)),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPRecordSet 中的 RecordSet 类尚未支持",
            ));
        }
    };
    let v2 = if version >= 2 {
        Some((read_string_10_1(reader, max_bytes)?, reader.read_u16_le()?))
    } else {
        None
    };
    let text = read_string_10_1(reader, max_bytes)?;
    Ok(GpRecordSet10_1 {
        records: RuleSlot10_1 { entry, body },
        v2,
        text,
    })
}

fn read_record_set(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RecordSet10_1> {
    if depth >= MAX_BOOKMARK_NAME_NESTING {
        return Err(io::Error::new(ErrorKind::InvalidData, "RecordSet 嵌套过深"));
    }
    let version = reader.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(ErrorKind::InvalidData, "RecordSet 版本超限"));
    }
    let fields = read_gp_fields_slot(reader, max_bytes, refs, depth + 1)?;
    let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 6)?;
    let mut rows = Vec::new();
    for _ in 0..count {
        let version = reader.read_u16_le()? as i16;
        if version > 1 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Record 版本超限"));
        }
        let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(read_padded_variant_object_stream_nested_10_1(
                reader,
                max_bytes,
                refs,
                depth + 1,
            )?);
        }
        rows.push(Record10_1 {
            version,
            raw_count,
            values,
        });
    }
    Ok(RecordSet10_1 {
        version,
        fields,
        raw_count,
        rows,
    })
}

fn read_gp_table_view(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpTableView10_1> {
    let first_text = read_string_10_1(reader, max_bytes)?;
    let table = read_gp_de_table_slot(reader, max_bytes, refs, depth)?;
    let second_text = read_string_10_1(reader, max_bytes)?;
    let fid_set = read_gp_fid_set_slot(reader, refs)?;
    let tail = if version >= 2 {
        Some(GpTableViewTail10_1 {
            table: read_gp_de_table_slot(reader, max_bytes, refs, depth)?,
            field_info: read_gp_coded_value_slot_of_class(
                reader,
                max_bytes,
                refs,
                depth,
                GP_FIELD_INFO_CLASS_ID,
            )?,
        })
    } else {
        None
    };
    Ok(GpTableView10_1 {
        first_text,
        table,
        second_text,
        fid_set,
        tail,
    })
}

fn read_gp_arc_info_items(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<(i32, Vec<RuleSlot10_1<GpCodedValueBody10_1>>)> {
    let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut items = Vec::new();
    for _ in 0..count {
        let entry = read_object_entry_10_1(reader, refs)?;
        match entry {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::NewObject {
                class_id: GP_ARC_INFO_ITEM_CLASS_ID,
                ..
            } => {}
            ObjectEntry10_1::BackReference { class_id, key }
                if refs.registered_class(key).unwrap_or(class_id) == GP_ARC_INFO_ITEM_CLASS_ID => {}
            _ => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "IGPArcInfoItem 槽包含不支持的类",
                ));
            }
        }
        items.push(read_gp_coded_value_entry(
            reader, max_bytes, refs, depth, entry,
        )?);
    }
    Ok((raw_count, items))
}

fn read_de_terrain_sources(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<(i32, Vec<DeTerrainSource10_1>)> {
    let (source_count, count) = read_gp_coded_value_count(reader, max_bytes, 62)?;
    let mut sources = Vec::new();
    for _ in 0..count {
        let version = reader.read_u32_le()?;
        if version > 3 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "DETerrain 源版本超限",
            ));
        }
        let mut longs = [0; 5];
        for value in &mut longs {
            *value = reader.read_u32_le()?;
        }
        let shorts = [
            reader.read_u16_le()?,
            reader.read_u16_le()?,
            reader.read_u16_le()?,
        ];
        let values = [
            reader.read_f64_le()?.to_bits(),
            reader.read_f64_le()?.to_bits(),
        ];
        let strings = [
            read_string_10_1(reader, max_bytes)?,
            read_string_10_1(reader, max_bytes)?,
            read_string_10_1(reader, max_bytes)?,
        ];
        let v2 = (version == 2).then(|| reader.read_u32_le()).transpose()?;
        let name_count = reader.read_u32_le()? as i32;
        if name_count < 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "DETerrain 源名称计数为负",
            ));
        }
        let mut names = Vec::new();
        // 标志 1 的名称由父对象上下文提供。
        if longs[3] != 1 {
            if name_count as usize > reader.remaining().min(max_bytes) / 4 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "DETerrain 源名称计数超限",
                ));
            }
            for _ in 0..name_count {
                names.push(read_string_10_1(reader, max_bytes)?);
            }
        }
        sources.push(DeTerrainSource10_1 {
            version,
            longs,
            shorts,
            values,
            strings,
            v2,
            name_count,
            names,
        });
    }
    Ok((source_count, sources))
}

type DeTerrainLevel10_1 = ([u32; 3], u64);

fn read_de_terrain_levels(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    level_type: u32,
) -> io::Result<(i32, Vec<DeTerrainLevel10_1>)> {
    let (level_count, count) = read_gp_coded_value_count(reader, max_bytes, 24)?;
    let mut levels = Vec::new();
    if count != 0 && level_type > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "DETerrain 层级类型尚未支持",
        ));
    }
    for _ in 0..count {
        if reader.read_u32_le()? != 1 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "DETerrain 层级版本超限",
            ));
        }
        levels.push((
            [
                reader.read_u32_le()?,
                reader.read_u32_le()?,
                reader.read_u32_le()?,
            ],
            reader.read_f64_le()?.to_bits(),
        ));
    }
    Ok((level_count, levels))
}

fn read_network_dataset_list<T>(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    min_size: usize,
    mut read: impl FnMut(&mut Reader<'_>) -> io::Result<T>,
) -> io::Result<(i32, Vec<T>)> {
    let raw_count = reader.read_u32_le()? as i32;
    let count = raw_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / min_size {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DENetworkDataset 列表数量超限",
        ));
    }
    let mut rows = Vec::new();
    for _ in 0..count {
        rows.push(read(reader)?);
    }
    Ok((raw_count, rows))
}

fn read_de_network_dataset(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeNetworkDataset10_1> {
    let version = reader.read_u16_le()?;
    if !(1..=4).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DENetworkDataset 版本超限",
        ));
    }
    let geo_dataset_version = read_de_layer_version(reader, 1, "DEGeoDataset")?;
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let extent = read_surround_envelope_slot(reader, max_bytes, refs, "DENetworkDataset 范围")?;
    let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
    let name = read_string_10_1(reader, max_bytes)?;
    let mut initial_values = [0; 6];
    for value in &mut initial_values {
        *value = reader.read_u32_le()?;
    }
    let directions = (initial_values[5] != 0)
        .then(|| read_network_directions_slot_10_1(reader, max_bytes, refs))
        .transpose()?;
    let properties = [
        read_gp_property_set_slot(reader, max_bytes, refs)?,
        read_gp_property_set_slot(reader, max_bytes, refs)?,
    ];
    let sources = read_network_dataset_list(reader, max_bytes, 16, |r| {
        read_network_source_slot_10_1(r, max_bytes, refs)
    })?;
    let attributes = read_network_dataset_list(reader, max_bytes, 16, |r| {
        read_network_attribute_slot_10_1(r, max_bytes, refs)
    })?;
    let evaluator_rows = read_network_dataset_list(reader, max_bytes, 48, |r| {
        let values = [
            r.read_u32_le()?,
            r.read_u32_le()?,
            r.read_u32_le()?,
            r.read_u32_le()?,
        ];
        let uid = read_gp_uid_slot(r, refs)?;
        let properties = read_gp_property_set_slot(r, max_bytes, refs)?;
        Ok(DeNetworkEvaluatorRow10_1 {
            values,
            uid,
            properties,
        })
    })?;
    let extra_name = read_string_10_1(reader, max_bytes)?;
    let attribute_flag = reader.read_u32_le()?;
    let extra_attribute = (attribute_flag != 0)
        .then(|| read_network_attribute_slot_10_1(reader, max_bytes, refs))
        .transpose()?;
    let values = read_network_dataset_list(reader, max_bytes, 4, |r| r.read_u32_le())?;
    let extra_value = (version >= 2).then(|| reader.read_u32_le()).transpose()?;
    let traffic = (version >= 3)
        .then(|| -> io::Result<_> {
            let values = [
                reader.read_u32_le()?,
                reader.read_u32_le()?,
                reader.read_u32_le()?,
            ];
            let data = (values[2] != 0)
                .then(|| read_traffic_data_slot_10_1(reader, max_bytes, refs))
                .transpose()?;
            let name = read_string_10_1(reader, max_bytes)?;
            Ok(DeNetworkTraffic10_1 { values, data, name })
        })
        .transpose()?;
    let final_name = (version == 4)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;
    Ok(DeNetworkDataset10_1 {
        version,
        geo_dataset_version,
        dataset,
        extent,
        spatial_reference,
        name,
        initial_values,
        directions,
        properties,
        sources,
        attributes,
        evaluator_rows,
        extra_name,
        attribute_flag,
        extra_attribute,
        values,
        extra_value,
        traffic,
        final_name,
    })
}

fn read_de_terrain_geo_dataset(
    reader: &mut Reader<'_>,
    version: u32,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeTerrainGeoDataset10_1> {
    let geo_dataset_version = read_de_layer_version(reader, 1, "DEGeoDataset")?;
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let extent = read_surround_envelope_slot(reader, max_bytes, refs, "DETerrain 范围")?;
    let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
    let level_type = reader.read_u32_le()?;
    let v6 = (version >= 6).then(|| reader.read_u32_le()).transpose()?;
    let v7_bits = (version >= 7)
        .then(|| reader.read_f64_le().map(f64::to_bits))
        .transpose()?;
    let v8 = (version >= 8).then(|| reader.read_u32_le()).transpose()?;
    let value_bits = reader.read_f64_le()?.to_bits();
    let longs = [reader.read_u32_le()?, reader.read_u32_le()?];
    let name = read_string_10_1(reader, max_bytes)?;
    let v9_tail = if version >= 9 {
        let v11 = (version >= 11).then(|| reader.read_u16_le()).transpose()?;
        let v13 = (version == 13).then(|| reader.read_u32_le()).transpose()?;
        let longs = [reader.read_u32_le()?, reader.read_u32_le()?];
        let mut envelopes = [[0; 6]; 2];
        for envelope in &mut envelopes {
            for value in envelope {
                *value = reader.read_f64_le()?.to_bits();
            }
        }
        Some(DeTerrainV9Tail10_1 {
            v11,
            v13,
            longs,
            envelopes,
        })
    } else {
        None
    };
    let (source_count, sources) = read_de_terrain_sources(reader, max_bytes)?;
    let (level_count, levels) = read_de_terrain_levels(reader, max_bytes, level_type)?;
    Ok(DeTerrainGeoDataset10_1 {
        version,
        geo_dataset_version,
        dataset,
        extent,
        spatial_reference,
        level_type,
        v6,
        v7_bits,
        v8,
        value_bits,
        longs,
        name,
        v9_tail,
        source_count,
        sources,
        level_count,
        levels,
    })
}

fn read_de_terrain_legacy(
    reader: &mut Reader<'_>,
    version: u32,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DeTerrainLegacy10_1> {
    let leading_long = reader.read_u32_le()?;
    let leading_value = reader.read_f64_le()?.to_bits();
    let longs = [reader.read_u32_le()?, reader.read_u32_le()?];
    let (source_count, sources) = read_de_terrain_sources(reader, max_bytes)?;
    let (level_count, levels) = read_de_terrain_levels(reader, max_bytes, leading_long)?;
    let v2 = if version >= 2 {
        let name = read_string_10_1(reader, max_bytes)?;
        let value = reader.read_u32_le()?;
        let mut shorts = [0; 5];
        for short in &mut shorts {
            *short = reader.read_u16_le()?;
        }
        Some((name, value, shorts))
    } else {
        None
    };
    let v3 = (version >= 3)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;
    let v4 = if version == 4 {
        Some((
            read_surround_envelope_slot(reader, max_bytes, refs, "DETerrain 范围")?,
            read_spatial_reference_entry_10_1(reader, max_bytes, refs)?,
        ))
    } else {
        None
    };
    Ok(DeTerrainLegacy10_1 {
        version,
        leading_long,
        leading_value,
        longs,
        source_count,
        sources,
        level_count,
        levels,
        v2,
        v3,
        v4,
    })
}

fn read_de_cadastral_fabric(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeCadastralFabric10_1> {
    let version = read_gp_name_version(reader, 3, "DECadastralFabric")?;
    let geo_dataset_version = read_de_layer_version(reader, 1, "DEGeoDataset")?;
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let extent = read_surround_envelope_slot(reader, max_bytes, refs, "DECadastralFabric 范围")?;
    let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
    let first = reader.read_u32_le()?;
    let name = read_string_10_1(reader, max_bytes)?;
    let second = reader.read_u32_le()?;
    let (edit_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut edits = Vec::new();
    for _ in 0..count {
        let entry = read_object_entry_10_1(reader, refs)?;
        let body = match entry {
            ObjectEntry10_1::Null => None,
            ObjectEntry10_1::BackReference { class_id, key }
                if refs.registered_class(key).unwrap_or(class_id)
                    == CADASTRAL_FIELD_EDITS_CLASS_ID =>
            {
                None
            }
            ObjectEntry10_1::NewObject {
                class_id: CADASTRAL_FIELD_EDITS_CLASS_ID,
                ..
            } => Some(Box::new(CadastralFieldEdits10_1 {
                version: read_gp_name_version(reader, 1, "CadastralTableFieldEdits")?,
                fields: read_gp_fields_slot(reader, max_bytes, refs, depth)?,
                name: read_string_10_1(reader, max_bytes)?,
            })),
            _ => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "地籍字段编辑槽包含不支持的类",
                ));
            }
        };
        edits.push(RuleSlot10_1 { entry, body });
    }
    let tail_version = reader.read_u16_le()? as i16;
    let tail = if tail_version >= 2 {
        let first = reader.read_u32_le()?;
        let second = reader.read_u32_le()?;
        let first_value = reader.read_f64_le()?.to_bits();
        let third = reader.read_u32_le()?;
        let name = read_string_10_1(reader, max_bytes)?;
        let second_value = reader.read_f64_le()?.to_bits();
        let fourth = reader.read_u32_le()?;
        let mut shorts = [0; 5];
        for value in &mut shorts {
            *value = reader.read_u16_le()?;
        }
        let properties = if tail_version >= 3 {
            Some([
                read_gp_property_set_slot(reader, max_bytes, refs)?,
                read_gp_property_set_slot(reader, max_bytes, refs)?,
                read_gp_property_set_slot(reader, max_bytes, refs)?,
                read_gp_property_set_slot(reader, max_bytes, refs)?,
                read_gp_property_set_slot(reader, max_bytes, refs)?,
            ])
        } else {
            None
        };
        Some(DeCadastralTail10_1 {
            longs: [first, second, third, fourth],
            values: [first_value, second_value],
            name,
            shorts,
            properties,
        })
    } else {
        None
    };
    Ok(DeCadastralFabric10_1 {
        version,
        geo_dataset_version,
        dataset,
        extent,
        spatial_reference,
        longs: [first, second],
        name,
        edit_count,
        edits,
        tail_version,
        tail,
    })
}

fn read_de_toolbox(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeChild10_1> {
    let mut trial = reader.clone();
    let version = read_de_layer_version(&mut trial, 2, "DEToolbox")?;
    let mut trial_refs = refs.clone();
    let dataset = match read_gp_de_dataset(&mut trial, max_bytes, &mut trial_refs, depth) {
        Ok(dataset) => dataset,
        Err(_) => {
            // 基类读取失败时，原生回到对象体起点并直接重读基类。
            return Ok(DeChild10_1::Toolbox {
                dataset: read_gp_de_dataset(reader, max_bytes, refs, depth)?,
                extension: None,
            });
        }
    };
    let name = read_bstr_string_10_1(&mut trial, max_bytes)?;
    let v2 = if version == 2 {
        Some((
            [
                read_bstr_string_10_1(&mut trial, max_bytes)?,
                read_bstr_string_10_1(&mut trial, max_bytes)?,
            ],
            trial.read_u32_le()?,
        ))
    } else {
        None
    };
    *reader = trial;
    *refs = trial_refs;
    Ok(DeChild10_1::Toolbox {
        dataset,
        extension: Some(DeToolboxExtension10_1 { version, name, v2 }),
    })
}

fn read_representation_slot<T>(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    expected: [u8; 16],
    read_body: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<RuleSlot10_1<T>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == expected =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected => {
            Some(Box::new(read_body(reader, refs)?))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "制图表达槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_representation_class(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeRepresentationClass10_1> {
    use crate::objects::display_symbol::{
        REPRESENTATION_RULE_CLASS_ID, read_graphic_attribute_list_10_1,
        read_representation_rule_body_10_1,
    };
    let version = read_gp_name_version(reader, 5, "DERepresentationClass")?;
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let first = (version <= 2).then(|| reader.read_u32_le()).transpose()?;
    let first_name = (version != 4)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;
    let names = [
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
    ];
    let legacy_longs = first
        .map(|value| reader.read_u32_le().map(|last| [value, last]))
        .transpose()?;
    let rules = read_representation_slot(
        reader,
        refs,
        REPRESENTATION_RULES_CLASS_ID,
        |reader, refs| {
            let version = read_gp_name_version(reader, 2, "RepresentationRules")?;
            let value = reader.read_u32_le()?;
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 24)?;
            let mut items = Vec::new();
            for _ in 0..count {
                let id = reader.read_u32_le()?;
                let name = read_string_10_1(reader, max_bytes)?;
                let v1 = (version == 1).then(|| reader.read_u32_le()).transpose()?;
                let rule = read_representation_slot(
                    reader,
                    refs,
                    REPRESENTATION_RULE_CLASS_ID,
                    |reader, refs| {
                        read_representation_rule_body_10_1(
                            reader,
                            max_bytes,
                            refs,
                            MAX_BOOKMARK_NAME_NESTING.saturating_sub(depth),
                        )?
                        .ok_or_else(|| {
                            io::Error::new(ErrorKind::Unsupported, "制图表达规则含未知对象体")
                        })
                    },
                )?;
                items.push(RepresentationRuleItem10_1 { id, name, v1, rule });
            }
            Ok(RepresentationRules10_1 {
                version,
                value,
                raw_count,
                items,
            })
        },
    )?;
    let attributes =
        read_representation_slot(reader, refs, CLASS_ATTRIBUTES_CLASS_ID, |reader, refs| {
            let version = read_gp_name_version(reader, 1, "ClassAttributes")?;
            let attributes = read_graphic_attribute_list_10_1(reader, max_bytes, refs, 0)?
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::Unsupported, "制图表达属性含未知对象体")
                })?;
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 8)?;
            let mut names = Vec::new();
            for _ in 0..count {
                names.push((reader.read_u32_le()?, read_string_10_1(reader, max_bytes)?));
            }
            Ok(ClassAttributes10_1 {
                version,
                attributes,
                raw_count,
                names,
            })
        })?;
    let trailing_short = (version > 1).then(|| reader.read_u16_le()).transpose()?;
    Ok(DeRepresentationClass10_1 {
        version,
        dataset,
        legacy_longs,
        first_name,
        names,
        rules,
        attributes,
        trailing_short,
    })
}

fn read_gp_data_element_child(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<DeChild10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                refs.registered_class(key).unwrap_or(class_id),
                DE_TABLE_CLASS_ID
                    | DE_FEATURE_CLASS_CLASS_ID
                    | DE_RASTER_CATALOG_CLASS_ID
                    | DE_SHAPE_FILE_CLASS_ID
                    | DE_FEATURE_CLASS_INFO_CLASS_ID
                    | DE_RASTER_CATALOG_INFO_CLASS_ID
                    | DE_COVERAGE_FEATURE_CLASS_ID
                    | DE_RASTER_DATASET_CLASS_ID
                    | DE_MOSAIC_DATASET_CLASS_ID
                    | DE_CAD_DRAWING_DATASET_CLASS_ID
                    | DE_TIN_CLASS_ID
                    | DE_NETWORK_DATASET_CLASS_ID
                    | DE_TERRAIN_CLASS_ID
                    | DE_CADASTRAL_FABRIC_CLASS_ID
                    | DE_TOOL_CLASS_ID
                    | DE_TOOLBOX_CLASS_ID
                    | DE_REPRESENTATION_CLASS_ID
                    | DE_COVERAGE_CLASS_ID
                    | DE_LAS_DATASET_CLASS_ID
                    | DE_VPF_COVERAGE_CLASS_ID
                    | DE_RASTER_BAND_CLASS_ID
                    | DE_FEATURE_DATASET_CLASS_ID
                    | DE_CATALOG_DATASET_CLASS_ID
                    | DE_SERVER_CONNECTION_CLASS_ID
                    | DE_GEO_DATA_SERVER_CLASS_ID
                    | DE_MAP_SERVER_CLASS_ID
                    | DE_GLOBE_SERVER_CLASS_ID
                    | DE_GP_SERVER_CLASS_ID
                    | DE_IMAGE_SERVER_CLASS_ID
                    | DE_WCS_COVERAGE_CLASS_ID
                    | DE_WMS_MAP_CLASS_ID
                    | DE_WORKSPACE_CLASS_ID
                    | DE_FILE_CLASS_ID
                    | DE_FOLDER_CLASS_ID
                    | DE_MAP_DOCUMENT_CLASS_ID
                    | DE_DISK_CONNECTION_CLASS_ID
                    | DE_SCHEMATIC_DATASET_CLASS_ID
                    | DE_SCHEMATIC_FOLDER_CLASS_ID
                    | DE_CATALOG_ROOT_CLASS_ID
                    | DE_REMOTE_DATABASE_FOLDER_CLASS_ID
                    | DE_SPATIAL_REFERENCES_FOLDER_CLASS_ID
                    | DE_PRJ_FILE_CLASS_ID
                    | DE_TEXT_FILE_CLASS_ID
                    | DE_DBASE_TABLE_CLASS_ID
                    | DE_ARC_INFO_TABLE_CLASS_ID
                    | DE_TABLE_INFO_CLASS_ID
                    | DE_VPF_TABLE_CLASS_ID
                    | DE_LAYER_CLASS_ID
                    | DE_SCHEMATIC_DIAGRAM_CLASS_ID
                    | DE_TOPOLOGY_CLASS_ID
                    | DE_GEOMETRIC_NETWORK_CLASS_ID
                    | DE_RELATIONSHIP_CLASS_CLASS_ID
                    | DE_RELATIONSHIP_CLASS_INFO_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (DE_TABLE_CLASS_ID
                | DE_FEATURE_CLASS_CLASS_ID
                | DE_RASTER_CATALOG_CLASS_ID
                | DE_SHAPE_FILE_CLASS_ID
                | DE_FEATURE_CLASS_INFO_CLASS_ID
                | DE_RASTER_CATALOG_INFO_CLASS_ID
                | DE_COVERAGE_FEATURE_CLASS_ID
                | DE_RASTER_DATASET_CLASS_ID
                | DE_MOSAIC_DATASET_CLASS_ID
                | DE_CAD_DRAWING_DATASET_CLASS_ID
                | DE_TIN_CLASS_ID
                | DE_NETWORK_DATASET_CLASS_ID
                | DE_TERRAIN_CLASS_ID
                | DE_CADASTRAL_FABRIC_CLASS_ID
                | DE_TOOL_CLASS_ID
                | DE_TOOLBOX_CLASS_ID
                | DE_REPRESENTATION_CLASS_ID
                | DE_COVERAGE_CLASS_ID
                | DE_LAS_DATASET_CLASS_ID
                | DE_VPF_COVERAGE_CLASS_ID
                | DE_RASTER_BAND_CLASS_ID
                | DE_FEATURE_DATASET_CLASS_ID
                | DE_CATALOG_DATASET_CLASS_ID
                | DE_SERVER_CONNECTION_CLASS_ID
                | DE_GEO_DATA_SERVER_CLASS_ID
                | DE_MAP_SERVER_CLASS_ID
                | DE_GLOBE_SERVER_CLASS_ID
                | DE_GP_SERVER_CLASS_ID
                | DE_IMAGE_SERVER_CLASS_ID
                | DE_WCS_COVERAGE_CLASS_ID
                | DE_WMS_MAP_CLASS_ID
                | DE_WORKSPACE_CLASS_ID
                | DE_FILE_CLASS_ID
                | DE_FOLDER_CLASS_ID
                | DE_MAP_DOCUMENT_CLASS_ID
                | DE_DISK_CONNECTION_CLASS_ID
                | DE_SCHEMATIC_DATASET_CLASS_ID
                | DE_SCHEMATIC_FOLDER_CLASS_ID
                | DE_CATALOG_ROOT_CLASS_ID
                | DE_REMOTE_DATABASE_FOLDER_CLASS_ID
                | DE_SPATIAL_REFERENCES_FOLDER_CLASS_ID
                | DE_PRJ_FILE_CLASS_ID
                | DE_TEXT_FILE_CLASS_ID
                | DE_DBASE_TABLE_CLASS_ID
                | DE_ARC_INFO_TABLE_CLASS_ID
                | DE_TABLE_INFO_CLASS_ID
                | DE_VPF_TABLE_CLASS_ID
                | DE_LAYER_CLASS_ID
                | DE_SCHEMATIC_DIAGRAM_CLASS_ID
                | DE_TOPOLOGY_CLASS_ID
                | DE_GEOMETRIC_NETWORK_CLASS_ID
                | DE_RELATIONSHIP_CLASS_CLASS_ID
                | DE_RELATIONSHIP_CLASS_INFO_CLASS_ID),
            ..
        } => {
            if depth >= MAX_BOOKMARK_NAME_NESTING {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "DataElement 嵌套过深",
                ));
            }
            let body = if class_id == DE_TABLE_CLASS_ID {
                DeChild10_1::Table(read_gp_de_table(reader, max_bytes, refs, depth + 1)?)
            } else if matches!(
                class_id,
                DE_SCHEMATIC_DATASET_CLASS_ID | DE_SCHEMATIC_FOLDER_CLASS_ID
            ) {
                let version = reader.read_u16_le()?;
                if version != 1 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "Schematic 数据元素只接受版本 1",
                    ));
                }
                DeChild10_1::SchematicDataset {
                    dataset: read_gp_de_dataset(reader, max_bytes, refs, depth + 1)?,
                }
            } else if matches!(
                class_id,
                DE_FILE_CLASS_ID
                    | DE_FOLDER_CLASS_ID
                    | DE_MAP_DOCUMENT_CLASS_ID
                    | DE_DISK_CONNECTION_CLASS_ID
                    | DE_CATALOG_ROOT_CLASS_ID
                    | DE_REMOTE_DATABASE_FOLDER_CLASS_ID
                    | DE_SPATIAL_REFERENCES_FOLDER_CLASS_ID
                    | DE_PRJ_FILE_CLASS_ID
            ) {
                let version = reader.read_u16_le()?;
                if version > 1 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "文件数据元素版本超限",
                    ));
                }
                let element = read_gp_data_element(reader, max_bytes, refs, depth + 1)?;
                let spatial_reference = (class_id == DE_PRJ_FILE_CLASS_ID)
                    .then(|| {
                        read_spatial_reference_entry_10_1(reader, max_bytes, refs).map(Box::new)
                    })
                    .transpose()?;
                DeChild10_1::FileSystem {
                    version,
                    element,
                    spatial_reference,
                }
            } else if class_id == DE_LAYER_CLASS_ID {
                DeChild10_1::Layer {
                    version: read_de_layer_version(reader, 1, "DELayer")?,
                    element: read_gp_data_element(reader, max_bytes, refs, depth + 1)?,
                    layer: read_gp_layer_slot(reader, max_bytes, refs, depth + 1)?,
                }
            } else if class_id == DE_TABLE_INFO_CLASS_ID {
                DeChild10_1::TableInfo(read_de_compact_table(reader, max_bytes, refs, depth + 1)?)
            } else if matches!(
                class_id,
                DE_DBASE_TABLE_CLASS_ID | DE_VPF_TABLE_CLASS_ID | DE_ARC_INFO_TABLE_CLASS_ID
            ) {
                let version = read_de_layer_version(reader, 1, "文件表数据元素")?;
                let table = read_de_compact_table(reader, max_bytes, refs, depth + 1)?;
                if class_id == DE_ARC_INFO_TABLE_CLASS_ID {
                    let (raw_item_count, items) =
                        read_gp_arc_info_items(reader, max_bytes, refs, depth + 1)?;
                    DeChild10_1::ArcInfoTable {
                        version,
                        table,
                        raw_item_count,
                        items,
                    }
                } else {
                    DeChild10_1::FileTable { version, table }
                }
            } else if class_id == DE_TEXT_FILE_CLASS_ID {
                let version = reader.read_u16_le()?;
                let base = match version {
                    1 => DeTextFileBase10_1::Element(read_gp_data_element(
                        reader,
                        max_bytes,
                        refs,
                        depth + 1,
                    )?),
                    0 | 2 => DeTextFileBase10_1::Table(read_de_compact_table(
                        reader,
                        max_bytes,
                        refs,
                        depth + 1,
                    )?),
                    _ => {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "DETextFile 版本超限",
                        ));
                    }
                };
                DeChild10_1::TextFile { version, base }
            } else if class_id == DE_REPRESENTATION_CLASS_ID {
                DeChild10_1::RepresentationClass(Box::new(read_de_representation_class(
                    reader,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if class_id == DE_TOOLBOX_CLASS_ID {
                read_de_toolbox(reader, max_bytes, refs, depth + 1)?
            } else if class_id == DE_TOOL_CLASS_ID {
                DeChild10_1::Tool {
                    version: read_de_layer_version(reader, 1, "DETool")?,
                    dataset: read_gp_de_dataset(reader, max_bytes, refs, depth + 1)?,
                }
            } else if class_id == DE_CADASTRAL_FABRIC_CLASS_ID {
                DeChild10_1::CadastralFabric(Box::new(read_de_cadastral_fabric(
                    reader,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if class_id == DE_NETWORK_DATASET_CLASS_ID {
                DeChild10_1::NetworkDataset(Box::new(read_de_network_dataset(
                    reader,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if class_id == DE_TERRAIN_CLASS_ID {
                let version = reader.read_u32_le()?;
                match version {
                    1..=4 => DeChild10_1::TerrainLegacy(Box::new(read_de_terrain_legacy(
                        reader, version, max_bytes, refs,
                    )?)),
                    5..=13 => DeChild10_1::TerrainGeoDataset(Box::new(
                        read_de_terrain_geo_dataset(reader, version, max_bytes, refs, depth + 1)?,
                    )),
                    _ => {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            format!("DETerrain 版本 {version} 尚未支持"),
                        ));
                    }
                }
            } else if class_id == DE_RASTER_BAND_CLASS_ID {
                DeChild10_1::RasterBand(Box::new(read_de_raster_band(
                    reader,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if class_id == DE_WORKSPACE_CLASS_ID {
                DeChild10_1::Workspace(Box::new(read_de_workspace(
                    reader,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if matches!(
                class_id,
                DE_SERVER_CONNECTION_CLASS_ID
                    | DE_GEO_DATA_SERVER_CLASS_ID
                    | DE_MAP_SERVER_CLASS_ID
                    | DE_GLOBE_SERVER_CLASS_ID
                    | DE_GP_SERVER_CLASS_ID
                    | DE_IMAGE_SERVER_CLASS_ID
                    | DE_WCS_COVERAGE_CLASS_ID
                    | DE_WMS_MAP_CLASS_ID
            ) {
                let version = reader.read_u16_le()?;
                if version > 1 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "服务数据元素版本超限",
                    ));
                }
                DeChild10_1::Server {
                    version,
                    element: read_gp_data_element(reader, max_bytes, refs, depth + 1)?,
                    strings: [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ],
                }
            } else if matches!(
                class_id,
                DE_RELATIONSHIP_CLASS_CLASS_ID | DE_RELATIONSHIP_CLASS_INFO_CLASS_ID
            ) {
                DeChild10_1::RelationshipClass(Box::new(read_de_relationship_class(
                    reader,
                    class_id,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if class_id == DE_MOSAIC_DATASET_CLASS_ID {
                DeChild10_1::MosaicDataset(Box::new(read_de_mosaic_dataset(
                    reader,
                    max_bytes,
                    refs,
                    depth + 1,
                )?))
            } else if matches!(
                class_id,
                DE_FEATURE_DATASET_CLASS_ID
                    | DE_SCHEMATIC_DIAGRAM_CLASS_ID
                    | DE_TOPOLOGY_CLASS_ID
                    | DE_GEOMETRIC_NETWORK_CLASS_ID
                    | DE_CATALOG_DATASET_CLASS_ID
                    | DE_RASTER_DATASET_CLASS_ID
                    | DE_CAD_DRAWING_DATASET_CLASS_ID
                    | DE_TIN_CLASS_ID
                    | DE_COVERAGE_CLASS_ID
                    | DE_LAS_DATASET_CLASS_ID
                    | DE_VPF_COVERAGE_CLASS_ID
            ) {
                let version = reader.read_u16_le()?;
                let vpf_base_version = (class_id == DE_VPF_COVERAGE_CLASS_ID)
                    .then(|| read_de_layer_version(reader, 1, "DEFeatureDatasetBase"))
                    .transpose()?;
                let geo_dataset_version = reader.read_u16_le()?;
                let max_version = match class_id {
                    DE_TOPOLOGY_CLASS_ID | DE_RASTER_DATASET_CLASS_ID | DE_LAS_DATASET_CLASS_ID => {
                        2
                    }
                    DE_GEOMETRIC_NETWORK_CLASS_ID => 6,
                    _ => 1,
                };
                let version_out_of_range = if class_id == DE_CATALOG_DATASET_CLASS_ID {
                    (version as i16) > 1
                } else if class_id == DE_SCHEMATIC_DIAGRAM_CLASS_ID {
                    version != 1
                } else {
                    version > max_version
                };
                if version_out_of_range || geo_dataset_version > 1 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "地理数据集或其基类版本超限",
                    ));
                }
                let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth + 1)?;
                let extent =
                    read_surround_envelope_slot(reader, max_bytes, refs, "DEGeoDataset 范围")?;
                let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
                if class_id == DE_SCHEMATIC_DIAGRAM_CLASS_ID {
                    DeChild10_1::SchematicDiagram {
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        text: read_string_10_1(reader, max_bytes)?,
                    }
                } else if class_id == DE_TOPOLOGY_CLASS_ID {
                    let value_bits = [
                        reader.read_f64_le()?.to_bits(),
                        reader.read_f64_le()?.to_bits(),
                    ];
                    let raw_long = reader.read_u32_le()? as i32;
                    let names = read_string_array_slot_10_1(reader, max_bytes, refs)?;
                    let mut rules = Vec::new();
                    let raw_rule_count = if version == 2 {
                        let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
                        for _ in 0..count {
                            rules.push(read_topology_rule_slot(reader, max_bytes, refs)?);
                        }
                        Some(raw_count)
                    } else {
                        None
                    };
                    DeChild10_1::Topology {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        value_bits,
                        raw_long,
                        names,
                        raw_rule_count,
                        rules,
                    }
                } else if class_id == DE_GEOMETRIC_NETWORK_CLASS_ID {
                    DeChild10_1::GeometricNetwork {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        network: read_de_geometric_network(reader, max_bytes, refs, version)?,
                    }
                } else if class_id == DE_RASTER_DATASET_CLASS_ID {
                    let strings = [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ];
                    let raw_long = reader.read_u32_le()? as i32;
                    let raw_short = reader.read_u16_le()?;
                    let storage = (version == 2)
                        .then(|| read_raster_storage_def_slot(reader, max_bytes, refs))
                        .transpose()?;
                    DeChild10_1::RasterDataset {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        strings,
                        raw_long,
                        raw_short,
                        storage,
                    }
                } else if class_id == DE_LAS_DATASET_CLASS_ID {
                    let tail = if version == 2 {
                        Some(DeLasDatasetTail10_1 {
                            raw_longs: [
                                reader.read_u32_le()? as i32,
                                reader.read_u32_le()? as i32,
                                reader.read_u32_le()? as i32,
                            ],
                            raw_shorts: [
                                reader.read_u16_le()?,
                                reader.read_u16_le()?,
                                reader.read_u16_le()?,
                            ],
                        })
                    } else {
                        None
                    };
                    DeChild10_1::LasDataset {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        tail,
                    }
                } else if class_id == DE_COVERAGE_CLASS_ID {
                    DeChild10_1::Coverage {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        properties: read_gp_property_set_slot(reader, max_bytes, refs)?,
                    }
                } else if class_id == DE_TIN_CLASS_ID {
                    DeChild10_1::Tin {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        value_bits: reader.read_f64_le()?.to_bits(),
                        raw_shorts: [
                            reader.read_u16_le()?,
                            reader.read_u16_le()?,
                            reader.read_u16_le()?,
                            reader.read_u16_le()?,
                        ],
                        fields: read_gp_fields_slot(reader, max_bytes, refs, depth + 1)?,
                    }
                } else if class_id == DE_CAD_DRAWING_DATASET_CLASS_ID {
                    DeChild10_1::CadDrawingDataset {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        raw_shorts: [
                            reader.read_u16_le()?,
                            reader.read_u16_le()?,
                            reader.read_u16_le()?,
                            reader.read_u16_le()?,
                        ],
                    }
                } else if class_id == DE_CATALOG_DATASET_CLASS_ID {
                    DeChild10_1::CatalogDataset {
                        version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                        raw_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
                    }
                } else {
                    DeChild10_1::FeatureDataset {
                        version,
                        vpf_base_version,
                        geo_dataset_version,
                        dataset,
                        extent,
                        spatial_reference,
                    }
                }
            } else {
                let derived_version = match class_id {
                    DE_RASTER_CATALOG_CLASS_ID | DE_RASTER_CATALOG_INFO_CLASS_ID => {
                        Some(read_de_layer_version(reader, 1, "DERasterCatalog")?)
                    }
                    DE_SHAPE_FILE_CLASS_ID => {
                        Some(read_de_layer_version(reader, 1, "DEShapeFile")?)
                    }
                    DE_COVERAGE_FEATURE_CLASS_ID => {
                        Some(read_de_layer_version(reader, 1, "DECoverageFeatureClass")?)
                    }
                    _ => None,
                };
                let version = reader.read_u16_le()?;
                if version > 1 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "DEFeatureClassBase 版本超限",
                    ));
                }
                // 文件要素类沿用基类默认的零扩展标志。
                let extended = !matches!(
                    class_id,
                    DE_SHAPE_FILE_CLASS_ID
                        | DE_COVERAGE_FEATURE_CLASS_ID
                        | DE_FEATURE_CLASS_INFO_CLASS_ID
                        | DE_RASTER_CATALOG_INFO_CLASS_ID
                );
                let table = if extended {
                    DeFeatureClassTable10_1::Extended(read_gp_de_table(
                        reader,
                        max_bytes,
                        refs,
                        depth + 1,
                    )?)
                } else {
                    DeFeatureClassTable10_1::Compact(read_de_compact_table(
                        reader,
                        max_bytes,
                        refs,
                        depth + 1,
                    )?)
                };
                DeChild10_1::FeatureClass {
                    derived_version,
                    version,
                    table,
                    raw_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
                    name: read_string_10_1(reader, max_bytes)?,
                    raw_shorts: [
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                    ],
                    extent: read_surround_envelope_slot(
                        reader,
                        max_bytes,
                        refs,
                        "DEFeatureClass 范围",
                    )?,
                    spatial_reference: read_spatial_reference_entry_10_1(reader, max_bytes, refs)?,
                    strings: extended
                        .then(|| {
                            Ok::<_, io::Error>([
                                read_string_10_1(reader, max_bytes)?,
                                read_string_10_1(reader, max_bytes)?,
                            ])
                        })
                        .transpose()?,
                    coverage: (class_id == DE_COVERAGE_FEATURE_CLASS_ID)
                        .then(|| {
                            let first = reader.read_u32_le()? as i32;
                            let short = reader.read_u16_le()?;
                            let second = reader.read_u32_le()? as i32;
                            let (raw_item_count, items) =
                                read_gp_arc_info_items(reader, max_bytes, refs, depth + 1)?;
                            Ok::<_, io::Error>(DeCoverageFeatureClassTail10_1 {
                                longs: [first, second],
                                short,
                                raw_item_count,
                                items,
                            })
                        })
                        .transpose()?,
                }
            };
            Some(Box::new(body))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IDataElement 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_compact_table(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeCompactTable10_1> {
    // 构造函数保留 DETableBase 的默认零扩展标志。
    let table_version = reader.read_u16_le()?;
    if table_version > 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DETableBase 版本超限",
        ));
    }
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let table_short = reader.read_u16_le()?;
    let table_string = read_string_10_1(reader, max_bytes)?;
    let fields = read_gp_fields_slot(reader, max_bytes, refs, depth)?;
    let indexes = read_de_table_indexes(reader, max_bytes, refs, depth)?;
    Ok(DeCompactTable10_1 {
        table_version,
        dataset,
        table_short,
        table_string,
        fields,
        indexes,
    })
}

fn read_de_raster_band(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeRasterBand10_1> {
    let version = reader.read_u16_le()?;
    if version > 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DERasterBand 版本超限",
        ));
    }
    let table = read_de_compact_table(reader, max_bytes, refs, depth)?;
    let raw_short = reader.read_u16_le()?;
    let double_bits = [
        reader.read_f64_le()?.to_bits(),
        reader.read_f64_le()?.to_bits(),
    ];
    let longs = [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32];
    let v3 = (version == 3)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    let no_data = if version >= 2 {
        RasterBandNoData10_1::Variant(read_variant_object_stream_value_10_1(
            reader, max_bytes, refs, depth,
        )?)
    } else {
        RasterBandNoData10_1::Legacy(reader.read_bytes(24)?.try_into().unwrap())
    };
    let trailing_longs = [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32];
    let extent = read_surround_envelope_slot(reader, max_bytes, refs, "DERasterBand 范围")?;
    let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
    Ok(DeRasterBand10_1 {
        version,
        table,
        raw_short,
        double_bits,
        longs,
        v3,
        no_data,
        trailing_longs,
        extent,
        spatial_reference,
    })
}

fn read_raster_storage_def_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<RasterStorageDef10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == RASTER_STORAGE_DEF_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: RASTER_STORAGE_DEF_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 4 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "RasterStorageDef 版本超限",
                ));
            }
            let mut longs = [0; 6];
            for value in &mut longs {
                *value = reader.read_u32_le()? as i32;
            }
            let point_flag = (version >= 2)
                .then(|| reader.read_bytes(1).map(|bytes| bytes[0]))
                .transpose()?;
            let xy_bits = if point_flag.is_some_and(|flag| flag != 0) {
                Some([
                    reader.read_f64_le()?.to_bits(),
                    reader.read_f64_le()?.to_bits(),
                ])
            } else {
                None
            };
            let point = if version >= 2 {
                let entry = read_object_entry_10_1(reader, refs)?;
                let body = match entry {
                    ObjectEntry10_1::Null => None,
                    ObjectEntry10_1::BackReference { class_id, key }
                        if refs.registered_class(key).unwrap_or(class_id) == POINT_CLASS_ID =>
                    {
                        None
                    }
                    ObjectEntry10_1::NewObject {
                        class_id: POINT_CLASS_ID,
                        ..
                    } => Some(Box::new(read_point_body_10_1(reader, max_bytes, refs)?)),
                    _ => {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "RasterStorageDef 的 IPoint 槽包含不支持的类",
                        ));
                    }
                };
                Some(RuleSlot10_1 { entry, body })
            } else {
                None
            };
            let short = (version >= 3).then(|| reader.read_u16_le()).transpose()?;
            let text = (version >= 4)
                .then(|| read_string_10_1(reader, max_bytes))
                .transpose()?;
            Some(Box::new(RasterStorageDef10_1 {
                version,
                longs,
                point_flag,
                xy_bits,
                point,
                short,
                text,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IRasterStorageDef 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_workspace(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeWorkspace10_1> {
    let version = reader.read_u16_le()?;
    if version > 5 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DEWorkspace 版本超限",
        ));
    }
    let element = read_gp_data_element(reader, max_bytes, refs, depth)?;
    let factory = if version >= 3 {
        Some((
            read_bstr_string_10_1(reader, max_bytes)?,
            [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ],
        ))
    } else {
        None
    };
    let properties = read_gp_property_set_slot(reader, max_bytes, refs)?;
    let (raw_domain_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut domains = Vec::new();
    for _ in 0..count {
        domains.push(read_domain_slot_nested_10_1(
            reader, max_bytes, refs, depth,
        )?);
    }
    let v1 = (version == 1)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    let v4 = if version >= 4 {
        Some([
            reader.read_u32_le()? as i32,
            reader.read_u32_le()? as i32,
            reader.read_u32_le()? as i32,
        ])
    } else {
        None
    };
    let v5 = (version == 5)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;
    Ok(DeWorkspace10_1 {
        version,
        element,
        factory,
        properties,
        raw_domain_count,
        domains,
        v1,
        v4,
        v5,
    })
}

fn read_de_relationship_class(
    reader: &mut Reader<'_>,
    class_id: [u8; 16],
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeRelationshipClass10_1> {
    let version = reader.read_u16_le()?;
    if version > 6 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DERelationshipClass 版本超限",
        ));
    }
    let base = if version < 5 {
        DeRelationshipBase10_1::Dataset(read_gp_de_dataset(reader, max_bytes, refs, depth)?)
    } else if class_id == DE_RELATIONSHIP_CLASS_INFO_CLASS_ID {
        DeRelationshipBase10_1::CompactTable(Box::new(read_de_compact_table(
            reader, max_bytes, refs, depth,
        )?))
    } else {
        DeRelationshipBase10_1::Table(Box::new(read_gp_de_table(reader, max_bytes, refs, depth)?))
    };
    let metadata = if version >= 2 {
        let names = [
            read_string_array_slot_10_1(reader, max_bytes, refs)?,
            read_string_array_slot_10_1(reader, max_bytes, refs)?,
        ];
        let leading = [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32];
        let strings = [
            read_string_10_1(reader, max_bytes)?,
            read_string_10_1(reader, max_bytes)?,
        ];
        let longs = [
            leading[0],
            leading[1],
            reader.read_u32_le()? as i32,
            reader.read_u32_le()? as i32,
        ];
        let shorts = [
            reader.read_u16_le()?,
            reader.read_u16_le()?,
            reader.read_u16_le()?,
        ];
        let mut raw_key_counts = [0; 2];
        let mut keys = [Vec::new(), Vec::new()];
        for (raw_count, entries) in raw_key_counts.iter_mut().zip(&mut keys) {
            let (raw, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
            *raw_count = raw;
            for _ in 0..count {
                entries.push(read_gp_relationship_key_slot(reader, max_bytes, refs)?);
            }
        }
        Some(DeRelationshipMetadata10_1 {
            names,
            longs,
            strings,
            shorts,
            raw_key_counts,
            keys,
        })
    } else {
        None
    };
    let mut rules = Vec::new();
    let raw_rule_count = if version >= 3 {
        let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        for _ in 0..count {
            rules.push(read_relationship_rule_slot(reader, max_bytes, refs)?);
        }
        Some(raw_count)
    } else {
        None
    };
    let fields = (version == 4
        || (version <= 3 && metadata.as_ref().is_some_and(|body| body.shorts[1] != 0)))
    .then(|| read_gp_fields_slot(reader, max_bytes, refs, depth))
    .transpose()?;
    let v6 = (version == 6).then(|| reader.read_u16_le()).transpose()?;
    Ok(DeRelationshipClass10_1 {
        version,
        base,
        metadata,
        raw_rule_count,
        rules,
        fields,
        v6,
    })
}

fn read_relationship_rule_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<RelationshipRule10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == RELATIONSHIP_RULE_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: RELATIONSHIP_RULE_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "RelationshipRule 版本超限",
                ));
            }
            let name = read_string_10_1(reader, max_bytes)?;
            let mut raw_longs = [0; 10];
            for value in &mut raw_longs {
                *value = reader.read_u32_le()? as i32;
            }
            Some(Box::new(RelationshipRule10_1 {
                version,
                name,
                raw_longs,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IRelationshipRule 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_relationship_key_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<GpRelationshipClassKey10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id)
                == GP_RELATIONSHIP_CLASS_KEY_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_RELATIONSHIP_CLASS_KEY_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPRelationshipClassKey 版本超限",
                ));
            }
            Some(Box::new(GpRelationshipClassKey10_1 {
                version,
                raw_long: reader.read_u32_le()? as i32,
                strings: [
                    read_string_10_1(reader, max_bytes)?,
                    read_string_10_1(reader, max_bytes)?,
                ],
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "关系键槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_geometric_network(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    version: u16,
) -> io::Result<DeGeometricNetwork10_1> {
    let metadata = if version >= 2 {
        Some((
            reader.read_u32_le()? as i32,
            read_string_10_1(reader, max_bytes)?,
            read_string_array_slot_10_1(reader, max_bytes, refs)?,
        ))
    } else {
        None
    };
    let mut rules = Vec::new();
    let mut weights = Vec::new();
    let mut associations = Vec::new();
    let raw_counts = if version >= 3 {
        let (raw_rules, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        for _ in 0..count {
            rules.push(read_connectivity_rule_slot(reader, max_bytes, refs)?);
        }
        let (raw_weights, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        for _ in 0..count {
            weights.push(read_net_weight_slot(
                reader,
                max_bytes,
                refs,
                NET_WEIGHT_CLASS_ID,
            )?);
        }
        let (raw_associations, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
        for _ in 0..count {
            associations.push(read_net_weight_slot(
                reader,
                max_bytes,
                refs,
                NET_WEIGHT_ASSOCIATION_CLASS_ID,
            )?);
        }
        Some([raw_rules, raw_weights, raw_associations])
    } else {
        None
    };
    let v4 = if version >= 4 {
        Some((
            reader.read_u16_le()?,
            [
                reader.read_f64_le()?.to_bits(),
                reader.read_f64_le()?.to_bits(),
            ],
        ))
    } else {
        None
    };
    let v5 = if version >= 5 {
        Some([reader.read_u32_le()? as i32, reader.read_u32_le()? as i32])
    } else {
        None
    };
    let v6 = (version >= 6).then(|| reader.read_u16_le()).transpose()?;
    Ok(DeGeometricNetwork10_1 {
        metadata,
        raw_counts,
        rules,
        weights,
        associations,
        v4,
        v5,
        v6,
    })
}

fn read_connectivity_rule_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<ConnectivityRule10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                refs.registered_class(key).unwrap_or(class_id),
                EDGE_CONNECTIVITY_RULE_CLASS_ID | JUNCTION_CONNECTIVITY_RULE_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (EDGE_CONNECTIVITY_RULE_CLASS_ID | JUNCTION_CONNECTIVITY_RULE_CLASS_ID),
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "网络连接规则版本超限",
                ));
            }
            let name = read_string_10_1(reader, max_bytes)?;
            let body = if class_id == EDGE_CONNECTIVITY_RULE_CLASS_ID {
                let mut longs = [0; 8];
                for value in &mut longs {
                    *value = reader.read_u32_le()? as i32;
                }
                let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 8)?;
                if count > 64 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "边连接规则条目超过原生数组容量",
                    ));
                }
                let mut pairs = Vec::new();
                for _ in 0..count {
                    pairs.push([reader.read_u32_le()? as i32, reader.read_u32_le()? as i32]);
                }
                ConnectivityRule10_1::Edge {
                    version,
                    name,
                    longs,
                    raw_count,
                    pairs,
                }
            } else {
                let mut longs = [0; 10];
                for value in &mut longs {
                    *value = reader.read_u32_le()? as i32;
                }
                ConnectivityRule10_1::Junction {
                    version,
                    name,
                    longs,
                    short: reader.read_u16_le()?,
                }
            };
            Some(Box::new(body))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IConnectivityRule 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_net_weight_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    expected_class: [u8; 16],
) -> io::Result<RuleSlot10_1<NetWeight10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == expected_class =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected_class => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "网络权重对象版本超限",
                ));
            }
            let raw_long = reader.read_u32_le()? as i32;
            let first = read_bstr_string_10_1(reader, max_bytes)?;
            let value = if expected_class == NET_WEIGHT_CLASS_ID {
                NetWeight10_1::Weight {
                    version,
                    raw_long,
                    name: first,
                    trailing_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
                }
            } else {
                NetWeight10_1::Association {
                    version,
                    raw_long,
                    strings: [first, read_bstr_string_10_1(reader, max_bytes)?],
                }
            };
            Some(Box::new(value))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "网络权重槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_topology_rule_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<TopologyRule10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == TOPOLOGY_RULE_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: TOPOLOGY_RULE_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            let base_version = reader.read_u16_le()?;
            if version > 1 || base_version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "TopologyRule 或 Rule 版本超限",
                ));
            }
            let base_name = read_string_10_1(reader, max_bytes)?;
            let base_longs = [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32];
            let strings = [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ];
            let mut longs = [0; 4];
            for value in &mut longs {
                *value = reader.read_u32_le()? as i32;
            }
            let mut shorts = [0; 5];
            for value in &mut shorts {
                *value = reader.read_u16_le()?;
            }
            Some(Box::new(TopologyRule10_1 {
                version,
                base_version,
                base_name,
                base_longs,
                strings,
                longs,
                shorts,
                trailing_long: reader.read_u32_le()? as i32,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "ITopologyRule 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_de_table(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeTable10_1> {
    let version = reader.read_u16_le()?;
    if version > 4 {
        return Err(io::Error::new(ErrorKind::InvalidData, "DETable 版本超限"));
    }
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let raw_short = reader.read_u16_le()?;

    // 标准 DETable 构造函数启用扩展字段，流中没有标志字节。
    let strings = [
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
    ];
    let raw_long = reader.read_u32_le()? as i32;
    let string_array = read_string_array_slot_10_1(reader, max_bytes, refs)?;
    let properties = read_gp_property_set_slot(reader, max_bytes, refs)?;
    let (raw_subtype_count, subtype_count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut subtypes = Vec::new();
    for _ in 0..subtype_count {
        subtypes.push(read_de_table_subtype(reader, max_bytes, refs, depth)?);
    }
    let (raw_membership_count, membership_count) =
        read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut memberships = Vec::new();
    for _ in 0..membership_count {
        memberships.push(read_de_table_membership(reader, max_bytes, refs)?);
    }
    let v2 = if version >= 2 {
        Some((
            reader.read_u16_le()?,
            [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ],
        ))
    } else {
        None
    };
    let v3 = if version >= 3 {
        Some((
            reader.read_u16_le()?,
            [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ],
        ))
    } else {
        None
    };
    let v4 = (version >= 4).then(|| reader.read_u16_le()).transpose()?;
    let fields = read_gp_fields_slot(reader, max_bytes, refs, depth)?;
    let indexes = read_de_table_indexes(reader, max_bytes, refs, depth)?;
    Ok(DeTable10_1 {
        version,
        dataset,
        raw_short,
        strings,
        raw_long,
        string_array,
        properties,
        raw_subtype_count,
        subtypes,
        raw_membership_count,
        memberships,
        v2,
        v3,
        v4,
        fields,
        indexes,
    })
}

fn read_de_mosaic_dataset(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeMosaicDataset10_1> {
    let version = reader.read_u16_le()? as i16;
    if version > 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DEMosaicDataset 版本超限",
        ));
    }
    let geo_dataset_version = read_de_layer_version(reader, 1, "DEGeoDataset")?;
    let dataset = read_gp_de_dataset(reader, max_bytes, refs, depth)?;
    let extent = read_surround_envelope_slot(reader, max_bytes, refs, "DEMosaicDataset 范围")?;
    let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
    let value = read_padded_variant_object_stream_nested_10_1(reader, max_bytes, refs, depth)?;
    let raw_short = reader.read_u16_le()?;
    let raw_long = (version >= 3)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    let catalog = if version >= 2 {
        let mut header = reader.clone();
        let entry = read_object_entry_10_1(&mut header, &mut refs.clone())?;
        let supported = match entry {
            ObjectEntry10_1::Null => true,
            ObjectEntry10_1::NewObject { class_id, .. } => class_id == DE_RASTER_CATALOG_CLASS_ID,
            ObjectEntry10_1::BackReference { class_id, key } => {
                refs.registered_class(key).unwrap_or(class_id) == DE_RASTER_CATALOG_CLASS_ID
            }
        };
        if !supported {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "DEMosaicDataset 的 IDERasterCatalog 类尚未支持",
            ));
        }
        Some(read_gp_data_element_child(reader, max_bytes, refs, depth)?)
    } else {
        None
    };
    let names = read_string_array_slot_10_1(reader, max_bytes, refs)?;
    let table = (version >= 4)
        .then(|| read_gp_de_table_slot(reader, max_bytes, refs, depth))
        .transpose()?;
    Ok(DeMosaicDataset10_1 {
        version,
        geo_dataset_version,
        dataset,
        extent,
        spatial_reference,
        value,
        raw_short,
        raw_long,
        catalog,
        names,
        table,
    })
}

fn read_gp_de_dataset(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DeDataset10_1> {
    let dataset_version = reader.read_u16_le()?;
    if dataset_version > 2 {
        return Err(io::Error::new(ErrorKind::InvalidData, "DEDataset 版本超限"));
    }
    let element = read_gp_data_element(reader, max_bytes, refs, depth)?;
    let dataset_long = reader.read_u32_le()? as i32;
    let dataset_shorts = [reader.read_u16_le()?, reader.read_u16_le()?];
    let dataset_longs = [dataset_long, reader.read_u32_le()? as i32];
    let dataset_string = (dataset_version >= 2)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;
    Ok(DeDataset10_1 {
        dataset_version,
        element,
        dataset_longs,
        dataset_shorts,
        dataset_string,
    })
}

fn read_gp_data_element(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DataElement10_1> {
    let element_version = reader.read_u16_le()?;
    let gp_value_version = reader.read_u16_le()?;
    if element_version > 2 || gp_value_version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "DataElement 或 GPValue 版本超限",
        ));
    }
    let gp_value_short = reader.read_u16_le()?;
    let element_strings = [
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
    ];
    let element_shorts = [
        reader.read_u16_le()?,
        reader.read_u16_le()?,
        reader.read_u16_le()?,
    ];
    let element_legacy_xml = (element_version == 2 && element_shorts[2] == u16::MAX)
        .then(|| read_bstr_string_10_1(reader, max_bytes))
        .transpose()?;
    let (raw_child_count, child_count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
    let mut children = Vec::new();
    for _ in 0..child_count {
        children.push(read_gp_data_element_child(reader, max_bytes, refs, depth)?);
    }

    Ok(DataElement10_1 {
        element_version,
        gp_value_version,
        gp_value_short,
        element_strings,
        element_shorts,
        element_legacy_xml,
        raw_child_count,
        children,
    })
}

fn read_de_table_membership(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<GpControllerMembership10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                refs.registered_class(key).unwrap_or(class_id),
                GP_TOPOLOGY_MEMBERSHIP_CLASS_ID | GP_GEOMETRIC_NETWORK_MEMBERSHIP_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (GP_TOPOLOGY_MEMBERSHIP_CLASS_ID | GP_GEOMETRIC_NETWORK_MEMBERSHIP_CLASS_ID),
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GP 控制器成员对象版本超限",
                ));
            }
            let value = if class_id == GP_TOPOLOGY_MEMBERSHIP_CLASS_ID {
                GpControllerMembership10_1::Topology {
                    version,
                    name: read_string_10_1(reader, max_bytes)?,
                    value_bits: reader.read_f64_le()?.to_bits(),
                    raw_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
                    raw_short: reader.read_u16_le()?,
                }
            } else {
                GpControllerMembership10_1::GeometricNetwork {
                    version,
                    strings: [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ],
                    raw_long: reader.read_u32_le()? as i32,
                }
            };
            Some(Box::new(value))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IGPControllerMembership 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_table_subtype(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<GpSubtype10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == GP_SUBTYPE_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_SUBTYPE_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(ErrorKind::InvalidData, "GPSubtype 版本超限"));
            }
            let name = read_string_10_1(reader, max_bytes)?;
            let raw_code = reader.read_u32_le()? as i32;
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 10)?;
            let mut fields = Vec::new();
            for _ in 0..count {
                fields.push(GpSubtypeField10_1 {
                    strings: [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ],
                    default: read_variant_object_stream_value_10_1(reader, max_bytes, refs, depth)?,
                });
            }
            Some(Box::new(GpSubtype10_1 {
                version,
                name,
                raw_code,
                raw_count,
                fields,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IGPSubtype 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_table_indexes(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<DeTableIndexes10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == INDEXES_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: INDEXES_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(ErrorKind::InvalidData, "Indexes 版本超限"));
            }
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 16)?;
            let mut entries = Vec::new();
            for _ in 0..count {
                entries.push(read_de_table_index(reader, max_bytes, refs, depth)?);
            }
            Some(Box::new(DeTableIndexes10_1 {
                version,
                raw_count,
                entries,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IIndexes 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_de_table_index(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<DeTableIndex10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == INDEX_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: INDEX_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            let gp_value_version = reader.read_u16_le()?;
            if version > 1 || gp_value_version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Index 或其 GPValue 版本超限",
                ));
            }
            Some(Box::new(DeTableIndex10_1 {
                version,
                gp_value_version,
                gp_value_short: reader.read_u16_le()?,
                name: read_string_10_1(reader, max_bytes)?,
                flags: reader.read_bytes(3)?.try_into().unwrap(),
                fields: read_gp_fields_slot(reader, max_bytes, refs, depth)?,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "IIndex 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_property_set_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<PropertySetObjectStream10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if table.registered_class(key).unwrap_or(class_id) == PROPERTY_SET_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: PROPERTY_SET_CLASS_ID,
            ..
        } => Some(Box::new(read_property_set_object_stream_10_1(
            reader, max_bytes, table,
        )?)),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GP 对象的 IPropertySet 槽包含不支持的类",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_raster_environment(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
    class_id: [u8; 16],
) -> io::Result<GpRasterEnvironment10_1> {
    use GpRasterEnvironmentParameters10_1 as Parameters;

    let name_version = if class_id == GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID {
        2
    } else {
        1
    };
    let legacy_name =
        read_gp_legacy_name(reader, max_bytes, table, depth, version == name_version)?;
    let read_longs = |reader: &mut Reader<'_>| -> io::Result<[i32; 2]> {
        Ok([reader.read_u32_le()? as i32, reader.read_u32_le()? as i32])
    };
    let parameters = match class_id {
        GP_RASTER_GDB_ENV_COMPRESSION_CLASS_ID => Parameters::Compression(read_longs(reader)?),
        GP_RASTER_GDB_ENV_TILE_SIZE_CLASS_ID => Parameters::TileSize(read_longs(reader)?),
        GP_RASTER_GDB_ENV_STATISTICS_CLASS_ID => Parameters::Statistics {
            raw_short: reader.read_u16_le()?,
            raw_longs: read_longs(reader)?,
            ignore_values: if version <= 1 {
                GpRasterStatisticsIgnoreValues10_1::LegacyBytes(
                    reader.read_bytes(4)?.try_into().unwrap(),
                )
            } else {
                GpRasterStatisticsIgnoreValues10_1::Text(read_bstr_string_10_1(reader, max_bytes)?)
            },
        },
        _ => Parameters::Pyramid {
            raw_short: reader.read_u16_le()?,
            raw_longs: read_longs(reader)?,
            version_three_longs: (version >= 3).then(|| read_longs(reader)).transpose()?,
            version_four_short: (version >= 4).then(|| reader.read_u16_le()).transpose()?,
        },
    };
    Ok(GpRasterEnvironment10_1 {
        legacy_name,
        parameters,
    })
}

fn read_gp_legacy_name(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    has_legacy_name: bool,
) -> io::Result<Option<Box<BookmarkNameEntry10_1>>> {
    if !has_legacy_name {
        return Ok(None);
    }
    if depth >= MAX_BOOKMARK_NAME_NESTING {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GP 值的旧 IName 槽嵌套层数超限",
        ));
    }
    read_bookmark_name_slot_10_1(reader, max_bytes, table, depth + 1)
        .map(|name| Some(Box::new(name)))
}

fn read_gp_sa_time_configuration(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<TimeConfiguration10_1>> {
    use TimeConfigurationParameters10_1 as Parameters;

    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if table.registered_class(key).unwrap_or(class_id) == TIME_CONFIGURATION_CLASS_ID =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: TIME_CONFIGURATION_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "日照时间配置版本超出 10.1 上限",
                ));
            }
            let kind = reader.read_u32_le()? as i32;
            let parameters = match kind {
                2 => Parameters::LongAndDoubles {
                    raw_long: reader.read_u32_le()? as i32,
                    double_bits: [
                        reader.read_f64_le()?.to_bits(),
                        reader.read_f64_le()?.to_bits(),
                    ],
                },
                3 => Parameters::Longs([
                    reader.read_u32_le()? as i32,
                    reader.read_u32_le()? as i32,
                    reader.read_u32_le()? as i32,
                ]),
                4 => Parameters::Long(reader.read_u32_le()? as i32),
                _ => Parameters::None,
            };
            Some(Box::new(TimeConfiguration10_1 {
                version,
                kind,
                parameters,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "日照时间配置槽的对象类不受支持",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_path_distance_factor_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    vertical: bool,
) -> io::Result<RuleSlot10_1<PathDistanceFactor10_1>> {
    use PathDistanceFactorParameters10_1 as Parameters;

    let expected_class = if vertical {
        PATH_DISTANCE_VERTICAL_FACTOR_CLASS_ID
    } else {
        PATH_DISTANCE_HORIZONTAL_FACTOR_CLASS_ID
    };
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if table.registered_class(key).unwrap_or(class_id) == expected_class =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected_class => {
            let version = reader.read_u16_le()?;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "距离因子对象版本超出 10.1 上限",
                ));
            }
            let kind = reader.read_u32_le()? as i32;
            // 分支依据 RasterDistanceOp 的两个 Load 跳转表，保留原始 double 位模式。
            let parameters = match (vertical, kind) {
                (false, 5) | (true, 4) => Parameters::Path(read_string_10_1(reader, max_bytes)?),
                _ => {
                    let count = match (vertical, kind) {
                        (false, 1 | 2) => 2,
                        (false, 3 | 4) | (true, 1 | 7 | 8) => 3,
                        (true, 2 | 3 | 5 | 6 | 9 | 10) => 4,
                        _ => 0,
                    };
                    if count == 0 {
                        Parameters::None
                    } else {
                        Parameters::Numbers(
                            (0..count)
                                .map(|_| reader.read_f64_le().map(f64::to_bits))
                                .collect::<io::Result<_>>()?,
                        )
                    }
                }
            };
            Some(Box::new(PathDistanceFactor10_1 {
                version,
                kind,
                parameters,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "距离因子槽的对象类不受支持",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn read_gp_sa_neighborhood(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpSaNeighborhood10_1> {
    use GpSaNeighborhoodParameters10_1 as Parameters;

    let legacy_name = read_gp_legacy_name(reader, max_bytes, table, depth, version == 1)?;
    let storage_kind = reader.read_u32_le()? as i32;
    let shape_kind = reader.read_u32_le()? as i32;
    let raw_long = reader.read_u32_le()? as i32;
    let parameters = match shape_kind {
        1..=4 => {
            let count = match shape_kind {
                2 => 1,
                4 => 3,
                _ => 2,
            };
            let dimensions = (0..count)
                .map(|_| reader.read_f64_le().map(f64::to_bits))
                .collect::<io::Result<_>>()?;
            Parameters::Dimensions(dimensions)
        }
        5 | 6 if storage_kind == 1 => Parameters::Embedded {
            raw_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
            values: Box::new(read_padded_variant_object_stream_nested_10_1(
                reader, max_bytes, table, depth,
            )?),
        },
        5 | 6 => Parameters::Path(read_string_10_1(reader, max_bytes)?),
        _ => Parameters::None,
    };
    Ok(GpSaNeighborhood10_1 {
        legacy_name,
        storage_kind,
        shape_kind,
        raw_long,
        parameters,
    })
}

fn read_gp_sa_weighted_overlay(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpSaWeightedOverlayTable10_1> {
    if depth >= MAX_BOOKMARK_NAME_NESTING {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GPSAWeightedOverlayTable 嵌套层数超限",
        ));
    }
    let depth = depth + 1;
    let legacy = version == 1;
    let scale = if legacy {
        GpSaOverlayScale10_1::Legacy {
            name: Box::new(read_bookmark_name_slot_10_1(
                reader, max_bytes, table, depth,
            )?),
            limits_bits: [
                reader.read_f64_le()?.to_bits(),
                reader.read_f64_le()?.to_bits(),
            ],
            raw_long: reader.read_u32_le()? as i32,
        }
    } else {
        GpSaOverlayScale10_1::Object(read_gp_overlay_value_slot(
            reader,
            max_bytes,
            table,
            depth,
            &[GP_EVALUATION_SCALE_CLASS_ID],
        )?)
    };
    let (raw_count, count) =
        read_gp_coded_value_count(reader, max_bytes, if legacy { 52 } else { 64 })?;
    let mut rows = Vec::new();
    for _ in 0..count {
        let values = [
            read_gp_coded_value_slot(reader, max_bytes, table, depth)?,
            read_gp_coded_value_slot(reader, max_bytes, table, depth)?,
            read_gp_coded_value_slot(reader, max_bytes, table, depth)?,
        ];
        let remap = if legacy {
            let (raw_count, count) = read_gp_coded_value_count(reader, max_bytes, 12)?;
            let mut rows = Vec::new();
            for _ in 0..count {
                rows.push(read_gp_string_remap_row(reader, max_bytes)?);
            }
            GpSaOverlayRemap10_1::Legacy { raw_count, rows }
        } else {
            GpSaOverlayRemap10_1::Object(read_gp_overlay_value_slot(
                reader,
                max_bytes,
                table,
                depth,
                &[
                    GP_SA_REMAP_CLASS_ID,
                    GP_SA_NUMBER_REMAP_CLASS_ID,
                    GP_SA_STRING_REMAP_CLASS_ID,
                ],
            )?)
        };
        rows.push(GpSaOverlayRow10_1 { values, remap });
    }
    Ok(GpSaWeightedOverlayTable10_1 {
        scale,
        raw_count,
        rows,
    })
}

fn read_gp_overlay_value_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    classes: &[[u8; 16]],
) -> io::Result<RuleSlot10_1<GpCodedValueBody10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let class = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key } => {
            Some(table.registered_class(key).unwrap_or(class_id))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => Some(class_id),
    };
    if class.is_some_and(|class| !classes.contains(&class)) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "加权叠加表的比例尺或重分类槽包含不支持的类",
        ));
    }
    read_gp_coded_value_entry(reader, max_bytes, table, depth, entry)
}

fn read_gp_sa_semi_variogram(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
) -> io::Result<GpSaSemiVariogram10_1> {
    let legacy_name = read_gp_legacy_name(reader, max_bytes, table, depth, version == 1)?;
    let variogram_type = reader.read_u32_le()? as i32;
    let lag = if version == 3 {
        GpSaSemiVariogramLag10_1::Number(reader.read_f64_le()?.to_bits())
    } else {
        GpSaSemiVariogramLag10_1::LegacyText(read_gp_narrow_text(reader, max_bytes)?)
    };
    Ok(GpSaSemiVariogram10_1 {
        legacy_name,
        variogram_type,
        lag,
        range_bits: reader.read_f64_le()?.to_bits(),
        partial_sill_bits: reader.read_f64_le()?.to_bits(),
        nugget_bits: reader.read_f64_le()?.to_bits(),
    })
}

fn read_gp_narrow_text(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<Vec<u8>> {
    let length = reader.read_u32_le()? as usize;
    if length > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "空间分析值的窄字符文本长度超限",
        ));
    }
    Ok(reader.read_bytes(length)?.to_vec())
}

fn read_gp_string_remap_row(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<GpSaRemapRow10_1> {
    Ok(GpSaRemapRow10_1::String {
        source_bytes: read_gp_narrow_text(reader, max_bytes)?,
        raw_long: reader.read_u32_le()? as i32,
        text_bytes: read_gp_narrow_text(reader, max_bytes)?,
    })
}

fn read_gp_sa_remap(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    version: u16,
    class_id: [u8; 16],
) -> io::Result<GpSaRemap10_1> {
    let legacy_name = read_gp_legacy_name(reader, max_bytes, table, depth, version == 1)?;
    let raw_long = reader.read_u32_le()? as i32;
    let raw_count = reader.read_u32_le()? as i32;
    let version_three_fields = if version >= 3 {
        Some((reader.read_u16_le()?, reader.read_u32_le()? as i32))
    } else {
        None
    };
    let numeric = class_id == GP_SA_NUMBER_REMAP_CLASS_ID
        || (class_id == GP_SA_REMAP_CLASS_ID && raw_long == 1);
    let count = raw_count.max(0) as usize;
    let min_row_bytes = if numeric { 24 } else { 12 };
    if count > reader.remaining().min(max_bytes) / min_row_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "空间分析重分类行数超限",
        ));
    }

    let mut rows = Vec::new();
    for _ in 0..count {
        let row = if numeric {
            GpSaRemapRow10_1::Number {
                range_bits: [
                    reader.read_f64_le()?.to_bits(),
                    reader.read_f64_le()?.to_bits(),
                ],
                raw_long: reader.read_u32_le()? as i32,
                text_bytes: read_gp_narrow_text(reader, max_bytes)?,
            }
        } else {
            read_gp_string_remap_row(reader, max_bytes)?
        };
        rows.push(row);
    }
    let version_four_fields = if class_id == GP_SA_REMAP_CLASS_ID && version >= 4 {
        Some((
            reader.read_u16_le()?,
            [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
        ))
    } else {
        None
    };
    Ok(GpSaRemap10_1 {
        legacy_name,
        raw_long,
        raw_count,
        version_three_fields,
        version_four_fields,
        rows,
    })
}

fn read_gp_value_table(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<GpCodedValue10_1> {
    if depth >= MAX_BOOKMARK_NAME_NESTING {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GPValueTable 嵌套层数超限",
        ));
    }
    let raw_column_count = reader.read_u32_le()? as i32;
    let columns = raw_column_count.max(0) as usize;
    if columns > reader.remaining().min(max_bytes) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GPValueTable 列数超限",
        ));
    }
    let mut column_types = Vec::new();
    for _ in 0..columns {
        let name = read_bookmark_name_slot_10_1(reader, max_bytes, table, depth + 1)?;
        if !is_gp_data_type_name(&name) {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPValueTable 的列类型名称类未支持",
            ));
        }
        column_types.push(name);
    }
    let raw_row_count = reader.read_u32_le()? as i32;
    let rows = raw_row_count.max(0) as usize;
    let cell_count = columns
        .checked_mul(rows)
        .filter(|&count| count <= reader.remaining().min(max_bytes) / 16)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "GPValueTable 单元格数量超限"))?;
    let mut cells = Vec::new();
    for _ in 0..cell_count {
        cells.push(read_gp_coded_value_slot(
            reader,
            max_bytes,
            table,
            depth + 1,
        )?);
    }
    Ok(GpCodedValue10_1::ValueTable {
        raw_column_count,
        column_types,
        raw_row_count,
        cells,
    })
}

fn read_gp_topo_domain_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    name_depth: usize,
) -> io::Result<RuleSlot10_1<GpTopoDomain10_1>> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if matches!(
                table.registered_class(key).unwrap_or(class_id),
                GP_NUMERIC_DOMAIN_CLASS_ID
                    | GP_FILE_DOMAIN_CLASS_ID
                    | GP_RANGE_DOMAIN_CLASS_ID
                    | GP_WORKSPACE_DOMAIN_CLASS_ID
                    | GP_DATASET_DOMAIN_CLASS_ID
                    | GP_FEATURE_CLASS_DOMAIN_CLASS_ID
                    | GP_ARC_INFO_ITEM_DOMAIN_CLASS_ID
                    | GP_FIELD_DOMAIN_CLASS_ID
                    | GP_MEASURE_FIELDS_DOMAIN_CLASS_ID
                    | GP_ROUTE_ID_FIELDS_DOMAIN_CLASS_ID
                    | GP_QUERY_TABLE_IDS_DOMAIN_CLASS_ID
                    | GP_VIRTUAL_TABLE_DOMAIN_CLASS_ID
                    | GP_ROUTE_MEASURE_EVENT_DOMAIN_CLASS_ID
                    | GP_ROUTE_DOMAIN_CLASS_ID
                    | GP_LAYERS_AND_TABLES_DOMAIN_CLASS_ID
                    | GP_SA_GEO_DATA_DOMAIN_CLASS_ID
                    | GP_SA_TEXT_TABLE_NAME_DOMAIN_CLASS_ID
                    | GP_SA_NAME_DOMAIN_CLASS_ID
                    | GP_SA_REMAP_DOMAIN_CLASS_ID
                    | GP_SA_NEIGHBORHOOD_DOMAIN_CLASS_ID
                    | GP_RA_FIELD_DOMAIN_CLASS_ID
                    | GP_REPRESENTATION_LAYER_DOMAIN_CLASS_ID
                    | GP_COMPOSITE_DOMAIN_CLASS_ID
                    | GP_CODED_VALUE_DOMAIN_CLASS_ID
                    | GP_FIELD_INFO_DOMAIN_CLASS_ID
                    | GP_DATA_ELEMENT_DOMAIN_CLASS_ID
                    | GP_CAD_ANNO_DOMAIN_CLASS_ID
                    | GP_COV_ANNO_DOMAIN_CLASS_ID
                    | GP_COVERAGE_DOMAIN_CLASS_ID
                    | GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID
                    | GP_GDB_ANNO_DOMAIN_CLASS_ID
                    | GP_TABLES_DOMAIN_CLASS_ID
                    | GP_QUERY_TABLE_FIELDS_DOMAIN_CLASS_ID
            ) =>
        {
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_NUMERIC_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::Numeric(GpNumericDomain10_1 {
            version: read_gp_name_version(reader, 1, "GPNumericDomain")?,
            first_short: reader.read_u16_le()?,
            first_double_bits: reader.read_f64_le()?.to_bits(),
            middle_shorts: [reader.read_u16_le()?, reader.read_u16_le()?],
            second_double_bits: reader.read_f64_le()?.to_bits(),
        }))),
        ObjectEntry10_1::NewObject {
            class_id: GP_GDB_ANNO_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::GdbAnnotation {
            version: read_gp_name_version(reader, 0, "GPGdbAnnoDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_COVERAGE_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPCoverageDomain")?;
            let entry = read_object_entry_10_1(reader, table)?;
            let body = match entry {
                ObjectEntry10_1::Null => None,
                ObjectEntry10_1::NewObject {
                    class_id: GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID,
                    ..
                } => Some(Box::new(read_gp_coverage_feature_class_domain(
                    reader, max_bytes,
                )?)),
                ObjectEntry10_1::BackReference { class_id, key }
                    if table.registered_class(key).unwrap_or(class_id)
                        == GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID =>
                {
                    None
                }
                _ => {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "GPCoverageDomain 的 IGPCoverageFeatureClassDomain 类未支持",
                    ));
                }
            };
            Some(Box::new(GpTopoDomain10_1::Coverage {
                version,
                feature_class: RuleSlot10_1 { entry, body },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_COVERAGE_FEATURE_CLASS_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::CoverageFeatureClass(
            read_gp_coverage_feature_class_domain(reader, max_bytes)?,
        ))),
        ObjectEntry10_1::NewObject {
            class_id: GP_COV_ANNO_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::CoverageAnnotation {
            version: read_gp_name_version(reader, 0, "GPCovAnnoDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_CAD_ANNO_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::CadAnnotation {
            version: read_gp_name_version(reader, 0, "GPCadAnnoDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_CODED_VALUE_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::CodedValue(
            read_gp_coded_value_domain(reader, max_bytes, table, name_depth)?,
        ))),
        ObjectEntry10_1::NewObject {
            class_id: GP_FIELD_INFO_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPFieldInfoDomain")?;
            let entry = read_object_entry_10_1(reader, table)?;
            let body = match entry {
                ObjectEntry10_1::Null => None,
                ObjectEntry10_1::BackReference { class_id, key }
                    if table.registered_class(key).unwrap_or(class_id)
                        == GP_CODED_VALUE_DOMAIN_CLASS_ID =>
                {
                    None
                }
                ObjectEntry10_1::NewObject {
                    class_id: GP_CODED_VALUE_DOMAIN_CLASS_ID,
                    ..
                } => Some(Box::new(read_gp_coded_value_domain(
                    reader, max_bytes, table, name_depth,
                )?)),
                _ => {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "GPFieldInfoDomain 的 IGPCodedValueDomain 类未支持",
                    ));
                }
            };
            Some(Box::new(GpTopoDomain10_1::FieldInfo {
                version,
                coded_values: RuleSlot10_1 { entry, body },
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_COMPOSITE_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPCompositeDomain")?;
            let raw_count = reader.read_u32_le()? as i32;
            let count = raw_count.max(0) as usize;
            if count > reader.remaining().min(max_bytes) / 16 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPCompositeDomain 子域数量超限",
                ));
            }
            if count > 0 && name_depth == MAX_BOOKMARK_NAME_NESTING {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPCompositeDomain 嵌套层数超限",
                ));
            }
            let mut domains = Vec::new();
            for _ in 0..count {
                domains.push(read_gp_topo_domain_slot_10_1(
                    reader,
                    max_bytes,
                    table,
                    name_depth + 1,
                )?);
            }
            Some(Box::new(GpTopoDomain10_1::Composite {
                version,
                raw_count,
                domains,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_DATA_ELEMENT_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "DataElementDomain")?;
            let (raw_count, strings) =
                read_gp_domain_strings(reader, max_bytes, "DataElementDomain")?;
            Some(Box::new(GpTopoDomain10_1::DataElement {
                version,
                raw_count,
                strings,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_REPRESENTATION_LAYER_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 2, "GPRepresentationLayerDomain")?;
            let v2_list = (version == 2)
                .then(|| read_gp_domain_longs(reader, max_bytes, "GPRepresentationLayerDomain"))
                .transpose()?;
            Some(Box::new(GpTopoDomain10_1::RepresentationLayer {
                version,
                v2_list,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_RA_FIELD_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::RaField(GpRaFieldDomain10_1 {
            version: read_gp_name_version(reader, 1, "GPRAFieldDomain")?,
            long_list: read_gp_domain_longs(reader, max_bytes, "GPRAFieldDomain")?,
            shorts: [reader.read_u16_le()?, reader.read_u16_le()?],
            string_lists: [
                read_gp_domain_strings(reader, max_bytes, "GPRAFieldDomain")?,
                read_gp_domain_strings(reader, max_bytes, "GPRAFieldDomain")?,
            ],
        }))),
        ObjectEntry10_1::NewObject {
            class_id: GP_SA_NEIGHBORHOOD_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPSANeighborhoodDomain")?;
            let (raw_count, values) =
                read_gp_domain_longs(reader, max_bytes, "GPSANeighborhoodDomain")?;
            Some(Box::new(GpTopoDomain10_1::SaNeighborhood {
                version,
                raw_count,
                values,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_SA_REMAP_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPSARemapDomain")?;
            let (raw_count, values) = read_gp_domain_longs(reader, max_bytes, "GPSARemapDomain")?;
            Some(Box::new(GpTopoDomain10_1::SaRemap {
                version,
                raw_count,
                values,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_SA_NAME_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::SaName {
            version: read_gp_name_version(reader, 1, "GPSANameDomain")?,
            long: reader.read_u32_le()? as i32,
            short: reader.read_u16_le()?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_SA_TEXT_TABLE_NAME_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::SaTextTableName {
            version: read_gp_name_version(reader, 1, "GPSATextTableNameDomain")?,
            string: read_string_10_1(reader, max_bytes)?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_SA_GEO_DATA_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPSAGeoDataDomain")?;
            let raw_name_count = reader.read_u32_le()? as i32;
            let count = raw_name_count.max(0) as usize;
            if count > reader.remaining().min(max_bytes) / 16 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPSAGeoDataDomain 名称数量超限",
                ));
            }
            let mut names = Vec::new();
            for _ in 0..count {
                names.push(read_bookmark_name_slot_10_1(
                    reader, max_bytes, table, name_depth,
                )?);
            }
            let first_list = read_gp_domain_longs(reader, max_bytes, "GPSAGeoDataDomain")?;
            let second_list = read_gp_domain_longs(reader, max_bytes, "GPSAGeoDataDomain")?;
            let trailing_short = reader.read_u16_le()?;
            let trailing_string = read_string_10_1(reader, max_bytes)?;
            Some(Box::new(GpTopoDomain10_1::SaGeoData(
                GpSaGeoDataDomain10_1 {
                    version,
                    raw_name_count,
                    names,
                    first_list,
                    second_list,
                    trailing_short,
                    trailing_string,
                },
            )))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_TABLES_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::Tables {
            version: read_gp_name_version(reader, 1, "GPTablesDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_LAYERS_AND_TABLES_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::LayersAndTables {
            version: read_gp_name_version(reader, 1, "GPLayersAndTablesDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_ROUTE_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::Route {
            version: read_gp_name_version(reader, 1, "GPRouteDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_ROUTE_MEASURE_EVENT_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::RouteMeasureEvent {
            version: read_gp_name_version(reader, 1, "GPRouteMeasureEventDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_VIRTUAL_TABLE_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::VirtualTable {
            version: read_gp_name_version(reader, 1, "GPVirtualTableDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_QUERY_TABLE_IDS_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::QueryTableIds {
            version: read_gp_name_version(reader, 1, "GPQueryTableIDsDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_QUERY_TABLE_FIELDS_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::QueryTableFields {
            version: read_gp_name_version(reader, 1, "GPQueryTableFieldsDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_ROUTE_ID_FIELDS_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::RouteIdFields {
            version: read_gp_name_version(reader, 1, "GPRouteIDFieldsDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_MEASURE_FIELDS_DOMAIN_CLASS_ID,
            ..
        } => Some(Box::new(GpTopoDomain10_1::MeasureFields {
            version: read_gp_name_version(reader, 1, "GPMeasureFieldsDomain")?,
        })),
        ObjectEntry10_1::NewObject {
            class_id: GP_FIELD_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 4, "GPFieldDomain")?;
            let (raw_count, values) = read_gp_domain_longs(reader, max_bytes, "GPFieldDomain")?;
            let shorts = [reader.read_u16_le()?, reader.read_u16_le()?];
            let v2_strings = (version >= 2)
                .then(|| read_gp_domain_strings(reader, max_bytes, "GPFieldDomain"))
                .transpose()?;
            let v3_strings = (version >= 3)
                .then(|| read_gp_domain_strings(reader, max_bytes, "GPFieldDomain"))
                .transpose()?;
            let v4_short = (version >= 4).then(|| reader.read_u16_le()).transpose()?;
            Some(Box::new(GpTopoDomain10_1::Field(GpFieldDomain10_1 {
                version,
                raw_count,
                values,
                shorts,
                v2_strings,
                v3_strings,
                v4_short,
            })))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_ARC_INFO_ITEM_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPArcInfoItemDomain")?;
            let (raw_count, values) =
                read_gp_domain_longs(reader, max_bytes, "GPArcInfoItemDomain")?;
            let trailing_long = reader.read_u32_le()? as i32;
            let trailing_shorts = [
                reader.read_u16_le()?,
                reader.read_u16_le()?,
                reader.read_u16_le()?,
                reader.read_u16_le()?,
            ];
            Some(Box::new(GpTopoDomain10_1::ArcInfoItem {
                version,
                raw_count,
                values,
                trailing_long,
                trailing_shorts,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_FEATURE_CLASS_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 3, "GPFeatureClassDomain")?;
            let (raw_count, values) =
                read_gp_domain_longs(reader, max_bytes, "GPFeatureClassDomain")?;
            let shorts = [reader.read_u16_le()?, reader.read_u16_le()?];
            let v2_shorts = (version >= 2)
                .then(|| -> io::Result<_> {
                    Ok([
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                        reader.read_u16_le()?,
                    ])
                })
                .transpose()?;
            let v3_list = (version >= 3)
                .then(|| read_gp_domain_longs(reader, max_bytes, "GPFeatureClassDomain"))
                .transpose()?;
            Some(Box::new(GpTopoDomain10_1::FeatureClass {
                version,
                raw_count,
                values,
                shorts,
                v2_shorts,
                v3_list,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_DATASET_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPDatasetDomain")?;
            let (raw_count, values) = read_gp_domain_longs(reader, max_bytes, "GPDatasetDomain")?;
            let trailing_shorts = [
                reader.read_u16_le()?,
                reader.read_u16_le()?,
                reader.read_u16_le()?,
                reader.read_u16_le()?,
            ];
            Some(Box::new(GpTopoDomain10_1::Dataset {
                version,
                raw_count,
                values,
                trailing_shorts,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_WORKSPACE_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPWorkspaceDomain")?;
            let (raw_count, values) = read_gp_domain_longs(reader, max_bytes, "GPWorkspaceDomain")?;
            Some(Box::new(GpTopoDomain10_1::Workspace {
                version,
                raw_count,
                values,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_RANGE_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPRangeDomain")?;
            let flags = reader.read_u32_le()?;
            let max_units = u32::try_from(max_bytes / 2).unwrap_or(u32::MAX);
            let mut read_bound = || {
                read_value_with_objects_10_1(
                    reader,
                    max_units,
                    VariantObjectContext10_1::ObjectStream(table),
                    |reader, header, context| {
                        let (Some(key), VariantObjectContext10_1::ObjectStream(table)) =
                            (header.key, context)
                        else {
                            return Err(io::Error::new(
                                ErrorKind::InvalidData,
                                "GPRangeDomain 缺少对象流上下文",
                            ));
                        };
                        read_variant_object_body_10_1(
                            reader,
                            max_bytes,
                            table,
                            name_depth,
                            ObjectEntry10_1::NewObject {
                                class_id: header.class_id,
                                key,
                            },
                        )
                        .map(Box::new)
                    },
                )
            };
            let bounds = [read_bound()?, read_bound()?];
            Some(Box::new(GpTopoDomain10_1::Range {
                version,
                flags,
                bounds,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_FILE_DOMAIN_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 1, "GPFileDomain")?;
            let (raw_count, strings) = read_gp_domain_strings(reader, max_bytes, "GPFileDomain")?;
            Some(Box::new(GpTopoDomain10_1::File {
                version,
                raw_count,
                strings,
            }))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPSATopoFeaturesTypeName 的 IGPDomain 类未支持",
            ));
        }
    };
    Ok(RuleSlot10_1 { entry, body })
}

fn is_gp_data_type_name(name: &BookmarkNameEntry10_1) -> bool {
    matches!(
        name,
        BookmarkNameEntry10_1::Null
            | BookmarkNameEntry10_1::GpDataTypeName(_)
            | BookmarkNameEntry10_1::GpMultiValueTypeName { .. }
            | BookmarkNameEntry10_1::GpValueTableTypeName { .. }
            | BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(_)
            | BookmarkNameEntry10_1::BackReference {
                class_id: GP_DATA_TYPE_NAME_CLASS_ID
                    | GP_MULTI_VALUE_TYPE_NAME_CLASS_ID
                    | GP_VALUE_TABLE_TYPE_NAME_CLASS_ID
                    | GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
                ..
            }
    )
}

fn read_gp_sa_topo_features_type_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    name_depth: usize,
) -> io::Result<GpSaTopoFeaturesTypeName10_1> {
    let version = read_gp_name_version(reader, 4, "GPSATopoFeaturesTypeName")?;
    let legacy_count = (version < 4)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    let count = legacy_count.unwrap_or(0).max(0) as usize;
    if count > 0 && name_depth == MAX_BOOKMARK_NAME_NESTING {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GPSATopoFeaturesTypeName 嵌套层数超限",
        ));
    }
    // 每条至少占一个 16 字节对象头、一个 6 字节空字符串及一个 LONG。
    let minimum_entry_bytes = if version == 2 { 26 } else { 42 };
    if count > reader.remaining().min(max_bytes) / minimum_entry_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GPSATopoFeaturesTypeName 条目数量超限",
        ));
    }
    let mut legacy_entries = Vec::new();
    for _ in 0..count {
        let data_type = read_bookmark_name_slot_10_1(reader, max_bytes, table, name_depth + 1)?;
        if !is_gp_data_type_name(&data_type) {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "GPSATopoFeaturesTypeName 的 IGPDataTypeName 类未支持",
            ));
        }
        let legacy_domain = (version < 2)
            .then(|| read_gp_topo_domain_slot_10_1(reader, max_bytes, table, name_depth + 1))
            .transpose()?;
        let string = read_string_10_1(reader, max_bytes)?;
        let value = reader.read_u32_le()? as i32;
        let uid = (version == 3)
            .then(|| read_gp_uid_slot(reader, table))
            .transpose()?;
        legacy_entries.push(GpSaTopoFeaturesTypeEntry10_1 {
            data_type,
            legacy_domain,
            string,
            value,
            uid,
        });
    }
    Ok(GpSaTopoFeaturesTypeName10_1 {
        version,
        legacy_count,
        legacy_entries,
    })
}

// 调用者负责将名称槽及其宿主的读取作为同一事务提交。
fn read_bookmark_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    name_depth: usize,
) -> io::Result<BookmarkNameEntry10_1> {
    let name_offset = reader.position();
    Ok(match read_object_entry_10_1(reader, table)? {
        ObjectEntry10_1::Null => BookmarkNameEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            // ObjectStream 在原实例上查询接口，回引头里的 CLSID 不参与查询。
            let class_id = table.registered_class(key).unwrap_or(class_id);
            if matches!(
                class_id,
                PROPERTY_SET_CLASS_ID | UID_CLASS_ID | GP_MULTI_VALUE_CLASS_ID
            ) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("IName 槽 offset 0x{name_offset:x}: 键 {key} 的对象不实现 IName"),
                ));
            }
            BookmarkNameEntry10_1::BackReference { class_id, key }
        }
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
            class_id: ROUTE_MEASURE_LOCATOR_NAME_CLASS_ID,
            ..
        } => {
            let mut stop = None;
            let body =
                read_route_measure_locator_name_body_10_1(reader, max_bytes, table, &mut stop)?;
            if let Some(stop) = stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: RouteMeasureLocatorName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::RouteMeasureLocatorName(Box::new(body))
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
            class_id: GP_VALUE_TABLE_TYPE_NAME_CLASS_ID,
            ..
        } => {
            let version = read_gp_name_version(reader, 3, "GPValueTableTypeName")?;
            let raw_column_count = reader.read_u32_le()? as i32;
            let count = raw_column_count.max(0) as usize;
            if count > reader.remaining().min(max_bytes) / 26
                || (count > 0 && name_depth >= MAX_BOOKMARK_NAME_NESTING)
            {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPValueTableTypeName 列数或嵌套层数超限",
                ));
            }
            let mut columns = Vec::new();
            for _ in 0..count {
                let data_type =
                    read_bookmark_name_slot_10_1(reader, max_bytes, table, name_depth + 1)?;
                if !is_gp_data_type_name(&data_type) {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "GPValueTableTypeName 的列类型名称类未支持",
                    ));
                }
                let legacy_domain = (version < 2)
                    .then(|| {
                        read_gp_topo_domain_slot_10_1(reader, max_bytes, table, name_depth + 1)
                    })
                    .transpose()?;
                let name = read_string_10_1(reader, max_bytes)?;
                let raw_long = reader.read_u32_le()? as i32;
                let uid = (version > 2)
                    .then(|| read_gp_uid_slot(reader, table))
                    .transpose()?;
                columns.push(GpValueTableColumnType10_1 {
                    data_type,
                    legacy_domain,
                    name,
                    raw_long,
                    uid,
                });
            }
            BookmarkNameEntry10_1::GpValueTableTypeName {
                version,
                raw_column_count,
                columns,
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_MULTI_VALUE_TYPE_NAME_CLASS_ID,
            ..
        } => {
            if name_depth == MAX_BOOKMARK_NAME_NESTING {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "GPMultiValueTypeName 嵌套层数超限",
                ));
            }
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("GPMultiValueTypeName 版本 {version} 超出 10.1 上限"),
                ));
            }
            let element_type =
                read_bookmark_name_slot_10_1(reader, max_bytes, table, name_depth + 1)?;
            if !matches!(
                element_type,
                BookmarkNameEntry10_1::Null
                    | BookmarkNameEntry10_1::GpDataTypeName(_)
                    | BookmarkNameEntry10_1::GpFunctionName(_)
                    | BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(_)
                    | BookmarkNameEntry10_1::GpMultiValueTypeName { .. }
                    | BookmarkNameEntry10_1::GpValueTableTypeName { .. }
                    | BookmarkNameEntry10_1::BackReference {
                        class_id: GP_DATA_TYPE_NAME_CLASS_ID
                            | GP_MULTI_VALUE_TYPE_NAME_CLASS_ID
                            | GP_VALUE_TABLE_TYPE_NAME_CLASS_ID
                            | GP_FUNCTION_NAME_CLASS_ID
                            | GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
                        ..
                    }
            ) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "GPMultiValueTypeName 的 IGPName 类未支持",
                ));
            }
            BookmarkNameEntry10_1::GpMultiValueTypeName {
                version,
                element_type: Box::new(element_type),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: GP_SA_TOPO_FEATURES_TYPE_NAME_CLASS_ID,
            ..
        } => BookmarkNameEntry10_1::GpSaTopoFeaturesTypeName(Box::new(
            read_gp_sa_topo_features_type_name_body_10_1(reader, max_bytes, table, name_depth)?,
        )),
        ObjectEntry10_1::NewObject {
            class_id: GP_FUNCTION_NAME_CLASS_ID,
            ..
        } => BookmarkNameEntry10_1::GpFunctionName(Box::new(read_gp_function_name_body_10_1(
            reader, max_bytes, table,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id: GP_DATA_TYPE_NAME_CLASS_ID,
            ..
        } => BookmarkNameEntry10_1::GpDataTypeName(Box::new(read_gp_data_type_name_body_10_1(
            reader, max_bytes, table,
        )?)),
        ObjectEntry10_1::NewObject {
            class_id: LOCATOR_NAME_CLASS_ID,
            ..
        } => {
            if name_depth == MAX_BOOKMARK_NAME_NESTING {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "LocatorName 嵌套层数超限",
                ));
            }
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("LocatorName 版本 {version} 超出 10.1 上限"),
                ));
            }
            let strings = [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ];
            let source = read_bookmark_name_slot_10_1(reader, max_bytes, table, name_depth + 1)?;
            BookmarkNameEntry10_1::LocatorName {
                version,
                strings,
                source: Box::new(source),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: LOCATOR_WORKSPACE_NAME_CLASS_ID,
            ..
        } => {
            if name_depth == MAX_BOOKMARK_NAME_NESTING {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "LocatorWorkspaceName 嵌套层数超限",
                ));
            }
            let version = reader.read_u16_le()? as i16;
            if version > 2 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("LocatorWorkspaceName 版本 {version} 超出 10.1 上限"),
                ));
            }
            let strings = [
                read_string_10_1(reader, max_bytes)?,
                read_string_10_1(reader, max_bytes)?,
            ];
            // v1 请求 IName，其余版本允许持久化 IUnknown。
            use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
            let source = match read_header_10_1(&mut reader.clone())? {
                ObjectHeader::Object { class_id, key } if version != 1 && table.contains(key) => {
                    read_object_entry_10_1(reader, table)?;
                    BookmarkNameEntry10_1::BackReference {
                        class_id: table.registered_class(key).unwrap_or(class_id),
                        key,
                    }
                }
                ObjectHeader::Object {
                    class_id: PROPERTY_SET_CLASS_ID,
                    ..
                } if version != 1 => {
                    read_object_entry_10_1(reader, table)?;
                    BookmarkNameEntry10_1::LocatorWorkspaceProperties(Box::new(
                        read_property_set_object_stream_10_1(reader, max_bytes, table)?,
                    ))
                }
                ObjectHeader::Object {
                    class_id: UID_CLASS_ID,
                    key,
                } if version != 1 => {
                    read_object_entry_10_1(reader, table)?;
                    BookmarkNameEntry10_1::LocatorWorkspaceUid(read_uid_body_10_1(reader, key)?)
                }
                _ => read_bookmark_name_slot_10_1(reader, max_bytes, table, name_depth + 1)?,
            };
            BookmarkNameEntry10_1::LocatorWorkspaceName {
                version,
                strings,
                source: Box::new(source),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: SHORTCUT_NAME_CLASS_ID,
            ..
        } => {
            if name_depth == MAX_BOOKMARK_NAME_NESTING {
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
            let target = read_bookmark_name_slot_10_1(reader, max_bytes, table, name_depth + 1)?;
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
        ObjectEntry10_1::NewObject {
            class_id: SCHEMATIC_DIAGRAM_NAME_CLASS_ID,
            ..
        } => {
            let mut stop = None;
            let body = read_schematic_diagram_name_body_10_1(reader, max_bytes, table, &mut stop)?;
            if let Some(stop) = stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: SchematicDiagramName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::SchematicDiagramName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id: SCHEMATIC_FOLDER_NAME_CLASS_ID,
            ..
        } => {
            let mut stop = None;
            let body = read_schematic_folder_name_body_10_1(reader, max_bytes, table, &mut stop)?;
            if let Some(stop) = stop {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: SchematicFolderName 未读完（{stop:?}）"
                    ),
                ));
            }
            BookmarkNameEntry10_1::SchematicFolderName(Box::new(body))
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (SCHEMATIC_DATASET_NAME_CLASS_ID | SCHEMATIC_WORKSPACE_NAME_CLASS_ID),
            key,
        } => {
            let (body, stop) =
                read_schematic_dataset_name_body_10_1(reader, max_bytes, table, key)?;
            let is_workspace = class_id == SCHEMATIC_WORKSPACE_NAME_CLASS_ID;
            if let Some(stop) = stop {
                let class_name = if is_workspace {
                    "SchematicWorkspaceName"
                } else {
                    "SchematicDatasetName"
                };
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: {class_name} 未读完（{stop:?}）"
                    ),
                ));
            }
            if is_workspace {
                BookmarkNameEntry10_1::SchematicWorkspaceName(Box::new(body))
            } else {
                BookmarkNameEntry10_1::SchematicDatasetName(Box::new(body))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gp_raster_data_layer_reads_feature_drawing_with_label_classes() {
        for (version, placement_class) in [
            (0_i16, POINT_PLACEMENT_CLASS_ID),
            (1, POINT_PLACEMENT_CLASS_ID),
            (2, POINT_PLACEMENT_CLASS_ID),
            (3, POINT_PLACEMENT_CLASS_ID),
            (4, POINT_PLACEMENT_CLASS_ID),
            (4, LINE_PLACEMENT_CLASS_ID),
            (5, POLYGON_PLACEMENT_CLASS_ID),
        ] {
            let mut bytes = GP_RASTER_DATA_LAYER_CLASS_ID.to_vec();
            bytes.extend(10_u32.to_le_bytes());
            bytes.extend([1, 0, 1, 0, 7, 0]);
            bytes.extend([0; 16]);
            let empty_string = [2, 0, 0, 0, 0, 0];
            bytes.extend(empty_string);
            bytes.extend(FEATURE_LAYER_DRAWING_CLASS_ID);
            bytes.extend(11_u32.to_le_bytes());
            let version_offset = bytes.len();
            bytes.extend(version.to_le_bytes());
            bytes.extend(crate::layers::feature_layer_tail::SIMPLE_RENDERER_CLASS_ID);
            bytes.extend(16_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend([0; 48]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(empty_string);
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(empty_string);
            if version >= 2 {
                bytes.extend(7_u16.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend([1, 0, 2, 0, 3, 0]);
            }
            if version >= 4 {
                for (class, key) in [(LABELING_CLASS_ID, 12_u32), (LABEL_CLASSES_CLASS_ID, 13)] {
                    bytes.extend(class);
                    bytes.extend(key.to_le_bytes());
                    bytes.extend(1_i16.to_le_bytes());
                }
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend(LABEL_CLASS_CLASS_ID);
                bytes.extend(14_u32.to_le_bytes());
                bytes.extend(3_i16.to_le_bytes());
                bytes.extend(placement_class);
                bytes.extend(15_u32.to_le_bytes());
                bytes.extend(1_i16.to_le_bytes());
                bytes.extend((-7_i32).to_le_bytes());
                bytes.extend(empty_string);
                bytes.extend([0; 16]);
                bytes.extend(2.5_f64.to_le_bytes());
                bytes.extend(5.0_f64.to_le_bytes());
                bytes.extend(9_u16.to_le_bytes());
                bytes.extend(30_i32.to_le_bytes());
                bytes.extend(empty_string);
                bytes.extend(LABEL_CLASS_CLASS_ID);
                bytes.extend(14_u32.to_le_bytes());
            }
            if version >= 5 {
                bytes.extend(123_i32.to_le_bytes());
            }
            let parse = |reader: &mut Reader<'_>| {
                read_gp_coded_value_slot(reader, 1024, &mut RefTable10_1::new(), 0)
            };
            for cut in 0..bytes.len() {
                assert!(
                    parse(&mut Reader::new(&bytes[..cut])).is_err(),
                    "截断于 {cut}"
                );
            }
            let mut invalid = bytes.clone();
            invalid[version_offset..version_offset + 2].copy_from_slice(&6_i16.to_le_bytes());
            assert_eq!(
                parse(&mut Reader::new(&invalid)).unwrap_err().kind(),
                ErrorKind::InvalidData
            );
            let end = bytes.len();
            bytes.push(0x5a);
            let mut reader = Reader::new(&bytes);
            let GpCodedValue10_1::RasterDataLayer { drawing, .. } =
                parse(&mut reader).unwrap().body.unwrap().value
            else {
                panic!("应读取 GPRasterDataLayer");
            };
            let LayerDrawingDescription10_1::Feature(drawing) = *drawing.body.unwrap() else {
                panic!("应读取矢量绘制描述");
            };
            assert_eq!(reader.position(), end);
            assert!(matches!(
                drawing.renderer,
                crate::layers::feature_layer::RendererSlot10_1::SimpleRenderer(_)
            ));
            assert_eq!(drawing.version, version);
            assert_eq!(drawing.raw_short, (version >= 2).then_some(7));
            assert_eq!(drawing.raw_shorts, (version >= 3).then_some([1, 2, 3]));
            assert_eq!(drawing.raw_long, (version >= 5).then_some(123));
            if let Some(labeling) = drawing.labeling {
                let classes = labeling.body.unwrap().classes.body.unwrap().classes;
                let class = classes[0].body.as_ref().unwrap();
                assert_eq!(class.placement.body.as_ref().unwrap().raw_long, -7);
                assert_eq!(class.raw_doubles, [2.5, 5.0]);
                assert_eq!(class.raw_short, Some(9));
                assert_eq!(class.tail, Some((30, String::new())));
                assert!(matches!(
                    classes[1].entry,
                    ObjectEntry10_1::BackReference { key: 14, .. }
                ));
            } else {
                assert!(version < 4);
            }
        }
    }

    #[test]
    fn gp_raster_data_layer_reads_versioned_drawing_tail() {
        for version in [-1_i16, 0, 1, 2] {
            let mut bytes = GP_RASTER_DATA_LAYER_CLASS_ID.to_vec();
            bytes.extend(10_u32.to_le_bytes());
            bytes.extend([1, 0, 1, 0, 7, 0]);
            bytes.extend([0; 16]);
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend(RASTER_LAYER_DRAWING_DESCRIPTION_CLASS_ID);
            bytes.extend(11_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 16]);
            if version == 2 {
                bytes.extend([1, 0, 2, 0, 3, 0]);
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0x5a);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let GpCodedValue10_1::RasterDataLayer { drawing, .. } = slot.body.unwrap().value else {
                panic!("应读取 GPRasterDataLayer");
            };
            let LayerDrawingDescription10_1::Raster(drawing) = *drawing.body.unwrap() else {
                panic!("应读取栅格绘制描述");
            };
            assert_eq!(drawing.version, version);
            assert_eq!(drawing.renderer.entry, ObjectEntry10_1::Null);
            assert_eq!(drawing.raw_shorts, (version == 2).then_some([1, 2, 3]));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn raster_builder_reuses_mosaic_raster_type_body() {
        let mut bytes = GP_RASTER_BUILDER_CLASS_ID.to_vec();
        bytes.extend(10_u32.to_le_bytes());
        bytes.extend([1, 0, 1, 0, 7, 0]);
        bytes.extend([
            0x57, 0x29, 0xee, 0xc7, 0x30, 0xe2, 0xe0, 0x4b, 0xae, 0x55, 0xa4, 0x59, 0xb9, 0x64,
            0x7e, 0x21,
        ]);
        bytes.extend(11_u32.to_le_bytes());
        bytes.extend([0; 18 + 16]);
        bytes.extend(2_u16.to_le_bytes());
        for cut in 0..bytes.len() {
            assert!(
                read_gp_coded_value_slot(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0x5a);
        let mut reader = Reader::new(&bytes);
        let slot =
            read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
        let GpCodedValue10_1::RasterBuilder(raster_type) = slot.body.unwrap().value else {
            panic!("应读取 GPRasterBuilder");
        };
        assert_eq!(raster_type.body.unwrap().mosaic_version, Some(2));
        assert_eq!(reader.position(), end);
        bytes[20] = 2;
        assert!(
            read_gp_coded_value_slot(&mut Reader::new(&bytes), 1024, &mut RefTable10_1::new(), 0,)
                .is_err()
        );
    }

    #[test]
    fn raster_formulated_reads_legacy_and_compact_variant_tails() {
        for version in 0_u16..=2 {
            let mut bytes = GP_RASTER_FORMULATED_CLASS_ID.to_vec();
            bytes.extend(10_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 1, 0, 7, 0]);
            bytes.extend(8_u32.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 32]);
            bytes.extend(9_u32.to_le_bytes());
            if version < 2 {
                bytes.extend([0xff; 24]);
            } else {
                bytes.extend([0; 2]);
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0x5a);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let GpCodedValue10_1::RasterFormulated(raster) = slot.body.unwrap().value else {
                panic!("应读取 GPRasterFormulated");
            };
            assert_eq!(
                (raster.raster_version, raster.raster_long, raster.raw_long),
                (1, 8, 9)
            );
            assert_eq!(raster.envelope, BookmarkEnvelopeEntry10_1::Null);
            assert_eq!(raster.spatial_reference, SpatialReferenceEntry10_1::Null);
            match raster.value {
                GpRasterFormulatedValue10_1::Legacy(value) => {
                    assert!(version < 2);
                    assert_eq!(value, [0xff; 24]);
                }
                GpRasterFormulatedValue10_1::Variant(_) => assert_eq!(version, 2),
            }
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_raster_data_reads_name_reference_and_trailing_text() {
        let mut name_header = RASTER_DATASET_NAME_CLASS_ID.to_vec();
        name_header.extend(11_u32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&name_header), &mut refs).unwrap();
        for version in [0, 1] {
            let mut bytes = GP_RASTER_DATA_CLASS_ID.to_vec();
            bytes.extend(10_u32.to_le_bytes());
            bytes.extend([version, 0, 1, 0, 7, 0]);
            bytes.extend(&name_header);
            bytes.extend([4, 0, 0, 0, b'x', 0, 0, 0]);
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut refs.clone(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0x5a);
            let mut reader = Reader::new(&bytes);
            let slot = read_gp_coded_value_slot(&mut reader, 1024, &mut refs.clone(), 0).unwrap();
            assert_eq!(
                slot.body.unwrap().value,
                GpCodedValue10_1::RasterData {
                    name: Box::new(BookmarkNameEntry10_1::BackReference {
                        class_id: RASTER_DATASET_NAME_CLASS_ID,
                        key: 11,
                    }),
                    text: "x".into(),
                },
            );
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_field_mapping_reads_rows_and_rejects_wrong_registered_class() {
        let mut bytes = GP_FIELD_MAPPING_CLASS_ID.to_vec();
        bytes.extend(10_u32.to_le_bytes());
        bytes.extend([2, 0, 1, 0, 0, 0]);
        bytes.extend(2_i32.to_le_bytes());
        let mut map_header = GP_FIELD_MAP_CLASS_ID.to_vec();
        map_header.extend(11_u32.to_le_bytes());
        bytes.extend(&map_header);
        bytes.extend([2, 0, 7, 0, 0, 0]);
        bytes.extend([2, 0, 0, 0, 0, 0]);
        bytes.extend([0; 16]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 32]);
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(5_u32.to_le_bytes());
        bytes.extend(&map_header);
        bytes.extend(0_i32.to_le_bytes());
        let tail_start = bytes.len();
        bytes.extend([0; 16]);

        for cut in 0..bytes.len() {
            assert!(
                read_gp_coded_value_slot(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0x5a);
        let mut reader = Reader::new(&bytes);
        let slot =
            read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
        let GpCodedValue10_1::FieldMapping { maps, tail, .. } = slot.body.unwrap().value else {
            panic!("应读取 GPFieldMapping");
        };
        let map = maps[0].body.as_deref().unwrap();
        assert_eq!((map.version, map.raw_long, map.raw_count), (2, 7, 1));
        assert_eq!(map.rows[0].raw_longs, [3, 5]);
        assert!(matches!(
            maps[1].entry,
            ObjectEntry10_1::BackReference { key: 11, .. }
        ));
        assert!(maps[1].body.is_none());
        assert_eq!(tail.unwrap().entry, ObjectEntry10_1::Null);
        assert_eq!(reader.position(), end);

        bytes.splice(tail_start..end, map_header);
        let error =
            read_gp_coded_value_slot(&mut Reader::new(&bytes), 1024, &mut RefTable10_1::new(), 0)
                .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
    }

    #[test]
    fn replica_value_reads_description_and_version_two_tail() {
        for version in 0_u16..=2 {
            let mut header = GP_REPLICA_CLASS_ID.to_vec();
            header.extend(10_u32.to_le_bytes());
            let mut bytes = header.clone();
            bytes.extend(version.to_le_bytes());
            bytes.extend(9_u32.to_le_bytes());
            bytes.extend([0; 16 + 2 + 4]);
            bytes.extend(GP_REPLICA_DESCRIPTION_CLASS_ID);
            bytes.extend(11_u32.to_le_bytes());
            bytes.extend([0; 2 + 10 + 16 + 16 + 2]);
            for value in 1_u32..=6 {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(42_u16.to_le_bytes());
            if version == 2 {
                bytes.extend([0; 6]);
                bytes.extend([1, 0, 2, 0, 3, 0]);
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.extend(header);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let slot = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            let GpCodedValue10_1::Replica(replica) = slot.body.unwrap().value else {
                panic!("应读取 GPReplica");
            };
            assert_eq!(replica.initial_long, 9);
            assert!(matches!(
                replica.description.body.as_deref().map(|body| &body.value),
                Some(GpCodedValue10_1::ReplicaDescription { .. })
            ));
            assert_eq!(replica.trailing_longs, [1, 2, 3, 4, 5, 6]);
            assert_eq!(replica.trailing_short, 42);
            assert_eq!(
                replica.v2.map(|tail| tail.raw_shorts),
                (version == 2).then_some([1, 2, 3])
            );
            assert_eq!(reader.position(), end);
            let reference = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                reference.entry,
                ObjectEntry10_1::BackReference { key: 10, .. }
            ));
            assert!(reference.body.is_none());
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn replica_description_reads_dataset_collection_and_child_reference() {
        let mut bytes = GP_REPLICA_DESCRIPTION_CLASS_ID.to_vec();
        bytes.extend(10_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 10 + 16]);
        bytes.extend(GP_REPLICA_DATASETS_CLASS_ID);
        bytes.extend(11_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(GP_REPLICA_DATASET_CLASS_ID);
        bytes.extend(12_u32.to_le_bytes());
        bytes.extend([0; 2 + 4 + 8 + 4 + 4 + 16]);
        bytes.extend(GP_REPLICA_DATASET_CLASS_ID);
        bytes.extend(12_u32.to_le_bytes());
        bytes.extend(42_u16.to_le_bytes());

        for cut in 0..bytes.len() {
            assert!(
                read_gp_coded_value_slot(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0x5a);
        let mut reader = Reader::new(&bytes);
        let result =
            read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
        let GpCodedValue10_1::ReplicaDescription {
            geometry,
            datasets,
            raw_shorts,
            ..
        } = result.body.unwrap().value
        else {
            panic!("应读取复制描述");
        };
        assert_eq!(raw_shorts, [0, 42]);
        assert_eq!(geometry.entry, ObjectEntry10_1::Null);
        let datasets = datasets.body.unwrap();
        assert_eq!((datasets.version, datasets.raw_count), (1, 2));
        assert!(datasets.datasets[0].body.is_some());
        assert_eq!(
            datasets.datasets[1].entry,
            ObjectEntry10_1::BackReference {
                class_id: GP_REPLICA_DATASET_CLASS_ID,
                key: 12
            }
        );
        assert!(datasets.datasets[1].body.is_none());
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn replica_dataset_value_reads_versioned_suffixes() {
        for version in 0_u16..=3 {
            for kind in [0_u32, 11] {
                let mut bytes = GP_REPLICA_DATASET_CLASS_ID.to_vec();
                bytes.extend(18_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
                bytes.extend(kind.to_le_bytes());
                bytes.extend([0; 12]);
                if kind == 11 {
                    bytes.extend(42_u32.to_le_bytes());
                    if version > 1 {
                        bytes.extend([0; 8]);
                    }
                }
                bytes.extend([0; 16]);
                if version == 3 {
                    bytes.extend([0; 20]);
                }
                for cut in 0..bytes.len() {
                    assert!(
                        read_gp_coded_value_slot(
                            &mut Reader::new(&bytes[..cut]),
                            1024,
                            &mut RefTable10_1::new(),
                            0,
                        )
                        .is_err()
                    );
                }
                let end = bytes.len();
                bytes.push(0x5a);
                let mut reader = Reader::new(&bytes);
                let result =
                    read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0)
                        .unwrap();
                let GpCodedValue10_1::ReplicaDataset {
                    raw_longs,
                    type_11,
                    v3,
                    ..
                } = result.body.unwrap().value
                else {
                    panic!("应读取 GPReplicaDataset");
                };
                assert_eq!(raw_longs, [kind, 0]);
                assert_eq!(type_11.is_some(), kind == 11);
                if let Some((raw, strings)) = type_11 {
                    assert_eq!(raw, 42);
                    assert_eq!(strings.is_some(), version > 1);
                }
                assert_eq!(v3.is_some(), version == 3);
                assert_eq!(reader.position(), end);
            }
        }
    }

    #[test]
    fn gp_line_and_polygon_follow_native_keyed_save_order() {
        for class_id in [GP_LINE_CLASS_ID, GP_POLYGON_CLASS_ID] {
            let mut header = class_id.to_vec();
            header.extend(0_u32.to_le_bytes());
            let mut bytes = header.clone();
            bytes.extend([1, 0]);
            bytes.extend(2_i32.to_le_bytes());
            let mut point = GP_POINT_CLASS_ID.to_vec();
            point.extend(1_u32.to_le_bytes());
            bytes.extend(&point);
            bytes.extend([1, 0, 1, 0, 0xff, 0xff]);
            bytes.extend([0; 16]);
            bytes.extend(&point);
            bytes.extend([1, 0, 0xff, 0xff]);
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.extend(header);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let body = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0)
                .unwrap()
                .body
                .unwrap();
            assert_eq!(
                (body.version, body.base_version, body.base_short),
                (1, 1, 0xffff)
            );
            let GpCodedValue10_1::LineOrPolygon { raw_count, points } = body.value else {
                panic!("应读取 GP 线面点集合");
            };
            assert_eq!(raw_count, 2);
            assert!(matches!(
                points[0].body.as_deref().map(|body| &body.value),
                Some(GpCodedValue10_1::Point {
                    x_bits: 0,
                    y_bits: 0
                })
            ));
            assert!(matches!(
                points[1].entry,
                ObjectEntry10_1::BackReference { key: 1, .. }
            ));
            assert_eq!(reader.position(), end);
            let reference = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                reference.entry,
                ObjectEntry10_1::BackReference { key: 0, .. }
            ));
            assert!(reference.body.is_none());
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn field_coded_value_reuses_field_body_and_reference() {
        for version in [0_u16, 1, 2] {
            let mut header = FIELD_CLASS_ID.to_vec();
            header.extend(55_u32.to_le_bytes());
            let mut bytes = header.clone();
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 7, 0]);
            for has_flag in [true, true, false, true] {
                bytes.extend([2, 0, 0, 0, 0, 0]);
                if has_flag {
                    bytes.push(1);
                }
            }
            bytes.extend([0; 2 + 16 + 5 + 16 + 16]);
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.extend(header);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let slot = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            let body = slot.body.unwrap();
            assert_eq!(
                (body.version, body.base_version, body.base_short),
                (version, 1, 7)
            );
            let GpCodedValue10_1::Field(field) = body.value else {
                panic!("应读取 Field 值对象");
            };
            assert_eq!(field.string_flags, [1; 3]);
            assert_eq!(reader.position(), end);
            let reference = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                reference.entry,
                ObjectEntry10_1::BackReference { key: 55, .. }
            ));
            assert!(reference.body.is_none());
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn fme_dataset_reads_unversioned_bstr_and_reference() {
        for (payload, expected) in [
            (vec![], None),
            (vec![0, 0], Some(vec![])),
            (vec![b'A', 0, 0, 0], Some(vec![b'A', 0])),
        ] {
            let mut header = FME_DATASET_CLASS_ID.to_vec();
            header.extend(55_u32.to_le_bytes());
            let mut bytes = header.clone();
            bytes.extend((payload.len() as u32).to_le_bytes());
            bytes.extend(payload);
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.extend(header);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let first = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            assert_eq!(
                first.body.unwrap().value,
                GpCodedValue10_1::FmeDataset(expected)
            );
            assert_eq!(reader.position(), end);
            let second = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                second.entry,
                ObjectEntry10_1::BackReference { key: 55, .. }
            ));
            assert!(second.body.is_none());
            assert_eq!(reader.remaining(), 0);
        }
        for length in [1_u32, 0x100001] {
            let mut bytes = FME_DATASET_CLASS_ID.to_vec();
            bytes.extend(55_u32.to_le_bytes());
            bytes.extend(length.to_le_bytes());
            let err = read_gp_coded_value_slot(
                &mut Reader::new(&bytes),
                usize::MAX,
                &mut RefTable10_1::new(),
                0,
            )
            .unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidData);
        }
    }

    #[test]
    fn de_representation_class_reads_rule_and_attribute_slots() {
        let empty = [2, 0, 0, 0, 0, 0];
        for version in 0_u16..=5 {
            let mut bytes = DE_REPRESENTATION_CLASS_ID.to_vec();
            bytes.extend(64_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 8]);
            bytes.extend(empty.repeat(3));
            bytes.extend([0; 22]);
            if version <= 2 {
                bytes.extend(7_u32.to_le_bytes());
            }
            bytes.extend(empty.repeat(if version == 4 { 2 } else { 3 }));
            if version <= 2 {
                bytes.extend(8_u32.to_le_bytes());
            }
            bytes.extend(REPRESENTATION_RULES_CLASS_ID);
            bytes.extend(65_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(9_u32.to_le_bytes());
            bytes.extend(empty);
            bytes.extend(10_u32.to_le_bytes());
            bytes.extend(crate::objects::display_symbol::REPRESENTATION_RULE_CLASS_ID);
            bytes.extend(66_u32.to_le_bytes());
            bytes.extend([0; 28]);
            bytes.extend(CLASS_ATTRIBUTES_CLASS_ID);
            bytes.extend(67_u32.to_le_bytes());
            bytes.extend([0; 8]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(11_u32.to_le_bytes());
            bytes.extend(empty);
            if version > 1 {
                bytes.extend(12_u16.to_le_bytes());
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let Some(DeChild10_1::RepresentationClass(body)) = slot.body.as_deref() else {
                panic!("应读取 DERepresentationClass");
            };
            assert_eq!(body.first_name.is_some(), version != 4);
            assert_eq!(body.legacy_longs, (version <= 2).then_some([7, 8]));
            let item = &body.rules.body.as_ref().unwrap().items[0];
            assert_eq!((item.id, item.v1), (9, Some(10)));
            assert!(item.rule.body.is_some());
            assert_eq!(
                body.attributes.body.as_ref().unwrap().names,
                [(11, String::new())]
            );
            assert_eq!(body.trailing_short, (version > 1).then_some(12));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn de_toolbox_reads_bstr_tail_and_retries_legacy_dataset() {
        for version in [None, Some(0_u16), Some(1), Some(2)] {
            let mut bytes = DE_TOOLBOX_CLASS_ID.to_vec();
            bytes.extend(63_u32.to_le_bytes());
            if let Some(version) = version {
                bytes.extend(version.to_le_bytes());
            }
            bytes.extend([0; 8]);
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
            bytes.extend([0; 22]);
            if version.is_some() {
                bytes.extend([1, 0, 0, 0, b'T', 0, 0, 0]);
            }
            if version == Some(2) {
                bytes.extend([0; 8]);
                bytes.extend(9_u32.to_le_bytes());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let Some(DeChild10_1::Toolbox { extension, .. }) = slot.body.as_deref() else {
                panic!("应读取 DEToolbox");
            };
            assert_eq!(extension.as_ref().map(|tail| tail.version), version);
            if let Some(tail) = extension {
                assert_eq!(tail.name, "T");
                assert_eq!(
                    tail.v2.as_ref().map(|(_, flag)| *flag),
                    (version == Some(2)).then_some(9)
                );
            }
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn de_tool_ends_after_dataset() {
        let mut bytes = DE_TOOL_CLASS_ID.to_vec();
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
        bytes.extend([0; 22]);
        for cut in 0..bytes.len() {
            assert!(
                read_gp_data_element_child(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let slot =
            read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
        assert!(matches!(
            slot.body.as_deref(),
            Some(DeChild10_1::Tool { version: 1, .. })
        ));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn de_mosaic_dataset_reads_versioned_slots() {
        for version in [-1_i16, 0, 1, 2, 3, 4] {
            let mut bytes = DE_MOSAIC_DATASET_CLASS_ID.to_vec();
            bytes.extend(62_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 8]);
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
            bytes.extend([0; 22]);
            bytes.extend([0; 32]);
            bytes.extend([0; 16]);
            bytes.extend(7_u16.to_le_bytes());
            if version >= 3 {
                bytes.extend((-9_i32).to_le_bytes());
            }
            if version >= 2 {
                bytes.extend([0; 16]);
            }
            bytes.extend([0; 16]);
            if version >= 4 {
                bytes.extend([0; 16]);
            }
            let read = |reader: &mut Reader<'_>| {
                read_gp_data_element_child(reader, 1024, &mut RefTable10_1::new(), 0)
            };
            for cut in 0..bytes.len() {
                assert!(read(&mut Reader::new(&bytes[..cut])).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let parsed = read(&mut reader).unwrap().body.unwrap();
            let DeChild10_1::MosaicDataset(body) = *parsed else {
                panic!("应读取 DEMosaicDataset");
            };
            assert_eq!(body.version, version);
            assert_eq!(body.raw_short, 7);
            assert_eq!(body.raw_long, (version >= 3).then_some(-9));
            assert_eq!(body.catalog.is_some(), version >= 2);
            assert_eq!(body.table.is_some(), version >= 4);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn de_schematic_diagram_reads_text_after_geo_dataset() {
        let mut bytes = DE_SCHEMATIC_DIAGRAM_CLASS_ID.to_vec();
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
        bytes.extend([0; 22]);
        bytes.extend([0; 32]);
        bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
        let read = |reader: &mut Reader<'_>| {
            read_gp_data_element_child(reader, 1024, &mut RefTable10_1::new(), 0)
        };
        for cut in 0..bytes.len() {
            assert!(read(&mut Reader::new(&bytes[..cut])).is_err());
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let slot = read(&mut reader).unwrap();
        assert!(
            matches!(slot.body.as_deref(), Some(DeChild10_1::SchematicDiagram {
            geo_dataset_version: 1, text, ..
        }) if text == "X")
        );
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn de_schematic_dataset_and_folder_end_after_dataset() {
        for class_id in [DE_SCHEMATIC_DATASET_CLASS_ID, DE_SCHEMATIC_FOLDER_CLASS_ID] {
            let mut bytes = class_id.to_vec();
            bytes.extend(62_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 8]);
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
            bytes.extend([0; 22]);
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            assert!(matches!(
                slot.body.as_deref(),
                Some(DeChild10_1::SchematicDataset { .. })
            ));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn cadastral_fabric_reads_field_edits_and_versioned_tail() {
        let empty = [2, 0, 0, 0, 0, 0];
        for tail_version in 0_u16..=3 {
            let mut bytes = DE_CADASTRAL_FABRIC_CLASS_ID.to_vec();
            bytes.extend(60_u32.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend([0; 10]);
            bytes.extend(empty.repeat(3));
            bytes.extend([0; 54]);
            bytes.extend(11_u32.to_le_bytes());
            bytes.extend(empty);
            bytes.extend(12_u32.to_le_bytes());
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(CADASTRAL_FIELD_EDITS_CLASS_ID);
            bytes.extend(61_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend(empty);
            bytes.extend(CADASTRAL_FIELD_EDITS_CLASS_ID);
            bytes.extend(61_u32.to_le_bytes());
            bytes.extend(tail_version.to_le_bytes());
            if tail_version >= 2 {
                bytes.extend([0; 20]);
                bytes.extend(empty);
                bytes.extend([0; 22]);
            }
            if tail_version == 3 {
                bytes.extend([0; 80]);
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let Some(DeChild10_1::CadastralFabric(body)) = slot.body.as_deref() else {
                panic!("应读取 DECadastralFabric");
            };
            assert_eq!(body.longs, [11, 12]);
            assert_eq!(body.edits[0].body.as_ref().unwrap().version, 1);
            assert!(matches!(
                body.edits[1].entry,
                ObjectEntry10_1::BackReference { key: 61, .. }
            ));
            assert_eq!(body.tail.is_some(), tail_version >= 2);
            assert_eq!(
                body.tail
                    .as_ref()
                    .and_then(|tail| tail.properties.as_ref())
                    .is_some(),
                tail_version == 3
            );
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn network_dataset_dispatches_versions_and_nonempty_slots() {
        use crate::objects::network_source_directions::{
            NETWORK_ATTRIBUTE_CLASS_ID, NETWORK_DIRECTIONS_CLASS_ID, TURN_FEATURE_SOURCE_CLASS_ID,
        };
        let empty = [2, 0, 0, 0, 0, 0];
        for version in 1_u16..=4 {
            let mut bytes = DE_NETWORK_DATASET_CLASS_ID.to_vec();
            bytes.extend(43_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 10]);
            bytes.extend(empty.repeat(3));
            bytes.extend([0; 54]);
            bytes.extend(empty);
            bytes.extend([0; 20]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(NETWORK_DIRECTIONS_CLASS_ID);
            bytes.extend(44_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(empty.repeat(3));
            bytes.extend([0; 36]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(TURN_FEATURE_SOURCE_CLASS_ID);
            bytes.extend(45_u32.to_le_bytes());
            bytes.extend([0; 10]);
            bytes.extend(empty);
            bytes.extend([0; 20]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(NETWORK_ATTRIBUTE_CLASS_ID);
            bytes.extend(46_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 4]);
            bytes.extend(empty);
            bytes.extend([0; 28]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend([0; 48]);
            bytes.extend(empty);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(NETWORK_ATTRIBUTE_CLASS_ID);
            bytes.extend(46_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(77_u32.to_le_bytes());
            if version >= 2 {
                bytes.extend(78_u32.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend([0; 12]);
                bytes.extend(empty);
            }
            if version >= 4 {
                bytes.extend(empty);
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let DeChild10_1::NetworkDataset(body) = *slot.body.unwrap() else {
                panic!("应读取 DENetworkDataset");
            };
            assert_eq!(reader.position(), end);
            assert_eq!(body.sources.1.len(), 1);
            assert_eq!(body.attributes.1.len(), 1);
            assert_eq!(body.evaluator_rows.1.len(), 1);
            assert_eq!(body.values.1, [77]);
            assert_eq!(body.final_name.is_some(), version == 4);
            assert!(matches!(
                body.extra_attribute.unwrap().entry,
                ObjectEntry10_1::BackReference { key: 46, .. }
            ));
            for cut in 0..end {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err(),
                    "截断位置 {cut}"
                );
            }
        }
    }

    #[test]
    fn terrain_v5_to_13_reads_versioned_fields_and_lists() {
        let empty = [2, 0, 0, 0, 0, 0];
        for version in 5_u32..=13 {
            let mut bytes = DE_TERRAIN_CLASS_ID.to_vec();
            bytes.extend(43_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 10]);
            bytes.extend(empty.repeat(3));
            bytes.extend([0; 54]);
            bytes.extend(1_u32.to_le_bytes());
            if version >= 6 {
                bytes.extend(6_u32.to_le_bytes());
            }
            if version >= 7 {
                bytes.extend(7_f64.to_le_bytes());
            }
            if version >= 8 {
                bytes.extend(8_u32.to_le_bytes());
            }
            bytes.extend([0; 16]);
            bytes.extend(empty);
            if version >= 9 {
                if version >= 11 {
                    bytes.extend(11_u16.to_le_bytes());
                }
                if version == 13 {
                    bytes.extend(13_u32.to_le_bytes());
                }
                bytes.extend(21_u32.to_le_bytes());
                bytes.extend(22_u32.to_le_bytes());
                for value in 1..=12 {
                    bytes.extend(f64::from(value).to_le_bytes());
                }
            }
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend([0; 46]);
            bytes.extend(empty.repeat(3));
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend([0; 20]);
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let Some(DeChild10_1::TerrainGeoDataset(body)) = slot.body.as_deref() else {
                panic!("应读取 DETerrain v5–13");
            };
            assert_eq!(body.v6, (version >= 6).then_some(6));
            assert_eq!(body.v7_bits, (version >= 7).then_some(7_f64.to_bits()));
            assert_eq!(body.v8, (version >= 8).then_some(8));
            assert_eq!(body.v9_tail.is_some(), version >= 9);
            if let Some(tail) = &body.v9_tail {
                assert_eq!(tail.v11, (version >= 11).then_some(11));
                assert_eq!(tail.v13, (version == 13).then_some(13));
                assert_eq!(tail.longs, [21, 22]);
                assert_eq!(tail.envelopes[0][0], 1_f64.to_bits());
                assert_eq!(tail.envelopes[1][5], 12_f64.to_bits());
            }
            assert_eq!((body.sources.len(), body.levels.len()), (1, 1));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn legacy_terrain_reads_sources_levels_and_version_tail() {
        let empty = [2, 0, 0, 0, 0, 0];
        for version in 1_u32..=4 {
            let mut bytes = DE_TERRAIN_CLASS_ID.to_vec();
            bytes.extend(42_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 20]);
            bytes.extend(2_u32.to_le_bytes());
            for flag in [0_u32, 1] {
                bytes.extend(2_u32.to_le_bytes());
                bytes.extend([0; 12]);
                bytes.extend(flag.to_le_bytes());
                bytes.extend([0; 26]);
                bytes.extend(empty.repeat(3));
                bytes.extend(17_u32.to_le_bytes());
                bytes.extend(1_u32.to_le_bytes());
                if flag == 0 {
                    bytes.extend(empty);
                }
            }
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend([0; 20]);
            if version >= 2 {
                bytes.extend(empty);
                bytes.extend([0; 14]);
            }
            if version >= 3 {
                bytes.extend(empty);
            }
            if version == 4 {
                bytes.extend([0; 32]);
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let Some(DeChild10_1::TerrainLegacy(body)) = slot.body.as_deref() else {
                panic!("应读取旧版 DETerrain");
            };
            assert_eq!(body.version, version);
            assert_eq!(body.sources[0].names, [""]);
            assert!(body.sources[1].names.is_empty());
            assert_eq!(body.sources[1].name_count, 1);
            assert_eq!(body.levels.len(), 1);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn relationship_class_v5_v6_read_table_base_and_version_tail() {
        for class_id in [
            DE_RELATIONSHIP_CLASS_CLASS_ID,
            DE_RELATIONSHIP_CLASS_INFO_CLASS_ID,
        ] {
            for version in [5_u16, 6] {
                let mut bytes = class_id.to_vec();
                bytes.extend(41_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                // DETableBase v0、DEDataset v0、DataElement v0、GPValue v0。
                bytes.extend([0; 10]);
                let empty_string = [2, 0, 0, 0, 0, 0];
                for _ in 0..3 {
                    bytes.extend(empty_string);
                }
                bytes.extend([0; 22]); // DataElement 和 DEDataset 的定宽字段。
                bytes.extend(17_u16.to_le_bytes()); // DETableBase 前导 short。
                bytes.extend(empty_string);
                if class_id == DE_RELATIONSHIP_CLASS_CLASS_ID {
                    for _ in 0..5 {
                        bytes.extend(empty_string);
                    }
                    bytes.extend([0; 44]);
                }
                bytes.extend([0; 16]); // 定宽字段、空对象槽和列表计数，直到 IIndexes。
                bytes.extend(INDEXES_CLASS_ID);
                bytes.extend(42_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());

                bytes.extend([0; 40]); // 两个 IStringArray 和两个 long。
                bytes.extend(empty_string);
                bytes.extend(empty_string);
                bytes.extend([0; 8]); // 两个 long。
                bytes.extend([0, 0, 1, 0, 0, 0]); // 旧版 IFields 标志非零。
                bytes.extend([0; 12]); // 两个键计数和一个规则计数。
                if version == 6 {
                    bytes.extend(23_u16.to_le_bytes());
                }

                for end in 0..bytes.len() {
                    let mut reader = Reader::new(&bytes[..end]);
                    let mut refs = RefTable10_1::new();
                    assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
                }
                let end = bytes.len();
                bytes.push(0xa5);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let body = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
                let Some(DeChild10_1::RelationshipClass(body)) = body.body.as_deref() else {
                    panic!("应读取关系类数据元素");
                };
                let indexes = match &body.base {
                    DeRelationshipBase10_1::Table(table)
                        if class_id == DE_RELATIONSHIP_CLASS_CLASS_ID =>
                    {
                        assert_eq!(table.raw_short, 17);
                        &table.indexes
                    }
                    DeRelationshipBase10_1::CompactTable(table)
                        if class_id == DE_RELATIONSHIP_CLASS_INFO_CLASS_ID =>
                    {
                        assert_eq!(table.table_short, 17);
                        &table.indexes
                    }
                    _ => panic!("v5–6 应读取对应的 DETableBase 布局"),
                };
                assert_eq!(indexes.body.as_ref().unwrap().raw_count, 0);
                assert_eq!(refs.registered_class(42), Some(INDEXES_CLASS_ID));
                assert_eq!(body.metadata.as_ref().unwrap().shorts, [0, 1, 0]);
                assert_eq!(body.raw_rule_count, Some(0));
                assert_eq!(body.fields, None);
                assert_eq!(body.v6, (version == 6).then_some(23));
                assert_eq!(reader.position(), end);
            }
        }
    }

    #[test]
    fn vpf_coverage_reads_its_own_and_base_versions() {
        let mut bytes = DE_VPF_COVERAGE_CLASS_ID.to_vec();
        bytes.extend(40_u32.to_le_bytes());
        bytes.extend([1, 0, 0, 0, 1, 0]);
        bytes.extend([0; 8]);
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]);
        }
        bytes.extend([0; 54]);
        for cut in 0..bytes.len() {
            assert!(
                read_gp_data_element_child(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let slot =
            read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
        assert!(matches!(
            slot.body.as_deref(),
            Some(DeChild10_1::FeatureDataset {
                version: 1,
                vpf_base_version: Some(0),
                geo_dataset_version: 1,
                ..
            })
        ));
        assert_eq!(reader.position(), end);
        bytes[22] = 2;
        assert_eq!(read_gp_data_element_child(
            &mut Reader::new(&bytes),
            1024,
            &mut RefTable10_1::new(),
            0,
        ).unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn las_dataset_reads_tail_only_in_version_two() {
        for version in 0_u16..=2 {
            let mut bytes = DE_LAS_DATASET_CLASS_ID.to_vec();
            bytes.extend(39_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 10]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 54]);
            if version == 2 {
                for value in [-1_i32, 2, 3] {
                    bytes.extend(value.to_le_bytes());
                }
                for value in [4_u16, u16::MAX, 6] {
                    bytes.extend(value.to_le_bytes());
                }
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let Some(DeChild10_1::LasDataset {
                version: parsed_version,
                tail,
                ..
            }) = slot.body.as_deref()
            else {
                panic!("应读取 DELasDataset");
            };
            assert_eq!(*parsed_version, version);
            assert_eq!(
                *tail,
                (version == 2).then_some(DeLasDatasetTail10_1 {
                    raw_longs: [-1, 2, 3],
                    raw_shorts: [4, u16::MAX, 6],
                })
            );
            assert_eq!(reader.position(), end);

            bytes[20] = 3;
            assert_eq!(
                read_gp_data_element_child(
                    &mut Reader::new(&bytes),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .unwrap_err()
                .kind(),
                ErrorKind::InvalidData,
            );
        }
    }

    #[test]
    fn coverage_data_element_reads_property_set() {
        let mut bytes = DE_COVERAGE_CLASS_ID.to_vec();
        bytes.extend(37_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 10]);
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]);
        }
        bytes.extend([0; 54]);
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(38_u32.to_le_bytes());
        bytes.extend([1, 0, 0, 0, 0, 0]);
        for cut in 0..bytes.len() {
            assert!(
                read_gp_data_element_child(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
        let Some(DeChild10_1::Coverage {
            version: 1,
            properties,
            ..
        }) = object.body.as_deref()
        else {
            panic!("应读取 DECoverage");
        };
        assert!(properties.body.is_some());
        assert_eq!(refs.registered_class(38), Some(PROPERTY_SET_CLASS_ID));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn tin_data_element_reads_tail_and_fields_slot() {
        let mut bytes = DE_TIN_CLASS_ID.to_vec();
        bytes.extend(35_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 10]);
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]);
        }
        bytes.extend([0; 54]);
        bytes.extend((-0.0_f64).to_le_bytes());
        for value in [1_u16, 0xffff, 3, 4] {
            bytes.extend(value.to_le_bytes());
        }
        let mut refs = RefTable10_1::new();
        let mut fields = FIELDS_CLASS_ID.to_vec();
        fields.extend(36_u32.to_le_bytes());
        read_object_entry_10_1(&mut Reader::new(&fields), &mut refs).unwrap();
        bytes.extend(fields);
        for cut in 0..bytes.len() {
            assert!(
                read_gp_data_element_child(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut refs.clone(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
        let Some(DeChild10_1::Tin {
            value_bits,
            raw_shorts,
            fields,
            ..
        }) = object.body.as_deref()
        else {
            panic!("应读取 DETin");
        };
        assert_eq!(*value_bits, (-0.0_f64).to_bits());
        assert_eq!(*raw_shorts, [1, 0xffff, 3, 4]);
        assert!(matches!(
            fields.entry,
            ObjectEntry10_1::BackReference { key: 36, .. }
        ));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn cad_drawing_dataset_reads_four_trailing_shorts() {
        let mut bytes = DE_CAD_DRAWING_DATASET_CLASS_ID.to_vec();
        bytes.extend(34_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 10]);
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]);
        }
        bytes.extend([0; 54]);
        for value in [1_u16, 0xffff, 3, 4] {
            bytes.extend(value.to_le_bytes());
        }
        for cut in 0..bytes.len() {
            assert!(
                read_gp_data_element_child(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let object =
            read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
        assert!(matches!(
            object.body.as_deref(),
            Some(DeChild10_1::CadDrawingDataset {
                version: 1,
                raw_shorts: [1, 0xffff, 3, 4],
                ..
            })
        ));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn catalog_dataset_reads_signed_version_and_trailing_longs() {
        for version in [1_u16, u16::MAX] {
            let mut bytes = DE_CATALOG_DATASET_CLASS_ID.to_vec();
            bytes.extend(34_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 10]); // DEGeoDataset、DEDataset、DataElement、GPValue。
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 54]); // 基类定宽字段、空范围和空间参考。
            bytes.extend((-2_i32).to_le_bytes());
            bytes.extend(11_i32.to_le_bytes());
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.extend(DE_CATALOG_DATASET_CLASS_ID);
            bytes.extend(34_u32.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                object.body.as_deref(),
                Some(DeChild10_1::CatalogDataset { version: value, raw_longs: [-2, 11], .. })
                    if *value == version
            ));
            assert_eq!(reader.position(), end);
            let back_reference =
                read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                back_reference.entry,
                ObjectEntry10_1::BackReference { key: 34, .. }
            ));
            assert!(back_reference.body.is_none());
            assert_eq!(reader.position(), bytes.len());
        }
    }

    #[test]
    fn server_elements_read_data_element_and_two_strings() {
        for class_id in [
            DE_SERVER_CONNECTION_CLASS_ID,
            DE_GEO_DATA_SERVER_CLASS_ID,
            DE_MAP_SERVER_CLASS_ID,
            DE_GLOBE_SERVER_CLASS_ID,
            DE_GP_SERVER_CLASS_ID,
            DE_IMAGE_SERVER_CLASS_ID,
            DE_WCS_COVERAGE_CLASS_ID,
            DE_WMS_MAP_CLASS_ID,
        ] {
            let mut bytes = class_id.to_vec();
            bytes.extend(35_u32.to_le_bytes());
            bytes.extend([1, 0, 0, 0, 1, 0, 7, 0]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 10]);
            for value in [b'A', b'B'] {
                bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
            }
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::Server {
                version,
                element,
                strings,
            }) = object.body.as_deref()
            else {
                panic!("应读取服务数据元素");
            };
            assert_eq!(*version, 1);
            assert_eq!(element.gp_value_short, 7);
            assert_eq!(strings, &["A", "B"]);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn de_workspace_reads_version_fields_in_wire_order() {
        for version in 0_u16..=5 {
            let mut bytes = DE_WORKSPACE_CLASS_ID.to_vec();
            bytes.extend(36_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 6]);
            let empty = [2, 0, 0, 0, 0, 0];
            for _ in 0..3 {
                bytes.extend(empty);
            }
            bytes.extend([0; 10]);
            if version >= 3 {
                bytes.extend([1, 0, 0, 0, b'F', 0, 0, 0]);
                bytes.extend(empty);
                bytes.extend(empty);
            }
            bytes.extend([0; 16]);
            bytes.extend((-1_i32).to_le_bytes());
            if version == 1 {
                bytes.extend(11_i32.to_le_bytes());
            }
            if version >= 4 {
                for value in [12_i32, 13, 14] {
                    bytes.extend(value.to_le_bytes());
                }
            }
            if version == 5 {
                bytes.extend([4, 0, 0, 0, b'T', 0, 0, 0]);
            }
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::Workspace(workspace)) = object.body.as_deref() else {
                panic!("应读取 DEWorkspace");
            };
            assert_eq!(workspace.version, version);
            assert_eq!(
                workspace.factory.as_ref().map(|factory| factory.0.as_str()),
                (version >= 3).then_some("F")
            );
            assert_eq!(workspace.raw_domain_count, -1);
            assert_eq!(workspace.v1, (version == 1).then_some(11));
            assert_eq!(workspace.v4, (version >= 4).then_some([12, 13, 14]));
            assert_eq!(workspace.v5.as_deref(), (version == 5).then_some("T"));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn de_raster_dataset_reads_storage_slot_only_in_v2() {
        for version in 0_u16..=2 {
            let mut bytes = DE_RASTER_DATASET_CLASS_ID.to_vec();
            bytes.extend(40_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 10]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 54]);
            for value in [b'A', b'B', b'C'] {
                bytes.extend([4, 0, 0, 0, value, 0, 0, 0]);
            }
            bytes.extend((-4_i32).to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            if version == 2 {
                bytes.extend([0; 16]);
            }
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::RasterDataset {
                strings,
                raw_long,
                raw_short,
                storage,
                ..
            }) = object.body.as_deref()
            else {
                panic!("应读取 DERasterDataset");
            };
            assert_eq!(strings, &["A", "B", "C"]);
            assert_eq!((*raw_long, *raw_short), (-4, 7));
            assert_eq!(
                storage.as_ref().map(|slot| slot.entry),
                (version == 2).then_some(ObjectEntry10_1::Null)
            );
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn raster_storage_def_reads_v4_and_nested_point_boundary() {
        let mut bytes = RASTER_STORAGE_DEF_CLASS_ID.to_vec();
        bytes.extend(51_u32.to_le_bytes());
        bytes.extend(4_u16.to_le_bytes());
        for value in 1_i32..=6 {
            bytes.extend(value.to_le_bytes());
        }
        bytes.push(1);
        bytes.extend(1.25_f64.to_le_bytes());
        bytes.extend(2.5_f64.to_le_bytes());
        bytes.extend(POINT_CLASS_ID);
        bytes.extend(52_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(20_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(3.0_f64.to_le_bytes());
        bytes.extend(4.0_f64.to_le_bytes());
        bytes.extend(8_u16.to_le_bytes());
        bytes.extend([4, 0, 0, 0, b'T', 0, 0, 0]);
        for cut in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_raster_storage_def_slot(&mut reader, 1024, &mut refs).is_err());
        }
        let end = bytes.len();
        bytes.extend(RASTER_STORAGE_DEF_CLASS_ID);
        bytes.extend(51_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_raster_storage_def_slot(&mut reader, 1024, &mut refs)
            .unwrap()
            .body
            .unwrap();
        assert_eq!(body.longs, [1, 2, 3, 4, 5, 6]);
        assert_eq!(body.xy_bits, Some([1.25_f64.to_bits(), 2.5_f64.to_bits()]));
        let point = body.point.unwrap();
        assert_eq!(
            point.entry,
            ObjectEntry10_1::NewObject {
                class_id: POINT_CLASS_ID,
                key: 52
            }
        );
        assert_eq!(point.body.unwrap().version, 1);
        assert_eq!(refs.registered_class(52), Some(POINT_CLASS_ID));
        assert_eq!(body.short, Some(8));
        assert_eq!(body.text.as_deref(), Some("T"));
        assert_eq!(reader.position(), end);
        let back_reference = read_raster_storage_def_slot(&mut reader, 1024, &mut refs).unwrap();
        assert_eq!(
            back_reference.entry,
            ObjectEntry10_1::BackReference {
                class_id: RASTER_STORAGE_DEF_CLASS_ID,
                key: 51,
            }
        );
        assert!(back_reference.body.is_none());
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn de_raster_band_reads_compact_table_and_versioned_variant() {
        for version in 0_u16..=3 {
            let mut bytes = DE_RASTER_BAND_CLASS_ID.to_vec();
            bytes.extend(53_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([0; 10]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 22]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 32]);
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend(1.5_f64.to_le_bytes());
            bytes.extend(2.5_f64.to_le_bytes());
            bytes.extend([0; 8]);
            if version == 3 {
                bytes.extend(9_i32.to_le_bytes());
            }
            if version >= 2 {
                bytes.extend(3_u16.to_le_bytes());
                bytes.extend(42_i32.to_le_bytes());
            } else {
                bytes.extend([0; 24]);
            }
            bytes.extend(11_i32.to_le_bytes());
            bytes.extend(12_i32.to_le_bytes());
            bytes.extend([0; 32]);
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::RasterBand(body)) = object.body.as_deref() else {
                panic!("应读取 DERasterBand");
            };
            assert_eq!(body.table.table_short, 7);
            assert_eq!(body.raw_short, 4);
            assert_eq!(body.v3, (version == 3).then_some(9));
            assert_eq!(body.trailing_longs, [11, 12]);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn file_system_elements_end_after_data_element() {
        for class_id in [
            DE_FILE_CLASS_ID,
            DE_FOLDER_CLASS_ID,
            DE_MAP_DOCUMENT_CLASS_ID,
            DE_DISK_CONNECTION_CLASS_ID,
            DE_CATALOG_ROOT_CLASS_ID,
            DE_REMOTE_DATABASE_FOLDER_CLASS_ID,
            DE_SPATIAL_REFERENCES_FOLDER_CLASS_ID,
            DE_PRJ_FILE_CLASS_ID,
        ] {
            let mut bytes = class_id.to_vec();
            bytes.extend(54_u32.to_le_bytes());
            bytes.extend([1, 0, 0, 0, 1, 0, 7, 0]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 10]);
            if class_id == DE_PRJ_FILE_CLASS_ID {
                bytes.extend([0; 16]);
            }
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::FileSystem {
                version,
                element,
                spatial_reference,
            }) = object.body.as_deref()
            else {
                panic!("应读取文件数据元素");
            };
            assert_eq!(*version, 1);
            assert_eq!(element.gp_value_short, 7);
            assert_eq!(
                spatial_reference.as_deref(),
                (class_id == DE_PRJ_FILE_CLASS_ID).then_some(&SpatialReferenceEntry10_1::Null),
            );
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_feature_record_set_layer_reads_drawing_and_templates() {
        use crate::layers::annotate_props::EDIT_TEMPLATE_MANAGER_CLASS_ID;

        for version in 0_u16..=3 {
            let mut bytes = GP_FEATURE_RECORD_SET_LAYER_CLASS_ID.to_vec();
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 9, 0]);
            bytes.extend([0; 16]);
            let empty_text = [2, 0, 0, 0, 0, 0];
            if version >= 2 {
                bytes.extend(empty_text);
                bytes.extend(7_u16.to_le_bytes());
            }
            bytes.extend(empty_text);
            bytes.extend(empty_text);
            bytes.extend(FEATURE_LAYER_DRAWING_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend([0; 16]);
            if version == 3 {
                bytes.extend(EDIT_TEMPLATE_MANAGER_CLASS_ID);
                bytes.extend(3_u32.to_le_bytes());
                bytes.extend(1_i16.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
            }
            let read = |reader: &mut Reader<'_>| {
                read_gp_coded_value_slot(reader, 1024, &mut RefTable10_1::new(), 0)
            };
            for cut in 0..bytes.len() {
                assert!(read(&mut Reader::new(&bytes[..cut])).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let coded = read(&mut reader).unwrap().body.unwrap();
            let GpCodedValue10_1::FeatureRecordSetLayer(layer) = coded.value else {
                panic!("应读取 GPFeatureRecordSetLayer");
            };
            assert!(matches!(
                layer.drawing.body.as_deref(),
                Some(LayerDrawingDescription10_1::Feature(_))
            ));
            assert_eq!(
                layer.record_set.v2,
                (version >= 2).then_some((String::new(), 7))
            );
            assert_eq!(
                layer
                    .templates
                    .and_then(|slot| slot.body)
                    .map(|body| body.version),
                (version == 3).then_some(1)
            );
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_record_set_reads_inline_rows_and_byte_arrays() {
        use crate::objects::property_set::PropertyValue10_1;
        use crate::stream::variant_array::ClassVariantArray10_1;

        for version in 0_u16..=2 {
            let mut bytes = GP_RECORD_SET_CLASS_ID.to_vec();
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 9, 0]);
            bytes.extend(RECORD_SET_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(FIELDS_CLASS_ID);
            bytes.extend(3_u32.to_le_bytes());
            bytes.extend([1, 0, 0, 0, 0, 0]);
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(0x2011_u16.to_le_bytes());
            bytes.extend([0; 14]);
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend([0x12, 0x34, 0x56]);
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            let empty_text = [2, 0, 0, 0, 0, 0];
            if version == 2 {
                bytes.extend(empty_text);
                bytes.extend(7_u16.to_le_bytes());
            }
            bytes.extend(empty_text);
            let read = |reader: &mut Reader<'_>| {
                read_gp_coded_value_slot(reader, 1024, &mut RefTable10_1::new(), 0)
            };
            for cut in 0..bytes.len() {
                assert!(read(&mut Reader::new(&bytes[..cut])).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let body = read(&mut reader).unwrap().body.unwrap();
            let GpCodedValue10_1::RecordSet(value) = body.value else {
                panic!("应读取 GPRecordSet");
            };
            let records = value.records.body.unwrap();
            assert_eq!(records.raw_count, 2);
            assert_eq!(
                records.rows[0].values,
                [PropertyValueObjectStream10_1::Inline(
                    PropertyValue10_1::Array(ClassVariantArray10_1::Fixed {
                        vartype: 17,
                        element_bytes: 1,
                        raw: vec![0x12, 0x34, 0x56],
                    })
                )]
            );
            assert!(records.rows[1].values.is_empty());
            assert_eq!(value.v2, (version == 2).then_some((String::new(), 7)));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_table_view_reads_versioned_slots() {
        for version in 0_u16..=2 {
            let mut bytes = GP_TABLE_VIEW_CLASS_ID.to_vec();
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 9, 0]);
            let empty_text = [2, 0, 0, 0, 0, 0];
            for _ in 0..2 {
                bytes.extend(empty_text);
                bytes.extend([0; 16]);
            }
            if version == 2 {
                bytes.extend([0; 16]);
                bytes.extend(GP_FIELD_INFO_CLASS_ID);
                bytes.extend(2_u32.to_le_bytes());
                bytes.extend([1, 0, 1, 0, 7, 0]);
                bytes.extend(0_i32.to_le_bytes());
            }
            let read = |reader: &mut Reader<'_>| {
                read_gp_coded_value_slot(reader, 1024, &mut RefTable10_1::new(), 0)
            };
            for cut in 0..bytes.len() {
                assert!(read(&mut Reader::new(&bytes[..cut])).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let coded = read(&mut reader).unwrap().body.unwrap();
            let GpCodedValue10_1::TableView(view) = coded.value else {
                panic!("应读取 GPTableView");
            };
            assert_eq!((coded.version, coded.base_short), (version, 9));
            assert_eq!(view.table.entry, ObjectEntry10_1::Null);
            assert_eq!(view.fid_set.entry, ObjectEntry10_1::Null);
            assert_eq!(
                view.tail
                    .and_then(|tail| tail.field_info.body)
                    .map(|body| body.base_short),
                (version == 2).then_some(7)
            );
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_raster_layer_reads_text_and_field_reference() {
        for version in 0_u16..=1 {
            let mut bytes = GP_RASTER_LAYER_CLASS_ID.to_vec();
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([2, 0, 1, 0, 9, 0]);
            let empty_text = [2, 0, 0, 0, 0, 0];
            bytes.extend(empty_text);
            bytes.extend([0; 32]);
            bytes.extend(empty_text);
            bytes.extend(FIELD_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());

            let mut refs = RefTable10_1::new();
            let mut field_header = FIELD_CLASS_ID.to_vec();
            field_header.extend(2_u32.to_le_bytes());
            read_object_entry_10_1(&mut Reader::new(&field_header), &mut refs).unwrap();
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_layer_slot(&mut Reader::new(&bytes[..cut]), 1024, &mut refs.clone(), 0)
                        .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let coded = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0)
                .unwrap()
                .body
                .unwrap();
            let GpCodedValue10_1::RasterLayer(layer) = coded.value else {
                panic!("应读取 GPRasterLayer");
            };
            assert_eq!((coded.version, layer.version), (version, 2));
            let raster = layer.raster.unwrap();
            assert!(raster.text.is_empty());
            assert!(matches!(
                raster.field.entry,
                ObjectEntry10_1::BackReference { key: 2, .. }
            ));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_layers_read_derived_version_and_base() {
        for class_id in [
            GP_TIN_LAYER_CLASS_ID,
            GP_TERRAIN_LAYER_CLASS_ID,
            GP_TOPOLOGY_LAYER_CLASS_ID,
            GP_NETWORK_DATASET_LAYER_CLASS_ID,
            GP_LAS_DATASET_LAYER_CLASS_ID,
        ] {
            for version in 0_u16..=1 {
                let mut bytes = class_id.to_vec();
                bytes.extend(1_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend([2, 0, 1, 0, 9, 0]);
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.extend([0; 32]);
                for cut in 0..bytes.len() {
                    assert!(
                        read_gp_layer_slot(
                            &mut Reader::new(&bytes[..cut]),
                            1024,
                            &mut RefTable10_1::new(),
                            0
                        )
                        .is_err()
                    );
                }
                if class_id == GP_LAS_DATASET_LAYER_CLASS_ID && version == 0 {
                    assert_eq!(
                        read_gp_layer_slot(
                            &mut Reader::new(&bytes),
                            1024,
                            &mut RefTable10_1::new(),
                            0
                        )
                        .unwrap_err()
                        .kind(),
                        ErrorKind::InvalidData
                    );
                    continue;
                }
                let end = bytes.len();
                bytes.push(0xa5);
                let mut reader = Reader::new(&bytes);
                let coded =
                    read_gp_coded_value_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0)
                        .unwrap()
                        .body
                        .unwrap();
                let layer = match (class_id, coded.value) {
                    (GP_TIN_LAYER_CLASS_ID, GpCodedValue10_1::TinLayer(layer))
                    | (GP_TERRAIN_LAYER_CLASS_ID, GpCodedValue10_1::TerrainLayer(layer))
                    | (GP_TOPOLOGY_LAYER_CLASS_ID, GpCodedValue10_1::TopologyLayer(layer))
                    | (
                        GP_NETWORK_DATASET_LAYER_CLASS_ID,
                        GpCodedValue10_1::NetworkDatasetLayer(layer),
                    )
                    | (GP_LAS_DATASET_LAYER_CLASS_ID, GpCodedValue10_1::LasDatasetLayer(layer)) => {
                        layer
                    }
                    _ => panic!("应读取对应的 GP 图层"),
                };
                assert_eq!(
                    (coded.version, coded.base_version, coded.base_short),
                    (version, 1, 9)
                );
                assert_eq!(layer.derived_version, Some(version));
                assert_eq!(layer.version, 2);
                assert_eq!(reader.position(), end);

                let mut reader = Reader::new(&bytes);
                let slot =
                    read_gp_layer_slot(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
                assert_eq!(slot.body.unwrap(), layer);
                assert_eq!(reader.position(), end);
                bytes[20] = 2;
                assert_eq!(
                    read_gp_layer_slot(&mut Reader::new(&bytes), 1024, &mut RefTable10_1::new(), 0)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
            }
        }
    }

    #[test]
    fn gp_feature_layers_read_versioned_feature_tail() {
        for class_id in [GP_FEATURE_LAYER_CLASS_ID, GP_RASTER_CATALOG_LAYER_CLASS_ID] {
            for version in 0_u16..=2 {
                let mut bytes = class_id.to_vec();
                bytes.extend(1_u32.to_le_bytes());
                bytes.extend(version.to_le_bytes());
                bytes.extend([2, 0, 1, 0, 9, 0]);
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.extend([0; 32]);
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.extend([0; 16]);
                if version == 2 {
                    bytes.extend([0; 16]);
                    bytes.extend(GP_FIELD_INFO_CLASS_ID);
                    bytes.extend(2_u32.to_le_bytes());
                    bytes.extend([1, 0, 1, 0, 7, 0]);
                    bytes.extend(0_i32.to_le_bytes());
                }
                let read = |reader: &mut Reader<'_>| {
                    read_gp_coded_value_slot(reader, 1024, &mut RefTable10_1::new(), 0)
                };
                for cut in 0..bytes.len() {
                    assert!(read(&mut Reader::new(&bytes[..cut])).is_err());
                }
                let end = bytes.len();
                bytes.push(0x5a);
                let mut reader = Reader::new(&bytes);
                let coded = read(&mut reader).unwrap().body.unwrap();
                assert_eq!(
                    (coded.version, coded.base_version, coded.base_short),
                    (version, 1, 9)
                );
                let layer = match (class_id, coded.value) {
                    (GP_FEATURE_LAYER_CLASS_ID, GpCodedValue10_1::FeatureLayer(layer))
                    | (
                        GP_RASTER_CATALOG_LAYER_CLASS_ID,
                        GpCodedValue10_1::RasterCatalogLayer(layer),
                    ) => layer,
                    _ => panic!("应读取对应的 GP 要素图层"),
                };
                let fields = layer.feature_layer.unwrap();
                assert_eq!(fields.fid_set.entry, ObjectEntry10_1::Null);
                assert_eq!(fields.table.is_some(), version == 2);
                assert_eq!(
                    fields
                        .field_info
                        .and_then(|slot| slot.body)
                        .map(|body| body.base_short),
                    (version == 2).then_some(7)
                );
                assert_eq!(reader.position(), end);
            }
        }
    }

    #[test]
    fn gp_na_layer_reads_versioned_tail() {
        use crate::layers::na_class::NA_CLASS_DEFINITION_CLASS_ID;
        use crate::layers::na_layer::{
            NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID, NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID,
            NA_LOCATOR_FEATURE_AGENT_CLASS_ID,
        };

        for version in [-1_i16, 0, 1, 2, 3] {
            let mut bytes = GP_NA_LAYER_CLASS_ID.to_vec();
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 2, 0, 1, 0, 9, 0]);
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 32]);
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend([0; 16]);
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(NA_CLASS_DEFINITION_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend([0; 54]);
            bytes.extend([0; 16]);
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(NA_LOCATOR_FEATURE_AGENT_CLASS_ID);
            bytes.extend(3_u32.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(4_i32.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend([b'X', 0, 0, 0]);
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(NA_LOCATOR_FEATURE_AGENT_CLASS_ID);
            bytes.extend(3_u32.to_le_bytes());
            bytes.extend(17_u16.to_le_bytes());
            bytes.extend([0; 32]);
            if version >= 1 {
                bytes.extend(NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID);
                bytes.extend(4_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend(NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID);
                bytes.extend(5_u32.to_le_bytes());
                bytes.extend([0; 2 + 8 + 4 + 16]);
                bytes.extend([0x42; 16]);
                bytes.extend(5_u32.to_le_bytes());
            }
            if version >= 2 {
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend([0; 16]);
            }
            if version >= 3 {
                bytes.extend(18_u16.to_le_bytes());
            }
            let parse = |reader: &mut Reader<'_>| {
                read_gp_coded_value_slot(reader, 1024, &mut RefTable10_1::new(), 0)
            };
            for cut in 0..bytes.len() {
                assert!(
                    parse(&mut Reader::new(&bytes[..cut])).is_err(),
                    "截断于 {cut}"
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let coded = parse(&mut reader).unwrap().body.unwrap();
            let GpCodedValue10_1::NaLayer(layer) = coded.value else {
                panic!("应读取 GPNALayer");
            };
            assert_eq!(coded.version as i16, version);
            let na = layer.na.unwrap();
            assert!(na.definitions[0].body.is_some());
            assert_eq!(na.agents[0].body.as_ref().unwrap().source, "X");
            assert!(matches!(
                na.agents[1].entry,
                ObjectEntry10_1::BackReference { key: 3, .. }
            ));
            assert_eq!(na.raw_short, 17);
            assert_eq!(na.parameters.is_some(), version >= 1);
            if let Some(parameters) = na.parameters {
                let body = parameters.body.unwrap();
                assert!(body.items[0].1.is_some());
                assert!(matches!(
                    body.items[1].0,
                    ObjectEntry10_1::BackReference { key: 5, .. }
                ));
            }
            assert_eq!(
                na.tables.map(|tables| tables.len()),
                (version >= 2).then_some(1)
            );
            assert_eq!(na.tail_short, (version == 3).then_some(18));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn gp_composite_derivatives_read_as_coded_values_and_layer_references() {
        for class_id in [
            GP_MOSAIC_LAYER_CLASS_ID,
            GP_CADASTRAL_FABRIC_LAYER_CLASS_ID,
            GP_GROUP_LAYER_CLASS_ID,
        ] {
            let mut bytes = class_id.to_vec();
            bytes.extend(1_u32.to_le_bytes());
            let version_offset = bytes.len();
            bytes.extend([1, 0, 0, 0, 2, 0, 1, 0, 9, 0]);
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 32]);
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(class_id);
            bytes.extend(1_u32.to_le_bytes());
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_coded_value_slot(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let mut invalid = bytes.clone();
            invalid[version_offset] = 2;
            assert_eq!(
                read_gp_coded_value_slot(
                    &mut Reader::new(&invalid),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .unwrap_err()
                .kind(),
                ErrorKind::InvalidData
            );

            let end = bytes.len();
            bytes.extend(class_id);
            bytes.extend(1_u32.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let coded = read_gp_coded_value_slot(&mut reader, 1024, &mut refs, 0)
                .unwrap()
                .body
                .unwrap();
            assert_eq!(
                (coded.version, coded.base_version, coded.base_short),
                (1, 1, 9)
            );
            let layer = match (class_id, coded.value) {
                (GP_MOSAIC_LAYER_CLASS_ID, GpCodedValue10_1::MosaicLayer(layer))
                | (
                    GP_CADASTRAL_FABRIC_LAYER_CLASS_ID,
                    GpCodedValue10_1::CadastralFabricLayer(layer),
                )
                | (GP_GROUP_LAYER_CLASS_ID, GpCodedValue10_1::GroupLayer(layer)) => layer,
                _ => panic!("应读取对应的 GP 组合图层"),
            };
            let members = layer.composite.unwrap();
            assert_eq!((members.base_version, members.raw_count), (0, 1));
            assert!(matches!(
                members.layers[0].entry,
                ObjectEntry10_1::BackReference { key: 1, .. }
            ));
            assert_eq!(reader.position(), end);
            let slot = read_gp_layer_slot(&mut reader, 1024, &mut refs, 0).unwrap();
            assert!(matches!(
                slot.entry,
                ObjectEntry10_1::BackReference { key: 1, .. }
            ));
            assert_eq!(reader.position(), bytes.len());
        }
    }

    #[test]
    fn gp_composite_layer_reads_members_and_back_reference() {
        let mut bytes = GP_COMPOSITE_LAYER_CLASS_ID.to_vec();
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([1, 0, 0, 0, 2, 0, 1, 0, 9, 0]);
        bytes.extend([2, 0, 0, 0, 0, 0]);
        bytes.extend([0; 32]);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(GP_COMPOSITE_LAYER_CLASS_ID);
        bytes.extend(1_u32.to_le_bytes());
        for cut in 0..bytes.len() {
            assert!(
                read_gp_layer_slot(
                    &mut Reader::new(&bytes[..cut]),
                    1024,
                    &mut RefTable10_1::new(),
                    0,
                )
                .is_err()
            );
        }
        let end = bytes.len();
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let outer = read_gp_layer_slot(&mut reader, 1024, &mut refs, 0).unwrap();
        let members = outer.body.unwrap().composite.unwrap();
        assert_eq!(
            (members.version, members.base_version, members.raw_count),
            (1, 0, 2)
        );
        assert_eq!(members.layers[0].entry, ObjectEntry10_1::Null);
        assert!(matches!(
            members.layers[1].entry,
            ObjectEntry10_1::BackReference { key: 1, .. }
        ));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn de_layer_reads_gp_layer_and_legacy_slot() {
        for version in 0_u16..=2 {
            let mut bytes = DE_LAYER_CLASS_ID.to_vec();
            bytes.extend(56_u32.to_le_bytes());
            bytes.extend([1, 0, 0, 0, 1, 0, 7, 0]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 10]);
            bytes.extend(GP_LAYER_CLASS_ID);
            bytes.extend(57_u32.to_le_bytes());
            let version_offset = bytes.len();
            bytes.extend(version.to_le_bytes());
            bytes.extend([1, 0, 9, 0, 4, 0, 0, 0, b'L', 0, 0, 0]);
            bytes.extend([0; 32]);
            if version < 2 {
                bytes.extend([0; 16]);
            }
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let object = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::Layer { layer, .. }) = object.body.as_deref() else {
                panic!("应读取 DELayer");
            };
            let layer = layer.body.as_deref().unwrap();
            assert_eq!(layer.version, version);
            assert_eq!(layer.gp_value_short, 9);
            assert_eq!(layer.name, "L");
            assert_eq!(layer.element.entry, ObjectEntry10_1::Null);
            assert_eq!(
                layer.legacy_layer.as_ref().map(|slot| slot.entry),
                (version < 2).then_some(ObjectEntry10_1::Null),
            );
            assert_eq!(reader.position(), end);

            bytes[version_offset] = 3;
            let err = read_gp_data_element_child(
                &mut Reader::new(&bytes),
                1024,
                &mut RefTable10_1::new(),
                0,
            )
            .unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidData);
        }
    }

    #[test]
    fn file_feature_classes_use_compact_layout() {
        for class_id in [
            DE_SHAPE_FILE_CLASS_ID,
            DE_COVERAGE_FEATURE_CLASS_ID,
            DE_FEATURE_CLASS_INFO_CLASS_ID,
            DE_RASTER_CATALOG_INFO_CLASS_ID,
        ] {
            let mut bytes = class_id.to_vec();
            bytes.extend(55_u32.to_le_bytes());
            let derived_version = (class_id != DE_FEATURE_CLASS_INFO_CLASS_ID).then_some(1_u16);
            if let Some(version) = derived_version {
                bytes.extend(version.to_le_bytes());
            }
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0; 10]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 22]);
            bytes.extend(9_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 32]);
            bytes.extend((-4_i32).to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 38]);
            if class_id == DE_COVERAGE_FEATURE_CLASS_ID {
                bytes.extend((-2_i32).to_le_bytes());
                bytes.extend(7_u16.to_le_bytes());
                bytes.extend(9_i32.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                for is_new in [true, false] {
                    bytes.extend(GP_ARC_INFO_ITEM_CLASS_ID);
                    bytes.extend(56_u32.to_le_bytes());
                    if is_new {
                        bytes.extend([0; 6]);
                        bytes.extend([2, 0, 0, 0, 0, 0].repeat(2));
                        bytes.extend([0; 26]);
                    }
                }
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            assert!(matches!(
                slot.body.as_deref(),
                Some(DeChild10_1::FeatureClass {
                    derived_version: parsed_derived_version,
                    version: 0,
                    table: DeFeatureClassTable10_1::Compact(DeCompactTable10_1 {
                        table_short: 9,
                        ..
                    }),
                    raw_longs: [-4, 7],
                    strings: None,
                    ..
                }) if *parsed_derived_version == derived_version
            ));
            let Some(DeChild10_1::FeatureClass { coverage, .. }) = slot.body.as_deref() else {
                panic!("应读取文件要素类");
            };
            assert_eq!(coverage.is_some(), class_id == DE_COVERAGE_FEATURE_CLASS_ID);
            if let Some(tail) = coverage {
                assert_eq!(
                    (tail.longs, tail.short, tail.raw_item_count),
                    ([-2, 9], 7, 2)
                );
                assert!(tail.items[0].body.is_some());
                assert!(matches!(
                    tail.items[1].entry,
                    ObjectEntry10_1::BackReference { key: 56, .. }
                ));
            }
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn file_table_elements_use_compact_table() {
        for class_id in [
            DE_DBASE_TABLE_CLASS_ID,
            DE_VPF_TABLE_CLASS_ID,
            DE_ARC_INFO_TABLE_CLASS_ID,
            DE_TABLE_INFO_CLASS_ID,
        ] {
            let mut bytes = class_id.to_vec();
            bytes.extend(55_u32.to_le_bytes());
            if class_id != DE_TABLE_INFO_CLASS_ID {
                bytes.extend([1, 0]);
            }
            bytes.extend([0; 10]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 22]);
            bytes.extend(9_u16.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            bytes.extend([0; 32]);
            if class_id == DE_ARC_INFO_TABLE_CLASS_ID {
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(GP_ARC_INFO_ITEM_CLASS_ID);
                bytes.extend(56_u32.to_le_bytes());
                bytes.extend([0; 6]);
                bytes.extend([2, 0, 0, 0, 0, 0].repeat(2));
                bytes.extend([0; 26]);
            }
            for cut in 0..bytes.len() {
                assert!(
                    read_gp_data_element_child(
                        &mut Reader::new(&bytes[..cut]),
                        1024,
                        &mut RefTable10_1::new(),
                        0,
                    )
                    .is_err()
                );
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let object =
                read_gp_data_element_child(&mut reader, 1024, &mut RefTable10_1::new(), 0).unwrap();
            let table = match object.body.as_deref() {
                Some(DeChild10_1::ArcInfoTable {
                    version,
                    table,
                    raw_item_count,
                    items,
                }) if class_id == DE_ARC_INFO_TABLE_CLASS_ID => {
                    assert_eq!(*raw_item_count, 1);
                    assert!(items[0].body.is_some());
                    assert_eq!(*version, 1);
                    table
                }
                Some(DeChild10_1::FileTable { version, table })
                    if class_id != DE_ARC_INFO_TABLE_CLASS_ID =>
                {
                    assert_eq!(*version, 1);
                    table
                }
                Some(DeChild10_1::TableInfo(table)) if class_id == DE_TABLE_INFO_CLASS_ID => table,
                _ => panic!("应读取文件表数据元素"),
            };
            assert_eq!(table.table_short, 9);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn de_text_file_selects_base_by_version() {
        for version in 0_u16..=2 {
            let mut bytes = DE_TEXT_FILE_CLASS_ID.to_vec();
            bytes.extend(55_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            if version != 1 {
                bytes.extend([0; 4]);
            }
            bytes.extend([0; 6]);
            for _ in 0..3 {
                bytes.extend([2, 0, 0, 0, 0, 0]);
            }
            bytes.extend([0; 10]);
            if version != 1 {
                bytes.extend([0; 12]);
                bytes.extend(9_u16.to_le_bytes());
                bytes.extend([2, 0, 0, 0, 0, 0]);
                bytes.extend([0; 32]);
            }
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).is_err());
            }
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value = read_gp_data_element_child(&mut reader, 1024, &mut refs, 0).unwrap();
            let Some(DeChild10_1::TextFile {
                version: parsed_version,
                base,
            }) = value.body.as_deref()
            else {
                panic!("应读取 DETextFile");
            };
            assert_eq!(*parsed_version, version);
            match base {
                DeTextFileBase10_1::Element(_) => assert_eq!(version, 1),
                DeTextFileBase10_1::Table(table) => {
                    assert_ne!(version, 1);
                    assert_eq!(table.table_short, 9);
                }
            }
            assert_eq!(reader.position(), end);
        }
    }
}
