# PageLayout 对象尾段与 GroupElement（ArcMap 10.1）

本轮使用 PE32 x86、版本资源 10.1.0.3035 的 PageLayout.dll、GraphicElements.dll 和 CartoConverter.dll。PageLayout.dll 采用已有 Ghidra 导出并用 `objdump` 复核（SHA-256 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`）；GraphicElements.dll（`f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9`）和本轮从 Desktop/Setup1.cab 补提取的 CartoConverter.dll（`5982306cbe6564505a3c37b240e0ea75d5255c13b10865880938c2e051c1f264`）用注册表、RTTI、虚表、导入和原始指令交叉核对。下列地址是静态映像 VA。

| 顺序 | 10.1 原生依据 | 已接入 |
| --- | --- | --- |
| PageLayout 元素循环之后的五个对象 | PageLayout Load `0x7b6e4e30`（虚表 `0x7b71fa34`），Save `0x7b6e6a00`：按顺序请求 IPage、IRulerSettings、ISnapGrid、两次辅助线接口 | Page、RulerSettings、SnapGrid、SnapGuides 两槽，继续用当前引用表 |
| Page | CLSID `{DD94D76F-836D-11D0-87EC-080009EC732A}`；GetClassID `0x7b6aaea0`；Load thunk `0x7b6b11a0` 到 `0x7b6aaee0`，Save `0x7b6ab2f0` | v0..1 的纸张编号、方向、两个尺寸 double、范围、原始标量、颜色、边框和背景；仅保留存盘值，不模拟打印机环境导致的原生重算 |
| 标尺与捕捉 | RulerSettings Load `0x7b6f7530`，SnapGrid `0x7b6fac00`，SnapGuides `0x7b6fb590`；PageLayout 通过 `0x7b6ce300/0x7b6ce050` 获取 GraphicSnapEnvironment，再经 IPersistStream +0x14 直读 `0x7b6f9880` | 标尺间距、网格间距、辅助线、捕捉容差和两个 GUID 集合。直接委派无对象头；两个集合是裸 GUID，不能登记 ObjectStream 键 |
| 关联地图及视图标量 | PageLayout Load 后续 IMap 槽、short×4、一个 WKSEnvelope、四个 double；后者经 XMin/YMin/Width/Height 设置器写入布局范围；版本门依次为 2、3、4、5、6、8、9 | IMap 空槽或已确认的根 Map 回引；视图范围、版面位置及宽高、版本标量按流内值保留 |
| 版本 ≥10 布局扩展 | PageLayout 调 ExtensionHelper；条目为 int64 体长、CLSID、体，无 ObjectStream 对象键。CartoConverter 的 MapOptimizationLayoutExtension GetClassID `0x6ccc0760`，虚表 `0x6cd77b34` 的 Load `0x6ccc0de0` 调 MappingCoreLib::LoadResults，再按 v2..6 读字符串、标量及对象槽 | 空准备结果、空对象槽以及 v2..6 的其余字段，核对消耗量与声明体长一致；其他实现或非空结果/对象槽保留未解析 |
| 版本 ≥11 摘要、12/13 旧分支、13/15 元素数组 | Load `0x7b6e4e30` 后半段：ExtraSafe 槽、仅 v12/v13 的两个 double、v13 起的 LONG 和逐元素 LONG 列表，v15 继续用同一计数读取 IEnvelope 槽与 double | 当前 ExtraSafe 空槽仅接受 A=B=0 或 A=16/B=0 加空对象头；两个数组按原生顺序且计数必须与元素列表相合 |
| 布局对象之外的流尾 | 文档读函数 `0x7b708d00` 在 ObjectStream +0x44 返回后用当前 IStream +0x0c 再读 2 字节。非零走辅助 `0x7b70e800`、IPaper 和 IID `{2AB49820-9406-11D0-87EF-080009EC732A}` 的 IPrinter | 读取打印设置标记及非零分支，支持范围见下文。独立于 PageLayout 对象体 |

GraphicElements.dll 的 GroupElement CLSID `{803577D2-F8A3-11D0-83AF-080009B996CC}` 由 GetClassID `0x75035e80` 与持久化虚表 `0x7506f770` 对位。Load `0x75035ef0` 先读 v0..3、委派 ElementProperties Load、读取计数及普通 IElement 对象列表；v3 还读取 IGeometry（IID 地址 `0x7506fae4`），最后无条件委派 BaseFrameElement Load（IAT `0x7506e260`）。已支持当前布局宿主中的嵌套组、矩形和通用元素，设递归上限并在有界体失败时回退整个组，避免登记未确认的子键。

66 份 10.1 安装介质工程均从 `/Maps` 顺序读到 `/PageLayout` 并完全消费其流：布局架构版本为 v9 三份、v10 三十四份、v14 十一份、v15 十八份，亦与保存版本表 `0x7b71c840` 的 `[7,9,9,10,14,15]` 相符。所有 66 份的对象体之后均是零打印标记。SDK LinearRef 的对象体在 `/PageLayout:0x48c` 结束；仓库参考工程在 `0xb15` 的 ExtraSafe 包装内读出 GroupElement 键 2208、两个子元素及 Polygon 几何，七个布局元素之后继续读取 PageLayout 对象尾部至 `0x1c62`，打印标记至 `0x1c64`。参考工程的其他未解数据（如 `/Maps` 中的有界栅格素材）仍按原样保留。

合成测试覆盖 v0..15 的尾部版本门、逐字节截断回退、计数不一致、引用表一致性和无法解析的非零打印设置内容保留。未遇到的扩展类、非空摘要槽和其他文档流仍待研究。旧版普通元素列表见下节。未在 ArcMap 10.1 或 QGIS 中人工打开验证。

## 非空打印设置

复用 PageLayout.dll `0x7b708d00` 的现有伪代码，检查其 PE 常量，并用 objdump、PyGhidra 补导 PrintOut.dll 和 ArcPressPlugin.dll 持久化入口。三文件 PE 均为 x86，资源中的 FileVersion 和 ProductVersion 均为 `10.1.0.3035`。

| 模块 | SHA-256 |
| --- | --- |
| PageLayout.dll | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` |
| PrintOut.dll | `ff9bdbf31a6d4c06573fa37e2da529217a93fec44c9715458bbaf1b4ac9a5bde` |
| ArcPressPlugin.dll | `547cbef6e256f78af2ed3518f534848039a6212600071cc11b3ea662226ca9a3` |

