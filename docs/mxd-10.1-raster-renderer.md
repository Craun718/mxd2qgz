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
| RasterDiscreteColorRenderer（DiscreteColorRenderer） | `{AC874573-D778-4421-B9C6-14557D8BD692}` | `0x7c9672f0` | `0x7c9b1aa4` | `0x7c967c60` / `0x7c967d70` |
| UniqueValueRenderer | `{0842B595-4F2F-11D2-9F43-00C04F8ECE3D}` | `0x7c99f900` | `0x7c9b6498` | `0x7c99f180` / `0x7c99f4d0` |
| RasterClassifyRenderer | `{CE8B2F44-A027-11D2-AAE7-00C04FA33416}` | `0x7c972450` | `0x7c9b277c` | `0x7c971ac0` / `0x7c972060` |
| RasterStretchRenderer | `{A301A3B2-74D7-11D2-9F29-00C04F8ED1D7}` | `0x7c97fc60` | `0x7c9b3edc` | `0x7c97f0d0` / `0x7c97f6b0` |

普通 RasterLayer::Load `0x7c73bd40` 调用 `0x7c73b380`。后者在每个
LoadObject(IName) 后，版本 >0 时再读一条 AfCore String，早先的名称体读取器未消费
此串；IName 循环成功返回后，主函数立即调用 LoadObject，`0x7c769b44` 的 IID 为
`{6F5322D1-3790-11D2-8D26-0000F8780535}`（IRasterRenderer）。本轮只在
ExtraSafe 有界体中，单个已支持的 IName 完整消费后读后续字符串和渲染器，避免把
多名称、非空未知 RasterIO 记录或损坏名称体中的字节误当成对象头。

## 已接入的读取范围

- 色表：int32 版本，int32 数量 + ILegendGroup 对象循环，int32 数量 + int32
  数组，再进入共用 Load `0x7c98c3b0`（Save `0x7c98c6b0`）。共用体为 int32
  版本、RasterIO magic `0x00ff00ff`、tag 1–5 的 int32 记录（2/3 受共同
  开关控制）、tag 6 的无键对象、tag 7 的三个 double 及 RasterIO 结束记录。
  共用版本 4/5/6/7/8/9 依次增加 short/int32/int32/byte/short/IColorRamp；
  总对象体末端是色表版本 ≥2 的一个 int32、≥3 的两个 int32。色表 Save 版本表
  `0x7c9b1190` 为 `[-1,1,1,2,2,3]`。共用体读取支持版本 2–9，
  tag 6 暂支持空与 RgbColor；ILegendGroup 和 IColorRamp 复用已有分派。
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

### RGB 的两个嵌套对象

- `StatsHistogram {BA3027C1-49CA-4788-8B5A-3A6B387DE78C}`：
  对 RasterDB.dll 10.1.0.3035（x86，ImageBase `0x7bfb0000`，SHA-256
  `97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`）
  补充 pyghidra 反编译。GetClassID `0x7c100650` 写该 CLSID；RTTI 的
  IPersistStream 虚表 `0x7c136850` 读写槽为 `0x7c101910`、`0x7c101b30`。
  对象体读 int32 版本、6 个 double、桶数组，版本 <3 固定 256 桶，≥3
  在数组前读 int32 数量；≥2、≥4 各追加一个 double，≥5 追加带 int32
  计数的 double 数组。读侧 v5 的容器调整通过 `0x7c101550` 保留正计数。
  参考工程三个对象均为 v5，各含 256 桶和 4 个追加值。
- `DoubleArray {60C06CA7-E09E-11D2-9F7B-00C04F8ECE27}`：
  类型库与 RasterRenderer.dll 请求的 `IDoubleArray {60C06CA6-…}` 对位。
  使用仓外现有 SystemCore.dll 10.1 反编译。虚表 `0x8007c22c` 的 Load
  `0x800419b0`、Save `0x80041a80` 先读写 short 和 int32 数量，再逐项
  调虚表；主表 `0x8007c1dc` 的 +0x28 槽 `0x80040520` 每项读 8 字节。
  这四个对象与 RGB 渲染器共用外层 ObjectStream 表；三个无键直方图不进表。

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

- 先依 AfCore LoadObject 读取 IDatasetName `{4BA33331-D55F-11D1-8882-0000F877762D}`；
  空标记为 16 个零字节，此后直接返回。过滤器允许两个槽都为空的编码。
- 非空名称在原生打开成功后还跟 int32 波段数量、AfCore String 循环、
  IGeodataXform、IEnvelope。名称回引只省略名称体，其后的波段和对象槽仍各自存在。
  解析器只读取持久化数据，不打开栅格。已接入普通 RasterDatasetName、FileGDB
  名称与其已证 RasterIO 记录，以及已有读取器支持的变换与包络。
- RasterIO 构造器 `0x7bf13ac0` 将传入的 IStream 原样存入 +0x0c；
  `0x7bf13e2e..0x7bf13e3a` 将同一指针传进过滤器的 Load。过滤器本身无键，
  其 LoadRaster 的名称、变换、包络在已接入的 MXD 宿主内仍走 AfCore
  LoadObject 的带键分支，复用宿主的 ObjectStream 表。未知类或内部前缀未闭合
  时保留 RGB 未读尾段及原表；普通 IStream 宿主和运行时数据源打开失败行为未验证。

