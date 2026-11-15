# ArcMap 10.1 DisplayCore.dll 符号族第二轮：11 类符号 + Template/LineDecoration
# 族闭合与 TextSymbol 虚表归属复核

记录日期：2026-11-15。对应上一轮记录（docs/arcmap-10.1-display-tails.md）的
「未解决事项」：同族未闭合符号 10 类、ITemplate `{7914E5F4}` /
ILineDecoration `{533D88F0}` 未定名与 Template `{41093A71}` /
LineDecoration `{533D88F5}` 对象体未闭合；以及
docs/arcmap-10.1-element-bodies.md「未解决事项 1」（TextSymbol
IPersistStream 虚表归属）。

本轮闭合：MultiLayerLineSymbol/MultiLayerMarkerSymbol/MultiLayerFillSymbol/
CharacterMarkerSymbol/PictureMarkerSymbol/PictureLineSymbol/ArrowMarkerSymbol/
LineFillSymbol/MarkerFillSymbol/GradientFillSymbol/DotDensityFillSymbol 共
**11 个符号 coclass**（上一轮「11 类」口径含 SimpleTextPath 之外的全部同族
成员）与 **Template/LineDecoration/SimpleLineDecorationElement 三个装饰
coclass**；接口名经 esriDisplay 类型库 dump 定名复核；MarkerLine/
CartographicLine/HashLine 三类的 ITemplate/ILineDecoration 槽停止点改为按
类分派（类匹配 → 解析体，未知实现类维持停止）；TextSymbol 真虚表定位闭合，
**既有结论 Load 实体 `0x6ebd0a60` 获虚表槽位级互证**（element_body.rs 的
读取器无需改动——虚表归属是证据修正而非代码变更）。

## 方法与文件身份