分流 `/PageLayout` 的标记非零时，`0x7b709518` 创建 `Paper`。`0x7b7207ec` 的原始字节 `63 4e 8c e2 55 3f d1 11 88 5e 00 00 f8 78 08 ee` 对应 CLSID `{E28C4E63-3F55-11D1-885E-0000F87808EE}`，`0x7b721678` 的 IPaper IID 为 `{387BC1D1-3F54-11D1-885E-0000F87808EE}`，两者与类型库及 PrintOut.dll 一致。`0x7b709526` 经 `0x7b70ea90 → 0x7b70ed00` 查询标准 IPersistStream，`0x7b709549` 直接调用 `+0x14` Load，纸张前无 CLSID 或键。`0x7b7095af` 才通过同一 ObjectStream 的 `+0x44` 读取带头的 IPrinter 槽。单流 `Mx Document` 的 `0x7b709105` 分支改走有对象头的 IPaper，属于另一宿主。

| 类 | 虚表及入口 | 持久化字段 |
| --- | --- | --- |
| Paper | IPersistStream `0x7bce0e20`；GetClassID `0x7bcd63f0`；Load `0x7bcd6c00`；Save `0x7bcd7070` | u16 版本 ≤3。v2 起有语言标识及 AfCore String 打印机描述。随后是 u32 长度加 DEVNAMES 字节，再是 u32 DEVMODE 分配大小和可选正文。v3 非空由 `0x7bcd6720` 读取 220 个固定成员字节，加 `dmDriverExtra`（`+0x46` 的 u16）个驱动字节，写侧 `0x7bcd71ce → 0x7bcd6430` 对位。 |
| EmfPrinter `{AE064D01-D6CE-11D0-867A-0000F8751720}` | IPersistStream `0x7bce073c`；GetClassID `0x7bcd1ce0`；Load `0x7bcd1d20`；Save `0x7bcd1d60` | 一个 u16 版本，最大 1。 |
| PsPrinter `{E28C4E61-3F55-11D1-885E-0000F87808EE}` | IPersistStream `0x7bce14ec`；GetClassID `0x7bcd7910`；Load 同 EmfPrinter；Save `0x7bcd8f30` | 一个 u16 版本，最大 1。 |
| ArcPressPrinter `{9E1B5153-1CA8-42B6-A7E0-4B215DAEA956}` | IPersistStream `0x1000e878`；GetClassID `0x10002130`；Load `0x10002180`；Save `0x100024f0` | u16 版本 ≤2、AfCore String、short、一个字节、AfCore BSTR 形式的驱动类字符串。v2 在 `0x100023a6` 读取 LONG 长度和驱动数据，三个版本均以五个 LONG 和一个字节收尾。 |

