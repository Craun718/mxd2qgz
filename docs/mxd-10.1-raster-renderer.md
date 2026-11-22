# ArcMap 10.1 栅格渲染器

## 原生取证

目标为 x86 Desktop 10.1.0.3035。RasterLayer.dll 沿用
[栅格续查](mxd-luna-followup-display-20261118.md)已核验的安装介质映像及仓外导出。
RasterRenderer.dll 未找到既有导出，本轮从提取目录
`temp/fl-envelope/desktop/` 取 PE32 映像（ImageBase `0x7c960000`，SHA-256
`cc56825cdf91d4b5aed88d0f766ee253ad30000c3079c40a21a2e9e6c6ce1390`），
以 objdump 定位类常量和虚表，再用本机 pyghidra 补充 Load/Save 与共用函数的反编译。
各类持久化入口由 GetClassID 写出的 CLSID、RTTI COL 与 IPersistStream 槽交叉定位。
下表地址均为安装介质映像 VA。

| 类型库名（RTTI 名） | CLSID | GetClassID | IPersistStream 虚表 | Load / Save |
| --- | --- | --- | --- | --- |
| RasterColormapRenderer（ColormapRenderer） | `{3B8283FA-1BD4-4212-B385-19E77B9E0D3B}` | `0x7c961bb0` | `0x7c9b146c` | `0x7c962350` / `0x7c962500` |
| RasterRGBRenderer（TrueColorRenderer） | `{577F1870-7037-11D2-9F29-00C04F8ED1D7}` | `0x7c996e60` | `0x7c9b5f1c` | `0x7c99ad10` / `0x7c99b300` |
| SimpleRasterRenderer | `{3232D0EF-3460-4879-A1E6-366590CF3EEC}` | `0x7c996ad0` | `0x7c9b5b1c` | `0x7c9969c0` / `0x7c996a30` |
| SAIRenderer | `{11AB0050-74F5-4C3E-9EF9-873B6BD85B2C}` | `0x7c994d90` | `0x7c9b5794` | `0x7c9955b0` / `0x7c995660` |
| RasterDiscreteColorRenderer（DiscreteColorRenderer） | `{AC874573-D778-4421-B9C6-14557D8BD692}` | `0x7c9672f0` | `0x7c9b1aa4` | `0x7c967c60` / `0x7c967d70` |
| UniqueValueRenderer | `{0842B595-4F2F-11D2-9F43-00C04F8ECE3D}` | `0x7c99f900` | `0x7c9b6498` | `0x7c99f180` / `0x7c99f4d0` |
| RasterClassifyRenderer | `{CE8B2F44-A027-11D2-AAE7-00C04FA33416}` | `0x7c972450` | `0x7c9b277c` | `0x7c971ac0` / `0x7c972060` |
| RasterStretchRenderer | `{A301A3B2-74D7-11D2-9F29-00C04F8ED1D7}` | `0x7c97fc60` | `0x7c9b3edc` | `0x7c97f0d0` / `0x7c97f6b0` |

