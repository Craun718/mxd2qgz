# 10.1 Variant 对象分支：QI 入口复核（2026-11-17）

本轮从 `IPersistVariant` 的实际 IID 及 QI 发起点追查，不把 WorkspaceName 的 PropertySet 值当作 VariantStream 宿主，也不修改 TODO。**尚未找到可证明的非空具体宿主、类体或可接入的读取链**；以下是新增静态证据与对旧函数归属的修正，不能代替 ArcMap 运行观察。

## 身份及方法

本轮重算 10.1 提取文件 SHA-256，并以 PE 版本资源核对 FileVersion/ProductVersion 均为 `10.1.0.3035`；文件摘要绑定静态地址，版本资源声明不等于签名认证：

| 文件 | 原始名称声明 | SHA-256 | 映像基址 |
| --- | --- | --- | --- |
| AfCore | `AfCore.DLL` | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` | `0x68a70000` |
| SystemCore | `SystemCore.dll` | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` | `0x7ffb0000` |
| afUILib | `AfUI.DLL` | `35d41cf850cbe13c202cca84441cee05db487bc54421ab9d5012dc0b1cad7da4` | `0x10000000` |
| GdbCore | `GdbCore.DLL` | `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4` | `0x708d0000` |

依据 PE 原文件字节、`objdump` x86 指令，交叉参照已有 Ghidra 桥接工具 6.9.0/Ghidra 12.2 AfCore 全函数导出（`temp/rea-decompile-20261117/AfCore/functions/`，伪代码不是原始源码）。IID 在映像中的扫描只用于定位候选，单个 QI 调用既不是接口实现的证明，也不是宿主存在性的证明；独立 CLI 和隔离补证的结果见文末。

## 对旧辅助函数归属的修正

在 AfCore PE 原始字节中按 RVA 读取 16 字节并以 COM GUID 内存序解码，再对照 QI 辅助函数中的 `push`：

| 辅助函数 | 指令压入的 IID 地址 | 实际 IID | 结论 |
| --- | --- | --- | --- |
| `0x68a7c320` | `0x68c6dc88` | `{67A4D651-69E0-11D2-8500-0000F875B9C6}` | **查询 IPersistVariant** |
| `0x68a7c390` | `0x68c6dc78` | `{00000109-0000-0000-C000-000000000046}` | 查询 IPersistStream |
| `0x68a7c0f0` | `0x68c6dcd8` | `{C9E7A782-3DDC-411F-8A66-908C579F123D}` | 不是 IPersistVariant；后续由 RTTI 确认为 [IDocumentVersionSupport](mxd-terra-round10-iid.md) |
| `0x68a7c160` | `0x68c6dc98` | `{BB75DA81-8DFE-4122-A9C2-86F07C539AD5}` | 非 IVariantStream；后续确认为 [IDocumentVersionSupportGEN](mxd-terra-round9-iid.md) |
| `0x68a7c1d0` | `0x68c6dcb8` | `{ECC43C55-0148-4EC1-BF87-B9A183C5DC98}` | 查询 IDocumentVersion |
| `0x68a7c240` | `0x68c6dcc8` | `{4BF42172-9DCE-4E03-A974-6C161F94293A}` | 不是 IPersistStream；后续由 RTTI 确认为 [INoPersistInArcGISDocument](mxd-terra-round10-iid.md) |

[旧对象流调查](mxd-parse-agent-object-stream-20261117.md)中把 `c0f0/c160/c240/c320/c390` 分别归为 `IPersistVariant/IVariantStream/IPersistStream/IDocumentVersion/IVariantStreamIO` 的说法不成立，须以本表实测 IID 为准；这不改变其余对象流逻辑自动成立与否，涉及这些标签的其他推断须重新核对。这里的函数向传入对象**发起** QI，不是对象自己的 QI 实现。

## 普通流上的读取和写入分派

`0x68a7b530` 对同一个非空对象先调用 `0x68a7c320`，再调用 `0x68a7c390`。`0x68a7b598..0x68a7b5dc` 检查前者结果，非空时以传入流构造 `VariantStream`，将包装对象 `+0x10` 接口传给查询所得对象的虚表 `+0x10`；`0x68a7b62a..0x68a7b637` 是 Variant 接口为空时用第二接口的 `+0x14`。若两种接口都没有，后者解引用的行为不能擅自补成已知的安全错误路径。此路径证明普通流也有 Variant 优先分派，**不证明任何特定类实际命中**。

