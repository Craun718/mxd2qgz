# FeatureLayer 尾段与历史分支续查

2026-11-17。范围对应交接时 TODO 261、281–283、296–297；未修改 TODO、公共入口或其他代理源码。10.1 FeatureLayer.dll 经 PE 版本资源核对为 x86、InternalName `FeatureLayer`、OriginalFilename `FeatureLayer.DLL`、文件版本 10.1.0.3035，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`。身份核对不证明签名或运行状态。

| 行 | 状态 | 已有证据与剩余条件 |
| --- | --- | --- |
| 261 | 部分，未关闭 | FieldInfo v0–2 的 Load 门及合成正常/所有截断测试已有；SouthernUSA 的真实宿主链是 v3，不能证明旧分支。见 [第四轮图层](mxd-round4-layers-20261117.md)。 |
| 281 | 部分，未关闭 | FeatureLayer 版本 0x1c..0x1f 的字段序与条件已接入 `feature_layer_tail`；10.1 Save 版本表为 17、22、23、25、34、34，不产生该区间。缺真实旧版对象链或旧版运行记录，详见 [分支记录](mxd-parse-agent-sample-evidence-edge-branches-20261117.md)。 |
| 282 | 部分，未关闭 | 时间族四段及嵌套 TimeReference/TimeExtent 等已有静态实现；参考样本四槽为空，真实非空链尚未对位。合成嵌套测试不等同于启用时间的实际图层。 |
| 283 | 部分，未关闭 | PropertySet v0/v1 静态走同一布局，Save 写 v1；独立测试覆盖合成非空 v0 和全部截断，仍缺宿主可确认的真实 v0 旧档。 |
| 296 | 部分，未关闭 | `+0x1f4` 是 IEnvelope，Load 后传入数据源 QI 接口的虚表 `+0x10`；正式属性名称、绘制路径和生命周期未确认。不得将“实际绘制/缓存范围”当作事实；详见 [空间调用链](mxd-parse-agent-map-layer-spatial-20261117.md)。 |
| 297 | 部分，未关闭 | `+0x1b4` 的 Save 将槽传入 SaveObject；通用空指针路径写 16 零字节，参考样本的已观察槽均为空。构造、赋值和运行时副作用尚未穷尽，不能推断成员始终为空或没有副作用，见同一空间调用链。 |

本次重跑 `cargo test --test round4_layers --test property_set_v0_independent` 和完整 `cargo test` 全部通过；完整测试包含 `src/layers/dep_tail.rs` 的时间族合成测试，其证据级别仍是构造输入。按资源提示复用现存 REA 静态证据，没有启动新 Ghidra 导入。未运行 ArcMap 或 QGIS、未做人工目视验收；这些条目不应因静态和单元测试勾选。后续分别需要含旧分支的可定位旧档、启用时间的工程和 10.1 接口/运行调用记录。
