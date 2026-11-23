//! 标注表达式引擎体：构造数据只验证读取器契约。

use mxd2qgz::layers::annotate_props::{
    ANNOTATION_JSCRIPT_ENGINE_CLASS_ID, ANNOTATION_PYTHON_ENGINE_CLASS_ID,
    ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID, AnnotationExpressionEngineKind10_1,
    ExpressionEngineSlot10_1, HATCH_JSCRIPT_ENGINE_CLASS_ID, HATCH_PYTHON_ENGINE_CLASS_ID,
    HATCH_VBSCRIPT_ENGINE_CLASS_ID, HOT_LINK_JSCRIPT_ENGINE_CLASS_ID,
    HOT_LINK_PYTHON_ENGINE_CLASS_ID, HOT_LINK_VBSCRIPT_ENGINE_CLASS_ID, LelpStop10_1,
    read_annotation_expression_engine_body_10_1, read_display_expression_props_prefix_10_1,
    read_label_engine_props_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn shared_body_preserves_class_and_version() {
    for (class_id, kind) in [
        (
            ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Vbscript,
        ),
        (
            ANNOTATION_JSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Jscript,
        ),
        (
            ANNOTATION_PYTHON_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Python,
        ),
    ] {
        for (bytes, version, short) in [
            (vec![0, 0], 0, None),
            (vec![1, 0], 1, None),
            (vec![2, 0, 0xff, 0xff], 2, Some(-1)),
        ] {
            let mut reader = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            let parsed = read_annotation_expression_engine_body_10_1(
                &mut reader,
                1024,
                &mut table,
                71,
                class_id,
            )
            .unwrap();
            assert_eq!(parsed.kind, kind);
            assert_eq!(
                (parsed.body.key, parsed.body.version, parsed.body.short),
                (71, version, short)
            );
            assert_eq!(reader.position(), bytes.len());
            assert!(table.is_empty());
        }
    }
}

fn empty_string(data: &mut Vec<u8>) {
    data.extend(2_u32.to_le_bytes());
    data.extend([0; 2]);
}

fn lelp_before_engine() -> Vec<u8> {
    let mut data = 7_u16.to_le_bytes().to_vec();
    empty_string(&mut data);
    data.extend([0; 2 + 8 + 4 + 2 * 3 + 4 + 8 * 2 + 2]);
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 32]);
    data
}

fn engine_entry(data: &mut Vec<u8>, class_id: [u8; 16], body: &[u8]) {
    data.extend(class_id);
    data.extend(71_u32.to_le_bytes());
    data.extend(body);
}

#[test]
fn both_host_slots_continue_for_new_engines() {
    let with_short = &[2, 0, 0xff, 0xff][..];
    let version_only = &[1, 0][..];
    for (class_id, kind, body) in [
        (
            ANNOTATION_JSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Jscript,
            with_short,
        ),
        (
            ANNOTATION_PYTHON_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Python,
            with_short,
        ),
        (
            HOT_LINK_VBSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::HotLinkVbscript,
            with_short,
        ),
        (
            HOT_LINK_JSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::HotLinkJscript,
            with_short,
        ),
        (
            HOT_LINK_PYTHON_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::HotLinkPython,
            with_short,
        ),
        (
            HATCH_VBSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::HatchVbscript,
            version_only,
        ),
        (
            HATCH_JSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::HatchJscript,
            version_only,
        ),
        (
            HATCH_PYTHON_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::HatchPython,
            version_only,
        ),
    ] {
        let expected_short = (body == with_short).then_some(-1);
        let mut lelp = lelp_before_engine();
        engine_entry(&mut lelp, class_id, body);
        let engine_end = lelp.len();
        lelp.extend([0; 16]);
        lelp.extend(12.5_f64.to_le_bytes());
        lelp.extend(4.5_f64.to_le_bytes());
        lelp.extend(17_i32.to_le_bytes());
        lelp.extend(23_i32.to_le_bytes());
        lelp.extend([0; 16]);
        lelp.extend((-1_i16).to_le_bytes());
        lelp.extend(29_i32.to_le_bytes());
        let mut reader = Reader::new(&lelp);
        let mut table = RefTable10_1::new();
        let result = read_label_engine_props_body_10_1(&mut reader, 1024, &mut table, 1).unwrap();
        assert!(
            matches!(result.engine, ExpressionEngineSlot10_1::OtherEngine(ref e) if e.kind == kind && e.body.key == 71 && e.body.short == expected_short)
        );
        assert_eq!(result.stop, None);
        assert_eq!(result.scale_ratio, Some(12.5));
        assert_eq!(result.reference_scale, Some(4.5));
        assert_eq!(result.units, Some(17));
        assert_eq!(result.int32_2c, Some(23));
        assert_eq!(result.annotation_class_id, Some(29));
        assert_eq!(reader.position(), lelp.len());
        assert!(table.contains(71));
        for cut in engine_end - body.len()..engine_end {
            let mut reader = Reader::new(&lelp[..cut]);
            let mut table = RefTable10_1::new();
            assert!(read_label_engine_props_body_10_1(&mut reader, 1024, &mut table, 1).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!table.contains(71));
        }

        let mut dep = 1_u16.to_le_bytes().to_vec();
        empty_string(&mut dep);
        dep.extend((-1_i16).to_le_bytes());
        engine_entry(&mut dep, class_id, body);
        let end = dep.len();
        dep.extend([0x5a, 0xa5]);
        let mut reader = Reader::new(&dep);
        let mut table = RefTable10_1::new();
        let result =
            read_display_expression_props_prefix_10_1(&mut reader, 1024, &mut table, 2).unwrap();
        assert!(
            matches!(result.engine, ExpressionEngineSlot10_1::OtherEngine(ref e) if e.kind == kind && e.body.key == 71 && e.body.short == expected_short)
        );
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(2).unwrap(), [0x5a, 0xa5]);
        assert!(table.contains(71));
        for cut in end - body.len()..end {
            let mut reader = Reader::new(&dep[..cut]);
            let mut table = RefTable10_1::new();
            assert!(
                read_display_expression_props_prefix_10_1(&mut reader, 1024, &mut table, 2)
                    .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(!table.contains(71));
        }
    }
}

