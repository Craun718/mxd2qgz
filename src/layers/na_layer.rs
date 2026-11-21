//! 已确认的 NetworkAnalyst.dll 10.1 NALayer 持久化前缀。

use super::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use crate::maps::standalone_table::{
    STANDALONE_TABLE_CLASS_ID, StandaloneTablePrefix10_1, StandaloneTableStop10_1,
    read_standalone_table_prefix_10_1,
};
use crate::objects::dataset_name::{
    FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1, read_feature_dataset_name_body_10_1,
};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, Envelope10_1, SpatialReferenceEntry10_1, read_envelope_10_1,
    read_spatial_reference_entry_10_1,
};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, PropertyValueObjectStream10_1,
    read_padded_variant_object_stream_10_1, read_property_set_object_stream_10_1,
};
use crate::objects::string_array::{
    StringArraySlot10_1, read_string_array_body_10_1, read_string_array_slot_10_1,
};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{
    native_string::{read_bstr_string_10_1, read_string_10_1},
    reader::Reader,
};
use std::io;

pub(crate) const NA_LAYER_CLASS_ID: [u8; 16] = [
    0x71, 0x85, 0xbd, 0x5f, 0x60, 0xa5, 0x4f, 0x49, 0x9f, 0xaf, 0x3e, 0xdc, 0xed, 0x5a, 0x78, 0x90,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NaLayerChildren10_1 {
    pub declared_count: i32,
    pub entries: Vec<(
        ObjectEntry10_1,
        Option<Box<super::feature_layer::FeatureLayerPrefix10_1>>,
    )>,
    pub stop: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLayerBitSet10_1 {
    pub header: [u32; 4],
    pub has_words: u8,
    pub words: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLayerTables10_1 {
    pub declared_count: i32,
    pub entries: Vec<(ObjectEntry10_1, Option<Box<StandaloneTablePrefix10_1>>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLayerTail10_1 {
    pub first_flag: u16,
    pub object: ObjectEntry10_1,
    pub envelope: Option<Box<Envelope10_1>>,
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub scalar_bits: [u64; 3],
    pub name: String,
    pub second_flag: u16,
    pub tables: Option<NaLayerTables10_1>,
    pub version_two_flag: Option<u16>,
    pub bitsets: Option<[NaLayerBitSet10_1; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLayerPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub shorts: [u16; 3],
    pub double_bits: [u64; 2],
    pub network_name: Option<(ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>)>,
    pub context: Option<(ObjectEntry10_1, Option<Box<NaContextPrefix10_1>>)>,
    pub children: Option<NaLayerChildren10_1>,
    pub tail: Option<NaLayerTail10_1>,
}

pub(super) fn read_na_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<NaLayerPrefix10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    if version as i16 > 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NALayer 版本超过 3",
        ));
    }
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let shorts = [
        trial.read_u16_le()?,
        trial.read_u16_le()?,
        trial.read_u16_le()?,
    ];
    let double_bits = [
        trial.read_f64_le()?.to_bits(),
        trial.read_f64_le()?.to_bits(),
    ];
    *reader = trial;
    Ok(NaLayerPrefix10_1 {
        version,
        name,
        shorts,
        double_bits,
        network_name: None,
        context: None,
        children: None,
        tail: None,
    })
}

/// NALayer::Load 0x7af1ffc0 在完整上下文之后读取 LONG 和普通 ILayer 槽。
pub(super) fn read_na_layer_children(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NaLayerChildren10_1> {
    let mut trial = reader.clone();
    let declared_count = trial.read_u32_le()? as i32;
    let count = declared_count.max(0) as usize;
    if count > trial.remaining().min(max) / 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NALayer 子图层数量超出可读范围",
        ));
    }
    *reader = trial;
    let mut entries = Vec::new();
    let mut stop = None;
    for _ in 0..count {
        let mut trial = reader.clone();
        let mut keys = refs.clone();
        match read_na_child_layer(&mut trial, max, &mut keys) {
            Ok(child) => {
                *reader = trial;
                *refs = keys;
                entries.push(child);
            }
            Err(error) => {
                stop = Some(error.to_string());
                break;
            }
        }
    }
    Ok(NaLayerChildren10_1 {
        declared_count,
        entries,
        stop,
    })
}

fn read_na_child_layer(
    reader: &mut Reader<'_>,
    max: usize,
    keys: &mut RefTable10_1,
) -> io::Result<(
    ObjectEntry10_1,
    Option<Box<super::feature_layer::FeatureLayerPrefix10_1>>,
)> {
    use super::feature_layer::{
        FEATURE_LAYER_CLASS_ID, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
    };
    use super::feature_layer_tail::SuccessorStop10_1;

    let entry = read_object_entry_10_1(reader, keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: FEATURE_LAYER_CLASS_ID,
            ..
        } => {
            let body = read_feature_layer_prefix_10_1(reader, max, keys)?;
            if body.stop != FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("NALayer 子图层 {:?} 读取至 {:?}", body.name, body.stop),
                ));
            }
            Some(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("NALayer 子图层类 {class_id:02x?} 未支持"),
            ));
        }
        _ => None,
    };
    Ok((entry, body))
}

/// 读取 NALayer::Load 首组子图层之后的字段及版本段。
pub(super) fn read_na_layer_tail(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    version: u16,
) -> io::Result<NaLayerTail10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let unsupported = |message| io::Error::new(io::ErrorKind::Unsupported, message);
    let first_flag = trial.read_u16_le()?;
    let object = read_object_entry_10_1(&mut trial, &mut keys)?;
    let envelope = match object {
        ObjectEntry10_1::NewObject {
            class_id: ENVELOPE_CLASS_ID,
            ..
        } => Some(Box::new(read_envelope_10_1(&mut trial, max, &mut keys)?)),
        ObjectEntry10_1::NewObject { .. } => {
            return Err(unsupported("NALayer 尾部 IEnvelope 对象类未支持"));
        }
        _ => None,
    };
    let scalar = trial.read_f64_le()?.to_bits();
    let extensions = read_layer_extensions_10_1(&mut trial, max, &mut keys)?;
    let name = read_string_10_1(&mut trial, max)?;
    let second_flag = trial.read_u16_le()?;
    let scalar_bits = [
        scalar,
        trial.read_f64_le()?.to_bits(),
        trial.read_f64_le()?.to_bits(),
    ];
    let version = version as i16;
    let tables = (version >= 1)
        .then(|| read_na_layer_tables(&mut trial, max, &mut keys))
        .transpose()?;
    let version_two_flag = (version >= 2).then(|| trial.read_u16_le()).transpose()?;
    let bitsets = (version >= 3)
        .then(|| {
            Ok::<_, io::Error>([
                read_na_layer_bitset(&mut trial, max)?,
                read_na_layer_bitset(&mut trial, max)?,
            ])
        })
        .transpose()?;
    *reader = trial;
    *refs = keys;
    Ok(NaLayerTail10_1 {
        first_flag,
        object,
        envelope,
        extensions,
        scalar_bits,
        name,
        second_flag,
        tables,
        version_two_flag,
        bitsets,
    })
}

/// NALayer::Load 的第二组对象槽请求 IStandaloneTable。
fn read_na_layer_tables(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NaLayerTables10_1> {
    let declared_count = reader.read_u32_le()? as i32;
    let count = declared_count.max(0) as usize;
    if count > reader.remaining().min(max) / 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NALayer 独立表数量超出可读范围",
        ));
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        let offset = reader.position();
        let entry = read_object_entry_10_1(reader, refs)?;
        let body = match entry {
            ObjectEntry10_1::NewObject {
                class_id: STANDALONE_TABLE_CLASS_ID,
                key,
            } => {
                let body = read_standalone_table_prefix_10_1(reader, refs, max, max)?;
                if body.stop != StandaloneTableStop10_1::Complete {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!(
                            "NALayer 独立表 offset 0x{offset:x} 键 {key} 读取至 {:?}",
                            body.stop
                        ),
                    ));
                }
                Some(Box::new(body))
            }
            ObjectEntry10_1::Null
            | ObjectEntry10_1::BackReference {
                class_id: STANDALONE_TABLE_CLASS_ID,
                ..
            } => None,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("NALayer 独立表 offset 0x{offset:x} 的对象类未支持"),
                ));
            }
        };
        entries.push((entry, body));
    }
    Ok(NaLayerTables10_1 {
        declared_count,
        entries,
    })
}

fn read_na_layer_bitset(reader: &mut Reader<'_>, max: usize) -> io::Result<NaLayerBitSet10_1> {
    let header = [
        reader.read_u32_le()?,
        reader.read_u32_le()?,
        reader.read_u32_le()?,
        reader.read_u32_le()?,
    ];
    let has_words = reader.read_bytes(1)?[0];
    let mut words = Vec::new();
    if has_words != 0 {
        let count = header[3] as usize;
        if count > reader.remaining().min(max) / 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NALayer 位集字数超出可读范围",
            ));
        }
        for _ in 0..count {
            words.push(reader.read_u32_le()?);
        }
    }
    Ok(NaLayerBitSet10_1 {
        header,
        has_words,
        words,
    })
}

const NETWORK_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x6a, 0xfe, 0x67, 0x81, 0x92, 0xe9, 0xcb, 0x4e, 0xb6, 0xae, 0xe5, 0xdf, 0xe4, 0x65, 0x52, 0x47,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkDatasetName10_1 {
    pub version: u16,
    pub name: String,
    pub second_string: String,
    pub dataset: (ObjectEntry10_1, Option<Box<FeatureDatasetNameBody10_1>>),
    pub workspace: (ObjectEntry10_1, Option<Box<WorkspaceNamePrefix10_1>>),
}

pub(super) fn read_na_network_name_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: NETWORK_DATASET_NAME_CLASS_ID,
            ..
        } => Some(Box::new(read_network_dataset_name(
            &mut trial, max_bytes, &mut keys,
        )?)),
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported_name("NALayer IName")),
        _ => None,
    };
    *reader = trial;
    *refs = keys;
    Ok((entry, body))
}

