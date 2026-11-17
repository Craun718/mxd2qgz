//! ArcMap 10.1 MapSurround 族开放对象体（LegendFormat/Scalebar 及四个子类/
//! ScaleText/MarkerNorthArrow/MapTitle）的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-legend-family.md`（2026-11-16 本轮研究记录）。
//! 二进制证据为 Map.dll 10.1.0.3035（SHA-256 `1c0675cc…b1bb8`，与既有基线
//! 一致）的静态反编译；各coclass 的身份经 esri.clsid.ecfg 声明模块 +
//! GetClassID 指令写出（或 RTTI TD→COL→identity 虚表 +0x14）三方互证。
//! 各类 Load 先读**自己的版本 short**，再调 MapSurround 基类共享前缀
//! （[`read_map_surround_common_prefix_10_1`]，`0x794413d0`），随后读各自
//! 尾段；Save 侧对称（版本经静态常量或 `AfStreamGetSaveVersion` 版本表）。
//!
//! **样本边界（先声明）**：仓库参考样本 `samples/测试底图.mxd` 的
//! MapSurround 循环计数为 0，本模块全部类 CLSID 在全流 0 命中，全部布局
//! 结论**无样本字节佐证**，仅有反编译一手证据；单元测试全部使用合成输入，
//! 不得声称样本验证。
//!
//! ## 持久化入口与字段序摘要（详见研究文档）
//!
//! | 类 | Load 实体 | Save 实体 | 版本门（读侧） | 10.1 写侧版本 |
//! | --- | --- | --- | --- | --- |
//! | LegendFormat `{7A3F91E5}` | `0x793e0c50` | `0x793e0ec0` | 有符号 >2 拒绝 | 表 `[1,1,1,1,1,2]` → 2 |
//! | Scalebar `{7A3F91DB}` | `0x794460b0` | `0x79446550` | 有符号 >1 拒绝 | 恒写 1 |
//! | ScaleText `{7A3F91DC}` | `0x79457410` | `0x794576a0` | 有符号 >3 拒绝 | 表 `[2,2,2,3]` → 3 |
//! | MarkerNorthArrow `{7A3F91DD}` | `0x79444b80` | `0x79444ca0` | 无拒绝门 | 恒写 2 |
//! | MapTitle `{7A3F91DF}` | `0x794426f0` | `0x79442740` | 无拒绝门 | 恒写 1 |
//! | Hollow/Alternating/DoubleAlternating `{6589F143/147/148}` | `0x793a1be0`（三类共享） | `0x793a1c60`/`0x793b3410` | 有符号 >1 拒绝 | 恒写 1 |
//! | SingleDivisionScaleBar `{6589F146}` | `0x79458830` | `0x794588b0` | 有符号 >1 拒绝 | 恒写 1 |
//!
//! 比例尺子类在自身 short 之后调**基类装载链**（全部含共同前缀）：基类
//! 段（Load `0x79451120`，版本门有符号 >6 拒绝，写侧表 `[3,3,3,3,4,6]` →
//! 10.1 恒写 6）→ 中段（Hollow/Alt/DblAlt 共享 `0x793b3bf0`，SingleDiv
//! `0x79459000`；各读 short + double + 1..2 个 IFillSymbol）→ 尾段（共享
//! `0x79454ee0`，版本门有符号 >2 拒绝，写侧恒写 2；2 个 ILineSymbol +
//! double ×2 + int32 ×2）。MarkerNorthArrow 在自身 short 之后调
//! BaseNorthArrow::Load（`0x793a3b90`，版本门有符号 >4 拒绝，写侧表
//! `[3,3,3,3,3,4]` → 10.1 恒写 4）。
//!
//! 嵌套对象槽按已闭合读取器分派：ITextSymbol 新对象且类标识为 TextSymbol
//! 时整体消费（[`crate::objects::element_body`]）、IFont 槽为 OLE StdFont
//! 时整体消费（[`crate::objects::ole_font`]）、ITextElement/IMarkerElement
//! 同理、IPoint 槽经几何读取器、ILinePatch/IAreaPatch 槽经
//! [`crate::layers::patch`]、INumberFormat 槽仅分派 NumericFormat、
//! ISymbol 族槽经 [`crate::objects::display_symbol::read_symbol_slot_full_10_1`]
//! 全量分派；其余实现类新对象为停止点（成功返回，游标回退到槽位起点）。
//!
//! 与 [`crate::layers::legend_item`] 的分派差异（写给整合轮）：本模块的
//! ITextSymbol/INumberFormat 槽分派已闭合类，而 legend_item（Horizontal-
//! LegendItem）在这些槽上一律停止；整合轮建议把 legend_item 的槽分派对齐
//! 到本模块口径，并复用本模块的 BaseLegendItem 字段序读取器消除副本。
//!
//! 调用约定与事务性：调用者已用
//! [`crate::storage::object_stream::read_header_10_1`] 读取对象头、比对
//! 相应 `*_CLASS_ID` 常量并取得键，从对象头之后的位置传入 Reader。停止点
//! 是成功返回（游标回退到槽位起点、该键不保留）；截断、版本超限为错误，
//! 游标与登记表整体不动。`max_bytes` 约束各 String 声明长度与
//! [`read_symbol_slot_full_10_1`] 的透传，`usize::MAX` 表示不额外设限。
//!
//! 未解决问题（详见研究文档开放项）：全部标量字段的业务语义未定性；
//! Scalebar 三个 IFont 槽、ScaleText 两个字符串等的候选语义按类名与写侧
//! 槽位推断；版本短体形态（全部 <10.1 写侧版本）无字节佐证；共同前缀的
//! 版本门语义沿用 [`read_map_surround_common_prefix_10_1`]。全部结论为
//! 静态反编译观察，未经目标软件运行验证，不得声称已验证。

