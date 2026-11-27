//! 读取 10.1 的 SystemCore StrArray 与 FeatureDataElements Names。

use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io;

pub const STR_ARRAY_CLASS_ID: [u8; 16] = [
    0x65, 0x20, 0xf9, 0xa7, 0xce, 0x36, 0xb6, 0x47, 0xa4, 0x63, 0x47, 0x63, 0xda, 0x94, 0x7c, 0xc2,
];

pub const NAMES_CLASS_ID: [u8; 16] = [
    0xaf, 0xc7, 0x41, 0xe1, 0x30, 0x1c, 0x67, 0x4b, 0x99, 0xa1, 0x2d, 0xdd, 0x2f, 0xf2, 0xc0, 0x4d,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringArraySlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<(u16, Vec<String>)>,
}

/// 读取 IStringArray 槽。宿主负责暂存游标与引用表。
pub(crate) fn read_string_array_slot_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<StringArraySlot10_1> {
    let mut entry = read_object_entry_10_1(reader, refs)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = refs.registered_class(*key)
    {
        if !matches!(registered, STR_ARRAY_CLASS_ID | NAMES_CLASS_ID) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("IStringArray 回引键 {key} 的登记类不受支持"),
            ));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::BackReference {
            class_id: STR_ARRAY_CLASS_ID | NAMES_CLASS_ID,
            key,
        } => refs.string_array(key, max),
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (STR_ARRAY_CLASS_ID | NAMES_CLASS_ID),
            key,
        } => {
            let start = reader.position();
            let body = read_string_array_body(reader, max, class_id)?;
            refs.mark_string_array(key, body.0, reader.position() - start, &body.1);
            Some(body)
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "IStringArray 未支持的新对象类 {}",
                    crate::storage::class_id_hex(&class_id),
                ),
            ));
        }
    };
    Ok(StringArraySlot10_1 { entry, body })
}

pub(crate) fn read_string_array_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
) -> io::Result<(u16, Vec<String>)> {
    read_string_array_body(reader, max, STR_ARRAY_CLASS_ID)
}

fn read_string_array_body(
    reader: &mut Reader<'_>,
    max: usize,
    class_id: [u8; 16],
) -> io::Result<(u16, Vec<String>)> {
    let mut body = reader.clone().read_region(reader.remaining().min(max))?;
    let version = body.read_u16_le()?;
    // Names 使用无符号比较，StrArray 使用有符号比较。
    if if class_id == NAMES_CLASS_ID {
        version > 1
    } else {
        version as i16 > 1
    } {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 IStringArray 版本 {version}"),
        ));
    }
    let count = (body.read_u32_le()? as i32).max(0) as usize;
    if count > body.remaining() / 6 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "IStringArray 数量超出可读范围",
        ));
    }
    let strings = (0..count)
        .map(|_| read_string_10_1(&mut body, max))
        .collect::<io::Result<_>>()?;
    reader.read_bytes(body.position())?;
    Ok((version, strings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_reads_strings_and_recovers_registered_back_reference() {
        let mut bytes = NAMES_CLASS_ID.to_vec();
        bytes.extend(19_u32.to_le_bytes());
        bytes.extend([1, 0, 1, 0, 0, 0, 4, 0, 0, 0, b'A', 0, 0, 0]);
        let end = bytes.len();
        bytes.extend(STR_ARRAY_CLASS_ID);
        bytes.extend(19_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let first = read_string_array_slot_10_1(&mut reader, 1024, &mut refs).unwrap();
        assert_eq!(first.body, Some((1, vec!["A".into()])));
        assert_eq!(reader.position(), end);
        let second = read_string_array_slot_10_1(&mut reader, 1024, &mut refs).unwrap();
        assert_eq!(
            second.entry,
            ObjectEntry10_1::BackReference {
                class_id: NAMES_CLASS_ID,
                key: 19,
            }
        );
        assert_eq!(second.body, first.body);
        assert_eq!(reader.remaining(), 0);

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_string_array_slot_10_1(&mut reader, 1024, &mut refs).is_err());
        }
        bytes[20..22].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(
            read_string_array_slot_10_1(&mut Reader::new(&bytes), 1024, &mut RefTable10_1::new(),)
                .is_err()
        );
    }
}
