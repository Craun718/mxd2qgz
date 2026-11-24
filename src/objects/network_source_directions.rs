//! GdbNetwork.dll 10.1 的网络源方向对象，见 docs/mxd-terra-round9-bookmark.md。

use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const NETWORK_SOURCE_DIRECTIONS_CLASS_ID: [u8; 16] = [
    0x5c, 0x82, 0xe5, 0x29, 0xcd, 0x6b, 0x63, 0x40, 0xa6, 0xda, 0xf9, 0x43, 0xe8, 0xb2, 0x62, 0xbd,
];
pub const SHIELDS_CLASS_ID: [u8; 16] = [
    0xb4, 0x46, 0xd6, 0xf2, 0xd6, 0xf0, 0xb5, 0x40, 0xa7, 0x98, 0x82, 0x3c, 0x1b, 0xfa, 0x4e, 0x99,
];
pub const STREET_NAME_FIELDS_CLASS_ID: [u8; 16] = [
    0x23, 0x28, 0x4b, 0xfe, 0x7b, 0x40, 0x8e, 0x41, 0xab, 0x25, 0x40, 0xd8, 0x57, 0x67, 0xea, 0x24,
];
pub const DIRECTIONS_FIELD_MAPPING_CLASS_ID: [u8; 16] = [
    0x55, 0xdf, 0xaa, 0x1a, 0x30, 0x75, 0xa5, 0x4c, 0x9d, 0x41, 0x90, 0xff, 0x94, 0xc9, 0x1c, 0x5a,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectionSlot10_1<T> {
    pub entry: ObjectEntry10_1,
    pub body: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSourceDirections10_1 {
    pub version: u16,
    pub name: String,
    pub shields_flag: u32,
    pub shields: Option<DirectionSlot10_1<Shields10_1>>,
    pub street_names: (i32, Vec<DirectionSlot10_1<StreetNameFields10_1>>),
    pub field_mappings: Option<(i32, Vec<DirectionSlot10_1<[String; 2]>>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shields10_1 {
    pub version: i16,
    pub names: [String; 3],
    pub flag: u32,
    pub lists: [(i32, Vec<String>); 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreetNameFields10_1 {
    pub version: u16,
    pub names: [String; 5],
    pub value: u32,
    pub extra_names: Vec<String>,
}

fn version_error(class: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, format!("{class} 版本超限"))
}

fn read_list<T>(
    reader: &mut Reader<'_>,
    min_size: usize,
    max_bytes: usize,
    mut read: impl FnMut(&mut Reader<'_>) -> io::Result<T>,
) -> io::Result<(i32, Vec<T>)> {
    let raw_count = reader.read_u32_le()? as i32;
    let count = raw_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / min_size {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "网络方向列表数量超限",
        ));
    }
    let mut values = Vec::new();
    for _ in 0..count {
        values.push(read(reader)?);
    }
    Ok((raw_count, values))
}

fn read_slot<T>(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    expected: [u8; 16],
    read: impl FnOnce(&mut Reader<'_>) -> io::Result<T>,
) -> io::Result<DirectionSlot10_1<T>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == expected =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected => Some(read(reader)?),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "网络方向槽包含未知对象类",
            ));
        }
    };
    Ok(DirectionSlot10_1 { entry, body })
}