use crate::layers::patch::{
    AREA_PATCH_CLASS_ID, LINE_PATCH_CLASS_ID, PatchBody10_1, read_patch_body_10_1,
};
use crate::maps::map_surround::{
    MapSurroundCommonPrefix10_1, read_map_surround_common_prefix_10_1,
};
use crate::objects::display_symbol::{
    DisplaySlotKind10_1, MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, SymbolStop10_1,
    read_symbol_slot_full_10_1, slot_full_stop_10_1,
};
use crate::objects::element_body::{
    MARKER_ELEMENT_CLASS_ID, MarkerElementBody10_1, TEXT_ELEMENT_CLASS_ID, TEXT_SYMBOL_CLASS_ID,
    TextElementBody10_1, TextSymbolBody10_1, read_marker_element_body_10_1,
    read_text_element_body_10_1, read_text_symbol_body_10_1,
};
use crate::objects::geometry::{POINT_CLASS_ID, PointBody10_1, read_point_body_10_1};
use crate::objects::number_format::{
    NUMERIC_FORMAT_CLASS_ID, NumericFormatBody10_1, read_numeric_format_body_10_1,
};
use crate::objects::ole_font::{STDFONT_CLASS_ID, StdFontBody10_1, read_std_font_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// LegendFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91E5-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.ecfg 声明（模块 Map.dll，类名 LegendFormat）；Map.dll
/// GetClassID `0x793dfd50` 写出（`DAT_79485128/79485130`）。样本全流
/// 0 命中。
pub const LEGEND_FORMAT_CLASS_ID: [u8; 16] = [
    0xe5, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// Scalebar 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91DB-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.ecfg 声明（模块 Map.dll）；Map.dll GetClassID `0x79446040`
/// 写出（`DAT_794850c8/794850d0`）。样本全流 0 命中。
pub const SCALEBAR_CLASS_ID: [u8; 16] = [
    0xdb, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// ScaleText 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91DC-B9E3-11D1-8756-0000F8751720}`。证据：
/// GetClassID `0x79455970` 写出（`DAT_794850d8/794850e0`）。样本全流
/// 0 命中。
pub const SCALE_TEXT_CLASS_ID: [u8; 16] = [
    0xdc, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// MarkerNorthArrow 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91DD-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.ecfg 声明（模块 Map.dll）；identity 虚表 +0x14 槽
/// `0x79444b40` 写出 `DAT_794850e8/794850f0`（该 CLSID）。样本全流 0 命中。
pub const MARKER_NORTH_ARROW_CLASS_ID: [u8; 16] = [
    0xdd, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// MapTitle 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{7A3F91DF-B9E3-11D1-8756-0000F8751720}`。证据：
/// esri.clsid.ecfg 声明（模块 Map.dll）；identity 虚表 +0x14 槽
/// `0x794426b0` 写出 `DAT_79485108/79485110`（该 CLSID）。样本全流 0 命中。
pub const MAP_TITLE_CLASS_ID: [u8; 16] = [
    0xdf, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8, 0x75, 0x17, 0x20,
];

/// HollowScaleBar 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{6589F143-F7F7-11D2-B872-00600802E603}`。证据：
/// esri.clsid.ecfg 声明（模块 Map.dll）；GetClassID `0x793c5ea0` 写出
/// （`DAT_79483b24/79483b2c`）。样本全流 0 命中。
pub const HOLLOW_SCALEBAR_CLASS_ID: [u8; 16] = [
    0x43, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// SingleDivisionScaleBar 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{6589F146-F7F7-11D2-B872-00600802E603}`。证据：
/// GetClassID `0x79457db0` 写出（`DAT_794850b8/794850c0`）。样本全流
/// 0 命中。
pub const SINGLE_DIVISION_SCALEBAR_CLASS_ID: [u8; 16] = [
    0x46, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// AlternatingScaleBar 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{6589F147-F7F7-11D2-B872-00600802E603}`。证据：
/// GetClassID `0x793a12b0` 写出（`DAT_79481430/79481438`）。样本全流
/// 0 命中。
pub const ALTERNATING_SCALEBAR_CLASS_ID: [u8; 16] = [
    0x47, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// DoubleAlternatingScaleBar 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{6589F148-F7F7-11D2-B872-00600802E603}`。证据：
/// GetClassID `0x793b2950` 写出（`DAT_79482900/79482908`）。样本全流
/// 0 命中。
pub const DOUBLE_ALTERNATING_SCALEBAR_CLASS_ID: [u8; 16] = [
    0x48, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// ScaleLine `{6589F140-F7F7-11D2-B872-00600802E603}`，见续查记录。
pub const SCALE_LINE_CLASS_ID: [u8; 16] = [
    0x40, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];
/// SteppedScaleLine `{6589F141-F7F7-11D2-B872-00600802E603}`。
pub const STEPPED_SCALE_LINE_CLASS_ID: [u8; 16] = [
    0x41, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// 版本门上限：Scalebar/比例尺子类自身版本（有符号 >1 拒绝；Save 恒写 1）。
const MAX_OWN_VERSION_1: i16 = 1;
/// 版本门上限：BaseNorthArrow 版本（有符号 >4 拒绝；10.1 写 4）。
const MAX_BASE_NORTH_ARROW_VERSION: i16 = 4;
/// 版本门上限：比例尺子类基类段版本（有符号 >6 拒绝；10.1 写 6）。
const MAX_SCALEBAR_BASE_VERSION: i16 = 6;
/// 版本门上限：比例尺子类中段版本（有符号 >1 拒绝；Save 恒写 1）。
const MAX_SCALEBAR_MID_VERSION: i16 = 1;
/// 版本门上限：比例尺子类尾段版本（有符号 >2 拒绝；Save 恒写 2）。
const MAX_SCALEBAR_TAIL_VERSION: i16 = 2;

/// 族内对象槽的通用分类结果：空标记 / 回引 / 已闭合类整体消费 / 其余类
/// 停止点（`Unsupported`，游标已回退到槽位起点）。
#[derive(Debug, Clone, PartialEq)]
pub enum SlotEntry10_1<T> {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识为分派目标类：已整体消费其对象体。
    Body(Box<T>),
    /// 新对象但类标识非分派目标类：停止点，游标回退到槽位起点、该键
    /// 不保留。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 族读取的停止原因（体内在首个不可闭合处停止，非错误）。
///
/// C 轮接线起被 map_object 的 MapSuffixStop（Copy）携带，补 `Copy`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilyStop10_1 {
    /// 第 `slot` 个对象槽（槽名诊断用）出现新对象且类体未闭合；游标已
    /// 回退到该槽位起点。
    SlotUnsupported {
        /// 槽名（诊断用，如 `"title_symbol(ITextSymbol)"`）。
        slot: &'static str,
        class_id: [u8; 16],
        key: u32,
    },
    /// MarkerElement 的内层读取未完成；具体原因保存在元素体的 stop 字段。
    MarkerElementIncomplete,
    /// 旧版 IMarkerSymbol 槽或其嵌套体未完成。
    LegacyMarkerSymbol(SymbolStop10_1),
}

impl FamilyStop10_1 {
    /// 停止原因的固定说明（供诊断直接输出）。
    pub fn describe(self) -> String {
        match self {
            Self::MarkerElementIncomplete => {
                "MarkerNorthArrow 的 MarkerElement 体内停止；详情见元素体".to_owned()
            }
            Self::LegacyMarkerSymbol(reason) => {
                format!("MarkerNorthArrow 的旧版符号槽停止：{reason:?}")
            }
            Self::SlotUnsupported {
                slot,
                class_id,
                key,
            } => {
                let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
                format!(
                    "MapSurround 族对象体的 {slot} 槽为新对象（类 {id}，键 {key}）：实现类\
                     体未闭合，停止并保留剩余"
                )
            }
        }
    }
}

/// 对象槽通用读取：空标记/回引分类；新对象类标识命中 `target_class` 时
/// 经 `read_body`（收到槽位对象键）整体消费，否则停止（游标回退到槽位
/// 起点、该键不保留）。
///
/// 返回槽分类与停止原因（`Some` 时调用者终止后续字段读取）。
fn read_typed_slot_10_1<T>(
    reader: &mut Reader<'_>,
    _max_bytes: usize,
    table: &mut RefTable10_1,
    slot: &'static str,
    target_class: &[u8; 16],
    read_body: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1, u32) -> io::Result<T>,
) -> io::Result<(SlotEntry10_1<T>, Option<FamilyStop10_1>)> {
    let origin = reader.clone();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => {
            *reader = trial;
            *table = staged;
            Ok((SlotEntry10_1::Null, None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *reader = trial;
            *table = staged;
            Ok((SlotEntry10_1::BackReference { class_id, key }, None))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if class_id == *target_class {
                let body = read_body(&mut trial, &mut staged, key)?;
                *reader = trial;
                *table = staged;
                Ok((SlotEntry10_1::Body(Box::new(body)), None))
            } else {
                *reader = origin;
                Ok((
                    SlotEntry10_1::Unsupported { class_id, key },
                    Some(FamilyStop10_1::SlotUnsupported {
                        slot,
                        class_id,
                        key,
                    }),
                ))
            }
        }
    }
}

/// 从对象头之后的位置读取 MapTitle（地图标题）对象体。
///
/// 字段序（Load `0x794426f0` / Save `0x79442740`）：① int16 自身版本
/// （原生无拒绝门、负值位模式通过；Save 恒写 1（`DAT_79480d48`），本
/// 读取器仅接受 0..=1 较原生更严）→ ② MapSurround 族共同前缀（含其
/// 自身的版本 short/String/IMap 槽/条件字段）。**MapTitle 的持久化体
/// 到此完结**（Save 侧同构：短 + 共同前缀 Save `0x79441510`）。
pub fn read_map_title_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<MapTitleBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("MapTitle 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出本读取器接受范围 0..=1（Load \
             0x794426f0 无拒绝门、负值位模式亦通过；Save 恒写 1，其余值无字节佐证，拒绝）"
        )));
    }
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, table)?;
    *reader = trial;
    Ok(MapTitleBody10_1 {
        key,
        version,
        common,
    })
}

/// MapTitle 对象体（Load `0x794426f0` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct MapTitleBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// int16 自身版本的无符号位模式（原生无拒绝门；Save 恒写 1；读取器
    /// 仅接受 0..=1，较原生更严）。
    pub version: u16,
    /// MapSurround 族共同前缀（第 2 字段起，含其版本门条件字段）。
    pub common: MapSurroundCommonPrefix10_1,
}

/// 从对象头之后的位置读取 Scalebar（比例尺）对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 字段序（Load `0x794460b0` / Save `0x79446550`，逐段对称）：① int16
/// 自身版本（有符号 >1 拒绝 `0x80004005`；Save 恒写 1 `DAT_79480d40`）
/// → ② 共同前缀 → ③ 3×int32、double、3×int16、16×4 字节原样标量 →
/// ④ IFont 槽（`this−0x6c`）→ ⑤ uint32+int32 → ⑥ IFont 槽（`−0x54`）→
/// ⑦ uint32 → ⑧ String（数字格式串候选）→ ⑨ 4×int32 → ⑩ IFont 槽
/// （`−0x2c`）→ ⑪ uint32。三个 IFont 槽（请求 IID
/// `{BEF6E002-A874-101A-8BBA-00AA00300CAB}`，OLE 标准 IFont）流内预期
/// 为 OLE StdFont（`{0BE35203-…}`），新对象命中时整体消费；其余类停止。
pub fn read_scalebar_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ScalebarBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Scalebar 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 1. 自身版本 short：原生有符号 `1 < v` 拒绝（负值位模式原生通过，
    //    一并拒绝较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_OWN_VERSION_1 || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出 Scalebar::Load 接受范围（有符号 \
             >1 拒绝 0x80004005；Save 恒写 1；负值位模式原生通过，此处一并拒绝）"
        )));
    }

    // 2. MapSurround 族共同前缀。
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, table)?;

    // 3. 标量段：3×int32 + double + 3×int16 + 16×4 字节原样（含符号差异，
    //    原样保留位模式）。
    let mut head_ints = [0_u32; 3];
    for slot in &mut head_ints {
        *slot = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }
    let bar_double = trial.read_f64_le()?;
    let mut bar_shorts = [0_u16; 3];
    for slot in &mut bar_shorts {
        *slot = trial.read_u16_le()?;
    }
    let mut raw_tail = [0_u32; 16];
    for slot in &mut raw_tail {
        *slot = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }

    let _ = key;
    // 4. IFont 槽（−0x6c）。
    let (font_a, mut stop) = read_typed_slot_10_1(
        &mut trial,
        max_bytes,
        table,
        "font_a(IFont)",
        &STDFONT_CLASS_ID,
        |reader, _table, _key| read_std_font_body_10_1(reader, max_bytes),
    )?;

    // 5. uint32 + int32（−0x5c/−0x58）。
    let mut mid_pair = [0_u32; 2];
    if stop.is_none() {
        for slot in &mut mid_pair {
            *slot = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
    }

    // 6. IFont 槽（−0x54）。
    let mut font_b = SlotEntry10_1::Null;
    if stop.is_none() {
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "font_b(IFont)",
            &STDFONT_CLASS_ID,
            |reader, _table, _key| read_std_font_body_10_1(reader, max_bytes),
        )?;
        font_b = slot;
        stop = reason;
    }

    // 7..11. uint32（−0x44）+ String（−0x40）+ 4×int32 + IFont 槽 +
    //         uint32（−0x1c）。
    let mut tail_uint = None;
    let mut tail_string = String::new();
    let mut tail_ints = [0_u32; 4];
    let mut font_c = SlotEntry10_1::Null;
    let mut last_uint = None;
    if stop.is_none() {
        tail_uint = Some(u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        tail_string = read_string_10_1(&mut trial, max_bytes)?;
        for slot in &mut tail_ints {
            *slot = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        }
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "font_c(IFont)",
            &STDFONT_CLASS_ID,
            |reader, _table, _key| read_std_font_body_10_1(reader, max_bytes),
        )?;
        font_c = slot;
        if reason.is_none() {
            last_uint = Some(u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        stop = reason;
    }

    let remaining = trial.remaining();
    *reader = trial;
    Ok(ScalebarBody10_1 {
        key,
        version,
        common,
        head_ints,
        bar_double,
        bar_shorts,
        raw_tail,
        font_a,
        mid_pair,
        font_b,
        tail_uint,
        tail_string,
        tail_ints,
        font_c,
        last_uint,
        stop,
        remaining_bytes: remaining,
    })
}

/// Scalebar 标量段 −0xac..−0x70 的 16 个 4 字节原样值的偏移对照。
///
/// 索引 n 对应 `this + (−0xac + 4n)`；原生读侧符号混合（Ghidra 标注
/// long/ulong 交替），读取器统一按位模式保留。
pub const SCALEBAR_RAW_TAIL_OFFSETS: [i32; 16] = [
    -0xac, -0xa8, -0xa4, -0xa0, -0x9c, -0x98, -0x94, -0x90, -0x8c, -0x88, -0x84, -0x80, -0x7c,
    -0x78, -0x74, -0x70,
];

/// Scalebar 对象体（Load `0x794460b0` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScalebarBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// int16 自身版本的无符号位模式（有符号 >1 拒绝；Save 恒写 1；负值
    /// 拒绝较原生更严）。
    pub version: u16,
    /// MapSurround 族共同前缀。
    pub common: MapSurroundCommonPrefix10_1,
    /// 3×int32（`−0xdc/−0xd8/−0xd4`，位模式原样）。
    pub head_ints: [u32; 3],
    /// double（`−0xcc`，位模式）。
    pub bar_double: f64,
    /// 3×int16（`−0xc4/−0xc2/−0xc0`，位模式）。
    pub bar_shorts: [u16; 3],
    /// 16×4 字节原样标量（偏移见 [`SCALEBAR_RAW_TAIL_OFFSETS`]）。
    pub raw_tail: [u32; 16],
    /// IFont 槽（`−0x6c`；字体候选）。
    pub font_a: SlotEntry10_1<StdFontBody10_1>,
    /// uint32 + int32（`−0x5c/−0x58`，位模式）。
    pub mid_pair: [u32; 2],
    /// IFont 槽（`−0x54`）。
    pub font_b: SlotEntry10_1<StdFontBody10_1>,
    /// uint32（`−0x44`，位模式）。
    pub tail_uint: Option<u32>,
    /// String（`−0x40`；数字格式串候选）。
    pub tail_string: String,
    /// 4×int32（`−0x3c/−0x38/−0x34/−0x30`，位模式）。
    pub tail_ints: [u32; 4],
    /// IFont 槽（`−0x2c`）。
    pub font_c: SlotEntry10_1<StdFontBody10_1>,
    /// 尾随 uint32（`−0x1c`，位模式）。
    pub last_uint: Option<u32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FamilyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 ScaleText（比例尺文本）对象体。
///
/// 字段序（Load `0x79457410` / Save `0x794576a0`）：① int16 自身版本
/// （有符号 >3 拒绝；Save 版本表 `DAT_79480d08`=[2,2,2,3] → 10.1 恒写 3）
/// → ② 共同前缀 → ③ 版本 <2（10.1 不可达）：String + ITextSymbol 槽
/// （经 +0x18/+0x10 虚表 setter 装入 `−0x38` 槽）→ ④ int32（`−0x34`）、
/// String（`−0x30`）、int32（`−0x2c`）、int32（`−0x28`）→ ⑤ 版本 <2：
/// int32 读入即丢；版本 ≥2：ITextElement 槽（`−0x38`，请求 IID
/// `{204034D2-…}`）、String（`−0x24`）、String（`−0x20`）、INumberFormat
/// 槽（`−0x18`，请求 IID `{7E4F470D-…}`）→ ⑥ 版本 ≥3：String（`−0x1c`，
/// 格式串候选；<3 时原生写内存默认，不读流）。
pub fn read_scale_text_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ScaleTextBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("ScaleText 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 1. 自身版本 short：有符号 `3 < v` 拒绝。
    let version = trial.read_u16_le()?;
    if (version as i16) > 3 || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出 ScaleText::Load 接受范围（有符号 \
             >3 拒绝 0x80004005；Save 版本表 DAT_79480d08 = [2,2,2,3]，10.1 恒写 3）"
        )));
    }

    // 2. 共同前缀。
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, table)?;

    // 3. 版本 <2 旧段（10.1 写侧恒写 3，不可达；实现保留分支）。
    let mut legacy_text = None;
    let mut legacy_symbol = SlotEntry10_1::Null;
    let mut stop = None;
    if version < 2 {
        legacy_text = Some(read_string_10_1(&mut trial, max_bytes)?);
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "legacy_symbol(ITextSymbol)",
            &TEXT_SYMBOL_CLASS_ID,
            |reader, table, _key| read_text_symbol_body_10_1(reader, max_bytes, table),
        )?;
        legacy_symbol = slot;
        stop = reason;
    }

    // 4. 恒有标量段。
    let mut raw_int32_a = None;
    let mut label = String::new();
    let mut raw_int32_b = None;
    let mut raw_int32_c = None;
    if stop.is_none() {
        raw_int32_a = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        label = read_string_10_1(&mut trial, max_bytes)?;
        raw_int32_b = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        raw_int32_c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
    }

    // 5. 版本分支。
    let mut legacy_discard = None;
    let mut text_element = SlotEntry10_1::Null;
    let mut format_prefix = String::new();
    let mut format_suffix = String::new();
    let mut number_format = SlotEntry10_1::Null;
    if stop.is_none() {
        if version < 2 {
            legacy_discard = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        } else {
            let (slot, reason) = read_typed_slot_10_1(
                &mut trial,
                max_bytes,
                table,
                "text_element(ITextElement)",
                &TEXT_ELEMENT_CLASS_ID,
                |reader, table, _key| read_text_element_body_10_1(reader, max_bytes, table),
            )?;
            text_element = slot;
            stop = reason;
            if stop.is_none() {
                format_prefix = read_string_10_1(&mut trial, max_bytes)?;
                format_suffix = read_string_10_1(&mut trial, max_bytes)?;
                let (slot, reason) = read_typed_slot_10_1(
                    &mut trial,
                    max_bytes,
                    table,
                    "number_format(INumberFormat)",
                    &NUMERIC_FORMAT_CLASS_ID,
                    |reader, table, slot_key| {
                        // NumericFormat 体无变长字段；槽位键来自槽位对象头，
                        // 与 feature_layer_tail 槽位分派同构。
                        read_numeric_format_body_10_1(reader, table, slot_key)
                    },
                )?;
                number_format = slot;
                stop = reason;
            }
        }
    }

    // 6. 版本 ≥3 格式串（<3 原生写内存默认，不读流）。
    let mut scale_format = None;
    if stop.is_none() && version >= 3 {
        scale_format = Some(read_string_10_1(&mut trial, max_bytes)?);
    }

    let remaining = trial.remaining();
    *reader = trial;
    Ok(ScaleTextBody10_1 {
        key,
        version,
        common,
        legacy_text,
        legacy_symbol,
        raw_int32_a,
        label,
        raw_int32_b,
        raw_int32_c,
        legacy_discard,
        text_element,
        format_prefix,
        format_suffix,
        number_format,
        scale_format,
        stop,
        remaining_bytes: remaining,
    })
}

