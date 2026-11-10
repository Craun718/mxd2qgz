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