### 旧版 Paper 的分流读取

复用现有 PageLayout 和 SystemCore 反编译，补核对 PrintOut 原始指令及 Ghidra 函数 `0x7bcd6720`。SystemCore.dll 为 x86 `10.1.0.3035`，SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。

- PageLayout 在 `0x7b708d99` 用 CLSID `{043731D0-A7CF-11D1-8BD1-080009EE4E41}` 新建 ObjectStream。SystemCore 构造函数 `0x7ffcf030` 将版本文本初始化为空字符串，切换底流的 `0x7ffcf7a0` 只替换底流引用。分流打开链将该对象连接到 DrawingDefaults、Maps 与 PageLayout，并于 `0x7b709549` 将其作为 IStream 传给 Paper。该分流链未从 `/Version` 设置此实例的版本文本。
- PrintOut `0x7bcd675e` 查询 IObjectStream，`0x7bcd6782` 调用 `+0x50` 取版本文本（SystemCore `0x7ffd0500`）。在空文本下，Paper v0–v3 均读 220 字节的连续固定成员，再读 `dmDriverExtra` 字节。原生写入内存的驱动数据起点为 `dmSize`（`0x7bcd6bb2..0x7bcd6bc1`）。声明值是分配大小，须容纳固定成员和 `dmSize + dmDriverExtra`；流中的数据固定紧接在 220 字节后，两种长度分别核对。
- 若其他宿主显式将 IObjectStream 版本文本设为 `9.0`，v0–v2 省读内存偏移 `+0xb0` 的 DWORD（`0x7bcd6a1f`），v2 又多读一个 DWORD 到 `+0xc4`（`0x7bcd6b05`）。因此该宿主 v0/v1 固定体为 216 字节，v2 长度仍为 220 字节但字段对位不同。此分支的实档宿主尚未核实。写侧 `Paper::Save` 对文档索引 0（对象版本表 `[2, 3]`）直接写整块内存（`0x7bcd71c3`）。该写法接线见下节。
- 72 份安装介质类型库公开的 IPrinter 类为 EmfPrinter、PsPrinter、ArcPressPrinter，均已接入表中的 Load 布局。ArcPress `0x100024f0` 写固定类版本 2，却只在文档索引 >1 时写驱动段。旧写侧接线见下节。

构造测试验证三类打印机、Paper v0–v3 的空及非空 DEVMODE、空/非空打印机、两种字符串编码、驱动数据与后继对象对位，以及短读和长度异常时保留原始打印设置尾段。既有 66 份 10.1 介质工程和参考工程的打印标记均为零，尚未取得非空设置的实档验证，也未在 ArcMap 或 QGIS 中人工检查这些变体。

### 旧保存索引的纸张及 ArcPressPrinter