#[test]
fn both_host_slots_rollback_unknown_classes() {
    let unknown = [0x42; 16];
    let mut lelp = lelp_before_engine();
    let slot = lelp.len();
    engine_entry(&mut lelp, unknown, &[2, 0, 0xff, 0xff]);
    let mut reader = Reader::new(&lelp);
    let mut table = RefTable10_1::new();
    let result = read_label_engine_props_body_10_1(&mut reader, 1024, &mut table, 1).unwrap();
    assert_eq!(
        result.stop,
        Some(LelpStop10_1::ExpressionSlotUnsupported {
            class_id: unknown,
            key: 71
        })
    );
    assert_eq!(reader.position(), slot);
    assert!(!table.contains(71));

    let mut dep = 1_u16.to_le_bytes().to_vec();
    empty_string(&mut dep);
    dep.extend(0_i16.to_le_bytes());
    let slot = dep.len();
    engine_entry(&mut dep, unknown, &[2, 0, 0xff, 0xff]);
    let mut reader = Reader::new(&dep);
    let mut table = RefTable10_1::new();
    let result =
        read_display_expression_props_prefix_10_1(&mut reader, 1024, &mut table, 2).unwrap();
    assert!(
        matches!(result.engine, ExpressionEngineSlot10_1::Unsupported { class_id, key: 71 } if class_id == unknown)
    );
    assert_eq!(reader.position(), slot);
    assert!(!table.contains(71));
}

