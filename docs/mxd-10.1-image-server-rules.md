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
  `RasterFunctionArguments {D553E917-…}` 来自 10.8 登记表。
  10.1 安装介质复核见[登记表复核](#局部运算公开类和登记表复核)。

两类都未在参考工程 /Maps 或已抽取的 199 个安装介质流中命中。
v22 合成宿主验证非空函数、参数和嵌套 PropertySet 的后继边界，并验证
未知参数或规则尾截断时，回滚整条 RenderingRule 及孙对象新键。
其他函数、参数实现仍保留为停止点，未做软件人工验收。

## 共用持久化入口与拉伸函数

在同一版本的 FunctionRasterDB.dll 中按 RTTI 枚举函数及参数类，
对照 QI 中的接口偏移、IPersistStream 虚表与 GetClassID 的两个
8 字节常量，并用安装介质的 `esri.clsid.xml` 复核类名和 CLSID。
下表所有地址为该 DLL 的装载虚拟地址。

- 39 个函数（含 IdentityFunction）在对象 +0x1c 的 Load 槽均指向
  `0x6fd1eda0`，读 BasicFunction 后仅再读一个有符号 short（>1 拒绝）。
- 35 个参数类（含 CachedRasterFunctionArguments）在对象 +0x18 的
  Load 和 Save 槽均指向 `0x6fd2ce4a/0x6fd2ce50`，经导入槽
  `0x6fd3e9c0/0x6fd3e9bc` 直达 RasterCoreLib.dll 的
  `BasicFunctionArguments::Load/Save`。参数体中的 IPropertySet
  继续使用同一 ObjectStream 引用表。
- StretchFunction `{E0C3A6D6-8892-4076-B614-255717A5CFDF}` 的
  IPersistStream 表为 `0x6fd4f730`，GetClassID 为 `0x6fd21410`，
  Load/Save 为 `0x6fd225f0/0x6fd226f0`。Ghidra 反编译显示，
  基类体之后是有符号 short（>2 拒绝），仅 v2 再读有符号 int32 数量；
  数量为正时逐个按 IUnknown 槽装入对象。Save 的版本表
  `0x6fd3f2a8` 为 `[-1,-1,-1,-1,1,2]`。Save 调用的
  `0x6fd23190` 通过 `0x6fc9a0a0` 创建 StatsHistogram
  （CLSID `BA3027C1-49CA-4788-8B5A-3A6B387DE78C` 位于
  `0x6fd450b0`），并查询 IStatsHistogram（IID 位于 `0x6fd45360`）。
  解析复用现有 StatsHistogram 读取器，列表沿用规则的引用表。

两组共用入口同时用于 RenderingRule 与 FunctionRasterDatasetName 的
相应对象槽。表中各 CLSID 在参考工程 /Maps 和已抽取的 199 个
安装介质流中均未出现。合成的 ImageServerLayer v22 验证
StretchFunction v2 的直方图新对象及回引、StretchFunctionArguments
中的 PropertySet、规则后继字段，以及数量过大或尾部截断时的回滚。
其他持久化入口和未覆盖的嵌套对象体仍待取证。软件人工验收待补。

### 共用函数读取入口的类

| 类 | CLSID | IPersistStream 虚表 | GetClassID |
| --- | --- | --- | --- |
| AspectFunction | `D8730557-5A60-4B99-9E63-43F56CC1259A` | `0x6fd3fea0` | `0x6fc2a640` |
| BandArithmeticFunction | `D2455FD9-9A43-4630-B928-5CC89E52BF4C` | `0x6fd40160` | `0x6fc31390` |
| ClipFunction | `DFF0D3B8-FD42-4250-BE65-842828DC8E99` | `0x6fd40de8` | `0x6fc5e2f0` |
| ColormapFunction | `5A5B350D-C1F0-4E50-983C-358DBC9C40B5` | `0x6fd41324` | `0x6fc605a0` |
| ColormapToRGBFunction | `6A8D5AAE-8C8E-4B67-89B2-25E1E393557F` | `0x6fd41708` | `0x6fc67d90` |
| ColorspaceConversionFunction | `13B16685-CA39-478E-9B4C-02F12A73552E` | `0x6fd419c4` | `0x6fc6a480` |
| ComplexFunction | `7F9B9795-6D98-4F8B-AEB8-71FC8FE92E49` | `0x6fd41c8c` | `0x6fc784b0` |
| CompositeBandFunction | `B314F443-3D35-4E34-A389-C0822A187395` | `0x6fd41e10` | `0x6fc78f00` |
| ConstantFunction | `1423A723-2865-4B58-98AD-01928DAF4CE8` | `0x6fd42094` | `0x6fc79dd0` |
| ContrastBrightnessFunction | `E5986C48-1F76-4914-BA6E-5EA65231D7DF` | `0x6fd42524` | `0x6fc7b7d0` |
| ConvolutionFunction | `35DBACDC-18E0-4C5B-A087-C6C5BB692289` | `0x6fd4291c` | `0x6fc7c810` |
| DodgingFunction | `C081E7AF-4878-4845-8AE6-87D42A4435C0` | `0x6fd42e30` | `0x6fc7e790` |
| ExcludedAreaFunction | `FC9A46CE-E198-48DB-AEC2-08486D19E605` | `0x6fd433c0` | `0x6fc80b30` |
| ExtractBandFunction | `F9FE7BB2-8910-4B5A-BCB5-FE990BD3E517` | `0x6fd43884` | `0x6fc84090` |
| FootprintFunction | `4861CC7D-5D08-4AF2-B69B-49C8E90CA84B` | `0x6fd43c78` | `0x6fc86160` |
| GrayscaleFunction | `060CC78F-D651-4DF3-AA47-95E1D0D1D311` | `0x6fd46c5c` | `0x6fca28e0` |
| HillshadeFunction | `77B291D1-4581-4B4F-A802-791DE874BC89` | `0x6fd46fd4` | `0x6fca78a0` |
| IdentityFunction | `DC71D132-E02E-4DFF-852B-7E1F10024D9B` | `0x6fd472dc` | `0x6fcaef00` |
| KauthThomasFunction | `A19473EA-D62D-4610-833F-F2B3D4BCB8E4` | `0x6fd4892c` | `0x6fcbbd10` |
| MaskFunction | `A5A0861F-1B50-420C-934B-53685732F07C` | `0x6fd48b3c` | `0x6fcbddf0` |
| MosaicRastersFunction | `33C422D8-B2E5-4BA3-BDD3-2D6000E49723` | `0x6fd4a238` | `0x6fcd4120` |
| NDVIFunction | `1F3AC8E8-3011-4B87-8D4A-8150A81FDC58` | `0x6fd4a760` | `0x6fcd61b0` |
| PanSharpenGramSchmidtFunction | `47775611-00DD-44DD-B771-A50932EBCD40` | `0x6fd4add4` | `0x6fce82d0` |
| PansharpeningFunction | `B54E8861-FCF0-4642-908A-360E1BF5EA79` | `0x6fd4b248` | `0x6fced8f0` |
| RGB32ToRGBFunction | `EE26ADD9-F748-45CD-B0AE-795979090A79` | `0x6fd4d3b8` | `0x6fd08e20` |
| RGBToColormapFunction | `465187C5-C8D4-462B-AF85-2B375DD7D7E6` | `0x6fd4d4f0` | `0x6fd0a2d0` |
| RadarCalibrationFunction | `8D9F5211-F745-45F8-94CF-205152875D87` | `0x6fd4b634` | `0x6fcefa70` |
| ReflectanceFunction | `5E5C1828-ECD1-4620-9CA1-970D2AA450DE` | `0x6fd4c774` | `0x6fcfb970` |
| RemapFunction | `19156A32-B9A0-4DDF-B79F-61D74F367F93` | `0x6fd4cc40` | `0x6fcfed60` |
| ReprojectFunction | `23D7CD5B-224B-42F3-945D-13ADFBB71B9F` | `0x6fd4cfc0` | `0x6fd06150` |
| SAIFunction | `0BDC2211-71C1-48DA-9CF6-6903730E302D` | `0x6fd4d658` | `0x6fd0b390` |
| SelectionFunction | `838E348B-33C6-4853-8E51-2E056884B669` | `0x6fd4da4c` | `0x6fd0d7f0` |
| ShadedReliefFunction | `ECB818C5-BC4E-4359-B40A-D2EF9E6717C2` | `0x6fd4dd98` | `0x6fd0f6e0` |
| SlopeFunction | `BAF9A2DE-8B1A-4DC5-AEAE-138E2A7BCF3D` | `0x6fd4e610` | `0x6fd12a10` |
| SpeckleFunction | `1C11C9E1-25F1-4042-B6E8-E9953D85392F` | `0x6fd4ea50` | `0x6fd1d3c0` |
| SpectralConversionFunction | `35EEAD6E-D973-41AC-837D-F16F6023E256` | `0x6fd4ee0c` | `0x6fd1e1d0` |
| StatisticsFunction | `DD760853-675F-4770-B678-AD1600E6DB5E` | `0x6fd4f170` | `0x6fd204c0` |
| TableFunction | `A3DA7CE4-0AF9-4130-954C-EA3294F549ED` | `0x6fd4fa78` | `0x6fd262f0` |
| TrendFunction | `12038DE4-0D34-4772-9F71-A1EA4D1EA83A` | `0x6fd4fda8` | `0x6fd27160` |

### 共用参数读写入口的类

| 类 | CLSID | IPersistStream 虚表 | GetClassID |
| --- | --- | --- | --- |
| ArithmeticFunctionArguments | `073C8688-949B-4483-9F34-8289ED1D1072` | `0x6fd3fb6c` | `0x6fc23d60` |
| BandArithmeticFunctionArguments | `6F19D624-24FE-4EAC-BF1F-A117928DE308` | `0x6fd40260` | `0x6fc313e0` |
| CachedRasterFunctionArguments | `4D463957-3B81-46BA-AA8C-4654DE7D94E5` | `0x6fd4087c` | `0x6fc5ce10` |
| ClipFunctionArguments | `F72BE5E4-FA3F-4626-95CF-4A78102F5F6E` | `0x6fd40ee8` | `0x6fc5e340` |
| ColormapFunctionArguments | `CAAF0682-8040-43D1-AE21-89F9684DF9B4` | `0x6fd41424` | `0x6fc5ff80` |
| ColorspaceConversionFunctionArguments | `069B72CD-D41C-4C1F-8784-76609830AB8A` | `0x6fd41ac4` | `0x6fc6ad30` |
| ConstantFunctionArguments | `4036254E-3CC0-460E-98BE-8E9FB309F0A3` | `0x6fd421a4` | `0x6fc79e20` |
| ContrastBrightnessFunctionArguments | `634CBD5D-BB9D-4372-A908-230FB8BA5355` | `0x6fd42624` | `0x6fc7b820` |
| ConvolutionFunctionArguments | `78F85973-AFD4-4DFA-9A65-246E40EA1EF9` | `0x6fd42a1c` | `0x6fc7c860` |
| DodgingFunctionArguments | `B4C026C5-A517-460F-88FD-96AFC79BDD4F` | `0x6fd42f48` | `0x6fc7e7e0` |
| ExcludedAreaFunctionArguments | `32248838-455A-42CA-98CD-F938E502F495` | `0x6fd434c0` | `0x6fc815b0` |
| ExtractBandFunctionArguments | `E35F696D-DC8D-4274-B5A6-7A15FB0CF512` | `0x6fd43984` | `0x6fc853a0` |
| FootprintFunctionArguments | `80FF48A6-E886-4F70-8022-ABF1B9275D31` | `0x6fd43d78` | `0x6fc86730` |
| GeometricFunctionArguments | `562F6FF5-2351-42D1-B464-BA03DE92E7EF` | `0x6fd46950` | `0x6fc9ff20` |
| GrayscaleFunctionArguments | `CD0E53BF-6D15-48B6-A7C4-3971365813AA` | `0x6fd46d5c` | `0x6fca2f50` |
| HillshadeFunctionArguments | `80186241-15DF-4C3A-94DA-50FA3F73FCB2` | `0x6fd470d4` | `0x6fca8430` |
| MaskFunctionArguments | `350D8526-9FAD-4842-A77D-B2A6EC143DBF` | `0x6fd48c3c` | `0x6fcbde40` |
| MosaicFunctionArguments | `62C333ED-A776-4A2E-8062-1959C70D2704` | `0x6fd49b00` | `0x6fcc51c0` |
| MosaicRastersFunctionArguments | `94F10296-5868-4B2D-9596-E17BDD077E25` | `0x6fd4a348` | `0x6fcd4fd0` |
| NDVIFunctionArguments | `F7871DDD-26CD-40D1-B86E-F5308E3828E2` | `0x6fd4a860` | `0x6fcd74a0` |
| PanSharpenGramSchmidtFunctionArguments | `767381FF-3ECC-4685-8997-4F2E0F523CEC` | `0x6fd4aed4` | `0x6fce5640` |
| PansharpeningFunctionArguments | `09BF8DB7-1A82-4AD2-A366-AEE762487F75` | `0x6fd4b348` | `0x6fceeb70` |
| RadarCalibrationFunctionArguments | `A78DFA71-D2D3-4766-85DD-C70C17DBE6B3` | `0x6fd4b734` | `0x6fcef030` |
| RemapFunctionArguments | `86ACB7D5-A0E7-4345-9FBF-8BF59D5C87B2` | `0x6fd4cd40` | `0x6fcfedb0` |
| ReprojectFunctionArguments | `453FBF56-CA41-410D-92CF-FB727CFDC1D1` | `0x6fd4d0c0` | `0x6fd07410` |
| SAIFunctionArguments | `314E643B-44A2-43AD-9374-6F4EEADE11E9` | `0x6fd4d758` | `0x6fd0b910` |
| SelectionFunctionArguments | `A7340AA2-4B37-4C71-9CAC-2C47D8AC1E09` | `0x6fd4db4c` | `0x6fd0ddc0` |
| ShadedReliefFunctionArguments | `C0C4F4DB-7F2A-4050-A1E2-89CD7868C43A` | `0x6fd4de98` | `0x6fd0e5c0` |
| SlopeFunctionArguments | `C39388B1-0EB2-4B50-97F6-47BEB2C60D86` | `0x6fd4e710` | `0x6fd140b0` |
| SpeckleFunctionArguments | `E0A62507-3AB0-42E9-B3A0-A58CD1FF0D6E` | `0x6fd4eb50` | `0x6fd1e050` |
| SpectralConversionFunctionArguments | `F5E78815-251E-4008-9C5B-F63ABDB3799E` | `0x6fd4ef0c` | `0x6fd1e220` |
| StatisticsFunctionArguments | `C26AAC72-5D86-482E-96D4-D56986EBF09C` | `0x6fd4f270` | `0x6fd20510` |
| StretchFunctionArguments | `D8261E56-178A-4A52-915B-788F9393951C` | `0x6fd4f830` | `0x6fd21460` |
| TableFunctionArguments | `4C912271-D13A-43BF-ADDE-722902DE115F` | `0x6fd4fb78` | `0x6fd26870` |
| TrendFunctionArguments | `954E5BB5-2892-49CB-9BF6-033F5A78705B` | `0x6fd4fea8` | `0x6fd27820` |

## 独立入口的四个函数与 Reflectance 参数

继续检查 FunctionRasterDB.dll 10.1.0.3035 在函数对象 +0x1c
和参数对象 +0x18 的 IPersistStream 虚表。用 objdump 逐条检查
Load/Save 的调用、GetClassID 的常量以及安装介质注册表。下列地址均为
该模块的装载虚拟地址。

| 类 | CLSID | 虚表 / GetClassID | Load / Save |
| --- | --- | --- | --- |
| ArithmeticFunction | `04E0F604-76F5-4A7A-A1C4-57F54B9CC4AF` | `0x6fd3fa6c / 0x6fc232d0` | `0x6fc23310 / 0x6fc23360` |
| CachedRasterFunction | `ACA03769-2D4E-484D-8E14-1BC4ACB17A59` | `0x6fd40768 / 0x6fc5c020` | `0x6fc5c060 / 0x6fc5c0d0` |
| GeometricFunction | `1013A267-0E0E-428F-8816-5551F450D711` | `0x6fd46828 / 0x6fc9fed0` | `0x6fca0e70 / 0x6fca0ec0` |
| RGB32Function | `95CD2068-42A4-43EE-BF08-7B2E562A9B47` | `0x6fd4d258 / 0x6fd07e00` | `0x6fd07e40 / 0x6fd07e80` |
| ReflectanceFunctionArguments | `082DC874-8ACC-4FD8-A1D1-AEAF73CA899A` | `0x6fd4c874 / 0x6fcfb9c0` | `0x6fcfd530 / 0x6fcfd540` |

- ArithmeticFunction 装入 BasicFunction 后只读一个 v1 short；Save
  使用版本表 `0x6fd3f5bc`。
- GeometricFunction 装入 BasicFunction 后只读派生 short，上限为 2；
  Save 使用版本表 `0x6fd3f4a4`。
- CachedRasterFunction 装入 BasicFunction 和派生 short（上限 1）后，
  无条件读取两条 AfCore String，位于 IPersistStream 子对象的
  `+0x44/+0x4c`。读写分别经导入槽 `0x6fd3e05c/0x6fd3e080`，
  Save 使用版本表 `0x6fd3f590`。两串业务名称尚未核对。
- RGB32Function 仅持久化自身的 short（上限 1），Save 使用版本表
  `0x6fd3f334`。解析结果用 `has_basic_function` 表示基类体是否存在。
- ReflectanceFunctionArguments 的独立 Load/Save 桩最终分别跳至
  `0x6fd3e9c0/0x6fd3e9bc` 对应的 BasicFunctionArguments 入口；
  因此复用 short 和 IPropertySet 槽读取器。

合成 RenderingRule 验证四种函数后均可继续读取 Reflectance 参数与规则的
剩余字段，并覆盖超版本拒绝。在参考工程 /Maps 和 199 个安装介质流
中均未发现上述 CLSID。ArcMap/QGIS 人工验收待补。

### MergeRastersFunction 的压缩负载

两个独立入口都在 FunctionRasterDB.dll 10.1.0.3035。

| 类 | CLSID | IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- |
| MergeRastersFunction | `96E60D70-6BF9-4486-B9ED-C41181259F71` | `0x6fd48e50` | `0x6fcc23e0 / 0x6fcc2600` |
| MergeRastersFunctionArguments | `9A388281-B984-4BD3-B799-348D01A05605` | `0x6fd48f60` | `0x6fcc3290 / 0x6fcc3570` |

- 函数先读 BasicFunction，再读派生 short（上限 2）。v2 再读一个
  `Variant::Read(false)`。解压后的字段依次为 IRasterInfo2 槽、int32
  数量及每项四个 double。IRasterInfo2 的 IID `28E745A8-0617-4EB0-B8BD-272A945ED117`
  由安装介质类型库定名，位于 `0x6fd49038`。Save 版本表
  `0x6fd3f438` 的 10.1 项为 2。
- 参数先读派生 short（上限 2），再调用 BasicFunctionArguments::Load，
  最后读两个压缩 Variant。解压后的每段数据各含一个 IUnknown 槽，
  由属性读取路径延迟打开，见下文。
  `0x6fcc339b/0x6fcc345f` 分别调用 `0x6fc5d070` 创建独立
  ObjectStream 后，各自用 `+0x38` 连接解压内存流。函数的
  `0x6fcc244c` 同样创建内层 ObjectStream。类工厂实参
  `0x6fd40964` 与 QI `0x6fd40a94` 分别是 ObjectStream CLSID、
  IObjectStream IID。传入外层 ObjectStream 的对象键不参与内层分派。
- RasterCoreLib.dll 的 `CompressBuffer/UncompressBuffer`
  （`0x7bf34100/0x7bf34380`）写两个 4 字节长度（原始与压缩）和
  压缩数据。SystemCore.dll（SHA-256
  `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）
  的 MemoryBlobStream IBlobStream 表 `0x80070994` 上，
  `0x7ffcc360/0x7ffcc400` 写入和检验 `VT_ARRAY | VT_UI1`（0x2011）。
  同模块的 Compression 的 IZlibCompression 表 `0x8006e0b8`
  将压缩和解压委派给导入的 `compress2/uncompress`
  （调用点 `0x7ffba34a/0x7ffba418`），使用 zlib 封装。
- Rust 读取器对压缩与解压大小应用宿主字节限额，验证完整 zlib 流
  及实际读写字节数。IRasterInfo2 为空时直接解析后续矩形数据。
  已支持的非空 RasterInfo 会与后继矩形一起解析。遇到未知信息对象或
  未支持的嵌套类型时，保留其头及解压边界内的剩余字节。

合成 RenderingRule 测试覆盖 v2 三段压缩数据、独立键、矩形、未知对象原值、
全部截断、失效校验和与长度异常。两种 MergeRasters CLSID 在参考工程
/Maps 及 199 个安装介质流中均无命中。ArcMap/QGIS 人工验收待补。

#### Rasters 和 Footprints 的压缩参数对象

复核同一 Desktop 10.1.0.3035 PE32 FunctionRasterDB.dll，SHA-256
`40af35694799c27fe2ace8bf28b12ca84a8f348122267180452291e0dd1c2701`。
静态初始化 `0x6fd3ba80 / 0x6fd3baa0` 把 UTF-16 字符串 `Rasters`、
`Footprints` 放入 `0x6fd7c064 / 0x6fd7c068`。
`Load 0x6fcc3290` 用有符号 short 比较派生版本上限 2，
读取基类及两个压缩 Variant 后，按上述顺序把两个独立 ObjectStream
作为对象值写入属性集（`0x6fcc3406 / 0x6fcc34bf`）。

两项属性访问器为 `0x6fcc2c70 / 0x6fcc2e40`，来自
IMergeRastersFunctionArguments 虚表 `0x6fd48fcc` 及 10.1 类型库。
如属性持有流，两者分别在 `0x6fcc2d0e / 0x6fcc2ede` 对流调用
LoadObject，请求 `0x6fd3f0ac` 的 IUnknown，随后将装入的对象替换到
属性集。Save 从这两个对象值调用 ConvertFunctionArgument，
在 `0x6fcc36cf / 0x6fcc3756` 各向独立 ObjectStream 写一个对象。
保存版本表 `0x6fd3f444` 的 10.1 项是 2。

Rust 在有界解压流中复用有键对象和栅格模板输入的已知体分派。已读完的
参数同时保留对象体原字节，未知类或未知嵌套仍保留解压后的边界及字节。
两个压缩输入各自登记对象键，递归读取沿用宿主剩余的栅格深度限额。
构造 RenderingRule 验证 Rasters 的栅格名称、Footprints 的 GeometryBag
及子 Point 回引、同键号的独立分派、有符号版本、尾随字节拒绝和截断。
实际保存的参数类集合和对应实档仍待取证，ArcMap/QGIS 人工验收未做。
135 份对照工程均读至 /Maps 和 /PageLayout 末尾，270 条流中未命中
MergeRastersFunction 或 MergeRastersFunctionArguments 的 CLSID。

压缩输入中的已知函数及参数类在共享 Variant 扩展后仍沿用栅格输入
分派。其独立对象表和宿主剩余深度预算一同传入读取器。合成压缩内层体
验证 IdentityFunction 有预算时完整读取、预算耗尽时返回错误。

### RasterInfo 二进制对象体

RasterInfo（CLSID `4F1E5FCA-2334-4280-B109-17B3CAF9290E`）的
QI `0x6fcf59e0` 将 IPersistStream 映射到对象 +0x14 的虚表
`0x6fd4c070`。GetClassID 为 `0x6fcf5610`，二进制 Load/Save
为 `0x6fcf6de0/0x6fcf71f0`，另有独立的 +0x1c XML 虚表。
Save 版本表 `0x6fd3f370` 为 `[-1,-1,-1,-1,6,7]`。

Load 按 short 版本（至 7）读取 IGeodataXform、IEnvelope、
ISpatialReference、IEnvelope 四槽，然后读两个 double、两个 int32、
Variant(false)、四个 int32 和 short。v2 增加 IPoint，v3 增加
AfCore String，v4 增加分别计数的统计及直方图对象序列，v5 增加
IRasterColormap，v6 增加请求 IPnt（esriGeoDatabase 类型库 IID
`55614793-3490-11D3-A627-0008C7D3AF56`）
的普通对象槽（常量 `0x6fd49cb8`），v7 再读 int32。初始槽和点
复用现有读取器。v6 新对象类体待补，空与回引可继续。

RasterDB.dll 10.1.0.3035（SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`）
的 RTTI、虚表和 objdump 对位结果如下。两个序列的请求 IID
`7E7E6103-FA34-11D1-8D22-0000F8780535` 与
`7ACA3D0B-0629-11D2-8D24-0000F8780535` 分别由 10.1 类型库定名为
IRasterStatistics 和 IRasterHistogram。

| 类 | CLSID | IPersistStream 虚表 / Load | 二进制对象体 |
| --- | --- | --- | --- |
| RasterStatistics | `8446EB83-3251-4F20-8B79-1D5421E39D4A` | `0x7c1346ec / 0x7c0cb940` | short（≤2）、六个 double、两个 int32、Variant(false)，v2 追加一个 Variant(false) |
| RasterHistogram | `B01293EA-88D0-4E7D-AF82-A0B10BAC9B33` | `0x7c134184 / 0x7c0c82e0` | short（≤1）、Variant(false)、三个 double、一个 int32、int32 个数及对应的 double 序列 |
| RasterColormap | `92195722-5732-11D2-AAAD-00C04FA375FB` | `0x7c131fec / 0x7c09cdb0` | short（≤2），v2 先读一个 Variant(false)，各版本再读一个 Variant(false)；QI `0x7c09b910` 匹配宿主 v5 的 IID `7E7E6105-FA34-11D1-8D22-0000F8780535` |

已从 RasterInfoFunction（`0x6fcf98e0/0x6fcf9960`，基类体、
派生 short 后装入 IRasterInfo）及合并函数的内层流两处接入。
合成 v7 包含同键回引、非空统计、直方图、色表与末尾 int32，覆盖两个
宿主的后继边界。两类 CLSID 在参考工程 /Maps 和 199 个安装介质流
中无命中，软件人工验收待补。

### MosaicFunction、MosaicLODInfos 与 FIDSet

FunctionRasterDB.dll 10.1.0.3035 的 MosaicFunction CLSID
`7406B10E-6009-4366-85FF-5A067B54B2AD` 来自虚表 `0x6fd49864`、
GetClassID `0x6fcc5050`。Ghidra 导出的 Load/Save
`0x6fcc9820/0x6fcc9c00` 互证字段顺序，保存版本表 `0x6fd3f42c`
为 `[-1,-1,-1,-1,6,9]`。Load 先委派 BasicFunction，再读派生 short
（上限 9）。以下各对象槽共用宿主引用表。

| 版本 | 派生 short 后的顺序 |
| --- | --- |
| 各版本 | IRasterInfo、五条 String、IMosaicLODInfos、两个 double、short、ISymbol、int32、String、Variant(false)、两条 String、IPoint、short、int32、两条 String |
| ≥2 | short |
| 3、4 | IPnt |
| ≥4 | IGeometry |
| ≥6 | IFIDSet |
| ≥7、≥8、≥9 | 各追加一个 short |

IMosaicLODInfos IID `DFD25C61-79F5-4735-8CBC-1606947ACFDE` 由
esriDataSourcesRaster 类型库与 MosaicLODInfos 的 QI `0x6fcd2440`
互证。MosaicLODInfos 的 CLSID 为
`E3F15E14-512A-49AF-8078-EF31813D6EAB`，
IPersistStream 虚表 `0x6fd4a098`，Load/Save
`0x6fcd2860/0x6fcd2970`，按 short（≤1）、int32 个数、按项两个
double、最后两个 double 读取。

IFIDSet 使用 esriGeoDatabase IID
`D79BDAF1-CAA8-11D2-B2BE-0000F878229E`，实际实现 FIDSet 的
CLSID 为 `D79BDAF0-CAA8-11D2-B2BE-0000F878229E`。GdbCore.dll
10.1.0.3035（SHA-256
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`）
的虚表 `0x70ad5744` 指向 Load/Save `0x709da710/0x709da7d0`。
short（≤1）、int32 数量、逐项 int32 与 FeatureIDSet 同形。
Load 循环通过 `0x709d9ac0` 加入成员，重复值合并且负 OID
被拒绝，Load 忽略单个 Add 的返回值。MosaicFunction 和
MosaicRule 两处 IFIDSet 槽接入该 FIDSet 实现。

MosaicFunction v3/v4 与 RasterInfo v6 请求同一个 IPnt IID。72 份
10.1 介质类型库中，公开实现为 Pnt（`064087E4-09B9-4892-A893-FCA89BB171D1`）
和 DblPnt（`1F1A2725-6B9D-44A5-B75E-9E4CD9B9F90A`）。在同一
RasterDB.dll 映像中，它们共用主 QI `0x7c0337c0`，两个直接比较项
`0x7c12add4 / 0x7c12ecf8` 分别为 IPnt 与
`DA8CBCBE-02C9-47B8-844E-71B502773284`。后备 `0x7c10a550` 只比较
`0x7c12215c / 0x7c1221dc / 0x7c1368e4`，未命中返回
E_NOINTERFACE。两张主表 `0x7c12ebd0 / 0x7c12ec3c` 均没有可用于
普通 ObjectStream 新对象的 IPersistStream 或 IPersistVariant 分派。
IPnt 空槽和回引可解析，其他实现写出的非空新对象仍待原生证据。
参考工程及 199 个介质流未发现前述 MosaicFunction、MosaicLODInfos、
FIDSet 和 Pnt 四种类的标识。
合成 v1–v9 覆盖版本分支、非空 MosaicLODInfos、FIDSet 的重复及负 ID、
后续 RenderingRule 字段、尾部截断和计数超限。ArcMap/QGIS 人工验收待补。

### RasterFunctionTemplate 与 RasterFunctionVariable

FunctionRasterDB.dll 10.1.0.3035 的三类虚表、加载入口和类标识
经 RTTI、安装注册表、objdump 与 Ghidra 对位。

| 类 | CLSID | IPersistStream 虚表 / GetClassID | Load / Save |
| --- | --- | --- | --- |
| RasterFunctionTemplate | `0B67BD89-4783-4A6F-BE52-143DB3660C11` | `0x6fd4bc54 / 0x6fcf2f70` | `0x6fcf2fb0 / 0x6fcf31b0` |
| RasterFunctionVariable | `1799317A-C5AB-4AEE-8C99-96A5CF1BA0E6` | `0x6fd4bee8 / 0x6fcf45c0` | `0x6fcf4e30 / 0x6fcf4f20` |
| RasterFunctionTemplateArguments | `E7E9FA4F-FCC6-483D-9450-FF38B6A2C315` | `0x6fd4bdd4 / 0x6fcf4310` | `0x6fd2ce4a / 0x6fd2ce50`（共用参数体） |

- 模板的 QI `0x6fcf2700` 匹配 IRasterFunction IID
  `CF7B7F99-3698-4F1F-AFF8-E3430E106B1E`。读取 BasicFunction
  和派生 short（≤3）后，读取 String、一项 IRasterFunction、
  有符号 int32 输入数、对应的 IUnknown 槽。非正数走单个输入槽；
  正数逐个读取。v2 起追加 String 和 IStringArray，v3 再追加
  String。保存表 `0x6fd3f388` 为 `[-1,-1,-1,-1,2,3]`。
- 变量读取 short（≤2）及 String。v2 追加 Variant(false)、String、
  short 和 IStringArray。保存表 `0x6fd3f37c` 为
  `[-1,-1,-1,-1,2]`。Save 的 `0x6fcf4fbf` 经导入槽调用
  RasterCoreLib 的 ConvertFunctionArgument，再写出 Variant。
  变量里的已知对象值可复用栅格名称、参数、变量或 PropertySet 读取器。
- 两处 IStringArray 槽均请求 `206216C9-7253-4A4D-92DD-60329FD9D792`，
  由 esriSystem 类型库定名，复用现有 StrArray 读取器。
  所有普通对象槽使用同一宿主键表。模板和栅格名称可能相互嵌套，
  两条分派共用有限的递归深度。

合成 v3 规则覆盖子函数、变量中非空栅格名称、变量及 StrArray 回引、
负计数单项、超限计数和递归限额。参考工程 /Maps 及 199 个介质流
均无这三个 CLSID 的命中。其他 IUnknown 类、尚未接入的嵌套对象及
ArcMap/QGIS 人工验收仍待补。

### 外部模块的栅格函数与参数

从 Desktop 10.1 安装介质对三份 PE32 模块核对 ProductVersion
`10.1.0.3035`、RTTI、IPersistStream 虚表及 GetClassID 常量。
SHA-256 分别为：

- 3DToRasterFunctions.dll（安装介质文件名带前置下划线）
  `70bf29f47ce98c1482552be6b215781a0e6cb30cac88eacb8f408b9f686a1901`。
- SARasterFunction.dll
  `5b8938501daae70bbd3185a726c233813e6eb4456b72032d5b080b0b4e27637d`。
- RasterRenderer.dll
  `cc56825cdf91d4b5aed88d0f766ee253ad30000c3079c40a21a2e9e6c6ce1390`。

| 函数及参数类 | 函数 CLSID / 参数 CLSID | 函数 IPersistStream 虚表 / Load | 参数虚表 / Load |
| --- | --- | --- | --- |
| LasDatasetToRasterFunction / Arguments | `06A78A67-BA8A-4592-83E5-5DD4362F6636` / `67876AA0-40EB-470A-ABC9-036085756C53` | `0x681e0994 / 0x681cdd30` | `0x681e0ab4 / 0x681db0b2` |
| LasToRasterFunction / Arguments | `9889D8FB-93F6-49B1-B352-D8E2B331C797` / `FB8E343B-8014-4CE9-B457-E72389D2B339` | `0x681e14b8 / 0x681cdd30` | `0x681e15d8 / 0x681db0b2` |
| TerrainToRasterFunction / Arguments | `D4D3A0BB-3770-40EF-ACF1-BB3C1C98A38E` / `8BF576CA-96C8-4106-9020-A2D78D221DC4` | `0x681e19b4 / 0x681cdd30` | `0x681e1ad4 / 0x681db0b2` |
| MLClassifyFunction / Arguments | `5803078B-AF97-4388-9B9E-7657AE76A310` / `C1570587-E9EA-43CC-B938-545B1EF289FE` | `0x7d0e3470 / 0x7d0b65f0` | `0x7d0e35ac / 0x7d0bf3dc` |
| RenderedRasterFunction / Arguments | `EED089C5-98F5-496D-903C-CE3F2A1956DC` / `A6E9CA22-4C2B-4B4C-AEA5-DA37566C3130` | `0x7c9b4b74 / 0x7c988f50` | `0x7c9b4c74 / 0x7c9a9888` |
| LocalFunctionImpl / LocalFunctionArgumentsImpl | `3ECB15F2-6443-443E-9228-659328507FEE` / `3B4A5E5B-4ACE-4219-BAA9-289B5C431E74` | `0x7d0dad40 / 0x7cd698a0` | `0x7d0db0f0 / 0x7cd6b0a0` |

前五个函数入口通过各模块的 `0x681df42c`、`0x7d0da368`
或 `0x7c9b0540` 调用 RasterCoreLib 的 BasicFunction::Load，
随后读取一个 short（上限 1）。前五个参数入口分别经
`0x681df330`、`0x7d0da404` 或 `0x7c9b03fc` 直接跳转
BasicFunctionArguments::Load，共用其属性集槽；Save 跳板对称。

LocalFunctionImpl 装入 BasicFunction 和派生 short（上限 1）后，
在 `0x7cd698f6` 读取一个 int32，并在 `0x7cd69911` 将这个值
传给对象内部接口。Save `0x7cd69930` 在 `0x7cd699d2` 写入它。
LocalFunctionArgumentsImpl 在 BasicFunctionArguments 体之后读取派生
short（上限 1），Save 对应 `0x7cd6b0f0`。局部运算 int32 的枚举
尚未定名，按原值保留。

SARasterFunction 的 MemoryPixelsIntersectFunction 也响应
IRasterFunction；QI `0x7d0b4760` 将 IPersistStream 映射至
+0x20 虚表 `0x7d0e317c`，其 Load/Save
`0x7d0b5240/0x7d0b5250` 都直接返回 `E_FAIL`。这条路径没有
可用对象体。ElevationVoidFill 和 VectorFieldRendererFunction 相关类的版本归属
见[登记表复核](#局部运算公开类和登记表复核)。

参考工程 /Maps 和 199 个安装介质流没有前述十二个新增 CLSID
的命中。合成 RenderingRule 覆盖 LAS 与局部运算代表对象、非空属性集、
外层后继字段和超版本拒绝。未做 ArcMap/QGIS 人工验收。

### 函数集合、栅格项与三个参数体

2026-11-19 核对 10.1.0.3035 PE32（原址），从 RTTI
IRasterFunction / IRasterFunctionArguments 基类定位实现，再以虚表、
GetClassID、导入表、类型库及 objdump 对照持久化路径。
FunctionRasterDB.dll SHA-256 为
`40af35694799c27fe2ace8bf28b12ca84a8f348122267180452291e0dd1c2701`，
ImageServerLayer.dll 为
`0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`。

| 类及 CLSID | IPersistStream 虚表 / GetClassID | Load / Save |
| --- | --- | --- |
| RasterFunctionCollection `1F120F8F-17AA-4B8E-8CD8-6007E0DEDB80` | `0x6fd4b8f0 / 0x6fcf08d0` | `0x6fcf0910 / 0x6fcf0a00` |
| RasterItemFunction `31E67A19-ED23-4825-BF1E-6A7D7B0EF190` | `0x78033368 / 0x78021d70` | `0x78021db0 / 0x78021e30` |
| RasterItemFunctionArguments `A2A534D3-D721-4C8C-BFAF-98C6D835DAF2` | `0x78033478 / 0x78020c70` | `0x78026210 / 0x78026216` |
| RasterFunctionTemplateArguments `E7E9FA4F-FCC6-483D-9450-FF38B6A2C315` | `0x6fd4bdd4 / 0x6fcf4310` | `0x6fd2ce4a / 0x6fd2ce50` |
| RasterInfoFunctionArguments `877BD868-EE05-4493-B586-BB71681F0665` | `0x6fd4c4e4 / 0x6fcf8aa0` | `0x6fd2ce4a / 0x6fcfb180` |

RasterFunctionCollection 体从 short 版本开始（有符号上限 1），接有符号
int32 计数与普通 IRasterFunction 槽循环。Load `0x6fcf0944..0x6fcf0999`
直接读取这些字段，Save `0x6fcf0a47..0x6fcf0aa7` 对称，无基类体。
计数非正时原生不进循环。正计数的槽返回后原生直接调用该对象的 AddRef，
所以 Rust 对空子函数报告非法数据。子函数共享宿主键表和栅格函数的递归深度
限额。栅格项先经 IAT `0x7802c46c` 读 BasicFunction，再读派生 short
（上限 1）和 IRasterInfo 槽。`0x78033518` 的 IID 为
`94058B85-ACF1-49BA-A2A7-A9CCC6F4C26A`，复用现有 RasterInfo 体读取器。

三个参数类均经导入桩委派 BasicFunctionArguments::Load，复用 short
版本和 IPropertySet 槽。RasterInfoFunctionArguments 的 Save 对内存属性集
做临时操作，再于 `0x6fcfb1da` 调 BasicFunctionArguments::Save，之后
恢复内存值。该段没有额外的流字段。

构造 RenderingRule 测试覆盖集合内的非空 RasterItemFunction 和
RasterInfo、子函数回引、派生 v0 的负计数集合、三个参数类的属性集、
后续表达式字符串与渲染器空槽。逐前缀截断、空子函数和过大计数均报错。
135 份本地工程的 /Maps 与 /PageLayout 都完整读取。这 270 条流中未命中
五类 CLSID 候选，尚无实际 RenderingRule 宿主字节或 ArcMap/QGIS 人工验收。

外部公开局部运算类的委派关系见[登记表复核](#局部运算公开类和登记表复核)。

### MosaicDB 的属性集参数

2026-11-19 使用安装介质 MosaicDB.dll 与 RasterCoreLib.dll 的
10.1.0.3035 PE32。SHA-256 分别为
`424814c3eceb88871df221bb42cb420b61c79c110a1e46a0e05af744db0a39ed` 和
`6d89601832b788177aac492e7fdfb0f1b012db86e2a75e6e480e6e647508eb1f`。
按 RTTI 的 IRasterFunctionArguments 与 IPersistStream 接口偏移
相交定位 70 个类，读取持久化虚表的 GetClassID、Load、Save，
用类型库与 objdump 复核。五个基类的 GetClassID 进入
`0x7a5c33f2`，该跳板转入 CRT `_purecall`，未纳入类表。
支持类的 CLSID 和版本范围见
[src/layers/image_server_rules/mosaic_arguments.rs](../src/layers/image_server_rules/mosaic_arguments.rs)。

MosaicDB 的 IAT `0x7a5f5eb0 / 0x7a5f5eac` 分别导入
RasterCoreLib 的 BasicFunctionArguments Load/Save。Load `0x7beb6460`
读取 short（上限 1）和普通 IPropertySet 槽。下表的派生版本字段
与基类版本是独立的 short。

| 类组 | 类数 | MosaicDB Load | 体序与派生版本 |
| --- | --- | --- | --- |
| 镶嵌作业参数及通用过滤器 | 25 | `0x7a441220`、`0x7a3d3d40`、`0x7a5a013c` | 只有基类体 |
| 文件及对象遍历器、MosaicDatasetJob | 10 | `0x7a40c480` | 基类体、派生 v1 |
| URIPropertiesFilter | 1 | `0x7a424070` | 派生 v0–1、基类体 |
| ISDefBuilder、RPDefBuilder | 2 | `0x7a490c10` | 基类体、派生 v2 |
| LasBuilder、LasDatasetBuilder、TerrainBuilder | 3 | `0x7a4bf2e0` | 基类体、派生 v0–2 |
| Applanix、Landsat 等构建器 | 12 | `0x7a4ccf90`、`0x7a4d20c0` | 基类体、派生 v0–1 |
| RasterDatasetBuilder、TableBuilder | 2 | `0x7a4dfb60`、`0x7a500bf0` | 基类体、派生 v3 |

表中构建器先经 `0x7a476660` 调基类读取，再从内存属性集同步成员。
其 Save 经 IAT 写基类体及派生 short。需要按有符号上限和精确版本分别
处理，例如 `0x7a40c4af..0x7a40c4b8` 实际要求派生值等于 1，
`0x7a4dfb8b..0x7a4dfb99` 要求等于 3。实现保留派生版本，
基类属性集沿用宿主对象表。RenderingRule 和模板输入共用参数分派。

合成 RenderingRule 对七种布局分别核对属性集、新键和对象尾部，
并覆盖允许的旧版本、被拒绝的派生版本和截断。135 份工程的 270 条
/Maps 和 /PageLayout 流均读至末尾，未命中这 55 个 CLSID。
这批类已具备原生持久化与接口证据，实际 RenderingRule 宿主、非空
属性集可能引入的其他对象类和 ArcMap/QGIS 人工验收仍待取证。

### MosaicDB 的旧版槽与内联参数

同一 MosaicDB.dll 10.1.0.3035 中，以下八个类需要按派生版本读取额外字段，
或直接调用成员对象的 IPersistStream Load。槽与内联对象中的属性集使用宿主引用表。

| 类 | Load / Save | 字段顺序 |
| --- | --- | --- |
| RasterDatasetCrawler | `0x7a411190 / 0x7a411260` | 基类体、派生 v1–v3。v3 追加 int32、IName 槽 |
| TableCrawler | `0x7a413af0 / 0x7a413cb0` | 基类体、派生 v1–v3。v1 追加 IName 槽，v2 追加 IName 槽和 int32 |
| HREBuilder | `0x7a4ca770 / 0x7a47e9a0` | 内联 RasterDatasetBuilder、外层 short，按有符号值检查上限 2 |
| NITFBuilder | `0x7a4ca770 / 0x7a4ca800` | 内联 RasterDatasetBuilder、外层 short，按有符号值检查上限 2 |
| PixFileBuilder | `0x7a4d0450 / 0x7a4d04e0` | 内联 RasterDatasetBuilder、外层 short，按有符号值检查上限 1 |
| RPFBuilder | `0x7a4f4c50 / 0x7a4f4cf0` | 内联 RasterDatasetBuilder、外层 short，按有符号值检查上限 940 |
| AddRastersParameters | `0x7a56bcf0 / 0x7a56bd60` | 基类体、内联 SynchronizeParameters |
| URIProductNameFilter | `0x7a423440 / 0x7a4233e0` | 派生 short。值 ≤1 时读取 AfCore String、short、两个 IStringArray 槽。值 2–940 时读取基类体 |

两个遍历器在 `0x7a4111ed`、`0x7a413bc9` 请求 IName（IID 位于
`0x7a5ff008`）。URIProductNameFilter 在 `0x7a423534`、
`0x7a42359b` 请求 IStringArray（IID 位于 `0x7a5ff048`），
Save 根据 `0x7a5f8640` 写 v940 与基类体。旧版直接字段保存原始 short
位模式，`has_basic_arguments` 标记流上是否有基类体。

四个构建器的构造路径在 `0x7a47b5b1`、`0x7a4c8be1`、
`0x7a4cfca7`、`0x7a4f3041` 指定 RasterDatasetBuilder
的 CLSID 常量 `0x7a5fe808`。Load 经 `0x7a514ac0` 请求
IPersistStream（IID `0x7a6017f0`），在同一流上直接调用
虚表 `+0x14` 的 Load。RasterDatasetBuilder 体为基类体加派生 v3。
AddRastersParameters 构造路径在 `0x7a56ba71..0x7a56ba85`
指定 SynchronizeParameters 的 CLSID 常量 `0x7a5fea38`，
Load 在 `0x7a56bd27..0x7a56bd36` 使用同样的直接调用。
SynchronizeParameters Load `0x7a441220` 和 Save `0x7a50aa30`
直接转入基类入口。两种直接调用均无单独的成员 CLSID 或引用键。

合成 RenderingRule 验证两种遍历器的三个版本、新名称对象、内联成员中
的跨体属性集回引、旧版过滤器的数组回引、有符号版本分支及逐字节截断。

### RasterType 和 OverviewItemURI

RasterType Load `0x7a431610` 先读取基类属性集，再读取普通 IName 槽。
`0x7a442310` 对所得名称请求 IRasterTypeName，类型库对应的具体实现
是 RasterTypeName。派生 short 在 `0x7a4316e9..0x7a4316f4`
按无符号区间检查 2–943。只有 v943 继续读取两个普通对象槽，
`0x7a600178` 按类型库对位 IAddRastersParameters，
`0x7a600198` 对位 ISynchronizeParameters。类型库列出
AddRastersParameters 同时实现两种接口，SynchronizeParameters 实现后者。
Save `0x7a431820` 的版本表 `0x7a5f85ec` 含 943。
合成 v2、v942、v943 覆盖非空名称、添加栅格参数对象及同步参数槽回引。

OverviewItemURI Load `0x7a41da70` 在基类体之后连续调用三次
IPersistStream Load，Save `0x7a41db70` 对称写入。构造路径
`0x7a41c98a..0x7a41ca08` 使用 CLSID 常量
`0x7a5fe798`、`0x7a5fe7d8`、`0x7a5ffce8`，分别对应 ItemURI、
CellSizeLevel、FunctionRasterDatasetName。三个成员内联，无单独的对象头。
ItemURI GetClassID `0x7a41ad30` 和 Load `0x7a41ad70` 对位类型库
的 `{FD27E0BE-6D83-40DF-A226-CFB614F7655D}`。其版本为 1–941，
之后读五条 AfCore String、一个 double（IAT `0x7a5f5238` 的
`AfStreamRead` double 重载）。v2 起读 IStringArray，v941 加读
IPropertySet，IID 常量分别在 `0x7a5ff048` 和 `0x7a5ff028`。
Save `0x7a41aea0` 使用版本表 `0x7a5f86b8`。CellSizeLevel 复用
基类参数体，FunctionRasterDatasetName 复用栅格名称对象体和递归限额。
合成 ItemURI v1、v940、v941 覆盖各分支及跨内联对象的属性集回引。

本次补齐该 RTTI 交集中的全部 65 个具体类。范围限于 MosaicDB.dll
中含 IRasterFunctionArguments 和 IPersistStream 相应 RTTI 的类。
RenderingRule 的其他模块类及实际宿主接受的参数集合仍待核对。
参考工程与两套安装介质共 135 份工程的 /Maps 和 /PageLayout 均读至末尾，
这些流未出现本次十个类的 CLSID。ArcMap/QGIS 人工验收尚未进行。


### 局部运算公开类和登记表复核

2026-11-20 复核 Desktop 10.1.0.3035 PE32 的 RasterLocalOp.dll
（SHA-256 `bbadb2a4e926b456f34444ee2300b3f3ae3d39bf1897aa45ddeddf1ffa6bcfbd`）
及 SARasterFunction.dll（SHA-256
`5b8938501daae70bbd3185a726c233813e6eb4456b72032d5b080b0b4e27637d`）。
公开 Local、Math 和 Conditional 的 CLSID 位于 RasterLocalOp.dll
`0x7c7cc5ec..0x7c7cc64b`，以 10.1 安装登记表及
esriSpatialAnalyst 类型库的 IRasterFunction、IRasterFunctionArguments 配对核实。

| 公开类 | CLSID |
| --- | --- |
| LocalFunction | `3DEDF3D2-61FC-4F9D-9EAE-5A3F5358308B` |
| MathFunction | `2E60049F-FE7F-4BCD-9B34-B6AC978B1E10` |
| ConditionalFunction | `C75DB7FB-1BED-46E3-8F96-17D24D8A03CE` |
| LocalFunctionArguments | `19DA76F9-E143-41B3-955C-38033A7189A8` |
| MathFunctionArguments | `DE07B947-BD0B-4851-AA70-3826B4983240` |
| ConditionalFunctionArguments | `35591E68-2DC3-4630-8BBA-48EAEA56F834` |

函数构造链均到达 `0x7c7c2880`，参数构造链到达 `0x7c7c2930`。
两处分别从 `0x7c7ccd50`、`0x7c7ccd60` 取 LocalFunctionImpl
与 LocalFunctionArgumentsImpl 的 CLSID，经 `0x7c7c3710` 的
AfCreateInstance 把聚合对象存于 `+0x14`。公开函数的 QI（例如
`0x7c7c2eb0`）和公开参数的 QI（例如 `0x7c7c3180`）把
IPersistStream 查询转给这个成员。对应底层持久化入口仍为
SARasterFunction 的 `0x7cd698a0 / 0x7cd69930` 与
`0x7cd6b0a0 / 0x7cd6b0f0`。解析器在对象体分派时复用这两个布局，
保留对象头原始 CLSID。SystemCore 对象流写出器
`0x7ffcf7e0` 经 `0x7ffd0be0` 请求 IPersistStream
（IID `0x8007127c`）并取其 GetClassID，因此静态路径预期保存时
使用底层 Impl CLSID。公开 CLSID 的读取按构造和 QI 路径验证。

登记清单重新从 10.1 Desktop CAB 提取 `esri.clsid.ecfg`
（SHA-256 `170ae1f812bafc317ac98455eb7be844dd37f8d3a158c454cedf12f5046c3486`）。
此前对照的 10.8 登记表含
RasterFunctionArguments、BufferedRasterFunction、MultidimensionalRasterFunction、
ElevationVoidFillFunction、VectorFieldRendererFunction 及其中可见的参数类。
这些名称未出现在本次 10.1 登记表中。两种接口的类型库、RTTI
与二进制筛查均以 10.1 Desktop 介质为准。MemoryPixelsIntersectFunction
已定位到返回 E_FAIL 的原生 Load/Save 桩，解析器保留该类停止点。
同模块登记的 ScalarFunction（`D3205BCF-767F-4E5E-AC12-C1CFC0286552`）
经 `0x7d0bd1e0` 和 `0x7d0bd830` 查询接口，两种 RenderingRule
槽 IID 均落入 `0x7d0bd922` 的 E_NOINTERFACE 分支。

合成 RenderingRule 覆盖三个公开函数和三个公开参数对象、局部运算值、
嵌套属性集、对象头 CLSID 与后继边界。参考工程和两套介质共 135 份工程的
/Maps 和 /PageLayout 均可完整读取，270 条流未出现六个公开 CLSID。
实际宿主字节、历史版本别名及第三方实现仍需独立证据。
ArcMap/QGIS 人工验收尚未进行。

### 规则与函数的有符号版本门

沿用上文经版本资源与 SHA-256 复核的 10.1.0.3035 x86 模块。对
esriDataSourcesRaster 及介质类型库导出中直接实现 IRasterFunction 或
IRasterFunctionArguments 的可见 coclass 逐项比对 CLSID，均已有分派。
继续扫描 1,115 个 Desktop DLL 的两个 IID 常量。命中也包括调用方，仅凭 IID
字节无法排除经继承、委派或动态注册提供的实现。

用 objdump 对照 short 比较和跳转，复核以下入口。地址为对应模块的映像 VA。

| 模块 | 已核对的有符号比较 |
| --- | --- |
| ImageServer.dll | RenderingRule `0x77f8656f` 比较上限 1；MosaicRule `0x77f83424` 比较上限 5，`0x77f834f6`、`0x77f8351b` 分别判断可选槽的下限 2、4 |
| RasterCoreLib.dll | BasicFunction `0x7beb27c8`、`0x7beb27da` 比较上限 4 和旧体分支 2；BasicFunctionArguments `0x7beb648b` 比较上限 1 |
| FunctionRasterDB.dll | 共用函数入口 `0x6fd1edce` 比较上限 1；独立函数的比较见 `0x6fc2334c`、`0x6fc5c08d`、`0x6fca0e9e`、`0x6fd07e6b`、`0x6fcf990d`、`0x6fcc2422`、`0x6fd2262c`、`0x6fcf0944` |
| FunctionRasterDB.dll | MosaicFunction `0x6fcc9858` 比较上限 9，`0x6fcc9ad7` 起按有符号版本追加字段；模板 `0x6fcf2fe8` 比较上限 3，`0x6fcf3128`、`0x6fcf3172` 控制尾部；模板变量的两次比较为 `0x6fcf4e5f`、`0x6fcf4e89` |
| SARasterFunction.dll | 局部运算函数 `0x7cd698db`、参数尾部 `0x7cd6b0dc`、MLClassifyFunction `0x7d0b661e` 均比较上限 1 |
| ImageServerLayer.dll、RasterRenderer.dll、3DToRasterFunctions.dll | 栅格项 `0x78021ddd`、渲染栅格函数 `0x7c988f7e` 和三维转栅格入口 `0x681cdd5e` 均比较上限 1 |
| SystemCore.dll | TimeInstant 在 `0x7ffea9d1` 比较上限 1，`0x7ffea9d6` 以有符号 jg 进入失败路径 |

解析器按 i16 判定这些分支，将原始 u16 保留在输出中。合成用例验证
负版本的规则、基类、模板及变量、镶嵌函数与局部运算参数，以及 v4
MosaicRule 中负版本 TimeInstant。正版本超限仍被拒绝。重新读取 134 份
10.1/10.8 介质工程与仓库参考工程，两种规则的 CLSID 在 135 份工程的
`/Maps` 和 `/PageLayout` 中均无字节命中，两流全部读至结束。
负版本位模式的 ArcMap 实际保存来源和实档仍待取证，未在 ArcMap/QGIS
中人工验收。

### Scalar 与 ArgumentArray

核对 Desktop 10.1.0.3035 PE32 FunctionRasterDB.dll，SHA-256
`40af35694799c27fe2ace8bf28b12ca84a8f348122267180452291e0dd1c2701`。
原生工厂表从 `0x6fd76658` 开始、步长为 `0x34`，在 `0x6fd7675c` 和
`0x6fd76790` 分别登记 Scalar、ArgumentArray，与同版介质注册表互证。

| 类 | CLSID | IPersistStream 虚表 | GetClassID / Load / Save |
| --- | --- | --- | --- |
| Scalar | `A5B2BF59-7876-4BAC-AB34-7C2F3684B95F` | `0x6fd4d860` | `0x6fd0cd60 / 0x6fd0cda0 / 0x6fd0ce00` |
| ArgumentArray | `926DCB90-DA33-45A2-8A24-907B66367AA9` | `0x6fd3f6d4` | 调整跳板到 `0x6fc21540 / 0x6fc21580 / 0x6fc21770` |

- Scalar::QI `0x6fd0cc40` 把持久化接口映射到对象 `+0x14`。
  Load 读有符号 short（上限 1），再通过 `0x6fd0cde5` 调
  `Variant::Read(false)`。写侧 `0x6fd0ce61` 同样使用
  `Variant::Write(false)`，版本表 `0x6fd3f304` 的有效项为 1。
- ArgumentArray::QI `0x6fc21270` 经虚基表返回 IPersistStream。
  Load 读同样的 short 版本、两条 AfCore String、signed LONG 计数，
  正计数时 `0x6fc216c6` 按项从原流 LoadObject，请求
  `0x6fd3f0ac` 的 IUnknown。空结果也作为参数传给集合方法，调用点忽略
  该方法返回值，随后判空再 Release。Rust 保留空对象项及原始计数。
  Save 版本表 `0x6fd3f5c8` 的有效项为 1，写侧在 `0x6fc2199b`
  调 SaveObject 写各项。
- 两类实现的本地接口分别由 IID 常量 `0x6fd4d8c4` 和
  `0x6fd3f760` 对位，在这里按可持久化的嵌套值解析。Rust 从
  RasterFunctionTemplate 的 IUnknown 输入和参数 PropertySet 的有键
  Variant 接入两类。ArgumentArray 的普通对象项与模板、参数属性集共享键。
  未知新项停在其所属对象的错误边界，递归项沿用 Variant 深度限额。

构造 RenderingRule 测试覆盖模板中新数组及 Scalar、同数组回引、空对象项，
和外层参数属性集中的新数组回引模板已登记的 Scalar。还核对不同 Variant
保留区、后继字段及属性集的截断和超计数回滚。更多 IUnknown 项、其他工厂类
和真实嵌套宿主仍待取证。ArcMap/QGIS 人工验收尚未进行。
135 份对照工程的 /Maps 和 /PageLayout 均读至末尾，270 条流中未检出
两类 CLSID。

### RasterInfos 与模板中的 RasterInfo

对照同一 FunctionRasterDB.dll 10.1.0.3035。RasterInfos
（`F01B2CC6-A685-456D-B23F-909596DFD9A2`）在原生工厂表
`0x6fd766f4` 登记，主 QI `0x6fcafe20` 将 IPersistStream 映到
对象 `+0x10` 的虚表 `0x6fd47570`。GetClassID `0x6fcafa00`
返回 `0x6fd48364` 的 CLSID，Load/Save 分别为
`0x6fcafa60 / 0x6fcafb30`。

Load 先读有符号 short（上限 1），再读 signed int32 成员数，
非正数不进入循环。正计数时，主虚表 `0x6fd4751c` 的
`+0x2c` 逐项委派 `0x6fcaf5b0`。该助手在 `0x6fcaf60b`
以原流调用 LoadObject，请求 IUnknown，随后在 `0x6fcaf62b`
对返回对象查询 `0x6fd422d0` 的 IRasterInfo。Save 固定写版本 1，
按计数通过 `0x6fcaf510` 和 `0x6fcaf577` 逐项调用 SaveObject。
72 份同版介质类型库只列出 RasterInfo 实现 IRasterInfo。
这些成员有普通对象头及会话键。读侧成功路径会解引用 LoadObject 的
输出指针，Rust 对集合成员的空槽报错。

模板的 IUnknown 输入和模板变量的对象值接入两类。集合中的成员、
独立 RasterInfo 及内部几何沿用同一引用表。RasterInfo 的版本门
`0x6fcf6e14` 和后续分支均按有符号 short 比较，解析器保留原值
的位模式。通过 RasterInfo 关联的栅格地理变换可能再进入函数栅格名称，
因此复用调用方剩余嵌套深度，并将其从 MosaicFunction、MergeRastersFunction
及单对象函数入口传入。压缩子流仍使用其独立键表。

合成 RenderingRule 覆盖集合内回引、独立 RasterInfo 对集合内点的回引、
模板变量对集合的回引、有符号版本、后继字段、空成员、非法计数和截断。
含忽略用例的全量测试共 1098 项通过。135 份对照工程均读至 /Maps
和 /PageLayout 末尾，两条流中未命中 RasterInfos、RasterInfo 和
DblPnt 的 CLSID。

#### 参数属性集与 ArgumentArray

在 10.1 AfCore.dll 的 `0x68a7ca4d` 复核 VT_DISPATCH 和 VT_UNKNOWN
均对原流调用 LoadObject，请求 IUnknown。ArgumentArray 的逐项读法
也请求 IUnknown（`0x6fc216c6`）。两处共享 ObjectStream 的引用表，
所以按 CLSID 复用上节的 RasterInfo 与 RasterInfos 读取器。
对象体保留类型化字段和实际消费的字节，用原始编码比较浮点位模式。

RasterInfo 可以经 Variant 或地理变换再次到达栅格名称与函数规则。
读取器把活动的栅格信息及 Variant 深度随会话状态传递，保持原有对象
深度限额。MergeRastersFunction 的压缩子流只继承这两个深度值，
子流的对象键独立登记。此处的限额和失败回滚是 Rust 读取器的边界检查。

合成属性集覆盖 ArgumentArray 中同时出现新集合、新 RasterInfo 及回引，
同一属性集的后继值复用数组内键，任意截断均回滚。递归测试核对 RasterInfo
再次进入 Variant 时继承外层深度，并允许限额处读取已有键。合并函数测试
验证压缩流可复用外层键号及深度限额继续生效。真实非空属性集和模板宿主仍待
实档或运行取证，ArcMap/QGIS 人工验收尚未进行。

### RenderingRule 登记候选复核

对照 10.1 Desktop 的解包登记文件 `esri.clsid.xml`（SHA-256
`13a72c90856b258dbd065b4cbc9a2eedf744b411edfa9ccb56b64ce5074eedce`），
在 FunctionRasterDB、RasterLocalOp、SARasterFunction、ImageServerLayer、
RasterRenderer、RasterCoreLib、MosaicDB、_3DToRasterFunctions 的登记模块下，
提取 progID 含 Function 或 Argument 的候选 CLSID，与 Rust 的内存序
CLSID 和构造常量比对。此条件命中 109 条登记。补以 72 份类型库中
直接或继承声明 IRasterFunction 的 coclass GUID，避免仅按名称筛选。
后一个集合中的具体类均能对应代码中的已知类标识。

登记名称筛查留下四个未声明解析常量的 CLSID：

- SARasterFunction 的 MemoryPixelsIntersectFunction 和 ScalarFunction
  分别已证持久化失败及宿主接口查询失败，见[登记表复核](#局部运算公开类和登记表复核)。
- FunctionRasterDB.dll 10.1.0.3035（摘要同本页前述原件）的
  RasterFunctionHelper（`B55F1F83-3745-4C88-BBE1-4FC7A0273409`）
  主 QI `0x6fcf0cb0` 比较 `0x6fd4ba78` 的 IID，再委派
  `0x6fd29d40`。这两个入口的比较常量没有 RenderingRule 的两个
  函数接口 IID，未命中时后者在 `0x6fd29e32` 返回 E_NOINTERFACE。
- 同版 FunctionRasterDataset（`D0CC7770-AA1A-4FE3-BD82-1D8BB7B3781F`）
  主 QI `0x6fc898e0` 的 42 个比较常量经 objdump 和 PE 字节复核。
  后备 `0x6fd2a280` 只增加一个比较并转入 `0x6fd29d40`。两条
  路径均不响应 RenderingRule 的函数或参数 IID。其名称对象
  FunctionRasterDatasetName 有独立 CLSID 和解析入口。

这轮清点排除了上述登记候选。其他模块的间接实现、旧版别名和第三方
类仍需要独立原生或真实宿主证据；未做 ArcMap/QGIS 人工验收。

### IRasterFunctionArguments 类型库覆盖

对同一批 10.1 Desktop 介质提取的 72 份 OLB，按 MSFT 类型库的接口
引用及继承关系，枚举直接或间接声明 `IRasterFunctionArguments`
（`04AA081E-28B0-47E4-B375-F367046DA924`）的 coclass。共 46 个：
esriDataSourcesRaster 39 个、esriGeoAnalyst 1 个、
esriGeoDatabaseExtensions 3 个、esriSpatialAnalyst 3 个。
逐个用 CLSID 调用解析器的 `supports_function_arguments`，复核这 46 个
都已命中分派。读取这些类体的原生入口和字段证据见本页各节。

这次清点只涉及类型库声明及现有代码入口。其他模块中的未公开实现、
第三方扩展和真实规则中的嵌套属性仍需分别核查。ArcMap 10.1 和 QGIS
人工验证尚未进行。

### 栅格函数变量的共用 Variant 对象

同版 FunctionRasterDB.dll 的 RasterFunctionVariable v2 Load
`0x6fcf4e30` 读取带保留区的 AfCore Variant，Save
`0x6fcf4f20` 写回同类编码。对象值沿用 AfCore 标准对象流，
见 [带保留区的对象值](arcmap-10.1-variant-propertyset-slots.md#带保留区的-objectstream-对象值)。
栅格名称和参数等专用分派继续优先。遇到专用分派不支持的对象值时，
解析器从原位置用共用 Variant 读取器重新解码。两次试读的登记表各自
隔离，成功的对象仍登记到同一宿主会话。

合成输入验证变量中的 NumericFormat、同键回引和对象体截断回滚。
实档变量及 ArcMap/QGIS 目视验证待补。

模板与函数栅格名称的 IUnknown 输入槽也可读取独立 SystemCore
PropertySet，其持久化体复用标准 AfCore ObjectStream 入口。模板 Load
`0x6fcf2fb0` 请求 IUnknown，核心对象头及引用表规则与同模板中已有
参数类一致。合成 RenderingRule 检查模板输入中的 PropertySet 首次
定义及同键回引，并继续消费外层规则的参数与渲染器槽。此处验证读取
边界，实档写出及 ArcMap/QGIS 目视验证待补。

### ArgumentArray 的栅格对象成员

ArgumentArray 的成员读取在 FunctionRasterDB.dll 10.1.0.3035
`0x6fc216c6` 请求 IUnknown，写侧 `0x6fc2199b` 按成员写普通对象头。
共用 Variant 分派之外，数组成员现在可走模板的栅格对象分派，并保留
原始对象体。两条试读路径隔离游标和登记表。转入栅格分派时继承
外层 Variant 深度限制，回到 Variant 时继续累计深度。
合成 RenderingRule 验证数组内 CachedRasterFunctionArguments 新对象
及同键回引，并核对后续外层属性集和渲染器槽边界。实档与目视验证待补。

### IUnknown 槽中的栅格函数对象

模板和函数栅格名称的 IUnknown 输入及 ArgumentArray 成员可以按对象头
CLSID 读取已有持久化证据的 IRasterFunction 对象。它们共享 RenderingRule
的函数版本分派，继续使用各自宿主剩余的递归预算和引用表。
合成 RenderingRule 在 ArgumentArray 中读取 IdentityFunction，并验证
后续输入、参数及渲染器仍按规则边界读取。实际工程中函数作为
IUnknown 输入的写出和 ArcMap/QGIS 目视验证待补。

### IRasterFunction 类型库覆盖

枚举同批 Desktop 10.1 介质的 72 份 MSFT 类型库，沿接口继承树
查找 `IRasterFunction`（`CF7B7F99-3698-4F1F-AFF8-E3430E106B1E`）。
共得 56 个 coclass CLSID：esriDataSourcesRaster 49 个、
esriGeoAnalyst 1 个、esriGeoDatabaseExtensions 3 个、esriSpatialAnalyst
3 个。将 GUID 转为对象头的内存字节序，逐项直接调用
`function_max_version`，56 项均返回可读取的版本上限。该检查验证
RenderingRule 函数槽的类分派，类体的原生读写证据见本页各节。

另复核 Desktop 解包 DLL 中接口常量的一处跨模块引用。
GpRasterFunctions.dll 10.1.0.3035 仅有一份该 IID 常量
`0x748a8ed4`，其指令引用 `0x7487c900` 将 IID 与输出槽交给现有
对象的虚表第 0 项，之后调用其 Release。这里是接口消费方。
其他模块尚未逐一核对，未知扩展与旧档类体仍待证据。ArcMap 10.1
和 QGIS 人工验证尚未进行。

### RenderingRule 接口的跨模块指令复查

对现有 10.1 Desktop 解包目录中的 1115 个 DLL 文件搜索两种
接口 IID 的内存序，34 个文件命中。将文件偏移换算为映像地址，
再用 `objdump -d -Mintel` 查直接引用。用 `mov` 装入 IID
地址的模块是 ImageServerLayer、RasterRenderer、FunctionRasterDB、
_3DToRasterFunctions、RasterLocalOp 和 SARasterFunction，均在
已有对象体研究范围内。使用 `push` 的引用也包括宿主读取及
调用者查询，单凭引用数无法判定有哪些类实现该接口。
AISClient、AISGlobalLib 和 AISServerManager 命中原始字节，
本次未找到对应的原生指令直接引用。

对其中两个模块的四个 `mov` 引用，按 RTTI 主虚表定位 QI，
再核对匹配分支返回对象偏移。两模块均为 PE32，FileVersion
10.1.0.3035。ImageServerLayer 的 SHA-256 见本页函数集合段，
RasterRenderer 的 SHA-256 为
`cc56825cdf91d4b5aed88d0f766ee253ad30000c3079c40a21a2e9e6c6ce1390`。

| 实现 | 主 QI / IID 比较 | 成功返回 |
| --- | --- | --- |
| RasterItemFunction | `0x78020f30 / 0x78020f37` | `0x78020f55` 返回 +0x10 |
| RasterItemFunctionArguments | `0x78022860 / 0x78022867` | `0x78022885` 返回 +0x10 |
| RenderedRasterFunction | `0x7c988970 / 0x7c988977` | `0x7c988995` 返回 +0x10 |
| RenderedRasterFunctionArguments | `0x7c9891f0 / 0x7c9891f7` | `0x7c989215` 返回 +0x10 |

四个类均已接入函数或参数分派。作为对照，ImageServer.dll
10.1.0.3035 的 `0x77f865a2 / 0x77f865c0` 把 IID 交给
RenderingRule 的 LoadObject，属于宿主消费。直接地址引用扫描
不能覆盖运行期构造 IID、间接表或第三方模块。剩余类待办保留。
本轮为静态取证，未在 ArcMap 10.1 或 QGIS 中人工验证。
