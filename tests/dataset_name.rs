//! `dataset_name` 读取器的合成输入测试。
//!
//! 字段序证据见 `src/dataset_name.rs` 模块文档（GdbCore.dll
//! `FdbFeatureDatasetName::Load 0x709b7c90` / `TopologyName::Load 0x7095e5c0`，
//! 2026-11-14 本轮闭合）。两个类的新对象体在仓库参考样本 /Maps 流中均未
//! 出现（CLSID 0 命中，见研究记录），全部用例使用合成输入、无字节佐证；
//! 样本中 FCN 选择子恒为 0（WorkspaceName 分支），本模块覆盖的分支仅由
//! 反编译证据支撑。

use mxd2qgz::dataset_name::{
    DatasetNameSlot10_1, FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameStop10_1,
    TOPOLOGY_NAME_CLASS_ID, TopologyNameStop10_1, WorkspaceNameSlot10_1,
    read_feature_dataset_name_body_10_1, read_topology_name_body_10_1,
};
use mxd2qgz::feature_layer::WORKSPACE_NAME_CLASS_ID;
use mxd2qgz::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

/// 合成 FeatureDatasetName 对象键。
const FDN_KEY: u32 = 90;
/// 合成 TopologyName 对象键。
const TN_KEY: u32 = 91;
/// 合成 WorkspaceName 对象键。
const WS_KEY: u32 = 92;
/// 合成 PropertySet 类标识（非全零即可触发新对象分类；真实 PropertySet
/// 体字段序未闭合）。
const PROPERTY_SET_CLASS_ID: [u8; 16] = [0x5a; 16];
/// 合成未知类标识（既非 WorkspaceName 也非 FeatureDatasetName）。
const UNKNOWN_CLASS_ID: [u8; 16] = [0xa5; 16];

/// AfCore String::Write 布局的合法输入：u32 字节长度前缀 + 含终止零的
/// UTF-16LE。
fn encoded(text: &str) -> Vec<u8> {
    let units: Vec<_> = text.encode_utf16().collect();
    let mut bytes = ((units.len() * 2 + 2) as u32).to_le_bytes().to_vec();
    for unit in units {
        bytes.extend(unit.to_le_bytes());
    }
    bytes.extend(0_u16.to_le_bytes());
    bytes
}

/// 空字符串（长度前缀 2 的 6 字节模式，勿误读为计数）。
fn empty_string() -> Vec<u8> {
    encoded("")
}

