//! ArcMap 10.1 DisplayCore SymbolCollection 对象体读取器。
//!
//! 依据 `docs/mxd-layer-luna.md` 的续研记录：short 版本、int32 计数、循环中的
//! int32 标识与符号对象，随后按版本读取名称及尾字段。重复标识不读名称目前
//! 仅有反编译控制流证据，尚未汇编或运行复核。未知符号体无外部长度，故在其
//! 对象头起点成功停止，保留此前已解析条目。

use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, SymbolStop10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// SymbolCollection 类标识（CLSID）磁盘字节顺序。
pub const SYMBOL_COLLECTION_CLASS_ID: [u8; 16] = [
    0x66, 0x8c, 0x5d, 0x0e, 0x91, 0x8d, 0xd3, 0x11, 0x9f, 0xca, 0x00, 0xc0, 0x4f, 0x6b, 0xc6, 0xa5,
];

const MAX_VERSION: i16 = 3;
const MAX_ITEMS: usize = 1_000_000;

/// 一条已读取的集合记录。
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolCollectionItem10_1 {
    /// 原生循环中的 int32 标识。
    pub id: i32,
    /// 符号对象条目；嵌套停止时保留已解析的部分体。
    pub symbol: SymbolSlotFull10_1,
    /// 版本大于 1 且该标识首次出现时读取的名称；其他情况下为空。
    pub name: Option<String>,
    /// 版本 1 按序号生成名称；原生名称文本规则尚未复核。
    pub generated_name_ordinal: Option<u32>,
}

/// 可识别的成功停止原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolCollectionStop10_1 {
    /// 新对象的类没有已闭合符号读取器，游标位于该符号对象头。
    UnsupportedSymbol { class_id: [u8; 16], key: u32 },
    /// 已识别符号体内部停止；具体内层信息保存在最后一条记录的符号体中。
    SymbolBodyIncomplete { reason: SymbolStop10_1 },
}

/// 已解析的 SymbolCollection 对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolCollectionBody10_1 {
    /// 集合对象键（由外层对象头解析器提供）。
    pub key: u32,
    /// 有符号 short 版本。
    pub version: i16,
    /// 已解析条目前缀。
    pub items: Vec<SymbolCollectionItem10_1>,
    /// 版本 2 的循环后字段。
    pub version_2_tail: Option<i32>,
    /// 版本大于 1 的循环后字段。
    pub modern_tail: Option<i32>,
    /// 符号停止原因；存在时尾字段尚未读取。
    pub stop: Option<SymbolCollectionStop10_1>,
}

