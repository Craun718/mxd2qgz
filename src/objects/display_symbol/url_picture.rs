//! WMSLayer.dll 的 URLPictureMarkerSymbol。只读取持久化字段。

use super::{
    DisplaySlotKind10_1, PICTURE_MARKER_SYMBOL_CLASS_ID, SymbolSlotFull10_1, SymbolStop10_1,
    read_symbol_slot_full_10_1, read_version_short, slot_full_stop_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io;

pub const URL_PICTURE_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0x01, 0xcb, 0x13, 0x42, 0xe0, 0xbe, 0x3f, 0x4f, 0xbf, 0x07, 0x0c, 0xd0, 0xa4, 0xc9, 0x5e, 0xb2,
];

#[derive(Debug, Clone, PartialEq)]
pub struct UrlPictureMarkerSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    /// 原生 String::Write 写出的两个字符串，按流中顺序保留。
    pub strings: [String; 2],
    pub picture_marker: SymbolSlotFull10_1,
    pub stop: Option<SymbolStop10_1>,
}

pub(super) fn read_url_picture_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<UrlPictureMarkerSymbolBody10_1> {
    let version = read_version_short(reader, 1, "URLPictureMarkerSymbol", reader.position())?;
    let strings = [
        read_string_10_1(reader, max_bytes)?,
        read_string_10_1(reader, max_bytes)?,
    ];
    let header = read_header_10_1(&mut reader.clone())?;
    let picture_marker = match header {
        ObjectHeader::Object { class_id, key }
            if class_id != PICTURE_MARKER_SYMBOL_CLASS_ID && !table.contains(key) =>
        {
            SymbolSlotFull10_1::Unsupported { class_id, key }
        }
        _ => read_symbol_slot_full_10_1(reader, max_bytes, table, depth)?,
    };
    let stop = slot_full_stop_10_1(&picture_marker, DisplaySlotKind10_1::MarkerSymbol);
    Ok(UrlPictureMarkerSymbolBody10_1 {
        key,
        version,
        strings,
        picture_marker,
        stop,
    })
}
