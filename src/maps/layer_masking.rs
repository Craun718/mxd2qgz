//! 10.1 LayerMasking 流的安全子集；QI 与 Map 门控证据见
//! docs/mxd-luna-deep-layer-masking-20261118.md，流结构边界见
//! docs/mxd-terra-round8-mask.md。

use crate::layers::layer_list::{LayerBodyPrefix10_1, read_complete_layer_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// Map 逐图层遮罩探测前的 Map 10.1 版本门结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerMaskingProbe10_1 {
    /// Map 版本小于 40 且图层支持 IBasemapLayer，原生跳过 ILayerMasking 查询。
    SkipLegacyBasemapLayer,
    /// 原生继续查询 ILayerMasking；查询失败时不消费遮罩流。
    QueryLayerMasking,
}

/// 按 Map.dll 10.1 的逐图层控制流决定是否查询 ILayerMasking。
///
/// `map_version` 是有符号 short：Map `0x79403d50` 在 `0x79403de5..0x79403df6`
/// 将 IBasemapLayer 查询成功与 `version < 40` 同时满足作为跳过条件；其他情况
/// 都继续执行 ILayerMasking 查询。此处只表达原生门控，不推断未知图层的 QI 结果。
pub fn layer_masking_probe_10_1(
    map_version: i16,
    basemap_interface_present: bool,
) -> LayerMaskingProbe10_1 {
    if basemap_interface_present && map_version < 40 {
        LayerMaskingProbe10_1::SkipLegacyBasemapLayer
    } else {
        LayerMaskingProbe10_1::QueryLayerMasking
    }
}

/// 已由 ArcMap 10.1.0.3035 静态 QI 路径确认的类级 ILayerMasking 结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerMaskingSupport10_1 {
    Supported,
    Unsupported,
    /// 未在本轮核实该类的主 QI 与委派/基类路径。
    Unresolved,
}

pub(crate) const DIMENSION_LAYER_CLASS_ID: [u8; 16] = [
    0x32, 0x7e, 0xe2, 0xf1, 0xa7, 0x0c, 0xd4, 0x11, 0x80, 0xd7, 0x00, 0xc0, 0x4f, 0x60, 0x15, 0x65,
];
pub(crate) const TOPOLOGY_LAYER_CLASS_ID: [u8; 16] = [
    0xd5, 0xba, 0x98, 0xde, 0x5e, 0x13, 0x36, 0x4b, 0x96, 0x53, 0x57, 0xb5, 0x51, 0x3f, 0xb7, 0xb2,
];
pub(crate) const NETWORK_LAYER_CLASS_ID: [u8; 16] = [
    0x4b, 0xe9, 0xf8, 0xd4, 0xf5, 0x5c, 0x8f, 0x4f, 0x8b, 0x4d, 0x5b, 0x25, 0xae, 0x4c, 0x0a, 0xf9,
];
const SCHEMATIC_LAYER_CLASS_ID: [u8; 16] = [
    0xd6, 0x82, 0xf0, 0x2e, 0x5f, 0x0b, 0x70, 0x4b, 0x94, 0x16, 0x35, 0xd7, 0x5e, 0xfa, 0x7f, 0xed,
];

const TEMPORAL_FEATURE_LAYER_CLASS_ID: [u8; 16] = [
    0x40, 0xce, 0x34, 0x33, 0x1d, 0x1a, 0xd5, 0x11, 0xb7, 0xcf, 0x00, 0x01, 0x02, 0x65, 0xad, 0xc5,
];

const WMS_LAYER_CLASS_ID: [u8; 16] = [
    0xf6, 0xa8, 0x0d, 0x5b, 0x43, 0x5e, 0xae, 0x40, 0x98, 0x71, 0x56, 0xba, 0x33, 0x93, 0x6f, 0x30,
];

const WMS_GROUP_LAYER_CLASS_ID: [u8; 16] = [
    0x62, 0xba, 0x77, 0xf6, 0xa7, 0x7c, 0x0a, 0x40, 0x9c, 0x59, 0x62, 0x93, 0x0a, 0x28, 0x2c, 0xeb,
];

const WMS_MAP_LAYER_CLASS_ID: [u8; 16] = [
    0xc0, 0x56, 0x8a, 0xe3, 0xbd, 0xd5, 0x99, 0x48, 0xb0, 0x89, 0xc8, 0xed, 0x4e, 0x38, 0xb7, 0x7f,
];

// 这些类尚无图层体读取器，但已确认不消费遮罩记录。
pub(crate) fn is_unparsed_layer_without_masking_10_1(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        DIMENSION_LAYER_CLASS_ID
            | TOPOLOGY_LAYER_CLASS_ID
            | NETWORK_LAYER_CLASS_ID
            | SCHEMATIC_LAYER_CLASS_ID
            | TEMPORAL_FEATURE_LAYER_CLASS_ID
            | WMS_LAYER_CLASS_ID
            | WMS_MAP_LAYER_CLASS_ID
            | WMS_GROUP_LAYER_CLASS_ID
    )
}

