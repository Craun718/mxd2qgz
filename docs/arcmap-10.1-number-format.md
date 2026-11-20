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
| ③ | NumericFormat::Load(`0x7ffce4b0`) 内嵌核心（short 版本 + int32 ×4 + short ×3） | 版本 ≥2 | 传入 this-0xc（读入地址低于该指针） |

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
| ④ | NumericFormat::Load 内嵌核心 | 版本 >1（≥2） | 传入 this-0x14（不覆盖 ②） |

Save 侧（写方向互证，均写**静态版本 short**，无 AfStreamGetSaveVersion 表，
版本常量字节逐一读出核对）：

| 变体 | Save 实体 | 版本常量地址 | 常量值 | 形态 |
| --- | --- | --- | --- | --- |
| NumericFormat（锚） | `0x7ffce590` | `DAT_8006d0f4` | 1 | short + int32 ×4 + short ×3 |
| PercentageFormat | `0x7ffd1230` | `DAT_8006d0f0` | 2 | short + ② + NumericFormat::Save |
| FractionFormat | `0x7ffc6920` | `DAT_8006d10c` | 1 | short + int32 ×2 |
| RateFormat | `0x7ffd6a90` | `DAT_8006d0e0` | 2 | short + double + String + NumericFormat::Save |

要点：

