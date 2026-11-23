//! SceneFilters.dll 10.1 四类 3DProperties 共用的 IPersistStream 对象体。

use super::raster_dataset_name::{
    MAX_NAME_NESTING, RasterDatasetWorkspaceSlot10_1, RasterNameSlot10_1, read_name_slot,
    read_workspace_slot,
};
use super::raster_layer_tail::{RasterLayerEnvelopeSlot10_1, read_envelope_slot};
use crate::objects::envelope::ENVELOPE_CLASS_ID;
use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io;

pub const BASIC_3D_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x61, 0xdc, 0xa8, 0xa5, 0x8c, 0x26, 0xd4, 0x11, 0x91, 0x4a, 0, 0x80, 0xc7, 0x07, 0x43, 0x26,
];
pub const TIN_3D_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x5d, 0xa2, 0xa0, 0xb7, 0xf7, 0x24, 0xd3, 0x11, 0x9f, 0x4d, 0, 0xc0, 0x4f, 0x6b, 0xc5, 0xf4,
];
pub const RASTER_3D_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xe3, 0xec, 0x4c, 0x4a, 0x3a, 0xf5, 0xd3, 0x11, 0x80, 0xd9, 0, 0xc0, 0x4f, 0xa0, 0xad, 0xf8,
];
pub const FEATURE_3D_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xe4, 0xec, 0x4c, 0x4a, 0x3a, 0xf5, 0xd3, 0x11, 0x80, 0xd9, 0, 0xc0, 0x4f, 0xa0, 0xad, 0xf8,
];
pub const TIN_NAME_CLASS_ID: [u8; 16] = [
    0x16, 0xf4, 0x13, 0x5f, 0xbc, 0x42, 0xd2, 0x11, 0xa5, 0x69, 0, 0, 0xf8, 0x77, 0x4f, 0x0f,
];

pub(super) fn is_scene_layer_properties(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        BASIC_3D_PROPERTIES_CLASS_ID
            | TIN_3D_PROPERTIES_CLASS_ID
            | RASTER_3D_PROPERTIES_CLASS_ID
            | FEATURE_3D_PROPERTIES_CLASS_ID
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct TinName10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub workspace: RasterDatasetWorkspaceSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceName10_1 {
    Raster(RasterNameSlot10_1),
    Tin {
        entry: ObjectEntry10_1,
        body: Option<Box<TinName10_1>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneLayerProperties10_1 {
    pub version: u16,
    pub surface_mode: i32,
    pub surface_expression: Option<String>,
    pub surface_name: Option<SurfaceName10_1>,
    pub surface_path: Option<String>,
    pub offset_expression: String,
    pub extrusion_expression: String,
    /// 顺序保留各段标量的磁盘字节，字符串和对象槽单独读取。
    pub scalars_before_offset: Vec<u8>,
    pub scalars_before_extrusion: [u8; 4],
    pub scalars_after_extrusion: Vec<u8>,
    pub envelope: Option<RasterLayerEnvelopeSlot10_1>,
}

pub(super) fn read_surface_name(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<SurfaceName10_1> {
    if reader.clone().read_bytes(16)? != TIN_NAME_CLASS_ID {
        return read_name_slot(reader, max, refs, MAX_NAME_NESTING).map(SurfaceName10_1::Raster);
    }

    let entry = read_object_entry_10_1(reader, refs)?;
    let body = if matches!(entry, ObjectEntry10_1::NewObject { .. }) {
        Some(Box::new(read_tin_name_body_10_1(reader, max, refs)?))
    } else {
        None
    };
    Ok(SurfaceName10_1::Tin { entry, body })
}

pub(crate) fn read_tin_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TinName10_1> {
    // TinDB.dll TinName::Load 使用有符号 short 上限 1。
    let version = reader.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TinName 版本 {version} 超出 10.1 上限"),
        ));
    }
    let strings = [
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
    ];
    let workspace = read_surface_workspace_name_10_1(reader, max, refs, "TinName")?;
    Ok(TinName10_1 {
        version,
        strings,
        workspace,
    })
}

pub(crate) fn read_surface_workspace_name_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    name_class: &str,
) -> io::Result<RasterDatasetWorkspaceSlot10_1> {
    let workspace = read_workspace_slot(reader, max, refs)?;
    match &workspace {
        RasterDatasetWorkspaceSlot10_1::WorkspaceName(body) if body.stop.is_some() => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{name_class} 工作空间体未完整读取"),
            ));
        }
        RasterDatasetWorkspaceSlot10_1::Unsupported { .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{name_class} 工作空间类未支持"),
            ));
        }
        RasterDatasetWorkspaceSlot10_1::BackReference { class_id, .. }
            if *class_id != WORKSPACE_NAME_CLASS_ID =>
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("{name_class} 工作空间回引类未支持"),
            ));
        }
        _ => {}
    }
    Ok(workspace)
}

