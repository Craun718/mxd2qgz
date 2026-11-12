# ArcGIS 运行库研究入口

## 目标身份与证据（2026-11-10）

研究子模块 `third_party/ForestRight` 固定于
`c70d6d9350e8b01ed2b698cc11f93134a4c916d3`，只用于实现研究，不参与构建或发布。

REA 静态托管分析得到：

| 程序集 | 版本 | SHA-256 |
| --- | --- | --- |
| ESRI.ArcGIS.Carto.dll | 10.1.0.0 | e638bbe014f6c39d3ec4792c3f410b3bc909d77ae6fe1178e8376c75f69745b4 |
| ESRI.ArcGIS.System.dll | 10.1.0.0 | 657c2072703fa9904a9f25bc56c4a46425b86ba2af89f9e7c3faddd6a307a766 |
| ESRI.ArcGIS.Version.dll | 10.1.0.0 | da2531f0129b8dd5df7cae7134010f74e622bed4c277ddc9e1a3a029bfee3eb7 |

- `inspect_managed_members` 的 Carto 证据
  `ev_a1409a817434c5aad579086cc9c75a2c85730b0e446d18ff834f4ee2ff1714bc`：
  `MapDocumentClass.Open` token `0x06004511`、`GetVersionInfo` token
  `0x0600451e` 均为 RVA 0、方法体 absent。该类型的 73 个方法未提供托管实现。
- System 证据 `ev_f4a33da50a7f31f33647503389affa163f2ca56e33ecc0f6984026ba8a86a509`：
  选中的 `IStream`、`ObjectStreamClass`、`IObjectStream`、`IPersistStream`、
  `MemoryBlobStreamClass` 共 101 个方法均无方法体。其中
  `ObjectStreamClass.LoadObject` token 为 `0x060001ee`。
- Version 证据 `ev_e0c0ab67f054400166b7bf286d7645c3c9acaee1cee6c58df3d7fea81024c91c`：
  `RuntimeManager.GetArcGISVersion` 包含托管代码及对本机互操作函数的调用；
  使用的字符串为 `\ArcGIS\bin\ArcGISVersion.dll`。这是运行库发现逻辑，
  不能当作 MXD `/Version` 的读取逻辑。

这些是接口和调用线索，不能确定 MXD 编码、对象边界或引用的磁盘表示。
REA evidence ID 仅供关联会话，复核仍需以上摘要、类型和 token。

## Desktop 10.8 原生分析入口

用户提供 ZIP 的 SHA-256 为
`b58d458a6095dcc5364b60b9fb21dac71b0189f4def19664418cdd6038146a54`。
REA `inspect_artifact` 证据为
`ev_e105833e97347c9a95f9974b5b76ad51e7d5710760e9305da346e90104f6cdc4`。
外层清单不是嵌套安装包全部内容的证明。

使用 7-Zip 静态提取 `ArcGIS_Desktop_108_172737.exe`，PE 版本信息显示
10.8.0.12790；其自解压数据包含 `Desktop/SetupFiles/Data1.cab` 与 `setup.msi`。
没有运行安装程序。CAB 中的文件名包含 MSI 文件键后缀，保留这些原名以追溯来源。

已提取并由 REA `open_binary` 识别为 x86 PE 的原生入口：

- `AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`
- SHA-256：`560be16e0a99b9de4599f8241c421bddf0dcc8b26509f584074dea53b0ac4aae`

CAB 清单另包含 `SystemCore.dll`、`MappingCore.dll`、`MappingCoreLib.dll`、
`Geometry.dll` 和 `ArcGISVersion.dll`，作为后续候选；文件名不能证明其职责。
提取物置于忽略的 `temp/arcgis/`，不提交安装包或 DLL。

## 尚未验证

2026-11-12 尝试通过 REA `open_binary` 分析已提取的 MappingCore 时，
返回 `provider_unavailable`，未产生反编译证据。诊断报告当前 Ghidra 为
12.2，而连接的 REA 仅接受 12.1.x（核验版本 12.1.4）；配置使用 Java
21.0.12.1，而提供程序要求完整的 64 位 JDK 25 或更新版本。
这是分析环境兼容性阻塞，不是目标 DLL 或 MXD 格式的结论。
后续须对齐 REA 支持的 Ghidra 版本和其启动环境中的 Java，再继续版本读取研究；
本次未修改工具配置，也未验证 `/Version` 字段。

