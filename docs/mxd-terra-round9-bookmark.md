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

Geocoding.dll 10.1.0.3035 的 `0x71367bcf` 先读取 IUnknown，
接口转换完成后由 `0x71367d93` 返回成功，转换不再读流。
Rust 在该入口复用 SystemCore PropertySet 持久化读取器。
构造输入验证 v2 的对象结束位置及引用键，v1 的 IName 新对象
路径遇同一类时回滚。尚无该组合的实档或人工验证。

SystemCore 10.1.0.3035 在 `0x7ffd0063` 按键定位原实例，再查询
宿主请求的接口。构造回引以 TableName 类标识指向已登记的
PropertySet。v2 保留登记类并读完书签，v1 退回书签起点并保留
此前登记的键。该组合尚无实档或人工验证。

同一 IUnknown 入口也复用 [UID 对象体读取器](arcmap-10.1-featurelayer-tail.md)。
构造 v2 书签验证 UID 字段、后继书签文本及 UID 截断时的回滚。
v1 的 IName 槽拒绝已登记的 UID 回引。尚无该组合的实档或人工验证。

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
构造书签还覆盖类型槽内的 GPMultiValueTypeName v1。外层名称
读取内嵌 GPDataTypeName，并由后续 GPMultiValue 和 GPValueTable
通过同一对象键回引。同一用例还验证 GPValueTableTypeName v3
空列定义和 GPSATopoFeaturesTypeName v4 短体，覆盖两个对象之后的
成员计数及同键回引。这些组合基于上文分别核对的 10.1 Load 布局。
构造测试分别从该槽进入当前四种已支持的数据类型名称，并读取
后续成员和回引。同版 GpObjects.dll 的 `0x746ef463` 会传播 Open 失败。
Open 成功后，`0x746ef472` 调用 `0x746a09a0 → 0x746a09d0`，
在 `0x746a0a11` 向返回对象查询 IID
`172A93A9-AFC1-432D-8F04-54B7309BA432`。查询失败时赋空成员，
调用者随后直接在 `0x746ef480` 读取成员计数。解析器按 IName
读取已确认的名称体，构造 GPFunctionName v2 验证后继成员和回引。
该用例只覆盖字节读取，ArcMap 中的 Open 结果和其他候选名称仍待验证。

GpObjects.dll 10.1.0.3035 的多值对象 QI（`0x746ede30`）及其
基类分派（`0x74786fe0`）均无 IName 分支。类型槽按登记类拒绝回引
GPMultiValue。构造输入以 GPDataTypeName 类标识回引宿主多值键，
验证书签游标和引用表整体回滚。尚无原生运行或实档验证。

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

### GPSAExtractValues

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946378` 对应持久化
虚表 `0x7492ed60`，CLSID `{C079D387-7473-41D7-BA10-AC9A92F81A0C}`
位于 `0x7492ef90`。用 objdump 核对 Load `0x748da550` 与 Save
`0x748da720`。Load 接受版本 0 或 1，读 GPValue 基类体和四字节有符号
行数。行数大于零时，每行在 `0x748da5fd/0x748da618` 顺序读取两个
请求 IGPValue IID 的对象。Save 依据 `0x7492cb98` 的版本映射，生成对象
体时写版本 1，其后仍是基类、行数、两列对象。

读取器保持行边界和原始行数。每行至少需要两个空对象头的总长度，
实际对象和回引共用宿主登记表。合成书签覆盖非正行数、两列配对、
跨行回引和截断回滚。真实 ArcMap 工具参数记录待验证。

### GPSATimeConfiguration

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946bb0` 对应持久化
虚表 `0x749313a4`，CLSID `{24733FD0-2CDE-4446-B65E-8F239D77DE7E}`
位于 `0x749314bc`。Load `0x748f6ca0` 读取版本 0 或 1、GPValue
基类体和请求 IID `{85811371-0D46-4593-BB58-649B2E58E2EC}` 的对象槽。
同模块的构造类标识和安装介质类注册清单将其对位到 TimeConfiguration
`{0AA31980-3D44-439A-A8BF-C7D4C9D8F30B}`。

内层来自 `RasterSolarOp.dll` 10.1.0.3035（SHA-256
`ec8a6b3569fbc555d3a48ee5d25957574ee926fd2c863a9a193fefe80cd0ea31`），
RTTI `0x7ca3261c` 对应持久化虚表 `0x7ca2bc10`。用 objdump 核对
Load `0x7ca23c00` 和 Save `0x7ca23d40`。Load 接受版本 0 或 1，
读取 LONG 选择子。值 2 后接一个 LONG 和两个 double，值 3 后接
三个 LONG，值 4 后接一个 LONG。其余选择子在该版本无后继字段。
写侧采用相同分支和顺序。GetClassID 在 `0x7ca23bb3` 取类标识常量
`0x7ca2a8c0`，与外层构造匹配。

解析保留选择子、各整数和 double 的原始位模式。合成书签验证三类
参数、默认分支及截断回滚。选择子的具体日照参数语义和真实工程仍待验证。

### GPSAWeightedSum

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946f28` 对应持久化
虚表 `0x74932ed8`，CLSID `{1F1DB699-D05C-4C50-B951-B7969E0397E3}`
位于 `0x74933038`。用 objdump 核对 Load `0x7490d070`、Save
`0x7490d2a0`。Load 接受版本 0 或 1，先读 GPValue 基类体和有符号
四字节行数，正数时每行在 `0x7490d12c/0x7490d14c/0x7490d171` 连读
三个 IGPValue 对象槽。Save 在 `0x7490d3a8/0x7490d426/0x7490d47f`
按相同顺序写三列。

读取器同 GPSAExtractValues 一样保留原始行数和行边界，各列共用宿主
登记表。合成书签覆盖负行数、三列记录、第三列的跨行回引及逐字节截断回滚。
实际 ArcMap 参数工程中的各列对象类型待实档验证。

### GPSARemap

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946968` 对应持久化
虚表 `0x74930820`。安装介质注册为
`{F402D0A9-F032-476A-BCBB-D48CF041C24E}`。用 objdump 核对
Load `0x748ee2d0`、Save `0x748ee750`。Load 接受版本 0 至 5，
依次读取 GPValue 基类、仅 v1 的 IName、LONG 模式、LONG 行数。
v3 及以后在行列表之前增加 short 和 LONG。模式值 1 走数值范围行，
其余走窄字符来源值行，两类都包含 LONG 目标值和带字节数的窄字符说明。
v4 及以后在列表之后再读一个 short 和两个 LONG，读取点为
`0x748ee6ca/0x748ee6f5/0x748ee70e`。Save 对应的尾段从 `0x748eebc0`
开始。

现有两个固定模式类继续按各自类标识确定行布局。合成书签覆盖通用类
的六个版本、两类行、后继字符串和截断回滚。旧文本的实际代码页及
通用对象在加权叠加表中的真实工程记录待验证。

