use mxd2qgz::layer_list::{
    BodyBoundBasis, COMPOSITE_GRAPHICS_LAYER_CLASS_ID, FEATURE_LAYER_CLASS_ID,
    GROUP_LAYER_CLASS_ID, LayerEntryObject, LayerListStop, read_layer_list_10_1,
};
use mxd2qgz::map_object::{
    MAP_CLASS_ID, MapSuffixStop, read_map_prefix_10_1, read_map_suffix_10_1,
};
use mxd2qgz::object_refs::{ObjectEntry10_1, RefTable10_1};
use mxd2qgz::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::reader::Reader;
use mxd2qgz::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
};
use std::io::ErrorKind;

/// 样本记载的 Map 架构版本（`/Maps:0x18`）。
const SAMPLE_VERSION: u16 = 51;
/// 样本首个 PCS 对象键（`/Maps:0x30` 键 30）。
const SAMPLE_PCS_KEY: u32 = 0x1e;
/// 样本记载的 7 个域 double（语义未确认，按证据文档原样引用）。
const SAMPLE_SEVEN_DOUBLES: [f64; 7] = [
    31876800.0,
    -10002100.0,
    450265407.00157917,
    -100000.0,
    10000.0,
    -100000.0,
    10000.0,
];
/// 样本记载的 3 个尾随 double。
const SAMPLE_THREE_DOUBLES: [f64; 3] = [0.001, 0.001, 0.001];

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

/// 追加版本 >2 分支的域字符串块（4 组长度前缀字符串 + 1 字节垂向标志）。
fn push_domain_block(bytes: &mut Vec<u8>, strings: &[&[u8]], vertical_flag: u8) {
    assert_eq!(strings.len(), 4);
    for string in strings {
        bytes.extend((string.len() as u32).to_le_bytes());
        bytes.extend(*string);
    }
    bytes.push(vertical_flag);
}

/// 追加版本 >=4 分支的 2 字节标志与 7 个 double。
fn push_precision_block(bytes: &mut Vec<u8>, flags: u16, doubles: &[f64; 7]) {
    bytes.extend(flags.to_le_bytes());
    for value in doubles {
        bytes.extend(value.to_le_bytes());
    }
}

/// 追加版本 >=5 分支的 3 个尾随 double。
fn push_trailing_doubles(bytes: &mut Vec<u8>, doubles: &[f64; 3]) {
    for value in doubles {
        bytes.extend(value.to_le_bytes());
    }
}

/// 样本布局（版本 6）的 PCS 对象体：4 个空域字符串、垂向标志 0、标志
/// 0x0001、样本记载的 7+3 个 double；字节布局与 tests/spatial_reference.rs
/// 的合成体一致（集成测试无法互相引用，此处按同一证据独立构造）。
fn sample_pcs_body(wkt: &str) -> Vec<u8> {
    let mut bytes = (6_u16).to_le_bytes().to_vec();
    bytes.extend((wkt.len() as u32).to_le_bytes());
    bytes.extend(wkt.as_bytes());
    push_domain_block(&mut bytes, &[b"", b"", b"", b""], 0x00);
    push_precision_block(&mut bytes, 0x0001, &SAMPLE_SEVEN_DOUBLES);
    push_trailing_doubles(&mut bytes, &SAMPLE_THREE_DOUBLES);
    bytes
}

/// 与参考样本布局对齐的完整已确认前缀：版本 51 + 「新建数据框」 + 空第二
/// 名称（`/Maps:0x2a` 的 6 字节模式）+ PCS 对象头（键 0x1e）+ 版本 6 PCS
/// 体 + 调用者提供的未知尾部。
fn sample_prefix_bytes(wkt: &str, tail: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(SAMPLE_VERSION.to_le_bytes());
    bytes.extend(encoded("新建数据框"));
    bytes.extend(encoded(""));
    bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    bytes.extend(SAMPLE_PCS_KEY.to_le_bytes());
    bytes.extend(sample_pcs_body(wkt));
    bytes.extend(tail);
    bytes
}

/// 与参考样本长度一致的 455 字节合成 WKT。
fn sample_length_wkt() -> String {
    let prefix = "PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\",";
    let wkt = format!("{prefix}{}]", ".".repeat(455 - prefix.len() - 1));
    assert_eq!(wkt.len(), 455);
    wkt
}

/// 断言嵌套 PCS 解析结果与样本记载逐项一致。
fn assert_sample_pcs(spatial_reference: &ProjectedCoordinateSystem10_1) {
    assert_eq!(spatial_reference.save_version, 6);
    assert_eq!(spatial_reference.wkt.len(), 455);
    let domains = spatial_reference.domains.as_ref().unwrap();
    assert_eq!(domains.strings, [vec![], vec![], vec![], vec![]]);
    assert_eq!(domains.vertical_flag, 0);
    let precision = spatial_reference.precision.as_ref().unwrap();
    assert_eq!(precision.flags, 0x0001);
    assert_eq!(precision.doubles, SAMPLE_SEVEN_DOUBLES);
    assert_eq!(
        spatial_reference.trailing_doubles,
        Some(SAMPLE_THREE_DOUBLES)
    );
}