- 尚未取得原生 MXD 读取函数的反编译结论；不得将 10.8 的潜在规则直接应用于 10.1。
- `/Version` 其余字段、投影文本的归属、对象流的边界和引用编码仍未知。
- 未在 ArcMap 或 QGIS 中人工打开并核对转换结果；当前没有转换结果。

通用 `Reader` 的 UTF-16LE 读取要求调用者显式传入字节数，不读取长度前缀，
不去除终止符，也不意味着 MXD 使用统一的字符串规则。

## AfCore 创建对象流的原生调用证据

Ghidra 12.1.2、默认分析配置，在上述 AfCore 摘要下：

- `search_procedures("ObjectStream")` 返回零匹配；这只说明已恢复函数名中
  没有匹配，不能证明对象流实现不存在。
- `search_procedures("CreateMemoryStream")` 定位 `0x10058310`。
  反编译证据 `ev_82fb7be463919c44d46862d08ea99eab0d618e0d084822eed2cd37744c8e799b`
  显示调用 `CreateStreamOnHGlobal`、`AfCreateInstance`、`OleRun`、
  `QueryInterface`，随后通过虚表 `+0x38` 的间接调用连接流。
- `AfCreateInstance` 所用类标识地址 `0x1025e970`，16 字节为
  `d0313704cfa7d1118bd1080009ee4e41`，按 Windows GUID 布局解码为
  `043731d0-a7cf-11d1-8bd1-080009ee4e41`。
- `QueryInterface` 所用接口标识地址 `0x1026c8dc`，16 字节为
  `a75ba4186612d11186ad0000f8751720`，按同一布局解码为
  `18a45ba7-1266-11d1-86ad-0000f8751720`。

以上地址是 AfCore 的分析虚拟地址，**不是 MXD 流内偏移**。虚表连接调用的
语义来自调用上下文，仍需实现或接口对应关系进一步核对；这里没有读取 MXD。
下一步沿类标识定位 COM 类工厂及持久化实现。

已从同一 CAB 提取 `SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`，
REA 确认 SHA-256 为
`48cbd921972d12156119d524f9c3923225fd30795e33c22c65c87db3a6c89aca`，x86 PE。
这是后续分析候选，不能仅凭名称认定包含上述 COM 类。

## SystemCore 的 ObjectStream 类关联

在上述 SystemCore 摘要、Ghidra 12.1.2 下，类 GUID 候选位于分析地址
`0x100fb260`（文件偏移 `0xfa660`）。REA 交叉引用证据
`ev_efeb1d4e85360e41d9f53d78363df48cf8a6f063588e478154df680efecc8712`
返回 `0x10001d10`、`0x1006f00b`、`0x1006f1c3`。

对首处交叉引用所属函数 `FUN_100014c0` 的反编译证据
`ev_845e266ceb8c06b86b1dde87fb2aaf73875c73cff5ef3043a46df66d0d2880d6`
显示连续初始化：

```text
_DAT_101306ac = &DAT_100fb260;
_DAT_101306b0 = L"ObjectStream";
_DAT_101306b4 = allocation(0xc);
*_DAT_101306b4 = ClassFactory<ObjectStream>::vftable;
```

`FUN_1006f1b0` 的反编译证据
`ev_74745e5b4bfe1e020a313f10512bd4c71e808a6d54cc34f84bf76ab4de92e74b`
还显示将该 GUID 的四个 DWORD 写入输出参数。这些证据支持该类标识与
SystemCore 的 ObjectStream 实现相对应；尚未确认其 MXD 磁盘编码。

恢复的名称清单（REA retained evidence
`ev_c57049e0d6375af478fa8cbbb09fedbaf0d1bd64b6f9000997c44fa1c7e9d391`）
含 `IObjectStream2::vftable`，地址 `0x100ff678`；
`ObjectStream::vftable` 出现在 `0x100ff6dc`、`0x100ff6f4`、`0x100ff70c`、
`0x100ff724`、`0x100ff788`、`0x100ff794`。
应先核对接口槽位及调整 thunk，再追踪 LoadObject、版本和字符串读取实现，
不能只按名称或槽位次序推断格式。

