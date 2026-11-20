use mxd2qgz::layers::annotate_props::EDIT_TEMPLATE_MANAGER_CLASS_ID;
use mxd2qgz::layers::dep_tail::TIME_CLASS_ID;
use mxd2qgz::layers::feature_id_set::FID_SET_CLASS_ID;
use mxd2qgz::layers::image_server_layer::{
    IMAGE_SERVER_LAYER_CLASS_ID, read_image_server_layer_prefix_10_1,
};
use mxd2qgz::layers::image_server_layer_tail::{
    IMAGE_SERVER_NAME_CLASS_ID, ImageServerTailField10_1, ImageServerTailObject10_1,
    read_image_server_layer_body_10_1,
};
use mxd2qgz::layers::image_server_rules::{
    CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID, IDENTITY_FUNCTION_CLASS_ID, MOSAIC_RULE_CLASS_ID,
    MosaicTimeValue10_1, RENDERING_RULE_CLASS_ID, TIME_INSTANT_CLASS_ID,
};
use mxd2qgz::layers::layer_list::{LayerBodyPrefix10_1, read_layer_list_10_1};
use mxd2qgz::layers::raster_renderer::RASTER_COLORMAP_RENDERER_CLASS_ID;
use mxd2qgz::objects::domain::{DomainValues10_1, RANGE_DOMAIN_CLASS_ID};
use mxd2qgz::objects::envelope::{
    ENVELOPE_CLASS_ID, SpatialReferenceEntry10_1, read_envelope_10_1,
};
use mxd2qgz::objects::field_defs::{GEOMETRY_DEF_CLASS_ID, RASTER_DEF_CLASS_ID};
use mxd2qgz::objects::fields::{FIELD_CLASS_ID, FIELDS_CLASS_ID};
use mxd2qgz::objects::geodata_xform::{
    ADJUST_XFORM_CLASS_ID, GeodataXformBody10_1, IDENTITY_XFORM_CLASS_ID,
};
use mxd2qgz::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertyValue10_1, PropertyValueObjectStream10_1,
};
use mxd2qgz::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use mxd2qgz::stream::variant_array::ClassVariantArray10_1;
use mxd2qgz::stream::variant_stream::{
    VariantObjectContext10_1, VariantValue10_1, read_value_with_objects_10_1,
};

fn variant(out: &mut Vec<u8>, vt: u16, bytes: &[u8]) {
    out.extend(vt.to_le_bytes());
    out.extend(bytes);
}

fn bstr(out: &mut Vec<u8>, text: &str) {
    let units = text.encode_utf16().collect::<Vec<_>>();
    variant(out, 8, &(units.len() as u32).to_le_bytes());
    if !units.is_empty() {
        for x in units.into_iter().chain([0]) {
            out.extend(x.to_le_bytes());
        }
    }
}

fn object(out: &mut Vec<u8>, class_id: [u8; 16], key: u32) {
    variant(out, 13, &class_id);
    out.extend(key.to_le_bytes());
}

fn envelope() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(
        [1.0_f64, 2.0, 10.0, 20.0]
            .into_iter()
            .flat_map(f64::to_le_bytes),
    );
    bytes
}

fn layer_properties(version: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    variant(&mut bytes, 3, &version.to_le_bytes());
    bstr(&mut bytes, "");
    bstr(&mut bytes, "影像图层");
    for value in [0_i16, 0, -1, 0] {
        variant(&mut bytes, 11, &value.to_le_bytes());
    }
    for value in [7.0_f64, 8.0, 9.0] {
        variant(&mut bytes, 5, &value.to_le_bytes());
    }
    bstr(&mut bytes, "");
    for value in [11.0_f64, 12.0] {
        variant(&mut bytes, 5, &value.to_le_bytes());
    }
    variant(&mut bytes, 11, &[0, 0]);
    for n in [28_i16, 29, 30] {
        variant(&mut bytes, 11, &[0, 0]);
        variant(&mut bytes, 2, &n.to_le_bytes());
    }
    bytes
}

fn image_server_layer() -> (Vec<u8>, usize) {
    // ImageServerLayer.dll Save 0x77fece60 的 v22 前缀。
    let mut bytes = layer_properties(22);
    variant(&mut bytes, 2, &2_i16.to_le_bytes());
    object(&mut bytes, EDIT_TEMPLATE_MANAGER_CLASS_ID, 51);
    bytes.extend([2, 0, 0, 0, 0, 0, 1]); // ETM v2，空集合，尾标志。
    object(&mut bytes, EDIT_TEMPLATE_MANAGER_CLASS_ID, 51); // 回引无体。
    object(&mut bytes, ENVELOPE_CLASS_ID, 52);
    let mut saved_extent = envelope();
    saved_extent[..2].copy_from_slice(&3_u16.to_le_bytes()); // 10.1 写侧版本。
    bytes.extend(saved_extent);
    bytes.extend(0_u32.to_le_bytes()); // 无 Z/M 属性。
    bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    bytes.extend(30_u32.to_le_bytes()); // Envelope 内是普通 ObjectStream 条目，无 VARTYPE。
    variant(&mut bytes, 13, &[0; 16]); // 写侧固定为空的兼容槽。
    let sr_offset = bytes.len();
    object(&mut bytes, PROJECTED_COORDINATE_SYSTEM_CLASS_ID, 30);
    (bytes, sr_offset)
}

