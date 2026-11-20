use mxd2qgz::layers::cad_layer::{
    CAD_ANNOTATION_LAYER_CLASS_ID, CAD_DRAWING_OBJECT_CLASS_ID, CAD_FEATURE_LAYER_CLASS_ID,
    CAD_LAYER_CLASS_ID,
};
use mxd2qgz::layers::coverage_annotation_layer::COVERAGE_ANNOTATION_LAYER_CLASS_ID;
use mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
use mxd2qgz::layers::feature_layer_tail::FIELD_INFO_CLASS_ID;
use mxd2qgz::layers::layer_list::{LayerBodyPrefix10_1, read_layer_list_10_1};
use mxd2qgz::objects::dataset_name_more::CAD_DRAWING_NAME_CLASS_ID;
use mxd2qgz::objects::display_color::RGB_COLOR_CLASS_ID;
use mxd2qgz::objects::ole_font::STDFONT_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::storage::object_stream::ObjectHeader;
use mxd2qgz::stream::reader::Reader;

fn string(bytes: &mut Vec<u8>, value: &str) {
    let utf16: Vec<_> = value.encode_utf16().chain([0]).collect();
    bytes.extend((2 * utf16.len() as u32).to_le_bytes());
    bytes.extend(utf16.into_iter().flat_map(u16::to_le_bytes));
}

fn cad_body(version: u16, count: u32) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    string(&mut bytes, "CAD");
    bytes.extend([0; 6 + 16 + 16]);
    if version >= 2 {
        bytes.extend([0; 8]);
        bytes.extend(CAD_DRAWING_NAME_CLASS_ID);
        bytes.extend(21_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        for name in ["drawing", "", ""] {
            string(&mut bytes, name);
        }
        bytes.extend([0; 16]);
    }
    drawing(&mut bytes, count);
    if version >= 3 {
        bytes.extend(0_u32.to_le_bytes());
    }
    if version >= 4 {
        string(&mut bytes, "project");
    }
    if version >= 5 {
        bytes.extend(1.25_f64.to_le_bytes());
    }
    if version >= 6 {
        bytes.extend(42_u32.to_le_bytes());
    }
    if version >= 7 {
        string(&mut bytes, "georeference");
    }
    if version == 7 {
        bytes.extend(5_u16.to_le_bytes());
    }
    if version == 8 {
        bytes.extend(13.0_f64.to_le_bytes());
        bytes.extend(19.0_f64.to_le_bytes());
    }
    bytes
}

fn drawing(bytes: &mut Vec<u8>, count: u32) {
    bytes.extend(CAD_DRAWING_OBJECT_CLASS_ID);
    bytes.extend(22_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    string(bytes, "map.mxd");
    string(bytes, "drawing.dwg");
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(17_u32.to_le_bytes());
    for x in 0..10 {
        bytes.extend(f64::from(x).to_le_bytes());
    }
    bytes.extend(count.to_le_bytes());
    bytes.extend(10_u32.to_le_bytes());
    bytes.extend(11_u32.to_le_bytes());
}

fn layer_list(class_id: [u8; 16], body: &[u8], bounded: bool) -> Vec<u8> {
    let mut bytes = 1_u32.to_le_bytes().to_vec();
    if bounded {
        let length = (20 + body.len()) as u32;
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(length.to_le_bytes());
        bytes.extend(length.to_le_bytes());
    } else {
        bytes.extend(1_u16.to_le_bytes());
    }
    bytes.extend(class_id);
    bytes.extend(20_u32.to_le_bytes());
    bytes.extend(body);
    bytes.extend(61_u32.to_le_bytes());
    bytes
}

#[test]
fn cad_layer_versions_end_before_the_active_layer_index() {
    for version in 1..=8 {
        for bounded in [false, true] {
            let bytes = layer_list(CAD_LAYER_CLASS_ID, &cad_body(version, 2), bounded);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(list.active_index, Some(61));
            assert_eq!(reader.remaining(), 0);
            let Some(LayerBodyPrefix10_1::CadLayer(layer)) = &list.entries[0].parsed else {
                panic!("图层列表未读取 CadLayer v{version}");
            };
            assert_eq!(layer.name, "CAD");
            assert_eq!(layer.legacy_georeference_flag, (version == 7).then_some(5));
            assert_eq!(
                layer.georeference_parameters,
                (version == 8).then_some([13.0, 19.0])
            );
            let drawing = layer.drawing.as_ref().unwrap();
            assert_eq!(drawing.paths, ["map.mxd", "drawing.dwg"]);
            assert_eq!(drawing.base, [0.0, 1.0, 2.0, 3.0]);
            assert_eq!(drawing.destination, [4.0, 5.0, 6.0, 7.0]);
            assert_eq!(drawing.transform_parameters, [8.0, 9.0]);
            assert_eq!(drawing.values, [10, 11]);
            assert!(refs.contains(20) && refs.contains(22));
            assert_eq!(refs.contains(21), version >= 2);
        }
    }
}

#[test]
fn malformed_bounded_cad_layer_preserves_following_fields_and_discards_nested_keys() {
    let body = cad_body(8, 2);
    for broken in [&body[..body.len() - 1], &cad_body(8, i32::MAX as u32)] {
        let bytes = layer_list(CAD_LAYER_CLASS_ID, broken, true);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
        assert!(list.entries[0].parsed.is_none());
        assert!(list.entries[0].parse_note.is_some());
        assert_eq!(list.active_index, Some(61));
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(20));
        assert!(!refs.contains(21) && !refs.contains(22));
    }
}

