//! `fdo_graphics_layer` 读取器的合成输入测试。
//!
//! 字段序证据见 `src/fdo_graphics_layer.rs` 模块文档（AnnoLayer.dll
//! `0x6b1955a0`，第六轮）。合成输入按样本字节布局构造（样本条目 1：键 31，
//! 体 `/Maps:0x28e..0x2d22`），不依赖样本文件；样本对位是辅助观察。

use mxd2qgz::fdo_graphics_layer::{
    AssociatedLayer10_1, ElementCollectionSlot10_1, FDO_GRAPHICS_LAYER_CLASS_ID,
    FdoGraphicsLayerStop10_1, read_fdo_graphics_layer_prefix_10_1,
};
use mxd2qgz::feature_layer::{
    FEATURE_CLASS_NAME_CLASS_ID, FEATURE_LAYER_CLASS_ID, FeatureLayerStop10_1,
};
use mxd2qgz::object_refs::RefTable10_1;
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

/// 样本内嵌 FeatureLayer 键（`/Maps:0x2a6`）。
const SAMPLE_LAYER_KEY: u32 = 32;
/// 样本 FDO 版本（`/Maps:0x28e`）。
const SAMPLE_VERSION: u16 = 11;
/// 样本图层名称。
const SAMPLE_NAME: &str = "龙安镇人民政府工作底图用界址点";

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

/// 合成 FeatureLayer 体（到渲染器空标记 + GUID 的完整前缀链）。
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

/// 样本镜像的 FDO 体前段：版本 11 + int32 3 + short 0xFFFF + 关联 FeatureLayer
/// （头 + 体）——读取器在其前缀停止点（AfterDataSourceGuid）停止。
fn sample_associated_chain() -> Vec<u8> {
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(entry_bytes(&FEATURE_LAYER_CLASS_ID, SAMPLE_LAYER_KEY));
    bytes.extend(feature_layer_body(SAMPLE_NAME));
    bytes
}

#[test]
fn fdo_class_id_constant_matches_documented_guid() {
    // {34B2EF83-F4AC-11D1-A245-080009B6F22B}：esri.clsid.ecfg 声明 +
    // AnnoLayer.dll GetClassID 0x6b195e60 写出（第六轮）+ 样本 /Maps:0x27a
    // 图层列表条目 1 对象头，三方互证（Data1-3 小端、Data4 原序）。
    assert_eq!(
        FDO_GRAPHICS_LAYER_CLASS_ID,
        [
            0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6,
            0xf2, 0x2b,
        ]
    );
}

#[test]
fn reads_associated_layer_prefix_and_stops_inside_its_body() {
    // 样本形态：版本 11 + int32 3 + short 0xFFFF + 关联 FeatureLayer 前缀，
    // 读取器停止在关联图层体的前缀停止点（渲染器空标记 + GUID 之后），
    // ElementCollection2 槽与委派体不可达。
    let bytes = sample_associated_chain();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, SAMPLE_VERSION);
    assert_eq!(parsed.int32_field, 3);
    assert_eq!(parsed.short_field, 0xFFFF);
    let AssociatedLayer10_1::FeatureLayer(layer) = &parsed.associated_layer else {
        panic!("关联图层应为 FeatureLayer：{:?}", parsed.associated_layer);
    };
    assert_eq!(layer.name, SAMPLE_NAME);
    assert_eq!(layer.stop, FeatureLayerStop10_1::AfterDataSourceGuid);
    assert_eq!(
        parsed.stop,
        FdoGraphicsLayerStop10_1::AssociatedLayerBodyStopped {
            layer_stop: FeatureLayerStop10_1::AfterDataSourceGuid
        }
    );
    assert_eq!(parsed.element_collection, None);
    assert_eq!(parsed.delegated_layer, None);
    // 游标停在关联图层前缀终点 = 输入末尾；其键保留登记（对象头已消费）。
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(parsed.remaining_bytes, 0);
    assert!(table.contains(SAMPLE_LAYER_KEY));
}

