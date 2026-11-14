//! `fdo_legacy` 读取器的合成输入测试。
//!
//! 字段序证据见 `src/fdo_legacy.rs` 模块文档（AnnoLayer.dll 旧加载器
//! `FUN_6b197dc0` 与 FDOGraphicsSublayer::Load `0x6b1a1b10`，第七轮）。
//! 两个读取器均无样本字节佐证（样本 FDO 版本 11、元素计数 0），合成输入
//! 按反编译字段序构造，不依赖样本文件。

use mxd2qgz::fdo_graphics_layer::AssociatedLayer10_1;
use mxd2qgz::fdo_legacy::{
    FDO_GRAPHICS_SUBLAYER_CLASS_ID, LegacyDataSource10_1, LegacyFdoStop10_1,
    LegacySpatialReferenceSlot10_1, read_fdo_graphics_layer_legacy_prefix_10_1,
    read_fdo_graphics_sublayer_10_1,
};
use mxd2qgz::feature_layer::{
    FEATURE_CLASS_NAME_CLASS_ID, FEATURE_LAYER_CLASS_ID, FeatureLayerStop10_1,
};
use mxd2qgz::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::reader::Reader;
use mxd2qgz::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use std::io::ErrorKind;

/// AfCore String::Write 布局的合法输入。
fn encoded(text: &str) -> Vec<u8> {
    let payload: Vec<_> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut bytes = (payload.len() as u32).to_le_bytes().to_vec();
    bytes.extend(payload);
    bytes
}

/// 对象头字节：16 字节类标识 + 4 字节键。
fn entry_bytes(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

/// 合成 FeatureLayer 体（到渲染器空标记 + GUID 的完整前缀链，与
/// `fdo_graphics_layer` 测试同构）。
fn feature_layer_body(name: &str) -> Vec<u8> {
    let mut bytes = 34_u16.to_le_bytes().to_vec();
    bytes.extend(encoded(name));
    bytes.extend(encoded(""));
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // 数据源：空标记
    bytes.extend([0_u8; 16]); // 渲染器：空标记
    bytes.extend([0x6A; 16]); // +0x144 GUID
    bytes
}

/// 合成 FeatureClassName 完整体：版本 2、四字符串、int32 ×2、选择子 0、
/// WorkspaceName 空标记、版本 >1 选择子 0（stop = None 形态）。
fn fcn_body_complete() -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(encoded("界址点"));
    bytes.extend(encoded(""));
    bytes.extend(encoded("Shapefile 要素类"));
    bytes.extend(encoded("Shape"));
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes()); // 选择子 0：IWorkspaceName 槽
    bytes.extend([0_u8; 16]); // WorkspaceName：空标记
    bytes.extend(0_u16.to_le_bytes()); // 版本 >1 选择子 0
    bytes
}

/// 合成 PCS 体（保存版本 1：仅 short + 长度 + ASCII WKT）。
fn pcs_body_v1() -> Vec<u8> {
    let wkt = b"GEOGCS[\"GCS_WGS_1984\"]";
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend((wkt.len() as u32).to_le_bytes());
    bytes.extend(wkt);
    bytes
}

/// 旧体头部（版本 + 名称 + 标量组），返回 (字节, 版本)。
fn legacy_head(version: u16, name: &str) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(encoded(name));
    bytes.extend(0x11_u16.to_le_bytes());
    bytes.extend(0x22_u16.to_le_bytes());
    bytes.extend(1.5_f64.to_le_bytes());
    bytes.extend(2.5_f64.to_le_bytes());
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes
}