/// ScaleText 对象体（Load `0x79457410` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleTextBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// int16 自身版本的无符号位模式（有符号 >3 拒绝；10.1 恒写 3；负值
    /// 拒绝较原生更严）。
    pub version: u16,
    /// MapSurround 族共同前缀。
    pub common: MapSurroundCommonPrefix10_1,
    /// 版本 <2 旧段文本（10.1 不可达；`None` = 未读取）。
    pub legacy_text: Option<String>,
    /// 版本 <2 旧段 ITextSymbol 槽（装入 −0x38 槽；10.1 不可达）。
    pub legacy_symbol: SlotEntry10_1<TextSymbolBody10_1>,
    /// int32（`−0x34`，位模式；写侧恒写）。
    pub raw_int32_a: Option<i32>,
    /// String（`−0x30`，文本候选）。
    pub label: String,
    /// int32（`−0x2c`，位模式）。
    pub raw_int32_b: Option<i32>,
    /// int32（`−0x28`，位模式）。
    pub raw_int32_c: Option<i32>,
    /// 版本 <2 读入即丢的 int32（10.1 不可达）。
    pub legacy_discard: Option<i32>,
    /// 版本 ≥2 的 ITextElement 槽（`−0x38`；文本元素候选）。
    pub text_element: SlotEntry10_1<TextElementBody10_1>,
    /// 版本 ≥2 的 String（`−0x24`，格式前缀候选）。
    pub format_prefix: String,
    /// 版本 ≥2 的 String（`−0x20`，格式后缀候选）。
    pub format_suffix: String,
    /// 版本 ≥2 的 INumberFormat 槽（`−0x18`；本读取器仅分派 NumericFormat）。
    pub number_format: SlotEntry10_1<NumericFormatBody10_1>,
    /// 版本 ≥3 的 String（`−0x1c`，格式串候选；<3 时原生写内存默认）。
    pub scale_format: Option<String>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FamilyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 MarkerNorthArrow（标记指北针）对象体。
///
/// 字段序（Load `0x79444b80` / Save `0x79444ca0`）：① int16 自身版本
/// （原生无拒绝门；Save 恒写 2（`DAT_79480d44`），本读取器仅接受 0..=2
/// 较原生更严）→ ② BaseNorthArrow 基类段（Load `0x793a3b90`）：int16
/// 基类版本（有符号 >4 拒绝；Save 版本表 `DAT_79480ea0`=[3,3,3,3,3,4] →
/// 10.1 恒写 4）→ 共同前缀 → double（`−0x2c`）→ double（`−0x24`）→
/// [基类版本 ≥2] double（`−0x1c`；<2 内存清零不读流）→ [≥3] IPoint 槽
/// （`−0x14`，请求 IID `{1B077915-…}`；<3 内存释放不读流）→ [≥4] int32
/// （`−0x34`；<4 内存清零）→ ③ 自身版本 <2（10.1 不可达）：int32 读入
/// 即丢 + IMarkerSymbol 槽（请求 IID `{E6BDAA7C-…}`，经虚表 setter 装入
/// `−0x1c`）→ ④ 自身版本 ≥2：IMarkerElement 槽（`−0x1c`，请求 IID
/// `{530FD711-…}`）。
pub fn read_marker_north_arrow_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<MarkerNorthArrowBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("MarkerNorthArrow 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 1. 自身版本 short：原生无拒绝门；仅接受 0..=2（Save 恒写 2，>2 与
    //    负值无字节佐证，一并拒绝较原生更严）。
    let version = trial.read_u16_le()?;
    if version > 2 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出本读取器接受范围 0..=2（Load \
             0x79444b80 无拒绝门；Save 恒写 2，其余值无字节佐证，拒绝）"
        )));
    }

    // 2. BaseNorthArrow 基类段。
    let base_version = trial.read_u16_le()?;
    if (base_version as i16) > MAX_BASE_NORTH_ARROW_VERSION || (base_version as i16) < 0 {
        return Err(invalid(format!(
            "BaseNorthArrow 版本位模式 {base_version}（0x{base_version:04x}）超出接受范围 \
             0..=4（Load 0x793a3b90 有符号 >4 拒绝 0x80004005；Save 版本表 DAT_79480ea0 \
             = [3,3,3,3,3,4]，10.1 恒写 4）"
        )));
    }
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, table)?;
    let base_double_a = trial.read_f64_le()?;
    let base_double_b = trial.read_f64_le()?;
    let mut base_double_c = None;
    if base_version >= 2 {
        base_double_c = Some(trial.read_f64_le()?);
    }
    let mut base_point: SlotEntry10_1<PointBody10_1> = SlotEntry10_1::Null;
    let mut stop = None;
    if base_version >= 3 {
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "base_point(IPoint)",
            &POINT_CLASS_ID,
            |reader, table, _key| read_point_body_10_1(reader, max_bytes, table),
        )?;
        base_point = slot;
        stop = reason;
    }
    let mut base_int32 = None;
    if stop.is_none() && base_version >= 4 {
        base_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
    }

    // 3. 自身版本分支。
    let mut legacy_marker = SymbolSlotFull10_1::Null;
    let mut marker_element: SlotEntry10_1<MarkerElementBody10_1> = SlotEntry10_1::Null;
    if stop.is_none() {
        if version < 2 {
            // 10.1 写侧不可达：int32 读入即丢，再归约符号槽内停止。
            let _ = trial.read_bytes(4)?;
            legacy_marker =
                read_symbol_slot_full_10_1(&mut trial, max_bytes, table, MAX_SYMBOL_NESTING_DEPTH)?;
            stop = slot_full_stop_10_1(&legacy_marker, DisplaySlotKind10_1::Symbol)
                .map(FamilyStop10_1::LegacyMarkerSymbol);
        } else {
            let (slot, reason) = read_typed_slot_10_1(
                &mut trial,
                max_bytes,
                table,
                "marker_element(IMarkerElement)",
                &MARKER_ELEMENT_CLASS_ID,
                |reader, table, _key| read_marker_element_body_10_1(reader, max_bytes, table),
            )?;
            stop = reason.or_else(|| match &slot {
                SlotEntry10_1::Body(body) if body.stop.is_some() => {
                    Some(FamilyStop10_1::MarkerElementIncomplete)
                }
                _ => None,
            });
            marker_element = slot;
        }
    }

    let _ = key;
    let remaining = trial.remaining();
    *reader = trial;
    Ok(MarkerNorthArrowBody10_1 {
        key,
        version,
        base_version,
        common,
        base_double_a,
        base_double_b,
        base_double_c,
        base_point,
        base_int32,
        legacy_marker,
        marker_element,
        stop,
        remaining_bytes: remaining,
    })
}

/// MarkerNorthArrow 对象体（Load `0x79444b80` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerNorthArrowBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// int16 自身版本的无符号位模式（原生无拒绝门；Save 恒写 2；读取器
    /// 仅接受 0..=2，较原生更严）。
    pub version: u16,
    /// BaseNorthArrow 基类版本的无符号位模式（有符号 >4 拒绝；10.1 恒写
    /// 4；负值拒绝较原生更严）。
    pub base_version: u16,
    /// MapSurround 族共同前缀。
    pub common: MapSurroundCommonPrefix10_1,
    /// 基类 double（`−0x2c`，位模式）。
    pub base_double_a: f64,
    /// 基类 double（`−0x24`，位模式）。
    pub base_double_b: f64,
    /// 基类版本 ≥2 的 double（`−0x1c`，位模式；<2 内存清零不读流）。
    pub base_double_c: Option<f64>,
    /// 基类版本 ≥3 的 IPoint 槽（`−0x14`；<3 不读流）。
    pub base_point: SlotEntry10_1<PointBody10_1>,
    /// 基类版本 ≥4 的 int32（`−0x34`，位模式；<4 不读流）。
    pub base_int32: Option<i32>,
    /// 自身版本 <2 的 IMarkerSymbol 槽（10.1 写侧不可达；停止向外传播）。
    pub legacy_marker: SymbolSlotFull10_1,
    /// 自身版本 ≥2 的 IMarkerElement 槽（`−0x1c`；标记元素候选）。
    pub marker_element: SlotEntry10_1<MarkerElementBody10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FamilyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 LegendFormat（图例格式）对象体。
