//! ArcMap Desktop 10.1 RasterDatasetName 对象体读取子集。
//!
//! 证据为 RasterDB.dll 10.1 CAB 映像（SHA-256
//! `97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`，PE32/i386，
//! ImageBase `0x7bfb0000`）：`Load 0x7c0ba5b0`、`Save 0x7c0ba790`、
//! `GetClassID 0x7c0ba570`。REA/Ghidra 导出记录为
//! `temp/luna-followup-20261118/raster-native/rasternameread.json`；函数边界与关键
//! IAT 分支由原始指令复核。样本只用于交叉验证，不用样本值命名字段。
//!
//! 调用者先读对象头并登记外层键，再把 RasterDatasetName 对象体的有界 Reader
//! 传入本模块。读取器最多消费四条顺序 String、IWorkspaceName 槽及该对象体
//! 版本门控制的两个 int32；未知嵌套类是成功停止点。

use super::cache_tile_reader::{CacheTileReaderBody10_1, read_tile_reader};
use super::feature_layer::{FeatureClassNamePrefix10_1, read_feature_class_name_payload};
use super::fgdb_raster_dataset_name::{
    FGDB_RASTER_DATASET_NAME_CLASS_ID, FgdbRasterDatasetNamePrefix10_1, FgdbRasterIoPrefix10_1,
    read_fgdb_raster_dataset_name_prefix_10_1, read_fgdb_raster_io_prefix_10_1,
};
use super::image_server_layer_tail::{
    IMAGE_SERVER_NAME_CLASS_ID, ImageServerName10_1, read_name as read_image_server_name,
};
use super::image_server_rules::{
    RasterFunctionArgumentsBody10_1, RasterFunctionBody10_1, RasterTemplateInput10_1, RuleSlot10_1,
    check_rule_reference, function_max_version, read_function, read_input_body, slot,
};
use super::raster_layer_tail::{RasterLayerEnvelopeSlot10_1, read_envelope_slot};
use super::raster_renderer::{raster_io_record, read_raster_io_end};
use super::wcs_name::{
    OgcConnectionName10_1, WCS_CONNECTION_NAME_CLASS_ID, WCS_COVERAGE_NAME_CLASS_ID,
    WMS_CONNECTION_NAME_CLASS_ID, WMTS_CONNECTION_NAME_CLASS_ID, WcsCoverageName10_1,
    read_ogc_connection_name, read_wcs_coverage_name,
};
use crate::objects::geodata_xform::{GeodataXformSlot10_1, read_geodata_xform_slot_with_depth};
use crate::objects::geometry::{GeometryBody10_1, read_keyed_geometry_body_10_1};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
pub use crate::objects::string_array::StringArraySlot10_1 as RasterStringArraySlot10_1;
use crate::objects::string_array::read_string_array_slot_10_1;
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, WorkspaceNameStop10_1,
    read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io;

/// RasterDatasetName 类标识 `{75BCE6E2-8AF5-478E-8892-FA45CA50AF4D}` 的
/// 16 字节 Windows GUID 内存序。
///
/// RasterDB.dll `GetClassID 0x7c0ba570` 将 `0x7c12745c..0x7c12746b` 的 16 字节
/// 写入输出；同一映像 RTTI 将 `RasterDatasetName@ESRI` 类型描述符连到 COL
/// `0x7c148bc0` 和虚表 `0x7c133608`。样本键 1545 的父对象头类标识与此值相同。
pub const RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0xe2, 0xe6, 0xbc, 0x75, 0xf5, 0x8a, 0x8e, 0x47, 0x88, 0x92, 0xfa, 0x45, 0xca, 0x50, 0xaf, 0x4d,
];
/// MapServerLayer.dll 10.1 CacheRasterDatasetName。
pub const CACHE_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x24, 0xb0, 0x3e, 0x34, 0x47, 0x5f, 0x13, 0x48, 0x8d, 0x4c, 0x66, 0xaa, 0x8a, 0x6d, 0x2f, 0x19,
];
pub const RASTER_DATASET_NAME83_CLASS_ID: [u8; 16] = [
    0x01, 0x0e, 0x36, 0x76, 0x46, 0xec, 0xd1, 0x11, 0x8d, 0x21, 0x00, 0x00, 0xf8, 0x78, 0x05, 0x35,
];
pub const RASTER_BAND_NAME_CLASS_ID: [u8; 16] = [
    0x13, 0xe1, 0x25, 0xbc, 0x8b, 0x16, 0xd2, 0x11, 0x8d, 0x25, 0, 0, 0xf8, 0x78, 0x05, 0x35,
];
pub const RASTER_BAND_COLLECTION_NAME_CLASS_ID: [u8; 16] = [
    0xb5, 0xc1, 0x0f, 0x71, 0x18, 0xa1, 0x5b, 0x44, 0x98, 0xd4, 0xef, 0x7d, 0x56, 0xe1, 0x8a, 0x88,
];
pub const RASTER_CATALOG_NAME_CLASS_ID: [u8; 16] = [
    0xfc, 0x62, 0xff, 0x33, 0xd6, 0xd7, 0xe8, 0x48, 0x82, 0xd9, 0x77, 0xc0, 0xaa, 0xdb, 0xf5, 0xd1,
];
pub const MOSAIC_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x6d, 0xbb, 0xd5, 0x5c, 0x49, 0x65, 0x9e, 0x42, 0x80, 0xf4, 0x0d, 0xd4, 0x73, 0x11, 0x98, 0xd1,
];
pub const ACCESS_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x31, 0xdc, 0xf9, 0xc8, 0x21, 0xd2, 0x35, 0x46, 0x92, 0x3e, 0x1a, 0x55, 0xa5, 0x25, 0xb1, 0xa1,
];
pub const SDE_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0xc5, 0x03, 0xc0, 0xd6, 0x50, 0xa0, 0x8e, 0x43, 0xbb, 0xd4, 0x97, 0x29, 0x4c, 0x3f, 0xdd, 0xc1,
];
pub const NETCDF_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x2b, 0xaa, 0xb4, 0x8c, 0xce, 0x58, 0x8f, 0x43, 0x8c, 0xd8, 0xf5, 0xf0, 0xbe, 0xf4, 0x1a, 0xaf,
];
pub const IS_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0xdd, 0x48, 0x2a, 0x2d, 0xa1, 0xff, 0x6e, 0x4b, 0x96, 0x91, 0xbf, 0xe8, 0xed, 0xba, 0x87, 0x61,
];
pub const FUNCTION_RASTER_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0xcc, 0xe8, 0x07, 0xeb, 0x78, 0xee, 0x8a, 0x43, 0xa2, 0xc1, 0xba, 0x80, 0x45, 0x5a, 0xcb, 0x02,
];
pub const INTERNET_TILED_LAYER_NAME_CLASS_ID: [u8; 16] = [
    0x63, 0xa5, 0x23, 0xa7, 0x3c, 0x0e, 0x84, 0x44, 0xbb, 0xd1, 0x68, 0xb8, 0x19, 0x29, 0xb5, 0xff,
];
pub const AIS_IMAGE_SERVICE_NAME_CLASS_ID: [u8; 16] = [
    0x9e, 0x16, 0x42, 0xb8, 0x85, 0x34, 0x8e, 0x45, 0x95, 0x22, 0x2d, 0x69, 0xdd, 0xea, 0x8b, 0x6c,
];
pub const RASTER_TYPE_NAME_CLASS_ID: [u8; 16] = [
    0x0b, 0x98, 0x7e, 0x50, 0x38, 0xd3, 0x36, 0x46, 0xa7, 0x29, 0xa9, 0x39, 0xbf, 0xca, 0xa1, 0x24,
];
pub(crate) const LONG_ARRAY_CLASS_ID: [u8; 16] = [
    0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0, 0xc0, 0x4f, 0x8e, 0xce, 0x27,
];
pub(super) const MAX_NAME_NESTING: usize = 16;

