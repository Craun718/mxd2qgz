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
