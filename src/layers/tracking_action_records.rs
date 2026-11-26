//! TrackingActions.dll 10.1 的动作基类、邮件告警和数据修改记录。

use super::image_server_rules::unsupported_class;
use super::layer_list::read_complete_layer_body_10_1;
use crate::objects::dataset_name_more::read_dataset_name_slot_10_1;
use crate::objects::envelope::read_spatial_reference_body_10_1;
use crate::objects::property_set::{PROPERTY_SET_CLASS_ID, read_property_set_object_stream_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_atl_bstr_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

pub(super) const EMAIL_ALERT_ACTION_CLASS_ID: [u8; 16] = [
    0xfa, 0xa8, 0x19, 0x27, 0x90, 0xad, 0x69, 0x4e, 0x8d, 0xb2, 0x20, 0x22, 0x4d, 0x4a, 0x1a, 0x6a,
];

pub(super) const DATA_MODIFICATION_ACTION_CLASS_ID: [u8; 16] = [
    0x9f, 0x7d, 0xe2, 0xd7, 0x10, 0x78, 0x10, 0x44, 0xa9, 0xca, 0x87, 0x8f, 0x42, 0xc7, 0xa1, 0xb2,
];

pub(super) const DATA_PARTITION_ACTION_CLASS_ID: [u8; 16] = [
    0xa8, 0x5e, 0xab, 0x65, 0x59, 0xdc, 0x7f, 0x4c, 0x87, 0xa1, 0xad, 0x9f, 0xcd, 0x70, 0xea, 0x40,
];

pub(super) const DATA_SUMMARY_ACTION_CLASS_ID: [u8; 16] = [
    0x32, 0x3b, 0xaf, 0xa5, 0x06, 0x35, 0x15, 0x41, 0x9c, 0x6e, 0xe3, 0x25, 0xe5, 0xc3, 0xde, 0xcd,
];

const MESSAGE_TEMPLATE_CLASS_ID: [u8; 16] = [
    0x92, 0xef, 0xb7, 0xa7, 0xae, 0x36, 0x18, 0x43, 0xa1, 0x22, 0x43, 0x1b, 0x88, 0x92, 0xe0, 0xf3,
];

fn read_strings(reader: &mut Reader<'_>, max: usize, count: usize) -> io::Result<()> {
    for _ in 0..count {
        read_atl_bstr_10_1(reader, max)?;
    }
    Ok(())
}

fn read_count(reader: &mut Reader<'_>, max: usize, min_item_bytes: usize) -> io::Result<usize> {
    let count = (reader.read_u32_le()? as i32).max(0) as usize;
    if count > reader.remaining().min(max) / min_item_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "时态动作记录条目数超过限额",
        ));
    }
    Ok(count)
}

fn read_variant_class(reader: &mut Reader<'_>, max: usize) -> io::Result<[u8; 16]> {
    match reader.read_u16_le()? {
        13 => Ok(reader.read_bytes(16)?.try_into().unwrap()),
        0 => {
            read_atl_bstr_10_1(reader, max)?;
            Ok([0; 16])
        }
        vt => Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("时态动作记录不支持 VARTYPE {vt}"),
        )),
    }
}

fn read_layer_slot(reader: &mut Reader<'_>, max: usize, refs: &mut RefTable10_1) -> io::Result<()> {
    match read_object_entry_10_1(reader, refs)? {
        ObjectEntry10_1::Null => {}
        ObjectEntry10_1::BackReference { class_id, key } => {
            if refs.registered_class(key) != Some(class_id) {
                return Err(unsupported_class("动作条件 ILayer 回引", &class_id));
            }
        }
        ObjectEntry10_1::NewObject { class_id, .. } => {
            read_complete_layer_body_10_1(reader, class_id, max, refs)?;
        }
    }
    Ok(())
}

