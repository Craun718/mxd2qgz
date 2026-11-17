# MXD 对象流与 Variant 对象分支研究（2026-11-16）

本轮处理 `TODO.md` MXD 解析行 95–103、112。目标版本为 ArcMap 10.1；本轮没有修改 TODO、解析器接线或既有研究文档。此前完整的 `cargo fmt --check`、`cargo test` 通过，参考样本 `parse` 停在 `0x1622`，本轮未重跑这些基线验证。

## 证据身份与方法

10.1 AfCore.dll 的 SHA-256 为 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，SystemCore.dll 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与 [10.1 原生文件基线](arcmap-10.1-native-baseline.md) 中从 ArcGIS 10.1.3035 安装介质提取并核验的文件一致。本轮确认的安装配置版本和 DLL 字节身份不代表运行时观察。

使用已保存的 Ghidra 桥接工具 6.8.0/Ghidra 12.1.4 及 Ghidra 桥接工具 6.9.0/Ghidra 12.2 反编译/指令产物，包括 `temp/pseudocode/SystemCore/functions/0x7ffcf030.c`、`0x7ffcf7e0.c`、`0x7ffd0950.c`、`temp/pseudocode/AfCore/functions/0x68a7af70.c`、`0x68a7c1d0.c` 至 `0x68a7c390.c`，以及隔离工作树中 `simple_save_object_decomp.c` 与 LoadObject 汇编。Ghidra 桥接工具 当前没有既存打开会话；本轮对 AfCore 的新鲜 `procedure_pseudo_code(0x68a7af70)` 在首次导入时失败，诊断报告 Ghidra 的 FunctionID 数据库写入 `/tmp` 时 ENOSPC（`/tmp` 当时仅余 86 MiB）。没有重试相同分析。以下静态结论来自已保存产物，不是此次失败请求的结果。未运行 ArcMap，未做 ArcMap/QGIS 人工检查。

## 按 TODO 行号逐项结论