本次 REA 名称清单超过 MCP 单次响应大小，已按工具建议保留完整 Evidence，
连同反编译记录导出到忽略的 `temp/arcgis/systemcore-snapshot.json`。
AfCore 记录位于 `temp/arcgis/afcore-snapshot.json`。两处会话均已关闭。

## ObjectStream 接口槽位候选（2026-11-12）

本次 REA `inspect_managed_members` 证据
`ev_cb3ddc1fe730389583d98ed2d7511c9f494b92893fedd3c386d624b92d4b677b`
来自上述 System 10.1.0.0 程序集，MVID 为
`4d0ea07a-333e-4626-b241-5525c1190e58`。选中的 `IObjectStream`
（类型 token `0x02000085`）含 18 个方法，均无托管方法体。
元数据顺序的前 11 项为 `RemoteRead` 至 `Clone`；其后为：

| 方法 | 方法 token | 参数 → 返回值（REA 签名表示） | x86 虚表偏移候选 |
| --- | --- | --- | --- |
| set_Stream | `0x060001ff` | `class:36` → `void` | `+0x38` |
| get_Stream | `0x06000200` | 无 → `class:36` | `+0x3c` |
| SaveObject | `0x06000201` | `object` → `void` | `+0x40` |
| LoadObject | `0x06000202` | `valuetype:45&`, `object` → `object` | `+0x44` |
| ReplaceObject | `0x06000203` | `object` → `void` | `+0x48` |
| set_Version | `0x06000204` | `string` → `void` | `+0x4c` |
| get_Version | `0x06000205` | 无 → `string` | `+0x50` |

**观察：** `ObjectStreamClass` 另声明 `set_DocumentVersion`
（`0x060001f2`）和 `get_DocumentVersion`（`0x060001f3`），而所选
`IObjectStream` 方法清单没有这两项。请求同时选择了 `IObjectStream2`，
返回的类型清单仅有 `ObjectStreamClass` 和 `IObjectStream`；不能据此给
10.8 原生 `IObjectStream2::vftable` 标注方法名。签名中的数字保留工具
返回的 TypeDefOrRefOrSpec 编码，尚未将 `valuetype:45` 认定为具体类型。

**推断：** 表中偏移按三个 IUnknown 槽、每槽四字节并延续元数据顺序计算。
`set_Stream` 的 `+0x38` 与 AfCore 创建流时的间接调用相符，支持沿该位置
继续查找；但 10.1 托管声明和 10.8 原生实现跨版本，且尚未核对接口 GUID、
调整 thunk 和实际虚表内容，因此这些偏移仅为查找候选，不能视为原生入口
已确认。`Version` 和 `DocumentVersion` 的接口属性不证明 `/Version`
数据流的字段意义或编码。

**未解决：** 本次 `binary_session` 再次报告原生提供程序不兼容
（Ghidra 12.2 / Java 21），未启动新的原生分析，也未运行 ArcMap 或 QGIS。
下一步在兼容环境中先核对接口 GUID 与虚表，沿 `LoadObject`、`Version`
和 `DocumentVersion` 的实现追踪实际读流调用及错误分支，再验证样本字节。

## 10.8 对象流实际接口与虚表（2026-11-12）

本次连接的 REA 6.8.0 已提供 Ghidra 12.1.4，默认自动分析完成后可查询
上述 SHA-256 对应的 SystemCore。此前的环境阻塞是历史记录，本次不再复现。
以下均为 DLL 分析虚拟地址或对象内存偏移，**不是 MXD 流内偏移**。
分析配置摘要为 `e763201e522a4b123fb330eede09834c44ddf2bb90702e50eea219801be95539`。

### 实际观察

- 构造函数 `0x1006f1e0` 的汇编证据
  `ev_e4e22b0d36e8549778008ef8d038edbff8bbf900cb72b20c44cac5a8832799ae`：
  `0x1006f23f` 先向对象 `+0x10` 写入 `0x100ff678`，
  `0x1006f268` 再将其覆盖为 `0x100ff724`。前者是构造期间的
  IObjectStream2 基类虚表，不能作为构造完成后的方法入口。