fn read_action_base(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<()> {
    let version = reader.read_f64_le()?;
    // 这两个写出版本均来自非零文档版本索引。
    if !matches!(version, 9.3 | 10.0) {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("时态动作 ActionBase 版本 {version}"),
        ));
    }

    read_strings(reader, max, 4)?;
    reader.read_bytes(14)?;
    read_layer_slot(reader, max, refs)?;
    reader.read_u32_le()?;

    let mut probe = reader.clone();
    if probe.read_u32_le()? == 0x000b_f894 {
        reader.read_bytes(12)?;
    }
    read_strings(reader, max, 2)?;
    reader.read_bytes(8)?;

    // 标记控制条件状态，读写两侧随后均消费 Variant。
    reader.read_u16_le()?;
    let class = read_variant_class(reader, max)?;
    match class {
        class if class == [0; 16] => {}
        PROPERTY_SET_CLASS_ID => {
            read_property_set_object_stream_10_1(reader, max, refs)?;
        }
        _ => return Err(unsupported_class("动作条件 IPropertySet", &class)),
    }

    read_strings(reader, max, 5)?;
    let count = read_count(reader, max, 4)?;
    read_strings(reader, max, count)?;
    reader.read_bytes(9)?;
    read_layer_slot(reader, max, refs)?;
    let mut stop = None;
    read_dataset_name_slot_10_1(reader, max, refs, &mut stop)?;
    if let Some(stop) = stop {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("时态动作 IName 未读完：{stop:?}"),
        ));
    }
    Ok(())
}

fn read_message_template(reader: &mut Reader<'_>, max: usize) -> io::Result<()> {
    reader.read_bytes(8)?;
    read_atl_bstr_10_1(reader, max)?;
    reader.read_u32_le()?;
    read_strings(reader, max, 4)?;
    let count = read_count(reader, max, 8)?;
    for _ in 0..count {
        read_atl_bstr_10_1(reader, max)?;
        reader.read_u32_le()?;
    }
    Ok(())
}

pub(super) fn read_email_alert_action(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<()> {
    read_action_base(reader, max, refs)?;
    reader.read_bytes(8)?;
    let class = read_variant_class(reader, max)?;
    match class {
        class if class == [0; 16] => {}
        MESSAGE_TEMPLATE_CLASS_ID => read_message_template(reader, max)?,
        _ => return Err(unsupported_class("邮件告警 IMessageTemplate", &class)),
    }
    read_strings(reader, max, 2)?;
    let count = read_count(reader, max, 4)?;
    read_strings(reader, max, count)
}

pub(super) fn read_data_modification_action(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<()> {
    read_action_base(reader, max, refs)?;
    // 写侧使用 1.0，读侧消费八字节后直接读字符串。
    reader.read_bytes(8)?;
    read_atl_bstr_10_1(reader, max)?;
    Ok(())
}

pub(super) fn read_data_partition_action(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<()> {
    read_action_base(reader, max, refs)?;
    let version = reader.read_f64_le()?;
    read_strings(reader, max, 2)?;
    reader.read_bytes(1)?;
    let pairs = read_count(reader, max, 8)?;
    for _ in 0..pairs {
        read_strings(reader, max, 2)?;
    }
    let values = read_count(reader, max, 4)?;
    reader.read_bytes(values * 4)?;
    if version >= 10.1 {
        let class_id = read_variant_class(reader, max)?;
        if class_id != [0; 16]
            && read_spatial_reference_body_10_1(
                reader,
                max,
                &ObjectEntry10_1::NewObject { class_id, key: 0 },
            )?
            .is_none()
        {
            return Err(unsupported_class("数据分区 ISpatialReference", &class_id));
        }
        read_strings(reader, max, 2)?;
    }
    Ok(())
}

pub(super) fn read_data_summary_action(
    reader: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<()> {
    read_action_base(reader, max, refs)?;
    let version = reader.read_f64_le()?;
    reader.read_bytes(16)?;
    let count = read_count(reader, max, 12)?;
    for _ in 0..count {
        read_atl_bstr_10_1(reader, max)?;
        reader.read_bytes(8)?;
    }
    read_strings(reader, max, 7)?;
    reader.read_bytes(3)?;
    if version >= 1.0 {
        read_atl_bstr_10_1(reader, max)?;
    }
    Ok(())
}
