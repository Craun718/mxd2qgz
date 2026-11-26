//! ArcMap 10.1 RasterLayer 对象体的已确认前缀。
//!
//! 证据来自 RasterLayer.dll 10.1 安装介质静态映像。文件 SHA-256 为
//! `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`，PE32
//! x86，ImageBase `0x7c710000`。主 QI `0x7c73aaa0` 将 IPersistStream IID
//! （`0x7c76778c`）映射到对象偏移 `+0x9c`；该偏移与 RTTI COL
//! `0x7c774088` 中的 `offset=0x9c` 一致。对应 IPersistStream 虚表
//! `0x7c76c2b0` 的 Load 槽 `+0x14` 指向 `0x7c73bd40`。
//!
//! Load `0x7c73bd40` 首先读有符号 int32 版本（`JG` 拒绝大于 17），随后
//! 读取第一条 AfCore String 到 persist-interface this+0x88。独立续段可再读
//! 第二串、定长短值及版本控制的 IName 列表。

use super::feature_layer_tail::{
    LEGEND_CLASS_CLASS_ID, LEGEND_GROUP_CLASS_ID, LegendClassEntry10_1, LegendClassFormatSlot10_1,
    LegendGroupSlot10_1, NumberFormatSlot10_1,
};
use super::legend_format::{
    LEGEND_CLASS_FORMAT_CLASS_ID, LegendClassFormatSlot10_1 as FormatMember,
};
use super::patch::{AREA_PATCH_CLASS_ID, LINE_PATCH_CLASS_ID, PatchGeometrySlot10_1};
use super::raster_classify_renderer::ClassificationMethod10_1;
use super::raster_dataset_name::{
    RasterDatasetNameEntryBody10_1, read_supported_raster_name_entry_10_1,
};
use super::raster_renderer::{
    DOUBLE_ARRAY_CLASS_ID, PansharpeningSlot10_1, RasterRendererPrefix10_1,
    RasterRendererShared10_1, RasterRendererSharedLayout10_1, is_supported_color_ramp_class,
    read_raster_renderer_slot_prefix_10_1, supports_raster_renderer_body_10_1,
};
use super::raster_unique_value::{RasterUniqueValueLayout10_1, UNIQUE_VALUES_CLASS_ID};
use crate::maps::map_surround::LegendItemSlot10_1;
use crate::objects::{
    callout_background::{
        BALLOON_CALLOUT_CLASS_ID, CalloutSlot10_1, GeometrySlotEntry10_1, LINE_CALLOUT_CLASS_ID,
        MARKER_TEXT_BACKGROUND_CLASS_ID, PointSlotEntry10_1, SIMPLE_LINE_CALLOUT_CLASS_ID,
    },
    color_ramp::ColorRampSlot10_1,
    display_color::{
        CMYK_COLOR_CLASS_ID, ColorSlot10_1, GRAY_COLOR_CLASS_ID, HLS_COLOR_CLASS_ID,
        HSV_COLOR_CLASS_ID, RGB_COLOR_CLASS_ID,
    },
    display_symbol::{
        SIMPLE_FILL_SYMBOL_CLASS_ID, SIMPLE_LINE_SYMBOL_CLASS_ID, SIMPLE_MARKER_SYMBOL_CLASS_ID,
        SymbolSlotFull10_1,
    },
    element_body::{FontSlot10_1, TEXT_SYMBOL_CLASS_ID},
    envelope::SpatialReferenceEntry10_1,
    geometry::{
        GeometryBody10_1, MULTIPOINT_CLASS_ID, POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID,
    },
    ole_font::STDFONT_CLASS_ID,
    property_set::{PropertyValue10_1, PropertyValueObjectStream10_1},
    spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
    spatial_reference_slots::{
        GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
    },
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

// 这些 IColor 实现的持久化与转换只访问颜色自身的状态。
const RASTER_RENDERER_COLOR_CLASSES: [[u8; 16]; 5] = [
    RGB_COLOR_CLASS_ID,
    GRAY_COLOR_CLASS_ID,
    HLS_COLOR_CLASS_ID,
    HSV_COLOR_CLASS_ID,
    CMYK_COLOR_CLASS_ID,
];

// DisplayCore.dll 内置的文字路径和解析器只由原样 CLSID 激活。
const SIMPLE_TEXT_PATH_CLASS_ID: [u8; 16] = [
    0x76, 0x3e, 0x5a, 0xb6, 0x93, 0x29, 0xd1, 0x11, 0x9a, 0x43, 0, 0x80, 0xc7, 0xec, 0x5c, 0x96,
];
const BEZIER_TEXT_PATH_CLASS_ID: [u8; 16] = [
    0, 0x10, 0xe2, 0x2d, 0xeb, 0xbd, 0xd1, 0x11, 0x97, 0x0b, 0, 0x80, 0xc7, 0xe0, 0x41, 0x96,
];
const SIMPLE_TEXT_PARSER_CLASS_ID: [u8; 16] = [
    0x91, 0x9b, 0x09, 0xd7, 0x98, 0xe2, 0x9e, 0x49, 0x97, 0x57, 0x72, 0xbb, 0x39, 0xe5, 0x5c, 0xf3,
];

fn simple_symbol_preserves_empty_dataset(
    symbol: &SymbolSlotFull10_1,
    table: &RefTable10_1,
) -> bool {
    let color_is_known = |color: &ColorSlot10_1| {
        matches!(color, ColorSlot10_1::Null) || is_known_renderer_color(color, table)
    };
    match symbol {
        SymbolSlotFull10_1::Null => true,
        SymbolSlotFull10_1::BackReference { key, .. } => matches!(
            table.registered_class(*key),
            Some(
                SIMPLE_MARKER_SYMBOL_CLASS_ID
                    | SIMPLE_LINE_SYMBOL_CLASS_ID
                    | SIMPLE_FILL_SYMBOL_CLASS_ID
            )
        ),
        SymbolSlotFull10_1::SimpleMarkerSymbol(marker) => {
            color_is_known(&marker.color)
                && marker.outline_color.as_ref().is_some_and(color_is_known)
        }
        SymbolSlotFull10_1::SimpleLineSymbol(line) => color_is_known(&line.color),
        SymbolSlotFull10_1::SimpleFillSymbol(fill) => {
            fill.color.as_ref().is_some_and(color_is_known)
                && simple_symbol_preserves_empty_dataset(&fill.outline, table)
        }
        _ => false,
    }
}

fn spatial_reference_preserves_empty_dataset(
    reference: Option<&SpatialReferenceEntry10_1>,
    table: &RefTable10_1,
) -> bool {
    match reference {
        None
        | Some(
            SpatialReferenceEntry10_1::Null
            | SpatialReferenceEntry10_1::Projected { .. }
            | SpatialReferenceEntry10_1::Geographic { .. }
            | SpatialReferenceEntry10_1::UnknownCoordinateSystem { .. },
        ) => true,
        Some(SpatialReferenceEntry10_1::BackReference { key, .. }) => {
            matches!(
                table.registered_class(*key),
                Some(
                    PROJECTED_COORDINATE_SYSTEM_CLASS_ID
                        | GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID
                        | UNKNOWN_COORDINATE_SYSTEM_CLASS_ID
                )
            )
        }
    }
}

fn callout_point_preserves_empty_dataset(
    anchor: Option<&PointSlotEntry10_1>,
    table: &RefTable10_1,
) -> bool {
    match anchor {
        Some(PointSlotEntry10_1::Null) => true,
        Some(PointSlotEntry10_1::BackReference { key, .. }) => {
            table.registered_class(*key) == Some(POINT_CLASS_ID)
        }
        Some(PointSlotEntry10_1::Point(point)) => {
            spatial_reference_preserves_empty_dataset(point.spatial_reference.as_ref(), table)
        }
        None => false,
    }
}

fn anchor_geometry_preserves_empty_dataset(
    geometry: &GeometryBody10_1,
    table: &RefTable10_1,
) -> bool {
    let reference = match geometry {
        GeometryBody10_1::Point(point) => &point.spatial_reference,
        GeometryBody10_1::Multipoint(points) => &points.spatial_reference,
        GeometryBody10_1::Polyline(line) => &line.spatial_reference,
        GeometryBody10_1::Polygon(area) => &area.spatial_reference,
        _ => return false,
    };
    spatial_reference_preserves_empty_dataset(reference.as_ref(), table)
}

fn legend_format_preserves_empty_dataset(
    format: &LegendClassFormatSlot10_1,
    table: &RefTable10_1,
) -> bool {
    match format {
        LegendClassFormatSlot10_1::Null => true,
        LegendClassFormatSlot10_1::BackReference { key, .. } => {
            table.registered_class(*key) == Some(LEGEND_CLASS_FORMAT_CLASS_ID)
        }
        LegendClassFormatSlot10_1::Format(format) => {
            let patch_is_known = |slot: &FormatMember, class_id| match slot {
                FormatMember::Null => true,
                FormatMember::BackReference { key, .. } => {
                    table.registered_class(*key) == Some(class_id)
                }
                FormatMember::LinePatch(patch) | FormatMember::AreaPatch(patch) => {
                    patch.version == 3
                        && match &patch.geometry {
                            PatchGeometrySlot10_1::Null => true,
                            PatchGeometrySlot10_1::BackReference { key, .. } => matches!(
                                table.registered_class(*key),
                                Some(POLYLINE_CLASS_ID | POLYGON_CLASS_ID)
                            ),
                            PatchGeometrySlot10_1::Geometry(body) => {
                                matches!(
                                    body,
                                    GeometryBody10_1::Polyline(_) | GeometryBody10_1::Polygon(_)
                                ) && anchor_geometry_preserves_empty_dataset(body, table)
                            }
                            _ => false,
                        }
                }
                _ => false,
            };
            let text_is_known = |slot: &FormatMember| match slot {
                FormatMember::Null => true,
                FormatMember::BackReference { key, .. } => {
                    table.registered_class(*key) == Some(TEXT_SYMBOL_CLASS_ID)
                }
                FormatMember::TextSymbol(text) => {
                    [
                        [0; 16],
                        SIMPLE_TEXT_PATH_CLASS_ID,
                        BEZIER_TEXT_PATH_CLASS_ID,
                    ]
                    .contains(&text.raw_path_clsid)
                        && text.tail_path_clsid.is_none_or(|class_id| {
                            class_id == [0; 16] || class_id == SIMPLE_TEXT_PARSER_CLASS_ID
                        })
                        && match &text.background {
                            None | Some(CalloutSlot10_1::Null) => true,
                            Some(CalloutSlot10_1::BackReference { key, .. }) => {
                                matches!(
                                    table.registered_class(*key),
                                    Some(
                                        MARKER_TEXT_BACKGROUND_CLASS_ID
                                            | BALLOON_CALLOUT_CLASS_ID
                                            | LINE_CALLOUT_CLASS_ID
                                            | SIMPLE_LINE_CALLOUT_CLASS_ID
                                    )
                                )
                            }
                            Some(CalloutSlot10_1::Balloon(callout)) => {
                                callout_point_preserves_empty_dataset(
                                    callout.anchor.as_ref(),
                                    table,
                                ) && simple_symbol_preserves_empty_dataset(&callout.symbol, table)
                            }
                            Some(CalloutSlot10_1::Line(callout)) => {
                                callout_point_preserves_empty_dataset(
                                    callout.anchor.as_ref(),
                                    table,
                                ) && callout.symbols.iter().all(|symbol| {
                                    simple_symbol_preserves_empty_dataset(symbol, table)
                                })
                            }
                            Some(CalloutSlot10_1::SimpleLine(callout)) => {
                                callout_point_preserves_empty_dataset(
                                    callout.anchor.as_ref(),
                                    table,
                                ) && match &callout.anchor_geometry {
                                    Some(GeometrySlotEntry10_1::Null) => true,
                                    Some(GeometrySlotEntry10_1::BackReference { key, .. }) => {
                                        matches!(
                                            table.registered_class(*key),
                                            Some(
                                                POINT_CLASS_ID
                                                    | POLYLINE_CLASS_ID
                                                    | POLYGON_CLASS_ID
                                                    | MULTIPOINT_CLASS_ID
                                            )
                                        )
                                    }
                                    Some(GeometrySlotEntry10_1::Body(body)) => {
                                        anchor_geometry_preserves_empty_dataset(body, table)
                                    }
                                    None => false,
                                } && match &callout.line_symbol {
                                    Some(SymbolSlotFull10_1::Null) => true,
                                    Some(symbol @ SymbolSlotFull10_1::SimpleLineSymbol(_)) => {
                                        simple_symbol_preserves_empty_dataset(symbol, table)
                                    }
                                    Some(SymbolSlotFull10_1::BackReference { key, .. }) => {
                                        table.registered_class(*key)
                                            == Some(SIMPLE_LINE_SYMBOL_CLASS_ID)
                                    }
                                    _ => false,
                                }
                            }
                            Some(CalloutSlot10_1::MarkerBackground(background)) => {
                                match &background.symbol {
                                    SymbolSlotFull10_1::BackReference { key, .. } => {
                                        table.registered_class(*key)
                                            == Some(SIMPLE_MARKER_SYMBOL_CLASS_ID)
                                    }
                                    SymbolSlotFull10_1::Null
                                    | SymbolSlotFull10_1::SimpleMarkerSymbol(_) => {
                                        simple_symbol_preserves_empty_dataset(
                                            &background.symbol,
                                            table,
                                        )
                                    }
                                    _ => false,
                                }
                            }
                        }
                        && [&text.color1, &text.color2].into_iter().all(|color| {
                            matches!(color, ColorSlot10_1::Null)
                                || is_known_renderer_color(color, table)
                        })
                        && simple_symbol_preserves_empty_dataset(&text.fill_symbol, table)
                        && simple_symbol_preserves_empty_dataset(&text.symbol2, table)
                        && match text.font {
                            FontSlot10_1::Null | FontSlot10_1::StdFont(_) => true,
                            FontSlot10_1::BackReference { key, .. } => {
                                table.registered_class(key) == Some(STDFONT_CLASS_ID)
                            }
                        }
                }
                _ => false,
            };
            text_is_known(&format.label_symbol)
                && text_is_known(&format.description_symbol)
                && patch_is_known(&format.line_patch, LINE_PATCH_CLASS_ID)
                && patch_is_known(&format.area_patch, AREA_PATCH_CLASS_ID)
        }
        LegendClassFormatSlot10_1::Unsupported { .. } => false,
    }
}

fn legend_groups_preserve_empty_dataset(
    groups: &[LegendGroupSlot10_1],
    table: &RefTable10_1,
) -> bool {
    groups.iter().all(|slot| match slot {
        LegendGroupSlot10_1::Null => true,
        LegendGroupSlot10_1::LegendGroup(group) => group.classes.iter().all(|entry| match entry {
            LegendClassEntry10_1::Null => true,
            LegendClassEntry10_1::BackReference { key, .. } => {
                table.registered_class(*key) == Some(LEGEND_CLASS_CLASS_ID)
            }
            LegendClassEntry10_1::NewObject(class) => {
                simple_symbol_preserves_empty_dataset(&class.symbol, table)
                    && legend_format_preserves_empty_dataset(&class.format, table)
            }
        }),
        LegendGroupSlot10_1::BackReference { key, .. } => {
            table.registered_class(*key) == Some(LEGEND_GROUP_CLASS_ID)
        }
    })
}

fn is_inline_unique_value_scalar(value: &PropertyValueObjectStream10_1) -> bool {
    matches!(
        value,
        PropertyValueObjectStream10_1::Inline(
            PropertyValue10_1::Raw4(_) | PropertyValue10_1::Raw8(_) | PropertyValue10_1::String(_)
        )
    )
}

pub use super::raster_layer_tail::RasterLayerTail10_1;

/// RasterLayer 10.1 Load 的已确认前缀。
#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerPrefix10_1 {
    /// 有符号 int32 版本；原生 `JG` 只拒绝大于 17 的值，负值按原生门通过。
    pub version: i32,
    /// RasterLayer::Load 的首条 String，即 ILayer.Name（完整对象 +0x124）。
    pub first_string: String,
    /// 成功读取第二条名称、标量及名称列表前导后填入。
    pub continuation: Option<RasterLayerContinuation10_1>,
    /// 已完整读取的名称条目；遇到未知对象时保留其之前的条目。
    pub names: Vec<RasterLayerName10_1>,
    pub names_stop: Option<String>,
    /// 仅在 IName 列表结束处读取；新渲染器只消费已确认的前缀。
    pub renderer: Option<Box<crate::layers::raster_renderer::RasterRendererSlot10_1>>,
    /// 已定位到槽位但无法读取时的原因；调用者仍保留未知体。
    pub renderer_stop: Option<String>,
    /// 渲染器完整消费后，继续读取的父图层尾段。
    pub tail: Option<Box<RasterLayerTail10_1>>,
    /// 尾段读失败时的原因；未解析字节仍由外层保留。
    pub tail_stop: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerName10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterDatasetNameEntryBody10_1>,
    /// 已确认的路径、数据集名和波段名。
    pub source_reference: Option<crate::storage::object_refs::DatasetSource10_1>,
    pub suffix: Option<String>,
    /// v5–v8 的 int32 选择子，非零时带原生波段集合的索引。
    pub legacy: Option<(i32, Option<i32>)>,
}

/// RasterLayer 第二条 String、三个定长短值及 `0x7c73b380` 循环首对象头。
///
/// 对象体仍未解析；`first_object_header` 对应原生首个 `LoadObject(IName)`
/// 的空标记或 GUID 与键，不代表该对象已加载。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterLayerContinuation10_1 {
    /// Load 中的第二条 String，原生先读入栈上临时 String；业务含义未定。
    pub second_string: String,
    /// 按原生顺序紧邻的三个 short 原值；第一个是 ILayer.Visible。
    pub fixed_shorts: [u16; 3],
    /// 版本大于 8 时 `0x7c73b380` 读取的有符号数量；其余版本没有该流字段。
    pub object_count: Option<i32>,
    /// `0x7c73b380` 循环首个 `LoadObject` 的对象头；对象体及后续循环均未读取。
    pub first_object_header: Option<ObjectHeader>,
}

/// 从 RasterLayer 对象头之后读取 10.1 已确认前缀。
///
/// 成功后游标停在第二条 String 之前。`max_bytes` 限制首条 String 的声明
/// 长度；任何错误均不推进调用者游标。
pub fn read_raster_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<RasterLayerPrefix10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let version = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if version > 17 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "RasterLayer 对象体 offset 0x{start:x}: 版本 {version} 超出 Load 接受范围（有符号比较 >17 拒绝）"
            ),
        ));
    }
    let first_string = read_string_10_1(&mut trial, max_bytes)?;
    *reader = trial;
    Ok(RasterLayerPrefix10_1 {
        version,
        first_string,
        continuation: None,
        names: Vec::new(),
        names_stop: None,
        renderer: None,
        renderer_stop: None,
        tail: None,
        tail_stop: None,
    })
}

/// 从首条 String 之后继续读 RasterLayer 10.1 的有界前缀。
///
/// 调用者须先成功调用 [`read_raster_layer_prefix_10_1`]，并传入其版本。
/// 成功后最多越过第二条 String、三个 short、版本相关数量和
/// `0x7c73b380` 循环首对象头；该对象体从返回游标处开始。主 Load 在该辅助函数
/// 返回后还会读取其他字段和另一个对象，本函数不推进到那些字段。任何错误都不
/// 推进输入。
pub fn read_raster_layer_continuation_10_1(
    reader: &mut Reader<'_>,
    version: i32,
    max_string_bytes: usize,
) -> io::Result<RasterLayerContinuation10_1> {
    let mut trial = reader.clone();
    let mut result = read_continuation_fields(&mut trial, version, max_string_bytes)?;
    if result.object_count.unwrap_or(1) > 0 {
        result.first_object_header = Some(read_header_10_1(&mut trial)?);
    }
    *reader = trial;
    Ok(result)
}

fn read_continuation_fields(
    reader: &mut Reader<'_>,
    version: i32,
    max_string_bytes: usize,
) -> io::Result<RasterLayerContinuation10_1> {
    let mut trial = reader.clone();
    if version > 17 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("RasterLayer 续段版本 {version} 超出 Load 接受范围（>17 拒绝）"),
        ));
    }
    let second_string = read_string_10_1(&mut trial, max_string_bytes)?;
    let fixed_shorts = [
        trial.read_u16_le()?,
        trial.read_u16_le()?,
        trial.read_u16_le()?,
    ];

    // RasterLayer::Load 0x7c73bd40 调 0x7c73b380；后者仅在版本 >8 时
    // 从流中覆盖初始数量 1。负数量按原生循环条件不进入首对象读取。
    let object_count = if version > 8 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    *reader = trial;
    Ok(RasterLayerContinuation10_1 {
        second_string,
        fixed_shorts,
        object_count,
        first_object_header: None,
    })
}

