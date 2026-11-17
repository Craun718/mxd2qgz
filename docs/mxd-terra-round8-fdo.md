# FDO 旧加载器属性与 COM 槽对位

2026-11-17。以[旧加载器汇编及 QI 接收者](mxd-terra-nextpass-fdo.md)为起点；
重新核对 10.1.0.3035 AnnoLayer.dll SHA-256
`b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`。
esriCarto 的提取 TYPELIB 摘要
`8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。
使用独立 Wine 前缀对序号 5/92/191/458 的实际 TYPEATTR IID 与 FUNCDESC 枚举交叉检查。
默认沙箱拒绝本地 Unix socket，改用获准执行；工具输出四个接口后退出码 3，
类型名宽字符打印有乱码，以下只采用有效的 GUID、方法名、参数和槽位行。
64 位环境的 oVft 减半得到目标 x86 槽。未启动 ArcMap。

| 字段 | 介质类型库的写入槽（x86）与静态参数 |
| --- | --- |
| 1–5 | ILayer.Name +0x10、Visible +0x30、ShowTips +0x38、MinimumScale +0x20、MaximumScale +0x28 |
| 9 | ILayer.SpatialReference +0x4c；FDO 先保有后转发 |
| 11 | IName 查询并取得 IFeatureClass 后调用 IFeatureLayer.FeatureClass +0x58 |
| 12–14 | IFeatureLayer.Selectable +0x70、DataSourceType +0x60、DisplayField +0x68 |
| 15/16 | IFeatureSelection.CombinationMethod +0x18、SelectionSymbol +0x38 |
| 17/18 | IFeatureLayer.ScaleSymbols +0x7c、IFeatureSelection.SetSelectionSymbol +0x28 |
| 19 | IFeatureSelection.SelectionColor +0x30 |
| 20 | IDataLayer.RelativeBase +0x20 |

字段 16 的请求常量 `0x6b1abd70` 原始 16 字节对应 esriDisplay ISymbol
`{F3435802-5779-11D0-98BF-00805F7CED21}`，并非旧记录写的 IFeatureRenderer。
读取器保持原始标量和对象槽；不模拟 setter 或将 CombinationMethod 擅自转换为枚举。
字段 6、7 为直接成员，未由此确定业务含义。真实旧档、版本 4 探测、运行时副作用和
ArcMap/QGIS 目视验证仍开放。
