//! 已确认的 NetworkAnalyst.dll 10.1 NALayer 持久化前缀。

use crate::objects::dataset_name::{
    FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1, read_feature_dataset_name_body_10_1,
};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
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
}
