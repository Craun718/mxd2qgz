# Maps 组复核与实现记录（2026-11-18）

范围：TODO 原行号 137、198、238、244–245、267、278、285、329–332。依照本轮约束未修改 TODO、README、lib.rs/mod.rs，也未更改 `src/layers`。

## 证据与方法

优先检查本机 ArcMap 10.1 反编译成果及仓库现有研究记录。直接核对 Map.dll 导出 `functions.tsv` 中 `0x793ff4d0`（Map 持久化 Load）、`0x793da250`（Legend Load）、`0x793b5c50`（FeatureBookmark Load）；对应伪代码为 `Map/793ff4d0.c`、`Map/793da250.c`、`Map/793b5c50.c`。模块身份及版本依据 `docs/arcmap-10.1-map-object.md`：Map.dll，10.1.0.3035，静态映像地址。本轮没有运行 ArcMap；REA 定向尝试及边界见下一段。Map::Load 伪代码明确按版本字段、名称、对象槽和计数循环顺序读流；FeatureBookmark 伪代码确认 short、int32、IName 对象及字符串的字段顺序。反编译伪代码不是原始源码，单元测试与合成流不能证明目标软件行为。

本机已有成果使用指南为 `docs/arcmap-10.1-research-source.md`；本轮所需函数均有 `ok` 导出记录。REA 6.9.0 定向调用 Map.dll `ram:0x793ff4d0`：CLI 环境检查确认 Ghidra 12.2/JDK 25 可用；首轮设定 60 秒上限后取消，诊断显示已启动 headless 并使用 Unix-socket bridge；第二轮 120 秒后 headless 在 bridge 启动前退出。完整 stdout/stderr、REA JSON 与 Ghidra 日志保存在本轮隔离临时目录；无 REA 成功伪代码产物，本轮结论继续依据已存在、`ok` 状态的 10.1 导出与原始 MXD 字节，不声称 REA 导出成功。Map::Save `0x79401850` 的现有伪代码也已检查，无需以失败的 REA 尝试替代或重复已有证据。

## 逐项结论

| TODO 原行号 | 状态 | 结论与边界 |
| --- | --- | --- |
| 137 | 部分 | Map 前缀、图层列表、MapSurround、WKSEnvelope、IEnvelope、书签、独立表及照明后已确认字段已有读取器并接入 Map 后缀。本轮修正 AOI/FeatureBookmark 两个 CLSID 的磁盘字节序，10.1 写侧 GetClassID 声明与 9.3 辅助档书签头现一致；NADCONTransformation 的有限类体/循环子集也已有实现。仍缺其他坐标变换类、独立表未支持嵌套类、书签后成员业务语义、照明后 12 字节语义、非空 IGeometry 和其他边框/背景实现；此复合条目不能整体勾选。主要证据/实现见 `docs/arcmap-10.1-map-object.md`、`docs/mxd-terra-chain-map.md`、`src/maps/map_object.rs`、`src/maps/standalone_table.rs`。 |
| 198 | 部分 | FeatureBookmark IName 槽已接 FeatureClassName 与 TableName 的保守读取子集；其余具体 IName 类及实际 Open/QI 结果未闭合。旧版辅助档中发现的非空项均为 AOIBookmark，没有 FeatureBookmark IName 新类分布佐证；仓库样本书签数为 0。详见 `docs/mxd-terra-round9-bookmark.md` 与本记录的介质复核。 |
| 238 | 待验证 | HorizontalLegendItem 的包装 int32 已确认与流向变更和布局计算相关，但 0..5 各值没有完整名称或业务映射。不能定性所有值。详见 `docs/mxd-terra-chain-legend.md`。 |
| 244 | 待验证 | MapSurround 共同前缀的字段顺序及读取行为已有静态证据和读取器；其中额外 short、32 字节块与若干标量的业务语义仍未知。样本 MapSurround 计数为 0。详见 `docs/arcmap-10.1-mapsurround-bookmark.md`。 |
| 245 | 待验证 | 书签共同前缀字段布局已有静态证据；旧档补足了非空 AOIBookmark 循环，未补足 FeatureBookmark 标量的业务含义。详见 `docs/arcmap-10.1-mapsurround-bookmark.md`。 |
| 267 | 待验证 | 指定装饰物/图例/patch 类的结构读取子集已有实现，标量字段业务语义仍未对位；合成输入不构成语义或目标运行验证。详见 `docs/arcmap-10.1-legend-family.md`、`docs/mxd-terra-depth-legend.md` 及 `src/maps/map_surround_family.rs`。 |
| 278 | 部分 | 10.1 安装介质 63 个 MXD + 3 个 MXT 均可解析；48 份可到达书签计数的文件全为 0，另 18 份在此前停止。旧版辅助档 68 份中，49 份可到达循环，18 份声明非空，共 133 项 AOIBookmark；修正 CLSID 后所有 133 项均经版本/Envelope/名称体解析并越过循环，没有 `BookmarkBody` 停止。真实非空循环已得到旧档正面证据，仍须区分目标版本/运行验收，不能代替 ArcMap 10.1 实档验证。 |
| 285 | 待验证 | 10.1 安装介质可定界读取的 28 个 Legend 主体版本为 v4（12 个）和 v5（16 个）；旧版辅助档 62 个为 v4（38 个）和 v5（24 个）。两批均无 v0/v1 字节实例；Legend v0/v1 分支仍只能由静态规则和合成测试覆盖。详见 `docs/arcmap-10.1-mapsurround-bookmark.md`。 |
| 329 | 部分 | `parse` 命令已调用 `mxd_maps::parse_maps_10_1`，首个 Map 的已支持对象体经读取器接入中间模型/诊断，未解析尾部保留为未知字段；完整 Map 对象体及所有对象体仍未覆盖。CLI 接线位于 `src/main.rs`，解析接线位于 `src/maps/mxd_maps.rs`。 |
| 330 | 待验证 | 当前只构造首个数据框。10.1/旧版辅助批次中分别有 0/12 份多数据框文档；旧版批次的 12 份均只解析首框。根计数是 Map 数，但 Map 对象体之间没有外部长度字段；现有 Map 后缀读取器停止点不能证明就是下一个 Map 对象头。PageLayout 文档根读取器 `0x7b708d00` 证明原生有根循环，`0x793ff4d0` 只读取单个 Map 对象；仅凭现有局部解析器不能安全接续多对象。 |
| 331 | 部分 | 图层列表条目按顺序接入 `DataFrame.children`；已解析回引复用同键节点，GroupLayer 子层递归构建树。未闭合/未分派体保留为未知并停止，绘制顺序仍属候选解释。代码位于 `src/maps/mxd_maps.rs` 与 `src/maps/map_object.rs`。未做 ArcMap/QGIS 目视验收。 |
| 332 | 部分 | 已确认名称、可见性、数据源连接、Map CRS 和 WKSEnvelope 范围接入模型；部分原始属性仅进诊断，未知字段未强行命名。独立表、装饰物、书签等未全部映射为业务模型属性，且部分语义未闭合。未做 ArcMap/QGIS 目视验收。 |

