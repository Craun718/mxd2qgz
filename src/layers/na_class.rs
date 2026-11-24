//! NetworkAnalyst.dll 10.1 的 NAClass 与 NAClassDefinition 持久化前缀。

use crate::layers::feature_layer_tail::{UID_CLASS_ID, UidBody10_1, read_uid_body_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::objects::fields::{FIELDS_CLASS_ID, FieldsBody10_1, read_fields_body_10_1};
use crate::objects::string_array::{StringArraySlot10_1, read_string_array_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::{read_bstr_string_10_1, read_string_10_1};
use crate::stream::reader::Reader;
use std::io;

pub const NA_CLASS_CLASS_ID: [u8; 16] = [
    0xe6, 0xf4, 0x7a, 0xcf, 0xba, 0x4f, 0xb2, 0x4f, 0xa4, 0x17, 0x92, 0xb0, 0x74, 0xb8, 0x8c, 0x90,
];
pub const NA_CLASS_DEFINITION_CLASS_ID: [u8; 16] = [
    0x8e, 0x05, 0xe2, 0x61, 0x04, 0xf1, 0xaa, 0x47, 0x8c, 0x62, 0x33, 0xd5, 0xbf, 0x43, 0xeb, 0xe8,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NaClassBody10_1 {
    pub class_base: InMemoryClassBase10_1,
    pub version: u16,
    pub definition: (ObjectEntry10_1, Option<Box<NaClassDefinition10_1>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InMemoryClassBase10_1 {
    pub version: u16,
    pub legacy_name: String,
    pub name: String,
    pub class_guid: [u8; 16],
    pub scalar_118: i32,
    pub feature_type: Option<i32>,
    pub flags: [u8; 4],
    pub fields: (ObjectEntry10_1, Option<Box<FieldsBody10_1>>),
    pub middle: ObjectEntry10_1,
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub scalar_after_spatial_reference: i32,
    pub record_count: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaClassDefinition10_1 {
    pub version: u16,
    pub name: String,
    pub uid: (ObjectEntry10_1, Option<Box<UidBody10_1>>),
    pub fields: (ObjectEntry10_1, Option<Box<FieldsBody10_1>>),
    pub lower_bound: i32,
    pub upper_bound: i32,
    pub is_input: u16,
    pub is_output: u16,
    pub field_types: Vec<NaClassFieldType10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NaClassFieldType10_1 {
    pub name: String,
    pub raw: i32,
    pub strings: StringArraySlot10_1,
}

pub(super) fn read_na_class_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Box<NaClassBody10_1>>)> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut keys)?;
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: NA_CLASS_CLASS_ID,
            ..
        } => Some(Box::new(read_na_class_body(
            &mut trial, max_bytes, &mut keys,
        )?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported("NALayer 命名项", &class_id));
        }
    };
    *reader = trial;
    *refs = keys;
    Ok((entry, body))
}

pub(super) fn read_na_class_body(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NaClassBody10_1> {
    let class_base = read_in_memory_class_base(reader, max_bytes, refs)?;
    let version = reader.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("NAClass 版本 {version} 超过 1"),
        ));
    }
    let definition = read_na_class_definition_slot(reader, max_bytes, refs)?;
    Ok(NaClassBody10_1 {
        class_base,
        version,
        definition,
    })
}

fn read_in_memory_class_base(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<InMemoryClassBase10_1> {
    let version = reader.read_u16_le()?;
    if version > 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("InMemoryClassBase 版本 {version} 超过 2"),
        ));
    }
    let legacy_name = read_string_10_1(reader, max_bytes)?;
    let name = read_string_10_1(reader, max_bytes)?;
    let class_guid = reader.read_bytes(16)?.try_into().unwrap();
    let scalar_118 = reader.read_u32_le()? as i32;
    let feature_type = (version >= 2)
        .then(|| reader.read_u32_le().map(|value| value as i32))
        .transpose()?;
    let flags = reader.read_bytes(4)?.try_into().unwrap();

    let fields_entry = read_object_entry_10_1(reader, refs)?;
    let fields_body = match fields_entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: FIELDS_CLASS_ID,
            ..
        } => Some(Box::new(read_fields_body_10_1(reader, max_bytes, refs)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported("IFields", &class_id));
        }
    };

    let middle = read_object_entry_10_1(reader, refs)?;
    if matches!(middle, ObjectEntry10_1::NewObject { .. }) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "InMemoryClassBase 中间对象槽有非空新对象，当前实现未支持",
        ));
    }
    let spatial_reference = read_spatial_reference_entry_10_1(reader, max_bytes, refs)?;
    let scalar_after_spatial_reference = reader.read_u32_le()? as i32;
    let record_count = reader.read_u32_le()? as i32;
    parse_records(reader, fields_body.as_deref(), record_count)?;

    Ok(InMemoryClassBase10_1 {
        version,
        legacy_name,
        name,
        class_guid,
        scalar_118,
        feature_type,
        flags,
        fields: (fields_entry, fields_body),
        middle,
        spatial_reference,
        scalar_after_spatial_reference,
        record_count,
    })
}

