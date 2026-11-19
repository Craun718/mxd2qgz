//! ArcMap 10.1 ImageServerLayer 的 IPersistVariant 前缀。
//! 原生读写及类证据见 docs/mxd-10.1-variant-image-server-layer.md。

use super::layer_extensions::{
    SERVER_LAYER_EXTENSION_CLASS_ID, ServerLayerExtension10_1, read_server_layer_extension,
};
use crate::layers::annotate_props::{
    EDIT_TEMPLATE_MANAGER_CLASS_ID, EditTemplateManagerBody10_1,
    read_edit_template_manager_body_10_1,
};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::objects::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use crate::objects::spatial_reference_slots::{
    GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, GeographicCoordinateSystem10_1,
    UNKNOWN_COORDINATE_SYSTEM_CLASS_ID, UnknownCoordinateSystem10_1,
    read_geographic_coordinate_system_10_1, read_unknown_coordinate_system_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::reader::Reader;
use crate::stream::variant_stream::{
    VariantObjectContext10_1, VariantObjectHeader10_1, VariantValue10_1, VariantValue10_8,
    read_value_10_1, read_value_with_objects_10_1,
};
use std::io::{self, ErrorKind};

/// ImageServerLayer 的文件字节序 CLSID。
pub const IMAGE_SERVER_LAYER_CLASS_ID: [u8; 16] = [
    0xe7, 0x13, 0x7d, 0x47, 0x68, 0x8d, 0xb6, 0x45, 0xa7, 0xfd, 0x2e, 0xf4, 0x42, 0xbc, 0xce, 0x95,
];

/// 已确认前缀；空间参考字段之后的 Variant 值未消费。
#[derive(Debug, Clone, PartialEq)]
pub struct ImageServerLayerPrefix10_1 {
    pub version: i32,
    pub name: String,
    /// ILayer.Visible 的原始 VARIANT_BOOL。
    pub visible_raw: i16,
    /// 版本之后的 19 个标量，按保存顺序保留原始值（含名称）。
    pub scalar_fields: Vec<VariantValue10_8>,
    pub extensions: Vec<VariantValue10_1<ImageServerLayerExtension10_1>>,
    /// IEnvelope 槽，保留空对象、回引和已读对象体。
    pub extent: VariantValue10_1<Envelope10_1>,
    /// ISpatialReference 槽，保留已支持的三个空间参考类。
    pub spatial_reference: VariantValue10_1<ImageServerSpatialReference10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageServerLayerExtension10_1 {
    EditTemplateManager(Box<EditTemplateManagerBody10_1>),
    ServerLayerExtension(Box<ServerLayerExtension10_1>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageServerSpatialReference10_1 {
    Projected(ProjectedCoordinateSystem10_1),
    Geographic(GeographicCoordinateSystem10_1),
    Unknown(UnknownCoordinateSystem10_1),
}

/// 从已确认的 ImageServerLayer 对象体起点读取前缀，适用标准 ObjectStream 包装。
/// 只接受 10.1 写侧会产出的版本；失败不推进游标或提交任何嵌套键。
pub fn read_image_server_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<ImageServerLayerPrefix10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let max_units = (max_bytes / 2).min(u32::MAX as usize) as u32;

    let VariantValue10_8::Raw4(version) = read_scalar(&mut trial, max_units, 3)? else {
        unreachable!("VT_I4 的载荷为四字节");
    };
    let version = i32::from_le_bytes(version);
    if !matches!(version, 8 | 13 | 18 | 22) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("未支持的 ImageServerLayer 保存版本 {version}"),
        ));
    }
    // 三处原生 Load 丢弃的值也按 Save 的 VT_BOOL 消费。
    let mut scalar_fields = Vec::with_capacity(19);
    for vt in [
        8, 8, 11, 11, 11, 11, 5, 5, 5, 8, 5, 5, 11, 11, 2, 11, 2, 11, 2,
    ] {
        scalar_fields.push(read_scalar(&mut trial, max_units, vt)?);
    }
    let VariantValue10_8::String(name) = &scalar_fields[1] else {
        unreachable!("名称是 VT_BSTR");
    };
    let name = name.clone();
    let VariantValue10_8::Raw2(visible) = scalar_fields[4] else {
        unreachable!("可见性是 VT_BOOL");
    };
    let visible_raw = i16::from_le_bytes(visible);
    let VariantValue10_8::Raw2(count) = read_scalar(&mut trial, max_units, 2)? else {
        unreachable!("VT_I2 的载荷为两字节");
    };
    let extension_count = usize::try_from(i16::from_le_bytes(count))
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "ImageServerLayer 扩展计数为负"))?;
    // 每项至少有一个 2 字节 VARTYPE 和 16 字节类标识。
    if extension_count > max_bytes / 18 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ImageServerLayer 扩展计数超过资源限额",
        ));
    }
    if extension_count > trial.remaining() / 18 {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "ImageServerLayer 扩展队列不足以容纳声明的项数",
        ));
    }
    let mut extensions = Vec::new();
    for _ in 0..extension_count {
        extensions.push(read_object(
            &mut trial,
            &mut staged,
            |body, header, refs| match header.class_id {
                SERVER_LAYER_EXTENSION_CLASS_ID => {
                    read_server_layer_extension(body, max_bytes, refs).map(|body| {
                        ImageServerLayerExtension10_1::ServerLayerExtension(Box::new(body))
                    })
                }
                EDIT_TEMPLATE_MANAGER_CLASS_ID => {
                    let parsed = read_edit_template_manager_body_10_1(
                        body,
                        max_bytes,
                        refs,
                        header.key.expect("ObjectStream 非空对象有键"),
                    )?;
                    if parsed.stop.is_some() {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "ImageServerLayer 扩展内的编辑模板尚未完整消费",
                        ));
                    }
                    Ok(ImageServerLayerExtension10_1::EditTemplateManager(
                        Box::new(parsed),
                    ))
                }
                _ => Err(super::image_server_rules::unsupported_class(
                    "ImageServerLayer 图层扩展",
                    &header.class_id,
                )),
            },
        )?);
    }
    let extent = read_object(&mut trial, &mut staged, |body, header, refs| {
        require_class(header, ENVELOPE_CLASS_ID, "IEnvelope")?;
        read_envelope_10_1(body, max_bytes, refs)
    })?;
    // 10.1 写侧在两个属性之间恒写 VT_UNKNOWN + 零 GUID。
    require_vt(&trial, 13)?;
    read_value_10_1(&mut trial, 0)?;
    let spatial_reference = read_object(&mut trial, &mut staged, |body, header, _| {
        match header.class_id {
            PROJECTED_COORDINATE_SYSTEM_CLASS_ID => {
                read_projected_coordinate_system_10_1(body, max_bytes)
                    .map(ImageServerSpatialReference10_1::Projected)
            }
            GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID => {
                read_geographic_coordinate_system_10_1(body, max_bytes)
                    .map(ImageServerSpatialReference10_1::Geographic)
            }
            UNKNOWN_COORDINATE_SYSTEM_CLASS_ID => {
                read_unknown_coordinate_system_10_1(body, max_bytes)
                    .map(ImageServerSpatialReference10_1::Unknown)
            }
            _ => Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "ImageServerLayer ISpatialReference 的类 {:?} 无已确认对象体",
                    header.class_id
                ),
            )),
        }
    })?;
    *reader = trial;
    *table = staged;
    Ok(ImageServerLayerPrefix10_1 {
        version,
        name,
        visible_raw,
        scalar_fields,
        extensions,
        extent,
        spatial_reference,
    })
}