#[test]
fn variant_extensions_share_keys_and_end_before_extent() {
    use mxd2qgz::layers::image_server_layer::ImageServerLayerExtension10_1;
    use mxd2qgz::maps::standalone_table::TABLE_NAME_CLASS_ID;
    use mxd2qgz::objects::dataset_name_more::DatasetNameMoreSlot10_1;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let dynamic_tiled = [
        0xd9, 0x35, 0x23, 0x7e, 0x8d, 0x1e, 0x88, 0x44, 0x9e, 0x1e, 0x88, 0x85, 0x57, 0xc3, 0x1b,
        0x3d,
    ];
    let feature_service_edit = [
        0x48, 0xda, 0x2b, 0x9a, 0x26, 0xad, 0x06, 0x47, 0xbb, 0x88, 0xf4, 0x7d, 0x7b, 0x0c, 0x8f,
        0x32,
    ];
    let mut bytes = layer_properties(22);
    variant(&mut bytes, 2, &3_i16.to_le_bytes());
    object(&mut bytes, PROPERTY_SET_CLASS_ID, 71);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    object(&mut bytes, dynamic_tiled, 72);
    bytes.extend(3_u16.to_le_bytes());
    native_string(&mut bytes, "缓存");
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(71_u32.to_le_bytes());
    native_string(&mut bytes, "");
    native_string(&mut bytes, "");
    bytes.extend([0; 4 + 8 + 10]);
    bytes.extend(23_u16.to_le_bytes());
    object(&mut bytes, feature_service_edit, 73);
    bytes.extend(1_u16.to_le_bytes());
    let name_offset = bytes.len();
    bytes.extend(TABLE_NAME_CLASS_ID);
    bytes.extend(74_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    for name in ["来源", "", ""] {
        native_string(&mut bytes, name);
    }
    bytes.extend([0; 16]);
    bytes.extend(TABLE_NAME_CLASS_ID);
    bytes.extend(74_u32.to_le_bytes());
    bytes.extend(1_u32.to_le_bytes());
    bytes.extend([b'E', 0, 0, 0]); // 自由函数 WriteString 的一个码元及结束零。
    object(&mut bytes, ENVELOPE_CLASS_ID, 75);
    bytes.extend(envelope());
    null_object(&mut bytes);
    null_object(&mut bytes);
    let end = bytes.len();
    bytes.push(0x53);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let parsed = read_image_server_layer_prefix_10_1(&mut reader, 4096, &mut refs).unwrap();
    let [
        VariantValue10_1::Object {
            body: ImageServerLayerExtension10_1::PropertySet(properties),
            ..
        },
        VariantValue10_1::Object {
            body: ImageServerLayerExtension10_1::DynamicTiledLayerProperties(dynamic),
            ..
        },
        VariantValue10_1::Object {
            body: ImageServerLayerExtension10_1::FeatureServiceLayerEditExtension(edit),
            ..
        },
    ] = parsed.extensions.as_slice()
    else {
        panic!("应读取三个完整扩展");
    };
    assert_eq!(properties.count, 0);
    assert_eq!(dynamic.strings[0], "缓存");
    assert_eq!(dynamic.version3_short, Some(23));
    assert!(matches!(
        dynamic.properties.entry,
        ObjectEntry10_1::BackReference { key: 71, .. }
    ));
    assert!(matches!(
        edit.names[1],
        DatasetNameMoreSlot10_1::BackReference { key: 74, .. }
    ));
    assert_eq!(edit.text, "E");
    assert!(
        matches!(parsed.extent, VariantValue10_1::Object { body, .. } if body.coords == [1.0, 2.0, 10.0, 20.0])
    );
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0x53]);
    assert!((71..=75).all(|key| refs.contains(key)));

    let mut unknown_name = bytes.clone();
    unknown_name[name_offset] ^= 0xff;
    for input in [&unknown_name[..], &bytes[..end - 1]] {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        refs.register(70);
        assert!(read_image_server_layer_prefix_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.contains(70) && refs.len() == 1);
    }
}

#[test]
fn image_server_extension_dispatch_shares_property_keys_with_the_prefix() {
    use mxd2qgz::layers::image_server_layer::{
        ImageServerLayerExtension10_1, read_image_server_layer_prefix_10_1,
    };
    use mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let extension = [
        0xc9, 0x0e, 0xfd, 0xf7, 0x15, 0xf2, 0x7f, 0x4e, 0xb1, 0xf8, 0x94, 0xba, 0x0e, 0xb6, 0x03,
        0xb9,
    ];
    let mut bytes = layer_properties(22);
    variant(&mut bytes, 2, &2_i16.to_le_bytes());
    object(&mut bytes, extension, 71);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    object(&mut bytes, extension, 73);
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(PROPERTY_SET_CLASS_ID);
    bytes.extend(72_u32.to_le_bytes());
    for _ in 0..3 {
        null_object(&mut bytes);
    }

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&bytes[..bytes.len() - 1]);
    assert!(read_image_server_layer_prefix_10_1(&mut reader, 4096, &mut refs).is_err());
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());

    let mut reader = Reader::new(&bytes);
    let parsed = read_image_server_layer_prefix_10_1(&mut reader, 4096, &mut refs).unwrap();
    let [
        VariantValue10_1::Object {
            body: ImageServerLayerExtension10_1::ServerLayerExtension(first),
            ..
        },
        VariantValue10_1::Object {
            body: ImageServerLayerExtension10_1::ServerLayerExtension(second),
            ..
        },
    ] = parsed.extensions.as_slice()
    else {
        panic!("应读取两个服务图层扩展");
    };
    assert_eq!(first.properties.body.as_ref().unwrap().count, 0);
    assert!(matches!(
        second.properties.entry,
        ObjectEntry10_1::BackReference { key: 72, .. }
    ));
    assert_eq!(reader.remaining(), 0);
    assert!(refs.contains(71) && refs.contains(72) && refs.contains(73));
}