`SimpleSaveObject 0x68a7af70` 的 `0x68a7b280..0x68a7b2b4` 同样查询 Variant 与 Stream，并先判断 Variant。命中后 `0x68a7b2c7..0x68a7b2ce` 调该接口 `+0x0c`；失败在 `0x68a7b327` 返回 `0x80004005`。成功时按调用者标志处理输出头，随后 `0x68a7b379..0x68a7b3b3` 构造包装并调 Variant 接口 `+0x14`，保存其返回值；未命中时 `0x68a7b43f..0x68a7b476` 检查并调用 Stream 接口 `+0x18`。这里只闭合上述局部分派，`+0x0c` 产物及全部可选头、递归和错误路径仍需逐项分析，不将其等同完整序列化格式。

## 新候选排除与接线边界

AfCore、SystemCore、MappingCore、MappingCoreLib、Map、FeatureLayer、GdbCore、DisplayCore、Layer、LayerLib 的本地提取文件中，完整 `IPersistVariant` IID 字节各有命中；对该 GUID 在各 PE 中的 VA 做四字节绝对地址匹配，每个文件各只有一个命中，指令检查的 MappingCore `0x798a4310`、GdbCore `0x709fa3c0`、DisplayCore `0x6eb75bd0` 和 SystemCore `0x7ffd0be0` 都是对传入对象的 QI 辅助入口。afUILib 的 IID 在 `0x101270cc`，两处引用 `0x10034513`、`0x100351d3` 也都是 QI 辅助函数；其中后一辅助函数在 `0x1002b202` 被调用，传入的是外部参数，尚未证明对象具体类。

GdbCore `0x709fa3c0` 的唯一已分析调用者为 `0x709fa494`，传入值来自调用者参数 `[ebp+8]`；调用者更远的来源及实际命中仍未知。此项另经隔离 pyghidra 对复制的 GdbCore 工程反编译和引用查询，反编译器未还原 QI 参数，IID 的确认仍以 `0x709fa3d0` 压入地址及 PE 原字节为准。其他扫描文件是候选定位，不作为本轮新增版本和身份结论。

上述是对**已提取文件中完整 IID 字节及直接绝对引用**的有限静态扫描，不排除间接引用、继承/聚合、不同编码、未提取模块或误识别；不能据此声称整个 10.1 不存在接口实现。下一步应追踪外部参数来源、扩充已核验版本的实际二进制，找到对象自身 QI 成功的完整分支，关联其工厂/CLSID、Load/Save、实际包含 VT 9/13 的字段，以及对象流会话表与嵌套边界。只有证明这些之后才能决定在 `variant_object_branch` 增加明确上下文和读取分派。

本轮未改 Rust：该模块没有流上下文与会话引用表参数，不能按非零 CLSID 猜测对象体长度，也不能将普通流的无键头套用于带键 ObjectStream；现有未知类停止行为保留。缺少实际类与流上下文时，构造一个“非空对象成功”测试会把假设写成规则。

独立 Ghidra 桥接工具 6.9.0 CLI 指向 AfCore、JDK 25、Ghidra 12.2，使用仓库内独占 `TMPDIR`；启动分析达到 330 秒期限后返回 `provider_timeout`，没有产生本轮反编译证据。随后使用 `GHIDRA_INSTALL_DIR=/opt/ghidra`、独立配置和复制工程运行 pyghidra，初次因 X11 不可用失败，设置 Java headless 后成功取得上述 GdbCore 函数及引用。没有切换共享 Ghidra 桥接工具 目标。忽略目录中的原始诊断可能含本机路径，分享前须脱敏。

## 可勾范围及验证限制

可单独记录：**10.1 AfCore 的这六个 QI 辅助函数的实际 IID 字节核对，以及普通流 Variant 优先读写分派的上述局部指令链**。不能勾选 VariantStream 非空对象体、宿主或引用表的完整 TODO；本轮没有证明具体类体、业务宿主、对象身份或实际字节。ArcMap 10.1 运行环境暂缺，未在 ArcMap 或 QGIS 中进行人工验证，合成测试亦不能代替运行验证。

`cargo test --lib variant_object_branch`：6 项通过、178 项过滤；Cargo 全局缓存清理有权限警告，测试通过。本轮没有 Rust 修改，未运行全量测试。
