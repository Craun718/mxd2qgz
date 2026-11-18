use mxd2qgz::{
    layers::{
        basemap_layer::BASEMAP_LAYER_CLASS_ID,
        composite_graphics::{
            COMPOSITE_GRAPHICS_LAYER_CLASS_ID, read_composite_graphics_layer_prefix_10_1,
        },
        layer_list::read_layer_list_10_1,
    },
    maps::map_object::MAP_CLASS_ID,
    objects::spatial_reference::{
        PROJECTED_COORDINATE_SYSTEM_CLASS_ID, read_projected_coordinate_system_10_1,
    },
    storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1},
    stream::{native_string::read_string_10_1, reader::Reader},
};
use std::io::Read;

#[test]
#[ignore = "需要本地 10.1 安装介质 Africa.mxd"]
fn basemap_embedded_map_uses_shared_reference_table() {
    let mut doc = cfb::open("temp/arcmap101-media-mxd/Africa.mxd").unwrap();
    let mut bytes = Vec::new();
    doc.open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let mut r = Reader::new(&bytes);
    let mut refs = RefTable10_1::new();
    assert_eq!(r.read_u32_le().unwrap(), 1);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            ..
        }
    ));
    r.read_u16_le().unwrap();
    read_string_10_1(&mut r, 1_000_000).unwrap();
    read_string_10_1(&mut r, 1_000_000).unwrap();
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            key: 39
        }
    ));
    read_projected_coordinate_system_10_1(&mut r, 1_000_000).unwrap();
    assert_eq!(r.read_u32_le().unwrap(), 1);
    assert_eq!(r.read_u16_le().unwrap(), 3);
    r.read_bytes(8).unwrap();
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: BASEMAP_LAYER_CLASS_ID,
            key: 40
        }
    ));
    let mut p = r.clone();
    let mut t = refs.clone();
    assert_eq!(p.read_u16_le().unwrap(), 11);
    read_string_10_1(&mut p, 1_000_000).unwrap();
    p.read_bytes(4 + 18 + 16).unwrap();
    assert!(matches!(
        read_object_entry_10_1(&mut p, &mut t).unwrap(),
        ObjectEntry10_1::BackReference { key: 39, .. }
    ));
    assert!(matches!(
        read_object_entry_10_1(&mut p, &mut t).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: mxd2qgz::layers::basemap_layer::ARRAY_OF_I_SHADER_CLASS_ID,
            key: 41
        }
    ));
    assert_eq!(p.read_u16_le().unwrap(), 1);
    assert_eq!(p.read_u32_le().unwrap(), 0);
    p.read_bytes(2).unwrap();
    assert_eq!(p.read_u32_le().unwrap(), 6);
    assert_eq!(p.read_u16_le().unwrap(), 2);
    p.read_bytes(8).unwrap();
    assert_eq!(
        read_object_entry_10_1(&mut p, &mut t).unwrap(),
        ObjectEntry10_1::Null
    );
    assert_eq!(p.read_u32_le().unwrap(), 2);
    assert_eq!(p.position(), 0x2ae);
    assert!(matches!(
        read_object_entry_10_1(&mut p, &mut t).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID,
            key: 42
        }
    ));
    let f =
        mxd2qgz::layers::feature_layer::read_feature_layer_prefix_10_1(&mut p, 1_000_000, &mut t)
            .unwrap();
    assert!(matches!(
        f.stop,
        mxd2qgz::layers::feature_layer::FeatureLayerStop10_1::Successor(
            mxd2qgz::layers::feature_layer_tail::SuccessorStop10_1::TailCompleted
        )
    ));
    assert_eq!(p.position(), 0x145d);
    let successor = f.successor.unwrap();
    assert_eq!(successor.field_info_count, 6);
    let fields: Vec<_> = successor
        .field_infos
        .iter()
        .map(|item| match &item.slot {
            mxd2qgz::layers::feature_layer_tail::FieldInfoEntry10_1::FieldInfo(body)
                if body.stop.is_none() =>
            {
                (body.key, body.version)
            }
            _ => panic!("FieldInfo 体未完整读取"),
        })
        .collect();
    assert_eq!(
        fields,
        [(61, 4), (62, 4), (63, 4), (65, 4), (66, 4), (68, 4)]
    );
    assert!(
        matches!(successor.envelope_2, Some(mxd2qgz::layers::annotate_props::EnvelopeSlot10_1::Envelope(ref body)) if body.version == 3)
    );
    let (mut r, mut refs) = (p, t);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: MAP_CLASS_ID,
            key: 91
        }
    ));
    assert_eq!(r.read_u16_le().unwrap(), 51);
    read_string_10_1(&mut r, 1_000_000).unwrap();
    read_string_10_1(&mut r, 1_000_000).unwrap();
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::BackReference {
            class_id: PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            key: 39
        }
    ));
    assert_eq!(r.position(), 0x14b9);
    let list = read_layer_list_10_1(&mut r, 1_000_000, &mut refs).unwrap();
    assert_eq!(list.count, 0);
    assert!(list.entries.is_empty());
    assert_eq!(list.active_index, Some(-1));
    assert_eq!(r.position(), 0x14c1);
    assert!(matches!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::NewObject {
            class_id: COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
            key: 92
        }
    ));
    let body = read_composite_graphics_layer_prefix_10_1(&mut r, 1_000_000, &mut refs).unwrap();
    assert_eq!(
        body.stop,
        mxd2qgz::layers::composite_graphics::CompositeGraphicsLayerStop10_1::Completed
    );
    assert_eq!(r.position(), 0x1581);
    assert_eq!(r.read_u32_le().unwrap(), 0);
    r.read_bytes(32 + 8).unwrap();
    assert_eq!(r.position(), 0x15ad);
    assert_eq!(
        read_object_entry_10_1(&mut r, &mut refs).unwrap(),
        ObjectEntry10_1::Null
    );
    assert_eq!(r.position(), 0x15bd);
}
