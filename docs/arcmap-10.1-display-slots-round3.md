# ArcMap 10.1 DisplayCore 槽实现类族与元素体版本分支（G 轮）：SimpleTextPath 负结果、Callout/背景族、IMarkerBackground、IColorRamp 族、PictureFillSymbol、TextElement ==7/==9、SymbolBorder/Background/Shadow

记录日期：2026-11-16。对应 TODO「MXD 解析」节「体未闭合类族」的 11 个开放
项（SimpleTextPath 体、Callout/背景族体、IMarkerBackground 实现类体、
IColorRamp 族实现类体、IPicture 实现类体、IPictureFillSymbol `{D842B082}`
实现类体、元素体版本 ==7 装饰块、元素体版本 ==9 尾加块、IBorder 槽、
IBackground 槽、IShadow 槽实现类体）。全部为静态反编译观察，未运行
ArcMap 10.1，未做人工目视验收；**仓库参考样本 `samples/测试底图.mxd` 中本
轮全部类 0 实例（无字节佐证）**，先例见 docs/arcmap-10.1-dataset-name.md。
不写本机绝对路径。

## 方法与文件身份

- 本机 pyghidra（Ghidra + JDK 25，脚本与产物在代理工作区
  `temp/agent-g-display/`），DisplayCore.dll 复用前轮工程缓存副本（独立锁，
  活工程路径不入库），GraphicElements.dll 与 CartoXProvider.dll 本轮自介质
  新提取并新建工程。
- 目标件身份（pefile 逐项复核，与既有记录一致）：
  - **DisplayCore.dll** 10.1.0.3035，SHA-256 `9b3e61f9ca553f14…a926e4d2`，
    ImageBase `0x6eb30000`；
  - **GraphicElements.dll** 10.1.0.3035，SHA-256
    `f24451c8bf25cce7…51f77ae9`，ImageBase `0x75030000`（与 element-bodies
    轮提取物逐字节同一文件，本轮从 `Desktop/Setup1.cab` 重新提取）；
  - **CartoXProvider.dll** 10.1.0.3035，SHA-256
    `1c4a6d3681e7527c…dd6f3cf5`，ImageBase `0x6ce00000`（同上，重新提取）。
- 定位链（对每类独立执行，证据强度：**确定**）：RTTI 类型描述符名串
  （`.?AV<类名>@@`）→ TD → RTTI 类层次描述符 CHD（COL+0x10，基类数组逐项
  展开，本轮上限 128 基类）→ 全层级 COL（判据 `[C]==0` 且 `[C+0xc]==TD` 且
  `[C+8] ≤ 0x10000`）→ 指向 COL 的双字 → 具体虚表 → **IPersistStream 子虚
  表认定：+0x0c 槽经 thunk 链（Ghidra thunk 标记 / `E9 rel32` 单转 /
  `8B 4C 24 04 2B 49 FC 89 4C 24 04 E9 rel32` 本地调整器桩）跟随到实体后，
  在实体自身字节内命中该类 `.rdata` CLSID 16 字节常量地址立即数（MOVQ XMM
  读常量形态）或 tail 4 字节立即数**。槽位 +0x14/+0x18 = Load/Save，Save 侧
  版本常量（直接常量或 `AfStreamGetSaveVersion` 表）逐字节读核。
