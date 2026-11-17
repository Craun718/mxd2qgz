# 介质深层宿主：Basemap 内嵌 Map（2026-11-17）

本轮继续 [首 Map 介质遍历](mxd-terra-follow-media.md) 的未覆盖宿主，复用 [Basemap 原生调用证据](mxd-round3-basemap-20261117.md)，不改产品代码和 TODO。以下是使用现有 10.1 读取规则对真实文件的结构化解释，不是 ArcMap 运行观察。外层 ExtraSafe 条目的 B=0，条目末界依赖 A 前缀的已有观察推定；未知区域中未登记的对象仍可能影响之后的回引分类。

## 新到达的宿主

10.1 安装介质 `temp/arcmap101-media-mxd/Africa.mxd` 文件 SHA-256 `ee6be21c409285644e81177a19878f5149cd2f3380b95f188fdfa5788d466674`；`/Version` 读取为文本 `10.1`、short 19、对象计数 637，保存索引 5 仅为静态候选。介质及头部均不能证明实际写出程序的运行版本。独立测试从 `/Maps` 根顺序建立引用表，进入 Map → BasemapLayer 键 40 → 版本 2 的第一条 PrepareResult → 列表首项 FeatureLayer 键 42；非从 GUID 搜索开始。该 FeatureLayer 在 `0x2ae` 对象头起，体完整消费至 `0x145d`：

- FieldInfo 声明及完整新对象均为 6 项，键 61、62、63、65、66、68，均为 v4。由读取器的宿主循环和完整体结果归属，不是 v0–2 证据。
- FeatureLayer 后继的第二 IEnvelope 槽读到完整 Envelope v3；第一个 IEnvelope 槽为空。这扩大了前轮对直接图层与 Map 槽的覆盖，但不证明 Envelope v0 或其他未到达宿主不存在。
- 测试独立重走 Basemap 前缀并核对键 42 体结束后，在 `0x145d` 到达第二列表对象 Map 新对象键 91；这是 **Basemap 准备结果内嵌 Map**，不是 `/Maps` 根循环的第二个 Map。

内嵌 Map 的对象头后，版本 51、两个字符串后，空间参考对象头是已登记的 PCS 键 39 回引，必须沿用同一引用表。测试直接按已证前缀顺序分类，未调用会误读该回引为新对象的 `read_map_suffix_10_1`。随后图层列表从 `0x14b9` 到 `0x14c1`：声明计数 0、活动索引 -1；基本图形层是 CompositeGraphicsLayer 新对象键 92，已确认体消费至 `0x1581`；MapSurround 计数为 0；32 字节范围和两个 int32 后的 IEnvelope 槽在 `0x15ad..0x15bd` 是空标记。测试在此停止，**未证明内嵌 Map 体结束，也未到达后续 PrepareResult**。由此仅能说这个内嵌 Map 已到达的图层列表没有 GroupLayer，不能推广到整个 Basemap 或其他 Map。

## 尚未闭合的边界

- GroupLayer 版本分支：此次到达的嵌套 Map 图层列表为零，未取得 GroupLayer 字节，旧版本仍开放。
- FieldInfo v0–2：新宿主只得到 v4，不闭合旧分支。
- Envelope v0：新宿主为 v3，内嵌 Map 的对应槽为空，不闭合 v0。
- 多根 Map：前轮 20 份声明多个根 Map 的文件，其首对象仍有未闭合尾段。本轮没有跨越该边界，不能把内嵌 Map 当第二个根 Map；后续需要在共享引用表下闭合首 Map 对象体，或从外部真实边界与完整对象表获得可信起点。读取器目前将 Map 空间参考回引当新对象体也是复用该入口的障碍。

既有 10.1 原生版本门证据参见 [图层](mxd-layer-luna.md)、[FieldInfo](mxd-round4-layers-20261117.md) 和 [Envelope](mxd-round2-spatial-20261117.md)。本轮源码和已有静态证据足以确定本次可追踪的宿主顺序，未新启动 Ghidra 桥接工具 或切换共享目标，也未将静态规则当实机证据。

## 复现与验证

独立测试 `tests/terra_depth_media.rs` 显式依赖本地介质，默认忽略：`TMPDIR="$PWD/temp/terra-depth-media/tmp" cargo test --offline --test terra_depth_media -- --ignored`。测试验证引用表连续性、6 个 FieldInfo 的版本、Envelope 槽及嵌套 Map 到空 Envelope 的偏移与停止边界；断言只验证本文件在现有读取器规则下的解释，不验证普遍格式兼容性。未在 ArcMap 10.1 或 QGIS 中打开文件，也未进行目视或运行验证。

验证结果：以上介质依赖测试显式运行 1 项通过；`cargo test --offline` 全仓 739 项通过、11 项默认忽略；本次独立测试文件的 `rustfmt --check` 通过。全仓 `cargo fmt --check` 当时报告其他并行修改中的 `src/layers/annotate_props.rs` 和 `tests/terra_depth_label.rs` 存在格式差异，本轮未触碰这些文件。