fn read_network_dataset_name(
    r: &mut Reader<'_>,
    max: usize,
    keys: &mut RefTable10_1,
) -> io::Result<NetworkDatasetName10_1> {
    let version = r.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NetworkDatasetName 版本超过 1",
        ));
    }
    let name = read_string_10_1(r, max)?;
    let second_string = read_string_10_1(r, max)?;
    let dataset = read_object_entry_10_1(r, keys)?;
    let dataset_body = match dataset {
        ObjectEntry10_1::NewObject {
            class_id: FEATURE_DATASET_NAME_CLASS_ID,
            key,
        } => {
            let body = read_feature_dataset_name_body_10_1(r, max, keys, key)?;
            if body.stop.is_some() {
                return Err(unsupported_name("FeatureDatasetName"));
            }
            Some(Box::new(body))
        }
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported_name("IDatasetName")),
        _ => None,
    };
    let workspace = read_object_entry_10_1(r, keys)?;
    let workspace_body = match workspace {
        ObjectEntry10_1::NewObject {
            class_id: WORKSPACE_NAME_CLASS_ID,
            key,
        } => {
            let body = read_workspace_name_body_10_1(r, max, keys, key)?;
            if body.stop.is_some() {
                return Err(unsupported_name("WorkspaceName"));
            }
            Some(Box::new(body))
        }
        ObjectEntry10_1::NewObject { .. } => return Err(unsupported_name("IWorkspaceName")),
        _ => None,
    };
    Ok(NetworkDatasetName10_1 {
        version,
        name,
        second_string,
        dataset: (dataset, dataset_body),
        workspace: (workspace, workspace_body),
    })
}

fn unsupported_name(slot: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        format!("{slot} 名称对象体未支持"),
    )
}

const NA_CONTEXT_CLASS_ID: [u8; 16] = [
    0x5b, 0xb9, 0x22, 0xe3, 0x89, 0xb6, 0x57, 0x4c, 0x99, 0x7f, 0xbd, 0xb2, 0x21, 0x20, 0x47, 0x38,
];
const NETWORK_ANALYSIS_WORKSPACE_CLASS_ID: [u8; 16] = [
    0xca, 0xb0, 0x00, 0xec, 0xb9, 0x1c, 0x38, 0x41, 0x8c, 0x88, 0x3a, 0x18, 0xfc, 0xf8, 0xd7, 0x08,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NaRouteSolverPrefix10_1 {
    pub version: u16,
    pub attribute_name: String,
    pub extra_collection_flag: u16,
    pub inline_collections: Option<NaRouteCollections10_1>,
    pub stored_strings: Option<StringArraySlot10_1>,
    pub hierarchy: Option<NaRouteHierarchy10_1>,
    pub properties: Option<(ObjectEntry10_1, Option<Box<PropertySetObjectStream10_1>>)>,
    pub tail: Option<NaRouteSolverTail10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaRouteCollections10_1 {
    pub extra: Option<(u16, Vec<String>)>,
    pub attributes: (u16, Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaRouteHierarchy10_1 {
    pub flag: u16,
    pub mode: i32,
    pub attribute_name: String,
    pub declared_count: i32,
    pub indexed_values: [Vec<i32>; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaRouteSolverTail10_1 {
    pub shorts: (u16, Option<u16>),
    pub parameters: Option<(ObjectEntry10_1, Option<NaParameterArray10_1>)>,
    pub common_v3: Option<(u64, i32)>,
    pub route_specific: Option<(u64, u16)>,
    pub closest_facility_specific: Option<(u64, i32)>,
    pub service_area_specific: Option<NaServiceAreaSolverTail10_1>,
    pub od_cost_matrix_v3: Option<(i32, u16)>,
    pub od_cost_matrix_v5: Option<(u64, i32)>,
    pub vrp: Option<NaVrpSolverTail10_1>,
    pub location_allocation: Option<NaLocationAllocationSolverTail10_1>,
}

const NA_LOCATOR_CLASS_ID: [u8; 16] = [
    0xe1, 0xad, 0x19, 0x43, 0xe5, 0xd6, 0x18, 0x48, 0x83, 0x80, 0x8e, 0xe3, 0x89, 0xd1, 0x48, 0xea,
];

const NA_LOCATOR_FEATURE_AGENT_CLASS_ID: [u8; 16] = [
    0xab, 0xc9, 0xb4, 0x22, 0xfa, 0x32, 0x86, 0x46, 0x8f, 0x5e, 0x8a, 0x2a, 0x37, 0x0b, 0x33, 0x50,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaLocationAllocationSolverTail10_1 {
    pub options: [i32; 4],
    pub scalar_bits: [u64; 3],
    pub mode: i32,
    pub version_three_values: Option<(u64, i32)>,
    pub version_four_bits: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaVrpSolverTail10_1 {
    pub mode: i32,
    pub flag: u16,
    pub options: [i32; 2],
    pub primary_bits: u64,
    pub primary_mode: i32,
    pub remaining_scalar_bits: [u64; 4],
    pub final_mode: i32,
    pub extra: Option<(ObjectEntry10_1, Option<NaDirectionsAgent10_1>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaServiceAreaSolverTail10_1 {
    pub flags: [u16; 3],
    pub scalar_bits: u64,
    pub mode: i32,
    pub version_two_flag: Option<u16>,
    pub version_four_values: Option<(u64, i32)>,
}

const NA_LOCATOR_LOCATION_FIELDS_AGENT_CLASS_ID: [u8; 16] = [
    0xf2, 0xaf, 0x79, 0x94, 0xde, 0x7f, 0x59, 0x4c, 0xb7, 0x51, 0x0c, 0xca, 0xd3, 0x8b, 0x08, 0x9e,
];

#[derive(Debug, Clone, PartialEq)]
pub enum NaLocatorAgent10_1 {
    Feature(NaLocatorFeatureAgent10_1),
    LocationFields {
        version: u16,
        fields: [String; 4],
        extra: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLocatorFeatureAgent10_1 {
    pub version: u16,
    pub scalars: (i32, u16, i32),
    pub source: String,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLocatorTail10_1 {
    pub agents: Vec<(ObjectEntry10_1, Option<NaLocatorAgent10_1>)>,
    pub distance_bits: Option<u64>,
    pub flag: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaLocatorPrefix10_1 {
    pub version: u16,
    pub scalar_bits: [u64; 2],
    pub mode: i32,
    pub flag: u16,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub agent_count: i32,
    pub tail: Option<NaLocatorTail10_1>,
}

const NA_NAMED_SET_CLASS_ID: [u8; 16] = [
    0x84, 0xfe, 0xb4, 0xf6, 0x02, 0x7a, 0x69, 0x49, 0x9e, 0x72, 0x11, 0xb4, 0xf8, 0x37, 0x71, 0x55,
];

const NA_STREET_DIRECTIONS_AGENT_CLASS_ID: [u8; 16] = [
    0xa1, 0xc9, 0x1b, 0xc2, 0x84, 0xb7, 0xf1, 0x48, 0x8f, 0xe7, 0xdf, 0x7f, 0x12, 0xf7, 0x7d, 0x72,
];
const NA_COMPACT_DIRECTIONS_AGENT_CLASS_ID: [u8; 16] = [
    0x09, 0x0a, 0xab, 0xf2, 0xc3, 0x43, 0xe3, 0x4b, 0xbb, 0x74, 0x1b, 0x60, 0xe0, 0x4b, 0x3d, 0xd0,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaDirectionsAgent10_1 {
    pub version: i32,
    pub mode: Option<i32>,
    pub street_fields: Option<[i32; 2]>,
    pub attribute: Option<String>,
    pub locale: Option<String>,
    pub profile: Option<(i32, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NaNamedSetObject10_1 {
    Class(Box<super::na_class::NaClassBody10_1>),
    Directions(NaDirectionsAgent10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaNamedSetItem10_1 {
    pub name: String,
    pub entry: ObjectEntry10_1,
    pub body: Option<NaNamedSetObject10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaNamedSetBody10_1 {
    pub version: u16,
    pub declared_count: i32,
    pub items: Vec<NaNamedSetItem10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaContextPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub workspace: ObjectEntry10_1,
    pub workspace_prefix: Option<Box<NetworkAnalysisWorkspacePrefix10_1>>,
    pub dataset: Option<(ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>)>,
    pub solver: Option<(ObjectEntry10_1, Option<NaRouteSolverPrefix10_1>)>,
    /// 求解器之后、属性集之前的对象槽。
    pub post_solver_object: Option<ObjectEntry10_1>,
    pub properties: Option<(ObjectEntry10_1, Option<Box<PropertySetObjectStream10_1>>)>,
    pub locator: Option<(ObjectEntry10_1, Option<NaLocatorPrefix10_1>)>,
    pub class_set: Option<(ObjectEntry10_1, Option<NaNamedSetBody10_1>)>,
    pub secondary_set: Option<(ObjectEntry10_1, Option<NaNamedSetBody10_1>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkAnalysisWorkspacePrefix10_1 {
    pub version: u16,
    pub strings: [String; 3],
    pub properties: ObjectEntry10_1,
    pub property_set: Option<Box<PropertySetObjectStream10_1>>,
    pub named_sets: Vec<NaNamedSet10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaNamedSet10_1 {
    pub name: String,
    pub class: (
        ObjectEntry10_1,
        Option<Box<super::na_class::NaClassBody10_1>>,
    ),
}

/// 读取目前已确认的上下文和工作空间前缀，游标停在首个未读字段。
///
/// 证据：NetworkAnalyst.dll 10.1.0.3035 的 NAContext::Load `0x7af12a10`
/// 读取有符号版本上限 0、AfCore String 和首个工作空间对象；
/// NetworkAnalysisWorkspace::Load `0x7af7ec40` 读取有符号版本上限 1 后委派
/// GdbCoreLib.dll 10.1.0.3035 的 InMemoryWorkspaceBase::LoadWorkspace
/// `0x70d31ca0`。后者依次读取三个 AfCore String、一个 IPropertySet 对象，
/// 再读取四个字节的命名项计数。上下文后续槽仅在前一对象体完整读取后推进。
pub(super) fn read_na_context_prefix_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Box<NaContextPrefix10_1>>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let context = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match context {
        ObjectEntry10_1::NewObject {
            class_id: NA_CONTEXT_CLASS_ID,
            ..
        } => {
            let version = trial.read_u16_le()?;
            if version as i16 > 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NAContext 版本超过 0",
                ));
            }
            let name = read_string_10_1(&mut trial, max)?;
            let workspace = read_object_entry_10_1(&mut trial, &mut keys)?;
            let workspace_prefix = match workspace {
                ObjectEntry10_1::NewObject {
                    class_id: NETWORK_ANALYSIS_WORKSPACE_CLASS_ID,
                    ..
                } => Some(Box::new(read_na_workspace_prefix(
                    &mut trial, max, &mut keys,
                )?)),
                ObjectEntry10_1::NewObject { .. } => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "NAContext 工作空间对象体未支持",
                    ));
                }
                _ => None,
            };
            let dataset = read_na_network_name_10_1(&mut trial, max, &mut keys)?;
            // 求解器只读取已确认前缀，失败时保留其对象头与后续字节。
            let solver = read_na_solver_prefix(&mut trial, max, &mut keys).ok();
            let solver_complete = solver
                .as_ref()
                .is_some_and(|(_, body)| body.as_ref().is_none_or(|body| body.tail.is_some()));
            let post_solver_object = solver_complete
                .then(|| read_na_context_post_solver_slot(&mut trial, &mut keys).ok())
                .flatten();
            let properties = post_solver_object
                .as_ref()
                .and_then(|_| read_na_property_set_slot(&mut trial, max, &mut keys).ok());
            let locator = properties
                .as_ref()
                .and_then(|_| read_na_locator_prefix(&mut trial, max, &mut keys).ok());
            let locator_complete = locator
                .as_ref()
                .is_some_and(|(_, body)| body.as_ref().is_none_or(|body| body.tail.is_some()));
            let class_set = locator_complete
                .then(|| read_na_named_set(&mut trial, max, &mut keys).ok())
                .flatten();
            let secondary_set = class_set
                .as_ref()
                .and_then(|_| read_na_named_set(&mut trial, max, &mut keys).ok());
            Some(Box::new(NaContextPrefix10_1 {
                version,
                name,
                workspace,
                workspace_prefix,
                dataset: Some(dataset),
                solver,
                post_solver_object,
                properties,
                locator,
                class_set,
                secondary_set,
            }))
        }
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NALayer 上下文对象体未支持",
            ));
        }
        _ => None,
    };
    *reader = trial;
    *refs = keys;
    Ok((context, body))
}

const NA_LOCATION_ALLOCATION_SOLVER_CLASS_ID: [u8; 16] = [
    0x5b, 0xc6, 0x9c, 0x65, 0x56, 0xd6, 0x32, 0x4f, 0x87, 0xf2, 0xfd, 0x0d, 0x30, 0x58, 0xdd, 0xf9,
];

const NA_VRP_SOLVER_CLASS_ID: [u8; 16] = [
    0x78, 0x22, 0xb6, 0x52, 0x5f, 0x73, 0xa1, 0x4d, 0x8a, 0x4e, 0x2e, 0x6f, 0x4e, 0x3b, 0xcd, 0x60,
];

const NA_OD_COST_MATRIX_SOLVER_CLASS_ID: [u8; 16] = [
    0x73, 0xe1, 0x5b, 0x86, 0x20, 0x5d, 0x11, 0x4f, 0x90, 0x16, 0xff, 0x27, 0xbc, 0xa1, 0x67, 0xe2,
];

const NA_SERVICE_AREA_SOLVER_CLASS_ID: [u8; 16] = [
    0x3c, 0x4d, 0xc7, 0xcf, 0x8a, 0x99, 0x33, 0x4d, 0xb7, 0xb2, 0x5d, 0x27, 0x9c, 0xaf, 0x07, 0x63,
];

const NA_CLOSEST_FACILITY_SOLVER_CLASS_ID: [u8; 16] = [
    0xd4, 0x83, 0x1f, 0x77, 0x98, 0xc6, 0x36, 0x48, 0x82, 0x22, 0xc0, 0xb6, 0x89, 0xa8, 0x58, 0x27,
];

const NA_ROUTE_SOLVER_CLASS_ID: [u8; 16] = [
    0x08, 0xa2, 0x8a, 0x1a, 0x4a, 0x3d, 0x0f, 0x42, 0xb8, 0x2c, 0xf5, 0x81, 0x94, 0xa7, 0x2c, 0xb0,
];

/// NALocator::Load 0x7af31e90 读取固定字段、ExtraSafe 空间参考槽及代理数量。
fn read_na_locator_prefix(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<NaLocatorPrefix10_1>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: NA_LOCATOR_CLASS_ID,
            ..
        } => {
            let version = trial.read_u16_le()?;
            if version as i16 > 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NALocator 版本超过 3",
                ));
            }
            let scalar_bits = [
                trial.read_f64_le()?.to_bits(),
                trial.read_f64_le()?.to_bits(),
            ];
            let mode = trial.read_u32_le()? as i32;
            let flag = trial.read_u16_le()?;
            let spatial_reference = read_na_locator_spatial_reference(&mut trial, max, &mut keys)?;
            let agent_count = trial.read_u32_le()? as i32;
            let tail = read_na_locator_tail(&mut trial, max, &mut keys, version, agent_count).ok();
            Some(NaLocatorPrefix10_1 {
                version,
                scalar_bits,
                mode,
                flag,
                spatial_reference,
                agent_count,
                tail,
            })
        }
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NAContext 定位器类未支持",
            ));
        }
        _ => None,
    };
    *reader = trial;
    *refs = keys;
    Ok((entry, body))
}

fn read_na_locator_tail(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    version: u16,
    agent_count: i32,
) -> io::Result<NaLocatorTail10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let count = agent_count.max(0) as usize;
    if count > trial.remaining().min(max) / 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "定位器代理数量超出可读范围",
        ));
    }
    let mut agents = Vec::new();
    for _ in 0..count {
        agents.push(read_na_locator_agent(&mut trial, max, &mut keys)?);
    }
    let distance_bits = (version as i16 >= 2)
        .then(|| trial.read_f64_le().map(f64::to_bits))
        .transpose()?;
    let flag = (version == 3).then(|| trial.read_u16_le()).transpose()?;
    *reader = trial;
    *refs = keys;
    Ok(NaLocatorTail10_1 {
        agents,
        distance_bits,
        flag,
    })
}

/// NALocator::Load 0x7af31e90 的 ExtraSafe ISpatialReference 槽。
fn read_na_locator_spatial_reference(
    reader: &mut Reader<'_>,
    max: usize,
    keys: &mut RefTable10_1,
) -> io::Result<SpatialReferenceEntry10_1> {
    let Some(mut region) = read_na_extra_safe_region(reader, max)? else {
        return Ok(SpatialReferenceEntry10_1::Null);
    };
    let spatial_reference = read_spatial_reference_entry_10_1(&mut region, max, keys)?;
    if region.remaining() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NALocator 空间参考未消费声明范围",
        ));
    }
    Ok(spatial_reference)
}