fn null_object(bytes: &mut Vec<u8>) {
    variant(bytes, 13, &[0; 16]);
}

fn i4(bytes: &mut Vec<u8>, value: i32) {
    variant(bytes, 3, &value.to_le_bytes());
}

fn bool_value(bytes: &mut Vec<u8>) {
    variant(bytes, 11, &[0; 2]);
}

fn native_string(bytes: &mut Vec<u8>, value: &str) {
    let text: Vec<_> = value
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    bytes.extend((text.len() as u32).to_le_bytes());
    bytes.extend(text);
}

fn complete_layer(version: i32) -> (Vec<u8>, usize, usize) {
    complete_layer_with_slots(version, None, None, None)
}

fn complete_layer_with_slots(
    version: i32,
    v14_fields: Option<&[u8]>,
    v19_rules: Option<[&[u8]; 2]>,
    v20_transform: Option<&[u8]>,
) -> (Vec<u8>, usize, usize) {
    let mut bytes = layer_properties(version);
    variant(&mut bytes, 2, &[0, 0]); // 无扩展。
    for _ in 0..3 {
        null_object(&mut bytes);
    }
    let prefix_end = bytes.len();
    bstr(&mut bytes, "兼容串");
    if version == 8 {
        bool_value(&mut bytes);
    }
    bstr(&mut bytes, "");
    let name_offset = bytes.len();
    if version == 22 {
        object(&mut bytes, IMAGE_SERVER_NAME_CLASS_ID, 60);
        bytes.extend(1_u16.to_le_bytes());
        native_string(&mut bytes, "示例服务");
        bytes.extend(PROPERTY_SET_CLASS_ID);
        bytes.extend(61_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        native_string(&mut bytes, "示例属性");
        i4(&mut bytes, 77); // 该属性仅使用两族共有的 VT_I4。
    } else {
        null_object(&mut bytes);
    }
    if version != 8 {
        null_object(&mut bytes); // IRasterRenderer。
        null_object(&mut bytes); // IEnvelope。
        bstr(&mut bytes, "");
        i4(&mut bytes, 120);
        bool_value(&mut bytes);
        bstr(&mut bytes, "别名");
        bstr(&mut bytes, "");
        i4(&mut bytes, 2); // v12 整数列表。
        i4(&mut bytes, 11);
        i4(&mut bytes, 22);
        bool_value(&mut bytes); // v13。
    }
    if version >= 18 {
        if let Some(fields) = v14_fields {
            bytes.extend(fields);
        } else {
            null_object(&mut bytes); // v14 IFields。
        }
        i4(&mut bytes, 1);
        null_object(&mut bytes); // v14 IFieldInfo。
        bstr(&mut bytes, "");
        bstr(&mut bytes, "说明");
        bool_value(&mut bytes);
        i4(&mut bytes, 1); // v15 整数列表。
        i4(&mut bytes, 31);
        null_object(&mut bytes); // IColor。
        null_object(&mut bytes); // ISymbol。
        bool_value(&mut bytes);
        bool_value(&mut bytes);
        i4(&mut bytes, 32);
        null_object(&mut bytes); // ISymbol。
        i4(&mut bytes, 1); // v16 整数列表。
        i4(&mut bytes, 41);
        bool_value(&mut bytes); // v17。
        bool_value(&mut bytes);
        null_object(&mut bytes); // v18 ITimeReference。
        for _ in 0..4 {
            bstr(&mut bytes, "");
        }
        null_object(&mut bytes); // v18 ITimeExtent。
        bool_value(&mut bytes);
        variant(&mut bytes, 5, &1.5_f64.to_le_bytes());
        i4(&mut bytes, 91);
        variant(&mut bytes, 5, &2.5_f64.to_le_bytes());
        i4(&mut bytes, 92);
    }
    if version == 22 {
        if let Some([mosaic, rendering]) = v19_rules {
            bytes.extend(mosaic);
            bytes.extend(rendering);
        } else {
            null_object(&mut bytes); // v19 IMosaicRule。
            null_object(&mut bytes); // v19 IRenderingRule。
        }
        if let Some(transform) = v20_transform {
            bytes.extend(transform);
        } else {
            null_object(&mut bytes); // v20 IGeodataXform。
        }
        i4(&mut bytes, 2);
        object(&mut bytes, PROPERTY_SET_CLASS_ID, 61); // v21 数组项回引。
        null_object(&mut bytes);
        bool_value(&mut bytes);
        bool_value(&mut bytes);
        i4(&mut bytes, 102);
    }
    (bytes, prefix_end, name_offset)
}

#[test]
fn image_layer_v20_dispatches_nonempty_geodata_xform() {
    let mut transform = Vec::new();
    object(&mut transform, ADJUST_XFORM_CLASS_ID, 66);
    transform.extend(1_i32.to_le_bytes());
    transform.extend(7_u16.to_le_bytes());
    transform.extend(8_i32.to_le_bytes());
    transform.extend([0; 16]);
    transform.extend(IDENTITY_XFORM_CLASS_ID);
    transform.extend(67_u32.to_le_bytes());
    transform.extend(2_i32.to_le_bytes());
    transform.extend([0; 16]);
    let (bytes, _, _) = complete_layer_with_slots(22, None, None, Some(&transform));
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let body = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(body.stop.is_none());
    assert!(body.tail.iter().any(|field| matches!(field,
        ImageServerTailField10_1::Object(VariantValue10_1::Object { body, .. })
        if matches!(body.as_ref(), ImageServerTailObject10_1::GeodataXform(
            GeodataXformBody10_1::Adjust { version: 1, short_c8: 7, long_a8: 8, .. }
        ))
    )));
    assert!(refs.contains(66));
    assert!(refs.contains(67));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn image_layer_fields_share_keys_and_read_the_field_default_as_an_afcore_variant() {
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut fields = Vec::new();
    object(&mut fields, FIELDS_CLASS_ID, 66);
    fields.extend(2_u16.to_le_bytes());
    fields.extend(3_u32.to_le_bytes());
    fields.extend(FIELD_CLASS_ID);
    fields.extend(67_u32.to_le_bytes());
    fields.extend(2_u16.to_le_bytes());
    fields.extend(1_u16.to_le_bytes());
    fields.extend(4_u16.to_le_bytes());
    native_string(&mut fields, "栅格字段");
    fields.push(1);
    native_string(&mut fields, "显示名");
    fields.push(0);
    native_string(&mut fields, "");
    native_string(&mut fields, "");
    fields.push(1);
    fields.extend(0x2002_u16.to_le_bytes()); // VT_ARRAY | VT_I2。
    fields.extend(2_i32.to_le_bytes());
    fields.extend((-7_i16).to_le_bytes());
    fields.extend(513_i16.to_le_bytes());
    fields.extend(9_i32.to_le_bytes()); // Raster。
    fields.extend([0; 12]);
    fields.extend([0; 5]);
    let domain_offset = fields.len();
    fields.extend(RANGE_DOMAIN_CLASS_ID);
    fields.extend(68_u32.to_le_bytes());
    fields.extend(1_u16.to_le_bytes()); // RangeDomain 版本。
    fields.extend(1_u16.to_le_bytes()); // 共享 Domain 版本。
    fields.extend([0; 12]);
    for text in ["范围", "", ""] {
        native_string(&mut fields, text);
    }
    fields.extend(0_u32.to_le_bytes());
    for bound in [10_i32, 40] {
        fields.extend(3_u16.to_le_bytes());
        fields.extend([0; 14]);
        fields.extend(bound.to_le_bytes());
    }
    fields.extend(GEOMETRY_DEF_CLASS_ID);
    fields.extend(69_u32.to_le_bytes());
    fields.extend(1_u16.to_le_bytes());
    fields.extend(4_i32.to_le_bytes());
    let grid_count_offset = fields.len();
    fields.extend(2_u32.to_le_bytes());
    fields.extend(100_f64.to_le_bytes());
    fields.extend(300_f64.to_le_bytes());
    fields.extend(0_u32.to_le_bytes());
    fields.extend([0; 16]); // GeometryDef 的空空间参考。
    fields.extend([0; 4]);
    fields.extend(RASTER_DEF_CLASS_ID);
    fields.extend(70_u32.to_le_bytes());
    fields.extend(2_u16.to_le_bytes());
    native_string(&mut fields, "栅格定义");
    fields.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    fields.extend(30_u32.to_le_bytes()); // 复用外层会话中的坐标系。
    for value in [5_u16, 6, 7] {
        fields.extend(value.to_le_bytes());
    }
    fields.extend(FIELD_CLASS_ID);
    fields.extend(67_u32.to_le_bytes()); // 同会话回引，没有 Field 体。
    fields.extend([0; 16]); // 空 Field 仍占 v2 位置。
    for scalar in [20_i32, 21, 22] {
        fields.extend(scalar.to_le_bytes());
    }

    let (bytes, _, _) = complete_layer_with_slots(18, Some(&fields), None, None);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    refs.register(30);
    let body = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(body.stop.is_none());
    let result = body
        .tail
        .iter()
        .find_map(|entry| {
            let ImageServerTailField10_1::Object(VariantValue10_1::Object { body, .. }) = entry
            else {
                return None;
            };
            match body.as_ref() {
                ImageServerTailObject10_1::Fields(value) => Some(value),
                _ => None,
            }
        })
        .unwrap();
    let field = result.fields[0].body.as_ref().unwrap();
    assert_eq!(field.strings[0], "栅格字段");
    assert_eq!(
        field.default,
        PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Array(
            ClassVariantArray10_1::Fixed {
                vartype: 2,
                element_bytes: 2,
                raw: vec![0xf9, 0xff, 1, 2],
            },
        ))
    );
    let geometry = field.geometry_def.body.as_ref().unwrap();
    assert_eq!(geometry.grid_sizes, [100.0, 300.0]);
    let raster = field.raster_def.as_ref().unwrap().body.as_ref().unwrap();
    assert_eq!(raster.string_c, "栅格定义");
    assert!(matches!(
        raster.spatial_reference,
        SpatialReferenceEntry10_1::BackReference { key: 30, .. }
    ));
    assert_eq!(raster.short_18, Some(7));
    assert_eq!(
        field.domain.body.as_ref().unwrap().values,
        DomainValues10_1::Range([
            PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(10_i32.to_le_bytes())),
            PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(40_i32.to_le_bytes()))
        ])
    );
    assert!(matches!(
        result.fields[1].entry,
        ObjectEntry10_1::BackReference { key: 67, .. }
    ));
    assert_eq!(result.fields[2].entry, ObjectEntry10_1::Null);
    assert_eq!(result.longs_v2.as_deref(), Some([20, 21, 22].as_slice()));
    assert!((66..=70).all(|key| refs.contains(key)));
    assert_eq!(reader.remaining(), 0);

    let offset = bytes
        .windows(fields.len())
        .position(|w| w == fields)
        .unwrap();
    let mut unknown = bytes.clone();
    unknown[offset + domain_offset..offset + domain_offset + 16].fill(0x91);
    let mut oversize = bytes.clone();
    oversize[offset + grid_count_offset..offset + grid_count_offset + 4]
        .copy_from_slice(&i32::MAX.to_le_bytes());
    for bad in [
        &bytes[..offset + fields.len() - 1],
        unknown.as_slice(),
        oversize.as_slice(),
    ] {
        let mut reader = Reader::new(bad);
        let mut refs = RefTable10_1::new();
        refs.register(30);
        let body = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(body.stop.unwrap().offset, offset);
        assert_eq!(reader.position(), offset);
        assert!((66..=70).all(|key| !refs.contains(key)));
        assert!(refs.contains(30));
    }
}

