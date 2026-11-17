//! 将 ArcMap 10.1 `/Maps` 数据流的已确认前缀与后缀解析为中间模型。
//!
//! 本模块是显式 10.1 读取器与 [`crate::model`] 中间模型的接线层，仅调用
//! 以下已确认读取器，不新增任何字段解释：
//!
//! - [`crate::maps::maps_stream::read_root_prefix_10_1`]：4 字节有符号数量、首个
//!   对象头、循环后的 4 字节尾部原始值（证据：`docs/arcmap-10.1-object-body.md`）；
//! - [`crate::storage::object_stream::read_header_10_1`]：Map 对象头；
//! - [`crate::maps::map_object::read_map_prefix_10_1`]：Map 对象体已确认前缀
//!   （架构版本 short、双名称字符串、嵌套空间参考对象）；
//! - [`crate::objects::spatial_reference::read_projected_coordinate_system_10_1`]：
//!   ProjectedCoordinateSystem 对象体（经 Map 前缀读取器内部调用）；
//! - [`crate::maps::map_object::read_map_suffix_10_1`]：Map 对象体后缀序列
//!   （图层列表 → 基本图形层 → MapSurround 循环 → WKSEnvelope → 两个
//!   int32 → IEnvelope → 书签循环，到书签循环为止，其后保留为未知尾部；
//!   内部经 [`crate::layers::layer_list::read_layer_list_10_1`] 读取图层列表）。
//!
//! 模型填充规则（仅已确认字段）：
//!
//! - 数据框名称与第二名称 → [`DataFrame`](crate::model::DataFrame) 的
//!   `name` 与 `secondary_name`；
//! - 嵌套 PCS 的 ASCII WKT → [`CoordinateSystem`](crate::model::CoordinateSystem)
//!   的 `definition`（域字符串、垂向标志、精度标志与 double 已解析但语义
//!   未确认，不写入模型，仅在诊断中说明）；
//! - WKSEnvelope 到达时 → `DataFrame.extent`（`[XMin,YMin,XMax,YMax]`，
//!   写方向确定）；未到达时保留 `None` 并在诊断中说明原因；
//! - 图层条目经 [`crate::layers::layer_list`] 的类分派解析出体前缀时构造
//!   [`LayerNode`](crate::model::LayerNode)（FeatureLayer → 名称/第二名称/
//!   可见性/数据源链；FDOGraphicsLayer → 以其关联要素图层的名称/可见性/
//!   数据源构造节点（第六轮），委派注记图层只进诊断；GroupLayer → 名称/
//!   可见性与经类分派递归的子图层节点），按条目顺序进入 `children`（绘制
//!   顺序候选）；回引条目命中先前已解析键时复用同对象节点。数据源链
//!   （FeatureClassName → WorkspaceName，GdbCore.dll 两个 Load 已闭合，第
//!   六轮）填充 `connection`（要素类名）与 `path`（工作空间路径，语义候选）；
//!   CompositeGraphicsLayer 图形层、未解析体与解析失败的条目不构造节点，
//!   体剩余/整段未解析体按条目保留为数据框未知字段，停止点之后的全部字节
//!   保留为尾部未知字段；
//! - 活动图层索引与各对象槽位只进诊断与报告
//!   （[`MapsParseReport10_1::layer_list`]）；
//! - 工程名称、未闭合的数据框对象与根尾部值之后的字段保留为
//!   [`UnknownField`](crate::model::UnknownField) 或空字段。
//!
//! 仅当当前 Map 对象体到达已确认的结束位置时，才循序读下一对象头。整个根
//! 循环完成后读出实际根尾值；正数量时，根前缀在首对象头后试读的 4 字节属于
//! 对象体，不解释为根尾。未知对象没有通用长度，不跳过。
//!
//! 解析失败（根前缀错误、首对象非 Map、已确认前缀或后缀内的截断与非法值）
//! 返回明确错误且不返回部分模型；后缀在结构停止点（标志 1 新对象体、未接入
//! 读取器的新对象体、书签循环之后的未闭合区）的输出是成功返回 + 未知尾部。
//! 解析结果与诊断都是候选观察：单元测试仅验证代码自身契约，解析规则未在
//! ArcMap 10.1 或 QGIS 中人工目视验证；通过解析不代表 MXD 格式已验证。

use crate::layers::fdo_graphics_layer::{AssociatedLayer10_1, ElementCollectionSlot10_1};
use crate::layers::feature_layer::{
    DataSource10_1, DatasetNameEntry10_1, FeatureClassNameNested10_1, WorkspaceNameEntry10_1,
};
use crate::layers::layer_list::{LayerBodyPrefix10_1, LayerEntryObject, LayerList10_1};
use crate::maps::map_object::{
    MAP_CLASS_ID, MapSurroundElement10_1, ObjectSlot10_1, read_map_prefix_object_stream_10_1,
    read_map_suffix_10_1,
};
use crate::maps::maps_stream::read_root_prefix_10_1;
use crate::model::{
    CoordinateSystem, DataFrame, DataSource, Extent, Layer, LayerGroup, LayerNode, Map,
    ProjectedCoordinateSystem, UnknownField,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::storage::object_stream::ObjectHeader;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 数据框数量限额：进入对象循环前拒绝超过该值的数量，避免按异常数量申请解析工作。
pub const MAX_MAP_COUNT: u32 = 1024;

/// 单字段资源限额：约束名称字符串声明长度，并原样转发给嵌套 PCS 对象体
/// （约束 WKT 与域字符串长度），在任何读取与分配之前检查。
pub const MAX_FIELD_BYTES: usize = 1 << 20;

/// `/Maps` 已确认前缀与后缀的解析报告。
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
    /// 仅在对象循环完整消费后确认的根尾值（流内偏移、原始位模式）。
    pub confirmed_tail: Option<(usize, u32)>,
    /// 首个数据框的图层列表解析结果（含条目、活动图层索引与停止信息）。
    ///
    /// 活动图层索引按约定放报告不放模型。仅存在被解析的 Map 对象时为
    /// `Some`（数量为 0 或首对象头为空标记时为 `None`）。
    pub layer_list: Option<LayerList10_1>,
    /// `/Version` 识别候选的摘要行。
    ///
    /// `/Maps` 解析不读取其他数据流，本字段由调用方（`parse` 命令）按需
    /// 填充，库内解析路径保持为空；每行都是候选观察，未经目标软件验证。
    pub version_candidates: Vec<String>,
    /// 逐条诊断：已确认与未确认内容的边界、未知字节数及未验证声明。
    pub diagnostics: Vec<String>,
}

/// 用默认限额（[`MAX_MAP_COUNT`]、[`MAX_FIELD_BYTES`]）解析 `/Maps` 完整字节。
///
/// 输入须是 OLE `/Maps` 数据流的完整字节；解析失败返回明确错误且不返回
/// 部分模型。这是显式 10.1 解析路径，不识别 MXD 版本（版本候选请配合
/// `version` 命令或 [`crate::version::mxd_version::identify_version`] 使用）。
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
    let mut report_layer_list: Option<LayerList10_1> = None;
    let mut confirmed_tail = None;

    if root.count == 0 {
        // 空循环：此处的 4 字节才是读侧循环结束后无条件读取的尾部值。
        diagnostics.push(format!(
            "循环尾部值原始位模式 0x{:08x}（写侧 0/1 语义属证据性解读，本模块不解释）",
            root.tail_raw
        ));
        let consumed = reader.position();
        confirmed_tail = Some((consumed - 4, root.tail_raw));
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
                let consumed = reader.position() - 4;
                if consumed < maps.len() {
                    map_unknown.push(UnknownField {
                        stream: "/Maps".to_string(),
                        offset: consumed as u64,
                        bytes: maps[consumed..].to_vec(),
                    });
                }
            }
            ObjectHeader::Object { class_id, key: _ } if class_id == MAP_CLASS_ID => {
                let mut refs = RefTable10_1::new();
                let mut map_indices = std::collections::HashMap::<u32, (usize, usize)>::new();
                let mut reference_copy_budget = maps.len();
                let mut objects = Reader::new(maps).reposition_to(4);
                let mut complete = true;
                for index in 0..root.count {
                    let entry_offset = objects.position();
                    let entry = read_object_entry_10_1(&mut objects, &mut refs);
                    match entry {
                        Ok(ObjectEntry10_1::NewObject { class_id, key })
                            if class_id == MAP_CLASS_ID =>
                        {
                            let body = parse_frame(
                                maps,
                                objects.position(),
                                index,
                                root.count,
                                &mut diagnostics,
                                max_bytes,
                                &mut refs,
                            );
                            let (frame, layer_list, end) = match body {
                                Ok(result) => result,
                                Err(error) if index == 0 => return Err(error),
                                Err(error) => {
                                    diagnostics.push(format!(
                                        "数据框体读取失败（0x{entry_offset:x}）：{error}"
                                    ));
                                    preserve_maps_tail(maps, entry_offset, &mut map_unknown);
                                    complete = false;
                                    break;
                                }
                            };
                            if let Some(end) = end {
                                map_indices.insert(key, (frames.len(), end - entry_offset));
                            }
                            frames.push(frame);
                            if index == 0 {
                                report_layer_list = layer_list;
                            }
                            let Some(end) = end else {
                                complete = false;
                                break;
                            };
                            objects = objects.reposition_to(end);
                        }
                        Ok(ObjectEntry10_1::BackReference { class_id, key })
                            if class_id == MAP_CLASS_ID && map_indices.contains_key(&key) =>
                        {
                            let (original, len) = map_indices[&key];
                            // 模型持有独立节点；限制大对象的重复回引复制量。
                            let Some(remaining) = reference_copy_budget.checked_sub(len) else {
                                diagnostics.push(format!(
                                    "数据框回引复制超过输入长度限额：0x{entry_offset:x}"
                                ));
                                preserve_maps_tail(maps, entry_offset, &mut map_unknown);
                                complete = false;
                                break;
                            };
                            reference_copy_budget = remaining;
                            frames.push(frames[original].clone());
                            diagnostics.push(format!(
                                "数据框 {}/{}：回引键 {key}",
                                index + 1,
                                root.count
                            ));
                        }
                        _ => {
                            diagnostics.push(format!(
                                "未支持的数据框对象或无法确认的回引：0x{entry_offset:x}"
                            ));
                            preserve_maps_tail(maps, entry_offset, &mut map_unknown);
                            complete = false;
                            break;
                        }
                    }
                }
                if complete {
                    let offset = objects.position();
                    if let Ok(raw) = objects.read_u32_le() {
                        confirmed_tail = Some((offset, raw));
                        diagnostics.push(format!(
                            "循环尾部值原始位模式 0x{raw:08x}（流内偏移 0x{offset:x}）"
                        ));
                    } else {
                        diagnostics
                            .push(format!("数据框对象循环完成，但根尾值在 0x{offset:x} 截断"));
                    }
                    preserve_maps_tail(maps, objects.position(), &mut map_unknown);
                }
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
        confirmed_tail,
        layer_list: report_layer_list,
        version_candidates: Vec::new(),
        diagnostics,
    })
}

fn preserve_maps_tail(maps: &[u8], offset: usize, unknown: &mut Vec<UnknownField>) {
    if let Some(bytes) = maps.get(offset..).filter(|bytes| !bytes.is_empty()) {
        unknown.push(UnknownField {
            stream: "/Maps".to_string(),
            offset: offset as u64,
            bytes: bytes.to_vec(),
        });
    }
}

