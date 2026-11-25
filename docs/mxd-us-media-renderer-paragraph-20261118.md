# 10.1 介质工程：唯一值渲染、坐标变换与段落文本

本轮按工程根和已有键表重放 [Basemap 续查](mxd-basemap-continuation-20261118.md)留下的九个停点。依据为版本资源声明 10.1.0.3035 的 PE32 x86 文件、现有 Ghidra 伪代码及 `objdump -d -Mintel` 的原始指令、导入和虚表；地址均为静态映像 VA。复核 SHA-256：

| 模块 | SHA-256 |
| --- | --- |
| FeatureLayer.dll | `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` |
| DisplayCore.dll | `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2` |
| Geometry.dll | `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47` |
| CartoXProvider.dll（本轮从 Desktop/Setup1.cab 补提取） | `1c4a6d3681e7527ca21e1b9afeb3b5a83922ad12f6c8a0e5c19dec33dd6f3cf5` |

## 对象体证据

- **UniqueValueRenderer**：FeatureLayer.dll 开放导出中的 GetClassID `0x6f72d490` 写 `{C3346D29-B2BC-11D1-8817-080009EC732A}`；RTTI 对应持久化虚表 `0x6f7b1ce8`，`+0x14/+0x18` thunk `0x6f736ed0/0x6f736fc0` 转到 Load/Save `0x6f72fce0/0x6f730140`。顺序为版本、字段计数、逐项 String 加 3 字节、分隔符、ISymbol、ILegendGroup 列表、ILegendItem、值记录计数及 String/12 字节/可选 String、short/旧旋转字段/String 三项/short。v1 不消耗默认标签字符串；v3 起委派新旋转和尺寸字段；v4 起读 IColorRamp；v5 再读两个 short。旧旋转委派的第三参数固定为假，LayerLib `0x786f93d0` 因此不额外读版本 short。保留尚未定名的标量位模式；此前已支持的符号、图例组、色带按共享键表读取，未支持的 ILegendItem 新对象和未知子类保持停点。
- **色带的 BSTR 名称**：DisplayCore 四个 IColorRamp 类的 Load `0x6eb3c190/0x6eb395b0/0x6eb3ae20/0x6eb316c0` 与写侧均调用 AfCore 的自由函数 ReadString/WriteString，而非成员 String::Read/Write。AfCore 10.1 导出 `0x68a7b7a0/0x68a7b800` 的实际指令为 u32 UTF-16 码元数；零长度无正文，非零消费 `2*n+2`（含末尾零）。USA 的 RandomColorRamp 在 `/Maps:0x2731`，键 132，名称长度 17。四个现有读取器与合成夹具都已改用原生 BSTR 布局。这是对[历史名称槽记录](arcmap-10.1-display-slots-round3.md)的更正。
- **CoordinateFrameTransformation**：Geometry.dll 的 RTTI 持久化虚表 `0x71a2786c`，GetClassID `0x719c4180` 写 `{0F024431-C1F8-11D2-BD07-0000F875BCCE}`，Load/Save `0x719c5d40/0x719c5ef0` 与 GeoTransformation、NADCONTransformation 完全共享。继续使用同一个版本、原始文本长度、原始文本读取器，不计算坐标变换。USA 根 Map 的变换循环在 `0xfd44` 读到该新对象。
- **TransparencyDisplayFilter**：DisplayCore GetClassID `0x6ebe7ca0` 写 `{AD754A65-13B4-11D3-B89D-00600802E603}`；持久化虚表 `0x6ec16998` 的 Load thunk `0x6ebe8250` 到 `0x6ebe7fc0`，读取 v0..1 与一项 short，随后委派 `0x6eb9f4c0` 读取基类版本 v0..1 与两个 4 字节字段。Save `0x6ebe8030` 和 `0x6eb9f530` 对称。Portland 从根顺序读取到的 18 个 v34 FeatureLayer 均完成四段 TimeData 字段；之前因渲染器或过滤器停止而未读到的四个图层均纳入复核。
- **ParagraphTextElement**：10.1 注册表指向 CartoXProvider.dll。从原 PE 补查 RTTI、持久化虚表 `0x6cee3bb0`、GetClassID `0x6ce52010`、Load `0x6ce52050`、Save `0x6ce521a0`。v0..1 先读 IBorder/IBackground/IShadow、double/int32/double，再直接调用 TextElement Load `0x6ce70e50`，没有额外对象头。Save `0x6ce5227e` 同样委派给 TextElement Save `0x6ce711c0`。已接入通用元素、元素集合和布局报告。CentralAmericaCaribbean 的 ExtraSafe 元素在 `/PageLayout:0x59a`、键 550；段落原样保留动态文本表达式，后续地图框键 556 在 `0x94e` 回引根 Map 键 38，全部 7 个元素到 `0x1dc9` 结束。

## 实档边界

此前尚未进入布局的 CentralUSA、ConterminousUSA、NortheasternUSA、NorthwesternUSA、SouthernUSA、SouthwesternUSA、USA、USACounties 均完整消费 Basemap、图例和 `/Maps` 根，继续读完整个布局元素列表。USA 有三个根 Map，`/Maps` 主体分别止于 `0x1037b/0x14ab1/0x1d673`，之后布局中的三个地图框分别回引根 Map 键 693、879、38。

10.1 Desktop 安装介质的 66 份 MXD/MXT（版本文本候选：9.2 三份、9.3 三十四份、10.0 十一份、10.1 十八份）全部进入布局并读完声明的元素列表。布局元素之后的尾段仍按原始字节保留；此结果不覆盖任意 MXD、全部图层类或全部对象属性，不等同于完整工程转换。未在 ArcMap 10.1 或 QGIS 中人工打开目视验收。

