# Graticule、定位框与填充轮廓

本轮以 [10.1 页面元素接线](mxd-page-layout-surround-20261118.md)的四个实档停止点为起点，核对原生持久化调用和实档边界。涉及 10.1.0.3035 的 MapElements.dll（SHA-256 `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`），用 PE 原始 RTTI、虚表、导入和 `objdump -d -Mintel` 复核；接口 GUID 与 10.1 esriCarto、esriDisplay、esriSystem 类型库互证。地址为静态映像 VA。未进行 ArcMap 或 QGIS 人工验收。

| 结构 | 原生读侧依据 | 读出的内容 |
| --- | --- | --- |
| Graticule | GetClassID `0x79559100` 从 `0x7959b3dc` 取 CLSID；Load `0x79559140` 的 `0x79559171` 拒绝版本 >5 | 地图网格基体后，short、4 个 double、LONG；v2 加 short，v3 加两个 double、线符号、标记符号、double、short，v4 加 double，v5 加 short 和两个 double。部分 double 经同一通用读函数槽读 8 字节，按原始位保留。 |
| MapGrid | Graticule `0x79559191` 委派 `0x79576ec0`，后者版本上限 1 | 名称、线符号、标记符号、IGridLabel、线符号、double、short、线符号、double、4 个 short、IMapGridBorder。 |
| DMSGridLabel | GetClassID `0x795513c0` 取 `0x7959ac60`，Load `0x79551400` 版本上限 4 | LONG、3 个 ITextSymbol、2 个 short、ILatLonFormat、double；v2 加 10 字节，v3、v4 各再加一个 short。经纬度显示属性未从标量位推定。 |
| SimpleMapGridBorder | 持久化虚表 `0x7959dca0` 的 Load `0x7958ee10` 和 Save `0x7958ee90` | short 版本（上限 1）、一个 ILineSymbol，具体实现仍由流内 CLSID 决定。 |
| LocatorRectangle | GetClassID `0x7956ab00`，持久化虚表 `0x7959c0e8` 的 Load `0x7956ab40` | short 版本（上限 3）、IMapFrame、IBorder、IBackground、ISymbol、short；v2 加 IShadow，v3 加 short。作用于已登记的 MapFrame 回引；未把新 IMapFrame 的递归分支视作已闭合。 |

持久化体复用既有符号、颜色、字体、经纬度格式和边框读取器。未知新类、未知版本、计数或长度越界仍使整个有界页面元素回滚，不登记尚未解析的新键。Graticule 的工作区入口为 `src/maps/map_grid.rs`；LocatorRectangle 由 `src/maps/map_frame.rs` 在实际宿主的 ILocatorRectangle 循环内分派。

- **MilitaryPortrait.mxd**：首个曾失败的 Graticule 键 159 位于 `/PageLayout:0x10d4`，Graticule v5 委派 MapGrid v1，名称为 Grid，DMSGridLabel v4 为键 162；后续另一网格的边框键 192 位于 `0x19df`，其 ILineSymbol 为已知 CartographicLineSymbol。定位框键 211 在 `0x221c`，内嵌 IMapFrame 回引键 158。11 个页面元素读至 `0x288d`，保留其后 607 字节。MilitaryPortrait.mxt 同样读完 11 项，尾段从 `0x2937` 保留。
- **ConservationLandscape.mxd**：原停止点为 `/PageLayout:0x161e` 的 RectangleElement：MultiLayerFillSymbol 键 125 → SimpleFillSymbol 键 127 → MultiLayerLineSymbol 键 128（`0x178d`）。SimpleFillSymbol 的原生 ISymbol 请求和已有 MultiLayerLineSymbol 读写证据见 [DisplayCore 符号记录](arcmap-10.1-display-symbols-round2.md)。修正轮廓槽使用早期分派、找不到已支持类的问题，继续读取后续填充色和标量。该工程读完 9 项到 `0x18a5`，.mxt 也读完 9 项到 `0x1835`。同一变更让 SDK Portland 的第 14 个 FeatureLayer（`/Maps:0x6af1d`）读到全部四段 TimeData；旧测试只覆盖此前可读的 13 项。
- **10.1 介质版本候选分组**：66 份中，能进入页面元素循环的 48 份现全部读完集合（此前 44 份）。按 `/Version` 头的文本候选分组，9.2 为 3/3，9.3 为 34/34，10.0 为 11/11；这些工程都检测到至少一个页面地图框回引。候选文本为 10.1 的 18 份均因 /Maps 前置未知内容未进入页面元素循环，目前主要见于 Basemap 和 MapSurround 子体；未由这批样本证明 10.1 流类别的全覆盖。10.8 介质仅作回归对照，不据此确认 10.8 原生实现。

