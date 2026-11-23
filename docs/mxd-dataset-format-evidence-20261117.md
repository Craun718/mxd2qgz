# 数据源、IID 与数字格式开放项复核（2026-11-17）

本记录对应本轮任务指定的 TODO 原行号；不修改 TODO、共享模块或既有成果。静态反编译、真实介质文件的局部结构、合成测试和 ArcMap 运行验证是不同证据等级。此前 Ghidra 桥接工具 证据仍以各原研究文档为准；本轮未重新启动 Ghidra 桥接工具，也未运行 ArcMap 或 QGIS。

## 原件、版本与查找范围

- 10.1 介质 Setup1.cab 包含 66 个 MXD/MXT，`temp/arcmap101-media-streams/index.json` 给出文件 SHA-256、版本候选及流清单，66 个均有 `/Maps` 和 `/PageLayout`。版本候选分布：9.3/16 34、9.2/14 3、10.0/18 11、10.1/19 18；并非运行时版本鉴定。
- 10.8 介质 Data1.cab 的 68 个模板索引在 `temp/arcmap108-media-streams/index.json`；磁盘版本候选是 10.1/19 26、9.3/16 18、9.2/14 12、10.0/18 11、8.1/10 1。10.8 介质出身不等于磁盘格式 10.8，也不自动证明目标版本的运行行为。
- GdbCore 10.1 已提取原件 SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，本轮重算一致。10.1 安装介质身份及静态 Load/Save、Ghidra 桥接工具 Evidence 见 [数据源局部证据](mxd-parse-agent-sample-evidence-data-20261117.md) 和 [数据集槽研究](arcmap-10.1-dataset-name-slots.md)。
- 本机仅发现仓库参考样本及上述介质模板；Wine 可执行程序存在，但 README 标注 ArcMap 10.1 未安装、未人工验收；不能把 Wine 可用视为可用 ArcObjects 运行环境。旧版软件或可以产生指定对象状态的受控运行环境尚未确认可用。

## 介质模板局部结构走查

10.1 介质 Africa.mxd SHA-256 `ee6be21c409285644e81177a19878f5149cd2f3380b95f188fdfa5788d466674`，`/Maps` SHA-256 `0d967ec22e59f6a8162c70df9614547cd636197fcbe3fa7d30b954c6e342401f`，`/Version` 文本 10.1、short 19。以下偏移均为 `/Maps`，字符串只记录编码长度，不记录潜在敏感值：

1. `0x31e` 完整 FileGDB FeatureClassName 类标识及键 43；包装 short 1、short 0，随后内联 FCN 版本 2。按 10.1 FileGDB/GdbCore 已闭合布局读取四个长度前缀字符串和两个 int32。
2. `0x3a6` 首选择子为 1；紧随 `0x3a8` 为完整 FeatureDatasetName CLSID 和键 44，体版本 1。四个字符串的字节长度依次为 12、2、66、62；标志字节为 0。
3. 体内 WorkspaceName 槽自 `0x45d` 起：完整 CLSID、键 45、版本 1、三个字符串长度 78、2、26；`0x4e9` 为 16 字节空 PropertySet 标记，后续工厂标志 1、16 字节工厂 CLSID、尾部 int32 1。工作空间体止于 `0x50e`，FCN 第二选择子为 0，后续偏移 `0x510`。
4. 相同规则对 10.1 介质 66 个 `/Maps` 的完整 FGDB CLSID 候选逐一尝试，145 处局部链到达非零选择子、FeatureDatasetName 版本 1 和 WorkspaceName CLSID；这是局部字节走查计数，不声称 145 个键均为全流首次出现，也不声称从 Map 根对象完整遍历到每一处。Africa 模板的图层外层被当前 CLI 保留为未知有界体，故尚未建立根到此数据源的完整路径。需要使用实际解析器和完整对象登记表复核登记/回引及样本链路。

针对同一批类做完整 16 字节 GUID 候选筛查：66 个 10.1 介质 `/Maps` 未发现 TopologyName 或 Hyperlink；10.8 介质 68 个 `/Maps` 中 FeatureDatasetName 290 次候选，其对象头后 short 均为 1，未发现 TopologyName、Hyperlink 或非 Numeric 的八种 NumberFormat GUID 候选。候选计数不证明对象数量；零命中只限所扫描的流，不能证明其他流或其他文件不存在。10.8 介质的 290 处未逐条建立对象边界。Numeric 在两批中均有候选，尚未从宿主槽对新模板的 Numeric 体进行结构走查，因此本轮不把它们升级为新样本验证。

## 指定 TODO 原行逐项裁决

