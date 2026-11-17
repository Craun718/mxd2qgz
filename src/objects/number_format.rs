//! ArcMap 10.1 INumberFormat 族全部九类（NumericFormat 锚 + PercentageFormat
//! / FractionFormat / RateFormat / CurrencyFormat / LatLonFormat / AngleFormat
//! / ScientificFormat / CustomNumberFormat，SystemCore.dll）对象体的严格读取
//! 子集；本模块同时提供 NumericFormat 核心体的 objects 层副本（见文末「与
//! feature_layer_tail 的重复」）。
//!
//! 证据来源：`docs/arcmap-10.1-number-format.md`（两轮研究记录；本轮扩展见
//! 该文末「同族五类闭合」节）。原生实现均为 **SystemCore.dll 10.1.0.3035**
//! （SHA-256 `f63cf949…`，ImageBase `0x7ffb0000`，pefile
//! FileVersion/ProductVersion 10.1.0.3035）；类身份三方互证：
//! `esri.clsid.ecfg` 声明（模块 SystemCore.dll，progIDPrefix esriSystem）、
//! GetClassID 立即数或 .rdata 静态字节写出（见各常量文档）、IPersistStream
//! 虚表槽位（见下表）；esriSystem 类型库 coclass 与主接口身份为类型库文件
//! 内静态数据。样本 `samples/测试底图.mxd` 的 /Maps 流内后五类 **0 实例**
//!（INumberFormat 槽仅出现 NumericFormat），五类无样本字节佐证，字段序全
//! 部来自反编译。
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
//! | CurrencyFormat | `0x800a5f40` | `0x800808c4` | `0x8006e294` | `0x7ffbb850` | `0x7ffbbbc0` | `0x7ffbbc00` |
//! | LatLonFormat | `0x800a64a0` | `0x80082114` | `0x80070250` | `0x7ffca1a0` | `0x7ffca1e0` | `0x7ffca2c0` |
//! | AngleFormat | `0x800a59a0` | `0x8007edf8` | `0x8006d37c` | `0x7ffb3360` | `0x7ffb33a0` | `0x7ffb3430` |
//! | ScientificFormat | `0x800a6aa8` | `0x80083e34` | `0x80072540` | `0x7ffda3f0` | `0x7ffda430` | `0x7ffda4a0` |
//! | CustomNumberFormat | `0x800a5f60` | `0x800809ec` | `0x8006e3cc` | `0x7ffbbdd0` | `0x7ffbc4d0` | `0x7ffbc520` |
//!
//! 各变体 GetClassID 分别以立即数对（前三变体）或 .rdata 静态 16 字节复制
//!（后五类）写出 CLSID（与 esri.clsid 声明互证，证据：确定的）；虚表
//! +0x14/+0x18 槽经 this 调整器（`mov ecx,[esp+4]; sub ecx,N; mov
//! [esp+4],ecx; jmp 实体`，后五类为 `sub ecx,[ecx-4]` 虚位移形态）指向
//! Load/Save 实体，锚 NumericFormat 与 PercentageFormat 的同名链逐槽复核
//! 与既有记录一致。
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
//! | ③ | NumericFormat::Load(`0x7ffce4b0`) 内嵌核心（short 版本 + int32 ×4 + short ×3） | 版本 ≥2 | 传入 this-0xc（核心字段在该指针之前） |
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
//! | ④ | NumericFormat::Load 内嵌核心 | 版本 >1（≥2） | 传入 this-0x14（核心字段在该指针之前） |
//!
//! **CurrencyFormat::Load `0x7ffbbbc0`**（`(short)ver < 2` 方可通过）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>1 拒绝） | — |
//!
//! 体止于版本 short（2 字节），无任何其他字段。
//!
//! **LatLonFormat::Load `0x7ffca1e0`**（`4 < (short)ver` 拒绝）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>4 拒绝） | — |
//! | ② | short | 恒有 | this-0x2c |
//! | ③ | short | 恒有 | this-0x2a |
//! | ④ | short + short | 版本 >1（≥2） | this-0xc / this-0xa（≤1 时原生置内存默认 -1/-1，流内无字段） |
//! | ⑤ | NumericFormat::Load 内嵌核心 | 版本 >2（≥3） | 传入 this-0x38（返回值受检） |
//! | ⑥ | short | 版本 >3（≥4） | this-0x8 |
//!
//! **AngleFormat::Load `0x7ffb33a0`**（`3 < (short)ver` 拒绝）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>3 拒绝） | — |
//! | ② | short | 恒有 | this-0xc |
//! | ③ | NumericFormat::Load 内嵌核心 | 版本 >1（≥2） | this-0x10（独立槽；**原生不检查返回值**，核心失败后继续读本函数后续字段；本读取器按严格子集改为整体失败，较原生更严） |
//! | ④ | short | 版本 >2（≥3） | this-0xa |
//!
//! **ScientificFormat::Load `0x7ffda430`**（`2 < (short)ver` 拒绝）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>2 拒绝） | — |
//! | ② | int32 | 版本 ≥2 | this-0x8（版本 ≤1 原生置内存默认 6，流内无字段） |
//!
//! **CustomNumberFormat::Load `0x7ffbc4d0`**（`1 < (short)ver` 拒绝）：
//!
//! | # | 字段 | 版本门 | 对象偏移 |
//! | --- | --- | --- | --- |
//! | ① | int16 版本 | 恒有（>1 拒绝） | — |
//! | ② | AfCore String::Read（与 RateFormat ③ 同一 IAT 项 `?Read@String@@QAEJPAUIStream@@@Z`） | 恒有 | this-0x8 |
//!
//! Save 侧（写方向互证；前三变体与 Currency/Scientific/Custom 写**静态版
//! 本 short**；LatLon/Angle 经 `AfStreamGetSaveVersion(文档版本, 表长, 表)`
//! 查表写版本，表字节均已读出核对）：
//!
//! | 变体 | Save 实体 | 版本常量/表 | 常量值/表内容 | 形态 |
//! | --- | --- | --- | --- | --- |
//! | NumericFormat（锚） | `0x7ffce590` | `DAT_8006d0f4` | 1 | short + int32 ×4 + short ×3 |
//! | PercentageFormat | `0x7ffd1230` | `DAT_8006d0f0` | 2 | short + ② + NumericFormat::Save |
//! | FractionFormat | `0x7ffc6920` | `DAT_8006d10c` | 1 | short + int32 ×2 |
//! | RateFormat | `0x7ffd6a90` | `DAT_8006d0e0` | 2 | short + double + String + NumericFormat::Save |
//! | CurrencyFormat | `0x7ffbbc00` | `DAT_8006d208` | 1 | 仅 short |
//! | LatLonFormat | `0x7ffca2c0` | `DAT_8006d104` 表 | (3, 3, 4) | short + ②③ + ④ + NumericFormat::Save + 文档版本 ≥2 时 ⑥ |
//! | AngleFormat | `0x7ffb3430` | `DAT_8006d20c` 表 | (2, 2, 2, 2, 3) | short + ② + NumericFormat::Save + 文档版本 ≥4 时 ④ |
//! | ScientificFormat | `0x7ffda4a0` | `DAT_8006d0c8` | 2 | short + int32 |
//! | CustomNumberFormat | `0x7ffbc520` | `DAT_8006d204` | 1 | short + String::Write |
//!
//! 要点：LatLon/Angle 的 Save 与读侧版本门一致（LatLon 表最小 3 ≥ ④的版本
//! 门 2、Angle 表最小 2 ≥ ③的版本门 2，恒写满含核心形态）；写侧版本表按
//! 文档版本取值的索引/封顶语义未反编译确认（见证据文档未解决事项）。
//!
//! 各标量/字符串的业务语义未定性（对齐方式、小数位数、分母样式、单位比率
//! 描述、经纬度换算参数、角度格式、科学计数参数、自定义格式串等均为按类
//! 名与 NumericFormat 既有样本形态的候选）。
//!
//! ## 与 feature_layer_tail 的重复
//!
//! [`NUMERIC_FORMAT_CLASS_ID`] 与 [`read_numeric_format_body_10_1`]/
//! [`NumericFormatBody10_1`] 在 `src/layers/feature_layer_tail.rs` 已有语义
//! 相同的副本（第八轮落位，本模块不反向依赖 layers 层）。两处必须保持同
//! 步；**迁移建议（写给整合轮）**：删除 feature_layer_tail 的副本、改从本
//! 模块 `pub use` 重导出（或直接调用），`FieldInfoBody10_1` 的 INumberFormat
//! 槽分派从四类扩为九类（本模块九个读取器可直接作为槽位分派目标，槽位新
//! 对象解析成功即整体消费、失败回退语义不变；详见证据文档「同族五类闭合」
//! 节的接入建议）。
//!
//! 版本门均为原生有符号 short 比较；负值位模式本读取器一并拒绝（负值版本
//! 无业务意义，较原生更严）。这是仅覆盖上述分支的严格安全子集：要求完整
//! 输入，任何失败不推进调用者游标、不修改登记表；未接入 MXD 解析主链，
//! 人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见证据文档）：各类标量/String 字段的业务语义未定性；
//! Percentage/Rate 内嵌核心调用指针与外壳字段地址相同，但核心字段未覆盖外壳；LatLon/
//! Angle 内嵌核心（独立槽）仅由反编译证据支撑，无样本字节；短体形态（各
//! 类版本 ≤门限的分支）无样本；LatLon/Angle Save 版本表的索引语义未确认；
//! Angle 原生忽略核心 Load 失败的容错路径未在读取器中模拟；全部结论未运
//! 行 ArcMap 验证。

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

