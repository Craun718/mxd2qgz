//! ArcMap 10.1 BasemapLayer 的可证前缀；遇到未闭合对象体即停止。
//! 原生调用与范围见 docs/mxd-round3-basemap-20261117.md。
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasemapStop10_1 {
    /// 未消费的普通对象头及其未闭合体。
    NewObject {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 数组非空时元素体尚未接线。
    ShaderElements { offset: usize, count: u32 },
    /// 结果容器结束后的字段尚未接线。
    AfterResults { offset: usize },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasemapPrefix10_1 {
    pub version: u16,
    pub name: String,
    pub results_count: Option<u32>,
    /// 已完整读取并登记的结果列表 FeatureLayer 键。
    pub feature_keys: Vec<u32>,
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
    let mut feature_keys = Vec::new();
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
                    if n != 0 {
                        break 'parse BasemapStop10_1::ShaderElements { offset, count: n };
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
            let n = count(&mut r, max_bytes / 10)?;
            result_count = Some(n);
            for _ in 0..n {
                let rv = r.read_u16_le()?;
                if rv > 2 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "PrepareResult 版本不支持",
                    ));
                }
                r.read_bytes(8)?;
                if let Some(s) = slot(&mut r, &mut t, max_bytes, &mut feature_keys, false)? {
                    break 'parse s;
                }
                let items = if rv == 1 {
                    2
                } else {
                    count(&mut r, max_bytes / 16)?
                };
                for _ in 0..items {
                    if let Some(s) = slot(&mut r, &mut t, max_bytes, &mut feature_keys, true)? {
                        break 'parse s;
                    }
                }
            }
        }
        BasemapStop10_1::AfterResults {
            offset: r.position(),
        }
    };
    *reader = r;
    *table = t;
    Ok(BasemapPrefix10_1 {
        version: v,
        name,
        results_count: result_count,
        feature_keys,
        stop,
    })
}
