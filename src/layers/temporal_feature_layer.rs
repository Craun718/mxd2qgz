//! TrackingLayer.dll 10.1 时态图层的版本化前段。

use super::dep_tail::{
    TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, read_time_extent_body_10_1,
    read_time_reference_body_10_1,
};
use super::feature_layer::{
    FeatureLayerPrefix10_1, FeatureLayerStop10_1, read_feature_layer_prefix_10_1,
};
use super::feature_layer_tail::SuccessorStop10_1;
use super::image_server_rules::unsupported_class;
use super::temporal_legend::{
    TEMPORAL_LEGEND_GROUP_CLASS_ID, TemporalLegendGroup10_1, read_temporal_legend_group_10_1,
};
use super::temporal_record_set::{TemporalRecordSet10_1, read_temporal_record_set_10_1};
use crate::objects::display_color::read_color_body_for_class_10_1;
use crate::objects::element_body::{TEXT_SYMBOL_CLASS_ID, read_text_symbol_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_atl_bstr_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalFeatureLayerPrefix10_1 {
    pub version: f64,
    pub feature: Box<FeatureLayerPrefix10_1>,
    pub operator_flags: u32,
    pub operator_text: Option<Vec<u8>>,
}

fn read_time_slot(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    expected_class: [u8; 16],
) -> io::Result<()> {
    match read_object_entry_10_1(reader, refs)? {
        ObjectEntry10_1::Null => Ok(()),
        ObjectEntry10_1::BackReference { class_id, key }
            if class_id == expected_class && refs.registered_class(key) == Some(class_id) =>
        {
            Ok(())
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == expected_class => {
            let complete = if class_id == TIME_REFERENCE_CLASS_ID {
                read_time_reference_body_10_1(reader, max, refs, key)?
                    .short_b
                    .is_some()
            } else {
                read_time_extent_body_10_1(reader, max, refs, key)?
                    .stop
                    .is_none()
            };
            if complete {
                Ok(())
            } else {
                Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "时态图层的前置时间对象体未读完",
                ))
            }
        }
        ObjectEntry10_1::NewObject { class_id, .. }
        | ObjectEntry10_1::BackReference { class_id, .. } => {
            Err(unsupported_class("时态图层前置时间对象", &class_id))
        }
    }
}

fn read_v10_prefix(reader: &mut Reader<'_>, max: usize, refs: &mut RefTable10_1) -> io::Result<()> {
    read_atl_bstr_10_1(reader, max)?;
    reader.read_bytes(6)?;
    read_time_slot(reader, max, refs, TIME_REFERENCE_CLASS_ID)?;
    read_atl_bstr_10_1(reader, max)?;
    reader.read_bytes(12)?;
    for _ in 0..2 {
        read_time_slot(reader, max, refs, TIME_EXTENT_CLASS_ID)?;
    }
    reader.read_u16_le()?;
    Ok(())
}

/// 从时态图层首个 Variant 读至聚合 FeatureLayer 的末尾。
/// 适用于 MXD 的 IObjectStream 分支与已核实的三个写出版本。
pub fn read_temporal_feature_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TemporalFeatureLayerPrefix10_1> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let (version, operator_flags, operator_text) = read_header(&mut trial, max, &mut staged)?;
    let feature = read_feature_layer_prefix_10_1(&mut trial, max, &mut staged)?;
    if !matches!(
        feature.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted)
    ) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "时态图层的聚合 FeatureLayer 未读完",
        ));
    }
    *reader = trial;
    *refs = staged;
    Ok(TemporalFeatureLayerPrefix10_1 {
        version,
        feature: Box::new(feature),
        operator_flags,
        operator_text,
    })
}

