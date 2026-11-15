# ArcMap 10.1 FeatureLayer 尾段链第一环：渲染器及直接后继类

记录日期：2026-11-14。对应 TODO「MXD 解析」节开放项中的「越过条目 1 需渲染器
/LegendGroup/FieldInfo/EditTemplate 等约 30 类尾段链闭合」——本轮闭合链条第一
环（SimpleRenderer 及其嵌套的符号、颜色、图例组、图例类共 5 类）。全部为静态
反编译观察，未运行 ArcMap，未做任何人工目视验收；实现见 `src/layers/feature_layer_tail.rs`
（模块文档含同源字段表）。

## 样本边界（先声明）

仓库参考样本 `samples/测试底图.mxd` 的图层列表条目 1（FDOGraphicsLayer）体内，
关联 FeatureLayer 的渲染器槽新对象（SimpleRenderer，键 35）位于 `/Maps:0x3ed`。
本轮闭合的 5 类全部有样本字节佐证：`0x3ed..0x602` 逐字段吻合（辅助观察，见
下文「样本对位」）。链条在此之后仍未闭合。

## 方法与文件身份

- REA MCP（Ghidra 12.2 提供者，JDK 25）反编译 + 样本字节对位（`temp/r6-maps.bin`，
  `cargo run -- hex` 与自写脚本）。与并行代理共享 REA 服务器，活动目标被切换
  后以「open + 操作同批」重试恢复。
- 本轮新提取 **DisplayCore.dll**（Setup1.cab，符号/颜色实现）：SHA-256
  `9b3e61f9…a926e4d2`，pefile 复核 `10.1.0.3035`。
- 复用既有基线：FeatureLayer.dll、Layer.dll（SHA-256 与第六轮记录一致）、
  LayerLib.dll。类名/接口名：`esri.clsid.ecfg` 与 10.1 类型库。
- 辅助脚本与提取件在 `temp/agent-fl-tail/`。

## 一、SimpleRenderer（链条入口，样本键 35 @0x3ed）

- 类身份三方互证：`esri.clsid` 声明（模块 FeatureLayer.dll）、GetClassID
  `0x6f722e00` 写出 `{F3435801-5779-11D0-98BF-00805F7CED21}`、样本 `0x3ed`
  对象头逐字节相同。
- IPersistStream 虚表 `0x6f7b0dfc`：槽 +0x14 = 调整器 thunk `0x6f724cf0` →
  实体 Load `0x6f722b70`；Save `0x6f722cf0`（对称）。
- 字段序：
  1. int16 版本：`3 < (short)ver` 有符号拒绝（样本 3）；
  2. LoadObject（ISymbol `{F3435802-5779-11D0-98BF-00805F7CED21}`）——样本为
     SimpleMarkerSymbol 新对象（键 36）；
  3. LoadObject（ILegendGroup `{167C5E9F-…}`）——样本 LegendGroup 新对象
     （键 39）；
  4. LoadObject（ILegendItem `{52AA5BC0-…}`）——样本 16 零空标记；
  5. int16 → obj+0x128（样本 0）；
  6. LoadLegacyRotationFieldAndType（LayerLib.dll `0x786f93d0`，本类调用不带
     版本 short）：String 旋转字段 + int32 旋转类型（样本空串 + 0）；
  7. String → obj+0x12c 标签（样本空串）；
  8. 版本 >2：RotationRenderer::Load（LayerLib.dll `0x786f7e40`：short 版本
     ≤1 + int32 ×2 + String ×3 + double ×6）+ SizeRenderer::Load
     （LayerLib.dll `0x7870bbc0`：short 版本 ≤1 + int32 + String + double ×2）。
- 写侧：Save 对称，版本表 docVersion→3；docVersion<4 时省略第 8 段（与读侧
  版本 >2 门在 10.1 一致）。

## 二、SimpleMarkerSymbol（样本键 36 @0x403，DisplayCore.dll）

- GetClassID `0x6ebbf310` 写出 `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`；
  IPersistStream 虚表 `0x6ec15724`：+0x14 thunk `0x6ebc0460` → Load
  `0x6ebbefa0`；Save `0x6ebbf190`。
- 字段序：① short 版本（>2 拒绝；样本 2）→ ② LoadObject（IColor
  `{20CD40B0-…}`）→ ③ double（样本 6.0，尺寸候选）→ ④ int32 ×3（样本
  0/13/0，13 为 marker 样式候选）→ ⑤ double ×3（样本 0）→ ⑥ 1 字节（样本 1）
  → ⑦ double（样本 1.0）→ ⑧ LoadObject（IColor，轮廓色）→ ⑨ 版本 >1：short
  （样本 0xFFFF）。
- 构造中的 `FUN_6eb406f0` 两处调用不读流（虚调用返回值），无流内字节。

## 三、RgbColor（样本键 37/38/154，DisplayCore.dll）

- GetClassID `0x6eb3d930` 写出 `{7EE9C496-D123-11D0-8383-080009B996CC}`；
  IPersistStream 虚表 `0x6ec0c70c`：+0x14 thunk `0x6eb3dbf0` → Load
  `0x6eb3d8b0` → Color 核心 `0x6eb35310`；Save `0x6eb3d8f0`/`0x6eb35470`。
- 字段序：① short 版本 ≤1 → ② 第二个 short 版本 ≤1（Color 核心）→ ③ 1 字节
  分支标志：==0 → 3 个 double（经 Color 虚表 +0x34 装载，样本三实例 flag 均
  0）；!=0 → 4 字节 CMYK（虚表 +0x1c）→ ④ 1 字节 → ⑤ 1 字节（内存值按
  `0xff - 流内字节` 取补）。
- flag==0 分支体长 31 字节，与样本三实例（键 37 @`0x419`、38 @`0x481`、
  154 @`0x2c0b`）逐字节吻合。

## 四、LegendGroup（样本键 39 @0x4b6，Layer.dll）

- GetClassID `0x78665c80` 写出 `{167C5EA2-AF20-11D1-8817-080009EC732A}`；
  IPersistStream 虚表 `0x7866fa3c`：+0x14 Load `0x78665cc0`，+0x18 Save
  `0x78665e20`。
