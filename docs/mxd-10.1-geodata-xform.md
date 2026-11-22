# ArcMap 10.1 IGeodataXform 持久化对象体

RasterLayer v13 与 ImageServerLayer v20 的请求 IID `{5E6CF525-4E74-419B-946F-7DA9BFDF815D}`
在 10.1 esriGeoDatabase 类型库中为 IGeodataXform。前者通过普通 LoadObject，
后者通过 Variant 对象分派；两者都以具体 CLSID 决定对象体读取器。

仓外现有导出中未找到 RasterDB.dll，使用 10.1 安装介质的 PE32 x86
RasterDB.dll 10.1.0.3035（ImageBase `0x7bfb0000`、SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`），
以 objdump 和本机 pyghidra 补充。主 QI 分别核对三个类，并确认 IPersist/IPersistStream
的 IID `0000010C/00000109` 走对象 +0x2c；这里与 XML 虚表不同。下面地址均为该映像 VA。

| 类 | 类型库/注册 CLSID | QI、IPersistStream 虚表 | GetClassID | Load / Save |
| --- | --- | --- | --- | --- |
| IdentityXform | `C4709A2E-299E-4609-9904-6C595319B30F` | `0x7bff1290` / `0x7c129a7c` | `0x7bff16d0` | `0x7bff1630` / `0x7bff15b0` |
| PolynomialXform | `B3CAA849-5523-4490-AA2D-780ACA41E1DD` | `0x7c034730` / `0x7c12eebc` | `0x7c036ff0` | `0x7c036a20` / `0x7c036680` |
| CompositeXform | `44923EBB-D988-4847-9B29-11AA8E6E132C` | `0x7bfbd4b0` / `0x7c124f94` | `0x7bfbd220` | `0x7bfbf580` / `0x7bfbf4a0` |

- Identity：int32 版本，≥2 时一个 ISpatialReference。Save 走 Get92SaveVersion，
  同版本使用 SaveObject；Load 对 ≤1 不读空间参考。读取器限制 0–2。
- Polynomial：int32 版本（≥3 拒绝），ISpatialReference，int32×3（后两项为
  系数数量与点数量），double×2，int32 标志×3（成员 +e4、+e8、+ec）。
  +ec 非零时先读两组二维点；+e4、+e8 非零时各读两组系数 double；
  +ec 非零时再各读两组点数量×3 的 double；版本 ≥2 最后读一条 AfCore String
  （Load 在 `0x7c036f5e` 比较版本，修正反编译将版本局部量误标为指针的结果）。
  对于 +ec 的最后两组数组，本实现按 Load 要求读取；Save 存在两个指针非零保护，
  未见实际缺写输出。超大、负数量与超界数组拒绝。
- Composite：int32 版本（>2 拒绝）、int32 数量、同一对象流的 IGeodataXform
  子槽循环；版本 ≥2 再读 short。读侧非正数量不进入循环。子变换有新对象时按上述
  三类递归分派，最大深度限制 16；回引不读取对象体。

Identity 和 Polynomial 的 Load 都请求 ISpatialReference `4434D680-…`
（`0x7c123c80`），与 Envelope 共享现有空间参考分派。普通槽整体使用同一
游标和对象表事务；Variant 宿主在提交前也会确认整个变换体。缺失、未知类或
截断不会泄漏子对象键。

参考工程 /Maps 与已抽取的 199 个 10.1 介质流文件没有这三个 CLSID 的字节命中；
正常、异常和两个图层宿主测试因此使用合成字节。覆盖多项式数组顺序、复合
子变换回引和深度、非法计数，以及任何截断时回退。未做 ArcMap/QGIS 人工验收。

## 其他五种 RasterDB 变换

沿用上表 SHA-256 对应的 10.1.0.3035 原件。用 RTTI 和内存序 CLSID 定位
IPersistStream，使用 pyghidra 补充导出并用 objdump 核对参数宽度、顺序和 QI。

| 类 | CLSID | GetClassID / IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- |
| AdjustXform | `1610EE70-8D50-41E8-B2F3-9B3002BE7BAA` | `0x7bfb1ae0` / `0x7c123b94` | `0x7bfb19d0` / `0x7bfb1910` |
| ApproximationXform | `C072EAE6-F85D-470F-ABB5-7CAD9657D16A` | `0x7bfb2b00` / `0x7c123f84` | `0x7bfb56d0` / `0x7bfb5550` |
| GCSShiftXform | `CF32BE9A-58C6-4F36-AC65-9263FCFEDE88` | `0x7bfea4e0` / `0x7c128a5c` | `0x7bfea3e0` / `0x7bfea330` |
| CoordinateXform | `A11AF4AB-9861-4E86-AC25-775668372108` | `0x7bfc41a0` / `0x7c1252fc` | `0x7bfc3f90` / `0x7bfc3e70` |
| RPCXform | `7485F1C6-398F-465A-910E-50F69C6F6F79` | `0x7c0e58d0` / `0x7c135e14` | `0x7c0e6940` / `0x7c0e6810` |

各对象以 int32 版本开始，字段按读写顺序保留。空间参考请求 IID 在
`0x7c123c80`，变换子槽在 `0x7c123ca0`。调用沿用宿主 ObjectStream，
子槽支持已知类、空引用和回引。

- Adjust v1 依次读 short、int32、ISpatialReference、IGeodataXform。
- Approximation v1 依次读 ISpatialReference、IGeodataXform、double、
  int32、两个 short、三个 double、两个 short。当前保留内部字段原值。
- GCSShift v1/v2 依次读 double、ISpatialReference。v2 起再读一个 short，
  版本门由 `0x7bfea481` 复核，10.1 写侧使用 v2。
- Coordinate v1 依次读两个 ISpatialReference、IGeoTransformationOperationSet、
  double、int32、两个 short。请求 IID `0049530E-E3EC-11D3-9F62-00C04F6BDD7F`
  在 `0x7c1253d0`。当前读取空槽及回引，见下方[操作集取证](#coordinatexform-操作集)。
- RPC v1/v2 先逐字节保留 `0x2d0` 字节参数块，再读 IGeodataXform 子槽、
  三个 double 和两个 short。v2 起委派 RasterCoreLib 的 LoadRaster，
  复用全色锐化的栅格读取器。栅格下的变换和普通子槽共用 16 层深度限额，
  截断、无体读取器或超深时由宿主回滚整棵子树的对象键。

参考工程和 66 份 10.1 介质 /Maps 流均无上述五个 CLSID 的字节命中。
合成用例覆盖多类嵌套、两级版本门、Coordinate 未知槽回退、RPC 经栅格
返回变换的递归与回引，以及 ImageServerLayer Variant 中的嵌套键。
未做 ArcMap/QGIS 人工验收。

## CoordinateXform 操作集

同一套 10.1 安装介质的 esriGeometry.olb（SHA-256
`d45fb2105c869cda034fa80101cf2d0b932e08ea6f31eca8d4322fe7f7cd3e8e`）
以 TypeInfo 37 描述 IGeoTransformationOperationSet；沿 RefTab 及接口继承
只找到 TypeInfo 113 GeoTransformationOperationSet
`{38AC4B99-A4C9-41E8-88E0-CEEA37888932}`，与注册表一致。
Geometry.dll 10.1.0.3035 的 RTTI `0x71a6b480` 与主虚表
`0x71a273e8` 对位。Ghidra 补充导出 QI `0x719cd0c0` 及
`0x719f1450`，objdump 复核所有比较常量。前者受理操作集 IID，
后者只受理 IID_IUnknown、`{DF0B3D60-548F-101B-8E65-08002B2BD119}`
与 `{9F9650F1-5F49-4041-BA0F-D10BAFF1D7BC}`，其他请求返回
`E_NOINTERFACE`。其中没有 IPersistStream 或 IPersistVariant。

Coordinate Save `0x7bfc3e70` 将操作集传给 AfCore SaveObject。
在标准 SystemCore ObjectStream 写入路径，两种持久化接口都取不到时，
`0x7ffcff1d..0x7ffcff26` 以空指针递归调用写入入口，输出 16 字节
零 GUID。Rust 保留空槽、回引与新类停止点的既有行为。此结论覆盖已核对的
内置类及标准 ObjectStream；附加 COM 类仍需各自的持久化体证据。

## SplineXform

安装介质将 `{4C777D30-C7AE-45BF-9781-19AF061A62E9}` 注册在
GdalRasterDB.dll。核对 PE32 x86 固定版本资源为 10.1.0.3035，映像基址
`0x70840000`，SHA-256 `d43b9307eb8c87210d825195722c53104def726967366f4a7f9109adfa4432c4`。
RTTI 和原始指令对位 GetClassID `0x7088da50`、QI `0x7088d370`、
IPersistStream 虚表 `0x708a1694`。Load `0x7088d900` 与 Save
`0x7088d860` 顺序读写 int32 版本、ISpatialReference、两个 IPointCollection。
Save 调 Get92SaveVersion 的版本实参为 1。两个集合请求相同 IID
`{DE08A214-4FC2-11D1-834E-0000F8775BE9}`，常量在 `0x708a17a0`，
对应 IPointCollection。末尾重建操作只接收已读对象，不再消费流。

Rust v1 复用 GeometricXform 的几何集合体读取器及宿主对象键表。合成测试验证
首个 Multipoint 新对象、第二槽回引、边界和逐字节截断回滚。参考工程和 66 份
10.1 介质 /Maps 流未检出 CLSID，尚未在 ArcMap 中运行验证。

## ProjectiveXform 的读写差异

RasterDB.dll 10.1.0.3035 的 GetClassID `0x7c03be20` 从 `0x7c12d53c` 写出
`CB7DC226-5AD2-4A40-88F4-3CEDB5D63BDE`。IPersistStream 虚表 `0x7c12f12c`
指向 Load `0x7c03bb70` 和 Save `0x7c03b9f0`。Save 在 `0x7c03ba19` 使用
`0x7c1236cc` 的版本表 `[-1,-1,-1,-1,1]`。v1 写 int32 版本、ISpatialReference、
IProjectiveTransformation2D、int32 点数、int32 标志。标志非零时接两个点数组，
各有点数指定数量的二维 WKSPoint（每个 16 字节）。点数、标志和数组分别从
相对 IPersistStream 的 `+0xa8`、`+0xb4`、`+0xac/+0xb0` 读取。

本轮复核并更正第二槽的身份。Save `0x7c03ba72..0x7c03ba83` 写的
是成员 `+0xa4`。Load 则依次读 ISpatialReference、IGeometryCollection、
IProjectiveTransformation2D。其中 `0x7c03bc20..0x7c03bc42` 把
几何集合放在 `+0x10`，第三槽读取 `0x7c03bc58` 后，经
`0x7c03c630` 对返回对象查询 IProjectiveTransformation2D 并写入
`+0xa4`。第三槽读取和该接口查询的 IID 均为
`1C8A72C4-5A43-48EF-938E-2119608D77D0`，常量分别在
`0x7c12f1d8` 和 `0x7c12957c`。

10.1 类型库列出 ProjectiveTransformation2D
`F82EB04A-4E5A-43C7-9A54-457439AA8E9E`。Geometry.dll 的
IPersistStream 虚表 `0x71a23588` 的 Load 和 Save 直接返回
`E_NOTIMPL`（`0x718ad650`）。Rust 在可由该 CLSID 明确识别的
第二或第三槽中保留已登记的回引，遇到新对象时停止。
显式槽通过已登记的类检查回引，避免把其他类的实例当作投影变换。
合成测试覆盖第二及第三槽中类不匹配时的整体回滚。
写侧两槽格式中的空投影槽可正常读取。原有读取器兼容的「第二槽为几何，
紧随点数」仍可消费，缺少原生 Save 输出佐证。若 Load 期待的第三槽为
零 GUID，开头八字节可与合法零点数和零标志重合。无独立体边界或
版本信息时保留已有分派规则，第三槽为空的旧档仍待取证。

2026-11-20 在 QEMU i386 中把 RasterDB.dll 的原始指令映射到原 VA，
执行 `0x7c03b9f0` 与 `0x7c03bb70`。驱动提供内存流、保存索引 4
及空对象读写回调。Save v1 写两槽共 44 字节，直接交给 Load 返回
`0x80004005`。在控制点字段前补一条空对象后，Load 返回 `S_OK`，
按空间参考、几何集合、投影变换三个 IID 请求对象，共消费 60 字节，
后继标记保持在偏移 60。驱动未加载 ArcMap，也未执行 COM 工厂或
原生对象流键表。临时驱动位于忽略目录 `temp/mxd-rule-objects/projective-native/`。

Rust 在有明确 ExtraSafe 边界的 RasterLayer 中先验证现有布局。若其
变换和包络后的字段无法恰好读到边界，才暂存游标及引用表试读多出的空槽。
第二种形态也必须完整读取包络及版本后缀，并恰好读到边界。
构造 v12、v13 和 v17 验证空或非空几何集合、空投影变换、后继包络的新键
以及失败回滚。普通对象流以及缺少精确结束位置的其他宿主仍缺旧档证据。另对本机参考档、提取的 Desktop 与 SDK MXD 中
221 条去重后的 Maps/PageLayout 流扫描 ProjectiveXform CLSID，未见候选。

合成测试覆盖写侧空槽、两种明确的回引位置、前后对象边界、新对象拒绝
和任意截断回滚。参考工程与两套介质共 135 份工程的 /Maps 和 /PageLayout
均读至末尾，270 条流未出现上述两个 CLSID。随后对同一批介质和
参考工程扩大到所有 CFB 流，按 ProjectiveXform 的 Windows GUID 磁盘序
`26 c2 7d cb d2 5a 40 4a 88 f4 3c ed b5 d6 3b de` 扫描。
10.1 介质 66 份共 2415 条流，10.8 介质 68 份共 2504 条流，
参考工程 1 份共 31 条流，三组均无命中。扫描范围包括其他流，
结果仍不足以验证旧档空第三槽的真实位置。ArcMap 和 QGIS 人工验收尚未进行。

## Geometric、LSR 与 Pictometry

沿用同一 RasterDB.dll 10.1.0.3035 原件。Geometric 的 Load、Save 导出均
出现 Ghidra 分析重启限制，以下版本门、调用次数和字段宽度用原始指令复核。

| 类 | CLSID | GetClassID / IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- |
| GeometricXform | `067C3B10-6091-4804-82AB-1E066F3151A6` | `0x7bfec9c0` / `0x7c12946c` | `0x7bfed530` / `0x7bfed1f0` |
| LSRXform | `15446949-7FD6-44B6-8BB2-1A66FF06837E` | `0x7c0208c0` / `0x7c12d764` | `0x7c020800` / `0x7c020730` |
| PictometryXform | `3ED1BB30-ECAB-4CF2-BE54-DE2BD8D71357` | `0x7c030be0` / `0x7c12e5c4` | `0x7c030b40` / `0x7c030ac0` |

- **Geometric v1/v2** 依次读取 int32 版本、ISpatialReference、
  IGeometryCollection（IID `C801565E-99CA-11D0-82F7-0000F8034032`）和
  1 字节选择子。几何集合复用 Polyline、Polygon、Multipoint 的对象体读取。
  选择子为零时再读 1 字节类型，0 为空，1 为六个 double，2 为八个 double。
  `0x7bfed6e9` 和 `0x7bfed769` 分别调用仿射和投影变换构造路径。
  选择子非零时通过 LoadObject 请求 ITransformation。写侧 `0x7bfee380`
  先查询 IPersistStream；对象分派接受空槽、回引和已核对的地理变换类。
  Geometry.dll 10.1 的 AffineTransformation2D 与 ProjectiveTransformation2D
  的 IPersistStream 读写槽都指向返回 `E_NOTIMPL` 的 `0x718ad650`，
  上述两个类的数值体来自 Geometric 的内联分支。独立持久化体及其余候选见
  [Geometry 变换类取证](mxd-terra-host-geo.md#geometry-类型库与派生变换)。
  版本 ≥2 最后读一个 double。
- **LSR v1** 先读 2 字节版本，再读 IPoint、连续九个 double 和两个独立
  double。IPoint 请求常量在 `0x7c1286e0`，新 Point 类复用 Geometry.dll
  的已证体，其引用键登记在宿主表。Save 表 `0x7c1237d0` 为
  `[-1,-1,-1,-1,1]`。
- **Pictometry v1/v2** 按 Save 布局读取 2 字节版本和一条 AfCore String。
  Save 表 `0x7c123768` 包含版本 1 与 2，两者都走同一字符串写入。
  Load `0x7c030b86` 先检查对象已有字符串长度，非零才读流中的字符串。
  当前依据确定的写侧格式消费字符串，原生状态门的实际运行影响尚未验证。

参考工程及 66 份 10.1 介质 /Maps 流均无上述三个 CLSID 命中。合成测试
覆盖版本门、内联与带键分支、几何体边界、空字符串及截断回滚。
未做 ArcMap/QGIS 人工验收。

### GeometricXform 的跨模块 ITransformation

`0x7bfed651` 压入的 `0x7c123c90` 是 ITransformation IID
`6F2CF924-7E00-11D0-82F4-0000F8034032`。扫描 10.1 介质 72 个可读取的类型库，
沿接口继承收集 coclass，再扫描 1115 个提取的 DLL 的 GUID 及指令交叉引用。
在 RasterDB.dll 同一 IID 的直接 QI 比较中，定位了以下 15 类（地址为主 QI）：
Identity `0x7bff1290`、Polynomial `0x7c034730`、Composite `0x7bfbd4b0`、
Adjust `0x7bfb1370`、Approximation `0x7bfb2ee0`、GCSShift `0x7bfe9850`、
Coordinate `0x7bfc0c90`、RPC `0x7c0e5bf0`、Geometric `0x7bfecc60`、
LSR `0x7c01fe30`、Pictometry `0x7c030720`、Frame `0x7bfe3330`、
Formosat2 `0x7bfde410`、SPOT5 `0x7c0fb120` 和 Projective `0x7c03b070`。
GdalRasterDB.dll 的 SplineXform（主 QI `0x7088d370`，匹配点 `0x7088d39f`）
和 Mensuration.dll 的 CSMXform（主 QI `0x7a1889f0`，匹配点 `0x7a188c6e`）
都将该 IID 映射到完整对象 `+0x24`。上述类复用各自 IGeodataXform 的已接入
二进制体与递归预算，含 GeometricXform 自身和经 LoadRaster 的交叉嵌套。

同版 DisplayCore.dll（SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`）
的 DisplayTransformation `0x6eba0210` 受理 ITransformation，但主 QI 与
`0x6ebf6ea0 → 0x6ebf6b50` 的基类链均不提供 IPersistStream 或 IPersistVariant。
CadastralFabric.dll（SHA-256 `ccaff2b3cd3379b7a7630ea6f2a41c06c75ef6f68969e8ec8f51f2f05133bfd6`）
的 CadastralPiecewiseTransformation `0x6c237b70` 也受理该 IID，基类
`0x6c26d580` 同样没有这两种持久化接口。Adjustment.dll 的
PiecewiseTransformation 见[先前取证](mxd-terra-host-geo.md#geometry-类型库与派生变换)。
这些判断限定于核对的内置实现和标准 ObjectStream。未发现的间接接口实现、
扩展模块或旧档类仍需针对实档取证。

## Frame、Formosat2 与 SPOT5

用同一 RasterDB.dll 10.1.0.3035 原件补充 pyghidra 导出，原始指令和
CLSID 常量核对对应对象。三类开头都是 **2 字节**版本，区别于常见的 int32。

| 类 | CLSID | GetClassID / IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- |
| FrameXform | `D3E347AC-80E5-4A75-B4A8-1E7AC5D50710` | `0x7bfe2f00` / `0x7c1285c4` | `0x7bfe4a10` / `0x7bfe4740` |
| Formosat2Xform | `A334516D-DE73-43BD-9CC5-1CC324705EF8` | `0x7bfddeb0` / `0x7c1281cc` | `0x7bfe0860` / `0x7bfe0450` |
| SPOT5Xform | `BBC93AE5-459D-4D82-A95F-2FA49E9AE0C7` | `0x7c0faae0` / `0x7c136634` | `0x7c0fdc90` / `0x7c0fd790` |

- Frame v1–v7 依次读取 ISpatialReference、三个 double、两个 short、
  LoadRaster、IGeodataXform、short、五个 double、两个 IPoint，
  再读 short、double、short、十个 double 和 int32。v5 起追加 double
  与 IGeodataXform，v6 起追加 int32，v7 追加两个 IDoubleArray。
  Save 表 `0x7c123900` 的有效版本为 7，旧版本门来自同版 Load。
- Formosat2 v1 依次读取十个 int32、六个 double、十四个 IDoubleArray、
  IPolynomialXform、三个 double、两个 short 和 LoadRaster。前六个
  double 的末两项顺序为成员 `+0xfc`、`+0xf4`。Save 表
  `0x7c123914` 的有效版本为 1。
- SPOT5 v1 在版本后读 AfCore String 与 short，v2 省略这两个字段。
  两版接着读三个 int32、三个 double、八个 int32、short、int32、
  三个 double，再读二十一个 IDoubleArray。最后三个 double 对应
  成员 `+0x14c`、`+0x15c`、`+0x154`。v2 追加三个 double、
  两个 short 和 LoadRaster。Save 表 `0x7c122374` 的有效版本为 2。

IDoubleArray 请求常量 `0x7c12829c` 对位 SystemCore.dll 的
[DoubleArray 持久化体](mxd-10.1-raster-renderer.md#rgb-的两个嵌套对象)。
IPolynomialXform 请求常量 `0x7c1282ec` 对位 esriDataSourcesRaster
10.1 TypeInfo 250，复用本文件的 PolynomialXform 体。IPoint 复用 LSR
的分派。所有嵌套对象沿用宿主表，LoadRaster 内再次进入 IGeodataXform
时与普通子变换共用深度限额。

参考工程及 66 份 10.1 介质 /Maps 流均未检出三类 CLSID。合成测试
覆盖各版本门、新数组与回引、Formosat2 中栅格回引已有 PolynomialXform
和逐字节截断回滚。未做 ArcMap/QGIS 人工验收。

## CSMXform

安装介质将 `{5F801A53-8D2A-4A41-B933-FEFFCF0CCAF8}` 注册在 Mensuration.dll。
PE32 x86 固定版本资源为 10.1.0.3035，映像基址 `0x7a180000`，
SHA-256 `c7d1ad745174d43c01463950f9e2337cc7adbb5d8d8b0722c905184f3cc00f1a`。
RTTI 对位 IPersistStream 虚表 `0x7a1c3f1c`，GetClassID
`0x7a1896b0` 从 `0x7a1c2974` 返回 CLSID。Load `0x7a18a1b0`、
Save `0x7a18a620` 用 pyghidra 补充导出，并由 objdump 复核。

Save 表 `0x7a1d873c` 为 `[-1,-1,1,2,2]`。两版均写 short 版本，
再写 int32 内部版本 1 和两个取值 0 或 1 的 int32。随后是两组 uint32
字节长度加对应的窄字符内容，零长度时无内容，长度不计结束零。第二组
非空的写分支在 `0x7a18a9f7`，核对长度和正文分别调用 IAT
`0x7a1c20ac` 与 `0x7a1c20a0`。本文按确定的写侧布局保留原始字节，
原生后续多字节转 BSTR 和传给模型的运行行为仍未验证。

参考工程及 66 份 10.1 介质 /Maps 流未检出该 CLSID。合成测试覆盖
v1/v2、空状态、原始高位字节，以及非法标志、长度越界和截断回退。

10.8 的登记表把 ADSXform、Transform3DXform、CameraXform 列在 RasterDB.dll 下。
这三类未出现在从 10.1 Desktop CAB 提取的 esri.clsid.ecfg 中。此前对提取的
1115 个 10.1 Desktop DLL 搜索三个 CLSID 的内存序字节亦无命中。

## 10.1 内置变换分派清点

以 esriGeoDatabase 的 IGeodataXform IID
`5E6CF525-4E74-419B-946F-7DA9BFDF815D` 为起点，扫描 72 个可读的
10.1 Desktop 类型库。按接口继承和 coclass RefTab 找到 16 个公开 coclass。
在 10.1 的 esri.clsid.ecfg 中，名字以 Xform 结尾的 17 个 CLSID
与 Rust 分派完全一致。多出的 PictometryXform 经 RasterDB 主 QI
`0x7c030720` 的 `0x7c030727` 直接返回 IGeodataXform。
提取的 PE 中搜到这个 IID 的直接比较分别归于 RasterDB、GdalRasterDB
和 Mensuration 中已确认的 17 类。其余指令交叉引用为调用方请求接口、
加载对象或辅助创建调用。已定位的 10.1 内置类均有对象体分派。
ProjectiveXform 的空第三槽已在有界 RasterLayer 接入，无界宿主仍开放。其他版本或附加组件带来的类应按其各自的原生证据确认。

补充用例将两个非空二维控制点数组放入带空第三槽的 v17
RasterLayer，验证数组各自的值、后继 Envelope 新键和图层终点。
在控制点字段或后继 Envelope 截断时，宿主游标和对象键表均回滚。
复跑上述 QEMU 原生驱动，仍得到两槽写出 44 字节、三槽读入 60
字节的结果。另在仓库、10.1 介质目录和本机共享样本目录打开 173
个可读 MXD/MXT 容器，扫描可读数据流，两个投影变换 CLSID 均未命中。
无界旧档的第三槽边界仍待真实宿主佐证。

ProjectiveXform 的几何集合槽请求 IGeometryCollection（RasterDB.dll
10.1.0.3035，`0x7c03bc20..0x7c03bc42`）。共享集合槽按 SystemCore
同版查键规则使用已登记类，并沿用已有的集合类支持范围。合成测试核对
Multipoint 回引的边界及 Point 类型不匹配时的整体回滚。普通流中空第三
槽的歧义仍待实档验证，ArcMap 10.1 与 QGIS 人工验收未进行。

### 地理变换的 GeometryBag 槽

Geometry.dll 10.1.0.3035 的 GeometryBag QI（`0x7186f650`）在
`0x7186f657` 比较 `0x71a12d34` 的 IGeometryCollection IID，
以 DLL 原始字节和 objdump 复核。GeometricXform 与 ProjectiveXform 的
该接口槽复用 GeometryBag 持久化体。合成 ProjectiveXform 覆盖控制点
字段对位与截断回滚。ArcMap 10.1 和 QGIS 人工验收未进行。

同一 Geometry.dll 中，MultiPatch 的 QI 在 `0x718ddbf9` 比较相同
IID，命中后由 `0x718ddc17` 返回对象 +0x54 的接口视图。
地理变换的 IGeometryCollection 槽据此接入已有 MultiPatch 体，
合成 ProjectiveXform 验证控制点边界和截断回滚。实档和人工验收待核对。

### SplineXform 的 MultiPatch 点集合

Geometry.dll 10.1.0.3035 的 MultiPatch QI 在 `0x718ddc2b`
引用 `0x71a13ac8` 的 IPointCollection IID，命中后在
`0x718ddc39` 转往返回接口视图的分支。该 IID 与 GdalRasterDB.dll
的 SplineXform 两槽一致。合成 SplineXform 验证 MultiPatch 新对象、
第二槽回引和截断回滚。ArcMap 10.1 和 QGIS 人工验收未进行。

RasterDB.dll 10.1.0.3035 的 ProjectiveXform Load 在
`0x7c03bcaa` 读取控制点计数，`0x7c03bcd9` 读取标志。
objdump 确认 `0x7c03bced` 在标志为零时直接跳过数组分配和读取，
因此保留该路径的原始有符号计数。标志非零且计数为负时，Rust
在分配前拒绝。合成用例验证后继边界与错误回滚；实档及人工验收待核对。
有界 RasterLayer v13 合成用例同时验证空第三槽、负计数、零标志及
后继包络键，截断时回滚整个图层。
