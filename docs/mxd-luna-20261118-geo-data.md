# geo-data 轮：数据集名称、几何与空间参考（2026-11-18）

本轮仅记录指定 TODO 原行的核对结果，不修改 `TODO.md`、`README.md`、`lib.rs`、`mod.rs` 或共享宿主。Rust 解析器现状按工作区当前代码核对；原生结论限 ArcMap 10.1.0.3035 本机介质文件的静态反编译、汇编和类型库证据。静态分析、真实安装介质模板字节、合成测试与目标软件运行是不同证据等级。

## 10.1 原生来源与研究工具

- [Geometry 研究记录](arcmap-10.1-geometry.md)及[空间参考研究记录](arcmap-10.1-spatial-reference.md)引用介质 Geometry.dll，SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`，固定文件版本 10.1.0.3035。Geometry::Load 地址包括 Polyline/Polygon `0x7192db20`、Point `0x7191c1a0`、Multipoint `0x718ff9c0`；Envelope::Load 为 `0x71869b60`。既有反编译、虚表与原始指令记录用于本轮静态核对，未从版本字符串扫描推断对象体。
- [DatasetName 研究记录](arcmap-10.1-dataset-name.md)、[字段调用链](mxd-terra-depth-names.md)及[六类实现记录](arcmap-10.1-dataset-name-slots.md)引用介质 GdbCore.dll，SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，固定文件版本 10.1.0.3035。FeatureDatasetName::Load `0x709b7c90`、TopologyName::Load `0x7095e5c0`；六类 IDatasetName Load 分别为 GeometricNetworkName `0x709b8ca0`、RelationshipClassName `0x70a3b330`、MemoryRelationshipClassName `0x70a402e0`、QueryTableName `0x70932a10`、RelQueryTableName `0x70a5a530`、DBTableName `0x708d7a40`。
- 本轮根据指派尝试恢复的 REA 6.9.0 CLI 对 Geometry.dll `0x7192db20` 执行函数分析。Ghidra 12.2/JDK 25 成功启动并装入目标二进制，但全模块自动分析在 25 秒和 150 秒两次限时内都未完成，REA 取消任务；没有获得函数结果或生成补导伪代码。命令日志及隔离分析目录在忽略的 `temp/luna-20261118/geo-data-rea/`。这是限时取消，不是成功的 REA 证据，也没有观察到 Unix socket 拒绝。未转而运行 pyghidra。本轮没有目标软件运行取证。

## TODO 原行逐项裁决

| 原行 | 状态 | 证据与缺口 |
| --- | --- | --- |
| 242 | 部分 | FeatureDatasetName 四字符串、标志和 WorkspaceName 槽结构由 GdbCore Load `0x709b7c90`/Save `0x709b7d50` 静态对位；前三个字符串已绑定 Name、IName.NameString、Category，第四串和标志对应成员及业务语义仍未定。`src/objects/dataset_name.rs` 已保留原始字段；没有依据再赋业务名称。介质 Africa 的 FileGDB 局部链只有 FeatureDatasetName v1，不能覆盖未定字段的实际行为。 |
| 259 | 待运行验证 | Geometry 10.1 静态记录已定位几何内核移交候选对象 `+0xc8` 内嵌指针及虚表 `+0x1cc` 槽，但运行期装配和内核侧校验仍未观察。读取器仅按静态包装字段/记录布局解析；不可由静态虚表或合成测试替代 ArcMap 运行证据。 |
| 266 | 部分 | GdbCore 六类 Load 及对象流字段顺序已记录；五条请求 IID 在 10.1 类型库中分别定名为 IName、IQueryDescription、IQueryFilter、ISelectionSet、ISpatialReference。字段槽和请求接口名不等于业务语义，六类标量/String 的访问器、类型库成员及同槽绑定仍未全闭合。`src/objects/dataset_name_more.rs` 保留尚未对位的标量、字符串和槽；缺证据的字段不改名。 |
| 268 | 部分 | PCS/GCS/Unknown 的持久化体版本布局、PCS 精度标志和坐标网格字段，以及 GCS/Unknown 三分派已有静态实现。四个域字符串的业务含义仍未逐项定性；Unknown 的四个 String 写入同一成员的两项覆盖已由读写静态访问对位，但这些字符串的业务名称和填充来源仍未确定。相关值在 `src/objects/spatial_reference.rs` 与 `src/objects/spatial_reference_slots.rs` 保持原样。 |
| 274 | 部分 | FeatureClassName 的第二选择子非零分支及 TopologyName 体读取器已在 `src/objects/dataset_name.rs` 接入，合成输入覆盖该读取链；本机参考样本和已检查的 10.1/10.8 安装介质 `/Maps` 未找到可确认对象边界的 TopologyName 实例/非零宿主。不能将合成输入升级为真实旧档佐证。 |
| 298 | 部分 | Geometry v0/v1 的 Load 版本门及“记录后不读空间参考”分支由 Geometry.dll `0x7192db20` 静态核实，`src/objects/geometry.rs` 已实现；`tests/geometry.rs` 有版本 1 与记录边界合成覆盖。已检查模板没有可完整走查的真实 v0/v1 Geometry 对象体，介质中字节候选不能独立确立宿主边界。 |
| 299 | 待验证 | Polyline/Polygon Load `0x7192db20` 的 AfStreamRead 返回 S_FALSE 捷径见既有 10.1 静态记录；当前严格 Reader 没有该 HRESULT 状态，也未实现原生“空流”捷径。仓库与介质检查未取得可复现该返回状态的宿主旧档，目标软件运行行为未取证，不扩写为猜测分支。 |
| 303 | 部分 | Envelope::Load `0x71869b60` 的 v0 与 v2 同体布局由 10.1 静态分支核对；`src/objects/envelope.rs` 已按 v0 读取标志及可选 Z/M 对，不读空间参考；`tests/envelope_v0_round2.rs` 覆盖四种标志组合、尾随边界和截断回滚。未发现有对象边界确认的真实 Envelope v0 旧档字节。 |
| 304 | 部分 | FeatureDatasetName::Load `0x709b7c90` 接受版本 0，字段布局与 v1 相同；当前读取器已接受并有合成覆盖。10.1 介质 Africa 的可走查对象为 v1，10.8 介质候选头后版本亦为 1；没有真实 v0 旧档体。 |
| 305 | 部分 | TopologyName::Load `0x7095e5c0` 接受版本 0，字段布局与 v1 相同；当前读取器已接受并有合成覆盖。已检查的参考样本与安装介质没有可确认边界的 TopologyName 对象，因此没有真实 v0 旧档字节。 |
| 344 | 部分 | 静态证据确认 GdbCore FeatureClassName::Load `0x709b5080` 的非零首选择子以 IID IDatasetName 装载对象；FeatureDatasetName::Load `0x709b7c90` 的末槽以 IID IWorkspaceName 装载 WorkspaceName。10.1 类型库成员序与 getter `0x7091b960` 将该槽定为 IDatasetName.WorkspaceName；getter 读取完整对象 `+0x4c`。`src/objects/dataset_name.rs` 提供 `workspace_path() -> Option<&str>`，从已解析的新 WorkspaceName 体借出路径原文；`None` 时原 `WorkspaceNameSlot10_1` 仍区分 Null、BackReference、Unsupported，嵌套停止原因仍由 `FeatureDatasetNameBody10_1::stop` 保留。Africa 局部宿主链和可构造槽输入均已通过测试。共享宿主的 `FeatureDatasetName` 分支由主代理另行接线，本轮未改该共享文件。未推断或规范化本机路径。 |

### 344 的 Africa 局部边界与访问器验证

10.1 安装介质 Africa `/Maps` 局部链的既有证据（[介质局部记录](mxd-dataset-format-evidence-20261117.md)）与当前实现再次验证一致：`0x31e` 为 FileGDB FeatureClassName 外层对象键 43；内层 FeatureClassName v2 的首选择子位于 `0x3a6` 且为 1，紧随其后的 FeatureDatasetName 对象从 `0x3a8` 起、键 44、版本 1；其 WorkspaceName 对象从 `0x45d` 起、键 45、版本 1。该 WorkspaceName 体的路径字段是流中字符串，PropertySet 槽为空标记，完整链边界为 `0x510`。本轮测试不输出路径值，只断言访问器借用值与已解析 WorkspaceName 的原始 `path` 完全相等并到达 `0x510`。这证明该局部宿主中的可解析分支，不证明 Africa 从 Map 根的完整链路。

GdbCore 原生证据链进一步确认字段 getter：FeatureClassName `0x709b5080` 在首选择子非零时请求 IDatasetName IID `{89E75917-C287-11D1-AA77-00C04FA33A15}`；FeatureDatasetName 的持久化体 `0x709b7c90` 以 IWorkspaceName IID `{FADD975C-E36F-11D1-AA81-00C04FA33A15}` 读取 WorkspaceName 槽；10.1 类型库的 IDatasetName 成员序将 `get_WorkspaceName` 放在继 Name/Type/Category 成员之后，GdbCore 虚表 +0x20 getter `0x7091b960` 读写目标为完整对象 `+0x4c`。槽体按 WorkspaceName Load `0x70a27950` 解析，`path` 是其首个 String 字段。静态 getter、成员序与 Africa 局部字节相互吻合；未运行 ArcMap。

## 代码、样本与验证边界

本次相邻动作修改 `src/objects/dataset_name.rs` 的路径访问器、`tests/dataset_name.rs` 的构造测试及本研究记录；未改共享宿主。`cargo fmt --check` 通过；工作区 `cargo test --test dataset_name` 13 项通过；全量 `cargo test` 通过（193 个单元测试及各集成测试通过，含 ignored 项按测试框架默认跳过）。另有 Africa `/Maps` 独立局部测试 1 项通过：验证非零选择子、FeatureDatasetName/WorkspaceName 路径借出和流边界 `0x510`，不输出路径内容。Africa 测试只验证该局部宿主链，不证明从 Map 根的完整链路。主代理此前提供的基线为 796 项通过、17 项 ignored 实跑 17 项通过，以及 10.1 介质 63 + 仓库样本 1 个可 parse；这些是另一轮基线。本轮未在 ArcMap 或 QGIS 中人工目视验收。
