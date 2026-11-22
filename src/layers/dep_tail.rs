//! ArcMap 10.1 TimeData 族槽实现类与「DisplayExpressionProperties 体之后」
//! 延续段的严格读取器（A 轮研究，2026-11-16）。
//!
//! 证据来源：`docs/arcmap-10.1-dep-tail.md`（本轮研究记录）。全部为静态
//! 反编译取证（本机 pyghidra，Ghidra + JDK 25），未运行目标软件，未人工
//! 目视验收；除延续段的样本分解外，本模块各类均无样本实例（样本槽位全为
//! 空标记），实现仅可用合成输入验证。
//!
//! ## 类身份（三方互证，证据：确定的）
//!
//! 四类均在 SystemCore.dll 10.1.0.3035（SHA-256 `f63cf949…`，ImageBase
//! `0x7ffb0000`；temp/arcgis101-bin/ 既有提取物，pefile 复核版本）：
//!
//! | 类 | CLSID | GetClassID | Load | Save |
//! | --- | --- | --- | --- | --- |
//! | TimeReference | `{EFB2E7DB-78F4-4E24-B01F-4F9C7AB800C5}` | `0x7ffeac00` | `0x7ffebbf0` | `0x7ffebc90` |
//! | TimeExtent | `{5DC783DE-283A-4963-AB53-25A05C5D76CC}` | `0x7ffe4650` | `0x7ffe81f0` | `0x7ffe82f0` |
//! | Time | `{E1721810-8210-45B1-8590-FC4C911FBA20}` | `0x7ffe14a0` | `0x7ffe1780` | `0x7ffe1870` |
//! | TimeZoneInfo | `{78FAD5F1-60FA-458A-8D93-630DA920448D}` | `0x7fff6330` | `0x7fff7790` | `0x7fff77f0` |
//!
//! 定位链（对每类独立执行）：esri.clsid 声明（TimeReference/TimeExtent/
//! Time/TimeZoneInfo progID）→ RTTI `.?AV<类名>@@` TypeDescriptor → COL 簇 →
//! IPersistStream 子虚表（GetClassID 槽 +0x0c 引用 CLSID 数据常量，与声明
//! 逐字节互证）→ +0x14/+0x18 = Load/Save。各 GetClassID 实体以
//! `*param = 常量; *(param+8) = 常量` 形式写出上表 CLSID。
//!
//! ## 槽接口（请求 IID，证据：确定的）
//!
//! - TimeData 段槽（LayerLib.dll `DAT_78721e70`）=
//!   ITimeReference `{F6CBA4A3-BC48-47BC-9F15-2A561B9E6C71}`；
//! - TimeTableDefinition 段槽（LayerLib.dll `DAT_78721e80`）=
//!   ITimeExtent `{BD724B95-018F-4367-9883-9560D92293A7}`；
//! - TimeReference 体内槽 = ITimeZoneInfo
//!   `{0F8FD6AB-CE6D-4BB2-A5EF-2F6035E304C5}`；
//! - TimeExtent 体内前两槽 = ITime
//!   `{66810D21-8DE0-44EE-B26E-465AC09F161F}`，第三槽 = ITimeReference。
//!
//! 请求 IID 只决定接口形状；槽内对象类身份由流内 16 字节类标识决定（与
//! 仓库既有读取器一致），本模块按类标识分派。
//!
//! ## 样本对位（辅助观察）
//!
//! 仓库参考样本 `samples/测试底图.mxd` 的 TimeData 族四段（延续段
//! `0x156e..0x15d7`）全部槽位为空标记，无 TimeReference/TimeExtent/Time/
//! TimeZoneInfo 实例；本模块对样本的验证仅覆盖
//! [`read_time_data_family_10_1`] 的空槽形态（`0x156e..0x15d7` 逐字节
//! 吻合，见研究记录），四个体读取器均无样本字节佐证（合成输入验证）。
//!
//! ## 建议接入点（归整合轮）
//!
//! - `feature_layer_tail::read_feature_layer_successor_10_1` ㊴ 槽
//!   DEP 前缀完成后的 `ExpressionPropsBody` 停止撤销（DEP 体在引擎槽处
//!   完结、无尾段；其后 ㊵/㊶/㊷/㊸ 既有代码直接可达）；
//! - 同函数 ㊱ TimeData/TimeTableDefinition 段的
//!   `SuccessorStop10_1::TimeSlotUnsupported` 停止臂改调本模块
//!   [`read_time_reference_slot_10_1`] / [`read_time_extent_slot_10_1`]
//!   （未知类维持停止语义）；FL 版本 0x1c..0x1f 分支的
//!   ITimeReference/ITimeExtent 槽同理；
//! - 停止点后方语义与样本对位结论见研究记录 `docs/arcmap-10.1-dep-tail.md`。

use std::io::{self, ErrorKind};

use crate::layers::annotate_props::read_stream_string_10_1;
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;

/// TimeReference 类标识（CLSID）的 16 字节内存序。
///
/// `{EFB2E7DB-78F4-4E24-B01F-4F9C7AB800C5}`。证据：esri.clsid 声明
/// （progID TimeReference）；GetClassID `0x7ffeac00` 写出常量
/// `0x8007360c`（与声明逐字节相同）；esriSystem 类型库 coclass
/// TimeReference。
pub const TIME_REFERENCE_CLASS_ID: [u8; 16] = [
    0xdb, 0xe7, 0xb2, 0xef, 0xf4, 0x78, 0x24, 0x4e, 0xb0, 0x1f, 0x4f, 0x9c, 0x7a, 0xb8, 0x00, 0xc5,
];

/// TimeExtent 类标识（CLSID）的 16 字节内存序。
///
/// `{5DC783DE-283A-4963-AB53-25A05C5D76CC}`。证据：esri.clsid 声明
/// （progID TimeExtent）；GetClassID `0x7ffe4650` 写出常量 `0x800735fc`。
pub const TIME_EXTENT_CLASS_ID: [u8; 16] = [
    0xde, 0x83, 0xc7, 0x5d, 0x3a, 0x28, 0x63, 0x49, 0xab, 0x53, 0x25, 0xa0, 0x5c, 0x5d, 0x76, 0xcc,
];

/// Time 类标识（CLSID）的 16 字节内存序。
///
/// `{E1721810-8210-45B1-8590-FC4C911FBA20}`。证据：esri.clsid 声明
/// （progID Time）；GetClassID `0x7ffe14a0` 写出常量 `0x800716cc`。
pub const TIME_CLASS_ID: [u8; 16] = [
    0x10, 0x18, 0x72, 0xe1, 0x10, 0x82, 0xb1, 0x45, 0x85, 0x90, 0xfc, 0x4c, 0x91, 0x1f, 0xba, 0x20,
];

/// TimeZoneInfo 类标识（CLSID）的 16 字节内存序。
///
/// `{78FAD5F1-60FA-458A-8D93-630DA920448D}`。证据：esri.clsid 声明
/// （progID TimeZoneInfo）；GetClassID `0x7fff6330` 写出常量 `0x80073dcc`。
pub const TIME_ZONE_INFO_CLASS_ID: [u8; 16] = [
    0xf1, 0xd5, 0xfa, 0x78, 0xfa, 0x60, 0x8a, 0x45, 0x8d, 0x93, 0x63, 0x0d, 0xa9, 0x20, 0x44, 0x8d,
];