#[test]
fn image_layer_continues_past_complete_colormap_but_rolls_back_a_truncated_renderer() {
    fn record(out: &mut Vec<u8>, tag: i32, value: &[u8]) {
        out.extend(tag.to_le_bytes());
        out.extend((value.len() as u32).to_le_bytes());
        out.extend(value);
    }

    let (base, _, name_offset) = complete_layer(13);
    // v13 的名称与渲染器原先各占 18 字节（VARTYPE + 空 CLSID）。
    let renderer_offset = name_offset + 18;
    let mut renderer = Vec::new();
    object(&mut renderer, RASTER_COLORMAP_RENDERER_CLASS_ID, 65);
    for long in [1_i32, 0, 1, 42, 2] {
        renderer.extend(long.to_le_bytes());
    }
    record(&mut renderer, 0x00ff_00ff, &[]);
    for tag in 1..=5 {
        record(&mut renderer, tag, &0_i32.to_le_bytes());
    }
    renderer.extend(6_i32.to_le_bytes());
    renderer.extend((-1_i32).to_le_bytes());
    renderer.extend([0; 16]);
    record(&mut renderer, 7, &[0; 24]);
    renderer.extend([0xff; 8]);
    let mut bytes = base[..renderer_offset].to_vec();
    bytes.extend(&renderer);
    let renderer_end = bytes.len();
    bytes.extend(&base[renderer_offset + 18..]);
    bytes.push(0x8a);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let parsed = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(parsed.stop.is_none());
    assert!(refs.contains(65));
    assert!(matches!(&parsed.tail[3], ImageServerTailField10_1::Object(
        VariantValue10_1::Object { body, .. })
        if matches!(body.as_ref(), ImageServerTailObject10_1::RasterRenderer(r) if r.is_complete())
    ));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x8a]);

    for cut in renderer_offset..renderer_end {
        let mut reader = Reader::new(&bytes[..cut]);
        let mut refs = RefTable10_1::new();
        let parsed = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.stop.unwrap().offset, renderer_offset, "cut={cut}");
        assert_eq!(reader.position(), renderer_offset, "cut={cut}");
        assert!(!refs.contains(65), "cut={cut}");
    }
}