- 与前轮记录的两处方法学差异（本轮以自有反编译为准，不直接改旧文档）：
  1. **「GetClassID 立即数写出」在本轮 9 类中均为「读 `.rdata` CLSID 常量
     写出参数」形态**（`MOVQ XMM0, qword ptr [常量]`），非指令内嵌立即数；
     前轮「两个 8 字节/四个 4 字节立即数」表述对应的是 tail 部分的写立即数
     形态与 .rdata 常量读取的混合，本轮定位统一以「实体字节内命中 CLSID
     常量（地址立即数或值立即数）」为准。
  2. **七槽主虚表族的本体化**：前轮（element-bodies）将「GetClassID 位于
     +0x14 的七槽表」排除在 IPersistStream 之外；本轮证实该表是对象偏移 0
     的 COM 身份表（[QI, AddRef, Release, 类名 getter, 析构, GetClassID,
     IsDirty 桩]），其 GetClassID 与真 IPersistStream 子虚表的 GetClassID
     常常是同一实体（如 RandomColorRamp `0x6eb3c4f0`），因此**不能以「出现在
     七槽表」为由否定一个 GetClassID**，IPersistStream 子虚表以「+0x0c 槽
     命中 CLSID + +0x10/+0x14/+0x18/+0x1c = IsDirty/Load/Save/GetSizeMax 四
     槽形状」认定（SimpleLineSymbol 的真子虚表 `0x6ec15498`、COL 偏移
     0xd4=212 与 display-tails 轮记录互证）。

## 一、SimpleTextPath 体：不可闭合（原生不实现 IPersistStream；负证据闭合）

**SimpleTextPath** `{B65A3E76-2993-11D1-9A43-0080C7EC5C96}`（esri.clsid 声明
DisplayCore.dll；类型库 dump coclass #357）的 RTTI 类层次（TD `0x6ec3e418`，
CHD 10 项）为：SimpleTextPath → Unknown（Esri 内部基类）→ IESRIInternalUnknown
→ UnknownErrorBase → **ITextPath**（对象偏移 0x10）→ IUnknown →
**ITextPath2**（偏移 0x14）→ IUnknown → **IPersist**（偏移 0x18）→ IUnknown。
全层级 COL 展开后仅有五张具体虚表：偏移 0 的七槽身份表（其 GetClassID 实体
`0x6ebc41e0` 读常量 `0x6ec0d4ac`）、偏移 4 的两槽桩族、偏移 0x10 的 12 槽
ITextPath 表、偏移 0x14 的 4 槽 ITextPath2 表、偏移 0x18 的 4 槽 IPersist 表
（GetClassID `0x6ebc4ba0`，读同一常量）。**全层级不存在 IPersistStream 形状
虚表**。同族 **BezierTextPath** `{2DE21000-BDEB-11D1-970B-0080C7E04196}`（
TD `0x6ec3cc20`）同样只有身份表（GetClassID `0x6eb49400` 一系）与接口表，
无 IPersistStream。

- **结论（证据强度：确定的反编译负证据）**：SimpleTextPath/BezierTextPath
  不实现 IPersistStream，`ObjectStream::LoadObject` 对其必然失败——**流内不
  可出现**，对象体无格式可闭合；读取器不实现（也无须实现）。TextSymbol 的
  ITextPath 槽（`0x6ebd0a60` 字段 10）能承载的流内实现类只能是实现了
  IPersistStream 的其他 TextPath（LabelPlacement.dll 的 WordTextPath/
  OverposterTextPath 等，不在本轮范围），样本该槽为空标记。
- 旁证：TextSymbol 版本门尾段的内置默认 TextPath GUID
  `{D7099B91-E298-499E-9757-72BB39E55CF3}` 经类型库 dump 对出 coclass
  **SimpleTextParser**（esriDisplay #375）——TextSymbol 以「原样 CLSID 字段
  + 运行期实例化」而非流内对象承载默认 TextPath，与上述负证据自洽。
- 同样负结果：**ColorRampElements** `{CCDBF37E-7A63-4788-A7CB-2A5B0E98A180}`
  （esri.clsid DisplayCore 条目，RTTI 层级 14 项）全层级无 IPersistStream
  形状虚表——同为不可流内持久化类，不实现读取器。

## 二、Callout/文字背景族（证据强度：确定；样本 0 实例）