#[test]
fn hatch_engines_reject_version_two() {
    for class_id in [
        HATCH_VBSCRIPT_ENGINE_CLASS_ID,
        HATCH_JSCRIPT_ENGINE_CLASS_ID,
        HATCH_PYTHON_ENGINE_CLASS_ID,
    ] {
        let mut reader = Reader::new(&[2, 0, 0xff, 0xff]);
        let mut table = RefTable10_1::new();
        assert_eq!(
            read_annotation_expression_engine_body_10_1(
                &mut reader,
                1024,
                &mut table,
                71,
                class_id
            )
            .unwrap_err()
            .kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
}

#[test]
fn invalid_input_does_not_advance() {
    for class_id in [
        ANNOTATION_JSCRIPT_ENGINE_CLASS_ID,
        ANNOTATION_PYTHON_ENGINE_CLASS_ID,
    ] {
        for bytes in [
            &[][..],
            &[2][..],
            &[2, 0][..],
            &[2, 0, 1][..],
            &[3, 0][..],
            &[255, 255][..],
        ] {
            let mut reader = Reader::new(bytes);
            let mut table = RefTable10_1::new();
            assert!(
                read_annotation_expression_engine_body_10_1(
                    &mut reader,
                    1024,
                    &mut table,
                    71,
                    class_id
                )
                .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(table.is_empty());
        }
    }
    let mut reader = Reader::new(&[2, 0, 1, 0]);
    let mut table = RefTable10_1::new();
    assert!(
        read_annotation_expression_engine_body_10_1(&mut reader, 1024, &mut table, 71, [0x42; 16])
            .is_err()
    );
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}

fn graphics_3d_extensions(data: &mut Vec<u8>) {
    use mxd2qgz::layers::annotate_props::EDIT_TEMPLATE_MANAGER_CLASS_ID;

    data.extend(2_u32.to_le_bytes());
    data.extend(0_u64.to_le_bytes());
    data.extend([0x78; 16]);
    data.extend(7_u64.to_le_bytes());
    data.extend(EDIT_TEMPLATE_MANAGER_CLASS_ID);
    data.extend([2, 0, 0, 0, 0, 0, 1]);
}

fn graphics_3d_body(
    version: u16,
    globe: bool,
    associated_layer: Option<([u8; 16], &[u8])>,
) -> Vec<u8> {
    use mxd2qgz::layers::graphics_3d::{GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID};
    use mxd2qgz::objects::element_body::MARKER_ELEMENT_CLASS_ID;
    use mxd2qgz::objects::envelope::ENVELOPE_CLASS_ID;
    use mxd2qgz::objects::geometry::POINT_CLASS_ID;

    let mut data = version.to_le_bytes().to_vec();
    empty_string(&mut data);
    data.extend([1, 0, 2, 0]);
    if !globe {
        data.extend([3, 0, 4, 0]);
    }
    if let Some((class_id, body)) = associated_layer {
        data.extend(class_id);
        data.extend(72_u32.to_le_bytes());
        data.extend(body);
    } else {
        data.extend(if globe {
            GLOBE_GRAPHICS_LAYER_CLASS_ID
        } else {
            GRAPHICS_LAYER_3D_CLASS_ID
        });
        data.extend(71_u32.to_le_bytes());
    }

    data.extend(ENVELOPE_CLASS_ID);
    data.extend(73_u32.to_le_bytes());
    data.extend(1_u16.to_le_bytes());
    for value in [1.0_f64, 2.0, 3.0, 4.0] {
        data.extend(value.to_le_bytes());
    }
    data.extend(2_u32.to_le_bytes());
    data.extend(MARKER_ELEMENT_CLASS_ID);
    data.extend(74_u32.to_le_bytes());
    data.extend(0_u16.to_le_bytes());
    data.extend(4_u16.to_le_bytes());
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 4 + 32]);

    for index in 0..2 {
        if index == 1 {
            // 第二个元素回引第一个，仍有独立的属性体。
            data.extend(MARKER_ELEMENT_CLASS_ID);
            data.extend(74_u32.to_le_bytes());
        }
        if globe && version >= 2 {
            let size = match version {
                2 => 215,
                3 | 4 => 216,
                _ => 258,
            };
            data.extend(vec![0x55 + 0x11 * index; size]);
            for geometry_version in 6..=version {
                if geometry_version == 6 {
                    data.extend(POINT_CLASS_ID);
                    data.extend(75_u32.to_le_bytes());
                    if index == 0 {
                        data.extend(1_u16.to_le_bytes());
                        data.extend(4_u32.to_le_bytes());
                        data.extend(0_u32.to_le_bytes());
                    }
                } else {
                    data.extend([0; 16]);
                }
            }
        }
    }

    if globe {
        graphics_3d_extensions(&mut data);
        data.extend([11, 0, 12, 0]);
        if version >= 4 {
            data.extend(ENVELOPE_CLASS_ID);
            data.extend(73_u32.to_le_bytes());
            data.extend([0; 16]);
        }
    } else {
        if version >= 2 {
            data.extend(11_u16.to_le_bytes());
        }
        if version >= 3 {
            data.extend([0; 16]);
        }
        if version >= 4 {
            graphics_3d_extensions(&mut data);
        }
        if version >= 5 {
            data.extend(12_u16.to_le_bytes());
        }
    }
    data
}

fn lelp_before_container() -> Vec<u8> {
    let mut data = lelp_before_engine();
    data.extend([0; 32 + 8 * 2 + 4 * 2]);
    data
}

#[test]
fn graphics_containers_continue_for_all_versions() {
    use mxd2qgz::layers::graphics_3d::{
        GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID, Graphics3dTail10_1,
    };
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for (class_id, globe, max_version) in [
        (GRAPHICS_LAYER_3D_CLASS_ID, false, 5),
        (GLOBE_GRAPHICS_LAYER_CLASS_ID, true, 7),
    ] {
        for version in 1..=max_version {
            let mut data = lelp_before_container();
            let start = data.len();
            data.extend(class_id);
            data.extend(71_u32.to_le_bytes());
            data.extend(graphics_3d_body(version, globe, None));
            let end = data.len();
            data.extend_from_slice(&[17, 0, 33, 0, 0, 0]);
            let mut reader = Reader::new(&data);
            let mut table = RefTable10_1::new();
            let parsed =
                read_label_engine_props_body_10_1(&mut reader, 4096, &mut table, 1).unwrap();
            assert_eq!(parsed.stop, None);
            assert_eq!(parsed.short_46, Some(17));
            assert_eq!(parsed.annotation_class_id, Some(33));
            assert_eq!(reader.position(), data.len());
            let Some(mxd2qgz::layers::annotate_props::LelpContainerBody10_1::Layer(container)) =
                parsed.container_body
            else {
                panic!("应读取图形图层");
            };
            let mxd2qgz::layers::layer_list::LayerBodyPrefix10_1::Graphics3d(body) = *container
            else {
                panic!("应读取三维容器");
            };
            assert_eq!(body.version, version);
            assert_eq!(
                body.shorts,
                if globe { vec![1, 2] } else { vec![1, 2, 3, 4] }
            );
            assert!(matches!(
                body.associated_layer,
                ObjectEntry10_1::BackReference { key: 71, .. }
            ));
            assert_eq!(
                body.envelope.body.as_ref().unwrap().coords,
                [1.0, 2.0, 3.0, 4.0]
            );
            assert!(body.elements[0].element.body.is_some());
            assert!(matches!(
                body.elements[1].element.entry,
                ObjectEntry10_1::BackReference { key: 74, .. }
            ));
            if globe {
                if version >= 2 {
                    let props = body.elements[1].globe_properties.as_ref().unwrap();
                    assert_eq!(props.geometries.len(), version.saturating_sub(5) as usize);
                    assert!(props.raw.iter().all(|&n| n == 0x66));
                    if version >= 6 {
                        assert!(matches!(
                            props.geometries[0].entry,
                            ObjectEntry10_1::BackReference { key: 75, .. }
                        ));
                        assert!(
                            body.elements[0]
                                .globe_properties
                                .as_ref()
                                .unwrap()
                                .geometries[0]
                                .body
                                .is_some()
                        );
                        assert!(table.contains(75));
                    }
                }
                assert!(matches!(
                    body.tail,
                    Graphics3dTail10_1::Globe {
                        shorts: [11, 12],
                        ..
                    }
                ));
            } else {
                assert!(matches!(
                    &body.tail,
                    Graphics3dTail10_1::Scene { version_2_short, version_5_short, .. }
                        if *version_2_short == (version >= 2).then_some(11)
                        && *version_5_short == (version == 5).then_some(12)
                ));
            }
            let extensions = match &body.tail {
                Graphics3dTail10_1::Scene { extensions, .. } => extensions.as_deref(),
                Graphics3dTail10_1::Globe { extensions, .. } => Some(extensions.as_slice()),
            };
            assert_eq!(extensions.is_some(), globe || version >= 4);
            if let Some(extensions) = extensions {
                assert_eq!(extensions.len(), 2);
                assert!(extensions[0].template_manager.is_none());
                let manager = extensions[1].template_manager.as_ref().unwrap();
                assert_eq!(manager.persisted_flag_is_set(), Some(true));
                assert!(!table.contains(0));
            }
            assert!(table.contains(71) && table.contains(73) && table.contains(74));
            for cut in start..end {
                let mut reader = Reader::new(&data[..cut]);
                let mut table = RefTable10_1::new();
                assert!(
                    read_label_engine_props_body_10_1(&mut reader, 4096, &mut table, 1).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(table.is_empty());
            }
        }
    }
}

#[test]
fn graphics_container_with_unknown_associated_layer_stops_before_slot() {
    use mxd2qgz::layers::graphics_3d::GLOBE_GRAPHICS_LAYER_CLASS_ID;

    let mut data = lelp_before_container();
    let offset = data.len();
    data.extend(GLOBE_GRAPHICS_LAYER_CLASS_ID);
    data.extend(71_u32.to_le_bytes());
    data.extend(graphics_3d_body(7, true, Some(([0x80; 16], &[]))));
    let mut reader = Reader::new(&data);
    let mut table = RefTable10_1::new();
    table.register(10);
    let parsed = read_label_engine_props_body_10_1(&mut reader, 4096, &mut table, 1).unwrap();
    assert!(matches!(
        parsed.stop,
        Some(LelpStop10_1::ContainerBodyUnsupported { key: 71, .. })
    ));
    assert_eq!(reader.position(), offset);
    assert!(table.contains(10));
    assert_eq!(table.len(), 1);
}

fn composite_body(nested: &[u8], key: u32) -> Vec<u8> {
    use mxd2qgz::layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID;

    let mut data = vec![0; 2 + 4];
    empty_string(&mut data);
    data.extend([0; 6 + 16 + 12]);
    data.extend(nested);
    data.extend([0; 4 + 4]);
    data.extend(COMPOSITE_GRAPHICS_LAYER_CLASS_ID);
    data.extend(key.to_le_bytes());
    data
}

#[test]
fn accepts_composite_container_and_associated_layer_new_object() {
    use mxd2qgz::layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID;
    use mxd2qgz::layers::graphics_3d::GRAPHICS_LAYER_3D_CLASS_ID;
    use mxd2qgz::layers::layer_list::LayerBodyPrefix10_1;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let class = COMPOSITE_GRAPHICS_LAYER_CLASS_ID;
    for (container_class, bytes) in [
        (class, composite_body(&[0; 16], 71)),
        (
            GRAPHICS_LAYER_3D_CLASS_ID,
            graphics_3d_body(5, false, Some((class, &composite_body(&[0; 16], 72)))),
        ),
    ] {
        let mut data = lelp_before_container();
        data.extend(container_class);
        data.extend(71_u32.to_le_bytes());
        data.extend(bytes);
        data.extend([17, 0, 33, 0, 0, 0]);
        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        let label = read_label_engine_props_body_10_1(&mut reader, 4096, &mut refs, 1).unwrap();
        assert!(label.stop.is_none());
        assert_eq!(reader.position(), data.len());
        assert_eq!(label.annotation_class_id, Some(33));
        let Some(mxd2qgz::layers::annotate_props::LelpContainerBody10_1::Layer(body)) =
            &label.container_body
        else {
            panic!("应读取图形图层");
        };
        let composite = match body.as_ref() {
            LayerBodyPrefix10_1::CompositeGraphicsLayer(body) => body,
            LayerBodyPrefix10_1::Graphics3d(layer) => {
                assert_eq!(
                    layer.associated_layer,
                    ObjectEntry10_1::NewObject {
                        class_id: class,
                        key: 72
                    }
                );
                assert!(refs.contains(72));
                let Some(LayerBodyPrefix10_1::CompositeGraphicsLayer(body)) =
                    layer.associated_layer_body.as_deref()
                else {
                    panic!("关联层应是 CompositeGraphicsLayer");
                };
                body
            }
            _ => panic!("应读取图形容器"),
        };
        assert_eq!(composite.sublayer_count, 0);
        assert!(refs.contains(71));
    }
}

#[test]
fn partial_associated_layer_rolls_back_through_both_hosts() {
    use mxd2qgz::layers::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID;
    use mxd2qgz::layers::graphics_3d::GRAPHICS_LAYER_3D_CLASS_ID;

    let mut unknown = vec![0xab; 16];
    unknown.extend(76_u32.to_le_bytes());
    let mut data = lelp_before_container();
    let start = data.len();
    data.extend(GRAPHICS_LAYER_3D_CLASS_ID);
    data.extend(71_u32.to_le_bytes());
    data.extend(graphics_3d_body(
        5,
        false,
        Some((
            COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
            &composite_body(&unknown, 72),
        )),
    ));
    data.extend([17, 0, 33, 0, 0, 0]);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    refs.register(1);
    let label = read_label_engine_props_body_10_1(&mut reader, 4096, &mut refs, 1).unwrap();
    assert!(matches!(
        label.stop,
        Some(LelpStop10_1::ContainerBodyUnsupported { key: 71, .. })
    ));
    assert!(label.container_body.is_none());
    assert_eq!(reader.position(), start);
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(1));
}

fn scene_version_one(associated_entry: &[u8]) -> Vec<u8> {
    let mut data = 1_u16.to_le_bytes().to_vec();
    empty_string(&mut data);
    data.extend([0; 8]);
    data.extend(associated_entry);
    data.extend([0; 16 + 4]);
    data
}

#[test]
fn graphics_recursion_limit_allows_back_references_and_rolls_back_new_objects() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};

    // 顶层对象的键 1 已由宿主登记，最深一层回引顶层。关联槽的新对象每层增加深度。
    let class_id = GRAPHICS_LAYER_3D_CLASS_ID;
    let entry = |key: u32| [class_id.as_slice(), &key.to_le_bytes()].concat();
    let nested = |depth: u32| {
        let mut bytes = scene_version_one(&entry(1));
        for key in (2..2 + depth).rev() {
            bytes = scene_version_one(&[entry(key), bytes].concat());
        }
        bytes
    };
    let at_limit = nested(8);
    let too_deep = nested(9);
    let mut refs = RefTable10_1::new();
    refs.register(1);
    let before = refs.clone();
    let mut reader = Reader::new(&too_deep);
    let error = read_graphics_3d_layer_10_1(&class_id, &mut reader, 4096, &mut refs).unwrap_err();
    assert!(error.to_string().contains("嵌套深度"));
    assert_eq!(reader.position(), 0);
    assert_eq!(refs, before);

    let mut reader = Reader::new(&at_limit);
    read_graphics_3d_layer_10_1(&class_id, &mut reader, 4096, &mut refs).unwrap();
    assert_eq!(reader.remaining(), 0);
    assert_eq!(refs.len(), 9);
}