///
/// 字段序（Load `0x793e0c50` / Save `0x793e0ec0`，逐段对称）：① int16
/// 版本（有符号 >2 拒绝；Save 版本表 `DAT_79480db8`=[1,1,1,1,1,2] →
/// 10.1 恒写 2）→ ② int16（`−0x8c`，原样保留）→ ③ ITextSymbol 槽
/// （`−0x88`，请求 IID `{A80B5E91-…}`；标题符号候选）→ ④ int32（`−0x84`）
/// → ⑤ double ×11（`−0x7c..−0x2c`）→ ⑥ ILinePatch 槽（`−0x14`，请求 IID
/// `{BEB4EDB7-…}`）→ ⑦ IAreaPatch 槽（`−0x10`，请求 IID `{BEB4EDB6-…}`）
/// → ⑧ int16（`−0xc`）→ ⑨ 版本 ≥2：double ×2（`−0x24/−0x1c`；<2 内存
/// 清零不读流）。
///
/// 接入点提示（写给整合轮）：本读取器即 `map_surround::read_legend_prefix_10_1`
/// 的 ILegendFormat 槽（`LegendStop10_1::LegendFormatBody` 停止点）与
/// LegendClassFormat 尾段族之外 Legend 体其余 ILegendFormat 槽的分派目标。
pub fn read_legend_format_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LegendFormatBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("LegendFormat 对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 1. 版本 short：有符号 `2 < v` 拒绝。
    let version = trial.read_u16_le()?;
    if (version as i16) > 2 || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出 LegendFormat::Load 接受范围（有符号 \
             >2 拒绝 0x80004005；Save 版本表 DAT_79480db8 = [1,1,1,1,1,2]，10.1 恒写 2）"
        )));
    }

    // 2. int16（−0x8c，原样保留）。
    let extra_short = trial.read_u16_le()?;

    // 3. ITextSymbol 槽（−0x88）。
    let (title_symbol, mut stop) = read_typed_slot_10_1(
        &mut trial,
        max_bytes,
        table,
        "title_symbol(ITextSymbol)",
        &TEXT_SYMBOL_CLASS_ID,
        |reader, table, _key| read_text_symbol_body_10_1(reader, max_bytes, table),
    )?;

    // 4. int32（−0x84）。
    let mut raw_int32 = None;
    if stop.is_none() {
        raw_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
    }

    // 5. double ×11（−0x7c..−0x2c）。
    let mut doubles = [0_f64; 11];
    if stop.is_none() {
        for slot in &mut doubles {
            *slot = trial.read_f64_le()?;
        }
    }

    // 6./7. ILinePatch / IAreaPatch 槽。
    let mut line_patch: SlotEntry10_1<PatchBody10_1> = SlotEntry10_1::Null;
    let mut area_patch: SlotEntry10_1<PatchBody10_1> = SlotEntry10_1::Null;
    if stop.is_none() {
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "line_patch(ILinePatch)",
            &LINE_PATCH_CLASS_ID,
            |reader, table, key| read_patch_body_10_1(reader, max_bytes, table, key),
        )?;
        line_patch = slot;
        stop = reason;
    }
    if stop.is_none() {
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "area_patch(IAreaPatch)",
            &AREA_PATCH_CLASS_ID,
            |reader, table, key| read_patch_body_10_1(reader, max_bytes, table, key),
        )?;
        area_patch = slot;
        stop = reason;
    }

    // 8. int16（−0xc）。
    let mut tail_short = None;
    if stop.is_none() {
        tail_short = Some(trial.read_u16_le()?);
    }

    // 9. 版本 ≥2：double ×2。
    let mut patch_doubles = None;
    if stop.is_none() && version >= 2 {
        let a = trial.read_f64_le()?;
        let b = trial.read_f64_le()?;
        patch_doubles = Some([a, b]);
    }

    let remaining = trial.remaining();
    *reader = trial;
    Ok(LegendFormatBody10_1 {
        key,
        version,
        extra_short,
        title_symbol,
        raw_int32,
        doubles,
        line_patch,
        area_patch,
        tail_short,
        patch_doubles,
        stop,
        remaining_bytes: remaining,
    })
}

/// LegendFormat 对象体（Load `0x793e0c50` 全部字段，体完全闭合）。
#[derive(Debug, Clone, PartialEq)]
pub struct LegendFormatBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// int16 版本的无符号位模式（有符号 >2 拒绝；10.1 恒写 2；负值拒绝
    /// 较原生更严）。
    pub version: u16,
    /// int16（`−0x8c`，原样位模式，语义未定）。
    pub extra_short: u16,
    /// ITextSymbol 槽（`−0x88`；标题符号候选）。
    pub title_symbol: SlotEntry10_1<TextSymbolBody10_1>,
    /// int32（`−0x84`，位模式）。
    pub raw_int32: Option<i32>,
    /// double ×11（`−0x7c..−0x2c`，位模式；索引 n 对应 `−0x7c + 8n`）。
    pub doubles: [f64; 11],
    /// ILinePatch 槽（`−0x14`；线补丁候选）。
    pub line_patch: SlotEntry10_1<PatchBody10_1>,
    /// IAreaPatch 槽（`−0x10`；面补丁候选）。
    pub area_patch: SlotEntry10_1<PatchBody10_1>,
    /// int16（`−0xc`，原样位模式）。
    pub tail_short: Option<u16>,
    /// 版本 ≥2 的 double ×2（`−0x24/−0x1c`）。
    pub patch_doubles: Option<[f64; 2]>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FamilyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 比例尺子类种类（流内布局仅中段填充符号个数不同：SingleDivision 1 个，
/// 其余 2 个；对象头 CLSID 由调用者比对对应常量区分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalebarSubclassKind10_1 {
    /// HollowScaleBar `{6589F143}`。
    Hollow,
    /// SingleDivisionScaleBar `{6589F146}`。
    SingleDivision,
    /// AlternatingScaleBar `{6589F147}`。
    Alternating,
    /// DoubleAlternatingScaleBar `{6589F148}`。
    DoubleAlternating,
}

impl ScalebarSubclassKind10_1 {
    /// 中段 IFillSymbol 槽个数（Hollow/Alt/DblAlt 共享 Load 读 2 个，
    /// SingleDivision 读 1 个）。
    pub fn fill_slot_count(self) -> usize {
        match self {
            Self::SingleDivision => 1,
            Self::Hollow | Self::Alternating | Self::DoubleAlternating => 2,
        }
    }
}

/// 从对象头之后的位置读取四个比例尺子类（Hollow/SingleDivision/Alternating/
/// DoubleAlternating）的对象体。
///
/// 四类共享同一层级布局：① int16 自身版本（有符号 >1 拒绝；Save 恒写 1）
/// → ② 基类段（Load `0x79451120`）：int16 基类版本（有符号 >6 拒绝；
/// Save 版本表 `DAT_79480d28`=[3,3,3,3,4,6] → 10.1 恒写 6）→ 共同前缀 →
/// double（`−0x17c`）→ int16 ×3（`−0x174/−0x172/−0x170`）→ int32（`−0x16c`）
/// → String（`−0x168`，数字格式串候选）→ int32（`−0x164`）→ double
/// （`−0x15c`）→ ITextSymbol 槽（`−0x154`）→ int32（`−0x150`）→ [基类
/// 版本 <2：int32 读入即丢（10.1 不可达）] int32（`−0x148`）→ double
/// （`−0x144`）→ ITextSymbol 槽（`−0x13c`）→ [≥2] INumberFormat 槽
/// （`−0x14c`）→ [≥3] int32（`−0x138`）→ [≥4] double（`−0xec`）→ [≥5]
/// int16（`−0xbc`）→ [≥6] int16（`−0xba`）→ ③ 中段：int16 中段版本
/// （有符号 >1 拒绝；Save 恒写 1）→ double（+8）→ [`kind`] 个 IFillSymbol
/// 槽（请求 IID `{E6BDAA7E-…}`）→ ④ 尾段：int16 尾段版本（有符号 >2
/// 拒绝；Save 恒写 2 `DAT_79480d10`）→ [<2] int16 标志（10.1 不可达）→
/// ILineSymbol 槽 ×2（`+4/+8`，请求 IID `{E6BDAA7D-…}`）→ double（+0x10）
/// → double（+0x18）→ int32（+0x20）→ [≥2] int32（+0x24；<2 原生按标志
/// 写内存默认 0/5，不读流）。
///
/// ISymbol 族槽经 [`read_symbol_slot_full_10_1`] 全量分派（未知符号类为
/// 槽内 `Unsupported` 枚举值，不整体停止——与其余槽的整体停止语义不同，
/// 与仓库既有符号槽约定一致）。
pub fn read_scalebar_subclass_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    kind: ScalebarSubclassKind10_1,
) -> io::Result<ScalebarSubclassBody10_1> {
    let start = reader.position();
    let mut trial = reader.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("比例尺子类对象体 offset 0x{start:x}: {reason}"),
        )
    };

    // 1. 自身版本 short：有符号 `1 < v` 拒绝。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_OWN_VERSION_1 || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出比例尺子类 Load 接受范围（有符号 \
             >1 拒绝 0x80004005；Save 恒写 1；负值位模式原生通过，此处一并拒绝）"
        )));
    }

    // 2. 基类段。
    let base_version = trial.read_u16_le()?;
    if (base_version as i16) > MAX_SCALEBAR_BASE_VERSION || (base_version as i16) < 0 {
        return Err(invalid(format!(
            "基类段版本位模式 {base_version}（0x{base_version:04x}）超出接受范围 0..=6 \
             （Load 0x79451120 有符号 >6 拒绝 0x80004005；Save 版本表 DAT_79480d28 = \
             [3,3,3,3,4,6]，10.1 恒写 6）"
        )));
    }
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, table)?;
    let base_double_a = trial.read_f64_le()?;
    let mut base_shorts = [0_u16; 3];
    for slot in &mut base_shorts {
        *slot = trial.read_u16_le()?;
    }
    let base_int32_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let base_number_string = read_string_10_1(&mut trial, max_bytes)?;
    let base_int32_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let base_double_b = trial.read_f64_le()?;
    let (label_symbol, mut stop) = read_typed_slot_10_1(
        &mut trial,
        max_bytes,
        table,
        "label_symbol(ITextSymbol)",
        &TEXT_SYMBOL_CLASS_ID,
        |reader, table, _key| read_text_symbol_body_10_1(reader, max_bytes, table),
    )?;
    let mut base_int32_c = None;
    let mut legacy_discard = None;
    let mut base_int32_d = None;
    let mut base_double_c = None;
    let mut second_symbol: SlotEntry10_1<TextSymbolBody10_1> = SlotEntry10_1::Null;
    let mut number_format: SlotEntry10_1<NumericFormatBody10_1> = SlotEntry10_1::Null;
    let mut tail_int32 = None;
    let mut tail_double = None;
    let mut tail_short_a = None;
    let mut tail_short_b = None;
    if stop.is_none() {
        base_int32_c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        if base_version < 2 {
            legacy_discard = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        base_int32_d = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        base_double_c = Some(trial.read_f64_le()?);
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "second_symbol(ITextSymbol)",
            &TEXT_SYMBOL_CLASS_ID,
            |reader, table, _key| read_text_symbol_body_10_1(reader, max_bytes, table),
        )?;
        second_symbol = slot;
        stop = reason;
    }
    if stop.is_none() && base_version >= 2 {
        let (slot, reason) = read_typed_slot_10_1(
            &mut trial,
            max_bytes,
            table,
            "number_format(INumberFormat)",
            &NUMERIC_FORMAT_CLASS_ID,
            |reader, table, slot_key| read_numeric_format_body_10_1(reader, table, slot_key),
        )?;
        number_format = slot;
        stop = reason;
    }
    if stop.is_none() {
        if base_version >= 3 {
            tail_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        }
        if base_version >= 4 {
            tail_double = Some(trial.read_f64_le()?);
        }
        if base_version >= 5 {
            tail_short_a = Some(trial.read_u16_le()?);
        }
        if base_version >= 6 {
            tail_short_b = Some(trial.read_u16_le()?);
        }
    }

    // 3. 中段：short + double + IFillSymbol 槽 ×N。
    let mut mid_version = None;
    let mut mid_double = None;
    let mut fill_symbols: Vec<SymbolSlotFull10_1> = Vec::new();
    if stop.is_none() {
        let mid = trial.read_u16_le()?;
        if (mid as i16) > MAX_SCALEBAR_MID_VERSION || (mid as i16) < 0 {
            return Err(invalid(format!(
                "中段版本位模式 {mid}（0x{mid:04x}）超出接受范围 0..=1（Load 0x793b3bf0/\
                 0x79459000 有符号 >1 拒绝 0x80004005；Save 恒写 1）"
            )));
        }
        mid_version = Some(mid);
        mid_double = Some(trial.read_f64_le()?);
        for index in 0..kind.fill_slot_count() {
            let _ = index;
            fill_symbols.push(read_symbol_slot_full_10_1(
                &mut trial,
                max_bytes,
                table,
                MAX_SYMBOL_NESTING_DEPTH,
            )?);
        }
    }

    // 4. 尾段：short + [<2] 标志 + ILineSymbol 槽 ×2 + double ×2 + int32 ×2。
    let mut tail_version = None;
    let mut legacy_flag = None;
    let mut line_symbols: Vec<SymbolSlotFull10_1> = Vec::new();
    let mut tail_doubles: Option<[f64; 2]> = None;
    let mut tail_ints: Option<[i32; 2]> = None;
    if stop.is_none() {
        let tail = trial.read_u16_le()?;
        if (tail as i16) > MAX_SCALEBAR_TAIL_VERSION || (tail as i16) < 0 {
            return Err(invalid(format!(
                "尾段版本位模式 {tail}（0x{tail:04x}）超出接受范围 0..=2（Load 0x79454ee0 \
                 有符号 >2 拒绝 0x80004005；Save 恒写 2）"
            )));
        }
        tail_version = Some(tail);
        if tail < 2 {
            legacy_flag = Some(trial.read_u16_le()?);
        }
        for _ in 0..2 {
            let slot =
                read_symbol_slot_full_10_1(&mut trial, max_bytes, table, MAX_SYMBOL_NESTING_DEPTH)?;
            line_symbols.push(slot);
        }
        let a = trial.read_f64_le()?;
        let b = trial.read_f64_le()?;
        tail_doubles = Some([a, b]);
        let c = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        let d = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
        tail_ints = Some([c, d]);
    }

    let remaining = trial.remaining();
    *reader = trial;
    Ok(ScalebarSubclassBody10_1 {
        key,
        kind,
        version,
        base_version,
        common,
        base_double_a,
        base_shorts,
        base_int32_a,
        base_number_string,
        base_int32_b,
        base_double_b,
        label_symbol,
        base_int32_c,
        legacy_discard,
        base_int32_d,
        base_double_c,
        second_symbol,
        number_format,
        tail_int32,
        tail_double,
        tail_short_a,
        tail_short_b,
        mid_version,
        mid_double,
        fill_symbols,
        tail_version,
        legacy_flag,
        line_symbols,
        tail_doubles,
        tail_ints,
        stop,
        remaining_bytes: remaining,
    })
}

