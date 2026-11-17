# MXD 解析第二轮复核（2026-11-17）

六个 `gpt-6.1-terra` 子代理并发覆盖本轮开始时的 85 个未勾选项。主审复核原生证据和实样链路后，在 TODO 中逐项勾选 7 项。剩余 78 项另见[运行取证、补样本及静态续查清单](mxd-round2-needs-20261117.md)：10 项要求目标运行，20 项要求对应的真实样本，48 项仍可沿现有材料继续研究。

## 本轮勾选

- 原 228、229：10.1 类型库区分 FeatureId、Link 和 LinkType；Layer.dll 的访问器再分别绑定三个成员。对应流字段为 ② FeatureId、③ Link、④ LinkType。只改归属说明，保持现有 Rust 字段名和读取布局。
- 原 252、261：10.1 安装介质 `LandscapeClassic.mxd` 的 Map 架构版本为 45。以实际 Map 对象头开始登记引用，连续消费 3 条 MapSurround：第二条 `MarkerNorthArrow`（`/Maps:0x885`）内联的 MarkerElement 体完整消费；第三条 Legend 位于 `0xa78`，其后到达 `0xef4` 的 WKSEnvelope。完整文件 SHA-256 为 `8e06e2ea0cd1e3392673404c22e3932c4529295b06b8a51f3173985022bdc469`。
- 原 256、257、258：10.8 安装介质附带的 `africa.mxt` **文件本身为 9.3**（`/Version` 的文本与 short 为 `9.3`/16），Map 架构版本 31。由 Map 根的真实引用表连续读取，第三条装饰物 Legend v4 位于 `/Maps:0x8b28`；三个 HorizontalLegendItem 的对象头分别位于 `0x8ede`、`0x956e`、`0x9e76`，自身和共享 BaseLegendItem 字段组版本均为 1。每条均内联并完整消费一个新 LegendClassFormat 体，随后通过 Legend 尾部，到达 `0xa892` 的 WKSEnvelope。完整文件 SHA-256 为 `39180b9d95cf1ade7fad3d4314b5157b9a58cfeced98ee54decb877659e8e884`。这是旧档字节证据，未验证 Legend 主体 v0/v1，也不是 10.8 保存结果。

可用 `cargo test --locked --test round2_legend_media -- --ignored` 复跑上述两份介质文件的读取，文件放置位置见测试。引用表从实际根对象开始登记，测试未预填其他键。

## 已实现，相关待办仍留开放

- Map 书签后的属性前导：10.1 Map.dll Load `0x793ff4d0` 与 Save `0x79401850` 从书签循环后对位。写侧在栈上初始化两个 short，但只写一个 `0xffff`，其后为 short `+0x19c`、double `+0x200`、short `+0x264` 和 int32 坐标变换数。参考样本的 `/Maps:0x24fa0..0x24fb2` 对位为 `0xffff, 0, 5000, 0, 0`；范围仍为 `0x24f64`，从 `0x24fb2` 保留 1467 个未知字节。读取器仅接受完整的已知前导；遇到截断、不同标记或不合法计数时保留原字节。成员业务含义、非空坐标变换和照明属性未实现。
- CompositeGraphicsLayer 元素组：按精确 CLSID 分派 TextElement、MarkerElement、LineElement，并透传内层停止位置。10.1 介质 `Africa.mxd` 的基本图形层读毕；随后图例需要键 42，前方跳过的 BasemapLayer 体尚未建立该键的归属。LineElement 已有局部真实字节和合成测试，尚未从原宿主走通真实引用表。
- Envelope v0：按 10.1 Geometry.dll Load 与 v2 相同的布局读取，无内嵌空间参考；保留真实 v0 旧档取证待办。
- 10.8 `/Version`：新增显式读取入口；有符号 short >25 拒绝，>11 读取并保留四字节，截断回滚。它不改变 10.1 识别逻辑或原 CLI 的候选分支；10.8 写侧回填与跨版本验证仍开放。
- 实样另确证两类嵌套符号和 StdPicture BMP 读界；剩余 DisplayCore 类和 StdPicture 第二字语义未因此勾选。

分组证据：[版本与对象流](mxd-round2-version-object-20261117.md)、[图层与标注](mxd-round2-layer-label-20261117.md)、[数据源及格式](mxd-round2-dataset-format-20261117.md)、[装饰与元素](mxd-round2-surround-elements-20261117.md)、[BasemapLayer](mxd-round2-basemap-20261117.md)、[符号与 OLE](mxd-round2-symbol-ole-20261117.md)、[空间对象](mxd-round2-spatial-20261117.md)。

`cargo fmt --check`、`git diff --check` 和全仓 `cargo test --locked` 通过（698 项通过，5 项默认忽略）；额外运行的 4 项本地介质测试全部通过。未在 ArcMap 10.1 或 QGIS 内打开工程进行人工验收，未验证转换结果。