/// CurrencyFormat::Load 接受的最大版本：反编译 `(short)ver < 2` 方可通过；
/// 写侧 Save 版本常量 `DAT_8006d208` 恒 1。
const MAX_CURRENCY_VERSION: i16 = 1;

/// LatLonFormat::Load 接受的最大版本：反编译 `4 < (short)ver` 有符号拒绝；
/// 写侧 Save 版本表 `DAT_8006d104` = (3, 3, 4)，最小 3、最大 4。
const MAX_LATLON_VERSION: i16 = 4;

/// AngleFormat::Load 接受的最大版本：反编译 `3 < (short)ver` 有符号拒绝；
/// 写侧 Save 版本表 `DAT_8006d20c` = (2, 2, 2, 2, 3)，最小 2、最大 3。
const MAX_ANGLE_VERSION: i16 = 3;

/// ScientificFormat::Load 接受的最大版本：反编译 `2 < (short)ver` 有符号拒
/// 绝；写侧 Save 版本常量 `DAT_8006d0c8` 恒 2。
const MAX_SCIENTIFIC_VERSION: i16 = 2;

/// CustomNumberFormat::Load 接受的最大版本：反编译 `1 < (short)ver` 有符号
/// 拒绝；写侧 Save 版本常量 `DAT_8006d204` 恒 1。
const MAX_CUSTOM_NUMBER_VERSION: i16 = 1;

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

/// CurrencyFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F471A-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F471A}` progID `CurrencyFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass CurrencyFormat；
/// SystemCore.dll GetClassID `0x7ffbb850` 从 .rdata 静态字节
/// `0x8006e2dc`/`0x8006e2e4` 写出 16 字节 `1a474f7e548ed211aad8000000000000`
///（IPersistStream 虚表 `0x8006e294` 槽 +0x0c 经虚位移调整器
/// `0x7ffbbd40` 指向）。样本 0 实例。
pub const CURRENCY_FORMAT_CLASS_ID: [u8; 16] = [
    0x1a, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// LatLonFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F471D-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F471D}` progID `LatLonFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass LatLonFormat；
/// SystemCore.dll GetClassID `0x7ffca1a0` 从 .rdata 静态字节
/// `0x800702b0`/`0x800702b8` 写出 16 字节 `1d474f7e548ed211aad8000000000000`
///（IPersistStream 虚表 `0x80070250` 槽 +0x0c 经虚位移调整器
/// `0x7ffca4a0` 指向）。样本 0 实例。
pub const LATLON_FORMAT_CLASS_ID: [u8; 16] = [
    0x1d, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// AngleFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F471E-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F471E}` progID `AngleFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass AngleFormat；
/// SystemCore.dll GetClassID `0x7ffb3360` 从 .rdata 静态字节
/// `0x8006d3f8`/`0x8006d400` 写出 16 字节 `1e474f7e548ed211aad8000000000000`
///（IPersistStream 虚表 `0x8006d37c` 槽 +0x0c 经虚位移调整器
/// `0x7ffb35e0` 指向）。样本 0 实例。
pub const ANGLE_FORMAT_CLASS_ID: [u8; 16] = [
    0x1e, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// ScientificFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F471F-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F471F}` progID `ScientificFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass ScientificFormat；
/// SystemCore.dll GetClassID `0x7ffda3f0` 从 .rdata 静态字节
/// `0x80072570`/`0x80072578` 写出 16 字节 `1f474f7e548ed211aad8000000000000`
///（IPersistStream 虚表 `0x80072540` 槽 +0x0c 经虚位移调整器
/// `0x7ffda6a0` 指向）。样本 0 实例。
pub const SCIENTIFIC_FORMAT_CLASS_ID: [u8; 16] = [
    0x1f, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// CustomNumberFormat 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7E4F4722-8E54-11D2-AAD8-000000000000}`。证据：
/// `esri.clsid.ecfg` 声明 `{7E4F4722}` progID `CustomNumberFormat`（模块
/// SystemCore.dll）；esriSystem 类型库 coclass CustomNumberFormat；
/// SystemCore.dll GetClassID `0x7ffbbdd0` 从 .rdata 静态字节
/// `0x8006e404`/`0x8006e40c` 写出 16 字节 `22474f7e548ed211aad8000000000000`
///（IPersistStream 虚表 `0x8006e3cc` 槽 +0x0c 经虚位移调整器
/// `0x7ffbc6b0` 指向）。样本 0 实例。
pub const CUSTOM_NUMBER_FORMAT_CLASS_ID: [u8; 16] = [
    0x22, 0x47, 0x4f, 0x7e, 0x54, 0x8e, 0xd2, 0x11, 0xaa, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
/// 建议见模块文档）；**注意**：作为 Percentage/Rate/LatLon/Angle 的内嵌核
/// 心复用时，流内没有独立对象头与键，`key` 传外层对象的键（同一流内对象）。
#[derive(Debug, Clone, PartialEq)]
pub struct NumericFormatBody10_1 {
    /// 对象头键（样本 42/53/55/57；内嵌核心复用时与外壳同键）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表（由槽位分派完成）。
    pub key: u32,
    /// 体首 short 版本（样本 1）；>1 原生拒绝。
    pub version: u16,
    /// 依次为 RoundingOption、RoundingValue、AlignmentOption、AlignmentWidth。
    /// 保留原始整数；Load 不应用属性 setter 的取值限制。
    pub int32s: [i32; 4],
    /// 依次为 UseSeparator、ZeroPad、ShowPlusSign，保留原始 16 位值。
    pub shorts: [i16; 3],
}

/// 从对象头之后的位置读取 10.1 NumericFormat 核心体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头（或作为 Percentage/Rate/LatLon/Angle 的内
/// 嵌核心从外壳字段之后调用）。证据：SystemCore.dll 10.1.0.3035 Load
/// `0x7ffce4b0`（版本门 `1 < (short)ver` 有符号拒绝）与 Save `0x7ffce590`
///（对称）。`table` 仅在签名上保持一致（无嵌套对象条目）。
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
/// ② short → obj-0xc → ③ 版本 ≥2：NumericFormat 核心体（无独立
/// 对象头）。版本 0..1 的短体仅 4 字节（②即终结），无样本佐证（仅反编译）。
#[derive(Debug, Clone, PartialEq)]
pub struct PercentageFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝，写侧恒 2。
    pub version: u16,
    /// short → obj-0xc（恒有）；IPercentageFormat 的 AdjustPercentage，保留原始位模式。
    pub short_0c: i16,
    /// 版本 ≥2 的内嵌 NumericFormat 核心体（`None` = 版本 ≤1）。
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
    // ③ 版本 ≥2：内嵌 NumericFormat 核心（无独立对象头，同键）。
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
    /// int32 → obj-0x10（恒有）；IFractionFormat 的 FractionOption 原始枚举值。
    pub int32_10: i32,
    /// int32 → obj-0xc（恒有）；IFractionFormat 的 FractionFactor 原始值。
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
/// Format 核心体（传入 obj-0x14，核心字段不覆盖 ②；无独立对象头）。版本
/// 0..1 的短体止于 ③，无样本佐证（仅反编译）。
#[derive(Debug, Clone, PartialEq)]
pub struct RateFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝，写侧恒 2。
    pub version: u16,
    /// double → obj-0x14（恒有）；IRateFormat 的 RateFactor，原样保留。
    pub double_14: f64,
    /// AfCore String → obj-0xc（恒有）；IRateFormat 的 RateString。
    pub string_0c: String,
    /// 版本 >1 的内嵌 NumericFormat 核心体（`None` = 版本 ≤1）。
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
    // ④ 版本 >1：内嵌 NumericFormat 核心（传入 obj-0x14；同键）。
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

/// CurrencyFormat（SystemCore.dll `0x7ffbbbc0`）体的已确认字段。
///
/// 体仅 2 字节（short 版本），无任何其他字段、无版本分支；写侧 Save 版本
/// 常量恒 1，形态与读侧一致（仅 short）。
#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝，写侧恒 1。
    pub version: u16,
}

/// 从对象头之后的位置读取 10.1 CurrencyFormat 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`CURRENCY_FORMAT_CLASS_ID`] 并取得
/// 键 `key`。证据：SystemCore.dll 10.1.0.3035 Load `0x7ffbbbc0` 与 Save
/// `0x7ffbbc00`（版本常量 1，对称）。`table` 仅在签名上保持一致（无嵌套
/// 对象条目）。
pub fn read_currency_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<CurrencyFormatBody10_1> {
    let _ = table; // 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >1 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(
        &mut trial,
        MAX_CURRENCY_VERSION,
        "CurrencyFormat",
        entry_offset,
    )?;
    let result = CurrencyFormatBody10_1 { key, version };
    *reader = trial;
    Ok(result)
}

/// LatLonFormat（SystemCore.dll `0x7ffca1e0`）体的已确认字段。
///
/// 字段序：① short 版本（`4 < (short)ver` 有符号拒绝）→ ② short → obj
/// -0x2c → ③ short → obj-0x2a → ④ 版本 ≥2：short ×2 → obj-0xc/-0xa（≤1 时
/// 原生置内存默认 -1/-1，流内无字段）→ ⑤ 版本 ≥3：NumericFormat 核心体
///（传入 obj-0x38；返回值受检）→ ⑥ 版本 ≥4：short →
/// obj-0x8。短体形态（版本 ≤门限）无样本佐证（仅反编译）。
#[derive(Debug, Clone, PartialEq)]
pub struct LatLonFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>4 原生拒绝，写侧版本表 (3, 3, 4)。
    pub version: u16,
    /// short → obj-0x2c（恒有）；ILatLonFormat 的 ShowDirections 原始位模式。
    pub short_2c: i16,
    /// short → obj-0x2a（恒有）；ILatLonFormat 的 IsLatitude 原始位模式。
    pub short_2a: i16,
    /// 版本 ≥2：short → obj-0xc；ILatLonFormat 的 ShowZeroMinutes（`None` = 版本 ≤1，原生内存默认 -1）。
    pub short_0c: Option<i16>,
    /// 版本 ≥2：short → obj-0xa；ILatLonFormat 的 ShowZeroSeconds（`None` = 版本 ≤1，原生内存默认 -1）。
    pub short_0a: Option<i16>,
    /// 版本 ≥3 的内嵌 NumericFormat 核心体（传入 obj-0x38）。
    pub numeric: Option<Box<NumericFormatBody10_1>>,
    /// 版本 ≥4：short → obj-0x8；非零时格式输出使用小数分并省略独立秒，
    /// 零时使用独立分、秒；保留原始位模式，正式接口属性名未确认。
    pub short_08: Option<i16>,
}

/// 从对象头之后的位置读取 10.1 LatLonFormat 对象体，失败时不推进游标、不
/// 修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LATLON_FORMAT_CLASS_ID`] 并取得键
/// `key`（内嵌核心与外壳同键）。证据：SystemCore.dll 10.1.0.3035 Load
/// `0x7ffca1e0` 与 Save `0x7ffca2c0`（Save 版本表 (3, 3, 4)，写侧恒含核心
/// 形态，与读侧版本门一致）。`table` 仅在签名上保持一致（核心无嵌套对象
/// 条目）。
pub fn read_latlon_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<LatLonFormatBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >4 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(&mut trial, MAX_LATLON_VERSION, "LatLonFormat", entry_offset)?;
    // ② short → obj-0x2c；③ short → obj-0x2a（均恒有）。
    let short_2c = trial.read_u16_le()? as i16;
    let short_2a = trial.read_u16_le()? as i16;
    // ④ 版本 ≥2：short ×2 → obj-0xc/-0xa。
    let (short_0c, short_0a) = if (version as i16) > 1 {
        (
            Some(trial.read_u16_le()? as i16),
            Some(trial.read_u16_le()? as i16),
        )
    } else {
        (None, None)
    };
    // ⑤ 版本 ≥3：内嵌 NumericFormat 核心（obj-0x38 独立槽；返回值原生受检）。
    let numeric = if (version as i16) > 2 {
        Some(Box::new(read_numeric_format_body_10_1(
            &mut trial, table, key,
        )?))
    } else {
        None
    };
    // ⑥ 版本 ≥4：short → obj-0x8。
    let short_08 = if (version as i16) > 3 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    let result = LatLonFormatBody10_1 {
        key,
        version,
        short_2c,
        short_2a,
        short_0c,
        short_0a,
        numeric,
        short_08,
    };
    *reader = trial;
    Ok(result)
}

/// AngleFormat（SystemCore.dll `0x7ffb33a0`）体的已确认字段。
///
/// 字段序：① short 版本（`3 < (short)ver` 有符号拒绝）→ ② short → obj
/// -0xc → ③ 版本 ≥2：NumericFormat 核心体（内嵌 obj-0x10 独立槽；**原生不
/// 检查核心 Load 返回值**，失败后继续读后续字段；本读取器按严格子集改为
/// 整体失败，较原生更严）→ ④ 版本 ≥3：short → obj-0xa。短体形态（版本
/// ≤门限）无样本佐证（仅反编译）。
#[derive(Debug, Clone, PartialEq)]
pub struct AngleFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>3 原生拒绝，写侧版本表 (2, 2, 2, 2, 3)。
    pub version: u16,
    /// short → obj-0xc（恒有）；IAngleFormat 的 AngleInDegrees 原始位模式。
    pub short_0c: i16,
    /// 版本 ≥2 的内嵌 NumericFormat 核心体（obj-0x10 独立槽）。
    pub numeric: Option<Box<NumericFormatBody10_1>>,
    /// 版本 ≥3：short → obj-0xa；IAngleFormat 的 DisplayDegrees 原始位模式。
    pub short_0a: Option<i16>,
}

