# 第四轮：名称、数据集和数字格式（2026-11-17）

## 身份、证据和范围

重新计算 GdbCore 与 SystemCore 提取文件 SHA-256，分别为 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`、`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`；用 PE 固定版本信息核对均为 10.1.0.3035，和[第三轮](mxd-round3-dataset-format-20261117.md)一致。REA 6.9.0 打开 GdbCore 时返回相同文件摘要；本轮批量请求 `0x708f4ab0`、`0x708f5070`、`0x708f5280`、`0x709b7c90`、`0x7095e5c0`，Ghidra 12.1.4 未识别这些函数入口，五项均失败（`ev_2c699b46ed1a5eb231e68067611f64e94ad8af4c6bdfd0e52e10994170d27883`）。失败不证明函数不存在，也不作为新反编译证据。此前 REA 证据及地址见 [TableQueryName](mxd-table-query-name-variant-20261116.md)、[数据集体](arcmap-10.1-dataset-name.md)、[数据集族](arcmap-10.1-dataset-name-slots.md)、[数字格式](arcmap-10.1-number-format.md)、[书签](arcmap-10.1-legend-family.md) 与第三轮记录。

已检查本地 10.1 介质提取的 66 个模板及其流索引存在，以及本机 Wine、既有 Wine 前缀和参考样本；之前的介质检索范围和结果见[介质记录](mxd-dataset-format-evidence-20261117.md)。空的 `temp/mxd101-run/bin` 不构成安装的 ArcMap；未发现可用的已安装 ArcMap 10.1 或能生成所需对象的受控目标环境。没有从字符串或原始 GUID 命中推断对象、字段或边界。本轮未在 ArcMap/QGIS 人工检查。

## 本轮实现和父代理接线

在 `src/objects/dataset_name_more.rs` 新增 `read_bookmark_name_slot_10_1` 和 `BookmarkNameSlot10_1`：空、回引、FeatureClassName 新对象及未知新对象。书签 Load 的请求 IID 是 IName；FeatureLayer 的数据源槽也请求 IName 并已有 FeatureClassName 类体证据，见 [Map 对象记录](arcmap-10.1-map-object.md)与 [书签记录](arcmap-10.1-legend-family.md)。仅复用该已知类体，不代表穷尽 IName 实现；不模拟加载后 QI、数据源打开或 IFeatureClass 实例化。未知类不消费对象头、不登记键；解析错误回滚游标和登记表。已知类若内部停止，保留 `body.stop`，游标停在内部未知对象处，调用方不能继续读取书签尾部。

父代理若接线，应由书签解析器调用此槽函数、透传内部停止和未知类、仅在完整体时继续读书签名称；并保留现有事务与共享对象引用表。此处不修改书签前缀、公共 mod/lib 或其他组文件，不声称真实书签样本验证。

## TODO 原行状态

| 原行 | 状态 | 证据与剩余工作 |
| --- | --- | --- |
| 178 | 部分 | 本轮实现 FeatureClassName、空和回引的 IName 槽子集；实现类分布、真实书签与加载后的 QI 仍待证。 |
| 182 | 部分 | TableQueryName 候选读写函数的局部静态证据见第三轮及变体流记录；本轮 REA 函数未识别。FeatureQueryName、helper 和完整读写闭合后才能实现。 |
| 222 | 部分 | FeatureDatasetName 四字符串及标志的对象偏移和读写结构已由 GdbCore 10.1 Load/Save 记录；访问器与业务字段未绑定。 |
| 223 | 部分 | TopologyName 两字符串和 IDatasetName 槽已有静态体布局；访问器和字段意义未绑定。 |
| 234 | 部分 | 九类字段序、版本门已有 10.1 SystemCore 静态记录，业务属性尚缺逐槽访问器映射。 |
| 235 | 部分 | RateFormat、CustomNumberFormat 字符串的读写已定位；业务名尚无访问器绑定。 |
| 236 | 部分 | 同槽和独立槽有静态 Load/Save 证据与既有构造测试；缺真实对象及运行状态证据。 |
| 245 | 部分 | 五个 IID 已由 10.1 类型库定名（IName、IQueryDescription、IQueryFilter、ISelectionSet、ISpatialReference），六类标量和字符串业务意义仍未定。 |
| 271 | 阻塞 | 短体分支已有静态结构与构造测试；尚无可确认边界的九类真实旧档或可控目标运行输出。 |
| 272 | 阻塞 | 缺真实对象或可控运行，静态同槽偏移不能升级为运行验证。 |
| 273 | 阻塞 | 记录的 Angle Load 忽略核心错误属于静态观察；原生流游标与后续字段行为缺运行取证，当前严格读取器不模拟。 |
| 283 | 阻塞 | GdbCore Load v0/v1 同布局且写侧恒写 1；缺确认对象边界的真实 v0 旧档。 |
| 284 | 阻塞 | 同上，TopologyName Load v0/v1 同布局、写侧恒写 1；缺确认对象边界的真实 v0 旧档。 |

## 检查

`cargo test --test round4_names_format`：3 项通过，覆盖完整 FeatureClassName 槽、回引、空值、未知类回退、逐字节截断事务与内部未知对象停止；均为按静态布局构造的输入，不代替实样。

`cargo fmt --check` 未通过，报告并行工作中的 `src/maps/map_object.rs`、`tests/round4_page.rs` 格式差异；本组未修改。`cargo test` 运行到 `tests/composite_graphics.rs` 时失败：`complete_sample_context_reads_cgl_sublayers_without_skipping` 断言 151540 与 151528 不同；本组不拥有该文件，未改动，后续测试未执行。Cargo 另提示全局缓存清理权限不足。因共享工作区并行变化，失败归因需父代理复核。