| 类 | CLSID | 虚表 | GetClassID | Load | Save | Save 版本 |
| --- | --- | --- | --- | --- | --- | --- |
| BalloonCallout | `{C8D09ED2-4FBB-11D1-9A72-0080C7EC5C96}` | `0x6ec0cdf4` | `0x6eb41b70`（读常量 `0x6ec0cf1c`） | `0x6eb41950` | `0x6eb41a70` | 恒 1（`DAT_6ec0b310`） |
| LineCallout | `{C8D09ED3-…}` | `0x6ec0e3ec` | `0x6eb59300`（读常量 `0x6ec0d6e4`） | `0x6eb58fd0` | `0x6eb59170` | 恒 1（`DAT_6ec0b2f0`） |
| SimpleLineCallout | `{FA37B822-A959-4ACD-834A-0E114BF420B8}` | `0x6ec10bc8` | `0x6eb82730`（读常量 `0x6ec10c00`） | `0x6eb82580` | `0x6eb82670` | 恒 1（`DAT_6ec0b294`） |
| MarkerTextBackground | `{C5C02D50-7282-11D2-9816-0080C7E04196}` | `0x6ec0ee34` | `0x6eb644b0`（读常量 `0x6ec0ee74`） | `0x6eb64380` | `0x6eb64440` | 恒 1（`DAT_6ec0b2dc`） |

**BalloonCallout::Load `0x6eb41950`**（版本 `1 < (short)ver` 拒绝）：

| # | 字段 | 原语 | 去向 | 证据 |
| - | ---- | ---- | ---- | ---- |
| ① | 版本 short | 2 字节 | — | Load `0x6eb41950` |
| ② | int32 | 4 字节 | this−0x7c（样式候选 esriBalloonCalloutStyle） | 同上 |
| ③ | double ×4 | 8×4 字节 | this−0x4c / −0x3c / −0x44 / −0x34（边距候选） | 同上 |
| ④ | LoadObject（ISymbol `DAT_6ec0ca2c`） | 20 头 + 体 | Save 槽 this−0x88（填充符号） | 同上（Save 对称） |
| ⑤ | LoadObject（**IPoint** `{1B077915-38C5-11D0-92D2-00805F7C28B0}`，`DAT_6ec0ce6c`） | 20 头 + 体 | Save 槽 this−0x68（锚点） | 同上 |
| ⑥ | double | 8 字节 | this−0x2c（圆角候选） | 同上 |

**LineCallout::Load `0x6eb58fd0`**（版本 ≤1）：

| # | 字段 | 原语 | 去向 | 证据 |
| - | ---- | ---- | ---- | ---- |
| ① | 版本 short | 2 字节 | — | Load `0x6eb58fd0` |
| ② | LoadObject（ISymbol）×3 | 20 头 + 体 ×3 | Save 槽 this−0xbc / −0xb8 / −0xb4（边界/强调条/引线符号候选，按 Save 序对位） | 同上 |
| ③ | LoadObject（IPoint `DAT_6ec0ce6c`） | 20 头 + 体 | Save 槽 this−0x84（锚点） | 同上 |
| ④ | double | 8 字节 | this−0xac | 同上 |
| ⑤ | int32 | 4 字节 | this−0xa4（样式候选 esriLineCalloutStyle） | 同上；**Save 在 docVer==0 且值为 6/7 时改写 1**（历史样式号映射，流内值原样保留） |
| ⑥ | double ×5 | 8×5 字节 | this−0x5c / −0x54 / −0x4c / −0x44 / −0x3c（边距候选） | 同上 |

**SimpleLineCallout::Load `0x6eb82580`**（版本 ≤1）：

| # | 字段 | 原语 | 去向 | 证据 |
| - | ---- | ---- | ---- | ---- |
| ① | 版本 short | 2 字节 | — | Load `0x6eb82580` |
| ② | double | 8 字节 | this−0x2c | 同上 |
| ③ | short | 2 字节 | this−0x24 | 同上 |
| ④ | LoadObject（**IGeometry** `{1B07790D-38C5-11D0-92D2-00805F7C28B0}`，`DAT_6ec0ce5c`） | 20 头 + 体 | Save 槽 this−0x20（锚几何） | 同上 |
| ⑤ | LoadObject（**ILineSymbol** `{E6BDAA7D-…}`，`DAT_6ec0d334`） | 20 头 + 体 | Save 槽 this−0x1c（引线符号） | 同上 |
| ⑥ | LoadObject（IPoint `DAT_6ec0ce6c`） | 20 头 + 体 | Save 槽 this−0x18（锚点） | 同上 |

