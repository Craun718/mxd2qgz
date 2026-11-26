//! TrackingWorkspace.dll 10.1 的 TemporalRecordSet 内联持久化体。

use crate::stream::native_string::read_atl_bstr_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalRecordSet10_1 {
    pub value: i32,
    pub settings_value: i32,
    pub first_text: Option<Vec<u8>>,
    pub second_text: Option<Vec<u8>>,
    pub settings_flag: i16,
    pub final_text: Option<Vec<u8>>,
}

/// 从外层图层的试读起点读取两个连续的 v1 记录。
/// 仅在完整读出后推进游标，便于外层按原生试读语义处理失败。
pub fn read_temporal_record_set_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<TemporalRecordSet10_1> {
    let mut trial = reader.clone();
    read_v1(&mut trial, "TemporalRecordSet")?;
    let value = trial.read_u32_le()? as i32;
    read_v1(&mut trial, "TemporalRecordSet 子记录")?;
    let body = TemporalRecordSet10_1 {
        value,
        settings_value: trial.read_u32_le()? as i32,
        first_text: read_atl_bstr_10_1(&mut trial, max_bytes)?,
        second_text: read_atl_bstr_10_1(&mut trial, max_bytes)?,
        settings_flag: trial.read_u16_le()? as i16,
        final_text: read_atl_bstr_10_1(&mut trial, max_bytes)?,
    };
    *reader = trial;
    Ok(body)
}

fn read_v1(reader: &mut Reader<'_>, record: &str) -> io::Result<()> {
    let offset = reader.position();
    let version = reader.read_u16_le()?;
    if version == 1 {
        return Ok(());
    }
    Err(io::Error::new(
        ErrorKind::InvalidData,
        format!("{record} offset 0x{offset:x}: 不支持的版本 {version}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend((-4_i32).to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(12_i32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend([0, 0]);
        bytes.extend((-1_i16).to_le_bytes());
        bytes.extend(5_u32.to_le_bytes());
        bytes.extend([b'a', 0, b'b', 0, 0]);
        bytes
    }

    #[test]
    fn reads_inline_record_and_leaves_following_field() {
        let mut bytes = record();
        bytes.extend(0xabcd_u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_temporal_record_set_10_1(&mut reader, 5).unwrap(),
            TemporalRecordSet10_1 {
                value: -4,
                settings_value: 12,
                first_text: None,
                second_text: Some(Vec::new()),
                settings_flag: -1,
                final_text: Some(b"a\0b".to_vec()),
            }
        );
        assert_eq!(reader.read_u16_le().unwrap(), 0xabcd);
    }

    #[test]
    fn invalid_records_leave_probe_position_unchanged() {
        let original = record();
        for end in 0..original.len() {
            let mut reader = Reader::new(&original[..end]);
            assert!(read_temporal_record_set_10_1(&mut reader, 5).is_err());
            assert_eq!(reader.position(), 0);
        }
        for version_offset in [0, 6] {
            let mut bytes = original.clone();
            bytes[version_offset] = 2;
            let mut reader = Reader::new(&bytes);
            assert!(read_temporal_record_set_10_1(&mut reader, 5).is_err());
            assert_eq!(reader.position(), 0);
        }
        let mut reader = Reader::new(&original);
        assert!(read_temporal_record_set_10_1(&mut reader, 4).is_err());
        assert_eq!(reader.position(), 0);
    }
}
