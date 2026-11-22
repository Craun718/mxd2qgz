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
    /// 成功读取第二条名称、标量及名称列表前导后填入。
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
    /// 已确认的路径、数据集名和波段名。
    pub source_reference: Option<crate::storage::object_refs::DatasetSource10_1>,
    pub suffix: Option<String>,
    /// v5–v8 的 int32 选择子，非零时带原生波段集合的索引。
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
            let source_reference = match &entry {
                ObjectEntry10_1::NewObject { class_id, key }
                | ObjectEntry10_1::BackReference { class_id, key } => {
                    staged.dataset_source(*class_id, *key).cloned()
                }
                _ => None,
            };
            Ok(RasterLayerName10_1 {
                entry,
                body: name,
                source_reference,
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

/// 读取普通 RasterLayer 对象体。子体未完整读取时，不提交对象键或游标。
pub(super) fn read_inline_raster_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    use super::raster_layer_tail::{RasterLayerBoundary, read_raster_layer_tail};

    let mut trial = reader.clone();
    let mut staged = table.clone();
    let mut body = read_raster_layer_prefix_10_1(&mut trial, max_bytes)?;
    if body.version == 12 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "RasterLayer v12 的可选变换和包络槽需要对象体边界",
        ));
    }
    let count = read_name_list_prefix(&mut trial, max_bytes, &mut body)?;
    read_raster_layer_names(&mut trial, max_bytes, &mut staged, &mut body, count)?;

    let renderer = read_raster_renderer_slot_prefix_10_1(&mut trial, max_bytes, &mut staged)?;
    if renderer
        .body
        .as_ref()
        .is_some_and(|body| !body.is_complete())
    {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "RasterLayer 渲染器体未完整读取",
        ));
    }
    body.renderer = Some(Box::new(renderer));
    let tail = read_raster_layer_tail(
        &mut trial,
        body.version,
        max_bytes,
        &mut staged,
        RasterLayerBoundary::Inline,
    )?;
    require_loaded_extensions(&tail)?;
    body.tail = Some(Box::new(tail));
    *reader = trial;
    *table = staged;
    Ok(body)
}

fn require_loaded_extensions(tail: &RasterLayerTail10_1) -> io::Result<()> {
    // 有界扩展可保留原始字节。普通图层须确认内部对象键已经读取。
    if tail.legacy_extensions.iter().any(|entry| {
        !entry.raw_body.is_empty()
            && entry.properties.is_none()
            && entry.edit_template_manager.is_none()
            && entry.server_layer_extension.is_none()
            && entry.representation_layer_extension_version.is_none()
            && entry.route_layer_extension.is_none()
            && entry.hatch_layer_extension.is_none()
            && entry.tracking_filter_layer_extension.is_none()
            && entry.e2_layer_extension_values.is_none()
            && entry.feature_service_layer_edit_extension.is_none()
            && entry.globe_layer_properties.is_none()
            && entry.time_layer_track_extension.is_none()
            && entry.dynamic_tiled_layer_properties.is_none()
            && entry.scene_layer_properties.is_none()
    }) || tail.extra_safe_extensions.iter().any(|entry| {
        matches!(entry.entry, ObjectEntry10_1::NewObject { .. })
            && entry.properties.is_none()
            && entry.edit_template_manager.is_none()
            && entry.server_layer_extension.is_none()
            && entry.representation_layer_extension_version.is_none()
            && entry.route_layer_extension.is_none()
            && entry.hatch_layer_extension.is_none()
            && entry.tracking_filter_layer_extension.is_none()
            && entry.e2_layer_extension_values.is_none()
            && entry.feature_service_layer_edit_extension.is_none()
            && entry.globe_layer_properties.is_none()
            && entry.time_layer_track_extension.is_none()
            && entry.dynamic_tiled_layer_properties.is_none()
            && entry.scene_layer_properties.is_none()
    }) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "RasterLayer 含有未完整读取的扩展对象",
        ));
    }
    Ok(())
}

/// 已界定的子层仍须完整消费，普通父对象才能依赖其引用表。
pub(super) fn read_complete_bounded_raster_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let body = read_bounded_raster_body_10_1(&mut trial, max_bytes, &mut staged)?;
    let tail = body
        .tail
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::Unsupported, "有界 RasterLayer 子层未完整读取"))?;
    require_loaded_extensions(tail)?;
    *reader = trial;
    *table = staged;
    Ok(body)
}

fn read_name_list_prefix(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    body: &mut RasterLayerPrefix10_1,
) -> io::Result<i32> {
    let mut continuation = read_continuation_fields(reader, body.version, max_bytes)?;
    let count = continuation.object_count.unwrap_or(1);
    if count > 0 {
        continuation.first_object_header = read_header_10_1(&mut reader.clone()).ok();
    }
    body.continuation = Some(continuation);
    Ok(count)
}

