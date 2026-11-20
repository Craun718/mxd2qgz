//! TrackingTools.dll 10.1 的跟踪过滤扩展。

use super::image_server_rules::unsupported_class;
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_atl_bstr_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub const TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID: [u8; 16] = [
    0xee, 0x72, 0x01, 0x39, 0x0d, 0x6b, 0xe5, 0x41, 0xa2, 0xf5, 0xee, 0x4a, 0x17, 0x38, 0x85, 0xfb,
];

#[derive(Debug, Clone, PartialEq)]
pub struct TrackingFilterLayerExtension10_1 {
    /// 原生 Save 写 1.0，Load 读取后不与常量比较。
    pub version: f64,
    /// 持久化接口 +0x20 的整数原值。
    pub value_20: i32,
    pub properties: [Option<Box<PropertySetObjectStream10_1>>; 2],
    /// 保留 BSTR 的字节序列，区别空指针和零长度的 BSTR。
    pub strings: [Option<Vec<u8>>; 2],
}

fn read_properties(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Option<Box<PropertySetObjectStream10_1>>> {
    let vt = r.read_u16_le()?;
    if vt != 13 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("TrackingFilterLayerExtension 未支持的 ATL VARTYPE {vt}"),
        ));
    }
    // ReadClassStm 后直接激活并加载对象。只有属性集内层可经原流登记键。
    let class = r.read_bytes(16)?.try_into().unwrap();
    match class {
        class if class == [0; 16] => Ok(None),
        PROPERTY_SET_CLASS_ID => {
            read_property_set_object_stream_10_1(r, max, refs).map(|body| Some(Box::new(body)))
        }
        _ => Err(unsupported_class(
            "TrackingFilterLayerExtension IPropertySet",
            &class,
        )),
    }
}

pub(super) fn read_bounded_tracking_filter_layer_extension(
    bytes: &[u8],
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<Box<TrackingFilterLayerExtension10_1>> {
    let mut r = Reader::new(bytes);
    let mut staged = refs.clone();
    let body = TrackingFilterLayerExtension10_1 {
        version: r.read_f64_le()?,
        value_20: r.read_u32_le()? as i32,
        properties: [
            read_properties(&mut r, max, &mut staged)?,
            read_properties(&mut r, max, &mut staged)?,
        ],
        strings: [
            read_atl_bstr_10_1(&mut r, max)?,
            read_atl_bstr_10_1(&mut r, max)?,
        ],
    };
    if r.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TrackingFilterLayerExtension 对象体消费与声明长度不一致",
        ));
    }
    *refs = staged;
    Ok(Box::new(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::raster_layer_tail::read_raster_layer_tail_10_1;
    use crate::objects::property_set::PropertyValueObjectStream10_1;

    #[test]
    fn tracking_filter_reads_atl_objects_without_keys_and_shares_inner_refs() {
        let class = TRACKING_FILTER_LAYER_EXTENSION_CLASS_ID;
        let mut body = 1_f64.to_le_bytes().to_vec();
        body.extend((-3_i32).to_le_bytes());
        for first in [true, false] {
            body.extend(13_u16.to_le_bytes());
            body.extend(PROPERTY_SET_CLASS_ID);
            body.extend(1_u16.to_le_bytes());
            body.extend(1_u32.to_le_bytes());
            body.extend([2, 0, 0, 0, 0, 0]); // 空属性名。
            body.extend(13_u16.to_le_bytes());
            body.extend(PROPERTY_SET_CLASS_ID);
            body.extend(92_u32.to_le_bytes());
            if first {
                body.extend(1_u16.to_le_bytes());
                body.extend(0_u32.to_le_bytes());
            }
        }
        body.extend(5_u32.to_le_bytes());
        body.extend([0xff, 0, 0x20, 0, 0]); // 三个原始 BSTR 字节及两个终止字节。
        body.extend(2_u32.to_le_bytes());
        body.extend([0, 0]); // 非空指针的空 BSTR。

        for version in [3, 8] {
            let build = |body: &[u8]| {
                let mut bytes = vec![0; 26];
                bytes.extend(1_i32.to_le_bytes());
                if version == 3 {
                    bytes.extend((body.len() as u64).to_le_bytes());
                    bytes.extend(class);
                    bytes.extend(body);
                } else {
                    bytes.extend((20 + body.len() as u32).to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                    bytes.extend(class);
                    bytes.extend(91_u32.to_le_bytes());
                    bytes.extend(body);
                    bytes.extend([0; 8 + 32 + 8 + 16]);
                }
                bytes
            };

            let bytes = build(&body);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let tail = read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).unwrap();
            let parsed = if version == 3 {
                let mut legacy = Reader::new(&bytes[26..]);
                let mut legacy_refs = RefTable10_1::new();
                let shared = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut legacy,
                    4096,
                    &mut legacy_refs,
                )
                .unwrap();
                assert_eq!(legacy.remaining(), 0);
                assert_eq!(legacy_refs.len(), 1);
                let entry = &tail.legacy_extensions[0].tracking_filter_layer_extension;
                assert_eq!(entry, &shared[0].tracking_filter_layer_extension);
                assert_eq!(refs.len(), 1);
                entry
            } else {
                assert!(refs.contains(91));
                assert_eq!(refs.len(), 2);
                &tail.extra_safe_extensions[0].tracking_filter_layer_extension
            }
            .as_ref()
            .unwrap();
            assert_eq!(reader.remaining(), 0);
            assert!(refs.contains(92));
            assert_eq!(parsed.version, 1.0);
            assert_eq!(parsed.value_20, -3);
            assert_eq!(parsed.strings, [Some(vec![0xff, 0, 0x20]), Some(vec![])]);
            let first = parsed.properties[0].as_ref().unwrap();
            assert!(matches!(
                first.entries[0].value,
                PropertyValueObjectStream10_1::Object { key: 92, .. }
            ));
            let second = parsed.properties[1].as_ref().unwrap();
            assert!(matches!(
                second.entries[0].value,
                PropertyValueObjectStream10_1::BackReference { key: 92, .. }
            ));

            let mut bad_length = body.clone();
            let last_length = bad_length.len() - 6;
            bad_length[last_length..last_length + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            let mut bad_class = body.clone();
            bad_class[14..30].fill(0x42);
            for invalid in [&body[..body.len() - 1], &bad_length, &bad_class] {
                let bytes = build(invalid);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_raster_layer_tail_10_1(&mut reader, version, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }
}
