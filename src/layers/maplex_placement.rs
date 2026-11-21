//! MaplexOverposterLayerProperties 的 10.1 流序与嵌套对象边界。

use std::io::{self, ErrorKind};

use super::annotate_props::{
    POINT_PLACEMENT_PRIORITIES_CLASS_ID, PointPlacementPrioritiesBody10_1,
    read_point_placement_priorities_body_10_1, read_stream_string_10_1,
};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;

pub(super) const MAPLEX_OVERPOSTER_LAYER_PROPS_CLASS_ID: [u8; 16] = [
    0x08, 0x48, 0x66, 0x20, 0xc9, 0x41, 0xd1, 0x11, 0x88, 0x0a, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

const MAPLEX_LABEL_STACKING_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x08, 0x48, 0x66, 0x20, 0xc9, 0x41, 0xd1, 0x11, 0x84, 0x0a, 0x08, 0xab, 0xc9, 0xed, 0x73, 0x1a,
];

const MAPLEX_OFFSET_ALONG_LINE_PROPERTIES_CLASS_ID: [u8; 16] = [
    0x08, 0x48, 0x66, 0x20, 0xa1, 0x4f, 0xd1, 0xc1, 0x8c, 0x0a, 0x08, 0xa2, 0xc9, 0xed, 0x53, 0x1a,
];

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexOffsetAlongLine10_1 {
    pub version: i16,
    /// v1 的流内计数与兼容记录，原生读完后丢弃。
    pub legacy_count: Option<i32>,
    pub legacy_entries: Vec<(String, [f64; 2])>,
    pub fields: Option<MaplexOffsetAlongLineFields10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexOffsetAlongLineFields10_1 {
    pub leading_longs: [i32; 2],
    pub doubles: [f64; 2],
    pub trailing_long: i32,
    pub trailing_short: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexLabelStacking10_1 {
    pub version: i16,
    /// 原生对象的首个 LONG。
    pub first_long: i32,
    pub separator_count: Option<i32>,
    pub separators: Vec<MaplexStackingSeparator10_1>,
    pub trailing_longs: Option<[i32; 3]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexStackingSeparator10_1 {
    pub text: String,
    /// 三组分隔符标志，保留流内原始值。
    pub flags: [i16; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexPlacement10_1 {
    pub version: u16,
    /// 从版本号到已确认边界的原始体，保留尚未命名的标量。
    pub raw_body: Vec<u8>,
    /// 顺序固定，版本中不存在或因前置停止未读到时为 None。
    pub slots: [Option<ObjectEntry10_1>; 6],
    pub offset_along_line: Option<MaplexOffsetAlongLine10_1>,
    pub stacking: Option<MaplexLabelStacking10_1>,
    pub point_priorities: Option<PointPlacementPrioritiesBody10_1>,
    pub properties: Option<PropertySetObjectStream10_1>,
    pub stop: Option<MaplexPlacementStop10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaplexPlacementStop10_1 {
    pub slot_index: usize,
    pub class_id: [u8; 16],
    pub key: u32,
}

pub(super) fn read_maplex_placement(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<MaplexPlacement10_1> {
    let mut r = reader.clone();
    let mut staged = refs.clone();
    let version = r.read_u16_le()?;
    if !(4..=22).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("MaplexOverposterLayerProperties 版本 {version} 超出 4–22"),
        ));
    }
    let mut body = MaplexPlacement10_1 {
        version,
        raw_body: Vec::new(),
        slots: [None; 6],
        offset_along_line: None,
        stacking: None,
        point_priorities: None,
        properties: None,
        stop: None,
    };
    'layout: {
        // LabelPlacement.dll 0x78566d40：各段按 10.1 Load 的读宽保留。
        r.read_bytes(5 * 2 + 6 * 4 + 2 * 8)?;
        if version <= 11 {
            read_stream_string_10_1(&mut r, max)?;
        }
        if !read_slot(&mut r, &mut staged, max, &mut body, 0)? {
            break 'layout;
        }
        r.read_bytes(5 * 2 + 4 * 8)?;
        if !read_slot(&mut r, &mut staged, max, &mut body, 1)? {
            break 'layout;
        }
        r.read_bytes(4 * 2 + 8)?;
        r.read_bytes(if version >= 7 { 4 * 8 } else { 4 * 4 })?;
        r.read_bytes(3 * 2 + 3 * 4 + 8)?;
        if version >= 6 {
            r.read_bytes(2)?;
        }
        if version >= 7 {
            r.read_bytes(2 + 8)?;
        }
        if (8..=13).contains(&version) && !read_slot(&mut r, &mut staged, max, &mut body, 2)? {
            break 'layout;
        }
        if version >= 9 {
            r.read_bytes(2)?;
            if !read_slot(&mut r, &mut staged, max, &mut body, 3)? {
                break 'layout;
            }
        }
        if version >= 10 {
            r.read_bytes(4)?;
        }
        if version >= 12 && !read_slot(&mut r, &mut staged, max, &mut body, 4)? {
            break 'layout;
        }
        if version >= 13 {
            r.read_bytes(4)?;
        }
        if version >= 15 {
            read_stream_string_10_1(&mut r, max)?;
        }
        r.read_bytes(usize::from(version.saturating_sub(15).min(4)) * 2)?;
        if version >= 20 {
            r.read_bytes(2 + 8)?;
        }
        if version >= 21 {
            r.read_bytes(8)?;
        }
        if version >= 22 {
            read_slot(&mut r, &mut staged, max, &mut body, 5)?;
        }
    }
    let length = r.position() - reader.position();
    body.raw_body = reader.read_bytes(length)?.to_vec();
    *refs = staged;
    Ok(body)
}

fn read_slot(
    reader: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    max: usize,
    body: &mut MaplexPlacement10_1,
    slot_index: usize,
) -> io::Result<bool> {
    let mut trial = reader.clone();
    let mut staged = refs.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    if let ObjectEntry10_1::NewObject { class_id, key } = entry {
        match (slot_index, class_id) {
            (0, POINT_PLACEMENT_PRIORITIES_CLASS_ID) => {
                body.point_priorities = Some(read_point_placement_priorities_body_10_1(
                    &mut trial,
                    max,
                    &mut staged,
                    key,
                )?);
            }
            (1, MAPLEX_LABEL_STACKING_PROPERTIES_CLASS_ID) => {
                body.stacking = Some(read_label_stacking(&mut trial, max)?);
            }
            (3, MAPLEX_OFFSET_ALONG_LINE_PROPERTIES_CLASS_ID) => {
                body.offset_along_line = Some(read_offset_along_line(&mut trial, max)?);
            }
            (5, PROPERTY_SET_CLASS_ID) => {
                body.properties = Some(read_property_set_object_stream_10_1(
                    &mut trial,
                    max,
                    &mut staged,
                )?);
            }
            _ => {
                body.stop = Some(MaplexPlacementStop10_1 {
                    slot_index,
                    class_id,
                    key,
                });
                return Ok(false);
            }
        }
    }
    body.slots[slot_index] = Some(entry);
    *reader = trial;
    *refs = staged;
    Ok(true)
}

fn read_label_stacking(reader: &mut Reader<'_>, max: usize) -> io::Result<MaplexLabelStacking10_1> {
    // LabelPlacement.dll 0x78562930：版本 2 起分隔符列表，版本 3 起三个 LONG。
    let version = reader.read_u16_le()? as i16;
    if version > 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("MaplexLabelStackingProperties 版本 {version} 超出 3"),
        ));
    }
    let first_long = reader.read_u32_le()? as i32;
    let mut separator_count = None;
    let mut separators = Vec::new();
    if version >= 2 {
        let count = reader.read_u32_le()? as i32;
        separator_count = Some(count);
        let count = count.max(0) as usize;
        if count > reader.remaining() / 10 || count > max / 10 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Maplex 分隔符计数超出可读范围",
            ));
        }
        for _ in 0..count {
            let text = read_stream_string_10_1(reader, max)?;
            let flags = [
                reader.read_u16_le()? as i16,
                reader.read_u16_le()? as i16,
                reader.read_u16_le()? as i16,
            ];
            separators.push(MaplexStackingSeparator10_1 { text, flags });
        }
    }
    let trailing_longs = if version >= 3 {
        Some([
            reader.read_u32_le()? as i32,
            reader.read_u32_le()? as i32,
            reader.read_u32_le()? as i32,
        ])
    } else {
        None
    };
    Ok(MaplexLabelStacking10_1 {
        version,
        first_long,
        separator_count,
        separators,
        trailing_longs,
    })
}

