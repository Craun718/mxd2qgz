# 第九轮：独立表 FieldInfo 与 SDK 样本（2026-11-17）

## 身份与证据

本轮重新计算 SDK Java `LinearRef.mxd` SHA-256 为 `5eb1835d15610b6b56c853153360476400f6d55cf0693dfb72305ded3ce2715d`；文件 `/Version` 9.2/14、来源及提取见 [第八轮旧档记录](mxd-terra-round8-old.md)。保存版本不能等同 DLL 版本。

重新核对 Map.dll：PE32 x86、ImageBase `0x793a0000`、固定文件版本 10.1.0.3035、SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；Layer.dll：PE32 x86、ImageBase `0x78650000`、固定文件版本 10.1.0.3035、SHA-256 `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`。哈希和版本资源用于核对身份，不证明签名或运行行为。

此前独立 Ghidra 桥接工具 CLI / Ghidra 的 Map `0x7945d630` Load 伪代码在 String + 对象的计数循环中请求 `DAT_7948dcd0`，证据及限制见 [宿主记录](mxd-terra-host-table.md)。本轮对上述 Map.dll 直接按 VA 检查 16 字节，结果 `2c ae ba a2 9b 96 d2 11 ae 77 08 00 09 ec 73 2a`，即 IID `{A2BAAE2C-969B-11D2-AE77-080009EC732A}`，与既有 IFieldInfo 识别一致。Layer.dll FieldInfo CLSID `{A2BAAE2D-969B-11D2-AE77-080009EC732A}`、GetClassID、Load/Save、虚表、v0–4 读侧顺序及版本门的原生证据见 [FieldInfo 研究](arcmap-10.1-featurelayer-tail.md#fieldinfo-元素体layerdll-0x7865b520证据确定的)。本轮复用这些已保存 Ghidra 桥接工具 静态证据，并新增二进制 IID 字节交叉核查；未切换共享 Ghidra 桥接工具 目标，也未新增目标软件运行观察。IID 与 CLSID 不同；本轮没有通过运行时 QI 验证对象。

## 接线与样本

独立表命名循环仅对新对象的 FieldInfo CLSID 分派现有 FieldInfo 读取器，保存对象体；空槽和回引保留原有分类，并以对齐的可选体区分。非 FieldInfo 新对象仍在头前停止且不登记；FieldInfo 内部遇未知数字格式时保留外层键、已读体和内部停止原因，游标停在未知对象头前。短读、版本错误及限额导致整次调用回滚。这里的安全行为不代表模拟原生错误处理或验证回引的 QI。

按已有 `/Maps` 宿主读取链，首表头 `0x6a50`、键 331、版本 9。其首个命名对象头 `0x6bed`、键 334；对该已到达边界核对原始字节，类为 FieldInfo，体首 `02 00` 为版本 2。首表命名循环共读取 24 个 FieldInfo 新对象体，均为版本 2、均无内部停止，整个首表在现有读取器的支持范围内报告 Complete。对象键依次为 334、335、337、339、341、342、343、345、347、348、350、351、353、355、357、359、360、361、362、363、364、365、366、368。此前第八轮已有另一个宿主的 5 个 v2 实例，故这里是补充证据，不是首次发现 v2。

读取器继续到第二表头 `0x74ef`，其体版本 9，也报告 Complete；继续到 Map 已有停止点 `0x8407`（`AfterBorderTail { stage: 38 }`）。这些结论依赖现有读取器和共享引用表，上游部分未知体按既有有界策略跳过；并非独立原生仪器对每一个边界的观测，也不证明所有表、其业务数据或 Map 全体均已闭合。样本中数据源等字符串未在此记录，避免泄露路径。

## 验证与可勾范围

`tests/terra_round9_table.rs` 构造测试覆盖 FieldInfo v2 完整分派、逐字节截断时游标/引用表回滚、内部未知数字格式停止与键保留；独立忽略测试从 SDK 文件的 Map 宿主进入，断言两表及首个 FieldInfo 的类、键、版本原始字节。旧的 SDK 测试同步更新到新的停止点。执行 `cargo fmt --check`、`cargo test --offline` 均通过；第九轮 SDK 忽略测试及两个受影响的第八轮 SDK 忽略测试显式运行均通过。构造测试和读取器自验都不能取代目标软件验证。

- 可单独落账：Map 独立表命名槽的 10.1 请求 IID 字节与既有 IFieldInfo 证据相符；FieldInfo 在该槽的保守分派及上述 SDK 两表读取器结果；首表 24 个 v2 是已知 v2 的额外宿主证据。
- TODO 第 135 行是多缺口合项：SDK 首表停止于命名对象的旧描述可更新为本轮结果，但其他类和字段仍开放，整项不能勾选；第 278 行 v0/v1 仍缺实档，不能勾选。本轮依用户要求未修改 TODO。
- 非空独立表的任意对象实现、真实数据源打开、回引接口兼容、其他版本及完整转换未闭合。ArcMap 10.1 暂不可用，未在 ArcMap 或 QGIS 打开、目视检查图层、表、数据、坐标系、范围、样式或布局。

## 关系名称、序列颜色和后续对象

在同一份 Map.dll 10.1.0.3035 的 Ghidra 导出 `0x7945d630` 与
objdump 指令中复核后续 LoadObject；IID 字节以 PE 映像为准，接口名由
10.1 介质类型库对位。

| 版本条件 | 引用 IID 数据的指令 | 请求接口与读取器 |
| --- | --- | --- |
| 起始名称及 v4 起第二名称 | 辅助函数 `0x7945d350`，第二槽 `0x7945dbbe` | IName，共用旧 Map 表名分派，加入 FileGDB 要素类和查询表名称 |
| v2 起关系循环 | `0x7945d829` | IRelationshipClassName，接入类型库公开的 RelationshipClassName 和 MemoryRelationshipClassName |
| v3 起压缩序列之前 | `0x7945da0b` | IColor，类型库公开的 RgbColor、CmykColor、HsvColor、GrayColor、HlsColor 均已有体读取器 |
| v11 起尾槽 | `0x7945dd8b` | IDisplayExpressionProperties，接入 DisplayExpressionProperties 和已支持的引擎 |
| 仅 v11、v12，额外 short 后 | `0x7945dde1`、`0x7945de09` | IUID、IAnnotateLayerPropertiesCollection，复用各自体读取器 |

ALPC 请求的完整 IID 为 `1D5849F2-0D33-11D2-A26F-080009B6F22B`，
与 [Annotation.dll 接口实现](mxd-terra-nextpass-expression.md) 一致。
v14 在时间字段之后调用 `ExtensionHelper::LoadExtensions`，Save
`0x7945e3c0` 调用 `ExtensionHelper::SaveExtensions`。独立表复用
LayerLib 的旧式尺寸、CLSID 和类体分派，已知扩展的嵌套键共享宿主表。
未知关系、尾槽和扩展停在对应条目起点，保留此前读完的对象及引用键。

构造测试覆盖非空关系名、同键回引、序列前非空颜色、v11/v12 UID 与标注
集合、v14 时间轨道扩展的 LongArray 新键和后继未知扩展，以及截断时的
回滚。参考工程和两套安装介质的 134 份工程仍读到两条根流的终点，这批
工程的 Map 解析未报告独立表体。LinearRef 的两张 v9 独立表仍由 SDK
回归用例覆盖，新增非空组合尚缺 ArcMap 10.1 实档和人工验收。


## ObjectClassName 与栅格表名称

2026-11-20 用 10.1.0.3035 的 GdbCore.dll、GdbCoreLib.dll、RasterDB.dll、
RasterCatalog.dll 及 10.1 安装登记表和类型库核对旧 Map 与独立表
的普通 IName 分派。旧 Map 逐项读取 IName，独立表从
`0x7945d350` 与 `0x7945dbbe` 请求相同接口。名称可打开的实际
数据源类型仍由原生运行状态决定。

- ObjectClassName `905CEC64-0B2C-11D4-A060-00C04F6BDD84` 的
  [工厂与构造链](mxd-round3-dataset-format-20261117.md#objectclassname-实现定位)
  建立到 FdbTableName。构造器 `0x709b9330` 安装 IPersistStream
  表 `0x70ad076c`。Load `0x70a63662` 经导入槽 `0x70a8759c`
  到 GdbCoreLib `0x70d4e980`，读 short、三条 AfCore String 和
  IWorkspaceName，布局与 TableName 相同。Save 经 `0x70a63668`
  进入 `0x70d4ea10`。GetClassID `0x709b9d40` 从
  `0x70ac318c` 返回 TableName CLSID。解析器也接受登记的
  ObjectClassName 作为读取别名，并保留原对象头。
- RasterBandName `BC25E113-168B-11D2-8D25-0000F8780535` 的 IName 虚表
  `0x7c131038` 中 Open 为 `0x7c053720`。类型库将 RasterBand
  列为 ITable 的公开实现，已知数据集波段 FullName 的类及布局见
  [波段证据](mxd-10.1-raster-layer-tail.md#波段名称与集合)。
- RasterCatalogName `33FF62FC-D7D6-48E8-82D9-77C0AADBF5D1` 的主 QI
  `0x7be36150` 将普通 IName 查询交给对象 `+0x1c` 的委派指针。类型库将 RasterCatalog
  列为 ITable 的公开实现。该名称的 Load `0x7be36460` 直接委派
  FeatureClassName，保留外层键并共享工作空间等嵌套键。
  读写对位见[目录名称证据](mxd-10.1-raster-layer-tail.md#rastercatalogname)。

两个栅格名称复用栅格读取器，保留原始类和对应的数据体。合成 Map v0、
v3、v6 以及 v0 StandaloneTable 验证三类的新对象、旧 Map 同键回引、
后继字段和独立表截断回滚。SdeRasterTableName 的 Load/Save 均返回 E_FAIL，
详见[栅格表名复核](mxd-10.1-raster-layer-tail.md)。
参考工程与两套介质的 135 份工程均读至 /Maps 和 /PageLayout 末尾，
270 条流未出现本轮三个 CLSID。ArcMap/QGIS 人工验收尚未进行。

## Schematic 图表类和元素类名称

取证使用 Desktop 10.1.0.3035 的 PE32 `SchematicDB.dll` 指令、RTTI 虚表、
10.1 类注册表及 esriSchematic 类型库。模块 SHA-256 为
`f8d9cc41cbb29b278f95a691b5b96b90184c8d1e636dd29c3ebc8eff5c2b7fca`。
地址为模块首选基址下的虚拟地址。

| 持久类 | 注册 CLSID | IPersistStream 虚表 | GetClassID | Load | Save |
| --- | --- | --- | --- | --- | --- |
| SchematicDiagramClassName | `359C39BC-FDB6-49B6-8A1C-2397B3EA9261` | `0x7d70c104` | `0x7d6d5cc0` | `0x7d6e30b0` | `0x7d6d7a70` |
| SchematicElementClassName | `6787EB86-1EB7-4B18-BB61-13A5D847DCDB` | `0x7d70d174` | `0x7d6e2ff0` | `0x7d6e30b0` | `0x7d6e3030` |
| SchematicDatasetName | `333DAE77-1AEB-447D-8E83-FE043E84A671` | `0x7d7091dc` | `0x7d6944e0` | `0x7d695340` | `0x7d6952c0` |

- 三类均读取有符号 short，接受 `<= 1`，随后读取三条 AfCore String。三个
  Save 的版本表位于 `0x7d704a20`、`0x7d704a18`、`0x7d704a24`，首项均为 1。
- 两个类名由 `0x7d6e3126` 调用 LoadObject，请求 `ISchematicDatasetName`
  （IID 常量 `0x7d709370`）。数据集名由 `0x7d6953b6` 调用 LoadObject，请求
  `IWorkspaceName`（IID 常量 `0x7d709350`）。两个普通对象槽均共享上游对象键表。
- 图表类名和元素类名的 IName 虚表分别为 `0x7d70c080`、`0x7d70d104`，
  Open 分别为 `0x7d6d61f0`、`0x7d6e2660`。两者调用内层数据集名的 IName::Open，
  然后进入各自的数据集查找接口。esriSchematic 类型库中的公开 SchematicDiagramClass
  和 SchematicElementClass 均声明 ITable。实际打开结果仍待运行验证。
- 已接入 Map v0–v10 旧表及 StandaloneTable 的 IName 分派。合成 Map v9 和
  StandaloneTable v4 覆盖普通对象、共享数据集与类名回引、未知类停止和截断回滚。
  内层数据集槽按 `ISchematicDatasetName` 分派，工作空间槽复用 `IWorkspaceName`
  读取器。ArcMap/QGIS 人工验收尚未进行。

参考工程和两套安装介质中的 135 份工程均读至 /Maps 和 /PageLayout 末尾。
这 270 条流中未命中上述三个 CLSID。

## 有属性的关系类名称

核对 Desktop 10.1.0.3035 的 PE32 GdbCore.dll、FileGDB.dll、GdbCoreLib.dll
指令及导入导出表。三个模块的 SHA-256 分别为
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`、
`73cef6bac97b0bdba8e4b26e2052cefab0b55a32e3a4141c01701d7cbc5f1230` 和
`10e0af85e3e3825647733f99d573c969d69ccaf47203365e51323fce07008baa`。

- RelationshipClassName 的 IName 虚表 `0x70adb990` 的 Open
  `0x70a3b060` 打开工作空间名称，查询 `IFeatureWorkspace`
  （IID 位于 `0x70ab85a4`），调用虚表 `+0x2c` 打开关系类，最终在
  `0x70a3b175` 查询返回对象的 IUnknown。
- FileGDB Workspace 在 `0x6fa8583f` 开始的 QI 分支返回 IFeatureWorkspace
  虚表 `0x6fb02a00`，`+0x2c` 的 `0x6fa8e1f0` 跳至对象 `+0x50`
  的虚表 `0x6fb02930` 中的 `0x6fa8dd70`。当基数为 3 或属性标志非零时，
  `0x6fa8e03c` 调用 AttributedRelationshipClass 构造器 `0x6fa01560`，
  后者在 `0x6fa015a1` 调用导出的 FdbAttributedRelationshipClass 基类构造器。
  GdbCoreLib 为该基类导出 ITable 虚表，10.1 类型库也声明
  AttributedRelationshipClass 实现 ITable。
- 共享表名槽复用 RelationshipClassName 的既有读法，保留原始基数、属性标志
  和工作空间或数据集槽。原生打开时查询的数据集定义决定是否返回有属性关系类，
  因此读取阶段不以名称体里的标志决定对象长度或拒绝已确认的名称布局。
  合成 StandaloneTable v0 检查表名体、空工作空间槽、键登记与后继字段。
- MemoryRelationshipClassName 的 IName Open `0x70a3fe30` 打开两个端点并调用
  MemoryRelationshipClassFactory。对应的 MemoryRelationshipClass 主 QI
  `0x70a3c220` 及后备 `0x70a60800` 均无 ITable 分支，10.1 类型库亦未声明
  ITable，故该名称只由关系名称槽等已有宿主处理。

同批 135 份工程的两条根流均读至末尾，未命中 RelationshipClassName CLSID。
有属性关系类的 ArcMap/QGIS 人工验收尚未进行。

### 独立表中的 TimeExtent 停止点

StandaloneTable v12 起复用 LayerLib TimeData 族读取器。
TimeTableDefinition Load `0x7870df80` 的 ITimeExtent 槽
委派到 SystemCore 10.1.0.3035 的 TimeExtent Load `0x7ffe81f0`。
其端点和时间参考的任一子槽未闭合时，TimeTableDefinition
的后继段无法定位。共享读取器现在在整个 ITimeExtent 槽
起点停止，保留已消费的 TimeData 和其他独立表前缀，
并撤销 TimeExtent 及其子对象的键。

合成 StandaloneTable v12–v14 测试覆盖非空 TimeExtent、
内嵌 Time 和 TimeReference、后继时间段与 v14 扩展边界。
替换为未知时区后，验证恢复到外层槽起点并撤销嵌套键。
尚未在 ArcMap 10.1 或 QGIS 人工验证这组对象。

## StandaloneTable 版本门

Map.dll 10.1.0.3035 的 Ghidra 导出 `Map/7945d630.c` 中，
`StandaloneTable::Load` 读入有符号 `short`，只拒绝大于 14 的值。
关系循环的首个版本门为 `1 < local_8[0]`，此后的版本段也按
有符号值比较。负值因此与 0、1 共享首个 IName、字段表和尾字符串布局。
读取器保留版本原始位模式。合成测试验证两种负值的消费边界和逐字节
截断回滚；介质中尚未确认带负版本值的独立表，ArcMap 10.1 和 QGIS
人工验收未进行。

## 字段表回引

Map.dll 10.1.0.3035 的 `0x7945d630` 在命名循环中用
`DAT_7948dcd0` 请求 IFieldInfo。SystemCore.dll 同版本读侧在
`0x7ffd0063` 查键，命中时对原实例查询接口。命名槽现在依据已登记类
判断回引是否属于已支持的 FieldInfo；只有键而缺少类信息时保留既有行为。
合成测试覆盖回引头和登记类不一致的两个方向，并验证不适用类的停止
位置和引用表。ArcMap 10.1 与 QGIS 人工验收未进行。

## 独立表的有符号计数

Map.dll 10.1.0.3035 的 Ghidra 导出 `Map/7945d630.c` 在读取字段表、
关系名和压缩序列后，分别按有符号 `local_24`、`local_28` 和
`local_18` 是否大于零决定进入循环。压缩序列还在正值分支内分配
展开缓冲区。读取器对这三处的非正计数跳过循环，并保留序列原始
计数。合成 v5 测试覆盖 0、-1 和最小有符号值，验证后续整数和
第二名称槽的边界。原生实档与 ArcMap/QGIS 人工验收待验证。

Map.dll 同一函数在 v10 起调用 LayerLib.dll 的
`CInternalLayerBase::ReadPreJoinDefExpressions`（`0x786c5d30`）。
该辅助函数只对正计数读取字符串对，见[委派函数证据](mxd-boundary-luna.md)。
独立表读取器据此对负计数返回空表达式列表。合成 v11 测试验证下一条
非空字符串、尾对象槽和截断回滚；ArcMap 10.1 与 QGIS 人工验收未进行。

## 独立表的未知扩展

Map.dll 10.1.0.3035 的 `StandaloneTable::Load`（`0x7945d630`）在
v14 委派 LayerLib.dll 的 `ExtensionHelper::LoadExtensions`
（`0x786c22e0`）。该函数在无法创建扩展实例时按声明的体长跳过，
见[原生读写证据](arcmap-10.1-save-side.md)。独立表按同一长度保留
未识别类的原始体并继续读取，长度受剩余输入及扩展预算约束。已识别
扩展体的内部不支持仍保留原停止行为。合成测试覆盖已知扩展的共享键、
未知体及下一条扩展，并验证短读回滚。ArcMap 10.1 与 QGIS 人工验收未进行。

用 objdump 复核 Desktop 介质中的 x86 LayerLib.dll。PE 版本资源为
10.1.0.3035，SHA-256 为
`55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`。
`0x786c2360..0x786c2369` 读取四字节计数，`0x786c237e` 与零比较，
`0x786c2386` 用有符号 `jle` 跳过扩展条目。独立表读取器据此保留
原始计数，并在非正计数时读完列表。合成 v14 测试验证两种负值、
此前的引用键和下一字节的位置。ArcMap 10.1 与 QGIS 人工验收未进行。

## 独立表尾槽回引

Map.dll 10.1.0.3035 的 `StandaloneTable::Load` 在 `0x7945dd8b`、
`0x7945dde1` 和 `0x7945de09` 分别请求显示表达式属性、UID 和
标注属性集合接口。SystemCore 在 `0x7ffd0063` 按键查询登记实例。
三个槽现在各自检查登记类，使用已支持类作为接口兼容性的静态依据。
合成 v11 测试验证每个槽接受或拒绝头部类标识与登记类不一致的回引，
以及停止位置和引用表。ArcMap 10.1 与 QGIS 人工验收未进行。