fn feature_layer_body(bytes: &mut Vec<u8>) {
    bytes.extend(1_u16.to_le_bytes());
    string(bytes, "CAD features");
    string(bytes, "");
    bytes.extend([0; 6 + 16 + 16 + 16]);
    string(bytes, "");
    bytes.extend([0; 16 + 2 + 4 + 16]);
    string(bytes, "");
    string(bytes, "");
    bytes.extend([0; 8 + 16 + 2 + 16 + 4 + 2]);
}

fn cad_feature_body(version: u16) -> Vec<u8> {
    let mut body = version.to_le_bytes().to_vec();
    body.extend(FEATURE_LAYER_CLASS_ID);
    body.extend(27_u32.to_le_bytes());
    feature_layer_body(&mut body);
    drawing(&mut body, 2);
    if version >= 2 {
        body.extend(2_u32.to_le_bytes());
        body.extend(3_u16.to_le_bytes());
        body.extend(9_u16.to_le_bytes());
    }
    if version >= 3 {
        string(&mut body, "world file");
    }
    if version == 3 {
        body.extend(23_u16.to_le_bytes());
    }
    if version == 4 {
        body.extend(13.0_f64.to_le_bytes());
        body.extend(19.0_f64.to_le_bytes());
    }
    body
}

#[test]
fn cad_feature_layer_reads_aggregate_without_registering_its_key() {
    for version in 1..=4 {
        for bounded in [false, true] {
            let bytes = layer_list(
                CAD_FEATURE_LAYER_CLASS_ID,
                &cad_feature_body(version),
                bounded,
            );
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(
                list.active_index,
                Some(61),
                "{:?}",
                list.entries[0].parse_note
            );
            assert_eq!(reader.remaining(), 0);
            let Some(LayerBodyPrefix10_1::CadFeatureLayer(layer)) = &list.entries[0].parsed else {
                panic!("图层列表未读取 CadFeatureLayer v{version}");
            };
            assert_eq!(
                layer.feature_header,
                ObjectHeader::Object {
                    class_id: FEATURE_LAYER_CLASS_ID,
                    key: 27
                }
            );
            assert_eq!(layer.feature.as_ref().unwrap().name, "CAD features");
            assert_eq!(layer.drawing.as_ref().unwrap().values, [10, 11]);
            assert_eq!(
                layer.layer_values,
                if version >= 2 { &[3, 9][..] } else { &[][..] }
            );
            assert_eq!(
                layer.georeference.as_deref(),
                (version >= 3).then_some("world file")
            );
            assert_eq!(layer.legacy_georeference_flag, (version == 3).then_some(23));
            assert_eq!(
                layer.georeference_parameters,
                (version == 4).then_some([13.0, 19.0])
            );
            assert!(refs.contains(20) && refs.contains(22));
            assert!(!refs.contains(27));
        }
    }
}

#[test]
fn malformed_cad_feature_layer_is_skipped_only_with_a_bound() {
    let mut mismatched = cad_feature_body(4);
    mismatched[2] ^= 1;
    let truncated = cad_feature_body(4);
    for broken in [&mismatched[..], &truncated[..truncated.len() - 10]] {
        for bounded in [false, true] {
            let bytes = layer_list(CAD_FEATURE_LAYER_CLASS_ID, broken, bounded);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(list.entries[0].parsed.is_none());
            assert!(list.entries[0].parse_note.is_some());
            assert_eq!(list.active_index, bounded.then_some(61));
            assert!(!refs.contains(22) && !refs.contains(27));
        }
    }
}