- 字段序：① short 版本（>3 拒绝；样本 3）→ ② short → obj+0xc（样本 -1）→
  ③ 版本 ≥2：short → obj+0xe（<2 时内存默认 -1；样本 -1）→ ④ String →
  obj+0x14（样本空串）→ ⑤ int32 计数 + 循环 LoadObject（ILegendClass
  `{167C5E9E-…}`）（样本 1 → LegendClass 键 40）→ ⑥ 版本 >2：short →
  obj+0x12（样本 0）。
- 写侧：版本表 docVersion→3；docVersion>4 时才写第 ⑥ 段（读侧按版本 >2 门，
  10.1 两者一致）。

## 五、LegendClass（样本键 40，Layer.dll）

- GetClassID `0x78662fe0` 写出 `{167C5EA3-AF20-11D1-8817-080009EC732A}`；
  IPersistStream 虚表 `0x7866f728`：+0x14 Load `0x78663560`，+0x18 Save
  `0x78663640`。
- 字段序：① short 版本（反编译未见上限拒绝，样本 2；实现按 ver>1 门分支、
  不设额外上限）→ ② LoadObject（ISymbol）——样本为对 SimpleMarkerSymbol
  键 36 的回引头，无体 → ③ String 标签（样本空串）→ ④ String 描述（样本
  空串）→ ⑤ LoadObject（ILegendClassFormat `{52AA5BC2-…}`）——样本 16 零
  空标记；新对象体未闭合 = 停止点 → ⑥ 版本 >1：int32 → obj+0x1c（样本 -1）。

## 样本对位（辅助观察）

`temp/r6-maps.bin` `0x3ed..0x602` 五类嵌套体逐字段吻合：SimpleRenderer 体
终结于 `0x5b0`；其后回到关联 FeatureLayer 的后继字段——`+0x144` GUID（16 零）、
String「BSM」、double ×2、short、IHyperlink 计数 0、IColor 空标记、String ×2、
int32，到 `0x602` 的 `{A2BAAE2C-…}` 计数循环（FieldInfo 族）之前。该循环及其
后的类不在本轮范围（`feature_layer` 读取器当前在 `+0x144` GUID 后即停止，
后继字段的字段序闭合属下一步集成范围）。

## IID 表更正

第三轮研究文档（docs/arcmap-10.1-map-object.md）IID 表中 `{F3435802-…}`
（ISymbol）的 Data2 误记为 5799；经本轮二进制字节与类型库双重复核应为
**5779**（与 SimpleRenderer CLSID 同 Data2）。

## 未解决事项

- ILegendItem 槽（`{52AA5BC0}`）coclass 体未闭合（样本空标记，无字节佐证），
  新对象即停止；
- ILegendClassFormat 槽（`{52AA5BC2}`）coclass 体未闭合（样本空标记），
  LegendClass 在此停止；
- ISymbol 槽只闭合 SimpleMarkerSymbol；SimpleLineSymbol / SimpleFillSymbol /
  MultiLayerMarkerSymbol 等同族未闭合（新对象即停止）；
- IColor 槽只闭合 RgbColor；CmykColor/HlsColor/HsvColor/GrayColor 同族未闭合；
- RotationRenderer/SizeRenderer 的 double ×6（样本前 3 个 0、后 3 个 360.0）
  语义为候选（「3D 随机旋转上下限」），对称写侧 getter 未逐个定名；各
  int32/String 业务命名按内容推断（style 候选等）；
- FeatureLayer 渲染器槽之后的后继字段段（`+0x144` GUID 至 FieldInfo 循环）
  的字段序闭合与接入未做；
- 全部结论为静态观察，未经目标软件运行验证；样本对位仅为辅助观察。

## FeatureLayer 后继字段段闭合：FieldInfo 循环、元素体与 RTM 3035 差异裁决（2026-11-14 第八轮）

对应 TODO「MXD 解析」开放项第一条「越过 SimpleRenderer 体后的 FeatureLayer 后继字段段
（+0x144 GUID 之后 String/double/IHyperlink/IColor 至 `{A2BAAE2C}` FieldInfo 循环——样本
对位存在但与 RTM 3035 反编译的不可调和差异未解决，读取器维持 AfterDataSourceGuid 停止）」。
本轮闭合后继字段段全程与 FieldInfo 元素体、嵌套 NumericFormat 体，并裁决：**「样本与
RTM 3035 反编译不可调和」的提法撤销，样本 0x5b0..0xafe 与 RTM 3035 反编译逐字节可调和**
（归因取证轮「双层误读」结论的延伸确证，见下文裁决节）。实现见
`src/layers/feature_layer_tail.rs`（`read_feature_layer_successor_10_1` 及 FieldInfo/
NumericFormat 体读取器）与 `src/layers/feature_layer.rs`（+0x144 GUID 后接入）。

### 方法与文件身份

- 本机 pyghidra（Ghidra + JDK 25）打开既有工程副本（temp/agent-fl-tail2/ghproj*，全部由
  /tmp/r3、/tmp/r7flt 既有工程复制，未动共享工程）。三份 DLL 均为 10.1.0.3035：
  - FeatureLayer.dll：沿用基线（SHA-256 `2baf8706…`，ImageBase 0x6f280000 段内 VA 即静态映像 VA）；
  - Layer.dll：SHA-256 `fe52bd9988b66f9d…`，ImageBase `0x78650000`（temp/agent-fl-tail/ 既有提取物，本轮 pefile 复核版本）；
  - SystemCore.dll：SHA-256 `f63cf9494acc155d…`，ImageBase `0x7ffb0000`（temp/arcgis101-bin/ 既有提取物，本轮 pefile 复核版本）。
- 样本字节对位：temp/r6-maps.bin（152941 字节，SHA-256 `fc0149f1…`，/Maps 流内偏移，
  与归因取证轮同源同摘要）；逐字节走查脚本 temp/agent-fl-tail2/fs_sample_walk.py。
- 类名/IID：esri.clsid.xml 全表（temp/agent-fl-tail/ecfg/）与 esriSystem 类型库 dump
  （/tmp/mxd-arcgis101-guiddocs/wine/）。全部为静态取证，未运行目标软件，未人工验证。

### RTM 3035 差异裁决（结论：不可调和提法撤销，异常系误读）

第一轮样本对位记录的开放项「+0x144 GUID 之后字段与 RTM 3035 反编译不可调和（工作假设
样本由服务包构建）」在本轮正式裁决：

