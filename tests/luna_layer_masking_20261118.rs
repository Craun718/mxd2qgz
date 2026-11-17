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