## MapFrame 跨流回引的版本清点

从参考工程和两套 Desktop 安装介质共 135 份 MXD/MXT 读取
`/Version`、`/Maps` 和 `/PageLayout`。按当前解析器顺序读取两条
对象流，逐份检查 Maps 根尾值及布局完成位置均到达对应流末尾。
遍历顶层地图框、MapSurroundFrame 内联地图框及 GroupElement 的递归
子项。只统计从已解析地图框的 IMap 槽取得、类型为 Map 且键等于
`/Maps` 首个 Map 对象键的回引。每份工程各有一个此类回引。

| 介质 | /Version 候选文本 | 工程数 | Map / PageLayout / MapFrame 对象版本 |
| --- | --- | ---: | --- |
| 10.1 Desktop | 9.2 | 3 | 30 / 9 / 2 |
| 10.1 Desktop | 9.3 | 34 | 31 / 10 / 2 |
| 10.1 Desktop | 10.0 | 11 | 45 / 14 / 4 |
| 10.1 Desktop | 10.1 | 18 | 51 / 15 / 4 |
| 10.8 Desktop | 8.1 | 1 | 19 / 6 / 2 |
| 10.8 Desktop | 9.2 | 12 | 29 / 9 / 2、30 / 9 / 2 |
| 10.8 Desktop | 9.3 | 18 | 31 / 10 / 2 |
| 10.8 Desktop | 10.0 | 11 | 45 / 14 / 4 |
| 10.8 Desktop | 10.1 | 26 | 51 / 15 / 4 |
| 仓库参考工程 | 10.1 | 1 | 51 / 15 / 4 |

表中候选文本来自 `/Version`，对象版本来自各自流内字段。10.8 是
安装介质来源，表内结果只证明这些随附工程的实际对象组合。此次新增
Map v29 与 PageLayout v9、MapFrame v2 共用会话的字节证据，
以及以上各组逐份复核的计数。其他对象版本组合和回引宿主仍需取证。
未在 ArcMap 10.1 或 QGIS 中打开工程作目视验证。

### 9.0 候选的 SDK MapFrame 回引

复跑 10.1 Java SDK 的两份工程，`parse` 在 `/PageLayout:0x3c` 读到
MapFrame v2 的 IMap 槽，并用同一引用表命中 `/Maps` 根 Map 键 31。
两个工程的 Maps 根尾值、布局对象及打印设置均读到对应流末尾。

| 工程 | /Version 文本及 short | Map / PageLayout / MapFrame 版本 | MapFrame 键 |
| --- | --- | --- | ---: |
| Yellowstone | 9.0 / 10 | 24 / 7 / 2 | 153 |
| portland_shp | 9.0 / 13 | 29 / 9 / 2 | 133 |

原生 10.1 文档加载器共享 ObjectStream 的证据见
[布局分流读取](mxd-page-layout-tail-20261118.md#旧版-paper-的分流读取)。
以上实档结果覆盖两种 9.0 头部组合，其他版本与宿主仍需核查。
ArcMap 10.1 和 QGIS 人工验证尚未进行。

### 非标准 /Version 组合的 SDK MapFrame 回引

ArcGIS 10.1 SDK 的两个工程包含未映射到 10.1 保存索引的头部组合。
按 10.1 的分流读取入口解析时，两条对象流均读至末尾。顶层
MapFrame 在 `/PageLayout:0x3c` 回引 `/Maps` 根 Map。

| 工程 | /Version 文本及 short | Map / PageLayout / MapFrame 版本 | MapFrame 键 → Map 键 |
| --- | --- | --- | --- |
| GISForXMLSample（.NET SDK） | 9.4 / 18 | 41 / 14 / 4 | 236 → 29 |
| World（Java SDK） | 9.3 / 15 | 31 / 9 / 2 | 332 → 31 |

两份 MXD 的 SHA-256 分别为
`63afbe61413b80a26944b7b063c151d3b49399867c0741f0bd31c0f57611d4f5`
及 `04b6086b15e694bea1e33a978b57265313cc4c44d3603488491b2062a7e90109`。
两种头部均由实档测试确认，实际写出应用版本及其余版本边界仍待取证。
ArcMap 10.1 和 QGIS 人工验证尚未进行。

### 无 MapFrame 的 PageLayout 尾段回引

以 `parse_mxd_streams_10_1` 重读本地 10.1 .NET SDK 的
Montgomery 与 PrintMultiPage，按顺序从 `/Maps` 的第一个对象头
读出根键，再核对布局尾段的 IMap 槽与两个流的完成位置。
两个布局均没有解析出的 MapFrame，尾段的 IMap 槽仍回引根键 29。
两流均完整消费，打印设置标记为零。

| 工程 | /Version 文本及 short | Map / PageLayout 对象版本 | PageLayout 流末尾 |
| --- | --- | --- | --- |
| Montgomery | 10.0 / 18 | 44 / 14 | `0x65a` |
| PrintMultiPage | 9.4 / 18 | 41 / 14 | `0x3fc10` |

两份 MXD 的 SHA-256 依次为
`c233c0bced7c454aa22be6a616c33cdf59256a5a129402557ebf7c2ff998d3bd`
与 `931634d87ef82835fb7360cb2dfd20368947f8cbe196d604fc139fd82cfd022d`。
这两份工程把跨流证据扩展到布局自身的 IMap 槽。其余版本边界、
实际写出程序版本及 ArcMap/QGIS 人工核对仍待补充。
