# ArcMap 10.1 DisplayCore.dll 简单符号与颜色族：Load 字段序闭合

记录日期：2026-11-15。对应 TODO「MXD 解析」节开放项（docs/arcmap-10.1-
featurelayer-tail.md「未解决事项」）中的「SimpleLine/FillSymbol 等同族、
CmykColor 等颜色族未闭合」——本轮闭合 DisplayCore.dll「简单符号与颜色族」
10 个 coclass 的 Load/Save 字段序（5 个目标类 + 同族 HlsColor 与三个线条
符号），并实现显式 10.1 严格读取器（今位于 `src/objects/display_symbol.rs`、
`src/objects/display_color.rs`；本轮未接入任何既有分派链，接入见下文
「接线轮（2026-11-15）」节）。全部为静态反编译观察，
未运行 ArcMap，未做人工目视验收；**仓库参考样本 `samples/测试底图.mxd`
本轮闭合的 10 类均无实例（样本只含 SimpleMarkerSymbol 与 RgbColor，前轮
已闭合），全部结论无样本字节佐证**，先例见 docs/arcmap-10.1-dataset-name.md。

## 方法与文件身份

- 本地 pyghidra（Ghidra + JDK 25，temp/agent-display-tails/ 内脚本
  dst_step1..14 与 *.c 反编译产物），复用前轮 DisplayCore.dll 工程缓存
  （temp/agent-display-tails/r7flt/ 副本）；目标件 temp/agent-fl-tail/
  DisplayCore.dll.33A02325_…（前轮提取，SHA-256 `9b3e61f9…a926e4d2`，
  pefile 复核 10.1.0.3035）。
- 定位链（对每类独立执行，证据强度：**确定**，全类一致）：
  1. RTTI 类型描述符名串（`.?AV<类名>@@`）→ TD 基址；
  2. 指向 TD 的双字 → 完整对象定位符 COL（判据 `[C]==0`（签名）且
     `[C+0xc]==TD`；**不要求 `[C+8]==0`**——x86 COL 的 cdOffset 对
     IPersistStream 子表为 4，前轮方法漏掉这一类 COL，本轮放宽）；
  3. 指向 COL 的双字 → 子虚表起点；
  4. 槽 +0x0c 函数（含一级 thunk 跟随）反编译，写出目标 CLSID 的两个
     8 字节立即数 → 确认 GetClassID 与 IPersistStream 虚表身份（同轮
     esri.clsid 声明三方互证）；
  5. 槽 +0x14/+0x18 = Load/Save（+0x10 IsDirty、+0x1c GetSizeMax 形状
     旁证）。
- GetClassID thunk 均为「`this − *(int*)(this−4)` 调整器 → 实体」形状；
  实体函数写出 CLSID 立即数（如 CmykColor `0x6eb34a70` 写
  `0x11d0d1237ee9c497` / `0xcc96b90900088383`）。
- Save 版本常量为虚表引用的只读数据（`DAT_6ec0b3xx`），逐项读字节核对。

## 一、颜色族：共享 Color 核心（RgbColor/HsvColor/HlsColor/GrayColor）

四类的 IPersistStream 子对象偏移同为 44（COL 偏移字段，cdOffset=4），Load
槽同为 thunk `0x6eb3dbf0` → **同一实体 Load `0x6eb3d8b0`** → 同一核心
`0x6eb35310`（证据强度：**确定**；流内布局对四类一致，每类对象内的虚槽
只影响字段语义不影响字节数）：

| # | 字段 | 读取原语 | 内存偏移 | 证据 |
| - | ---- | -------- | -------- | ---- |
| ① | 版本 short | 2 字节，`1 < (short)ver` 拒绝 `0xA0000005` | — | 实体 `0x6eb3d8b0` |
| ② | 核心版本 short | 2 字节，同上限 | — | 核心 `0x6eb35310` |
| ③ | 分支标志 | 1 字节 | — | 同上 |
| ④a | flag==0：3 个 double | 3×8 字节 → 虚表 +0x34 装载 | — | 同上；HsvColor `0x6eb36d30`，其余共享 `0x6eb34170`→`0x6eb35130` |
| ④b | flag!=0：4 字节 | 4 字节 → 虚表 +0x1c 装载 | — | rgb `0x6eb3d690`、`RgbToHsv 0x6eb36c10`、`RgbToGray 0x6eb35d30`、`RgbToHls 0x6eb36470` |
| ⑤ | 字节 | 1 字节 | this−8 | 同上 |
| ⑥ | 字节 | 1 字节，装载按 `0xff−b` 取补 | this−7 | 同上 |

