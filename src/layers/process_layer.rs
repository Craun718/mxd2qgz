//! GpObjects.dll 10.1 的 ProcessLayer 持久化对象体。

use super::group_layer::{GROUP_LAYER_CLASS_ID, GroupLayerPrefix10_1};
use super::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const PROCESS_LAYER_CLASS_ID: [u8; 16] = [
    0x16, 0x2e, 0xe9, 0xd6, 0x56, 0xe6, 0x19, 0x41, 0xb8, 0xe1, 0x88, 0x57, 0x6f, 0x68, 0x33, 0xe6,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessLayer10_1 {
    pub version: u16,
    pub group_key: u32,
    pub group: GroupLayerPrefix10_1,
    /// 单独的进程流，保留原始字节。
    pub process_stream: Vec<u8>,
    pub trailing_short: u16,
}

pub(crate) fn read_process_layer_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ProcessLayer10_1> {
    let mut trial = reader.clone();
    let mut refs = table.clone();
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ProcessLayer 版本超界",
        ));
    }
    let ObjectEntry10_1::NewObject {
        class_id: GROUP_LAYER_CLASS_ID,
        key: group_key,
    } = read_object_entry_10_1(&mut trial, &mut refs)?
    else {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "ProcessLayer 的 IGroupLayer 槽未定义支持的 GroupLayer",
        ));
    };
    let LayerBodyPrefix10_1::GroupLayer(group) =
        read_complete_layer_body_10_1(&mut trial, GROUP_LAYER_CLASS_ID, max_bytes, &mut refs)?
    else {
        unreachable!("GroupLayer 类标识对应 GroupLayer 对象体");
    };
    // 原生将该段读入独立流后加载进程对象；其内部键不加入 Map 对象流。
    let len = trial.read_u32_le()? as usize;
    if len > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ProcessLayer 进程流超过限额",
        ));
    }
    let process_stream = trial.read_bytes(len)?.to_vec();
    let trailing_short = trial.read_u16_le()?;
    *reader = trial;
    *table = refs;
    Ok(ProcessLayer10_1 {
        version,
        group_key,
        group: *group,
        process_stream,
        trailing_short,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_group_and_process_stream_transactionally() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(GROUP_LAYER_CLASS_ID);
        bytes.extend(42_u32.to_le_bytes());
        // GroupLayer v1，空名称、三个 short、两个 double、零子层。
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend([0; 2 + 6 + 16 + 4]);
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend([0xde, 0xad, 0xff]);
        bytes.extend(0xffff_u16.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_process_layer_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.group_key, 42);
        assert!(parsed.group.children.is_empty());
        assert_eq!(parsed.process_stream, [0xde, 0xad, 0xff]);
        assert_eq!(parsed.trailing_short, 0xffff);
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(42));

        let original = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = original.clone();
            assert!(read_process_layer_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, original);
        }
        let len_offset = bytes.len() - 9;
        bytes[len_offset..len_offset + 4].copy_from_slice(&4097_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = original.clone();
        let err = read_process_layer_10_1(&mut reader, 4096, &mut refs).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);
    }
}