### GPSAWeightedOverlayTable

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946e80` 对应持久化
虚表 `0x74932a34`，CLSID `{96017C94-F471-42A7-86EF-633B40650F8C}`
位于 `0x74932c30`。用 objdump 核对 Load `0x74902bf0`、Save
`0x74903160`。Load 接受版本 0 至 2，先读 GPValue 基类，再读下面两种
结构。Save 写 v2，键与其他同流对象共用 ObjectStream。

| 版本 | 行计数前 | 每行的三个 IGPValue 槽之后 |
| --- | --- | --- |
| 1 | IName、两个 double、一个 LONG | 带符号 LONG 重分类行数，以及每行的窄字符来源值、LONG 目标值、窄字符说明 |
| 0、2 | IGPEvaluationScale 对象槽 | IGPSARemap 对象槽 |

v1 数值读取点为 `0x74902d0d/0x74902d20/0x74902d33`，旧行数据由
`0x74902f04` 起的循环读取。普通 IGPValue 三列的读取点为
`0x74902e80/0x74902ea7/0x74902ece`。新版槽分别请求
`{B465C093-81F8-4DEA-A133-F2231D1D1DBC}` 和
`{B60AF0D2-4C41-4B41-BB28-C37728BA1FBB}`。根据同版
`esriSpatialAnalyst.olb` 条目 78、81 的接口和 coclass 关联，以及
GPSACore 的类型实现，比例尺度槽接入 GPEvaluationScale，重分类槽复用
GPSARemap、GPSANumberRemap、GPSAStringRemap。

读取器保留旧行原字节、表行边界和回引，沿用 FeatureBookmark 的事务回退。
合成测试覆盖三种版本、旧行空计数、新对象、跨行回引和逐字节截断。
实际 ArcMap 加权叠加表中的三列对象及旧 IName 的打开结果待实档验证。

### GPSATopoFeatures

ArcMap 10.1.0.3035 的 `GPSACore.dll` RTTI `0x74946c50` 对应持久化
虚表 `0x749316f8`，CLSID `{8371765F-0F0B-4868-B786-015B2E2C1C10}`
位于 `0x74931830`。用 objdump 将 Load/Save 的跳板
`0x749215b4/0x749215ba` 对到导入表 `0x7492c248/0x7492c244`，
目标是 GeoprocessingLib.dll 的 GPValueTableBase
`0x71de7e50/0x71de8090`，与 [GPValueTable](#gpvaluetable) 共用流布局。

读取器沿用列 IName、行列计数和 IGPValue 单元格解析。合成书签
验证列名和单元格的回引、后继字段及逐字节截断回滚。
拓扑工具生成的真实工程和其他列 IName 的类覆盖仍待验证。

### 栅格环境的压缩、金字塔与分块大小

证据来自 ArcMap 10.1.0.3035 的 `GpRasterFunctions.dll`（SHA-256
`2922bd518dedfb15444de5a57fffb22f21c38432cea9b323e4ba931f12eed964`）。
用 RTTI 对位完整对象的 `+0x1c` 持久化接口，按 objdump 和导入表核对
Load/Save。类标识与安装介质的注册清单及 PE 内 GUID 一致。

| 对象 | RTTI／持久化虚表 | Load／Save | Load 版本与参数 |
| --- | --- | --- | --- |
| GPRasterGDBEnvCompression | `0x748b9b28 / 0x748a7de0` | `0x74873930 / 0x74873a30` | v0–v2，两个 LONG |
| GPRasterGDBEnvPyramid | `0x748b9bd8 / 0x748a812c` | `0x748752c0 / 0x74875460` | v0–v4，short 和两个 LONG；v3 起再加两个 LONG，v4 再加 short |
| GPRasterGDBEnvTileSize | `0x748b9d98 / 0x748a8698` | `0x74877c50 / 0x74877d50` | v0–v2，两个 LONG |

三类均先读自身版本和 GPValue 基类。仅 v1 在表中参数之前读取一个
IName 槽，IID 常量位于 `0x748a614c`。复用现有名称分派，参数保留原始位模式。
合成书签测试覆盖各版本、回引、后继数据及截断回滚。枚举语义以及真实
ArcMap 书签中的参数值待验证。

### GPRasterGDBEnvStatistics

同版 `GpRasterFunctions.dll` 的 RTTI `0x748b9cf0` 对应持久化虚表
`0x748a844c`，CLSID `{94F81626-002E-4C51-8829-A04C6CB5E8EA}` 位于
`0x748a84e4`。Load `0x748769d0` 接受版本 0 至 3，读 GPValue 基类后
仅 v2 消费 IName。接着读一个 short、两个 LONG 和版本化忽略值。
同版 `esriGeoprocessing.olb` 中的 IGPRasterGDBEnvStatistics 声明依次为
CalculateStatistics、XSkipFactor、YSkipFactor、IgnoreValue。

v0/v1 的 `0x74876ae4` 将固定长度 4 传给 AfCore
`AfStreamRead(IStream*, wchar_t*, …)`。同版 AfCore 入口 `0x68a84830`
原样将长度传给 IStream::Read。这四字节读入随后释放的临时槽，
解析器保留原字节。v2/v3 的 `0x74876ab8` 改用自由函数 ReadString，
保存侧 `0x74876b70` 固定写 v3 和 BSTR。合成测试覆盖四版、旧字段后继
边界、BSTR 结束符和截断回滚。四字节的历史用途与实档仍待验证。

### GPAnalysisCellSize

同版 `GpRasterFunctions.dll` 的 RTTI `0x748b8134` 对应持久化虚表
`0x748a4cf4`，CLSID `{D042DFF8-EAA2-4706-B3DE-6E62CFEDA76A}` 位于
`0x748a4e5c`。Load `0x748621c0` 接受 0 或 1，读取 GPValue 基类，
在 `0x74862223` 读取 double，随后 `0x74862235` 调 AfCore 成员函数
String::Read。Save `0x748622b0` 写固定版本 1，然后写基类、double 和
成员 String。合成书签覆盖两种版本、数值位模式、空和非空文本、回引和截断。
文本后续用途及真实 ArcMap 参数工程仍待验证。

### GPArcInfoItem

证据来自 ArcMap 10.1.0.3035 的 `FileDataElements.dll`（SHA-256
`5c4703596f243daf7025cc1d0e435cf6522dbcb4a02b77529d0694aef4ef5f3c`）。
RTTI `0x6f9efeb4` 对应持久化虚表 `0x6f9dd350`，CLSID
`{6DDDBBB2-41DC-4F12-913E-803196EB33FA}` 位于 `0x6f9d86a4`。
用 objdump 对位 Load `0x6f9c90c0` 和 Save `0x6f9c9200`。
Load 接受 v0/v1，GPValue 基类后依次读两个 AfCore 成员 String、
五个 LONG、三个 short。Save 对称写入。读取器按流序保留标量。
合成书签覆盖两种版本、非空名称、后继字段、回引、超限版本和逐字节
截断回滚。成员语义及原生书签待实档验证。

### GPDataFile 和 GDSData

ArcMap 10.1.0.3035 的 `GpObjects.dll` RTTI `0x747ff7f4` 对应持久化
虚表 `0x747ac59c`，CLSID `{763035E4-E5C9-4AAF-A60C-65C38ADDB5EF}`
位于 `0x747ac65c`。Load `0x746aac00` 接受版本 0 或 1，先读 GPValue，
再于 `0x746aac69` 读取请求 IID
`{BDFD716D-E902-4907-9C97-65DA73F02AE9}` 的对象，最后连续读取三个
AfCore 成员 String。Save `0x746aacc0` 顺序相同。同版
`esriGeoDatabaseDistributed.olb` 条目 85 将该 IID 定名为 IGDSData。

内层证据来自 `GeoDataServer.dll` 10.1.0.3035（SHA-256
`2c89c12ffc1097e2fe4ebf769683ca5940f40ba1933f4edf18aebbd25742bc17`）。
GDSData 的持久化虚表 `0x71718994` 经 RTTI `0x717240c0` 对位，
CLSID `{8B77E69C-2A95-42E3-88E3-22A612B69752}` 位于 `0x71718a70`。
Load `0x716f1a20` 读取有符号 short 版本（上限 3），随后顺序读取
LONG 长度、该长度的原字节、成员 String、LONG、short。v2 增加
IPropertySet 槽（`0x716f1b34`），v3 追加成员 String
（`0x716f1b4c`）。Save 入口 `0x716f1b70` 按文档版本选择布局。
用 objdump 和 PE 导入表核对读写调用。

读取器保留原字节，限制长度，且复用 PropertySet 及共享引用表。
合成书签覆盖 v0 至 v3、有符号版本比较、非空数据和属性对象、
父对象后继字符串及逐字节截断回滚。负数据长度作为无效输入拒绝。
嵌入数据的内部编码和真实 ArcMap 工程待查。

### GPWorkspaceExtension

ArcMap 10.1.0.3035 的 `FeatureDataElements.dll`（SHA-256
`7cd1153712d8af23f6c2263b6444c1a77e16b818fb5fb946d0c3d9106be5f429`）
RTTI `0x6f680da0` 对应持久化虚表 `0x6f663a80`，CLSID
`{8B233DBE-0591-459B-BFD1-468D2DB5267D}` 位于 `0x6f663b00`。
用 objdump 核对 Load `0x6f63a730`、Save `0x6f63a7b0`。
Load 接受 v0/v1，读 GPValue 基类，随后于 `0x6f63a780` 调用
AfCore String::Read，再于 `0x6f63a793` 读 16 字节 GUID。
Save 以相同顺序写出这两个字段。读取器保留名称及 GUID 原字节。
合成书签覆盖两种版本、后继字段和截断回滚。真实 ArcMap 参数工程待验证。

### GPFieldInfo

ArcMap 10.1.0.3035 的 `GpObjects.dll` RTTI `0x748001b8` 对应持久化
虚表 `0x747af44c`，类标识为 `{2ECE5CD7-8567-44C7-A6A3-C86FAF2B5550}`。
依据安装介质类注册清单、objdump 和 PE 导入表核对 Load `0x746d4f30`、
Save `0x746d51f0`。Load 接受 v0 至 v2，读 GPValue 基类和有符号 LONG
计数。正数时每行依次读 LONG、两个自由函数 ReadString 的 BSTR、LONG、
short。v2 每行随后追加 BSTR、LONG、BSTR、两个 LONG，读取点为
`0x746d50aa/0x746d50c4/0x746d50e0/0x746d50fa/0x746d5114`。
Save 根据文档版本选择是否写行扩展。

解析保留各行及原始标量，依据最小行宽检查计数。合成书签覆盖三种
版本、空及非空文本、多行、非正计数、后继数据和截断回滚。
实际 ArcMap GP 参数工程与各标量业务语义待验证。

## LocatorWorkspaceName 回引的接口检查

SystemCore 10.1.0.3035 的 `0x7ffd0063` 在登记实例上查询宿主请求
的接口。通用名称读取器据此使用登记类检查 IName 回引，对已确认
缺少该接口的 PropertySet、UID 和 GPMultiValue 返回错误。
LocatorWorkspaceName 的 IUnknown 入口按登记键保留回引。

构造测试以 TableName 类标识指向这三个已登记类，确认 FeatureBookmark
退回起点并保留引用表。其余实现仍需核对接口，尚无该组合的实档或
ArcMap 10.1 人工验证。

### NAClassFieldMap

NetworkAnalyst.dll PE32 10.1.0.3035 的 SHA-256 为
`8c4ebaee71aad63c25616aa806674018479829d97bc01c7ed3b8845813698532`。
RTTI 主表 `0x7af9de24` 的 QI `0x7af05f30` 比较 IGPValue
（IID 常量 `0x7af9dffc`）及 IPersistStream。持久化表
`0x7af9dea8` 的 GetClassID `0x7af07510` 返回
`{E61C9FF8-06BF-42E5-AD66-849364B48549}`。Load
`0x7af07570` 读有符号版本（上限 1），在 `0x7af07604`
调用 DECoreLib GPValue::Load，再读有符号行数。非正值跳过循环。
每行 `0x7af07671`、`0x7af07690` 调用 AfCore ReadString
读取 BSTR，`0x7af076a9` 调 Variant::Read(false)，全程沿用
宿主对象流。Save `0x7af077e0` 写同样的字段序。

构造书签覆盖非空行、Variant、同键回引、负版本、负计数和截断回滚。
尚无该类在实际编码值域中的实档或 ArcMap/QGIS 人工验收。

### GPNAHierarchySettings

GPNAFunctions.dll PE32 10.1.0.3035 的 SHA-256 为
`3c6f3dc1d5bfd9d848035bbd5b9c599859330759f29b26b2654838e91ce9178c`。
安装介质文件以 objdump 复核。RTTI 对应的 QI `0x74555ff0`
在 `0x74556029` 比较 IGPValue IID。持久化表 `0x745e1d38`
的 GetClassID `0x74556230` 返回
`{07F27D22-A56C-419B-9931-DD38C60897F1}`。

Load `0x74556320` 读取无符号版本，拒绝 0，随后调用
GPValue::Load（导入槽 `0x745dc29c`），再读取两个有符号
32 位值，分别写入持久化接口指针的 `+0x34` 和 `+0x38`。
Save `0x74556290` 写版本 0，再调用 GPValue::Save 并写回两个
值。解析按 Load 的接受条件处理，保留原始标量。构造书签覆盖
版本 1 和 65535、拒绝版本 0、同键回引和截断回滚。
实际编码值域宿主、标量语义及 ArcMap/QGIS 人工验收仍待确认。

### GP3DADecimate 与 GP3DAInterpolate

GpTinFunctions.dll PE32 10.1.0.3035 的 SHA-256 为
`c474482ba93c5d3a5baa110a7341df950bff2804fbd880f7895ce931737283cd`。
安装介质文件以 objdump 复核。RTTI 将持久化表 `0x74ef9344`
和 `0x74ef97c4` 分别对应到 GP3DADecimate 与 GP3DAInterpolate。
两表的 GetClassID 通过 `0x74edbf98` 调 DECoreLib GPValue::GetClassID
（`0x6e955720`），从接口指针 `+8` 返回构造函数写入的 GUID。
两类持久化接口位于完整对象 `+0x1c`，构造函数在
`0x74d41498` 和 `0x74d42f36` 分别向完整对象 `+0x24` 写入
`{1C8379B1-158A-49C2-979C-E948E07BA303}` 与
`{41850EFE-51DE-43A4-8147-B88CCA19CF3C}`。QI 共用的 IGPValue
IID 常量位于 `0x74ef93e4`。

Load `0x74d41fb0` 和 `0x74d44240` 都接受无符号版本 0、1，
然后调用 GPValue::Load。前者再依次读 long、double、long，
后者再读六个 long 和一个 double。导入槽 `0x74ef8050` 和
`0x74ef8054` 分别为 AfStreamRead 的 long 与 double 重载。
Save `0x74d42080` 和 `0x74d44360` 按相同字段顺序写出。
构造书签覆盖两个读取版本、拒绝超限版本、同键回引和截断回滚。
字段语义、实际编码值域宿主及 ArcMap/QGIS 人工验收仍待确认。

### GPFieldInfoEx

FeatureDataElements.dll PE32 10.1.0.3035 的 SHA-256 为
`7cd1153712d8af23f6c2263b6444c1a77e16b818fb5fb946d0c3d9106be5f429`。
安装介质文件以 objdump 复核。QI 在 `0x6f627897` 比较 IGPValue IID
（常量 `0x6f658604`）。RTTI 对应的持久化表 `0x6f661888` 的
GetClassID `0x6f628440` 返回 `{4BC6F3FB-4666-425D-BBB5-F09BE77A4E5E}`。

Load `0x6f6284a0` 接受无符号版本 0 和 1，在 `0x6f6284e2`
调用 GPValue::Load，再在 `0x6f6284f0` 调用 AfCore 的 String::Read。
Save `0x6f628510` 写版本 1，按相同顺序保存基类和字符串。
构造书签覆盖两个可读版本、超限版本、同键回引和截断回滚。
实际编码值域宿主及 ArcMap/QGIS 人工验收仍待确认。

### GPAddressLocatorStyle

Geocoding.dll PE32 10.1.0.3035 的 SHA-256 为
`e5dc7d6cf515b7c7e9fd9e0342780818d261c2762ebbd348c5c2fc318c71819c`。
安装介质文件以 objdump 复核。RTTI 对应持久化表 `0x714fbf94`
位于对象 `+0x1c`。构造函数 `0x71377b1b` 向对象 `+0x24`
写入 `{C0B10685-71F8-4CD2-B33A-8C9190B8E807}`。GetClassID
经 `0x714cd5a2` 调 GPValue::GetClassID，复制接口指针 `+8` 的 GUID。

Load `0x71379020` 接受无符号版本 0 至 4，调用 GPValue::Load，
随后调用两次 String::Read 并读一个 short。该 short 为零时，
`0x713790c8` 直接跳到成功返回，没有版本尾段。构造书签覆盖
五个版本的空配置、超限版本、回引和截断回滚。

非零时，Load 依次读取三个 IID `{6B2072E3-23F7-11D1-89D8-006097AFF44E}`
的接口槽、一个有符号行数及按版本读取的行（`0x7137b2e0`）。
版本 1 再读一个字节，版本 2 至 4 再读 IPropertySet，版本 3 至 4
再读一个 short。最后一个 short 非零时读取 Variant::Read(false)
和 IID `{D7FCBB04-8E59-4FFB-9EE5-EC670AA892E0}` 的接口槽。
解析器已支持三个前导槽为空或为已知 Fields 的配置，保留版本 1 的字节、
已知 PropertySet 槽及版本 3、4 的尾段标志和 Variant。Variant 后的
接口槽支持空标记。

参考数据行 Load `0x7137b2e0` 读 String，版本 4 再读 short，
接着读第一个 long。版本 2 至 4 按第一个 long 的正值循环加载
IID `{1714D59B-FB22-11D1-94A2-080009EEBECB}` 对象
（`0x7137b403`）。所有版本随后读第二个 long 及接口槽
（`0x7137b4ce`）。按第二个 long 读取值时，版本 0、1 每值
包含字符串计数及 String 列表（`0x7137b550`、`0x7137b59a`），
版本 2 至 4 每值为 Variant::Read(false)（`0x7137b50e`）。
末尾读取另一接口槽（`0x7137b690`）。构造书签覆盖两类值、
版本 4 的附加 short、宿主回引及截断。esriSystem 10.1 类型库将该列表的 IID 对应到 IUID，SystemCore 的 UID
对象体见 [原生读写记录](mxd-boundary-luna.md)。解析沿用同一对象表，
构造书签覆盖列表中的 UID 新对象和同键回引。其余未核对的非空接口
对象仍返回明确错误，实际宿主及人工验收仍待确认。

从 10.1 esriGeoDatabase.olb 的 MSFT 类型库资源按本地类型定义解析，
`{6B2072E3-23F7-11D1-89D8-006097AFF44E}` 对应 IFields，
`{D7FCBB04-8E59-4FFB-9EE5-EC670AA892E0}` 对应 IDETable。
配置的三个 IFields 槽与参考数据行的 IFields 槽接入了同一个
[GdbCore Fields 读取器](mxd-10.1-fields.md)。构造书签覆盖从行槽回引到
配置槽内的新 Fields；其他 IFields 实现仍待核对，IDETable 槽的读取范围见下节。

### DETable

10.1 esriGeoDatabase 类型库将 DETable 对应到
`{9C8405DB-BB87-4BD6-932D-7FFB35B04BA0}`。FeatureDataElements.dll
10.1.0.3035 的 DETable 构造函数将持久化虚表 `0x6f65eb84` 装到
对象 `+0x1c`，从 `0x6f65b9ac` 复制该 CLSID 到对象 `+0x24`。
该表的 Load 和 Save 分别经 `0x6f64a4ce`、`0x6f64a4d4` 转发到
DECoreLib.dll 10.1.0.3035。后者的 SHA-256 为
`a57d35741decebe0765e8942268a934990763f52813b55eadf126de7d3042c2f`。
以下偏移相对于持久化接口指针，以 objdump 复核。

| Load | 读取顺序 |
| --- | --- |
| DataElement `0x6e942b50` | 无符号版本（上限 2）、GPValue、三个 String、三个 short。v2 且第三个 short 为 `0xffff` 时先调 `0x6e943ac0`。随后读有符号计数，逐项按 IDataElement IID 调 LoadObject。 |
| DEDataset `0x6e9453b0` | 无符号版本（上限 2）、DataElement、long、两个 short、long，v2 再读一个 String。 |
| DETableBase `0x6e94e540` | 无符号版本（上限 4）、DEDataset、short、一个 String。内部字节 `+0xa4` 非零时再读五个 String、long、IStringArray、IPropertySet、有符号计数及 IGPSubtype 槽、有符号计数及 IGPControllerMembership 槽。该分支在 v2+ 再读 short 和两个 String，v3+ 再读 short 和四个 String，v4 再读 short。所有路径末尾读取 IFields 和 IIndexes。 |

DETable 构造函数在 `0x6f61daa2` 将对象 `+0xc0`（相当于持久化
接口 `+0xa4`）置 1。Load 在 `0x6e94e5cf` 直接测试内存中的该字节，
此前未从流读取它。Save `0x6e94e890` 在 `0x6e94e929` 也测试
该内存字节。对标准 DETable 构造路径，读取器应按非零路径取字段，
无需在流中读额外标志。请求的子槽接口由各 LoadObject 的 IID 和
10.1 esriGeoDatabase、esriSystem 类型库核对。已在两个 IDETable 槽接入标准 DETable 对象体及 StrArray、PropertySet、
Fields。构造测试覆盖版本 0 至 4、嵌套 Fields 回引及截断回滚。
DataElement 的其他递归子元素，以及
IGPSubtype、IGPControllerMembership 的合法实现边界仍需核对。

#### Indexes 与 Index

GdbCore.dll 10.1.0.3035（SHA-256
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`）
的 RTTI 将持久化表 `0x70ad70d8`、`0x70ad6c80` 分别对应到
IndexSet 和 Index。GetClassID `0x709f2600`、`0x709efc90`
返回 `{03859813-4DA5-11D1-8824-0000F877762D}` 和
`{826E2701-4DA6-11D1-8824-0000F877762D}`，与类型库的
Indexes、Index coclass 对应。

