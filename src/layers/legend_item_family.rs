//! ArcMap 10.1 ILegendItem 其余实现类（HorizontalBarLegendItem/
//! NestedLegendItem/VerticalLegendItem）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-legend-family.md`（2026-11-16 本轮研究记录）。
//! 二进制证据为 Renderers.dll 10.1.0.3035（本轮从安装介质 `Setup1.cab` 条目
//! `Renderers.dll.6BF03936_…` 提取复核，257024 字节，SHA-256
//! `399c80c9…56274`，pefile 复核 PE32 x86、ImageBase `0x7cac0000`、版本
//! 10.1.0.3035，与 legend-tails 轮记录一致）的静态反编译。三类与已闭合的
//! HorizontalLegendItem（[`crate::layers::legend_item`]）同构：identity 虚表
//! 槽序 QI/AddRef/Release/GetRuntimeClassName（返回类名字串）/析构/
//! GetClassID（写出 CLSID）/空实现，真正的 IPersistStream 虚表由构造函数
//! 动态安装；其 Load 实体（经 IAT `0x7cae9398` 对 LayerLib.dll 导出
//! `?Load@BaseLegendItem@@UAGJPAUIStream@@@Z` 的尾调引用定位，四个实现类
//! 恰好四条引用）读 int16 条目版本后按各自顺序嵌入 BaseLegendItem 字段序。
//!
//! | 类 | CLSID | GetClassID | Load 实体 | Save 实体 | 版本门 | 10.1 写侧 |
//! | --- | --- | --- | --- | --- | --- | --- |
//! | HorizontalBarLegendItem | `{2B65D211-C2C7-11D3-92F3-00600802E603}` | `0x7cad29d0` | `0x7cad3040` | `0x7cad30e0`（版本常量 `DAT_7cae9764`=1） | 有符号 >1 拒绝 | 1 |
//! | NestedLegendItem | `{2B65D212-C2C7-11D3-92F3-00600802E603}` | `0x7cad7a10` | `0x7cad8fd0` | `0x7cad9100`（`DAT_7cae974c`=1） | 同上 | 1 |
//! | VerticalLegendItem | `{A9401A48-4649-11D1-880B-080009EC732A}` | `0x7cae1550` | `0x7cae1a90` | `0x7cae1b00`（`DAT_7cae9738`=1） | 同上 | 1 |
//!
//! 字段序（三类流内布局互不相同）：
//!
//! - **HorizontalBarLegendItem**（Load `0x7cad3040`）：条目版本 short →
//!   BaseLegendItem 字段序（嵌入基类于 `this−0x20`）→ int32（`−0x1c`）→
//!   double（`−0x14`）→ double（`−0xc`）。
//! - **NestedLegendItem**（Load `0x7cad8fd0`）：条目版本 short → Base-
//!   LegendItem 字段序（`this−0x50`）→ int32（`−0x2c`）→ double（`−0x24`）
//!   → IFillSymbol 槽（`−0x1c`，请求 IID `{E6BDAA7E-…}`）→ ILineSymbol 槽
//!   （`−0x18`，请求 IID `{E6BDAA7D-…}`）→ int16 ×3（`−0x14/−0x12/−0x10`）
//!   → int32（`−0xc`）。
//! - **VerticalLegendItem**（Load `0x7cae1a90`）：条目版本 short → int32
//!   （`−8`）→ BaseLegendItem 字段序（`this−8`）。**与 HorizontalLegendItem
//!   （[`crate::layers::legend_item`]）的流内布局逐字节同序**（两者 C++
//!   基类槽偏移同为 −8；区别只在运行时类身份）。
//!
//! BaseLegendItem 字段序（LayerLib.dll `0x786a2b00`，字段明细见
//! [`crate::layers::legend_item`] 模块文档）：int16 基类版本 → ILayer 槽 →
//! int16 ×4 → ITextSymbol 槽 → int32 → int16 → ITextSymbol 槽 →
//! ILegendClassFormat 槽 → int16 ×2 → [v2 ≥2] int16/int16/String/String/
//! int16 → [v2 >2] INumberFormat 槽。
//!
//! **C 轮统一说明**：本字段序的唯一读取器为
//! [`crate::layers::legend_item::read_base_legend_item_fields_10_1`]（其槽
//! 位口径已按本模块研究记录统一——ITextSymbol/INumberFormat 闭合类整体
//! 消费）；本模块原持的本地副本已删除，三类经统一读取器复用。
//!
//! 样本边界：仓库参考样本 `samples/测试底图.mxd` /Maps 流三类 CLSID 0 命中，
//! 全部布局结论**无样本字节佐证**；单元测试全部使用合成输入。
//!
//! 调用约定与事务性：调用者已用
//! [`crate::storage::object_stream::read_header_10_1`] 读取对象头、比对
//! 相应 `*_CLASS_ID` 常量并取得键，从对象头之后的位置传入 Reader。停止点
//! 是成功返回（游标回退到槽位起点、该键不保留）；截断、版本超限为错误，
//! 游标与登记表整体不动。全部结论为静态反编译观察，未经目标软件运行
//! 验证，不得声称已验证。