/// 比例尺子类对象体（Hollow/SingleDivision/Alternating/DoubleAlternating
/// 共享层级布局）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScalebarSubclassBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 子类种类（决定中段填充符号槽个数；CLSID 由调用者比对）。
    pub kind: ScalebarSubclassKind10_1,
    /// int16 自身版本的无符号位模式（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// 基类段版本的无符号位模式（有符号 >6 拒绝；10.1 恒写 6）。
    pub base_version: u16,
    /// MapSurround 族共同前缀。
    pub common: MapSurroundCommonPrefix10_1,
    /// 基类 double（`−0x17c`，位模式）。
    pub base_double_a: f64,
    /// 基类 int16 ×3（`−0x174/−0x172/−0x170`，位模式）。
    pub base_shorts: [u16; 3],
    /// 基类 int32（`−0x16c`，位模式）。
    pub base_int32_a: i32,
    /// 基类 String（`−0x168`，数字格式串候选）。
    pub base_number_string: String,
    /// 基类 int32（`−0x164`，位模式）。
    pub base_int32_b: i32,
    /// 基类 double（`−0x15c`，位模式）。
    pub base_double_b: f64,
    /// 基类 ITextSymbol 槽（`−0x154`；标签符号候选）。
    pub label_symbol: SlotEntry10_1<TextSymbolBody10_1>,
    /// 基类 int32（`−0x150`，位模式）。
    pub base_int32_c: Option<i32>,
    /// 基类版本 <2 读入即丢的 int32（10.1 不可达）。
    pub legacy_discard: Option<i32>,
    /// 基类 int32（`−0x148`，位模式）。
    pub base_int32_d: Option<i32>,
    /// 基类 double（`−0x144`，位模式）。
    pub base_double_c: Option<f64>,
    /// 基类第二 ITextSymbol 槽（`−0x13c`）。
    pub second_symbol: SlotEntry10_1<TextSymbolBody10_1>,
    /// 基类版本 ≥2 的 INumberFormat 槽（`−0x14c`）。
    pub number_format: SlotEntry10_1<NumericFormatBody10_1>,
    /// 基类版本 ≥3 的 int32（`−0x138`，位模式）。
    pub tail_int32: Option<i32>,
    /// 基类版本 ≥4 的 double（`−0xec`，位模式）。
    pub tail_double: Option<f64>,
    /// 基类版本 ≥5 的 int16（`−0xbc`，位模式）。
    pub tail_short_a: Option<u16>,
    /// 基类版本 ≥6 的 int16（`−0xba`，位模式）。
    pub tail_short_b: Option<u16>,
    /// 中段版本的无符号位模式（有符号 >1 拒绝；Save 恒写 1）。
    pub mid_version: Option<u16>,
    /// 中段 double（+8，位模式）。
    pub mid_double: Option<f64>,
    /// 中段 IFillSymbol 槽（`+0x10`、`+0x14`；个数见
    /// [`ScalebarSubclassKind10_1::fill_slot_count`]）。
    pub fill_symbols: Vec<SymbolSlotFull10_1>,
    /// 尾段版本的无符号位模式（有符号 >2 拒绝；Save 恒写 2）。
    pub tail_version: Option<u16>,
    /// 尾段版本 <2 的 int16 标志（10.1 不可达；≥2 时原生置 +0x24 内存
    /// 默认 0/5）。
    pub legacy_flag: Option<u16>,
    /// 尾段 ILineSymbol 槽 ×2（`+4/+8`；刻度线符号候选）。
    pub line_symbols: Vec<SymbolSlotFull10_1>,
    /// 尾段 double ×2（`+0x10/+0x18`，位模式）。
    pub tail_doubles: Option<[f64; 2]>,
    /// 尾段 int32 ×2（`+0x20/+0x24`，位模式）。
    pub tail_ints: Option<[i32; 2]>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FamilyStop10_1>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 两个具体类共享的读取入口类别；由调用者核对对象头 CLSID 后选择。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleLineKind10_1 {
    ScaleLine,
    SteppedScaleLine,
}

/// 已消费的字段；未定名标量以原始小端字节保留，顺序与原生流一致。
#[derive(Debug, Clone, PartialEq)]
pub enum ScaleLineField10_1 {
    Raw { name: &'static str, bytes: Vec<u8> },
    Text { name: &'static str, value: String },
    TextSymbol(SlotEntry10_1<TextSymbolBody10_1>),
    NumberFormat(SlotEntry10_1<NumericFormatBody10_1>),
    LineSymbol(SymbolSlotFull10_1),
}

/// 未闭合对象槽：整个槽及其新增登记均回滚，偏移指向槽头。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleLineStop10_1 {
    Object {
        slot: &'static str,
        offset: usize,
        class_id: [u8; 16],
        key: u32,
    },
    Symbol {
        slot: &'static str,
        offset: usize,
        reason: SymbolStop10_1,
    },
}

/// 已确认的比例尺线对象体；`stop == None` 才表示完整消费。
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleLineBody10_1 {
    pub key: u32,
    pub kind: ScaleLineKind10_1,
    pub version: u16,
    pub base_version: u16,
    pub common: MapSurroundCommonPrefix10_1,
    pub fields: Vec<ScaleLineField10_1>,
    pub stop: Option<ScaleLineStop10_1>,
    pub remaining_bytes: usize,
}

fn scale_line_version(reader: &mut Reader<'_>, max: u16, label: &str) -> io::Result<u16> {
    let offset = reader.position();
    let value = reader.read_u16_le()?;
    // 负值原生比较可通过，但不属于已确认的写侧版本。
    if (value as i16) < 0 || value > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{label} 版本 {value} 在 0x{offset:x} 超出 0..={max}"),
        ));
    }
    Ok(value)
}

fn scale_line_object<T>(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    slot: &'static str,
    expected: &[u8; 16],
    body_reader: impl FnOnce(&mut Reader<'_>, &mut RefTable10_1, u32) -> io::Result<(T, bool)>,
) -> io::Result<Result<SlotEntry10_1<T>, ScaleLineStop10_1>> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let result = match entry {
        ObjectEntry10_1::Null => SlotEntry10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            SlotEntry10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if class_id != *expected {
                return Ok(Err(ScaleLineStop10_1::Object {
                    slot,
                    offset: reader.position(),
                    class_id,
                    key,
                }));
            }
            let (body, complete) = body_reader(&mut trial, &mut staged, key)?;
            if !complete {
                return Ok(Err(ScaleLineStop10_1::Object {
                    slot,
                    offset: reader.position(),
                    class_id,
                    key,
                }));
            }
            SlotEntry10_1::Body(Box::new(body))
        }
    };
    *reader = trial;
    *table = staged;
    Ok(Ok(result))
}

fn scale_line_symbol(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    slot: &'static str,
) -> io::Result<Result<SymbolSlotFull10_1, ScaleLineStop10_1>> {
    use crate::objects::display_symbol::{
        CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID, HASH_LINE_SYMBOL_CLASS_ID, MARKER_LINE_SYMBOL_CLASS_ID,
        MULTI_LAYER_LINE_SYMBOL_CLASS_ID, PICTURE_LINE_SYMBOL_CLASS_ID,
        SIMPLE_LINE_SYMBOL_CLASS_ID,
    };
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let mut probe = trial.clone();
    let mut probe_table = staged.clone();
    if let ObjectEntry10_1::NewObject { class_id, key } =
        read_object_entry_10_1(&mut probe, &mut probe_table)?
    {
        if ![
            SIMPLE_LINE_SYMBOL_CLASS_ID,
            MARKER_LINE_SYMBOL_CLASS_ID,
            CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID,
            HASH_LINE_SYMBOL_CLASS_ID,
            MULTI_LAYER_LINE_SYMBOL_CLASS_ID,
            PICTURE_LINE_SYMBOL_CLASS_ID,
        ]
        .contains(&class_id)
        {
            return Ok(Err(ScaleLineStop10_1::Object {
                slot,
                offset: reader.position(),
                class_id,
                key,
            }));
        }
    }
    let symbol =
        read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut staged, MAX_SYMBOL_NESTING_DEPTH)?;
    if let Some(reason) = slot_full_stop_10_1(&symbol, DisplaySlotKind10_1::Symbol) {
        return Ok(Err(ScaleLineStop10_1::Symbol {
            slot,
            offset: reader.position(),
            reason,
        }));
    }
    *reader = trial;
    *table = staged;
    Ok(Ok(symbol))
}

