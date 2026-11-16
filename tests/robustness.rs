//! 跨读取器的异常输入健壮性测试（通过 `mxd2qgz::` 公共 API 访问）。
//!
//! 这些测试证明的是各严格读取器自身的健壮性共性：
//! - 对每个读取器的合法输入，任意截断前缀（含空输入）都返回明确错误且不 panic；
//! - 失败后游标不推进，且同一失败输入在同一 Reader 上重读得到相同错误类别
//!   （clone-trial 事务性的可观察行为）；
//! - `u32::MAX`/`usize::MAX` 级别的声明长度、码元数或限额参数不引发算术溢出、
//!   panic 或巨大分配，而是在分配前被明确拒绝；
//! - 未知类标识与键按 `object_stream` 契约原样保留；VariantStream 的整个
//!   `u16` VARTYPE 空间都有显式分支；
//! - 当前读取器不递归、不自我放大，顺序读取在输入耗尽时有界完成。
//!
//! 这些测试不证明 MXD 格式规则、字段归属、版本语义、对象边界或引用关系
//! 已经验证；它们只刻画合成输入上的实现性质，格式结论仍以原生研究记录
//! 为准。底层 `reader::Reader` 原语的同等性质已由 `tests/reader.rs` 与
//! `src/reader.rs` 单元测试覆盖，此处仅在超大长度一节触及公共原语，不重复。
//! 对象引用编码已由 10.1 原生研究闭合（键回引按已见键集合判定），循环
//! 引用、重复标识与缺失引用的语义覆盖见 `src/object_refs.rs` 与
//! `tests/object_refs.rs`；本清单把条目分类入口纳入统一的截断与游标共性。

use mxd2qgz::layers::annotate_props::{
    read_annotate_props_collection_body_10_1, read_annotation_vbscript_engine_body_10_1,
    read_bolp_body_10_1, read_display_expression_props_prefix_10_1,
    read_edit_template_manager_body_10_1, read_hot_link_expression_props_body_10_1,
    read_label_engine_props_body_10_1, read_line_label_placement_priorities_body_10_1,
    read_line_label_position_body_10_1, read_point_placement_priorities_body_10_1,
};
use mxd2qgz::layers::composite_graphics::read_composite_graphics_layer_prefix_10_1;
use mxd2qgz::layers::dep_tail::{
    read_time_body_10_1, read_time_data_family_10_1, read_time_extent_body_10_1,
    read_time_reference_body_10_1, read_time_zone_info_body_10_1,
};
use mxd2qgz::layers::fdo_legacy::read_fdo_graphics_layer_legacy_prefix_10_1;
use mxd2qgz::layers::feature_id_set::read_feature_id_set_body_10_1;
use mxd2qgz::layers::feature_layer::read_feature_layer_prefix_10_1;
use mxd2qgz::layers::feature_layer_tail::read_uid_body_10_1;
use mxd2qgz::layers::group_layer::read_group_layer_prefix_10_1;
use mxd2qgz::layers::layer_list::read_layer_list_10_1;
use mxd2qgz::layers::legend_item::read_base_legend_item_fields_10_1;
use mxd2qgz::layers::legend_item_family::{
    read_horizontal_bar_legend_item_body_10_1, read_nested_legend_item_body_10_1,
    read_vertical_legend_item_body_10_1,
};
use mxd2qgz::layers::patch::read_patch_body_10_1;
use mxd2qgz::maps::map_object::{read_map_prefix_10_1, read_map_suffix_10_1};
use mxd2qgz::maps::map_surround::{
    read_aoi_bookmark_10_1, read_feature_bookmark_10_1, read_legend_prefix_10_1,
};
use mxd2qgz::maps::map_surround_family::{
    read_legend_format_body_10_1, read_map_title_body_10_1, read_marker_north_arrow_body_10_1,
    read_scale_text_body_10_1, read_scalebar_body_10_1, read_scalebar_subclass_body_10_1,
};
use mxd2qgz::maps::maps_stream::{read_root_prefix_10_1, read_root_prefix_10_8};
use mxd2qgz::objects::callout_background::{
    read_balloon_callout_body_10_1, read_line_callout_body_10_1, read_marker_background_slot_10_1,
    read_marker_text_background_body_10_1, read_simple_line_callout_body_10_1,
};
use mxd2qgz::objects::color_ramp::{
    read_algorithmic_color_ramp_body_10_1, read_color_ramp_slot_10_1,
    read_multi_part_color_ramp_body_10_1, read_preset_color_ramp_body_10_1,
    read_random_color_ramp_body_10_1,
};
use mxd2qgz::objects::dataset_name::{
    read_feature_dataset_name_body_10_1, read_topology_name_body_10_1,
};
use mxd2qgz::objects::dataset_name_more::{
    read_dataset_name_slot_10_1, read_db_table_name_body_10_1,
    read_geometric_network_name_body_10_1, read_memory_relationship_class_name_body_10_1,
    read_query_table_name_body_10_1, read_rel_query_table_name_body_10_1,
    read_relationship_class_name_body_10_1,
};
use mxd2qgz::objects::display_color::{RGB_COLOR_CLASS_ID, read_rgb_color_body_10_1};
use mxd2qgz::objects::display_symbol::{
    SIMPLE_MARKER_SYMBOL_CLASS_ID, read_simple_marker_symbol_body_10_1, read_symbol_slot_full_10_1,
};
use mxd2qgz::objects::element_body::{
    LINE_ELEMENT_CLASS_ID, MARKER_ELEMENT_CLASS_ID, TEXT_ELEMENT_CLASS_ID, TEXT_SYMBOL_CLASS_ID,
    read_element_body_of_class_10_1,
};
use mxd2qgz::objects::element_decor::{
    read_background_slot_10_1, read_border_slot_10_1, read_shadow_slot_10_1,
    read_symbol_background_body_10_1, read_symbol_border_body_10_1, read_symbol_shadow_body_10_1,
};
use mxd2qgz::objects::envelope::read_envelope_10_1;
use mxd2qgz::objects::geometry::{
    POLYLINE_CLASS_ID, read_geometry_body_10_1, read_geometry_body_of_class_10_1,
    read_multipoint_body_10_1, read_point_body_10_1, read_polygon_body_10_1,
    read_polyline_body_10_1,
};
use mxd2qgz::objects::hyperlink::read_hyperlink_body_10_1;
use mxd2qgz::objects::number_format::{
    read_angle_format_body_10_1, read_currency_format_body_10_1,
    read_custom_number_format_body_10_1, read_fraction_format_body_10_1,
    read_latlon_format_body_10_1, read_numeric_format_body_10_1, read_percentage_format_body_10_1,
    read_rate_format_body_10_1, read_scientific_format_body_10_1,
};
use mxd2qgz::objects::ole_font::{read_std_font_body_10_1, read_std_picture_body_10_1};
use mxd2qgz::objects::picture_fill_symbol::{
    read_picture_fill_symbol_body_10_1, read_picture_fill_symbol_slot_10_1,
};
use mxd2qgz::objects::property_set::read_property_set_10_1;
use mxd2qgz::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use mxd2qgz::objects::spatial_reference_slots::{
    read_geographic_coordinate_system_10_1, read_unknown_coordinate_system_10_1,
};
use mxd2qgz::objects::variant_object_branch::read_variant_object_branch_10_1;
use mxd2qgz::storage::object_refs::{RefTable10_1, read_object_entry_10_1};
use mxd2qgz::storage::object_stream::{ObjectHeader, read_header_10_1, read_header_10_8};
use mxd2qgz::stream::native_string::{read_string_10_1, read_string_10_8};
use mxd2qgz::stream::reader::Reader;
use mxd2qgz::stream::variant_stream::{VariantValue10_8, read_value_10_8};
use mxd2qgz::version::version_stream::{
    read_arcmap_header_10_1, read_layout_10_1, read_layout_10_8,
};
use std::io::{self, ErrorKind};

/// 构造 AfCore String::Write 布局的合法输入：u32 字节长度前缀 + 含终止零的 UTF-16LE。
fn encoded(text: &str) -> Vec<u8> {
    let payload: Vec<_> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes
}

/// 构造 VariantStream VT_BSTR 型载荷：u32 码元数 + UTF-16LE + 两个终止零字节。
fn bstr_payload_bytes(value: &str) -> Vec<u8> {
    let units: Vec<u16> = value.encode_utf16().collect();
    let mut bytes = Vec::with_capacity(4 + units.len() * 2 + 2);
    bytes.extend_from_slice(&(units.len() as u32).to_le_bytes());
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

/// 构造 VariantStream 值：u16 VARTYPE + 载荷。
fn value_bytes(value_type: u16, payload: &[u8]) -> Vec<u8> {
    value_type
        .to_le_bytes()
        .into_iter()
        .chain(payload.iter().copied())
        .collect()
}

/// BaseLegendItem 字段序（基类版本 3、全部槽空标记）——本轮 LegendItem 族
/// 三个读取器与统一字段序入口共用。
fn base_legend_item_v3_bytes() -> Vec<u8> {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend([0_u8; 16]); // ILayer 槽
    for _ in 0..4 {
        bytes.extend(0_u16.to_le_bytes());
    }
    bytes.extend([0_u8; 16]); // ITextSymbol 槽
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // 第二 ITextSymbol 槽
    bytes.extend([0_u8; 16]); // ILegendClassFormat 槽
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    // v2 ≥2 组。
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(encoded(" ("));
    bytes.extend(encoded(")"));
    bytes.extend(0xFFFF_u16.to_le_bytes());
    // v2 >2 的 INumberFormat 槽。
    bytes.extend([0_u8; 16]);
    bytes
}

/// 最小 RgbColor 体（版本 1，flag==0）。
fn rgb_color_body_bytes() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend([0x01, 0x11, 0x22, 0x33, 0x00]);
    bytes.extend([0xff, 0x00]);
    bytes
}

/// 最小 RandomColorRamp 体（版本 1）。
fn random_color_ramp_body_bytes() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(encoded("梯度"));
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend([0x01, 0x00]);
    bytes.extend(42_i32.to_le_bytes());
    for value in [10_u16, 200, 20, 210, 30, 220] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// 最小 Point 体（版本 2：声明 20、类型 1 + X/Y + 空标记）。
fn point_body_bytes() -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(20_u32.to_le_bytes());
    bytes.extend(1_u32.to_le_bytes());
    bytes.extend(1.5_f64.to_le_bytes());
    bytes.extend(2.5_f64.to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes
}

/// 参与统一截断扫描的读取器；每个变体对应一个公共读取函数入口。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReaderKind {
    NativeString10_1,
    NativeString10_8,
    Layout10_1,
    ArcMapHeader10_1,
    Layout10_8,
    MapsRoot10_8,
    MapsRoot10_1,
    MapPrefix10_1,
    MapSuffix10_1,
    LayerList10_1,
    FeatureLayerPrefix10_1,
    FdoGraphicsLayerPrefix10_1,
    GroupLayerPrefix10_1,
    CompositeGraphicsPrefix10_1,
    Envelope10_1,
    ObjectHeader10_1,
    ObjectHeader10_8,
    ObjectEntry10_1,
    VariantString10_8,
    VariantFixed10_8,
    SimpleRendererBody10_1,
    ElementCollectionBody10_1,
    FeatureDatasetNameBody10_1,
    TopologyNameBody10_1,
    FdoLegacyPrefix10_1,
    AoiBookmark10_1,
    FeatureBookmark10_1,
    LegendPrefix10_1,
    CoreColorBody10_1,
    CmykColorBody10_1,
    ColorSlot10_1,
    SimpleLineSymbolBody10_1,
    SimpleFillSymbolBody10_1,
    MarkerLineSymbolBody10_1,
    CartographicLineSymbolBody10_1,
    HashLineSymbolBody10_1,
    DisplaySymbolSlot10_1,
    FeatureLayerSuccessor10_1,
    LegendClassFormatBody10_1,
    HorizontalLegendItemBody10_1,
    EditTemplateBody10_1,
    PropertySetBody10_1,
    TextElementBody10_1,
    MarkerElementBody10_1,
    LineElementBody10_1,
    RgbColorBody10_1,
    SimpleMarkerSymbolBody10_1,
    TextSymbolBody10_1,
    // —— 接线轮扩充（A：标注环/要素 ID 集；B：符号族第二轮 14 直读入口 +
    //    全闭合槽分派；C：几何体与类分派；D：数值格式三变体 + NumericFormat
    //    迁移路径 + Hyperlink 体）——
    AnnotatePropsBody10_1,
    LabelEnginePropsBody10_1,
    EditTemplateManagerBody10_1,
    FeatureIdSetBody10_1,
    TemplateBody10_1,
    LineDecorationBody10_1,
    SimpleLineDecorationElementBody10_1,
    MultiLayerLineSymbolBody10_1,
    MultiLayerMarkerSymbolBody10_1,
    MultiLayerFillSymbolBody10_1,
    CharacterMarkerSymbolBody10_1,
    PictureMarkerSymbolBody10_1,
    PictureLineSymbolBody10_1,
    ArrowMarkerSymbolBody10_1,
    LineFillSymbolBody10_1,
    MarkerFillSymbolBody10_1,
    GradientFillSymbolBody10_1,
    DotDensityFillSymbolBody10_1,
    SymbolSlotFull10_1,
    PolylineBody10_1,
    GeometryBodyOfClass10_1,
    NumericFormatBody10_1,
    PercentageFormatBody10_1,
    FractionFormatBody10_1,
    RateFormatBody10_1,
    HyperlinkBody10_1,
    // —— 整合接线轮扩充（E：StdFont/StdPicture 体；F：BOLP 族/引擎/UID/
    //    HLEP/DEP 前缀；G：Polygon/Point/Multipoint 体 + 类型化分派；
    //    H：数值格式同族五类）——
    StdFontBody10_1,
    StdPictureBody10_1,
    BolpBody10_1,
    LineLabelPositionBody10_1,
    LineLabelPlacementPrioritiesBody10_1,
    PointPlacementPrioritiesBody10_1,
    VbscriptEngineBody10_1,
    UidBody10_1,
    HotLinkExpressionPropsBody10_1,
    DisplayExpressionPropsPrefix10_1,
    PolygonBody10_1,
    PointBody10_1,
    MultipointBody10_1,
    GeometryBody10_1,
    CurrencyFormatBody10_1,
    LatLonFormatBody10_1,
    AngleFormatBody10_1,
    ScientificFormatBody10_1,
    CustomNumberFormatBody10_1,
    // —— 本轮整合接线（A：dep_tail TimeData 族；B：IDatasetName 槽族六类 +
    //    分派；C：patch/LegendItem 族/MapSurround 族；D：Variant 对象分支；
    //    F：GCS/Unknown 坐标系；G：ColorRamp/Callout/PictureFill/元素装饰）——
    TimeBody10_1,
    TimeZoneInfoBody10_1,
    TimeReferenceBody10_1,
    TimeExtentBody10_1,
    TimeDataFamily10_1,
    GeometricNetworkNameBody10_1,
    RelationshipClassNameBody10_1,
    MemoryRelationshipClassNameBody10_1,
    QueryTableNameBody10_1,
    RelQueryTableNameBody10_1,
    DbTableNameBody10_1,
    DatasetNameSlot10_1,
    PatchBody10_1,
    HorizontalBarLegendItemBody10_1,
    NestedLegendItemBody10_1,
    VerticalLegendItemBody10_1,
    BaseLegendItemFields10_1,
    MapTitleBody10_1,
    ScalebarBody10_1,
    ScaleTextBody10_1,
    MarkerNorthArrowBody10_1,
    LegendFormatBody10_1,
    ScalebarSubclassBody10_1,
    VariantObjectBranch10_1,
    GeographicCoordinateSystem10_1,
    UnknownCoordinateSystem10_1,
    RandomColorRampBody10_1,
    MultiPartColorRampBody10_1,
    PresetColorRampBody10_1,
    AlgorithmicColorRampBody10_1,
    ColorRampSlot10_1,
    BalloonCalloutBody10_1,
    LineCalloutBody10_1,
    SimpleLineCalloutBody10_1,
    MarkerTextBackgroundBody10_1,
    MarkerBackgroundSlot10_1,
    PictureFillSymbolBody10_1,
    PictureFillSymbolSlot10_1,
    SymbolBorderBody10_1,
    SymbolBackgroundBody10_1,
    SymbolShadowBody10_1,
    BorderSlot10_1,
    BackgroundSlot10_1,
    ShadowSlot10_1,
}

