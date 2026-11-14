//! ArcMap 10.1 Map（数据框）对象体已确认前缀与后缀序列的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`（2026-11-13/14 各轮）。字段序来自
//! Map.dll 10.1.0.3035 的 `IPersistStream::Load`（RTTI `.?AVMap@@` →
//! IPersistStream 子接口虚表 `0x794874a4` 槽 +0x14 = `0x793ff4d0`）与对称的
//! Save 主体 `0x79401850`，并与仓库参考样本 `samples/测试底图.mxd` 的 `/Maps`
//! 导出逐字节对照吻合：`/Maps:0x18` 版本 short=51、`0x1a` 名称「新建数据框」、
//! `0x2a` 空名称（`02 00 00 00 00 00`）、`0x30` CRS 对象头（键 30/0x1e）、
//! `0x44` CRS 体版本 6 与 455 字节 WKT。
//!
//! ## 已确认前缀（`read_map_prefix_10_1`；Load 第 1-4 步）
//!
//! 1. int16 架构版本（>0x33/51 直接拒绝；样本值 51）；
//! 2. 字符串 → obj+0x160（数据框名称）；
//! 3. 字符串 → obj+0x164（第二名称，样本为空串）；
//! 4. 嵌套对象 ×1（空间参考，类身份由流内 16 字节对象头决定）：样本为
//!    ProjectedCoordinateSystem；16 零字节的空头按"无空间参考"处理；其他
//!    类标识显式拒绝（对象体无外部长度字段，无法跳过未知类）。
//!
//! 字符串布局已由样本字节闭合：AfCore String::Write 普通输出，4 字节长度
//! 前缀 = 含 UTF-16LE 终止零的字节数 n*2+2。样本「新建数据框」前缀 12
//! （5 字 ×2 + 终止零 2），空名称前缀 2（仅终止零，即文档强调的 6 字节
//! `02 00 00 00 00 00` 模式，勿误读为 int32 计数）。与
//! [`crate::native_string::read_string_10_1`] 的布局一致，名称读取据此
//! 接入该严格读取器。
//!
//! ## 后缀序列（`read_map_suffix_10_1`；Save `0x79401850` 与 Load 对称段）
//!
//! 前缀之后依次：**图层列表**（读侧 `FUN_794035f0` ver≥0x17，见
//! [`crate::layer_list`]）→ 基本图形层对象（SaveObject(+0xf8)，请求 IID
//! `{34B2EF81-…}`；样本为 CompositeGraphicsLayer 键 1553 @/Maps:0x16fba）→
//! int32 计数 + 循环（MapSurround，请求 IID `{FE3115E9-…}`；样本计数 0
//! @/Maps:0x24f60）→ **WKSEnvelope 32 字节**（`[XMin,YMin,XMax,YMax]`，写方向
//! 确定；样本 @0x24f64）→ int32、int32（用途未定性；样本 9、9 @0x24f84）→
//! IEnvelope 对象（请求 IID `{1B077914-…}`；样本为空标记）→ int32 计数 + 循环
//! （书签，请求 IID `{EE53528B-…}`；样本计数 0 @/Maps:0x24f9c）→ **到此为止**。
//! 其后字段（两个 0xFFFF short 等，样本 @0x24fa0 起）与 Map::Load 第 10/11 步
//! 的对位未闭合（文档「未解决事项（第五轮新增）」第 1 条），一律保留为未知
//! 尾部，不读取。
//!
//! 调用约定：调用者先用 [`crate::object_stream::read_header_10_1`] 读取
//! Map 对象头并比对 [`MAP_CLASS_ID`]，从对象头之后的位置调用
//! [`read_map_prefix_10_1`]；本模块不重复读取对象头。后缀读取器同样从对象体
//! 起点接收 Reader（内部重读前缀以登记嵌套空间参考键并定位图层列表），
//! 并要求调用者提供跨流共享的键登记表（Map 对象自身的键由调用者在读取
//! Map 对象头时登记）。
//!
//! 样本核对（2026-11-14，辅助观察）：完整样本在图层列表条目 1（标志 1
//! 新对象）停止，尾部起点 0x28e 与证据文档一致；用样本前缀 + 合成空图层
//! 列表 + 样本 0x24f60..0x24fa0 尾段驱动本读取器，WKSEnvelope 四 double
//! 读出 37392944.284/2520083.285/37394230.529/2520951.835、其后 int32 对
//! (9, 9)、IEnvelope 空标记、书签计数 0、停止 AfterBookmarks，全部与证据
//! 文档一致（尾段核对同时支持 IEnvelope 空标记 16 字节消费的判断，见
//! [`read_map_suffix_10_1`] 文档）。
//!
//! 这是仅覆盖上述已确认字段的严格安全子集：要求完整输入、任何失败不推进
//! 调用者游标、不修改共享登记表。停止点（如标志 1 新对象体、未接入读取器的
//! 新对象体）是成功返回：已闭合部分 + 剩余字节 + 停止原因，与错误区分。
//! 因此不代表完整 Map 体行为，未接入完整 MXD 解析，人工目视验收未做。
//!
//! 未解决问题（详见证据文档「未解决事项」）：WKSEnvelope 之后两个 int32 的
//! 用途未定性；MapSurround/书签循环元素的 coclass 语义开放（样本计数均为 0，
//! 非空循环未验证）；版本 short 比较的符号性未在文档闭合（本读取器按无符号
//! 解释，负值位模式一并拒绝，较有符号原生更严，与 spatial_reference 的版本
//! 处理一致）；IEnvelope 槽位空标记的消费长度见
//! [`read_map_suffix_10_1`] 内注释（样本 16 字节 + 紧随计数 0 与文档计数偏移
//! 0x24f9c 互证；任务摘要曾按 20 字节解读，与样本 0x24fa0=0xffff 矛盾）。