普通 RasterLayer::Load `0x7c73bd40` 在名称循环返回成功后读取
IRasterRenderer，请求 IID 位于 `0x7c769b44`，为
`{6F5322D1-3790-11D2-8D26-0000F8780535}`。名称循环按 ExtraSafe 已知边界
读取，详见[名称列表取证](mxd-10.1-raster-layer-tail.md#名称列表与渲染器边界)。

## 已接入的读取范围

- 色表：int32 版本，int32 数量 + ILegendGroup 对象循环，int32 数量 + int32
  数组，再进入共用 Load `0x7c98c3b0`（Save `0x7c98c6b0`）。共用体为 int32
  版本、RasterIO magic `0x00ff00ff`、tag 1–5 的 int32 记录（2/3 受共同
  开关控制）、tag 6 的无键对象、tag 7 的三个 double 及 RasterIO 结束记录。
  共用版本 4/5/6/7/8/9 依次增加 short/int32/int32/byte/short/IColorRamp；
  总对象体末端是色表版本 ≥2 的一个 int32、≥3 的两个 int32。色表 Save 版本表
  `0x7c9b1190` 为 `[-1,1,1,2,2,3]`。共用体读取支持版本 2–9，
  tag 6 支持空与五种已确认的 IColor 实现；ILegendGroup 和 IColorRamp 复用已有分派。
  RasterIO 对象的 16 字节 CLSID 后直接进入对象体，不登记外层 ObjectStream 键。
- RGB：版本 4–9 进入同一 magic 的 RGB 专用 RasterIO 段，tag 1/2/3 为
  红绿蓝零基波段索引；tag 4–11 的长度经写侧对位；tag 12 是无键颜色，
  tag 14–16 是三个统计直方图，tag 18 的独立对象槽见[全色锐化](#rgb-tag-18全色锐化)。
  tag 21/22 读侧从版本 5 起读取；6 起跟一个 int32（参考档和 Save 写 `-1`）；
  7 起跟一个 short 和两个 IDoubleArray，9 再增加两个 IDoubleArray。
  RasterIO::End 的 `(-1,-1)` 结束对位于数组之后，其后进入共用体。
  版本 1–3 走独立的[旧版布局](#rgb-v1v3-与共用体-v1)；参考实档为 v9。
- ImageServerLayer 的 Variant 对象槽调用同一体读取器，仅接受完整结果。未知
  类及已知但不完整的体回退到 VARTYPE 起点，不登记渲染器或其嵌套对象的键。
  普通 RasterLayer 仍有按外层范围保留的未知后缀。

SystemCore.dll 10.1.0.3035 在 `0x7ffd0063` 按键取回对象后查询
ILegendGroup。共享读取器按登记类校验。构造色表渲染器 v1 验证合法
回引后可读取共用体，误引 RGB 颜色则回滚游标和新建的渲染器键。
该组合尚无实档或 ArcMap/QGIS 人工验证。

分级、拉伸和唯一值渲染器的 ILegendItem 槽沿用同一查键规则。
构造输入验证有效的水平图例项回引保留后继标记，并拒绝误引 RGB 颜色。
尚无该组合的实档或人工验证。

RasterRenderer.dll 10.1.0.3035 的色表 Load `0x7c962350` 未设置类体
版本范围，尾部仅在 `0x7c9624ab` 和 `0x7c9624c5` 以有符号比较选择
附加字段。构造输入覆盖负版本、0、1、2、3 和最大 int32，验证读取
0、1 或 3 个尾部整数后的边界。写侧常规版本之外尚无实档或人工验证。

共用体 tag 6 通过 `0x7c983bd0 → 0x7c983db0` 查询 IColor，
RGB tag 12 通过 `0x7c99d9a0` 查询同一 IID（`0x7c9b17ac`）。
两槽复用 DisplayCore 五类颜色的已确认对象体，无键外壳不进入宿主
引用表。构造 RGB tag 12 的 HsvColor 验证后续 tag 18 和共用体边界。
非 RGB 组合尚无实档或 ArcMap/QGIS 人工验证。

### RGB 的两个嵌套对象

- `StatsHistogram {BA3027C1-49CA-4788-8B5A-3A6B387DE78C}`：
  对 RasterDB.dll 10.1.0.3035（x86，ImageBase `0x7bfb0000`，SHA-256
  `97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`）
  补充 pyghidra 反编译。GetClassID `0x7c100650` 写该 CLSID；RTTI 的
  IPersistStream 虚表 `0x7c136850` 读写槽为 `0x7c101910`、`0x7c101b30`。
  对象体读 int32 版本、6 个 double、桶数组，版本 <3 固定 256 桶，≥3
  在数组前读 int32 数量；≥2、≥4 各追加一个 double，≥5 追加带 int32
  计数的 double 数组。读侧 v5 的容器调整通过 `0x7c101550` 保留正计数。
  Load `0x7c10194a` 只拒绝有符号版本大于 5；`0x7c1019ff` 对
  低于 3 的版本使用固定 256 桶。构造 Variant 覆盖负版本和版本 0 的
  固定桶边界，以及版本 6 的回滚。负版本尚无实档或人工验证。
  参考工程三个对象均为 v5，各含 256 桶和 4 个追加值。
  AfCore 的 ObjectStream Variant 通用对象分派也复用该体读取器。
  参考工程的三个实例来自 RGB 专用槽。Variant 包装仅经构造输入检验，
  覆盖 v3 的结束位置和截断时的游标及登记键回滚，尚无该包装的实档或人工验收。
- `DoubleArray {60C06CA7-E09E-11D2-9F7B-00C04F8ECE27}`：
  类型库与 RasterRenderer.dll 请求的 `IDoubleArray {60C06CA6-…}` 对位。
  使用仓外现有 SystemCore.dll 10.1 反编译。虚表 `0x8007c22c` 的 Load
  `0x800419b0`、Save `0x80041a80` 先读写 short 和 int32 数量，再逐项
  调虚表；主表 `0x8007c1dc` 的 +0x28 槽 `0x80040520` 每项读 8 字节。
  这四个对象与 RGB 渲染器共用外层 ObjectStream 表；三个无键直方图不进表。

  SystemCore 10.1.0.3035 在 `0x7ffd0063` 先按键查找原实例，再查询
  IDoubleArray。栅格渲染器和传感器变换共用的读取器按登记类校验回引。
  Formosat2 v1 构造输入覆盖类标识不同的有效回引及误引 RasterDatasetName
  时的整体回滚。该回引尚无实档或 ArcMap/QGIS 人工验证。

### RGB v1–v3 与共用体 v1

用 objdump 复核同一 10.1 RasterRenderer.dll：Load `0x7c99ad10` 先读 int32
版本，小于 4 调 `0x7c99aae0`，后者拒绝小于 1。旧体无 RasterIO 头，按顺序为：

1. 波段索引三个 int32、0x930 字节查找表、一个 byte。
2. int32、double、两个 int32、两组各四个 double。第一组 double 加载时读
   入局部栈区，解析器照样保留。宽度和类型由 0x7c9b003c / 0x7c9b00b8 /
   0x7c9b0100 / 0x7c9b0104 的 AfStreamRead 导入签名和实参交叉核对。
3. 两个 AfCore LoadObject 请求 `IColor {20CD40B0-8F3D-11D0-8590-0800091A2A72}`
   （`0x7c99ac61` / `0x7c99ac86`；IID 在 `0x7c9b4260`）。与外层共享
   ObjectStream 键，复用颜色族读取器。旧体的颜色槽与新体的无键颜色记录独立。
4. 共用 Load `0x7c98c3b0`，随后 RGB v2 再读一个原生按非零布尔化的
   int32，v3 再增加一个 int32。
5. 共用 Load v1 进入 `0x7c98c310`，依次只读四个 int32（共用接口偏移
   +0xe4、+0x114、+0x118、+0x110），之后直接返回。共用 v0 被拒绝，
   v2–9 仍走原来的 RasterIO 路径。

合成测试分别覆盖三个版本、颜色新对象与同键回引、后继字节位置、非法版本、
未知颜色以及逐字节截断时游标和键表不提交。本轮扫描了 66 个 10.1 及 68 个
10.8 介质工程的 /Maps 流，没有 RGB CLSID 命中；参考工程只见 v9。10.1
Save 使用新布局，旧版规则来自本版 Load，未取得旧版实档或运行验证。

### RGB tag 18：全色锐化

在已核验的 RasterDB.dll 10.1.0.3035 中补充 objdump、pyghidra 取证。
PansharpeningFilter 的 IPersistStream 表 `0x7c12e284`（对象偏移 +0x14）
含 GetClassID `0x7c02c050`、Load `0x7c02c190`、Save `0x7c02c090`。
GetClassID 从 `0x7c12d46c` 写
`{B58B1271-E6FB-4B27-8417-2B8D62D654B0}`。RasterIO 的对象读取助手
`0x7bf13d90` 消费 CLSID 后直接调用 Load，不为过滤器自身读取外层键。

Save 调 `Get92SaveVersion(stream, 1)`，可写时保存 int32 版本、两次
`SaveRaster`、int32 方法类型及四个 double 权重。Load 顺序对称，
虽未检查读到的版本，本实现只接受有写侧依据的 1。权重与
`PutWeights`（`0x7c02b2b0`，接口偏移 +0x2c）写入的成员对位。
两级嵌套规则由 RasterCoreLib.dll 10.1.0.3035（ImageBase `0x7beb0000`，
SHA-256 `6d89601832b788177aac492e7fdfb0f1b012db86e2a75e6e480e6e647508eb1f`）
的 LoadRaster `0x7bf2edf0` 与 SaveRaster `0x7bf2e950` 互证：

- 先依 AfCore LoadObject 读取 IName `{4BA33331-D55F-11D1-8882-0000F877762D}`；
  空标记为 16 个零字节，此后直接返回。过滤器允许两个槽都为空的编码。
- IName 指针非空时，原生在 `0x7bf2eeb4` 调 Open，随即读取 int32 波段数量，
  中间没有检查 Open 的 HRESULT。`0x7bf2ef53` 在数量非正时跳过名称字符串循环。
  波段名称之后还有 IGeodataXform 和 IEnvelope。名称回引只省略名称体。
  解析器只读取持久化数据，不打开栅格。已接入普通 RasterDatasetName、FileGDB
  等名称与其已证 RasterIO 记录，保留波段数量原值，并复用已有变换与包络读取器。
- RasterIO 构造器 `0x7bf13ac0` 将传入的 IStream 原样存入 +0x0c；
  `0x7bf13e2e..0x7bf13e3a` 将同一指针传进过滤器的 Load。过滤器本身无键，
  其 LoadRaster 的名称、变换、包络在已接入的 MXD 宿主内仍走 AfCore
  LoadObject 的带键分支，复用宿主的 ObjectStream 表。未知类或内部前缀未闭合
  时保留 RGB 未读尾段及原表；普通 IStream 宿主和运行时数据源打开失败行为未验证。

SaveRaster 在 `0x7bf2ea16` 经 `0x7bf3e0d0 → 0x7bec4dc0` 请求 IDataset
`{2D04C042-7766-11D0-B77D-00805F7CED21}`，在 `0x7bf2ea3d` 调虚表 `+0x10`
取得 FullName，`0x7bf2ea84` 以 SaveObject 写出。该取名路径与 RasterLayer
tag 30 共享多种候选对象体，两处的外层持久化方式不同。[FullName 来源复核](mxd-10.1-raster-layer-tail.md#三个名称宿主的接口与-fullname-来源)

参考工程和 66 个 10.1 介质 /Maps 流均未找到该 CLSID。合成 v4 RGB 测试覆盖
带过滤器的 tag 18、其后 tag 19 和共用体的边界，另验证两个非空槽中普通与
FileGDB 名称体、新对象和回引、波段名称独立读取、逐字节截断及超大计数回滚。
未做 ArcMap 运行验证。

全色锐化入口也用含 CacheRasterDatasetName 和 NetCDFRasterDatasetName
的合成 RGB 对象验证。第一个槽中的缓存名称定义 WorkspaceName 和 Envelope；
第二个槽的 NetCDF 名称回引 WorkspaceName，两个栅格的范围都回引同一
Envelope。正常输入继续读完 tag 19 与共用体；第二个槽截断或者名称类未知
时回滚整个 tag 18 记录及其新键。复查 134 份 10.1/10.8 介质工程与仓库
参考工程，135 份工程的 `/Maps` 和 `/PageLayout` 均完整读完，其中未命中
PansharpeningFilter 的 CLSID 字节。本轮没有实际全色锐化工程，也未做
ArcMap/QGIS 人工验收。

### SimpleRasterRenderer

RasterRenderer.dll 10.1 的 Load 先读 int32 版本，版本 ≥2 读一个
short（接口 +0x188），然后直接进入共用体 `0x7c98c3b0`，没有类专属尾段。
Save 在同一位置写 short，再调用 `0x7c98c6b0`。Save 版本表 `0x7c9b101c`
为 `[-1,1,1,2]`。以 RasterRenderer.dll 10.1.0.3035 原件的 objdump
复核：Load `0x7c996a04` 使用有符号小于跳转，两个分支在
`0x7c996a21` 均调用共用体读取器，未见独立版本上限或下限。

参考工程及 66 个 10.1 介质 /Maps 流均无该类 CLSID 命中。构造版本
覆盖负值、0、1、2 及大于 2 的值，验证 short 版本门、共用体边界和截断
回滚。尚无这些组合的实档或 ArcMap/QGIS 人工验收。

### SAIRenderer

安装介质 `esri.clsid.ecfg` 把该 CLSID 绑定到 RasterRenderer.dll 的
`esriCarto.SAIRenderer`。同一 10.1.0.3035 映像中，GetClassID 复制
`0x7c9b22fc`，QI `0x7c995040` 委派给 `0x7c98a270`，其比较的
`0x7c9b2378` 是 IRasterRenderer IID。

Load 读 int32 版本及 short，随后检查持久化接口 `+0x148`。新对象的
基类构造器 `0x7c989a20` 在 `0x7c989d29` 将对应的完整对象 `+0x18c`
清零，派生类构造和 Load 在读取此状态前不修改它，因此正常新对象分支
接着消费两个 double，再调用共用 Load `0x7c98c3b0`。Save 的同一状态门
控制写出两个 double，版本表 `0x7c9b102c` 为 `[-1,-1,-1,-1,1]`。
Load 在 `0x7c9955f0` 以有符号比较拒绝大于 1 的版本，其余版本进入同一
short 和对象状态分支。构造 v1、0 和负版本验证新对象布局，v2 在渲染器
头处停止。低版本和 Save 前状态位非零的形态仍缺实档。

参考工程与 66 个 10.1、68 个 10.8 介质 /Maps 流均无该 CLSID 命中。
合成测试经 RasterLayer 的正常分派进入 SAIRenderer，并从共用体继续读取
图层尾段到界。渲染器体逐字节截断时保留此前已读名称，回滚渲染器键。
该分支未在 ArcMap 中运行验证。

同一介质的 esriCarto 类型库 RefTab 直接列出的六个 IRasterRenderer
coclass 已全部接入；SAIRenderer 和 SimpleRasterRenderer 由注册表、
QI 和持久化虚表补证。注册表还列出 VectorFieldRenderer
`{BF3B4791-C58D-4FDF-A8FA-D0D8D7DA7061}`，但本次提取的 Desktop
PE DLL 中未找到该 CLSID 的内存序字节；当前没有足以识别其接口或对象体的
10.1 证据。该搜索范围也不能排除其他宿主 DLL 或外部注册实现。

### RasterDiscreteColorRenderer

在同一 RasterRenderer.dll 10.1 映像用 RTTI（`DiscreteColorRenderer`、
TypeDescriptor `0x7c9c4910`）定位 IPersistStream 虚表；GetClassID 复制
`0x7c9b1c20` 的 CLSID。Load 与 Save 都按以下顺序消费：

1. int32 版本、int32 颜色数量。Save 的表 `0x7c9b1180` 为 `[-1,-1,-1,1]`，
   读取器接入写侧已证的 v1。原生 Load 只有大于 1 的拒绝门。
2. 批量读写 `数量 × 4` 字节；IAT `0x7c9b0074` / `0x7c9b007c` 是
   unsigned long 重载。AfCore `0x68a84830` 直接委派 IStream::Read，颜色值
   的 32 位全部保留，未解释其颜色通道和高位业务语义。
3. AfCore String（接口 +0x124）；随后是带键 IColorRamp，Load 的请求 IID
   来自 `0x7c9b1be0`：`{BEB87091-C0B4-11D0-8379-080009B996CC}`。
4. 共用体 `0x7c98c3b0` / `0x7c98c6b0`。无类专属尾段；共用体 v9
   中可能另有一个 IColorRamp，不能跳过上一步的梯度。

参考工程及 66 个 10.1 介质 /Maps 流无此 CLSID 命中。合成测试核对带键梯度、
后继偏移、非法长度、资源限额和逐字节截断时的游标与登记表回退；未做 ArcMap
运行验证。

同版 RasterRenderer.dll 的 Load 在 `0x7c967ca2` 按有符号比较拒绝版本
大于 1 的输入。较低版本仍进入相同的颜色数组、梯度和共用体布局。
构造版本 0、-1 和最小 int32 验证边界，版本 2 验证回滚。
较低版本尚无实档或 ArcMap/QGIS 人工验证。

### RasterUniqueValueRenderer

CLSID 与入口见上表。用 pyghidra 补充现代 Load/Save 与旧体专用
Load `0x7c9a3960` / Save `0x7c9a40b0`，并对照 IAT 和分支指令。
Save 版本表 `0x7c9b0f80` 为 `[2,3,4]`，旧 Load 拒绝小于 1；
现代 Load 没有上限门，`0x7c99f47d` 对有符号版本大于 3 时读取梯度槽。
构造版本 5 与最大 int32 验证梯度槽、共用体边界与逐字节截断回滚。
这两种版本尚无实档或 ArcMap/QGIS 人工验证。

- v1/v2：两个 AfCore String、short、ISymbol、String、int32、计数加
  ILegendGroup 循环、ILegendItem，随后读分组值。后接 short、旧统计对象槽、
  int32 数量与 `double + int32` 对循环，最后进入共用体。v1 会交换分组键的
  高低半字，读取器保留原值。10.1 旧体 Save 在统计槽恒写空对象；原生也请求
  `{FAFDB8B5-5FE3-11D2-9F24-00C04F8ED1D7}`，与 esriCarto 10.1
  TypeInfo 46 的 `IEnumRasterUniqueValueColor` 对位；非空体见
  [旧版唯一值颜色枚举](#旧版唯一值颜色枚举)。
- v3/v4：三个 AfCore String、short、ISymbol、String、计数加 ILegendGroup
  循环，随后读分组值。后接 short、IUniqueValues，v4 再读 IColorRamp，最后
  进入共用体。请求 IID 来自 `0x7c9b2a44`：`{B2768AB2-E583-4CCF-831D-36237C2F520A}`，
  与 10.1 esriDataSourcesRaster 类型库 TypeInfo 85 的 IUniqueValues 对位。
- 两种分组值布局均为 int32 类别数，逐类别读取 4 字节键、int32 值数、
  `Variant::Read(false)` 循环；VARTYPE 后有 14 字节保留区，复用标量和
  已证数组读取器。空对象、回引及新 PropertySet 复用宿主键表，见[对象值证据](arcmap-10.1-variant-propertyset-slots.md#带保留区的-objectstream-对象值)；
  未知新类仍显式停止。
- RasterDB.dll 10.1 的 UniqueValues 类（非渲染器）CLSID
  `{9C81F1C7-792B-467E-AC80-069E4FDF4DEF}` 来自 IPersistStream 虚表
  `0x7c1370c4` 的 GetClassID `0x7c108e30`，Load `0x7c108c00` /
  Save `0x7c108d10` 对称读写 int32 版本、int32 计数及
  `Variant(false) + int32` 对。Save 表 `0x7c122248` 为 `[-1,1,1,1,1]`，
  读取器保留版本原值，并复用宿主对象键表。

参考工程、66 个 10.1 及 68 个 10.8 介质 /Maps 流中未检出这两个类。
合成测试覆盖四个版本、v4 额外梯度、数组值、非空 UniqueValues、数量越界和
逐字节截断后的引用表回退；未做 ArcMap 运行验证。

#### 旧版唯一值颜色枚举

10.1.0.3035 的 RasterRendererUI.dll（x86，ImageBase `0x10000000`，
SHA-256 `03701af97bb319d179deec8052c158a1bf4aa72438721d764e6ddf340b66e937`）
提供 `RasterUniqueValueColors`，注册 CLSID
`{D10498BE-6616-11D2-AABC-00C04FA33416}`。RTTI TypeDescriptor
`0x100a7310`、QI `0x1006a150` 和 GetClassID `0x100695c0` 将请求 IID
和该类对位。IPersistStream 虚表 `0x10094a04` 的 Load `0x10069ad0`
与 Save `0x10069eb0` 由 pyghidra 补充导出，并用 objdump 核对导入和分支。

对象流使用带键头，新建体依次读取 int32 版本 1、int32 类别数、
int32 键类型标志。标志为零时每类先读 double 键，非零时用 AfCore String
读键。其后每类读 int32 颜色数和该数量的 4 字节颜色值。Save 通过枚举
接口槽 `+0x10` 选类别，`+0x14` 取下一色值，类型库对应
`set_Class` 和 `Next`。代码保留原始颜色位模式；空槽和已见类键回引
不消费对象体。外层 10.1 RasterUniqueValueRenderer 的兼容 Save 仍在这
个槽写空对象。

Load 在 `0x10069e03` 以有符号比较检查色值索引和当前类别的色值计数。
零或负色值计数均跳到下一类别，Rust 同样保留该类别及其键并返回空列表。
外层类别数在 `0x10069c1a`（字符串键）或 `0x10069ca1`（数值键）参与
无符号分配大小计算，Rust 对负外层计数返回错误。

参考工程及 66 个 10.1、68 个 10.8 介质 /Maps 流未见该 CLSID。
合成测试覆盖两个键分支、后继采样与共用体、超大计数和逐字节截断后的
游标及键回滚。尚无旧版生产文件或 ArcMap 运行佐证。

### RasterClassifyRenderer

同一 RasterRenderer.dll 10.1 映像，Load `0x7c971ac0`、Save
`0x7c972060`（入口见上表）。Save 版本表 `0x7c9b1134` 为
`[3,4,4,4,4,5]`；由 Load 还可对位较旧的 1、2。Load 在 `0x7c971b3a` 拒绝有符号版本小于 1，
并在 `0x7c97200d` 对版本 ≥5 读取尾 short，未见版本上限。
构造版本 6 与最大 int32 验证共用体边界和逐字节截断回滚；
超出写侧版本表的组合尚无实档或人工验证。

1. v1 先读一个 int32；读两条 AfCore String。v1 接一条旧分类方法 String，
   其余版本改为单字节选择子，非零再读内联 16 字节 GUID。
   随后各版本均读第三条 String、计数与 ILegendGroup 循环、ILegendItem。
2. v1 在断点前额外有计数加 int32 数组；各版本断点均为 int32 计数加 double
   循环。v2 起再读 short、INumberFormat、short 和 double。INumberFormat
   请求 GUID 位于 `0x7c9b29e4`，复用已确认的九类数值格式。
3. v3 起读两个 `Variant::Read(false)`、short 选择子，再依原生 Load
   `0x7c971eea` 分派：−1 不消费对象（原生从现有图例取对象），其余读
   ILegendClass（IID 地址 `0x7c9b2ac4`）。10.1 Save 只对选择子 0 写对象，
   未经运行验证的其他值按 Load 决定是否读取。
4. v4 起读取 IUniqueValues（IID 地址 `0x7c9b2a44`）。其后可有 RasterIO
   magic `0x00ff00ff`，tag 1 为 int32、tag 2 为 double，末尾为
   `(-1,-1)`。RasterCoreLib.dll 的 InitCheck `0x7bf13b30` 对不匹配
   恢复流位置，End `0x7bf13bd0` 在未初始化时不继续读。当前只接入这两个
   有证据的标签。v5 再读一个 short，各版本随后进入共用体。

参考工程及 66 个 10.1、68 个 10.8 介质 /Maps 均无该类 CLSID 命中。
合成测试覆盖五个版本、非空数字格式和图例类、选择子 −1、RasterIO 缺省及
存在、非法数量和逐字节截断回退。未做 ArcMap 运行验证。

分级渲染器的 ILegendClass 回引沿用同版 SystemCore `0x7ffd0063`
的查键及接口查询顺序。构造输入在该槽误引此前的 CurrencyFormat，
验证整个渲染器游标和引用表回滚。尚无该组合的实档或人工验证。

INumberFormat（请求常量 `0x7c9b29e4`）同样按 SystemCore 的查键顺序
校验。构造 v2 验证 CurrencyFormat 回引后的字段边界，以及误引 RGB
颜色时的游标和键表回滚。该组合尚无实档或人工验证。

### RasterStretchRenderer

沿用已核验的 RasterRenderer.dll 10.1，补充 pyghidra 的旧 Load
`0x7c97fde0` 并用 objdump 复核。Save 表 `0x7c9b10e4` 为
`[3,4,4,4,6,9]`。Load 在 `0x7c97f119` 以有符号版本 3 为分界，
旧体在 `0x7c97fe2e` 拒绝低于 1 的值。RasterIO 路径未设上限，
在 `0x7c97f626` 对版本 ≥9 读取最后一对 double。构造版本 10 与最大
int32 验证该字段与共用体边界，尚无这些版本的实档或人工验证。

- v1/v2：波段 int32、AfCore String、0x930 字节查找表、int32、double、
  int32×2、double，随后依次是 IColorRamp、IColor×2、ILegendGroup、
  ILegendItem。IColor 请求 IID 来自 `0x7c9b4260`，ILegendGroup 和
  ILegendItem 分别来自 `0x7c9b1690`、`0x7c9b2ad4`。v2 比 v1 多
  一个尾部 int32，然后进入共用体。
- v3–v9：字符串之后是 RasterIO magic `0x00ff00ff`。tag 1–6 为
  int32、0x930 字节、int32、double、int32、double；保留拉伸类型原值，
  不做原生加载时的旧类型替换。tag 7/8/9/17 是无键对象：梯度、
  颜色、图例组、直方图。tag 9 必须为非空图例组（`0x7c97f2d9`），
  其余可省略；tag 9 的类由 Save 的构造函数 `0x7c9657b0`
  与 CLSID 地址 `0x7c9b16e0` 互证；tag 17 经 `0x7c983ba0` /
  `0x7c983d00` 查询 `0x7c9b43d0` 的 IStatsHistogram。RasterIO
  把原流传给对象体，其内部 LoadObject 可沿用宿主键表，不为无键根登记键。
  tag 8 的五种颜色复用 IColor 对象体分派。构造 v9 及以上的
  GrayColor 验证原始对象字节、无键边界及后继记录。该组合尚无实档或人工验证。
- tag 10 只在 tag 9 读到非空图例组后尝试读取，为 int32。tag 11–16
  依次为 int32、short、两个 double、int32、int32、double。v5 起
  增加 tag 18–22：int32、double、int32、计数 int32、字符串开关 int32。
  按 tag 21 的数量直接读 double，tag 22 非零时每个 double 后另读一条
  AfCore String。随后 v6、v7、v9 依次追加两个 double、一个 int32 和
  两个 double，都在 RasterIO 结束对之前。读完结束对才进入共用体。

参考工程、66 个 10.1 及 68 个 10.8 介质 /Maps 流均无该 CLSID 命中。
合成验证覆盖 v1–v9、原流引用键、附有字符串的采样数组、非法计数及逐字节
截断回退。未做 ArcMap 运行验证。

## 其余注册候选的持久化核对

### Raster3DRenderer

SceneRenderers.dll x86 10.1.0.3035 的 SHA-256 为
`1674d87a74366cff2943e45d3dd1fd9e10d89960b9480d1abea44ffd84c56da7`。
Raster3DRenderer `{4A524E70-93CC-11D3-A40C-0004AC1B1D86}` 的
RTTI `0x7d4bf750` 对位对象 +0x1c 的名义持久化表 `0x7d4ba038`；
GetClassID `0x7d4b00b0` 复制 `0x7d4b99c0` 的 CLSID。
QI `0x7d4ada40` 以 `0x7d4ba0f4` 的 IID 匹配 IRasterRenderer，
返回完整对象 +0x38。

类工厂 CreateInstance `0x7d4a42d0` 的非聚合分支在
`0x7d4a4395..0x7d4a43e4` 分配 0x5c 字节、调用构造器 `0x7d4ad8b0`，
再查询所请求的接口。构造器 `0x7d4ad90c` 将转发 QI 使用的完整对象
+0x44 清零。新实例的 Raster3DRenderer QI 对其自身接口逐项比较后，
经 `0x7d4adbbb` 到基类 QI `0x7d4b2f40`，再到
`0x7d4b5180`。这条链没有命中 IPersistStream（`00000109-…`）
或 IPersistVariant（`67A4D651-…`）的分支，最终返回
`E_NOINTERFACE`（`0x7d4b5272`）。其中辅助分支的独立对象 IID
为 `DF0B3D60-…` 和 `9F9650F1-…`，与持久化 IID 不同。

名义虚表的 Load `0x7d4b4c50`、Save `0x7d4b00f0` 处理 short
版本并按持久化接口 +0x2c 的指针调用委派体，流中没有该委派槽的对象头。
普通 ObjectStream 载入新实例时无法经上述 QI 取得这张虚表。
因此不能将这个 short 当成 MXD 中已确认的完整对象体。运行期
装配内部对象后是否有其他持久化路径仍待实档或运行取证。参考工程
和 66 份 10.1 介质的 /Maps 均无此 CLSID 的命中。

### RasterRenderer.dll 的实际类工厂

复核 72 份 Desktop 10.1 OLB 中的 coclass RefTab、导入接口与接口继承，
只有 esriCarto 的六类在此范围声明 IRasterRenderer。注册表中另有
SimpleRasterRenderer、SAIRenderer 和 VectorFieldRenderer。前两类
已按原生 QI 和持久化入口接入。

同版 SystemCore 在 `0x7ffd0063` 查键命中后对原实例请求
IRasterRenderer。共用槽按已登记的类判定，构造用例验证 RGB 渲染器
回引可读至后继标记，误引 RGB 颜色时游标及引用表回滚。
该组合尚无实档或人工验证。

RasterRenderer.dll 10.1.0.3035 的导出 DllGetClassObject
`0x7c96b420` 跳到 `0x7c9a76f0`。后者以 0x34 为步长扫描
`0x7c9c45a0` 起的类表，并在空指针处返回
`CLASS_E_CLASSNOTAVAILABLE`（`0x7c9a772d`）。首项 CLSID 指针
在 PE .data 中指向 `0x7c9b22bc`；初始化器 `0x7c9ae130`
填入其余十三项，并在 `0x7c9ae7d6` 写入类表的空终止项
`0x7c9c4878`。14 个实际工厂依次为：

1. RasterStretchColorRampRenderer、RasterClassifyColorRampRenderer、
   RasterUniqueValueRenderer、RasterRGBRenderer、RasterColormapRenderer、
   RasterDiscreteColorRenderer。
2. RasterRendererMakerDefault、RasterCatalogRendererPickerDefault、
   SimpleRasterRenderer、SAIRenderer。
3. FootprintRasterizer、RenderedRaster、RenderedRasterFunction、
   RenderedRasterFunctionArguments。

VectorFieldRenderer（`BF3B4791-C58D-4FDF-A8FA-D0D8D7DA7061`）
及注册在同一模块中的 VectorFieldRendererFunction 和参数类都不在
此类工厂表中。三者的 CLSID 在 1,115 个已提取的 Desktop DLL 中
也没有内存序命中。这里确认的是 10.1.0.3035 介质中的该模块
不会经其导出工厂创建这三类。第三方实现、其他补丁或平台需要独立核对。

## 样本与待查项

参考工程 `/Maps` 的两处偏移由有界结构读取到达（样本仅作佐证）：

| 渲染器 | 对象头 | 已消费内容 | 下一字节 |
| --- | --- | --- | --- |
| 色表 v3，键 262 | `0x523f` | 256 个整数值 0–255、图例组、共用体 v9 与色表尾字段 | `0x14479`，回到父图层 |
| RGB v9，键 1548 | `0x148c8` | 波段 0/1/2、三个 v5 直方图、四个带键数组、共用体 v9 | `0x16e8c`，回到父图层 |

两条 RasterLayer 在渲染器结束后均余 298 字节，由后续的
[RasterLayer 尾段读取器](mxd-10.1-raster-layer-tail.md)完整消费。
未知的 LoadRaster 内嵌类及尚无实档的渲染器宿主仍待核对。尚无带这两种渲染器的真实 ImageServerLayer 宿主；测试以 v13
合成宿主分别嵌入色表合成体与 RGB 实档体，验证渲染器后能继续读取，且遇
截断、未知类时回退到槽位起点，不提交渲染器及嵌套对象的新键。未做
ArcMap 或 QGIS 人工验收。

RasterDB.dll 10.1 的 UniqueValues IPersistStream Load
`0x7c108c00` 亦用于共用 Variant 分派。子 Variant 沿用宿主键表和
递归限额。合成测试覆盖嵌套集合、后继字段、截断和超限回滚。
真实 Variant 宿主与 ArcMap 10.1、QGIS 人工验收待核对。

RasterDB.dll 10.1.0.3035（SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`）
的 UniqueValues Load 在 `0x7c108c5b..0x7c108c68` 用有符号
`jle` 跳过非正条目计数。共享对象体保留原始计数。合成 Variant
测试验证空集合及下一字节位置。实档和人工验收待验证。

同摘要 RasterDB.dll 10.1.0.3035 的 UniqueValues Load
在 `0x7c108c2c..0x7c108c3b` 读取版本并检查读取结果，
随后在 `0x7c108c46..0x7c108c55` 直接读计数。余下路径只以
计数控制循环，没有依版本值选择布局。共享读取器据此保留原始
版本并读取同一布局，合成测试覆盖 v2 和 -1 的嵌套组合。
非 v1 实档及 ArcMap 10.1、QGIS 人工验收尚未验证。

唯一值和分类渲染器共享 IUniqueValues 槽。按 SystemCore.dll
10.1.0.3035 的 `0x7ffd0063` 查键路径，槽读取器使用已登记对象的
类判定回引。构造测试覆盖有效回引、错误登记类及既有引用表保留。
ArcMap 10.1 与 QGIS 人工验收未进行。

RasterRendererUI.dll 10.1.0.3035 的 RasterUniqueValueColors
IPersistStream Load（`0x10069ad0`）也接入共享 Variant 分派。
合成测试验证数值键、原始色值、后继字节及截断回滚。
真实 Variant 宿主和 ArcMap 10.1、QGIS 人工验收仍待验证。

同摘要 RasterRendererUI.dll 的 Load 在 `0x10069b18..0x10069b1c`
使用等值比较，仅版本 1 进入后续字段。合成 Variant 测试验证版本零、
版本 2 和最小有符号版本均失败，保留已登记键及入口游标。

旧版枚举槽按 SystemCore.dll 10.1.0.3035 的查键路径（`0x7ffd0063`）
识别回引实例。合成用例交叉使用 RasterUniqueValueColors 与 UniqueValues
的类标识，验证分派依据是登记类，并核对有效回引后的字节和已有键表。
ArcMap 10.1 与 QGIS 人工验收待验证。
