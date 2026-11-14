//! ArcMap 10.1 MapSurround 与空间书签非空循环元素体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-mapsurround-bookmark.md`（2026-11-14 本轮研究
//! 记录）。核心结论全部来自 Map.dll 10.1.0.3035（SHA-256 `1c0675cc…b1bb8`，
//! temp/agent-map-surround/ 本轮提取复核）与 AfCore.dll 10.1.0.3035
//! （`031858c4…7c17`，temp/arcgis101-bin 既有基线）的静态反编译，接口名称
//! 经 10.1 类型库（wine `LoadTypeLibEx` dump，esriCarto）核对。
//!
//! **重要边界：仓库参考样本 `samples/测试底图.mxd` 的 MapSurround 计数
//! （`/Maps:0x24f60`）与书签计数（`0x24f9c`）均为 0，以下全部结论无样本
//! 字节佐证**；本模块只能用合成输入测试，不得声称样本验证。循环结构本身
//! （int32 计数 + 每元素普通 `LoadObject`，20 字节对象头 + 体，无 ExtraSafe
//! 8 字节前缀）与 `map_object::read_map_suffix_10_1` 现有骨架逐条吻合，
//! 衔接点为 [`crate::maps::map_object::MapSuffixStop::MapSurroundBody`] 与
//! [`crate::maps::map_object::MapSuffixStop::BookmarkBody`] 携带的流内类标识
//! （按 CLSID 分派到本模块各读取器）。
//!
//! ## 循环结构（Map::Load `0x793ff4d0` / Map::Save `0x79401850`，确定的）
//!
//! - 第 5 步（IMapSurround，请求 IID `{FE3115E9-…}`，`DAT_79481a84`）：
//!   `AfStreamRead(4)` int32 计数 + 循环 `LoadObject`；元素经 `(this−0x40)`
//!   子接口虚表 `+0x84` 加入集合。
//! - 第 9 步（ISpatialBookmark，请求 IID `{EE53528B-…}`，`DAT_794815a0`）：
//!   int32 计数 + 循环 `LoadObject`；每元素 QI（`{3036D35F-…}`
//!   IFeatureBookmark，第五轮已闭合）后 `FUN_7941c760` 加入集合。
//! - 写侧对称：MapSurround `int32(*(this+0x130)) + 循环 SaveObject`（数组
//!   `*(this+0x12c)`）；书签 `int32(*(this+0x174)) + 循环 SaveObject`（数组
//!   `*(this+0x170)`）。两循环均无 ExtraSafe 前缀。
//!
//! ## MapSurround 族共同前缀（Load `0x794413d0` / Save `0x79441510`）
//!
//! 装饰物族各类（Legend/Scalebar/ScaleText/MarkerNorthArrow/MapTitle 等）
//! 的 `IPersistStream::Load` 先调共享基类前缀读取（`0x794413d0` 共 9 个
//! 调用者，含 Legend `0x793da2f2`、Scalebar `0x794460fd`），再读各自尾段：
//!
//! 1. int16 版本（**原生无上限拒绝**，负值位模式也通过；本读取器仅接受
//!    0..=3 并拒绝其余，较原生更严——写侧版本表 `DAT_79480d58` =
//!    `[2,2,2,2,3]`（`AfStreamGetSaveVersion(docVer, 5, 表)`，docVer≥N 取
//!    `表[N−1]`，见 AfCore `0x68a847f0`），10.1（esriArcGISVersion=5）恒写
//!    3，>3 无写侧来源）；
//! 2. String（名称 → `this−0xa0`，AfCore String::Read 布局）；
//! 3. `LoadObject`（请求 IID `{E6BDAA75-…}` IMap）：关联地图对象。流内为
//!    对已持久化 Map 的键回引头（20 字节，无体）；空标记与回引之外的新对象
//!   属未预期形态，本读取器报不支持错误；
//! 4. 版本 ≥2：int16（`this−0x8c`，原样保留）；
//! 5. 版本 ≥3：32 字节原始读取（WKSEnvelope 候选；原生读取后未存储——读入
//!    栈后丢弃，写侧取 `this−0x84` 恒写，字段语义开放）。原样保留字节。
//!
//! ## AOIBookmark（空间书签，体完全闭合）
//!
//! CLSID `{BBB1AE73-41E3-11D2-AE1E-080009EC732A}`（esri.clsid 声明 Map.dll；
//! GetClassID `0x793a2600` 写出该 CLSID，虚表 `0x79481550`，adjustor 0x14）。
//! Load `0x793a2cd0`：① int16 版本（有符号比较 `1 < (short)v`，仅 0..=1 接受，
//! 负值原生即拒绝）→ ② `LoadObject`（请求 IID `{1B077914-…}` IEnvelope，
//! 位置包络）→ ③ String（书签名称）。Save `0x793a2d60` 恒写版本 1
//! （`DAT_79480eac`）+ `SaveObject` + `String::Write`，读写对称。
//!
//! ## FeatureBookmark（要素书签，体完全闭合）
//!
//! CLSID `{EC65B35B-4342-11D2-AE22-080009EC732A}`（GetClassID `0x793b5520`，
//! 虚表 `0x79482be8`）。Load `0x793b5c50`：① int16 版本（0..=1）→ ② int32
//! （`this+4`，语义未定，原样保留）→ ③ `LoadObject`（请求 IID
//! `{4BA33331-…}` IName，名称对象）→ ④ String（书签名称）。Save
//! `0x793b5d70` 恒写版本 1（`DAT_79480e94`）+ int32 + `SaveObject` +
//! `String::Write`，对称。名称对象的新对象体无读取器（IName 实现类众多），
//! 本模块只分类条目、不解析体。
//!
//! ## Legend（图例，已确认前缀）
//!
//! CLSID `{7A3F91E4-B9E3-11D1-8756-0000F875172A}`（GetClassID `0x793dede0`
//! → `0x793d1330` 写出，IPersistStream 虚表 `0x794859c0`，COL
//! `0x79493048` disp 0x1f4）。Load（虚表槽 `0x793ded80` → `0x793da250`）：
//!
//! 1. int16 Legend 版本：汇编 `CMP word,5; JLE` 有符号比较，>5 拒绝
//!    `0x80004005`（负值原生通过，本读取器一并拒绝，较原生更严）。写侧
//!    版本表 `DAT_79480dc4` = `[3,4,4,4,4,5]`（`AfStreamGetSaveVersion
//!    (docVer, 6, 表)`），10.1 恒写 5；
//! 2. 共同前缀（上节 5 字段）；
//! 3. `LoadObject`（请求 IID `{52AA5BC1-…}` ILegendFormat）：新对象体
//!    （LegendFormat `{7A3F91E5-…}` 等）无读取器——在此停止并保留剩余；
//! 4. String（`this−0xf8`，标题候选）；
//! 5. 版本 ==1：double ×2（读后丢弃的旧版遗留段，原样保留位模式；写侧
//!    无对应写出）；
//! 6. int16 ×3（`this−0xf0/−0xee/−0xec`，原样保留）；
//! 7. int16（传入集合接口 `+0xa8`；写侧恒写 `0xFFFF`）；
//! 8. int32 计数 + 循环 `LoadObject`（请求 IID `{52AA5BC0-…}` ILegendItem）：
//!    空标记 / 回引条目消费后继续；新对象且类标识为 HorizontalLegendItem 时
//!    整体解析其体（接线轮接入 `legend_item` 模块，体内停止透传），其余
//!    LegendItem 实现类新对象在该条目停止并保留剩余；
//! 9. 版本 ≥3：int16（`this−0xe8`）；版本 ≥4：int16（`this−0xe6`）；
//! 10. 版本 ≥5：int16 ×3 + double（`this−0x9c..−0x94`；写侧 docVersion≥5
//!     才写，与版本 5 恒写一致）；<5 时原生写默认值（不读流）。
//!
//! Save（虚表槽 `0x793df010` → `0x793da650`）逐段对称（含 `0xFFFF` short、
//! int32 计数 + 循环 SaveObject 条目）。
//!
//! 调用约定与事务性：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头、比对相应 `*_CLASS_ID` 常量确认类身份，从对象头之后的位置
//! 传入 Reader；本模块不重复读取对象头。全部字段在 trial 游标与 trial
//! 登记表上读取，完整成功（含停止点返回）才一并提交；任何错误既不推进
//! 调用者游标、也不修改登记表。`max_bytes` 约束各 String 声明长度与循环
//! 「计数 × 最小条目字节数」预检，`usize::MAX` 表示不额外设限。这是仅
//! 覆盖上述字段的严格安全子集：无样本字节佐证，未接入 `parse` 命令与
//! 中间模型，人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见研究记录开放项）：共同前缀第 5 段 32 字节与各 short
//! 的业务语义未定性；FeatureBookmark int32 语义未定；LegendFormat 与
//! ILegendItem 实现类（HorizontalBarLegendItem 等）体未闭合；Legend 之外
//! 的装饰物类（Scalebar/ScaleText/MarkerNorthArrow/MapTitle 等）只列清单
//! 未逐类反编译；共同前缀版本 0/1（docVersion 83..93 写 2）无门字段差异
//! 之外的字节佐证。

