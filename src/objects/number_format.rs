//! ArcMap 10.1 INumberFormat 族剩余三个变体（PercentageFormat /
//! FractionFormat / RateFormat，SystemCore.dll，NumericFormat 同族）对象体的
//! 严格读取子集；本模块同时提供 NumericFormat 核心体的 objects 层副本（见
//! 文末「与 feature_layer_tail 的重复」）。
//!
//! 证据来源：`docs/arcmap-10.1-number-format.md`（本轮研究记录）。原生实现
//! 均为 **SystemCore.dll 10.1.0.3035**（SHA-256 `f63cf949…`，ImageBase
//! `0x7ffb0000`，pefile FileVersion/ProductVersion 10.1.0.3035）；类身份三
//! 方互证：`esri.clsid.ecfg` 声明（模块 SystemCore.dll）、GetClassID 立即数
//! 写出（见各常量文档）、IPersistStream 虚表槽位（见下表）；esriSystem 类
//! 型库 coclass PercentageFormat/FractionFormat/RateFormat（typekind=5）与
//! 主接口 IPercentageFormat `{7E4F4711}`/IFractionFormat `{7E4F4713}`/
//! IRateFormat `{7E4F4717}`（接口身份为类型库文件内静态数据）。样本
//! `samples/测试底图.mxd` 的 /Maps 流内 **0 实例**（INumberFormat 槽仅出现
//! NumericFormat），三个变体无样本字节佐证，字段序全部来自反编译。
//!
//! ## 定位链（RTTI TD→COL→虚表→GetClassID，10.1 静态映像 VA；判据
//! [C]==0 且 [C+0xc]==TD）
//!
//! | 变体 | RTTI TD | 首 COL（IPersistStream 虚表前置） | 虚表 | GetClassID | Load | Save |
//! | --- | --- | --- | --- | --- | --- | --- |
//! | NumericFormat（既有锚） | `0x800a5984` | `0x80082d74` | `0x80070ec4` | `0x7ffce470` | `0x7ffce4b0` | `0x7ffce590` |
//! | PercentageFormat | `0x800a6878` | `0x80083280` | `0x80071404` | `0x7ffd1170` | `0x7ffd11b0` | `0x7ffd1230` |
//! | FractionFormat | `0x800a6398` | `0x80081cc0` | `0x8006fc38` | `0x7ffc6120` | `0x7ffc68a0` | `0x7ffc6920` |
//! | RateFormat | `0x800a6a18` | `0x80083a88` | `0x80071d94` | `0x7ffd65f0` | `0x7ffd6a10` | `0x7ffd6a90` |
//!
//! 三变体 GetClassID 分别以立即数对 `0x11D28E547E4F471B`/`0xD8AA`、
//! `0x…471C`/`0xD8AA`、`0x…4721`/`0xD8AA` 写出 CLSID（与 esri.clsid 声明互
//! 证，证据：确定的）；虚表 +0x14/+0x18 槽经 this 调整器（`mov ecx,[esp+4];
//! sub ecx,N; mov [esp+4],ecx; jmp 实体`）指向 Load/Save 实体，锚
//! NumericFormat 的同名链逐槽复核与既有记录一致。
//!
//! ## 字段序（Load 反编译；Save 对称互证）
//!
//! **PercentageFormat::Load `0x7ffd11b0`**（版本门 `2 < (short)ver` 有符号
//! 拒绝 `0x80004005`）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>2 拒绝） | — |
//! | ② | short | 恒有 | this-0xc |
//! | ③ | NumericFormat::Load(`0x7ffce4b0`) 内嵌核心（short 版本 + int32 ×4 + short ×3） | 版本 ≥2 | this-0xc（同槽升级） |
//!
//! **FractionFormat::Load `0x7ffc68a0`**（`1 < (short)ver` 拒绝）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>1 拒绝） | — |
//! | ② | int32 | 恒有 | this-0x10 |
//! | ③ | int32 | 恒有 | this-0xc |
//!
//! **RateFormat::Load `0x7ffd6a10`**（`2 < (short)ver` 拒绝）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>2 拒绝） | — |
//! | ② | double | 恒有 | this-0x14 |
//! | ③ | AfCore String::Read | 恒有 | this-0xc |
//! | ④ | NumericFormat::Load 内嵌核心 | 版本 >1（≥2） | this-0x14（同槽升级，覆盖 ② 的 double） |
//!
//! Save 侧（写方向互证，均写**静态版本 short**，无 AfStreamGetSaveVersion
//! 表）：PercentageFormat Save `0x7ffd1230` 写版本常量 `DAT_8006d0f0`=**2**
//! + ② + NumericFormat::Save；FractionFormat Save `0x7ffc6920` 写
//! `DAT_8006d10c`=**1** + ②③；RateFormat Save `0x7ffd6a90` 写
//! `DAT_8006d0e0`=**2** + ②③④（版本常量字节均已读出核对）。写侧恒写满
//! 形态（Percentage/Rate 恒含内嵌核心），与读侧版本门一致。
//!
//! 各标量/字符串的业务语义未定性（对齐方式、小数位数、分母样式、单位比率
//! 描述、链接目标等均为按类名与 NumericFormat 既有样本形态的候选）。
//!
//! ## 与 feature_layer_tail 的重复
//!
//! [`NUMERIC_FORMAT_CLASS_ID`] 与 [`read_numeric_format_body_10_1`]/
//! [`NumericFormatBody10_1`] 在 `src/layers/feature_layer_tail.rs` 已有语义
//! 相同的副本（第八轮落位，本模块不反向依赖 layers 层）。两处必须保持同
//! 步；**迁移建议（写给整合轮）**：删除 feature_layer_tail 的副本、改从本
//! 模块 `pub use` 重导出（或直接调用），`FieldInfoBody10_1` 的
//! INumberFormat 槽分派同步扩为 PercentageFormat/FractionFormat/RateFormat
//! 三类（本模块读取器可直接作为槽位分派目标，槽位新对象解析成功即整体消
//! 费、失败回退语义不变）。
//!
//! 版本门均为原生有符号 short 比较；负值位模式本读取器一并拒绝（负值版本
//! 无业务意义，较原生更严）。这是仅覆盖上述分支的严格安全子集：要求完整
//! 输入，任何失败不推进调用者游标、不修改登记表；未接入 MXD 解析主链，
//! 人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见证据文档）：三变体全部标量/String 字段的业务语义未定
//! 性；Percentage/Rate 内嵌核心与外壳的槽位重叠（版本 ≥2 同槽升级）仅由
//! 反编译证据支撑，无样本字节；版本 ≤门限（Percentage 0..1、Fraction 0、
//! Rate 0..1）的短体形态无样本；全部结论未运行 ArcMap 验证。

