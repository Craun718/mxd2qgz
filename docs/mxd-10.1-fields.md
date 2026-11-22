# ArcMap 10.1 Fields 与 Field 对象体

ImageServerLayer 版本 ≥14 读取 `IFields {6B2072E3-23F7-11D1-89D8-006097AFF44E}`。
此处是一个 VariantStream 对象槽；集合体内的 `IField` 槽以及字段的对象成员
则用普通 LoadObject，与外层共享 ObjectStream 键。

采用 10.1.0.3035 x86 安装介质，使用 pyghidra 补充 GdbCore.dll 反编译，
并用 objdump 检查引用和调用指令。GdbCore.dll 的 SHA-256 为
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`。
类分别以 GetClassID 常量和 IPersistStream 虚表复核。地址为静态映像 VA。

| 类 | CLSID | GetClassID | IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- | --- |
| Fields | `F94F7535-9FDF-11D0-BEC7-00805F7C4268` | `0x709e7010` | `0x70ad642c` | `0x709e8840` / `0x709e8990` |
| Field | `F94F7534-9FDF-11D0-BEC7-00805F7C4268` | `0x709dfd80` | `0x70ad5d28` | `0x709e1730` / `0x709e1990` |

Fields 先读无符号 short（>2 拒绝）、int32 数量，然后逐项读取
`IField {FCB01CB3-9F0B-11D0-BEC7-00805F7C4268}`（`0x70ac4ca8`）。
v2 最后读取数量个 int32，空字段对象在数组中仍占一个位置，值语义暂保留原值。
Fields 的 Save 版本表 `0x70a89038` 为 `[-1,2]`。

Field 的顺序：

1. 无符号 short 版本（>2 拒绝），再直调 GPValue::Load。GdbCore 的导入指向
   DECoreLib.dll，10.1.0.3035 x86，SHA-256
   `a57d35741decebe0765e8942268a934990763f52813b55eadf126de7d3042c2f`。
   其 Load `0x6e955770` 读无符号 short 版本（>1 拒绝）和一个 short；
   Save `0x6e9557e0` 写版本 1 和同一成员。
2. String、byte、String、byte、String×2、byte（AfCore String）。
3. AfCore `Variant::Read(true)` 默认值。复用支持 ObjectStream 的读取器；
   BSTR 的长度沿用该序列化器的 String 布局。
4. int32×4（第一个是 esriFieldType）、byte×5。
5. LoadObject(IDomain)、LoadObject(IGeometryDef)；版本 ≥2 且字段类型等于 9
   时另读 LoadObject(IRasterDef)。请求 IID 依次为 `DF18FE0A-84BA-11D2-AB61-000000000000`、
   `439A0D50-3915-11D1-9CA7-0000F8780619`、`CDBA994F-8F18-4ABE-8A39-502BED07265E`，
   已按 esriGeoDatabase 类型库复核。Field 的 Save 版本表 `0x70aa1644`
   为 `[-1,2]`。

[读取器](../src/objects/fields.rs)接入 Fields、Field 及以上三个定义槽的空值与
回引。未知新类导致整个 Variant 槽回退，不提交集合及其字段的键。
新对象体见[值域](#idomain范围值域与编码值域)和[定义对象](#几何定义与栅格定义)。

参考工程 /Maps 及 199 个已抽取的 10.1 介质流中均未找到这两个 CLSID。
合成 ImageServerLayer v18 测试覆盖非空 Fields、Field 新对象和回引、空字段位置、
中文 BSTR 默认值、Raster 类型专属槽和 v2 尾数组；截断及未知 Domain 验证回退。
未做 ArcMap/QGIS 人工验收。

## IDomain：范围值域与编码值域

沿用上述 GdbCore.dll 映像，继续用 pyghidra 导出对象的读写方法，
用 objdump 核对 GetClassID 常量和循环计数槽。地址为 10.1.0.3035 映像 VA。

| 类 | CLSID | IPersistStream 虚表 | GetClassID | Load / Save |
| --- | --- | --- | --- | --- |
| RangeDomain | `F84C6C1A-47FF-11D2-9933-0000F80372B4` | `0x70ac5740` | `0x70934d80` | `0x70935390` / `0x70935420` |
| CodedValueDomain | `F84C6C1B-47FF-11D2-9933-0000F80372B4` | `0x70ab7f10` | `0x708d4820` | `0x708d5320` / `0x708d54c0` |

两者先读无符号 short（>1 拒绝），再调公共体
`0x708d93d0`（Save `0x708d9480`）：无符号 short 版本（>1 拒绝）、
int32×3、AfCore String×3。回到派生对象后再读 int32：
RangeDomain 紧跟两个 `Variant::Read(false)` 边界值；CodedValueDomain
紧跟 int32 数量，逐项读自由函数 `ReadString(BSTR*, IStream*)` 的标签和
`Variant::Read(false)` 的编码值。标签读取与循环次数在 `0x708d53e0..0x708d5466`
复核，反编译把计数栈槽与版本复用误合并。

`Variant::Read(false)` 在 VARTYPE 后跳过 14 字节；对称的 Write 写 14 个零字节，
见 [AfCore 取证](arcmap-10.1-variant-propertyset-slots.md#2-variantreadafcore-0x68a7c6a0导出序号-1771全貌)。
自由函数标签长度是 UTF-16 码元数（非零时另含终止零）。如果编码值本身是 BSTR，
该 Variant 内的长度仍是包含终止零的字节数。Field 的默认值依然走
`Variant::Read(true)`，没有这段填充。

两个值域都支持已证实的标量、定宽数组和 BSTR 数组。参考工程 /Maps
及 199 个介质流没有这两个 CLSID 的命中；验证用合成字节，分别覆盖 v18
影像服务图层中的非空范围域、编码域的两种字符串长度、计数和截断回退。
另以合成字段默认值和编码值验证数组及后继字段的边界。带保留区的空对象、
PropertySet 与回引也可读取，见[对象值证据](arcmap-10.1-variant-propertyset-slots.md#带保留区的-objectstream-对象值)。
未确认的 Variant 新对象类保留为停止点，数组分派见[读写分发表复核](arcmap-10.1-variant-propertyset-slots.md#值域数组的读写分发表复核)。未做
ArcMap 运行验证。

## BitMaskCodedValueDomain

介质注册 CLSID `{C4048916-410F-4DCE-9024-BE5BE6BA0A23}`。该类的
RTTI 与持久化实现位于 NetworkAnalystSolvers.dll x86 10.1.0.3035，
映像基址 `0x7b070000`，SHA-256
`a88f590573b8115a05921d624258214f9ea73baa7eaa720d182b7b5c2798234c`。
RTTI `0x7b231868`、构造器 `0x7b08f740` 和 QI `0x7b08f9f0` 互证类身份。
QI 在 `0x7b08f9f7` 比对 Field 所请求的 IDomain IID，返回完整对象
`+0x18`。委派 QI `0x7b1d1800` 返回 IPersistStream 虚基偏移
`+0x6c` 的虚表 `0x7b1fd7e8`。虚表跳板最终到达 GetClassID
`0x7b090460`、Load `0x7b0904c0` 和 Save `0x7b0906d0`。

按 objdump 核对，两侧字段序为 short 版本、三个 int32、三个 AfCore
String、int32 标签数量及一个 ExtraSafe 对象。Save 版本表 `0x7b1fc610`
的有效项为 0。Save `0x7b0907d2` 调 `0x7b091100` 创建 StrArray，
插入相同数量的标签后写出；Load `0x7b0905c3` 调 ExtraSafeReadObject
请求 IStringArray，并在 `0x7b090600..0x7b090674` 用标签索引生成
`1 << i` 的 int32 编码。位掩码通过标签顺序表达。

Rust 复用 MosaicDatasetName 的 StrArray 读取器，限制 ExtraSafe 边界，
保留原始计数与标签顺序，并在嵌套体失败时回滚整个值域。合成测试覆盖
新标签对象、宿主后继字段、数量不匹配和逐字节截断。参考工程加 66 份
10.1 介质的 /Maps 均未检出该 CLSID，未做 ArcMap 运行验证。

## Field 的 IDomain 实现范围

对 10.1 介质中 72 个可解码的 Desktop MSFT 类型库，沿 coclass
引用表与已解析接口继承核对 IDomain
`DF18FE0A-84BA-11D2-AB61-000000000000`。类型库声明了
RangeDomain 与 CodedValueDomain。对已提取的 1,115 个 Desktop DLL
扫描 IID 并复核 28 个命中模块的代码引用，共定位到三处实现侧 IID 比较：

| 模块 | 指令地址 | 实现类 |
| --- | --- | --- |
| GdbCore.dll | `0x708d3777` | CodedValueDomain |
| GdbCore.dll | `0x70934557` | RangeDomain |
| NetworkAnalystSolvers.dll | `0x7b08f9f7` | BitMaskCodedValueDomain |

其他直接引用把 IID 传给现有对象的 QI、LoadObject 或创建入口。
特别是 TableDefUI.dll x86 10.1.0.3035（SHA-256
`ecee68d2e003cb24634871a0ff1e1048140e7e27a26d185ad8e9888c84d6c247`）的
`0x1001a200` 根据 1/2/3 选择 RangeDomain、CodedValueDomain
或 `F84C6C1C-47FF-11D2-9933-0000F80372B4`，再请求 IDomain。
第三项 CLSID 位于 `0x100b3934`，在 `0x1001a2c3` 作为
AfCreateInstance 的类参数。它只在该调用方出现，未出现在同批
esri.clsid.xml、已提取的其他 Desktop DLL/OLB 或 200 个已扫描工程流中。
这证实了调用点，当前 10.1 介质没有为该候选提供可接入的持久化体。

因此已接入本次 10.1 Desktop 核对范围中确认的三种普通 IDomain
持久化实现。若今后取得旧类的实际工厂或对象体，再单独分析其布局。
目前没有这组值域的新增真实 MXD 或 ArcMap 运行验证。

## 几何定义与栅格定义

使用安装介质中已核验的 GdbCore.dll 10.1.0.3035；另外以 pyghidra 导入
RasterCatalog.dll 10.1.0.3035 x86（ImageBase `0x7be30000`，SHA-256
`0b6ac9b5e0c3e8efb715d68c5a55de89275def9ec7f666ffae91f0ff7972c585`）。
根据 RTTI、GetClassID 与 QueryInterface 交叉确认，下表为二进制 IPersistStream
入口（映像 VA）。

| 类 | CLSID | GetClassID / IPersistStream 表 | Load / Save |
| --- | --- | --- | --- |
| GeometryDef | `439A0D52-3915-11D1-9CA7-0000F8780619` | `0x709ec530` / `0x70ad688c` | `0x709ed730` / `0x709ed860` |
| RasterDef | `A8386192-3659-4525-984F-5D643A40EE8C` | `0x7be36b10` / `0x7be403fc` | `0x7be37540` / `0x7be37620` |

- GeometryDef：short 版本（无符号 >1 拒绝）、int32 类型、int32 网格计数、
  同数量的 double、int32、LoadObject(ISpatialReference)、short×2。
  Load 的空间参考 IID 在 `0x70ab8504`；与 Save 字段逐项对称。主 QI
  `0x709ec770` 把 IPersist/IPersistStream 映射到 +0x20；+0x1c 的接口
  走另一套方法，不能用其入口推断该二进制体。
- RasterDef：short 版本（原生有符号 >2 拒绝，本实现也拒绝负值）、AfCore
  String、LoadObject(ISpatialReference)、short×2、版本 ≥2 再读 short。
  Load 的空间参考 IID 在 `0x7be4047c`；Save 版本表 `0x7be3f664`
  为 `[-1,1,1,1,2]`。
- 两处空间参考 IID 均为 `4434D680-F4DA-11D1-BCA6-0000F875BCCE`，
  共享现有空间参考分派和父对象流引用表。RasterDB 的 EmptyRasterDef、
  GlobeRasterDef 主 QI `0x7bfcda30/0x7bfef4b0` 各只识别自己的接口，
  其余交给 `0x7c10a550`；检查这条静态路径未见 IPersistStream 或
  IVariantStream，不能把它们解释成有空二进制体的 RasterDef 替代类。

[定义对象读取器](../src/objects/field_defs.rs)接入 Field 两个非空槽。
/Maps 参考档和已提取的 199 个介质流无上述两个 CLSID 命中。
合成测试验证计数、RasterDef v1/v2 尾段区别、空间参考回引，以及异常计数或
截断时外层 Fields 回退且不提交定义对象的新键。未做 ArcMap/QGIS 人工验收。

### Variant 中的值域对象

GdbCore.dll 10.1.0.3035 的 RangeDomain 与 CodedValueDomain
分别经 IPersistStream 的 `0x70935390`、`0x708d5320` 读取。
AfCore 对象 Variant 在同一 ObjectStream 中按持久化接口委派，
因此共用两类值域对象体读取器，并将子 Variant 纳入同一递归限额。
合成范围值域覆盖嵌套编码值域、同键回引和截断回滚。此组合尚缺
真实 MXD 观测；ArcMap 10.1 与 QGIS 人工验收未进行。

位掩码值域也通过 NetworkAnalystSolvers.dll 10.1.0.3035 的
IPersistStream Load `0x7b0904c0` 进入共用 Variant 分派。合成用例
验证 ExtraSafe 标签顺序、宿主键表、后继字段及截断回滚。尚缺真实
MXD 中的 Variant 宿主观察，ArcMap 10.1 与 QGIS 人工验收未进行。