- 本机 pyghidra（Ghidra + JDK 25，temp/agent-display2/ 内脚本 d2_step1..5
  与 *.c/*.asm 反编译产物），复用前轮 DisplayCore.dll 工程缓存副本（把主
  检出的 temp/agent-display-tails/r7flt/dc 复制到本机临时目录打开，独立
  锁，活工程路径不入库）；目标件与前轮同一提取：DisplayCore.dll 10.1.0.3035
  （SHA-256 `9b3e61f9…a926e4d2`）。- 接口定名：esriDisplay 类型库 dump（dump2_esriDisplay.olb…big.tlb.txt），
  逐 IID 比对；esri.clsid 全表复核 coclass 归属（**Template
  `{41093A71-CCE1-11D0-BFAA-0080C7E24280}`、LineDecoration
  `{533D88F5-0A1A-11D2-B27F-0000F878229E}`、SimpleLineDecorationElement
  `{533D88F3-…}` 均在 DisplayCore.dll**，无需另提取 DLL）。
- 定位链（对每类独立执行，证据强度：**确定**，全类一致）：RTTI 类型描述符
  名串（`.?AV<类名>@@`）→ TD 基址 → 指向 TD 的双字 → COL（判据 `[C]==0`
  且 `[C+0xc]==TD`，放宽判据沿上一轮）→ 指向 COL 的双字 → IPersistStream
  子虚表 → 槽 +0x0c GetClassID（一级 thunk 跟随）反编译 → **CLSID 立即数
  互证**（两个 8 字节/四个 4 字节立即数模式任二命中；esri.clsid 声明三方
  互证）→ 槽 +0x14/+0x18 = Load/Save（+0x10 IsDirty、+0x1c GetSizeMax 形
  状旁证）。本轮 15 类全部经该链唯一确认。
- Save 版本常量：直接常量逐字节读核（Template `DAT_6ec0b280`=1、PicLine
  `0b2a8`=1、LineFill `0b2e8`=1、MarkerFill `0b2e4`=1、GradientFill
  `0b2f8`=1、DotDensity `0b2fc`=1、ArrowMarker `0b314`=2、LineDecoration
  `0b2ec`=1、SimpleLineDecorationElement `0b290`=1）；`AfStreamGetSaveVersion(
  docVer, N, table)` 版本表按短整表逐项读核（MultiLayerLine `DAT_6ec0b2c4`
  docVer 0..4 → 1,1,1,1,2；MultiLayerMarker `0b2b8` → 2,2,2,2,3；
  MultiLayerFill `0b2d0` → 1,1,1,1,2；PictureMarker `0b29c` → 4,4,4,5,8,9；
  CharacterMarker `0b270` → …,4）。**表索引语义在 AfCore（未提取），表内
  各项为直接读数；10.1 写侧取值按 Load 侧上限与表尾项推断为候选**。读取器
  仅实现 Load 侧，不受此影响。
- 样本对位：**本轮全部 15 类在仓库参考样本 `samples/测试底图.mxd` 中 0 实
  例（无字节佐证）**；读取器仅以合成输入验证，不得声称样本验证。先例见
  docs/arcmap-10.1-dataset-name.md 与上一轮。

## 一、TextSymbol IPersistStream 虚表归属复核（确定；闭合上一轮开放项）

上一轮经「GetClassID 的 .rdata 数据引用 = 虚表 GetClassID 槽」定位到
`0x6ec16334`，其 Load 槽值 `0x6ec22e60` 实为下一 COL 地址（5 槽形状误读）。
本轮以放宽 COL 判据反查：TextSymbol TD（`0x6ec3e5e0`）名串
`.?AVTextSymbol@@` 名 `0x6ec3e5e8`；指向 TD 的双字共 24 处满足
`[C]==0 且 [C+0xc]==TD`（COL `0x6ec22cbc`…`0x6ec22e60`、`0x6ec23168`），
逐 COL 反查虚表候选 23 处；其中 **`0x6ec166e8`**（COL `0x6ec22e60`）的
GetClassID 槽 `0x6ebe5bf0`（thunk）→ 实体 `0x6ebd08e0` **写出 TextSymbol
CLSID `{B65A3E74-2993-11D1-9A43-0080C7EC5C96}` 立即数**
（`0x965cecc7`/`0x965cecc78000439a`），Load 槽 `0x6ebe5bd0`（thunk）→
**`0x6ebd0a60`**（thunk 体 ≤0x10 字节单转发，实体即上一轮由字段序-样本
互证确认的实现），Save 槽 `0x6ebe5db0` → `0x6ebd0330`。三方（COL 判据、
GetClassID 立即数、Load 实体与样本对位）互证闭环。

- **结论**：TextSymbol 真 IPersistStream 子虚表为 `0x6ec166e8`；上一轮的
  `0x6ec16334` 区域为同区七槽主虚表族/两槽空桩族的成员（其 GetClassID 槽
  读常量 `0x6ec167a8` 的实现 `0x6ebc9540` 面向外部查询，不承担持久化），
  其「Load 槽」实为相邻 COL 指针——5 槽形状误读按本轮判据排除。
- **element_body.rs 的 TextSymbol 读取器（Load 实体 `0x6ebd0a60`）无需改
  动**：本轮为证据形态补全（虚表槽位级互证），非代码变更。
- **版本门尾段（字段 15/16）反编译证据成型**（`0x6ebd0a60` 重反编译 +
  常量核对；样本不可达无字节佐证，证据强度：**确定（反编译）/无样本**）：
  - 版本 >1：short（2 字节，→ this−0x4e4）；
  - 版本 ≥3：16 字节原样 GUID，与零 GUID（常量 `DAT_6ec0b0cc`）比较，
    默认路径取内置 `{D7099B91-E298-499E-9757-72BB39E55CF3}`（常量
    `DAT_6ec15a94`，逐字节核对一致）；
  - 版本 ≥4：short（2 字节，→ this−0x384）。
  IFontDisp 槽（请求 IID `{BEF6E002-A874-101A-8BBA-00AA00300CAB}`，常量
  `DAT_6ec0b07c`）位于 ISymbol #2 之后、版本门尾段之前，与本轮反编译一致。

## 二、装饰三件：Template / LineDecoration / SimpleLineDecorationElement
（证据强度：确定；样本 0 实例）

| 类 | CLSID | 虚表 | GetClassID | Load | Save | Save 版本 |
| --- | --- | --- | --- | --- | --- | --- |
| Template | `{41093A71-CCE1-11D0-BFAA-0080C7E24280}` | `0x6ec1114c` | `0x6eb87aa0` | `0x6eb87ad0` | `0x6eb87bb0` | 恒 1（`DAT_6ec0b280`） |
| LineDecoration | `{533D88F5-0A1A-11D2-B27F-0000F878229E}` | `0x6ec0e4e4` | `0x6eb5c0a0` | `0x6eb5c640` | `0x6eb5c740` | 恒 1（`DAT_6ec0b2ec`） |
| SimpleLineDecorationElement | `{533D88F3-0A1A-11D2-B27F-0000F878229E}` | `0x6ec10d54` | `0x6eb840e0` | `0x6eb84120` | `0x6eb84270` | 恒 1（`DAT_6ec0b290`） |

- **Template**：short 版本（`1 < ver` 拒绝 `0x80004005`）→ double（→
  this−0x94，间隔候选）→ int32 count → count × 16 字节（每标记 2 个
  double：位置 + 角度候选；Save 槽 this−0x88 数组，逐项 16 字节对称写）。
  Save 侧 `if (param_3 != 0)` 写链与上一轮同型（写 `this−0x9c` 链一字节，
  不影响流内字节，目标未解析）。
- **LineDecoration**：short 版本（`1 < ver` 拒绝）→ int32 count →
  count × LoadObject（请求 IID `DAT_6ec0e50c` =
  **ILineDecorationElement `{533D88F2-0A1A-11D2-B27F-0000F878229E}`**，经
  类型库定名 → 集合 this−0x1c；元素接口 +0xc Add/+0x14 Clear/+0x1c 取/
  +0x20 Count 与 Save 对称）。实现类族中仅 SimpleLineDecorationElement
  体本轮闭合；其余 ILineDecorationElement 实现类 = 停止点。
- **SimpleLineDecorationElement**：short 版本（`1 < ver` 拒绝）→ byte ×3
  （→ this−0x18/−0x17/−0x16，语义未定）→ short（→ this−0x14，语义未定）→
  LoadObject（请求 IID `DAT_6ec0ca2c` = ISymbol `{F3435802-…}` → Save 槽
  this−0x24；标记符号候选）→ int32 count → count × double（→ this−0x40
  集合 +0xc Add/+0x14 Clear/+0x20 Count/+0x24 取，位置候选）。
- **槽接线**：MarkerLine/CartographicLine/HashLine 三类的
  ITemplate/ILineDecoration 槽由「未闭合停止」改为 `read_decoration_slot_
  10_1` 按类分派：空标记与回引照旧消费；新对象类标识为 Template/
  LineDecoration 时解析其体（Template 体无嵌套；LineDecoration → 元素 →
  符号槽逐层下钻，嵌套深度限额同符号槽）；其余新实现类维持停止（游标回
  退槽位起点、键不保留）。

## 三、11 个符号 coclass 字段序（证据强度：确定；样本 0 实例）

「helper」= `FUN_6eb406f0`（Simple 族共享）或 `FUN_6eb68fc0`（层/标记数
组重建），反编译确认无 AfStreamRead、无流内字节。槽归属多数被 Ghidra 遮
蔽（`FUN_6eb32c80`），以 Save 侧 `SaveObject(槽)` 序对称印证；count 为流
内 int32，读取器以 max_bytes 设循环上限（读取器自身限额，非原生限制）。

### 1. MultiLayerLineSymbol `{7914E5FA-C892-11D0-8BB6-080009EE4E41}`

虚表 `0x6ec0f410`；Load `0x6eb68280`；Save `0x6eb68610`；GetClassID
`0x6eb67350`。

| # | 字段 | 原语 | 内存偏移/去向 | 版本门 | 证据 |
| - | ---- | ---- | ------------ | ------ | ---- |
| ① | 版本 short | 2 字节，`2 < ver` 拒绝 | — | — | Load `0x6eb68280` |
| ②③ | int32 ×2 | 4×2 字节 | this−0xc4 / −0xbc | — | 同上 |
| ④ | int32 count | 4 字节 | — | — | 同上 |
| ⑤ | count × LoadObject（ILineSymbol `{E6BDAA7D-…}`，`DAT_6ec0d334`） | 20 字节头 + 体 | Save 槽 this−0x74 数组 | — | 同上（Save 倒序写出） |
| ⑥⑦ | count × int32 ×2 组 | 4 字节 ×2 组 | Save 槽 this−0x60 / −0x4c 数组 | — | 同上 |
| ⑧ | count × String | AfCore String::Read | Save 向量 this−0x38 | ≥2；版本 <2 无流内串（内存补空串） | 同上 |

### 2. MultiLayerMarkerSymbol `{7914E5FF-…}`

虚表 `0x6ec0f754`；Load `0x6eb6b010`；Save `0x6eb6b3d0`；GetClassID
`0x6eb691c0`。

| # | 字段 | 原语 | 内存偏移/去向 | 版本门 | 证据 |
| - | ---- | ---- | ------------ | ------ | ---- |
| ① | 版本 short | 2 字节，`3 < ver` 拒绝 | — | — | Load `0x6eb6b010` |
| ②③ | int32 ×2 | 4×2 字节 | this−0x10c / −0x104 | — | 同上 |
| ④-⑦ | double ×4 | 8×4 字节 | this−0x5c / −0x44 / −0x54 / −0x4c | — | 同上 |
| ⑧ | LoadObject（IColor `DAT_6ec0b4f4`） | 20 头 + 体 | Save 槽 this−0x3c | — | 同上 |
| ⑨⑩ | int32 + double | 4+8 字节 | this−0x34 / −0x24 | — | 同上 |
| ⑪ | LoadObject（ISymbol `DAT_6ec0ca2c`） | 20 头 + 体 | Save 槽 this−0x30 | — | 同上 |
| ⑫ | int32 count | 4 字节 | this−0xb8 | — | 同上 |
| ⑬ | count × LoadObject（IMarkerSymbol `DAT_6ec0ca3c`） | 20 头 + 体 | Save 槽 this−0xbc 数组 | — | 同上（Save 倒序） |
| ⑭⑮ | count × int32 ×2 组 | 4×count ×2 | Save 槽 this−0xa8 / −0x94 数组 | — | 同上 |
| ⑯ | LoadObject（**IMarkerBackground `{AE41E4F0-DDE7-11D3-8216-0080C79F0371}`**，`DAT_6ec0d6d4`，类型库定名） | 20 头 + 体 | Save 槽 this−0xc | ≥2 | 同上 |
| ⑰ | count × String | String::Read | Save 向量 this−0x7c | ≥3；版本 <3 无流内串 | 同上 |

### 3. MultiLayerFillSymbol `{7914E604-…}`

虚表 `0x6ec0f0fc`；Load `0x6eb65af0`；Save `0x6eb65e60`；GetClassID
`0x6eb64d00`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`2 < ver` 拒绝 | — | — | Load `0x6eb65af0` |
| ②③ | int32 ×2 | 4×2 字节 | this−0xb4 / −0xac | — | 同上 |
| ④ | LoadObject（IColor） | 20 头 + 体 | Save 槽 this−0xc | — | 同上 |
| ⑤ | int32 count | 4 字节 | this−0x60 | — | 同上 |
| ⑥ | count × LoadObject（IFillSymbol `{E6BDAA7E-…}`，`DAT_6ec0ce8c`） | 20 头 + 体 | Save 槽 this−0x64 数组 | — | 同上（Save 倒序） |
| ⑦⑧ | count × int32 ×2 组 | — | Save 槽 this−0x50 / −0x3c 数组 | — | 同上 |
| ⑨ | count × String | String::Read | Save 向量 this−0x24 | ≥2；版本 <2 无流内串 | 同上 |

### 4. CharacterMarkerSymbol `{7914E600-…}`

虚表 `0x6ec11fa0`；Load `0x6eb9b150`；Save `0x6eb9b5b0`；GetClassID
`0x6eb9b9b0`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`4 < ver` 拒绝 | — | — | Load `0x6eb9b150` |
| ② | LoadObject（IColor） | 20 头 + 体 | this−0xc4 | — | 同上 |
| ③ | `FUN_6eb406f0` | 不读流 | bool this−0x54 | — | 同上 |
| ④ | int32 | 4 字节 | this−0xc0 | — | 同上 |
| ⑤-⑩ | double ×6 | 8×6 字节 | this−0xb4 / −0xac / −0xa4 / −0x9c / −0x94 / −0x8c | — | 同上（⑤ 装载后虚表 +0x10 回调） |
| ⑪ | LoadObject（**IFontDisp `{BEF6E002-A874-101A-8BBA-00AA00300CAB}`**，`DAT_6ec0b07c`，OLE 标准接口） | 20 头 + 体 | Save 槽 this−0xbc | <3 | 同上（Save 在 docVer<3 分支写同槽） |
| ⑫⑬ | int32 ×2 | 4×2 字节 | this−0x124 / −0x11c | — | 同上 |
| ⑭ | short | 2 字节 | this−0x10c | >1 | 同上；**读取后内存槽被 `Symbol::GetDefaultRotateWithTransformValue` 覆盖，流内值读取即弃**（Save 写槽内默认值；反汇编 `0x6eb9b…` 与 ArrowMarker `0x6eb3edde..0x6eb3edf0` 同型核实） |
| ⑮ | String + int32 ×4 + short ×2 + int32 ×2 | String::Read + 4+2+2+4+2+4+4 字节 | 字体描述符（Save 由 this−0xbc 字体对象逐属性回写：名 BSTR + vt+0x1c/0x24/0x2c/0x34 四 int32 + vt+0x3c/0x44 两 short + vt+0x14 两 int32） | ≥3 | 同上 |
| ⑯ | LoadObject（IFontDisp 同上常量） | 20 头 + 体 | Save 恒写同槽 this−0xbc 重写 | >3 | 同上 |

### 5. PictureMarkerSymbol `{7914E602-…}`

虚表 `0x6ec10738`；Load `0x6eb7a650`；Save `0x6eb7ab00`；GetClassID
`0x6eb77da0`。版本 ≤8 历史路径 `FUN_6eb7d750`（Save `FUN_6eb7d3c0`）。

**版本 ==9**：

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`9 < ver` 拒绝 | — | — | Load `0x6eb7a650` |
| ② | int32 图像类型 | 4 字节 | this−0xc | ==9；越界（非 1/2/3）拒绝 `0x80004005` | 同上 |
| ③ | int32 | 4 字节 | this−0x10 | ==9 | 同上 |
| ④ | 类型分支 | — | — | ==9 | 同上 |
| ④a | 类型 1：String + double | String::Read + 8 字节 | 路径候选 this−0x18 + this−0xac | — | 同上 |
| ④b | 类型 2：int32 N +（N≠0 时 N 字节） | AfStreamRead ×2 | 全局内存块（Save `Ordinal_23` 取尺寸对称写） | — | 同上；**反汇编核实尺寸来源**：`0x6eb7a7af` 读入 N、`0x6eb7a7b5` 判零跳过、`0x6eb7a7de` 以 N 为长度读块 |
| ④c | 类型 3：LoadObject（**IPicture `{7BF80980-BF32-101A-8BBB-00AA00300CAB}`**，`DAT_6ec0b08c`，OLE 标准接口） | 20 头 + 体 | this−0x88 | — | 同上 |
| ⑤ | LoadObject（IColor）×3 | 20 头 + 体 ×3 | Save 三连 SaveObject | — | 同上 |
| ⑥ | double ×6 | 8×6 字节 | this−0xdc / −0xc4 / −0xbc / −0xb4 / −0xd4 / −0xcc | — | 同上 |
| ⑦ | int32 ×2 | 4×2 字节 | this−0x14c / −0x144 | — | 同上 |
| ⑧⑨⑩ | byte + short + short ×3 | 1+2+2×3 字节 | this−0x48 / −0x134（读取即弃，同上 quirk）/ −0x20 / −0x1e / −0x1c | ==9 恒有 | 同上 |

**版本 ≤8（历史路径 `FUN_6eb7d750`，obj 基址 = this−0x1a4）**：版本 ≤6 时
（版本 <4 LoadObject（IPicture）→ 槽 +0x118）LoadObject（IPicture）→ 槽
+0x11c；版本 ≥7 String → 槽 +0x18c（串空 → LoadObject（IPicture）/ 串非
空 → double → +0xf8）→ LoadObject（IColor）×3（+0x120 / +0x124 / 遮蔽）→
double ×6（+0xc8 / +0xe0 / +0xe8 / +0xf0 / +0xd0 / +0xd8）→ int32 ×2
（+0x58 / +0x60）→ 版本 >1 byte（+0x15c）→ 版本 >2 short（+0x70，读取即弃
同上 quirk）→ 版本 ≥4 且 ≤8 图像类型内存修正（+0x198=3，无流内字节）→ 版
本 >4 short ×3（+0x184 / +0x186 / +0x188）→ 版本 ==6 String + byte +
double（+0x18c / 局部 / +0xf8）→ 版本 >7（v8）int32 N +（N≠0 时 N 字节，
`AfStreamRead(流, 锁定指针, N, 0)` 反编译直读）→ 尾段同上 quirk 覆盖。

### 6. PictureLineSymbol `{22C8C5A1-84FC-11D4-834D-0080C79F0371}`

虚表 `0x6ec10344`；Load `0x6eb77250`；Save `0x6eb77300`；GetClassID
`0x6eb76160`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | — | Load `0x6eb77250` |
| ② | byte | 1 字节 | this−0x30 | — | 同上 |
| ③④ | double ×2 | 8×2 字节 | this−0x2c / −0x24 | — | 同上 |
| ⑤ | LoadObject（**IPictureFillSymbol `{D842B081-330C-11D2-9168-0000F87808EE}`**，`DAT_6ec0ffa4`，类型库定名） | 20 头 + 体 | Save 槽 this−0x14 | — | 同上；实现类 PictureFillSymbol 未闭合 = 停止点 |

### 7. ArrowMarkerSymbol `{88539431-E06E-11D1-B277-0000F878229E}`

虚表 `0x6ec0c9dc`；Load `0x6eb3ec40`；Save `0x6eb3ee10`；GetClassID
`0x6eb3ef60`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`2 < ver` 拒绝 | — | — | Load `0x6eb3ec40` |
| ② | LoadObject（IColor） | 20 头 + 体 | this−0x40 | — | 同上 |
| ③ | `FUN_6eb406f0` | 不读流 | bool this−8 | — | 同上 |
| ④-⑥ | double ×3 | 8×3 字节 | this−0x8c / −0x7c / −0x6c（装载后 ×0.5 存 −0x84 / −0x74） | — | 同上 |
| ⑦⑧⑨ | int32 ×3 | 4×3 字节 | this−0x64 / −0xdc / −0xd4 | — | 同上 |
| ⑩⑪ | double ×2 | 8×2 字节 | this−0x2c / −0x24 | — | 同上 |
| ⑫ | short | 2 字节 | this−0xc4 | ≥2；**读取后内存槽被 `Symbol::GetDefaultRotateWithTransformValue` 覆盖（反汇编 `0x6eb3edde..0x6eb3edf0`：`CMP AX,ver` 门控读入 → 无条件 `MOV [槽],AX` 覆写）**，Save 恒写槽内默认值 | 同上 |

### 8. LineFillSymbol `{7914E606-…}`

虚表 `0x6ec0e72c`；Load `0x6eb5e390`；Save `0x6eb5e4e0`；GetClassID
`0x6eb5caf0`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | — | Load `0x6eb5e390` |
| ② | LoadObject（IColor） | 20 头 + 体 | this−0x6c（目标可见） | — | 同上 |
| ③ | LoadObject（ISymbol `DAT_6ec0ca2c`） | 20 头 + 体 | Save 槽 this−0x74（填充线符号候选） | — | 同上 |
| ④ | LoadObject（ISymbol 同上） | 20 头 + 体 | Save 槽 this−0x70（轮廓符号候选） | — | 同上 |
| ⑤-⑦ | double ×3 | 8×3 字节 | this−0x64 / −0x5c / −0x54 | — | 同上 |
| ⑧⑨ | int32 ×2 | 4×2 字节 | this−0xc4 / −0xbc | — | 同上 |

### 9. MarkerFillSymbol `{7914E608-…}`

虚表 `0x6ec0e9fc`；Load `0x6eb606c0`；Save `0x6eb60850`；GetClassID
`0x6eb5f3b0`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | — | Load `0x6eb606c0` |
| ② | int32 | 4 字节 | this−0x7c（样式候选） | — | 同上 |
| ③-⑥ | double ×4 | 8×4 字节 | this−0x74 / −0x6c / −0x64 / −0x5c | — | 同上 |
| ⑦ | LoadObject（IColor） | 20 头 + 体 | Save 槽 this−0x34 | — | 同上 |
| ⑧ | LoadObject（ISymbol） | 20 头 + 体 | Save 槽 this−0x80（标记符号候选） | — | 同上 |
| ⑨ | LoadObject（ISymbol） | 20 头 + 体 | Save 槽 this−0x30（轮廓符号候选） | — | 同上 |
| ⑩⑪ | int32 ×2 | 4×2 字节 | this−0xd4 / −0xcc | — | 同上 |
| ⑫ | double | 8 字节 | this−0xc（网格角候选） | — | 同上 |

### 10. GradientFillSymbol `{7914E609-…}`

虚表 `0x6ec0dd5c`；Load `0x6eb52710`；Save `0x6eb52870`；GetClassID
`0x6eb50b90`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | — | Load `0x6eb52710` |
| ② | LoadObject（**IColorRamp `{BEB87091-C0B4-11D0-8379-080009B996CC}`**，`DAT_6ec0b524`，类型库定名） | 20 头 + 体 | this−0x4c（目标可见） | — | 同上；实现类（RandomColorRamp 等）未闭合 = 停止点 |
| ③ | LoadObject（IColor） | 20 头 + 体 | Save 槽 this−0x34 | — | 同上 |
| ④ | LoadObject（ISymbol） | 20 头 + 体 | Save 槽 this−0x30 | — | 同上 |
| ⑤ | double | 8 字节 | this−0x44（角度候选） | — | 同上 |
| ⑥ | int32 | 4 字节 | this−0x2c（样式候选） | — | 同上 |
| ⑦ | double | 8 字节 | this−0x24（间隔候选） | — | 同上 |
| ⑧⑨⑩ | int32 ×3 | 4×3 字节 | this−0xc / −0x9c / −0x94 | — | 同上 |

### 11. DotDensityFillSymbol `{9A1EBA10-CDF9-11D3-81EB-0080C79F0371}`

虚表 `0x6ec0da18`；Load `0x6eb4ded0`；Save `0x6eb4e180`；GetClassID
`0x6eb4de90`。

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节，`1 < ver` 拒绝 | — | — | Load `0x6eb4ded0` |
| ② | byte | 1 字节 | this−0x34 | — | 同上 |
| ③ | double | 8 字节 | this−0x3c（点尺寸候选） | — | 同上 |
| ④ | byte | 1 字节 | this−0x28 | — | 同上 |
| ⑤ | LoadObject（IColor） | 20 头 + 体 | Save 槽 this−0x54 | — | 同上 |
| ⑥ | LoadObject（ISymbol） | 20 头 + 体 | Save 槽 this−0x58（轮廓候选） | — | 同上 |
| ⑦-⑨ | int32 ×3 | 4×3 字节 | this−0xd4 / −0xcc / −0x1c | — | 同上 |
| ⑩ | double | 8 字节 | this−0x44 | — | 同上 |
| ⑪ | byte | 1 字节 | this−0x27 | — | 同上 |
| ⑫ | int32 count | 4 字节 | this−0x78 | — | 同上 |
| ⑬ | count × LoadObject（IMarkerSymbol `DAT_6ec0ca3c`） | 20 头 + 体 | Save 槽 this−0x7c 数组 | — | 同上（**Save 倒序写出**，Load 顺序消费） |
| ⑭ | count × int32 | 4×count 字节 | Save 槽 this−0x68 数组（点值候选） | — | 同上（Save 倒序） |
| ⑮ | LoadObject（IColor） | 20 头 + 体 | Save 槽 this−0x50 | — | 同上 |

## 四、实现与测试

- `src/objects/display_symbol.rs`：新增 14 个类常量（CLSID 以 esri.clsid 表
  复核 + GetClassID 立即数互证）、14 个体读取器
  （`read_multi_layer_line_symbol_body_10_1` 等 ×11 +
  `read_template_body_10_1` / `read_line_decoration_body_10_1` /
  `read_simple_line_decoration_element_body_10_1`）、新槽入口
  `read_symbol_slot_full_10_1`（全闭合类分派，停止归约
  `slot_full_stop_10_1`）、装饰槽分派 `read_decoration_slot_10_1`
  （Template/LineDecoration 类匹配 → 解析体，未知实现类维持停止）、未闭
  合槽分类 `read_unclosed_slot_10_1`（IFontDisp/IPicture/IColorRamp/
  IMarkerBackground/IPictureFillSymbol 槽：空标记与回引可消费，新对象 = 停
  止）、循环长度限额 `read_loop_count_10_1` 与位图块限额
  `read_blob_size`（均 max_bytes 上限，读取器自身限额，非原生限制）。
  既有入口 `read_display_symbol_slot_10_1` 与其返回枚举
  `DisplaySymbolSlot10_1` 保持第一轮闭合集不变（SimpleRenderer/LegendClass/
  TextSymbol 等既有宿主的匹配无需改动）；第二轮闭合体的全部嵌套符号槽经
  新入口分派，**整合轮将既有宿主的 ISymbol 槽调用切换到
  `read_symbol_slot_full_10_1` 即获得第二轮符号递归解析**（未知类仍
  `Unsupported` 停止，语义不变）。第一轮闭合体的内部嵌套槽仍走第一轮入口
  （其闭合集以第一轮为界，集成切换时一并解决）。
- 版本 short 一律按原生有符号比较设上限并拒绝负值位模式（较原生更严，与
  既有读取器一致）；未知字节原样保留（含三处「读取即弃」short 的流内位模
  式、位图 blob 的原始字节）；截断与非法长度明确报错；循环长度负值位模式
  拒绝；失败游标与登记表不推进（trial 副本整体提交）。
- 测试：tests/display_symbol.rs 27 项（新增 15 项），覆盖各类合成输入的
  完整消费、版本门拒绝、截断事务性、未知类停止点回退、嵌套解析
  （MultiLayer 层、点密度标记、LineDecoration 元素）与 Template/装饰槽按
  类分派。`cargo fmt --check` 与 `cargo test` 全过。
- **本轮未改** `tests/robustness.rs`、`src/layers/`、`src/maps/`、
  `src/objects/element_body.rs`、`TODO.md`、`README.md`。

## 五、给整合轮的新读取器入口清单（建议加 robustness 集中清单）

直读入口（合成输入即可）：`read_template_body_10_1`、
`read_line_decoration_body_10_1`、
`read_simple_line_decoration_element_body_10_1`、
`read_multi_layer_line_symbol_body_10_1`、
`read_multi_layer_marker_symbol_body_10_1`、
`read_multi_layer_fill_symbol_body_10_1`、
`read_character_marker_symbol_body_10_1`、
`read_picture_marker_symbol_body_10_1`（版本 1..9 全分支）、
`read_picture_line_symbol_body_10_1`、`read_arrow_marker_symbol_body_10_1`、
`read_line_fill_symbol_body_10_1`、`read_marker_fill_symbol_body_10_1`、
`read_gradient_fill_symbol_body_10_1`、
`read_dot_density_fill_symbol_body_10_1`；分派入口
`read_symbol_slot_full_10_1`（全闭合类 + 未知类回退）。
既有宿主切换：SimpleRenderer/LegendClass/TextSymbol 的 ISymbol 槽调用
`read_display_symbol_slot_10_1` → `read_symbol_slot_full_10_1`（停止语义
不变）。

## 六、未解决事项（开放项）

- **无字节佐证**：本轮 15 类全部无样本实例；读取器仅以合成输入验证，不得
  声称样本验证。
- Save 版本表（`AfStreamGetSaveVersion(docVer, N, table)`）的表索引语义在
  AfCore（未提取）：表内各项为本轮直接读数，10.1 写侧取值为候选。
- 三处「装载后内存槽被 `Symbol::GetDefaultRotateWithTransformValue` 覆盖」
  的 short（ArrowMarker ⑫ / CharacterMarker ⑭ / PictureMarker v9 ⑨ 与历
  史路径 +0x70）：流内字节布局确定（版本门 + 2 字节），但读侧「读取即弃」
  语义为反汇编级观察——原生读入后写侧恒写默认值，意味着 v≥2 文件中该
  short 的值恒为写时默认（候选验证点，样本可达时核对）。
- 槽实现类未闭合（新对象维持停止）：IMarkerBackground（MultiLayerMarker
  v≥2）、IColorRamp 族（GradientFill）、IPicture（PictureMarker v9 类型
  3 / 历史 v<4、v≤6、v≥7 空串分支）、IPictureFillSymbol（PictureLine —
  实现类 PictureFillSymbol `{D842B082}` 与 OLE StdFont `{0BE35203}`/
  StdPicture（IFontDisp/IPicture 槽常见实现）均在 DisplayCore 之外或未闭
  合）。
- 同族仍未闭合：TextSymbol（体已由 element-bodies 轮闭合；本轮仅补虚表归
  属与尾段证据）、SimpleTextPath、Callout/背景族等 esri.clsid DisplayCore
  其余条目。
- 语义候选：MultiLayer 两级 int 数组与字符串的业务语义；CharacterMarker
  字体描述符八字段与 IFontDisp 属性的对应、六个 double 的业务命名；Template
  标记 16 字节的两 double 归属（位置/角度）；SimpleLineDecorationElement
  三 byte + short 的语义；各类 int32/double/byte 的业务命名（按符号学惯例
  推断）；三个 Save 的 `if (param_3 != 0)` 写链目标未解析（不影响流内字
  节）。
- 全部结论为静态反编译观察，未经 ArcMap 10.1 运行验证；样本对位无从谈起
  （无实例），人工目视验收未做。