/// 完整版本 3 旧体（全部槽位空标记）。
fn complete_legacy_body_v3() -> Vec<u8> {
    let mut bytes = legacy_head(3, "旧注记层");
    bytes.extend([0_u8; 16]); // ILayer：空标记
    bytes.extend([0_u8; 16]); // ISpatialReference：空标记
    bytes.extend([0_u8; 16]); // IElementCollection2：空标记
    bytes.extend([0_u8; 16]); // IName：空标记
    // 版本 ≥2 块。
    bytes.extend(0x33_u16.to_le_bytes());
    bytes.extend(encoded("甲"));
    bytes.extend(encoded("乙"));
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend([0_u8; 16]); // IFeatureRenderer：空标记
    bytes.extend(0x44_u16.to_le_bytes());
    bytes.extend(0x55_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // IColor：空标记
    bytes.extend(encoded("收尾")); // 版本 ≥3 收尾 String
    bytes
}

#[test]
fn legacy_reader_rejects_versions_outside_legacy_branch() {
    // 版本 4（探测函数未展开）与 5..=11（主分支）都不属于旧加载器；负值位
    // 模式一并拒绝（较原生更严）。全部失败不推进游标。
    for version in [4_u16, 5, 11, 12, 0x8000, 0xFFFF] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let first =
            read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert!(first.to_string().contains("旧加载器"), "{first}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn reads_complete_version_three_body() {
    // 版本 3 完整旧体：stop = None（旧加载器收尾 String 后返回，体完整终
    // 结），游标到达输入末尾。
    let bytes = complete_legacy_body_v3();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.name, "旧注记层");
    assert_eq!(parsed.short_a, 0x11);
    assert_eq!(parsed.short_b, 0x22);
    assert_eq!(parsed.double_a, 1.5);
    assert_eq!(parsed.double_b, 2.5);
    assert_eq!(parsed.int32_field, 3);
    assert_eq!(parsed.short_field, 0xFFFF);
    assert_eq!(parsed.associated_layer, AssociatedLayer10_1::Null);
    assert_eq!(
        parsed.spatial_reference,
        LegacySpatialReferenceSlot10_1::Null
    );
    assert_eq!(parsed.element_collection, Some(ObjectEntry10_1::Null));
    assert_eq!(parsed.data_source, LegacyDataSource10_1::Null);
    assert_eq!(parsed.v2_short, Some(0x33));
    assert_eq!(parsed.v2_string_a.as_deref(), Some("甲"));
    assert_eq!(parsed.v2_string_b.as_deref(), Some("乙"));
    assert_eq!(parsed.v2_int32, Some(7));
    assert_eq!(parsed.renderer_slot, Some(ObjectEntry10_1::Null));
    assert_eq!(parsed.v2_short_x, Some(0x44));
    assert_eq!(parsed.v2_short_y, Some(0x55));
    assert_eq!(parsed.color_slot, Some(ObjectEntry10_1::Null));
    assert_eq!(parsed.tail_string.as_deref(), Some("收尾"));
    assert_eq!(parsed.stop, None);
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.is_empty());
}