/// RasterDatasetName 体内 IWorkspaceName 请求槽的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub enum RasterDatasetWorkspaceSlot10_1 {
    /// 空对象头；无对象体。
    Null,
    /// 已在当前对象流登记的键；无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 WorkspaceName 对象及其已确认体字段。
    WorkspaceName(Box<WorkspaceNamePrefix10_1>),
    /// 新对象，但类不是已支持的 WorkspaceName；游标停在该对象头前。
    Unsupported { class_id: [u8; 16], key: u32 },
}

pub(super) fn read_workspace_slot(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterDatasetWorkspaceSlot10_1> {
    use RasterDatasetWorkspaceSlot10_1 as Slot;
    Ok(match read_object_entry_10_1(reader, table)? {
        ObjectEntry10_1::Null => Slot::Null,
        ObjectEntry10_1::BackReference { class_id, key } => match table.registered_class(key) {
            Some(class_id) if class_id != WORKSPACE_NAME_CLASS_ID => {
                Slot::Unsupported { class_id, key }
            }
            registered => Slot::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            },
        },
        ObjectEntry10_1::NewObject {
            class_id: WORKSPACE_NAME_CLASS_ID,
            key,
        } => Slot::WorkspaceName(Box::new(read_workspace_name_body_10_1(
            reader, max, table, key,
        )?)),
        ObjectEntry10_1::NewObject { class_id, key } => Slot::Unsupported { class_id, key },
    })
}

/// RasterDatasetName 对象体的有界停止位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterDatasetNameStop10_1 {
    /// WorkspaceName 体在其 IPropertySet 新对象处遇到未支持的类。
    NestedWorkspaceName(WorkspaceNameStop10_1),
    /// IWorkspaceName 请求槽的新对象类尚无体读取器。
    UnsupportedWorkspaceClass { class_id: [u8; 16], key: u32 },
}

/// RasterDatasetName 10.1 Load 已确认的体字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RasterDatasetNameBody10_1 {
    /// 外层 RasterDatasetName 对象头中的键；调用者负责登记。
    pub key: u32,
    /// Load 先读取的 4 字节原值。Save 侧由 AfStreamGetSaveVersion 产生；
    /// 这里保留整数，不把它改写成字段或成员名称。
    pub version_gate: i32,
    /// 前三条分别对应 Name、NameString 和 Category，第四条保留原文。
    pub strings: Option<[String; 4]>,
    /// 紧随四条 String 的 LoadObject（请求 IWorkspaceName IID）。
    pub workspace: Option<RasterDatasetWorkspaceSlot10_1>,
    /// 已从当前对象流的工作空间回引确认的路径。
    pub workspace_back_reference_path: Option<String>,
    /// `version_gate >= 3` 时读取的 4 字节原值（对象偏移 +0x6c）。
    pub tail_at_6c: Option<i32>,
    /// `version_gate >= 4` 时读取的 4 字节原值（对象偏移 +0x70）。
    pub tail_at_70: Option<i32>,
    /// 成功停止点；`None` 表示已消费本类 Load 当前已闭合的字段序列。
    pub stop: Option<RasterDatasetNameStop10_1>,
}

impl RasterDatasetNameBody10_1 {
    /// IDatasetName::Name 的持久化原文。
    pub fn dataset_name(&self) -> Option<&str> {
        self.strings.as_ref().map(|strings| strings[0].as_str())
    }

    pub fn workspace_path(&self) -> Option<&str> {
        match &self.workspace {
            Some(RasterDatasetWorkspaceSlot10_1::WorkspaceName(body)) => Some(&body.path),
            Some(RasterDatasetWorkspaceSlot10_1::BackReference { .. }) => {
                self.workspace_back_reference_path.as_deref()
            }
            _ => None,
        }
    }
}