参考工程和 66 个 10.1 介质 /Maps 流均未找到该 CLSID。合成 v4 RGB 测试覆盖
带过滤器的 tag 18、其后 tag 19 和共用体的边界，另验证两个非空槽中普通与
FileGDB 名称体、新对象和回引、波段名称独立读取、逐字节截断及超大计数回滚。
未做 ArcMap 运行验证。

### SimpleRasterRenderer

RasterRenderer.dll 10.1 的 Load 先读 int32 版本，版本 ≥2 读一个
short（接口 +0x188），然后直接进入共用体 `0x7c98c3b0`，没有类专属尾段。
Save 在同一位置写 short，再调用 `0x7c98c6b0`。Save 版本表 `0x7c9b101c`
为 `[-1,1,1,2]`；读取器只接受写侧已证的 v1、v2，原生 Load 未显式拒绝其他
版本。参考工程及 66 个 10.1 介质 /Maps 流均无该类 CLSID 命中；合成测试验证
v1/v2 的 short 门、共用体终点与截断回滚。未做 ArcMap 运行验证。

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

### RasterUniqueValueRenderer

CLSID 与入口见上表。用 pyghidra 补充现代 Load/Save 与旧体专用
Load `0x7c9a3960` / Save `0x7c9a40b0`，并对照 IAT 和分支指令。
Save 版本表 `0x7c9b0f80` 为 `[2,3,4]`，旧 Load 拒绝小于 1；
现代 Load 没有上限门，本实现限于已经对位的 1–4。

- v1/v2：两个 AfCore String、short、ISymbol、String、int32、计数加
  ILegendGroup 循环、ILegendItem，随后读分组值。后接 short、旧统计对象槽、
  int32 数量与 `double + int32` 对循环，最后进入共用体。v1 会交换分组键的
  高低半字，读取器保留原值。10.1 旧体 Save 在统计槽恒写空对象；原生也请求
  `{FAFDB8B5-5FE3-11D2-9F24-00C04F8ED1D7}`，与 esriCarto 10.1
  TypeInfo 46 的 `IEnumRasterUniqueValueColor` 对位；非空实现体尚未确认。
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
  读取器只接入写侧已证的 v1，并复用宿主对象键表。

参考工程、66 个 10.1 及 68 个 10.8 介质 /Maps 流中未检出这两个类。
合成测试覆盖四个版本、v4 额外梯度、数组值、非空 UniqueValues、数量越界和
逐字节截断后的引用表回退；未做 ArcMap 运行验证。

### RasterClassifyRenderer

同一 RasterRenderer.dll 10.1 映像，Load `0x7c971ac0`、Save
`0x7c972060`（入口见上表）。Save 版本表 `0x7c9b1134` 为
`[3,4,4,4,4,5]`；由 Load 还可对位较旧的 1、2。原生只拒绝小于 1，
此处限于已对位的 1–5。

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

### RasterStretchRenderer

沿用已核验的 RasterRenderer.dll 10.1，补充 pyghidra 的旧 Load
`0x7c97fde0` 并用 objdump 复核。Save 表 `0x7c9b10e4` 为
`[3,4,4,4,6,9]`，Load 以 3 为分界；读取器支持已对位的 1–9。

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
- tag 10 只在 tag 9 读到非空图例组后尝试读取，为 int32。tag 11–16
  依次为 int32、short、两个 double、int32、int32、double。v5 起
  增加 tag 18–22：int32、double、int32、计数 int32、字符串开关 int32。
  按 tag 21 的数量直接读 double，tag 22 非零时每个 double 后另读一条
  AfCore String。随后 v6、v7、v9 依次追加两个 double、一个 int32 和
  两个 double，都在 RasterIO 结束对之前。读完结束对才进入共用体。

参考工程、66 个 10.1 及 68 个 10.8 介质 /Maps 流均无该 CLSID 命中。
合成验证覆盖 v1–v9、原流引用键、附有字符串的采样数组、非法计数及逐字节
截断回退。未做 ArcMap 运行验证。

## 样本与待查项

参考工程 `/Maps` 的两处偏移由有界结构读取到达（样本仅作佐证）：

| 渲染器 | 对象头 | 已消费内容 | 下一字节 |
| --- | --- | --- | --- |
| 色表 v3，键 262 | `0x523f` | 256 个整数值 0–255、图例组、共用体 v9 与色表尾字段 | `0x14479`，回到父图层 |
| RGB v9，键 1548 | `0x148c8` | 波段 0/1/2、三个 v5 直方图、四个带键数组、共用体 v9 | `0x16e8c`，回到父图层 |

两条 RasterLayer 在渲染器结束后均余 298 字节，由后续的
[RasterLayer 尾段读取器](mxd-10.1-raster-layer-tail.md)完整消费。
其他 IRasterRenderer 具体类、多 IName 循环及未知的 LoadRaster 内嵌类仍待闭合。尚无带这两种渲染器的真实 ImageServerLayer 宿主；测试以 v13
合成宿主分别嵌入色表合成体与 RGB 实档体，验证渲染器后能继续读取，且遇
截断、未知类时回退到槽位起点，不提交渲染器及嵌套对象的新键。未做
ArcMap 或 QGIS 人工验收。
