//! FunctionRasterDB.dll 10.1 的 RasterFunctionTemplate 与变量对象体。

use super::raster_info::{self, RasterInfo10_1, RasterInfos10_1};
use super::{
    RasterFunctionArgumentsBody10_1, RasterFunctionBody10_1, RuleSlot10_1, class_ids,
    read_arguments, read_function, signed_version, slot, supports_function_arguments,
};
use crate::layers::raster_dataset_name::{RasterDatasetNameEntryBody10_1, read_name_body};
use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertyValueObjectStream10_1, RASTER_ARGUMENT_ARRAY_CLASS_ID,
    RASTER_SCALAR_CLASS_ID, read_padded_variant_object_stream_10_1, read_variant_object_body_10_1,
};
use crate::objects::string_array::{StringArraySlot10_1, read_string_array_slot_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionTemplate10_1 {
    pub first_string: String,
    pub function: RuleSlot10_1<RasterFunctionBody10_1>,
    pub input_count: i32,
    pub inputs: Vec<RuleSlot10_1<RasterTemplateInput10_1>>,
    pub second_string: Option<String>,
    pub string_array: Option<StringArraySlot10_1>,
    pub third_string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterTemplateInput10_1 {
    Arguments(RasterFunctionArgumentsBody10_1),
    Variable(RasterFunctionVariable10_1),
    RasterName(RasterDatasetNameEntryBody10_1),
    RasterInfo(Box<RasterInfo10_1>),
    RasterInfos(RasterInfos10_1),
    NativeArgument(PropertyValueObjectStream10_1),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionVariable10_1 {
    pub version: u16,
    pub first_string: String,
    pub value: Option<RasterFunctionVariableValue10_1>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFunctionVariableValue10_1 {
    pub value: RasterVariableVariant10_1,
    pub second_string: String,
    pub flags: u16,
    pub string_array: StringArraySlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RasterVariableVariant10_1 {
    Value(PropertyValueObjectStream10_1),
    Object {
        vartype: u16,
        object: RuleSlot10_1<RasterTemplateInput10_1>,
    },
}

pub(super) fn read_template(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    body_version: i16,
    child_depth: usize,
) -> io::Result<RasterFunctionTemplate10_1> {
    let first_string = read_string_10_1(r, max)?;
    let function = slot(r, refs, |r, refs, id, _| {
        read_function(r, max, refs, id, child_depth)
    })?;
    let input_count = r.read_u32_le()? as i32;
    // 非正计数表示单项 IUnknown。正计数后的每项使用同一宿主对象表。
    let count = input_count.max(1) as usize;
    if count > r.remaining().min(max) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RasterFunctionTemplate 输入数超过限额",
        ));
    }
    let inputs = (0..count)
        .map(|_| read_input(r, max, refs, child_depth))
        .collect::<io::Result<_>>()?;
    let second_string = (body_version >= 2)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    let string_array = (body_version >= 2)
        .then(|| read_string_array_slot_10_1(r, max, refs))
        .transpose()?;
    let third_string = (body_version >= 3)
        .then(|| read_string_10_1(r, max))
        .transpose()?;
    Ok(RasterFunctionTemplate10_1 {
        first_string,
        function,
        input_count,
        inputs,
        second_string,
        string_array,
        third_string,
    })
}

fn read_variable(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    child_depth: usize,
) -> io::Result<RasterFunctionVariable10_1> {
    let version = signed_version(r, 2, "RasterFunctionVariable")?;
    let first_string = read_string_10_1(r, max)?;
    let value = (version == 2)
        .then(|| {
            Ok::<_, io::Error>(RasterFunctionVariableValue10_1 {
                value: read_variable_value(r, max, refs, child_depth)?,
                second_string: read_string_10_1(r, max)?,
                flags: r.read_u16_le()?,
                string_array: read_string_array_slot_10_1(r, max, refs)?,
            })
        })
        .transpose()?;
    Ok(RasterFunctionVariable10_1 {
        version,
        first_string,
        value,
    })
}

fn read_input(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RuleSlot10_1<RasterTemplateInput10_1>> {
    slot(r, refs, |r, refs, id, key| {
        read_input_body(r, max, refs, id, key, depth)
    })
}

pub(super) fn read_input_body(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    id: [u8; 16],
    key: u32,
    depth: usize,
) -> io::Result<RasterTemplateInput10_1> {
    match id {
        class_ids::RASTER_FUNCTION_VARIABLE => {
            let child_depth = depth.checked_sub(1).ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidData, "栅格函数变量嵌套超过限额")
            })?;
            read_variable(r, max, refs, child_depth).map(RasterTemplateInput10_1::Variable)
        }
        _ if supports_function_arguments(id) => {
            read_arguments(r, max, refs, id, depth).map(RasterTemplateInput10_1::Arguments)
        }
        RASTER_SCALAR_CLASS_ID | RASTER_ARGUMENT_ARRAY_CLASS_ID => read_variant_object_body_10_1(
            r,
            max,
            refs,
            0,
            ObjectEntry10_1::NewObject { class_id: id, key },
        )
        .map(RasterTemplateInput10_1::NativeArgument),
        raster_info::CLASS_ID => raster_info::read_body(r, max, refs, depth)
            .map(|info| RasterTemplateInput10_1::RasterInfo(Box::new(info))),
        raster_info::RASTER_INFOS_CLASS_ID => {
            raster_info::read_collection_body(r, max, refs, depth)
                .map(RasterTemplateInput10_1::RasterInfos)
        }
        _ => read_name_body(r, max, refs, id, key, depth).map(RasterTemplateInput10_1::RasterName),
    }
}

fn read_variable_value(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<RasterVariableVariant10_1> {
    let mut object_wire = r.clone();
    let vartype = object_wire.read_u16_le()?;
    if matches!(vartype, 9 | 13) {
        object_wire.read_bytes(14)?;
        if object_wire.clone().read_bytes(16)? != PROPERTY_SET_CLASS_ID {
            let object = read_input(&mut object_wire, max, refs, depth)?;
            *r = object_wire;
            return Ok(RasterVariableVariant10_1::Object { vartype, object });
        }
    }
    read_padded_variant_object_stream_10_1(r, max, refs).map(RasterVariableVariant10_1::Value)
}