use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// NumericFormat::Load 接受的最大版本（有符号比较 `1 < ver` 拒绝；与
/// feature_layer_tail 的既有常量同值）。证据：SystemCore.dll `0x7ffce4b0`。
const MAX_NUMERIC_FORMAT_VERSION: i16 = 1;

/// PercentageFormat::Load 接受的最大版本：反编译 `2 < (short)ver` 有符号拒
/// 绝；写侧 Save 版本常量 `DAT_8006d0f0` 恒 2。
const MAX_PERCENTAGE_VERSION: i16 = 2;

/// FractionFormat::Load 接受的最大版本：反编译 `1 < (short)ver` 拒绝；写侧
/// Save 版本常量 `DAT_8006d10c` 恒 1。
const MAX_FRACTION_VERSION: i16 = 1;

/// RateFormat::Load 接受的最大版本：反编译 `2 < (short)ver` 有符号拒绝；写
/// 侧 Save 版本常量 `DAT_8006d0e0` 恒 2。
const MAX_RATE_VERSION: i16 = 2;

/// NumericFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F4719-8E54-11D2-AAD8-000000000000}`。证据与
/// `feature_layer_tail::NUMERIC_FORMAT_CLASS_ID` 相同（esri.clsid 声明 +
/// GetClassID `0x7ffce470` 立即数写出 + 样本对象头），此处为 objects 层副
/// 本（迁移建议见模块文档）。
pub const NUMERIC_FORMAT_CLASS_ID: [u8; 16] = [
    0x19, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// PercentageFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F471B-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F471B}` progID `PercentageFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass #214 PercentageFormat；
/// SystemCore.dll GetClassID `0x7ffd1170` 以立即数对
/// `0x11D28E547E4F471B`/`0x000000000000D8AA` 写出（IPersistStream 虚表
/// `0x80071404` 槽 +0x0c 经调整器 `0x7ffd12c0` 指向）。样本 0 实例。
pub const PERCENTAGE_FORMAT_CLASS_ID: [u8; 16] = [
    0x1b, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// FractionFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F471C-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F471C}` progID `FractionFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass #215 FractionFormat；
/// SystemCore.dll GetClassID `0x7ffc6120` 以立即数对
/// `0x11D28E547E4F471C`/`0x000000000000D8AA` 写出（IPersistStream 虚表
/// `0x8006fc38` 槽 +0x0c 经调整器 `0x7ffc6a90` 指向）。样本 0 实例。
pub const FRACTION_FORMAT_CLASS_ID: [u8; 16] = [
    0x1c, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// RateFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F4721-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F4721}` progID `RateFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass #220 RateFormat；
/// SystemCore.dll GetClassID `0x7ffd65f0` 以立即数对
/// `0x11D28E547E4F4721`/`0x000000000000D8AA` 写出（IPersistStream 虚表
/// `0x80071d94` 槽 +0x0c 经调整器 `0x7ffd6b90` 指向）。样本 0 实例。
pub const RATE_FORMAT_CLASS_ID: [u8; 16] = [
    0x21, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// 读取 int16 并按原生有符号比较校验版本上限；负值位模式一并拒绝（较原生
/// 更严，与 `feature_layer_tail::read_version_short` 同语义）。
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
                 {what}::Load 接受范围（有符号比较 >{max} 拒绝；负值位模式一并拒绝，较原生更严）"
            ),
        ));
    }
    Ok(version)
}