## 批量介质复核与修复

只读调用仓库现有 `target/debug/mxd2qgz parse` 并留存每份输出。ArcMap 10.1 安装介质资料 63 个 `.mxd`、3 个 `.mxt` 均退出码 0；48 份到达书签计数，全部为 0，另 18 份在书签阶段前遇未闭合 MapSurround/图层体而停。28 个可解析 Legend 对象头之后的主体版本 short 由流偏移直接抽读：v4=12、v5=16。没有发现 Legend v0/v1。

辅助档批次取自另一套安装介质，共 68 个文件，均退出码 0；`/Version` 文本分布为 8.1（1）、9.2（12）、9.3（18）、10.0（11）、10.1（26）。49 份到达书签循环，其中 31 份计数 0，18 份计数非零（合计 133，全部来自 9.3 候选）；未到达的 19 份不能推断书签数。非空书签对象头的实际字节是 `73 ae b1 bb e3 41 d2 11 ae 1e 08 00 09 ec 73 2a`，对应 AOIBookmark `{BBB1AE73-41E3-11D2-AE1E-080009EC732A}` 的 Windows GUID 磁盘序。原常量把 Data2/Data3 片段按 UUID 文本顺序排列，未命中该实档 CLSID，因此 133 个体此前均停在对象头。FeatureBookmark 常量同样有 Data3 顺序错误，按其 `{EC65B35B-4342-11D2-AE22-080009EC732A}` 修正为磁盘序。修正后，133 个 AOIBookmark 均被解析为 v1、嵌套 Envelope v3/引用或空值与名称字符串，并顺利到达计数循环之后；这证明该批旧版流上的读取器边界与条目体可以工作，不证明 ArcMap 10.1 真实分布。

辅助档 62 个可定界 Legend 主体均为 v4/v5（v4=38、v5=24），无 v0/v1。共 12 份报告 Map 计数大于 1，均仍仅解析首框；因此多 Map 直接接线仍开放。

Map::Save `0x79401850` 已在本机 `Map/79401850.c` 静态导出并交叉核对 Map::Load `0x793ff4d0`：在 `+0x170` 书签数组的计数循环之后，Save 将局部 `asStack_4c[0]` 设为 `-1` 并写出 2 字节；紧接着写 `+0x19c` 的 short。Load 在对应位置读 2 字节到仅被此处写入的栈 `local_5c`，随后调用 Map 子接口虚表 `+0x44`，再读入 `+0x19c`。这定性为书签循环后的固定 `0xffff` 流字段及其消费位置；字段的业务名称、该调用是否使用其值仍未证实，不能将它映射成命名属性。Map::Save 同时还在后续字段保存前写出 `+0x19c`、double `+0x200`、short `+0x264`、独立表计数/循环等。之后坐标变换及尾段仍开放。本轮新增安全实现是 Legend 主体版本诊断接入 parse，以及修正两个书签 CLSID 磁盘序；正常/异常合成测试覆盖 v1/v5 Legend 主体、非空 AOI 书签通过、书签名称截断时停在体起点并保留未知尾部。

## 验证状态

最终共享工作区中 `cargo fmt --check` 通过；`cargo test --offline --lib maps::mxd_maps::tests::` 的 21 项测试通过（含 Legend v1/v5、非空 AOI 及截断异常测试）；随后 `cargo test --offline` 全套通过。主审核随后报告全仓 827 项通过，最终 CLI 复跑确认 18 份 9.3 候选中的 133 个非空书签均消费；这是主审核提供的复核结果。完整 ArcMap/QGIS 人工验收未做。