#[test]
fn map_class_id_constant_matches_documented_guid() {
    // 证据：{E6BDAA76-4D35-11D0-98BE-00805F7CED21}，esri.clsid.ecfg 声明 +
    // Map.dll 0x79482adc 字节宿主 + RTTI .?AVMap@@ 三重印证，且与样本首
    // 对象头 /Maps:0x04..0x13 逐字节相同（Data1-3 小端、Data4 原序）。
    assert_eq!(
        MAP_CLASS_ID,
        [
            0x76, 0xaa, 0xbd, 0xe6, 0x35, 0x4d, 0xd0, 0x11, 0x98, 0xbe, 0x00, 0x80, 0x5f, 0x7c,
            0xed, 0x21,
        ]
    );
    // 与嵌套 PCS 常量不是同一个标识。
    assert_ne!(MAP_CLASS_ID, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
}

#[test]
fn reads_sample_layout_prefix_and_preserves_remaining_bytes() {
    // 样本镜像：版本 51、中文名称、空第二名称、455 字节 WKT 的 PCS 嵌套，
    // 之后是本读取器不解释的未知尾部。
    let wkt = sample_length_wkt();
    let bytes = sample_prefix_bytes(&wkt, b"rest");
    let prefix_length = bytes.len() - 4;
    let mut reader = Reader::new(&bytes);
    let parsed = read_map_prefix_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.version, 51);
    assert_eq!(parsed.name, "新建数据框");
    assert_eq!(parsed.secondary_name, "");
    assert_sample_pcs(parsed.spatial_reference.as_ref().unwrap());
    // 已确认前缀恰好停在 PCS 体末尾；未知尾部保留为 remaining_bytes。
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), prefix_length);
    assert_eq!(reader.read_bytes(4).unwrap(), b"rest");
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reads_version_boundaries_zero_and_51() {
    // 无符号比较的接受范围 0..=51：下界 0 与上界 51（样本值）都必须接受。
    // 空间参考用 16 零字节空头，聚焦版本边界本身。
    for version in [0_u16, 51] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded(""));
        bytes.extend(encoded(""));
        bytes.extend([0_u8; 16]);
        bytes.extend(b"tail");
        let mut reader = Reader::new(&bytes);
        let parsed = read_map_prefix_10_1(&mut reader, 1024).unwrap();
        assert_eq!(parsed.version, version, "version={version}");
        assert_eq!(parsed.name, "");
        assert_eq!(parsed.secondary_name, "");
        assert_eq!(parsed.spatial_reference, None, "version={version}");
        assert_eq!(parsed.remaining_bytes, 4, "version={version}");
        assert_eq!(reader.position(), bytes.len() - 4, "version={version}");
    }
}

#[test]
fn reads_both_names_empty_with_documented_six_byte_pattern() {
    // 文档强调：/Maps:0x2a 的 6 字节 `02 00 00 00 00 00` 是空名称字符串
    // （长度前缀 2 = 仅终止零），不是 int32 计数。构造并核对这一模式。
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(encoded(""));
    bytes.extend(encoded(""));
    bytes.extend([0_u8; 16]);
    let empty_pattern = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(
        bytes
            .windows(6)
            .filter(|window| window == &empty_pattern)
            .count(),
        2,
        "两个空名称都应是 6 字节模式"
    );
    let mut reader = Reader::new(&bytes);
    let parsed = read_map_prefix_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.name, "");
    assert_eq!(parsed.secondary_name, "");
    assert_eq!(parsed.spatial_reference, None);
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reads_distinct_names_in_documented_order() {
    // 第 2/3 字段按顺序读取：obj+0x160 名称在前、obj+0x164 第二名称在后，
    // 不得交换；名称可含代理对 emoji 与非 ASCII 中文。
    for (first, second) in [("Layers", "新建数据框"), ("地图😀", "底图")] {
        let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
        bytes.extend(encoded(first));
        bytes.extend(encoded(second));
        bytes.extend([0_u8; 16]);
        let mut reader = Reader::new(&bytes);
        let parsed = read_map_prefix_10_1(&mut reader, 1024).unwrap();
        assert_eq!(parsed.name, first, "first={first}");
        assert_eq!(parsed.secondary_name, second, "first={first}");
        assert_eq!(parsed.spatial_reference, None, "first={first}");
        assert_eq!(reader.remaining(), 0, "first={first}");
    }
}

#[test]
fn reads_null_spatial_reference_from_sixteen_zero_bytes() {
    // 嵌套位置的全零 16 字节是空头（S_FALSE 约定：只消耗 16 字节、不读键），
    // 解析为 None 且不吞并其后字节。
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(encoded("新建数据框"));
    bytes.extend(encoded(""));
    bytes.extend([0_u8; 16]);
    bytes.extend(b"unparsed");
    let header_end = bytes.len() - 8;
    let mut reader = Reader::new(&bytes);
    let parsed = read_map_prefix_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.spatial_reference, None);
    assert_eq!(parsed.remaining_bytes, 8);
    assert_eq!(reader.position(), header_end);
    assert_eq!(reader.read_bytes(8).unwrap(), b"unparsed");
}

