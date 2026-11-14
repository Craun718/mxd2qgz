use mxd2qgz::feature_layer::{
    DataSource10_1, FEATURE_CLASS_NAME_CLASS_ID, FEATURE_LAYER_CLASS_ID, Visibility10_1,
    read_feature_layer_prefix_10_1,
};
use mxd2qgz::object_refs::RefTable10_1;
use mxd2qgz::object_stream::{ObjectHeader, read_header_10_1};
use mxd2qgz::reader::Reader;
use std::io::ErrorKind;

/// 样本记载的首个 FeatureLayer 架构版本（`/Maps:0x2aa`）。
const SAMPLE_VERSION: u16 = 34;
/// 样本首个 FeatureLayer 对象键（`/Maps:0x2a6`）。
const SAMPLE_LAYER_KEY: u32 = 32;
/// 样本内嵌 FeatureClassName 对象键（`/Maps:0x306`）。
const SAMPLE_FCN_KEY: u32 = 33;
/// 样本 FeatureClassName 体首 int16（`/Maps:0x30a`）。
const SAMPLE_FCN_LEADING_SHORT: u16 = 2;
/// 样本图层名（`/Maps:0x2ac` 长度 32 = 15 字 ×2 + 终止零 2）。
const SAMPLE_NAME: &str = "龙安镇人民政府工作底图用界址点";
/// 样本第二名称（`/Maps:0x2d0` 长度 28 = 13 字 ×2 + 终止零 2）。
const SAMPLE_SECONDARY_NAME: &str = "Shapefile 要素类";

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

/// 通用前缀构造：版本 + 两名称 + 三个 short（可见性 + 两个未知）+ 数据源
/// 条目字节 + 调用者提供的未知尾部。
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

/// FeatureClassName 新对象条目：16 字节类标识 + 键 + int16 + 要素类名。
fn fcn_entry(key: u32, leading_short: u16, feature_class_name: &str) -> Vec<u8> {
    let mut bytes = FEATURE_CLASS_NAME_CLASS_ID.to_vec();
    bytes.extend(key.to_le_bytes());
    bytes.extend(leading_short.to_le_bytes());
    bytes.extend(encoded(feature_class_name));
    bytes
}

/// 与参考样本布局对齐的完整已确认前缀：版本 34 + 中文名 + 第二名称 +
/// 可见性 0xFFFF + 两个未知 short 0 + FCN（键 33、int16 02、与图层名同文的
/// 要素类名）+ 调用者提供的未知尾部。已确认前缀共 134 字节
/// （2+36+32+6+20+2+36），对应样本 `0x2aa..0x330`。
fn sample_prefix_bytes(tail: &[u8]) -> Vec<u8> {
    layer_prefix_bytes(
        SAMPLE_VERSION,
        SAMPLE_NAME,
        SAMPLE_SECONDARY_NAME,
        [0xFFFF, 0x0000, 0x0000],
        &fcn_entry(SAMPLE_FCN_KEY, SAMPLE_FCN_LEADING_SHORT, SAMPLE_NAME),
        tail,
    )
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
    // 声明 + GdbCore 族字节命中，且与样本 /Maps:0x2f6 数据源对象头逐字节相同。
    assert_eq!(
        FEATURE_CLASS_NAME_CLASS_ID,
        [
            0xd0, 0x46, 0x88, 0x19, 0x42, 0xca, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3,
            0x3a, 0x15,
        ]
    );
    // 两个类标识不得混淆。
    assert_ne!(FEATURE_LAYER_CLASS_ID, FEATURE_CLASS_NAME_CLASS_ID);
}

#[test]
fn reads_sample_layout_prefix_and_preserves_remaining_bytes() {
    // 样本镜像：版本 34、中文名称、中文混排第二名称、可见性 0xFFFF、两个
    // 未知 short 0、FCN 新对象（键 33、int16 02、与图层名同文），之后是本
    // 读取器不解释的未知尾部（真实样本此处是 FCN 中段起未闭合字段）。
    let bytes = sample_prefix_bytes(b"rest");
    let prefix_length = bytes.len() - 4;
    assert_eq!(
        prefix_length, 134,
        "已确认前缀长度应与样本 0x2aa..0x330 对应"
    );
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(parsed.version, SAMPLE_VERSION);
    assert_eq!(parsed.name, SAMPLE_NAME);
    assert_eq!(parsed.secondary_name, SAMPLE_SECONDARY_NAME);
    assert_eq!(parsed.visibility_raw, 0xFFFF);
    assert_eq!(parsed.visibility(), Visibility10_1::True);
    assert_eq!(parsed.unknown_short_a, 0);
    assert_eq!(parsed.unknown_short_b, 0);
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!(
            "数据源应分类为新 FeatureClassName：{:?}",
            parsed.data_source
        );
    };
    assert_eq!(fcn.key, SAMPLE_FCN_KEY);
    assert_eq!(fcn.leading_short, SAMPLE_FCN_LEADING_SHORT);
    assert_eq!(fcn.feature_class_name, SAMPLE_NAME);
    // 已确认前缀恰好停在要素类名之后；FCN 中段等保留为 remaining_bytes。
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), prefix_length);
    assert_eq!(reader.read_bytes(4).unwrap(), b"rest");
    assert_eq!(reader.remaining(), 0);
    // 成功提交后登记表包含 FCN 键（登记先于对象体解析的原生语义）。
    assert!(table.contains(SAMPLE_FCN_KEY));
    assert_eq!(table.len(), 1);
}