/// 从已确定且已由调用者限定范围的 RasterDatasetName 对象体起点读取 10.1 字段。
///
/// `max_bytes` 限制每条 AfCore String。嵌套 IWorkspaceName 新对象按已确认类
/// 标识调用现有 WorkspaceName 读取器；其他新类在对象头处停止。读取以临时游标
/// 和登记表副本进行，格式错误不推进调用者状态；成功停止点提交已确认前缀。
pub fn read_raster_dataset_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RasterDatasetNameBody10_1> {
    let mut trial = reader.clone();
    let mut staged_table = table.clone();
    let version_gate = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    // 原生 Load 回退四字节后将零门值读作字符串长度，String::Read 返回 E_FAIL。
    if version_gate == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "RasterDatasetName 版本门为零，回退后首个 AfCore 字符串长度为零",
        ));
    }

    let strings = [
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
    ];
    let before_nested = trial.clone();
    let mut stop = None;
    let workspace = read_workspace_slot(&mut trial, max_bytes, &mut staged_table)?;
    match &workspace {
        RasterDatasetWorkspaceSlot10_1::WorkspaceName(body) => {
            if let Some(nested_stop) = body.stop {
                stop = Some(RasterDatasetNameStop10_1::NestedWorkspaceName(nested_stop));
            }
        }
        RasterDatasetWorkspaceSlot10_1::Unsupported { class_id, key } => {
            stop = Some(RasterDatasetNameStop10_1::UnsupportedWorkspaceClass {
                class_id: *class_id,
                key: *key,
            });
            trial = before_nested;
            staged_table = table.clone();
        }
        _ => {}
    }

    let (tail_at_6c, tail_at_70) = if stop.is_none() {
        let at_6c = if version_gate >= 3 {
            Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
        } else {
            None
        };
        let at_70 = if version_gate >= 4 {
            Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
        } else {
            None
        };
        (at_6c, at_70)
    } else {
        (None, None)
    };

    let workspace_back_reference_path = match &workspace {
        RasterDatasetWorkspaceSlot10_1::BackReference { class_id, key }
            if *class_id == WORKSPACE_NAME_CLASS_ID =>
        {
            staged_table.workspace_path(*key).map(str::to_owned)
        }
        _ => None,
    };
    *reader = trial;
    *table = staged_table;
    Ok(RasterDatasetNameBody10_1 {
        key,
        version_gate,
        strings: Some(strings),
        workspace: Some(workspace),
        workspace_back_reference_path,
        tail_at_6c,
        tail_at_70,
        stop,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterDatasetNameEntryBody10_1 {
    Dataset(Box<RasterDatasetNameBody10_1>),
    Cache(Box<CacheRasterDatasetName10_1>),
    WcsCoverage(Box<WcsCoverageName10_1>),
    OgcConnection(Box<OgcConnectionName10_1>),
    Dataset83 {
        name: Box<RasterDatasetNameBody10_1>,
        records: Option<RasterDatasetName83Records10_1>,
    },
    FileGeodatabase {
        name: Box<FgdbRasterDatasetNamePrefix10_1>,
        records: Box<FgdbRasterIoPrefix10_1>,
    },
    Band(Box<RasterBandName10_1>),
    BandCollection(Box<RasterBandCollectionName10_1>),
    Catalog(Box<FeatureClassNamePrefix10_1>),
    Mosaic(Box<MosaicDatasetName10_1>),
    Database(Box<DatabaseRasterDatasetName10_1>),
    NetCdf(Box<NetCdfRasterDatasetName10_1>),
    ImageService(Box<IsRasterDatasetName10_1>),
    ImageServerReference(Box<ImageServerName10_1>),
    Function(Box<FunctionRasterDatasetName10_1>),
    InternetTiledLayer(Box<InternetTiledLayerName10_1>),
    AisImageService(AisImageServiceName10_1),
    RasterType(Box<RasterTypeName10_1>),
}

impl RasterDatasetNameEntryBody10_1 {
    pub(super) fn open_without_result(&self) -> bool {
        match self {
            Self::RasterType(name) => {
                name.name.is_empty() && name.mosaic_dataset.entry == ObjectEntry10_1::Null
            }
            // IName::Open 在瓦片读取器为空或整数字段为 -1 时返回 S_FALSE。
            Self::InternetTiledLayer(name) => {
                name.tile_reader.entry == ObjectEntry10_1::Null || name.integer == -1
            }
            // IName::Open 在连接名称为空时直接返回 E_FAIL。
            Self::WcsCoverage(name) => name.connection.entry == ObjectEntry10_1::Null,
            // NetCDF 的工作空间打开助手先检查已加载的工作空间名称。
            Self::NetCdf(name) => matches!(name.workspace, RasterDatasetWorkspaceSlot10_1::Null),
            // 内嵌 IName 为空时，影像服务栅格名称直接返回 E_PENDING。
            Self::ImageService(name) => name.name.entry == ObjectEntry10_1::Null,
            // 空文本和空属性集使 ImageServerName::Open 返回 E_PENDING。
            Self::ImageServerReference(name) => {
                name.text.is_empty() && name.properties == ObjectEntry10_1::Null
            }
            _ => false,
        }
    }

    pub(crate) fn dataset_name_body(&self) -> Option<&RasterDatasetNameBody10_1> {
        match self {
            Self::Dataset(name) | Self::Dataset83 { name, .. } => Some(name),
            Self::FileGeodatabase { name, .. } => Some(&name.delegated_name),
            Self::Database(name) => Some(&name.delegated_name),
            _ => None,
        }
    }
}

/// MapServerLayer.dll 10.1 的 InternetTiledLayerName。
#[derive(Debug, Clone, PartialEq)]
pub struct InternetTiledLayerName10_1 {
    pub version: i16,
    pub name_string: String,
    /// 持久化接口 +0x14 的整数原值。
    pub integer: i32,
    pub tile_reader: RuleSlot10_1<CacheTileReaderBody10_1>,
}

/// ImageServer.dll 10.1 的 AISImageServiceName。
#[derive(Debug, Clone, PartialEq)]
pub struct AisImageServiceName10_1 {
    pub version: i32,
    /// 第一串对应 IName.NameString，后两串原样保留。
    pub strings: [String; 3],
}

/// MosaicDB.dll 10.1 的 RasterTypeName。
#[derive(Debug, Clone, PartialEq)]
pub struct RasterTypeName10_1 {
    pub version: u16,
    pub name_string: String,
    pub name: String,
    pub instance_id: i32,
    pub mosaic_dataset: RuleSlot10_1<MosaicDatasetName10_1>,
}

/// MapServerLayer.dll 10.1 Load 0x79e22a40、Save 0x79e22b20。
#[derive(Debug, Clone, PartialEq)]
pub struct CacheRasterDatasetName10_1 {
    /// 原生无版本门，原样保留。
    pub version: u16,
    pub strings: [String; 2],
    pub workspace: RasterDatasetWorkspaceSlot10_1,
    pub tile_reader: ObjectEntry10_1,
    pub tile_reader_body: Option<Box<CacheTileReaderBody10_1>>,
    pub envelope: RasterLayerEnvelopeSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRasterDatasetName10_1 {
    pub version: u16,
    pub strings: [String; 3],
    pub workspace: RasterDatasetWorkspaceSlot10_1,
    pub path: String,
    pub legacy: Option<FunctionRasterLegacyName10_1>,
    pub function: RuleSlot10_1<RasterFunctionBody10_1>,
    pub input_count: Option<i32>,
    pub inputs: Vec<RuleSlot10_1<FunctionRasterInput10_1>>,
    pub properties: Option<RuleSlot10_1<PropertySetObjectStream10_1>>,
    pub geometry: Option<RuleSlot10_1<GeometryBody10_1>>,
    pub string_40: Option<String>,
    pub string_48: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRasterLegacyName10_1 {
    pub text: String,
    pub long: i32,
    pub name: RasterNameSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionRasterInput10_1 {
    Arguments(RasterFunctionArgumentsBody10_1),
    Name(RasterDatasetNameEntryBody10_1),
    Other(Box<RasterTemplateInput10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsRasterDatasetName10_1 {
    pub version: u16,
    pub strings: [String; 6],
    pub long_44: i32,
    pub workspace: RasterDatasetWorkspaceSlot10_1,
    pub name: RasterNameSlot10_1,
    pub string_50: Option<String>,
    pub string_54: Option<String>,
    pub transform: Option<GeodataXformSlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetCdfRasterDatasetName10_1 {
    pub version: i32,
    pub strings: [String; 8],
    pub legacy: Option<(String, RasterLayerEnvelopeSlot10_1)>,
    pub labels: RasterStringArraySlot10_1,
    pub values: RasterLongArraySlot10_1,
    pub workspace: RasterDatasetWorkspaceSlot10_1,
}

impl NetCdfRasterDatasetName10_1 {
    /// IMDRasterDatasetView::Variable 对应 Load 中的第五条 String。
    pub fn variable(&self) -> &str {
        &self.strings[4]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterLongArraySlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<(u16, Vec<i32>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseRasterDatasetName10_1 {
    pub class_id: [u8; 16],
    pub delegated_name: RasterDatasetNameBody10_1,
    pub records: Option<DatabaseRasterDatasetRecords10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseRasterDatasetRecords10_1 {
    pub longs_1_4: [i32; 2],
    pub strings_2_3_5: [String; 3],
    pub tag6_name: Option<DatabaseRasterTag6Name10_1>,
    pub tag7_long: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseRasterTag6Name10_1 {
    FeatureClass(Box<FeatureClassNamePrefix10_1>),
    Raster {
        class_id: [u8; 16],
        body: RasterDatasetNameEntryBody10_1,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MosaicDatasetName10_1 {
    pub version: u16,
    pub strings: [String; 2],
    pub workspace: RasterDatasetWorkspaceSlot10_1,
    pub short_44: Option<u16>,
    pub long_48: Option<i32>,
    pub string_array: Option<RasterStringArraySlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterBandName10_1 {
    pub has_version_marker: bool,
    pub strings: [String; 3],
    pub selector: u16,
    pub source: RasterBandNameSource10_1,
}

impl RasterBandName10_1 {
    /// 原生成功打开栅格后，传给 IRasterBandCollection::BandByName 的名称。
    pub fn band_lookup_name(&self) -> &str {
        self.strings[0]
            .rsplit_once('\\')
            .map_or(self.strings[0].as_str(), |(_, name)| name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterBandNameSource10_1 {
    Workspace(RasterDatasetWorkspaceSlot10_1),
    Dataset(RasterNameSlot10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterBandCollectionName10_1 {
    pub version: u16,
    pub text: String,
    pub name: RasterNameSlot10_1,
    pub array_entry: ObjectEntry10_1,
    pub array: Option<(u16, Vec<i32>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterNameSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterDatasetNameEntryBody10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterDatasetName83Records10_1 {
    pub first_long: i32,
    pub text: String,
}

/// 在宿主已建立的 ObjectStream 表中读取名称条目；调用者须整体暂存游标与键表。
pub(super) fn read_supported_raster_name_entry_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<RasterDatasetNameEntryBody10_1>)> {
    let slot = read_name_slot(reader, max_bytes, table, MAX_NAME_NESTING)?;
    Ok((slot.entry, slot.body))
}

pub(super) fn read_name_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RasterNameSlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered_class) = table.registered_class(*key)
    {
        *class_id = registered_class;
    }
    let body = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id:
                RASTER_DATASET_NAME_CLASS_ID
                | CACHE_RASTER_DATASET_NAME_CLASS_ID
                | WCS_COVERAGE_NAME_CLASS_ID
                | WMS_CONNECTION_NAME_CLASS_ID
                | WMTS_CONNECTION_NAME_CLASS_ID
                | RASTER_DATASET_NAME83_CLASS_ID
                | FGDB_RASTER_DATASET_NAME_CLASS_ID
                | RASTER_BAND_NAME_CLASS_ID
                | RASTER_BAND_COLLECTION_NAME_CLASS_ID
                | RASTER_CATALOG_NAME_CLASS_ID
                | MOSAIC_DATASET_NAME_CLASS_ID
                | ACCESS_RASTER_DATASET_NAME_CLASS_ID
                | SDE_RASTER_DATASET_NAME_CLASS_ID
                | NETCDF_RASTER_DATASET_NAME_CLASS_ID
                | IS_RASTER_DATASET_NAME_CLASS_ID
                | IMAGE_SERVER_NAME_CLASS_ID
                | FUNCTION_RASTER_DATASET_NAME_CLASS_ID
                | INTERNET_TILED_LAYER_NAME_CLASS_ID
                | AIS_IMAGE_SERVICE_NAME_CLASS_ID
                | RASTER_TYPE_NAME_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject { class_id, key } => Some(read_name_body(
            reader, max_bytes, table, class_id, key, depth,
        )?),
        ObjectEntry10_1::BackReference { class_id, .. } => {
            return Err(unsupported_raster_name_class(&class_id));
        }
    };
    if let ObjectEntry10_1::NewObject { class_id, key } = entry
        && let Some(body) = &body
    {
        if body.open_without_result() {
            table.mark_raster_name_without_open_result(key);
        }
        if let Some(name) = body.dataset_name_body().filter(|name| name.stop.is_none()) {
            table.mark_dataset_source(
                class_id,
                key,
                name.dataset_name().unwrap_or_default(),
                name.workspace_path(),
                None,
            );
        } else if let Some(name) = match body {
            RasterDatasetNameEntryBody10_1::WcsCoverage(name) => Some(name.identifier.as_str()),
            RasterDatasetNameEntryBody10_1::InternetTiledLayer(name) => {
                Some(name.name_string.as_str())
            }
            RasterDatasetNameEntryBody10_1::AisImageService(name) => Some(name.strings[0].as_str()),
            _ => None,
        } {
            table.mark_dataset_source(class_id, key, name, None, None);
        } else if let Some((name, workspace)) = match body {
            RasterDatasetNameEntryBody10_1::NetCdf(name) => {
                Some((name.variable(), &name.workspace))
            }
            RasterDatasetNameEntryBody10_1::Mosaic(name) => {
                Some((name.strings[0].as_str(), &name.workspace))
            }
            RasterDatasetNameEntryBody10_1::Cache(name) => {
                Some((name.strings[0].as_str(), &name.workspace))
            }
            _ => None,
        } {
            let path = match workspace {
                RasterDatasetWorkspaceSlot10_1::WorkspaceName(workspace) => {
                    Some(workspace.path.as_str())
                }
                RasterDatasetWorkspaceSlot10_1::BackReference { class_id, key }
                    if *class_id == WORKSPACE_NAME_CLASS_ID =>
                {
                    table.workspace_path(*key)
                }
                _ => None,
            };
            let path = path.map(str::to_owned);
            table.mark_dataset_source(class_id, key, name, path.as_deref(), None);
        } else if let RasterDatasetNameEntryBody10_1::Band(band) = body {
            let (name, path) = match &band.source {
                RasterBandNameSource10_1::Dataset(slot) => match slot.entry {
                    ObjectEntry10_1::NewObject { class_id, key }
                    | ObjectEntry10_1::BackReference { class_id, key } => table
                        .dataset_source(class_id, key)
                        .map(|source| (source.name.as_ref(), source.path.as_deref()))
                        .unwrap_or_default(),
                    ObjectEntry10_1::Null => ("", None),
                },
                RasterBandNameSource10_1::Workspace(workspace) => (
                    "",
                    match workspace {
                        RasterDatasetWorkspaceSlot10_1::WorkspaceName(body) => {
                            Some(body.path.as_str())
                        }
                        RasterDatasetWorkspaceSlot10_1::BackReference { class_id, key }
                            if *class_id == WORKSPACE_NAME_CLASS_ID =>
                        {
                            table.workspace_path(*key)
                        }
                        _ => None,
                    },
                ),
            };
            let name = name.to_owned();
            let path = path.map(str::to_owned);
            table.mark_dataset_source(
                class_id,
                key,
                &name,
                path.as_deref(),
                Some(band.band_lookup_name()),
            );
        } else if let RasterDatasetNameEntryBody10_1::BandCollection(collection) = body {
            let parent = match collection.name.entry {
                ObjectEntry10_1::NewObject { class_id, key }
                | ObjectEntry10_1::BackReference { class_id, key }
                    if matches!(
                        class_id,
                        RASTER_DATASET_NAME_CLASS_ID
                            | RASTER_DATASET_NAME83_CLASS_ID
                            | FGDB_RASTER_DATASET_NAME_CLASS_ID
                            | ACCESS_RASTER_DATASET_NAME_CLASS_ID
                            | SDE_RASTER_DATASET_NAME_CLASS_ID
                            | NETCDF_RASTER_DATASET_NAME_CLASS_ID
                    ) =>
                {
                    table.dataset_source(class_id, key).cloned()
                }
                _ => None,
            };
            table.mark_band_collection_source(
                class_id,
                key,
                parent,
                collection
                    .array
                    .as_ref()
                    .map(|(_, values)| values.as_slice()),
            );
        }
    }
    Ok(RasterNameSlot10_1 { entry, body })
}

fn unsupported_raster_name_class(class_id: &[u8; 16]) -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "未支持的栅格名称类 {}",
            crate::storage::class_id_hex(class_id)
        ),
    )
}

/// 读取已知 CLSID 的名称对象体。无键宿主传 0，嵌套对象仍使用宿主表。
pub(crate) fn read_supported_raster_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<RasterDatasetNameEntryBody10_1> {
    read_name_body(reader, max_bytes, table, class_id, key, MAX_NAME_NESTING)
}

pub(super) fn read_name_body(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
    depth: usize,
) -> io::Result<RasterDatasetNameEntryBody10_1> {
    if depth == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "栅格名称嵌套超过限额",
        ));
    }
    let incomplete = |reason: String| io::Error::new(io::ErrorKind::Unsupported, reason);
    match class_id {
        CACHE_RASTER_DATASET_NAME_CLASS_ID => Ok(RasterDatasetNameEntryBody10_1::Cache(Box::new(
            read_cache_raster_name(reader, max_bytes, table, depth - 1)?,
        ))),
        INTERNET_TILED_LAYER_NAME_CLASS_ID => {
            let version = reader.read_u16_le()? as i16;
            if version > 1 {
                return Err(incomplete(format!(
                    "InternetTiledLayerName 未支持版本 {version}"
                )));
            }
            let name_string = read_string_10_1(reader, max_bytes)?;
            let integer = reader.read_u32_le()? as i32;
            let tile_reader = slot(reader, table, |r, refs, class, _| {
                read_tile_reader(r, max_bytes, refs, class, depth - 1)
            })?;
            Ok(RasterDatasetNameEntryBody10_1::InternetTiledLayer(
                Box::new(InternetTiledLayerName10_1 {
                    version,
                    name_string,
                    integer,
                    tile_reader,
                }),
            ))
        }
        AIS_IMAGE_SERVICE_NAME_CLASS_ID => {
            let version = reader.read_u32_le()? as i32;
            if version > 1 {
                return Err(incomplete(format!(
                    "AISImageServiceName 未支持版本 {version}"
                )));
            }
            Ok(RasterDatasetNameEntryBody10_1::AisImageService(
                AisImageServiceName10_1 {
                    version,
                    strings: [
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                        read_string_10_1(reader, max_bytes)?,
                    ],
                },
            ))
        }
        WCS_COVERAGE_NAME_CLASS_ID => Ok(RasterDatasetNameEntryBody10_1::WcsCoverage(Box::new(
            read_wcs_coverage_name(reader, max_bytes, table)?,
        ))),
        WCS_CONNECTION_NAME_CLASS_ID
        | WMS_CONNECTION_NAME_CLASS_ID
        | WMTS_CONNECTION_NAME_CLASS_ID => {
            Ok(RasterDatasetNameEntryBody10_1::OgcConnection(Box::new(
                read_ogc_connection_name(reader, max_bytes, table, class_id)?,
            )))
        }
        RASTER_TYPE_NAME_CLASS_ID => {
            let version = reader.read_u16_le()?;
            if version != 1 {
                return Err(incomplete(format!("RasterTypeName 未支持版本 {version}")));
            }
            let name_string = read_string_10_1(reader, max_bytes)?;
            let name = read_string_10_1(reader, max_bytes)?;
            let instance_id = reader.read_u32_le()? as i32;
            let mosaic_dataset = slot(reader, table, |r, refs, class, _| {
                if class != MOSAIC_DATASET_NAME_CLASS_ID {
                    return Err(unsupported_raster_name_class(&class));
                }
                read_mosaic_dataset_name(r, max_bytes, refs)
            })?;
            if let ObjectEntry10_1::BackReference { class_id, .. } = mosaic_dataset.entry
                && class_id != MOSAIC_DATASET_NAME_CLASS_ID
            {
                return Err(unsupported_raster_name_class(&class_id));
            }
            Ok(RasterDatasetNameEntryBody10_1::RasterType(Box::new(
                RasterTypeName10_1 {
                    version,
                    name_string,
                    name,
                    instance_id,
                    mosaic_dataset,
                },
            )))
        }
        RASTER_DATASET_NAME_CLASS_ID | RASTER_DATASET_NAME83_CLASS_ID => {
            let name = read_raster_dataset_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = name.stop {
                return Err(incomplete(format!("RasterDatasetName 未读完：{stop:?}")));
            }
            let name = Box::new(name);
            if class_id == RASTER_DATASET_NAME_CLASS_ID {
                return Ok(RasterDatasetNameEntryBody10_1::Dataset(name));
            }
            // RasterDatasetName83::Load 的 InitCheck 在无 magic 时恢复流位置。
            let has_records =
                reader.remaining() >= 4 && reader.clone().read_u32_le()? == 0x00ff_00fa;
            let records = has_records
                .then(|| -> io::Result<_> {
                    raster_io_record(reader, 0x00ff_00fa, 0)?;
                    let first_long =
                        i32::from_le_bytes(raster_io_record(reader, 1, 4)?.try_into().unwrap());
                    if reader.read_u32_le()? != 2 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "RasterDatasetName83 缺少 RasterIO tag 2 字符串",
                        ));
                    }
                    let text = read_string_10_1(reader, max_bytes)?;
                    read_raster_io_end(reader)?;
                    Ok(RasterDatasetName83Records10_1 { first_long, text })
                })
                .transpose()?;
            Ok(RasterDatasetNameEntryBody10_1::Dataset83 { name, records })
        }
        FGDB_RASTER_DATASET_NAME_CLASS_ID => {
            let name = read_fgdb_raster_dataset_name_prefix_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = name.delegated_name.stop {
                return Err(incomplete(format!("FileGDB 委派名称未读完：{stop:?}")));
            }
            let records = read_fgdb_raster_io_prefix_10_1(reader, max_bytes)?;
            if let Some(stop) = records.stop {
                return Err(incomplete(format!("FileGDB 名称记录未读完：{stop:?}")));
            }
            Ok(RasterDatasetNameEntryBody10_1::FileGeodatabase {
                name: Box::new(name),
                records: Box::new(records),
            })
        }
        RASTER_BAND_NAME_CLASS_ID => Ok(RasterDatasetNameEntryBody10_1::Band(Box::new(
            read_band_name(reader, max_bytes, table, depth - 1)?,
        ))),
        RASTER_BAND_COLLECTION_NAME_CLASS_ID => {
            Ok(RasterDatasetNameEntryBody10_1::BandCollection(Box::new(
                read_band_collection_name(reader, max_bytes, table, depth - 1)?,
            )))
        }
        RASTER_CATALOG_NAME_CLASS_ID => {
            // 构造器内建 FeatureClassName，直接委派同一流的 Load。
            let name = read_feature_class_name_payload(reader, max_bytes, table, key, 0)?;
            if let Some(stop) = &name.stop {
                return Err(incomplete(format!(
                    "RasterCatalogName 委派体未读完：{stop:?}"
                )));
            }
            Ok(RasterDatasetNameEntryBody10_1::Catalog(Box::new(name)))
        }
        MOSAIC_DATASET_NAME_CLASS_ID => Ok(RasterDatasetNameEntryBody10_1::Mosaic(Box::new(
            read_mosaic_dataset_name(reader, max_bytes, table)?,
        ))),
        NETCDF_RASTER_DATASET_NAME_CLASS_ID => Ok(RasterDatasetNameEntryBody10_1::NetCdf(
            Box::new(read_netcdf_dataset_name(reader, max_bytes, table)?),
        )),
        IS_RASTER_DATASET_NAME_CLASS_ID => {
            Ok(RasterDatasetNameEntryBody10_1::ImageService(Box::new(
                read_is_raster_dataset_name(reader, max_bytes, table, depth - 1)?,
            )))
        }
        IMAGE_SERVER_NAME_CLASS_ID => Ok(RasterDatasetNameEntryBody10_1::ImageServerReference(
            Box::new(read_image_server_name(reader, max_bytes, table)?),
        )),
        FUNCTION_RASTER_DATASET_NAME_CLASS_ID => {
            Ok(RasterDatasetNameEntryBody10_1::Function(Box::new(
                read_function_raster_dataset_name(reader, max_bytes, table, depth - 1)?,
            )))
        }
        ACCESS_RASTER_DATASET_NAME_CLASS_ID | SDE_RASTER_DATASET_NAME_CLASS_ID => {
            let name = read_raster_dataset_name_body_10_1(reader, max_bytes, table, key)?;
            if let Some(stop) = name.stop {
                return Err(incomplete(format!("数据库栅格委派名称未读完：{stop:?}")));
            }
            let records = if reader.remaining() >= 4 && reader.clone().read_u32_le()? == 0x00ff_00fa
            {
                Some(read_database_raster_records(
                    reader,
                    max_bytes,
                    table,
                    class_id,
                    depth - 1,
                )?)
            } else {
                None
            };
            Ok(RasterDatasetNameEntryBody10_1::Database(Box::new(
                DatabaseRasterDatasetName10_1 {
                    class_id,
                    delegated_name: name,
                    records,
                },
            )))
        }
        _ => Err(unsupported_raster_name_class(&class_id)),
    }
}

fn read_function_raster_dataset_name(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<FunctionRasterDatasetName10_1> {
    let version = reader.read_u16_le()? as i16;
    if version > 7 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 FunctionRasterDatasetName 版本 {version}"),
        ));
    }
    let strings = [
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
    ];
    let workspace = read_complete_workspace_slot(reader, max, table, "FunctionRasterDatasetName")?;
    let legacy_string = (version < 4)
        .then(|| read_string_10_1(reader, max))
        .transpose()?;
    let path = read_string_10_1(reader, max)?;
    let legacy = legacy_string
        .map(|text| -> io::Result<_> {
            Ok(FunctionRasterLegacyName10_1 {
                text,
                long: reader.read_u32_le()? as i32,
                name: read_name_slot(reader, max, table, child_depth)?,
            })
        })
        .transpose()?;
    let mut function = slot(reader, table, |r, refs, class_id, _| {
        read_function(r, max, refs, class_id, child_depth)
    })?;
    check_rule_reference(&mut function.entry, table, "IRasterFunction", |id| {
        function_max_version(id).is_some()
    })?;
    let input_count = (version >= 2)
        .then(|| reader.read_u32_le().map(|n| n as i32))
        .transpose()?;
    // 旧版本和非正计数均读取一个 IUnknown；正计数逐项读取。
    let count = input_count.unwrap_or(0).max(1) as usize;
    if count > reader.remaining().min(max) / 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "函数栅格名称输入数量超限",
        ));
    }
    let inputs = (0..count)
        .map(|_| {
            slot(reader, table, |r, refs, class_id, key| {
                Ok(
                    match read_input_body(r, max, refs, class_id, key, child_depth)? {
                        RasterTemplateInput10_1::Arguments(body) => {
                            FunctionRasterInput10_1::Arguments(body)
                        }
                        RasterTemplateInput10_1::RasterName(body) => {
                            FunctionRasterInput10_1::Name(body)
                        }
                        other => FunctionRasterInput10_1::Other(Box::new(other)),
                    },
                )
            })
        })
        .collect::<io::Result<_>>()?;
    let properties = (version >= 3)
        .then(|| {
            slot(reader, table, |r, refs, class_id, _| {
                if class_id != PROPERTY_SET_CLASS_ID {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "函数栅格名称未支持的 IPropertySet 类",
                    ));
                }
                read_property_set_object_stream_10_1(r, max, refs)
            })
        })
        .transpose()?;
    let geometry = (3..=4)
        .contains(&version)
        .then(|| {
            slot(reader, table, |r, refs, class_id, key| {
                read_keyed_geometry_body_10_1(&class_id, key, r, max, refs)
            })
        })
        .transpose()?;
    Ok(FunctionRasterDatasetName10_1 {
        version: version as u16,
        strings,
        workspace,
        path,
        legacy,
        function,
        input_count,
        inputs,
        properties,
        geometry,
        string_40: (version >= 6)
            .then(|| read_string_10_1(reader, max))
            .transpose()?,
        string_48: (version >= 7)
            .then(|| read_string_10_1(reader, max))
            .transpose()?,
    })
}

fn read_is_raster_dataset_name(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<IsRasterDatasetName10_1> {
    let version = reader.read_u16_le()? as i16;
    if version > 4 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 ISRasterDatasetName 版本 {version}"),
        ));
    }
    let mut strings: [String; 6] = Default::default();
    for string in &mut strings {
        *string = read_string_10_1(reader, max)?;
    }
    Ok(IsRasterDatasetName10_1 {
        version: version as u16,
        strings,
        long_44: reader.read_u32_le()? as i32,
        workspace: read_complete_workspace_slot(reader, max, table, "ISRasterDatasetName")?,
        name: read_name_slot(reader, max, table, child_depth)?,
        string_50: (version >= 2)
            .then(|| read_string_10_1(reader, max))
            .transpose()?,
        string_54: (version >= 3)
            .then(|| read_string_10_1(reader, max))
            .transpose()?,
        transform: (version >= 4)
            .then(|| read_geodata_xform_slot_with_depth(reader, max, table, child_depth))
            .transpose()?,
    })
}

fn read_netcdf_dataset_name(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<NetCdfRasterDatasetName10_1> {
    let version = reader.read_u32_le()? as i32;
    if version > 2 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 NetCDFRasterDatasetName 版本 {version}"),
        ));
    }
    let mut strings: [String; 8] = Default::default();
    for string in &mut strings {
        *string = read_string_10_1(reader, max)?;
    }
    let legacy = if version == 1 {
        Some((
            read_string_10_1(reader, max)?,
            read_envelope_slot(reader, max, table)?,
        ))
    } else {
        None
    };
    let labels = read_string_array_slot_10_1(reader, max, table)?;
    Ok(NetCdfRasterDatasetName10_1 {
        version,
        strings,
        legacy,
        labels,
        values: read_long_array_slot(reader, max, table)?,
        workspace: read_complete_workspace_slot(reader, max, table, "NetCDFRasterDatasetName")?,
    })
}