/// 读取 int16 版本并按原生有符号比较校验上限；负值位模式一并拒绝
/// （负值版本无业务意义，较原生更严，与仓库既有读取器一致）。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    what: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let version = trial.read_u16_le()?;
    let signed = version as i16;
    if signed > max || signed < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{what} 对象体 offset 0x{entry_offset:x}: 版本 {version} 超出 \
                 {what}::Load 接受范围（有符号比较 >{max} 拒绝；负值位模式一并拒绝）"
            ),
        ));
    }
    Ok(version)
}

/// 回退到 `target`（必须在停止前的已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// ITime 槽条目分类（请求 IID `{66810D21-…}`；实现类 Time）。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 Time 对象：已解析体。
    Body(Box<TimeBody10_1>),
    /// 新对象但类标识未闭合：停止点（游标已回退到槽位起点、键不保留）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// ITimeReference 槽条目分类（请求 IID `{F6CBA4A3-…}`；实现类 TimeReference）。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeReferenceSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TimeReference 对象：已解析体。
    Body(Box<TimeReferenceBody10_1>),
    /// 新对象但类标识未闭合：停止点（游标已回退到槽位起点、键不保留）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// ITimeZoneInfo 槽条目分类（请求 IID `{0F8FD6AB-…}`；实现类 TimeZoneInfo）。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeZoneInfoSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TimeZoneInfo 对象：已解析体。
    Body(Box<TimeZoneInfoBody10_1>),
    /// 新对象但类标识未闭合：停止点（游标已回退到槽位起点、键不保留）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// ITimeExtent 槽条目分类（请求 IID `{BD724B95-…}`；实现类 TimeExtent）。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeExtentSlot10_1 {
    /// 空引用。
    Null,
    /// 回引条目。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TimeExtent 对象：已解析体。
    Body(Box<TimeExtentBody10_1>),
    /// 新对象但类标识未闭合：停止点（游标已回退到槽位起点、键不保留）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// Time 体（SystemCore.dll `0x7ffe1780`）的已确认字段。
///
/// 证据：确定的（定位链见模块文档；Save `0x7ffe1870` 恒写版本常量
/// `0x8006d0c0`=1，读侧版本 <2 接受，读写对称）。字段序：① int16 版本
/// （接受 0/1，无样本）→ ② int32 判别值：==1 → 8 字节整块读（OA 日期
/// double 候选）；==2 → 16 字节 `WKSDateTime` 整块读；其他 → 原生记
/// `0x80004005`。两类值均原样保留位模式，语义为候选。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeBody10_1 {
    /// 对象头键（槽位新对象时由宿主传入；登记由对象头读取完成）。
    pub key: u32,
    /// 体首 short 版本（接受 0/1；负值位模式拒绝，较原生更严）。
    pub version: u16,
    /// 判别后的值。
    pub value: TimeValue10_1,
}

/// Time 体判别值（`0x7ffe1780` 的 int32 分支）。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeValue10_1 {
    /// 判别值 1：8 字节整块读（OA 日期 double 候选；位模式原样保留）。
    OaDate(i64),
    /// 判别值 2：16 字节 `WKSDateTime` 整块读（位模式原样保留）。
    WksDateTime([u8; 16]),
}

fn read_time_body_with_offset(
    trial: &mut Reader<'_>,
    entry_offset: usize,
    key: u32,
) -> io::Result<TimeBody10_1> {
    let version = read_version_short(trial, 1, "Time", entry_offset)?;
    let discriminant = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let value = match discriminant {
        1 => TimeValue10_1::OaDate(i64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap())),
        2 => TimeValue10_1::WksDateTime(trial.read_bytes(16)?.try_into().unwrap()),
        other => {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Time 对象体 offset 0x{entry_offset:x}: 判别值 {other} 非 1/2 \
                     （原生记 0x80004005）"
                ),
            ));
        }
    };
    Ok(TimeBody10_1 {
        key,
        version,
        value,
    })
}

/// 从对象头之后的位置读取 10.1 Time 体，失败时不推进游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`TIME_CLASS_ID`] 并取得键 `key`。
pub fn read_time_body_10_1(
    reader: &mut Reader<'_>,
    _max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TimeBody10_1> {
    let entry_offset = reader.position();
    let mut trial = reader.clone();
    let body = read_time_body_with_offset(&mut trial, entry_offset, key)?;
    *reader = trial;
    Ok(body)
}

/// SystemCore 10.1 的时区规则块，按 `0x7fff5e90` / `0x7fff5f50` 的顺序保存。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TziRule10_1 {
    pub int_a: i32,
    /// 第一组定长 UTF-16LE 名称（32 个码元），原样保留。
    pub name_a: [u8; 64],
    /// 第一组 16 字节转换日期，原样保留。
    pub date_a: [u8; 16],
    pub int_b: i32,
    /// 第二组定长 UTF-16LE 名称（32 个码元），原样保留。
    pub name_b: [u8; 64],
    /// 第二组 16 字节转换日期，原样保留。
    pub date_b: [u8; 16],
    pub int_c: i32,
}

impl TziRule10_1 {
    const BYTE_LEN: usize = 172;

    fn read(trial: &mut Reader<'_>) -> io::Result<Self> {
        Ok(Self {
            int_a: trial.read_u32_le()? as i32,
            name_a: trial.read_bytes(64)?.try_into().unwrap(),
            date_a: trial.read_bytes(16)?.try_into().unwrap(),
            int_b: trial.read_u32_le()? as i32,
            name_b: trial.read_bytes(64)?.try_into().unwrap(),
            date_b: trial.read_bytes(16)?.try_into().unwrap(),
            int_c: trial.read_u32_le()? as i32,
        })
    }
}

/// TimeZoneInfo 规则数组条目（`0x7fff6010` 循环体，每条 176 字节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TziRuleEntry10_1 {
    /// int32（语义未定）。
    pub int_a: i32,
    /// 172 字节规则块。
    pub rule: TziRule10_1,
}

/// TimeZoneInfo 体（SystemCore.dll `0x7fff7790` → 体 helper `0x7fff6010`）
/// 的已确认字段。
///
/// 证据：确定的（定位链见模块文档；Save `0x7fff77f0` 恒写版本常量
/// `0x8006d040`=2，与读侧版本 2 分支逐字段对称）。字段序：① int16 版本
/// （`2 < (short)ver` 拒绝）→ ② 版本 <2：int32（bias 候选，内存
/// 缺省 -1）→ ③ String ×3（标准时/夏令时/显示名候选）→ ④ 版本 <2：int32
/// （内存缺省 -1）→ ⑤ int32（→ this+0xc）→ ⑥ 规则块（172 字节）→
/// ⑦ int16 计数（≤0 不进循环，原样保留）→ ⑧ 计数 × [int32 + 规则块]。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeZoneInfoBody10_1 {
    /// 对象头键（槽位新对象时由宿主传入）。
    pub key: u32,
    /// 体首 short 版本（≤2；负值位模式拒绝，较原生更严）。
    pub version: u16,
    /// 版本 <2 的 int32（bias 候选）。`None` = 版本 ≥2。
    pub bias_a: Option<i32>,
    /// String ×3（时区名候选）。
    pub names: [String; 3],
    /// 版本 <2 的 int32。`None` = 版本 ≥2。
    pub bias_b: Option<i32>,
    /// int32（→ this+0xc，语义未定）。
    pub int_a: i32,
    /// 规则块（⑥）。
    pub rule: TziRule10_1,
    /// int16 计数（⑦）。负数/零不进循环，原样保留。
    pub count: i16,
    /// 计数 × [int32 + 规则块]（⑧）。计数 ≤0 时为空。
    pub rules: Vec<TziRuleEntry10_1>,
}

