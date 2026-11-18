//! ArcMap 10.1 DisplayCore.dll IColorRamp 族（GradientFillSymbol 颜色梯度槽的
//! 流内实现类）对象体严格读取子集与槽位分派。
//!
//! 证据来源：`docs/arcmap-10.1-display-slots-round3.md`（G 轮研究记录）。四类
//! 宿主经「esri.clsid 声明模块 → RTTI TypeDescriptor → COL → IPersistStream
//! 虚表 → GetClassID 读 `.rdata` CLSID 常量」链在 DisplayCore.dll 10.1.0.3035
//! （SHA-256 `9b3e61f9…6e4d2`，ImageBase `0x6eb30000`）中逐类闭环：
//!
//! - **RandomColorRamp** `{BEB87094-C0B4-11D0-8379-080009B996CC}`：IPersistStream
//!   虚表 `0x6ec0c558`（COL `0x6ec1a0b4`，接口偏移 0x54），GetClassID
//!   `0x6eb3c4f0`（读常量 `0x6ec0c588`）、Load `0x6eb3c190`、Save `0x6eb3c350`。
//! - **MultiPartColorRamp** `{BEB87099-…}`：虚表 `0x6ec0c19c`，GetClassID
//!   `0x6eb39860`（读常量 `0x6ec0c230`）、Load `0x6eb395b0`、Save `0x6eb39720`。
//! - **PresetColorRamp** `{BEB8709A-…}`：虚表 `0x6ec0c378`，GetClassID
//!   `0x6eb3a6e0`（读常量 `0x6ec0c3b8`）、Load `0x6eb3ae20`、Save `0x6eb3afa0`。
//! - **AlgorithmicColorRamp** `{BEB8709B-…}`：虚表 `0x6ec0b468`，GetClassID
//!   `0x6eb31010`（读常量 `0x6ec0b544`）、Load `0x6eb316c0`、Save `0x6eb317f0`。
//!
//! 名称字段均调用 AfCore 自由函数 ReadString/WriteString（BSTR 码元数），不是 String::Read/Write。
//!
//! ## 字段序（Load 反编译 + Save 对称互证；证据强度：确定的；样本 0 实例）
//!
//! **RandomColorRamp::Load `0x6eb3c190`**（版本 `2 < (short)ver` 拒绝
//! `0x80004005`；Save 恒写版本 1，常量 `DAT_6ec0b31c`=1）：
//!
//! 1. int16 版本；
//! 2. String → this−0x30（名称候选）；
//! 3. int32（Load 读入局部变量后将 this−0x34 清零；Save 写该槽，该槽也是
//!    IColorRamp::Size setter 写入的请求值；流内 4 字节原样保留）；
//! 4. 2 字节 short，整体与零比较后写入 bool this−0x28；流内值原样保留；
//! 5. int32 → this−0x2c；
//! 6. short ×6 → this−0x26（装载同写 this−0x24 同值）/ −0x22 / −0x20 /
//!    −0x1e / −0x1c / −0x1a（IRandomColorRamp 色相/饱和度/明度端点候选）。
//!
//! **MultiPartColorRamp::Load `0x6eb395b0`**（`3 < (short)ver` 拒绝；Save 版
//! 本表 `DAT_6ec0b324`（5 项短整）= docVer 0..4 → 1,1,1,1,2，逐项字节读核）：
//!
//! 1. int16 版本；
//! 2. String → this−0x44（名称候选）；
//! 3. int32 count（Save 写 this−0x24；读取器以 max_bytes 设循环上限）；
//! 4. count × LoadObject（请求 IID `DAT_6ec0b524` = IColorRamp
//!    `{BEB87091-…}`，类型库定名）→ 子梯度槽：经 [`read_color_ramp_slot_10_1`]
//!    分派（四闭合类递归解析，其余实现类停止；Save 槽 this−0x28 数组顺序
//!    写出，Load 顺序消费，不倒序）；
//! 5. 版本 ≥2：count × 8 字节（count 个 double 原样保留；Save 在 docVer ≥4
//!    分支写 this−0x14 缓冲 count×8 字节——Load 门「版本 ≥2」与 Save 门
//!    「docVer ≥4」不对称。旧版本每个子渐变的权重初始化为 1.0；该数组
//!    参与 CreateRamp 的子渐变颜色数量分配，见 `docs/mxd-terra-host-color.md`）。
//!
//! **PresetColorRamp::Load `0x6eb3ae20`**（`2 < (short)ver` 拒绝；Save 恒写
//! 版本 1，常量 `DAT_6ec0b320`=1）：
//!
//! 1. int16 版本；2. String；3. int32；4. int32 count；
//! 5. count × LoadObject（请求 IID `DAT_6ec0b4f4` = IColor）→ 颜色槽：经
//!    [`crate::objects::display_color::read_color_slot_10_1`] 五类闭合颜色
//!    分派（Save 槽 this−0x14 数组）。
//!
//! **AlgorithmicColorRamp::Load `0x6eb316c0`**（`2 < (short)ver` 拒绝；Save
//! 恒写版本 1，常量 `DAT_6ec0b358`=1）：
//!
//! 1. int16 版本；2. String → this−0x20；3. int32 → this−0x1c；
//! 4. LoadObject（IColor）×2 → 起止颜色槽（Save 槽 this−0x28 / this−0x24，
//!    顺序对称）。
//!
//! 同族负结果（反编译 RTTI 层级证据）：**ColorRampElements**
//! `{CCDBF37E-7A63-4788-A7CB-2A5B0E98A180}`（esri.clsid DisplayCore 条目）层
//! 级内无 IPersistStream 虚表——与 SimpleTextPath 同形态的不可流内持久化类，
//! 不实现读取器（见研究记录）。
//!
//! ## 槽位分派
//!
//! [`read_color_ramp_slot_10_1`]：GradientFillSymbol 的 IColorRamp 槽（以及
//! MultiPart 子梯度槽共用）——空标记与回引可消费；新对象按类分派四闭合类，
//! 其余实现类回退到槽位起点、不保留该键，以停止原因返回。建议接入点：
//! `display_symbol.rs` 的 `read_unclosed_slot_10_1`
//! （`DisplaySlotKind10_1::ColorRamp`）替换为本分派（接线归整合轮）。
//!
//! 事务性与限额约定同仓内其他读取器：调用者已读对象头并登记新键，从对象头
//! 之后的位置传入 Reader；全部字段在 trial 游标与 trial 登记表上读取，成功
//! （含停止点）才提交；版本 short 一律按原生有符号比较设上限并拒绝负值位模
//! 式（较原生更严）；截断与非法长度明确报错；`max_bytes` 约束全部字符串声明
//! 长度与循环计数（读取器自身限额，非原生限制）。仓库参考样本
//! `samples/测试底图.mxd` 中四类 0 实例，全部结论无样本字节佐证，单元测试
//! 仅合成输入，不代表原生已验证。

