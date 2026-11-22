//! ArcMap 10.1 DocumentInfo 对 /Metadata 的局部读取规则。

use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

/// 由宿主提供布局。旧式场景布局还依赖 DocumentInfo 的版本成员。
#[derive(Debug, Clone, Copy)]
pub enum MetadataLayout10_1 {
    Versioned,
    LegacyMap,
    LegacyScene { has_tail: bool },
}

#[derive(Debug, PartialEq)]
pub struct DocumentMetadata10_1 {
    pub version: Option<u16>,
    pub title: String,
    pub author: String,
    pub comments: String,
    pub subject: String,
    pub category: String,
    pub keywords: String,
    pub hyperlink_base: String,
    pub credits: Option<String>,
    pub path: Option<String>,
    pub relative_paths: Option<bool>,
    /// 按文件顺序保留尚未核对语义的字符串与数值。
    pub unknown_strings: Vec<String>,
    pub unknown_words: Vec<u32>,
    pub date_bits: Option<[u64; 3]>,
}

/// 读取独立 Metadata 流；要求字段恰好消费完，`max_string_bytes` 为单串上限。
pub fn read_document_metadata_10_1(
    bytes: &[u8],
    layout: MetadataLayout10_1,
    max_string_bytes: usize,
) -> io::Result<DocumentMetadata10_1> {
    let mut reader = Reader::new(bytes);
    let version = match layout {
        MetadataLayout10_1::Versioned => {
            let version = reader.read_u16_le()?;
            if version > 4 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Metadata 版本不受支持",
                ));
            }
            Some(version)
        }
        _ => None,
    };
    let mut metadata = DocumentMetadata10_1 {
        version,
        title: read_string_10_1(&mut reader, max_string_bytes)?,
        author: read_string_10_1(&mut reader, max_string_bytes)?,
        comments: read_string_10_1(&mut reader, max_string_bytes)?,
        subject: read_string_10_1(&mut reader, max_string_bytes)?,
        category: read_string_10_1(&mut reader, max_string_bytes)?,
        keywords: read_string_10_1(&mut reader, max_string_bytes)?,
        hyperlink_base: String::new(),
        credits: None,
        path: None,
        relative_paths: None,
        unknown_strings: Vec::new(),
        unknown_words: Vec::new(),
        date_bits: None,
    };
    if !matches!(layout, MetadataLayout10_1::LegacyScene { .. }) {
        metadata.unknown_words.push(reader.read_u32_le()?);
    }
    metadata.hyperlink_base = read_string_10_1(&mut reader, max_string_bytes)?;
    if !matches!(layout, MetadataLayout10_1::LegacyScene { .. }) {
        metadata
            .unknown_strings
            .push(read_string_10_1(&mut reader, max_string_bytes)?);
    }
    if let Some(version) = version {
        metadata.credits = Some(read_string_10_1(&mut reader, max_string_bytes)?);
        if version < 2 {
            metadata
                .unknown_strings
                .push(read_string_10_1(&mut reader, max_string_bytes)?);
        }
        metadata.path = Some(read_string_10_1(&mut reader, max_string_bytes)?);
        if version < 2 {
            metadata
                .unknown_strings
                .push(read_string_10_1(&mut reader, max_string_bytes)?);
        }
        let mut dates = [0; 3];
        for date in &mut dates {
            *date = reader.read_f64_le()?.to_bits();
        }
        metadata.date_bits = Some(dates);
        if version >= 3 {
            metadata.relative_paths = Some(reader.read_u32_le()? != 0);
        }
        if version >= 4 {
            metadata
                .unknown_strings
                .push(read_string_10_1(&mut reader, max_string_bytes)?);
        }
    } else if matches!(layout, MetadataLayout10_1::LegacyScene { has_tail: true }) {
        metadata.relative_paths = Some(reader.read_u32_le()? != 0);
        metadata.unknown_words.push(reader.read_u32_le()?);
    }
    if reader.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Metadata 流有多余字节",
        ));
    }
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(bytes: &mut Vec<u8>, value: &str) {
        let units: Vec<_> = value.encode_utf16().chain([0]).collect();
        bytes.extend(((units.len() * 2) as u32).to_le_bytes());
        for unit in units {
            bytes.extend(unit.to_le_bytes());
        }
    }

    #[test]
    fn reads_versioned_and_legacy_metadata_to_stream_end() {
        for layout in [
            MetadataLayout10_1::Versioned,
            MetadataLayout10_1::LegacyMap,
            MetadataLayout10_1::LegacyScene { has_tail: true },
        ] {
            let mut bytes = Vec::new();
            if matches!(layout, MetadataLayout10_1::Versioned) {
                bytes.extend(4_u16.to_le_bytes());
            }
            for value in ["标题", "作者", "", "", "", "关键字"] {
                string(&mut bytes, value);
            }
            if !matches!(layout, MetadataLayout10_1::LegacyScene { .. }) {
                bytes.extend(1_u32.to_le_bytes());
            }
            string(&mut bytes, "base");
            match layout {
                MetadataLayout10_1::Versioned => {
                    for value in ["raw", "credits", "path"] {
                        string(&mut bytes, value);
                    }
                    bytes.extend([0; 24]);
                    bytes.extend(1_u32.to_le_bytes());
                    string(&mut bytes, "tail");
                }
                MetadataLayout10_1::LegacyMap => string(&mut bytes, "raw"),
                MetadataLayout10_1::LegacyScene { .. } => bytes.extend([0; 8]),
            }
            let metadata = read_document_metadata_10_1(&bytes, layout, 128).unwrap();
            assert_eq!(metadata.title, "标题");
            assert_eq!(metadata.hyperlink_base, "base");
            assert_eq!(
                metadata.relative_paths,
                match layout {
                    MetadataLayout10_1::Versioned => Some(true),
                    MetadataLayout10_1::LegacyMap => None,
                    MetadataLayout10_1::LegacyScene { .. } => Some(false),
                }
            );
            for end in 0..bytes.len() {
                assert!(read_document_metadata_10_1(&bytes[..end], layout, 128).is_err());
            }
            bytes.push(0);
            assert!(read_document_metadata_10_1(&bytes, layout, 128).is_err());
        }
    }
}