- 虚表 +0x34 两实现均做范围校验：第一值 ∈ [−0.0001, 100.0001]、后两值 ∈
  [−120.0001, 120.0001]，越界 `0x80070057`（**确定**；分量语义为候选
  HSV/HLS）；+0x1c 实现把 4 字节当 packed RGB 解释（函数名
  RgbToHsv/RgbToGray/RgbToHls 为 Ghidra 签名，**候选**）。
- Save 对称（**确定**）：各实体 Save 恒写版本 1（HsvColor `0x6eb36f70`
  常量 `DAT_6ec0b330`、GrayColor `0x6eb35e50`/`0b338`、HlsColor
  `0x6eb365d0`/`0b334`、RgbColor `0x6eb3d8f0`/`0b318`）→ 共享核心 Save
  `0x6eb35470`（版本常量 `DAT_6ec0b33c`=1），经虚表 +0x38/+0x20 取值。
- 每类 IPersistStream 虚表：HsvColor `0x6ec0be5c`、GrayColor `0x6ec0bb1c`、
  HlsColor `0x6ec0bcbc`（RgbColor `0x6ec0c70c` 前轮已记）；GetClassID 实体
  HsvColor `0x6eb36fb0`、GrayColor `0x6eb35e90`、HlsColor `0x6eb36610`。
- CLSID：HsvColor `{7EE9C492-D123-11D0-8383-080009B996CC}`、HlsColor
  `{7EE9C493-…}`、GrayColor `{7EE9C495-…}`（esri.clsid 声明 + GetClassID
  立即数互证）。

## 二、CmykColor（版本分支与四类不同）

IPersistStream 虚表 `0x6ec0b82c`（COL 偏移 60，cdOffset=4）；Load thunk
`0x6eb34ee0` → 实体 `0x6eb34850`；Save thunk `0x6eb34f90` → 实体
`0x6eb34970`。CLSID `{7EE9C497-D123-11D0-8383-080009B996CC}`
（GetClassID `0x6eb34a70`）。

| # | 字段 | 原语 | 内存偏移 | 版本门 | 证据 |
| - | ---- | ---- | -------- | ------ | ---- |
| ① | 版本 short | 2 字节，`4 < (short)ver` 拒绝 | — | — | 实体 `0x6eb34850` |
| ② | 字节 | 1 字节 | this−0x10 | ≥2 | 同上 |
| ③ | 标志字节 | 1 字节 | this−0xf | ≥2 | 同上 |
| ④ | String | AfCore String::Read | this−0xc | ≥2 且 ③≠0 | 同上（默认重置为空串证明字段归属） |
| ⑤ | short | 2 字节 | this−0x8 | ≥2 且 ③≠0 | 同上 |
| ⑥ | 4 字节 CMYK | 4 字节 | this−0x14 | ≥3 | 同上 |
| ⑦ | 字节 | 1 字节 | this−0x24 | ≥4（`3 < ver`） | 同上 |
| ⑧ | 字节 | 1 字节，装载取补 `0xff−b` | this−0x23 | ≥4 | 同上 |
| ⑨ | 共享核心段 | 上表 ②–⑥（核心在 `this−0x1c`） | — | ≤2（含 ≤1 直入） | 同上 `FUN_6eb35310(this−0x1c)` |

- Save 恒写版本 **4**（`DAT_6ec0b340`），与版本 4 读分支逐字段对称（含
  ⑧ 取补写）；版本 ≤3 分支在本二进制无写侧来源（**确定**，历史兼容分支）。
- ⑦⑧ 与核心段 ⑤⑥ 同落在 this−0x24/−0x23（版本 4 读侧独立读入同槽），
  字段语义未定（候选：核心标志与补码字节的版本 4 重复表达）。

## 五类颜色的原生读写与 CMYK 旧版核心

2026-11-19。重新核对 DisplayCore.dll x86 10.1.0.3035，SHA-256
`9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。
采用既有反编译、objdump 指令、PE 导入表及原始虚表。通过 pefile 按
PE 地址提取 IPersistStream 表、GetClassID/Load/Save 入口、thunk、核心
及常量，在 `qemu-i386` 运行。读写通过真实 AfCore.dll 10.1.0.3035
的流转发和字符串例程完成（SHA-256
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`）。
探针提供内存 IStream、BSTR 分配和颜色分量转换槽，验证颜色持久化
字节和游标。转换槽记录收到的分量或 packed 值。