/// 追加嵌套对象条目头（16 字节类标识 + 4 字节键）。
fn push_entry_header(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 可完整消费的 WorkspaceName 体（版本 1：路径 + 空第二路径 + 类型描述 +
/// IPropertySet 空标记 + 标志 0（无工厂 CLSID）+ int32 0）。返回（字节,
/// 体长）。
fn full_workspace_body() -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(encoded("D:\\测试.gdb"));
    bytes.extend(empty_string());
    bytes.extend(encoded("File Geodatabase"));
    bytes.extend([0_u8; 16]); // IPropertySet 空标记（16 字节零，无键）
    bytes.push(0); // 工厂标志 0：无 16 字节工厂 CLSID
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

/// 在 WorkspaceName 体内 IPropertySet 槽位携带新对象（体未闭合）的体：
/// 返回（字节, 属性集槽位在体内的偏移）。
fn workspace_body_with_propset_new() -> (Vec<u8>, usize) {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(encoded("D:\\topo.gdb"));
    bytes.extend(empty_string());
    bytes.extend(encoded("File Geodatabase"));
    let slot = bytes.len();
    push_entry_header(&mut bytes, &PROPERTY_SET_CLASS_ID, 93);
    (bytes, slot)
}

/// 合成 FeatureDatasetName 体（版本 + 四字符串 + 标志 + 槽位尾段由调用者
/// 追加）。返回（字节, 槽位起点偏移）。
fn feature_dataset_prefix(version: u16, flag: u8) -> (Vec<u8>, usize) {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(encoded("数据集"));
    bytes.extend(empty_string());
    bytes.extend(encoded("File Geodatabase Feature Dataset"));
    bytes.extend(encoded("拓扑数据集"));
    bytes.push(flag);
    let slot = bytes.len();
    (bytes, slot)
}

/// 合成 TopologyName 体前段（版本 + 两字符串）。返回（字节, 槽位起点偏移）。
fn topology_prefix(version: u16) -> (Vec<u8>, usize) {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend(encoded("拓扑"));
    bytes.extend(empty_string());
    let slot = bytes.len();
    (bytes, slot)
}

/// 读取一个对象条目并在类标识匹配时解析 FeatureDatasetName 体。
fn read_fdn_entry(
    bytes: &[u8],
    table: &mut RefTable10_1,
) -> std::io::Result<(u32, mxd2qgz::dataset_name::FeatureDatasetNameBody10_1)> {
    let mut reader = Reader::new(bytes);
    match read_object_entry_10_1(&mut reader, table)? {
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == FEATURE_DATASET_NAME_CLASS_ID =>
        {
            let body = read_feature_dataset_name_body_10_1(&mut reader, usize::MAX, table, key)?;
            Ok((key, body))
        }
        other => panic!("合成输入应为 FeatureDatasetName 新对象：{other:?}"),
    }
}

/// 读取一个对象条目并在类标识匹配时解析 TopologyName 体。
fn read_tn_entry(
    bytes: &[u8],
    table: &mut RefTable10_1,
) -> std::io::Result<(u32, mxd2qgz::dataset_name::TopologyNameBody10_1)> {
    let mut reader = Reader::new(bytes);
    match read_object_entry_10_1(&mut reader, table)? {
        ObjectEntry10_1::NewObject { class_id, key } if class_id == TOPOLOGY_NAME_CLASS_ID => {
            let body = read_topology_name_body_10_1(&mut reader, usize::MAX, table, key)?;
            Ok((key, body))
        }
        other => panic!("合成输入应为 TopologyName 新对象：{other:?}"),
    }
}

/// 完整 TN→FDN→WS(new) 链的字节与游标锚点。
fn full_topology_chain() -> (Vec<u8>, usize, usize) {
    // FDN 头 + 体（含 WS 头 + 体）
    let (mut fdn, fdn_ws_slot) = feature_dataset_prefix(1, 1);
    push_entry_header(&mut fdn, &WORKSPACE_NAME_CLASS_ID, WS_KEY);
    fdn.extend(full_workspace_body());
    let fdn_ws_header = fdn_ws_slot;
    // TN 头 + 体（含 FDN 头 + 体）
    let (mut tn, tn_slot) = topology_prefix(1);
    push_entry_header(&mut tn, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    tn.extend(fdn.clone());
    // 锚点：TN 的 IDatasetName 槽起点、FDN 的 IWorkspaceName 槽起点（TN 流内）
    let tn_dataset_slot = tn_slot;
    let tn_fdnd_ws_slot = tn_slot + 20 + fdn_ws_header;
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &TOPOLOGY_NAME_CLASS_ID, TN_KEY);
    bytes.extend(tn);
    (bytes, tn_dataset_slot + 20, tn_fdnd_ws_slot + 20)
}

#[test]
fn reads_full_feature_dataset_name_chain_with_workspace() {
    let (mut body, ws_slot) = feature_dataset_prefix(1, 1);
    push_entry_header(&mut body, &WORKSPACE_NAME_CLASS_ID, WS_KEY);
    body.extend(full_workspace_body());
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    bytes.extend(body);

    let mut table = RefTable10_1::new();
    let (key, parsed) = read_fdn_entry(&bytes, &mut table).unwrap();
    assert_eq!(key, FDN_KEY);
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.string_a, "数据集");
    assert_eq!(parsed.string_b, "");
    assert_eq!(parsed.string_c, "File Geodatabase Feature Dataset");
    assert_eq!(parsed.string_d, "拓扑数据集");
    assert_eq!(parsed.flag, 1);
    let WorkspaceNameSlot10_1::WorkspaceName(ws) = &parsed.workspace else {
        panic!("工作空间槽应解析 WorkspaceName 体：{:?}", parsed.workspace);
    };
    assert_eq!(ws.version, 1);
    assert_eq!(ws.path, "D:\\测试.gdb");
    assert_eq!(ws.stop, None);
    assert_eq!(parsed.stop, None, "链完整消费不应停止");
    // 登记：FDN 键 + WS 键均已提交。
    assert!(table.contains(FDN_KEY));
    assert!(table.contains(WS_KEY));
    assert_eq!(table.len(), 2);
    // 停止点后无剩余字节：解析终结于流尾（新表重读，避免首读已登记的键把
    // 嵌套条目变成回引）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    read_feature_dataset_name_body_10_1(&mut reader, usize::MAX, &mut table, FDN_KEY).unwrap();
    assert_eq!(reader.remaining(), 0);
    // 槽位锚点应位于工作空间对象头起点（游标事务性的参照）。
    assert_eq!(
        ws_slot,
        2 + encoded("数据集").len()
            + empty_string().len()
            + encoded("File Geodatabase Feature Dataset").len()
            + encoded("拓扑数据集").len()
            + 1
    );
}

