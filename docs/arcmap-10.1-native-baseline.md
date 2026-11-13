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

基线建立时没有比较函数实现，也没有验证任何 10.1 的对象头、字符串、Version 或 `/Maps` 规则；随后开展的局部对照见下节。10.8 研究结论和读取器仍只标注 10.8。完整文档打开链、类 GUID、虚表、错误处理、版本字段及跨版本差异均待确认。ArcMap 10.1 与 QGIS 的人工打开和视觉核对均未进行。

## 10.1 Version 局部读取和构建号（2026-11-13）

再次核对 PageLayout.dll 的 SHA-256 与上表一致。以下为 REA 6.8.0、Ghidra 12.1.4 静态反编译，分析配置摘要为 `e763201e522a4b123fb330eede09834c44ddf2bb90702e50eea219801be95539`；地址绑定上述文件，不是运行时地址。证据包保存在忽略目录 `temp/arcgis/arcmap101-version-local-evidence.json`，含本机路径，分享前需脱敏。PDB 加载状态未知，虚函数类型和原始名称可能不完整。

- `0x7b70a8e0`（`ev_7ff48670ca2f323b29dd5a989d0667c687994ef8d4d95e9f3fd73a00f8769566`）在状态允许时将输出初始化为 `5`，获得已有存储或调用 `StgOpenStorage`。取得相关接口后，通过存储虚表 `+0x10` 打开 `Version`，连接一个流对象，调用 `String::Read`，随后从底层流调用 `AfStreamRead` 读取 2 字节到初始化为零的 short。该有符号 short 小于 `0x12` 时输出变为 `3`；之后调用接口 `+0x28(0x12)`。两个读取调用的返回值在此处未被检查。未打开 Version 或未取得接口时不能据此推断其内容。
- 静态直接调用者 `0x7b69f6b0`（调用关系 `ev_57a7348949d6962ee9bd1dc996cfa1939921e96137f010b4c74a9b1786288a7a`、伪代码 `ev_ddbfede90d4badb17f0f9688fef19cd6b550b1c3a35d3228072824d7f03da5e6`）在一个分支调用 `0x7b7098b0` 成功后，按另一虚调用的结果条件性调用该读取函数，并据其输出是否小于 4 决定后续虚调用。它没有检查此读取函数的返回值；该调用者的完整类身份和文档打开入口尚未闭合。静态调用者列表不覆盖未解析的间接调用。
- `0x7b6a53e0`（`ev_04c4d415fbbd9e11e915ee2de58e71b826106f8db74b7ad527b0596543655fdf`）操作另一条名为 `Mx Document Version Info` 的流，写 4 字节并构造 `Build Number = 10.1.0.3035` 属性。其反编译带有重启次数超限警告，后续序列须结合汇编复核。它与 `Version` 的局部读取不是同一条流，不能把构建号当作 `Version` 的字符串或 short。

与 Git 历史 `c17b324:docs/mapdocument-native.md` 记录的 10.8 `0x10076ef0` 比较，两者局部顺序均为打开 `Version`、`String::Read`、读 2 字节、以有符号 short 比较 `0x12`，且小于阈值时输出 3；**默认输出不同**：10.1 为 5，10.8 为 `0xb`。这只是局部静态行为对照，不证明字符串的 10.1 字节布局、字段语义或两版本文件兼容。10.8 亦有单独的 `Mx Document Version Info` 构建号写入分支，写入 `10.8.0.12790`。

此时尚未确认 10.1 的 `String::Read` 实现（后续复核见下节）、`Version` 写入者、short 和尾部来源、完整错误及短读行为、实际版本映射或接口归属；不得据此把已有 10.8 读取器应用于 10.1。未在 ArcMap 10.1 或 QGIS 中人工验证。

## 10.1 AfCore 字符串函数（2026-11-13）

后续复核 AfCore.dll 的 SHA-256 与上表一致；REA 6.8.0 / Ghidra 12.1.4。导出表 `ev_f4c633d630f11595222bc5b5740eb493a0518c3f5a8ae101a8ac076fcb659f6e` 将 `String::Read` 和 `String::Write` 分别定位到 `0x68ab46f0` 和 `0x68ab4680`。证据包位于忽略目录 `temp/arcgis/arcmap101-afcore-strings-evidence.json`，包含本机路径，分享前需脱敏。

- `String::Read`（`ev_09d2a18062449c0bafef450a9fad8523558c7387560e0f26ac74ef561334f01f`）先请求 4 字节作为长度；返回值非负且读取量与长度均非零时按长度分配，再读同样字节数，返回值非负且读取量非零时赋值。它未检查两次读取是否完成请求，也未显式检查长度偶数或末尾零。PageLayout 上节的 Version 分支调用了此导出，但中间流对象的读行为仍需确认。
- `String::Write`（`ev_0fece9ecedc0df634a797a98e560cfb96cfc5a063ce911573357779071bedb1b`）计算 `cb = n * 2 + 2`，写入 4 字节 `cb & 0x7fffffff`，再写 `cb` 字节；普通输出包含两字节终止零。算术溢出及掩码不应作为安全读取规则。
- 另有独立的 `ReadString` / `WriteString` 导出（分别为 `ev_8c6838cc7ff0770ea71ca4c9b2fc323da53c2a405967a0d7fdcecc7fbf1e4222`、`ev_be0ffb257d439103a9cfe48ba6c6ca183132976c2dd2ec720f2a627f5f99cedf`），其长度处理不同；不可仅按字符串名称把两组函数当成相同格式。

上述 `String` 局部逻辑与 Git 历史 `7326c98:docs/native-strings.md` 所记 10.8 分支相似，支持进一步核对严格读取子集，但不证明全部字段、全部版本或异常输入兼容。`Version` 写入调用者、ObjectStream 转发、short 和尾部依然未知；不能据此识别 MXD 版本。未运行 ArcMap，未做人工验收。

`native_string::read_string_10_1` 为明确调用点提供普通写出布局的严格安全子集：限制资源、检查完整长度、UTF-16 和终止零，失败时保持游标。它拒绝原生函数可能接受的短读及提前零；合成测试不证明任何 MXD 字段或软件兼容性，也未接入 Version 流。