fn read_header(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(f64, u32, Option<Vec<u8>>)> {
    if reader.read_u16_le()? != 5 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "时态图层首字段不是 VT_R8",
        ));
    }
    let version = reader.read_f64_le()?;
    if !matches!(version, 1.51 | 9.3 | 10.0) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("时态图层不支持对象版本 {version}"),
        ));
    }
    reader.read_u32_le()?;
    for _ in 0..4 {
        read_atl_bstr_10_1(reader, max)?;
    }
    if version == 10.0 {
        read_v10_prefix(reader, max, refs)?;
    }
    // 版本至少 1.5 有一个四字节值，至少 1.2 另有两个 short。
    reader.read_bytes(8)?;
    let operator_flags = reader.read_u32_le()?;
    reader.read_bytes(if operator_flags & 0xff00 == 0 { 28 } else { 12 })?;
    let operator_text = read_atl_bstr_10_1(reader, max)?;
    Ok((version, operator_flags, operator_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_collection_reads_versioned_prefix_and_unkeyed_entries() {
        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend(ACTION_COLLECTION_CLASS_ID);
        bytes.extend(1.0_f64.to_le_bytes());
        bytes.extend([0; 42]);
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.extend(0xabcd_u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        read_tail_variant(
            &mut reader,
            4096,
            &mut refs,
            TailVariantSlot::ActionCollection,
        )
        .unwrap();
        assert_eq!(reader.read_u16_le().unwrap(), 0xabcd);
        assert!(refs.is_empty());

        bytes[end - 16] = 1;
        let mut reader = Reader::new(&bytes);
        let error = read_tail_variant(
            &mut reader,
            4096,
            &mut refs,
            TailVariantSlot::ActionCollection,
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        let mut reader = Reader::new(&bytes[..end - 1]);
        assert!(
            read_tail_variant(
                &mut reader,
                4096,
                &mut refs,
                TailVariantSlot::ActionCollection
            )
            .is_err()
        );
    }

    #[test]
    fn tail_variants_consume_unkeyed_color_and_empty_bstr() {
        use crate::objects::display_color::RGB_COLOR_CLASS_ID;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend(RGB_COLOR_CLASS_ID);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([1, 1, 2, 3, 0, 0, 255]);
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(0xabcd_u16.to_le_bytes());

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        read_tail_variant(&mut reader, 4096, &mut refs, TailVariantSlot::DisplayObject).unwrap();
        read_tail_variant(
            &mut reader,
            4096,
            &mut refs,
            TailVariantSlot::ActionCollection,
        )
        .unwrap();
        assert_eq!(reader.read_u16_le().unwrap(), 0xabcd);
        assert!(refs.is_empty());
        let mut reader = Reader::new(&bytes[..28]);
        assert!(
            read_tail_variant(&mut reader, 4096, &mut refs, TailVariantSlot::DisplayObject)
                .is_err()
        );
    }

    #[test]
    fn trailing_variants_preserve_boundary_for_every_writer_version() {
        for version in [1.51, 9.3, 10.0] {
            let mut bytes = vec![0; 16];
            bytes.extend(5_u16.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
            bytes.extend(8_u16.to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            if version >= 9.3 {
                bytes.extend([0; 16]);
                bytes.extend(13_u16.to_le_bytes());
                bytes.extend([0; 16]);
                bytes.extend([0; 8]);
                for _ in 0..2 {
                    bytes.extend(11_u16.to_le_bytes());
                    bytes.extend(u16::MAX.to_le_bytes());
                }
                bytes.extend([0; 26]);
            }
            let end = bytes.len();
            bytes.extend(0xabcd_u16.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            read_trailing_fields(&mut reader, 4096, &mut RefTable10_1::new(), version).unwrap();
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_u16_le().unwrap(), 0xabcd);
            let mut reader = Reader::new(&bytes[..end - 1]);
            assert!(
                read_trailing_fields(&mut reader, 4096, &mut RefTable10_1::new(), version).is_err()
            );
        }
    }

    #[test]
    fn all_writer_versions_reach_the_aggregate_without_consuming_its_version() {
        for version in [1.51_f64, 9.3, 10.0] {
            for operator_flags in [0_u32, 0x100] {
                let mut bytes = 5_u16.to_le_bytes().to_vec();
                bytes.extend(version.to_le_bytes());
                bytes.extend([0; 20]);
                if version == 10.0 {
                    // 两个空 BSTR、三个空对象槽和定宽字段。
                    bytes.extend([0; 76]);
                }
                bytes.extend([0; 8]);
                bytes.extend(operator_flags.to_le_bytes());
                bytes.resize(bytes.len() + if operator_flags == 0 { 28 } else { 12 }, 0);
                bytes.extend(5_u32.to_le_bytes());
                bytes.extend([b'a', 0, b'b', 0, 0]);
                bytes.extend(34_u16.to_le_bytes());

                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert_eq!(
                    read_header(&mut reader, 4096, &mut refs).unwrap(),
                    (version, operator_flags, Some(b"a\0b".to_vec()))
                );
                assert_eq!(reader.read_u16_le().unwrap(), 34);

                let mut reader = Reader::new(&bytes);
                assert!(
                    read_temporal_feature_layer_prefix_10_1(&mut reader, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }
}

const TEMPORAL_REFERENCE_CLASS_ID: [u8; 16] = [
    0x3a, 0x42, 0xc7, 0x78, 0xcf, 0x17, 0xd5, 0x11, 0xb7, 0xcf, 0, 1, 2, 0x65, 0xad, 0xc5,
];

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalFeatureLayer10_1 {
    pub prefix: TemporalFeatureLayerPrefix10_1,
    pub legend: Option<Box<TemporalLegendGroup10_1>>,
    pub record_set: Option<TemporalRecordSet10_1>,
}

fn read_late_slot(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    expected: [u8; 16],
) -> io::Result<bool> {
    match read_object_entry_10_1(reader, refs)? {
        ObjectEntry10_1::Null => Ok(false),
        ObjectEntry10_1::BackReference { class_id, key }
            if class_id == expected && refs.registered_class(key) == Some(class_id) =>
        {
            Ok(false)
        }
        ObjectEntry10_1::NewObject { class_id, .. } if class_id == expected => Ok(true),
        ObjectEntry10_1::BackReference { class_id, .. }
        | ObjectEntry10_1::NewObject { class_id, .. } => {
            Err(unsupported_class("时态图层后继对象", &class_id))
        }
    }
}

const ACTION_COLLECTION_CLASS_ID: [u8; 16] = [
    0x4e, 0xc6, 0x48, 0x4c, 0x99, 0x91, 0xd5, 0x11, 0xb7, 0xfe, 0, 1, 2, 0x65, 0xad, 0xc5,
];

fn read_action_collection(reader: &mut Reader<'_>, max: usize) -> io::Result<()> {
    let version = reader.read_f64_le()?;
    for _ in 0..4 {
        read_atl_bstr_10_1(reader, max)?;
    }
    reader.read_bytes(14)?;
    if version >= 1.0 {
        reader.read_bytes(12)?;
        let count = (reader.read_u32_le()? as i32).max(0) as usize;
        if count > reader.remaining().min(max) / 18 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "动作集合条目数超过限额",
            ));
        }
        for _ in 0..count {
            if reader.read_u16_le()? != 13 {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "动作集合条目不是 VT_UNKNOWN",
                ));
            }
            let class = reader.read_bytes(16)?.try_into().unwrap();
            if class != [0; 16] {
                return Err(unsupported_class("动作集合 IAction", &class));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum TailVariantSlot {
    TextSymbol,
    ActionCollection,
    DisplayObject,
}

fn read_tail_variant(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    slot: TailVariantSlot,
) -> io::Result<()> {
    let vartype = reader.read_u16_le()?;
    let size = match vartype {
        2 | 11 | 18 => 2,
        3 | 4 | 10 | 19 | 22 | 23 => 4,
        5..=7 | 20 | 21 => 8,
        16 | 17 => 1,
        0 | 1 | 8 => {
            read_atl_bstr_10_1(reader, max)?;
            return Ok(());
        }
        9 | 13 => {
            let class = reader.read_bytes(16)?.try_into().unwrap();
            if class == [0; 16] {
                return Ok(());
            }
            match slot {
                TailVariantSlot::TextSymbol if class == TEXT_SYMBOL_CLASS_ID => {
                    if read_text_symbol_body_10_1(reader, max, refs)?
                        .stop
                        .is_none()
                    {
                        return Ok(());
                    }
                }
                TailVariantSlot::ActionCollection if class == ACTION_COLLECTION_CLASS_ID => {
                    return read_action_collection(reader, max);
                }
                TailVariantSlot::DisplayObject => {
                    // Variant 只写 CLSID，诊断占位键不加入引用表。
                    if read_color_body_for_class_10_1(class, reader, max, refs, 0)?.is_some() {
                        return Ok(());
                    }
                }
                _ => {}
            }
            return Err(unsupported_class("时态图层尾部 Variant", &class));
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("时态图层尾部 VARTYPE {vartype}"),
            ));
        }
    };
    reader.read_bytes(size)?;
    Ok(())
}

fn read_trailing_fields(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    version: f64,
) -> io::Result<()> {
    reader.read_bytes(12)?;
    read_atl_bstr_10_1(reader, max)?;
    read_tail_variant(reader, max, refs, TailVariantSlot::TextSymbol)?;
    read_tail_variant(reader, max, refs, TailVariantSlot::ActionCollection)?;
    if version >= 9.3 {
        reader.read_bytes(16)?;
        read_tail_variant(reader, max, refs, TailVariantSlot::DisplayObject)?;
        reader.read_bytes(8)?;
        read_tail_variant(reader, max, refs, TailVariantSlot::DisplayObject)?;
        read_tail_variant(reader, max, refs, TailVariantSlot::DisplayObject)?;
        reader.read_bytes(26)?;
    }
    Ok(())
}

pub(super) fn read_temporal_feature_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<TemporalFeatureLayer10_1> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let prefix = read_temporal_feature_layer_prefix_10_1(&mut trial, max, &mut staged)?;
    let legend = if read_late_slot(&mut trial, &mut staged, TEMPORAL_LEGEND_GROUP_CLASS_ID)? {
        Some(Box::new(read_temporal_legend_group_10_1(
            &mut trial,
            max,
            &mut staged,
        )?))
    } else {
        None
    };
    if read_late_slot(&mut trial, &mut staged, TEMPORAL_REFERENCE_CLASS_ID)? {
        trial.read_bytes(32)?;
    }
    trial.read_u32_le()?;
    read_atl_bstr_10_1(&mut trial, max)?;
    trial.read_u16_le()?;

    // 原生试读成功时沿内联记录继续，失败时从原偏移读取后续字段。
    let record_set = if prefix.version == 10.0 {
        read_temporal_record_set_10_1(&mut trial, max).ok()
    } else {
        None
    };
    read_trailing_fields(&mut trial, max, &mut staged, prefix.version)?;
    *reader = trial;
    *refs = staged;
    Ok(TemporalFeatureLayer10_1 {
        prefix,
        legend,
        record_set,
    })
}