**MarkerTextBackground::Load `0x6eb64380`**（版本 ≤1）：

| # | 字段 | 原语 | 去向 | 证据 |
| - | ---- | ---- | ---- | ---- |
| ① | 版本 short | 2 字节 | — | Load `0x6eb64380` |
| ② | short | 2 字节 | this−0x1c | 同上 |
| ③ | LoadObject（ISymbol `DAT_6ec0ca2c`） | 20 头 + 体 | Save 槽 this−0x2c（标记符号） | 同上（Save 对称） |

## 三、IMarkerBackground 实现类 = MarkerTextBackground（闭合）

IMarkerBackground `{AE41E4F0-DDE7-11D3-8216-0080C79F0371}`（MultiLayerMarker
v≥2 槽请求 IID）的流内实现类：esri.clsid 全表检索仅 **MarkerTextBackground**
（DisplayCore）一个候选 coclass（`MarkerBackground` 名称检索无其他 ArcGIS
条目）；本轮将其体闭合（上表第四行，体仅 short + 标记符号槽）。其余可能的
IMarkerBackground 实现类无静态线索，槽分派对未知实现类维持停止。

## 四、IColorRamp 族（GradientFill 槽；四类闭合）

| 类 | CLSID | 虚表 | GetClassID | Load | Save | Save 版本 |
| --- | --- | --- | --- | --- | --- | --- |
| RandomColorRamp | `{BEB87094-C0B4-11D0-8379-080009B996CC}` | `0x6ec0c558`（COL `0x6ec1a0b4`，接口偏移 0x54） | `0x6eb3c4f0`（读常量 `0x6ec0c588`） | `0x6eb3c190` | `0x6eb3c350` | 恒 1（`DAT_6ec0b31c`） |
| MultiPartColorRamp | `{BEB87099-…}` | `0x6ec0c19c` | `0x6eb39860`（读常量 `0x6ec0c230`） | `0x6eb395b0` | `0x6eb39720` | 表 `DAT_6ec0b324`（5 项短整）= docVer 0..4 → **1,1,1,1,2** |
| PresetColorRamp | `{BEB8709A-…}` | `0x6ec0c378` | `0x6eb3a6e0`（读常量 `0x6ec0c3b8`） | `0x6eb3ae20` | `0x6eb3afa0` | 恒 1（`DAT_6ec0b320`） |
| AlgorithmicColorRamp | `{BEB8709B-…}` | `0x6ec0b468` | `0x6eb31010`（读常量 `0x6ec0b544`） | `0x6eb316c0` | `0x6eb317f0` | 恒 1（`DAT_6ec0b358`） |

> 后续实档及 AfCore 指令复核：以下四类的名称用自由函数 ReadString/WriteString 的 BSTR 布局，长度为 UTF-16 码元数。名称槽的早期 String::Read 归类已更正，见[唯一值渲染器续查](mxd-us-media-renderer-paragraph-20261118.md)。

**RandomColorRamp::Load `0x6eb3c190`**（`2 < (short)ver` 拒绝）：

| # | 字段 | 原语 | 去向 | 证据 |
| - | ---- | ---- | ---- | ---- |
| ① | 版本 short | 2 字节 | — | Load `0x6eb3c190` |
| ② | 名称 | AfCore ReadString（BSTR） | this−0x30（名称候选） | 同上 |
| ③ | int32 | 4 字节 | this−0x34（**Load 读入后内存槽置 0**——反编译观察的「读取即弃」形态；Save 写槽值；流内 4 字节原样保留） | 同上；Save `0x6eb3c350` 写 this−0x34 |
| ④ | 2 字节 short | 2 字节 | 全 16 位与零比较，结果写入 bool this−0x28；见[第四轮指令复核](mxd-round4-display-20261117.md) | 同上 |
| ⑤ | int32 | 4 字节 | this−0x2c（Seed 候选） | 同上 |
| ⑥ | short ×6 | 2×6 字节 | this−0x26（装载同写 this−0x24 同值）/ −0x22 / −0x20 / −0x1e / −0x1c / −0x1a（IRandomColorRamp 色相/饱和度/明度端点候选） | 同上 |

