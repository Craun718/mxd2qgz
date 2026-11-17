# 标注与图形容器续研（2026-11-17）

本表使用本次任务指定的 TODO 行号。未修改 TODO。静态反编译、构造输入和介质字节均不是 ArcMap 运行或 QGIS 人工验收。

## 新证据：PageLayout 实现图形容器

复核本机 `temp/pseudocode/pagelayout/functions/0x7b6ccaf0.c`：REA/Ghidra 12.1.4 伪代码显示同一 PageLayout 构造过程在对象 +0x4c 安装 `IGraphicsContainer::vftable`，随后覆盖为 PageLayout 自身虚表。此前 [页面研究](mxd-round3-line-element-20261117.md) 从 RTTI 和持久化虚表 `0x7b71fa34` 定位 GetClassID `0x7b6e4e00`、Load `0x7b6e4e30`。本轮复读 GetClassID 伪代码，写出的 CLSID 为 `{DD94D76E-836D-11D0-87EC-080009EC732A}`。本轮重算输入 PageLayout.dll 的 SHA-256 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`，与导出及版本 10.1.0.3035 基线一致。

这证明 PageLayout 是一个静态可识别的 IGraphicsContainer 实现候选，且具有持久化实现；**不证明 LELP 实际保存过该类**。页面 Load 只解析了元素循环前缀及部分元素，其完整对象体尚未闭合，不能将此类直接接入 LELP 的容器槽。`tests/terra_next_label.rs` 用构造体检查该 CLSID 遇到未知体时在对象头停止、不登记键，以及截断时回滚；这不是目标软件存储该槽的证据。

## 逐项状态

| 行 | 状态 | 证据与剩余工作 |
| --- | --- | --- |
| 159 | 推进，未闭合 | VBScript 体已实现；类型库还登记 JScript/Python，仍缺其持久化接口、GetClassID 和 Load/Save 对应关系。见 [标注复核](mxd-round3-labels-featurelayer-20261117.md)。 |
| 160 | 推进，未闭合 | 上述 PageLayout 构造、CLSID 和持久化表建立一个实现候选；样本 LELP 槽为空，PageLayout 完整体及其他实现未知。 |
| 161 | 推进，未闭合 | 已知符号和 TextSymbol 已分派，但未穷尽实现类。注意此项的槽名有误：LELP ㉖ 是 int32，既有记录所指 ISymbol 位于 FeatureLayer 后继段，见 [标注记录](mxd-label-luna.md)。 |
| 229 | 推进，未闭合 | ALPC 读取结构和公开集合接口已知，流内标量与属性访问器尚未逐一绑定。 |
| 230 | 推进，未闭合 | LELP 的静态读取序和类型库公开 Expression 等接口已知，但缺对应成员偏移与访问器的逐项证据；样本文字不能单独证明字段名。 |
| 231 | 推进，未闭合 | ETM 结构、Count/EditTemplate 接口和样本尾字节 `0x01` 已知，标量及尾字节语义仍未逐一对应。 |
| 232 | 推进，未闭合 | FeatureIDSet Load 与公开 Add 操作同一集合，支持整数为集合成员；未证明它们必为具体数据源 OID，见 [第三轮](mxd-round3-labels-featurelayer-20261117.md)。 |
| 278 | 推进，未闭合 | UID/DEP 已有原生 Load 和样本结构；BOLP/HLEP 有样本边界，但各自的 GetClassID、持久化虚表和 Load 唯一对应仍未闭合，见 [边界证据](mxd-parse-agent-sample-evidence-edge-branches-20261117.md)。 |

## 验证

`cargo test --test terra_next_label` 两项通过；与 `dataset_name`、`fdo_legacy`、`terra_fdo_legacy`、`round3_labels_sample` 一起运行共 38 项通过。后两项标注样本测试有真实参考文件，但不构成目标软件运行验证。没有改动解析器或公共入口。本轮未启动新 REA 实例，使用已保留的隔离导出；未打开 ArcMap/QGIS。

全仓 `cargo test` 通过，包含既有忽略项；`cargo fmt --check` 因并行新增的 `tests/terra_next_geometry.rs` 格式差异失败，本组没有修改该文件。`rustfmt --check --edition 2024 tests/terra_next_label.rs` 通过。
