# Basemap 第三轮：PrepareResult 与 Legend 回引

## 额外优先项 Basemap：PARTIAL

从 10.1 介质 Africa.mxd 的 `/Maps` 根顺序读取，Map 空间参考新对象键 39，顶层 BasemapLayer 新对象键 40；Basemap 第一个普通槽是键 39 回引，第二个普通槽是 ArrayOfIShader 新对象键 41（版本 1、计数 0）。LoadResults 的计数为 6。第一条 PersistedPrepareResult 的版本为 2，其第一个列表对象是 FeatureLayer 新对象键 42，现有 FeatureLayer 读取器完整消费该体至 `0x145d` 并提交其中真实读取的引用表。下一个对象头是 Map 新对象键 91，当前在该对象头之前停止，不登记键 91。键 42 首次出现在 PrepareResult 列表，不能断言它首次出现在后面的正式子层循环。

顶层条目为 ExtraSafe 标志 3，前缀 A=25585、B=0。现有代码对 B=0 的条目继续沿用 A 为对象总体积的**观察推定**，并非原生读侧的强同步；在此界内解析成功前缀，将 `0x145d..0x65fd` 保留为未解析体。`layer_list` 使用有界切片和独立试探引用表，防止读取或登记超越外层范围；体读取失败则不提交试探结果。Basemap 当前不生成图层模型，诊断记录停止原因。

真实根回归在原生 Map 前缀、顶层 Basemap 和后续 Legend 之间使用同一个引用表；Legend 第一个 HorizontalLegendItem 的 ILayer 槽被分类为 FeatureLayer 键 42 的回引。第二个条目的键 140 尚未登记，Legend 在该处停止；不能声称整个 Legend 或六条 PrepareResult 已读完，也不能声称正式 ExtraSafe 子层表已经读到。未知范围内可能还有须登记的对象，不能将有界跳过解释为完整解析。

## 10.1 原生证据

- BasemapLayer.dll：FileVersion/ProductVersion `10.1.0.3035`，SHA-256 `c8f4692970c5311e5d591f317c5e6c0379e9e6b9ad62a2383c71ecb6c8f64aad`。Basemap `Load` `0x6b646260` 与 ExtraSafe 调用见上一轮报告。配置表及 `0x6b66eb80` GetClassID 对应 ArrayOfIShader `{E0BBDBAF-0059-449C-B6F1-EDF1353A2C54}`；RTTI `.?AVArrayOfIShader@@` 的子接口虚表 `0x6b6adf30`、Load 槽 `0x6b66f7d0` 读 short 版本（有符号 >1 拒绝）、int32 数量，再按数量调用元素读取；Save `0x6b66f8a0` 对称写版本 1 与计数。当前仅接入空数组，新非空数组明确停止。
- MappingCoreLib.dll：FileVersion/ProductVersion `10.1.0.3035`，SHA-256 `c2bf81dcfcb65473ce35b953c4f4b76392a594085460c299326cc20bed59986a`。导出 `CPrepareResultsContainer::LoadResults` `0x79b719a0` 读取 int32 计数并循环构造 `PersistedPrepareResult(IStream*)`；构造函数 `0x79b6f710` 委派 `Load` `0x79b6fa20`。后者读 short 版本（有符号 >2 拒绝）、int32×2、一个 LoadObject；版本 1 接着读两个 LoadObject，其他版本读 int32 计数及相应 LoadObject 循环。Save `0x79b6f8b0` 对版本 2 数量分支提供写侧佐证。对象槽新类只有完整读取后才能继续；当前仅接入完整 FeatureLayer。
- Ghidra 桥接工具 6.9.0 独立 CLI 验证目标摘要时，Ghidra 桥接因沙箱禁止 Unix socket bind 返回 `Operation not permitted`。本轮改用隔离的本地 pyghidra/Ghidra 12.2 项目定点反编译，原始输出与脚本仅在 `temp/`；伪代码并非原始源码。未切换共享 Ghidra 桥接工具 目标，也未运行 ArcMap。

## 尚需主代理协调的共享入口

Map 键 91 体中的空间参考是此前登记的 PCS 键 39 回引；现有 `read_map_suffix_10_1` 重读前缀时按 PCS 新对象体处理，出现 WKT 长度错误。需要让 Map 前缀使用调用方共享引用表，区分已登记对象和新对象，并明确 Map 全体完成条件；随后才能安全读取这个无外部长度的普通 Map 对象、继续六条结果、正式 ExtraSafe 子层区及其余 Legend 条目。不能通过扫描 GUID 或预填引用表绕过。

## 验证与限制

`tests/round3_basemap.rs` 包含正常空结果、非法版本和截断回滚测试，以及依赖本地 10.1 介质的两条忽略测试：从 Map 根到完整键 42 和停止点、从 Map 根到 Legend 键 42 回引。该两条介质测试已显式运行通过。静态逆向和样本读取不代替 ArcMap 10.1 / QGIS 打开与目视验证，本轮未做人工验证。

### 准备结果的有符号判断

Basemap 和布局优化扩展均调用 MappingCoreLib.dll 10.1.0.3035
的 `0x79b719a0`。该入口按有符号 LONG 遍历结果；记录体
`0x79b6fa20` 对版本使用有符号 short 上限 2，非 v1 再按
有符号 LONG 遍历对象。Rust 在 Basemap 宿主接受非正计数的
空循环、保留外层计数原值，并按剩余字节和资源限额约束正计数。
构造测试覆盖负计数和负版本后继续读正式子层计数。此组合的
真实宿主和 ArcMap/QGIS 人工核对仍待验证。