1. 归因取证轮（docs/arcmap-10.1-spatial-reference.md）已把该「不可调和」定性为两层误读
   的产物（WorkspaceName 体边界误读 + 渲染器槽位误读），本轮在其基础上补齐最后一块：
   旧对位只走到 FieldInfo 计数（`0x602`）之前，本轮按反编译字段序把 FieldInfo 计数循环
   全部 14 条目逐字节走到循环终点 `0xafe`——恰为下一个对象头（ALPC `{1D5849F3}` 键 59）
   之前，无一处分歧，无剩余字节。
2. 反编译侧以 10.1.0.3035 FeatureLayer.dll `0x6f6e9940` 一手伪代码为准，关键版本门
   另以汇编复核：FieldInfo 循环条目名的「架构版本 >5」门在 `0x6f6e9c91`
   （`CMP word ptr [ESP+0x20],0x5` / `JLE 0x6f6e9cb5`，证据：确定的）。
3. 结论：样本可由 10.1.0.3035 的读/写侧行为完整解释，「SP 构建」工作假设不再必要
   （与归因取证轮一致）；旧提法在本文件第一轮「样本对位」节中保留原文，以本节为准。

### 后继字段段字段序（FeatureLayer::Load `0x6f6e9940`，证据：确定的）

渲染器槽（`{40A9E884}`=IFeatureRenderer）新对象体之后的序列（请求 IID 均为前轮 IID 表
既有数据，本轮逐一从工程副本复核字节）：

| # | 字段 | 原语/请求 IID | 对象偏移 | 样本偏移（层 32） | 证据 |
| --- | --- | --- | --- | --- | --- |
| 1 | 原样 16 字节 GUID | AfStreamRead(0x10) | +0x144 | `0x5b0`（16 零） | 伪代码 + 样本（既有） |
| 2 | String | AfCore String::Read | +0x154 | `0x5c0`（cb=8，「BSM」） | 伪代码 92 行 |
| 3 | double | AfStreamRead(8) | +0x158 | `0x5cc`（0.0） | 伪代码 96 行 |
| 4 | double | AfStreamRead(8) | +0x160 | `0x5d4`（0.0） | 伪代码 100 行 |
| 5 | short | AfStreamRead(2) | +0x168 | `0x5dc`（0） | 伪代码 104 行 |
| 6 | int32 计数 + 循环 LoadObject(IHyperlink `{3036D35D-…}`) | DAT_6f7ac2a4 | 计数无槽 | `0x5de`（0，无条目） | 伪代码 108-123 行；每项仅 LoadObject + `0x6f701ad0` 数组入集（反编译核对不读流） |
| 7 | LoadObject(IColor `{20CD40B0-…}`) | DAT_6f7aaf98 | — | `0x5e2`（16 零空标记） | 伪代码 124-128 行 |
| 8 | String | String::Read | +0x1a8 | `0x5f2`（空串） | 伪代码 132 行（读后赋值到 -0x88，不读流） |
| 9 | String | String::Read | +0x184 | `0x5f8`（空串） | 伪代码 137 行 |
| 10 | int32 | AfStreamRead(4) | +0x188 | `0x5fe`（0） | 伪代码 141 行（汇编 `0x6f6e9bb6`：LEA ECX,[EBX+0x188]） |
| 11 | int32 FieldInfo 计数 + 循环 [（版本 >5）String 条目名 + LoadObject(IFieldInfo `{A2BAAE2C-…}`)] | DAT_6f7ab098；版本门汇编 `0x6f6e9c91` | 登记经 +0x1d0 接口虚表 +0x18 | 计数 `0x602`（14），条目 `0x606..0xafe` | 伪代码 162-221 行 |

中间的 `FUN_6f6e0c00`（GUID 后调用）、`FUN_6f6c8100`、`FUN_6f6acc10`、`FUN_6f701ad0`、
`FUN_6f6dd210`（循环后调用）经反编译核对均无 AfStreamRead/String::Read/LoadObject
（证据：确定的；temp/agent-fl-tail2/fl_verify.txt），无流内字节。IHyperlink 循环、
IColor 槽之后到 FieldInfo 计数之间无版本门（伪代码 unconditional 链）。

### FieldInfo 元素体（Layer.dll `0x7865b520`，证据：确定的）

类身份三方互证：esri.clsid 声明 `{A2BAAE2D-969B-11D2-AE77-080009EC732A}`（模块
Layer.dll，progID FieldInfo）；GetClassID `0x7865b4e0` 以立即数对
`0x11D2969BA2BAAE2D`/`0x2A73EC09000877AE` 写出该 CLSID；IPersistStream 虚表
`0x7866dd58`（COL `0x786707b8`，RTTI `.?AVFieldInfo@@` TD `0x78679708` 的 9 连 COL 簇
之一）槽 +0x0c 经调整器 thunk `0x7865b8f0` 指向 GetClassID、+0x14 thunk `0x7865b8e0`
→ 实体 Load `0x7865b520`、+0x18 thunk `0x7865b7c0` → Save `0x7865b660`。样本 14 个
对象头（键 41/43/44/45/46/47/48/49/50/51/52/54/56/58，`{A2BAAE2D}`）逐字节相同。
工厂宿主差异备查：FeatureLayer.dll `0x6f6c86f0` 以 AfCreateInstance(DAT_6f7ab188=该
CLSID) 创建实例——创建点在 FeatureLayer.dll、实现类在 Layer.dll（与「工厂宿主/声明
模块」既有惯例一致）。

| # | 字段 | 版本门 | 对象偏移（接口 this 相对） | 样本（14 实例） |
| --- | --- | --- | --- | --- |
| ① | int16 版本 | `4 < (short)ver` 有符号拒绝 | — | 全部 4 |
| ② | short | 恒有 | -0x16 | OBJECTID 0xFFFF、BSM 0x0000，余 0xFFFF（可见性候选） |
| ③ | short | 版本 >1 | -0x14 | 全部 0 |
| ④ | String 别名 | 恒有 | -0x10 | 与循环条目名同文（OBJECTID/BSM/FID/Shape/ZDZHDM/BDCDYBSM/YSDM/JZDH/JBLX/JZDLX/XZBZ/YZBZ/ZZBZ/Ring） |
| ⑤ | LoadObject(**INumberFormat** `{7E4F470D-…}`，DAT_7866dd9c) | 恒有 | -0xc | OBJECTID/XZBZ/YZBZ/ZZBZ 四条为 NumericFormat 新对象（键 42/53/55/57），其余 16 零空标记 |
| ⑥ | short + int32 | 版本 >2 | -0x18 / -0x1c | 全部 -1 / -1 |
| ⑦ | short + short | 版本 >3 | -0x8 / -0x6 | 全部 0 / -1（FID）或 0 / 0 |