fn read_database_raster_records(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    class: [u8; 16],
    depth: usize,
) -> io::Result<DatabaseRasterDatasetRecords10_1> {
    use super::fgdb_raster_dataset_name::{
        read_raster_io_i32_10_1, read_raster_io_record_header_10_1, read_raster_io_string_10_1,
    };
    use crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID;

    raster_io_record(reader, 0x00ff_00fa, 0)?;
    let first = read_raster_io_i32_10_1(reader, 1)?;
    let second = read_raster_io_string_10_1(reader, 2, max)?;
    let third = read_raster_io_string_10_1(reader, 3, max)?;
    let fourth = read_raster_io_i32_10_1(reader, 4)?;
    let fifth = read_raster_io_string_10_1(reader, 5, max)?;
    if read_raster_io_record_header_10_1(reader)? != (6, -1) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "数据库栅格名称的 RasterIO tag 6 记录头不匹配",
        ));
    }

    // RasterIO 内嵌外壳只有 CLSID，子对象沿用宿主 ObjectStream。
    let class_id: [u8; 16] = reader.read_bytes(16)?.try_into().unwrap();
    let tag6_name = match class_id {
        id if id == [0; 16] => None,
        FEATURE_CLASS_NAME_CLASS_ID => {
            let name = read_feature_class_name_payload(reader, max, table, 0, 0)?;
            if let Some(stop) = &name.stop {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("RasterIO tag 6 FeatureClassName 未读完：{stop:?}"),
                ));
            }
            Some(DatabaseRasterTag6Name10_1::FeatureClass(Box::new(name)))
        }
        _ => Some(DatabaseRasterTag6Name10_1::Raster {
            class_id,
            body: read_name_body(reader, max, table, class_id, 0, depth)?,
        }),
    };
    let tag7_long = (class == ACCESS_RASTER_DATASET_NAME_CLASS_ID
        && reader.clone().read_u32_le()? == 7)
        .then(|| read_raster_io_i32_10_1(reader, 7))
        .transpose()?;
    read_raster_io_end(reader)?;
    Ok(DatabaseRasterDatasetRecords10_1 {
        longs_1_4: [first, fourth],
        strings_2_3_5: [second, third, fifth],
        tag6_name,
        tag7_long,
    })
}