| 原行 | 状态 | 证据与剩余缺口 |
| --- | --- | --- |
| 176 | 部分 | TableQueryName 候选 Load `0x708f5070`/Save `0x708f5280` 有 Ghidra 桥接工具 静态观察，仍有间接调用/写侧 helper 与完整字段边界未解；FeatureQueryName 的完整体未定位。详见 [变体流研究](mxd-table-query-name-variant-20261116.md)。需验证虚表归属、helper 与对象状态，不能实现猜测布局。 |
| 177 | 阻塞 | ObjectClassName 的 CLSID 配置已知，GdbCore 静态检索仅发现接口 TD，未找到实现类 RTTI/COL/虚表；不能以接口 RTTI 冒充类。需重新定位实现类或确认其实际模块。 |
| 216 | 部分 | FDN 四字符串及标志的对象偏移与顺序已有静态读写证据，介质局部链新增真实 v1 体；仍缺 getter/setter 或受控状态变化以绑定业务语义。 |
| 217 | 部分 | TopologyName 两字符串和槽顺序有静态读写证据，缺字段业务语义和可走查的实际对象。 |
| 225 | 已闭合 | 10.1 esriCarto 类型库直接导出 Document=0、URL=1、Macro=2、Script=3；枚举与 Hyperlink 体字段的绑定属于 226，未由此证明。 |
| 226 | 部分 | Hyperlink Load 首 int32 → 对象 +4；尚缺 getter/setter 指令与接口槽对应，不能断言就是 LinkType。 |
| 227 | 部分 | 第三字段是 String，尚缺 accessor 与成员偏移绑定，不能断言即热链字段。 |
| 228–229 | 部分 | 九类布局与版本门有 SystemCore 静态证据，但标量与 String 的业务名称未完整绑定；需对应类 accessor、类型库成员及原生同槽指令。见 [格式研究](arcmap-10.1-number-format.md)。 |
| 230 | 部分 | 四类内嵌 Numeric 核心的调用/对象槽已有静态证据；未观察到对应真实样本或运行状态，需受控保存并核查对象成员、嵌套字段。 |
| 238 | 部分 | 下节六个 IID 的身份已按 10.1 类型库与各模块调用点绑定；未穷尽所有模块的其余未定名 IID，不能整体勾选。第 237 行明确排除。 |
| 239 | 部分 | 五条请求 IID 的名称可由 10.1 类型库确认（见下）；六类标量/String 语义和请求槽内具体实现类未全闭合。 |
| 246 | 局部对象分支已闭合 | 原生静态解析、既有接线、合成测试及 Africa 真实模板 Rust 试读均支持非零选择子、FDN v1 和 WorkspaceName 体；空表登记三个实际键，游标到 0x510。根遍历与目标软件人工验收另行保留，详见下文。 |
| 247 | 部分 | 静态解析与接线存在；模板 `/Maps` 未发现完整 TopologyName GUID 候选。需实际拓扑工程的非零第二选择子对象或受控 ArcObjects 流。 |
| 262 | 部分 | Hyperlink v0 与 v1 同字段顺序已有静态证据和既有合成测试，未发现宿主真实 v0 字节；需旧档或受控历史写侧。 |
| 263 | 部分 | ObjectStream 通用空标记/回引和现有测试已知；没有 Hyperlink 宿主循环专属的空槽或重复对象字节。需带该循环形态的工程或受控保存。 |
| 264 | 部分 | 九类短体静态布局与合成测试已有；缺历史短体对象级样本，尤其 Percentage/Rate v0–1、LatLon v0–2、Angle v0–1、Scientific v0–1。 |
| 265 | 部分 | 同槽与独立槽有静态证据及合成实现；缺相应四类对象的结构样本和运行时槽状态。 |
| 266 | 部分 | Angle 核心失败返回值被忽略有静态观察；读取器仍严格报错。需受控失败流、HRESULT、游标与对象副作用后才能安全对齐。 |
| 276 | 部分 | FDN v0 与 v1 静态同字段且已有合成测试；介质走查得到的是 v1，10.8 介质 290 个 FDN 候选的头后版本也均为 1。需真实历史 v0 或受控旧版流。 |
| 277 | 部分 | TN v0 与 v1 静态同字段且已有合成测试；两批 `/Maps` 无 TN GUID 候选。需真实历史 v0 或受控旧版流。 |

## IID 对照：定名不代表对象字段定性

下列四个 FDO QI 项与 [AnnoLayer 原生 QI 表](arcmap-10.1-fdo-open-points.md) 对照，接口身份来自 10.1 安装介质类型库现存导出 `/tmp/mxd-arcgis101-guiddocs/wine/`：

| 完整 IID | 10.1 类型库名 | 原生 QI 子对象偏移 | 转储 |
| --- | --- | --- | --- |
| `{34C20004-4D3C-11D0-92D8-00805F7C28B0}` | IGeoDataset | +0x2c | esriGeoDatabase 第 20 行 |
| `{12E361F0-1907-11D3-80C7-0080C79F0371}` | IDisplayAdmin | +0x38 | esriDisplay 第 52 行 |
| `{EEF01B2D-8FE1-4759-8B93-51EE22DED70A}` | ILayerDrawingPhase | +0x3c | esriCarto 第 104 行 |
| `{2425B413-1FFC-4407-B9F0-B4AC9ED50BDE}` | ILayerGeneralProperties | +0x44 | esriCarto 第 106 行 |

