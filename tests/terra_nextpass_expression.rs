use mxd2qgz::layers::annotate_props::{
    AlpcEntrySlot10_1, LABEL_ENGINE_PROPS_CLASS_ID, read_annotate_props_collection_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn alpc_v2_null_has_no_id_before_a_back_reference() {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend([0; 16]);
    bytes.extend(LABEL_ENGINE_PROPS_CLASS_ID);
    bytes.extend(17_u32.to_le_bytes());
    bytes.extend(41_i32.to_le_bytes());
    bytes.extend(42_i32.to_le_bytes());
    let end = bytes.len();
    bytes.push(0xae);
    let mut table = RefTable10_1::new();
    table.register(17);
    let mut reader = Reader::new(&bytes);
    let body = read_annotate_props_collection_body_10_1(&mut reader, 1024, &mut table, 1).unwrap();
    assert!(matches!(body.entries[0].slot, AlpcEntrySlot10_1::Null));
    assert_eq!(body.entries[0].entry_key, None);
    assert!(matches!(
        body.entries[1].slot,
        AlpcEntrySlot10_1::BackReference { key: 17, .. }
    ));
    assert_eq!(body.entries[1].entry_key, Some(41));
    assert_eq!(body.trailing_int32, Some(42));
    assert_eq!(reader.position(), end);
    for cut in 0..end {
        let mut truncated = Reader::new(&bytes[..cut]);
        let mut staged = table.clone();
        assert!(
            read_annotate_props_collection_body_10_1(&mut truncated, 1024, &mut staged, 1).is_err()
        );
        assert_eq!(truncated.position(), 0);
        assert_eq!(staged.len(), 1);
        assert!(staged.contains(17));
    }
}

#[test]
fn parse_reports_label_properties_from_reference_mxd() {
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/测试底图.mxd");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mxd2qgz"))
        .arg("parse")
        .arg(sample)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for (index, key, expression, layer_flag, size_cy) in [
        (1, 60, "[JZDH]", 0, 120000),
        (3, 202, "[收购合同号]", 0, 80000),
        (4, 239, "[DJZQMC]", -1, 160000),
    ] {
        let layer_note = format!(
            "条目 {index}：图层标注开关（IGeoFeatureLayer::DisplayAnnotation 原始值 {layer_flag}）"
        );
        assert!(stdout.lines().any(|line| line.ends_with(&layer_note)));

        let expression_note = format!("表达式 {expression:?}、标注类名");
        let class_note = format!("条目 {index}：标注属性集合");
        assert!(
            stdout.lines().any(|line| {
                line.contains(&class_note)
                    && line.contains("条目 LELP")
                    && line.contains(&expression_note)
                    && line.contains("标注类开关（DisplayAnnotation 原始值 -1）")
            }),
            "未在条目 {index} 读到预期的表达式与标注类开关"
        );

        let font_note = format!(
            "条目 {index}：标注类 LELP（键 {key}）字体 StdFont（名称字节 [cb, ce, cc, e5]，字符集 Some(134)，字重 400，字号 CY {size_cy}，样式位 0x00）"
        );
        assert!(stdout.lines().any(|line| line.ends_with(&font_note)));

        let placement_note = format!("条目 {index}：标注类 LELP（键 {key}）放置属性");
        assert!(stdout.lines().any(|line| {
            line.contains(&placement_note)
                && line.contains("Offset 0")
                && line.contains("LineLabelPosition")
                && line.contains("Above -1，Below 0，OnTop 0，Left 0，Right 0，InLine -1，AtStart 0，AtEnd 0，Parallel -1，Perpendicular 0，Horizontal 0，Offset 0，ProduceCurvedLabels 0")
                && line.contains("线放置优先级 [0, 3, 3, 3, 0, 0, 3, 3, 3, 0, 0, 3, 3, 3, 0]")
                && line.contains("点放置优先级 [2, 2, 1, 2, 2, 3, 3, 3]")
        }));
    }
}

#[test]
fn alpc_reads_maplex_version_gates_and_stops_before_an_unknown_placement_body() {
    use mxd2qgz::layers::annotate_props::{
        AnnotatePropsStop10_1, LelpStop10_1, MAPLEX_LABEL_ENGINE_PROPS_CLASS_ID,
    };

    for version in [1_u16, 6, 9, 11] {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(MAPLEX_LABEL_ENGINE_PROPS_CLASS_ID);
        bytes.extend(17_u32.to_le_bytes());
        bytes.extend(version.to_le_bytes());
        bytes.extend([4, 0, 0, 0, b'X', 0, 0, 0]);
        bytes.extend([0; 2]);
        if version == 1 {
            bytes.extend([0; 2]);
        }
        bytes.extend([0; 40]);
        for _ in 0..2 {
            bytes.extend([2, 0, 0, 0, 0, 0]);
        }
        bytes.extend([0; 48]);
        if version < 11 {
            bytes.extend([0; 16]);
        }
        bytes.extend([0; 24]);
        if version >= 3 {
            bytes.extend([0; 16]);
        }
        if version >= 4 {
            bytes.extend([0; 2]);
        }
        let placement_offset = bytes.len();
        if version >= 5 {
            bytes.extend([0; 16]);
        }
        if version == 6 {
            bytes.extend(123_i32.to_le_bytes());
        }
        if version >= 8 {
            bytes.extend(51_i32.to_le_bytes());
        }
        if version == 9 {
            bytes.extend(9_i16.to_le_bytes());
        }
        bytes.extend(41_i32.to_le_bytes());
        bytes.extend(42_i32.to_le_bytes());

        for cut in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(
                read_annotate_props_collection_body_10_1(&mut reader, 1024, &mut refs, 1).is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let result =
            read_annotate_props_collection_body_10_1(&mut reader, 1024, &mut refs, 1).unwrap();
        let AlpcEntrySlot10_1::MaplexLabelEngineProps(body) = &result.entries[0].slot else {
            panic!("应读取 Maplex 标注类");
        };
        assert_eq!(body.expression, "X");
        assert_eq!(body.annotation_class_id, (version >= 8).then_some(51));
        assert_eq!(
            body.maplex_legacy_fields,
            ((version == 6).then_some(123), (version == 9).then_some(9))
        );
        assert_eq!(result.entries[0].entry_key, Some(41));
        assert_eq!(result.trailing_int32, Some(42));
        assert_eq!(reader.position(), bytes.len());

        if version == 11 {
            bytes.splice(
                placement_offset..placement_offset + 16,
                [0x7e; 16].into_iter().chain(19_u32.to_le_bytes()),
            );
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result =
                read_annotate_props_collection_body_10_1(&mut reader, 1024, &mut refs, 1).unwrap();
            assert!(matches!(
                result.stop,
                Some(AnnotatePropsStop10_1::LelpBody(
                    LelpStop10_1::MaplexOverposterSlotUnsupported { key: 19, .. }
                ))
            ));
            assert_eq!(result.entries[0].entry_key, None);
            assert_eq!(reader.position(), placement_offset);
            assert!(refs.contains(17));
            assert!(!refs.contains(19));
        }
    }
}
