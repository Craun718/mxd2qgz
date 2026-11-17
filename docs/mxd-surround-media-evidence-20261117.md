# 装饰物、书签与元素：2026-11-17 补充核查

范围：本轮 TODO 原行 172、211–214、218–219、240、250–251、253–255、257–259。未改 TODO 或共享分派文件。原生静态证据与参考样本详见 `mxd-surround-luna.md` 和 `mxd-parse-agent-sample-evidence-elements-20261117.md`；后者关于仅有参考样本的零命中不能推广到新发现的安装介质模板。

## 模板局部走查及限制

安装介质 66 份模板的提取与索引由主代理提供：`temp/arcmap101-media-mxd/`、`temp/arcmap101-media-streams/index.json`。`Africa.mxd` SHA-256 `ee6be21c409285644e81177a19878f5149cd2f3380b95f188fdfa5788d466674`，`/Version` 文本 10.1、short 19（程序识别候选，不等同运行验证）。`Utilities.mxd` SHA-256 `9c439b8d6624325b6b1a2e1199a7396772b6e5249c61f1076025e35323bd0eb6`，文本 10.0、short 18。

以类 GUID 扫描仅定位候选，再调用实际对象读取器；测试人为将对象键 0 至候选键预登记，以模拟之前流已读的键。这个近似没有重建跨流的实际登记表，也没有从 Map 起点走到目标对象，所以局部结果不是完整对象链或循环边界验证，尤其不能用它证明回引的先前对象身份。诊断输出 `temp/surround-media-check/probe.txt` 只供本地复核，不纳入版本控制。

- Africa `/Maps:0x9683` 的 Legend 候选，键 418：读取器消费到 `0xb4a3`、版本 5、共同前缀版本 3，得到 LegendFormat 与三个 HorizontalLegendItem，结束位置恰为 MarkerNorthArrow GUID 候选。条目候选 `0x9a69` 键 428 读至 `0xa121` 下个条目候选；其内 LegendClassFormat `0x9dab` 键 437 版本 2 读至 `0xa0f5`，两 TextSymbol 体完整、补丁槽为空、两个 double 为零。按相同假定，Legend 整体读取也消费这三个条目。连续嵌套结构与静态字段序相符，但需从真实流起点重建登记表和外层归属后才可判定非空 MapSurround 循环闭合。
- Africa `/Maps:0xb53b` MarkerElement 候选键 487：读取到版本 3、ElementProperties 版本 6，类型字符串 Marker，然后在 IMarkerSymbol 新对象键 488 处停止。该候选位于上述 Legend 终点后的 MarkerNorthArrow 区域，外层 MarkerNorthArrow 仍需逐层读取来确认归属。MarkerElement 符号槽目前只分类，后续几何和尾段未验证。
- Utilities `/PageLayout:0x20cf` LineElement GUID 候选键 254：以同一假定读取到版本 3、ElementProperties 版本 6、类型字符串 Line，在 ILineSymbol 新对象键 255 处停止。外层布局对象和此项的容器归属未走查，不能将扫描命中当作完整实例验证；该模板是 10.0 版本候选。
- 66 份 `/Maps` 和 `/PageLayout` 的固定 GUID 检索未发现 AOIBookmark 或 FeatureBookmark 候选；该有界负结果不证明所有书签对象不存在。各格式候选分布见忽略目录 `temp/surround-media-check/hits.json`。

## 实现与测试

`legend_format.rs` 的两个 ITextSymbol 槽对精确 TextSymbol CLSID 调用已有 10.1 文字体读取器，成功后继续读取；其内部停止向上报告并保留符号体中的详细停止原因。未知类仍在槽头停止；截断时游标和登记表回滚。新独立合成测试覆盖完整体、内部未知类停止和逐前缀截断。Africa 局部走查实际经过两个非空 TextSymbol，但不是 ArcMap 运行验证，也未证明完整文档的外层对象链。

## 原 TODO 行号状态与缺口

