# 第四轮：Legend、装饰物、元素属性与书签

范围为本轮分派的 TODO 原行 217–220、224–225、246、257、264。不改 TODO、README、公共模块及 Map 根。以下状态针对原行完整内容；未人工打开 ArcMap 10.1 或 QGIS。

## 原生证据与方法

复核 `temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，与 [10.1 原生研究](mxd-open-legend-20261116.md)中的 PE x86、Esri Map DLL、文件和产品版本 10.1.0.3035 身份一致。本轮 Ghidra 桥接工具 / Ghidra 12.1.4 批量反编译证据 `ev_4d7eb5fab3f6287cfa9fb3d1dbe2a71d9b5b408d2202b175dc24907e260e0deb` 中，FeatureBookmark Load `0x793b5c50` 解析成功：short 版本、4 字节成员、LoadObject 和 String 的顺序与既有读取器一致；IName 对象若非空会再经虚表 +0x14 调用。此结果未证明 int32 的业务名。批量四项中两项失败，不能把失败结果当作新证据；三个 ScaleLine 调用者的既有反编译记录见 [第三轮](mxd-round3-line-element-20261117.md)及 [开放项核查](mxd-open-legend-20261116.md)，调用者身份随后已核定为 ImageInset、MapInset、Overview，并非 ScaleLine，见 [页面续查](mxd-terra-integrate-page.md)。Ghidra 桥接工具 结果为静态反编译而非原始源码或运行行为。

## 介质与环境查找

对既有 10.1 Desktop `Setup1.cab` 中提取的 66 份 MXD/MXT 的 `/Maps` 流逐文件按源码中的 AOIBookmark、FeatureBookmark 和 Legend 16 字节类标识查找：前两种均零命中；Legend 32 处候选，按对象头后 20 字节处读取的候选版本均为 4 或 5。候选搜索不能证明对象边界、循环归属或所有书签不存在；后者尤其不能据零命中排除其他书签类、其他流或提取范围之外的样本。现有可从 Map 根连续验证的非空 Legend/MapSurround 及对象引用条件见 [第二轮复核](mxd-round2-review-20261117.md)和 [第三轮复核](mxd-round3-review-20261117.md)。此查找未发现能够验证 Legend v0/v1 或非空书签循环的真实材料。

本机 `wine` 可执行，但 PATH 中未发现 `ArcMap.exe`，默认 Wine 的 Program Files 下未发现 ArcGIS 安装；这不是对所有挂载位置或其他主机的穷举，不能据此声称不存在可用目标环境。尚未建立可在 ArcMap 10.1 保存非空书签或旧版 Legend 的运行环境。需要真实旧文件或可运行目标环境，不能由构造字节替代。

## 原行状态

| 原行 | 状态 | 本轮证据及剩余工作 |
| --- | --- | --- |
| 217 | 部分 | [LegendClassFormat 读写](arcmap-10.1-legend-tails.md)已确认文本符号、补丁对象槽和两 double 的布局；仍需属性访问器成员偏移对位，两个 double 语义未确定。 |
| 218 | 部分 | HorizontalLegendItem 与 BaseLegendItem 已有结构读取及 Africa 实例；标量需接口访问器与持久化成员逐项对位。 |
| 219 | 部分 | Legend 版本门、字段序和集合选择 short 已定位；各 short/int32 业务语义尚未闭合。 |
| 220 | 部分 | ElementProperties 的序列和版本门已有 [原生证据](arcmap-10.1-element-bodies.md)；字段名需属性访问器对位。 |
| 224 | 部分 | MapSurround 的名称、IMap 槽、short 和 32 字节组有 Load/Save 证据；32 字节读后丢弃，不可仅凭长度命名为范围，short 语义未定。 |
| 225 | 部分 | AOI 与 Feature 书签的流内前缀不同；本轮 Ghidra 桥接工具 复核 Feature int32 读取位置，但未确定其业务名。IName 实现类与非空对象运行行为未闭合。 |
| 246 | 部分 | 三个共同前缀调用者已由先前工作反编译，原行“未反编译”已过时；它们的 coclass 及各装饰类标量的业务语义未逐项闭合。 |
| 257 | 阻塞 | 上述介质候选查找无已知书签类命中，参考样本书签计数为 0。需要从 Map 根顺序解析的真实非空书签或目标运行产物及真实引用状态。 |
| 264 | 阻塞 | 介质候选未发现 v0/v1；静态版本门已有读取器，本轮新增构造输入边界/截断测试，但它们不能验证真实旧版文件。需要真实旧版体及可重建的对象表。 |

## 代码、验证及接线

`tests/round4_page.rs` 针对现有 `read_legend_prefix_10_1` 的 v0/v1 构造体验证版本 1 才读取两个旧 double、后续 short/计数和末尾字节边界，并检查每个截断位置失败后游标和引用表保持；这只是解析器契约测试，不是旧样本证据。没有在缺少成员对位证据时修改字段语义或伪造未知值，也没有新增解析器或改共享接线。

如后续得到书签样本，父代理需从 Map 根及之前的跨流操作建立真实对象引用表，再调用现有 Map 后缀和书签分派；FeatureBookmark 的新 IName 实现未支持时还需要对象族接线。Legend 旧样本同样需要真实前序登记表。不要预填对象键作为完整链验证。

本轮 `cargo test --test round4_page` 两项通过；全仓 `cargo test` 在既有 `tests/composite_graphics.rs::complete_sample_context_reads_cgl_sublayers_without_skipping` 失败（实际偏移 151540，断言 151528），后续测试未执行。`cargo fmt --check` 仍因本组禁止编辑的 `src/maps/map_object.rs:954` 格式差异失败；本组新增测试文件已用 rustfmt 格式化。上述共享文件的状态可能由并行工作变化，未擅自修改。`git diff --check` 通过（不覆盖未跟踪文件）。
