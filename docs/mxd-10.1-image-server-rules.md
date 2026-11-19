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
  最后读两个压缩 Variant。解压后的每段数据各含一个 IUnknown 槽。
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
  未支持的嵌套类型时，保留其头及解压边界内的剩余字节。参数的非空
  IUnknown 保留对象头后的原始体。

合成 RenderingRule 测试覆盖 v2 三段压缩数据、独立键、矩形、未知对象原值、
全部截断、失效校验和与长度异常。两种 MergeRasters CLSID 在参考工程
/Maps 及 199 个安装介质流中均无命中。ArcMap/QGIS 人工验收待补。

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

MosaicFunction v3/v4 与 RasterInfo v6 请求同一个 IPnt IID。
RasterDB 的 Pnt 类（`064087E4-09B9-4892-A893-FCA89BB171D1`）
QI `0x7c0337c0` 对应该 IID，但该 QI 可见路径未定位到其
普通 IPersistStream 装载入口。IPnt 的非空新对象仍待原生证据，
空和回引可解析。参考工程及 199 个介质流未发现这四种类的标识。
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
可用对象体。登记表中的 ElevationVoidFill 两类和 VectorFieldRendererFunction
两类在对应的这份 PE 中未找到各自 CLSID 常量，持久化入口仍待定位。

参考工程 /Maps 和 199 个安装介质流没有前述十二个新增 CLSID
的命中。合成 RenderingRule 覆盖 LAS 与局部运算代表对象、非空属性集、
外层后继字段和超版本拒绝。未做 ArcMap/QGIS 人工验收。