#[test]
fn version_below_five_stops_after_version_short() {
    // 版本 <5：旧加载器与版本 4 探测分支未展开，体全部保留；版本 4 同样停
    // 止（探测函数读流，本读取器不模拟）。
    for version in [0_u16, 3, 4] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0xEE; 8]);
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap();
        assert_eq!(
            parsed.stop,
            FdoGraphicsLayerStop10_1::VersionBranchNotExpanded { version },
            "version={version}"
        );
        assert_eq!(parsed.remaining_bytes, 8, "version={version}");
        assert_eq!(reader.position(), 2, "version={version}");
    }
}

#[test]
fn rejects_versions_above_eleven_without_advancing() {
    // 原生有符号比较（汇编 CMP AX,0xb; JG）>11 拒绝；负值位模式一并拒绝
    // （较原生更严，负值版本无业务意义）。
    for version in [12_u16, 100, 0x8000, 0xFFFF] {
        let bytes = version.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        let first =
            read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert!(first.to_string().contains("FDOGraphicsLayer"), "{first}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
        let second =
            read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
                .unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn null_and_back_reference_associated_layers_advance_to_element_collection() {
    // 关联图层槽为空标记/回引时推进到 IElementCollection2 槽；空标记/回引继
    // 续到委派体（此处构造一个可解析的委派 FeatureLayer 体，验证委派分支）。
    let mut head = SAMPLE_VERSION.to_le_bytes().to_vec();
    head.extend(3_i32.to_le_bytes());
    head.extend(0_u16.to_le_bytes()); // short 0：原生跳过关联接线
    head.extend([0_u8; 16]); // ILayer 槽：空标记
    head.extend([0_u8; 16]); // IElementCollection2 槽：空标记
    head.extend(feature_layer_body("注记层")); // 委派体
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&head);
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.associated_layer, AssociatedLayer10_1::Null);
    assert_eq!(
        parsed.element_collection,
        Some(ElementCollectionSlot10_1::Null)
    );
    let delegated = parsed.delegated_layer.expect("委派体应解析");
    assert_eq!(delegated.name, "注记层");
    assert_eq!(
        parsed.stop,
        FdoGraphicsLayerStop10_1::DelegatedLayerBodyStopped {
            layer_stop: FeatureLayerStop10_1::AfterDataSourceGuid
        }
    );
    assert_eq!(reader.position(), head.len());

    // 回引形态：预置键后 20 字节回引头推进到 ElementCollection2 槽的新对象
    //（停止点——ElementCollection 体未闭合，游标回退到其头起点）。
    let mut table = RefTable10_1::new();
    table.register(SAMPLE_LAYER_KEY);
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(entry_bytes(&FEATURE_LAYER_CLASS_ID, SAMPLE_LAYER_KEY)); // 回引
    bytes.extend(entry_bytes(
        &mxd2qgz::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID, // 任意类
        86,
    ));
    bytes.extend(b"opaque");
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert!(matches!(
        parsed.associated_layer,
        AssociatedLayer10_1::BackReference {
            key: SAMPLE_LAYER_KEY,
            ..
        }
    ));
    assert_eq!(
        parsed.stop,
        FdoGraphicsLayerStop10_1::ElementCollectionNewObject {
            class_id: mxd2qgz::composite_graphics::COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
            key: 86
        }
    );
    // 回退到该对象头起点；其键不保留登记（体未消费）。
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(86));
}

#[test]
fn unsupported_associated_class_rolls_back_to_header() {
    // ILayer 槽新对象非 FeatureLayer：体未知且无外部长度，回退到对象头起点
    // 停止，键不登记。
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, 42)); // 非 FL 类
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        FdoGraphicsLayerStop10_1::AssociatedLayerUnsupported {
            class_id: FEATURE_CLASS_NAME_CLASS_ID,
            key: 42
        }
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
    assert!(!table.contains(42));
}

