//! ArcMap 10.1 BasemapLayer 的可证前缀；遇到未闭合对象体即停止。
//! 原生调用与范围见 docs/mxd-round3-basemap-20261117.md。
use crate::maps::map_object::{MAP_CLASS_ID, MapSuffixStop, read_map_suffix_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::{native_string::read_string_10_1, reader::Reader};
use std::io::{self, ErrorKind};

pub const BASEMAP_LAYER_CLASS_ID: [u8; 16] = [
    0xbf, 0x22, 0x41, 0xda, 0x07, 0x7b, 0x58, 0x41, 0x88, 0xb0, 0x19, 0xd3, 0x42, 0xbe, 0xd8, 0xba,
];
pub const ARRAY_OF_I_SHADER_CLASS_ID: [u8; 16] = [
    0xaf, 0xdb, 0xbb, 0xe0, 0x59, 0x00, 0x9c, 0x44, 0xb6, 0xf1, 0xed, 0xf1, 0x35, 0x3a, 0x2c, 0x54,
];

pub(super) const RASTER_SHADER_CLASS_ID: [u8; 16] = [
    0xac, 0x5d, 0x89, 0x9a, 0x65, 0xe5, 0x8e, 0x48, 0xa5, 0xf4, 0x8b, 0x39, 0x53, 0x27, 0xe2, 0xbe,
];
pub(super) const RASTER_DRA_SHADER_CLASS_ID: [u8; 16] = [
    0x00, 0x72, 0x60, 0x61, 0x6f, 0x1e, 0x9e, 0x48, 0x88, 0xae, 0xbc, 0xce, 0xe8, 0x9d, 0x25, 0xc4,
];

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderBody10_1 {
    RasterDra {
        version: u16,
        integer: i32,
        values: [f32; 3],
    },
    Raster {
        version: u16,
        integers: [i32; 2],
        value: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderEntry10_1 {
    pub entry: ObjectEntry10_1,
    pub body: Option<ShaderBody10_1>,
}

pub(super) fn read_shader_entries(
    r: &mut Reader<'_>,
    refs: &mut RefTable10_1,
    n: u32,
    max: usize,
) -> io::Result<Vec<ShaderEntry10_1>> {
    if n as usize > max / 20 || n as usize > r.remaining() / 20 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "着色器数组计数超出可读范围",
        ));
    }
    (0..n)
        .map(|_| {
            let mut entry = read_object_entry_10_1(r, refs)?;
            let body = match &mut entry {
                ObjectEntry10_1::NewObject { class_id, .. }
                    if matches!(
                        *class_id,
                        RASTER_SHADER_CLASS_ID | RASTER_DRA_SHADER_CLASS_ID
                    ) =>
                {
                    let version = r.read_u16_le()?;
                    if version > 1 {
                        return Err(io::Error::new(ErrorKind::InvalidData, "着色器对象版本超限"));
                    }
                    Some(if *class_id == RASTER_DRA_SHADER_CLASS_ID {
                        ShaderBody10_1::RasterDra {
                            version,
                            integer: r.read_u32_le()? as i32,
                            values: [
                                f32::from_bits(r.read_u32_le()?),
                                f32::from_bits(r.read_u32_le()?),
                                f32::from_bits(r.read_u32_le()?),
                            ],
                        }
                    } else {
                        ShaderBody10_1::Raster {
                            version,
                            integers: [r.read_u32_le()? as i32, r.read_u32_le()? as i32],
                            value: r.read_f64_le()?,
                        }
                    })
                }
                ObjectEntry10_1::BackReference { class_id, key }
                    if matches!(
                        refs.registered_class(*key),
                        Some(RASTER_SHADER_CLASS_ID | RASTER_DRA_SHADER_CLASS_ID)
                    ) =>
                {
                    *class_id = refs.registered_class(*key).unwrap();
                    None
                }
                _ => {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "IShader 元素需提供受支持的非空对象",
                    ));
                }
            };
            Ok(ShaderEntry10_1 { entry, body })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum BasemapStop10_1 {
    /// 未消费的普通对象头及其未闭合体。
    NewObject {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 要素图层未完全读取；保留其对象头及未提交的体。
    FeatureLayer {
        key: u32,
        body_stop_offset: usize,
        reason: crate::layers::feature_layer::FeatureLayerStop10_1,
    },
    /// 数组非空时元素体尚未接线。
    ShaderElements {
        offset: usize,
        count: u32,
    },
    /// 非空扩展可能登记后续引用使用的对象，保留整个扩展段。
    Extensions {
        offset: usize,
        count: u32,
    },
    Complete,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BasemapPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub results_count: Option<u32>,
    pub shader_entries: Vec<ShaderEntry10_1>,
    /// 已完整读取并登记的结果列表 FeatureLayer 键。
    pub feature_keys: Vec<u32>,
    pub sublayers: Vec<ObjectEntry10_1>,
    pub sublayer_bodies: Vec<(u32, Box<super::layer_list::LayerBodyPrefix10_1>)>,
    pub stop: BasemapStop10_1,
}
fn count(reader: &mut Reader<'_>, limit: usize) -> io::Result<u32> {
    let n = reader.read_u32_le()?;
    if n > i32::MAX as u32 || n as usize > limit {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Basemap 计数为负或超过资源限额",
        ));
    }
    Ok(n)
}
fn slot(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    feature_keys: &mut Vec<u32>,
    allow_feature: bool,
) -> io::Result<Option<BasemapStop10_1>> {
    let offset = reader.position();
    let mut probe = reader.clone();
    match read_header_10_1(&mut probe)? {
        ObjectHeader::Object { class_id, key } if !table.contains(key) => {
            if allow_feature && class_id == crate::layers::feature_layer::FEATURE_LAYER_CLASS_ID {
                let mut refs = table.clone();
                refs.register(key);
                let body = crate::layers::feature_layer::read_feature_layer_prefix_10_1(
                    &mut probe, max_bytes, &mut refs,
                )?;
                if matches!(
                    body.stop,
                    crate::layers::feature_layer::FeatureLayerStop10_1::Successor(
                        crate::layers::feature_layer_tail::SuccessorStop10_1::TailCompleted
                    )
                ) {
                    *reader = probe;
                    *table = refs;
                    feature_keys.push(key);
                    return Ok(None);
                }
                return Ok(Some(BasemapStop10_1::FeatureLayer {
                    key,
                    body_stop_offset: probe.position(),
                    reason: body.stop,
                }));
            }
            if allow_feature && class_id == MAP_CLASS_ID {
                let mut refs = table.clone();
                read_object_entry_10_1(&mut reader.clone(), &mut refs)?;
                refs.mark_map(key);
                let parsed = refs
                    .with_nested_graphics(|refs| read_map_suffix_10_1(&mut probe, max_bytes, refs));
                if matches!(parsed, Ok(body) if body.stop == MapSuffixStop::Complete) {
                    *reader = probe;
                    *table = refs;
                    return Ok(None);
                }
            }
            Ok(Some(BasemapStop10_1::NewObject {
                offset,
                class_id,
                key,
            }))
        }
        _ => {
            *reader = probe;
            Ok(None)
        }
    }
}
/// 读取已登记的 Basemap 新对象体；错误时游标和引用表均不变。
pub fn read_basemap_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<BasemapPrefix10_1> {
    let mut r = reader.clone();
    let mut t = table.clone();
    let v = r.read_u16_le()?;
    if v > 11 {
        return Err(io::Error::new(ErrorKind::InvalidData, "Basemap 版本不支持"));
    }
    let name = read_string_10_1(&mut r, max_bytes)?;
    r.read_bytes(if v < 8 { 6 } else { 4 })?;
    r.read_bytes(18)?;
    if v > 1 {
        r.read_bytes(16)?;
    }
    let mut result_count = None;
    let mut shader_entries = Vec::new();
    let mut feature_keys = Vec::new();
    let mut sublayers = Vec::new();
    let mut sublayer_bodies = Vec::new();
    let stop = 'parse: {
        if v > 2
            && let Some(s) = slot(&mut r, &mut t, max_bytes, &mut feature_keys, false)?
        {
            break 'parse s;
        }
        if v > 3 {
            let offset = r.position();
            let mut probe = r.clone();
            let mut refs = t.clone();
            match read_object_entry_10_1(&mut probe, &mut refs)? {
                ObjectEntry10_1::NewObject { class_id, key: _ }
                    if class_id == ARRAY_OF_I_SHADER_CLASS_ID =>
                {
                    let av = probe.read_u16_le()?;
                    if av > 1 {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "ArrayOfIShader 版本不支持",
                        ));
                    }
                    let n = count(&mut probe, max_bytes / 16)?;
                    match read_shader_entries(&mut probe, &mut refs, n, max_bytes) {
                        Ok(entries) => shader_entries = entries,
                        Err(error) if error.kind() == ErrorKind::Unsupported => {
                            break 'parse BasemapStop10_1::ShaderElements { offset, count: n };
                        }
                        Err(error) => return Err(error),
                    }
                    r = probe;
                    t = refs;
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    break 'parse BasemapStop10_1::NewObject {
                        offset,
                        class_id,
                        key,
                    };
                }
                _ => {
                    r = probe;
                    t = refs;
                }
            }
        }
        if (v > 4 && v < 7) || v > 8 {
            r.read_bytes(2)?;
        }
        if v > 5 {
            let raw_count = r.read_u32_le()?;
            result_count = Some(raw_count);
            let n = (raw_count as i32).max(0) as usize;
            if n > max_bytes.min(r.remaining()) / 30 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Basemap 准备结果计数超过范围",
                ));
            }
            for _ in 0..n {
                let rv = r.read_u16_le()?;
                if rv as i16 > 2 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "PrepareResult 版本不支持",
                    ));
                }
                r.read_bytes(8)?;
                use crate::maps::page_layout_tail::{CONVERTER_OPTION, read_converter_option};
                if matches!(
                    read_header_10_1(&mut r.clone())?,
                    ObjectHeader::Object { class_id, key }
                        if t.registered_class(key).unwrap_or(class_id) == CONVERTER_OPTION
                ) {
                    read_converter_option(&mut r, max_bytes, &mut t)?;
                } else if let Some(s) = slot(&mut r, &mut t, max_bytes, &mut feature_keys, false)? {
                    break 'parse s;
                }
                let items = if rv == 1 {
                    2
                } else {
                    let n = (r.read_u32_le()? as i32).max(0) as u32;
                    if n as usize > max_bytes.min(r.remaining()) / 16 {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            "Basemap 准备结果对象计数超过范围",
                        ));
                    }
                    n
                };
                for item in 0..items {
                    if rv == 1
                        && item == 1
                        && let ObjectHeader::Object { class_id, key } =
                            read_header_10_1(&mut r.clone())?
                        && !(t.contains_map(key)
                            || t.registered_class(key).unwrap_or(class_id) == MAP_CLASS_ID)
                    {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "Basemap 准备结果 v1 的 IMap 槽未指向 Map",
                        ));
                    }
                    if let Some(s) = slot(&mut r, &mut t, max_bytes, &mut feature_keys, true)? {
                        break 'parse s;
                    }
                }
            }
        }
        if v >= 9 {
            r.read_u16_le()?;
        }
        let n = count(&mut r, max_bytes / 8)?;
        for _ in 0..n {
            let mut probe = r.clone();
            let a = probe.read_u32_le()?;
            let b = probe.read_u32_le()?;
            if (a, b) == (0, 0) {
                r = probe;
                continue;
            }
            let offset = probe.position();
            let ObjectHeader::Object { class_id, key } = read_header_10_1(&mut probe)? else {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "Basemap 非空子图层条目缺少对象头",
                ));
            };
            if !t.contains(key)
                && matches!(
                    class_id,
                    super::group_layer::GROUP_LAYER_CLASS_ID
                        | super::feature_layer::FEATURE_LAYER_CLASS_ID
                )
            {
                let length = if b == 0 { a } else { b };
                if length >= 20 && length as usize <= max_bytes {
                    let mut candidate = probe.clone();
                    let mut region = candidate.read_region(length as usize - 20)?;
                    let mut refs = t.clone();
                    refs.register(key);
                    if let Ok(body) = super::layer_list::read_complete_layer_body_10_1(
                        &mut region,
                        class_id,
                        max_bytes,
                        &mut refs,
                    ) && region.remaining() == 0
                    {
                        sublayer_bodies.push((key, Box::new(body)));
                        sublayers.push(ObjectEntry10_1::NewObject { class_id, key });
                        r = candidate;
                        t = refs;
                        continue;
                    }
                }
            }
            if !t.contains(key) {
                break 'parse BasemapStop10_1::NewObject {
                    offset,
                    class_id,
                    key,
                };
            }
            if (a, b) != (20, 0) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "Basemap 子图层回引的 ExtraSafe 边界尚未支持",
                ));
            }
            sublayers.push(ObjectEntry10_1::BackReference { class_id, key });
            r = probe;
        }
        if v >= 10 {
            let mut probe = r.clone();
            let n = count(&mut probe, max_bytes / 24)?;
            if n != 0 {
                break 'parse BasemapStop10_1::Extensions {
                    offset: r.position(),
                    count: n,
                };
            }
            read_string_10_1(&mut probe, max_bytes)?;
            probe.read_bytes(16)?;
            r = probe;
        }
        if v >= 11 {
            r.read_bytes(8)?;
        }
        BasemapStop10_1::Complete
    };
    *reader = r;
    *table = t;
    Ok(BasemapPrefix10_1 {
        version: v,
        name,
        results_count: result_count,
        shader_entries,
        feature_keys,
        sublayers,
        sublayer_bodies,
        stop,
    })
}