/// 只接受已由 ExtraSafe 界定的图层体。未知名称不读取体后的流段。
pub(super) fn read_bounded_raster_body_10_1(
    body: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<RasterLayerPrefix10_1> {
    let mut prefix = read_raster_layer_prefix_10_1(body, max_bytes)?;
    let Ok(count) = read_name_list_prefix(body, max_bytes, &mut prefix) else {
        return Ok(prefix);
    };
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

    fn raster_with_empty_slots(version: i32, transform_pair: bool) -> Vec<u8> {
        let mut bytes = body_prefix(version, 1);
        dataset_name(&mut bytes, 60);
        if version > 0 {
            bytes.extend(encoded_string(""));
        }
        if (5..=8).contains(&version) {
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes.extend([0; 16]); // 空渲染器。
        bytes.extend([0; 22]); // 两个 double、int32、short。
        for (first, size) in [(2, 4), (3, 4), (4, 8), (5, 32), (6, 8), (7, 16)] {
            if version >= first {
                bytes.resize(bytes.len() + size, 0);
            }
        }
        if version >= 10 {
            bytes.extend(encoded_string("尾段"));
        }
        if version == 10 {
            bytes.extend(10_u16.to_le_bytes());
        }
        if version >= 11 {
            bytes.extend([0; 16]);
        }
        if version >= 13 || (version == 12 && transform_pair) {
            bytes.extend([0; 32]);
        }
        if version >= 14 {
            bytes.extend([0; 16 + 4]);
        }
        if version >= 15 {
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend([0; 20]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 26]);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded_string(""));
            bytes.extend(encoded_string(""));
        }
        if version >= 16 {
            bytes.extend([0; 16]);
        }
        if version >= 17 {
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn layer_list_loads_wcs_wrappers_and_keeps_nested_raster_keys() {
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_layer_list_10_1,
        };
        use crate::layers::wcs_layer::WCS_LAYER_CLASS_ID;
        use crate::objects::property_set::{PropertyValue10_1, PropertyValueObjectStream10_1};

        for (version, raster, flag) in [(15_i32, 0, 1_u16), (16, 12, 2), (17, 17, 3)] {
            let mut body = version.to_le_bytes().to_vec();
            for value in ["覆盖", "地址", "标识"] {
                body.extend(encoded_string(value));
            }
            body.extend(14_i32.to_le_bytes());
            body.extend(15_i32.to_le_bytes());
            body.extend(encoded_string("额外字段"));
            if version >= 16 {
                body.extend(encoded_string("选项"));
                body.extend([0; 16]); // Variant::Write(false) 的空标量。
            }
            if version >= 17 {
                body.extend(encoded_string("扩展字符串"));
            }
            body.extend(RASTER_LAYER_CLASS_ID);
            body.extend(52_u32.to_le_bytes());
            body.extend(raster_with_empty_slots(raster, raster == 12));

            if raster == 12 {
                let mut unbounded = body.clone();
                unbounded.extend([0xa5; 16]); // 外层流还有后继字段。
                let mut reader = Reader::new(&unbounded);
                let mut refs = RefTable10_1::new();
                refs.register(5);
                let before = refs.clone();
                let error = crate::layers::wcs_layer::read_wcs_layer_body_10_1(
                    &mut reader,
                    4096,
                    &mut refs,
                    false,
                )
                .unwrap_err();
                assert_eq!(error.kind(), io::ErrorKind::Unsupported);
                assert_eq!(reader.position(), 0);
                assert_eq!(refs, before);
            }

            let mut truncated = Reader::new(&body[..body.len() - 1]);
            let mut refs = RefTable10_1::new();
            refs.register(5);
            assert!(
                crate::layers::wcs_layer::read_wcs_layer_body_10_1(
                    &mut truncated,
                    4096,
                    &mut refs,
                    true,
                )
                .is_err()
            );
            assert_eq!(truncated.position(), 0);
            assert!(refs.contains(5));
            assert_eq!(refs.len(), 1);

            let mut bytes = 2_i32.to_le_bytes().to_vec();
            bytes.extend(flag.to_le_bytes());
            if flag != 1 {
                bytes.extend((20 + body.len() as u32).to_le_bytes());
                bytes.extend(0_u32.to_le_bytes());
            }
            bytes.extend(WCS_LAYER_CLASS_ID);
            bytes.extend(51_u32.to_le_bytes());
            bytes.extend(body);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(RASTER_LAYER_CLASS_ID);
            bytes.extend(52_u32.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.push(0x7f);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(LayerBodyPrefix10_1::WcsLayer(layer)) = &parsed.entries[0].parsed else {
                panic!("{:?}", parsed.entries[0].parse_note);
            };
            assert_eq!(layer.version, version);
            assert_eq!(layer.v14_longs, Some([14, 15]));
            assert_eq!(layer.v15_string.as_deref(), Some("额外字段"));
            if version >= 16 {
                assert!(matches!(
                    &layer.v16_fields,
                    Some((value, PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Empty)))
                        if value == "选项"
                ));
            } else {
                assert!(layer.v16_fields.is_none());
            }
            assert_eq!(
                layer.v17_string.as_deref(),
                (version == 17).then_some("扩展字符串")
            );
            assert_eq!(layer.raster.body.as_ref().unwrap().version, raster);
            assert!(matches!(
                parsed.entries[1].object,
                crate::layers::layer_list::LayerEntryObject::Entry(
                    ObjectEntry10_1::BackReference { key: 52, .. }
                )
            ));
            assert_eq!(parsed.active_index, Some(1));
            assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
            assert!(refs.contains(51) && refs.contains(52) && refs.contains(60));
        }
    }

    #[test]
    fn ordinary_raster_self_delimiting_versions_leave_the_next_object_unread() {
        for version in 0..=17 {
            if version == 12 {
                continue;
            }
            let mut bytes = raster_with_empty_slots(version, false);
            let end = bytes.len();
            bytes.extend([0x53; 20]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(parsed.version, version);
            assert_eq!(
                parsed.tail.unwrap().geodata_xform_v13.is_some(),
                version >= 13
            );
            assert!(refs.contains(60));
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(20).unwrap(), [0x53; 20]);

            for cut in 0..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err(),
                    "版本 {version}，截断位置 {cut}"
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn raster_v12_uses_only_a_declared_boundary_to_choose_its_optional_slots() {
        for pair in [false, true] {
            let bytes = raster_with_empty_slots(12, pair);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let error = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::Unsupported);
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
            let parsed =
                read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(parsed.tail.unwrap().geodata_xform_v13.is_some(), pair);
            assert_eq!(reader.remaining(), 0);
            assert!(refs.contains(60));
        }
    }

    #[test]
    fn legacy_map_extra_safe_passes_boundary_to_raster_v12_and_wcs() {
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, LayerEntryObject, RASTER_LAYER_CLASS_ID,
            read_legacy_layer_lists_10_1,
        };
        use crate::layers::wcs_layer::WCS_LAYER_CLASS_ID;

        for wrapper in [false, true] {
            for (pair, truncated) in [(false, false), (true, false), (true, true)] {
                let mut object = if wrapper {
                    let mut object = WCS_LAYER_CLASS_ID.to_vec();
                    object.extend(51_u32.to_le_bytes());
                    object.extend(1_i32.to_le_bytes());
                    for text in ["覆盖", "地址", "标识"] {
                        object.extend(encoded_string(text));
                    }
                    object
                } else {
                    Vec::new()
                };
                object.extend(RASTER_LAYER_CLASS_ID);
                object.extend(52_u32.to_le_bytes());
                object.extend(raster_with_empty_slots(12, pair));
                if truncated {
                    object.pop();
                }
                let mut bytes = 1_i32.to_le_bytes().to_vec();
                let object_start = bytes.len();
                if wrapper {
                    bytes.extend(0_u32.to_le_bytes());
                    bytes.extend((object.len() as u32).to_le_bytes());
                } else {
                    bytes.extend((object.len() as u32).to_le_bytes());
                    bytes.extend(0_u32.to_le_bytes());
                }
                bytes.extend(object);
                bytes.extend(1_i32.to_le_bytes()); // 第二组复用第一组的栅格层。
                bytes.extend(RASTER_LAYER_CLASS_ID);
                bytes.extend(52_u32.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
                let end = bytes.len();
                bytes.push(0x53);

                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                refs.register(5);
                let list = read_legacy_layer_lists_10_1(&mut reader, 4096, &mut refs, 19).unwrap();
                if truncated {
                    assert!(list.stop.is_some());
                    assert_eq!(reader.position(), object_start);
                    assert_eq!(refs.len(), 1);
                    assert!(refs.contains(5));
                    continue;
                }
                let raster = match list.entries[0].parsed.as_ref() {
                    Some(LayerBodyPrefix10_1::RasterLayer(raster)) => raster,
                    Some(LayerBodyPrefix10_1::WcsLayer(wcs)) => wcs.raster.body.as_ref().unwrap(),
                    unexpected => panic!("预期完整有界栅格，读到 {unexpected:?}"),
                };
                let tail = raster.tail.as_ref().unwrap();
                assert_eq!(raster.version, 12);
                assert_eq!(tail.geodata_xform_v13.is_some(), pair);
                assert!(matches!(
                    list.entries[1].object,
                    LayerEntryObject::Entry(ObjectEntry10_1::BackReference { key: 52, .. })
                ));
                assert!(list.stop.is_none());
                assert_eq!(list.active_index, Some(0));
                assert!(refs.contains(52) && refs.contains(60));
                assert_eq!(reader.position(), end);
                assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);
            }
        }
    }

    #[test]
    fn bounded_raster_distinguishes_projective_third_null_slot_at_the_layer_end() {
        use crate::objects::envelope::ENVELOPE_CLASS_ID;
        use crate::objects::geodata_xform::{GeodataXformBody10_1, PROJECTIVE_XFORM_CLASS_ID};
        use crate::objects::geometry::MULTIPOINT_CLASS_ID;

        let transform_offset = raster_with_empty_slots(12, false).len();
        for (version, legacy, geometry, point_count) in [
            (12, false, false, 0_i32),
            (12, true, false, 0),
            (12, true, true, 0),
            (13, true, true, 0),
            (13, true, true, -1),
            (14, true, true, 0),
            (17, true, true, 1),
        ] {
            let mut bytes = raster_with_empty_slots(version, true);
            let mut pair = PROJECTIVE_XFORM_CLASS_ID.to_vec();
            pair.extend(71_u32.to_le_bytes());
            pair.extend(1_i32.to_le_bytes());
            pair.extend([0; 16]); // 空空间参考。
            if legacy {
                if geometry {
                    pair.extend(MULTIPOINT_CLASS_ID);
                    pair.extend(72_u32.to_le_bytes());
                    pair.extend(2_u16.to_le_bytes());
                    pair.extend(4_u32.to_le_bytes());
                    pair.extend(0_u32.to_le_bytes());
                    pair.extend([0; 16]);
                } else {
                    pair.extend([0; 16]); // 原生入口实验中的空几何集合。
                }
            }
            pair.extend([0; 16]); // 空投影变换。
            let points = (point_count > 0).then_some([[1.0_f64, 2.0], [3.0, 4.0]]);
            let points_flag = i32::from(points.is_some());
            pair.extend(point_count.to_le_bytes());
            pair.extend(points_flag.to_le_bytes());
            if let Some(points) = points {
                for coordinate in points.into_iter().flatten() {
                    pair.extend(coordinate.to_le_bytes());
                }
            }
            let point_end = transform_offset + pair.len();
            pair.extend(ENVELOPE_CLASS_ID);
            pair.extend(73_u32.to_le_bytes());
            pair.extend(1_u16.to_le_bytes());
            pair.extend([0; 32]);
            let missing_byte = transform_offset + pair.len() - 1;
            bytes.splice(transform_offset..transform_offset + 32, pair);
            if version == 14 {
                // 保留空颜色槽，追加一个非空索引序列。
                let count = bytes.len() - 4;
                bytes[count..].copy_from_slice(&2_i32.to_le_bytes());
                bytes.extend(4_i32.to_le_bytes());
                bytes.extend(7_i32.to_le_bytes());
            }

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let layer =
                read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(reader.remaining(), 0);
            let tail = layer.tail.unwrap();
            let transform = tail.geodata_xform_v13.unwrap();
            let Some(GeodataXformBody10_1::Projective(projective)) = transform.body.as_deref()
            else {
                panic!("应读取 ProjectiveXform");
            };
            assert_eq!(
                projective.transformation,
                legacy.then_some(ObjectEntry10_1::Null)
            );
            assert_eq!(projective.geometry.body.is_some(), geometry);
            assert_eq!(projective.point_count, point_count);
            assert_eq!(projective.points_flag, points_flag);
            assert_eq!(
                projective.point_arrays,
                points.map(|points| points.map(|point| vec![point]))
            );
            assert!(tail.envelope_v13.is_some());
            assert_eq!(
                tail.color_indices_v14,
                if version == 14 { &[4, 7][..] } else { &[] }
            );
            assert!(refs.contains(60) && refs.contains(71) && refs.contains(73));
            assert_eq!(refs.contains(72), geometry);

            bytes.push(0xa5);
            for end in [point_end - 1, missing_byte, bytes.len()] {
                let mut reader = Reader::new(&bytes[..end]);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_complete_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn ordinary_group_can_use_its_extra_safe_raster_child_boundary() {
        use crate::layers::group_layer::{GROUP_LAYER_CLASS_ID, GroupChildEntry10_1};
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_complete_layer_body_10_1,
        };

        for pair in [false, true] {
            let raster = raster_with_empty_slots(12, pair);
            let mut bytes = 11_u16.to_le_bytes().to_vec();
            bytes.extend(encoded_string("组"));
            bytes.extend([0; 6 + 16]);
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend((20 + raster.len() as u32).to_le_bytes());
            bytes.extend(0_u32.to_le_bytes());
            bytes.extend(RASTER_LAYER_CLASS_ID);
            bytes.extend(55_u32.to_le_bytes());
            bytes.extend(raster);
            bytes.extend([0; 2 + 16 + 8 + 4]);
            bytes.extend(encoded_string(""));
            bytes.extend([0; 2 + 16 + 2 + 6]);
            let end = bytes.len();
            bytes.extend([0x55; 16]);

            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes);
            let LayerBodyPrefix10_1::GroupLayer(group) =
                read_complete_layer_body_10_1(&mut reader, GROUP_LAYER_CLASS_ID, 4096, &mut refs)
                    .unwrap()
            else {
                panic!("应读取组图层");
            };
            let GroupChildEntry10_1::SkippedNewObject {
                parsed: Some(raster),
                body_length: 0,
                ..
            } = &group.children[0]
            else {
                panic!("ExtraSafe 内的栅格子体应完整消费");
            };
            assert!(
                matches!(raster.as_ref(), LayerBodyPrefix10_1::RasterLayer(body)
                if body.tail.as_ref().unwrap().geodata_xform_v13.is_some() == pair)
            );
            assert!(refs.contains(55) && refs.contains(60));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn ordinary_raster_reads_globe_properties_and_later_back_reference() {
        use crate::layers::globe_layer_properties::GLOBE_LAYER_PROPERTIES_CLASS_ID;

        let mut bytes = raster_with_empty_slots(8, false);
        let extension_count = bytes.len() - (4 + 8 + 32 + 8 + 16);
        let mut globe = 1_u16.to_le_bytes().to_vec();
        globe.extend(0_u32.to_le_bytes());
        globe.extend(encoded_string(""));
        globe.extend([0; 16 + 2 + 4]);
        let mut extensions = 2_u32.to_le_bytes().to_vec();
        extensions.extend((20 + globe.len() as u32).to_le_bytes());
        extensions.extend(0_u32.to_le_bytes());
        extensions.extend(GLOBE_LAYER_PROPERTIES_CLASS_ID);
        extensions.extend(61_u32.to_le_bytes());
        extensions.extend(globe);
        extensions.extend(20_u32.to_le_bytes());
        extensions.extend(0_u32.to_le_bytes());
        extensions.extend(GLOBE_LAYER_PROPERTIES_CLASS_ID);
        extensions.extend(61_u32.to_le_bytes());
        bytes.splice(extension_count..extension_count + 4, extensions);
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let tail = raster.tail.unwrap();
        assert!(
            tail.extra_safe_extensions[0]
                .globe_layer_properties
                .is_some()
        );
        assert!(matches!(
            tail.extra_safe_extensions[1].entry,
            ObjectEntry10_1::BackReference { key: 61, .. }
        ));
        assert!(refs.contains(61));
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn ordinary_raster_reads_scene_properties_in_both_extension_encodings() {
        use crate::layers::scene_layer_properties::{
            RASTER_3D_PROPERTIES_CLASS_ID, SurfaceName10_1,
        };
        use crate::objects::envelope::ENVELOPE_CLASS_ID;

        let mut properties = 13_u16.to_le_bytes().to_vec();
        properties.extend(1_u32.to_le_bytes());
        // RasterLayer 的首个数据源名称已经登记过。
        properties.extend(crate::layers::raster_dataset_name::RASTER_DATASET_NAME_CLASS_ID);
        properties.extend(60_u32.to_le_bytes());
        properties.extend(encoded_string("地表路径"));
        properties.extend([0; 20]);
        properties.extend(encoded_string("偏移"));
        properties.extend([0; 4]);
        properties.extend(encoded_string("拉伸"));
        properties.extend([0; 106]);
        properties.extend(ENVELOPE_CLASS_ID);
        properties.extend(71_u32.to_le_bytes());
        properties.extend(1_u16.to_le_bytes());
        properties.extend([0; 32]);
        for version in [3, 8] {
            let mut bytes = raster_with_empty_slots(version, false);
            let extension_count = bytes.len() - if version == 3 { 4 } else { 68 };
            let mut encoded = 1_u32.to_le_bytes().to_vec();
            let size = properties.len() as u32 + if version == 3 { 0 } else { 20 };
            encoded.extend(u64::from(size).to_le_bytes());
            encoded.extend(RASTER_3D_PROPERTIES_CLASS_ID);
            if version == 8 {
                encoded.extend(70_u32.to_le_bytes());
            }
            encoded.extend(&properties);
            bytes.splice(extension_count..extension_count + 4, encoded);
            let end = bytes.len();
            bytes.push(0xa5);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let raster = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let tail = raster.tail.unwrap();
            let properties = if version == 3 {
                &tail.legacy_extensions[0].scene_layer_properties
            } else {
                &tail.extra_safe_extensions[0].scene_layer_properties
            }
            .as_ref()
            .unwrap();
            assert!(matches!(
                properties.surface_name,
                Some(SurfaceName10_1::Raster(ref name))
                    if matches!(name.entry, ObjectEntry10_1::BackReference { key: 60, .. })
            ));
            assert!(refs.contains(71));
            assert_eq!(refs.contains(70), version == 8);
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn ordinary_raster_reads_direct_property_set_and_dynamic_extension_in_both_encodings() {
        use crate::layers::layer_extensions::DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID;
        use crate::objects::property_set::{PROPERTY_SET_CLASS_ID, PropertyValueObjectStream10_1};

        let mut direct = 1_u16.to_le_bytes().to_vec();
        direct.extend(1_u32.to_le_bytes());
        direct.extend(encoded_string("配置"));
        direct.extend(13_u16.to_le_bytes());
        direct.extend(PROPERTY_SET_CLASS_ID);
        direct.extend(72_u32.to_le_bytes());
        direct.extend(1_u16.to_le_bytes());
        direct.extend(0_u32.to_le_bytes());

        let mut dynamic = 1_u16.to_le_bytes().to_vec();
        dynamic.extend(encoded_string("瓦片源"));
        dynamic.extend(PROPERTY_SET_CLASS_ID);
        dynamic.extend(72_u32.to_le_bytes());
        dynamic.extend(encoded_string(""));
        dynamic.extend(encoded_string(""));
        dynamic.extend([0; 12]);

        for version in [3, 8] {
            let build = |direct: &[u8]| {
                let mut bytes = raster_with_empty_slots(version, false);
                let extension_count = bytes.len() - if version == 3 { 4 } else { 68 };
                let mut encoded = 2_u32.to_le_bytes().to_vec();
                for (class_id, key, body) in [
                    (PROPERTY_SET_CLASS_ID, 70_u32, direct),
                    (DYNAMIC_TILED_LAYER_PROPERTIES_CLASS_ID, 71, &dynamic),
                ] {
                    let size = body.len() as u32 + if version == 8 { 20 } else { 0 };
                    encoded.extend(u64::from(size).to_le_bytes());
                    encoded.extend(class_id);
                    if version == 8 {
                        encoded.extend(key.to_le_bytes());
                    }
                    encoded.extend(body);
                }
                bytes.splice(extension_count..extension_count + 4, encoded);
                (bytes, extension_count)
            };

            let (mut bytes, extension_count) = build(&direct);
            let end = bytes.len();
            bytes.push(0xa5);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(reader.position(), end);
            assert!(refs.contains(72));
            assert_eq!(refs.contains(70) && refs.contains(71), version == 8);
            let tail = result.tail.unwrap();
            let (properties, dynamic) = if version == 3 {
                let mut legacy = Reader::new(&bytes[extension_count..end]);
                let mut legacy_refs = RefTable10_1::new();
                let shared = crate::layers::layer_extensions::read_layer_extensions_10_1(
                    &mut legacy,
                    4096,
                    &mut legacy_refs,
                )
                .unwrap();
                assert_eq!(legacy.remaining(), 0);
                assert_eq!(shared[0].properties, tail.legacy_extensions[0].properties);
                assert!(legacy_refs.contains(72));
                (
                    tail.legacy_extensions[0].properties.as_deref().unwrap(),
                    tail.legacy_extensions[1]
                        .dynamic_tiled_layer_properties
                        .as_deref()
                        .unwrap(),
                )
            } else {
                (
                    tail.extra_safe_extensions[0].properties.as_deref().unwrap(),
                    tail.extra_safe_extensions[1]
                        .dynamic_tiled_layer_properties
                        .as_deref()
                        .unwrap(),
                )
            };
            assert!(matches!(
                properties.entries[0].value,
                PropertyValueObjectStream10_1::Object { key: 72, .. }
            ));
            assert!(matches!(
                dynamic.properties.entry,
                ObjectEntry10_1::BackReference { key: 72, .. }
            ));

            let mut excessive = direct.clone();
            excessive.push(0xa5);
            for invalid in [&direct[..direct.len() - 1], &excessive] {
                let (bytes, _) = build(invalid);
                let mut reader = Reader::new(&bytes);
                let mut refs = RefTable10_1::new();
                assert!(read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).is_err());
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn ordinary_raster_rejects_unknown_bounded_extension_without_leaking_keys() {
        let mut bytes = raster_with_empty_slots(8, false);
        let extension_count = bytes.len() - (4 + 8 + 32 + 8 + 16);
        let mut extension = 1_u32.to_le_bytes().to_vec();
        extension.extend(21_u32.to_le_bytes());
        extension.extend(0_u32.to_le_bytes());
        extension.extend([0x53; 16]);
        extension.extend(61_u32.to_le_bytes());
        extension.push(0xaa);
        bytes.splice(extension_count..extension_count + 4, extension);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error = read_inline_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
        assert!(
            read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs)
                .unwrap()
                .tail
                .is_some()
        );
    }

    #[test]
    fn raster_associated_layer_and_map_list_read_past_the_raster_body() {
        use crate::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
        use crate::layers::layer_list::{
            LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_layer_list_10_1,
        };

        let mut entry = RASTER_LAYER_CLASS_ID.to_vec();
        entry.extend(55_u32.to_le_bytes());
        entry.extend(raster_with_empty_slots(17, false));
        let mut scene = 1_u16.to_le_bytes().to_vec();
        scene.extend(encoded_string("Scene"));
        scene.extend([0; 8]);
        scene.extend(&entry);
        scene.extend([0; 16 + 4]);
        scene.push(0x55);
        let mut reader = Reader::new(&scene);
        let mut refs = RefTable10_1::new();
        let scene =
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .unwrap();
        assert!(
            matches!(scene.associated_layer_body.as_deref(), Some(LayerBodyPrefix10_1::RasterLayer(body))
            if body.version == 17 && body.tail.is_some())
        );
        assert_eq!(reader.read_bytes(1).unwrap(), [0x55]);
        assert!(refs.contains(55) && refs.contains(60));

        let mut list = 2_u32.to_le_bytes().to_vec();
        list.extend(1_u16.to_le_bytes());
        list.extend(entry);
        list.extend(1_u16.to_le_bytes());
        list.extend(RASTER_LAYER_CLASS_ID);
        list.extend(55_u32.to_le_bytes());
        list.extend(1_i32.to_le_bytes());
        let mut reader = Reader::new(&list);
        let mut refs = RefTable10_1::new();
        let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.active_index, Some(1));
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(55) && refs.contains(60));
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
        assert_eq!(
            result.names[2]
                .source_reference
                .as_ref()
                .map(|source| source.name.as_ref()),
            Some("data"),
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
    fn service_and_tiled_names_share_layer_keys_and_reach_renderer() {
        use super::super::raster_dataset_name::{
            AIS_IMAGE_SERVICE_NAME_CLASS_ID, INTERNET_TILED_LAYER_NAME_CLASS_ID,
        };
        use super::super::raster_renderer::{
            RasterRendererPrefix10_1, SIMPLE_RASTER_RENDERER_CLASS_ID,
        };
        use crate::layers::cache_tile_reader::CacheTileReaderBody10_1;

        let mut bytes = body_prefix(17, 4);
        bytes.extend(INTERNET_TILED_LAYER_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("瓦片"));
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend([
            0x16, 0xec, 0xb2, 0x48, 0xdb, 0xb0, 0x85, 0x4d, 0x9a, 0x86, 0x1d, 0xdb, 0x04, 0x04,
            0x85, 0x79,
        ]); // MapTileReader。
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string(""));
        bytes.extend(encoded_string(""));
        bytes.extend([0; 4]);
        bytes.extend(encoded_string("tiles"));
        bytes.extend([0; 16 + 2]); // 空栅格名称和选择子。
        bytes.extend(encoded_string("瓦片后缀"));

        bytes.extend(AIS_IMAGE_SERVICE_NAME_CLASS_ID);
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        for text in ["service", "server", ""] {
            bytes.extend(encoded_string(text));
        }
        bytes.extend(encoded_string("服务后缀"));
        bytes.extend(INTERNET_TILED_LAYER_NAME_CLASS_ID);
        bytes.extend(60_u32.to_le_bytes());
        bytes.extend(encoded_string("回引后缀"));
        bytes.extend(AIS_IMAGE_SERVICE_NAME_CLASS_ID);
        bytes.extend(62_u32.to_le_bytes());
        bytes.extend(encoded_string(""));
        bytes.extend(SIMPLE_RASTER_RENDERER_CLASS_ID);
        bytes.extend(63_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);

        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(result.names_stop, None);
        let Some(RasterDatasetNameEntryBody10_1::InternetTiledLayer(tiled)) = &result.names[0].body
        else {
            panic!("应读取 IName 瓦片名称");
        };
        assert_eq!((tiled.name_string.as_str(), tiled.integer), ("瓦片", 7));
        assert!(matches!(
            tiled.tile_reader.body.as_deref(),
            Some(CacheTileReaderBody10_1::MapTile { path, .. }) if path == "tiles"
        ));
        assert!(matches!(
            &result.names[1].body,
            Some(RasterDatasetNameEntryBody10_1::AisImageService(name)) if name.strings[0] == "service"
        ));
        assert_eq!(
            result.names[2].entry,
            ObjectEntry10_1::BackReference {
                class_id: INTERNET_TILED_LAYER_NAME_CLASS_ID,
                key: 60,
            }
        );
        for (definition, reference, expected) in [(0, 2, "瓦片"), (1, 3, "service")] {
            let source = result.names[definition].source_reference.as_ref().unwrap();
            assert_eq!(source.name.as_ref(), expected);
            assert!(source.path.is_none());
            assert!(result.names[reference].body.is_none());
            assert_eq!(
                result.names[reference].source_reference.as_ref(),
                Some(source)
            );
        }
        assert!(matches!(
            result.renderer.unwrap().body,
            Some(RasterRendererPrefix10_1::Simple { .. })
        ));
        assert!(refs.contains(61));
        assert_eq!(refs.len(), 4);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn bounded_sai_renderer_reaches_raster_layer_tail() {
        use super::super::raster_renderer::{RasterRendererPrefix10_1, SAI_RENDERER_CLASS_ID};

        let mut bytes = body_prefix(1, 1);
        dataset_name(&mut bytes, 60);
        bytes.extend(encoded_string(""));
        let renderer_start = bytes.len();
        bytes.extend(SAI_RENDERER_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(17_u16.to_le_bytes());
        bytes.extend(0.75_f64.to_le_bytes());
        bytes.extend(19.0_f64.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes()); // 共用体 v1。
        bytes.extend([0; 16]);
        let renderer_end = bytes.len();
        bytes.extend(2.5_f64.to_le_bytes()); // 图层尾段。
        bytes.extend([0; 14]);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        match result.renderer.unwrap().body.unwrap() {
            RasterRendererPrefix10_1::Sai {
                version,
                short,
                doubles,
                shared,
            } => {
                assert_eq!(
                    (version, short, doubles, shared.version),
                    (1, 17, [0.75, 19.0], 1)
                );
            }
            other => panic!("预期 SAIRenderer，实际为 {other:?}"),
        }
        assert_eq!(result.tail.unwrap().doubles_bc_c4, [2.5, 0.0]);
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(61));

        for cut in renderer_start..renderer_end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(result.renderer_stop.is_some(), "截断位置 {cut}");
            assert_eq!(reader.position(), renderer_start);
            assert!(refs.contains(60));
            assert!(!refs.contains(61));
        }
    }

    #[test]
    fn band_name_reads_nested_collection_and_rolls_back_object_keys() {
        use super::super::raster_dataset_name::{
            RASTER_BAND_COLLECTION_NAME_CLASS_ID, RASTER_BAND_NAME_CLASS_ID,
            RasterBandNameSource10_1, RasterDatasetWorkspaceSlot10_1,
        };
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = body_prefix(1, 1);
        let name_start = bytes.len();
        bytes.extend(RASTER_BAND_NAME_CLASS_ID);
        bytes.extend(71_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        for value in ["目录\\首波段", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(RASTER_BAND_COLLECTION_NAME_CLASS_ID);
        bytes.extend(72_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("波段集合"));
        bytes.extend(RASTER_BAND_NAME_CLASS_ID);
        bytes.extend(73_u32.to_le_bytes());
        for value in ["旧名称", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(74_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["栅格目录", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 21]); // 空属性集、空工厂和尾 int32。
        bytes.extend([
            0x08, 0xb8, 0xbf, 0x98, 0x1f, 0xe9, 0xd2, 0x11, 0x9f, 0x81, 0x00, 0xc0, 0x4f, 0x8e,
            0xce, 0x27,
        ]);
        bytes.extend(75_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        let array_count_offset = bytes.len();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend((-5_i32).to_le_bytes());
        bytes.extend(14_i32.to_le_bytes());
        bytes.extend(encoded_string("图层名称后缀"));
        let name_end = bytes.len();
        bytes.extend([0; 16 + 22]); // 空渲染器与 v1 尾段。

        let assert_name_rollback = |input: &[u8]| {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.names_stop.is_some());
            assert_eq!(reader.position(), name_start);
            assert!(refs.is_empty());
        };
        for cut in name_start..name_end {
            assert_name_rollback(&bytes[..cut]);
        }
        let mut malformed = bytes.clone();
        malformed[array_count_offset..array_count_offset + 4]
            .copy_from_slice(&i32::MAX.to_le_bytes());
        assert_name_rollback(&malformed);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.names[0].suffix.as_deref(), Some("图层名称后缀"));
        let Some(RasterDatasetNameEntryBody10_1::Band(band)) = &parsed.names[0].body else {
            panic!("预期 RasterBandName");
        };
        assert!(band.has_version_marker);
        assert_eq!(band.band_lookup_name(), "首波段");
        let RasterBandNameSource10_1::Dataset(dataset) = &band.source else {
            panic!("预期 IDatasetName");
        };
        let Some(RasterDatasetNameEntryBody10_1::BandCollection(collection)) = &dataset.body else {
            panic!("预期 RasterBandCollectionName");
        };
        assert_eq!(collection.array, Some((1, vec![-5, 14])));
        let Some(RasterDatasetNameEntryBody10_1::Band(legacy_band)) = &collection.name.body else {
            panic!("预期内嵌 RasterBandName");
        };
        assert!(!legacy_band.has_version_marker);
        assert_eq!(legacy_band.band_lookup_name(), "旧名称");
        let legacy_source = refs.dataset_source(RASTER_BAND_NAME_CLASS_ID, 73).unwrap();
        assert_eq!(legacy_source.path.as_deref(), Some("栅格目录"));
        assert_eq!(legacy_source.band_name.as_deref(), Some("旧名称"));
        assert!(legacy_source.name.is_empty());
        assert!(matches!(
            legacy_band.source,
            RasterBandNameSource10_1::Workspace(RasterDatasetWorkspaceSlot10_1::WorkspaceName(_))
        ));
        assert!(parsed.tail.is_some());
        assert_eq!(refs.len(), 5);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn band_collection_nesting_allows_back_reference_at_depth_limit() {
        use super::super::raster_dataset_name::RASTER_BAND_COLLECTION_NAME_CLASS_ID as CLASS_ID;

        for count in [16_u32, 17] {
            let mut bytes = body_prefix(1, 1);
            let name_start = bytes.len();
            for key in 71..71 + count {
                bytes.extend(CLASS_ID);
                bytes.extend(key.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(encoded_string(""));
            }
            bytes.extend(CLASS_ID);
            bytes.extend(71_u32.to_le_bytes());
            bytes.resize(bytes.len() + 16 * count as usize, 0); // 每层 ILongArray 为空。
            bytes.extend(encoded_string(""));
            bytes.extend([0; 16 + 22]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            if count == 16 {
                assert!(parsed.tail.is_some());
                assert_eq!(refs.len(), count as usize);
                assert_eq!(reader.remaining(), 0);
            } else {
                assert!(parsed.names_stop.unwrap().contains("名称嵌套超过限额"));
                assert!(refs.is_empty());
                assert_eq!(reader.position(), name_start);
            }
        }
    }

    #[test]
    fn image_service_name_reads_name_and_transform_with_shared_keys() {
        use super::super::image_server_layer_tail::IMAGE_SERVER_NAME_CLASS_ID;
        use super::super::raster_dataset_name::IS_RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID;
        use crate::objects::property_set::PROPERTY_SET_CLASS_ID;

        for version in 1_u16..=4 {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            bytes.extend(IS_RASTER_DATASET_NAME_CLASS_ID);
            bytes.extend(72_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for text in ["服务栅格", "", "", "", "", ""] {
                bytes.extend(encoded_string(text));
            }
            bytes.extend(7_i32.to_le_bytes());
            bytes.extend([0; 16]); // 空工作空间。
            bytes.extend(IMAGE_SERVER_NAME_CLASS_ID);
            bytes.extend(73_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(encoded_string("影像服务"));
            bytes.extend(PROPERTY_SET_CLASS_ID);
            bytes.extend(74_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(0_i32.to_le_bytes());
            if version >= 2 {
                bytes.extend(encoded_string("版本 2"));
            }
            if version >= 3 {
                bytes.extend(encoded_string("版本 3"));
            }
            if version >= 4 {
                bytes.extend(IDENTITY_XFORM_CLASS_ID);
                bytes.extend(75_u32.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                bytes.extend([0; 16]);
            }
            bytes.extend(encoded_string("图层后缀"));
            let end = bytes.len();
            bytes.extend([0; 16 + 22]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::ImageService(name)) = &result.names[0].body
            else {
                panic!("预期 ISRasterDatasetName");
            };
            let Some(RasterDatasetNameEntryBody10_1::ImageServerReference(server)) =
                &name.name.body
            else {
                panic!("预期 ImageServerName");
            };
            assert_eq!(server.text, "影像服务");
            assert!(server.new_properties.is_some());
            assert_eq!(name.long_44, 7);
            assert_eq!(
                name.string_50.as_deref(),
                (version >= 2).then_some("版本 2")
            );
            assert_eq!(
                name.string_54.as_deref(),
                (version >= 3).then_some("版本 3")
            );
            assert_eq!(name.transform.is_some(), version == 4);
            assert!((72..=74).all(|key| refs.contains(key)));
            assert_eq!(refs.contains(75), version == 4);
            assert!(result.tail.is_some());
            assert_eq!(reader.remaining(), 0);
            for cut in start..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(result.names_stop.is_some());
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn raster_name_and_transform_share_the_recursion_limit() {
        use super::super::raster_dataset_name::IS_RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::geodata_xform::RPC_XFORM_CLASS_ID;

        for count in [2_u32, 20] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            for level in 0..count {
                bytes.extend(IS_RASTER_DATASET_NAME_CLASS_ID);
                bytes.extend((100 + level * 2).to_le_bytes());
                bytes.extend(4_u16.to_le_bytes());
                for _ in 0..6 {
                    bytes.extend(encoded_string(""));
                }
                bytes.extend([0; 4 + 16 + 16]); // int32、工作空间、IName。
                bytes.extend(encoded_string(""));
                bytes.extend(encoded_string(""));
                if level + 1 == count {
                    bytes.extend([0; 16]); // 末级空变换。
                } else {
                    bytes.extend(RPC_XFORM_CLASS_ID);
                    bytes.extend((101 + level * 2).to_le_bytes());
                    bytes.extend(2_i32.to_le_bytes());
                    bytes.extend([0; 0x2d0 + 16 + 24 + 4]); // RPC 参数、空内层变换、标量。
                }
            }
            bytes.resize(bytes.len() + (count as usize - 1) * (4 + 16 + 16), 0);
            bytes.extend(encoded_string(""));
            bytes.extend([0; 16 + 22]);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            if count == 2 {
                assert!(result.tail.is_some());
                assert_eq!(refs.len(), 3);
                assert_eq!(reader.remaining(), 0);
            } else {
                assert!(result.names_stop.unwrap().contains("嵌套"));
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn function_raster_name_reads_legacy_single_and_modern_counted_inputs() {
        use super::super::image_server_rules::RasterTemplateInput10_1;
        use super::super::image_server_rules::{
            CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID, IDENTITY_FUNCTION_CLASS_ID,
        };
        use super::super::raster_dataset_name::{
            FUNCTION_RASTER_DATASET_NAME_CLASS_ID, FunctionRasterInput10_1,
        };
        use crate::objects::property_set::{
            PROPERTY_SET_CLASS_ID, PropertyValueObjectStream10_1, RASTER_SCALAR_CLASS_ID,
        };

        for version in [1_u16, 4, 7] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            bytes.extend(FUNCTION_RASTER_DATASET_NAME_CLASS_ID);
            bytes.extend(80_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for text in ["函数栅格", "", ""] {
                bytes.extend(encoded_string(text));
            }
            bytes.extend([0; 16]); // 空工作空间名称。
            if version < 4 {
                bytes.extend(encoded_string("旧版字段"));
            }
            bytes.extend(encoded_string("栅格.afr"));
            if version < 4 {
                bytes.extend(12_i32.to_le_bytes());
                bytes.extend([0; 16]); // 旧版 IName。
            }
            bytes.extend(IDENTITY_FUNCTION_CLASS_ID);
            bytes.extend(81_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(9_i32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            if version >= 2 {
                bytes.extend((if version == 7 { 4_i32 } else { 0 }).to_le_bytes());
            }
            bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
            bytes.extend(82_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend([0; 16]);
            if version == 7 {
                bytes.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
                bytes.extend(82_u32.to_le_bytes());
                bytes.extend(RASTER_SCALAR_CLASS_ID);
                bytes.extend(84_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 16]); // 空 Variant。
                bytes.extend(RASTER_SCALAR_CLASS_ID);
                bytes.extend(84_u32.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend(PROPERTY_SET_CLASS_ID);
                bytes.extend(83_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(0_i32.to_le_bytes());
            }
            if (3..=4).contains(&version) {
                bytes.extend([0; 16]);
            }
            if version >= 6 {
                bytes.extend(encoded_string("附加字段"));
            }
            if version >= 7 {
                bytes.extend(encoded_string("末尾字段"));
            }
            bytes.extend(encoded_string("图层后缀"));
            let end = bytes.len();
            bytes.extend([0; 16 + 22]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::Function(name)) = &result.names[0].body else {
                panic!("预期 FunctionRasterDatasetName");
            };
            assert_eq!(name.path, "栅格.afr");
            assert_eq!(name.function.body.as_ref().unwrap().base_long, 9);
            assert!(matches!(
                name.inputs[0].body.as_deref(),
                Some(FunctionRasterInput10_1::Arguments(_))
            ));
            assert_eq!(
                name.legacy.as_ref().map(|legacy| legacy.long),
                (version < 4).then_some(12)
            );
            assert_eq!(name.properties.is_some(), version >= 3);
            assert_eq!(name.geometry.is_some(), version == 4);
            assert_eq!(
                name.string_48.as_deref(),
                (version >= 7).then_some("末尾字段")
            );
            if version == 7 {
                assert!(matches!(
                    name.inputs[1].entry,
                    ObjectEntry10_1::BackReference { key: 82, .. }
                ));
                assert!(matches!(
                    name.inputs[2].body.as_deref(),
                    Some(FunctionRasterInput10_1::Other(body))
                        if matches!(**body, RasterTemplateInput10_1::NativeArgument(
                            PropertyValueObjectStream10_1::RasterScalar { key: 84, .. }
                        ))
                ));
                assert_eq!(
                    name.inputs[3].entry,
                    ObjectEntry10_1::BackReference {
                        class_id: RASTER_SCALAR_CLASS_ID,
                        key: 84,
                    }
                );
            }
            assert!((80..=82).all(|key| refs.contains(key)));
            assert_eq!(refs.contains(83), version >= 3);
            assert!(result.tail.is_some());
            assert_eq!(reader.remaining(), 0);
            for cut in start..end {
                let mut reader = Reader::new(&bytes[..cut]);
                let mut refs = RefTable10_1::new();
                let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(result.names_stop.is_some());
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn cache_raster_name_reference_retains_workspace_and_dataset() {
        use super::super::raster_dataset_name::CACHE_RASTER_DATASET_NAME_CLASS_ID;
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = body_prefix(9, 2);
        bytes.extend(CACHE_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("cache_dataset"));
        bytes.extend(encoded_string("显示名称"));
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["cache", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 21]); // 空属性集、空工厂及尾整数。
        bytes.extend([0; 32]); // 空瓦片读取器及包络。
        bytes.extend(encoded_string(""));
        bytes.extend(CACHE_RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(encoded_string(""));

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(parsed.names_stop.is_none());
        assert_eq!(parsed.names.len(), 2);
        let source = parsed.names[0].source_reference.as_ref().unwrap();
        assert_eq!(source.name.as_ref(), "cache_dataset");
        assert_eq!(source.path.as_deref(), Some("cache"));
        assert!(parsed.names[1].body.is_none());
        assert_eq!(parsed.names[1].source_reference.as_ref(), Some(source));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn wcs_coverage_name_reference_retains_identifier() {
        use crate::layers::wcs_name::WCS_COVERAGE_NAME_CLASS_ID;

        let mut bytes = body_prefix(9, 2);
        bytes.extend(WCS_COVERAGE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(1_i16.to_le_bytes());
        let identifier = "coverage_a";
        bytes.extend((identifier.len() as u32).to_le_bytes());
        for unit in identifier.encode_utf16() {
            bytes.extend(unit.to_le_bytes());
        }
        bytes.extend(0_u16.to_le_bytes());
        bytes.extend([0; 16]); // 空连接名称。
        bytes.extend(encoded_string(""));
        bytes.extend(WCS_COVERAGE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(encoded_string(""));

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(parsed.names_stop.is_none());
        assert_eq!(parsed.names.len(), 2);
        let source = parsed.names[0].source_reference.as_ref().unwrap();
        assert_eq!(source.name.as_ref(), identifier);
        assert!(source.path.is_none());
        assert!(parsed.names[1].body.is_none());
        assert_eq!(parsed.names[1].source_reference.as_ref(), Some(source));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn netcdf_name_keeps_array_keys_and_respects_v1_envelope_gate() {
        use super::super::raster_dataset_name::{
            LONG_ARRAY_CLASS_ID, NETCDF_RASTER_DATASET_NAME_CLASS_ID,
        };
        use crate::objects::envelope::ENVELOPE_CLASS_ID;
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        for version in [1_i32, 2] {
            let mut bytes = body_prefix(1, 1);
            let start = bytes.len();
            bytes.extend(NETCDF_RASTER_DATASET_NAME_CLASS_ID);
            bytes.extend(76_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for value in ["首串", "", "", "", "栅格变量", "", "", ""] {
                bytes.extend(encoded_string(value));
            }
            if version == 1 {
                bytes.extend(encoded_string("旧版字段"));
                bytes.extend(ENVELOPE_CLASS_ID);
                bytes.extend(79_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend([0; 32]);
            }
            bytes.extend(STR_ARRAY_CLASS_ID);
            bytes.extend(77_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(encoded_string("时间"));
            bytes.extend(LONG_ARRAY_CLASS_ID);
            bytes.extend(78_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            let count_offset = bytes.len();
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(2026_i32.to_le_bytes());
            bytes.extend(WORKSPACE_NAME_CLASS_ID);
            bytes.extend(80_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            for value in ["climate.nc", "", ""] {
                bytes.extend(encoded_string(value));
            }
            bytes.extend([0; 21]); // 空属性集、空工厂及尾整数。
            bytes.extend(encoded_string("图层名称后缀"));
            let end = bytes.len();
            bytes.extend([0; 16 + 22]);

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::NetCdf(name)) = &result.names[0].body else {
                panic!("预期 NetCDFRasterDatasetName");
            };
            assert_eq!(name.strings[0], "首串");
            assert_eq!(name.variable(), "栅格变量");
            let source = result.names[0].source_reference.as_ref().unwrap();
            assert_eq!(source.name.as_ref(), "栅格变量");
            assert_eq!(source.path.as_deref(), Some("climate.nc"));
            assert_eq!(
                refs.dataset_source(NETCDF_RASTER_DATASET_NAME_CLASS_ID, 76),
                Some(source)
            );
            assert_eq!(name.legacy.is_some(), version == 1);
            assert_eq!(name.labels.body.as_ref().unwrap().1, ["时间"]);
            assert_eq!(name.values.body, Some((1, vec![2026])));
            assert!(refs.contains(76) && refs.contains(77) && refs.contains(78));
            assert_eq!(refs.contains(79), version == 1);
            assert!(result.tail.is_some());
            assert_eq!(reader.remaining(), 0);

            let mut reference = body_prefix(1, 1);
            reference.extend(NETCDF_RASTER_DATASET_NAME_CLASS_ID);
            reference.extend(76_u32.to_le_bytes());
            reference.extend(encoded_string(""));
            reference.extend([0; 16 + 22]);
            let mut reader = Reader::new(&reference);
            let referenced = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(referenced.names[0].body.is_none());
            assert_eq!(referenced.names[0].source_reference.as_ref(), Some(source));
            assert_eq!(reader.remaining(), 0);

            let mut invalid = bytes.clone();
            invalid[count_offset..count_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
            for input in (start..end)
                .map(|cut| &bytes[..cut])
                .chain([invalid.as_slice()])
            {
                let mut reader = Reader::new(input);
                let mut refs = RefTable10_1::new();
                let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(result.names_stop.is_some());
                assert_eq!(reader.position(), start);
                assert!(refs.is_empty());
            }
        }
    }

    #[test]
    fn sde_name_reads_tag6_without_a_key_and_shares_workspace_key() {
        use super::super::raster_dataset_name::{
            DatabaseRasterTag6Name10_1, RASTER_DATASET_NAME_CLASS_ID,
            SDE_RASTER_DATASET_NAME_CLASS_ID,
        };
        use crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID;

        let mut bytes = body_prefix(1, 1);
        let delegator = bytes.len();
        dataset_name(&mut bytes, 76);
        bytes[delegator..delegator + 16].copy_from_slice(&SDE_RASTER_DATASET_NAME_CLASS_ID);
        let record = |bytes: &mut Vec<u8>, tag: i32, data: &[u8]| {
            bytes.extend(tag.to_le_bytes());
            bytes.extend((data.len() as i32).to_le_bytes());
            bytes.extend(data);
        };
        record(&mut bytes, 0xff00fa, &[]);
        record(&mut bytes, 1, &33_i32.to_le_bytes());
        record(&mut bytes, 2, &[0; 2]);
        record(&mut bytes, 3, &[0; 2]);
        record(&mut bytes, 4, &12_i32.to_le_bytes());
        record(&mut bytes, 5, &[0; 2]);
        bytes.extend(6_i32.to_le_bytes());
        bytes.extend((-1_i32).to_le_bytes());
        let tag6 = bytes.len();
        bytes.extend(RASTER_DATASET_NAME_CLASS_ID);
        bytes.extend(2_i32.to_le_bytes());
        for value in ["嵌套栅格", "", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend(WORKSPACE_NAME_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for value in ["工作空间", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 21]); // 空属性集、空工厂和尾 int32。
        bytes.extend([0xff; 8]); // SDE 到 tag 6 后即读取结束对。
        bytes.extend(encoded_string("图层名称后缀"));
        bytes.extend([0; 16 + 22]);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let Some(RasterDatasetNameEntryBody10_1::Database(name)) = &result.names[0].body else {
            panic!("预期 SdeRasterDatasetName");
        };
        assert_eq!(name.delegated_name.key, 76);
        let records = name.records.as_ref().unwrap();
        let Some(DatabaseRasterTag6Name10_1::Raster {
            body: RasterDatasetNameEntryBody10_1::Dataset(nested),
            ..
        }) = &records.tag6_name
        else {
            panic!("预期无键 tag 6 栅格名称");
        };
        assert_eq!(nested.key, 0);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(76) && refs.contains(77));
        assert!(result.tail.is_some());
        assert_eq!(reader.remaining(), 0);

        bytes[tag6] ^= 0xff;
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(result.names_stop.is_some());
        assert_eq!(reader.position(), delegator);
        assert!(refs.is_empty());
    }

    #[test]
    fn raster_type_name_reads_mosaic_name_and_preserves_the_next_slot() {
        use super::super::raster_dataset_name::{
            MOSAIC_DATASET_NAME_CLASS_ID, RASTER_TYPE_NAME_CLASS_ID,
        };

        let mut bytes = body_prefix(17, 2);
        let name_start = bytes.len();
        bytes.extend(RASTER_TYPE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("名称串"));
        bytes.extend(encoded_string("栅格类型"));
        bytes.extend((-12_i32).to_le_bytes());
        bytes.extend(MOSAIC_DATASET_NAME_CLASS_ID);
        bytes.extend(77_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(encoded_string("镶嵌源"));
        bytes.extend(encoded_string("数据集"));
        bytes.extend([0; 16]); // 空工作空间。
        bytes.extend(encoded_string("首名称后缀"));
        let next_name = bytes.len();
        bytes.extend(RASTER_TYPE_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(encoded_string("回引后缀"));
        bytes.extend([0; 16]); // 空渲染器。

        for cut in name_start..next_name {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.names_stop.is_some());
            assert_eq!(reader.position(), name_start);
            assert!(refs.is_empty());
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(parsed.names_stop.is_none());
        let Some(RasterDatasetNameEntryBody10_1::RasterType(name)) = &parsed.names[0].body else {
            panic!("预期 RasterTypeName");
        };
        assert_eq!(name.version, 1);
        assert_eq!(name.name_string, "名称串");
        assert_eq!(name.name, "栅格类型");
        assert_eq!(name.instance_id, -12);
        assert_eq!(
            name.mosaic_dataset.body.as_ref().unwrap().strings,
            ["镶嵌源", "数据集"]
        );
        assert_eq!(parsed.names[0].suffix.as_deref(), Some("首名称后缀"));
        assert_eq!(
            parsed.names[1].entry,
            ObjectEntry10_1::BackReference {
                class_id: RASTER_TYPE_NAME_CLASS_ID,
                key: 76,
            }
        );
        assert_eq!(parsed.names[1].suffix.as_deref(), Some("回引后缀"));
        assert!(refs.contains(76));
        assert!(refs.contains(77));
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn mosaic_dataset_name_reads_versioned_fields_and_rolls_back_keys() {
        use super::super::raster_dataset_name::MOSAIC_DATASET_NAME_CLASS_ID;
        use crate::objects::string_array::STR_ARRAY_CLASS_ID;

        for version in 1_u16..=4 {
            let mut bytes = body_prefix(1, 1);
            let name_start = bytes.len();
            bytes.extend(MOSAIC_DATASET_NAME_CLASS_ID);
            bytes.extend(76_u32.to_le_bytes());
            bytes.extend(version.to_le_bytes());
            for value in ["镶嵌数据集", "数据表"] {
                bytes.extend(encoded_string(value));
            }
            bytes.extend(crate::objects::workspace_name::WORKSPACE_NAME_CLASS_ID);
            bytes.extend(78_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            for value in ["mosaic.gdb", "", ""] {
                bytes.extend(encoded_string(value));
            }
            bytes.extend([0; 21]); // 空属性集、空工厂及尾整数。
            if version >= 2 {
                bytes.extend(7_u16.to_le_bytes());
            }
            if version >= 3 {
                bytes.extend((-12_i32).to_le_bytes());
            }
            let array_start = bytes.len();
            if version >= 4 {
                bytes.extend(STR_ARRAY_CLASS_ID);
                bytes.extend(77_u32.to_le_bytes());
                bytes.extend(1_u16.to_le_bytes());
                bytes.extend(2_i32.to_le_bytes());
                for value in ["名称甲", "名称乙"] {
                    bytes.extend(encoded_string(value));
                }
            }
            bytes.extend(encoded_string("图层后缀"));
            let name_end = bytes.len();
            bytes.extend([0; 16 + 22]); // 空渲染器与 v1 尾段。

            let assert_rollback = |input: &[u8]| {
                let mut reader = Reader::new(input);
                let mut refs = RefTable10_1::new();
                let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
                assert!(parsed.names_stop.is_some());
                assert_eq!(reader.position(), name_start);
                assert!(refs.is_empty());
            };
            for cut in name_start..name_end {
                assert_rollback(&bytes[..cut]);
            }
            if version >= 4 {
                let mut malformed = bytes.clone();
                malformed[array_start + 22..array_start + 26]
                    .copy_from_slice(&i32::MAX.to_le_bytes());
                assert_rollback(&malformed);
            }

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::Mosaic(name)) = &parsed.names[0].body else {
                panic!("预期 MosaicDatasetName");
            };
            assert_eq!(name.version, version);
            assert_eq!(name.strings, ["镶嵌数据集", "数据表"]);
            let source = parsed.names[0].source_reference.as_ref().unwrap();
            assert_eq!(source.name.as_ref(), "镶嵌数据集");
            assert_eq!(source.path.as_deref(), Some("mosaic.gdb"));

            let mut reference = body_prefix(1, 1);
            reference.extend(MOSAIC_DATASET_NAME_CLASS_ID);
            reference.extend(76_u32.to_le_bytes());
            reference.extend(encoded_string(""));
            reference.extend([0; 16 + 22]);
            let mut reference_reader = Reader::new(&reference);
            let referenced =
                read_bounded_raster_body_10_1(&mut reference_reader, 4096, &mut refs).unwrap();
            assert!(referenced.names[0].body.is_none());
            assert_eq!(referenced.names[0].source_reference.as_ref(), Some(source));
            assert_eq!(reference_reader.remaining(), 0);
            assert_eq!(name.short_44, (version >= 2).then_some(7));
            assert_eq!(name.long_48, (version >= 3).then_some(-12));
            if version >= 4 {
                let array = name.string_array.as_ref().unwrap();
                let (array_version, strings) = array.body.as_ref().unwrap();
                assert_eq!(*array_version, 1);
                assert_eq!(strings, &["名称甲", "名称乙"]);
            } else {
                assert!(name.string_array.is_none());
            }
            assert!(refs.contains(76));
            assert_eq!(refs.contains(77), version >= 4);
            assert_eq!(parsed.names[0].suffix.as_deref(), Some("图层后缀"));
            assert!(parsed.tail.is_some());
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn catalog_name_reuses_feature_class_body_without_a_second_header() {
        use super::super::raster_dataset_name::RASTER_CATALOG_NAME_CLASS_ID;

        let mut bytes = body_prefix(1, 1);
        let name_start = bytes.len();
        bytes.extend(RASTER_CATALOG_NAME_CLASS_ID);
        bytes.extend(76_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        for value in ["栅格目录名", "", "", ""] {
            bytes.extend(encoded_string(value));
        }
        bytes.extend([0; 8 + 2 + 16 + 2]); // 两个 int32、选择子、空工作空间、选择子。
        bytes.extend(encoded_string("图层后缀"));
        let name_end = bytes.len();
        bytes.extend([0; 16 + 22]);
        for cut in name_start..name_end {
            let mut refs = RefTable10_1::new();
            let mut reader = Reader::new(&bytes[..cut]);
            let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(parsed.names_stop.is_some());
            assert_eq!(reader.position(), name_start);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let Some(RasterDatasetNameEntryBody10_1::Catalog(catalog)) = &parsed.names[0].body else {
            panic!("预期 RasterCatalogName");
        };
        assert_eq!(catalog.name, "栅格目录名");
        assert_eq!(catalog.key, 76);
        assert_eq!(parsed.names[0].suffix.as_deref(), Some("图层后缀"));
        assert!(parsed.tail.is_some());
        assert_eq!(refs.len(), 1);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn dataset_name83_leaves_layer_suffix_after_optional_records() {
        use super::super::raster_dataset_name::RASTER_DATASET_NAME83_CLASS_ID;

        for has_records in [false, true] {
            let mut bytes = body_prefix(1, 1);
            let name_start = bytes.len();
            dataset_name(&mut bytes, 62);
            bytes[name_start..name_start + 16].copy_from_slice(&RASTER_DATASET_NAME83_CLASS_ID);
            if has_records {
                for value in [0x00ff_00fa_i32, 0, 1, 4, 27, 2] {
                    bytes.extend(value.to_le_bytes());
                }
                bytes.extend(encoded_string("栅格"));
                bytes.extend([0xff; 8]);
            }
            bytes.extend(encoded_string("后继名称"));
            bytes.extend([0; 16 + 22]); // 空渲染器与 v1 尾段标量。
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result = read_bounded_raster_body_10_1(&mut reader, 4096, &mut refs).unwrap();
            let Some(RasterDatasetNameEntryBody10_1::Dataset83 { records, .. }) =
                &result.names[0].body
            else {
                panic!("预期 RasterDatasetName83");
            };
            assert_eq!(
                records
                    .as_ref()
                    .map(|value| (value.first_long, value.text.as_str())),
                has_records.then_some((27, "栅格")),
            );
            assert_eq!(result.names[0].suffix.as_deref(), Some("后继名称"));
            assert!(result.tail.is_some());
            assert!(refs.contains(62));
            assert_eq!(reader.remaining(), 0);
        }
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