fn read_na_locator_agent(
    reader: &mut Reader<'_>,
    max: usize,
    keys: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<NaLocatorAgent10_1>)> {
    let Some(mut region) = read_na_extra_safe_region(reader, max)? else {
        return Ok((ObjectEntry10_1::Null, None));
    };
    let entry = read_object_entry_10_1(&mut region, keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: NA_LOCATOR_FEATURE_AGENT_CLASS_ID,
            ..
        } => {
            let version = region.read_u16_le()?;
            if version as i16 > 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "定位器要素代理版本超过 2",
                ));
            }
            let scalars = (
                region.read_u32_le()? as i32,
                region.read_u16_le()?,
                region.read_u32_le()? as i32,
            );
            let source = read_bstr_string_10_1(&mut region, max)?;
            let extra = (version == 2)
                .then(|| read_bstr_string_10_1(&mut region, max))
                .transpose()?;
            Some(NaLocatorAgent10_1::Feature(NaLocatorFeatureAgent10_1 {
                version,
                scalars,
                source,
                extra,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id: NA_LOCATOR_LOCATION_FIELDS_AGENT_CLASS_ID,
            ..
        } => {
            let version = region.read_u16_le()?;
            if version as i16 > 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "定位器位置字段代理版本超过 2",
                ));
            }
            let fields = [
                read_bstr_string_10_1(&mut region, max)?,
                read_bstr_string_10_1(&mut region, max)?,
                read_bstr_string_10_1(&mut region, max)?,
                read_bstr_string_10_1(&mut region, max)?,
            ];
            let extra = (version == 2)
                .then(|| read_bstr_string_10_1(&mut region, max))
                .transpose()?;
            Some(NaLocatorAgent10_1::LocationFields {
                version,
                fields,
                extra,
            })
        }
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "定位器 ExtraSafe 对象体未支持",
            ));
        }
        _ => None,
    };
    if region.remaining() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "定位器 ExtraSafe 对象未消费声明范围",
        ));
    }
    Ok((entry, body))
}

fn read_na_extra_safe_region<'a>(
    reader: &mut Reader<'a>,
    max: usize,
) -> io::Result<Option<Reader<'a>>> {
    let a = reader.read_u32_le()? as usize;
    let b = reader.read_u32_le()? as i32;
    if a == 0 && b == 0 {
        return Ok(None);
    }
    let length = if b == 0 {
        a
    } else {
        usize::try_from(b).unwrap_or_default()
    };
    if !(16..=max).contains(&length) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "网络分析 ExtraSafe 槽边界无效",
        ));
    }
    reader.read_region(length).map(Some)
}