use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::display_symbol::{DisplaySlotKind10_1, SymbolStop10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_bstr_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// RandomColorRamp 类标识（CLSID）的 16 字节内存序。证据：esri.clsid 声明
/// （模块 DisplayCore.dll）+ GetClassID `0x6eb3c4f0` 读常量 `0x6ec0c588` 逐
/// 字节互证。
pub const RANDOM_COLOR_RAMP_CLASS_ID: [u8; 16] = [
    0x94, 0x70, 0xb8, 0xbe, 0xb4, 0xc0, 0xd0, 0x11, 0x83, 0x79, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// MultiPartColorRamp 类标识。证据：esri.clsid + GetClassID `0x6eb39860` 读
/// 常量 `0x6ec0c230` 互证。
pub const MULTI_PART_COLOR_RAMP_CLASS_ID: [u8; 16] = [
    0x99, 0x70, 0xb8, 0xbe, 0xb4, 0xc0, 0xd0, 0x11, 0x83, 0x79, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// PresetColorRamp 类标识。证据：esri.clsid + GetClassID `0x6eb3a6e0` 读常量
/// `0x6ec0c3b8` 互证。
pub const PRESET_COLOR_RAMP_CLASS_ID: [u8; 16] = [
    0x9a, 0x70, 0xb8, 0xbe, 0xb4, 0xc0, 0xd0, 0x11, 0x83, 0x79, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// AlgorithmicColorRamp 类标识。证据：esri.clsid + GetClassID `0x6eb31010` 读
/// 常量 `0x6ec0b544` 互证。
pub const ALGORITHMIC_COLOR_RAMP_CLASS_ID: [u8; 16] = [
    0x9b, 0x70, 0xb8, 0xbe, 0xb4, 0xc0, 0xd0, 0x11, 0x83, 0x79, 0x08, 0x00, 0x09, 0xb9, 0x96, 0xcc,
];

/// 各类 Load 接受的最大版本（有符号 short 比较；负值位模式原生通过，本读取
/// 器一并拒绝）。Random/MultiPart ≤1 时 MultiPart 为 2——分别常量化。
const MAX_RANDOM_VERSION: i16 = 1;
const MAX_MULTI_PART_VERSION: i16 = 2;
const MAX_PRESET_VERSION: i16 = 1;
const MAX_ALGORITHMIC_VERSION: i16 = 1;

/// 读取流内 int16 版本并按原生有符号上限校验（负值位模式拒绝，较原生更严）。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    label: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let raw = trial.read_u16_le()?;
    let signed = raw as i16;
    if signed < 0 || signed > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{label} 对象体 offset 0x{entry_offset:x}: 版本位模式 {raw}（0x{raw:04x}）\
                 超出原生接受范围 0..={max}"
            ),
        ));
    }
    Ok(raw)
}