| 类和版本 | 输入依据 | 原生完整消费 |
| --- | --- | ---: |
| Rgb、Hsv、Hls、Gray v1 | 四类分别经原生 Save 写三 double，再读回 | 各 31 字节 |
| Rgb、Hsv、Hls、Gray v1 | 受控 packed 分支，Load 经类的原生入口 | 各 11 字节 |
| Cmyk v0、v1 | 受控三 double / packed 共享核心，各版两组 | 各 31 / 11 字节 |
| Cmyk v2 | 无字符串的三 double 核心、含字符串的 packed 核心 | 33 / 23 字节 |
| Cmyk v3 | 无字符串、含字符串 | 8 / 18 字节 |
| Cmyk v4 | 原生 Save 写含字符串的对象体，再读回 | 20 字节 |

旧版 Cmyk Load `0x6eb34850` 已读外层版本，v2 读完可选字符串和
short 后，于 `0x6eb3490b` 直接调用 `0x6eb35310`。共享核心先读
自己的版本，然后读分支字节。旧 Rust 入口把独立颜色类的外层版本也
计入内嵌核心，经原生探针构造的六份 v0–v2 体均解析失败。修正后只读
核心自身版本，`CoreColorBody10_1.version_outer` 沿用外层 Cmyk 版本。
独立颜色类仍读取各自外层版本及核心版本。Rust 对 17 组数据经 IColor
分派均完整消费并保留紧随其后的哨兵字节，回归用例另检验紧接空对象
和截断时的游标、键回滚。

探针代码和结果位于忽略目录 `temp/mxd-display-color-native/`，
`result.bin` SHA-256 为
`5bb67931d3a50db33d19970a7bb53f99a31b368a6acd3101ab4aa45d945e3c3f`。
受控运行验证了颜色类和持久化格式，未验证颜色转换数值、
带这些分支的 ArcMap 工程及 QGIS 渲染效果。

## 三、SimpleLineSymbol

CLSID `{7914E5F9-C892-11D0-8BB6-080009EE4E41}`；IPersistStream 虚表
`0x6ec15498`（COL 偏移 212，cdOffset=4）；Load thunk `0x6ebbda20` → 实体
`0x6ebbced0`；Save `0x6ebbd9f0` → `0x6ebbcfc0`；GetClassID `0x6ebbd090`
（写 CLSID 立即数，esri.clsid 声明互证）。

| # | 字段 | 原语 | 内存偏移 | 证据 |
| - | ---- | ---- | -------- | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 `0x80004005` | — | 实体 `0x6ebbced0` |
| ② | LoadObject（IColor `{20CD40B0-…}`，`DAT_6ec0b4f4`） | 20 字节头 + 体 | this−0x2c | 同上 |
| — | `FUN_6eb406f0` 返回值 | 不读流（前轮已证） | this−0xc（bool） | 同上 |
| ③ | double（宽度候选） | 8 字节 | this−0x24 | 同上 |
| ④ | int32（样式候选） | 4 字节 | this−0x28 | 同上 |
| ⑤⑥ | int32 ×2 | 4×2 字节 | this−0x7c / this−0x74 | 同上 |

- Save 恒写版本 1（`DAT_6ec0b250`）+ `SaveObject(this−0x2c)` + ③④⑤⑥，
  读写对称（**确定**）。

## 四、SimpleFillSymbol

CLSID `{7914E603-C892-11D0-8BB6-080009EE4E41}`；虚表 `0x6ec152ac`（偏移
212）；Load `0x6ebbc650` → `0x6ebbb4e0`；Save `0x6ebbc6c0` → `0x6ebbb5e0`；
GetClassID `0x6ebbb6a0`。

| # | 字段 | 原语 | 内存偏移 | 证据 |
| - | ---- | ---- | -------- | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | 实体 `0x6ebbb4e0` |
| ② | LoadObject（ISymbol `{F3435802-…}`，`DAT_6ec0ca2c`；轮廓线符号候选） | 20 字节头 + 体 | this−0x24（Save 槽） | 同上；Load 目标被 `FUN_6eb32c80` 遮蔽，槽归属以 Save `SaveObject(−0x24)` 序印证 |
| ③ | LoadObject（IColor `DAT_6ec0b4f4`；填充色候选） | 20 字节头 + 体 | this−0x28（Save 槽，同上印证） | 同上 |
| — | `FUN_6eb406f0` | 不读流 | this−0x8（bool） | 同上 |
| ④ | int32（样式候选） | 4 字节 | this−0x7c | 同上 |
| ⑤⑥ | int32 ×2 | 4×2 字节 | this−0x74 / this−0xc | 同上 |

