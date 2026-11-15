//! `number_format` 读取器的合成输入测试（SystemCore.dll 反编译字段序；
//! 样本 /Maps 流内后五类 0 实例，全部为合成覆盖）。

use mxd2qgz::objects::number_format::{
    ANGLE_FORMAT_CLASS_ID, AngleFormatBody10_1, CURRENCY_FORMAT_CLASS_ID,
    CUSTOM_NUMBER_FORMAT_CLASS_ID, CurrencyFormatBody10_1, CustomNumberFormatBody10_1,
    FRACTION_FORMAT_CLASS_ID, FractionFormatBody10_1, LATLON_FORMAT_CLASS_ID, LatLonFormatBody10_1,
    NUMERIC_FORMAT_CLASS_ID, NumericFormatBody10_1, PERCENTAGE_FORMAT_CLASS_ID,
    PercentageFormatBody10_1, RATE_FORMAT_CLASS_ID, RateFormatBody10_1, SCIENTIFIC_FORMAT_CLASS_ID,
    ScientificFormatBody10_1, read_angle_format_body_10_1, read_currency_format_body_10_1,
    read_custom_number_format_body_10_1, read_fraction_format_body_10_1,
    read_latlon_format_body_10_1, read_numeric_format_body_10_1, read_percentage_format_body_10_1,
    read_rate_format_body_10_1, read_scientific_format_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

const KEY: u32 = 77;
const MAX_BYTES: usize = 0x1000;

/// AfCore String::Write 布局：u32 字节长度前缀 + 含终止零的 UTF-16LE。
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

/// NumericFormat 核心体（版本 1）：int32 ×4 + short ×3。
fn numeric_core(int32s: [i32; 4]) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    for v in int32s {
        bytes.extend(v.to_le_bytes());
    }
    for v in [0_i16; 3] {
        bytes.extend(v.to_le_bytes());
    }
    bytes
}

/// PercentageFormat 版本 2 满形态：short 版本 + short -0xc + 内嵌核心。
fn percentage_v2(short_0c: i16) -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(short_0c.to_le_bytes());
    bytes.extend(numeric_core([0, 6, 0, 12]));
    bytes
}

/// RateFormat 版本 2 满形态：short 版本 + double + String + 内嵌核心。
fn rate_v2(double_14: f64, text: &str) -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(double_14.to_le_bytes());
    bytes.extend(encoded(text));
    bytes.extend(numeric_core([1, 2, 3, 4]));
    bytes
}

