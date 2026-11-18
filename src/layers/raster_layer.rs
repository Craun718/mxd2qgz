//! ArcMap 10.1 RasterLayer 对象体的已确认前缀。
//!
//! 证据来自 RasterLayer.dll 10.1 安装介质静态映像。文件 SHA-256 为
//! `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`，PE32
//! x86，ImageBase `0x7c710000`。主 QI `0x7c73aaa0` 将 IPersistStream IID
//! （`0x7c76778c`）映射到对象偏移 `+0x9c`；该偏移与 RTTI COL
//! `0x7c774088` 中的 `offset=0x9c` 一致。对应 IPersistStream 虚表
//! `0x7c76c2b0` 的 Load 槽 `+0x14` 指向 `0x7c73bd40`。
//!
//! Load `0x7c73bd40` 首先读有符号 int32 版本（`JG` 拒绝大于 17），随后
//! 读取第一条 AfCore String 到 persist-interface this+0x88。独立续段可再读
//! 第二串、定长短值及版本控制的 IName 列表。

use super::raster_dataset_name::{
    RasterDatasetNameEntryBody10_1, read_supported_raster_name_entry_10_1,
};
use super::raster_renderer::read_raster_renderer_slot_prefix_10_1;
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub use super::raster_layer_tail::RasterLayerTail10_1;

/// RasterLayer 10.1 Load 的已确认前缀。
#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerPrefix10_1 {
    /// 有符号 int32 版本；原生 `JG` 只拒绝大于 17 的值，负值按原生门通过。
    pub version: i32,
    /// RasterLayer::Load 的首条 String，即 ILayer.Name（完整对象 +0x124）。
    pub first_string: String,
    /// 仅在外层已知边界内成功读取续段时填入。
    pub continuation: Option<RasterLayerContinuation10_1>,
    /// 已完整读取的名称条目；遇到未知对象时保留其之前的条目。
    pub names: Vec<RasterLayerName10_1>,
    pub names_stop: Option<String>,
    /// 仅在 IName 列表结束处读取；新渲染器只消费已确认的前缀。
    pub renderer: Option<Box<crate::layers::raster_renderer::RasterRendererSlot10_1>>,
    /// 已定位到槽位但无法读取时的原因；调用者仍保留未知体。
    pub renderer_stop: Option<String>,
    /// 渲染器完整消费后，继续读取的父图层尾段。
    pub tail: Option<Box<RasterLayerTail10_1>>,
    /// 尾段读失败时的原因；未解析字节仍由外层保留。
    pub tail_stop: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterLayerName10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<RasterDatasetNameEntryBody10_1>,
    pub suffix: Option<String>,
    /// v5–v8 的 int32 选择子，非零时带一个 int32。
    pub legacy: Option<(i32, Option<i32>)>,
}

/// RasterLayer 第二条 String、三个定长短值及 `0x7c73b380` 循环首对象头。
///
/// 对象体仍未解析；`first_object_header` 对应原生首个 `LoadObject(IName)`
/// 的空标记或 GUID 与键，不代表该对象已加载。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterLayerContinuation10_1 {
    /// Load 中的第二条 String，原生先读入栈上临时 String；业务含义未定。
    pub second_string: String,
    /// 按原生顺序紧邻的三个 short 原值；第一个是 ILayer.Visible。
    pub fixed_shorts: [u16; 3],
    /// 版本大于 8 时 `0x7c73b380` 读取的有符号数量；其余版本没有该流字段。
    pub object_count: Option<i32>,
    /// `0x7c73b380` 循环首个 `LoadObject` 的对象头；对象体及后续循环均未读取。
    pub first_object_header: Option<ObjectHeader>,
}

/// 从 RasterLayer 对象头之后读取 10.1 已确认前缀。
///
/// 成功后游标停在第二条 String 之前。`max_bytes` 限制首条 String 的声明
/// 长度；任何错误均不推进调用者游标。
pub fn read_raster_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<RasterLayerPrefix10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let version = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if version > 17 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "RasterLayer 对象体 offset 0x{start:x}: 版本 {version} 超出 Load 接受范围（有符号比较 >17 拒绝）"
            ),
        ));
    }
    let first_string = read_string_10_1(&mut trial, max_bytes)?;
    *reader = trial;
    Ok(RasterLayerPrefix10_1 {
        version,
        first_string,
        continuation: None,
        names: Vec::new(),
        names_stop: None,
        renderer: None,
        renderer_stop: None,
        tail: None,
        tail_stop: None,
    })
}