fn read_i32_le(trial: &mut Reader<'_>) -> io::Result<i32> {
    Ok(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
}

/// 读取 8 字节 LE u64（Reader 无 u64 原语，本模块用原样位模式保存 double）。
fn read_u64_le(trial: &mut Reader<'_>) -> io::Result<u64> {
    Ok(u64::from_le_bytes(trial.read_bytes(8)?.try_into().unwrap()))
}

/// 以 `max_bytes` 设循环长度上限（读取器自身限额，非原生限制）；负值位模式
/// 一并拒绝。
fn read_loop_count(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    label: &str,
    entry_offset: usize,
) -> io::Result<u32> {
    let raw = read_i32_le(trial)?;
    if raw < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{label} 对象体 offset 0x{entry_offset:x}: 循环长度 {raw} 为负值位模式"),
        ));
    }
    let count = raw as u32;
    if count as usize > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{label} 对象体 offset 0x{entry_offset:x}: 循环长度 {count} 超过读取器 \
                 max_bytes 限额 {max_bytes}（读取器自身限额，非原生限制）"
            ),
        ));
    }
    Ok(count)
}

/// RandomColorRamp 体（Load `0x6eb3c190`；字段序见模块文档）。
#[derive(Debug, Clone, PartialEq)]
pub struct RandomColorRampBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// String → this−0x30（名称候选）。
    pub name: String,
    /// 流内请求 Size；Load 读入局部变量后将对应的对象槽清零，原值在此保留。
    pub raw_int32: i32,
    /// 2 字节 short（整体与零比较后写入 bool this−0x28；原样保留）。
    pub bool_short: u16,
    /// int32 → this−0x2c。
    pub seed_int32: i32,
    /// short ×6 → this−0x26(=−0x24)/−0x22/−0x20/−0x1e/−0x1c/−0x1a。
    pub shorts: [u16; 6],
}

impl RandomColorRampBody10_1 {
    /// 原生 Load 对该标志作整字非零判断；标志的业务含义尚未确定。
    pub fn loaded_flag(&self) -> bool {
        self.bool_short != 0
    }
}

/// 从对象头之后的位置读取 10.1 RandomColorRamp 体，失败时不推进游标、不修改
/// 登记表。
pub fn read_random_color_ramp_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RandomColorRampBody10_1> {
    let _ = table; // 体无嵌套对象条目，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_RANDOM_VERSION,
        "RandomColorRamp",
        entry_offset,
    )?;
    let name = read_bstr_string_10_1(&mut trial, max_bytes)?;
    let raw_int32 = read_i32_le(&mut trial)?;
    let bool_short = trial.read_u16_le()?;
    let seed_int32 = read_i32_le(&mut trial)?;
    let mut shorts = [0u16; 6];
    for slot in shorts.iter_mut() {
        *slot = trial.read_u16_le()?;
    }
    *reader = trial;
    Ok(RandomColorRampBody10_1 {
        key,
        version,
        name,
        raw_int32,
        bool_short,
        seed_int32,
        shorts,
    })
}

/// MultiPartColorRamp 体（Load `0x6eb395b0`；子梯度槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPartColorRampBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝。
    pub version: u16,
    /// String → this−0x44（名称候选）。
    pub name: String,
    /// int32 循环长度（子梯度数候选）。
    pub ramp_count: u32,
    /// count × IColorRamp 子梯度槽（经 [`read_color_ramp_slot_10_1`] 分派；
    /// 嵌套停止经各槽 `stop` 上抛）。
    pub ramps: Vec<ColorRampSlot10_1>,
    /// 版本 ≥2：每个子渐变的权重，按 double 位模式原样保留。
    pub tail_doubles: Option<Vec<u64>>,
    /// 嵌套停止（子梯度槽出现未闭合实现类时非 None；游标停在内层停止点）。
    pub stop: Option<SymbolStop10_1>,
}

