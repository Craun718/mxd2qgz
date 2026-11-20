//! ArcMap 10.1 GroupLayer（图层组）对象体已确认前缀的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`「字段序闭合补充（2026-11-13
//! 第二轮）」的 GroupLayer::Load 完整字段表、「Map::Save 方向、ExtraSafe
//! 前缀与图层列表样本闭合（2026-11-14 第四轮）」的 ExtraSafeReadObject
//! 节、「图层树、可见性与数据源连接」节及各轮「未解决事项」。字段序与
//! 分支条件以 FeatureLayer.dll 10.1.0.3035 的 `GroupLayer::Load`
//! （`0x6f711270`，IPersistStream 虚表 `0x6f7af244` +0x14；RTTI
//! `.?AVGroupLayer@@`，TD `0x6f7d3458`）反编译为准（研究产物
//! `/tmp/r3/grouplayer_load.c`，见证据文档「证据与复现」节）；本模块实现
//! 时逐行复核了该反编译，与文档字段表不同的分支条件在下方逐处标注。
//!
//! 已确认字段序（编号 = 流内顺序）：
//!
//! 1. int16 版本：原生以有符号 short 比较 `0xc < ver` 拒绝（0x80004005），
//!    上限 12。反编译的符号性未在文档闭合，本读取器按无符号解释，负值位
//!    模式（0x8000..）一并拒绝，较原生更严（与 map_object、
//!    spatial_reference 的版本处理一致）；
//! 2. AfCore String 名称 → obj+0x9c（[`crate::stream::native_string::read_string_10_1`]）；
//! 3. int16 ×3 → +0xa0/+0xa2/+0xa4（+0xa0 为可见性「候选」，语义未定，
//!    原样保留）；
//! 4. double ×2 → +0xa8/+0xb0；
//! 5. int32 子图层计数（原生按有符号比较，<=0 不迭代）+ 每项一条：
//!    版本 <11 为普通嵌套对象（16 字节类标识 + 4 字节键 + 体）；版本 >=11
//!    每项先读 8 字节 ExtraSafe 前缀 `[A:int32][B:int32]`（AfCore
//!    `ExtraSafeReadObject 0x68a7be60` 反编译）：`A==0 && B==0` 为空条目，
//!    否则仍是普通嵌套对象布局，B!=0 时原生 Seek 到「前缀后位置 + B」强
//!    同步。读侧反编译未见原生消费 A；写侧观察值 = 对象头 20 字节 + 体的
//!    总体积（`ExtraSafeWriteObject 0x68a7bd70`，/Maps 图层列表逐条吻合，
//!    第四轮），本读取器据此把 A 用作新对象体的有界跳过；
//! 6. int16 可见性 → +0xbc（VARIANT_BOOL，与 FeatureLayer +0x128 同构，
//!    「确定的」）：版本 <2 不读取（原生默认 0xffff=可见），>=2 读取；
//! 7. 版本 >2：嵌套对象（请求 IID {1B077914-38C5-11D0-92D2-00805F7C28B0}，
//!    第五轮类型库考证为 IEnvelope）。[文档字段表标注 >=2，反编译为
//!    `2 < ver`，以反编译为准]；
//! 8. 版本 >=4：double → +0xc8；
//! 9. 版本 >=5：LayerLib `ExtensionHelper::LoadExtensions`；
//! 10. 版本 >=6 读 String（版本 6 再读 short）；>=7 读 short；>=8 读
//!     double ×2；>=9 读 short；>=10 读 short ×3；版本 12 读 IDisplayFilter。
//!     随后的接口释放和遮罩初始化不消费流。
//!
//! 普通关联槽需要完整消费，使用与 3D 图层共用的无界分派。ExtraSafe 子层在
//! 条目边界内读完才提交；未知子体会使整个普通关联对象回滚。旧的前缀入口仍
//! 提供部分解析信息，用于外层有界图层列表的诊断和保留。
//!
//! ## 子图层递归（2026-11-14 第六轮接入）
//!
//! 子图层新对象体经 [`crate::layers::layer_list::parse_body_prefix`] 类分派解析已
//! 确认前缀（FeatureLayer / GroupLayer / CompositeGraphicsLayer /
//! FDOGraphicsLayer；组图层套组图层经同一分派递归，深度限额见
//! [`MAX_RECURSION_DEPTH`]，非原生限制）。版本 >=11 的 ExtraSafe 条目解析
//! 前缀后把剩余按 A 界跳过、循环继续；版本 <11 的普通条目解析成功时游标
//! 停在子图层体的前缀停止点并停止循环（体剩余无外部长度，见
//! [`GroupLayerStop10_1::ChildLayerBody`]），类无读取器或解析失败时回退到
//! 该对象头起点停止（[`GroupLayerStop10_1::ChildNewObject`]）。证据范围：
//! 参考样本无组图层，子图层条目的分派行为无样本字节佐证，仅 ExtraSafe
//! `[A][B]` 布局有 /Maps 图层列表的逐条目吻合（第四轮）。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`GROUP_LAYER_CLASS_ID`]，从对象头之后的位置调用
//! [`read_group_layer_prefix_10_1`]；本模块不重复读取对象头。
//!
//! 仓库参考工程无组图层。SDK 的 brazil.mxd 与 SchematicsForServerSample.mxd
//! 提供版本 11/12 子层包装的字节；brazil.mxd 首个版本 11 体已核对完整边界。
//! 版本 <11 的真实对象与 ArcMap/QGIS 验收仍待验证。

