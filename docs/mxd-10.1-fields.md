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
未知新类、未确认的数组元素类型及其他未识别值域类仍保留为停止点。未做
ArcMap 运行验证。

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