impl MultiPartColorRampBody10_1 {
    /// 成功读取完整体后，返回指定子渐变的原生初始权重位模式。
    /// 旧版本无权重数组，原生为每个子渐变初始化 1.0。
    pub fn loaded_weight_bits(&self, index: usize) -> Option<u64> {
        if self.stop.is_some() || index >= self.ramps.len() {
            return None;
        }
        match &self.tail_doubles {
            Some(values) => values.get(index).copied(),
            None if self.version < 2 => Some(1.0_f64.to_bits()),
            None => None,
        }
    }
}

/// 从对象头之后的位置读取 10.1 MultiPartColorRamp 体。
///
/// 嵌套停止与游标：子梯度槽未知实现类由 [`read_color_ramp_slot_10_1`] 回退并
/// 报停止，本读取器立即停止消费——游标停在该子梯度槽起点（同符号族体读取器
/// 的嵌套停止语义），该槽及其后的 double 尾段留给调用者。体自身失败不推进
/// 游标、不修改登记表。
pub fn read_multi_part_color_ramp_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MultiPartColorRampBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_MULTI_PART_VERSION,
        "MultiPartColorRamp",
        entry_offset,
    )?;
    let name = read_bstr_string_10_1(&mut trial, max_bytes)?;
    let ramp_count = read_loop_count(&mut trial, max_bytes, "MultiPartColorRamp", entry_offset)?;
    let mut ramps = Vec::with_capacity((ramp_count as usize).min(4096));
    let mut stop = None;
    for _ in 0..ramp_count {
        let (slot, slot_stop) = read_color_ramp_slot_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            depth,
            entry_offset,
        )?;
        match slot {
            Some(slot) => {
                // 子槽成功但携带嵌套停止（如子 MultiPart 内层停止）时，游标停
                // 在内层停止点，其余子槽与 double 尾段一并停止消费。
                ramps.push(slot);
                if slot_stop.is_some() {
                    stop = slot_stop;
                    break;
                }
            }
            None => {
                stop = slot_stop;
                break;
            }
        }
    }
    let mut tail_doubles = None;
    if stop.is_none() && version >= 2 {
        let mut values = Vec::with_capacity((ramp_count as usize).min(4096));
        for _ in 0..ramp_count {
            values.push(read_u64_le(&mut trial)?);
        }
        tail_doubles = Some(values);
    }
    let result = MultiPartColorRampBody10_1 {
        key,
        version,
        name,
        ramp_count,
        ramps,
        tail_doubles,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// PresetColorRamp 体（Load `0x6eb3ae20`；颜色槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct PresetColorRampBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// String（名称候选）。
    pub name: String,
    /// int32。
    pub raw_int32: i32,
    /// int32 循环长度（预设颜色数候选）。
    pub color_count: u32,
    /// count × IColor 槽（五类闭合颜色分派）。
    pub colors: Vec<ColorSlot10_1>,
    /// 嵌套停止（颜色槽未知实现类时非 None）。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 PresetColorRamp 体。
pub fn read_preset_color_ramp_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<PresetColorRampBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_PRESET_VERSION,
        "PresetColorRamp",
        entry_offset,
    )?;
    let name = read_bstr_string_10_1(&mut trial, max_bytes)?;
    let raw_int32 = read_i32_le(&mut trial)?;
    let color_count = read_loop_count(&mut trial, max_bytes, "PresetColorRamp", entry_offset)?;
    let mut colors = Vec::with_capacity((color_count as usize).min(4096));
    let mut stop = None;
    for _ in 0..color_count {
        match read_color_slot_10_1(&mut trial, max_bytes, &mut trial_table)? {
            ColorSlot10_1::Unsupported { class_id, key } => {
                stop = Some(SymbolStop10_1::Slot {
                    kind: crate::objects::display_symbol::DisplaySlotKind10_1::Color,
                    class_id,
                    key,
                });
                break;
            }
            slot => colors.push(slot),
        }
    }
    let result = PresetColorRampBody10_1 {
        key,
        version,
        name,
        raw_int32,
        color_count,
        colors,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// AlgorithmicColorRamp 体（Load `0x6eb316c0`；颜色槽停止经 `stop` 传播）。
#[derive(Debug, Clone, PartialEq)]
pub struct AlgorithmicColorRampBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝。
    pub version: u16,
    /// String → this−0x20（名称候选）。
    pub name: String,
    /// int32 → this−0x1c。
    pub raw_int32: i32,
    /// 起止颜色槽 ×2（IColor；Save 槽 this−0x28 / this−0x24）。
    pub colors: [Option<ColorSlot10_1>; 2],
    /// 嵌套停止（颜色槽未知实现类时非 None；游标停在该槽起点）。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 AlgorithmicColorRamp 体。
///
/// 停止语义与 MultiPart 一致：颜色槽未知类回退并上抛停止后，本读取器不再
/// 消费后续槽（第二槽与尾段一并留给调用者，游标停在未知槽起点）。
pub fn read_algorithmic_color_ramp_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<AlgorithmicColorRampBody10_1> {
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_ALGORITHMIC_VERSION,
        "AlgorithmicColorRamp",
        entry_offset,
    )?;
    let name = read_bstr_string_10_1(&mut trial, max_bytes)?;
    let raw_int32 = read_i32_le(&mut trial)?;
    let mut colors = [None, None];
    let mut stop = None;
    for slot in colors.iter_mut() {
        match read_color_slot_10_1(&mut trial, max_bytes, &mut trial_table)? {
            ColorSlot10_1::Unsupported { class_id, key } => {
                stop = Some(SymbolStop10_1::Slot {
                    kind: crate::objects::display_symbol::DisplaySlotKind10_1::Color,
                    class_id,
                    key,
                });
                break;
            }
            ok => *slot = Some(ok),
        }
    }
    let result = AlgorithmicColorRampBody10_1 {
        key,
        version,
        name,
        raw_int32,
        colors,
        stop,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// IColorRamp 类槽位条目（GradientFillSymbol 主槽与 MultiPart 子梯度槽共用）。
#[derive(Debug, Clone, PartialEq)]
pub enum ColorRampSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 RandomColorRamp 对象（体闭合；`0x6eb3c190`）。
    Random(Box<RandomColorRampBody10_1>),
    /// 新 MultiPartColorRamp 对象（体闭合；`0x6eb395b0`；嵌套停止经其 `stop`
    /// 传播为嵌套未完整）。
    MultiPart(Box<MultiPartColorRampBody10_1>),
    /// 新 PresetColorRamp 对象（体闭合；`0x6eb3ae20`）。
    Preset(Box<PresetColorRampBody10_1>),
    /// 新 AlgorithmicColorRamp 对象（体闭合；`0x6eb316c0`）。
    Algorithmic(Box<AlgorithmicColorRampBody10_1>),
}

/// 读取一个 IColorRamp 类槽位条目，返回 `(槽位, 停止)`。
///
/// 空标记与回引可消费；新对象按类分派四闭合类（`depth` 供 MultiPart 子梯度
/// 递归限额），其余实现类回退到槽位起点、登记表不保留该键，以停止原因返回。
/// `entry_offset` 仅用于错误消息定位（传体起点即可）。
pub fn read_color_ramp_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
    entry_offset: usize,
) -> io::Result<(Option<ColorRampSlot10_1>, Option<SymbolStop10_1>)> {
    const MAX_COLOR_RAMP_NESTING_DEPTH: usize = 16;
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(ColorRampSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(ColorRampSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "颜色梯度槽位 offset 0x{entry_offset:x}: 嵌套深度超过读取器限额 \
                         {MAX_COLOR_RAMP_NESTING_DEPTH}（防栈溢出，非原生限制）"
                    ),
                ));
            }
            let slot = read_color_ramp_body_for_class_10_1(
                &mut forward,
                max_bytes,
                staged,
                depth,
                class_id,
                key,
            )?;
            match slot {
                Some(slot) => {
                    let stop = nested_ramp_stop(&slot);
                    *trial = forward;
                    Ok((Some(slot), stop))
                }
                None => {
                    *trial = origin;
                    *staged = saved;
                    Ok((
                        None,
                        Some(SymbolStop10_1::Slot {
                            kind: DisplaySlotKind10_1::ColorRamp,
                            class_id,
                            key,
                        }),
                    ))
                }
            }
        }
    }
}