use crate::layers::legend_item::{BaseLegendItemFields10_1, read_base_legend_item_fields_10_1};
use crate::objects::display_symbol::{
    MAX_SYMBOL_NESTING_DEPTH, SymbolSlotFull10_1, read_symbol_slot_full_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// HorizontalBarLegendItem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{2B65D211-C2C7-11D3-92F3-00600802E603}`。证据：
/// esri.clsid.ecfg 声明（模块 Renderers.dll，类名 HorizontalBarLegendItem）；
/// Renderers.dll GetClassID `0x7cad29d0` 写出（`DAT_7caeae70/7caeae78`）。
/// 仓库参考样本 /Maps 流 0 命中。
pub const HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID: [u8; 16] = [
    0x11, 0xd2, 0x65, 0x2b, 0xc7, 0xc2, 0xd3, 0x11, 0x92, 0xf3, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// NestedLegendItem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{2B65D212-C2C7-11D3-92F3-00600802E603}`。证据：
/// esri.clsid.ecfg 声明；Renderers.dll GetClassID `0x7cad7a10` 写出
/// （`DAT_7caeb5f0/7caeb5f8`）。样本全流 0 命中。
pub const NESTED_LEGEND_ITEM_CLASS_ID: [u8; 16] = [
    0x12, 0xd2, 0x65, 0x2b, 0xc7, 0xc2, 0xd3, 0x11, 0x92, 0xf3, 0x00, 0x60, 0x08, 0x02, 0xe6, 0x03,
];

/// VerticalLegendItem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{A9401A48-4649-11D1-880B-080009EC732A}`。证据：
/// esri.clsid.ecfg 声明；Renderers.dll GetClassID `0x7cae1550` 写出
/// （`DAT_7caeb5e0/7caeb5e8`）。样本全流 0 命中。
pub const VERTICAL_LEGEND_ITEM_CLASS_ID: [u8; 16] = [
    0x48, 0x1a, 0x40, 0xa9, 0x49, 0x46, 0xd1, 0x11, 0x88, 0x0b, 0x08, 0x00, 0x09, 0xec, 0x73, 0x2a,
];

/// 条目版本上限：三类 Load 实体有符号 `1 < v` 拒绝 `0x80004005`；Save
/// 恒写静态常量 1。
const MAX_ITEM_VERSION: i16 = 1;

/// 从对象头之后的位置读取 HorizontalBarLegendItem 对象体。
///
/// 字段序（Load `0x7cad3040`）：条目版本 short（有符号 >1 拒绝）→
/// BaseLegendItem 字段序 → int32（`−0x1c`，位模式）→ double ×2（`−0x14/
/// −0xc`，位模式）。
pub fn read_horizontal_bar_legend_item_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<HorizontalBarLegendItemBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("HorizontalBarLegendItem 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ITEM_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "条目版本 {version} 超出接受范围（有符号 >1 拒绝 0x80004005；Save 恒写 1；\
             负值位模式原生通过，此处一并拒绝）"
        )));
    }
    let base = read_base_legend_item_fields_10_1(&mut trial, max_bytes, table)?;
    let mut raw_int32 = None;
    let mut doubles = None;
    if base.stop.is_none() {
        raw_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        let a = trial.read_f64_le()?;
        let b = trial.read_f64_le()?;
        doubles = Some([a, b]);
    }
    *reader = trial;
    Ok(HorizontalBarLegendItemBody10_1 {
        key,
        version,
        base,
        raw_int32,
        doubles,
        remaining_bytes: reader.remaining(),
    })
}