- Save 恒写版本 1（`DAT_6ec0b254`）+ SaveObject ×2 + int32 ×3，对称
  （**确定**）。

## 五、MarkerLineSymbol

CLSID `{7914E5FD-C892-11D0-8BB6-080009EE4E41}`；虚表 `0x6ec0eca0`（偏移
8588）；Load `0x6eb636a0` → `0x6eb62810`；Save `0x6eb63630` → `0x6eb629e0`；
GetClassID `0x6eb61410`。

| # | 字段 | 原语 | 内存偏移 | 版本门 | 证据 |
| - | ---- | ---- | -------- | ------ | ---- |
| ① | 版本 short | 2 字节，`2 < ver` 拒绝 | — | — | 实体 `0x6eb62810` |
| ② | byte | 1 字节 | this−0x20d8 | — | 同上 |
| ③ | double | 8 字节 | this−0x20d4 | — | 同上 |
| ④ | LoadObject（IMarkerSymbol `{E6BDAA7C-…}`，`DAT_6ec0ca3c`；接口名经 docs/arcmap-10.1-map-object.md IID 表） | 20 字节头 + 体 | this−0x20e0（Save 槽） | — | 同上 |
| — | `FUN_6eb68fc0` | 不读流（反编译无 AfStreamRead，仅释放/重建辅助） | — | — | 同上 |
| ⑤ | LoadObject（`{7914E5F4-…}`，ITemplate 候选） | 20 字节头 + 体 | this−0x20cc（Save 槽） | — | 同上 |
| ⑥ | LoadObject（`{533D88F0-…}`，ILineDecoration 候选） | 20 字节头 + 体 | this−0x20c8（Save 槽） | — | 同上 |
| ⑦⑧ | int32 ×2 | 4×2 字节 | this−0x2134 / this−0x212c | — | 同上 |
| ⑨ | byte | 1 字节 | this−0x20ac | — | 同上 |
| ⑩ | double | 8 字节 | this−0x20a4 | — | 同上 |
| ⑪⑫⑬ | int32 ×2 + double | 4+4+8 字节 | this−0x2094 / −0x2090 / −0x208c | ≥2 | 同上 |

- Save 恒写版本 **2**（`DAT_6ec0b2e0`，含 ⑪–⑬），对称（**确定**）。

## 六、CartographicLineSymbol

CLSID `{7914E5FB-C892-11D0-8BB6-080009EE4E41}`；虚表 `0x6ec11ca8`（偏移
4436）；Load `0x6eb99e80` → `0x6eb99140`；Save `0x6eb99e00` → `0x6eb99330`；
GetClassID `0x6eb99100`。

| # | 字段 | 原语 | 内存偏移 | 证据 |
| - | ---- | ---- | -------- | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | 实体 `0x6eb99140` |
| ②③ | int32 ×2 | 4×2 字节 | this−0x10a8 / −0x10a4 | 同上 |
| ④ | double | 8 字节 | this−0x109c | 同上 |
| ⑤ | byte | 1 字节 | this−0x1094 | 同上 |
| ⑥ | double | 8 字节 | this−0x108c | 同上 |
| ⑦ | LoadObject（IColor） | 20 字节头 + 体 | this−0x10ac（Save 槽） | 同上 |
| — | `FUN_6eb406f0` | 不读流 | this−0x104c（bool） | 同上 |
| ⑧⑨ | LoadObject（ITemplate 候选 / ILineDecoration 候选） | 20 字节头 + 体 | this−0x1084 / −0x107c（Save 槽） | 同上 |
| ⑩⑪ | int32 ×2 | 4×2 字节 | this−0x10fc / −0x10f4 | 同上 |
| ⑫ | byte | 1 字节 | this−0x1060 | 同上 |
| ⑬⑭ | double ×2 | 8×2 字节 | this−0x105c / −0x1054 | 同上 |

- Save 恒写版本 1（`DAT_6ec0b27c`），对称（**确定**）。

## 七、HashLineSymbol

CLSID `{7914E5FC-C892-11D0-8BB6-080009EE4E41}`；虚表 `0x6ec0e0b8`（偏移
8660）；Load `0x6eb56f30` → `0x6eb554a0`；Save `0x6eb56e90` → `0x6eb556d0`；
GetClassID `0x6eb53c30`。

