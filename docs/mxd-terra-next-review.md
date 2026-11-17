# MXD 解析 Terra 并行复核（2026-11-17）

以 ad0d6f4 工作副本的「MXD 解析」小节为基线，75 个未勾选项全部分给 6 个 GPT‑6.1 Terra 子代理。表中行号是基线定位，不随 TODO 增删重编号。

| 范围（基线行号） | 分项证据 |
| --- | --- |
| 77–95 中的开放项 | [版本流](mxd-terra-next-version.md) |
| 106–113、123 中的开放项 | [对象流](mxd-terra-next-stream.md) |
| 119–121、132、180、259、284 | [Map 与书签](mxd-terra-next-map.md) |
| 159–161、229–232、278 | [标注与图形容器](mxd-terra-next-label.md) |
| 184、224–225、228、245–247、255–256、285–286 | [名称、FDO 和 IID](mxd-terra-next-names.md) |
| 219–222、226–227、248、266 | [页面与图例](mxd-terra-next-page.md) |
| 216、218、223、250、269–270、276–277 | [显示与 OLE](mxd-terra-next-display.md) |
| 236–238、271–275 | [数字格式与超链接](mxd-terra-next-format.md) |
| 240、249、279–280 | [几何与坐标系](mxd-terra-next-geometry.md) |
| 261、281–283、296–297 | [FeatureLayer 尾段](mxd-terra-next-feature.md) |

主审核对了指派范围，无遗漏、无重复。将 FeatureLayer 后继段的 ISymbol 槽与 LELP 序列区分；修正 Percentage/Rate 的核心读入地址解释，并同步更正旧研究文档。构造测试只验证解析器自身的边界及事务语义；特别是 PageLayout 类仅作为 LELP 容器槽的静态候选，当前测试检查安全停止，不代表 ArcMap 生成过该槽值。

参考样本、介质旧模板和原生反编译相互补充。另用 Ghidra 桥接工具 打开 FeatureLayer.dll，确认 x86 PE 与 SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`；Ghidra 12.1.4 对 `0x6f6e9940` 的请求在导入阶段因 `/tmp` 空间不足终止（run `3637c374-2c1f-40bd-8add-d7fc8d391b20`），未返回新伪代码，已关闭会话。部分代理的隔离 CLI 也遇到 Unix 套接字限制；各报告区分了既有反编译、新静态复核与未取得的运行证据。

设备上未找到可直接运行并确认已授权的 ArcMap 10.1/10.8 安装。真实旧档分支需要从宿主对象和引用表顺序到达，不能用 GUID 扫描或构造体替代。人工验收未做。

主审复核了 10.8 SystemCore 的 Ghidra 桥接工具 证据记录及返回槽，并核对 DisplayLib 的 SHA-256、默认函数伪代码和 RotateWithTransform 属性访问器。关闭三个标记符号的 10.1 静态字段语义一项；版本流读取器新增条件字段的候选解释，保留原字节。剩余 74 项继续开放。

主审复跑 `cargo fmt --check`、`cargo test --offline` 和 `git diff --check`：727 项通过，10 项默认忽略；另显式运行 `round2_symbol_ole_media` 的两项介质测试，全部通过。参考样本的 `parse` 文本输出与本轮基线逐字节一致（1 个数据框、6 条图层条目，能读到显示范围）。这不代表已完成 MXD 解析或目标软件验收。
