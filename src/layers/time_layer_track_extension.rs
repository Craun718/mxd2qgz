//! MapAnimation.dll 10.1 的图层时间轨道扩展与 TimeWindow 持久化布局。

use super::image_server_rules::{RuleSlot10_1, slot, unsupported_class};
use super::raster_dataset_name::{RasterLongArraySlot10_1, read_long_array_slot};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const TIME_LAYER_TRACK_EXTENSION_CLASS_ID: [u8; 16] = [
    0xc5, 0x44, 0xdf, 0x99, 0xd3, 0x02, 0xf2, 0x4c, 0xad, 0x42, 0x59, 0xfd, 0x26, 0xb6, 0x14, 0xfa,
];
const TIME_WINDOW_CLASS_ID: [u8; 16] = [
    0x3a, 0x3b, 0xba, 0x01, 0xab, 0x80, 0x3b, 0x42, 0xbb, 0x7c, 0x93, 0x13, 0x7c, 0xda, 0xaf, 0xe5,
];

#[derive(Debug, Clone, PartialEq)]
pub struct TimeLayerTrackExtension10_1 {
    pub version: u16,
    pub long_array: RasterLongArraySlot10_1,
    pub windows: Vec<RuleSlot10_1<TimeWindow10_1>>,
    pub shorts_v2: Option<[u16; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeWindow10_1 {
    pub version: u16,
    pub strings: [String; 2],
    pub fields_v2: Option<([String; 2], u16)>,
    pub string_v3: Option<String>,
}

pub(super) fn read_bounded_time_layer_track_extension(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<TimeLayerTrackExtension10_1>> {
    let mut body = Reader::new(bytes);
    let mut staged = refs.clone();
    let version = body.read_u16_le()?;
    if !(1..=2).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "不支持的 TimeLayerTrackExtension 版本",
        ));
    }
    let long_array = read_long_array_slot(&mut body, max, &mut staged)?;
    let count = (body.read_u32_le()? as i32).max(0) as usize;
    if count > body.remaining().min(max) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ITimeWindow 数量超过可读范围",
        ));
    }
    let windows = (0..count)
        .map(|_| {
            let mut window = slot(&mut body, &mut staged, |r, _, class_id, _| {
                if class_id != TIME_WINDOW_CLASS_ID {
                    return Err(unsupported_class("ITimeWindow", &class_id));
                }
                read_time_window(r, max)
            })?;
            if let ObjectEntry10_1::BackReference { class_id, key } = &mut window.entry
                && let Some(registered) = staged.registered_class(*key)
            {
                if registered != TIME_WINDOW_CLASS_ID {
                    return Err(unsupported_class("ITimeWindow 回引", &registered));
                }
                *class_id = registered;
            }
            Ok(window)
        })
        .collect::<io::Result<_>>()?;
    let shorts_v2 = (version >= 2)
        .then(|| -> io::Result<_> {
            Ok([
                body.read_u16_le()?,
                body.read_u16_le()?,
                body.read_u16_le()?,
            ])
        })
        .transpose()?;
    if body.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TimeLayerTrackExtension 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(TimeLayerTrackExtension10_1 {
        version,
        long_array,
        windows,
        shorts_v2,
    }))
}