use crate::layers::legend_item::{
    HORIZONTAL_LEGEND_ITEM_CLASS_ID, HorizontalLegendItemBody10_1, HorizontalLegendItemStop10_1,
    read_horizontal_legend_item_body_10_1,
};
use crate::objects::envelope::{ENVELOPE_CLASS_ID, Envelope10_1, read_envelope_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// AOIBookmark（空间书签）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{BBB1AE73-41E3-11D2-AE1E-080009EC732A}`。证据：
/// esri.clsid.xml 声明 Map.dll 模块类名 `AOIBookmark`；Map.dll 10.1.0.3035
/// GetClassID `0x793a2600` 写出该 CLSID（`DAT_79481590`）。样本中无实例
/// （书签计数 0），无字节佐证。
pub const AOI_BOOKMARK_CLASS_ID: [u8; 16] = [
    0x73, 0xae, 0xb1, 0xbb, 0x41, 0xe3, 0x11, 0xd2, 0xae, 0x1e, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// FeatureBookmark（要素书签）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{EC65B35B-4342-11D2-AE22-080009EC732A}`。证据：
/// esri.clsid.xml 声明 Map.dll；Map.dll GetClassID `0x793b5520` 写出
/// （`DAT_79482c18`）。样本中无实例，无字节佐证。
pub const FEATURE_BOOKMARK_CLASS_ID: [u8; 16] = [
    0x5b, 0xb3, 0x65, 0xec, 0x42, 0x43, 0x11, 0xd2, 0xae, 0x22, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// Legend（图例）类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91E4-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.xml 声明 Map.dll；Map.dll GetClassID `0x793d1330` 写出
/// （`DAT_79485118`）。样本中无实例，无字节佐证。
pub const LEGEND_CLASS_ID: [u8; 16] = [
    0xe4, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// LegendFormat 类标识（CLSID）的 16 字节内存序，供诊断对照。
///
/// GUID 字符串形式为 `{7A3F91E5-B9E3-11D1-8756-0000F8751720}`（esri.clsid
/// 声明 Map.dll）。该类对象体未反编译；[`LegendStop10_1::LegendFormatBody`]
/// 停止点的类标识是否为本常量由流内字节决定。
pub const LEGEND_FORMAT_CLASS_ID: [u8; 16] = [
    0xe5, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// 共同前缀与书签循环条目的最小字节数（16 字节空标记；回引/新对象头 20 字节）。
const MIN_ENTRY_BYTES: usize = 16;

/// 类标识的十六进制描述（诊断用，与 layer_list 的 class_id_hex 同构）。
fn class_id_hex(class_id: &[u8; 16]) -> String {
    class_id.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// MapSurround 族共同前缀第 3 段（关联地图）的分类结果。
///
/// 真实流中该槽为对已持久化 Map 对象的键回引（登记先于持久化，Map 恒先
/// 于其装饰物写出）；空标记表示无关联地图。新对象（整棵 Map 对象体）未
/// 预期，读取时直接报不支持错误，不产生本枚举值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapBackReference10_1 {
    /// 回引条目头携带的类标识（原样保留；应为 Map 的 CLSID，但不强校验）。
    pub class_id: [u8; 16],
    /// 回引目标键（应为本会话已登记的 Map 对象键）。
    pub key: u32,
}

/// MapSurround 族共同前缀（Load `0x794413d0`）的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MapSurroundCommonPrefix10_1 {
    /// int16 版本的无符号位模式。
    ///
    /// 原生无上限拒绝、负值位模式通过；本读取器仅接受 0..=3（写侧版本表
    /// `[2,2,2,2,3]`，10.1 恒写 3；>3 与负值均无字节佐证，一并拒绝，较
    /// 原生更严）。版本 0/1 无门字段，2 追加 short，3 再追加 32 字节。
    pub version: u16,
    /// 装饰物名称（String::Read → `this−0xa0`）。
    pub name: String,
    /// 关联地图条目：`None` = 16 字节空标记；`Some` = 20 字节回引。
    pub map_entry: Option<MapBackReference10_1>,
    /// 版本 ≥2 的 int16 原样位模式（`this−0x8c`，语义未定）。
    pub extra_short: Option<u16>,
    /// 版本 ≥3 的 32 字节原始读取（WKSEnvelope 候选；原生读入后丢弃，
    /// 写侧取 `this−0x84` 恒写。仅原样保留字节，不做数值解释）。
    pub raw_envelope: Option<[u8; 32]>,
}

/// AOIBookmark 位置包络槽（请求 IID `{1B077914-…}` IEnvelope）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkEnvelopeEntry10_1 {
    /// 16 字节全零类标识：空引用标记，无键、无对象体。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 未见过的新键且类标识为 Envelope：已读取完整 Envelope 对象体
    /// （[`crate::objects::envelope`] 覆盖全部版本分支，整体消费）。
    Envelope { key: u32, body: Envelope10_1 },
}

/// AOIBookmark 对象体（Load `0x793a2cd0` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct AoiBookmark10_1 {
    /// int16 版本的无符号位模式（原生有符号比较仅接受 0..=1，负值即拒绝；
    /// Save 恒写 1）。
    pub version: u16,
    /// 位置包络条目（第 2 字段）。
    pub envelope: BookmarkEnvelopeEntry10_1,
    /// 书签名称（第 3 字段，String::Read → `this+8`）。
    pub name: String,
    /// 体之后的剩余字节数（体无外部长度；正常输入应为 0）。
    pub remaining_bytes: usize,
}

/// FeatureBookmark 名称对象槽（请求 IID `{4BA33331-…}` IName）的分类结果。
///
/// IName 实现类众多（FeatureClassName 等已由 `feature_layer` 在图层上下文
/// 闭合，但该槽不限于这些类）；新对象体一律不支持，只分类空标记与回引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkNameEntry10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// FeatureBookmark 对象体（Load `0x793b5c50` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureBookmark10_1 {
    /// int16 版本的无符号位模式（原生有符号比较仅接受 0..=1；Save 恒写 1）。
    pub version: u16,
    /// 第 2 字段 int32 的原样位模式（`this+4`，语义未定）。
    pub raw_int32: i32,
    /// 名称对象条目（第 3 字段）。
    pub name_object: BookmarkNameEntry10_1,
    /// 书签名称（第 4 字段，String::Read → `this+0xc`）。
    pub name: String,
    /// 体之后的剩余字节数（正常输入应为 0）。
    pub remaining_bytes: usize,
}

/// Legend 第 3 字段（请求 IID `{52AA5BC1-…}` ILegendFormat）的分类结果。
/// 新对象体无读取器（LegendFormat 等类未反编译）——触发停止而非枚举值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendFormatEntry10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// Legend 条目循环内一个对象条目（请求 IID `{52AA5BC0-…}` ILegendItem）的
/// 分类结果（接线轮起 HorizontalLegendItem 体闭合并接入分派）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendItemSlot10_1 {
    /// 16 字节全零类标识：空引用标记。
    Null,
    /// 键已在本会话登记：20 字节回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 HorizontalLegendItem 对象（类标识经
    /// [`HORIZONTAL_LEGEND_ITEM_CLASS_ID`] 校验）：已读取其闭合体（Renderers
    /// 包装 + LayerLib BaseLegendItem 字段序；体内停止经
    /// [`LegendStop10_1::LegendItemBodyIncomplete`] 上抛）。
    HorizontalLegendItem(Box<HorizontalLegendItemBody10_1>),
}