#[test]
fn map_layer_list_consumes_ordinary_and_bounded_3d_layers() {
    use mxd2qgz::layers::graphics_3d::GRAPHICS_LAYER_3D_CLASS_ID;
    use mxd2qgz::layers::layer_list::{LayerBodyPrefix10_1, read_layer_list_10_1};

    let mut data = 2_u32.to_le_bytes().to_vec();
    let body = scene_version_one(&[0; 16]);
    for (flag, key) in [(1_u16, 71_u32), (2, 72)] {
        data.extend(flag.to_le_bytes());
        if flag == 2 {
            data.extend((20 + body.len() as u32).to_le_bytes());
            data.extend(0_u32.to_le_bytes());
        }
        data.extend(GRAPHICS_LAYER_3D_CLASS_ID);
        data.extend(key.to_le_bytes());
        data.extend(&body);
    }
    data.extend(1_i32.to_le_bytes());

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
    assert!(list.stop.is_none());
    assert_eq!(list.active_index, Some(1));
    assert_eq!(list.entries.len(), 2);
    assert!(
        list.entries
            .iter()
            .all(|entry| matches!(entry.parsed, Some(LayerBodyPrefix10_1::Graphics3d(_))))
    );
    assert!(refs.contains(71) && refs.contains(72));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn map_and_page_layout_objects_reuse_the_reference_session() {
    use mxd2qgz::layers::annotate_props::LelpContainerBody10_1;
    use mxd2qgz::maps::map_object::MapSuffixStop;
    use std::io::Read;

    let mut document = cfb::open("samples/测试底图.mxd").unwrap();
    let mut extract = |name| {
        let mut bytes = Vec::new();
        document
            .open_stream(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let maps = extract("/Maps");
    let page = extract("/PageLayout");
    // 用实档中按原生边界完整消费的对象体，构造两个普通 LELP 宿主。
    let mut refs = RefTable10_1::new();
    for (map_body, object) in [(true, &maps[4..0x25569]), (false, &page[..0x1c62])] {
        let mut data = lelp_before_container();
        data.extend(object);
        let end = data.len();
        data.extend([17, 0, 33, 0, 0, 0]);
        let mut reader = Reader::new(&data);
        let parsed = read_label_engine_props_body_10_1(&mut reader, 1 << 20, &mut refs, 1).unwrap();
        assert!(parsed.stop.is_none(), "{:?}", parsed.stop);
        assert_eq!(parsed.short_46, Some(17));
        assert_eq!(parsed.annotation_class_id, Some(33));
        assert_eq!(reader.position(), data.len());
        match (map_body, parsed.container_body.unwrap()) {
            (true, LelpContainerBody10_1::Map { prefix, suffix }) => {
                assert_eq!(prefix.version, 51);
                assert_eq!(suffix.stop, MapSuffixStop::Complete);
            }
            (false, LelpContainerBody10_1::PageLayout(layout)) => {
                assert_eq!(layout.consumed_elements, 7);
                assert_eq!(layout.print_settings_flag, None);
                assert_eq!(layout.tail.as_ref().unwrap().end_offset, end);
                assert!(!layout.map_frames.is_empty());
            }
            _ => panic!("应读取 Map 或 PageLayout"),
        }
    }
}

#[test]
fn unknown_map_link_rolls_back_nested_page_layout_key() {
    use std::io::Read;

    let mut page = vec![];
    cfb::open("samples/测试底图.mxd")
        .unwrap()
        .open_stream("/PageLayout")
        .unwrap()
        .read_to_end(&mut page)
        .unwrap();
    let mut data = lelp_before_container();
    let offset = data.len();
    data.extend(&page[..0x1c62]);
    let mut refs = RefTable10_1::new();
    refs.register(70);
    let mut reader = Reader::new(&data);
    let result = read_label_engine_props_body_10_1(&mut reader, 1 << 20, &mut refs, 1).unwrap();
    assert!(matches!(
        result.stop,
        Some(LelpStop10_1::ContainerBodyUnsupported { .. })
    ));
    assert_eq!(reader.position(), offset);
    assert_eq!(refs.len(), 1);
}

#[test]
fn fdo_container_and_globe_associated_object_read_reference_fdo_body() {
    use mxd2qgz::layers::annotate_props::LelpContainerBody10_1;
    use mxd2qgz::layers::fdo_graphics_layer::FDO_GRAPHICS_LAYER_CLASS_ID;
    use mxd2qgz::layers::graphics_3d::GLOBE_GRAPHICS_LAYER_CLASS_ID;
    use mxd2qgz::layers::layer_list::LayerBodyPrefix10_1;
    use std::io::Read;

    let mut maps = Vec::new();
    cfb::open("samples/测试底图.mxd")
        .unwrap()
        .open_stream("/Maps")
        .unwrap()
        .read_to_end(&mut maps)
        .unwrap();
    let reference = &maps[0x28e..0x2d22];
    for (key, class_id, bytes) in [
        (31_u32, FDO_GRAPHICS_LAYER_CLASS_ID, reference.to_vec()),
        (28, GLOBE_GRAPHICS_LAYER_CLASS_ID, {
            let mut body = 1_u16.to_le_bytes().to_vec();
            empty_string(&mut body);
            body.extend([0; 4]);
            body.extend(FDO_GRAPHICS_LAYER_CLASS_ID);
            body.extend(31_u32.to_le_bytes());
            body.extend(reference);
            body.extend([0; 16 + 4 + 4 + 4]);
            body
        }),
    ] {
        // Globe 合成对象的键应避开参考 FDO 内部使用的键。
        let mut data = lelp_before_container();
        data.extend(class_id);
        data.extend(key.to_le_bytes());
        data.extend(bytes);
        data.extend([17, 0, 33, 0, 0, 0]);
        let mut reader = Reader::new(&data);
        let mut refs = RefTable10_1::new();
        // 原始 FDO 位于 Map 图层列表前部，前方已登记 Map 和空间参考。
        let mut map = Reader::new(&maps[4..]);
        mxd2qgz::storage::object_refs::read_object_entry_10_1(&mut map, &mut refs).unwrap();
        let prefix =
            mxd2qgz::maps::map_object::read_map_prefix_object_stream_10_1(&mut map, 1 << 20, &refs)
                .unwrap();
        refs.register(prefix.spatial_reference_key.unwrap());
        let parsed = read_label_engine_props_body_10_1(&mut reader, 1 << 20, &mut refs, 1).unwrap();
        let Some(LelpContainerBody10_1::Layer(body)) = parsed.container_body else {
            panic!("{:?}", parsed.stop);
        };
        let fdo = match body.as_ref() {
            LayerBodyPrefix10_1::FdoGraphicsLayer(fdo) => fdo,
            LayerBodyPrefix10_1::Graphics3d(layer) => {
                let Some(LayerBodyPrefix10_1::FdoGraphicsLayer(fdo)) =
                    layer.associated_layer_body.as_deref()
                else {
                    panic!("应读取关联 FDO 图层");
                };
                fdo
            }
            _ => panic!("应读取 FDO 图层"),
        };
        assert!(fdo.completion().is_some());
        assert!(refs.contains(32));
        assert_eq!(reader.position(), data.len());
        assert_eq!(parsed.annotation_class_id, Some(33));
    }
}

#[test]
fn scene_graphics_reads_rectangle_and_its_back_reference() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::maps::rectangle_element::RECTANGLE_ELEMENT_CLASS_ID;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut data = scene_version_one(&[0; 16]);
    data.truncate(data.len() - 4);
    data.extend(2_u32.to_le_bytes());
    data.extend(RECTANGLE_ELEMENT_CLASS_ID);
    data.extend(74_u32.to_le_bytes());
    data.extend(1_u16.to_le_bytes());
    data.extend(1_u16.to_le_bytes());
    data.extend([0; 16]); // 矩形几何。
    data.extend(4_u16.to_le_bytes());
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 4 + 16]); // 空属性、自动变换和填充符号。
    data.extend(RECTANGLE_ELEMENT_CLASS_ID);
    data.extend(74_u32.to_le_bytes());
    let end = data.len();
    data.push(0xa5);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let layer =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    assert_eq!(layer.elements[0].rectangle.as_ref().unwrap().key, 74);
    assert!(matches!(
        layer.elements[1].element.entry,
        ObjectEntry10_1::BackReference { key: 74, .. }
    ));
    assert_eq!(reader.position(), end);

    let mut reader = Reader::new(&data[..end - 1]);
    let mut refs = RefTable10_1::new();
    assert!(
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .is_err()
    );
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn scene_graphics_reads_group_and_its_back_reference() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::maps::group_element::{GROUP_ELEMENT_CLASS_ID, GroupChild10_1};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let properties = |bytes: &mut Vec<u8>| {
        bytes.extend(0_u16.to_le_bytes());
        empty_string(bytes);
        empty_string(bytes);
        bytes.extend([0; 4 + 16]); // 旧标量和空自定义属性。
    };
    let mut data = scene_version_one(&[0; 16]);
    data.truncate(data.len() - 4);
    data.extend(2_u32.to_le_bytes());
    let entry = [GROUP_ELEMENT_CLASS_ID.as_slice(), &74_u32.to_le_bytes()].concat();
    data.extend(&entry);
    data.extend(2_u16.to_le_bytes());
    properties(&mut data);
    data.extend(1_u32.to_le_bytes());
    data.extend(&entry); // 组的已登记键。
    data.extend([0; 2 + 2 + 16]); // BaseFrameElement 和矩形几何前缀。
    properties(&mut data);
    data.extend([0; 16 + 16 + 2]); // 边框、背景及草稿模式。
    data.extend(&entry);
    let end = data.len();
    data.push(0xa5);

    let mut refs = RefTable10_1::new();
    let mut reader = Reader::new(&data);
    let layer =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    let group = layer.elements[0].group.as_ref().unwrap();
    assert!(matches!(
        group.children.as_slice(),
        [GroupChild10_1::Indirect(ObjectEntry10_1::BackReference {
            key: 74,
            ..
        })]
    ));
    assert!(matches!(
        layer.elements[1].element.entry,
        ObjectEntry10_1::BackReference { key: 74, .. }
    ));
    assert_eq!(reader.position(), end);
}