| 行 | 状态和缺口 |
|---|---|
| 172 | 部分：FeatureBookmark 的 IName 请求后 QI 已有静态证据；缺具体实现类及可访问的含 FeatureBookmark 样本或受控 COM 运行。 |
| 211 | 部分：格式体及两个 double 的布局确定，缺成员 getter/setter 与字段偏移对应证据；Africa 有局部版本 2 结构。 |
| 212 | 部分：HorizontalLegendItem 标量字段布局确定；Africa 有局部版本 1 结构，缺成员级映射。 |
| 213 | 部分：Legend 数个 short/int32 仍缺成员映射；Africa 有版本 5 局部结构。 |
| 214 | 部分：类型字符串与对象已读位置相符；名称、类型、自定义属性和参考比例仍需 getter/setter 偏移互证。 |
| 218 | 部分：名称和 IMap 槽可定性，short 与 32 字节块业务语义缺证据。 |
| 219 | 部分：AOI/Feature 名称、接口槽已知；FeatureBookmark 整数与 IName 动态对象关系需运行或 getter 偏移。 |
| 240 | 部分：族体字段序已有静态证据；标量映射及 ScaleLine 三个调用者仍需独立反编译和成员偏移互证。 |
| 250 | 部分：Africa Legend 局部完整嵌套；须从 Map 起点和真实登记表证实循环计数、对象所属与连续边界。 |
| 251 | 阻塞：66 个模板未命中书签 CLSID 候选；缺含 AOI/Feature 书签的真实样本或 ArcMap 受控保存。 |
| 253 | 部分：Africa 格式体局部读取完整，仍需真实键登记表和从外层连贯追踪。 |
| 254–255 | 部分：Africa 条目版本 1、基类版本 3 局部读取完整，仍需整流键状态与外层连贯追踪。 |
| 257 | 部分：版本 0/1 有静态分支与合成覆盖；需要可确认的旧版本实例或运行取证。 |
| 258 | 部分：Africa MarkerElement 候选读到符号槽；须接入符号实现并从外层确认几何与尾段。 |
| 259 | 部分：Utilities LineElement 候选读到符号槽；须接入符号实现并从布局外层确认几何与尾段。 |

本机命令可发现 Wine 和 QGIS，搜索默认 Wine 前缀未发现 ArcMap.exe；尚未在 ArcMap 10.1 或 QGIS 打开模板或转换结果。完整 `cargo test` 在现有 `composite_graphics` 的三个测试失败，输出涉及既有并行图层改动，不能归因于本轮改动；需主代理在整合工作区重新复核。

## 10.8 介质的交叉核对

主代理提取的 `temp/arcmap108-media-streams/index.json` 显示其中 `africa.mxd.2319C344_B117_4BE0_8B98_93D6E4E94502` 的文件 SHA-256 与上文 Africa.mxd 完全相同，两个 `/Maps` 流摘要亦相同；其 `/Version` 为 10.1/19 候选。因此这是同一文档的重复来源，不是额外独立样本，也不增加运行验证。未按 10.8 介质推断该文档为 10.8。

## 点、线元素符号槽后续接线

基于 GraphicElements.dll 10.1.0.3035 MarkerElement Load `0x7504ea50` 和 LineElement Load `0x7504a100` 的 IMarkerSymbol/ILineSymbol 请求，以及 DisplayCore 既有类体记录，点符号槽仅对 SimpleMarker、MultiLayerMarker、CharacterMarker、PictureMarker、ArrowMarker 新对象调用现有全符号读取器；线符号槽仅对 SimpleLine、MarkerLine、CartographicLine、HashLine、MultiLayerLine、PictureLine 新对象调用。空标记和回引沿用原先分类规则；其余新类停在对象头且不登记，已知体的嵌套停止保留内层游标和原因，截断整体回滚。符号槽字段类型现为 `SymbolSlotFull10_1`，调用方若直接匹配字段需适配。

| 介质、流与候选 | 局部读取结果（对象头含类和键） |
|---|---|
| 10.1 介质 Africa，`/Maps:0xb53b` | CharacterMarkerSymbol 后读到 Point 和版本 3 尾段，结束 `0xb6c3`。 |
| 10.1 介质 Utilities，`/PageLayout:0x20cf` | SimpleLineSymbol 后读到 Polyline 和尾段，结束 `0x21ec`；文件 `/Version` 10.0/18 候选。 |
| 10.8 介质 Asia，`/Maps:0xae25` | CharacterMarkerSymbol、Point 和尾段，结束 `0xafad`；文件 SHA-256 `314520b11e8ccf4badafb3fbaca978d2aff516fce539e0195cd53b5526f32b8b`，`/Version` 10.1/19 候选。 |
| 10.8 介质 CentralUSA，`/PageLayout:0xa0c` | SimpleLineSymbol、Polyline 和尾段，结束 `0xb29`；文件 SHA-256 `b3b594837ab29c9435c2214793d1dcc65d2985a73d26b6619d83704f99005bac`，`/Version` 10.1/19 候选。 |

以上探针均假定候选对象键及之前的键已登记，并不是从流起点重建的状态；不据此确定空间参考回引的实际对象身份或布局容器归属。默认合成测试覆盖正确点/线符号、错误接口新类、符号内部未知颜色、逐前缀截断及引用表回滚；真实介质测试为 ignored，需要显式 `cargo test --test element_body media_shape_candidates -- --ignored`，缺文件即失败。

共享接线留给主代理：`map_surround_family::read_typed_slot_10_1` 当前将已知对象体返回的内部停止当作外层无停止；MarkerNorthArrow 的 MarkerElement 若在内部符号处停止，需要将其停止原因向外传播，避免上层误认为整个装饰物已完成。不得从本轮独立元素读取通过推断该共享路径已闭合。TODO 原行 258、259 因整流归属和引用表未复核仍保持部分完成。