/// Legend 条目循环中一个已消费的对象条目（含触发停止的新对象头）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendItemEntry10_1 {
    /// 条目头起点偏移（相对传入 Reader 的底层字节）。
    pub offset: usize,
    /// 条目分类；未知 LegendItem 实现类的新对象不出现在本枚举（触发停止）。
    pub entry: LegendItemSlot10_1,
}

/// Legend 版本 ≥5 尾组的 4 个原样标量（`this−0x9c..−0x94`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegendTailGroup10_1 {
    /// 三个 int16 的原样位模式。
    pub shorts: [u16; 3],
    /// 尾随 double 的原样位模式（<5 时原生写默认 4.5，不读流）。
    pub value: f64,
}

/// Legend 前缀读取的停止原因（非错误：已消费内容有效，其后保留为剩余）。
#[derive(Debug, Clone, PartialEq)]
pub enum LegendStop10_1 {
    /// ILegendFormat 槽位为新对象：类体无读取器，已消费 20 字节对象头，
    /// 游标停在头之后。
    LegendFormatBody {
        /// 对象头起点偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// ILegendItem 条目为新对象且类标识非 HorizontalLegendItem：实现类体无
    /// 读取器，已消费该条目 20 字节对象头，游标停在头之后。
    LegendItemBody {
        /// 条目序号（0 起，含触发停止的条目）。
        index: usize,
        /// 条目对象头起点偏移。
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    /// 接线轮新增：ILegendItem 条目为 HorizontalLegendItem 且其体内出现停
    /// 止（原因透传）。游标停在其内层停止点；该条目对象键已登记，条目以
    /// 部分消费状态记录。
    LegendItemBodyIncomplete {
        /// 条目序号（0 起）。
        index: usize,
        /// HorizontalLegendItem 体内的停止原因。
        reason: HorizontalLegendItemStop10_1,
    },
    /// 版本门尾段读毕（Load `0x793da250` 自然返回点）。
    Completed,
}

impl LegendStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(self) -> String {
        match self {
            Self::LegendFormatBody {
                offset,
                class_id,
                key,
            } => format!(
                "Legend 的 ILegendFormat 槽位（offset 0x{offset:x}）为新对象（类 {}，键 \
                 {key}）：LegendFormat 体未闭合，停止并保留剩余",
                class_id_hex(&class_id)
            ),
            Self::LegendItemBody {
                index,
                offset,
                class_id,
                key,
            } => format!(
                "Legend 条目 {index}（offset 0x{offset:x}）为新对象（类 {}，键 {key}）：\
                 ILegendItem 实现类体未闭合，停止并保留剩余",
                class_id_hex(&class_id)
            ),
            Self::LegendItemBodyIncomplete { index, reason } => format!(
                "Legend 条目 {index}（HorizontalLegendItem）体在停止点停止：{reason:?}；\
                 游标停在内层停止点，其后条目不可达"
            ),
            Self::Completed => {
                "Legend 已确认前缀读毕（版本门尾段之后为该对象体自然终点)".to_string()
            }
        }
    }
}

/// Legend 对象体已确认前缀（Load `0x793da250`，至版本门尾段止）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendPrefix10_1 {
    /// int16 Legend 版本的无符号位模式（原生有符号 `>5` 拒绝、负值通过；
    /// 本读取器仅接受 0..=5，负值一并拒绝较原生更严。10.1 写侧恒写 5）。
    pub version: u16,
    /// MapSurround 族共同前缀（第 2 字段起）。
    pub common: MapSurroundCommonPrefix10_1,
    /// ILegendFormat 槽位条目（第 3 字段；到达即有值）。
    pub legend_format: LegendFormatEntry10_1,
    /// 标题候选字符串（第 4 字段，`this−0xf8`）。
    pub title: String,
    /// 版本 ==1 的两个 double 原样位模式（读后丢弃的旧版遗留段；写侧无
    /// 对应写出，仅旧文档出现）。
    pub legacy_doubles: Option<[f64; 2]>,
    /// 三个 int16 原样位模式（第 6 字段，`this−0xf0/−0xee/−0xec`）。
    pub shorts: [u16; 3],
    /// 集合接口 `+0xa8` 的 int16 原样位模式（第 7 字段；写侧恒写 `0xFFFF`）。
    pub selection_short: u16,
    /// 条目循环 int32 计数原始值（第 8 字段；负数拒绝，较原生更严）。
    pub item_count: i32,
    /// 已消费的条目（按流内顺序；末位可能是触发停止的新对象头）。
    pub items: Vec<LegendItemEntry10_1>,
    /// 版本 ≥3 的 int16 原样位模式（`this−0xe8`）。
    pub tail_short_1: Option<u16>,
    /// 版本 ≥4 的 int16 原样位模式（`this−0xe6`）。
    pub tail_short_2: Option<u16>,
    /// 版本 ≥5 的尾组（int16 ×3 + double）；<5 时原生写默认值不读流。
    pub tail_group: Option<LegendTailGroup10_1>,
    /// 停止原因。
    pub stop: LegendStop10_1,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 MapSurround 族共同前缀，失败时不推进游标。
///
/// 起点约定：调用者已确认这是某个 MapSurround 族对象体的共同前缀起点
/// （各类 Load 先调 `0x794413d0`；独立暴露供研究性读取与测试）。字段序、
/// 版本门与证据见模块文档。`max_bytes` 约束名称字符串；`table` 为跨流共享
/// 键登记表（第 3 段 IMap 槽的分类依赖它；新对象在本读取器中报不支持
/// 错误，因此本函数自身不引入新键，trial/commit 语义仍完整保留）。
pub fn read_map_surround_common_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<MapSurroundCommonPrefix10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("MapSurround 共同前缀 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本：原生无上限拒绝、负值通过；仅接受 0..=3（较原生更严，
    //    写侧 10.1 恒写 3，其余位模式无字节佐证）。
    let version = trial.read_u16_le()?;
    if version > 3 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出本读取器接受范围 0..=3；原生读取\
             （0x794413d0）无上限拒绝、负值位模式亦通过，写侧版本表 DAT_79480d58 \
             = [2,2,2,2,3]（10.1 恒写 3），其余值无字节佐证，拒绝"
        )));
    }

    // 2. 名称字符串（String::Read 布局 → this−0xa0）。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    // 3. LoadObject(IMap {E6BDAA75-…})：关联地图。登记先于持久化使该槽在
    //    真实流中恒为回引；空标记与回引之外的新对象（整棵 Map 体，无外部
    //    长度）不可跳过，报不支持。
    let map_offset = trial.position();
    let map_entry = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, key } => {
            Some(MapBackReference10_1 { class_id, key })
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "MapSurround 共同前缀关联地图槽 offset 0x{map_offset:x}: 类标识 {}（键 \
                     {key}）为新对象；该槽应为对已持久化 Map 的键回引（或空标记），新对象\
                     体为整棵 Map 树且无外部长度字段，无法读取或跳过",
                    class_id_hex(&class_id)
                ),
            ));
        }
    };

    // 4. 版本 ≥2：int16 原样（this−0x8c）。
    let extra_short = if version >= 2 {
        Some(trial.read_u16_le()?)
    } else {
        None
    };

    // 5. 版本 ≥3：32 字节原样（WKSEnvelope 候选；原生读入后丢弃，写侧取
    //    this−0x84 恒写）。
    let raw_envelope = if version >= 3 {
        let bytes = trial.read_bytes(32)?;
        Some(bytes.try_into().expect("固定 32 字节"))
    } else {
        None
    };

    let result = MapSurroundCommonPrefix10_1 {
        version,
        name,
        map_entry,
        extra_short,
        raw_envelope,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 AOIBookmark（空间书签）对象体，失败时不推进
/// 游标、不修改共享登记表。
///
/// 起点约定：调用者已读取对象头并比对 [`AOI_BOOKMARK_CLASS_ID`]。字段序
/// 与证据见模块文档（Load `0x793a2cd0` / Save `0x793a2d60`）。嵌套包络为
/// 新对象且类标识是 Envelope 时用其读取器整体消费；其他新对象类报
/// [`ErrorKind::Unsupported`]（体无外部长度字段，无法跳过）。
pub fn read_aoi_bookmark_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<AoiBookmark10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("AOIBookmark 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本：原生 `lVar4<0 || 1<(short)v` 拒绝，即有符号 0..=1
    //    （负值原生即拒绝，与位模式 >1 的拒绝合并为同一无符号判定）。
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=1（Load \
             0x793a2cd0 有符号比较，>1 或负值拒绝 0x80004005；Save 恒写 1）"
        )));
    }

    // 2. LoadObject(IEnvelope {1B077914-…})：位置包络。
    let envelope_offset = trial.position();
    let envelope = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => BookmarkEnvelopeEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            BookmarkEnvelopeEntry10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == ENVELOPE_CLASS_ID => {
            // Envelope 全部版本分支已闭合：整体消费对象体（含嵌套空间参考）。
            let body = read_envelope_10_1(&mut trial, max_bytes, &mut trial_table)?;
            BookmarkEnvelopeEntry10_1::Envelope { key, body }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "AOIBookmark 位置包络槽 offset 0x{envelope_offset:x}: 类标识 {}（键 \
                     {key}）不是 Envelope；本读取器仅支持 Envelope 新对象嵌套，且对象体\
                     无外部长度字段，无法跳过未知类继续读取",
                    class_id_hex(&class_id)
                ),
            ));
        }
    };

    // 3. 书签名称（String::Read → this+8）。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    let result = AoiBookmark10_1 {
        version,
        envelope,
        name,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 FeatureBookmark（要素书签）对象体，失败时不
