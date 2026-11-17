# Terra 宿主解析复核（2026-11-17）

接续[持久化链复核](mxd-terra-chain-review.md)。六个 GPT‑6.1 Terra 子代理并行取证；主审接线、复查事务边界并更新 TODO。关闭 G 轮三项静态读写特例合并项，原有开放项剩 67 项。

| 范围 | 复核结果 |
| --- | --- |
| [Map 变换后尾部](mxd-terra-host-tail.md) | 接入两个定长字段、空背景槽和 BasicOverposterProperties；参考样本停在 `0x252be`，余 687 字节。后续字段、其他对象类与业务属性仍开放。 |
| [坐标变换类](mxd-terra-host-geo.md) | 证实 GeoTransformation 与 NADCON 使用同一持久化读写函数；接入同一严格读取器，并用构造宿主测试两类分派。没有运行坐标变换。 |
| [独立表](mxd-terra-host-table.md) | 确认 StandaloneTable 版本门与首段，接入 Map 首个新对象。未知嵌套体停止，错误保留整个表对象原位，不当作完整表。 |
| [DisplayCore 特例](mxd-terra-host-color.md) | 对位 Random 的 Size 请求槽、PictureFill 写出计算尺寸和 MultiPart 子渐变权重；纠正恒零说法，读取原始位不变。 |
| [StdPicture](mxd-terra-host-ole.md) | 核实介质 OLE 中第二字的读写处理及首层分派。没有推定字段业务名或 Windows 运行库差异。 |
| [标注与模板](mxd-terra-host-label.md) | 对位 FeatureIDSet、EditTemplate 和 ETM 的若干成员，保留原值；实际 OID 对应和尾标志完整生命周期仍开放。 |

没有可用的 ArcMap 环境。没有在 ArcMap 或 QGIS 人工打开验收；静态伪代码、构造测试和参考样本不替代实际运行。

主审复跑 `cargo fmt --check`、`cargo test --offline` 及 `git diff --check`，760 项通过，11 项默认忽略。参考样本 CLI 仍读出六条图层记录；未知 Map 尾部偏移为 `0x252be`。