/// 读取 ScaleLine 或 SteppedScaleLine 的对象体；对象头须由调用者读取。
///
/// 依据 Map.dll 10.1 的 `0x794537f0` / `0x794682d0`，证据及旧版分支见
/// `docs/mxd-terra-nextpass-scale.md`。错误回滚整个调用；遇到未知或未完成的
/// 嵌套体时返回停止原因，回滚整个对应槽，之前完整字段和登记保留。
pub fn read_scale_line_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    kind: ScaleLineKind10_1,
) -> io::Result<ScaleLineBody10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = scale_line_version(&mut trial, 1, "ScaleLine")?;
    let base_version = scale_line_version(&mut trial, 6, "ScaleLine 基类")?;
    let common = read_map_surround_common_prefix_10_1(&mut trial, max_bytes, &mut staged)?;
    let mut fields = Vec::new();
    let mut stop = None;
    // 闭包在首次未闭合槽停止；外层统一提交已完成前缀。
    (|| -> io::Result<()> {
        macro_rules! raw {
            ($name:expr, $len:expr) => {
                fields.push(ScaleLineField10_1::Raw {
                    name: $name,
                    bytes: trial.read_bytes($len)?.to_vec(),
                })
            };
        }
        macro_rules! object {
            ($result:expr, $variant:ident) => {
                match $result? {
                    Ok(value) => fields.push(ScaleLineField10_1::$variant(value)),
                    Err(reason) => {
                        stop = Some(reason);
                        return Ok(());
                    }
                }
            };
        }
        raw!("base_double_a", 8);
        raw!("base_shorts", 6);
        raw!("base_int_a", 4);
        fields.push(ScaleLineField10_1::Text {
            name: "base_string",
            value: read_string_10_1(&mut trial, max_bytes)?,
        });
        raw!("base_int_b", 4);
        raw!("base_double_b", 8);
        object!(
            scale_line_object(
                &mut trial,
                &mut staged,
                "label_symbol",
                &TEXT_SYMBOL_CLASS_ID,
                |r, t, _| {
                    let body = read_text_symbol_body_10_1(r, max_bytes, t)?;
                    let complete = body.stop.is_none();
                    Ok((body, complete))
                }
            ),
            TextSymbol
        );
        raw!("base_int_c", 4);
        if base_version < 2 {
            raw!("base_legacy_discard", 4);
        }
        raw!("base_int_d", 4);
        raw!("base_double_c", 8);
        object!(
            scale_line_object(
                &mut trial,
                &mut staged,
                "second_symbol",
                &TEXT_SYMBOL_CLASS_ID,
                |r, t, _| {
                    let body = read_text_symbol_body_10_1(r, max_bytes, t)?;
                    let complete = body.stop.is_none();
                    Ok((body, complete))
                }
            ),
            TextSymbol
        );
        if base_version >= 2 {
            object!(
                scale_line_object(
                    &mut trial,
                    &mut staged,
                    "number_format",
                    &NUMERIC_FORMAT_CLASS_ID,
                    |r, t, k| Ok((read_numeric_format_body_10_1(r, t, k)?, true))
                ),
                NumberFormat
            );
        }
        if base_version >= 3 {
            raw!("base_tail_int", 4);
        }
        if base_version >= 4 {
            raw!("base_tail_double", 8);
        }
        if base_version >= 5 {
            raw!("base_tail_short_a", 2);
        }
        if base_version >= 6 {
            raw!("base_tail_short_b", 2);
        }
        let mid = scale_line_version(&mut trial, 1, "ScaleLineImpl")?;
        fields.push(ScaleLineField10_1::Raw {
            name: "line_version",
            bytes: mid.to_le_bytes().to_vec(),
        });
        object!(
            scale_line_symbol(&mut trial, &mut staged, max_bytes, "line_symbol"),
            LineSymbol
        );
        let tail = scale_line_version(&mut trial, 2, "ScaleLine 尾段")?;
        fields.push(ScaleLineField10_1::Raw {
            name: "tail_version",
            bytes: tail.to_le_bytes().to_vec(),
        });
        if tail < 2 {
            raw!("tail_legacy_flag", 2);
        }
        object!(
            scale_line_symbol(&mut trial, &mut staged, max_bytes, "tail_symbol_a"),
            LineSymbol
        );
        object!(
            scale_line_symbol(&mut trial, &mut staged, max_bytes, "tail_symbol_b"),
            LineSymbol
        );
        raw!("tail_double_a", 8);
        raw!("tail_double_b", 8);
        raw!("tail_int_a", 4);
        if tail >= 2 {
            raw!("tail_int_b", 4);
        }
        if kind == ScaleLineKind10_1::SteppedScaleLine {
            raw!("stepped_double", 8);
        }
        Ok(())
    })()?;
    let remaining_bytes = trial.remaining();
    *reader = trial;
    *table = staged;
    Ok(ScaleLineBody10_1 {
        key,
        kind,
        version,
        base_version,
        common,
        fields,
        stop,
        remaining_bytes,
    })
}

/// ScaleLine 专用入口。
pub fn read_scale_line_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ScaleLineBody10_1> {
    read_scale_line_body_10_1(reader, max_bytes, table, key, ScaleLineKind10_1::ScaleLine)
}