| TODO 行 | 状态 | 证据结论 | 剩余缺口 |
| --- | --- | --- | --- |
| 95 跨流复用 ObjectStream 对象体 | 部分 | 10.1 `PageLayout` 的文档加载链对同一 ObjectStream 调用 `SetStream` 以切换 OLE 流；SystemCore `0x7ffcf7a0` 不重置读/写对象表。对象头后没有通用体长，未命中键先登记，再直接调用对象的 `Load(ObjectStream)` 或包装的 `Load(VariantStream)`；体由具体 COM 类的持久化方法消费。因此跨流共享的是引用上下文，体边界仍由每类 Load 决定，无法对未知类安全跳过。`docs/arcmap-10.1-object-body.md` 的读取分支、SetStream 和错误分支记录及 `temp/pseudocode/SystemCore/functions/0x7ffcff70.c` 支持此结论。 | 尚未覆盖所有类的 Load/Save，也没有真实运行验证。要做到完整 MXD 对象体解析，仍需逐类闭合并接入；不得给未知类推定长度。 |
| 96 跨流 ObjectStream 的版本适用性 | 部分 | 上述 SetStream、读表生命周期和普通/VariantStream 分支均直接来自 SHA-256 已核验的 ArcMap 10.1.0.3035 SystemCore。既有 `docs/arcmap-10.1-object-body.md` 的跨版对照记录 10.1 与 10.8 同构的接口 IID、SetStream 不清表及包装形态；10.1 的具体结论可以用于该版本目标。 | 10.8 侧属于既有静态研究，未在本轮重新打开或核验 10.8 原生文件；也没有证据支持外推到其他版本。完整的类级对象体不因容器机制相同而自动跨版本成立。 |
| 99 VariantStream 对象分支的对象体 | 部分 | 10.1 VariantStream 的 9/13 分支进入 `LoadObject`，然后为新对象建立流包装并调用对象持久化 Load；SystemCore ObjectStream 分支同样会给对象创建 VariantStream 包装。对象体没有通用长度，未知类体无法解析或跳过。`docs/arcmap-10.1-variant-stream.md`、`docs/arcmap-10.1-object-body.md` 和 `temp/pseudocode/AfCore/functions/0x68a7af70.c` 的对称写侧提供静态支撑。 | 对象体字段必须逐类依据具体 Load 闭合；非零类 GUID 需要 COM 类实例化和对象体解码。本轮无可控 Variant 对象样本或目标软件运行证据。 |
| 100 VariantStream 对象分支引用表 | 部分 | Variant 对象读取调用 AfCore `LoadObject`；该入口先查询流上下文接口。若该流暴露 ObjectStream 上下文，则转发到其对象头/键查表分支，复用会话引用表；没有该上下文则按普通对象流路径读取类 GUID 并创建实例。因而 Variant 本身不另带独立引用表，是否使用 ObjectStream 表取决于实际传入流包装及 QI 结果。依据 `docs/arcmap-10.1-variant-stream.md` 的 `LoadObject` 路径、`temp/pseudocode/SystemCore/functions/0x7ffcff70.c` 和保存的 AfCore LoadObject 反编译。 | `LoadObject` 后续包装函数及每种嵌套流组合的完整接口传播链未全部闭合；不能仅凭 VARTYPE 9/13 判断键引用必然存在。无 10.1 运行样本验证 QI 命中情形。 |
| 102 SimpleSaveObject 委派链 | 部分 | `SaveObject` 是薄包装，调用 `StrmUtil::SimpleSaveObject(..., writeHeader=1)`。SimpleSaveObject 的普通持久化分支在可选 CLSID 头后直接调用 `IPersistStream::Save(stream, clearDirty)`；Variant 分支递归保存值对象或空对象，并建立 VariantStream 包装。SystemCore 的写侧 ObjectStream 路径独立负责写 CLSID、对象键、回引以及随后调用具体对象 Save。已有伪代码能解释各路径职责，但普通分支不写键，不能把它单独视为 ObjectStream 写出器。 | 查询包装器的准确 IID/成功分支、包装接口与 ObjectStream `+0x40` 写槽的完整端到端到达路径仍有间接调用和寄存器归属不确定；本轮 Ghidra 桥接工具 新鲜分析因 ENOSPC 未能补足。报告为部分，不把伪代码的局部调用关系冒充完整委派闭环。 |
| 103 ObjectStream 键计数器构造初值 | 完成（10.1 静态） | SystemCore 构造函数 `0x7ffcf030` 显式执行 `param_1[0xf] = 0`。该对象的 `IObjectStream2` 接口位于基址 `+0x10`，故构造字段基址 `+0x3c` 对应写侧方法所见 `this+0x2c` 键计数器。写侧 `0x7ffcf7e0` 先把该字段写入 4 字节键，再递增；因此新建的 10.1 ObjectStream 首个分配键初值为 0。该构造器完整反编译保存于 `temp/pseudocode/SystemCore/functions/0x7ffcf030.c`，写侧见 `0x7ffcf7e0.c`。 | 结论绑定该 10.1.0.3035 SystemCore 实现；其他版本未核验。构造函数地址证据是静态反编译，不代表通过 ArcMap 运行确认。 |
| 112 重复键替换分支的触发与结果/回引交互 | 阻塞（运行场景） | 10.1 SystemCore 插入/更新函数 `0x7ffd0950` 在已有键时覆盖读表节点的对象指针并返回 1；该函数本身不释放旧对象引用。正常读入口先查表，仅未命中时才创建并登记，所以正常 MXD 读取不会调用此替换分支。写入口则先按对象指针查写表；命中时直接写既有键并结束，不会通过同一正常保存路径反复插入相同键。依据 `docs/arcmap-10.1-object-body.md` 与 `temp/pseudocode/SystemCore/functions/0x7ffd0950.c`、`0x7ffcf7e0.c`。 | 必须构造能令原生插入/更新函数收到已存在键的运行场景，再观察覆盖对象和后续回引行为；当前未安装/运行 ArcMap，也无触发样本。静态调用路径表明普通加载/保存不触发，不能用合成 Rust 登记表测试代替原生场景。 |

## 实现与接线

本轮没有修改 Rust 源码：可实现的对象体信息仍缺少各类具体边界证据；补一个通用消费长度或对象回引逻辑会把未知误当已知。现有 `object_stream`、`object_refs` 和 Variant 对象分支读取器维持严格停止/显式不支持的安全行为。此次也没有新增合成测试，因为未新增解析行为；现有正常/异常测试仍为此前基线，不替代目标软件运行验证。

主代理需在总研究汇总中记录：行 103 可按本报告勾闭；行 95、96、99、100、102 属部分，行 112 需真实 ArcMap 场景，保持开放。后续解析接线需要调用方持有跨流共享引用上下文，并按类 Load 结果推进游标；Variant 对象分支只有在确认具体流 QI/类身份及对象体边界后才可扩展。所有依据均为静态证据，未完成人工验收。