use crate::composite_graphics::{
    COMPOSITE_GRAPHICS_LAYER_CLASS_ID, CompositeGraphicsLayerPrefix10_1,
    read_composite_graphics_layer_prefix_10_1,
};
use crate::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::layer_list::{LayerList10_1, LayerListStop, class_id_hex, read_layer_list_10_1};
use crate::native_string::read_string_10_1;
use crate::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::object_stream::{ObjectHeader, read_header_10_1};
use crate::reader::Reader;
use crate::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use std::io::{self, ErrorKind};

/// Map::Load 接受的最大架构版本：>0x33（51）直接拒绝；样本值 51。
const MAX_ARCHITECTURE_VERSION: u16 = 51;

/// 后缀中 MapSurround / 书签循环单个条目的最小字节数（16 零空标记）。
const MIN_LOOP_ENTRY_BYTES: usize = 16;

/// WKSEnvelope 32 字节原始读取结果（非对象；AfCore 写方向确定的字段序）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WksEnvelope10_1 {
    /// 第 1 个 double（AfCore `?XMin@WKSEnvelope` 读结构 +0x00，写方向确定）。
    pub xmin: f64,
    /// 第 2 个 double（结构 +0x08）。
    pub ymin: f64,
    /// 第 3 个 double（结构 +0x10）。
    pub xmax: f64,
    /// 第 4 个 double（结构 +0x18）。
    pub ymax: f64,
}

/// 后缀中一个对象槽位（基本图形层 / MapSurround 条目 / IEnvelope / 书签条目）
/// 的分类结果与流内偏移。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectSlot10_1 {
    /// 对象头起点偏移（相对传入 Reader 的底层字节）。
    pub offset: usize,
    /// 经 [`crate::object_refs`] 分类的对象条目（空标记 / 回引 / 新对象）。
    pub entry: ObjectEntry10_1,
}

impl ObjectSlot10_1 {
    /// 单行诊断描述：分类、类标识十六进制与键。
    pub fn describe(&self) -> String {
        match &self.entry {
            ObjectEntry10_1::Null => "空标记（16 零类标识）".to_string(),
            ObjectEntry10_1::BackReference { class_id, key } => {
                format!("回引 类={} 键 {key}", class_id_hex(class_id))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                format!("新对象 类={} 键 {key}", class_id_hex(class_id))
            }
        }
    }
}

