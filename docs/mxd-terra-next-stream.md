# ObjectStream 与 VariantStream 续查（2026-11-17）

范围为任务指定 TODO 当前行号 106–113、123，不修改 TODO。这里的「闭合」指所述静态机制，不能代替目标软件运行或跨版本兼容性验证。

## 输入与依据

本轮重新计算 `temp/arcgis101-bin/` 提取物 SHA-256：AfCore.dll `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`、SystemCore.dll `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与 [10.1 对象体基线](arcmap-10.1-object-body.md)一致，对应介质 ArcGIS 10.1.0.3035。读取本机 Ghidra（经桥接工具） 12.2 导出的 AfCore `0x68a7b490`、`0x68a7b530`、`0x68a7b6d0`、`0x68a7af70` 等伪代码，以及仓库中的 SystemCore、文档链与汇编记录；反编译不是原始源码。`0x68a7b530` 和 `0x68a7b6d0` 的输出含不可达块警告，不能独立证明所有路径。

补核已有 Ghidra 桥接工具 10.8 SystemCore 静态证据 `temp/round3-systemcore108-setstream.json`，目标 SHA-256 `48cbd921972d12156119d524f9c3923225fd30795e33c22c65c87db3a6c89aca`，Ghidra 12.2，`0x1006f8c0` 仅替换底流并 AddRef/Release，未见表清理。它只证明该函数的局部静态行为，不证明跨版本文件兼容或类体相同。

## 逐项结果

| 行 | 状态 | 证据与余项 |
| --- | --- | --- |
| 106 | 推进 | 10.1 SetStream 保留会话对象表；对象头无通用体长，已登记键走回引，未登记键先登记再由类的 Load 消费，见 [对象体记录](arcmap-10.1-object-body.md)及 [对象流续查](mxd-open-streams-20261116.md)。完整对象体须逐类研究，未知类不可跳过。 |
| 107 | 推进 | 10.1 机制绑定上述摘要；10.8 `0x1006f8c0` 的 SetStream 不清表是独立局部静态证据。其余版本、跨版本完整引用表及类体兼容性未闭合。 |
| 108 | 已勾选背景 | SetStream、登记时机、生命周期与接口分支有静态证据，完整类体和运行验证另论。 |
| 109 | 已勾选背景 | 10.8 Variant 已知类型宽度与字符串读取规则有既有静态证据；对象分支另论。 |
| 110 | 推进 | 10.1 `VariantStream::Read` 委派全局 Read，9/13 调 LoadObject；零 GUID 空值子集已存在，非空体取决于流上下文及具体持久化类，未知类无通用边界。见 [VariantStream 记录](arcmap-10.1-variant-stream.md)和 `src/stream/variant_stream.rs`。不能据空值实现宣布全部对象体完成。 |
| 111 | 推进 | 10.1 带键 ObjectStream 读路径使用同一会话表；普通流读取无带键前缀。实际包装传递、QI 命中及嵌套类行为尚需指令与受控运行验证，不能只按 VARTYPE 判断引用表。已有 [对象流调查](mxd-parse-agent-object-stream-20261117.md)对包装 QI 有推断和不一致表述，应以汇编及实际接口为准。 |
| 112 | 已勾选背景 | 10.8 的固定类型与字符串严格子集及不支持对象分支已经实现；合成测试不是运行验证。 |
| 113 | 推进 | 已有 [对象流调查](mxd-parse-agent-object-stream-20261117.md)记录 SaveObject 的 IObjectStream QI 命中委派写槽 +0x40、否则 SimpleSaveObject；普通 IPersistStream 分支写可选 CLSID 并调用 Save。`0x68a7af70` 的 Variant 递归存在，但其反编译栈变量混淆，全部递归分支和错误传播仍需指令级追踪，暂不声称完整链闭合；10.8 未复核。 |
| 123 | 阻塞（原生触发） | 10.1 `0x7ffd0950` 已有键时覆盖节点值；普通读取先查表，命中键直接回引，不会因流中重复键而自然进入插入替换。写侧按对象指针查表后命中即返回，不应采信旧文档中「同对象重复保存必触发替换」的说法。需受控调用实际替换分支并观察旧对象生命周期和后续回引；纯 Rust 集合重复登记不证明原生行为。 |

旧记录对普通流零 GUID 判断曾有相反结论；[第四轮修正](arcmap-10.1-variant-stream.md)记载按比较函数及汇编核实为零 GUID 返回空、非零创建实例。保存伪代码仍含寄存器和不可达块缺陷，涉及接口传播的结论应保留限制。

## 本轮实现与验证

没有修改对象流或 Variant 实现：现有可安全读取的空对象子集已存在，非空类体、包装 QI 及原生替换没有足够新证据支持扩展。未运行 ArcMap 或 QGIS；现有合成测试不是原生行为验证。