**MultiPartColorRamp::Load `0x6eb395b0`**（`3 < (short)ver` 拒绝）：

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节 | — | — | Load `0x6eb395b0` |
| ② | 名称 | AfCore ReadString（BSTR） | this−0x44（名称候选） | — | 同上 |
| ③ | int32 count | 4 字节 | this−0x24（Save 序） | — | 同上 |
| ④ | count × LoadObject（**IColorRamp** `{BEB87091-…}`，`DAT_6ec0b524`） | 20 头 + 体 | 子梯度槽（Save 槽 this−0x28 数组顺序写出，Load 顺序消费，**不倒序**） | — | 同上 |
| ⑤ | count × 8 字节 double | 8×count 字节 | this−0x14 缓冲（尺寸候选） | Load 版本 ≥2；Save docVer ≥4（**读写门不对称**，按各自反编译实现） | 同上；Save `if docVer < 4 跳过` |

**PresetColorRamp::Load `0x6eb3ae20`**（`2 < (short)ver` 拒绝）：版本 short →
String → int32 → int32 count → count × LoadObject（**IColor**
`DAT_6ec0b4f4`；Save 槽 this−0x14 数组）。

**AlgorithmicColorRamp::Load `0x6eb316c0`**（`2 < (short)ver` 拒绝）：版本
short → String → this−0x20 → int32 → this−0x1c → LoadObject（IColor）×2
（Save 槽 this−0x28 / this−0x24，起止颜色候选）。

## 五、IPicture 实现类：OLE StdPicture（既有闭合 + 本轮 Esri 侧负证据）

- IPicture `{7BF80980-BF32-101A-8BBB-00AA00300CAB}` 的流内实现类即 OLE
  **StdPicture** `{0BE35204-8F91-11CE-9DE3-00AA004BB851}`（oleaut32），体已由
  docs/arcmap-10.1-ole-stdfont.md 轮闭合、`display_symbol.rs` 的四处
  PictureMarker 槽已接线。
- **本轮 Esri 侧负证据**：IPicture IID 16 字节在 DisplayCore 仅两处 `.rdata`
  常量——`0x6ec0b08c`（LoadObject 请求，PictureMarker/Fill 用）与
  `0x6ec10010`（消费方 QI 助手，引用函数 `0x6eb755f0`/`0x6eb7deb0` 均为对
  传入对象查询该 IID 的消费者，非实现）；GraphicElements/CartoXProvider 中
  0 处；三模块 `.text`/`.rdata` 均无 StdPicture CLSID 常量；esri.clsid 全表
  无实现 IPicture 的 Esri coclass。**结论：PictureMarker v9 类型 3 / 旧路径
  各 IPicture 槽无 Esri 侧实现类，实现类族 = StdPicture 一种（已闭合）**，
  该 TODO 项以既有闭合 + 本轮负证据闭合。
- PictureFillSymbol 的 IPicture 槽同上（见下节类型 3）。

## 六、PictureFillSymbol `{D842B082-330C-11D2-9168-0000F87808EE}`（闭合）

虚表 `0x6ec0febc`（COL `0x6ec1db94`）；GetClassID `0x6eb710d0`（读常量
`0x6ec0ffd4`）；Load `0x6eb71110`；Save `0x6eb71580`。版本门 `8 < (short)ver`
拒绝；**Save 版本表 `DAT_6ec0b2ac`（6 项短整，逐项字节读核）= docVer 0..5 →
3,3,3,4,7,8**——10.1（Current）恒写版本 8，版本 ≤7 为旧文档历史路径（Load
旧路径 `0x6eb74aa0`、Save 旧路径 `0x6eb74740`）。

