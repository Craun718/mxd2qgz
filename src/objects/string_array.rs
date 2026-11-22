//! SystemCore.dll 10.1 的 StrArray，见 docs/mxd-10.1-raster-layer-tail.md。

use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io;

pub const STR_ARRAY_CLASS_ID: [u8; 16] = [
    0x65, 0x20, 0xf9, 0xa7, 0xce, 0x36, 0xb6, 0x47, 0xa4, 0x63, 0x47, 0x63, 0xda, 0x94, 0x7c, 0xc2,
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
        if registered != STR_ARRAY_CLASS_ID {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("IStringArray 回引键 {key} 的登记类不受支持"),
            ));
        }
        *class_id = registered;
    }
    let body = match entry {
        ObjectEntry10_1::BackReference {
            class_id: STR_ARRAY_CLASS_ID,
            key,
        } => refs.string_array(key, max),
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: STR_ARRAY_CLASS_ID,
            key,
        } => {
            let start = reader.position();
            let body = read_string_array_body_10_1(reader, max)?;
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
    let mut body = reader.clone().read_region(reader.remaining().min(max))?;
    let version = body.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("未支持的 StrArray 版本 {version}"),
        ));
    }
    let count = (body.read_u32_le()? as i32).max(0) as usize;
    if count > body.remaining() / 6 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "StrArray 数量超出可读范围",
        ));
    }
    let strings = (0..count)
        .map(|_| read_string_10_1(&mut body, max))
        .collect::<io::Result<_>>()?;
    reader.read_bytes(body.position())?;
    Ok((version, strings))
}