#[test]
fn feature_dataset_name_workspace_null_and_back_reference_consume_no_body() {
    // 空标记：16 字节零，无键、无体。
    let (mut body, _) = feature_dataset_prefix(1, 0);
    body.extend([0_u8; 16]);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    bytes.extend(body);
    let mut table = RefTable10_1::new();
    let (_, parsed) = read_fdn_entry(&bytes, &mut table).unwrap();
    assert_eq!(parsed.workspace, WorkspaceNameSlot10_1::Null);
    assert_eq!(parsed.stop, None);
    assert_eq!(table.len(), 1, "空标记不产生新键");

    // 回引：键预先登记（登记先于持久化），条目无体。
    let (mut body, _) = feature_dataset_prefix(1, 0);
    push_entry_header(&mut body, &WORKSPACE_NAME_CLASS_ID, WS_KEY);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    bytes.extend(body);
    let mut table = RefTable10_1::new();
    table.register(WS_KEY);
    let (_, parsed) = read_fdn_entry(&bytes, &mut table).unwrap();
    assert_eq!(
        parsed.workspace,
        WorkspaceNameSlot10_1::BackReference {
            class_id: WORKSPACE_NAME_CLASS_ID,
            key: WS_KEY
        }
    );
    assert_eq!(parsed.stop, None);
    assert_eq!(table.len(), 2);
}

#[test]
fn feature_dataset_name_unknown_workspace_class_stops_and_rolls_back() {
    let (mut body, ws_slot) = feature_dataset_prefix(1, 1);
    push_entry_header(&mut body, &UNKNOWN_CLASS_ID, WS_KEY);
    body.extend([0xee; 8]); // 假想的未知类体：读取器不得消费
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    let stream_len = bytes.len() + body.len();
    bytes.extend(body);

    let mut table = RefTable10_1::new();
    let (_, parsed) = read_fdn_entry(&bytes, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(FeatureDatasetNameStop10_1::WorkspaceUnsupported {
            class_id: UNKNOWN_CLASS_ID,
            key: WS_KEY
        })
    );
    assert_eq!(
        parsed.workspace,
        WorkspaceNameSlot10_1::Unsupported {
            class_id: UNKNOWN_CLASS_ID,
            key: WS_KEY
        }
    );
    // 游标回退到工作空间槽位起点（对象头之后 + 前缀长度），未知类键不登记
    //（新表重读，避免首读登记影响分类）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    read_feature_dataset_name_body_10_1(&mut reader, usize::MAX, &mut table, FDN_KEY).unwrap();
    assert_eq!(reader.position(), 20 + ws_slot);
    assert_eq!(reader.remaining(), stream_len - 20 - ws_slot);
    assert!(table.contains(FDN_KEY));
    assert!(!table.contains(WS_KEY), "停止回退不保留未知类键");
}

