//! GdbCore.dll 10.1 的 RangeDomain 和 CodedValueDomain。

use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::{read_bstr_string_10_1, read_string_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const RANGE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x1a, 0x6c, 0x4c, 0xf8, 0xff, 0x47, 0xd2, 0x11, 0x99, 0x33, 0x00, 0x00, 0xf8, 0x03, 0x72, 0xb4,
];
pub const CODED_VALUE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x1b, 0x6c, 0x4c, 0xf8, 0xff, 0x47, 0xd2, 0x11, 0x99, 0x33, 0x00, 0x00, 0xf8, 0x03, 0x72, 0xb4,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<DomainBody10_1>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainBody10_1 {
    pub version: u16,
    pub base_version: u16,
    pub base_longs: [i32; 3],
    pub base_strings: [String; 3],
    pub long_before_values: i32,
    pub values: DomainValues10_1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainValues10_1 {
    Range([PropertyValueObjectStream10_1; 2]),
    Coded(Vec<CodedDomainValue10_1>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodedDomainValue10_1 {
    pub label: String,
    pub code: PropertyValueObjectStream10_1,
}

fn version(reader: &mut Reader<'_>) -> io::Result<u16> {
    let version = reader.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("Domain 未支持版本 {version}"),
        ));
    }
    Ok(version)
}

fn long(reader: &mut Reader<'_>) -> io::Result<i32> {
    Ok(reader.read_u32_le()? as i32)
}

/// 从普通 ObjectStream 槽开始读取。失败时不更改游标或引用表。
pub fn read_domain_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<DomainSlot10_1> {
    let mut r = reader.clone();
    let mut staged = refs.clone();
    let entry = read_object_entry_10_1(&mut r, &mut staged)?;
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject { class_id, .. } => Some(Box::new(read_body(
            &mut r,
            max_bytes,
            &mut staged,
            class_id,
        )?)),
    };
    *reader = r;
    *refs = staged;
    Ok(DomainSlot10_1 { entry, body })
}