#[test]
fn class_id_constants_match_documented_clsids() {
    // 16 字节内存序（Data1-3 小端、Data4 原序）与 esri.clsid 声明互证。
    assert_eq!(
        PERCENTAGE_FORMAT_CLASS_ID,
        [
            0x1b, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        FRACTION_FORMAT_CLASS_ID,
        [
            0x1c, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        RATE_FORMAT_CLASS_ID,
        [
            0x21, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        NUMERIC_FORMAT_CLASS_ID,
        [
            0x19, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        CURRENCY_FORMAT_CLASS_ID,
        [
            0x1a, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        LATLON_FORMAT_CLASS_ID,
        [
            0x1d, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        ANGLE_FORMAT_CLASS_ID,
        [
            0x1e, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        SCIENTIFIC_FORMAT_CLASS_ID,
        [
            0x1f, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        CUSTOM_NUMBER_FORMAT_CLASS_ID,
        [
            0x22, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0, 0, 0, 0, 0, 0
        ]
    );
}

#[test]
fn numeric_core_reads_24_byte_body() {
    let mut bytes = numeric_core([0, 0, 0, 12]);
    bytes.extend([0xaa, 0xbb]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_numeric_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        NumericFormatBody10_1 {
            key: KEY,
            version: 1,
            int32s: [0, 0, 0, 12],
            shorts: [0, 0, 0],
        }
    );
    assert_eq!(reader.position(), 24, "未知尾部字节不被消费");
}

#[test]
fn percentage_version_2_reads_embedded_core() {
    let body_bytes = percentage_v2(-1);
    let mut bytes = body_bytes.clone();
    bytes.extend([0x11, 0x22]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_percentage_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    let numeric = body.numeric.as_ref().expect("版本 2 应含内嵌核心");
    assert_eq!(
        body,
        PercentageFormatBody10_1 {
            key: KEY,
            version: 2,
            short_0c: -1,
            numeric: Some(Box::new(NumericFormatBody10_1 {
                key: KEY,
                version: 1,
                int32s: [0, 6, 0, 12],
                shorts: [0, 0, 0],
            })),
        }
    );
    assert_eq!(numeric.key, KEY, "内嵌核心与外壳同键");
    assert_eq!(reader.position(), body_bytes.len(), "未知尾部不被消费");
}

#[test]
fn percentage_version_1_short_body_has_no_core() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(7_i16.to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_percentage_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.short_0c, 7);
    assert!(body.numeric.is_none(), "版本 ≤1 无内嵌核心");
    assert_eq!(reader.position(), 4);
}

#[test]
fn percentage_rejects_out_of_range_and_negative_versions_without_advancing() {
    for version in [3_i16, 4, 0x7fff, -1, 0x8000_u16 as i16] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend(0_i16.to_le_bytes());
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(
            read_percentage_format_body_10_1(&mut reader, &mut table, KEY).is_err(),
            "版本 {version} 应被拒绝"
        );
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert!(table.is_empty(), "失败路径不修改登记表");
    }
}

#[test]
fn fraction_reads_10_byte_body_and_rejects_version_2() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0x0a_i32.to_le_bytes());
    bytes.extend((-3_i32).to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_fraction_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        FractionFormatBody10_1 {
            key: KEY,
            version: 1,
            int32_10: 0x0a,
            int32_0c: -3,
        }
    );
    assert_eq!(reader.position(), 10);
    // 版本 2 原生拒绝（`1 < (short)ver`）。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend([0; 8]);
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    assert!(read_fraction_format_body_10_1(&mut reader, &mut table, KEY).is_err());
    assert_eq!(reader.position(), 0);
}

#[test]
fn rate_version_2_reads_double_string_and_embedded_core() {
    let body_bytes = rate_v2(2.5, "米/秒");
    let mut bytes = body_bytes.clone();
    bytes.push(0xff);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_rate_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).unwrap();
    let numeric = body.numeric.as_ref().expect("版本 2 应含内嵌核心");
    assert_eq!(body.key, KEY);
    assert_eq!(body.version, 2);
    assert_eq!(body.double_14, 2.5);
    assert_eq!(body.string_0c, "米/秒");
    assert_eq!(
        numeric.int32s,
        [1, 2, 3, 4],
        "内嵌核心字段与流内一致（同槽升级）"
    );
    assert_eq!(numeric.key, KEY);
    assert_eq!(reader.position(), body_bytes.len());
}

#[test]
fn rate_version_1_short_body_stops_after_string() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0.5_f64.to_le_bytes());
    bytes.extend(encoded("km"));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_rate_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.double_14, 0.5);
    assert_eq!(body.string_0c, "km");
    assert!(body.numeric.is_none(), "版本 ≤1 无内嵌核心");
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn rate_rejects_bad_versions_and_oversized_strings() {
    // 版本 3 超限；负值位模式拒绝（较原生更严）。
    for version in [3_i16, -2] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend(0.0_f64.to_le_bytes());
        bytes.extend(encoded("x"));
        bytes.extend(numeric_core([0; 4]));
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(read_rate_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
    // 声明字符串超过 max_bytes：在任何分配之前拒绝。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(1.0_f64.to_le_bytes());
    bytes.extend(encoded("长度超限"));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    assert!(read_rate_format_body_10_1(&mut reader, 4, &mut table, KEY).is_err());
    assert_eq!(reader.position(), 0);
}

#[test]
fn truncated_bodies_fail_without_advancing() {
    let full = rate_v2(1.0, "x");
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_rate_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).is_err(),
            "前缀长度 {end} 应截断失败"
        );
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert!(table.is_empty(), "失败路径不修改登记表");
    }
    let pct = percentage_v2(0);
    for end in 0..pct.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&pct[..end]);
        assert!(read_percentage_format_body_10_1(&mut reader, &mut table, KEY).is_err());
        assert_eq!(reader.position(), 0);
    }
    let frac = {
        let mut b = 1_u16.to_le_bytes().to_vec();
        b.extend(1_i32.to_le_bytes());
        b.extend(2_i32.to_le_bytes());
        b
    };
    for end in 0..frac.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&frac[..end]);
        assert!(read_fraction_format_body_10_1(&mut reader, &mut table, KEY).is_err());
        assert_eq!(reader.position(), 0);
    }
}

// ===== 同族五类（Currency / LatLon / Angle / Scientific / CustomNumber）=====