**版本 ==8（主路径；Load `0x6eb71110`）**：

| # | 字段 | 原语 | 去向 | 版本门 | 证据 |
| - | ---- | ---- | ---- | ------ | ---- |
| ① | 版本 short | 2 字节 | — | — | Load `0x6eb71110` |
| ② | int32 图像类型 | 4 字节 | this−0x10（==1 路径 / ==2 位图块 / ==3 IPicture；越界拒绝 `0x80004005`） | ==8 | 同上 |
| ③ | int32 | 4 字节 | this−0x14 | ==8 | 同上 |
| ④a | 类型 1：String + double ×2 | String::Read + 8×2 字节 | this−0x20（路径候选）+ this−0x4c / −0x44 | ==8 | 同上 |
| ④b | 类型 2：int32 N +（N≠0 时 N 字节） | AfStreamRead ×2 | 内存块（Save `Ordinal_23` 取尺寸对称写） | ==8 | 同上 |
| ④c | 类型 3：LoadObject（**IPicture** `DAT_6ec0b08c`） | 20 头 + 体 | Save 槽 this−0xa4 | ==8 | 同上 |
| ⑤ | LoadObject（IColor `DAT_6ec0b4f4`）×3 | 20 头 + 体 ×3 | Save 槽 this−0xa0 / −0x9c / −0x98（颜色候选） | ==8 | 同上 |
| ⑥ | LoadObject（ISymbol `DAT_6ec0ca2c`）×1 | 20 头 + 体 | Save 槽 this−0x94（轮廓符号候选） | ==8 | 同上 |
| ⑦ | double ×7 + double ×2 | 8×9 字节 | this−0x8c / −0x84 / −0x7c / −0x74 / −0x6c / −0x64 / −0x5c + 2 个读后即弃（Save 先置零再计算尺寸，见 [后续指令复核](mxd-terra-host-color.md)） | ==8 | 同上 |
| ⑧ | int32 ×2 | 4×2 字节 | this−0x114 / −0x10c | ==8 | 同上 |
| ⑨ | byte | 1 字节 | this−0x38 | ==8 | 同上 |
| ⑩ | short ×3 | 2×3 字节 | this−0x26 / −0x24 / −0x22 | ==8 | 同上 |

**版本 ≤7（旧路径 `0x6eb74aa0`，字段序反编译记录；无样本）**：版本 <6：
LoadObject（IPicture）→ 图像对象槽；版本 ≥6：String（路径候选），串空 →
LoadObject（IPicture），串非空 → double ×2 → 随后 IColor ×3 + ISymbol ×1 +
double ×7 + double ×2（读后即弃）+ int32 ×2 + 版本 >1 byte + 版本 >3
short ×3 + 版本 >6 int32 N +（N≠0 时 N 字节位图块）。与 PictureMarker 历史
路径（display-symbols-round2 记录）同族同型。

## 七、元素体版本 ==7 装饰块 / ==9 尾加块（CartoXProvider 复核；闭合）

TextElement::Load `0x6ce70e50`（CartoXProvider.dll，本轮重新提取后反编译互
证，与 docs/arcmap-10.1-element-bodies.md 记录逐字段一致）：

- **版本 ==7 精确块**（`local_8[0] != 7` 跳过）：LoadObject（请求 IID 常量
  `0x6cee3f38` = **IBorder** `{E4240307-DD41-11D2-B868-00600802E603}`）→
  LoadObject（`0x6cee3f48` = **IBackground** `{E4240309-…}`）→ LoadObject
  （`0x6cee3f58` = **IShadow** `{A9258348-1753-4B99-820A-FCF74429ABC1}`）→
  double（8 字节）→ int32（4 字节）→ double（8 字节）。三个请求 IID 逐字节
  读核，与 element-bodies 轮记录一致。