复用 ArcMap.exe 的保存入口 `0x466fb0` 反编译，对原始指令补查。映像为 x86
`10.1.0.3035`，SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`。

- ArcMap `0x467428` 取文档保存索引，`0x46743d` 查询同一个 ObjectStream 的
  IDocumentVersion，`0x467474` 写入索引。SystemCore 的 `0x7ffd0610 / 0x7ffd0630`
  写读同一内存字段。ArcMap `0x467508..` 按此索引选择 `/Version` 文本资源，
  又在 `0x467664` 以相同索引选择 short 映射表 `0x54bfb4`。
- ArcMap `0x469b3c` 创建 `/PageLayout`，切换同一 ObjectStream 并在
  `0x469c9c` 保存布局。布局对象读写版本表的索引 0、1 分别为 v7、v9。
  `0x469e32..0x469e66` 写 0 或 1 打印标记，非零时 `0x46a055` 直接委派
  Paper 的 Save，`0x46a0fd` 再保存带键的打印机对象。成功后 `0x46a26a`
  已开始创建下一条 `/View` 流。AfCore `0x68a7b640` 将打印机保存委派给
  SystemCore `0x7ffcf7e0`，后者把同一 ObjectStream 传入标准持久化 Save。
- 因此 ArcMap 10.1 按 8.3 保存时，Paper v2 的非空 DEVMODE 以声明的分配长度
  原样写盘，包含可能存在的尾部填充。按 8.3 或 9.0 保存时，ArcPress v2 依
  `0x10002528` 取得的文档索引省略整个驱动段，BSTR 后立即写五个 LONG 和
  一个字节。该形式符合保存端证据。10.1 的 ArcPress Load 仍按类版本读取驱动段。
- Rust 的 `parse` 复用 `/Version` 识别结果。仅在版本文本及 short 成对命中
  8.3 或 9.0、同时布局版本与打印标记匹配上述写端时，启用对应旧写法。
  ArcPress 的旧短体还需恰好到达流尾。其他上下文沿用标准 Load 布局，截断时仍保留
  打印设置尾段。通过合成 OLE 文件验证了命令行接线，并测试了匹配和不匹配的版本、
  原样分配块的字节保留及后续打印机对位。旧版的真实非空打印设置及 ArcMap/QGIS
  人工核验仍缺。

## v0–v6 的普通页面元素

使用现有 PageLayout.dll x86 `10.1.0.3035` 的 Load `0x7b6e4e30` 导出，并用
objdump 核对。版本门 `0x7b6e4f02` 在 v0–v6 进入 AfCore `LoadObject`
调用 `0x7b6e4f25`，v7–v15 则走 `0x7b6e4f9e` 的 ExtraSafeReadObject。
两条路径都请求 `0x7b71fcc4` 的 IElement
`{9B001319-9ED7-11D0-859E-0800091A2A72}`，读取顺序均为声明计数次元素。
分流入口沿用此前 `/Maps` 的 ObjectStream 键表。

两个入口共用已支持的元素类分派。普通对象头直接读取，随后按该类的持久化体确定
结束位置。遇到未知、嵌套未完或截断的对象，回退到该元素头并保留后续字节。ExtraSafe
仍校验整个声明区间被消费。测试用参考工程 `/Maps` 与构造的 v6 `/PageLayout` 验证
MapFrame 的跨流回引、同流回引、空槽、布局尾段和失败时保留原始字节。

10.8 介质的 `calibrate.mxd`（SHA-256
`f883510f90d05a01c4ae152aa008a74b14cab0e8469177b98ef7e3b40fc533b4`）的 `/Version`
原文为 `8.1`。以其原始 `/PageLayout` 配合空 `/Maps` 隔离检查，v6 声明的 13 个
普通对象读完 12 个，止于末项 MapFrame 的起点 `0x22da6`。该对象内的 IMap 槽
`0x22dbc` 与真实 `/Maps` 根唯一对象的 Map 类标识及键 31 一致。CLI 的整档
读取仍先在真实 `/Maps:0x52` 的 Map v19 旧列表停下，因此尚未在真实跨流会话中
读完该布局，也未在 ArcMap 或 QGIS 中人工验证。


旧列表及 [Map v19 尾段](mxd-terra-round9-old.md#双列表接线与-map-v19v22-尾段)
接入后，整档从 `/Maps:0x4db` 的根尾转入 `/PageLayout`。最后一个地图框的 Polygon
在 `0x22e0c` 声明 148 字节，其中二维记录为 128 字节，余下 20 字节仍在记录区间内。
Geometry.dll `0x7192db20` 先按声明长度读完整缓冲，再调用共享二维记录解码，
下一个 ISpatialReference 槽从 `0x22ea4` 开始。Rust 的 Polyline/Polygon
读取器校验最小记录长度、保留多出的字节，并从声明边界读空间参考。
整档验证读完 13/13 个布局元素，最后一个 MapFrame 回引真实 /Maps 的键 31，
布局对象至 `0x23123`，随后读完零打印标记。

## GroupElement 内嵌 MapFrame

10.1.0.3035 的 GraphicElements.dll（SHA-256
`f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9`）
在 GroupElement Load `0x75035ef0` 的 `0x75035f66..0x75035f6c`
用普通 LoadObject 读取 IElement，IID 为 `9B001319-9ED7-11D0-859E-0800091A2A72`。
MapFrame 的类与持久化链沿用 [MapFrame 对象体](mxd-map-frame-body-20261118.md)
的 MapElements.dll 证据。组内读取使用同一 MapFrame 对象体读取器和跨流键表，
地图框前无额外 ExtraSafe 包装。截断、未知对象或非根 Map 回引时回滚整组。

SDK .NET 的 PrintMultiPage（`/Version` 文本 9.4，PageLayout 架构 v14）在
`/PageLayout:0x1e` 有一个 ExtraSafe 元素。组对象头位于 `0x26`，键 51，
共七个子元素，其中 `0x1fc24` 是 MapFrame v4（键 71）。
其 IMap 槽 `0x1fc3a` 指向 `/Maps` 的根 Map 键 29，布局读取到
`0x3fc10` 流尾。替换为同会话内基本图形层的键 30 时，读取器在
组包装起点 `0x1e` 停止。构造用例还验证组内地图框可被后续页面元素回引，
及子体截断时整组不提交。ArcMap 与 QGIS 人工验收未执行。

### Yellowstone 的短 DEVMODE 体

10.1 SDK Java 的 Yellowstone 使用 `/Version` 文本 9.0、short 10，
布局 v7、打印标记 1、Paper v2。对象体从 `/PageLayout:0x406` 起，
DEVMODE 数据自 `0x4be` 起，声明分配长度 1172、dmSize 220、
dmDriverExtra 952。按 220 字节固定体计算会落到 `0x952`。按
212 字节固定体加 952 字节计算，后接 `0x94a` 的原生 EmfPrinter 类标识、
对象键 164 与版本 1，结束于流尾 `0x960`。

用 objdump 复核 10.1.0.3035 的 x86 PrintOut.dll（SHA-256
`ff9bdbf31a6d4c06573fa37e2da529217a93fec44c9715458bbaf1b4ac9a5bde`）。
`0x7bcd6720` 的版本文本比较常量 `0x7bce0ce8` 确为 9.0；
该分支对 v2 在 `0x7bcd6a24` 跳过一个 DWORD，又在 `0x7bcd6b05`
后补读一个 DWORD。样本的短体写出来源仍待定位。

读取器在标准规则失败后，仅在上述版本和标记组合下尝试已观察到的
212 字节布局。只有完整读出流尾的 EmfPrinter v1 才提交游标和对象键。
构造测试覆盖短体、缺少版本上下文、截断和流尾附加字节。通过 `parse`
命令验证 Yellowstone 的布局到达流尾。ArcMap 10.1 和 QGIS 尚未人工核验。

Java SDK 的两份实档已加入按需运行的测试。Yellowstone
（SHA-256 `05ff0f36d97048eef5f241659664a57e24b3ce822212205ad0bbdbf9a6307e73`）
校验 Paper v2 的原始 DEVMODE 区间及 EmfPrinter 键 164。
YellowstoneGDB
（SHA-256 `f758b58fbad9275bd4a9491f2d3ae424d70cb85a7df674247bfe3739fc49ee89`）
校验 Paper v3 及打印设置结束于 `0x1296`。两份工程均从
`/Version`、`/Maps`、`/PageLayout` 入口验证，布局无未解析区间。
这两份工程仍缺显式设置 ObjectStream 版本文本为 9.0 的运行期证据。

### 10.1 介质非空打印设置复核

用 `mxd2qgz parse` 复跑 10.1 Desktop 与两套 SDK 介质中本地已
提取的 110 个 MXD 路径。按路径计数，保留同内容的不同提取件。
所有命令均成功返回，108 个布局完整消费。两份 `mole_data` 在
/Maps 的逐图层遮罩阶段停止，布局保留原始字节。

对打印标记为 1 的 17 条布局，再通过
`parse_mxd_streams_10_1_with_version` 检查纸张和打印机字段。
17 条均读出 EmfPrinter，且打印设置结束偏移等于 /PageLayout
长度。Yellowstone 是唯一 Paper v2（/Version 9.0 / short 10），
其余 16 条 Paper v3 的 DEVMODE 实际字节数等于声明长度，
范围为 364–7640。Paper v3 的头部组合包括 9.0 / 13、
9.2 / 14、9.3 / 16、9.4 / 18 和 10.0 / 18。
这些头部组合反映档案内容，显式设置 ObjectStream 版本文本为
9.0 的运行期证据、PostScript/ArcPress/打印机回引实档仍待补充。
ArcMap/QGIS 人工验收未执行。

### 分流 IPrinter 回引

PageLayout.dll 10.1.0.3035 的 `0x7b7095af` 通过共享
ObjectStream 请求 IPrinter。SystemCore.dll 同版本的 `0x7ffd0063`
按键找到登记实例后查询接口。因此分流读取器用登记类校验 IPrinter
回引。构造测试覆盖引用头与登记类不同但接口匹配时的读取边界，
以及错引 Map 时的游标和对象键回滚。真实 IPrinter 回引宿主待验证。

### 打印设置对象的版本比较与实档检索

对前述 10.1.0.3035 PE32 映像补查 objdump 指令。PrintOut.dll
的 Paper 在 `0x7bcd6c34 / 0x7bcd6c39` 以 `cmpw / jbe` 接受
无符号版本 0–3；EmfPrinter 和 PsPrinter 共用入口在
`0x7bcd1d4d / 0x7bcd1d51` 以低 16 位比较的借位标志拒绝版本大于 1。
ArcPressPlugin.dll（SHA-256
`547cbef6e256f78af2ed3518f534848039a6212600071cc11b3ea662226ca9a3`）
的 `0x100021c1 / 0x100021c7` 同样以无符号比较接受 0–2，
`0x10002382 / 0x1000238e` 跳过 v0/v1 的驱动段。Rust 的原有
无符号读取规则与上述指令一致，构造测试补验高位值拒绝及打印设置回滚。

从本地 `temp/` 下按 SHA-256 去重后的 142 份 OLE MXD/MXT
提取到 140 条 `/PageLayout` 流。扫描 PsPrinter 和 ArcPressPrinter
的 CLSID 内存序，两类均无命中。此检索限于可提取的分流布局。
两类打印机的真实宿主和 ArcMap 人工验证仍待补充。

### 布局元素的非正计数

PageLayout.dll 10.1.0.3035 的 Load `0x7b6e4e30` 在
`0x7b6e4eee / 0x7b6e4ef1` 以有符号比较判断循环下标是否
达到元素计数。下标从零开始，非正计数直接进入对象尾段。
读取器保留有符号声明值并只在正值时读取元素，版本门仍按
`0x7b6e4e70` 的零扩展值判断。合成 v6 通过分流入口验证
负计数后的对象尾段和打印设置标记。真实负计数工程尚未验证。

### 旧版布局元素错引 Map

PageLayout.dll 10.1.0.3035 的 `0x7b6e4f25` 在 v0–v6
调用普通 LoadObject 请求 IElement。SystemCore.dll 同版本
`0x7ffd0063` 向登记实例查询接口后交还 S_OK。Map 的 IElement
输出为空。PageLayout 的 `0x7b6dcfd0` 跳过空元素的接口调用，
`0x7b6c9540` 构造 SuperElement 时调用 `0x7b6c9720`，该函数
允许保存空内部指针。随后 `0x7b6eead0` 插入该包装，循环继续。
因此原先在此槽停止的 Rust 处理已改为保留未解析的回引信息并接续。
测试使用参考工程的 `/Maps` 和构造布局，验证下一元素与布局尾段。
真实错引工程及 ArcMap 人工验证仍待补充。

### ExtraSafe 元素占位恢复

PageLayout.dll 10.1.0.3035 在 `0x7b6e4f9e` 加载 ExtraSafe IElement，
并在返回值非零或 IsValidElement 失败时转到 `0x7b6e4fd0`。
`0x7b71fd64` 的 CLSID 为 `{EC5114AF-4933-4629-91DE-A1C073E400C9}`，
与同版本 GraphicElements.dll 中 PlaceholderElement 的 GetClassID
`0x75053de0` 一致。占位对象接收当前流和包装起点，其虚表入口
`0x75071f0c` 指向 `0x75055050`。此函数回到包装起点，复制 8 字节
前缀及对象字节，并在 `0x75055171–0x7505518b` 将流定位到包装结束。
AfCore 对应读写器的低高字运算佐证前缀是单个 64 位长度。

当受支持的包装内只有一个指向已确认 Map 的回引头时，该键已有实例，
IElement 查询失败不会引入新对象键。读取器在 v7–v15 保存包装原始字节，
通过常规解析入口报告占位并继续读取下一个元素。测试使用参考工程
的 `/Maps` 配合构造布局，覆盖错引后读取下一元素、读完布局尾段及截断回滚。
未知新对象可能已改变原生键表，其恢复及实际占位工程仍待验证。
未做 ArcMap 人工验证。

#### 零长度包装

AfCore.dll 10.1.0.3035 在 `0x68a7be89–0x68a7be95` 对完整的零长度
前缀返回 S_FALSE，PageLayout.dll 的 `0x7b6e4fae` 据此进入占位路径。
同版本 GraphicElements.dll 的 `0x75054e9b–0x75054e9e` 对零长度
直接跳过类标识和版本探读，复制和重定位仍按前缀起点加 8 字节完成。
Rust 记录这 8 字节的占位包装并接续，不向共享键表登记空槽。
v7–v15 的构造布局验证后一元素、布局尾段和截断前缀。
真实零长度包装及 ArcMap 人工验收待补充。

#### 包装内的空对象头

SystemCore.dll 10.1.0.3035 的 `0x7ffcff70` 对全零 CLSID
返回 S_FALSE 并保留空输出。即使调用包装将空输出归一为 S_OK，
PageLayout.dll 的 `0x7b6e4fc0` 仍用 IsValidElement 检查该指针。
该导入指向 LayerLib.dll 10.1.0.3035（SHA-256
`55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）
的 `0x786bd090`，空指针在 `0x786bd173` 跳到
`0x786bd1b3` 返回 false。宿主据此进入 PlaceholderElement 恢复。
GraphicElements.dll 的 `0x75054ecd–0x75054ed4` 在全零 CLSID 时
跳过键与版本探读。构造测试覆盖 v7–v15 的完整 24 字节包装、
截断及之后的布局尾段。v0–v6 的普通空引用由原有测试覆盖。
真实实档与 ArcMap 人工验证待补充。