- GUID 读取证据
  `ev_156e4eaf234c7e53a5c5c0d4e1519524039277444242b354c59def8e4d1953e0`
  与 `ev_8c9418167bc5331398b1ee83cae2f55a6964f933535cfdd0695c986fdcec720b`：
  `0x100ff634` 为 `18a45ba7-1266-11d1-86ad-0000f8751720`，
  与 AfCore 所请求接口一致；`0x100ff624` 为
  `22f26dbb-c6c9-41dd-bdfd-bc189772dbfd`。
- `0x1006f570` 的反编译及汇编证据
  `ev_a0e9d81c4aa8d3a57b84bfdf57f477438467916c1efe0e49e44faef042b9cfdf`、
  `ev_95811a325adf04a3b8ba349a17cbd1b66b7bf3139016bb8db2cb0346d8048573`
  显示两个 GUID 均返回对象基址 `+0x10` 的接口指针。
- `0x100ff724` 的完整读取证据
  `ev_f988e876575340439434f530bc3f18d5e2187a6d290de6764870555c5445d2ca`
  给出下表的实际指针。前三槽 `0x10005ca0`、`0x10005cc0`、
  `0x10005ce0` 通过接口指针 `-8` 处保存的控制对象指针转发调用；
  不是直接对接口指针减 `0x10` 后调用。首槽汇编证据
  `ev_603b62ac086466b898c29830f455c75cfafa9b7785b6a50a4a0266594a71eeaf`
  可复核 `MOV EAX,[EAX-8]`、替换首参数和间接 `JMP`。

| 接口槽偏移 | 实际入口 | 原生观察及名称判断 |
| --- | --- | --- |
| `+0x38` | `0x1006f8c0` | 更换接口相对 `+0x10` 的底层流，正确增减引用；与 set_Stream 候选相符 |
| `+0x3c` | `0x1006f890` | 返回同一底层流并增加引用，空输出参数返回 `0x80004003` |
| `+0x40` | `0x1006f8f0` | 写流入口，另见后续对象读写研究 |
| `+0x44` | `0x10070020` | 读流并创建或复用对象，与 LoadObject 候选相符 |
| `+0x48` | `0x100704e0` | 已读取入口地址，本次未解读 |
| `+0x4c` | `0x10070640` | 对接口相对 `+0x40` 的 String 赋值 |
| `+0x50` | `0x10070610` | 从同一 String 生成 BSTR；空输出参数返回 `0x80004003` |

连接/返回流的反编译证据分别为
`ev_1a69d78b5faa809ff5f4b203b4e0d7f182eb9f45980a2731a537a488e83b5864`、
`ev_480d48cc696d801c0feaf473fd65bcd363638d52351f33c7e3526de78c6c0e4f`；
字符串赋值/返回证据分别为
`ev_1d94cebdac0e895f0bcdd4b5e2e64000449d0e80e3192e9be9300baa45fc958a`、
`ev_abe583aa04b284e59a3ff1028375ccf1bda88ccb41f5ad2a96bdd69477d6d2bd`。

### 结论与限制

AfCore 的接口 GUID、SystemCore 的接口返回位置、构造后的虚表以及
IUnknown 转发方式已相互对应。方法名仍结合先前 10.1 托管声明和本次
10.8 原生行为判断，未取得同版本类型库的完整接口声明。
特别是 Version 候选槽只是内存属性访问，没有在此读写 `/Version`，
不能据此确认保存版本。尚未执行 ArcMap 或 QGIS，也未验证 10.1 磁盘兼容性。

## 10.8 对象头、引用表及局部读取器（2026-11-12）

沿上一节实际虚表，在同一 SystemCore 摘要及分析配置下得到以下静态证据。
这里描述的是对象流当前游标处的相对字节布局；没有确定它在 `/Maps`
中的任何绝对偏移，也没有证明 `/Maps` 全部由该种记录组成。

### 实际观察

- 读入口 `0x10070020`，证据
  `ev_e66c6155653da0ba5901726fdff392d6a3011656f50ebd0968c42ce0717be65c`：
  从底层流调用 `AfStreamRead` 读取 16 字节，与 `0x100f1ee0` 的
  16 个零字节比较（字节证据
  `ev_684d52c0d9bbd48aa5015dbc224f31b7936b678567b2564a22f4920bbac02beb`）。
  全零时不继续读取键；非零时再读取 4 字节键，调用 `0x10070ae0` 查表。
  命中时对已有对象查询所请求接口；未命中时以读出的类标识调用
  `AfCreateInstance`，将键与新对象加入表，再调用对象的持久化读取接口。
