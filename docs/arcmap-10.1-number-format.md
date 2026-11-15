# ArcMap 10.1 INumberFormat 族三变体：PercentageFormat / FractionFormat / RateFormat

记录日期：2026-11-15。本轮闭合 FieldInfo INumberFormat 槽（`{7E4F470D}`）同族的
剩余三个 NumericFormat 变体体。全部为静态取证（本机 pyghidra，Ghidra 12.2 +
JDK 25，打开既有工程副本），未运行 ArcMap 10.1，未人工验证。样本
`samples/测试底图.mxd` 的 /Maps 流内三变体 **0 实例**（INumberFormat 槽 14 条
FieldInfo 中仅 4 条非空且均为 NumericFormat），字段序全部来自反编译，经
Save 侧对称与版本常量字节互证；合成输入测试覆盖（`tests/number_format.rs`）。
本文无本机绝对路径；地址均为 SystemCore.dll 10.1.0.3035 静态映像 VA。

## 文件身份

- SystemCore.dll：Setup1.cab 条目 `SystemCore.dll.017F52BF_…`，1134080 字节，
  SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`
  （与基线一致），pefile 复核 PE x86、ImageBase `0x7ffb0000`、VS_FIXEDFILEINFO
  10.1.0.3035。工程缓存为既有 sc 工程（`featurelayer-tail` 第八轮所建）的副本，
  打开时逐锚点复核（NumericFormat GetClassID `0x7ffce470`、Load `0x7ffce4b0`、
  Save `0x7ffce590`、IPersistStream 虚表 `0x80070ec4`，与
  `docs/arcmap-10.1-featurelayer-tail.md`「NumericFormat 体」节逐项一致）。
- 类名/IID：`esri.clsid.ecfg` 全表（`PercentageFormat` `{7E4F471B}`、
  `FractionFormat` `{7E4F471C}`、`RateFormat` `{7E4F4721}`，模块均声明
  SystemCore.dll）与 esriSystem 类型库 dump（coclass #214/#215/#220
  typekind=5；接口 IPercentageFormat `{7E4F4711}`、IFractionFormat
  `{7E4F4713}`、IRateFormat `{7E4F4717}` typekind=3；接口名与 GUID 的绑定
  为类型库文件内静态数据）。

## 定位链（RTTI TD→COL→虚表→GetClassID；判据 [C]==0 且 [C+0xc]==TD）

方法备注：32 位 MSVC 的 TypeDescriptor 名字字符串在 TD+8（不是 TD+0xc）；
本节所有 TD 由 RTTI 名字符串 VA-8 取得，COL 由 TD 绝对指针在 .rdata 的 4 字
节对齐命中取得（[C]==0 逐一核对）。虚表 = 持有 COL 指针的位置 +4；槽位函数
是 `mov ecx,[esp+4]; sub ecx,N; mov [esp+4],ecx; jmp 实体` 形态的 this 调整
器，实体由末尾 `jmp rel32` 解出。

| 变体 | RTTI TD | 首 COL | 虚表 | +0x0c GetClassID | +0x14 Load | +0x18 Save | 证据 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| NumericFormat（锚，既有） | `0x800a5984` | `0x80082d74` | `0x80070ec4` | `0x7ffce470` | `0x7ffce4b0` | `0x7ffce590` | 确定的 |
| PercentageFormat | `0x800a6878` | `0x80083280` | `0x80071404` | `0x7ffd1170` | `0x7ffd11b0` | `0x7ffd1230` | 确定的 |
| FractionFormat | `0x800a6398` | `0x80081cc0` | `0x8006fc38` | `0x7ffc6120` | `0x7ffc68a0` | `0x7ffc6920` | 确定的 |
| RateFormat | `0x800a6a18` | `0x80083a88` | `0x80071d94` | `0x7ffd65f0` | `0x7ffd6a10` | `0x7ffd6a90` | 确定的 |

三方互证：esri.clsid 声明 ↔ GetClassID 立即数写出（`0x11D28E547E4F471B`/
`0x000000000000D8AA`、`0x…471C`、`0x…4721` 同模式）↔ 虚表槽 +0x0c 指向。
CLSID 常量字节 `0x8007143c`/`0x8006fc70`/`0x80071dcc`（.rdata）另有
`DllGetClassObject` 类表数据引用（同函数 `0x80066910` 内，与锚
NumericFormat `0x8006d3e8` 同模式），属类表登记而非第二实现。

## 字段序（Load 反编译；证据：确定的——反编译 + Save 对称互证）

**PercentageFormat::Load `0x7ffd11b0`**（版本门 `2 < (short)ver` 有符号拒绝
`0x80004005`）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>2 拒绝） | — |
| ② | short | 恒有 | this-0xc |
| ③ | NumericFormat::Load(`0x7ffce4b0`) 内嵌核心（short 版本 + int32 ×4 + short ×3） | 版本 ≥2 | this-0xc（同槽升级） |

**FractionFormat::Load `0x7ffc68a0`**（`1 < (short)ver` 拒绝）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>1 拒绝） | — |
| ② | int32 | 恒有 | this-0x10 |
| ③ | int32 | 恒有 | this-0xc |

**RateFormat::Load `0x7ffd6a10`**（`2 < (short)ver` 拒绝）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>2 拒绝） | — |
| ② | double | 恒有 | this-0x14 |
| ③ | AfCore String::Read | 恒有 | this-0xc |
| ④ | NumericFormat::Load 内嵌核心 | 版本 >1（≥2） | this-0x14（同槽升级，覆盖 ②） |

Save 侧（写方向互证，均写**静态版本 short**，无 AfStreamGetSaveVersion 表，
版本常量字节逐一读出核对）：

| 变体 | Save 实体 | 版本常量地址 | 常量值 | 形态 |
| --- | --- | --- | --- | --- |
| NumericFormat（锚） | `0x7ffce590` | `DAT_8006d0f4` | 1 | short + int32 ×4 + short ×3 |
| PercentageFormat | `0x7ffd1230` | `DAT_8006d0f0` | 2 | short + ② + NumericFormat::Save |
| FractionFormat | `0x7ffc6920` | `DAT_8006d10c` | 1 | short + int32 ×2 |
| RateFormat | `0x7ffd6a90` | `DAT_8006d0e0` | 2 | short + double + String + NumericFormat::Save |

要点：

- Percentage/Rate 的内嵌 NumericFormat 核心与外壳**同槽升级**（版本 ≥2 时
  `this-0xc` / `this-0x14` 由单标量升级为核心对象起点），流内为顺序字节
  （外壳字段后紧跟核心的版本 short + int32 ×4 + short ×3），无独立对象头、
  无独立键；这是「版本化继承」写法，与锚 NumericFormat 自身体完全同构。
- 版本 ≤门限（Percentage 0..1、Fraction 0、Rate 0..1）的短体形态只有反编
  译证据，无样本；写侧恒写满形态（版本常量 2/1/2）。
- 版本门全部为原生有符号 short 比较；负值位模式原生可通过门（落到读字段
  路径）。读取器一律拒绝负值位模式（较原生更严），与仓库既有读取器约定
  一致。

## 实现（src/）

- 新模块 `src/objects/number_format.rs`：
  四个类标识常量与四个体读取器（`read_numeric_format_body_10_1`/
  `read_percentage_format_body_10_1`/`read_fraction_format_body_10_1`/
  `read_rate_format_body_10_1`），严格事务子集（失败不推进游标、不修改登
  记表；Rate 的 String 受 `max_bytes` 限额）。
- **与 feature_layer_tail 的重复（迁移建议，写给整合轮）**：
  `NUMERIC_FORMAT_CLASS_ID`/`NumericFormatBody10_1`/
  `read_numeric_format_body_10_1` 在 `src/layers/feature_layer_tail.rs` 有
  语义相同的既有副本（第八轮落位）；objects 层不反向依赖 layers 层，故本
  模块自带副本，两处必须同步。整合轮建议：删除 feature_layer_tail 副本改
  为重导出/调用本模块，并把 `FieldInfoBody10_1` 的 INumberFormat 槽分派从
  「仅 NumericFormat」扩为四类（Percentage/Fraction/Rate 读取器可直接作为
  槽位分派目标；未知类仍为停止点回退）。

## 测试

`tests/number_format.rs` 10 项：类标识字节互证、NumericFormat 核心 24 字节
体、Percentage 版本 2 满形态/版本 1 短体/版本超限与负值位模式拒绝、
Fraction 10 字节体与版本 2 拒绝、Rate 版本 2 满形态（double+中文 String+
内嵌核心）/版本 1 短体/版本与限额拒绝、三读取器逐前缀截断的事务性。全部
为合成输入（样本 0 实例），通过不代表 MXD 格式已验证。

## 未解决事项

1. 三变体全部标量与 String 字段的业务语义未定性（对齐方式/位数/分母样
   式/单位描述等均为按类名与 NumericFormat 样本形态的候选）。
2. Percentage/Rate 内嵌核心的「同槽升级」仅由反编译证据支撑（读侧写 this
   -0xc/-0x14、写侧对称），无样本字节；短体形态（版本 ≤门限）同样无样
   本。
3. INumberFormat 槽位的四类分派接线未做（本模块只提供读取器；见上文迁移
   建议）。esriSystem 类型库中其余同族类（CurrencyFormat `{7E4F471A}`、
   LatLonFormat `{7E4F471D}`、AngleFormat `{7E4F471E}`、ScientificFormat
   `{7E4F471F}`、CustomNumberFormat `{7E4F4722}`）体未研究。
4. 全部结论为静态取证；未运行 ArcMap 10.1，未人工目视验收，不声称已验证。

## 接线轮落位（2026-11-15）

迁移建议已落位：`feature_layer_tail` 的 NumericFormat 类标识/体结构/读取器
副本删除，改为自本模块 `pub use` 重导出（既有调用方与测试零改动）；FieldInfo
体 ⑤ INumberFormat 槽分派扩为 NumericFormat/PercentageFormat/FractionFormat/
RateFormat 四类（三变体读取器直接作为槽位分派目标，满形态完整消费，其余
esriSystem 同族类维持 `NumberFormatUnsupported` 停止）。样本槽仅含
NumericFormat，样本行为不变；四读取器已入 robustness 集中清单。