#[test]
fn feature_dataset_name_propset_stop_propagates_with_cursor_at_slot() {
    let (mut body, _) = feature_dataset_prefix(1, 1);
    push_entry_header(&mut body, &WORKSPACE_NAME_CLASS_ID, WS_KEY);
    let (ws, propset_slot) = workspace_body_with_propset_new();
    let ws_len = ws.len();
    body.extend(ws);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    bytes.extend(body);
    let propset_abs = bytes.len() - ws_len + propset_slot;

    let mut table = RefTable10_1::new();
    let (_, parsed) = read_fdn_entry(&bytes, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(FeatureDatasetNameStop10_1::WorkspacePropertySetNewObject {
            class_id: PROPERTY_SET_CLASS_ID,
            key: 93
        })
    );
    // 游标停在 WorkspaceName 读取器的停止点（属性集槽位起点；新表重读）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    read_feature_dataset_name_body_10_1(&mut reader, usize::MAX, &mut table, FDN_KEY).unwrap();
    assert_eq!(reader.position(), propset_abs);
    // FDN 与 WS 键保留，属性集键不登记。
    assert!(table.contains(FDN_KEY));
    assert!(table.contains(WS_KEY));
    assert!(!table.contains(93));
}

#[test]
fn feature_dataset_name_version_gate_is_signed_and_strict() {
    for (version, ok) in [(0_u16, true), (1, true), (2, false), (0xFFFF, false)] {
        let (mut body, _) = feature_dataset_prefix(version, 0);
        body.extend([0_u8; 16]);
        let mut bytes = Vec::new();
        push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
        bytes.extend(body);
        let mut table = RefTable10_1::new();
        let result = read_fdn_entry(&bytes, &mut table);
        assert_eq!(result.is_ok(), ok, "版本 {version}（0x{version:04x}）");
        if !ok {
            let error = result.unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert!(error.to_string().contains("FeatureDatasetName"), "{error}");
            // 条目分类登记（调用方语义）保留，体读取器失败不新增登记。
            assert_eq!(table.len(), 1, "失败不提交体内登记");
        }
    }
}

#[test]
fn feature_dataset_name_failures_keep_cursor_and_table_unchanged() {
    let (mut body, _) = feature_dataset_prefix(1, 1);
    push_entry_header(&mut body, &WORKSPACE_NAME_CLASS_ID, WS_KEY);
    body.extend(full_workspace_body());
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    bytes.extend(body);
    // 逐前缀截断：严格前缀全部失败（完整链在流尾才闭合），失败游标停在
    // 对象头之后（条目分类已消费 20 字节）、登记表保持仅 FDN 键。
    for end in 0..bytes.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes[..end]);
        let classified = read_object_entry_10_1(&mut reader, &mut table);
        if classified.is_err() {
            continue; // 对象头本身截断：由 object_stream/object_refs 契约覆盖
        }
        let before = reader.position();
        let body_len = end - before;
        let result =
            read_feature_dataset_name_body_10_1(&mut reader, usize::MAX, &mut table, FDN_KEY);
        assert!(result.is_err(), "截断前缀 0x{end:x} 应失败");
        assert_eq!(reader.position(), before, "失败不推进游标（体起点）");
        assert_eq!(reader.remaining(), body_len);
        assert_eq!(table.len(), 1, "失败不提交体内登记");
        assert!(table.contains(FDN_KEY));
    }
    // 字符串超限同样事务性：限额小于第四个字符串声明长度。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    let before = reader.position();
    let limit = encoded("拓扑数据集").len() - 2;
    assert!(read_feature_dataset_name_body_10_1(&mut reader, limit, &mut table, FDN_KEY).is_err());
    assert_eq!(reader.position(), before);
    assert_eq!(table.len(), 1);
}