use super::cache_tile_reader::is_tiled_layer;
use super::wcs_layer::{WCS_LAYER_CLASS_ID, read_wcs_layer_body_10_1};
use crate::layers::feature_layer_tail::{
    DisplayFilterSlot10_1, RefSlot10_1, TRANSPARENCY_DISPLAY_FILTER_CLASS_ID,
    read_transparency_display_filter_10_1,
};
use crate::layers::layer_extensions::{LayerExtensionEntry10_1, read_layer_extensions_10_1};
use crate::layers::layer_list::{
    LayerBodyPrefix10_1, RASTER_LAYER_CLASS_ID, read_complete_layer_body_10_1,
};
use crate::layers::raster_layer::read_complete_bounded_raster_body_10_1;
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// GroupLayer::Load 接受的最大架构版本：>12（0x0c）原生拒绝 0x80004005。
const MAX_ARCHITECTURE_VERSION: u16 = 12;

/// 嵌套对象头体积：16 字节类标识 + 4 字节键。
const OBJECT_HEADER_BYTES: u32 = 20;

/// 子图层递归解析的最大深度（Rust 侧防栈溢出的安全限额，非原生限制：原生
/// 按 C++ 递归无显式上限；每层至少消费 22 字节，超深输入在本读取器内按未
/// 知体跳过/停止处理）。
pub(crate) const MAX_RECURSION_DEPTH: u32 = 64;

/// GroupLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{EDAD6647-1810-11D1-86AE-0000F8751720}`。证据：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 GroupLayer（模块
/// FeatureLayer.dll）；FeatureLayer.dll RTTI 存在 `.?AVGroupLayer@@`
/// （TD `0x6f7d3458`），其 IPersistStream::Load 即 `0x6f711270`（文档类清
/// 单「确定的」）。参考样本无组图层，无样本头佐证。
pub const GROUP_LAYER_CLASS_ID: [u8; 16] = [
    0x47, 0x66, 0xad, 0xed, 0x10, 0x18, 0xd1, 0x11, 0x86, 0xae, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// 已消费的嵌套对象条目（新对象是停止点，不出现在记录中）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumedEntry10_1 {
    /// 16 字节全零类标识的空引用：无键、无对象体。
    Null,
    /// 键已在此前登记：回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// 子图层条目的分类记录（字段序第 5 步的循环）。
#[derive(Debug, Clone, PartialEq)]
pub enum GroupChildEntry10_1 {
    /// ExtraSafe 前缀 `[A=0][B=0]` 的空条目（仅版本 >=11 出现）：原生
    /// `ExtraSafeReadObject` 返回 1，调用方跳过。
    Empty,
    /// 16 字节全零类标识的空引用。
    Null,
    /// 回引条目：无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 版本 >=11 ExtraSafe 条目内的新对象：头已分类，体先经类分派解析已确
    /// 认前缀（`parsed`），再把剩余按 `A-20` 字节未知体范围跳过（读侧原生
    /// 不消费 A，A 的体积语义是写侧观察，见模块文档；类无读取器或解析失败
    /// 时 `parsed` 为 `None`、整个体按未知跳过）。
    SkippedNewObject {
        class_id: [u8; 16],
        key: u32,
        /// A 前缀的原始值（写侧观察 = 头 20 字节 + 体的总体积）。
        declared_total: u32,
        /// 未知体范围在传入 Reader 窗口内的起始偏移。
        body_offset: usize,
        /// 未知体范围的字节长度（= A - 20 减去已解析前缀覆盖的部分）。
        body_length: usize,
        /// 经类分派解析出的体前缀（2026-11-14 第六轮接入递归）。
        parsed: Option<Box<LayerBodyPrefix10_1>>,
    },
    /// 版本 <11 普通条目的新对象经类分派解析出前缀：子图层体剩余无外部长
    /// 度，读取器停在其前缀停止点（循环无法继续，见停止原因
    /// [`GroupLayerStop10_1::ChildLayerBody`]）。
    ParsedNewObject {
        class_id: [u8; 16],
        key: u32,
        parsed: Box<LayerBodyPrefix10_1>,
    },
}

/// 已确认前缀的停止原因；本读取器对任何成功输入都返回停止原因（见模块文档）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupLayerStop10_1 {
    /// 版本 <11 的子图层循环中出现新对象且无对应读取器或前缀解析失败：其
    /// 体没有外部长度字段，本读取器无法消费。`index` 为条目序号（0 起）；
    /// 游标与登记表回退到该对象头起点；`note` 为解析失败时的原因（无读取
    /// 器时为 `None`）。
    ChildNewObject {
        index: usize,
        class_id: [u8; 16],
        key: u32,
        note: Option<String>,
    },
    /// 版本 <11 的子图层新对象经类分派解析出前缀：子图层体剩余无外部长度，
    /// 循环无法继续，读取器停在其前缀停止点。
    ChildLayerBody { index: usize },
    /// 版本 >2 的嵌套对象（IEnvelope 候选）是新对象：停止并回退到其头起点。
    NestedNewObject { class_id: [u8; 16], key: u32 },
    /// 版本所需的全部对象体字段已消费。
    Complete,
    /// 前缀入口停在版本 >=5 的扩展列表前。
    ExtensionHelperNotExpanded,
    /// 前缀入口的旧版终点（版本 <5）。
    TailNotExpanded,
}

impl GroupLayerStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(&self) -> String {
        match self {
            Self::ChildNewObject { index, note, .. } => match note {
                None => format!(
                    "子图层条目 {index} 为新对象且无对应读取器：其体没有外部长度字段，\
                     无法消费，游标回退到该对象头起点"
                ),
                Some(note) => format!(
                    "子图层条目 {index} 的新对象前缀解析失败（{note}），体按未知保留，\
                     游标回退到该对象头起点"
                ),
            },
            Self::ChildLayerBody { index } => format!(
                "子图层条目 {index} 的新对象经类分派解析出前缀：子图层体剩余无外部\
                 长度，循环无法继续，读取器停在其前缀停止点"
            ),
            Self::NestedNewObject { .. } => {
                "版本 >2 的嵌套对象（IEnvelope 候选）为新对象：停止并回退到其头起点".to_string()
            }
            Self::Complete => "GroupLayer 对象体已完整消费".into(),
            Self::ExtensionHelperNotExpanded => "前缀读取停在版本 >=5 的扩展列表前".to_string(),
            Self::TailNotExpanded => "前缀读取到达版本 <5 的末尾".to_string(),
        }
    }
}

/// 10.1 GroupLayer::Load 在版本 5 起读取的尾段。
#[derive(Debug, Clone, PartialEq)]
pub struct GroupLayerTail10_1 {
    pub extensions: Vec<LayerExtensionEntry10_1>,
    pub string_dc: Option<String>,
    pub legacy_v6_short: Option<u16>,
    pub short_d6: Option<u16>,
    pub doubles_e0_e8: Option<[f64; 2]>,
    pub short_d8: Option<u16>,
    pub v10_shorts: Option<[u16; 3]>,
    pub display_filter: Option<DisplayFilterSlot10_1>,
}