/// NamedSet::Load 0x7af43920 逐项读取 AfCore String 和普通对象槽。
fn read_na_named_set(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<NaNamedSetBody10_1>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: NA_NAMED_SET_CLASS_ID,
            ..
        } => {
            let version = trial.read_u16_le()?;
            if version as i16 > 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NamedSet 版本超过 1",
                ));
            }
            let declared_count = trial.read_u32_le()? as i32;
            let count = declared_count.max(0) as usize;
            // 每项至少有六字节空字符串和十六字节对象头。
            if count > trial.remaining().min(max) / 22 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NamedSet 条目数量超出可读范围",
                ));
            }
            let mut items = Vec::new();
            for _ in 0..count {
                let name = read_string_10_1(&mut trial, max)?;
                let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
                let body = match entry {
                    ObjectEntry10_1::BackReference { .. } => None,
                    ObjectEntry10_1::NewObject {
                        class_id: super::na_class::NA_CLASS_CLASS_ID,
                        ..
                    } => Some(NaNamedSetObject10_1::Class(Box::new(
                        super::na_class::read_na_class_body(&mut trial, max, &mut keys)?,
                    ))),
                    ObjectEntry10_1::NewObject {
                        class_id:
                            class_id @ (NA_STREET_DIRECTIONS_AGENT_CLASS_ID
                            | NA_COMPACT_DIRECTIONS_AGENT_CLASS_ID),
                        ..
                    } => Some(NaNamedSetObject10_1::Directions(read_na_directions_agent(
                        &mut trial,
                        max,
                        class_id == NA_STREET_DIRECTIONS_AGENT_CLASS_ID,
                    )?)),
                    ObjectEntry10_1::Null => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "NamedSet 条目对象为空",
                        ));
                    }
                    ObjectEntry10_1::NewObject { .. } => {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "NamedSet 条目对象体未支持",
                        ));
                    }
                };
                items.push(NaNamedSetItem10_1 { name, entry, body });
            }
            Some(NaNamedSetBody10_1 {
                version,
                declared_count,
                items,
            })
        }
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NAContext 命名集合类未支持",
            ));
        }
        _ => None,
    };
    *reader = trial;
    *refs = keys;
    Ok((entry, body))
}

/// Directions.dll 的街道和紧凑街道方向代理读侧分别为 0x6ea2d510、0x6ea224a0。
fn read_na_directions_agent(
    reader: &mut Reader<'_>,
    max: usize,
    street: bool,
) -> io::Result<NaDirectionsAgent10_1> {
    let version = reader.read_u32_le()? as i32;
    if version > 2 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "方向代理版本超过 2",
        ));
    }
    let read_long = |reader: &mut Reader<'_>| reader.read_u32_le().map(|value| value as i32);
    let mode = (version >= 0).then(|| read_long(reader)).transpose()?;
    let street_before = (street && version >= 0)
        .then(|| read_long(reader))
        .transpose()?;
    let attribute = (version >= 0)
        .then(|| read_string_10_1(reader, max))
        .transpose()?;
    let street_fields = street_before
        .map(|before| Ok::<_, io::Error>([before, read_long(reader)?]))
        .transpose()?;
    let locale = (version >= 1)
        .then(|| read_string_10_1(reader, max))
        .transpose()?;
    let profile = (version >= 2)
        .then(|| Ok::<_, io::Error>((read_long(reader)?, read_string_10_1(reader, max)?)))
        .transpose()?;
    Ok(NaDirectionsAgent10_1 {
        version,
        mode,
        street_fields,
        attribute,
        locale,
        profile,
    })
}

fn read_na_context_post_solver_slot(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
) -> io::Result<ObjectEntry10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    if matches!(entry, ObjectEntry10_1::NewObject { .. }) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "NAContext 求解器后首个对象体未支持",
        ));
    }
    *reader = trial;
    *refs = keys;
    Ok(entry)
}

fn read_na_solver_prefix(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<NaRouteSolverPrefix10_1>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (NA_ROUTE_SOLVER_CLASS_ID
                | NA_CLOSEST_FACILITY_SOLVER_CLASS_ID
                | NA_SERVICE_AREA_SOLVER_CLASS_ID
                | NA_OD_COST_MATRIX_SOLVER_CLASS_ID
                | NA_VRP_SOLVER_CLASS_ID
                | NA_LOCATION_ALLOCATION_SOLVER_CLASS_ID),
            ..
        } => {
            let version = trial.read_u16_le()?;
            let max_version = match class_id {
                NA_VRP_SOLVER_CLASS_ID => 2,
                NA_ROUTE_SOLVER_CLASS_ID => 3,
                NA_OD_COST_MATRIX_SOLVER_CLASS_ID => 5,
                _ => 4,
            };
            if version as i16 > max_version {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "网络分析求解器版本超过支持范围",
                ));
            }
            let attribute_name = read_bstr_string_10_1(&mut trial, max)?;
            let extra_collection_flag = trial.read_u16_le()?;
            let inline_collections =
                read_na_route_collections(&mut trial, max, extra_collection_flag).ok();
            let stored_strings = inline_collections
                .as_ref()
                .and_then(|_| read_na_route_string_slot(&mut trial, max, &mut keys).ok());
            let hierarchy = stored_strings
                .as_ref()
                .and_then(|_| read_na_route_hierarchy(&mut trial, max).ok());
            let properties = hierarchy
                .as_ref()
                .and_then(|_| read_na_property_set_slot(&mut trial, max, &mut keys).ok());
            let tail = properties.as_ref().and_then(|_| {
                read_na_solver_tail(&mut trial, max, &mut keys, version, class_id).ok()
            });
            Some(NaRouteSolverPrefix10_1 {
                version,
                attribute_name,
                extra_collection_flag,
                inline_collections,
                stored_strings,
                hierarchy,
                properties,
                tail,
            })
        }
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NAContext 求解器类未支持",
            ));
        }
        _ => None,
    };
    *reader = trial;
    *refs = keys;
    Ok((entry, body))
}

fn read_na_route_collections(
    reader: &mut Reader<'_>,
    max: usize,
    extra_collection_flag: u16,
) -> io::Result<NaRouteCollections10_1> {
    let mut trial = reader.clone();
    let extra = (extra_collection_flag != 0)
        .then(|| read_string_array_body_10_1(&mut trial, max))
        .transpose()?;
    let attributes = read_string_array_body_10_1(&mut trial, max)?;
    *reader = trial;
    Ok(NaRouteCollections10_1 { extra, attributes })
}

fn read_na_route_string_slot(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<StringArraySlot10_1> {
    let mut trial = reader.clone();
    let a = trial.read_u32_le()?;
    let b = trial.read_u32_le()? as i32;
    if a == 0 && b == 0 {
        *reader = trial;
        return Ok(StringArraySlot10_1 {
            entry: ObjectEntry10_1::Null,
            body: None,
        });
    }

    // B 非零时是原生同步边界，否则 A 给出写侧对象头和体的总长度。
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "求解器 ExtraSafe 字符串数组边界无效",
        )
    };
    let length = if b == 0 {
        a as usize
    } else {
        usize::try_from(b).map_err(|_| invalid())?
    };
    if length < 16 || length > max {
        return Err(invalid());
    }
    let mut body = trial.read_region(length)?;
    let mut keys = refs.clone();
    let slot = read_string_array_slot_10_1(&mut body, max, &mut keys)?;
    if body.remaining() != 0 {
        return Err(invalid());
    }
    *reader = trial;
    *refs = keys;
    Ok(slot)
}

fn read_na_route_hierarchy(
    reader: &mut Reader<'_>,
    max: usize,
) -> io::Result<NaRouteHierarchy10_1> {
    let mut trial = reader.clone();
    let flag = trial.read_u16_le()?;
    let mode = trial.read_u32_le()? as i32;
    let attribute_name = read_bstr_string_10_1(&mut trial, max)?;
    let declared_count = trial.read_u32_le()? as i32;
    let count = declared_count.max(0) as usize;
    let mut indexed_values = [Vec::new(), Vec::new()];
    for list in &mut indexed_values {
        if trial.read_u32_le()? as i32 != declared_count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "求解器列表计数不一致",
            ));
        }
        if count > trial.remaining().min(max) / 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "求解器索引列表超出可读范围",
            ));
        }
        for index in 1..=count {
            if trial.read_u32_le()? as usize != index {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "求解器列表索引不连续",
                ));
            }
            list.push(trial.read_u32_le()? as i32);
        }
    }
    *reader = trial;
    Ok(NaRouteHierarchy10_1 {
        flag,
        mode,
        attribute_name,
        declared_count,
        indexed_values,
    })
}

fn read_na_property_set_slot(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Box<PropertySetObjectStream10_1>>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: PROPERTY_SET_CLASS_ID,
            ..
        } => Some(Box::new(read_property_set_object_stream_10_1(
            &mut trial, max, &mut keys,
        )?)),
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "网络分析属性对象体未支持",
            ));
        }
        _ => None,
    };
    *reader = trial;
    *refs = keys;
    Ok((entry, body))
}

const NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID: [u8; 16] = [
    0xa8, 0x3f, 0xed, 0xf8, 0x8b, 0x0b, 0x56, 0x4a, 0xa5, 0xb1, 0x22, 0x97, 0x3b, 0x32, 0x90, 0xc9,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NaParameterArray10_1 {
    pub version: u16,
    pub declared_count: i32,
    pub items: Vec<(ObjectEntry10_1, Option<NaParameterValue10_1>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaParameterValue10_1 {
    pub version: u16,
    pub attribute: String,
    pub parameter: String,
    pub mode: i32,
    pub value: PropertyValueObjectStream10_1,
}

/// 数组 Load 0x7b09f110 经 0x7b09ec60 逐项读取普通对象。
fn read_na_parameter_array(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NaParameterArray10_1> {
    let version = reader.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "网络属性参数数组版本超过 1",
        ));
    }
    let declared_count = reader.read_u32_le()? as i32;
    let count = declared_count.max(0) as usize;
    if count > reader.remaining().min(max) / 20 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "网络属性参数数量超出可读范围",
        ));
    }
    let mut items = Vec::new();
    for _ in 0..count {
        let entry = read_object_entry_10_1(reader, refs)?;
        let body = match entry {
            ObjectEntry10_1::BackReference {
                class_id: NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID,
                ..
            } => None,
            ObjectEntry10_1::NewObject {
                class_id: NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID,
                ..
            } => {
                let version = reader.read_u16_le()?;
                if version as i16 > 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "网络属性参数版本超过 0",
                    ));
                }
                let attribute = read_bstr_string_10_1(reader, max)?;
                let parameter = read_bstr_string_10_1(reader, max)?;
                let mode = reader.read_u32_le()? as i32;
                let value = read_padded_variant_object_stream_10_1(reader, max, refs)?;
                Some(NaParameterValue10_1 {
                    version,
                    attribute,
                    parameter,
                    mode,
                    value,
                })
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "网络属性参数项对象类未支持",
                ));
            }
        };
        items.push((entry, body));
    }
    Ok(NaParameterArray10_1 {
        version,
        declared_count,
        items,
    })
}

const NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID: [u8; 16] = [
    0x6b, 0x9e, 0x93, 0x61, 0x11, 0x80, 0xc5, 0x40, 0xb6, 0x72, 0x73, 0xea, 0x1e, 0x02, 0x6f, 0xec,
];

fn read_na_solver_tail(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    version: u16,
    class_id: [u8; 16],
) -> io::Result<NaRouteSolverTail10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    // 原生先验有符号上限，再将 0、1、2 之外的值映射到共用 v3 分支。
    let common_version = match (class_id, version) {
        (NA_OD_COST_MATRIX_SOLVER_CLASS_ID, 3) => 2,
        (NA_VRP_SOLVER_CLASS_ID | NA_LOCATION_ALLOCATION_SOLVER_CLASS_ID, _) => {
            if version as i16 > 1 { 3 } else { 2 }
        }
        _ => version.min(3),
    };
    let first = trial.read_u16_le()?;
    let second = (common_version >= 1)
        .then(|| trial.read_u16_le())
        .transpose()?;
    let parameters = if common_version >= 2 {
        let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
        let array_header = match entry {
            ObjectEntry10_1::NewObject {
                class_id: NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID,
                ..
            } => Some(read_na_parameter_array(&mut trial, max, &mut keys)?),
            ObjectEntry10_1::NewObject { .. } => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "求解器参数槽对象体未支持",
                ));
            }
            _ => None,
        };
        Some((entry, array_header))
    } else {
        None
    };
    let common_v3 = (common_version == 3)
        .then(|| Ok::<_, io::Error>((trial.read_f64_le()?.to_bits(), trial.read_u32_le()? as i32)))
        .transpose()?;
    let route_specific = (class_id == NA_ROUTE_SOLVER_CLASS_ID && version != 0)
        .then(|| Ok::<_, io::Error>((trial.read_f64_le()?.to_bits(), trial.read_u16_le()?)))
        .transpose()?;
    let closest_facility_specific = (class_id == NA_CLOSEST_FACILITY_SOLVER_CLASS_ID
        && version == 4)
        .then(|| Ok::<_, io::Error>((trial.read_f64_le()?.to_bits(), trial.read_u32_le()? as i32)))
        .transpose()?;
    let service_area_specific = (class_id == NA_SERVICE_AREA_SOLVER_CLASS_ID && version != 0)
        .then(|| read_na_service_area_tail(&mut trial, version as i16))
        .transpose()?;
    let od_cost_matrix_v3 = (class_id == NA_OD_COST_MATRIX_SOLVER_CLASS_ID && version as i16 >= 3)
        .then(|| Ok::<_, io::Error>((trial.read_u32_le()? as i32, trial.read_u16_le()?)))
        .transpose()?;
    let od_cost_matrix_v5 = (class_id == NA_OD_COST_MATRIX_SOLVER_CLASS_ID && version == 5)
        .then(|| Ok::<_, io::Error>((trial.read_f64_le()?.to_bits(), trial.read_u32_le()? as i32)))
        .transpose()?;
    let vrp = (class_id == NA_VRP_SOLVER_CLASS_ID)
        .then(|| read_na_vrp_tail(&mut trial, max, &mut keys, version))
        .transpose()?;
    let location_allocation = (class_id == NA_LOCATION_ALLOCATION_SOLVER_CLASS_ID)
        .then(|| read_na_location_allocation_tail(&mut trial, version as i16))
        .transpose()?;
    if trial.position() - reader.position() > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "求解器尾字段超过资源限额",
        ));
    }
    *reader = trial;
    *refs = keys;
    Ok(NaRouteSolverTail10_1 {
        shorts: (first, second),
        parameters,
        common_v3,
        route_specific,
        closest_facility_specific,
        service_area_specific,
        od_cost_matrix_v3,
        od_cost_matrix_v5,
        vrp,
        location_allocation,
    })
}

// NALocationAllocationSolver::Load 0x7b0d2cb0.
fn read_na_location_allocation_tail(
    reader: &mut Reader<'_>,
    version: i16,
) -> io::Result<NaLocationAllocationSolverTail10_1> {
    let options = [
        reader.read_u32_le()? as i32,
        reader.read_u32_le()? as i32,
        reader.read_u32_le()? as i32,
        reader.read_u32_le()? as i32,
    ];
    let scalar_bits = [
        reader.read_f64_le()?.to_bits(),
        reader.read_f64_le()?.to_bits(),
        reader.read_f64_le()?.to_bits(),
    ];
    let mode = reader.read_u32_le()? as i32;
    let version_three_values = (version >= 3)
        .then(|| {
            Ok::<_, io::Error>((
                reader.read_f64_le()?.to_bits(),
                reader.read_u32_le()? as i32,
            ))
        })
        .transpose()?;
    let version_four_bits = (version >= 4)
        .then(|| reader.read_f64_le().map(f64::to_bits))
        .transpose()?;
    Ok(NaLocationAllocationSolverTail10_1 {
        options,
        scalar_bits,
        mode,
        version_three_values,
        version_four_bits,
    })
}

// NAVRPSolver::Load 0x7b14b230.
fn read_na_vrp_tail(
    reader: &mut Reader<'_>,
    max: usize,
    keys: &mut RefTable10_1,
    version: u16,
) -> io::Result<NaVrpSolverTail10_1> {
    let mode = reader.read_u32_le()? as i32;
    let flag = reader.read_u16_le()?;
    let options = [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32];
    let primary_bits = reader.read_f64_le()?.to_bits();
    let primary_mode = reader.read_u32_le()? as i32;
    let remaining_scalar_bits = [
        reader.read_f64_le()?.to_bits(),
        reader.read_f64_le()?.to_bits(),
        reader.read_f64_le()?.to_bits(),
        reader.read_f64_le()?.to_bits(),
    ];
    let final_mode = reader.read_u32_le()? as i32;
    let extra = if version != 0 {
        if let Some(mut region) = read_na_extra_safe_region(reader, max)? {
            let entry = read_object_entry_10_1(&mut region, keys)?;
            let agent = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: NA_STREET_DIRECTIONS_AGENT_CLASS_ID,
                    ..
                } => Some(read_na_directions_agent(&mut region, max, true)?),
                ObjectEntry10_1::NewObject {
                    class_id: NA_COMPACT_DIRECTIONS_AGENT_CLASS_ID,
                    ..
                } => Some(read_na_directions_agent(&mut region, max, false)?),
                ObjectEntry10_1::NewObject { .. } => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "车辆路径求解器 ExtraSafe 对象体未支持",
                    ));
                }
                _ => None,
            };
            if region.remaining() != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "车辆路径求解器 ExtraSafe 槽未消费声明范围",
                ));
            }
            Some((entry, agent))
        } else {
            Some((ObjectEntry10_1::Null, None))
        }
    } else {
        None
    };
    Ok(NaVrpSolverTail10_1 {
        mode,
        flag,
        options,
        primary_bits,
        primary_mode,
        remaining_scalar_bits,
        final_mode,
        extra,
    })
}

fn read_na_service_area_tail(
    reader: &mut Reader<'_>,
    version: i16,
) -> io::Result<NaServiceAreaSolverTail10_1> {
    let flags = [
        reader.read_u16_le()?,
        reader.read_u16_le()?,
        reader.read_u16_le()?,
    ];
    let scalar_bits = reader.read_f64_le()?.to_bits();
    let mode = reader.read_u32_le()? as i32;
    let version_two_flag = (version >= 2).then(|| reader.read_u16_le()).transpose()?;
    let version_four_values = (version >= 4)
        .then(|| {
            Ok::<_, io::Error>((
                reader.read_f64_le()?.to_bits(),
                reader.read_u32_le()? as i32,
            ))
        })
        .transpose()?;
    Ok(NaServiceAreaSolverTail10_1 {
        flags,
        scalar_bits,
        mode,
        version_two_flag,
        version_four_values,
    })
}