/// 从对象头之后的位置读取 10.1 AngleFormat 对象体，失败时不推进游标、不修
/// 改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`ANGLE_FORMAT_CLASS_ID`] 并取得键
/// `key`（内嵌核心与外壳同键）。证据：SystemCore.dll 10.1.0.3035 Load
/// `0x7ffb33a0` 与 Save `0x7ffb3430`（Save 版本表 (2, 2, 2, 2, 3)，写侧恒
/// 含核心形态，与读侧版本门一致）。`table` 仅在签名上保持一致（核心无嵌
/// 套对象条目）。
pub fn read_angle_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<AngleFormatBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >3 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(&mut trial, MAX_ANGLE_VERSION, "AngleFormat", entry_offset)?;
    // ② short → obj-0xc。
    let short_0c = trial.read_u16_le()? as i16;
    // ③ 版本 ≥2：内嵌 NumericFormat 核心（obj-0x10 独立槽）。原生忽略核心
    // 失败继续读 ④；此处按严格子集传播失败（较原生更严）。
    let numeric = if (version as i16) > 1 {
        Some(Box::new(read_numeric_format_body_10_1(
            &mut trial, table, key,
        )?))
    } else {
        None
    };
    // ④ 版本 ≥3：short → obj-0xa。
    let short_0a = if (version as i16) > 2 {
        Some(trial.read_u16_le()? as i16)
    } else {
        None
    };
    let result = AngleFormatBody10_1 {
        key,
        version,
        short_0c,
        numeric,
        short_0a,
    };
    *reader = trial;
    Ok(result)
}