/// 10.1 GroupLayer::Load（`0x6f711270`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct GroupLayerPrefix10_1 {
    /// 2 字节版本 short 的无符号位模式，>12 一律拒绝。
    pub version: u16,
    /// 名称（→ obj+0x9c）。
    pub name: String,
    /// 名称后的三个 short（→ +0xa0/+0xa2/+0xa4），语义未定性、原样保留；
    /// +0xa0 是可见性「候选」。
    pub shorts_after_name: [u16; 3],
    /// 两个 double（→ +0xa8/+0xb0），原样保留。
    pub doubles: [f64; 2],
    /// 子图层计数的原始位模式；原生按有符号比较，<=0 不迭代。
    pub child_count: u32,
    /// 已成功分类的子图层条目；触发 [`GroupLayerStop10_1::ChildNewObject`]
    /// 停止的条目不记录。
    pub children: Vec<GroupChildEntry10_1>,
    /// 可见性（→ +0xbc，VARIANT_BOOL 位模式）。`None` = 版本 <2 未读取
    /// （原生默认 0xffff=可见），或在可见性之前已停止。
    pub visibility: Option<u16>,
    /// 版本 >2 的嵌套对象分类（请求 IID {1B077914-…}=IEnvelope 候选）。
    /// `None` = 版本条件未满足，或该对象是新对象（停止原因
    /// [`GroupLayerStop10_1::NestedNewObject`]）。
    pub nested_after_visibility: Option<ConsumedEntry10_1>,
    /// 新 Envelope 对象的键和对象体，由完整体入口读取。
    pub envelope: Option<(u32, Box<Envelope10_1>)>,
    /// 版本 >=4 的 double（→ +0xc8）。`None` = 版本条件未满足或已停止。
    pub double_after_nested: Option<f64>,
    pub tail: Option<GroupLayerTail10_1>,
    /// 完整体读取达到版本所需终点时为 Complete。
    pub stop: GroupLayerStop10_1,
    /// 停止位置之后的剩余字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 GroupLayer 对象体的已确认前缀。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`GROUP_LAYER_CLASS_ID`] 确认类身份，并从对象头之后的位置传入
/// Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额，应用于名称字符串的声明长度（含终止零、不含
/// 4 字节前缀），在任何读取与分配之前检查；`usize::MAX` 表示不额外设限。
/// `table` 是跨流共享的键登记表：新对象键在消费其体（或按 ExtraSafe 的
/// A 前缀跳过其体）前登记；回引判定只依赖登记表状态。
///
/// 事务性：全部字段在 trial 游标与 trial 登记表上读取，成功返回才提交；
/// 任何失败都不推进调用者游标、不修改登记表。触发停止的返回是成功：游标
/// 推进到停止点（新对象停止时回退到该对象头起点，调用者可把剩余字节交给
/// 对应的对象体读取器），已消费条目登记进调用者登记表。错误位置相对于传
/// 入 Reader，调用方需另外记录流和对象身份。这是严格安全子集，不代表完
/// 整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_group_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GroupLayerPrefix10_1> {
    read_group_layer_prefix_with_depth_10_1(reader, max_bytes, table, 0)
}

/// 带递归深度的内部实现：深度超过 [`MAX_RECURSION_DEPTH`] 时子图层新对象
/// 不再做类分派（按未知体处理），防栈溢出（非原生限制，见常量文档）。
pub(crate) fn read_group_layer_prefix_with_depth_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: u32,
) -> io::Result<GroupLayerPrefix10_1> {
    read_group_layer_10_1(reader, max_bytes, table, depth, false)
}

pub(super) fn read_group_layer_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<GroupLayerPrefix10_1> {
    read_group_layer_10_1(reader, max_bytes, table, 0, true)
}

