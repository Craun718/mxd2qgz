//! `hyperlink` 读取器的合成输入测试（FeatureLayer.dll 反编译字段序；样本
//! FL 后继段 IHyperlink 计数恒 0，无实例，全部为合成覆盖）。

use mxd2qgz::objects::hyperlink::{
    HYPERLINK_CLASS_ID, HyperlinkBody10_1, read_hyperlink_body_10_1,
};
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

const KEY: u32 = 99;
const MAX_BYTES: usize = 0x400;

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

/// Hyperlink 体（版本 1）：int32 + String + int32。
fn hyperlink_body(link: &str) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend(2_i32.to_le_bytes());
    bytes.extend(encoded(link));
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

#[test]
fn class_id_constant_matches_documented_clsid() {
    // {3036D35E-EDE5-11D0-87FE-080009EC732A} 内存序；FeatureLayer.dll
    // GetClassID 0x6f6d9310 立即数互证。
    assert_eq!(
        HYPERLINK_CLASS_ID,
        [
            0x5e, 0xd3, 0x36, 0x30, 0xe5, 0xed, 0xd0, 0x11, 0x87, 0xfe, 0x08, 0x00, 0x09, 0xec,
            0x73, 0x2a,
        ]
    );
}

#[test]
fn reads_version_1_body_with_all_fields() {
    let body_bytes = hyperlink_body("https://example.invalid/a?b=1");
    let mut bytes = body_bytes.clone();
    bytes.extend([0xaa, 0xbb]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_hyperlink_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).unwrap();
    assert_eq!(
        body,
        HyperlinkBody10_1 {
            key: KEY,
            version: 1,
            type_04: 2,
            link: "https://example.invalid/a?b=1".to_string(),
            int32_0c: -1,
        }
    );
    assert_eq!(reader.position(), body_bytes.len(), "未知尾部字节不被消费");
}

#[test]
fn version_0_is_accepted_like_signed_gate_allows() {
    // 反编译门 `1 < (short)ver`：版本 0 通过（写侧恒 1，读侧按门接受）。
    let mut bytes = 0_u16.to_le_bytes().to_vec();
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(encoded(""));
    bytes.extend(0_i32.to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_hyperlink_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).unwrap();
    assert_eq!(body.version, 0);
    assert_eq!(body.link, "");
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn rejects_out_of_range_versions_without_advancing() {
    // 版本 2 超限；负值位模式拒绝（较原生更严：原生门对负值放行）。
    for version in [2_i16, 3, 0x7fff, -1, i16::MIN] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend(hyperlink_body("x"));
        let mut table = RefTable10_1::new();
        table.register(1234);
        let mut reader = Reader::new(&bytes);
        let error = read_hyperlink_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert_eq!(table.len(), 1, "失败路径不修改登记表");
        assert!(table.contains(1234));
    }
}

#[test]
fn oversized_link_is_rejected_before_allocation() {
    let bytes = hyperlink_body("这是一个超过限额的链接目标");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    // 声明长度超过 max_bytes（4）：在任何读取与分配之前拒绝。
    assert!(read_hyperlink_body_10_1(&mut reader, 4, &mut table, KEY).is_err());
    assert_eq!(reader.position(), 0);
}

#[test]
fn truncated_bodies_fail_without_advancing() {
    let full = hyperlink_body("ab");
    for end in 0..full.len() {
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&full[..end]);
        assert!(
            read_hyperlink_body_10_1(&mut reader, MAX_BYTES, &mut table, KEY).is_err(),
            "前缀长度 {end} 应截断失败"
        );
        assert_eq!(reader.position(), 0, "失败路径不推进游标");
        assert!(table.is_empty(), "失败路径不修改登记表");
    }
}