fn read_band_name(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<RasterBandName10_1> {
    // 旧布局没有版本字。原生回退 4 字节，再读第一条 String。
    let has_version_marker = reader.clone().read_u32_le()? == 1;
    if has_version_marker {
        reader.read_u32_le()?;
    }
    let strings = [
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
    ];
    let selector = reader.read_u16_le()?;
    let source = if selector == 0 {
        RasterBandNameSource10_1::Workspace(read_complete_workspace_slot(
            reader,
            max,
            table,
            "RasterBandName",
        )?)
    } else {
        let dataset = read_name_slot(reader, max, table, child_depth)?;
        if dataset.entry == ObjectEntry10_1::Null {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "RasterBandName 的 IDatasetName 为空",
            ));
        }
        RasterBandNameSource10_1::Dataset(dataset)
    };
    Ok(RasterBandName10_1 {
        has_version_marker,
        strings,
        selector,
        source,
    })
}

fn read_cache_raster_name(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<CacheRasterDatasetName10_1> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let version = trial.read_u16_le()?;
    let strings = [
        read_string_10_1(&mut trial, max)?,
        read_string_10_1(&mut trial, max)?,
    ];
    let workspace =
        read_complete_workspace_slot(&mut trial, max, &mut staged, "CacheRasterDatasetName")?;
    let tile_reader = slot(&mut trial, &mut staged, |r, refs, class, _| {
        read_tile_reader(r, max, refs, class, child_depth)
    })?;
    let envelope = read_envelope_slot(&mut trial, max, &mut staged)?;
    *reader = trial;
    *refs = staged;
    Ok(CacheRasterDatasetName10_1 {
        version,
        strings,
        workspace,
        tile_reader: tile_reader.entry,
        tile_reader_body: tile_reader.body,
        envelope,
    })
}

