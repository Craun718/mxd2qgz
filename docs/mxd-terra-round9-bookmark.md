# 第九轮：FeatureBookmark 的 IName 与 TableName

日期：2026-11-17。此处为静态原生证据和合成测试，不是目标软件运行记录；ArcMap 10.1 暂不可用，QGIS 也未做目视检查。仓库参考样本书签计数为 0；没有声称在书签中实际观察到 TableName 或证明名称对象的分布。

## 身份与方法

本轮重算 Map.dll（`temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`）SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，以及 GdbCore.dll（`temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`）SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；两者 PE32 x86，文件/产品版本均声明 `10.1.0.3035`，原始文件名分别 Map.DLL、GdbCore.DLL，与前轮一致。这些是静态身份和自述版本，不是签名认证。`third_party/ForestRight/vendor/ESRI.ArcGIS.System.dll` 的 SHA-256 为 `657c2072703fa9904a9f25bc56c4a46425b86ba2af89f9e7c3faddd6a307a766`，文件和产品版本资源均为 10.1.0.3035。GdbCoreLib 的同版身份及持久化链见 [第八轮](mxd-terra-round8-tablename.md)，本轮未重新计算其哈希。

独立 Ghidra 桥接工具 6.9.0 CLI、JDK 25、`/opt/ghidra`、仓库内专用 TMPDIR 分析 Map.dll Load；run `d4dda94a-5c97-4f39-b047-a2736cc97f45` 的 Ghidra 桥接 Unix socket 报 `Operation not permitted`，未产生函数 Evidence。按预定回退使用复制的独立 Map/GdbCore 工程、pyghidra/Ghidra 12.2 和 JDK 25，隔离项目、缓存及临时目录，重新反编译目标及辅助函数；另用 PE 原始虚表、objdump 指令和既有类型库转储核查。未连接或切换共享 Ghidra 桥接工具 目标。`temp/terra-round9-bookmark/` 为忽略的临时证据，分享前应检查敏感路径。反编译不是原始源码。

## 构造、虚表、读写与语义

1. Map `0x793b5560` 构造完整对象，在 +0x10 安装 `0x79482bac`、+0x14 安装 `0x79482be8`；后者虚表 +0x14/+0x18 分别指向 Load `0x793b5c50` / Save `0x793b5d70`，+0x0c 的 `0x793b5520` 写出 FeatureBookmark CLSID。构造初始化完整对象 +0x18/+0x1c 为零，+0x20 初始化字符串；Load 的 this 是 +0x14 视图，因此 Load this+4/+8/+0xc 正好对应完整对象 +0x18/+0x1c/+0x20。
2. Load 依次读 short、int32、请求 IName `{4BA33331-D55F-11D1-8882-0000F877762D}` 的 LoadObject 和字符串。Save 写版本 1、int32、名称对象和字符串。`0x793b5c8c..91` 是有符号 `cmp 1; jg`：**负值也通过原生版本门**；旧文档称负值拒绝是错误的。读取器按同样的有符号上限比较，并保留原始位模式。构造用例覆盖负值、版本 2 拒绝及后继字段边界。负值版本尚无真实旧档验证。
3. +0x10 视图虚表 `0x79482bac` 的 +0x1c/+0x20 指向 `0x793b5ba0`/`0x793b5bd0`；两者分别读取/写入该视图 +8，即完整对象 +0x18，与持久化 int32 同一成员。尚未把访问器与类型库成员名可靠对位，因此不把它命名为特定业务属性。
4. Load `0x793b5d13..2a` 对加载的 IName 指针调用其 +0x14 槽，传入输出指针；结果经 `0x793acdc0` / `0x793ad1c0` 对返回对象查询 IFeatureClass `{D4803EE6-79F4-11D0-97FC-0080C7F79481}`，成功保存该接口，失败或空值使目标引用为空。该调用的 HRESULT 未被此处用于终止读取。旧 [Legend 记录](arcmap-10.1-legend-family.md)声称 IName 只有 get_NameString、该槽越过接口的方法数，这一判断不成立；QI 是针对调用所得对象，而非直接对 IName 本身。
5. GdbCore `0x709b9330` 构造 FdbTableName：完整对象 +0x10 安装 `0x70ad071c`；QI `0x709b9647..64` 将 IName 映射到该视图。该虚表 +0x0c/+0x10 为跳板 `0x70a63626`/`0x70a6362c`，导入符号分别为 `FdbDatasetName::put_NameString` 和 `get_NameString`；+0x14 为 `0x709b9910`。新反编译显示后者检查工作空间引用、对名称对象再调用 +0x14 并在得到的对象上按字符串访问，最后向输出参数查询接口；主审提供的 CLR COM 元数据提取经本轮复核：`dotnet temp/terra-round9-iid/probe.dll third_party/ForestRight/vendor/ESRI.ArcGIS.System.dll 4ba33331-d55f-11d1-8882-0000f877762d` 输出 IName 的 GuidAttribute 与方法顺序 `set_NameString`、`get_NameString`、`Open`；结合 IUnknown 三槽和 x86 虚表，+0x14 确认为 IName::Open。不保证该路径返回 IFeatureClass；TableName 可持久化为 IName 这一事实不证明其能形成有效要素书签。
6. TableName 的 CLSID、IPersistStream 视图、经 GdbCoreLib `FdbDatasetName::Load/Save` 的版本、三字符串及 IWorkspaceName 体见 [第八轮](mxd-terra-round8-tablename.md)。本次重新核对构造和 IName 虚表，使其与书签的请求接口相连；未以其他 IName 实现的名称推断其可用性。

## 实现与验证