/// LatLonFormat 版本 4 满形态：short 版本 + short ×2 + short ×2 + 内嵌核心
/// + short。
fn latlon_v4() -> Vec<u8> {
    let mut bytes = 4_u16.to_le_bytes().to_vec();
    for v in [-2_i16, 3] {
        bytes.extend(v.to_le_bytes());
    }
    for v in [7_i16, 8] {
        bytes.extend(v.to_le_bytes());
    }
    bytes.extend(numeric_core([5, 6, 7, 8]));
    bytes.extend(9_i16.to_le_bytes());
    bytes
}

/// AngleFormat 版本 3 满形态：short 版本 + short + 内嵌核心 + short。
fn angle_v3() -> Vec<u8> {
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(4_i16.to_le_bytes());
    bytes.extend(numeric_core([9, 10, 11, 12]));
    bytes.extend(5_i16.to_le_bytes());
    bytes
}

#[test]
fn currency_reads_two_byte_body() {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend([0xaa, 0xbb]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_currency_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        CurrencyFormatBody10_1 {
            key: KEY,
            version: 1,
        }
    );
    assert_eq!(reader.position(), 2, "未知尾部字节不被消费");
    // 逐前缀截断（仅版本 short 一个字段，前缀 0 即空输入）。
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&[]);
    assert!(read_currency_format_body_10_1(&mut reader, &mut table, KEY).is_err());
    assert_eq!(reader.position(), 0);
}

#[test]
fn currency_rejects_out_of_range_and_negative_versions() {
    for version in [2_i16, 3, 0x7fff, -1, 0x8000_u16 as i16] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 4]);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(
            read_currency_format_body_10_1(&mut reader, &mut table, KEY).is_err(),
            "版本 {version} 应被拒绝"
        );
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert!(table.is_empty(), "失败路径不修改登记表");
    }
}

#[test]
fn latlon_version_4_reads_full_form() {
    let body_bytes = latlon_v4();
    let mut bytes = body_bytes.clone();
    bytes.push(0xee);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_latlon_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        LatLonFormatBody10_1 {
            key: KEY,
            version: 4,
            short_2c: -2,
            short_2a: 3,
            short_0c: Some(7),
            short_0a: Some(8),
            numeric: Some(Box::new(NumericFormatBody10_1 {
                key: KEY,
                version: 1,
                int32s: [5, 6, 7, 8],
                shorts: [0, 0, 0],
            })),
            short_08: Some(9),
        }
    );
    assert_eq!(reader.position(), body_bytes.len(), "未知尾部不被消费");
}

#[test]
fn latlon_version_branches_stop_at_the_right_fields() {
    // 版本 3：有 ④ 与核心，无 ⑥。
    let mut bytes = 3_u16.to_le_bytes().to_vec();
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend(3_i16.to_le_bytes());
    bytes.extend(4_i16.to_le_bytes());
    bytes.extend(numeric_core([1, 1, 1, 1]));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_latlon_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.version, 3);
    assert_eq!(body.short_08, None, "版本 3 无 ⑥");
    assert!(body.numeric.is_some(), "版本 3 含内嵌核心");
    assert_eq!(reader.position(), bytes.len());
    // 版本 2：有 ④，无核心与 ⑥。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(2_i16.to_le_bytes());
    bytes.extend((-1_i16).to_le_bytes());
    bytes.extend((-2_i16).to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_latlon_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.short_0c, Some(-1));
    assert_eq!(body.short_0a, Some(-2));
    assert!(body.numeric.is_none(), "版本 2 无内嵌核心");
    assert_eq!(body.short_08, None);
    assert_eq!(reader.position(), bytes.len());
    // 版本 1：仅 ②③（④⑤⑥ 流内无字段；原生内存默认 -1 不写入读取结果）。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(11_i16.to_le_bytes());
    bytes.extend(22_i16.to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_latlon_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.short_2c, 11);
    assert_eq!(body.short_2a, 22);
    assert_eq!(body.short_0c, None);
    assert_eq!(body.short_0a, None);
    assert!(body.numeric.is_none());
    assert_eq!(body.short_08, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn latlon_rejects_bad_versions_and_truncation() {
    for version in [5_i16, 6, 0x7fff, -3, 0x8000_u16 as i16] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 16]);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(
            read_latlon_format_body_10_1(&mut reader, &mut table, KEY).is_err(),
            "版本 {version} 应被拒绝"
        );
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
    let full = latlon_v4();
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_latlon_format_body_10_1(&mut reader, &mut table, KEY).is_err(),
            "前缀长度 {end} 应截断失败"
        );
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert!(table.is_empty(), "失败路径不修改登记表");
    }
}

