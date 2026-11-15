//! `objects::geometry` 的单元测试：Polyline 对象体（`IPersistStream::Load`
//! 版本 2 布局）的合成输入与仓库参考样本形态镜像。全部为合成或样本字节
//! 形态镜像，不依赖外部文件；样本吻合为辅助观察，不构成对读取器正确性的
//! 验证（见 `docs/arcmap-10.1-geometry.md`）。

use mxd2qgz::objects::envelope::SpatialReferenceEntry10_1;
use mxd2qgz::objects::geometry::{
    POLYLINE_CLASS_ID, PolylineBody10_1, read_geometry_body_of_class_10_1, read_polyline_body_10_1,
};
use mxd2qgz::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;
use std::io::ErrorKind;

/// 样本键 1618（/Maps:0x18724）记录块的 80 字节字节形态：类型 3、包络
/// (37398133.4893, 2528216.1470, 37398162.2060, 2528216.1470)、路径数 1、
/// 点数 2、路径起始 `[0]`、2 个二维点（辅助观察镜像）。
const SAMPLE_RECORD_KEY_1618: [u8; 80] = [
    0x03, 0x00, 0x00, 0x00, 0x07, 0x2a, 0xea, 0xab, 0x33, 0xd5, 0x81, 0x41, 0x70, 0x4a, 0xd2, 0x12,
    0xec, 0x49, 0x43, 0x41, 0x40, 0xf1, 0xa5, 0x91, 0x34, 0xd5, 0x81, 0x41, 0x70, 0x4a, 0xd2, 0x12,
    0xec, 0x49, 0x43, 0x41, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x07, 0x2a, 0xea, 0xab, 0x33, 0xd5, 0x81, 0x41, 0x70, 0x4a, 0xd2, 0x12, 0xec, 0x49, 0x43, 0x41,
    0x40, 0xf1, 0xa5, 0x91, 0x34, 0xd5, 0x81, 0x41, 0x70, 0x4a, 0xd2, 0x12, 0xec, 0x49, 0x43, 0x41,
];

