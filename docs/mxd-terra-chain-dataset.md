# FeatureDatasetName 字段调用链（2026-11-17）

本轮只研究 10.1 静态实现，沿用 [字段对位](mxd-terra-depth-names.md) 的持久化接口偏移和对象基址约定，不修改 TODO。结论适用于下列文件，地址是静态映像 VA，不是运行地址。

## 身份与方法

重新读取介质提取的 `GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`：PE32 x86，ImageBase `0x708d0000`，版本资源 FileVersion/ProductVersion `10.1.0.3035`、OriginalFilename `GdbCore.DLL`、CompanyName `Esri`；SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，与既有基线一致。10.1 esriSystem 类型库资源 SHA-256 `0f3bc6496f2665cd6b03fa0071c0077f3b6fdab8f96c8b80f02beec36d7ad7a8`。版本声明和摘要不等于签名认证。

前轮独立 REA CLI 以 JDK 25 分析同一文件时 UNIX socket 返回 `Operation not permitted`，见 [续查](mxd-terra-follow-names.md)；本轮未更改共享 REA 会话，使用 JDK 25、Ghidra 12.2 和隔离 pyghidra 对复制的既有工程重新反编译访问器与读写函数，仓库内独立 TMPDIR、配置及项目位于 `temp/terra-chain-dataset/`。另以 objdump 校对指令、PE 原始 GUID 和虚表，用 `winedump` 读取类型库；本轮单独启动 Wine 枚举时 socket 亦被拒绝，未据此声称 Wine 运行成功。临时文件可能含本机路径，分享前须检查脱敏。反编译是静态证据，非原始源码和目标程序运行。

## 第二串：IName.NameString

1. QI `0x709b6560` 将 GUID 常量 `0x70ab84e4` 与请求 IID 比较，匹配后返回完整对象 `+0x10`。原始字节为 `{4BA33331-D55F-11D1-8882-0000F877762D}`；10.1 esriSystem 类型库把该 IID 绑定到 `IName`。构造函数 `0x709b63b4` 将虚表 `0x70ad0414` 写入此视图。
2. `winedump` 对上述 10.1 类型库的 TypeInfo 71 显示：`+0x0c` 为接收 BSTR 的 NameString，`+0x10` 为输出 BSTR 的同一 NameString，`+0x14` 输出 IUnknown；名称表关联 `IName`、`NameString`，前两项具有相同成员 ID。此原生 x86 类型库槽位与虚表指针对应，未采用 64 位槽位。作为交叉参考，[Esri IName 文档](https://desktop.arcgis.com/en/arcobjects/10.7/net/IName.htm)说明 NameString 是可读写属性；该文档为 10.7，仅辅助理解，10.1 对位依据是本地类型库和二进制。
3. 虚表 `+0x0c` 指向 `0x709b67d0`，将 BSTR 赋给接口 this `+0x2c`；`+0x10` 指向 `0x70a401e0`，从同一位置制作 BSTR。换算为完整对象 `+0x3c`。`IPersistStream` 位于完整对象 `+0x2c`，Load `0x709b7c90` 和 Save `0x709b7d50` 的第二个 String 均访问持久化接口 this `+0x10`，即同一位置。因此第二串静态对位为 **IName.NameString**，不同于第一串 `IDatasetName.Name`；不可据此假设两个值总相同或为完整路径。
4. `IName` 虚表 `+0x14` 是 `0x709b6800`。它读取完整对象 `+0x4c` 的工作空间相关指针，在后续间接调用前将完整对象 `+0x38` 的第一串转换为 BSTR（指令 `0x709b68c6..0x709b68f0`）。这条静态路径不证明第二串在 Open 中被使用，也不证明间接调用在运行时成功。

## 第四串与标志：尚未闭合

第四串 Load/Save 均为 IPersistStream this `+0x18`，即完整对象 `+0x44`。构造/析构和持久化访问已核对，但本轮没有找到能与该字段对位并具备可靠属性名称或业务含义的访问器或实际调用；仍原样保存，不命名。

标志 Load/Save 均为 IPersistStream this `+0x1c`，即完整对象 `+0x48`。QI 匹配 `{3C5BBE8C-C9C3-11D1-AA7C-00C04FA33A15}` 后返回对象 `+0x20`，虚表 `0x70ad0498` 的 `+0x14/+0x18` 为 `0x709b74d0/0x709b7510`，访问接口 this `+0x28`。getter 仅在字节等于 1 时返回 `VARIANT_TRUE`，setter 仅对输入 `-1` 写入 1；流读取和写出均保留单字节。当前仍未证明该 IID 对应的接口/属性名，不把规范化后的布尔值替代原始字节，也不推测业务用途。

## 结论与限制

可单独闭合「10.1 FeatureDatasetName 第二串静态对位 IName.NameString」子项；现有「FeatureDatasetName 字段业务语义」整项仍不能勾选：第四串和标志含义未知，且该结论不包含真实运行行为。版本 0 的真实对象、目标软件交互及 ArcMap 10.1/QGIS 人工目视验收均未验证。代码只补充已证实的中文注释，公共字段名、原始值和解析行为保持不变。

验证：`cargo fmt --check`、`cargo test` 和 `git diff --check` 均通过；全量测试中依赖本地安装介质的用例按既有设置忽略。本轮只修改注释，没有新增解析规则或测试。测试不能替代目标软件运行与人工验收。