- 查表 `0x10070ae0`，证据
  `ev_58470d02474d8658854f4f7cd78e2cef554c95543b55660471cab393f538e524`：
  按无符号键取模选桶并比较键，命中返回保存的对象指针。
  插入 `0x10070b30`，证据
  `ev_ba5b4a78bd8c52a7049fa9fe451955ab518fee817030342e56025d08969e4291`：
  新键保存对象指针；重复键分支会替换指针。本次没有据此定义解析器的
  重复键策略，也没有证明重复键可合法出现在磁盘中。
- 写入口 `0x1006f8f0`，证据
  `ev_66747c25129da79415faa5f17a9aec26984af4022c512c5c2e25c50da4c94d14`：
  空对象分支写 16 个零字节。普通持久化分支先写 16 字节类标识，再查对象表；
  已有对象写 4 字节已有键，新对象写 4 字节计数器值、递增计数器，
  再将对象体交给持久化接口写入。存在替换对象和 VariantStream 分支，
  不能将普通分支当作所有对象体的结构。

### 有限结论及实现

在已知调用上述 10.8 对象流读取器的起点，可以读取局部头：
全零标识是 16 字节空头；非零标识后有 4 字节小端键，总头长 20 字节。
这里没有长度字段说明对象体的结束位置。引用由键是否已在对象表中判断，
不是由类标识或键的某个高位判断；不能把每个非零头都当作新对象。
先注册对象再读取其对象体支持递归引用的可能性，但循环处理尚未验证。

`src/object_stream.rs` 的 `read_header_10_8` 仅实现该局部头读取，
保留类标识原始 Windows 字节顺序与完整键值；不创建 COM 对象，
不扫描起点，不解析对象体，也不参与当前 CLI 的 `/Maps` 解释。
调用者负责提供已确认的版本和起点。失败时保持输入游标不变；
截断输入返回明确错误，而不是仿照尚未核对的原生错误处理。

合成测试覆盖空头后续字段、16 个不同位置的非零 GUID 字节、零键、
最大键、字节序、不透明对象体、全部头截断长度、非零起点与子范围隔离。
这些测试只验证实现与上述局部规则一致，不是目标软件验证。

**未解决：** `AfStreamRead` 对短读的行为、完整 HRESULT 返回路径、
持久化接口的同版本 GUID 与方法定义、对象体边界、替换分支、循环引用、
`/Maps` 的根对象起点及 10.1 兼容性。未在 ArcMap/QGIS 中打开或目视检查。

本次 27 条 REA 记录已随关闭会话保存至忽略目录中的
`temp/arcgis/systemcore-10_8-headers-snapshot.json`；不提交原生 DLL、
提取流或完整诊断输出。公开复核依据是本文的目标摘要、分析配置、入口地址及证据 ID。

## /Version 原生读取入口调查（2026-11-12）

本轮只补充静态研究，不修改解析接口或 CLI。结果为**部分推进**：
已定位版本辅助函数及其一个原生调用者，但没有把文档打开、版本查询与
OLE `/Version` 的实际读写连接起来，不能据此实现版本识别。

### 目标身份与复核环境

目标均来自前述 Desktop 10.8 安装包；REA 6.8.0、Ghidra 12.1.4、
默认自动分析，分析配置摘要仍为
`e763201e522a4b123fb330eede09834c44ddf2bb90702e50eea219801be95539`。
本轮本地检查各 DLL 的 `VS_FIXEDFILEINFO`，文件版本及产品版本均为
10.8.0.12790。这是 DLL 身份信息，不是 MXD 保存版本。

