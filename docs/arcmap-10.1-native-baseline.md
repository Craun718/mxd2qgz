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

## 10.1 SystemCore 接口候选（2026-11-13）

复核 SystemCore.dll 的 SHA-256 与上表一致。REA 6.8.0 / Ghidra 12.1.4 搜索 10.8 研究记录中的接口 GUID `18a45ba7-1266-11d1-86ad-0000f8751720` 的内存字节，在已加载块找到 `0x80070fb0` 和 `0x8007123c` 两处（`ev_712f93100d5351914d87edbed5ae74d2b31b3dee650382be3b65875b4cd0e22c`）。静态引用分别指向 `0x7ffcef90` 和 `0x7ffcf3f7`（`ev_9a6535d653026afef9108d944ad6981796d5706292b49f3888a8b7e6c9307fab`、`ev_2ae2363dfa066dc275fe3a1384eb403b281f37d1f890f3ada28d64259564ab3b`）。

反编译 `ev_9f7b307745651c16e84d3acce94d1db216033768f5e8bf496b45d2999c3e2e28` 显示前者所在函数创建实例并用该 GUID 查询接口；`ev_6c3bc7a592db323f0f4a39001ab7e65f95f1d2871297c61897bb3e36c5556fc5` 显示后者所在函数对该 GUID 的匹配分支返回对象基址 `+0x10` 的指针。仅凭这两处尚不能确定对象身份、构造后的实际虚表、SetStream、读取转发或持久化规则；还需追踪类工厂和构造过程。静态引用可能遗漏间接访问。证据包在忽略目录 `temp/arcgis/arcmap101-systemcore-guid-evidence.json`，包含本机路径，分享前需脱敏。

## 10.1 ObjectStream 构造、转发和局部对象头（2026-11-13）

再次核对五个文件 SHA-256 均与上表一致。以下为 REA 6.8.0 / Ghidra 静态分析，地址和结论仅绑定上表 10.1.0.3035 文件；证据包为忽略目录 `temp/arcgis/arcmap101-systemcore-followup.json` 和 `temp/arcgis/arcmap101-pagelayout-followup.json`，含本机路径。无目标软件运行观察，PDB 加载状态未知。

- SystemCore `0x80047930` 分配对象并调用 `0x7ffcf030` 构造（`ev_9bc60c6f4ec67af8bb83aa60f0c84497fc7c9d972d26646171497ee9711e1d93`、`ev_1c033973c9ad90c7195b2bfcb5c9b3e5c7ec2271ae350ed0f452e7f96d68d972`）。构造函数在基址 `+0x10` 写入虚表 `0x800710dc`，在 `+0x14` 写入 `0x80071140`，并把基址 `+0x54` 初始化为 5。此前 GUID 匹配分支返回基址 `+0x10`。`DllGetClassObject` 跳至 `0x80054220`，后者遍历类表并间接调用工厂；尚未把该类表项与构造入口完全闭合。
- 静态映像中 `0x800710dc + 0x0c` 指向 `0x7ffcf520`：该函数对接口指针 `+0x10` 的底层流调用 `AfStreamRead`，流为空时返回错误（`ev_c92caa49d5ae7b26e9b882b07f09ee7343cef079bbbfb9136bf30a2f8ad24084`）。`0x80071140` 的 `+0x0c` 和 `+0x10` 分别指向 `0x7ffd0610`、`0x7ffd0630`，写入和读取接口指针 `+0x40`，即基址 `+0x54`；这只是内存属性，不证明磁盘版本映射（`ev_9bdf4e90472f1cab336dd2150f277f949c61b3a9df84b37aba2848585c22d2f6`、`ev_2d3a8eb592c8597a227bfe3e74c588d52da6a754ee6c7d649de00977fa09e059`）。
- 接口虚表 `+0x44` 指向 `0x7ffcff70`（`ev_9de2daa6a8b5b655f72ff271eb5264acc7ae71b76a574db85f3567e44c6d0162`）：从底层流请求 16 字节 GUID，与零 GUID 比较；非零时请求 4 字节键，查询对象表，未命中时创建实例、登记并进入普通或 VariantStream 持久化分支。局部结构与既有 10.8 对象头研究一致；读取量未严格检查，具体失败恢复、替换分支及对象体边界尚未知。析构 `0x7ffcf200` 遍历两张表并释放对象（`ev_ec7a972576ac3aeacffee0a2c5915759a49608f1bde254b83d9f6056f0d1a7f1`）。
- PageLayout `0x7b70a8e0` 使用类 GUID `043731d0-a7cf-11d1-8bd1-080009ee4e41` 创建流对象、调用接口 `+0x38` 连接 Version 流，再经该对象调用 String::Read，直接从底层流读取 short（`ev_1e02d5298f0482880a788a40d804effcee0ea0bcd35375ce75bd60c9bec5d13c`）。同一 GUID 也出现在 SystemCore `0x7ffcef40` 的创建调用；但类表映射、连接槽位及实例身份仍须闭合，不能据此认定 Version 流全部字节规则。`0x7b7098b0` 在打开存储后调用 `0x7b708d00`；后者大型反编译的值流被截断，完整文档打开和写入路径仍待逐分支复核。