/// HorizontalBarLegendItem 对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct HorizontalBarLegendItemBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 条目版本 short 的无符号位模式（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// BaseLegendItem 字段序。
    pub base: BaseLegendItemFields10_1,
    /// int32（`−0x1c`，位模式）。
    pub raw_int32: Option<i32>,
    /// double ×2（`−0x14/−0xc`，位模式）。
    pub doubles: Option<[f64; 2]>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 NestedLegendItem 对象体。
///
/// 字段序（Load `0x7cad8fd0`）：条目版本 short（有符号 >1 拒绝）→
/// BaseLegendItem 字段序 → int32（`−0x2c`，位模式）→ double（`−0x24`）→
/// IFillSymbol 槽（`−0x1c`）→ ILineSymbol 槽（`−0x18`）→ int16 ×3
/// （`−0x14/−0x12/−0x10`）→ int32（`−0xc`）。两个符号槽经
/// [`read_symbol_slot_full_10_1`] 全量分派（未知符号类为槽内枚举值，
/// 不整体停止）。
pub fn read_nested_legend_item_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<NestedLegendItemBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("NestedLegendItem 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ITEM_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "条目版本 {version} 超出接受范围（有符号 >1 拒绝 0x80004005；Save 恒写 1；\
             负值位模式原生通过，此处一并拒绝）"
        )));
    }
    let base = read_base_legend_item_fields_10_1(&mut trial, max_bytes, table)?;
    let mut raw_int32 = None;
    let mut raw_double = None;
    let mut fill_symbol = SymbolSlotFull10_1::Null;
    let mut line_symbol = SymbolSlotFull10_1::Null;
    let mut shorts = [0_u16; 3];
    let mut raw_int32_b = None;
    if base.stop.is_none() {
        raw_int32 = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        raw_double = Some(trial.read_f64_le()?);
        fill_symbol =
            read_symbol_slot_full_10_1(&mut trial, max_bytes, table, MAX_SYMBOL_NESTING_DEPTH)?;
        line_symbol =
            read_symbol_slot_full_10_1(&mut trial, max_bytes, table, MAX_SYMBOL_NESTING_DEPTH)?;
        for slot in &mut shorts {
            *slot = trial.read_u16_le()?;
        }
        raw_int32_b = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
    }
    *reader = trial;
    Ok(NestedLegendItemBody10_1 {
        key,
        version,
        base,
        raw_int32,
        raw_double,
        fill_symbol,
        line_symbol,
        shorts,
        raw_int32_b,
        remaining_bytes: reader.remaining(),
    })
}

/// NestedLegendItem 对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct NestedLegendItemBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 条目版本 short 的无符号位模式（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// BaseLegendItem 字段序。
    pub base: BaseLegendItemFields10_1,
    /// int32（`−0x2c`，位模式）。
    pub raw_int32: Option<i32>,
    /// double（`−0x24`，位模式）。
    pub raw_double: Option<f64>,
    /// IFillSymbol 槽（`−0x1c`）。
    pub fill_symbol: SymbolSlotFull10_1,
    /// ILineSymbol 槽（`−0x18`）。
    pub line_symbol: SymbolSlotFull10_1,
    /// int16 ×3（`−0x14/−0x12/−0x10`，位模式）。
    pub shorts: [u16; 3],
    /// int32（`−0xc`，位模式）。
    pub raw_int32_b: Option<i32>,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 VerticalLegendItem 对象体。
