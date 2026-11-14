//! 将 ArcMap 10.1 `/Maps` 数据流的已确认前缀与后缀解析为中间模型。
//!
//! 本模块是显式 10.1 读取器与 [`crate::model`] 中间模型的接线层，仅调用
//! 以下已确认读取器，不新增任何字段解释：
//!
//! - [`crate::maps_stream::read_root_prefix_10_1`]：4 字节有符号数量、首个
//!   对象头、循环后的 4 字节尾部原始值（证据：`docs/arcmap-10.1-object-body.md`）；
//! - [`crate::object_stream::read_header_10_1`]：Map 对象头；
//! - [`crate::map_object::read_map_prefix_10_1`]：Map 对象体已确认前缀
//!   （架构版本 short、双名称字符串、嵌套空间参考对象）；
//! - [`crate::spatial_reference::read_projected_coordinate_system_10_1`]：
//!   ProjectedCoordinateSystem 对象体（经 Map 前缀读取器内部调用）；
//! - [`crate::map_object::read_map_suffix_10_1`]：Map 对象体后缀序列
//!   （图层列表 → 基本图形层 → MapSurround 循环 → WKSEnvelope → 两个
//!   int32 → IEnvelope → 书签循环，到书签循环为止，其后保留为未知尾部；
//!   内部经 [`crate::layer_list::read_layer_list_10_1`] 读取图层列表）。
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
//! - 图层条目经 [`crate::layer_list`] 的类分派解析出体前缀时构造
//!   [`LayerNode`](crate::model::LayerNode)（FeatureLayer → 名称/第二名称/
//!   可见性/数据源 FeatureClassName 要素类名；GroupLayer → 名称/可见性），
//!   按条目顺序进入 `children`（绘制顺序候选）；回引条目命中先前已解析键
//!   时复用同对象节点。CompositeGraphicsLayer 图形层、未解析体与解析失败
//!   的条目不构造节点，体剩余/整段未解析体按条目保留为数据框未知字段，
//!   停止点之后的全部字节保留为尾部未知字段；
//! - 活动图层索引与各对象槽位只进诊断与报告
//!   （[`MapsParseReport10_1::layer_list`]）；
//! - 工程名称、其余数据框对象与根尾部值之后的字段没有已确认证据，保留为
//!   [`UnknownField`](crate::model::UnknownField) 或空字段，不做任何推断。
//!
//! 对象循环边界未闭合：Map 之后的对象体没有外部长度字段，本模块不跳过未知
//! 对象，因此无论数量是多少，最多只解析首个 Map 对象。正数量时根前缀读取器
//! 在首对象头之后读取的 4 字节按写侧证据（数量 → 对象循环 → 循环尾部值）
//! 属于首对象体，不是循环尾部值，本模块不把它解释为尾部。
//!
//! 解析失败（根前缀错误、首对象非 Map、已确认前缀或后缀内的截断与非法值）
//! 返回明确错误且不返回部分模型；后缀在结构停止点（标志 1 新对象体、未接入
//! 读取器的新对象体、书签循环之后的未闭合区）的输出是成功返回 + 未知尾部。
//! 解析结果与诊断都是候选观察：单元测试仅验证代码自身契约，解析规则未在
//! ArcMap 10.1 或 QGIS 中人工目视验证；通过解析不代表 MXD 格式已验证。

use crate::feature_layer::DataSource10_1;
use crate::layer_list::{LayerBodyPrefix10_1, LayerEntryObject, LayerList10_1};
use crate::map_object::{MAP_CLASS_ID, ObjectSlot10_1, read_map_prefix_10_1, read_map_suffix_10_1};
use crate::maps_stream::read_root_prefix_10_1;
use crate::model::{
    CoordinateSystem, DataFrame, DataSource, Extent, Layer, LayerGroup, LayerNode, Map,
    UnknownField,
};
use crate::object_refs::{ObjectEntry10_1, RefTable10_1};
use crate::object_stream::{ObjectHeader, read_header_10_1};
use crate::reader::Reader;
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
    let mut report_layer_list: Option<LayerList10_1> = None;

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
                let (frame, layer_list) =
                    parse_first_frame(maps, root.count, &mut diagnostics, max_bytes)?;
                frames.push(frame);
                report_layer_list = layer_list;
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
        layer_list: report_layer_list,
        version_candidates: Vec::new(),
        diagnostics,
    })
}