/// 从对象头之后的位置读取 10.1 TimeZoneInfo 体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`TIME_ZONE_INFO_CLASS_ID`] 并取得键
/// `key`。体内无对象槽，故无停止点。
pub fn read_time_zone_info_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    _table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TimeZoneInfoBody10_1> {
    let entry_offset = reader.position();
    let mut trial = reader.clone();
    let version = read_version_short(&mut trial, 2, "TimeZoneInfo", entry_offset)?;
    let bias_a = if version < 2 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    let mut names: [String; 3] = Default::default();
    for name in &mut names {
        *name = read_stream_string_10_1(&mut trial, max_bytes)?;
    }
    let bias_b = if version < 2 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    let int_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let rule = TziRule10_1::read(&mut trial)?;
    let count = trial.read_u16_le()? as i16;
    let count_usize = count.max(0) as usize;
    let rules_bytes = count_usize * (4 + TziRule10_1::BYTE_LEN);
    if rules_bytes > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "TimeZoneInfo 规则数组超过限额",
        ));
    }
    let mut rules_reader = trial.read_region(rules_bytes)?;
    let mut rules = Vec::with_capacity(count_usize);
    for _ in 0..count_usize {
        let int_a = rules_reader.read_u32_le()? as i32;
        let rule = TziRule10_1::read(&mut rules_reader)?;
        rules.push(TziRuleEntry10_1 { int_a, rule });
    }
    *reader = trial;
    Ok(TimeZoneInfoBody10_1 {
        key,
        version,
        bias_a,
        names,
        bias_b,
        int_a,
        rule,
        count,
        rules,
    })
}

/// TimeReference 体（SystemCore.dll `0x7ffebbf0`）的已确认字段。
///
/// 证据：确定的（定位链见模块文档；Save `0x7ffebc90` 恒写版本常量
/// `0x8006d050`=1，读写对称）。字段序：① int16 版本（`1 < (short)ver`
/// 拒绝）→ ② LoadObject(ITimeZoneInfo) 槽（样本无实例）→ ③ short（→
/// this+0xc）→ ④ short（→ this+0xe）。两个标量语义未定；② 为
/// `Unsupported` 停止点时二者为 `None` 且游标停在槽位起点。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeReferenceBody10_1 {
    /// 对象头键（槽位新对象时由宿主传入）。
    pub key: u32,
    /// 体首 short 版本（≤1；负值位模式拒绝，较原生更严）。
    pub version: u16,
    /// ITimeZoneInfo 槽分类。
    pub zone_info: TimeZoneInfoSlot10_1,
    /// short（→ this+0xc，语义未定）。`None` = 槽位停止未达。
    pub short_a: Option<i16>,
    /// short（→ this+0xe，语义未定）。`None` = 槽位停止未达。
    pub short_b: Option<i16>,
}

/// 从对象头之后的位置读取 10.1 TimeReference 体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`TIME_REFERENCE_CLASS_ID`] 并取得
/// 键 `key`。ITimeZoneInfo 槽的新对象类标识为 [`TIME_ZONE_INFO_CLASS_ID`]
/// 时整体解析其体，未知类为 [`TimeZoneInfoSlot10_1::Unsupported`]（游标
/// 回退到槽位起点、键不保留、登记表回滚），由宿主映射为停止原因。
pub fn read_time_reference_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TimeReferenceBody10_1> {
    let entry_offset = reader.position();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = read_version_short(&mut trial, 1, "TimeReference", entry_offset)?;

    let zone_info = {
        let slot_offset = trial.position();
        let pre_slot_table = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => TimeZoneInfoSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key }
                if staged
                    .registered_class(key)
                    .is_none_or(|id| id == TIME_ZONE_INFO_CLASS_ID) =>
            {
                TimeZoneInfoSlot10_1::BackReference {
                    class_id: staged.registered_class(key).unwrap_or(class_id),
                    key,
                }
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == TIME_ZONE_INFO_CLASS_ID => {
                let body = read_time_zone_info_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
                TimeZoneInfoSlot10_1::Body(Box::new(body))
            }
            ObjectEntry10_1::BackReference { class_id, key }
            | ObjectEntry10_1::NewObject { class_id, key } => {
                trial = reposition_to(&reader.clone(), slot_offset);
                staged = pre_slot_table;
                TimeZoneInfoSlot10_1::Unsupported { class_id, key }
            }
        }
    };
    let (short_a, short_b) = if matches!(zone_info, TimeZoneInfoSlot10_1::Unsupported { .. }) {
        (None, None)
    } else {
        (
            Some(trial.read_u16_le()? as i16),
            Some(trial.read_u16_le()? as i16),
        )
    };
    *table = staged;
    *reader = trial;
    Ok(TimeReferenceBody10_1 {
        key,
        version,
        zone_info,
        short_a,
        short_b,
    })
}

/// TimeExtent 体（SystemCore.dll `0x7ffe81f0`）的已确认字段。
///
/// 证据：确定的（定位链见模块文档；Save `0x7ffe82f0` 恒写版本常量
/// `0x8006d070`=2，与读侧版本 2 分支逐字段对称）。字段序：① int16 版本
/// （`2 < (short)ver` 拒绝）→ ② LoadObject(ITime) 起点槽 → ③ LoadObject
/// (ITime) 终点槽 → ④ LoadObject(ITimeReference) 槽 → ⑤ 版本 >1：1 字节
/// （→ this+0x14；Save 恒写，语义未定）。槽内 `Unsupported` 停止点出现时，
/// 其后字段保持 `None`/`Null` 占位且游标停在该槽位起点（由宿主决定是否
/// 继续）；因槽位在体中部，`Unsupported` 后的 ⑤ 按「未达」处理。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeExtentBody10_1 {
    /// 对象头键（槽位新对象时由宿主传入）。
    pub key: u32,
    /// 体首 short 版本（≤2；负值位模式拒绝，较原生更严）。
    pub version: u16,
    /// ITime 起点槽。
    pub start: TimeSlot10_1,
    /// ITime 终点槽。
    pub end: TimeSlot10_1,
    /// ITimeReference 槽。
    pub reference: TimeReferenceSlot10_1,
    /// 版本 >1 的 1 字节（Save 恒写）。`None` = 版本 ≤1 或前置槽位停止。
    pub tail_byte: Option<u8>,
    /// 槽位停止原因：任一子槽为未知类时记录（宿主映射为停止原因）。
    pub stop: Option<TimeExtentStop10_1>,
}