fn all_kinds() -> [ReaderKind; 137] {
    [
        ReaderKind::NativeString10_1,
        ReaderKind::NativeString10_8,
        ReaderKind::Layout10_1,
        ReaderKind::ArcMapHeader10_1,
        ReaderKind::Layout10_8,
        ReaderKind::MapsRoot10_8,
        ReaderKind::MapsRoot10_1,
        ReaderKind::MapPrefix10_1,
        ReaderKind::MapSuffix10_1,
        ReaderKind::LayerList10_1,
        ReaderKind::FeatureLayerPrefix10_1,
        ReaderKind::FdoGraphicsLayerPrefix10_1,
        ReaderKind::GroupLayerPrefix10_1,
        ReaderKind::CompositeGraphicsPrefix10_1,
        ReaderKind::Envelope10_1,
        ReaderKind::ObjectHeader10_1,
        ReaderKind::ObjectHeader10_8,
        ReaderKind::ObjectEntry10_1,
        ReaderKind::VariantString10_8,
        ReaderKind::VariantFixed10_8,
        ReaderKind::SimpleRendererBody10_1,
        ReaderKind::ElementCollectionBody10_1,
        ReaderKind::FeatureDatasetNameBody10_1,
        ReaderKind::TopologyNameBody10_1,
        ReaderKind::FdoLegacyPrefix10_1,
        ReaderKind::AoiBookmark10_1,
        ReaderKind::FeatureBookmark10_1,
        ReaderKind::LegendPrefix10_1,
        ReaderKind::CoreColorBody10_1,
        ReaderKind::CmykColorBody10_1,
        ReaderKind::ColorSlot10_1,
        ReaderKind::SimpleLineSymbolBody10_1,
        ReaderKind::SimpleFillSymbolBody10_1,
        ReaderKind::MarkerLineSymbolBody10_1,
        ReaderKind::CartographicLineSymbolBody10_1,
        ReaderKind::HashLineSymbolBody10_1,
        ReaderKind::DisplaySymbolSlot10_1,
        ReaderKind::FeatureLayerSuccessor10_1,
        ReaderKind::LegendClassFormatBody10_1,
        ReaderKind::HorizontalLegendItemBody10_1,
        ReaderKind::EditTemplateBody10_1,
        ReaderKind::PropertySetBody10_1,
        ReaderKind::TextElementBody10_1,
        ReaderKind::MarkerElementBody10_1,
        ReaderKind::LineElementBody10_1,
        ReaderKind::RgbColorBody10_1,
        ReaderKind::SimpleMarkerSymbolBody10_1,
        ReaderKind::TextSymbolBody10_1,
        // A 轮：标注环三体 + FeatureIDSet 体。
        ReaderKind::AnnotatePropsBody10_1,
        ReaderKind::LabelEnginePropsBody10_1,
        ReaderKind::EditTemplateManagerBody10_1,
        ReaderKind::FeatureIdSetBody10_1,
        // B 轮：符号族第二轮 14 直读入口 + 全闭合槽分派。
        ReaderKind::TemplateBody10_1,
        ReaderKind::LineDecorationBody10_1,
        ReaderKind::SimpleLineDecorationElementBody10_1,
        ReaderKind::MultiLayerLineSymbolBody10_1,
        ReaderKind::MultiLayerMarkerSymbolBody10_1,
        ReaderKind::MultiLayerFillSymbolBody10_1,
        ReaderKind::CharacterMarkerSymbolBody10_1,
        ReaderKind::PictureMarkerSymbolBody10_1,
        ReaderKind::PictureLineSymbolBody10_1,
        ReaderKind::ArrowMarkerSymbolBody10_1,
        ReaderKind::LineFillSymbolBody10_1,
        ReaderKind::MarkerFillSymbolBody10_1,
        ReaderKind::GradientFillSymbolBody10_1,
        ReaderKind::DotDensityFillSymbolBody10_1,
        ReaderKind::SymbolSlotFull10_1,
        // C 轮：Polyline 体 + 几何类分派。
        ReaderKind::PolylineBody10_1,
        ReaderKind::GeometryBodyOfClass10_1,
        // D 轮：数值格式三变体 + NumericFormat（objects 层迁移路径）+ Hyperlink 体。
        ReaderKind::NumericFormatBody10_1,
        ReaderKind::PercentageFormatBody10_1,
        ReaderKind::FractionFormatBody10_1,
        ReaderKind::RateFormatBody10_1,
        ReaderKind::HyperlinkBody10_1,
        // 整合接线轮：E（StdFont/StdPicture 体）+ F（BOLP 族/引擎/UID/HLEP/
        // DEP 前缀）+ G（Polygon/Point/Multipoint 体 + 类型化分派）+ H（数值
        // 格式同族五类）。
        ReaderKind::StdFontBody10_1,
        ReaderKind::StdPictureBody10_1,
        ReaderKind::BolpBody10_1,
        ReaderKind::LineLabelPositionBody10_1,
        ReaderKind::LineLabelPlacementPrioritiesBody10_1,
        ReaderKind::PointPlacementPrioritiesBody10_1,
        ReaderKind::VbscriptEngineBody10_1,
        ReaderKind::UidBody10_1,
        ReaderKind::HotLinkExpressionPropsBody10_1,
        ReaderKind::DisplayExpressionPropsPrefix10_1,
        ReaderKind::PolygonBody10_1,
        ReaderKind::PointBody10_1,
        ReaderKind::MultipointBody10_1,
        ReaderKind::GeometryBody10_1,
        ReaderKind::CurrencyFormatBody10_1,
        ReaderKind::LatLonFormatBody10_1,
        ReaderKind::AngleFormatBody10_1,
        ReaderKind::ScientificFormatBody10_1,
        ReaderKind::CustomNumberFormatBody10_1,
        // —— 本轮整合接线：A（dep_tail）+ B（IDatasetName 槽族）——
        ReaderKind::TimeBody10_1,
        ReaderKind::TimeZoneInfoBody10_1,
        ReaderKind::TimeReferenceBody10_1,
        ReaderKind::TimeExtentBody10_1,
        ReaderKind::TimeDataFamily10_1,
        ReaderKind::GeometricNetworkNameBody10_1,
        ReaderKind::RelationshipClassNameBody10_1,
        ReaderKind::MemoryRelationshipClassNameBody10_1,
        ReaderKind::QueryTableNameBody10_1,
        ReaderKind::RelQueryTableNameBody10_1,
        ReaderKind::DbTableNameBody10_1,
        ReaderKind::DatasetNameSlot10_1,
        // —— C（patch + LegendItem 族 + MapSurround 族）——
        ReaderKind::PatchBody10_1,
        ReaderKind::HorizontalBarLegendItemBody10_1,
        ReaderKind::NestedLegendItemBody10_1,
        ReaderKind::VerticalLegendItemBody10_1,
        ReaderKind::BaseLegendItemFields10_1,
        ReaderKind::MapTitleBody10_1,
        ReaderKind::ScalebarBody10_1,
        ReaderKind::ScaleTextBody10_1,
        ReaderKind::MarkerNorthArrowBody10_1,
        ReaderKind::LegendFormatBody10_1,
        ReaderKind::ScalebarSubclassBody10_1,
        // —— D（Variant 对象分支）+ F（GCS/Unknown 坐标系）——
        ReaderKind::VariantObjectBranch10_1,
        ReaderKind::GeographicCoordinateSystem10_1,
        ReaderKind::UnknownCoordinateSystem10_1,
        // —— G（ColorRamp/Callout/PictureFill/元素装饰）——
        ReaderKind::RandomColorRampBody10_1,
        ReaderKind::MultiPartColorRampBody10_1,
        ReaderKind::PresetColorRampBody10_1,
        ReaderKind::AlgorithmicColorRampBody10_1,
        ReaderKind::ColorRampSlot10_1,
        ReaderKind::BalloonCalloutBody10_1,
        ReaderKind::LineCalloutBody10_1,
        ReaderKind::SimpleLineCalloutBody10_1,
        ReaderKind::MarkerTextBackgroundBody10_1,
        ReaderKind::MarkerBackgroundSlot10_1,
        ReaderKind::PictureFillSymbolBody10_1,
        ReaderKind::PictureFillSymbolSlot10_1,
        ReaderKind::SymbolBorderBody10_1,
        ReaderKind::SymbolBackgroundBody10_1,
        ReaderKind::SymbolShadowBody10_1,
        ReaderKind::BorderSlot10_1,
        ReaderKind::BackgroundSlot10_1,
        ReaderKind::ShadowSlot10_1,
    ]
}

/// 每个读取器一个确定合法的完整输入；完整输入必须被接受并完整消费。
/// LELP 体（版本 7）的最小完整形态：全部标量段 + 四槽与容器槽空标记 +
/// 版本门尾段，完整消费（零停止）。字节序与 `tests/feature_layer_tail.rs`
/// 的 `lelp_body_v7(None)` 同构（标量取零值）。
fn lelp_body_v7_sample() -> Vec<u8> {
    let mut bytes = 7_u16.to_le_bytes().to_vec();
    bytes.extend(encoded(""));
    bytes.extend(0_i16.to_le_bytes()); // ③ short -0x74
    // ④ 版本 ≤1 的 short：版本 7 不写。
    bytes.extend(0_f64.to_le_bytes()); // ⑤ double -0x54
    bytes.extend(0_i32.to_le_bytes()); // ⑥ int32 -0x60
    bytes.extend(0_i16.to_le_bytes()); // ⑦ short -0x5c
    bytes.extend(0_i16.to_le_bytes()); // ⑧ short -0x4c
    bytes.extend(0_i16.to_le_bytes()); // ⑨ short -0x4a
    bytes.extend(0_i32.to_le_bytes()); // ⑩ int32 -0x80
    bytes.extend(0_f64.to_le_bytes()); // ⑪ double -0x34
    bytes.extend(0_f64.to_le_bytes()); // ⑫ double -0x3c
    bytes.extend(0_i16.to_le_bytes()); // ⑬ short -0x48
    bytes.extend(encoded("")); // ⑭ 串 2
    bytes.extend(encoded("")); // ⑮ 串 3
    bytes.extend([0_u8; 16]); // ⑯ ITextSymbol 槽（空标记）
    bytes.extend([0_u8; 16]); // ⑰ Overposter 槽
    bytes.extend([0_u8; 16]); // ⑱ 表达式引擎槽
    bytes.extend([0_u8; 16]); // ⑲ Envelope 槽
    bytes.extend(0_f64.to_le_bytes()); // ⑳ double -0x24
    bytes.extend(0_f64.to_le_bytes()); // ㉑ double -0x1c
    bytes.extend(0_i32.to_le_bytes()); // ㉒ int32 -0x14
    bytes.extend(0_i32.to_le_bytes()); // ㉓ int32 -0x2c
    bytes.extend([0_u8; 16]); // ㉔ 容器槽（版本 >2）
    bytes.extend(0_i16.to_le_bytes()); // ㉕ short -0x46（版本 ≥4）
    bytes.extend(0_i32.to_le_bytes()); // ㉖ int32 -0x7c（版本 ≥5）
    bytes
}

