//! 将 ArcMap 10.1 `/Maps` 数据流的已确认前缀解析为中间模型。
//!
//! 本模块是显式 10.1 读取器与 [`crate::model`] 中间模型的接线层，仅调用
//! 以下已确认读取器，不新增任何字段解释：
//!
//! - [`crate::maps_stream::read_root_prefix_10_1`]：4 字节有符号数量、首个
//!   对象头、循环后的 4 字节尾部原始值（证据：`docs/arcmap-10.1-object-body.md`）；
//! - [`crate::object_stream::read_header_10_1`]：Map 对象头与嵌套空间参考对象头；
//! - [`crate::map_object::read_map_prefix_10_1`]：Map 对象体已确认前缀
//!   （架构版本 short、双名称字符串、嵌套空间参考对象，第 5 步起的字段一律不读）；
//! - [`crate::spatial_reference::read_projected_coordinate_system_10_1`]：
//!   ProjectedCoordinateSystem 对象体（经 Map 前缀读取器内部调用）。
//!
//! 模型填充规则（仅已确认字段）：
//!
//! - 数据框名称与第二名称 → [`DataFrame`](crate::model::DataFrame) 的
//!   `name` 与 `secondary_name`；
//! - 嵌套 PCS 的 ASCII WKT → [`CoordinateSystem`](crate::model::CoordinateSystem)
//!   的 `definition`（域字符串、垂向标志、精度标志与 double 已解析但语义
//!   未确认，不写入模型，仅在诊断中说明）；
//! - 图层、范围、工程名称、其余数据框对象与根尾部值之后的字节全部没有
//!   已确认证据，保留为 [`UnknownField`](crate::model::UnknownField) 或空
//!   字段，不做任何推断。
//!
//! 对象循环边界未闭合：对象体没有外部长度字段，本模块不跳过未知对象，
//! 因此无论数量是多少，最多只解析首个对象；其余对象全部保留为未知字节。
//! 正数量时根前缀读取器在首对象头之后读取的 4 字节按写侧证据
//! （数量 → 对象循环 → 循环尾部值）属于首对象体，不是循环尾部值，本模块
//! 不把它解释为尾部。
//!
//! 解析结果与诊断都是候选观察：单元测试仅验证代码自身契约，解析规则未在
//! ArcMap 10.1 或 QGIS 中人工目视验证；通过解析不代表 MXD 格式已验证。

use crate::map_object::{MAP_CLASS_ID, read_map_prefix_10_1};
use crate::maps_stream::read_root_prefix_10_1;
use crate::model::{CoordinateSystem, DataFrame, Map, UnknownField};
use crate::object_stream::{ObjectHeader, read_header_10_1};
use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// 数据框数量限额：进入对象循环前拒绝超过该值的数量，避免按异常数量申请解析工作。
pub const MAX_MAP_COUNT: u32 = 1024;

/// 单字段资源限额：约束名称字符串声明长度，并原样转发给嵌套 PCS 对象体
/// （约束 WKT 与域字符串长度），在任何读取与分配之前检查。
pub const MAX_FIELD_BYTES: usize = 1 << 20;

/// `/Maps` 已确认前缀的解析报告。
#[derive(Debug, Clone, PartialEq)]
pub struct MapsParseReport10_1 {
    /// 填充了已确认字段的中间模型；未确认字段保留为未知或空值。
    pub map: Map,
    /// 根前缀的有符号数量原始值（写侧为数据框个数）。
    pub count: i32,
    /// 根前缀读取器在首对象头之后读取的 4 字节原始位模式。
    ///
    /// 仅数量为 0（空循环）时它才是循环尾部值；数量为正时按写侧证据该位置
    /// 属于首对象体，本模块不解释，字段仅保留位模式供诊断。
    pub tail_raw: u32,
    /// 逐条诊断：已确认与未确认内容的边界、未知字节数及未验证声明。
    pub diagnostics: Vec<String>,
}

/// 用默认限额（[`MAX_MAP_COUNT`]、[`MAX_FIELD_BYTES`]）解析 `/Maps` 完整字节。
///
/// 输入须是 OLE `/Maps` 数据流的完整字节；解析失败返回明确错误且不返回
/// 部分模型。这是显式 10.1 解析路径，不识别 MXD 版本（版本候选请配合
/// `version` 命令或 [`crate::mxd_version::identify_version`] 使用）。
pub fn parse_maps_10_1(maps: &[u8]) -> io::Result<MapsParseReport10_1> {
    parse_maps_10_1_with_limits(maps, MAX_MAP_COUNT, MAX_FIELD_BYTES)
}