#[test]
fn reads_full_topology_name_chain_with_nested_feature_dataset() {
    let (bytes, _, _) = full_topology_chain();
    let mut table = RefTable10_1::new();
    let (key, parsed) = read_tn_entry(&bytes, &mut table).unwrap();
    assert_eq!(key, TN_KEY);
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.string_a, "拓扑");
    assert_eq!(parsed.string_b, "");
    let DatasetNameSlot10_1::FeatureDataset(fdn) = &parsed.dataset else {
        panic!("数据集槽应解析 FeatureDatasetName 体：{:?}", parsed.dataset);
    };
    assert_eq!(fdn.version, 1);
    assert!(matches!(
        fdn.workspace,
        WorkspaceNameSlot10_1::WorkspaceName(_)
    ));
    assert_eq!(fdn.stop, None);
    assert_eq!(parsed.stop, None);
    assert!(table.contains(TN_KEY));
    assert!(table.contains(FDN_KEY));
    assert!(table.contains(WS_KEY));
    assert_eq!(table.len(), 3);
    // 新表重读：完整链无剩余字节（避免首读登记把嵌套条目变成回引）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    read_topology_name_body_10_1(&mut reader, usize::MAX, &mut table, TN_KEY).unwrap();
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn topology_name_dataset_slot_null_back_reference_and_unknown() {
    // 空标记。
    let (mut body, _) = topology_prefix(1);
    body.extend([0_u8; 16]);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &TOPOLOGY_NAME_CLASS_ID, TN_KEY);
    bytes.extend(body);
    let mut table = RefTable10_1::new();
    let (_, parsed) = read_tn_entry(&bytes, &mut table).unwrap();
    assert_eq!(parsed.dataset, DatasetNameSlot10_1::Null);
    assert_eq!(parsed.stop, None);

    // 回引（键预先登记）。
    let (mut body, _) = topology_prefix(1);
    push_entry_header(&mut body, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &TOPOLOGY_NAME_CLASS_ID, TN_KEY);
    bytes.extend(body);
    let mut table = RefTable10_1::new();
    table.register(FDN_KEY);
    let (_, parsed) = read_tn_entry(&bytes, &mut table).unwrap();
    assert_eq!(
        parsed.dataset,
        DatasetNameSlot10_1::BackReference {
            class_id: FEATURE_DATASET_NAME_CLASS_ID,
            key: FDN_KEY
        }
    );
    assert_eq!(parsed.stop, None);

    // 未知类（其他 IDatasetName 实现类未闭合）：停止并回退到槽位起点。
    let (mut body, slot) = topology_prefix(1);
    push_entry_header(&mut body, &UNKNOWN_CLASS_ID, FDN_KEY);
    body.extend([0xdd; 6]);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &TOPOLOGY_NAME_CLASS_ID, TN_KEY);
    bytes.extend(body);
    let mut table = RefTable10_1::new();
    let (_, parsed) = read_tn_entry(&bytes, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(TopologyNameStop10_1::DatasetNameUnsupported {
            class_id: UNKNOWN_CLASS_ID,
            key: FDN_KEY
        })
    );
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    read_topology_name_body_10_1(&mut reader, usize::MAX, &mut table, TN_KEY).unwrap();
    assert_eq!(reader.position(), 20 + slot);
    assert!(table.contains(TN_KEY));
    assert!(!table.contains(FDN_KEY));
}

#[test]
fn topology_name_version_gate_is_signed_and_strict() {
    for (version, ok) in [(0_u16, true), (1, true), (2, false), (0xFFFF, false)] {
        let (mut body, _) = topology_prefix(version);
        body.extend([0_u8; 16]);
        let mut bytes = Vec::new();
        push_entry_header(&mut bytes, &TOPOLOGY_NAME_CLASS_ID, TN_KEY);
        bytes.extend(body);
        let mut table = RefTable10_1::new();
        let result = read_tn_entry(&bytes, &mut table);
        assert_eq!(result.is_ok(), ok, "版本 {version}（0x{version:04x}）");
        if !ok {
            let error = result.unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert!(error.to_string().contains("TopologyName"), "{error}");
            // 条目分类登记（调用方语义）保留，体读取器失败不新增登记。
            assert_eq!(table.len(), 1, "失败不提交体内登记");
        }
    }
}