#[test]
fn version_gates_skip_later_fields() {
    // 版本 0/1：无版本 ≥2 块、无收尾 String——IName 空标记后即完整终结。
    for version in [0_u16, 1] {
        let mut bytes = legacy_head(version, "旧层");
        bytes.extend([0_u8; 16]); // ILayer：空标记
        bytes.extend([0_u8; 16]); // ISpatialReference：空标记
        bytes.extend([0_u8; 16]); // IElementCollection2：空标记
        bytes.extend([0_u8; 16]); // IName：空标记
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap();
        assert_eq!(parsed.stop, None, "version={version}");
        assert_eq!(parsed.v2_short, None, "version={version}");
        assert_eq!(parsed.tail_string, None, "version={version}");
        assert_eq!(reader.position(), bytes.len(), "version={version}");
    }
    // 版本 2：读版本 ≥2 块、无收尾 String。
    let mut bytes = legacy_head(2, "旧层");
    bytes.extend([0_u8; 16]); // ILayer
    bytes.extend([0_u8; 16]); // ISpatialReference
    bytes.extend([0_u8; 16]); // IElementCollection2
    bytes.extend([0_u8; 16]); // IName
    bytes.extend(0x33_u16.to_le_bytes());
    bytes.extend(encoded("甲"));
    bytes.extend(encoded("乙"));
    bytes.extend(7_i32.to_le_bytes());
    bytes.extend([0_u8; 16]); // IFeatureRenderer
    bytes.extend(0x44_u16.to_le_bytes());
    bytes.extend(0x55_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // IColor
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap();
    assert_eq!(parsed.stop, None);
    assert_eq!(parsed.v2_short, Some(0x33));
    assert_eq!(parsed.tail_string, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn associated_layer_feature_layer_stops_inside_its_body() {
    // ILayer 槽新 FeatureLayer：解析前缀后在 AfterDataSourceGuid 停止，旧体
    // 后续槽位不可达；键保留登记（对象头已消费）。
    let mut bytes = legacy_head(1, "旧层");
    bytes.extend(entry_bytes(&FEATURE_LAYER_CLASS_ID, 77));
    bytes.extend(feature_layer_body("关联层"));
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let AssociatedLayer10_1::FeatureLayer(layer) = &parsed.associated_layer else {
        panic!("应为 FeatureLayer：{:?}", parsed.associated_layer);
    };
    assert_eq!(layer.name, "关联层");
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::AssociatedLayerBodyStopped {
            layer_stop: FeatureLayerStop10_1::AfterDataSourceGuid
        })
    );
    assert_eq!(
        reader.position(),
        bytes.len() - b"opaque".len(),
        "停在关联图层前缀终点"
    );
    assert!(table.contains(77));
}

#[test]
fn associated_layer_unsupported_rolls_back_to_header() {
    // ILayer 槽新对象非 FeatureLayer：回退到对象头起点，键不登记。
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend(entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, 42));
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::AssociatedLayerUnsupported {
            class_id: FEATURE_CLASS_NAME_CLASS_ID,
            key: 42
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(42));
}

#[test]
fn spatial_reference_pcs_is_consumed_and_reading_continues() {
    // ISpatialReference 槽新 PCS：体完整消费后继续 IElementCollection2 与
    // IName 槽（版本 0：其后即终结）。
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend([0_u8; 16]); // ILayer：空标记
    bytes.extend(entry_bytes(&PROJECTED_COORDINATE_SYSTEM_CLASS_ID, 9));
    bytes.extend(pcs_body_v1());
    bytes.extend([0_u8; 16]); // IElementCollection2：空标记
    bytes.extend([0_u8; 16]); // IName：空标记
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let LegacySpatialReferenceSlot10_1::ProjectedCoordinateSystem(pcs) = &parsed.spatial_reference
    else {
        panic!("应为 PCS：{:?}", parsed.spatial_reference);
    };
    assert_eq!(pcs.wkt, "GEOGCS[\"GCS_WGS_1984\"]");
    assert_eq!(parsed.element_collection, Some(ObjectEntry10_1::Null));
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(9));
}

#[test]
fn spatial_reference_unsupported_rolls_back_to_header() {
    // ISpatialReference 槽新对象非 PCS（如 GCS 类）：回退停止，键不登记。
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend([0_u8; 16]); // ILayer：空标记
    bytes.extend(entry_bytes(&[0x99; 16], 10));
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::SpatialReferenceUnsupported {
            class_id: [0x99; 16],
            key: 10
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(10));
}

#[test]
fn element_collection_new_object_rolls_back_to_header() {
    // IElementCollection2 槽任何新对象（体布局未闭合，Layer.dll）：回退停
    // 止，键不登记。
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend([0_u8; 16]); // ILayer：空标记
    bytes.extend([0_u8; 16]); // ISpatialReference：空标记
    bytes.extend(entry_bytes(&[0xAB; 16], 11));
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::ElementCollectionNewObject {
            class_id: [0xAB; 16],
            key: 11
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(11));
}

