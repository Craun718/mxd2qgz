//! 10.1 LayerMasking 流的安全子集；QI 与 Map 门控证据见
//! docs/mxd-luna-deep-layer-masking-20261118.md，流结构边界见
//! docs/mxd-terra-round8-mask.md。

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

/// 返回本轮已核实的类级 ILayerMasking QI 结果；未知类保持未决。
///
/// 这里的类标识只用于静态能力表，不能代替运行时 QI，也不涵盖插件替换类。
pub fn known_layer_masking_support_10_1(class_id: &[u8; 16]) -> LayerMaskingSupport10_1 {
    use crate::layers::fdo_graphics_layer::FDO_GRAPHICS_LAYER_CLASS_ID;
    use crate::layers::fdo_legacy::FDO_GRAPHICS_SUBLAYER_CLASS_ID;
    use crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
    use crate::layers::group_layer::GROUP_LAYER_CLASS_ID;

    use crate::layers::layer_list::RASTER_LAYER_CLASS_ID;

    if *class_id == GROUP_LAYER_CLASS_ID {
        LayerMaskingSupport10_1::Supported
    } else if *class_id == FEATURE_LAYER_CLASS_ID
        || *class_id == FDO_GRAPHICS_LAYER_CLASS_ID
        || *class_id == FDO_GRAPHICS_SUBLAYER_CLASS_ID
        || *class_id == RASTER_LAYER_CLASS_ID
    {
        LayerMaskingSupport10_1::Unsupported
    } else {
        LayerMaskingSupport10_1::Unresolved
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskEntry10_1 {
    pub object: ObjectEntry10_1,
    pub targets: Vec<(ObjectEntry10_1, Option<String>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerMasking10_1 {
    pub version: u16,
    pub entries: Vec<MaskEntry10_1>,
    pub tail: u16,
}

fn count(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<usize> {
    let value = i32::from_le_bytes(reader.read_bytes(4)?.try_into().unwrap());
    let n = usize::try_from(value)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "遮罩计数为负"))?;
    if n.checked_mul(16).is_none_or(|bytes| bytes > max_bytes) {
        return Err(io::Error::new(ErrorKind::InvalidData, "遮罩计数超过限额"));
    }
    Ok(n)
}

fn reference(reader: &mut Reader<'_>, refs: &mut RefTable10_1) -> io::Result<ObjectEntry10_1> {
    match read_object_entry_10_1(reader, refs)? {
        ObjectEntry10_1::NewObject { .. } => {
            Err(io::Error::new(ErrorKind::Unsupported, "遮罩对象体尚未支持"))
        }
        entry => Ok(entry),
    }
}

/// 读取已确认接口存在时的集合；失败时游标和引用表均不变。
pub fn read_layer_masking_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<LayerMasking10_1> {
    let mut trial = reader.clone();
    let mut table = refs.clone();
    let version = trial.read_u16_le()?;
    if version > 2 {
        return Err(io::Error::new(ErrorKind::InvalidData, "遮罩版本不受支持"));
    }
    let n = count(&mut trial, max_bytes)?;
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

/// 消费已确认的外层计数之后的条目；未知对象体使整个操作回退。
pub fn read_entries_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    version: u16,
    n: usize,
) -> io::Result<Vec<MaskEntry10_1>> {
    if version > 2 || n.checked_mul(16).is_none_or(|bytes| bytes > max_bytes) {
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
        let object = reference(&mut trial, &mut table)?;
        let inner = count(&mut trial, max_bytes)?;
        total = total
            .checked_add(inner)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "遮罩计数溢出"))?;
        if total.checked_mul(16).is_none_or(|bytes| bytes > max_bytes) {
            return Err(io::Error::new(ErrorKind::InvalidData, "遮罩总计数超过限额"));
        }
        let mut targets = Vec::new();
        for _ in 0..inner {
            let target = reference(&mut trial, &mut table)?;
            let name = if version > 1 {
                Some(read_string_10_1(&mut trial, max_bytes)?)
            } else {
                None
            };
            targets.push((target, name));
        }
        entries.push(MaskEntry10_1 { object, targets });
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
