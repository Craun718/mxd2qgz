//! MaplexAnnotation.dll 与 LabelPlacement.dll 10.1 的地图级标注设置。

use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::{read_bstr_string_10_1, read_string_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const MAPLEX_ANNOTATE_MAP_CLASS_ID: [u8; 16] = [
    8, 0x48, 0x66, 0x20, 0xec, 0x14, 0xd2, 0x11, 0xa2, 0x7e, 8, 0, 9, 0xb6, 0xf2, 0x2b,
];
pub const MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID: [u8; 16] = [
    8, 0x48, 0x66, 0x20, 0xc2, 0xa8, 0xd1, 0xc1, 0xac, 0xdc, 0x17, 8, 0xf9, 0x5c, 0x73, 0x21,
];
const DICTIONARIES: [u8; 16] = [
    8, 0x48, 0x66, 0x20, 0xdb, 0x56, 0x28, 0xbd, 0x67, 0xdc, 2, 0xe3, 0x3d, 0xec, 0xf3, 0x21,
];
const DICTIONARY: [u8; 16] = [
    8, 0x48, 0x66, 0x20, 0xe2, 0x3d, 0xd1, 0x1c, 0x8a, 9, 8, 0xe0, 0x0d, 0xec, 0x73, 0x21,
];
const DICTIONARY_ENTRY: [u8; 16] = [
    8, 0x48, 0x66, 0x20, 0x45, 0xfe, 0x18, 0x90, 0xbe, 0x1f, 0x66, 0xeb, 0x0d, 0xec, 0x73, 0x21,
];
const KEY_NUMBER_GROUPS: [u8; 16] = [
    0xb8, 0x3b, 0x5f, 0x75, 0xfb, 0x63, 0x4b, 0xe5, 0x88, 0x4c, 0x74, 0xf3, 0xdd, 0x2a, 0x8a, 0x81,
];
const KEY_NUMBER_GROUP: [u8; 16] = [
    0x5d, 0xef, 0xdd, 0x66, 0xb3, 0x1a, 0x31, 0x5f, 0x7b, 0xc1, 0x75, 0x5d, 0xde, 0xf3, 0x3d, 0x32,
];

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexSlot10_1<T> {
    pub entry: ObjectEntry10_1,
    pub body: Option<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexCollection10_1<T> {
    pub version: i16,
    pub items: Vec<MaplexSlot10_1<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexAnnotateMap10_1 {
    pub version: i16,
    pub v2_prefix: Option<(i32, u16)>,
    pub properties: Option<MaplexSlot10_1<MaplexOverposterProperties10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexOverposterProperties10_1 {
    pub version: i16,
    pub long_38: i32,
    pub short_34: u16,
    pub dictionaries: Option<MaplexSlot10_1<MaplexCollection10_1<MaplexDictionary10_1>>>,
    pub v3: Option<MaplexOverposterV3Fields10_1>,
    pub v4_v5_long: Option<i32>,
    pub short_1c: Option<u16>,
    pub v5_shorts: Option<[u16; 2]>,
    pub key_number_groups: Option<MaplexSlot10_1<MaplexCollection10_1<MaplexKeyNumberGroup10_1>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexOverposterV3Fields10_1 {
    pub short_2c: u16,
    pub long_28: i32,
    pub double_24: u64,
    pub color: ColorSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexDictionary10_1 {
    pub version: i16,
    pub entries: MaplexDictionaryEntries10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaplexDictionaryEntries10_1 {
    LegacyArrays {
        first: Vec<String>,
        second: Vec<String>,
    },
    Objects {
        items: Vec<MaplexSlot10_1<MaplexDictionaryEntry10_1>>,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexDictionaryEntry10_1 {
    pub version: i16,
    pub strings: [String; 2],
    pub long: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexKeyNumberGroup10_1 {
    pub version: i16,
    pub name: String,
    pub longs: [i32; 3],
    pub second_string: String,
    pub last_long: i32,
}

pub(super) fn read_annotate_map(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MaplexAnnotateMap10_1> {
    let version = version(reader, 4, "MaplexAnnotateMap")?;
    let v2_prefix = (version == 2)
        .then(|| -> io::Result<_> { Ok((long(reader)?, reader.read_u16_le()?)) })
        .transpose()?;
    let properties = (version <= 3)
        .then(|| {
            read_slot(
                reader,
                limit,
                refs,
                MAPLEX_OVERPOSTER_PROPERTIES_CLASS_ID,
                read_overposter_properties,
            )
        })
        .transpose()?;
    Ok(MaplexAnnotateMap10_1 {
        version,
        v2_prefix,
        properties,
    })
}

pub(super) fn read_overposter_properties(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MaplexOverposterProperties10_1> {
    let version = version(reader, 7, "MaplexOverposterProperties")?;
    let long_38 = long(reader)?;
    let short_34 = reader.read_u16_le()?;
    let dictionaries = (version >= 2)
        .then(|| {
            read_slot(reader, limit, refs, DICTIONARIES, |r, limit, refs| {
                read_collection(
                    r,
                    limit,
                    refs,
                    "MaplexDictionaries",
                    DICTIONARY,
                    read_dictionary,
                )
            })
        })
        .transpose()?;
    let v3 = (version >= 3)
        .then(|| -> io::Result<_> {
            let short_2c = reader.read_u16_le()?;
            let long_28 = long(reader)?;
            let double_24 = reader.read_f64_le()?.to_bits();
            let color = read_color_slot_10_1(reader, limit, refs)?;
            if matches!(color, ColorSlot10_1::Unsupported { .. }) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "不支持的 Maplex 颜色对象",
                ));
            }
            Ok(MaplexOverposterV3Fields10_1 {
                short_2c,
                long_28,
                double_24,
                color,
            })
        })
        .transpose()?;
    let v4_v5_long = matches!(version, 4 | 5).then(|| long(reader)).transpose()?;
    let short_1c = (version >= 4).then(|| reader.read_u16_le()).transpose()?;
    let v5_shorts = (version >= 5)
        .then(|| -> io::Result<_> { Ok([reader.read_u16_le()?, reader.read_u16_le()?]) })
        .transpose()?;
    let key_number_groups = (version >= 7)
        .then(|| {
            read_slot(reader, limit, refs, KEY_NUMBER_GROUPS, |r, limit, refs| {
                read_collection(
                    r,
                    limit,
                    refs,
                    "MaplexKeyNumberGroups",
                    KEY_NUMBER_GROUP,
                    read_key_number_group,
                )
            })
        })
        .transpose()?;
    Ok(MaplexOverposterProperties10_1 {
        version,
        long_38,
        short_34,
        dictionaries,
        v3,
        v4_v5_long,
        short_1c,
        v5_shorts,
        key_number_groups,
    })
}

fn read_dictionary(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MaplexDictionary10_1> {
    let version = version(reader, 2, "MaplexDictionary")?;
    let entries = if version == 1 {
        // 旧体先写完第一列，再写第二列；两列使用同一个数量字段。
        let count = read_count(reader, limit, 8)?;
        let first = (0..count)
            .map(|_| read_bstr_string_10_1(reader, limit))
            .collect::<io::Result<_>>()?;
        let second = (0..count)
            .map(|_| read_bstr_string_10_1(reader, limit))
            .collect::<io::Result<_>>()?;
        MaplexDictionaryEntries10_1::LegacyArrays { first, second }
    } else {
        let items = read_items(reader, limit, refs, DICTIONARY_ENTRY, |r, limit, _| {
            Ok(MaplexDictionaryEntry10_1 {
                version: self::version(r, 1, "MaplexDictionaryEntry")?,
                strings: [read_string_10_1(r, limit)?, read_string_10_1(r, limit)?],
                long: long(r)?,
            })
        })?;
        let name = read_string_10_1(reader, limit)?;
        MaplexDictionaryEntries10_1::Objects { items, name }
    };
    Ok(MaplexDictionary10_1 { version, entries })
}

fn read_key_number_group(
    reader: &mut Reader<'_>,
    limit: usize,
    _refs: &mut RefTable10_1,
) -> io::Result<MaplexKeyNumberGroup10_1> {
    Ok(MaplexKeyNumberGroup10_1 {
        version: version(reader, 1, "MaplexKeyNumberGroup")?,
        name: read_string_10_1(reader, limit)?,
        longs: [long(reader)?, long(reader)?, long(reader)?],
        second_string: read_string_10_1(reader, limit)?,
        last_long: long(reader)?,
    })
}

fn read_collection<T>(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    name: &str,
    class: [u8; 16],
    read_body: fn(&mut Reader<'_>, usize, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<MaplexCollection10_1<T>> {
    let version = version(reader, 1, name)?;
    let items = read_items(reader, limit, refs, class, read_body)?;
    Ok(MaplexCollection10_1 { version, items })
}

fn read_items<T>(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    class: [u8; 16],
    read_body: fn(&mut Reader<'_>, usize, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<Vec<MaplexSlot10_1<T>>> {
    let count = read_count(reader, limit, 16)?;
    (0..count)
        .map(|_| read_slot(reader, limit, refs, class, read_body))
        .collect()
}

fn read_slot<T>(
    reader: &mut Reader<'_>,
    limit: usize,
    refs: &mut RefTable10_1,
    class: [u8; 16],
    read_body: fn(&mut Reader<'_>, usize, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<MaplexSlot10_1<T>> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, .. } if class_id == class => None,
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == class => {
            Some(read_body(reader, limit, refs)?)
        }
        _ => return Err(super::map_frame::unsupported("Maplex", entry)),
    };
    Ok(MaplexSlot10_1 { entry, body })
}

fn read_count(reader: &mut Reader<'_>, limit: usize, min_width: usize) -> io::Result<usize> {
    usize::try_from(long(reader)?)
        .ok()
        .filter(|count| *count <= reader.remaining().min(limit) / min_width)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Maplex 集合计数超出范围"))
}

fn long(reader: &mut Reader<'_>) -> io::Result<i32> {
    Ok(reader.read_u32_le()? as i32)
}

fn version(reader: &mut Reader<'_>, max: i16, name: &str) -> io::Result<i16> {
    let number = reader.read_u16_le()? as i16;
    if number > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{name} 版本 {number} 超界"),
        ));
    }
    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maplex_overposter_version_gates_leave_the_next_field_unread() {
        for v in 0_i16..=7 {
            let mut bytes = v.to_le_bytes().to_vec();
            bytes.extend([0; 4 + 2]);
            if v >= 2 {
                bytes.extend([0; 16]);
            }
            if v >= 3 {
                bytes.extend([0; 2 + 4 + 8 + 16]);
            }
            if matches!(v, 4 | 5) {
                bytes.extend((-42_i32).to_le_bytes());
            }
            if v >= 4 {
                bytes.extend(43_u16.to_le_bytes());
            }
            if v >= 5 {
                bytes.extend(44_u16.to_le_bytes());
                bytes.extend(45_u16.to_le_bytes());
            }
            if v >= 7 {
                bytes.extend([0; 16]);
            }
            bytes.extend(0x1357_u16.to_le_bytes());

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let body = read_overposter_properties(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(body.version, v);
            assert_eq!(body.dictionaries.is_some(), v >= 2);
            assert_eq!(body.v3.is_some(), v >= 3);
            assert_eq!(body.v4_v5_long, matches!(v, 4 | 5).then_some(-42));
            assert_eq!(body.short_1c, (v >= 4).then_some(43));
            assert_eq!(body.v5_shorts, (v >= 5).then_some([44, 45]));
            assert_eq!(body.key_number_groups.is_some(), v >= 7);
            assert_eq!(reader.read_u16_le().unwrap(), 0x1357);
            assert!(refs.is_empty());
        }
    }
}
