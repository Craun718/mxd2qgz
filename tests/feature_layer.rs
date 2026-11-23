//! `feature_layer` 读取器的合成输入测试。
//!
//! 字段序证据见 `src/feature_layer.rs` 模块文档（FeatureLayer.dll
//! `0x6f6e9940` 头部段 + GdbCore.dll `FeatureClassName::Load 0x709b5080`、
//! `WorkspaceName::Load 0x70a27950`，第六轮）。合成输入按样本字节布局构造
//! （`samples/测试底图.mxd` /Maps 首个 FeatureLayer：键 32，体 `0x2aa` 起），
//! 不依赖样本文件本身；样本对位是辅助观察。

use mxd2qgz::layers::feature_layer::{
    DataSource10_1, DatasetNameEntry10_1, FEATURE_CLASS_NAME_CLASS_ID, FEATURE_LAYER_CLASS_ID,
    FeatureClassNameNested10_1, FeatureClassNameStop10_1, FeatureLayerStop10_1, RendererSlot10_1,
    TopologyNameEntry10_1, Visibility10_1, WORKSPACE_NAME_CLASS_ID, WorkspaceNameEntry10_1,
    WorkspaceNameStop10_1, read_feature_class_name_body_10_1, read_feature_layer_prefix_10_1,
    read_workspace_name_body_10_1,
};
use mxd2qgz::layers::feature_layer_tail::SuccessorStop10_1;
use mxd2qgz::objects::dataset_name::{FEATURE_DATASET_NAME_CLASS_ID, TOPOLOGY_NAME_CLASS_ID};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::storage::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// 样本 FeatureLayer 对象键（`/Maps:0x2a6`）。
const SAMPLE_LAYER_KEY: u32 = 32;
/// 样本内嵌 FeatureClassName 对象键（`/Maps:0x306`）。
const SAMPLE_FCN_KEY: u32 = 33;
/// 样本内嵌 WorkspaceName 对象键（`/Maps:0x380`）。
const SAMPLE_WS_KEY: u32 = 34;
/// 样本 FeatureLayer 架构版本（`/Maps:0x2aa`）。
const SAMPLE_VERSION: u16 = 34;
/// 样本图层名称（与要素类名同文）。
const SAMPLE_NAME: &str = "龙安镇人民政府工作底图用界址点";
/// 样本第二名称。
const SAMPLE_SECONDARY_NAME: &str = "Shapefile 要素类";
/// SimpleRenderer 类标识 {F3435801-5779-11D0-98BF-00805F7CED21}（esri.clsid
/// 声明；样本 /Maps:0x3ed 渲染器槽位对象头，第六轮对位）。
const SIMPLE_RENDERER_CLASS_ID: [u8; 16] = [
    0x01, 0x58, 0x43, 0xf3, 0x79, 0x57, 0xd0, 0x11, 0x98, 0xbf, 0x00, 0x80, 0x5f, 0x7c, 0xed, 0x21,
];

/// AfCore String::Write 布局的合法输入：u32 字节长度前缀 + 含终止零的 UTF-16LE。
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

/// WorkspaceName 体（样本镜像：`/Maps:0x384..0x3eb`）。
fn workspace_name_body() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec(); // 版本 1
    bytes.extend(encoded("..\\22.界址点")); // 路径
    bytes.extend(encoded("")); // 第二路径（空串）
    bytes.extend(encoded("Shapefile Data")); // 类型描述
    bytes.extend([0_u8; 16]); // IPropertySet 槽：空标记
    bytes.push(1); // 工厂标志
    bytes.extend([
        // ShapefileWorkspaceFactory {A06ADB96-D95C-11D1-AA81-00C04FA33A15}
        0x96, 0xdb, 0x6a, 0xa0, 0x5c, 0xd9, 0xd1, 0x11, 0xaa, 0x81, 0x00, 0xc0, 0x4f, 0xa3, 0x3a,
        0x15,
    ]);
    bytes.extend(0_i32.to_le_bytes()); // 尾随 int32
    bytes
}

/// FeatureClassName 体（样本镜像：`/Maps:0x30a..0x3ed`，到渲染器槽前）：
/// 版本 2 + 名称 + 空第二名称 + 类型描述 + 数据集名 + int32 ×2 + 选择子 0 +
/// 内嵌 WorkspaceName（头 + 体）+ 版本 >1 选择子 0。
fn feature_class_name_body() -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(encoded(SAMPLE_NAME));
    bytes.extend(encoded("")); // 空第二名称（长度前缀 2 的 6 字节模式）
    bytes.extend(encoded("Shapefile 要素类"));
    bytes.extend(encoded("Shape"));
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(0_u16.to_le_bytes()); // 选择子 0 → IWorkspaceName 槽
    bytes.extend(entry_bytes(&WORKSPACE_NAME_CLASS_ID, SAMPLE_WS_KEY));
    bytes.extend(workspace_name_body());
    bytes.extend(0_u16.to_le_bytes()); // 版本 >1 选择子 0
    bytes
}

/// FeatureLayer 前缀（版本 + 双名称 + short ×3 + 数据源条目字节 + 渲染器
/// 槽位字节 + 尾部）。
fn layer_prefix_bytes(
    version: u16,
    name: &str,
    secondary: &str,
    shorts: [u16; 3],
    data_source: &[u8],
    tail: &[u8],
) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(encoded(name));
    bytes.extend(encoded(secondary));
    bytes.extend(shorts[0].to_le_bytes());
    bytes.extend(shorts[1].to_le_bytes());
    bytes.extend(shorts[2].to_le_bytes());
    bytes.extend(data_source);
    bytes.extend(tail);
    bytes
}

/// 最小合法 SimpleRenderer 体（版本 1）：三个槽位空标记 + short + 双空串 +
/// int32。版本 1 ≤2 无 RotationRenderer/SizeRenderer 尾段。
fn simple_renderer_body_v1() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend([0_u8; 16]); // ISymbol 槽：空标记
    bytes.extend([0_u8; 16]); // ILegendGroup 槽：空标记
    bytes.extend([0_u8; 16]); // ILegendItem 槽：空标记
    bytes.extend(0_i16.to_le_bytes());
    bytes.extend(encoded(""));
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(encoded(""));
    bytes
}

