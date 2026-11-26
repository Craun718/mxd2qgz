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