#[test]
fn angle_version_branches_read_core_from_version_2() {
    // 版本 3 满形态。
    let body_bytes = angle_v3();
    let mut bytes = body_bytes.clone();
    bytes.push(0x11);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_angle_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        AngleFormatBody10_1 {
            key: KEY,
            version: 3,
            short_0c: 4,
            numeric: Some(Box::new(NumericFormatBody10_1 {
                key: KEY,
                version: 1,
                int32s: [9, 10, 11, 12],
                shorts: [0, 0, 0],
            })),
            short_0a: Some(5),
        }
    );
    assert_eq!(reader.position(), body_bytes.len());
    // 版本 2：有核心，无 ④。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(1_i16.to_le_bytes());
    bytes.extend(numeric_core([2, 2, 2, 2]));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_angle_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.short_0c, 1);
    assert!(body.numeric.is_some(), "版本 2 含内嵌核心");
    assert_eq!(body.short_0a, None, "版本 2 无 ④");
    assert_eq!(reader.position(), bytes.len());
    // 版本 1：仅 ②。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(6_i16.to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_angle_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.short_0c, 6);
    assert!(body.numeric.is_none(), "版本 1 无内嵌核心");
    assert_eq!(body.short_0a, None);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn angle_rejects_bad_versions_and_truncation() {
    for version in [4_i16, 0x7fff, -4, 0x8000_u16 as i16] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 12]);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(
            read_angle_format_body_10_1(&mut reader, &mut table, KEY).is_err(),
            "版本 {version} 应被拒绝"
        );
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
    let full = angle_v3();
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_angle_format_body_10_1(&mut reader, &mut table, KEY).is_err(),
            "前缀长度 {end} 应截断失败"
        );
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
}

#[test]
fn scientific_reads_int32_from_version_2_only() {
    // 版本 2：版本 + int32。
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(6_i32.to_le_bytes());
    bytes.extend([0x77]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_scientific_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        ScientificFormatBody10_1 {
            key: KEY,
            version: 2,
            int32_08: Some(6),
        }
    );
    assert_eq!(reader.position(), 6, "未知尾部字节不被消费");
    // 版本 1：流内无字段（原生置内存默认 6，读取结果为 None）。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(0xaa_bb_u16.to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_scientific_format_body_10_1(&mut reader, &mut table, KEY).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.int32_08, None, "版本 ≤1 流内无 int32");
    assert_eq!(reader.position(), 2, "版本 1 不消费后续字节");
    // 版本 3 与负值位模式拒绝。
    for version in [3_i16, -5, 0x8000_u16 as i16] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend([0; 4]);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(read_scientific_format_body_10_1(&mut reader, &mut table, KEY).is_err());
        assert_eq!(reader.position(), 0);
    }
    // 截断扫描。
    let full = {
        let mut b = 2_u16.to_le_bytes().to_vec();
        b.extend(6_i32.to_le_bytes());
        b
    };
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(read_scientific_format_body_10_1(&mut reader, &mut table, KEY).is_err());
        assert_eq!(reader.position(), 0);
    }
}

#[test]
fn custom_number_reads_string_and_rejects_bad_versions() {
    // 版本 1：版本 + AfCore String。
    let body_bytes = {
        let mut b = 1_u16.to_le_bytes().to_vec();
        b.extend(encoded("0.00E+00"));
        b
    };
    let mut bytes = body_bytes.clone();
    bytes.push(0x99);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body =
        read_custom_number_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        CustomNumberFormatBody10_1 {
            key: KEY,
            version: 1,
            string_08: "0.00E+00".to_string(),
        }
    );
    assert_eq!(reader.position(), body_bytes.len(), "未知尾部不被消费");
    // 版本 2 与负值位模式拒绝（`1 < (short)ver`）。
    for version in [2_i16, -6, 0x8000_u16 as i16] {
        let mut bytes = version.to_le_bytes().to_vec();
        bytes.extend(encoded("x"));
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert!(
            read_custom_number_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).is_err(),
            "版本 {version} 应被拒绝"
        );
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
    // 声明字符串超过 max_bytes：在任何分配之前拒绝。
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(encoded("超出限额"));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    assert!(read_custom_number_format_body_10_1(&mut reader, 4, &mut table, KEY).is_err());
    assert_eq!(reader.position(), 0);
    // 截断扫描。
    for end in 0..body_bytes.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&body_bytes[..end]);
        assert!(
            read_custom_number_format_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).is_err(),
            "前缀长度 {end} 应截断失败"
        );
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert!(table.is_empty(), "失败路径不修改登记表");
    }
}