- **版本 ==9 尾加块**（`==9` 置标志后按 8 处理，尾加在 8 路径末尾）：
  LoadObject（`0x6cedd170` = **ISpatialReference**
  `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`）→ int32 → this+0x4d → 32 字节
  原样 → this+0x53（WKSEnvelope 形态）。
- Save `0x6ce711c0`：版本表 `DAT_6ced7ef8` 逐字节读核 = **docVer 0 → 6、
  docVer ≥1 → 8**（10.1 恒写 8），无 ==7/==9 块——两块是旧文档历史形态，
  读取器按「分类槽 + 标量」覆盖（已由 element_body.rs 实现），本轮闭合的
  是**槽内实现类**（见下节）。
- ==9 块的 ISpatialReference 槽：闭合实现类 = ProjectedCoordinateSystem
  （既有 `spatial_reference` 读取器）；GeographicCoordinateSystem/
  UnknownCoordinateSystem（Geometry.dll）仍未闭合，保留在 TODO
  「ISpatialReference 槽实现类体」项（不在本轮 11 项内）。

## 八、SymbolBorder / SymbolBackground / SymbolShadow（GraphicElements；闭合）

| 类 | CLSID | 虚表 | GetClassID | Load | Save | Save 版本 |
| --- | --- | --- | --- | --- | --- | --- |
| SymbolBorder | `{A5D0F017-62DD-11D2-87BE-0000F8751720}` | `0x75072c04` | `0x7505ec40`（读常量 `0x750710d8`） | `0x7505ec70` | `0x7505ed40` | 恒 2（`DAT_7506e7b8`） |
| SymbolBackground | `{1BAA33E9-E13B-11D2-B868-00600802E603}` | `0x75072a64` | `0x7505c4e0`（读常量 `0x750710e8`） | `0x7505c510` | `0x7505c5e0` | 恒 2（`DAT_7506e7bc`） |
| SymbolShadow | `{A8861E66-57AA-47D0-AAF8-B288B4FD5240}` | `0x75072d74` | `0x750601b0`（读常量 `0x750710f8`） | `0x750601e0` | `0x750602b0` | 恒 1（`DAT_7506e7b4`） |

**SymbolBorder::Load `0x7505ec70`**（版本 `2 < (short)ver` 拒绝）：

| # | 字段 | 原语 | 去向 | 证据 |
| - | ---- | ---- | ---- | ---- |
| ① | 版本 short | 2 字节 | — | Load `0x7505ec70` |
| ② | double | 8 字节 | this−0x1c（尺寸候选，装载即复制到 this−0x14） | 同上 |
| ③ | LoadObject（**ILineSymbol** `{E6BDAA7D-…}`，`DAT_7507143c`） | 20 头 + 体 | Save 槽 this−0xc（边界线符号） | 同上（Save 对称） |
| ④ | short | 2 字节 | this−8 | 同上 |
| ⑤ | double | 8 字节 | this−0x14 | 版本 ≥2（Save 无条件写——版本表恒 2，读写门在流内不可区分） |

**SymbolBackground::Load `0x7505c510`**（版本 ≤2）：与 SymbolBorder 同构，
符号槽请求 IID 为 `DAT_7506ebcc` = **IFillSymbol** `{E6BDAA7E-…}`（背景填充
符号）。

**SymbolShadow::Load `0x750601e0`**（版本 `1 < (short)ver` 拒绝）：版本
short → LoadObject（IFillSymbol `DAT_7506ebcc`）→ Save 槽 this−0xc → short →
this−8 → double → this−0x1c → double → this−0x14（无版本门尾段；Save 恒写
版本 1）。

## 实现（src/objects/ 四个新模块；只读模块未动）

- `src/objects/color_ramp.rs`：四类常量 + 四个体读取器
  （`read_random_color_ramp_body_10_1` 等）+ 槽分派
  `read_color_ramp_slot_10_1`（GradientFill 主槽与 MultiPart 子梯度槽共用；
  MultiPart 递归分派，嵌套深度限额 16 为防栈溢出的读取器自身限额）。建议
  接入点：`display_symbol.rs` 的 `read_unclosed_slot_10_1`（`ColorRamp` 槽）
  改调本分派。