/// 返回本轮已核实的类级 ILayerMasking QI 结果；未知类保持未决。
///
/// 这里的类标识只用于静态能力表，不能代替运行时 QI，也不涵盖插件替换类。
pub fn known_layer_masking_support_10_1(class_id: &[u8; 16]) -> LayerMaskingSupport10_1 {
    use crate::layers::basemap_layer::BASEMAP_LAYER_CLASS_ID;
    use crate::layers::cache_tile_reader::is_tiled_layer;
    use crate::layers::cad_layer::{
        CAD_ANNOTATION_LAYER_CLASS_ID, CAD_FEATURE_LAYER_CLASS_ID, CAD_LAYER_CLASS_ID,
    };
    use crate::layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID;
    use crate::layers::coverage_annotation_layer::COVERAGE_ANNOTATION_LAYER_CLASS_ID;
    use crate::layers::fdo_graphics_layer::FDO_GRAPHICS_LAYER_CLASS_ID;
    use crate::layers::fdo_legacy::FDO_GRAPHICS_SUBLAYER_CLASS_ID;
    use crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
    use crate::layers::graphics_3d::{GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID};
    use crate::layers::group_layer::GROUP_LAYER_CLASS_ID;
    use crate::layers::image_server_layer::IMAGE_SERVER_LAYER_CLASS_ID;
    use crate::layers::process_layer::PROCESS_LAYER_CLASS_ID;

    use crate::layers::layer_list::RASTER_LAYER_CLASS_ID;
    use crate::layers::wcs_layer::WCS_LAYER_CLASS_ID;

    if matches!(
        *class_id,
        GROUP_LAYER_CLASS_ID | BASEMAP_LAYER_CLASS_ID | PROCESS_LAYER_CLASS_ID
    ) {
        LayerMaskingSupport10_1::Supported
    } else if *class_id == FEATURE_LAYER_CLASS_ID
        || *class_id == COMPOSITE_GRAPHICS_LAYER_CLASS_ID
        || *class_id == FDO_GRAPHICS_LAYER_CLASS_ID
        || *class_id == FDO_GRAPHICS_SUBLAYER_CLASS_ID
        || *class_id == RASTER_LAYER_CLASS_ID
        || *class_id == CAD_LAYER_CLASS_ID
        || *class_id == CAD_FEATURE_LAYER_CLASS_ID
        || *class_id == CAD_ANNOTATION_LAYER_CLASS_ID
        || *class_id == COVERAGE_ANNOTATION_LAYER_CLASS_ID
        || *class_id == WCS_LAYER_CLASS_ID
        || *class_id == IMAGE_SERVER_LAYER_CLASS_ID
        || *class_id == GRAPHICS_LAYER_3D_CLASS_ID
        || *class_id == GLOBE_GRAPHICS_LAYER_CLASS_ID
        || is_tiled_layer(*class_id)
        || is_unparsed_layer_without_masking_10_1(*class_id)
    {
        LayerMaskingSupport10_1::Unsupported
    } else {
        LayerMaskingSupport10_1::Unresolved
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaskEntry10_1 {
    /// 被遮罩的 ILayer 对象槽。
    pub object: ObjectEntry10_1,
    /// 提供遮罩的对象槽及其 LevelMasking 字符串。旧版本无此字符串。
    pub targets: Vec<(ObjectEntry10_1, Option<String>)>,
    /// 本条记录首次出现的完整图层体，按读取顺序保留对象键。
    pub new_layer_bodies: Vec<(u32, LayerBodyPrefix10_1)>,
}

impl MaskEntry10_1 {
    pub fn masked_layer(&self) -> ObjectEntry10_1 {
        self.object
    }

    pub fn masking_layers(&self) -> &[(ObjectEntry10_1, Option<String>)] {
        &self.targets
    }

    /// 读取指定遮罩层的已识别层级设置。
    pub fn masking_levels(&self, index: usize) -> Option<MaskingLevels10_1> {
        let (_, text) = self.targets.get(index)?;
        decode_masking_levels_10_1(text.as_deref()?)
    }
}

/// 符号层级过滤器。空字符串匹配所有层级。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaskingLevels10_1 {
    All,
    Selected(Vec<std::ops::RangeInclusive<i32>>),
}

/// 解码单个层级和闭区间，无法识别时由调用者继续保留原字符串。
pub fn decode_masking_levels_10_1(text: &str) -> Option<MaskingLevels10_1> {
    let text = text.trim();
    if text.is_empty() {
        return Some(MaskingLevels10_1::All);
    }
    let mut levels = Vec::new();
    for token in text.split([',', ';']).filter(|token| !token.is_empty()) {
        let (start, end) = if let Some((start, end)) = token.split_once('-') {
            (start.trim().parse().ok()?, end.trim().parse().ok()?)
        } else {
            let level = token.trim().parse().ok()?;
            (level, level)
        };
        levels.push(start..=end);
    }
    Some(MaskingLevels10_1::Selected(levels))
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerMasking10_1 {
    pub version: u16,
    pub entries: Vec<MaskEntry10_1>,
    /// ILayerMasking::UseMasking 的原始 VARIANT_BOOL 位模式。
    pub tail: u16,
}

impl LayerMasking10_1 {
    pub fn use_masking_raw(&self) -> u16 {
        self.tail
    }
}

pub(super) fn read_masking_count_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<i32> {
    let value = i32::from_le_bytes(reader.read_bytes(4)?.try_into().unwrap());
    // 原生只循环正数，诊断保留计数的原始有符号值。
    let n = value.max(0) as usize;
    if n.checked_mul(16).is_none_or(|bytes| bytes > max_bytes) {
        return Err(io::Error::new(ErrorKind::InvalidData, "遮罩计数超过限额"));
    }
    Ok(value)
}

// 外层请求 ILayer，内层请求 IUnknown。同一图层体分派适用于两处；
// 内层其他可持久化对象仍需该类的完整读取器。
fn read_mask_object(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    new_layer_bodies: &mut Vec<(u32, LayerBodyPrefix10_1)>,
) -> io::Result<ObjectEntry10_1> {
    let entry = read_object_entry_10_1(reader, refs)?;
    if let ObjectEntry10_1::NewObject { class_id, key } = entry {
        let body = read_complete_layer_body_10_1(reader, class_id, max_bytes, refs)?;
        new_layer_bodies.push((key, body));
    }
    Ok(entry)
}

pub(super) fn read_masking_version_10_1(reader: &mut Reader<'_>) -> io::Result<u16> {
    let version = reader.read_u16_le()?;
    // 原生按有符号 short 比较，诊断继续保留原始位模式。
    if version as i16 > 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "遮罩版本不受支持",
        ));
    }
    Ok(version)
}

