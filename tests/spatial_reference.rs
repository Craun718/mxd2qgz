use mxd2qgz::{
    object_stream::{ObjectHeader, read_header_10_1},
    reader::Reader,
    spatial_reference::{
        PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
        read_projected_coordinate_system_10_1,
    },
};
use std::io::ErrorKind;

/// 参考样本记载的 7 个域 double（语义未确认，按证据文档原样引用）。
const SAMPLE_SEVEN_DOUBLES: [f64; 7] = [
    31876800.0,
    -10002100.0,
    450265407.00157917,
    -100000.0,
    10000.0,
    -100000.0,
    10000.0,
];

/// 参考样本记载的 3 个尾随 double。
const SAMPLE_THREE_DOUBLES: [f64; 3] = [0.001, 0.001, 0.001];

/// 版本 short + 4 字节 WKT 长度 + WKT 字节（所有版本分支共有部分）。
fn raw_body(version: u16, wkt: &[u8]) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend((wkt.len() as u32).to_le_bytes());
    bytes.extend(wkt);
    bytes
}

fn wkt_body(version: u16, wkt: &str) -> Vec<u8> {
    raw_body(version, wkt.as_bytes())
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

/// 参考样本布局的版本 6 对象体：4 个空域字符串、垂向标志 0、标志 0x0001、
/// 样本记载的 7+3 个 double。
fn sample_body(wkt: &str) -> Vec<u8> {
    let mut bytes = wkt_body(6, wkt);
    push_domain_block(&mut bytes, &[b"", b"", b"", b""], 0x00);
    push_precision_block(&mut bytes, 0x0001, &SAMPLE_SEVEN_DOUBLES);
    push_trailing_doubles(&mut bytes, &SAMPLE_THREE_DOUBLES);
    bytes
}

/// 断言解析结果与参考样本记载逐项一致。
fn assert_sample_values(body: &ProjectedCoordinateSystem10_1) {
    assert_eq!(body.save_version, 6);
    let domains = body.domains.as_ref().unwrap();
    assert_eq!(domains.strings, [vec![], vec![], vec![], vec![]]);
    assert_eq!(domains.vertical_flag, 0);
    let precision = body.precision.as_ref().unwrap();
    assert_eq!(precision.flags, 0x0001);
    assert_eq!(precision.doubles, SAMPLE_SEVEN_DOUBLES);
    assert_eq!(body.trailing_doubles, Some(SAMPLE_THREE_DOUBLES));
}

#[test]
fn class_id_constant_matches_documented_memory_bytes() {
    // 证据：Geometry.dll 10.1.0.3035 类注册 0x71a13098 处 16 字节，与样本
    // PCS 对象头逐字节相同（{2A626700-1DD2-11B2-BF51-08002022F573}）。
    assert_eq!(
        PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
        [
            0x00, 0x67, 0x62, 0x2a, 0xd2, 0x1d, 0xb2, 0x11, 0xbf, 0x51, 0x08, 0x00, 0x20, 0x22,
            0xf5, 0x73,
        ]
    );
}

#[test]
fn reads_sample_layout_and_stops_before_following_bytes() {
    let mut bytes = sample_body("PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\"]");
    let body_length = bytes.len();
    bytes.extend([0xab, 0xcd, 0xef]);
    let mut reader = Reader::new(&bytes);
    let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.wkt, "PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\"]");
    assert_sample_values(&body);
    assert_eq!(reader.position(), body_length);
    assert_eq!(reader.read_bytes(3).unwrap(), [0xab, 0xcd, 0xef]);
}