#[test]
fn reads_version_boundaries_zero_and_34() {
    // 无符号比较的接受范围 0..=34：下界 0 与上界 34（样本值）都必须接受。
    // 数据源用 16 零字节空头，聚焦版本边界本身。
    for version in [0_u16, 34] {
        let bytes = layer_prefix_bytes(version, "图层", "", [0; 3], &[0_u8; 16], b"tail");
        let mut reader = Reader::new(&bytes);
        let parsed =
            read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        assert_eq!(parsed.version, version, "version={version}");
        assert_eq!(
            parsed.data_source,
            DataSource10_1::Null,
            "version={version}"
        );
        assert_eq!(parsed.remaining_bytes, 4, "version={version}");
        assert_eq!(reader.position(), bytes.len() - 4, "version={version}");
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
        // 失败可重复：重读得到相同错误类别且游标仍在入口。
        let second = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new())
            .unwrap_err();
        assert_eq!(first.kind(), second.kind(), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
}

#[test]
fn reads_visibility_bit_patterns_with_interpretation_helper() {
    // 可见性按原始 u16 位模式保留：规范值 0xFFFF/0x0000 之外（含单真位、
    // 符号位、-2 等）一律 NonCanonical 原样保留，不做「非零即真」假设；
    // 其后两个未知 short 同样原样保留、不校验。
    for (raw, expected) in [
        (0xFFFF_u16, Visibility10_1::True),
        (0x0000, Visibility10_1::False),
        (0x0001, Visibility10_1::NonCanonical(0x0001)),
        (0x00FF, Visibility10_1::NonCanonical(0x00FF)),
        (0x8000, Visibility10_1::NonCanonical(0x8000)),
        (0xFFFE, Visibility10_1::NonCanonical(0xFFFE)),
    ] {
        let bytes = layer_prefix_bytes(
            SAMPLE_VERSION,
            "图层😀",
            "MZD1",
            [raw, 0x1234, 0xABCD],
            &[0_u8; 16],
            b"",
        );
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
    // 数据源位置的全零 16 字节是空引用标记（无键、无对象体），解析为 Null
    // 且不吞并其后字节，登记表保持为空。
    let bytes = layer_prefix_bytes(SAMPLE_VERSION, "图层", "", [0; 3], &[0_u8; 16], b"unparsed");
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
    // 新 FCN 对象成功解析后登记表包含键；用同一登记表重读同一字节时，该键
    // 已见，同一条目分类为回引（无体）——流内无法局部区分回引与新对象，
    // 与原生语义一致（object_refs 模块文档）。
    let bytes = sample_prefix_bytes(b"");
    let mut table = RefTable10_1::new();

    let mut first_reader = Reader::new(&bytes);
    let first = read_feature_layer_prefix_10_1(&mut first_reader, 1024, &mut table).unwrap();
    assert!(matches!(
        first.data_source,
        DataSource10_1::FeatureClassName(_)
    ));
    assert!(table.contains(SAMPLE_FCN_KEY));
    assert_eq!(first_reader.position(), 134);

    let mut second_reader = Reader::new(&bytes);
    let second = read_feature_layer_prefix_10_1(&mut second_reader, 1024, &mut table).unwrap();
    assert_eq!(
        second.data_source,
        DataSource10_1::BackReference {
            class_id: FEATURE_CLASS_NAME_CLASS_ID,
            key: SAMPLE_FCN_KEY,
        }
    );
    // 回引无体：位置停在 20 字节条目头之后，未读的 FCN 体（38 字节）归入
    // remaining_bytes。
    assert_eq!(second_reader.position(), 76 + 20);
    assert_eq!(second.remaining_bytes, 38);
    // 回引不重复登记。
    assert_eq!(table.len(), 1);
}

#[test]
fn reads_back_reference_with_previously_registered_key() {
    // 两图层场景：第一个图层的 FCN 新对象（键 33）提交登记后，第二个图层
    // 的数据源条目只写 20 字节回引头（无体），读取后尾部紧接条目头。
    let first_bytes = sample_prefix_bytes(b"");
    let mut table = RefTable10_1::new();
    let mut first_reader = Reader::new(&first_bytes);
    assert!(read_feature_layer_prefix_10_1(&mut first_reader, 1024, &mut table).is_ok());
    assert!(table.contains(SAMPLE_FCN_KEY));

    let mut back_reference_entry = FEATURE_CLASS_NAME_CLASS_ID.to_vec();
    back_reference_entry.extend(SAMPLE_FCN_KEY.to_le_bytes());
    let second_bytes = layer_prefix_bytes(
        SAMPLE_VERSION,
        "注记",
        "",
        [0xFFFF, 0, 0],
        &back_reference_entry,
        b"tail",
    );
    let entry_end = second_bytes.len() - 4;
    let mut reader = Reader::new(&second_bytes);
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap();
    assert_eq!(
        parsed.data_source,
        DataSource10_1::BackReference {
            class_id: FEATURE_CLASS_NAME_CLASS_ID,
            key: SAMPLE_FCN_KEY,
        }
    );
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), entry_end);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
    assert_eq!(table.len(), 1, "回引不得新增登记");
}

