//! 10.1 Field 的 IDomain 对象，见 docs/mxd-10.1-fields.md。

use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_nested_10_1,
};
use crate::objects::string_array::{StringArraySlot10_1, read_string_array_slot_10_1};
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
pub const BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID: [u8; 16] = [
    0x16, 0x89, 0x04, 0xc4, 0x0f, 0x41, 0xce, 0x4d, 0x90, 0x24, 0xbe, 0x5b, 0xe6, 0xba, 0x0a, 0x23,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<DomainBody10_1>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainBody10_1 {
    pub version: u16,
    pub base_version: Option<u16>,
    pub base_longs: [i32; 3],
    pub base_strings: [String; 3],
    pub long_before_values: i32,
    pub values: DomainValues10_1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainValues10_1 {
    Range([PropertyValueObjectStream10_1; 2]),
    Coded(Vec<CodedDomainValue10_1>),
    BitMask {
        prefix: [u32; 2],
        labels: StringArraySlot10_1,
    },
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
        ObjectEntry10_1::NewObject { class_id, .. } => Some(Box::new(read_domain_body_10_1(
            &mut r,
            max_bytes,
            &mut staged,
            class_id,
            0,
        )?)),
    };
    *reader = r;
    *refs = staged;
    Ok(DomainSlot10_1 { entry, body })
}

pub(crate) fn read_domain_body_10_1(
    r: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    class_id: [u8; 16],
    depth: usize,
) -> io::Result<DomainBody10_1> {
    if !matches!(
        class_id,
        RANGE_DOMAIN_CLASS_ID | CODED_VALUE_DOMAIN_CLASS_ID | BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID
    ) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!(
                "IDomain 未支持的新对象类 {}",
                crate::storage::class_id_hex(&class_id)
            ),
        ));
    }
    let outer_version = version(r)?;
    let is_bit_mask = class_id == BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID;
    if is_bit_mask && outer_version != 0 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("BitMaskCodedValueDomain 未支持版本 {outer_version}"),
        ));
    }
    let base_version = (!is_bit_mask).then(|| version(r)).transpose()?;
    let base_longs = [long(r)?, long(r)?, long(r)?];
    let base_strings = [
        read_string_10_1(r, max_bytes)?,
        read_string_10_1(r, max_bytes)?,
        read_string_10_1(r, max_bytes)?,
    ];
    let long_before_values = long(r)?;
    let values = if is_bit_mask {
        read_bit_mask_values(r, max_bytes, refs, long_before_values)?
    } else if class_id == RANGE_DOMAIN_CLASS_ID {
        DomainValues10_1::Range([
            read_padded_variant_object_stream_nested_10_1(r, max_bytes, refs, depth)?,
            read_padded_variant_object_stream_nested_10_1(r, max_bytes, refs, depth)?,
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
                code: read_padded_variant_object_stream_nested_10_1(r, max_bytes, refs, depth)?,
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

fn read_bit_mask_values(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    count: i32,
) -> io::Result<DomainValues10_1> {
    let prefix = [reader.read_u32_le()?, reader.read_u32_le()?];
    let [a, b] = prefix;
    let size = if b == 0 { a } else { b } as usize;
    if size > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BitMaskCodedValueDomain 的 ExtraSafe 长度超过限额",
        ));
    }
    let mut region = reader.read_region(size)?;
    let labels = if size == 0 {
        StringArraySlot10_1 {
            entry: ObjectEntry10_1::Null,
            body: None,
        }
    } else {
        read_string_array_slot_10_1(&mut region, max_bytes, refs)?
    };
    // 非空数组逐项与第一个 int32 数量对应，回引的标签保留在原对象上。
    let saved_count = match &labels.body {
        Some((_, strings)) => Some(strings.len()),
        None if labels.entry == ObjectEntry10_1::Null => Some(0),
        None => None,
    };
    if region.remaining() != 0 || count < 0 || saved_count.is_some_and(|len| len != count as usize)
    {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BitMaskCodedValueDomain 的标签数量或 ExtraSafe 边界不符",
        ));
    }
    Ok(DomainValues10_1::BitMask { prefix, labels })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::property_set::{
        PROPERTY_SET_CLASS_ID, PropertyValue10_1, PropertyValueObjectStream10_1::Inline,
    };
    use crate::stream::variant_array::ClassVariantArray10_1;

    #[test]
    fn bit_mask_domain_reads_bounded_labels_and_shares_their_key() {
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;

        let mut bytes = BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID.to_vec();
        bytes.extend(81_u32.to_le_bytes());
        bytes.extend(0_u16.to_le_bytes());
        for value in [3_i32, 2, 1] {
            bytes.extend(value.to_le_bytes());
        }
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]); // 三个空字符串。
        }
        let count_offset = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        let mut array = STR_ARRAY_CLASS_ID.to_vec();
        array.extend(82_u32.to_le_bytes());
        array.extend(1_u16.to_le_bytes());
        array.extend(2_i32.to_le_bytes());
        for label in ['甲', '乙'] {
            array.extend(4_u32.to_le_bytes());
            array.extend((label as u16).to_le_bytes());
            array.extend(0_u16.to_le_bytes());
        }
        let array_size = array.len() as u32;
        bytes.extend(array_size.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(array);
        let end = bytes.len();
        bytes.extend(0x1234_u16.to_le_bytes()); // 宿主的下一字段。

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_domain_slot_10_1(&mut reader, 4096, &mut refs)
            .unwrap()
            .body
            .unwrap();
        assert_eq!(parsed.base_version, None);
        assert_eq!(parsed.base_longs, [3, 2, 1]);
        assert_eq!(parsed.long_before_values, 2);
        let DomainValues10_1::BitMask { prefix, labels } = parsed.values else {
            panic!("预期掩码值域");
        };
        assert_eq!(prefix, [array_size, 0]);
        assert_eq!(
            labels.body,
            Some((1, vec!["甲".to_owned(), "乙".to_owned()]))
        );
        assert!(refs.contains(81) && refs.contains(82));
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);

        let mut wrong_count = bytes.clone();
        wrong_count[count_offset..count_offset + 4].copy_from_slice(&1_i32.to_le_bytes());
        for input in (0..end)
            .map(|cut| &bytes[..cut])
            .chain([wrong_count.as_slice()])
        {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            assert!(read_domain_slot_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn range_domain_reads_a_nested_coded_domain_and_its_reference() {
        fn header(bytes: &mut Vec<u8>, class: [u8; 16], key: u32) {
            bytes.extend(class);
            bytes.extend(key.to_le_bytes());
        }
        fn base(bytes: &mut Vec<u8>) {
            bytes.extend([0; 4 + 12]);
            bytes.extend([2, 0, 0, 0, 0, 0].repeat(3));
            bytes.extend([0; 4]);
        }

        let mut bytes = Vec::new();
        header(&mut bytes, RANGE_DOMAIN_CLASS_ID, 1);
        base(&mut bytes);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        header(&mut bytes, CODED_VALUE_DOMAIN_CLASS_ID, 2);
        base(&mut bytes);
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        header(&mut bytes, CODED_VALUE_DOMAIN_CLASS_ID, 2);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_domain_slot_10_1(&mut reader, 4096, &mut refs).unwrap();
        let DomainValues10_1::Range([lower, upper]) = parsed.body.unwrap().values else {
            panic!("缺少范围值域");
        };
        assert!(matches!(lower, PropertyValueObjectStream10_1::Domain {
            class_id: CODED_VALUE_DOMAIN_CLASS_ID, key: 2, body,
        } if matches!(&body.values, DomainValues10_1::Coded(values) if values.is_empty())));
        assert_eq!(
            upper,
            PropertyValueObjectStream10_1::BackReference {
                class_id: CODED_VALUE_DOMAIN_CLASS_ID,
                key: 2,
            }
        );
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(refs.len(), 2);

        let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_domain_slot_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn range_domain_shares_variant_spatial_reference_with_envelope() {
        use crate::objects::envelope::{ENVELOPE_CLASS_ID, SpatialReferenceEntry10_1};
        use crate::objects::geometry::GeometryBody10_1;
        use crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
        use crate::objects::spatial_reference_slots::{
            GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
        };

        let empty_string = [2, 0, 0, 0, 0, 0];
        for (class_id, body) in [
            (PROJECTED_COORDINATE_SYSTEM_CLASS_ID, vec![1, 0, 0, 0, 0, 0]),
            (
                GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID,
                vec![1, 0, 0, 0, 0, 0],
            ),
            (
                UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
                [vec![0, 0], empty_string.repeat(4)].concat(),
            ),
        ] {
            let mut bytes = RANGE_DOMAIN_CLASS_ID.to_vec();
            bytes.extend(71_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 12]);
            bytes.extend(empty_string.repeat(3));
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0; 14]);
            let class_offset = bytes.len();
            bytes.extend(class_id);
            bytes.extend(72_u32.to_le_bytes());
            bytes.extend(&body);
            bytes.extend(9_u16.to_le_bytes());
            bytes.extend([0; 14]);
            bytes.extend(ENVELOPE_CLASS_ID);
            bytes.extend(73_u32.to_le_bytes());
            bytes.extend(3_u16.to_le_bytes());
            bytes.extend([0; 36]); // 四个坐标和空 Z/M 标志。
            bytes.extend(class_id);
            bytes.extend(72_u32.to_le_bytes());
            let end = bytes.len();
            bytes.push(0x53);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let slot = read_domain_slot_10_1(&mut reader, 256, &mut refs).unwrap();
            let DomainValues10_1::Range([reference, bounds]) = slot.body.unwrap().values else {
                panic!("应读到范围值域");
            };
            let PropertyValueObjectStream10_1::SpatialReference(reference) = reference else {
                panic!("应读到空间参考 Variant");
            };
            assert_eq!((reference.class_id, reference.key), (class_id, 72));
            assert_eq!(reference.bytes(), body);
            let decoded_class = match reference.body() {
                SpatialReferenceEntry10_1::Projected { .. } => PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
                SpatialReferenceEntry10_1::Geographic { .. } => {
                    GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID
                }
                SpatialReferenceEntry10_1::UnknownCoordinateSystem { .. } => {
                    UNKNOWN_COORDINATE_SYSTEM_CLASS_ID
                }
                other => panic!("应保留新空间参考对象体，读到 {other:?}"),
            };
            assert_eq!(decoded_class, class_id);
            let PropertyValueObjectStream10_1::Geometry(bounds) = bounds else {
                panic!("应读到带空间参考回引的包络 Variant");
            };
            let GeometryBody10_1::Envelope(envelope) = bounds.body() else {
                panic!("应读到包络对象体");
            };
            assert_eq!(
                envelope.spatial_reference,
                Some(SpatialReferenceEntry10_1::BackReference { class_id, key: 72 })
            );
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);
            assert_eq!(refs.len(), 3);

            let mut unknown = bytes.clone();
            unknown[class_offset] ^= 0xff;
            let mut existing = RefTable10_1::new();
            existing.register(70);
            for input in (class_offset..end)
                .map(|cut| &bytes[..cut])
                .chain([unknown.as_slice()])
            {
                let mut reader = Reader::new(input);
                let mut refs = existing.clone();
                assert!(read_domain_slot_10_1(&mut reader, 256, &mut refs).is_err());
                assert_eq!(reader.position(), 0);
                assert_eq!(refs, existing);
            }
        }
    }

    #[test]
    fn coded_domain_label_and_code_use_distinct_string_lengths() {
        use crate::layers::raster_dataset_name::LONG_ARRAY_CLASS_ID;
        use crate::objects::geometry::{GeometryBody10_1, POINT_CLASS_ID};
        use crate::objects::property_set::VariantArrayObjectValues10_1;

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
        fixture.extend(4_u32.to_le_bytes());
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
        fixture.extend(0_u32.to_le_bytes()); // 第三项为空标签。
        fixture.extend(13_u16.to_le_bytes());
        fixture.extend([0; 14]);
        fixture.extend(LONG_ARRAY_CLASS_ID);
        fixture.extend(72_u32.to_le_bytes());
        fixture.extend(1_u16.to_le_bytes());
        fixture.extend(2_i32.to_le_bytes());
        fixture.extend((-7_i32).to_le_bytes());
        fixture.extend(24_i32.to_le_bytes());
        fixture.extend(0_u32.to_le_bytes()); // 第四项为空标签。
        fixture.extend(13_u16.to_le_bytes());
        fixture.extend([0; 14]);
        fixture.extend(POINT_CLASS_ID);
        fixture.extend(73_u32.to_le_bytes());
        let point_start = fixture.len();
        fixture.extend(1_u16.to_le_bytes());
        fixture.extend(20_i32.to_le_bytes());
        fixture.extend(1_i32.to_le_bytes());
        fixture.extend((-0.0_f64).to_le_bytes());
        fixture.extend(42.5_f64.to_le_bytes());

        for cut in 0..fixture.len() {
            let mut reader = Reader::new(&fixture[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_domain_slot_10_1(&mut reader, 64, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let end = fixture.len();
        fixture.push(0x53);
        let mut reader = Reader::new(&fixture);
        let mut refs = RefTable10_1::new();
        let slot = read_domain_slot_10_1(&mut reader, 64, &mut refs).unwrap();
        let DomainValues10_1::Coded(values) = slot.body.unwrap().values else {
            panic!("应读到编码值域");
        };
        assert_eq!(values.len(), 4);
        assert_eq!(
            values[..3],
            [
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
                CodedDomainValue10_1 {
                    label: String::new(),
                    code: PropertyValueObjectStream10_1::ArrayObject {
                        class_id: LONG_ARRAY_CLASS_ID,
                        key: 72,
                        version: 1,
                        values: VariantArrayObjectValues10_1::Longs(vec![-7, 24]),
                    },
                },
            ],
        );
        let PropertyValueObjectStream10_1::Geometry(geometry) = &values[3].code else {
            panic!("应读到点几何代码");
        };
        assert_eq!((geometry.class_id, geometry.key), (POINT_CLASS_ID, 73));
        assert_eq!(geometry.bytes(), &fixture[point_start..end]);
        assert!(matches!(
            geometry.body(),
            GeometryBody10_1::Point(point)
                if point.record.x.unwrap().to_bits() == (-0.0_f64).to_bits()
                    && point.record.y == Some(42.5)
        ));
        assert!((71..=73).all(|key| refs.contains(key)));
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

        // 这些类型不能按普通数组读。第二个值失败时首个值的对象键也要撤销。
        for element_type in [0_u16, 1, 9, 12, 13, 14, 15, 20, 21, 0x4011] {
            let mut invalid = fixture.clone();
            invalid[end - 36..end - 34].copy_from_slice(&(0x2000 | element_type).to_le_bytes());
            let mut input = Reader::new(&invalid);
            let mut keys = RefTable10_1::new();
            keys.register(70);
            let original = keys.clone();
            assert!(read_domain_slot_10_1(&mut input, 4096, &mut keys).is_err());
            assert_eq!(input.position(), 0);
            assert_eq!(keys, original);
        }

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