/// 读取已确认接口存在时的集合；失败时游标和引用表均不变。
pub fn read_layer_masking_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LayerMasking10_1> {
    let mut trial = reader.clone();
    let mut table = refs.clone();
    let version = read_masking_version_10_1(&mut trial)?;
    let n = read_masking_count_10_1(&mut trial, max_bytes)?.max(0) as usize;
    let entries = read_entries_10_1(&mut trial, max_bytes, &mut table, version, n)?;
    let tail = trial.read_u16_le()?;
    *reader = trial;
    *refs = table;
    Ok(LayerMasking10_1 {
        version,
        entries,
        tail,
    })
}

/// 消费已确认的外层计数之后的条目；新图层体必须完整读取，未知对象使整个操作回退。
pub fn read_entries_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    version: u16,
    n: usize,
) -> io::Result<Vec<MaskEntry10_1>> {
    if version as i16 > 2 || n.checked_mul(16).is_none_or(|bytes| bytes > max_bytes) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "遮罩版本或计数不受支持",
        ));
    }
    let mut trial = reader.clone();
    let mut table = refs.clone();
    let mut entries = Vec::new();
    let mut total = n;
    for _ in 0..n {
        let mut new_layer_bodies = Vec::new();
        let object = read_mask_object(&mut trial, max_bytes, &mut table, &mut new_layer_bodies)?;
        let inner = read_masking_count_10_1(&mut trial, max_bytes)?.max(0) as usize;
        total = total
            .checked_add(inner)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "遮罩计数溢出"))?;
        if total.checked_mul(16).is_none_or(|bytes| bytes > max_bytes) {
            return Err(io::Error::new(ErrorKind::InvalidData, "遮罩总计数超过限额"));
        }
        let mut targets = Vec::new();
        for _ in 0..inner {
            let target =
                read_mask_object(&mut trial, max_bytes, &mut table, &mut new_layer_bodies)?;
            let name = if version as i16 > 1 {
                Some(read_string_10_1(&mut trial, max_bytes)?)
            } else {
                None
            };
            targets.push((target, name));
        }
        entries.push(MaskEntry10_1 {
            object,
            targets,
            new_layer_bodies,
        });
    }
    *reader = trial;
    *refs = table;
    Ok(entries)
}

/// 接口不存在时原生函数不读流；接口状态须由调用者提供。
pub fn read_optional_layer_masking_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    interface_present: bool,
) -> io::Result<Option<LayerMasking10_1>> {
    if interface_present {
        read_layer_masking_10_1(reader, max_bytes, refs).map(Some)
    } else {
        Ok(None)
    }
}
