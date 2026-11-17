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
