//! 标注表达式引擎体：构造数据只验证读取器契约。

use mxd2qgz::layers::annotate_props::{
    ANNOTATION_JSCRIPT_ENGINE_CLASS_ID, ANNOTATION_PYTHON_ENGINE_CLASS_ID,
    ANNOTATION_VBSCRIPT_ENGINE_CLASS_ID, AnnotationExpressionEngineKind10_1,
    ExpressionEngineSlot10_1, LelpStop10_1, read_annotation_expression_engine_body_10_1,
    read_display_expression_props_prefix_10_1, read_label_engine_props_body_10_1,
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

fn engine_entry(data: &mut Vec<u8>, class_id: [u8; 16]) {
    data.extend(class_id);
    data.extend(71_u32.to_le_bytes());
    data.extend([2, 0, 0xff, 0xff]);
}

#[test]
fn both_host_slots_continue_for_new_engines() {
    for (class_id, kind) in [
        (
            ANNOTATION_JSCRIPT_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Jscript,
        ),
        (
            ANNOTATION_PYTHON_ENGINE_CLASS_ID,
            AnnotationExpressionEngineKind10_1::Python,
        ),
    ] {
        let mut lelp = lelp_before_engine();
        engine_entry(&mut lelp, class_id);
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
            matches!(result.engine, ExpressionEngineSlot10_1::OtherEngine(ref e) if e.kind == kind && e.body.key == 71)
        );
        assert_eq!(result.stop, None);
        assert_eq!(result.scale_ratio, Some(12.5));
        assert_eq!(result.reference_scale, Some(4.5));
        assert_eq!(result.units, Some(17));
        assert_eq!(result.int32_2c, Some(23));
        assert_eq!(result.annotation_class_id, Some(29));
        assert_eq!(reader.position(), lelp.len());
        assert!(table.contains(71));
        for cut in engine_end - 4..engine_end {
            let mut reader = Reader::new(&lelp[..cut]);
            let mut table = RefTable10_1::new();
            assert!(read_label_engine_props_body_10_1(&mut reader, 1024, &mut table, 1).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!table.contains(71));
        }

        let mut dep = 1_u16.to_le_bytes().to_vec();
        empty_string(&mut dep);
        dep.extend((-1_i16).to_le_bytes());
        engine_entry(&mut dep, class_id);
        let end = dep.len();
        dep.extend([0x5a, 0xa5]);
        let mut reader = Reader::new(&dep);
        let mut table = RefTable10_1::new();
        let result =
            read_display_expression_props_prefix_10_1(&mut reader, 1024, &mut table, 2).unwrap();
        assert!(
            matches!(result.engine, ExpressionEngineSlot10_1::OtherEngine(ref e) if e.kind == kind && e.body.key == 71)
        );
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(2).unwrap(), [0x5a, 0xa5]);
        assert!(table.contains(71));
        for cut in end - 4..end {
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
    engine_entry(&mut lelp, unknown);
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
    engine_entry(&mut dep, unknown);
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