#[test]
fn reads_455_byte_wkt_matching_reference_sample_length() {
    // 样本中 11 处 WKT 恒为 455 字节；以同长度合成 PROJCS 文本复现该布局。
    let prefix = "PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\",";
    let wkt = format!("{prefix}{}]", ".".repeat(455 - prefix.len() - 1));
    assert_eq!(wkt.len(), 455);
    let bytes = sample_body(&wkt);
    // 与样本首个 PCS 对象体长度一致：0x274-0x30=580，去 20 字节对象头。
    assert_eq!(bytes.len(), 560);
    let mut reader = Reader::new(&bytes);
    let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.wkt.len(), 455);
    assert_sample_values(&body);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn every_version_branch_reads_exactly_the_native_field_set() {
    // 字段存在性与原生 Load 分支一致：1/2（及无表证据的 0）仅 WKT；3 加
    // 域块；4 加标志与 7 个 double；5/6/7 再加 3 个 double。
    for (version, domains, precision, trailing) in [
        (0_u16, false, false, false),
        (1, false, false, false),
        (2, false, false, false),
        (3, true, false, false),
        (4, true, true, false),
        (5, true, true, true),
        (6, true, true, true),
        (7, true, true, true),
    ] {
        let mut bytes = wkt_body(version, "PROJCS[\"x\"]");
        if domains {
            push_domain_block(&mut bytes, &[b"a", b"", b"c", b""], 0x00);
        }
        if precision {
            push_precision_block(&mut bytes, 0x0001, &SAMPLE_SEVEN_DOUBLES);
        }
        if trailing {
            push_trailing_doubles(&mut bytes, &SAMPLE_THREE_DOUBLES);
        }
        let body_length = bytes.len();
        let mut reader = Reader::new(&bytes);
        let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
        assert_eq!(body.save_version, version);
        assert_eq!(body.domains.is_some(), domains, "version={version}");
        assert_eq!(body.precision.is_some(), precision, "version={version}");
        assert_eq!(
            body.trailing_doubles.is_some(),
            trailing,
            "version={version}"
        );
        if let Some(domain_block) = body.domains {
            assert_eq!(
                domain_block.strings,
                [b"a".to_vec(), vec![], b"c".to_vec(), vec![]],
                "version={version}"
            );
        }
        // 各版本分支恰好读完自己的字段集，不多读一个字节。
        assert_eq!(reader.position(), body_length, "version={version}");
        assert_eq!(reader.remaining(), 0, "version={version}");
    }
}

#[test]
fn rejects_versions_above_native_limit_without_advancing() {
    // 原生对 >7 记录 0x80004005；0x8000.. 位模式在无符号解释下同样大于 7，
    // 若原生为有符号比较则严格子集更严（负值无保存表证据）。
    for version in [8_u16, 9, 100, 0x1234, 0x8000, 0xffff] {
        let bytes = wkt_body(version, "PROJCS[\"x\"]");
        let mut reader = Reader::new(&bytes);
        let error = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "version={version:#x}");
        assert!(error.to_string().contains("版本"), "version={version:#x}");
        assert_eq!(reader.position(), 0, "version={version:#x}");
    }
    // 仅 2 字节版本即足以触发拒绝，不依赖对象体其余部分。
    let bytes = 8_u16.to_le_bytes();
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_projected_coordinate_system_10_1(&mut reader, 1024)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
}