版本 3 差异（反编译 && 链短路语义，Save `0x785b660` 按 docVersion 对称：>2 写 ⑥、
>3 写 ⑦）：版本 3 无 ⑦（-0x8/-0x6 均不写）；版本 ≤2 无 ⑥⑦。样本全部为版本 4，
③⑥⑦ 无字节佐证的边界仅在合成测试覆盖。

### NumericFormat 体（SystemCore.dll `0x7ffce4b0`，证据：确定的）

类身份三方互证：esri.clsid 声明 `{7E4F4719-8E54-11D2-AAD8-000000000000}`（模块
SystemCore.dll）；esriSystem 类型库 coclass #212 NumericFormat（#60 INumberFormat =
`{7E4F470D}`，即 ⑤ 的请求 IID）；GetClassID `0x7ffce470` 立即数写出该 CLSID，
IPersistStream 虚表 `0x80070ec4` 槽 +0x14 thunk `0x7ffcea50` → Load `0x7ffce4b0`、
+0x18 thunk `0x7ffcea90` → Save `0x7ffce590`。字段序（体长恒 24 字节，无版本分支、
无嵌套对象）：

| # | 字段 | 版本门 | 对象偏移 | 样本 4 实例 |
| --- | --- | --- | --- | --- |
| ① | int16 版本 | `1 < (short)ver` 拒绝 | — | 全部 1 |
| ② | int32 ×4 | 恒有 | -0x1c/-0x18/-0x14/-0x10 | OBJECTID 条目 0/0/0/12；XZBZ/YZBZ/ZZBZ 条目 0/6/0/12 |
| ③ | short ×3 | 恒有 | -0xc/-0xa/-0x8 | 全部 0 |

### 样本逐字节对位（辅助观察）

`0x5b0..0xafe` 按上述字段序走查（temp/agent-fl-tail2/fs_sample_walk.py，断言终点恰为
ALPC `{1D5849F3-0D33-11D1-A26F-080009B6F22B}` 键 59 对象头 @`0xafe`）：全部逐字节吻合。
分层核对：+0x144 GUID @`0x5b0`（16 零）→ String「BSM」@`0x5c0` → double 0.0 ×2 →
short 0 → IHyperlink 计数 0 @`0x5de` → IColor 空标记 @`0x5e2` → 空串 ×2 → int32 0 →
FieldInfo 计数 14 @`0x602` → 14 条目（每条 = String 条目名 + `{A2BAAE2D}` 头 + 版本 4
体；四条含 NumericFormat 嵌套）→ `0xafe` 循环终点。

### 实现接线（src/）

- `feature_layer_tail` 扩展：`FIELD_INFO_CLASS_ID`/`NUMERIC_FORMAT_CLASS_ID` 常量、
  `read_numeric_format_body_10_1`（无停止点）、`read_field_info_body_10_1`（INumberFormat
  槽未知类为停止点，回退到槽位起点不保留键）、
  `read_feature_layer_successor_10_1`（IHyperlink 循环新对象、IColor 非 RgbColor 新
  对象、FieldInfo 循环条目非 FieldInfo 新对象均为停止点并回退；FieldInfo 体内停止传播
  为 `SuccessorStop10_1::FieldInfoBody`；条目名按 FL 架构版本 >5 门读取，≤5 保留 None；
  负计数不进循环原样保留）。停止语义与前轮一致：首个未闭合点成功停止，剩余字节保留。
- `feature_layer` 接入：`AfterDataSourceGuid` 停止撤销，GUID 后调用后继段读取器；
  `FeatureLayerStop10_1::AfterDataSourceGuid` 由 `Successor(SuccessorStop10_1)` 取代；
  前缀结果新增 `successor` 字段。
- `mxd_maps`：FeatureLayer/FDO 关联图层节点追加后继段消费诊断（停止点、FieldInfo/
  IHyperlink 计数、+0x154 字符串原样输出）；模型字段不变（后继段标量语义未定，不进模型）。
- 测试：tests/feature_layer_tail.rs 25 → 43 项（NumericFormat/FieldInfo/后继段的正常、
  版本门、逐字节截断、停止点回退与登记表事务性、负计数、回引、版本 >5/≤5 条目名门）；
  tests/robustness.rs 集中清单 28 → 29 入口（新增 FeatureLayerSuccessor10_1，另两处
  样本扩展后继段）；tests/feature_layer.rs、tests/fdo_graphics_layer.rs、
  tests/fdo_legacy.rs、tests/group_layer.rs、mxd_maps 内嵌测试的合成 FeatureLayer 体
  同步补后继段。
- 仓库样本 `parse` 停止点由 AfterDataSourceGuid（游标 `0x5c0`）推进到 **`0xafe`**
  （FieldInfo 循环终点；其后 ALPC `{1D5849F3}` 键 59 头起保留为未知）。

### 本轮未解决事项

- IHyperlink 循环 coclass（Hyperlink 族）体未闭合（样本计数恒 0，无字节佐证）；
  INumberFormat 槽只闭合 NumericFormat（PercentageFormat/FractionFormat/RateFormat 等
  同族未闭合）；
- 后继段与 FieldInfo/NumericFormat 各标量业务语义未定性（可见性、别名等均为候选，
  见模块文档清单）；FieldInfo 条目名与别名在样本中恒同文，分歧场景未见样本；
- FieldInfo 版本 ≤3 分支（⑥⑦ 门）与 NumericFormat 版本边界仅有反编译证据，无样本
  字节佐证；
- FieldInfo 循环之后的 ALPC 槽、short +0x1bc、FeatureIDSet 槽、选择集 ID 数组及更深的
  版本分支字段未闭合（下一环）；样本 11 层中其余 10 个 FeatureLayer 体的后继段未逐一
  走查（层 32 已覆盖全部分支形态：两槽空标记、嵌套 NumericFormat 与空标记并存）；
