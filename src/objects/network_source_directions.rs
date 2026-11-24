//! GdbNetwork.dll 10.1 的网络数据集子对象，见 docs/mxd-terra-round9-bookmark.md。

use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, PropertyValueObjectStream10_1,
    read_padded_variant_object_stream_10_1, read_property_set_object_stream_10_1,
};
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
    pub field_mappings: Option<(i32, Vec<DirectionSlot10_1<[String; 4]>>)>,
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
    read: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<DirectionSlot10_1<T>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key }
            if refs.registered_class(key).unwrap_or(class_id) == expected =>
        {
            None
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected => {
            Some(read(reader, refs)?)
        }
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
            read_slot(reader, refs, SHIELDS_CLASS_ID, |r, _| {
                read_shields(r, max_bytes)
            })
        })
        .transpose()?;
    let street_names = read_list(reader, 16, max_bytes, |r| {
        read_slot(r, refs, STREET_NAME_FIELDS_CLASS_ID, |r, _| {
            read_street_names(r, max_bytes)
        })
    })?;
    let field_mappings = (version == 2)
        .then(|| {
            read_list(reader, 16, max_bytes, |r| {
                read_slot(r, refs, DIRECTIONS_FIELD_MAPPING_CLASS_ID, |r, _| {
                    if r.read_u16_le()? != 1 {
                        return Err(version_error("DirectionsFieldMapping"));
                    }
                    Ok([
                        read_string_10_1(r, max_bytes)?,
                        read_string_10_1(r, max_bytes)?,
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

pub const NETWORK_SOURCE_CLASS_ID: [u8; 16] = [
    0x6d, 0xd0, 0xe3, 0x32, 0x92, 0xf7, 0xeb, 0x45, 0x10, 0x89, 0xa6, 0xea, 0x57, 0x23, 0x2b, 0xc0,
];
pub const EDGE_FEATURE_SOURCE_CLASS_ID: [u8; 16] = [
    0x50, 0x0d, 0x14, 0xf2, 0x26, 0x6b, 0x4d, 0x41, 0xa3, 0x9f, 0xf3, 0x5a, 0xc5, 0x4d, 0x10, 0xde,
];
pub const JUNCTION_FEATURE_SOURCE_CLASS_ID: [u8; 16] = [
    0xbd, 0x71, 0xc8, 0x74, 0x8e, 0x64, 0xc0, 0x40, 0xa4, 0x6a, 0xe3, 0x85, 0x21, 0x21, 0x6a, 0x7b,
];
pub const SYSTEM_JUNCTION_SOURCE_CLASS_ID: [u8; 16] = [
    0x1f, 0x18, 0x9f, 0x61, 0x40, 0x4a, 0x95, 0x4a, 0xbc, 0xeb, 0x5c, 0xa7, 0xc8, 0xc6, 0x3b, 0x60,
];
pub const TURN_FEATURE_SOURCE_CLASS_ID: [u8; 16] = [
    0x7b, 0x04, 0xbb, 0x44, 0x80, 0xc0, 0x97, 0x45, 0xb3, 0x6b, 0xe5, 0x69, 0x05, 0x98, 0x3c, 0x4c,
];

pub const SDC_NETWORK_SOURCE_CLASS_ID: [u8; 16] = [
    0x09, 0x0c, 0xe0, 0x5b, 0x3f, 0xa0, 0x69, 0x4a, 0xb6, 0x78, 0x98, 0x51, 0x08, 0xa5, 0x00, 0xd3,
];

#[derive(Debug, Clone, PartialEq)]
pub struct SdcNetworkSourceFields10_1 {
    pub short_value: u16,
    pub directions: DirectionSlot10_1<NetworkSourceDirections10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkSource10_1 {
    pub version: i16,
    pub initial_values: [u32; 2],
    pub name: String,
    pub value: u32,
    pub properties: DirectionSlot10_1<PropertySetObjectStream10_1>,
    pub extension: Option<NetworkSourceExtension10_1>,
    pub sdc_fields: Option<SdcNetworkSourceFields10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkSourceExtension10_1 {
    pub version: i16,
    pub directions: Option<(u32, Option<DirectionSlot10_1<NetworkSourceDirections10_1>>)>,
    pub names: Vec<(u32, Option<String>)>,
    pub properties: Option<DirectionSlot10_1<PropertySetObjectStream10_1>>,
}

fn read_source_version(reader: &mut Reader<'_>) -> io::Result<i16> {
    let version = reader.read_u16_le()? as i16;
    if version > 1 {
        return Err(version_error("NetworkSource"));
    }
    Ok(version)
}

fn read_flagged<T>(
    reader: &mut Reader<'_>,
    read: impl FnOnce(&mut Reader<'_>) -> io::Result<T>,
) -> io::Result<(u32, Option<T>)> {
    let flag = reader.read_u32_le()?;
    Ok((flag, (flag != 0).then(|| read(reader)).transpose()?))
}

/// 读取 INetworkSource 槽。宿主负责在失败时回退游标及引用表。
pub fn read_network_source_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DirectionSlot10_1<NetworkSource10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let class_id = match entry {
        ObjectEntry10_1::Null => return Ok(DirectionSlot10_1 { entry, body: None }),
        ObjectEntry10_1::BackReference { class_id, key } => {
            refs.registered_class(key).unwrap_or(class_id)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => class_id,
    };
    let name_count = match class_id {
        EDGE_FEATURE_SOURCE_CLASS_ID => 2,
        JUNCTION_FEATURE_SOURCE_CLASS_ID | SYSTEM_JUNCTION_SOURCE_CLASS_ID => 1,
        NETWORK_SOURCE_CLASS_ID | TURN_FEATURE_SOURCE_CLASS_ID | SDC_NETWORK_SOURCE_CLASS_ID => 0,
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "INetworkSource 槽包含未知对象类",
            ));
        }
    };
    if matches!(entry, ObjectEntry10_1::BackReference { .. }) {
        return Ok(DirectionSlot10_1 { entry, body: None });
    }

    let read_properties = |r: &mut Reader<'_>, refs: &mut RefTable10_1| {
        read_slot(r, refs, PROPERTY_SET_CLASS_ID, |r, refs| {
            read_property_set_object_stream_10_1(r, max_bytes, refs)
        })
    };
    let is_sdc = class_id == SDC_NETWORK_SOURCE_CLASS_ID;
    let version = read_source_version(reader)?;
    if is_sdc && version != 1 {
        return Err(version_error("SDCNetworkSource"));
    }
    let initial_values = [reader.read_u32_le()?, reader.read_u32_le()?];
    let short_value = is_sdc.then(|| reader.read_u16_le()).transpose()?;
    let sdc_value = is_sdc.then(|| reader.read_u32_le()).transpose()?;
    let name = read_string_10_1(reader, max_bytes)?;
    let value = match sdc_value {
        Some(value) => value,
        None => reader.read_u32_le()?,
    };
    let sdc_fields = short_value
        .map(|short_value| {
            let directions = read_slot(
                reader,
                refs,
                NETWORK_SOURCE_DIRECTIONS_CLASS_ID,
                |r, refs| read_network_source_directions_body_10_1(r, max_bytes, refs),
            )?;
            Ok::<_, io::Error>(SdcNetworkSourceFields10_1 {
                short_value,
                directions,
            })
        })
        .transpose()?;
    let properties = read_properties(reader, refs)?;
    let extension = (name_count != 0)
        .then(|| -> io::Result<_> {
            let version = read_source_version(reader)?;
            let feature_source = class_id != SYSTEM_JUNCTION_SOURCE_CLASS_ID;
            let directions = feature_source
                .then(|| {
                    read_flagged(reader, |r| {
                        read_slot(r, refs, NETWORK_SOURCE_DIRECTIONS_CLASS_ID, |r, refs| {
                            read_network_source_directions_body_10_1(r, max_bytes, refs)
                        })
                    })
                })
                .transpose()?;
            let mut names = Vec::new();
            for _ in 0..name_count {
                names.push(read_flagged(reader, |r| read_string_10_1(r, max_bytes))?);
            }
            let properties = feature_source
                .then(|| read_properties(reader, refs))
                .transpose()?;
            Ok(NetworkSourceExtension10_1 {
                version,
                directions,
                names,
                properties,
            })
        })
        .transpose()?;
    let body = NetworkSource10_1 {
        version,
        initial_values,
        name,
        value,
        properties,
        extension,
        sdc_fields,
    };
    Ok(DirectionSlot10_1 {
        entry,
        body: Some(body),
    })
}

pub const NETWORK_DIRECTIONS_CLASS_ID: [u8; 16] = [
    0x59, 0xbd, 0x21, 0xf9, 0xad, 0xa8, 0x25, 0x44, 0xb5, 0xa0, 0x75, 0xbc, 0xe8, 0x8d, 0x22, 0xfe,
];
pub const DIRECTIONS_ATTRIBUTE_MAPPING_CLASS_ID: [u8; 16] = [
    0x60, 0x61, 0x36, 0xd8, 0x1f, 0x0e, 0x09, 0x41, 0xa7, 0x65, 0xfa, 0xc6, 0xd3, 0x7c, 0x2e, 0xf2,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDirections10_1 {
    pub version: u16,
    pub names: [String; 3],
    pub value: u32,
    pub extra_names: Option<[String; 2]>,
    pub mappings: Option<(i32, Vec<DirectionSlot10_1<[String; 2]>>)>,
}

/// 读取 INetworkDirections 槽。宿主负责在失败时回退游标及引用表。
pub fn read_network_directions_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DirectionSlot10_1<NetworkDirections10_1>> {
    read_slot(reader, refs, NETWORK_DIRECTIONS_CLASS_ID, |reader, refs| {
        let version = reader.read_u16_le()?;
        if !(1..=3).contains(&version) {
            return Err(version_error("NetworkDirections"));
        }
        let names = [
            read_string_10_1(reader, max_bytes)?,
            read_string_10_1(reader, max_bytes)?,
            read_string_10_1(reader, max_bytes)?,
        ];
        let value = reader.read_u32_le()?;
        let extra_names = (version >= 2)
            .then(|| {
                Ok::<_, io::Error>([
                    read_string_10_1(reader, max_bytes)?,
                    read_string_10_1(reader, max_bytes)?,
                ])
            })
            .transpose()?;
        let mappings = (version == 3)
            .then(|| {
                read_list(reader, 16, max_bytes, |r| {
                    read_slot(r, refs, DIRECTIONS_ATTRIBUTE_MAPPING_CLASS_ID, |r, _| {
                        if r.read_u16_le()? != 1 {
                            return Err(version_error("DirectionsAttributeMapping"));
                        }
                        Ok([
                            read_string_10_1(r, max_bytes)?,
                            read_string_10_1(r, max_bytes)?,
                        ])
                    })
                })
            })
            .transpose()?;
        Ok(NetworkDirections10_1 {
            version,
            names,
            value,
            extra_names,
            mappings,
        })
    })
}

pub const TRAFFIC_DATA_CLASS_ID: [u8; 16] = [
    0x66, 0xcd, 0xa1, 0xe7, 0xef, 0xc6, 0x0b, 0x44, 0x91, 0xca, 0xd6, 0xb8, 0xdb, 0x9c, 0x01, 0xf9,
];
pub const TRAFFIC_FEED_DIRECTORY_CLASS_ID: [u8; 16] = [
    0xd8, 0x45, 0x4b, 0x2c, 0x88, 0x72, 0xd9, 0x4b, 0x9f, 0x7e, 0x41, 0x67, 0xde, 0xce, 0x6d, 0x34,
];
pub const TRAFFIC_FEED_GP_SERVICE_CLASS_ID: [u8; 16] = [
    0x70, 0x88, 0xa6, 0xc8, 0x69, 0x37, 0x71, 0x42, 0xb4, 0x80, 0xe9, 0x5e, 0x80, 0x6b, 0x72, 0xdf,
];

#[derive(Debug, Clone, PartialEq)]
pub enum TrafficFeedLocation10_1 {
    Directory(String),
    GpService {
        properties: DirectionSlot10_1<PropertySetObjectStream10_1>,
        names: [String; 2],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrafficData10_1 {
    pub version: u16,
    pub first_name: String,
    pub first_value: u32,
    pub double_bits: u64,
    pub names: [String; 4],
    pub second_value: u32,
    pub daily_names: [String; 7],
    pub extension: Option<TrafficDataExtension10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrafficDataExtension10_1 {
    pub initial_names: [String; 2],
    pub value: u32,
    pub final_names: [String; 2],
    pub feed: Option<(u32, Option<DirectionSlot10_1<TrafficFeedLocation10_1>>)>,
}

fn read_feed_location(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DirectionSlot10_1<TrafficFeedLocation10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let (class, is_new) = match entry {
        ObjectEntry10_1::Null => return Ok(DirectionSlot10_1 { entry, body: None }),
        ObjectEntry10_1::BackReference { class_id, key } => {
            (refs.registered_class(key).unwrap_or(class_id), false)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => (class_id, true),
    };
    if ![
        TRAFFIC_FEED_DIRECTORY_CLASS_ID,
        TRAFFIC_FEED_GP_SERVICE_CLASS_ID,
    ]
    .contains(&class)
    {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "ITrafficFeedLocation 槽包含未知对象类",
        ));
    }
    let body = is_new
        .then(|| {
            if reader.read_u16_le()? != 1 {
                return Err(version_error("TrafficFeedLocation"));
            }
            if class == TRAFFIC_FEED_DIRECTORY_CLASS_ID {
                Ok(TrafficFeedLocation10_1::Directory(read_string_10_1(
                    reader, max_bytes,
                )?))
            } else {
                let properties = read_slot(reader, refs, PROPERTY_SET_CLASS_ID, |r, refs| {
                    read_property_set_object_stream_10_1(r, max_bytes, refs)
                })?;
                let names = [
                    read_string_10_1(reader, max_bytes)?,
                    read_string_10_1(reader, max_bytes)?,
                ];
                Ok(TrafficFeedLocation10_1::GpService { properties, names })
            }
        })
        .transpose()?;
    Ok(DirectionSlot10_1 { entry, body })
}

/// 读取 ITrafficData 槽。宿主负责在失败时回退游标及引用表。
pub fn read_traffic_data_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DirectionSlot10_1<TrafficData10_1>> {
    read_slot(reader, refs, TRAFFIC_DATA_CLASS_ID, |reader, refs| {
        let version = reader.read_u16_le()?;
        if !(1..=3).contains(&version) {
            return Err(version_error("TrafficData"));
        }
        let first_name = read_string_10_1(reader, max_bytes)?;
        let first_value = reader.read_u32_le()?;
        let double_bits = reader.read_f64_le()?.to_bits();
        let mut names: [String; 4] = Default::default();
        let mut daily_names: [String; 7] = Default::default();
        for name in &mut names {
            *name = read_string_10_1(reader, max_bytes)?;
        }
        let second_value = reader.read_u32_le()?;
        for name in &mut daily_names {
            *name = read_string_10_1(reader, max_bytes)?;
        }
        let extension = (version >= 2)
            .then(|| -> io::Result<_> {
                let initial_names = [
                    read_string_10_1(reader, max_bytes)?,
                    read_string_10_1(reader, max_bytes)?,
                ];
                let value = reader.read_u32_le()?;
                let final_names = [
                    read_string_10_1(reader, max_bytes)?,
                    read_string_10_1(reader, max_bytes)?,
                ];
                let feed = (version == 3)
                    .then(|| read_flagged(reader, |r| read_feed_location(r, max_bytes, refs)))
                    .transpose()?;
                Ok(TrafficDataExtension10_1 {
                    initial_names,
                    value,
                    final_names,
                    feed,
                })
            })
            .transpose()?;
        Ok(TrafficData10_1 {
            version,
            first_name,
            first_value,
            double_bits,
            names,
            second_value,
            daily_names,
            extension,
        })
    })
}

pub const NETWORK_ATTRIBUTE_CLASS_ID: [u8; 16] = [
    0xe3, 0x7c, 0x6f, 0xa8, 0xe0, 0x37, 0xef, 0x4b, 0xb6, 0x6c, 0x72, 0xb4, 0x17, 0x96, 0xcf, 0x03,
];
pub const EVALUATED_NETWORK_ATTRIBUTE_CLASS_ID: [u8; 16] = [
    0x2f, 0x7b, 0x8d, 0xcf, 0x8b, 0x62, 0x9d, 0x4f, 0xb1, 0x49, 0xd5, 0xdb, 0x3c, 0xd3, 0x2a, 0x35,
];
pub const NETWORK_ATTRIBUTE_PARAMETER_CLASS_ID: [u8; 16] = [
    0x15, 0xab, 0x43, 0xba, 0x5d, 0x04, 0xb1, 0x4e, 0x85, 0x26, 0xea, 0x65, 0xc4, 0x54, 0x8d, 0x4a,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkAttributeParameter10_1 {
    pub version: u16,
    pub name: String,
    pub value: u32,
    pub variants: [PropertyValueObjectStream10_1; 2],
    pub extra_value: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkAttribute10_1 {
    pub version: u16,
    pub initial_value: u32,
    pub name: String,
    pub values: [u32; 3],
    pub properties: DirectionSlot10_1<PropertySetObjectStream10_1>,
    pub extra_value: Option<u32>,
    pub parameters: Option<(i32, Vec<DirectionSlot10_1<NetworkAttributeParameter10_1>>)>,
}

fn read_attribute_parameter(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NetworkAttributeParameter10_1> {
    let version = reader.read_u16_le()?;
    if !(1..=2).contains(&version) {
        return Err(version_error("NetworkAttributeParameter"));
    }
    let name = read_string_10_1(reader, max_bytes)?;
    let value = reader.read_u32_le()?;
    let variants = [
        read_padded_variant_object_stream_10_1(reader, max_bytes, refs)?,
        read_padded_variant_object_stream_10_1(reader, max_bytes, refs)?,
    ];
    let extra_value = (version == 2).then(|| reader.read_u32_le()).transpose()?;
    Ok(NetworkAttributeParameter10_1 {
        version,
        name,
        value,
        variants,
        extra_value,
    })
}

/// 读取 INetworkAttribute 槽。两个 GdbNetwork 类共用持久化体。
pub fn read_network_attribute_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DirectionSlot10_1<NetworkAttribute10_1>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let (class, is_new) = match entry {
        ObjectEntry10_1::Null => return Ok(DirectionSlot10_1 { entry, body: None }),
        ObjectEntry10_1::BackReference { class_id, key } => {
            (refs.registered_class(key).unwrap_or(class_id), false)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => (class_id, true),
    };
    if ![
        NETWORK_ATTRIBUTE_CLASS_ID,
        EVALUATED_NETWORK_ATTRIBUTE_CLASS_ID,
    ]
    .contains(&class)
    {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "INetworkAttribute 槽包含未知对象类",
        ));
    }
    let body = is_new
        .then(|| -> io::Result<_> {
            let version = reader.read_u16_le()?;
            if !(1..=3).contains(&version) {
                return Err(version_error("NetworkAttribute"));
            }
            let initial_value = reader.read_u32_le()?;
            let name = read_string_10_1(reader, max_bytes)?;
            let values = [
                reader.read_u32_le()?,
                reader.read_u32_le()?,
                reader.read_u32_le()?,
            ];
            let properties = read_slot(reader, refs, PROPERTY_SET_CLASS_ID, |r, refs| {
                read_property_set_object_stream_10_1(r, max_bytes, refs)
            })?;
            let extra_value = (version >= 2).then(|| reader.read_u32_le()).transpose()?;
            let parameters = (version == 3)
                .then(|| {
                    read_list(reader, 16, max_bytes, |r| {
                        read_slot(r, refs, NETWORK_ATTRIBUTE_PARAMETER_CLASS_ID, |r, refs| {
                            read_attribute_parameter(r, max_bytes, refs)
                        })
                    })
                })
                .transpose()?;
            Ok(NetworkAttribute10_1 {
                version,
                initial_value,
                name,
                values,
                properties,
                extra_value,
                parameters,
            })
        })
        .transpose()?;
    Ok(DirectionSlot10_1 { entry, body })
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
    fn sdc_network_source_reads_unconditional_direction_slot() {
        let mut bytes = Vec::new();
        object(&mut bytes, SDC_NETWORK_SOURCE_CLASS_ID, 1);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend(17_u16.to_le_bytes());
        bytes.extend(28_u32.to_le_bytes());
        bytes.extend(EMPTY);
        object(&mut bytes, NETWORK_SOURCE_DIRECTIONS_CLASS_ID, 2);
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(EMPTY);
        bytes.extend([0; 24]);
        let end = bytes.len();
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        let body =
            read_network_source_slot_10_1(&mut reader, bytes.len(), &mut RefTable10_1::new())
                .unwrap()
                .body
                .unwrap();
        assert_eq!(reader.position(), end);
        assert_eq!(body.value, 28);
        let sdc = body.sdc_fields.unwrap();
        assert_eq!(sdc.short_value, 17);
        assert!(sdc.directions.body.is_some());
        for cut in 0..end {
            assert!(
                read_network_source_slot_10_1(
                    &mut Reader::new(&bytes[..cut]),
                    bytes.len(),
                    &mut RefTable10_1::new(),
                )
                .is_err(),
                "截断位置 {cut}"
            );
        }
        bytes[20] = 0;
        assert!(
            read_network_source_slot_10_1(
                &mut Reader::new(&bytes),
                bytes.len(),
                &mut RefTable10_1::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn network_attributes_read_common_body_and_parameters() {
        for class in [
            NETWORK_ATTRIBUTE_CLASS_ID,
            EVALUATED_NETWORK_ATTRIBUTE_CLASS_ID,
        ] {
            for version in 1_u16..=3 {
                let mut bytes = Vec::new();
                object(&mut bytes, class, 1);
                bytes.extend(version.to_le_bytes());
                bytes.extend(44_u32.to_le_bytes());
                bytes.extend(EMPTY);
                bytes.extend([0; 28]);
                if version >= 2 {
                    bytes.extend(9_u32.to_le_bytes());
                }
                if version == 3 {
                    bytes.extend(1_u32.to_le_bytes());
                    object(&mut bytes, NETWORK_ATTRIBUTE_PARAMETER_CLASS_ID, 2);
                    bytes.extend(2_u16.to_le_bytes());
                    bytes.extend(EMPTY);
                    bytes.extend(6_u32.to_le_bytes());
                    bytes.extend([0; 32]);
                    bytes.extend(12_u32.to_le_bytes());
                }
                let end = bytes.len();
                bytes.push(0xee);
                let mut reader = Reader::new(&bytes);
                let body = read_network_attribute_slot_10_1(
                    &mut reader,
                    bytes.len(),
                    &mut RefTable10_1::new(),
                )
                .unwrap()
                .body
                .unwrap();
                assert_eq!(reader.position(), end);
                assert_eq!(body.extra_value, (version >= 2).then_some(9));
                if version == 3 {
                    let (_, parameters) = body.parameters.unwrap();
                    assert_eq!(parameters[0].body.as_ref().unwrap().extra_value, Some(12));
                }
                for end in 0..end {
                    assert!(
                        read_network_attribute_slot_10_1(
                            &mut Reader::new(&bytes[..end]),
                            bytes.len(),
                            &mut RefTable10_1::new(),
                        )
                        .is_err(),
                        "截断位置 {end}"
                    );
                }
            }
        }
    }

    #[test]
    fn traffic_data_checks_each_version_and_feed_class() {
        for (version, feed_class) in [
            (1_u16, None),
            (2, None),
            (3, None),
            (3, Some(TRAFFIC_FEED_DIRECTORY_CLASS_ID)),
            (3, Some(TRAFFIC_FEED_GP_SERVICE_CLASS_ID)),
        ] {
            let mut bytes = Vec::new();
            object(&mut bytes, TRAFFIC_DATA_CLASS_ID, 1);
            bytes.extend(version.to_le_bytes());
            bytes.extend(EMPTY);
            bytes.extend(23_u32.to_le_bytes());
            bytes.extend(2.5_f64.to_le_bytes());
            bytes.extend(EMPTY.repeat(4));
            bytes.extend(13_u32.to_le_bytes());
            bytes.extend(EMPTY.repeat(7));
            if version >= 2 {
                bytes.extend(EMPTY.repeat(2));
                bytes.extend(5_u32.to_le_bytes());
                bytes.extend(EMPTY.repeat(2));
            }
            if version == 3 {
                bytes.extend(u32::from(feed_class.is_some()).to_le_bytes());
                if let Some(class) = feed_class {
                    object(&mut bytes, class, 2);
                    bytes.extend(1_u16.to_le_bytes());
                    if class == TRAFFIC_FEED_GP_SERVICE_CLASS_ID {
                        bytes.extend([0; 16]);
                        bytes.extend(EMPTY);
                    }
                    bytes.extend(EMPTY);
                }
            }
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let body =
                read_traffic_data_slot_10_1(&mut reader, bytes.len(), &mut RefTable10_1::new())
                    .unwrap()
                    .body
                    .unwrap();
            assert_eq!(reader.position(), end);
            assert_eq!(body.double_bits, 2.5_f64.to_bits());
            assert_eq!(body.extension.is_some(), version >= 2);
            let feed = body
                .extension
                .and_then(|tail| tail.feed)
                .and_then(|(_, slot)| slot);
            assert_eq!(feed.is_some(), feed_class.is_some());
            for end in 0..end {
                assert!(
                    read_traffic_data_slot_10_1(
                        &mut Reader::new(&bytes[..end]),
                        bytes.len(),
                        &mut RefTable10_1::new(),
                    )
                    .is_err(),
                    "截断位置 {end}"
                );
            }
        }
    }

    #[test]
    fn network_directions_reads_each_version_to_its_end() {
        for version in 1_u16..=3 {
            let mut bytes = Vec::new();
            object(&mut bytes, NETWORK_DIRECTIONS_CLASS_ID, 1);
            bytes.extend(version.to_le_bytes());
            bytes.extend(EMPTY.repeat(3));
            bytes.extend(12_u32.to_le_bytes());
            if version >= 2 {
                bytes.extend(EMPTY.repeat(2));
            }
            if version == 3 {
                bytes.extend(1_u32.to_le_bytes());
                object(&mut bytes, DIRECTIONS_ATTRIBUTE_MAPPING_CLASS_ID, 2);
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(EMPTY.repeat(2));
            }
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let body = read_network_directions_slot_10_1(
                &mut reader,
                bytes.len(),
                &mut RefTable10_1::new(),
            )
            .unwrap()
            .body
            .unwrap();
            assert_eq!(reader.position(), end);
            assert_eq!(body.extra_names.is_some(), version >= 2);
            assert_eq!(
                body.mappings.as_ref().map(|(_, rows)| rows.len()),
                (version == 3).then_some(1)
            );
            for end in 0..end {
                assert!(
                    read_network_directions_slot_10_1(
                        &mut Reader::new(&bytes[..end]),
                        bytes.len(),
                        &mut RefTable10_1::new(),
                    )
                    .is_err(),
                    "截断位置 {end}"
                );
            }
        }
    }

    #[test]
    fn network_source_reads_all_derived_boundaries() {
        for (class, names, directions) in [
            (NETWORK_SOURCE_CLASS_ID, 0, false),
            (TURN_FEATURE_SOURCE_CLASS_ID, 0, false),
            (SYSTEM_JUNCTION_SOURCE_CLASS_ID, 1, false),
            (JUNCTION_FEATURE_SOURCE_CLASS_ID, 1, true),
            (EDGE_FEATURE_SOURCE_CLASS_ID, 2, true),
        ] {
            let mut bytes = Vec::new();
            object(&mut bytes, class, 1);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 8]);
            bytes.extend(EMPTY);
            bytes.extend([0; 20]);
            if names > 0 {
                bytes.extend(1_u16.to_le_bytes());
                if directions {
                    bytes.extend(1_u32.to_le_bytes());
                    object(&mut bytes, NETWORK_SOURCE_DIRECTIONS_CLASS_ID, 2);
                    bytes.extend(0_u16.to_le_bytes());
                    bytes.extend(EMPTY);
                    bytes.extend([0; 8]);
                }
                for _ in 0..names {
                    bytes.extend(4_u32.to_le_bytes());
                    bytes.extend(EMPTY);
                }
                if directions {
                    bytes.extend([0; 16]);
                }
            }
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let slot =
                read_network_source_slot_10_1(&mut reader, bytes.len(), &mut RefTable10_1::new())
                    .unwrap();
            assert_eq!(reader.position(), end);
            let extension = slot.body.unwrap().extension;
            assert_eq!(extension.as_ref().map_or(0, |ext| ext.names.len()), names);
            assert_eq!(
                extension
                    .as_ref()
                    .is_some_and(|ext| ext.directions.is_some()),
                directions
            );
            for end in 0..end {
                assert!(
                    read_network_source_slot_10_1(
                        &mut Reader::new(&bytes[..end]),
                        bytes.len(),
                        &mut RefTable10_1::new(),
                    )
                    .is_err(),
                    "截断位置 {end}"
                );
            }
        }
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
        bytes.extend(EMPTY.repeat(4));

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