Indexes 的 Load `0x709f2b20` 读无符号版本（上限 1）和有符号计数，
按正计数加载 IIndex（IID 常量 `0x70ab9e0c`）。Index 的 Load
`0x709f04d0` 同样接受版本 0、1，接着调用 GPValue::Load，
读取 String、三个单字节字段和 IFields 槽。Save 入口分别为
`0x709f2c30` 和 `0x709f05a0`。构造书签覆盖 DETable 到 Indexes、
Index 的连续读取和 Index 回引先前 Fields 的边界，并覆盖截断回滚。
其他接口实现和实际 MXD 宿主仍待核对。

#### GPSubtype

FeatureDataElements.dll 10.1.0.3035 的 GPSubtype 持久化表
`0x6f663534` 的 GetClassID `0x6f637350` 返回
`{741B5302-2917-4AAB-952A-DF5F0E019EB6}`，与类型库一致。
Load `0x6f637390` 接受无符号版本 0、1，读取 String、long，
随后委派 `0x6f6376e0` 读取有符号行数及逐行两个 String、
Variant::Read(true)（`0x6f6377a1`）。紧随 Variant 的辅助调用
`0x6f638500` 创建内存对象，无流参数。Save `0x6f637420`
对应写入 String、long 和字段行。构造书签覆盖非空子类型和字段行的
紧凑 Variant，实际 MXD 宿主及人工验收仍待确认。