| # | 字段 | 原语 | 内存偏移 | 证据 |
| - | ---- | ---- | -------- | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | 实体 `0x6eb554a0` |
| ② | double | 8 字节 | this−0x2124 | 同上 |
| ③④ | int32 ×2 | 4×2 字节 | this−0x2118 / −0x2114 | 同上 |
| ⑤ | double | 8 字节 | this−0x2104 | 同上 |
| ⑥ | byte | 1 字节 | this−0x20fc | 同上 |
| ⑦ | double | 8 字节 | this−0x20f4 | 同上 |
| ⑧ | LoadObject（ISymbol `{F3435802-…}`，`DAT_6ec0ca2c`） | 20 字节头 + 体 | this−0x20e4（Save 槽） | 同上 |
| ⑨ | LoadObject（IColor） | 20 字节头 + 体 | this−0x211c（Save 槽） | 同上 |
| — | `FUN_6eb406f0` | 不读流 | this−0x2094（bool） | 同上 |
| ⑩⑪ | LoadObject（ITemplate 候选 / ILineDecoration 候选） | 20 字节头 + 体 | this−0x20ec / −0x20e8（Save 槽） | 同上 |
| ⑫⑬ | int32 ×2 | 4×2 字节 | this−0x217c / −0x2174 | 同上 |
| ⑭ | byte | 1 字节 | this−0x20a4 | 同上 |
| ⑮⑯ | double ×2 | 8×2 字节 | this−0x209c / −0x210c | 同上 |

- Save 恒写版本 1（`DAT_6ec0b2f4`），SaveObject 序（−0x20e4/−0x211c/
  −0x20ec/−0x20e8）与 Load 四条目对位（**确定**）。

## 实现与测试

- `src/objects/display_color.rs`（接线轮自 src 根级移入）：共享核心读取器
  `read_core_color_body_10_1`、`read_cmyk_color_body_10_1`（版本 0..=4 全
  分支如实覆盖）与五类颜色槽位分派 `read_color_slot_10_1`；RgbColor 体读
  取器 `read_rgb_color_body_10_1` 接线轮自 `feature_layer_tail` 移入本模块
  （消除 objects→layers 依赖倒置）。
- `src/objects/display_symbol.rs`（接线轮自 src 根级移入）：五类符号体读
  取器与符号槽位分派 `read_display_symbol_slot_10_1`（嵌套深度限额 16 为
  防栈溢出的读取器自身限额，非原生限制）；SimpleMarkerSymbol 体读取器接
  线轮自 `feature_layer_tail` 移入本模块。ITemplate/ILineDecoration 候选槽
  新对象（coclass 体未闭合）为停止点并回退到槽位起点。
- 版本 short 一律按原生有符号比较设上限并拒绝负值位模式（较原生更严，
  与既有读取器一致）；未知字节原样保留（含取补字节的流内位模式）、截断
  与非法长度明确报错、失败游标与登记表不推进（trial 副本整体提交）。
- 测试：tests/display_color.rs 8 项、tests/display_symbol.rs 12 项；集中
  健壮性清单 tests/robustness.rs 增至 **37 个入口**（新增 9 个：共享颜色
  体、CmykColor 体、颜色槽位、五个符号体、符号槽位），覆盖逐字节截断、
  失败游标不变与重读一致、限额与版本拒绝。`cargo fmt --check` 与
  `cargo test` 全过（27 个测试目标，338 项 + 1 项默认忽略）。
- **本轮未接入** `parse` 命令、中间模型与任何既有分派链；`feature_layer_tail.rs`
  本轮未改动（接线轮状态见下节）。

## 接线轮（2026-11-15）

研究结论无改动；本节只记录接入点落位与目录修正。

- 目录分层修正：两模块自 `src/` 根级移入 `src/objects/`；`read_rgb_color_body_10_1`
  及 `RGB_COLOR_CLASS_ID` 自 `feature_layer_tail.rs` 移入
  `src/objects/display_color.rs`，SimpleMarkerSymbol 体读取器及其类标识同
  步移入 `src/objects/display_symbol.rs`（否则移入 objects 后仍存在
  objects→layers 依赖倒置）——原「发现的既有问题」归属文件随之变为
  `src/objects/display_symbol.rs`（该 key 阴影问题已由提交 954794d 修复）。
  两常量在 `feature_layer_tail` 保留 `pub use` 重导出，调用方与测试同步更
  新到新路径。