`cargo fmt --check`、`git diff --check` 与 `cargo test --no-fail-fast -- --include-ignored` 通过（887 项）。新增合成覆盖 Graticule v0–v5 门、DMS 标签、简单网格边框，以及逐字节截断后的游标和键表回滚。实档测试覆盖两条此前失败的宿主路径。从布局元素集合结束到 PageLayout 根对象结束的字段，以及其他网格、标签、边框实现仍待研究。

## MeasuredGrid、IndexGrid 与 CalibratedMapGridBorder

复用安装介质中的 x86 `MapElements.dll`（`10.1.0.3035`，SHA-256 `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`），用 objdump 补查持久化虚表、委派和字段读写。以下均为该 PE 的虚拟地址。

| 类 | 身份与持久化入口 | 字段顺序 |
| --- | --- | --- |
| MeasuredGrid `{03762C90-F4D0-11D1-ADE8-080009EC732A}` | 持久化虚表 `0x7959d58c`；GetClassID `0x79581a70`；Load `0x79581ab0`；Save `0x79581bd0` | 有符号 short 版本，最大 2。`0x79581afa` 调 MapGrid 基体读取器，随后四个 double、一个 LONG、ISpatialReference 对象槽（`0x79581b8b`）。仅版本 2 再读一个 short（`0x79581baa`）。 |
| IndexGrid `{03762C91-F4D0-11D1-ADE8-080009EC732A}` | 持久化虚表 `0x7959b8cc`；GetClassID `0x79563200`；Load `0x79565300`；Save `0x795654a0` | 有符号 short 版本，最大 1。`0x7956534d` 调 MapGrid 基体读取器，随后两个 LONG，再读两组有符号 LONG 计数与 AfCore String。字符串分别由 `0x795653cd` 和 `0x7956543e` 读取。非正计数跳过循环。 |
| CalibratedMapGridBorder `{6CA416B0-E160-11D2-9F4E-00C04F6BC78E}` | 持久化虚表 `0x79599eb0`；GetClassID `0x79544b50`；Load `0x79546a70`；Save `0x79546b60` | 有符号 short 版本，最大 1。`0x79546add` 和 `0x79546afb` 各读一个 IFillSymbol 对象槽，使用 IID `{E6BDAA7E-4D35-11D0-98BE-00805F7CED21}`；之后是两个 double 及一个 short。 |

三类对象从 `/PageLayout` 的 MapFrame 网格集合读取。网格基体、空间参考和填充符号复用已有读取器，嵌套对象与地图框共用对象键。构造测试在一个地图框内依次读取两版 MeasuredGrid 和 IndexGrid，覆盖填充对象新键及回引、跨网格边框回引、空间参考回引、空标签和中文标签，并检查版本越界、计数越界、截断时的游标和引用表回滚。

在两套介质提取的 268 个 `/Maps` 与 `/PageLayout` 流中未命中这三个 CLSID。当前以静态布局和构造测试验证，尚未在 ArcMap 10.1 中打开含这三类对象的工程。未定标量保留原值，未知标签或符号类仍可使所属地图框停止读取。

## FormattedGridLabel、MixedFontGridLabel 与 CornerGridLabel

