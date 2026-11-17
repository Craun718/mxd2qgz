# AfCore 对象流包装与委派续查（2026-11-17）

本轮只研究对象体、引用表和读写委派，不修改 TODO 或公共入口。静态地址均为映像 VA。重新计算介质提取的 AfCore.dll SHA-256 为 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，SystemCore.dll 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与 [10.1.0.3035 身份基线](arcmap-10.1-object-body.md)一致。用 `objdump` 新查两 DLL 的指令、导入表和虚表，并复核仓库 `temp/rea-decompile-20261117/AfCore/functions/` 中 Ghidra 桥接工具 6.9.0 / Ghidra 12.2 的函数导出；这些是静态证据，非原始源码或运行观察。未更改共享 Ghidra 桥接工具 服务器目标，未使用 `/tmp`。

## 包装接口和对象值访问的区别

1. SystemCore 对象读头 `0x7ffd0232..0x7ffd02a5` 在插表调用后分配 0x1c 字节，以当前 ObjectStream 接口 `edi` 调用导入的 AfCore VariantStream 构造函数，之后仅覆盖**对象基址**的虚表为 `0x80071220`，把对象基址 +0x10 交给 IPersistVariant 的 `Load`。构造函数 `0x68a7a2c0` 的 `0x68a7a337..0x68a7a352` 设置 +0x10 接口虚表 `0x68c6dc3c`，并将传入指针保存在基址 +0x18、非空时 AddRef。原记录中 `0x80071220` 称为 VariantStream「接口虚表」容易误解：它是这里的对象基址虚表，交给 Load 的 +0x10 接口使用的是另一张表。
2. 新核对 `0x68c6dc3c` 表：QI 为 `0x68a7a280`，Read 为 `0x68a7bc50`，Write 为 `0x68a7bc70`。QI thunk 从接口 this 的 -8 取构造函数设置的基址指针，再调用**该对象基址**虚表首槽；SystemCore 的基址首槽 `0x8005afb8` 经 IAT `0x8006b2d4` 导入 AfCore `VariantStream::InternalQueryInterface`，即 `0x68a7bbc0`。这不是对底流接口直接调用 QI。
3. `0x68a7bbc0` 比较 `0x68c6bda4`（`67a4d650-69e0-11d2-8500-0000f875b9c6`，IVariantStream）和 `0x68c6bdf4`（`ecc43c55-0148-4ec1-bf87-b9a183c5dc98`，IDocumentVersion），未匹配进入 `0x68ad8050`。后者仅比较三个本地 IID 常量 `0x68c6bc2c`、`0x68c6bc5c`、`0x68c6be64`，其余走 `0x68ad813a`，置空并返回 `0x80004002`。本轮核对 AfCore 的 IObjectStream 常量 `0x68c6dca8` 为 `18a45ba7-1266-11d1-86ad-0000f8751720`，与这五个常量不同。对上述**标准 SystemCore 包装对象**，接口 QI IObjectStream 的静态路径因此不支持旧文档「包装流 QI 转发到底层 ObjectStream」的说法；不排除其他实现或对象覆盖不同虚表。旧 [对象流调查](mxd-parse-agent-object-stream-20261117.md)第 88 行的机制解释应以此修正。
4. 但这不意味着标准 Variant 值路径丢失对象表。`0x68a7bc50` 从接口 this +8（对象基址 +0x18）直接取原始 IStream，作为参数传给全局 `Read 0x68a7b9a0`；Ghidra 桥接工具 导出的 Read 对 VT 9/13 将该**底流**传给 `LoadObject`。`0x68a7b6d0..0x68a7b72c` 对收到的底流 QI IObjectStream，命中调用 +0x44。由于构造实参是原 ObjectStream 接口，正常 COM QI 成功时，嵌套读取回到同一 ObjectStream 的读表；这是由调用及数据流作出的静态推论，不是对 QI 成功或对象身份的运行观察。写侧 `0x68a7bc70` 同样传底流给全局 Write，后续 SaveObject 先 QI、命中 +0x40。结论限标准 Read/Write 调用，不覆盖对象绕开这些方法自行传递接口的场景。

## SimpleSaveObject 的新指令核对

`SaveObject 0x68a7b640` 对输入流 QI IObjectStream；返回指针非空时调用 +0x40，否则调用 `SimpleSaveObject 0x68a7af70`，参数 `writeHeader=1`。不能把后者内部的某分支直接称为 ObjectStream 写槽。进一步核对：

- `0x68a7b004` 的 `0x68a7c1d0` 使用 IID 地址 `0x68c6dcb8`，不是 `0x68c6dca8`；前者的字节及既有 IID 记录对应 IDocumentVersion，**不是 IID_IUnknown**。因此 [对象流调查](mxd-parse-agent-object-stream-20261117.md)第 105–106 行把该查询称为 IUnknown 的解释有误。若流上该查询成功，函数使用所得接口 +0x10，再尝试对象的 `0x68a7c240` 接口；命中时 `0x68a7b069` 直接返回 `0x32`，不能把该路径简化为普通 IPersistStream Save。
- `0x68a7b099..0x68a7b0e8` 的第一组候选在检查标志为零后取出接口并递归调用 `SimpleSaveObject`，原流及原第三参数继续传入、第四参数强制为 1；递归返回值经 `edi` 保存并在清理后返回。`0x68a7b141..0x68a7b24a` 的另一组读取 VARIANT 后，VT 13 直接传对象、VT 9 经辅助 QI 后传对象，其他类型传空指针，均递归相同函数且第四参数为 1；返回值经 `ebx` 保留。这里的分支条件、接口名和具体实现的副作用仍需结合接口定义和调用点逐条验证，不能把递归直接认作经过 SaveObject 的 +0x40 分派。
- 后续 `0x68a7b280..0x68a7b47c` 还涉及对象上的 IDocumentVersion/IVariantStreamIO、可选头和新包装，分别调用不同虚表槽；若 `0x68a7b2ce` 调用失败则清理并返回 E_FAIL。`0x68a7b530` 读侧、各子调用失败与输出对应关系仍未完整闭合；没有足够证据把整条委派链标为完成。

## 待办判断与验证边界

| 当前 TODO | 本轮判断 | 还需证据 |
| --- | --- | --- |
| 108 跨流 ObjectStream 对象体 | 不可勾 | 逐类体及未知类边界；无通用长度不能安全跳过。 |
| 112 VariantStream 对象体 | 不可勾 | 非空 IPersistVariant 类的具体体及宿主；真实或受控 10.1 文档及运行对照。 |
| 113 VariantStream 引用表 | 不可勾 | 标准包装路径有新的 10.1 静态数据流证据，但实文件键序、对象身份和原生回引未验证；其他包装和 10.8 未覆盖。 |
| 115 SimpleSaveObject 委派链 | 不可勾 | 读侧 `0x68a7b530`、全部递归及错误传播和 10.8 对应路径需继续核对。 |

本轮没有新增解析规则或测试，因证据尚不足以安全扩展非空对象体。验证为 DLL 摘要、上述 objdump 字节与指令、既有 Ghidra 桥接工具 导出交叉检查和 `git diff --check`；未运行 ArcMap 10.1 或 QGIS，未人工核对图层、样式或转换结果，也没有把构造输入或静态推论当成运行取证。`/tmp` 所在 tmpfs 仅剩约 66 MiB，未启动需要临时数据库的新 Ghidra 会话。