///
/// 字段序（Load `0x7cae1a90`）：条目版本 short（有符号 >1 拒绝）→ int32
/// （`−8`，位模式）→ BaseLegendItem 字段序。**流内布局与 Horizontal-
/// LegendItem 逐字节同序**（C++ 基类槽同为 −8）；区别只在对象头类身份。
pub fn read_vertical_legend_item_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<VerticalLegendItemBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("VerticalLegendItem 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_ITEM_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "条目版本 {version} 超出接受范围（有符号 >1 拒绝 0x80004005；Save 恒写 1；\
             负值位模式原生通过，此处一并拒绝）"
        )));
    }
    let raw_int32 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let base = read_base_legend_item_fields_10_1(&mut trial, max_bytes, table)?;
    *reader = trial;
    Ok(VerticalLegendItemBody10_1 {
        key,
        version,
        raw_int32,
        base,
        remaining_bytes: reader.remaining(),
    })
}

/// VerticalLegendItem 对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct VerticalLegendItemBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 条目版本 short 的无符号位模式（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// int32（`−8`，位模式；与 HorizontalLegendItem 的 int32 同位）。
    pub raw_int32: i32,
    /// BaseLegendItem 字段序。
    pub base: BaseLegendItemFields10_1,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::legend_format::LEGEND_CLASS_FORMAT_CLASS_ID;
    use crate::layers::legend_item::{
        BaseLegendItemStop10_1, LegendItemClassFormatSlot10_1, LegendItemSlot10_1,
    };
    use crate::objects::number_format::NUMERIC_FORMAT_CLASS_ID;

    /// Map 类标识（回引用对象头用）：`{E6BDAA76-…}`。
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

    /// 最小 NumericFormat 体。
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

    /// 最小 LegendClassFormat 体（版本 2 + 四个空槽 + double ×2）。
    fn legend_class_format_body_bytes() -> Vec<u8> {
        let mut bytes = 2u16.to_le_bytes().to_vec();
        for _ in 0..4 {
            bytes.extend_from_slice(&[0u8; 16]);
        }
        bytes.extend(1.0f64.to_le_bytes());
        bytes.extend(2.0f64.to_le_bytes());
        bytes
    }

    /// BaseLegendItem 字段序（基类版本 v2，ILayer 为键 30 回引，符号槽全部
    /// 空标记；v2 ≥2 组含两个字符串；v2 >2 的 INumberFormat 槽为 NumericFormat
    /// 新对象开关）。
    fn base_fields_bytes(base_version: u16, with_number_format: bool) -> Vec<u8> {
        let mut bytes = base_version.to_le_bytes().to_vec();
        // ILayer 槽：键 30 回引。
        bytes.extend_from_slice(&MAP_CLSID);
        bytes.extend_from_slice(&30u32.to_le_bytes());
        // int16 ×4。
        for _ in 0..4 {
            bytes.extend(1u16.to_le_bytes());
        }
        // ITextSymbol 槽（空标记）。
        bytes.extend_from_slice(&[0u8; 16]);
        // int32 + int16。
        bytes.extend(2i32.to_le_bytes());
        bytes.extend(3u16.to_le_bytes());
        // 第二 ITextSymbol 槽（空标记）。
        bytes.extend_from_slice(&[0u8; 16]);
        // ILegendClassFormat 槽（空标记；调用方按需替换）。
        bytes.extend_from_slice(&[0u8; 16]);
        // int16 ×2。
        bytes.extend(4u16.to_le_bytes());
        bytes.extend(5u16.to_le_bytes());
        if base_version >= 2 {
            // ≥2 组：short + short + String + String + short。
            bytes.extend(6u16.to_le_bytes());
            bytes.extend(7u16.to_le_bytes());
            bytes.extend_from_slice(&string_bytes(" ("));
            bytes.extend_from_slice(&string_bytes(")"));
            bytes.extend(0xffffu16.to_le_bytes());
        }
        if base_version > 2 {
            if with_number_format {
                bytes.extend_from_slice(&NUMERIC_FORMAT_CLASS_ID);
                bytes.extend(40u32.to_le_bytes());
                bytes.extend_from_slice(&numeric_format_bytes());
            } else {
                bytes.extend_from_slice(&[0u8; 16]);
            }
        }
        bytes
    }

    fn read_fields(data: &[u8]) -> io::Result<BaseLegendItemFields10_1> {
        let mut table = RefTable10_1::default();
        table.register(30);
        let mut reader = Reader::new(data);
        read_base_legend_item_fields_10_1(&mut reader, 4096, &mut table)
    }

    fn read_item<T>(
        data: &[u8],
        read: impl FnOnce(&mut Reader<'_>, usize, &mut RefTable10_1, u32) -> io::Result<T>,
    ) -> io::Result<T> {
        let mut table = RefTable10_1::default();
        table.register(30);
        let mut reader = Reader::new(data);
        reader.read_bytes(20).unwrap();
        read(&mut reader, 4096, &mut table, 5)
    }

    /// 对象头（类标识占位 + 键）。
    fn header() -> Vec<u8> {
        let mut bytes = vec![0xaau8; 16];
        bytes.extend_from_slice(&9u32.to_le_bytes());
        bytes
    }

    #[test]
    fn class_id_constants_match_documented_guids() {
        // 证据见各常量文档注释（GetClassID 写出 + esri.clsid 声明）。
        assert_eq!(
            HORIZONTAL_BAR_LEGEND_ITEM_CLASS_ID,
            [
                0x11, 0xd2, 0x65, 0x2b, 0xc7, 0xc2, 0xd3, 0x11, 0x92, 0xf3, 0x00, 0x60, 0x08, 0x02,
                0xe6, 0x03,
            ]
        );
        assert_eq!(
            NESTED_LEGEND_ITEM_CLASS_ID,
            [
                0x12, 0xd2, 0x65, 0x2b, 0xc7, 0xc2, 0xd3, 0x11, 0x92, 0xf3, 0x00, 0x60, 0x08, 0x02,
                0xe6, 0x03,
            ]
        );
        assert_eq!(
            VERTICAL_LEGEND_ITEM_CLASS_ID,
            [
                0x48, 0x1a, 0x40, 0xa9, 0x49, 0x46, 0xd1, 0x11, 0x88, 0x0b, 0x08, 0x00, 0x09, 0xec,
                0x73, 0x2a,
            ]
        );
    }

    #[test]
    fn reads_base_fields_v3_full() {
        let data = base_fields_bytes(3, true);
        let fields = read_fields(&data).expect("基类字段序");
        assert_eq!(fields.base_version, 3);
        assert!(matches!(
            fields.layer,
            LegendItemSlot10_1::BackReference { key: 30, .. }
        ));
        assert_eq!(fields.shorts_a, [1; 4]);
        assert_eq!(fields.raw_int32_a, 2);
        assert_eq!(fields.short_b, 3);
        assert!(matches!(
            fields.class_format,
            LegendItemClassFormatSlot10_1::Null
        ));
        assert_eq!(fields.shorts_b, [4, 5]);
        let group = fields.tail_group.as_ref().expect("v2 ≥2 组");
        assert_eq!(group.prefix, " (");
        assert_eq!(group.suffix, ")");
        assert_eq!(group.short_c, 0xffff);
        match fields.number_format.as_ref().expect("v2 >2 槽") {
            LegendItemSlot10_1::NumericFormat(nf) => assert_eq!(nf.int32s, [1, 2, 3, 4]),
            other => panic!("number_format 应为 NumericFormat，得到 {other:?}"),
        }
        assert_eq!(fields.stop, None);
    }

    #[test]
    fn reads_base_fields_v1_short_form() {
        // 版本 1：无 ≥2 组与 INumberFormat 槽。
        let data = base_fields_bytes(1, false);
        let fields = read_fields(&data).expect("v1 字段序");
        assert_eq!(fields.tail_group, None);
        assert_eq!(fields.number_format, None);
        assert_eq!(fields.stop, None);
    }

    #[test]
    fn base_fields_ilayer_new_object_stops_and_rolls_back() {
        let mut data = base_fields_bytes(3, true);
        // ILayer 槽（偏移 2..22）换成未知类（键 99 未登记 → 新对象分类）。
        for byte in &mut data[2..18] {
            *byte = 0x55;
        }
        data[18..22].copy_from_slice(&99u32.to_le_bytes());
        let fields = read_fields(&data).expect("停止是成功返回");
        match fields.layer {
            LegendItemSlot10_1::Unsupported { key, .. } => assert_eq!(key, 99),
            other => panic!("应为 Unsupported，得到 {other:?}"),
        }
        assert_eq!(fields.shorts_a, [0; 4]);
        assert_eq!(fields.tail_group, None);
        assert!(matches!(
            fields.stop,
            Some(BaseLegendItemStop10_1::SlotUnsupported {
                slot: "layer(ILayer)",
                ..
            })
        ));
    }

    #[test]
    fn base_fields_class_format_slot_consumes_legend_class_format() {
        // 类格式槽在基类版本 2 起（无 ≥2 组）：偏移 = 2 + 20 + 8 + 16 + 4 + 2
        // + 16 + 16 = 84？逐段核对：版本 short(2) + ILayer(20) + shorts×4(8)
        // + TextSymbol(16) + int32+int16(6) + 第二 TextSymbol(16) = 68 为槽
        // 起点。构造：槽之前的字节 + 槽头（LCF 类 + 键 50）+ LCF 体 + 其后
        // 字段（shorts ×2，版本 1 无尾组）。
        let full = base_fields_bytes(1, false);
        let slot_at = 68;
        let mut data = Vec::new();
        data.extend_from_slice(&full[..slot_at]);
        data.extend_from_slice(&LEGEND_CLASS_FORMAT_CLASS_ID);
        data.extend_from_slice(&50u32.to_le_bytes());
        data.extend_from_slice(&legend_class_format_body_bytes());
        // 原槽仅占 16 字节（68..84），其后为 shorts ×2（84..88）。
        data.extend_from_slice(&full[slot_at + 16..]);
        let mut table = RefTable10_1::default();
        table.register(30);
        let mut reader = Reader::new(&data);
        let fields = read_base_legend_item_fields_10_1(&mut reader, 4096, &mut table)
            .expect("类格式槽整体消费");
        match fields.class_format {
            LegendItemClassFormatSlot10_1::ClassFormat(body) => {
                assert_eq!(body.key, 50);
                assert_eq!(body.version, 2);
            }
            other => panic!("应为 ClassFormat，得到 {other:?}"),
        }
        assert_eq!(fields.stop, None);
    }

    #[test]
    fn base_fields_rejects_base_version_above_3() {
        let mut data = base_fields_bytes(3, true);
        data[0] = 4;
        data[1] = 0;
        assert!(read_fields(&data).is_err());
        data[0] = 0xff;
        data[1] = 0xff;
        assert!(read_fields(&data).is_err());
    }

    /// HorizontalBar 完整体。
    fn hbar_bytes() -> Vec<u8> {
        let mut data = header();
        data.extend(1u16.to_le_bytes()); // 条目版本
        data.extend_from_slice(&base_fields_bytes(3, true));
        data.extend(8i32.to_le_bytes()); // −0x1c
        data.extend(1.5f64.to_le_bytes()); // −0x14
        data.extend(2.5f64.to_le_bytes()); // −0xc
        data
    }

    #[test]
    fn reads_horizontal_bar_full_body() {
        let data = hbar_bytes();
        let body =
            read_item(&data, read_horizontal_bar_legend_item_body_10_1).expect("HorizontalBar 体");
        assert_eq!(body.key, 5);
        assert_eq!(body.version, 1);
        assert_eq!(body.base.base_version, 3);
        assert_eq!(body.raw_int32, Some(8));
        assert_eq!(body.doubles, Some([1.5, 2.5]));
        assert_eq!(body.base.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn rejects_horizontal_bar_item_version_above_1() {
        let mut data = hbar_bytes();
        data[20] = 2;
        assert!(read_item(&data, read_horizontal_bar_legend_item_body_10_1).is_err());
    }

    /// Nested 完整体。
    fn nested_bytes() -> Vec<u8> {
        let mut data = header();
        data.extend(1u16.to_le_bytes());
        data.extend_from_slice(&base_fields_bytes(3, false));
        data.extend(8i32.to_le_bytes()); // −0x2c
        data.extend(1.5f64.to_le_bytes()); // −0x24
        data.extend_from_slice(&[0u8; 16]); // IFillSymbol 槽
        data.extend_from_slice(&[0u8; 16]); // ILineSymbol 槽
        for _ in 0..3 {
            data.extend(2u16.to_le_bytes()); // −0x14/−0x12/−0x10
        }
        data.extend(3i32.to_le_bytes()); // −0xc
        data
    }

    #[test]
    fn reads_nested_full_body() {
        let data = nested_bytes();
        let body = read_item(&data, read_nested_legend_item_body_10_1).expect("Nested 体");
        assert_eq!(body.raw_int32, Some(8));
        assert_eq!(body.raw_double, Some(1.5));
        assert!(matches!(body.fill_symbol, SymbolSlotFull10_1::Null));
        assert!(matches!(body.line_symbol, SymbolSlotFull10_1::Null));
        assert_eq!(body.shorts, [2; 3]);
        assert_eq!(body.raw_int32_b, Some(3));
        assert_eq!(body.base.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    /// Vertical 完整体（与 HorizontalLegendItem 流内同序：short + int32 + 基类；
    /// 基类字段不含 INumberFormat 新对象，保证两读取器等价可达体尾）。
    fn vertical_bytes() -> Vec<u8> {
        let mut data = header();
        data.extend(1u16.to_le_bytes());
        data.extend(8i32.to_le_bytes()); // −8
        data.extend_from_slice(&base_fields_bytes(3, false));
        data
    }

    #[test]
    fn reads_vertical_full_body() {
        let data = vertical_bytes();
        let body = read_item(&data, read_vertical_legend_item_body_10_1).expect("Vertical 体");
        assert_eq!(body.raw_int32, 8);
        assert_eq!(body.base.base_version, 3);
        assert_eq!(body.base.stop, None);
        assert_eq!(body.remaining_bytes, 0);
    }

    #[test]
    fn vertical_layout_matches_horizontal_legend_item_byte_order() {
        // 与 crate::layers::legend_item 的 HorizontalLegendItem 读取器对同一
        // 合成流（条目版本 + int32 + BaseLegendItem 字段序）产出同布局结论：
        // 两者流内逐字节同序（C++ 基类槽同为 −8）。
        use crate::layers::legend_item::read_horizontal_legend_item_body_10_1;
        let data = vertical_bytes();
        let vertical = read_item(&data, read_vertical_legend_item_body_10_1).expect("Vertical 体");
        let horizontal = read_item(&data, read_horizontal_legend_item_body_10_1)
            .expect("HorizontalLegendItem 同流解析");
        assert_eq!(horizontal.version, vertical.version);
        assert_eq!(horizontal.raw_int32, vertical.raw_int32);
        assert_eq!(horizontal.base_version, vertical.base.base_version);
        assert_eq!(horizontal.remaining_bytes, vertical.remaining_bytes);
    }

    #[test]
    fn truncation_leaves_reader_unmoved() {
        let data = nested_bytes();
        for cut in 20..data.len() {
            let mut table = RefTable10_1::default();
            table.register(30);
            let mut reader = Reader::new(&data);
            reader.read_bytes(20).unwrap();
            let pos = reader.position();
            let result = read_nested_legend_item_body_10_1(&mut reader, 4096, &mut table, 5);
            if result.is_err() {
                assert_eq!(reader.position(), pos, "截断 {cut} 处失败必须不推进游标");
            }
        }
    }
}
