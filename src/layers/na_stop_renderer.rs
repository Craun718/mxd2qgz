use super::feature_layer_tail::{
    LEGEND_GROUP_CLASS_ID, LegendGroupSlot10_1, read_legend_group_body_10_1,
};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
    slot_full_stop_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io;

pub(super) const NA_STOP_RENDERER_CLASS_ID: [u8; 16] = [
    0xe3, 0xf6, 0xf9, 0x7c, 0x6a, 0x37, 0xb2, 0x49, 0xba, 0xee, 0xc8, 0x30, 0x07, 0x98, 0xae, 0xce,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NaStopRenderer10_1 {
    pub version: i16,
    pub legend: LegendGroupSlot10_1,
    pub color: ColorSlot10_1,
    pub text_symbol: SymbolSlotFull10_1,
    pub visibility: Vec<u16>,
    pub indices: Vec<i32>,
}

/// NetworkAnalyst.dll 10.1 的 0x7af6a360：旧版以图例头开场，新版写零 GUID 和版本。
pub(super) fn read_na_stop_renderer(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<NaStopRenderer10_1> {
    let mut trial = reader.clone();
    let mut keys = refs.clone();
    let unsupported = |message| io::Error::new(io::ErrorKind::Unsupported, message);
    let mut probe = trial.clone();
    let version = match probe.read_bytes(16)? {
        bytes if bytes == LEGEND_GROUP_CLASS_ID => 1,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
            trial = probe;
            trial.read_u16_le()? as i16
        }
        _ => return Err(unsupported("NAStopRenderer 前导标识未支持")),
    };
    let (legend, count) = match read_object_entry_10_1(&mut trial, &mut keys)? {
        ObjectEntry10_1::NewObject {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        } => {
            let body = read_legend_group_body_10_1(&mut trial, max, &mut keys, key)?;
            if body.stop.is_some() {
                return Err(unsupported("NAStopRenderer 图例组未完整读取"));
            }
            let count = body.classes.len();
            (LegendGroupSlot10_1::LegendGroup(Box::new(body)), count)
        }
        ObjectEntry10_1::BackReference {
            class_id: LEGEND_GROUP_CLASS_ID,
            key,
        } => {
            let count = if version < 2 {
                0
            } else {
                keys.legend_group_count(key)
                    .ok_or_else(|| unsupported("NAStopRenderer 图例组回引缺少分类数"))?
            };
            (
                LegendGroupSlot10_1::BackReference {
                    class_id: LEGEND_GROUP_CLASS_ID,
                    key,
                },
                count,
            )
        }
        _ => return Err(unsupported("NAStopRenderer 图例组或回引分类数未支持")),
    };
    let color = read_color_slot_10_1(&mut trial, max, &mut keys)?;
    if matches!(color, ColorSlot10_1::Unsupported { .. }) {
        return Err(unsupported("NAStopRenderer 颜色对象体未支持"));
    }
    let text_symbol =
        read_symbol_slot_full_10_1(&mut trial, max, &mut keys, MAX_SYMBOL_NESTING_DEPTH)?;
    if slot_full_stop_10_1(&text_symbol, DisplaySlotKind10_1::Symbol).is_some() {
        return Err(unsupported("NAStopRenderer 文本符号未完整读取"));
    }
    let mut visibility = Vec::new();
    let mut indices = Vec::new();
    if version >= 2 {
        if count > trial.remaining().min(max) / 6 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "NAStopRenderer 分类尾段不足",
            ));
        }
        for _ in 0..count {
            visibility.push(trial.read_u16_le()?);
        }
        for _ in 0..count {
            indices.push(trial.read_u32_le()? as i32);
        }
    }
    *reader = trial;
    *refs = keys;
    Ok(NaStopRenderer10_1 {
        version,
        legend,
        color,
        text_symbol,
        visibility,
        indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_two_reads_arrays_after_slots_and_rolls_back_on_truncation() {
        let mut bytes = vec![0; 16];
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(LEGEND_GROUP_CLASS_ID);
        bytes.extend(262_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend([0; 4]);
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend([0; 2]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 48]);
        bytes.extend(0xffff_u16.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_na_stop_renderer(&mut reader, 512, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let body = read_na_stop_renderer(&mut reader, 512, &mut refs).unwrap();
        assert_eq!(body.visibility, [0xffff]);
        assert_eq!(body.indices, [4]);
        assert!(refs.contains(262));
        assert_eq!(reader.remaining(), 0);

        let mut reference = bytes[..38].to_vec(); // 版本字段和图例组对象头。
        reference.extend(&bytes[bytes.len() - 38..]); // 两个空对象槽和分类数组。
        let mut unparsed = RefTable10_1::new();
        unparsed.register(262);
        let mut reader = Reader::new(&reference);
        assert!(read_na_stop_renderer(&mut reader, 512, &mut unparsed).is_err());
        assert_eq!(reader.position(), 0);
        let body = read_na_stop_renderer(&mut reader, 512, &mut refs).unwrap();
        assert!(matches!(
            body.legend,
            LegendGroupSlot10_1::BackReference { key: 262, .. }
        ));
        assert_eq!(body.visibility, [0xffff]);
        assert_eq!(body.indices, [4]);
        assert_eq!(reader.remaining(), 0);
    }
}