/// 以调用者提供的限额解析 `/Maps` 完整字节；限额语义见各常量文档。
pub fn parse_maps_10_1_with_limits(
    maps: &[u8],
    max_count: u32,
    max_bytes: usize,
) -> io::Result<MapsParseReport10_1> {
    let mut diagnostics = Vec::new();
    let mut reader = Reader::new(maps);
    let root = read_root_prefix_10_1(&mut reader, max_count)?;

    diagnostics.push(format!(
        "数量候选 {}：写侧该值为文档的数据框（Map 对象）个数",
        root.count
    ));

    let mut frames: Vec<DataFrame> = Vec::new();
    let mut map_unknown: Vec<UnknownField> = Vec::new();

    if root.count == 0 {
        // 空循环：此处的 4 字节才是读侧循环结束后无条件读取的尾部值。
        diagnostics.push(format!(
            "循环尾部值原始位模式 0x{:08x}（写侧 0/1 语义属证据性解读，本模块不解释）",
            root.tail_raw
        ));
        let consumed = reader.position();
        if root.remaining_bytes > 0 {
            map_unknown.push(UnknownField {
                stream: "/Maps".to_string(),
                offset: consumed as u64,
                bytes: maps[consumed..].to_vec(),
            });
            diagnostics.push(format!(
                "尾部值之后保留 {} 字节未知数据（流内偏移 0x{consumed:x} 起），不解释",
                root.remaining_bytes
            ));
        }
    } else {
        let first_header = root
            .first_header
            .expect("正数量时根前缀读取器必返回首对象头");
        match first_header {
            ObjectHeader::Null => {
                // 全零对象头：没有类标识可解析，其后字节（含根前缀读取器在
                // 空头后读取的 4 字节）全部保留为未知，不猜测空头的语义。
                diagnostics.push(
                    "首对象头为全零空引用标记：无类标识可解析，其后字节保留为未知".to_string(),
                );
                let consumed = reader.position();
                if root.remaining_bytes > 0 {
                    map_unknown.push(UnknownField {
                        stream: "/Maps".to_string(),
                        offset: consumed as u64,
                        bytes: maps[consumed..].to_vec(),
                    });
                }
            }
            ObjectHeader::Object { class_id, key: _ } if class_id == MAP_CLASS_ID => {
                let frame = parse_first_frame(maps, root.count, &mut diagnostics, max_bytes)?;
                frames.push(frame);
            }
            ObjectHeader::Object { class_id, key } => {
                // 首对象不是 Map 类：对象体无外部长度字段，无法跳过未知类
                // 继续解析；返回明确错误，不返回部分模型。
                let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "/Maps 首对象头类标识 {id}（键 {key}）不是 Map 类；对象体无外部长度字段，\
                         本解析路径无法跳过未知类继续解析"
                    ),
                ));
            }
        }
    }

    diagnostics.push(
        "解析仅覆盖显式 10.1 已确认前缀；单元测试不代表目标软件验证，\
         未在 ArcMap 10.1 或 QGIS 中人工目视验证"
            .to_string(),
    );

    Ok(MapsParseReport10_1 {
        map: Map {
            // 工程名称在 /Maps 中没有已确认字段，保留为未知。
            name: None,
            frames,
            unknown_fields: map_unknown,
        },
        count: root.count,
        tail_raw: root.tail_raw,
        diagnostics,
    })
}