/// 后缀读取的停止原因（非错误：已消费内容有效，其后保留为未知尾部）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapSuffixStop {
    /// 图层列表在标志 1 新对象体处停止（体无外部长度、读取器未接入；集成点）。
    /// 明细见 [`LayerList10_1::stop`]。
    LayerListFlag1Body {
        /// 图层列表内的条目序号。
        index: usize,
        /// 条目起点（标志处）偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 基本图形层对象为新对象：该槽位对象体无外部长度，无法跳过。类标识为
    /// CompositeGraphicsLayer 时已解析已确认体前缀
    /// （[`MapSuffix10_1::base_graphics_layer_parsed`]），游标停在其停止点
    /// （嵌套新对象处已回退到该对象头起点）；其余情况停在对象头之后。
    BaseGraphicsLayerBody {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// MapSurround 循环条目为新对象：体无外部长度且读取器未接入。
    MapSurroundBody {
        index: usize,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// IEnvelope 槽位对象为新对象且未被 Envelope 读取器整体消费（非
    /// Envelope 类，或 Envelope 体解析失败）：体无外部长度，无法跳过。
    EnvelopeObjectBody {
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 书签循环条目为新对象：体无外部长度且读取器未接入。
    BookmarkBody {
        index: usize,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 书签循环之后的正常终点：其后字段与 Map::Load 第 10/11 步对位未闭合，
    /// 全部保留为未知尾部。
    AfterBookmarks,
}

impl MapSuffixStop {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(self) -> String {
        match self {
            Self::LayerListFlag1Body { index, offset, .. } => format!(
                "图层列表条目 {index}（offset 0x{offset:x}）为标志 1 新对象：体无外部\
                 长度，类分派读取器只覆盖已确认前缀，停止"
            ),
            Self::BaseGraphicsLayerBody { offset, .. } => format!(
                "基本图形层对象（offset 0x{offset:x}）为新对象：体无外部长度；\
                 CompositeGraphicsLayer 已解析体前缀时游标停在其停止点，否则停在\
                 对象头之后"
            ),
            Self::MapSurroundBody { index, offset, .. } => format!(
                "MapSurround 条目 {index}（offset 0x{offset:x}）为新对象：体无外部长度\
                 且元素类没有读取器，停止"
            ),
            Self::EnvelopeObjectBody { offset, .. } => format!(
                "IEnvelope 槽位对象（offset 0x{offset:x}）为新对象且未被 Envelope 读取\
                 器整体消费（非 Envelope 类或解析失败）：体无外部长度，停止"
            ),
            Self::BookmarkBody { index, offset, .. } => format!(
                "书签条目 {index}（offset 0x{offset:x}）为新对象：体无外部长度且读取器\
                 未接入，停止"
            ),
            Self::AfterBookmarks => "书签循环之后字段未闭合（两个 0xFFFF short 等与 \
                Map::Load 第 10/11 步对位未定），到此为止，其后保留为未知尾部"
                .to_string(),
        }
    }
}

/// Map 对象体后缀序列（图层列表起，书签循环止）的解析结果。
///
/// 各阶段字段仅在读取到达时为 `Some`/非空；中途停止时其后阶段全部保持
/// `None`，停止点之后的字节保留在 [`MapSuffix10_1::remaining_bytes`]。
#[derive(Debug, Clone, PartialEq)]
pub struct MapSuffix10_1 {
    /// 图层列表（含条目、活动索引与列表自身的停止信息）。
    pub layer_list: LayerList10_1,
    /// 基本图形层对象槽位（到达时）。
    pub base_graphics_layer: Option<ObjectSlot10_1>,
    /// 基本图形层为 CompositeGraphicsLayer 新对象时的已确认体前缀（读取器
    /// 在首个不可闭合字段处停止，体剩余保留在未知尾部中；解析失败时为
    /// `None`，停止原因仍为
    /// [`MapSuffixStop::BaseGraphicsLayerBody`]）。
    pub base_graphics_layer_parsed: Option<CompositeGraphicsLayerPrefix10_1>,
    /// MapSurround 计数原始值（到达时；负数在读取时已拒绝）。
    pub map_surround_count: Option<i32>,
    /// 已消费的 MapSurround 条目（按流内顺序）。
    pub map_surrounds: Vec<ObjectSlot10_1>,
    /// WKSEnvelope 显示范围（到达时）。
    pub extent: Option<WksEnvelope10_1>,
    /// WKSEnvelope 起点偏移（诊断用）。
    pub extent_offset: Option<usize>,
    /// WKSEnvelope 之后的两个 int32 原始位模式（用途未定性，原样保留）。
    pub raw_int32_pair: Option<(i32, i32)>,
    /// IEnvelope 槽位对象（到达时）。
    pub envelope_object: Option<ObjectSlot10_1>,
    /// IEnvelope 槽位为 Envelope 新对象时的完整对象体（Envelope 读取器覆盖
    /// 全部版本分支，读取成功即整体消费，后缀可继续；解析失败时为 `None`，
    /// 停止原因仍为 [`MapSuffixStop::EnvelopeObjectBody`]）。
    pub envelope_object_parsed: Option<Envelope10_1>,
    /// 书签计数原始值（到达时；负数在读取时已拒绝）。
    pub bookmark_count: Option<i32>,
    /// 已消费的书签条目（按流内顺序）。
    pub bookmarks: Vec<ObjectSlot10_1>,
    /// 停止原因（恒有：书签循环之后也属停止点）。
    pub stop: MapSuffixStop,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// Map 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`。三重印证：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 Map（模块 Map.dll）；
/// 该 CLSID 的字节宿主在 Map.dll `0x79482adc`（GetClassID 引用）；Map.dll
/// RTTI 存在 `.?AVMap@@`。仓库参考样本 `/Maps:0x04..0x13` 首对象头的类
/// 标识与本常量逐字节相同。
pub const MAP_CLASS_ID: [u8; 16] = [
    0x76, 0xaa, 0xbd, 0xe6, 0x35, 0x4d, 0xd0, 0x11, 0x98, 0xbe, 0x00, 0x80, 0x5f, 0x7c, 0xed, 0x21,
];

/// 10.1 Map::Load（`0x793ff4d0`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MapPrefix10_1 {
    /// 2 字节架构版本 short 的无符号位模式，>51 的位模式一律拒绝。
    ///
    /// 证据：Load `0x793ff4d0` 开头 `AfStreamRead(2)` 后对 >0x33（51）直接
    /// 拒绝，样本值 51。反编译比较的符号性未闭合：按无符号解释接受 0..=51；
    /// 若原生为有符号比较，负值位模式（0x8000..）可能通过原生检查，此处
    /// 一并拒绝（较原生更严，与 spatial_reference 的版本处理一致）。
    pub version: u16,
    /// 数据框名称（Load 第 2 步，写入 obj+0x160；样本「新建数据框」）。
    pub name: String,
    /// 第二名称字段（Load 第 3 步，写入 obj+0x164；样本为空串）。
    pub secondary_name: String,
    /// Load 第 4 步的嵌套空间参考对象体。
    ///
    /// 16 零字节空头 → `None`（沿用对象头的 S_FALSE 约定，仅消耗 16 字节、
    /// 不读键）；类标识等于 PCS 常量 → 已解析对象体；其他类标识 →
    /// [`read_map_prefix_10_1`] 直接报错。对象头键值原样丢弃，未保留。
    pub spatial_reference: Option<ProjectedCoordinateSystem10_1>,
    /// 已确认前缀之后剩余的字节数。
    ///
    /// 对应 Load 第 5 步起的字段：其中「图层列表 → … → 书签循环」一段可由
    /// [`read_map_suffix_10_1`] 继续读取，其余（书签循环之后）仍为未闭合字段。
    /// 本前缀读取器不读取、不解释。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 Map 对象体的已确认前缀，失败时不推进游标。
///
/// 起点约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取
/// Map 对象头、比对 [`MAP_CLASS_ID`] 确认类身份，并从对象头之后的位置传
/// 入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：应用于两个名称字符串的声明长度（含终止零、
/// 不含 4 字节前缀），并原样转发给嵌套投影坐标系对象体（约束 WKT 与域字
/// 符串长度）；限额在任何读取与分配之前检查，`usize::MAX` 表示不额外设
/// 限，此时超长流按缺短读失败。
///
/// 嵌套空间参考对象的类身份由流内对象头决定：全零 16 字节为空头
/// （`spatial_reference = None`），类标识等于
/// [`PROJECTED_COORDINATE_SYSTEM_CLASS_ID`] 时用
/// [`read_projected_coordinate_system_10_1`] 读取对象体；其他类标识返回
/// [`ErrorKind::Unsupported`]——嵌套对象体没有外部长度字段，本读取器无法
/// 跳过未知类继续解析，调用者不能把该流继续当作 Map 前缀消费。
///
/// 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进调用
/// 者游标。错误位置相对于传入 Reader，调用方需另外记录流和对象身份。这是
/// 严格安全子集，不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_map_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<MapPrefix10_1> {
    // 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进原游标。
    let mut trial = reader.clone();
    let (result, _spatial_reference_key) = read_map_prefix_inner(&mut trial, max_bytes)?;
    *reader = trial;
    Ok(result)
}

/// 前缀读取的内部实现：额外返回嵌套空间参考对象的键（空标记为 `None`）。
///
/// 键不参与前缀的公开行为（保持 [`read_map_prefix_10_1`] 的签名与语义不变），
/// 仅供 [`read_map_suffix_10_1`] 登记进共享键表——流内后续对象对 Map 空间
/// 参考的引用（如 Envelope 内的回引）按键命中，样本中 Map 的 PCS 键 30 被
/// 多个 Envelope 回引（证据文档「Envelope 对象」节）。
fn read_map_prefix_inner(
    trial: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<(MapPrefix10_1, Option<u32>)> {
    let entry_offset = trial.position();

    // 1. 2 字节架构版本：无符号解释，>51 直接拒绝（原生 >0x33 拒绝）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 对象体 offset 0x{entry_offset:x}: 架构版本 {version} 超出 \
                 Map::Load 接受范围（>0x33/51 直接拒绝）"
            ),
        ));
    }

    // 2/3. 两个名称字符串：AfCore String 布局（样本字节闭合，见模块文档），
    // 各自受 max_bytes 限额约束。
    let name = read_string_10_1(trial, max_bytes)?;
    let secondary_name = read_string_10_1(trial, max_bytes)?;

    // 4. 嵌套空间参考对象：先读对象头（类身份完全由流内对象头决定）。
    let nested_offset = trial.position();
    let mut spatial_reference_key = None;
    let spatial_reference = match read_header_10_1(trial)? {
        // 全零 16 字节：沿用对象头读取器的 S_FALSE 约定，仅消耗对象头本身、
        // 不读键；语义上表示该 Map 没有空间参考对象。
        ObjectHeader::Null => None,
        ObjectHeader::Object { class_id, key }
            if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // PCS 对象体由专门读取器解析，max_bytes 原样转发；键原样返回
            // 供调用者登记（本函数不修改登记表）。
            spatial_reference_key = Some(key);
            Some(read_projected_coordinate_system_10_1(trial, max_bytes)?)
        }
        ObjectHeader::Object { class_id, key } => {
            // 严格子集只认投影坐标系嵌套。嵌套对象体无外部长度字段，无法
            // 跳过未知类继续读取；显式报错，原游标不推进。
            let id = class_id_hex(&class_id);
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Map 对象体嵌套对象 offset 0x{nested_offset:x}: 类标识 {id}（键 \
                     {key}）不是投影坐标系；本读取器仅支持投影坐标系嵌套，且对象体 \
                     无外部长度字段，无法跳过未知类继续读取"
                ),
            ));
        }
    };

    Ok((
        MapPrefix10_1 {
            version,
            name,
            secondary_name,
            spatial_reference,
            remaining_bytes: trial.remaining(),
        },
        spatial_reference_key,
    ))
}