/// 已消费对象类标识时的颜色梯度体分派。调用者负责事务和层数检查。
pub(crate) fn read_color_ramp_body_for_class_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<Option<ColorRampSlot10_1>> {
    Ok(if class_id == RANDOM_COLOR_RAMP_CLASS_ID {
        let body = read_random_color_ramp_body_10_1(reader, max_bytes, table, key)?;
        Some(ColorRampSlot10_1::Random(Box::new(body)))
    } else if class_id == MULTI_PART_COLOR_RAMP_CLASS_ID {
        let body = read_multi_part_color_ramp_body_10_1(
            reader,
            max_bytes,
            table,
            key,
            depth.saturating_sub(1),
        )?;
        Some(ColorRampSlot10_1::MultiPart(Box::new(body)))
    } else if class_id == PRESET_COLOR_RAMP_CLASS_ID {
        let body = read_preset_color_ramp_body_10_1(reader, max_bytes, table, key)?;
        Some(ColorRampSlot10_1::Preset(Box::new(body)))
    } else if class_id == ALGORITHMIC_COLOR_RAMP_CLASS_ID {
        let body = read_algorithmic_color_ramp_body_10_1(reader, max_bytes, table, key)?;
        Some(ColorRampSlot10_1::Algorithmic(Box::new(body)))
    } else {
        None
    })
}

