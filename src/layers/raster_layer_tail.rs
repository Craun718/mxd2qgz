//! RasterLayer.dll 10.1 在 IRasterRenderer 后的对象体。原生读写与取证偏移见
//! docs/mxd-10.1-raster-layer-tail.md。

use crate::layers::dep_tail::{
    TimeDataSegment10_1, TimeDimensionSegment10_1, TimeDisplaySegment10_1,
    read_time_data_segment_10_1, read_time_dimension_segment_10_1, read_time_display_segment_10_1,
};
use crate::layers::raster_renderer::{
    RasterHistogram10_1, STATS_HISTOGRAM_CLASS_ID, read_stats_histogram_body_10_1,
};
use crate::objects::dataset_name_more::{
    MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID, MemoryRelationshipClassNameBody10_1,
    RELATIONSHIP_CLASS_NAME_CLASS_ID, RelationshipClassNameBody10_1,
    read_memory_relationship_class_name_body_10_1, read_relationship_class_name_body_10_1,
};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::geodata_xform::{GeodataXformSlot10_1, read_geodata_xform_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerTail10_1 {
    /// 保存到持久化接口 this+0xbc、+0xc4 的两个原值。
    pub doubles_bc_c4: [f64; 2],
    pub long_f4: i32,
    pub short_94: u16,
    pub long_100: i32,
    pub double_104: f64,
    /// 内联 WKSEnvelope 的四个原始位模式。空包络可以含 NaN。
    pub wks_bits: [u64; 4],
    pub related_name: Option<RasterLayerRelationshipSlot10_1>,
    pub related_objects: Vec<RasterLayerRelationshipSlot10_1>,
    pub envelope_f0: RasterLayerEnvelopeSlot10_1,
    pub raster_io: RasterLayerIo10_1,
    pub string_128: String,
    pub doubles_12c_134: [f64; 2],
    pub geodata_xform_v13: GeodataXformSlot10_1,
    pub envelope_v13: RasterLayerEnvelopeSlot10_1,
    pub color_v14: Option<ColorSlot10_1>,
    pub time: Option<Box<RasterLayerTime10_1>>,
    pub pixel_filter_v16: Option<ObjectEntry10_1>,
    pub histograms_v17: Vec<RasterLayerHistogramSlot10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterLayerRelationshipBody10_1 {
    Relationship(Box<RelationshipClassNameBody10_1>),
    MemoryRelationship(Box<MemoryRelationshipClassNameBody10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerRelationshipSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterLayerRelationshipBody10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerHistogramSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<RasterHistogram10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerEnvelopeSlot10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<Box<Envelope10_1>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterLayerIo10_1 {
    /// 仅支持省略或显式空对象；保持两种记录形态的区别。
    pub tag30_is_explicit_null: bool,
    pub tag31_string: Option<String>,
    pub shorts_35_to_37: [u16; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerTime10_1 {
    pub data: TimeDataSegment10_1,
    pub display: TimeDisplaySegment10_1,
    pub dimension: TimeDimensionSegment10_1,
}

fn unsupported(label: &str) -> io::Error {
    io::Error::new(ErrorKind::Unsupported, format!("RasterLayer {label}"))
}

fn invalid(label: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, format!("RasterLayer {label}"))
}

fn read_long(reader: &mut Reader<'_>) -> io::Result<i32> {
    Ok(reader.read_u32_le()? as i32)
}

fn read_known_header(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    label: &str,
) -> io::Result<ObjectEntry10_1> {
    let entry = read_object_entry_10_1(reader, table)?;
    if let ObjectEntry10_1::NewObject { class_id, .. } = entry {
        return Err(unsupported(&format!(
            "{label} 含未支持的对象类 {}",
            crate::storage::class_id_hex(&class_id)
        )));
    }
    Ok(entry)
}

fn read_slots<T>(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    label: &str,
    read_entry: impl Fn(&mut Reader<'_>, &mut RefTable10_1) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let count = read_long(reader)?.max(0) as usize;
    if count > reader.remaining() / 16 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            format!("RasterLayer {label} 计数超出可读范围"),
        ));
    }
    (0..count).map(|_| read_entry(reader, table)).collect()
}

fn read_relationship_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerRelationshipSlot10_1> {
    let entry = read_object_entry_10_1(reader, table)?;
    use RasterLayerRelationshipBody10_1::{MemoryRelationship, Relationship};
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body = read_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            if body.stop.is_some() {
                return Err(unsupported("RelationshipClassName 含未闭合的嵌套对象"));
            }
            Some(Relationship(Box::new(body)))
        }
        ObjectEntry10_1::NewObject {
            class_id: MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID,
            key,
        } => {
            let body =
                read_memory_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            if body.stop.is_some() {
                return Err(unsupported(
                    "MemoryRelationshipClassName 含未闭合的嵌套对象",
                ));
            }
            Some(MemoryRelationship(Box::new(body)))
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "关联 IName 槽含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerRelationshipSlot10_1 { entry, body })
}

fn read_histogram_slot(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerHistogramSlot10_1> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: STATS_HISTOGRAM_CLASS_ID,
            ..
        } => Some(Box::new(read_stats_histogram_body_10_1(reader)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "v17 IStatsHistogram 槽含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerHistogramSlot10_1 { entry, body })
}

pub(super) fn read_envelope_slot(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerEnvelopeSlot10_1> {
    let entry = read_object_entry_10_1(reader, table)?;
    let body = match entry {
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
        ObjectEntry10_1::NewObject {
            class_id: ENVELOPE_CLASS_ID,
            ..
        } => Some(Box::new(read_envelope_10_1(reader, max_bytes, table)?)),
        ObjectEntry10_1::NewObject { class_id, .. } => {
            return Err(unsupported(&format!(
                "IEnvelope 槽含未支持的对象类 {}",
                crate::storage::class_id_hex(&class_id)
            )));
        }
    };
    Ok(RasterLayerEnvelopeSlot10_1 { entry, body })
}

fn expect_record(reader: &mut Reader<'_>, tag: i32, length: i32) -> io::Result<()> {
    let actual_tag = read_long(reader)?;
    let actual_length = read_long(reader)?;
    if (actual_tag, actual_length) != (tag, length) {
        return Err(invalid(&format!(
            "RasterIO 预期 ({tag},{length})，实际 ({actual_tag},{actual_length})"
        )));
    }
    Ok(())
}

fn read_raster_io(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<RasterLayerIo10_1> {
    expect_record(reader, 0xff00ff, 0)?;
    let tag30_is_explicit_null = read_long(&mut reader.clone())? == 30;
    if tag30_is_explicit_null {
        expect_record(reader, 30, -1)?;
        if reader.read_bytes(16)? != [0; 16] {
            return Err(unsupported("RasterIO tag 30 的非空对象尚未支持"));
        }
    }
    let tag31_string = if read_long(&mut reader.clone())? == 31 {
        reader.read_u32_le()?;
        // RasterIO 宽字符串记录的 length 是含结束零的字节数，与此读取器一致。
        Some(read_string_10_1(reader, max_bytes)?)
    } else {
        None
    };
    let mut shorts_35_to_37 = [0; 3];
    for (tag, value) in (35..=37).zip(&mut shorts_35_to_37) {
        expect_record(reader, tag, 2)?;
        *value = reader.read_u16_le()?;
    }
    expect_record(reader, -1, -1)?;
    Ok(RasterLayerIo10_1 {
        tag30_is_explicit_null,
        tag31_string,
        shorts_35_to_37,
    })
}

/// 从已闭合的 IRasterRenderer 之后读取较新版 RasterLayer 的有界尾段。
/// 任何不支持分支或截断均回退到渲染器后的起点，不污染对象键。
pub(crate) fn read_raster_layer_tail_10_1(
    reader: &mut Reader<'_>,
    version: i32,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerTail10_1> {
    if !(13..=17).contains(&version) {
        return Err(unsupported(&format!("尚未接入 v{version} 尾段")));
    }
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let doubles_bc_c4 = [trial.read_f64_le()?, trial.read_f64_le()?];
    let long_f4 = read_long(&mut trial)?;
    let short_94 = trial.read_u16_le()?;
    let long_100 = read_long(&mut trial)?;
    if read_long(&mut trial)? != 0 {
        return Err(unsupported("非空 ExtraSafe 扩展列表尚未支持"));
    }
    let double_104 = trial.read_f64_le()?;
    let mut wks_bits = [0; 4];
    for bits in &mut wks_bits {
        *bits = u64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap());
    }
    let related_name = (read_long(&mut trial)? != 0)
        .then(|| read_relationship_slot(&mut trial, max_bytes, &mut staged))
        .transpose()?;
    let related_objects = read_slots(&mut trial, &mut staged, "关联对象列表", |r, t| {
        read_relationship_slot(r, max_bytes, t)
    })?;
    let envelope_f0 = read_envelope_slot(&mut trial, max_bytes, &mut staged)?;
    let raster_io = read_raster_io(&mut trial, max_bytes)?;
    let string_128 = read_string_10_1(&mut trial, max_bytes)?;
    let doubles_12c_134 = [trial.read_f64_le()?, trial.read_f64_le()?];
    let geodata_xform_v13 = read_geodata_xform_slot_10_1(&mut trial, max_bytes, &mut staged)?;
    let envelope_v13 = read_envelope_slot(&mut trial, max_bytes, &mut staged)?;
    let color_v14 = if version >= 14 {
        let color = read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?;
        if matches!(color, ColorSlot10_1::Unsupported { .. }) {
            return Err(unsupported("v14 IColor 类尚未支持"));
        }
        if read_long(&mut trial)? != 0 {
            return Err(unsupported("v14 非空压缩索引列表尚未支持"));
        }
        Some(color)
    } else {
        None
    };
    let time = if version >= 15 {
        Some(Box::new(RasterLayerTime10_1 {
            data: read_time_data_segment_10_1(&mut trial, max_bytes, &mut staged)?,
            display: read_time_display_segment_10_1(&mut trial)?,
            dimension: read_time_dimension_segment_10_1(&mut trial, max_bytes)?,
        }))
    } else {
        None
    };
    let pixel_filter_v16 = (version >= 16)
        .then(|| read_known_header(&mut trial, &mut staged, "v16 IPixelFilter"))
        .transpose()?;
    let histograms_v17 = if version >= 17 {
        read_slots(
            &mut trial,
            &mut staged,
            "v17 IStatsHistogram 列表",
            read_histogram_slot,
        )?
    } else {
        Vec::new()
    };
    if trial.remaining() != 0 {
        return Err(invalid("尾段终点与 ExtraSafe 条目边界不一致"));
    }
    *table = staged;
    *reader = trial;
    Ok(RasterLayerTail10_1 {
        doubles_bc_c4,
        long_f4,
        short_94,
        long_100,
        double_104,
        wks_bits,
        related_name,
        related_objects,
        envelope_f0,
        raster_io,
        string_128,
        doubles_12c_134,
        geodata_xform_v13,
        envelope_v13,
        color_v14,
        time,
        pixel_filter_v16,
        histograms_v17,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_consumes_v17_only_with_matching_end_and_rolls_back_envelope_key() {
        let mut bytes = vec![0; 0x4e]; // 标量、空扩展、内联范围、选择器和关联计数。
        bytes.extend(ENVELOPE_CLASS_ID);
        bytes.extend(51_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 32]); // Envelope v1 不带嵌套坐标系。
        bytes.extend(0x00ff_00ff_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        for tag in 35_i32..=37 {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(0_u16.to_le_bytes());
        }
        bytes.extend([0xff; 8]);
        let mut put_empty_string = 2_u32.to_le_bytes().to_vec();
        put_empty_string.extend(0_u16.to_le_bytes());
        bytes.extend(&put_empty_string);
        bytes.extend([0; 16]); // v11 两个 double。
        bytes.extend(crate::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID);
        bytes.extend(53_u32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 16]); // Identity 的空空间参考。
        bytes.extend([0; 16]); // v13 空包络。
        bytes.extend([0; 20]); // v14 空颜色与零索引计数。
        bytes.extend(2_u16.to_le_bytes()); // TimeData。
        bytes.extend([0; 20]);
        bytes.extend(1_u16.to_le_bytes()); // TimeDataDisplay。
        bytes.extend([0; 26]);
        bytes.extend(1_u16.to_le_bytes()); // TimeDimensionDefinition。
        bytes.extend(&put_empty_string);
        bytes.extend(&put_empty_string);
        bytes.extend([0; 16]); // v16 空对象。
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(STATS_HISTOGRAM_CLASS_ID);
        bytes.extend(52_u32.to_le_bytes());
        bytes.extend(5_u32.to_le_bytes());
        bytes.extend([0; 48]); // 六个统计值。
        bytes.extend(1_u32.to_le_bytes()); // 一个直方图桶。
        bytes.extend(10_f64.to_le_bytes());
        bytes.extend([0; 16]); // 从版本 2 与 4 起的 double。
        bytes.extend(0_u32.to_le_bytes()); // v5 空追加数组。

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let tail = read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).unwrap();
        assert!(tail.envelope_f0.body.is_some());
        assert!(refs.contains(51));
        assert!(refs.contains(52));
        assert!(refs.contains(53));
        assert!(tail.geodata_xform_v13.body.is_some());
        assert_eq!(tail.histograms_v17[0].body.as_ref().unwrap().bins, [10.0]);
        assert_eq!(reader.remaining(), 0);

        let complete_end = bytes.len();
        bytes.push(0x90);
        for cut in (0..complete_end).chain([bytes.len()]) {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(
                read_raster_layer_tail_10_1(&mut reader, 17, 4096, &mut refs).is_err(),
                "cut={cut}"
            );
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(51));
            assert!(!refs.contains(52));
            assert!(!refs.contains(53));
        }
    }
}