pub(super) fn require_vt(reader: &Reader<'_>, expected: u16) -> io::Result<()> {
    let actual = reader.clone().read_u16_le()?;
    if actual != expected {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "ImageServerLayer offset 0x{:x}：期望 VARTYPE {expected}，实得 {actual}",
                reader.position()
            ),
        ));
    }
    Ok(())
}

pub(super) fn read_scalar(
    reader: &mut Reader<'_>,
    max_units: u32,
    vt: u16,
) -> io::Result<VariantValue10_8> {
    require_vt(reader, vt)?;
    let VariantValue10_1::Scalar(value) = read_value_10_1(reader, max_units)? else {
        unreachable!("已限制为标量 VARTYPE");
    };
    Ok(value)
}

pub(super) fn read_object<B>(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    read_body: impl FnOnce(&mut Reader<'_>, VariantObjectHeader10_1, &mut RefTable10_1) -> io::Result<B>,
) -> io::Result<VariantValue10_1<B>> {
    require_vt(reader, 13)?;
    read_value_with_objects_10_1(
        reader,
        0,
        VariantObjectContext10_1::ObjectStream(table),
        |reader, header, context| {
            let VariantObjectContext10_1::ObjectStream(refs) = context else {
                unreachable!("ImageServerLayer 前缀只处理 ObjectStream 宿主");
            };
            read_body(reader, header, refs)
        },
    )
}

fn require_class(
    header: VariantObjectHeader10_1,
    expected: [u8; 16],
    slot: &str,
) -> io::Result<()> {
    if header.class_id != expected {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!(
                "ImageServerLayer {slot} 的类 {:?} 无已确认对象体",
                header.class_id
            ),
        ));
    }
    Ok(())
}