- Percentage/Rate 的外壳字段后顺序保存内嵌 NumericFormat 核心，无独立对象头或键。
  Load 传入外壳字段地址，但核心各读入地址位于该指针之前，不会覆盖外壳字段，
  见 [指令复核](mxd-terra-next-format.md#原生证据与更正)。
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
2. Percentage/Rate 内嵌核心及各旧版短体的边界已补做
   [原生取证](#内嵌核心与旧版对象体的原生取证)。暂无含这些分支的实际 MXD 宿主。
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

## 同族五类闭合（2026-11-15）

本轮闭合 esriSystem 同族其余五类：CurrencyFormat `{7E4F471A}`、LatLonFormat
`{7E4F471D}`、AngleFormat `{7E4F471E}`、ScientificFormat `{7E4F471F}`、
CustomNumberFormat `{7E4F4722}`。全部为静态取证（本机 pyghidra，Ghidra 12.2
+ JDK 25，打开第九/杂项轮工程副本并逐锚复核：NumericFormat/Percentage 四锚
与上文表格逐项一致），未运行 ArcMap 10.1，未人工验证。样本
`samples/测试底图.mxd` 全文件（全部流）按 16 字节 CLSID 内存序原样字节扫描，
五类 **0 命中**（NumericFormat 138 命中、Percentage/Fraction/Rate 同为 0，
与上文一致），即五类**无样本字节佐证**，字段序全部来自反编译。地址均为
SystemCore.dll 10.1.0.3035（SHA 与文件身份同上文）静态映像 VA；类身份证据
补充：五类 CLSID 在 `esri.clsid.ecfg`（本轮以解码表复核）均声明于
`<Module name="SystemCore.dll" progIDPrefix="esriSystem">` 下。

### 定位链（同判据 [C]==0 且 [C+0xc]==TD；TD=RTTI 名字符串 VA-8，名形
`.?AV<类名>@@`）

| 变体 | RTTI TD | 首 COL | 虚表 | +0x0c GetClassID | +0x14 Load | +0x18 Save | 证据 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CurrencyFormat | `0x800a5f40` | `0x800808c4` | `0x8006e294` | `0x7ffbb850` | `0x7ffbbbc0` | `0x7ffbbc00` | 确定的 |
| LatLonFormat | `0x800a64a0` | `0x80082114` | `0x80070250` | `0x7ffca1a0` | `0x7ffca1e0` | `0x7ffca2c0` | 确定的 |
| AngleFormat | `0x800a59a0` | `0x8007edf8` | `0x8006d37c` | `0x7ffb3360` | `0x7ffb33a0` | `0x7ffb3430` | 确定的 |
| ScientificFormat | `0x800a6aa8` | `0x80083e34` | `0x80072540` | `0x7ffda3f0` | `0x7ffda430` | `0x7ffda4a0` | 确定的 |
| CustomNumberFormat | `0x800a5f60` | `0x800809ec` | `0x8006e3cc` | `0x7ffbbdd0` | `0x7ffbc4d0` | `0x7ffbc520` | 确定的 |

与既有四变体的两点实现差异（均逐槽复核确认）：

- GetClassID 写出形态：五类均不从指令立即数写 CLSID，而是从 .rdata 静态
  16 字节复制（两段 8 字节）：Currency `0x8006e2dc`/`0x8006e2e4`、LatLon
  `0x800702b0`/`0x800702b8`、Angle `0x8006d3f8`/`0x8006d400`、Scientific
  `0x80072570`/`0x80072578`、CustomNumber `0x8006e404`/`0x8006e40c`；字节
  逐一读出，分别为 `1a474f7e…`/`1d474f7e…`/`1e474f7e…`/`1f474f7e…`/
  `22474f7e…` + `548ed211aad8000000000000`，与 esri.clsid 声明一致。
- this 调整器形态：五类虚表槽为 `mov ecx,[esp+4]; sub ecx,[ecx-4];
  mov [esp+4],ecx; jmp rel32`（虚位移从对象读，非 imm8）；Load/Save 实体
  由末尾 `jmp` 解出（上表）。Percentage 锚的同名链逐槽复核与上文一致。

### 字段序（Load 反编译 + 关键分支指令级复核；证据：确定的）

**CurrencyFormat::Load `0x7ffbbbc0`**（`(short)ver < 2` 方可通过）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>1 拒绝） | — |

体止于版本 short（2 字节），无任何其他字段。

**LatLonFormat::Load `0x7ffca1e0`**（`4 < (short)ver` 有符号拒绝）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>4 拒绝） | — |
| ② | short | 恒有 | this-0x2c |
| ③ | short | 恒有 | this-0x2a |
| ④ | short + short | 版本 >1（≥2） | this-0xc / this-0xa（≤1 时原生置内存默认 -1/-1，流内无字段） |
| ⑤ | NumericFormat::Load(`0x7ffce4b0`) 内嵌核心 | 版本 >2（≥3） | 传入 this-0x38（返回值受检，失败即 Load 失败） |
| ⑥ | short | 版本 >3（≥4） | this-0x8 |

**AngleFormat::Load `0x7ffb33a0`**（`3 < (short)ver` 有符号拒绝）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>3 拒绝） | — |
| ② | short | 恒有 | this-0xc |
| ③ | NumericFormat::Load 内嵌核心 | 版本 >1（≥2） | this-0x10（独立槽；**原生不检查返回值**（指令级核实：`CALL 0x7ffce4b0` 后无 TEST/JL 检查），核心失败后继续读本函数 ④；读取器按严格子集改为整体失败，较原生更严） |
| ④ | short | 版本 >2（≥3） | this-0xa |

**ScientificFormat::Load `0x7ffda430`**（`2 < (short)ver` 有符号拒绝）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>2 拒绝） | — |
| ② | int32 | 版本 ≥2 | this-0x8（版本 ≤1 原生置内存默认 6，流内无字段；精度候选） |

**CustomNumberFormat::Load `0x7ffbc4d0`**（`1 < (short)ver` 有符号拒绝）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有（>1 拒绝） | — |
| ② | AfCore String::Read | 恒有 | this-0x8（自定义格式串候选） |

CustomNumberFormat ② 与 RateFormat ③ 为**同一 IAT 项**
`0x8006b1f0`（AfCore.dll `?Read@String@@QAEJPAUIStream@@@Z`，导入表逐一
解析复核），与仓库既有 `read_string_10_1` 覆盖的写侧布局同源。

**核心升级形态归属（逐类核实，未类推）**：内嵌 NumericFormat 核心仅
LatLon（版本 ≥3，this-0x38 独立槽，受检）与 Angle（版本 ≥2，this-0x10 独
立槽，**原生不受检**）两类有；Currency/Scientific/CustomNumber **无核心**
（反编译无 `0x7ffce4b0` 调用）。Percentage/Rate 的传参指针和外壳字段地址相同，
但核心并不写入该地址；以上传参指针均不能直接当作读入偏移。

### Save 侧（写方向互证；版本常量/表字节均已读出核对）

| 变体 | Save 实体 | 版本常量/表 | 常量值/表内容 | 形态 |
| --- | --- | --- | --- | --- |
| CurrencyFormat | `0x7ffbbc00` | `DAT_8006d208` | 1 | 仅 short |
| LatLonFormat | `0x7ffca2c0` | `DAT_8006d104` 表 | (3, 3, 4) | short + ②③ + ④ + NumericFormat::Save + 文档版本 ≥2 时 ⑥ |
| AngleFormat | `0x7ffb3430` | `DAT_8006d20c` 表 | (2, 2, 2, 2, 3) | short + ② + NumericFormat::Save + 文档版本 ≥4 时 ④ |
| ScientificFormat | `0x7ffda4a0` | `DAT_8006d0c8` | 2 | short + int32 |
| CustomNumberFormat | `0x7ffbc520` | `DAT_8006d204` | 1 | short + String::Write |

LatLon/Angle 的 Save 经 `AfStreamGetSaveVersion(文档版本, 表长, 表)`
（AfCore.dll 导入，签名由符号名解析）查表写版本，与上文四变体的静态常量
写法不同；表最小值（3 / 2）不低于各自内嵌核心的读侧版本门（3 / 2），写
侧恒为含核心的满形态，与读侧版本门一致。**表的索引/封顶语义未反编译**：
仅确认调用形态与表字节，按文档版本取值的规则（含越界封顶行为）是候选而
非结论。

### 实现（src/objects/number_format.rs 扩展）

五个类标识常量（`CURRENCY_FORMAT_CLASS_ID` 等）与五个体读取器
（`read_currency_format_body_10_1`/`read_latlon_format_body_10_1`/
`read_angle_format_body_10_1`/`read_scientific_format_body_10_1`/
`read_custom_number_format_body_10_1`），严格事务子集（失败不推进游标、
不修改登记表；CustomNumber 的 String 受 `max_bytes` 限额）。与原生的已知
偏离（均在读取器文档注明）：除 Angle 内嵌核心外，负值版本位模式拒绝；
Angle 内嵌核心的版本拒绝路径见下文。LatLon ④ 的内存默认 -1/-1 与
Scientific ② 的内存默认 6 是对象内存
状态，不在流内，读取结果以 `Option::None` 表示。

### 测试（tests/number_format.rs，10 → 19 项）

新增九项：类标识字节互证扩五类；Currency 2 字节体/空输入截断/版本 2 与负
值拒绝；LatLon 版本 4 满形态/版本 3·2·1 分支停止点/版本 5 与负值拒绝/逐
前缀截断事务性；Angle 版本 3·2·1 分支/版本 4 与负值拒绝/截断扫描；
Scientific 版本 2 int32/版本 1 短体（后续字节不消费）/版本 3 与负值拒绝/
截断扫描；CustomNumber 版本 1 含中文格式串/版本 2 与负值拒绝/max_bytes
限额/截断扫描。全部为合成输入（样本 0 实例），通过不代表 MXD 格式已验证。

### 九类分派接入建议（写给整合轮；本不改 src/layers/）

1. `feature_layer_tail` 的 `pub use` 重导出清单追加五常量、五读取器（及
   五个 Body 类型，如 `NumberFormatSlot10_1` 枚举需要）。
2. `NumberFormatSlot10_1` 追加五变体：`CurrencyFormat`/`LatLonFormat`/
   `AngleFormat`/`ScientificFormat`/`CustomNumberFormat`（Box 载体与既有
   四类一致）；`read_object_entry_10_1` 分派处追加五个 match 臂，读取器
   直接作槽位分派目标——语义不变（成功=整体消费、失败=停止点回退）；
   `max_bytes` 沿用 RateFormat 臂的同一限额传参（CustomNumber 用）。
3. `FieldInfoStop10_1::NumberFormatUnsupported` 的文档注释目前列举「其余
   esriSystem 同族类（Currency/LatLon/Angle/Scientific/CustomNumber，体
   未研究）」，接线后应改为九类闭合、列举移除（注释级修改）。
4. robustness 集中清单同步补入五读取器入口。
5. 样本 `samples/测试底图.mxd` 槽仅含 NumericFormat（本轮全流扫描 0 命中
   复核），接线不改变样本行为，可直接以样本回归核对。

### 未解决事项

1. 五类全部标量/String 字段的业务语义未定性（经纬度换算参数、角度格式、
   科学计数参数、自定义格式串等均为候选）。
2. LatLon ④ 的 -1/-1 与 Scientific ② 的 6 已确认是旧版 Load 设置的内存默认值。
   LatLon/Angle 内嵌核心的独立槽由[原生写读往返](#内嵌核心与旧版对象体的原生取证)复核。
3. LatLon/Angle Save 版本表的索引/封顶语义未反编译；`esriArcGISVersion`
   文档版本枚举值与产品版本的对应未确认。
4. Angle 内嵌核心的 IStream 短读和游标传播已补做原生指令取证，见
   [短读取证](#angleformat-短读取证)。读取器明确拒绝截断。
5. v0 短体可由 10.1 Load 接受。历史版本实际写出情况仍缺 MXD 样本。
6. 未运行 ArcMap 10.1，未做目标软件内的人工目视验收。

### AngleFormat 短读取证

从 10.1.0.3035 的 SystemCore.dll 提取 AngleFormat Load `0x7ffb33a0` 和
NumericFormat Load `0x7ffce4b0`，从同版 AfCore.dll 提取两个导入共用的
`AfStreamRead` `0x68a84830`。SystemCore 和 AfCore 的 SHA-256 分别为
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` 和
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`。
按原虚拟地址布置未修改的指令字节，以 QEMU i386 执行。测试 IStream 记录每次
Read 的起始位置、请求长度、实际传输长度和 HRESULT。未加载 ArcMap 应用程序，
也未覆盖 Windows 真实复合文档流。

每个输入从位置 0 开始，外壳版本、外壳 short、内核版本各占 2 字节。
内核首个 LONG 在位置 6 只读到 `55 66`，对象内存预填 `cc`。

| Angle 版本 | IStream 短读与后续字节 | Load 返回 | 结束位置 | 观察 |
| --- | --- | --- | --- | --- |
| 2 | 首个 LONG 短读返回 `0x8003001e` | `S_OK` | 8 | 核心提前返回；只留下部分写入 `55 66 cc cc` |
| 3 | 首个 LONG 同上，之后无字节 | `0x80004005` | 8 | 外层继续尝试读尾 short，尾部 Read 仍返回 `0x8003001e` |
| 3 | 强制首个 LONG 短读并返回 `0x8003001e`，位置 8 尚有 `77 88` | `S_OK` | 10 | 外层从位置 8 读入 `77 88` 作为尾 short |
| 2 | 首个 LONG 及后续 EOF 返回 `S_FALSE` | `S_OK` | 8 | 核心执行余下 6 次读取，均取到 0 字节 |
| 3 | 首个 LONG 及后续 EOF 返回 `S_FALSE` | `S_OK` | 8 | 包含尾 short 在内的后续 7 次读取均取到 0 字节 |

失败返回不会撤销 IStream 已消费的字节。核心因负 HRESULT 提前返回后，
Angle v3 在当前位置读尾 short。原生只判断 HRESULT 的符号，`S_FALSE`
会通过各标量读取门。Rust 对截断仍返回错误并恢复游标，避免将缺少字段的对象
引入后续解析。

### 内嵌核心与旧版对象体的原生取证

沿用短读实验核对过 SHA-256 的 SystemCore.dll 和 AfCore.dll 10.1.0.3035。把
上表四类及 NumericFormat 的 Load/Save、原生版本表、AfCore 的读写转发、
`AfStreamGetSaveVersion`、String::Read/Write 和字符串赋值指令映射到原址，
在 QEMU i386 中运行。测试提供顺序内存 IStream、指定的文档版本索引和固定容量的
字符串缓冲。String 保留原生长度和数据读写过程，底层内存分配及 BSTR 分配由
测试实现。输入是在每类对象的已知标量位置填入的受控值，没有启动 ArcMap、实例化
完整 COM 类或打开 MXD。

从对象体起点计算偏移。Rate 使用 `42.25` 和 `km/h`，字符串含终止零共 10 字节，
所以核心起点是 `2 + 8 + 4 + 10 = 24`。

| 外层对象 | 文档版本索引 | 对象体版本 | 内嵌核心范围 | 对象体长度 |
| --- | ---: | ---: | --- | ---: |
| Percentage | 5 | 2 | `[4, 28)` | 28 |
| Rate | 5 | 2 | `[24, 48)` | 48 |
| LatLon | 0 | 3 | `[10, 34)` | 34 |
| LatLon | 5 | 4 | `[10, 34)` | 36 |
| Angle | 0 | 2 | `[4, 28)` | 28 |
| Angle | 5 | 3 | `[4, 28)` | 30 |

六次原生 Save 和 Load 均返回 `S_OK`。核心依次写读版本 1、四个 LONG 和三个
short，共 24 字节，无额外对象头或键。对比 Load 后的对象内存，四类核心标量全部
恢复，Percentage 的外层 short、Rate 的 double 和字符串也分别恢复。读取跟踪显示
LatLon v4 和 Angle v3 在核心之后各读一个 short。Rust 对每份原生 Save 输出均
恢复七个核心标量，并在相同的对象边界前停止，下一条目的 `de ad` 标记保持可读。

#### 旧版读取分支

另用 19 个手工构造的旧版输入调用九个 10.1 原生 Load 入口。表中长度均包含
两字节版本字段，String 测试值仍为 `km/h`。

| 类 | v0/v1 字节数 | 其他旧版或字段结果 |
| --- | ---: | --- |
| Numeric | 24 | v0/v1 均读全七个字段 |
| Percentage | 4 | 读外层 short，省略核心 |
| Fraction | 10 | v0/v1 均读两个 LONG |
| Rate | 24 | 读 double 和 String，省略核心 |
| Currency | 2 | 只读版本 |
| LatLon | 6 | v0/v1 将两项默认为 -1，v2 读取两项后在第 10 字节结束 |
| Angle | 4 | 读外层 short，省略核心 |
| Scientific | 2 | v0/v1 设置内存默认值 6 |
| CustomNumber | 16 | v0/v1 都在 String 终止零后结束 |

全部原生调用成功，Rust 的读取结束位置相同。以上验证了目标版本读端接受这些
字节形态。实际旧版 MXD 是否曾写出每个分支，以及它们在真实宿主中的装载情况，
仍缺实档证据。

### AngleFormat 内嵌核心版本拒绝

用同一 SystemCore.dll 10.1.0.3035 映像复核原始指令。核心
`0x7ffce4b0` 先在 `0x7ffce4c0` 读取 2 字节版本，
`0x7ffce4d7..0x7ffce4dc` 对有符号版本大于 1 直接返回错误。
调用方 `0x7ffb33f4..0x7ffb340e` 忽略核心返回值，Angle v2 在已消费
的核心版本后结束，v3 从紧随其后的 short 读取 DisplayDegrees。
读取器保留被拒绝的核心版本，并按原生位置消费外层字段。构造测试确认
两种外层版本的对象边界、后续字节及 v3 尾字段截断回滚。ArcMap 运行中
的 IStream 短读传播和真实失败样本尚未验证。

### 接线轮落位（整合接线轮，2026-11-15）

「九类分派接入建议」1-4 已落位（建议 5 的样本回归同步完成）：

- **feature_layer_tail.rs**：`pub use` 重导出清单追加五常量、五读取器与五
  Body 类型；`NumberFormatSlot10_1` 追加 `CurrencyFormat`/`LatLonFormat`/
  `AngleFormat`/`ScientificFormat`/`CustomNumberFormat` 五变体（Box 载体与
  既有四类一致）；`read_field_info_body_10_1` 分派追加五 match 臂，读取器
  直接作槽位分派目标（成功=整体消费、失败=停止点回退语义不变）；
  CustomNumber 臂沿用 Rate 臂的同一 `max_bytes` 限额传参；
  `FieldInfoStop10_1::NumberFormatUnsupported` 注释改为九类闭合后语义
  （「体未研究」列举移除）。
- **测试**：`tests/feature_layer_tail.rs` 新增五变体槽级分派用例（各取合
  法形态、FieldInfo 版本门尾段可达断言）；未知类停止用例以 RgbColor 保持
  覆盖（九类之外的实现类语义不变）。
- **robustness**（建议 4）：五读取器入口并入 `tests/robustness.rs` 集中清
  单。
- **样本回归**（建议 5）：样本槽仅含 NumericFormat（本轮全流扫描 0 命中
  复核），接线后 `parse samples/测试底图.mxd` 停止点不变（`0x155a`）。
  `cargo fmt --check` 与全量 `cargo test` 绿；未声称运行验证。
