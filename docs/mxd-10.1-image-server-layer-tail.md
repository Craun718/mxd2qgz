# ArcMap 10.1 ImageServerLayer 后续 Variant 字段（2026-11-18）

适用 ImageServerLayer.dll 10.1.0.3035 x86 PE，SHA-256
`0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`。
接续[前缀与包装取证](mxd-10.1-variant-image-server-layer.md)。
复用 10.1 原始二进制，补用隔离 PyGhidra / Ghidra 12.2 导出
`0x77feb390`、`0x77fece60`、`0x77ffc340`、`0x77ffc4f0` 等函数。
长函数伪代码对部分栈变量识别有误，以下版本门、VARTYPE 和调用参数以
objdump 列表为准。临时输出在忽略目录 `temp/variant-object-101/`。

## 空间参考后的字段顺序

这里只实现 10.1 原生保存表中的 **v8、v13、v18、v22**。表中较低版本门
用于说明这四个版本累加的字段组，不表示已支持其他历史保存版本。
每项都经 IVariantStream，包含自己的 VARTYPE；对象项为 VT_UNKNOWN，
计数与整数列表项均为 VT_I4。

| 门控 | 字段顺序 | Load 对位 |
| --- | --- | --- |
| 基础组 | BSTR，v8 再有一个兼容 BOOL，BSTR，IName | `0x77feb9ad`、`0x77feb9f3`、`0x77feba19`、`0x77feba4c` 委派到 `0x77ffc340` |
| ≥9 | IRasterRenderer、IEnvelope、BSTR、I4、BOOL | `0x77feba77..0x77febb4c`；已支持写出版本中本组从 v13 起出现 |
| ≥11 | 两个 BSTR | `0x77febb83/0x77febbbb`；保存分支与上组一起写出 |
| ≥12 | I4 计数 + 同数量 I4 | `0x77febbfc` 和循环 `0x77febc2d`；保存端 `0x77fed96e/0x77feda5a` 写计数和项目 |
| ≥13 | 一个 BOOL | `0x77febc6d`，保存于 `0x77fedc85` |
| ≥14 | IFields，I4 计数 + 同数量 IFieldInfo，BSTR | `0x77febca9..0x77febd95`；请求接口为 `6B2072E3-23F7-11D1-89D8-006097AFF44E` 与 `A2BAAE2C-969B-11D2-AE77-080009EC732A` |
| ≥15 | BSTR、BOOL、I4 计数 + 同数量 I4，IColor、ISymbol、BOOL×2、I4、ISymbol | `0x77febde5..0x77fec10e`，对象查询在 `0x77febf8e`、`0x77fec016`、`0x77fec127` |
| ≥16 | I4 计数 + 同数量 I4 | `0x77fec1af`、循环 `0x77fec245` |
| ≥17 | 一个 BOOL | `0x77fec2dc` |
| ≥18 | BOOL、ITimeReference、BSTR×4、ITimeExtent、BOOL、R8、I4、R8、I4 | `0x77fec322..0x77fec61f`；请求时间对象的 IID 在 `0x7802fc28/0x7802fc38` |
| ≥19 | IMosaicRule、IRenderingRule | `0x77fec663/0x77fec6b3`，请求 IID 在 `0x780300b8/0x780300c8` |
| ≥20 | IGeodataXform | `0x77fec74c`，IID `5E6CF525-4E74-419B-946F-7DA9BFDF815D`，由 10.1 esriGeoDatabase 类型库定名 |
| ≥21 | I4 计数 + 同数量 VT_UNKNOWN | `0x77fec855`、循环 `0x77fec893`；IArray 实例另行创建，不是磁盘上的一个外层对象头 |
| ≥22 | BOOL、BOOL、I4 | `0x77fec8dd/0x77fec910/0x77fec943`；后续为内存属性初始化；Save 最后一次写入在 `0x77feeaee` |

前三个整数队列均只对正计数进入循环；安全读取器拒绝负数和超限计数。
对于当前接受的保存版本，v8 的兼容 BOOL 经 `0x77fed604` 写零值；
Load 一般性条件在 `0x77feb9ce` 还覆盖 v3–v9，不把该宽条件用作其他版本的支持证明。
名称委派的 Save `0x77ffc4f0` 通过导入 `Variant(IUnknown*)` 包装
成员；AfCore `0x68a78d16..0x68a78d28` 确认构造结果为 VT_UNKNOWN，
因此不是 BSTR 或普通无 VARTYPE 对象头。

## ImageServerName 对象体

ImageServer.dll 固定版本 10.1.0.3035，x86，SHA-256
`b6a7bca69fe4b36e2ff0e973e07875b33030bc957fe765644c29ed276580cb80`。
esriCarto TypeInfo 1632、类标识字节 `0x77fa3e94` 和 GetClassID
`0x77f4f7b0` 一致，CLSID 为 `BD552FEC-FF0B-4BF1-8C46-B6886E71A47C`。
RTTI 基类描述符 `0x77fabf58` 和 `0x77fabeac` 将 IName / IPersistStream
分别绑定到完整对象 `+0x14/+0x18`。后者的虚表 `0x77fa3e34`
之 +0x14/+0x18 是 Load `0x77f4f7f0` / Save `0x77f4f870`。

- Load 为有符号 short（`>1` 拒绝）→ AfCore `String::Read` → 普通
  `LoadObject`。读取点为 `0x77f4f815`、`0x77f4f83b`、`0x77f4f856`。
  最后一槽请求 IID `F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`，即 IPropertySet。
  Save 三项对称；版本表 `0x77f9af28 = [-1,-1,-1,-1,-1,1]`，只有最后一档
  会保存本类的 v1 体，不能由此推断早期文档实际保存的替代名称类型。
- 直接 String 与外层 BSTR 的长度单位不同。IPropertySet 走普通对象头，
  在标准 ObjectStream 包装内仍共享外层键表，不额外读 VARTYPE；
  PropertySet 自身的键值对继续走独立的 `Variant::Read` 语法。

## 接入和验证

有界图层条目会继续消费到首个无法解析的字段，或到四个版本各自的对象终点。
已支持 ImageServerName、PropertySet、Envelope、FieldInfo、Color/Symbol、
TimeReference、TimeExtent 的现有安全子集。IRasterRenderer 已接入
[色表和 RGB 渲染器完整体](mxd-10.1-raster-renderer.md)，IGeodataXform 已接入
[三个变换类](mxd-10.1-geodata-xform.md)。IFields 已接入
[Fields 与 Field](mxd-10.1-fields.md)。已接入[镶嵌与渲染规则](mxd-10.1-image-server-rules.md)。其他未闭合的渲染器、字段定义或规则子槽
的空对象和回引可读，未见过的新键停在该处；
不会将失败槽的键提交，也不会用 ExtraSafe 末尾反推其长度。

合成测试验证四种保存版本各自的终点、三个有计数的整数队列、v21 对象队列、
ImageServerName 的非空 PropertySet 与 v21 回引共用键、未知类和截断时的
前缀保留。尚无包含该类的受控实档或运行结果；没有在 ArcMap 或 QGIS 中验收。
