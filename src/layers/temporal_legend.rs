//! TrackingRenderer.dll 10.1 时态图例记录。

use super::image_server_rules::unsupported_class;
use super::legend_format::{
    LEGEND_CLASS_FORMAT_CLASS_ID, LegendClassFormatBody10_1, read_legend_class_format_body_10_1,
};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, SymbolSlotFull10_1, read_unkeyed_symbol_slot_10_1, slot_full_stop_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_atl_bstr_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const TEMPORAL_LEGEND_GROUP_CLASS_ID: [u8; 16] = [
    0x61, 0x72, 0x94, 0x5f, 0x06, 0x14, 0xd5, 0x11, 0xb7, 0xca, 0, 1, 2, 0x65, 0xad, 0xc5,
];

const TEMPORAL_LEGEND_CLASS_ID: [u8; 16] = [
    0x20, 0xc7, 0x53, 0xcb, 0xa6, 0x14, 0xd5, 0x11, 0xb7, 0xce, 0, 1, 2, 0x65, 0xad, 0xc5,
];

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalLegendGroup10_1 {
    pub version: f64,
    pub texts: [Option<Vec<u8>>; 2],
    pub classes: Vec<Option<TemporalLegendClass10_1>>,
    pub flags: [i16; 2],
    pub extra_flag: Option<i16>,
    pub extra_text: Option<Option<Vec<u8>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalLegendClass10_1 {
    pub version: f64,
    pub value: i32,
    pub symbol: Option<Box<SymbolSlotFull10_1>>,
    pub texts: [Option<Vec<u8>>; 2],
    pub format: Option<Box<LegendClassFormatBody10_1>>,
    pub first_number: f64,
    pub flag: i32,
    pub second_number: f64,
    pub extra_value: Option<i32>,
}

fn read_unknown_type(reader: &mut Reader<'_>) -> io::Result<()> {
    let offset = reader.position();
    let vartype = reader.read_u16_le()?;
    if vartype == 13 {
        return Ok(());
    }
    Err(io::Error::new(
        ErrorKind::Unsupported,
        format!("时态图例 offset 0x{offset:x}: 不支持的 VARTYPE {vartype}"),
    ))
}

fn read_class(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TemporalLegendClass10_1> {
    let version = reader.read_f64_le()?;
    let value = reader.read_u32_le()? as i32;
    let symbol = if reader.read_u16_le()? == u16::MAX {
        read_unknown_type(reader)?;
        let slot = read_unkeyed_symbol_slot_10_1(reader, max, refs)?;
        if let Some(stop) = slot_full_stop_10_1(&slot, DisplaySlotKind10_1::Symbol) {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("时态图例符号未读完：{stop:?}"),
            ));
        }
        (!matches!(slot, SymbolSlotFull10_1::Null)).then(|| Box::new(slot))
    } else {
        None
    };
    let texts = [
        read_atl_bstr_10_1(reader, max)?,
        read_atl_bstr_10_1(reader, max)?,
    ];
    let format = if reader.read_u16_le()? == u16::MAX {
        read_unknown_type(reader)?;
        let class = reader.read_bytes(16)?.try_into().unwrap();
        match class {
            class if class == [0; 16] => None,
            LEGEND_CLASS_FORMAT_CLASS_ID => {
                let body = read_legend_class_format_body_10_1(reader, max, refs, 0)?;
                if let Some(stop) = body.stop {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        format!("时态图例格式未读完：{stop:?}"),
                    ));
                }
                Some(Box::new(body))
            }
            _ => return Err(unsupported_class("时态图例格式", &class)),
        }
    } else {
        None
    };
    Ok(TemporalLegendClass10_1 {
        version,
        value,
        symbol,
        texts,
        format,
        first_number: reader.read_f64_le()?,
        flag: reader.read_u32_le()? as i32,
        second_number: reader.read_f64_le()?,
        extra_value: (version >= 2.0)
            .then(|| reader.read_u32_le().map(|value| value as i32))
            .transpose()?,
    })
}

/// 从组对象体开始读取，失败时保留输入位置和引用表。
pub fn read_temporal_legend_group_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TemporalLegendGroup10_1> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let version = trial.read_f64_le()?;
    let texts = [
        read_atl_bstr_10_1(&mut trial, max)?,
        read_atl_bstr_10_1(&mut trial, max)?,
    ];
    let count = (trial.read_u32_le()? as i32).max(0) as usize;
    if count > trial.remaining().min(max) / 18 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "时态图例类别数超过剩余字节或限额",
        ));
    }
    let mut classes = Vec::new();
    for _ in 0..count {
        read_unknown_type(&mut trial)?;
        let class = trial.read_bytes(16)?.try_into().unwrap();
        classes.push(match class {
            class if class == [0; 16] => None,
            TEMPORAL_LEGEND_CLASS_ID => Some(read_class(&mut trial, max, &mut staged)?),
            _ => return Err(unsupported_class("时态图例类别", &class)),
        });
    }
    let group = TemporalLegendGroup10_1 {
        version,
        texts,
        classes,
        flags: [trial.read_u16_le()? as i16, trial.read_u16_le()? as i16],
        extra_flag: (version >= 2.0)
            .then(|| trial.read_u16_le().map(|value| value as i16))
            .transpose()?,
        extra_text: (version >= 10.1)
            .then(|| read_atl_bstr_10_1(&mut trial, max))
            .transpose()?,
    };
    *reader = trial;
    *refs = staged;
    Ok(group)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::display_color::RGB_COLOR_CLASS_ID;
    use crate::objects::display_symbol::SIMPLE_LINE_SYMBOL_CLASS_ID;

    fn group() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(10.1_f64.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend(TEMPORAL_LEGEND_CLASS_ID);
        bytes.extend(2.0_f64.to_le_bytes());
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend(u16::MAX.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend(SIMPLE_LINE_SYMBOL_CLASS_ID);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(RGB_COLOR_CLASS_ID);
        bytes.extend(42_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([1, 1, 2, 3, 0, 0, 255]);
        bytes.extend([0; 20]);
        bytes.extend([0; 8]);
        bytes.extend(u16::MAX.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend(LEGEND_CLASS_FORMAT_CLASS_ID);
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend([0; 80]);
        bytes.extend([0; 20]);
        bytes.extend(99_i32.to_le_bytes());
        bytes.extend([0; 6]);
        bytes.extend(0_u32.to_le_bytes());
        bytes
    }

    #[test]
    fn reads_unkeyed_objects_and_registers_nested_color() {
        let mut bytes = group();
        bytes.extend(0xabcd_u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::default();
        let group = read_temporal_legend_group_10_1(&mut reader, bytes.len(), &mut refs).unwrap();
        let class = group.classes[0].as_ref().unwrap();
        assert!(matches!(
            class.symbol.as_deref(),
            Some(SymbolSlotFull10_1::SimpleLineSymbol(_))
        ));
        assert_eq!(
            class.format.as_ref().unwrap().patch_doubles,
            Some([0.0, 0.0])
        );
        assert_eq!(class.extra_value, Some(99));
        assert_eq!(group.extra_text, Some(None));
        assert_eq!(refs.registered_class(42), Some(RGB_COLOR_CLASS_ID));
        assert_eq!(refs.registered_class(0), None);
        assert_eq!(reader.read_u16_le().unwrap(), 0xabcd);
    }

    #[test]
    fn truncated_group_rolls_back_nested_registration() {
        let mut bytes = group();
        bytes.pop();
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::default();
        assert!(read_temporal_legend_group_10_1(&mut reader, bytes.len(), &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.registered_class(42), None);
    }
}
