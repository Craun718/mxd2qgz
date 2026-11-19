//! CadLayer.dll 与 CadFDB.dll 10.1.0.3035 的 CAD 绘图图层持久化体。

use std::io::{self, ErrorKind};

use crate::layers::feature_layer::{
    FEATURE_LAYER_CLASS_ID, FeatureLayerPrefix10_1, FeatureLayerStop10_1,
    read_feature_layer_prefix_10_1,
};
use crate::layers::feature_layer_tail::SuccessorStop10_1;
use crate::layers::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use crate::objects::dataset_name_more::{DatasetNameMoreSlot10_1, read_dataset_name_slot_10_1};
use crate::objects::envelope::{SpatialReferenceEntry10_1, read_spatial_reference_entry_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;

pub const CAD_LAYER_CLASS_ID: [u8; 16] = [
    0xbd, 0xad, 0x99, 0xe2, 0xc3, 0xa5, 0xd2, 0x11, 0x9b, 0x10, 0, 0xc0, 0x4f, 0xa3, 0x32, 0x99,
];

pub const CAD_FEATURE_LAYER_CLASS_ID: [u8; 16] = [
    0xb6, 0x84, 0xf3, 0xe0, 0xc1, 0xe0, 0xd2, 0x11, 0x9b, 0x30, 0, 0xc0, 0x4f, 0xa3, 0x32, 0x99,
];

pub const CAD_DRAWING_OBJECT_CLASS_ID: [u8; 16] = [
    0xb8, 0x84, 0xf3, 0xe0, 0xc1, 0xe0, 0xd2, 0x11, 0x9b, 0x30, 0, 0xc0, 0x4f, 0xa3, 0x32, 0x99,
];

#[derive(Debug, Clone, PartialEq)]
pub struct CadDrawingObject10_1 {
    pub paths: [String; 2],
    pub transform_enabled: u16,
    pub flags: u32,
    pub base: [f64; 4],
    pub destination: [f64; 4],
    pub transform_parameters: [f64; 2],
    pub values: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CadLayer10_1 {
    pub version: u16,
    pub name: String,
    pub shorts: [u16; 3],
    pub scales: [f64; 2],
    pub spatial_reference: SpatialReferenceEntry10_1,
    pub additional_shorts: Option<[u16; 4]>,
    pub drawing_name: Option<DatasetNameMoreSlot10_1>,
    pub drawing_entry: ObjectEntry10_1,
    pub drawing: Option<Box<CadDrawingObject10_1>>,
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub path: Option<String>,
    pub value: Option<f64>,
    pub flags: Option<u32>,
    pub georeference: Option<String>,
    pub legacy_georeference_flag: Option<u16>,
    pub georeference_parameters: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CadFeatureLayer10_1 {
    pub version: u16,
    pub feature_header: ObjectHeader,
    pub feature: Option<Box<FeatureLayerPrefix10_1>>,
    pub drawing_entry: ObjectEntry10_1,
    pub drawing: Option<Box<CadDrawingObject10_1>>,
    pub layer_values: Vec<u16>,
    pub georeference: Option<String>,
    pub legacy_georeference_flag: Option<u16>,
    pub georeference_parameters: Option<[f64; 2]>,
}

fn read_drawing_10_1(r: &mut Reader<'_>, max: usize) -> io::Result<CadDrawingObject10_1> {
    let version = r.read_u16_le()?;
    if version != 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("CadDrawingObject 不支持版本 {version}"),
        ));
    }
    let paths = [read_string_10_1(r, max)?, read_string_10_1(r, max)?];
    let transform_enabled = r.read_u16_le()?;
    let flags = r.read_u32_le()?;
    let base = [
        r.read_f64_le()?,
        r.read_f64_le()?,
        r.read_f64_le()?,
        r.read_f64_le()?,
    ];
    let destination = [
        r.read_f64_le()?,
        r.read_f64_le()?,
        r.read_f64_le()?,
        r.read_f64_le()?,
    ];
    let transform_parameters = [r.read_f64_le()?, r.read_f64_le()?];
    // 原生对非正计数不读整数，正计数的所有整数均由流消费。
    let count = (r.read_u32_le()? as i32).max(0) as usize;
    let bytes = count
        .checked_mul(4)
        .filter(|len| *len <= max)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "CAD 绘图整数列表超过限额"))?;
    let mut values = r.read_region(bytes)?;
    let values = (0..count)
        .map(|_| values.read_u32_le())
        .collect::<io::Result<_>>()?;
    Ok(CadDrawingObject10_1 {
        paths,
        transform_enabled,
        flags,
        base,
        destination,
        transform_parameters,
        values,
    })
}

fn read_drawing_slot_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(ObjectEntry10_1, Option<Box<CadDrawingObject10_1>>)> {
    let entry = read_object_entry_10_1(reader, refs)?;
    let drawing = match entry {
        ObjectEntry10_1::Null
        | ObjectEntry10_1::BackReference {
            class_id: CAD_DRAWING_OBJECT_CLASS_ID,
            ..
        } => None,
        ObjectEntry10_1::NewObject {
            class_id: CAD_DRAWING_OBJECT_CLASS_ID,
            ..
        } => Some(Box::new(read_drawing_10_1(reader, max)?)),
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("CAD 绘图槽含未知对象：{entry:?}"),
            ));
        }
    };
    Ok((entry, drawing))
}