#[test]
fn data_source_fcn_complete_body_advances_to_version_two_block() {
    // IName 槽新 FeatureClassName（完整体）：消费后继续版本 ≥2 块（版本 2
    // 无收尾 String），FCN 键保留登记。
    let mut bytes = legacy_head(2, "旧层");
    bytes.extend([0_u8; 16]); // ILayer：空标记
    bytes.extend([0_u8; 16]); // ISpatialReference：空标记
    bytes.extend([0_u8; 16]); // IElementCollection2：空标记
    bytes.extend(entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, 33));
    bytes.extend(fcn_body_complete());
    bytes.extend(0x66_u16.to_le_bytes()); // v2 short
    bytes.extend(encoded("表达式"));
    bytes.extend(encoded(""));
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend([0_u8; 16]); // IFeatureRenderer：空标记
    bytes.extend(0x77_u16.to_le_bytes());
    bytes.extend(0x88_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // IColor：空标记
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let LegacyDataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!("应为 FCN：{:?}", parsed.data_source);
    };
    assert_eq!(fcn.name, "界址点");
    assert_eq!(fcn.stop, None);
    assert_eq!(parsed.v2_short, Some(0x66));
    assert_eq!(parsed.v2_string_a.as_deref(), Some("表达式"));
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert!(table.contains(33));
}

#[test]
fn data_source_fcn_internal_stop_stops_legacy_body() {
    // FCN 体内停止（选择子非 0 → IDatasetName 新对象）：旧体后续字段不可
    // 达，停止原因透传。
    let mut fcn = 2_u16.to_le_bytes().to_vec();
    fcn.extend(encoded("名"));
    fcn.extend(encoded(""));
    fcn.extend(encoded("类型"));
    fcn.extend(encoded("集"));
    fcn.extend(0_i32.to_le_bytes());
    fcn.extend(0_i32.to_le_bytes());
    fcn.extend(1_u16.to_le_bytes()); // 选择子非 0：IDatasetName 槽
    fcn.extend(entry_bytes(&[0xCD; 16], 55));
    fcn.extend(b"opaque");
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend([0_u8; 16]); // ILayer
    bytes.extend([0_u8; 16]); // ISpatialReference
    bytes.extend([0_u8; 16]); // IElementCollection2
    bytes.extend(entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, 33));
    bytes.extend(&fcn);
    bytes.extend(b"opaque2");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(matches!(
        parsed.stop,
        Some(LegacyFdoStop10_1::DataSourceStopped { .. })
    ));
    // 游标停在 FCN 的停止点——其内部对 IDatasetName 槽回退到对象头起点
    //（20 字节头不消费），其后保留未知。
    assert_eq!(
        reader.position(),
        bytes.len() - b"opaque2".len() - b"opaque".len() - 20
    );
    assert!(table.contains(33));
    assert!(!table.contains(55));
}

#[test]
fn data_source_unsupported_rolls_back_to_header() {
    // IName 槽新对象非 FeatureClassName：回退停止，键不登记。
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend([0_u8; 16]); // ILayer
    bytes.extend([0_u8; 16]); // ISpatialReference
    bytes.extend([0_u8; 16]); // IElementCollection2
    bytes.extend(entry_bytes(&FEATURE_LAYER_CLASS_ID, 66));
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::DataSourceUnsupported {
            class_id: FEATURE_LAYER_CLASS_ID,
            key: 66
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(66));
}

