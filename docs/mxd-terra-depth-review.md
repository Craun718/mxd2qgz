# Terra 原生取证与接线复核（2026-11-17）

接续 [72 项基线](mxd-terra-integrate-review.md)，本轮六个 GPT‑6.1 Terra 子代理并发交付，主审复核并关闭四项；剩余 68 项。

| 工作 | 可验证结果及边界 |
| --- | --- |
| [Map 尾部](mxd-terra-depth-maptail.md) | 接入 IBorder 后四个 short、ILayer/IEnvelope 空槽和 double。非空或截断保留当前位置；未知尾部推进到参考样本 `0x25044`。后续字段和非空对象仍待闭合。 |
| [数字格式](mxd-terra-depth-latlon.md) | 末字段控制小数分；九类已知布局的标量语义关闭。正式属性名和实际运行仍未验证。 |
| [标注引擎](mxd-terra-depth-label.md) | 注册、QI 和持久化子虚表确认 JScript、Python 与 VBScript 共享体；接入两个宿主。其他引擎和真实宿主保持开放。主审核对并更正标准 IPersist/IPersistStream 的 IID 名称顺序。 |
| [名称对象](mxd-terra-depth-names.md) | TopologyName 三个持久化字段对位，关闭静态语义项；FeatureDatasetName 的两个字符串和标志仍未知。 |
| [图例](mxd-terra-depth-legend.md) | LegendClassFormat 的两个标量对位，关闭该项；HorizontalLegendItem 和 Legend 仍有未定字段。 |
| [历史介质](mxd-terra-depth-media.md) | 从实际基图宿主读取一个内嵌 Map、其 CRS 回引和空 Envelope 槽；未找到本轮要求的三种旧版分支，且不视为第二个根 Map。 |
| [FeatureLayer IID](mxd-terra-depth-iid.md) | 主审核对版本门、调用点和 PE GUID 原始字节；关闭对已知 IUID 槽的错误 IID 归属。 |

主审使用仓库内独占 TMPDIR 复跑 `cargo fmt --check`、`cargo test --offline`、`git diff --check`；741 项通过，11 项默认忽略。另显式运行本轮依赖安装介质的单项测试并通过。样例 CLI 仍报告六条图层记录。未在 ArcMap 或 QGIS 人工打开验证。