/// 样本镜像的完整已确认链：FCN（新对象，头 + 体）→ 渲染器槽位（SimpleRenderer
/// 新对象头 + 版本 1 最小体）→ +0x144 GUID → 未知尾部。
fn sample_prefix_bytes(tail: &[u8]) -> Vec<u8> {
    let mut data_source = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    data_source.extend(feature_class_name_body());
    let mut renderer = entry_bytes(&SIMPLE_RENDERER_CLASS_ID, 35);
    renderer.extend(simple_renderer_body_v1());
    renderer.extend([0_u8; 16]); // +0x144 原样 GUID（样本形态：16 零）
    renderer.extend(successor_tail());
    let mut bytes = layer_prefix_bytes(
        SAMPLE_VERSION,
        SAMPLE_NAME,
        SAMPLE_SECONDARY_NAME,
        [0xFFFF, 0x0000, 0x0000],
        &data_source,
        &renderer,
    );
    bytes.extend_from_slice(tail);
    bytes
}

/// 最小合法后继字段段（样本形态的空形态）：IHyperlink 计数 0 + IColor 空
/// 标记 + 空串 ×2 + int32 0 + FieldInfo 计数 0。FeatureLayer 版本 >5 时才
/// 有条目名字段，计数 0 无条目。
fn successor_tail() -> Vec<u8> {
    let mut bytes = encoded(""); // +0x154 空串（语义未定）
    bytes.extend(0_f64.to_le_bytes()); // +0x158
    bytes.extend(0_f64.to_le_bytes()); // +0x160
    bytes.extend(0_i16.to_le_bytes()); // +0x168
    bytes.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
    bytes.extend([0_u8; 16]); // IColor：空标记
    bytes.extend(encoded("")); // +0x1a8 空串
    bytes.extend(encoded("")); // +0x184 空串
    bytes.extend(0_i32.to_le_bytes()); // +0x188
    bytes.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
    bytes
}

/// 空数据源 + 空渲染器 + GUID + 最小后继段的最短完整链（停止点
/// Successor(FieldInfoLoopCompleted) 恰在输入末尾）。
fn minimal_full_chain(tail: &[u8]) -> Vec<u8> {
    let mut bytes = layer_prefix_bytes(SAMPLE_VERSION, "层", "", [0; 3], &[0_u8; 16], b"");
    bytes.extend([0_u8; 16]); // 渲染器：空标记
    bytes.extend([0x6A; 16]); // +0x144 原样 GUID
    bytes.extend(successor_tail());
    bytes.extend_from_slice(tail);
    bytes
}

#[test]
fn feature_layer_class_id_constants_match_documented_guids() {
    // FeatureLayer {E663A651-8AAD-11D0-BEC7-00805F7C4268}：esri.clsid.ecfg 声明
    // + FeatureLayer.dll RTTI .?AVFeatureLayer@@ + GetClassID 写出三重印证，
    // 且与样本 /Maps:0x296 对象头逐字节相同（Data1-3 小端、Data4 原序）。
    assert_eq!(
        FEATURE_LAYER_CLASS_ID,
        [
            0x51, 0xa6, 0x63, 0xe6, 0xad, 0x8a, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c,
            0x42, 0x68,
        ]
    );
    // FeatureClassName {198846D0-CA42-11D1-AA7C-00C04FA33A15}：esri.clsid.ecfg
    // 声明 + GdbCore.dll GetClassID 0x709b3a40 写出（第六轮），且与样本
    // /Maps:0x2f6 数据源对象头逐字节相同。
    assert_eq!(
        FEATURE_CLASS_NAME_CLASS_ID,
        [
            0xd0, 0x46, 0x88, 0x19, 0x42, 0xca, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3,
            0x3a, 0x15,
        ]
    );
    // WorkspaceName {5A350011-E371-11D1-AA82-00C04FA33A15}：esri.clsid.ecfg 声明
    // + GdbCore.dll GetClassID 0x70a27910 写出（第六轮），且与样本 /Maps:0x370
    // 内嵌对象头逐字节相同。
    assert_eq!(
        WORKSPACE_NAME_CLASS_ID,
        [
            0x11, 0x00, 0x35, 0x5a, 0x71, 0xe3, 0xd1, 0x11, 0xaa, 0x82, 0x00, 0xc0, 0x4f, 0xa3,
            0x3a, 0x15,
        ]
    );
}