fn read_time_window(reader: &mut Reader<'_>, max: usize) -> io::Result<TimeWindow10_1> {
    let version = reader.read_u16_le()?;
    if !(1..=3).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "不支持的 TimeWindow 版本",
        ));
    }
    let strings = [
        read_string_10_1(reader, max)?,
        read_string_10_1(reader, max)?,
    ];
    let fields_v2 = (version >= 2)
        .then(|| -> io::Result<_> {
            Ok((
                [
                    read_string_10_1(reader, max)?,
                    read_string_10_1(reader, max)?,
                ],
                reader.read_u16_le()?,
            ))
        })
        .transpose()?;
    let string_v3 = (version >= 3)
        .then(|| read_string_10_1(reader, max))
        .transpose()?;
    Ok(TimeWindow10_1 {
        version,
        strings,
        fields_v2,
        string_v3,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::raster_dataset_name::LONG_ARRAY_CLASS_ID;
    use crate::layers::raster_layer_tail::read_raster_layer_tail_10_1;
    use crate::storage::object_refs::ObjectEntry10_1;

    #[test]
    fn time_track_window_keys_and_version_fields_in_both_raster_wrappers() {
        for (raster_version, version) in [(3, 1_u16), (8, 2)] {
            let mut track = version.to_le_bytes().to_vec();
            track.extend(LONG_ARRAY_CLASS_ID);
            track.extend(91_u32.to_le_bytes());
            track.extend(1_u16.to_le_bytes());
            track.extend(1_u32.to_le_bytes());
            track.extend((-6_i32).to_le_bytes());
            track.extend(4_u32.to_le_bytes());
            for window_version in 1_u16..=3 {
                track.extend(TIME_WINDOW_CLASS_ID);
                track.extend((100 + u32::from(window_version)).to_le_bytes());
                track.extend(window_version.to_le_bytes());
                let mut push_string = || {
                    track.extend(4_u32.to_le_bytes());
                    track.extend(('窗' as u16).to_le_bytes());
                    track.extend(0_u16.to_le_bytes());
                };
                push_string();
                push_string();
                if window_version >= 2 {
                    push_string();
                    push_string();
                    track.extend(7_u16.to_le_bytes());
                }
                if window_version == 3 {
                    track.extend(2_u32.to_le_bytes());
                    track.extend(0_u16.to_le_bytes());
                }
            }
            let reference_offset = track.len();
            track.extend([0x7d; 16]);
            track.extend(101_u32.to_le_bytes());
            if version == 2 {
                track.extend([1, 0, 2, 0, 3, 0]);
            }

            let wrap = |track: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(1_i32.to_le_bytes());
                let header_len = if raster_version == 8 { 20 } else { 0 };
                bytes.extend(((track.len() + header_len) as u64).to_le_bytes());
                bytes.extend(TIME_LAYER_TRACK_EXTENSION_CLASS_ID);
                if raster_version == 8 {
                    bytes.extend(90_u32.to_le_bytes());
                }
                bytes.extend(track);
                if raster_version == 8 {
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };
            let bytes = wrap(&track);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail =
                read_raster_layer_tail_10_1(&mut reader, raster_version, 4096, &mut refs).unwrap();
            let track_body = if raster_version == 3 {
                assert_eq!(refs.len(), 4);
                tail.legacy_extensions[0]
                    .time_layer_track_extension
                    .as_deref()
                    .unwrap()
            } else {
                assert_eq!(refs.len(), 5);
                tail.extra_safe_extensions[0]
                    .time_layer_track_extension
                    .as_deref()
                    .unwrap()
            };
            assert_eq!(track_body.long_array.body, Some((1, vec![-6])));
            assert!(
                track_body.windows[0]
                    .body
                    .as_ref()
                    .unwrap()
                    .fields_v2
                    .is_none()
            );
            assert_eq!(
                track_body.windows[1]
                    .body
                    .as_ref()
                    .unwrap()
                    .fields_v2
                    .as_ref()
                    .unwrap()
                    .1,
                7
            );
            assert_eq!(
                track_body.windows[2]
                    .body
                    .as_ref()
                    .unwrap()
                    .string_v3
                    .as_deref(),
                Some("")
            );
            assert!(matches!(
                track_body.windows[3].entry,
                ObjectEntry10_1::BackReference {
                    class_id: TIME_WINDOW_CLASS_ID,
                    key: 101
                }
            ));
            assert_eq!(track_body.shorts_v2, (version == 2).then_some([1, 2, 3]));
            assert_eq!(reader.remaining(), 0);

            // 旧式和 ExtraSafe 包装均须回滚不受支持的版本。
            let first_window_version = 2 + 20 + 2 + 4 + 4 + 4 + 20;
            for (offset, limit) in [(0, 2_u16), (first_window_version, 3)] {
                for invalid in [0_u16, limit + 1, u16::MAX] {
                    let mut wrong_version = track.clone();
                    wrong_version[offset..offset + 2].copy_from_slice(&invalid.to_le_bytes());
                    let bytes = wrap(&wrong_version);
                    let mut reader = Reader::new(&bytes);
                    let mut refs = RefTable10_1::new();
                    refs.register(7);
                    let before = refs.clone();
                    let error =
                        read_raster_layer_tail_10_1(&mut reader, raster_version, 4096, &mut refs)
                            .unwrap_err();
                    assert_eq!(error.kind(), ErrorKind::Unsupported);
                    assert_eq!(reader.position(), 0);
                    assert_eq!(refs, before);
                }
            }

            let mut wrong = track.clone();
            wrong[reference_offset + 16..reference_offset + 20]
                .copy_from_slice(&91_u32.to_le_bytes());
            let mut staged = RefTable10_1::new();
            let error =
                read_bounded_time_layer_track_extension(&wrong, 4096, &mut staged).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Unsupported);
            assert!(staged.is_empty());

            let bytes = wrap(&track[..track.len() - 1]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            assert!(
                read_raster_layer_tail_10_1(&mut reader, raster_version, 4096, &mut refs).is_err()
            );
            assert!(refs.is_empty());
            assert_eq!(reader.position(), 0);
        }
    }
}