/// 从对象体首字节读取 SymbolCollection（对象头已由调用方读取并登记）。
///
/// 成功完整消费时 `stop` 为 `None`。遇到未知符号类时成功返回已解析前缀，
/// 游标停在该符号对象头，且其临时登记不会提交。已识别符号体内停止时保留
/// 部分条目及已登记键，游标停在内层停止点，名称和尾字段均不读取。
/// 格式错误、截断及限额错误
/// 均返回 `Err`，不改变传入游标或引用表。`depth` 传递给符号体分派器。
pub fn read_symbol_collection_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SymbolCollectionBody10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = trial.read_u16_le()? as i16;
    if version > MAX_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("SymbolCollection 版本 {version} 超过 10.1 上限 {MAX_VERSION}"),
        ));
    }
    let count = trial.read_u32_le()? as i32;
    if count < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "SymbolCollection 计数为负数",
        ));
    }
    let count = count as usize;
    if count > MAX_ITEMS
        || count
            .checked_mul(20)
            .is_none_or(|minimum| minimum > max_bytes)
    {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "SymbolCollection 条目数超过读取限额",
        ));
    }

    let mut items = Vec::with_capacity(count);
    let mut ids = std::collections::HashSet::new();
    let mut stop = None;
    for ordinal in 0..count {
        let id = trial.read_u32_le()? as i32;
        let symbol_start = trial.clone();
        let before_symbol = staged.clone();
        let symbol = read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut staged, depth)?;
        if let SymbolSlotFull10_1::Unsupported {
            class_id,
            key: symbol_key,
        } = &symbol
        {
            trial = symbol_start;
            staged = before_symbol;
            stop = Some(SymbolCollectionStop10_1::UnsupportedSymbol {
                class_id: *class_id,
                key: *symbol_key,
            });
            break;
        }
        if let Some(reason) = slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol) {
            items.push(SymbolCollectionItem10_1 {
                id,
                symbol,
                name: None,
                generated_name_ordinal: None,
            });
            stop = Some(SymbolCollectionStop10_1::SymbolBodyIncomplete { reason });
            break;
        }
        let first_id = ids.insert(id);
        let (name, generated_name_ordinal) = if first_id && version > 1 {
            (Some(read_string_10_1(&mut trial, max_bytes)?), None)
        } else if first_id && version == 1 {
            (None, Some(ordinal as u32))
        } else {
            (None, None)
        };
        items.push(SymbolCollectionItem10_1 {
            id,
            symbol,
            name,
            generated_name_ordinal,
        });
    }
    let (version_2_tail, modern_tail) = if stop.is_none() {
        let v2 = (version == 2)
            .then(|| trial.read_u32_le())
            .transpose()?
            .map(|v| v as i32);
        let modern = (version > 1)
            .then(|| trial.read_u32_le())
            .transpose()?
            .map(|v| v as i32);
        (v2, modern)
    } else {
        (None, None)
    };

    *reader = trial;
    *table = staged;
    Ok(SymbolCollectionBody10_1 {
        key,
        version,
        items,
        version_2_tail,
        modern_tail,
        stop,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::display_symbol::{
        SIMPLE_MARKER_SYMBOL_CLASS_ID, SimpleMarkerSymbolStop10_1,
    };

    #[test]
    fn parses_sample_shaped_empty_version_three_body() {
        let bytes = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_symbol_collection_body_10_1(&mut reader, 1024, &mut refs, 156, 32).unwrap();
        assert_eq!(body.version, 3);
        assert!(body.items.is_empty());
        assert_eq!(body.modern_tail, Some(0));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn errors_are_transactional() {
        let bytes = [4, 0, 0, 0, 0, 0];
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        assert!(read_symbol_collection_body_10_1(&mut reader, 1024, &mut refs, 1, 4).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn nested_stop_preserves_partial_symbol_and_reference_state() {
        let unknown_color = [0x71; 16];
        let mut bytes = 3_i16.to_le_bytes().to_vec();
        bytes.extend(2_i32.to_le_bytes());
        // 第一条为空引用及合法名称，确认已完成的前缀得以保留。
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend([b'A', 0, 0, 0]);
        bytes.extend(8_i32.to_le_bytes());
        bytes.extend(SIMPLE_MARKER_SYMBOL_CLASS_ID);
        bytes.extend(36_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        let stopped_at = bytes.len();
        bytes.extend(unknown_color);
        bytes.extend(920_u32.to_le_bytes());
        // 若误读后续名称，则将未知颜色头当成字符串长度而报错。
        bytes.extend(0_i32.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        refs.register(100);
        let body = read_symbol_collection_body_10_1(&mut reader, 1024, &mut refs, 156, 32).unwrap();
        assert_eq!(reader.position(), stopped_at);
        assert_eq!(body.items.len(), 2);
        assert_eq!(body.items[0].name.as_deref(), Some("A"));
        assert_eq!(body.items[1].id, 8);
        assert_eq!(body.items[1].name, None);
        assert_eq!(body.modern_tail, None);
        assert_eq!(body.version_2_tail, None);
        assert_eq!(
            body.stop,
            Some(SymbolCollectionStop10_1::SymbolBodyIncomplete {
                reason: SymbolStop10_1::NestedIncomplete {
                    class_id: SIMPLE_MARKER_SYMBOL_CLASS_ID,
                    key: 36
                },
            })
        );
        match &body.items[1].symbol {
            SymbolSlotFull10_1::SimpleMarkerSymbol(symbol) => assert_eq!(
                symbol.stop,
                Some(SimpleMarkerSymbolStop10_1::ColorUnsupported {
                    class_id: unknown_color,
                    key: 920
                }),
            ),
            other => panic!("期望部分符号体，得到 {other:?}"),
        }
        assert!(refs.contains(100));
        assert!(refs.contains(36));
        assert!(!refs.contains(920));
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn unknown_symbol_stops_at_header_without_registering_key() {
        let mut bytes = 3_i16.to_le_bytes().to_vec();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(8_i32.to_le_bytes());
        let stopped_at = bytes.len();
        bytes.extend([0x72; 16]);
        bytes.extend(37_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_symbol_collection_body_10_1(&mut reader, 1024, &mut refs, 156, 32).unwrap();
        assert_eq!(reader.position(), stopped_at);
        assert!(refs.is_empty());
        assert!(body.items.is_empty());
        assert_eq!(
            body.stop,
            Some(SymbolCollectionStop10_1::UnsupportedSymbol {
                class_id: [0x72; 16],
                key: 37
            })
        );
        assert_eq!(body.modern_tail, None);
    }
}