#[test]
fn reads_sample_chain_with_workspace_and_stops_at_renderer() {
    // 样本镜像全链：FCN 体四字符串 + int32 ×2 + 选择子 0 + WorkspaceName 体
    // （路径/空串/类型描述/空 IPropertySet/标志 1/工厂 CLSID/int32 0）+ 版本
    // >1 选择子 0，随后渲染器槽位的 SimpleRenderer 新对象为停止点。
    let bytes = sample_prefix_bytes(b"rest");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();

    assert_eq!(parsed.version, SAMPLE_VERSION);
    assert_eq!(parsed.name, SAMPLE_NAME);
    assert_eq!(parsed.secondary_name, SAMPLE_SECONDARY_NAME);
    assert_eq!(parsed.visibility(), Visibility10_1::True);
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!(
            "数据源应分类为新 FeatureClassName：{:?}",
            parsed.data_source
        );
    };
    assert_eq!(fcn.key, SAMPLE_FCN_KEY);
    assert_eq!(fcn.version, 2);
    assert_eq!(fcn.name, SAMPLE_NAME);
    assert_eq!(fcn.secondary_name, "");
    assert_eq!(fcn.type_description, "Shapefile 要素类");
    assert_eq!(fcn.dataset_name, "Shape");
    assert_eq!((fcn.int32_a, fcn.int32_b), (1, 1));
    assert_eq!(fcn.selector, 0);
    assert_eq!(fcn.version1_selector, Some(0));
    assert_eq!(fcn.stop, None, "样本形态的 FCN 体应完整消费");
    let FeatureClassNameNested10_1::WorkspaceName(WorkspaceNameEntry10_1::NewObject(ws)) =
        &fcn.nested
    else {
        panic!("选择子 0 分支应解析 WorkspaceName 体：{:?}", fcn.nested);
    };
    assert_eq!(ws.key, SAMPLE_WS_KEY);
    assert_eq!(ws.version, 1);
    assert_eq!(ws.path, "..\\22.界址点");
    assert_eq!(ws.secondary_path, "");
    assert_eq!(ws.type_description, "Shapefile Data");
    assert_eq!(
        ws.property_set,
        mxd2qgz::layers::feature_layer::PropertySetSlot10_1::Null
    );
    assert_eq!(ws.factory_flag, Some(1));
    assert_eq!(
        ws.factory_class_id,
        Some([
            0x96, 0xdb, 0x6a, 0xa0, 0x5c, 0xd9, 0xd1, 0x11, 0xaa, 0x81, 0x00, 0xc0, 0x4f, 0xa3,
            0x3a, 0x15,
        ]),
        "工厂 CLSID 原样保留（ShapefileWorkspaceFactory）"
    );
    assert_eq!(ws.trailing_int32, Some(0));
    assert_eq!(ws.stop, None);
    // 渲染器槽位：SimpleRenderer 新对象按 feature_layer_tail 分派读取其闭合
    // 体（版本 1 最小体），随后 +0x144 GUID 读取后到达开放区停止。
    let Some(RendererSlot10_1::SimpleRenderer(simple_renderer)) = &parsed.renderer else {
        panic!("渲染器槽位应解析 SimpleRenderer 体：{:?}", parsed.renderer);
    };
    assert_eq!(simple_renderer.key, 35);
    assert_eq!(simple_renderer.version, 1);
    assert_eq!(simple_renderer.stop, None);
    assert_eq!(parsed.guid_after_renderer, Some([0_u8; 16]));
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::FieldInfoLoopCompleted)
    );
    // 停止点 = 渲染器体 + GUID + 后继段之后（尾部 4 字节保留）。
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(parsed.remaining_bytes, 4);
    // 成功提交：FCN、WS 与 SimpleRenderer 的键均登记。
    assert!(table.contains(SAMPLE_FCN_KEY));
    assert!(table.contains(SAMPLE_WS_KEY));
    assert!(table.contains(35));
    assert_eq!(table.len(), 3);
}

#[test]
fn reads_guid_after_null_renderer_and_stops() {
    // 渲染器槽位为空标记时继续读取 +0x144 原样 GUID，随后停在 SP 异常开放区。
    let bytes = minimal_full_chain(b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(parsed.renderer, Some(RendererSlot10_1::Null));
    assert_eq!(parsed.guid_after_renderer, Some([0x6A; 16]));
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::FieldInfoLoopCompleted)
    );
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
}

#[test]
fn workspace_property_set_new_object_stops_inside_body() {
    // WorkspaceName 体内 IPropertySet 槽的新对象：WS 在该处停止（标志及之后
    // 字段不可达），FCN 与 FeatureLayer 逐层传播同一停止原因。
    let mut data_source = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    let mut fcn = feature_class_name_body();
    // 把体尾「空标记 + 标志 1 + 工厂 CLSID + int32 + 版本 >1 选择子」替换为
    // PropertySet 新对象。
    let ws_tail_start = fcn.len() - (16 + 1 + 16 + 4 + 2);
    fcn.truncate(ws_tail_start);
    fcn.extend(entry_bytes(&[0xB9; 16], 77));
    data_source.extend(&fcn);
    data_source.extend([0_u8; 16]); // 渲染器槽：空标记（不可达，仅占位）
    let bytes = layer_prefix_bytes(SAMPLE_VERSION, "层", "", [0; 3], &data_source, b"");
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!("FCN 应已解析");
    };
    let FeatureClassNameNested10_1::WorkspaceName(WorkspaceNameEntry10_1::NewObject(ws)) =
        &fcn.nested
    else {
        panic!("WS 应已解析");
    };
    assert_eq!(
        ws.stop,
        Some(WorkspaceNameStop10_1::PropertySetNewObject {
            class_id: [0xB9; 16],
            key: 77
        })
    );
    assert_eq!(ws.factory_flag, None);
    assert_eq!(ws.trailing_int32, None);
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::DataSource(FeatureClassNameStop10_1::WorkspacePropertySetNewObject {
            class_id: [0xB9; 16],
            key: 77
        })
    );
    // 游标停在 WS 体的 IPropertySet 对象头起点（回退语义）：其后是渲染器
    // 槽位的 16 字节空标记（不可达）。
    assert_eq!(reader.position(), bytes.len() - 16 - 20);
}

