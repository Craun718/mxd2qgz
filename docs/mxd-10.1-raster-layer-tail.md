# ArcMap 10.1 RasterLayer：渲染器后的图层尾段

依据为 x86 Desktop 10.1.0.3035 的 RasterLayer.dll（ImageBase
`0x7c710000`，SHA-256 `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`）。
复用仓外导出的 Load `0x7c73bd40`、Save `0x7c73d0a0`、扩展列表 Load
`0x7c73ded0`，并用 objdump 核对 Save 版本表 `0x7c7678a8`：
`[7,11,13,13,16,17]`。表不是加载器接受版本的全集。接口身份按同一介质里
esriGeoDatabase、esriDataSourcesRaster 类型库的 GUID、TypeInfo 与名称表
对位。RasterIO 记录头和结束对沿用 [RasterCoreLib 取证](mxd-luna-deep-fgdb-raster-20261118.md#rastercorelib-导出与-rasterio-记录)。

## 字段顺序

两条参考工程 v17 图层在渲染器体后均剩 298 字节，尾段内容逐字节相同。
下表偏移相对于渲染器之后第一个字节。色表图层此处为 `/Maps:0x14479`，
RGB 图层为 `/Maps:0x16e8c`。

| 相对偏移（参考工程） | Load / Save 序列 | 当前读取范围 |
| --- | --- | --- |
| `0x00` | double×2、int32、short、int32（最后一项版本 ≥2） | 原样保留 |
| `0x1a` | 扩展列表，版本 ≥3；≥8 调 `0x7c73ded0`，int32 计数 + ExtraSafeReadObject 循环 | 两代均分派已确认的扩展，保留扩展边界和原始体 |
| `0x1e` | double（≥4）、内联 WKSEnvelope（≥5） | 四个 8 字节坐标原样读入；参考档首项为 NaN，不映射为有效范围 |
| `0x46` | ≥6：int32 选择子；非零再读一个 IName。之后 int32 数量 + IRelationshipClassName 循环（IID `311C2EE2-…`） | 复用 RelationshipClassName 和 MemoryRelationshipClassName；未闭合体停止 |
| `0x4e` | ≥7：IEnvelope（IID `1B077914-…`） | 空、回引或已支持的 Envelope 类 |
| `0x5e` | RasterIO：magic `0x00ff00ff`；可选 tag 30 对象与 tag 31 宽字符串；tag 35/36/37 三个 short；`(-1,-1)` | tag 30 支持空对象及已接入的栅格名称类 |
| `0x8c` | AfCore String（≥10）、v10 专用 short、double×2（≥11） | v8/v9 无此段，v10 读取字符串和 short，v11 起读取字符串和两个 double |
| `0xa2` | 一对对象：IGeodataXform（IID `5E6CF525-…`）、IEnvelope；v12 依赖原生数据集状态，≥13 总是读取 | v12 根据父条目边界识别两槽是否存在。地理数据变换支持空、回引和[已确认类](mxd-10.1-geodata-xform.md)；Envelope 复用已有读取器 |
| `0xc2` | ≥14：IColor、int32 数量、压缩索引列表 | 支持现有颜色类和按声明数量展开索引 |
| `0xd6` | ≥15：TimeData → TimeDataDisplay → TimeDimensionDefinition | 复用 LayerLib 同名段读取器；没有 FeatureLayer 的 TimeTableDefinition，且显示段在维度段之前 |
| `0x116` | ≥16：IPixelFilter（IID `90E3DE9D-…`） | 空、回引、PansharpeningFilter、LutFilter 和 HistogramSpecification |
| `0x126` | ≥17：int32 数量 + IStatsHistogram（IID `2B9062E8-…`） | 空、回引和 StatsHistogram；与 RGB 的无键 RasterIO 对象共用体读取器，此处使用 ObjectStream 键 |
| `0x12a` | 对象体结束 | 两个条目都与外部 ExtraSafe 边界一致（`0x145a3` / `0x16fb6`） |

## 两个包络槽的加载用途

对上述 10.1.0.3035 DLL 用 objdump 复核。Load 在 `0x7c73c3c1` 将
持久化接口相对偏移 `+0xf0` 的成员作为 v7 起 IEnvelope 槽的读取目标。
后一个 IEnvelope 在 `0x7c73c73c` 读入局部变量。Load 随后检查对象
是否存在、能否从成员 `+0xb0` 取得接口以及包络是否为空，再进入
`0x7c73c79e–0x7c73c8ef` 的处理。这里根据接口查询结果走不同分支，
其中一支通过 IAT `0x7c7675d0` 调用导入的
`CopySensorXformProperties(IGeodataXform *, IUnknown *)`，将查询得到的变换接口
和局部包络作为实参。另一支从包络取出接口后调用成员上的方法。
局部包络在 `0x7c73c90c` 释放。

构造函数 `0x7c7367c0` 将完整对象 `+0x88` 设为 IGeoDataset 子对象，
虚表位于 `0x7c76c114`。按 10.1 类型库槽位，空间参考与范围 getter 分别
为 `0x7c73a0d0` 和 `0x7c73a170`。两者都检查 IGeoDataset 相对
`+0xc4`，即完整对象 `+0x14c`，与 Load 的数据集成员 `+0xb0` 对位。
范围 getter 在该成员为空时于 `0x7c73a29c` 返回空指针和 `S_FALSE`，
有数据集时于 `0x7c73a1d2` 调用数据集范围槽，并在后续处理里读取
IGeoDataset 相对 `+0xe0` 的空间参考成员。ILayer2 虚表
`0x7c76c22c` 的 AOI 槽 `+0x18` 经 `0x7c74d9b0 → 0x7c737550`
转调同一 IGeoDataset 范围 getter。

v7 持久化包络位于完整对象 `+0x18c`。这里确认的范围 getter 在数据集
缺失时返回空值，v13 包络则参与加载期的变换属性恢复。解析器继续保留
两个包络的原始读取结果。数据集打开后的地理变换效果仍需在 ArcMap 10.1
及 QGIS 中人工核对。

## 对象分派与读取边界

IGeodataXform 也是 ImageServerLayer v20 的请求接口；RGB 的 RasterIO tag 18
另经 `0x7c99da10` 调 QueryInterface（IID `A9D01E68-…`，
esriDataSourcesRaster 中的 IPansharpeningFilter）。已接入的 IGeodataXform 类见[对象体记录](mxd-10.1-geodata-xform.md)；
RGB 过滤器的已确认范围见[全色锐化](mxd-10.1-raster-renderer.md#rgb-tag-18全色锐化)。
普通 RasterLayer 的 tag 30 是另一条路径：Load `0x7c73bd40` 取对象后经
`0x7c731840` 得名称接口，读 tag 31，并经 ConvertToFullPath 和 IName::Open
打开数据；不能按 RGB tag 18 的过滤器对象体分派。

实现 [raster_layer_tail.rs](../src/layers/raster_layer_tail.rs) 仅在父条目已有长度
且渲染器完整消费时启动；尾段顺序读完且与条目边界一致才提交游标和引用键。
扩展对象体的解析、RasterIO tag 30 的其他名称、其他变换和像素过滤器类
仍待补充。合成 v17 尾段含非空 IEnvelope 与
IStatsHistogram；逐字节截断及越过条目边界时均验证回退，不保留新键。
参考工程仅用于字节和边界对照，未做 ArcMap/QGIS 软件验收。

## 名称列表与渲染器边界

沿用上述 RasterLayer.dll 10.1.0.3035 映像，核对已导出的 `0x7c73b380`
和原始指令。版本大于 8 时从流读取 int32 数量（`0x7c73b3b5` 附近），
较旧版本使用隐含数量 1。每项调用 AfCore LoadObject 读取 IName，版本大于
0 时从 `0x7c73b478..0x7c73b493` 再读一条 AfCore String，空对象与回引
仍经过此步骤。版本 5–8 在 `0x7c73b4ab..0x7c73b4f5` 接着读取 int32
选择子，非零再读取 int32。两项原值保留。

循环后的 `0x7c73b61f..0x7c73b63e` 检查对象中已加载名称的数量，为零返回
`0x8000ffff`。主 Load 在 `0x7c73bea5` 检查结果，成功才在
`0x7c73bee4` 进入 IRasterRenderer。静态解析支持普通栅格、FileGDB、Access、
SDE、NetCDF、RasterDatasetName83、RasterBandName、RasterBandCollectionName、
RasterCatalogName、MosaicDatasetName、影像服务及函数栅格名称的新对象及回引，
也保留已消费的空槽。遇到未知名称或截断时，游标停在该
条目起点，保留前面完整条目及其引用键。零、负数量与全空列表停在渲染器之前。
名称能否在 ArcMap 打开仍取决于运行时数据源。

参考工程两个 RasterLayer 均为 v17 单项。多项混排、旧版选择子、空列表和
未知条目回退用合成输入验证，未在 ArcMap 实测。

### 瓦片图层与 AIS 影像服务名称

RasterLayer 10.1.0.3035 在 `0x7c73b380` 的名称循环请求的
IID 数据位于 `0x7c76c8a4`，原始字节与 IName 一致。以下两类经同版
组件的 PE RTTI、QI、GetClassID、Load 和 Save 指令互证。

- **InternetTiledLayerName**（CLSID `A723A563-0E3C-4484-BBD1-68B81929B5FF`）
  来自 MapServerLayer.dll（SHA-256
  `59957732405d0ada878e85d002cc465f76fdcef97faced0f8a6d03`）。
  QI `0x79e36b30` 把 IName 和 IPersistStream 分别映到完整对象
  `+0x14`、`+0x18`。IPersistStream 虚表 `0x79ebec38` 中
  GetClassID `0x79e36e90` 写出 `0x79ebdde4` 的 CLSID。Load/Save
  `0x79e36ed0 / 0x79e36f60` 依次读写 short 版本、AfCore
  String（IName.NameString）、int32 和 ITileReader。Load 的有符号
  上限为 1，写表 `0x79ebbd2c` 为 1。名称串写在持久化接口
  `+0x18`，IName getter `0x79e26c10` 对位到同一完整对象偏移
  `+0x30`。LoadObject 调用 `0x79e36f45` 请求
  `{6FE312B6-1796-4AA2-BC1E-B3722BA1DCC8}`，复用已有 ITileReader 分派。
- **AISImageServiceName**（CLSID `B842169E-3485-458E-9522-2D69DDEA8B6C`）
  来自 ImageServer.dll（SHA-256
  `b6a7bca69fe4b36e2ff0e973e07875b33030bc957fe765644c29ed276580cb80`）。
  QI `0x77f011f0` 把 IName 和 IPersistStream 分别映到完整对象
  `+0x14`、`+0x18`。IPersistStream 虚表 `0x77f9b1a0` 中
  GetClassID `0x77f014f0` 写出 `0x77f9b270` 的 CLSID。
  Load/Save `0x77f01530 / 0x77f015c0` 读写 int32 版本和三个
  AfCore String。Load 的有符号上限为 1。第一串由 IName getter
  `0x77f01340` 对位为 NameString，其余两串保留原样。

两类接入共享名称读取器。有界 RasterLayer 构造用例读取两类名称、
新 MapTileReader 与回引，随后读到 SimpleRasterRenderer。RasterIO tag 30
用例覆盖无键外壳、后续 tag 31 和 35–37、以及逐字节截断回滚。两套介质
的 134 个 `/Maps` 流和参考 MXD 原文件均未检出两个 CLSID 的候选字节。
名称从何种真实宿主写出、IName::Open 后的对象及数据源结果仍需在 ArcMap
取证，QGIS 人工检查亦未进行。

### RasterTypeName

从 MosaicDB.dll 10.1.0.3035（SHA-256
`424814c3eceb88871df221bb42cb420b61c79c110a1e46a0e05af744db0a39ed`）
补充反汇编。QueryInterface `0x7a45b890` 将 IName、IRasterTypeName
分别映到完整对象的 `+0x18` 和 `+0x20`。IPersistStream 通过虚基表
`0x7a607740` 映到 `+0x44`，构造函数在 `0x7a45b70a` 安装虚表
`0x7a60c4bc`。该表的 GetClassID、Load、Save 经调整跳板分别到
`0x7a45c550 / 0x7a45c590 / 0x7a45c670`。GetClassID 复制
`0x7a5feb78` 的 `507E980B-D338-4636-A729-A939BFCAA124`。

Load 在 `0x7a45c5be..0x7a45c5c5` 对读出的 short 减一并比较低 16 位，
只接受版本 1。随后按顺序读取两条 AfCore String、一个有符号 int32 和
一个对象槽。字段按完整对象偏移分别是 `+0x2c`（IName.NameString，
getter `0x7a426640`）、`+0x30`（Name，setter `0x7a45c3b0`）、
`+0x38`（InstanceID，setter `0x7a45c530`）、`+0x34`（MosaicDatasetName，
getter `0x7a416780`）。LoadObject 调用 `0x7a45c618` 请求 IID
`8911E302-D22B-42CB-B994-D3774B276460`，MosaicDatasetName 的
QueryInterface 在 `0x7a3f4ac9` 比较同一 IID，并在 `0x7a3f4d4f`
返回接口。SaveObject 调用 `0x7a45c6f4` 写出该子对象。

共享读取器复用 MosaicDatasetName 解析和当前引用表。有界 RasterLayer
构造用例覆盖非空子对象、外层回引及名称后缀；RasterIO tag 30 构造用例
覆盖无键外壳和空子对象。两套介质的 134 个 `/Maps` 流及参考 MXD
原文件未命中 RasterTypeName CLSID。实际宿主和数据源仍待 ArcMap
运行取证，QGIS 人工检查未进行。

### RasterFunctionTableName 候选

10.1 介质的 `esri.clsid.ecfg` 将
`22DAB0AE-B857-43B4-B25B-66E7A5699350` 声明为
FunctionRasterDB.dll 的 RasterFunctionTableName。核对该 DLL
10.1.0.3035（SHA-256
`40af35694799c27fe2ace8bf28b12ca84a8f348122267180452291e0dd1c2701`），
的导出 DllGetClassObject `0x6fcaf0f0` 跳转到 `0x6fd2b560`，
逐项遍历 `0x6fd76658` 起、步长 `0x34` 的工厂表。GUID 比较函数
`0x6fc21f70` 比较完整 16 字节，遍历遇空指针后返回
`CLASS_E_CLASSNOTAVAILABLE`。结合首项 PE 数据、初始化代码
`0x6fd38a71..0x6fd3b87c` 中的其余 100 个 CLSID 指针写入、
及 `0x6fd3b8f2` 的终止槽，重建了该表的 101 个 CLSID。
表中包含同一注册模块的 RasterFunctionVariable，但没有
RasterFunctionTableName 或 RasterFunctionTable 的 CLSID。
这一结论限于该映像的工厂表初始化及其导出路径。其他运行期工厂、IName
和持久化入口仍待取证，RasterLayer 名称列表及 tag 30 对此类继续保留未知。

## RasterDatasetName83

介质将 `{76360E01-EC46-11D1-8D21-0000F8780535}` 注册在 RasterDB.dll。
沿用 [RasterDatasetName 取证](mxd-luna-followup-rasterdatasetname-20261118.md)
中核验的 10.1.0.3035 映像。RTTI TypeDescriptor `0x7c1727b4` 对位
IPersistStream 虚表 `0x7c1338d4`，Load `0x7c0bca70`、Save
`0x7c0bcbc0` 用 pyghidra 补充。GetClassID `0x7c0bc9c0` 的直接分支可以
返回普通名称和 Name83 的两个 CLSID，另有委派分支。

Load 首先委派给普通名称体 `0x7c0ba5b0`，然后调用 RasterIO InitCheck
检查 `0x00ff00fa`。命中后，tag 1 读取 int32，tag 2 读取宽字符串，
最后调用 End。未命中时原生恢复流位置。有委派对象的 Save 走委派对象的
持久化虚表，其余情况写普通 RasterDatasetName 体。

Rust 在名称对象分派中接入这两种读取形态，复用普通名称的 WorkspaceName 键表。
测试覆盖 RasterLayer 随后的名称后缀、渲染器和尾段，以及 LoadRaster 中的
波段数组、变换和名称回引。参考工程和 66 个 10.1 介质 /Maps 流未检出
Name83 CLSID，当前只有静态证据和合成验证。

## 波段名称与集合

沿用上节 10.1.0.3035 RasterDB.dll，通过 RTTI、注册表和 GetClassID 对位
后补充 pyghidra 导出。以下读写地址按 PE 指令复核。

| 类 | CLSID | IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- |
| RasterBandName | `BC25E113-168B-11D2-8D25-0000F8780535` | `0x7c131088` | `0x7c0539e0` / `0x7c053b40` |
| RasterBandCollectionName | `710FC1B5-A118-445B-98D4-EF7D56E18A88` | `0x7c130f34` | `0x7c052da0` / `0x7c052e40` |

RasterBandName 的 Save 写 int32 版本 1，Load 在首 int32 不等于 1 时
回退四字节，再按旧式形态读第一条 String。随后共三条 String 和 short
选择子。零选择子加载 IWorkspaceName（IID 常量 `0x7c131138`），非零
选择子加载 IDatasetName（`0x7c130fc8`），该分支要求非空。两个类的 QI
均可匹配 IName 和 IDatasetName，因此普通和无键名称槽，以及 LoadRaster
的请求共用同一对象体分派。

RasterBandCollectionName 依次读 short 版本、String、IName 和 ILongArray。
在 RasterDB.dll 10.1.0.3035（SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`）
的 Load `0x7c052da0` 中，objdump 显示 `0x7c052dcb / 0x7c052dd0`
用 `cmpw $0 / jge` 排除负版本，随后各槽均直接读取。构造测试覆盖
0、2、32767 的相同体边界和 LongArray 回引，以及负版本拒绝。
这些版本的真实宿主及原生打开仍待验证。
Save 版本表 `0x7c123680` 为 `[-1,-1,-1,-1,1]`。末槽请求
`{54F9FFB6-E91F-11D2-9F81-00C04F8ECE27}`，由同版 esriSystem TypeInfo 92
定名 ILongArray。SystemCore.dll（10.1.0.3035，SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）
LongArray 的 CLSID 为 `{98BFB808-E91F-11D2-9F81-00C04F8ECE27}`。
已有导出中 IPersistStream 表 `0x8007c32c` 的 Load `0x80042aa0` /
Save `0x800422f0` 读写 short 版本、int32 数量，逐项委派
`0x800407c0` / `0x80040790` 消费 int32。

同一 SystemCore.dll（10.1.0.3035）的 objdump 指令显示，共用 Load
在 `0x80042ad7` 执行 `cmpw $1`，随后在 `0x80042adc` 以有符号
`jle` 进入数量读取。LongArray 与 StrArray 均按有符号 short 判断，
返回值保留原始 16 位位模式。构造测试覆盖最高位置位的版本、
后继字节及正版本 2 的拒绝。最高位置位的真实旧档及原生打开尚未验证。

名称递归沿用宿主引用表，最多读取 16 层新对象。回引无需再读对象体。
合成图层覆盖新旧名称形态、带键 WorkspaceName、LongArray、名称后缀及
图层尾段，并验证逐字节截断、异常数组计数和超深时回滚。参考工程及
66 份 10.1 介质 /Maps 流未检出两个名称类的 CLSID，未做 ArcMap 运行验证。

## RasterCatalogName

介质将 `{33FF62FC-D7D6-48E8-82D9-77C0AADBF5D1}` 注册在
RasterCatalog.dll。核对 x86 固定版本资源 10.1.0.3035、映像基址
`0x7be30000`、SHA-256
`0b6ac9b5e0c3e8efb715d68c5a55de89275def9ec7f666ffae91f0ff7972c585`。
RTTI TypeDescriptor `0x7be457a0` 对位 IPersistStream 虚表
`0x7be400bc`，GetClassID `0x7be35ee0` 复制注册 CLSID。
构造路径 `0x7be36002` 通过 AfCreateInstance 创建
FeatureClassName（`0x7be400dc` 为 `{198846D0-CA42-11D1-AA7C-00C04FA33A15}`）。

补充导出的 Load `0x7be36460` 与 Save `0x7be364a0` 对委派对象查询
IPersistStream 后直接转发当前 IStream。Load 的原始指令
`0x7be3647f..0x7be36489` 验证流指针与虚表槽，外壳和委派体之间无第二个
对象头。Rust 以外壳键复用 FeatureClassName 的读法，委派体未闭合时由宿主
回滚槽位。合成图层确认委派体后仍能读取名称后缀并到达图层末尾。
参考工程和 66 份 10.1 介质 /Maps 流未检出该 CLSID，未做 ArcMap 运行验证。

## MosaicDatasetName

MosaicDB.dll x86 固定版本资源为 10.1.0.3035，映像基址为
`0x7a3c0000`，SHA-256 为
`424814c3eceb88871df221bb42cb420b61c79c110a1e46a0e05af744db0a39ed`。
RTTI `0x7a66fc80` 对位 IPersistStream 表 `0x7a602728`。
GetClassID `0x7a3f47b0` 读取 `0x7a5fe6f8` 的
`{5CD5BB6D-6549-429E-80F4-0DD4731198D1}`。

Ghidra 补充导出的 Load `0x7a3f5b30` 读取 short 版本、两个 AfCore
String、IWorkspaceName，再依次读取 v2 起的 short、v3 起的 int32
和 v4 起的 IStringArray。IStringArray 的 IID 在 `0x7a5ff048`，
与介质 esriSystem.tlb 的 TypeInfo 94 一致。Save `0x7a3f5c80` 的
目标版本表位于 `0x7a5f8754`，可写版本为 2、4，v4 追加 int32
和字符串数组。Load 在 `0x7a3f5b64 / 0x7a3f5b69` 以 `cmpw / jle` 接受有符号
short ≤4，并在 `0x7a3f5bcb`、`0x7a3f5beb`、`0x7a3f5c25` 以
有符号 `jl` 跳过尚未引入的字段。Rust 保留原始版本位模式，正版本
超限时保留原槽位并报告停止。

SystemCore.dll 10.1.0.3035 的 StrArray
`{A7F92065-36CE-47B6-A463-4763DA947CC2}` 使用 IPersistStream 表
`0x8007c42c` 的 Load `0x80042aa0` 与 Save `0x800422f0`。数组体
包含 short 版本、int32 数量和按 `0x80040930` 读取的 AfCore String。
Rust 沿用宿主引用表并限制数组可读字节数。合成测试覆盖 v1–v4、
后续名称后缀和图层尾段、逐字节截断、异常数量及键回滚。
参考工程和 66 份 10.1 介质 /Maps 流未检出该名称 CLSID，
未做 ArcMap 运行验证。

## Access 与 SDE 栅格名称

安装介质的两个映像均为 x86 10.1.0.3035。使用 RTTI、GetClassID、
Ghidra 补充导出及 objdump 原始指令核对读写路径。

| DLL（SHA-256） | 类 CLSID | IPersistStream 表 | GetClassID | Load / Save |
| --- | --- | --- | --- | --- |
| RasterJET.dll（`b5722104bd55f6f1b72263151918c5294037c8fc36629a820f0e06c6e23ac31a`） | `C8F9DC31-D221-4635-923E-1A55A525B1A1` | `0x7c6fcec4` | `0x7c6f5730` | `0x7c6f6990` / `0x7c6f6b70` |
| SdeRasterDB.dll（`5e4e4afe64ed7669626b1ca05c622c25473747ed1f28c09b1efcc05ba1401157`） | `D6C003C5-A050-438E-BBD4-97294C3FDDC1` | `0x7f62fdd8` | `0x7f61d5f0` | `0x7f61e160` / `0x7f61e330` |

构造路径分别在 `0x7c6f5955` 与 `0x7f61d735` 压入普通
RasterDatasetName 的 CLSID 常量。两类从当前流直接调用内部名称的 Load，
随后通过 RasterIO InitCheck 探测 `0x00ff00fa`，缺省时恢复位置。
记录 1、4 为 int32，2、3、5 为 UTF-16LE 字符串，6 为无键对象。
两个读侧赋值函数 `0x7c6f7ae0` / `0x7f61f9f0` 均查询 IName。
Access 在 tag 6 后尝试读取 int32 tag 7，再消费结束记录；SDE 在 tag 6
后直接读取结束记录。Rust 对嵌套的已知栅格名称及 FeatureClassName
复用体读取器并保留深度限额，未知类或不完整记录使当前名称回滚。

合成 RasterLayer 验证无键 tag 6 与其 WorkspaceName 的宿主键登记。
合成 RGB 全色锐化体核对两类名称的 LoadRaster 入口、后续波段字段和回引。
参考工程及 66 份介质 /Maps 未检出这两个 CLSID，尚无 ArcMap 运行验证。

## NetCDF 栅格名称

NetCDFDB.dll 为 x86 10.1.0.3035，SHA-256
`e78ea582fdd66f45dfa717eb22346e9f691e1bb4e26587a1d09f72b0d1bc5ebd`。
RTTI `0x7ae79dcc` 对位 IPersistStream 表 `0x7ae615e4`，
GetClassID `0x7ae287a0` 复制 `{8CB4AA2B-58CE-438F-8CD8-F5F0BEF41AAF}`。
Ghidra 导出的 Load `0x7ae29e30` 读取 int32 版本和八条 AfCore String，
仅 v1 追加一条 String 与 IEnvelope。后继槽依次为 IStringArray、
ILongArray 与 IWorkspaceName，沿用宿主对象键。第二个数组的 IID
位于 `0x7ae6062c`，与 RasterBandCollectionName 请求的 ILongArray
一致。Save `0x7ae2a0f0` 固定写 v2，原始 call 指令
`0x7ae2a1b3` / `0x7ae2a1c2` 分别调用 `0x7ae204b0` /
`0x7ae205b0`，后者用 `0x7ae6065c` 的 LongArray CLSID 创建数组。
Rust 复用 [StrArray](#mosaicdatasetname) 和 [LongArray](#波段名称与集合)
读取器。合成 v1、v2 图层验证数组键、后继尾段、
逐字节截断与异常计数回滚。参考工程和 66 份介质 /Maps 均无该名称 CLSID，
尚无 ArcMap 运行验证。

## 影像服务栅格名称

ImageServerLayer.dll x86 10.1.0.3035 的 SHA-256 为
`0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`。
ISRasterDatasetName `{2D2A48DD-FFA1-4B6E-9691-BFE8EDBA8761}`
的 RTTI 为 `0x78048788`，IPersistStream 表为 `0x78031d30`。
GetClassID `0x780120a0` 从 `0x78030cd4` 复制注册 CLSID。

补充的 Ghidra Load `0x78012bb0` 和 Save `0x78012d00` 互证字段序：
short 版本、六条 AfCore String、int32、IWorkspaceName、IName。
v2 起追加一条 String，v3 起再追加一条 String，v4 起追加 IGeodataXform。
读侧三个请求 IID 分别在 `0x78031e64`、`0x78030630`、`0x78031e54`。
Save 版本表 `0x7802ca7c` 的有效项为 1、3、4。
objdump 显示 `0x78012bdf / 0x78012be4` 以 `cmpw / jle` 接受有符号
short ≤4，`0x78012caa`、`0x78012cbf`、`0x78012cd4` 的有符号
`jl` 控制后缀。两类名称均用构造数据验证负值、零、正版本 5
和后继图层字段边界。负值版本的真实旧档尚未在 ArcMap 中验证。
ImageServer.dll 中 ImageServerName 的 QI `0x77f4f4e0` 使用
`0x77f9b200` 处的 IName IID，解析复用
[已证的 ImageServerName 体](mxd-10.1-image-server-layer-tail.md#imageservername-对象体)。

合成 v1–v4 图层验证串与变换版本门、嵌套名称和 PropertySet 键、逐字节
截断回滚。栅格名称、IGeodataXform 与 RPC LoadRaster 间的递归沿用同一
深度预算，构造交替嵌套验证超深时整个名称及其键回滚。

## 函数栅格名称

FunctionRasterDB.dll x86 10.1.0.3035 的 SHA-256 为
`40af35694799c27fe2ace8bf28b12ca84a8f348122267180452291e0dd1c2701`。
RTTI `0x6fd78b28` 对位 IPersistStream 表 `0x6fd457cc`，
GetClassID `0x6fc9d580` 从 `0x6fd45140` 复制
`{EB07E8CC-EE78-438A-A2C1-BA80455ACB02}`。

Load `0x6fc9dcf0` 读取 short 版本、三条 AfCore String 和
IWorkspaceName。之后读路径 String。低于 v4 时在路径前多一条 String，
路径后多一个 int32 和 IName。接着读取 IRasterFunction、新旧输入槽，
v3 起接 IPropertySet，v3–v4 继续读 IGeometry，v6、v7 各追加一条
String。输入槽从 v2 起先读 int32 数量，正值按数量读 IUnknown；
更旧版本或非正值读取一个。文中请求接口按 `0x6fd45900`、
`0x6fd4422c`、`0x6fd3fc4c`、`0x6fd3f0ac`、`0x6fd44e70` 和
`0x6fd40994` 的 IID 对位。Save `0x6fc9e110` 的 `0x6fd3f4b0`
版本表提供 v7。

Load 在 `0x6fc9dd34` 用有符号 short 比较版本上限，后继门同样
使用有符号条件。构造 -1 与最小 int16 验证旧字符串、单个 IUnknown
输入和后继图层边界。读取器保留版本原始位模式，尚无负版本实档或人工验证。

IRasterFunction 槽沿用 SystemCore 同版 `0x7ffd0063` 的查键和
接口查询顺序。构造 v1、v4、v7 误引宿主名称的自身键时，在名称
对象起点停止并撤销登记。此回引尚无实档或人工验证。

函数及已知参数复用 [IdentityFunction、CachedRasterFunctionArguments 读取器](mxd-10.1-image-server-rules.md#非空栅格函数与参数)。
输入槽另按 CLSID 分派已知栅格名称，属性集与几何槽复用对应读取器。
合成 v1、v4、v7 覆盖旧版单输入、零计数回退、正计数和键回引，
并验证截断回滚。未知函数或输入类保留为该名称停止点。

函数栅格名称与 RasterFunctionTemplate 的输入均经同版 AfCore
LoadObject 请求 IUnknown。名称体现在复用模板的输入分派，从而也能
读取 RasterFunctionVariable、RasterInfo、RasterInfos、Scalar 和
ArgumentArray。保留原有名称及参数表示，新增类仍用宿主对象表。
合成 v7 在既有参数回引之后读取 Scalar 新对象及同键回引，随后读取
属性集、名称后缀和图层尾段。实档中的新增输入类仍待取证。

参考工程和 66 份 10.1 介质的 /Maps 均未检出这两个栅格名称类的 CLSID，
ArcMap 运行与 QGIS 人工验收尚未进行。

## v8–v11 尾段

`RasterLayer::Load 0x7c73bd40` 在 v8 起调用 `0x7c73ded0` 读取 ExtraSafe
扩展，随后沿用关系名称与包络序列。原生调用 RasterIO InitCheck 后仅在命中
magic 时读标签。未命中时 RasterCoreLib.dll `0x7bf13b30` 恢复游标，故
有界读取可处理缺省记录段。之后 v8/v9 到对象末尾，v10 读取 AfCore String
与一个 short，v11 读取 String 与两个 double。合成测试核对四个版本、
缺省记录段与截断。

## v0–v7 尾段

复用 RasterLayer.dll `Load 0x7c73bd40` 的导出，核对固定标量和 v2–v7
各字段门。v3–v7 的调用经 RasterLayer.dll 导入表对位到
LayerLib.dll `ExtensionHelper::LoadExtensions 0x786c22e0`。
LayerLib.dll 的摘要与[版本资源记录](mxd-terra-depth-legend.md)一致。
[FeatureLayer 同函数取证](arcmap-10.1-featurelayer-tail.md#扩展段格式修正第九轮记录撤销)
证实每项布局为 int64 体长、16 字节 CLSID、声明长度的对象体。扩展顶层
没有对象键。Rust 依据长度保留扩展体并继续读取后续字段。v0–v7 各版本门、非空旧式扩展、
异常长度与逐字节截断由合成输入覆盖。

旧式列表每条固定头为 24 字节。解析器将固定头和各对象体一并计入
调用方的列表读取限额，空体同样占用头部空间。构造用例验证恰好到限额、
头部超限及累计体长超限。未知扩展仍按其声明边界保留，内部键和
ArcMap/QGIS 实档行为尚待验证。

共用 LayerLib 读取器和 StandaloneTable 的逐条入口同样按每条
24 字节头加声明体长扣减限额。体长加头部使用检查加法，成功读完
条目才扣减。合成用例验证多个空体仍累计固定头，并覆盖逐条入口
头部超限及恰好到限额。

RasterLayer v6 的关系名称列表请求 IRelationshipClassName。依据同版
SystemCore 在 `0x7ffd0063` 先查键再查询接口的规则，列表按登记类
校验回引。其前面的可选 IName 槽仍使用各自的接口范围。构造 v6 验证
关系名称回引到达图层尾部，误引 Envelope 时回滚。尚无相同组合的
实档或 ArcMap/QGIS 人工验证。

RasterLayer.dll 10.1.0.3035 的 Load 在 `0x7c73c19a` 使用
`0x7c76c8a4` 的 IName IID (`4BA33331-D55F-11D1-8882-0000F877762D`)；
既有反编译表明读完后调用 IName::Open。后续关系列表在 `0x7c73c2d0`
请求 IRelationshipClassName。可选 IName 槽复用已确认的 TableName
对象体及工作空间槽。构造 v6 验证该名称读至关系列表，并验证列表误引
同一个 TableName 键时回滚。尚无相同组合的实档或人工验证。

同一可选槽接入 FileGDB.dll 10.1.0.3035 的 FgdbTableName。
Load `0x6fa33290` 读两个 short 后委派至 TableName，复用
[原生读写证据](mxd-terra-round9-old.md#fgdbtablename)中的读取器。
构造 v6 验证包装字段、列表边界及关系槽误引时的回滚。
尚无该组合的实档或 ArcMap/QGIS 人工验证。

可选槽复用独立表的 IName 分派和嵌套停止规则。新增构造
ObjectClassName，同一字节布局已由 [10.1 GdbCoreLib 的持久化委派](mxd-terra-round9-table.md#objectclassname-与栅格表名称)
确认。其他共用表名复用各自已核对的对象体。未知类或未闭合的体仍在宿主
事务内停止。尚无这些名称作为该槽实档的证据，未做 ArcMap/QGIS 人工验证。

## v8 起的 ExtraSafe 扩展列表

RasterLayer.dll `0x7c73ded0` 读取 int32 数量后逐项调用
AfCore `ExtraSafeReadObject 0x68a7be60`。读取器沿用[图层条目边界依据](arcmap-10.1-map-object.md)，
`B != 0` 使用原生同步长度，`B == 0` 使用已在图层条目观察验证的写侧 A。
已登记键用来识别回引，新对象的顶层键进入暂存表。合成 v8 测试覆盖空项、
新对象、回引、A/B 边界和逐字节截断回滚。

SystemCore 10.1.0.3035 在 `0x7ffd0063` 按键查找原实例后请求
IUnknown。Rust 对 ExtraSafe 扩展回引保留登记类。构造 v8 的未知
扩展后接 CLSID 不同的同键引用，验证只消费引用头并保留后续字段。
该组合尚无实档或人工验证。

复核 10.1.0.3035 原生循环门。LayerLib 的旧式读取器在
`0x786c237e..0x786c2386` 对非正计数跳过循环；RasterLayer 的
ExtraSafe 读取器在 `0x7c73df64..0x7c73df6a` 同样使用有符号比较，
回边 `0x7c73dfda` 使用有符号小于。构造 v7/v8 尾段验证零、-1
及最小 int32 均保留后续 double 和尾段边界。负计数尚无实档或人工验证。

## 扩展对象体与嵌套键

RasterLayer.dll `0x7c73ded0` 的请求 GUID `0x7c76775c` 为 IID_IUnknown
（`{00000000-0000-0000-C000-000000000046}`）。Save `0x7c73de10`
逐项把集合成员传给 ExtraSafeWriteObject。旧式 LayerLib 入口按流内
CLSID 创建对象，直接调用 IPersistStream，零长度时跳过对象体。
两条路径都没有在读取前按具体扩展类筛选。

两代包装现已共用同一个类分派器。对 10.1 中已确认具有 IPersistStream
对象体的内置扩展，测试逐项送入 SceneFilters 四类 3DProperties、
DynamicTiledLayerProperties、GlobeLayerProperties、
TimeLayerTrackExtension、FeatureServiceLayerEditExtension、
E2LayerExtensionValues、TrackingFilterLayerExtension、HatchLayerExtension、
RouteLayerExtension、RepresentationLayerExtension、ServerLayerExtension、
PropertySet 和 EditTemplateManager。每个类标识都必须进入专属体读取器，
不能再落到“未支持的图层扩展”分支。各类正常体、嵌套键、长度不符和截断
回滚仍由本页对应小节的专用测试覆盖；E2LayerDescriptionValues 没有二进制
持久化接口，GlobeLayerExtensionsImporter 也没有 IPersistStream，二者不会
作为首次定义对象体进入列表。ArcMap 10.1 与 QGIS 人工验证尚未进行。

EditTemplateManager `{A0162E85-E170-4B10-A370-CEE08D8D4B8C}`
沿用 FeatureLayer.dll 10.1.0.3035 的 Load `0x6f6d0a80` /
Save `0x6f6d0bc0` 和[模板键取证](arcmap-10.1-featurelayer-tail.md#扩展段格式修正第九轮记录撤销)。
旧式扩展无外壳键，ExtraSafe 新对象带外壳键。两路都把嵌套模板登记在宿主
ObjectStream 表。Rust 在声明体边界内读取，并在截断、长度不符或模板未读完时
回滚尾段游标和键。合成 v3/v8 测试覆盖非空模板、旧式零长度条目和
ExtraSafe 跨条目的嵌套键回引。

ServerLayerExtension（CLSID `F7FD0EC9-F215-4E7F-B1F8-94BA0EB603B9`）由 LayerLib 在 `0x786c2d16` 创建。MapServer.dll 10.1.0.3035（SHA-256 `1da625c8b314283b9fed35b3118d1637bf7dade21a973cadc78173000edd0270`）的主 QI `0x79dcb320` 将 IPersistStream 映射至 `+0x14`。GetClassID `0x79dcb5e0` 和虚表 `0x79df65ec` 对位 Load/Save `0x79dcb620 / 0x79dcb6a0`。体为版本 short（≤1）和 IPropertySet 普通对象槽（IID `F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`，常量 `0x79df6634`）。两代 RasterLayer 扩展及共享 LayerLib 扩展读取器都按声明长度读取，保留属性集的新键与回引。构造测试验证两种包装内跨条目回引、对象体截断和多余字节回滚。

ServerLayerExtension 的属性回引沿用同版 SystemCore 的按键查询规则
（`0x7ffd0063`）。结合 MapServer Load `0x79dcb620` 请求的
IPropertySet，读取器按登记实例的类校验回引。两种构造包装均验证头部
CLSID 不同而登记类匹配时恢复登记类，以及错引其他类时回滚整段。
ArcMap 真实扩展宿主尚未验证。

### RepresentationLayerExtension

10.1 介质注册该类为 `DE7C3791-49E4-48EA-BFE3-7363DD47CD49`。
对 RepresentationLayer.dll 10.1.0.3035（SHA-256
`a1e6dd834d7062986a75eaed890fe4db7ed7a3db2b1ad4985f86ef3a8558f010`）
补充反汇编。RTTI `0x7cb9444c` 标记的主 QI `0x7cb6a9e0` 将
IPersist 和 IPersistStream 映到 `+0x18`，虚表 `0x7cb8bd3c`。该表
的 GetClassID `0x7cb6a8a0` 返回 `0x7cb8b71c` 中的注册 CLSID。
Load `0x7cb6aca0` 仅调用 AfStreamRead 读取一个有符号 short，大于 1
报错。Save `0x7cb6acf0` 从 `0x7cb8bca0` 写出常量 1。
对象体没有嵌套槽。两代 RasterLayer 包装、FeatureLayer 旧式扩展和
图形容器共用体读取器。构造用例覆盖正负版本、旧式零长度跳过、
ExtraSafe 外层回引、截断、版本越界以及体长不符。两套介质各 66 和 68 个
`/Maps` 流均未命中该 CLSID；实际宿主仍待 ArcMap 运行取证。

### RouteLayerExtension

RouteEvents.dll 10.1.0.3035（SHA-256
`c4655ccbb93363fff759825d7664b9897be64d328d975ed1c5827db97148097c`）
中由虚表 `0x7cc8e5c8` 对位 GetClassID `0x7cc4f260` 和 Load/Save
`0x7cc502d0 / 0x7cc503a0`。GetClassID 写出 `0x7cc8bf40` 中的
`7AE2EC78-3A86-4B7B-901B-A30D1D99F4CA`。对象体依次为版本 short、
AfCore String 和两个 LoadObject 槽。两槽的请求 IID 分别由
RouteAnomalyProperties 的 QI `0x7cc3bd00` 与 RouteIdentifyProperties
的 QI `0x7cc4edb0` 互证。

- RouteAnomalyProperties（CLSID `17DAC573-752F-483F-BBA2-3E7FA4AB34C1`）
  由虚表 `0x7cc8c4f4` 对位 Load/Save `0x7cc3c080 / 0x7cc3c270`。
  版本后读两个 double、四个 short、IQueryFilter、signed LONG 数量和
  ISymbol 循环。AfStreamRead 导入 `0x7cc8a090` 是 double 重载，
  `0x7cc8a098` 是 LONG 重载。IQueryFilter 和 ISymbol 的 LoadObject
  调用位于 `0x7cc3c190`、`0x7cc3c1e1`。循环在
  `0x7cc3c1c8..0x7cc3c1ce` 跳过非正数量，使用 signed 比较
  `0x7cc3c217..0x7cc3c21d` 继续下一项。原生成功路径在
  `0x7cc3c20b..0x7cc3c214` 使用新符号指针，读取器拒绝空符号。
- RouteIdentifyProperties（CLSID `F96F4050-0E8F-4504-ACDF-784E4D240509`）
  由虚表 `0x7cc8e47c` 对位 Load/Save `0x7cc4efa0 / 0x7cc4f060`。
  版本后读六条 AfCore String 和一个 signed LONG。

三个 Load 的版本门均为有符号 short ≤1，Save 均写 1。两代 RasterLayer
包装、FeatureLayer 旧式扩展和图形容器共用有界读取器。子槽复用
QueryFilter、SpatialFilter 和已有 ISymbol 读取器，并提交子对象的新键。
构造用例覆盖跨扩展条目的属性回引、符号回引、截断、未知符号和计数越界；
两套介质的 134 个 `/Maps` 流及参考 MXD 未命中这三个 CLSID。
标量的业务定名及实际宿主仍待进一步取证，ArcMap/QGIS 人工检查未进行。

### HatchLayerExtension

Hatch.dll 10.1.0.3035（SHA-256
`a756f3867987695cdcf9b5e087f82ac2603e7d7a17d2c3e7f5d895dbf47972ba`）
的构造指令 `0x7582856a / 0x758285c6` 使用虚基表 `0x758380d8`
将持久化虚表 `0x758380b8` 安装至完整对象 `+0x3c`。
GetClassID 经 `0x75829260` 到 `0x75828aa0`，返回 `0x75838118`
的 `37AD48DC-12BE-45A0-A406-26C4827E4AF5`。Load 经调整跳板
`0x75829230` 到 `0x75828ae0`，先读版本 short（有符号上限 1）
和一个字节，再委派 `0x75828170`。Save 经 `0x758292d0`
到 `0x75828b60`，写对应前缀并委派 `0x75828010`。

用 10.1 PE 虚表、objdump 指令和独立 pyghidra 反编译复核后续对象体：

- 扩展的委派体先读 signed LONG 计数。正计数每项读 AfCore String 和
  一个 HatchClass 槽（CLSID `FBE27C32-80E5-49EF-B2A4-B7F956992C7C`）。
  `0x75828202` 使用原流调用 AfCore LoadObject，请求 IID
  `2FA4E172-5F34-4911-B38B-FF8EA28C316B`；写侧
  `0x758280ff` 使用同一流调用 SaveObject。类槽因此使用宿主引用表。
- HatchClass 的持久化虚表 `0x75837514` 经跳板到
  Load/Save `0x75822460 / 0x758225f0`。体为版本 short、一个布尔字节、
  HatchTemplate 槽、IQueryFilter 槽、两个 double 和四个刻度定义槽。
- HatchTemplate（CLSID `440A75BC-A9C7-4C50-A4A7-0FABB0E4C521`）
  的虚表 `0x758388e8` 经跳板到 `0x7582d1d0 / 0x7582d3f0`。
  版本后依次为 AfCore String、double、刻度定义槽、double、LONG、
  两个布尔字节、LONG、布尔字节和 signed LONG 计数。每个计数项为
  LONG 与刻度定义槽，LoadObject 位于 `0x7582d396`。
- HatchLineDefinition（CLSID `1E1F49F5-1067-4073-AD2E-12FA4A1950F8`）
  和 HatchMarkerDefinition（CLSID `CE0785E9-CA51-4B83-BA73-3FBFDA99F3E2`）
  的 Load 分别为 `0x75829ad0` 和 `0x7582ad20`。两者读取版本后均委派
  `0x758239a0`，依次读 ISymbol、ITextSymbol 槽、double、LONG、三个
  布尔字节、三个 AfCore String 和三个 LONG。线定义随后再读两个 double。

以上各 Load 允许 signed short 版本 ≤1，两个计数段跳过非正数量。
两代 RasterLayer、FeatureLayer 旧式扩展和图形容器共用解析器。构造测试
覆盖线与标记定义的不同边界、跨槽回引、未知定义、计数越界和截断回滚。
两套介质 134 个 `/Maps` 流及参考 MXD 未命中上述五个 CLSID。
仍缺真实非空扩展和目标软件人工验证。

### TrackingFilterLayerExtension

TrackingTools.dll 10.1.0.3035（SHA-256
`dd5dcd1e633d8eb5a6e9ab8f34775c71e4785b9ab1ec957fb5d17a4c5d8dff02`）
的 ATL CComObject RTTI `0x100abe48` 指向持久化虚表 `0x1008f3e4`。
GetClassID `0x1001a640` 写出
`390172EE-6B0D-41E5-A2F5-EE4A173885FB`。Load/Save
`0x1001a680 / 0x1001a8a0` 依次处理 double、signed int、两个 ATL
Variant 对象槽及两个 ATL BSTR。写侧 double 恒为 1.0，读侧不检查其值。

- Save `0x1001a937 / 0x1001a982` 恒以 VT_UNKNOWN（13）调用 ATL
  入口 `0x1001b1a0`。它在 `0x1001b2e5` 调用 OleSaveToStream，
  空指针则在 `0x1001b31a` 用 WriteClassStm 写零 CLSID。Load
  经 `0x1001b4b0` 到 `0x1001b800`，直接 ReadClassStm、激活并将
  原流传给 IPersistStream::Load（`0x1001b9c4`）。因此两槽外层均无键，
  内层 PropertySet 的对象属性仍使用同一宿主引用表。
  Load 对结果请求 IPropertySet（IID 数据 `0x10094f6c`），默认实例为
  `588E5A11-D09B-11D1-AA7C-00C04FA33A15`。读取器覆盖原生写出的
  VT_UNKNOWN、零 CLSID 及这个 PropertySet。其他 ATL VARTYPE 和
  IPropertySet 实现的实际扩展宿主仍待取证。
- BSTR 从 `0x1000d4f0` 读 4 字节长度。零长度表示空指针，非零长度
  含 2 个终止字节，原生上限 `0x100000`。原生按字节分配 BSTR，并
  单独消费终止字节。读取器保留载荷原始字节及空指针与空 BSTR 的区别。

两代 RasterLayer、FeatureLayer 旧式扩展和图形容器均接入。构造用例覆盖
两个属性集之间的内部键回引、空槽、BSTR 边界、未知类及截断回滚。
参考 MXD 及两套介质 134 个 `/Maps` 流未命中该 CLSID，实际非空宿主
和 ArcMap/QGIS 人工检查仍待验证。

### E2LayerExtensionValues

NMFDocument.dll 10.1.0.3035（SHA-256
`31471c4f234bfcad1a780fe3f60ab2640646ea7628777ab9d33d1ff7a4311d0f`）。
`CComObject<CE2LayerExtensionValues>` 主 QI `0x7b307a30` 使用
`0x7b32857c` 的 ATL 接口表，IPersist 和 IPersistStream 共用
`+0x04`、虚表 `0x7b3285e0`。GetClassID `0x7b3077e0` 返回
`0E8C466B-2ED5-4B59-8B51-7F7541F3FF52`。
Load `0x7b307820` 读版本 signed short（≤1）、描述对象槽、一个布尔
字节。Save `0x7b3078c0` 的版本表 `0x7b327cd4` 为
`[-1, -1, -1, -1, -1, 1]`，表项为 `-1` 时直接返回。

LoadObject 调用 `0x7b307888` 请求
`732ABF5D-9AF4-44B3-8F6C-6D72AAF22986`。同模块
E2LayerDescriptionValues（CLSID `FD7E95FC-F4F8-486F-A3DE-FBAF111404F5`）
的 QI `0x7b306260` 使用 `0x7b327eb0` 的接口表。该表除请求的描述
接口外只有 IXMLSerialize（`C8545045-6615-48E3-AF27-52A0E5FC35E2`）。
两代 RasterLayer、FeatureLayer 旧式扩展及图形容器接入描述空槽及同类已见
键的形态。新描述对象缺少二进制体证据，失败时回滚。构造用例覆盖版本门、
描述槽边界、回引和截断。两套介质 134 个 `/Maps` 流及参考 MXD 未命中
扩展和描述 CLSID，实际宿主仍待 ArcMap/QGIS 验证。

复核同版原始 PE 的 ATL 描述接口表。每项 12 字节，`0x7b327eb0`
返回描述接口（对象偏移 0），`0x7b327ebc` 返回 IXMLSerialize（偏移 4），
`0x7b327ec8` 终止。没有转发项、IPersistStream 或 IPersistVariant。
72 份 10.1 介质类型库也未发布该私有描述 IID。因此原生
E2LayerDescriptionValues 缺少从普通 ObjectStream 读取首次定义所需的
持久化接口，不能按该类的 XML 体推断流字节。其他实现和原生回引来源仍开放。

### E2LayerPropertiesValues 候选

同版 NMFDocument.dll 的类工厂记录 `0x7b3333dc` 指向注册 CLSID
`DB77093F-1BA5-43EC-947B-5F220F0586C9`。用 RTTI 将
`CComObject<CE2LayerPropertiesValues>` 的主虚表 `0x7b3287ac` 对到
QI `0x7b30b260`，再以 objdump 和 PE 原始字节核对它使用的 ATL 接口表
`0x7b32871c`。表中两条具名接口分别是私有接口
`BF234C64-B7E6-4A7E-A968-3C20CB1F5D1D`（偏移 0）和
IXMLSerialize `C8545045-6615-48E3-AF27-52A0E5FC35E2`（偏移 4）。
第三条为无 IID 的回调 `0x7b30bf20`，写出原对象指针并返回成功；
随后在 `0x7b328740` 终止。该回调没有返回独立的持久化虚表。

这组虚表及接口表未提供用于两代 RasterLayer 扩展包装的 IPersistStream
对象体，不能沿用 E2LayerExtensionValues 的二进制读法。旧式包装可以包含
零长度条目，以上证据仅限定首次定义的对象体，不推断条目是否出现。聚合宿主
提供的额外接口仍待取证，未在 ArcMap 中运行验证。

### FeatureServiceLayerEditExtension

介质将 `{9A2BDA48-AD26-4706-BB88-F47D7B0C8F32}` 注册到
MapServerLayer.dll。复核 x86 `10.1.0.3035`，SHA-256
`59957732405d0ada878e85d002cc465f271b1b25fe56fdcef97faced0f8a6d03`。
主 QI `0x79ea9a00` 在 `0x79ea9a39` 比较 IPersistStream，并返回对象
`+0x14`。对应虚表 `0x79ec7a38` 的 GetClassID `0x79ea9b90` 返回注册 CLSID，
Load/Save 为 `0x79ea9bd0 / 0x79ea9ca0`。Load 先读有符号 short（上限 1），
在 `0x79ea9c3e`、`0x79ea9c5b` 两次请求 `0x79ebc2e0` 的 IName，
其后 `0x79ea9c72` 调用 AfCore `ReadString(BSTR*, IStream*)`。Save 在
`0x79ea9cf4` 和 `0x79ea9d00` 保存两个名称对象，再调用 `WriteString`。
名称槽复用已知的表或要素类名称体，按宿主 ObjectStream 键表处理回引。尚未
对位 BSTR 及两个名称成员的业务属性。

### TimeLayerTrackExtension

介质将 `{99DF44C5-02D3-4CF2-AD42-59FD26B614FA}` 和 TimeWindow
`{01BA3B3A-80AB-423B-BB7C-93137CDAAFE5}` 注册到 MapAnimation.dll。
复核 x86 `10.1.0.3035`，SHA-256
`34f688f6b5f390554f9b7ac016dd6ad0bbefdcc5d2bbb69c666e1cc0a7794eed`。
`0x794e3f40` 枚举图层的 ILayerExtensions，缺少该类时经 `0x794e4001`
创建，在 `0x794e4014` 调其 `+0x14` 槽加入集合。
扩展的 IPersistStream 虚表 `0x794fd828` 以 `0x794f5450` 为 Load，依次
读取 short 版本（1–2）、ILongArray、signed LONG 数量和 ITimeWindow
列表。v2 从 `0x794f5583`、`0x794f5598` 与 `0x794f55f4` 读取三个 short。
保存侧 `0x794f5280` 的版本表 `0x794fcc5c` 为 `[-1,2,2,2]`，有效项写 2。

ITimeWindow IID `0x794fd34c` 为 `{6ECFE5F7-1227-4565-AC21-626316A84D0A}`。
类型库扫描只定位到 TimeWindow，持久化虚表 `0x794fd954` 的 Load/Save 为
`0x794f6fe0 / 0x794f6f30`。Load 接受 short 版本 1–3，随后从
`0x794f7024` 和 `0x794f702e` 读取两个 AfCore String。v2 起加两个 String
和一个 short。v3 在 `0x794f7070` 再读一个 String。Rust 复用 LongArray
体读取器，TimeWindow 新对象及回引共用宿主键表。标量和字符串暂按原始顺序保留。

两个扩展接入 RasterLayer 的 v3–v7 旧式包装与 v8 起的 ExtraSafe 包装，
同时接入 FeatureLayer 和共用 LayerLib 旧式包装。构造测试覆盖嵌套名称、
跨条目回引、TimeWindow 各版本、截断及引用回滚。两套介质的 268 个
`/Maps` 和 `/PageLayout` 流均未检出这两个扩展 CLSID，尚无对应非空扩展实档，
ArcMap/QGIS 人工验收未做。

### DynamicTiledLayerProperties

10.1 介质的 `esri.clsid.ecfg` 将 `7E2335D9-1E8D-4488-9E1E-888557C31B3D`
注册到 DynamicDisplay.dll。复核 x86 10.1.0.3035，SHA-256 为
`7f57f7db88f69f95f10087716a1da338947a5754125fda8a08445ecd6415b278`。
创建调用 `0x6ef8f4db` 使用该 CLSID。原生路径经 `0x6ef8ad14` 查询
ILayerExtensions（IID `BB6A78CA-9C28-4F75-899F-DFF65174082C`），
在 `0x6ef8ad2b` 创建对象并设置属性，随后在 `0x6ef8add9` 调用
该接口的 `+0x14` 将对象加入集合。另一条路径同样在
`0x6efa6881` 创建对象并在 `0x6efa699d` 加入扩展。

IPersistStream 虚表 `0x6efc1ac4` 的 GetClassID、Load、Save 分别为
`0x6ef87020`、`0x6ef878d0`、`0x6ef87a40`。现有导出缺少此模块
的函数，使用 Ghidra 补导 Load/Save，并对照 PE 导入表与原始指令。
Load 读取有符号版本 short（上限 3）、AfCore String、IPropertySet、
另两个 AfCore String、两个 short 和一个 double。IPropertySet
读取点 `0x6ef8793b` 使用 AfCore LoadObject。v2 起追加 LONG、short、
LONG，v3 加读一个 short。Save 版本表 `0x6efbe9c0` 为 1、2、3。

两代 RasterLayer 包装、FeatureLayer 旧式包装及共用 LayerLib 分派
使用同一读取器。属性集对象复用宿主引用表，扩展体截断或长度不符时
回滚子键。构造用例覆盖三个写出版本、跨扩展条目的属性集回引、
截断及多余字节。原生调用确认可挂接到 ILayerExtensions，具体
RasterLayer 宿主仍须实档验证。
全量 Rust 测试含忽略项共 1071 项通过。参考工程与 134 份介质工程的
`/Maps` 和 `/PageLayout` 均读至流末尾，两个流中未命中本类 CLSID。
ArcMap 与 QGIS 人工验收未做。

objdump 复核同一 DynamicDisplay.dll，`0x6ef8792d` 将 IPropertySet
IID 地址 `0x6efc1b0c` 传入 `0x6ef8793b` 的 LoadObject 调用。
同版 SystemCore 在 `0x7ffd0063` 按键查询登记实例的接口。两代
RasterLayer 扩展的构造测试覆盖登记类匹配时采用登记类，以及错引
非属性集对象时回滚整个尾段。真实宿主仍待 ArcMap 10.1 验证。

### GlobeLayerProperties

GlobeLayers.dll 10.1.0.3035（SHA-256
`79346fc1fa0d25c01ac41ae2a7b0ee3014847c38f1661ff7d23cce75fe6fd1b8`）
的持久化虚表 `0x72f7720c` 对位 GetClassID `0x72f40af0`、
Load `0x72f47ae0` 和 Save `0x72f47180`。类标识
`2C0D96EA-CAAF-4BDE-848C-3FF86BD9F525` 取自 `0x72f77798`。
Load 版本门接受 1–48，Save 版本表 `0x72f745f0` 的六个值为
21、21、36、39、45、48。Ghidra 反编译和 objdump 指令显示，
`0x72f68bf0` 的 gmx 数据导入路径经 `0x72f69280` 实例化此类，
在 `0x72f68e53` 调用 ILayerExtensions 虚表 `+0x14` 加入扩展。

体首为 short 版本、LONG、AfCore String、普通 IGlobeHeightProperties
槽及控制两个 AfCore String 的 short。此后固定宽度字段分段保留，按 Load
的版本门消费。v26 short 先于 v23 double，v27 LONG 先于 v24 LONG。
v28、v35 各追加一个 AfCore String，v40 的 String 位于两个 short 与五个
double 之间。v42 加入普通 IPropertySet（LoadObject `0x72f48940`），
v48 在四段各 48 字节的数据后加入普通 IFeatureIDSet（IID
`D5BB4B87-E0A1-11D2-9F4D-00C04F6BC78E`，实现 CLSID 首组为
`D5BB4B88`）。两槽复用已有对象体读取器。原生读取后对旧枚举和标量的
调整不改变流消费，读取器保留原始标量字节。

GlobeHeightProperties 的虚表 `0x72f75a38` 对位 GetClassID
`0x72f3b100`、Load `0x72f3b140` 和 Save `0x72f3b420`。CLSID
`69FBCDDF-43DE-4227-8B7C-A19CC688B913`，Load 版本门接受 1–7，
Save 表 `0x72f74608` 的值为 −1、4、7、7。版本与模式 LONG 之后，
v4 起或模式 1 有一个 AfCore String。模式 2 额外读旧式图层名
（v1–v4）或 ExtraSafe ILayer（v5 起，IID `34C20002-4D3C-11D0-92D8-00805F7C28B0`，
调用 `0x72f3b233`）。其后依次读一个 double、v1–v2 的旧式 String
或 v3 起的 double、v2 起及 v3 起各一个 short、v4 起的 LONG 和
String、v6 起的 short。

两代 RasterLayer 扩展包装、FeatureLayer 和共用 LayerLib 旧式分派使用同一
有界体读取器。高程图层槽复用完整图层体分派和嵌套深度限制，已知新对象及回引
共用宿主引用表。遇到未支持的图层类、短读或长度不符时回滚。
构造用例覆盖写端版本、旧式高程模式、新图层的两种 ExtraSafe 同步边界、
新键、回引和截断。介质的 268 个 `/Maps` 和 `/PageLayout` 流未检出
此扩展，实际 MXD 宿主与 3D 设置语义尚待运行取证，ArcMap/QGIS 人工验收未做。

### SceneFilters 的四类 3DProperties

从 10.1 介质的 esri3DAnalyst 类型库定名，再对照 SceneFilters.dll x86
10.1.0.3035 的 CLSID 常量和 IPersistStream 虚表。模块 SHA-256 为
`fc605f3ade84db3379ffed69d2fc93effa61599ceff5a2b08cb228f4678d2bbe`。
既有反编译导出缺少 SceneFilters.dll，使用独立 pyghidra/Ghidra 12.2
导出 Load，并用 objdump 复核分支、调用和 Save 的版本表。反编译及汇编
在忽略目录 `temp/mxd-scene-filters/` 和 `temp/mxd-extension-SceneFilters.asm`。

| 类 | CLSID | IPersistStream 虚表 | GetClassID |
| --- | --- | --- | --- |
| Basic3DProperties | `A5A8DC61-268C-11D4-914A-0080C7074326` | `0x7d30ee5c` | `0x7d305880` |
| Tin3DProperties | `B7A0A25D-24F7-11D3-9F4D-00C04F6BC5F4` | `0x7d30f4a4` | `0x7d30aa90` |
| Raster3DProperties | `4A4CECE3-F53A-11D3-80D9-00C04FA0ADF8` | `0x7d30eabc` | `0x7d3056f0` |
| Feature3DProperties | `4A4CECE4-F53A-11D3-80D9-00C04FA0ADF8` | `0x7d30e4e4` | `0x7d301010` |

四张虚表均以 `0x7d3088c0` 加载、`0x7d3090b0` 保存。Load 接受
版本 1–13，Save 在 `0x7d30eb38` 的六档表为 6、6、9、10、10、13。
SceneRenderers.dll 10.1.0.3035（SHA-256
`1674d87a74366cff2943e45d3dd1fd9e10d89960b9480d1abea44ffd84c56da7`）
在 `0x7d4b31e5` 查询 ILayerExtensions，经 `0x7d4b3225` 枚举扩展，
在 `0x7d4b3244` 查询条目的 I3DProperties（IID `BC6F44CC-21E8-11D4-A383-00C04F6BC619`）。
该调用链确认此接口用于读取图层扩展配置；实际 MXD 的宿主分布仍待实档验证。

- 版本 short 后是地表模式 LONG。v1 总是读 String。v2 起模式 0 读 String，
  模式 1 调 `0x7d3089fa` 读普通 IName，v4 起紧接路径 String，其他模式跳过这组字段。
  随后的 IName::Open 调用处理打开的数据源，Rust 保留流中对象及引用。
- 字符串之间按 `0x7d308bb9..0x7d308d44` 的版本门消费固定宽度字段。
  v5 起的 4 字节在 v3 起的 8 字节之前，随后是 8 字节、String、4 字节、
  String 及 18 字节。其后 v3、v6、v7、v8、v9、v10、v11、v12 分别追加
  4、4、8、4、4、8、48、8 字节。v13 在末尾经 `0x7d30907d` 调普通
  LoadObject，请求 IEnvelope（IID `1B077914-38C5-11D0-92D2-00805F7C28B0`）。
- IName 槽复用已支持的栅格名称分派，并接入 TinDB.dll 的 TinName。
  TinDB.dll x86 10.1.0.3035 的 SHA-256 为
  `4b9c7a5155fb0b18a0e819937dde40115f613bd0831bb45b0947fe64d287c86d`。
  虚表 `0x8037c74c` 的 GetClassID `0x80354020` 返回
  `5F13F416-42BC-11D2-A569-0000F8774F0F`。Load `0x803540e0` 用
  有符号 short 上限 1，然后读三个 AfCore String 和一个普通 IWorkspaceName，
  写侧 `0x80354070` 对应。已支持的名称和包络从宿主 ObjectStream 表登记或回引。

Rust 对两代 RasterLayer、FeatureLayer 旧式扩展和共用 LayerLib 扩展
分派使用同一有界读取器。对象体读完且声明长度吻合时才提交子键。
合成测试覆盖 v1–v13、地表模式、普通 RasterLayer 两种扩展包装、跨层体
名称回引、TinName 的工作空间和嵌套包络新键，以及逐字节截断回滚。
`cargo test --offline` 共 1051 项通过、28 项忽略。参考工程与两套介质
的 134 份工程均完整读到 `/Maps` 和 `/PageLayout` 末尾，270 个流中
没有命中本组四个 CLSID。其他 IName 实现、带本组扩展的实档以及
ArcMap/QGIS 人工验证仍开放。

### 独立 PropertySet 扩展与普通图层入口

复核 RasterLayer.dll 10.1.0.3035 的 ILayerExtensions 委派。主 QI 在
`0x7c73afe3..0x7c73b00b` 对 IID `0x7c7690b0` 返回完整对象
`+0xec`。构造指令 `0x7c736afe` 安装虚表 `0x7c76c5ec`，
其 `+0x14` 指向 AddExtension `0x7c743150`。现有伪代码与
objdump 显示，该入口只检查 IUnknown 指针是否为空，然后 AddRef
并加入数组。RasterLayer 的 Save `0x7c73d0a0` 依据文档保存版本
把同一数组交给旧式 LayerLib SaveExtensions 或 `0x7c73de10` 的
ExtraSafeWriteObject 循环。两代包装均允许直接添加具有
IPersistStream 的 SystemCore PropertySet。PropertySet Load
`0x7ffd3080` 接收原流，内部对象值沿用宿主引用表。

Rust 接入两代 RasterLayer 及共用旧式图层扩展入口中的独立 PropertySet，
按声明边界确认对象体及内部新键。普通图层在完整性检查中同时认可已有的
DynamicTiledLayerProperties 对象体。合成 v3/v8 普通 RasterLayer
验证独立 PropertySet 登记内部对象，随后 DynamicTiledLayerProperties
回引同一键，并验证后继边界、截断、体积不符和失败回滚。
未定位到实档中的独立 PropertySet 扩展宿主，ArcMap/QGIS 待人工验收。

### GlobeLayerExtensionsImporter 候选

介质将 `BCCF4E7B-A9C6-4441-96B4-4F7E078E6A4C` 注册到
GlobeLayers.dll 10.1.0.3035（SHA-256
`79346fc1fa0d25c01ac41ae2a7b0ee3014847c38f1661ff7d23cce75fe6fd1b8`）。
其主 QI `0x72f3e4e0` 只直接返回导入器接口，未命中时委派
`0x72f6d780`。后者只比较三个 GUID（数据
`0x72f744ec / 0x72f7450c / 0x72f79348`），其余返回 E_NOINTERFACE。
两处均没有 IPersistStream 或 IPersistVariant 的分派。不能从该导入器
的注册项推导扩展流中可用的对象体。

参考工程两个有界 RasterLayer v17 的扩展列表均为空。66 份 10.1 介质
/Maps 中未定位到带 ExtraSafe 前缀的 RasterLayer 候选，尚缺带此扩展的
真实 RasterLayer 或 ArcMap 运行验证。其他扩展类保留原始体，内部对象键尚待取证。

## v12 变换与包络

`0x7c73bd40` 在 v12 检查持久化接口 `this+0xb0` 对应的数据集对象是否
已打开，然后才进入 `0x7c73c716` 的两个 LoadObject 调用。两槽分别请求
IGeodataXform 和 IEnvelope，之后下一段由版本 ≥14 控制，因此 v12 中两槽
构成可选末段。Rust 使用父 ExtraSafe 条目界判断省略形态。存在剩余字节时，
两槽均需读完并恰好到界。合成测试覆盖完整与省略形态、截断和多余字节。

### v12 尾段的原生读取实验

使用本文开头同 SHA-256 的 x86 RasterLayer.dll 10.1.0.3035。对照
仓外导出和 objdump，主 Load 在 `0x7c73be97..0x7c73be9e` 将
`this-0x9c` 传给名称读取器 `0x7c73b380`。单名称路径在
`0x7c73b752` 调 IName::Open，在 `0x7c73b77a` 查询打开结果的
接口，成功分支在 `0x7c73b7bf..0x7c73b7ea` 把对象成员
`+0x14c` 交给该接口的 `+0x18` 方法作为输出地址。
`0x14c-0x9c=0xb0`，正是主 Load 判断是否读 v12 尾段的成员。
多个名称的路径也调用 Open（`0x7c73b9f8`），并可能在
`0x7c73bb0c` 更新同一成员。可选 RasterIO 名称在
`0x7c73c574` 调 Open，非空结果进一步进入 `0x7c757df0`
及图层虚调用，见 `0x7c73c583..0x7c73c597`。因此即使保存的名称
是已知类且成功读取，仍需实际数据源状态才能按原生分支决定游标。

将原件 `0x7c73c6f6..0x7c73c767`、`0x7c73c903..0x7c73c920`
及两个小型引用助手按原 VA 映射到 ELF，在 QEMU i386 中执行。
驱动直接提供 `this+0xb0`，由 IAT `0x7c7670f8` 上的回调记录
LoadObject 请求并读取 16 字节空对象标记。四组使用完全相同的输入，
即两个空对象标记（32 字节）后跟一个标记字节。

| 图层版本 | `this+0xb0` | 被调用的对象槽 | 末段消费字节 |
| --- | --- | --- | --- |
| 11 | 非空 | 无 | 0 |
| 12 | 空 | 无 | 0 |
| 12 | 非空 | IGeodataXform、IEnvelope | 32 |
| 13 | 空 | IGeodataXform、IEnvelope | 32 |

两处 IID 由原件 `0x7c76c914` 和 `0x7c768fa0` 读出并在回调
中核对。实验在版本判断后的汇合点截停，未执行完整 COM 加载、名称打开
或 ArcMap 文档打开。脚本和二进制输出位于忽略目录
`temp/mxd-rule-objects/raster-v12-native/`。普通对象的读取仍需
从宿主结构或真实旧档得到独立的边界证据。

### v12 单名称打开失败后的读取

复核同摘要 RasterLayer.dll 10.1.0.3035 的 `0x7c73b380`。
当列表有且仅有一个名称时，`0x7c73b754..0x7c73b78b` 检查
Open 输出对象以及后续查询得到的接口指针。任一指针为空时，
`0x7c73b7a0` 返回 S_FALSE（1），该路径没有写入数据集成员
+0x14c。若接口存在，`0x7c73b7dd` 先清空成员，`0x7c73b7ea`
请求填充；输出仍为空时，`0x7c73b7ec..0x7c73b815` 汇入同一个
S_FALSE 返回。主 Load 在 `0x7c73bea5..0x7c73bea7` 仅对负值
提前返回，S_FALSE 也会在 `0x7c73becc` 继续读取渲染器。

因此，一个成功反序列化的非空名称可以进入后继读取而没有打开
数据集。后续 RasterIO 槽仍可能改变该状态，只有检查完该路径后
才能依据数据集状态选择 v12 尾段。单名称数量及名称 CLSID 无法
独立确定普通对象的终点。本次核对使用 Ghidra 导出和 objdump，未在
ArcMap 10.1 或 QGIS 运行验收。

### v12 RasterIO 二次打开的派发

复核同摘要 RasterLayer.dll 10.1.0.3035。RasterIO tag 30 的 IName
在 `0x7c73c574` 被打开。输出为空时，`0x7c73c576..0x7c73c57b`
跳过二次安装。输出非空时，`0x7c757df0` 经 `0x7c74b3f0`
查询 `0x7c769370` 的 IID，查询失败时保留空指针。

构造器 `0x7c736a36` 将持久化表安装到完整对象 +0x9c。
Load 以该接口为 this，所以 `0x7c73c58c..0x7c73c597` 使用的
+0x68 接口对应完整对象 +0x104。构造器 `0x7c736b3a` 在那里安装
虚表 `0x7c76c688`，其 +0x18 槽指向 `0x7c745410`。该入口在
传入空接口或该接口的 +0x18 调用未给出对象时，经 `0x7c74546c`
返回 S_FALSE。Load 在 `0x7c73c599` 继续清理临时接口，未检查
该 HRESULT。由此可确认，出现 tag 30 乃至名称 Open 输出非空，
都不足以证明数据集已安装。成功路径对 v12 选择成员的影响仍需追踪。
本节为静态取证，未进行 ArcMap 10.1 或 QGIS 人工验收。

#### 二次打开要求图层已有栅格成员

对同摘要 RasterLayer.dll 10.1.0.3035 的 Ghidra 导出
`7c745410.c` 和 `7c737a90.c` 用 objdump 复核。
`0x7c745434` 取得二次打开的栅格后，`0x7c74543f..0x7c745452`
又从完整对象 +0x90 的接口调用 +0x58。构造器 `0x7c736a18`
安装的虚表 `0x7c76c17c` 在 +0x58 指向 `0x7c737a90`。
该 getter 在 `0x7c737aa6` 读取接口相对 +0x1c0，即完整对象
+0x250。该成员为空时返回 S_FALSE；调用方预先清零输出，
随后在 `0x7c74545a` 因输出为空进入 `0x7c74546c`
的 S_FALSE 返回。只有二次打开结果和已有成员都非空，
才在 `0x7c745480` 调用双栅格辅助例程 `0x7c736470`。

单名称打开的一个成功分支在 `0x7c73b83c..0x7c73b859`
将 +0x14c 的数据集成员引用复制到 +0x250。因此二次打开的
这条路径在已有栅格成员为空时提前结束，即使 tag 30 成功打开。
v12 末段仍检查另一个成员 +0x14c。现有静态证据无法将磁盘
上的名称直接映射到这两个运行时状态，也未覆盖二次打开成功
路径中所有间接调用的副作用。普通无界 v12 仍待宿主边界或
数据源打开取证，ArcMap 10.1 与 QGIS 人工验收未进行。

### 旧 Map 的 ExtraSafe 图层

复核 x86 Map.dll 10.1.0.3035（SHA-256
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`）
的双列表读入口 `0x794038a0`。v18–v22 的第一组在 `0x79403afd`
经 IAT `0x794801e4` 调用 AfCore ExtraSafeReadObject，第二组在
`0x794039f9` 经 IAT `0x79480048` 调用 LoadObject。解析器现将第一组
的实际对象终点传给 RasterLayer，以及最后一个槽为底层栅格的 WCSLayer。
合成 v19 双列表覆盖 A 长度、B 同步长度、两种可选尾段、第二组的栅格
回引和活动图层索引。可选尾段截断时停在第一组条目起点，保留进入
该条目前的引用表。

复查 134 份 10.1/10.8 介质工程和仓库参考工程，135 份的 `/Maps`
和 `/PageLayout` 均完整读至结束。两条流中共见两处 RasterLayer
CLSID 字节命中，均无对象头后版本 12 的候选。

同版 RasterLayer::Save 在 `0x7c73d0a0` 使用版本表 `0x7c7678a8`，
选择 7、11、13、16 或 17。原生 v12 加载分支的两种形态仍需更早写出的档案或
运行取证。普通 LoadObject 槽缺少外层终点时，需要证明数据集状态或后继
对象边界。ArcMap/QGIS 人工验收尚未进行。

## v14 颜色索引

反汇编 `Load 0x7c73c983..0x7c73ca88` 确认颜色后 int32 计数是展开后的
元素数。非负 int32 为一个索引，负数的绝对值为从上一索引加一开始的连续
区段终点（含终点）。`Save 0x7c73d0a0` 的对应循环只在写出起始索引后
才写连续区段，反汇编用于纠正 Load 反编译中临时变量的混叠。
读取器限额内展开并保留序列，拒绝超长、倒序终点和整数溢出。
合成 v17 测试在非空索引后继续读取时间段、像素过滤器槽和直方图。

RasterLayer.dll 10.1.0.3035（SHA-256
`9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`）
在 `0x7c73c9b7` 先对声明数量作无符号乘法，用于分配索引缓冲区，
到 `0x7c73c9fb` 才检查循环条件。Rust 将负展开数量视为无效数据。
构造输入覆盖 -1 和最小 int32。未在 ArcMap 或 QGIS 中验证异常输入。

## v16 像素过滤器

`RasterLayer::Load 0x7c73bd40` 在 v16 起通过 AfCore LoadObject 请求
IPixelFilter，槽头遵循宿主 ObjectStream 的键规则。来自同一介质的
`esriDataSourcesRaster` 类型库（SHA-256
`40239c273d40f058799a3d6a3f7de513b547042790ae92c485b1bb852545a793`）
中，从 coclass 的 `datatype1` 沿 RefTab 链检查实现接口。共 14 个
coclass 引用 IPixelFilter（TypeInfo 120），为 TypeInfo 212–225。
RasterDB.dll 10.1.0.3035 SHA-256 见 [名称对象取证](mxd-10.1-raster-renderer.md)。
按 RTTI 和原始虚表核对其中的 IPersistStream 入口：

| 类 | 虚表 | 对象体依据 |
| --- | --- | --- |
| PansharpeningFilter | `0x7c12e284` | 复用 [RGB tag 18 的持久化体](mxd-10.1-raster-renderer.md#rgb-tag-18全色锐化) |
| LutFilter `{E825FC58-151A-4BC6-8AB5-2CC1481FE403}` | `0x7c12d8e8` | GetClassID `0x7c021700`，Ghidra 导出 Load `0x7c021810` 与 Save `0x7c021740`。short 版本、两个 int32、int32 数量后逐项读写 int32。Save 版本表 `0x7c1237bc` 只写版本 1 |
| HistogramSpecification `{B66278B0-8352-499B-AD5C-5140C54EC52F}` | `0x7c12983c` | GetClassID `0x7bff0690`，Load `0x7bff0730` 与 Save `0x7bff06d0` 只处理一个 short。Save 从 `Get92SaveVersion(stream, 1)` 选择版本，返回 −1 时省略写入 |

BackgroundFilter、ColormapFilter、RasterConvolutionFilter、
PixelFilterCollection、MultibandFilter、RemapFilter、ClipFilter、
NoDataFilter、FocalMeanFilter、FocalStatisticsFilter 和 SpeckleFilter
继承 BasicFilter 的 GetClassID、Load 和 Save。RasterDB.dll 虚表槽
分别指向导入跳板 `0x7c10d6d6`、`0x7c10d6d0`、`0x7c10d6ca`。
对应 RasterCoreLib.dll 10.1.0.3035（SHA-256
`6d89601832b788177aac492e7fdfb0f1b012db86e2a75e6e480e6e647508eb1f`）
导出 `0x7beb1100`、`0x7beb1100`、`0x7bef1e40` 均直接返回
`E_NOTIMPL`。未知 CLSID 仍保留为停止点。

v17 合成测试验证三种有键过滤器后继续读取直方图，覆盖逐字节截断、
负数和超限 LUT 数量时的尾段回滚。LutFilter 与 HistogramSpecification
没有 ArcMap 载入验证。

同版 SystemCore 在 `0x7ffd0063` 先按键定位原实例，再查询
IPixelFilter。Rust 以登记类校验回引。构造 v17 误引先前登记的
包络键时，图层尾段的游标和对象键全部回滚。尚无该组合的实档或人工验证。

RasterDB.dll 10.1.0.3035 的 LutFilter Load 在 `0x7c021844` 按有符号
short 接受版本 ≤1。Rust 保留版本位模式。构造 v17 图层分别使用
LutFilter 版本 1、0 和 -1，验证数组和后继直方图的边界。负数组数
仍视为无效，原生在 `0x7c02189e` 先用无符号乘法计算分配大小。
负版本尚无实档或人工验证。

## RasterIO tag 30 名称槽

`RasterLayer::Load 0x7c73bd40` 在 ReadObj(30) 后调用 `0x7c731840`，
经 `0x7c75c2a0` 查询 IID `{4BA33331-D55F-11D1-8882-0000F877762D}`，
由同版 esriSystem 类型库确认是 IName。得到名称时要求读到 tag 31，
之后调用 IName::Open。写侧 `0x7c73d68d` 先写 tag 30 对象，再写 tag 31 字符串。
RasterCoreLib.dll `0x7bf13d90` 在读取 CLSID 后直接调用持久化 Load，
外层名称无键，嵌套对象仍使用传入的 ObjectStream。
对已知名称复用对象体分派，未知类保留为停止点。
合成测试用普通 RasterDatasetName 验证无键外壳、WorkspaceName 键共享、
后续 short 记录，以及缺失 tag 31 时的回滚。参考工程两个图层均省略 tag 30。

本轮新增形态尚未在对应版本的真实工程或 ArcMap 中复核。

### 三个名称宿主的接口与 FullName 来源

在 x86 10.1.0.3035 介质中，将 RasterLayer 名称列表、RasterIO tag 30 和 LoadRaster 的名称 IID 对到同一 IName `{4BA33331-D55F-11D1-8882-0000F877762D}`。IID 数据分别在 RasterLayer 的 `0x7c76c8a4`、`0x7c768564` 和 RasterCoreLib 的 `0x7bf5e5e8`。RasterLayer 写 tag 30 与 RasterCoreLib SaveRaster 都从 **IDataset** 的 `+0x10` 取得 FullName。IDataset、IName 和 IDatasetName 是三个接口，类型库中仅声明 IName 无法证明对象会出现在栅格宿主。

用 RTTI 的 IDataset 基类偏移定位下列虚表，再用 objdump 复核 `+0x10` 函数及创建点。下列创建的类均已有共享名称读取器。

| 10.1 模块 | IDataset 实现 | get_FullName 与新建名称 |
| --- | --- | --- |
| RasterDB.dll | RasterDataset / RasterBand | `0x7c0ac450` 创建 RasterDatasetName，`0x7c048980` 创建 RasterBandName。 |
| NetCDFDB.dll | NetCDFRasterDataset / NetCDFRasterBand | `0x7ae259a0` 创建 NetCDFRasterDatasetName，`0x7ae20ea0` 创建 RasterBandName。 |
| FunctionRasterDB.dll | FunctionRasterDataset / FunctionRasterBand | `0x6fc8c2e0` 创建 FunctionRasterDatasetName，`0x6fc87c50` 创建 RasterBandName。 |
| GdalRasterDB.dll | GdlRasterDataset / GdlRasterBand | `0x70861d20` 创建 RasterDatasetName，`0x708590c0` 创建 RasterBandName。 |
| CacheRasterDB.dll | CacheRasterDataset / CacheRasterBand | `0x6c0e42a0` 创建 CacheRasterDatasetName，`0x6c0e1490` 创建 RasterBandName。 |
| MapServerLayer.dll | InternetTiledLayer | `0x79e32960 → 0x79e357c0` 在 `0x79e357eb` 创建 InternetTiledLayerName。 |

补查已提取的 10.1.0.3035 PE32，按 RTTI 的 IDataset 基类偏移配对完整对象虚表，
再核对 objdump 中的创建参数、IName 查询与底层转发。SdeRasterDB、FgdbRasterDB
和 ImageServerLayer 的栅格波段都使用已有 RasterBandName CLSID
`BC25E113-168B-11D2-8D25-0000F8780535`。

| 模块 | 虚表 `+0x10` 的入口及观察结果 |
| --- | --- |
| RasterJET.dll | AccessRasterDataset `0x7c6f1610` 查询已缓存名称的 IName，缓存为空时转发到底层 IDataset 的 FullName。 |
| SdeRasterDB.dll | SdeRasterBand `0x7f60ac40` 创建 RasterBandName。SdeRasterDataset `0x7f60fcc0` 对已有名称成员请求 IName，并在返回前将底层 FullName 填入原名称对象。 |
| FgdbRasterDB.dll | FgdbRasterBand `0x6f921d20` 创建 RasterBandName。FgdbRasterDataset `0x6f9278d0` 从名称成员查询 IName。UnmanagedFgdbRasterDataset `0x6f939580` 在 IName 缓存与底层 FullName 之间选择。 |
| MosaicDB.dll | MosaicDataset `0x7a3d6950` 创建已支持的 MosaicDatasetName。ReferencedTable `0x7a597f00` 将底层 FullName 包在 ReferencedTableName 中，创建助手为 `0x7a3e0e00`，属于表名称路径。 |
| ImageServerLayer.dll | ISRasterBand `0x7800b1e0` 创建 RasterBandName，ISRasterDataset `0x7800e380` 创建已支持的 ISRasterDatasetName，ISRasterTable `0x780144d0` 返回 `E_NOTIMPL`。 |
| RasterDB.dll | RasterTable `0x7c024010` 返回 `E_FAIL`。 |

模块 SHA-256 与对应名称小节一致。补充校验 RasterJET 为
`b5722104bd55f6f1b72263151918c5294037c8fc36629a820f0e06c6e23ac31a`，
FgdbRasterDB 为 `571be820c79b217571ea8f902c091e44f6268566bd35880488e8c4ef74008fef`，
ImageServerLayer 为 `0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`。
上述转发分支仍取决于底层数据集对象；RTTI 清点也可能遗漏不携带相应类型描述符的实现。

FunctionRasterDB 的 SimpleRasterTable 在 `0x6fcf2f60` 直接返回 `E_FAIL`，不产出名称。RasterDB 的 RasterClassNames 主 QI `0x7c05a740` 只匹配公开的 IRasterClassNames，未命中继续到 `0x7c10a550`。该助手检查 IUnknown 及两个框架 IID，随后返回 `E_NOINTERFACE`。介质登记表含 RasterBandNames CLSID `{92C9131C-DBD1-4A0C-B7A1-D2985E958C06}`，在本轮扫描的 1,115 个提取 DLL 中未命中其 CLSID，也未建立栅格宿主的写入证据。它与已有 RasterBandName、RasterBandCollectionName 是不同登记项。

上述模块版本和摘要沿用本文各名称小节。RasterDB.dll 的本轮 SHA-256 为 `97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`。结论限于已定位的 FullName 分支，未覆盖所有缓存名称来源、动态类工厂和第三方数据集。未在 ArcMap 或 QGIS 中人工验证。

#### Grid 与基础栅格类的名称来源复核

扫描 Desktop 介质中已提取的 DLL，按 MSVC RTTI 的 IRasterDataset、
IRasterBand 和 IDataset 检索，补查 GridCore、RasterCoreLib、RasterLayer
及 NetCDFDB 的基础实现。以下均为 PE32 x86 10.1.0.3035，以 PE 版本资源、
虚表原字节和 objdump 复核。

- GridCore.dll（SHA-256
  `43251973fbe1cf92b35250e035e9128ed466b78ce67138ad6fa9f2bc91f70f51`）
  的 EsriGrid 与 SABand 的 IDataset 虚表分别为 `0x7519717c` 和
  `0x7519d12c`。两者的 get_FullName（`0x750ed090`、`0x75143780`）
  都调用 `0x750f22d0` 查询底层对象的 IDataset，随后调结果的
  `+0x10`。IID 由 `0x75195be4` 的原始字节确认。这两条路径的持久化名称
  由底层对象决定，Grid 包装层没有独立创建名称。
- RasterLayer.dll 的 CompositeRasterDataset 之 IDataset 虚表
  `0x7c76813c` 的 get_FullName（`0x7c748620`）直接返回
  `E_NOTIMPL`。同模块的名称列表 Save（`0x7c73ced0`）逐项把
  `+0x13c` 集合中的现有 IName 交给 `0x7c748c80`。该助手可能为名称
  改写相对路径，仍把原名称指针传给 SaveObject。因此该列表的可能来源
  不能仅以另两个宿主的 IDataset::get_FullName 路径界定。
- RasterCoreLib.dll 的 BasicRasterBand 和 BasicRasterDataset 的
  get_FullName 项，以及 BasicRasterName 的 IPersistStream 虚表
  `0x7bf59534` 的 GetClassID、Load 和 Save 三项，均指向
  `0x7bf4497c`，跳转到 MSVCR90.dll 的 `_purecall`。NetCDFDB.dll
  （SHA-256 `e78ea582fdd66f45dfa717eb22346e9f691e1bb4e26587a1d09f72b0d1bc5ebd`）
  中的两个 BasicRaster 基类 get_FullName 也在 `0x7ae588f0` 跳转
  `_purecall`，由派生类提供名称。这些基础实现没有可据以解码的独立名称体。

已知缓存和 NetCDF 名称体复用同一个带键 LoadRaster 入口，见
[RGB tag 18 回归](mxd-10.1-raster-renderer.md#rgb-tag-18全色锐化)。
未确认动态底层实现实际返回的名字与旧版或外部组件注册的扩展类。

### WCS 名称与 GDAL 数据集全名

OGCClient.dll 10.1.0.3035（SHA-256 `4a7a3a39eafa295677ccf8b1de5ffbab50544bc4c43b6aba343992a21ab28fca`）的 WCSCoverageName（CLSID `C6A467CF-262D-47F0-BF56-DD3A29C56B5F`）主 QI `0x7b424610` 把 IName 映射到 `+0x14`，IPersistStream 由 `+0x18` 的虚表提供。Load/Save `0x7b424bd0 / 0x7b424d00` 读取版本 short（≤1）、AfCore 自由函数 ReadString 的覆盖标识和 IWCSConnectionName 槽。CoverageContentsHandler 在 `0x7b4298ff` 创建该类。连接名 CLSID `B96CFE9E-44C7-4616-8A2A-F1FBA8642AF6`，Load/Save `0x7b420ee0 / 0x7b421090`，读取版本 short（≤3）、IPropertySet，v2 起追加第二个 IPropertySet 槽。接入共享栅格名称分派，构造 tag 30 覆盖连接名 v1/v3、属性集键回引、后续记录边界与逐前缀截断回滚。

同版 RTTI 另把 WMSConnectionName
（CLSID `98B0E997-F21D-4195-8E06-F9CD2AD97165`）和
WMTSConnectionName（CLSID `93B061BB-DB92-4E90-9927-48C031B33FE8`）
的 IName 与 IPersistStream 接口分别映射到完整对象 `+0x18`、`+0x1c`。
两者的持久化入口与 WCSConnectionName 相同，均跳到
`0x7b420ee0 / 0x7b421090`；`0x7b420f1c..0x7b420f23`
对 short 版本作有符号 `≤3` 判断，`0x7b420f64..0x7b420fe0`
先读 IPropertySet，版本 ≥2 时再读第二个 IPropertySet。
CLSID 与 IName 关系由同批 10.1 Desktop 类型库
`esriGISClient` 条目互证。读取器保留端类型 CLSID、版本及两个槽，
因此同一分派同时覆盖 RasterLayer 名称列表、RasterRGBRenderer tag 18
的 LoadRaster 和 RasterLayer tag 30。构造用例分别验证 v1/v2/v3、
属性集回引、后继记录和截断回滚；两套介质未命中 WMS 或 WMTS
连接名 CLSID，实际宿主数据源及 ArcMap/QGIS 人工检查仍待验证。

RasterLayer.dll 的 WCSLayer 有独立 Load/Save `0x7c75b9b0 / 0x7c75bbb0`。其连接入口 `0x7c75bd00` 从服务参数构造 `<WCS_GDAL` 描述，创建 GdalRasterDataset（CLSID `51298BDE-B405-446C-8A12-3EABC2716B0B`，常量 `0x7c769050`）并将其交给底层图层。GdalRasterDB.dll 10.1.0.3035（SHA-256 `d43b9307eb8c87210d825195722c53104def726967366f4a7f9109adfa4432c4`）的 GdlRasterDataset 主 QI `0x7085f4b0` 在 `0x7085f537` 匹配 IDataset，返回 `+0x64`。虚表 `0x7089f13c` 的 `+0x10` 即 get_FullName `0x70861d20`。当名称缓存为空时，它在 `0x70861d43` 以已接入的 RasterDatasetName CLSID `75BCE6E2-8AF5-478E-8892-FA45CA50AF4D` 调用创建助手 `0x7085d5c0`。该链给出标准 WCSLayer 路径上的 GDAL 数据集名称来源。WCSCoverageName 出现在普通 RasterLayer tag 30 的实际写入证据仍缺。

扫描本地 64 个工程的 128 个 /Maps 和 /PageLayout 流，未命中 WCSCoverageName、WCSConnectionName、WCSLayer 或 ServerLayerExtension 的 CLSID 候选。

### WCSLayer 图层体

完整对象的 `+0x2c` 是 IPersistStream，虚表 `0x7c76e5cc` 的 GetClassID `0x7c759dc0` 写出 WCSLayer CLSID `239C10F1-6E2A-4250-B43A-7920DB1CDC88`。读取入口 `0x7c75b9b0` 先读有符号 LONG 版本（上限 17）与三个 AfCore String，v14 起读两个 LONG，v15 起读一个 String，v16 起读一个 String 和 `Variant::Read(false)`，v17 起再读一个 String。Save `0x7c75bbb0` 的版本表 `0x7c767870` 选出 15、16、17，较早布局由 Load 分支确认。

最后一个 LoadObject 请求 IUnknown，随后原生不再消费流。构造函数在 `0x7c759f61..0x7c759f72` 使用 `0x7c7691d0` 的 RasterLayer CLSID 创建底层对象，Save 将该成员写成普通对象槽。Rust 解析时共享宿主键表，通过底层 RasterLayer 提取图层名称和可见性。WCSLayer 若有 ExtraSafe 边界，最后一个槽可以把剩余范围传给底层 v12 RasterLayer 来判定可选变换/包络。普通无界槽仍按已支持的自定界版本读取。构造测试覆盖 WCS v15–17、普通和 ExtraSafe 包装、底层 RasterLayer 的回引、活动索引以及截断回滚。

WCSLayer 主 QI `0x7c75a0e0` 的直接比较无 ILayerMasking。唯一底层转发分支请求的是 `0x7c7683e0` 的 IDataset（`2D04C042-7766-11D0-B77D-00805F7CED21`）。未匹配的遮罩查询落入 `0x7c75d680` 并返回 `E_NOINTERFACE`，因此 Map 解析对已读完的 WCSLayer 不消费逐图层遮罩字节。新增分支暂无真实工程和 ArcMap/QGIS 人工复核。

## 普通 LoadObject 条目

复用 RasterLayer.dll 10.1.0.3035（SHA-256 `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`）的 Load `0x7c73bd40` 与 Save `0x7c73d0a0`。Load 先按现有名称、渲染器、扩展与关联对象的读序推进，再通过 RasterIO InitCheck 探测可选段。RasterCoreLib.dll 10.1.0.3035（SHA-256 `6d89601832b788177aac492e7fdfb0f1b012db86e2a75e6e480e6e647508eb1f`）的 `0x7bf13b30` 先取得流位置再读记录，短读或标签不匹配时恢复位置；RasterLayer 调用的 magic 为 `0xff00ff`，已支持标签及终止记录与有界尾段共用。

objdump 复核 `0x7c73c6fa..0x7c73c716`。只有 v12 测试持久化对象当前的 `this+0xb0`，为零则跳到 `0x7c73cdbd`，否则进入 IGeodataXform、IEnvelope 两个连续 LoadObject；v13 起始终进入，v11 以下始终跳过。数据集名称和可选 RasterIO 名称都可触发打开数据集，静态字节读取无法确定打开后的对象状态。因此普通槽接入 v0–11、v13–17；v12 在 ExtraSafe 范围内以实际终点判定两种形态。当 GroupLayer 作为普通对象出现、其内部 RasterLayer 子项使用 ExtraSafe 时，同样按子项范围读取 v12。

普通槽只有完整读取已支持的名称、渲染器和尾段后才提交；有界未知扩展可能含内部对象键，因此普通层和要求完整依赖的子层拒绝跳过非空未知扩展。有界 Map 图层的保留诊断行为沿用既有路径。构造测试覆盖各支持版本、逐前缀截断回滚、未知扩展、Scene 普通关联槽和组内有界 v12。另用参考工程 `/Maps:0x4fbc`、`0x145a3` 的两个原始 v17 对象体，将图层列表包装改成标志 1 普通对象，复核两个图层、活动索引和 Map 的完整结尾。参考档本身采用 ExtraSafe 包装，未找到原生写出的这两个普通槽样本，未执行目标软件人工验收。

### 缓存栅格名称与 SDE 表名称

RasterLayer Save 在 `0x7c73d60b..0x7c73d68d` 对当前数据集的接口指针调用查询助手 `0x7c757dc0 → 0x7c727d50`，得到 `0x7c768480` 的 IDataset，再调用其虚表 `+0x10` 取得 FullName，交给 RasterIO WriteObj(30)。CacheRasterDB.dll 10.1.0.3035（SHA-256 `59e29df9f5e3f652e86bf602b209c68a069f7cb58446840bbcf94b6c29ea0420`）中 CacheRasterDataset 主 QI `0x6c0e2ac0` 对同一 IID 返回 `+0x4c`。该 IDataset 虚表 `0x6c0eedf4` 的 `+0x10` 项是 `0x6c0e42a0`，经 `0x6c0e42bc` 引用 IAT `0x6c0ee070` 导入的 CLSID_CacheRasterDatasetName 并实例化。该创建链提供 tag 30 的具体写侧来源。

MapServerLayer.dll 10.1.0.3035（SHA-256 `59957732405d0ada878e85d002cc465f271b1b25fe56fdcef97faced0f8a6d03`）的 CacheRasterDatasetName 主 QI `0x79e21ec0` 对 IName 返回 `+0x10`，对 IPersistStream 返回 `+0x18`。该虚表 `0x79ebc208` 的 GetClassID `0x79e22a00` 写出注册 CLSID `343EB024-5F47-4813-8D4C-66AA8A6D2F19`。Ghidra 补导出 Load `0x79e22a40` 和 Save `0x79e22b20`，并以 objdump 对照。体依次为版本 short、AfCore String×2、IWorkspaceName 槽、瓦片读取器槽、IEnvelope 槽。Load 在读取版本后直接读字符串，没有版本分支。当前保留 short 原值，复用 WorkspaceName 与 Envelope 的读取器；瓦片读取器的空标记、已登记回引和已确认的新对象均使用同一对象表。

瓦片槽 IID 为 `6FE312B6-1796-4AA2-BC1E-B3722BA1DCC8`。MapServerLayer.dll 上四个实现的主 QI 分别为 InternetTileReader `0x79e37470`、MapServerRESTTileReader `0x79e69920`、MapServerTileReader `0x79e732e0`、MapTileReader `0x79e748b0`，均命中该 IID。其持久化入口分别为 `0x79e38520/0x79e386f0`、`0x79e6a450/0x79e6a5d0`、`0x79e73e10/0x79e73f90` 和 `0x79e75120/0x79e755b0`。

RasterSDE8x.dll 10.1.0.3035（SHA-256 `085aa62b32a0fac60a8f02b9244c9d1940ada727603f4ca3a79b4758246c482c`）主 QI `0x7c9e2a40` 对 IName 返回 `+0x10`，对 IPersistStream 返回 `+0x18`。SdeRasterTableName 的虚表 `0x7c9e8928` 由 GetClassID `0x7c9e2e20` 写出 `942AC1C0-FEC4-11D3-8D6F-00C04F5B87B2`。Load `0x7c9e2e70`、Save `0x7c9e2e60` 均直接返回 `E_FAIL`，没有可实现的独立持久化布局。该类与已支持的 SdeRasterDatasetName 是不同 CLSID。

构造 tag 30 覆盖无键 CacheRasterDatasetName、新 WorkspaceName 和 Envelope、瓦片读者回引和空标记，并核对后续 short、未知瓦片对象和截断回滚。66 个本地工程路径的两个流共 132 个流中未命中 CacheRasterDatasetName 或 SdeRasterTableName 的 CLSID 候选。尚无真实 tag 30 宿主字节与 ArcMap/QGIS 验收。

#### 缓存瓦片读取器

补导出上述四组 Load/Save 及共用配置读取助手 `0x79e2ef10`。普通槽使用宿主 ObjectStream 对象表，内联配置没有外层 CLSID、对象键或长度。在名字体全部读完后才提交游标和对象表。

| 读取器与 CLSID | 10.1 读取顺序 |
| --- | --- |
| InternetTileReader `034DA201-5CD6-474A-BFA8-55BA06EB3C4D` | short ≤1、AfCore String×2、byte、int32×3、内联配置、提供器槽 |
| MapServerRESTTileReader `27D88560-868E-4869-BF54-7EECA522594E` | short ≤1、byte、IColor 槽、内联配置、AGSServerObjectName 接口槽、short 选择子 |
| MapServerTileReader `DF008379-1C82-4E42-B3A9-F12A5648D7F0` | short ≤2、byte、IColor 槽、内联配置、AGSServerObjectName 接口槽，v2 追加 short 选择子 |
| MapTileReader `48B2EC16-B0DB-4D85-9A86-1DDB04048579` | short ≤2、AfCore String×2、short×2、路径 String、IName 槽、short 选择子，v2 追加配置三槽 |

内联配置为 AfCore String 和 IEnvelope、ITileCacheInfo、ITileImageInfo 三个普通槽。MapTileReader 的 v2 已先在体内读取字符串字段，尾部直接读三槽。两个服务类遇非零选择子时，`0x79e6ab10` 创建 MapServerREST（CLSID `D9EE02B7-5C71-4CAC-96AA-C26AD097E90F`），以 IPersistStream 的 `+0x14` 调用 Load，流内无第二个对象头。主 QI `0x79e53420` 在 `0x79e53581` 匹配 IPersistStream，返回 `this+0x2c`，对应虚表 `0x79ec1a04` 的 Load `0x79e50eb0` 和 Save `0x79e50ba0`。提供器和 AGSServerObjectName 槽支持已确认类的新对象体，以及空标记和回引。

MapServerREST 先读版本 short（≤5）和标志 short，再读一个 AfCore String 及三组「AfCore String、JSON」。v2 追加一个 String，v3 追加三个 JSON，v4 追加普通 IName 槽，v5 追加普通 IPropertySet 槽（IID `F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`）。本地源码 MapLib.dll 10.1.0.3035（SHA-256 `c5f2e3dc3aef8426f0557d92b7a5b5dfbe897e1a7c83c8322d23c364aa87849f`）的 JSONSerializerCarto::LoadJSON `0x79840400`、SaveJSON `0x79840290` 显示 JSON 以 32 位字节数定界，零表示空。Rust 检查长度及读取范围并保留 JSON 原字节，IName 和 IPropertySet 复用已支持的对象分派。

CartoXProvider.dll 10.1.0.3035（PE32，SHA-256 `1c4a6d3681e7527ca21e1b9afeb3b5a83922ad12f6c8a0e5c19dec33dd6f3cf5`）中，经类注册和 IPersistStream 虚表配对确认以下对象。以 Ghidra 导出和 objdump 核对版本门及导入函数。

| 类与 CLSID | Load / Save | 对象体 |
| --- | --- | --- |
| TileCacheInfo `01312017-7D38-4D2B-91A0-05C548ADE7F3` | `0x6ce7b180 / 0x6ce7b2e0` | short ≤3、IPoint、两个 int32 瓦片尺寸、ILODInfos。v2 起追加 ISpatialReference 和 DPI，v2 的 DPI 为 int32，v3 为 double。 |
| TileImageInfo `0CA787D5-FCAE-412D-85D8-61AA304B1AE1` | `0x6ce7a5b0 / 0x6ce7a600` | AfCore String、int32。 |
| LODInfos `8CD19A51-334B-4C0A-BC97-A3FDEA0A1CE0` | `0x6ce8d700 / 0x6ce8d080` | short ≤1、int32 计数、对应数量的 ILODInfo 普通槽。非正计数不进入循环。 |
| LODInfo `35BD0F76-3CF3-4D0C-9C4A-D728CFE593A5` | `0x6ce7b890 / 0x6ce7b920` | short ≤1、int32 层级、两个 double。 |

LODInfos 的 Load 在 `0x6ce8d78d` 通过主虚表 `0x6cee9b54` 的 `+0x2c` 分派到 `0x6ce906c0`，调用导入的 AfCore LoadObject，先以 IUnknown 接收再查询目标接口。由此确认循环中的槽使用普通对象头和键。

InternetTileReader 的提供器槽请求 IID `FC89CF39-5A9A-474B-B479-E978259A98C4`。MapServerLayer.dll 四个主 QI 命中此 IID，RTTI 与虚表对应七个类。Ghidra 补导出下列入口，以 objdump 核对版本门、分支和嵌套 IID。七类均接入。

| 提供器 | GetClassID / Load / Save |
| --- | --- |
| ArcGISOnlineProvider | `0x79e219f0 / 0x79e21a30 / 0x79e21ac0` |
| GoogleMapsLayerProvider | `0x79e25f30 / 0x79e25f80 / 0x79e26040` |
| IMSLayerProvider | `0x79e28540 / 0x79e28580 / 0x79e287e0` |
| MSVirtualEarthLayerProvider | `0x79e76c90 / 0x79e76cd0 / 0x79e76e20` |
| OpenStreetMapProvider | `0x79e7a0a0 / 0x79e7a0e0 / 0x79e7a1c0` |
| WMSLayerProvider | `0x79e80690 / 0x79e806d0 / 0x79e80820` |
| WMTSLayerProvider | `0x79e83ac0 / 0x79e83b00 / 0x79e83e70` |

多个提供器请求 `A8ED7FA5-572B-4668-A6CC-F3EC9E6E06A8`。CartoXProvider.dll 的 CacheInfo 主 QI `0x6ce79ae0` 命中该 IID。其 GetClassID、Load、Save 分别为 `0x6ce79ef0`、`0x6ce79f30`、`0x6ce79fc0`。CacheInfo 与已接入的 TileCacheInfo 各有独立虚表。

CacheInfo（CLSID `262E345E-B6CB-41CD-9F88-5453741E63F4`）无版本字段，依次读 ITileCacheInfo 和 ITileImageInfo。ArcGISOnlineProvider v1 直接在两个 String 后读取 ICacheInfo 普通槽；GoogleMapsLayerProvider v≤1、OpenStreetMapProvider v1–2 和 MSVirtualEarthLayerProvider v≤4 的共有字段为 String×3、short、String、ICacheInfo。OpenStreetMapProvider v2 多读一个 String，MSVirtualEarthLayerProvider v2、v3、v4 各追加一个 String。IMSLayerProvider v≤2 与 WMSLayerProvider v1 直接读取 ITileCacheInfo 和 ITileImageInfo，随后分别读取 5 或 4 个 String、2 或 1 个 IEnvelope、IColor 及各自 String 与 short 尾段。IMS v2 对尾部的最后一个字符串尝试解密，Rust 保留流内字符串。

WMTSLayerProvider v≤1 以 short、ICacheInfo、八个 String、ISpatialReference、IEnvelope 开头，然后读取三个 u32 计数集合，元素依次为 String 对、四个 double 的 WKSEnvelope 和单个 String。最后读取 String、short 和 IWMTSLayer 槽。WMTSLayer 的 CLSID 为 `61C743A1-8317-416A-8317-10964DADC6AD`，主 QI `0x79e82030` 将 IPersistStream 映射到 `+0x14`，Load/Save 为 `0x79e82190 / 0x79e82270`。构造 `0x79e81de0` 创建 InternetTiledLayer 并在内部保存其 IPersistStream，因此外层版本 short 后，直接委派 InternetTiledLayer Load/Save `0x79e31a90 / 0x79e31790`，然后读取提供器普通槽。内联 InternetTiledLayer v0–2 同样可以含提供器槽，解析复用对象表并限制首次定义的递归深度。旧式扩展沿用已确认体的读取路径，未知非空扩展仍返回明确错误。

### 瓦片图层的独立 ILayer 宿主

复用 x86 `MapServerLayer.dll` `10.1.0.3035`（SHA-256 `59957732405d0ada878e85d002cc465f271b1b25fe56fdcef97faced0f8a6d03`）的已有持久化体，并补查普通 ILayer 接口。InternetTiledLayer 的 GetClassID `0x79e31730` 写出 `{A4BADC1B-EBED-4A29-99DC-C6334DE352AD}`，IPersistStream 虚表 `0x79ebe51c`。主 QI `0x79e30790` 在 `0x79e307ff` 请求常量处匹配 ILayer，返回完整对象 `+0x34`。其虚表 `0x79ebe4ac` 的 `+0x0c` getter `0x79e30c50` 对完整对象 `+0x94` 的 String 调 MakeBSTR。Load 在 `0x79e31add` 读取的第一串正好落在该地址（`IPersistStream+0x58`）。

WMTSLayer GetClassID `0x79e82370` 对位独立类标识，构造指令 `0x79e81e58..0x79e81ea2` 把 InternetTiledLayer 聚合接口存入 `+0x20`，`0x79e82123..0x79e82135` 向该接口转发 ILayer 查询。两个主查询路径都没有匹配 ILayerMasking。InternetTiledLayer 的后备 `0x79eaa6e0 → 0x79ea9f30` 检查基础接口后在 `0x79eaa022` 返回 E_NOINTERFACE。WMTSLayer 的转发由同一聚合对象处理。因此读完两类瓦片图层后，Map 不消费逐图层遮罩字节。

两类对象现在可从 Map 图层列表、组图层和三维容器的关联 ILayer 入口读取。ExtraSafe 条目要求对象体在声明范围内精确读完。组图层中存在非零同步量 B 时按 B 定界。构造测试覆盖普通及有界条目、Scene 关联 WMTS 图层、GroupLayer 同步边界、有界截断和内部提供器键回滚。两套介质的 134 个 `/Maps` 流及参考 MXD 未命中这两个类标识。ArcMap/QGIS 人工验收尚未进行，瓦片图层的转换仍待实现。

AGSServerObjectName（CLSID `6C22971C-D450-4B4D-9422-EE96A40FACC5`）取证来自 AGSClient.dll 10.1.0.3035（SHA-256 `3bebe94f58b3aed1dca1294aecaf8e080f235a92b9c6aa51376e16ae7cbe351c`）。主 QI `0x697e8480` 对 IPersistStream 返回 `+0x24`，虚表 `0x698d7a30` 的 GetClassID、Load、Save 分别是 `0x697e9880`、`0x697e9b30`、`0x697e98b0`。对象体为 short 版本（≤3）、三个 AfCore 自由函数 ReadString 的 BSTR 前缀字符串、IAGSServerConnectionName 普通槽，v2 加两个 BSTR 字符串，v3 加第三个。其连接名 CLSID 为 `CBA35C3F-EDEF-408F-8B51-510784C78EB9`，Load/Save 为 `0x697e6070 / 0x697e6170`，读取 short 版本（≤2）、IPropertySet，v2 起追加一个 AfCore 成员函数 String::Read 字段。Rust 按各自字符串编码读取，并复用 PropertySet 对象表。

构造用例覆盖四类读取器的版本分支、配置中 LODInfo 首次定义与回引、TileCacheInfo v1–v3 及 TileImageInfo；内联 MapServerREST 测试覆盖 v1–v5、非空 JSON、PropertySet 新键与截断。栅格尾段测试从 RasterIO tag 30 继续读到新读取器后的 Envelope 和 tag 35–37。参考工程命令行解析输出与上一轮一致。另覆盖七类提供器、WMTS 集合和同键循环、AGS 连接名及 PropertySet 新键和截断。当前无真实新瓦片读取器样本，尚未在 ArcMap 或 QGIS 中人工验收。

### 空名称与有效名称数量

同版 RasterLayer.dll 的 `0x7c73b445..0x7c73b461` 只将非空 IName
交给原生数组追加助手 `0x7c75e670`。空槽仍消费它的版本化后缀字节。
循环后 `0x7c73b6eb..0x7c73b701` 取得数组首元素；随后
`0x7c73b71d` 对数组中的实际名称数区分单项与多项路径。
据此，图层模型在完整列表里恰有一条非空名称时仍提取该数据源，
即使它前后有空槽。多条非空名称依然保留在解析结果中。多项原生分支
也打开首项，但另走 `0x7c73b9c6` 开始的逻辑，尚不能据此推定首项
足以表示整个图层的数据源。

使用参考工程的已读栅格对象构造首尾空槽变体，验证模型仍给出同一
数据源。再加入第二条非空名称，验证保留图层及多名称解析结果。
未在 ArcMap 中人工打开这些变体。

### 多名称栅格的波段追加

复用本篇同 SHA-256 的 RasterLayer.dll。多项路径先对数组首项执行
IName::Open（`0x7c73b9f8`），用 `0x7c74bb40` 对打开结果请求
IID `31E6C041-EC46-11D1-8D21-0000F8780535`。成功时执行初始化，
并以 `0x7c7145c0` 查询 `7ACA3D10-0629-11D2-8D24-0000F8780535`。
10.1 esriDataSourcesRaster 类型库确认后者是 IRasterBandCollection，
其 x86 虚表 `+0x14` 为 Clear，`+0x30` 为 AppendBand。
指令 `0x7c73bb41` 清空集合，`0x7c73bb4d` 追加首个波段。
随后 `0x7c73bb60..0x7c73bbca` 按原始顺序逐项调用剩余名称的 Open，
对每个成功取得波段接口的结果调用 AppendBand（`0x7c73bbbe`）。

因此模型中单一 DataSource 无法完整表达这条组合路径。解析保留各名称
条目并报告原因，构造变体检查多名称诊断。集合成员在 ArcMap 中实际
打开的结果及 QGIS 表达仍未验证。

### 组合栅格的模型来源

依据上一节的数组遍历和 AppendBand 顺序，完整多名称列表的非空条目
按流序进入模型 `raster_band_sources`。每项保留外层类标识和对象键；
已确认的直接数据集名称、整体名称回引以及新波段名称内的已读数据集名
可提供路径和名称。缺少来源字段的项仍占对应位置，空标记跳过。
这组条目描述保存的输入，实际能追加哪些波段取决于 ArcMap 打开结果。

构造回归覆盖首尾空槽、已知来源、未恢复来源和条目顺序。波段选择器、
其他名称体的来源提取及 QGIS 中的波段组合仍待完成。ArcMap 和 QGIS
人工验收尚未进行。

### 数据集内波段名称

复核同一 RasterDB.dll 的 RasterBandName IName 虚表 `0x7c131038`，
Open 为 `0x7c053720`。非空 IDatasetName 分支先打开父数据集，
经 `0x7bfec230 → 0x7bfec290` 请求 IRasterBandCollection
（IID `0x7c124d9c`）。`0x7c053803..0x7c053822` 从 IName 接口
`+0x18` 的字符串取最后一个反斜杠后的内容，若无反斜杠则使用整串。
该地址与 Load `0x7c053a5e` 的首字符串存储一致。随后
`0x7c053842` 调集合虚表 `+0x20`，同版类型库确认该槽为 BandByName。

Rust 保留原始名称，并在组合栅格模型中记录传给 BandByName 的名称。
构造测试覆盖带反斜杠前缀的名称，并区分直接工作空间分支。
数值波段索引仍需由实际数据集的 BandByName 结果确定，ArcMap/QGIS
人工验收未进行。

### 波段名称对象的回引

数据集分支的 RasterBandName 完整读取后，按实际类标识和对象键保存
BandByName 参数及已确认的父数据集来源。父对象的新定义和回引共用
当前 ObjectStream 的数据集缓存。RasterLayer 可在引用整个波段名称时
恢复同一组字段，继续保留原始回引条目。合成用例覆盖父数据集回引、
外层波段名称回引、类标识不匹配和结束位置。

直接工作空间分支及 ArcMap/QGIS 人工验收仍待核对。

### 波段名称的直接工作空间分支

同版 RasterDB.dll 的 RasterBandName::Load 在 `0x7c053abf` 把零选择子
的 IWorkspaceName 槽读入持久化接口 `+0x34`，与 IName 视图的
`+0x3c` 对应。Open 在 `0x7c05374e` 读取该成员，经
`0x7bfec340` 查询 IName 并于 `0x7c05378c` 打开。
`0x7c0537c3` 尝试取得栅格接口，成功后进入同一 BandByName 路径。
只有工作空间名时，将路径原文和波段查找名称分别保留，数据集名字段
保留为空。匹配的 WorkspaceName 回引沿用同一键表。

构造测试覆盖直接工作空间的非空路径与存盘波段名称。实际工作空间
是否能打开为所需接口、波段索引和 ArcMap/QGIS 人工验收仍待验证。

### 波段集合的索引选择

复核同版 RasterDB.dll 的 RasterBandCollectionName IName 虚表
`0x7c130ee8`，Open 为 `0x7c052890`。该函数在 `0x7c0528d1`
打开内含名称，有索引数组时查询 IRasterBandCollection。
集合计数和 LongArray 数量相同，且数组内容逐项等于从 0 开始的序号时，
可走原集合路径（`0x7c052932..0x7c052967`）。其他情况在
`0x7c052999` 取出每个 LONG，直接传给集合虚表 `+0x28` 的 Item
（`0x7c0529ac`），再通过 `+0x30` 的 AppendBand 追加非空结果。
没有索引数组时走 `0x7c052a1c` 的原打开结果路径。

模型保留保存顺序和索引原值，包括仅有一个 RasterBandCollectionName
的图层。构造模型回归覆盖重排及重复索引，并验证已读父数据集名称和
工作空间路径同时保留。数组对象回引及 ArcMap/QGIS 人工验收仍待验证。

### 波段索引数组的回引

LongArray 经栅格名称槽或 PropertySet 对象流首次完整读取后，
沿宿主 ObjectStream 键表保存版本和数组。后继波段集合槽校验
LongArray CLSID 和已登记键，再恢复索引；原始回引对象头仍保留。
合成测试验证带重复值的非单调序列、回引结束位置及类标识不匹配。
整个波段集合名称对象的回引、真实工程和目标软件人工验收仍待验证。

### 整个波段集合名称的回引

读取器按实际类标识与对象键缓存波段集合索引和已确认的直接父数据集
来源。RasterLayer 遇到整个集合名称的回引时进入同一模型路径，包括
唯一非空名称就是集合回引的情况。嵌套组合仍保留在原始解析对象树，
不会把外层索引解释为根数据集的索引。合成测试覆盖父名称回引、
集合名称回引、重排和重复索引以及读取结束位置。目标软件人工验收未做。

### v5–v8 的内联波段索引

复核同一 RasterLayer.dll 10.1.0.3035 的名称列表 Load
`0x7c73b380`。旧版选择子非零时，`0x7c73b522` 调列表首项的
IName::Open，`0x7c73b529` 经 `0x7c7145c0` 查询
IRasterBandCollection，随后 `0x7c73b546–0x7c73b54e`
把流中第二个 LONG 直接传给集合 +0x28 的 Item。
`0x7c73b555` 查询所得波段的 IDataset，`0x7c73b577` 取 FullName，
再清空名称数组并追加该名称。能否运行到替换位置取决于数据源打开结果。

模型保留原始容器来源和请求的波段索引，避免把旧单波段请求当成
整个栅格数据集。构造回归覆盖 v8 非零选择子及已知父数据源。
ArcMap 10.1 中实际打开的波段及 QGIS 目视验证待补。

### 仅含一条波段名称的图层

同版 RasterLayer.dll 的单项分支在 `0x7c73b752` 调用 IName::Open，
经 `0x7c74b3f0` 查询 IRasterDataset（IID 常量 `0x7c769370`）。
未取得该接口时，后续路径可在 `0x7c73b88e` 经 `0x7c74bb40`
查询 IRasterBand。10.1 类型库同时声明 RasterBand 实现
IRasterDataset 和 IRasterBand，因此进入单项分支也可能仍是波段来源。
模型对单个 RasterBandName 的新对象和回引均保留查找名称及可用的父
数据源。构造回归验证回引的波段名与父数据源。ArcMap/QGIS 目视待补。

### NetCDF 变量名与数据源回引

NetCDFDB.dll 10.1.0.3035 的 NetCDFRasterDatasetName 主 QI
`0x7ae28bf0` 对 IMDRasterDatasetView（IID 常量 `0x7ae617a8`）
返回完整对象 +0x24。类型库的 Variable getter 位于接口表
`0x7ae61654` 的 +0x24，入口 `0x7ae29a80` 读接口相对 +0x60，
即完整对象 +0x84。持久化接口位于完整对象 +0x18，
Load `0x7ae29e30` 在 `0x7ae29eca` 读取第五条 String 至
持久化接口 +0x6c，对应同一成员。模块摘要见本篇基础栅格类小节。

共享名称读取器在成功读取带键 NetCDF 名称后登记变量名及可用的
工作空间路径，RasterLayer 的新对象与同键回引都能取得这些值。
构造测试覆盖名称 v1/v2、不同的首串与变量名、非空工作空间、回引
及名称体截断回滚。维度选择仍由原始名称体保留，ArcMap 10.1 和
QGIS 中的实际数据源打开结果尚未人工验证。

NetCDFRasterDataset 的主 QI `0x7ae24f00` 在 `0x7ae24fad` 比较
`0x7ae613d4` 的 IRasterBandCollection IID，匹配时返回完整对象
+0x10 的虚表 `0x7ae60f7c`。因此 RasterBandCollectionName 的
Item 路径也能使用 NetCDF 父数据集。读取器在集合键下保留该父名称
的已确认来源与索引数组。构造回归覆盖父名称回引后再次回引集合，
并核对重复索引、顺序和后继边界。ArcMap/QGIS 实档目视尚未进行。

### 镶嵌数据集名称与回引

同版 MosaicDB.dll 的 Load 在 `0x7a3f5b83/0x7a3f5b91` 依次
读取两个 String 至完整对象 +0x28/+0x2c。名称接口入口
`0x7a5a0160` 经 IAT 委派给 RasterCoreLib.dll 的
BasicRasterName::get_Name（`0x7bed1100`），以 +0x14 的接口
视图调用，`0x7bed111c` 取首串对应的 +0x28 成员。另一接口的
`0x7a5a015a` 委派 get_NameString（`0x7bed10a0`），以 +0x10
视图调用并访问 +0x2c。解析模型保留原始数据集名和可用的工作空间
路径，同键回引可恢复这两个值。构造测试覆盖 v1–v4、非空工作空间、
回引和截断回滚。目标软件实际打开结果尚未人工验证。

### WCS 覆盖标识回引

同版 OGCClient.dll 的 WCSCoverageName Load 在 `0x7b424c68`
把读取的字符串写入完整对象 +0x24。IName 虚表 `0x7b450254`
的 `0x7b4248e0` 经 +0x10 的接口视图调用 `0x7b424800`，
读取同一字符串成员。带键名称现在登记该覆盖标识，供新对象和回引
恢复。连接属性仍由名称体保留。构造名称列表验证两条记录与后继边界，
尚无实际 MXD 和 ArcMap/QGIS 人工验证。

### 缓存栅格名称来源回引

同版 MapServerLayer.dll 的 CacheRasterDatasetName Load 在
`0x79e22a82/0x79e22a90` 把两个字符串读至完整对象 +0x28/+0x2c。
对象 +0x14 的虚表 `0x79ebc1d8` 通过 `0x79eacb92` 和
IAT `0x79ebb634` 调用 BasicRasterName::get_Name。其读取的接口
+0x14 对应首串的 +0x28 成员。带键名称现在登记该原始数据集名称和
可用的工作空间路径。构造名称列表验证新对象、同键回引和后继边界。
ArcMap/QGIS 实际打开和目视核验尚未进行。

### 瓦片与 AIS 服务名称回引

上述两个 10.1 原生 IName::NameString 访问器分别对位
InternetTiledLayerName 的单条字符串与 AISImageServiceName 的首串。
共享名称读取器将这两个值登记为带键的来源名称，RasterLayer 的
首次定义与回引均可取回。瓦片读取器和服务的其余字符串仍由原始
名称体保留。构造 v17 名称列表覆盖两种定义、两次回引、后继渲染器
及结束位置。ArcMap 10.1 和 QGIS 人工验证尚未进行。

### XmlPropertySet 扩展候选

Desktop 将 `99F50761-C137-11D2-9F83-00C04F8ED211` 注册为
XmlPropertySet。GdbCore.dll 10.1.0.3035（SHA-256
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`）
的 RTTI 与 GetClassID `0x70a2a320` 对位。主 QI `0x70a2a5d0`
对 IPropertySet 返回对象 +0x14，对 IPersistStreamInit
（`0x70a889cc`）和 IPersist 返回 +0x10。主入口及
`0x70a60800` 回退均未匹配 IPersistStream 或 IPersistVariant，
回退在 `0x70a608f2` 返回 E_NOINTERFACE。

复核同版 AfCore.dll。对象持久化查询助手
`0x68a7c320` 和 `0x68a7c390` 使用 IPersistVariant 和
IPersistStream 的 IID（`0x68c6dc88/0x68c6dc78`）。因此，仅凭
XmlPropertySet 的 IPropertySet 和 IPersistStreamInit，不能把它作为
旧 LayerLib IPersistStream 扩展或普通 ExtraSafe 对象接入。继续保留
未知体的边界。未在 ArcMap 运行验证。

### CIMMapLayerProperties 与 CIM3DLayerProperties 候选

10.1 Desktop 登记文件将 `{B5E7EBB6-01DD-4219-80F4-B9C9352E8810}`
和 `{24A16361-9259-4779-8E80-624A5F8119B3}` 分别登记为
CIMMapLayerProperties 与 CIM3DLayerProperties。CIM.dll PE32
版本资源为 10.1.0.3035，SHA-256 为
`ab6d55b8fb888e289f7303cefa91bd1903616f8b24ec2542ea31627323548f88`。

RTTI 的完整对象主虚表分别是 `0x6d0f8174`、`0x6d0e8940`，
对应 QI `0x6d034220`、`0x6cfb0b60`。两者对
IXMLSerialize 的 IID `0x6d0e5720` 返回 +0x18，分别进入
虚表 `0x6d0f8204`、`0x6d0e8a04`。主入口的 IID 比较以及
共用后备 `0x6d095040` 均无 IPersistStream
`{00000109-0000-0000-C000-000000000046}` 或 IPersistVariant
`{67A4D651-69E0-11D2-8500-0000F875B9C6}`。后备未匹配时在
`0x6d095132` 返回 E_NOINTERFACE。两张 +0x14 视图虽有
持久化形状的虚表，但 +0x14/+0x18 均指向直接返回
E_NOTIMPL 的 `0x6d01b980`。

这两个名称含 LayerProperties 的登记类无法按已核对的
二进制路径作为旧式扩展或 ExtraSafe 新对象读取。若真实输入
出现对应类标识，仍须分析具体序列化路径。此次为静态核对，
未在 ArcMap 10.1 或 QGIS 人工验证。

RasterCoreLib.dll 10.1.0.3035 的 LoadRaster（`0x7bf2edf0`）用
IName 请求读取名称。SystemCore 同版本的回引路径在 `0x7ffd0063`
按键找到实例再查询接口。共享带键栅格名称槽据此以登记类执行分派。
合成 RGB tag 18 测试覆盖五种名称的有效回引，并确认变换对象的键即使
带有名称类头也会使外层事务回滚。ArcMap 10.1 与 QGIS 人工验收未进行。

### v12 空名称列表的原生边界

复核同摘要 RasterLayer.dll 10.1.0.3035 的导出
`RasterLayer/7c73b380.c` 及 objdump 指令。名称读取器先按磁盘计数
遍历名称，随后在 `0x7c73b61f` 检查运行时列表的成员数
（完整对象 +0x140）。为空时 `0x7c73b63e` 返回 `0x8000ffff`；
主 Load 在 `0x7c73bea5..0x7c73beb4` 直接传播失败，未到
可选变换与包络的分支。因此，在新对象的名称列表保持为空时，
零名称计数不能用于推出一个成功的普通 v12 无尾布局。
普通 v12 的成功路径仍需要可证的数据源打开结果或独立宿主边界。
本次为静态核对，ArcMap 10.1 和 QGIS 人工验收未进行。

### v12 直接宿主的边界传播

普通图层分派和无界 WCSLayer 进入 `read_inline_raster_body_10_1`。
RasterLayer.dll 10.1.0.3035 的 WCSLayer Load 在底层普通对象槽后结束，
所以 WCSLayer 带 ExtraSafe 外界时，剩余区间可作为子 RasterLayer 的边界。
GlobeHeightProperties 的图层本身带 ExtraSafe 尺寸，Map 和 GroupLayer
的有界图层条目也直接传递声明区间。普通图层分派会被多个宿主共用，
其 Reader 的剩余字节可能包含其他对象与父字段。合成 WCSLayer 用例
验证同一 v12 子体在有界路径中读完，在无界路径中保留游标和引用表。
普通 v12 的数据源打开结果仍待取证，ArcMap 10.1 与 QGIS 人工验收未进行。

RasterLayer v12 尾部的 IEnvelope IID 来自 RasterLayer.dll 10.1.0.3035
的 `0x7c768fa0`。与 LoadRaster 共用的包络槽遵循 SystemCore 按键
拣取登记实例的规则。RGB tag 18 合成测试验证包络回引的类匹配及
误引变换键时的外层回滚。ArcMap 10.1 与 QGIS 人工验收未进行。

TimeLayerTrackExtension 的 ITimeWindow 槽以登记实例的类校验回引，
沿用 10.1 MapAnimation.dll `0x794f5450` 路径请求的接口及 SystemCore
按键查询规则。合成时间轨道用例验证有效回引和误引 LongArray 后的键回滚。
ArcMap 10.1 与 QGIS 人工验收未进行。

复查同版 MapAnimation.dll 的版本判断。`0x794f548a..0x794f548f`
与 `0x794f700e..0x794f7013` 先将版本减一，再按无符号 16 位值
与 1、2 比较，分别限定为 1–2 和 1–3。合成用例补验两个对象
遇到零、超限值和 `0xffff` 时，两种包装均保留入口位置及既有键。
ArcMap 10.1 与 QGIS 人工验收未进行。

MapAnimation.dll 10.1.0.3035 的 TimeWindow IPersistStream Load
`0x794f6fe0` 亦用于共享 Variant 对象体分派。合成 v3 测试覆盖
版本字段、原流后继字节以及截断时的游标和键回滚。尚缺真实 Variant
槽实档与 ArcMap 10.1、QGIS 人工验收。

MapAnimation.dll 10.1.0.3035 的 TimeLayerTrackExtension
IPersistStream Load（`0x794f5450`）以窗口循环和版本尾字段确定体边界。
共享 Variant 使用同一读取器，有界扩展包装仍检查完整消费。合成 v1/v2
验证嵌套键、后继字节及截断回滚。真实 Variant 宿主和人工验收待验证。

RasterDB.dll 10.1.0.3035 的栅格名称读取路径请求 IWorkspaceName
（IID 常量 `0x7c131138`）。共享栅格工作空间槽依据 SystemCore
同版 `0x7ffd0063` 的查键行为，以登记类校验回引。合成测试覆盖
恢复工作空间路径，以及不适用的登记类在槽起点停止并保留既有键。
ArcMap 10.1 与 QGIS 人工验收未进行。

RasterLayer v17 的 IStatsHistogram 列表按 SystemCore 10.1.0.3035
查键路径（`0x7ffd0063`）以登记对象的类分派回引。合成图层验证
有效回引及误引像元过滤器时图层尾段回滚。实档及人工验收待验证。

### Java SDK 的两份 RasterLayer v12 实档

10.1 Java SDK 的 conservationGDB 和 conservationMDB 均为 /Version
9.2 / short 14。文件 SHA-256 依次为
`2f0c5a0d9cb522765a0813e44cc80c995cf3c7826543f6fac91d2e75168659ab` 与
`8b164ca283f62403df936d806bf423506c772b519a44ad27e51bbb3b4e94868f`。
两份工程的第十条图层分别起于 /Maps `0xbee3` 与 `0xd381`，
均带 ExtraSafe A=11408/B=0。Rust 从 Map 入口顺序读取，两个 v12
尾段均含 IGeodataXform 与 IEnvelope 槽，并恰好耗尽有界体。
按需测试位于 `tests/terra_round8_old.rs`。无外层区间的普通 v12
及数据集打开状态仍缺证据。ArcMap/QGIS 人工验收未执行。

逐槽复核表明，两份实档均保存新的 CompositeXform（键 648、709）
与 Envelope（键 653、714）。RasterIO 记录存在，但均省略 tag 30。
这为有界 v12 提供了不依赖后置名称的非空尾槽实例。原生成功打开
哪一数据源、普通无界对象的终点以及 ArcMap/QGIS 人工验收仍待验证。

### RasterTypeName 的空标识打开路径

复核 MosaicDB.dll 10.1.0.3035（SHA-256
`424814c3eceb88871df221bb42cb420b61c79c110a1e46a0e05af744db0a39ed`）。
RTTI `0x7a670fd0` 的 +0x18 视图虚表为 `0x7a60c434`，
其 IName::Open 槽 +0x14 指向 `0x7a45ba30`。
该入口在 `0x7a45ba53` 清空输出，并通过 +0x20 视图的 +0x10
槽调用 `0x7a45c3e0` 取 Name。当 MosaicDatasetName 成员为空时，
getter 在 `0x7a45c4fd` 读 +0x20 视图相对 +0x10 的字符串，
即 Load 保存的第二条串。为空时在 `0x7a45c51d` 返回
`0x8000000a`，调用者的暂存字符串保持初始空值。
Open 在 `0x7a45ba83` 跳到 `0x7a45c1da`，空镶嵌名称分支
直接到 `0x7a45c35f` 清理并在 `0x7a45c375` 返回 E_FAIL，
未填充打开结果。

这为静态判断一类名称打开失败提供了条件。还需结合 RasterLayer
的名称列表、已有成员状态和 RasterIO 分支检查无界 v12 终点。
两份 conservation 实档使用普通 RasterDatasetName，不能验证
本分支，也未进行 ArcMap/QGIS 人工验收。

### 无界 v12 的初始数据集成员

复核同摘要 RasterLayer.dll 10.1.0.3035 的构造路径
`0x7c7367c0`。构造器在 `0x7c7367f2` 清零 EDI，
保留该值至 `0x7c736bd6`，将完整对象 +0x14c 置零。
`0x7c736d1f` 再清零 EDI，`0x7c736da9` 将 +0x250 置零。
二者分别是 v12 尾槽条件读取的数据集和 RasterIO 二次打开所需的
已有栅格成员。这个初始状态适用于新对象。

主 Load 在 `0x7c73c027` 检查持久化视图 +0xb0
（完整对象 +0x14c）。为零时跳到 `0x7c73c153`，
不进入依赖已有数据集的范围更新链。由此前的单名称失败路径、
构造初始值与本处跳转，可排除该范围更新链在初次打开失败后的影响。
即使已知名称不会打开，判断普通 v12 尾段前仍需核对渲染器和扩展
子对象的间接调用及 RasterIO 路径。尚无该组合的原生运行证据，
ArcMap/QGIS 人工验收未执行。

### 无界 v12 的空渲染器、空扩展与 RasterIO 门

同版 RasterLayer.dll 的主 Load 在 `0x7c73bee4` 将渲染器
LoadObject 的输出槽定在持久化视图 +0xb4，与 v12 末段读取条件
使用的 +0xb0 分开。空对象头没有渲染器体可执行。
v12 在 `0x7c73bfa9` 调 ExtraSafe 扩展集合加载 `0x7c73ded0`，
后者在 `0x7c73df64..0x7c73df6a` 对非正计数跳过对象循环。

`0x7c73c3eb` 以 magic `0xff00ff` 调 RasterIO::InitCheck。
返回零时，`0x7c73c3f2` 直接跳到 `0x7c73c668`，
跳过整个 tag 30 读取与打开路径。即使有 RasterIO 记录，
`0x7c73c408` 的 tag 30 读取之后若暂存名称为空，
`0x7c73c41d` 也会跳过该名称的 Open。
需继续核对空相关对象槽与后续例程能否改变数据集成员，
并验证在这些条件下的完整宿主边界。ArcMap/QGIS 人工验收未执行。

### 无界 v12 的已知打开失败组合

RasterIO 未命中时，Load 从 `0x7c73c668` 到
`0x7c73c6fa` 只读取 +0x128 的 AfCore String 和
+0x12c、+0x134 的两个 double。v12 跳过 v10 专属 short。
这条路径到 `0x7c73c6ff` 前不写 +0xb0。

普通槽读取器据此接入一个可静态判定的组合。新对象只有一个
内联 RasterTypeName，其 Name 和镶嵌数据集槽均空，且渲染器、
扩展、关联项和旧包络均空，RasterIO 不存在。按上述构造与读取链，
两个可选末槽省略。构造测试覆盖后继字节和逐字节截断回滚。
其他依赖运行时打开结果的普通 v12 仍需取证，真实宿主及 ArcMap/QGIS
人工验收未执行。

普通图层列表的构造回归使用标志 1 装载这个无界 v12 对象，
下一条记录回引同一图层键，随后读取活动层索引。测试核对图层键、
名称键和父列表的准确终点，也确认父对象后续字节留在流中。

同版 RasterLayer Load 在 RasterIO 名称输出为空时转到 `0x7c73c5ac`。
该段通过 `0x7c71ea30` 查询渲染器辅助接口。渲染器为空时，助手在
`0x7c71ea41` 跳过接口调用，保持输出为空，后续 `0x7c73c5e2`
和 `0x7c73c620` 均跳过虚调用。其余记录在
`0x7c73c5c8 / 0x7c73c60a / 0x7c73c648` 读入 +0xf8 至 +0xfc
的三个 short，随后结束 RasterIO。该路径保持数据集成员为空。

已知打开失败的无界 v12 因此也支持无 tag 31、且 tag 30 省略或
显式为空的 RasterIO 块。构造测试核对两种形式、三个 short 和
后继边界。真实无界宿主和 ArcMap/QGIS 人工验收仍待验证。

当 tag 30 保存第二个 RasterTypeName 时，空 tag 31 字符串使
`0x7c73c523` 跳过名称调整助手。在 `0x7c73c574` 打开该名称时，
若其 Name 字符串和镶嵌数据集槽均空，仍命中已核实的空输出路径。
`0x7c73c57b` 据此跳过二次安装。Rust 共用第一次打开的判定条件，
构造测试验证两次打开都为空时的三个 RasterIO short 和后续边界。
真实无界宿主及 ArcMap/QGIS 人工验收仍待验证。

名称列表中的空槽不进入原生有效名称数组。无界 v12 的已知打开
失败条件按非空名称数判断，并保留磁盘上的空槽和各自后缀。
构造测试在唯一 RasterTypeName 前后各加一条空槽，验证图层
体终点及名称键。该组合的真实宿主与原生运行仍待验证。

多个有效名称时，RasterLayer.dll 10.1.0.3035 仍先取数组首项。
`0x7c73b9f8` 调用其 Open，输出为空时经 `0x7c73ba04` 到
`0x7c73ba2b`，再跳往单名称共用的 `0x7c73b78d` 清理分支。
后者在 `0x7c73b7a0` 返回 S_FALSE。后续名称的 Open 循环位于
成功取得首个波段接口的路径，首项输出为空时不会到达该循环。

因此在其余尾段条件相同的无界 v12 中，可按首个有效名称判断
已知打开失败。构造测试对调两条名称的顺序，验证首项为可判定失败
的 RasterTypeName 时读至准确边界，而首项打开结果未知时保持回滚。
真实无界宿主及 ArcMap/QGIS 人工验收仍待验证。

普通 IName 回引沿用 SystemCore.dll `0x7ffd0063` 按对象键取回原
实例的路径。共享栅格名称读取器在完整读出已知空打开结果的名称后，
随事务保存其对象键。后续无界 v12 的首项可为该键的回引，仍按此前的
空结果证据判断。构造测试覆盖两次读取共用引用表、第二次不带名称体
及尾段边界。真实无界回引宿主和原生运行仍待核对。