#[test]
fn renderer_and_color_new_objects_roll_back_to_header() {
    // 版本 ≥2 的渲染器/颜色槽新对象（体未闭合）：各自回退停止，键不登记。
    let mut bytes = legacy_head(2, "旧层");
    bytes.extend([0_u8; 16]); // ILayer
    bytes.extend([0_u8; 16]); // ISpatialReference
    bytes.extend([0_u8; 16]); // IElementCollection2
    bytes.extend([0_u8; 16]); // IName
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(entry_bytes(&[0xF3; 16], 35)); // SimpleRenderer 类候选字节
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::RendererNewObject {
            class_id: [0xF3; 16],
            key: 35
        })
    );
    assert_eq!(parsed.v2_short_x, None, "渲染器停止后不可达");
    let header_end = bytes.len() - b"opaque".len() - 20;
    assert_eq!(reader.position(), header_end);
    assert!(!table.contains(35));

    // 颜色槽：渲染器空标记后两个 short，再 IColor 新对象。
    let mut bytes = legacy_head(2, "旧层");
    bytes.extend([0_u8; 16]);
    bytes.extend([0_u8; 16]);
    bytes.extend([0_u8; 16]);
    bytes.extend([0_u8; 16]);
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend([0_u8; 16]); // IFeatureRenderer：空标记
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes());
    bytes.extend(entry_bytes(&[0xEE; 16], 36)); // RgbColor 类候选字节
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(LegacyFdoStop10_1::ColorNewObject {
            class_id: [0xEE; 16],
            key: 36
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(36));
}

#[test]
fn rejects_every_truncated_legacy_body_without_advancing() {
    // 逐字节截断扫描：任何截断都必须失败、不推进游标、不修改登记表，且重
    // 读得到相同错误类别。
    let full = complete_legacy_body_v3();
    let mut boundary = Reader::new(&full);
    let parsed =
        read_fdo_graphics_layer_legacy_prefix_10_1(&mut boundary, 1024, &mut RefTable10_1::new())
            .unwrap();
    assert_eq!(parsed.stop, None);
    assert_eq!(boundary.remaining(), 0);
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        let first = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}: 失败不得修改登记表");
        let second = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}");
    }
}

#[test]
fn back_reference_entries_advance_without_bodies() {
    // 各槽位回引形态：预置键后 20 字节回引头不携体，直接推进到下一槽位。
    let mut table = RefTable10_1::new();
    table.register(29);
    table.register(30);
    table.register(86);
    table.register(33);
    let mut bytes = legacy_head(0, "旧层");
    bytes.extend(entry_bytes(&FEATURE_LAYER_CLASS_ID, 29)); // ILayer 回引
    bytes.extend(entry_bytes(&PROJECTED_COORDINATE_SYSTEM_CLASS_ID, 30)); // SR 回引
    bytes.extend(entry_bytes(&[0xCE; 16], 86)); // IElementCollection2 回引
    bytes.extend(entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, 33)); // IName 回引
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(matches!(
        parsed.associated_layer,
        AssociatedLayer10_1::BackReference { key: 29, .. }
    ));
    assert!(matches!(
        parsed.spatial_reference,
        LegacySpatialReferenceSlot10_1::BackReference { key: 30, .. }
    ));
    assert!(matches!(
        parsed.element_collection,
        Some(ObjectEntry10_1::BackReference { key: 86, .. })
    ));
    assert!(matches!(
        parsed.data_source,
        LegacyDataSource10_1::BackReference { key: 33, .. }
    ));
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(table.len(), 4);
}

#[test]
fn limit_is_forwarded_to_legacy_strings() {
    // 限额转发：名称声明长度超限在读取与分配之前拒绝（UnexpectedEof 之外
    // 的内容性失败按错误向上传播——名称不在槽位回退保护范围内）。
    let bytes = complete_legacy_body_v3();
    let mut reader = Reader::new(&bytes);
    let error =
        read_fdo_graphics_layer_legacy_prefix_10_1(&mut reader, 4, &mut RefTable10_1::new())
            .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}

