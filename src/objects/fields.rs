//! ArcMap 10.1 GdbCore 的 Fields 和 Field 对象体；证据见 docs/mxd-10.1-fields.md。

use crate::objects::domain::{DomainSlot10_1, read_domain_slot_nested_10_1};
use crate::objects::field_defs::{
    FieldDefinitionSlot10_1, GeometryDef10_1, RasterDef10_1, read_geometry_def_slot_10_1,
    read_raster_def_slot_10_1,
};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_variant_object_stream_value_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const FIELD_CLASS_ID: [u8; 16] = [
    0x34, 0x75, 0x4f, 0xf9, 0xdf, 0x9f, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c, 0x42, 0x68,
];
pub const FIELDS_CLASS_ID: [u8; 16] = [
    0x35, 0x75, 0x4f, 0xf9, 0xdf, 0x9f, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c, 0x42, 0x68,
];

#[derive(Debug, Clone, PartialEq)]
pub struct FieldsBody10_1 {
    pub version: u16,
    pub fields: Vec<FieldSlot10_1>,
    /// v2 为每个位置各存一个 int32，包含空引用所占的位置。
    pub longs_v2: Option<Vec<i32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<FieldBody10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldBody10_1 {
    pub version: u16,
    pub gp_value_version: u16,
    pub gp_value_short: u16,
    pub strings: [String; 4],
    pub string_flags: [u8; 3],
    pub default: PropertyValueObjectStream10_1,
    /// 第一个成员为 esriFieldType；其余成员保留原值。
    pub longs_74_to_80: [i32; 4],
    pub trailing_flags: [u8; 5],
    pub domain: DomainSlot10_1,
    pub geometry_def: FieldDefinitionSlot10_1<GeometryDef10_1>,
    pub raster_def: Option<FieldDefinitionSlot10_1<RasterDef10_1>>,
}

fn version(reader: &mut Reader<'_>, high: u16, kind: &str) -> io::Result<u16> {
    let value = reader.read_u16_le()?;
    if value > high {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{kind} 未支持版本 {value}"),
        ));
    }
    Ok(value)
}

fn bounded_count(reader: &mut Reader<'_>, min_size: usize) -> io::Result<usize> {
    let n = usize::try_from(reader.read_u32_le()? as i32)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Fields 计数为负"))?;
    if n > reader.remaining() / min_size {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Fields 计数超出可读范围",
        ));
    }
    Ok(n)
}

fn unsupported_class(kind: &str, class_id: &[u8; 16]) -> io::Error {
    io::Error::new(
        ErrorKind::Unsupported,
        format!(
            "{kind} 未支持的对象类 {}",
            crate::storage::class_id_hex(class_id)
        ),
    )
}

/// 调用者已消费 Fields 对象头。任意嵌套对象未闭合时，回退游标和全部字段键。
pub fn read_fields_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<FieldsBody10_1> {
    read_fields_body_nested_10_1(reader, max_bytes, refs, 0)
}

pub(crate) fn read_fields_body_nested_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<FieldsBody10_1> {
    let mut r = reader.clone();
    let mut staged = refs.clone();
    let version = version(&mut r, 2, "Fields")?;
    let count = bounded_count(&mut r, if version >= 2 { 20 } else { 16 })?;
    let mut fields = Vec::new();
    for _ in 0..count {
        let mut entry = read_object_entry_10_1(&mut r, &mut staged)?;
        if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
            && let Some(registered) = staged.registered_class(*key)
        {
            if registered != FIELD_CLASS_ID {
                return Err(unsupported_class("IField 回引", &registered));
            }
            *class_id = registered;
        }
        let body = match entry {
            ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
            ObjectEntry10_1::NewObject {
                class_id: FIELD_CLASS_ID,
                ..
            } => Some(Box::new(read_field_body_10_1(
                &mut r,
                max_bytes,
                &mut staged,
                depth,
            )?)),
            ObjectEntry10_1::NewObject { class_id, .. } => {
                return Err(unsupported_class("IField", &class_id));
            }
        };
        fields.push(FieldSlot10_1 { entry, body });
    }
    let longs_v2 = (version >= 2)
        .then(|| {
            (0..count)
                .map(|_| r.read_u32_le().map(|n| n as i32))
                .collect::<io::Result<_>>()
        })
        .transpose()?;
    *reader = r;
    *refs = staged;
    Ok(FieldsBody10_1 {
        version,
        fields,
        longs_v2,
    })
}

pub(crate) fn read_field_body_10_1(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<FieldBody10_1> {
    let field_version = version(r, 2, "Field")?;
    let gp_value_version = version(r, 1, "GPValue")?;
    let gp_value_short = r.read_u16_le()?;
    let a = read_string_10_1(r, max_bytes)?;
    let flag_a = r.read_bytes(1)?[0];
    let b = read_string_10_1(r, max_bytes)?;
    let flag_b = r.read_bytes(1)?[0];
    let c = read_string_10_1(r, max_bytes)?;
    let d = read_string_10_1(r, max_bytes)?;
    let flag_c = r.read_bytes(1)?[0];
    let default = read_variant_object_stream_value_10_1(r, max_bytes, refs, depth)?;
    let mut longs_74_to_80 = [0; 4];
    for long in &mut longs_74_to_80 {
        *long = r.read_u32_le()? as i32;
    }
    let trailing_flags = r.read_bytes(5)?.try_into().unwrap();
    let domain = read_domain_slot_nested_10_1(r, max_bytes, refs, depth)?;
    let geometry_def = read_geometry_def_slot_10_1(r, max_bytes, refs)?;
    let raster_def = (field_version >= 2 && longs_74_to_80[0] == 9)
        .then(|| read_raster_def_slot_10_1(r, max_bytes, refs))
        .transpose()?;
    Ok(FieldBody10_1 {
        version: field_version,
        gp_value_version,
        gp_value_short,
        strings: [a, b, c, d],
        string_flags: [flag_a, flag_b, flag_c],
        default,
        longs_74_to_80,
        trailing_flags,
        domain,
        geometry_def,
        raster_def,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_reference_uses_the_registered_class() {
        for (registered, claimed) in [
            (FIELD_CLASS_ID, FIELDS_CLASS_ID),
            (FIELDS_CLASS_ID, FIELD_CLASS_ID),
        ] {
            let mut header = registered.to_vec();
            header.extend(41_u32.to_le_bytes());
            let mut refs = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&header), &mut refs).unwrap();
            let before = refs.clone();

            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(claimed);
            bytes.extend(41_u32.to_le_bytes());
            bytes.extend(19_i32.to_le_bytes());
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let result = read_fields_body_10_1(&mut reader, 128, &mut refs);
            assert_eq!(refs, before);

            if registered == FIELD_CLASS_ID {
                let fields = result.unwrap();
                assert_eq!(
                    fields.fields[0].entry,
                    ObjectEntry10_1::BackReference {
                        class_id: FIELD_CLASS_ID,
                        key: 41,
                    }
                );
                assert_eq!(fields.longs_v2, Some(vec![19]));
                assert_eq!(reader.position(), end);
            } else {
                assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
            }
        }
    }
}