fn read_na_workspace_prefix(
    reader: &mut Reader<'_>,
    max: usize,
    keys: &mut RefTable10_1,
) -> io::Result<NetworkAnalysisWorkspacePrefix10_1> {
    let version = reader.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NetworkAnalysisWorkspace 版本超过 1",
        ));
    }
    let strings = [
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
    ];
    let properties = read_object_entry_10_1(reader, keys)?;
    let property_set = match properties {
        ObjectEntry10_1::NewObject {
            class_id: PROPERTY_SET_CLASS_ID,
            ..
        } => Some(Box::new(read_property_set_object_stream_10_1(
            reader, max, keys,
        )?)),
        ObjectEntry10_1::NewObject { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NetworkAnalysisWorkspace 属性对象体未支持",
            ));
        }
        _ => None,
    };
    let count = usize::try_from(reader.read_u32_le()? as i32)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "NALayer 命名项数量为负"))?;
    if count > reader.remaining() / 10 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "NALayer 命名项数量超出可读范围",
        ));
    }
    let mut named_sets = Vec::with_capacity(count);
    for _ in 0..count {
        let name = read_string_10_1(reader, max)?;
        let class = super::na_class::read_na_class_10_1(reader, max, keys)?;
        named_sets.push(NaNamedSet10_1 { name, class });
    }
    Ok(NetworkAnalysisWorkspacePrefix10_1 {
        version,
        strings,
        properties,
        property_set,
        named_sets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        let utf16 = value.encode_utf16().collect::<Vec<_>>();
        let cb = u32::try_from((utf16.len() + 1) * 2).unwrap();
        bytes.extend(cb.to_le_bytes());
        for unit in utf16 {
            bytes.extend(unit.to_le_bytes());
        }
        bytes.extend(0_u16.to_le_bytes());
    }

    #[test]
    fn reads_prefix_and_leaves_nested_objects_for_the_host() {
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend([b'R', 0, 0, 0]);
        bytes.extend([1_u16, 2, 3].into_iter().flat_map(u16::to_le_bytes));
        bytes.extend([4_u64, 5].into_iter().flat_map(u64::to_le_bytes));
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_layer_prefix_10_1(&mut reader, 64).is_err());
            assert_eq!(reader.position(), 0);
        }

        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let result = read_na_layer_prefix_10_1(&mut reader, 64).unwrap();
        assert_eq!((result.version, result.name.as_str()), (3, "R"));
        assert_eq!(result.shorts, [1, 2, 3]);
        assert_eq!(result.double_bits, [4, 5]);
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }

    #[test]
    fn network_name_commits_nested_reference_keys_only_after_both_slots() {
        let mut bytes = NETWORK_DATASET_NAME_CLASS_ID.to_vec();
        bytes.extend(17_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for ch in [b'N', b'D'] {
            bytes.extend(4_u32.to_le_bytes());
            bytes.extend([ch, 0, 0, 0]);
        }
        bytes.extend(FEATURE_DATASET_NAME_CLASS_ID);
        bytes.extend(18_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for _ in 0..4 {
            bytes.extend(2_u32.to_le_bytes());
            bytes.extend([0; 2]);
        }
        bytes.push(0);
        bytes.extend([0; 16]); // IWorkspaceName 内槽。
        bytes.extend([0; 16]); // NetworkDatasetName 自身的 IWorkspaceName 槽。
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_network_name_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let (_, body) = read_na_network_name_10_1(&mut reader, 1024, &mut refs).unwrap();
        let body = body.unwrap();
        assert_eq!(body.name, "N");
        assert!(body.dataset.1.is_some());
        assert!(refs.contains(17) && refs.contains(18));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn context_prefix_reads_workspace_named_class_and_uid_transactionally() {
        let mut bytes = NA_CONTEXT_CLASS_ID.to_vec();
        bytes.extend(35_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
        push_string(&mut bytes, "Route");
        bytes.extend(NETWORK_ANALYSIS_WORKSPACE_CLASS_ID);
        bytes.extend(36_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        push_string(&mut bytes, "Route");
        push_string(&mut bytes, "Route:{08932E9A-AAED-4D8A-842D-2B2B0AEC6080}");
        push_string(&mut bytes, "{08932E9A-AAED-4D8A-842D-2B2B0AEC6080}");
        bytes.extend([0x11, 0x5a, 0x8e, 0x58, 0x9b, 0xd0, 0xd1, 0x11]);
        bytes.extend([0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3, 0x3a, 0x15]);
        bytes.extend(37_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes()); // 命名项计数。
        push_string(&mut bytes, "routes");
        bytes.extend(super::super::na_class::NA_CLASS_CLASS_ID);
        bytes.extend(38_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        push_string(&mut bytes, "");
        push_string(&mut bytes, "routes");
        bytes.extend([0; 16 + 4 + 4 + 4]); // 类标识、标量、要素类型、标记。
        bytes.extend([0; 16 * 3 + 8]); // 三个空对象槽、标量与记录计数。
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(super::super::na_class::NA_CLASS_DEFINITION_CLASS_ID);
        bytes.extend(39_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes()); // 空 BSTR。
        bytes.extend(crate::layers::feature_layer_tail::UID_CLASS_ID);
        bytes.extend(40_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 21]); // UID 标志、GUID、末尾整数。
        bytes.extend([0; 16 + 4 * 3 + 2 * 2]); // 空 Fields、边界及空字段类型列表。
        bytes.extend([0; 16]); // NAContext 的 IName 槽。

        for end in 0..bytes.len() {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..end]);
            assert!(
                read_na_context_prefix_10_1(&mut reader, 1024, &mut refs).is_err(),
                "截断前缀 {end} 不应被接受"
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }

        bytes.extend([0; 16]); // 空求解器。
        bytes.extend([0; 16]); // 求解器后的空槽。
        let property_at = bytes.len();
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(41_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        for end in property_at..bytes.len() {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..end]);
            let (_, context) = read_na_context_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
            assert!(context.unwrap().properties.is_none());
            assert_eq!(reader.position(), property_at);
            assert!(!refs.contains(41));
        }
        bytes.push(0x7f);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let (_, context) = read_na_context_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
        let context = context.unwrap();
        assert_eq!(context.post_solver_object, Some(ObjectEntry10_1::Null));
        assert!(context.properties.unwrap().1.is_some());
        assert!(refs.contains(41));
        let workspace = context.workspace_prefix.unwrap();
        assert_eq!(context.version, 0);
        assert_eq!(workspace.version, 1);
        assert_eq!(workspace.strings[0], "Route");
        assert_eq!(workspace.property_set.as_ref().unwrap().count, 0);
        let class = workspace.named_sets[0].class.1.as_ref().unwrap();
        assert_eq!(class.class_base.name, "routes");
        let definition = class.definition.1.as_ref().unwrap();
        assert_eq!(definition.uid.1.as_ref().unwrap().key, 40);
        assert!(refs.contains(38) && refs.contains(39) && refs.contains(40));
        assert!(refs.contains(35) && refs.contains(36) && refs.contains(37));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }

    #[test]
    fn na_layer_tail_reads_bounded_property_set_extension() {
        let mut bytes = vec![0; 2 + 16 + 8];
        bytes.extend(1_u32.to_le_bytes());
        let length_offset = bytes.len();
        bytes.extend(6_u64.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        push_string(&mut bytes, "Tail");
        bytes.extend([0; 2 + 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let body = read_na_layer_tail(&mut reader, 128, &mut refs, 0).unwrap();
        assert_eq!(body.name, "Tail");
        assert_eq!(body.extensions.len(), 1);
        assert!(body.extensions[0].properties.is_some());
        assert_eq!(reader.remaining(), 0);

        bytes[length_offset..length_offset + 8].copy_from_slice(&7_u64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert!(read_na_layer_tail(&mut reader, 128, &mut refs, 0).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn na_layer_tail_reads_envelope_and_rolls_back_its_key_on_truncation() {
        let mut bytes = vec![0; 2];
        bytes.extend(ENVELOPE_CLASS_ID);
        bytes.extend(54_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in [1.0_f64, 2.0, 3.0, 4.0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend([0; 8 + 4]);
        push_string(&mut bytes, "");
        bytes.extend([0; 2 + 16]);

        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_layer_tail(&mut reader, 256, &mut refs, 0).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let tail = read_na_layer_tail(&mut reader, 256, &mut refs, 0).unwrap();
        assert_eq!(tail.envelope.unwrap().coords, [1.0, 2.0, 3.0, 4.0]);
        assert!(refs.contains(54));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn na_layer_tail_resolves_standalone_table_references() {
        let mut bytes = vec![0; 2 + 16 + 8 + 4];
        push_string(&mut bytes, "");
        bytes.extend([0; 2 + 16]);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(STANDALONE_TABLE_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend([0; 16]);
        let mut refs = RefTable10_1::new();
        refs.register(77);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_layer_tail(&mut reader, 128, &mut refs, 1).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 1);
        }
        let mut reader = Reader::new(&bytes);
        let tables = read_na_layer_tail(&mut reader, 128, &mut refs, 1)
            .unwrap()
            .tables
            .unwrap();
        assert_eq!(tables.declared_count, 2);
        assert_eq!(
            tables.entries,
            [
                (
                    ObjectEntry10_1::BackReference {
                        class_id: STANDALONE_TABLE_CLASS_ID,
                        key: 77
                    },
                    None
                ),
                (ObjectEntry10_1::Null, None),
            ]
        );
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn na_layer_tail_reads_version_three_bitsets_and_rolls_back() {
        let mut bytes = vec![0; 2 + 16 + 8 + 4];
        push_string(&mut bytes, "");
        bytes.extend([0; 2 + 16 + 4]);
        bytes.extend(0xffff_u16.to_le_bytes());
        for value in [5_u32, 5, 5, 1] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.push(1);
        bytes.extend(0b10101_u32.to_le_bytes());
        bytes.extend([0; 17]);
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_layer_tail(&mut reader, 128, &mut refs, 3).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let tail = read_na_layer_tail(&mut reader, 128, &mut refs, 3).unwrap();
        assert_eq!(tail.version_two_flag, Some(0xffff));
        let [first, second] = tail.bitsets.unwrap();
        assert_eq!(first.words, [0b10101]);
        assert!(second.words.is_empty());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn na_children_keep_confirmed_references_and_roll_back_unknown_child() {
        let class_id = super::super::feature_layer::FEATURE_LAYER_CLASS_ID;
        let mut bytes = 2_i32.to_le_bytes().to_vec();
        bytes.extend(class_id);
        bytes.extend(258_u32.to_le_bytes());
        let second = bytes.len();
        bytes.extend([0xa5; 16]);
        bytes.extend(259_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(258);
        let children = read_na_layer_children(&mut reader, 128, &mut refs).unwrap();
        assert_eq!(children.declared_count, 2);
        assert_eq!(
            children.entries,
            [(ObjectEntry10_1::BackReference { class_id, key: 258 }, None)]
        );
        assert!(children.stop.is_some());
        assert_eq!(reader.position(), second);
        assert!(!refs.contains(259));

        let mut reader = Reader::new(&bytes[..second]);
        assert!(read_na_layer_children(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn named_set_resolves_prior_class_key_and_rolls_back_on_truncation() {
        let mut bytes = NA_NAMED_SET_CLASS_ID.to_vec();
        bytes.extend(254_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        push_string(&mut bytes, "Stops");
        bytes.extend(super::super::na_class::NA_CLASS_CLASS_ID);
        bytes.extend(178_u32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        refs.register(178);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_named_set(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(254));
        }
        let mut reader = Reader::new(&bytes);
        let (_, body) = read_na_named_set(&mut reader, 128, &mut refs).unwrap();
        let body = body.unwrap();
        assert_eq!(body.items[0].name, "Stops");
        assert!(matches!(
            body.items[0].entry,
            ObjectEntry10_1::BackReference { key: 178, .. }
        ));
        assert!(refs.contains(254));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn named_set_reads_both_directions_agents_and_rolls_back() {
        let mut bytes = NA_NAMED_SET_CLASS_ID.to_vec();
        bytes.extend(255_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        for (class_id, key, street) in [
            (NA_STREET_DIRECTIONS_AGENT_CLASS_ID, 256_u32, true),
            (NA_COMPACT_DIRECTIONS_AGENT_CLASS_ID, 257, false),
        ] {
            push_string(&mut bytes, "Agent");
            bytes.extend(class_id);
            bytes.extend(key.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(5_i32.to_le_bytes());
            if street {
                bytes.extend((-1_i32).to_le_bytes());
            }
            push_string(&mut bytes, "TravelTime");
            if street {
                bytes.extend(9_i32.to_le_bytes());
            }
            push_string(&mut bytes, "en-US");
            bytes.extend(0_i32.to_le_bytes());
            push_string(&mut bytes, "NA Desktop");
        }
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_named_set(&mut reader, 512, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let (_, set) = read_na_named_set(&mut reader, 512, &mut refs).unwrap();
        for (item, fields) in set.unwrap().items.iter().zip([Some([-1, 9]), None]) {
            let Some(NaNamedSetObject10_1::Directions(agent)) = item.body.as_ref() else {
                panic!("应读到方向代理");
            };
            assert_eq!(agent.street_fields, fields);
            assert_eq!(agent.attribute.as_deref(), Some("TravelTime"));
        }
        assert!(refs.contains(255) && refs.contains(256) && refs.contains(257));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn locator_prefix_reads_new_spatial_reference_within_envelope() {
        use crate::objects::spatial_reference_slots::UNKNOWN_COORDINATE_SYSTEM_CLASS_ID;

        let mut spatial = UNKNOWN_COORDINATE_SYSTEM_CLASS_ID.to_vec();
        spatial.extend(252_u32.to_le_bytes());
        spatial.extend(0_u16.to_le_bytes());
        for _ in 0..4 {
            push_string(&mut spatial, "");
        }
        let mut bytes = NA_LOCATOR_CLASS_ID.to_vec();
        bytes.extend(251_u32.to_le_bytes());
        bytes.extend([0; 2 + 16 + 4 + 2]);
        let offset = bytes.len();
        bytes.extend((spatial.len() as u32).to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(&spatial);
        bytes.extend(0_i32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let (_, locator) = read_na_locator_prefix(&mut reader, 128, &mut refs).unwrap();
        assert!(matches!(
            locator.unwrap().spatial_reference,
            SpatialReferenceEntry10_1::UnknownCoordinateSystem { key: 252, .. }
        ));
        assert!(refs.contains(251) && refs.contains(252));
        assert_eq!(reader.remaining(), 0);

        bytes[offset..offset + 4].copy_from_slice(&(spatial.len() as u32 + 1).to_le_bytes());
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(read_na_locator_prefix(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn locator_prefix_registers_key_only_after_agent_count() {
        let mut bytes = NA_LOCATOR_CLASS_ID.to_vec();
        bytes.extend(251_u32.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(1.25_f64.to_le_bytes());
        bytes.extend(2.5_f64.to_le_bytes());
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend(0xffff_u16.to_le_bytes());
        bytes.extend(16_u32.to_le_bytes());
        bytes.extend([0; 20]); // 同步长度和空对象头。
        bytes.extend(2_i32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_locator_prefix(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let (_, locator) = read_na_locator_prefix(&mut reader, 128, &mut refs).unwrap();
        let locator = locator.unwrap();
        assert_eq!(locator.scalar_bits, [1.25_f64.to_bits(), 2.5_f64.to_bits()]);
        assert_eq!(locator.spatial_reference, SpatialReferenceEntry10_1::Null);
        assert_eq!(locator.agent_count, 2);
        assert!(refs.contains(251));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }

    #[test]
    fn locator_location_fields_version_gate_preserves_item_boundary() {
        for version in [1_u16, 2] {
            let mut object = NA_LOCATOR_LOCATION_FIELDS_AGENT_CLASS_ID.to_vec();
            object.extend(252_u32.to_le_bytes());
            object.extend(version.to_le_bytes());
            object.extend([0; 16]);
            if version == 2 {
                object.extend(1_u32.to_le_bytes());
                object.extend([b'X', 0, 0, 0]);
            }
            let mut bytes = (object.len() as u32).to_le_bytes().to_vec();
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(object);
            let mut refs = RefTable10_1::new();
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(read_na_locator_tail(&mut reader, 128, &mut refs, 0, 1).is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
            bytes.push(0x7f);
            let mut reader = Reader::new(&bytes);
            let tail = read_na_locator_tail(&mut reader, 128, &mut refs, 0, 1).unwrap();
            let Some(NaLocatorAgent10_1::LocationFields { fields, extra, .. }) = &tail.agents[0].1
            else {
                panic!("应读到位置字段代理");
            };
            assert!(fields.iter().all(String::is_empty));
            assert_eq!(extra.as_deref(), (version == 2).then_some("X"));
            assert!(refs.contains(252));
            assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
        }
    }

    #[test]
    fn locator_tail_reads_feature_agent_and_rolls_back_on_truncation() {
        let mut object = NA_LOCATOR_FEATURE_AGENT_CLASS_ID.to_vec();
        object.extend(252_u32.to_le_bytes());
        object.extend(2_u16.to_le_bytes());
        object.extend(4_i32.to_le_bytes());
        object.extend(0_u16.to_le_bytes());
        object.extend((-1_i32).to_le_bytes());
        object.extend(1_u32.to_le_bytes());
        object.extend([b'X', 0, 0, 0]);
        object.extend(0_u32.to_le_bytes());
        let mut bytes = (object.len() as u32).to_le_bytes().to_vec();
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(object);
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.extend(9_u16.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_locator_tail(&mut reader, 128, &mut refs, 3, 1).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(252));
        }
        let mut reader = Reader::new(&bytes);
        let tail = read_na_locator_tail(&mut reader, 128, &mut refs, 3, 1).unwrap();
        let Some(NaLocatorAgent10_1::Feature(agent)) = tail.agents[0].1.as_ref() else {
            panic!("应读到定位器要素代理");
        };
        assert_eq!(agent.source, "X");
        assert_eq!(agent.extra.as_deref(), Some(""));
        assert_eq!(tail.distance_bits, Some(1.5_f64.to_bits()));
        assert_eq!(tail.flag, Some(9));
        assert!(refs.contains(252));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn location_allocation_tail_uses_native_version_mapping() {
        for version in 0..=4 {
            let mut bytes = vec![0; 4 + 16];
            if version >= 2 {
                bytes.extend([0; 12]);
            }
            bytes.extend([0; 16 + 24]);
            bytes.extend(42_i32.to_le_bytes());
            if version >= 3 {
                bytes.extend(1.5_f64.to_le_bytes());
                bytes.extend((-7_i32).to_le_bytes());
            }
            if version >= 4 {
                bytes.extend(2.5_f64.to_le_bytes());
            }
            let mut refs = RefTable10_1::new();
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(
                    read_na_solver_tail(
                        &mut reader,
                        128,
                        &mut refs,
                        version,
                        NA_LOCATION_ALLOCATION_SOLVER_CLASS_ID
                    )
                    .is_err()
                );
                assert_eq!(reader.position(), 0);
            }
            let mut reader = Reader::new(&bytes);
            let tail = read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                version,
                NA_LOCATION_ALLOCATION_SOLVER_CLASS_ID,
            )
            .unwrap();
            assert_eq!(tail.common_v3.is_some(), version >= 2);
            let body = tail.location_allocation.unwrap();
            assert_eq!(body.mode, 42);
            assert_eq!(
                body.version_three_values,
                (version >= 3).then_some((1.5_f64.to_bits(), -7))
            );
            assert_eq!(
                body.version_four_bits,
                (version >= 4).then_some(2.5_f64.to_bits())
            );
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn vrp_solver_tail_maps_versions_and_checks_truncation() {
        for version in 0..=2 {
            let mut bytes = vec![0; 4 + 16];
            if version == 2 {
                bytes.extend([0; 12]);
            }
            bytes.extend([0; 4 + 2 + 8 + 8 + 4 + 32]);
            bytes.extend(11_i32.to_le_bytes());
            if version != 0 {
                bytes.extend([0; 8]);
            }
            let mut refs = RefTable10_1::new();
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(
                    read_na_solver_tail(
                        &mut reader,
                        128,
                        &mut refs,
                        version,
                        NA_VRP_SOLVER_CLASS_ID
                    )
                    .is_err()
                );
                assert_eq!(reader.position(), 0);
            }
            let mut reader = Reader::new(&bytes);
            let tail =
                read_na_solver_tail(&mut reader, 128, &mut refs, version, NA_VRP_SOLVER_CLASS_ID)
                    .unwrap();
            assert_eq!(tail.common_v3.is_some(), version == 2);
            let vrp = tail.vrp.unwrap();
            assert_eq!(vrp.final_mode, 11);
            assert_eq!(
                vrp.extra,
                (version != 0).then_some((ObjectEntry10_1::Null, None))
            );
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn od_cost_matrix_tail_maps_outer_versions_and_checks_truncation() {
        for version in 2..=5 {
            let mut bytes = vec![0; 4 + 16];
            if version >= 4 {
                bytes.extend([0; 12]);
            }
            if version >= 3 {
                bytes.extend((-3_i32).to_le_bytes());
                bytes.extend(7_u16.to_le_bytes());
            }
            if version == 5 {
                bytes.extend(1.5_f64.to_le_bytes());
                bytes.extend(9_i32.to_le_bytes());
            }
            let mut refs = RefTable10_1::new();
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(
                    read_na_solver_tail(
                        &mut reader,
                        128,
                        &mut refs,
                        version,
                        NA_OD_COST_MATRIX_SOLVER_CLASS_ID
                    )
                    .is_err()
                );
                assert_eq!(reader.position(), 0);
            }
            let mut reader = Reader::new(&bytes);
            let tail = read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                version,
                NA_OD_COST_MATRIX_SOLVER_CLASS_ID,
            )
            .unwrap();
            assert_eq!(tail.common_v3.is_some(), version >= 4);
            assert_eq!(tail.od_cost_matrix_v3, (version >= 3).then_some((-3, 7)));
            assert_eq!(
                tail.od_cost_matrix_v5,
                (version == 5).then_some((1.5_f64.to_bits(), 9))
            );
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn service_area_tail_respects_version_gates() {
        let mut bytes = vec![0; 4 + 16 + 12];
        let common_end = bytes.len();
        bytes.extend([0; 18]);
        bytes.extend(9_u16.to_le_bytes());
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.extend(7_i32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for version in [3, 4] {
            let mut reader = Reader::new(&bytes);
            let tail = read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                version,
                NA_SERVICE_AREA_SOLVER_CLASS_ID,
            )
            .unwrap();
            let area = tail.service_area_specific.unwrap();
            assert_eq!(area.version_two_flag, Some(9));
            assert_eq!(
                area.version_four_values,
                (version == 4).then_some((1.5_f64.to_bits(), 7))
            );
            assert_eq!(
                reader.position(),
                common_end + 20 + if version == 4 { 12 } else { 0 }
            );
        }
        let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
        assert!(
            read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                4,
                NA_SERVICE_AREA_SOLVER_CLASS_ID
            )
            .is_err()
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn closest_facility_adds_its_own_tail_only_at_version_four() {
        let mut bytes = vec![0; 4 + 16 + 12];
        let shared_end = bytes.len();
        bytes.extend(1.5_f64.to_le_bytes());
        bytes.extend((-7_i32).to_le_bytes());
        let mut refs = RefTable10_1::new();
        for version in [3, 4] {
            let mut reader = Reader::new(&bytes);
            let tail = read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                version,
                NA_CLOSEST_FACILITY_SOLVER_CLASS_ID,
            )
            .unwrap();
            assert!(tail.route_specific.is_none());
            assert_eq!(
                tail.closest_facility_specific,
                (version == 4).then_some((1.5_f64.to_bits(), -7))
            );
            assert_eq!(
                reader.position(),
                if version == 4 {
                    bytes.len()
                } else {
                    shared_end
                }
            );
        }
        let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
        assert!(
            read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                4,
                NA_CLOSEST_FACILITY_SOLVER_CLASS_ID
            )
            .is_err()
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn solver_parameters_read_new_item_and_reference_transactionally() {
        let mut bytes = vec![0; 4];
        bytes.extend(NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID);
        bytes.extend(249_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID);
        bytes.extend(250_u32.to_le_bytes());
        bytes.extend([0; 2 + 8 + 4 + 16]); // 参数版本、两个空 BSTR、LONG 和 VT_EMPTY。
        bytes.extend(NA_ATTRIBUTE_PARAMETER_VALUE_CLASS_ID);
        bytes.extend(250_u32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(
                read_na_solver_tail(
                    &mut reader,
                    256,
                    &mut refs,
                    2,
                    NA_CLOSEST_FACILITY_SOLVER_CLASS_ID
                )
                .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let tail = read_na_solver_tail(
            &mut reader,
            256,
            &mut refs,
            2,
            NA_CLOSEST_FACILITY_SOLVER_CLASS_ID,
        )
        .unwrap();
        let items = tail.parameters.unwrap().1.unwrap().items;
        assert!(items[0].1.is_some());
        assert!(matches!(
            items[1].0,
            ObjectEntry10_1::BackReference { key: 250, .. }
        ));
        assert!(refs.contains(249) && refs.contains(250));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn route_solver_prefix_commits_only_after_the_flag() {
        let mut bytes = NA_ROUTE_SOLVER_CLASS_ID.to_vec();
        bytes.extend(246_u32.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([b'T', 0, 0, 0]);
        bytes.extend(0xffff_u16.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_solver_prefix(&mut reader, 64, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let (_, solver) = read_na_solver_prefix(&mut reader, 64, &mut refs).unwrap();
        let solver = solver.unwrap();
        assert_eq!(
            (
                solver.version,
                solver.attribute_name.as_str(),
                solver.extra_collection_flag
            ),
            (3, "T", 0xffff)
        );
        assert!(refs.contains(246));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }

    #[test]
    fn route_tail_reads_versioned_fields_and_rolls_back() {
        for version in 0..=3 {
            let mut bytes = 11_u16.to_le_bytes().to_vec();
            if version >= 1 {
                bytes.extend(12_u16.to_le_bytes());
            }
            if version >= 2 {
                bytes.extend(NA_ATTRIBUTE_PARAMETER_VALUE_ARRAY_CLASS_ID);
                bytes.extend(249_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
            }
            if version == 3 {
                bytes.extend(1.25_f64.to_le_bytes());
                bytes.extend((-4_i32).to_le_bytes());
            }
            if version != 0 {
                bytes.extend(2.5_f64.to_le_bytes());
                bytes.extend(13_u16.to_le_bytes());
            }
            let mut refs = RefTable10_1::new();
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                assert!(
                    read_na_solver_tail(
                        &mut reader,
                        128,
                        &mut refs,
                        version,
                        NA_ROUTE_SOLVER_CLASS_ID
                    )
                    .is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
            bytes.push(0x7f);
            let mut reader = Reader::new(&bytes);
            let tail = read_na_solver_tail(
                &mut reader,
                128,
                &mut refs,
                version,
                NA_ROUTE_SOLVER_CLASS_ID,
            )
            .unwrap();
            assert_eq!(tail.shorts, (11, (version >= 1).then_some(12)));
            assert_eq!(
                tail.common_v3,
                (version == 3).then_some((1.25_f64.to_bits(), -4))
            );
            assert_eq!(
                tail.route_specific,
                (version != 0).then_some((2.5_f64.to_bits(), 13))
            );
            assert_eq!(refs.contains(249), version >= 2);
            assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
        }
    }

    #[test]
    fn route_collections_read_both_flag_forms_and_roll_back_on_truncation() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(0_u32.to_le_bytes());
        let attributes_at = bytes.len();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend([b'X', 0, 0, 0]);
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_route_collections(&mut reader, 64, 0xffff).is_err());
            assert_eq!(reader.position(), 0);
        }

        let mut reader = Reader::new(&bytes);
        let collections = read_na_route_collections(&mut reader, 64, 0xffff).unwrap();
        assert_eq!(collections.extra, Some((1, Vec::new())));
        assert_eq!(collections.attributes, (1, vec![String::from("X")]));
        assert_eq!(reader.remaining(), 0);

        let mut reader = Reader::new(&bytes[attributes_at..]);
        let collections = read_na_route_collections(&mut reader, 64, 0).unwrap();
        assert!(collections.extra.is_none());
        assert_eq!(collections.attributes, (1, vec![String::from("X")]));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn route_string_slot_honors_boundary_and_rolls_back_keys() {
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;

        let mut bytes = 26_u32.to_le_bytes().to_vec();
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(247_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());

        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_route_string_slot(&mut reader, 64, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut oversized = bytes.clone();
        oversized[..4].copy_from_slice(&27_u32.to_le_bytes());
        oversized.push(0x7f);
        let mut reader = Reader::new(&oversized);
        assert!(read_na_route_string_slot(&mut reader, 64, &mut refs).is_err());
        assert!(refs.is_empty());

        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let slot = read_na_route_string_slot(&mut reader, 64, &mut refs).unwrap();
        assert_eq!(slot.body, Some((1, Vec::new())));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
        assert!(refs.contains(247));

        let mut reference = 0_u32.to_le_bytes().to_vec();
        reference.extend(20_u32.to_le_bytes());
        reference.extend(STR_ARRAY_CLASS_ID);
        reference.extend(247_u32.to_le_bytes());
        let mut reader = Reader::new(&reference);
        let slot = read_na_route_string_slot(&mut reader, 64, &mut refs).unwrap();
        assert!(matches!(
            slot.entry,
            ObjectEntry10_1::BackReference { key: 247, .. }
        ));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn route_hierarchy_checks_both_counts_and_each_index() {
        let mut bytes = 0xffff_u16.to_le_bytes().to_vec();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes()); // 空 BSTR。
        bytes.extend(2_i32.to_le_bytes());
        for values in [[2_i32, i32::MAX], [9, 1]] {
            bytes.extend(2_i32.to_le_bytes());
            for (index, value) in (1_i32..).zip(values) {
                bytes.extend(index.to_le_bytes());
                bytes.extend(value.to_le_bytes());
            }
        }

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_route_hierarchy(&mut reader, 128).is_err());
            assert_eq!(reader.position(), 0);
        }
        for offset in [14, 18, 34, 38] {
            let mut invalid = bytes.clone();
            invalid[offset] = 0;
            let mut reader = Reader::new(&invalid);
            assert!(read_na_route_hierarchy(&mut reader, 128).is_err());
            assert_eq!(reader.position(), 0);
        }

        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let hierarchy = read_na_route_hierarchy(&mut reader, 128).unwrap();
        assert_eq!(hierarchy.declared_count, 2);
        assert_eq!(hierarchy.indexed_values, [vec![2, i32::MAX], vec![9, 1]]);
        assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    }
}
