# 第九轮：FeatureBookmark 的 IName 与 TableName

日期：2026-11-17。此处为静态原生证据和合成测试，不是目标软件运行记录；ArcMap 10.1 暂不可用，QGIS 也未做目视检查。仓库参考样本书签计数为 0；没有声称在书签中实际观察到 TableName 或证明名称对象的分布。

## 身份与方法

本轮重算 Map.dll（`temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`）SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，以及 GdbCore.dll（`temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`）SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；两者 PE32 x86，文件/产品版本均声明 `10.1.0.3035`，原始文件名分别 Map.DLL、GdbCore.DLL，与前轮一致。这些是静态身份和自述版本，不是签名认证。`third_party/ForestRight/vendor/ESRI.ArcGIS.System.dll` 的 SHA-256 为 `657c2072703fa9904a9f25bc56c4a46425b86ba2af89f9e7c3faddd6a307a766`，文件和产品版本资源均为 10.1.0.3035。GdbCoreLib 的同版身份及持久化链见 [第八轮](mxd-terra-round8-tablename.md)，本轮未重新计算其哈希。

独立 Ghidra 桥接工具 6.9.0 CLI、JDK 25、`/opt/ghidra`、仓库内专用 TMPDIR 分析 Map.dll Load；run `d4dda94a-5c97-4f39-b047-a2736cc97f45` 的 Ghidra 桥接 Unix socket 报 `Operation not permitted`，未产生函数 Evidence。按预定回退使用复制的独立 Map/GdbCore 工程、pyghidra/Ghidra 12.2 和 JDK 25，隔离项目、缓存及临时目录，重新反编译目标及辅助函数；另用 PE 原始虚表、objdump 指令和既有类型库转储核查。未连接或切换共享 Ghidra 桥接工具 目标。`temp/terra-round9-bookmark/` 为忽略的临时证据，分享前应检查敏感路径。反编译不是原始源码。

## 构造、虚表、读写与语义

1. Map `0x793b5560` 构造完整对象，在 +0x10 安装 `0x79482bac`、+0x14 安装 `0x79482be8`；后者虚表 +0x14/+0x18 分别指向 Load `0x793b5c50` / Save `0x793b5d70`，+0x0c 的 `0x793b5520` 写出 FeatureBookmark CLSID。构造初始化完整对象 +0x18/+0x1c 为零，+0x20 初始化字符串；Load 的 this 是 +0x14 视图，因此 Load this+4/+8/+0xc 正好对应完整对象 +0x18/+0x1c/+0x20。
2. Load 依次读 short、int32、请求 IName `{4BA33331-D55F-11D1-8882-0000F877762D}` 的 LoadObject 和字符串。Save 写版本 1、int32、名称对象和字符串。`0x793b5c8c..91` 是有符号 `cmp 1; jg`：**负值也通过原生版本门**；旧文档称负值拒绝是错误的。读取器仍拒绝负值位模式和大于 1 的值，是保守子集，不是原生全部接受范围。
3. +0x10 视图虚表 `0x79482bac` 的 +0x1c/+0x20 指向 `0x793b5ba0`/`0x793b5bd0`；两者分别读取/写入该视图 +8，即完整对象 +0x18，与持久化 int32 同一成员。尚未把访问器与类型库成员名可靠对位，因此不把它命名为特定业务属性。
4. Load `0x793b5d13..2a` 对加载的 IName 指针调用其 +0x14 槽，传入输出指针；结果经 `0x793acdc0` / `0x793ad1c0` 对返回对象查询 IFeatureClass `{D4803EE6-79F4-11D0-97FC-0080C7F79481}`，成功保存该接口，失败或空值使目标引用为空。该调用的 HRESULT 未被此处用于终止读取。旧 [Legend 记录](arcmap-10.1-legend-family.md)声称 IName 只有 get_NameString、该槽越过接口的方法数，这一判断不成立；QI 是针对调用所得对象，而非直接对 IName 本身。
5. GdbCore `0x709b9330` 构造 FdbTableName：完整对象 +0x10 安装 `0x70ad071c`；QI `0x709b9647..64` 将 IName 映射到该视图。该虚表 +0x0c/+0x10 为跳板 `0x70a63626`/`0x70a6362c`，导入符号分别为 `FdbDatasetName::put_NameString` 和 `get_NameString`；+0x14 为 `0x709b9910`。新反编译显示后者检查工作空间引用、对名称对象再调用 +0x14 并在得到的对象上按字符串访问，最后向输出参数查询接口；主审提供的 CLR COM 元数据提取经本轮复核：`dotnet temp/terra-round9-iid/probe.dll third_party/ForestRight/vendor/ESRI.ArcGIS.System.dll 4ba33331-d55f-11d1-8882-0000f877762d` 输出 IName 的 GuidAttribute 与方法顺序 `set_NameString`、`get_NameString`、`Open`；结合 IUnknown 三槽和 x86 虚表，+0x14 确认为 IName::Open。不保证该路径返回 IFeatureClass；TableName 可持久化为 IName 这一事实不证明其能形成有效要素书签。
6. TableName 的 CLSID、IPersistStream 视图、经 GdbCoreLib `FdbDatasetName::Load/Save` 的版本、三字符串及 IWorkspaceName 体见 [第八轮](mxd-terra-round8-tablename.md)。本次重新核对构造和 IName 虚表，使其与书签的请求接口相连；未以其他 IName 实现的名称推断其可用性。

## 实现与验证

`map_surround` 中书签名称槽一次读取对象头并按空值、回引、FeatureClassName、TableName 分派；TableName 复用 `standalone_table::read_table_name_body_10_1`，该函数仅扩大为模块内可见。TableName 嵌套工作空间或其子对象未读完时书签返回 Unsupported，整个书签游标和引用表回滚；不模拟原生实例创建、Open、QI 或数据源访问。由于限定修改范围，原先 `dataset_name_more` 的独立书签槽助手没有扩展；此新能力属于 FeatureBookmark 入口。TableName 读取器的版本限制同样比原生有符号门更严。

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
  FeatureClassName 并保存其非委派 IUnknown。外层 QI 在
  `0x6fa3317d..0x6fa3318d` 将 IName 请求交给该内置对象。
  外层 IPersistStream 仍使用独立的 `0x6fafe0c4` 虚表和
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