#[test]
fn topology_name_nested_stop_propagates_to_surface() {
    // TN → FDN → WS(属性集新对象)：FDN 停止原因透传，游标停在属性集槽位。
    let (mut fdn, _) = feature_dataset_prefix(1, 1);
    push_entry_header(&mut fdn, &WORKSPACE_NAME_CLASS_ID, WS_KEY);
    let (ws, propset_slot) = workspace_body_with_propset_new();
    fdn.extend(ws);
    let (mut tn, _) = topology_prefix(1);
    push_entry_header(&mut tn, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    tn.extend(fdn);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &TOPOLOGY_NAME_CLASS_ID, TN_KEY);
    bytes.extend(tn);
    // 属性集槽位绝对偏移 = TN 头 20 + TN 前段 + FDN 头 20 + FDN 前段 + WS 头 20 + 槽内偏移
    let propset_abs = 20
        + topology_prefix(1).0.len()
        + 20
        + feature_dataset_prefix(1, 1).0.len()
        + 20
        + propset_slot;

    let mut table = RefTable10_1::new();
    let (_, parsed) = read_tn_entry(&bytes, &mut table).unwrap();
    assert_eq!(
        parsed.stop,
        Some(TopologyNameStop10_1::FeatureDataset(
            FeatureDatasetNameStop10_1::WorkspacePropertySetNewObject {
                class_id: PROPERTY_SET_CLASS_ID,
                key: 93
            }
        ))
    );
    // 新表重读：游标停在属性集槽位起点（WorkspaceName 读取器的停止点）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    read_topology_name_body_10_1(&mut reader, usize::MAX, &mut table, TN_KEY).unwrap();
    assert_eq!(reader.position(), propset_abs);
    assert!(table.contains(TN_KEY) && table.contains(FDN_KEY) && table.contains(WS_KEY));
    assert!(!table.contains(93));
}

#[test]
fn topology_name_truncation_is_transactional_and_limit_forwards() {
    let (bytes, _, _) = full_topology_chain();
    for end in 0..bytes.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes[..end]);
        let classified = read_object_entry_10_1(&mut reader, &mut table);
        if classified.is_err() {
            continue;
        }
        let before = reader.position();
        let result = read_topology_name_body_10_1(&mut reader, usize::MAX, &mut table, TN_KEY);
        assert!(result.is_err(), "截断前缀 0x{end:x} 应失败");
        assert_eq!(reader.position(), before);
        // 失败回滚后登记表只保留条目分类已提交的 TN 键。
        assert!(table.contains(TN_KEY));
    }
    // 限额转发：TN 自身第一个字符串（“拓扑”声明 8 字节）在限额 4 下于进入
    // 嵌套 FDN 之前失败，整体事务性（游标不动、体内不新增登记）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    read_object_entry_10_1(&mut reader, &mut table).unwrap();
    let before = reader.position();
    assert!(read_topology_name_body_10_1(&mut reader, 4, &mut table, TN_KEY).is_err());
    assert_eq!(reader.position(), before);
    assert_eq!(table.len(), 1);
}

#[test]
fn non_canonical_flag_byte_is_preserved_verbatim() {
    // 原生按 bool 存储；非 0/1 位模式不做解释，原样保留（工作空间槽走空
    // 标记以隔离该字段）。
    let (mut body, _) = feature_dataset_prefix(1, 0x5A);
    body.extend([0_u8; 16]);
    let mut bytes = Vec::new();
    push_entry_header(&mut bytes, &FEATURE_DATASET_NAME_CLASS_ID, FDN_KEY);
    bytes.extend(body);
    let mut table = RefTable10_1::new();
    let (_, parsed) = read_fdn_entry(&bytes, &mut table).unwrap();
    assert_eq!(parsed.flag, 0x5A);
    assert_eq!(parsed.stop, None);
}
