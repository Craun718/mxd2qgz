//! 已确认的 NetworkAnalyst.dll 10.1 NALayer 持久化前缀。

use crate::objects::dataset_name::{
    FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1, read_feature_dataset_name_body_10_1,
};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
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
pub struct NaLayerPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub shorts: [u16; 3],
    pub double_bits: [u64; 2],
    pub network_name: Option<(ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>)>,
    pub context: Option<(ObjectEntry10_1, Option<Box<NaContextPrefix10_1>>)>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaContextPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub workspace: ObjectEntry10_1,
    pub workspace_prefix: Option<Box<NetworkAnalysisWorkspacePrefix10_1>>,
    pub dataset: Option<(ObjectEntry10_1, Option<Box<NetworkDatasetName10_1>>)>,
    pub solver: Option<(ObjectEntry10_1, Option<NaRouteSolverPrefix10_1>)>,
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
/// 再读取四个字节的命名项计数。本函数目前停在计数之前，后续 NAClass
/// 条目仍由外层有界未知区保留。
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
            let solver = read_na_route_solver_prefix(&mut trial, max, &mut keys).ok();
            Some(Box::new(NaContextPrefix10_1 {
                version,
                name,
                workspace,
                workspace_prefix,
                dataset: Some(dataset),
                solver,
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

const NA_ROUTE_SOLVER_CLASS_ID: [u8; 16] = [
    0x08, 0xa2, 0x8a, 0x1a, 0x4a, 0x3d, 0x0f, 0x42, 0xb8, 0x2c, 0xf5, 0x81, 0x94, 0xa7, 0x2c, 0xb0,
];

fn read_na_route_solver_prefix(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<NaRouteSolverPrefix10_1>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::NewObject {
            class_id: NA_ROUTE_SOLVER_CLASS_ID,
            ..
        } => {
            let version = trial.read_u16_le()?;
            if version as i16 > 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NARouteSolver 版本超过 3",
                ));
            }
            let attribute_name = read_bstr_string_10_1(&mut trial, max)?;
            let extra_collection_flag = trial.read_u16_le()?;
            Some(NaRouteSolverPrefix10_1 {
                version,
                attribute_name,
                extra_collection_flag,
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
        fn push_string(bytes: &mut Vec<u8>, value: &str) {
            let utf16 = value.encode_utf16().collect::<Vec<_>>();
            let cb = u32::try_from((utf16.len() + 1) * 2).unwrap();
            bytes.extend(cb.to_le_bytes());
            for unit in utf16 {
                bytes.extend(unit.to_le_bytes());
            }
            bytes.extend(0_u16.to_le_bytes());
        }

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

        bytes.push(0x7f);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let (_, context) = read_na_context_prefix_10_1(&mut reader, 1024, &mut refs).unwrap();
        let context = context.unwrap();
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
            assert!(read_na_route_solver_prefix(&mut reader, 64, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        bytes.push(0x7f);
        let mut reader = Reader::new(&bytes);
        let (_, solver) = read_na_route_solver_prefix(&mut reader, 64, &mut refs).unwrap();
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
}