fn read_shields(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<Shields10_1> {
    let version = reader.read_u16_le()? as i16;
    if version > 1 {
        return Err(version_error("Shields"));
    }
    let names = [
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
    ];
    let flag = reader.read_u32_le()?;
    let mut read_names = || read_list(reader, 6, max_bytes, |r| read_string_10_1(r, max_bytes));
    let lists = [read_names()?, read_names()?];
    Ok(Shields10_1 {
        version,
        names,
        flag,
        lists,
    })
}

fn read_street_names(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<StreetNameFields10_1> {
    let version = reader.read_u16_le()?;
    if !(1..=3).contains(&version) {
        return Err(version_error("StreetNameFields"));
    }
    let names = [
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
    ];
    let value = reader.read_u32_le()?;
    let mut extra_names = Vec::new();
    let extra_count = [0, 1, 3][usize::from(version - 1)];
    for _ in 0..extra_count {
        extra_names.push(read_string_10_1(reader, max_bytes)?);
    }
    Ok(StreetNameFields10_1 {
        version,
        names,
        value,
        extra_names,
    })
}

/// 读取对象体。宿主负责在失败时回退游标及引用表。
pub fn read_network_source_directions_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NetworkSourceDirections10_1> {
    let version = reader.read_u16_le()?;
    if version > 2 {
        return Err(version_error("NetworkSourceDirections"));
    }
    let name = read_string_10_1(reader, max_bytes)?;
    let shields_flag = reader.read_u32_le()?;
    let shields = (shields_flag != 0)
        .then(|| {
            read_slot(reader, refs, SHIELDS_CLASS_ID, |r| {
                read_shields(r, max_bytes)
            })
        })
        .transpose()?;
    let street_names = read_list(reader, 16, max_bytes, |r| {
        read_slot(r, refs, STREET_NAME_FIELDS_CLASS_ID, |r| {
            read_street_names(r, max_bytes)
        })
    })?;
    let field_mappings = (version == 2)
        .then(|| {
            read_list(reader, 16, max_bytes, |r| {
                read_slot(r, refs, DIRECTIONS_FIELD_MAPPING_CLASS_ID, |r| {
                    if r.read_u16_le()? != 1 {
                        return Err(version_error("DirectionsFieldMapping"));
                    }
                    Ok([
                        read_string_10_1(r, max_bytes)?,
                        read_string_10_1(r, max_bytes)?,
                    ])
                })
            })
        })
        .transpose()?;
    Ok(NetworkSourceDirections10_1 {
        version,
        name,
        shields_flag,
        shields,
        street_names,
        field_mappings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: [u8; 6] = [2, 0, 0, 0, 0, 0];

    fn object(bytes: &mut Vec<u8>, class: [u8; 16], key: u32) {
        bytes.extend(class);
        bytes.extend(key.to_le_bytes());
    }

    fn read(bytes: &[u8]) -> io::Result<(NetworkSourceDirections10_1, usize)> {
        let mut reader = Reader::new(bytes);
        let body = read_network_source_directions_body_10_1(
            &mut reader,
            bytes.len(),
            &mut RefTable10_1::new(),
        )?;
        Ok((body, reader.position()))
    }

    #[test]
    fn network_source_directions_reads_children_and_references() {
        let mut bytes = vec![2, 0];
        bytes.extend(EMPTY);
        bytes.extend(1_u32.to_le_bytes());
        object(&mut bytes, SHIELDS_CLASS_ID, 1);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(EMPTY.repeat(3));
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(EMPTY);
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        object(&mut bytes, STREET_NAME_FIELDS_CLASS_ID, 2);
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend(EMPTY.repeat(5));
        bytes.extend(39_u32.to_le_bytes());
        bytes.extend(EMPTY.repeat(3));
        object(&mut bytes, STREET_NAME_FIELDS_CLASS_ID, 2);
        bytes.extend(1_u32.to_le_bytes());
        object(&mut bytes, DIRECTIONS_FIELD_MAPPING_CLASS_ID, 3);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(EMPTY.repeat(2));

        let end = bytes.len();
        bytes.push(0xaa);
        let (body, position) = read(&bytes).unwrap();
        assert_eq!(position, end);
        assert_eq!(body.shields.unwrap().body.unwrap().lists[1], (-1, vec![]));
        assert_eq!(
            body.street_names.1[0]
                .body
                .as_ref()
                .unwrap()
                .extra_names
                .len(),
            3
        );
        assert!(matches!(
            body.street_names.1[1].entry,
            ObjectEntry10_1::BackReference { key: 2, .. }
        ));
        assert!(body.field_mappings.unwrap().1[0].body.is_some());
        for end in 0..end {
            assert!(read(&bytes[..end]).is_err(), "截断位置 {end}");
        }
    }

    #[test]
    fn network_source_directions_checks_versions_counts_and_classes() {
        for version in [0, 1] {
            let mut bytes = vec![version, 0];
            bytes.extend(EMPTY);
            bytes.extend([0; 8]);
            assert!(read(&bytes).unwrap().0.field_mappings.is_none());
            bytes[12..16].copy_from_slice(&i32::MAX.to_le_bytes());
            assert!(read(&bytes).is_err());
        }
        assert_eq!(read(&[3, 0]).unwrap_err().kind(), ErrorKind::InvalidData);

        let mut bytes = vec![1, 0];
        bytes.extend(EMPTY);
        bytes.extend(1_u32.to_le_bytes());
        object(&mut bytes, STREET_NAME_FIELDS_CLASS_ID, 1);
        assert_eq!(read(&bytes).unwrap_err().kind(), ErrorKind::Unsupported);
    }
}