#[test]
fn rejects_versions_above_51_without_advancing() {
    // >0x33/51 直接拒绝（原生行为）；0x8000.. 位模式在无符号解释下同样
    // 超界，若原生为有符号比较则本读取器更严（符号性假设见模块文档）。
    for version in [52_u16, 100, 0x1234, 0x8000, 0xffff] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded(""));
        bytes.extend(encoded(""));
        bytes.extend([0_u8; 16]);
        let mut reader = Reader::new(&bytes);
        let first = read_map_prefix_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(first.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert!(first.to_string().contains("版本"), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
        // 失败可重复：重读得到相同错误类别且游标仍在入口。
        let second = read_map_prefix_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn rejects_unknown_nested_class_id_and_preserves_cursor() {
    // 未知嵌套类标识显式拒绝：本读取器只支持投影坐标系嵌套，且嵌套对象体
    // 无外部长度字段、无法跳过未知类。第一个 GUID 是样本中出现的候选
    // {34B2EF83-F4AC-11D1-A245-080009B6F22B}（FDOGraphicsLayer），第二个是
    // 纯合成标识。
    let sample_candidate = [
        0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6, 0xf2,
        0x2b,
    ];
    let synthetic = [
        0xd8, 0x1c, 0x6c, 0x2e, 0x31, 0xae, 0x44, 0xd8, 0x8e, 0x7a, 0x11, 0x0b, 0x2f, 0x93, 0x7d,
        0xe4,
    ];
    for class_id in [&sample_candidate, &synthetic] {
        assert_ne!(*class_id, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
        let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
        bytes.extend(encoded("新建数据框"));
        bytes.extend(encoded(""));
        bytes.extend(*class_id);
        bytes.extend(0x1f_u32.to_le_bytes());
        bytes.extend(b"opaque body");
        let mut reader = Reader::new(&bytes);
        let first = read_map_prefix_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(first.kind(), ErrorKind::Unsupported, "{class_id:?}");
        let message = first.to_string();
        assert!(message.contains("投影坐标系"), "{message}");
        assert!(message.contains("无法跳过"), "{message}");
        assert!(message.contains("键 31"), "{message}");
        assert_eq!(reader.position(), 0, "游标必须留在入口");
        let second = read_map_prefix_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "重读结果不一致");
        assert_eq!(reader.position(), 0, "重读后游标仍在入口");
    }
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断扫描覆盖每个字段的内部：版本（2 字节）、两个名称的前缀与
    // 载荷、嵌套对象头的 GUID 与键、PCS 体的版本/WKT/域串/标志/double。
    // 任何截断都必须失败、不推进游标，且重读得到相同错误类别。
    let full = sample_prefix_bytes(&sample_length_wkt(), b"rest");
    let prefix_length = full.len() - 4;
    assert!(prefix_length > 600, "完整输入应覆盖所有字段区域");
    // 边界情形：恰好在前缀末尾截断（未知尾部缺失）是合法完整输入——尾部
    // 本就不属于已确认前缀，此处应成功且 remaining_bytes == 0。
    let mut boundary = Reader::new(&full[..prefix_length]);
    let parsed = read_map_prefix_10_1(&mut boundary, 1024).unwrap();
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(boundary.remaining(), 0);
    for end in 0..prefix_length {
        let mut reader = Reader::new(&full[..end]);
        let first = read_map_prefix_10_1(&mut reader, 1024)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 截断前缀未返回错误"));
        assert_eq!(reader.position(), 0, "end={end}");
        let second = read_map_prefix_10_1(&mut reader, 1024)
            .err()
            .unwrap_or_else(|| panic!("end={end}: 重读未返回错误"));
        assert_eq!(first.kind(), second.kind(), "end={end}");
        assert_eq!(reader.position(), 0, "end={end}");
    }
    // 完整输入本身必须成功，未知尾部保留在游标之后。
    let mut reader = Reader::new(&full);
    assert!(read_map_prefix_10_1(&mut reader, 1024).is_ok());
    assert_eq!(reader.remaining(), 4);
}

#[test]
fn rejects_over_limit_lengths_before_reading_or_allocating() {
    // 名称限额边界：声明长度等于限额可读（后接空名称与空头）。
    let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
    bytes.extend(encoded("新建数据框"));
    bytes.extend(encoded(""));
    bytes.extend([0_u8; 16]);
    let name_declared = 12_u32; // 5 字 ×2 + 终止零 2。
    let mut reader = Reader::new(&bytes);
    assert!(read_map_prefix_10_1(&mut reader, name_declared as usize).is_ok());
    // 限额少一字节即在读取与分配之前拒绝。
    let mut reader = Reader::new(&bytes);
    let error = read_map_prefix_10_1(&mut reader, name_declared as usize - 1).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // 限额同样作用于第二名称：第一名称声明 4 字节在限额内，第二名称 6 字
    // 字节超过剩余限额被拒，游标不推进。
    let mut second = SAMPLE_VERSION.to_le_bytes().to_vec();
    second.extend(encoded("A"));
    second.extend(encoded("AB"));
    second.extend([0_u8; 16]);
    let mut reader = Reader::new(&second);
    let error = read_map_prefix_10_1(&mut reader, 4).unwrap_err();
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
        read_map_prefix_10_1(&mut reader, 1024).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&huge);
    assert_eq!(
        read_map_prefix_10_1(&mut reader, usize::MAX)
            .unwrap_err()
            .kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn forwards_max_bytes_limit_to_the_nested_pcs_body() {
    // 调用者限额转发给嵌套 PCS：WKT 长度 455 在限额内可读，少一字节即被
    // PCS 读取器以限额错误拒绝（名称声明 12 与 2 均在两种限额之内）。
    let wkt = sample_length_wkt();
    let bytes = sample_prefix_bytes(&wkt, b"");
    let mut reader = Reader::new(&bytes);
    assert!(read_map_prefix_10_1(&mut reader, 455).is_ok());
    let mut reader = Reader::new(&bytes);
    let error = read_map_prefix_10_1(&mut reader, 454).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert!(error.to_string().contains("WKT"), "{error}");
    assert_eq!(reader.position(), 0);
}

#[test]
fn reads_map_header_then_prefix_in_full_chain() {
    // 组合链路：read_header_10_1 确认 Map 头（类标识 + 键）→
    // read_map_prefix_10_1 读取已确认前缀 → 尾部留给调用者。
    let mut bytes = MAP_CLASS_ID.to_vec();
    bytes.extend(0x1d_u32.to_le_bytes());
    bytes.extend(SAMPLE_VERSION.to_le_bytes());
    bytes.extend(encoded("新建数据框"));
    bytes.extend(encoded(""));
    bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    bytes.extend(SAMPLE_PCS_KEY.to_le_bytes());
    bytes.extend(sample_pcs_body("PROJCS[\"x\"]"));
    bytes.extend(b"tail");
    let mut reader = Reader::new(&bytes);
    let header = read_header_10_1(&mut reader).unwrap();
    let ObjectHeader::Object { class_id, key } = header else {
        panic!("Map 对象头不应为 Null：{header:?}");
    };
    assert_eq!(class_id, MAP_CLASS_ID);
    assert_eq!(key, 0x1d);
    let parsed = read_map_prefix_10_1(&mut reader, 1024).unwrap();
    assert_eq!(parsed.version, 51);
    assert_eq!(parsed.name, "新建数据框");
    assert_eq!(parsed.secondary_name, "");
    assert_eq!(parsed.spatial_reference.unwrap().wkt, "PROJCS[\"x\"]");
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");

    // 反例：顶部对象头是 PCS 标识时类身份不匹配 Map 常量，调用者须拒绝而
    // 不能进入 Map 前缀读取（类身份确认是调用者的职责）。
    let mut other = PROJECTED_COORDINATE_SYSTEM_CLASS_ID.to_vec();
    other.extend(0x1e_u32.to_le_bytes());
    let mut reader = Reader::new(&other);
    match read_header_10_1(&mut reader).unwrap() {
        ObjectHeader::Object { class_id, key } => {
            assert_ne!(class_id, MAP_CLASS_ID);
            assert_eq!(key, 0x1e);
        }
        other_header => panic!("PCS 对象头不应为 Null：{other_header:?}"),
    }
}

// —— 图层列表与 Map 后缀序列（10.1）——
// 合成字节按证据文档独立构造（集成测试与库内测试无法互相引用）：
// 标志 1/2/3、ExtraSafe 前缀 [A][B]、A 观察推定的有界跳过、B 原生同步、
// 空条目、回引、活动索引与各停止点。仓库样本的逐条目对照见
// src/layer_list.rs 模块文档（样本吻合为辅助观察；样本在条目 1 的标志 1
// 新对象体处停止是预期行为——FDOGraphicsLayer 体未反编译、读取器未接入）。

/// 对象头字节：16 字节类标识 + 4 字节键。
fn object_header(class_id: &[u8; 16], key: u32) -> Vec<u8> {
    let mut bytes = class_id.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes
}

/// 样本条目 1 的 FDOGraphicsLayer 类标识（/Maps:0x27a，键 31；类名归属候选）。
fn fdo_graphics_layer_class_id() -> [u8; 16] {
    [
        0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6, 0xf2,
        0x2b,
    ]
}

#[test]
fn layer_list_class_id_constants_match_documented_guids() {
    // 集成点常量的字节复核（内存序：Data1-3 小端、Data4 原序）：
    // FeatureLayer {E663A651-8AAD-11D0-BEC7-00805F7C4268}（样本 /Maps:0x296 键 32）；
    // GroupLayer {EDAD6647-1810-11D1-86AE-0000F8751720}（FeatureLayer.dll RTTI）；
    // CompositeGraphicsLayer {9646BB83-9512-11D2-A2F6-080009B6F22B}
    //（样本 /Maps:0x16fba 键 1553，Map 基本图形层）。
    assert_eq!(
        FEATURE_LAYER_CLASS_ID,
        [
            0x51, 0xa6, 0x63, 0xe6, 0xad, 0x8a, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c,
            0x42, 0x68,
        ]
    );
    assert_eq!(
        GROUP_LAYER_CLASS_ID,
        [
            0x47, 0x66, 0xad, 0xed, 0x10, 0x18, 0xd1, 0x11, 0x86, 0xae, 0x00, 0x00, 0xf8, 0x75,
            0x17, 0x20,
        ]
    );
    assert_eq!(
        COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
        [
            0x83, 0xbb, 0x46, 0x96, 0x12, 0x95, 0xd2, 0x11, 0xa2, 0xf6, 0x08, 0x00, 0x09, 0xb6,
            0xf2, 0x2b,
        ]
    );
}

#[test]
fn layer_list_reads_flags_prefixes_backref_empty_and_active_index() {
    let unknown_class = [0x99_u8; 16];
    let mut bytes = 3_i32.to_le_bytes().to_vec();
    // 条目 0：标志 2，前缀 [A=28][B=0]，未知类新对象键 51，体 8 字节（A-20）。
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(28_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(object_header(&unknown_class, 51));
    bytes.extend([0xee_u8; 8]);
    // 条目 1：标志 2，前缀 [20][0]，对键 51 的回引（无体；样本条目 2 同构）。
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(20_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(object_header(&unknown_class, 51));
    // 条目 2：标志 3，前缀 [0][0]：空条目（原生返回 1 并跳过）。
    bytes.extend(3_u16.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    bytes.extend(0_u32.to_le_bytes());
    // 活动图层索引 + 未知尾部。
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(b"tail");

    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let list = read_layer_list_10_1(&mut reader, 1024, &mut table).unwrap();

    assert_eq!(list.count, 3);
    assert_eq!(list.stop, None);
    assert_eq!(list.active_index, Some(2));
    assert_eq!(list.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
    // 条目 0：新对象（返回前已登记），体按 A 推定有界跳过。
    let first = &list.entries[0];
    assert_eq!(first.flag, 2);
    assert_eq!(first.prefix, Some((28, 0)));
    assert_eq!(first.offset, 4);
    assert!(matches!(
        first.object,
        LayerEntryObject::Entry(ObjectEntry10_1::NewObject { key: 51, .. })
    ));
    let body = first.body.expect("未知类新对象应有有界体");
    assert_eq!(body.len, 8);
    assert_eq!(body.offset, 4 + 2 + 8 + 20);
    assert_eq!(body.basis, BodyBoundBasis::PrefixAInference);
    // 条目 1：同键回引，无体。
    let second = &list.entries[1];
    assert_eq!(second.offset, 42);
    assert!(matches!(
        second.object,
        LayerEntryObject::Entry(ObjectEntry10_1::BackReference { key: 51, .. })
    ));
    assert_eq!(second.body, None);
    // 条目 2：空条目（前缀原样保留）。
    assert_eq!(list.entries[2].object, LayerEntryObject::Empty);
    assert_eq!(list.entries[2].prefix, Some((0, 0)));
    assert_eq!(list.entries[2].offset, 72);
    assert!(table.contains(51));
    assert_eq!(table.len(), 1);
}

#[test]
fn layer_list_b_prefix_seeks_natively_and_rejects_backward_sync() {
    let class_id = [0x77_u8; 16];
    // B=30：跳到「前缀后位置 + 30」（原生 Seek，确定行为）；头 20 + 体 10。
    let mut bytes = 1_i32.to_le_bytes().to_vec();
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(1_u32.to_le_bytes()); // A 原样保留，B!=0 时不参与跳过
    bytes.extend(30_u32.to_le_bytes());
    bytes.extend(object_header(&class_id, 9));
    bytes.extend([0xdd_u8; 10]);
    bytes.extend(7_i32.to_le_bytes()); // 活动索引
    let mut reader = Reader::new(&bytes);
    let list = read_layer_list_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(list.active_index, Some(7));
    assert_eq!(list.entries[0].prefix, Some((1, 30)));
    let body = list.entries[0].body.expect("B!=0 应按原生同步跳过");
    assert_eq!(body.len, 10);
    assert_eq!(body.basis, BodyBoundBasis::NativeSeek);
    assert_eq!(reader.remaining(), 0);

    // B=10 < 头 20：同步终点早于对象头终点（原生 Seek 允许倒退，本子集拒绝）。
    let mut backward = 1_i32.to_le_bytes().to_vec();
    backward.extend(2_u16.to_le_bytes());
    backward.extend(1_u32.to_le_bytes());
    backward.extend(10_u32.to_le_bytes());
    backward.extend(object_header(&class_id, 9));
    backward.extend(b"junk");
    let mut reader = Reader::new(&backward);
    let error = read_layer_list_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("早于当前位置"), "{error}");
    assert_eq!(reader.position(), 0);
}

#[test]
fn layer_list_flag1_new_object_stops_after_header() {
    // 与仓库样本条目 1 同构（/Maps:0x278 标志 1 + FDOGraphicsLayer 键 31）：
    // 停止点在对象头之后（键已登记、体起点即停止点），活动索引不读取。
    // 样本在此停止是预期行为：FDOGraphicsLayer 体未反编译、读取器未接入。
    let class_id = fdo_graphics_layer_class_id();
    let mut bytes = 2_i32.to_le_bytes().to_vec();
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(object_header(&class_id, 31));
    // 样本体首字节形态（版本 11 + int32 3 + 0xffff，不解释）。
    bytes.extend([0x0b, 0x00, 0x03, 0x00, 0x00, 0x00, 0xff, 0xff]);
    bytes.extend(0_i32.to_le_bytes()); // 不会读到的活动索引

    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let list = read_layer_list_10_1(&mut reader, 1024, &mut table).unwrap();

    assert_eq!(list.entries.len(), 1);
    assert_eq!(list.active_index, None);
    assert_eq!(
        list.stop,
        Some(LayerListStop::Flag1NewObjectBody {
            index: 0,
            offset: 4,
            class_id,
            key: 31,
        })
    );
    // 游标停在对象头之后：体与后续条目保留在 remaining 中。
    assert_eq!(reader.position(), 4 + 2 + 20);
    assert_eq!(list.remaining_bytes, bytes.len() - reader.position());
    // 键 31 已登记（登记先于对象体解析，与原生一致）。
    assert!(table.contains(31));
}

#[test]
fn layer_list_rejects_invalid_flags_negative_counts_and_over_limit_counts() {
    let class_id = [0x99_u8; 16];
    // 标志 0/4/0xffff：原生 0x80004005（汇编 0x794036b2 CMP AX,1/2/3）。
    for flag in [0_u16, 4, 0xffff] {
        let mut bytes = 1_i32.to_le_bytes().to_vec();
        bytes.extend(flag.to_le_bytes());
        bytes.extend(object_header(&class_id, 7));
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let error = read_layer_list_10_1(&mut reader, 1024, &mut table).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "flag={flag}");
        assert!(error.to_string().contains("0x80004005"), "{error}");
        assert_eq!(reader.position(), 0, "flag={flag}");
        assert!(table.is_empty(), "flag={flag}");
    }
    // 负计数：原生仅按有符号比较跳过循环，负数语义未知，这里显式拒绝。
    let negative = (-1_i32).to_le_bytes().to_vec();
    let mut reader = Reader::new(&negative);
    let error = read_layer_list_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("负数"), "{error}");
    assert_eq!(reader.position(), 0);
    // 超限计数：预检在任何条目读取之前拒绝。
    let huge = i32::MAX.to_le_bytes().to_vec();
    let mut reader = Reader::new(&huge);
    let error = read_layer_list_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);
}

#[test]
fn layer_list_truncation_and_table_changes_are_transactional() {
    let unknown_class = [0x99_u8; 16];
    let mut full = 1_i32.to_le_bytes().to_vec();
    full.extend(2_u16.to_le_bytes());
    full.extend(28_u32.to_le_bytes());
    full.extend(0_u32.to_le_bytes());
    full.extend(object_header(&unknown_class, 51));
    full.extend([0xee_u8; 8]);
    full.extend(5_i32.to_le_bytes());
    full.extend(b"tail");
    // 逐字节截断到列表自身终点（活动索引末尾）为止：截断进入列表内部的任何
    // 长度都必须失败、不推进游标、不修改登记表；其后的未知尾部不属于列表。
    let list_end = full.len() - 4;
    for end in 0..list_end {
        let mut table = RefTable10_1::new();
        table.register(999);
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_layer_list_10_1(&mut reader, 1024, &mut table).is_err(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
        assert_eq!(table.len(), 1, "end={end}");
        assert!(table.contains(999), "end={end}");
        assert!(!table.contains(51), "end={end}");
    }
    // 恰好截到列表终点是合法完整输入（未知尾部为空）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&full[..list_end]);
    let list = read_layer_list_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(list.remaining_bytes, 0);
    // 完整输入成功：键登记在成功路径提交。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&full);
    assert!(read_layer_list_10_1(&mut reader, 1024, &mut table).is_ok());
    assert!(table.contains(51));
}

/// Map 对象体（样本布局前缀 + 完整后缀链）：返回字节与 WKSEnvelope 起点。
fn map_body_with_full_suffix(wkt: &str) -> (Vec<u8>, usize) {
    let mut bytes = sample_prefix_bytes(wkt, b"");
    bytes.extend(0_i32.to_le_bytes()); // 图层列表计数 0
    bytes.extend((-1_i32).to_le_bytes()); // 活动图层索引（样本 -1）
    bytes.extend([0_u8; 16]); // 基本图形层：空标记
    bytes.extend(0_i32.to_le_bytes()); // MapSurround 计数（样本 0）
    let extent_offset = bytes.len();
    for value in [1.5_f64, 2.5, 3.5, 4.5] {
        bytes.extend(value.to_le_bytes()); // WKSEnvelope
    }
    bytes.extend(9_i32.to_le_bytes()); // 两个 int32（第二个用 -3 验证顺序）
    bytes.extend((-3_i32).to_le_bytes());
    bytes.extend([0_u8; 16]); // IEnvelope：空标记
    bytes.extend(0_i32.to_le_bytes()); // 书签计数（样本 0）
    (bytes, extent_offset)
}

#[test]
fn map_suffix_reads_chain_to_bookmark_loop_and_fills_extent() {
    let (bytes, extent_offset) = map_body_with_full_suffix("PROJCS[\"x\"]");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut table).unwrap();

    assert_eq!(suffix.stop, MapSuffixStop::AfterBookmarks);
    assert_eq!(suffix.remaining_bytes, 0);
    assert_eq!(reader.position(), bytes.len());
    assert_eq!(suffix.layer_list.count, 0);
    assert_eq!(suffix.layer_list.active_index, Some(-1));
    // 基本图形层与 IEnvelope 槽位为空标记（16 零字节；IEnvelope 槽位的 16 字节
    // 消费依据见 map_object 模块文档——样本 0x24f8c 的 20 个零含书签计数 0）。
    assert!(matches!(
        suffix.base_graphics_layer.unwrap().entry,
        ObjectEntry10_1::Null
    ));
    assert_eq!(suffix.map_surround_count, Some(0));
    assert_eq!(suffix.map_surrounds, []);
    let extent = suffix.extent.expect("完整链应到达 WKSEnvelope");
    assert_eq!(
        (extent.xmin, extent.ymin, extent.xmax, extent.ymax),
        (1.5, 2.5, 3.5, 4.5)
    );
    assert_eq!(suffix.extent_offset, Some(extent_offset));
    assert_eq!(suffix.raw_int32_pair, Some((9, -3)));
    assert!(matches!(
        suffix.envelope_object.unwrap().entry,
        ObjectEntry10_1::Null
    ));
    assert_eq!(suffix.bookmark_count, Some(0));
    assert_eq!(suffix.bookmarks, []);
    // 前缀嵌套 PCS 键（样本键 30）已由后缀读取器登记进共享表。
    assert!(table.contains(SAMPLE_PCS_KEY));
}

#[test]
fn map_suffix_shared_table_resolves_slot_back_references() {
    // 基本图形层、IEnvelope 与书签条目写成对前缀 PCS 键 30 的回引（样本中
    // Envelope 对 Map 空间参考的回引同构）：经共享登记表分类为回引，链路
    // 继续到书签循环之后。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    bytes.extend(0_i32.to_le_bytes()); // 图层列表计数 0
    bytes.extend((-1_i32).to_le_bytes()); // 活动索引
    bytes.extend(object_header(
        &PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        SAMPLE_PCS_KEY,
    ));
    bytes.extend(0_i32.to_le_bytes()); // MapSurround 计数 0
    for value in [1.5_f64, 2.5, 3.5, 4.5] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(object_header(&[0x88; 16], SAMPLE_PCS_KEY)); // IEnvelope 回引
    bytes.extend(1_i32.to_le_bytes()); // 书签计数 1
    bytes.extend(object_header(&[0x77; 16], SAMPLE_PCS_KEY)); // 书签条目回引
    bytes.extend(b"tail");

    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut table).unwrap();

    assert_eq!(suffix.stop, MapSuffixStop::AfterBookmarks);
    assert_eq!(suffix.remaining_bytes, 4);
    assert!(matches!(
        suffix.base_graphics_layer.unwrap().entry,
        ObjectEntry10_1::BackReference { key, .. } if key == SAMPLE_PCS_KEY
    ));
    assert!(matches!(
        suffix.envelope_object.unwrap().entry,
        ObjectEntry10_1::BackReference { key, .. } if key == SAMPLE_PCS_KEY
    ));
    assert_eq!(suffix.bookmarks.len(), 1);
    assert!(matches!(
        suffix.bookmarks[0].entry,
        ObjectEntry10_1::BackReference { key, .. } if key == SAMPLE_PCS_KEY
    ));
    assert_eq!(suffix.extent.unwrap().xmin, 1.5);
}

#[test]
fn map_suffix_stops_at_each_unwired_new_object_slot() {
    let prefix_and_empty_list = |bytes: &mut Vec<u8>| {
        bytes.extend(0_i32.to_le_bytes()); // 图层列表计数 0
        bytes.extend((-1_i32).to_le_bytes()); // 活动索引
    };
    // (a) 基本图形层新对象（CompositeGraphicsLayer，样本 0x16fba 键 1553 同款）。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    prefix_and_empty_list(&mut bytes);
    let slot_offset = bytes.len();
    bytes.extend(object_header(&COMPOSITE_GRAPHICS_LAYER_CLASS_ID, 1553));
    bytes.extend([0x55; 16]); // 未接入读取器的体 + 后续字段
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        suffix.stop,
        MapSuffixStop::BaseGraphicsLayerBody {
            offset: slot_offset,
            class_id: COMPOSITE_GRAPHICS_LAYER_CLASS_ID,
            key: 1553,
        }
    );
    assert_eq!(suffix.map_surround_count, None);
    assert_eq!(suffix.extent, None);
    assert_eq!(reader.position(), slot_offset + 20);
    assert_eq!(suffix.remaining_bytes, 16);
    assert!(table.contains(1553));

    // (b) MapSurround 条目新对象：计数后首条目即停止，WKSEnvelope 不读取。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    prefix_and_empty_list(&mut bytes);
    bytes.extend([0_u8; 16]); // 基本图形层空标记
    bytes.extend(1_i32.to_le_bytes()); // MapSurround 计数 1
    let slot_offset = bytes.len();
    bytes.extend(object_header(&[0x66; 16], 5));
    bytes.extend([0x77; 8]);
    let mut reader = Reader::new(&bytes);
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(
        suffix.stop,
        MapSuffixStop::MapSurroundBody {
            index: 0,
            offset: slot_offset,
            class_id: [0x66; 16],
            key: 5,
        }
    );
    assert_eq!(suffix.map_surround_count, Some(1));
    assert_eq!(suffix.extent, None);

    // (c) IEnvelope 槽位新对象：WKSEnvelope 已到达，书签计数不读取。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    prefix_and_empty_list(&mut bytes);
    bytes.extend([0_u8; 16]); // 基本图形层空标记
    bytes.extend(0_i32.to_le_bytes()); // MapSurround 计数 0
    for value in [1.5_f64, 2.5, 3.5, 4.5] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(9_i32.to_le_bytes());
    let slot_offset = bytes.len();
    bytes.extend(object_header(&[0x5a; 16], 6));
    bytes.extend([0x88; 4]);
    let mut reader = Reader::new(&bytes);
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(
        suffix.stop,
        MapSuffixStop::EnvelopeObjectBody {
            offset: slot_offset,
            class_id: [0x5a; 16],
            key: 6,
        }
    );
    assert_eq!(suffix.extent.map(|e| e.xmax), Some(3.5));
    assert_eq!(suffix.bookmark_count, None);

    // (d) 书签条目新对象：最后阶段停止，extent 与计数均已到达。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    prefix_and_empty_list(&mut bytes);
    bytes.extend([0_u8; 16]);
    bytes.extend(0_i32.to_le_bytes());
    for value in [1.5_f64, 2.5, 3.5, 4.5] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend([0_u8; 16]); // IEnvelope 空标记
    bytes.extend(1_i32.to_le_bytes()); // 书签计数 1
    let slot_offset = bytes.len();
    bytes.extend(object_header(&[0x3c; 16], 8));
    bytes.extend(b"tail");
    let mut reader = Reader::new(&bytes);
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
    assert_eq!(
        suffix.stop,
        MapSuffixStop::BookmarkBody {
            index: 0,
            offset: slot_offset,
            class_id: [0x3c; 16],
            key: 8,
        }
    );
    assert_eq!(suffix.extent.map(|e| e.ymin), Some(2.5));
    assert_eq!(suffix.bookmark_count, Some(1));
    assert_eq!(suffix.remaining_bytes, 4);
}

#[test]
fn map_suffix_propagates_layer_list_flag1_stop() {
    // 样本行为镜像：图层列表首条目即标志 1 新对象（FDOGraphicsLayer 键 31），
    // 后缀在图层列表内停止，基本图形层等后续阶段均未到达。
    let class_id = fdo_graphics_layer_class_id();
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    bytes.extend(6_i32.to_le_bytes()); // 图层列表计数（样本 6）
    bytes.extend(1_u16.to_le_bytes()); // 标志 1（样本条目 1）
    bytes.extend(object_header(&class_id, 31));
    bytes.extend([0xab; 32]); // 未解释体
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let suffix = read_map_suffix_10_1(&mut reader, 1024, &mut table).unwrap();

    assert_eq!(
        suffix.stop,
        MapSuffixStop::LayerListFlag1Body {
            index: 0,
            offset: bytes.len() - 32 - 22,
            class_id,
            key: 31,
        }
    );
    assert_eq!(suffix.base_graphics_layer, None);
    assert_eq!(suffix.extent, None);
    assert_eq!(suffix.remaining_bytes, 32);
    // 前缀 PCS 键与停止条目的键都已登记（成功路径提交 trial 表）。
    assert!(table.contains(SAMPLE_PCS_KEY));
    assert!(table.contains(31));
}

#[test]
fn map_suffix_rejects_negative_loop_counts_transactionally() {
    // MapSurround 计数 -1：整体失败，游标与登记表不变。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes.extend((-1_i32).to_le_bytes()); // MapSurround 计数为负
    bytes.extend([0xff; 8]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error = read_map_suffix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("负数"), "{error}");
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());

    // 书签计数 -1：失败回滚包括前缀 PCS 键的登记。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes.extend(0_i32.to_le_bytes());
    for value in [1.5_f64, 2.5, 3.5, 4.5] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend(9_i32.to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes.extend((-1_i32).to_le_bytes()); // 书签计数为负
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    let error = read_map_suffix_10_1(&mut reader, 1024, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("负数"), "{error}");
    assert_eq!(reader.position(), 0);
    assert!(!table.contains(SAMPLE_PCS_KEY), "失败不得提交登记表");
}

#[test]
fn map_suffix_truncation_is_transactional() {
    // 逐字节截断完整后缀链：任何截断都必须失败、不推进游标、不修改登记表。
    let (bytes, _) = map_body_with_full_suffix("PROJCS[\"x\"]");
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end]);
        let mut table = RefTable10_1::new();
        assert!(
            read_map_suffix_10_1(&mut reader, 1024, &mut table).is_err(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(table.is_empty(), "end={end}");
    }
    // 完整输入成功。
    let mut reader = Reader::new(&bytes);
    assert!(read_map_suffix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).is_ok());
}

#[test]
fn map_suffix_prechecks_loop_counts_against_limit() {
    // MapSurround 计数 1000 × 最小条目 16 字节 > max_bytes 1024：预检拒绝。
    let mut bytes = sample_prefix_bytes("PROJCS[\"x\"]", b"");
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend([0_u8; 16]);
    bytes.extend(1000_i32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    let error = read_map_suffix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);
}