`map_surround` 中书签名称槽一次读取对象头并按空值、回引、FeatureClassName、TableName 分派；TableName 复用 `standalone_table::read_table_name_body_10_1`，该函数仅扩大为模块内可见。TableName 嵌套工作空间或其子对象未读完时书签返回 Unsupported，整个书签游标和引用表回滚；不模拟原生实例创建、Open、QI 或数据源访问。由于限定修改范围，原先 `dataset_name_more` 的独立书签槽助手没有扩展；此新能力属于 FeatureBookmark 入口。TableName 版本门见[独立表名称取证](mxd-terra-round8-tablename.md#tablename-版本门复核)。

`tests/terra_round9_bookmark.rs` 的合成测试覆盖 TableName 空工作空间、回引、新 WorkspaceName、嵌套未知对象停止、截断、版本及字符串限额，核对成功位置和失败事务回滚。专门测试 3 项通过；`cargo fmt --check` 通过；`cargo test --offline` 全量通过（794 项通过、16 项默认忽略）。

可单独落账：FeatureBookmark 10.1 的 IName 请求、构造/虚表定位、名称对象调用后对结果 QI 的静态路径；TableName 的 IName 视图和此宿主下可完整读取子集的分派。仍不可将「FeatureBookmark 全部 IName 具体实现及实际分布」「非空书签真实读取」「int32 业务属性」「ArcMap/QGIS 人工验收」整体勾选。其他实现、真实打开结果及软失败行为仍需目标运行取证。

## 栅格目录与关系查询名称

复核 Desktop 10.1.0.3035 x86 的 RasterCatalog.dll（SHA-256
`0b6ac9b5e0c3e8efb715d68c5a55de89275def9ec7f666ffae91f0ff7972c585`）
及 GdbCore.dll（SHA-256
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`）。
以 objdump、PE 虚表原字节和 esriGeoDatabase 类型库交叉核对。

- RasterCatalogName 的构造在 `0x7be36002` 创建 FeatureClassName，
  `0x7be36237..0x7be3623f` 将 IName 查询转发给该成员。其持久化
  Load `0x7be36460` 直接委派同一流，内部名称没有第二个对象头。
  GdbCore 的 FeatureClassName IName 虚表 `0x70ad01bc` 的 Open
  槽指向 `0x709b42c0`。10.1 esriGeoDatabase 把 RasterCatalog
  （`55E13E7E-52BD-4145-9ADF-9CF8B776C346`）列为 IFeatureClass 实现。
- RelQueryTableName 的 IName 虚表 `0x70add028` 的 Open 槽指向
  `0x70a5a0d0`。调用打开源关系，并依方向在关系对象虚表
  `+0x20/+0x24` 取表，再在 `0x70a5a389` 附近查询返回对象的
  IUnknown。类型库把 RelQueryTable
  （`365F6148-1D24-11D3-9F5C-00C04F6BC886`）列为可提供
  IFeatureClass 的类。该接口能否取得取决于关系查询及其底层表。
  名称的 Load/Save `0x70a5a530/0x70a5a6f0` 已有
  [对象体证据](arcmap-10.1-dataset-name-slots.md#relquerytablename-的非空关系类名称)。

FeatureBookmark 和 CustomOverlayGrid 两处复用已有名称体读取器，
保留外层身份及关系名称回引。合成书签覆盖栅格目录委派体、关系查询
v3 尾字段、后续书签名、未知关系类与截断回滚。合成 MapFrame 覆盖
栅格目录格网来源及其后续字段。135 份仓库参考及 10.1/10.8 介质工程的
`/Maps` 和 `/PageLayout` 均读至结束，两类 CLSID 字节均未命中。
原生关系查询实际返回的接口、实档对象体以及 ArcMap/QGIS 人工验收仍待核对。

## FileGDB 与查询名称的后续接线

再次核对 Desktop 10.1.0.3035 的 FileGDB.dll 和 GdbCore.dll，摘要分别为
`73cef6bac97b0bdba8e4b26e2052cefab0b55a32e3a4141c01701d7cbc5f1230` 与
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`。

- FgdbFeatureClassName（`75E10086-4226-42AC-AFEC-CBB9B748F847`）构造器
  在 `0x6fa31731..0x6fa31745` 以外层对象为聚合者，创建 GdbCore
  FeatureClassName 并保存其非委派 IUnknown。外层 IPersistStream 仍使用独立的 `0x6fafe0c4` 虚表和
  `0x6fa33290` Load，因此书签直接复用读取 FileGDB 包装和内嵌名称体的入口。
- TableQueryName 的 QI `0x708f4960` 在 `0x708f4967` 比对
  `0x70ab84e4` 的 IName IID，返回对象 +0x10；该视图的 Open 槽
  指向 `0x708f4ab0`。FeatureQueryName 的 QI `0x708f5700` 对
  该请求转入同一基类入口。两类的 IPersistStream 读写顺序已由
  [查询名称取证](mxd-terra-integrate-query.md)核对，按流内 CLSID 复用。

合成书签覆盖三个非空名称、书签字符串的后继边界及未知工作空间和截断时
的事务回滚。参考工程和 199 个介质流里有 FileGDB 名称的类标识命中，
尚未通过结构遍历确认它在书签槽内出现，两种查询名称也没有类标识命中。
其余 IName 实现仍需核对，软件实际打开和人工验收待补。

## ObjectClassName 与 QueryTableName 的后续接线

2026-11-21 继续按书签宿主核对两类已闭合名称体。证据仍为 Desktop
10.1.0.3035 的静态反编译和合成流，没有在 ArcMap/QGIS 中打开，也没有在
真实书签中观察到这两类新对象。

- ObjectClassName `{905CEC64-0B2C-11D4-A060-00C04F6BDD84}` 的 GdbCore
  注册工厂初始化位于 `0x70a808c5`，写入的类名是 `ObjectClassName`；
  工厂实际创建实例的 RTTI 为 `.?AVFdbTableName@@`。其持久化因此复用
  TableName/FdbTableName 的 GdbCoreLib `FdbDatasetName::Load/Save` 跳板
  `0x70a63662/0x70a63668`，对应导出 `0x70d4e980/0x70d4ea10`：有符号版本
  门（本读取器保守拒绝负值）、三条字符串和 IWorkspaceName 槽。书签读取器
  复用 `read_table_name_body_10_1`，但保留
  `BookmarkNameEntry10_1::ObjectClassName`，避免把流内身份折叠为
  `TableName`。参考样本和已遍历介质对象中没有该类书签实例。
- QueryTableName `{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}` 由 GdbCore
  GetClassID `0x709329d0`、Load `0x70932a10`、Save `0x70932ae0` 定名。
  Load 的无符号版本门仅接受 1/2；体依次为三条字符串、IWorkspaceName
  槽和 IQueryDescription 槽。工作空间只接入空值、回引和已闭合
  WorkspaceName；查询描述只保留空值与回引，新对象没有已确认体，作为停止点
  返回 `Unsupported`。

`tests/terra_round9_bookmark.rs` 新增 ObjectClassName 的空工作空间、回引、
新 WorkspaceName 和逐前缀截断测试，以及 QueryTableName v1/v2、查询槽
空值/回引、未知查询描述和逐前缀截断测试。所有失败均断言书签游标停在 0、
引用表不变。定向测试 13 项通过，`cargo fmt --check` 通过。

原生 FeatureBookmark 在 `LoadObject` 后调用 IName::Open，再对返回对象查询
IFeatureClass；本次仍只校验 IPersistStream 对象体，不模拟打开数据源、对象
替换、Open 或 QI。因此能够完整读取名称体不等于该名称在 10.1 中一定能返回
IFeatureClass。其余已注册 IName 实现和真实书签运行结果仍作独立开放项。

## TrackingWorkspace 的 AMSDatasetName

Desktop 介质中的 `TrackingWorkspace.dll` 为 PE32 x86，版本资源的文件版本和产品
版本均为 10.1.0.3035，原始名 `trackingworkspace.DLL`，SHA-256
`892c7b52d27d0f0c3d2f4ee8954273cc603e744d1ee79292183a4517eea21e8e`。
在既有反编译导出之外，用 PE RTTI、原始虚表、导入表和 objdump 核对本模块。
`esri.clsid.xml` 在该 DLL 下注册 AMSDatasetName
`{A677AB67-2FB8-11D5-B7E2-00010265ADC5}`。同值由
`0x80ad158c` 虚表的 GetClassID `0x80a61290` 写出，Load/Save 位分别指向
`0x80a612d0`/`0x80a61450`。RTTI 将对象的 +0x8 视图
`0x80ad151c` 对位为 IName。其 Open `0x80a60d70` 在
`0x80a60de8` 依据数据集类型值 5 分派到 IAMSWorkspace 的 +0x14 槽，
对应虚表 `0x80ad1d44` 的 `0x80a63bd0`。后者可在
`0x80a63cf9..0x80a63cff` 对工作空间内取得的对象查询 IFeatureClass。
书签能否成功打开要素仍取决于工作空间和数据。

Load 与 Save 无对象体版本字段，顺序相同。字段偏移相对于 IPersistStream 视图。

| 流内顺序 | 原生证据 |
| --- | --- |
| ATL BSTR ×4 | Load `0x80a61ee0`，Save `0x80a61b00`。依次对应 +0x2c、+0x28、+0x40、+0x48。u32 字节长度 0 表示空指针，非零长度包含两个终止字节。Load 接受 2 至 0x100000，可装载奇数长度的原始 BSTR 载荷。 |
| int32 ×3 | +0x3c、+0x38、+0x44，读写均走 AfCore 整数重载。 |
| IWorkspaceName | `0x80a61ce0` 读 ATL Variant，AMSDatasetName 要求 VT_UNKNOWN（13）及非空值。下游 `0x80a61b70` 经 ReadClassStm 读 16 字节 CLSID，激活类、查询 IPersistStream，并以同一 IStream 调 Load。Save `0x80a61840` 经 `0x80acd4c8` 导入的 OleSaveToStream 写对象。 |
| int16 | +0x30，位于内嵌 WorkspaceName 完整体之后。 |

TrackingWorkspace 在 `0x80a655fa`、`0x80a6f78e` 和 `0x80a712e5`
直接请求 IID `{FADD975C-E36F-11D1-AA81-00C04FA33A15}`，用
`0x80ad16e0` 的 `{5A350011-71E3-11D1-AA82-00C04FA33A15}` 创建
GdbCore WorkspaceName。书签分派复用其 Load 读取器。OLE 头只有 CLSID，
内联工作空间自身无 ObjectStream 键。工作空间体内部用 AfCore LoadObject
读 IPropertySet，原流仍带会话接口，因此继续共享外层登记表。
原生 Load 可接受其他实现 IPersistStream 和 IWorkspaceName 的类。Rust 当前在
未知 CLSID 处返回 Unsupported，字符串和未对位标量保留原始值。

合成书签检查了内嵌工作空间与共享属性集键，并覆盖逐前缀截断、BSTR 限额、
错误 Variant 类型、空和未知工作空间的事务回滚。参考工程与 134 份安装介质
工程的 /Maps 和 /PageLayout 均读取到流尾，两流均未扫描到 AMSDatasetName
的 CLSID。仍待真实书签宿主字节和 ArcMap 10.1/QGIS 人工验证。

## 关系类名称的后续接线

继续在 FeatureBookmark 的 IName 槽接入 GdbCore.dll 10.1.0.3035 的
RelationshipClassName `{311C2EE1-DA65-11D2-8A54-000000000000}` 与
MemoryRelationshipClassName `{6DBA211B-EBDB-11D3-9F84-00C04F6BC886}`。
对象体字段序、Load/Save 地址及版本门沿用
[IDatasetName 槽实现记录](arcmap-10.1-dataset-name-slots.md#字段序load-反编译save-逐字段对称互证)：

- RelationshipClassName Load `0x70a3b330`：版本、AfCore String 三条、
  int32 两条、int16 两条、int16 选择子。选择子为零时保存
  IWorkspaceName，非零时保存 IDatasetName；两个分支分别对应
  WorkspaceNameSlot10_1 和 DatasetNameMoreSlot10_1。
- MemoryRelationshipClassName Load `0x70a402e0`：版本、AfCore String
  三条、int32 两条、int16 两条、两个请求 IName 的 LoadObject 槽，以及
  正向路径标签、反向路径标签、OriginPrimaryKey、OriginForeignKey 四条
  AfCore String。端点槽复用表、要素类和查询名称读取器，并共享外层书签
  的引用表。

书签分派保留两个独立枚举变体，不把类别折叠成查询表名称。两个读取器在
嵌套槽返回停止点时，外层 FeatureBookmark 返回 `Unsupported`，候选游标和
引用表均不提交。`tests/terra_round9_bookmark.rs` 覆盖关系类的零/非零
选择子，空值、回引和新建 WorkspaceName，内存关系类的新 FeatureClassName
端点、表名称回引及四条尾串；未知嵌套类和每个截断前缀均断言整个书签回滚
到偏移 0 且引用表不变。定向测试 15 项通过。

10.1 类型库确认两个类都实现 IName、IPersist、IPersistStream 和
IDatasetName，但没有直接实现 IFeatureClass。FeatureBookmark Load 的
后续仍会调用 IName::Open，再对打开结果查询 IFeatureClass。因此本次只证明
持久化对象体可完整读取并完成宿主接线，不证明 Open 成功、返回对象提供
IFeatureClass，也不证明真实书签中一定出现这两类名称。仓库参考样本没有
提供这两类的非空书签实例，ArcMap 10.1 和 QGIS 中也未做运行或目视验证。

## IName 回引与 FileGDB 接口复核

Map.dll 10.1.0.3035 的书签 Load `0x793b5c50` 使用普通 IName 槽。
同版 SystemCore 在 `0x7ffd0063` 按对象键查询登记实例的接口。
合成测试以 FgdbFeatureClassName 登记键，使用不同类标识的回引头，
验证书签保留登记类并正确读取后继名称。该测试未执行 IName::Open。

复查上述摘要的 FileGDB.dll 时，发现 `0x6fa3317d..0x6fa3318d`
所属函数是 FgdbTableName 主虚表 `0x6fafeca4` 的 QI
`0x6fa33080`。该分支比较的 `0x6fafe10c` 字节对应
`{9F9650F1-5F49-4041-BA0F-D10BAFF1D7BC}`。此前把这段指令
归为 FgdbFeatureClassName 的 IName 分支有误。FgdbFeatureClassName
的持久化虚表及 Load 由其自身类体证据确认。FgdbTableName 的 IName
路径以及两类名称在书签里的真实宿主仍待核对。

后续接线将 FgdbTableName `{846C88C5-6CC5-49C7-A6FE-8FA03D79BD07}`
直接加入书签名称分派。FileGDB.dll Load `0x6fa33290` 先读包装版本和
一个 short，再把同一流交给内联 TableName；`read_fgdb_table_name_body_10_1`
沿用该已闭合布局。合成书签验证包装字段、空工作空间、后继书签名、
版本越界和每个截断前缀的整书签回滚。真实书签实例、IName::Open 的
返回对象及 IFeatureClass 查询仍未运行验证。

同一轮把共享表名分派中已闭合、但书签入口漏掉的
RasterBandName `{9F9650F1-5F49-4041-BA0F-D10BAFF1D7BC}`、
SchematicDiagramClassName 与 SchematicElementClassName 接入书签。
RasterBandName 沿用版本标记、三串、选择子及工作空间/父名称槽；
两个 Schematic 类沿用版本、三串及 SchematicDatasetName 槽，并由
`read_schematic_class_name_body_10_1` 保持嵌套读取的事务边界。
合成书签覆盖两类 Schematic、栅格波段名称、共享键和逐前缀截断回滚。
仍未执行这些名称的 IName::Open、IFeatureClass 查询或目标软件验收。

### 公开 IName 实现全集审计

在同一套 10.1 Desktop 的 79 份 OLB 中枚举全部 MSFT 类型库，按
`IName {4BA33331-D55F-11D1-8882-0000F877762D}` 的接口继承筛选，
共得到 65 个公开 coclass。书签现有实现已经覆盖其中所有有证据可
产生要素类或表的名称：FeatureClassName、FgdbFeatureClassName、
CoverageFeatureClassName、RasterCatalogName、RasterBandName、
TableName、FgdbTableName、ObjectClassName、QueryTableName、
TableQueryName、FeatureQueryName、RelQueryTableName、
DBTableName、NetCDFTableName、NetCDFFeatureClassName、
XYEventSourceName、RouteEventSourceName、AMSDatasetName、
SchematicDiagramClassName、SchematicElementClassName、
CadDrawingName、GeometricNetworkName、RelationshipClassName 与
MemoryRelationshipClassName。

其余 41 个公开类由既有对象流研究归入以下非 IFeatureClass 来源：
WorkspaceName / CoverageName / FeatureDatasetName / TopologyName、
RasterDatasetName 系与 NetCDFRasterDatasetName、TinName /
TerrainName / LasDatasetName / CadastralFabricName /
NetworkDatasetName、AGS/IMS/WMS/WMTS/WCS 连接名、GP 名称、
LocatorName 系、SchematicWorkspaceName / FolderName /
DatasetName / DiagramName、RepresentationClassName、
UnknownDatasetName、FileName、ShortcutName 等。它们不代表书签
能打开要素；名称体若未接入时仍按未知新对象停在对象头，不跨对象推断。
该分类闭合的是公开实现清点，不替代原生 IName::Open、返回对象 QI
及真实非空书签的运行验证。

## NetCDF 表名称

NetCDFDB.dll 10.1.0.3035（SHA-256
`e78ea582fdd66f45dfa717eb22346e9f691e1bb4e26587a1d09f72b0d1bc5ebd`）
的 NetCDFTableName QI `0x7ae2eff0` 首先比较 `0x7ae5ffac` 的
IName IID，命中后在 `0x7ae2f015` 返回完整对象的 `+0x10`。
书签复用 Load `0x7ae2fdc0` 对应的已知表名读取器。
构造测试覆盖对象键、五个空子槽、后继书签字符串和截断回滚。
真实书签宿主和 Open 结果仍待 ArcMap 10.1 验证。

## 数据库表名称

GdbCore.dll 10.1.0.3035（摘要见 FileGDB 与查询名称段）的
DBTableName QI `0x708d7cc0` 在 `0x708d7d0b` 比较
`0x70ab84e4` 的 IName IID，命中后由 `0x708d7d25` 返回
完整对象的 `+0x10`。书签复用 Load `0x708d7a40` 对应的读取器。
构造测试从无版本字段的首串读到书签名称，并检查截断后游标和键回滚。
DBTableName 的实际书签宿主、Open 结果及 ArcMap 10.1 人工验证尚待补充。

## CAD 图纸名称

CadFDB.dll 10.1.0.3035（摘要及持久化入口见
[旧表名称取证](mxd-terra-round9-old.md#caddrawingname)）的
CadDrawingName 主虚表 `0x6cad93d4` 指向 QI `0x6cab6b60`。
该函数在 `0x6cab6b67` 取 `0x6cad911c` 的 IName IID，
命中后在 `0x6cab6b85` 返回对象 `+0x10`。
书签复用 Load `0x6cab7030` 对应的名称读取器。构造测试验证
三串、名称键、后继书签名及逐前缀截断的回滚。
原生 Open 的结果和实际书签工程仍待 ArcMap 10.1 验证。

## ReferencedTableName

MosaicDB.dll 10.1.0.3035 的 QI `0x7a59a9eb..0x7a59aa05`
返回 IName，书签复用 Load `0x7a59b300` 对应的包装体读取器。
同版 objdump 在 `0x7a59b32b / 0x7a59b330` 确认其版本上限
为有符号 short 1，共享读取器据此接受所有非正版本。
详见[模块摘要及子槽取证](mxd-terra-round9-old.md#referencedtablename)。
构造书签验证负版本、子表键、后继字符串及失败回滚。
实际书签宿主和原生 Open 结果仍待验证。

## 几何网络名称

GdbCore.dll 10.1.0.3035（摘要同数据库表名称）的
GeometricNetworkName 构造路径 `0x709b8633` 安装主虚表
`0x70ad05b8`。QI `0x709b87d0` 比较 `0x70ab84e4` 的 IName
IID，命中后在 `0x709b87f5` 返回对象 `+0x10`。书签复用 Load
`0x709b8ca0` 对应的读取器。构造测试覆盖名称键、空数据集槽、后继
书签字符串及截断回滚。实际书签工程和原生 Open 结果仍待验证。

## TopologyName

GdbCore.dll 10.1.0.3035（摘要同数据库表名称）的 TopologyName
RTTI 给出主虚表 `0x70aca364`，其 QI `0x7095e160` 在
`0x7095e167` 比较 IName IID，命中后在 `0x7095e185`
返回对象 +0x10。Load `0x7095e5c0` 的
`0x7095e5e7–0x7095e5ec` 采用有符号版本上限 1。
书签复用现有名称体读取器，并与其他宿主共用有符号版本门。
构造测试覆盖负版本、两个名称串、空 IDatasetName、后继书签
字符串和截断回滚。真实书签宿主、Open 返回对象和 ArcMap
人工验证仍待补充。

## FeatureDatasetName

GdbCore.dll 10.1.0.3035（摘要同 TopologyName）的
FeatureDatasetName 主虚表 `0x70ad03e8` 指向 QI `0x709b6560`，
命中 IName 时在 `0x709b6581` 返回对象 +0x10。
Load `0x709b7c90` 在 `0x709b7cc2–0x709b7cc7` 使用有符号
上限 1，接受负版本。书签复用原有名称体，包括四串、标志、
WorkspaceName 槽及子槽停止语义。构造测试覆盖负版本、空工作
空间、后继书签字符串以及逐字节截断回滚。实际书签宿主、
Open 返回对象和 ArcMap 人工验证仍待补充。

## CoverageName

CoverageFDB.dll 10.1.0.3035 的 CoverageName 主虚表
`0x6e181324` 指向 QI `0x6e0c2f30`。IName IID 不命中
外层专用分支，`0x6e0c2ff7–0x6e0c3007` 转交完整对象
+0x1c 的聚合接口。该成员连接由 `0x6e0c2e2b` 创建的
GdbCore FeatureDatasetName，后者已确认支持 IName。
外层 Load `0x6e0c3360` 在 `0x6e0c338e/0x6e0c3393`
使用有符号上限 1。详见[模块摘要及委派 Load](mxd-terra-round9-old.md#coveragefeatureclassname-与-coveragename)。

书签复用 CoverageName 读取器。构造测试覆盖负外层版本、原始 LONG、
委派的名称体、后继书签名和逐字节截断回滚。只登记外层实际对象键。
真实书签宿主和原生 Open 结果仍待 ArcMap 10.1 验证。

## WorkspaceName

同版 GdbCore.dll 的 WorkspaceName 主虚表 `0x70ada828`
指向 QI `0x70a26e30`。`0x70a26e37` 比较 IName IID，
`0x70a26e55` 返回对象 +0x10。Load `0x70a27950` 在
`0x70a2799d/0x70a279a2` 通过有符号 short 判断上限 1。
书签复用现有 WorkspaceName 读取器，包括路径缓存及子槽停止。
构造测试覆盖负版本、空属性集、后继书签字段及截断回滚。
真实书签宿主、原生 Open 结果和 ArcMap 人工验证仍待补充。

## RasterDatasetName

用 RasterDB.dll 10.1.0.3035 原始 PE 复核
`0x7c1335b8` 的 IName 虚表。类的主 QI 返回完整对象 +0x10，
持久化 Load `0x7c0ba5b0` 对应已确认的共享栅格名称体。
模块摘要、类标识及[原生字段顺序](mxd-luna-followup-rasterdatasetname-20261118.md)
与栅格图层分派相同。书签复用该体读取器及工作空间子槽处理。
构造测试覆盖控制值 4、四串、空工作空间、两个后继 LONG、
书签名及每个截断前缀的回滚。真实书签宿主和原生 Open
结果仍待 ArcMap 10.1 验证。

## RasterDatasetName83

同一 RasterDB.dll 10.1.0.3035 的 RasterDatasetName83 主虚表
`0x7c133858` 指向 QI `0x7c0bc8f0`。普通接口查询在
`0x7c0bc974` 委派给 `0x7c0b9af0`，IName 返回完整对象 +0x10。
书签复用 [Load 及可选 RasterIO 记录读取器](mxd-10.1-raster-layer-tail.md#rasterdatasetname83)。
构造测试覆盖有无记录、记录后的书签文本与对象边界、逐字节截断回滚。
真实书签宿主和原生 Open 返回对象仍待 ArcMap 10.1 验证。

## RasterBandCollectionName

复用同版 RasterDB.dll 的 IName 与 IPersistStream 取证，见
[波段名称与集合](mxd-10.1-raster-layer-tail.md#波段名称与集合)。
Load `0x7c052da0` 的共用读取器已接入 FeatureBookmark。
构造书签读取集合版本 2、内联 RasterDatasetName 和 LongArray，
确认波段索引按顺序保留重复项，三个键在同一对象表登记；
后继书签字符串及逐字节截断回滚均纳入测试。
真实书签宿主及原生 Open 结果仍待 ArcMap 10.1 验证。

## NetCDFRasterDatasetName

NetCDFDB.dll 10.1.0.3035（摘要同 NetCDF 表名称）中，
主 QI `0x7ae28bf0` 在 `0x7ae28bf7` 比较 IName IID，
命中后在 `0x7ae28c15` 返回对象 +0x10。Load
`0x7ae29e30` 在 `0x7ae29e74/0x7ae29e78` 按有符号 LONG
判断上限 2。书签复用[共享名称体](mxd-10.1-raster-layer-tail.md)，
该读取器据此允许负版本通过版本门。构造书签覆盖变量名、v1
特有字符串与包络槽、维度标签对象键、后继书签名及逐字节回滚。
真实书签宿主和原生 Open 返回值仍待 ArcMap 10.1 验证。

## CacheRasterDatasetName

MapServerLayer.dll 10.1.0.3035 的 CacheRasterDatasetName
主 QI `0x79e21ec0` 接受 IName，Load `0x79e22a40`
对应[共享栅格名称体](mxd-10.1-raster-layer-tail.md)。书签复用
该读取器。构造测试覆盖原始版本字、两串、空工作空间和瓦片读取器、
非空包络与后继书签字段，以及逐字节截断回滚。真实书签宿主和
原生 Open 结果仍待 ArcMap 10.1 验证。

## MosaicDatasetName

同版 MosaicDB.dll 的主虚表 `0x7a6026ac` 指向 QI
`0x7a3f4ab0`。`0x7a3f4af1` 引用 `0x7a5ff008` 的
IName IID，命中后在 `0x7a3f4b0c` 返回对象 +0x10。
书签复用 [Load `0x7a3f5b30` 对应的共享名称体](mxd-10.1-raster-layer-tail.md#mosaicdatasetname)。
构造测试覆盖负版本、v4 标量与字符串数组子键、后继书签名和逐字节截断。
真实宿主及原生 Open 结果仍待 ArcMap 10.1 验证。

## FgdbRasterDatasetName

FgdbRasterDB.dll 10.1.0.3035 的构造指令 `0x6f933a5a`
安装主虚表 `0x6f9474d0`。其 QI `0x6f933c50` 在委派名称
已经建立时，于 `0x6f933c84` 比较 IName IID，命中后
在 `0x6f933c94` 返回完整对象 +0x10。书签复用 Load
`0x6f934910` 对应的委派名称和 RasterIO 读取器。
见[模块摘要、记录边界和样本来源](mxd-luna-deep-fgdb-raster-20261118.md)。

回归测试将参考工程 /Maps 中 `0x500a..0x51d9` 的真实名称对象
装入构造书签，验证委派名称键、工作空间键、RasterIO 结束对、
后继书签名以及每个截断前缀的回滚。真实书签宿主及原生
Open 结果仍待 ArcMap 10.1 验证。

## FunctionRasterDatasetName

FunctionRasterDB.dll 10.1.0.3035 的主虚表 `0x6fd45750` 指向
QI `0x6fc9d8a0`，在 `0x6fc9d8b9` 比较 `0x6fd4422c`
的 IName IID，命中后在 `0x6fc9d8d4` 返回对象 +0x10。
书签复用 [Load `0x6fc9dcf0` 对应的名称体](mxd-10.1-raster-layer-tail.md#函数栅格名称)。
构造测试覆盖 IdentityFunction 新键及 v7 后置字符串，验证
后继书签文本和逐字节截断回滚。真实宿主和原生 Open
结果仍待 ArcMap 10.1 验证。

## ISRasterDatasetName

ImageServerLayer.dll 10.1.0.3035 的 ISRasterDatasetName RTTI
对应主虚表 `0x78031cb4`。QI `0x780123b0` 在
`0x780123c9` 比较 `0x78030630` 的 IName IID，命中后
于 `0x780123e4` 返回对象 +0x10。书签复用
[Load `0x78012bb0` 对应的共享读取器](mxd-10.1-raster-layer-tail.md)。
构造用例验证 v4 名称中的后置字符串、IdentityXform 新键、
后继书签名及逐字节截断回滚。真实书签宿主和原生 Open
结果仍待 ArcMap 10.1 验证。

## ImageServerName

同版 ImageServer.dll 的主 QI `0x77f4f4e0` 在
`0x77f4f529` 比较 IName IID，命中后于
`0x77f4f544` 返回完整对象 +0x14。书签复用
[Load `0x77f4f7f0` 的共享读取器](mxd-10.1-image-server-layer-tail.md#imageservername-对象体)。
构造测试覆盖普通属性集新键、书签名和逐字节截断回滚。
真实书签宿主和原生 Open 结果仍待 ArcMap 10.1 验证。

## AccessRasterDatasetName 与 SdeRasterDatasetName

同版 RasterJET.dll 的主虚表 `0x7c6fce38` 指向 QI
`0x7c6f5ac0`，在 `0x7c6f5b09` 比较 IName IID。
SdeRasterDB.dll 的主虚表 `0x7f62fce8` 指向 QI
`0x7f61d890`，委派成员建立后在 `0x7f61d8c4` 比较
`0x7f62e918` 的同一 IID。两类均返回完整对象 +0x10。
书签复用[各自的委派名称和可选 RasterIO 读取器](mxd-10.1-raster-layer-tail.md#access-与-sde-栅格名称)。
构造测试覆盖各类有无记录、Access 有 tag 7、SDE 直接结束、
后继书签名以及逐字节截断回滚。真实书签宿主及原生 Open
结果仍待 ArcMap 10.1 验证。

## InternetTiledLayerName 与 AISImageServiceName

MapServerLayer.dll 10.1.0.3035 的 InternetTiledLayerName QI
`0x79e36b30` 与同版 ImageServer.dll 的 AISImageServiceName QI
`0x77f011f0` 均把 IName 映到完整对象 +0x14。
[模块摘要、Load 与接口证据](mxd-10.1-raster-layer-tail.md#瓦片图层与-ais-影像服务名称)
已用于栅格名称读取器，FeatureBookmark 复用同一分派。
构造测试验证瓦片名称的 short、LONG 和空 ITileReader 及 AIS
名称的 LONG 和三串，检查后继书签文本、边界和逐字节截断回滚。
真实书签宿主及 Open 结果仍待 ArcMap 10.1 验证。

## WCSCoverageName

OGCClient.dll 10.1.0.3035 的 WCSCoverageName 主 QI
`0x7b424610` 将 IName 映到完整对象 +0x14。书签复用
[Load `0x7b424bd0` 的覆盖名称及连接槽读取器](mxd-10.1-raster-layer-tail.md)。
构造书签验证 BSTR 覆盖标识、v3 连接名称、属性集回引、书签文本和
逐字节截断回滚。真实书签宿主及原生 Open 结果仍待 ArcMap 10.1 验证。

## WMSConnectionName 与 WMTSConnectionName

同一 OGCClient.dll 10.1.0.3035 的 RTTI 和 esriGISClient 类型库
确认两类的 IName 视图均位于完整对象 +0x18，IPersistStream 位于
+0x1c 并委派给共用 Load `0x7b420ee0`。FeatureBookmark 复用
[现有 OGC 连接名称分派](mxd-10.1-raster-layer-tail.md)，保留原类标识。
构造两类 v3 连接，验证属性集新键和回引、后续书签名及逐字节截断回滚。
真实宿主、原生 Open 结果和 ArcMap 人工验证仍待补充。

## WCSConnectionName

OGCClient.dll 10.1.0.3035 的 WCSConnectionName RTTI 对应
主虚表 `0x7b44fb50`。QI `0x7b420a40` 的通用分支在
`0x7b420c1e` 委派给 `0x7b414c90`，后者在 `0x7b414d03`
比较 `0x7b453460` 的 IName IID，命中后于 `0x7b414d38`
返回完整对象 +0x18。直接书签槽沿用共用连接名 Load
`0x7b420ee0` 的读取规则。构造 v3 连接验证原 CLSID、
属性集新键与回引、后继字段和逐字节截断回滚。真实书签
宿主和 Open 结果仍待 ArcMap 10.1 验证。

## RasterTypeName

MosaicDB.dll 10.1.0.3035 的 QueryInterface `0x7a45b890` 将
RasterTypeName 的 IName 映到完整对象 +0x18。FeatureBookmark
复用[已确认的 Load `0x7a45c590` 读取规则](mxd-10.1-raster-layer-tail.md#rastertypename)。
构造书签验证两个字符串、有符号 InstanceID、内嵌 MosaicDatasetName、
后继书签文本和逐字节截断回滚。真实书签宿主、原生 Open 结果和
QGIS 人工检查待验证。

## NetworkDatasetName

ArcMap 10.1.0.3035 的 NALayer 通过同一 IName IID 读取真实
NetworkDatasetName 对象。GdbNetwork.dll 的 GetClassID `0x71044240`
和 Load `0x71044ff0` 已由[原生取证](mxd-luna-deep-layer-masking-20261118.md)
确认。书签复用 NALayer 已使用的对象体读取器，保留 IDatasetName
和 IWorkspaceName 槽。构造测试验证字段、后继书签文本和逐字节
截断回滚。真实书签宿主、原生 Open 结果和 QGIS 人工检查待验证。

## TinName 与 TerrainName

ArcMap 10.1.0.3035 的 3D 图层属性和 TerrainLayer 都通过
IName 装载地表名称。书签复用 [TinDB.dll Load `0x803540e0`](mxd-10.1-raster-layer-tail.md)
及 [Terrain.dll Load `0x80170130`](mxd-luna-deep-layer-masking-20261118.md#terrainname)
的读取规则。两类都读版本和三条字符串，TinName 的尾槽是
IWorkspaceName，TerrainName 的尾槽是 IDatasetName。构造测试覆盖
两类分派、空子槽、后继书签文本和逐字节截断回滚。真实书签宿主、
原生 Open 结果和 QGIS 人工检查待验证。

## LasDatasetName

使用 objdump 补查 10.1.0.3035 的 LasDataset.dll PE32，SHA-256 为
`2f116940021eb5264dcc578f0e42385b0e4f4fa6e0acfac62ced4cced4ca1f3d`。
RTTI `0x78604720` 指向 LasDatasetName。QI `0x785adc20` 比较
`0x785f4650` 的 IName IID，匹配后在 `0x785adc45` 返回完整对象
+0x10。IPersistStream 视图位于 +0x18，虚表 `0x785f45d4` 的
GetClassID `0x785ad990` 从 `0x785f38c0` 复制
`2029A994-9BC0-4094-A1C7-E5381869E4E3`，与类型库一致。

Load `0x785ae120` 读取 short，并在 `0x785ae14b` 以有符号上限 1
校验。在 `0x785ae16b / 0x785ae175 / 0x785ae17f` 读取三个 AfCore
String，再在 `0x785ae196` 调用 LoadObject，所请求 IID 为
`FADD975C-E36F-11D1-AA81-00C04FA33A15`（IWorkspaceName）。
Save `0x785ae080` 对称写出上述字段。构造测试覆盖负版本、
超上限版本、新建工作空间、后继书签文本与逐字节截断回滚。
三个字符串的业务含义、真实书签宿主、原生 Open 与 QGIS
人工检查待验证。

## CadastralFabricName

核对 CadastralFabric.dll PE32 10.1.0.3035，SHA-256 为
`ccaff2b3cd3379b7a7630ea6f2a41c06c75ef6f68969e8ec8f51f2f05133bfd6`。
objdump 与 PE RTTI 共同定位类描述符 `0x6c2d12e0`。
QI `0x6c1fa440` 在 `0x6c1fa4f7` 比较 IName IID，匹配时
于 `0x6c1fa515` 返回完整对象 +0x18。持久化虚表 `0x6c29d63c`
的 GetClassID `0x6c1fae00` 写出 `0x6c296400` 的
`5C947220-FFB5-4E71-A059-AF201AE081A6`，与同版类型库一致。

Load `0x6c1fae40` 依次读取有符号上限为 1 的 short、两个
AfCore String，再由 `0x6c1faed4` 和 `0x6c1faefe` 读取
IDatasetName（`89E75917-C287-11D1-AA77-00C04FA33A15`）和
IWorkspaceName（`FADD975C-E36F-11D1-AA81-00C04FA33A15`）。
Save `0x6c1faf20` 确认字段顺序。构造测试覆盖空数据集槽、
新建工作空间、后继书签文本、未知数据集类与逐字节截断回滚。
字符串业务含义、真实书签宿主、原生 Open 和 QGIS 人工验收待查。

## AGSServerObjectName 与 AGSServerConnectionName

ArcMap 10.1 的 esriGISClient 类型库列出两个 AGS 名称类的 IName，
CLSID 分别为 `6C22971C-D450-4B4D-9422-EE96A40FACC5` 和
`CBA35C3F-EDEF-408F-8B51-510784C78EB9`。书签复用
[AGSClient.dll 10.1.0.3035 的持久化布局](mxd-10.1-raster-layer-tail.md)
和瓦片读取器使用的对象体分派。objdump 核对对象名的 `0x697e85be`
与连接名的 `0x697e5335` 都比较 `0x698e8174` 的 IName IID，
分别在 `0x697e85f3` 和 `0x697e536a` 返回完整对象 +0x18。对象名 Load `0x697e9b30` 读取 BSTR
字段与连接名称槽，连接名 Load `0x697e6070` 读取属性集及 v2 的
AfCore 成员 String。构造测试覆盖直接连接、嵌入连接、对象键、
后继书签文本及逐字节截断回滚。真实书签宿主、原生 Open 和
QGIS 人工验收待验证。

## FileName

在[已核对身份的 SystemCore.dll 10.1.0.3035](mxd-terra-round9-old.md#filename-与-shortcutname-的旧表槽判定)
中，PE RTTI `0x800a62e4` 指向 FileName，持久化虚表
`0x8006f8dc` 关联 Load `0x7ffc3730` 和 Save `0x7ffc37b0`。
objdump 显示 Load 在 `0x7ffc376a` 比较有符号 short 上限 1，
再在 `0x7ffc3783` 调用 AfCore 自由函数 ReadString
（导入槽 `0x8006b228`，输出 BSTR）。Save 在 `0x7ffc37f7`
调用对应的 WriteString。构造测试覆盖负版本、超上限版本、
UTF-16 BSTR、后继书签字段及逐字节截断回滚。静态证据显示
FileName 的 Open 写空输出并返回 S_FALSE，真实书签出现情况
与 ArcMap/QGIS 人工检查待验证。

## ShortcutName

同版 SystemCore.dll 的 RTTI `0x800a6b78` 和持久化虚表
`0x800727fc` 将 ShortcutName 的 Load、Save 定位到
`0x7ffdc250`、`0x7ffdc310`。Load 对 short 使用有符号上限 1，
在 `0x7ffdc2a4` 读取 BSTR，在 `0x7ffdc2e4` 以
`0x8006f8fc` 的 IName IID 调用 LoadObject。Save 在
`0x7ffdc359` 写字符串，并于 `0x7ffdc387` 写出子对象。
GetClassID `0x7ffdbe50` 从 `0x8007282c` 复制
`3BEB09E4-3941-4A07-9D1A-EC2B43BA7D50`。

子槽沿用书签 IName 分派和同一引用表。为限制解析栈，Rust
对连续的新建 ShortcutName 设置 16 层上限。构造测试覆盖
FileName 子对象、登记类回引、书签文本边界和事务回滚。
静态证据显示 ShortcutName 的 Open 返回 S_FALSE 且输出为空。
真实书签出现情况及 ArcMap/QGIS 人工验收待验证。

## RepresentationClassName

objdump 补查 [RepresentationDB.dll 10.1.0.3035](mxd-terra-round9-old.md#representationclassname-在旧表槽中的-open)。
GetClassID `0x7cb284d0` 返回 `4E74AA22-A1B6-47D4-9914-20F8C6C2B699`。
持久化虚表 `0x7cb425e4` 的 Load `0x7cb29260` 在 `0x7cb2928b`
校验 short 的有符号上限 1，在 `0x7cb292ab / 0x7cb292b5 /
0x7cb292bf` 读取三个 AfCore String。LoadObject 调用
`0x7cb292ec` 使用 `0x7cb4262c` 的 IWorkspaceName IID。
Save `0x7cb29310` 同序写出。构造测试覆盖负版本、三条字符串、
空工作空间槽、后继书签字段和逐字节截断回滚。
真实书签宿主及 ArcMap/QGIS 人工验收待验证。

## SchematicDatasetName

ArcMap 10.1 的 esriSchematic 类型库将
`333DAE77-1AEB-447D-8E83-FE043E84A671` 列为 IName 实现。SchematicDB.dll 的 QI `0x7d6947b0` 比较
`0x7d7092f0` 的 IName IID，匹配时于 `0x7d6947d1` 返回对象 +0x10。
书签复用[由 SchematicDB.dll 10.1.0.3035 Load `0x7d695340` 确认的布局](mxd-terra-round9-table.md)，
读取有符号版本、三条 AfCore String 和 IWorkspaceName 槽。
构造测试覆盖后继书签名称、未知工作空间类和逐字节截断回滚。
真实书签宿主、原生 Open 和 QGIS 人工验收待验证。

## RouteMeasureLocatorName

ArcMap 10.1 的 esriLocation 类型库将
`8CC373A6-2121-11D4-9FC2-00C04F6BDF06` 列为 IName 实现。
书签复用[路线事件名称取证](mxd-terra-round9-old.md)所确认的
Load `0x7cc7cad0` 和基类 Load `0x7cc52540` 布局。基类的路线名称
沿用既有分派，遇未闭合子体时回滚整个书签。构造测试覆盖 v2
基类特有尾段、后继书签字段和逐字节截断。真实书签宿主、
原生 Open 和 QGIS 人工验收待验证。

## UnknownDatasetName

按 [GdbCore.dll 10.1.0.3035 镜像身份及既有 Open 取证](mxd-terra-round9-old.md#unknowndatasetname-在旧表槽中的-open)
复核 PE RTTI 和 objdump。类描述符 `0x70b34bd0` 属于
FdbUnknownDatasetName，注册 CLSID `06783DB1-E5EE-11D1-B0A2-0000F878082E`
位于 `0x70ac322c`。QI `0x709ba910` 的局部分支比较 IName 和
IDatasetName。回退 `0x70a60800` 比较的三个 IID 常量位于
`0x70a8841c`、`0x70a8846c`、`0x70ac8cd0`，均与
ObjectStream 普通对象请求的 IPersistStream IID `00000109-0000-0000-C000-000000000046`
不同。未匹配分支在 `0x70a608ec` 清空输出并返回 E_NOINTERFACE。
因此该内置实现的新对象没有可供普通书签对象流调用的 Load。

构造用例使用注册 CLSID 并紧接可解码的书签字符串，确认读取仍返回
Unsupported，回滚书签游标和新键，保留先前登记的键。
结论限于该镜像的内置实现。ArcMap/QGIS 人工验收未进行。

## SchematicFolderName

在已核对 SHA-256 的 SchematicDB.dll 10.1.0.3035 中，PE RTTI
`0x7d72a128` 关联持久化虚表 `0x7d70d574`。GetClassID
`0x7d6e7fd0` 返回 `9DAB0502-6139-4310-9FA0-1A0EDCB91C7B`，
与 esriSchematic 类型库一致。QI `0x7d6e7250` 比较
`0x7d7092f0` 的 IName IID，匹配时返回对象 +0x10。Load `0x7d6e80c0`
检查有符号 short 上限 1，于 `0x7d6e810b / 0x7d6e8115 /
0x7d6e811f / 0x7d6e8129` 读取四条 AfCore String，
在 `0x7d6e8140` 请求 `0x7d709370` 的 ISchematicDatasetName。
Save `0x7d6e8030` 的字段顺序与之对应。

书签复用已有的示意图数据集名称槽读取器。构造测试覆盖内嵌数据集、
两级对象键、后继书签字段边界和逐字节截断回滚。
字段业务语义、真实书签宿主、原生 Open 和 QGIS 人工验收待验证。

## SchematicDiagramName

同版 SchematicDB.dll 的 RTTI `0x7d729ba8` 对应
SchematicDiagramName，持久化虚表为 `0x7d70c2c0`。
GetClassID `0x7d6d7e00` 对应 CLSID
`E6B54966-9B84-42EE-B456-2E0DEB8159BC`，主 QI
`0x7d6d80f0` 对 IName 返回完整对象 +0x10。
Load `0x7d6d88c0` 检查有符号 short 上限 1，读取三个
AfCore String。随后调用 String::Empty 清空运行时字段，
并于 `0x7d6d8944` 以 `0x7d709380` 的 IID
`4560DEDE-1591-4699-BF88-D856FD46F194` 调用 LoadObject。
同版 SchematicDiagramClassName 的接口查询在 `0x7d6d6091`
引用该 IID。Save `0x7d6d8840` 写出三串和子对象。

构造用例覆盖三串、嵌套图表类名、后继书签字段以及错类和逐字节截断
回滚。真实书签宿主、原生 Open 和 QGIS 人工验收待验证。

## SchematicWorkspaceName

同版 SchematicDB.dll 的 RTTI `0x7d72a5f4` 关联持久化虚表
`0x7d70ebe4`。GetClassID `0x7d6f5ba0` 对应
`AF54E472-0A8A-4CDA-A9ED-B350A0BA9EBB`。QI `0x7d6f5520`
比较 `0x7d7092f0` 的 IName IID，在 `0x7d6f5545` 返回对象 +0x10。
Load `0x7d6f5c00` 检查有符号 short 上限 1，在
`0x7d6f5c4b / 0x7d6f5c55 / 0x7d6f5c5f` 读取三个 AfCore String，
然后在 `0x7d6f5c76` 用 `0x7d709350` 指向的 IWorkspaceName IID
调用 LoadObject。Save `0x7d6f5ca0` 对称写出三串和子对象。
书签复用与 SchematicDatasetName 相同的字段布局，保留类身份。
构造测试覆盖有符号版本、工作空间新键、后继书签字段及逐字节
截断回滚。真实书签宿主、原生 Open 和 QGIS 人工验收待验证。

## LocatorWorkspaceName

Geocoding.dll 10.1.0.3035 的 SHA-256 为
`e5dc7d6cf515b7c7e9fd9e0342780818d261c2762ebbd348c5c2fc318c71819c`。
PE RTTI `0x71550e98` 关联持久化虚表 `0x714fa8a0`，
注册 CLSID 为 `9685CAF0-BF13-4283-B388-A5730352C778`。
QI `0x71367150` 比较 `0x714f6454` 的 IName IID，匹配时返回
对象 +0x10。Load `0x71367ac0` 检查有符号 short 上限 2，
在 `0x71367b4f / 0x71367b5d` 读取两个 AfCore String。
v1 在 `0x71367b97` 通过 IName IID 读取普通对象，其余版本
在 `0x71367bcf` 请求 IUnknown。后续接口转换使用已读取的对象。

书签共用现有名称分派，消费空槽、回引或已支持名称的新对象。
LocatorWorkspaceName 与 ShortcutName 共用 16 层嵌套限制。
构造测试覆盖有符号版本、v1 内嵌 FileName、其余版本空槽、
后继书签字段边界和截断回滚。v2 IUnknown 槽的其他持久化类、
真实书签宿主、原生 Open 和 QGIS 人工验收待验证。

### IUnknown 子槽的接口复核

沿 Geocoding.dll 10.1.0.3035 的 Load `0x71367ac0` 跟踪
IUnknown 子对象的后续 QueryInterface。以同版 72 个 Desktop
类型库核对帮助函数的 IID 名称。

| QI 帮助函数 | 请求接口 |
| --- | --- |
| `0x71368170` | IWorkspaceName |
| `0x71368220` | IName |
| `0x713682d0` | IAGSServerConnectionName |
| `0x71368380` | IAGSServerConnection |
| `0x71368430` | IAGSServerObjectName |

后三个接口的公开内置类分别为 AGSServerConnectionName、
AGSServerConnection、AGSServerObjectName。前后两个名称类已有
持久化读取器。AGSClient.dll 10.1.0.3035（SHA-256
`3bebe94f58b3aed1dca1294aecaf8e080f235a92b9c6aa51376e16ae7cbe351c`）
中的 AGSServerConnection（CLSID `B49DEFD0-85C0-46F4-918B-1FB7E02A8C70`）
由 RTTI `0x6990d3d4` 定位。QI `0x697c83c0` 及其回退
`0x697c1230` 均无 IPersistStream 分支。普通对象流要求此接口，
所以读取器保持该类的新对象停止点。

构造 v2 书签验证可持久化的 AGSServerConnectionName 能读到
后继书签文本，将同一子槽改为 AGSServerConnection 则回滚。
原生转换路径、其他注册实现和真实书签中的子类仍待运行取证。

## LocatorName

同版 Geocoding.dll 的 PE RTTI `0x71550d68` 对应持久化虚表
`0x714f8978` 与注册类 `C24FFE64-88BE-11D3-9F66-00C04F6BDF06`。
GetClassID `0x7135d870` 从 `0x714f8a88` 复制该 CLSID。
QI `0x7135dbd0` 比较 `0x714f6454` 的 IName IID，在 `0x7135dbf1`
返回对象 +0x10。Load `0x7135e1e0` 检查有符号版本上限 1，
读取四条 AfCore String 后在 `0x7135e260` 以同一 IName IID
调用 LoadObject。Save `0x7135e280` 写出对应字段。

书签子槽复用已有名称分派及其 16 层深度上限。构造测试覆盖新建
LocatorWorkspaceName 子对象、后续书签文本、超上限版本和逐字节
截断回滚。四条字符串的业务含义、真实书签宿主、原生 Open
及 ArcMap/QGIS 人工验收待验证。

## GPDataTypeName

objdump 与 PE RTTI 复核使用 10.1.0.3035 的 FeatureDataElements.dll
（SHA-256 `7cd1153712d8af23f6c2263b6444c1a77e16b818fb5fb946d0c3d9106be5f429`）
及 DECoreLib.dll（SHA-256
`a57d35741decebe0765e8942268a934990763f52813b55eadf126de7d3042c2f`）。
RTTI `0x6f680704` 对应持久化虚表 `0x6f66102c`，CLSID
`960CB02A-8CAF-442B-ACF7-6B6256FC19F9` 由 GetClassID `0x6f6226c0`
从 `0x6f6595a4` 复制。QI `0x6f6228b0`
匹配 `0x6f65b8ec` 的 IName IID，返回对象 +0x10。

外层 Load `0x6f622b90` 读取有符号上限为 2 的 short，再委派
DECoreLib 的 GPName::Load `0x6e954a00`。外层版本小于 2
时，在基类之后于 `0x6f622c03` 额外读取 IUID 槽。
GPName::Load 检查有符号上限 3，读取四个 AfCore 自由函数
ReadString 的 BSTR。基类版本 3 于 `0x6e954ad8` 读取 IUID 槽，
版本小于 2 于 `0x6e954b34` 读取第五个 BSTR。两个 UID 槽
均请求 `1714D59B-FB22-11D1-94A2-080009EEBECB`。
Save `0x6f622c60` 和 GPName::Save `0x6e954c00` 对应新版布局。

构造测试覆盖版本组合、UID 新键和回引、后继书签字段和截断回滚。
字段业务含义、真实书签宿主、原生 Open 和 QGIS 人工验收待验证。

## GPMultiValueTypeName

GpObjects.dll 10.1.0.3035 的 SHA-256 为
`c9da22687b40e070d7178538846f707c04d26474d2fa835acb01ceaa46d2f896`。
PE RTTI `0x74800b5c` 关联持久化虚表 `0x747b25a4`，
GetClassID `0x746f0ca0` 对应
`35DE3370-678E-40FB-9B7E-FFAF08224B99`。QI `0x746f0810`
于 `0x746f0817` 比较 IName IID，于 `0x746f0835` 返回 +0x10。
Load `0x746f0d00` 读取有符号上限为 1 的 short，
再于 `0x746f0d68` 以 `0x747ab0ec` 的 IGPName IID
`17A47938-E1ED-4D82-A1D1-D274465FCBC4` 调用 LoadObject。
Save `0x746f0d90` 写出版本和同一个子槽。

书签共用名称读取和深度预算，并按 IGPName 类型约束接受已支持的
GPDataTypeName 和 GPMultiValueTypeName。构造测试覆盖内嵌 GP
数据类型、后继书签字段、错类及逐字节截断回滚。
其余 IGPName 实现、真实书签宿主、原生 Open 和 QGIS 人工验收待查。

## GPFunctionName

同版 GpObjects.dll 的 RTTI `0x7480046c` 对应持久化虚表
`0x747b01f4`，GetClassID `0x746e0ee0` 对应
`9A332F42-377F-4213-954B-B68CC8B9CE46`。esriGeoprocessing 类型库
声明该类实现 IName 与 IGPName。

Load `0x746e0f20` 读取有符号上限为 4 的 short，通过导入槽
`0x747a3650` 调用 DECoreLib 的 GPName::Load。外层版本小于 2 时在
`0x746e0f99` 读取 IUID。v3 起在 `0x746e101d / 0x746e1051`
读取两个 32 位值，中间在 `0x746e1039` 调用 ReadString 读取 BSTR。v4 在 `0x746e1080` 再读一个 32 位值。
Save `0x746e10e0` 对应新版布局。截断时回滚整个书签及引用表。

构造测试覆盖外层 v1–v4、共享 GPName 基类和 GPMultiValueTypeName
中的嵌套 v4 对象。数值业务含义、真实书签宿主、原生 Open
和 ArcMap/QGIS 人工验收待验证。

## GPSATopoFeaturesTypeName

GPSACore.dll PE32 10.1.0.3035 的 SHA-256 为
`0aa35b1be22c61742f36a651679e3028c363c40eebe859cda285fa6c6c2a5292`。
PE RTTI `0x74946cd8` 对应持久化虚表 `0x749319a8`。
GetClassID `0x748f9960` 对应
`AA10557F-0B72-4335-90AD-00655B0D8BBE`。QI `0x748f9600`
对 IName、IGPName、IGPDataTypeName 分别返回 +0x10、+0x14、+0x18。

Load `0x748f99c0` 的有符号版本上限为 4，v4 仅消费版本。
较早版本读取有符号记录数，非正数跳过循环。每条在
`0x748f9a43` 请求 IGPDataTypeName，版本小于 2 在
`0x748f9a6d` 另请求 IGPDomain，随后读 AfCore 成员 String
与 LONG。v3 在 `0x748f9ad3` 追加 IUID。Save `0x748f9b70`
写 v4。当前 v2–v3 分派支持已确认的数据类型和 UID，
更早版本已支持空域以及下述 GPNumericDomain、GPFileDomain、GPRangeDomain 与 GPWorkspaceDomain 的已确认布局，遇其余未确认类时回滚。

构造书签验证多值名称中的 v2–v4 子对象、旧记录、后继字段边界
和逐字节截断回滚。旧域子体、真实书签宿主、原生 Open 及
ArcMap/QGIS 人工验收仍待验证。

同版 72 个 Desktop 类型库共列出四个公开 IGPName 类，
GPMultiValueTypeName 子槽已接入这四类。GPSATopoFeaturesTypeName
内部旧 IGPDomain 槽作为独立缺口继续跟踪。

### GPNumericDomain

同版 GpObjects.dll 的 RTTI `0x74800ecc` 对应持久化虚表
`0x747b296c`，类标识为 `ADDDE180-3922-48F7-87C8-342FFAE41161`。
QI `0x746f2390` 在 `0x746f23c9` 比较 IGPDomain IID，匹配时
返回完整对象 +0x10。Load `0x746f2800` 接受有符号版本 <=1，
随后按 short、double、short、short、double 读取，
两个 double 的导入槽为 `0x747a31b0`。Save `0x746f28c0` 同序写出。

书签旧记录读取空、新建 GPNumericDomain 或登记类匹配的回引，
保留 double 的磁盘位。构造测试覆盖三种槽值、后继字段边界及
逐字节截断回滚。其余域类以及 ArcMap/QGIS 人工验收待查。

### GPFileDomain

同版 GpObjects.dll 的 PE RTTI `0x7480041c` 对应持久化虚表
`0x747b0068`，类型库 CLSID 为 `3EDC8E3A-FF87-47E1-AF34-09BB4CB22DA9`。
QI `0x746df850` 于 `0x746df889` 比较 IGPDomain IID，
匹配时返回 +0x10。Load `0x746e0490` 检查有符号 short 上限 1，
于 `0x746e04dc` 读取有符号 int32 字符串计数，非正数跳过循环。
循环在 `0x746e0516` 调用 AfCore 成员 String::Read。
Save `0x746e0560` 对应同一字段顺序。

构造旧版拓扑记录验证有符号计数、多个字符串、后继字段边界及
截断和超量计数回滚。真实 ArcMap 书签及 QGIS 人工验收待验证。

### GPRangeDomain

同版 GpObjects.dll 的 RTTI `0x74800ecc` 对应持久化虚表
`0x747b3e20`，类型库 CLSID 为 `5B2241C0-A61C-11D5-931D-0080C71A3226`。
Load `0x746f7020` 检查有符号 short 上限 1，随后读取一个
LONG，再于 `0x746f70fb` 和 `0x746f711e` 调用 AfCore 全局
Variant Read（IAT `0x747a32bc`）。Save `0x746f71e0` 同序写出。
端点复用[紧凑 VariantStream 布局](arcmap-10.1-variant-stream.md)；
非空对象端点的宿主行为见下段。构造旧版拓扑记录验证 LONG、两个
紧凑数值端点、后继字段边界及逐字节截断时的回滚。
真实 ArcMap 书签及 QGIS 人工验收待验证。

GPRangeDomain 的两个全局 Read 调用直接传入宿主 IStream。
AfCore 全局 Read `0x68a7b9a0` 对 VT_DISPATCH、VT_UNKNOWN
转发 LoadObject，使用宿主 ObjectStream 键表。Rust 共用现有
Variant 对象体分派。构造测试以 PropertySet 验证新键、第二端点
回引、未知类停止和逐字节截断回滚。真实工程中非空端点的类分布
及其在 ArcMap 中的语义待验证。

### GPWorkspaceDomain

同版 GpObjects.dll 的 RTTI `0x74801e84` 对应持久化虚表
`0x747b9c94`，类标识为 `0238381E-083F-4D29-933D-09B46ECCCF73`。
QI `0x7475a090` 在 `0x7475a0fb` 比较 IGPDomain IID，
匹配时返回 +0x10。Load `0x7475a7c0` 接受有符号 short 版本
<=1，然后在 `0x7475a80c` 读有符号 int32 计数。正计数逐项
由 `0x7475a839` 读取 LONG。Save `0x7475a880` 对称写出。

构造旧记录覆盖负计数、有符号列表值、后继字段边界以及截断和
超量计数回滚。列表值的业务含义、真实 ArcMap 书签和 QGIS
人工验收待验证。

### GPDatasetDomain

同版 GpObjects.dll 的 RTTI `0x747ff86c` 对应持久化虚表
`0x747ac768`，类标识为 `507CDA98-CEFD-4BFC-BF14-21A41B773E2D`。
QI `0x746ab560` 在 `0x746ab5cb` 比较 IGPDomain IID，匹配时
返回 +0x10。Load `0x746abec0` 检查有符号 short 上限 1，
在 `0x746abf0c` 读取有符号 int32 计数，正计数逐项读取 LONG。
随后无条件读取四个 short（`0x746abf6d / 0x746abf7f /
0x746abf91 / 0x746abfa3`）。Save `0x746abfd0` 同序写出。

构造测试覆盖正计数、负计数时仍读取四个 short、后继书签文本
及逐字节截断回滚。字段含义、真实书签及 ArcMap/QGIS 人工验收待验证。

### GPFeatureClassDomain

同版 GpObjects.dll 的 RTTI `0x747ffb38` 对应持久化虚表
`0x747adeb8`，CLSID 为 `652EC5FF-1772-49AB-B7C7-439143D44D56`。
QI `0x746c63f0` 在 `0x746c645b` 比较 IGPDomain IID，匹配时返回 +0x10。
Load `0x746c71e0` 检查有符号 short 上限 3，读取带有符号
计数的 LONG 列表和两个 short。v2 起在 `0x746c72d6` 至
`0x746c7314` 读取四个附加 short。v3 在 `0x746c7335` 起
读取第二个带有符号计数的 LONG 列表。Save `0x746c73a0`
对应 v3 布局。构造测试覆盖 v1–v3、后续书签字段和截断回滚。
字段业务含义、真实书签及 ArcMap/QGIS 人工验收待验证。

### GPArcInfoItemDomain

GpObjects.dll 10.1.0.3035 的 RTTI `0x747feeb4` 对应持久化虚表
`0x747a9618`，CLSID 为 `01C5FF8C-E01B-4A6D-A965-E7E30249E641`。
QI `0x74692c00` 在 `0x74692c39` 比较 IGPDomain IID，匹配时
返回 +0x10。Load `0x746935c0` 检查有符号 short 上限 1，
随后读取有符号计数与 LONG 列表。计数为非正时跳过列表，
仍在 `0x74693667` 读取一个 LONG，再在 `0x74693683` 至
`0x746936b9` 读取四个 short。Save 入口为 `0x746936e0`。

构造测试覆盖正负计数、尾部字段、后续书签文本和逐字节截断回滚。
字段含义、真实书签及 ArcMap/QGIS 人工验收待验证。

### GPFieldDomain

同版 GpObjects.dll 的 RTTI `0x7480011c` 对应持久化虚表
`0x747af028`，注册 CLSID 为 `D7FFAFF1-5990-4828-8263-AA93C1C0BD6A`。
Load `0x746d15b0` 使用有符号版本上限 4，先读有符号计数、
LONG 列表和两个 short。v2、v3 分别在 `0x746d1694`、
`0x746d1702` 起追加带有符号计数的 AfCore 成员 String 列表，
v4 在 `0x746d176c` 再读一个 short。Save 入口为 `0x746d17a0`。

构造测试覆盖 v1–v4、负计数字符串列表、后续书签字段和逐字节截断
回滚。字段业务含义、真实书签及 ArcMap/QGIS 人工验收待验证。

### GPMeasureFieldsDomain

GpObjects.dll 10.1.0.3035 的 RTTI `0x74804a08` 对应持久化虚表
`0x747bc378`，类标识为 `C06E2425-30D9-4C9D-8CD3-7FE243B3AFCB`。
QI `0x747792a0` 于 `0x7477930b` 比较 IGPDomain IID，匹配时
返回 +0x10。Load `0x74779a00` 只读取有符号 short，拒绝大于 1
的版本，并清除运行时脏标记。Save `0x74779a50` 只写版本。
构造测试覆盖负版本、超限版本、后继字段及逐字节截断回滚。
真实 ArcMap 书签和 QGIS 人工验收待验证。

### GPRouteIDFieldsDomain

同版 GpObjects.dll 的 RTTI `0x74804a2c` 对应独立 CLSID
`4A4F70B0-913C-4A82-A33F-E190FFA409EA` 和持久化虚表
`0x747bc278`。主 QI `0x74778990` 在 `0x747789fb`
比较 IGPDomain IID，匹配时返回 +0x10。该虚表与
GPMeasureFieldsDomain 共用 Load `0x74779a00`，
Save `0x74778fb0` 同样只写一个 short。

构造测试分别核对两种类的身份、负版本、超限版本、后继字段及
逐字节截断回滚。真实 ArcMap 书签和 QGIS 人工验收待验证。

### GPQueryTableIDsDomain 与 GPQueryTableFieldsDomain

GpObjects.dll 10.1.0.3035 的 PE RTTI 和持久化虚表对应关系如下。

| 类 | CLSID | RTTI | 持久化虚表 | Save |
| --- | --- | --- | --- | --- |
| GPQueryTableIDsDomain | `74F6B060-5EB6-4851-8FFD-8B188A845F37` | `0x74804a74` | `0x747bc0c8` | `0x74777eb0` |
| GPQueryTableFieldsDomain | `9CC43668-43D8-4182-9820-B2D620305BED` | `0x74804a98` | `0x747bbfc4` | `0x747772d0` |

两个虚表共用 Load `0x74779a00`，只读取有符号版本 short，
上限为 1。各自 Save 也只写一个 short。
构造测试核对各自的类身份、负版本、超限版本、后继字段边界
和逐字节截断回滚。真实 ArcMap 书签及 QGIS 人工验收待验证。

### GPVirtualTableDomain

同版 GpObjects.dll 的 RTTI `0x74804a50` 对应持久化虚表
`0x747bc1c8` 和 CLSID `BE355340-B5A9-4432-B2F4-497A50E05115`。
该虚表复用 Load `0x74779a00`，只读有符号 short，版本上限为 1。
Save `0x74778650` 在 `0x7477866d` 写出两个字节。构造测试
核对类身份、负版本、超限版本、后继书签字段和截断回滚。
真实 ArcMap 书签及 QGIS 人工验收待验证。

### GPRouteDomain 与 GPRouteMeasureEventDomain

同版 GpObjects.dll 中，两类的 IPersistStream 虚表共用 Load
`0x74779a00`，只读取有符号 short，版本上限为 1。

| 类 | CLSID | 持久化虚表 | Save |
| --- | --- | --- | --- |
| GPRouteDomain | `193FFF2F-C923-430B-879F-50BF65F61703` | `0x747bbe34` | `0x74775df0` |
| GPRouteMeasureEventDomain | `5B7D4CF9-95E0-4990-AD6E-D25EF89BAEA8` | `0x747bbefc` | `0x74776630` |

各 Save 只写两个字节。构造测试覆盖两类身份、负版本、超限版本、
后续书签字段及逐字节截断回滚。真实 ArcMap 书签和 QGIS 人工验收待验证。

### GPTablesDomain 与 GPLayersAndTablesDomain

GpCoreFunctions.dll 10.1.0.3035 的 SHA-256 为
`7048e718213b98fceae3f57267fcaa52d000287bcd94de42a5d32c27a0dee381`。
PE RTTI 和原生虚表对应如下。

| 类 | CLSID | RTTI | 持久化虚表 | Save |
| --- | --- | --- | --- | --- |
| GPTablesDomain | `5BDA0002-2574-492E-A397-8255FDFF2D58` | `0x7411074c` | `0x73feb6c0` | `0x7382b530` |
| GPLayersAndTablesDomain | `ECE7A40D-C09C-4DA1-BE5D-1A6D17FE6A82` | `0x74110608` | `0x73feaab0` | `0x7381e6e0` |

共用 Load `0x7381e690` 只读有符号 short，版本上限为 1。
两个 Save 均只写 short。构造测试覆盖两类身份、负版本、超限、
后继书签字段与逐字节截断回滚。真实书签及 ArcMap/QGIS 人工验收待验证。

### GPSAGeoDataDomain

GpRasterFunctions.dll 10.1.0.3035 的 SHA-256 为
`2922bd518dedfb15444de5a57fffb22f21c38432cea9b323e4ba931f12eed964`。
RTTI `0x748ba028` 对应持久化虚表 `0x748a9e78` 和 CLSID
`616F905F-756D-46C1-A5E9-A77076C8C740`。Load `0x74894920`
检查有符号 short 上限 1。首个有符号计数控制普通 IName 对象循环，
`0x748949c2` 请求 `0x748a614c` 的 IName IID。随后是两个各带
有符号计数的 LONG 列表，末尾依次为 short 和 AfCore String
（`0x74894b1d / 0x74894b2e`）。Save `0x74894b80` 对应同一顺序。

名称槽复用书签名称分派及现有嵌套上限。构造用例覆盖名称新对象与
回引、负计数、尾部字段及逐字节截断回滚。字段语义、真实书签及
ArcMap/QGIS 人工验收待验证。

### GPSANameDomain 与 GPSATextTableNameDomain

取证来自前述 GPSACore.dll 10.1.0.3035 的 PE RTTI 与 objdump。

| 类 | CLSID | 持久化虚表 | Load | Save |
| --- | --- | --- | --- | --- |
| GPSANameDomain | `C6A78F4E-A1F5-421A-AD8C-C4C9EC854A40` | `0x7492fa74` | `0x748e22f0` | `0x748e2380` |
| GPSATextTableNameDomain | `A51B925A-781E-4A3B-9920-B76FC2C432DA` | `0x749311a4` | `0x748f52b0` | `0x748f5310` |

两类检查有符号 short 上限 1。GPSANameDomain 随后在
`0x748e233d / 0x748e2353` 读取 LONG、short。
GPSATextTableNameDomain 在 `0x748f52f3` 读取 AfCore 成员 String。
构造测试覆盖两类独立身份、尾部字段、后继书签边界和截断回滚。
字段业务含义、真实 ArcMap 书签及 QGIS 人工验收待验证。

### GPSARemapDomain

同版 GPSACore.dll 的 RTTI `0x749469bc` 对应持久化虚表
`0x74930aa4`，CLSID 为 `ACA8516D-30D9-4002-8744-6241350E6F73`。
QI `0x748efb40` 在 `0x748efb79` 比较 IGPDomain IID，匹配时返回 +0x10。
Load `0x748eff50` 检查有符号 short 上限 1，于 `0x748effa8`
读取有符号 LONG 计数，正计数逐项在 `0x748effd9` 读取 LONG。
Save `0x748f0030` 同序写出。构造测试覆盖正负计数、数值、
后继书签边界及逐字节截断回滚。字段业务含义和真实 ArcMap/QGIS
人工验收待验证。

### GPSANeighborhoodDomain

GPSACore.dll 10.1.0.3035 的 RTTI `0x749467c4` 对应虚表
`0x7492ffb0` 与 CLSID `254C70FC-F9F5-4BAF-A039-0C5CD3733805`。
Load `0x748e72b0` 接受有符号版本 <=1，在 `0x748e7308`
读取有符号 LONG 计数，正计数在 `0x748e7339` 逐项读取 LONG。
Save `0x748e7380` 同序写出。构造测试验证正负计数、类身份、
后继书签字段及逐字节截断回滚。列表业务含义及 ArcMap/QGIS
人工验收待验证。

### GPRAFieldDomain

同版 GpRasterFunctions.dll 的 RTTI `0x748b98d0` 对应持久化虚表
`0x748a609c`，CLSID 为 `4B6CA858-5716-4AC3-A2EE-70EE2D29C1BD`。
Load `0x7486a200` 检查有符号 short 上限 1，先读带有符号计数的
LONG 列表，再在 `0x7486a2ab / 0x7486a2c1` 读取两个 short。
随后依次读取两组有符号计数和 AfCore 成员 String 列表，
字符串调用点为 `0x7486a30d / 0x7486a370`。Save 入口为
`0x7486a3e0`。构造测试覆盖负计数、后继字段和逐字节截断回滚。
字段含义及 ArcMap/QGIS 人工验收待验证。

### GPRepresentationLayerDomain

GPCartography.dll 10.1.0.3035 的 SHA-256 为
`325443ba4efaafb9b9e308f8a0aeda90fd2791fdf2353eb17619b8511897bbba`。
RTTI `0x7328236c` 对应持久化虚表 `0x7325e634` 和 CLSID
`821ACB85-5063-48A3-B2D5-3A95F1C8C73D`。Load `0x731f5860`
检查有符号版本上限 2，版本不大于 1 时只消费 short。
v2 在 `0x731f58b7` 读取有符号计数，正计数在 `0x731f58df`
逐项读取 LONG。Save `0x731f5920` 对应 v2 布局。
构造测试覆盖两个版本、正负计数、后续字段和截断回滚。
列表业务含义、真实书签及 ArcMap/QGIS 人工验收待验证。

### GP 注记域

GpObjects.dll 10.1.0.3035 的 PE RTTI 给出三类各自的持久化虚表。

| 类 | CLSID | RTTI | 持久化虚表 |
| --- | --- | --- | --- |
| GPGdbAnnoDomain | `07DB9832-C818-41D3-86D7-138F84B0A33F` | `0x747fee34` | `0x747a9038` |
| GPCovAnnoDomain | `B5C0DBCB-9953-4E01-A753-88C7EC07AF4F` | `0x747fedf8` | `0x747a9100` |
| GPCadAnnoDomain | `C78F0CD6-A4C2-4EAE-A1A9-BC805D135C62` | `0x747fedd8` | `0x747a91c8` |

三类共用 Load `0x74691130` 和 Save `0x74691180`。
Load 只消费一个有符号 short，`0x74691162` 的版本上限为 0。
Save 只写这个 short。构造测试覆盖类身份、负版本、拒绝 v1、
后续书签字段及逐字节截断回滚。真实书签及 ArcMap/QGIS 人工验收待验证。

### GP Coverage 域

同版 GpObjects.dll 的 PE RTTI 定位如下。

| 类 | CLSID | 持久化虚表 | Load | Save |
| --- | --- | --- | --- | --- |
| GPCoverageDomain | `56781190-D49B-11D5-9341-0080C71A3226` | `0x747ac09c` | `0x746a5fd0` | `0x746a6090` |
| GPCoverageFeatureClassDomain | `3FE7B260-CE52-11D5-933A-0080C71A3226` | `0x747ac238` | `0x746a6db0` | `0x746a6ea0` |

前者检查有符号 short 上限 1，在 `0x746a6018` 调用 LoadObject，
请求 `197512A0-CD81-11D5-9339-0080C71A3226`
（类型库中的 IGPCoverageFeatureClassDomain）。后者检查版本上限 2，
读取带有符号计数的 LONG 列表，v2 在 `0x746a6e58 / 0x746a6e6e`
追加 short、LONG。构造测试覆盖直接与嵌套子域、两个版本、
后继字段边界和截断回滚。字段语义及 ArcMap/QGIS 人工验收待查。

### DataElementDomain 与 GPCompositeDomain

同版 GpObjects.dll 的 PE RTTI 分别关联持久化虚表
`0x747a7824` 和 `0x747ab1f8`。类标识为
`B6F39D28-C6F6-45B4-A60C-2601A11BE4BD` 和
`F9AE1125-48EF-412D-AB8C-AD917F3D5F8E`。
两类 Load（`0x7467a150`、`0x746a1490`）都检查有符号
short 上限 1，后读有符号 LONG 计数，非正数跳过列表。
DataElementDomain 在 `0x7467a1d6` 读取 AfCore 成员 String。
GPCompositeDomain 在 `0x746a151f` 以 IGPDomain IID 调用 LoadObject。
对应 Save 入口为 `0x7467a220`、`0x746a1560`。

组合域复用域分派和书签嵌套预算。构造测试覆盖数据元素子域、
回引、后续书签字段及逐字节截断回滚。字段含义、真实书签和
ArcMap/QGIS 人工验收待验证。

### GPFieldInfoDomain 与 GPCodedValueDomain

GpObjects.dll 10.1.0.3035 的持久化虚表、Load 和 Save 分别为
GPFieldInfoDomain 的 `0x747af810 / 0x746d6e00 / 0x746d6ec0`，
以及 GPCodedValueDomain 的 `0x747aac88 / 0x7469b660 / 0x7469b900`。
两类检查有符号 short 上限 1。前者在 `0x746d6e48` 请求
IGPCodedValueDomain（`41DC44A0-D2EB-11D5-933F-0080C71A3226`）。
后者先读取有符号计数与 AfCore String 标签列表，再读有符号值计数，
在 `0x7469b751` 逐项请求 IGPValue。两个计数独立。

GPString Load `0x74703c90` 和 GPLong Load `0x74708ef0`
各读无符号版本（上限 1），再调用 DECoreLib 同版 GPValue::Load
`0x6e955770` 消费两个 short（基类无符号版本上限 1），最后分别读取
AfCore String 和 LONG。合成测试覆盖直接及内嵌编码值域、
两个值类、回引、后继字段和逐字节截断回滚。其余 IGPValue 类、
字段语义及 ArcMap/QGIS 人工验收待验证。

对照 72 个同版 Desktop 类型库，33 个公开 IGPDomain 类均已有分派。
另从 10.1 Desktop 互操作程序集提取直接列出 IGPDomain 的 COM 类
GUID，逐项与 Rust 新对象分派对比。两边各有 33 个 CLSID，集合完全一致，
且新对象与回引均支持这些类。此范围闭合了旧记录的公开域类型分派。
编码值域中的其余 IGPValue 实现及范围域中其余未确认的 Variant 对象类
仍是各自的子槽缺口。

#### GPDouble、GPBoolean、GPDate

同版 GpObjects.dll 的持久化虚表和原生读写入口如下。

| 类 | CLSID | 虚表 | Load | Save |
| --- | --- | --- | --- | --- |
| GPDouble | `374C4574-5D6E-4D77-9A67-5489C6BBE932` | `0x747b5900` | `0x74705930` | `0x747059b0` |
| GPBoolean | `66E9D559-5E0E-45B4-ADA8-D7C207BA6B49` | `0x747b530c` | `0x74702c70` | `0x74702cf0` |
| GPDate | `BED6F22B-AD8E-440E-8DD5-B416396B6FD4` | `0x747b56e8` | `0x74704c30` | `0x74704cb0` |

三类均检查无符号版本上限 1，再调用 GPValue::Load。随后
分别读取 8 字节 double、2 字节 short，以及 AfCore
Variant::Read(false)（`0x74704c82`，导入槽 `0x747a32e8`）。
GPDate 复用带 14 字节保留区的 Variant 读取器。构造测试覆盖
负零位模式、布尔 short、VT_DATE、后续书签字段和截断回滚。
真实书签及 ArcMap/QGIS 人工验收待验证。

#### GPLinearUnit 与 GPVariant

GpObjects.dll 10.1.0.3035 的 PE RTTI、持久化虚表和读写入口如下。

| 类 | CLSID | RTTI | 持久化虚表 | Load | Save |
| --- | --- | --- | --- | --- | --- |
| GPLinearUnit | `842DE5C7-80FB-4C41-8621-69DEAD3CC024` | `0x748008c0` | `0x747b1a94` | `0x746ea8b0` | `0x746ea940` |
| GPVariant | `5B82FF5E-3115-40E4-BAC8-E62105CA2811` | `0x74801e2c` | `0x747b9a00` | `0x74758980` | `0x74758a00` |

构造路径分别从 `0x747b1b34` 和 `0x747b9ae0` 复制 CLSID。
QI 在 `0x746ea07b` 和 `0x74758173` 使用编码域请求的同一
IGPValue IID。Load 均先检查无符号 short 版本上限 1，再调用
GPValue::Load。GPLinearUnit 在 `0x746ea905` 和 `0x746ea91b`
按顺序读 double 和有符号 LONG。GPVariant 在 `0x747589d2`
调用与 GPDate 相同的 AfCore Variant::Read(false)，复用带 14 字节
保留区的 Variant 读取器。

构造书签覆盖线性单位数值与有符号单位码、VT_I4、GPVariant 回引、
后续字段和逐字节截断回滚。单位码映射、真实书签及 ArcMap/QGIS
人工验收待验证。

#### GPArealUnit

同版 GpObjects.dll 的 RTTI `0x747fef48` 对应持久化虚表
`0x747a9ca8`。构造函数在 `0x74694aa5` 和 `0x74694ab2`
从 `0x747a9dac` 复制 CLSID `62C28A96-B84E-49F8-ACF5-0B1B5A87EF08`。
QI 在 `0x74694bf9` 使用编码域请求的 IGPValue IID。
GPArealUnit 复用 GPLinearUnit 的 Load `0x746ea8b0`，先读版本和
GPValue 基类，再读 double 和有符号 LONG。独立 Save `0x74695450`
在 `0x746954a4` 和 `0x746954ba` 按相同顺序写值。

构造书签核对面积数值、原始单位码、对象回引、后续字段边界和
逐字节截断回滚。单位码映射、真实书签及 ArcMap/QGIS 人工验收待验证。

#### GPEncryptedString

GpObjects.dll 10.1.0.3035 的 RTTI `0x747ff9c8` 对应持久化虚表
`0x747ad728`。构造函数从 `0x747ad7f4` 复制类标识
`B76CD4DB-843B-4723-B174-A0833095F866`，QI 在 `0x746bf1cb`
使用编码域请求的 IGPValue IID。Load `0x746bf650` 以无符号 short
检查类版本上限 2，调用 GPValue::Load 后，在 `0x746bf6a0` 调用
AfCore 成员 String::Read。Save `0x746bf6d0` 的版本分支最后均
调用成员 String::Write。解析器保留磁盘中的字符串内容。

构造书签覆盖 v2、对象回引、后续字段、超限版本和逐字节截断回滚。
真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPRandomNumberGenerator

GpObjects.dll 10.1.0.3035 的 RTTI `0x74800e24` 对应持久化虚表
`0x747b3c6c`，构造函数从 `0x747b3d38` 复制类标识
`A5C6674B-BCA6-46C8-8205-B5B958EEAEB9`。QI 在 `0x746f4c5b`
使用编码域请求的 IGPValue IID。Load `0x746f5970` 检查无符号
short 版本上限 1，读取 GPValue 基类后在 `0x746f59eb` 和
`0x746f59f6` 依次读取种子和生成器选择码两个 LONG。原生将
0、1、2 之外的选择码回退为 1，再用两个参数初始化内嵌生成器。
Save `0x746f5a40` 按相同顺序写两个 LONG。解析器保留原始选择码。

构造书签验证两个有符号字段、未知选择码、对象回引、后续字段和
逐字节截断回滚。真实书签及 ArcMap/QGIS 人工验收待验证。

#### GPMultiValue

GpObjects.dll 10.1.0.3035 的 RTTI `0x74800aa8` 对应持久化虚表
`0x747b2294`。构造函数从 `0x747b241c` 复制类标识
`6A34BC94-06B2-481B-B104-6663C4A344EF`。QI 在 `0x746ede9b`
接受 IGPValue。Load `0x746ef3b0` 检查无符号 short 版本上限 1，
读取 GPValue 基类后在 `0x746ef429` 读取类型名称对象（请求 IName IID
`4BA33331-D55F-11D1-8882-0000F877762D`）。类型对象为空时跳过
其虚调用，随后在 `0x746ef480` 读取有符号成员计数，仅在计数
为正时于 `0x746ef4ac` 逐项读取 IGPValue。Save 入口是
`0x746ef520`。

空类型槽和递归成员复用编码域的值读取器和对象表。构造书签覆盖
跨列表回引、负计数、后续字段及逐字节截断回滚。真实书签和 ArcMap/QGIS 人工验收待验证。

非空名称在 `0x746ef45f` 调用 IName 虚表 +0x14（Open）。
Save 在 `0x746ef5ac` 调用内嵌数据类型接口的 +0x18 取名称，
并在 `0x746ef5bb` 保存该对象。GpObjects 对应数据类型虚表的
+0x18 导入 DECoreLib::GPDataType::get_FullName，后者返回 IName。
解析复用已有 GP 数据类型名称分派并保留对象回引。构造书签
覆盖非空 GPDataTypeName、嵌套名称回引和逐字节截断回滚。
其余名称类和 Open 的运行结果待验证。

#### GPValueTable

GpObjects.dll 10.1.0.3035 的 RTTI `0x74801cd4` 对应持久化虚表
`0x747b9334`，构造函数从 `0x747b9580` 复制类标识
`98A7A0CD-3282-44CD-90B7-4F39CDB85B84`。QI 在 `0x747554c9`
匹配 IGPValue。虚表 Load/Save 跳板 `0x7478b69c / 0x7478b6a2`
转到 GeoprocessingLib.dll 10.1.0.3035 的 GPValueTableBase
`0x71de7e50 / 0x71de8090`。该 DLL 的 SHA-256 为
`8626273d18ff3f166f26e6aba61a4dfcdc9ad633206b9004ba05437f3352b3d6`。

Load 检查无符号 short 版本上限 1，调用 GPValue::Load 后在
`0x71de7ebf` 读取有符号列数，在 `0x71de7eee` 为每列读取 IName。
随后于 `0x71de7f7f` 读取有符号行数，按行、列顺序于
`0x71de7fc0` 读取每个 IGPValue 对象。非正计数跳过相应循环。
解析共用引用表，并检查行列乘积和最小对象头所需字节。

构造书签覆盖两行两列、列名和单元格引用、空槽、超限计数、
后续字段及逐字节截断回滚。尚待验证真实书签、其他列类型名称及
ArcMap/QGIS 人工验收。

#### GPValueTableTypeName

GpObjects.dll 10.1.0.3035 的 RTTI `0x74801d60` 对应持久化虚表
`0x747b96c4`。GetClassID `0x74755730` 从 `0x747b9778` 复制
`65521A5F-7DA6-4893-AE58-F70FD3CD1DEE`。QI 分别在
`0x747559f7 / 0x74755a29 / 0x74755a5b` 匹配 IName、IGPName
和 IGPDataTypeName。Load `0x74756190` 检查有符号版本上限 3，
随后读取有符号列数，非正计数跳过列循环。每列按以下顺序读取。

1. `0x7475621e` 读取 IGPDataTypeName。
2. 版本小于 2 时，`0x74756245` 读取 IGPDomain，IID 地址与
   GPCompositeDomain 的域槽相同。
3. `0x7475626e / 0x74756285` 读取成员 String 和有符号 LONG。
4. 版本大于 2 时，`0x747562b5` 读取 IUID。

构造书签覆盖 v1–v3、旧域、UID、后续字段及逐字节截断回滚。
LONG 的业务含义、真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPPoint

在 10.1 Desktop 互操作程序集的 IGPValue 元数据中定位候选，
再以 GpObjects.dll 10.1.0.3035 的 PE RTTI 和 objdump 复核。
RTTI `0x74801504` 对应持久化虚表 `0x747b6184`，构造函数
从 `0x747b5dd8` 复制 CLSID `A1E93082-BABE-4D44-8CE3-4637748EDDDD`。
Load `0x74709d50` 用无符号 short 上限 1 检查版本，调用
DECoreLib::GPValue::Load 后，在 `0x74709db7 / 0x74709dc9` 通过
AfCore 的 double 重载依次读取两个坐标。Save `0x74709df0` 在
`0x74709e55 / 0x74709e67` 对称写入。解析保留两个浮点位模式。

构造书签覆盖坐标、回引、后继字段、超限版本及逐字节截断回滚。
真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPSQLExpression 与 GPINFOExpression

10.1 Desktop 互操作程序集的两类 CLSID 与 GpObjects.dll
10.1.0.3035 的构造路径一致。原始指令和虚表复核如下。

| 类 | CLSID | RTTI / 持久化虚表 | Save |
| --- | --- | --- | --- |
| GPSQLExpression | `F966FB74-EA71-41C7-A848-8D26A71DB91A` | `0x748016ec / 0x747b6940` | `0x7470f3d0` |
| GPINFOExpression | `32A265A6-78D2-49A5-A65B-572973A7B6A0` | `0x74800724 / 0x747b10f8` | `0x746e7a80` |

QI 分别在 `0x7470ed7e` 和 `0x746e75fe` 引用编码域请求的 IGPValue IID。
共同 Load `0x7470f350` 检查无符号 short 版本上限 1，读取 GPValue
基类，然后于 `0x7470f3a0` 读取 AfCore 成员 String。两类 Save
均按此顺序写入。解析保留表达式文本及其具体类型。

构造书签覆盖两个类的字符串、回引、后继字段、超限版本和逐字节截断回滚。
真实书签和 ArcMap/QGIS 人工验收待验证。

#### GP 空间参考值

10.1 Desktop 互操作程序集将下列五类列为 IGPValue 实现。
用 GpObjects.dll 10.1.0.3035 的 RTTI、虚表和 objdump 复核
持久化入口。类标识也在同版 PE 的构造数据中核对。

| 类 | CLSID | 持久化虚表 | Load / Save |
| --- | --- | --- | --- |
| GPSpatialReference | `503307EA-4131-456B-9B9A-15989534BE4D` | `0x747b6668` | `0x7470e1d0 / 0x7470e270` |
| GPCoordinateSystem | `B3E1D7ED-A8D6-44A7-A911-817C361B574C` | `0x747abeac` | `0x746a50c0 / 0x746a5150` |
| GPXYDomain | `9776287B-3D47-4DB7-801C-D679D7AA6872` | `0x747b9e6c` | `0x746ebdb0 / 0x7475c240` |
| GPZDomain | `44F4FE86-8870-4375-A9C2-96AEBB9DE3BC` | `0x747ba01c` | `0x746ebdb0 / 0x7475d230` |
| GPMDomain | `84259B9E-73C9-462A-BEB1-3BB3051E593A` | `0x747b1cd4` | `0x746ebdb0 / 0x746ebe40` |

Load 首先读取无符号 short 版本，GPSpatialReference 上限为 3，
其余上限为 1。接着读 GPValue 基类，在
`0x7470e22e / 0x746a511d / 0x746ebe0d` 请求同一个
ISpatialReference IID `4434D680-F4DA-11D1-BCA6-0000F875BCCE`。
GPSpatialReference 仅在版本恰为 2 时于 `0x7470e24c` 读一个
short 至局部栈变量。Save 写 v3，省略此 short。解析复用已有
ISpatialReference 对象槽，并保留 v2 的原始值。

构造书签覆盖五个类、空空间参考、GPSpatialReference v1–v3、
值对象回引、后继字段、超限版本和逐字节截断回滚。真实书签
及 ArcMap/QGIS 人工验收待验证。

#### GPEnvelope 与 GPExtent

在 ArcMap 10.1 Desktop 互操作程序集定位两类，并用 GpObjects.dll
10.1.0.3035 的 PE RTTI、虚表及 objdump 复核流边界。

| 类 | CLSID | RTTI / 持久化虚表 | Load / Save |
| --- | --- | --- | --- |
| GPEnvelope | `01A9676B-E251-4A40-8544-5E5B65E2983C` | `0x748013bc / 0x747b5b38` | `0x74706c00 / 0x74706c90` |
| GPExtent | `754F9CDC-EE51-412A-97EA-E573077C559F` | `0x747ffad0 / 0x747adc80` | `0x746c54c0 / 0x746c5570` |

两类 Load 都检查无符号 short 上限 1，调用 GPValue 基类，
再于 `0x74706c5d / 0x746c553b` 请求同一个 IEnvelope
IID `1B077914-38C5-11D0-92D2-00805F7C28B0`。GPExtent 在
该对象槽前于 `0x746c5517` 额外读取一个 LONG，Save 在
`0x746c55ca` 对称写出。解析保留原始模式值，共用 AOIBookmark
包络槽的对象边界和回引类校验。

构造书签覆盖非空 Envelope、空槽、有符号模式值、值对象回引、
后继字段、超限版本和逐字节截断回滚。模式枚举、真实书签及
ArcMap/QGIS 人工验收待验证。

#### GPCellSizeXY

GpObjects.dll 10.1.0.3035 的 PE RTTI `0x747ff310` 对应持久化
虚表 `0x747aa9c4`。PE 类标识常量 `0x747aaa64` 与 Desktop
互操作程序集的 `4EB9896A-27B8-4CC2-A0C2-55E40D2E928C` 相符。
该类复用 GPPoint 的 Load `0x74709d50`，按序读取类版本、
GPValue 基类及两个 double。objdump 复核独立 Save `0x74699ca0`
在 `0x74699d05 / 0x74699d17` 写入两个 double。

构造书签覆盖两轴像元大小、对象回引、后继字段、超限版本和
逐字节截断回滚。真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPCalculatorExpression

GpObjects.dll 10.1.0.3035 的 PE RTTI `0x747ff218` 对应持久化
虚表 `0x747aa6f0`。PE 类标识常量 `0x747aa7e0` 与 Desktop
互操作程序集的 `952C38A0-4CEF-488A-8C99-7D19B6F74F91` 相符。
Load `0x74698b60` 以无符号 short 检查类版本上限 1，调用
GPValue::Load 后，于 `0x74698bb0` 读取 AfCore 成员 String。
Save `0x74698be0` 在 `0x74698c2f` 对称写入该字符串。
解析保留表达式文本。

构造书签覆盖文本、对象回引、后继字段、超限版本和逐字节截断
回滚。真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPRouteMeasureEventProperties

GpObjects.dll 10.1.0.3035 的 PE RTTI `0x74804b04` 对应持久化
虚表 `0x747bbd4c`。类标识常量 `0x747bba68` 与 Desktop
互操作程序集的 `35527F95-BB45-4BCE-A813-EA1369E735BC` 相符。
QI 在 `0x7477468a` 引用编码域请求的 IGPValue IID。
Load `0x74774d50` 检查无符号 short 版本上限 1，调用 GPValue
基类后，于 `0x74774dae / 0x74774de2 / 0x74774df9 / 0x74774e10`
连续调用 AfCore String::Read。Save `0x74774e90` 对称写四个字符串。
保留流中字段顺序及原值。

构造书签覆盖字符串顺序、回引、后继字段、超限版本和逐字节截断
回滚。四字段的业务名、真实书签以及 ArcMap/QGIS 人工验收待验证。

#### 其余 IGPValue 候选与 GPLine、GPPolygon 的对象边界

从本地 ArcMap 10.1 Desktop 互操作程序集的 .NET 元数据枚举 COM 类，
筛选直接列出 IGPValue 接口的类型并以 CLSID 去重。得到 138 个候选，
其中 25 个已在编码值槽实现，113 个待进一步核对。后者包括 47 个类名
以 DE 开头的候选。接口清单只用于定位实现。判断能否进入编码值域还需
核对同版原生类的 QueryInterface、持久化入口以及宿主写侧。

同版 GpObjects.dll 的两个优先候选如下。虚表和读写顺序由 PE RTTI
与 objdump 核对。

| 类 | CLSID | RTTI / 持久化虚表 | Load / Save |
| --- | --- | --- | --- |
| GPLine | `435F80E4-6DB2-476B-B977-B06BFF52F978` | `0x74801434 / 0x747b5d4c` | `0x7470ae60 / 0x74707f90` |
| GPPolygon | `EC1B8F0B-DF38-4444-BE03-0116E09537DF` | `0x7480156c / 0x747b635c` | `0x7470ae60 / 0x7470af60` |

共享 Load 以无符号 short 上限 1 检查版本，在 `0x7470aea7`
先读 GPValue 基类，`0x7470aeba` 再读有符号 LONG 计数，
`0x7470aefb` 循环读取 IID
`A3CD267B-86A1-4ED6-A8DD-5AE09B84F00E` 的点对象。
两个 Save 都先写版本，再写计数和点对象，最后调用 GPValue::Save
（`0x7470800f / 0x7470afdf`）。同版 DECoreLib 的 GPValue::Save
在 `0x6e9557ff / 0x6e955823` 写入两个 short。两种字段顺序使用
相同的类和版本，且普通 ObjectStream 无对象长度。
当前保留这两类的停止边界，等待真实档案或运行取证确认
编码值域实际使用的字段顺序。尚未在 ArcMap/QGIS 人工验证。

#### GPSACellSize

GPSACore.dll 10.1.0.3035 的 SHA-256 为
`0aa35b1be22c61742f36a651679e3028c363c40eebe859cda285fa6c6c2a5292`。
RTTI `0x749461d4` 对应持久化虚表 `0x7492d69c`，类标识常量
`0x7492d71c` 与 10.1 Desktop 互操作程序集声明的
`0882F404-551D-4DAC-9842-3DC2F27F5B53` 一致。用 objdump 核对，
Load `0x748d6a70` 检查无符号 short 版本上限 1，读取 GPValue
基类后在 `0x748d6aca / 0x748d6afa / 0x748d6b0c` 依次读取
LONG、double、double。Save `0x748d6b40` 按相同顺序写入。
解析保留模式值和两个 double 的原始位模式。

构造书签覆盖字段、对象回引、超限版本、后继字符串和逐字节截断回滚。
模式枚举、真实书签和 ArcMap/QGIS 人工验收待验证。

#### 空间分析值的旧 IName 槽

对照 ArcMap 10.1 Desktop 互操作程序集的 CLSID，用同版
GPSACore.dll 的 PE RTTI、虚表和 objdump 查证三种 IGPValue。

| 类 | CLSID | 持久化虚表 | Load / Save |
| --- | --- | --- | --- |
| GPSARadius | `2B3FC07F-103B-41C4-AA42-34A243FD7B87` | `0x749305b0` | `0x748ec1c0 / 0x748ec2e0` |
| GPEvaluationScale | `C81671B5-B23C-4AAE-BFA8-B8A66FC173B7` | `0x7492cdac` | `0x748d2130 / 0x748d2260` |
| GPSAMapAlgebraExp | `ACEABE44-324C-455B-B635-C8764FDB0029` | `0x7492f8f8` | `0x748e18b0 / 0x748e19d0` |

三个 Load 均用无符号 short 拒绝大于 2 的版本，随后读 GPValue
基类。只有版本恰为 1 时分别在 `0x748ec240 / 0x748d21b0 /
0x748e192e` 请求 `0x7492ce50` 的 IName IID
`4BA33331-D55F-11D1-8882-0000F877762D` 并释放返回对象。
该槽复用书签名称的共享键表和嵌套预算。

随后 GPSARadius 在 `0x748ec274 / 0x748ec28c / 0x748ec29e` 读
两个 LONG 和一个 double。GPEvaluationScale 在 `0x748d21e4 /
0x748d21f6 / 0x748d2208` 连读三个 LONG。GPSAMapAlgebraExp 在
`0x748e1965` 调自由函数 AfCore ReadString，Save 在 `0x748e1a29`
调配对 WriteString，采用 UTF-16 码元数量前缀。三个 Save 均省略
旧名称对象槽。构造测试涵盖 v1 的空名称槽、v2、省略与空槽的区分、
共享值回引、后继字段和逐字节截断回滚。其余名称实现、数值属性的
具体含义与 ArcMap/QGIS 实档验收待验证。

#### GPSAFuzzyFunction

GPSACore.dll 10.1.0.3035 的 RTTI `0x74946474` 对应持久化虚表
`0x7492f278`，类标识常量 `0x7492f440` 与 Desktop 互操作程序集的
`F31DFC96-01E0-4F01-962B-064D48D5C246` 一致。
Load `0x748dda10` 检查无符号 short 版本上限 1，读 GPValue
基类后在 `0x748dda82 / 0x748ddaa0 / 0x748ddabe` 请求
`0x7492ce70` 的 IGPValue IID `92EE08B6-C757-4178-803C-67CD665AC801`。
随后在 `0x748ddaea / 0x748ddb0a` 各读一个字节，转换为 VARIANT_BOOL。
Save `0x748ddb50` 的版本表为 `[-1,-1,-1,-1,1]`，写出时顺序一致。
解析使用共享对象键、值分派和嵌套预算，保留两个标志的原始字节。

构造书签涵盖新值、回引、空值、非规范非零标志、后继字段、
超限版本和逐字节截断回滚。三个参数与两个开关的业务名、
真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPSANumberRemap 与 GPSAStringRemap

GPSACore.dll 10.1.0.3035 的 RTTI 和持久化虚表按 PE 字节定位，
使用 objdump 核对调用顺序和导入函数，类标识对照 Desktop 互操作程序集。

| 类 | CLSID | RTTI / 持久化虚表 | Load / Save |
| --- | --- | --- | --- |
| GPSANumberRemap | `3156C3FF-80CE-498F-B16F-F3A81A45908F` | `0x74946830 / 0x7493024c` | `0x748e8cb0 / 0x748e8fb0` |
| GPSAStringRemap | `D0532089-344D-4D23-A67C-5DFCD14D6694` | `0x74946ab0 / 0x74930ff8` | `0x748f3750 / 0x748f3aa0` |

两者接受无符号 short 版本上限 4，顺序读取 GPValue 基类、仅
v1 的 IName、一个 LONG、有符号 LONG 行数。v3 起在行循环之前
增加 short 和 LONG。数值行在 `0x748e8e02 / 0x748e8e19 /
0x748e8e30` 读取 double、double、LONG，再于 `0x748e8e47 /
0x748e8e65` 读取 DWORD 字节数和相应窄字节文本。
字符串行在 `0x748f38b2 / 0x748f38d0 / 0x748f390c /
0x748f3929 / 0x748f3944` 读取带 DWORD 字节数的窄字节文本、
LONG、另一段带 DWORD 字节数的窄字节文本。两类 Save 都以
DWORD 计数写出文本字节。原生读取后在临时缓冲区追加终止零。
解析原样保留文本字节，避免按未知代码页转码，并检查行数、
文本长度与输入范围。

构造书签覆盖 v1–v4、两类重分类行、非 UTF-8 文本字节、空文本、
值回引、后继字段、超限版本和逐字节截断回滚。文本代码页、
LONG 业务含义、真实书签和 ArcMap/QGIS 人工验收待验证。

#### GPSANeighborhood

同版 GPSACore.dll 的 RTTI `0x74946718` 对应持久化虚表
`0x7492fd10`，CLSID `6FA435F7-9046-482B-86CC-089E1682EA3F`
由 PE 字节和 10.1 Desktop 互操作程序集交叉核对。Load `0x748e58e0`
接受无符号 short 上限 2，读取 GPValue 基类、仅 v1 的 IName，
再于 `0x748e59a6 / 0x748e59c0 / 0x748e59da` 读三个 LONG。
按第二个 LONG 减 1 索引 `0x748e5ce8` 跳表，越界直接进入收尾。

| 形状选择码 | 对象体后继数据 |
| --- | --- |
| 1、2、3、4 | 分别是 2、1、2、3 个 double |
| 5、6 且第一个 LONG 为 1 | 两个 LONG、AfCore Variant::Read(false)，调用 `0x748e5bfe` |
| 5、6 的其余情况 | 一个 AfCore 成员 String，调用 `0x748e5c77` |
| 其余值 | 无后继字段 |

Save `0x748e5d00` 的尺寸分派、内嵌 Variant 和成员 String
路径按 objdump 与上述读侧对位。解析保留原始选择码和 double 位模式，
沿用 Variant 与名称槽的共享对象键，不打开字符串所指向的文件。

构造书签覆盖 v1/v2、各选择码、内嵌 Variant、路径、未知选择码、
对象回引、后继字段及逐字节截断回滚。内嵌数组的真实取值与
ArcMap/QGIS 实档验收待查。

### GPSAHorizontalFactor 与 GPSAVerticalFactor

两类来自 ArcMap 10.1.0.3035 的 `GPSACore.dll`。沿 RTTI 确认持久化接口
`+0x1c` 后，水平 Load/Save 为 `0x748e00f0/0x748e01b0`，垂直为
`0x748fead0/0x748feb90`。两者读版本 0 或 1、共有 GPValue 基类体及
一个 LoadObject 槽，写侧固定写版本 1。对象槽分别请求 IID
`{6FD6F6E0-EFB0-42BC-8BA2-843FFDFE77CE}` 与
`{42AA082B-2FC2-4939-AD2E-92F563E4356F}`。同模块的构造引用和 10.1
安装介质的类注册清单将槽的常规实现对应到 PathDistanceHorizontalFactor
`{9FDAFBBC-A491-43D7-9D41-D288BBA568E1}` 与 PathDistanceVerticalFactor
`{1D4EFECC-B85C-4EB9-B09B-CEF8C225B5A3}`。

内层证据为 `RasterDistanceOp.dll` 10.1.0.3035（SHA-256
`338c4149bcc91d2b503544270c72e80aaf89c7bec54b95a8a28154d455eccaae`）。
持久化接口虚表 `0x7c1d9070/0x7c1d9850` 的 Load/Save 分别为
`0x7c1cf6c0/0x7c1cfb00`、`0x7c1d30d0/0x7c1d3ba0`。GetClassID 写入
的常量位于 `0x7c1d8cac/0x7c1d8cbc`。两类接受版本 0 或 1，均先读
u16 版本和 LONG 选择子。按 Load 跳转表 `0x7c1cfae0/0x7c1d3b78` 与
AfCore 的读写调用复核其余段：

| 内层类 | 选择子 | 顺序消费 |
| --- | --- | --- |
| 水平 | 1、2 | 2 个 double |
| 水平 | 3、4 | 3 个 double |
| 水平 | 5 | `String::Read` 路径 |
| 垂直 | 1、7、8 | 3 个 double |
| 垂直 | 2、3、5、6、9、10 | 4 个 double |
| 垂直 | 4 | `String::Read` 路径 |
| 两类 | 其他值 | 无后继字段 |

解析保留选择子、原始 double 位模式和路径，支持空对象、同类回引及完整对象体。
遇到不同对象类无法确定边界，沿用外层 FeatureBookmark 的事务回退。
合成工程覆盖各选择子、版本和截断。路径读出后 ArcMap 尝试加载外部因子表，
本解析器仅保留路径文本。实际工具参数及 ArcMap 工程读写待实档验证。

### GPSASemiVariogram

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946a20` 对应持久化
虚表 `0x74930d60`，CLSID `{7A732377-B6B8-41AB-AD2D-C2051E82A494}`
位于 `0x74930e3c`，与安装介质类注册清单一致。用 objdump 检查 Load
`0x748f2030` 和 Save `0x748f2290`。Load 读版本 0 至 3、GPValue
基类体、仅 v1 存在的 IName 槽（IID 常量 `0x7492ce50`）及 LONG
变异函数类型。v0 至 v2 的 Lag 以 LONG 字节数加原始窄字符存储，
读取后将文本 `#` 视作缺省标记。v3 的 Lag 改为 double。其后依次读取
Range、PartialSill、Nugget 三个 double，Save 固定写 v3。

字段名与安装介质 `esriSpatialAnalyst.olb` 的 IGPSASemiVariogram
类型库条目 88 对照。Save 经 `+0x14`、`+0x28`、`+0x18`、
`+0x20`、`+0x1c` 取值，分别对应 VariogramType、Lag、Range、
PartialSill、Nugget。读取器保留原始枚举和 double 位模式，旧 Lag
保留窄字符字节。合成测试覆盖 0 至 3 版、回引和逐字节截断回滚。
旧值的实际代码页及 ArcMap 工程内对象待实档验证。

### GPRasterCalculatorExpression

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946094` 对应持久化
虚表 `0x7492d1fc`，CLSID `{F745D0A1-22A2-4AEE-893E-01EAB8603BB8}`
在 `0x7492d364`。用 objdump 核对 Load `0x748d4670`、Save
`0x748d47e0`。Load 接受版本 0 或 1，读 GPValue 基类、AfCore
`String::Read` 表达式、带符号的 LONG 参数数量。数量大于零时，每行
经 `0x748d4727` 读取一个请求 IID
`{92EE08B6-C757-4178-803C-67CD665AC801}` 的 IGPValue。
Save 固定写 1，按同一顺序写表达式、数量和对象槽。

分派复用编码值域的 IGPValue 列表、登记表和嵌套限制。合成书签覆盖
表达式、负数量、对象及回引、空槽和截断后的事务回滚。
实际 ArcMap 工具参数工程待验证。
