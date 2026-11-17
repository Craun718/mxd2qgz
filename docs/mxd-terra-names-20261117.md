# 名称、数字格式与 Hyperlink 基线复核（2026-11-17）

本表行号固定为本次任务给出的 TODO 基线，不随以后插行重编号。未修改 TODO；“不可勾选”指对应完整缺口仍未闭合。此前结果及 Ghidra 桥接工具 Evidence 的完整上下文见[第四轮名称取证](mxd-round4-names-format-20261117.md)、[变体流](mxd-table-query-name-variant-20261116.md)、[数字格式](arcmap-10.1-number-format.md)、[样本检索](mxd-dataset-format-evidence-20261117.md)和 [Hyperlink](arcmap-10.1-hyperlink.md)。本轮没有重新运行 Ghidra 桥接工具，引用的是明确标明版本的既有 Ghidra 桥接工具 静态结果，不将其描述为运行观察。

## 逐行裁决

| 基线行 | 可勾选性 | 证据、阻碍和后续复核 |
| --- | --- | --- |
| 184 | 不可 | GdbCore 10.1 TableQueryName 候选 Load `0x708f5070`、Save `0x708f5280` 已有 Ghidra 桥接工具 伪代码；`Write_exref`、间接调用与虚表归属链及 FeatureQueryName 完整读写仍未闭合。进一步定位两类虚表、GetIDType、helper 和实际流消费；不能把另一类 QueryTableName 的读取器当作实现。 |
| 224 | 不可 | FeatureDatasetName Load `0x709b7c90` 的四个字符串、标志和 Workspace 槽有静态布局；Africa 有局部 v1 对象链。各字段与访问器的业务属性尚未逐一绑定，需要类型库成员、实际虚表访问器及字段偏移交叉验证。 |
| 225 | 不可 | TopologyName Load `0x7095e5c0` 的两个字符串及数据集名称槽有静态布局；缺访问器字段映射和拓扑对象实证，不能据顺序猜测名称。 |
| 236 | 不可 | 九类 SystemCore Load/Save 已记录版本门与标量布局，但未完成所有标量到接口属性的逐类访问器绑定。须逐类关联类型库、虚表及成员偏移。 |
| 237 | 不可 | RateFormat 和 CustomNumberFormat 字符串槽有静态读写证据，但未完成属性与访问器绑定，类名不足以证明业务含义。 |
| 238 | 不可 | Percentage/Rate 同槽和 LatLon/Angle 独立槽为 10.1 静态反编译结论，既有合成输入仅检验读取布局；需可控目标运行或从宿主槽建立真实对象边界并观察状态。 |
| 247 | 不可 | 六类体结构与五项 IID 定名见第四轮；属性与调用用途仍需逐项从接口访问器和调用链确认，IID 名称不证明各字段意义。 |
| 271 | 不可 | Hyperlink v0 在 10.1 Load 门内，与 v1 字段序相同，现有 `tests/hyperlink.rs` 为合成测试。需旧档中宿主循环可确认的 v0 对象和后继边界，或受控运行取证；10.1 Save 恒写 v1。 |
| 272 | 不可 | ObjectStream 的通用空引用和回引规则及接线测试存在，参考样本对应循环计数为 0；需从真实 Hyperlink 宿主循环或受控保存确认两种条目。 |
| 273 | 不可 | 九类短体版本门有静态证据与构造测试，尚无逐类可确认边界的旧档或运行输出；不能把 10.1 写侧满体或合成体当短体实测。 |
| 274 | 不可 | 静态对象偏移支持同槽与独立槽区分，缺实际对象或目标运行观察；与 238 相同证据缺口，但此行为样本验证要求。 |
| 275 | 不可 | Angle Load `0x7ffb33a0` 忽略内嵌 NumericFormat Load 的返回值；严格读取器遇核心失败整体回滚。本轮新增非法核心版本且尾字段可读的合成测试，固定严格策略；原生失败时游标、HRESULT、部分对象状态及后续消费仍需运行取证，不能据此修改为容错。 |
| 285 | 不可 | FeatureDatasetName 的 10.1 Load v0/v1 同布局、Save 恒写 v1；既有实样为 v1，需可确认对象边界的 v0 旧档或受控运行。 |
| 286 | 不可 | TopologyName 的 10.1 Load v0/v1 同布局、Save 恒写 v1；未获该类可确认对象边界的 v0 旧档或受控运行。 |

## 环境与证据身份

本轮重新计算提取文件 SHA-256 并读取 PE 固定 FileVersion：GdbCore `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`、SystemCore `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`、FeatureLayer `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，均为 10.1.0.3035，与既有研究一致。仓库参考 MXD 和 `temp/arcmap101-media-mxd/` 模板存在；第四轮及样本记录检索了 10.1 介质 66 个 `/Maps`，候选筛选不能代替完整对象遍历，零命中仅对所扫描流有效。第四轮记录另核查了 10.8 介质，但不能将其视为 10.1 原生实现。

本机 Wine 可执行，提取的 ArcMap.exe 存在，但 `temp/mxd101-run/bin` 为空；未发现可用的已安装 ArcMap 10.1 或能生成所需对象的受控运行环境。本机可找到 QGIS 启动器；本轮未打开任何工程，未在 ArcMap 或 QGIS 做人工验证。未更改其他代理的研究文档和共享交界模块。

## 本轮变更与复核命令

`src/objects/hyperlink.rs` 修正对象键登记职责的注释：体读取器不登记，由读取对象头的调用方登记；不改变解析行为。`tests/terra_names_20261117.rs` 覆盖上述严格回滚，合成测试不构成真实档验证。

在仓库根目录复核：

```sh
sha256sum temp/agent-dataset-name/GdbCore.dll.* temp/arcgis101-bin/SystemCore.dll.* temp/arcgis101-bin/featurelayerdll/FeatureLayer.dll.*
cargo test --test terra_names_20261117 --test hyperlink --test number_format --test dataset_name --test round4_names_format
cargo fmt --check
cargo test
```

本轮 `cargo test` 通过，标记忽略的介质依赖测试未运行。`cargo fmt --check` 未通过，差异来自并行改动的 `src/layers/fdo_legacy.rs`；对本轮两个 Rust 文件运行 `rustfmt --check --edition 2024 src/objects/hyperlink.rs tests/terra_names_20261117.rs` 通过。Cargo 有全局缓存自动清理权限警告，未影响测试。