DisplayCore 读槽与 [display-tails ⑤⑥](arcmap-10.1-display-tails.md) 对照：`{7914E5F4-C892-11D0-8BB6-080009EE4E41}` 为 ITemplate、`{533D88F0-0A1A-11D2-B27F-0000F878229E}` 为 ILineDecoration；分别对应 esriDisplay 转储第 208、209 行。此处确认接口名称，完整槽实现类另依后续 display 记录。

数据集六类请求 IID 的 10.1 类型库来源和精确转储行见 [数据源额外核查](mxd-open-dataset-extra-20261116.md)：`{4BA33331}` IName、`{2E060684}` IQueryDescription、`{FDFEBD93}` IQueryFilter、`{F7AD47D1}` ISelectionSet、`{4434D680}` ISpatialReference。与 [数据集槽 LoadObject 请求](arcmap-10.1-dataset-name-slots.md) 按完整 GUID 对照；类型库并不证明目标槽里具体 coclass 或其他业务字段。

## 测试、接线与限制

现有 `tests/dataset_name.rs`、`tests/hyperlink.rs`、`tests/number_format.rs` 是构造输入，不代表模板或 ArcMap 验收。本轮 `cargo fmt --check` 通过；定向 `cargo test --test dataset_name --test hyperlink --test number_format` 在编译阶段失败：并行修改的 `src/layers/legend_format.rs` 对含 `ElementBodyStop10_1` 的枚举派生 `Copy`/`Eq`，而该字段不实现这两个 trait（E0204/E0277）。不修改授权范围外的并行工作；主代理可在其变更稳定后处理并重跑。无本轮 Rust 改动，未运行全量测试。

FCN 与九类格式已有对应接线；无证据表明需要新增共享模块入口。主代理可考虑在自己的测试范围以脱敏、可再生成的真实模板链做结构回归，并在解析器可以完整追踪 Africa 的图层对象及键表后更新 TODO 246 的证据等级。不得把静态布局、GUID 定位或合成测试记为 ArcMap/QGIS 运行验收。

## 补充：246 的真实模板 Rust 试读和 225 枚举值

本轮在 `temp/dataset-format-media-20261117/rust-check/` 建立当前源码的隔离副本和一次性测试 `tests/africa_local.rs`，从 Africa `/Maps:0x31e` 开始，用 `read_object_entry_10_1` 对空 `RefTable10_1` 读取并精确比对 FileGDB CLSID、登记新键 43，再以 `read_fgdb_feature_class_name_body_10_1` 解析内联 FCN。实测断言：包装 v1、FCN v2、首选择子 1、DatasetName 新对象 FeatureDatasetName 键 44/v1、WorkspaceName 新对象键 45/v1、空 PropertySet、三个对象无停止点、第二选择子 0、最终绝对偏移 `0x510`，表内恰好为 43/44/45，未预登记任何键。`cargo test --test africa_local`：1 通过、0 失败。没有把流内字符串输出。

隔离构建是必要的：并行修改中的 `legend_format.rs` 无法编译；副本仅将该无关文件恢复为 HEAD，测试时 FeatureLayer、dataset_name、dataset_name_more、workspace_name 均与当时工作区源码逐字节一致。该结果验证从选定对象头起的局部对象链，空表是显式测试前提，不证明实际整流会话在该处也拥有空表，也不声称根对象遍历或 ArcMap 运行验证。按 TODO 246 的“解析选择子”范围，可建议主代理勾选，根遍历及软件验收另记。

`esriCarto.olb` 10.1 原件 SHA-256 `b95e4a043c5776c41cee5601be0400a77a61cb1f4475b8bd2a5c43f4893cc51c`；所使用的嵌入 MSFT 类型库长度 1,460,724，SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`，与原 OLB 中 `0x21af8` 处完整连续字节逐字节相同。使用 Wine 的 `LoadTypeLibEx`、`ITypeLib::GetTypeInfo`、`ITypeInfo::GetVarDesc` 读取类型 #390 GUID `{1558D2D3-8D39-11D1-B981-080009EE4E51}`，四个 `VAR_CONST` 均为 VT_I4：

| 10.1 类型库成员 | 数值 |
| --- | ---: |
| esriHyperlinkTypeDocument | 0 |
| esriHyperlinkTypeURL | 1 |
| esriHyperlinkTypeMacro | 2 |
| esriHyperlinkTypeScript | 3 |

一次性导出器为 `temp/dataset-format-media-20261117/memberdump.c`。该类型库枚举值足以支持原行 225；不证明 Hyperlink Load 的 int32 对应哪个接口属性，原行 226–227 仍开放。

### TopologyName 槽的引用类

FeatureClassName 的第二选择子非零时，GdbCore.dll 10.1.0.3035 的 Load `0x709b5080` 请求
ITopologyName。SystemCore 同版在 `0x7ffd0063` 按键查询登记实例的接口。
构造测试确认：头部类标识与登记类不一致时，以登记类判定；误引
WorkspaceName 时保留该槽及其后字节，已有对象键不变。
仍缺第二选择子非零的真实工程和 ArcMap 10.1 打开验证。