#### 控制器成员

FeatureDataElements.dll 10.1.0.3035 的持久化表
`0x6f663778` 和 `0x6f661aa4` 分别对应 GPTopologyMembership
和 GPGeometricNetworkMembership。GetClassID `0x6f638ac0` 和
`0x6f628bd0` 返回 `{5DA78FF6-546B-4AF0-84B8-50B68484D9D5}`
和 `{7EE891D6-6ACD-4687-9BF4-FCE49FC47157}`。Load
`0x6f638b20` 和 `0x6f629040` 均接受无符号版本 0、1。
拓扑成员读取 String、double、两个 long、short，几何网络成员读取
三个 String 和 long。读取器保留原始标量及浮点位模式。构造书签覆盖
同一 DETable 的两类非空成员和截断回滚。其他实现类及实际宿主仍待核对。

#### DataElement 子对象

DataElement 的正子对象计数经 `0x6e942c94` 调 LoadObject，
请求 IDataElement，并沿用传入的流。读取器已接入该列表中的 DETable，
按同一引用表读取新对象和回引。新对象受书签嵌套上限约束，
回引只消耗对象头。构造书签覆盖内嵌 DETable、同键回引及截断回滚。
其余 IDataElement 实现及真实宿主仍待核对。

#### DataElement 旧兼容 XML