`object_stream::read_header_10_1` 仅对调用者已确认的对象起点提供严格完整读取：零 GUID 消耗 16 字节，否则保留原始 GUID 与 4 字节键；未知对象体不读取，失败不推进游标。与原生可能接受的短读不同；固定宽度没有外部长度字段，偏移溢出由 `Reader` 的检查保护。它不接入 `/Maps`，不识别未知类或引用，更不证明转换能力。ArcMap 10.1 和 QGIS 的源工程/转换结果目视对照仍未进行。

## 10.1 版本 IID 和写入候选（2026-11-13）

再次核对 AfCore、SystemCore、PageLayout 三个 DLL 的 SHA-256 与基线一致。以下是 REA 6.8.0 / Ghidra 12.1.4 的静态观察，地址只适用于上表 10.1.0.3035 文件。三个证据包分别保存在忽略目录 `temp/arcgis/arcmap101-version-iid-afcore.json`、`arcmap101-version-iid-systemcore.json` 和 `arcmap101-version-write-candidates.json`；含本机路径，分享前须脱敏。PDB 加载状态未知，间接调用和实际运行情况未覆盖。

- AfCore `AfStreamGetDocumentVersion`（`0x68a84760`）先将结果初始化为 5，对非空输入调用 `0x68a7c1d0`；取得接口后调用其虚表 `+0x10`，最后返回结果。汇编显示 `0x68a7c1d0` 用 `0x68c6dcb8` 处的 IID 查询输入接口（`ev_ab79989d80acf259ea479e4f00b6b62b5c87ebc7eda573976e3e5d343ef1d909`、`ev_2a16cd07d8b4514456cf763585d21960a60d01ac86c9dc8df1fd36e36c87cf4e`）。该处 16 字节为 `55 3c c4 ec 48 01 c1 4e bf 87 b9 a1 83 c5 dc 98`，即 `ecc43c55-0148-4ec1-bf87-b9a183c5dc98`（`ev_ae3a6b672b96a8b912de6ceb924a6782b515f4ae3afc49a98e39c125008f23cb`）。
- SystemCore 对同一字节的加载块搜索命中 `0x8006db38` 和 `0x80074b58`（`ev_4d910cf97bc32c6b91e2be2b5a67c832a32c5f33fdef4461488d037251e9050f`）。前者的四个静态引用均在 GUID 比较分支中：`0x7ffcf3f0` 匹配后返回对象基址 `+0x14`，与上节 ObjectStream 构造后的 `0x80071140` 属性虚表相连（`ev_823f560dec0dda3aba675063d6346d30dc5f5799e30dafcb1edc7c6d97a49b71`）；`0x7ffc5a90`、`0x7ffcbb50`、`0x7fff9ee0` 均返回各自对象基址 `+0x18`（`ev_12f86a8317173f4f4591575e033c1a6d97217fd5ba5c6b56cd0cda978d8983f9`、`ev_6d11a5edfe85f142b68b6b9d8bf9c316d6c47269ce99dc4059886b359b722345`、`ev_e663ad7bc7fb9e5f802185bcd2c18a96806af44b592959a863fb38d42c6de1b2`）。后三者的类名、构造后虚表和实际实例尚未闭合，不能认为只有 ObjectStream 实现此接口。第二处引用 `0x7fffb920` 是接口查询，不是实现证据（`ev_528986a60a4753e48cc01803d4367d878a080e57785cc9185296bc0c92149fbb`）。静态引用不构成全安装包实现者清单。
- PageLayout 中对 `String::Write` 导入的已解析直接调用者为 `0x7b691b50`、`0x7b692ce0`、`0x7b692e00`、`0x7b6be580`（`ev_d50028d7601beda089756d254d84cc0f91ad3b7ec38ba47b141f4770f7bc9eb7`）。前三者属于以 `Metadata` 为名创建流的路径及其分支（`ev_10a4bc2d2655aa770b1cf2192c699231922a0fc8a6914026133936f8c3f33411`、`ev_50db9824972419e6d338bf0a36926fdd757bdd266d415247a8999c05a2bb66f2`、`ev_b95185c5df4a85eec7ce1ceb22f89bc3eb5dd749133be47a98933e1d7656caa8`）；第四者在对象保存序列中循环写字符串和其他字段（`ev_b950132320e3cd511970f807385233a69e6568ac92a663ca6d6201b7232a7aea`）。这些观察未将任何调用闭合到 `Version` 写入；直接调用者清单也不能排除其他模块或间接调用。