/// SteppedScaleLine 专用入口。
pub fn read_stepped_scale_line_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ScaleLineBody10_1> {
    read_scale_line_body_10_1(
        reader,
        max_bytes,
        table,
        key,
        ScaleLineKind10_1::SteppedScaleLine,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::patch::PatchGeometrySlot10_1;
    use crate::objects::geometry::{GeometryBody10_1, POLYLINE_CLASS_ID};

    /// Map 类标识（CLSID，回引用对象头用）：`{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`。
    const MAP_CLSID: [u8; 16] = [
        0x76, 0xaa, 0xbd, 0xe6, 0x35, 0x4d, 0xd0, 0x11, 0x98, 0xbe, 0x00, 0x80, 0x5f, 0x7c, 0xed,
        0x21,
    ];

    fn string_bytes(text: &str) -> Vec<u8> {
        let mut units: Vec<u16> = text.encode_utf16().collect();
        units.push(0);
        let mut bytes = ((units.len() * 2) as u32).to_le_bytes().to_vec();
        for unit in units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes
    }

    fn backref_header(class_id: &[u8; 16], key: u32) -> Vec<u8> {
        let mut bytes = class_id.to_vec();
        bytes.extend_from_slice(&key.to_le_bytes());
        bytes
    }

    /// MapSurround 族共同前缀（版本 3）：short 3 + 名称 + IMap 回引 + short + 32 字节。
    fn common_prefix_bytes() -> Vec<u8> {
        let mut bytes = 3u16.to_le_bytes().to_vec();
        bytes.extend_from_slice(&string_bytes("装饰物"));
        bytes.extend_from_slice(&backref_header(&MAP_CLSID, 30));
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 32]);
        bytes
    }

    /// 最小 StdFont 体（版本 1 + charset + flags + weight + size + 名字）。
    fn std_font_bytes() -> Vec<u8> {
        let mut bytes = vec![1u8];
        bytes.extend_from_slice(&134u16.to_le_bytes());
        bytes.push(0u8);
        bytes.extend_from_slice(&400u16.to_le_bytes());
        bytes.extend_from_slice(&120000i32.to_le_bytes());
        bytes.push(6u8);
        bytes.extend_from_slice(b"SimSun");
        bytes
    }

    /// 最小 NumericFormat 体（版本 1 + int32 ×4 + short ×3）。
    fn numeric_format_bytes() -> Vec<u8> {
        let mut bytes = 1u16.to_le_bytes().to_vec();
        for value in [1i32, 2, 3, 4] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [5i16, 6, 7] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    /// 最小闭合 Polyline 几何对象头 + 体（版本 2 + 44+4+16 字节记录 + 空标记）。
    fn polyline_object_entry(key: u32) -> Vec<u8> {
        let record = {
            let mut bytes = 3u32.to_le_bytes().to_vec();
            for value in [0.0f64, 0.0, 1.0, 1.0] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend(1u32.to_le_bytes());
            bytes.extend(2u32.to_le_bytes());
            bytes.extend(0u32.to_le_bytes());
            bytes.extend(0.5f64.to_le_bytes());
            bytes.extend(0.5f64.to_le_bytes());
            bytes.extend(0.25f64.to_le_bytes());
            bytes.extend(0.75f64.to_le_bytes());
            bytes
        };
        let mut bytes = POLYLINE_CLASS_ID.to_vec();
        bytes.extend_from_slice(&key.to_le_bytes());
        bytes.extend(2u16.to_le_bytes());
        bytes.extend((record.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&record);
        bytes.extend_from_slice(&[0u8; 16]);
        bytes
    }

    /// 在 `data` 的 `at` 偏移处替换为对象头 `class_id`+`key`（20 字节）。
    fn replace_entry(data: &mut [u8], at: usize, class_id: &[u8; 16], key: u32) {
        data[at..at + 16].copy_from_slice(class_id);
        data[at + 16..at + 20].copy_from_slice(&key.to_le_bytes());
    }

    fn read_body<T>(
        data: &[u8],
        read: impl FnOnce(&mut Reader<'_>, usize, &mut RefTable10_1, u32) -> io::Result<T>,
    ) -> io::Result<T> {
        let mut table = RefTable10_1::default();
        // 共同前缀的关联地图槽是键 30 回引（登记先于持久化）：预登记。
        table.register(30);
        let mut reader = Reader::new(data);
        reader.read_bytes(20).unwrap();
        read(&mut reader, 4096, &mut table, 5)
    }

    /// GUID 文本 → 内存序字节（Data1-3 小端、Data4 原序）的黄金样表，
    /// 与各常量文档注释中的字串逐一对应。
    fn documented_guid_bytes() -> Vec<(&'static str, [u8; 16])> {
        vec![
            (
                "{7A3F91E5-B9E3-11D1-8756-0000F8751720}",
                LEGEND_FORMAT_CLASS_ID,
            ),
            ("{7A3F91DB-B9E3-11D1-8756-0000F8751720}", SCALEBAR_CLASS_ID),
            (
                "{7A3F91DC-B9E3-11D1-8756-0000F8751720}",
                SCALE_TEXT_CLASS_ID,
            ),
            (
                "{7A3F91DD-B9E3-11D1-8756-0000F8751720}",
                MARKER_NORTH_ARROW_CLASS_ID,
            ),
            ("{7A3F91DF-B9E3-11D1-8756-0000F8751720}", MAP_TITLE_CLASS_ID),
            (
                "{6589F143-F7F7-11D2-B872-00600802E603}",
                HOLLOW_SCALEBAR_CLASS_ID,
            ),
            (
                "{6589F146-F7F7-11D2-B872-00600802E603}",
                SINGLE_DIVISION_SCALEBAR_CLASS_ID,
            ),
            (
                "{6589F147-F7F7-11D2-B872-00600802E603}",
                ALTERNATING_SCALEBAR_CLASS_ID,
            ),
            (
                "{6589F148-F7F7-11D2-B872-00600802E603}",
                DOUBLE_ALTERNATING_SCALEBAR_CLASS_ID,
            ),
        ]
    }

    #[test]
    fn class_id_constants_match_documented_guids() {
        // 证据见各常量文档注释（GetClassID 指令写出 + esri.clsid 声明）；
        // 黄金样表手工编码自 GetClassID 常量字节（read_guids.py 复核）。
        let expected: Vec<(&str, [u8; 16])> = vec![
            (
                "LegendFormat",
                [
                    0xe5, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8,
                    0x75, 0x17, 0x20,
                ],
            ),
            (
                "Scalebar",
                [
                    0xdb, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8,
                    0x75, 0x17, 0x20,
                ],
            ),
            (
                "ScaleText",
                [
                    0xdc, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8,
                    0x75, 0x17, 0x20,
                ],
            ),
            (
                "MarkerNorthArrow",
                [
                    0xdd, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8,
                    0x75, 0x17, 0x20,
                ],
            ),
            (
                "MapTitle",
                [
                    0xdf, 0x91, 0x3f, 0x7a, 0xe3, 0xb9, 0xd1, 0x11, 0x87, 0x56, 0x00, 0x00, 0xf8,
                    0x75, 0x17, 0x20,
                ],
            ),
            (
                "HollowScaleBar",
                [
                    0x43, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08,
                    0x02, 0xe6, 0x03,
                ],
            ),
            (
                "SingleDivisionScaleBar",
                [
                    0x46, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08,
                    0x02, 0xe6, 0x03,
                ],
            ),
            (
                "AlternatingScaleBar",
                [
                    0x47, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08,
                    0x02, 0xe6, 0x03,
                ],
            ),
            (
                "DoubleAlternatingScaleBar",
                [
                    0x48, 0xf1, 0x89, 0x65, 0xf7, 0xf7, 0xd2, 0x11, 0xb8, 0x72, 0x00, 0x60, 0x08,
                    0x02, 0xe6, 0x03,
                ],
            ),
        ];
        let table = documented_guid_bytes();
        for ((_name, guid), (doc_name, bytes)) in table.iter().zip(expected.iter()) {
            assert_eq!(guid, bytes, "{doc_name} 常量字节与证据不符");
        }
        // 自洽性：九个常量互不相同且与文档注释字串一一对应（黄金样表
        // documented_guid_bytes 覆盖全部九类）。
        assert_eq!(table.len(), 9);
        for i in 0..table.len() {
            for j in i + 1..table.len() {
                assert_ne!(table[i].1, table[j].1);
            }
        }
    }

    #[test]
    fn reads_map_title_full_body() {
        // 版本 1 + 共同前缀（版本 3）。
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(1u16.to_le_bytes());
        data.extend_from_slice(&common_prefix_bytes());
        let body = read_body(&data, read_map_title_body_10_1).expect("MapTitle 体");
        assert_eq!(body.key, 5);
        assert_eq!(body.version, 1);
        assert_eq!(body.common.version, 3);
        assert_eq!(body.common.name, "装饰物");
        assert_eq!(body.common.extra_short, Some(0));
        assert!(body.common.raw_envelope.is_some());
    }

    #[test]
    fn rejects_map_title_version_out_of_range() {
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(2u16.to_le_bytes()); // >1 拒绝
        data.extend_from_slice(&common_prefix_bytes());
        assert!(read_body(&data, read_map_title_body_10_1).is_err());
        let mut negative = data;
        negative[20] = 0xff;
        negative[21] = 0xff;
        assert!(read_body(&negative, read_map_title_body_10_1).is_err());
    }

    /// Scalebar 完整体（版本 1 + 前缀 + 标量段 + 3 个 StdFont 槽 + 尾段）。
    fn scalebar_full_bytes() -> Vec<u8> {
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(1u16.to_le_bytes());
        data.extend_from_slice(&common_prefix_bytes());
        for _ in 0..3 {
            data.extend(7i32.to_le_bytes());
        }
        data.extend(2.5f64.to_le_bytes());
        for _ in 0..3 {
            data.extend(1u16.to_le_bytes());
        }
        for _ in 0..16 {
            data.extend(0x11111111u32.to_le_bytes());
        }
        // IFont 槽 A（StdFont 新对象）。
        data.extend_from_slice(&crate::objects::ole_font::STDFONT_CLASS_ID);
        data.extend(11u32.to_le_bytes());
        data.extend_from_slice(&std_font_bytes());
        // uint32 + int32。
        data.extend(0x22222222u32.to_le_bytes());
        data.extend(8i32.to_le_bytes());
        // IFont 槽 B（空标记）。
        data.extend_from_slice(&[0u8; 16]);
        // uint32 + String + 4×int32。
        data.extend(0x33333333u32.to_le_bytes());
        data.extend_from_slice(&string_bytes("0.1"));
        for _ in 0..4 {
            data.extend(9i32.to_le_bytes());
        }
        // IFont 槽 C（StdFont 新对象）。
        data.extend_from_slice(&crate::objects::ole_font::STDFONT_CLASS_ID);
        data.extend(12u32.to_le_bytes());
        data.extend_from_slice(&std_font_bytes());
        // 尾随 uint32。
        data.extend(0x44444444u32.to_le_bytes());
        data
    }

    #[test]
    fn reads_scalebar_full_body() {
        let data = scalebar_full_bytes();
        let body = read_body(&data, read_scalebar_body_10_1).expect("Scalebar 体");
        assert_eq!(body.version, 1);
        assert_eq!(body.common.name, "装饰物");
        assert_eq!(body.head_ints, [7; 3]);
        assert_eq!(body.bar_double, 2.5);
        assert_eq!(body.raw_tail, [0x11111111; 16]);
        match &body.font_a {
            SlotEntry10_1::Body(font) => {
                assert_eq!(font.weight, 400);
                assert_eq!(font.size_cy_lo, 120000);
            }
            other => panic!("font_a 应为 Body，得到 {other:?}"),
        }
        assert!(matches!(body.font_b, SlotEntry10_1::Null));
        assert_eq!(body.tail_string, "0.1");
        assert!(matches!(body.font_c, SlotEntry10_1::Body(_)));
        assert_eq!(body.last_uint, Some(0x44444444));
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn scalebar_font_slot_unsupported_stops_at_slot_start() {
        let mut data = scalebar_full_bytes();
        // 把槽 A 类标识换成未知类。
        let slot_a_offset = 20 + 2 + common_prefix_bytes().len() + (4 * 3) + 8 + (2 * 3) + 4 * 16;
        replace_entry(&mut data, slot_a_offset, &[0x55; 16], 99);
        let body = read_body(&data, read_scalebar_body_10_1).expect("停止是成功返回");
        match body.font_a {
            SlotEntry10_1::Unsupported { key, .. } => assert_eq!(key, 99),
            other => panic!("应为 Unsupported，得到 {other:?}"),
        }
        assert_eq!(body.mid_pair, [0; 2]);
        assert!(matches!(body.font_b, SlotEntry10_1::Null));
        assert_eq!(body.tail_string, "");
        assert_eq!(body.last_uint, None);
        assert!(matches!(
            body.stop,
            Some(FamilyStop10_1::SlotUnsupported { .. })
        ));
    }

    /// ScaleText 完整体（版本 3 + 前缀 + 标量段 + TextElement 空标记 +
    /// 两个字符串 + NumericFormat 新对象 + 格式串）。
    fn scale_text_v3_bytes() -> Vec<u8> {
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(3u16.to_le_bytes());
        data.extend_from_slice(&common_prefix_bytes());
        data.extend(1i32.to_le_bytes());
        data.extend_from_slice(&string_bytes("1:1000"));
        data.extend(2i32.to_le_bytes());
        data.extend(3i32.to_le_bytes());
        // TextElement 槽（空标记）。
        data.extend_from_slice(&[0u8; 16]);
        data.extend_from_slice(&string_bytes("%"));
        data.extend_from_slice(&string_bytes("m"));
        // NumericFormat 槽（新对象）。
        data.extend_from_slice(&NUMERIC_FORMAT_CLASS_ID);
        data.extend(21u32.to_le_bytes());
        data.extend_from_slice(&numeric_format_bytes());
        // 版本 ≥3 格式串。
        data.extend_from_slice(&string_bytes("1:%s"));
        data
    }

    #[test]
    fn reads_scale_text_v3_body() {
        let data = scale_text_v3_bytes();
        let body = read_body(&data, read_scale_text_body_10_1).expect("ScaleText 体");
        assert_eq!(body.version, 3);
        assert_eq!(body.label, "1:1000");
        assert!(body.legacy_text.is_none());
        assert!(matches!(body.text_element, SlotEntry10_1::Null));
        assert_eq!(body.format_prefix, "%");
        assert_eq!(body.format_suffix, "m");
        match &body.number_format {
            SlotEntry10_1::Body(nf) => assert_eq!(nf.int32s, [1, 2, 3, 4]),
            other => panic!("number_format 应为 NumericFormat，得到 {other:?}"),
        }
        assert_eq!(body.scale_format.as_deref(), Some("1:%s"));
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn rejects_scale_text_version_out_of_range() {
        let mut data = scale_text_v3_bytes();
        data[20] = 4;
        assert!(read_body(&data, read_scale_text_body_10_1).is_err());
    }

    /// MarkerNorthArrow 完整体（版本 2 + 基类版本 4 + 前缀 + 基类字段 +
    /// MarkerElement 槽空标记）。
    fn mna_v2_bytes() -> Vec<u8> {
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(2u16.to_le_bytes()); // 自身版本 ≥2
        data.extend(4u16.to_le_bytes()); // 基类版本
        data.extend_from_slice(&common_prefix_bytes());
        data.extend(1.0f64.to_le_bytes());
        data.extend(2.0f64.to_le_bytes());
        data.extend(3.0f64.to_le_bytes()); // 基类 ≥2 double
        data.extend_from_slice(&[0u8; 16]); // 基类 ≥3 IPoint 槽（空标记）
        data.extend(4i32.to_le_bytes()); // 基类 ≥4 int32
        // IMarkerElement 槽（空标记）。
        data.extend_from_slice(&[0u8; 16]);
        data
    }

    #[test]
    fn reads_marker_north_arrow_v2_body() {
        let data = mna_v2_bytes();
        let body = read_body(&data, read_marker_north_arrow_body_10_1).expect("MNA 体");
        assert_eq!(body.version, 2);
        assert_eq!(body.base_version, 4);
        assert_eq!(body.base_double_c, Some(3.0));
        assert!(matches!(body.base_point, SlotEntry10_1::Null));
        assert_eq!(body.base_int32, Some(4));
        assert!(matches!(body.marker_element, SlotEntry10_1::Null));
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    /// 元素共同前缀版本 6；VT_EMPTY、空名称与类型字符串。
    fn marker_element_prefix() -> Vec<u8> {
        let mut bytes = 3u16.to_le_bytes().to_vec();
        bytes.extend(6u16.to_le_bytes());
        bytes.extend(string_bytes(""));
        bytes.extend(string_bytes("Marker"));
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes.extend(0f64.to_le_bytes());
        bytes.extend(0i32.to_le_bytes());
        bytes
    }

    #[test]
    fn marker_north_arrow_propagates_element_stop_to_map_suffix() {
        use crate::maps::map_object::{MapSuffixStop, read_map_suffix_10_1};
        use crate::objects::element_body::ElementBodyStop10_1;
        let mut data = 51u16.to_le_bytes().to_vec();
        data.extend(string_bytes("Map"));
        data.extend(string_bytes(""));
        data.extend([0u8; 16]); // 地图空间参考为空。
        data.extend(0i32.to_le_bytes()); // 空图层列表。
        data.extend((-1i32).to_le_bytes());
        data.extend([0u8; 16]); // 基本图形层为空。
        data.extend(1i32.to_le_bytes());
        let surround_offset = data.len();
        data.extend(MARKER_NORTH_ARROW_CLASS_ID);
        data.extend(41u32.to_le_bytes());
        let arrow = mna_v2_bytes();
        data.extend_from_slice(&arrow[20..arrow.len() - 16]);
        data.extend(MARKER_ELEMENT_CLASS_ID);
        data.extend(42u32.to_le_bytes());
        data.extend(marker_element_prefix());
        let symbol_offset = data.len();
        data.extend([0x77u8; 16]); // 未知 IMarkerSymbol 新对象。
        data.extend(43u32.to_le_bytes());
        data.extend([0u8; 64]); // 确保错误继续时能观察是否读取后续字段。

        let mut table = RefTable10_1::new();
        table.register(30); // 共同前缀中的地图回引。
        let mut reader = Reader::new(&data);
        let suffix = read_map_suffix_10_1(&mut reader, 4096, &mut table).unwrap();
        assert_eq!(
            suffix.stop,
            MapSuffixStop::MapSurroundFamilyBody {
                index: 0,
                offset: surround_offset,
                class_id: MARKER_NORTH_ARROW_CLASS_ID,
                key: 41,
                reason: FamilyStop10_1::MarkerElementIncomplete,
            }
        );
        assert_eq!(reader.position(), symbol_offset);
        assert!(table.contains(30) && table.contains(41) && table.contains(42));
        assert!(!table.contains(43));
        assert!(suffix.extent.is_none());
        let Some(crate::maps::map_object::MapSurroundElement10_1::MarkerNorthArrow(arrow)) =
            &suffix.map_surround_parsed[0]
        else {
            panic!("缺少指北针体")
        };
        let SlotEntry10_1::Body(element) = &arrow.marker_element else {
            panic!("缺少元素体")
        };
        assert!(
            matches!(element.stop, Some(ElementBodyStop10_1::NestedObjectNotClosed { key: 43, offset, .. }) if offset == symbol_offset)
        );
    }

    #[test]
    fn marker_north_arrow_legacy_symbol_stops_are_propagated() {
        use crate::objects::display_symbol::SIMPLE_MARKER_SYMBOL_CLASS_ID;
        for nested in [false, true] {
            let mut data = mna_v2_bytes();
            data[20..22].copy_from_slice(&1u16.to_le_bytes());
            data.truncate(data.len() - 16);
            data.extend(0i32.to_le_bytes());
            if nested {
                data.extend(SIMPLE_MARKER_SYMBOL_CLASS_ID);
                data.extend(50u32.to_le_bytes());
                data.extend(1u16.to_le_bytes());
            }
            let stop_offset = data.len();
            data.extend([0x77u8; 16]);
            data.extend(51u32.to_le_bytes());
            let mut table = RefTable10_1::new();
            table.register(30);
            let mut reader = Reader::new(&data);
            reader.read_bytes(20).unwrap();
            let body = read_marker_north_arrow_body_10_1(&mut reader, 4096, &mut table, 9).unwrap();
            let expected = if nested {
                SymbolStop10_1::NestedIncomplete {
                    class_id: SIMPLE_MARKER_SYMBOL_CLASS_ID,
                    key: 50,
                }
            } else {
                SymbolStop10_1::Slot {
                    kind: DisplaySlotKind10_1::Symbol,
                    class_id: [0x77; 16],
                    key: 51,
                }
            };
            assert_eq!(
                body.stop,
                Some(FamilyStop10_1::LegacyMarkerSymbol(expected))
            );
            assert_eq!(reader.position(), stop_offset);
            assert_eq!(table.contains(50), nested);
            assert!(!table.contains(51));
        }
    }

    #[test]
    fn rejects_marker_north_arrow_version_out_of_range() {
        // 自身版本 3 拒绝（>2 无写侧来源）。
        let mut data = mna_v2_bytes();
        data[20] = 3;
        assert!(read_body(&data, read_marker_north_arrow_body_10_1).is_err());
        // 基类版本 5 拒绝（>4 原生 0x80004005）。
        let mut data = mna_v2_bytes();
        data[22] = 5;
        assert!(read_body(&data, read_marker_north_arrow_body_10_1).is_err());
    }

    /// LegendFormat 完整体（版本 2 + TextSymbol 空标记 + 标量 + 线/面补丁）。
    fn legend_format_v2_bytes() -> Vec<u8> {
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(2u16.to_le_bytes());
        data.extend(5u16.to_le_bytes()); // −0x8c
        data.extend_from_slice(&[0u8; 16]); // TextSymbol 槽空标记
        data.extend(1i32.to_le_bytes()); // −0x84
        for _ in 0..11 {
            data.extend(1.5f64.to_le_bytes());
        }
        // ILinePatch 槽（LinePatch 新对象，内嵌 Polyline）。
        data.extend_from_slice(&LINE_PATCH_CLASS_ID);
        data.extend(31u32.to_le_bytes());
        data.extend(3u16.to_le_bytes()); // Patch 版本
        data.extend_from_slice(&polyline_object_entry(32));
        data.extend_from_slice(&string_bytes("线补丁"));
        data.extend(6u16.to_le_bytes());
        // IAreaPatch 槽（空标记）。
        data.extend_from_slice(&[0u8; 16]);
        // −0xc + 版本 ≥2 double ×2。
        data.extend(7u16.to_le_bytes());
        data.extend(1.0f64.to_le_bytes());
        data.extend(2.0f64.to_le_bytes());
        data
    }

    #[test]
    fn reads_legend_format_v2_body() {
        let data = legend_format_v2_bytes();
        let body = read_body(&data, read_legend_format_body_10_1).expect("LegendFormat 体");
        assert_eq!(body.version, 2);
        assert_eq!(body.extra_short, 5);
        assert!(matches!(body.title_symbol, SlotEntry10_1::Null));
        assert_eq!(body.raw_int32, Some(1));
        assert_eq!(body.doubles, [1.5; 11]);
        match &body.line_patch {
            SlotEntry10_1::Body(patch) => {
                assert_eq!(patch.key, 31);
                assert_eq!(patch.name, "线补丁");
                assert!(matches!(
                    patch.geometry,
                    PatchGeometrySlot10_1::Geometry(GeometryBody10_1::Polyline(_))
                ));
            }
            other => panic!("line_patch 应为 Body，得到 {other:?}"),
        }
        assert!(matches!(body.area_patch, SlotEntry10_1::Null));
        assert_eq!(body.tail_short, Some(7));
        assert_eq!(body.patch_doubles, Some([1.0, 2.0]));
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn legend_format_patch_slot_unsupported_stops() {
        let mut data = legend_format_v2_bytes();
        // 线补丁槽起点 = 头 20 + 版本 2 + short 2 + TextSymbol 16 + int32 4
        // + double ×11 = 20+2+2+16+4+88 = 132。
        replace_entry(&mut data, 132, &[0x66; 16], 41);
        let body = read_body(&data, read_legend_format_body_10_1).expect("停止是成功返回");
        assert!(matches!(
            body.line_patch,
            SlotEntry10_1::Unsupported { key: 41, .. }
        ));
        assert!(matches!(body.area_patch, SlotEntry10_1::Null));
        assert_eq!(body.tail_short, None);
        assert_eq!(body.patch_doubles, None);
        assert!(body.stop.is_some());
    }

    #[test]
    fn rejects_legend_format_version_out_of_range() {
        let mut data = legend_format_v2_bytes();
        data[20] = 3;
        assert!(read_body(&data, read_legend_format_body_10_1).is_err());
    }

    /// 比例尺子类完身体（SingleDivision：中段 1 个填充槽）。
    fn subclass_full_bytes() -> Vec<u8> {
        let mut data = vec![0xaau8; 16];
        data.extend_from_slice(&9u32.to_le_bytes());
        data.extend(1u16.to_le_bytes()); // 自身版本
        data.extend(6u16.to_le_bytes()); // 基类版本
        data.extend_from_slice(&common_prefix_bytes());
        data.extend(1.0f64.to_le_bytes()); // −0x17c
        for _ in 0..3 {
            data.extend(2u16.to_le_bytes()); // −0x174..−0x170
        }
        data.extend(3i32.to_le_bytes()); // −0x16c
        data.extend_from_slice(&string_bytes("#,##0")); // −0x168
        data.extend(4i32.to_le_bytes()); // −0x164
        data.extend(5.0f64.to_le_bytes()); // −0x15c
        data.extend_from_slice(&[0u8; 16]); // TextSymbol 槽 −0x154（空标记）
        data.extend(6i32.to_le_bytes()); // −0x150
        data.extend(7i32.to_le_bytes()); // −0x148
        data.extend(8.0f64.to_le_bytes()); // −0x144
        data.extend_from_slice(&[0u8; 16]); // TextSymbol 槽 −0x13c（空标记）
        data.extend_from_slice(&[0u8; 16]); // NumberFormat 槽 −0x14c（空标记）
        data.extend(9i32.to_le_bytes()); // −0x138（≥3）
        data.extend(10.0f64.to_le_bytes()); // −0xec（≥4）
        data.extend(11u16.to_le_bytes()); // −0xbc（≥5）
        data.extend(12u16.to_le_bytes()); // −0xba（≥6）
        // 中段：版本 1 + double + 1 个 IFillSymbol 槽（空标记）。
        data.extend(1u16.to_le_bytes());
        data.extend(13.0f64.to_le_bytes());
        data.extend_from_slice(&[0u8; 16]);
        // 尾段：版本 2 + 2 个 ILineSymbol 槽（空标记）+ double ×2 + int32 ×2。
        data.extend(2u16.to_le_bytes());
        data.extend_from_slice(&[0u8; 16]);
        data.extend_from_slice(&[0u8; 16]);
        data.extend(14.0f64.to_le_bytes());
        data.extend(15.0f64.to_le_bytes());
        data.extend(16i32.to_le_bytes());
        data.extend(17i32.to_le_bytes());
        data
    }

    fn subclass_full_bytes_two_fills() -> Vec<u8> {
        let data = subclass_full_bytes();
        // 在中段填充槽后再补一个填充槽（Hollow/Alt/DblAlt 共 2 个）。
        // 尾段总长：版本 2 + 两符号槽 16×2 + double ×2 + int32 ×2 = 58；
        // 第二个填充槽插在尾段起点（即首个填充槽之后）。
        let insert_at = data.len() - 58;
        let mut out = Vec::new();
        out.extend_from_slice(&data[..insert_at]);
        out.extend_from_slice(&[0u8; 16]); // 第二个填充槽
        out.extend_from_slice(&data[insert_at..]);
        out
    }

    #[test]
    fn reads_single_division_full_body() {
        let data = subclass_full_bytes();
        let body = read_body(&data, |r, m, t, k| {
            read_scalebar_subclass_body_10_1(r, m, t, k, ScalebarSubclassKind10_1::SingleDivision)
        })
        .expect("SingleDivision 体");
        assert_eq!(body.version, 1);
        assert_eq!(body.base_version, 6);
        assert_eq!(body.base_number_string, "#,##0");
        assert_eq!(body.tail_int32, Some(9));
        assert_eq!(body.tail_double, Some(10.0));
        assert_eq!(body.tail_short_a, Some(11));
        assert_eq!(body.tail_short_b, Some(12));
        assert_eq!(body.mid_version, Some(1));
        assert_eq!(body.mid_double, Some(13.0));
        assert_eq!(body.fill_symbols.len(), 1);
        assert_eq!(body.tail_version, Some(2));
        assert!(body.legacy_flag.is_none());
        assert_eq!(body.line_symbols.len(), 2);
        assert_eq!(body.tail_doubles, Some([14.0, 15.0]));
        assert_eq!(body.tail_ints, Some([16, 17]));
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn reads_hollow_with_two_fill_slots() {
        let data = subclass_full_bytes_two_fills();
        let body = read_body(&data, |r, m, t, k| {
            read_scalebar_subclass_body_10_1(r, m, t, k, ScalebarSubclassKind10_1::Hollow)
        })
        .expect("Hollow 体");
        assert_eq!(body.fill_symbols.len(), 2);
        assert_eq!(body.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn rejects_subclass_version_out_of_range() {
        // 自身版本 >1。
        let mut data = subclass_full_bytes();
        data[20] = 2;
        assert!(
            read_body(&data, |r, m, t, k| {
                read_scalebar_subclass_body_10_1(r, m, t, k, ScalebarSubclassKind10_1::Alternating)
            })
            .is_err()
        );
        // 基类版本 >6。
        let mut data = subclass_full_bytes();
        data[22] = 7;
        assert!(
            read_body(&data, |r, m, t, k| {
                read_scalebar_subclass_body_10_1(r, m, t, k, ScalebarSubclassKind10_1::Alternating)
            })
            .is_err()
        );
    }

    #[test]
    fn truncation_leaves_reader_unmoved_and_table_clean() {
        // 对 Scalebar 完整体逐前缀截断：失败时游标与登记表保持不变。
        let data = scalebar_full_bytes();
        for cut in 20..data.len() {
            let mut table = RefTable10_1::default();
            let mut reader = Reader::new(&data);
            reader.read_bytes(20).unwrap();
            let pos = reader.position();
            let result = read_scalebar_body_10_1(&mut reader, 4096, &mut table, 5);
            if result.is_err() {
                assert_eq!(reader.position(), pos, "截断 {cut} 处失败必须不推进游标");
                assert!(
                    !table.contains(11) && !table.contains(12),
                    "截断 {cut} 处失败不得登记槽位键"
                );
            }
        }
    }
}