| DLL（省略 CAB 文件键后缀） | SHA-256 | 版本结构的 DLL 文件偏移 |
| --- | --- | --- |
| MappingCore.dll | `48b93d5aa9e3681f19e76564a6e42a7ce4cf951770e62699d2ee0893f14b1248` | `0x247e88` |
| MappingCoreLib.dll | `268a84465970808f50c7ca6ed014c98bca7019db4377caf5008394b860398838` | `0x144738` |
| AfCore.dll | `560be16e0a99b9de4599f8241c421bddf0dcc8b26509f584074dea53b0ac4aae` | `0x2ae888` |
| Map.dll | `40592a416cd5891e86749026b44e9fdd0480c619c9c7f94f737faf924013f6e3` | `0x130820` |

下文 `0x100...`、`0x101...`、`0x102...` 均为**所属 DLL 的分析虚拟地址**，
`+0x10` 为接口虚表槽偏移，均不是 MXD 流内偏移。

### 原生静态观察

1. MappingCore 中 `search_procedures("MapDocument|Version", regex)`
   只返回版本命名空间映射导入和两个 `ChangeVersion` 方法，证据
   `ev_0679dc1523cce0202856a26f38be5ff07ed79032a2c2934b33a984eb20542524`。
   `MapDocument` 字符串位于 `0x101dd3fc`，其交叉引用为 `0x1000f0b0`，证据
   `ev_1a6363e6a73ba3813064441bec1a60dc9e52dfaf8816024aac674e4061c0f985`。
   该地址的指令是 `PUSH 0x101dd3fc`，Ghidra 未把它归属到已分析函数，证据
   `ev_efcbf140f67898c4a7a4d5d690c95562ab3723a0b887c599188fd80135bb48b5`。
   周围字节及全局地址 `0x1024a1d8` 的引用分别见
   `ev_0624edd5dfc49136efa65b2cea44ba6ad39d2d17ea2d10f8a5f0e6bb25c5ff38`、
   `ev_fe9eabc95b98d086018e13ea30be817b4270f3eaeb518e3d08f746c0cebcc53d`。
   这些只支持全局字符串初始化/清理的线索，没有定位到 MapDocument 类工厂或打开实现。
2. MappingCoreLib 导入 `AfStreamGetDocumentVersion` 与
   `AfStreamGetSaveVersion`，证据
   `ev_044a8bf8644d95a6fed4ab02b16a339dbdf0ed02e425531dc80c1d746f6e45db`。
   两个实际调用点为 `0x10082f61`、`0x10082f6f`，均属于
   `PersistedPrepareResult::Save`（`0x10082f30`），反编译证据
   `ev_c007acd408b72b9ab59bc12721f854cf79508b0a328551899a83084483dcab4e`。
   它把查询得到的枚举传入保存版本表查询，再以 `AfStreamWrite` 向传入流
   写入 2 字节局部值，随后写其他成员和对象。未观察到它打开 `/Version`；
   不能把这里的 2 字节值认定为版本流尾部字段。
3. AfCore 的 `AfStreamGetDocumentVersion` 位于 `0x100070d0`，反编译证据
   `ev_03be041d12eb6a60f2e1569c1bd744da2cea8616244b66600fd32884ff0b6b23`。
   汇编证据
   `ev_85b78354bdff5ea3d39db4e2353b377e319f1c7d5d8a767459d5edfe69267efb`
   显示：输出局部值初始化为 `0xb`；传入指针非空时，以地址
   `0x1025cc40` 处的 IID 查询接口；查询成功后通过虚表 `+0x10`
   调用，仅传入接口指针与局部值地址，最后释放接口并返回该值。
   空指针或 QueryInterface 失败会保留默认值；后续接口调用的 HRESULT
   未参与分支判断。本轮未解码该 IID，也未核对该枚举的同版本定义。
   **反编译类型限制：** 伪代码把该间接调用显示为 `IStream::Write`，
   但汇编没有传入 Write 所需的长度和已写字节数参数；必须保留为未知接口方法，
   不能据伪代码的类型标注声称它写入磁盘流。
4. AfCore 的 `AfStreamGetSaveVersion` 位于 `0x10007160`，证据
   `ev_bfadab2cf3d9f416bdae212eba94dd7d89523e6b88382f0c9851ac320c2791b1`。
   该函数没有读流：表指针非空且 short 参数非零时，根据传入枚举与
   `short 参数 - 1` 的有符号比较选择 short 表项，否则返回 `-1`。
   这说明此函数用于调用方传入的版本表选择，不能证明 `/Version` 的编码。