/// ScientificFormat（SystemCore.dll `0x7ffda430`）体的已确认字段。
///
/// 字段序：① short 版本（`2 < (short)ver` 有符号拒绝）→ ② 版本 ≥2：int32
/// → obj-0x8；版本 ≤1 流内无字段（原生置内存默认 6，精度候选）。写侧恒写
/// 版本 2 + int32（满形态）。无内嵌核心、无字符串。
#[derive(Debug, Clone, PartialEq)]
pub struct ScientificFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝，写侧恒 2。
    pub version: u16,
    /// 版本 ≥2：int32 → obj-0x8；IScientificNumberFormat 的 DecimalPlaces（`None` = 版本 ≤1，原生内存默认 6）。
    pub int32_08: Option<i32>,
}

/// 从对象头之后的位置读取 10.1 ScientificFormat 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SCIENTIFIC_FORMAT_CLASS_ID`] 并取
/// 得键 `key`。证据：SystemCore.dll 10.1.0.3035 Load `0x7ffda430` 与 Save
/// `0x7ffda4a0`（版本常量 2，对称）。`table` 仅在签名上保持一致（无嵌套
/// 对象条目）。
pub fn read_scientific_format_body_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ScientificFormatBody10_1> {
    let _ = table; // 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >2 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(
        &mut trial,
        MAX_SCIENTIFIC_VERSION,
        "ScientificFormat",
        entry_offset,
    )?;
    // ② 版本 ≥2：int32 → obj-0x8。
    let int32_08 = if (version as i16) > 1 {
        Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
    } else {
        None
    };
    let result = ScientificFormatBody10_1 {
        key,
        version,
        int32_08,
    };
    *reader = trial;
    Ok(result)
}