fn read_group_layer_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: u32,
    complete: bool,
) -> io::Result<GroupLayerPrefix10_1> {
    let entry_offset = reader.position();
    // 入口克隆：Reader 没有随机定位原语，B 同步与停止回退都用「入口克隆 +
    // 前向读取」把游标重定位到窗口内绝对偏移；所有目标都在入口之后。
    let origin = reader.clone();
    let limit = entry_offset + origin.remaining();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("GroupLayer 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    // 全部字段在 trial 游标与 trial 登记表上读取，成功才提交；任何失败路径
    // 都不推进原游标、不改原登记表。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 2 字节版本：无符号解释，>12 拒绝（原生 0xc < ver 记 0x80004005）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(invalid(format!(
            "架构版本 {version} 超出 GroupLayer::Load 接受范围（>12 直接拒绝）"
        )));
    }

    // 2. 名称：AfCore String 布局，受 max_bytes 限额约束。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    // 3. 三个 short：语义未定性，原样保留。
    let mut shorts_after_name = [0_u16; 3];
    for slot in &mut shorts_after_name {
        *slot = trial.read_u16_le()?;
    }

    // 4. 两个 double：原样保留。
    let mut doubles = [0.0; 2];
    for slot in &mut doubles {
        *slot = trial.read_f64_le()?;
    }

    // 5. 子图层计数 + 循环。原生按有符号比较，<=0 不迭代；计数位模式原样保留。
    let child_count = trial.read_u32_le()?;
    let mut children = Vec::new();
    let mut stop: Option<GroupLayerStop10_1> = None;
    if (child_count as i32) > 0 {
        'children: for index in 0..child_count as i32 as usize {
            if version >= 11 {
                // 版本 >=11：ExtraSafeReadObject 条目，先读 8 字节 [A][B] 前缀。
                let prefix_offset = trial.position();
                let a = trial.read_u32_le()?;
                let b = trial.read_u32_le()?;
                if a == 0 && b == 0 {
                    // 空条目：原生返回 1、调用方跳过，无对象头。
                    children.push(GroupChildEntry10_1::Empty);
                    continue;
                }
                let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                match entry {
                    ObjectEntry10_1::Null => {
                        children.push(GroupChildEntry10_1::Null);
                    }
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        children.push(GroupChildEntry10_1::BackReference { class_id, key });
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        let bounded_body = is_tiled_layer(class_id)
                            || matches!(
                                class_id,
                                crate::layers::cad_layer::CAD_LAYER_CLASS_ID
                                    | crate::layers::cad_layer::CAD_FEATURE_LAYER_CLASS_ID
                                    | crate::layers::cad_layer::CAD_ANNOTATION_LAYER_CLASS_ID
                                    | crate::layers::coverage_annotation_layer::COVERAGE_ANNOTATION_LAYER_CLASS_ID
                            );
                        let exact_boundary = complete || bounded_body;
                        // A 的体积语义是写侧观察（读侧原生不消费 A）：A>=20 时
                        // 体按「前缀终点到 A 界」有界处理；A<20 与写侧不变量
                        // 矛盾，按本读取器报错。
                        if a < OBJECT_HEADER_BYTES && (!exact_boundary || b == 0) {
                            return Err(invalid(format!(
                                "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 A={a} \
                                 小于对象头 {OBJECT_HEADER_BYTES} 字节，无法用作有界跳过"
                            )));
                        }
                        let body_offset = trial.position();
                        let full_body_length = if exact_boundary && b != 0 {
                            (b as i32)
                                .checked_sub(OBJECT_HEADER_BYTES as i32)
                                .and_then(|n| usize::try_from(n).ok())
                                .ok_or_else(|| {
                                    invalid(format!("子图层同步边界 B={b} 小于对象头"))
                                })?
                        } else {
                            (a - OBJECT_HEADER_BYTES) as usize
                        };
                        // 类分派递归（第六轮接入）：体前缀先解析（组图层套组
                        // 图层经同一分派递归，深度受 MAX_RECURSION_DEPTH 限
                        // 制），再把剩余按 A 界跳过；解析失败或类无读取器时
                        // 整个体按未知跳过。
                        let mut parsed = None;
                        if complete {
                            let mut region = trial.read_region(full_body_length)?;
                            let body = if class_id == RASTER_LAYER_CLASS_ID {
                                LayerBodyPrefix10_1::RasterLayer(
                                    read_complete_bounded_raster_body_10_1(
                                        &mut region,
                                        max_bytes,
                                        &mut trial_table,
                                    )?,
                                )
                            } else if class_id == WCS_LAYER_CLASS_ID {
                                LayerBodyPrefix10_1::WcsLayer(Box::new(read_wcs_layer_body_10_1(
                                    &mut region,
                                    max_bytes,
                                    &mut trial_table,
                                    true,
                                )?))
                            } else {
                                read_complete_layer_body_10_1(
                                    &mut region,
                                    class_id,
                                    max_bytes,
                                    &mut trial_table,
                                )?
                            };
                            if region.remaining() != 0 {
                                return Err(invalid("ExtraSafe 子图层体未完整消费声明范围".into()));
                            }
                            parsed = Some(Box::new(body));
                        } else if bounded_body {
                            let mut source = trial.clone();
                            let mut region = source.read_region(full_body_length)?;
                            let mut refs = trial_table.clone();
                            if let Ok(body) = read_complete_layer_body_10_1(
                                &mut region,
                                class_id,
                                max_bytes,
                                &mut refs,
                            ) && region.remaining() == 0
                            {
                                trial = source;
                                trial_table = refs;
                                parsed = Some(Box::new(body));
                            }
                        } else if class_id == WCS_LAYER_CLASS_ID {
                            let mut source = trial.clone();
                            let mut region = source.read_region(full_body_length)?;
                            if let Ok(body) = read_wcs_layer_body_10_1(
                                &mut region,
                                max_bytes,
                                &mut trial_table,
                                true,
                            ) {
                                trial = source;
                                parsed =
                                    Some(Box::new(LayerBodyPrefix10_1::WcsLayer(Box::new(body))));
                            }
                        } else if depth < MAX_RECURSION_DEPTH {
                            match crate::layers::layer_list::parse_body_prefix(
                                &mut trial,
                                class_id,
                                max_bytes,
                                &mut trial_table,
                                depth + 1,
                            ) {
                                Ok(prefix_result) => parsed = prefix_result.map(Box::new),
                                // 解析失败（截断、版本超限等）：真实截断会在
                                // A 界跳过时因超出流长度整体失败；内容不支持
                                // 则整个体按未知跳过，条目边界不受影响。
                                Err(_) => {}
                            }
                        }
                        if trial.position() > body_offset + full_body_length {
                            return Err(invalid(format!(
                                "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 体前缀终点\
                                 超出 A={a} 界定的条目边界，前缀解析与 A 界矛盾"
                            )));
                        }
                        // 未知体范围（前缀终点到 A 界）必须完整落在输入内（否则
                        // 按缺短读失败）；记录的是实际跳过的剩余长度。
                        let remaining_skip = body_offset + full_body_length - trial.position();
                        trial.read_bytes(remaining_skip)?;
                        children.push(GroupChildEntry10_1::SkippedNewObject {
                            class_id,
                            key,
                            declared_total: a,
                            body_offset,
                            body_length: remaining_skip,
                            parsed,
                        });
                    }
                }
                if b != 0 {
                    // B!=0 强同步：原生 Seek 到「前缀后位置 + B」。与 A 跳过的
                    // 终点冲突时以 B 同步为准（反编译证据）；负值位模式的回退
                    // Seek 行为未证实，直接拒绝。
                    if (b as i32) < 0 {
                        return Err(invalid(format!(
                            "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 B={b} 为\
                             负值位模式，原生行为未证实"
                        )));
                    }
                    let target = prefix_offset
                        .checked_add(8)
                        .and_then(|position| position.checked_add(b as usize))
                        .ok_or_else(|| {
                            invalid(format!(
                                "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 B={b} \
                                 同步目标溢出"
                            ))
                        })?;
                    if complete && target != trial.position() {
                        return Err(invalid(format!(
                            "子图层同步位置 {target:#x} 与实际消费不一致"
                        )));
                    }
                    trial = reposition(&origin, entry_offset, target, limit).map_err(|_| {
                        invalid(format!(
                            "ExtraSafe 子图层条目 offset 0x{prefix_offset:x}: 前缀 B={b} 同步\
                             目标 0x{target:x} 超出输入末尾"
                        ))
                    })?;
                }
            } else {
                // 版本 <11：普通 LoadObject 条目，无前缀。
                let item_offset = trial.position();
                let saved_table = trial_table.clone();
                let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
                match entry {
                    ObjectEntry10_1::Null => {
                        children.push(GroupChildEntry10_1::Null);
                    }
                    ObjectEntry10_1::BackReference { class_id, key } => {
                        children.push(GroupChildEntry10_1::BackReference { class_id, key });
                    }
                    ObjectEntry10_1::NewObject { class_id, key } => {
                        if complete {
                            let body = read_complete_layer_body_10_1(
                                &mut trial,
                                class_id,
                                max_bytes,
                                &mut trial_table,
                            )?;
                            children.push(GroupChildEntry10_1::ParsedNewObject {
                                class_id,
                                key,
                                parsed: Box::new(body),
                            });
                            continue 'children;
                        }
                        // 新对象体没有外部长度字段。先经类分派解析已确认前缀
                        //（2026-11-14 第六轮接入递归，替代原先一律回退的行
                        // 为）：解析成功时游标停在其停止点并记录
                        // ParsedNewObject（循环无法继续——见停止原因
                        // ChildLayerBody）；类无读取器或解析失败时回退登记表
                        // 与游标到该对象头起点，在此停止（ChildNewObject）。
                        let dispatch = if depth < MAX_RECURSION_DEPTH {
                            crate::layers::layer_list::parse_body_prefix(
                                &mut trial,
                                class_id,
                                max_bytes,
                                &mut trial_table,
                                depth + 1,
                            )
                        } else {
                            Ok(None)
                        };
                        match dispatch {
                            Ok(Some(prefix)) => {
                                children.push(GroupChildEntry10_1::ParsedNewObject {
                                    class_id,
                                    key,
                                    parsed: Box::new(prefix),
                                });
                                stop = Some(GroupLayerStop10_1::ChildLayerBody { index });
                                break 'children;
                            }
                            Ok(None) => {
                                trial_table = saved_table;
                                trial = reposition(&origin, entry_offset, item_offset, limit)
                                    .expect("回退目标在停止前的已读位置，必在窗口内");
                                stop = Some(GroupLayerStop10_1::ChildNewObject {
                                    index,
                                    class_id,
                                    key,
                                    note: None,
                                });
                                break 'children;
                            }
                            Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                                // 截断向上传播为错误（保持「截断输入必须失败」
                                // 的读取器契约）。
                                return Err(error);
                            }
                            Err(error) => {
                                // 内容性失败：回退并停止，原因进诊断。
                                trial_table = saved_table;
                                trial = reposition(&origin, entry_offset, item_offset, limit)
                                    .expect("回退目标在停止前的已读位置，必在窗口内");
                                stop = Some(GroupLayerStop10_1::ChildNewObject {
                                    index,
                                    class_id,
                                    key,
                                    note: Some(error.to_string()),
                                });
                                break 'children;
                            }
                        }
                    }
                }
            }
        }
    }

    // 6. 可见性（→ +0xbc）：版本 <2 不读（原生默认 0xffff），>=2 读取。
    let mut visibility = None;
    if stop.is_none() && version >= 2 {
        visibility = Some(trial.read_u16_le()?);
    }

    // 7. 版本 >2（反编译 `2 < ver`，文档表格标注 >=2，以反编译为准）：
    //    嵌套对象（IEnvelope 候选）。
    let mut nested_after_visibility = None;
    let mut envelope = None;
    if stop.is_none() && version > 2 {
        let header_offset = trial.position();
        let saved_table = trial_table.clone();
        let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
        match entry {
            ObjectEntry10_1::Null => {
                nested_after_visibility = Some(ConsumedEntry10_1::Null);
            }
            ObjectEntry10_1::BackReference { class_id, key } => {
                nested_after_visibility = Some(ConsumedEntry10_1::BackReference { class_id, key });
            }
            ObjectEntry10_1::NewObject {
                class_id: ENVELOPE_CLASS_ID,
                key,
            } if complete => {
                envelope = Some((
                    key,
                    Box::new(read_envelope_10_1(&mut trial, max_bytes, &mut trial_table)?),
                ));
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                trial_table = saved_table;
                trial = reposition(&origin, entry_offset, header_offset, limit)
                    .expect("回退目标在停止前的已读位置，必在窗口内");
                stop = Some(GroupLayerStop10_1::NestedNewObject { class_id, key });
            }
        }
    }

    // 8. 版本 >=4：double（→ +0xc8）。
    let mut double_after_nested = None;
    if stop.is_none() && version >= 4 {
        double_after_nested = Some(trial.read_f64_le()?);
    }

    let tail = if complete && stop.is_none() && version >= 5 {
        Some(read_group_layer_tail(
            &mut trial,
            max_bytes,
            &mut trial_table,
            version,
        )?)
    } else {
        None
    };
    let stop = match stop {
        Some(stop) => stop,
        None if complete => GroupLayerStop10_1::Complete,
        None if version >= 5 => GroupLayerStop10_1::ExtensionHelperNotExpanded,
        None => GroupLayerStop10_1::TailNotExpanded,
    };

    let result = GroupLayerPrefix10_1 {
        version,
        name,
        shorts_after_name,
        doubles,
        child_count,
        children,
        visibility,
        nested_after_visibility,
        envelope,
        double_after_nested,
        tail,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 把游标重定位到「传入 Reader 窗口」的绝对偏移 `target`。
///
/// Reader 没有随机定位原语；`origin` 是函数入口时的游标克隆（位置 =
/// `entry_offset`），这里用入口克隆前向读取实现重定位。`target` 必须在
/// `entry_offset` 与 `limit`（窗口末尾）之间，越界返回错误。
fn reposition<'a>(
    origin: &Reader<'a>,
    entry_offset: usize,
    target: usize,
    limit: usize,
) -> io::Result<Reader<'a>> {
    if !(entry_offset..=limit).contains(&target) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("重定位目标 0x{target:x} 超出输入窗口 [0x{entry_offset:x}, 0x{limit:x}]"),
        ));
    }
    let mut positioned = origin.clone();
    // 长度已由上界校验限制在窗口内，前向读取不会失败。
    positioned
        .read_bytes(target - entry_offset)
        .expect("目标位置已校验在窗口内");
    Ok(positioned)
}