/// TimeExtent 体内槽位的停止原因。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeExtentStop10_1 {
    /// 起点/终点 ITime 槽未知类；`which` 0 = 起点、1 = 终点。
    TimeSlot {
        class_id: [u8; 16],
        key: u32,
        which: u8,
    },
    /// ITimeReference 槽未闭合，包括引用对象内的未知时区。
    ReferenceSlot { class_id: [u8; 16], key: u32 },
}

/// 从对象头之后的位置读取 10.1 TimeExtent 体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`TIME_EXTENT_CLASS_ID`] 并取得键
/// `key`。三个子槽的新对象按类标识分派（Time/TimeReference），未知类记入
/// [`TimeExtentBody10_1::stop`]（游标回退到该槽位起点、键不保留、登记表
/// 回滚）并返回已解析前缀。
pub fn read_time_extent_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TimeExtentBody10_1> {
    let entry_offset = reader.position();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = read_version_short(&mut trial, 2, "TimeExtent", entry_offset)?;

    // ② 起点 ITime 槽。
    let mut stop = None;
    let start = {
        let slot_offset = trial.position();
        let pre_slot_table = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => TimeSlot10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key }
                if staged
                    .registered_class(key)
                    .is_none_or(|id| id == TIME_CLASS_ID) =>
            {
                TimeSlot10_1::BackReference {
                    class_id: staged.registered_class(key).unwrap_or(class_id),
                    key,
                }
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == TIME_CLASS_ID => {
                let body = read_time_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
                TimeSlot10_1::Body(Box::new(body))
            }
            ObjectEntry10_1::BackReference { class_id, key }
            | ObjectEntry10_1::NewObject { class_id, key } => {
                trial = reposition_to(&reader.clone(), slot_offset);
                staged = pre_slot_table;
                stop = Some(TimeExtentStop10_1::TimeSlot {
                    class_id,
                    key,
                    which: 0,
                });
                TimeSlot10_1::Unsupported { class_id, key }
            }
        }
    };

    // ③ 终点 ITime 槽（前置停止时不读取）。
    let mut end = TimeSlot10_1::Null;
    let mut reference = TimeReferenceSlot10_1::Null;
    let mut tail_byte = None;
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre_slot_table = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key }
                if staged
                    .registered_class(key)
                    .is_none_or(|id| id == TIME_CLASS_ID) =>
            {
                end = TimeSlot10_1::BackReference {
                    class_id: staged.registered_class(key).unwrap_or(class_id),
                    key,
                };
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == TIME_CLASS_ID => {
                let body = read_time_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
                end = TimeSlot10_1::Body(Box::new(body));
            }
            ObjectEntry10_1::BackReference { class_id, key }
            | ObjectEntry10_1::NewObject { class_id, key } => {
                trial = reposition_to(&reader.clone(), slot_offset);
                staged = pre_slot_table;
                stop = Some(TimeExtentStop10_1::TimeSlot {
                    class_id,
                    key,
                    which: 1,
                });
                end = TimeSlot10_1::Unsupported { class_id, key };
            }
        }
    }

    // ④ ITimeReference 槽（前置停止时不读取）。
    if stop.is_none() {
        let slot_offset = trial.position();
        let pre_slot_table = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key }
                if staged
                    .registered_class(key)
                    .is_none_or(|id| id == TIME_REFERENCE_CLASS_ID) =>
            {
                reference = TimeReferenceSlot10_1::BackReference {
                    class_id: staged.registered_class(key).unwrap_or(class_id),
                    key,
                };
            }
            ObjectEntry10_1::NewObject { class_id, key } if class_id == TIME_REFERENCE_CLASS_ID => {
                let body = read_time_reference_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
                if matches!(body.zone_info, TimeZoneInfoSlot10_1::Unsupported { .. }) {
                    // 时区体边界未知，整个 ITimeReference 槽均未读完。
                    trial = reposition_to(reader, slot_offset);
                    staged = pre_slot_table;
                    stop = Some(TimeExtentStop10_1::ReferenceSlot { class_id, key });
                    reference = TimeReferenceSlot10_1::Unsupported { class_id, key };
                } else {
                    reference = TimeReferenceSlot10_1::Body(Box::new(body));
                }
            }
            ObjectEntry10_1::BackReference { class_id, key }
            | ObjectEntry10_1::NewObject { class_id, key } => {
                trial = reposition_to(&reader.clone(), slot_offset);
                staged = pre_slot_table;
                stop = Some(TimeExtentStop10_1::ReferenceSlot { class_id, key });
                reference = TimeReferenceSlot10_1::Unsupported { class_id, key };
            }
        }
    }

    // ⑤ 版本 >1：1 字节（前置停止时不读取）。
    if stop.is_none() && version > 1 {
        tail_byte = Some(trial.read_bytes(1)?[0]);
    }

    *table = staged;
    *reader = trial;
    Ok(TimeExtentBody10_1 {
        key,
        version,
        start,
        end,
        reference,
        tail_byte,
        stop,
    })
}

/// TimeData 段（LayerLib.dll `0x7870d8b0`，版本 ≤2）的段级字段。
///
/// 字段序同 `feature_layer_tail::TimeDataBody10_1`；差别在槽位：本模块把
/// ITimeReference 槽的新对象（类标识 [`TIME_REFERENCE_CLASS_ID`]）整体
/// 解析为体，未知类为 [`TimeReferenceSlot10_1::Unsupported`] 停止点。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDataSegment10_1 {
    /// 体首 short 版本（有符号 >2 拒绝；负值位模式一并拒绝，较原生更严）。
    pub version: u16,
    /// short（→ this+6），语义未定。
    pub short_a: i16,
    /// ITimeReference 槽分类。
    pub slot: TimeReferenceSlot10_1,
    /// 版本 ≥2 的 short（→ this+8），语义未定。
    pub tail_short: Option<i16>,
}

/// TimeTableDefinition 段（LayerLib.dll `0x7870df80`，版本 ≤1）的段级字段。
///
/// 字段序同 `feature_layer_tail::TimeTableDefinitionBody10_1`；差别在槽位：
/// ITimeExtent 槽的新对象（类标识 [`TIME_EXTENT_CLASS_ID`]）整体解析为体，
/// 未知类为 [`TimeExtentSlot10_1::Unsupported`] 停止点。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeTableSegment10_1 {
    /// 体首 short 版本（有符号 >1 拒绝；负值位模式一并拒绝）。
    pub version: u16,
    /// String ×4（→ this+8/+0xc/+0x10/+0x14；时间字段名候选），语义未定。
    pub strings: [String; 4],
    /// ITimeExtent 槽分类。
    pub slot: TimeExtentSlot10_1,
}

/// TimeDimensionDefinition 段（LayerLib.dll `0x7870e250`，版本 ≤1）的段级
/// 字段（无槽位、无停止点）。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDimensionSegment10_1 {
    /// 体首 short 版本（有符号 >1 拒绝；负值位模式一并拒绝）。
    pub version: u16,
    /// String ×2（→ this+4/+8；时间维度名/格式候选），语义未定。
    pub strings: [String; 2],
}