对同一 `MapElements.dll` x86 `10.1.0.3035` 映像补查原始虚表与读写指令。MapGrid 在 `0x79576f77` 传入 `{CE41C501-9DF9-11D2-AADE-000000000000}`（IGridLabel）。扫描 72 份介质类型库并沿接口继承关系查找，公开的四个 coclass 为 DMSGridLabel 和下表三类。

| 类 | IPersistStream 虚表、GetClassID、Load / Save | 版本与后继字段 |
| --- | --- | --- |
| FormattedGridLabel `{CE41C507-9DF9-11D2-AADE-000000000000}` | `0x7959ae04`、`0x79555600`、`0x79555640 / 0x79555720` | 有符号 short，最大 2。共同字段之后，v2 追加一个 short（`0x79555702`）。 |
| MixedFontGridLabel `{CE41C508-9DF9-11D2-AADE-000000000000}` | `0x7959da94`、`0x7958d420`、`0x7958d460 / 0x7958d640` | 有符号 short，最大 3。共同字段之后读取第二个 ITextSymbol（`0x7958d539`），v2 起读 5 个 short，v3 再读一个 short。 |
| CornerGridLabel `{FD52B61A-71CD-4108-A916-F818969404EA}` | `0x7959a314`、`0x7954a7d0`、`0x7954a810 / 0x7954aa50` | 有符号 short，最大 2。共同字段之后读取第二个 ITextSymbol（`0x7954a8e9`）、5 个 short、4 个 short、2 个 LONG。v2 追加 3 个 AfCore String，读取入口 `0x7954aa1c / 0x7954aa29 / 0x7954aa36`。 |

共同字段指版本后的 ITextSymbol、INumberFormat 和 double。原生请求 IID 位于 `0x79599948` 和 `0x7959a334`，Rust 复用相应的符号与数字格式分派，并沿用宿主对象键。MixedFont 和 Corner 在第二个 ITextSymbol 加载失败时有运行时恢复路径。读取器只在确认完整消费槽位时继续，避免未知符号体错位到后继字段。标量暂按原值保留。

地图框构造测试覆盖三个标签的各写入版本及其旧门、非空数字格式、标签紧随字段和截断回滚。268 个介质 `/Maps` 和 `/PageLayout` 流未命中这三个 CLSID。尚缺实际标签工程和 ArcMap/QGIS 人工验收。

## CustomOverlayGrid 与 MgrsGrid

本轮核对 ArcMap x86 `10.1.0.3035` 安装介质的 72 份类型库。以
`IMapGrid` IID `{03762C8B-F4D0-11D1-ADE8-080009EC732A}` 沿接口继承关系
枚举，公开类为 Graticule、MeasuredGrid、IndexGrid、CustomOverlayGrid、MgrsGrid。
`IMapGridBorder` IID `{AC81ECF9-9EE4-11D2-AADF-000000000000}` 的公开类为
SimpleMapGridBorder 和 CalibratedMapGridBorder。

现有源码导出未覆盖对应读写体。使用 objdump 核对 `MapElements.dll`
（SHA-256 `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`），
以 Ghidra 补充 Load/Save 反编译。两类的 QueryInterface 分别从
`0x7954b430`、`0x795828a0` 委派至 `0x79575bb0`，后者处理
IPersistStream 并返回虚基类视图。以构造函数安装的虚表和 GetClassID
返回值绑定持久化方法。

| 类 | CLSID | IPersistStream 虚表 | GetClassID / Load / Save |
| --- | --- | --- | --- |
| CustomOverlayGrid | `{FF501C8A-D74B-11D2-9F43-00C04F6BC78E}` | `0x7959a700` | `0x7954ea30` / `0x7954ea70` / `0x7954ebb0` |
| MgrsGrid | `{35F4D526-895D-4F85-B97F-1A3B84C629AD}` | `0x7959d864` | `0x7958a4e0` / `0x7958a520` / `0x7958a710` |