DataElement v2 的第三个 short 为 `0xffff` 时，Load `0x6e942c54`
转入 `0x6e943ac0`。辅助函数先创建内存对象，随后在 `0x6e943b3e`
调用 AfCore ReadString(BSTR*, IStream*)，并在 `0x6e943b92`
把结果传给内存对象的方法。接口 IID
`{01700025-EF62-47E3-BD1E-7A47D0696541}` 对应类型库的 IXmlPropertySet2。Save 侧 `0x6e942d9f` 调
`0x6e943c30`，后者在 `0x6e943cc2` 调 WriteString(BSTR, IStream*)。
因此此分支在子对象计数前额外消耗一个 BSTR。构造书签覆盖非空
UTF-16 XML、随后递归对象和截断回滚。实际宿主仍待确认。

#### DEFeatureClass

FeatureDataElements.dll 10.1.0.3035 的持久化表 `0x6f658858`
对应 DEFeatureClass，构造函数从 `0x6f658eb0` 复制类 GUID
`{00BB7F40-3E51-4864-A66C-7F0C8EE9A842}`。Load 入口
`0x6f64a28e` 转发到 DECoreLib 的 `0x6e949720`。它接受版本
0、1，调用 DETableBase::Load，随后读两个 long、一个 String、
三个 short、IEnvelope 与 ISpatialReference 槽。标准构造函数
在 `0x6f5f31a2` 启用扩展标志，因此尾部还读两个 String。
构造书签覆盖子对象的新对象、同键回引和截断回滚，实际 MXD 宿主仍待验证。

#### DEFeatureDataset

FeatureDataElements.dll 10.1.0.3035 的 DEFeatureDataset 持久化表
`0x6f659284` 将 Load 经 `0x6f64a462` 转发到 DECoreLib
`0x6e94b280`。类型库标明类 GUID
`{6AC619FE-191F-432D-B1A3-28CE85A591AF}`。该入口读取无符号版本
（上限 1），然后调用 DEGeoDataset::Load `0x6e94b860`。后者读取
自身版本（上限 1），调用 DEDataset::Load，再加载 IEnvelope
和 ISpatialReference。结构化书签覆盖两个版本字段、空范围与空间
参考槽、后续字段定位及截断回滚。实际 MXD 宿主仍待验证。

#### DETopology

FeatureDataElements.dll 10.1.0.3035 的 DETopology 持久化表
`0x6f65f1dc` 对应类型库类 `{ACC4637D-D5E0-47D5-ABCD-9F25AB326E6F}`。
Load `0x6f61f6f0` 接受无符号版本 0 至 2，调用 DEGeoDataset::Load
后读两个 double、一个 long 和 IStringArray。v2 从 `0x6f61f7d2`
再读有符号规则计数，按正值在 `0x6f61f803` 加载
IID `{A53B59A5-2EFC-41C4-9769-C0FAA13E62E9}` 对象。
构造书签验证作为 IDataElement 子对象的字段边界及截断回滚。
实际 MXD 宿主仍待核对。

#### TopologyRule

esriGeoDatabase 10.1 类型库确认 DETopology 请求 ITopologyRule。
GdbTopo.dll 10.1.0.3035（SHA-256
`cbaa77c4de34be8d8ff4f95cfc833709d2ac0db54c68075aa917a717c8de1c42`）
的 TopologyRule 持久化表 `0x7129cff8` 中 GetClassID `0x71284160`
返回 `{8AD43889-F2A4-43B2-83DC-6D1D38DDBCC8}`。Load
`0x712841c0` 接受无符号版本 0、1，调用 GdbCoreLib Rule::Load
`0x70d36110` 后读取两个 String、四个 long、五个 short 和一个 long。
Rule::Load 接受版本 0、1，读一个 String 和两个 long。
构造书签覆盖非空规则列表、新对象、同键回引及截断回滚。
其他可能的接口实现和实际 MXD 宿主仍待核对。

