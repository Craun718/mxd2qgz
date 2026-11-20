use mxd2qgz::layers::fdo_graphics_layer::FDO_GRAPHICS_LAYER_CLASS_ID;
use mxd2qgz::layers::fdo_legacy::FDO_GRAPHICS_SUBLAYER_CLASS_ID;
use mxd2qgz::layers::feature_layer::FEATURE_LAYER_CLASS_ID;
use mxd2qgz::layers::group_layer::GROUP_LAYER_CLASS_ID;
use mxd2qgz::maps::layer_masking::{
    LayerMaskingProbe10_1, LayerMaskingSupport10_1, known_layer_masking_support_10_1,
    layer_masking_probe_10_1,
};

#[test]
fn map_version_and_basemap_qi_control_the_masking_probe() {
    assert_eq!(
        layer_masking_probe_10_1(39, true),
        LayerMaskingProbe10_1::SkipLegacyBasemapLayer
    );
    assert_eq!(
        layer_masking_probe_10_1(40, true),
        LayerMaskingProbe10_1::QueryLayerMasking
    );
    assert_eq!(
        layer_masking_probe_10_1(39, false),
        LayerMaskingProbe10_1::QueryLayerMasking
    );
    assert_eq!(
        layer_masking_probe_10_1(-1, true),
        LayerMaskingProbe10_1::SkipLegacyBasemapLayer
    );
}

#[test]
fn verified_layer_qi_results_are_explicit_and_unknowns_stay_unknown() {
    assert_eq!(
        known_layer_masking_support_10_1(&GROUP_LAYER_CLASS_ID),
        LayerMaskingSupport10_1::Supported
    );
    for class_id in [
        FEATURE_LAYER_CLASS_ID,
        FDO_GRAPHICS_LAYER_CLASS_ID,
        FDO_GRAPHICS_SUBLAYER_CLASS_ID,
        [
            0xc9, 0x71, 0x23, 0xd0, 0xf7, 0x35, 0xd2, 0x11, 0xb1, 0xf2, 0x00, 0xc0, 0x4f, 0x8e,
            0xde, 0xff,
        ],
    ] {
        assert_eq!(
            known_layer_masking_support_10_1(&class_id),
            LayerMaskingSupport10_1::Unsupported
        );
    }
    assert_eq!(
        known_layer_masking_support_10_1(&[0; 16]),
        LayerMaskingSupport10_1::Unresolved
    );
}

#[test]
fn negative_masking_version_omits_target_names_and_preserves_raw_value() {
    use mxd2qgz::maps::layer_masking::read_layer_masking_10_1;
    use mxd2qgz::storage::object_refs::RefTable10_1;
    use mxd2qgz::stream::reader::Reader;

    let mut bytes = 0x8000_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]); // 外层空对象。
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend([0; 16]); // 内层空对象，旧版本后面直接跟尾 short。
    bytes.extend(0xffff_u16.to_le_bytes());
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let mut refs = RefTable10_1::new();
        assert!(read_layer_masking_10_1(&mut reader, 1024, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
    }

    let mut reader = Reader::new(&bytes);
    let result = read_layer_masking_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(result.version, 0x8000);
    assert_eq!(result.entries[0].targets[0].1, None);
    assert_eq!(result.tail, 0xffff);
    assert_eq!(reader.position(), bytes.len());
}