/// 从首条 String 之后继续读 RasterLayer 10.1 的有界前缀。
///
/// 调用者须先成功调用 [`read_raster_layer_prefix_10_1`]，并传入其版本。
/// 成功后最多越过第二条 String、三个 short、版本相关数量和
/// `0x7c73b380` 循环首对象头；该对象体从返回游标处开始。主 Load 在该辅助函数
/// 返回后还会读取其他字段和另一个对象，本函数不推进到那些字段。任何错误都不
/// 推进输入。
pub fn read_raster_layer_continuation_10_1(
    reader: &mut Reader<'_>,
    version: i32,
    max_string_bytes: usize,
) -> io::Result<RasterLayerContinuation10_1> {
    let mut trial = reader.clone();
    let mut result = read_continuation_fields(&mut trial, version, max_string_bytes)?;
    if result.object_count.unwrap_or(1) > 0 {
        result.first_object_header = Some(read_header_10_1(&mut trial)?);
    }
    *reader = trial;
    Ok(result)
}

fn read_continuation_fields(
    reader: &mut Reader<'_>,
    version: i32,
    max_string_bytes: usize,
) -> io::Result<RasterLayerContinuation10_1> {
    let mut trial = reader.clone();
    if version > 17 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("RasterLayer 续段版本 {version} 超出 Load 接受范围（>17 拒绝）"),
        ));
    }
    let second_string = read_string_10_1(&mut trial, max_string_bytes)?;
    let fixed_shorts = [
        trial.read_u16_le()?,
        trial.read_u16_le()?,
        trial.read_u16_le()?,
    ];

    // RasterLayer::Load 0x7c73bd40 调 0x7c73b380；后者仅在版本 >8 时
    // 从流中覆盖初始数量 1。负数量按原生循环条件不进入首对象读取。
    let object_count = if version > 8 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    *reader = trial;
    Ok(RasterLayerContinuation10_1 {
        second_string,
        fixed_shorts,
        object_count,
        first_object_header: None,
    })
}

fn read_raster_layer_names(
    body: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    prefix: &mut RasterLayerPrefix10_1,
    raw_count: i32,
) -> io::Result<()> {
    // 0x7c73b380 没有可用名称时返回失败，主 Load 不进入渲染器。
    let count = usize::try_from(raw_count).unwrap_or_default();
    if count == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterLayer 没有可用名称",
        ));
    }
    let version = prefix.version;
    let legacy = (5..=8).contains(&version);
    let min_size = 16 + usize::from(version > 0) * 6 + usize::from(legacy) * 4;
    if count > max_bytes / min_size || count > body.remaining() / min_size {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterLayer 名称计数超出限额或剩余范围",
        ));
    }
    let mut any_name = false;
    for index in 0..count {
        let mut trial = body.clone();
        let mut staged = refs.clone();
        let result = (|| -> io::Result<RasterLayerName10_1> {
            let (entry, name) =
                read_supported_raster_name_entry_10_1(&mut trial, max_bytes, &mut staged)?;
            let suffix = (version > 0)
                .then(|| read_string_10_1(&mut trial, max_bytes))
                .transpose()?;
            let legacy = legacy
                .then(|| -> io::Result<_> {
                    let selector = trial.read_u32_le()? as i32;
                    let index = (selector != 0)
                        .then(|| trial.read_u32_le().map(|value| value as i32))
                        .transpose()?;
                    Ok((selector, index))
                })
                .transpose()?;
            Ok(RasterLayerName10_1 {
                entry,
                body: name,
                suffix,
                legacy,
            })
        })()
        .map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("RasterLayer 名称条目 {}：{err}", index + 1),
            )
        })?;
        any_name |= result.entry != ObjectEntry10_1::Null;
        prefix.names.push(result);
        *body = trial;
        *refs = staged;
    }
    if !any_name {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterLayer 名称列表全为空对象",
        ));
    }
    Ok(())
}

