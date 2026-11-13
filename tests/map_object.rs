use mxd2qgz::map_object::{MAP_CLASS_ID, read_map_prefix_10_1};
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