#[test]
fn dataset_name_selector_branch_stops_at_new_object() {
    // 选择子 !=0 分支：IDatasetName 槽的新对象为停止点，版本 >1 选择子不可达。
    let mut body = 2_u16.to_le_bytes().to_vec();
    body.extend(encoded("类"));
    body.extend(encoded(""));
    body.extend(encoded("T"));
    body.extend(encoded("D"));
    body.extend(0_i32.to_le_bytes());
    body.extend(0_i32.to_le_bytes());
    body.extend(1_u16.to_le_bytes()); // 选择子 1 → IDatasetName 槽
    let mut full = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    full.extend(&body);
    full.extend(entry_bytes(&[0xC7; 16], 88)); // IDatasetName 新对象（未知类）
    full.extend(b"opaque");
    let bytes = layer_prefix_bytes(SAMPLE_VERSION, "层", "", [0; 3], &full, b"");
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!("FCN 应已解析");
    };
    assert_eq!(fcn.selector, 1);
    let FeatureClassNameNested10_1::DatasetName(entry) = &fcn.nested else {
        panic!("选择子 1 应进入 IDatasetName 分支：{:?}", fcn.nested);
    };
    assert_eq!(
        *entry,
        DatasetNameEntry10_1::Unsupported {
            class_id: [0xC7; 16],
            key: 88
        }
    );
    assert_eq!(fcn.version1_selector, None);
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::DataSource(FeatureClassNameStop10_1::DatasetNameNewObject {
            class_id: [0xC7; 16],
            key: 88
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
}

#[test]
fn topology_selector_branch_stops_at_new_object() {
    // 版本 >1 且第二选择子非 0：ITopologyName 槽的新对象为停止点。
    let mut body = feature_class_name_body();
    // 体尾的「版本 >1 选择子 0」替换为非 0 + ITopologyName 新对象。
    body.truncate(body.len() - 2);
    body.extend(5_u16.to_le_bytes()); // 第二选择子 5
    body.extend(entry_bytes(&[0xD3; 16], 99));
    body.extend(b"opaque");
    let mut full = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    full.extend(&body);
    let bytes = layer_prefix_bytes(SAMPLE_VERSION, "层", "", [0; 3], &full, b"");
    let mut reader = Reader::new(&bytes);
    let parsed =
        read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!("FCN 应已解析");
    };
    assert_eq!(fcn.version1_selector, Some(5));
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::DataSource(FeatureClassNameStop10_1::TopologyNameNewObject {
            class_id: [0xD3; 16],
            key: 99
        })
    );
    assert_eq!(reader.position(), bytes.len() - b"opaque".len() - 20);
}

#[test]
fn dataset_name_selector_reads_feature_dataset_name_body() {
    // 选择子 !=0 分支：IDatasetName 槽的 FeatureDatasetName 新对象读取其闭合
    // 体（dataset_name 模块：版本 1 + 四字符串 + 标志 + WorkspaceName 体）；
    // 体完整消费后 FCN 的版本 >1 选择子照常读取。
    let mut fdn = 1_u16.to_le_bytes().to_vec();
    fdn.extend(encoded("数据集"));
    fdn.extend(encoded(""));
    fdn.extend(encoded("FD"));
    fdn.extend(encoded("DS"));
    fdn.push(1); // 标志
    fdn.extend(entry_bytes(&WORKSPACE_NAME_CLASS_ID, 60));
    fdn.extend(workspace_name_body());
    let mut body = 2_u16.to_le_bytes().to_vec();
    body.extend(encoded("类"));
    body.extend(encoded(""));
    body.extend(encoded("T"));
    body.extend(encoded("D"));
    body.extend(0_i32.to_le_bytes());
    body.extend(0_i32.to_le_bytes());
    body.extend(1_u16.to_le_bytes()); // 选择子 1 → IDatasetName 槽
    body.extend(entry_bytes(&FEATURE_DATASET_NAME_CLASS_ID, 59));
    body.extend(&fdn);
    body.extend(0_u16.to_le_bytes()); // 版本 >1 选择子 0
    let mut reader = Reader::new(&body);
    let parsed =
        read_feature_class_name_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 58).unwrap();
    let FeatureClassNameNested10_1::DatasetName(DatasetNameEntry10_1::NewObject(body_parsed)) =
        &parsed.nested
    else {
        panic!("选择子 1 应解析 FeatureDatasetName 体：{:?}", parsed.nested);
    };
    // B 轮接线后该槽为七类统一体：FeatureDatasetName 走 FeatureDataset 变体。
    let mxd2qgz::objects::dataset_name_more::DatasetNameMoreBody10_1::FeatureDataset(fdn_parsed) =
        &**body_parsed
    else {
        panic!("FeatureDatasetName 新对象应映射为 FeatureDataset 变体：{body_parsed:?}");
    };
    assert_eq!(fdn_parsed.key, 59);
    assert_eq!(fdn_parsed.version, 1);
    assert_eq!(fdn_parsed.string_a, "数据集");
    assert_eq!(fdn_parsed.flag, 1);
    assert!(fdn_parsed.stop.is_none(), "样本形态的 FDN 体应完整消费");
    assert_eq!(parsed.version1_selector, Some(0));
    assert_eq!(parsed.topology_slot, None);
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn topology_selector_reads_topology_name_body() {
    // 版本 >1 且第二选择子非 0：ITopologyName 槽的 TopologyName 新对象读取
    // 其闭合体（版本 1 + 两字符串 + IDatasetName 槽空标记）。
    let mut tn = 1_u16.to_le_bytes().to_vec();
    tn.extend(encoded("拓扑"));
    tn.extend(encoded(""));
    tn.extend([0_u8; 16]); // IDatasetName 槽：空标记
    let mut body = feature_class_name_body();
    body.truncate(body.len() - 2);
    body.extend(5_u16.to_le_bytes()); // 第二选择子 5
    body.extend(entry_bytes(&TOPOLOGY_NAME_CLASS_ID, 99));
    body.extend(&tn);
    let mut reader = Reader::new(&body);
    let parsed =
        read_feature_class_name_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 58).unwrap();
    let Some(TopologyNameEntry10_1::NewObject(tn_parsed)) = &parsed.topology_slot else {
        panic!(
            "第二选择子应解析 TopologyName 体：{:?}",
            parsed.topology_slot
        );
    };
    assert_eq!(tn_parsed.key, 99);
    assert_eq!(tn_parsed.version, 1);
    assert_eq!(tn_parsed.string_a, "拓扑");
    assert!(tn_parsed.stop.is_none());
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn workspace_body_reader_rejects_version_above_one() {
    // WorkspaceName::Load 的版本上限 1：版本 2 拒绝（原生 `1 < ver`）。
    for version in [2_u16, 7, 0x8000] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded("p"));
        let mut reader = Reader::new(&bytes);
        let error = read_workspace_name_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1)
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "version={version}");
        assert!(error.to_string().contains("WorkspaceName"), "{error}");
        assert_eq!(reader.position(), 0, "version={version}");
    }
}