- 按本记录「建议接入点」接线：`feature_layer_tail` 的 SimpleRenderer 与
  LegendClass 的 ISymbol 槽改调 `read_display_symbol_slot_10_1`（未知符号
  类维持停止回退语义，新增 `SymbolNested` 停止变体透传嵌套停止）；SMS 双
  IColor 槽、后继字段段 IColor 槽改调 `read_color_slot_10_1`（五类闭合颜
  色整体解析，未知类维持 `ColorUnsupported`/`ColorUnsupported` 停止）；
  TextSymbol 的 ISymbol #2 槽改调符号分派（样本 {7914E603} = SimpleFillSymbol，
  体解析后 TextSymbol 推进到 IFontDisp 槽）。SMS 体读取器签名新增
  `max_bytes` 参数（CmykColor 串分支限额转发）。
- robustness 集中清单 37 → **48 个入口**（新增 RgbColor 体、SMS 体、
  TextSymbol 体三个直读入口）。样本解析停止点不变（0xafe，见
  featurelayer-tail 接线轮）。

## 建议接入点

- `feature_layer_tail::read_simple_renderer_body_10_1` 的 ISymbol 槽
  （`SymbolUnsupported` 停止点）与 `read_legend_class_body_10_1` 的 ISymbol
  槽（`LegendClassStop10_1::SymbolUnsupported`）可改调
  `display_symbol::read_display_symbol_slot_10_1`；
- `feature_layer_tail::read_simple_marker_symbol_body_10_1` 的两个 IColor
  槽（`ColorUnsupported`/`OutlineColorUnsupported` 停止点）可改调
  `display_color::read_color_slot_10_1`；
- 渲染器体内其余符号槽/颜色槽（未来闭合的渲染器）同样经这两个分派入口。

## 发现的既有问题（归属 feature_layer_tail.rs 所有者）

- `read_simple_marker_symbol_body_10_1` 首色槽 `ColorUnsupported` 停止路径
  中，`SimpleMarkerSymbolPrefix10_1.key` 被匹配臂解构的 **颜色条目键** 阴影
  （应为 SMS 自身键）；本模块嵌套测试（tests/display_symbol.rs
  `sms_stop_propagates_as_nested_incomplete`）暴露。该文件本轮不动，修复
  建议由其所有者执行（停止路径改用独立命名的解构变量）。

## 未解决事项（开放项）

- **无字节佐证**：本轮 10 类全部无样本实例；读取器仅可用合成输入验证，
  不得声称样本验证。
- ITemplate `{7914E5F4}` / ILineDecoration `{533D88F0}` 接口名未定名（本
  机无 esriDisplay 类型库，仅按 esri.clsid 同族 coclass Template
  `{41093A71}`、LineDecoration `{533D88F5}` 与符号属性面推断）；两 coclass
  的对象体未闭合，槽内新对象一律停止。
- 颜色族语义候选：共享核心两个版本 short 的独立含义、3 个 double 的分量
  归属（HSV/HLS）、4 字节分支的 packed RGB 位序与 bit25 可见性标志、
  `this−8`/`this−7` 两字节语义、CmykColor 版本 ≥2 的 byte/String/short
  字段语义、⑦⑧ 与核心尾字节同槽的关系。
- 符号族语义候选：各 int32/double/byte 业务命名（宽度/样式按符号学惯例
  推断）；`FUN_6eb406f0`/`FUN_6eb68fc0` 副作用仅确认不读流；三个 Save 的
  `if (param_3 != 0) { *(chain) = 0 }` 写入链目标未解析（不影响流字节）。
- 同族未闭合符号（新对象仍为停止点）：MultiLayerLineSymbol
  `{7914E5FA}`、MultiLayerMarkerSymbol `{7914E5FF}`、CharacterMarkerSymbol
  `{7914E600}`、PictureMarkerSymbol `{7914E602}`、ArrowMarkerSymbol
  `{88539431}`、PictureLineSymbol `{22C8C5A1}`、LineFillSymbol
  `{7914E606}`、MarkerFillSymbol `{7914E608}`、GradientFillSymbol
  `{7914E609}`、DotDensityFillSymbol `{9A1EBA10}`、TextSymbol 及标注/
  Callout 族等（esri.clsid DisplayCore 全表 73 类中其余条目）。
- 全部结论为静态反编译观察，未经 ArcMap 10.1 运行验证；样本对位无从谈起
  （无实例），人工目视验收未做。