#### DEGeometricNetwork

FeatureDataElements.dll 10.1.0.3035 的持久化表 `0x6f65acb8`
由构造函数在 `0x6f5f8415` 写入，RTTI 对应 DEGeometricNetwork，
类型库类 GUID 为 `{FE68BA8C-935D-4F6F-80D5-7957F8DBEFDF}`。
Load `0x6f5f9c10` 接受无符号版本 0 至 6，先调用
DEGeoDataset::Load。下表为累积版本门。

| 版本 | 新增字段，按流顺序 |
| --- | --- |
| 2 | long、String、IStringArray 槽（LoadObject `0x6f5f9cb9`）。 |
| 3 | 三组有符号计数和对象列表。按顺序请求 IConnectivityRule（`0x6f5f9d13`）、INetWeight（`0x6f5f9d7c`）、INetWeightAssociation（`0x6f5f9de0`）。每组非正计数跳过循环。 |
| 4 | short、两个 double。 |
| 5 | 两个 long。 |
| 6 | short。 |

三个列表的 IID 分别为 `{F366A3FA-5362-11D2-AB1B-000000000000}`、
`{027E7415-981B-11D1-B264-0000F878229E}`、
`{19BDC490-6201-11D1-B255-0000F878229E}`，已与安装介质
esriGeoDatabase.olb 的本地接口定义对位。主体已接入，构造书签覆盖版本 2 至 6 的字段门及截断回滚。尚需核对各列表的具体持久化
对象体，并接入解析。实际 MXD 宿主尚未验证。

#### NetWeight 与 NetWeightAssociation

GdbNetwork.dll 10.1.0.3035 安装介质 SHA-256 为
`36b1fcd7ea5951aea6c53e03855993a2fcbb2b78c3e8b068642a890c44a0995c`。
RTTI 持久化表 `0x7118ca94`、`0x7118cc74` 的 GetClassID
分别返回 `{3EBBE031-557D-11D1-B254-0000F878229E}` 和
`{19BDC491-6201-11D1-B255-0000F878229E}`。
Load `0x70f9d230` 和 `0x70f9dee0` 均接受无符号版本 0、1，
读取 long 后以 AfCore ReadString 取 BSTR。NetWeight 的 BSTR 后
是两个 long，NetWeightAssociation 则再读一个 BSTR。构造书签
覆盖两个非空列表及其后的版本字段和截断回滚。连接规则列表及实档待查。

#### 网络连接规则

GdbNetwork.dll 10.1.0.3035 的持久化表 `0x7119dcd0`、
`0x711a1760` 对应 EdgeConnectivityRule、JunctionConnectivityRule。
类型库 CLSID 分别为 `{F290D9E0-58E5-11D2-AB26-000000000000}`、
`{F290D9E1-58E5-11D2-AB26-000000000000}`。Load
`0x710d6330`、`0x71144aa0` 均接受版本 0、1，先读 String。
边规则随后读八个 long 和有符号计数，逐项在 `0x710d647f`、
`0x710d6491` 读两个 long。两段原生数组相隔 256 字节，
解析器将正计数限制为 64。交汇点规则读十个 long 和一个 short。
构造书签覆盖两类规则及后续权重列表、版本尾段和截断回滚。
实际 MXD 宿主仍待验证。

#### DERelationshipClass

FeatureDataElements.dll 10.1.0.3035 的 RTTI 对应持久化表
`0x6f65da14`，构造函数在 `0x6f617bdd` 写入该表。类型库的
DERelationshipClass CLSID 为 `{D41F5487-6B16-42FD-AB61-63A55B519B71}`。
Load `0x6f619e90` 接受无符号版本 0 至 6。v0–4 在 `0x6f619ee6`
调用 DEDataset::Load，v5–6 则在 `0x6f619ef8` 调用 DETableBase::Load。

v2 起，依次读取两个 IStringArray 槽、两个 long、两个 String、
两个 long、三个 short，以及两组关系键列表。第二个 short 存到
持久化接口 `+0x13a`，参与下方的旧版字段门。两组列表都调用
`0x6f61ba10`，该函数读有符号计数，逐项请求
`{7C2C531E-4882-40F5-BACE-323EF546D33C}`（IGPRelationshipClassKey）。
v3 起，再读一个有符号计数和 IRelationshipRule 槽列表，IID 为
`{EB3B2F82-7E5C-11D2-AAB5-00C04FA37B82}`。

随后，v0–3 在接口 `+0x13a` 的 short 非零时加载 IFields，
v4 无条件加载 IFields，v5–6 的 Fields 由 DETableBase 读取。
v6 在 `0x6f61a14c` 最后追加一个 short。实际 MXD 宿主仍待验证。

构造函数 `0x6f617cca` 将完整对象 `+0x156`（持久化接口
`+0x13a`）初始化为零，因此 v0–1 不消耗尾部 IFields 槽。
在 `0x6f617c3c` 设置完整对象 `+0xc0` 的扩展字段开关，
v5–6 走 DETableBase 的完整路径。主体已接入，构造书签验证
v0–4、旧版字段槽回引和截断回滚。
专项构造测试另覆盖 v5–6 的 DETableBase 切换、Indexes 对象、
跳过旧版 IFields 槽、v6 尾字段及每处截断。

#### GPRelationshipClassKey

FeatureDataElements.dll 10.1.0.3035 的 GPRelationshipClassKey
持久化表 `0x6f6623fc` 中 GetClassID `0x6f62fb00` 返回
`{5DCA4E1F-81A4-41C3-B160-818D22FC50B0}`，与类型库一致。
Load `0x6f62fb40` 接受无符号版本 0、1，再读一个 long 和两个
AfCore String。Save 入口为 `0x6f62fbc0`。构造书签覆盖两组非空
关系键列表及跨列表同键回引、后续字段边界和截断回滚。
其他接口实现及实际 MXD 宿主仍待核对。

#### RelationshipRule

GdbCore.dll 10.1.0.3035 的 RelationshipRule 持久化表
`0x70adc420` 中 GetClassID `0x70a45270` 返回
`{F84C6C1E-47FF-11D2-9933-0000F80372B4}`。Load `0x70a452d0`
接受无符号版本 0、1，直接读取一个 AfCore String 和十个 long，
Save 入口为 `0x70a45400`。构造书签覆盖非空列表、回引及截断回滚，
并验证随后 IFields 槽的读取边界。其他实现及实际 MXD 宿主仍待核对。

#### DECatalogDataset

FeatureDataElements.dll 10.1.0.3035 的持久化表 `0x6f658404`
对应 RTTI DECatalogDataset。GetClassID `0x6f5f1a90` 返回
`{74549FCD-9FEA-480F-A974-AE003684564D}`。Load `0x6f5f2020`
在 `0x6f5f2052` 按有符号短整数比较版本，`0x6f67e878` 的
上限为 1。随后调用 DEGeoDataset::Load，最后读两个 long。
构造测试覆盖版本 1、负版本、尾部标量、回引和截断。
实际 MXD 宿主仍待验证。

#### DEServerConnection