#### 空对象头之后的占位字节

用已对照摘要的 GraphicElements.dll 10.1.0.3035 补查 objdump。
`0x75054ed4` 在零 CLSID 时跳过键、类版本与类专用探读。
占位入口 `0x75055078–0x7505509f` 回到包装起点并重读长度，
`0x750550c3–0x75055130` 以长度加八为复制范围，
`0x75055171–0x7505518b` 将宿主流定位到包装末尾。因此即使
零 CLSID 后还有包装内字节，也可保留这些字节并接续布局。
构造输入覆盖恰好一个空头、带额外字节的空头及逐字节截断。
带此形态的实档与 ArcMap 人工验证仍待补充。

#### 已登记元素回引的声明边界

SystemCore.dll 10.1.0.3035 的 `0x7ffd0063–0x7ffd007d`
在读完键并命中登记表后查询接口，随即返回，无后续持久化体。
AfCore.dll 10.1.0.3035 的 `0x68a7bef3–0x68a7bf0f` 在
实际位置与 64 位声明长度计算出的终点不一致时按声明位置 Seek。
PageLayout.dll 主元素循环随后做 IsValidElement 检查并继续。
Rust 对登记类已经明确的地图框、装饰框、矩形及组元素回引记录额外
字节，按声明位置继续。合成装饰框测试覆盖回引、额外字节、
下一元素和截断回滚。未知登记类型、其他新对象的区间差异、
真实带尾字节的回引及 ArcMap 人工验证仍待补查。