/// 版本 short + 声明长度 + 记录字节块 + 16 字节全零空标记（版本 2 的
/// LoadObject ISpatialReference 空引用，样本 16/16 形态）。
fn sample_shaped_body() -> Vec<u8> {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend((SAMPLE_RECORD_KEY_1618.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RECORD_KEY_1618);
    bytes.extend_from_slice(&[0_u8; 16]);
    bytes
}

/// 记录布局常量（44 + 路径数×4 + 点数×16）下的合成记录。
fn record_bytes(kind: u32, envelope: &[f64; 4], starts: &[u32], points: &[[f64; 2]]) -> Vec<u8> {
    let mut bytes = kind.to_le_bytes().to_vec();
    for value in envelope {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend((starts.len() as u32).to_le_bytes());
    bytes.extend((points.len() as u32).to_le_bytes());
    for start in starts {
        bytes.extend(start.to_le_bytes());
    }
    for [x, y] in points {
        bytes.extend(x.to_le_bytes());
        bytes.extend(y.to_le_bytes());
    }
    bytes
}

/// 版本 short + 声明长度 + 记录 + 附加尾段的完整对象体。
fn body_with(version: u16, record: &[u8], tail: &[u8]) -> Vec<u8> {
    let mut bytes = version.to_le_bytes().to_vec();
    bytes.extend((record.len() as u32).to_le_bytes());
    bytes.extend_from_slice(record);
    bytes.extend_from_slice(tail);
    bytes
}

/// 合成包络坐标（次序关系与样本一致：d0 ≤ d2、d1 ≤ d3）。
const COORDS: [f64; 4] = [10.0, 20.0, 30.0, 40.0];
/// 两个合成点（落在包络角上，与样本「包络即点的包络」形态一致）。
const POINTS: [[f64; 2]; 2] = [[10.0, 20.0], [30.0, 40.0]];

/// 追加嵌套对象条目头（16 字节类标识 + 4 字节键；回引与新对象同构）。
fn push_entry_header(bytes: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
    bytes.extend_from_slice(class_id);
    bytes.extend(key.to_le_bytes());
}

/// 版本 1 布局的最小合法 PCS 对象体（与 tests/envelope.rs 同形）。
fn minimal_pcs_body(wkt: &str) -> Vec<u8> {
    let mut bytes = 1_u16.to_le_bytes().to_vec();
    bytes.extend((wkt.len() as u32).to_le_bytes());
    bytes.extend(wkt.as_bytes());
    bytes
}

#[test]
fn polyline_class_id_constant_matches_documented_guid() {
    // 证据：{30707210-52D5-11D0-A8F2-00608C85EDE5}，esri.clsid.xml 声明
    // Geometry.dll 模块；Geometry.dll 10.1.0.3035 GetClassID 0x719402d0 以
    // movq 复制常量 0x71a1187c（本轮取证，证据等级「确定的」）；内存序 =
    // Data1-3 小端 + Data4 原序，与样本 /Maps:0x18724 起的对象头逐字节相同
    // （辅助观察）。
    assert_eq!(
        POLYLINE_CLASS_ID,
        [
            0x10, 0x72, 0x70, 0x30, 0xd5, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85,
            0xed, 0xe5,
        ]
    );
    assert_ne!(POLYLINE_CLASS_ID, PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
}

/// 从记录块内按布局读取一个小端 double（期望值与输入字节同源，保证按位相等）。
fn record_f64(record: &[u8; 80], byte_offset: usize) -> f64 {
    f64::from_le_bytes(record[byte_offset..byte_offset + 8].try_into().unwrap())
}

#[test]
fn reads_sample_shaped_version_two_body_with_null_spatial_reference() {
    // 样本 16/16 形态：版本 2 + 80 字节记录 + 16 字节全零空标记。期望值从
    // 同一 fixture 字节解码（按位相等），不引入十进制近似。
    let bytes = sample_shaped_body();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(body.version, 2);
    assert_eq!(body.record.kind, 3);
    assert_eq!(
        body.record.envelope,
        [
            record_f64(&SAMPLE_RECORD_KEY_1618, 4),
            record_f64(&SAMPLE_RECORD_KEY_1618, 12),
            record_f64(&SAMPLE_RECORD_KEY_1618, 20),
            record_f64(&SAMPLE_RECORD_KEY_1618, 28),
        ]
    );
    assert_eq!(body.record.path_starts, [0]);
    assert_eq!(body.record.points.len(), 2);
    assert_eq!(
        body.record.points[0],
        [
            record_f64(&SAMPLE_RECORD_KEY_1618, 48),
            record_f64(&SAMPLE_RECORD_KEY_1618, 56),
        ]
    );
    assert_eq!(
        body.record.points[1],
        [
            record_f64(&SAMPLE_RECORD_KEY_1618, 64),
            record_f64(&SAMPLE_RECORD_KEY_1618, 72),
        ]
    );
    // 样本值形态辅助观察：点的包络即记录包络（水平引线，Y0 == Y1）。
    assert_eq!(body.record.envelope[0], body.record.points[0][0]);
    assert_eq!(body.record.envelope[1], body.record.points[0][1]);
    assert_eq!(body.record.envelope[2], body.record.points[1][0]);
    assert_eq!(body.record.envelope[3], body.record.points[1][1]);
    assert_eq!(
        body.spatial_reference,
        Some(SpatialReferenceEntry10_1::Null)
    );
    // 恰好消费全部字节：6 字节包装头 + 80 字节记录 + 16 字节空标记。
    assert_eq!(reader.position(), bytes.len());
    assert!(table.is_empty());
}

#[test]
fn synthetic_record_round_trips_declared_layout() {
    let record = record_bytes(3, &COORDS, &[0], &POINTS);
    let bytes = body_with(2, &record, &[0_u8; 16]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(body.record.kind, 3);
    assert_eq!(body.record.envelope, COORDS);
    assert_eq!(body.record.path_starts, [0]);
    assert_eq!(body.record.points, POINTS);
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn reads_empty_geometry_record_of_four_bytes() {
    // ShapePolyline::Parse 对类型 0 提前返回（*consumed = 4）：4 字节记录、
    // 无包络无计数无点。版本 1 无空间参考段。
    let bytes = body_with(1, &0_u32.to_le_bytes(), b"tail");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(body.version, 1);
    assert_eq!(body.record.kind, 0);
    assert!(body.record.path_starts.is_empty());
    assert!(body.record.points.is_empty());
    assert_eq!(body.spatial_reference, None);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

#[test]
fn version_one_skips_spatial_reference_segment() {
    // 版本 ≤1：记录之后直接是后续数据（原生 `jl` 跳过 LoadObject 段）。
    let record = record_bytes(3, &COORDS, &[0], &POINTS);
    let bytes = body_with(1, &record, b"next");
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(body.spatial_reference, None);
    assert_eq!(reader.read_bytes(4).unwrap(), b"next");
}

#[test]
fn back_reference_spatial_reference_keeps_registration_table() {
    // 20 字节回引条目（既有键 30 = 样本 Map 的 PCS 键形态）。
    let mut table = RefTable10_1::new();
    table.register(30);
    table.register(7);
    let record = record_bytes(3, &COORDS, &[0], &POINTS);
    let mut bytes = body_with(2, &record, &[]);
    push_entry_header(&mut bytes, &PROJECTED_COORDINATE_SYSTEM_CLASS_ID, 30);
    bytes.extend_from_slice(b"tail");
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert_eq!(
        body.spatial_reference,
        Some(SpatialReferenceEntry10_1::BackReference {
            class_id: PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            key: 30,
        })
    );
    assert_eq!(table.len(), 2);
    assert_eq!(reader.read_bytes(4).unwrap(), b"tail");
}

#[test]
fn new_projected_spatial_reference_body_is_parsed() {
    // 新键 + 投影坐标系：嵌套 PCS 体被解析并消费。
    let record = record_bytes(3, &COORDS, &[0], &POINTS);
    let mut bytes = body_with(2, &record, &[]);
    push_entry_header(&mut bytes, &PROJECTED_COORDINATE_SYSTEM_CLASS_ID, 81);
    bytes.extend(minimal_pcs_body("PROJCS[\"x\"]"));
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    match body.spatial_reference {
        Some(SpatialReferenceEntry10_1::Projected { key, .. }) => assert_eq!(key, 81),
        other => panic!("嵌套 PCS 未解析: {other:?}"),
    }
    // 新键登记先于对象体解析（登记先于持久化的读侧对偶）。
    assert!(table.contains(81));
    assert_eq!(reader.position(), bytes.len());
}

#[test]
fn rejects_versions_beyond_native_gate_and_negative_bit_patterns() {
    let record = record_bytes(3, &COORDS, &[0], &POINTS);
    for version in [3_u16, 4, 0x8000, 0xFFFF] {
        let bytes = body_with(version, &record, &[]);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let error = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "版本 {version}");
        // 失败不推进游标、不修改登记表。
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
}

#[test]
fn rejects_declared_length_exceeding_resource_limit() {
    let mut bytes = 2_u16.to_le_bytes().to_vec();
    bytes.extend(0xFFFF_FFFF_u32.to_le_bytes());
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    // max_bytes = 0：声明长度 0xFFFF_FFFF 被限额拒绝（原生无上限，较原生更严）。
    let error = read_polyline_body_10_1(&mut reader, 0, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);
    // 限额恰好容纳完整体时成功。
    let full = sample_shaped_body();
    let mut reader = Reader::new(&full);
    let record_len = 6 + SAMPLE_RECORD_KEY_1618.len() as usize;
    read_polyline_body_10_1(&mut reader, record_len, &mut table).unwrap();
    assert_eq!(reader.position(), full.len());
}

#[test]
fn rejects_truncated_bodies_at_every_prefix() {
    let bytes = sample_shaped_body();
    let mut table = RefTable10_1::new();
    table.register(5);
    for end in 0..bytes.len() {
        let mut table = table.clone();
        let mut reader = Reader::new(&bytes[..end]);
        assert!(
            read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).is_err(),
            "前缀长度 {end} 应失败"
        );
        // 失败不推进游标、不修改登记表（既有键保留、无新键）。
        assert_eq!(reader.position(), 0, "前缀长度 {end}");
        assert_eq!(table.len(), 1);
        assert!(table.contains(5));
    }
}

#[test]
fn rejects_inconsistent_lengths_and_degenerate_counts() {
    // 声明长度与布局常量不一致：44 + 1×4 + 2×16 = 80，构造 79 与 81。
    let mut record = record_bytes(3, &COORDS, &[0], &POINTS);
    record.pop();
    let bytes = body_with(2, &record, &[]);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let error = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    // 原生同样拒绝的单点折线（0x80040221）。
    let single = record_bytes(3, &COORDS, &[0], &[[10.0, 20.0]]);
    let bytes = body_with(2, &single, &[]);
    let mut reader = Reader::new(&bytes);
    let error = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    // 声明长度 4 但类型非 0（原生会越界继续解析，本读取器拒绝）。
    let bytes = body_with(2, &3_u32.to_le_bytes(), &[]);
    let mut reader = Reader::new(&bytes);
    let error = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(reader.position(), 0);

    // 声明长度 44（无路径无点）：布局常量恰闭合，读为空集。
    let mut record = Vec::new();
    record.extend(3_u32.to_le_bytes());
    for value in COORDS {
        record.extend(value.to_le_bytes());
    }
    record.extend(0_u32.to_le_bytes());
    record.extend(0_u32.to_le_bytes());
    // 版本 2 在记录之后还有空间参考段：附 16 字节空标记。
    let bytes = body_with(2, &record, &[0_u8; 16]);
    let mut reader = Reader::new(&bytes);
    let body = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap();
    assert!(body.record.path_starts.is_empty());
    assert!(body.record.points.is_empty());
    // 声明长度 4 但类型非 0 已拒绝；声明长度 0/1..3 不足头部，同样拒绝。
    for len in [0_u32, 1, 2, 3] {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend(len.to_le_bytes());
        bytes.extend([0_u8; 8]);
        let mut reader = Reader::new(&bytes);
        let error = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData, "声明长度 {len}");
    }
}

#[test]
fn unknown_nested_spatial_reference_class_is_unsupported_and_rolled_back() {
    // 新键 + 非投影坐标系类：Unsupported，游标与登记表整体回滚。
    let record = record_bytes(3, &COORDS, &[0], &POINTS);
    let mut bytes = body_with(2, &record, &[]);
    push_entry_header(&mut bytes, &[0xa5; 16], 90);
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let error = read_polyline_body_10_1(&mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(!table.contains(90));
}

#[test]
fn dispatch_closes_polyline_and_rejects_other_geometry_classes() {
    let bytes = sample_shaped_body();
    let mut table = RefTable10_1::new();
    let mut reader = Reader::new(&bytes);
    let body: PolylineBody10_1 =
        read_geometry_body_of_class_10_1(&POLYLINE_CLASS_ID, &mut reader, usize::MAX, &mut table)
            .unwrap();
    assert_eq!(body.version, 2);
    assert_eq!(reader.position(), bytes.len());

    // 同族未开放类：Unsupported，游标与登记表不动。
    let mut reader = Reader::new(&bytes);
    let other = [0xa5; 16];
    let error =
        read_geometry_body_of_class_10_1(&other, &mut reader, usize::MAX, &mut table).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(reader.position(), 0);
    assert!(table.is_empty());
}