#[test]
fn rgb_nested_keys_commit_only_after_the_shared_tail_is_complete() {
    use std::io::Read;

    let (base, _, name_offset) = complete_layer(13);
    let renderer_offset = name_offset + 18;
    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut maps = Vec::new();
    document
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    let mut bytes = base[..renderer_offset].to_vec();
    bytes.extend(13_u16.to_le_bytes());
    let rgb_offset = bytes.len();
    // 只借用有原生读写证据的 RGB 对象体；父级 Variant 与后续字段为合成数据。
    bytes.extend(&maps[0x148c8..0x16e8c]);
    let renderer_end = bytes.len();
    bytes.extend(&base[renderer_offset + 18..]);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let parsed = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(parsed.stop.is_none());
    assert_eq!(reader.remaining(), 0);
    assert!((1548..=1552).all(|key| refs.contains(key)));

    // 各对象头之后的截断、共享体截断，以及未知直方图类均不得泄漏任何键。
    let mut bad_class = bytes.clone();
    bad_class[rgb_offset + 0x152fe - 0x148c8] ^= 0xf0;
    for bad in [
        &bytes[..rgb_offset + 0x16d0c - 0x148c8],
        &bytes[..renderer_end - 1],
        bad_class.as_slice(),
    ] {
        let mut reader = Reader::new(bad);
        let mut refs = RefTable10_1::new();
        let parsed = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(parsed.stop.unwrap().offset, renderer_offset);
        assert_eq!(reader.position(), renderer_offset);
        assert!((1548..=1552).all(|key| !refs.contains(key)));
    }
}

#[test]
fn four_native_write_versions_end_before_the_next_value() {
    for (version, count) in [(8, 4), (13, 14), (18, 43), (22, 52)] {
        let (mut bytes, _, _) = complete_layer(version);
        bytes.push(0xcc);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(body.stop, None, "v{version}");
        assert_eq!(body.tail.len(), count, "v{version}");
        assert_eq!(reader.read_bytes(1).unwrap(), [0xcc]);
        if version == 22 {
            assert_eq!(refs.len(), 2);
            assert!(matches!(
                &body.tail[2],
                ImageServerTailField10_1::Object(VariantValue10_1::Object {body, .. })
                if matches!(body.as_ref(), ImageServerTailObject10_1::Name(name)
                    if name.text == "示例服务" && name.new_properties.as_ref()
                        .is_some_and(|set| set.entries[0].key == "示例属性"))
            ));
            assert!(body.tail.iter().any(|field| matches!(
                field,
                ImageServerTailField10_1::Object(VariantValue10_1::BackReference { key: 61, .. })
            )));
        }
    }
}

