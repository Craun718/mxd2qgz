//! RasterLayer.dll 10.1 在 IRasterRenderer 后的对象体。原生读写与取证偏移见
//! docs/mxd-10.1-raster-layer-tail.md。

use crate::layers::annotate_props::{
    EDIT_TEMPLATE_MANAGER_CLASS_ID, EditTemplateManagerBody10_1,
    read_edit_template_manager_body_10_1,
};
use crate::layers::dep_tail::{
    TimeDataSegment10_1, TimeDimensionSegment10_1, TimeDisplaySegment10_1,
    read_time_data_segment_10_1, read_time_dimension_segment_10_1, read_time_display_segment_10_1,
};
use crate::layers::globe_layer_properties::{
    GLOBE_LAYER_PROPERTIES_CLASS_ID, GlobeLayerProperties10_1, read_bounded_globe_layer_properties,
};
pub use crate::layers::hatch_layer_extension::HatchLayerExtension10_1;
use crate::layers::hatch_layer_extension::{
    HATCH_LAYER_EXTENSION_CLASS_ID, read_bounded_hatch_layer_extension,
};
use crate::layers::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
use crate::layers::layer_extensions::{
    DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID, E2_LAYER_EXTENSION_VALUES_CLASS_ID,
    FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID, LEGACY_EXTENSION_HEADER_SIZE,
    REPRESENTATION_LAYER_EXTENSION_CLASS_ID, SERVER_LAYER_EXTENSION_CLASS_ID,
    read_bounded_dynamic_tiled_layer_properties, read_bounded_e2_layer_extension_values,
    read_bounded_extension_property_set, read_bounded_feature_service_layer_edit_extension,
    read_bounded_representation_layer_extension, read_bounded_server_layer_extension,
};
pub use crate::layers::layer_extensions::{
    DynamicTiledLayerProperties10_1, E2LayerExtensionValues10_1,
    FeatureServiceLayerEditExtension10_1, ServerLayerExtension10_1,
};
use crate::layers::raster_dataset_name::{
    RasterDatasetNameEntryBody10_1, RasterDatasetWorkspaceSlot10_1,
    read_supported_raster_name_body_10_1, read_supported_raster_name_entry_10_1,
};
use crate::layers::raster_renderer::{
    PANSHARPENING_FILTER_CLASS_ID, PansharpeningFilter10_1, RasterHistogram10_1,
    STATS_HISTOGRAM_CLASS_ID, read_pansharpening_filter, read_stats_histogram_body_10_1,
};
pub use crate::layers::route_layer_extension::RouteLayerExtension10_1;
use crate::layers::route_layer_extension::{
    ROUTE_LAYER_EXTENSION_CLASS_ID, read_bounded_route_layer_extension,
};
use crate::layers::scene_layer_properties::{
    SceneLayerProperties10_1, is_scene_layer_properties, read_bounded_scene_layer_properties,
};
pub use crate::layers::time_layer_track_extension::TimeLayerTrackExtension10_1;
use crate::layers::time_layer_track_extension::{
    TIME_LAYER_TRACK_EXTENSION_CLASS_ID, read_bounded_time_layer_track_extension,
};
pub use crate::layers::tracking_filter_layer_extension::TrackingFilterLayerExtension10_1;
use crate::layers::tracking_filter_layer_extension::{
    TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID, read_bounded_tracking_filter_layer_extension,
};
use crate::maps::standalone_table::{
    OBJECT_CLASS_NAME_CLASS_ID, TABLE_NAME_CLASS_ID, TableNameBody10_1,
};
use crate::objects::dataset_name_more::{
    DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1, MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID,
    MemoryRelationshipClassNameBody10_1, RELATIONSHIP_CLASS_NAME_CLASS_ID,
    RelationshipClassNameBody10_1, read_memory_relationship_class_name_body_10_1,
    read_relationship_class_name_body_10_1, read_tabular_name_slot_10_1,
};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
};
use crate::objects::geodata_xform::{
    GeodataXformSlot10_1, PROJECTIVE_XFORM_CLASS_ID, read_bounded_projective_candidate,
    read_geodata_xform_slot_10_1,
};
use crate::objects::property_set::{PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1};
use crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use crate::objects::spatial_reference_slots::{
    GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

const LUT_FILTER_CLASS_ID: [u8; 16] = [
    0x58, 0xfc, 0x25, 0xe8, 0x1a, 0x15, 0xc6, 0x4b, 0x8a, 0xb5, 0x2c, 0xc1, 0x48, 0x1f, 0xe4, 0x03,
];
const HISTOGRAM_SPECIFICATION_CLASS_ID: [u8; 16] = [
    0xb0, 0x78, 0x62, 0xb6, 0x52, 0x83, 0x9b, 0x49, 0xad, 0x5c, 0x51, 0x40, 0xc5, 0x4e, 0xc5, 0x2f,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerTail10_1 {
    /// 保存到持久化接口 this+0xbc、+0xc4 的两个原值。
    pub doubles_bc_c4: [f64; 2],
    pub long_f4: i32,
    pub short_94: u16,
    pub long_100: Option<i32>,
    pub legacy_extensions: Vec<RasterLayerLegacyExtension10_1>,
    pub extra_safe_extensions: Vec<RasterLayerExtraSafeExtension10_1>,
    pub double_104: Option<f64>,
    /// 内联 WKSEnvelope 的四个原始位模式。空包络可以含 NaN。
    pub wks_bits: Option<[u64; 4]>,
    pub related_name: Option<RasterLayerRelationshipSlot10_1>,
    pub related_objects: Vec<RasterLayerRelationshipSlot10_1>,
    pub envelope_f0: Option<RasterLayerEnvelopeSlot10_1>,
    pub raster_io: Option<RasterLayerIo10_1>,
    pub string_128: Option<String>,
    pub short_v10: Option<u16>,
    pub doubles_12c_134: Option<[f64; 2]>,
    pub geodata_xform_v13: Option<GeodataXformSlot10_1>,
    pub envelope_v13: Option<RasterLayerEnvelopeSlot10_1>,
    pub color_v14: Option<ColorSlot10_1>,
    pub color_indices_v14: Vec<i32>,
    pub time: Option<Box<RasterLayerTime10_1>>,
    pub pixel_filter_v16: Option<RasterLayerPixelFilterSlot10_1>,
    pub histograms_v17: Vec<RasterLayerHistogramSlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerLegacyExtension10_1 {
    pub class_id: [u8; 16],
    pub raw_body: Vec<u8>,
    pub properties: Option<Box<PropertySetObjectStream10_1>>,
    pub edit_template_manager: Option<Box<EditTemplateManagerBody10_1>>,
    pub server_layer_extension: Option<Box<ServerLayerExtension10_1>>,
    pub representation_layer_extension_version: Option<i16>,
    pub route_layer_extension: Option<Box<RouteLayerExtension10_1>>,
    pub hatch_layer_extension: Option<Box<HatchLayerExtension10_1>>,
    pub tracking_filter_layer_extension: Option<Box<TrackingFilterLayerExtension10_1>>,
    pub e2_layer_extension_values: Option<E2LayerExtensionValues10_1>,
    pub feature_service_layer_edit_extension: Option<Box<FeatureServiceLayerEditExtension10_1>>,
    pub globe_layer_properties: Option<Box<GlobeLayerProperties10_1>>,
    pub time_layer_track_extension: Option<Box<TimeLayerTrackExtension10_1>>,
    pub dynamic_tiled_layer_properties: Option<Box<DynamicTiledLayerProperties10_1>>,
    pub scene_layer_properties: Option<Box<SceneLayerProperties10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerExtraSafeExtension10_1 {
    pub prefix: [u32; 2],
    pub entry: ObjectEntry10_1,
    pub raw_body: Vec<u8>,
    pub properties: Option<Box<PropertySetObjectStream10_1>>,
    pub edit_template_manager: Option<Box<EditTemplateManagerBody10_1>>,
    pub server_layer_extension: Option<Box<ServerLayerExtension10_1>>,
    pub representation_layer_extension_version: Option<i16>,
    pub route_layer_extension: Option<Box<RouteLayerExtension10_1>>,
    pub hatch_layer_extension: Option<Box<HatchLayerExtension10_1>>,
    pub tracking_filter_layer_extension: Option<Box<TrackingFilterLayerExtension10_1>>,
    pub e2_layer_extension_values: Option<E2LayerExtensionValues10_1>,
    pub feature_service_layer_edit_extension: Option<Box<FeatureServiceLayerEditExtension10_1>>,
    pub globe_layer_properties: Option<Box<GlobeLayerProperties10_1>>,
    pub time_layer_track_extension: Option<Box<TimeLayerTrackExtension10_1>>,
    pub dynamic_tiled_layer_properties: Option<Box<DynamicTiledLayerProperties10_1>>,
    pub scene_layer_properties: Option<Box<SceneLayerProperties10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerPixelFilterSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterLayerPixelFilterBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterLayerPixelFilterBody10_1 {
    Pansharpening(Box<PansharpeningFilter10_1>),
    Lut {
        version: u16,
        longs: [i32; 2],
        values: Vec<i32>,
    },
    HistogramSpecification {
        version: u16,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterLayerRelationshipBody10_1 {
    TableName(Box<TableNameBody10_1>),
    TabularName(Box<DatasetNameMoreBody10_1>),
    RasterName(Box<RasterDatasetNameEntryBody10_1>),
    Relationship(Box<RelationshipClassNameBody10_1>),
    MemoryRelationship(Box<MemoryRelationshipClassNameBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerRelationshipSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterLayerRelationshipBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerHistogramSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<RasterHistogram10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerEnvelopeSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<Envelope10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerIo10_1 {
    /// 保留省略和显式空对象的区别；两种形态均无名称对象体。
    pub tag30_is_explicit_null: bool,
    pub tag30_name: Option<Box<RasterDatasetNameEntryBody10_1>>,
    pub tag31_string: Option<String>,
    pub shorts_35_to_37: [u16; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerTime10_1 {
    pub data: TimeDataSegment10_1,
    pub display: TimeDisplaySegment10_1,
    pub dimension: TimeDimensionSegment10_1,
}

fn unsupported(label: &str) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, format!("RasterLayer {label}"))
}

fn invalid(label: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, format!("RasterLayer {label}"))
}

fn read_long(reader: &mut Reader<'_>) -> io::Result<i32> {
    Ok(reader.read_u32_le()? as i32)
}

fn read_pixel_filter(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerPixelFilterSlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if !matches!(
            registered,
            PANSHARPENING_FILTER_CLASS_ID | LUT_FILTER_CLASS_ID | HISTOGRAM_SPECIFICATION_CLASS_ID
        ) {
            return Err(unsupported(&format!(
                "v16 IPixelFilter 回引键 {key} 的登记类不受支持"
            )));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: PANSHARPENING_FILTER_CLASS_ID,
            ..
        } => Some(RasterLayerPixelFilterBody10_1::Pansharpening(Box::new(
            read_pansharpening_filter(reader, max_bytes, table)?,
        ))),
        ObjectEntry10_1::NewObject {
            class_id: LUT_FILTER_CLASS_ID,
            ..
        } => {
            let version = reader.read_u16_le()?;
            if version as i16 > 1 {
                return Err(unsupported(&format!("LutFilter 版本 {version}")));
            }
            let longs = [read_long(reader)?, read_long(reader)?];
            let count =
                usize::try_from(read_long(reader)?).map_err(|_| invalid("LutFilter 数量为负数"))?;
            if count > reader.remaining().min(max_bytes) / 4 {
                return Err(invalid("LutFilter 数组超出可读范围"));
            }
            let values = (0..count)
                .map(|_| read_long(reader))
                .collect::<io::Result<_>>()?;
            Some(RasterLayerPixelFilterBody10_1::Lut {
                version,
                longs,
                values,
            })
        }
        ObjectEntry10_1::NewObject {
            class_id: HISTOGRAM_SPECIFICATION_CLASS_ID,
            ..
        } => Some(RasterLayerPixelFilterBody10_1::HistogramSpecification {
            version: reader.read_u16_le()?,
        }),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "v16 IPixelFilter 含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerPixelFilterSlot10_1 { entry, body })
}

fn read_slots<T>(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    label: &str,
    read_entry: impl Fn(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let count = read_long(reader)?.max(0) as usize;
    if count > reader.remaining() / 16 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!("RasterLayer {label} 计数超出可读范围"),
        ));
    }
    (0..count).map(|_| read_entry(reader, table)).collect()
}

fn read_related_name(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerRelationshipSlot10_1> {
    let header = read_header_10_1(&mut reader.clone())?;
    if header == ObjectHeader::Null {
        return Err(invalid("RasterLayer 关联名称标志非零，但 IName 为空"));
    }
    if let ObjectHeader::Object { class_id, key } = header
        && table.registered_class(key).unwrap_or(class_id) == IMAGE_SERVER_NAME_CLASS_ID
    {
        let (entry, body) = read_supported_raster_name_entry_10_1(reader, max_bytes, table)?;
        return Ok(RasterLayerRelationshipSlot10_1 {
            entry,
            body: body.map(|body| RasterLayerRelationshipBody10_1::RasterName(Box::new(body))),
        });
    }
    if let ObjectHeader::Object { class_id, key } = header
        && !table.contains(key)
        && !matches!(
            class_id,
            RELATIONSHIP_CLASS_NAME_CLASS_ID | MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID
        )
    {
        let mut stop = None;
        let name = read_tabular_name_slot_10_1(reader, max_bytes, table, &mut stop)?;
        if let DatasetNameMoreSlot10_1::Body(body) = name
            && stop.is_none()
        {
            let body = match body {
                DatasetNameMoreBody10_1::Table(body) => {
                    RasterLayerRelationshipBody10_1::TableName(body)
                }
                body => RasterLayerRelationshipBody10_1::TabularName(Box::new(body)),
            };
            return Ok(RasterLayerRelationshipSlot10_1 {
                entry: ObjectEntry10_1::NewObject { class_id, key },
                body: Some(body),
            });
        }
        return Err(unsupported(&format!("关联 IName 槽未闭合：{stop:?}")));
    }
    read_relationship_slot(reader, max_bytes, table)
}

fn read_relationship_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerRelationshipSlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        *class_id = registered;
    }
    use RasterLayerRelationshipBody10_1::{MemoryRelationship, Relationship};
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body = read_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            if body.stop.is_some() {
                return Err(unsupported("RelationshipClassName 含未闭合的嵌套对象"));
            }
            Some(Relationship(Box::new(body)))
        }
        ObjectEntry10_1::NewObject {
            class_id: MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body =
                read_memory_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            if body.stop.is_some() {
                return Err(unsupported(
                    "MemoryRelationshipClassName 含未闭合的嵌套对象",
                ));
            }
            Some(MemoryRelationship(Box::new(body)))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "关联 IName 槽含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerRelationshipSlot10_1 { entry, body })
}

fn read_histogram_slot(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerHistogramSlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if registered != STATS_HISTOGRAM_CLASS_ID {
            return Err(unsupported(&format!(
                "v17 IStatsHistogram 回引键 {key} 的登记类不受支持"
            )));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: STATS_HISTOGRAM_CLASS_ID,
            ..
        } => Some(Box::new(read_stats_histogram_body_10_1(reader)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "v17 IStatsHistogram 槽含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerHistogramSlot10_1 { entry, body })
}

pub(super) fn read_envelope_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerEnvelopeSlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if registered != ENVELOPE_CLASS_ID {
            return Err(unsupported(&format!(
                "IEnvelope 回引键 {key} 的登记类不受支持"
            )));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: ENVELOPE_CLASS_ID,
            ..
        } => Some(Box::new(read_envelope_10_1(reader, max_bytes, table)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "IEnvelope 槽含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerEnvelopeSlot10_1 { entry, body })
}

fn expect_record(reader: &mut Reader<'_>, tag: i32, length: i32) -> io::Result<()> {
    let actual_tag = read_long(reader)?;
    let actual_length = read_long(reader)?;
    if (actual_tag, actual_length) != (tag, length) {
        return Err(invalid(&format!(
            "RasterIO 预期 ({tag},{length})，实际 ({actual_tag},{actual_length})"
        )));
    }
    Ok(())
}

fn read_raster_io(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerIo10_1> {
    expect_record(reader, 0xff00ff, 0)?;
    let (tag30_is_explicit_null, tag30_name) = if read_long(&mut reader.clone())? == 30 {
        expect_record(reader, 30, -1)?;
        let class_id = reader.read_bytes(16)?.try_into().unwrap();
        if class_id == [0; 16] {
            (true, None)
        } else {
            let name = read_supported_raster_name_body_10_1(reader, max_bytes, table, class_id, 0)?;
            (false, Some(Box::new(name)))
        }
    } else {
        (false, None)
    };
    let tag31_string = if read_long(&mut reader.clone())? == 31 {
        reader.read_u32_le()?;
        // RasterIO 宽字符串记录的 length 是含结束零的字节数，与此读取器一致。
        Some(read_string_10_1(reader, max_bytes)?)
    } else {
        if tag30_name.is_some() {
            return Err(invalid("RasterIO 非空 tag 30 缺少 tag 31"));
        }
        None
    };
    let mut shorts_35_to_37 = [0; 3];
    for (tag, value) in (35..=37).zip(&mut shorts_35_to_37) {
        expect_record(reader, tag, 2)?;
        *value = reader.read_u16_le()?;
    }
    expect_record(reader, -1, -1)?;
    Ok(RasterLayerIo10_1 {
        tag30_is_explicit_null,
        tag30_name,
        tag31_string,
        shorts_35_to_37,
    })
}

fn read_template_extension(
    raw_body: &[u8],
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<Box<EditTemplateManagerBody10_1>> {
    let mut body = Reader::new(raw_body);
    let manager = read_edit_template_manager_body_10_1(&mut body, max_bytes, table, key)?;
    if manager.stop.is_some() {
        return Err(unsupported("扩展内的 EditTemplateManager 尚未完整消费"));
    }
    if body.remaining() != 0 {
        return Err(invalid("EditTemplateManager 消费长度与扩展声明不符"));
    }
    Ok(Box::new(manager))
}

fn read_legacy_extensions(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    count: usize,
    max_bytes: usize,
) -> io::Result<Vec<RasterLayerLegacyExtension10_1>> {
    if count > reader.remaining() / LEGACY_EXTENSION_HEADER_SIZE
        || count > max_bytes / LEGACY_EXTENSION_HEADER_SIZE
    {
        return Err(invalid("旧式扩展列表计数超出可读范围或限额"));
    }
    let mut budget = max_bytes - count * LEGACY_EXTENSION_HEADER_SIZE;
    (0..count)
        .map(|_| {
            // LayerLib ExtensionHelper 的长度仅覆盖对象体，CLSID 后无对象键。
            let size = u64::from_le_bytes(reader.read_bytes(8)?.try_into().unwrap());
            let class_id = reader.read_bytes(16)?.try_into().unwrap();
            let size = usize::try_from(size)
                .ok()
                .filter(|&size| size <= budget)
                .ok_or_else(|| invalid("旧式扩展对象体长度超过限额"))?;
            let raw_body = reader.read_bytes(size)?;
            let dynamic_tiled_layer_properties = (class_id
                == DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID
                && size != 0)
                .then(|| read_bounded_dynamic_tiled_layer_properties(raw_body, max_bytes, table))
                .transpose()?;
            // 长度为零时，原生旧式加载器跳过对象体。
            let properties = (class_id == PROPERTY_SET_CLASS_ID && size != 0)
                .then(|| read_bounded_extension_property_set(raw_body, max_bytes, table))
                .transpose()?;
            let edit_template_manager = (class_id == EDIT_TEMPLATE_MANAGER_CLASS_ID && size != 0)
                .then(|| read_template_extension(raw_body, max_bytes, table, 0))
                .transpose()?;
            let server_layer_extension = (class_id == SERVER_LAYER_EXTENSION_CLASS_ID && size != 0)
                .then(|| read_bounded_server_layer_extension(raw_body, max_bytes, table))
                .transpose()?;
            let representation_layer_extension_version =
                (class_id == REPRESENTATION_LAYER_EXTENSION_CLASS_ID && size != 0)
                    .then(|| read_bounded_representation_layer_extension(raw_body))
                    .transpose()?;
            let route_layer_extension = (class_id == ROUTE_LAYER_EXTENSION_CLASS_ID && size != 0)
                .then(|| read_bounded_route_layer_extension(raw_body, max_bytes, table))
                .transpose()?;
            let hatch_layer_extension = (class_id == HATCH_LAYER_EXTENSION_CLASS_ID && size != 0)
                .then(|| read_bounded_hatch_layer_extension(raw_body, max_bytes, table))
                .transpose()?;
            let tracking_filter_layer_extension = (class_id
                == TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID
                && size != 0)
                .then(|| read_bounded_tracking_filter_layer_extension(raw_body, max_bytes, table))
                .transpose()?;
            let e2_layer_extension_values = (class_id == E2_LAYER_EXTENSION_VALUES_CLASS_ID
                && size != 0)
                .then(|| read_bounded_e2_layer_extension_values(raw_body, table))
                .transpose()?;
            let feature_service_layer_edit_extension = (class_id
                == FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID
                && size != 0)
                .then(|| {
                    read_bounded_feature_service_layer_edit_extension(raw_body, max_bytes, table)
                })
                .transpose()?;
            let globe_layer_properties = (class_id == GLOBE_LAYER_PROPERTIES_CLASS_ID
                && !raw_body.is_empty())
            .then(|| read_bounded_globe_layer_properties(raw_body, max_bytes, table))
            .transpose()?;
            let time_layer_track_extension = (class_id == TIME_LAYER_TRACK_EXTENSION_CLASS_ID
                && size != 0)
                .then(|| read_bounded_time_layer_track_extension(raw_body, max_bytes, table))
                .transpose()?;
            let scene_layer_properties = (is_scene_layer_properties(class_id) && size != 0)
                .then(|| read_bounded_scene_layer_properties(raw_body, max_bytes, table))
                .transpose()?;
            budget -= size;
            Ok(RasterLayerLegacyExtension10_1 {
                class_id,
                raw_body: raw_body.to_vec(),
                properties,
                edit_template_manager,
                server_layer_extension,
                representation_layer_extension_version,
                route_layer_extension,
                hatch_layer_extension,
                tracking_filter_layer_extension,
                e2_layer_extension_values,
                feature_service_layer_edit_extension,
                globe_layer_properties,
                time_layer_track_extension,
                dynamic_tiled_layer_properties,
                scene_layer_properties,
            })
        })
        .collect()
}

pub(super) fn read_extra_safe_extensions(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    count: usize,
    max_bytes: usize,
) -> io::Result<Vec<RasterLayerExtraSafeExtension10_1>> {
    if count > reader.remaining() / 8 || count > max_bytes / 8 {
        return Err(invalid("ExtraSafe 扩展列表计数超出可读范围"));
    }
    let mut budget = max_bytes - count * 8;
    (0..count)
        .map(|_| {
            let a = reader.read_u32_le()?;
            let b = reader.read_u32_le()?;
            // 与图层列表使用相同的 ExtraSafe 边界。零 B 时使用写侧 A。
            let size = if b == 0 { a } else { b } as usize;
            if size > budget {
                return Err(invalid("ExtraSafe 扩展对象长度超过限额"));
            }
            budget -= size;
            let mut region = reader.read_region(size)?;
            let mut entry = if size == 0 {
                ObjectEntry10_1::Null
            } else {
                read_object_entry_10_1(&mut region, table)?
            };
            if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
                && let Some(registered) = table.registered_class(*key)
            {
                *class_id = registered;
            }
            if !matches!(entry, ObjectEntry10_1::NewObject { .. }) && region.remaining() != 0 {
                return Err(invalid("ExtraSafe 空对象或回引有多余字节"));
            }
            let raw_body = region.read_bytes(region.remaining())?;
            let properties = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: PROPERTY_SET_CLASS_ID,
                    ..
                } => Some(read_bounded_extension_property_set(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let scene_layer_properties = match entry {
                ObjectEntry10_1::NewObject { class_id, .. }
                    if is_scene_layer_properties(class_id) =>
                {
                    Some(read_bounded_scene_layer_properties(
                        raw_body, max_bytes, table,
                    )?)
                }
                _ => None,
            };
            let dynamic_tiled_layer_properties = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID,
                    ..
                } => Some(read_bounded_dynamic_tiled_layer_properties(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let edit_template_manager = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: EDIT_TEMPLATE_MANAGER_CLASS_ID,
                    key,
                } => Some(read_template_extension(raw_body, max_bytes, table, key)?),
                _ => None,
            };
            let server_layer_extension = if matches!(
                entry,
                ObjectEntry10_1::NewObject {
                    class_id: SERVER_LAYER_EXTENSION_CLASS_ID,
                    ..
                }
            ) {
                Some(read_bounded_server_layer_extension(
                    raw_body, max_bytes, table,
                )?)
            } else {
                None
            };
            let representation_layer_extension_version = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: REPRESENTATION_LAYER_EXTENSION_CLASS_ID,
                    ..
                } => Some(read_bounded_representation_layer_extension(raw_body)?),
                _ => None,
            };
            let route_layer_extension = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: ROUTE_LAYER_EXTENSION_CLASS_ID,
                    ..
                } => Some(read_bounded_route_layer_extension(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let hatch_layer_extension = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: HATCH_LAYER_EXTENSION_CLASS_ID,
                    ..
                } => Some(read_bounded_hatch_layer_extension(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let tracking_filter_layer_extension = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID,
                    ..
                } => Some(read_bounded_tracking_filter_layer_extension(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let e2_layer_extension_values = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: E2_LAYER_EXTENSION_VALUES_CLASS_ID,
                    ..
                } => Some(read_bounded_e2_layer_extension_values(raw_body, table)?),
                _ => None,
            };
            let feature_service_layer_edit_extension = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID,
                    ..
                } => Some(read_bounded_feature_service_layer_edit_extension(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let globe_layer_properties = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: GLOBE_LAYER_PROPERTIES_CLASS_ID,
                    ..
                } => Some(read_bounded_globe_layer_properties(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            let time_layer_track_extension = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: TIME_LAYER_TRACK_EXTENSION_CLASS_ID,
                    ..
                } => Some(read_bounded_time_layer_track_extension(
                    raw_body, max_bytes, table,
                )?),
                _ => None,
            };
            Ok(RasterLayerExtraSafeExtension10_1 {
                prefix: [a, b],
                entry,
                raw_body: raw_body.to_vec(),
                properties,
                edit_template_manager,
                server_layer_extension,
                representation_layer_extension_version,
                route_layer_extension,
                hatch_layer_extension,
                tracking_filter_layer_extension,
                e2_layer_extension_values,
                feature_service_layer_edit_extension,
                globe_layer_properties,
                time_layer_track_extension,
                dynamic_tiled_layer_properties,
                scene_layer_properties,
            })
        })
        .collect()
}

fn read_color_indices(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<Vec<i32>> {
    let count = usize::try_from(read_long(reader)?).map_err(|_| invalid("颜色索引展开数量为负"))?;
    if count > max_bytes / 4 {
        return Err(invalid("颜色索引展开长度超过限额"));
    }
    let mut indices = Vec::<i32>::new();
    while indices.len() < count {
        let token = read_long(reader)?;
        if token >= 0 {
            indices.push(token);
            continue;
        }
        // 非负值是单个索引，负值绝对值是紧接上一个索引的连续区段终点。
        let start = indices.last().and_then(|&index| index.checked_add(1));
        let end = token.checked_neg();
        match (start, end) {
            (Some(start), Some(end))
                if start <= end && ((end - start) as usize) < count - indices.len() =>
            {
                indices.extend(start..=end);
            }
            _ => return Err(invalid("颜色索引连续区段超出声明范围")),
        }
    }
    Ok(indices)
}

/// RasterLayer v12 依赖外层边界或已证的空数据集路径。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RasterLayerBoundary {
    Bounded,
    Inline,
    InlineWithoutDataset,
}

/// 从已闭合的 IRasterRenderer 之后读取 RasterLayer 的有界尾段。
/// 任何不支持分支或截断均回退到渲染器后的起点，不污染对象键。
pub(crate) fn read_raster_layer_tail_10_1(
    reader: &mut Reader<'_>,
    version: i32,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerTail10_1> {
    read_raster_layer_tail(
        reader,
        version,
        max_bytes,
        table,
        RasterLayerBoundary::Bounded,
    )
}

pub(super) fn read_raster_layer_tail(
    reader: &mut Reader<'_>,
    version: i32,
    max_bytes: usize,
    table: &mut RefTable10_1,
    boundary: RasterLayerBoundary,
) -> io::Result<RasterLayerTail10_1> {
    if !(0..=17).contains(&version) {
        return Err(unsupported(&format!("尚未接入 v{version} 尾段")));
    }
    if version == 12 && boundary == RasterLayerBoundary::Inline {
        return Err(unsupported("v12 需要对象体边界以判定可选变换和包络槽"));
    }
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let doubles_bc_c4 = [trial.read_f64_le()?, trial.read_f64_le()?];
    let long_f4 = read_long(&mut trial)?;
    let short_94 = trial.read_u16_le()?;
    let long_100 = (version >= 2).then(|| read_long(&mut trial)).transpose()?;
    let (legacy_extensions, extra_safe_extensions) = if version >= 8 {
        let count = read_long(&mut trial)?.max(0) as usize;
        let extensions = read_extra_safe_extensions(&mut trial, &mut staged, count, max_bytes)?;
        (Vec::new(), extensions)
    } else if version >= 3 {
        let count = read_long(&mut trial)?.max(0) as usize;
        let extensions = read_legacy_extensions(&mut trial, &mut staged, count, max_bytes)?;
        (extensions, Vec::new())
    } else {
        (Vec::new(), Vec::new())
    };
    let double_104 = (version >= 4).then(|| trial.read_f64_le()).transpose()?;
    let wks_bits = (version >= 5)
        .then(|| {
            let mut bits = [0; 4];
            for slot in &mut bits {
                *slot = u64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap());
            }
            Ok::<_, io::Error>(bits)
        })
        .transpose()?;
    let (related_name, related_objects) = if version >= 6 {
        let name = (read_long(&mut trial)? != 0)
            .then(|| read_related_name(&mut trial, max_bytes, &mut staged))
            .transpose()?;
        let objects = read_slots(&mut trial, &mut staged, "关联对象列表", |r, t| {
            let slot = read_relationship_slot(r, max_bytes, t)?;
            if let ObjectEntry10_1::BackReference { key, .. } = slot.entry
                && t.registered_class(key).is_some_and(|class| {
                    !matches!(
                        class,
                        RELATIONSHIP_CLASS_NAME_CLASS_ID | MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID
                    )
                })
            {
                return Err(unsupported("RasterLayer 关系名称列表回引的登记类不受支持"));
            }
            Ok(slot)
        })?;
        (name, objects)
    } else {
        (None, Vec::new())
    };
    let envelope_f0 = (version >= 7)
        .then(|| read_envelope_slot(&mut trial, max_bytes, &mut staged))
        .transpose()?;
    // InitCheck 缺少 magic 时恢复游标，旧尾段可能直接结束或进入字符串。
    let raster_io = (trial.remaining() >= 4 && read_long(&mut trial.clone())? == 0xff00ff)
        .then(|| read_raster_io(&mut trial, max_bytes, &mut staged))
        .transpose()?;
    let string_128 = (version >= 10)
        .then(|| read_string_10_1(&mut trial, max_bytes))
        .transpose()?;
    let short_v10 = (version == 10).then(|| trial.read_u16_le()).transpose()?;
    let doubles_12c_134 = (version >= 11)
        .then(|| -> io::Result<_> { Ok([trial.read_f64_le()?, trial.read_f64_le()?]) })
        .transpose()?;
    if boundary == RasterLayerBoundary::InlineWithoutDataset
        && (extra_safe_extensions
            .iter()
            .any(|slot| slot.entry != ObjectEntry10_1::Null)
            || related_name.as_ref().is_some_and(|slot| {
                // 已证空打开结果经关联安装路径保留空数据集成员。
                match slot.entry {
                    ObjectEntry10_1::NewObject { key, .. }
                    | ObjectEntry10_1::BackReference { key, .. } => {
                        !match staged.registered_class(key) {
                            Some(TABLE_NAME_CLASS_ID | OBJECT_CLASS_NAME_CLASS_ID) => {
                                staged.table_name_has_null_workspace(key)
                            }
                            Some(IMAGE_SERVER_NAME_CLASS_ID) => {
                                staged.raster_name_opens_without_result(key)
                            }
                            _ => false,
                        }
                    }
                    ObjectEntry10_1::Null => true,
                }
            })
            || related_objects
                .iter()
                .any(|slot| slot.entry != ObjectEntry10_1::Null)
            || envelope_f0.as_ref().is_some_and(|slot| {
                // 旧包络只写入持久化视图 +0xf0。回引按登记对象查询接口，不重读对象体。
                match slot.entry {
                    ObjectEntry10_1::Null => false,
                    ObjectEntry10_1::BackReference { key, .. } => {
                        staged.registered_class(key) != Some(ENVELOPE_CLASS_ID)
                    }
                    ObjectEntry10_1::NewObject { .. } => slot.body.as_ref().is_some_and(|body| {
                        // 新空间参考由 Geometry 的已知持久化读取器解析。回引须能确认接口类。
                        match body.spatial_reference {
                            Some(SpatialReferenceEntry10_1::BackReference { key, .. }) => {
                                !matches!(
                                    staged.registered_class(key),
                                    Some(
                                        PROJECTED_COORDINATE_SYSTEM_CLASS_ID
                                            | GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID
                                            | UNKNOWN_COORDINATE_SYSTEM_CLASS_ID
                                    )
                                )
                            }
                            _ => false,
                        }
                    }),
                }
            })
            || raster_io.as_ref().is_some_and(|io| {
                match (&io.tag30_name, io.tag31_string.as_deref()) {
                    // 无名称时原生跳过 tag 31，查找 tag 35。
                    (None, _) => false,
                    (Some(name), Some(path)) => {
                        // 这些已知空结果的名称不会被路径展开修改。
                        !name.open_without_result()
                            || (!path.is_empty()
                                && !matches!(
                                    name.as_ref(),
                                    RasterDatasetNameEntryBody10_1::RasterType(_)
                                        | RasterDatasetNameEntryBody10_1::WcsCoverage(_)
                                        | RasterDatasetNameEntryBody10_1::InternetTiledLayer(_)
                                        | RasterDatasetNameEntryBody10_1::NetCdf(_)
                                        | RasterDatasetNameEntryBody10_1::ImageServerReference(_)
                                )
                                && !matches!(
                                    name.as_ref(),
                                    RasterDatasetNameEntryBody10_1::ImageService(name)
                                        if name.workspace == RasterDatasetWorkspaceSlot10_1::Null
                                ))
                    }
                    _ => true,
                }
            }))
    {
        return Err(unsupported("v12 含尚未核对的子对象或 RasterIO 回调"));
    }
    // v12 由原生数据集打开结果选择是否读取这对槽，且其后没有其他字段。
    // 父条目到界则省略，有剩余时两槽必须完整读取并恰好到界。
    let has_transform_pair = version >= 13
        || (version == 12 && boundary == RasterLayerBoundary::Bounded && trial.remaining() != 0);
    let end = |reader: &mut Reader<'_>, refs: &mut RefTable10_1, has_empty_third_slot| {
        read_raster_layer_end(
            reader,
            refs,
            version,
            max_bytes,
            boundary,
            has_transform_pair,
            has_empty_third_slot,
        )
    };
    // 先验证写侧布局；仅有界 ProjectiveXform 可用完整尾段验证空第三槽。
    let suffix = if boundary == RasterLayerBoundary::Bounded
        && has_transform_pair
        && trial
            .clone()
            .read_bytes(16)
            .is_ok_and(|id| id == PROJECTIVE_XFORM_CLASS_ID)
    {
        let mut candidate = trial.clone();
        let mut candidate_refs = staged.clone();
        match end(&mut candidate, &mut candidate_refs, false) {
            Ok(suffix) => {
                trial = candidate;
                staged = candidate_refs;
                suffix
            }
            Err(original) => end(&mut trial, &mut staged, true).map_err(|_| original)?,
        }
    } else {
        end(&mut trial, &mut staged, false)?
    };
    *table = staged;
    *reader = trial;
    Ok(RasterLayerTail10_1 {
        doubles_bc_c4,
        long_f4,
        short_94,
        long_100,
        legacy_extensions,
        extra_safe_extensions,
        double_104,
        wks_bits,
        related_name,
        related_objects,
        envelope_f0,
        raster_io,
        string_128,
        short_v10,
        doubles_12c_134,
        geodata_xform_v13: suffix.transform,
        envelope_v13: suffix.envelope,
        color_v14: suffix.color,
        color_indices_v14: suffix.color_indices,
        time: suffix.time,
        pixel_filter_v16: suffix.pixel_filter,
        histograms_v17: suffix.histograms,
    })
}

struct RasterLayerEnd {
    transform: Option<GeodataXformSlot10_1>,
    envelope: Option<RasterLayerEnvelopeSlot10_1>,
    color: Option<ColorSlot10_1>,
    color_indices: Vec<i32>,
    time: Option<Box<RasterLayerTime10_1>>,
    pixel_filter: Option<RasterLayerPixelFilterSlot10_1>,
    histograms: Vec<RasterLayerHistogramSlot10_1>,
}

fn read_raster_layer_end(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    version: i32,
    max_bytes: usize,
    boundary: RasterLayerBoundary,
    has_transform_pair: bool,
    has_empty_third_slot: bool,
) -> io::Result<RasterLayerEnd> {
    let transform = has_transform_pair
        .then(|| {
            if boundary == RasterLayerBoundary::Bounded
                && matches!(
                    read_header_10_1(&mut reader.clone())?,
                    ObjectHeader::Object {
                        class_id: PROJECTIVE_XFORM_CLASS_ID, key
                    } if !table.contains(key)
                )
            {
                read_bounded_projective_candidate(reader, max_bytes, table, has_empty_third_slot)
            } else {
                read_geodata_xform_slot_10_1(reader, max_bytes, table)
            }
        })
        .transpose()?;
    let envelope = has_transform_pair
        .then(|| read_envelope_slot(reader, max_bytes, table))
        .transpose()?;
    let (color, color_indices) = if version >= 14 {
        let color = read_color_slot_10_1(reader, max_bytes, table)?;
        if matches!(color, ColorSlot10_1::Unsupported { .. }) {
            return Err(unsupported("v14 IColor 类尚未支持"));
        }
        (Some(color), read_color_indices(reader, max_bytes)?)
    } else {
        (None, Vec::new())
    };
    let time = if version >= 15 {
        Some(Box::new(RasterLayerTime10_1 {
            data: read_time_data_segment_10_1(reader, max_bytes, table)?,
            display: read_time_display_segment_10_1(reader)?,
            dimension: read_time_dimension_segment_10_1(reader, max_bytes)?,
        }))
    } else {
        None
    };
    let pixel_filter = (version >= 16)
        .then(|| read_pixel_filter(reader, max_bytes, table))
        .transpose()?;
    let histograms = if version >= 17 {
        read_slots(
            reader,
            table,
            "v17 IStatsHistogram 列表",
            read_histogram_slot,
        )?
    } else {
        Vec::new()
    };
    if boundary == RasterLayerBoundary::Bounded && reader.remaining() != 0 {
        return Err(invalid("尾段终点与 ExtraSafe 条目边界不一致"));
    }
    Ok(RasterLayerEnd {
        transform,
        envelope,
        color,
        color_indices,
        time,
        pixel_filter,
        histograms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_raster_extension_budget_counts_headers_and_bodies() {
        let mut bytes = Vec::new();
        for value in [0x11, 0x22] {
            bytes.extend(1_u64.to_le_bytes());
            bytes.extend([0x53; 16]);
            bytes.push(value);
        }

        let mut refs = RefTable10_1::new();
        for limit in [47, 49] {
            let error =
                read_legacy_extensions(&mut Reader::new(&bytes), &mut refs, 2, limit).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
        }

        let mut reader = Reader::new(&bytes);
        let entries = read_legacy_extensions(&mut reader, &mut refs, 2, 50).unwrap();
        assert_eq!(entries[0].raw_body, [0x11]);
        assert_eq!(entries[1].raw_body, [0x22]);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn dynamic_tiled_properties_share_nested_keys_in_both_extension_formats() {
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        fn text(buf: &mut Vec<u8>, text: &str) {
            let utf16: Vec<_> = text.encode_utf16().chain([0]).collect();
            buf.extend((utf16.len() as u32 * 2).to_le_bytes());
            buf.extend(utf16.into_iter().flat_map(u16::to_le_bytes));
        }

        for host_version in [3, 8] {
            for version in [1_u16, 2, 3] {
                let body = |new_properties, class_id, key: u32| {
                    let mut bytes = version.to_le_bytes().to_vec();
                    text(&mut bytes, "tile source");
                    bytes.extend(class_id);
                    bytes.extend(key.to_le_bytes());
                    if new_properties {
                        bytes.extend(1_u16.to_le_bytes());
                        bytes.extend(0_u32.to_le_bytes());
                    }
                    text(&mut bytes, "second");
                    text(&mut bytes, "third");
                    bytes.extend(4_u16.to_le_bytes());
                    bytes.extend(5_u16.to_le_bytes());
                    bytes.extend(123.0_f64.to_le_bytes());
                    if version >= 2 {
                        bytes.extend(23_u32.to_le_bytes());
                        bytes.extend(24_u16.to_le_bytes());
                        bytes.extend(25_u32.to_le_bytes());
                    }
                    if version == 3 {
                        bytes.extend(31_u16.to_le_bytes());
                    }
                    bytes
                };
                let second = body(false, DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID, 91);
                let first = body(true, PROPERTY_SET_CLASS_ID, 91);
                let build = |first_body: &[u8], second_body: &[u8]| {
                    let mut bytes = vec![0; 26];
                    bytes.extend(2_u32.to_le_bytes());
                    for (key, body) in [(80_u32, first_body), (81, second_body)] {
                        if host_version == 3 {
                            bytes.extend((body.len() as u64).to_le_bytes());
                        } else {
                            bytes.extend((20 + body.len() as u32).to_le_bytes());
                            bytes.extend(0_u32.to_le_bytes());
                        }
                        bytes.extend(DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID);
                        if host_version == 8 {
                            bytes.extend(key.to_le_bytes());
                        }
                        bytes.extend(body);
                    }
                    if host_version == 8 {
                        bytes.extend([0; 64]);
                    }
                    bytes
                };
                let bytes = build(&first, &second);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let tail = read_raster_layer_tail_10_1(&mut reader, host_version, 4096, &mut refs)
                    .unwrap();
                let [first_parsed, second_parsed] = if host_version == 3 {
                    assert_eq!(refs.len(), 1);
                    [
                        tail.legacy_extensions[0]
                            .dynamic_tiled_layer_properties
                            .as_deref()
                            .unwrap(),
                        tail.legacy_extensions[1]
                            .dynamic_tiled_layer_properties
                            .as_deref()
                            .unwrap(),
                    ]
                } else {
                    assert_eq!(refs.len(), 3);
                    [
                        tail.extra_safe_extensions[0]
                            .dynamic_tiled_layer_properties
                            .as_deref()
                            .unwrap(),
                        tail.extra_safe_extensions[1]
                            .dynamic_tiled_layer_properties
                            .as_deref()
                            .unwrap(),
                    ]
                };
                assert_eq!(first_parsed.strings, ["tile source", "second", "third"]);
                assert_eq!(first_parsed.value, 123.0);
                assert_eq!(first_parsed.shorts, [4, 5]);
                assert_eq!(
                    first_parsed.version2_fields,
                    (version >= 2).then_some((23, 24, 25))
                );
                assert_eq!(first_parsed.version3_short, (version == 3).then_some(31));
                assert_eq!(first_parsed.properties.body.as_ref().unwrap().count, 0);
                assert!(matches!(
                    second_parsed.properties.entry,
                    ObjectEntry10_1::BackReference {
                        key: 91,
                        class_id: PROPERTY_SET_CLASS_ID
                    }
                ));
                assert_eq!(reader.remaining(), 0);

                if host_version == 3 {
                    let mut reader = Reader::new(&bytes[26..]);
                    let mut refs = RefTable10_1::new();
                    let shared = crate::layers::layer_extensions::read_layer_extensions_10_1(
                        &mut reader,
                        4096,
                        &mut refs,
                    )
                    .unwrap();
                    assert_eq!(
                        shared[1].dynamic_tiled_layer_properties.as_deref(),
                        Some(second_parsed)
                    );
                }

                let wrong_reference = body(false, PROPERTY_SET_CLASS_ID, 92);
                let invalid = build(&first, &wrong_reference);
                let mut reader = Reader::new(&invalid);
                let mut refs = RefTable10_1::new();
                let mut existing = DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID.to_vec();
                existing.extend(92_u32.to_le_bytes());
                read_object_entry_10_1(&mut Reader::new(&existing), &mut refs).unwrap();
                assert_eq!(
                    read_raster_layer_tail_10_1(&mut reader, host_version, 4096, &mut refs)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::Unsupported
                );
                assert_eq!(reader.position(), 0);
                assert_eq!(refs.len(), 1);
                assert!(refs.contains(92));

                let mut oversized = first.clone();
                oversized.push(0);
                for broken in [&first[..first.len() - 1], &oversized] {
                    let bytes = build(broken, &second);
                    let mut reader = Reader::new(&bytes);
                    let mut refs = RefTable10_1::new();
                    assert!(
                        read_raster_layer_tail_10_1(&mut reader, host_version, 4096, &mut refs)
                            .is_err()
                    );
                    assert_eq!(reader.position(), 0);
                    assert!(refs.is_empty());
                }
            }
        }
    }

    #[test]
    fn tail_v17_reads_supported_filters_before_histograms_and_rolls_back_keys() {
        let mut bytes = vec![0; 0x4e]; // 标量、空扩展、内联范围、选择器和关联计数。
        bytes.extend(ENVELOPE_CLASS_ID);
        bytes.extend(51_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 32]); // Envelope v1 不带嵌套坐标系。
        bytes.extend(0x00ff_00ff_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        for tag in 35_i32..=37 {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
        }
        bytes.extend([0xff; 8]);
        let mut put_empty_string = 2_u32.to_le_bytes().to_vec();
        put_empty_string.extend(0_u16.to_le_bytes());
        bytes.extend(&put_empty_string);
        bytes.extend([0; 16]); // v11 两个 double。
        bytes.extend(crate::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID);
        bytes.extend(53_u32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 16]); // Identity 的空空间参考。
        bytes.extend([0; 16]); // v13 空包络。
        bytes.extend([0; 16]); // v14 空颜色。
        for value in [6_i32, 0, -3, 10, -11] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(2_u16.to_le_bytes()); // TimeData。
        bytes.extend([0; 20]);
        bytes.extend(1_u16.to_le_bytes()); // TimeDataDisplay。
        bytes.extend([0; 26]);
        bytes.extend(1_u16.to_le_bytes()); // TimeDimensionDefinition。
        bytes.extend(&put_empty_string);
        bytes.extend(&put_empty_string);
        let filter_start = bytes.len();
        bytes.extend(PANSHARPENING_FILTER_CLASS_ID);
        bytes.extend(54_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 32]); // 两个空栅格名称。
        bytes.extend(2_i32.to_le_bytes()); // 锐化方法。
        bytes.extend([0; 32]); // 四个权重。
        let filter_end = bytes.len();
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(STATS_HISTOGRAM_CLASS_ID);
        bytes.extend(52_u32.to_le_bytes());
        bytes.extend(5_u32.to_le_bytes());
        bytes.extend([0; 48]); // 六个统计值。
        bytes.extend(1_u32.to_le_bytes()); // 一个直方图桶。
        bytes.extend(10_f64.to_le_bytes());
        bytes.extend([0; 16]); // 从版本 2 与 4 起的 double。
        bytes.extend(0_u32.to_le_bytes()); // v5 空追加数组。
        let reference = bytes.len();
        bytes.extend([0x7d; 16]);
        bytes.extend(52_u32.to_le_bytes());

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let tail = read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).unwrap();
        assert!(tail.envelope_f0.unwrap().body.is_some());
        assert!(refs.contains(51));
        assert!(refs.contains(52));
        assert!(refs.contains(53));
        assert!(refs.contains(54));
        assert!(tail.geodata_xform_v13.unwrap().body.is_some());
        assert_eq!(tail.color_indices_v14, [0, 1, 2, 3, 10, 11]);
        assert!(tail.pixel_filter_v16.unwrap().body.is_some());
        assert_eq!(tail.histograms_v17[0].body.as_ref().unwrap().bins, [10.0]);
        assert_eq!(reader.remaining(), 0);

        assert_eq!(
            tail.histograms_v17[1].entry,
            ObjectEntry10_1::BackReference {
                class_id: STATS_HISTOGRAM_CLASS_ID,
                key: 52,
            }
        );

        let mut wrong = bytes.clone();
        wrong[reference..reference + 16].copy_from_slice(&STATS_HISTOGRAM_CLASS_ID);
        wrong[reference + 16..reference + 20].copy_from_slice(&54_u32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&wrong);
        assert_eq!(
            read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
        assert!(refs.is_empty());
        assert_eq!(reader.position(), 0);

        let mut wrong = bytes.clone();
        let reference = PANSHARPENING_FILTER_CLASS_ID
            .into_iter()
            .chain(51_u32.to_le_bytes());
        wrong.splice(filter_start..filter_end, reference);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&wrong);
        assert_eq!(
            read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
        assert!(refs.is_empty());
        assert_eq!(reader.position(), 0);

        let complete_end = bytes.len();
        bytes.push(0x90);
        for cut in (0..complete_end).chain([bytes.len()]) {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(
                read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).is_err(),
                "cut={cut}"
            );
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(51));
            assert!(!refs.contains(52));
            assert!(!refs.contains(53));
            assert!(!refs.contains(54));
        }

        bytes.truncate(complete_end);
        let mut lut = LUT_FILTER_CLASS_ID.to_vec();
        lut.extend(54_u32.to_le_bytes());
        lut.extend(1_u16.to_le_bytes());
        for value in [4_i32, 8, 2, 17, -9] {
            lut.extend(value.to_le_bytes());
        }
        let lut_end = filter_start + lut.len();
        bytes.splice(filter_start..filter_end, lut);
        for version in [1_u16, 0, u16::MAX] {
            bytes[filter_start + 20..filter_start + 22].copy_from_slice(&version.to_le_bytes());
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let tail = read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).unwrap();
            assert_eq!(
                tail.pixel_filter_v16.unwrap().body,
                Some(RasterLayerPixelFilterBody10_1::Lut {
                    version,
                    longs: [4, 8],
                    values: vec![17, -9],
                })
            );
            assert!(refs.contains(54));
            assert_eq!(tail.histograms_v17[0].body.as_ref().unwrap().bins, [10.0]);
            assert_eq!(reader.remaining(), 0);
        }

        for count in [-1_i32, i32::MAX] {
            bytes[filter_start + 30..filter_start + 34].copy_from_slice(&count.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            assert!(read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }

        let mut specification = HISTOGRAM_SPECIFICATION_CLASS_ID.to_vec();
        specification.extend(54_u32.to_le_bytes());
        specification.extend(1_u16.to_le_bytes());
        bytes.splice(filter_start..lut_end, specification);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let tail = read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).unwrap();
        assert_eq!(
            tail.pixel_filter_v16.unwrap().body,
            Some(RasterLayerPixelFilterBody10_1::HistogramSpecification { version: 1 })
        );
        assert!(refs.contains(54));
        assert_eq!(tail.histograms_v17[0].body.as_ref().unwrap().bins, [10.0]);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn color_indices_reject_overflow_and_invalid_runs() {
        for encoded in [
            [-1, 1, 2],
            [i32::MIN, 1, 2],
            [32, 1, 2],
            [2, 3, -7],
            [2, 3, -3],
            [2, 3, i32::MIN],
            [2, -4, 0],
            [2, i32::MAX, -4],
        ] {
            let bytes: Vec<_> = encoded.into_iter().flat_map(i32::to_le_bytes).collect();
            assert!(read_color_indices(&mut Reader::new(&bytes), 64).is_err());
        }
    }

    #[test]
    fn raster_io_tag30_cache_name_keeps_nested_keys_and_stops_at_unknown_tile_reader() {
        use crate::layers::raster_dataset_name::{
            CACHE_RASTER_DATASET_NAME_CLASS_ID, RasterDatasetWorkspaceSlot10_1,
        };
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = vec![0; 22];
        for value in [0xff00ff_i32, 0, 30, -1] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(CACHE_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(1_u16.to_le_bytes());
        let empty_string = [2, 0, 0, 0, 0, 0];
        bytes.extend(empty_string.repeat(2));
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(29_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(empty_string.repeat(3));
        bytes.extend([0; 21]);
        let tile_offset = bytes.len();
        let tile_class = [
            0x60, 0x85, 0xd8, 0x27, 0x8e, 0x86, 0x69, 0x48, 0xbf, 0x54, 0x7e, 0xec, 0xa5, 0x22,
            0x59, 0x4e,
        ];
        bytes.extend(tile_class);
        bytes.extend(90_u32.to_le_bytes());
        bytes.extend(ENVELOPE_CLASS_ID);
        bytes.extend(30_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in [1.0_f64, 2.0, 3.0, 4.0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(31_i32.to_le_bytes());
        bytes.extend(empty_string);
        for tag in 35_i32..=37 {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend((tag as u16).to_le_bytes());
        }
        bytes.extend([0xff; 8]);

        let mut refs = RefTable10_1::new();
        refs.register(90);
        let saved = refs.clone();
        let mut reader = Reader::new(&bytes);
        let tail = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs).unwrap();
        let records = tail.raster_io.unwrap();
        let Some(RasterDatasetNameEntryBody10_1::Cache(name)) = records.tag30_name.as_deref()
        else {
            panic!("应读取无键 CacheRasterDatasetName 对象");
        };
        assert_eq!(name.version, 1);
        assert!(
            matches!(&name.workspace, RasterDatasetWorkspaceSlot10_1::WorkspaceName(body)
            if body.key == 29)
        );
        assert_eq!(
            name.tile_reader,
            ObjectEntry10_1::BackReference {
                class_id: tile_class,
                key: 90
            }
        );
        assert_eq!(
            name.envelope.body.as_ref().unwrap().coords,
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(records.shorts_35_to_37, [35, 36, 37]);
        assert!(refs.contains(29) && refs.contains(30));
        assert_eq!(refs.len(), 3);
        assert_eq!(reader.remaining(), 0);

        let mut with_tile = bytes.clone();
        let mut tile_body = vec![1, 0, 1];
        tile_body.extend([0; 16]); // IColor
        tile_body.extend(empty_string);
        tile_body.extend([0; 16 * 4]); // 配置三槽与服务名称。
        tile_body.extend(0_i16.to_le_bytes());
        with_tile.splice(tile_offset + 20..tile_offset + 20, tile_body);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&with_tile);
        let records = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs)
            .unwrap()
            .raster_io
            .unwrap();
        let Some(RasterDatasetNameEntryBody10_1::Cache(name)) = records.tag30_name.as_deref()
        else {
            panic!("应读取 CacheRasterDatasetName");
        };
        assert!(matches!(
            name.tile_reader_body.as_deref(),
            Some(
                crate::layers::cache_tile_reader::CacheTileReaderBody10_1::MapServer {
                    rest: true,
                    rest_selector: Some(0),
                    ..
                }
            )
        ));
        assert_eq!(
            name.envelope.body.as_ref().unwrap().coords,
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(records.shorts_35_to_37, [35, 36, 37]);
        assert!(refs.contains(90) && refs.contains(29) && refs.contains(30));
        assert_eq!(reader.remaining(), 0);

        let mut unknown = bytes.clone();
        unknown[tile_offset..tile_offset + 16].fill(0x42);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&unknown);
        assert_eq!(
            read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
        assert!(refs.is_empty());
        assert_eq!(reader.position(), 0);

        let mut no_tile = bytes.clone();
        no_tile.splice(tile_offset..tile_offset + 20, [0; 16]);
        let mut refs = RefTable10_1::new();
        let records = read_raster_layer_tail_10_1(&mut Reader::new(&no_tile), 1, 4096, &mut refs)
            .unwrap()
            .raster_io
            .unwrap();
        assert!(matches!(records.tag30_name.as_deref(),
            Some(RasterDatasetNameEntryBody10_1::Cache(body)) if body.tile_reader == ObjectEntry10_1::Null));

        for end in tile_offset..bytes.len() {
            let mut refs = saved.clone();
            let mut reader = Reader::new(&bytes[..end]);
            assert!(
                read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs).is_err(),
                "end={end}"
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, saved);
        }
    }

    #[test]
    fn raster_io_tag30_reads_unkeyed_name_and_requires_tag31() {
        use crate::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = vec![0; 22]; // v1 标量。
        for value in [0xff00ff_i32, 0, 30, -1] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(1_i32.to_le_bytes());
        let empty_string = [2, 0, 0, 0, 0, 0];
        bytes.extend(empty_string.repeat(4));
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(29_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(empty_string.repeat(3));
        bytes.extend([0; 21]); // 空属性集、空工厂标志及尾 int32。
        let tag31_start = bytes.len();
        bytes.extend(31_i32.to_le_bytes());
        bytes.extend(empty_string);
        let tag31_end = bytes.len();
        for tag in 35_i32..=37 {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend((tag as u16).to_le_bytes());
        }
        bytes.extend([0xff; 8]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let tail = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs).unwrap();
        let records = tail.raster_io.unwrap();
        assert!(matches!(
            records.tag30_name.as_deref(),
            Some(RasterDatasetNameEntryBody10_1::Dataset(_))
        ));
        assert_eq!(records.tag31_string.as_deref(), Some(""));
        assert_eq!(records.shorts_35_to_37, [35, 36, 37]);
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(29));
        assert_eq!(reader.remaining(), 0);

        bytes.drain(tag31_start..tag31_end);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs).is_err());
        assert!(refs.is_empty());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn raster_io_tag30_reads_service_tiled_and_type_names_without_outer_keys() {
        use crate::layers::raster_dataset_name::{
            AIS_IMAGE_SERVICE_NAME_CLASS_ID, INTERNET_TILED_LAYER_NAME_CLASS_ID,
            RASTER_TYPE_NAME_CLASS_ID,
        };

        for class in [
            AIS_IMAGE_SERVICE_NAME_CLASS_ID,
            INTERNET_TILED_LAYER_NAME_CLASS_ID,
            RASTER_TYPE_NAME_CLASS_ID,
        ] {
            let mut bytes = vec![0; 22];
            for value in [0xff00ff_i32, 0, 30, -1] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(class);
            let empty_string = [2, 0, 0, 0, 0, 0];
            match class {
                AIS_IMAGE_SERVICE_NAME_CLASS_ID => {
                    bytes.extend(1_i32.to_le_bytes());
                    bytes.extend(empty_string.repeat(3));
                }
                INTERNET_TILED_LAYER_NAME_CLASS_ID | RASTER_TYPE_NAME_CLASS_ID => {
                    bytes.extend(1_u16.to_le_bytes());
                    bytes.extend(empty_string);
                    if class == RASTER_TYPE_NAME_CLASS_ID {
                        bytes.extend(empty_string);
                    }
                    bytes.extend(8_i32.to_le_bytes());
                    bytes.extend([0; 16]);
                }
                _ => unreachable!(),
            }
            bytes.extend(31_i32.to_le_bytes());
            bytes.extend(empty_string);
            for tag in 35_i32..=37 {
                bytes.extend(tag.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend((tag as u16).to_le_bytes());
            }
            bytes.extend([0xff; 8]);
            for end in 0..bytes.len() {
                let mut refs = RefTable10_1::new();
                let mut reader = Reader::new(&bytes[..end]);
                let result = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs);
                if end == 22 {
                    // v1 可恰好在标量后结束，整个 RasterIO 段省略。
                    assert!(result.unwrap().raster_io.is_none());
                    assert_eq!(reader.position(), end);
                } else {
                    assert!(result.is_err(), "截断位置 {end}");
                    assert_eq!(reader.position(), 0);
                }
                assert!(refs.is_empty());
            }
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let tail = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs).unwrap();
            let records = tail.raster_io.unwrap();
            match records.tag30_name.as_deref() {
                Some(RasterDatasetNameEntryBody10_1::InternetTiledLayer(name)) => {
                    assert_eq!(class, INTERNET_TILED_LAYER_NAME_CLASS_ID);
                    assert_eq!(name.integer, 8);
                    assert_eq!(name.tile_reader.entry, ObjectEntry10_1::Null);
                }
                Some(RasterDatasetNameEntryBody10_1::AisImageService(name)) => {
                    assert_eq!(class, AIS_IMAGE_SERVICE_NAME_CLASS_ID);
                    assert_eq!(name.version, 1);
                }
                Some(RasterDatasetNameEntryBody10_1::RasterType(name)) => {
                    assert_eq!(class, RASTER_TYPE_NAME_CLASS_ID);
                    assert_eq!(name.instance_id, 8);
                    assert_eq!(name.mosaic_dataset.entry, ObjectEntry10_1::Null);
                }
                other => panic!("未分派名称对象：{other:?}"),
            }
            assert_eq!(records.tag31_string.as_deref(), Some(""));
            assert_eq!(records.shorts_35_to_37, [35, 36, 37]);
            assert_eq!(reader.remaining(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn raster_io_tag30_reads_wcs_name_and_nested_property_keys() {
        use crate::layers::wcs_name::{WCS_CONNECTION_NAME_CLASS_ID, WCS_COVERAGE_NAME_CLASS_ID};
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        for version in [1_u16, 3] {
            let mut bytes = vec![0; 22];
            for value in [0xff00ff_i32, 0, 30, -1] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(WCS_COVERAGE_NAME_CLASS_ID);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([1, 0, 0, 0, b'c', 0, 0, 0]);
            bytes.extend(WCS_CONNECTION_NAME_CLASS_ID);
            bytes.extend(81_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(82_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            if version >= 2 {
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(82_u32.to_le_bytes());
            }
            bytes.extend(31_i32.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]);
            for tag in 35_i32..=37 {
                bytes.extend(tag.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend((tag as u16).to_le_bytes());
            }
            bytes.extend([0xff; 8]);

            for end in 30..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                let mut refs = RefTable10_1::new();
                assert!(read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs).is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let records = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs)
                .unwrap()
                .raster_io
                .unwrap();
            let Some(RasterDatasetNameEntryBody10_1::WcsCoverage(name)) =
                records.tag30_name.as_deref()
            else {
                panic!("应读取无键 WCSCoverageName");
            };
            assert_eq!(name.identifier, "c");
            let connection = name.connection.body.as_ref().unwrap();
            assert_eq!(connection.properties.body.as_ref().unwrap().count, 0);
            assert_eq!(
                connection
                    .extra_properties
                    .as_ref()
                    .map(|slot| slot.entry.clone()),
                (version >= 2).then_some(ObjectEntry10_1::BackReference {
                    class_id: PROPERTY_SET_CLASS_ID,
                    key: 82,
                }),
            );
            assert_eq!(records.shorts_35_to_37, [35, 36, 37]);
            assert!(refs.contains(81) && refs.contains(82));
            assert_eq!(refs.len(), 2);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn raster_io_tag30_reads_wms_and_wmts_connection_names() {
        use crate::layers::wcs_name::{
            WMS_CONNECTION_NAME_CLASS_ID, WMTS_CONNECTION_NAME_CLASS_ID,
        };
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        for class_id in [WMS_CONNECTION_NAME_CLASS_ID, WMTS_CONNECTION_NAME_CLASS_ID] {
            for version in [1_u16, 2, 3] {
                let mut bytes = vec![0; 22];
                for value in [0xff00ff_i32, 0, 30, -1] {
                    bytes.extend(value.to_le_bytes());
                }
                bytes.extend(class_id);
                bytes.extend(version.to_le_bytes());
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(81_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
                if version >= 2 {
                    bytes.extend(PROPERTY_SET_CLASS_ID);
                    bytes.extend(81_u32.to_le_bytes());
                }
                bytes.extend(31_i32.to_le_bytes());
                bytes.extend([2, 0, 0, 0, 0, 0]);
                for tag in 35_i32..=37 {
                    bytes.extend(tag.to_le_bytes());
                    bytes.extend(2_i32.to_le_bytes());
                    bytes.extend((tag as u16).to_le_bytes());
                }
                bytes.extend([0xff; 8]);

                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let records = read_raster_layer_tail_10_1(&mut reader, 1, 4096, &mut refs)
                    .unwrap()
                    .raster_io
                    .unwrap();
                let Some(RasterDatasetNameEntryBody10_1::OgcConnection(name)) =
                    records.tag30_name.as_deref()
                else {
                    panic!("应读取无键 OGC 连接名称");
                };
                assert_eq!(name.class_id, class_id);
                assert_eq!(name.version, version as i16);
                assert_eq!(name.properties.body.as_ref().unwrap().count, 0);
                assert_eq!(
                    name.extra_properties
                        .as_ref()
                        .map(|slot| slot.entry.clone()),
                    (version >= 2).then_some(ObjectEntry10_1::BackReference {
                        class_id: PROPERTY_SET_CLASS_ID,
                        key: 81,
                    }),
                );
                assert_eq!(records.shorts_35_to_37, [35, 36, 37]);
                assert!(refs.contains(81));
                assert_eq!(reader.remaining(), 0);
            }
        }
    }

    #[test]
    fn template_extension_shares_nested_keys_in_both_layer_encodings() {
        use crate::layers::annotate_props::EtmEntry10_1;
        use crate::layers::edit_template::EDIT_TEMPLATE_CLASS_ID;

        let mut manager = 2_u16.to_le_bytes().to_vec();
        manager.extend(1_i32.to_le_bytes());
        manager.extend(EDIT_TEMPLATE_CLASS_ID);
        manager.extend(83_u32.to_le_bytes());
        manager.extend(2_u16.to_le_bytes());
        for _ in 0..2 {
            manager.extend([2, 0, 0, 0, 0, 0]); // 空名称和描述。
        }
        manager.extend([0; 16 + 4]); // 工具 GUID 和字段数量。
        manager.extend([2, 0, 0, 0, 0, 0]); // 空标签。
        manager.push(1);

        for version in [3, 8] {
            let build = |manager_body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(2_i32.to_le_bytes());
                if version == 3 {
                    bytes.extend((manager_body.len() as u64).to_le_bytes());
                    bytes.extend(EDIT_TEMPLATE_MANAGER_CLASS_ID);
                    bytes.extend(manager_body);
                    bytes.extend(0_u64.to_le_bytes());
                    bytes.extend(EDIT_TEMPLATE_MANAGER_CLASS_ID); // 零长度跳过。
                } else {
                    bytes.extend((20 + manager_body.len() as u32).to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                    bytes.extend(EDIT_TEMPLATE_MANAGER_CLASS_ID);
                    bytes.extend(82_u32.to_le_bytes());
                    bytes.extend(manager_body);
                    bytes.extend(20_u32.to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                    bytes.extend(EDIT_TEMPLATE_CLASS_ID); // 回引前一扩展登记的嵌套键。
                    bytes.extend(83_u32.to_le_bytes());
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            let bytes = build(&manager);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            let parsed = if version == 3 {
                assert!(tail.legacy_extensions[1].edit_template_manager.is_none());
                assert_eq!(refs.len(), 1); // 旧扩展外壳无对象键。
                tail.legacy_extensions[0].edit_template_manager.as_ref()
            } else {
                assert!(matches!(
                    tail.extra_safe_extensions[1].entry,
                    ObjectEntry10_1::BackReference { key: 83, .. }
                ));
                assert!(refs.contains(82));
                tail.extra_safe_extensions[0].edit_template_manager.as_ref()
            };
            let parsed = parsed.unwrap();
            assert!(
                matches!(&parsed.entries[..], [EtmEntry10_1::EditTemplate(template)] if template.key == 83)
            );
            assert_eq!(parsed.tail_byte, Some(1));
            assert!(refs.contains(83));
            assert_eq!(reader.remaining(), 0);

            let mut too_long = manager.clone();
            too_long.push(0);
            for body in [&manager[..manager.len() - 1], &too_long] {
                let bytes = build(body);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn server_extension_resolves_property_keys_in_both_encodings() {
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        let mut first = 1_u16.to_le_bytes().to_vec();
        first.extend(PROPERTY_SET_CLASS_ID);
        first.extend(83_u32.to_le_bytes());
        let second = first.clone();
        first.extend(1_u16.to_le_bytes());
        first.extend(0_u32.to_le_bytes());

        for version in [3, 8] {
            let build = |first_body: &[u8], second_body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(2_i32.to_le_bytes());
                for (key, body) in [(81_u32, first_body), (82, second_body)] {
                    if version == 3 {
                        bytes.extend((body.len() as u64).to_le_bytes());
                        bytes.extend(SERVER_LAYER_EXTENSION_CLASS_ID);
                    } else {
                        bytes.extend((20 + body.len() as u32).to_le_bytes());
                        bytes.extend(0_u32.to_le_bytes());
                        bytes.extend(SERVER_LAYER_EXTENSION_CLASS_ID);
                        bytes.extend(key.to_le_bytes());
                    }
                    bytes.extend(body);
                }
                if version == 8 {
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            let mut reference = second.clone();
            reference[2..18].copy_from_slice(&SERVER_LAYER_EXTENSION_CLASS_ID);
            let bytes = build(&first, &reference);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            let parsed = if version == 3 {
                assert_eq!(refs.len(), 1);
                [
                    tail.legacy_extensions[0].server_layer_extension.as_deref(),
                    tail.legacy_extensions[1].server_layer_extension.as_deref(),
                ]
            } else {
                assert_eq!(refs.len(), 3);
                [
                    tail.extra_safe_extensions[0]
                        .server_layer_extension
                        .as_deref(),
                    tail.extra_safe_extensions[1]
                        .server_layer_extension
                        .as_deref(),
                ]
            };
            let first_parsed = parsed[0].unwrap();
            assert_eq!(first_parsed.properties.body.as_ref().unwrap().count, 0);
            assert!(matches!(
                parsed[1].unwrap().properties.entry,
                ObjectEntry10_1::BackReference {
                    key: 83,
                    class_id: PROPERTY_SET_CLASS_ID
                }
            ));
            assert!(refs.contains(83));
            assert_eq!(reader.remaining(), 0);

            if version == 3 {
                let mut reader = Reader::new(&bytes[26..]);
                let mut refs = RefTable10_1::new();
                let entries = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut reader,
                    4096,
                    &mut refs,
                )
                .unwrap();
                assert_eq!(entries[1].server_layer_extension.as_deref(), parsed[1]);
                assert!(refs.contains(83));
                assert_eq!(reader.remaining(), 0);
            }

            reference[18..22].copy_from_slice(&84_u32.to_le_bytes());
            let mut existing = SERVER_LAYER_EXTENSION_CLASS_ID.to_vec();
            existing.extend(84_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&existing), &mut refs).unwrap();
            let invalid = build(&first, &reference);
            let mut reader = Reader::new(&invalid);
            assert_eq!(
                read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
            assert!(refs.contains(84));

            let mut too_long = first.clone();
            too_long.push(0);
            for body in [&first[..first.len() - 1], too_long.as_slice()] {
                let bytes = build(body, &second);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn feature_service_edit_extension_shares_name_keys_in_both_encodings() {
        use crate::maps::standalone_table::TABLE_NAME_CLASS_ID;
        use crate::objects::dataset_name_more::{DatasetNameMoreBody10_1, DatasetNameMoreSlot10_1};

        let mut first = 1_u16.to_le_bytes().to_vec();
        first.extend(TABLE_NAME_CLASS_ID);
        first.extend(83_u32.to_le_bytes());
        let mut second = first.clone();
        first.extend(1_u16.to_le_bytes());
        for _ in 0..3 {
            first.extend(2_u32.to_le_bytes());
            first.extend(0_u16.to_le_bytes());
        }
        first.extend([0; 16]); // WorkspaceName 空槽。
        for body in [&mut first, &mut second] {
            body.extend(TABLE_NAME_CLASS_ID);
            body.extend(83_u32.to_le_bytes());
            body.extend(1_u32.to_le_bytes()); // BSTR 码元数不计结束零。
            body.extend(('表' as u16).to_le_bytes());
            body.extend(0_u16.to_le_bytes());
        }

        for version in [3, 8] {
            let build = |second_body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(2_i32.to_le_bytes());
                for (key, body) in [(81_u32, first.as_slice()), (82, second_body)] {
                    bytes.extend(
                        ((body.len() + if version == 8 { 20 } else { 0 }) as u64).to_le_bytes(),
                    );
                    bytes.extend(FEATURE_SERVICE_LAYER_EDIT_EXTENSION_CLASS_ID);
                    if version == 8 {
                        bytes.extend(key.to_le_bytes());
                    }
                    bytes.extend(body);
                }
                if version == 8 {
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            let bytes = build(&second);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            let parsed = if version == 3 {
                assert_eq!(refs.len(), 1);
                std::array::from_fn::<_, 2, _>(|i| {
                    tail.legacy_extensions[i]
                        .feature_service_layer_edit_extension
                        .as_deref()
                        .unwrap()
                })
            } else {
                assert_eq!(refs.len(), 3);
                std::array::from_fn::<_, 2, _>(|i| {
                    tail.extra_safe_extensions[i]
                        .feature_service_layer_edit_extension
                        .as_deref()
                        .unwrap()
                })
            };
            assert!(matches!(
                parsed[0].names[0],
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Table(_))
            ));
            for name in [
                &parsed[0].names[1],
                &parsed[1].names[0],
                &parsed[1].names[1],
            ] {
                assert!(matches!(
                    name,
                    DatasetNameMoreSlot10_1::BackReference { key: 83, .. }
                ));
            }
            assert_eq!(parsed[1].text, "表");
            assert!(refs.contains(83));
            assert_eq!(reader.remaining(), 0);

            if version == 3 {
                let mut reader = Reader::new(&bytes[26..]);
                let mut refs = RefTable10_1::new();
                let entries = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut reader,
                    4096,
                    &mut refs,
                )
                .unwrap();
                assert_eq!(
                    entries[1].feature_service_layer_edit_extension.as_deref(),
                    Some(parsed[1])
                );
                assert_eq!(reader.remaining(), 0);
            }

            let bytes = build(&second[..second.len() - 1]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            assert!(read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn route_extension_registers_nested_objects_in_both_encodings() {
        use crate::layers::route_layer_extension::{
            ROUTE_ANOMALY_PROPERTIES_CLASS_ID, ROUTE_IDENTIFY_PROPERTIES_CLASS_ID,
        };
        use crate::objects::display_symbol::{SIMPLE_LINE_SYMBOL_CLASS_ID, SymbolSlotFull10_1};
        use crate::objects::query_filter::{
            QUERY_FILTER_CLASS_ID, QueryFilterBody10_1, QueryFilterObject10_1,
        };

        let empty = [2, 0, 0, 0, 0, 0]; // 空 AfCore String。
        let mut first = 1_i16.to_le_bytes().to_vec();
        first.extend(empty);
        first.extend(ROUTE_ANOMALY_PROPERTIES_CLASS_ID);
        first.extend(83_u32.to_le_bytes());
        first.extend(1_i16.to_le_bytes());
        for value in [0.25_f64, 1.5] {
            first.extend(value.to_le_bytes());
        }
        for value in [1_i16, 0, -1, 2] {
            first.extend(value.to_le_bytes());
        }
        first.extend(QUERY_FILTER_CLASS_ID);
        first.extend(84_u32.to_le_bytes());
        first.extend(1_u16.to_le_bytes());
        for _ in 0..3 {
            first.extend(empty);
        }
        first.extend([0; 16 + 8 + 16]); // 空空间参考、分辨率和空 FIDSet。
        let count_pos = first.len();
        first.extend(2_i32.to_le_bytes());
        first.extend(SIMPLE_LINE_SYMBOL_CLASS_ID);
        first.extend(85_u32.to_le_bytes());
        first.extend(1_u16.to_le_bytes());
        first.extend([0; 16 + 8 + 12]); // 空颜色、线宽和三个整数。
        first.extend(SIMPLE_LINE_SYMBOL_CLASS_ID);
        first.extend(85_u32.to_le_bytes());
        first.extend(ROUTE_IDENTIFY_PROPERTIES_CLASS_ID);
        first.extend(86_u32.to_le_bytes());
        first.extend(1_i16.to_le_bytes());
        for _ in 0..6 {
            first.extend(empty);
        }
        first.extend((-3_i32).to_le_bytes());

        let mut second = 0_i16.to_le_bytes().to_vec();
        second.extend(empty);
        second.extend(ROUTE_ANOMALY_PROPERTIES_CLASS_ID);
        second.extend(83_u32.to_le_bytes());
        second.extend(ROUTE_IDENTIFY_PROPERTIES_CLASS_ID);
        second.extend(86_u32.to_le_bytes());

        for version in [3, 8] {
            let build = |first_body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(2_i32.to_le_bytes());
                for (key, body) in [(81_u32, first_body), (82, second.as_slice())] {
                    if version == 3 {
                        bytes.extend((body.len() as u64).to_le_bytes());
                        bytes.extend(ROUTE_LAYER_EXTENSION_CLASS_ID);
                    } else {
                        bytes.extend((20 + body.len() as u32).to_le_bytes());
                        bytes.extend(0_u32.to_le_bytes());
                        bytes.extend(ROUTE_LAYER_EXTENSION_CLASS_ID);
                        bytes.extend(key.to_le_bytes());
                    }
                    bytes.extend(body);
                }
                if version == 8 {
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            let bytes = build(&first);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            let parsed = if version == 3 {
                assert_eq!(refs.len(), 4);
                let mut extensions = Reader::new(&bytes[26..]);
                let mut shared_refs = RefTable10_1::new();
                let shared = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut extensions,
                    4096,
                    &mut shared_refs,
                )
                .unwrap();
                assert_eq!(extensions.remaining(), 0);
                assert_eq!(shared_refs.len(), refs.len());
                assert_eq!(
                    shared[0].route_layer_extension,
                    tail.legacy_extensions[0].route_layer_extension
                );
                [
                    tail.legacy_extensions[0]
                        .route_layer_extension
                        .as_deref()
                        .unwrap(),
                    tail.legacy_extensions[1]
                        .route_layer_extension
                        .as_deref()
                        .unwrap(),
                ]
            } else {
                assert_eq!(refs.len(), 6);
                [
                    tail.extra_safe_extensions[0]
                        .route_layer_extension
                        .as_deref()
                        .unwrap(),
                    tail.extra_safe_extensions[1]
                        .route_layer_extension
                        .as_deref()
                        .unwrap(),
                ]
            };
            let anomaly = parsed[0].anomaly.body.as_ref().unwrap();
            assert_eq!(anomaly.doubles, [0.25, 1.5]);
            assert_eq!(anomaly.shorts, [1, 0, -1, 2]);
            assert!(
                matches!(&anomaly.filter, QueryFilterObject10_1::Body { key: 84, body } if matches!(**body, QueryFilterBody10_1::Query(_)))
            );
            assert!(
                matches!(&anomaly.symbols[..], [SymbolSlotFull10_1::SimpleLineSymbol(line), SymbolSlotFull10_1::BackReference { key: 85, .. }] if line.key == 85)
            );
            assert_eq!(parsed[0].identify.body.as_ref().unwrap().long, -3);
            assert!(matches!(
                parsed[1].anomaly.entry,
                ObjectEntry10_1::BackReference { key: 83, .. }
            ));
            assert!(matches!(
                parsed[1].identify.entry,
                ObjectEntry10_1::BackReference { key: 86, .. }
            ));
            assert!((83..=86).all(|key| refs.contains(key)));
            assert_eq!(reader.remaining(), 0);

            let mut oversized_count = first.clone();
            oversized_count[count_pos..count_pos + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            let mut unknown_class = first.clone();
            unknown_class[count_pos + 4..count_pos + 20].fill(0x7f);
            let mut trailing = first.clone();
            trailing.push(0);
            for bad in [
                &first[..first.len() - 1],
                &oversized_count,
                &unknown_class,
                &trailing,
            ] {
                let bytes = build(bad);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                refs.register(7);
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert_eq!(refs.len(), 1);
                assert!(refs.contains(7));
            }
        }
    }

    #[test]
    fn representation_extension_reads_signed_version_in_both_encodings() {
        let class = REPRESENTATION_LAYER_EXTENSION_CLASS_ID;
        for version in [3, 8] {
            let build = |body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(2_i32.to_le_bytes());
                if version == 3 {
                    bytes.extend((body.len() as u64).to_le_bytes());
                    bytes.extend(class);
                    bytes.extend(body);
                    bytes.extend(0_u64.to_le_bytes());
                    bytes.extend(class);
                } else {
                    for region in [body, &[]] {
                        bytes.extend((20 + region.len() as u32).to_le_bytes());
                        bytes.extend(0_u32.to_le_bytes());
                        bytes.extend(class);
                        bytes.extend(81_u32.to_le_bytes());
                        bytes.extend(region);
                    }
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };

            let check_version = |value: i16| {
                let bytes = build(&value.to_le_bytes());
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let tail =
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
                if version == 3 {
                    assert_eq!(
                        tail.legacy_extensions[0].representation_layer_extension_version,
                        Some(value)
                    );
                    assert_eq!(
                        tail.legacy_extensions[1].representation_layer_extension_version,
                        None
                    );
                    assert!(refs.is_empty());

                    let mut reader = Reader::new(&bytes[26..]);
                    let entries = crate::layers::layer_extensions::read_layer_extensions_10_1(
                        &mut reader,
                        4096,
                        &mut refs,
                    )
                    .unwrap();
                    assert_eq!(
                        entries[0].representation_layer_extension_version,
                        Some(value)
                    );
                    assert_eq!(entries[1].representation_layer_extension_version, None);
                    assert_eq!(reader.remaining(), 0);
                } else {
                    assert_eq!(
                        tail.extra_safe_extensions[0].representation_layer_extension_version,
                        Some(value)
                    );
                    assert_eq!(
                        tail.extra_safe_extensions[1].representation_layer_extension_version,
                        None
                    );
                    assert_eq!(
                        tail.extra_safe_extensions[1].entry,
                        ObjectEntry10_1::BackReference {
                            class_id: class,
                            key: 81
                        }
                    );
                    assert_eq!(refs.len(), 1);
                }
                assert_eq!(reader.remaining(), 0);
            };

            for value in [-1, 0, 1] {
                check_version(value);
            }
            for body in [&[1][..], &[1, 0, 0], &[2, 0]] {
                let bytes = build(body);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn raster_related_name_reads_table_before_relationship_list() {
        use crate::maps::standalone_table::OBJECT_CLASS_NAME_CLASS_ID;
        use crate::maps::standalone_table::{FGDB_TABLE_NAME_CLASS_ID, TABLE_NAME_CLASS_ID};

        for (class_id, wrapper) in [
            (TABLE_NAME_CLASS_ID, None),
            (OBJECT_CLASS_NAME_CLASS_ID, None),
            (FGDB_TABLE_NAME_CLASS_ID, Some((1_u16, 17_u16))),
        ] {
            let mut bytes = vec![0; 26 + 4 + 8 + 32];
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(class_id);
            bytes.extend(71_u32.to_le_bytes());
            if let Some((version, flags)) = wrapper {
                bytes.extend(version.to_le_bytes());
                bytes.extend(flags.to_le_bytes());
            }
            bytes.extend(1_u16.to_le_bytes());
            for _ in 0..3 {
                bytes.extend(2_u32.to_le_bytes());
                bytes.extend(0_u16.to_le_bytes());
            }
            bytes.extend([0; 16]); // 空工作空间。
            let list_offset = bytes.len();
            bytes.extend(0_i32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let tail = read_raster_layer_tail_10_1(&mut reader, 6, 4096, &mut refs).unwrap();
            assert!(matches!(
                tail.related_name.unwrap().body,
                Some(RasterLayerRelationshipBody10_1::TableName(body)) if body.fgdb_wrapper == wrapper
            ));
            assert!(tail.related_objects.is_empty());
            assert_eq!(reader.position(), bytes.len());
            assert_eq!(refs.registered_class(71), Some(class_id));

            bytes[list_offset..].copy_from_slice(&1_i32.to_le_bytes());
            bytes.extend(class_id);
            bytes.extend(71_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            assert_eq!(
                read_raster_layer_tail_10_1(&mut reader, 6, 4096, &mut refs)
                    .unwrap_err()
                    .kind(),
                ErrorKind::Unsupported
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn raster_related_name_rejects_null_with_nonzero_presence_flag() {
        let mut bytes = vec![0; 26 + 4 + 8 + 32];
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(0_i32.to_le_bytes());

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let error = read_raster_layer_tail_10_1(&mut reader, 6, 4096, &mut refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn raster_relationship_list_checks_registered_class() {
        for registered in [RELATIONSHIP_CLASS_NAME_CLASS_ID, ENVELOPE_CLASS_ID] {
            let mut previous = registered.to_vec();
            previous.extend(71_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&previous), &mut refs).unwrap();
            let before = refs.clone();

            let mut bytes = vec![0; 26 + 4 + 8 + 32 + 4];
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(RELATIONSHIP_CLASS_NAME_CLASS_ID);
            bytes.extend(71_u32.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let result = read_raster_layer_tail_10_1(&mut reader, 6, 4096, &mut refs);
            if registered == RELATIONSHIP_CLASS_NAME_CLASS_ID {
                assert_eq!(result.unwrap().related_objects.len(), 1);
                assert_eq!(reader.position(), bytes.len());
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
            }
            assert_eq!(refs, before);
        }
    }

    #[test]
    fn nonpositive_extension_counts_reach_later_raster_fields() {
        for count in [0_i32, -1, i32::MIN] {
            let mut bytes = vec![0; 26];
            bytes.extend(count.to_le_bytes());
            bytes.extend(2.5_f64.to_le_bytes());
            bytes.extend([0; 32 + 8 + 16]);
            for version in [7, 8] {
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                let tail =
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
                assert!(tail.legacy_extensions.is_empty());
                assert!(tail.extra_safe_extensions.is_empty());
                assert_eq!(tail.double_104, Some(2.5));
                assert_eq!(reader.position(), bytes.len());
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn old_tail_versions_end_at_their_own_fields() {
        for version in 0..=12 {
            let mut bytes = vec![0; 22];
            if version >= 2 {
                bytes.extend(101_i32.to_le_bytes());
            }
            if (3..8).contains(&version) {
                bytes.extend(1_i32.to_le_bytes());
                bytes.extend(3_i64.to_le_bytes());
                bytes.extend([0x49; 16]);
                bytes.extend([2, 4, 6]);
            } else if version == 8 {
                bytes.extend(3_i32.to_le_bytes());
                bytes.extend([0; 8]); // ExtraSafe 空项。
                bytes.extend(23_u32.to_le_bytes()); // A 主边界。
                bytes.extend(0_u32.to_le_bytes());
                bytes.extend([0x49; 16]);
                bytes.extend(81_u32.to_le_bytes());
                bytes.extend([2, 4, 6]);
                bytes.extend(0_u32.to_le_bytes());
                bytes.extend(20_u32.to_le_bytes()); // B 同步边界。
                bytes.extend([0x50; 16]);
                bytes.extend(81_u32.to_le_bytes());
            } else if version >= 9 {
                bytes.extend(0_i32.to_le_bytes());
            }
            if version >= 4 {
                bytes.extend(2.5_f64.to_le_bytes());
            }
            if version >= 5 {
                bytes.extend([0x3f; 32]);
            }
            if version >= 6 {
                bytes.extend([0; 8]);
            }
            if version >= 7 {
                bytes.extend([0; 16]);
            }
            let optional_records_start = bytes.len();
            // v9 带 RasterIO；其余输入测试 InitCheck 未命中。
            if version == 9 {
                bytes.extend(0xff00ff_i32.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
                for tag in 35_i32..=37 {
                    bytes.extend(tag.to_le_bytes());
                    bytes.extend(2_i32.to_le_bytes());
                    bytes.extend((tag as u16).to_le_bytes());
                }
                bytes.extend([0xff; 8]);
            }
            if version >= 10 {
                bytes.extend(4_u32.to_le_bytes());
                bytes.extend((b'x' as u16).to_le_bytes());
                bytes.extend(0_u16.to_le_bytes());
            }
            if version == 10 {
                bytes.extend(87_u16.to_le_bytes());
            }
            if version >= 11 {
                bytes.extend(0.25_f64.to_le_bytes());
                bytes.extend(0.5_f64.to_le_bytes());
            }

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            assert_eq!(tail.long_100, (version >= 2).then_some(101));
            assert_eq!(tail.double_104, (version >= 4).then_some(2.5));
            assert_eq!(
                tail.wks_bits,
                (version >= 5).then_some([0x3f3f3f3f3f3f3f3f; 4])
            );
            assert_eq!(tail.envelope_f0.is_some(), version >= 7);
            let has_legacy_extension = (3..8).contains(&version);
            assert_eq!(
                tail.legacy_extensions.len(),
                usize::from(has_legacy_extension)
            );
            if has_legacy_extension {
                assert_eq!(tail.legacy_extensions[0].class_id, [0x49; 16]);
                assert_eq!(tail.legacy_extensions[0].raw_body, [2, 4, 6]);
                let mut invalid_size = bytes.clone();
                invalid_size[30..38].copy_from_slice(&u64::MAX.to_le_bytes());
                let mut reader = Reader::new(&invalid_size);
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
            }
            if version == 8 {
                let extensions = &tail.extra_safe_extensions;
                assert_eq!(extensions.len(), 3);
                assert_eq!(extensions[0].entry, ObjectEntry10_1::Null);
                assert_eq!(extensions[1].prefix, [23, 0]);
                assert_eq!(extensions[1].raw_body, [2, 4, 6]);
                assert_eq!(extensions[2].prefix, [0, 20]);
                assert_eq!(
                    extensions[2].entry,
                    ObjectEntry10_1::BackReference {
                        key: 81,
                        class_id: [0x49; 16]
                    },
                );
                assert!(extensions[2].raw_body.is_empty());
                assert!(refs.contains(81));
            }
            assert_eq!(tail.string_128.as_deref(), (version >= 10).then_some("x"));
            assert_eq!(tail.short_v10, (version == 10).then_some(87));
            assert_eq!(tail.doubles_12c_134, (version >= 11).then_some([0.25, 0.5]));
            assert!(tail.geodata_xform_v13.is_none() && tail.envelope_v13.is_none());
            assert_eq!(
                tail.raster_io.map(|io| io.shorts_35_to_37),
                (version == 9).then_some([35, 36, 37])
            );
            assert_eq!(reader.remaining(), 0);
            for cut in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                if version == 9 && cut == optional_records_start {
                    // 完全未写 RasterIO 的 v9 也是原生可接受的字节形态。
                    continue;
                }
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn v12_transform_pair_must_end_at_bound() {
        let mut bytes = vec![0; 94]; // 至 IEnvelope 空槽为止。
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend([0; 16]); // v11 的两个 double。
        let pair_start = bytes.len();
        bytes.extend(crate::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID);
        bytes.extend(53_u32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 16]); // Identity 的空空间参考。
        bytes.extend([0; 16]); // 第二个空包络。
        let end = bytes.len();
        bytes.push(0xff);

        for bound in (pair_start..=end + 1).rev() {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..bound]);
            let tail = read_raster_layer_tail_10_1(&mut reader, 12, 4096, &mut refs);
            if bound == pair_start || bound == end {
                let tail = tail.unwrap();
                assert_eq!(reader.remaining(), 0);
                assert_eq!(tail.geodata_xform_v13.is_some(), bound == end);
                assert_eq!(tail.envelope_v13.is_some(), bound == end);
                assert_eq!(refs.contains(53), bound == end);
            } else {
                assert!(tail.is_err(), "bound={bound}");
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }
}