/// TimeDataDisplay 段（LayerLib.dll `0x7870f6c0`，版本 ≤1）的段级字段
/// （无槽位、无停止点）。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDisplaySegment10_1 {
    /// 体首 short 版本（有符号 >1 拒绝；负值位模式一并拒绝）。
    pub version: u16,
    /// short（→ this+4），语义未定。
    pub short_a: i16,
    /// double（→ this+8），语义未定。
    pub double_a: f64,
    /// int32（→ this+0x10），语义未定。
    pub int_a: i32,
    /// double（→ this+0x18），语义未定。
    pub double_b: f64,
    /// int32（→ this+0x20），语义未定。
    pub int_b: i32,
}

/// TimeData 族四段的停止原因（槽位未知类，游标停在该槽位起点）。
#[derive(Debug, Clone, PartialEq)]
pub enum TimeFamilyStop10_1 {
    /// TimeData 段的 ITimeReference 槽未知类。
    TimeDataSlot { class_id: [u8; 16], key: u32 },
    /// TimeTableDefinition 段的 ITimeExtent 槽未知类。
    TimeTableSlot { class_id: [u8; 16], key: u32 },
}

/// TimeData 族四段（FL 后继段 ㊶，版本 ≥0x20 分支）的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeDataFamily10_1 {
    /// TimeData 段；前置停止时为 `None`。
    pub time_data: Option<TimeDataSegment10_1>,
    /// TimeTableDefinition 段；前置停止时为 `None`。
    pub time_table: Option<TimeTableSegment10_1>,
    /// TimeDimensionDefinition 段；前置停止时为 `None`。
    pub time_dimension: Option<TimeDimensionSegment10_1>,
    /// TimeDataDisplay 段；前置停止时为 `None`。
    pub time_display: Option<TimeDisplaySegment10_1>,
    /// 槽位停止原因；`None` = 四段全部完整消费。
    pub stop: Option<TimeFamilyStop10_1>,
    /// 停止点（或四段终点）之后的剩余字节数（相对传入 Reader 窗口）。
    pub remaining_bytes: usize,
}

/// 从 TimeData 段版本 short 的位置读取 TimeData 族四段（不含版本 ≥0x20
/// 门——该门由调用者按 FL 架构版本判断），失败时不推进游标、不修改登记表。
///
/// 段序与版本门按 LayerLib.dll 反编译（证据：确定的）：TimeData（≤2）→
/// TimeTableDefinition（≤1）→ TimeDimensionDefinition（≤1）→
/// TimeDataDisplay（≤1）。两个槽位的新对象按类标识分派
/// （TimeReference/TimeExtent，见模块文档），未知类记入
/// [`TimeDataFamily10_1::stop`]（游标回退到该槽位起点、键不保留）。
pub fn read_time_data_family_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<TimeDataFamily10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let mut stop = None;

    // ① TimeData（版本 ≤2）。
    let time_data = match read_time_data_segment(&mut trial, max_bytes, &mut staged)? {
        Ok(segment) => Some(segment),
        Err(reason) => {
            stop = Some(reason);
            None
        }
    };

    // ② TimeTableDefinition（版本 ≤1）。
    let mut time_table = None;
    if stop.is_none() {
        let entry_offset = trial.position();
        let version = read_version_short(&mut trial, 1, "TimeTableDefinition", entry_offset)?;
        let mut strings: [String; 4] = Default::default();
        for s in &mut strings {
            *s = read_stream_string_10_1(&mut trial, max_bytes)?;
        }
        let mut slot = TimeExtentSlot10_1::Null;
        let slot_offset = trial.position();
        let pre_slot_table = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, key } => {
                let registered = staged.registered_class(key);
                if registered.is_some_and(|id| id != TIME_EXTENT_CLASS_ID) {
                    trial = reposition_to(reader, slot_offset);
                    staged = pre_slot_table;
                    stop = Some(TimeFamilyStop10_1::TimeTableSlot { class_id, key });
                } else {
                    slot = TimeExtentSlot10_1::BackReference {
                        class_id: registered.unwrap_or(class_id),
                        key,
                    };
                }
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                if class_id == TIME_EXTENT_CLASS_ID {
                    let body = read_time_extent_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
                    if body.stop.is_none() {
                        slot = TimeExtentSlot10_1::Body(Box::new(body));
                    }
                }
                if matches!(slot, TimeExtentSlot10_1::Null) {
                    // 对象头已登记，体未闭合时回退到整个 ITimeExtent 槽。
                    trial = reposition_to(reader, slot_offset);
                    staged = pre_slot_table;
                    stop = Some(TimeFamilyStop10_1::TimeTableSlot { class_id, key });
                }
            }
        }
        if stop.is_none() {
            time_table = Some(TimeTableSegment10_1 {
                version,
                strings,
                slot,
            });
        }
    }

    // ③ TimeDimensionDefinition（版本 ≤1）。
    let time_dimension = stop
        .is_none()
        .then(|| read_time_dimension_segment_10_1(&mut trial, max_bytes))
        .transpose()?;

    // ④ TimeDataDisplay（版本 ≤1）。
    let time_display = stop
        .is_none()
        .then(|| read_time_display_segment_10_1(&mut trial))
        .transpose()?;

    let remaining_bytes = trial.remaining();
    *table = staged;
    *reader = trial;
    Ok(TimeDataFamily10_1 {
        time_data,
        time_table,
        time_dimension,
        time_display,
        stop,
        remaining_bytes,
    })
}

/// 读取独立的 TimeData 段；不消费宿主中紧随其后的其他时间段。
/// 未知类或截断均不修改调用者游标和引用表。
pub(crate) fn read_time_data_segment_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<TimeDataSegment10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let segment = read_time_data_segment(&mut trial, max_bytes, &mut staged)?.map_err(|stop| {
        io::Error::new(
            ErrorKind::Unsupported,
            format!("TimeData 对象槽未闭合：{stop:?}"),
        )
    })?;
    *reader = trial;
    *table = staged;
    Ok(segment)
}