fn read_complete_workspace_slot(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    label: &str,
) -> io::Result<RasterDatasetWorkspaceSlot10_1> {
    let workspace = read_workspace_slot(reader, max, table)?;
    match &workspace {
        RasterDatasetWorkspaceSlot10_1::Unsupported { class_id, .. } => {
            Err(unsupported_raster_name_class(class_id))
        }
        RasterDatasetWorkspaceSlot10_1::WorkspaceName(body) if body.stop.is_some() => {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{label} 的 WorkspaceName 体未读完"),
            ))
        }
        _ => Ok(workspace),
    }
}

fn read_mosaic_dataset_name(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<MosaicDatasetName10_1> {
    let version = reader.read_u16_le()? as i16;
    if version > 4 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 MosaicDatasetName 版本 {version}"),
        ));
    }
    Ok(MosaicDatasetName10_1 {
        version: version as u16,
        strings: [
            read_string_10_1(reader, max)?,
            read_string_10_1(reader, max)?,
        ],
        workspace: read_complete_workspace_slot(reader, max, table, "MosaicDatasetName")?,
        short_44: (version >= 2).then(|| reader.read_u16_le()).transpose()?,
        long_48: (version >= 3)
            .then(|| reader.read_u32_le().map(|n| n as i32))
            .transpose()?,
        string_array: (version >= 4)
            .then(|| read_string_array_slot_10_1(reader, max, table))
            .transpose()?,
    })
}