#[test]
fn feature_class_name_body_reader_rejects_version_above_two() {
    // FeatureClassName::Load 的版本上限 2（原生有符号 `2 < ver`）。
    for version in [3_u16, 9, 0x8000] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded("n"));
        let mut reader = Reader::new(&bytes);
        let error =
            read_feature_class_name_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 1)
                .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "version={version}");
        assert!(error.to_string().contains("FeatureClassName"), "{error}");
        assert_eq!(reader.position(), 0, "version={version}");
    }
}

#[test]
fn version_one_fcn_skips_version_one_selector() {
    // 版本 1（<=1）的 FCN 不读第二选择子：WS 槽消费后体即完整。
    let mut body = 1_u16.to_le_bytes().to_vec();
    body.extend(encoded("类"));
    body.extend(encoded(""));
    body.extend(encoded("T"));
    body.extend(encoded("D"));
    body.extend(2_i32.to_le_bytes());
    body.extend(3_i32.to_le_bytes());
    body.extend(0_u16.to_le_bytes()); // 选择子 0 → WS 空标记
    body.extend([0_u8; 16]);
    let mut reader = Reader::new(&body);
    let parsed =
        read_feature_class_name_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 5).unwrap();
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.version1_selector, None, "版本 1 无第二选择子");
    assert_eq!(parsed.stop, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reads_version_boundaries_zero_and_34() {
    // 无符号比较的接受范围 0..=34：下界 0 与上界 34（样本值）都必须接受；
    // 数据源与渲染器槽均用 16 零字节空标记，聚焦版本边界本身。
    for version in [0_u16, 34] {
        let mut bytes = layer_prefix_bytes(version, "图层", "", [0; 3], &[0_u8; 16], b"");
        bytes.extend([0_u8; 16]); // 渲染器：空标记
        bytes.extend([0x6A; 16]); // GUID
        bytes.extend(successor_tail());
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(parsed.version, version, "version={version}");
        assert_eq!(
            parsed.data_source,
            DataSource10_1::Null,
            "version={version}"
        );
        assert_eq!(
            parsed.stop,
            FeatureLayerStop10_1::Successor(SuccessorStop10_1::FieldInfoLoopCompleted),
            "version={version}"
        );
        assert_eq!(parsed.remaining_bytes, 0, "version={version}");
        assert_eq!(reader.position(), bytes.len(), "version={version}");
    }
}

#[test]
fn rejects_versions_above_34_without_advancing() {
    // >0x22/34 直接拒绝（原生行为）；0x8000.. 位模式在无符号解释下同样超界，
    // 若原生为有符号比较则本读取器更严（符号性假设见模块文档）。
    for version in [35_u16, 100, 0x1234, 0x8000, 0xffff] {
        let bytes = layer_prefix_bytes(version, "图层", "", [0; 3], &[0_u8; 16], b"");
        let mut reader = Reader::new(&bytes);
        let first = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert!(first.to_string().contains("版本"), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
        let second = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn reads_visibility_bit_patterns_with_interpretation_helper() {
    // 可见性按原始 u16 位模式保留：规范值 0xFFFF/0x0000 之外一律 NonCanonical
    // 原样保留，不做「非零即真」假设；其后两个未知 short 同样原样保留。
    for (raw, expected) in [
        (0xFFFF_u16, Visibility10_1::True),
        (0x0000, Visibility10_1::False),
        (0x0001, Visibility10_1::NonCanonical(0x0001)),
        (0x8000, Visibility10_1::NonCanonical(0x8000)),
        (0xFFFE, Visibility10_1::NonCanonical(0xFFFE)),
    ] {
        let mut bytes = layer_prefix_bytes(
            SAMPLE_VERSION,
            "图层",
            "",
            [raw, 0x1234, 0xABCD],
            &[0_u8; 16],
            b"",
        );
        bytes.extend([0_u8; 16]); // 渲染器：空标记
        bytes.extend([0x6A; 16]); // GUID
        bytes.extend(successor_tail());
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(parsed.visibility_raw, raw, "raw={raw:#x}");
        assert_eq!(parsed.visibility(), expected, "raw={raw:#x}");
        assert_eq!(parsed.unknown_short_a, 0x1234, "raw={raw:#x}");
        assert_eq!(parsed.unknown_short_b, 0xABCD, "raw={raw:#x}");
        assert_eq!(parsed.remaining_bytes, 0, "raw={raw:#x}");
    }
}

#[test]
fn reads_null_data_source_without_touching_the_table() {
    // 数据源位置的全零 16 字节是空引用标记（无键、无对象体）：渲染器空标记
    // 与 GUID 正常继续，登记表保持为空。
    let bytes = minimal_full_chain(b"unparsed");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.data_source, DataSource10_1::Null);
    assert_eq!(parsed.remaining_bytes, 8);
    assert_eq!(reader.position(), bytes.len() - 8);
    assert_eq!(reader.read_bytes(8).unwrap(), b"unparsed");
    assert!(table.is_empty());
}

#[test]
fn reads_new_feature_class_name_and_commits_registration() {
    // 新 FCN 对象成功解析后登记表包含 FCN/WS 两键；用同一登记表重读同一字
    // 节时，FCN 条目键已见、分类为回引（无体）——流内无法局部区分回引与新
    // 对象，与原生语义一致（object_refs 模块文档）。
    let bytes = sample_prefix_bytes(b"");
    let mut table = RefTable10_1::new();

    let mut first_reader = Reader::new(&bytes);
    let first = read_feature_layer_prefix_10_1(&mut first_reader, 1024, &mut table).unwrap();
    assert!(matches!(
        first.data_source,
        DataSource10_1::FeatureClassName(_)
    ));
    assert!(table.contains(SAMPLE_FCN_KEY));
    assert!(table.contains(SAMPLE_WS_KEY));
    // 停止点 = SimpleRenderer 体 + GUID 之后（tail 为空，完整消费）。
    assert_eq!(first_reader.position(), bytes.len());
    assert_eq!(first.remaining_bytes, 0);

    // 第二读换用回引形态的独立字节流：回引头之后接渲染器空标记与 GUID
    //（真实流中回引后不会重复出现被引对象的体，直接复用首读字节会把体首
    // 字节误读为渲染器头——该误读本身即「流内无法局部区分」的原生语义）。
    let mut second_bytes = layer_prefix_bytes(
        SAMPLE_VERSION,
        SAMPLE_NAME,
        SAMPLE_SECONDARY_NAME,
        [0xFFFF, 0, 0],
        &entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY),
        &[0_u8; 16],
    );
    second_bytes.extend([0x6A; 16]); // +0x144 GUID
    second_bytes.extend(successor_tail());
    let mut second_reader = Reader::new(&second_bytes);
    let second = read_feature_layer_prefix_10_1(&mut second_reader, 1024, &mut table).unwrap();
    assert_eq!(
        second.data_source,
        DataSource10_1::BackReference {
            class_id: FEATURE_CLASS_NAME_CLASS_ID,
            key: SAMPLE_FCN_KEY,
        }
    );
    // 回引无体：渲染器空标记、GUID 与后继段照常读取。
    assert_eq!(
        second.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::FieldInfoLoopCompleted),
        "回引数据源不阻塞后续槽位"
    );
    assert_eq!(second_reader.position(), second_bytes.len());
    assert_eq!(
        table.len(),
        3,
        "回引不重复登记（首读已登记 FCN/WS/SimpleRenderer 三键）"
    );
}

#[test]
fn reads_back_reference_with_previously_registered_key() {
    // 预置键后，数据源条目只写 20 字节回引头（无体），渲染器与 GUID 继续。
    let mut table = RefTable10_1::new();
    table.register(SAMPLE_FCN_KEY);
    let back_reference_entry = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    let mut bytes = layer_prefix_bytes(
        SAMPLE_VERSION,
        "注记",
        "",
        [0xFFFF, 0, 0],
        &back_reference_entry,
        &[0_u8; 16],
    );
    bytes.extend([0x6A; 16]); // GUID
    bytes.extend(successor_tail());
    bytes.extend(b"tail");
    let mut reader = Reader::new(&bytes);
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.data_source,
        DataSource10_1::BackReference {
            class_id: FEATURE_CLASS_NAME_CLASS_ID,
            key: SAMPLE_FCN_KEY,
        }
    );
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::FieldInfoLoopCompleted)
    );
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(table.len(), 1, "回引不得新增登记");
}