#[test]
fn suffix_preserves_the_first_unsupported_or_truncated_object() {
    let (mut bytes, _, name_offset) = complete_layer(22);
    let cut_inside_properties =
        name_offset + 22 + 2 + 4 + "示例服务".encode_utf16().count() * 2 + 2 + 20;
    {
        let bytes = &bytes[..cut_inside_properties];
        let mut reader = Reader::new(bytes);
        let mut refs = RefTable10_1::new();
        let body = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(body.stop.unwrap().offset, name_offset);
        assert_eq!(reader.position(), name_offset);
        assert!(refs.is_empty());
    }
    bytes[name_offset + 2..name_offset + 18].fill(0x91);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let body = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    let stop = body.stop.unwrap();
    assert_eq!(stop.error_kind, std::io::ErrorKind::Unsupported);
    assert_eq!(reader.position(), name_offset);
    assert!(refs.is_empty());
}

#[test]
fn bounded_image_layer_uses_one_table_for_new_bodies_and_back_references() {
    let (mut body, _) = image_server_layer();
    body.extend([0xc0, 0x12]); // 后续尚未解析。
    let mut list = 1_i32.to_le_bytes().to_vec();
    list.extend(2_u16.to_le_bytes()); // ExtraSafe。
    list.extend(0_u32.to_le_bytes());
    list.extend((20_u32 + body.len() as u32).to_le_bytes());
    list.extend(IMAGE_SERVER_LAYER_CLASS_ID);
    list.extend(50_u32.to_le_bytes());
    list.extend(body);
    list.extend(0_i32.to_le_bytes()); // 活动索引。
    list.extend([0x7f]);
    let mut reader = Reader::new(&list);
    let mut refs = RefTable10_1::new();
    refs.register(30);

    let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
    let entry = &list.entries[0];
    let Some(LayerBodyPrefix10_1::ImageServerLayer(parsed)) = &entry.parsed else {
        panic!("{:?}", entry.parse_note);
    };
    assert!(parsed.stop.is_some());
    let parsed = &parsed.prefix;
    assert_eq!(parsed.name, "影像图层");
    assert_eq!(parsed.visible_raw, -1);
    assert!(matches!(
        &parsed.extensions[..],
        [VariantValue10_1::Object { body, .. }, VariantValue10_1::BackReference { key: 51, .. }]
        if matches!(body, mxd2qgz::layers::image_server_layer::ImageServerLayerExtension10_1::EditTemplateManager(manager)
            if manager.tail_byte == Some(1))
    ));
    assert!(matches!(
        &parsed.extent,
        VariantValue10_1::Object { body, header, .. }
            if header.key == Some(52) && body.coords == [1.0, 2.0, 10.0, 20.0]
            && matches!(body.spatial_reference,
                Some(mxd2qgz::objects::envelope::SpatialReferenceEntry10_1::BackReference { key: 30, .. }))
    ));
    assert!(matches!(
        parsed.spatial_reference,
        VariantValue10_1::BackReference { key: 30, .. }
    ));
    assert_eq!(entry.body.as_ref().unwrap().len, 2);
    assert_eq!(reader.read_bytes(1).unwrap(), [0x7f]);
    assert_eq!(refs.len(), 4);

    // 独立 Reader 中另一个 Variant 值使用同一会话表，不能再次尝试加载对象体。
    let mut second_stream = Vec::new();
    object(&mut second_stream, ENVELOPE_CLASS_ID, 52);
    second_stream.push(0x99);
    let mut reader = Reader::new(&second_stream);
    let second: VariantValue10_1<()> = read_value_with_objects_10_1(
        &mut reader,
        0,
        VariantObjectContext10_1::ObjectStream(&mut refs),
        |_, _, _| panic!("已登记的 Variant 对象回引没有体"),
    )
    .unwrap();
    assert!(matches!(
        second,
        VariantValue10_1::BackReference { key: 52, .. }
    ));
    assert_eq!(reader.read_bytes(1).unwrap(), [0x99]);
}

#[test]
fn failed_layer_prefix_rolls_back_keys_and_does_not_borrow_tail_bytes() {
    let (full, sr_offset) = image_server_layer();
    let mut unknown = full.clone();
    unknown[sr_offset + 2..sr_offset + 18].fill(0x9b);
    unknown[sr_offset + 18..sr_offset + 22].copy_from_slice(&53_u32.to_le_bytes());
    for bytes in (0..full.len())
        .map(|cut| &full[..cut])
        .chain([unknown.as_slice()])
    {
        let mut reader = Reader::new(bytes);
        let mut refs = RefTable10_1::new();
        refs.register(30);
        let original = refs.clone();
        assert!(read_image_server_layer_prefix_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);
    }
}

#[test]
fn plain_variant_uses_a_class_id_without_a_key() {
    let mut bytes = Vec::new();
    variant(&mut bytes, 9, &ENVELOPE_CLASS_ID);
    bytes.extend(envelope());
    bytes.push(0x41);
    let mut reader = Reader::new(&bytes);
    let value = read_value_with_objects_10_1(
        &mut reader,
        0,
        VariantObjectContext10_1::Plain,
        |body, header, context| {
            assert_eq!(header.class_id, ENVELOPE_CLASS_ID);
            assert_eq!(header.key, None);
            assert!(matches!(context, VariantObjectContext10_1::Plain));
            // Envelope v1 无需进一步读取引用表。
            read_envelope_10_1(body, 0, &mut RefTable10_1::new())
        },
    )
    .unwrap();
    assert!(
        matches!(value, VariantValue10_1::Object { value_type: 9, body, .. }
        if body.coords[0] == 1.0)
    );
    assert_eq!(reader.read_bytes(1).unwrap(), [0x41]);
}