/// CadLayer.dll 0x6cb07b10 与 0x6cb08340 的持久化布局。
pub(crate) fn read_cad_feature_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<CadFeatureLayer10_1> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()?;
    if !(1..=4).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("CadFeatureLayer 不支持版本 {version}"),
        ));
    }

    // IObjectStream +0x48 读到调用方预先创建的对象，键不进入读表。
    let feature_header = read_header_10_1(&mut r)?;
    let feature = match feature_header {
        ObjectHeader::Null => None,
        ObjectHeader::Object {
            class_id: FEATURE_LAYER_CLASS_ID,
            key,
        } => {
            // 写侧对已写过的对象只写回引头，不能按已有对象加载体。
            if refs.contains(key) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "CAD 聚合要素层使用了已有对象键",
                ));
            }
            let body = read_feature_layer_prefix_10_1(&mut r, max, &mut refs)?;
            if body.stop != FeatureLayerStop10_1::Successor(SuccessorStop10_1::TailCompleted) {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!("CAD 聚合要素层未读完：{:?}", body.stop),
                ));
            }
            Some(Box::new(body))
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("CAD 聚合要素层对象头不匹配：{feature_header:?}"),
            ));
        }
    };
    let (drawing_entry, drawing) = read_drawing_slot_10_1(&mut r, max, &mut refs)?;
    let layer_values = if version >= 2 {
        // 按写入计数消费全部 short。ArcMap 加载时因数据源可达性可少读。
        let count = (r.read_u32_le()? as i32).max(0) as usize;
        let bytes = count
            .checked_mul(2)
            .filter(|len| *len <= max)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "CAD 图层 short 列表超过限额"))?;
        let mut values = r.read_region(bytes)?;
        (0..count)
            .map(|_| values.read_u16_le())
            .collect::<io::Result<_>>()?
    } else {
        Vec::new()
    };
    let body = CadFeatureLayer10_1 {
        version,
        feature_header,
        feature,
        drawing_entry,
        drawing,
        layer_values,
        georeference: (version >= 3)
            .then(|| read_string_10_1(&mut r, max))
            .transpose()?,
        legacy_georeference_flag: (version == 3).then(|| r.read_u16_le()).transpose()?,
        georeference_parameters: (version == 4)
            .then(|| -> io::Result<_> { Ok([r.read_f64_le()?, r.read_f64_le()?]) })
            .transpose()?,
    };
    *reader = r;
    *table = refs;
    Ok(body)
}

/// 读取经类 ID 确认的 CadLayer。图层对象头的登记由调用者完成。
pub(crate) fn read_cad_layer_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<CadLayer10_1> {
    let mut r = reader.clone();
    let mut refs = table.clone();
    let version = r.read_u16_le()?;
    if !(1..=8).contains(&version) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("CadLayer 不支持版本 {version}"),
        ));
    }
    let name = read_string_10_1(&mut r, max)?;
    let shorts = [r.read_u16_le()?, r.read_u16_le()?, r.read_u16_le()?];
    let scales = [r.read_f64_le()?, r.read_f64_le()?];
    let spatial_reference = read_spatial_reference_entry_10_1(&mut r, max, &mut refs)?;
    let additional_shorts = (version >= 2)
        .then(|| -> io::Result<_> {
            Ok([
                r.read_u16_le()?,
                r.read_u16_le()?,
                r.read_u16_le()?,
                r.read_u16_le()?,
            ])
        })
        .transpose()?;
    let drawing_name = if version >= 2 {
        let mut stop = None;
        let slot = read_dataset_name_slot_10_1(&mut r, max, &mut refs, &mut stop)?;
        if let Some(reason) = stop {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("CadLayer 的 IName 槽未读完：{reason:?}"),
            ));
        }
        Some(slot)
    } else {
        None
    };
    // SaveObject 写出带键的对象槽。数据源打开失败时原生按同一布局读入，
    // 打开成功时则将流中的对象体加载到现有的 CadDrawingObject。
    let (drawing_entry, drawing) = read_drawing_slot_10_1(&mut r, max, &mut refs)?;
    let body = CadLayer10_1 {
        version,
        name,
        shorts,
        scales,
        spatial_reference,
        additional_shorts,
        drawing_name,
        drawing_entry,
        drawing,
        extensions: if version >= 3 {
            read_layer_extensions_10_1(&mut r, max, &mut refs)?
        } else {
            Vec::new()
        },
        path: (version >= 4)
            .then(|| read_string_10_1(&mut r, max))
            .transpose()?,
        value: (version >= 5).then(|| r.read_f64_le()).transpose()?,
        flags: (version >= 6).then(|| r.read_u32_le()).transpose()?,
        georeference: (version >= 7)
            .then(|| read_string_10_1(&mut r, max))
            .transpose()?,
        legacy_georeference_flag: (version == 7).then(|| r.read_u16_le()).transpose()?,
        georeference_parameters: (version == 8)
            .then(|| -> io::Result<_> { Ok([r.read_f64_le()?, r.read_f64_le()?]) })
            .transpose()?,
    };
    *reader = r;
    *table = refs;
    Ok(body)
}