- **CustomOverlayGrid** 先读有符号 short 版本（最大 1，Save 写 1），
  调 `0x79576ec0` 读取 MapGrid 基体，再读 `IName`（IID
  `{4BA33331-D55F-11D1-8882-0000F877762D}`）和一个 AfCore String。
  原生在加载 IName 后调用其 Open，再读取字符串。Rust 保留数据源引用、
  已解码的名称体及字符串，遇见未知 IName 或内嵌名称体读取停止时保留
  MapFrame 边界。名称类型与接口的对应关系见下节。
- **MgrsGrid** 先读有符号 short 版本（最大 3，Save 写 3）并读取 MapGrid
  基体，之后依次读取 LONG、ISpatialReference、short、ITextSymbol、
  ILineSymbol、double、short、ITextSymbol、两个 LONG。版本 ≥2 加一个
  short，版本 ≥3 再加 LONG 和 ILineSymbol。各槽以类型库 IID 验证，
  字段中尚未绑定至属性的值按原始位模式保留。

合成 MapFrame 覆盖 MgrsGrid v1–v3、自定义叠加网格的 GdbCore
FeatureClassName、符号回引，以及不支持的版本、未知 IName 与截断时的游标和
引用表恢复。已提取的 268 个介质 `/Maps` 和 `/PageLayout` 流未命中这两个
CLSID 的内存序字节。参考工程原文件同样无命中。尚未在 ArcMap 或 QGIS 中
人工核验这两种网格。

### 自定义叠加格网的要素来源

在 x86 ArcMap 10.1.0.3035 的 `MapElements.dll`（SHA-256
`67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`）
中，用 `objdump` 复核 `CustomOverlayGrid::Load/Save` 的接口调用。
Load 在 `0x7954ead4` 附近读取 IName，在 `0x7954eb25` 调用其 Open。
打开失败时清空临时指针并继续。打开结果在 `0x7954eb61` 传入
`0x7954f530`，其中 `0x7954f569` 附近用
`{D4803EE6-79F4-11D0-97FC-0080C7F79481}` 查询 IFeatureClass。
`0x7954eb6a` 继续读字符串。Save 在 `0x7954ec2c` 调用
`0x7954f5e0`，后者用 `{2D04C042-7766-11D0-B77D-00805F7CED21}`
查询 IDataset，再在 `0x7954ec53` 附近调用 IDataset::get_FullName，
并在 `0x7954ec63` 用 SaveObject 写回该名称。

Rust 在此槽复用已有的 GdbCore、FileGDB 和 Coverage 的 FeatureClassName，
以及 FeatureQueryName、NetCDFFeatureClassName、XYEventSourceName、
RouteEventSourceName 和 AMSDatasetName 的名称体读取器。各名称的版本门和
内嵌对象顺序见 [Coverage、NetCDF 和事件源](mxd-terra-round9-old.md)、
[书签名称槽](mxd-terra-round9-bookmark.md)和
[查询名称接入](mxd-terra-integrate-query.md)。AMSDatasetName 允许多个数据集类型，
其中类型 5 的 Open 分支返回 IFeatureClass。该槽解码持久化数据，保留原始
类型值。合成 MapFrame 验证了含 TableName 和 XYEventProperties 的事件源、
后续格网编号、嵌套引用注册，以及未知内嵌表类时的完整回滚。重新读取 134 份
10.1/10.8 介质工程及仓库参考工程，135 份工程的 `/Maps` 与 `/PageLayout`
均完整消费，其中无 CustomOverlayGrid CLSID 字节命中。扩展名称的原生
工程保存、打开和显示仍待人工验证。

### 叠加格网的关系查询名称

在 MapElements.dll 10.1.0.3035 的 CustomOverlayGrid Load
`0x7954ea70` 所请求的 IName 槽，对现有共享分派补充构造验证。
构造 RelQueryTableName v3 及嵌套 RelationshipClassName，按同版
GdbCore.dll 的 Load `0x70a5a530 / 0x70a3b330` 编码。
测试经 MapFrame 入口确认两个对象键及后继格网字符串的位置。
此测试验证持久化分派，SDK 和介质中仍待取得该格网的真实宿主，
实际 Open 返回的接口及原生显示尚未验证。

### 定位框的地图框回引

