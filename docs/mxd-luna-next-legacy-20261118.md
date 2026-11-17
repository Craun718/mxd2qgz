# Luna 旧档边界续核：Hyperlink、GroupLayer、Envelope（2026-11-18）

本轮按 `/Maps` 根与宿主顺序复核三项旧版对象候选，并对照 ArcMap 10.1 Desktop/SDK 与另一代介质。所有原生实现结论来自版本标明的静态反编译；没有 ArcMap 运行环境，本轮未运行 ArcMap。类标识扫描、局部字节形态和合成测试均不单独作为实档实例证据。

## 样本与原生文件身份

- 10.1 `FeatureLayer.dll`：提取文件 SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，FileVersion 10.1.0.3035。现有反编译导出中 `FeatureLayer::Load 0x6f6e9940`、`FeatureHyperlink::Load 0x6f6d9170`、`FeatureHyperlink::Save 0x6f6d9210`、`GroupLayer::Load 0x6f711270` 和 `GroupLayer::Save 0x6f7119b0` 均有伪代码；详细记录见 [Hyperlink 族体](arcmap-10.1-hyperlink.md)、[FeatureLayer 后继段](arcmap-10.1-featurelayer-tail.md)及 [Map/GroupLayer 研究](arcmap-10.1-map-object.md)。模块导出限制、版本身份边界见[研究成果指南](arcmap-10.1-research-source.md)。
- 10.1 `Geometry.dll`：SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`，FileVersion 10.1.0.3035。`Envelope::GetClassID 0x71864170`、`Load 0x71869b60`、`Save 0x71869da0` 的已有反编译与调用证据见[空间参考记录](arcmap-10.1-spatial-reference.md)。
- 实档输入包括仓库参考 `samples/测试底图.mxd`、10.1 SDK Java `brazil.mxd`、10.1 SDK .NET `Continents.mxd`，以及另一代介质中的 `calibrate.mxd` 和已记录的模板批次。文件 SHA-256 与 `/Version`、来源限制列在下文及[第八轮介质记录](mxd-terra-round8-old.md)。介质来源不等于文件保存版本。

## Hyperlink 版本与宿主槽

10.1 静态证据能闭合读取器与宿主槽，但本轮没有取得真实 Hyperlink 对象体：

1. `FeatureLayer::Load 0x6f6e9940` 在后继段读取标量后读 int32 Hyperlink 计数，循环以 `LoadObject` 请求 IID `{3036D35D-…}`（IHyperlink）；`FeatureLayer::Save 0x6f6eb1b0` 在同一段写计数并逐项 `SaveObject`。循环之后才读 IColor 槽。由调用次序可确认这是 FeatureLayer 持有的 Hyperlink 集合槽。
2. IID `{3036D35D-…}` 是请求接口；新对象类身份另由 `FeatureHyperlink::GetClassID 0x6f6d9310` 写出的 CLSID `{3036D35E-…}` 确认。对象体 `Load 0x6f6d9170` 先读 int16 版本，再读 int32、AfCore String、int32；有符号版本门允许 0 和 1、拒绝大于 1。`Save 0x6f6d9210` 恒写版本 1。因此 v0 是 10.1 静态读侧兼容分支，不是 10.1 写侧格式证据。
3. 新增独立测试从参考 MXD `/Maps` 根计数、Map 对象头开始，沿已解析图层列表到 FeatureLayer 后继段。它实际到达偏移 `0x2d40`/键 158 与 `0x42bc`/键 218 的 Hyperlink 宿主计数，均为 0。两处不含 Hyperlink 实例，也不证明未到达的尾部或其它流中不存在实例。
4. 10.8 介质 68 个 `/Maps` 的已有候选筛查未见 Hyperlink CLSID；该批扫描没有逐个从根走完宿主链，不能据此断言 10.8 的版本门、字段序或实例分布。

因此可记录 10.1 的**宿主槽位置、请求 IID/实例 CLSID 区别及 v0/v1 读侧规则**；Hyperlink v0 实档体、空槽/回引的 Hyperlink 专属形态，以及 10.8 对应实现仍未证明。已有[合成 Hyperlink 测试](../tests/hyperlink.rs)覆盖 v0、v1、越界和截断；本轮新增的宿主测试证明参考样本的两个计数为零，不把该结果升级为对象体验证。

## 旧版 GroupLayer（对象版本 <11）

10.1 `GroupLayer::Load 0x6f711270` 的静态分支明确：读取组图层自身版本和前置字段、子图层数后，**对象版本 `<11` 的子项调用普通 `LoadObject`，版本 `>=11` 改走 `ExtraSafeReadObject`**。GroupLayer 的子项请求 ILayer；实际类仍由子对象流头确定。此门是对象体版本门，不能与 Save 侧按文档版本选择普通/ExtraSafe 写出方式混为一谈。

真实宿主检查目前只到达新分支：

- 10.1 SDK Java `brazil.mxd`（SHA-256 `26ae235ddcbbd8a501e6029d53a9ddec073382d87f71fb25067625ed8f52ba30`，`/Version` 9.2/14）从 `/Maps` 根、Map 到图层列表，顺序到达 GroupLayer 偏移 `0x15a`、`0x1f6`、`0x8d70`；版本均为 11，子层计数分别 0、7、13。可见子项走 ExtraSafe 路径。
- 10.1 SDK .NET `SchematicsForServerSample.mxd`（SHA-256 `31ae0b799e3783f5bbc174e89a4a0840795fcf1f68d0d7600254c89e86898f56`，`/Version` 10.0/18）顺序到达 GroupLayer `0x162` 与 `0xb234`，版本均为 12，子层计数 7、1。
- 另一代介质中的 8.1 `calibrate.mxd`（SHA-256 `f883510f90d05a01c4ae152aa008a74b14cab0e8469177b98ef7e3b40fc533b4`）从 `/Maps` 根到 Map 架构版本 19；旧的两个 Map 图层集合计数均为 0，活动索引为 -1，随后到 CompositeGraphicsLayer 头。该样本没有 GroupLayer 实例。该结论不适用于未遍历的其他 Map。

因此旧版普通 `LoadObject` 分支有静态规则与合成 v10 边界测试，但**没有真实 `<11` GroupLayer 实例字节**。SDK v11/v12 样本验证的是另一条 ExtraSafe 分支，8.1 样本的空集合不能补足旧组图层对象。独立合成覆盖见[第九轮旧版测试](../tests/terra_round9_old.rs)与[GroupLayer 测试](../tests/group_layer.rs)。

## Envelope v0

10.1 `Envelope::Load 0x71869b60` 中版本 0 与版本 2 进入同一坐标/标志读取路径，版本恰为 1 有单独短分支，只有版本恰为 3 才继续读取空间参考对象。由 `Save 0x71869da0` 和 WKSEnvelope 访问器对位，v0 体序列为：`int16 version=0`、4 个小端 double `[XMin,YMin,XMax,YMax]`、int32 标志、依 bit 6/7 出现的 Z/M 两对 double。没有空间参考子对象或已见填充；四种 Z/M 组合体长为 38、54、54、70 字节。Save 恒写 v3，所以不能由 10.1 写侧得到 v0 实档。

跨代核对没有找到真实 v0：10.8 介质的 Map 宿主顺序样本中，已到达的 18 个 Map IEnvelope 均为 v3；8.1 `calibrate.mxd` 的旧 Map 路径没有走到 Envelope。10.1 SDK .NET `Continents.mxd` 的 `/ESRI ArcToolbox` 流两处 Envelope CLSID 偏移 `0x306a6`、`0x5b73c` 未建立宿主、引用表或对象边界。顺序字节更符合 `+16` 处版本 3、其后四坐标和标志；`+20` 的零是错位读取，不能算版本 0。

故 v0 布局已有 10.1 静态证据和合成边界覆盖，但**没有从 MXD 根/宿主顺序到达且确认对象终点的 Envelope v0 实例**。现有[Envelope v0 测试](../tests/envelope_v0_round2.rs)覆盖标志组合与截断回滚，另有[旧档边界测试](../tests/terra_round9_old.rs)覆盖无 Z/M 体；测试不是实档证据。

## 验证与范围

本轮新增 `tests/luna_next_legacy_20261118.rs`：从参考样本根顺序到达两个 FeatureLayer Hyperlink 计数（均 0）；另从 10.1 SDK Java/.NET 的 `/Maps` 根顺序到达 Brazil 的三个 v11 和 Schematics 的两个 v12 GroupLayer，并检查其子项数量与解析条目数相符。以下定向测试通过：

```text
cargo test --offline --test luna_next_legacy_20261118 --test envelope_v0_round2 --test hyperlink --test terra_round9_old -- --include-ignored
```

共 14 项通过，包括已提取 SDK/旧介质输入上的旧 Map 空列表与 Envelope 错位候选检查。另单独运行 `terra_depth_media` 的提取介质测试，1 项通过。`rustfmt --edition 2024 tests/luna_next_legacy_20261118.rs` 通过。全仓 `cargo fmt --check` 未通过：共享工作树另有 `tests/luna_cross_stream_20261118.rs`、`src/maps/map_persist_collections.rs`、`tests/map_persist_collections_20261118.rs` 的既有格式差异，本轮未改动这些文件。单独运行 `terra_round8_old -- --include-ignored` 时，旧 GroupLayer 检查已先通过，但同一测试后续在 LinearRef 的 `AfterBorderTail { stage: 39 }` 断言失败；新建的 GroupLayer 根遍历测试独立通过，未修改该既有断言。

没有修改生产代码或 `TODO.md`，没有启动 ArcMap，也没有进行 ArcMap/QGIS 人工打开检查。样本根遍历只证明测试实际到达的对象和边界；其它未遍历流、未达 Map 或未知体仍不在本轮结论范围内。