#[test]
fn rejects_unknown_new_class_id_without_advancing_or_registering() {
    // 未知类标识的新对象显式拒绝：本读取器只支持 FeatureClassName 数据源，
    // 且新对象体没有外部长度字段、无法跳过未知类。第一个 GUID 是样本中出
    // 现的 WorkspaceName {5A350011-E371-11D1-AA82-00C04FA33A15}（真实流里嵌
    // 在 FCN 体内，此处直接出现在数据源位置即未知类），第二个是纯合成标识。
    let workspace_name = [
        0x11, 0x00, 0x35, 0x5a, 0x71, 0xe3, 0xd1, 0x11, 0xaa, 0x82, 0x00, 0xc0, 0x4f, 0xa3, 0x3a,
        0x15,
    ];
    let synthetic = [
        0x3c, 0x92, 0xf1, 0x48, 0x20, 0xb6, 0x4c, 0xb9, 0x9e, 0x05, 0x31, 0xd8, 0x7a, 0x2e, 0xc4,
        0x66,
    ];
    for class_id in [&workspace_name, &synthetic] {
        assert_ne!(*class_id, FEATURE_CLASS_NAME_CLASS_ID);
        let mut entry = class_id.to_vec();
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
        assert_eq!(first.kind(), ErrorKind::Unsupported, "{class_id:?}");
        let message = first.to_string();
        assert!(message.contains("FeatureClassName"), "{message}");
        assert!(message.contains("无法跳过"), "{message}");
        assert!(message.contains("键 33"), "{message}");
        assert_eq!(reader.position(), 0, "游标必须留在入口");
        // 分类状态机已把键登记进副本表，但失败路径必须整体回滚：调用者的
        // 登记表不得包含该键。
        assert!(!table.contains(SAMPLE_FCN_KEY), "{class_id:?}");
        assert!(table.is_empty(), "{class_id:?}");
        let second = read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).unwrap_err();
        assert_eq!(first.kind(), second.kind(), "重读结果不一致");
        assert_eq!(reader.position(), 0, "重读后游标仍在入口");
        assert!(table.is_empty(), "重读失败仍不得修改登记表");
    }
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断扫描覆盖每个字段的内部：版本（2 字节）、两个名称的前缀与
    // 载荷、三个 short、FCN 对象头的 GUID 与键、FCN 体 int16 与要素类名的
    // 前缀与载荷。任何截断都必须失败、不推进游标、不修改登记表，且重读得
    // 到相同错误类别。
    let full = sample_prefix_bytes(b"rest");
    let prefix_length = full.len() - 4;
    assert_eq!(prefix_length, 134);
    assert!(prefix_length > 100, "完整输入应覆盖所有字段区域");
    // 边界情形：恰好在前缀末尾截断（未知尾部缺失）是合法完整输入——尾部
    // 本就不属于已确认前缀，此处应成功且 remaining_bytes == 0。
    let mut boundary = Reader::new(&full[..prefix_length]);
    let mut table = RefTable10_1::new();
    let parsed = read_feature_layer_prefix_10_1(&mut boundary, 1024, &mut table).unwrap();
    assert_eq!(parsed.remaining_bytes, 0);
    assert_eq!(boundary.remaining(), 0);
    assert!(table.contains(SAMPLE_FCN_KEY));
    for end in 0..prefix_length {
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
    // 完整输入本身必须成功，未知尾部保留在游标之后。
    let mut reader = Reader::new(&full);
    let mut table = RefTable10_1::new();
    assert!(read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).is_ok());
    assert_eq!(reader.remaining(), 4);
}