#[test]
fn associated_layer_failure_rolls_back_with_reason() {
    // 关联 FeatureLayer 前缀解析失败（版本 99 超限）：回退到对象头起点停
    // 止，停止原因携带失败说明，键不登记。
    let mut body = 99_u16.to_le_bytes().to_vec(); // 超出 FeatureLayer 上限 34
    body.extend(encoded("坏层"));
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend(entry_bytes(&FEATURE_LAYER_CLASS_ID, SAMPLE_LAYER_KEY));
    bytes.extend(&body);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let FdoGraphicsLayerStop10_1::AssociatedLayerFailed { reason } = &parsed.stop else {
        panic!("应为 AssociatedLayerFailed：{:?}", parsed.stop);
    };
    assert!(reason.contains("版本"), "{reason}");
    assert_eq!(
        reader.position(),
        bytes.len() - body.len() - 20,
        "回退到关联图层对象头起点"
    );
    assert!(!table.contains(SAMPLE_LAYER_KEY));
}

#[test]
fn delegated_layer_failure_keeps_body_unknown() {
    // 委派体按 FeatureLayer 字段序解析失败（FgdbFeatureClassName 等未支持数
    // 据源类）：回退到委派体起点，体保留未知——即样本委派注记图层的形态
    //（其数据源为 FgdbFeatureClassName，feature_layer 读取器不支持）。
    let mut delegated = 34_u16.to_le_bytes().to_vec();
    delegated.extend(encoded("注记"));
    delegated.extend(encoded(""));
    delegated.extend(0xFFFF_u16.to_le_bytes());
    delegated.extend(0_u16.to_le_bytes());
    delegated.extend(0_u16.to_le_bytes());
    delegated.extend(entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, 87)); // 非 FCN 支持路径
    // 注：FCN 类标识本身受支持，但体截断（无任何体字节）会让前缀失败。
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(0xFFFF_u16.to_le_bytes());
    bytes.extend([0_u8; 16]); // ILayer 槽：空标记
    bytes.extend([0_u8; 16]); // IElementCollection2 槽：空标记
    bytes.extend(&delegated);
    bytes.extend(b"opaque");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let FdoGraphicsLayerStop10_1::DelegatedLayerBodyFailed { reason } = &parsed.stop else {
        panic!("应为 DelegatedLayerBodyFailed：{:?}", parsed.stop);
    };
    assert!(!reason.is_empty());
    assert_eq!(
        reader.position(),
        bytes.len() - b"opaque".len() - delegated.len(),
        "回退到委派体起点"
    );
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断扫描：任何截断都必须失败、不推进游标、不修改登记表，且重
    // 读得到相同错误类别。
    let full = sample_associated_chain();
    // 恰好在停止点截断是合法完整输入。
    let mut boundary = Reader::new(&full);
    let mut table = RefTable10_1::new();
    let parsed = read_fdo_graphics_layer_prefix_10_1(&mut boundary, 1024, &mut table).unwrap();
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(boundary.remaining(), 0);
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        let first = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}: 失败不得修改登记表");
        let second = read_fdo_graphics_layer_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}");
    }
}

#[test]
fn limit_is_forwarded_to_associated_layer_strings() {
    // 限额转发给关联 FeatureLayer 前缀的名称字符串：声明长度超限在读取与分
    // 配之前拒绝（此处按关联图层前缀失败回退到其对象头起点，为成功停止）。
    let bytes = sample_associated_chain();
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_fdo_graphics_layer_prefix_10_1(&mut reader, 30, &mut RefTable10_1::new()).unwrap();
    assert!(matches!(
        parsed.stop,
        FdoGraphicsLayerStop10_1::AssociatedLayerFailed { .. }
    ));
    // 名称声明 32（样本名 15 字）：限额 32 可读、31 在分配前拒绝使关联前缀
    // 失败——成功停止（AssociatedLayerFailed），游标回退到对象头起点。
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_fdo_graphics_layer_prefix_10_1(&mut reader, 31, &mut RefTable10_1::new()).unwrap();
    assert!(matches!(
        parsed.stop,
        FdoGraphicsLayerStop10_1::AssociatedLayerFailed { .. }
    ));
}