fn read_offset_along_line(
    reader: &mut Reader<'_>,
    max: usize,
) -> io::Result<MaplexOffsetAlongLine10_1> {
    // LabelPlacement.dll 0x785638d0：v1 兼容记录与 v2 固定字段互斥。
    let version = reader.read_u16_le()? as i16;
    if version > 2 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("MaplexOffsetAlongLineProperties 版本 {version} 超出 2"),
        ));
    }
    let mut body = MaplexOffsetAlongLine10_1 {
        version,
        legacy_count: None,
        legacy_entries: Vec::new(),
        fields: None,
    };
    if version == 1 {
        let count = reader.read_u32_le()? as i32;
        body.legacy_count = Some(count);
        let count = count.max(0) as usize;
        if count > reader.remaining() / 20 || count > max / 20 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Maplex 沿线偏移记录计数超出可读范围",
            ));
        }
        for _ in 0..count {
            let text = read_stream_string_10_1(reader, max)?;
            let values = [reader.read_f64_le()?, reader.read_f64_le()?];
            body.legacy_entries.push((text, values));
        }
    }
    if version == 2 {
        body.fields = Some(MaplexOffsetAlongLineFields10_1 {
            leading_longs: [reader.read_u32_le()? as i32, reader.read_u32_le()? as i32],
            doubles: [reader.read_f64_le()?, reader.read_f64_le()?],
            trailing_long: reader.read_u32_le()? as i32,
            trailing_short: reader.read_u16_le()? as i16,
        });
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_along_line_reads_legacy_rows_and_rejects_truncation() {
        let mut bytes = 1_i16.to_le_bytes().to_vec();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
        bytes.extend(1.25_f64.to_le_bytes());
        bytes.extend((-2.5_f64).to_le_bytes());
        for cut in 0..bytes.len() {
            assert!(read_offset_along_line(&mut Reader::new(&bytes[..cut]), 64).is_err());
        }
        let body = read_offset_along_line(&mut Reader::new(&bytes), 64).unwrap();
        assert_eq!(body.legacy_entries, vec![("X".into(), [1.25, -2.5])]);
        assert_eq!(body.fields, None);
    }

    #[test]
    fn stacking_distinguishes_legacy_versions_and_bounds_the_separator_count() {
        let mut bytes = 1_i16.to_le_bytes().to_vec();
        bytes.extend(4_i32.to_le_bytes());
        let body = read_label_stacking(&mut Reader::new(&bytes), 64).unwrap();
        assert_eq!(body.first_long, 4);
        assert_eq!(body.separator_count, None);

        bytes[..2].copy_from_slice(&2_i16.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        let body = read_label_stacking(&mut Reader::new(&bytes), 64).unwrap();
        assert_eq!(body.separator_count, Some(-1));
        assert!(body.separators.is_empty());
        assert_eq!(body.trailing_longs, None);

        bytes[6..10].copy_from_slice(&1_i32.to_le_bytes());
        assert!(read_label_stacking(&mut Reader::new(&bytes), 64).is_err());
        bytes[..2].copy_from_slice(&4_i16.to_le_bytes());
        assert!(read_label_stacking(&mut Reader::new(&bytes), 64).is_err());
    }

    #[test]
    fn placement_reads_versioned_boundaries_and_preserves_nested_keys() {
        for version in 4_u16..=22 {
            let mut bytes = version.to_le_bytes().to_vec();
            bytes.extend([0; 50]);
            if version <= 11 {
                bytes.extend(0_u32.to_le_bytes());
            }
            bytes.extend(POINT_PLACEMENT_PRIORITIES_CLASS_ID);
            bytes.extend(11_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            for priority in 1_i32..=8 {
                bytes.extend(priority.to_le_bytes());
            }
            bytes.extend([0; 42]);
            let second_slot = bytes.len();
            bytes.extend(MAPLEX_LABEL_STACKING_PROPERTIES_CLASS_ID);
            bytes.extend(14_u32.to_le_bytes());
            bytes.extend(3_i16.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend([4, 0, 0, 0, b'/', 0, 0, 0]);
            bytes.extend([0xff, 0xff, 0, 0, 1, 0]);
            for value in [4_i32, 8, 16] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend([0; 16]);
            bytes.resize(bytes.len() + if version < 7 { 16 } else { 32 }, 0);
            bytes.extend([0; 26]);
            if version >= 6 {
                bytes.extend([0; 2]);
            }
            if version >= 7 {
                bytes.extend([0; 10]);
            }
            if (8..=13).contains(&version) {
                bytes.extend([0; 16]);
            }
            if version >= 9 {
                bytes.extend([0; 2]);
                bytes.extend(MAPLEX_OFFSET_ALONG_LINE_PROPERTIES_CLASS_ID);
                bytes.extend(15_u32.to_le_bytes());
                bytes.extend(2_i16.to_le_bytes());
                bytes.extend(3_i32.to_le_bytes());
                bytes.extend(4_i32.to_le_bytes());
                bytes.extend(1.25_f64.to_le_bytes());
                bytes.extend((-2.5_f64).to_le_bytes());
                bytes.extend(5_i32.to_le_bytes());
                bytes.extend((-1_i16).to_le_bytes());
            }
            for (first_version, len) in [
                (10, 4),
                (12, 16),
                (13, 4),
                (15, 4),
                (16, 2),
                (17, 2),
                (18, 2),
                (19, 2),
                (20, 10),
                (21, 8),
            ] {
                if version >= first_version {
                    bytes.resize(bytes.len() + len, 0);
                }
            }
            if version == 22 {
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(12_u32.to_le_bytes());
                bytes.extend([0; 6]);
            }
            let end = bytes.len();
            bytes.push(0xcc);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let body = read_maplex_placement(&mut reader, 1024, &mut refs).unwrap();
            assert_eq!(reader.position(), end, "版本 {version}");
            assert_eq!(body.raw_body, bytes[..end]);
            assert_eq!(body.stop, None);
            assert_eq!(
                body.point_priorities.unwrap().ints,
                [1, 2, 3, 4, 5, 6, 7, 8]
            );
            let stacking = body.stacking.unwrap();
            assert_eq!(stacking.separators[0].text, "/");
            assert_eq!(stacking.separators[0].flags, [-1, 0, 1]);
            assert_eq!(stacking.trailing_longs, Some([4, 8, 16]));
            if version >= 9 {
                let offset = body.offset_along_line.unwrap().fields.unwrap();
                assert_eq!(offset.leading_longs, [3, 4]);
                assert_eq!(offset.doubles, [1.25, -2.5]);
                assert_eq!(offset.trailing_long, 5);
                assert_eq!(offset.trailing_short, -1);
            }
            assert_eq!(refs.contains(15), version >= 9);
            assert!(refs.contains(14));
            assert!(refs.contains(11));
            assert_eq!(refs.contains(12), version == 22);

            let mut truncated = Reader::new(&bytes[..end - 1]);
            let mut empty = RefTable10_1::new();
            assert!(read_maplex_placement(&mut truncated, 1024, &mut empty).is_err());
            assert_eq!(truncated.position(), 0);
            assert!(empty.is_empty());

            bytes.splice(
                second_slot..second_slot + 16,
                [0x7e; 16].into_iter().chain(13_u32.to_le_bytes()),
            );
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let body = read_maplex_placement(&mut reader, 1024, &mut refs).unwrap();
            assert_eq!(reader.position(), second_slot);
            assert_eq!(body.stop.unwrap().key, 13);
            assert!(refs.contains(11));
            assert!(!refs.contains(13));
        }
    }
}