fn read_raster_layer_names(
    body: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    prefix: &mut RasterLayerPrefix10_1,
    raw_count: i32,
) -> io::Result<()> {
    // 0x7c73b380 没有可用名称时返回失败，主 Load 不进入渲染器。
    let count = usize::try_from(raw_count).unwrap_or_default();
    if count == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterLayer 没有可用名称",
        ));
    }
    let version = prefix.version;
    let legacy = (5..=8).contains(&version);
    let min_size = 16 + usize::from(version > 0) * 6 + usize::from(legacy) * 4;
    if count > max_bytes / min_size || count > body.remaining() / min_size {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterLayer 名称计数超出限额或剩余范围",
        ));
    }
    let mut any_name = false;
    for index in 0..count {
        let mut trial = body.clone();
        let mut staged = refs.clone();
        let result = (|| -> io::Result<RasterLayerName10_1> {
            let (entry, name) =
                read_supported_raster_name_entry_10_1(&mut trial, max_bytes, &mut staged)?;
            let suffix = (version > 0)
                .then(|| read_string_10_1(&mut trial, max_bytes))
                .transpose()?;
            let legacy = legacy
                .then(|| -> io::Result<_> {
                    let selector = trial.read_u32_le()? as i32;
                    let index = (selector != 0)
                        .then(|| trial.read_u32_le().map(|value| value as i32))
                        .transpose()?;
                    Ok((selector, index))
                })
                .transpose()?;
            let source_reference = match &entry {
                ObjectEntry10_1::NewObject { class_id, key }
                | ObjectEntry10_1::BackReference { class_id, key } => {
                    staged.dataset_source(*class_id, *key).cloned()
                }
                _ => None,
            };
            Ok(RasterLayerName10_1 {
                entry,
                body: name,
                source_reference,
                suffix,
                legacy,
            })
        })()
        .map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("RasterLayer 名称条目 {}：{err}", index + 1),
            )
        })?;
        any_name |= result.entry != ObjectEntry10_1::Null;
        prefix.names.push(result);
        *body = trial;
        *refs = staged;
    }
    if !any_name {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterLayer 名称列表全为空对象",
        ));
    }
    Ok(())
}

/// 读取普通 RasterLayer 对象体。子体未完整读取时，不提交对象键或游标。
pub(super) fn read_inline_raster_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    use super::raster_layer_tail::{RasterLayerBoundary, read_raster_layer_tail};

    let mut trial = reader.clone();
    let mut staged = table.clone();
    let mut body = read_raster_layer_prefix_10_1(&mut trial, max_bytes)?;
    let count = read_name_list_prefix(&mut trial, max_bytes, &mut body)?;
    read_raster_layer_names(&mut trial, max_bytes, &mut staged, &mut body, count)?;

    let renderer = read_raster_renderer_slot_prefix_10_1(&mut trial, max_bytes, &mut staged)?;
    if renderer
        .body
        .as_ref()
        .is_some_and(|body| !body.is_complete())
    {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "RasterLayer 渲染器体未完整读取",
        ));
    }
    let first_name = body
        .names
        .iter()
        .find(|name| name.entry != ObjectEntry10_1::Null);
    let has_scalar_renderer = matches!(
        renderer.body.as_ref(),
        Some(
            RasterRendererPrefix10_1::Simple { shared, .. }
            | RasterRendererPrefix10_1::Sai { shared, .. }
        ) if renderer_shared_preserves_empty_dataset(shared, &staged)
    );
    let has_legacy_rgb_renderer = matches!(
        renderer.body.as_ref(),
        Some(RasterRendererPrefix10_1::LegacyRgb(rgb))
            if renderer_shared_preserves_empty_dataset(&rgb.shared, &staged)
                && rgb.colors.iter().all(|color| is_known_renderer_color(color, &staged))
    );
    let has_rgb_renderer = matches!(
        renderer.body.as_ref(),
        Some(RasterRendererPrefix10_1::Rgb { tail: Some(rgb), .. })
            if is_known_renderer_io_color(&rgb.color)
                && match &rgb.pansharpening {
                    None | Some(PansharpeningSlot10_1::Null) => true,
                    Some(PansharpeningSlot10_1::Filter(filter)) => {
                        filter.rasters.iter().all(Option::is_none)
                    }
                }
                && rgb.double_arrays.iter().all(|slot| match slot.entry {
                    ObjectEntry10_1::Null => true,
                    ObjectEntry10_1::NewObject { key, .. }
                    | ObjectEntry10_1::BackReference { key, .. } => {
                        staged.registered_class(key) == Some(DOUBLE_ARRAY_CLASS_ID)
                    }
                })
                && renderer_shared_preserves_empty_dataset(&rgb.shared, &staged)
    );
    let has_colormap_renderer = matches!(
        renderer.body.as_ref(),
        Some(RasterRendererPrefix10_1::Colormap { legend_groups, tail: Some(tail), .. })
            if legend_groups_preserve_empty_dataset(legend_groups, &staged)
                && renderer_shared_preserves_empty_dataset(&tail.shared, &staged)
    );
    let has_discrete_renderer = matches!(
        renderer.body.as_ref(),
        Some(RasterRendererPrefix10_1::Discrete(discrete))
            if color_ramp_preserves_empty_dataset(&discrete.ramp, &staged)
                && renderer_shared_preserves_empty_dataset(&discrete.shared, &staged)
    );
    let has_unique_value_renderer = matches!(
        renderer.body.as_ref(),
        Some(RasterRendererPrefix10_1::UniqueValue(unique))
            if simple_symbol_preserves_empty_dataset(&unique.symbol, &staged)
                && legend_groups_preserve_empty_dataset(&unique.legend_groups, &staged)
                && unique.classes.iter().all(|class| {
                    class.values.iter().all(is_inline_unique_value_scalar)
                })
                && match &unique.layout {
                    RasterUniqueValueLayout10_1::Legacy { legend_item, colors, samples, .. } => {
                        matches!(legend_item, LegendItemSlot10_1::Null)
                            && colors.entry == ObjectEntry10_1::Null
                            && samples.is_empty()
                    }
                    RasterUniqueValueLayout10_1::Modern { unique_values, ramp } => {
                        matches!(unique.version, 3 | 4)
                            && match unique_values.entry {
                                ObjectEntry10_1::Null => true,
                                ObjectEntry10_1::BackReference { class_id, .. } => {
                                    class_id == UNIQUE_VALUES_CLASS_ID
                                }
                                ObjectEntry10_1::NewObject { .. } => {
                                    unique_values.body.as_ref().is_some_and(|body| {
                                        body.values.iter().all(|(value, _)| {
                                            is_inline_unique_value_scalar(value)
                                        })
                                    })
                                }
                            }
                            && ramp.as_ref().is_none_or(|ramp| {
                                color_ramp_preserves_empty_dataset(ramp, &staged)
                            })
                    }
                }
                && renderer_shared_preserves_empty_dataset(&unique.shared, &staged)
    );
    let has_classify_renderer = matches!(
        renderer.body.as_ref(),
        Some(RasterRendererPrefix10_1::Classify(classify))
            if matches!(classify.version, 1..=5)
                && matches!(classify.classification_method, ClassificationMethod10_1::LegacyString(_) | ClassificationMethod10_1::Guid { flag: 0, .. })
                && legend_groups_preserve_empty_dataset(&classify.legend_groups, &staged)
                && matches!(classify.legend_item, LegendItemSlot10_1::Null)
                && classify.numeric_fields.as_ref().is_none_or(|fields| {
                    matches!(fields.format, NumberFormatSlot10_1::Null)
                })
                && classify.value_bounds.as_ref().is_none_or(|bounds| {
                    bounds.values.iter().all(|value| {
                        matches!(value, PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Empty))
                            || is_inline_unique_value_scalar(value)
                    })
                        && bounds.legend_class.as_ref().is_some_and(|slot| {
                            slot.entry == ObjectEntry10_1::Null
                        })
                })
                && classify.unique_values.as_ref().is_none_or(|slot| slot.entry == ObjectEntry10_1::Null)
                && renderer_shared_preserves_empty_dataset(&classify.shared, &staged)
    );
    let has_known_renderer_reference = matches!(
        renderer.entry,
        ObjectEntry10_1::BackReference { key, .. }
            if staged.registered_class(key).is_some_and(supports_raster_renderer_body_10_1)
    );
    let boundary = if body.version == 12
        && (renderer.entry == ObjectEntry10_1::Null
            || has_scalar_renderer
            || has_legacy_rgb_renderer
            || has_rgb_renderer
            || has_colormap_renderer
            || has_discrete_renderer
            || has_unique_value_renderer
            || has_classify_renderer
            || has_known_renderer_reference)
        && let Some(name) = first_name
        && match name.entry {
            ObjectEntry10_1::NewObject { key, .. } | ObjectEntry10_1::BackReference { key, .. } => {
                staged.raster_name_opens_without_result(key)
            }
            ObjectEntry10_1::Null => false,
        } {
        // 首个有效名称的已知 Open 分支未提供数据集。
        RasterLayerBoundary::InlineWithoutDataset
    } else {
        RasterLayerBoundary::Inline
    };
    body.renderer = Some(Box::new(renderer));
    let tail = read_raster_layer_tail(&mut trial, body.version, max_bytes, &mut staged, boundary)?;
    require_loaded_extensions(&tail)?;
    body.tail = Some(Box::new(tail));
    *reader = trial;
    *table = staged;
    Ok(body)
}

fn renderer_shared_preserves_empty_dataset(
    shared: &RasterRendererShared10_1,
    table: &RefTable10_1,
) -> bool {
    match &shared.layout {
        RasterRendererSharedLayout10_1::Legacy(_) => true,
        RasterRendererSharedLayout10_1::RasterIo { color, .. } => {
            is_known_renderer_io_color(color)
                && shared
                    .color_ramp
                    .as_ref()
                    .is_none_or(|ramp| color_ramp_preserves_empty_dataset(ramp, table))
        }
    }
}

fn is_known_renderer_io_color(color: &[u8]) -> bool {
    color.get(..16).is_some_and(|class_id| {
        RASTER_RENDERER_COLOR_CLASSES
            .iter()
            .any(|known| class_id == known)
    })
}

fn is_known_renderer_color(color: &ColorSlot10_1, table: &RefTable10_1) -> bool {
    match color {
        ColorSlot10_1::Color { class_id, .. } => RASTER_RENDERER_COLOR_CLASSES.contains(class_id),
        ColorSlot10_1::BackReference { key, .. } => table
            .registered_class(*key)
            .is_some_and(|class_id| RASTER_RENDERER_COLOR_CLASSES.contains(&class_id)),
        _ => false,
    }
}

fn color_ramp_preserves_empty_dataset(ramp: &ColorRampSlot10_1, table: &RefTable10_1) -> bool {
    match ramp {
        ColorRampSlot10_1::Null | ColorRampSlot10_1::Random(_) => true,
        ColorRampSlot10_1::Preset(ramp) => {
            ramp.raw_int32 <= 0
                && ramp
                    .colors
                    .iter()
                    .all(|color| is_known_renderer_color(color, table))
        }
        ColorRampSlot10_1::Algorithmic(ramp) => ramp.colors.iter().all(|color| match color {
            Some(ColorSlot10_1::Null) => true,
            Some(color) => is_known_renderer_color(color, table),
            None => false,
        }),
        ColorRampSlot10_1::BackReference { key, .. } => table
            .registered_class(*key)
            .is_some_and(is_supported_color_ramp_class),
        ColorRampSlot10_1::MultiPart(ramp) => ramp
            .ramps
            .iter()
            .all(|child| color_ramp_preserves_empty_dataset(child, table)),
    }
}

fn require_loaded_extensions(tail: &RasterLayerTail10_1) -> io::Result<()> {
    // 有界扩展可保留原始字节。普通图层须确认内部对象键已经读取。
    if tail.legacy_extensions.iter().any(|entry| {
        !entry.raw_body.is_empty()
            && entry.properties.is_none()
            && entry.edit_template_manager.is_none()
            && entry.server_layer_extension.is_none()
            && entry.representation_layer_extension_version.is_none()
            && entry.route_layer_extension.is_none()
            && entry.hatch_layer_extension.is_none()
            && entry.tracking_filter_layer_extension.is_none()
            && entry.e2_layer_extension_values.is_none()
            && entry.feature_service_layer_edit_extension.is_none()
            && entry.globe_layer_properties.is_none()
            && entry.time_layer_track_extension.is_none()
            && entry.dynamic_tiled_layer_properties.is_none()
            && entry.scene_layer_properties.is_none()
    }) || tail.extra_safe_extensions.iter().any(|entry| {
        matches!(entry.entry, ObjectEntry10_1::NewObject { .. })
            && entry.properties.is_none()
            && entry.edit_template_manager.is_none()
            && entry.server_layer_extension.is_none()
            && entry.representation_layer_extension_version.is_none()
            && entry.route_layer_extension.is_none()
            && entry.hatch_layer_extension.is_none()
            && entry.tracking_filter_layer_extension.is_none()
            && entry.e2_layer_extension_values.is_none()
            && entry.feature_service_layer_edit_extension.is_none()
            && entry.globe_layer_properties.is_none()
            && entry.time_layer_track_extension.is_none()
            && entry.dynamic_tiled_layer_properties.is_none()
            && entry.scene_layer_properties.is_none()
    }) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "RasterLayer 含有未完整读取的扩展对象",
        ));
    }
    Ok(())
}

/// 已界定的子层仍须完整消费，普通父对象才能依赖其引用表。
pub(super) fn read_complete_bounded_raster_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let body = read_bounded_raster_body_10_1(&mut trial, max_bytes, &mut staged)?;
    let tail = body
        .tail
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::Unsupported, "有界 RasterLayer 子层未完整读取"))?;
    require_loaded_extensions(tail)?;
    *reader = trial;
    *table = staged;
    Ok(body)
}

fn read_name_list_prefix(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    body: &mut RasterLayerPrefix10_1,
) -> io::Result<i32> {
    let mut continuation = read_continuation_fields(reader, body.version, max_bytes)?;
    let count = continuation.object_count.unwrap_or(1);
    if count > 0 {
        continuation.first_object_header = read_header_10_1(&mut reader.clone()).ok();
    }
    body.continuation = Some(continuation);
    Ok(count)
}

