//! AGSClient.dll 10.1 的 ArcGIS Server 对象名与连接名。

use super::{
    PropertySetObjectStream10_1, Reader, RefTable10_1, RuleSlot10_1, read_properties_slot,
    read_string_10_1, slot, unsupported_class, version,
};
use crate::stream::native_string::read_bstr_string_10_1;
use std::io;

const AGS_SERVER_OBJECT_NAME: [u8; 16] = [
    0x1c, 0x97, 0x22, 0x6c, 0x50, 0xd4, 0x4d, 0x4b, 0x94, 0x22, 0xee, 0x96, 0xa4, 0x0f, 0xac, 0xc5,
];
const AGS_SERVER_CONNECTION_NAME: [u8; 16] = [
    0x3f, 0x5c, 0xa3, 0xcb, 0xef, 0xed, 0x8f, 0x40, 0x8b, 0x51, 0x51, 0x07, 0x84, 0xc7, 0x8e, 0xb9,
];

#[derive(Debug, Clone, PartialEq)]
pub struct AgsServerObjectName10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub connection: RuleSlot10_1<AgsServerConnectionName10_1>,
    pub extra_strings: Option<[String; 2]>,
    pub last_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgsServerConnectionName10_1 {
    pub version: i16,
    pub properties: RuleSlot10_1<PropertySetObjectStream10_1>,
    pub extra_string: Option<String>,
}

pub(super) fn read_server_object_name_slot(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RuleSlot10_1<AgsServerObjectName10_1>> {
    slot(r, refs, |r, refs, class, _| {
        if class != AGS_SERVER_OBJECT_NAME {
            return Err(unsupported_class("AGSServerObjectName", &class));
        }
        let version = version(r, 3, "AGSServerObjectName")?;
        let strings = [
            read_bstr_string_10_1(r, max)?,
            read_bstr_string_10_1(r, max)?,
            read_bstr_string_10_1(r, max)?,
        ];
        let connection = slot(r, refs, |r, refs, class, _| {
            if class != AGS_SERVER_CONNECTION_NAME {
                return Err(unsupported_class("IAGSServerConnectionName", &class));
            }
            let version = super::version(r, 2, "AGSServerConnectionName")?;
            Ok(AgsServerConnectionName10_1 {
                version,
                properties: read_properties_slot(r, max, refs)?,
                extra_string: (version >= 2)
                    .then(|| read_string_10_1(r, max))
                    .transpose()?,
            })
        })?;
        let extra_strings = if version >= 2 {
            Some([
                read_bstr_string_10_1(r, max)?,
                read_bstr_string_10_1(r, max)?,
            ])
        } else {
            None
        };
        let last_string = (version >= 3)
            .then(|| read_bstr_string_10_1(r, max))
            .transpose()?;
        Ok(AgsServerObjectName10_1 {
            version,
            strings,
            connection,
            extra_strings,
            last_string,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::cache_tile_reader::{
        CacheTileReaderBody10_1, MAP_SERVER_TILE_READER, PROPERTY_SET_CLASS_ID, read_tile_reader,
    };

    #[test]
    fn map_server_reader_consumes_a_new_name_and_connection() {
        let empty_string = [2, 0, 0, 0, 0, 0];
        let bstr = [1, 0, 0, 0, b'x', 0, 0, 0];
        let mut bytes = vec![2, 0, 0];
        bytes.extend([0; 16]);
        bytes.extend(empty_string);
        bytes.extend([0; 48]);
        bytes.extend(AGS_SERVER_OBJECT_NAME);
        bytes.extend(71_u32.to_le_bytes());
        bytes.extend(3_i16.to_le_bytes());
        bytes.extend(bstr.repeat(3));
        bytes.extend(AGS_SERVER_CONNECTION_NAME);
        bytes.extend(72_u32.to_le_bytes());
        bytes.extend(2_i16.to_le_bytes());
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(73_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(empty_string);
        bytes.extend(bstr.repeat(3));
        bytes.extend(0_u16.to_le_bytes());
        for end in 0..bytes.len() {
            assert!(
                read_tile_reader(
                    &mut Reader::new(&bytes[..end]),
                    4096,
                    &mut RefTable10_1::new(),
                    MAP_SERVER_TILE_READER,
                    16,
                )
                .is_err(),
                "错误接受截断的 AGSServerObjectName 槽，长度 {end}"
            );
        }
        bytes.extend(0x5a3e_u16.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let CacheTileReaderBody10_1::MapServer {
            server_name_body: Some(name),
            rest_selector: Some(0),
            ..
        } = read_tile_reader(&mut reader, 4096, &mut refs, MAP_SERVER_TILE_READER, 16).unwrap()
        else {
            panic!("应读取 AGSServerObjectName");
        };
        assert_eq!(name.strings, ["x", "x", "x"]);
        assert_eq!(name.extra_strings.unwrap(), ["x", "x"]);
        assert_eq!(name.last_string.as_deref(), Some("x"));
        let connection = name.connection.body.unwrap();
        assert_eq!(connection.properties.body.unwrap().count, 0);
        assert_eq!(connection.extra_string.as_deref(), Some(""));
        assert!(refs.contains(71) && refs.contains(72) && refs.contains(73));
        assert_eq!(reader.read_u16_le().unwrap(), 0x5a3e);
    }
}