- `src/objects/callout_background.rs`：四类常量 + 四个体读取器 + 槽分派
  `read_callout_slot_10_1`（三 Callout 族，`allow_marker_background` 参数
  区分接口族）与 `read_marker_background_slot_10_1`（IMarkerBackground 槽，
  仅接受 MarkerTextBackground）。建议接入点：`display_symbol.rs`
  `read_unclosed_slot_10_1`（`MarkerBackground` 槽）改调
  `read_marker_background_slot_10_1`。
- `src/objects/picture_fill_symbol.rs`：类常量 + 体读取器
  `read_picture_fill_symbol_body_10_1`（版本 ≤8 全分支：==8 主路径三类型 +
  ≤7 旧路径）+ 槽分派 `read_picture_fill_symbol_slot_10_1`。建议接入点：
  `display_symbol.rs` 的 PictureLineSymbol 符号槽（IPictureFillSymbol 槽，
  现为 `UnclosedSlot10_1` 停止）改调本分派。
- `src/objects/element_decor.rs`：三类常量 + 三个体读取器 + 三槽分派
  `read_border_slot_10_1` / `read_background_slot_10_1` / `read_shadow_slot_10_1`。
  建议接入点：`element_body.rs` 的 `read_text_element_body_10_1` 版本 ==7 块
  三处 `read_classified_slot` 改调三槽分派（停止语义不变：未知实现类回退）。
- **本轮未改**：`display_symbol.rs`、`display_color.rs`、`element_body.rs`、
  `element_collection.rs`、`TODO.md`、`tests/robustness.rs`（集中清单归整合
  轮）、任何分派点/调用点；`src/objects/mod.rs` 仅追加四个 `pub mod` 声明行。
- 事务性与限额约定同仓内其他读取器：trial 游标/登记表、版本 short 有符号
  上限并拒绝负值位模式（较原生更严）、截断与非法长度明确报错、停止路径键不
  登记、`max_bytes` 约束字符串/循环计数/位图块。测试全在模块内
  `#[cfg(test)]`（四模块共 22 项合成用例），robustness 集中清单未并入（归
  整合轮）。

## 开放项（未解决事项）

- **无字节佐证**：本轮全部类（含负结果的 SimpleTextPath/BezierTextPath/
  ColorRampElements 的「流内不出现」结论）无样本实例；读取器仅以合成输入验
  证，不得声称样本验证。
- 三类「流内不出现」的负证据为静态反编译（RTTI 层级无 IPersistStream 虚
  表）；未运行目标软件证实 LoadObject 对其失败的行为。
- 各类 int32/double/short 的业务命名（Callout 边距/圆角/样式、ColorRamp 色
  相端点/种子等）为符号学惯例候选，读取器原样保留。
- RandomColorRamp ③ 的「Load 读入后内存槽置 0」与 PictureFillSymbol ⑦ 的
  两 double「读后即弃」为反编译级观察（Save 写槽值/恒写 0.0），与
  display-symbols-round2 记录的三处 short 同族 quirk，样本可达时核对。
- MultiPartColorRamp 尾段 double 的 Save 门（docVer ≥4）与 Load 门（版本
  ≥2）不对称，写侧取值按版本表推断（10.1 docVer 5 → 版本 2 → 写尾段）。
- SymbolBorder/Background 版本 ≥2 的 double 在 Save 侧无条件写（版本表恒
  2），版本 0/1 的流内形态无写侧来源（历史兼容分支，读取器已按门覆盖）。
- WordTextPath / OverposterTextPath（LabelPlacement.dll）等其余 ITextPath
  实现、MapElements.dll 的网格边框类（IBorder 族）不在本轮范围。
- 全部结论为静态反编译观察，未经 ArcMap 10.1 运行验证；人工目视验收未做。