fn sample(kind: ReaderKind) -> Vec<u8> {
    match kind {
        ReaderKind::NativeString10_1 | ReaderKind::NativeString10_8 => encoded("地图😀"),
        ReaderKind::Layout10_1 | ReaderKind::Layout10_8 => {
            let mut bytes = encoded("10.1");
            bytes.extend(0x13_i16.to_le_bytes());
            bytes
        }
        ReaderKind::ArcMapHeader10_1 => {
            let mut bytes = encoded("10.1");
            bytes.extend(0x13_i16.to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MapsRoot10_8 => {
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend([0xa5_u8; 16]);
            bytes.extend(7_u32.to_le_bytes());
            bytes
        }
        ReaderKind::MapsRoot10_1 => {
            // 10.1 读侧在对象循环结束后还须读取 4 字节尾部值，合法样本必须
            // 包含它才能被完整消费。
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend([0xa5_u8; 16]);
            bytes.extend(7_u32.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes
        }
        ReaderKind::MapPrefix10_1 => {
            // 版本 51 + 两个名称 + 嵌套 PCS 头 + 保存版本 0 的最小 PCS 体
            // （仅版本 + WKT 分支），全部被读取器完整消费。
            let mut bytes = 51_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("新建数据框"));
            bytes.extend(encoded(""));
            bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
            bytes.extend(0x1e_u32.to_le_bytes());
            let wkt = b"PROJCS[\"x\"]";
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend((wkt.len() as u32).to_le_bytes());
            bytes.extend(wkt);
            bytes
        }
        ReaderKind::MapSuffix10_1 => {
            // 版本 51 + 双名称 + 空间参考空标记 + 图层列表（计数 0 + 活动索引
            // -1）+ 空标记基本图形层 + MapSurround 计数 0 + WKSEnvelope + 两个
            // int32 + 空标记 IEnvelope + 书签计数 0：后缀在 AfterBookmarks
            // 停止且无剩余字节。
            let mut bytes = 51_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("框"));
            bytes.extend(encoded(""));
            bytes.extend([0_u8; 16]); // 空间参考：空标记
            bytes.extend(0_i32.to_le_bytes()); // 图层列表计数 0
            bytes.extend((-1_i32).to_le_bytes()); // 活动图层索引
            bytes.extend([0_u8; 16]); // 基本图形层：空标记
            bytes.extend(0_i32.to_le_bytes()); // MapSurround 计数
            for value in [1.0_f64, 2.0, 3.0, 4.0] {
                bytes.extend(value.to_le_bytes()); // WKSEnvelope
            }
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend([0_u8; 16]); // IEnvelope：空标记
            bytes.extend(0_i32.to_le_bytes()); // 书签计数
            bytes
        }
        ReaderKind::LayerList10_1 => {
            // 计数 1 + 标志 2 前缀 [A=20][B=0] + 新对象头（未知类，A 推定跳过
            // 0 字节）+ 活动索引：无停止点，完整消费。
            let mut bytes = 1_i32.to_le_bytes().to_vec();
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(20_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend([0xa5_u8; 16]);
            bytes.extend(7_u32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes()); // 活动图层索引
            bytes
        }
        ReaderKind::FeatureLayerPrefix10_1 => {
            // 版本 34 + 双名称 + 可见性 0xFFFF + 两个未知 short + 空标记数据源
            // + 空标记渲染器 + +0x144 原样 GUID + 最小后继段（IHyperlink 计数
            // 0 + IColor 空标记 + 空串 ×2 + int32 + FieldInfo 计数 0）：
            // Successor(FieldInfoLoopCompleted) 停止且无剩余字节。
            let mut bytes = 34_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("层"));
            bytes.extend(encoded(""));
            bytes.extend(0xFFFF_u16.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // 数据源：空标记
            bytes.extend([0_u8; 16]); // 渲染器：空标记
            bytes.extend([0x5A; 16]); // +0x144 GUID
            bytes.extend(encoded("")); // +0x154 空串（语义未定）
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
            bytes.extend([0_u8; 16]); // IColor：空标记
            bytes.extend(encoded(""));
            bytes.extend(encoded(""));
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
            bytes
        }
        ReaderKind::FdoGraphicsLayerPrefix10_1 => {
            // 版本 11 + int32 + short + 关联 FeatureLayer（头 + 前缀体，其数据
            // 源与渲染器均为空标记、含 GUID）：AssociatedLayerBodyStopped 停止
            // 且无剩余字节。
            let mut layer = 34_u16.to_le_bytes().to_vec();
            layer.extend(encoded("注记"));
            layer.extend(encoded(""));
            layer.extend(0xFFFF_u16.to_le_bytes());
            layer.extend(0_u16.to_le_bytes());
            layer.extend(0_u16.to_le_bytes());
            layer.extend([0_u8; 16]); // 数据源：空标记
            layer.extend([0_u8; 16]); // 渲染器：空标记
            layer.extend([0x5A; 16]); // GUID
            layer.extend(encoded("")); // +0x154 空串（语义未定）
            layer.extend(0_f64.to_le_bytes());
            layer.extend(0_f64.to_le_bytes());
            layer.extend(0_i16.to_le_bytes());
            layer.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
            layer.extend([0_u8; 16]); // IColor：空标记
            layer.extend(encoded(""));
            layer.extend(encoded(""));
            layer.extend(0_i32.to_le_bytes());
            layer.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
            let mut bytes = 11_u16.to_le_bytes().to_vec();
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(0xFFFF_u16.to_le_bytes());
            bytes.extend(mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID);
            bytes.extend(32_u32.to_le_bytes());
            bytes.extend(&layer);
            bytes
        }
        ReaderKind::GroupLayerPrefix10_1 => {
            // 版本 4：名称 + short ×3 + double ×2 + 子图层计数 0 + 可见性
            // 0xFFFF + 空标记嵌套对象（版本 >2）+ >=4 的 double → 版本 <5 无
            // ExtensionHelper，尾段停止点恰在样本末尾。
            let mut bytes = 4_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("组"));
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes()); // 子图层计数 0
            bytes.extend(0xFFFF_u16.to_le_bytes()); // 可见性 VARIANT_TRUE
            bytes.extend([0_u8; 16]); // 嵌套对象：空标记
            bytes.extend(9.0_f64.to_le_bytes()); // >=4 的 double
            bytes
        }
        ReaderKind::CompositeGraphicsPrefix10_1 => {
            // 版本 6：元素组计数 0 + 名称 + short ×3 + double ×2 + int32 ×3 +
            // 空标记嵌套对象 + 版本 >1 的 double+int32 + 子图层计数 0 → 尾段
            // 停止点恰在样本末尾。
            let mut bytes = 6_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes()); // 元素组计数 0
            bytes.extend(encoded("G"));
            bytes.extend(0xffff_u16.to_le_bytes());
            bytes.extend(0xffff_u16.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend([0_u8; 16]); // 嵌套对象：空标记
            bytes.extend(0_f64.to_le_bytes()); // 版本 >1：double
            bytes.extend(0_i32.to_le_bytes()); // 版本 >1：int32
            bytes.extend(0_i32.to_le_bytes()); // 子图层计数 0
            bytes
        }
        ReaderKind::Envelope10_1 => {
            // 版本 3：4 double + int32 0（无 Z/M 追加段）+ 空标记空间参考。
            let mut bytes = 3_u16.to_le_bytes().to_vec();
            for value in [1.0_f64, 2.0, 3.0, 4.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend([0_u8; 16]); // 空间参考：空标记
            bytes
        }
        ReaderKind::ObjectHeader10_1 | ReaderKind::ObjectHeader10_8 => {
            let mut bytes = vec![0xa5_u8; 16];
            bytes.extend(7_u32.to_le_bytes());
            bytes
        }
        ReaderKind::ObjectEntry10_1 => {
            let mut bytes = vec![0xa5_u8; 16];
            bytes.extend(7_u32.to_le_bytes());
            bytes
        }
        ReaderKind::SimpleRendererBody10_1 => {
            // 版本 1 最小体：三槽位空标记 + short + 双空串 + int32（版本 ≤2
            // 无 RotationRenderer/SizeRenderer 尾段）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]); // ISymbol 槽：空标记
            bytes.extend([0_u8; 16]); // ILegendGroup 槽：空标记
            bytes.extend([0_u8; 16]); // ILegendItem 槽：空标记
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(encoded(""));
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(encoded(""));
            bytes
        }
        ReaderKind::ElementCollectionBody10_1 => {
            // 版本 4 + 计数 0（样本 FDO 槽位键 86 的 6 字节形态）。
            let mut bytes = 4_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::FeatureDatasetNameBody10_1 => {
            // 版本 1 + 四空串 + 标志 0 + IWorkspaceName 槽的 WorkspaceName 新
            // 对象（头 + 版本 1 体：三空串 + 空 IPropertySet + 标志 0 + int32）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            for _ in 0..4 {
                bytes.extend(encoded(""));
            }
            bytes.push(0); // 标志 0：无工厂 CLSID 段
            bytes.extend(mxd2qgz::layers::feature_layer::WORKSPACE_NAME_CLASS_ID);
            bytes.extend(60_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes()); // WS 版本
            for _ in 0..3 {
                bytes.extend(encoded(""));
            }
            bytes.extend([0_u8; 16]); // IPropertySet：空标记
            bytes.push(0); // 工厂标志 0
            bytes.extend(0_i32.to_le_bytes()); // 尾随 int32
            bytes
        }
        ReaderKind::TopologyNameBody10_1 => {
            // 版本 1 + 两空串 + IDatasetName 槽空标记。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded(""));
            bytes.extend(encoded(""));
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::FdoLegacyPrefix10_1 => {
            // 版本 0：旧加载器字段 1-11（版本 ≥2/≥3 的字段无门），四个接口槽
            // 均空标记，收尾即旧体完整终结。
            let mut bytes = 0_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("")); // String
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            for _ in 0..4 {
                bytes.extend([0_u8; 16]); // ILayer/ISR/IEC2/IName 槽：空标记
            }
            bytes
        }
        ReaderKind::AoiBookmark10_1 => {
            // 版本 1 + 空 IEnvelope + 空名称。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded(""));
            bytes
        }
        ReaderKind::FeatureBookmark10_1 => {
            // 版本 1 + int32 + 空 IName + 名称。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded(""));
            bytes
        }
        ReaderKind::LegendPrefix10_1 => {
            // Legend 版本 1 + 共同前缀版本 1（版本 short + 空名称 + 关联地图
            // 空标记）：ILegendFormat 空标记 + 空标题 + 版本 1 旧遗留 double
            // ×2 + short ×4 + ILegendItem 计数 0；版本 <3/4/5 的尾段无门。
            let mut bytes = 1_u16.to_le_bytes().to_vec(); // Legend 版本
            bytes.extend(1_u16.to_le_bytes()); // 共同前缀版本
            bytes.extend(encoded("")); // 名称
            bytes.extend([0_u8; 16]); // 关联地图：空标记
            bytes.extend([0_u8; 16]); // ILegendFormat：空标记
            bytes.extend(encoded("")); // 标题
            bytes.extend(0_f64.to_le_bytes()); // 版本 1 旧遗留段
            bytes.extend(0_f64.to_le_bytes());
            for _ in 0..4 {
                bytes.extend(0_u16.to_le_bytes()); // short ×3 + 集合接口 short
            }
            bytes.extend(0_i32.to_le_bytes()); // ILegendItem 计数 0
            bytes
        }
        ReaderKind::CoreColorBody10_1 => {
            // 共享 Color 核心（版本 1/1，flag==0）：short ×2 + byte +
            // double ×3 + byte ×2。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(1_u16.to_le_bytes());
            bytes.push(0);
            for value in [1.0_f64, 2.0, 3.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(0);
            bytes.push(0xFF);
            bytes
        }
        ReaderKind::CmykColorBody10_1 => {
            // CmykColor 版本 4（无串分支）：short + byte ×2 + int32 + byte ×2。
            let mut bytes = 4_u16.to_le_bytes().to_vec();
            bytes.push(0);
            bytes.push(0);
            bytes.extend(0x1234_5678_u32.to_le_bytes());
            bytes.push(0x21);
            bytes.push(0x43);
            bytes
        }
        ReaderKind::ColorSlot10_1 => {
            // 颜色槽位：RgbColor 新对象（头 + 版本 1 体，flag==0）。
            let mut bytes = RGB_COLOR_CLASS_ID.to_vec();
            bytes.extend(37_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.push(0);
            for value in [1.0_f64, 2.0, 3.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(0);
            bytes.push(0xFF);
            bytes
        }
        ReaderKind::SimpleLineSymbolBody10_1 => {
            // SimpleLineSymbol 版本 1：空颜色槽 + double + int32 ×3。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]); // IColor：空标记
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::SimpleFillSymbolBody10_1 => {
            // SimpleFillSymbol 版本 1：轮廓与填充色槽空标记 + int32 ×3。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]); // ISymbol：空标记
            bytes.extend([0_u8; 16]); // IColor：空标记
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MarkerLineSymbolBody10_1 => {
            // MarkerLineSymbol 版本 2：byte + double + 三槽空标记 + int32 ×2
            // + byte + double + 尾段 int32 ×2 + double。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.push(0);
            bytes.extend(1.0_f64.to_le_bytes());
            for _ in 0..3 {
                bytes.extend([0_u8; 16]); // IMarkerSymbol/ITemplate/ILineDecoration：空标记
            }
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.push(0);
            bytes.extend(0.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::CartographicLineSymbolBody10_1 => {
            // CartographicLineSymbol 版本 1：int32 ×2 + double + byte +
            // double + 颜色槽与两候选槽空标记 + int32 ×2 + byte + double ×2。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.push(0);
            bytes.extend(2.0_f64.to_le_bytes());
            for _ in 0..3 {
                bytes.extend([0_u8; 16]); // IColor/ITemplate/ILineDecoration：空标记
            }
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.push(0);
            bytes.extend(0.0_f64.to_le_bytes());
            bytes.extend(0.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::HashLineSymbolBody10_1 => {
            // HashLineSymbol 版本 1：double + int32 ×2 + double + byte +
            // double + 四槽空标记 + int32 ×2 + byte + double ×2。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.push(0);
            bytes.extend(3.0_f64.to_le_bytes());
            for _ in 0..4 {
                bytes.extend([0_u8; 16]); // ISymbol/IColor/ITemplate/ILineDecoration：空标记
            }
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.push(0);
            bytes.extend(0.0_f64.to_le_bytes());
            bytes.extend(0.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::DisplaySymbolSlot10_1 => {
            // 符号槽位：SimpleMarkerSymbol 新对象（头 + 版本 1 最小体：双
            // 颜色槽空标记 + 标量段）。
            let mut bytes = SIMPLE_MARKER_SYMBOL_CLASS_ID.to_vec();
            bytes.extend(36_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // 主色：空标记
            bytes.extend(6.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(13_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(1);
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]); // 轮廓色：空标记
            bytes
        }
        ReaderKind::LegendClassFormatBody10_1 => {
            // 版本 2 + 四个对象槽全空标记 + double ×2（版本 >1 段）。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            for _ in 0..4 {
                bytes.extend([0_u8; 16]);
            }
            bytes.extend(1.5_f64.to_le_bytes());
            bytes.extend((-2.5_f64).to_le_bytes());
            bytes
        }
        ReaderKind::HorizontalLegendItemBody10_1 => {
            // 条目版本 1 + int32 + 基类版本 3 + ILayer 空标记 + short ×4 +
            // ITextSymbol 空标记 + int32 + short + ITextSymbol 空标记 +
            // ILegendClassFormat 空标记 + short ×2 + ≥2 组（short、short、
            // String、String、short）+ INumberFormat 空标记（对象槽全空标记，
            // 与集中清单的空登记表相容）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0x0102_0304_u32.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // ILayer
            for value in [1_u16, 2, 3, 4] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // ITextSymbol
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend(8_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // ITextSymbol
            bytes.extend([0_u8; 16]); // ILegendClassFormat
            bytes.extend(9_u16.to_le_bytes());
            bytes.extend(10_u16.to_le_bytes());
            bytes.extend(11_u16.to_le_bytes());
            bytes.extend(12_u16.to_le_bytes());
            bytes.extend(encoded(" ("));
            bytes.extend(encoded(")"));
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // INumberFormat
            bytes
        }
        ReaderKind::EditTemplateBody10_1 => {
            // 版本 2 + 双名称 + 16 字节 GUID + 计数 1 的 [String 键][Variant
            // BSTR 值] + 版本 ≥2 的第三 String。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("模板"));
            bytes.extend(encoded(""));
            bytes.extend([0x5A; 16]);
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(encoded("字段"));
            bytes.extend(8_u16.to_le_bytes());
            bytes.extend(encoded("默认"));
            bytes.extend(encoded("点"));
            bytes
        }
        ReaderKind::VariantString10_8 => value_bytes(8, &bstr_payload_bytes("地图")),
        ReaderKind::VariantFixed10_8 => value_bytes(5, &[0; 8]),
        ReaderKind::FeatureLayerSuccessor10_1 => {
            // 样本形态的最小后继段：标量段 + IHyperlink 计数 0 + IColor 空标
            // 记 + 空串 ×2 + int32 + FieldInfo 计数 1（条目名 + FieldInfo 新
            // 对象：版本 4 体含 NumericFormat 新对象嵌套）；循环完整消费。
            let mut bytes = encoded("BSM"); // +0x154
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
            bytes.extend([0_u8; 16]); // IColor：空标记
            bytes.extend(encoded(""));
            bytes.extend(encoded(""));
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes()); // FieldInfo 计数 1
            bytes.extend(encoded("OBJECTID")); // 条目名（版本 34 > 5）
            bytes.extend(mxd2qgz::layers::feature_layer_tail::FIELD_INFO_CLASS_ID);
            bytes.extend(41_u32.to_le_bytes());
            bytes.extend(4_u16.to_le_bytes()); // FI 版本
            bytes.extend(0xFFFF_u16.to_le_bytes()); // -0x16
            bytes.extend(0_u16.to_le_bytes()); // -0x14（版本 >1）
            bytes.extend(encoded("OBJECTID")); // 别名
            bytes.extend(mxd2qgz::layers::feature_layer_tail::NUMERIC_FORMAT_CLASS_ID);
            bytes.extend(42_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes()); // NF 版本
            for value in [0_i32, 0, 0, 12] {
                bytes.extend(value.to_le_bytes()); // NF int32 ×4
            }
            for value in [0_i16; 3] {
                bytes.extend(value.to_le_bytes()); // NF short ×3
            }
            bytes.extend((-1_i16).to_le_bytes()); // -0x18（版本 >2）
            bytes.extend((-1_i32).to_le_bytes()); // -0x1c
            bytes.extend(0_i16.to_le_bytes()); // -0x8（版本 >3）
            bytes.extend(0_i16.to_le_bytes()); // -0x6
            bytes
        }
        ReaderKind::PropertySetBody10_1 => {
            // 样本实例 1 的体形态（键 1547 @/Maps 0x147f9）：版本 1 + 计数 1
            // + 键 "DATABASE" + VT_BSTR 路径值。样本对位见研究文档。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(encoded("DATABASE"));
            bytes.extend(value_bytes(8, &encoded("D:\\g")));
            bytes
        }
        ReaderKind::TextElementBody10_1 => {
            // 样本 TextElement 体形态（键 1613 @/Maps 0x18551）：版本 8 + EP
            // 前缀（版本 6）+ 文本 + 布尔 + TextSymbol 新对象（版本 4，双
            // RgbColor）+ ……到 ISymbol #2 空标记为止（后续槽位留空标记使读取
            // 完整消费，见 element_body 测试的停止点形态）。
            let mut bytes = 8_u16.to_le_bytes().to_vec();
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(encoded(""));
            bytes.extend(encoded("文本"));
            bytes.extend(value_bytes(0, &[]));
            bytes.extend(0xffff_u16.to_le_bytes());
            bytes.extend(5393_f64.to_le_bytes());
            bytes.extend(6_i32.to_le_bytes());
            bytes.extend(encoded("T35"));
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(TEXT_SYMBOL_CLASS_ID);
            bytes.extend(1614_u32.to_le_bytes());
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend(RGB_COLOR_CLASS_ID);
            bytes.extend(1615_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.push(0);
            for value in [1.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(0);
            bytes.push(0);
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.push(0);
            bytes.push(0);
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(13_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(RGB_COLOR_CLASS_ID);
            bytes.extend(1616_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.push(0);
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(0);
            bytes.push(0);
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(100_f64.to_le_bytes());
            bytes.extend(100_f64.to_le_bytes());
            bytes.push(1);
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(90_f64.to_le_bytes());
            bytes.push(1);
            bytes.extend(TEXT_SYMBOL_CLASS_ID);
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded("T35"));
            bytes.extend(8_f64.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(1.5_f64.to_le_bytes());
            bytes.extend([0_u8; 16]); // ISymbol #2 空标记
            bytes.extend([0_u8; 16]); // IFontDisp 空标记
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(TEXT_SYMBOL_CLASS_ID);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // IGeometry 空标记
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend((-1_i32).to_le_bytes());
            bytes
        }
        ReaderKind::MarkerElementBody10_1 => {
            // 版本 3：EP 前缀 + 符号/几何空标记 + int32 + 32 字节。
            let mut bytes = 3_u16.to_le_bytes().to_vec();
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(encoded(""));
            bytes.extend(encoded("文本"));
            bytes.extend(value_bytes(0, &[]));
            bytes.extend(0xffff_u16.to_le_bytes());
            bytes.extend(5393_f64.to_le_bytes());
            bytes.extend(6_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(5_i32.to_le_bytes());
            bytes.extend([0xa5_u8; 32]);
            bytes
        }
        ReaderKind::LineElementBody10_1 => {
            // 版本 3：EP 前缀 + 符号/几何空标记 + int32 + int16。
            let mut bytes = 3_u16.to_le_bytes().to_vec();
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(encoded(""));
            bytes.extend(encoded("文本"));
            bytes.extend(value_bytes(0, &[]));
            bytes.extend(0xffff_u16.to_le_bytes());
            bytes.extend(5393_f64.to_le_bytes());
            bytes.extend(6_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(5_i32.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            bytes
        }
        ReaderKind::RgbColorBody10_1 => {
            // RgbColor 版本 1/1（flag==0，样本三实例同构）：short ×2 + byte +
            // double ×3 + byte ×2。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(1_u16.to_le_bytes());
            bytes.push(0);
            for value in [1.0_f64, 2.0, 3.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(0);
            bytes.push(0xFF);
            bytes
        }
        ReaderKind::SimpleMarkerSymbolBody10_1 => {
            // SMS 版本 1 最小体（接线轮移入 display_symbol）：双 IColor 空标
            // 记 + 标量段（颜色槽经五类颜色分派，空标记不消耗深度）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]); // 主色：空标记
            bytes.extend(6.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(13_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(1);
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]); // 轮廓色：空标记
            bytes
        }
        ReaderKind::TextSymbolBody10_1 => {
            // TextSymbol 版本 4（样本 TextElement 符号槽的流内类，接线轮符号
            // 槽 #2 经对象层符号分派）：双 RgbColor 新对象 + 全部标量组 +
            // 原样 GUID + 双空标记 + 串 + 标量组 + SimpleFillSymbol 新对象
            // （#2 分派）+ IFontDisp 空标记 + 版本门尾段。
            let rgb = |key: u32| {
                let mut bytes = RGB_COLOR_CLASS_ID.to_vec();
                bytes.extend(key.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.push(0);
                for value in [0.0_f64; 3] {
                    bytes.extend(value.to_le_bytes());
                }
                bytes.push(0);
                bytes.push(0);
                bytes
            };
            let mut bytes = 4_u16.to_le_bytes().to_vec();
            bytes.extend(&rgb(1615)); // IColor #1
            bytes.extend((-1_i32).to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.push(0);
            bytes.push(0);
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(13_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(&rgb(1616)); // IColor #2
            for value in [0.0_f64; 2] {
                bytes.extend(value.to_le_bytes());
            }
            for value in [0_i32; 2] {
                bytes.extend(value.to_le_bytes());
            }
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(1);
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(90_f64.to_le_bytes());
            bytes.push(1);
            bytes.extend(TEXT_SYMBOL_CLASS_ID); // 原样 GUID
            bytes.extend([0_u8; 16]); // ITextPath
            bytes.extend([0_u8; 16]); // ISymbol #1
            bytes.extend(encoded("T35"));
            bytes.extend(8_f64.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(1.5_f64.to_le_bytes());
            // ISymbol #2：SimpleFillSymbol 新对象（体：轮廓空标记 + 填充色
            // 空标记 + int32 ×3）。
            bytes.extend(mxd2qgz::objects::display_symbol::SIMPLE_FILL_SYMBOL_CLASS_ID);
            bytes.extend(1593_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            for value in [0_i32; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // IFontDisp
            bytes.extend(1_u16.to_le_bytes()); // 版本 >1
            bytes.extend(TEXT_SYMBOL_CLASS_ID); // 版本 ≥3 GUID
            bytes.extend(1_u16.to_le_bytes()); // 版本 ≥4
            bytes
        }
        // —— A 轮：标注环三体 + FeatureIDSet 体（计数 0 最小合法形态）——
        ReaderKind::AnnotatePropsBody10_1 => {
            // 版本 2 + 计数 0 + 尾 int32。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::LabelEnginePropsBody10_1 => lelp_body_v7_sample(),
        ReaderKind::EditTemplateManagerBody10_1 => {
            // 版本 2 + 计数 0 + 版本 ≥2 尾字节。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.push(0);
            bytes
        }
        ReaderKind::FeatureIdSetBody10_1 => {
            // 版本 1 + 计数 0。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        // —— B 轮：符号族第二轮直读入口（最小合法形态，全部完整消费）——
        ReaderKind::TemplateBody10_1 => {
            // 版本 1 + double 间隔 + 计数 0。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(2.5_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::LineDecorationBody10_1 => {
            // 版本 1 + 元素计数 0。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::SimpleLineDecorationElementBody10_1 => {
            // 版本 1 + byte ×3 + short + ISymbol 空标记 + 位置计数 0。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([1_u8, 0, 1]);
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MultiLayerLineSymbolBody10_1 => {
            // 版本 1：标量 + 层计数 0 + 数组计数 0 ×2；版本 <2 无串段。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MultiLayerMarkerSymbolBody10_1 => {
            // 版本 1：标量 + 颜色/符号空标记 + 层计数 0 + 数组 0 ×2；无背景
            // 槽（v≥2）与串段（v≥3）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            for _ in 0..4 {
                bytes.extend(0_f64.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // IColor
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]); // ISymbol
            // 层计数 0：同时界定 ⑭⑮ 两个 int32 数组（无独立计数）。
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MultiLayerFillSymbolBody10_1 => {
            // 版本 1：标量 + 颜色空标记 + 层计数 0 + 数组 0 ×2；版本 <2 无串。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            // 层计数 0：同时界定 ⑦⑧ 两个 int32 数组（无独立计数）。
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::CharacterMarkerSymbolBody10_1 => {
            // 版本 1：颜色空标记 + 标量 + v<3 的 IFontDisp 对象槽（空标记）；
            // v>1 的 short 与 v≥3 描述符、v>3 第二槽均不出现。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend(0_i32.to_le_bytes());
            for _ in 0..6 {
                bytes.extend(0_f64.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // v<3 IFontDisp 槽
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::PictureMarkerSymbolBody10_1 => {
            // 版本 9 类型 2 且 N=0（无流内位图字节）+ 共享尾段。
            let mut bytes = 9_u16.to_le_bytes().to_vec();
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes()); // N=0
            for _ in 0..3 {
                bytes.extend([0_u8; 16]); // IColor ×3
            }
            for _ in 0..6 {
                bytes.extend(0_f64.to_le_bytes());
            }
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.push(1);
            bytes.extend(0_i16.to_le_bytes());
            for _ in 0..3 {
                bytes.extend(0_i16.to_le_bytes());
            }
            bytes
        }
        ReaderKind::PictureLineSymbolBody10_1 => {
            // 版本 1 + byte + double ×2 + IPictureFillSymbol 空标记。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.push(1);
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend(3.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::ArrowMarkerSymbolBody10_1 => {
            // 版本 1：颜色空标记 + 标量（v>1 无尾 short）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            for _ in 0..3 {
                bytes.extend(0_f64.to_le_bytes());
            }
            for _ in 0..3 {
                bytes.extend(0_i32.to_le_bytes());
            }
            for _ in 0..2 {
                bytes.extend(0_f64.to_le_bytes());
            }
            bytes
        }
        ReaderKind::LineFillSymbolBody10_1 => {
            // 版本 1：颜色空标记 + 双 ISymbol 槽空标记 + 标量段。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            for _ in 0..3 {
                bytes.extend(0_f64.to_le_bytes());
            }
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::MarkerFillSymbolBody10_1 => {
            // 版本 1：int32 + double ×4 + 颜色/双 ISymbol 空标记 + 标量。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            for _ in 0..4 {
                bytes.extend(0_f64.to_le_bytes());
            }
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(45.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::GradientFillSymbolBody10_1 => {
            // 版本 1：色带/颜色/符号空标记 + 标量段。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(30.0_f64.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(0.5_f64.to_le_bytes());
            for _ in 0..3 {
                bytes.extend(0_i32.to_le_bytes());
            }
            bytes
        }
        ReaderKind::DotDensityFillSymbolBody10_1 => {
            // 版本 1：颜色 + 符号空标记 + 标量 + 标记计数 0 + 值计数 0 + 尾颜色。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.push(1);
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.push(0);
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            for _ in 0..3 {
                bytes.extend(0_i32.to_le_bytes());
            }
            bytes.extend(4.0_f64.to_le_bytes());
            bytes.push(1);
            // 标记计数 0（无独立计数字段）：值为空数组，随后尾颜色空标记。
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::SymbolSlotFull10_1 => {
            // 全闭合槽位：SimpleMarkerSymbol 新对象（头 + 版本 1 最小体）。
            let mut bytes = SIMPLE_MARKER_SYMBOL_CLASS_ID.to_vec();
            bytes.extend(36_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0_u8; 16]); // 主色：空标记
            bytes.extend(6.0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(13_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            for value in [0.0_f64; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(1);
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]); // 轮廓色：空标记
            bytes
        }
        // —— C 轮：几何体 + 类分派 ——
        ReaderKind::PolylineBody10_1 => {
            // 版本 1（无空间参考段）+ 类型 0 空几何记录（4 字节）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes
        }
        ReaderKind::GeometryBodyOfClass10_1 => {
            // 类分派入口：同一 Polyline 最小体（类标识为 Polyline 常量）。
            sample(ReaderKind::PolylineBody10_1)
        }
        // —— D 轮：数值格式 + Hyperlink 体 ——
        ReaderKind::NumericFormatBody10_1 => {
            // 版本 1 + int32 ×4 + short ×3。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            for value in [0_i32; 4] {
                bytes.extend(value.to_le_bytes());
            }
            for value in [0_i16; 3] {
                bytes.extend(value.to_le_bytes());
            }
            bytes
        }
        ReaderKind::PercentageFormatBody10_1 => {
            // 版本 2 满形态：short + 内嵌 NumericFormat 核心（同键无对象头）。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(3_i16.to_le_bytes());
            bytes.extend(&sample(ReaderKind::NumericFormatBody10_1));
            bytes
        }
        ReaderKind::FractionFormatBody10_1 => {
            // 版本 1 + int32 ×2（恒 10 字节体）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend((-2_i32).to_le_bytes());
            bytes
        }
        ReaderKind::RateFormatBody10_1 => {
            // 版本 2 满形态：double + String + 内嵌核心。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(1.5_f64.to_le_bytes());
            bytes.extend(encoded("米每秒"));
            bytes.extend(&sample(ReaderKind::NumericFormatBody10_1));
            bytes
        }
        ReaderKind::HyperlinkBody10_1 => {
            // 版本 1 满体：int32 + String + int32。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(encoded("链接"));
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        // —— 整合接线轮：E 轮 StdFont/StdPicture 体 ——
        ReaderKind::StdFontBody10_1 => {
            // 样本首例形态（键 68 @0xd6c）：版本 1 + charset 134 + flags 0 +
            // weight 400 + 字号 CY.lo 120000 + 名字 4 字节（GBK「宋体」）。
            [
                0x01_u8, 0x86, 0x00, 0x00, 0x90, 0x01, 0xc0, 0xd4, 0x01, 0x00, 0x04, 0xcb, 0xce,
                0xcc, 0xe5,
            ]
            .to_vec()
        }
        ReaderKind::StdPictureBody10_1 => {
            // 头形态：魔数 0x746c + 第二字 0 + 声明 0（空图合法形态）。
            let mut bytes = 0x746c_u16.to_le_bytes().to_vec();
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes
        }
        // —— 整合接线轮：F 轮 BOLP 族 / 引擎 / UID / HLEP / DEP 前缀 ——
        ReaderKind::BolpBody10_1 => {
            // 三槽全空标记的最小完整体：20B 标量前缀 + 槽 1/2 空 + 槽间 16B
            // + 槽 3 空 + 44B 尾段（count 0 / double / int / String cb=0 /
            // int / 20B 未定名块）。
            let mut bytes = 0_i16.to_le_bytes().to_vec();
            for _ in 0..3 {
                bytes.extend(0_i16.to_le_bytes());
            }
            for _ in 0..3 {
                bytes.extend(0_i32.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // 槽 1 LLP 空标记
            bytes.extend([0_u8; 16]); // 槽 2 LLPP 空标记
            bytes.extend(0_i16.to_le_bytes()); // 槽间块
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend([0_u8; 16]); // 槽 3 PPP 空标记
            bytes.extend(0_i32.to_le_bytes()); // 尾段 count 0
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes()); // String cb=0（空 BSTR）
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes()); // 20B 未定名块
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend((-1_i16).to_le_bytes());
            bytes
        }
        ReaderKind::LineLabelPositionBody10_1 => {
            // 版本 2 满形态：short ×11 + double + 尾 short。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            for _ in 0..11 {
                bytes.extend(0_i16.to_le_bytes());
            }
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes
        }
        ReaderKind::LineLabelPlacementPrioritiesBody10_1 => {
            // 版本 1 + int32 ×15。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            for _ in 0..15 {
                bytes.extend(0_i32.to_le_bytes());
            }
            bytes
        }
        ReaderKind::PointPlacementPrioritiesBody10_1 => {
            // 版本 1 + int32 ×8。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            for _ in 0..8 {
                bytes.extend(0_i32.to_le_bytes());
            }
            bytes
        }
        ReaderKind::VbscriptEngineBody10_1 => {
            // 版本 2 满形态：short。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend((-1_i16).to_le_bytes());
            bytes
        }
        ReaderKind::UidBody10_1 => {
            // 样本形态 A：版本 1 + 标志 1 + GUID。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.push(1);
            bytes.extend([0xa5_u8; 16]);
            bytes
        }
        ReaderKind::HotLinkExpressionPropsBody10_1 => {
            // 样本形态：版本 1 + String cb=2（空串）+ short + 16B 零块。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend((-1_i16).to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::DisplayExpressionPropsPrefix10_1 => {
            // 样本形态前缀：版本 1 + String "[x]" + short + 引擎槽空标记
            //（前缀读取器读到引擎槽后即成功返回）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("[x]"));
            bytes.extend((-1_i16).to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        // —— 整合接线轮：G 轮 Polygon/Point/Multipoint 体 + 类型化分派 ——
        ReaderKind::PolygonBody10_1 => {
            // 与 Polyline 同构：版本 1（无空间参考段）+ 类型 0 空几何记录。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes
        }
        ReaderKind::PointBody10_1 => {
            // 版本 1 + 声明 4 + 类型字 0（空点恰 4 字节）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes
        }
        ReaderKind::MultipointBody10_1 => {
            // 版本 1 + 声明 4 + 类型字低字节 0（空点集恰 4 字节）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes
        }
        ReaderKind::GeometryBody10_1 => {
            // 类型化分派入口：同一 Polyline 最小体。
            sample(ReaderKind::PolylineBody10_1)
        }
        // —— 整合接线轮：H 轮数值格式同族五类 ——
        ReaderKind::CurrencyFormatBody10_1 => {
            // 恒 2 字节体（short 版本）。
            1_u16.to_le_bytes().to_vec()
        }
        ReaderKind::LatLonFormatBody10_1 => {
            // 版本 4 满形态：short ×4 + 内嵌 NumericFormat 核心 + short。
            let mut bytes = 4_u16.to_le_bytes().to_vec();
            for _ in 0..4 {
                bytes.extend(0_i16.to_le_bytes());
            }
            bytes.extend(&sample(ReaderKind::NumericFormatBody10_1));
            bytes.extend(0_i16.to_le_bytes());
            bytes
        }
        ReaderKind::AngleFormatBody10_1 => {
            // 版本 3 满形态：short + 内嵌核心 + short。
            let mut bytes = 3_u16.to_le_bytes().to_vec();
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(&sample(ReaderKind::NumericFormatBody10_1));
            bytes.extend(0_i16.to_le_bytes());
            bytes
        }
        ReaderKind::ScientificFormatBody10_1 => {
            // 版本 2 满形态：int32。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(6_i32.to_le_bytes());
            bytes
        }
        ReaderKind::CustomNumberFormatBody10_1 => {
            // 版本 1 + String 格式串。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("0.000"));
            bytes
        }
        // —— 本轮整合接线：A 轮 dep_tail TimeData 族 ——
        ReaderKind::TimeBody10_1 => {
            // 版本 0（判别 1：OA 日期 8 字节）。
            let mut bytes = 0_u16.to_le_bytes().to_vec();
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend((-1_i64).to_le_bytes());
            bytes
        }
        ReaderKind::TimeZoneInfoBody10_1 => {
            // 版本 2 满形态（无 bias 段）：3 串 + int32 + 20 字节规则块 +
            // 计数 1 + 一条规则条目。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("中国标准时间"));
            bytes.extend(encoded("中国夏令时间"));
            bytes.extend(encoded("(UTC+08:00)"));
            bytes.extend(480_i32.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(3_i16.to_le_bytes());
            bytes.extend(5_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(10_i16.to_le_bytes());
            bytes.extend(5_i16.to_le_bytes());
            bytes.extend(60_i32.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(9_i32.to_le_bytes());
            bytes.extend(4_i16.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(11_i16.to_le_bytes());
            bytes.extend(1_i16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::TimeReferenceBody10_1 => {
            // 版本 1 + ITimeZoneInfo 槽空标记 + short ×2。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend((-1_i16).to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes
        }
        ReaderKind::TimeExtentBody10_1 => {
            // 版本 2：起点/终点 Time 新对象（判别 2 WKSDateTime）+
            // ITimeReference 槽空标记 + 尾字节。
            let mut time_body = 1_u16.to_le_bytes().to_vec();
            time_body.extend(2_i32.to_le_bytes());
            time_body.extend([0x42_u8; 16]);
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(mxd2qgz::layers::dep_tail::TIME_CLASS_ID);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(&time_body);
            bytes.extend(mxd2qgz::layers::dep_tail::TIME_CLASS_ID);
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend(&time_body);
            bytes.extend([0_u8; 16]);
            bytes.push(0x01);
            bytes
        }
        ReaderKind::TimeDataFamily10_1 => {
            // 四段全空槽形态：TimeData(v2) + TimeTable(v1) + TimeDimension(v1)
            // + TimeDisplay(v1)。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            for _ in 0..4 {
                bytes.extend(encoded(""));
            }
            bytes.extend([0_u8; 16]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded(""));
            bytes.extend(encoded(""));
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i16.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(0_f64.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        // —— B 轮 IDatasetName 槽族六类 + 分派 ——
        ReaderKind::GeometricNetworkNameBody10_1 => {
            // 版本 1 + 3 串 + IDatasetName 槽空标记。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("管网"));
            bytes.extend(encoded(""));
            bytes.extend(encoded("几何网络"));
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::RelationshipClassNameBody10_1 => {
            // 版本 1 + 3 串 + 2×int32 + 2×short + 选择子 0（工作空间槽空标记）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("源类"));
            bytes.extend(encoded("目标类"));
            bytes.extend(encoded(""));
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend((-9_i32).to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::MemoryRelationshipClassNameBody10_1 => {
            // 版本 1 + 3 串 + 2×int32 + 2×short + 双槽空标记 + 4 串。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("内存关系"));
            bytes.extend(encoded(""));
            bytes.extend(encoded("类"));
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            for _ in 0..4 {
                bytes.extend(encoded(""));
            }
            bytes
        }
        ReaderKind::QueryTableNameBody10_1 => {
            // 版本 1（无符号门内）+ 3 串 + 工作空间槽/查询槽空标记。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("查询表"));
            bytes.extend(encoded(""));
            bytes.extend(encoded(""));
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::RelQueryTableNameBody10_1 => {
            // 版本 2 满形态（读 0x26、不读 0x28）。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(encoded(""));
            bytes.extend(encoded(""));
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded(""));
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(7_u16.to_le_bytes());
            bytes
        }
        ReaderKind::DbTableNameBody10_1 => {
            // 无版本字段：3 串 + 工作空间槽空标记 + 第 4 串 + int32 + 尾槽空标记。
            let mut bytes = encoded("目录");
            bytes.extend(encoded("表名"));
            bytes.extend(encoded("类别"));
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded("第四串"));
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::DatasetNameSlot10_1 => {
            // 分派入口：GeometricNetworkName 新对象（头 + 版本 1 体，尾槽空标记）。
            let mut bytes =
                mxd2qgz::objects::dataset_name_more::GEOMETRIC_NETWORK_NAME_CLASS_ID.to_vec();
            bytes.extend(21_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded("管网"));
            bytes.extend(encoded(""));
            bytes.extend(encoded("几何网络"));
            bytes.extend([0_u8; 16]);
            bytes
        }
        // —— C 轮 patch + LegendItem 族 + MapSurround 族 ——
        ReaderKind::PatchBody10_1 => {
            // 版本 1（无 ≥2 的尾 short）+ 几何槽空标记 + 补丁名。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded("补丁"));
            bytes
        }
        ReaderKind::HorizontalBarLegendItemBody10_1 => {
            // 条目版本 1 + BaseLegendItem 字段序（基类版本 3 全空槽）+ 尾段
            // int32（−0x1c）+ double ×2（−0x14/−0xc）。注意 int32 在基类字段
            // 序之后（与 VerticalLegendItem 的「版本 → int32 → 基类」不同）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(base_legend_item_v3_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::NestedLegendItemBody10_1 => {
            // 条目版本 1 + 基类字段序 + 尾段 int32（−0x2c）+ double（−0x24）
            // + 两个符号槽 + 3 short + int32（−0xc）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(base_legend_item_v3_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            bytes
        }
        ReaderKind::VerticalLegendItemBody10_1 => {
            // 条目版本 1 + int32 + 基类字段序（与 HorizontalLegendItem 同序）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(base_legend_item_v3_bytes());
            bytes
        }
        ReaderKind::BaseLegendItemFields10_1 => {
            // 统一基类字段序入口：基类版本 3 全空槽形态。
            base_legend_item_v3_bytes()
        }
        ReaderKind::MapTitleBody10_1 => {
            // 自身版本 1 + 共同前缀版本 3（名称 + 关联地图空标记 + short +
            // 32 字节）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(encoded("标题"));
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 32]);
            bytes
        }
        ReaderKind::ScalebarBody10_1 => {
            // 版本 1 满形态（字体槽全部空标记，避免嵌套类头）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(encoded("比例尺"));
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 32]);
            for _ in 0..3 {
                bytes.extend(7_i32.to_le_bytes());
            }
            bytes.extend(2.5_f64.to_le_bytes());
            for _ in 0..3 {
                bytes.extend(1_u16.to_le_bytes());
            }
            for _ in 0..16 {
                bytes.extend(0x1111_1111_u32.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // IFont 槽 A
            bytes.extend(0x2222_2222_u32.to_le_bytes());
            bytes.extend(8_i32.to_le_bytes());
            bytes.extend([0_u8; 16]); // IFont 槽 B
            bytes.extend(0x3333_3333_u32.to_le_bytes());
            bytes.extend(encoded("0.1"));
            for _ in 0..4 {
                bytes.extend(9_i32.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // IFont 槽 C
            bytes.extend(0x4444_4444_u32.to_le_bytes());
            bytes
        }
        ReaderKind::ScaleTextBody10_1 => {
            // 版本 3 满形态（TextElement/NumericFormat 槽空标记）。
            let mut bytes = 3_u16.to_le_bytes().to_vec();
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(encoded("比例尺文本"));
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 32]);
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(encoded("1:1000"));
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded("%"));
            bytes.extend(encoded("m"));
            bytes.extend([0_u8; 16]);
            bytes.extend(encoded("1:%s"));
            bytes
        }
        ReaderKind::MarkerNorthArrowBody10_1 => {
            // 自身版本 2 + 基类版本 4 + 前缀 + 基类字段 + MarkerElement 槽
            // 空标记。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(encoded("指北针"));
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 32]);
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend(3.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(4_i32.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::LegendFormatBody10_1 => {
            // 版本 2 满形态（TextSymbol/补丁槽空标记）。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(5_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(1_i32.to_le_bytes());
            for _ in 0..11 {
                bytes.extend(1.5_f64.to_le_bytes());
            }
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::ScalebarSubclassBody10_1 => {
            // SingleDivision 满形态（基类版本 6、中段 1 个填充槽、尾段版本 2，
            // 全部符号槽空标记）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(6_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend(encoded("子类"));
            bytes.extend([0_u8; 16]);
            bytes.extend(0_u16.to_le_bytes());
            bytes.extend([0_u8; 32]);
            bytes.extend(1.0_f64.to_le_bytes());
            for _ in 0..3 {
                bytes.extend(2_u16.to_le_bytes());
            }
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(encoded("#,##0"));
            bytes.extend(4_i32.to_le_bytes());
            bytes.extend(5.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(6_i32.to_le_bytes());
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend(8.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(9_i32.to_le_bytes());
            bytes.extend(10.0_f64.to_le_bytes());
            bytes.extend(11_u16.to_le_bytes());
            bytes.extend(12_u16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(13.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes.extend(14.0_f64.to_le_bytes());
            bytes.extend(15.0_f64.to_le_bytes());
            bytes.extend(16_i32.to_le_bytes());
            bytes.extend(17_i32.to_le_bytes());
            bytes
        }
        // —— D 轮 Variant 对象分支 + F 轮 GCS/Unknown ——
        ReaderKind::VariantObjectBranch10_1 => {
            // 零 GUID 空对象编码（18 字节）。
            let mut bytes = 9_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::GeographicCoordinateSystem10_1 => {
            // 保存版本 2（仅 WKT 块）。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend((b"GEOGCS[x]".len() as u32).to_le_bytes());
            bytes.extend_from_slice(b"GEOGCS[x]");
            bytes
        }
        ReaderKind::UnknownCoordinateSystem10_1 => {
            // 保存版本 0：4 个字符串，无精度/容差段。
            let mut bytes = 0_u16.to_le_bytes().to_vec();
            for _ in 0..4 {
                bytes.extend(encoded(""));
            }
            bytes
        }
        // —— G 轮 ColorRamp/Callout/PictureFill/元素装饰 ——
        ReaderKind::RandomColorRampBody10_1 => random_color_ramp_body_bytes(),
        ReaderKind::MultiPartColorRampBody10_1 => {
            // 版本 2 + 1 个 Random 子色带 + 尾段 1 个 double。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("mp"));
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(mxd2qgz::objects::color_ramp::RANDOM_COLOR_RAMP_CLASS_ID);
            bytes.extend(5_u32.to_le_bytes());
            bytes.extend(&random_color_ramp_body_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::PresetColorRampBody10_1 => {
            // 版本 1 + 1 个 RgbColor 颜色（raw_int32=0、color_count=1）。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("preset"));
            bytes.extend(0_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(RGB_COLOR_CLASS_ID);
            bytes.extend(7_u32.to_le_bytes());
            bytes.extend(&rgb_color_body_bytes());
            bytes
        }
        ReaderKind::AlgorithmicColorRampBody10_1 => {
            // 版本 1 + 起止两个 RgbColor。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(encoded("algo"));
            bytes.extend(3_i32.to_le_bytes());
            bytes.extend(RGB_COLOR_CLASS_ID);
            bytes.extend(10_u32.to_le_bytes());
            bytes.extend(&rgb_color_body_bytes());
            bytes.extend(RGB_COLOR_CLASS_ID);
            bytes.extend(11_u32.to_le_bytes());
            bytes.extend(&rgb_color_body_bytes());
            bytes
        }
        ReaderKind::ColorRampSlot10_1 => {
            // 槽分派入口：RandomColorRamp 新对象（头 + 体）。
            let mut bytes = mxd2qgz::objects::color_ramp::RANDOM_COLOR_RAMP_CLASS_ID.to_vec();
            bytes.extend(6_u32.to_le_bytes());
            bytes.extend(&random_color_ramp_body_bytes());
            bytes
        }
        ReaderKind::BalloonCalloutBody10_1 => {
            // 版本 1 + 样式 + 4 double + 填充符号槽空标记 + Point 锚点 +
            // 圆角 double。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(2_i32.to_le_bytes());
            for value in [1.0_f64, 2.0, 3.0, 4.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend([0_u8; 16]);
            bytes.extend(mxd2qgz::objects::geometry::POINT_CLASS_ID);
            bytes.extend(21_u32.to_le_bytes());
            bytes.extend(&point_body_bytes());
            bytes.extend(0.5_f64.to_le_bytes());
            bytes
        }
        ReaderKind::LineCalloutBody10_1 => {
            // 版本 1 + 3 个符号槽空标记 + 锚点空标记 + 风格 + 5 边距。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            for _ in 0..3 {
                bytes.extend([0_u8; 16]);
            }
            bytes.extend([0_u8; 16]);
            bytes.extend(9.0_f64.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes
        }
        ReaderKind::SimpleLineCalloutBody10_1 => {
            // 版本 1 + double + 几何槽 Point 新对象 + 引线/锚点槽空标记。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(mxd2qgz::objects::geometry::POINT_CLASS_ID);
            bytes.extend(41_u32.to_le_bytes());
            bytes.extend(&point_body_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::MarkerTextBackgroundBody10_1 => {
            // 版本 1 + short + 符号槽空标记。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::MarkerBackgroundSlot10_1 => {
            // 槽分派入口：MarkerTextBackground 新对象（头 + 体，符号槽空标记）。
            let mut bytes =
                mxd2qgz::objects::callout_background::MARKER_TEXT_BACKGROUND_CLASS_ID.to_vec();
            bytes.extend(55_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes
        }
        ReaderKind::PictureFillSymbolBody10_1 => {
            // 版本 8 主路径（图像路径 + 3 色 + 空轮廓槽 + 标量组）。
            let mut bytes = 8_u16.to_le_bytes().to_vec();
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(5_i32.to_le_bytes());
            bytes.extend(encoded("C:/pic.png"));
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            for key in [3_u32, 4, 5] {
                bytes.extend(RGB_COLOR_CLASS_ID);
                bytes.extend(key.to_le_bytes());
                bytes.extend(&rgb_color_body_bytes());
            }
            bytes.extend([0_u8; 16]);
            for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // 读后即弃 double ×2
            bytes.extend(11_i32.to_le_bytes());
            bytes.extend(12_i32.to_le_bytes());
            bytes.extend([0x01_u8]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes
        }
        ReaderKind::PictureFillSymbolSlot10_1 => {
            // 槽分派入口：PictureFillSymbol 新对象（头 + 版本 8 体）。
            let mut bytes =
                mxd2qgz::objects::picture_fill_symbol::PICTURE_FILL_SYMBOL_CLASS_ID.to_vec();
            bytes.extend(61_u32.to_le_bytes());
            bytes.extend(8_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(5_i32.to_le_bytes());
            bytes.extend(encoded("C:/pic.png"));
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            for key in [3_u32, 4, 5] {
                bytes.extend(RGB_COLOR_CLASS_ID);
                bytes.extend(key.to_le_bytes());
                bytes.extend(&rgb_color_body_bytes());
            }
            bytes.extend([0_u8; 16]);
            for value in [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend([0_u8; 16]); // 读后即弃 double ×2
            bytes.extend(11_i32.to_le_bytes());
            bytes.extend(12_i32.to_le_bytes());
            bytes.extend([0x01_u8]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes
        }
        ReaderKind::SymbolBorderBody10_1 => {
            // 版本 2 满形态：double + 符号槽空标记 + short + double。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(3.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::SymbolBackgroundBody10_1 => {
            // 同 Border 布局。
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(3.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::SymbolShadowBody10_1 => {
            // 版本 1：符号槽空标记 + short + double ×2。
            let mut bytes = 1_u16.to_le_bytes().to_vec();
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::BorderSlot10_1 => {
            // 槽分派入口：SymbolBorder 新对象（头 + 版本 2 体）。
            let mut bytes = mxd2qgz::objects::element_decor::SYMBOL_BORDER_CLASS_ID.to_vec();
            bytes.extend(70_u32.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(3.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::BackgroundSlot10_1 => {
            // 槽分派入口：SymbolBackground 新对象（头 + 版本 2 体）。
            let mut bytes = mxd2qgz::objects::element_decor::SYMBOL_BACKGROUND_CLASS_ID.to_vec();
            bytes.extend(71_u32.to_le_bytes());
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(3.0_f64.to_le_bytes());
            bytes
        }
        ReaderKind::ShadowSlot10_1 => {
            // 槽分派入口：SymbolShadow 新对象（头 + 版本 1 体）。
            let mut bytes = mxd2qgz::objects::element_decor::SYMBOL_SHADOW_CLASS_ID.to_vec();
            bytes.extend(72_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0_u8; 16]);
            bytes.extend(7_u16.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes
        }
    }
}

/// 以统一限额调用目标读取器，只保留成败与游标行为，供跨读取器断言复用。
fn try_read(kind: ReaderKind, reader: &mut Reader<'_>) -> io::Result<()> {
    match kind {
        ReaderKind::NativeString10_1 => read_string_10_1(reader, 1024).map(|_| ()),
        ReaderKind::NativeString10_8 => read_string_10_8(reader, 1024).map(|_| ()),
        ReaderKind::Layout10_1 => read_layout_10_1(reader, 1024).map(|_| ()),
        ReaderKind::ArcMapHeader10_1 => read_arcmap_header_10_1(reader, 1024).map(|_| ()),
        ReaderKind::Layout10_8 => read_layout_10_8(reader, 1024).map(|_| ()),
        ReaderKind::MapsRoot10_8 => read_root_prefix_10_8(reader, 16).map(|_| ()),
        ReaderKind::MapsRoot10_1 => read_root_prefix_10_1(reader, 16).map(|_| ()),
        ReaderKind::MapPrefix10_1 => read_map_prefix_10_1(reader, 1024).map(|_| ()),
        ReaderKind::MapSuffix10_1 => {
            read_map_suffix_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::LayerList10_1 => {
            read_layer_list_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::FeatureLayerPrefix10_1 => {
            read_feature_layer_prefix_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::FdoGraphicsLayerPrefix10_1 => {
            mxd2qgz::layers::fdo_graphics_layer::read_fdo_graphics_layer_prefix_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
            )
            .map(|_| ())
        }
        ReaderKind::GroupLayerPrefix10_1 => {
            read_group_layer_prefix_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::CompositeGraphicsPrefix10_1 => {
            read_composite_graphics_layer_prefix_10_1(reader, 1024, &mut RefTable10_1::new())
                .map(|_| ())
        }
        ReaderKind::Envelope10_1 => {
            read_envelope_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::ObjectHeader10_1 => read_header_10_1(reader).map(|_| ()),
        ReaderKind::ObjectHeader10_8 => read_header_10_8(reader).map(|_| ()),
        // 条目分类在空表上进行：完整输入（新键）会被登记，截断前缀的失败
        // 路径不得修改登记表（该性质由 object_refs 单元测试单独断言）。
        ReaderKind::ObjectEntry10_1 => {
            read_object_entry_10_1(reader, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::SimpleRendererBody10_1 => {
            mxd2qgz::layers::feature_layer_tail::read_simple_renderer_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                35,
            )
            .map(|_| ())
        }
        ReaderKind::ElementCollectionBody10_1 => {
            mxd2qgz::objects::element_collection::read_element_collection_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
            )
            .map(|_| ())
        }
        ReaderKind::FeatureDatasetNameBody10_1 => {
            read_feature_dataset_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 59)
                .map(|_| ())
        }
        ReaderKind::TopologyNameBody10_1 => {
            read_topology_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 60).map(|_| ())
        }
        ReaderKind::FdoLegacyPrefix10_1 => {
            read_fdo_graphics_layer_legacy_prefix_10_1(reader, 1024, &mut RefTable10_1::new())
                .map(|_| ())
        }
        ReaderKind::AoiBookmark10_1 => {
            read_aoi_bookmark_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::FeatureBookmark10_1 => {
            read_feature_bookmark_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::LegendPrefix10_1 => {
            read_legend_prefix_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::CoreColorBody10_1 => {
            mxd2qgz::objects::display_color::read_core_color_body_10_1(
                reader,
                &mut RefTable10_1::new(),
                91,
                "X",
            )
            .map(|_| ())
        }
        ReaderKind::CmykColorBody10_1 => {
            mxd2qgz::objects::display_color::read_cmyk_color_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                92,
            )
            .map(|_| ())
        }
        ReaderKind::ColorSlot10_1 => mxd2qgz::objects::display_color::read_color_slot_10_1(
            reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .map(|_| ()),
        ReaderKind::SimpleLineSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_simple_line_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                21,
            )
            .map(|_| ())
        }
        ReaderKind::SimpleFillSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_simple_fill_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                22,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::MarkerLineSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_marker_line_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                23,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::CartographicLineSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_cartographic_line_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                24,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::HashLineSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_hash_line_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                25,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::DisplaySymbolSlot10_1 => {
            mxd2qgz::objects::display_symbol::read_display_symbol_slot_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::LegendClassFormatBody10_1 => {
            mxd2qgz::layers::legend_format::read_legend_class_format_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                61,
            )
            .map(|_| ())
        }
        ReaderKind::HorizontalLegendItemBody10_1 => {
            mxd2qgz::layers::legend_item::read_horizontal_legend_item_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                62,
            )
            .map(|_| ())
        }
        ReaderKind::EditTemplateBody10_1 => {
            mxd2qgz::layers::edit_template::read_edit_template_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                63,
            )
            .map(|_| ())
        }
        ReaderKind::VariantString10_8 | ReaderKind::VariantFixed10_8 => {
            read_value_10_8(reader, u32::MAX).map(|_| ())
        }
        ReaderKind::FeatureLayerSuccessor10_1 => {
            mxd2qgz::layers::feature_layer_tail::read_feature_layer_successor_10_1(
                reader,
                34,
                1024,
                &mut RefTable10_1::new(),
            )
            .map(|_| ())
        }
        ReaderKind::PropertySetBody10_1 => {
            read_property_set_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::TextElementBody10_1 => read_element_body_of_class_10_1(
            &TEXT_ELEMENT_CLASS_ID,
            reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .map(|_| ()),
        ReaderKind::MarkerElementBody10_1 => read_element_body_of_class_10_1(
            &MARKER_ELEMENT_CLASS_ID,
            reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .map(|_| ()),
        ReaderKind::LineElementBody10_1 => read_element_body_of_class_10_1(
            &LINE_ELEMENT_CLASS_ID,
            reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .map(|_| ()),
        ReaderKind::RgbColorBody10_1 => {
            read_rgb_color_body_10_1(reader, &mut RefTable10_1::new(), 93).map(|_| ())
        }
        ReaderKind::SimpleMarkerSymbolBody10_1 => {
            read_simple_marker_symbol_body_10_1(reader, 1024, &mut RefTable10_1::new(), 94)
                .map(|_| ())
        }
        ReaderKind::TextSymbolBody10_1 => {
            mxd2qgz::objects::element_body::read_text_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
            )
            .map(|_| ())
        }
        // —— 接线轮扩充读取器 ——
        ReaderKind::AnnotatePropsBody10_1 => {
            read_annotate_props_collection_body_10_1(reader, 1024, &mut RefTable10_1::new(), 59)
                .map(|_| ())
        }
        ReaderKind::LabelEnginePropsBody10_1 => {
            read_label_engine_props_body_10_1(reader, 1024, &mut RefTable10_1::new(), 60)
                .map(|_| ())
        }
        ReaderKind::EditTemplateManagerBody10_1 => {
            read_edit_template_manager_body_10_1(reader, 1024, &mut RefTable10_1::new(), 76)
                .map(|_| ())
        }
        ReaderKind::FeatureIdSetBody10_1 => {
            read_feature_id_set_body_10_1(reader, &mut RefTable10_1::new(), 75).map(|_| ())
        }
        ReaderKind::TemplateBody10_1 => mxd2qgz::objects::display_symbol::read_template_body_10_1(
            reader,
            1024,
            &mut RefTable10_1::new(),
            71,
        )
        .map(|_| ()),
        ReaderKind::LineDecorationBody10_1 => {
            mxd2qgz::objects::display_symbol::read_line_decoration_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                80,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::SimpleLineDecorationElementBody10_1 => {
            mxd2qgz::objects::display_symbol::read_simple_line_decoration_element_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                81,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::MultiLayerLineSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_multi_layer_line_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                60,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::MultiLayerMarkerSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_multi_layer_marker_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                70,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::MultiLayerFillSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_multi_layer_fill_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                72,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::CharacterMarkerSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_character_marker_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                75,
            )
            .map(|_| ())
        }
        ReaderKind::PictureMarkerSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_picture_marker_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                90,
            )
            .map(|_| ())
        }
        ReaderKind::PictureLineSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_picture_line_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                100,
            )
            .map(|_| ())
        }
        ReaderKind::ArrowMarkerSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_arrow_marker_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                110,
            )
            .map(|_| ())
        }
        ReaderKind::LineFillSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_line_fill_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                102,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::MarkerFillSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_marker_fill_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                103,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::GradientFillSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_gradient_fill_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                104,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::DotDensityFillSymbolBody10_1 => {
            mxd2qgz::objects::display_symbol::read_dot_density_fill_symbol_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                120,
                mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
            )
            .map(|_| ())
        }
        ReaderKind::SymbolSlotFull10_1 => read_symbol_slot_full_10_1(
            reader,
            1024,
            &mut RefTable10_1::new(),
            mxd2qgz::objects::display_symbol::MAX_SYMBOL_NESTING_DEPTH,
        )
        .map(|_| ()),
        ReaderKind::PolylineBody10_1 => {
            read_polyline_body_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::GeometryBodyOfClass10_1 => read_geometry_body_of_class_10_1(
            &POLYLINE_CLASS_ID,
            reader,
            1024,
            &mut RefTable10_1::new(),
        )
        .map(|_| ()),
        ReaderKind::NumericFormatBody10_1 => {
            read_numeric_format_body_10_1(reader, &mut RefTable10_1::new(), 42).map(|_| ())
        }
        ReaderKind::PercentageFormatBody10_1 => {
            read_percentage_format_body_10_1(reader, &mut RefTable10_1::new(), 43).map(|_| ())
        }
        ReaderKind::FractionFormatBody10_1 => {
            read_fraction_format_body_10_1(reader, &mut RefTable10_1::new(), 44).map(|_| ())
        }
        ReaderKind::RateFormatBody10_1 => {
            read_rate_format_body_10_1(reader, 1024, &mut RefTable10_1::new(), 45).map(|_| ())
        }
        ReaderKind::HyperlinkBody10_1 => {
            read_hyperlink_body_10_1(reader, 1024, &mut RefTable10_1::new(), 80).map(|_| ())
        }
        // —— 整合接线轮：E 轮 StdFont/StdPicture 体 ——
        ReaderKind::StdFontBody10_1 => read_std_font_body_10_1(reader, 1024).map(|_| ()),
        ReaderKind::StdPictureBody10_1 => read_std_picture_body_10_1(reader, 1024).map(|_| ()),
        // —— 整合接线轮：F 轮 BOLP 族 / 引擎 / UID / HLEP / DEP 前缀 ——
        ReaderKind::BolpBody10_1 => {
            read_bolp_body_10_1(reader, 1024, &mut RefTable10_1::new(), 46).map(|_| ())
        }
        ReaderKind::LineLabelPositionBody10_1 => {
            read_line_label_position_body_10_1(reader, 1024, &mut RefTable10_1::new(), 47)
                .map(|_| ())
        }
        ReaderKind::LineLabelPlacementPrioritiesBody10_1 => {
            read_line_label_placement_priorities_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                48,
            )
            .map(|_| ())
        }
        ReaderKind::PointPlacementPrioritiesBody10_1 => {
            read_point_placement_priorities_body_10_1(reader, 1024, &mut RefTable10_1::new(), 49)
                .map(|_| ())
        }
        ReaderKind::VbscriptEngineBody10_1 => {
            read_annotation_vbscript_engine_body_10_1(reader, 1024, &mut RefTable10_1::new(), 50)
                .map(|_| ())
        }
        ReaderKind::UidBody10_1 => read_uid_body_10_1(reader, 51).map(|_| ()),
        ReaderKind::HotLinkExpressionPropsBody10_1 => {
            read_hot_link_expression_props_body_10_1(reader, 1024, &mut RefTable10_1::new(), 52)
                .map(|_| ())
        }
        ReaderKind::DisplayExpressionPropsPrefix10_1 => {
            read_display_expression_props_prefix_10_1(reader, 1024, &mut RefTable10_1::new(), 53)
                .map(|_| ())
        }
        // —— 整合接线轮：G 轮 Polygon/Point/Multipoint 体 + 类型化分派 ——
        ReaderKind::PolygonBody10_1 => {
            read_polygon_body_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::PointBody10_1 => {
            read_point_body_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::MultipointBody10_1 => {
            read_multipoint_body_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::GeometryBody10_1 => {
            read_geometry_body_10_1(&POLYLINE_CLASS_ID, reader, 1024, &mut RefTable10_1::new())
                .map(|_| ())
        }
        // —— 整合接线轮：H 轮数值格式同族五类 ——
        ReaderKind::CurrencyFormatBody10_1 => {
            read_currency_format_body_10_1(reader, &mut RefTable10_1::new(), 54).map(|_| ())
        }
        ReaderKind::LatLonFormatBody10_1 => {
            read_latlon_format_body_10_1(reader, &mut RefTable10_1::new(), 55).map(|_| ())
        }
        ReaderKind::AngleFormatBody10_1 => {
            read_angle_format_body_10_1(reader, &mut RefTable10_1::new(), 56).map(|_| ())
        }
        ReaderKind::ScientificFormatBody10_1 => {
            read_scientific_format_body_10_1(reader, &mut RefTable10_1::new(), 57).map(|_| ())
        }
        ReaderKind::CustomNumberFormatBody10_1 => {
            read_custom_number_format_body_10_1(reader, 1024, &mut RefTable10_1::new(), 58)
                .map(|_| ())
        }
        // —— 本轮整合接线：A 轮 dep_tail TimeData 族 ——
        ReaderKind::TimeBody10_1 => {
            read_time_body_10_1(reader, 1024, &mut RefTable10_1::new(), 60).map(|_| ())
        }
        ReaderKind::TimeZoneInfoBody10_1 => {
            read_time_zone_info_body_10_1(reader, 1024, &mut RefTable10_1::new(), 61).map(|_| ())
        }
        ReaderKind::TimeReferenceBody10_1 => {
            read_time_reference_body_10_1(reader, 1024, &mut RefTable10_1::new(), 62).map(|_| ())
        }
        ReaderKind::TimeExtentBody10_1 => {
            read_time_extent_body_10_1(reader, 1024, &mut RefTable10_1::new(), 63).map(|_| ())
        }
        ReaderKind::TimeDataFamily10_1 => {
            read_time_data_family_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        // —— B 轮 IDatasetName 槽族六类 + 分派 ——
        ReaderKind::GeometricNetworkNameBody10_1 => {
            read_geometric_network_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 63)
                .map(|_| ())
        }
        ReaderKind::RelationshipClassNameBody10_1 => {
            read_relationship_class_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 64)
                .map(|_| ())
        }
        ReaderKind::MemoryRelationshipClassNameBody10_1 => {
            read_memory_relationship_class_name_body_10_1(
                reader,
                1024,
                &mut RefTable10_1::new(),
                65,
            )
            .map(|_| ())
        }
        ReaderKind::QueryTableNameBody10_1 => {
            read_query_table_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 66).map(|_| ())
        }
        ReaderKind::RelQueryTableNameBody10_1 => {
            read_rel_query_table_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 67)
                .map(|_| ())
        }
        ReaderKind::DbTableNameBody10_1 => {
            read_db_table_name_body_10_1(reader, 1024, &mut RefTable10_1::new(), 68).map(|_| ())
        }
        ReaderKind::DatasetNameSlot10_1 => {
            read_dataset_name_slot_10_1(reader, 1024, &mut RefTable10_1::new(), &mut None)
                .map(|_| ())
        }
        // —— C 轮 patch + LegendItem 族 + MapSurround 族 ——
        ReaderKind::PatchBody10_1 => {
            read_patch_body_10_1(reader, 1024, &mut RefTable10_1::new(), 69).map(|_| ())
        }
        ReaderKind::HorizontalBarLegendItemBody10_1 => {
            read_horizontal_bar_legend_item_body_10_1(reader, 1024, &mut RefTable10_1::new(), 70)
                .map(|_| ())
        }
        ReaderKind::NestedLegendItemBody10_1 => {
            read_nested_legend_item_body_10_1(reader, 1024, &mut RefTable10_1::new(), 71)
                .map(|_| ())
        }
        ReaderKind::VerticalLegendItemBody10_1 => {
            read_vertical_legend_item_body_10_1(reader, 1024, &mut RefTable10_1::new(), 72)
                .map(|_| ())
        }
        ReaderKind::BaseLegendItemFields10_1 => {
            read_base_legend_item_fields_10_1(reader, 1024, &mut RefTable10_1::new()).map(|_| ())
        }
        ReaderKind::MapTitleBody10_1 => {
            read_map_title_body_10_1(reader, 1024, &mut RefTable10_1::new(), 73).map(|_| ())
        }
        ReaderKind::ScalebarBody10_1 => {
            read_scalebar_body_10_1(reader, 1024, &mut RefTable10_1::new(), 74).map(|_| ())
        }
        ReaderKind::ScaleTextBody10_1 => {
            read_scale_text_body_10_1(reader, 1024, &mut RefTable10_1::new(), 75).map(|_| ())
        }
        ReaderKind::MarkerNorthArrowBody10_1 => {
            read_marker_north_arrow_body_10_1(reader, 1024, &mut RefTable10_1::new(), 76)
                .map(|_| ())
        }
        ReaderKind::LegendFormatBody10_1 => {
            read_legend_format_body_10_1(reader, 1024, &mut RefTable10_1::new(), 77).map(|_| ())
        }
        ReaderKind::ScalebarSubclassBody10_1 => read_scalebar_subclass_body_10_1(
            reader,
            1024,
            &mut RefTable10_1::new(),
            78,
            mxd2qgz::maps::map_surround_family::ScalebarSubclassKind10_1::SingleDivision,
        )
        .map(|_| ()),
        // —— D 轮 Variant 对象分支 + F 轮 GCS/Unknown 坐标系 ——
        ReaderKind::VariantObjectBranch10_1 => read_variant_object_branch_10_1(reader).map(|_| ()),
        ReaderKind::GeographicCoordinateSystem10_1 => {
            read_geographic_coordinate_system_10_1(reader, 1024).map(|_| ())
        }
        ReaderKind::UnknownCoordinateSystem10_1 => {
            read_unknown_coordinate_system_10_1(reader, 1024).map(|_| ())
        }
        // —— G 轮 ColorRamp/Callout/PictureFill/元素装饰 ——
        ReaderKind::RandomColorRampBody10_1 => {
            read_random_color_ramp_body_10_1(reader, 1024, &mut RefTable10_1::new(), 79).map(|_| ())
        }
        ReaderKind::MultiPartColorRampBody10_1 => {
            read_multi_part_color_ramp_body_10_1(reader, 1024, &mut RefTable10_1::new(), 80, 8)
                .map(|_| ())
        }
        ReaderKind::PresetColorRampBody10_1 => {
            read_preset_color_ramp_body_10_1(reader, 1024, &mut RefTable10_1::new(), 81).map(|_| ())
        }
        ReaderKind::AlgorithmicColorRampBody10_1 => {
            read_algorithmic_color_ramp_body_10_1(reader, 1024, &mut RefTable10_1::new(), 82)
                .map(|_| ())
        }
        ReaderKind::ColorRampSlot10_1 => {
            read_color_ramp_slot_10_1(reader, 1024, &mut RefTable10_1::new(), 8, 0).map(|_| ())
        }
        ReaderKind::BalloonCalloutBody10_1 => {
            read_balloon_callout_body_10_1(reader, 1024, &mut RefTable10_1::new(), 83, 8)
                .map(|_| ())
        }
        ReaderKind::LineCalloutBody10_1 => {
            read_line_callout_body_10_1(reader, 1024, &mut RefTable10_1::new(), 84, 8).map(|_| ())
        }
        ReaderKind::SimpleLineCalloutBody10_1 => {
            read_simple_line_callout_body_10_1(reader, 1024, &mut RefTable10_1::new(), 85, 8)
                .map(|_| ())
        }
        ReaderKind::MarkerTextBackgroundBody10_1 => {
            read_marker_text_background_body_10_1(reader, 1024, &mut RefTable10_1::new(), 86, 8)
                .map(|_| ())
        }
        ReaderKind::MarkerBackgroundSlot10_1 => {
            read_marker_background_slot_10_1(reader, 1024, &mut RefTable10_1::new(), 8).map(|_| ())
        }
        ReaderKind::PictureFillSymbolBody10_1 => {
            read_picture_fill_symbol_body_10_1(reader, 1024, &mut RefTable10_1::new(), 87, 8)
                .map(|_| ())
        }
        ReaderKind::PictureFillSymbolSlot10_1 => {
            read_picture_fill_symbol_slot_10_1(reader, 1024, &mut RefTable10_1::new(), 8)
                .map(|_| ())
        }
        ReaderKind::SymbolBorderBody10_1 => {
            read_symbol_border_body_10_1(reader, 1024, &mut RefTable10_1::new(), 88, 8).map(|_| ())
        }
        ReaderKind::SymbolBackgroundBody10_1 => {
            read_symbol_background_body_10_1(reader, 1024, &mut RefTable10_1::new(), 89, 8)
                .map(|_| ())
        }
        ReaderKind::SymbolShadowBody10_1 => {
            read_symbol_shadow_body_10_1(reader, 1024, &mut RefTable10_1::new(), 90, 8).map(|_| ())
        }
        ReaderKind::BorderSlot10_1 => {
            read_border_slot_10_1(reader, 1024, &mut RefTable10_1::new(), 8).map(|_| ())
        }
        ReaderKind::BackgroundSlot10_1 => {
            read_background_slot_10_1(reader, 1024, &mut RefTable10_1::new(), 8).map(|_| ())
        }
        ReaderKind::ShadowSlot10_1 => {
            read_shadow_slot_10_1(reader, 1024, &mut RefTable10_1::new(), 8).map(|_| ())
        }
    }
}

#[test]
fn every_reader_rejects_every_truncated_prefix_without_panicking_or_advancing() {
    for kind in all_kinds() {
        let input = sample(kind);
        // 完整输入必须成功且被完整消费，证明该样本对目标读取器确实合法。
        let mut reader = Reader::new(&input);
        try_read(kind, &mut reader)
            .unwrap_or_else(|error| panic!("{kind:?}: 合法样本被拒绝: {error}"));
        assert_eq!(reader.position(), input.len(), "{kind:?}");
        // 0..n 的每个截断前缀：必须失败、不 panic、游标不推进，且重读结果一致。
        for end in 0..input.len() {
            let mut reader = Reader::new(&input[..end]);
            let first = try_read(kind, &mut reader)
                .err()
                .unwrap_or_else(|| panic!("{kind:?}: 截断前缀 end={end} 未返回错误"));
            assert_eq!(reader.position(), 0, "{kind:?}: end={end}");
            let second = try_read(kind, &mut reader)
                .err()
                .unwrap_or_else(|| panic!("{kind:?}: 重读 end={end} 未返回错误"));
            assert_eq!(first.kind(), second.kind(), "{kind:?}: end={end}");
            assert_eq!(reader.position(), 0, "{kind:?}: end={end}");
        }
    }
}

#[test]
fn non_eof_failures_leave_the_cursor_local_and_repeatable() {
    // 对象头读取没有非 EOF 失败类别，其事务性已由截断扫描覆盖。
    let surrogate = {
        // 声明 4 字节，内容是孤立高代理项。
        let mut bytes = 4_u32.to_le_bytes().to_vec();
        bytes.extend([0x00, 0xd8, 0x00, 0x00]);
        bytes
    };
    let short_out_of_range = {
        let mut bytes = encoded("x");
        bytes.extend(20_i16.to_le_bytes());
        bytes
    };
    let cases: Vec<(ReaderKind, Vec<u8>, ErrorKind, &str)> = vec![
        (
            ReaderKind::NativeString10_8,
            3_u32.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "奇数字节长度",
        ),
        (
            ReaderKind::NativeString10_1,
            u32::MAX.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "非法声明长度",
        ),
        (
            ReaderKind::Layout10_1,
            surrogate.clone(),
            ErrorKind::InvalidData,
            "孤立高代理项",
        ),
        (
            ReaderKind::Layout10_8,
            surrogate,
            ErrorKind::InvalidData,
            "孤立高代理项",
        ),
        (
            ReaderKind::ArcMapHeader10_1,
            short_out_of_range,
            ErrorKind::InvalidData,
            "short 超出头部支持范围",
        ),
        (
            ReaderKind::MapsRoot10_8,
            (-1_i32).to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "负数量",
        ),
        (
            ReaderKind::MapsRoot10_8,
            17_i32.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "数量超过限额",
        ),
        (
            ReaderKind::MapsRoot10_1,
            (-1_i32).to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "10.1 负数量",
        ),
        (
            ReaderKind::MapsRoot10_1,
            17_i32.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "10.1 数量超过限额",
        ),
        (
            ReaderKind::VariantString10_8,
            value_bytes(0xffff, &[1, 2, 3, 4]),
            ErrorKind::InvalidData,
            "未知 VARTYPE",
        ),
        (
            ReaderKind::VariantString10_8,
            value_bytes(9, &[1, 2, 3, 4]),
            ErrorKind::Unsupported,
            "Variant 对象分支",
        ),
        (
            ReaderKind::VariantFixed10_8,
            value_bytes(13, &[1, 2, 3, 4]),
            ErrorKind::Unsupported,
            "Variant 对象分支",
        ),
        (
            ReaderKind::LayerList10_1,
            i32::MAX.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "图层列表计数超限",
        ),
        (
            ReaderKind::LayerList10_1,
            (-1_i32).to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "图层列表负计数",
        ),
        (
            ReaderKind::LayerList10_1,
            [
                1_i32.to_le_bytes().as_slice(),
                4_u16.to_le_bytes().as_slice(),
            ]
            .concat(),
            ErrorKind::InvalidData,
            "图层列表非法标志",
        ),
        (
            ReaderKind::FeatureLayerPrefix10_1,
            35_u16.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "FeatureLayer 版本超限",
        ),
        (
            ReaderKind::Envelope10_1,
            4_u16.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "Envelope 版本超限",
        ),
        (
            ReaderKind::GroupLayerPrefix10_1,
            13_u16.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "GroupLayer 版本超限",
        ),
        (
            ReaderKind::CompositeGraphicsPrefix10_1,
            7_u16.to_le_bytes().to_vec(),
            ErrorKind::InvalidData,
            "CGL 版本超限",
        ),
    ];
    for (kind, bytes, expected, label) in cases {
        let mut reader = Reader::new(&bytes);
        let first = try_read(kind, &mut reader)
            .err()
            .unwrap_or_else(|| panic!("{label}: 该输入应当失败"));
        assert_eq!(first.kind(), expected, "{label}");
        assert_eq!(reader.position(), 0, "{label}: 失败后游标必须回到入口");
        // 失败可重复：同一 Reader 再次读取得到相同错误类别，游标仍在入口。
        let second = try_read(kind, &mut reader)
            .err()
            .unwrap_or_else(|| panic!("{label}: 重读应当失败"));
        assert_eq!(second.kind(), expected, "{label}: 重读结果不一致");
        assert_eq!(reader.position(), 0, "{label}: 重读后游标仍在入口");
    }
}

#[test]
fn huge_declared_lengths_and_limits_fail_explicitly_without_overflow() {
    // AfCore 字符串与 Version 布局：u32::MAX-1 是最大偶数声明长度，在
    // usize::MAX 限额下于读取阶段明确短读；u32::MAX 是奇数，在校验阶段被拒。
    let reads: [&dyn Fn(&mut Reader<'_>, usize) -> io::Result<()>; 5] = [
        &|r: &mut Reader<'_>, m: usize| read_string_10_1(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_string_10_8(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_layout_10_1(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_arcmap_header_10_1(r, m).map(|_| ()),
        &|r: &mut Reader<'_>, m: usize| read_layout_10_8(r, m).map(|_| ()),
    ];
    for read in reads {
        for (declared, expected) in [
            (u32::MAX - 1, ErrorKind::UnexpectedEof),
            (u32::MAX, ErrorKind::InvalidData),
        ] {
            let bytes = declared.to_le_bytes();
            let mut reader = Reader::new(&bytes);
            let error = read(&mut reader, usize::MAX)
                .err()
                .unwrap_or_else(|| panic!("declared={declared:#x}: 应当失败"));
            assert_eq!(error.kind(), expected, "declared={declared:#x}");
            assert_eq!(reader.position(), 0, "declared={declared:#x}");
        }
    }

    // VariantStream：n*2+2 在 u32 上的溢出边界。0x7fff_fffe 仍可表达并在
    // 读取阶段短读；更大的码元数在任何分配前被拒。限额取 u32::MAX 以隔离限额检查。
    for (units, expected) in [
        (0x7fff_fffe_u32, ErrorKind::UnexpectedEof),
        (0x7fff_ffff, ErrorKind::InvalidData),
        (0x8000_0000, ErrorKind::InvalidData),
        (u32::MAX, ErrorKind::InvalidData),
    ] {
        let bytes = value_bytes(8, &units.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let error = read_value_10_8(&mut reader, u32::MAX)
            .err()
            .unwrap_or_else(|| panic!("units={units:#x}: 应当失败"));
        assert_eq!(error.kind(), expected, "units={units:#x}");
        assert_eq!(reader.position(), 0, "units={units:#x}");
    }

    // /Maps：i32::MAX 数量在限额之内也不做任何分配，进入对象循环时立即短读。
    // 10.1 与 10.8 数量校验边界一致；10.1 在对象循环之后还须读取 4 字节尾部值。
    let count_only = i32::MAX.to_le_bytes();
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_8(&mut reader, i32::MAX as u32).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    // 超限额数量被拒；限额参数 u32::MAX 无法由 i32 表示，在读取任何对象前被拒。
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_8(&mut reader, 16).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_8(&mut reader, u32::MAX).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, i32::MAX as u32).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, 16).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&count_only);
    let error = read_root_prefix_10_1(&mut reader, u32::MAX).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(reader.position(), 0);

    // 底层 UTF-16 读取：usize::MAX（奇数）与 usize::MAX-1（偶数）分别被
    // 奇偶校验和边界检查拒绝，游标不变。
    let mut reader = Reader::new(&[0, 0]);
    for (length, expected) in [
        (usize::MAX, ErrorKind::InvalidData),
        (usize::MAX - 1, ErrorKind::UnexpectedEof),
    ] {
        let error = reader.read_utf16_le(length).unwrap_err();
        assert_eq!(error.kind(), expected, "length=0x{length:x}");
        assert_eq!(reader.position(), 0, "length=0x{length:x}");
    }
}

#[test]
fn unknown_class_ids_and_keys_are_preserved_verbatim_without_interpretation() {
    // 合成的未知 GUID；读取器契约是不校验注册，原样保留磁盘字节顺序与键的
    // 全部位模式。全零 16 字节才是空头；其他任何内容都按对象头处理。
    let class_id = [
        0xd8, 0x1c, 0x6c, 0x2e, 0x31, 0xae, 0x44, 0xd8, 0x8e, 0x7a, 0x11, 0x0b, 0x2f, 0x93, 0x7d,
        0xe4,
    ];
    for read_header in [read_header_10_1, read_header_10_8] {
        for key in [0_u32, 1, 0x7fff_ffff, 0x8000_0000, 0xdead_beef, u32::MAX] {
            let mut bytes = class_id.to_vec();
            bytes.extend(key.to_le_bytes());
            bytes.extend(b"opaque body");
            let mut reader = Reader::new(&bytes);
            let header = read_header(&mut reader)
                .unwrap_or_else(|error| panic!("key={key:#x}: 未知类标识不应被拒绝: {error}"));
            assert_eq!(header, ObjectHeader::Object { class_id, key });
            assert_eq!(reader.position(), 20);
            assert_eq!(reader.read_bytes(11).unwrap(), b"opaque body");
        }
    }
    // Null 判定只取决于 16 字节是否全零：首字节或末字节非零都按对象处理。
    for index in [0usize, 15] {
        let mut id = [0_u8; 16];
        id[index] = 1;
        let mut bytes = id.to_vec();
        bytes.extend(0_u32.to_le_bytes());
        for read_header in [read_header_10_1, read_header_10_8] {
            let mut reader = Reader::new(&bytes);
            assert_eq!(
                read_header(&mut reader).unwrap(),
                ObjectHeader::Object {
                    class_id: id,
                    key: 0
                }
            );
        }
    }
    // 键被截断时失败且游标不推进（与截断扫描结论一致）。
    let mut reader = Reader::new(&class_id);
    let error = read_header_10_1(&mut reader).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(reader.position(), 0);
}

#[test]
fn every_vartype_value_has_an_explicit_branch_without_panicking() {
    // 已知分支之外的整个 u16 空间都必须显式拒绝；9/13 是显式不支持的对象分支。
    // 载荷取 8 个非零字节，使固定宽度分支完整、字符串分支因声明码元数
    // 0xa5a5a5a5 超过限额而被拒；字符串分支的完整行为由模块测试覆盖。
    let payload = [0xa5_u8; 8];
    for value_type in 0..=u16::MAX {
        let bytes = value_bytes(value_type, &payload);
        let mut reader = Reader::new(&bytes);
        let result = read_value_10_8(&mut reader, 4);
        match value_type {
            0 => {
                assert_eq!(result.unwrap(), VariantValue10_8::Empty, "VT={value_type}");
                assert_eq!(reader.position(), 2);
            }
            1 => {
                assert_eq!(result.unwrap(), VariantValue10_8::Null, "VT={value_type}");
                assert_eq!(reader.position(), 2);
            }
            2 | 11 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw2([0xa5, 0xa5]),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 4);
            }
            3 | 4 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw4([0xa5; 4]),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 6);
            }
            5 | 7 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw8(payload),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 10);
            }
            17 => {
                assert_eq!(
                    result.unwrap(),
                    VariantValue10_8::Raw1([0xa5]),
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 3);
            }
            8 => {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::InvalidData,
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 0);
            }
            9 | 13 => {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::Unsupported,
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 0);
            }
            _ => {
                assert_eq!(
                    result.unwrap_err().kind(),
                    ErrorKind::InvalidData,
                    "VT={value_type}"
                );
                assert_eq!(reader.position(), 0);
            }
        }
    }
}

#[test]
fn current_readers_complete_boundedly_without_recursive_self_application() {
    // 循环引用（对象经引用表互相引用）需要对象体与引用表编码才能表达；
    // 当前读取器不解析对象体，也不维护对象表，自引用结构在其输入布局中
    // 不可表达。待对象引用编码确定后补充缺失引用、重复标识及循环引用的
    // 覆盖（TODO「自动验证」节）。此处断言现有读取器不递归、不自我放大：
    // 大量连续对象在输入耗尽时以有界步骤完成。
    let mut header = vec![0xa5_u8; 16];
    header.extend(7_u32.to_le_bytes());
    let bytes = header.repeat(512);
    let mut reader = Reader::new(&bytes);
    for index in 0..512 {
        let read_header = if index % 2 == 0 {
            read_header_10_8
        } else {
            read_header_10_1
        };
        assert_eq!(
            read_header(&mut reader).unwrap(),
            ObjectHeader::Object {
                class_id: [0xa5; 16],
                key: 7
            }
        );
    }
    assert_eq!(reader.remaining(), 0);

    // Variant 固定宽度值顺序读取同样有界。
    let value = value_bytes(4, &[1, 2, 3, 4]);
    let bytes = value.repeat(1024);
    let mut reader = Reader::new(&bytes);
    for _ in 0..1024 {
        assert_eq!(
            read_value_10_8(&mut reader, 4).unwrap(),
            VariantValue10_8::Raw4([1, 2, 3, 4])
        );
    }
    assert_eq!(reader.remaining(), 0);

    // 长字符串一次性读取，不因长度递归。
    let text = "地图".repeat(500);
    let bytes = encoded(&text);
    let mut reader = Reader::new(&bytes);
    assert_eq!(read_string_10_8(&mut reader, bytes.len()).unwrap(), text);
    assert_eq!(reader.remaining(), 0);
}