#[test]
fn rejects_every_truncated_prefix_without_advancing() {
    // 逐字节截断覆盖每个字段内部：版本、长度前缀、WKT、域串、垂向标志、
    // 标志以及 80 字节 double 区域的每一个字节位置。
    let full = sample_body("PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\"]");
    for end in 0..full.len() {
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_projected_coordinate_system_10_1(&mut reader, 1024).is_err(),
            "end={end}"
        );
        assert_eq!(reader.position(), 0, "end={end}");
    }
    // 失败游标重读一致：同一读取器重复读取得到相同错误类型与位置。
    let mut reader = Reader::new(&full[..full.len() - 1]);
    for _ in 0..2 {
        let error = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn rejects_over_limit_lengths_before_reading_or_allocating() {
    let wkt = "PROJCS[\"x\"]";
    // WKT 长度等于限额时可读（版本 1 的布局只有版本与 WKT）；超过限额在
    // 读取与分配之前拒绝。
    let bytes = wkt_body(1, wkt);
    let mut reader = Reader::new(&bytes);
    assert!(read_projected_coordinate_system_10_1(&mut reader, wkt.len()).is_ok());
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_projected_coordinate_system_10_1(&mut reader, wkt.len() - 1)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);

    // u32::MAX 长度（版本 1 前缀保证先通过版本检查）：有限额时按
    // InvalidData 拒绝；限额放开时因流过短报 UnexpectedEof；两者都不推进
    // 游标、不做无界分配。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(u32::MAX.to_le_bytes());
    bytes.extend(b"PROJCS");
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_projected_coordinate_system_10_1(&mut reader, 1024)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_projected_coordinate_system_10_1(&mut reader, usize::MAX)
            .unwrap_err()
            .kind(),
        ErrorKind::UnexpectedEof
    );
    assert_eq!(reader.position(), 0);

    // 域字符串长度同样受限额约束（版本 3 分支）。
    let mut bytes = wkt_body(3, wkt);
    bytes.extend(1025_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    assert_eq!(
        read_projected_coordinate_system_10_1(&mut reader, 1024)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidData
    );
    assert_eq!(reader.position(), 0);
    // 长度等于限额的第一个域字符串可读，其余域为空、垂向标志正常。
    let mut bytes = wkt_body(3, wkt);
    bytes.extend(1024_u32.to_le_bytes());
    bytes.extend([0x41; 1024]);
    bytes.extend([0_u8; 12]); // 三个空域字符串的长度前缀。
    bytes.push(0); // 垂向标志。
    let mut reader = Reader::new(&bytes);
    let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
    assert_eq!(body.domains.as_ref().unwrap().strings[0].len(), 1024);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn rejects_non_ascii_and_nul_wkt_stricter_than_native() {
    // 原生读取不校验 WKT 内容；严格子集拒绝 >=0x80 字节与嵌入 NUL。
    for wkt in [
        &b"PROJCS[\x80]"[..],          // ANSI 单字节高位。
        "PROJCS[\"底图\"]".as_bytes(), // UTF-8 多字节序列。
        b"PROJCS\0[\"x\"]",            // 嵌入 NUL。
    ] {
        let bytes = raw_body(6, wkt);
        let mut reader = Reader::new(&bytes);
        let error = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("WKT"));
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn preserves_domain_bytes_without_ascii_or_encoding_validation() {
    // 域字符串随 ANSI 代码页（如 GBK 的 >=0x80 字节）且语义未确认；读取器
    // 保留原始字节（含 NUL），与 WKT 的校验策略差异见模块文档。
    let mut bytes = wkt_body(3, "PROJCS[\"x\"]");
    push_domain_block(
        &mut bytes,
        &[b"\x80\x81\xd6\xd0", b"abc\0", b"", b"\xff"],
        0x2a,
    );
    let mut reader = Reader::new(&bytes);
    let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
    let domains = body.domains.unwrap();
    assert_eq!(
        domains.strings,
        [
            b"\x80\x81\xd6\xd0".to_vec(),
            b"abc\0".to_vec(),
            vec![],
            b"\xff".to_vec(),
        ]
    );
    assert_eq!(domains.vertical_flag, 0x2a);
    assert_eq!(body.precision, None);
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn preserves_flag_and_double_bit_patterns_without_validation() {
    // 10 个 double 仅原样保留小端位模式：NaN、无穷、-0.0 等都接受；标志与
    // 垂向标志字段同样不校验数值范围。
    let doubles = [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -0.0,
        f64::MIN,
        f64::MAX,
        5.0e-324,
    ];
    let mut bytes = wkt_body(7, "PROJCS[\"x\"]");
    push_domain_block(&mut bytes, &[b"", b"", b"", b""], 0xff);
    push_precision_block(&mut bytes, 0xffff, &doubles);
    push_trailing_doubles(&mut bytes, &[-1.0, 0.0, 1.0]);
    let mut reader = Reader::new(&bytes);
    let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
    let precision = body.precision.unwrap();
    assert_eq!(precision.flags, 0xffff);
    assert_eq!(body.domains.unwrap().vertical_flag, 0xff);
    for (read, expected) in precision.doubles.iter().zip(doubles.iter()) {
        assert_eq!(read.to_bits(), expected.to_bits());
    }
    assert_eq!(body.trailing_doubles, Some([-1.0, 0.0, 1.0]));
    assert_eq!(reader.remaining(), 0);
}

#[test]
fn reads_pcs_header_then_body_from_object_stream() {
    // 组合用例：调用者先用对象头读取器确认 PCS 类标识，再读对象体。
    let mut bytes = PROJECTED_COORDINATE_SYSTEM_CLASS_ID.to_vec();
    bytes.extend(0x1e_u32.to_le_bytes()); // 样本首个 PCS 对象键。
    bytes.extend(sample_body("PROJCS[\"CGCS2000_3_Degree_GK_Zone_37\"]"));
    let mut reader = Reader::new(&bytes);
    let header = read_header_10_1(&mut reader).unwrap();
    let ObjectHeader::Object { class_id, key } = header else {
        panic!("PCS 对象头不应为 Null：{header:?}");
    };
    assert_eq!(class_id, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    assert_eq!(key, 0x1e);
    let body = read_projected_coordinate_system_10_1(&mut reader, 1024).unwrap();
    assert_sample_values(&body);
    assert_eq!(reader.remaining(), 0);

    // 样本中 PCS 之后的未知类标识 {34B2EF83-F4AC-11D1-A245-080009B6F22B}
    // 不匹配常量，调用者须拒绝而非继续按 PCS 读取。
    let unknown_class_id = [
        0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6, 0xf2,
        0x2b,
    ];
    assert_ne!(unknown_class_id, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
    let mut bytes = unknown_class_id.to_vec();
    bytes.extend(0x1f_u32.to_le_bytes());
    let mut reader = Reader::new(&bytes);
    match read_header_10_1(&mut reader).unwrap() {
        ObjectHeader::Object { class_id, key } => {
            assert_ne!(class_id, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
            assert_eq!(key, 0x1f);
        }
        other => panic!("非零类标识不应为 Null：{other:?}"),
    }
}