/// 推进游标、不修改共享登记表。
///
/// 起点约定：调用者已读取对象头并比对 [`FEATURE_BOOKMARK_CLASS_ID`]。
/// 名称对象槽（IName）只分类空标记与回引；新对象体一律报
/// [`ErrorKind::Unsupported`]（IName 实现类众多，无外部长度字段）。
pub fn read_feature_bookmark_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<FeatureBookmark10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FeatureBookmark 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本：原生有符号 0..=1。
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=1（Load \
             0x793b5c50 有符号比较；Save 恒写 1）"
        )));
    }

    // 2. int32 原样（this+4，语义未定）。
    let raw_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());

    // 3. LoadObject(IName {4BA33331-…})：名称对象，只分类空标记/回引。
    let name_offset = trial.position();
    let name_object = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => BookmarkNameEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            BookmarkNameEntry10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "FeatureBookmark 名称对象槽 offset 0x{name_offset:x}: 类标识 {}（键 \
                     {key}）为新对象；IName 实现类众多且对象体无外部长度字段，本读取器\
                     不解析任何 IName 新对象体",
                    class_id_hex(&class_id)
                ),
            ));
        }
    };

    // 4. 书签名称（String::Read → this+0xc）。
    let name = read_string_10_1(&mut trial, max_bytes)?;

    let result = FeatureBookmark10_1 {
        version,
        raw_int32,
        name_object,
        name,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 从对象头之后的位置读取 Legend 对象体的已确认前缀，失败时不推进游标、
/// 不修改共享登记表；停止点为成功返回（已消费部分 + 剩余字节 + 原因）。
///
/// 起点约定：调用者已读取对象头并比对 [`LEGEND_CLASS_ID`]。字段序与停止
/// 规则见模块文档（Load `0x793da250`）：ILegendFormat 槽与 ILegendItem 条目
/// 的新对象体无读取器，消费 20 字节对象头后停止；版本门尾段读毕为
/// [`LegendStop10_1::Completed`]。`max_bytes` 约束两个字符串与条目计数
/// 预检（计数 × 16 最小条目字节数）。
pub fn read_legend_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<LegendPrefix10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Legend 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 Legend 版本：汇编 `CMP word,5; JLE` 有符号（>5 拒绝
    //    0x80004005，负值原生通过）；仅接受 0..=5，负值一并拒绝较原生更严。
    let version = trial.read_u16_le()?;
    if version > 5 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=5（Load \
             0x793da250 有符号比较，>5 拒绝 0x80004005；写侧版本表 DAT_79480dc4 = \
             [3,4,4,4,4,5]，10.1 恒写 5；负值位模式原生通过，此处一并拒绝，较原生更严）"
        )));
    }

    // 2. MapSurround 族共同前缀（含 IMap 槽新对象的Unsupported 转发）。
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, &mut trial_table)?;

    // 3. LoadObject(ILegendFormat {52AA5BC1-…})：空标记/回引消费后继续；
    //    新对象体无读取器，消费头后停止。
    let format_offset = trial.position();
    let legend_format;
    let mut stop = match read_object_entry_10_1(&mut trial, &mut trial_table)? {
        ObjectEntry10_1::Null => {
            legend_format = LegendFormatEntry10_1::Null;
            None
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            legend_format = LegendFormatEntry10_1::BackReference { class_id, key };
            None
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            legend_format = LegendFormatEntry10_1::Null;
            Some(LegendStop10_1::LegendFormatBody {
                offset: format_offset,
                class_id,
                key,
            })
        }
    };

    let mut title = String::new();
    let mut legacy_doubles = None;
    let mut shorts = [0_u16; 3];
    let mut selection_short = 0;
    let mut item_count = 0;
    let mut items = Vec::new();
    let mut tail_short_1 = None;
    let mut tail_short_2 = None;
    let mut tail_group = None;

    if stop.is_none() {
        // 4. 标题候选字符串（this−0xf8）。
        title = read_string_10_1(&mut trial, max_bytes)?;

        // 5. 版本 ==1：double ×2（读后丢弃的旧版遗留段，原样保留）。
        if version == 1 {
            let mut pair = [0_f64; 2];
            for slot in &mut pair {
                *slot = trial.read_f64_le()?;
            }
            legacy_doubles = Some(pair);
        }

        // 6/7. int16 ×3 + int16（集合接口 +0xa8；写侧恒写 0xFFFF）。
        for slot in &mut shorts {
            *slot = trial.read_u16_le()?;
        }
        selection_short = trial.read_u16_le()?;
    }

    if stop.is_none() {
        // 8. int32 条目计数 + 循环 LoadObject(ILegendItem {52AA5BC0-…})。
        let count_offset = trial.position();
        let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        if count < 0 {
            return Err(invalid(format!(
                "条目计数 offset 0x{count_offset:x}: 计数为负数（{count}）；原生读取仅\
                 正数进入循环（负数跳过），本读取器拒绝以避免静默错位"
            )));
        }
        item_count = count;
        let count_usize = usize::try_from(count).expect("计数已确认为非负");
        let min_total = count_usize
            .checked_mul(MIN_ENTRY_BYTES)
            .ok_or_else(|| invalid("条目计数 × 最小条目字节数溢出".to_string()))?;
        if min_total > max_bytes {
            return Err(invalid(format!(
                "条目计数 offset 0x{count_offset:x}: 声明计数 {count} × 最小条目字节数 \
                 {MIN_ENTRY_BYTES} 超过资源限额 {max_bytes}，在条目读取之前拒绝"
            )));
        }
        for index in 0..count_usize {
            let offset = trial.position();
            let raw = read_object_entry_10_1(&mut trial, &mut trial_table)?;
            let entry = match raw {
                ObjectEntry10_1::Null => LegendItemSlot10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    LegendItemSlot10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == HORIZONTAL_LEGEND_ITEM_CLASS_ID =>
                {
                    // 接线：HorizontalLegendItem 体闭合（Renderers + LayerLib
                    // BaseLegendItem 字段序），整体消费；体内停止透传。
                    let body = read_horizontal_legend_item_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut trial_table,
                        key,
                    )?;
                    if let Some(reason) = body.stop.clone() {
                        stop = Some(LegendStop10_1::LegendItemBodyIncomplete { index, reason });
                    }
                    LegendItemSlot10_1::HorizontalLegendItem(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    // 其余 LegendItem 实现类体无读取器：已消费 20 字节对象头，
                    // 游标停在头之后（键已按「登记先于持久化」登记）。
                    stop = Some(LegendStop10_1::LegendItemBody {
                        index,
                        offset,
                        class_id,
                        key,
                    });
                    LegendItemSlot10_1::Null
                }
            };
            items.push(LegendItemEntry10_1 { offset, entry });
            if stop.is_some() {
                break;
            }
        }
    }

    if stop.is_none() {
        // 9. 版本 ≥3：int16（this−0xe8）。
        if version >= 3 {
            tail_short_1 = Some(trial.read_u16_le()?);
        }
        // 10. 版本 ≥4：int16（this−0xe6）。
        if version >= 4 {
            tail_short_2 = Some(trial.read_u16_le()?);
        }
        // 11. 版本 ≥5：int16 ×3 + double（<5 时原生写默认 4.5，不读流）。
        if version >= 5 {
            let mut group = LegendTailGroup10_1 {
                shorts: [0; 3],
                value: 0.0,
            };
            for slot in &mut group.shorts {
                *slot = trial.read_u16_le()?;
            }
            group.value = trial.read_f64_le()?;
            tail_group = Some(group);
        }
    }

    let stop = stop.unwrap_or(LegendStop10_1::Completed);
    let result = LegendPrefix10_1 {
        version,
        common,
        legend_format,
        title,
        legacy_doubles,
        shorts,
        selection_short,
        item_count,
        items,
        tail_short_1,
        tail_short_2,
        tail_group,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}