fn read_time_data_segment(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<Result<TimeDataSegment10_1, TimeFamilyStop10_1>> {
    let start = reader.position();
    let version = read_version_short(reader, 2, "TimeData", start)?;
    let short_a = reader.read_u16_le()? as i16;
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let slot = match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => TimeReferenceSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            TimeReferenceSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == TIME_REFERENCE_CLASS_ID => {
            let body = read_time_reference_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
            if body.short_b.is_none() {
                return Ok(Err(TimeFamilyStop10_1::TimeDataSlot { class_id, key }));
            }
            TimeReferenceSlot10_1::Body(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Ok(Err(TimeFamilyStop10_1::TimeDataSlot { class_id, key }));
        }
    };
    let tail_short = (version >= 2)
        .then(|| trial.read_u16_le().map(|value| value as i16))
        .transpose()?;
    *reader = trial;
    *table = staged;
    Ok(Ok(TimeDataSegment10_1 {
        version,
        short_a,
        slot,
        tail_short,
    }))
}

pub(crate) fn read_time_dimension_segment_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<TimeDimensionSegment10_1> {
    let mut trial = reader.clone();
    let version = read_version_short(&mut trial, 1, "TimeDimensionDefinition", reader.position())?;
    let strings = [
        read_stream_string_10_1(&mut trial, max_bytes)?,
        read_stream_string_10_1(&mut trial, max_bytes)?,
    ];
    *reader = trial;
    Ok(TimeDimensionSegment10_1 { version, strings })
}

pub(crate) fn read_time_display_segment_10_1(
    reader: &mut Reader<'_>,
) -> io::Result<TimeDisplaySegment10_1> {
    let mut trial = reader.clone();
    let version = read_version_short(&mut trial, 1, "TimeDataDisplay", reader.position())?;
    let short_a = trial.read_u16_le()? as i16;
    let double_a = trial.read_f64_le()?;
    let int_a = trial.read_u32_le()? as i32;
    let double_b = trial.read_f64_le()?;
    let int_b = trial.read_u32_le()? as i32;
    *reader = trial;
    Ok(TimeDisplaySegment10_1 {
        version,
        short_a,
        double_a,
        int_a,
        double_b,
        int_b,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 位精确地写一个 AfCore String::Read 布局字符串（cb 含终止零；cb=0
    /// 表示无载荷空串）。
    fn put_string(buf: &mut Vec<u8>, payload: &str) {
        let units: Vec<u16> = payload.encode_utf16().collect();
        let cb = units.len() * 2 + 2;
        buf.extend_from_slice(&(cb as u32).to_le_bytes());
        for u in &units {
            buf.extend_from_slice(&u.to_le_bytes());
        }
        buf.extend_from_slice(&[0, 0]);
    }

    fn null_slot(buf: &mut Vec<u8>) {
        buf.extend_from_slice(&[0u8; 16]);
    }

    fn object_entry(buf: &mut Vec<u8>, class_id: &[u8; 16], key: u32) {
        buf.extend_from_slice(class_id);
        buf.extend_from_slice(&key.to_le_bytes());
    }

    /// TimeReference 体合成字节（版本、时区槽形态、两个 short）。
    fn time_reference_bytes(zone_info: Option<&[u8]>) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        match zone_info {
            Some(entry) => buf.extend_from_slice(entry),
            None => null_slot(&mut buf),
        }
        buf.extend_from_slice(&(-1i16).to_le_bytes());
        buf.extend_from_slice(&0i16.to_le_bytes());
        buf
    }

    /// TimeZoneInfo 体合成字节（版本 2 分支：无 bias）。
    fn time_zone_info_bytes() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&2u16.to_le_bytes());
        put_string(&mut buf, "中国标准时间");
        put_string(&mut buf, "中国夏令时间");
        put_string(&mut buf, "(UTC+08:00)");
        buf.extend_from_slice(&480i32.to_le_bytes());
        // 基本规则和一条年度规则，名称及日期按定长字段写入。
        buf.extend(0_i32.to_le_bytes());
        buf.extend([0x31; 64]);
        buf.extend([0x32; 16]);
        buf.extend(9_i32.to_le_bytes());
        buf.extend([0x41; 64]);
        buf.extend([0x42; 16]);
        buf.extend(60_i32.to_le_bytes());
        buf.extend(1_i16.to_le_bytes());
        buf.extend(1_i32.to_le_bytes());
        buf.extend([0; TziRule10_1::BYTE_LEN]);
        buf
    }

    #[test]
    fn time_reference_with_null_zone_info() {
        let bytes = time_reference_bytes(None);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let body = read_time_reference_body_10_1(&mut reader, 256, &mut table, 7).unwrap();
        assert_eq!(body.key, 7);
        assert_eq!(body.version, 1);
        assert_eq!(body.zone_info, TimeZoneInfoSlot10_1::Null);
        assert_eq!(body.short_a, Some(-1));
        assert_eq!(body.short_b, Some(0));
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn time_reference_with_nested_time_zone_info() {
        let entry = {
            let mut e = Vec::new();
            object_entry(&mut e, &TIME_ZONE_INFO_CLASS_ID, 99);
            e.extend_from_slice(&time_zone_info_bytes());
            e
        };
        let bytes = time_reference_bytes(Some(&entry));
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let body = read_time_reference_body_10_1(&mut reader, 256, &mut table, 7).unwrap();
        match body.zone_info {
            TimeZoneInfoSlot10_1::Body(tzi) => {
                assert_eq!(tzi.key, 99);
                assert_eq!(tzi.version, 2);
                assert_eq!(tzi.bias_a, None);
                assert_eq!(tzi.names[0], "中国标准时间");
                assert_eq!(tzi.int_a, 480);
                assert_eq!(tzi.rule.name_a, [0x31; 64]);
                assert_eq!(tzi.rule.date_a, [0x32; 16]);
                assert_eq!(tzi.rule.int_b, 9);
                assert_eq!(tzi.rule.name_b, [0x41; 64]);
                assert_eq!(tzi.rule.date_b, [0x42; 16]);
                assert_eq!(tzi.rule.int_c, 60);
                assert_eq!(tzi.count, 1);
                assert_eq!(tzi.rules.len(), 1);
                assert_eq!(tzi.rules[0].int_a, 1);
            }
            other => panic!("应解析为嵌套 TimeZoneInfo 体，实际 {other:?}"),
        }
        assert!(table.contains(99), "嵌套新对象键应已登记");
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn time_reference_slot_unsupported_rolls_back() {
        let mut entry = Vec::new();
        object_entry(&mut entry, &[0xab; 16], 5);
        entry.extend_from_slice(&[1, 2, 3]);
        let bytes = time_reference_bytes(Some(&entry));
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let body = read_time_reference_body_10_1(&mut reader, 256, &mut table, 7).unwrap();
        match body.zone_info {
            TimeZoneInfoSlot10_1::Unsupported { class_id, key } => {
                assert_eq!(class_id, [0xab; 16]);
                assert_eq!(key, 5);
            }
            other => panic!("应为 Unsupported 停止，实际 {other:?}"),
        }
        // 游标停在槽位起点；后续两个 short 未读（None）。
        assert_eq!(body.short_a, None);
        assert_eq!(body.short_b, None);
        assert_eq!(reader.position(), 2);
        assert!(!table.contains(5), "停止路径不保留键");
    }

    #[test]
    fn time_reference_zone_back_reference_uses_registered_class() {
        for registered in [TIME_ZONE_INFO_CLASS_ID, TIME_CLASS_ID] {
            let mut previous = Vec::new();
            object_entry(&mut previous, &registered, 5);
            let mut table = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&previous), &mut table).unwrap();
            let before = table.clone();

            let mut entry = Vec::new();
            object_entry(&mut entry, &[0xab; 16], 5);
            let bytes = time_reference_bytes(Some(&entry));
            let mut reader = Reader::new(&bytes);
            let body = read_time_reference_body_10_1(&mut reader, 256, &mut table, 7).unwrap();
            if registered == TIME_ZONE_INFO_CLASS_ID {
                assert_eq!(
                    body.zone_info,
                    TimeZoneInfoSlot10_1::BackReference {
                        class_id: TIME_ZONE_INFO_CLASS_ID,
                        key: 5,
                    }
                );
                assert_eq!(reader.position(), bytes.len());
            } else {
                assert!(matches!(
                    body.zone_info,
                    TimeZoneInfoSlot10_1::Unsupported { key: 5, .. }
                ));
                assert_eq!(body.short_a, None);
                assert_eq!(reader.position(), 2);
            }
            assert_eq!(table, before);
        }
    }

    #[test]
    fn time_extent_round_trip() {
        // 版本 2：起点/终点 Time 体（判别 2 WKSDateTime 分支）+ TimeReference
        // 回引 + 尾字节。
        let ref_entry = {
            let mut e = Vec::new();
            object_entry(&mut e, &TIME_REFERENCE_CLASS_ID, 3);
            e.extend_from_slice(&time_reference_bytes(None));
            e
        };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes());
        let mut time_body = Vec::new();
        time_body.extend_from_slice(&1u16.to_le_bytes());
        time_body.extend_from_slice(&2i32.to_le_bytes());
        time_body.extend_from_slice(&[0x42; 16]);
        object_entry(&mut bytes, &TIME_CLASS_ID, 1);
        bytes.extend_from_slice(&time_body);
        object_entry(&mut bytes, &TIME_CLASS_ID, 2);
        bytes.extend_from_slice(&time_body);
        bytes.extend_from_slice(&ref_entry);
        bytes.push(0x01);

        let mut reader = Reader::new(&bytes);
        // 预登记回引目标键 3。
        let mut pre = RefTable10_1::default();
        let _ = pre.register(3);
        let body = read_time_extent_body_10_1(&mut reader, 256, &mut pre, 9).unwrap();
        assert_eq!(body.version, 2);
        match (&body.start, &body.end) {
            (TimeSlot10_1::Body(a), TimeSlot10_1::Body(b)) => {
                assert_eq!(a.value, TimeValue10_1::WksDateTime([0x42; 16]));
                assert_eq!(b.key, 2);
            }
            other => panic!("两槽应解析为 Time 体，实际 {other:?}"),
        }
        match body.reference {
            TimeReferenceSlot10_1::BackReference { key, .. } => assert_eq!(key, 3),
            other => panic!("第三槽应为回引，实际 {other:?}"),
        }
        assert_eq!(body.tail_byte, Some(0x01));
        assert_eq!(body.stop, None);
        // 第三槽按回引消费（键 3 预登记）：其后预置的 22 字节 TR 体不被
        // 消费（该形态仅在真实流中省略——此处仅验证回引语义）。
        assert_eq!(reader.position(), bytes.len() - 22);
    }

    #[test]
    fn time_extent_unknown_time_slot_stops() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u16.to_le_bytes());
        object_entry(&mut bytes, &[0xcd; 16], 4);
        bytes.extend_from_slice(&[9, 9, 9]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let body = read_time_extent_body_10_1(&mut reader, 256, &mut table, 9).unwrap();
        assert_eq!(
            body.stop,
            Some(TimeExtentStop10_1::TimeSlot {
                class_id: [0xcd; 16],
                key: 4,
                which: 0,
            })
        );
        assert_eq!(reader.position(), 2, "游标应停在槽位起点");
        assert!(!table.contains(4));
    }

    #[test]
    fn time_extent_rejects_back_references_to_wrong_interfaces() {
        for (index, expected, wrong) in [
            (0, TIME_CLASS_ID, TIME_REFERENCE_CLASS_ID),
            (1, TIME_CLASS_ID, TIME_REFERENCE_CLASS_ID),
            (2, TIME_REFERENCE_CLASS_ID, TIME_CLASS_ID),
        ] {
            let mut registered = Vec::new();
            object_entry(&mut registered, &wrong, 4);
            let mut table = RefTable10_1::new();
            read_object_entry_10_1(&mut Reader::new(&registered), &mut table).unwrap();
            let before = table.clone();
            let mut bytes = 2_u16.to_le_bytes().to_vec();
            bytes.resize(2 + 16 * index, 0);
            let offset = bytes.len();
            object_entry(&mut bytes, &expected, 4);
            let mut reader = Reader::new(&bytes);
            let body = read_time_extent_body_10_1(&mut reader, 256, &mut table, 9).unwrap();
            let stop = if index == 2 {
                TimeExtentStop10_1::ReferenceSlot {
                    class_id: expected,
                    key: 4,
                }
            } else {
                TimeExtentStop10_1::TimeSlot {
                    class_id: expected,
                    key: 4,
                    which: index as u8,
                }
            };
            assert_eq!(body.stop, Some(stop));
            assert_eq!(reader.position(), offset);
            assert_eq!(table, before);
        }
    }

    #[test]
    fn time_body_discriminants() {
        let mut oa = Vec::new();
        oa.extend_from_slice(&0u16.to_le_bytes());
        oa.extend_from_slice(&1i32.to_le_bytes());
        oa.extend_from_slice(&(-1i64).to_le_bytes());
        let mut reader = Reader::new(&oa);
        let mut table = RefTable10_1::default();
        let body = read_time_body_10_1(&mut reader, 64, &mut table, 1).unwrap();
        assert_eq!(body.version, 0);
        assert_eq!(body.value, TimeValue10_1::OaDate(-1));

        let mut bad = Vec::new();
        bad.extend_from_slice(&1u16.to_le_bytes());
        bad.extend_from_slice(&3i32.to_le_bytes());
        bad.extend_from_slice(&[0u8; 16]);
        let mut reader = Reader::new(&bad);
        let mut table = RefTable10_1::default();
        let err = read_time_body_10_1(&mut reader, 64, &mut table, 1).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0, "失败不推进游标");
    }

    #[test]
    fn version_gates_reject() {
        // TimeReference 版本 2 → 拒绝。
        let mut bytes = vec![2u8, 0];
        bytes.extend_from_slice(&[0u8; 22]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        assert_eq!(
            read_time_reference_body_10_1(&mut reader, 64, &mut table, 1)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(reader.position(), 0);

        // TimeExtent 版本 3 → 拒绝。
        let mut bytes = vec![3u8, 0];
        bytes.extend_from_slice(&[0u8; 40]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        assert_eq!(
            read_time_extent_body_10_1(&mut reader, 64, &mut table, 1)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );

        // Time 版本 2 → 拒绝。
        let mut bytes = vec![2u8, 0, 1, 0, 0, 0];
        bytes.extend_from_slice(&[0u8; 8]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        assert_eq!(
            read_time_body_10_1(&mut reader, 64, &mut table, 1)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );

        // TimeZoneInfo 版本 0xffff（负值位模式）→ 拒绝。
        let mut bytes = vec![0xff, 0xff];
        bytes.extend_from_slice(&[0u8; 8]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        assert_eq!(
            read_time_zone_info_body_10_1(&mut reader, 64, &mut table, 1)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn time_zone_info_version_1_has_bias_fields() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&(-60i32).to_le_bytes());
        put_string(&mut bytes, "");
        put_string(&mut bytes, "");
        put_string(&mut bytes, "");
        bytes.extend_from_slice(&7i32.to_le_bytes());
        bytes.extend_from_slice(&11i32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; TziRule10_1::BYTE_LEN]);
        bytes.extend_from_slice(&(-3i16).to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let body = read_time_zone_info_body_10_1(&mut reader, 256, &mut table, 2).unwrap();
        assert_eq!(body.bias_a, Some(-60));
        assert_eq!(body.bias_b, Some(7));
        assert_eq!(body.int_a, 11);
        assert_eq!(body.count, -3);
        assert!(body.rules.is_empty(), "负计数不进循环");
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn time_data_family_sample_shape() {
        // 仓库样本 /Maps:0x156e..0x15d7 的分解（逐字节对照，辅助观察）：
        // TimeData ver2/short0/null/tail0 + TimeTable ver1/4×空串/null +
        // TimeDimension ver1/2×空串 + TimeDisplay ver1/0/0.0/4/0.0/8。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&0i16.to_le_bytes());
        null_slot(&mut bytes);
        bytes.extend_from_slice(&0i16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        for _ in 0..4 {
            put_string(&mut bytes, "");
        }
        null_slot(&mut bytes);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        put_string(&mut bytes, "");
        put_string(&mut bytes, "");
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&0i16.to_le_bytes());
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&4i32.to_le_bytes());
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&8i32.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let family = read_time_data_family_10_1(&mut reader, 256, &mut table).unwrap();
        assert!(family.stop.is_none());
        let td = family.time_data.unwrap();
        assert_eq!(td.version, 2);
        assert_eq!(td.slot, TimeReferenceSlot10_1::Null);
        assert_eq!(td.tail_short, Some(0));
        let tt = family.time_table.unwrap();
        assert_eq!(tt.version, 1);
        assert_eq!(tt.slot, TimeExtentSlot10_1::Null);
        assert_eq!(family.time_dimension.unwrap().version, 1);
        let tdisp = family.time_display.unwrap();
        assert_eq!((tdisp.int_a, tdisp.int_b), (4, 8));
        assert_eq!(family.remaining_bytes, 0);
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn time_data_family_slot_unsupported_keeps_prefix() {
        // TimeData 槽未知类 → 停止记录、前缀 TimeData 段标量保留、游标停
        // 在槽位起点。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&2i16.to_le_bytes());
        object_entry(&mut bytes, &[0xee; 16], 8);
        bytes.extend_from_slice(&[1, 2, 3, 4]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let family = read_time_data_family_10_1(&mut reader, 256, &mut table).unwrap();
        assert_eq!(
            family.stop,
            Some(TimeFamilyStop10_1::TimeDataSlot {
                class_id: [0xee; 16],
                key: 8,
            })
        );
        assert!(family.time_data.is_none());
        assert!(family.time_table.is_none());
        assert_eq!(reader.position(), 4);
        assert!(!table.contains(8));
    }

    #[test]
    fn time_data_family_with_time_reference_and_extent_bodies() {
        // TimeData 槽 = TimeReference 新对象（时区槽空），TimeTable 槽 =
        // TimeExtent 新对象（两 Time 空 + 回引 + 尾字节）。
        let tr_entry = {
            let mut e = Vec::new();
            object_entry(&mut e, &TIME_REFERENCE_CLASS_ID, 10);
            e.extend_from_slice(&time_reference_bytes(None));
            e
        };
        let te_entry = {
            let mut e = Vec::new();
            object_entry(&mut e, &TIME_EXTENT_CLASS_ID, 11);
            e.extend_from_slice(&1u16.to_le_bytes()); // 版本 1：无尾字节
            null_slot(&mut e); // 起点
            null_slot(&mut e); // 终点
            let mut backref = Vec::new();
            object_entry(&mut backref, &TIME_REFERENCE_CLASS_ID, 10);
            e.extend_from_slice(&backref);
            e
        };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&0i16.to_le_bytes());
        bytes.extend_from_slice(&tr_entry);
        bytes.extend_from_slice(&0i16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        for _ in 0..4 {
            put_string(&mut bytes, "");
        }
        let extent_offset = bytes.len();
        bytes.extend_from_slice(&te_entry);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        put_string(&mut bytes, "");
        put_string(&mut bytes, "");
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&0i16.to_le_bytes());
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes());
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&0i32.to_le_bytes());

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::default();
        let family = read_time_data_family_10_1(&mut reader, 256, &mut table).unwrap();
        assert!(family.stop.is_none());
        match family.time_data.unwrap().slot {
            TimeReferenceSlot10_1::Body(tr) => {
                assert_eq!(tr.key, 10);
                assert_eq!(tr.zone_info, TimeZoneInfoSlot10_1::Null);
            }
            other => panic!("TimeData 槽应为 TimeReference 体，实际 {other:?}"),
        }
        match family.time_table.unwrap().slot {
            TimeExtentSlot10_1::Body(te) => {
                assert_eq!(te.key, 11);
                assert_eq!(te.start, TimeSlot10_1::Null);
                match te.reference {
                    TimeReferenceSlot10_1::BackReference { key, .. } => assert_eq!(key, 10),
                    other => panic!("TimeExtent 第三槽应为回引，实际 {other:?}"),
                }
            }
            other => panic!("TimeTable 槽应为 TimeExtent 体，实际 {other:?}"),
        }
        assert!(table.contains(10) && table.contains(11));
        assert_eq!(reader.position(), bytes.len());

        let mut wrong = bytes[..extent_offset].to_vec();
        object_entry(&mut wrong, &TIME_EXTENT_CLASS_ID, 10);
        wrong.extend_from_slice(&bytes[extent_offset + te_entry.len()..]);
        let mut reader = Reader::new(&wrong);
        let mut table = RefTable10_1::new();
        let family = read_time_data_family_10_1(&mut reader, 256, &mut table).unwrap();
        assert!(matches!(
            family.stop,
            Some(TimeFamilyStop10_1::TimeTableSlot {
                class_id: TIME_EXTENT_CLASS_ID,
                key: 10,
            })
        ));
        assert_eq!(reader.position(), extent_offset);
        assert_eq!(table.registered_class(10), Some(TIME_REFERENCE_CLASS_ID));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn truncation_sweep_leaves_state_untouched() {
        // 对 TimeReference（含嵌套 TimeZoneInfo）规范体做逐字节截断：任何
        // 前缀长度都必须报错且游标/登记表保持原位。
        let entry = {
            let mut e = Vec::new();
            object_entry(&mut e, &TIME_ZONE_INFO_CLASS_ID, 99);
            e.extend_from_slice(&time_zone_info_bytes());
            e
        };
        let bytes = time_reference_bytes(Some(&entry));
        for cut in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut table = RefTable10_1::default();
            let result = read_time_reference_body_10_1(&mut reader, 256, &mut table, 7);
            assert!(result.is_err(), "截断到 {cut} 字节应失败");
            assert_eq!(reader.position(), 0, "失败游标不推进（cut={cut}）");
            assert!(table.is_empty(), "失败不修改登记表");
        }
    }

    #[test]
    fn time_zone_info_rejects_rule_array_over_resource_limit() {
        let bytes = time_zone_info_bytes();
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let error = read_time_zone_info_body_10_1(&mut reader, 175, &mut table, 1).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }
}