### 有边界的搜索结果与推断

- MappingCore 已初始化映像中，UTF-16LE `Version` 加终止零的字节搜索
  只命中 `.rsrc` 的两处后缀，证据
  `ev_aa2fa2dd87425024535400bda137b97c5e48f6839c89fe18fe2d34e130e83eba`。
  这不排除动态构造流名、间接调用或其他编码。
- MappingCoreLib 的同一字节搜索命中 `.rdata` 的 `0x100eda0e`、
  `0x100ee73a` 和两处资源后缀，证据
  `ev_7288d82da1091c13f9da1a3bcacb834861279d00f86e8c92e1a6430f79170839`。
  字符串清单证据
  `ev_51e368301fedd37f51569dc7d0625cdc0e945c70d2fa69e0c2491efc3ef09be9`
  显示前两处分别位于 `TilePackageVersion`（`0x100ed9f8`）
  和 `currentVersion`（`0x100ee72c`）内部，不能当作 OLE 流名。
  这两个后缀地址的精确交叉引用为空；不代表完整字符串或动态访问没有引用。
- 扩展检查同一安装包的 Map.dll，函数名及字符串搜索证据为
  `ev_dc13fc61aa397a255f6f8d0ef493270987fb47e78ff2d3178780438303cc4e28`、
  `ev_1f6663990a64a51c2d595aec887d6b0952373b1841d97c437034f2a263cf23f5`。
  已恢复名称中未找到 MapDocument 打开/查询实现；版本相关命中含上述
  AfCore 导入、TilePackageVersion 和 PackageVersion，未建立 `/Version` 关联。

以上负结果均受 Ghidra 名称恢复、已定义字符串及静态引用覆盖范围限制。
**推断：** 已定位的版本辅助函数消费内存接口或版本表，真正的文档版本流
解码应在其上游或另一实现中；尚未确定具体 DLL，不能仅凭名称指定归属。

### 样本辅助核对、未知项及下一入口

仓库参考样本身份及与历史样本的区别见
[样本记录](maps-analysis.md)。本轮复核参考样本 `/Version`：

| `/Version` 流内半开区间 | 字节观察 | 目前可作的判断 |
| --- | --- | --- |
| `0x00..0x04` | `0a 00 00 00` | 小端值 10；长度前缀仍是假设 |
| `0x04..0x0e` | `31 00 30 00 2e 00 31 00 00 00` | 可按 UTF-16LE 解码为 `10.1` 和终止零；实际解码器及终止符规则未知 |
| `0x0e..0x14` | `13 00 db 08 00 00` | 6 字节未知数据；字段划分、符号性及含义未确认 |

当前没有原生 `/Version` 读写分支证据，故长度计量单位、是否包含终止符、
空字符串、非 ASCII 编码、尾部字段、短读和非法长度的错误处理均未验证。
不能以 10.8 的内存枚举、局部对象版本或 DLL 产品版本解释 `10.1` 候选，
也不能把尾部任意拆成整数后赋予保存版本含义。

下一轮先从已知托管 `MapDocumentClass.Open`（`0x06004511`）和
`GetVersionInfo`（`0x0600451e`）的类/接口 GUID 入手，核对 10.8 安装注册
或类工厂与实际虚表，定位真正的原生入口；再沿 OLE OpenStream/CreateStream
与读写帮助函数确认 `/Version`。另一条具体入口是 AfCore `0x1025cc40`
的 IID 及其实现者，需要追踪谁设置文档版本属性。找到读写实现后逐一核对
长度、编码、终止符、尾部和错误分支，再考虑版本读取实现。

旧 AfCore 快照因分析配置不匹配被 REA 拒绝导入，本轮已按工具提示从原 DLL
重新分析，未将旧记录当成本轮证据。新记录分别保存在忽略的
`temp/arcgis/` 下 `mappingcore-version[-followup]-snapshot.json`、
`mappingcorelib-version[-followup]-snapshot.json`、`afcore-version-snapshot.json`
及 `map-version-snapshot.json`，会话均已关闭。这些记录仅保留本地；
不提交 DLL、样本、提取流或完整诊断输出。

未运行 ArcMap 或 QGIS，未人工核对源工程或转换结果，未验证跨版本兼容性。
