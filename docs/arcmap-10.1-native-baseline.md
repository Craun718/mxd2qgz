# ArcMap 10.1 原生文件基线

记录日期：2026-11-13。此记录是静态文件取证和入口定位，不代表运行观察或 MXD 读取规则已经验证。

## 来源与可复核方法

本机 `README.md` 所列 Desktop 介质中，`setup.msi` 的 SHA-256 为 `55813a637950acbe397b896485fc293dc71d65acf9d27a58ec393deee5b1f775`，`Setup1.cab` 为 `6833035af901b4b46fe508485cdd5d26534bc27630c512f8bf87c5db417ddfe3`。`Setup.ini` 声明产品版本 `10.1.3035`；这是安装配置，不是下表 DLL 自身的版本证据。未检查 MSI File/Media 表，故下表仅确认 CAB 条目及其解压结果，不声称已核实安装目录或 MSI 组件映射。

使用 `cabextract -l Desktop/Setup1.cab` 列出以下条目；同样检查 `Setup2.cab` 的目标名称未命中。对 `Setup1.cab` 使用 `cabextract -F '*Core*.dll.*'` 和 `cabextract -F 'PageLayout.dll.*'` 分别提取到两个独立临时目录；逐文件 SHA-256 均一致，文件大小均与 CAB 列表一致。使用 Python pefile 2024.8.26 读取 PE 标头和版本资源：五个文件均为 PE x86（Machine `0x14c`），固定文件版本为 `10.1.0.3035`；版本字符串中的 FileVersion 和 ProductVersion 也为 `10.1.0.3035`，CompanyName 为 Esri、ProductName 为 ArcGIS。各文件的 InternalName 与下表模块名一致，OriginalFilename 为对应模块的 DLL 文件名。这是资源内的身份声明及字节身份，并非签名真实性验证。

| 模块 | Setup1.cab 条目后缀 | 大小（字节） | 解压文件 SHA-256 |
| --- | --- | ---: | --- |
| AfCore.dll | `017F52BF_20D0_487A_A583_436DA6406CFF` | 2594304 | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` |
| SystemCore.dll | `017F52BF_20D0_487A_A583_436DA6406CFF` | 1134080 | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` |
| PageLayout.dll | `6BF03936_8AC4_4632_8FB2_A6A34A19655A` | 715264 | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` |
| MappingCore.dll | `93A93CDE_D64F_4B7A_A2BA_829DB1A9C7E4` | 2586112 | `ac6f99a6345bab8834bcd8824e50bf93c5ab153de1ce2889fe9e613f080921e7` |
| MappingCoreLib.dll | `93A93CDE_D64F_4B7A_A2BA_829DB1A9C7E4` | 1375744 | `c2bf81dcfcb65473ce35b953c4f4b76392a594085460c299326cc20bed59986a` |

完整 CAB 条目名为模块名加 `.` 和表中后缀。原文件和安装介质未纳入仓库；本机临时提取位置为 `/tmp/mxd-arcgis101` 和 `/tmp/mxd-arcgis101-recheck`，可能被系统清理。

## REA 入口与证据

REA 6.8.0 的 `inspect_native_exports` 对上述五个已核对文件的 PE 导出表覆盖均为 complete，依表格顺序证据 ID 为 `ev_f4c633d630f11595222bc5b5740eb493a0518c3f5a8ae101a8ac076fcb659f6e`、`ev_f217002ad0a532cebdf52a50a64ed4b54d099458dc028405f6ae2c710d0096a4`、`ev_698bbaf2463e636c2a63ccba64b4a1c5b8bd110683a74751f0d7d6de39fd78bc`、`ev_6933687f93119067692ae6e9aa50aa2411f54f76dd7dfc025aea9bf367060069`、`ev_a2d4f52f4c26296aa31ee1bbd989e70bc373240a40e4457959632a981f2c92ca`。地址是各 PE 的静态映像虚拟地址，绑定上述文件摘要，不是运行时地址。

- PageLayout `DllGetClassObject` 在 `0x7b698800`，SystemCore 在 `0x800413b0`，MappingCore 在 `0x799fdf80`；这些是后续查找 COM 类工厂的入口，不证明具体类的实现归属。
- AfCore 导出 `AfStreamGetDocumentVersion`（`0x68a84760`）、`AfStreamGetSaveVersion`（`0x68a847f0`）、`CreateMemoryStream`（`0x68a7bad0`）以及 VariantStream 的 `get_DocumentVersion`（`0x68a7bd00`）和 `put_DocumentVersion`（`0x68a7bc90`）。名称和签名提供版本与对象流调查入口，尚未关联 OLE `/Version`。
- MappingCoreLib 导出一个以 `IMapDocument` 为参数的 `CMSDCreator` 构造函数（`0x79ba5c30`），仅说明导出签名中的接口候选，不证明 MXD 打开路径。
- Ghidra 12.1.4 对 PageLayout 的字符串清单中有 `MapDocument`、`ClassFactory<MapDocument>`、`IMapDocument` 等名称（`ev_5957d65dc4cef64677352bb634946a8588e7bdd5f4c38d9382696ccddf2d9e2d`），以及 `Version`、`Mx Document Version Info` 和 `IDocumentVersion` 等名称（`ev_d2dedda45f9fd98b62bf6c1000af959ac256f1c2ab868eb285fb4a215283b87c`）。这些是候选线索，不单独证明调用关系。
- 静态引用把 `Version` 字符串 `0x7b721234` 指向函数 `0x7b70a8e0` 内的 `0x7b70aaa0`（`ev_78ec5a5525050cb275475bd13672d2c099fba79e7a871ec3a94cc966278256d1`、`ev_9f14cb274251c77b6c7612b8ffb600afbd0a2ff644f5bbea85774e8c163559ba`）；`Mx Document Version Info` 的 `0x7b71dc38` 被函数 `0x7b6a53e0` 和 `0x7b708650` 引用（`ev_8701839e4854bd95c0191dd66a4871d30e71ad9636c5106921c79e2e451d73ff`）。后续须检查具体代码和调用上下文，不能仅凭字符串断言读写行为。

证据包在本机忽略目录 `temp/arcgis/arcmap101-baseline-evidence.json`，包含本机路径，分享前应检查脱敏；REA 已关闭会话。Ghidra 只覆盖已定义的字符串及已恢复的函数，间接调用可能遗漏，PDB 加载状态未知。这些观察不代表执行过目标软件。

## 未解决事项

10.8 研究结论和读取器仍只标注 10.8；本轮没有比较函数实现，也没有验证任何 10.1 的对象头、字符串、Version 或 `/Maps` 规则。下一步从 PageLayout 类工厂和上述引用追踪 MapDocument 的 Open/GetVersionInfo 与实际流操作，再沿 SystemCore 的 ObjectStream 接口和 AfCore 的版本辅助函数逐项复核。类 GUID、虚表、错误处理、版本字段及跨版本差异均待确认。ArcMap 10.1 与 QGIS 的人工打开和视觉核对均未进行。