/// 归约 MultiPart 子梯度槽的嵌套停止为外层嵌套未完整（与装饰槽分派同型）。
pub(crate) fn nested_ramp_stop(slot: &ColorRampSlot10_1) -> Option<SymbolStop10_1> {
    match slot {
        ColorRampSlot10_1::MultiPart(body) => body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
            class_id: MULTI_PART_COLOR_RAMP_CLASS_ID,
            key: body.key,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::display_color::RGB_COLOR_CLASS_ID;

    /// 构造对象条目字节：16 字节类标识 + u32 键。
    fn entry(class_id: &[u8; 16], key: u32) -> Vec<u8> {
        let mut bytes = class_id.to_vec();
        bytes.extend(key.to_le_bytes());
        bytes
    }

    /// 构造 AfCore BSTR 字符串：u32 码元数，非空另有终止零。
    fn string_bytes(text: &str) -> Vec<u8> {
        let mut bytes = (text.encode_utf16().count() as u32).to_le_bytes().to_vec();
        for unit in text.encode_utf16() {
            bytes.extend(unit.to_le_bytes());
        }
        if !text.is_empty() {
            bytes.extend(0_u16.to_le_bytes());
        }
        bytes
    }

    /// 构造 RGB 颜色体（共享 Color 核心：版本 1 + 核心版本 1 + flag!=0 的
    /// 4 字节 packed RGB + 尾随两字节，与 display_color 轮记录同构）。
    fn rgb_body() -> Vec<u8> {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0x01, 0x11, 0x22, 0x33, 0x00]);
        bytes.extend([0xff, 0x00]);
        bytes
    }

    fn random_body_bytes() -> Vec<u8> {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(string_bytes("梯度"));
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend([0x01, 0x00]);
        bytes.extend(42_i32.to_le_bytes());
        for v in [10, 200, 20, 210, 30, 220] {
            bytes.extend((v as u16).to_le_bytes());
        }
        bytes
    }

    #[test]
    fn random_color_ramp_full_body_roundtrip() {
        let bytes = random_body_bytes();
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_random_color_ramp_body_10_1(&mut reader, 1024, &mut table, 9)
            .expect("RandomColorRamp 体应整体消费");
        assert_eq!(body.key, 9);
        assert_eq!(body.version, 1);
        assert_eq!(body.name, "梯度");
        assert_eq!(body.raw_int32, 7);
        assert_eq!(body.bool_short, 0x0001);
        assert_eq!(body.seed_int32, 42);
        assert_eq!(body.shorts, [10, 200, 20, 210, 30, 220]);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn random_color_ramp_flag_compares_the_full_word() {
        let mut bytes = random_body_bytes();
        let offset = 2 + string_bytes("梯度").len() + 4;
        for (raw, expected) in [(0_u16, false), (1, true), (0x0100, true), (0xffff, true)] {
            bytes[offset..offset + 2].copy_from_slice(&raw.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let body =
                read_random_color_ramp_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 9)
                    .unwrap();
            assert_eq!(body.bool_short, raw);
            assert_eq!(body.loaded_flag(), expected);
            assert_eq!(reader.remaining(), 0);
        }
        let mut reader = Reader::new(&bytes[..offset + 1]);
        assert!(
            read_random_color_ramp_body_10_1(&mut reader, 1024, &mut RefTable10_1::new(), 9,)
                .is_err()
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn random_color_ramp_rejects_version_and_negative_patterns() {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend([0u8; 32]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_random_color_ramp_body_10_1(&mut reader, 1024, &mut table, 1).is_err());
        // 负值位模式（较原生更严）。
        let bytes = 0x8000_u16.to_le_bytes().to_vec();
        let mut reader = Reader::new(&bytes);
        assert!(read_random_color_ramp_body_10_1(&mut reader, 1024, &mut table, 1).is_err());
    }

    #[test]
    fn random_color_ramp_truncation_is_transactional() {
        let full = random_body_bytes();
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let mut table = RefTable10_1::new();
            assert!(
                read_random_color_ramp_body_10_1(&mut reader, 1024, &mut table, 1).is_err(),
                "截断到 {cut} 字节应失败"
            );
            assert!(table.is_empty(), "失败不得修改登记表");
        }
    }

    #[test]
    fn multi_part_color_ramp_with_random_child_and_tail() {
        let mut bytes = 2_u16.to_le_bytes().to_vec(); // 版本 2（含 double 尾段）
        bytes.extend(string_bytes("mp"));
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(entry(&RANDOM_COLOR_RAMP_CLASS_ID, 5));
        bytes.extend(random_body_bytes());
        bytes.extend(2.5_f64.to_le_bytes()); // count=1 → 尾段 1 个 double
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_multi_part_color_ramp_body_10_1(&mut reader, 1024, &mut table, 3, 8)
            .expect("MultiPart 体应整体消费");
        assert_eq!(body.version, 2);
        assert_eq!(body.ramp_count, 1);
        assert_eq!(body.ramps.len(), 1);
        assert!(matches!(body.ramps[0], ColorRampSlot10_1::Random(_)));
        assert_eq!(body.tail_doubles, Some(vec![2.5_f64.to_bits()]));
        assert_eq!(body.loaded_weight_bits(0), Some(2.5_f64.to_bits()));
        assert_eq!(body.loaded_weight_bits(1), None);
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn multi_part_old_version_defaults_weights_without_consuming_tail() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(string_bytes(""));
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0_u8; 16]);
        bytes.extend([0_u8; 16]);
        bytes.push(0xa5);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_multi_part_color_ramp_body_10_1(&mut reader, 1024, &mut table, 3, 8)
            .expect("旧版渐变体应读取成功");
        assert_eq!(body.tail_doubles, None);
        assert_eq!(body.loaded_weight_bits(0), Some(1.0_f64.to_bits()));
        assert_eq!(body.loaded_weight_bits(1), Some(1.0_f64.to_bits()));
        assert_eq!(body.loaded_weight_bits(2), None);
        assert_eq!(reader.read_bytes(1).unwrap(), &[0xa5]);
    }

    #[test]
    fn multi_part_color_ramp_unknown_child_stops() {
        let unknown = [0x11_u8; 16];
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(string_bytes(""));
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(entry(&unknown, 6));
        bytes.extend([0xAA]); // 停止点之后的哨兵字节
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_multi_part_color_ramp_body_10_1(&mut reader, 1024, &mut table, 3, 8)
            .expect("嵌套停止属于成功返回（带停止原因）");
        assert!(body.ramps.is_empty());
        assert!(body.tail_doubles.is_none());
        assert_eq!(body.loaded_weight_bits(0), None);
        match body.stop.expect("应携带停止原因") {
            SymbolStop10_1::Slot { class_id, key, .. } => {
                assert_eq!(class_id, unknown);
                assert_eq!(key, 6);
            }
            other => panic!("停止原因应为槽位停止，实际 {other:?}"),
        }
        // 游标停在槽位对象头起点：20 字节条目 + 1 字节哨兵未消费，键未登记。
        assert_eq!(reader.remaining(), 21);
        assert!(table.is_empty());
    }

    #[test]
    fn multi_part_color_ramp_version_gate() {
        let mut bytes = 3_u16.to_le_bytes().to_vec();
        bytes.extend([0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_multi_part_color_ramp_body_10_1(&mut reader, 1024, &mut table, 1, 8).is_err());
    }

    fn preset_body_bytes() -> Vec<u8> {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(string_bytes("preset"));
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(entry(&RGB_COLOR_CLASS_ID, 7));
        bytes.extend(rgb_body());
        bytes.extend(entry(&RGB_COLOR_CLASS_ID, 8));
        bytes.extend(rgb_body());
        bytes
    }

    #[test]
    fn preset_color_ramp_with_two_colors() {
        let bytes = preset_body_bytes();
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_preset_color_ramp_body_10_1(&mut reader, 1024, &mut table, 2)
            .expect("Preset 体应整体消费");
        assert_eq!(body.color_count, 2);
        assert_eq!(body.colors.len(), 2);
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn preset_color_ramp_unknown_color_stops_at_slot_origin() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(string_bytes(""));
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        let unknown = [0x77_u8; 16];
        bytes.extend(entry(&unknown, 9));
        bytes.extend([0xBB]); // 哨兵
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_preset_color_ramp_body_10_1(&mut reader, 1024, &mut table, 2)
            .expect("嵌套停止属于成功返回");
        assert!(body.colors.is_empty());
        assert!(body.stop.is_some());
        assert_eq!(reader.remaining(), 21, "游标应停在未知颜色槽起点");
        assert!(table.is_empty());
    }

    #[test]
    fn algorithmic_color_ramp_two_color_slots() {
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(string_bytes("algo"));
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend(entry(&RGB_COLOR_CLASS_ID, 10));
        bytes.extend(rgb_body());
        bytes.extend(entry(&RGB_COLOR_CLASS_ID, 11));
        bytes.extend(rgb_body());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_algorithmic_color_ramp_body_10_1(&mut reader, 1024, &mut table, 4)
            .expect("Algorithmic 体应整体消费");
        assert!(body.colors.iter().all(|c| c.is_some()));
        assert!(body.stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn color_ramp_slot_dispatch_matrix() {
        // 空标记。
        let mut reader = Reader::new(&[0u8; 16]);
        let mut table = RefTable10_1::new();
        let (slot, stop) = read_color_ramp_slot_10_1(&mut reader, 1024, &mut table, 8, 0).unwrap();
        assert!(matches!(slot, Some(ColorRampSlot10_1::Null)) && stop.is_none());
        // 回引：先登记键。
        let mut table = RefTable10_1::new();
        table.register(31);
        let bytes = entry(&RANDOM_COLOR_RAMP_CLASS_ID, 31);
        let mut reader = Reader::new(&bytes);
        let (slot, stop) = read_color_ramp_slot_10_1(&mut reader, 1024, &mut table, 8, 0).unwrap();
        assert!(
            matches!(slot, Some(ColorRampSlot10_1::BackReference { key: 31, .. }))
                && stop.is_none()
        );
        assert_eq!(reader.remaining(), 0);
        // 未知实现类：停止回退，键不登记。
        let unknown = [0x99_u8; 16];
        let bytes = entry(&unknown, 32);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let (slot, stop) = read_color_ramp_slot_10_1(&mut reader, 1024, &mut table, 8, 0).unwrap();
        assert!(slot.is_none());
        match stop.expect("未知类应停止") {
            SymbolStop10_1::Slot {
                kind,
                class_id,
                key,
            } => {
                assert_eq!(kind, DisplaySlotKind10_1::ColorRamp);
                assert_eq!(class_id, unknown);
                assert_eq!(key, 32);
            }
            other => panic!("停止原因形态错误 {other:?}"),
        }
        assert_eq!(reader.remaining(), bytes.len(), "游标应回退到槽位起点");
        assert!(table.is_empty(), "停止路径键不登记");
        // 嵌套深度限额。
        let bytes = entry(&RANDOM_COLOR_RAMP_CLASS_ID, 33);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        assert!(read_color_ramp_slot_10_1(&mut reader, 1024, &mut table, 0, 0).is_err());
    }

    #[test]
    fn class_id_constants_match_guid_strings() {
        // {BEB87094-C0B4-11D0-8379-080009B996CC} 的 LE 内存序。
        let expect_random: [u8; 16] = [
            0x94, 0x70, 0xb8, 0xbe, 0xb4, 0xc0, 0xd0, 0x11, 0x83, 0x79, 0x08, 0x00, 0x09, 0xb9,
            0x96, 0xcc,
        ];
        assert_eq!(RANDOM_COLOR_RAMP_CLASS_ID, expect_random);
        // 四类 CLSID 仅首双字不同（同族后缀）。
        assert_eq!(
            &MULTI_PART_COLOR_RAMP_CLASS_ID[4..],
            &RANDOM_COLOR_RAMP_CLASS_ID[4..]
        );
        assert_eq!(
            &PRESET_COLOR_RAMP_CLASS_ID[4..],
            &RANDOM_COLOR_RAMP_CLASS_ID[4..]
        );
        assert_eq!(
            &ALGORITHMIC_COLOR_RAMP_CLASS_ID[4..],
            &RANDOM_COLOR_RAMP_CLASS_ID[4..]
        );
        assert_eq!(MULTI_PART_COLOR_RAMP_CLASS_ID[0], 0x99);
        assert_eq!(PRESET_COLOR_RAMP_CLASS_ID[0], 0x9a);
        assert_eq!(ALGORITHMIC_COLOR_RAMP_CLASS_ID[0], 0x9b);
    }
}