本节调查时尚未找到 `Version` 写入者；后续追踪 ArcMap.exe 的结果见下节。目标软件运行行为和 ArcMap/QGIS 人工目视检查均未验证。

## 10.1 ArcMap Version 头部读取与写入（2026-11-13）

从 README 所列 Desktop `Setup1.cab` 提取 `ArcMap.exe.E97001B0_1E76_4BE8_B51F_269775FE0A80`。PE 机器类型为 x86，版本资源声明 FileVersion/ProductVersion `10.1.0.3035`、InternalName `ArcMap`、OriginalFilename `ArcMap.exe`、CompanyName `Esri`；SHA-256 为 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`。这是文件身份声明和摘要，不是签名真实性或运行验证。REA 6.8.0 / Ghidra 12.1.4，地址为该静态映像虚拟地址；证据包 `temp/arcgis/arcmap101-arcmap-version.json`、`arcmap101-arcmap-version-followup.json`、`arcmap101-save-version-table.json`、`arcmap101-objectstream-count.json` 位于忽略目录，包含本机路径，分享前须脱敏。

- ArcMap 保存入口 `0x466fb0` 创建 `Version` 流，向连接的对象流调用 `String::Write`，再向底层流写 2 字节 short。版本索引由虚调用提供，初值为 5；文本由资源 ID 根据索引选择：0、1、2、3、4、其他分别为 `0x14d`、`0x173`、`0x186`、`0x11a`、`0xe804`、`0x193`（`ev_0eed88af68bd8247c1b0dd3ff7b9746c42b71db0706a30785bed8d3b3b7af7b3`）。资源句柄由 `ESRILocale::LoadLocalizedRes` 获取，失败时使用传入模块句柄（`ev_b02d3d1104e69ced5a723bf7805378401b5ef0b5ddb831d9f49dec8bae88b944`）；不能仅根据一个文件的资源表假定所有运行环境的具体文本。输入索引大于 0 时，保存代码记住当前位置并先写 4 字节零，后续回到该位置，用对象流 `+0x5c(0, &value)` 的结果回填。反编译把长函数的跳转拆成多个函数，回填汇编位于 `0x46bfe4..0x46c07c`；原保存函数和后续片段分别见 `ev_0eed88af68bd8247c1b0dd3ff7b9746c42b71db0706a30785bed8d3b3b7af7b3`、`ev_9c07a62fc7e28a0869a85e543734a3d5277d7dfe49a6962bc8a07fa4600d0a15`。
- short 来自 `AfStreamGetSaveVersion(index, 6, table)`；ArcMap 静态表 `0x54bfb4` 的六个小端 short 为 `[10, 13, 14, 16, 18, 19]`。10.1 AfCore `0x68a847f0` 在表非空且长度非零时，对超过末项的索引使用最后一项，否则索引表；空表或零长度返回 -1（`ev_abca4692ee1d98e672132f9cf11684cb698e485cbb2b938f5559ffc66a2b4b0c`）。原生分支没有检查负索引；Rust 写入器支持非负 i32 索引并按原生规则截到最后一项，拒绝可能导致越界访问的负索引。索引大于 5 时的回退行为不表明它是已知版本。
- ArcMap 读取入口 `0x4764d0` 打开 Version，尝试读取字符串，读取成功时将文本传给对象流相应槽位；随后读取有符号 short，读取返回值为负时退出，short 大于 `0x13` 时拒绝。short 大于 `0x0b` 时再请求 4 字节，并将值传给对象流 `+0x54(-1, value)`；该 4 字节读取返回值在此处未检查（`ev_1d35977adeea827f978a8bf7905268b79aea75c99d491dc10b23f69c2fdaf67a`）。原生短读未严格检查，Rust 则要求完整输入并保留失败游标。PageLayout 的另一个读取分支输出 3 或 5，不能与此处 short 支持上限混为一谈。
- SystemCore ObjectStream 虚表 `0x800710dc` 的 `+0x5c` 为 `0x7ffd05d0`，根据布尔参数返回两张表之一的计数字段；`+0x54` 为 `0x7ffd0550`，把传入计数除以 10 并调用表重分配函数，后者至少分配 17 个桶（`ev_e903330ad760224fa227a17c39f77e94ccd4a5e4ea49446c797787529c96a67f`、汇编 `ev_ef70c36c1dde1a1ca58ab444267ad14885c584b4694ecd33afd88e60f09effb2`、`ev_7f110c62e3c4a18763c4f9b771d79160cc3bed7e9845d0e319573d3cf7c1b66e`）。写入取参数 0 的表，读取传 -1 选择另一张表；这是一项容量预估信息，不能解释为图层数。
- `MxTools.dll` 10.1.0.3035（SHA-256 `d29677c8514e705c6585b1b5a27d4e5cf32f7326891ccfe6ed7beeb70ca51347`）`0x100e6630` 将读过 Version 前缀的对象流传入 `DocumentInfo`（`ev_307b5b91ea44cc7b987db49ff39314f5250632eb772757c204df4d3d95db7e4d`）。PageLayout 的类表将 CLSID `5012efc9-6d2c-4f0e-8513-5349010cbce3` 关联到 DocumentInfo（`ev_956968fff9a3b39fd0bc53425996c0f3d123b8edc1db293cee388ea74d3d4618`）；其读取函数 `0x7b691650` 先把该对象流重新连接 `Metadata`，因此后续读的是 Metadata，不是 Version 尾部（`ev_014bceef23ed5ff296690dd48adc003fc0e7b56c1bdcc92379b5542b38d30c8b`）。

Rust 的 `read_arcmap_header_10_1` 和 `write_arcmap_header_10_1` 实现上述正常头部的严格读写：保留任意剩余数据，要求完整计数字节，拒绝过大 short、无效 UTF-16、缺失终止符及不匹配的保存参数。它们不声称替代完整保存过程，文本由调用者提供，对象计数须由实际对象序列化获得。合成测试覆盖六项映射、条件边界、截断和失败游标。仓库参考样本辅助观察为文本 `10.1`、short 19、计数 2267、无剩余字节；该样本不是独立的目标软件执行验证。

SystemCore 其他 IID 匹配分支的构造与虚表复核见下节。保存版本索引 3 的运行时资源解析、Version 在所有旧版分支及失败情况下的流边界和目标软件运行行为仍需进一步验证。ArcMap 10.1 与 QGIS 的人工目视检查未进行。

## 10.1 版本接口实现及保存分支复核（2026-11-13）

以下继续使用上文 SystemCore 和 ArcMap 可执行文件的摘要；REA 6.8.0 / Ghidra 12.1.4，静态证据包为忽略目录 `temp/arcgis/arcmap101-version-iid-constructors.json` 与 `arcmap101-version-followup-2.json`，含本机路径。类名取自 Ghidra 的类型信息，构造后的具体虚表和访问方法另以地址复核；不表示已运行这些对象。

- 四个构造函数将 `IDocumentVersion` 子对象放在对象基址 `+0x18`，最终虚表依次为：`BufferedFileStream` `0x8006db14`、`FileStream` `0x8006fb1c`、`MemoryBlobStream` `0x800709ac`、`VariantStreamIO` `0x80074708`（`ev_ee2865fbaf435c709273fb7349c590dfa454b763949541c0d1a78678be0742e8`、`ev_68052f8df22606836258cb02bfabde30358aac22ffd9af74b7193560ac52b69a`、`ev_ce0e6f3a9865cdda7140735305b96e7f090e49c89fbf0c557a094e8c13c76758`、`ev_d2211dca6de4067c3ae228deff3d6879e306241c8eb7d525637623d1d4e7a718`；汇编 `ev_ac354ed65929904eeba0b1fb1ed200bcad6ccc6c7c244813576a57655ad36717`、`ev_7771a6828258c43acb786d5ecc8af66579b0b2e0bcb188aa0067f987001cc59c`、`ev_615ae2f39a8488a1de0cbdddfdd17ef08ca79a04d2b33958a4eb4488495cc750`、`ev_e09a1e01853e1d272eb34323fe7865a7e17472be57768d825244f0c5c4dde9d4`）。前两类共享查询分支 `0x7ffc5a90`，另两类分别使用 `0x7ffcbb50` 和 `0x7fff9ee0`；这些分支都将该 IID 映射到 `+0x18`。连同上文 ObjectStream，这是本次在 SystemCore 中确认的五个类，并非整个安装包的穷举。
- 前两类的虚表 `+0x0c/+0x10` 分别指向 `0x7ffb7ae0/0x7ffc1e10`，读写接口指针 `+0x14`（对象基址 `+0x2c`）；MemoryBlobStream 指向 `0x7ffcc460/0x7ffcc480`，读写接口指针 `+0x1c`（对象基址 `+0x34`）。三个构造函数均将对应字段初始化为 5；getter 的空输出指针返回 `0x80004003`。VariantStreamIO 的 `0x7fffa060/0x7fffa090` 在内部对象为空时返回 `0x80004005`，否则分别转发到内部接口的 `+0x0c/+0x10`（函数证据依次为 `ev_6bd88102d0f40d3b26e78cc9e75e7365a309016d72a0e63ba2a6bf11c69bb85c`、`ev_bec5abff7e42b02ef4d1a026e7554847e30c89a064e1b82e2a1d043e6647ed9a`、`ev_9b8aa90e5ca10ebbd7f979cf8d0dc83d7a32ef81c6fc317097df544089c72f26`、`ev_1d58fabfbf88ab57ce009b0c6c59854c1daf07a9932e801ef0639c4cb9ca0f7b`、`ev_859624aef0f636f9e2d93cd4795ace60dbcf6d42129139a67935ecc9d907bb9c`、`ev_c816ca7c8111acbf5fbb1d4ab5cae63b65ae27949684de3ff60c95d7f19e8b42`）。这些方法操作内存属性或转发，不是 Version 磁盘字段的读写方法。
- ArcMap 的 `0x472ac0` 打开 `Templates` 并在成功时将对象流重新连接该流，再读取模板数量和条目（`ev_199ef1d55356a5eececc4f036ec0268efa7bdcd1506c82ce8bc8815c06797db6`）；对应写入函数 `0x472e30` 创建并连接 `Templates`（`ev_3c2977eb36ade34db171f1ca33042628f703b668c36ab49edf5e500111f00fc7`）。另有 `0x46cde0` 在取得存储参数时创建并连接 `DrawingDefaults`，然后写入一组对象及两个 8 字节值（`ev_1903b11affe928cf82713eafc8b4e5258e90f33907cde0ada96bae6567b293be`）。这些写入不能只因复用对象流而认定属于 Version；失败或替代路径仍须按实际连接状态判断。
- `0x47a3f0` 的输出初值为 5；获取构建号属性成功时还会解析构建号并有条件地输出 3，否则在相应分支按 short 小于 `0x12` 输出 3（`ev_a74bbb9c6bea13321e4037134ac0e56150d621e72aeacd06cf85b48654666599`）。该条件涉及独立的 Build Number 属性，不能仅凭 Version short 建立完整版本映射。
- 独立使用 pefile 检查 ArcMap.exe 自身英文字符串资源：ID `0x14d/0x173/0x186/0xe804/0x193` 分别是 `8.3/9.0/9.2/10.0/10.1`；`0x11a` 却是打印错误文本。由于保存代码使用的句柄可能指向本地化资源模块，不能把 exe 表中的 `0x11a` 文本当成所有运行环境的实际输出，也不能猜测为某个版本号。资源候选仅为静态信息，资源句柄在目标运行环境中尚未确认。
- AfCore `ESRILocale::LoadLocalizedRes` 将传入模块的 `.exe` 或 `.ocx` 扩展名改为 `.dll`，取文件名并加 `Res` 前缀，再交给 `LoadLocalizedResFromFile`（`ev_a6659937223baa1554cbad185f9e04ccc46546738ac2ca7361796c1b1f02978d`）。后者在组合后的语言标识为 `0x409` 时直接返回空句柄，否则按配置目录查找并加载文件（`ev_0eed7e47f87c4c4d5d5a0f1857e7238a9a485ea71bc538abe41ee911fa27a54a`、汇编 `ev_b3119126fa317cb4bdcac528256891bdaeda8813c4e74e49b0241554b50f5afc`）。调用方随后在空句柄时回退到传入模块；但尚未在运行时确认该模块句柄、语言标识和 ID 3 的写出文本。证据包 `temp/arcgis/arcmap101-version-localization.json` 含本机路径。