FeatureDataElements.dll 10.1.0.3035 的构造函数在 `0x6f61c284`
写入 RTTI 对应的持久化表 `0x6f65e830`。类型库 CLSID 为
`{887D4940-9343-42A3-8C8F-5C41D4DCFAFD}`。Load `0x6f61c7d0`
接受无符号版本 0、1，经导入槽 `0x6f657584` 调 DataElement::Load，
随后在 `0x6f61c827`、`0x6f61c831` 读取两个 AfCore String。
构造测试覆盖基类、双字符串、对象边界和逐字节截断。
实际 MXD 宿主仍待验证。

#### DEWorkspace

FeatureDataElements.dll 10.1.0.3035 的 DEWorkspace 持久化表
`0x6f65f778` 对应类型库 CLSID `{2127AF36-CAE1-427E-B1A8-BF0F1AD19F6E}`。
Load `0x6f621300` 接受无符号版本 0 至 5，先调用 DataElement::Load。
v3 起在 `0x6f621378` 读工厂 BSTR，再在成功的工厂解析路径读取两个
AfCore String。其后所有版本读取 IPropertySet 和有符号 IDomain 计数，
正计数在 `0x6f62146c` 请求 IID `{DF18FE0A-84BA-11D2-AB61-000000000000}`。
v1 另读一个四字节值，v4 起读三个 long，v5 再读 String。
解析器保留工厂标识字符串，沿用现有属性集与域读取器。
构造测试覆盖全部版本、非正域计数和截断。实际 MXD 宿主仍待验证。

#### DERasterCatalog

FeatureDataElements.dll 10.1.0.3035 的构造函数在 `0x6f61492a`
写入持久化表 `0x6f65ccc0`，在 `0x6f6149a2` 复制 CLSID
`{80207192-7CE8-4D06-9542-763E78518FFB}`，并在 `0x6f6149e0`
启用扩展字段。Load `0x6f615610` 读取无符号版本 0、1，
经导入槽 `0x6f65735c` 调 DEFeatureClassBase::Load 后返回。
解析沿用 DEFeatureClassBase 读取逻辑并保留独立的目录版本字段。
构造书签覆盖两类对象、同键回引和截断。实际 MXD 宿主仍待验证。

#### DERasterDataset

FeatureDataElements.dll 10.1.0.3035 的持久化表 `0x6f65d634`
对应类型库 CLSID `{8295BB68-F200-4CA6-803F-979952D732BD}`。
Load `0x6f616870` 接受无符号版本 0–2，调用 DEGeoDataset::Load，
再读取三个 AfCore String、long 和 short。v2 在 `0x6f61694e`
经 LoadObject 请求 `{30B9828A-85A6-4228-A7A4-7AFB8A9842CC}`，
类型库对应 IRasterStorageDef。构造测试覆盖所有版本、空存储槽和
截断。实际 MXD 宿主仍待核对。

#### RasterStorageDef

RasterCatalog.dll 10.1.0.3035 的安装介质 SHA-256 为
`0b6ac9b5e0c3e8efb715d68c5a55de89275def9ec7f666ffae91f0ff7972c585`。
持久化表 `0x7be40ae4` 的 GetClassID `0x7be37950`
返回 `{82ABC602-67F9-4042-BEE1-645B0139FD70}`。
Load `0x7be397e0` 接受版本 0–4，先读六个 long。v2 起读
一字节标志，非零时读两个 double，之后读取 IPoint 槽
（IID `{1B077915-38C5-11D0-92D2-00805F7C28B0}`），
v3 起读 short，v4 再读 String。解析保留坐标位模式和点对象槽。
构造测试覆盖 v4、非空 Point、后续字段边界、存储定义回引和逐字节截断。

#### DERasterBand

FeatureDataElements.dll 10.1.0.3035 的持久化表 `0x6f65c898`
对应 CLSID `{37C63B33-C7D6-4311-91D5-9E98A2A0E39A}`。
Load `0x6f613350` 接受版本 0–3，先加载 DETableBase。
构造函数在 `0x6f6121f4` 调基类构造函数，后者在
`0x6e94cbc8` 清零扩展标志，DERasterBand 构造路径保留该值。
因此基类在前导 short 和 String 后直接读 IFields、IIndexes。
其后读取 short、两个 double、两个 long，v3 再读一个 long。
v0–1 保留 `0x6f61348d` 读取的 24 字节，v2–3 读取紧凑 Variant。
末尾为两个 long、IEnvelope 和 ISpatialReference。构造测试覆盖各版本
分支及截断。实际 MXD 宿主仍待验证。

#### DEServerObjectBase

FeatureDataElements.dll 10.1.0.3035 的 RTTI 持久化表和
esriGeoDatabase.olb 类型库对应如下。

| 类 | 持久化表 | CLSID |
| --- | --- | --- |
| DEGeoDataServer | `0x6f65a86c` | `{D6E941E4-9EF4-438D-AE31-2FB082A2814D}` |
| DEMapServer | `0x6f65c14c` | `{69D74387-7684-458E-84A3-BD9E013669C0}` |
| DEGlobeServer | `0x6f65b1b4` | `{9ADF8E53-B3FC-4348-B0C1-27BF1A5F5F39}` |
| DEGPServer | `0x6f65b4a0` | `{A8177ADA-390D-420C-A433-EE67DD2E8638}` |

四表的 Load 槽均经 `0x6f64a492` 转发 DECoreLib
DEServerObjectBase::Load `0x6e94c430`。该函数接受无符号版本
0、1，调用 DataElement::Load，再读两个 AfCore String。
布局与 DEServerConnection 相同，解析复用该路径，类标识由对象头保留。
构造测试验证五类对象体和截断，实际 MXD 宿主仍待核对。

#### 文件数据元素

FileDataElements.dll 10.1.0.3035 的 SHA-256 为
`5c4703596f243daf7025cc1d0e435cf6522dbcb4a02b77529d0694aef4ef5f3c`。
RTTI 与 esriDataSourcesFile 类型库对位如下。

| 类 | 持久化表 | CLSID |
| --- | --- | --- |
| DEFile | `0x6f9da29c` | `{175CF2C5-12BF-413A-8FDF-3DA30CC09BEE}` |
| DEFolder | `0x6f9da5fc` | `{273CCEBC-1AA2-4429-BBD5-4CB7C6E20465}` |
| DEMapDocument | `0x6f9db2b8` | `{C0BED5D9-13E2-469B-A7D8-22704B0C8630}` |
| DEDiskConnection | `0x6f9d9f80` | `{8E5E855A-7D60-4CE1-AFD0-45929C9F492C}` |

四类共用 Load `0x6f9c3b90`，接受无符号版本 0、1，
随后经 `0x6f9d7454` 调 DataElement::Load，基类之后无新增流字段。
构造测试覆盖四类对象、结束位置及逐字节截断。实际 MXD 宿主仍待验证。

#### DEPrjFile