#[test]
fn rejects_unknown_new_class_id_without_advancing_or_registering() {
    // 未知类标识的新对象显式拒绝：本读取器只支持 FeatureClassName 数据源，
    // 且新对象体没有外部长度字段、无法跳过未知类。第一个 GUID 是纯合成标识。
    let synthetic = [
        0x3c, 0x92, 0xf1, 0x48, 0x20, 0xb6, 0x4c, 0xb9, 0x9e, 0x05, 0x31, 0xd8, 0x7a, 0x2e, 0xc4,
        0x66,
    ];
    assert_ne!(synthetic, FEATURE_CLASS_NAME_CLASS_ID);
    let mut entry = synthetic.to_vec();
    entry.extend(SAMPLE_FCN_KEY.to_le_bytes());
    entry.extend(b"opaque body");
    let bytes = layer_prefix_bytes(
        SAMPLE_VERSION,
        SAMPLE_NAME,
        SAMPLE_SECONDARY_NAME,
        [0xFFFF, 0, 0],
        &entry,
        b"",
    );
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let first = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(first.kind(), ErrorKind::Unsupported);
    let message = first.to_string();
    assert!(message.contains("FeatureClassName"), "{message}");
    assert!(message.contains("无法跳过"), "{message}");
    assert!(message.contains("键 33"), "{message}");
    assert_eq!(reader.position(), 0, "游标必须留在入口");
    assert!(!table.contains(SAMPLE_FCN_KEY));
    assert!(table.is_empty());
    let second = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(first.kind(), second.kind(), "重读结果不一致");
    assert_eq!(reader.position(), 0, "重读后游标仍在入口");
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断扫描覆盖每个字段的内部：版本、两个名称、三个 short、FCN 对
    // 象头与体（四字符串、int32 ×2、选择子、WS 头与体、版本 >1 选择子）、渲
    // 染器对象头。任何截断都必须失败、不推进游标、不修改登记表，且重读得
    // 到相同错误类别。
    let full = sample_prefix_bytes(b"");
    // 前缀终点 = 输入末尾（SimpleRenderer 体与 GUID 均为已确认区）。
    let prefix_length = full.len();
    assert!(prefix_length > 180, "完整输入应覆盖所有字段区域");
    // 边界情形：恰好在前缀末尾截断是合法完整输入（停止点即输入末尾）。
    let mut boundary = Reader::new(&full);
    let mut table = RefTable10_1::new();
    let parsed = read_feature_layer_prefix_10_1(&mut boundary, 1024, &mut table).unwrap();
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(boundary.position(), prefix_length);
    assert!(table.contains(SAMPLE_FCN_KEY));
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        let first = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}: 失败不得修改登记表");
        let second = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}: 重读失败仍不得修改登记表");
    }
}

#[test]
fn failure_inside_feature_class_name_body_rolls_back_registration() {
    // 关键事务性用例：条目分类成功（键已按原生语义登记进副本表）之后、FCN
    // 体或 WS 体读取失败（截断），整个调用失败且调用者的登记表不得留下任何
    // 新键。
    let full = sample_prefix_bytes(b"");
    // 截断点取 FCN 头之后、FCN 体内部、WS 头之后、WS 体内部与渲染器头内部。
    let fcn_entry_start = 2 + encoded(SAMPLE_NAME).len() + encoded(SAMPLE_SECONDARY_NAME).len() + 6;
    assert_eq!(
        full[fcn_entry_start..fcn_entry_start + 16],
        FEATURE_CLASS_NAME_CLASS_ID
    );
    let ws_entry_start = fcn_entry_start
        + 20
        + 2
        + encoded(SAMPLE_NAME).len()
        + 6
        + encoded("Shapefile 要素类").len()
        + encoded("Shape").len()
        + 8
        + 2;
    assert_eq!(
        full[ws_entry_start..ws_entry_start + 16],
        WORKSPACE_NAME_CLASS_ID
    );
    for end in [
        fcn_entry_start + 20,
        fcn_entry_start + 22,
        ws_entry_start,
        ws_entry_start + 20 + 4,
        full.len() - 10,
    ] {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).is_err(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(!table.contains(SAMPLE_FCN_KEY), "end={end}");
        assert!(!table.contains(SAMPLE_WS_KEY), "end={end}");
        assert!(table.is_empty(), "end={end}");
    }
}