/// 只接受已由 ExtraSafe 界定的图层体。未知名称不读取体后的流段。
pub(super) fn read_bounded_raster_body_10_1(
    body: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    let mut prefix = read_raster_layer_prefix_10_1(body, max_bytes)?;
    let Ok(mut continuation) = read_continuation_fields(body, prefix.version, max_bytes) else {
        return Ok(prefix);
    };
    let count = continuation.object_count.unwrap_or(1);
    if count > 0 {
        continuation.first_object_header = read_header_10_1(&mut body.clone()).ok();
    }
    prefix.continuation = Some(continuation);
    if let Err(error) = read_raster_layer_names(body, max_bytes, refs, &mut prefix, count) {
        prefix.names_stop = Some(error.to_string());
        return Ok(prefix);
    }
    match read_raster_renderer_slot_prefix_10_1(body, max_bytes, refs) {
        Ok(renderer) => prefix.renderer = Some(Box::new(renderer)),
        Err(error) => prefix.renderer_stop = Some(error.to_string()),
    }
    if prefix
        .renderer
        .as_ref()
        .is_some_and(|slot| slot.body.as_ref().is_none_or(|body| body.is_complete()))
    {
        match super::raster_layer_tail::read_raster_layer_tail_10_1(
            body,
            prefix.version,
            max_bytes,
            refs,
        ) {
            Ok(tail) => prefix.tail = Some(Box::new(tail)),
            Err(error) => prefix.tail_stop = Some(error.to_string()),
        }
    }
    Ok(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_string(text: &str) -> Vec<u8> {
        let payload: Vec<_> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
        bytes.extend(payload);
        bytes
    }

    #[test]
    fn reads_confirmed_raster_layer_prefix_and_stops_before_next_field() {
        let mut bytes = 17_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("Raster title"));
        let prefix_len = bytes.len();
        bytes.extend(encoded_string("unparsed second string"));

        let mut reader = Reader::new(&bytes);
        let parsed = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.version, 17);
        assert_eq!(parsed.first_string, "Raster title");
        assert_eq!(reader.position(), prefix_len);
    }

    #[test]
    fn accepts_negative_version_bits_as_native_signed_gate_does() {
        let mut bytes = (-1_i32).to_le_bytes().to_vec();
        bytes.extend(encoded_string("legacy"));
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap();

        assert_eq!(parsed.version, -1);
        assert_eq!(parsed.first_string, "legacy");
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn rejects_version_above_native_limit_without_advancing() {
        let bytes = 18_i32.to_le_bytes();
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_prefix_10_1(&mut reader, 0x1000).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn every_truncated_prefix_rolls_back() {
        let mut bytes = 4_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("landsat"));

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(
                read_raster_layer_prefix_10_1(&mut reader, 0x1000).is_err(),
                "prefix length {end} must be rejected"
            );
            assert_eq!(reader.position(), 0, "prefix length {end} must roll back");
        }
    }

    #[test]
    fn oversized_first_string_is_rejected_without_advancing() {
        let mut bytes = 1_i32.to_le_bytes().to_vec();
        bytes.extend(encoded_string("too long"));
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_prefix_10_1(&mut reader, 4).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn reads_second_string_fixed_shorts_count_and_only_first_object_header() {
        let mut bytes = encoded_string("unbound second string");
        bytes.extend([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
        bytes.extend(1_i32.to_le_bytes());
        let header_start = bytes.len();
        bytes.extend([0x7a; 16]);
        bytes.extend(0x1234_u32.to_le_bytes());
        bytes.extend([0xaa, 0xbb]);

        let mut reader = Reader::new(&bytes);
        let parsed = read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).unwrap();

        assert_eq!(parsed.second_string, "unbound second string");
        assert_eq!(parsed.fixed_shorts, [0x2211, 0x4433, 0x6655]);
        assert_eq!(parsed.object_count, Some(1));
        assert_eq!(
            parsed.first_object_header,
            Some(ObjectHeader::Object {
                class_id: [0x7a; 16],
                key: 0x1234,
            })
        );
        assert_eq!(reader.position(), header_start + 20);
        assert_eq!(reader.read_bytes(2).unwrap(), &[0xaa, 0xbb]);
    }

    #[test]
    fn pre_v9_continuation_has_no_stream_count_before_first_object() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend([0x5b; 16]);
        bytes.extend(9_u32.to_le_bytes());
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_continuation_10_1(&mut reader, 8, 0x1000).unwrap();

        assert_eq!(parsed.object_count, None);
        assert_eq!(
            parsed.first_object_header,
            Some(ObjectHeader::Object {
                class_id: [0x5b; 16],
                key: 9,
            })
        );
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn zero_count_stops_before_any_nested_object() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0x5b; 20]);
        let mut reader = Reader::new(&bytes);

        let parsed = read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).unwrap();

        assert_eq!(parsed.object_count, Some(0));
        assert_eq!(parsed.first_object_header, None);
        assert_eq!(reader.position(), bytes.len() - 20);
    }

    #[test]
    fn continuation_rejects_version_above_native_limit_without_advancing() {
        let bytes = encoded_string("second");
        let mut reader = Reader::new(&bytes);

        let error = read_raster_layer_continuation_10_1(&mut reader, 18, 0x1000).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn truncated_continuation_rolls_back_at_every_boundary() {
        let mut bytes = encoded_string("second");
        bytes.extend([0; 6]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0x5b; 20]);

        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_raster_layer_continuation_10_1(&mut reader, 17, 0x1000).is_err());
            assert_eq!(reader.position(), 0, "截断位置 {end} 应回滚");
        }
    }

    fn body_prefix(version: i32, count: i32) -> Vec<u8> {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded_string("栅格图层"));
        bytes.extend(encoded_string(""));
        bytes.extend([0; 6]);
        if version >= 9 {
            bytes.extend(count.to_le_bytes());
        }
        bytes
    }

    fn dataset_name(bytes: &mut Vec<u8>, key: u32) {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(key.to_le_bytes());
        bytes.extend(4_i32.to_le_bytes());
        for text in ["data", "", "", ""] {
            bytes.extend(encoded_string(text));
        }
        bytes.extend([0; 16]); // 工作空间。
        bytes.extend([0; 8]); // v4 两个 int32。
    }

    #[test]
    fn bounded_names_read_known_objects_backreferences_and_nulls_before_renderer() {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID;
        use super::super::raster_renderer::{
            RasterRendererPrefix10_1, SIMPLE_RASTER_RENDERER_CLASS_ID,
        };

        let mut bytes = body_prefix(17, 3);
        dataset_name(&mut bytes, 60);
        bytes.extend(encoded_string("第一项"));
        bytes.extend([0; 16]);
        bytes.extend(encoded_string("空项后缀"));
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(encoded_string("回引后缀"));
        bytes.extend(SIMPLE_RASTER_RENDERER_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes()); // SimpleRasterRenderer v1。
        bytes.extend(1_i32.to_le_bytes()); // 共用体版本。
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0xa1);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(result.names_stop.is_none());
        assert_eq!(result.names.len(), 3);
        assert!(matches!(
            result.names[0].body,
            Some(RasterDatasetNameEntryBody10_1::Dataset(_))
        ));
        assert_eq!(result.names[1].entry, ObjectEntry10_1::Null);
        assert_eq!(result.names[1].suffix.as_deref(), Some("空项后缀"));
        assert_eq!(
            result.names[2].entry,
            ObjectEntry10_1::BackReference {
                class_id: RASTER_DATASET_NAME_CLASS_ID,
                key: 60
            }
        );
        assert_eq!(result.names[2].suffix.as_deref(), Some("回引后缀"));
        assert!(matches!(
            result.renderer.unwrap().body,
            Some(RasterRendererPrefix10_1::Simple { .. })
        ));
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xa1]);
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn bounded_older_names_consume_versioned_legacy_fields_and_suffix() {
        for (version, selector) in [(0, 0_i32), (4, 0), (5, 0), (8, -1), (9, 0)] {
            let mut bytes = body_prefix(version, 1);
            dataset_name(&mut bytes, 60);
            if version > 0 {
                bytes.extend(encoded_string(""));
            }
            if (5..=8).contains(&version) {
                bytes.extend(selector.to_le_bytes());
                if selector != 0 {
                    bytes.extend(91_i32.to_le_bytes());
                }
            }
            bytes.extend([0; 16]); // 空渲染器，无键。
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.names_stop.is_none());
            assert_eq!(
                result.names[0].suffix.as_deref(),
                (version > 0).then_some("")
            );
            assert_eq!(
                result.names[0].legacy,
                (5..=8)
                    .contains(&version)
                    .then_some((selector, (selector != 0).then_some(91)))
            );
            assert_eq!(result.renderer.unwrap().entry, ObjectEntry10_1::Null);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn failed_name_slot_preserves_earlier_keys_and_does_not_reach_renderer() {
        for unknown in [false, true] {
            let mut bytes = body_prefix(17, 2);
            dataset_name(&mut bytes, 60);
            bytes.extend(encoded_string("可读项"));
            let stop = bytes.len();
            if unknown {
                bytes.extend([0x9c; 16]);
                bytes.extend(62_u32.to_le_bytes());
                bytes.extend(encoded_string(""));
                bytes.extend([0; 16]); // 不能越过未知体去读渲染器。
            } else {
                dataset_name(&mut bytes, 62);
                bytes.extend(42_u32.to_le_bytes()); // 后缀串截断，连同该名称键回退。
            }
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(result.names.len(), 1);
            assert!(result.names_stop.is_some());
            assert!(result.renderer.is_none());
            assert_eq!(reader.position(), stop);
            assert!(refs.contains(60) && !refs.contains(62));
        }
    }

    #[test]
    fn zero_negative_and_all_null_names_do_not_reach_renderer() {
        for count in [0, -1, 1, i32::MAX] {
            let mut bytes = body_prefix(17, count);
            if count == 1 {
                bytes.extend([0; 16]);
                bytes.extend(encoded_string(""));
            }
            let end = bytes.len();
            bytes.extend([0; 16]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.names_stop.is_some());
            assert!(result.renderer.is_none() && refs.is_empty());
            assert_eq!(reader.position(), end);
        }
    }
}
