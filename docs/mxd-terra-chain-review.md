# Terra 持久化链复核（2026-11-17）

接续 [68 项清单](mxd-terra-depth-review.md)。六个 GPT‑6.1 Terra 子代理并发取证及接线；本轮有新的实现和静态证据，原有综合条目的其余验收要求尚未满足，仍有 68 个开放项。

| 范围 | 主审结论与仍缺证据 |
| --- | --- |
| [Map 坐标变换](mxd-terra-chain-map.md) | 定位 NADCONTransformation 原生持久化并接入 Map 变换循环；参考样本从根顺序读到新对象及回引，未知尾部从 `0x2524f` 起保留。其他变换类型、非空独立表和循环后的字段仍开放。 |
| [图例](mxd-terra-chain-legend.md) | Legend 集合前 short 在该原生版本中读而不用、固定写 `0xffff`；重新提取 Renderers 后证实 Horizontal 包装 int32 为参与布局且受流向影响的模式。各模式的完整含义仍未知。 |
| [名称对象](mxd-terra-chain-dataset.md) | FeatureDatasetName 第二串对位 IName.NameString；第四串及标志的业务名仍未证。 |
| [版本保存链](mxd-terra-chain-version.md) | 标准 MFC Save As 不进入已知的非零参数分支；“Save A Copy” 英文资源到过滤索引已对位，实际命令派发、其他语言、10.8 完整写入仍开放。 |
| [页面元素](mxd-terra-chain-element.md) | ElementProperties 和 TextSymbol 多个属性与持久化偏移对位，更新中文注释；部分彩色槽及其余字段尚未定性。 |
| [FeatureLayer](mxd-terra-chain-fl.md) | 纠正两级查询中的第二个 IID，并在 ArcMap.exe 找到同一接口的另一方法槽；未找到实际实现及 Envelope 属性名。 |
| [AnnoLayer 接口](mxd-terra-chain-iid.md) | 主审对照三个目标版本类型库及实际 QI 加数：确认四个接口名，并更正旧表错位。主 QI 末项仍未定名。 |

未在 ArcMap 或 QGIS 中运行和目视验收。原生静态证据、样例对位及构造数据各自记录，不作替代。

主审复跑 `cargo fmt --check`、`cargo test --offline` 和 `git diff --check`：744 项通过，11 项默认忽略。参考样本 CLI 仍读出 6 条图层记录，从 `0x2524f` 保留 798 字节。