#[test]
fn rejects_over_limit_lengths_before_reading_or_allocating() {
    // 名称限额边界：样本链最长的字符串声明 32（图层名/要素类名），限额 32
    // 恰好全部可读；少一字节即在读取与分配之前拒绝。
    let bytes = sample_prefix_bytes(b"");
    let mut reader = Reader::new(&bytes);
    assert!(read_feature_layer_prefix_10_1(&mut reader, 32, &mut RefTable10_1::new()).is_ok());
    let mut reader = Reader::new(&bytes);
    let error =
        read_feature_layer_prefix_10_1(&mut reader, 31, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // 限额原样转发给 FCN/WS 体内的字符串：WorkspaceName 路径（20 字节声明）
    // 超限时在读取前拒绝。
    let mut reader = Reader::new(&bytes);
    let error =
        read_feature_layer_prefix_10_1(&mut reader, 18, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // u32::MAX-1（最大偶数声明长度）：有限额时在分配前按限额拒绝；限额
    // 放开时因流过短缺读拒绝；两者都不推进游标、不做无界分配。
    let mut huge = SAMPLE_VERSION.to_le_bytes().to_vec();
    huge.extend((u32::MAX - 1).to_le_bytes());
    huge.extend(b"new");
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_feature_layer_prefix_10_1(&mut reader, usize::MAX, &mut RefTable10_1::new())
            .unwrap_err()
            .kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn reads_feature_layer_header_then_prefix_in_full_chain() {
    // 组合链路：read_header_10_1 确认 FeatureLayer 头（类标识 + 键）→
    // read_feature_layer_prefix_10_1 读取已确认前缀 → 尾部留给调用者。
    let mut bytes = FEATURE_LAYER_CLASS_ID.to_vec();
    bytes.extend(SAMPLE_LAYER_KEY.to_le_bytes());
    bytes.extend(sample_prefix_bytes(b"tail"));
    let mut reader = Reader::new(&bytes);
    let header = read_header_10_1(&mut reader).unwrap();
    let ObjectHeader::Object { class_id, key } = header else {
        panic!("FeatureLayer 对象头不应为 Null：{header:?}");
    };
    assert_eq!(class_id, FEATURE_LAYER_CLASS_ID);
    assert_eq!(key, SAMPLE_LAYER_KEY);
    let mut table = RefTable10_1::new();
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, SAMPLE_VERSION);
    assert_eq!(parsed.name, SAMPLE_NAME);
    assert_eq!(parsed.visibility(), Visibility10_1::True);
    assert!(matches!(
        parsed.data_source,
        DataSource10_1::FeatureClassName(_)
    ));
    // 停止点 = SimpleRenderer 体 + GUID 之后：其后 4 字节尾部未消费。
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

// ---------------------------------------------------------------------------
// 接线轮：WorkspaceName 的 IPropertySet 槽接入 read_property_set_10_1
// （`docs/arcmap-10.1-property-set.md` 建议接入点；样本两实例 1547/2027 为
// 该形态）。正常路径 + PropertySet 体内部失败的事务性。
// ---------------------------------------------------------------------------

/// PropertySet 体（样本实例 2 形态，键 2027）：版本 1 + 计数 1 + 键
/// "DATABASE" + VT_BSTR 路径值。
fn property_set_body() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(encoded("DATABASE"));
    bytes.extend(8_u16.to_le_bytes()); // VT_BSTR
    bytes.extend(encoded("D:\\g"));
    bytes
}

/// WorkspaceName 体，IPropertySet 槽为 PropertySet 新对象（键 2027）。
fn workspace_name_body_with_property_set() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(encoded("..\\22.界址点"));
    bytes.extend(encoded(""));
    bytes.extend(encoded("Shapefile Data"));
    bytes.extend(entry_bytes(
        &mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID,
        2027,
    ));
    bytes.extend(property_set_body());
    bytes.push(1); // 工厂标志
    bytes.extend([0xa5_u8; 16]); // 工厂 CLSID（原样保留）
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

#[test]
fn workspace_property_set_new_object_parses_body_and_completes() {
    // 正常路径：IPropertySet 槽为 PropertySet 新对象 → 体整体解析（版本 +
    // 计数 + [键名 + VT_BSTR 值]），其后的工厂标志/CLSID/int32 可达——样本
    // FCN 第二实例（键 2026 链）在该槽停止的分段由此推进到第二选择子。
    let mut data_source = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    let mut fcn = 2_u16.to_le_bytes().to_vec();
    fcn.extend(encoded(SAMPLE_NAME));
    fcn.extend(encoded(""));
    fcn.extend(encoded("Shapefile 要素类"));
    fcn.extend(encoded("Shape"));
    fcn.extend(1_i32.to_le_bytes());
    fcn.extend(1_i32.to_le_bytes());
    fcn.extend(0_u16.to_le_bytes()); // 选择子 0 → IWorkspaceName 槽
    fcn.extend(entry_bytes(&WORKSPACE_NAME_CLASS_ID, SAMPLE_WS_KEY));
    fcn.extend(workspace_name_body_with_property_set());
    fcn.extend(0_u16.to_le_bytes()); // 版本 >1 选择子 0（接线前不可达）
    data_source.extend(&fcn);
    data_source.extend([0_u8; 16]); // 渲染器槽：空标记
    // WS 体闭合后 FL 继续读 +0x144 GUID 与后继段：补最小后继段（两计数 0）。
    let mut tail = [0x6A_u8; 16].to_vec(); // +0x144 原样 GUID
    tail.extend(encoded("")); // +0x154
    tail.extend(0_f64.to_le_bytes());
    tail.extend(0_f64.to_le_bytes());
    tail.extend(0_i16.to_le_bytes()); // +0x168
    tail.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
    tail.extend([0_u8; 16]); // IColor 槽：空标记
    tail.extend(encoded("")); // +0x1a8
    tail.extend(encoded("")); // +0x184
    tail.extend(0_i32.to_le_bytes()); // +0x188
    tail.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
    let bytes = layer_prefix_bytes(SAMPLE_VERSION, "层", "", [0; 3], &data_source, &tail);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!("FCN 应已解析");
    };
    let FeatureClassNameNested10_1::WorkspaceName(WorkspaceNameEntry10_1::NewObject(ws)) =
        &fcn.nested
    else {
        panic!("WS 应已解析");
    };
    assert_eq!(ws.stop, None, "PropertySet 体闭合后 WS 体完整消费");
    let mxd2qgz::layers::feature_layer::PropertySetSlot10_1::PropertySet(body) = &ws.property_set
    else {
        panic!("槽位应为 PropertySet 体：{:?}", ws.property_set);
    };
    assert_eq!(body.count, 1);
    assert_eq!(body.entries[0].key, "DATABASE");
    assert_eq!(
        body.entries[0].value,
        mxd2qgz::objects::property_set::PropertyValueObjectStream10_1::Inline(
            mxd2qgz::objects::property_set::PropertyValue10_1::String("D:\\g".to_string())
        )
    );
    // 其后字段可达：工厂标志 + CLSID + int32，FCN 选择子 0，渲染器空标记。
    assert_eq!(ws.factory_flag, Some(1));
    assert_eq!(ws.factory_class_id, Some([0xa5; 16]));
    assert_eq!(ws.trailing_int32, Some(0));
    assert_eq!(fcn.version1_selector, Some(0));
    assert_eq!(parsed.renderer, Some(RendererSlot10_1::Null));
    // PropertySet 新对象键 2027 登记（经会话登记表）；FL 推进至后继段终点。
    assert!(table.contains(2027));
    assert_eq!(
        parsed.stop,
        FeatureLayerStop10_1::Successor(SuccessorStop10_1::FieldInfoLoopCompleted)
    );
}

#[test]
fn workspace_property_set_truncated_body_fails_without_advancing() {
    // 异常路径：PropertySet 体被截断 → 整体失败（错误传播，事务性回滚由调
    // 用者 trial 保证），游标不动。构造含截断 PropertySet 体的 FCN。
    let mut fcn = 2_u16.to_le_bytes().to_vec();
    fcn.extend(encoded(SAMPLE_NAME));
    fcn.extend(encoded(""));
    fcn.extend(encoded("Shapefile 要素类"));
    fcn.extend(encoded("Shape"));
    fcn.extend(1_i32.to_le_bytes());
    fcn.extend(1_i32.to_le_bytes());
    fcn.extend(0_u16.to_le_bytes());
    fcn.extend(entry_bytes(&WORKSPACE_NAME_CLASS_ID, SAMPLE_WS_KEY));
    // WS 体到 IPropertySet 槽为止（版本 + 三串）。
    let mut ws = 1_u16.to_le_bytes().to_vec();
    ws.extend(encoded("..\\22.界址点"));
    ws.extend(encoded(""));
    ws.extend(encoded("Shapefile Data"));
    ws.extend(entry_bytes(
        &mxd2qgz::objects::property_set::PROPERTY_SET_CLASS_ID,
        2027,
    ));
    ws.extend(&property_set_body()[..5]); // 截断的 PropertySet 体
    fcn.extend(&ws);
    fcn.extend(0_u16.to_le_bytes());
    let mut data_source = entry_bytes(&FEATURE_CLASS_NAME_CLASS_ID, SAMPLE_FCN_KEY);
    data_source.extend(&fcn);
    let bytes = layer_prefix_bytes(SAMPLE_VERSION, "层", "", [0; 3], &data_source, b"");
    let mut reader = Reader::new(&bytes);
    let error = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
        .expect_err("截断的 PropertySet 体应失败");
    assert_eq!(error.kind(), ErrorKind::InvalidData, "{error}");
    assert_eq!(reader.position(), 0, "失败不得推进游标");
}

#[test]
fn topology_selector_checks_the_registered_reference_class() {
    use mxd2qgz::storage::object_refs::read_object_entry_10_1;

    let mut bytes = feature_class_name_body();
    bytes.truncate(bytes.len() - 2);
    bytes.extend(5_u16.to_le_bytes());
    let slot_offset = bytes.len();

    for (registered, header) in [
        (TOPOLOGY_NAME_CLASS_ID, WORKSPACE_NAME_CLASS_ID),
        (WORKSPACE_NAME_CLASS_ID, TOPOLOGY_NAME_CLASS_ID),
    ] {
        let mut refs = RefTable10_1::new();
        let definition = entry_bytes(&registered, 99);
        read_object_entry_10_1(&mut Reader::new(&definition), &mut refs).unwrap();
        bytes.truncate(slot_offset);
        bytes.extend(entry_bytes(&header, 99));
        bytes.push(0x42);
        let mut reader = Reader::new(&bytes);
        let name = read_feature_class_name_body_10_1(&mut reader, 1024, &mut refs, 58).unwrap();
        if registered == TOPOLOGY_NAME_CLASS_ID {
            assert_eq!(name.stop, None);
            assert_eq!(
                name.topology_slot,
                Some(TopologyNameEntry10_1::BackReference {
                    class_id: TOPOLOGY_NAME_CLASS_ID,
                    key: 99,
                })
            );
            assert_eq!(reader.read_bytes(1).unwrap(), &[0x42]);
        } else {
            assert_eq!(
                name.stop,
                Some(FeatureClassNameStop10_1::TopologyNameBackReference {
                    class_id: WORKSPACE_NAME_CLASS_ID,
                    key: 99,
                })
            );
            assert_eq!(reader.position(), slot_offset);
        }
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(99));
        assert!(refs.contains(34));
    }
}
