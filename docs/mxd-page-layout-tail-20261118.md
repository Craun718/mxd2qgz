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

### 分流 IPrinter 回引

PageLayout.dll 10.1.0.3035 的 `0x7b7095af` 通过共享
ObjectStream 请求 IPrinter。SystemCore.dll 同版本的 `0x7ffd0063`
按键找到登记实例后查询接口。因此分流读取器用登记类校验 IPrinter
回引。构造测试覆盖引用头与登记类不同但接口匹配时的读取边界，
以及错引 Map 时的游标和对象键回滚。真实 IPrinter 回引宿主待验证。