#[test]
fn sublayer_class_id_constant_matches_documented_guid() {
    // {4ABA3851-BC3A-43C2-8FBE-93A65AD91E83}：AnnoLayer.dll
    // FDOGraphicsSublayer IPersistStream GetClassID 0x6b1a1ae0 与主虚表
    // +0x14 的 0x6b1a1590 两处写出互证（Data1-3 小端、Data4 原序）；未登
    // 记于 esri.clsid.ecfg（内部子对象，无对象头，不出现于流中）。
    assert_eq!(
        FDO_GRAPHICS_SUBLAYER_CLASS_ID,
        [
            0x51, 0x38, 0xba, 0x4a, 0x3a, 0xbc, 0xc2, 0x43, 0x8f, 0xbe, 0x93, 0xa6, 0x5a, 0xd9,
            0x1e, 0x83,
        ]
    );
}

/// 子层体公共前段（版本 + int32 + String + short + double ×2 + String）。
fn sublayer_common(version: u16) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend(encoded("子层"));
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(0.25_f64.to_le_bytes());
    bytes.extend(0.5_f64.to_le_bytes());
    bytes.extend(encoded("第二"));
    bytes
}

#[test]
fn reads_sublayer_bodies_for_all_versions() {
    // 版本 0：无追加字段；版本 1：追加 int16；版本 2：追加 double ×2。
    let v0 = sublayer_common(0);
    let mut reader = Reader::new(&v0);
    let parsed = read_fdo_graphics_sublayer_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.version, 0);
    assert_eq!(parsed.int32_field, -1);
    assert_eq!(parsed.name, "子层");
    assert_eq!(parsed.short_field, 0xFFFF);
    assert_eq!(parsed.double_a, 0.25);
    assert_eq!(parsed.double_b, 0.5);
    assert_eq!(parsed.secondary_name, "第二");
    assert_eq!(parsed.version1_short, None);
    assert_eq!(parsed.version2_doubles, None);
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(reader.position(), v0.len());

    let mut v1 = sublayer_common(1);
    v1.extend(0x11_u16.to_le_bytes());
    let mut reader = Reader::new(&v1);
    let parsed = read_fdo_graphics_sublayer_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.version1_short, Some(0x11));
    assert_eq!(parsed.version2_doubles, None);
    assert_eq!(reader.position(), v1.len());

    let mut v2 = sublayer_common(2);
    v2.extend(1.25_f64.to_le_bytes());
    v2.extend(2.25_f64.to_le_bytes());
    v2.extend(b"tail");
    let mut reader = Reader::new(&v2);
    let parsed = read_fdo_graphics_sublayer_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.version1_short, None);
    assert_eq!(parsed.version2_doubles, Some([1.25, 2.25]));
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), v2.len() - 4);
}

#[test]
fn sublayer_rejects_versions_above_two_and_negative_patterns() {
    // 原生 2 < (short)版本 拒绝（0x80004005）；负值位模式一并拒绝（较原生
    // 更严）。失败不推进游标、重读一致。
    for version in [3_u16, 100, 0x8000, 0xFFFF] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let first = read_fdo_graphics_sublayer_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert!(first.to_string().contains("FDOGraphicsSublayer"), "{first}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
        let second = read_fdo_graphics_sublayer_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn sublayer_rejects_every_truncated_body_without_advancing() {
    // 逐字节截断扫描：任何截断都必须失败、不推进游标、重读一致。
    let mut full = sublayer_common(2);
    full.extend(1.0_f64.to_le_bytes());
    full.extend(2.0_f64.to_le_bytes());
    for end in 0..full.len() {
        let mut reader = Reader::new(&full[..end]);
        let first = read_fdo_graphics_sublayer_10_1(&mut reader, 1024)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_fdo_graphics_sublayer_10_1(&mut reader, 1024)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
    // 恰好完整是合法输入。
    let mut reader = Reader::new(&full);
    let parsed = read_fdo_graphics_sublayer_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.remaining_bytes, 0);
}

#[test]
fn sublayer_limit_rejects_oversized_string_declarations() {
    // 限额约束两个 String 的声明长度：超限在读取与分配之前拒绝。
    let bytes = sublayer_common(0);
    let mut reader = Reader::new(&bytes);
    let error = read_fdo_graphics_sublayer_10_1(&mut reader, 4).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
}