fn coverage_annotation_body(version: u16) -> Vec<u8> {
    let mut body = version.to_le_bytes().to_vec();
    if version == 5 {
        feature_layer_body(&mut body);
    } else {
        string(&mut body, "旧注记");
        body.extend([0; 6 + 16 + 16]);
    }
    body.extend(1_u32.to_le_bytes());
    body.extend(FIELD_INFO_CLASS_ID);
    body.extend(51_u32.to_le_bytes());
    body.extend([0; 4]);
    string(&mut body, "Annotation");
    body.extend([0; 16]);
    for value in [10_u32, 2, 11, 12, 1] {
        body.extend(value.to_le_bytes());
    }
    body.extend(STDFONT_CLASS_ID);
    body.extend(52_u32.to_le_bytes());
    body.extend([0; 9]);
    body.extend(1_u32.to_le_bytes());
    body.extend(RGB_COLOR_CLASS_ID);
    body.extend(53_u32.to_le_bytes());
    body.extend([1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0]);
    body.extend(1_u32.to_le_bytes());
    body.extend([0; 16]);
    for value in [20_u32, 1, 21, 2] {
        body.extend(value.to_le_bytes());
    }
    body.extend(31_u16.to_le_bytes());
    body.extend(32_u16.to_le_bytes());
    body.extend(30_u32.to_le_bytes());
    body.extend(40_u32.to_le_bytes());
    body.extend([0; 16]);
    if version == 5 {
        body.extend(67_u16.to_le_bytes());
        body.extend([0; 32]);
    } else {
        if version >= 2 {
            string(&mut body, "selection");
            string(&mut body, "display");
            body.extend(23_u16.to_le_bytes());
        }
        if version >= 3 {
            body.extend(5_u32.to_le_bytes());
            body.extend([0; 16]);
        }
        if version == 4 {
            string(&mut body, "expression");
            body.extend(24_u16.to_le_bytes());
        }
    }
    body
}

#[test]
fn coverage_annotation_layer_reads_legacy_and_inline_base_versions() {
    for version in 0..=5 {
        let body = coverage_annotation_body(version);
        for bounded in [false, true] {
            let bytes = layer_list(COVERAGE_ANNOTATION_LAYER_CLASS_ID, &body, bounded);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let list = read_layer_list_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert_eq!(
                list.active_index,
                Some(61),
                "{:?}",
                list.entries[0].parse_note
            );
            assert_eq!(reader.remaining(), 0);
            let Some(LayerBodyPrefix10_1::CoverageAnnotationLayer(layer)) = &list.entries[0].parsed
            else {
                panic!("未读取 CoverageAnnotationLayer v{version}");
            };
            assert_eq!(
                layer.name(),
                if version == 5 {
                    "CAD features"
                } else {
                    "旧注记"
                }
            );
            assert_eq!(layer.longs, [10, 20, 30, 40]);
            assert_eq!(layer.integer_lists, [vec![11, 12], vec![21]]);
            assert_eq!(layer.short_list, [31, 32]);
            assert_eq!(layer.annotation_short, (version == 5).then_some(67));
            assert_eq!(
                layer.legacy_expression.as_ref().map(|(_, value)| *value),
                (version == 4).then_some(24)
            );
            assert!(refs.contains(51) && refs.contains(52) && refs.contains(53));
        }
    }
}

#[test]
fn cad_annotation_layer_registers_only_ordinary_nested_objects() {
    for version in 1_u16..=4 {
        let mut body = version.to_le_bytes().to_vec();
        body.extend(COVERAGE_ANNOTATION_LAYER_CLASS_ID);
        body.extend(27_u32.to_le_bytes());
        body.extend(coverage_annotation_body(if version >= 3 { 5 } else { 2 }));
        drawing(&mut body, 2);
        if version >= 2 {
            body.extend(1_u32.to_le_bytes());
            body.extend(9_u16.to_le_bytes());
        }
        if version >= 3 {
            string(&mut body, "world file");
        }
        if version == 3 {
            body.extend(23_u16.to_le_bytes());
        }
        if version == 4 {
            body.extend(13.0_f64.to_le_bytes());
            body.extend(19.0_f64.to_le_bytes());
        }
        let bytes = layer_list(CAD_ANNOTATION_LAYER_CLASS_ID, &body, true);
        let mut refs = RefTable10_1::new();
        let list = read_layer_list_10_1(&mut Reader::new(&bytes), 4096, &mut refs).unwrap();
        let Some(LayerBodyPrefix10_1::CadAnnotationLayer(layer)) = &list.entries[0].parsed else {
            panic!(
                "未读取 CadAnnotationLayer v{version}：{:?}",
                list.entries[0].parse_note
            );
        };
        assert_eq!(list.active_index, Some(61));
        assert_eq!(layer.tail.drawing.as_ref().unwrap().values, [10, 11]);
        assert_eq!(layer.annotation.as_ref().unwrap().short_list, [31, 32]);
        assert!(refs.contains(22) && refs.contains(51) && refs.contains(52) && refs.contains(53));
        assert!(!refs.contains(27));

        let truncated = layer_list(CAD_ANNOTATION_LAYER_CLASS_ID, &body[..body.len() - 1], true);
        let mut refs = RefTable10_1::new();
        let list = read_layer_list_10_1(&mut Reader::new(&truncated), 4096, &mut refs).unwrap();
        assert!(list.entries[0].parsed.is_none());
        assert_eq!(list.active_index, Some(61));
        assert!(
            !refs.contains(22) && !refs.contains(51) && !refs.contains(52) && !refs.contains(53)
        );
    }
}