#[test]
fn failure_inside_feature_class_name_body_rolls_back_registration() {
    // 关键事务性用例：条目分类成功（键已按原生语义登记进副本表）之后、
    // FCN 体读取失败（int16 或要素类名截断），整个调用失败且调用者的登记
    // 表不得留下该键。
    let full = sample_prefix_bytes(b"");
    let prefix_length = full.len();
    assert_eq!(prefix_length, 134);
    // 条目头起点 = 2 + 36 + 32 + 6 = 76；截断点分别取 FCN 头之后（int16 缺
    // 失）、int16 之后（要素类名前缀缺失）与前缀末字节缺失（载荷截断）。
    let entry_start = 76;
    assert_eq!(
        full[entry_start..entry_start + 16],
        FEATURE_CLASS_NAME_CLASS_ID
    );
    for end in [entry_start + 20, entry_start + 20 + 2, prefix_length - 1] {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_feature_layer_prefix_10_1(&mut reader, 1024, &mut table).is_err(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
        assert!(!table.contains(SAMPLE_FCN_KEY), "end={end}");
        assert!(table.is_empty(), "end={end}");
    }
}

#[test]
fn rejects_over_limit_lengths_before_reading_or_allocating() {
    // 名称限额边界：样本布局三个字符串声明长度为 32/28/32，限额 32 恰好全
    // 部可读；限额少一字节即在读取与分配之前拒绝（首个名称声明 32 超限）。
    let bytes = sample_prefix_bytes(b"");
    let mut reader = Reader::new(&bytes);
    assert!(read_feature_layer_prefix_10_1(&mut reader, 32, &mut RefTable10_1::new()).is_ok());
    let mut reader = Reader::new(&bytes);
    let error =
        read_feature_layer_prefix_10_1(&mut reader, 31, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // 限额同样作用于第二名称：第一名称声明 4 字节在限额内，第二名称 6 字
    // 节超过限额被拒，游标不推进。
    let second = layer_prefix_bytes(SAMPLE_VERSION, "A", "AB", [0; 3], &[0_u8; 16], b"");
    let mut reader = Reader::new(&second);
    let error =
        read_feature_layer_prefix_10_1(&mut reader, 4, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);

    // 限额原样转发给 FCN 体的要素类名：两个名称在限额内、要素类名（16 字
    // → 声明 34）超限被拒；限额放到 34 则完整可读。
    let long_name = "龙安镇人民政府工作底图用界址点X";
    assert_eq!(encoded(long_name).len(), 4 + 34);
    let with_long_fcn = layer_prefix_bytes(
        SAMPLE_VERSION,
        "图层",
        "MZD1",
        [0xFFFF, 0, 0],
        &fcn_entry(SAMPLE_FCN_KEY, SAMPLE_FCN_LEADING_SHORT, long_name),
        b"",
    );
    let mut reader = Reader::new(&with_long_fcn);
    let error =
        read_feature_layer_prefix_10_1(&mut reader, 32, &mut RefTable10_1::new()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("限额"), "{error}");
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&with_long_fcn);
    let mut table = RefTable10_1::new();
    let parsed = read_feature_layer_prefix_10_1(&mut reader, 34, &mut table).unwrap();
    let DataSource10_1::FeatureClassName(fcn) = &parsed.data_source else {
        panic!("应解析出 FCN 前缀：{:?}", parsed.data_source);
    };
    assert_eq!(fcn.feature_class_name, long_name);
    assert!(table.contains(SAMPLE_FCN_KEY));

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
    assert_eq!(parsed.remaining_bytes, 4);
    assert_eq!(reader.position(), bytes.len() - 4);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");

    // 反例：顶部对象头是 FeatureClassName 标识时类身份不匹配 FeatureLayer
    // 常量，调用者须拒绝而不能进入 FeatureLayer 前缀读取（类身份确认是调
    // 用者的职责）。
    let mut other = FEATURE_CLASS_NAME_CLASS_ID.to_vec();
    other.extend(SAMPLE_FCN_KEY.to_le_bytes());
    let mut reader = Reader::new(&other);
    match read_header_10_1(&mut reader).unwrap() {
        ObjectHeader::Object { class_id, key } => {
            assert_ne!(class_id, FEATURE_LAYER_CLASS_ID);
            assert_eq!(key, SAMPLE_FCN_KEY);
        }
        other_header => panic!("FCN 对象头不应为 Null：{other_header:?}"),
    }
}