fn read_band_collection_name(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<RasterBandCollectionName10_1> {
    let version = reader.read_u16_le()?;
    if (version as i16) < 0 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 RasterBandCollectionName 版本 {version}"),
        ));
    }
    let text = read_string_10_1(reader, max)?;
    let name = read_name_slot(reader, max, table, child_depth)?;
    let RasterLongArraySlot10_1 {
        entry: array_entry,
        body: array,
    } = read_long_array_slot(reader, max, table)?;
    Ok(RasterBandCollectionName10_1 {
        version,
        text,
        name,
        array_entry,
        array,
    })
}

pub(crate) fn read_long_array_slot(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLongArraySlot10_1> {
    let mut entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = table.registered_class(*key)
    {
        if registered != LONG_ARRAY_CLASS_ID {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("ILongArray 回引键 {key} 的登记类不受支持"),
            ));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::BackReference {
            class_id: LONG_ARRAY_CLASS_ID,
            key,
        } => table
            .long_array(key)
            .filter(|(_, values)| values.len() <= max / 4)
            .map(|(version, values)| (version, values.to_vec())),
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: LONG_ARRAY_CLASS_ID,
            key,
        } => {
            let body = read_long_array_body(reader, max)?;
            table.mark_long_array(key, body.0, &body.1);
            Some(body)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "ILongArray 未支持的新对象类 {}",
                    crate::storage::class_id_hex(&class_id),
                ),
            ));
        }
    };
    Ok(RasterLongArraySlot10_1 { entry, body })
}