/// 解析首个 Map 对象体的已确认前缀并构造数据框。
///
/// `maps` 是完整流字节；本函数从数量之后重新读取 Map 对象头（与根前缀读取
/// 器的确定性行为一致），再调用 [`read_map_prefix_10_1`] 读取对象体前缀。
/// 数量为正时根前缀读取器在对象头后消耗的 4 字节属于首对象体（见模块文档），
/// 因此未知区域的起点按对象体前缀的终点计算，与根前缀的消耗无关。
fn parse_first_frame(
    maps: &[u8],
    count: i32,
    diagnostics: &mut Vec<String>,
    max_bytes: usize,
) -> io::Result<DataFrame> {
    // 从数量之后重读首对象头：read_header_10_1 与根前缀读取器使用同一实现，
    // 头部消耗量确定（对象 20 字节），重读不引入新的解释。
    let mut body = Reader::new(maps);
    body.read_bytes(4)?;
    let header = read_header_10_1(&mut body)?;
    debug_assert!(matches!(
        header,
        ObjectHeader::Object { class_id, .. } if class_id == MAP_CLASS_ID
    ));

    let prefix = read_map_prefix_10_1(&mut body, max_bytes)?;

    diagnostics.push(format!(
        "数据框 1/{count}：名称 {:?}，第二名称 {:?}，架构版本 {}（>51 已由读取器拒绝）",
        prefix.name, prefix.secondary_name, prefix.version
    ));
    match &prefix.spatial_reference {
        Some(pcs) => diagnostics.push(format!(
            "空间参考：ProjectedCoordinateSystem（保存版本 {}，WKT {} 字节）；\
             域字符串、垂向标志、精度标志与 double 已由读取器解析但语义未确认，不写入模型",
            pcs.save_version,
            pcs.wkt.len()
        )),
        None => diagnostics
            .push("空间参考：嵌套对象头为 16 零字节空引用标记，该数据框未设置坐标系".to_string()),
    }

    // 已确认前缀的终点即未知区域起点：其中含其余数据框对象、图层、循环尾部
    // 值等未闭合内容，全部保留为未知字节。
    let unknown_offset = body.position();
    let unknown_len = prefix.remaining_bytes;
    if unknown_len > 0 {
        diagnostics.push(format!(
            "已确认前缀之后保留 {unknown_len} 字节未知数据（流内偏移 \
             0x{unknown_offset:x} 起）：Map::Load 第 5 步起的字段、其余数据框对象与循环尾部均未解析"
        ));
    }
    if count > 1 {
        diagnostics.push(format!(
            "其余 {} 个数据框对象未解析：对象循环边界未闭合，对象体无外部长度字段，不跳过未知对象",
            count - 1
        ));
    }

    Ok(DataFrame {
        name: prefix.name,
        secondary_name: Some(prefix.secondary_name),
        // 图层树没有已确认字段；对应内容保留在上面的未知字节中。
        children: Vec::new(),
        crs: prefix.spatial_reference.map(|pcs| CoordinateSystem {
            definition: pcs.wkt,
            unknown_fields: Vec::new(),
        }),
        // 显示范围字段位于未确认前缀中，保留为未知。
        extent: None,
        unknown_fields: if unknown_len > 0 {
            vec![UnknownField {
                stream: "/Maps".to_string(),
                offset: unknown_offset as u64,
                bytes: maps[unknown_offset..].to_vec(),
            }]
        } else {
            Vec::new()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
    use std::io::ErrorKind;

    /// 样本首个 PCS 对象键（`/Maps:0x30` 键 30）；键值原样保留，不参与分派。
    const SAMPLE_PCS_KEY: u32 = 0x1e;
    /// 样本记载的 Map 架构版本（`/Maps:0x18`）。
    const SAMPLE_VERSION: u16 = 51;
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

    /// 版本 6 布局的 PCS 对象体：4 个空域字符串、垂向标志 0、标志 0x0001、
    /// 样本记载的 7+3 个 double（与 tests/map_object.rs 的合成体一致；
    /// 集成测试与单元测试无法互相引用，按同一证据独立构造）。
    fn pcs_body(wkt: &str) -> Vec<u8> {
        let mut bytes = (6_u16).to_le_bytes().to_vec();
        bytes.extend((wkt.len() as u32).to_le_bytes());
        bytes.extend(wkt.as_bytes());
        for _ in 0..4 {
            bytes.extend(0_u32.to_le_bytes()); // 4 个空域字符串
        }
        bytes.push(0x00);
        bytes.extend(0x0001_u16.to_le_bytes());
        for value in SAMPLE_SEVEN_DOUBLES {
            bytes.extend(value.to_le_bytes());
        }
        for value in SAMPLE_THREE_DOUBLES {
            bytes.extend(value.to_le_bytes());
        }
        bytes
    }

    /// Map 对象体的已确认前缀：版本 51 + 双名称 + 嵌套 PCS（对象头 + 体）。
    fn frame_body(name: &str, secondary: &str, wkt: &str) -> Vec<u8> {
        let mut bytes = SAMPLE_VERSION.to_le_bytes().to_vec();
        bytes.extend(encoded(name));
        bytes.extend(encoded(secondary));
        bytes.extend(PROJECTED_COORDINATE_SYSTEM_CLASS_ID);
        bytes.extend(SAMPLE_PCS_KEY.to_le_bytes());
        bytes.extend(pcs_body(wkt));
        bytes
    }

    /// 完整 `/Maps` 合成流：数量 + 首对象头（Map 类 + 键）+ 对象体 + 未知尾部。
    fn maps_stream(count: i32, body: &[u8], tail: &[u8]) -> Vec<u8> {
        let mut bytes = count.to_le_bytes().to_vec();
        bytes.extend(MAP_CLASS_ID);
        bytes.extend(0x2a_u32.to_le_bytes());
        bytes.extend(body);
        bytes.extend(tail);
        bytes
    }

    /// 单数据框参考流：与仓库样本布局对齐（「新建数据框」、空第二名称、PCS）。
    fn sample_stream(tail: &[u8]) -> Vec<u8> {
        maps_stream(
            1,
            &frame_body("新建数据框", "", "PROJCS[\"CGCS2000\"]"),
            tail,
        )
    }

    #[test]
    fn fills_names_and_wkt_into_model_and_preserves_unknown_tail() {
        let tail = [0xaa, 0xbb, 0xcc];
        let stream = sample_stream(&tail);
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.count, 1);
        assert_eq!(report.map.frames.len(), 1);
        let frame = &report.map.frames[0];
        assert_eq!(frame.name, "新建数据框");
        assert_eq!(frame.secondary_name.as_deref(), Some(""));
        assert_eq!(frame.children, []);
        assert_eq!(frame.extent, None);
        let crs = frame.crs.as_ref().unwrap();
        assert_eq!(crs.definition, "PROJCS[\"CGCS2000\"]");
        assert_eq!(crs.unknown_fields, []);
        // 未知尾部 = Map::Load 第 5 步起的全部字节，含调用者追加的尾字节；
        // 起点是已确认前缀的终点。
        assert_eq!(frame.unknown_fields.len(), 1);
        let unknown = &frame.unknown_fields[0];
        assert_eq!(unknown.stream, "/Maps");
        assert_eq!(unknown.offset as usize, stream.len() - tail.len());
        assert_eq!(unknown.bytes, tail);
        // 工程名称与根级未知字段均未确认。
        assert_eq!(report.map.name, None);
        assert_eq!(report.map.unknown_fields, []);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("其余") || note.contains("未知"))
        );
    }

    #[test]
    fn zero_count_reports_tail_raw_and_keeps_rest_unknown() {
        let mut stream = 0_i32.to_le_bytes().to_vec();
        stream.extend(1_u32.to_le_bytes()); // 循环尾部值原始位模式
        stream.extend_from_slice(&[0xde, 0xad]); // 尾部值之后的未知字节
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.count, 0);
        assert_eq!(report.tail_raw, 1);
        assert_eq!(report.map.frames, []);
        assert_eq!(report.map.unknown_fields.len(), 1);
        let unknown = &report.map.unknown_fields[0];
        assert_eq!(unknown.offset, 8);
        assert_eq!(unknown.bytes, [0xde, 0xad]);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("0x00000001"))
        );
    }

    #[test]
    fn null_first_header_keeps_everything_unknown_without_guessing() {
        let mut stream = 2_i32.to_le_bytes().to_vec();
        stream.extend([0_u8; 16]); // 全零对象头
        stream.extend(0_u32.to_le_bytes()); // 根前缀读取器照读的 4 字节
        stream.extend_from_slice(&[0x11, 0x22]);
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.map.frames, []);
        assert_eq!(report.map.unknown_fields.len(), 1);
        assert_eq!(report.map.unknown_fields[0].offset, 24);
        assert_eq!(report.map.unknown_fields[0].bytes, [0x11, 0x22]);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("空引用标记"))
        );
    }

    #[test]
    fn only_first_frame_is_parsed_when_count_exceeds_one() {
        let stream = maps_stream(3, &frame_body("甲框", "", "PROJCS[\"X\"]"), &[0x55, 0x66]);
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.map.frames.len(), 1);
        assert_eq!(report.map.frames[0].name, "甲框");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("其余 2 个数据框对象未解析"))
        );
        // 第 2/3 个数据框对象与循环尾部都落在未知字节中。
        let unknown = &report.map.frames[0].unknown_fields[0];
        assert_eq!(unknown.bytes.len(), stream.len() - unknown.offset as usize);
    }

    #[test]
    fn empty_spatial_reference_yields_frame_without_crs() {
        // 嵌套空间参考为 16 零字节空引用标记（样本语义：未设置坐标系）。
        let mut body = SAMPLE_VERSION.to_le_bytes().to_vec();
        body.extend(encoded("空框"));
        body.extend(encoded(""));
        body.extend([0_u8; 16]);
        let stream = maps_stream(1, &body, &[]);
        let report = parse_maps_10_1(&stream).unwrap();

        let frame = &report.map.frames[0];
        assert_eq!(frame.crs, None);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("未设置坐标系"))
        );
    }

    #[test]
    fn negative_and_over_limit_counts_are_rejected() {
        let mut negative = (-1_i32).to_le_bytes().to_vec();
        negative.extend([0; 24]);
        let error = parse_maps_10_1(&negative).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);

        let mut over_limit = (MAX_MAP_COUNT as i32 + 1).to_le_bytes().to_vec();
        over_limit.extend([0; 24]);
        let error = parse_maps_10_1(&over_limit).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn non_map_first_class_is_rejected_without_skipping() {
        let mut stream = 1_i32.to_le_bytes().to_vec();
        let mut class_id = MAP_CLASS_ID;
        class_id[0] = 0x77; // 任意非 Map 类标识
        stream.extend(class_id);
        stream.extend(1_u32.to_le_bytes());
        stream.extend([0xff; 8]);
        let error = parse_maps_10_1(&stream).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert!(error.to_string().contains("不是 Map 类"));
    }

    #[test]
    fn unknown_nested_class_fails_with_explicit_error() {
        let mut body = SAMPLE_VERSION.to_le_bytes().to_vec();
        body.extend(encoded("框"));
        body.extend(encoded(""));
        let mut nested = [0_u8; 16];
        nested[0] = 0x99; // 非 PCS、非全零的嵌套类标识
        body.extend(nested);
        body.extend(7_u32.to_le_bytes());
        let stream = maps_stream(1, &body, &[]);
        let error = parse_maps_10_1(&stream).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
    }

    #[test]
    fn truncated_streams_fail_without_partial_model() {
        let tail = [0xaa, 0xbb];
        let stream = sample_stream(&tail);
        // 已确认前缀的终点：截断进入前缀内部的任何长度都必须整体失败
        // （全部严格读取器都是事务性的）。
        let prefix_end = stream.len() - tail.len();
        for length in 0..prefix_end {
            assert!(
                parse_maps_10_1(&stream[..length]).is_err(),
                "前缀长度 {length} 不应解析成功"
            );
        }
        // 截断落在未解释的尾部区域时前缀仍完整，解析成功且未知字节随之缩短；
        // 恰好截到前缀终点时未知字节为空（unknown_fields 无条目）。
        let report = parse_maps_10_1(&stream[..prefix_end]).unwrap();
        assert_eq!(report.map.frames[0].unknown_fields, []);
        for length in (prefix_end + 1)..stream.len() {
            let report = parse_maps_10_1(&stream[..length]).unwrap();
            assert_eq!(
                report.map.frames[0].unknown_fields[0].bytes.len(),
                length - prefix_end
            );
        }
        assert!(parse_maps_10_1(&stream).is_ok());
        assert!(parse_maps_10_1(&[]).is_err());
    }

    #[test]
    fn resource_limit_rejects_oversized_declared_name() {
        // 声明长度超过 max_bytes：在任何读取与分配之前显式拒绝。
        let mut body = SAMPLE_VERSION.to_le_bytes().to_vec();
        body.extend(u32::MAX.to_le_bytes());
        body.extend([0x41; 8]);
        let stream = maps_stream(1, &body, &[]);
        let error = parse_maps_10_1_with_limits(&stream, MAX_MAP_COUNT, 16).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        // 限额同样转发给嵌套 PCS：WKT 超限时同样拒绝（小 WKT 流不受影响）。
        let short = maps_stream(1, &frame_body("框", "", "P"), &[]);
        assert!(parse_maps_10_1_with_limits(&short, MAX_MAP_COUNT, 16).is_ok());
        let long_wkt = maps_stream(1, &frame_body("框", "", "PROJCS[\"超限WKT字符串\"]"), &[]);
        let error = parse_maps_10_1_with_limits(&long_wkt, MAX_MAP_COUNT, 16).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }
}