pub(super) fn read_bounded_scene_layer_properties(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<SceneLayerProperties10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let version = body.read_u16_le()?;
    if !(1..=13).contains(&version) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("3DProperties 版本 {version} 超出 10.1 范围"),
        ));
    }

    let surface_mode = body.read_u32_le()? as i32;
    let surface_expression = (version == 1 || surface_mode == 0)
        .then(|| read_string_10_1(&mut body, max))
        .transpose()?;
    let surface_name = (version >= 2 && surface_mode == 1)
        .then(|| read_surface_name(&mut body, max, &mut staged))
        .transpose()?;
    let surface_path = (version >= 4 && surface_name.is_some())
        .then(|| read_string_10_1(&mut body, max))
        .transpose()?;

    let start = body.position();
    if version >= 5 {
        body.read_bytes(4)?;
    }
    if version >= 3 {
        body.read_bytes(8)?;
    }
    body.read_bytes(8)?;
    let scalars_before_offset = bytes[start..body.position()].to_vec();
    let offset_expression = read_string_10_1(&mut body, max)?;
    let scalars_before_extrusion = body.read_bytes(4)?.try_into().unwrap();
    let extrusion_expression = read_string_10_1(&mut body, max)?;

    let start = body.position();
    body.read_bytes(4 + 8 + 2 + 4)?;
    if version >= 3 {
        body.read_bytes(4)?;
    }
    if version >= 6 {
        body.read_bytes(4)?;
    }
    if version >= 7 {
        body.read_bytes(8)?;
    }
    if version >= 8 {
        body.read_bytes(4)?;
    }
    if version >= 9 {
        body.read_bytes(4)?;
    }
    if version >= 10 {
        body.read_bytes(8)?;
    }
    if version >= 11 {
        body.read_bytes(48)?;
    }
    if version >= 12 {
        body.read_bytes(8)?;
    }
    let scalars_after_extrusion = bytes[start..body.position()].to_vec();

    let envelope = (version >= 13)
        .then(|| read_envelope_slot(&mut body, max, &mut staged))
        .transpose()?;
    if let Some(RasterLayerEnvelopeSlot10_1 {
        entry: ObjectEntry10_1::BackReference { class_id, .. },
        ..
    }) = &envelope
        && *class_id != ENVELOPE_CLASS_ID
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "3DProperties 包络回引类未支持",
        ));
    }
    if body.remaining() != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "3DProperties 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(SceneLayerProperties10_1 {
        version,
        surface_mode,
        surface_expression,
        surface_name,
        surface_path,
        offset_expression,
        extrusion_expression,
        scalars_before_offset,
        scalars_before_extrusion,
        scalars_after_extrusion,
        envelope,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(bytes: &mut Vec<u8>, value: &str) {
        let utf16: Vec<_> = value
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        bytes.extend((utf16.len() as u32).to_le_bytes());
        bytes.extend(utf16);
    }

    fn append_display_fields(bytes: &mut Vec<u8>, version: u16) {
        let before = 8 + 4 * usize::from(version >= 5) + 8 * usize::from(version >= 3);
        bytes.extend(vec![0x31; before]);
        text(bytes, "偏移");
        bytes.extend([0x32; 4]);
        text(bytes, "拉伸");
        let after = 18
            + [
                (3, 4),
                (6, 4),
                (7, 8),
                (8, 4),
                (9, 4),
                (10, 8),
                (11, 48),
                (12, 8),
            ]
            .into_iter()
            .filter(|(since, _)| version >= *since)
            .map(|(_, width)| width)
            .sum::<usize>();
        bytes.extend(vec![0x33; after]);
    }

    #[test]
    fn scene_properties_reads_versions_and_source_selectors() {
        for version in 1_u16..=13 {
            for mode in [0_i32, 1, 2] {
                let mut bytes = version.to_le_bytes().to_vec();
                bytes.extend(mode.to_le_bytes());
                if version == 1 || mode == 0 {
                    text(&mut bytes, "来源");
                } else if mode == 1 {
                    bytes.extend([0; 16]);
                    if version >= 4 {
                        text(&mut bytes, "路径");
                    }
                }
                append_display_fields(&mut bytes, version);
                if version == 13 {
                    bytes.extend([0; 16]);
                }

                let mut refs = RefTable10_1::new();
                let value = read_bounded_scene_layer_properties(&bytes, 4096, &mut refs).unwrap();
                assert_eq!(value.version, version);
                assert_eq!(value.surface_mode, mode);
                assert_eq!(
                    value.surface_expression.as_deref(),
                    (version == 1 || mode == 0).then_some("来源")
                );
                assert_eq!(
                    value.surface_path.as_deref(),
                    (version >= 4 && mode == 1).then_some("路径")
                );
                assert_eq!(value.offset_expression, "偏移");
                assert_eq!(value.extrusion_expression, "拉伸");
                assert_eq!(value.envelope.is_some(), version == 13);
                for end in 0..bytes.len() {
                    assert!(
                        read_bounded_scene_layer_properties(&bytes[..end], 4096, &mut refs)
                            .is_err()
                    );
                }
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn scene_properties_reads_nested_tin_workspace_and_envelope_keys() {
        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(TIN_NAME_CLASS_ID);
        bytes.extend(81_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["tin", "b", "c"] {
            text(&mut bytes, value);
        }
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(82_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["目录", "", "文件系统"] {
            text(&mut bytes, value);
        }
        bytes.extend([0; 16 + 1 + 4]);
        text(&mut bytes, "地表路径");
        append_display_fields(&mut bytes, 13);
        bytes.extend(ENVELOPE_CLASS_ID);
        bytes.extend(83_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 32]);

        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            assert!(read_bounded_scene_layer_properties(&bytes[..end], 4096, &mut refs).is_err());
            assert!(refs.is_empty());
        }
        let value = read_bounded_scene_layer_properties(&bytes, 4096, &mut refs).unwrap();
        assert!(matches!(
            value.surface_name,
            Some(SurfaceName10_1::Tin { entry: ObjectEntry10_1::NewObject { key: 81, .. }, body: Some(body) })
                if body.strings[0] == "tin"
                   && matches!(&body.workspace, RasterDatasetWorkspaceSlot10_1::WorkspaceName(workspace) if workspace.path == "目录")
        ));
        assert!(matches!(
            value.envelope.unwrap().entry,
            ObjectEntry10_1::NewObject { key: 83, .. }
        ));
        assert!(refs.contains(81) && refs.contains(82) && refs.contains(83));
    }
}
