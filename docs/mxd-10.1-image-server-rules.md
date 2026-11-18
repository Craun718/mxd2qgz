# ArcMap 10.1 ImageServerLayer 的镶嵌与渲染规则

证据来自安装介质的 ImageServer.dll 10.1.0.3035，PE32 x86，
ImageBase `0x77f00000`，SHA-256
`b6a7bca69fe4b36e2ff0e973e07875b33030bc957fe765644c29ed276580cb80`。
没有可复用的导出，用 objdump 与 pyghidra 补取。类由注册表项、GetClassID
及 RTTI 持久化虚表交叉确认。下表为映像 VA。

| 类 | CLSID | GetClassID / IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- |
| MosaicRule | `33638700-D926-4ECD-9CE9-BCAA96E8E89C` | `0x77f82a40` / `0x77fab0a0` | `0x77f833f0` / `0x77f83570` |
| RenderingRule | `912FC6F2-4B5F-4AA3-AF24-18704AB58F6E` | `0x77f86090` / `0x77fab59c` | `0x77f86540` / `0x77f86620` |

ImageServerLayer v19 起从 VariantStream 顺序装入这两个对象；两种规则体内
的嵌套对象用普通 ObjectStream 头，共享外层引用表。

MosaicRule 的二进制序列是 short 版本（有符号 >5 拒绝）、AfCore
String×2、`Variant::Read(false)`、AfCore String、short、int32×2、IPoint，
然后在 v2 起读 IFIDSet、v4 起读 ITimeValue。仅 v3/v4 读最后一个 short
（`0x77f83540` 的 `(u16)(version-3) <= 1`）。Save 版本表 `0x77f9ae34`
为 `[-1,-1,-1,-1,5]`：10.1 正常写 v5，三个对象槽都写，没有尾 short。
IPoint 使用 `1B077915-…`，IFIDSet 使用 `D79BDAF1-…`，ITimeValue
使用 `4CE86E17-…`（分别位于 `0x77f9c500/0x77fa1ec0/0x77fab138`）。
SystemCore.dll 的 TimeExtent 与 TimeInstant QI 均匹配 ITimeValue
（`0x8007362c`）。复用已有 Point、FeatureIDSet、TimeExtent 体；TimeInstant
复用仓外导出：IPersistStream 表 `0x800737c0` 的 Load `0x7ffea9a0` 和
Save `0x7ffeaa50` 为 short 版本、ITime、ITimeReference，本轮接入。

RenderingRule 是 short 版本（有符号 >1 拒绝）、AfCore String×2、
IRasterFunction、IRasterFunctionArguments、AfCore String、IRasterRenderer。
10.1 Save 版本表 `0x77f9ae04` 为 `[-1,-1,-1,-1,1]`。
请求 IID `CF7B7F99-…/04AA081E-…/6F5322D1-…`，分别由 esriDataSourcesRaster
和 esriCarto 类型库定名。[下节](#非空栅格函数与参数)接入 IdentityFunction 和
CachedRasterFunctionArguments；最后的渲染器支持空、回引以及现有可完整消费
的 RGB、色表对象体。其他新类保持为停止点。

参考工程 /Maps 及 199 个 10.1 介质流未找到两种规则的 CLSID。合成的
v22 宿主验证非空 MosaicRule（包括 IFIDSet、TimeInstant 嵌套 Time）、
非空 RenderingRule、两规则后的字段边界。未知函数参数会退至规则的 VARTYPE，
前一个已完成规则的键仍有效。未做 ArcMap/QGIS 人工验收。

## 非空栅格函数与参数

进一步对照安装介质中 `FunctionRasterDB.dll` 与 `RasterCoreLib.dll`
（均为 10.1.0.3035、PE32 x86）。前者 ImageBase `0x6fc20000`、SHA-256
`40af35694799c27fe2ace8bf28b12ca84a8f348122267180452291e0dd1c2701`；
后者 ImageBase `0x7beb0000`、SHA-256
`6d89601832b788177aac492e7fdfb0f1b012db86e2a75e6e480e6e647508eb1f`。
通过 objdump 核对 QI、RTTI、GetClassID、导入跳板和字段宽度，并用
pyghidra 的 RasterCoreLib 基类导出辅助复核。

- `IdentityFunction {DC71D132-E02E-4DFF-852B-7E1F10024D9B}`：
  QI `0x6fd084e0` 把 IPersistStream 映射到对象 +0x1c（表
  `0x6fd472dc`）。Load `0x6fd1eda0` 委派 `BasicFunction::Load`
  （导入槽 `0x6fd3ea9c` → `0x7beb2790`），随后只读一个 short
  类版本（>1 拒绝）。Save `0x6fcaef40` 同序委派
  `0x7beb28d0` 再写类版本，版本表 `0x6fd3f478` 为 `[-1,-1,-1,-1,1]`。
  基类按自己的 short 版本分支：v0/v1 读 int32；v2 读 int32、String；
  v3/v4 读 String×2、int32；>4 拒绝。基类写出版本表
  `0x7bf567d0` 为 `[-1,-1,-1,-1,3,4]`。
- `CachedRasterFunctionArguments {4D463957-3B81-46BA-AA8C-4654DE7D94E5}`：
  QI `0x6fc5c4a0` 同时支持 IRasterFunctionArguments 和 IPersistStream。
  后者映射到 +0x18（表 `0x6fd4087c`），Load/Save 槽
  `0x6fd2ce4a/0x6fd2ce50` 直跳 RasterCoreLib 的
  `BasicFunctionArguments::Load/Save`（`0x7beb6460/0x7beb64e0`）。
  二进制体为 short 版本（≤1）及一个 IPropertySet 普通对象槽
  （IID `F0BA6ABC-…` 存于 `0x7bf56d94`）；支持空、回引与
  PropertySet 新对象。保存版本表 `0x7bf565c4` 为 `[-1,-1,-1,-1,1]`。
  安装注册表同时列有 `RasterFunctionArguments {D553E917-…}`，
  尚未定位该 CLSID 的原生持久化入口，未套用缓存类布局。

两类都未在参考工程 /Maps 或已抽取的 199 个安装介质流中命中。
v22 合成宿主验证非空函数、参数和嵌套 PropertySet 的后继边界，并验证
未知参数或规则尾截断时，回滚整条 RenderingRule 及孙对象新键。
其他函数、参数实现仍保留为停止点，未做软件人工验收。