/// 解析首个 Map 对象体的已确认前缀与后缀并构造数据框。
///
/// `maps` 是完整流字节；本函数从数量之后重新读取 Map 对象头（与根前缀读取
/// 器的确定性行为一致），再依次调用 [`read_map_prefix_10_1`] 与
/// [`read_map_suffix_10_1`]。后缀使用跨流共享的键登记表：Map 对象自身的键
/// 在读取对象头时登记（与原生 ObjectStream `+0x44` 读头即查表/登记一致），
/// 嵌套 PCS 键与后缀新对象键由后缀读取器登记。返回数据框与图层列表
/// （图层列表放报告不放模型，见模块文档）。
fn parse_first_frame(
    maps: &[u8],
    count: i32,
    diagnostics: &mut Vec<String>,
    max_bytes: usize,
) -> io::Result<(DataFrame, Option<LayerList10_1>)> {
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

    // 后缀读取：图层列表 → 基本图形层 → MapSurround → WKSEnvelope → 两个
    // int32 → IEnvelope → 书签循环，到书签循环为止。后缀读取器从 Map 对象体
    // 起点接收 Reader（内部重读前缀以登记嵌套 PCS 键并定位图层列表起点），
    // 因此这里从数量之后重建一个 Reader；共享登记表先登记 Map 对象自身的键
    //（对象头在上方重读时获得）。
    let map_key = match header {
        ObjectHeader::Object { key, .. } => key,
        ObjectHeader::Null => unreachable!("首对象为 Map 类时对象头不是空标记"),
    };
    let mut table = RefTable10_1::new();
    table.register(map_key);
    let mut suffix_reader = Reader::new(maps);
    suffix_reader.read_bytes(4)?;
    read_header_10_1(&mut suffix_reader)?;
    let suffix = read_map_suffix_10_1(&mut suffix_reader, max_bytes, &mut table)?;

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
                let source = match &prefix.data_source {
                    DataSource10_1::FeatureClassName(fcn) => {
                        diagnostics.push(format!(
                            "条目 {}：数据源 FeatureClassName 前缀（键 {}）→ connection={:?}；\
                             工作空间路径位于 FCN 体未闭合中段，不写入模型",
                            entry.index + 1,
                            fcn.key,
                            fcn.feature_class_name
                        ));
                        Some(DataSource {
                            provider: None,
                            path: None,
                            connection: Some(fcn.feature_class_name.clone()),
                            unknown_fields: Vec::new(),
                        })
                    }
                    DataSource10_1::Null => None,
                    DataSource10_1::BackReference { key, .. } => {
                        diagnostics.push(format!(
                            "条目 {}：数据源为回引条目（键 {key}），无体可解析，source 保留为 None",
                            entry.index + 1
                        ));
                        None
                    }
                };
                Some(LayerNode::Layer(Layer {
                    name: prefix.name.clone(),
                    visible,
                    source,
                    // 图层坐标系宿主为图层体内 Envelope（中段未闭合），保留为 None。
                    crs: None,
                    extent: None,
                    unknown_fields: Vec::new(),
                }))
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
                diagnostics.push(format!(
                    "条目 {}：组图层子图层循环未完全解析（子图层体读取器按类分派，\
                     递归未接入），children 保留为空",
                    entry.index + 1
                ));
                Some(LayerNode::Group(LayerGroup {
                    name: prefix.name.clone(),
                    visible,
                    children: Vec::new(),
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
             {} 组），子图层计数 {}（已消费 {} 条）；读取器在其停止点（{}）处结束，\
             体剩余保留为未知尾部",
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
            "MapSurround 计数 {surround_count}（地图装饰物集合候选，元素 coclass 语义\
             开放，样本计数 0 未经非空实例验证）；已消费 {} 个条目",
            suffix.map_surrounds.len()
        ));
        push_slot_diagnostics(&suffix.map_surrounds, "MapSurround 条目", diagnostics);
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
                Some(crate::envelope::SpatialReferenceEntry10_1::Null) => "空引用标记".to_string(),
                Some(crate::envelope::SpatialReferenceEntry10_1::BackReference { key, .. }) =>
                    format!("回引（键 {key}）"),
                Some(crate::envelope::SpatialReferenceEntry10_1::Projected { key, .. }) =>
                    format!("内嵌 PCS（键 {key}）"),
                None => "该版本分支无此段".to_string(),
            }
        ));
    }
    if let Some(bookmark_count) = suffix.bookmark_count {
        diagnostics.push(format!(
            "书签计数 {bookmark_count}（空间书签集合候选，样本计数 0 未经非空实例\
             验证）；已消费 {} 个条目",
            suffix.bookmarks.len()
        ));
        push_slot_diagnostics(&suffix.bookmarks, "书签条目", diagnostics);
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
    diagnostics.push(format!(
        "后缀读取停止：{}；其后 {} 字节保留为数据框未知尾部（流内偏移 0x{stop_offset:x} \
         起），不解释",
        suffix.stop.describe(),
        suffix.remaining_bytes
    ));
    if suffix.remaining_bytes > 0 {
        frame_unknown.push(UnknownField {
            stream: "/Maps".to_string(),
            offset: stop_offset as u64,
            bytes: maps[stop_offset..].to_vec(),
        });
    }
    if count > 1 {
        diagnostics.push(format!(
            "其余 {} 个数据框对象未解析：对象循环边界未闭合，对象体无外部长度字段，不跳过未知对象",
            count - 1
        ));
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
             （FeatureLayer 名称/可见性/数据源要素类名、GroupLayer 名称/可见性），\
             其余字段保留为空或未知",
            children.len()
        ));
    }

    let frame = DataFrame {
        name: prefix.name,
        secondary_name: Some(prefix.secondary_name),
        children,
        crs: prefix.spatial_reference.map(|pcs| CoordinateSystem {
            definition: pcs.wkt,
            unknown_fields: Vec::new(),
        }),
        extent,
        unknown_fields: frame_unknown,
    };
    Ok((frame, Some(suffix.layer_list.clone())))
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
            crate::layer_list::LayerEntryObject::Entry(
                crate::object_refs::ObjectEntry10_1::BackReference { key: 51, .. }
            )
        ));
        assert_eq!(list.entries[1].body, None);
        // 条目 3：空条目。
        assert!(matches!(
            list.entries[2].object,
            crate::layer_list::LayerEntryObject::Empty
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
        suffix.extend(crate::layer_list::COMPOSITE_GRAPHICS_LAYER_CLASS_ID);
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
        // double+int32、子图层计数 0 → 读取器停在未展开尾段。该槽位对象体无
        // 外部长度，后缀仍在槽位停止，但体前缀进入诊断。
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
        let mut suffix = 0_i32.to_le_bytes().to_vec();
        suffix.extend((-1_i32).to_le_bytes());
        suffix.extend(crate::layer_list::COMPOSITE_GRAPHICS_LAYER_CLASS_ID);
        suffix.extend(1553_u32.to_le_bytes());
        suffix.extend(&cgl_body);
        suffix.extend([0x77; 4]); // CGL 未展开尾段（后缀停止点之后的未知字节）
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
        // 未知尾部 = CGL 体的未展开尾段（读取器停止点之后的字节）。
        let unknown = &report.map.frames[0].unknown_fields[0];
        assert_eq!(unknown.bytes, vec![0x77; 4]);
    }

    #[test]
    fn layer_entries_with_parsed_prefixes_fill_children_in_order() {
        // 条目 1：标志 2 的 FeatureLayer 新对象（A 界）：体前缀（版本 34、双名
        // 称、可见性 0xFFFF、FCN 数据源）解析后，剩余 2 字节按 A 界跳过；
        // 条目 2：对同一键的回引 → 复用条目 1 的节点（原生同一对象的再现）。
        // 两个节点按条目顺序进入 children（绘制顺序候选）。
        let mut fl_body = (34_u16).to_le_bytes().to_vec();
        fl_body.extend(encoded("路"));
        fl_body.extend(encoded("要素"));
        fl_body.extend(0xFFFF_u16.to_le_bytes()); // 可见性 VARIANT_TRUE
        fl_body.extend(0_u16.to_le_bytes());
        fl_body.extend(0_u16.to_le_bytes());
        fl_body.extend(crate::feature_layer::FEATURE_CLASS_NAME_CLASS_ID);
        fl_body.extend(78_u32.to_le_bytes()); // FCN 键
        fl_body.extend(2_u16.to_le_bytes()); // FCN 前缀 short（样本 02）
        fl_body.extend(encoded("JZD")); // 要素类名
        fl_body.extend([0xaa, 0xaa]); // 体前缀之后的未解析剩余（A 界内）

        let mut suffix = 2_i32.to_le_bytes().to_vec(); // 图层列表计数
        // 条目 1：标志 2 + 前缀 [A=头 20+体][B=0] + FL 头（键 77）+ 体。
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend((20 + fl_body.len() as u32).to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(crate::layer_list::FEATURE_LAYER_CLASS_ID);
        suffix.extend(77_u32.to_le_bytes());
        suffix.extend(&fl_body);
        // 条目 2：标志 2 + 前缀 [A=20][B=0] + FL 头（键 77 → 回引，无体）。
        suffix.extend(2_u16.to_le_bytes());
        suffix.extend(20_u32.to_le_bytes());
        suffix.extend(0_u32.to_le_bytes());
        suffix.extend(crate::layer_list::FEATURE_LAYER_CLASS_ID);
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