pub(crate) fn read_long_array_body(
    reader: &mut Reader<'_>,
    max: usize,
) -> io::Result<(u16, Vec<i32>)> {
    let version = reader.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 LongArray 版本 {version}"),
        ));
    }
    let count = (reader.read_u32_le()? as i32).max(0) as usize;
    if count > max / 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "LongArray 超过长度限额",
        ));
    }
    if count > reader.remaining() / 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "LongArray 超出剩余范围",
        ));
    }
    let values = (0..count)
        .map(|_| reader.read_u32_le().map(|value| value as i32))
        .collect::<io::Result<_>>()?;
    Ok((version, values))
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

    fn body_with_null_workspace(gate: i32) -> Vec<u8> {
        let mut bytes = gate.to_le_bytes().to_vec();
        for value in ["one", "two", "three", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 16]);
        if gate >= 3 {
            bytes.extend(0x1122_3344_i32.to_le_bytes());
        }
        if gate >= 4 {
            bytes.extend(0x5566_7788_i32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn system_core_arrays_use_signed_version_gate() {
        let mut longs = vec![0xff, 0xff, 1, 0, 0, 0];
        longs.extend_from_slice(&42i32.to_le_bytes());
        longs.push(0xaa);
        let mut reader = Reader::new(&longs);
        assert_eq!(
            read_long_array_body(&mut reader, 1024).unwrap(),
            (0xffff, vec![42])
        );
        assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);

        let strings = [0, 0x80, 0, 0, 0, 0, 0xaa];
        let mut reader = Reader::new(&strings);
        use crate::objects::string_array::read_string_array_body_10_1;
        assert_eq!(
            read_string_array_body_10_1(&mut reader, 1024).unwrap(),
            (0x8000, vec![])
        );
        assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);

        assert_eq!(
            read_long_array_body(&mut Reader::new(&[2, 0]), 1024)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(
            read_string_array_body_10_1(&mut Reader::new(&[2, 0]), 1024)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn band_collection_back_reference_retains_source_and_indices() {
        for parent_class in [
            RASTER_DATASET_NAME_CLASS_ID,
            NETCDF_RASTER_DATASET_NAME_CLASS_ID,
        ] {
            let mut refs = RefTable10_1::new();
            refs.register(21);
            refs.mark_dataset_source(parent_class, 21, "raster", Some("data.gdb"), None);

            let mut bytes = RASTER_BAND_COLLECTION_NAME_CLASS_ID.to_vec();
            bytes.extend(22_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded_string("集合"));
            bytes.extend(parent_class);
            bytes.extend(21_u32.to_le_bytes());
            bytes.extend(LONG_ARRAY_CLASS_ID);
            bytes.extend(23_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            for index in [2_i32, 0, 2] {
                bytes.extend(index.to_le_bytes());
            }
            let first_end = bytes.len();
            bytes.extend(RASTER_BAND_COLLECTION_NAME_CLASS_ID);
            bytes.extend(22_u32.to_le_bytes());

            let mut reader = Reader::new(&bytes);
            read_name_slot(&mut reader, 1024, &mut refs, 16).unwrap();
            assert_eq!(reader.position(), first_end);
            let reference = read_name_slot(&mut reader, 1024, &mut refs, 16).unwrap();
            assert!(matches!(
                reference.entry,
                ObjectEntry10_1::BackReference { key: 22, .. }
            ));
            let source = refs
                .dataset_source(RASTER_BAND_COLLECTION_NAME_CLASS_ID, 22)
                .unwrap();
            assert_eq!(source.name.as_ref(), "raster");
            assert_eq!(source.path.as_deref(), Some("data.gdb"));
            assert_eq!(source.band_indices.as_deref(), Some(&[2, 0, 2][..]));
            assert_eq!(reader.position(), bytes.len());
        }
    }

    #[test]
    fn band_collection_resolves_ordered_long_array_back_reference() {
        let mut definition = LONG_ARRAY_CLASS_ID.to_vec();
        definition.extend(27_u32.to_le_bytes());
        definition.extend(1_u16.to_le_bytes());
        definition.extend(3_i32.to_le_bytes());
        for index in [2_i32, 0, 2] {
            definition.extend(index.to_le_bytes());
        }
        let mut refs = RefTable10_1::new();
        read_long_array_slot(&mut Reader::new(&definition), 1024, &mut refs).unwrap();

        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(encoded_string("组合"));
        bytes.extend([0; 16]);
        let array_offset = bytes.len();
        bytes.extend(LONG_ARRAY_CLASS_ID);
        bytes.extend(27_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let body = read_band_collection_name(&mut reader, 1024, &mut refs, 16).unwrap();
        assert!(matches!(
            body.array_entry,
            ObjectEntry10_1::BackReference { key: 27, .. }
        ));
        assert_eq!(body.array, Some((1, vec![2, 0, 2])));
        assert_eq!(reader.position(), bytes.len());

        for version in [0_u16, 2, i16::MAX as u16] {
            bytes[..2].copy_from_slice(&version.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let body = read_band_collection_name(&mut reader, 1024, &mut refs, 16).unwrap();
            assert_eq!(body.version, version);
            assert_eq!(body.array, Some((1, vec![2, 0, 2])));
            assert_eq!(reader.remaining(), 0);
        }

        bytes[..2].copy_from_slice(&(-1_i16).to_le_bytes());
        let err =
            read_band_collection_name(&mut Reader::new(&bytes), 1024, &mut refs, 16).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Unsupported);

        bytes[..2].copy_from_slice(&1_u16.to_le_bytes());
        bytes[array_offset..array_offset + 16].fill(0x42);
        let body =
            read_band_collection_name(&mut Reader::new(&bytes), 1024, &mut refs, 16).unwrap();
        assert_eq!(body.array, Some((1, vec![2, 0, 2])));
    }

    #[test]
    fn band_name_back_reference_resolves_its_parent_dataset_fields() {
        let mut refs = RefTable10_1::new();
        refs.register(12);
        refs.mark_dataset_source(
            RASTER_DATASET_NAME_CLASS_ID,
            12,
            "image",
            Some("rasters.gdb"),
            None,
        );

        let mut bytes = RASTER_BAND_NAME_CLASS_ID.to_vec();
        bytes.extend(13_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        for text in ["image\\Band_2", "", ""] {
            bytes.extend(encoded_string(text));
        }
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(12_u32.to_le_bytes());
        let first_end = bytes.len();
        bytes.extend(RASTER_BAND_NAME_CLASS_ID);
        bytes.extend(13_u32.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        read_name_slot(&mut reader, 1024, &mut refs, 16).unwrap();
        assert_eq!(reader.position(), first_end);
        let slot = read_name_slot(&mut reader, 1024, &mut refs, 16).unwrap();
        assert!(matches!(
            slot.entry,
            ObjectEntry10_1::BackReference { key: 13, .. }
        ));
        let source = refs.dataset_source(RASTER_BAND_NAME_CLASS_ID, 13).unwrap();
        assert_eq!(source.name.as_ref(), "image");
        assert_eq!(source.path.as_deref(), Some("rasters.gdb"));
        assert_eq!(source.band_name.as_deref(), Some("Band_2"));
        assert!(
            refs.dataset_source(RASTER_DATASET_NAME_CLASS_ID, 13)
                .is_none()
        );
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn workspace_back_reference_requires_the_matching_class() {
        let mut bytes = body_with_null_workspace(4);
        let offset = bytes.len() - 24;
        bytes.splice(
            offset..offset + 16,
            WORKSPACE_NAME_CLASS_ID
                .into_iter()
                .chain(12_u32.to_le_bytes()),
        );
        let mut refs = RefTable10_1::new();
        refs.register(12);
        refs.mark_workspace_path(12, "raster.gdb");

        for (class, expected) in [
            (WORKSPACE_NAME_CLASS_ID, Some("raster.gdb")),
            ([0x42; 16], None),
        ] {
            bytes[offset..offset + 16].copy_from_slice(&class);
            let mut reader = Reader::new(&bytes);
            let body = read_raster_dataset_name_body_10_1(&mut reader, 1024, &mut refs, 9).unwrap();
            assert_eq!(body.workspace_path(), expected);
            assert_eq!(reader.position(), bytes.len());
        }
    }

    #[test]
    fn workspace_back_reference_uses_the_registered_class() {
        for registered in [WORKSPACE_NAME_CLASS_ID, [0x42; 16]] {
            let mut refs = RefTable10_1::new();
            let mut header = registered.to_vec();
            header.extend(12_u32.to_le_bytes());
            read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
            refs.mark_workspace_path(12, "raster.gdb");
            let before = refs.clone();

            let mut bytes = body_with_null_workspace(4);
            let offset = bytes.len() - 24;
            let claimed = if registered == WORKSPACE_NAME_CLASS_ID {
                [0x42; 16]
            } else {
                WORKSPACE_NAME_CLASS_ID
            };
            bytes.splice(
                offset..offset + 16,
                claimed.into_iter().chain(12_u32.to_le_bytes()),
            );
            let mut reader = Reader::new(&bytes);
            let body = read_raster_dataset_name_body_10_1(&mut reader, 1024, &mut refs, 9).unwrap();
            assert_eq!(refs, before);
            if registered == WORKSPACE_NAME_CLASS_ID {
                assert_eq!(body.workspace_path(), Some("raster.gdb"));
                assert_eq!(reader.position(), bytes.len());
            } else {
                assert!(
                    matches!(body.stop, Some(RasterDatasetNameStop10_1::UnsupportedWorkspaceClass {
                    class_id, key: 12,
                }) if class_id == registered)
                );
                assert_eq!(reader.position(), offset);
            }
        }
    }

    #[test]
    fn parses_modern_gate_strings_null_slot_and_versioned_tail() {
        let bytes = body_with_null_workspace(4);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();

        let parsed =
            read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut table, 1545).unwrap();

        assert_eq!(parsed.version_gate, 4);
        assert_eq!(
            parsed.strings.as_ref().unwrap(),
            &["one", "two", "three", ""]
        );
        assert!(matches!(
            parsed.workspace,
            Some(RasterDatasetWorkspaceSlot10_1::Null)
        ));
        assert_eq!(parsed.tail_at_6c, Some(0x1122_3344));
        assert_eq!(parsed.tail_at_70, Some(0x5566_7788));
        assert_eq!(reader.position(), bytes.len());
        assert!(table.is_empty());
    }

    #[test]
    fn gate_three_reads_only_the_first_trailing_integer() {
        let mut bytes = body_with_null_workspace(3);
        bytes.extend([0xaa; 4]);
        let expected_end = bytes.len() - 4;
        let mut reader = Reader::new(&bytes);

        let parsed =
            read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut RefTable10_1::new(), 9)
                .unwrap();

        assert_eq!(parsed.tail_at_6c, Some(0x1122_3344));
        assert_eq!(parsed.tail_at_70, None);
        assert_eq!(reader.position(), expected_end);
    }

    #[test]
    fn zero_gate_rejects_zero_length_first_string_and_rolls_back() {
        let bytes = body_with_null_workspace(0);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();

        let error =
            read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut table, 9).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn truncation_rolls_back_cursor_and_reference_table() {
        let bytes = body_with_null_workspace(4);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            assert!(
                read_raster_dataset_name_body_10_1(&mut reader, 0x1000, &mut table, 9).is_err()
            );
            assert_eq!(reader.position(), 0, "截断位置 {end} 应回滚");
            assert!(table.is_empty());
        }
    }
}