装饰框等四类已有跨宿主登记类的证据。对已经完整从布局元素槽
读出的其他对象，Rust 也记录键，在同一 ObjectStream 会话中
沿用这一证据。新键随试读事务提交，避免声明范围校验失败时
传播元素判定。SystemCore 的 `0x7ffd0063–0x7ffd007d` 查询的是
键命中的实例，因此图片回归测试使用与首次登记不同的回引头 CLSID，
验证仍以该图片的已确认键处理尾字节。ArcMap 人工验证仍待补充。

### 布局扩展的有界续读

PageLayout.dll x86 10.1.0.3035（SHA-256
`ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`）
在 `0x7b6e5f76` 调用 IAT `0x7b71c468` 的 LayerLib
`ExtensionHelper::LoadExtensions`。同版 LayerLib.dll（SHA-256
`55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）
的 `0x786c240f–0x786c2426` 在没有实例时将两个长度 LONG 传给
IStream::Seek，起点为当前位置。`0x786c23f4–0x786c23fe`
在有实例且长度为零时绕过 Load。`0x786c24b3–0x786c24bb`
按有符号计数迭代。

读取器保留零长度条目及未识别类的序号、CLSID 和有界原始体，再读
布局后续字段。构造测试覆盖未知类、零长度已知类、随后的优化扩展、
v11 槽和超限事务回滚。已识别优化扩展的非空准备结果和非空对象槽、
其他扩展类的字段及 ArcMap/QGIS 人工验收仍待验证。

### 优化扩展的 ArcGIS Server 连接名

CartoConverter.dll x86 10.1.0.3035（SHA-256
`5982306cbe6564505a3c37b240e0ea75d5255c13b10865880938c2e051c1f264`）
的 Load `0x6ccc0de0` 在 v3 起读取两个 LONG 后，将流和
`0x6cd7547c` 的 IID 传给 AfCore LoadObject（`0x6ccc0f91–0x6ccc0fa2`）。
10.1 介质 esriGISClient.olb（SHA-256
`8695beba425c46d252e43bd8469d52e2a88847acdb4cdeadc48ed838302cb067`）
将 `{298CCE6A-4D86-40EE-8F4A-BEBC419B9A4D}` 定名为
IAGSServerConnectionName，公开实现为 AGSServerConnectionName。
布局复用 [AGSClient 的连接名类体读取器](mxd-10.1-raster-layer-tail.md)，
沿用布局 ObjectStream 的引用表。构造测试覆盖新对象、第二条扩展的
回引、后续布局槽及整体截断回滚。非空准备结果、其他注册实现和
真实连接名布局宿主仍待取证。ArcMap/QGIS 人工验收未进行。

### 布局准备结果与 ConverterOption

MappingCoreLib.dll 10.1.0.3035 的反编译导出
`79b719a0.c`、`79b6fa20.c` 确认有符号结果计数、short 版本门、
两个 LONG 和转换选项槽。v1 随后读取 IUnknown 和 IMap，其他版本按
有符号个数读取 IUnknown。三处 IID 常量在 `0x79bfc284`、
`0x79bf51fc`、`0x79bfc274`。IUnknown 槽中的 FeatureLayer
曾在 [10.1 介质的 Basemap 宿主](mxd-round3-basemap-20261117.md) 实档中确认。

同版 CartoConverter.dll 在 QI `0x6ccc0029–0x6ccc0055` 将
`{0A22B09C-7BA4-4AA0-998A-48535DD07252}` 映射到类的 `+0x10`。
虚表 `0x6cd7760c` 的 RTTI `0x6cd9f02c` 定名 ConverterOption。
其持久化视图虚表 `0x6cd77634` 的 GetClassID `0x6ccbfe90` 返回
`{97314224-2DA2-4939-ACC5-DD7C0A7BFC8B}`，Load `0x6ccc01b0`
读取有符号版本（上限 1）、LONG 和 IPropertySet
（`0x6ccc021d–0x6ccc0229`，IID 在 `0x6cd74d8c`）。

布局读取器复用引用表及属性集读取器。构造测试核对 v1 固定槽、
v2 对象计数、转换选项新体和回引、后续布局槽及截断回滚。
v1 的新 Map、其他 IUnknown 实现及真实非空布局宿主仍待取证。
ArcMap/QGIS 人工验收未进行。

#### 准备结果中的 Map

MappingCoreLib::PersistedPrepareResult::Load `0x79b6fa20` 在 v1
的第二个固定对象槽请求 `0x79bfc274` 的
`{E6BDAA75-4D35-11D0-98BE-00805F7CED21}`。10.1 的
esriCarto 类型库将其定名为 IMap。v2 的 IUnknown 列表可装入
Map，10.1 介质 Africa.mxd 的 [Basemap 准备结果](mxd-basemap-continuation-20261118.md) 提供该类的宿主证据。
两个槽均复用普通 Map 读取器及图形嵌套深度限制。完成 Map 尾段后
才提交布局读事务。固定 IMap 槽的回引按登记实例类别校验。
构造输入覆盖两个槽的新 Map、跨结果回引和错误类回滚。
真实布局优化扩展中的 Map 及 ArcMap/QGIS 人工验收仍待验证。

#### 优化扩展的有符号版本门

同摘要 CartoConverter.dll 的 `0x6ccc0e0a–0x6ccc0e0f` 用
有符号 short 判断版本是否超过 6。`0x6ccc0e42–0x6ccc0e47`
用有符号比较决定是否进入 v2 字符串和后续字段。Rust 保留
原始 16 位版本值，负版本仍读取准备结果，随后结束扩展体。
构造测试用 `0xffff` 版本和负计数核对后续布局槽边界。
真实负版本扩展及 ArcMap/QGIS 人工验收仍待验证。

#### 布局类型槽的回引类判定

SystemCore.dll 10.1.0.3035 的 `0x7ffd0063–0x7ffd007d`
在键命中时直接向登记实例查询槽要求的接口。对已核实实现类的布局
类型槽，Rust 用登记类判定，并保留回引头中的原始 CLSID。
构造连接名扩展验证两种情况：回引头的 CLSID 与登记类不同但键
指向连接名时可以续读，回引头写成连接名但键指向 Page 时回滚。
真实布局中这种不一致的回引及 ArcMap/QGIS 人工验收仍待验证。

### PageLayout 尾段的新 Map 与回引

PageLayout.dll 10.1.0.3035 的 `0x7b6e5539` 将
`0x7b71feb4` 的 IMap IID `{E6BDAA75-4D35-11D0-98BE-00805F7CED21}`
交给普通 LoadObject。反编译导出 `PageLayout/7b6e4e30.c` 显示，
加载成功后先把 Map 传给布局，再继续读取 short 字段。
SystemCore 按对象键区分新体和回引。

读取器在新体处复用布局准备结果的 Map 解析入口，沿用同一引用表
及嵌套深度限制。回引用已确认的 Map 键校验，保留头中的原始类标识。
构造测试覆盖新 Map 和跨读取回引。SDK 中已核实
[尾段回引根 Map 的宿主](mxd-us-media-renderer-paragraph-20261118.md#无-mapframe-的-pagelayout-尾段回引)。
尾段首次定义 Map 的真实宿主和 ArcMap/QGIS 人工验收仍待验证。