fn parse_records(
    reader: &mut Reader<'_>,
    fields: Option<&FieldsBody10_1>,
    count: i32,
) -> io::Result<()> {
    let count = count.max(0) as usize;
    if count > reader.remaining() / 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "InMemoryClassBase 记录数量超出可读范围",
        ));
    }
    if count == 0 {
        return Ok(());
    }
    let fields = fields.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "InMemoryClassBase 有记录但 Fields 为回引或空槽，无法确定字段数",
        )
    })?;
    if fields.fields.iter().any(|field| field.body.is_none()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "InMemoryClassBase 的记录布局依赖未展开的字段对象",
        ));
    }
    let field_count = fields.fields.len();
    let oid_index = field_type_index(fields, 6);
    let geometry_index = field_type_index(fields, 7);
    for _ in 0..count {
        let _oid = reader.read_u32_le()?;
        if geometry_index.is_some() {
            let length = reader.read_u32_le()? as usize;
            reader.read_bytes(length)?;
        }
        for index in 0..field_count {
            if Some(index) == oid_index || Some(index) == geometry_index {
                continue;
            }
            let tag = reader.read_u16_le()?;
            match tag {
                2 | 11 => {
                    reader.read_bytes(2)?;
                }
                3 | 4 => {
                    reader.read_bytes(4)?;
                }
                5 | 7 => {
                    reader.read_bytes(8)?;
                }
                8 => {
                    let length = (reader.read_u32_le()? & 0x7fff_ffff) as usize;
                    reader.read_bytes(length)?;
                }
                13 => {
                    let length = reader.read_u32_le()? as usize;
                    reader.read_bytes(length)?;
                }
                17 => {
                    reader.read_bytes(1)?;
                }
                0 | 1 => {}
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!("InMemoryClassBase 记录值类型 {tag} 未支持"),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn field_type_index(fields: &FieldsBody10_1, field_type: i32) -> Option<usize> {
    fields.fields.iter().position(|field| {
        field
            .body
            .as_ref()
            .is_some_and(|body| body.longs_74_to_80[0] == field_type)
    })
}

pub(crate) fn read_na_class_definition_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Box<NaClassDefinition10_1>>)> {
    let mut trial = reader.clone();
    let mut staged_refs = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged_refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key } => {
            let registered = staged_refs.registered_class(key).unwrap_or(class_id);
            if registered != NA_CLASS_DEFINITION_CLASS_ID {
                return Err(unsupported("INAClassDefinition 回引", &registered));
            }
            None
        }
        ObjectEntry10_1::NewObject {
            class_id: NA_CLASS_DEFINITION_CLASS_ID,
            ..
        } => Some(Box::new(read_na_class_definition(
            &mut trial,
            max_bytes,
            &mut staged_refs,
        )?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported("INAClassDefinition", &class_id));
        }
    };
    *reader = trial;
    *refs = staged_refs;
    Ok((entry, body))
}

fn read_na_class_definition(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NaClassDefinition10_1> {
    let version = reader.read_u16_le()?;
    if version as i16 > 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("NAClassDefinition 版本 {version} 超过 0"),
        ));
    }
    let name = read_bstr_string_10_1(reader, max_bytes)?;

    let uid_entry = read_object_entry_10_1(reader, refs)?;
    let uid_body = match uid_entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: UID_CLASS_ID,
            key,
        } => Some(Box::new(read_uid_body_10_1(reader, key)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported("IUID", &class_id));
        }
    };

    let fields_entry = read_object_entry_10_1(reader, refs)?;
    let fields_body = match fields_entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: FIELDS_CLASS_ID,
            ..
        } => Some(Box::new(read_fields_body_10_1(reader, max_bytes, refs)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported("IFields", &class_id));
        }
    };

    let lower_bound = reader.read_u32_le()? as i32;
    let upper_bound = reader.read_u32_le()? as i32;
    let is_input = reader.read_u16_le()?;
    let is_output = reader.read_u16_le()?;
    let count = usize::try_from(reader.read_u32_le()? as i32)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "NAClassDefinition 字段数为负"))?;
    if count > reader.remaining() / 10 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "NAClassDefinition 字段数超出可读范围",
        ));
    }
    let mut field_types = Vec::with_capacity(count);
    for _ in 0..count {
        let name = read_bstr_string_10_1(reader, max_bytes)?;
        let raw = reader.read_u32_le()? as i32;
        let strings = read_string_array_slot_10_1(reader, max_bytes, refs)?;
        field_types.push(NaClassFieldType10_1 { name, raw, strings });
    }

    Ok(NaClassDefinition10_1 {
        version,
        name,
        uid: (uid_entry, uid_body),
        fields: (fields_entry, fields_body),
        lower_bound,
        upper_bound,
        is_input,
        is_output,
        field_types,
    })
}

fn unsupported(slot: &str, class_id: &[u8; 16]) -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "{slot} 未支持的新对象类 {}",
            crate::storage::class_id_hex(class_id)
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_slot_keeps_its_boundary_and_validates_reference_class() {
        let mut bytes = NA_CLASS_DEFINITION_CLASS_ID.to_vec();
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend([0; 32]);
        bytes.extend([0; 16]);
        let end = bytes.len();
        let mut reference = [0x42; 16].to_vec();
        reference.extend(1_u32.to_le_bytes());
        bytes.extend(&reference);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let (_, body) = read_na_class_definition_slot(&mut reader, 1024, &mut refs).unwrap();
        assert!(body.unwrap().field_types.is_empty());
        assert_eq!(reader.position(), end);
        let (entry, body) = read_na_class_definition_slot(&mut reader, 1024, &mut refs).unwrap();
        assert!(matches!(
            entry,
            ObjectEntry10_1::BackReference { key: 1, .. }
        ));
        assert!(body.is_none());

        let mut refs = RefTable10_1::new();
        read_object_entry_10_1(&mut Reader::new(&reference), &mut refs).unwrap();
        let mut reader = Reader::new(&reference);
        assert!(read_na_class_definition_slot(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
    }
}