- 全部结论为静态取证，未运行 ArcMap 10.1，未人工目视验收；样本对位为辅助观察。


## 接线轮（2026-11-15）

研究结论无改动；本节记录接入点落位、目录修正与样本停止点对比。

- 目录修正：RgbColor 体读取器及 `RGB_COLOR_CLASS_ID` 移入
  `src/objects/display_color.rs`，SimpleMarkerSymbol 体读取器及其类标识移
  入 `src/objects/display_symbol.rs`（消除 B 轮两模块移入 objects 后残留的
  objects→layers 依赖倒置）；两常量在本模块保留 `pub use` 重导出，内部调
  用点改为调对象层函数。
- ISymbol 槽接线（display-tails 建议接入点）：`read_simple_renderer_body_10_1`
  与 `read_legend_class_body_10_1` 的 ISymbol 槽改调
  `objects::display_symbol::read_display_symbol_slot_10_1`——六类闭合符号
  （SMS + 五类线条/填充）整体解析，未知符号类维持停止回退；嵌套停止新增
  `SymbolNested` 变体透传（`SymbolBody` 保留 SMS 原语义）。
- IColor 槽接线：SMS 双色槽（display-symbol 内）与
  `read_feature_layer_successor_10_1` 的 IColor 槽改调
  `objects::display_color::read_color_slot_10_1`；五类闭合颜色整体解析，
  未知颜色类维持 `ColorUnsupported`/`SuccessorStop10_1::ColorUnsupported`
  停止（槽位字段停止时记 `Null`，与原语义一致）。
- ILegendClassFormat 槽接线（legend-tails 建议接入点）：LegendClass 的该槽
  新对象且类标识为 LegendClassFormat 时接
  `read_legend_class_format_body_10_1`，体内停止以新增
  `LegendClassStop10_1::FormatBody` 透传；其余类维持
  `FormatNewObject` 停止（槽位字段改为 `LegendClassFormatSlot10_1` 枚举，
  未闭合类记 `Unsupported`）。
- 测试：tests/feature_layer_tail.rs 增至 48 项（新增接线正常路径、嵌套停
  止透传、后继段 CmykColor 解析等 5 项；原 43 项在更新类型/签名后全部保
  持通过，样本形态分段无回退）。
- **样本停止点不变**：`parse samples/测试底图.mxd` 仍停在流内 0xafe
  （`Successor(FieldInfoLoopCompleted)`，剩余 150127 字节）。原因：0xafe 处
  是后继段之后的 ALPC 槽（请求 IID `{1D5849F2-…}`）新对象（流内类
  `{1D5849F3-0D33-11D2-A26F-080009B6F22B}`，键 59）——该 coclass 体未闭合
  且无外部长度，本轮四族接入点全部位于其后方（键 76/146 的 EditTemplate、
  键 1547 的 PropertySet、键 1612 起的元素簇、键 2026 链的第二 FCN），在
  ALPC 槽闭合前不可达。样本分段的既有验证（渲染器链 `0x3ed..0x5b0`、后继
  段 `0x5b0..0xafe`）全部保持。

## 第九轮：ALPC 环、FeatureIDSet、图层扩展段（EditTemplate 宿主裁决）与 TimeData 族（2026-11-15）

对应第八轮遗留开放项「FieldInfo 循环之后的 ALPC 槽、short +0x1bc、FeatureIDSet 槽、
选择集 ID 数组及更深的版本分支字段未闭合」。本轮闭合：ALPC/
LabelEngineLayerProperties（LELP）/EditTemplateManager（ETM）/FeatureIDSet 四类体、
FeatureLayer::Load 后继段从 ALPC 槽到版本 >0x21 门 `short(+0x2ca)` 的全部剩余字段
（含版本 0x1c..0x1f 分支与 TimeData 族四段），并裁决 **EditTemplate 宿主**：宿主 =
后继段图层扩展段中的 EditTemplateManager（LayerLib.dll `ExtensionHelper::
LoadExtensions` 装载），第八轮「键序 75→76 → `SaveObject(+0x218)` 槽」假设撤销。
实现：`src/layers/annotate_props.rs`（ALPC/LELP/ETM）、`src/layers/feature_id_set.rs`、
`src/layers/feature_layer_tail.rs`（后继段延续，在 `read_feature_layer_successor_10_1`
内原位扩展）。

### 方法与文件身份

- 本机 pyghidra（Ghidra + JDK 25）工程副本：FeatureLayer.dll 复用
  temp/agent-fl-tail2/ghproj 复制工程的原始分析副本（/tmp/r3/ghproj_fl 的再复制），
  LabelPlacement.dll 新建工程（temp/ 下既有提取物复制，SHA-256 `8415f63c…35ea3d`，
  pefile 复核 10.1.0.3035，ImageBase 0x78530000），LayerLib.dll 新建工程
  （temp/agent-fl-tail/ 既有提取物），DisplayCore.dll 新建工程（temp/agent-fl-tail/
  既有提取物，本轮 analyze）。全部为工程副本，未打开主检出区工程锁。
- 样本字节：temp/r6-maps.bin 副本（152941 字节，SHA-256 `fc0149f1…acfe`），逐字节
  走查脚本在 temp/agent-fl-tail3/（本轮工作副本，含全部闭合段的走查器）。
- 类名/IID：esri.clsid 全表（temp/agent-fl-tail/ecfg/）、esriCarto 类型库 dump。
  全部静态取证，未运行目标软件，未人工目视验收。

### ALPC 体（LabelPlacement.dll `0x78531be0`，证据：确定的）

类身份三方互证：esri.clsid 声明 `{1D5849F3-0D33-11D2-A26F-080009B6F22B}`（progID
AnnotateLayerPropertiesCollection，esriCarto coclass）；GetClassID `0x78531ba0` 以
两个 8 字节立即数写出该 CLSID；样本 `/Maps:0xafe` 头（键 59）逐字节相同。请求 IID
`{1D5849F2-…}`（FL::Load `DAT_6f7adda4`）= IAnnotateLayerPropertiesCollection。
注：LabelPlacement.dll 内本类无 RTTI COL，定位经 GetClassID 立即数 + 子 IID
`{01004142-…}` 引用 + Load 形状（版本+计数+循环）三线合证。

