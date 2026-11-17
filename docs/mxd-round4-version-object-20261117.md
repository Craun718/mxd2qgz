# 第四轮：Version 与 ObjectStream（2026-11-17）

范围以本轮开始时 TODO.md 原行号为准；未修改 TODO。状态表示原条目整体，局部实现不等于闭合。未运行 ArcMap 或 QGIS。

## 本轮证据与实现

复用第三轮与既有 REA/Ghidra 静态证据，重新核对介质提取文件 SHA-256：10.1 ArcMap.exe `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`、AfCore.dll `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`、SystemCore.dll `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与 10.1.0.3035 基线一致。10.8 ArcMap.exe `a9a963b9dffbf0cb346c67c98bce0880b7e64363410a788dccb9597cff3c75fe`、SystemCore `48cbd921972d12156119d524f9c3923225fd30795e33c22c65c87db3a6c89aca` 为 10.8.0.12790，结论只限该版本。原 PE 虚拟地址非运行地址。

新增 `stream::variant_stream::read_value_10_1`：复用经 [10.1 VariantStream 逐分支核对](arcmap-10.1-variant-stream.md) 的固定宽度及字符串安全子集；9/13 且全零 CLSID 返回空对象，非零 CLSID 明确 Unsupported，截断和错误均不推进游标。全局 VariantStream 与类方法 `Variant::Read` 是不同序列化器，不能借用后者独有 VARTYPE；对象零标记的极性以[后续更正](arcmap-10.1-variant-propertyset-slots.md)为准。测试是合成边界，不证明目标软件运行行为。尚无公共路由接入；由整合方按已证字段归属调用。

本轮以 objdump 复核 AfCore `0x68a7b640..0x68a7b6a4` 与 `0x68a7b6d0..0x68a7b76b`：SaveObject/LoadObject 均对流调用 `0x68a7c4b0`；命中时分别调用返回接口虚表 +0x40/+0x44，未命中时写侧进 SimpleSaveObject，读侧进 `0x68a7b490`，非空成功后进入 `0x68a7b530`。`0x68a7c4b0` 在 `0x68a7c4c0` 压入 IID 地址 **0x68c6dca8**；而 SimpleSaveObject 内 `0x68a7c1d0` 的 QI 在 `0x68a7c1e0` 压入 **0x68c6dcb8**，二者不可混同。部分旧注释将前者写为 dcb8；尚未在本轮解析这两个 IID 的全部身份。`0x68a7b530` 既有 Ghidra 12.2 伪代码提示大量不可达块，不能将其伪代码当作包装接口传播的完整证明。

REA 6.9.0 已识别上述 10.1 AfCore PE 哈希；新发起的批量反编译尚未返回时，发现 `/tmp` 可用空间降至约 7 MiB，主动中止等待以免继续增加 tmpfs 压力；没有把该次请求当作新反编译证据。其他结论依赖既有 REA 证据及本轮 objdump。没有清理其他任务或系统文件。

## 逐项状态

| 原行 | 状态 | 已有依据与具体剩余工作 |
| --- | --- | --- |
| 75 | 部分 | 134 个介质条目已知头后无尾字节；未见非空尾实例或完整消费者证据，仍须有非空文件并追踪消费方。 |
| 77 | 阻塞 | 严格读写仅合成测试；需可运行且已授权的 10.1，另存、重开并核对头部与显示。 |
| 78 | 部分 | 保存端消费版本索引；尚未逐槽闭合 SaveAs UI 到 put 路径，需继续追踪调用或调试。 |
| 79 | 部分 | 英文资源已查；缺其他语言资源与实际资源选择/识别证据，需对应语言包和环境。 |
| 80 | 阻塞 | 静态构造初值及 put 已定位，旧文档打开后是否改写未证；需跟踪旧文档加载或在 10.1 观察属性。 |
| 81 | 阻塞 | 已知原生 short 表和介质对照不是运行时验证；需运行 10.1 比较各保存选项。 |
| 82 | 阻塞 | 索引候选由静态表推导；需在 10.1 观察原生版本属性并比较输出。 |
| 92 | 部分 | 10.8 文本资源、short 表、条件占位与回填调用链见[第三轮](mxd-round3-version-object-20261117.md)；+0x5c 内部语义及完整尾部仍未定，不应把 10.1 规则当作 10.8 证明。 |
| 93 | 阻塞 | 需 10.1 和 10.8 合法运行环境及受控文件互相打开、保存、重开；不能从头部推定兼容性。 |
| 104 | 部分 | 同一对象表跨流保留，非空对象体仍由具体类 Load 决定；必须逐类闭合，不能对未知类猜长度。 |
| 105 | 部分 | 10.1 与 10.8 SetStream 静态不清表，其他版本未核，类体兼容性也不能由机制一致推出。 |
| 108 | 部分 | 本轮提供 10.1 VariantStream 空对象子集；非空对象需具体类体，不能据头部直接跳过。 |
| 109 | 部分 | 本轮复核 QI 命中与否决定经 +0x44 或普通路径；嵌套包装的 QI 传播、引用表身份和原生回引仍需逐层取证。 |
| 111 | 部分 | 本轮逐指令区分 SaveObject +0x40 路径和 SimpleSaveObject 回退，并区分两个 QI 常量地址；SimpleSaveObject 的包装递归及 10.8 对应实现未闭合。 |
| 118 | 部分 | 仅证 10.1 和显式 10.8 的局部调用点；需历史版本二进制及逐字段证据，不能扩展为所有 MXD 字符串规则。 |
| 119 | 部分 | 属性按类和自身版本门变化；需历史版本逐类 Load/Save 及受控样本。 |
| 121 | 阻塞 | 10.1 `0x7ffd0950` 有替换节点值分支，正常读先查表；需能够触发原生重复登记的合法运行场景并观测后续回引，Rust 合成登记不能替代。 |

## 可用运行环境

`temp/mxd101-run` 的 bin 为空，`temp/mxd101-wine` 有 Wine 前缀/registry，但不是已确认完整 ArcMap 安装；主审在默认 Wine 前缀中也未找到 ArcMap 安装文件。`virsh -c qemu:///session list --all` 因 `/run/user/1000/libvirt` 只读无法连接；VBox 提示 `/dev/vboxdrv` 不存在。Wine 可执行文件本身不证明 ArcMap 可运行、授权可用或具备对应语言包。动态验证需可访问的已授权 10.1/10.8 安装及许可，并保留保存文件、软件版本、对比过程和观察结果。

## 验证与整合提示

`cargo test --test round4_version_object` 两项通过；本组 Rust 文件 `rustfmt --check` 通过。全仓 `cargo fmt --check` 当时在其他组的 `src/maps/map_object.rs:954` 报格式差异。全量 `cargo test` 已执行，但在其他范围的 `tests/composite_graphics.rs:967` 失败（实际偏移 151540、期望 151528），因此未完成后续测试；需要整合方按其当前修改复核。两项共享文件均未改动。