/// NumericFormat（SystemCore.dll `0x7ffce4b0`）核心体的已确认字段。
///
/// 体长恒 24 字节（short 版本 + int32 ×4 + short ×3），无嵌套对象条目、无
/// 字符串，无停止点：要么完整消费，要么失败（截断/版本超限）。各
/// int32/short 语义未定（样本形态 0/0/0/12 与 0/6/0/12 + 0/0/0）。
///
/// 本结构与 `feature_layer_tail::NumericFormatBody10_1` 字段完全一致（迁移
/// 建议见模块文档）；**注意**：作为 Percentage/Rate 的内嵌核心复用时，流
/// 内没有独立对象头与键，`key` 传外层对象的键（同一流内对象）。
#[derive(Debug, Clone, PartialEq)]
pub struct NumericFormatBody10_1 {
    /// 对象头键（样本 42/53/55/57；内嵌核心复用时与外壳同键）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表（由槽位分派完成）。
    pub key: u32,
    /// 体首 short 版本（样本 1）；>1 原生拒绝。
    pub version: u16,
    /// int32 ×4 → obj-0x1c/-0x18/-0x14/-0x10，语义未定，原样保留。
    pub int32s: [i32; 4],
    /// short ×3 → obj-0xc/-0xa/-0x8，语义未定，原样保留。
    pub shorts: [i16; 3],
}

/// 从对象头之后的位置读取 10.1 NumericFormat 核心体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头（或作为 Percentage/Rate 版本 ≥2 的内嵌核
/// 心从外壳字段之后调用）。证据：SystemCore.dll 10.1.0.3035 Load
/// `0x7ffce4b0`（版本门 `1 < (short)ver` 有符号拒绝）与 Save `0x7ffce590`
/// （对称）。`table` 仅在签名上保持一致（无嵌套对象条目）。
pub fn read_numeric_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<NumericFormatBody10_1> {
    let _ = table;
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let version = read_version_short(
        &mut trial,
        MAX_NUMERIC_FORMAT_VERSION,
        "NumericFormat",
        entry_offset,
    )?;
    let mut int32s = [0_i32; 4];
    for slot in int32s.iter_mut() {
        *slot = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    }
    let mut shorts = [0_i16; 3];
    for slot in shorts.iter_mut() {
        *slot = trial.read_u16_le()? as i16;
    }
    let result = NumericFormatBody10_1 {
        key,
        version,
        int32s,
        shorts,
    };
    *reader = trial;
    Ok(result)
}

/// PercentageFormat（SystemCore.dll `0x7ffd11b0`）体的已确认字段。
///
/// 字段序：① short 版本（`2 < (short)ver` 有符号拒绝；写侧恒 2）→
/// ② short → obj-0xc → ③ 版本 ≥2：NumericFormat 核心体（内嵌同槽，无独立
/// 对象头）。版本 0..1 的短体仅 4 字节（②即终结），无样本佐证（仅反编译）。
#[derive(Debug, Clone, PartialEq)]
pub struct PercentageFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝，写侧恒 2。
    pub version: u16,
    /// short → obj-0xc（恒有），语义未定，原样保留。
    pub short_0c: i16,
    /// 版本 ≥2 的内嵌 NumericFormat 核心体（同槽升级，`None` = 版本 ≤1）。
    pub numeric: Option<Box<NumericFormatBody10_1>>,
}

