//! Desktop 10.8 AfCore VariantStream 已知前缀的严格读取子集。
//!
//! 字符串前缀是 UTF-16 码元数量，随后是 `n * 2 + 2` 字节载荷；这与
//! AfCore `String::Read` 的含终止零字节长度不同。证据和边界见
//! 本模块不解析，也不定位 MXD 字段归属。

use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// 保留 VariantStream 已确认分支的原始读取结果。
///
/// 固定宽度值不转换成有符号数、浮点数或布尔语义；调用者按字段证据解释。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantValue10_8 {
    Empty,
    Null,
    Raw1([u8; 1]),
    Raw2([u8; 2]),
    Raw4([u8; 4]),
    Raw8([u8; 8]),
    String(String),
}

/// 从已确认的 VariantStream 值起点读取 10.8 已知前缀，失败时不推进游标。
///
/// `max_string_units` 限制 4 字节前缀声明的 UTF-16 码元数，`0` 允许空字符串。
/// 嵌入 NUL 是 BSTR 型载荷的自然内容；当前证据只确认末尾必须有一个
/// UTF-16 终止零，是否所有原生读取路径都容忍嵌入 NUL 仍未在运行行为中验证。
pub fn read_value_10_8(
    reader: &mut Reader<'_>,
    max_string_units: u32,
) -> io::Result<VariantValue10_8> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Variant 值 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let value_type = trial.read_u16_le()?;
    let value = match value_type {
        0 => VariantValue10_8::Empty,
        1 => VariantValue10_8::Null,
        2 | 11 => VariantValue10_8::Raw2(trial.read_bytes(2)?.try_into().unwrap()),
        3 | 4 => VariantValue10_8::Raw4(trial.read_bytes(4)?.try_into().unwrap()),
        5 | 7 => VariantValue10_8::Raw8(trial.read_bytes(8)?.try_into().unwrap()),
        8 => {
            let units = trial.read_u32_le()?;
            if units > max_string_units {
                return Err(invalid("字符串码元数超过调用者资源限额"));
            }
            // 空字符串只写入 VT_BSTR 和零长度；终止零本身计入 n。
            if units == 0 {
                VariantValue10_8::String(String::new())
            } else {
                let payload_length = units
                    .checked_mul(2)
                    .and_then(|length| length.checked_add(2))
                    .and_then(|length| usize::try_from(length).ok())
                    .ok_or_else(|| invalid("字符串载荷长度不能由本平台表示"))?;
                let payload = trial.read_bytes(payload_length)?;
                let content_length = payload_length - 2;
                if payload[content_length..] != [0, 0] {
                    return Err(invalid("缺少末尾 UTF-16LE 终止零"));
                }
                let value = Reader::new(&payload[..content_length])
                    .read_utf16_le(content_length)
                    .map_err(|_| invalid("非法 UTF-16LE 内容"))?;
                VariantValue10_8::String(value)
            }
        }
        9 | 13 => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "Variant 对象分支依赖原生 IStream QI/COM 上下文，边界未闭环",
            ));
        }
        17 => VariantValue10_8::Raw1(trial.read_bytes(1)?.try_into().unwrap()),
        _ => {
            return Err(invalid("原生 Read 分支返回 E_FAIL 的未知 VARTYPE"));
        }
    };
    *reader = trial;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_bytes(value_type: u16, payload: &[u8]) -> Vec<u8> {
        value_type
            .to_le_bytes()
            .into_iter()
            .chain(payload.iter().copied())
            .collect()
    }

    fn bstr_payload_bytes(value: &str) -> Vec<u8> {
        let units: Vec<u16> = value.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(4 + units.len() * 2);
        bytes.extend_from_slice(&(units.len() as u32).to_le_bytes());
        for unit in units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0]);
        bytes
    }

    #[test]
    fn reads_fixed_width_bit_patterns_and_keeps_tail() {
        let cases = [
            (0u16, Vec::new(), VariantValue10_8::Empty),
            (1, Vec::new(), VariantValue10_8::Null),
            (2, vec![0x34, 0x12], VariantValue10_8::Raw2([0x34, 0x12])),
            (11, vec![0xff, 0xff], VariantValue10_8::Raw2([0xff, 0xff])),
            (
                3,
                vec![0x78, 0x56, 0x34, 0x12],
                VariantValue10_8::Raw4([0x78, 0x56, 0x34, 0x12]),
            ),
            (
                4,
                vec![0x00, 0x00, 0x00, 0x80],
                VariantValue10_8::Raw4([0x00, 0x00, 0x00, 0x80]),
            ),
            (
                5,
                [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x7f].to_vec(),
                VariantValue10_8::Raw8([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x7f]),
            ),
            (
                7,
                [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff].to_vec(),
                VariantValue10_8::Raw8([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            ),
            (17, vec![0x9c], VariantValue10_8::Raw1([0x9c])),
        ];
        for (value_type, payload, expected) in cases {
            let mut bytes = value_bytes(value_type, &payload);
            bytes.extend_from_slice(b"TAIL");
            let mut reader = Reader::new(&bytes);
            assert_eq!(read_value_10_8(&mut reader, u32::MAX).unwrap(), expected);
            assert_eq!(reader.read_bytes(4).unwrap(), b"TAIL");
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn reads_empty_ascii_chinese_and_surrogate_strings() {
        let bytes = value_bytes(8, &[0, 0, 0, 0]);
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_value_10_8(&mut reader, 0).unwrap(),
            VariantValue10_8::String(String::new())
        );
        assert_eq!(reader.remaining(), 0);

        for (value, max_units) in [("Maps", 4u32), ("地图", 2), ("😀", 2)] {
            let bytes = value_bytes(8, &bstr_payload_bytes(value));
            let mut reader = Reader::new(&bytes);
            assert_eq!(
                read_value_10_8(&mut reader, max_units).unwrap(),
                VariantValue10_8::String(value.to_string())
            );
            assert_eq!(reader.remaining(), 0);
        }

        // BSTR 型载荷允许嵌入 NUL；只有末尾两个零是原生写入的终止符。
        let bytes = value_bytes(8, &bstr_payload_bytes("A\0B"));
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_value_10_8(&mut reader, 3).unwrap(),
            VariantValue10_8::String("A\0B".to_string())
        );
    }

    #[test]
    fn string_failures_do_not_advance_cursor() {
        let valid = value_bytes(8, &bstr_payload_bytes("地图"));
        let content_units = 2usize;
        let cases: Vec<(Vec<u8>, ErrorKind, &str)> = vec![
            (
                value_bytes(8, &[2, 0]),
                ErrorKind::UnexpectedEof,
                "前缀截断",
            ),
            (
                value_bytes(8, &(content_units as u32).to_le_bytes()),
                ErrorKind::UnexpectedEof,
                "载荷截断",
            ),
            (
                value_bytes(8, &(content_units as u32 + 1).to_le_bytes()),
                ErrorKind::InvalidData,
                "声明超过限额",
            ),
            (
                value_bytes(8, &(content_units as u32).to_le_bytes())
                    .into_iter()
                    .chain([0x41, 0x00, 0x42, 0x00, 0x43, 0x00])
                    .collect(),
                ErrorKind::InvalidData,
                "声明载荷完整但缺少终止零",
            ),
            (
                value_bytes(8, &(content_units as u32).to_le_bytes())
                    .into_iter()
                    .chain([0x41, 0x00, 0x00, 0xd8, 0x00, 0x00])
                    .collect(),
                ErrorKind::InvalidData,
                "非法孤立高代理",
            ),
            (
                value_bytes(8, &(content_units as u32).to_le_bytes())
                    .into_iter()
                    .chain([0x41, 0x00, 0x42, 0x00])
                    .collect(),
                ErrorKind::UnexpectedEof,
                "终止零也被截断",
            ),
        ];
        for (bytes, expected_kind, label) in cases {
            let mut reader = Reader::new(&bytes);
            let error = read_value_10_8(&mut reader, 2).unwrap_err();
            assert_eq!(error.kind(), expected_kind, "{label}");
            assert_eq!(reader.position(), 0, "{label}");
        }

        // 字符串失败后同一个 Reader 可以从事务边界重试成功值。
        let mut bytes = valid.clone();
        bytes.extend_from_slice(b"TAIL");
        let mut reader = Reader::new(&bytes);
        assert!(read_value_10_8(&mut reader, 1).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(
            read_value_10_8(&mut reader, 2).unwrap(),
            VariantValue10_8::String("地图".to_string())
        );
        assert_eq!(reader.read_bytes(4).unwrap(), b"TAIL");
    }

    #[test]
    fn rejects_resource_limit_and_declared_length_overflow() {
        let bytes = value_bytes(8, &[1, 0, 0, 0, 0x41, 0, 0, 0]);
        let mut reader = Reader::new(&bytes);
        assert!(read_value_10_8(&mut reader, 0).is_err());
        assert_eq!(reader.position(), 0);

        // u32::MAX 个码元使 2n+2 在 u32 内溢出；在任何读取分配前拒绝。
        let bytes = value_bytes(8, &[0xff, 0xff, 0xff, 0xff]);
        let mut reader = Reader::new(&bytes);
        assert!(read_value_10_8(&mut reader, u32::MAX).is_err());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn object_branch_is_explicitly_unsupported_without_advancing() {
        for value_type in [9u16, 13] {
            let mut bytes = value_bytes(value_type, &[1, 2, 3, 4]);
            bytes.extend_from_slice(b"TAIL");
            let mut reader = Reader::new(&bytes);
            let error = read_value_10_8(&mut reader, 8).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert_eq!(reader.remaining(), bytes.len());
        }
    }

    #[test]
    fn unknown_vartypes_and_truncation_do_not_advance() {
        let unknown_types = [6u16, 10, 12, 14, 15, 16, 18, 0xffff];
        for value_type in unknown_types {
            let mut bytes = value_bytes(value_type, &[1, 2, 3, 4]);
            bytes.extend_from_slice(b"TAIL");
            let mut reader = Reader::new(&bytes);
            let error = read_value_10_8(&mut reader, 8).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert_eq!(reader.position(), 0);
            assert_eq!(reader.remaining(), bytes.len());
        }

        // 每个固定宽度分支在其声明载荷完成前失败，游标回到入口。
        for (value_type, complete_length) in [(2u16, 2), (3, 4), (5, 8), (17, 1)] {
            for length in 0..complete_length {
                let payload = vec![0xa5; length];
                let bytes = value_bytes(value_type, &payload);
                let mut reader = Reader::new(&bytes);
                assert!(read_value_10_8(&mut reader, 8).is_err());
                assert_eq!(reader.position(), 0);
            }
        }

        let mut reader = Reader::new(&[8, 0, 2, 0]);
        assert!(read_value_10_8(&mut reader, 8).is_err());
        assert_eq!(reader.position(), 0);
    }
}