/// 从已登记的 Map 对象头之后解析一个数据框。仅完整对象返回下一根字段的偏移。
fn parse_frame(
    maps: &[u8],
    body_offset: usize,
    index: i32,
    count: i32,
    diagnostics: &mut Vec<String>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<(DataFrame, Option<LayerList10_1>, Option<usize>)> {
    let mut body = Reader::new(maps).reposition_to(body_offset);
    let prefix = read_map_prefix_object_stream_10_1(&mut body, max_bytes, table)?;

    diagnostics.push(format!(
        "数据框 {}/{count}：名称 {:?}，第二名称 {:?}，架构版本 {}（>51 已由读取器拒绝）",
        index + 1,
        prefix.name,
        prefix.secondary_name,
        prefix.version
    ));
    match &prefix.spatial_reference {
        Some(sr) => diagnostics.push(match sr {
            crate::maps::map_object::MapSpatialReference10_1::Projected(pcs) => format!(
                "空间参考：ProjectedCoordinateSystem（保存版本 {}，WKT {} 字节）；\
                 域字符串、垂向标志、精度标志与 double 已由读取器解析但语义未确认，不写入模型",
                pcs.save_version,
                pcs.wkt.len()
            ),
            crate::maps::map_object::MapSpatialReference10_1::Geographic(gcs) => format!(
                "空间参考：GeographicCoordinateSystem（保存版本 {}，WKT {} 字节；F 轮三分派）；\
                 域字符串、范围与精度块已由读取器解析但语义未确认，不写入模型",
                gcs.save_version,
                gcs.wkt.len()
            ),
            crate::maps::map_object::MapSpatialReference10_1::UnknownCoordinateSystem(unk) => {
                format!(
                    "空间参考：UnknownCoordinateSystem（保存版本 {}，字符串 4 条，无 WKT；\
                     F 轮三分派）；字符串与精度块已由读取器解析但语义未确认，不写入模型",
                    unk.save_version
                )
            }
        }),
        None if prefix.spatial_reference_back_reference.is_some() => {
            if let Some(key) = prefix.spatial_reference_key {
                diagnostics.push(format!("空间参考：会话内回引键 {key}"));
            }
        }
        None => diagnostics
            .push("空间参考：嵌套对象头为 16 零字节空引用标记，该数据框未设置坐标系".to_string()),
    }
    let map_crs = prefix
        .spatial_reference
        .as_ref()
        .and_then(map_spatial_reference_to_model);

    // 旧版 Map 使用另一条图层列表读取路径，不能用新版列表推算后续边界。
    if prefix.version < 0x17 {
        let stop_offset = body.position();
        let remaining = &maps[stop_offset..];
        diagnostics.push(format!(
            "Map 架构版本 {} 小于 23（0x17）：旧版图层列表尚未接入，后缀从 \
             /Maps:0x{stop_offset:x} 起的 {} 字节保留为未知；图层和显示范围未解析",
            prefix.version,
            remaining.len()
        ));
        if let remaining_count @ 1.. = count - index - 1 {
            diagnostics.push(format!(
                "其余 {remaining_count} 个数据框对象未解析：当前 Map 后缀边界未知"
            ));
        }
        let unknown_fields = if remaining.is_empty() {
            Vec::new()
        } else {
            vec![UnknownField {
                stream: "/Maps".to_string(),
                offset: stop_offset as u64,
                bytes: remaining.to_vec(),
            }]
        };
        return Ok((
            DataFrame {
                name: prefix.name,
                secondary_name: Some(prefix.secondary_name),
                children: Vec::new(),
                crs: map_crs,
                extent: None,
                unknown_fields,
            },
            None,
            None,
        ));
    }
    let map_crs_key = prefix.spatial_reference_key;

    // 后缀读取器从对象体起点重读前缀；所有 Map 共用根流会话键表。
    let mut suffix_reader = Reader::new(maps).reposition_to(body_offset);
    let suffix = read_map_suffix_10_1(&mut suffix_reader, max_bytes, table)?;

    // —— 图层列表诊断、体前缀模型填充与未知体保留 ——
    let list = &suffix.layer_list;
    diagnostics.push(format!(
        "图层列表候选：声明计数 {}，已消费 {} 个条目（元素为 Map 的图层列表与绘制顺序\
         候选，逐条目见下；样本闭合为辅助观察）",
        list.count,
        list.entries.len()
    ));
    let mut frame_unknown: Vec<UnknownField> = Vec::new();
    let mut children: Vec<LayerNode> = Vec::new();
    // 键 → 已构造节点：回引条目复用同对象节点（原生同一图层对象的再现）。
    let mut parsed_by_key: Vec<(u32, LayerNode)> = Vec::new();
    for entry in &list.entries {
        diagnostics.push(format!(
            "图层条目 {}/{}（offset 0x{:x}）：{}",
            entry.index + 1,
            list.count,
            entry.offset,
            entry.describe()
        ));
        // 体前缀按类分派结果构造图层节点；仅已确认字段进入模型。
        let mut node: Option<LayerNode> = match &entry.parsed {
            Some(LayerBodyPrefix10_1::FeatureLayer(prefix)) => {
                let (visible, note) = variant_bool_to_model(prefix.visibility_raw, "可见性");
                if let Some(note) = note {
                    diagnostics.push(format!("条目 {}：{note}", entry.index + 1));
                }
                feature_layer_successor_note(prefix, entry.index, diagnostics);
                let source = feature_layer_source(prefix, entry.index, diagnostics);
                let crs = feature_layer_crs(
                    prefix,
                    map_crs_key,
                    map_crs.as_ref(),
                    entry.index,
                    diagnostics,
                );
                Some(LayerNode::Layer(Layer {
                    name: prefix.name.clone(),
                    visible,
                    source,
                    crs,
                    extent: None,
                    unknown_fields: Vec::new(),
                }))
            }
            Some(LayerBodyPrefix10_1::FdoGraphicsLayer(prefix)) => {
                // FDO 注记图层：以关联要素图层（ILayer 槽）的名称/可见性/数
                // 源构造节点；委派注记图层只进诊断（第六轮）。
                match &prefix.associated_layer {
                    AssociatedLayer10_1::FeatureLayer(associated) => {
                        let (visible, note) =
                            variant_bool_to_model(associated.visibility_raw, "可见性");
                        if let Some(note) = note {
                            diagnostics.push(format!("条目 {}：{note}", entry.index + 1));
                        }
                        diagnostics.push(format!(
                            "条目 {}：FDOGraphicsLayer（版本 {}，int32={}，short=0x{:04x}）关\
                             联要素图层前缀已解析；{}",
                            entry.index + 1,
                            prefix.version,
                            prefix.int32_field,
                            prefix.short_field,
                            if prefix.completion().is_some() {
                                "委派 FeatureLayer 和 FDO 后续字段已完整消费".to_string()
                            } else if let Some(tail) = &prefix.fdo_tail {
                                format!("FDO 后续字段在未知内容处停止：{:?}", tail.stop)
                            } else {
                                prefix.stop.describe()
                            }
                        ));
                        feature_layer_successor_note(associated, entry.index, diagnostics);
                        let source = feature_layer_source(associated, entry.index, diagnostics);
                        let crs = feature_layer_crs(
                            associated,
                            map_crs_key,
                            map_crs.as_ref(),
                            entry.index,
                            diagnostics,
                        );
                        // IElementCollection2 槽（A 轮起 FL 体 TailCompleted 后
                        // 可达）：透传其消费摘要。
                        if let Some(slot) = &prefix.element_collection {
                            let detail = match slot {
                                ElementCollectionSlot10_1::Null => "空标记".to_string(),
                                ElementCollectionSlot10_1::BackReference { key, .. } => {
                                    format!("回引（键 {key}）")
                                }
                                ElementCollectionSlot10_1::ElementCollection(body) => format!(
                                    "ElementCollection 新对象：版本 {}，元素声明 {}、已消费 \
                                     {}（{}）",
                                    body.version,
                                    body.count,
                                    match &body.body {
                                        crate::objects::element_collection::ElementCollectionBody10_1::Interleaved { slots } => slots.len(),
                                        crate::objects::element_collection::ElementCollectionBody10_1::Sequential { elements, .. } => elements.len(),
                                    },
                                    match &body.stop {
                                        Some(stop) => format!("体内停止：{stop:?}"),
                                        None => "体完整消费".to_string(),
                                    }
                                ),
                                ElementCollectionSlot10_1::Unsupported { class_id, key } => format!(
                                    "新对象非 ElementCollection（类 {}，键 {key}）",
                                    crate::storage::class_id_hex(class_id)
                                ),
                            };
                            diagnostics.push(format!(
                                "条目 {}：IElementCollection2 槽：{detail}",
                                entry.index + 1
                            ));
                        }
                        if let Some(delegated) = &prefix.delegated_layer {
                            diagnostics.push(format!(
                                "条目 {}：委派注记图层（无对象头，按 FeatureLayer 字段序）前\
                                 缀：名称 {:?}，第二名称 {:?}；后续流剩余 {} 字节，\
                                 停止状态 {:?}，不构造节点",
                                entry.index + 1,
                                delegated.name,
                                delegated.secondary_name,
                                delegated.remaining_bytes,
                                delegated.stop
                            ));
                        }
                        Some(LayerNode::Layer(Layer {
                            name: associated.name.clone(),
                            visible,
                            source,
                            crs,
                            extent: None,
                            unknown_fields: Vec::new(),
                        }))
                    }
                    other => {
                        let slot = match other {
                            AssociatedLayer10_1::Null => "空标记".to_string(),
                            AssociatedLayer10_1::BackReference { key, .. } => {
                                format!("回引（键 {key}）")
                            }
                            AssociatedLayer10_1::FeatureLayer(_) => unreachable!("上方已匹配"),
                        };
                        diagnostics.push(format!(
                            "条目 {}：FDOGraphicsLayer（版本 {}）关联要素图层槽为 {slot}，无前\
                             缀可填充；委派体与尾段保留未知",
                            entry.index + 1,
                            prefix.version
                        ));
                        None
                    }
                }
            }
            Some(LayerBodyPrefix10_1::GroupLayer(prefix)) => {
                let (visible, note) = match prefix.visibility {
                    Some(raw) => variant_bool_to_model(raw, "可见性"),
                    None => (
                        true,
                        Some(format!(
                            "条目 {}：可见性字段未读取（版本 <2 原生默认可见，或停止于\
                             读取点之前），按可见处理",
                            entry.index + 1
                        )),
                    ),
                };
                if let Some(note) = note {
                    diagnostics.push(note);
                }
                // 子图层递归（第六轮接入）：经类分派解析出前缀的子图层构造
                // 节点（回引子图层暂不在此复用——跨层键到节点的映射未建立）。
                let mut group_children = Vec::new();
                for child in &prefix.children {
                    match child {
                        crate::layers::group_layer::GroupChildEntry10_1::ParsedNewObject {
                            parsed,
                            ..
                        } => {
                            if let Some(node) = body_prefix_node(
                                parsed,
                                entry.index,
                                map_crs_key,
                                map_crs.as_ref(),
                                diagnostics,
                            ) {
                                group_children.push(node);
                            }
                        }
                        crate::layers::group_layer::GroupChildEntry10_1::SkippedNewObject {
                            parsed: Some(parsed),
                            ..
                        } => {
                            if let Some(node) = body_prefix_node(
                                parsed,
                                entry.index,
                                map_crs_key,
                                map_crs.as_ref(),
                                diagnostics,
                            ) {
                                group_children.push(node);
                            }
                        }
                        _ => {}
                    }
                }
                diagnostics.push(format!(
                    "条目 {}：组图层子图层声明 {} 个，已消费 {} 条，其中 {} 条经类分派解析\
                     出前缀并填充 children；解析失败的子图层体与尾段保留未知（无组图层样\
                     本，分派行为无字节佐证）",
                    entry.index + 1,
                    prefix.child_count,
                    prefix.children.len(),
                    group_children.len()
                ));
                Some(LayerNode::Group(LayerGroup {
                    name: prefix.name.clone(),
                    visible,
                    children: group_children,
                    unknown_fields: Vec::new(),
                }))
            }
            Some(LayerBodyPrefix10_1::CompositeGraphicsLayer(prefix)) => {
                diagnostics.push(format!(
                    "条目 {}：CompositeGraphicsLayer（名称 {:?}）为图形层，中间模型\
                     没有对应节点类型，仅保留诊断；其子图层循环见报告",
                    entry.index + 1,
                    prefix.name
                ));
                None
            }
            Some(LayerBodyPrefix10_1::RasterLayer(raster)) => {
                raster_node(raster, entry.index, diagnostics)
            }
            None => None,
        };
        // 回引条目：键命中先前已解析条目时复用同对象节点（原生同一图层对象
        // 的再次引用，样本条目 2 即此形态；未命中时不构造节点，只保留诊断）。
        if let (None, LayerEntryObject::Entry(ObjectEntry10_1::BackReference { key, .. })) =
            (&node, &entry.object)
        {
            if let Some((_, existing)) = parsed_by_key.iter().find(|(seen, _)| seen == key) {
                diagnostics.push(format!(
                    "条目 {}：回引（键 {key}）命中先前已解析对象，复用其图层节点",
                    entry.index + 1
                ));
                node = Some(existing.clone());
            }
        }
        if let Some(node) = node {
            if let LayerEntryObject::Entry(ObjectEntry10_1::NewObject { key, .. }) = &entry.object {
                parsed_by_key.push((*key, node.clone()));
            }
            children.push(node);
        }
        if let Some(range) = &entry.body {
            // 体前缀之后被有界跳过的剩余（或整段未解析体）：字节按条目保留为
            // 数据框未知字段，不解释。
            if range.len > 0 {
                let start = range.offset;
                let end = start
                    .checked_add(range.len)
                    .expect("有界跳过已校验不超过流长度");
                frame_unknown.push(UnknownField {
                    stream: "/Maps".to_string(),
                    offset: start as u64,
                    bytes: maps[start..end].to_vec(),
                });
                diagnostics.push(format!(
                    "条目 {} 的体剩余 {} 字节未解析，保留为数据框未知字段（流内偏移 \
                     0x{start:x} 起）",
                    entry.index + 1,
                    range.len
                ));
            }
        }
    }
    match list.active_index {
        Some(index) => diagnostics.push(format!(
            "活动图层索引原始值 {index}（写侧初值 -1 = 无活动图层候选；0<=idx<count 时\
             指向活动层，语义候选；放报告不放模型）"
        )),
        None => diagnostics.push("活动图层索引未读取：图层列表中途停止（见停止原因）".to_string()),
    }

    // —— 基本图形层 / MapSurround / IEnvelope / 书签槽位诊断 ——
    if let Some(slot) = &suffix.base_graphics_layer {
        diagnostics.push(format!(
            "基本图形层对象（offset 0x{:x}）：{}",
            slot.offset,
            slot.describe()
        ));
    }
    if let Some(parsed) = &suffix.base_graphics_layer_parsed {
        diagnostics.push(format!(
            "基本图形层体前缀（CompositeGraphicsLayer）：名称 {:?}，元素组计数 {}（已消费 \
             {} 组），子图层计数 {}（已消费 {} 条）；{}",
            parsed.name,
            parsed.group_count,
            parsed.element_groups.len(),
            parsed.sublayer_count,
            parsed.sublayers.len(),
            parsed.stop.describe()
        ));
    }
    if let Some(surround_count) = suffix.map_surround_count {
        diagnostics.push(format!(
            "MapSurround 计数 {surround_count}（地图装饰物集合）；已消费 {} 个条目",
            suffix.map_surrounds.len()
        ));
        push_slot_diagnostics(&suffix.map_surrounds, "MapSurround 条目", diagnostics);
        for (index, parsed) in suffix.map_surround_parsed.iter().enumerate() {
            if let Some(MapSurroundElement10_1::Legend(legend)) = parsed {
                diagnostics.push(format!(
                    "MapSurround Legend 条目 {}：主体版本 {}",
                    index + 1,
                    legend.version
                ));
            }
        }
    }
    if let Some(envelope) = &suffix.envelope_object {
        diagnostics.push(format!(
            "IEnvelope 槽位对象（offset 0x{:x}）：{}（空标记按样本消费 16 字节，\
             依据见 map_object 模块文档）",
            envelope.offset,
            envelope.describe()
        ));
    }
    if let Some(parsed) = &suffix.envelope_object_parsed {
        diagnostics.push(format!(
            "IEnvelope 槽位 Envelope 对象体：版本 {}，坐标 [{}, {}, {}, {}]，int32 标志 \
             0x{:08x}；嵌套空间参考：{}；该对象语义（候选：地图全图范围）未闭合，\
             不写入模型",
            parsed.version,
            parsed.coords[0],
            parsed.coords[1],
            parsed.coords[2],
            parsed.coords[3],
            parsed.attribute_flags.unwrap_or(0),
            match &parsed.spatial_reference {
                Some(crate::objects::envelope::SpatialReferenceEntry10_1::Null) =>
                    "空引用标记".to_string(),
                Some(crate::objects::envelope::SpatialReferenceEntry10_1::BackReference {
                    key,
                    ..
                }) => format!("回引（键 {key}）"),
                Some(crate::objects::envelope::SpatialReferenceEntry10_1::Projected {
                    key,
                    ..
                }) => format!("内嵌 PCS（键 {key}）"),
                Some(crate::objects::envelope::SpatialReferenceEntry10_1::Geographic {
                    key,
                    ..
                }) => format!("内嵌 GCS（键 {key}；F 轮三分派）"),
                Some(
                    crate::objects::envelope::SpatialReferenceEntry10_1::UnknownCoordinateSystem {
                        key,
                        ..
                    },
                ) => format!("内嵌未知坐标系（键 {key}；F 轮三分派）"),
                None => "该版本分支无此段".to_string(),
            }
        ));
    }
    if let Some(bookmark_count) = suffix.bookmark_count {
        diagnostics.push(format!(
            "书签计数 {bookmark_count}（空间书签集合）；已消费 {} 个条目",
            suffix.bookmarks.len()
        ));
        push_slot_diagnostics(&suffix.bookmarks, "书签条目", diagnostics);
    }
    if let Some(post) = suffix.post_bookmark {
        diagnostics.push(format!(
            "书签后属性（offset 0x{:x}）：+0x19c={}，+0x200={}，+0x264={}，\
             独立表计数 {}；未定性成员不写入模型",
            post.offset,
            post.short_19c,
            post.double_200,
            post.short_264,
            post.standalone_table_count,
        ));
    }

    for (index, (slot, body)) in suffix.standalone_tables.iter().enumerate() {
        diagnostics.push(format!(
            "独立表 {}（offset 0x{:x}）：{}",
            index + 1,
            slot.offset,
            slot.describe(),
        ));
        if let Some(body) = body {
            diagnostics.push(format!("独立表体版本 {}", body.version));
        }
    }

    if let Some(illumination) = suffix.illumination {
        diagnostics.push(format!(
            "IlluminationProps 槽位（offset 0x{:x}）：{}",
            illumination.object.offset,
            illumination.object.describe(),
        ));
    }
    if let Some(post) = suffix.post_illumination {
        diagnostics.push(format!(
            "照明后定长字段（offset 0x{:x}）：+0x1a0=0x{:04x}，临时值=0x{:04x}，\
             +0x1b0=0x{:08x}，+0x1ac=0x{:04x}，+0x268=0x{:04x}",
            post.offset,
            post.short_1a0,
            post.discarded_short,
            post.int_1b0,
            post.short_1ac,
            post.short_268,
        ));
    }
    if let Some(geometry) = suffix.post_illumination_null_geometry {
        diagnostics.push(format!(
            "照明后 IGeometry 槽位（offset 0x{:x}）：{}",
            geometry.offset,
            geometry.describe(),
        ));
    }
    if let Some((offset, border)) = &suffix.post_illumination_border {
        diagnostics.push(format!(
            "照明后 IBorder 槽位（offset 0x{:x}）：{:?}",
            offset, border,
        ));
    }

    if suffix.post_border_tail.short_308.is_some() {
        let tail = &suffix.post_border_tail;
        diagnostics.push(format!(
            "IBorder 后版本块：short={:?}、{:?}、{:?}、{:?}，double 位={:?}；\
             两个 int32={:?}、{:?}，临时 short={:?}、double 位={:?}；\
             坐标变换计数={:?}，已读前导 {} 条、完整对象槽 {} 条",
            tail.short_308,
            tail.short_310,
            tail.short_318,
            tail.short_31a,
            tail.double_bits,
            tail.int_1a4,
            tail.int_1a8,
            tail.temporary_short,
            tail.temporary_double_bits,
            tail.transformation_count,
            tail.transformations.len(),
            tail.transformations
                .iter()
                .filter(|item| item.object.is_some())
                .count()
        ));
    }

    if let Some(value) = suffix.post_border_tail.post_transform_double_bits {
        let tail = &suffix.post_border_tail;
        diagnostics.push(format!(
            "变换循环后：double 位={value:?}，short={:?}，背景槽={:?}，后续空对象槽={:?}",
            tail.post_transform_short,
            tail.post_transform_background,
            tail.post_transform_null_object
        ));
    }

    if let Some((slot, body)) = &suffix.post_border_tail.post_transform_overposter {
        diagnostics.push(format!(
            "BasicOverposterProperties（offset 0x{:x}）：{}，版本 {}，颜色槽 {:?}",
            slot.offset,
            slot.describe(),
            body.version,
            body.color
        ));
    }

    let tail = &suffix.post_border_tail;
    if tail.short_32a.is_some() {
        diagnostics.push(format!(
            "后续版本字段：short={:?}，double 位={:?}，short={:?}；IEnvelope 槽={:?}，double 位={:?}、{:?}，int32={:?}",
            tail.short_32a, tail.double_320_bits, tail.short_328,
            tail.scale_envelopes, tail.scale_double_2b8_bits, tail.scale_double_2d8_bits, tail.scale_ints
        ));
    }
    if tail.masking_version.is_some() {
        diagnostics.push(format!(
            "Map LayerMasking：版本={:?}，计数={:?}，已读条目={:?}，尾 short={:?}",
            tail.masking_version,
            tail.masking_count,
            tail.masking_entries.as_ref().map(Vec::len),
            tail.masking_empty_short
        ));
    }

    if let Some(fields) = &tail.post_masking {
        diagnostics.push(format!(
            "Map 逐图层遮罩循环不消费字节；之后的 IsFramed 原始位={:?}，+0x398 字符串偏移={:?}，+0x3a0 short={:?}，+0x15d 字节={:?}",
            fields.is_framed_bits,
            fields.string_398.as_ref().map(|(offset, _)| offset),
            fields.short_3a0,
            fields.byte_15d,
        ));
    }

    if let Some(collections) = &tail.persist_collections {
        diagnostics.push(format!(
            "Map 版本 33 集合：声明 {}，已读 {}，后续 LONG={:?}，停止={:?}",
            collections.count,
            collections.collection.len(),
            collections.long_value,
            collections.stop
        ));
    }

    // —— 显示范围（WKSEnvelope，到达时才写入模型）——
    let extent = suffix.extent.map(|wks| Extent {
        xmin: wks.xmin,
        ymin: wks.ymin,
        xmax: wks.xmax,
        ymax: wks.ymax,
    });
    match (&suffix.extent, &suffix.raw_int32_pair) {
        (Some(wks), Some((first, second))) => diagnostics.push(format!(
            "显示范围（WKSEnvelope，offset 0x{:x}，字段序 [XMin,YMin,XMax,YMax] 写方向\
             确定）：xmin={} ymin={} xmax={} ymax={}；其后两个 int32 原样（{first}, \
             {second}），用途未定性，不写入模型",
            suffix.extent_offset.unwrap_or_default(),
            wks.xmin,
            wks.ymin,
            wks.xmax,
            wks.ymax
        )),
        _ => diagnostics.push(format!(
            "显示范围未到达，extent 保留为 None：{}",
            suffix.stop.describe()
        )),
    }

    // —— 停止点与尾部未知区 ——
    let stop_offset = suffix_reader.position();
    let complete = suffix.stop == crate::maps::map_object::MapSuffixStop::Complete;
    if complete {
        diagnostics.push(format!("Map 对象体完整消费至 /Maps:0x{stop_offset:x}"));
    } else {
        diagnostics.push(format!(
            "后缀读取停止：{}；其后 {} 字节保留为数据框未知尾部（流内偏移 0x{stop_offset:x} 起），不解释",
            suffix.stop.describe(), suffix.remaining_bytes
        ));
        preserve_maps_tail(maps, stop_offset, &mut frame_unknown);
        if let remaining_count @ 1.. = count - index - 1 {
            diagnostics.push(format!(
                "其余 {remaining_count} 个数据框对象未解析：对象循环边界未闭合，对象体无外部长度字段，不跳过未知对象"
            ));
        }
    }
    if children.is_empty() {
        diagnostics.push(
            "图层树保持空（children=[]）：无经类分派解析出体前缀的图层条目，未解析\
             条目不构造 LayerNode"
                .to_string(),
        );
    } else {
        diagnostics.push(format!(
            "图层树已按条目顺序填充 {} 个节点：节点仅含各体前缀读取器的已确认字段\
             （FeatureLayer/FDOGraphicsLayer 名称/可见性/数据源链（要素类名 + 工作空间路\
             径）、GroupLayer 名称/可见性/子图层递归），其余字段保留为空或未知",
            children.len()
        ));
    }

    let frame = DataFrame {
        name: prefix.name,
        secondary_name: Some(prefix.secondary_name),
        children,
        crs: map_crs,
        extent,
        unknown_fields: frame_unknown,
    };
    Ok((
        frame,
        Some(suffix.layer_list),
        complete.then_some(stop_offset),
    ))
}

/// VARIANT_BOOL 位模式 → 模型布尔值；非规范位模式按非零=真处理并返回说明。
///
/// 规范取值仅 0xFFFF（TRUE）与 0x0000（FALSE）；其余位模式的原生比较语义未
/// 取证，按 COM 惯例的非零=真处理，同时把位模式原样写入诊断，不静默吞掉。
fn variant_bool_to_model(raw: u16, what: &str) -> (bool, Option<String>) {
    match raw {
        0xFFFF => (true, None),
        0x0000 => (false, None),
        other => (
            true,
            Some(format!(
                "{what}为非规范 VARIANT_BOOL 位模式 0x{other:04x}，按非零=真处理\
                 （COM 惯例，原生比较语义未取证）"
            )),
        ),
    }
}

fn map_spatial_reference_to_model(
    sr: &crate::maps::map_object::MapSpatialReference10_1,
) -> Option<CoordinateSystem> {
    match sr {
        crate::maps::map_object::MapSpatialReference10_1::Projected(pcs) => {
            Some(CoordinateSystem {
                definition: pcs.wkt.clone(),
                projected: Some(ProjectedCoordinateSystem {
                    save_version: pcs.save_version,
                    domains: pcs.domains.as_ref().map(|domains| domains.strings.clone()),
                    vertical_flag: pcs.domains.as_ref().map(|domains| domains.vertical_flag),
                    precision_flags: pcs.precision.as_ref().map(|p| p.flags),
                    precision_doubles: pcs.precision.as_ref().map(|p| p.doubles),
                    trailing_doubles: pcs.trailing_doubles,
                }),
                unknown_fields: Vec::new(),
            })
        }
        crate::maps::map_object::MapSpatialReference10_1::Geographic(gcs) => {
            Some(CoordinateSystem {
                definition: gcs.wkt.clone(),
                projected: None,
                unknown_fields: Vec::new(),
            })
        }
        crate::maps::map_object::MapSpatialReference10_1::UnknownCoordinateSystem(_) => None,
    }
}

/// 只把 FeatureLayer 已确认的 +0x1f4 Envelope 槽坐标系接入图层模型。
fn feature_layer_crs(
    layer: &crate::layers::feature_layer::FeatureLayerPrefix10_1,
    map_crs_key: Option<u32>,
    map_crs: Option<&CoordinateSystem>,
    entry_index: usize,
    diagnostics: &mut Vec<String>,
) -> Option<CoordinateSystem> {
    use crate::layers::annotate_props::EnvelopeSlot10_1;

    let envelope = layer.successor.as_ref().and_then(|s| s.envelope_2.as_ref());
    let reference = match envelope {
        Some(EnvelopeSlot10_1::Envelope(body)) => body.spatial_reference.as_ref(),
        Some(EnvelopeSlot10_1::BackReference { key, .. }) => {
            diagnostics.push(format!(
                "条目 {}：+0x1f4 Envelope 本身为回引（键 {key}），没有该 Envelope 的已解析体；不推断图层 CRS",
                entry_index + 1
            ));
            return None;
        }
        Some(EnvelopeSlot10_1::Null) | None => return None,
    };
    envelope_reference_to_model(reference, map_crs_key, map_crs, entry_index, diagnostics)
}

fn envelope_reference_to_model(
    reference: Option<&crate::objects::envelope::SpatialReferenceEntry10_1>,
    map_crs_key: Option<u32>,
    map_crs: Option<&CoordinateSystem>,
    entry_index: usize,
    diagnostics: &mut Vec<String>,
) -> Option<CoordinateSystem> {
    use crate::objects::envelope::SpatialReferenceEntry10_1;

    match reference {
        Some(SpatialReferenceEntry10_1::Projected { body, .. }) => map_spatial_reference_to_model(
            &crate::maps::map_object::MapSpatialReference10_1::Projected(body.clone()),
        ),
        Some(SpatialReferenceEntry10_1::Geographic { body, .. }) => map_spatial_reference_to_model(
            &crate::maps::map_object::MapSpatialReference10_1::Geographic(body.clone()),
        ),
        Some(SpatialReferenceEntry10_1::BackReference { class_id, key }) => {
            let expected_class_id = map_crs.map(|crs| {
                if crs.projected.is_some() {
                    crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID
                } else {
                    crate::objects::spatial_reference_slots::GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID
                }
            });
            if map_crs_key == Some(*key) && expected_class_id == Some(*class_id) {
                return map_crs.cloned();
            }
            diagnostics.push(format!(
                "条目 {}：+0x1f4 Envelope 空间参考回引键 {key}/类标识 {} 与 Map CRS 键 {:?}/类标识 {:?} 不匹配；图层 CRS 留空",
                entry_index + 1,
                crate::storage::class_id_hex(class_id),
                map_crs_key,
                expected_class_id.map(|id| crate::storage::class_id_hex(&id))
            ));
            None
        }
        Some(SpatialReferenceEntry10_1::UnknownCoordinateSystem { key, .. }) => {
            diagnostics.push(format!(
                "条目 {}：+0x1f4 Envelope 空间参考为无已确认 WKT 的 UnknownCoordinateSystem（键 {key}）；图层 CRS 留空",
                entry_index + 1
            ));
            None
        }
        Some(SpatialReferenceEntry10_1::Null) | None => None,
    }
}

/// 追加 FeatureLayer 后继字段段（+0x144 GUID 之后，第八轮闭合至 FieldInfo
/// 计数循环）的消费诊断；仅记录停止点与 FieldInfo 条目数，不解释语义。
fn feature_layer_successor_note(
    prefix: &crate::layers::feature_layer::FeatureLayerPrefix10_1,
    entry_index: usize,
    diagnostics: &mut Vec<String>,
) {
    let Some(successor) = &prefix.successor else {
        return;
    };
    diagnostics.push(format!(
        "条目 {}：渲染器槽后的后继字段段已消费至 {:?} 停止（FieldInfo 循环声明 {} 条、\
         已消费 {} 条；IHyperlink 循环声明 {} 条、已消费 {} 条；+0x154 字符串 {:?} 不解释\
         语义），其后字节保留未知",
        entry_index + 1,
        successor.stop,
        successor.field_info_count,
        successor.field_infos.len(),
        successor.hyperlink_count,
        successor.hyperlinks.len(),
        successor.string_154,
    ));
    // ㊴ DisplayExpressionProperties 完整体（A 轮接线起样本可达）：透传体
    //   内容摘要，字段风格循既有诊断。DEP::Load 读流序列仅四项（版本/
    //   表达式/int16/引擎槽），体在引擎槽处完结、无尾段
    //   （docs/arcmap-10.1-dep-tail.md 第一节）。
    if let Some(crate::layers::feature_layer_tail::ExpressionPropsSlot10_1::Display(prefix)) =
        &successor.expression_props
    {
        let engine_note = match &prefix.engine {
            crate::layers::annotate_props::ExpressionEngineSlot10_1::VbscriptEngine(engine) => {
                format!("AnnotationVBScriptEngine（键 {}）", engine.key)
            }
            crate::layers::annotate_props::ExpressionEngineSlot10_1::OtherEngine(engine) => {
                let name = match engine.kind {
                    crate::layers::annotate_props::AnnotationExpressionEngineKind10_1::Vbscript => {
                        "AnnotationVBScriptEngine"
                    }
                    crate::layers::annotate_props::AnnotationExpressionEngineKind10_1::Jscript => {
                        "AnnotationJScriptEngine"
                    }
                    crate::layers::annotate_props::AnnotationExpressionEngineKind10_1::Python => {
                        "AnnotationPythonEngine"
                    }
                };
                format!("{name}（键 {}）", engine.body.key)
            }
            crate::layers::annotate_props::ExpressionEngineSlot10_1::BackReference {
                key, ..
            } => {
                format!("回引（键 {key}）")
            }
            crate::layers::annotate_props::ExpressionEngineSlot10_1::Null => "空标记".to_string(),
            crate::layers::annotate_props::ExpressionEngineSlot10_1::Unsupported {
                key, ..
            } => {
                format!("未知类（键 {key}，槽前停止）")
            }
        };
        diagnostics.push(format!(
            "条目 {}：㊴ DisplayExpressionProperties（esri.clsid 声明）体完整消费\
             （版本 {}、表达式 {:?}、int16 {}、引擎槽 {}；A 轮裁决无尾段）",
            entry_index + 1,
            prefix.version,
            prefix.string,
            prefix.short,
            engine_note,
        ));
    }
    // 标注环（ALPC/LELP/TextSymbol 链）的已解析内容与停止点（接线轮起
    // TextSymbol 体解析推进，停止点深入其体内；字段风格循既有诊断）。
    annotate_chain_note(successor, entry_index, diagnostics);
}

/// 标注环诊断：ALPC 槽为已解析集合时逐条列出 LELP 表达式/类名与 TextSymbol
/// 体形态；均为已确认字段，业务语义不解释。
fn annotate_chain_note(
    successor: &crate::layers::feature_layer_tail::FeatureLayerSuccessor10_1,
    entry_index: usize,
    diagnostics: &mut Vec<String>,
) {
    let Some(crate::layers::feature_layer_tail::AnnotatePropsSlot10_1::Collection(alpc)) =
        &successor.annotate_props
    else {
        return;
    };
    for entry in &alpc.entries {
        let crate::layers::annotate_props::AlpcEntrySlot10_1::LabelEngineProps(lelp) = &entry.slot
        else {
            continue;
        };
        let text_symbol_note = match &lelp.text_symbol {
            crate::layers::annotate_props::LelpTextSymbolSlot10_1::TextSymbol(body) => format!(
                "ITextSymbol 槽 TextSymbol 体（版本 {}）已解析{}",
                body.version,
                match &body.stop {
                    Some(reason) => format!("，体内停止：{reason:?}"),
                    None => "（体完整消费）".to_string(),
                }
            ),
            crate::layers::annotate_props::LelpTextSymbolSlot10_1::Null => {
                "ITextSymbol 槽为空标记".to_string()
            }
            crate::layers::annotate_props::LelpTextSymbolSlot10_1::BackReference {
                key, ..
            } => {
                format!("ITextSymbol 槽为回引（键 {key}）")
            }
        };
        // LELP 体内的停止（接线轮起 typed 槽链可达 ⑰ BOLP 体等，停止原因
        // 透传；风格循 TextSymbol 体内停止的 {:?} 透传）。
        let lelp_stop_note = match &lelp.stop {
            Some(reason) => format!("；LELP 体内停止：{reason:?}"),
            None => String::new(),
        };
        diagnostics.push(format!(
            "条目 {}：标注属性集合（键 {}）条目 LELP（键 {}，版本 {}）：表达式 {:?}、\
             标注类名 {:?}；{}{}；条目键 {}",
            entry_index + 1,
            alpc.key,
            lelp.key,
            lelp.version,
            lelp.expression,
            lelp.name,
            text_symbol_note,
            lelp_stop_note,
            match entry.entry_key {
                Some(k) => format!("{k}"),
                None => "未读（LELP 体内停止）".to_string(),
            }
        ));
    }
}

/// 由 FeatureLayer 前缀构造数据源模型项与诊断（FCN 名称 → connection，
/// WorkspaceName 路径 → path；均为已确认字段，语义见 feature_layer 模块）。
fn feature_layer_source(
    prefix: &crate::layers::feature_layer::FeatureLayerPrefix10_1,
    entry_index: usize,
    diagnostics: &mut Vec<String>,
) -> Option<DataSource> {
    match &prefix.data_source {
        DataSource10_1::FeatureClassName(fcn) => {
            let (path, workspace_note) = match &fcn.nested {
                FeatureClassNameNested10_1::WorkspaceName(WorkspaceNameEntry10_1::NewObject(
                    ws,
                )) => {
                    let note = format!(
                        "工作空间（键 {}）：路径 {:?}，类型描述 {:?}{}{}",
                        ws.key,
                        ws.path,
                        ws.type_description,
                        match ws.factory_class_id {
                            Some(class_id) => format!(
                                "，工厂 CLSID {}（原样保留）",
                                crate::layers::layer_list::class_id_hex(&class_id)
                            ),
                            None => String::new(),
                        },
                        if ws.stop.is_some() {
                            "；IPropertySet 槽新对象处停止，标志及之后字段未读取"
                        } else {
                            ""
                        }
                    );
                    (Some(ws.path.clone()), Some(note))
                }
                FeatureClassNameNested10_1::WorkspaceName(_) => {
                    (None, Some("工作空间槽为空标记/回引，无路径".to_string()))
                }
                FeatureClassNameNested10_1::DatasetName(DatasetNameEntry10_1::NewObject(body)) => {
                    use crate::objects::dataset_name::WorkspaceNameSlot10_1;
                    use crate::objects::dataset_name_more::DatasetNameMoreBody10_1;

                    if let DatasetNameMoreBody10_1::FeatureDataset(dataset) = body.as_ref() {
                        let detail = match &dataset.workspace {
                            WorkspaceNameSlot10_1::WorkspaceName(ws) => match &ws.stop {
                                Some(stop) => {
                                    format!("工作空间键 {}，路径已读，后续停止：{stop:?}", ws.key)
                                }
                                None => format!("工作空间键 {}，路径已读", ws.key),
                            },
                            WorkspaceNameSlot10_1::Null => "工作空间槽为空".to_string(),
                            WorkspaceNameSlot10_1::BackReference { key, .. } => {
                                format!("工作空间槽回引键 {key}，当前条目无路径体")
                            }
                            WorkspaceNameSlot10_1::Unsupported { class_id, key } => format!(
                                "工作空间槽类 {}（键 {key}）未解析",
                                crate::layers::layer_list::class_id_hex(class_id),
                            ),
                        };
                        (
                            dataset.workspace_path().map(str::to_owned),
                            Some(format!(
                                "FeatureDatasetName（键 {}）：{detail}",
                                dataset.key
                            )),
                        )
                    } else {
                        (
                            None,
                            Some("IDatasetName 当前类未映射工作空间路径".to_string()),
                        )
                    }
                }
                FeatureClassNameNested10_1::DatasetName(_) => (
                    None,
                    Some("选择子 !=0 的 IDatasetName 分支：槽位保留未知".to_string()),
                ),
            };
            if let Some(note) = workspace_note {
                diagnostics.push(format!("条目 {}：{note}", entry_index + 1));
            }
            diagnostics.push(format!(
                "条目 {}：数据源 FeatureClassName（键 {}，版本 {}）：类型描述 {:?}，数据集名 \
                 {:?}，int32 对 ({}, {})；connection=要素类名，path=工作空间路径（均不解释\
                 语义）",
                entry_index + 1,
                fcn.key,
                fcn.version,
                fcn.type_description,
                fcn.dataset_name,
                fcn.int32_a,
                fcn.int32_b
            ));
            Some(DataSource {
                provider: None,
                path,
                connection: Some(fcn.name.clone()),
                unknown_fields: Vec::new(),
            })
        }
        DataSource10_1::Null => None,
        DataSource10_1::BackReference { key, .. } => {
            diagnostics.push(format!(
                "条目 {}：数据源为回引条目（键 {key}），无体可解析，source 保留为 None",
                entry_index + 1
            ));
            None
        }
    }
}

/// 由体前缀构造图层节点（组图层子图层递归共用；仅已确认字段）。
///
/// 返回 `None` 的情况与图层列表条目级填充一致：CompositeGraphicsLayer 无对应
/// 节点类型、FDO 关联槽无前缀等，只保留诊断。
fn body_prefix_node(
    prefix: &LayerBodyPrefix10_1,
    entry_index: usize,
    map_crs_key: Option<u32>,
    map_crs: Option<&CoordinateSystem>,
    diagnostics: &mut Vec<String>,
) -> Option<LayerNode> {
    match prefix {
        LayerBodyPrefix10_1::FeatureLayer(fl) => {
            let (visible, note) = variant_bool_to_model(fl.visibility_raw, "可见性");
            if let Some(note) = note {
                diagnostics.push(note);
            }
            let source = feature_layer_source(fl, entry_index, diagnostics);
            let crs = feature_layer_crs(fl, map_crs_key, map_crs, entry_index, diagnostics);
            Some(LayerNode::Layer(Layer {
                name: fl.name.clone(),
                visible,
                source,
                crs,
                extent: None,
                unknown_fields: Vec::new(),
            }))
        }
        LayerBodyPrefix10_1::FdoGraphicsLayer(fdo) => match &fdo.associated_layer {
            AssociatedLayer10_1::FeatureLayer(associated) => {
                let (visible, note) = variant_bool_to_model(associated.visibility_raw, "可见性");
                if let Some(note) = note {
                    diagnostics.push(note);
                }
                let source = feature_layer_source(associated, entry_index, diagnostics);
                let crs =
                    feature_layer_crs(associated, map_crs_key, map_crs, entry_index, diagnostics);
                Some(LayerNode::Layer(Layer {
                    name: associated.name.clone(),
                    visible,
                    source,
                    crs,
                    extent: None,
                    unknown_fields: Vec::new(),
                }))
            }
            _ => None,
        },
        LayerBodyPrefix10_1::GroupLayer(group) => {
            let (visible, note) = match group.visibility {
                Some(raw) => variant_bool_to_model(raw, "可见性"),
                None => (true, None),
            };
            if let Some(note) = note {
                diagnostics.push(note);
            }
            let mut children = Vec::new();
            for child in &group.children {
                match child {
                    crate::layers::group_layer::GroupChildEntry10_1::ParsedNewObject {
                        parsed,
                        ..
                    } => {
                        if let Some(node) =
                            body_prefix_node(parsed, entry_index, map_crs_key, map_crs, diagnostics)
                        {
                            children.push(node);
                        }
                    }
                    crate::layers::group_layer::GroupChildEntry10_1::SkippedNewObject {
                        parsed: Some(parsed),
                        ..
                    } => {
                        if let Some(node) =
                            body_prefix_node(parsed, entry_index, map_crs_key, map_crs, diagnostics)
                        {
                            children.push(node);
                        }
                    }
                    _ => {}
                }
            }
            Some(LayerNode::Group(LayerGroup {
                name: group.name.clone(),
                visible,
                children,
                unknown_fields: Vec::new(),
            }))
        }
        LayerBodyPrefix10_1::RasterLayer(raster) => raster_node(raster, entry_index, diagnostics),
        LayerBodyPrefix10_1::CompositeGraphicsLayer(cgl) => {
            diagnostics.push(format!(
                "条目 {}：CompositeGraphicsLayer（名称 {:?}）为图形层，中间模型没有对应\
                 节点类型，仅保留诊断",
                entry_index + 1,
                cgl.name
            ));
            None
        }
    }
}

/// RasterLayer 的名称和可见性均经 10.1 ILayer 虚表和 Load 的完整对象偏移对位。
fn raster_node(
    prefix: &crate::layers::raster_layer::RasterLayerPrefix10_1,
    entry_index: usize,
    diagnostics: &mut Vec<String>,
) -> Option<LayerNode> {
    let continuation = prefix.continuation.as_ref()?;
    let (visible, note) = variant_bool_to_model(continuation.fixed_shorts[0], "可见性");
    if let Some(note) = note {
        diagnostics.push(format!("条目 {}：{note}", entry_index + 1));
    }
    Some(LayerNode::Layer(Layer {
        name: prefix.first_string.clone(),
        visible,
        source: None,
        crs: None,
        extent: None,
        unknown_fields: Vec::new(),
    }))
}

/// 输出后缀对象槽位（MapSurround / 书签条目）的诊断行。
fn push_slot_diagnostics(slots: &[ObjectSlot10_1], label: &str, diagnostics: &mut Vec<String>) {
    for (position, slot) in slots.iter().enumerate() {
        diagnostics.push(format!(
            "{label} {}/{}（offset 0x{:x}）：{}",
            position + 1,
            slots.len(),
            slot.offset,
            slot.describe()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
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

    /// 后缀链（与样本同构的最小完整链）：图层列表计数 0 + 活动索引 -1 +
    /// 16 零空标记基本图形层 + MapSurround 计数 0 + WKSEnvelope（样本同值的
    /// 写方向取证见空间参考文档）+ 两个 int32（样本值 9、9）+ 16 零空标记
    /// IEnvelope + 书签计数 0。
    fn suffix_chain() -> Vec<u8> {
        let mut bytes = 0_i32.to_le_bytes().to_vec(); // 图层列表计数
        bytes.extend((-1_i32).to_le_bytes()); // 活动图层索引（样本 -1）
        bytes.extend([0_u8; 16]); // 基本图形层：空标记
        bytes.extend(0_i32.to_le_bytes()); // MapSurround 计数（样本 0）
        for value in [37392944.284_f64, 2520083.2851, 37394230.529, 2520951.8351] {
            bytes.extend(value.to_le_bytes()); // WKSEnvelope（样本值）
        }
        bytes.extend(9_i32.to_le_bytes()); // 两个 int32（样本 9、9）
        bytes.extend(9_i32.to_le_bytes());
        bytes.extend([0_u8; 16]); // IEnvelope：空标记
        bytes.extend(0_i32.to_le_bytes()); // 书签计数（样本 0）
        bytes
    }

    /// 可闭合 Legend 对象体，用于核对解析诊断直连流内主体版本。
    fn suffix_chain_with_legend(legend_version: u16) -> Vec<u8> {
        let mut bytes = 0_i32.to_le_bytes().to_vec(); // 图层列表计数
        bytes.extend((-1_i32).to_le_bytes()); // 活动图层索引
        bytes.extend([0_u8; 16]); // 基本图形层空标记
        bytes.extend(1_i32.to_le_bytes()); // 一个 MapSurround
        bytes.extend(crate::maps::map_surround::LEGEND_CLASS_ID);
        bytes.extend(0x2b_u32.to_le_bytes()); // Legend 对象键

        bytes.extend(legend_version.to_le_bytes());
        bytes.extend(3_u16.to_le_bytes()); // 共同前缀版本
        bytes.extend(encoded("图例"));
        bytes.extend([0_u8; 16]); // 关联地图空槽
        bytes.extend(0_u16.to_le_bytes()); // 共同前缀 short
        bytes.extend([0_u8; 32]); // 共同前缀原始块
        bytes.extend([0_u8; 16]); // LegendFormat 空槽
        bytes.extend(encoded("标题"));
        if legend_version == 1 {
            bytes.extend(1.0_f64.to_le_bytes());
            bytes.extend(2.0_f64.to_le_bytes());
        }
        bytes.extend([0_u16.to_le_bytes(); 3].concat()); // 三个 short
        bytes.extend(0xffff_u16.to_le_bytes()); // 集合前 short
        bytes.extend(0_i32.to_le_bytes()); // LegendItem 计数
        if legend_version >= 3 {
            bytes.extend(0_u16.to_le_bytes());
        }
        if legend_version >= 4 {
            bytes.extend(0_u16.to_le_bytes());
        }
        if legend_version >= 5 {
            bytes.extend([0_u16.to_le_bytes(); 3].concat());
            bytes.extend(0.0_f64.to_le_bytes());
        }

        for value in [0.0_f64; 4] {
            bytes.extend(value.to_le_bytes()); // WKSEnvelope
        }
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0_u8; 16]); // IEnvelope 空槽
        bytes.extend(0_i32.to_le_bytes()); // 书签计数
        bytes
    }

    fn suffix_chain_with_aoi_bookmark() -> Vec<u8> {
        let mut bytes = 0_i32.to_le_bytes().to_vec(); // 图层列表计数
        bytes.extend((-1_i32).to_le_bytes()); // 活动图层索引
        bytes.extend([0_u8; 16]); // 基本图形层空标记
        bytes.extend(0_i32.to_le_bytes()); // MapSurround 计数
        for value in [0.0_f64; 4] {
            bytes.extend(value.to_le_bytes()); // WKSEnvelope
        }
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend([0_u8; 16]); // IEnvelope 空槽
        bytes.extend(1_i32.to_le_bytes()); // 一个 AOIBookmark
        bytes.extend(crate::maps::map_surround::AOI_BOOKMARK_CLASS_ID);
        bytes.extend(0x2b_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes()); // AOIBookmark 版本
        bytes.extend([0_u8; 16]); // 位置 Envelope 空槽
        bytes.extend(encoded("样例书签"));
        bytes
    }

    /// 完整 `/Maps` 合成流：数量 + 首对象头（Map 类 + 键）+ 对象体（前缀 +
    /// 后缀链）+ 未知尾部。
    fn maps_stream(count: i32, body: &[u8], tail: &[u8]) -> Vec<u8> {
        let mut bytes = count.to_le_bytes().to_vec();
        bytes.extend(MAP_CLASS_ID);
        bytes.extend(0x2a_u32.to_le_bytes());
        bytes.extend(body);
        bytes.extend(tail);
        bytes
    }

    /// 单数据框参考流：与仓库样本布局对齐（「新建数据框」、空第二名称、PCS、
    /// 完整后缀链）+ 调用者提供的未知尾部。
    fn sample_stream(tail: &[u8]) -> Vec<u8> {
        let mut body = frame_body("新建数据框", "", "PROJCS[\"CGCS2000\"]");
        body.extend(suffix_chain());
        maps_stream(1, &body, tail)
    }

    #[test]
    fn fills_names_wkt_extent_and_preserves_unknown_tail() {
        let tail = [0xaa, 0xbb, 0xcc];
        let stream = sample_stream(&tail);
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.count, 1);
        assert_eq!(report.map.frames.len(), 1);
        let frame = &report.map.frames[0];
        assert_eq!(frame.name, "新建数据框");
        assert_eq!(frame.secondary_name.as_deref(), Some(""));
        // 图层列表计数 0：无可解析条目，children 保持空，不伪造节点。
        assert_eq!(frame.children, []);
        // WKSEnvelope 到达：extent 按写方向确定的 [XMin,YMin,XMax,YMax] 填充。
        let extent = frame.extent.expect("完整后缀链应到达 WKSEnvelope");
        assert_eq!(extent.xmin, 37392944.284);
        assert_eq!(extent.ymin, 2520083.2851);
        assert_eq!(extent.xmax, 37394230.529);
        assert_eq!(extent.ymax, 2520951.8351);
        let crs = frame.crs.as_ref().unwrap();
        assert_eq!(crs.definition, "PROJCS[\"CGCS2000\"]");
        let projected = crs.projected.as_ref().expect("PCS 体字段应进入模型");
        assert_eq!(projected.save_version, 6);
        assert_eq!(projected.domains, Some([vec![], vec![], vec![], vec![]]));
        assert_eq!(projected.vertical_flag, Some(0));
        assert_eq!(projected.precision_flags, Some(1));
        assert_eq!(projected.precision_doubles, Some(SAMPLE_SEVEN_DOUBLES));
        assert_eq!(projected.trailing_doubles, Some(SAMPLE_THREE_DOUBLES));
        assert_eq!(crs.unknown_fields, []);
        // 图层列表为空（计数 0）：除尾部外没有其他未知体；尾部起点是书签
        // 计数之后的停止点。
        assert_eq!(frame.unknown_fields.len(), 1);
        let unknown = &frame.unknown_fields[0];
        assert_eq!(unknown.stream, "/Maps");
        assert_eq!(unknown.offset as usize, stream.len() - tail.len());
        assert_eq!(unknown.bytes, tail);
        // 报告携带图层列表（活动索引放报告不放模型）。
        let list = report.layer_list.as_ref().unwrap();
        assert_eq!(list.count, 0);
        assert_eq!(list.entries, []);
        assert_eq!(list.active_index, Some(-1));
        assert_eq!(list.stop, None);
        assert_eq!(report.version_candidates, Vec::<String>::new());
        // 工程名称与根级未知字段均未确认。
        assert_eq!(report.map.name, None);
        assert_eq!(report.map.unknown_fields, []);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("图层列表候选"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("活动图层索引原始值 -1"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("显示范围"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("书签计数 0"))
        );
    }

    #[test]
    fn envelope_crs_back_reference_requires_exact_map_object_key() {
        let map_crs = CoordinateSystem {
            definition: "GEOGCS[\"Map CRS\"]".to_string(),
            projected: None,
            unknown_fields: Vec::new(),
        };
        let reference = crate::objects::envelope::SpatialReferenceEntry10_1::BackReference {
            class_id:
                crate::objects::spatial_reference_slots::GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID,
            key: SAMPLE_PCS_KEY,
        };
        let mut diagnostics = Vec::new();
        let matched = envelope_reference_to_model(
            Some(&reference),
            Some(SAMPLE_PCS_KEY),
            Some(&map_crs),
            0,
            &mut diagnostics,
        );
        assert_eq!(matched, Some(map_crs.clone()));

        let mismatched = envelope_reference_to_model(
            Some(&reference),
            Some(SAMPLE_PCS_KEY + 1),
            Some(&map_crs),
            0,
            &mut diagnostics,
        );
        assert_eq!(mismatched, None);
        assert!(diagnostics.iter().any(|note| note.contains("不匹配")));

        // 同键但不同类也不能复用 Map CRS。
        let wrong_class = crate::objects::envelope::SpatialReferenceEntry10_1::BackReference {
            class_id: crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            key: SAMPLE_PCS_KEY,
        };
        assert_eq!(
            envelope_reference_to_model(
                Some(&wrong_class),
                Some(SAMPLE_PCS_KEY),
                Some(&map_crs),
                0,
                &mut diagnostics,
            ),
            None
        );
    }

    #[test]
    fn envelope_inline_crs_is_independent_of_map_crs() {
        use crate::objects::envelope::SpatialReferenceEntry10_1;

        let bytes = pcs_body("PROJCS[\"Layer CRS\"]");
        let body = crate::objects::spatial_reference::read_projected_coordinate_system_10_1(
            &mut Reader::new(&bytes),
            MAX_FIELD_BYTES,
        )
        .unwrap();
        let reference = SpatialReferenceEntry10_1::Projected {
            key: 81,
            body: body.clone(),
        };
        let mut diagnostics = Vec::new();
        let crs =
            envelope_reference_to_model(Some(&reference), None, None, 0, &mut diagnostics).unwrap();
        assert_eq!(crs.definition, body.wkt);
        assert_eq!(
            crs.projected.unwrap().precision_doubles,
            body.precision.map(|p| p.doubles)
        );
        assert!(diagnostics.is_empty());
        let map_crs = CoordinateSystem {
            definition: "GEOGCS[\"Map CRS\"]".to_string(),
            projected: None,
            unknown_fields: Vec::new(),
        };
        assert_eq!(
            envelope_reference_to_model(
                Some(&SpatialReferenceEntry10_1::Null),
                None,
                Some(&map_crs),
                0,
                &mut diagnostics,
            ),
            None
        );
    }

    #[test]
    fn reference_sample_associated_layer_crs_reaches_model() {
        // 仅检查仓库样本的解析接线，不代表 ArcMap/QGIS 软件验收。
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/测试底图.mxd");
        let mut document = cfb::open(path).unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut document.open_stream("/Maps").unwrap(), &mut bytes)
            .unwrap();
        let report = parse_maps_10_1(&bytes).unwrap();
        let list = report.layer_list.as_ref().expect("应解析图层列表");
        assert_eq!(list.entries.len(), 6, "FDO 完整消费后应继续读取后续图层");
        assert_eq!(list.entries[1].offset, 0x2d22);
        assert_eq!(list.active_index, Some(-1));
        let Some(LayerBodyPrefix10_1::FdoGraphicsLayer(fdo)) = &list.entries[0].parsed else {
            panic!("首个图层应为已解析的 FDO 对象");
        };
        assert!(fdo.completion().is_some());
        assert_eq!(
            fdo.fdo_tail
                .as_ref()
                .unwrap()
                .sublayers
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        let frame = &report.map.frames[0];
        let LayerNode::Layer(layer) = &frame.children[0] else {
            panic!("参考样本首节点应为 FDO 关联要素图层");
        };
        let crs = layer.crs.as_ref().expect("已解析 Envelope 应提供图层 CRS");
        assert!(!crs.definition.is_empty());
        assert!(crs.projected.is_some());
        let model_rasters = &frame.children[frame.children.len() - 2..];
        for ((index, expected_visible), model_node) in
            [(4, false), (5, true)].into_iter().zip(model_rasters)
        {
            let Some(LayerBodyPrefix10_1::RasterLayer(raster)) = &list.entries[index].parsed else {
                panic!("参考样本应读取栅格图层前缀");
            };
            let LayerNode::Layer(node) = model_node else {
                panic!("栅格应成为叶节点");
            };
            assert!(!node.name.is_empty());
            assert_eq!(node.name, raster.first_string);
            assert_eq!(node.visible, expected_visible);
        }
        let Some(LayerBodyPrefix10_1::RasterLayer(raster)) = &list.entries[5].parsed else {
            unreachable!();
        };
        let name = raster
            .dataset_name
            .as_ref()
            .expect("普通栅格名称体应已消费");
        assert_eq!(name.key, 1545);
        assert!(name.stop.is_none());
        assert_eq!(list.entries[5].body.unwrap().offset, 0x14862);
    }

    #[test]
    fn layer_entries_are_reported_and_bodies_kept_as_unknown() {
        // 条目 1：标志 2 前缀 [A=26][B=0]，未知类新对象，体 6 字节 → 有界跳过
        //（A 观察推定：体 = A - 20）；条目 2：标志 2 前缀 [20][0] 对条目 1 键的
        // 回引（无体）；条目 3：标志 3 前缀 [0][0] 空条目；条目 4：标志 1 新对象
        //（样本条目 1 同款 FDOGraphicsLayer 类标识）→ 体无外部长度，停止。
        let mut suffix = 4_i32.to_le_bytes().to_vec();
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend(26_u32.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend([0x99_u8; 16]);
        suffix.extend(51_u32.to_le_bytes());
        suffix.extend([0xee_u8; 6]);
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend(20_u32.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend([0x99_u8; 16]);
        suffix.extend(51_u32.to_le_bytes());
        suffix.extend(3_u16.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(1_u16.to_le_bytes());
        suffix.extend([
            0x83, 0xef, 0xb2, 0x34, 0xac, 0xf4, 0xd1, 0x11, 0xa2, 0x45, 0x08, 0x00, 0x09, 0xb6,
            0xf2, 0x2b,
        ]);
        suffix.extend(31_u32.to_le_bytes());
        let tail = [0x11, 0x22, 0x33];
        let mut body = frame_body("框", "", "PROJCS[\"X\"]");
        body.extend(&suffix);
        let stream = maps_stream(1, &body, &tail);
        let report = parse_maps_10_1(&stream).unwrap();

        let list = report.layer_list.as_ref().unwrap();
        assert_eq!(list.count, 4);
        assert_eq!(list.entries.len(), 4);
        assert_eq!(list.active_index, None, "中途停止不得读取活动索引");
        // 条目 1：有界未知体（A 推定）。
        let body_range = list.entries[0].body.expect("未知类新对象应有有界体");
        assert_eq!(body_range.len, 6);
        // 条目 2：回引（同键命中登记表）。
        assert!(matches!(
            list.entries[1].object,
            crate::layers::layer_list::LayerEntryObject::Entry(
                crate::storage::object_refs::ObjectEntry10_1::BackReference { key: 51, .. }
            )
        ));
        assert_eq!(list.entries[1].body, None);
        // 条目 3：空条目。
        assert!(matches!(
            list.entries[2].object,
            crate::layers::layer_list::LayerEntryObject::Empty
        ));
        // 条目 4：标志 1 新对象 → 列表停止。
        assert_eq!(list.entries[3].flag, 1);
        assert_eq!(list.entries[3].body, None);

        let frame = &report.map.frames[0];
        // 有界未知体按条目保留为数据框未知字段，其后是停止点起的未知尾部；
        // 不为未解析体构造 LayerNode。
        assert_eq!(frame.children, []);
        assert_eq!(frame.unknown_fields.len(), 2);
        assert_eq!(frame.unknown_fields[0].bytes, vec![0xee; 6]);
        assert_eq!(
            frame.unknown_fields[1].offset as usize,
            stream.len() - tail.len()
        );
        assert_eq!(frame.unknown_fields[1].bytes, tail);
        // 显示范围未到达（在图层列表条目 4 停止），诊断必须说明原因。
        assert_eq!(frame.extent, None);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("显示范围未到达"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("图层条目 1/4"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("条目 1 的体剩余 6 字节未解析"))
        );
    }

    #[test]
    fn suffix_stops_when_base_graphics_cgl_body_fails_to_parse() {
        // 图层列表计数 0 + 活动索引 -1 之后：基本图形层为 CompositeGraphicsLayer
        // 新对象（样本 0x16fba 同款类标识）。体首 short=0x5555 超出版本上限，
        // CGL 前缀解析失败按无前缀结果降级，后缀在该槽位停止；MapSurround
        // 计数不读取，体保留为未知尾部。
        let mut suffix = 0_i32.to_le_bytes().to_vec();
        suffix.extend((-1_i32).to_le_bytes());
        suffix.extend(crate::layers::layer_list::COMPOSITE_GRAPHICS_LAYER_CLASS_ID);
        suffix.extend(1553_u32.to_le_bytes());
        suffix.extend([0x55; 8]);
        let mut body = frame_body("框", "", "PROJCS[\"X\"]");
        body.extend(&suffix);
        let stream = maps_stream(1, &body, &[]);
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.map.frames[0].extent, None);
        assert_eq!(report.map.frames[0].children, []);
        let list = report.layer_list.as_ref().unwrap();
        assert_eq!(list.active_index, Some(-1));
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("基本图形层对象") && note.contains("新对象"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("后缀读取停止"))
        );
        // 前缀解析失败：不出现「基本图形层体前缀」诊断。
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|note| note.contains("基本图形层体前缀"))
        );
        // 未知尾部从基本图形层对象头之后开始（体起点）。
        let unknown = &report.map.frames[0].unknown_fields[0];
        assert_eq!(unknown.bytes, vec![0x55; 8]);
    }

    #[test]
    fn suffix_parses_cgl_base_graphics_prefix_before_stopping() {
        // 基本图形层为合法 CompositeGraphicsLayer 前缀体：版本 6、元素组计数 0、
        // 名称、short ×3、double ×2、int32 ×3、嵌套空标记、版本 >1 的
        // double+int32、子图层计数 0，然后尾段遇未知新对象。该槽位对象体无
        // 外部长度，后缀保留未知头及其后数据，但已读部分进入诊断。
        let mut cgl_body = (6_u16).to_le_bytes().to_vec();
        cgl_body.extend(0_i32.to_le_bytes()); // 元素组计数 0
        cgl_body.extend(encoded("G")); // 名称
        cgl_body.extend([0xff_u8; 6]); // short ×3（原样保留）
        cgl_body.extend([0_u8; 16]); // double ×2
        cgl_body.extend(
            [
                3_u32.to_le_bytes(),
                1_u32.to_le_bytes(),
                1_u32.to_le_bytes(),
            ]
            .concat(),
        );
        cgl_body.extend([0_u8; 16]); // 嵌套对象：空标记
        cgl_body.extend(0_f64.to_le_bytes()); // 版本 >1：double
        cgl_body.extend(0_i32.to_le_bytes()); // 版本 >1：int32
        cgl_body.extend(0_i32.to_le_bytes()); // 子图层计数 0
        cgl_body.extend(1_u32.to_le_bytes()); // 循环后 int32
        let unknown_entry = [0x77; 20]; // 未知类标识 + 未登记对象键
        let mut suffix = 0_i32.to_le_bytes().to_vec();
        suffix.extend((-1_i32).to_le_bytes());
        suffix.extend(crate::layers::layer_list::COMPOSITE_GRAPHICS_LAYER_CLASS_ID);
        suffix.extend(1553_u32.to_le_bytes());
        suffix.extend(&cgl_body);
        suffix.extend(unknown_entry);
        let mut body = frame_body("框", "", "PROJCS[\"X\"]");
        body.extend(&suffix);
        let stream = maps_stream(1, &body, &[]);
        let report = parse_maps_10_1(&stream).unwrap();

        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("基本图形层体前缀") && note.contains("\"G\""))
        );
        // MapSurround 计数仍未读取（槽位体无外部长度，后缀停止）。
        assert_eq!(report.map.frames[0].extent, None);
        // 未知尾部从新对象头起保留。
        let unknown = &report.map.frames[0].unknown_fields[0];
        assert_eq!(unknown.bytes, unknown_entry);
    }

    #[test]
    fn feature_dataset_workspace_path_reaches_the_layer_source() {
        use crate::layers::feature_layer::{
            FEATURE_CLASS_NAME_CLASS_ID, WORKSPACE_NAME_CLASS_ID, read_feature_layer_prefix_10_1,
        };
        use crate::objects::dataset_name::FEATURE_DATASET_NAME_CLASS_ID;

        let mut body = 34_u16.to_le_bytes().to_vec();
        body.extend(encoded("道路层"));
        body.extend(encoded(""));
        body.extend([0; 6]); // 可见性等三个 short
        body.extend(FEATURE_CLASS_NAME_CLASS_ID);
        body.extend(10_u32.to_le_bytes());
        body.extend(2_u16.to_le_bytes());
        for value in ["道路", "", "要素类", "交通"] {
            body.extend(encoded(value));
        }
        body.extend([0; 8]); // 两个 int32
        body.extend(1_u16.to_le_bytes()); // 选择 IDatasetName
        body.extend(FEATURE_DATASET_NAME_CLASS_ID);
        body.extend(11_u32.to_le_bytes());
        body.extend(1_u16.to_le_bytes());
        for value in ["交通", "", "要素数据集", ""] {
            body.extend(encoded(value));
        }
        body.push(0);
        body.extend(WORKSPACE_NAME_CLASS_ID);
        body.extend(12_u32.to_le_bytes());
        body.extend(1_u16.to_le_bytes());
        for value in ["..\\data.gdb", "", "File Geodatabase"] {
            body.extend(encoded(value));
        }
        body.extend([0; 16 + 1 + 4]); // 空 PropertySet、工厂标志 0、尾 int32
        body.extend(0_u16.to_le_bytes()); // 不含 TopologyName
        body.extend([0xa5; 20]); // 未知渲染器，读取器在该槽停止

        let mut reader = Reader::new(&body);
        let prefix =
            read_feature_layer_prefix_10_1(&mut reader, 1024, &mut RefTable10_1::new()).unwrap();
        let source = feature_layer_source(&prefix, 0, &mut Vec::new()).unwrap();
        assert_eq!(source.path.as_deref(), Some("..\\data.gdb"));
        assert_eq!(source.connection.as_deref(), Some("道路"));
    }

    #[test]
    fn layer_entries_with_parsed_prefixes_fill_children_in_order() {
        // 条目 1：标志 2 的 FeatureLayer 新对象（A 界）：体前缀（版本 34、双名
        // 称、可见性 0xFFFF、FCN 数据源全链——含 WorkspaceName 体与版本 >1
        // 选择子）解析后，剩余 2 字节按 A 界跳过；
        // 条目 2：对同一键的回引 → 复用条目 1 的节点（原生同一对象的再现）。
        // 两个节点按条目顺序进入 children（绘制顺序候选）。
        let mut fl_body = (34_u16).to_le_bytes().to_vec();
        fl_body.extend(encoded("路"));
        fl_body.extend(encoded("要素"));
        fl_body.extend(0xFFFF_u16.to_le_bytes()); // 可见性 VARIANT_TRUE
        fl_body.extend(0_u16.to_le_bytes());
        fl_body.extend(0_u16.to_le_bytes());
        fl_body.extend(crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID);
        fl_body.extend(78_u32.to_le_bytes()); // FCN 键
        fl_body.extend(2_u16.to_le_bytes()); // FCN 版本（样本 02）
        fl_body.extend(encoded("JZD")); // 要素类名
        fl_body.extend(encoded("")); // 空第二名称
        fl_body.extend(encoded("Shapefile 要素类")); // 类型描述
        fl_body.extend(encoded("Shape")); // 数据集名
        fl_body.extend(1_i32.to_le_bytes());
        fl_body.extend(1_i32.to_le_bytes());
        fl_body.extend(0_u16.to_le_bytes()); // 选择子 0 → WS 槽
        fl_body.extend(crate::layers::feature_layer::WORKSPACE_NAME_CLASS_ID);
        fl_body.extend(79_u32.to_le_bytes()); // WS 键
        fl_body.extend(1_u16.to_le_bytes()); // WS 版本
        fl_body.extend(encoded("..\\data")); // 路径
        fl_body.extend(encoded("")); // 空第二路径
        fl_body.extend(encoded("Shapefile Data")); // 类型描述
        fl_body.extend([0_u8; 16]); // IPropertySet 槽：空标记
        fl_body.push(1); // 工厂标志
        fl_body.extend([0x96; 16]); // 工厂 CLSID（原样）
        fl_body.extend(0_i32.to_le_bytes()); // 尾随 int32
        fl_body.extend(0_u16.to_le_bytes()); // 版本 >1 选择子 0
        fl_body.extend([0_u8; 16]); // 渲染器槽：空标记
        fl_body.extend([0x6A; 16]); // +0x144 GUID
        fl_body.extend(encoded("")); // +0x154 空串（语义未定）
        fl_body.extend(0_f64.to_le_bytes()); // +0x158
        fl_body.extend(0_f64.to_le_bytes()); // +0x160
        fl_body.extend(0_i16.to_le_bytes()); // +0x168
        fl_body.extend(0_i32.to_le_bytes()); // IHyperlink 计数 0
        fl_body.extend([0_u8; 16]); // IColor：空标记
        fl_body.extend(encoded("")); // +0x1a8
        fl_body.extend(encoded("")); // +0x184
        fl_body.extend(0_i32.to_le_bytes()); // +0x188
        fl_body.extend(0_i32.to_le_bytes()); // FieldInfo 计数 0
        fl_body.extend([0xaa, 0xaa]); // 体前缀之后的未解析剩余（A 界内）

        let mut suffix = 2_i32.to_le_bytes().to_vec(); // 图层列表计数
        // 条目 1：标志 2 + 前缀 [A=头 20+体][B=0] + FL 头（键 77）+ 体。
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend((20 + fl_body.len() as u32).to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(crate::layers::layer_list::FEATURE_LAYER_CLASS_ID);
        suffix.extend(77_u32.to_le_bytes());
        suffix.extend(&fl_body);
        // 条目 2：标志 2 + 前缀 [A=20][B=0] + FL 头（键 77 → 回引，无体）。
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend(20_u32.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(crate::layers::layer_list::FEATURE_LAYER_CLASS_ID);
        suffix.extend(77_u32.to_le_bytes());
        suffix.extend((-1_i32).to_le_bytes()); // 活动图层索引
        suffix.extend([0_u8; 16]); // 基本图形层：空标记
        suffix.extend(0_i32.to_le_bytes()); // MapSurround 计数
        for value in [1.0_f64, 2.0, 3.0, 4.0] {
            suffix.extend(value.to_le_bytes()); // WKSEnvelope
        }
        suffix.extend(0_i32.to_le_bytes());
        suffix.extend(0_i32.to_le_bytes());
        suffix.extend([0_u8; 16]); // IEnvelope：空标记
        suffix.extend(0_i32.to_le_bytes()); // 书签计数

        let mut body = frame_body("框", "", "PROJCS[\"X\"]");
        body.extend(&suffix);
        let stream = maps_stream(1, &body, &[]);
        let report = parse_maps_10_1(&stream).unwrap();

        let frame = &report.map.frames[0];
        assert_eq!(frame.children.len(), 2);
        let LayerNode::Layer(layer) = &frame.children[0] else {
            panic!("条目 1 应构造 Layer 节点");
        };
        assert_eq!(layer.name, "路");
        assert!(layer.visible);
        assert_eq!(
            layer
                .source
                .as_ref()
                .and_then(|source| source.connection.as_deref()),
            Some("JZD")
        );
        // 回引条目复用同对象节点（与首个节点完全一致）。
        assert_eq!(frame.children[1], frame.children[0]);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("回引（键 77）命中先前已解析对象"))
        );
        // 条目 1 体前缀之后的 2 字节保留为未知字段。
        assert!(
            frame
                .unknown_fields
                .iter()
                .any(|unknown| unknown.bytes == vec![0xaa, 0xaa])
        );
        // 显示范围与活动索引正常到达/读取。
        let extent = frame.extent.expect("后缀应到达 WKSEnvelope");
        assert_eq!(
            (extent.xmin, extent.ymin, extent.xmax, extent.ymax),
            (1.0, 2.0, 3.0, 4.0)
        );
        assert_eq!(report.layer_list.as_ref().unwrap().active_index, Some(-1));
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
        assert_eq!(report.map.unknown_fields[0].offset, 20);
        assert_eq!(report.map.unknown_fields[0].bytes, [0, 0, 0, 0, 0x11, 0x22]);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("空引用标记"))
        );
    }

    #[test]
    fn only_first_frame_is_parsed_when_count_exceeds_one() {
        let mut body = frame_body("甲框", "", "PROJCS[\"X\"]");
        body.extend(suffix_chain());
        let stream = maps_stream(3, &body, &[0x55, 0x66]);
        let report = parse_maps_10_1(&stream).unwrap();

        assert_eq!(report.map.frames.len(), 1);
        assert_eq!(report.map.frames[0].name, "甲框");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|note| note.contains("其余 2 个数据框对象未解析"))
        );
        // 第 2/3 个数据框对象与循环尾部都落在未知尾部中。
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
        body.extend(suffix_chain());
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
    fn parsed_legend_versions_are_reported_for_old_and_current_layouts() {
        for version in [1_u16, 5] {
            let mut body = SAMPLE_VERSION.to_le_bytes().to_vec();
            body.extend(encoded("图例测试框"));
            body.extend(encoded(""));
            body.extend([0_u8; 16]); // Map 空间参考
            body.extend(suffix_chain_with_legend(version));
            let report = parse_maps_10_1(&maps_stream(1, &body, &[])).unwrap();

            assert!(report.diagnostics.iter().any(|line| {
                line.contains(&format!("MapSurround Legend 条目 1：主体版本 {version}"))
            }));
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|line| line.contains("书签计数 0"))
            );
        }
    }

    #[test]
    fn nonempty_aoi_bookmark_is_consumed() {
        let mut body = SAMPLE_VERSION.to_le_bytes().to_vec();
        body.extend(encoded("书签测试框"));
        body.extend(encoded(""));
        body.extend([0_u8; 16]); // Map 空间参考
        body.extend(suffix_chain_with_aoi_bookmark());
        let report = parse_maps_10_1(&maps_stream(1, &body, &[])).unwrap();

        assert!(
            report
                .diagnostics
                .iter()
                .any(|line| line.contains("书签计数 1"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|line| line.contains("书签条目 1/1"))
        );
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|line| line.contains("后缀读取停止：书签条目 0"))
        );
    }

    #[test]
    fn truncated_nonempty_aoi_bookmark_stops_at_its_body_and_keeps_tail() {
        let mut body = SAMPLE_VERSION.to_le_bytes().to_vec();
        body.extend(encoded("书签测试框"));
        body.extend(encoded(""));
        body.extend([0_u8; 16]); // Map 空间参考
        let mut suffix = suffix_chain_with_aoi_bookmark();
        suffix.pop(); // 破坏名称字符串的 UTF-16LE 终止符，模拟短读
        body.extend(suffix);
        let report = parse_maps_10_1(&maps_stream(1, &body, &[])).unwrap();

        assert!(
            report
                .diagnostics
                .iter()
                .any(|line| line.contains("后缀读取停止：书签条目 0"))
        );
        assert!(!report.map.frames[0].unknown_fields.is_empty());
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
        // 前缀 + 后缀链的终点：截断进入其内部的任何长度都必须整体失败
        //（全部严格读取器都是事务性的，后缀截断同样属于失败而非停止点）。
        let suffix_end = stream.len() - tail.len();
        assert!(suffix_end > 250, "完整输入应覆盖所有字段区域");
        for length in 0..suffix_end {
            assert!(
                parse_maps_10_1(&stream[..length]).is_err(),
                "长度 {length} 不应解析成功"
            );
        }
        // 截断落在书签计数之后的未知尾部时后缀已完整，解析成功且未知字节随之
        // 缩短；恰好截到终点时未知字节为空（unknown_fields 无条目）。
        let report = parse_maps_10_1(&stream[..suffix_end]).unwrap();
        assert_eq!(report.map.frames[0].unknown_fields, []);
        for length in (suffix_end + 1)..stream.len() {
            let report = parse_maps_10_1(&stream[..length]).unwrap();
            assert_eq!(
                report.map.frames[0].unknown_fields[0].bytes.len(),
                length - suffix_end
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
        let mut short = frame_body("框", "", "P");
        short.extend(suffix_chain());
        let short = maps_stream(1, &short, &[]);
        assert!(parse_maps_10_1_with_limits(&short, MAX_MAP_COUNT, 16).is_ok());
        let mut long_wkt = frame_body("框", "", "PROJCS[\"超限WKT字符串\"]");
        long_wkt.extend(suffix_chain());
        let long_wkt = maps_stream(1, &long_wkt, &[]);
        let error = parse_maps_10_1_with_limits(&long_wkt, MAX_MAP_COUNT, 16).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }
}