/// 从对象头之后的位置读取 10.1 PercentageFormat 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`PERCENTAGE_FORMAT_CLASS_ID`] 并取
/// 得键 `key`（内嵌核心与外壳同键）。证据：SystemCore.dll 10.1.0.3035 Load
/// `0x7ffd11b0` 与 Save `0x7ffd1230`（版本常量 2，对称）。
pub fn read_percentage_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<PercentageFormatBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >2 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(
        &mut trial,
        MAX_PERCENTAGE_VERSION,
        "PercentageFormat",
        entry_offset,
    )?;
    // ② short → obj-0xc。
    let short_0c = trial.read_u16_le()? as i16;
    // ③ 版本 ≥2：内嵌 NumericFormat 核心（同槽升级；无独立对象头，同键）。
    let numeric = if (version as i16) > 1 {
        Some(Box::new(read_numeric_format_body_10_1(
            &mut trial, table, key,
        )?))
    } else {
        None
    };
    let result = PercentageFormatBody10_1 {
        key,
        version,
        short_0c,
        numeric,
    };
    *reader = trial;
    Ok(result)
}

/// FractionFormat（SystemCore.dll `0x7ffc68a0`）体的已确认字段。
///
/// 体长恒 10 字节（short 版本 + int32 ×2），无嵌套对象、无字符串、无版本
/// 分支字段；写侧 Save 版本常量恒 1。各 int32 语义未定。
#[derive(Debug, Clone, PartialEq)]
pub struct FractionFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝，写侧恒 1。
    pub version: u16,
    /// int32 → obj-0x10（恒有），语义未定，原样保留。
    pub int32_10: i32,
    /// int32 → obj-0xc（恒有），语义未定，原样保留。
    pub int32_0c: i32,
}

/// 从对象头之后的位置读取 10.1 FractionFormat 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`FRACTION_FORMAT_CLASS_ID`] 并取得
/// 键 `key`。证据：SystemCore.dll 10.1.0.3035 Load `0x7ffc68a0` 与 Save
/// `0x7ffc6920`（版本常量 1，对称）。
pub fn read_fraction_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FractionFormatBody10_1> {
    let _ = table; // 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >1 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(
        &mut trial,
        MAX_FRACTION_VERSION,
        "FractionFormat",
        entry_offset,
    )?;
    // ② int32 → obj-0x10；③ int32 → obj-0xc。
    let int32_10 = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let int32_0c = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let result = FractionFormatBody10_1 {
        key,
        version,
        int32_10,
        int32_0c,
    };
    *reader = trial;
    Ok(result)
}

/// RateFormat（SystemCore.dll `0x7ffd6a10`）体的已确认字段。
///
/// 字段序：① short 版本（`2 < (short)ver` 有符号拒绝；写侧恒 2）→
/// ② double → obj-0x14 → ③ AfCore String → obj-0xc → ④ 版本 >1：Numeric
/// Format 核心体（内嵌 obj-0x14 同槽升级，覆盖 ②；无独立对象头）。版本
/// 0..1 的短体止于 ③，无样本佐证（仅反编译）。
#[derive(Debug, Clone, PartialEq)]
pub struct RateFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝，写侧恒 2。
    pub version: u16,
    /// double → obj-0x14（恒有），语义未定，原样保留。
    pub double_14: f64,
    /// AfCore String → obj-0xc（恒有），语义未定（单位/比率描述候选）。
    pub string_0c: String,
    /// 版本 >1 的内嵌 NumericFormat 核心体（同槽升级，`None` = 版本 ≤1）。
    pub numeric: Option<Box<NumericFormatBody10_1>>,
}

/// 从对象头之后的位置读取 10.1 RateFormat 对象体，失败时不推进游标、不修
/// 改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`RATE_FORMAT_CLASS_ID`] 并取得键
/// `key`。`max_bytes` 是资源限额，应用于 ③ 的声明字符串长度（含终止零、
/// 不含 4 字节前缀），在任何读取与分配之前检查；`usize::MAX` 表示不额外设
/// 限。证据：SystemCore.dll 10.1.0.3035 Load `0x7ffd6a10` 与 Save
/// `0x7ffd6a90`（版本常量 2，对称）。
pub fn read_rate_format_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RateFormatBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >2 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(&mut trial, MAX_RATE_VERSION, "RateFormat", entry_offset)?;
    // ② double → obj-0x14。
    let double_14 = trial.read_f64_le()?;
    // ③ AfCore String → obj-0xc。
    let string_0c = read_string_10_1(&mut trial, max_bytes)?;
    // ④ 版本 >1：内嵌 NumericFormat 核心（obj-0x14 同槽升级；同键）。
    let numeric = if (version as i16) > 1 {
        Some(Box::new(read_numeric_format_body_10_1(
            &mut trial, table, key,
        )?))
    } else {
        None
    };
    let result = RateFormatBody10_1 {
        key,
        version,
        double_14,
        string_0c,
        numeric,
    };
    *reader = trial;
    Ok(result)
}