| # | 字段 | 版本门 | 样本 |
| --- | --- | --- | --- |
| ① | int16 版本 | `2 < (short)ver` 拒绝 | 2 |
| ② | int32 计数 | 恒有（负数原生不进循环） | 1 |
| ③ | 循环 [LoadObject(IAnnotateLayerProperties) + （版本 ≥2）int32 条目键] | 条目键版本 ≥2 | 1 条：LELP 新对象（键 60）；条目键未读（LELP 体内停止） |
| ④ | int32 尾 int32 | 版本 ≥2 | 未达（停止在前） |

### LELP 体（LabelPlacement.dll `0x785468c0`，证据：确定的）

类身份：esri.clsid 声明 `{01004145-0D1C-11D2-A26F-080009B6F22B}`（progID
LabelEngineLayerProperties）；样本 `/Maps:0xb18` 头（键 60）逐字节相同
（LabelPlacement 实现类 GetClassID 为常量拷贝形，代码无立即数）。

| # | 字段 | 版本门 | 对象偏移 | 样本 |
| --- | --- | --- | --- | --- |
| ① | int16 版本 | `7 < (short)ver` 拒绝 | — | 7 |
| ② | String | 恒有 | -0x78 | `[JZDH]`（标注表达式候选） |
| ③ | short | 恒有 | -0x74 | -1 |
| ④ | short | 版本 ≤1 | — | 版本 7 不读 |
| ⑤ | double | 恒有 | -0x54 | 0.0 |
| ⑥ | int32 | 恒有 | -0x60 | 0 |
| ⑦⑧⑨ | short ×3 | 恒有 | -0x5c/-0x4c/-0x4a | -1/0/-1 |
| ⑩ | int32 | 恒有 | -0x80 | 0 |
| ⑪⑫ | double ×2 | 恒有 | -0x34/-0x3c | 0.0/0.0 |
| ⑬ | short | 恒有 | -0x48 | -1 |
| ⑭ | String | 恒有 | -0x68 | `默认`（标注类名候选） |
| ⑮ | String | 恒有 | -0x64 | 空串 |
| ⑯ | LoadObject(ITextSymbol `{A80B5E91-…}`) | 恒有 | 槽 | **TextSymbol 新对象（键 61）= 样本停止点** |
| ⑰ | LoadObject(IBasicOverposterLayerProperties `{3141F2FB-…}`) | 恒有 | 槽 | 未达 |
| ⑱ | LoadObject(IAnnotationExpressionEngine `{AA157206-…}`) | 恒有 | 槽 | 未达 |
| ⑲ | LoadObject(IEnvelope `{1B077914-…}`) | 恒有 | 槽 | 未达（归因轮证据：键 74 = 本槽 Envelope） |
| ⑳㉑ | double ×2 | 恒有 | -0x24/-0x1c | 未达 |
| ㉒㉓ | int32 ×2 | 恒有 | -0x14/-0x2c | 未达 |
| ㉔ | LoadObject(IGraphicsContainer `{D09411D1-…}`) | 版本 >2 | 槽 | 未达 |
| ㉕ | short | 版本 ≥4 | -0x46 | 未达 |
| ㉖ | int32 | 版本 ≥5 | -0x7c | 未达 |
| ㉗ | short | 版本 ==6 | — | 版本 7 不读 |

### FeatureIDSet 体（FeatureLayer.dll `0x6f6d97d0`，证据：确定的）

类身份：esri.clsid 声明 `{D5BB4B88-E0A1-11D2-9F4D-00C04F6BC78E}`（progID
FeatureIDSet，**实现模块 = FeatureLayer.dll**，与宿主同 DLL；CLSID 数据常量
`0x6f7ac390` 紧邻请求 IID 常量 `0x6f7ac380`）；样本 `/Maps:0xf63` 头（键 75）逐
字节相同。Load 经类描述块（函数三连 + IID + CLSID）定位，Save `0x6f6d9880` 对称
互证。字段序：① int16 版本（`1 < (short)ver` 有符号拒绝；样本 1）→ ② int32 计数
（负数原生不进循环；样本 0）→ ③ count × int32（要素 OID，逐元素读）。体长 6 字
节，样本消费 0xf63..0xf7c。

### FeatureLayer 后继段延续字段序（`0x6f6e9940`，证据：确定的）

FieldInfo 循环之后（版本门均伪代码复核，关键门另有写侧对照；ver = FeatureLayer
体版本，样本 34=0x22）：