fn read_body(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
) -> io::Result<DomainBody10_1> {
    let is_range = match class_id {
        RANGE_DOMAIN_CLASS_ID => true,
        CODED_VALUE_DOMAIN_CLASS_ID => false,
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "IDomain 未支持的新对象类 {}",
                    crate::storage::class_id_hex(&class_id)
                ),
            ));
        }
    };
    let outer_version = version(r)?;
    let base_version = version(r)?;
    let base_longs = [long(r)?, long(r)?, long(r)?];
    let base_strings = [
        read_string_10_1(r, max_bytes)?,
        read_string_10_1(r, max_bytes)?,
        read_string_10_1(r, max_bytes)?,
    ];
    let long_before_values = long(r)?;
    let values = if is_range {
        DomainValues10_1::Range([
            read_padded_variant_object_stream_10_1(r, max_bytes, refs)?,
            read_padded_variant_object_stream_10_1(r, max_bytes, refs)?,
        ])
    } else {
        let count = usize::try_from(long(r)?)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "编码值域计数为负"))?;
        if count > r.remaining() / 20 {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "编码值域计数超出可读范围",
            ));
        }
        let mut entries = Vec::new();
        for _ in 0..count {
            entries.push(CodedDomainValue10_1 {
                // 自由函数 ReadString 用码元数量，后跟 Variant::Read(false) 的值。
                label: read_bstr_string_10_1(r, max_bytes)?,
                code: read_padded_variant_object_stream_10_1(r, max_bytes, refs)?,
            });
        }
        DomainValues10_1::Coded(entries)
    };
    Ok(DomainBody10_1 {
        version: outer_version,
        base_version,
        base_longs,
        base_strings,
        long_before_values,
        values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::property_set::{
        PROPERTY_SET_CLASS_ID, PropertyValue10_1, PropertyValueObjectStream10_1::Inline,
    };
    use crate::stream::variant_array::ClassVariantArray10_1;

    #[test]
    fn coded_domain_label_and_code_use_distinct_string_lengths() {
        let mut fixture = CODED_VALUE_DOMAIN_CLASS_ID.to_vec();
        fixture.extend(71_u32.to_le_bytes());
        fixture.extend(1_u16.to_le_bytes());
        fixture.extend(1_u16.to_le_bytes());
        fixture.extend([0; 12]);
        for _ in 0..3 {
            fixture.extend(2_u32.to_le_bytes());
            fixture.extend(0_u16.to_le_bytes());
        }
        fixture.extend(0_u32.to_le_bytes());
        let count_offset = fixture.len();
        fixture.extend(2_u32.to_le_bytes());
        fixture.extend(1_u32.to_le_bytes()); // 自由 ReadString：一个 UTF-16 码元。
        fixture.extend(('甲' as u16).to_le_bytes());
        fixture.extend(0_u16.to_le_bytes());
        fixture.extend(8_u16.to_le_bytes()); // VT_BSTR。
        fixture.extend([0; 14]);
        fixture.extend(4_u32.to_le_bytes()); // Variant 的 String：含零共四字节。
        fixture.extend(('乙' as u16).to_le_bytes());
        fixture.extend(0_u16.to_le_bytes());
        fixture.extend(0_u32.to_le_bytes()); // 第二项为空标签。
        fixture.extend(0x2011_u16.to_le_bytes()); // VT_ARRAY | VT_UI1。
        fixture.extend([0; 14]);
        fixture.extend(3_i32.to_le_bytes());
        fixture.extend([0x02, 0x80, 0xff]);

        for cut in 0..fixture.len() {
            let mut reader = Reader::new(&fixture[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_domain_slot_10_1(&mut reader, 64, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(71));
        }
        fixture.push(0x53);
        let mut reader = Reader::new(&fixture);
        let mut refs = RefTable10_1::new();
        let slot = read_domain_slot_10_1(&mut reader, 64, &mut refs).unwrap();
        assert_eq!(
            slot.body.unwrap().values,
            DomainValues10_1::Coded(vec![
                CodedDomainValue10_1 {
                    label: "甲".into(),
                    code: Inline(PropertyValue10_1::String("乙".into())),
                },
                CodedDomainValue10_1 {
                    label: String::new(),
                    code: Inline(PropertyValue10_1::Array(ClassVariantArray10_1::Fixed {
                        vartype: 17,
                        element_bytes: 1,
                        raw: vec![0x02, 0x80, 0xff],
                    })),
                },
            ]),
        );
        assert!(refs.contains(71));
        assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);

        fixture[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        let mut reader = Reader::new(&fixture);
        let mut refs = RefTable10_1::new();
        assert!(read_domain_slot_10_1(&mut reader, 64, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn range_domain_padded_objects_share_keys_and_roll_back_on_unknown_class() {
        let mut fixture = RANGE_DOMAIN_CLASS_ID.to_vec();
        fixture.extend(71_u32.to_le_bytes());
        fixture.extend(1_u16.to_le_bytes());
        fixture.extend(1_u16.to_le_bytes());
        fixture.extend([0; 12]);
        for _ in 0..3 {
            fixture.extend(2_u32.to_le_bytes());
            fixture.extend(0_u16.to_le_bytes());
        }
        fixture.extend(0_i32.to_le_bytes());
        for vt in [13_u16, 9] {
            fixture.extend(vt.to_le_bytes());
            // Load 跳过原字节，无需把保留区解释成独立字段。
            fixture.extend([0xa5; 14]);
            fixture.extend(PROPERTY_SET_CLASS_ID);
            fixture.extend(72_u32.to_le_bytes());
            if vt == 13 {
                fixture.extend(1_u16.to_le_bytes());
                fixture.extend(0_i32.to_le_bytes());
            }
        }
        let end = fixture.len();
        fixture.push(0x78);
        let mut reader = Reader::new(&fixture);
        let mut refs = RefTable10_1::new();
        let domain = read_domain_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let DomainValues10_1::Range([lower, upper]) = domain.body.unwrap().values else {
            panic!("应读到范围值域");
        };
        assert!(matches!(lower,
            PropertyValueObjectStream10_1::Object { class_id: PROPERTY_SET_CLASS_ID, key: 72, body }
                if body.count == 0
        ));
        assert_eq!(
            upper,
            PropertyValueObjectStream10_1::BackReference {
                class_id: PROPERTY_SET_CLASS_ID,
                key: 72
            }
        );
        assert_eq!(reader.position(), end);
        assert!(refs.contains(71) && refs.contains(72));

        // 回引改成未见过的键和未知类；须同时回退已读的第一个值。
        fixture[end - 20] ^= 1;
        fixture[end - 4..end].copy_from_slice(&73_u32.to_le_bytes());
        let mut reader = Reader::new(&fixture);
        let mut refs = RefTable10_1::new();
        refs.register(70);
        let before = refs.clone();
        assert_eq!(
            read_domain_slot_10_1(&mut reader, 4096, &mut refs)
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, before);
    }
}