fn read_group_layer_tail(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    version: u16,
) -> io::Result<GroupLayerTail10_1> {
    let extensions = read_layer_extensions_10_1(reader, max_bytes, table)?;
    let string_dc = (version >= 6)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;
    let legacy_v6_short = (version == 6).then(|| reader.read_u16_le()).transpose()?;
    let short_d6 = (version >= 7).then(|| reader.read_u16_le()).transpose()?;
    let doubles_e0_e8 = (version >= 8)
        .then(|| Ok::<_, io::Error>([reader.read_f64_le()?, reader.read_f64_le()?]))
        .transpose()?;
    let short_d8 = (version >= 9).then(|| reader.read_u16_le()).transpose()?;
    let v10_shorts = (version >= 10)
        .then(|| {
            Ok::<_, io::Error>([
                reader.read_u16_le()?,
                reader.read_u16_le()?,
                reader.read_u16_le()?,
            ])
        })
        .transpose()?;
    let display_filter = if version == 12 {
        Some(match read_object_entry_10_1(reader, table)? {
            ObjectEntry10_1::Null => DisplayFilterSlot10_1::Reference(RefSlot10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                DisplayFilterSlot10_1::Reference(RefSlot10_1::BackReference { class_id, key })
            }
            ObjectEntry10_1::NewObject {
                class_id: TRANSPARENCY_DISPLAY_FILTER_CLASS_ID,
                key,
            } => read_transparency_display_filter_10_1(reader, key)?,
            ObjectEntry10_1::NewObject { class_id, key } => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "GroupLayer IDisplayFilter 类 {}（键 {key}）未支持",
                        crate::storage::class_id_hex(&class_id)
                    ),
                ));
            }
        })
    } else {
        None
    };
    // 原生其后仅清理接口、更新内存中的图层遮罩，不消费字节。
    Ok(GroupLayerTail10_1 {
        extensions,
        string_dc,
        legacy_v6_short,
        short_d6,
        doubles_e0_e8,
        short_d8,
        v10_shorts,
        display_filter,
    })
}