#[test]
fn rendering_rule_reads_stretch_histograms_and_derived_arguments() {
    use mxd2qgz::layers::image_server_rules::STRETCH_FUNCTION_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let histogram = [
        0xc1, 0x27, 0x30, 0xba, 0xca, 0x49, 0x88, 0x47, 0x8b, 0x5a, 0x3a, 0x6b, 0x38, 0x7d, 0xe7,
        0x8c,
    ];
    let stretch_args = [
        0x56, 0x1e, 0x26, 0xd8, 0x8a, 0x17, 0x52, 0x4a, 0x91, 0x5b, 0x78, 0x8f, 0x93, 0x93, 0x95,
        0x1c,
    ];
    let mut rule = Vec::new();
    object(&mut rule, RENDERING_RULE_CLASS_ID, 84);
    rule.extend(1_u16.to_le_bytes());
    native_string(&mut rule, "拉伸");
    native_string(&mut rule, "");
    rule.extend(STRETCH_FUNCTION_CLASS_ID);
    rule.extend(85_u32.to_le_bytes());
    rule.extend(3_u16.to_le_bytes());
    native_string(&mut rule, "Stretch");
    native_string(&mut rule, "");
    rule.extend(3_i32.to_le_bytes());
    rule.extend(2_u16.to_le_bytes());
    let count_offset = rule.len();
    rule.extend(2_i32.to_le_bytes());
    rule.extend(histogram);
    rule.extend(86_u32.to_le_bytes());
    rule.extend(3_i32.to_le_bytes());
    rule.extend([0; 48]); // 六项统计值。
    rule.extend(1_i32.to_le_bytes());
    rule.extend(10_f64.to_le_bytes());
    rule.extend(2_f64.to_le_bytes());
    rule.extend(histogram);
    rule.extend(86_u32.to_le_bytes());
    rule.extend(stretch_args);
    rule.extend(87_u32.to_le_bytes());
    rule.extend(1_u16.to_le_bytes());
    rule.extend(PROPERTY_SET_CLASS_ID);
    rule.extend(88_u32.to_le_bytes());
    rule.extend(1_u16.to_le_bytes());
    rule.extend(0_i32.to_le_bytes());
    native_string(&mut rule, "输出");
    rule.extend([0; 16]);

    let mut mosaic = Vec::new();
    null_object(&mut mosaic);
    let (bytes, _, _) = complete_layer_with_slots(22, None, Some([&mosaic, &rule]), None);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(result.stop.is_none(), "{:?}", result.stop);
    let parsed = result
        .tail
        .iter()
        .find_map(|field| {
            let ImageServerTailField10_1::Object(VariantValue10_1::Object { body, .. }) = field
            else {
                return None;
            };
            if let ImageServerTailObject10_1::RenderingRule(rule) = body.as_ref() {
                Some(rule)
            } else {
                None
            }
        })
        .unwrap();
    let histograms = parsed
        .function
        .body
        .as_ref()
        .unwrap()
        .histograms
        .as_ref()
        .unwrap();
    assert_eq!(histograms[0].body.as_ref().unwrap().bins, [10.0]);
    assert!(matches!(
        histograms[1].entry,
        ObjectEntry10_1::BackReference { key: 86, .. }
    ));
    assert!(
        parsed
            .arguments
            .body
            .as_ref()
            .unwrap()
            .properties
            .body
            .is_some()
    );
    assert_eq!(parsed.strings[2], "输出");
    assert!((84..=88).all(|key| refs.contains(key)));
    assert_eq!(reader.remaining(), 0);

    let offset = bytes
        .windows(rule.len())
        .position(|window| window == rule)
        .unwrap();
    let mut invalid = bytes.clone();
    invalid[offset + count_offset..offset + count_offset + 4]
        .copy_from_slice(&i32::MAX.to_le_bytes());
    for input in [&invalid[..], &bytes[..offset + rule.len() - 1]] {
        let mut reader = Reader::new(input);
        let mut refs = RefTable10_1::new();
        let result = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert_eq!(result.stop.unwrap().offset, offset);
        assert!((84..=88).all(|key| !refs.contains(key)));
    }
}