/// CustomNumberFormat（SystemCore.dll `0x7ffbc4d0`）体的已确认字段。
///
/// 字段序：① short 版本（`1 < (short)ver` 有符号拒绝；写侧恒 1）→ ②
/// AfCore String → obj-0x8（FormatString；与 RateFormat ③ 同一 IAT 项
/// `?Read@String@@QAEJPAUIStream@@@Z`，AfCore.dll）。无内嵌核心、无版本
/// 分支字段。
#[derive(Debug, Clone, PartialEq)]
pub struct CustomNumberFormatBody10_1 {
    /// 对象头键（槽位分派传入；样本 0 实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝，写侧恒 1。
    pub version: u16,
    /// AfCore String → obj-0x8（恒有）；ICustomNumberFormat 的 FormatString。
    pub string_08: String,
}

/// 从对象头之后的位置读取 10.1 CustomNumberFormat 对象体，失败时不推进游
/// 标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`CUSTOM_NUMBER_FORMAT_CLASS_ID`] 并
/// 取得键 `key`。`max_bytes` 是资源限额，应用于 ② 的声明字符串长度（含终
/// 止零、不含 4 字节前缀），在任何读取与分配之前检查；`usize::MAX` 表示不
/// 额外设限。证据：SystemCore.dll 10.1.0.3035 Load `0x7ffbc4d0` 与 Save
/// `0x7ffbc520`（版本常量 1，对称）。
pub fn read_custom_number_format_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<CustomNumberFormatBody10_1> {
    let _ = table; // 无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    // ① 版本 short（有符号 >1 拒绝；负值位模式一并拒绝，较原生更严）。
    let version = read_version_short(
        &mut trial,
        MAX_CUSTOM_NUMBER_VERSION,
        "CustomNumberFormat",
        entry_offset,
    )?;
    // ② AfCore String → obj-0x8。
    let string_08 = read_string_10_1(&mut trial, max_bytes)?;
    let result = CustomNumberFormatBody10_1 {
        key,
        version,
        string_08,
    };
    *reader = trial;
    Ok(result)
}