/// 从 Map 对象体起点读取图层列表起、书签循环止的后缀序列，失败时不推进
/// 游标、不修改共享登记表。
///
/// 起点约定：调用者已读取 Map 对象头并把 Reader 定位在对象体起点（2 字节
/// 架构版本处）——与 [`read_map_prefix_10_1`] 的起点相同；本函数先按同一
/// 规则重读已确认前缀（用于把嵌套空间参考对象的键登记进共享表并定位图层
/// 列表起点，前缀字段不再重复报告），再继续读取后缀序列。`table` 是跨流
/// 共享的会话键登记表：本函数在 trial 副本上登记（含前缀嵌套 PCS 键与后缀
/// 全部新对象键），仅在成功返回（含停止点）时提交；任何错误路径不推进游标、
/// 不改动调用者的登记表。Map 对象自身的键由调用者在读取 Map 对象头时登记，
/// 本函数从对象体起点开始、看不到对象头。
///
/// 各阶段停止规则（均为成功返回 + [`MapSuffixStop`]，剩余字节保留）：
///
/// - 图层列表标志 1 新对象体（无外部长度；集成点）；
/// - 基本图形层、MapSurround 条目、IEnvelope 槽位、书签条目的**新对象**体
///   （普通 `LoadObject` 槽位无 ExtraSafe 前缀，体无外部长度；基本图形层与
///   图层体同样预留集成点）。空标记与回引条目只消耗对象头，继续；
/// - 书签循环之后的字段未闭合，正常终点即停止（[`MapSuffixStop::AfterBookmarks`]）。
///
/// IEnvelope 槽位的空标记消费 **16 字节**（沿用 [`crate::object_stream`]
/// S_FALSE 约定）。注意与任务摘要的 20 字节解读不同：样本 `/Maps:0x24f8c` 起
/// 有 20 个零字节，其后 `0x24fa0` 为 `ff ff`；若空标记消费 20 字节，书签计数
/// 将位于 0x24fa0 且读出 0xffff（65535 个书签），与样本无书签及证据文档
/// 「`0x24f9c` int32=0（书签计数）」矛盾。故按 16 字节 + 计数位于 0x24f9c
/// 消费；「SaveObject 空标记是否写 4 字节键 0」仍列为开放问题（前缀嵌套
/// 空头的同项开放问题见模块文档）。
///
/// `max_bytes` 语义与前缀相同（名称字符串与嵌套 PCS 体），并预检图层列表、
/// MapSurround、书签三个循环的「声明计数 × 最小条目字节数」。所有偏移相对
/// 传入 Reader 的底层字节。这是严格安全子集，不代表完整原生 Load 行为。
pub fn read_map_suffix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MapSuffix10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 0. 重读已确认前缀：登记嵌套 PCS 键（新键，样本键 30 首现于此），
    //    前缀字段本身不重复报告。
    let (_prefix, spatial_reference_key) = read_map_prefix_inner(&mut trial, max_bytes)?;
    if let Some(key) = spatial_reference_key {
        trial_table.register(key);
    }

    // 1. 图层列表（ver>=0x17 分支；样本 ver=51）。
    let layer_list = read_layer_list_10_1(&mut trial, max_bytes, &mut trial_table)?;
    // 图层列表的停止原因目前仅有标志 1 新对象体一种（LayerListStop 单变体）。
    let mut stop = layer_list.stop.map(
        |LayerListStop::Flag1NewObjectBody {
             index,
             offset,
             class_id,
             key,
         }| MapSuffixStop::LayerListFlag1Body {
            index,
            offset,
            class_id,
            key,
        },
    );

    let mut base_graphics_layer = None;
    let mut base_graphics_layer_parsed = None;
    let mut map_surround_count = None;
    let mut map_surrounds = Vec::new();
    let mut extent = None;
    let mut extent_offset = None;
    let mut raw_int32_pair = None;
    let mut envelope_object = None;
    let mut envelope_object_parsed = None;
    let mut bookmark_count = None;
    let mut bookmarks = Vec::new();

    if stop.is_none() {
        // 2. 基本图形层对象（SaveObject(+0xf8)，请求 IID {34B2EF81-…}）。
        //    类标识为 CompositeGraphicsLayer 时解析已确认体前缀（样本即此
        //    情况，键 1553）：读取器在首个不可闭合字段处停止并把游标回退到
        //    该字段起点。该槽位对象体无外部长度，后缀仍在此停止，剩余保留
        //    为未知尾部；其余类（或前缀解析失败）不消费体、停在对象头之后。
        let offset = trial.position();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        if let ObjectEntry10_1::NewObject { class_id, .. } = entry {
            if class_id == COMPOSITE_GRAPHICS_LAYER_CLASS_ID {
                base_graphics_layer_parsed = read_composite_graphics_layer_prefix_10_1(
                    &mut trial,
                    max_bytes,
                    &mut trial_table,
                )
                .ok();
            }
        }
        base_graphics_layer = Some(ObjectSlot10_1 { offset, entry });
        if let ObjectEntry10_1::NewObject { class_id, key } = entry {
            stop = Some(MapSuffixStop::BaseGraphicsLayerBody {
                offset,
                class_id,
                key,
            });
        }
    }

    if stop.is_none() {
        // 3. MapSurround 计数 + 循环（请求 IID {FE3115E9-…}；样本计数 0）。
        let count = read_loop_count(&mut trial, "MapSurround", max_bytes)?;
        map_surround_count = Some(count);
        for index in 0..usize::try_from(count).expect("计数已确认为非负") {
            let offset = trial.position();
            let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
            map_surrounds.push(ObjectSlot10_1 { offset, entry });
            if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                stop = Some(MapSuffixStop::MapSurroundBody {
                    index,
                    offset,
                    class_id,
                    key,
                });
                break;
            }
        }
    }

    if stop.is_none() {
        // 4. WKSEnvelope 32 字节（非对象；[XMin,YMin,XMax,YMax]，写方向确定）。
        let offset = trial.position();
        let mut coords = [0_f64; 4];
        for slot in &mut coords {
            *slot = trial.read_f64_le()?;
        }
        extent_offset = Some(offset);
        extent = Some(WksEnvelope10_1 {
            xmin: coords[0],
            ymin: coords[1],
            xmax: coords[2],
            ymax: coords[3],
        });

        // 5. 两个 int32 原样（Load 第 7 步写入子接口 +0x38/+0x40，用途未定性）。
        let first = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let second = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        raw_int32_pair = Some((first, second));
    }

    if stop.is_none() {
        // 6. IEnvelope 槽位对象（请求 IID {1B077914-…}；样本为空标记，消费
        //    长度见函数文档）。类标识为 Envelope 时用其读取器整体消费对象
        //    体（Envelope 全部版本分支已闭合）：读取成功则后缀继续到书签
        //    循环；其余类或解析失败在该槽位停止。
        let offset = trial.position();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        let mut envelope_consumed = false;
        if let ObjectEntry10_1::NewObject { class_id, .. } = entry {
            if class_id == ENVELOPE_CLASS_ID {
                match read_envelope_10_1(&mut trial, max_bytes, &mut trial_table) {
                    Ok(parsed) => {
                        envelope_object_parsed = Some(parsed);
                        envelope_consumed = true;
                    }
                    Err(_) => {
                        // 解析失败（截断、嵌套 SR 非 PCS 等）：在该槽位停止，
                        // 失败细节由调用者用 Envelope 读取器重放诊断。
                    }
                }
            }
        }
        envelope_object = Some(ObjectSlot10_1 { offset, entry });
        if !envelope_consumed {
            if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                stop = Some(MapSuffixStop::EnvelopeObjectBody {
                    offset,
                    class_id,
                    key,
                });
            }
        }
    }

    if stop.is_none() {
        // 7. 书签计数 + 循环（请求 IID {EE53528B-…}；样本计数 0）。
        let count = read_loop_count(&mut trial, "书签", max_bytes)?;
        bookmark_count = Some(count);
        for index in 0..usize::try_from(count).expect("计数已确认为非负") {
            let offset = trial.position();
            let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
            bookmarks.push(ObjectSlot10_1 { offset, entry });
            if let ObjectEntry10_1::NewObject { class_id, key } = entry {
                stop = Some(MapSuffixStop::BookmarkBody {
                    index,
                    offset,
                    class_id,
                    key,
                });
                break;
            }
        }
    }

    // 8. 书签循环之后字段未闭合：正常终点也是停止点，剩余保留为未知尾部。
    let stop = stop.unwrap_or(MapSuffixStop::AfterBookmarks);

    let result = MapSuffix10_1 {
        layer_list,
        base_graphics_layer,
        base_graphics_layer_parsed,
        map_surround_count,
        map_surrounds,
        extent,
        extent_offset,
        raw_int32_pair,
        envelope_object,
        envelope_object_parsed,
        bookmark_count,
        bookmarks,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 读取后缀循环（MapSurround / 书签）的 int32 计数：拒绝负数并预检限额。
///
/// 原生读取仅按有符号比较决定是否进入循环（写侧计数非负）；负数语义未知，
/// 这里显式拒绝，与 maps_stream 的数量处理保持同一严格程度。
fn read_loop_count(trial: &mut Reader<'_>, label: &str, max_bytes: usize) -> io::Result<i32> {
    let offset = trial.position();
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 后缀 {label} 计数 offset 0x{offset:x}: 计数为负数（{count}）；原生 \
                 读取仅正数进入循环，负数语义未知，本读取器拒绝"
            ),
        ));
    }
    let count_usize = usize::try_from(count).expect("计数已确认为非负");
    let min_total = count_usize
        .checked_mul(MIN_LOOP_ENTRY_BYTES)
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("Map 后缀 {label} 计数 offset 0x{offset:x}: 计数 × 最小条目字节数溢出"),
            )
        })?;
    if min_total > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 后缀 {label} 计数 offset 0x{offset:x}: 声明计数 {count} × 最小条目 \
                 字节数 {MIN_LOOP_ENTRY_BYTES} 超过资源限额 {max_bytes}，在条目读取之前拒绝"
            ),
        ));
    }
    Ok(count)
}
