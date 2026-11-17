# AfCore 版本兼容辅助接口定名

2026-11-17。此前[原始 IID 复核](mxd-terra-round8-variant.md)确认 AfCore
`0x68a7c160` 使用 `0x68c6dc98` 的 IID
`{BB75DA81-8DFE-4122-A9C2-86F07C539AD5}`，不是 IVariantStream。
该轮未取得名字。

本轮以 .NET PE 元数据读取器静态解析仓库中的
`third_party/ForestRight/vendor/ESRI.ArcGIS.System.dll`，
SHA-256 `657c2072703fa9904a9f25bc56c4a46425b86ba2af89f9e7c3faddd6a307a766`，
文件固定版本 10.1.0.3035。匹配到
`ESRI.ArcGIS.esriSystem.IDocumentVersionSupportGEN` 的
`System.Runtime.InteropServices.GuidAttribute`，不是任意字符串命中。
类型为 COM Import 接口，元数据顺序为 `IsSupportedAtVersion`、
`ConvertToSupportedObject`。不执行目标程序集，也不把托管签名当作原生 ABI。

AfCore 10.1 `SimpleSaveObject` 的既有反编译及原生分支表明，该辅助 QI 成功后依次
调用接口 `+0x0c` 和 `+0x10`：分别检查版本支持、在不支持路线上尝试取得可保存的
替代对象，其后对替代对象递归委派。接口名、顺序和分支互证；这不是新的
VariantStream 宿主证据。另一辅助 IID `{C9E7A782-…}` 后续由 [RTTI 与虚表](mxd-terra-round10-iid.md)
确认为 IDocumentVersionSupport。失败传播和旧版保存的实际结果仍未闭合。未修改解析器，未运行 ArcMap 或 QGIS。