#[test]
fn graphics_3d_reads_circle_and_ellipse_bodies() {
    use mxd2qgz::layers::graphics_3d::{
        GLOBE_GRAPHICS_LAYER_CLASS_ID, GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1,
    };
    use mxd2qgz::maps::rectangle_element::{CIRCLE_ELEMENT_CLASS_ID, ELLIPSE_ELEMENT_CLASS_ID};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    for (layer_class, globe) in [
        (GRAPHICS_LAYER_3D_CLASS_ID, false),
        (GLOBE_GRAPHICS_LAYER_CLASS_ID, true),
    ] {
        for class_id in [CIRCLE_ELEMENT_CLASS_ID, ELLIPSE_ELEMENT_CLASS_ID] {
            let mut data = 1_u16.to_le_bytes().to_vec();
            empty_string(&mut data);
            data.extend(vec![0; if globe { 4 } else { 8 }]);
            data.extend([0; 32]); // 空关联图层和包络。
            data.extend(2_i32.to_le_bytes());
            let header = [class_id.as_slice(), &74_u32.to_le_bytes()].concat();
            data.extend(&header);
            let version_offset = data.len();
            data.extend((-1_i16).to_le_bytes());
            data.extend(1_u16.to_le_bytes());
            data.extend([0; 16]);
            data.extend(4_u16.to_le_bytes());
            empty_string(&mut data);
            empty_string(&mut data);
            data.extend([0; 4 + 16]); // 元素属性尾段和空填充符号。
            data.extend(&header);
            if globe {
                data.extend([0; 4 + 4]); // 空扩展列表和两个 short。
            }
            let end = data.len();
            data.push(0xa5);

            let mut reader = Reader::new(&data);
            let mut refs = RefTable10_1::new();
            let layer =
                read_graphics_3d_layer_10_1(&layer_class, &mut reader, 4096, &mut refs).unwrap();
            let shape = layer.elements[0].circle_or_ellipse.as_ref().unwrap();
            assert_eq!((shape.key, shape.version), (74, u16::MAX));
            assert_eq!(
                layer.elements[1].element.entry,
                ObjectEntry10_1::BackReference { class_id, key: 74 }
            );
            assert_eq!(reader.position(), end);

            for cut in 0..end {
                let mut reader = Reader::new(&data[..cut]);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_graphics_3d_layer_10_1(&layer_class, &mut reader, 4096, &mut refs)
                        .is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
            data[version_offset..version_offset + 2].copy_from_slice(&2_u16.to_le_bytes());
            let mut reader = Reader::new(&data);
            let mut refs = RefTable10_1::new();
            assert_eq!(
                read_graphics_3d_layer_10_1(&layer_class, &mut reader, 4096, &mut refs)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::InvalidData
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }
}

#[test]
fn graphics_3d_reads_a_standalone_frame_and_its_back_reference() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::maps::page_layout_prefix::FRAME_ELEMENT_CLASS_ID;
    use mxd2qgz::objects::element_body::ElementBody10_1;
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut data = scene_version_one(&[0; 16]);
    data.truncate(data.len() - 4);
    data.extend(2_i32.to_le_bytes());
    let header = [FRAME_ELEMENT_CLASS_ID.as_slice(), &74_u32.to_le_bytes()].concat();
    data.extend(&header);
    data.extend(2_i16.to_le_bytes()); // BaseFrameElement v2。
    data.extend([0; 2 + 16 + 2]); // RectShapeElement、空几何、ElementProperties v0。
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 4 + 16 + 16 + 16]);
    data.extend(7_u16.to_le_bytes());
    data.extend([0; 16]); // v2 的阴影槽。
    data.extend(&header);
    let end = data.len();
    data.push(0xa5);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let result =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    assert!(matches!(
        result.elements[0].element.body.as_deref(),
        Some(ElementBody10_1::FrameElement(body)) if body.version == 2 && body.draft_mode_bits == 7
    ));
    assert!(refs.contains(74));
    assert_eq!(
        result.elements[1].element.entry,
        ObjectEntry10_1::BackReference {
            class_id: FRAME_ELEMENT_CLASS_ID,
            key: 74,
        }
    );
    assert_eq!(reader.position(), end);

    for cut in 0..end {
        let mut reader = Reader::new(&data[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .is_err()
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn scene_graphics_reads_a_multipatch_element_with_a_new_geometry() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::objects::element_body::{ElementBody10_1, MULTIPATCH_ELEMENT_CLASS_ID};
    use mxd2qgz::objects::geometry::{GeometryBody10_1, MULTIPATCH_CLASS_ID};

    let mut data = scene_version_one(&[0; 16]);
    data.truncate(data.len() - 4);
    data.extend(1_i32.to_le_bytes());
    data.extend(MULTIPATCH_ELEMENT_CLASS_ID);
    data.extend(74_u32.to_le_bytes());
    data.extend((-1_i16).to_le_bytes());
    data.extend(0_u16.to_le_bytes());
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 4 + 16 + 16]); // 旧标量、空属性和空填充符号。
    data.extend(MULTIPATCH_CLASS_ID);
    data.extend(75_u32.to_le_bytes());
    data.extend(2_u16.to_le_bytes());
    data.extend(4_u32.to_le_bytes());
    data.extend([0; 4 + 16]); // 空几何记录和空间参考。
    data.extend((-17_i32).to_le_bytes());
    data.extend(7_u16.to_le_bytes());
    let end = data.len();
    data.push(0xa5);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let result =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    let Some(ElementBody10_1::MultiPatchElement(body)) = result.elements[0].element.body.as_deref()
    else {
        panic!("应读取 MultiPatchElement");
    };
    assert_eq!(
        (body.version, body.tail_long, body.tail_short),
        (u16::MAX, -17, 7)
    );
    assert!(matches!(
        body.geometry.body.as_deref(),
        Some(GeometryBody10_1::MultiPatch(_))
    ));
    assert!(refs.contains(74) && refs.contains(75));
    assert_eq!(reader.position(), end);

    for cut in 0..end {
        let mut reader = Reader::new(&data[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .is_err()
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}

#[test]
fn scene_graphics_reads_text_3d_strings_anchor_and_tail() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::objects::element_body::{
        ElementBody10_1, PointSlot10_1, TEXT_3D_ELEMENT_CLASS_ID,
    };
    use mxd2qgz::objects::geometry::{GeometryBody10_1, POINT_CLASS_ID};

    let mut data = scene_version_one(&[0; 16]);
    data.truncate(data.len() - 4);
    data.extend(1_i32.to_le_bytes());
    data.extend(TEXT_3D_ELEMENT_CLASS_ID);
    data.extend(74_u32.to_le_bytes());
    data.extend(1_i16.to_le_bytes());
    data.extend(0_u16.to_le_bytes());
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 4 + 16 + 16]); // 旧标量、空属性和空填充符号。
    data.extend(POINT_CLASS_ID);
    data.extend(75_u32.to_le_bytes());
    data.extend(2_u16.to_le_bytes());
    data.extend(20_u32.to_le_bytes());
    data.extend(1_u32.to_le_bytes());
    data.extend(2.5_f64.to_le_bytes());
    data.extend(3.5_f64.to_le_bytes());
    data.extend([0; 16]); // 空空间参考。
    data.extend((-8_i32).to_le_bytes());
    data.extend(3_u16.to_le_bytes());
    for value in ["高程", "文本"] {
        data.extend(6_u32.to_le_bytes());
        data.extend(value.encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
    }
    data.extend([4_u16, 5].into_iter().flat_map(u16::to_le_bytes));
    data.extend(POINT_CLASS_ID);
    let anchor_key_offset = data.len();
    data.extend(75_u32.to_le_bytes());
    let leading = [(-0.0_f64).to_bits(), 10.0_f64.to_bits()];
    let trailing = [20.0_f64.to_bits(), 21.0_f64.to_bits()];
    data.extend(leading.into_iter().flat_map(u64::to_le_bytes));
    data.extend([-6_i32, 7, 8, 9].into_iter().flat_map(i32::to_le_bytes));
    data.extend(trailing.into_iter().flat_map(u64::to_le_bytes));
    let end = data.len();
    data.push(0xa5);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let result =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    let Some(ElementBody10_1::Text3dElement(body)) = result.elements[0].element.body.as_deref()
    else {
        panic!("应读取 Text3DElement");
    };
    assert!(matches!(
        body.geometry.body.as_deref(),
        Some(GeometryBody10_1::Point(_))
    ));
    assert_eq!((body.raw_int, body.leading_short), (-8, 3));
    assert_eq!(body.strings, ["高程", "文本"]);
    assert_eq!(body.anchor_shorts, [4, 5]);
    assert_eq!(
        body.anchor,
        PointSlot10_1::BackReference {
            class_id: POINT_CLASS_ID,
            key: 75
        }
    );
    assert_eq!(body.leading_double_bits, leading);
    assert_eq!(body.tail_ints, [-6, 7, 8, 9]);
    assert_eq!(body.trailing_double_bits, trailing);
    assert_eq!(reader.position(), end);

    for cut in 0..end {
        let mut reader = Reader::new(&data[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .is_err()
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
    data[anchor_key_offset..anchor_key_offset + 4].copy_from_slice(&74_u32.to_le_bytes());
    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    assert_eq!(
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::Unsupported
    );
    assert_eq!(reader.position(), 0);
    assert!(refs.is_empty());
}

#[test]
fn scene_graphics_reads_a_pmf_title_through_the_text_element_layout() {
    use mxd2qgz::layers::graphics_3d::{GRAPHICS_LAYER_3D_CLASS_ID, read_graphics_3d_layer_10_1};
    use mxd2qgz::objects::element_body::{ElementBody10_1, PMF_TITLE_TEXT_ELEMENT_CLASS_ID};
    use mxd2qgz::storage::object_refs::ObjectEntry10_1;

    let mut data = scene_version_one(&[0; 16]);
    data.truncate(data.len() - 4);
    data.extend(2_i32.to_le_bytes());
    let header = [
        PMF_TITLE_TEXT_ELEMENT_CLASS_ID.as_slice(),
        &74_u32.to_le_bytes(),
    ]
    .concat();
    data.extend(&header);
    data.extend([0; 2 + 2]); // 内嵌 TextElement 与 ElementProperties 的版本。
    empty_string(&mut data);
    empty_string(&mut data);
    data.extend([0; 4 + 16]);
    data.extend(6_u32.to_le_bytes());
    data.extend("标题".encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
    data.extend([0; 4 + 16 + 16]); // 标志、符号和几何。
    data.extend(&header);
    let end = data.len();
    data.push(0xa5);

    let mut reader = Reader::new(&data);
    let mut refs = RefTable10_1::new();
    let result =
        read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
            .unwrap();
    assert!(matches!(
        result.elements[0].element.body.as_deref(),
        Some(ElementBody10_1::PmfTitleTextElement(body)) if body.text == "标题"
    ));
    assert_eq!(
        result.elements[1].element.entry,
        ObjectEntry10_1::BackReference {
            class_id: PMF_TITLE_TEXT_ELEMENT_CLASS_ID,
            key: 74
        }
    );
    assert_eq!(reader.position(), end);

    for cut in 0..end {
        let mut reader = Reader::new(&data[..cut]);
        let mut refs = RefTable10_1::new();
        assert!(
            read_graphics_3d_layer_10_1(&GRAPHICS_LAYER_3D_CLASS_ID, &mut reader, 4096, &mut refs)
                .is_err()
        );
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }
}
