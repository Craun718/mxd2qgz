# MXD Version 与保存路径：静态复核

日期：2026-11-16。范围为 TODO 第 66–84 行中的 Version、SaveAs、语言和运行验证事项。以下地址仅适用于所列文件，静态证据不代表程序运行结果。

## 文件身份与证据

- 10.1：从 README 指定的 `Desktop/Setup1.cab` 提取 CAB 条目 `ArcMap.exe.E97001B0_1E76_4BE8_B51F_269775FE0A80`。PE x86；版本资源 FileVersion/ProductVersion `10.1.0.3035`，InternalName `ArcMap`，OriginalFilename `ArcMap.exe`，CompanyName `Esri`；SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`，与既有基线一致。
- 10.8：从本机 `temp/arcgis/unpacked/Desktop/SetupFiles/Data1.cab` 提取同名条目。PE x86；版本资源 FileVersion/ProductVersion `10.8.0.12790`，InternalName `ArcMap`，OriginalFilename `ArcMap.exe`，CompanyName `Esri`；SHA-256 `a9a963b9dffbf0cb346c67c98bce0880b7e64363410a788dccb9597cff3c75fe`。版本资源是文件身份声明，不是签名真实性验证。
- 本轮使用 Ghidra 桥接工具 / Ghidra 12.1.4。两版共 21 条 Evidence 保存在本机 `/tmp/mxd-version-luna-evidence.json`（同内容检查点 `/tmp/mxd-version-luna-checkpoint.json`）；该临时位置可能被清理，包中包含本机路径，分享前须脱敏。分析目标已关闭，会话已清理。PDB 加载状态未知；Ghidra 未解析的虚调用与异常路径不能当作不存在。

本记录更正上一轮误判：安装介质可以重新提取，10.8 CAB 也存在；仅凭先前提取目录或 Ghidra 桥接工具 会话为空，不能认定目标文件不可用。

## 10.1 Version 和版本分支

ArcMap.exe `0x466fb0` 的本轮反编译 `ev_4ac6e29e677912a04960a0289c7b8a03672c64d360677c40fd02367ed2b33f20` 显示：保存时通过虚槽 `+0x14c` 请求索引；创建 `Version`，按索引选择资源文本并调用 String::Write，然后写入经 AfStreamGetSaveVersion 得到的两字节 short；索引大于零时记录位置并写 4 字节占位。该函数很长，Ghidra 报告类型传播不收敛、移除不可达块，后续回填应结合既有基线的汇编证据，不能仅凭伪代码断定整个流末尾。

读取器 `0x4764d0`（`ev_0213c08931750571481722d0370f6abec6e5d69a78d2d4ab8b97c6efca0a62d7`）打开 `Version`、读取 String 与 short，short 大于 `0x13` 时拒绝，大于 `0x0b` 时读 4 字节并传给对象流的 `+0x54`。随后对象流被传给文档加载接口；该接口可能重新连接其他命名流，因此后续经对象流的读操作不能仅凭局部变量名称归属于 Version 尾部。既有基线将回填值解释为对象流表键计数；参考样本已知头部后无剩余字节，样本不能证明其他路径也没有尾部。

本轮进一步复核 `0x47a3f0`（`ev_d2a8255a7dbea21596b10c5b497e5cb6bc10d3521342923f6bb98ea1acac108c`）：该函数默认输出 5，除 short 比较外还有读取独立 `Build Number` 属性并解析的分支；满足其中一组版本条件时输出 3。不能把它概括为所有路径均由 `/Version` short 单独决定，也不能据此声称其结果写入 AfDoc 的保存索引。该条件的完整语义和运行分支仍需进一步核对。

## SaveAs 与 put 路径

`0x42b120`（`ev_7230475da905b46f203c2a3317f49dfa784276682bb928454b0bdcc1796eee79`）仅将输入写到接口 this 的 `+0x28`。Ghidra 桥接工具 读取 `0x545e40` 虚表（`ev_44d31ae9620c60348de197fd7b7a49130e0faa0350474b1fce392a413b14e8d2`）确认 `+0x0c` 指向该实现，`+0x10` 指向 `0x42b140`。对目标映像初始化内存查找其小端地址字节仅命中 `0x545e4c`（`ev_3c6ab0710141dbabeb812e431d324e7e6c652ef48f7d83113b154c54c43b0f16`）；这只是有界静态字节观察，不能排除间接调用、动态计算或其他模块调用。

IDocumentVersion IID 字节在 `0x554e9c` 和 `0x555854` 两处命中（`ev_04e7b66cec1fe287ce72edda05638dee0b0065ca9043b08420d03d44bf69e262`）。Ghidra 桥接工具 在后者解析到 `0x48a2a3` 引用（`ev_7a3036d87dd0885241ce0a1b32521f61fa28629d887cbe575814da7bc1242aeb`），所在函数 `0x48a250` 用该 IID 查询输入接口并保存结果（`ev_39a8528a34d9f7371020d5744edc16858418cea658be4bb77c374d179ad9b418`），已解析直接调用者为 `0x488bf0`（`ev_3a87014cd4cb008880964c399010b12b16f675968e9d4c1a3397dd9ce8a789e8`）。写入器反编译调用 `0x488bf0`，之后对取得的接口虚槽 `+0x0c` 传入保存索引；该观察有助于定位写入器向流侧传递索引，但尚不能证明 SaveAs 对话框到 AfDoc put 的路径。另一 IID 地址的空引用结果受静态覆盖限制。

`0x47a740`（`ev_3ba7402216a5aa09a26c953a583fe353d842630543a8863edde7296c8d3c87f7`）按序列化方向分派 10.1 读写，符合既有记录；其反编译存在终止调用导致的函数体覆盖限制。SaveAs UI 到 put、打开旧文档后保存索引是否被改写仍未闭合。

## 10.8 定位结果

Ghidra 桥接工具 字符串清单在 `0x55e6d0` 发现 UTF-16 `Version`（`ev_09249a3b7a24b755bf8643d4aa68f1dbc6433e1a5deca2818a447ef73d0610fc`），已解析静态引用 `0x47a770`、`0x48a8b7`（`ev_88b066f0f902e2973206a58e73d517c0be439aae9323c39d6690aa8e3e28f988`），分别位于 `0x47a250` 和 `0x48a380`（`ev_60d77c01f301ee7edc62c1eb23fa25d91645d858ac03c22d352e9115b3b94154`、`ev_25573faf0d7b37750bca26451df803181cdb3a858f96e00a424406bef5121989`）。用户要求收束时两函数的反编译调用尚未返回，因此本轮只记录候选入口，未将它们定性为读或写，也未闭合 10.8 写入规则或跨版本兼容性。

## 未解决事项与验证

TODO 第 66–84 行没有可凭本轮证据新增勾选的未完成项：10.1 未知尾部语义与跨版本差异、SaveAs UI 到 put、旧文档后的索引状态、10.8 写入规则和兼容性均需继续研究。本地化语言包的实际文本及版本识别影响仍需取得并核对相应资源。版本映射、索引输出及 Version 读写尚未在 ArcMap 10.1 中运行验证；当前未安装可运行的目标环境。不能以静态分析替代 ArcMap/QGIS 人工检查。

本轮只更新本文档，删除了上一轮未接入的 `src/version_tail_luna.rs` 草案；没有新增生产源码。此前独立草案测试及仓库 `cargo fmt --check`、`cargo test` 的结果不构成本轮原生运行验证。