#[test]
fn mosaic_and_rendering_rules_read_nested_objects_and_stop_at_an_unknown_argument() {
    let mut mosaic = Vec::new();
    object(&mut mosaic, MOSAIC_RULE_CLASS_ID, 80);
    mosaic.extend(5_u16.to_le_bytes());
    native_string(&mut mosaic, "第一串");
    native_string(&mut mosaic, "第二串");
    mosaic.extend(13_u16.to_le_bytes());
    mosaic.extend([0; 14]); // Variant(false)。
    mosaic.extend(PROPERTY_SET_CLASS_ID);
    mosaic.extend(88_u32.to_le_bytes());
    mosaic.extend(1_u16.to_le_bytes());
    mosaic.extend(0_i32.to_le_bytes());
    native_string(&mut mosaic, "第三串");
    mosaic.extend([0; 10]);
    mosaic.extend([0; 16]); // 空 Point。
    mosaic.extend(FID_SET_CLASS_ID);
    mosaic.extend(81_u32.to_le_bytes());
    mosaic.extend(1_u16.to_le_bytes());
    mosaic.extend(1_i32.to_le_bytes());
    mosaic.extend(6_i32.to_le_bytes());
    mosaic.extend(TIME_INSTANT_CLASS_ID);
    mosaic.extend(82_u32.to_le_bytes());
    mosaic.extend(1_u16.to_le_bytes());
    mosaic.extend(TIME_CLASS_ID);
    mosaic.extend(83_u32.to_le_bytes());
    mosaic.extend(1_u16.to_le_bytes());
    mosaic.extend(1_i32.to_le_bytes());
    mosaic.extend(45555_f64.to_le_bytes());
    mosaic.extend([0; 16]); // 空 TimeReference。

    let mut rendering = Vec::new();
    object(&mut rendering, RENDERING_RULE_CLASS_ID, 84);
    rendering.extend(1_u16.to_le_bytes());
    native_string(&mut rendering, "函数");
    native_string(&mut rendering, "");
    rendering.extend(IDENTITY_FUNCTION_CLASS_ID);
    rendering.extend(85_u32.to_le_bytes());
    rendering.extend(4_u16.to_le_bytes()); // BasicFunction v4：两串后才是 int32。
    native_string(&mut rendering, "Identity");
    native_string(&mut rendering, "原样输出");
    rendering.extend(3_i32.to_le_bytes());
    rendering.extend(1_u16.to_le_bytes()); // IdentityFunction 自身版本。
    let argument_offset = rendering.len();
    rendering.extend(CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID);
    rendering.extend(86_u32.to_le_bytes());
    rendering.extend(1_u16.to_le_bytes());
    rendering.extend(PROPERTY_SET_CLASS_ID);
    rendering.extend(87_u32.to_le_bytes());
    rendering.extend(1_u16.to_le_bytes());
    rendering.extend(1_u32.to_le_bytes());
    native_string(&mut rendering, "数值");
    i4(&mut rendering, 11);
    native_string(&mut rendering, "输出");
    rendering.extend([0; 16]); // 空 IRasterRenderer。
    let (bytes, _, _) = complete_layer_with_slots(22, None, Some([&mosaic, &rendering]), None);
    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let result = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(result.stop.is_none());
    assert_eq!(reader.remaining(), 0);
    assert!((80..=88).all(|key| refs.contains(key)));
    let mut rule_count = 0;
    for field in &result.tail {
        if let ImageServerTailField10_1::Object(VariantValue10_1::Object { body, .. }) = field {
            match body.as_ref() {
                ImageServerTailObject10_1::MosaicRule(rule) => {
                    assert!(matches!(
                        &rule.sort_value,
                        PropertyValueObjectStream10_1::Object { class_id: PROPERTY_SET_CLASS_ID, key: 88, body }
                            if body.version == 1 && body.entries.is_empty()
                    ));
                    assert_eq!(rule.strings[2], "第三串");
                    assert_eq!(rule.fids.as_ref().unwrap().body.as_ref().unwrap().ids, [6]);
                    assert!(matches!(
                        rule.time_value.as_ref().unwrap().body.as_deref(),
                        Some(MosaicTimeValue10_1::Instant { time, .. }) if time.body.is_some()
                    ));
                    rule_count += 1;
                }
                ImageServerTailObject10_1::RenderingRule(rule) => {
                    assert_eq!(rule.strings[2], "输出");
                    let function = rule.function.body.as_ref().unwrap();
                    assert_eq!(function.base_strings[0].as_deref(), Some("Identity"));
                    assert_eq!(function.base_long, 3);
                    let args = rule.arguments.body.as_ref().unwrap();
                    let properties = args.properties.body.as_ref().unwrap();
                    assert_eq!(properties.entries[0].key, "数值");
                    assert_eq!(
                        properties.entries[0].value,
                        PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(
                            11_i32.to_le_bytes()
                        ))
                    );
                    rule_count += 1;
                }
                _ => {}
            }
        }
    }
    assert_eq!(rule_count, 2);

    let offset = bytes
        .windows(rendering.len())
        .position(|w| w == rendering)
        .unwrap();
    let mut bad = bytes.clone();
    bad[offset + argument_offset..offset + argument_offset + 16].fill(0x91);
    for (fixture, expected) in [
        (bad.as_slice(), Some("IRasterFunctionArguments")),
        (&bytes[..offset + rendering.len() - 1], None),
    ] {
        let mut reader = Reader::new(fixture);
        let mut refs = RefTable10_1::new();
        let result = read_image_server_layer_body_10_1(&mut reader, 4096, &mut refs).unwrap();
        let stop = result.stop.unwrap();
        assert_eq!(stop.offset, offset);
        if let Some(expected) = expected {
            assert!(stop.reason.contains(expected));
        }
        assert!((80..=83).all(|key| refs.contains(key)));
        assert!((84..=87).all(|key| !refs.contains(key)));
    }
}

#[test]
fn scene_graphics_reads_image_server_layer_and_preserves_shared_keys() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let (body, _, _) = complete_layer(22);
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    native_string(&mut bytes, "Scene");
    bytes.extend([0; 8]);
    bytes.extend(IMAGE_SERVER_LAYER_CLASS_ID);
    bytes.extend(51_u32.to_le_bytes());
    let body_offset = bytes.len();
    bytes.extend(&body);
    bytes.extend([0; 16 + 4]);
    let end = bytes.len();
    bytes.push(0xcc);

    let mut reader = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    let parsed =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    assert_eq!(
        parsed.associated_layer,
        ObjectEntry10_1::NewObject {
            class_id: IMAGE_SERVER_LAYER_CLASS_ID,
            key: 51
        }
    );
    assert!(matches!(
        parsed.associated_layer_body.as_deref(),
        Some(LayerBodyPrefix10_1::ImageServerLayer(layer)) if layer.stop.is_none()
    ));
    assert!(refs.contains(51) && refs.contains(60) && refs.contains(61));
    assert_eq!(reader.position(), end);
    assert_eq!(reader.read_bytes(1).unwrap(), [0xcc]);

    // 从图层体最后一个字节之前截断，验证内部键和外部图层键都回退。
    let mut reader = Reader::new(&bytes[..body_offset + body.len() - 1]);
    let mut refs = RefTable10_1::new();
    refs.register(99);
    assert!(
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .is_err()
    );
    assert_eq!(reader.position(), 0);
    assert!(refs.contains(99));
    assert_eq!(refs.len(), 1);
}
