//! DisplayCore.dll 10.1 统计图符号。原生读写依据见 `docs/arcmap-10.1-dep-tail.md`。

use super::{
    DisplaySlotKind10_1, FILL_SYMBOL_CLASSES, LINE_SYMBOL_CLASSES, SymbolSlotFull10_1, read_i32_le,
    read_loop_count_10_1, read_symbol_slot_full_10_1, read_version_short, slot_full_stop_10_1,
};
use crate::objects::callout_background::{
    CalloutSlot10_1, LINE_CALLOUT_CLASS_ID, read_callout_slot_10_1,
    read_marker_background_slot_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const BAR_CHART_SYMBOL_CLASS_ID: [u8; 16] = [
    0x6a, 0x73, 0x31, 0x50, 0x70, 0xbd, 0xd3, 0x11, 0x9f, 0x79, 0x00, 0xc0, 0x4f, 0x6b, 0xc7, 0x09,
];
pub const PIE_CHART_SYMBOL_CLASS_ID: [u8; 16] = [
    0x68, 0x73, 0x31, 0x50, 0x70, 0xbd, 0xd3, 0x11, 0x9f, 0x79, 0x00, 0xc0, 0x4f, 0x6b, 0xc7, 0x09,
];
pub const STACKED_CHART_SYMBOL_CLASS_ID: [u8; 16] = [
    0x69, 0x73, 0x31, 0x50, 0x70, 0xbd, 0xd3, 0x11, 0x9f, 0x79, 0x00, 0xc0, 0x4f, 0x6b, 0xc7, 0x09,
];

/// 具体符号在首段共同字段之后写入的标量，按流中顺序保留。
#[derive(Debug, Clone, PartialEq)]
pub enum ChartSymbolFields10_1 {
    Bar { flags: [u8; 2], values: [f64; 2] },
    Pie { value: i32, flags: [u8; 2] },
    Stacked { flags: [u8; 3], value: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartSymbolBody10_1 {
    pub key: u32,
    pub version: u16,
    pub initial_values: [f64; 4],
    pub flag: u8,
    pub value: f64,
    pub fields: ChartSymbolFields10_1,
    pub outline: SymbolSlotFull10_1,
    pub common_version: u16,
    pub common_integers: [i32; 2],
    pub common_value: f64,
    pub legacy_callout: Option<CalloutSlot10_1>,
    pub entries: Vec<(SymbolSlotFull10_1, f64)>,
    pub marker_background: CalloutSlot10_1,
}

fn require_known_class(
    reader: &Reader<'_>,
    table: &RefTable10_1,
    supported_classes: &[[u8; 16]],
) -> io::Result<()> {
    let mut probe = reader.clone();
    if let ObjectHeader::Object { class_id, key } = read_header_10_1(&mut probe)?
        && !table.contains(key)
        && !supported_classes.contains(&class_id)
    {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!(
                "统计图符号嵌套槽 offset 0x{:x}: 不支持该类 {class_id:02x?}",
                reader.position()
            ),
        ));
    }
    Ok(())
}

fn read_chart_line_or_fill(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    fill: bool,
) -> io::Result<SymbolSlotFull10_1> {
    let (classes, kind) = if fill {
        (FILL_SYMBOL_CLASSES, DisplaySlotKind10_1::FillSymbol)
    } else {
        (LINE_SYMBOL_CLASSES, DisplaySlotKind10_1::LineSymbol)
    };
    require_known_class(reader, table, classes)?;
    let symbol = read_symbol_slot_full_10_1(reader, max_bytes, table, depth)?;
    if let Some(stop) = slot_full_stop_10_1(&symbol, kind) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("统计图符号嵌套槽未读完: {stop:?}"),
        ));
    }
    Ok(symbol)
}

fn read_chart_callout(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    legacy: bool,
) -> io::Result<CalloutSlot10_1> {
    // LineCallout 同时实现 ILineCallout 和 IMarkerBackground。
    let (slot, stop) = if legacy {
        require_known_class(reader, table, &[LINE_CALLOUT_CLASS_ID])?;
        read_callout_slot_10_1(
            reader,
            max_bytes,
            table,
            depth,
            false,
            DisplaySlotKind10_1::MarkerBackground,
        )?
    } else {
        read_marker_background_slot_10_1(reader, max_bytes, table, depth)?
    };
    match (slot, stop) {
        (Some(slot), None) => Ok(slot),
        (_, stop) => Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("统计图符号标注或背景槽未读完: {stop:?}"),
        )),
    }
}

/// 在外层符号槽的试读游标和登记表上读取全体。任一嵌套槽未闭合时由外层整体回退。
pub(super) fn read_chart_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<ChartSymbolBody10_1> {
    let offset = reader.position();
    let version = read_version_short(reader, 1, "ChartSymbol", offset)?;
    let mut initial_values = [0.0; 4];
    for value in &mut initial_values {
        *value = reader.read_f64_le()?;
    }
    let flag = reader.read_bytes(1)?[0];
    let value = reader.read_f64_le()?;
    let fields = match class_id {
        BAR_CHART_SYMBOL_CLASS_ID => ChartSymbolFields10_1::Bar {
            flags: reader.read_bytes(2)?.try_into().unwrap(),
            values: [reader.read_f64_le()?, reader.read_f64_le()?],
        },
        PIE_CHART_SYMBOL_CLASS_ID => ChartSymbolFields10_1::Pie {
            value: read_i32_le(reader)?,
            flags: reader.read_bytes(2)?.try_into().unwrap(),
        },
        STACKED_CHART_SYMBOL_CLASS_ID => ChartSymbolFields10_1::Stacked {
            flags: reader.read_bytes(3)?.try_into().unwrap(),
            value: reader.read_f64_le()?,
        },
        _ => unreachable!("调用者已识别统计图符号类"),
    };
    let outline = read_chart_line_or_fill(
        reader,
        max_bytes,
        table,
        depth,
        class_id == STACKED_CHART_SYMBOL_CLASS_ID,
    )?;

    let common_offset = reader.position();
    let common_version = read_version_short(reader, 2, "ChartSymbol 公共段", common_offset)?;
    let common_integers = [read_i32_le(reader)?, read_i32_le(reader)?];
    let common_value = reader.read_f64_le()?;
    let legacy_callout = if common_version == 1 {
        Some(read_chart_callout(reader, max_bytes, table, depth, true)?)
    } else {
        None
    };
    let count = read_loop_count_10_1(reader, max_bytes, "ChartSymbol", common_offset)? as usize;
    // 每项至少包含空对象头和 f64，末尾还需要一个背景对象头。
    if count > reader.remaining().saturating_sub(16) / 24 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "统计图符号填充数组截断",
        ));
    }
    let mut entries = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        let fill = read_chart_line_or_fill(reader, max_bytes, table, depth, true)?;
        entries.push((fill, reader.read_f64_le()?));
    }
    let marker_background = read_chart_callout(reader, max_bytes, table, depth, false)?;
    Ok(ChartSymbolBody10_1 {
        key,
        version,
        initial_values,
        flag,
        value,
        fields,
        outline,
        common_version,
        common_integers,
        common_value,
        legacy_callout,
        entries,
        marker_background,
    })
}