| # | 字段 | 版本门 | 对象偏移 | 样本偏移（层 32） |
| --- | --- | --- | --- | --- |
| ⑪ | LoadObject(ALPC `{1D5849F2-…}`) | 恒有 | +0x1cc | `0xafe`（键 59） |
| ⑫ | short | 恒有 | +0x1bc | —（停止点后未定位） |
| ⑬ | LoadObject(FeatureIDSet `{D5BB4B87-…}`) | 恒有 | +0x1dc | `0xf63`（键 75） |
| ⑭ | int32 选择集计数 + count×int32（原生单次整块读；负数跳过） | 恒有 | — | `0xf7d`（0） |
| ⑮ | short | 恒有 | +0x1fe | `0xf81`（0xFFFF） |
| ⑯ | LoadObject(IDisplayFilter `{19F85377-…}`) | 版本 ≥2 | +0x218 | `0xf83`（空标记） |
| ⑰ | short ×3 | 版本 ≥3 | +0x21c/+0x220/+0x21e | `0xf93`（0,0,0） |
| ⑱ | int32 | 版本 ≥0xc | +0x22c | `0xf99`（0） |
| ⑲ | LoadObject(IName `{4BA33331-…}`) | 版本 ≥4 | +0x230 | `0xf9d`（空标记） |
| ⑳ | int32 关系类计数 + 循环 LoadObject({311C2EE2-…}=IRelationshipClassName) | 版本 >6 | — | `0xfad`（0） |
| ㉑ | String | 版本 ≥8 | +0x18c | `0xfb1`（cb=2 空串） |
| ㉒ | LoadObject(IEnvelope `{1B077914-…}`) | 版本 >8 | 槽 | `0xfb7`（空标记） |
| ㉓ | int32 范围计数 + count×int32（逐元素读） | 版本 >9 | — | `0xfc7`（0） |
| ㉔ | **图层扩展段**（LayerLib.dll `ExtensionHelper::LoadExtensions` `0x786c22e0`：[int32 计数][每条 int64 尺寸 + 对象头 + 体]；未知类按尺寸 Seek 跳过） | 版本 ≥0xb | +0x26c | `0xfcb`（1 条：EditTemplateManager，尺寸 387） |
| ㉕ | double | 版本 ≥0xd | +0x1e8 | —（停止点后） |
| ㉖ | LoadObject(ISymbol `{F3435802-…}`) | 版本 >0xd | 槽 | —（停止点后） |
| ㉗ | short + int32 | 版本 >0xe | +0x12e/+0x1f0 | —（停止点后） |
| ㉘ | LoadObject(IEnvelope) | 版本 >0xe | 槽 | —（停止点后） |
| ㉙ | LoadObject(IUID `{1714D59B-…}`) | 版本 >0x10 | 槽 | —（停止点后） |
| ㉚ | String | 版本 ≥0x12 | +0x28c | —（停止点后） |
| ㉛ | short（==0x12/0x13 临时 short；≥0x13 的 +0x1d8） | 见门 | +0x1d8 | —（停止点后） |
| ㉜ | double ×2 | 版本 ≥0x14 | +0x290/+0x298 | —（停止点后） |
| ㉝ | short | 版本 ≥0x15 | +0x1da | —（停止点后） |
| ㉞ | int32 | 版本 ≥0x16 | +0x244 | —（停止点后） |
| ㉟ | short | 版本 ≥0x17 | +0x1be | —（停止点后） |
| ㊱ | CInternalLayerBase::ReadPreJoinDefExpressions（LayerLib.dll `0x786c5d30`：int32 计数 + count×[String+String]） | 版本 >0x17 | -0xa8 段 | —（停止点后） |
| ㊲ | int32 + short ×3 | 版本 ≥0x19 | +0x2c0/+0x2c4/+0x2c6/+0x2c8 | —（停止点后） |
| ㊳ | String ×4 | 版本 ≥0x19 | +0x2cc/+0x2d0/+0x2d4/+0x2d8 | —（停止点后） |
| ㊴ | LoadObject({A62B9EC2-…}=IDisplayExpressionProperties) | 版本 >0x19 | 槽 | —（停止点后） |
| ㊵ | short + String ×3 | 版本 >0x1a | +0x300/+0x304/+0x308/+0x30c | —（停止点后） |
| ㊶ | TimeData 族四段（LayerLib.dll `0x7870d8b0`/`0x7870df80`/`0x7870e250`/`0x7870f6c0`，本轮闭合：TimeData 版本 ≤2 short+short+槽+（≥2）short；TimeTable 版本 ≤1 String×4+槽；TimeDimension 版本 ≤1 String×2；TimeDisplay 版本 ≤1 short+double+int32+double+int32） | 版本 ≥0x20 | -0x68/-0x80/-0x58/-0x48 段 | —（停止点后） |
| ㊷ | LoadObject({E720618B-…}=IHotLinkExpressionProperties) | 版本 >0x20 | 槽 | —（停止点后） |
| ㊸ | short | 版本 >0x21 | +0x2ca | —（停止点后） |

版本 0x1c..0x1f 分支（`0x1b < ver && ver < 0x20`，10.1 docVersion 不写）：short +
String ×4 + short + double + int32（→ -0x62/-0x78/-0x74/-0x70/-0x6c/-0x44/-0x40/
-0x38）→ 版本 ≥0x1d LoadObject({F6CBA4A3-…}=ITimeReference)、版本 ≥0x1e
LoadObject({BD724B95-…}=ITimeExtent)、版本 ==0x1f LoadObject({E720618B-…})。字段
序静态闭合，无样本字节佐证（实现按门接入、槽位新对象为停止点并注明）。

中间的 `FUN_6f6e0c00`（+0x144 GUID 后）、`FUN_6f6c8100`、`FUN_6f6acc10`、
`FUN_6f701ad0`、`FUN_6f6dd210`、`FUN_6f6fd550`（MakeBSTR+QI）、`FUN_6f7025b0`/
`FUN_6f6df550`（选择集刷新）经反编译核对均不读流，无流内字节。

### EditTemplate 宿主裁决（撤销 +0x218 假设；证据：确定的）

- **扩展段格式**（LayerLib.dll `ExtensionHelper::LoadExtensions` `0x786c22e0`，
  FeatureLayer.dll 导入）：[int32 计数][每条 [int64 尺寸][16 字节 CLSID][4 字节
  键][体]]。已知类由 CLSID 建实例后调 IPersistStream::Load；未知类按尺寸
  `IStream::Seek` 整体跳过。尺寸字段 = 「类标识+键+体」全段字节数（不含尺寸字
  段本身；样本条目：声明 387，头 20 + 体 367 恰等）。对称 SaveExtensions。
- **EditTemplateManager 体**（FeatureLayer.dll `0x6f6d0a80`；GetClassID
  `0x6f6d0a50` 写出 `{A0162E85-E170-4B10-A370-CEE08D8D4B8C}`；esri.clsid 声明
  progID EditTemplateManager；esriCarto coclass）：① int16 版本（`2 < (short)ver`
  拒绝；样本 2）→ ② int32 计数 → ③ 循环 LoadObject(IEditTemplate
  `{0952F6C1-…}`；样本 = EditTemplate 新对象，键 76 @`0xfed`) → ④ 版本 ≥2：
  1 字节（样本 0x00）。Save `0x6f6d0bc0` 对称（版本表按 docVer 查询）。
- 样本对位：扩展段 @`0xfcb` = 计数 1、尺寸 387、CLSID {A0162E85} @`0xfd7`、体
  `0xfe7..`（版本 2、计数 1、EditTemplate 键 76、尾字节 0x00），逐字节吻合。
- **结论**：样本 10 个 EditTemplate 实例（键 76/146/1605/…）的宿主 = 所在图层
  FeatureLayer 后继段图层扩展段中的 EditTemplateManager。第八轮「键序 75→76 →
  紧随 FeatureIDSet 的 SaveObject(+0x218) 槽」假设撤销：该槽（⑯）请求 IID
  `{19F85377-…}`=IDisplayFilter，样本空标记。实例前的 `02 00 01 00 00 00` 尾段
  = ETM 体首（版本 2 + 计数 1），未定名问题闭合。
- 接线：`annotate_props::read_edit_template_manager_body_10_1` 循环内接
  `edit_template::read_edit_template_body_10_1`（既有读取器，无停止点）。

