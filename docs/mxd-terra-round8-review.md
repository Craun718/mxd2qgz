# Terra 第八轮复核

2026-11-17。接续 [66 项基线](mxd-terra-nextpass-review.md)，六个 GPT‑6.1 Terra 子代理并发取证，主审接续类型库对位及 SDK 宿主验收。关闭 FDO 旧加载器的静态接口槽语义，余 65 项。

| 范围 | 结果与边界 |
| --- | --- |
| [Map 遮罩](mxd-terra-round8-mask.md) | 接入地图自身的非空遮罩结构子集；图层明确无接口时零消费。逐图层接口状态和 Map 终点未闭合，参考样本仍停在 `0x253be`。 |
| [边框与背景](mxd-terra-round8-border.md) | 接入两种已确认的非空实现；不完整或未知体回退到槽头。 |
| [介质实档](mxd-terra-round8-old.md) | 新定位 Desktop 与 SDK 工程；从真实宿主验证 FieldInfo v2、GroupLayer v11/v12 和非空独立表 v9，所需更早分支仍待证。 |
| [TableName](mxd-terra-round8-tablename.md) | SDK 实档反证旧有不可持久化推断。定位跨 DLL 的基类跳板并接入，实档首表读过名称与工作空间后停在下一对象 `0x6bed`。 |
| [FDO 接口](mxd-terra-round8-fdo.md) | 对位实际写入槽，纠正 SelectionSymbol 请求 IID，保留原始标量。旧体实档及运行行为另验。 |
| [空间参考](mxd-terra-round8-sr.md) | 静态定名 GetPrecisionSizeMax 和 GCS 编辑接口，守卫来源和域串业务名未闭合。 |
| [版本保存](mxd-terra-round8-version.md) | 追通 “Save A Copy” 子类型 12 到非零版本选择的静态链，实际派发及旧档是否改写索引仍须运行取证。 |
| [VariantStream](mxd-terra-round8-variant.md) | 更正 AfCore QI 辅助函数及普通流分派，仍缺已证实的非空具体宿主。 |

未做 ArcMap/QGIS 人工验收。静态原生证据、实档字节及合成测试分别记录。

主审复跑 `cargo fmt --check`、`cargo test --offline` 和 `git diff --check`：
786 项通过、14 默认忽略；另显式运行三项介质依赖测试，全部通过。
参考样本 CLI 仍读出六条图层记录、保留从 `0x253be` 开始的 431 字节。