/// 只接受已由 ExtraSafe 界定的图层体。未知名称不读取体后的流段。
pub(super) fn read_bounded_raster_body_10_1(
    body: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    let mut prefix = read_raster_layer_prefix_10_1(body, max_bytes)?;
    let Ok(count) = read_name_list_prefix(body, max_bytes, &mut prefix) else {
        return Ok(prefix);
    };
    if let Err(error) = read_raster_layer_names(body, max_bytes, refs, &mut prefix, count) {
        prefix.names_stop = Some(error.to_string());
        return Ok(prefix);
    }
    match read_raster_renderer_slot_prefix_10_1(body, max_bytes, refs) {
        Ok(renderer) => prefix.renderer = Some(Box::new(renderer)),
        Err(error) => prefix.renderer_stop = Some(error.to_string()),
    }
    if prefix
        .renderer
        .as_ref()
        .is_some_and(|slot| slot.body.as_ref().is_none_or(|body| body.is_complete()))
    {
        match super::raster_layer_tail::read_raster_layer_tail_10_1(
            body,
            prefix.version,
            max_bytes,
            refs,
        ) {
            Ok(tail) => prefix.tail = Some(Box::new(tail)),
            Err(error) => prefix.tail_stop = Some(error.to_string()),
        }
    }
    Ok(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_string(text: &str) -> Vec<u8> {
        let payload: Vec<_> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
        bytes.extend(payload);
        bytes
    }

    #[test]
    fn reads_confirmed_raster_layer_prefix_and_stops_before_next_field() {
        let mut bytes = 17_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("Raster title"));
        let prefix_len = bytes.len();
        bytes.extend(encoded_string("unparsed second string"));

        let mut reader = Reader::new(&bytes);
        let parsed = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.version, 17);
        assert_eq!(parsed.first_string, "Raster title");
        assert_eq!(reader.position(), prefix_len);
    }

    #[test]
    fn accepts_negative_version_bits_as_native_signed_gate_does() {
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        bytes.extend(encoded_string("legacy"));
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.version, -1);
        assert_eq!(parsed.first_string, "legacy");
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn rejects_version_above_native_limit_without_advancing() {
        let bytes = 18_i32.to_le_bytes();
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn every_truncated_prefix_rolls_back() {
        let mut bytes = 4_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("landsat"));

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(
                read_raster_layer_prefix_10_1(&mut reader, 0x1000).is_err(),
                "prefix length {end} must be rejected"
            );
            assert_eq!(reader.position(), 0, "prefix length {end} must roll back");
        }
    }

    #[test]
    fn oversized_first_string_is_rejected_without_advancing() {
        let mut bytes = 1_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("too long"));
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_prefix_10_1(&mut reader, 4).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_second_string_fixed_shorts_count_and_only_first_object_header() {
        let mut bytes = encoded_string("unbound second string");
        bytes.extend([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        bytes.extend(1_i32.to_le_bytes());
        let header_start = bytes.len();
        bytes.extend([0x7a; 16]);
        bytes.extend(0x1234_u32.to_le_bytes());
        bytes.extend([0xaa, 0xbb]);

        let mut reader = Reader::new(&bytes);
        let parsed = read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).unwrap();

        assert_eq!(parsed.second_string, "unbound second string");
        assert_eq!(parsed.fixed_shorts, [0x2211, 0x4433, 0x6655]);
        assert_eq!(parsed.object_count, Some(1));
        assert_eq!(
            parsed.first_object_header,
            Some(ObjectHeader::Object {
                class_id: [0x7a; 16],
                key: 0x1234,
            })
        );
        assert_eq!(reader.position(), header_start + 20);
        assert_eq!(reader.read_bytes(2).unwrap(), &[0xaa, 0xbb]);
    }

    #[test]
    fn pre_v9_continuation_has_no_stream_count_before_first_object() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend([0x5b; 16]);
        bytes.extend(9_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_continuation_10_1(&mut reader, 8, 0x1000).unwrap();

        assert_eq!(parsed.object_count, None);
        assert_eq!(
            parsed.first_object_header,
            Some(ObjectHeader::Object {
                class_id: [0x5b; 16],
                key: 9,
            })
        );
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn zero_count_stops_before_any_nested_object() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0x5b; 20]);
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).unwrap();

        assert_eq!(parsed.object_count, Some(0));
        assert_eq!(parsed.first_object_header, None);
        assert_eq!(reader.position(), bytes.len() - 20);
    }

    #[test]
    fn continuation_rejects_version_above_native_limit_without_advancing() {
        let bytes = encoded_string("second");
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_continuation_10_1(&mut reader, 18, 0x1000).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn truncated_continuation_rolls_back_at_every_boundary() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0x5b; 20]);

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).is_err());
            assert_eq!(reader.position(), 0, "截断位置 {end} 应回滚");
        }
    }

    fn body_prefix(version: i32, count: i32) -> Vec<u8> {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded_string("栅格图层"));
        bytes.extend(encoded_string(""));
        bytes.extend([0; 6]);
        if version >= 9 {
            bytes.extend(count.to_le_bytes());
        }
        bytes
    }

    fn dataset_name(bytes: &mut Vec<u8>, key: u32) {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(key.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        for text in ["data", "", "", ""] {
            bytes.extend(encoded_string(text));
        }
        bytes.extend([0; 16]); // 工作空间。
        bytes.extend([0; 8]); // v4 两个 int32。
    }

    fn raster_with_empty_slots(version: i32, transform_pair: bool) -> Vec<u8> {
        let mut bytes = body_prefix(version, 1);
        dataset_name(&mut bytes, 60);
        if version > 0 {
            bytes.extend(encoded_string(""));
        }
        if (5..=8).contains(&version) {
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes.extend([0; 16]); // 空渲染器。
        bytes.extend([0; 22]); // 两个 double、int32、short。
        for (first, size) in [(2, 4), (3, 4), (4, 8), (5, 32), (6, 8), (7, 16)] {
            if version >= first {
                bytes.resize(bytes.len() + size, 0);
            }
        }
        if version >= 10 {
            bytes.extend(encoded_string("尾段"));
        }
        if version == 10 {
            bytes.extend(10_u16.to_le_bytes());
        }
        if version >= 11 {
            bytes.extend([0; 16]);
        }
        if version >= 13 || (version == 12 && transform_pair) {
            bytes.extend([0; 32]);
        }
        if version >= 14 {
            bytes.extend([0; 16 + 4]);
        }
        if version >= 15 {
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend([0; 20]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 26]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded_string(""));
            bytes.extend(encoded_string(""));
        }
        if version >= 16 {
            bytes.extend([0; 16]);
        }
        if version >= 17 {
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn layer_list_loads_wcs_wrappers_and_keeps_nested_raster_keys() {
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_layer_list_10_1,
        };
        use crate::layers::wcs_layer::WCS_LAYER_CLASS_ID;
        use crate::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};

        for (version, raster, flag) in [(15_i32, 0, 1_u16), (16, 12, 2), (17, 17, 3)] {
            let mut body = version.to_le_bytes().to_vec();
            for value in ["覆盖", "地址", "标识"] {
                body.extend(encoded_string(value));
            }
            body.extend(14_i32.to_le_bytes());
            body.extend(15_i32.to_le_bytes());
            body.extend(encoded_string("额外字段"));
            if version >= 16 {
                body.extend(encoded_string("选项"));
                body.extend([0; 16]); // Variant::Write(false) 的空标量。
            }
            if version >= 17 {
                body.extend(encoded_string("扩展字符串"));
            }
            body.extend(RASTER_LAYER_CLASS_ID);
            body.extend(52_u32.to_le_bytes());
            body.extend(raster_with_empty_slots(raster, raster == 12));

            if raster == 12 {
                let mut unbounded = body.clone();
                unbounded.extend([0xa5; 16]); // 外层流还有后继字段。
                let mut reader = Reader::new(&unbounded);
                let mut refs = RefTable10_1::new();
                refs.register(5);
                let before = refs.clone();
                let error = crate::layers::wcs_layer::read_wcs_layer_body_10_1(
                    &mut reader,
                    4096,
                    &mut refs,
                    false,
                )
                .unwrap_err();
                assert_eq!(error.kind(), io::ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
                assert_eq!(refs, before);
            }

            let mut truncated = Reader::new(&body[..body.len() - 1]);
            let mut refs = RefTable10_1::new();
            refs.register(5);
            assert!(
                crate::layers::wcs_layer::read_wcs_layer_body_10_1(
                    &mut truncated,
                    4096,
                    &mut refs,
                    true,
                )
                .is_err()
            );
            assert_eq!(truncated.position(), 0);
            assert!(refs.contains(5));
            assert_eq!(refs.len(), 1);

            let mut bytes = 2_i32.to_le_bytes().to_vec();
            bytes.extend(flag.to_le_bytes());
            if flag != 1 {
                bytes.extend((20 + body.len() as u32).to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
            }
            bytes.extend(WCS_LAYER_CLASS_ID);
            bytes.extend(51_u32.to_le_bytes());
            bytes.extend(body);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(RASTER_LAYER_CLASS_ID);
            bytes.extend(52_u32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.push(0x7f);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(LayerBodyPrefix10_1::WcsLayer(layer)) = &parsed.entries[0].parsed else {
                panic!("{:?}", parsed.entries[0].parse_note);
            };
            assert_eq!(layer.version, version);
            assert_eq!(layer.v14_longs, Some([14, 15]));
            assert_eq!(layer.v15_string.as_deref(), Some("额外字段"));
            if version >= 16 {
                assert!(matches!(
                    &layer.v16_fields,
                    Some((value, PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Empty)))
                        if value == "选项"
                ));
            } else {
                assert!(layer.v16_fields.is_none());
            }
            assert_eq!(
                layer.v17_string.as_deref(),
                (version == 17).then_some("扩展字符串")
            );
            assert_eq!(layer.raster.body.as_ref().unwrap().version, raster);
            assert!(matches!(
                parsed.entries[1].object,
                crate::layers::layer_list::LayerEntryObject::Entry(
                    ObjectEntry10_1::BackReference { key: 52, .. }
                )
            ));
            assert_eq!(parsed.active_index, Some(1));
            assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
            assert!(refs.contains(51) && refs.contains(52) && refs.contains(60));
        }
    }

    #[test]
    fn ordinary_raster_self_delimiting_versions_leave_the_next_object_unread() {
        for version in 0..=17 {
            if version == 12 {
                continue;
            }
            let mut bytes = raster_with_empty_slots(version, false);
            let end = bytes.len();
            bytes.extend([0x53; 20]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(parsed.version, version);
            assert_eq!(
                parsed.tail.unwrap().geodata_xform_v13.is_some(),
                version >= 13
            );
            assert!(refs.contains(60));
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(20).unwrap(), [0x53; 20]);

            for cut in 0..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err(),
                    "版本 {version}，截断位置 {cut}"
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn inline_raster_v12_accepts_internet_tile_name_without_open_result() {
        use super::super::raster_dataset_name::INTERNET_TILED_LAYER_NAME_CLASS_ID;

        let mut standard_prefix = body_prefix(12, 1);
        dataset_name(&mut standard_prefix, 60);
        let standard = raster_with_empty_slots(12, false);

        let mut bytes = body_prefix(12, 1);
        bytes.extend(INTERNET_TILED_LAYER_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(encoded_string("Tiles"));
        let index_offset = bytes.len();
        bytes.extend((-1_i32).to_le_bytes());
        let tile_offset = bytes.len();
        bytes.extend([
            0x16, 0xec, 0xb2, 0x48, 0xdb, 0xb0, 0x85, 0x4d, 0x9a, 0x86, 0x1d, 0xdb, 0x04, 0x04,
            0x85, 0x79,
        ]); // MapTileReader。
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string(""));
        bytes.extend(encoded_string(""));
        bytes.extend([0; 4]);
        bytes.extend(encoded_string("tiles"));
        bytes.extend([0; 16 + 2]); // 空栅格名称和选择子。
        let name_end = bytes.len();
        bytes.extend(&standard[standard_prefix.len()..]);
        let end = bytes.len();
        bytes.extend([0xa5; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(raster.tail.unwrap().geodata_xform_v13.is_none());
        assert_eq!(reader.position(), end);
        assert!(refs.raster_name_opens_without_result(60));

        let name_body = body_prefix(12, 1).len() + 20;
        let mut back_reference = bytes.clone();
        back_reference.drain(name_body..name_end);
        let mut reader = Reader::new(&back_reference);
        read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(reader.position(), end - (name_end - name_body));

        bytes[index_offset..index_offset + 4].copy_from_slice(&0_i32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());

        bytes.splice(tile_offset..name_end, [0; 16]);
        let mut reader = Reader::new(&bytes);
        read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(reader.position(), end - (name_end - tile_offset) + 16);
    }

    #[test]
    fn inline_raster_v12_ends_without_transform_after_known_open_failure() {
        use super::super::raster_dataset_name::RASTER_TYPE_NAME_CLASS_ID;
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, LayerEntryObject, RASTER_LAYER_CLASS_ID, read_layer_list_10_1,
        };

        let suffix_offset = {
            let mut prefix = body_prefix(12, 1);
            dataset_name(&mut prefix, 60);
            prefix.len()
        };
        let standard = raster_with_empty_slots(12, false);
        let mut bytes = body_prefix(12, 1);
        bytes.extend(RASTER_TYPE_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string(""));
        bytes.extend(encoded_string(""));
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(&standard[suffix_offset..]);
        let end = bytes.len();
        bytes.extend([0xa5; 16]);
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let tail = layer.tail.unwrap();
        assert!(tail.geodata_xform_v13.is_none());
        assert!(tail.envelope_v13.is_none());
        assert_eq!(reader.position(), end);
        assert_eq!(refs.registered_class(60), Some(RASTER_TYPE_NAME_CLASS_ID));

        let mut simple = super::super::raster_renderer::SIMPLE_RASTER_RENDERER_CLASS_ID.to_vec();
        simple.extend(65_u32.to_le_bytes());
        simple.extend(1_i32.to_le_bytes()); // 渲染器版本。
        simple.extend(1_i32.to_le_bytes()); // 共用体版本。
        simple.extend([0; 16]);
        let renderer_at = end - (standard.len() - suffix_offset) + encoded_string("").len();
        let mut sai = super::super::raster_renderer::SAI_RENDERER_CLASS_ID.to_vec();
        sai.extend(65_u32.to_le_bytes());
        sai.extend(1_i32.to_le_bytes());
        sai.extend([0; 2 + 16]); // short 和两个 double。
        sai.extend(1_i32.to_le_bytes());
        sai.extend([0; 16]);
        let mut simple_raster_io = simple[..24].to_vec();
        simple_raster_io.extend(8_i32.to_le_bytes());
        simple_raster_io.extend(0x00ff_00ff_i32.to_le_bytes());
        simple_raster_io.extend(0_i32.to_le_bytes());
        for tag in 1_i32..=5 {
            simple_raster_io.extend(tag.to_le_bytes());
            simple_raster_io.extend(4_i32.to_le_bytes());
            simple_raster_io.extend(0_i32.to_le_bytes());
        }
        simple_raster_io.extend(6_i32.to_le_bytes());
        simple_raster_io.extend((-1_i32).to_le_bytes());
        let color_at = simple_raster_io.len();
        simple_raster_io.extend([0; 16]); // 空颜色。
        simple_raster_io.extend(7_i32.to_le_bytes());
        simple_raster_io.extend(24_i32.to_le_bytes());
        simple_raster_io.extend([0; 24]);
        simple_raster_io.extend([0xff; 8]);
        simple_raster_io.extend([0; 2 + 4 + 4 + 1 + 2]); // v4–v8 的标量后缀。
        let mut rgb = RGB_COLOR_CLASS_ID.to_vec();
        rgb.extend([1, 0, 1, 0, 1]); // 两个版本与整数颜色分支。
        rgb.extend([0; 6]); // 颜色 LONG、标志和透明度。
        let mut colored_v8 = simple_raster_io.clone();
        colored_v8.splice(color_at..color_at + 16, rgb);
        let mut colored_v9 = colored_v8.clone();
        colored_v9[24..28].copy_from_slice(&9_i32.to_le_bytes());
        colored_v9.extend([0; 16]); // 空色带。
        use crate::objects::color_ramp::RANDOM_COLOR_RAMP_CLASS_ID;
        let mut random_ramp = colored_v9[..colored_v9.len() - 16].to_vec();
        random_ramp.extend(RANDOM_COLOR_RAMP_CLASS_ID);
        random_ramp.extend(66_u32.to_le_bytes());
        random_ramp.extend(1_u16.to_le_bytes());
        random_ramp.extend(0_u32.to_le_bytes()); // 空 BSTR。
        random_ramp.extend(7_i32.to_le_bytes());
        random_ramp.extend(1_u16.to_le_bytes());
        random_ramp.extend(42_i32.to_le_bytes());
        for value in [10_u16, 200, 20, 210, 30, 220] {
            random_ramp.extend(value.to_le_bytes());
        }
        let mut preset_ramp = colored_v9[..colored_v9.len() - 16].to_vec();
        preset_ramp.extend(crate::objects::color_ramp::PRESET_COLOR_RAMP_CLASS_ID);
        preset_ramp.extend(67_u32.to_le_bytes());
        preset_ramp.extend(1_u16.to_le_bytes());
        preset_ramp.extend(0_u32.to_le_bytes()); // 空 BSTR。
        preset_ramp.extend(0_i32.to_le_bytes()); // 请求尺寸。
        preset_ramp.extend(6_i32.to_le_bytes()); // 五条新颜色和一条回引。
        for (key, class_id) in (68_u32..).zip(RASTER_RENDERER_COLOR_CLASSES) {
            preset_ramp.extend(class_id);
            preset_ramp.extend(key.to_le_bytes());
            preset_ramp.extend([1, 0, 1, 0, 1]);
            preset_ramp.extend([0; 6]);
        }
        preset_ramp.extend(RGB_COLOR_CLASS_ID);
        preset_ramp.extend(72_u32.to_le_bytes());
        let mut algorithmic_ramp = colored_v9[..colored_v9.len() - 16].to_vec();
        algorithmic_ramp.extend(crate::objects::color_ramp::ALGORITHMIC_COLOR_RAMP_CLASS_ID);
        algorithmic_ramp.extend(67_u32.to_le_bytes());
        algorithmic_ramp.extend(1_u16.to_le_bytes());
        algorithmic_ramp.extend(0_u32.to_le_bytes()); // 空 BSTR。
        algorithmic_ramp.extend(0_i32.to_le_bytes());
        let endpoint_offset = algorithmic_ramp.len();
        algorithmic_ramp.extend(RGB_COLOR_CLASS_ID);
        algorithmic_ramp.extend(68_u32.to_le_bytes());
        algorithmic_ramp.extend([1, 0, 1, 0, 1]);
        algorithmic_ramp.extend([0; 6]);
        algorithmic_ramp.extend(GRAY_COLOR_CLASS_ID);
        algorithmic_ramp.extend(68_u32.to_le_bytes());
        let mut empty_algorithmic_ramp = algorithmic_ramp[..endpoint_offset].to_vec();
        empty_algorithmic_ramp.extend([0; 32]);
        let mut multipart_ramp = colored_v9[..colored_v9.len() - 16].to_vec();
        multipart_ramp.extend(crate::objects::color_ramp::MULTI_PART_COLOR_RAMP_CLASS_ID);
        multipart_ramp.extend(69_u32.to_le_bytes());
        multipart_ramp.extend(2_u16.to_le_bytes());
        multipart_ramp.extend(0_u32.to_le_bytes()); // 空 BSTR。
        multipart_ramp.extend(2_i32.to_le_bytes());
        multipart_ramp.extend(&random_ramp[colored_v9.len() - 16..]);
        multipart_ramp.extend(&algorithmic_ramp[colored_v9.len() - 16..]);
        multipart_ramp.extend(0.25_f64.to_le_bytes());
        multipart_ramp.extend(0.75_f64.to_le_bytes());
        let mut legacy_rgb = super::super::raster_renderer::RASTER_RGB_RENDERER_CLASS_ID.to_vec();
        legacy_rgb.extend(65_u32.to_le_bytes());
        legacy_rgb.extend(3_i32.to_le_bytes());
        legacy_rgb.extend([0; 12 + 0x930 + 1 + 4 + 8 + 8 + 64]);
        legacy_rgb.extend(RGB_COLOR_CLASS_ID);
        legacy_rgb.extend(73_u32.to_le_bytes());
        legacy_rgb.extend([1, 0, 1, 0, 1]);
        legacy_rgb.extend([0; 6]);
        legacy_rgb.extend(RGB_COLOR_CLASS_ID);
        legacy_rgb.extend(73_u32.to_le_bytes());
        let shared_at = legacy_rgb.len();
        legacy_rgb.extend(1_i32.to_le_bytes());
        legacy_rgb.extend([0; 16 + 8]); // 共用标量与旧 RGB 版本后缀。
        let mut legacy_rgb_with_ramp = legacy_rgb[..shared_at].to_vec();
        legacy_rgb_with_ramp.extend(&multipart_ramp[24..]);
        legacy_rgb_with_ramp.extend([0; 8]); // RGB v2/v3 后缀。
        let mut tagged_rgb = legacy_rgb[..20].to_vec();
        tagged_rgb.extend(4_i32.to_le_bytes());
        tagged_rgb.extend(0x00ff_00ff_i32.to_le_bytes());
        tagged_rgb.extend(0_i32.to_le_bytes());
        for (tag, width) in (1_i32..=11).zip([4_i32, 4, 4, 0x930, 1, 4, 8, 4, 4, 32, 2]) {
            tagged_rgb.extend(tag.to_le_bytes());
            tagged_rgb.extend(width.to_le_bytes());
            tagged_rgb.resize(tagged_rgb.len() + width as usize, 0);
        }
        tagged_rgb.extend(12_i32.to_le_bytes());
        tagged_rgb.extend((-1_i32).to_le_bytes());
        tagged_rgb.extend(&colored_v8[color_at..color_at + 27]);
        tagged_rgb.extend([0xff; 8]); // RGB RasterIO 结束。
        tagged_rgb.extend(&simple[24..]); // 共用体 v1。
        let tagged_tail_at = tagged_rgb.len() - 8 - (simple.len() - 24);
        let mut tagged_rgb_with_arrays = tagged_rgb[..tagged_tail_at].to_vec();
        tagged_rgb_with_arrays[20..24].copy_from_slice(&9_i32.to_le_bytes());
        tagged_rgb_with_arrays.extend((-1_i32).to_le_bytes());
        tagged_rgb_with_arrays.extend(0_u16.to_le_bytes());
        let arrays_at = tagged_rgb_with_arrays.len();
        tagged_rgb_with_arrays.extend([0; 16 * 4]);
        tagged_rgb_with_arrays.extend(&tagged_rgb[tagged_tail_at..]);
        let mut double_array = DOUBLE_ARRAY_CLASS_ID.to_vec();
        double_array.extend(74_u32.to_le_bytes());
        double_array.extend(1_u16.to_le_bytes());
        double_array.extend(3_i32.to_le_bytes());
        for number in [1.0_f64, 2.0, 3.0] {
            double_array.extend(number.to_le_bytes());
        }
        double_array.extend(RGB_COLOR_CLASS_ID);
        double_array.extend(74_u32.to_le_bytes());
        let mut tagged_rgb_with_array_reference = tagged_rgb_with_arrays.clone();
        tagged_rgb_with_array_reference.splice(arrays_at..arrays_at + 32, double_array);
        let mut histogram = 14_i32.to_le_bytes().to_vec();
        histogram.extend((-1_i32).to_le_bytes());
        histogram.extend(super::super::raster_renderer::STATS_HISTOGRAM_CLASS_ID);
        histogram.extend(3_i32.to_le_bytes());
        histogram.extend([0; 6 * 8]);
        histogram.extend(1_i32.to_le_bytes());
        histogram.extend(5.0_f64.to_le_bytes());
        histogram.extend(0.0_f64.to_le_bytes());
        let mut tagged_rgb_with_histogram = tagged_rgb_with_array_reference.clone();
        tagged_rgb_with_histogram.splice(tagged_tail_at..tagged_tail_at, histogram);
        let mut filter = 18_i32.to_le_bytes().to_vec();
        filter.extend((-1_i32).to_le_bytes());
        filter.extend(super::super::raster_renderer::PANSHARPENING_FILTER_CLASS_ID);
        filter.extend(1_i32.to_le_bytes());
        filter.extend([0; 32]); // 两个 LoadRaster 空槽。
        filter.extend(3_i32.to_le_bytes());
        filter.extend([0; 32]); // 权重。
        let mut tagged_rgb_with_filter = tagged_rgb.clone();
        tagged_rgb_with_filter.splice(tagged_tail_at..tagged_tail_at, filter);
        let mut colormap =
            super::super::raster_renderer::RASTER_COLORMAP_RENDERER_CLASS_ID.to_vec();
        colormap.extend(65_u32.to_le_bytes());
        colormap.extend(3_i32.to_le_bytes());
        colormap.extend(0_i32.to_le_bytes()); // 无图例组。
        colormap.extend(2_i32.to_le_bytes());
        colormap.extend(0x010203_i32.to_le_bytes());
        colormap.extend(0x040506_i32.to_le_bytes());
        colormap.extend(&colored_v9[24..]);
        colormap.extend([0; 12]); // 色表 v2/v3 后缀。
        let mut groups = 3_i32.to_le_bytes().to_vec();
        groups.extend([0; 16]);
        groups.extend(LEGEND_GROUP_CLASS_ID);
        groups.extend(75_u32.to_le_bytes());
        groups.extend(1_u16.to_le_bytes());
        groups.extend(0_i16.to_le_bytes());
        groups.extend(encoded_string(""));
        groups.extend(3_i32.to_le_bytes());
        groups.extend([0; 16]); // 空图例类别槽。
        groups.extend(LEGEND_CLASS_CLASS_ID);
        groups.extend(76_u32.to_le_bytes());
        groups.extend(1_u16.to_le_bytes());
        groups.extend(SIMPLE_MARKER_SYMBOL_CLASS_ID);
        groups.extend(77_u32.to_le_bytes());
        groups.extend(1_u16.to_le_bytes());
        groups.extend([0; 16]); // 空主色。
        groups.extend([0; 53]); // 标量、角度、偏移和轮廓宽度。
        groups.extend([0; 16]); // 空轮廓色。
        groups.extend(encoded_string("类别"));
        groups.extend(encoded_string(""));
        groups.extend(LEGEND_CLASS_FORMAT_CLASS_ID);
        groups.extend(80_u32.to_le_bytes());
        groups.extend(2_u16.to_le_bytes());
        groups.extend(TEXT_SYMBOL_CLASS_ID);
        groups.extend(96_u32.to_le_bytes());
        groups.extend(4_u16.to_le_bytes());
        groups.extend([0; 16 + 12 + 2 + 24 + 8 + 16 + 16 + 8 + 24 + 1 + 8 + 4 + 8 + 1]);
        groups.extend(SIMPLE_TEXT_PATH_CLASS_ID);
        let background_at = groups.len();
        groups.extend(MARKER_TEXT_BACKGROUND_CLASS_ID);
        groups.extend(97_u32.to_le_bytes());
        groups.extend(1_u16.to_le_bytes());
        groups.extend(0_u16.to_le_bytes());
        groups.extend(RGB_COLOR_CLASS_ID); // 标记符号使用既有键。
        groups.extend(77_u32.to_le_bytes());
        let background_end = groups.len();
        groups.extend([0; 16]); // 空填充符号。
        groups.extend(encoded_string("标签"));
        groups.extend([0; 8 + 4 + 8 + 16 + 16]); // 大小、遮罩参数、符号及字体。
        groups.extend(0_u16.to_le_bytes());
        groups.extend(SIMPLE_TEXT_PARSER_CLASS_ID);
        groups.extend(0_u16.to_le_bytes());
        groups.extend(RGB_COLOR_CLASS_ID);
        groups.extend(96_u32.to_le_bytes()); // 描述符号回引。
        let mut patch_geometries = Vec::new();
        for (class_id, key) in [(LINE_PATCH_CLASS_ID, 81_u32), (AREA_PATCH_CLASS_ID, 82)] {
            groups.extend(class_id);
            groups.extend(key.to_le_bytes());
            groups.extend(3_u16.to_le_bytes());
            let (geometry_class, kind, points) = if class_id == LINE_PATCH_CLASS_ID {
                (POLYLINE_CLASS_ID, 3_u32, &[[0.0_f64, 0.0], [1.0, 1.0]][..])
            } else {
                (
                    POLYGON_CLASS_ID,
                    5_u32,
                    &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]][..],
                )
            };
            let geometry_at = groups.len();
            groups.extend(geometry_class);
            groups.extend((key + 10).to_le_bytes());
            groups.extend(2_u16.to_le_bytes());
            groups.extend((48 + points.len() as u32 * 16).to_le_bytes());
            groups.extend(kind.to_le_bytes());
            for coordinate in [0.0_f64, 0.0, 1.0, 1.0] {
                groups.extend(coordinate.to_le_bytes());
            }
            groups.extend(1_u32.to_le_bytes()); // 一个路径或环。
            groups.extend((points.len() as u32).to_le_bytes());
            groups.extend(0_u32.to_le_bytes()); // 起点索引。
            for [x, y] in points {
                groups.extend(x.to_le_bytes());
                groups.extend(y.to_le_bytes());
            }
            if geometry_class == POLYGON_CLASS_ID {
                groups.extend(RGB_COLOR_CLASS_ID); // 回引头按既有键识别。
                groups.extend(94_u32.to_le_bytes());
            } else {
                groups.extend(UNKNOWN_COORDINATE_SYSTEM_CLASS_ID);
                groups.extend(95_u32.to_le_bytes());
                groups.extend(1_u16.to_le_bytes());
                for value in ["Unknown", "", "", ""] {
                    groups.extend(encoded_string(value));
                }
                groups.extend([0; 7 * 8]); // v1 精度参数。
            }
            let mut anchor = groups[geometry_at..].to_vec();
            anchor[16..20].copy_from_slice(&(key + 20).to_le_bytes());
            if geometry_class == POLYLINE_CLASS_ID {
                let reference_at = 20 + 6 + 48 + points.len() * 16;
                anchor[reference_at + 16..reference_at + 20]
                    .copy_from_slice(&105_u32.to_le_bytes());
            }
            patch_geometries.push(anchor);
            groups.extend(encoded_string("补丁"));
            groups.extend(0_u16.to_le_bytes());
        }
        groups.extend([0; 16]); // 补丁宽高。
        groups.extend(RGB_COLOR_CLASS_ID);
        groups.extend(76_u32.to_le_bytes());
        groups.extend(RGB_COLOR_CLASS_ID);
        groups.extend(75_u32.to_le_bytes());
        let mut colormap_with_groups = colormap.clone();
        colormap_with_groups.splice(24..28, groups.iter().copied());
        let colormaps_with_callouts = [
            (
                BALLOON_CALLOUT_CLASS_ID,
                4 + 4 * 8 + 2 * 16 + 8,
                4 + 4 * 8 + 16,
            ),
            (LINE_CALLOUT_CLASS_ID, 4 * 16 + 8 + 4 + 5 * 8, 3 * 16),
            (SIMPLE_LINE_CALLOUT_CLASS_ID, 8 + 2 + 3 * 16, 8 + 2),
        ]
        .map(|(class_id, body_len, point_offset)| {
            let mut background = class_id.to_vec();
            background.extend(97_u32.to_le_bytes());
            background.extend(1_u16.to_le_bytes());
            let point_at = background.len() + point_offset;
            background.resize(background.len() + body_len, 0);
            if class_id == SIMPLE_LINE_CALLOUT_CLASS_ID {
                let point_slot = point_at + 32;
                let mut point_ref = RGB_COLOR_CLASS_ID.to_vec();
                point_ref.extend(98_u32.to_le_bytes());
                background.splice(point_slot..point_slot + 16, point_ref);
            }
            let mut point = POINT_CLASS_ID.to_vec();
            point.extend(98_u32.to_le_bytes());
            point.extend(2_u16.to_le_bytes());
            point.extend(20_u32.to_le_bytes());
            point.extend(1_u32.to_le_bytes());
            point.extend(1.0_f64.to_le_bytes());
            point.extend(2.0_f64.to_le_bytes());
            point.extend([0; 16]); // 空空间参考。
            background.splice(point_at..point_at + 16, point);
            let mut groups = groups.clone();
            groups.splice(background_at..background_end, background);
            let mut renderer = colormap.clone();
            renderer.splice(24..28, groups);
            renderer
        });
        let mut multipoint = MULTIPOINT_CLASS_ID.to_vec();
        multipoint.extend(103_u32.to_le_bytes());
        multipoint.extend(2_u16.to_le_bytes());
        multipoint.extend(20_u32.to_le_bytes());
        multipoint.extend(1_u32.to_le_bytes());
        multipoint.extend(1.0_f64.to_le_bytes());
        multipoint.extend(2.0_f64.to_le_bytes());
        multipoint.extend([0; 16]); // 空空间参考。
        patch_geometries.push(multipoint);
        let colormaps_with_shape_anchors: Vec<_> = patch_geometries
            .into_iter()
            .map(|geometry| {
                let mut background = SIMPLE_LINE_CALLOUT_CLASS_ID.to_vec();
                background.extend(97_u32.to_le_bytes());
                background.extend(1_u16.to_le_bytes());
                background.extend([0; 10]); // 标量。
                background.extend(geometry);
                background.extend([0; 32]); // 空引线和点锚。
                let mut groups = groups.clone();
                groups.splice(background_at..background_end, background);
                let mut renderer = colormap.clone();
                renderer.splice(24..28, groups);
                renderer
            })
            .collect();
        let mut discrete = super::super::raster_renderer::DISCRETE_COLOR_RENDERER_CLASS_ID.to_vec();
        discrete.extend(65_u32.to_le_bytes());
        discrete.extend(1_i32.to_le_bytes());
        discrete.extend(2_i32.to_le_bytes());
        discrete.extend(0x010203_u32.to_le_bytes());
        discrete.extend(0x040506_u32.to_le_bytes());
        discrete.extend(encoded_string("颜色"));
        discrete.extend(&random_ramp[colored_v9.len() - 16..]);
        discrete.extend(&colored_v9[24..]);
        let mut unique =
            super::super::raster_unique_value::RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID.to_vec();
        unique.extend(65_u32.to_le_bytes());
        unique.extend(3_i32.to_le_bytes());
        for _ in 0..3 {
            unique.extend(encoded_string(""));
        }
        unique.extend(0_u16.to_le_bytes());
        let unique_symbol_at = unique.len();
        unique.extend([0; 16]);
        unique.extend(encoded_string(""));
        unique.extend(0_i32.to_le_bytes()); // 图例组数。
        let unique_classes_at = unique.len();
        unique.extend([0; 6]); // 类别数、第二个 short。
        unique.extend([0; 16]); // 空 IUniqueValues。
        let mut unique_v1 = unique[..20].to_vec();
        unique_v1.extend(1_i32.to_le_bytes());
        for _ in 0..2 {
            unique_v1.extend(encoded_string(""));
        }
        unique_v1.extend(0_u16.to_le_bytes());
        unique_v1.extend([0; 16]); // 空符号。
        unique_v1.extend(encoded_string(""));
        unique_v1.extend(0_i32.to_le_bytes()); // 图例组前的标量。
        let legacy_groups_at = unique_v1.len();
        unique_v1.extend(0_i32.to_le_bytes()); // 图例组数。
        unique_v1.extend([0; 16]); // 空图例项。
        let legacy_classes_at = unique_v1.len();
        unique_v1.extend([0; 6]); // 类别数、第二个 short。
        unique_v1.extend([0; 20]); // 空旧统计对象及样本数。
        unique_v1.extend(&colored_v9[24..]);
        let mut unique_v2 = unique_v1.clone();
        unique_v2[20..24].copy_from_slice(&2_i32.to_le_bytes());
        let mut unique_with_values = unique[..unique.len() - 16].to_vec();
        unique_with_values.extend(UNIQUE_VALUES_CLASS_ID);
        unique_with_values.extend(67_u32.to_le_bytes());
        unique_with_values.extend(1_i32.to_le_bytes());
        unique_with_values.extend(1_i32.to_le_bytes());
        unique_with_values.extend(3_u16.to_le_bytes()); // VT_I4。
        unique_with_values.extend([0; 14]);
        unique_with_values.extend(17_i32.to_le_bytes());
        unique_with_values.extend(4_i32.to_le_bytes());
        unique_with_values.extend(&colored_v9[24..]);
        let mut unique_with_classes = unique_with_values.clone();
        let mut class = 1_i32.to_le_bytes().to_vec();
        class.extend(9_u32.to_le_bytes());
        class.extend(1_i32.to_le_bytes());
        class.extend(3_u16.to_le_bytes()); // VT_I4。
        class.extend([0; 14]);
        class.extend(17_i32.to_le_bytes());
        let mut unique_v1_with_classes = unique_v1.clone();
        unique_v1_with_classes.splice(legacy_classes_at..legacy_classes_at + 4, class.clone());
        let mut unique_v2_with_classes = unique_v2.clone();
        unique_v2_with_classes.splice(legacy_classes_at..legacy_classes_at + 4, class.clone());
        unique_with_classes.splice(unique_classes_at..unique_classes_at + 4, class);
        unique_v1_with_classes.splice(
            legacy_groups_at..legacy_groups_at + 4,
            groups.iter().copied(),
        );
        unique_with_classes.splice(unique_classes_at - 4..unique_classes_at, groups);
        let mut unique_with_ramp = unique.clone();
        unique_with_ramp[20..24].copy_from_slice(&4_i32.to_le_bytes());
        unique_with_ramp.extend(&random_ramp[colored_v9.len() - 16..]);
        unique.extend(&colored_v9[24..]);
        unique_with_ramp.extend(&colored_v9[24..]);
        let mut fill = SIMPLE_FILL_SYMBOL_CLASS_ID.to_vec();
        fill.extend(78_u32.to_le_bytes());
        fill.extend(1_u16.to_le_bytes());
        fill.extend(SIMPLE_LINE_SYMBOL_CLASS_ID);
        fill.extend(79_u32.to_le_bytes());
        fill.extend(1_u16.to_le_bytes());
        fill.extend([0; 36]); // 线符号空颜色和标量。
        fill.extend([0; 28]); // 面符号空颜色和标量。
        let mut unique_with_fill = unique.clone();
        unique_with_fill.splice(unique_symbol_at..unique_symbol_at + 16, fill);
        let classify = [1_i32, 2, 3, 4, 5].map(|version| {
            let mut bytes =
                super::super::raster_classify_renderer::RASTER_CLASSIFY_RENDERER_CLASS_ID.to_vec();
            bytes.extend(65_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            if version == 1 {
                bytes.extend(7_i32.to_le_bytes());
            }
            bytes.extend(encoded_string(""));
            bytes.extend(encoded_string(""));
            if version == 1 {
                bytes.extend(encoded_string("旧分类方法"));
            } else {
                bytes.push(0); // 无分类方法 GUID。
            }
            bytes.extend(encoded_string(""));
            bytes.extend([0; 20]); // 无图例组及图例项。
            if version == 1 {
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(17_i32.to_le_bytes());
            }
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            if version >= 2 {
                bytes.extend([0; 28]); // 数值字段及空格式。
            }
            if version >= 3 {
                bytes.extend([0; 50]); // VT_EMPTY ×2、选择子 0、空图例类别。
            }
            if version >= 4 {
                bytes.extend([0; 16]); // 空统计槽。
            }
            if version >= 5 {
                bytes.extend(0_u16.to_le_bytes());
            }
            bytes.extend(&colored_v9[24..]);
            bytes
        });
        for renderer in [
            &simple,
            &sai,
            &colored_v8,
            &colored_v9,
            &random_ramp,
            &preset_ramp,
            &algorithmic_ramp,
            &empty_algorithmic_ramp,
            &multipart_ramp,
            &legacy_rgb,
            &legacy_rgb_with_ramp,
            &tagged_rgb,
            &tagged_rgb_with_arrays,
            &tagged_rgb_with_array_reference,
            &tagged_rgb_with_histogram,
            &tagged_rgb_with_filter,
            &colormap,
            &colormap_with_groups,
            &colormaps_with_callouts[0],
            &colormaps_with_callouts[1],
            &colormaps_with_callouts[2],
            &colormaps_with_shape_anchors[0],
            &colormaps_with_shape_anchors[1],
            &colormaps_with_shape_anchors[2],
            &discrete,
            &unique_v1,
            &unique_v2,
            &unique_v1_with_classes,
            &unique_v2_with_classes,
            &unique,
            &unique_with_ramp,
            &unique_with_fill,
            &unique_with_values,
            &unique_with_classes,
            &classify[0],
            &classify[1],
            &classify[2],
            &classify[3],
            &classify[4],
        ] {
            let mut with_renderer = bytes.clone();
            with_renderer.splice(renderer_at..renderer_at + 16, renderer.iter().copied());
            let mut reader = Reader::new(&with_renderer);
            let mut refs = RefTable10_1::new();
            let mut gcs = GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID.to_vec();
            gcs.extend(94_u32.to_le_bytes());
            crate::storage::object_refs::read_object_entry_10_1(&mut Reader::new(&gcs), &mut refs)
                .unwrap();
            let parsed = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.renderer.unwrap().body.is_some());
            assert!(parsed.tail.unwrap().geodata_xform_v13.is_none());
            assert_eq!(reader.position(), end + renderer.len() - 16);
            assert!(refs.contains(65));
        }

        // 回引头的类标识不参与原生接口查询。
        let mut ramp_reference = random_ramp[..colored_v9.len() - 16].to_vec();
        ramp_reference.extend(RGB_COLOR_CLASS_ID);
        ramp_reference.extend(66_u32.to_le_bytes());
        let mut with_ramp_reference = bytes.clone();
        with_ramp_reference.splice(
            renderer_at..renderer_at + 16,
            ramp_reference.iter().copied(),
        );
        for class_id in [
            RANDOM_COLOR_RAMP_CLASS_ID,
            crate::objects::color_ramp::PRESET_COLOR_RAMP_CLASS_ID,
            crate::objects::color_ramp::ALGORITHMIC_COLOR_RAMP_CLASS_ID,
            crate::objects::color_ramp::MULTI_PART_COLOR_RAMP_CLASS_ID,
        ] {
            let mut registered = RefTable10_1::new();
            let mut registered_ramp = class_id.to_vec();
            registered_ramp.extend(66_u32.to_le_bytes());
            crate::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&registered_ramp),
                &mut registered,
            )
            .unwrap();
            let mut reader = Reader::new(&with_ramp_reference);
            read_inline_raster_body_10_1(&mut reader, 4096, &mut registered).unwrap();
            assert_eq!(reader.position(), end + ramp_reference.len() - 16);
        }

        let mut wrong_class = RefTable10_1::new();
        let ramp_header = &ramp_reference[ramp_reference.len() - 20..];
        crate::storage::object_refs::read_object_entry_10_1(
            &mut Reader::new(ramp_header),
            &mut wrong_class,
        )
        .unwrap();
        let mut reader = Reader::new(&with_ramp_reference);
        assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut wrong_class).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(wrong_class.registered_class(66), Some(RGB_COLOR_CLASS_ID));
        assert!(!wrong_class.contains(65));

        for class_id in [
            GRAY_COLOR_CLASS_ID,
            HLS_COLOR_CLASS_ID,
            HSV_COLOR_CLASS_ID,
            CMYK_COLOR_CLASS_ID,
        ] {
            let mut renderer = colored_v8.clone();
            renderer[color_at..color_at + 16].copy_from_slice(&class_id);
            let mut with_color = bytes.clone();
            with_color.splice(renderer_at..renderer_at + 16, renderer.iter().copied());
            let mut reader = Reader::new(&with_color);
            let mut refs = RefTable10_1::new();
            read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(reader.position(), end + renderer.len() - 16);
        }

        for original in [&simple, &sai] {
            let mut with_renderer = bytes.clone();
            with_renderer.splice(renderer_at..renderer_at + 16, original.iter().copied());
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&with_renderer);
            read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();

            let name_header = body_prefix(12, 1).len() + 20;
            let removed = 2 + encoded_string("").len() * 2 + 4 + 16;
            let mut with_renderer_ref = bytes.clone();
            let mut renderer_ref = RGB_COLOR_CLASS_ID.to_vec();
            renderer_ref.extend(65_u32.to_le_bytes());
            with_renderer_ref.splice(renderer_at..renderer_at + 16, renderer_ref);
            with_renderer_ref.drain(name_header..name_header + removed);
            let mut reader = Reader::new(&with_renderer_ref);
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(matches!(
                layer.renderer.unwrap().entry,
                ObjectEntry10_1::BackReference { key: 65, .. }
            ));
            assert_eq!(reader.position(), end + 4 - removed);
        }

        for class_id in [
            super::super::raster_renderer::RASTER_COLORMAP_RENDERER_CLASS_ID,
            super::super::raster_renderer::RASTER_RGB_RENDERER_CLASS_ID,
            super::super::raster_renderer::DISCRETE_COLOR_RENDERER_CLASS_ID,
            super::super::raster_unique_value::RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID,
            super::super::raster_classify_renderer::RASTER_CLASSIFY_RENDERER_CLASS_ID,
            super::super::raster_stretch_renderer::RASTER_STRETCH_RENDERER_CLASS_ID,
        ] {
            let mut refs = RefTable10_1::new();
            let mut registered_renderer = class_id.to_vec();
            registered_renderer.extend(65_u32.to_le_bytes());
            crate::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&registered_renderer),
                &mut refs,
            )
            .unwrap();
            let mut with_renderer_ref = bytes.clone();
            let mut renderer_ref = RGB_COLOR_CLASS_ID.to_vec();
            renderer_ref.extend(65_u32.to_le_bytes());
            with_renderer_ref.splice(renderer_at..renderer_at + 16, renderer_ref);
            let mut reader = Reader::new(&with_renderer_ref);
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(
                layer.renderer.unwrap().entry,
                ObjectEntry10_1::BackReference { class_id, key: 65 }
            );
            assert_eq!(reader.position(), end + 4);
        }

        let mut with_empty_color = bytes.clone();
        with_empty_color.splice(renderer_at..renderer_at + 16, simple_raster_io);
        let mut reader = Reader::new(&with_empty_color);
        let mut rejected_refs = RefTable10_1::new();
        let error =
            read_inline_raster_body_10_1(&mut reader, 4096, &mut rejected_refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert!(rejected_refs.is_empty());

        let header_offset = body_prefix(12, 1).len();
        let body_end = header_offset + 20 + 2 + encoded_string("").len() * 2 + 4 + 16;
        let mut reference = bytes.clone();
        reference.drain(header_offset + 20..body_end);
        let mut reader = Reader::new(&reference);
        let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(matches!(
            layer.names[0].entry,
            ObjectEntry10_1::BackReference { key: 60, .. }
        ));
        assert!(layer.tail.unwrap().geodata_xform_v13.is_none());
        assert_eq!(reader.position(), end - (body_end - header_offset - 20));

        let mut with_nulls = body_prefix(12, 3);
        let name_offset = with_nulls.len();
        let mut null = vec![0; 16];
        null.extend(encoded_string(""));
        let name_end = name_offset + 20 + 2 + encoded_string("").len() * 3 + 4 + 16;
        with_nulls.extend(&null);
        with_nulls.extend_from_slice(&bytes[name_offset..name_end]);
        with_nulls.extend(&null);
        with_nulls.extend_from_slice(&bytes[name_end..]);
        let mut reader = Reader::new(&with_nulls);
        let mut refs = RefTable10_1::new();
        let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(layer.names.len(), 3);
        assert_eq!(layer.names[0].entry, ObjectEntry10_1::Null);
        assert_eq!(layer.names[2].entry, ObjectEntry10_1::Null);
        assert!(layer.tail.unwrap().geodata_xform_v13.is_none());
        assert_eq!(reader.position(), end + 2 * null.len());
        assert_eq!(refs.registered_class(60), Some(RASTER_TYPE_NAME_CLASS_ID));

        let extension_count_at = name_end + 16 + 22 + 4;
        let relationship_count_at = extension_count_at + 4 + 8 + 32 + 4;
        let mut with_empty_slots = bytes.clone();
        with_empty_slots[relationship_count_at..relationship_count_at + 4]
            .copy_from_slice(&1_i32.to_le_bytes());
        with_empty_slots.splice(
            relationship_count_at + 4..relationship_count_at + 4,
            [0; 16],
        );
        with_empty_slots[extension_count_at..extension_count_at + 4]
            .copy_from_slice(&2_i32.to_le_bytes());
        let mut extensions = vec![0; 8];
        extensions.extend(16_u32.to_le_bytes());
        extensions.extend([0; 20]);
        with_empty_slots.splice(extension_count_at + 4..extension_count_at + 4, extensions);
        let mut reader = Reader::new(&with_empty_slots);
        let mut refs = RefTable10_1::new();
        let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let tail = layer.tail.unwrap();
        assert_eq!(tail.extra_safe_extensions.len(), 2);
        assert!(
            tail.extra_safe_extensions
                .iter()
                .all(|slot| slot.entry == ObjectEntry10_1::Null)
        );
        assert_eq!(tail.related_objects.len(), 1);
        assert_eq!(tail.related_objects[0].entry, ObjectEntry10_1::Null);
        assert_eq!(reader.position(), end + 8 + 24 + 16);

        use crate::maps::standalone_table::{OBJECT_CLASS_NAME_CLASS_ID, TABLE_NAME_CLASS_ID};

        use super::super::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
        // 同一 RasterTypeName 键可作为关联槽的 IName 回引。
        let mut related_reference = 1_i32.to_le_bytes().to_vec();
        related_reference.extend(IMAGE_SERVER_NAME_CLASS_ID);
        related_reference.extend(60_u32.to_le_bytes());
        let mut with_related_reference = bytes.clone();
        with_related_reference.splice(
            relationship_count_at - 4..relationship_count_at,
            related_reference,
        );
        let mut reader = Reader::new(&with_related_reference);
        let mut refs = RefTable10_1::new();
        let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(
            layer.tail.unwrap().related_name.unwrap().entry,
            ObjectEntry10_1::BackReference {
                class_id: RASTER_TYPE_NAME_CLASS_ID,
                key: 60,
            }
        );
        assert_eq!(reader.position(), end + 20);
        with_related_reference.splice(renderer_at..renderer_at + 16, simple.iter().copied());
        let mut reader = Reader::new(&with_related_reference);
        let mut refs = RefTable10_1::new();
        let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(layer.renderer.unwrap().body.is_some());
        assert!(layer.tail.unwrap().related_name.is_some());
        assert_eq!(reader.position(), end + 20 + simple.len() - 16);

        for text in ["", "service"] {
            let mut related = 1_i32.to_le_bytes().to_vec();
            related.extend(IMAGE_SERVER_NAME_CLASS_ID);
            related.extend(63_u32.to_le_bytes());
            related.extend(1_u16.to_le_bytes());
            related.extend(encoded_string(text));
            related.extend([0; 16]);
            let end = end + related.len() - 4;
            let mut with_related = bytes.clone();
            with_related.splice(relationship_count_at - 4..relationship_count_at, related);
            let mut reader = Reader::new(&with_related);
            let mut refs = RefTable10_1::new();
            let result = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs);
            if text.is_empty() {
                assert!(
                    result
                        .unwrap()
                        .tail
                        .unwrap()
                        .related_name
                        .unwrap()
                        .body
                        .is_some()
                );
                assert_eq!(reader.position(), end);
                assert!(refs.raster_name_opens_without_result(63));
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }

        for table_class in [TABLE_NAME_CLASS_ID, OBJECT_CLASS_NAME_CLASS_ID] {
            let name_flag_at = relationship_count_at - 4;
            let mut related_name = 1_i32.to_le_bytes().to_vec();
            related_name.extend(table_class);
            related_name.extend(63_u32.to_le_bytes());
            related_name.extend(1_u16.to_le_bytes());
            for _ in 0..3 {
                related_name.extend(encoded_string(""));
            }
            related_name.extend([0; 16]);
            let name_len = related_name.len();
            let mut with_related_name = bytes.clone();
            with_related_name.splice(name_flag_at..relationship_count_at, related_name);
            let mut reader = Reader::new(&with_related_name);
            let mut refs = RefTable10_1::new();
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = layer.tail.unwrap();
            assert!(tail.related_name.unwrap().body.is_some());
            assert!(tail.geodata_xform_v13.is_none());
            assert_eq!(reader.position(), end + name_len - 4);
            assert_eq!(refs.registered_class(63), Some(table_class));

            let reference_flag_at = name_flag_at - (body_end - header_offset - 20);
            let mut with_related_reference = reference.clone();
            let mut related_reference = 1_i32.to_le_bytes().to_vec();
            related_reference.extend(RASTER_TYPE_NAME_CLASS_ID);
            related_reference.extend(63_u32.to_le_bytes());
            with_related_reference
                .splice(reference_flag_at..reference_flag_at + 4, related_reference);
            let mut reader = Reader::new(&with_related_reference);
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = layer.tail.unwrap();
            let name = tail.related_name.unwrap();
            assert_eq!(
                name.entry,
                ObjectEntry10_1::BackReference {
                    class_id: table_class,
                    key: 63
                }
            );
            assert!(name.body.is_none());
            assert!(tail.geodata_xform_v13.is_none());
            assert_eq!(
                reader.position(),
                end - (body_end - header_offset - 20) + 20
            );

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let mut header = table_class.to_vec();
            header.extend(63_u32.to_le_bytes());
            crate::storage::object_refs::read_object_entry_10_1(
                &mut Reader::new(&header),
                &mut refs,
            )
            .unwrap();
            let before = refs.clone();
            let mut reader = Reader::new(&with_related_reference);
            assert_eq!(
                read_inline_raster_body_10_1(&mut reader, 4096, &mut refs)
                    .unwrap_err()
                    .kind(),
                ErrorKind::Unsupported
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, before);

            let mut reader = Reader::new(&with_related_name[..name_flag_at + name_len - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }

        let mut second_name = Vec::new();
        dataset_name(&mut second_name, 61);
        second_name.extend(encoded_string(""));
        for first_fails in [false, true] {
            let mut multiple = body_prefix(12, 2);
            if !first_fails {
                multiple.extend(&second_name);
            }
            multiple.extend_from_slice(&bytes[name_offset..name_end]);
            if first_fails {
                multiple.extend(&second_name);
            }
            multiple.extend_from_slice(&bytes[name_end..]);
            let mut reader = Reader::new(&multiple);
            let mut refs = RefTable10_1::new();
            let result = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs);
            if first_fails {
                let layer = result.unwrap();
                assert_eq!(layer.names.len(), 2);
                assert!(layer.tail.unwrap().geodata_xform_v13.is_none());
                assert_eq!(reader.position(), end + second_name.len());
                assert!(refs.contains(60) && refs.contains(61));
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }

        let io_offset = end - 16 - encoded_string("尾段").len();
        let mut envelope_reference = Vec::from(RASTER_TYPE_NAME_CLASS_ID);
        envelope_reference.extend(61_u32.to_le_bytes());
        let mut with_reference = reference.clone();
        let reference_io_offset = io_offset - (body_end - header_offset - 20);
        with_reference.splice(
            reference_io_offset - 16..reference_io_offset,
            envelope_reference,
        );
        for (version, with_spatial_reference) in [(1_u16, false), (3, false), (3, true)] {
            use crate::objects::envelope::ENVELOPE_CLASS_ID;
            use crate::objects::spatial_reference_slots::UNKNOWN_COORDINATE_SYSTEM_CLASS_ID;

            let mut envelope = Vec::from(ENVELOPE_CLASS_ID);
            envelope.extend(61_u32.to_le_bytes());
            envelope.extend(version.to_le_bytes());
            for coordinate in [1.0_f64, 2.0, 3.0, 4.0] {
                envelope.extend(coordinate.to_le_bytes());
            }
            if version == 3 {
                envelope.extend([0; 4]);
                if with_spatial_reference {
                    envelope.extend(UNKNOWN_COORDINATE_SYSTEM_CLASS_ID);
                    envelope.extend(62_u32.to_le_bytes());
                    envelope.extend(1_u16.to_le_bytes());
                    for _ in 0..4 {
                        envelope.extend(encoded_string(""));
                    }
                    envelope.extend([0; 56]);
                } else {
                    envelope.extend([0; 16]);
                }
            }
            let mut with_envelope = bytes.clone();
            with_envelope.splice(io_offset - 16..io_offset, envelope.iter().copied());
            let mut reader = Reader::new(&with_envelope);
            let mut refs = RefTable10_1::new();
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = layer.tail.unwrap();
            let envelope_body = tail.envelope_f0.unwrap().body.unwrap();
            if with_spatial_reference {
                use crate::objects::envelope::SpatialReferenceEntry10_1;

                assert!(matches!(
                    envelope_body.spatial_reference,
                    Some(SpatialReferenceEntry10_1::UnknownCoordinateSystem { key: 62, .. })
                ));
                assert_eq!(
                    refs.registered_class(62),
                    Some(UNKNOWN_COORDINATE_SYSTEM_CLASS_ID)
                );
                let mut with_sr_reference = reference.clone();
                let sr_body_at = envelope.len() - (2 + 4 * encoded_string("").len() + 56);
                let mut referenced_envelope = envelope[..sr_body_at].to_vec();
                referenced_envelope[16..20].copy_from_slice(&63_u32.to_le_bytes());
                with_sr_reference.splice(
                    reference_io_offset - 16..reference_io_offset,
                    referenced_envelope,
                );
                let mut sr_reader = Reader::new(&with_sr_reference);
                let layer = read_inline_raster_body_10_1(&mut sr_reader, 4096, &mut refs).unwrap();
                assert!(matches!(
                    layer
                        .tail
                        .unwrap()
                        .envelope_f0
                        .unwrap()
                        .body
                        .unwrap()
                        .spatial_reference,
                    Some(SpatialReferenceEntry10_1::BackReference { key: 62, .. })
                ));
                assert_eq!(
                    sr_reader.position(),
                    reference_io_offset + (end - io_offset) + sr_body_at - 16
                );
                let cut = io_offset - 16 + envelope.len() - 1;
                let mut truncated = Reader::new(&with_envelope[..cut]);
                let mut empty_refs = RefTable10_1::new();
                assert!(
                    read_inline_raster_body_10_1(&mut truncated, 4096, &mut empty_refs).is_err()
                );
                assert_eq!(truncated.position(), 0);
                assert!(empty_refs.is_empty());
            }
            assert!(tail.geodata_xform_v13.is_none());
            assert_eq!(reader.position(), end + envelope.len() - 16);
            assert_eq!(refs.registered_class(61), Some(ENVELOPE_CLASS_ID));

            let mut reader = Reader::new(&with_reference);
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = layer.tail.unwrap();
            let slot = tail.envelope_f0.unwrap();
            assert_eq!(
                slot.entry,
                ObjectEntry10_1::BackReference {
                    class_id: ENVELOPE_CLASS_ID,
                    key: 61,
                }
            );
            assert!(slot.body.is_none());
            assert!(tail.geodata_xform_v13.is_none());
            assert_eq!(
                reader.position(),
                reference_io_offset + (end - io_offset) + 4
            );
        }

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(!refs.register(61));
        let mut reader = Reader::new(&with_reference);
        assert_eq!(
            read_inline_raster_body_10_1(&mut reader, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(reader.position(), 0);

        use super::super::raster_dataset_name::{
            INTERNET_TILED_LAYER_NAME_CLASS_ID, NETCDF_RASTER_DATASET_NAME_CLASS_ID,
        };
        use crate::layers::wcs_name::WCS_COVERAGE_NAME_CLASS_ID;

        use super::super::raster_dataset_name::IS_RASTER_DATASET_NAME_CLASS_ID;

        for (tag30, tag31, class_id) in [
            (None, None, [0; 16]),
            (Some(false), None, [0; 16]),
            (None, Some("路径"), [0; 16]),
            (Some(false), Some("路径"), [0; 16]),
            (Some(true), Some(""), RASTER_TYPE_NAME_CLASS_ID),
            (Some(true), Some("路径"), RASTER_TYPE_NAME_CLASS_ID),
            (Some(true), Some("路径"), WCS_COVERAGE_NAME_CLASS_ID),
            (Some(true), Some("路径"), IMAGE_SERVER_NAME_CLASS_ID),
            (Some(true), Some("路径"), IS_RASTER_DATASET_NAME_CLASS_ID),
            (Some(true), Some("路径"), INTERNET_TILED_LAYER_NAME_CLASS_ID),
            (
                Some(true),
                Some("路径"),
                NETCDF_RASTER_DATASET_NAME_CLASS_ID,
            ),
        ] {
            let mut records = Vec::new();
            records.extend(0xff00ff_i32.to_le_bytes());
            records.extend(0_i32.to_le_bytes());
            if let Some(has_name) = tag30 {
                records.extend(30_i32.to_le_bytes());
                records.extend((-1_i32).to_le_bytes());
                if has_name {
                    records.extend(class_id);
                    match class_id {
                        RASTER_TYPE_NAME_CLASS_ID => {
                            records.extend(1_u16.to_le_bytes());
                            records.extend(encoded_string(""));
                            records.extend(encoded_string(""));
                            records.extend((-1_i32).to_le_bytes());
                        }
                        IS_RASTER_DATASET_NAME_CLASS_ID => {
                            records.extend(1_u16.to_le_bytes());
                            for _ in 0..6 {
                                records.extend(encoded_string(""));
                            }
                            records.extend(0_i32.to_le_bytes());
                            records.extend([0; 16]); // 空工作空间。
                        }
                        IMAGE_SERVER_NAME_CLASS_ID => {
                            records.extend(1_u16.to_le_bytes());
                            records.extend(encoded_string(""));
                        }
                        WCS_COVERAGE_NAME_CLASS_ID => {
                            records.extend(1_u16.to_le_bytes());
                            records.extend(1_u32.to_le_bytes());
                            records.extend(u16::from(b'a').to_le_bytes());
                            records.extend(0_u16.to_le_bytes());
                        }
                        INTERNET_TILED_LAYER_NAME_CLASS_ID => {
                            records.extend(1_u16.to_le_bytes());
                            records.extend(encoded_string("Tiles"));
                            records.extend((-1_i32).to_le_bytes());
                        }
                        NETCDF_RASTER_DATASET_NAME_CLASS_ID => {
                            records.extend(2_i32.to_le_bytes());
                            for _ in 0..8 {
                                records.extend(encoded_string(""));
                            }
                            records.extend([0; 32]); // 两个数组均为空。
                        }
                        _ => unreachable!(),
                    }
                    records.extend([0; 16]);
                } else {
                    records.extend([0; 16]);
                }
            }
            if tag30 == Some(true) {
                // 将同一个名称体分别放入关联槽的新对象和回引记录。
                let name_body = &records[32..];
                let flag_at = relationship_count_at - 4;
                let mut refs = RefTable10_1::new();
                for is_new in [true, false] {
                    let key = 64_u32;
                    let mut related = 1_i32.to_le_bytes().to_vec();
                    related.extend(class_id);
                    related.extend(key.to_le_bytes());
                    if is_new {
                        related.extend(name_body);
                    }
                    let related_len = related.len();
                    let (mut with_related, removed) = if is_new {
                        (bytes.clone(), 0)
                    } else {
                        (reference.clone(), body_end - header_offset - 20)
                    };
                    with_related
                        .splice(flag_at - removed..relationship_count_at - removed, related);
                    let mut reader = Reader::new(&with_related);
                    let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                    let tail = layer.tail.unwrap();
                    assert_eq!(tail.related_name.unwrap().body.is_some(), is_new);
                    assert!(tail.geodata_xform_v13.is_none());
                    assert_eq!(refs.registered_class(key), Some(class_id));
                    assert_eq!(reader.position(), end - removed + related_len - 4);
                }
            }
            if let Some(path) = tag31 {
                records.extend(31_i32.to_le_bytes());
                records.extend(encoded_string(path));
            }
            for tag in 35_i32..=37 {
                records.extend(tag.to_le_bytes());
                records.extend(2_i32.to_le_bytes());
                records.extend(7_u16.to_le_bytes());
            }
            records.extend([0xff; 8]);
            let records_len = records.len();
            let mut with_io = bytes.clone();
            with_io.splice(io_offset..io_offset, records);
            let mut reader = Reader::new(&with_io);
            let mut refs = RefTable10_1::new();
            let layer = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = layer.tail.unwrap();
            assert!(tail.geodata_xform_v13.is_none());
            assert!(tail.envelope_v13.is_none());
            let io = tail.raster_io.unwrap();
            assert_eq!(io.tag30_is_explicit_null, tag30 == Some(false));
            assert_eq!(io.tag30_name.is_some(), tag30 == Some(true));
            assert_eq!(io.tag31_string.as_deref(), tag31);
            assert_eq!(io.shorts_35_to_37, [7; 3]);
            assert_eq!(reader.position(), end + records_len);
        }

        let mut list = 2_i32.to_le_bytes().to_vec();
        for is_new in [true, false] {
            list.extend(1_u16.to_le_bytes()); // 普通对象，无 ExtraSafe 外界。
            list.extend(RASTER_LAYER_CLASS_ID);
            list.extend(55_u32.to_le_bytes());
            if is_new {
                list.extend(&bytes[..end]);
            }
        }
        list.extend(1_i32.to_le_bytes()); // 活动图层索引。
        let list_end = list.len();
        list.extend([0xa5; 4]);
        let mut reader = Reader::new(&list);
        let mut refs = RefTable10_1::new();
        let report = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(report.stop.is_none());
        assert!(
            matches!(report.entries[0].parsed.as_ref(), Some(LayerBodyPrefix10_1::RasterLayer(layer))
            if layer.version == 12 && layer.tail.as_ref().is_some_and(|tail| tail.geodata_xform_v13.is_none()))
        );
        assert!(matches!(
            report.entries[1].object,
            LayerEntryObject::Entry(ObjectEntry10_1::BackReference { key: 55, .. })
        ));
        assert_eq!(report.active_index, Some(1));
        assert_eq!(reader.position(), list_end);
        assert_eq!(refs.registered_class(55), Some(RASTER_LAYER_CLASS_ID));
        assert_eq!(refs.registered_class(60), Some(RASTER_TYPE_NAME_CLASS_ID));
    }

    #[test]
    fn raster_v12_uses_only_a_declared_boundary_to_choose_its_optional_slots() {
        for pair in [false, true] {
            let bytes = raster_with_empty_slots(12, pair);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let error = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
            let parsed =
                read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(parsed.tail.unwrap().geodata_xform_v13.is_some(), pair);
            assert_eq!(reader.remaining(), 0);
            assert!(refs.contains(60));
        }
    }

    #[test]
    fn legacy_map_extra_safe_passes_boundary_to_raster_v12_and_wcs() {
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, LayerEntryObject, RASTER_LAYER_CLASS_ID,
            read_legacy_layer_lists_10_1,
        };
        use crate::layers::wcs_layer::WCS_LAYER_CLASS_ID;

        for wrapper in [false, true] {
            for (pair, truncated) in [(false, false), (true, false), (true, true)] {
                let mut object = if wrapper {
                    let mut object = WCS_LAYER_CLASS_ID.to_vec();
                    object.extend(51_u32.to_le_bytes());
                    object.extend(1_i32.to_le_bytes());
                    for text in ["覆盖", "地址", "标识"] {
                        object.extend(encoded_string(text));
                    }
                    object
                } else {
                    Vec::new()
                };
                object.extend(RASTER_LAYER_CLASS_ID);
                object.extend(52_u32.to_le_bytes());
                object.extend(raster_with_empty_slots(12, pair));
                if truncated {
                    object.pop();
                }
                let mut bytes = 1_i32.to_le_bytes().to_vec();
                let object_start = bytes.len();
                if wrapper {
                    bytes.extend(0_u32.to_le_bytes());
                    bytes.extend((object.len() as u32).to_le_bytes());
                } else {
                    bytes.extend((object.len() as u32).to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                }
                bytes.extend(object);
                bytes.extend(1_i32.to_le_bytes()); // 第二组复用第一组的栅格层。
                bytes.extend(RASTER_LAYER_CLASS_ID);
                bytes.extend(52_u32.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
                let end = bytes.len();
                bytes.push(0x53);

                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                refs.register(5);
                let list = read_legacy_layer_lists_10_1(&mut reader, 4096, &mut refs, 19).unwrap();
                if truncated {
                    assert!(list.stop.is_some());
                    assert_eq!(reader.position(), object_start);
                    assert_eq!(refs.len(), 1);
                    assert!(refs.contains(5));
                    continue;
                }
                let raster = match list.entries[0].parsed.as_ref() {
                    Some(LayerBodyPrefix10_1::RasterLayer(raster)) => raster,
                    Some(LayerBodyPrefix10_1::WcsLayer(wcs)) => wcs.raster.body.as_ref().unwrap(),
                    unexpected => panic!("预期完整有界栅格，读到 {unexpected:?}"),
                };
                let tail = raster.tail.as_ref().unwrap();
                assert_eq!(raster.version, 12);
                assert_eq!(tail.geodata_xform_v13.is_some(), pair);
                assert!(matches!(
                    list.entries[1].object,
                    LayerEntryObject::Entry(ObjectEntry10_1::BackReference { key: 52, .. })
                ));
                assert!(list.stop.is_none());
                assert_eq!(list.active_index, Some(0));
                assert!(refs.contains(52) && refs.contains(60));
                assert_eq!(reader.position(), end);
                assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);
            }
        }
    }

    #[test]
    fn bounded_raster_distinguishes_projective_third_null_slot_at_the_layer_end() {
        use crate::objects::envelope::ENVELOPE_CLASS_ID;
        use crate::objects::geodata_xform::{GeodataXformBody10_1, PROJECTIVE_XFORM_CLASS_ID};
        use crate::objects::geometry::MULTIPOINT_CLASS_ID;

        let transform_offset = raster_with_empty_slots(12, false).len();
        for (version, legacy, geometry, point_count) in [
            (12, false, false, 0_i32),
            (12, true, false, 0),
            (12, true, true, 0),
            (13, true, true, 0),
            (13, true, true, -1),
            (14, true, true, 0),
            (17, true, true, 1),
        ] {
            let mut bytes = raster_with_empty_slots(version, true);
            let mut pair = PROJECTIVE_XFORM_CLASS_ID.to_vec();
            pair.extend(71_u32.to_le_bytes());
            pair.extend(1_i32.to_le_bytes());
            pair.extend([0; 16]); // 空空间参考。
            if legacy {
                if geometry {
                    pair.extend(MULTIPOINT_CLASS_ID);
                    pair.extend(72_u32.to_le_bytes());
                    pair.extend(2_u16.to_le_bytes());
                    pair.extend(4_u32.to_le_bytes());
                    pair.extend(0_u32.to_le_bytes());
                    pair.extend([0; 16]);
                } else {
                    pair.extend([0; 16]); // 原生入口实验中的空几何集合。
                }
            }
            pair.extend([0; 16]); // 空投影变换。
            let points = (point_count > 0).then_some([[1.0_f64, 2.0], [3.0, 4.0]]);
            let points_flag = i32::from(points.is_some());
            pair.extend(point_count.to_le_bytes());
            pair.extend(points_flag.to_le_bytes());
            if let Some(points) = points {
                for coordinate in points.into_iter().flatten() {
                    pair.extend(coordinate.to_le_bytes());
                }
            }
            let point_end = transform_offset + pair.len();
            pair.extend(ENVELOPE_CLASS_ID);
            pair.extend(73_u32.to_le_bytes());
            pair.extend(1_u16.to_le_bytes());
            pair.extend([0; 32]);
            let missing_byte = transform_offset + pair.len() - 1;
            bytes.splice(transform_offset..transform_offset + 32, pair);
            if version == 14 {
                // 保留空颜色槽，追加一个非空索引序列。
                let count = bytes.len() - 4;
                bytes[count..].copy_from_slice(&2_i32.to_le_bytes());
                bytes.extend(4_i32.to_le_bytes());
                bytes.extend(7_i32.to_le_bytes());
            }

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let layer =
                read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(reader.remaining(), 0);
            let tail = layer.tail.unwrap();
            let transform = tail.geodata_xform_v13.unwrap();
            let Some(GeodataXformBody10_1::Projective(projective)) = transform.body.as_deref()
            else {
                panic!("应读取 ProjectiveXform");
            };
            assert_eq!(
                projective.transformation,
                legacy.then_some(ObjectEntry10_1::Null)
            );
            assert_eq!(projective.geometry.body.is_some(), geometry);
            assert_eq!(projective.point_count, point_count);
            assert_eq!(projective.points_flag, points_flag);
            assert_eq!(
                projective.point_arrays,
                points.map(|points| points.map(|point| vec![point]))
            );
            assert!(tail.envelope_v13.is_some());
            assert_eq!(
                tail.color_indices_v14,
                if version == 14 { &[4, 7][..] } else { &[] }
            );
            assert!(refs.contains(60) && refs.contains(71) && refs.contains(73));
            assert_eq!(refs.contains(72), geometry);

            bytes.push(0xa5);
            for end in [point_end - 1, missing_byte, bytes.len()] {
                let mut reader = Reader::new(&bytes[..end]);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn projective_null_third_slot_has_two_valid_raster_prefix_boundaries() {
        use crate::objects::geodata_xform::{GeodataXformBody10_1, PROJECTIVE_XFORM_CLASS_ID};

        let offset = raster_with_empty_slots(12, false).len();
        let mut bytes = raster_with_empty_slots(13, true);
        let mut pair = PROJECTIVE_XFORM_CLASS_ID.to_vec();
        pair.extend(71_u32.to_le_bytes());
        pair.extend(1_i32.to_le_bytes());
        pair.extend([0; 16 * 3 + 8 + 16]); // 空空间参考、几何、变换、零点数和标志、空包络。
        bytes.splice(offset..offset + 32, pair);
        let long_end = bytes.len();
        let short_end = long_end - 16;

        for (end, expected) in [(short_end, None), (long_end, Some(ObjectEntry10_1::Null))] {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = RefTable10_1::new();
            let layer =
                read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let slot = layer.tail.unwrap().geodata_xform_v13.unwrap();
            let GeodataXformBody10_1::Projective(body) = *slot.body.unwrap() else {
                panic!("应读取 ProjectiveXform");
            };
            assert_eq!(body.transformation, expected);
            assert_eq!(reader.position(), end);
            assert!(refs.contains(71));
        }
    }

    #[test]
    fn ordinary_group_can_use_its_extra_safe_raster_child_boundary() {
        use crate::layers::group_layer::{GROUP_LAYER_CLASS_ID, GroupChildEntry10_1};
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_complete_layer_body_10_1,
        };

        for pair in [false, true] {
            let raster = raster_with_empty_slots(12, pair);
            let mut bytes = 11_u16.to_le_bytes().to_vec();
            bytes.extend(encoded_string("组"));
            bytes.extend([0; 6 + 16]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend((20 + raster.len() as u32).to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(RASTER_LAYER_CLASS_ID);
            bytes.extend(55_u32.to_le_bytes());
            bytes.extend(raster);
            bytes.extend([0; 2 + 16 + 8 + 4]);
            bytes.extend(encoded_string(""));
            bytes.extend([0; 2 + 16 + 2 + 6]);
            let end = bytes.len();
            bytes.extend([0x55; 16]);

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let LayerBodyPrefix10_1::GroupLayer(group) =
                read_complete_layer_body_10_1(&mut reader, GROUP_LAYER_CLASS_ID, 4096, &mut refs)
                    .unwrap()
            else {
                panic!("应读取组图层");
            };
            let GroupChildEntry10_1::SkippedNewObject {
                parsed: Some(raster),
                body_length: 0,
                ..
            } = &group.children[0]
            else {
                panic!("ExtraSafe 内的栅格子体应完整消费");
            };
            assert!(
                matches!(raster.as_ref(), LayerBodyPrefix10_1::RasterLayer(body)
                if body.tail.as_ref().unwrap().geodata_xform_v13.is_some() == pair)
            );
            assert!(refs.contains(55) && refs.contains(60));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn ordinary_raster_reads_globe_properties_and_later_back_reference() {
        use crate::layers::globe_layer_properties::GLOBE_LAYER_PROPERTIES_CLASS_ID;

        let mut bytes = raster_with_empty_slots(8, false);
        let extension_count = bytes.len() - (4 + 8 + 32 + 8 + 16);
        let mut globe = 1_u16.to_le_bytes().to_vec();
        globe.extend(0_u32.to_le_bytes());
        globe.extend(encoded_string(""));
        globe.extend([0; 16 + 2 + 4]);
        let mut extensions = 2_u32.to_le_bytes().to_vec();
        extensions.extend((20 + globe.len() as u32).to_le_bytes());
        extensions.extend(0_u32.to_le_bytes());
        extensions.extend(GLOBE_LAYER_PROPERTIES_CLASS_ID);
        extensions.extend(61_u32.to_le_bytes());
        extensions.extend(globe);
        extensions.extend(20_u32.to_le_bytes());
        extensions.extend(0_u32.to_le_bytes());
        extensions.extend(GLOBE_LAYER_PROPERTIES_CLASS_ID);
        extensions.extend(61_u32.to_le_bytes());
        bytes.splice(extension_count..extension_count + 4, extensions);
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let tail = raster.tail.unwrap();
        assert!(
            tail.extra_safe_extensions[0]
                .globe_layer_properties
                .is_some()
        );
        assert!(matches!(
            tail.extra_safe_extensions[1].entry,
            ObjectEntry10_1::BackReference { key: 61, .. }
        ));
        assert!(refs.contains(61));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn ordinary_raster_reads_scene_properties_in_both_extension_encodings() {
        use crate::layers::scene_layer_properties::{
            RASTER_3D_PROPERTIES_CLASS_ID, SurfaceName10_1,
        };
        use crate::objects::envelope::ENVELOPE_CLASS_ID;

        let mut properties = 13_u16.to_le_bytes().to_vec();
        properties.extend(1_u32.to_le_bytes());
        // RasterLayer 的首个数据源名称已经登记过。
        properties.extend(crate::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID);
        properties.extend(60_u32.to_le_bytes());
        properties.extend(encoded_string("地表路径"));
        properties.extend([0; 20]);
        properties.extend(encoded_string("偏移"));
        properties.extend([0; 4]);
        properties.extend(encoded_string("拉伸"));
        properties.extend([0; 106]);
        properties.extend(ENVELOPE_CLASS_ID);
        properties.extend(71_u32.to_le_bytes());
        properties.extend(1_u16.to_le_bytes());
        properties.extend([0; 32]);
        for version in [3, 8] {
            let mut bytes = raster_with_empty_slots(version, false);
            let extension_count = bytes.len() - if version == 3 { 4 } else { 68 };
            let mut encoded = 1_u32.to_le_bytes().to_vec();
            let size = properties.len() as u32 + if version == 3 { 0 } else { 20 };
            encoded.extend(u64::from(size).to_le_bytes());
            encoded.extend(RASTER_3D_PROPERTIES_CLASS_ID);
            if version == 8 {
                encoded.extend(70_u32.to_le_bytes());
            }
            encoded.extend(&properties);
            bytes.splice(extension_count..extension_count + 4, encoded);
            let end = bytes.len();
            bytes.push(0xa5);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = raster.tail.unwrap();
            let properties = if version == 3 {
                &tail.legacy_extensions[0].scene_layer_properties
            } else {
                &tail.extra_safe_extensions[0].scene_layer_properties
            }
            .as_ref()
            .unwrap();
            assert!(matches!(
                properties.surface_name,
                Some(SurfaceName10_1::Raster(ref name))
                    if matches!(name.entry, ObjectEntry10_1::BackReference { key: 60, .. })
            ));
            assert!(refs.contains(71));
            assert_eq!(refs.contains(70), version == 8);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn ordinary_raster_reads_direct_property_set_and_dynamic_extension_in_both_encodings() {
        use crate::layers::layer_extensions::DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID;
        use crate::objects::property_set::{PROPERTY_SET_CLASS_ID, PropertyValueObjectStream10_1};

        let mut direct = 1_u16.to_le_bytes().to_vec();
        direct.extend(1_u32.to_le_bytes());
        direct.extend(encoded_string("配置"));
        direct.extend(13_u16.to_le_bytes());
        direct.extend(PROPERTY_SET_CLASS_ID);
        direct.extend(72_u32.to_le_bytes());
        direct.extend(1_u16.to_le_bytes());
        direct.extend(0_u32.to_le_bytes());

        let mut dynamic = 1_u16.to_le_bytes().to_vec();
        dynamic.extend(encoded_string("瓦片源"));
        dynamic.extend(PROPERTY_SET_CLASS_ID);
        dynamic.extend(72_u32.to_le_bytes());
        dynamic.extend(encoded_string(""));
        dynamic.extend(encoded_string(""));
        dynamic.extend([0; 12]);

        for version in [3, 8] {
            let build = |direct: &[u8]| {
                let mut bytes = raster_with_empty_slots(version, false);
                let extension_count = bytes.len() - if version == 3 { 4 } else { 68 };
                let mut encoded = 2_u32.to_le_bytes().to_vec();
                for (class_id, key, body) in [
                    (PROPERTY_SET_CLASS_ID, 70_u32, direct),
                    (DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID, 71, &dynamic),
                ] {
                    let size = body.len() as u32 + if version == 8 { 20 } else { 0 };
                    encoded.extend(u64::from(size).to_le_bytes());
                    encoded.extend(class_id);
                    if version == 8 {
                        encoded.extend(key.to_le_bytes());
                    }
                    encoded.extend(body);
                }
                bytes.splice(extension_count..extension_count + 4, encoded);
                (bytes, extension_count)
            };

            let (mut bytes, extension_count) = build(&direct);
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(reader.position(), end);
            assert!(refs.contains(72));
            assert_eq!(refs.contains(70) && refs.contains(71), version == 8);
            let tail = result.tail.unwrap();
            let (properties, dynamic) = if version == 3 {
                let mut legacy = Reader::new(&bytes[extension_count..end]);
                let mut legacy_refs = RefTable10_1::new();
                let shared = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut legacy,
                    4096,
                    &mut legacy_refs,
                )
                .unwrap();
                assert_eq!(legacy.remaining(), 0);
                assert_eq!(shared[0].properties, tail.legacy_extensions[0].properties);
                assert!(legacy_refs.contains(72));
                (
                    tail.legacy_extensions[0].properties.as_deref().unwrap(),
                    tail.legacy_extensions[1]
                        .dynamic_tiled_layer_properties
                        .as_deref()
                        .unwrap(),
                )
            } else {
                (
                    tail.extra_safe_extensions[0].properties.as_deref().unwrap(),
                    tail.extra_safe_extensions[1]
                        .dynamic_tiled_layer_properties
                        .as_deref()
                        .unwrap(),
                )
            };
            assert!(matches!(
                properties.entries[0].value,
                PropertyValueObjectStream10_1::Object { key: 72, .. }
            ));
            assert!(matches!(
                dynamic.properties.entry,
                ObjectEntry10_1::BackReference { key: 72, .. }
            ));

            let mut excessive = direct.clone();
            excessive.push(0xa5);
            for invalid in [&direct[..direct.len() - 1], &excessive] {
                let (bytes, _) = build(invalid);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn ordinary_raster_rejects_unknown_bounded_extension_without_leaking_keys() {
        let mut bytes = raster_with_empty_slots(8, false);
        let extension_count = bytes.len() - (4 + 8 + 32 + 8 + 16);
        let mut extension = 1_u32.to_le_bytes().to_vec();
        extension.extend(21_u32.to_le_bytes());
        extension.extend(0_u32.to_le_bytes());
        extension.extend([0x53; 16]);
        extension.extend(61_u32.to_le_bytes());
        extension.push(0xaa);
        bytes.splice(extension_count..extension_count + 4, extension);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
        assert!(
            read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs)
                .unwrap()
                .tail
                .is_some()
        );
    }

    #[test]
    fn raster_associated_layer_and_map_list_read_past_the_raster_body() {
        use crate::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_layer_list_10_1,
        };

        let mut entry = RASTER_LAYER_CLASS_ID.to_vec();
        entry.extend(55_u32.to_le_bytes());
        entry.extend(raster_with_empty_slots(17, false));
        let mut scene = 1_u16.to_le_bytes().to_vec();
        scene.extend(encoded_string("Scene"));
        scene.extend([0; 8]);
        scene.extend(&entry);
        scene.extend([0; 16 + 4]);
        scene.push(0x55);
        let mut reader = Reader::new(&scene);
        let mut refs = RefTable10_1::new();
        let scene =
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .unwrap();
        assert!(
            matches!(scene.associated_layer_body.as_deref(), Some(LayerBodyPrefix10_1::RasterLayer(body))
            if body.version == 17 && body.tail.is_some())
        );
        assert_eq!(reader.read_bytes(1).unwrap(), [0x55]);
        assert!(refs.contains(55) && refs.contains(60));

        let mut list = 2_u32.to_le_bytes().to_vec();
        list.extend(1_u16.to_le_bytes());
        list.extend(entry);
        list.extend(1_u16.to_le_bytes());
        list.extend(RASTER_LAYER_CLASS_ID);
        list.extend(55_u32.to_le_bytes());
        list.extend(1_i32.to_le_bytes());
        let mut reader = Reader::new(&list);
        let mut refs = RefTable10_1::new();
        let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.active_index, Some(1));
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(55) && refs.contains(60));
    }

    #[test]
    fn bounded_names_read_known_objects_backreferences_and_nulls_before_renderer() {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        use super::super::raster_renderer::{
            RasterRendererPrefix10_1, SIMPLE_RASTER_RENDERER_CLASS_ID,
        };

        let mut bytes = body_prefix(17, 3);
        dataset_name(&mut bytes, 60);
        bytes.extend(encoded_string("第一项"));
        bytes.extend([0; 16]);
        bytes.extend(encoded_string("空项后缀"));
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(encoded_string("回引后缀"));
        bytes.extend(SIMPLE_RASTER_RENDERER_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes()); // SimpleRasterRenderer v1。
        bytes.extend(1_i32.to_le_bytes()); // 共用体版本。
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0xa1);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(result.names_stop.is_none());
        assert_eq!(result.names.len(), 3);
        assert!(matches!(
            result.names[0].body,
            Some(RasterDatasetNameEntryBody10_1::Dataset(_))
        ));
        assert_eq!(result.names[1].entry, ObjectEntry10_1::Null);
        assert_eq!(result.names[1].suffix.as_deref(), Some("空项后缀"));
        assert_eq!(
            result.names[2].entry,
            ObjectEntry10_1::BackReference {
                class_id: RASTER_DATASET_NAME_CLASS_ID,
                key: 60
            }
        );
        assert_eq!(
            result.names[2]
                .source_reference
                .as_ref()
                .map(|source| source.name.as_ref()),
            Some("data"),
        );
        assert_eq!(result.names[2].suffix.as_deref(), Some("回引后缀"));
        assert!(matches!(
            result.renderer.unwrap().body,
            Some(RasterRendererPrefix10_1::Simple { .. })
        ));
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa1]);
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn service_and_tiled_names_share_layer_keys_and_reach_renderer() {
        use super::super::raster_dataset_name::{
            AIS_IMAGE_SERVICE_NAME_CLASS_ID, INTERNET_TILED_LAYER_NAME_CLASS_ID,
        };
        use super::super::raster_renderer::{
            RasterRendererPrefix10_1, SIMPLE_RASTER_RENDERER_CLASS_ID,
        };
        use crate::layers::cache_tile_reader::CacheTileReaderBody10_1;

        let mut bytes = body_prefix(17, 4);
        bytes.extend(INTERNET_TILED_LAYER_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("瓦片"));
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend([
            0x16, 0xec, 0xb2, 0x48, 0xdb, 0xb0, 0x85, 0x4d, 0x9a, 0x86, 0x1d, 0xdb, 0x04, 0x04,
            0x85, 0x79,
        ]); // MapTileReader。
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string(""));
        bytes.extend(encoded_string(""));
        bytes.extend([0; 4]);
        bytes.extend(encoded_string("tiles"));
        bytes.extend([0; 16 + 2]); // 空栅格名称和选择子。
        bytes.extend(encoded_string("瓦片后缀"));

        bytes.extend(AIS_IMAGE_SERVICE_NAME_CLASS_ID);
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        for text in ["service", "server", ""] {
            bytes.extend(encoded_string(text));
        }
        bytes.extend(encoded_string("服务后缀"));
        bytes.extend(INTERNET_TILED_LAYER_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(encoded_string("回引后缀"));
        bytes.extend(AIS_IMAGE_SERVICE_NAME_CLASS_ID);
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(encoded_string(""));
        bytes.extend(SIMPLE_RASTER_RENDERER_CLASS_ID);
        bytes.extend(63_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(result.names_stop, None);
        let Some(RasterDatasetNameEntryBody10_1::InternetTiledLayer(tiled)) = &result.names[0].body
        else {
            panic!("应读取 IName 瓦片名称");
        };
        assert_eq!((tiled.name_string.as_str(), tiled.integer), ("瓦片", 7));
        assert!(matches!(
            tiled.tile_reader.body.as_deref(),
            Some(CacheTileReaderBody10_1::MapTile { path, .. }) if path == "tiles"
        ));
        assert!(matches!(
            &result.names[1].body,
            Some(RasterDatasetNameEntryBody10_1::AisImageService(name)) if name.strings[0] == "service"
        ));
        assert_eq!(
            result.names[2].entry,
            ObjectEntry10_1::BackReference {
                class_id: INTERNET_TILED_LAYER_NAME_CLASS_ID,
                key: 60,
            }
        );
        for (definition, reference, expected) in [(0, 2, "瓦片"), (1, 3, "service")] {
            let source = result.names[definition].source_reference.as_ref().unwrap();
            assert_eq!(source.name.as_ref(), expected);
            assert!(source.path.is_none());
            assert!(result.names[reference].body.is_none());
            assert_eq!(
                result.names[reference].source_reference.as_ref(),
                Some(source)
            );
        }
        assert!(matches!(
            result.renderer.unwrap().body,
            Some(RasterRendererPrefix10_1::Simple { .. })
        ));
        assert!(refs.contains(61));
        assert_eq!(refs.len(), 4);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn wms_and_wmts_names_share_layer_keys_and_reach_renderer() {
        use super::super::raster_renderer::{
            RasterRendererPrefix10_1, SIMPLE_RASTER_RENDERER_CLASS_ID,
        };
        use crate::layers::wcs_name::{
            WMS_CONNECTION_NAME_CLASS_ID, WMTS_CONNECTION_NAME_CLASS_ID,
        };
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        let mut bytes = body_prefix(17, 2);
        bytes.extend(WMS_CONNECTION_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(encoded_string("WMS"));

        bytes.extend(WMTS_CONNECTION_NAME_CLASS_ID);
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes()); // WMS 属性集回引。
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(63_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(encoded_string("WMTS"));

        bytes.extend(SIMPLE_RASTER_RENDERER_CLASS_ID);
        bytes.extend(64_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(result.names_stop, None);
        let Some(RasterDatasetNameEntryBody10_1::OgcConnection(wms)) = &result.names[0].body else {
            panic!("首项应为 WMSConnectionName");
        };
        assert_eq!(wms.class_id, WMS_CONNECTION_NAME_CLASS_ID);
        assert_eq!(wms.version, 1);
        assert_eq!(wms.properties.body.as_ref().unwrap().count, 0);
        assert!(wms.extra_properties.is_none());
        let Some(RasterDatasetNameEntryBody10_1::OgcConnection(wmts)) = &result.names[1].body
        else {
            panic!("次项应为 WMTSConnectionName");
        };
        assert_eq!(wmts.class_id, WMTS_CONNECTION_NAME_CLASS_ID);
        assert_eq!(wmts.version, 2);
        assert_eq!(
            wmts.properties.entry,
            ObjectEntry10_1::BackReference {
                class_id: PROPERTY_SET_CLASS_ID,
                key: 61,
            }
        );
        assert_eq!(
            wmts.extra_properties
                .as_ref()
                .map(|slot| slot.entry.clone()),
            Some(ObjectEntry10_1::NewObject {
                class_id: PROPERTY_SET_CLASS_ID,
                key: 63,
            })
        );
        assert_eq!(result.names[0].suffix.as_deref(), Some("WMS"));
        assert_eq!(result.names[1].suffix.as_deref(), Some("WMTS"));
        assert!(matches!(
            result.renderer.unwrap().body,
            Some(RasterRendererPrefix10_1::Simple { .. })
        ));
        assert_eq!(refs.len(), 5);
        for key in [60, 61, 62, 63, 64] {
            assert!(refs.contains(key));
        }
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn bounded_sai_renderer_reaches_raster_layer_tail() {
        use super::super::raster_renderer::{RasterRendererPrefix10_1, SAI_RENDERER_CLASS_ID};

        let mut bytes = body_prefix(1, 1);
        dataset_name(&mut bytes, 60);
        bytes.extend(encoded_string(""));
        let renderer_start = bytes.len();
        bytes.extend(SAI_RENDERER_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(17_u16.to_le_bytes());
        bytes.extend(0.75_f64.to_le_bytes());
        bytes.extend(19.0_f64.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes()); // 共用体 v1。
        bytes.extend([0; 16]);
        let renderer_end = bytes.len();
        bytes.extend(2.5_f64.to_le_bytes()); // 图层尾段。
        bytes.extend([0; 14]);

        let version_offset = renderer_start + 20;
        for version in [1_i32, 0, -1, i32::MIN] {
            bytes[version_offset..version_offset + 4].copy_from_slice(&version.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            match result.renderer.unwrap().body.unwrap() {
                RasterRendererPrefix10_1::Sai {
                    version: actual_version,
                    short,
                    doubles,
                    shared,
                } => {
                    assert_eq!(
                        (actual_version, short, doubles, shared.version),
                        (version, 17, [0.75, 19.0], 1)
                    );
                }
                other => panic!("预期 SAIRenderer，实际为 {other:?}"),
            }
            assert_eq!(result.tail.unwrap().doubles_bc_c4, [2.5, 0.0]);
            assert_eq!(reader.remaining(), 0);
            assert!(refs.contains(61));
        }

        bytes[version_offset..version_offset + 4].copy_from_slice(&2_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(result.renderer_stop.is_some());
        assert_eq!(reader.position(), renderer_start);
        assert!(refs.contains(60));
        assert!(!refs.contains(61));

        bytes[version_offset..version_offset + 4].copy_from_slice(&1_i32.to_le_bytes());
        for cut in renderer_start..renderer_end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.renderer_stop.is_some(), "截断位置 {cut}");
            assert_eq!(reader.position(), renderer_start);
            assert!(refs.contains(60));
            assert!(!refs.contains(61));
        }
    }

    #[test]
    fn band_name_reads_nested_collection_and_rolls_back_object_keys() {
        use super::super::raster_dataset_name::{
            RASTER_BAND_COLLECTION_NAME_CLASS_ID, RASTER_BAND_NAME_CLASS_ID,
            RasterBandNameSource10_1, RasterDatasetWorkspaceSlot10_1,
        };
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = body_prefix(1, 1);
        let name_start = bytes.len();
        bytes.extend(RASTER_BAND_NAME_CLASS_ID);
        bytes.extend(71_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        for value in ["目录\\首波段", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(RASTER_BAND_COLLECTION_NAME_CLASS_ID);
        bytes.extend(72_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("波段集合"));
        bytes.extend(RASTER_BAND_NAME_CLASS_ID);
        bytes.extend(73_u32.to_le_bytes());
        for value in ["旧名称", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(74_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["栅格目录", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 21]); // 空属性集、空工厂和尾 int32。
        bytes.extend([
            0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0x00, 0xc0, 0x4f, 0x8e,
            0xce, 0x27,
        ]);
        bytes.extend(75_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        let array_count_offset = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend((-5_i32).to_le_bytes());
        bytes.extend(14_i32.to_le_bytes());
        bytes.extend(encoded_string("图层名称后缀"));
        let name_end = bytes.len();
        bytes.extend([0; 16 + 22]); // 空渲染器与 v1 尾段。

        let assert_name_rollback = |input: &[u8]| {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.names_stop.is_some());
            assert_eq!(reader.position(), name_start);
            assert!(refs.is_empty());
        };
        for cut in name_start..name_end {
            assert_name_rollback(&bytes[..cut]);
        }
        let mut malformed = bytes.clone();
        malformed[array_count_offset..array_count_offset + 4]
            .copy_from_slice(&i32::MAX.to_le_bytes());
        assert_name_rollback(&malformed);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.names[0].suffix.as_deref(), Some("图层名称后缀"));
        let Some(RasterDatasetNameEntryBody10_1::Band(band)) = &parsed.names[0].body else {
            panic!("预期 RasterBandName");
        };
        assert!(band.has_version_marker);
        assert_eq!(band.band_lookup_name(), "首波段");
        let RasterBandNameSource10_1::Dataset(dataset) = &band.source else {
            panic!("预期 IDatasetName");
        };
        let Some(RasterDatasetNameEntryBody10_1::BandCollection(collection)) = &dataset.body else {
            panic!("预期 RasterBandCollectionName");
        };
        assert_eq!(collection.array, Some((1, vec![-5, 14])));
        let Some(RasterDatasetNameEntryBody10_1::Band(legacy_band)) = &collection.name.body else {
            panic!("预期内嵌 RasterBandName");
        };
        assert!(!legacy_band.has_version_marker);
        assert_eq!(legacy_band.band_lookup_name(), "旧名称");
        let legacy_source = refs.dataset_source(RASTER_BAND_NAME_CLASS_ID, 73).unwrap();
        assert_eq!(legacy_source.path.as_deref(), Some("栅格目录"));
        assert_eq!(legacy_source.band_name.as_deref(), Some("旧名称"));
        assert!(legacy_source.name.is_empty());
        assert!(matches!(
            legacy_band.source,
            RasterBandNameSource10_1::Workspace(RasterDatasetWorkspaceSlot10_1::WorkspaceName(_))
        ));
        assert!(parsed.tail.is_some());
        assert_eq!(refs.len(), 5);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn band_collection_nesting_allows_back_reference_at_depth_limit() {
        use super::super::raster_dataset_name::RASTER_BAND_COLLECTION_NAME_CLASS_ID as CLASS_ID;

        for count in [16_u32, 17] {
            let mut bytes = body_prefix(1, 1);
            let name_start = bytes.len();
            for key in 71..71 + count {
                bytes.extend(CLASS_ID);
                bytes.extend(key.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(encoded_string(""));
            }
            bytes.extend(CLASS_ID);
            bytes.extend(71_u32.to_le_bytes());
            bytes.resize(bytes.len() + 16 * count as usize, 0); // 每层 ILongArray 为空。
            bytes.extend(encoded_string(""));
            bytes.extend([0; 16 + 22]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            if count == 16 {
                assert!(parsed.tail.is_some());
                assert_eq!(refs.len(), count as usize);
                assert_eq!(reader.remaining(), 0);
            } else {
                assert!(parsed.names_stop.unwrap().contains("名称嵌套超过限额"));
                assert!(refs.is_empty());
                assert_eq!(reader.position(), name_start);
            }
        }
    }

    #[test]
    fn inline_raster_v12_accepts_image_service_without_inner_name() {
        use super::super::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
        use super::super::raster_dataset_name::IS_RASTER_DATASET_NAME_CLASS_ID;

        let mut standard_prefix = body_prefix(12, 1);
        dataset_name(&mut standard_prefix, 60);
        let standard = raster_with_empty_slots(12, false);
        let mut bytes = body_prefix(12, 1);
        bytes.extend(IS_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for _ in 0..6 {
            bytes.extend(encoded_string(""));
        }
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0; 16]); // 空工作空间。
        let inner_offset = bytes.len();
        bytes.extend([0; 16]);
        bytes.extend(&standard[standard_prefix.len()..]);
        let end = bytes.len();
        bytes.extend([0xa5; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(raster.tail.unwrap().geodata_xform_v13.is_none());
        assert_eq!(reader.position(), end);
        assert!(refs.raster_name_opens_without_result(60));

        let mut inner = IMAGE_SERVER_NAME_CLASS_ID.to_vec();
        inner.extend(61_u32.to_le_bytes());
        inner.extend(1_u16.to_le_bytes());
        inner.extend(encoded_string(""));
        inner.extend([0; 16]);
        bytes.splice(inner_offset..inner_offset + 16, inner);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let error = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn image_service_name_reads_name_and_transform_with_shared_keys() {
        use super::super::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
        use super::super::raster_dataset_name::IS_RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID;
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        for version in [i16::MIN, -1, 0, 1, 2, 3, 4] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            bytes.extend(IS_RASTER_DATASET_NAME_CLASS_ID);
            bytes.extend(72_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for text in ["服务栅格", "", "", "", "", ""] {
                bytes.extend(encoded_string(text));
            }
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend([0; 16]); // 空工作空间。
            bytes.extend(IMAGE_SERVER_NAME_CLASS_ID);
            bytes.extend(73_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded_string("影像服务"));
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(74_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            if version >= 2 {
                bytes.extend(encoded_string("版本 2"));
            }
            if version >= 3 {
                bytes.extend(encoded_string("版本 3"));
            }
            if version >= 4 {
                bytes.extend(IDENTITY_XFORM_CLASS_ID);
                bytes.extend(75_u32.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend([0; 16]);
            }
            bytes.extend(encoded_string("图层后缀"));
            let end = bytes.len();
            bytes.extend([0; 16 + 22]);
            let mut unsupported = bytes.clone();
            unsupported[start + 20..start + 22].copy_from_slice(&5_u16.to_le_bytes());
            let mut unsupported_reader = Reader::new(&unsupported);
            let mut unsupported_refs = RefTable10_1::new();
            let unsupported_result =
                read_bounded_raster_body_10_1(&mut unsupported_reader, 4096, &mut unsupported_refs)
                    .unwrap();
            assert!(unsupported_result.names_stop.is_some());
            assert_eq!(unsupported_reader.position(), start);
            assert!(unsupported_refs.is_empty());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::ImageService(name)) = &result.names[0].body
            else {
                panic!("预期 ISRasterDatasetName");
            };
            let Some(RasterDatasetNameEntryBody10_1::ImageServerReference(server)) =
                &name.name.body
            else {
                panic!("预期 ImageServerName");
            };
            assert_eq!(server.text, "影像服务");
            assert!(server.new_properties.is_some());
            assert_eq!(name.version, version as u16);
            assert_eq!(name.long_44, 7);
            assert_eq!(
                name.string_50.as_deref(),
                (version >= 2).then_some("版本 2")
            );
            assert_eq!(
                name.string_54.as_deref(),
                (version >= 3).then_some("版本 3")
            );
            assert_eq!(name.transform.is_some(), version == 4);
            assert!((72..=74).all(|key| refs.contains(key)));
            assert_eq!(refs.contains(75), version == 4);
            assert!(result.tail.is_some());
            assert_eq!(reader.remaining(), 0);
            for cut in start..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(result.names_stop.is_some());
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn raster_name_and_transform_share_the_recursion_limit() {
        use super::super::raster_dataset_name::IS_RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::geodata_xform::RPC_XFORM_CLASS_ID;

        for count in [2_u32, 20] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            for level in 0..count {
                bytes.extend(IS_RASTER_DATASET_NAME_CLASS_ID);
                bytes.extend((100 + level * 2).to_le_bytes());
                bytes.extend(4_u16.to_le_bytes());
                for _ in 0..6 {
                    bytes.extend(encoded_string(""));
                }
                bytes.extend([0; 4 + 16 + 16]); // int32、工作空间、IName。
                bytes.extend(encoded_string(""));
                bytes.extend(encoded_string(""));
                if level + 1 == count {
                    bytes.extend([0; 16]); // 末级空变换。
                } else {
                    bytes.extend(RPC_XFORM_CLASS_ID);
                    bytes.extend((101 + level * 2).to_le_bytes());
                    bytes.extend(2_i32.to_le_bytes());
                    bytes.extend([0; 0x2d0 + 16 + 24 + 4]); // RPC 参数、空内层变换、标量。
                }
            }
            bytes.resize(bytes.len() + (count as usize - 1) * (4 + 16 + 16), 0);
            bytes.extend(encoded_string(""));
            bytes.extend([0; 16 + 22]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            if count == 2 {
                assert!(result.tail.is_some());
                assert_eq!(refs.len(), 3);
                assert_eq!(reader.remaining(), 0);
            } else {
                assert!(result.names_stop.unwrap().contains("嵌套"));
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn function_raster_name_reads_legacy_single_and_modern_counted_inputs() {
        use super::super::image_server_rules::RasterTemplateInput10_1;
        use super::super::image_server_rules::{
            CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID, IDENTITY_FUNCTION_CLASS_ID,
        };
        use super::super::raster_dataset_name::{
            FUNCTION_RASTER_DATASET_NAME_CLASS_ID, FunctionRasterInput10_1,
        };
        use crate::objects::property_set::{
            PROPERTY_SET_CLASS_ID, PropertyValueObjectStream10_1, RASTER_SCALAR_CLASS_ID,
        };

        for version in [i16::MIN, -1, 1, 4, 7] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            bytes.extend(FUNCTION_RASTER_DATASET_NAME_CLASS_ID);
            bytes.extend(80_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for text in ["函数栅格", "", ""] {
                bytes.extend(encoded_string(text));
            }
            bytes.extend([0; 16]); // 空工作空间名称。
            if version < 4 {
                bytes.extend(encoded_string("旧版字段"));
            }
            bytes.extend(encoded_string("栅格.afr"));
            if version < 4 {
                bytes.extend(12_i32.to_le_bytes());
                bytes.extend([0; 16]); // 旧版 IName。
            }
            let function_offset = bytes.len();
            bytes.extend(IDENTITY_FUNCTION_CLASS_ID);
            bytes.extend(81_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(9_i32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            if version >= 2 {
                bytes.extend((if version == 7 { 4_i32 } else { 0 }).to_le_bytes());
            }
            bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
            bytes.extend(82_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 16]);
            if version == 7 {
                bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
                bytes.extend(82_u32.to_le_bytes());
                bytes.extend(RASTER_SCALAR_CLASS_ID);
                bytes.extend(84_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 16]); // 空 Variant。
                bytes.extend(RASTER_SCALAR_CLASS_ID);
                bytes.extend(84_u32.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(83_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
            }
            if (3..=4).contains(&version) {
                bytes.extend([0; 16]);
            }
            if version >= 6 {
                bytes.extend(encoded_string("附加字段"));
            }
            if version >= 7 {
                bytes.extend(encoded_string("末尾字段"));
            }
            bytes.extend(encoded_string("图层后缀"));
            let end = bytes.len();
            bytes.extend([0; 16 + 22]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::Function(name)) = &result.names[0].body else {
                panic!("预期 FunctionRasterDatasetName");
            };
            assert_eq!(name.version as i16, version);
            assert_eq!(name.path, "栅格.afr");
            assert_eq!(name.function.body.as_ref().unwrap().base_long, 9);
            assert!(matches!(
                name.inputs[0].body.as_deref(),
                Some(FunctionRasterInput10_1::Arguments(_))
            ));
            assert_eq!(
                name.legacy.as_ref().map(|legacy| legacy.long),
                (version < 4).then_some(12)
            );
            assert_eq!(name.properties.is_some(), version >= 3);
            assert_eq!(name.geometry.is_some(), version == 4);
            assert_eq!(
                name.string_48.as_deref(),
                (version >= 7).then_some("末尾字段")
            );
            if version == 7 {
                assert!(matches!(
                    name.inputs[1].entry,
                    ObjectEntry10_1::BackReference { key: 82, .. }
                ));
                assert!(matches!(
                    name.inputs[2].body.as_deref(),
                    Some(FunctionRasterInput10_1::Other(body))
                        if matches!(**body, RasterTemplateInput10_1::NativeArgument(
                            PropertyValueObjectStream10_1::RasterScalar { key: 84, .. }
                        ))
                ));
                assert_eq!(
                    name.inputs[3].entry,
                    ObjectEntry10_1::BackReference {
                        class_id: RASTER_SCALAR_CLASS_ID,
                        key: 84,
                    }
                );
            }
            assert!((80..=82).all(|key| refs.contains(key)));
            assert_eq!(refs.contains(83), version >= 3);
            assert!(result.tail.is_some());
            assert_eq!(reader.remaining(), 0);
            let mut wrong = bytes.clone();
            wrong[function_offset + 16..function_offset + 20]
                .copy_from_slice(&80_u32.to_le_bytes());
            let mut reader = Reader::new(&wrong);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.names.is_empty());
            assert_eq!(reader.position(), start);
            assert!(refs.is_empty());

            for cut in start..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(result.names_stop.is_some());
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn cache_raster_name_reference_retains_workspace_and_dataset() {
        use super::super::raster_dataset_name::CACHE_RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = body_prefix(9, 2);
        bytes.extend(CACHE_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("cache_dataset"));
        bytes.extend(encoded_string("显示名称"));
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["cache", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 21]); // 空属性集、空工厂及尾整数。
        bytes.extend([0; 32]); // 空瓦片读取器及包络。
        bytes.extend(encoded_string(""));
        bytes.extend(CACHE_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(encoded_string(""));

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(parsed.names_stop.is_none());
        assert_eq!(parsed.names.len(), 2);
        let source = parsed.names[0].source_reference.as_ref().unwrap();
        assert_eq!(source.name.as_ref(), "cache_dataset");
        assert_eq!(source.path.as_deref(), Some("cache"));
        assert!(parsed.names[1].body.is_none());
        assert_eq!(parsed.names[1].source_reference.as_ref(), Some(source));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn inline_raster_v12_accepts_empty_image_server_name() {
        use super::super::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;

        let mut standard_prefix = body_prefix(12, 1);
        dataset_name(&mut standard_prefix, 60);
        let standard = raster_with_empty_slots(12, false);

        for text in ["", "service"] {
            let mut bytes = body_prefix(12, 1);
            bytes.extend(IMAGE_SERVER_NAME_CLASS_ID);
            bytes.extend(60_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded_string(text));
            bytes.extend([0; 16]);
            bytes.extend(&standard[standard_prefix.len()..]);
            let end = bytes.len();
            bytes.extend([0xa5; 16]);

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let result = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs);
            if text.is_empty() {
                assert!(result.unwrap().tail.unwrap().geodata_xform_v13.is_none());
                assert_eq!(reader.position(), end);
                assert!(refs.raster_name_opens_without_result(60));
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn inline_raster_v12_accepts_wcs_coverage_without_connection() {
        use crate::layers::wcs_name::{WCS_CONNECTION_NAME_CLASS_ID, WCS_COVERAGE_NAME_CLASS_ID};

        let mut standard_prefix = body_prefix(12, 1);
        dataset_name(&mut standard_prefix, 60);
        let standard = raster_with_empty_slots(12, false);

        let mut bytes = body_prefix(12, 1);
        bytes.extend(WCS_COVERAGE_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(u16::from(b'a').to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
        let connection_offset = bytes.len();
        bytes.extend([0; 16]);
        bytes.extend(&standard[standard_prefix.len()..]);
        let end = bytes.len();
        bytes.extend([0xa5; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(raster.tail.unwrap().geodata_xform_v13.is_none());
        assert_eq!(reader.position(), end);
        assert!(refs.raster_name_opens_without_result(60));

        let mut connection = WCS_CONNECTION_NAME_CLASS_ID.to_vec();
        connection.extend(61_u32.to_le_bytes());
        connection.extend(1_i16.to_le_bytes());
        connection.extend([0; 16]);
        bytes.splice(connection_offset..connection_offset + 16, connection);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn wcs_coverage_name_reference_retains_identifier() {
        use crate::layers::wcs_name::WCS_COVERAGE_NAME_CLASS_ID;

        let mut bytes = body_prefix(9, 2);
        bytes.extend(WCS_COVERAGE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        let identifier = "coverage_a";
        bytes.extend((identifier.len() as u32).to_le_bytes());
        for unit in identifier.encode_utf16() {
            bytes.extend(unit.to_le_bytes());
        }
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend([0; 16]); // 空连接名称。
        bytes.extend(encoded_string(""));
        bytes.extend(WCS_COVERAGE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(encoded_string(""));

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(parsed.names_stop.is_none());
        assert_eq!(parsed.names.len(), 2);
        let source = parsed.names[0].source_reference.as_ref().unwrap();
        assert_eq!(source.name.as_ref(), identifier);
        assert!(source.path.is_none());
        assert!(parsed.names[1].body.is_none());
        assert_eq!(parsed.names[1].source_reference.as_ref(), Some(source));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn inline_raster_v12_accepts_netcdf_name_without_workspace() {
        use super::super::raster_dataset_name::NETCDF_RASTER_DATASET_NAME_CLASS_ID;

        let mut standard_prefix = body_prefix(12, 1);
        dataset_name(&mut standard_prefix, 60);
        let standard = raster_with_empty_slots(12, false);

        let mut bytes = body_prefix(12, 1);
        bytes.extend(NETCDF_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        let name_body = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        for _ in 0..8 {
            bytes.extend(encoded_string(""));
        }
        bytes.extend([0; 16 * 3]); // 两个数组和工作空间名称均为空。
        let name_end = bytes.len();
        bytes.extend(&standard[standard_prefix.len()..]);
        let end = bytes.len();
        bytes.extend([0xa5; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(raster.tail.unwrap().geodata_xform_v13.is_none());
        assert_eq!(reader.position(), end);
        assert!(refs.raster_name_opens_without_result(60));

        bytes.drain(name_body..name_end);
        let mut reader = Reader::new(&bytes);
        read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(reader.position(), end - (name_end - name_body));
    }

    #[test]
    fn netcdf_name_keeps_array_keys_and_respects_v1_envelope_gate() {
        use super::super::raster_dataset_name::{
            LONG_ARRAY_CLASS_ID, NETCDF_RASTER_DATASET_NAME_CLASS_ID,
        };
        use crate::objects::envelope::ENVELOPE_CLASS_ID;
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        for version in [1_i32, 2] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            bytes.extend(NETCDF_RASTER_DATASET_NAME_CLASS_ID);
            bytes.extend(76_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for value in ["首串", "", "", "", "栅格变量", "", "", ""] {
                bytes.extend(encoded_string(value));
            }
            if version == 1 {
                bytes.extend(encoded_string("旧版字段"));
                bytes.extend(ENVELOPE_CLASS_ID);
                bytes.extend(79_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 32]);
            }
            bytes.extend(STR_ARRAY_CLASS_ID);
            bytes.extend(77_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(encoded_string("时间"));
            bytes.extend(LONG_ARRAY_CLASS_ID);
            bytes.extend(78_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            let count_offset = bytes.len();
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(2026_i32.to_le_bytes());
            bytes.extend(WORKSPACE_NAME_CLASS_ID);
            bytes.extend(80_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            for value in ["climate.nc", "", ""] {
                bytes.extend(encoded_string(value));
            }
            bytes.extend([0; 21]); // 空属性集、空工厂及尾整数。
            bytes.extend(encoded_string("图层名称后缀"));
            let end = bytes.len();
            bytes.extend([0; 16 + 22]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::NetCdf(name)) = &result.names[0].body else {
                panic!("预期 NetCDFRasterDatasetName");
            };
            assert_eq!(name.strings[0], "首串");
            assert_eq!(name.variable(), "栅格变量");
            let source = result.names[0].source_reference.as_ref().unwrap();
            assert_eq!(source.name.as_ref(), "栅格变量");
            assert_eq!(source.path.as_deref(), Some("climate.nc"));
            assert_eq!(
                refs.dataset_source(NETCDF_RASTER_DATASET_NAME_CLASS_ID, 76),
                Some(source)
            );
            assert_eq!(name.legacy.is_some(), version == 1);
            assert_eq!(name.labels.body.as_ref().unwrap().1, ["时间"]);
            assert_eq!(name.values.body, Some((1, vec![2026])));
            assert!(refs.contains(76) && refs.contains(77) && refs.contains(78));
            assert_eq!(refs.contains(79), version == 1);
            assert!(result.tail.is_some());
            assert_eq!(reader.remaining(), 0);

            let mut reference = body_prefix(1, 1);
            reference.extend(NETCDF_RASTER_DATASET_NAME_CLASS_ID);
            reference.extend(76_u32.to_le_bytes());
            reference.extend(encoded_string(""));
            reference.extend([0; 16 + 22]);
            let mut reader = Reader::new(&reference);
            let referenced = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(referenced.names[0].body.is_none());
            assert_eq!(referenced.names[0].source_reference.as_ref(), Some(source));
            assert_eq!(reader.remaining(), 0);

            let mut invalid = bytes.clone();
            invalid[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            for input in (start..end)
                .map(|cut| &bytes[..cut])
                .chain([invalid.as_slice()])
            {
                let mut reader = Reader::new(input);
                let mut refs = RefTable10_1::new();
                let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(result.names_stop.is_some());
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn sde_name_reads_tag6_without_a_key_and_shares_workspace_key() {
        use super::super::raster_dataset_name::{
            DatabaseRasterTag6Name10_1, RASTER_DATASET_NAME_CLASS_ID,
            SDE_RASTER_DATASET_NAME_CLASS_ID,
        };
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = body_prefix(1, 1);
        let delegator = bytes.len();
        dataset_name(&mut bytes, 76);
        bytes[delegator..delegator + 16].copy_from_slice(&SDE_RASTER_DATASET_NAME_CLASS_ID);
        let record = |bytes: &mut Vec<u8>, tag: i32, data: &[u8]| {
            bytes.extend(tag.to_le_bytes());
            bytes.extend((data.len() as i32).to_le_bytes());
            bytes.extend(data);
        };
        record(&mut bytes, 0xff00fa, &[]);
        record(&mut bytes, 1, &33_i32.to_le_bytes());
        record(&mut bytes, 2, &[0; 2]);
        record(&mut bytes, 3, &[0; 2]);
        record(&mut bytes, 4, &12_i32.to_le_bytes());
        record(&mut bytes, 5, &[0; 2]);
        bytes.extend(6_i32.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        let tag6 = bytes.len();
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(2_i32.to_le_bytes());
        for value in ["嵌套栅格", "", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["工作空间", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 21]); // 空属性集、空工厂和尾 int32。
        bytes.extend([0xff; 8]); // SDE 到 tag 6 后即读取结束对。
        bytes.extend(encoded_string("图层名称后缀"));
        bytes.extend([0; 16 + 22]);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let Some(RasterDatasetNameEntryBody10_1::Database(name)) = &result.names[0].body else {
            panic!("预期 SdeRasterDatasetName");
        };
        assert_eq!(name.delegated_name.key, 76);
        let records = name.records.as_ref().unwrap();
        let Some(DatabaseRasterTag6Name10_1::Raster {
            body: RasterDatasetNameEntryBody10_1::Dataset(nested),
            ..
        }) = &records.tag6_name
        else {
            panic!("预期无键 tag 6 栅格名称");
        };
        assert_eq!(nested.key, 0);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(76) && refs.contains(77));
        assert!(result.tail.is_some());
        assert_eq!(reader.remaining(), 0);

        bytes[tag6] ^= 0xff;
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(result.names_stop.is_some());
        assert_eq!(reader.position(), delegator);
        assert!(refs.is_empty());
    }

    #[test]
    fn raster_type_name_reads_mosaic_name_and_preserves_the_next_slot() {
        use super::super::raster_dataset_name::{
            MOSAIC_DATASET_NAME_CLASS_ID, RASTER_TYPE_NAME_CLASS_ID,
        };

        let mut bytes = body_prefix(17, 2);
        let name_start = bytes.len();
        bytes.extend(RASTER_TYPE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("名称串"));
        bytes.extend(encoded_string("栅格类型"));
        bytes.extend((-12_i32).to_le_bytes());
        bytes.extend(MOSAIC_DATASET_NAME_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("镶嵌源"));
        bytes.extend(encoded_string("数据集"));
        bytes.extend([0; 16]); // 空工作空间。
        bytes.extend(encoded_string("首名称后缀"));
        let next_name = bytes.len();
        bytes.extend(RASTER_TYPE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(encoded_string("回引后缀"));
        bytes.extend([0; 16]); // 空渲染器。

        for cut in name_start..next_name {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.names_stop.is_some());
            assert_eq!(reader.position(), name_start);
            assert!(refs.is_empty());
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(parsed.names_stop.is_none());
        let Some(RasterDatasetNameEntryBody10_1::RasterType(name)) = &parsed.names[0].body else {
            panic!("预期 RasterTypeName");
        };
        assert_eq!(name.version, 1);
        assert_eq!(name.name_string, "名称串");
        assert_eq!(name.name, "栅格类型");
        assert_eq!(name.instance_id, -12);
        assert_eq!(
            name.mosaic_dataset.body.as_ref().unwrap().strings,
            ["镶嵌源", "数据集"]
        );
        assert_eq!(parsed.names[0].suffix.as_deref(), Some("首名称后缀"));
        assert_eq!(
            parsed.names[1].entry,
            ObjectEntry10_1::BackReference {
                class_id: RASTER_TYPE_NAME_CLASS_ID,
                key: 76,
            }
        );
        assert_eq!(parsed.names[1].suffix.as_deref(), Some("回引后缀"));
        assert!(refs.contains(76));
        assert!(refs.contains(77));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn mosaic_dataset_name_reads_versioned_fields_and_rolls_back_keys() {
        use super::super::raster_dataset_name::MOSAIC_DATASET_NAME_CLASS_ID;
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;

        for version in [i16::MIN, -1, 0, 1, 2, 3, 4] {
            let mut bytes = body_prefix(1, 1);
            let name_start = bytes.len();
            bytes.extend(MOSAIC_DATASET_NAME_CLASS_ID);
            bytes.extend(76_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for value in ["镶嵌数据集", "数据表"] {
                bytes.extend(encoded_string(value));
            }
            bytes.extend(crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID);
            bytes.extend(78_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            for value in ["mosaic.gdb", "", ""] {
                bytes.extend(encoded_string(value));
            }
            bytes.extend([0; 21]); // 空属性集、空工厂及尾整数。
            if version >= 2 {
                bytes.extend(7_u16.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend((-12_i32).to_le_bytes());
            }
            let array_start = bytes.len();
            if version >= 4 {
                bytes.extend(STR_ARRAY_CLASS_ID);
                bytes.extend(77_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                for value in ["名称甲", "名称乙"] {
                    bytes.extend(encoded_string(value));
                }
            }
            bytes.extend(encoded_string("图层后缀"));
            let name_end = bytes.len();
            bytes.extend([0; 16 + 22]); // 空渲染器与 v1 尾段。

            let assert_rollback = |input: &[u8]| {
                let mut reader = Reader::new(input);
                let mut refs = RefTable10_1::new();
                let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(parsed.names_stop.is_some());
                assert_eq!(reader.position(), name_start);
                assert!(refs.is_empty());
            };
            let mut unsupported = bytes.clone();
            unsupported[name_start + 20..name_start + 22].copy_from_slice(&5_u16.to_le_bytes());
            assert_rollback(&unsupported);
            for cut in name_start..name_end {
                assert_rollback(&bytes[..cut]);
            }
            if version >= 4 {
                let mut malformed = bytes.clone();
                malformed[array_start + 22..array_start + 26]
                    .copy_from_slice(&i32::MAX.to_le_bytes());
                assert_rollback(&malformed);
            }

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::Mosaic(name)) = &parsed.names[0].body else {
                panic!("预期 MosaicDatasetName");
            };
            assert_eq!(name.version, version as u16);
            assert_eq!(name.strings, ["镶嵌数据集", "数据表"]);
            let source = parsed.names[0].source_reference.as_ref().unwrap();
            assert_eq!(source.name.as_ref(), "镶嵌数据集");
            assert_eq!(source.path.as_deref(), Some("mosaic.gdb"));

            let mut reference = body_prefix(1, 1);
            reference.extend(MOSAIC_DATASET_NAME_CLASS_ID);
            reference.extend(76_u32.to_le_bytes());
            reference.extend(encoded_string(""));
            reference.extend([0; 16 + 22]);
            let mut reference_reader = Reader::new(&reference);
            let referenced =
                read_bounded_raster_body_10_1(&mut reference_reader, 4096, &mut refs).unwrap();
            assert!(referenced.names[0].body.is_none());
            assert_eq!(referenced.names[0].source_reference.as_ref(), Some(source));
            assert_eq!(reference_reader.remaining(), 0);
            assert_eq!(name.short_44, (version >= 2).then_some(7));
            assert_eq!(name.long_48, (version >= 3).then_some(-12));
            if version >= 4 {
                let array = name.string_array.as_ref().unwrap();
                let (array_version, strings) = array.body.as_ref().unwrap();
                assert_eq!(*array_version, 1);
                assert_eq!(strings, &["名称甲", "名称乙"]);
            } else {
                assert!(name.string_array.is_none());
            }
            assert!(refs.contains(76));
            assert_eq!(refs.contains(77), version >= 4);
            assert_eq!(parsed.names[0].suffix.as_deref(), Some("图层后缀"));
            assert!(parsed.tail.is_some());
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn catalog_name_reuses_feature_class_body_without_a_second_header() {
        use super::super::raster_dataset_name::RASTER_CATALOG_NAME_CLASS_ID;

        let mut bytes = body_prefix(1, 1);
        let name_start = bytes.len();
        bytes.extend(RASTER_CATALOG_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        for value in ["栅格目录名", "", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 8 + 2 + 16 + 2]); // 两个 int32、选择子、空工作空间、选择子。
        bytes.extend(encoded_string("图层后缀"));
        let name_end = bytes.len();
        bytes.extend([0; 16 + 22]);
        for cut in name_start..name_end {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..cut]);
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.names_stop.is_some());
            assert_eq!(reader.position(), name_start);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let Some(RasterDatasetNameEntryBody10_1::Catalog(catalog)) = &parsed.names[0].body else {
            panic!("预期 RasterCatalogName");
        };
        assert_eq!(catalog.name, "栅格目录名");
        assert_eq!(catalog.key, 76);
        assert_eq!(parsed.names[0].suffix.as_deref(), Some("图层后缀"));
        assert!(parsed.tail.is_some());
        assert_eq!(refs.len(), 1);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn dataset_name83_leaves_layer_suffix_after_optional_records() {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME83_CLASS_ID;

        for has_records in [false, true] {
            let mut bytes = body_prefix(1, 1);
            let name_start = bytes.len();
            dataset_name(&mut bytes, 62);
            bytes[name_start..name_start + 16].copy_from_slice(&RASTER_DATASET_NAME83_CLASS_ID);
            if has_records {
                for value in [0x00ff_00fa_i32, 0, 1, 4, 27, 2] {
                    bytes.extend(value.to_le_bytes());
                }
                bytes.extend(encoded_string("栅格"));
                bytes.extend([0xff; 8]);
            }
            bytes.extend(encoded_string("后继名称"));
            bytes.extend([0; 16 + 22]); // 空渲染器与 v1 尾段标量。
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::Dataset83 { records, .. }) =
                &result.names[0].body
            else {
                panic!("预期 RasterDatasetName83");
            };
            assert_eq!(
                records
                    .as_ref()
                    .map(|value| (value.first_long, value.text.as_str())),
                has_records.then_some((27, "栅格")),
            );
            assert_eq!(result.names[0].suffix.as_deref(), Some("后继名称"));
            assert!(result.tail.is_some());
            assert!(refs.contains(62));
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn bounded_older_names_consume_versioned_legacy_fields_and_suffix() {
        for (version, selector) in [(0, 0_i32), (4, 0), (5, 0), (8, -1), (9, 0)] {
            let mut bytes = body_prefix(version, 1);
            dataset_name(&mut bytes, 60);
            if version > 0 {
                bytes.extend(encoded_string(""));
            }
            if (5..=8).contains(&version) {
                bytes.extend(selector.to_le_bytes());
                if selector != 0 {
                    bytes.extend(91_i32.to_le_bytes());
                }
            }
            bytes.extend([0; 16]); // 空渲染器，无键。
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.names_stop.is_none());
            assert_eq!(
                result.names[0].suffix.as_deref(),
                (version > 0).then_some("")
            );
            assert_eq!(
                result.names[0].legacy,
                (5..=8)
                    .contains(&version)
                    .then_some((selector, (selector != 0).then_some(91)))
            );
            assert_eq!(result.renderer.unwrap().entry, ObjectEntry10_1::Null);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn failed_name_slot_preserves_earlier_keys_and_does_not_reach_renderer() {
        for unknown in [false, true] {
            let mut bytes = body_prefix(17, 2);
            dataset_name(&mut bytes, 60);
            bytes.extend(encoded_string("可读项"));
            let stop = bytes.len();
            if unknown {
                bytes.extend([0x9c; 16]);
                bytes.extend(62_u32.to_le_bytes());
                bytes.extend(encoded_string(""));
                bytes.extend([0; 16]); // 不能越过未知体去读渲染器。
            } else {
                dataset_name(&mut bytes, 62);
                bytes.extend(42_u32.to_le_bytes()); // 后缀串截断，连同该名称键回退。
            }
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(result.names.len(), 1);
            assert!(result.names_stop.is_some());
            assert!(result.renderer.is_none());
            assert_eq!(reader.position(), stop);
            assert!(refs.contains(60) && !refs.contains(62));
        }
    }

    #[test]
    fn zero_negative_and_all_null_names_do_not_reach_renderer() {
        for count in [0, -1, 1, i32::MAX] {
            let mut bytes = body_prefix(17, count);
            if count == 1 {
                bytes.extend([0; 16]);
                bytes.extend(encoded_string(""));
            }
            let end = bytes.len();
            bytes.extend([0; 16]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.names_stop.is_some());
            assert!(result.renderer.is_none() && refs.is_empty());
            assert_eq!(reader.position(), end);
        }
    }
}
