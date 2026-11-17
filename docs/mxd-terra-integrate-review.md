# Terra 解析接线复核（2026-11-17）

接续[定点续查](mxd-terra-follow-review.md)，本轮并发复核并接线六组。原有 74 项中关闭两项，余 72 项；其他有进展但尚未满足原条目完整条件。

| 范围 | 接线与闭合边界 |
| --- | --- |
| [查询名称](mxd-terra-integrate-query.md) | 对两类静态 QI 全链确认标准 ObjectStream 走 IPersistStream，原 IPersistVariant 判定撤销。已接入 IDatasetName 分派；主审另确认其 QI 常量包含该请求 IID。未知 workspace 子类保留原位，截断整体回滚。尚无真实宿主或原生运行验证。 |
| [Map 照明后](mxd-terra-integrate-map.md) | 两个相邻零 GUID 依次是 IGeometry、IBorder 空槽，主审复核原生调用顺序并接入主流程。非空或截断停在所在槽的起点。 |
| [NumberFormat](mxd-terra-integrate-numeric.md) | RateString、FormatString 是这九类已确认布局中的两个持久化 String，均已与访问器对位；另外 11 个标量属性定名。LatLon v4 末尾 short 仍未定性。 |
| [TextSymbol](mxd-terra-integrate-text.md) | 多个标量和 Size 相关存储已与访问器对位，代码注释保留原始值契约；未对位的属性和运行效果仍开放。 |
| [页面装饰](mxd-terra-integrate-page.md) | 三个共同前缀调用者为 ImageInset、MapInset、Overview；更正早期 ScaleLine 候选归因，真正 ScaleLine 及未定标量仍开放。 |
| [空间参考](mxd-terra-integrate-sr.md) | 更正 PCS/GCS v2 域串版本门以及 PCS 非零垂向单位段，避免精度块错位；GCS 守卫的业务含义仍未确认。 |

仍无可用 ArcMap 环境，未做 ArcMap/QGIS 人工验收。静态指令与反编译、样例解析及构造测试分别记录，互不替代。

主审以仓库独占 TMPDIR 复跑 `cargo fmt --check`、`cargo test --offline`、`git diff --check`，736 项通过，10 项默认忽略。参考样本 CLI 仍输出 6 个图层条目，并将未知 Map 尾部起点推进到 `0x25014`。
