//! 参考样本中的既有对象边界回归；不替代 ArcMap 运行验证。

use mxd2qgz::layers::annotate_props::{
    HOT_LINK_EXPRESSION_PROPS_CLASS_ID, read_hot_link_expression_props_body_10_1,
};
use mxd2qgz::layers::feature_id_set::{FEATURE_ID_SET_CLASS_ID, read_feature_id_set_body_10_1};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::storage::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::Read;

fn maps() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/测试底图.mxd");
    let mut compound = cfb::open(path).unwrap();
    let mut bytes = Vec::new();
    compound
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

#[test]
fn reference_sample_feature_id_set_boundary() {
    let bytes = maps();
    let mut reader = Reader::new(&bytes[0xf63..0xf7d]);
    assert_eq!(
        read_header_10_1(&mut reader).unwrap(),
        ObjectHeader::Object {
            class_id: FEATURE_ID_SET_CLASS_ID,
            key: 75,
        }
    );
    let body = read_feature_id_set_body_10_1(&mut reader, &mut RefTable10_1::new(), 75).unwrap();
    assert_eq!((body.version, body.count, body.ids.len()), (1, 0, 0));
    assert_eq!(reader.position(), 26);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reference_sample_hotlink_prefix_and_truncation() {
    let bytes = maps();
    let mut reader = Reader::new(&bytes[0x15d8..0x1606]);
    assert_eq!(
        read_header_10_1(&mut reader).unwrap(),
        ObjectHeader::Object {
            class_id: HOT_LINK_EXPRESSION_PROPS_CLASS_ID,
            key: 85,
        }
    );
    let start = reader.position();
    let body =
        read_hot_link_expression_props_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 85)
            .unwrap();
    assert_eq!(
        (body.version, body.string.as_str(), body.short),
        (1, "", -1)
    );
    assert!(matches!(
        body.engine,
        mxd2qgz::layers::annotate_props::ExpressionEngineSlot10_1::Null
    ));
    assert_eq!(reader.remaining(), 0);
    let data = &bytes[0x15d8 + start..0x1606];
    for cut in 0..data.len() {
        let mut truncated = Reader::new(&data[..cut]);
        assert!(
            read_hot_link_expression_props_body_10_1(
                &mut truncated,
                1024,
                &mut RefTable10_1::new(),
                85
            )
            .is_err()
        );
        assert_eq!(truncated.position(), 0);
    }
}

#[test]
fn reference_sample_label_objects_follow_the_fdo_associated_layer() {
    use mxd2qgz::layers::{
        annotate_props::{AlpcEntrySlot10_1, BolpSlot10_1, ExpressionEngineSlot10_1},
        fdo_graphics_layer::{AssociatedLayer10_1, ElementCollectionSlot10_1},
        feature_layer_tail::{
            AnnotatePropsSlot10_1, ExpressionPropsSlot10_1, HotLinkSlot10_1, SuccessorStop10_1,
            UidSlot10_1,
        },
        layer_list::LayerBodyPrefix10_1,
    };
    use mxd2qgz::maps::mxd_maps::parse_maps_10_1;

    let bytes = maps();
    let report = parse_maps_10_1(&bytes).unwrap();
    assert_eq!(report.confirmed_tail.unwrap().0 + 4, bytes.len());
    let layers = report.layer_list.unwrap();
    let first = &layers.entries[0];
    assert_eq!(first.offset, 0x278);
    let Some(LayerBodyPrefix10_1::FdoGraphicsLayer(fdo)) = &first.parsed else {
        panic!("应为 FDOGraphicsLayer");
    };
    let AssociatedLayer10_1::FeatureLayer(associated) = &fdo.associated_layer else {
        panic!("应为关联 FeatureLayer");
    };
    assert!(matches!(
        fdo.element_collection,
        Some(ElementCollectionSlot10_1::ElementCollection(_))
    ));

    let mut next = Reader::new(&bytes[0x1608..]);
    assert_eq!(
        read_header_10_1(&mut next).unwrap(),
        ObjectHeader::Object {
            class_id: mxd2qgz::objects::element_collection::ELEMENT_COLLECTION_CLASS_ID,
            key: 86,
        }
    );
    let tail = associated.successor.as_ref().unwrap();
    assert_eq!(tail.stop, SuccessorStop10_1::TailCompleted);
    let Some(AnnotatePropsSlot10_1::Collection(alpc)) = &tail.annotate_props else {
        panic!("应为标注集合");
    };
    assert!(alpc.stop.is_none());
    let AlpcEntrySlot10_1::LabelEngineProps(label) = &alpc.entries[0].slot else {
        panic!("应为标注引擎图层属性");
    };
    assert!(matches!(
        &label.overposter,
        BolpSlot10_1::Bolp(body) if body.key == 69 && body.version == 6 && body.stop.is_none()
    ));
    assert!(matches!(
        &tail.uid,
        Some(UidSlot10_1::Uid(body)) if body.key == 82 && body.version == 1
    ));
    assert!(matches!(
        &tail.expression_props,
        Some(ExpressionPropsSlot10_1::Display(body)) if body.key == 83
            && body.version == 1
            && matches!(&body.engine, ExpressionEngineSlot10_1::VbscriptEngine(engine) if engine.key == 84)
    ));
    assert!(matches!(
        &tail.hot_link_expression_props,
        Some(HotLinkSlot10_1::Body(body)) if body.key == 85
            && body.version == 1 && body.engine == ExpressionEngineSlot10_1::Null
    ));
    assert!(tail.short_300.is_some());
    assert_eq!(tail.short_2ca, Some(0));
}