### 样本对位（辅助观察）与实现接线

- `0xafe..0xb8e` 逐字节吻合（temp/agent-fl-tail3/ 走查脚本全链走查；ALPC 头+
  版本+计数 → LELP 头+版本 7 全标量段 → ITextSymbol 槽头）。停止点 = LELP ⑯ 槽
  TextSymbol `{B65A3E74-2993-11D1-9A43-0080C7EC5C96}` 新对象（键 61，对象头
  @`0xb7a`、体起点 @`0xb8e`）。`cargo run -- parse samples/测试底图.mxd` 停止点
  由 `0xafe`（剩余 150127 字节）推进到 **`0xb7a`**（剩余 150003 字节），停止原因
  `Successor(AnnotatePropsBody(LelpBody(SymbolSlotUnsupported)))`。
- `0xf63..0xfe6` 段（FIDSet、选择集、IDisplayFilter/IName/Envelope 空标记、关系
  类计数 0、String、范围计数 0、扩展段头）由走查脚本独立证实（读取器在 TextSymbol
  槽停止，暂不可达；TextSymbol 闭合后可达）。
- 实现：`feature_layer_tail::read_feature_layer_successor_10_1` 原位扩展（⑪-㊸
  全部门接入）；`SuccessorStop10_1` 扩展（TailCompleted/FeatureIdSetUnsupported/
  DisplayFilterUnsupported/NameUnsupported/RelationshipClassUnsupported/
  EnvelopeUnsupported/LayerExtensionUnsupported/EditTemplateManagerBody/
  SymbolSlotUnsupported/UidUnsupported/ExpressionPropsUnsupported/
  LegacyTimeSlotUnsupported/TimeSlotUnsupported；FieldInfoLoopCompleted 保留为
  「延续段流耗尽/ALPC 槽头非 ALPC」的兼容语义——停止点与第八轮一致，仅停止原因
  不再细分，既有调用方断言兼容）；`annotate_props` 三读取器与 `feature_id_set`
  读取器（trial 游标 + 暂存登记表，槽位未知类回退不保留键；ETM 尺寸声明与实际
  消费不一致为显式错误——原生不校验，注明较严）。
- tests/feature_layer_tail.rs 48 → 54 项（新增 tail_continuation 模块 6 项：样
  本形态停止、全链合成消费、ALPC/LELP/FeatureIDSet/ETM 契约、截断扫描、尺寸不
  一致事务性）。tests/robustness.rs 未动；`src/maps/`、`src/objects/` 未动
  （`mxd_maps` 的延续段诊断输出与 robustness 集中清单接入点留给整合轮：
  `read_annotate_props_collection_body_10_1`、`read_label_engine_props_body_10_1`、
  `read_edit_template_manager_body_10_1`、`read_feature_id_set_body_10_1` 与后继
  段新停止变体均已随 `read_feature_layer_successor_10_1` 自动生效，无需额外接线）。

### 本轮未解决事项

- **TextSymbol 体未闭合**（DisplayCore.dll；CLSID `{B65A3E74-…}` 数据常量
  `0x6ec167a8`、GetClassID 实体 `0x6ebd08e0` 与调整器 thunk `0x6ebe5bfb` 已定
  位，但 IPersistStream 虚表在本轮静态检索中无指针可达路径——无立即数三方互
  证形、无 RTTI COL 引用其 TD `0x6ec3e5dc`）：样本停止点即此（键 61）。其后
  的 BOLP（`{EE535289-…}`，LabelPlacement.dll，CLSID 常量 `0x7857a814`，实现
  函数未定位）、标注表达式引擎族、LELP ⑲/㉔ 槽 Envelope/IGraphicsContainer
  实现类、㉖ ISymbol 槽、㉘/㉙ Envelope/IUID 实现类、㊴/㊷ 表达式属性族、
  TimeData 族槽实现类均在停止点之后，体未闭合（读取器新对象即停止）。
- 版本 0x1c..0x1f 分支与 TimeData 族仅有反编译证据，无样本字节佐证（样本 34 走
  ≥0x20 门，0x1c..0x1f 分支不可达）。
- ALPC/LELP/ETM/FeatureIDSet 各标量与字符串的业务语义为候选（标注表达式、类
  名、OID 等），原样保留；ETM 尾字节（样本 0x00）语义未定。
- LELP 条目键（ALPC ③ 的 int32）与 ALPC 尾 int32 在样本中未达（LELP 体内停
  止），字段序仅有反编译证据。
- 全部结论为静态取证，未运行 ArcMap 10.1，未人工目视验收；样本对位为辅助观察。

### 接线轮（2026-11-15，第二轮：TextSymbol 体分派）

第九轮遗留的 LELP ⑯ ITextSymbol 槽已接体：新对象类标识为 TextSymbol
（`{B65A3E74-…}`）时经 `element_body::read_text_symbol_body_10_1` 整体解析
（`annotate_props.rs` 新 `LelpTextSymbolSlot10_1` 槽枚举与
`LelpStop10_1::TextSymbolBody { key, reason }` 透传变体；槽键按「登记先于
持久化」保留）；其余 ITextSymbol 实现类维持 `SymbolSlotUnsupported` 停止回
退。停止链携带体类型，`SuccessorStop10_1`/`FeatureLayerStop10_1` 去
`Copy`/`Eq`。样本重走：停止点 0xb7a（LELP 槽 TextSymbol 头，余 150003 字
节）推进至 **0xd6c**（余 149505 字节，+498），新停止 = TextSymbol 体内
IFontDisp 槽 OLE StdFont `{0BE35203-8F91-11CE-9DE3-00AA004BB851}` 新对象
（键 68；GUID 由诊断 class_id 字节 [3,82,227,11,145,143,206,17,157,227,0,170,0,75,184,81]
按内存序解码）——与第九轮「虚表静态不可达」的 TextSymbol 体未闭合判断吻合（体
字段序已由元素体轮闭合并接线，字体槽实现类 OLE StdFont 在 DisplayCore 之
外，仍未闭合）。`mxd_maps` 诊断同步透传（标注环逐条诊断 + IHyperlink 消费
计数）；FieldInfo INumberFormat 槽四类分派与后继段 IHyperlink 循环接体的落
位记录见 number-format/hyperlink 两文档文末。