MapElements.dll 10.1.0.3035 的 LocatorRectangle Load
`0x7956ab40` 请求 IMapFrame。SystemCore `0x7ffd0063` 按键
查询登记实例。MapFrame 读取器登记键及类，定位框据此恢复登记类。MilitaryPortrait 的
`/PageLayout:0x2232` 引用键 158。修改构造副本的头部类标识仍能读到
相同布局边界，改为 Graticule 键 159 则保留整个失败布局元素。
新地图框递归分支见下节。ArcMap 人工验证仍待补充。

### 定位框首次定义地图框

MapElements.dll 10.1.0.3035 的 LocatorRectangle Load 在
`0x7956ab9c / 0x7956aba8` 把 IMapFrame IID `0x7959c180`
交给普通 LoadObject，成功后从 `0x7956abb9` 读取边框槽。
首次出现的 MapFrame 复用布局地图框读取器、同一对象表和流偏移，
嵌套新对象受现有图形深度预算约束。构造测试覆盖内联地图框、
根 Map 回引、定位框后继字段、截断及超深回滚。
MilitaryPortrait 仍提供回引分支实档。首次定义分支暂无实档及 ArcMap 人工验证。

LocatorRectangle 的版本门也按有符号 short 比较。MapElements.dll
10.1.0.3035 的 `0x7956ab75 / 0x7956ab7a` 拒绝大于 3 的值，
`0x7956ac36 / 0x7956ac3a` 及 `0x7956ac65 / 0x7956ac6c`
分别控制阴影槽与尾 short。构造测试覆盖负版本的内联地图框、
后继字段边界及大于 3 时的回滚。真实负版本定位框仍待取证。

### 网格标签与边框回引

MapElements.dll 10.1.0.3035 的 MapGrid Load `0x79576ec0` 由
`0x79576f83` 和 `0x795770a9` 读取 IGridLabel 与 IMapGridBorder。
两槽按 SystemCore `0x7ffd0063` 的对象键查询使用登记类。
构造地图框验证标签与边框回引保留已登记的类，误引填充符号时
停止并回滚布局元素。ArcMap 人工验证尚未进行。

### MapGrid 和 SimpleMapGridBorder 的版本上限

MapElements.dll 10.1.0.3035 的 `0x79576efa / 0x79576eff`
及 `0x7958ee45 / 0x7958ee4a` 均用有符号比较拒绝版本大于 1。
构造 Graticule 覆盖两个负版本值的原始位模式和后续字段边界，
并沿用逐前缀截断回滚检查。真实负版本工程尚未验证。

### Graticule 与 DMSGridLabel 的旧版本字段

MapElements.dll 10.1.0.3035 的 Graticule Load
`0x79559171 / 0x79559176` 与 DMSGridLabel Load
`0x7955143a / 0x7955143f` 都使用有符号版本上限。两套后缀
比较分别位于 `0x7955924a..0x79559349` 与
`0x79551534..0x79551575`，负值跳过版本化后缀。
构造测试校验原始版本、后继字段边界和截断回滚。真实负版本尚未验证。

### 经纬网标签的格式回引

MapElements.dll 10.1.0.3035 的 DMSGridLabel Load `0x79551400`
在 `0x79551501 / 0x7955150f` 请求 ILatLonFormat。
按 SystemCore `0x7ffd0063` 的对象键查询规则，读取器保留登记的
LatLonFormat 类。构造地图框验证格式回引头类标识不一致时继续读取，
以及误引地图框自身时整体回滚。ArcMap 人工验证尚未进行。

### 数值网格标签的格式回引

MapElements.dll 10.1.0.3035 的三类数值标签加载路径请求
`0x7959a334` 的 INumberFormat。按 SystemCore 同版 `0x7ffd0063`
的规则，标签和 FieldInfo 共享已支持格式的登记类检查。
构造测试覆盖三类标签的有效 CurrencyFormat 回引、误引标签自身及
失败时的游标与对象键回滚。真实回引宿主和 ArcMap 人工验证尚待补充。