FileDataElements.dll 10.1.0.3035 的 DEPrjFile 持久化表
`0x6f9db56c` 对应类型库 CLSID `{7E704BB0-CFBE-11D5-933D-0080C71A3226}`。
Load `0x6f9c17c0` 接受无符号版本 0、1，读 DataElement 后
在 `0x6f9c181d` 加载 IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`
的 ISpatialReference。解析复用空间参考读取器，构造测试覆盖空槽和
后续字节边界。非空实档和 ArcMap/QGIS 人工验收仍待验证。

#### DETextFile

FileDataElements.dll 10.1.0.3035 的持久化表 `0x6f9dc14c`
对应 CLSID `{E98D0766-C13E-45CA-BA78-1DEF8B60CAB2}`。
Load `0x6f9c4890` 接受无符号版本 0–2，v1 调 DataElement::Load，
v0 和 v2 调 DETableBase::Load，各版本均在基类后结束。
构造函数在 `0x6f9c42c5` 调 DETableBase 构造函数，
保留基类默认的零扩展标志，和 DERasterBand 共用简短表布局。
构造测试覆盖三个版本、基类选择、结束位置和逐字节截断。
实际 MXD 宿主仍待核对。

#### DELayer 与 GPLayer

FileDataElements.dll 10.1.0.3035 的 DELayer（CLSID
`BDD71708-3463-4787-A491-B9363B937DBE`）持久化表为
`0x6f9dafac`。Load `0x6f9bf970` 检查无符号版本上限 1，
调用 DataElement::Load 后，以 `0x6f9db0a4` 的
`EA30B18D-2D75-4593-BF44-B3620F41D8DB` 请求 IGPLayer，
接口名称由 esriDataSourcesFile 类型库核对。

esriGeoprocessing 类型库将 `6A47591E-AA16-443E-B50D-D8926659A4E1`
标为 GPLayer。GpObjects.dll 10.1.0.3035 的持久化表
`0x747b164c` 的 Load 跳板 `0x7478b37e` 转入 GeoprocessingLib.dll
10.1.0.3035 的 GPLayerBase::Load `0x71d72200`。该函数读取
无符号版本（上限 2）、GPValue、AfCore String、IEnvelope 和
IDataElement。版本 0、1 还读取并释放 ILayer。三个接口 IID
分别来自 `0x71e349b4`、`0x71e34a04`、`0x71e34c64`，均经 10.1
类型库核对。旧版 ILayer 复用已有的完整图层体读取器。

构造测试覆盖 GPLayerBase 三个版本、逐字节截断、超限版本和
结束位置。真实 MXD 宿主和旧版非空 ILayer 仍待验证。


#### GPCompositeLayer

esriGeoprocessing 类型库将 `22994B51-C6BA-41FF-ABE6-549615441E3F`
标为 GPCompositeLayer。GpObjects.dll 10.1.0.3035 的持久化表
`0x747ab418` 指向 Load `0x746ecce0`，读取无符号版本（上限 1）
后经跳板 `0x7478afe8` 调入 GeoprocessingLib.dll 10.1.0.3035
的 GPCompositeLayerBase::Load `0x71d0c040`。基类另读无符号
版本（上限 1），调用 GPLayerBase::Load，随后读取有符号
32 位数量。正数逐项以 `0x71e39b80` 的 IGPLayer IID 调用
LoadObject，非正数跳过循环。空项仍占流内位置。
构造测试覆盖成员循环、空项、回引、逐字节截断和结束位置。
真实 MXD 宿主仍待验证。

#### 目录数据元素

FileDataElements.dll 10.1.0.3035 的 PE RTTI 持久化表及
esriDataSourcesFile 类型库的 CLSID 如下。

| 类 | 持久化表 | CLSID |
| --- | --- | --- |
| DECatalogRoot | `0x6f9d8f34` | `01D6EBDD-DE9A-4217-8BFE-327265855FA2` |
| DERemoteDatabaseFolder | `0x6f9db84c` | `DB3E6FDF-D5F8-4C94-B743-47AB4DE79A05` |
| DESpatialReferencesFolder | `0x6f9dbf8c` | `3189F8FC-A9C8-4B8E-8C47-43C758A7A8BD` |

三个持久化表共享 Load `0x6f9c3b90`。它检查无符号版本上限 1，
经导入槽 `0x6f9d7454` 调用 DataElement::Load 后结束。
构造测试覆盖三个 CLSID、基类结束位置和逐字节截断。
真实 MXD 宿主仍待验证。

#### DEDbaseTable 与 DEVPFTable

FileDataElements.dll 10.1.0.3035 的 PE RTTI 持久化表与
esriDataSourcesFile 类型库对应关系如下。

| 类 | 持久化表 | CLSID |
| --- | --- | --- |
| DEDbaseTable | `0x6f9d9ae4` | `69E84176-FB98-4036-A8FF-E69C8E82F013` |
| DEVPFTable | `0x6f9dceac` | `48AAB232-6F1F-4261-9DAC-CBA55D183649` |

两类共享 Load `0x6f9badf0`，读取无符号版本（上限 1），
经导入槽 `0x6f9d7438` 调用 DETableBase::Load 后结束。
构造路径分别于 `0x6f9ba955`、`0x6f9c7be5` 调用基类构造函数，
保留扩展标志默认值，复用简短表体读取布局。
构造测试检查两类对象的表字段、结束位置和逐字节截断。
真实 MXD 宿主仍待验证。

#### DECadDrawingDataset

FileDataElements.dll 10.1.0.3035 的持久化表 `0x6f9d8b8c`
对应 esriDataSourcesFile 类型库的 CLSID
`F786332D-20B2-4607-BF34-0A53E4D0B413`。
Load `0x6f9b6540` 接受无符号版本 0、1，经导入槽
`0x6f9d71e4` 调 DEGeoDataset::Load，随后从
`0x6f9b659e` 起读取四个 short，依次写入 +0x90、+0x92、
+0x94、+0x96。构造测试覆盖四个字段、截断与结束位置。
字段语义和真实 MXD 宿主仍待核对。

#### DETin

FileDataElements.dll 10.1.0.3035 的持久化表 `0x6f9dc720`
对应 esriDataSourcesFile 类型库中的 `5EDC32F1-C1B2-4808-88E0-2ED68E77269D`。
Load `0x6f9c5900` 检查无符号版本上限 1，调用
DEGeoDataset::Load，再依次读取 double、四个 short 和 IFields 槽。
`0x6f9c59df` 的 LoadObject 使用 `0x6f9dc880` 的 IID
`6B2072E3-23F7-11D1-89D8-006097AFF44E`，由 esriGeoDatabase
类型库确认为 IFields。构造测试验证原始位模式、字段回引、结束位置
与逐字节截断。标量字段语义及真实 MXD 宿主仍待核对。

#### DECoverage

FileDataElements.dll 10.1.0.3035 的持久化表 `0x6f9d920c`
对应 esriDataSourcesFile 类型库中的 `96B70C47-2C7F-405F-B9FD-DEAA3CDC084C`。
Load `0x6f9b8020` 检查无符号版本上限 1，经导入槽
`0x6f9d71e4` 调用 DEGeoDataset::Load，随后在 `0x6f9b8080`
以 `0x6f9d8674` 的 IID `F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`
调用 LoadObject。esriSystem 类型库确认该 IID 为 IPropertySet，
读完该槽即完成对象体。构造测试覆盖新 PropertySet 对象、登记键、
逐字节截断和结束位置。真实 MXD 宿主仍待验证。
