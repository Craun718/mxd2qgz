//! RouteEvents.dll 10.1 的路线图层扩展及属性对象。

use super::image_server_rules::{RuleSlot10_1, slot, unsupported_class};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::objects::query_filter::{
    QueryFilterBody10_1, QueryFilterObject10_1, read_query_filter_slot_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const ROUTE_LAYER_EXTENSION_CLASS_ID: [u8; 16] = [
    0x78, 0xec, 0xe2, 0x7a, 0x86, 0x3a, 0x7b, 0x4b, 0x90, 0x1b, 0xa3, 0x0d, 0x1d, 0x99, 0xf4, 0xca,
];
pub const ROUTE_ANOMALY_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x73, 0xc5, 0xda, 0x17, 0x2f, 0x75, 0x3f, 0x48, 0xbb, 0xa2, 0x3e, 0x7f, 0xa4, 0xab, 0x34, 0xc1,
];
pub const ROUTE_IDENTIFY_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x50, 0x40, 0x6f, 0xf9, 0x8f, 0x0e, 0x04, 0x45, 0xac, 0xdf, 0x78, 0x4e, 0x4d, 0x24, 0x05, 0x09,
];

#[derive(Debug, Clone, PartialEq)]
pub struct RouteLayerExtension10_1 {
    pub version: i16,
    pub string: String,
    pub anomaly: RuleSlot10_1<RouteAnomalyProperties10_1>,
    pub identify: RuleSlot10_1<RouteIdentifyProperties10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteAnomalyProperties10_1 {
    pub version: i16,
    pub doubles: [f64; 2],
    pub shorts: [i16; 4],
    pub filter: QueryFilterObject10_1<QueryFilterBody10_1>,
    /// 原生按正计数读取，其他计数原样保留。
    pub symbol_count: i32,
    pub symbols: Vec<SymbolSlotFull10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteIdentifyProperties10_1 {
    pub version: i16,
    pub strings: [String; 6],
    pub long: i32,
}

fn read_version(r: &mut Reader<'_>, label: &str) -> io::Result<i16> {
    let version = r.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{label} 未支持版本 {version}"),
        ));
    }
    Ok(version)
}

fn read_anomaly_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RouteAnomalyProperties10_1> {
    let version = read_version(r, "RouteAnomalyProperties")?;
    let doubles = [r.read_f64_le()?, r.read_f64_le()?];
    let mut shorts = [0; 4];
    for value in &mut shorts {
        *value = r.read_u16_le()? as i16;
    }
    let mut stop = None;
    let filter = read_query_filter_slot_10_1(r, max, refs, &mut stop)?;
    if let Some(reason) = stop {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("RouteAnomalyProperties 查询过滤器未完整读取：{reason:?}"),
        ));
    }
    let symbol_count = r.read_u32_le()? as i32;
    let count = symbol_count.max(0) as usize;
    if count > r.remaining().min(max) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RouteAnomalyProperties 符号数量超出可读范围",
        ));
    }
    let mut symbols = Vec::new();
    for _ in 0..count {
        let symbol = read_symbol_slot_full_10_1(r, max, refs, MAX_SYMBOL_NESTING_DEPTH)?;
        if let Some(reason) = slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol) {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("RouteAnomalyProperties 符号未完整读取：{reason:?}"),
            ));
        }
        if matches!(symbol, SymbolSlotFull10_1::Null) {
            return Err(io::Error::new(ErrorKind::InvalidData, "路线异常符号为空"));
        }
        symbols.push(symbol);
    }
    Ok(RouteAnomalyProperties10_1 {
        version,
        doubles,
        shorts,
        filter,
        symbol_count,
        symbols,
    })
}

fn read_identify_properties(
    r: &mut Reader<'_>,
    max: usize,
) -> io::Result<RouteIdentifyProperties10_1> {
    let version = read_version(r, "RouteIdentifyProperties")?;
    let mut strings = std::array::from_fn(|_| String::new());
    for value in &mut strings {
        *value = read_string_10_1(r, max)?;
    }
    let long = r.read_u32_le()? as i32;
    Ok(RouteIdentifyProperties10_1 {
        version,
        strings,
        long,
    })
}

pub(super) fn read_bounded_route_layer_extension(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<RouteLayerExtension10_1>> {
    let mut r = Reader::new(bytes);
    let mut staged = refs.clone();
    let version = read_version(&mut r, "RouteLayerExtension")?;
    let string = read_string_10_1(&mut r, max)?;
    let anomaly = slot(&mut r, &mut staged, |r, refs, class, _| {
        if class != ROUTE_ANOMALY_PROPERTIES_CLASS_ID {
            return Err(unsupported_class("RouteAnomalyProperties", &class));
        }
        read_anomaly_properties(r, max, refs)
    })?;
    let identify = slot(&mut r, &mut staged, |r, _, class, _| {
        if class != ROUTE_IDENTIFY_PROPERTIES_CLASS_ID {
            return Err(unsupported_class("RouteIdentifyProperties", &class));
        }
        read_identify_properties(r, max)
    })?;
    for (entry, expected, label) in [
        (
            anomaly.entry,
            ROUTE_ANOMALY_PROPERTIES_CLASS_ID,
            "RouteAnomalyProperties",
        ),
        (
            identify.entry,
            ROUTE_IDENTIFY_PROPERTIES_CLASS_ID,
            "RouteIdentifyProperties",
        ),
    ] {
        if let ObjectEntry10_1::BackReference { class_id, .. } = entry
            && class_id != expected
        {
            return Err(unsupported_class(label, &class_id));
        }
    }
    if r.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RouteLayerExtension 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(RouteLayerExtension10_1 {
        version,
        string,
        anomaly,
        identify,
    }))
}
