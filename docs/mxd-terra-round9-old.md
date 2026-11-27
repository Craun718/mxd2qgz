# 第九轮：旧版本对象和 SDK 候选复核

2026-11-17。本轮承接 [第八轮](mxd-terra-round8-old.md)，只研究 FieldInfo v0/v1、GroupLayer <11 和 Envelope v0 的真实字节证据与到达路径；没有修改 TODO 或产品读取器。静态反编译与宿主读取是不同证据，均不是 ArcMap 运行结果。SDK 的 ArcToolbox 流尚未建立宿主结构、引用表或对象边界；扫描值不能当作读取结果。

## 新发现：ArcToolbox 的 Envelope 候选可能错位

10.1 SDK .NET `Continents.mxd` SHA-256 `d20e7f7fea729c1ae9afc7d742745a2cdf16a0472ccedea2a2f46058758685e3`，`/Version` 文本 9.2、short 14；不把介质版本当文件保存版本。`/ESRI ArcToolbox` 的 `0x306a6` 和 `0x5b73c` 有 Envelope 类标识，之前以“16 字节类标识 + 4 字节键”猜测体首，两处 +20 都是 `00 00`。本轮核对连续字节，发现另一种更符合既有 Envelope 结构的解释：

| 类标识偏移 | +16 short | +18 起四个小端 double | +50 int32 | +54 起 16 字节 |
| --- | ---: | --- | ---: | --- |
| `0x306a6` | 3 | -180、约 -89.9、约 180.000009、约 83.6236 | 0 | GeographicCoordinateSystem 类标识 |
| `0x5b73c` | 3 | 四个 NaN 位模式 | 0 | 相同类标识 |

按 +20 为 v0 解码时，第一处的坐标得到极小的非地理数值，所谓标志 `0xa6a87a80` 又跨入后续类标识。两处相同的 +16 版本、32 字节坐标、四字节标志和后接的空间参考类标识强烈提示此前 +20 的 0 是**错位候选**。但无法仅由形态确定实际宿主是否采用无键格式、是否确为 Envelope 新对象或确认对象终点；也不能把四个 NaN 赋予业务语义。未将两处统计为 v0 或 v3 的宿主实证。测试独立断言上述原始字节及两种偏移，不调用对象条目分类器来制造宿主证据。

## 10.1 原生旧 Map 列表与 8.1 样本

本轮重核 Map.dll PE：x86、文件版本 `10.1.0.3035`、SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。这只确定资源声明与文件身份，不证明签名或实际运行。隔离 pyghidra/Ghidra 12.2 反编译结果（忽略目录 `temp/terra-round9-old/native.json`）补足旧入口：

- `0x794035f0` 对有符号架构版本 `<23` 调用 `0x794038a0`；后者对版本 17 直接返回错误。其余版本先读 int32 计数并循环：版本 `<17` 使用普通 `LoadObject`，版本 `>17` 使用 `ExtraSafeReadObject`；再读第二个 int32 计数并循环普通 `LoadObject`；最后读取 int32 活动索引。两个循环中的对象和应用层插入副作用不能单凭计数概括。
- `Map::Load` `0x793ff4d0` 在列表后、基本图形层前，对版本 `<22` 调 `0x79403ec0`；该函数对于版本 19 不消费流字节。故本样本中以下边界与原生静态路径相容。
- 10.8 Desktop CAB 里的 `calibrate.mxd` SHA-256 `f883510f90d05a01c4ae152aa008a74b14cab0e8469177b98ef7e3b40fc533b4`，`/Version` 原文 8.1、short 10，非 10.8 保存档。独立测试从 `/Maps` 根计数、Map 对象头和既有 Map 前缀顺序读取：Map 架构版本 19，前缀止于 `0x52`；两旧列表计数 `0x52=0`、`0x56=0`，活动索引 `0x5a=-1`，`0x5e` 接 CompositeGraphicsLayer 类标识及键 32。没有遍历旧列表元素，也不声明整个 Map 完整。独立表从本流根建立，跨流先前键未知，故键 32 的新对象分类仍受会话范围限制；类标识和偏移是独立可核对的字节观察。

旧列表的分支和零计数真实字节可独立记为**静态规则与空集合对位**；该样本没有由此到达任何 FieldInfo、GroupLayer 或 Envelope 旧版本。非空集合、版本 17 错误路径的实档行为及其余旧档均未验证。现行解析接线见下文。

## 独立测试及可勾范围

`tests/terra_round9_old.rs` 新增三个合成边界测试：FieldInfo v0/v1 空格式槽与尾部对位、每个前缀截断时游标和引用表回滚；GroupLayer v10 普通空子层、后续字段和逐字节截断；Envelope v0 无附加维度的字段及尾界。另有两个依赖本机介质的 ignored 测试，分别核对上面的两个 ArcToolbox 局部候选和 calibrate 旧 Map 宿主路径。合成数据是回归测试，不是旧档；ArcToolbox 测试是错位排查，不是宿主解析。两种 ignored 测试路径与诊断输出仅为本地研究，分享前检查敏感路径。

- **FieldInfo v0/v1：不可勾选实档分支。** 新增合成正常与截断测试；第八轮 v2 证据仍只覆盖 v2。
- **GroupLayer <11：不可勾选真实普通子层。** 新增 v10 合成回归；旧 Map 空列表无法提供 GroupLayer 实例。
- **Envelope v0：不可勾选真实旧档。** 新增合成无维度边界测试，先前完整截断与 Z/M 组合测试仍见 `tests/envelope_v0_round2.rs`；SDK 两处 +20=0 不再作为有意义的 v0 候选。

执行 `TMPDIR="$PWD/temp/terra-round9-old/tmp" cargo test --offline --test terra_round9_old -- --include-ignored`：5 项通过；`cargo fmt --check` 通过；完整 `cargo test --offline`：794 通过、17 忽略；`git diff --check` 通过。独立 Ghidra 桥接工具 6.9.0 CLI 使用 JDK 25、Ghidra `/opt/ghidra`、仓库内 TMPDIR，55 秒本轮时限到达后被取消，输出标记 `cancelled`，没有产出函数证据，不能说是 socket 失败或 provider 自身超时。随后采用独立拷贝的 Ghidra 工程和隔离配置，通过 pyghidra 获取上述静态结果，没有切换共享 Ghidra 桥接工具 目标。

ArcMap 当前不可用；未在 ArcMap 10.1 或 QGIS 中人工打开源工程与转换结果，不能声称旧版本兼容、转换结果或目标软件运行已验证。后续需取得能顺序到达对应旧对象体的可控旧档，或先闭合 ArcToolbox 宿主及对象引用机制；必要时追加旧介质。更多来源的全流扫描只能缩小候选范围，不能代替顺序读取。

## 双列表接线与 Map v19–v22 尾段

`Map.dll` x86 `10.1.0.3035` 的 `0x794038a0` 双列表分支已接入产品入口：
首表随架构版本选普通对象或 ExtraSafe，第二表固定为普通对象，最后读取活动索引。
两表分别保留计数、共用键表。未知新对象体从条目起点保留，声明计数超限、v17、
短读或 ExtraSafe 长度不合时不提交试读。非空旧列表由合成条目覆盖，暂缺真实非空列表。

`Map::Load 0x793ff4d0` 在 v20、v21 的基本图形层前调用 `0x79403ec0`，v22
则在基本图形层后调用。此路径读取必须为零的 LONG，v21 起再有一个 short。
查阅现有反编译以及 10.1 映像的接口常量，确认 v19–v22 从书签继续走照明、几何、
边框、变换列表、背景，最后读取 `IAnnotateMap`。版本门 0x13..0x16 请求
`{8C439003-14EC-11D2-A27E-080009B6F22B}`。后续读盘项由更高版本门控制。

现有源码导出未覆盖 AnnotateMap 的持久化体，使用 objdump 补查
`Annotation.dll` x86 `10.1.0.3035`（SHA-256
`66d00f95840e3762f31a12f23ae29499862a4b37b88da2754d0a0966025df773`）。
构造函数在 `0x6b1c5e5e` 设置虚表 `0x6b1d6a60`，QI `0x6b1c6099` 按
IPersist / IPersistStream 返回该子对象。GetClassID `0x6b1c68d0` 返回
`{8C439002-14EC-11D2-A27E-080009B6F22B}`，Load `0x6b1c6910` 只读
有符号 short 并拒绝大于 1，Save `0x6b1c6960` 写版本 1。支持该类的新对象、
回引及空槽。MaplexAnnotateMap 见下节。更早的 Map 后段仍待接线。

`calibrate.mxd` 的 `/Maps` 中两个列表都为空。根 Map 键 31 经基本图形层、
照明、两个变换条目与 AnnotateMap v1 完整读至 `0x4db`，随后正确读取四字节根尾值。
`/PageLayout` 共享该键表，完整读取 13 个元素，末项 MapFrame 键 2047 回引键 31。
参见[旧版布局与 Polygon](mxd-page-layout-tail-20261118.md#v0v6-的普通页面元素)。
原始介质工程已通过解析器回归，未在 ArcMap 10.1 或 QGIS 中人工核验。

## Map v14/v15/v16/v18 的对象结束位置

核对同一份 Map.dll 10.1.0.3035 x86 的反编译和 objdump 指令，摘要见[文件身份记录](mxd-round4-map-spatial-20261117.md)。`0x794002c3` 从 v13 起在变换段后读取 double，`0x79400301` 从 v15 起读取一个 short，`0x7940032b..0x7940034a` 从 v16 起请求 `0x79487fd0` 的 IBackground。`0x7940038a..0x79400394` 将 IAnnotateMap 限于 v19–v22，后面的持久化字段门也不适用于 v14–v18。由此确认 v14 结束在 double 之后，v15 结束在 short 之后，v16 和 v18 结束在背景槽之后。双列表入口 `0x794038a0` 在 v17 返回 `E_FAIL`。

从 `/Maps` 根入口分别构造这四个版本的后缀，核对各版本的非零根尾值以及最后一个字段截断时无法声明对象读完。四个新接入版本尚无实档对照，未在 ArcMap 10.1 或 QGIS 人工核验。

与同轮 MultiPatch 改动一起执行全量测试含忽略项，共 1051 项通过。仓库参考工程、66 份 10.1 介质工程和 68 份 10.8 补充对照中的 `/Maps` 与 `/PageLayout` 仍读至各自流终点。

## Map v0–v13 的对象结束位置

继续使用同一份 Map.dll 10.1.0.3035 x86 的 `0x793ff4d0` 反编译及原始指令。版本门按 Map 对象架构版本判断。`0x793ffa43..0x793ffa68` 的书签计数和 ISpatialBookmark 循环适用于各版。`0x793ffb46..0x793ffb5b` 读取书签后的第一个 short 并检查读取结果，未校验它的位模式。普通 Save 采用 `0xffff`，Rust 保留存盘值。`0x793ffc13` 以 v11 为界，早期正计数逐项在 `0x793ffc3a` 请求 IName，v11 起在 `0x793ffe0a` 请求 IStandaloneTable。旧名槽复用已有表和要素类名称体，未完整读完的名称及其键回退到条目起点。

| 版本门 | 原生读侧 | 持久化字段 |
| --- | --- | --- |
| v0–v3 | `0x793ffd17..0x793ffd7f` | 内联照明的三个点坐标 double 和一个附加 double。 |
| v4 起 | `0x793ffe6e` | 对象化照明，进入已有 IlluminationProps 读取器。 |
| v2 / v3 / v5 起 | `0x793ffe93..0x793fffb8` | 分别纳入两个 short 加一个 LONG、DelayBackgroundDraw、Expanded。数值原样保留。 |
| v6 / v7 起 | `0x793fffb8..0x79400036` | 分别开启 IGeometry 与 IBorder 两槽、边框后的显示字段块。 |
| v8 / v9 / v10 起 | `0x79400188..0x79400278` | 分别开启两个 LONG、临时 short、临时 double。 |
| v12 / v13 | `0x7940027a..0x79400291`、`0x794004cc..0x7940051c` | 读变换计数，再逐个读 IGeoTransformation 对象。v14 起每条对象前额外有 LONG。 |
| v13 起 | `0x794002c3` | 变换段后读一个 double，v13 在此结束。 |

构造测试逐个覆盖 v0–v13 的 Map 对象边界、非零 `/Maps` 根尾、最后字段截断、旧 TableName 新对象及回引、未知 IName 回退，以及两个无前导 LONG 的变换对象和回引。旧版本样本仍待补充，ArcMap/QGIS 人工核验未做。未知嵌套类仍会在对应槽位停止。

### FgdbTableName

FileGDB.dll 10.1.0.3035（SHA-256
`73cef6bac97b0bdba8e4b26e2052cefab0b55a32e3a4141c01701d7cbc5f1230`）
的 FgdbTableName RTTI 对应持久化虚表 `0x6fafed1c`。GetClassID
`0x6fa33250` 从 `0x6fafc6e4` 返回
`846C88C5-6CC5-49C7-A6FE-8FA03D79BD07`。构造函数 `0x6fa32e20`
在 `0x6fa32f25` 聚合普通 TableName
`06783DB1-E5EE-11D1-B0A2-0000F8780820`，在 `0x6fa32f55`
将它的 IPersistStream 保存在完整对象 `+0x24`。

Load `0x6fa33290` 读取 short 版本和完整对象 `+0x28` 的 short，
随后在 `0x6fa332f7` 经内层 IPersistStream `+0x14` 将同一流交给
TableName。Save `0x6fa33310` 写版本 1 和相同 short，再调用
内层 Save。内层沿用 [TableName 读写链](mxd-terra-round8-tablename.md)，
没有独立对象头。对相同摘要的 DLL 用 objdump 复核，
`0x6fa332be` 比较 16 位版本和 1，`0x6fa332c3` 以有符号 `jle`
接受不大于 1 的值。旧 Map 表槽及 StandaloneTable 共用此分派并保留
原始版本位模式。构造 Map 验证版本 1、-1、新对象、回引，以及超限版本
处游标和引用表回滚。ArcMap 10.1 与 QGIS 人工验收未进行。
旧 Map 中实际出现该类的版本和样本仍待验证。

### NetCDFTableName

NetCDFDB.dll 10.1.0.3035（SHA-256
`e78ea582fdd66f45dfa717eb22346e9f691e1bb4e26587a1d09f72b0d1bc5ebd`）
的 QI `0x7ae2eff0` 在 `0x7ae2f155..0x7ae2f1c3` 将
IPersistStream 映射到完整对象 `+0x2c`，对应虚表 `0x7ae61b80`。
GetClassID `0x7ae2fd80` 写出 `0x7ae5f7f0` 的
`2D597A23-A989-43C1-9B1B-19E75BBFB78F`。Ghidra 和 objdump 对照
Load `0x7ae2fdc0`、Save `0x7ae2fff0`，确认 short 版本、三个
AfCore String，随后五个普通 LoadObject 槽，顺序为 IWorkspaceName、
两个成员 IStringArray、维度名称 IStringArray 和维度索引 ILongArray。
Load 的有符号上限为 1，Save 写 1。共享名称读取器按有符号上限比较，
并保留原始版本位模式。旧 Map 合成测试覆盖 1、-1、最小有符号值和
超限版本，核对数组、后继回引及停止位置。负版本实档及 ArcMap 10.1、
QGIS 人工验收尚未验证。

Save `0x7ae300a6`、`0x7ae300b5` 分别创建 StrArray 与 LongArray
以写出后两个槽。Load 在 `0x7ae2ff21` 将维度计数置零，`0x7ae2ff2a` 在维度名称为空时
跳过取计数，`0x7ae2ff40` 在计数非正时跳过循环。索引数组的虚表只在
循环内 `0x7ae2ff71..0x7ae2ff7f` 使用。Rust 因此接受空维度名称与
空索引的组合，并拒绝已知非空维度名称缺少索引。用同摘要 DLL 的 objdump
复核；旧 Map 合成测试验证后继表名回引和错误回滚，尚无原生运行验证。
读取器复用工作空间和两个数组的对象体，保留存盘条目及宿主引用键。
合成旧 Map 案例覆盖非空数组、跨槽回引、版本越界和键回滚。
两套安装介质的 134 份工程与参考工程均读到 `/Maps` 和
`/PageLayout` 终点。这两个流的字节扫描未命中 FgdbTableName 或
NetCDFTableName 的 CLSID。旧 Map 中真实出现这两个类的版本仍待运行确认，
尚未做 ArcMap/QGIS 人工验收。

### ReferencedTableName

复核 MosaicDB.dll x86 10.1.0.3035，SHA-256
`424814c3eceb88871df221bb42cb420b61c79c110a1e46a0e05af744db0a39ed`。
RTTI 标识位于 `0x7a6722d8`，IPersistStream 虚表位于 `0x7a61fdbc`。
GetClassID `0x7a59b2c0` 从 `0x7a5fed58` 复制类标识
`AB144E5E-9475-473E-B078-4E95440C52FE`。
主 QI 在 `0x7a59a9eb..0x7a59aa05` 将 IName
`4BA33331-D55F-11D1-8882-0000F877762D` 映射到完整对象 `+0x14`。

用 Ghidra 和 objdump 交叉核对 Load `0x7a59b300` 及 Save `0x7a59b380`。
对象体只有 short 版本及一个普通 IName 槽，`0x7a59b367` 的 LoadObject
请求与主 QI 一致的 IID。保存时 `0x7a5f8228` 处的有符号版本表为
`[-1, -1, -1, -1, 1]`。版本 short 写成功后仍写 IName 槽。
读取端不为 -1 设专门分支，随后按同一规则加载子名字。

共享表名读取器按原生的有符号上限 1 比较，并保留原始版本值。子名字复用现有分派和宿主引用表；
旧 Map 测试覆盖 -1、新子对象及外层同键回引，未知或截断的子名字会把
旧 Map 槽和键回退到包装对象头。StandaloneTable 测试覆盖版本 0、1 和
子名字回引。尚无旧 Map 或 StandaloneTable 中的此类实档宿主证据。
参考 MXD 和两套 Desktop 介质 134 份工程均读至 `/Maps` 与 `/PageLayout`
终点。另对上述工程与 37 份 SDK 工程做两条流的类标识扫描，均未命中该类。
ArcMap 及 QGIS 人工验收尚未进行。

### NetCDFFeatureClassName

同一模块的 `NetCDFFeatureClassName` 持久化虚表位于 `0x7ae605f4`。
GetClassID `0x7ae1fd00` 复制 `0x7ae5f800` 的
`EE5EF168-7B62-4ED5-89BB-FF73D390AF93`，与类型库一致。
Ghidra 导出 Load/Save `0x7ae1fd40 / 0x7ae20010` 显示，short 版本之后为四个
AfCore String（成员偏移依次为 `+8 / +0xc / +0x14 / +0x10`）和两个
LONG，随后五个对象槽的请求 IID 顺序与 NetCDFTableName 一致；v2 起在
槽后追加四个 String。Save 写 v2。NetCDFFeatureClass 的 IDataset 虚表
`0x7ae5fcbc` 的 FullName 槽指向 `0x7ae14d60`，该函数在
`0x7ae14d91` 以此 CLSID 创建名称对象，说明其来源与图层数据源相符。

FeatureLayer、FeatureBookmark 和共享表名槽接入同一读取器。复用对象槽后
保留原始字符串和 LONG；维度名称为空时，原生不进入逐项索引循环。构造
用例核对 v0–v2、数组新键、同键回引、截断回滚及书签、独立表和图层后续
字段的位置。NetCDF 要素数据源尚未映射到 QGIS，运行和人工验收待补。

### XYEventSourceName

XYEvents.dll 10.1.0.3035（SHA-256
`296c314ff8b0f3dd4af91ec03106c20fac77716b33603b0f84352662409e228a`）
的持久化虚表 `0x8103e330` 对应 GetClassID `0x81035b90`、
Load `0x810365d0` 和 Save `0x810366a0`。GetClassID 返回
`309AA920-EAEC-11D3-9F8A-00C04F6BDF06`，与类型库一致。
XYEventSource 的 IDataset 虚表 `0x8103da4c` 中 FullName
`0x8102cb70` 经 `0x81033330` 创建同一 CLSID。

Ghidra 与 objdump 对位显示，体为 short 版本、两个 AfCore String，随后
三个普通 LoadObject 槽，依次请求 IName、IXYEventProperties 和
ISpatialReference。类型库中 IXYEventProperties 的实现
XYEvent2FieldsProperties（CLSID `71045CA2-7902-11D4-9FE5-00C04F6BDF06`）
的持久化虚表为 `0x8103d3f8`，Load/Save 为
`0x8102ad70 / 0x8102adf0`。该对象的体为 short 版本和三个
AfCore String。IXYEvent2FieldsProperties 的 CLR 接口成员顺序与
`0x8103d43c` 的访问器偏移相符，三个字符串依次为 X、Y、Z 字段名。
两个对象的 Save 均写版本 1，Rust 读取 0–1。

FeatureLayer、FeatureBookmark 及共享表名槽均分派该名称，内层表名与
空间参考复用既有读取器。构造书签核对表名和属性的新键、XYZ 字段、空间
参考回引、随后书签名称、未知源和短读回滚。尚无 ArcMap 10.1 宿主实档
和人工验收，未映射为 QGIS 图层。

参考工程与两套安装介质的 134 份工程均读至 `/Maps` 和 `/PageLayout`
的终点。两条流的字节扫描未命中 NetCDFFeatureClassName 或
XYEventSourceName 的 CLSID，这批工程用于回归通用解析链。

### RouteEventSourceName

RouteEvents.dll 10.1.0.3035（SHA-256
`c4655ccbb93363fff759825d7664b9897be64d328d975ed1c5827db97148097c`）
的 `0x7cc8e250` 持久化虚表对应 GetClassID `0x7cc4b480`、
Load `0x7cc4bf60` 和 Save `0x7cc4c020`。GetClassID 复制
`63BE9174-B8C7-11D3-9F7C-00C04F6BDF06`。RouteEventSource 的
IDataset 虚表 `0x7cc8d634` 的 FullName `0x7cc43200` 经
`0x7cc49c60` 创建该类，创建调用点为 `0x7cc49c86`。

对照 Ghidra 和 objdump，外层版本 short 后有两个 AfCore String，之后
三个 LoadObject 槽依次请求 IRouteLocatorName、IName、
IRouteEventProperties。保存版本为 1。嵌套对象同用宿主引用表。

| 类 | 持久化入口 | 字节布局 |
| --- | --- | --- |
| RouteMeasureLocatorName（`8CC373A6-2121-11D4-9FC2-00C04F6BDF06`） | `0x7cc7cad0 / 0x7cc7cb30` | 外层 short 后委派 `0x7cc52540 / 0x7cc52640`。基类版本 short 后为四个 String、IName、一个 String、v0–v3 的 short、LONG 和仅 v2 的额外 LONG。Save 写外层 v1、基类 v4。 |
| RouteMeasureLineProperties（`35BDF2F0-3B21-11D4-9FCB-00C04F6BDF06`） | `0x7cc6c320 / 0x7cc6a9f0` | 共用读取 `0x7cc6a8d0`。版本 short、四个 String、一个布尔字节和 LONG。v3 起再读 short 和 String，v5 追加 short。Save 写 v5。 |
| RouteMeasurePointProperties（`C13D6537-3C80-11D4-9FCD-00C04F6BDF06`） | `0x7cc6ba20 / 0x7cc6bae0` | 先读同一公共体。v1 之外再读两个 short、String 和 short；v4 起再加一个 short。 |

FeatureLayer、FeatureBookmark 和共享表名分派均接入。构造用例覆盖定位器
基类 v0–v4、度量属性 v0–v5、点线两类、表名新键和跨槽回引，以及未知
内层路线名称、截断和后继字段边界。尚缺 ArcMap 10.1 实档及人工验收。

### CoverageFeatureClassName 与 CoverageName

CoverageFDB.dll 10.1.0.3035（SHA-256
`d467ccf6733ab083b3cd5fdd783965c25968290285e23f98dc264508527a7270`）
中，`FeatureClassName` RTTI 所属持久化虚表为 `0x6e18340c`。
GetClassID `0x6e0e5990` 返回 CoverageFeatureClassName
`72F77DE8-122A-11D3-9F31-00C04F79927C`。Load/Save
`0x6e0e6150 / 0x6e0e6200` 读写 short 版本、两个 LONG 和一个
布尔字节，再在同一流上调用被聚合对象的 IPersistStream。
构造调用点 `0x6e0e5aaf` 的创建参数是 GdbCore FeatureClassName
CLSID `198846D0-CA42-11D1-AA7C-00C04FA33A15`，
`0x6e0e5ac8` 查询它的 IPersistStream 并写入完整对象 `+0x20`。

CoverageName（`1E921C72-122F-11D3-9F31-00C04F79927C`）持久化
虚表 `0x6e181390` 的 Load/Save 为 `0x6e0c3360 / 0x6e0c33e0`，
前缀只有 short 版本和 LONG。`0x6e0c2e2b` 创建 GdbCore
FeatureDatasetName `198846CF-CA42-11D1-AA7C-00C04FA33A15`，
同样经 IPersistStream 委派。两个外层 Save 常量均为 1。两处委派消费
内联对象体，引用表只登记真实对象头的键。

FeatureLayer、FeatureBookmark 和共享表名分派可读取 CoverageFeatureClassName，
IDatasetName 槽可读取两类。构造书签验证 v0/v1 的两个外层、内联对象的键、
工作空间回引、后续选择子及截断回滚。Coverage 数据源未映射到 QGIS。
参考工程及两套安装介质的 134 份工程均读至 `/Maps` 和 `/PageLayout`
终点。两条流的字节扫描未命中 RouteEventSourceName、CoverageFeatureClassName
或 CoverageName 的 CLSID，新增分支仍待 ArcMap 实档和软件人工验收。

本轮全量测试含忽略项 1052 项通过。仓库参考工程和 134 份介质工程的 `/Maps` 与 `/PageLayout` 均读至流末尾。另对两套介质、10.1 Desktop 补充条目和 10.1 SDK Java/.NET 条目核对首个 `/Maps` 对象头。181 个条目的首个对象确认为 Map，最早架构版本为 v19，另有 3 个条目无法确认首头。清点只覆盖首个对象，不涉及后续 Map 或嵌套 Map 的版本分布。

### CadDrawingName

现有导出缺少这组持久化函数，使用 Ghidra 补导，并与原始 x86 指令交叉核对。
CadFDB.dll 10.1.0.3035（SHA-256
`bce09ad55072092626021a77c6fee857112f9ab3b850eb2aa675de30471f52a4`）中，
CadDrawingDataset 的 IDataset 虚表为 `0x6cad9024`，FullName `0x6cab5410`
创建 CadDrawingName（`D4224309-A5CB-11D2-9B10-00C04FA33299`）。
CadDrawingName 构造器在 `0x6cab69a1` 安装 IPersistStream 虚表
`0x6cad944c`，GetClassID `0x6cab68b0` 返回同一 CLSID。

Load `0x6cab7030` 只接受 short 版本 1，与 Save `0x6cab6fc0` 写出一致。
接下来依次读写三个 AfCore String 和一个 IWorkspaceName 对象槽。LoadObject
调用点 `0x6cab70a6` 的目标 IID 为 `FADD975C-E36F-11D1-AA81-00C04FA33A15`。
读取器接入共享 IDatasetName 分派，复用工作空间回引表和未支持对象的停止位置。
构造用例验证了 FeatureClassName 内嵌该名称时能接读后续字段，以及未知版本、
截断和引用表回滚。CadFeatureClass 的 FullName `0x6cac0950` 创建通用
GdbCore FeatureClassName，CadFeatureDataset 的 FullName `0x6cac4970` 创建
通用 GdbCore FeatureDatasetName。

### CAD 图层的对象槽

CadLayer.dll 10.1.0.3035（SHA-256
`4ba623aefccd169f9fa43ec453d1bb680767d240f0afbff908e46ac7dcf7df9c`）的
IPersistStream 入口由 Ghidra 补导，版本判断、IID 和调用参数由 x86 指令复核。

| 类 | IPersistStream 虚表 | Load / Save | Load 接受版本 |
| --- | --- | --- | --- |
| CadLayer | `0x6cb22f4c` | `0x6cb11c10 / 0x6cb12430` | 1–8 |
| CadFeatureLayer | `0x6cb2256c` | `0x6cb07b10 / 0x6cb08340` | 1–4 |
| CadAnnotationLayer | `0x6cb21b44` | `0x6cb01f40 / 0x6cb026b0` | 1–4 |

按表中顺序，GetClassID `0x6cb11bc0`、`0x6cb06bb0`、`0x6cb01f00`
分别返回 `E299ADBD-A5C3-11D2-9B10-00C04FA33299`、
`E0F384B6-E0C1-11D2-9B30-00C04FA33299` 和
`E1B71879-A5DF-11D4-A215-444553547777`。

CadLayer 的公共前缀含 short 版本、String、三个 short、两个 double
和 ISpatialReference 槽。v1 随后请求专用 IID
`E0F384B7-E0C1-11D2-9B30-00C04FA33299`。
v2 起在公共前缀后读四个 short 和 IName 槽。随后处理同一个 IUnknown
对象槽，源数据打开失败走 LoadObject（`0x6cb121c6`），成功则将已创建的
对象传给 IObjectStream 的 `+0x48`（`0x6cb1226c`）。

CadFDB.dll 的 CadDrawingObject 支持该专用 IID，QueryInterface
`0x6cab7fe0` 返回对象 `+0x10` 的接口。CadLayer 成功打开源数据时，辅助函数
`0x6cb13d90 → 0x6cb05c80` 同样查询此 IID。CadDrawingObject 的 CLSID
为 `E0F384B8-E0C1-11D2-9B30-00C04FA33299`，IPersistStream 虚表
`0x6cad9b88` 的 GetClassID、Load 和 Save 分别为
`0x6cabca30`、`0x6cabca70`、`0x6cabcee0`。Ghidra 补导经指令复核，
对象体含固定版本 1、两个路径 String、一个 short 和一个 LONG、两个
32 字节 WKSPoint 块、两个 double，以及 LONG 计数和同数量的 LONG。
原生 Load 在路径重定位后设置变换，在本机数组容量不足时仍消费多出的整数。

CadLayer v3 增加 LayerLib 扩展，v4 增加路径，v5 增加 double，v6 增加
LONG，v7 增加 String 和专有 short，v8 以两个 double 取代该 short。
`0x6cb21410` 的写版本表为 6 和 8。读取器按 SaveObject 的磁盘结构保留
绘图对象头、对象体和会话键，适用于普通和有界图层列表。原生打开成功时
`+0x48` 只消费键并加载到已有对象，真实工程中后续回引及源文件可达性对
运行行为的影响仍待确认。

CadLayer 主 QI `0x6cb106a0` 经 `0x6cb1b590` 转到 `0x6cb1b1c0`。
比对三个函数的实际比较常量，ILayerMasking 与 IBasemapLayer 均无命中，
未匹配项在 `0x6cb1b2b2` 返回 E_NOINTERFACE。Map 逐图层遮罩遍历
可按该结果处理 CadLayer。

CadFeatureLayer 和 CadAnnotationLayer 在版本字段之后即调用同一个槽
（`0x6cb07b91`、`0x6cb01fc1`）。辅助函数 `0x6cb05b60` 查询的 IID
`18A45BA7-1266-11D1-86AD-0000F8751720` 确认是 IObjectStream。
SystemCore 该槽的实现校验流中的 CLSID，读取键，再把对象体载入调用方
提供的对象。[ObjectStream 原生取证](arcmap-10.1-object-body.md#保存侧辅助-0x7ffd0370虚表-0x48)

CadFeatureLayer 与 CadAnnotationLayer 的构造器分别在 `0x6cb06e01` 和
`0x6cb0125a` 创建将用于 `+0x48` 的聚合对象。其 CLSID 经 esriCarto
10.1 类型库确认为 FeatureLayer（`E663A651-8AAD-11D0-BEC7-00805F7C4268`）
及 CoverageAnnotationLayer（`0C22A4C9-DAFD-11D2-9F46-00C04F6BC78E`）。
两类外层的第二槽按 IUnknown 读取。路径重定位分支经
`0x6cb07c96` 和 `0x6cb020c6` 查询与 CadDrawingObject 匹配的专用 IID，
确认该槽可装入 CadDrawingObject。CoverageAnnotationLayer 的
IPersistStream 虚表 `0x6f7aac1c` 位于 FeatureLayer.dll 10.1.0.3035
（SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`）。
Load `0x6f6beb60` 将 v4 及以前委派到 `0x6f6becc0`，v5 先内联加载
聚合对象（调用点 `0x6f6bebcf`），再经 `0x6f6bf2e0` 读取两代共用
数据段。v5 追加 short、ILineSymbol 和 IMarkerSymbol 槽，两个 IID
地址分别为 `0x6f7aafd8` 和 `0x6f7aafc8`，由 esriDisplay 类型库确认。
Save `0x6f6bf8e0` 的写版本表 `0x6f794b3c` 为 2 和 5。
聚合注记层的详细布局见下节。

构造用例从普通和 ExtraSafe 图层列表入口覆盖 CadLayer v1–v8，检查活动图层
索引边界，以及截断和超限后的有界跳过与子对象键回滚。全量 Rust 测试含忽略项
共 1065 项通过。参考工程和 134 份介质工程的 `/Maps` 和 `/PageLayout`
均读至流末尾。两条流的字节扫描未命中 CadDrawingName、CadDrawingObject
或上述三类图层的 CLSID，CAD 分支仍待 ArcMap 实档及软件人工验收。

### CadFeatureLayer

CadFeatureLayer 的 Save `0x6cb08340` 写出两个 SaveObject 槽，首槽
写聚合 FeatureLayer，第二槽可写 CadDrawingObject。Load
`0x6cb07b10` 对首槽调用 IObjectStream `+0x48`，解析器核对 CLSID、
保留键并复用完整 FeatureLayer 体读取。与 SystemCore `0x7ffd0370`
一致，首槽的键不进入读表，体内调用普通 LoadObject 的字段仍按该流的
共享表读取。

写版本表 `0x6cb21414` 为 2 和 4。v1 不带外层 short 列表，v2 起
为有符号 LONG 计数和 short 列表，v3 起追加 String。v3 在 String
之后额外读取一个 short，v4 在同一位置读取两个 double。版本分支
由 `0x6cb07ed3` 至 `0x6cb08059` 和 `0x6cb08193` 至
`0x6cb082c6` 的读侧指令交叉核对。原生 Load 的实际消费受要素
源打开结果和接口计数影响，Rust 按磁盘声明计数读全字段，ExtraSafe
条目校验外层声明长度。CadFeatureLayer 进入中间模型时保留名称、可见性
和已解析坐标系，CAD 数据提供者尚未映射。

主 QI `0x6cb07040` 对未知 IID 依次尝试基础实现 `0x6cb1b1c0` 和
构造时聚合的 FeatureLayer（委派点 `0x6cb07555`）。检查本类比较
常量和 FeatureLayer 的非委派主查询路径后，ILayerMasking 与
IBasemapLayer 都不能命中。构造用例验证两种列表入口各版本结束位置、
聚合键不登记以及截断与未知聚合类时的事务行为。仍待含 CadFeatureLayer
的 ArcMap 实档及软件人工验收。

### CoverageAnnotationLayer 与 CadAnnotationLayer

依据 10.1 FeatureLayer.dll 已有 Ghidra 导出 `6f6beb60.c`、
`6f6becc0.c`、`6f6be810.c`、`6f6bf2e0.c`、`6f6bf8e0.c`，
结合安装介质 x86 指令复核。CoverageAnnotationLayer 的构造代码
`0x6f6b8128` 至 `0x6f6b8155` 创建 FeatureLayer 并取 IPersistStream，
将接口保存在完整对象 `+0x70`。v5 的 Load 直接调用该接口读取内联体，
流中没有独立的 FeatureLayer 头。旧版前缀读 String、三个 short、
IName 槽及两个 double。两者随后进入同一个集合读取函数。

| 读侧地址 | 磁盘字段 |
| --- | --- |
| `0x6f6bf323`–`0x6f6bf379` | LONG 计数、IFieldInfo 槽序列 |
| `0x6f6bf38c`–`0x6f6bf472` | LONG、LONG 计数、等量 LONG |
| `0x6f6bf487`–`0x6f6bf68a` | 各自带 LONG 计数的 OLE 字体、IColor 与 ITextSymbol 槽序列 |
| `0x6f6bf6a8`–`0x6f6bf89b` | LONG、带计数的 LONG 序列、带计数的 short 序列、两个 LONG、ILineSymbol 槽 |

字体槽的 IID 来自 `0x6f74ebdc`（`BEF6E002-A874-101A-8BBA-00AA00300CAB`），
现识别 StdFont。v2 起旧分支追加两个 String 和 short，v3 起追加 LONG
及 IDisplayFilter，v4 再加一个 String 和 short。v5 从公共段直接接读
short、ILineSymbol 和 IMarkerSymbol。未知子类或未完整读取的内联体
在保留外层边界的情况下整体回退。

CadAnnotationLayer `0x6cb01f40` 经 IObjectStream `+0x48` 读入预创建的
CoverageAnnotationLayer，聚合键保留在对象头且不登记到读表。其后复用
CadFeatureLayer 的绘图槽及尾段，写版本表 `0x6cb21418` 为 2 和 4。
`0x6cb026b0` 写入 short 列表的计数。读取器以磁盘计数确定边界，
ArcMap 运行时的源数据打开分支仍待验证。外层 QI `0x6cb014f0` 经
`0x6cb019e5` 转发至 CoverageAnnotationLayer 主 QI `0x6f6b8ab0`，
后者未命中时转发至同一构造流程创建的 FeatureLayer。沿实际比较和
委派路径复核，两个注记类均不提供 ILayerMasking 或 IBasemapLayer。

构造用例覆盖旧前缀和内联基类、非空字段及字体和颜色集合、外层 v1–v4
和 ExtraSafe 截断后的键回滚。注记层目前在解析结果和诊断中保留。
全量 Rust 测试含忽略项共 1069 项通过。参考工程和 134 份介质工程的
`/Maps` 与 `/PageLayout` 均读至流末尾，两条流均未扫描到这两类
注记层的 CLSID。两类仍待 ArcMap 实档及 ArcMap、QGIS 人工验收。

## Maplex 标注引擎与地图级属性

现有导出未涵盖这两个模块的相关 Load。使用原始 x86 指令核对构造器安装的虚表、
GetClassID 返回的 GUID 及 AfCore 导入，并用 Ghidra 补导 LabelPlacement.dll 读侧。
两个 DLL 的版本资源均为 `10.1.0.3035`。

| 模块 | SHA-256 |
| --- | --- |
| MaplexAnnotation.dll | `52ddc6e3f605826cacbbcca43046682ba38feb764260af083ed73eca7bfa9ceb` |
| LabelPlacement.dll | `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d` |

| 类 | IPersistStream 虚表 | GetClassID / Load / Save |
| --- | --- | --- |
| MaplexAnnotateMap | `0x7960fad4` | `0x795f6b80` / `0x795f76b0` / `0x795f7780` |
| MaplexOverposterProperties | `0x7857efa0` | `0x7856bc20` / `0x7856bc60` / `0x7856be60` |
| MaplexDictionaries | `0x7857d334` | `0x7855b8e0` / `0x7855c4d0` / `0x7855c5c0` |
| MaplexDictionary | `0x7857d2a0` | `0x7855b880` / `0x7855bf90` / `0x7855c150` |
| MaplexDictionaryEntry | `0x7857d200` | `0x7855b830` / `0x7855bb70` / `0x7855bbe0` |
| MaplexKeyNumberGroups | `0x7857d630` | `0x7855faf0` / `0x785602e0` / `0x785603d0` |
| MaplexKeyNumberGroup | `0x7857d57c` | `0x7855faa0` / `0x7855fe40` / `0x7855ff00` |

- Map v19–v22 请求 `IAnnotateMap`。类型库公开 AnnotateMap 和
  MaplexAnnotateMap。后者有符号版本上限为 4，保存侧写 4；仅 v2 读一个
  LONG 和一个 short，随后 v0–v3 读 IMaplexOverposterProperties，v4 至此结束。
  旧属性槽即使在原生中读出后被释放，也要消耗整个对象体并沿用共享键表。
- Map v23 起请求 `IOverposterProperties`，10.1 类型库公开 Basic 和
  Maplex 两类。MaplexOverposterProperties 上限 7，依次读 LONG、short，
  v2 起加 IMaplexDictionaries，v3 起加 short、LONG、double、IColor，
  v4/v5 各额外读一个 LONG，v4 起继续读一个 short，v5 起再读两个 short，
  v7 另加 IMaplexKeyNumberGroups。
- Dictionaries 与 KeyNumberGroups 为 short 版本加 LONG 计数及普通对象槽。
  Dictionary v1 的同一计数控制两列连续 BSTR。其余可接受版本读有键的
  DictionaryEntry 列表，之后读名称 String。DictionaryEntry 两个 String 后
  接 LONG，KeyNumberGroup 为 String、三个 LONG、String、一个 LONG。
  对尚未逐一绑定 COM 属性的标量保留类型及位模式。

合成测试在 Map v19 和 v51 的后缀入口分别验证旧标注体、词典的两种布局、词典
条目回引、编号组及后继字段对位。截断和超限计数使整个 Maplex 槽回退，撤销嵌套键。
两个地图级 Maplex CLSID 在已提取的 268 个介质流中均无命中，尚无对应实档和
ArcMap／QGIS 人工核验。

### 跟踪工作空间的表名称

同版 Map.dll `0x793ffc3a` 请求 IName，`0x793ffc8b` 调用 IName::Open，
成功后经 `0x793acce0 / 0x79466970` 对输出查询
`{FCB01CB5-9F0B-11D0-BEC7-00805F7C4268}`（ITable，常量 `0x79482000`）。
TrackingWorkspace.dll 10.1.0.3035 的 AMSDatasetName 可响应同一
IName。Open `0x80a60d70` 对类型值 5 调 IAMSWorkspace +0x14，
其余值调 +0x18（`0x80a60dfc / 0x80a60e01`）。后者对位
IAMSWorkspace 虚表 `0x80ad1d44` 的 `0x80a64010`，在
`0x80a640e0` 开始查询同一 ITable IID，常量 `0x80acf47c`。
介质类型库还列出 TemporalFeatureClass 实现 ITable。模块身份和持久化路径见
[AMSDatasetName 原生取证](mxd-terra-round9-bookmark.md#trackingworkspace-的-amsdatasetname)。

旧 Map 和 StandaloneTable 的共享表名称分派复用 AMSDatasetName 读取器。
合成 Map v7 验证新名称对象、后续回引和内嵌 WorkspaceName 的带键属性集，
也验证截断后回滚整个条目及两个新键。真实旧 Map 中的该名称尚无样本，
原生 Open 及 ArcMap/QGIS 人工验收尚未执行。

### SdeRasterTableName 的持久化入口

检查 Desktop 10.1.0.3035 的 RasterSDE8x.dll（SHA-256 `085aa62b32a0fac60a8f02b9244c9d1940ada727603f4ca3a79b4758246c482c`）。RTTI 标识 SdeRasterTableName，构造路径 `0x7c9e28e7` 安装持久化虚表 `0x7c9e8928`。GetClassID `0x7c9e2e20` 返回 `0x7c9e8588` 的 `942AC1C0-FEC4-11D3-8D6F-00C04F5B87B2`，与同版注册表一致。该表的 Load 和 Save 槽分别是 `0x7c9e2e70` 和 `0x7c9e2e60`。原始指令显示两个入口均直接返回 `E_FAIL`，无流读取、写入或委派。旧 Map 表槽继续保留该类为未知体。其他实现及历史版本未由本证据覆盖。

### 旧 Map 表槽中的空 IName

Map.dll 10.1.0.3035 的旧表循环 `0x793ffc30` 先清零输出指针，`0x793ffc4c` 调 LoadObject 后仅检查负 HRESULT。紧接的 `0x793ffc6b..0x793ffc73` 直接解引用该输出并定位 IName::Open，没有空指针检查，也没有跳到下一条的分支。空对象头不会产生 IName 实例，Rust 保留现有行为，在这一项起点停止并保留字节。构造回归将旧表条目替换为空标记，验证游标停在项首、未登记对象键。该异常输入未在 ArcMap 中运行。

### 旧表名称注册项清点

Desktop 10.1.0.3035 的 `esri.clsid.xml`（SHA-256
`13a72c90856b258dbd065b4cbc9a2eedf744b411edfa9ccb56b64ce5074eedce`）
按 ProgID 后缀 `TableName`、`FeatureClassName`、`ObjectClassName`、
`TableQueryName`、`FeatureQueryName` 和 `RasterBandName` 筛选。表名称读取器已接入
TableName、FeatureClassName、NetCDFTableName、DBTableName、
CoverageFeatureClassName、FgdbFeatureClassName、QueryTableName、
FgdbTableName、ObjectClassName、ReferencedTableName、RasterBandName、
RelQueryTableName、NetCDFFeatureClassName、TableQueryName 和 FeatureQueryName。
剩余 SdeRasterTableName 的读写入口见上一节。该筛选仅清点命名符合条件的注册项。
原生槽请求 IName，随后以 ITable 查询 Open 的结果，因此还需按接口检查其它命名形式
以及第三方类；本次未做目标软件运行验证。

结合后续的 Open 指令复核，旧表槽的非表结果已按接口结果分组：
FileName、ShortcutName、UnknownDatasetName 均不能给出有效 ITable；
RepresentationClassName 打开的是表示类，SdeRasterTableName 的持久化入口
直接返回 E_FAIL。其余已确认能打开表或要素类的内置名称均由共享表名分派
处理。`map_legacy_version_dispatch` 现在把五个不可用类标识逐一放入旧 Map
新对象与回引槽，验证游标停在条目头且不适用的登记键不冒充当前槽类型。
WMS/WMTS 等 OGC 连接名属于栅格服务名称，不属于旧表槽的 FullName 来源。
第三方工作空间扩展仍可能提供其它 IName 实现，需按其二进制另取证；
ArcMap 10.1 与 QGIS 人工验收尚未进行。

### UnknownDatasetName 在旧表槽中的 Open

复核 Desktop 10.1.0.3035 的 GdbCore.dll（SHA-256
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`）
和本篇已核对的同版 Map.dll。UnknownDatasetName 主 QI `0x709ba910` 在
`0x709ba917` 比较的 IID 常量 `0x70ab84e4` 与旧 Map LoadObject 使用的
`0x79482c08` 同为 IName `{4BA33331-D55F-11D1-8882-0000F877762D}`。
匹配时在 `0x709ba931` 返回对象 `+0x10`，对应虚表 `0x70ad0908`。
Map 在 `0x793ffc73` 选用该接口的 `+0x14` 项，实际入口 `0x709baa40`
对有效输出槽写入 `-1`，调用错误助手传入 `0x80040315` 并返回其结果。
因此这个 10.1 GdbCore 实现只提供失败的 Open 路径，无法为旧 Map 提供
ITable。读取器继续在该未确认对象体前停止。本结论来自指令与虚表，未在
ArcMap 运行验证，也未推及其它 DLL 的实现。

### RepresentationClassName 在旧表槽中的 Open

Desktop 10.1.0.3035 的 RepresentationDB.dll（SHA-256
`e9595cf4febf06df301eb0c1e3eeedf28f694e2d0b16526f61779d0932fe32af`）
中，注册类 RepresentationClassName
`4E74AA22-A1B6-47D4-9914-20F8C6C2B699` 的 QI
`0x7cb28880` 在 `0x7cb288cb` 比较 IName，返回对象 +0x14
的虚表 `0x7cb42590`。该表的 Open `0x7cb28e60`
先于 `0x7cb28ed4` 打开内层名称，再通过
RepresentationWorkspaceExtension（CLSID 位于 `0x7cb41988`）
于 `0x7cb2902c` 另行打开表示类，并于 `0x7cb2904e` 查询其 IUnknown。
因此旧 Map 槽的 ITable 查询落在表示类上，不能以其内层名称打开的表
作为该槽支持此类的依据。

表示类主虚表 `0x7cb41fd4` 的 QI `0x7cb20510` 及回退
`0x7cb3a480` 未直接接受 ITable。模块唯一的 ITable IID 常量
`0x7cb422c0` 在 `0x7cb26839` 被作为参数交给现有对象的 QI，
属于接口消费。72 份 Desktop MSFT 类型库也未声明 RepresentationClass
实现 ITable。旧表读取器仍沿用现有分派。这里核对的是内置表示类，
未运行 ArcMap 或验证替代工作空间扩展的返回对象。

### FileName 与 ShortcutName 的旧表槽判定

SystemCore.dll PE32 版本资源为 10.1.0.3035，SHA-256 为
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。
FileName `{53DA75DF-D01A-11D2-9F27-00C04F6BC69E}`
和 ShortcutName `{3BEB09E4-3941-4A07-9D1A-EC2B43BA7D50}`
的主 QI 分别在 `0x7ffc35e7` 和 `0x7ffdc047`
比较同一 IName IID 常量 `0x8006f8fc`，返回 +0x10 视图。
两张虚表 `0x8006f8a8`、`0x800727b0` 的 Open 槽
（+0x14）共用 `0x7ffdc190`。有效输出参数收到空指针，
随后返回 S_FALSE。两类均无法向旧 Map 提供 ITable。

合成旧表测试使用这两个真实类标识，验证读取器在条目起点
保留字节且不登记对象键。未在 ArcMap 10.1 或 QGIS 中运行
这两种合成输入。

旧 Map 在取得 IName 后调用 Open 再查询 ITable（Map.dll 10.1.0.3035，
读取调用点 `0x793ffc3a`，Open 调用点 `0x793ffc8b`），分派要求也适用于回引。共用名称槽读取器
对新对象和回引使用同一类过滤。合成用例先登记不适用的类，再从旧表槽
读取回引，验证游标停在条目起点且保留既有键。

SystemCore.dll 10.1.0.3035 的 ObjectStream 读取器在 `0x7ffd0063`
按磁盘键查表，命中后对已登记实例执行接口查询。名称槽据此使用首次
登记的类来做分派，共用对象头读取器继续返回磁盘头原值。旧 Map
合成测试覆盖两种类标识不一致的情况：有效表名仍能回引，已登记的
不适用类仍停在当前条目。指令来源见[重复键与替换调用点补查](arcmap-10.1-object-body.md#重复键与替换调用点补查)。
ArcMap 10.1 与 QGIS 人工验收未进行。

### NetCDF 名称的数组回引

NetCDFDB.dll 10.1.0.3035 的 `0x7ae2fdc0` 表名读取路径请求
IStringArray 和 ILongArray。SystemCore 同版本在 `0x7ffd0063`
按键找到原实例后查询接口。共享数组槽据登记类检查回引，并以登记类
返回有效条目。旧 Map 合成测试覆盖头部不一致的有效字符串数组回引、
长整数数组槽误引字符串数组时的停止位置及回滚。ArcMap 10.1 与 QGIS
人工验收未进行。

同模块的 NetCDFFeatureClassName Load（`0x7ae1fd40`）按维度名称
数组内容决定是否逐项取索引。共享 IStringArray 槽登记读完的数组与
体长度，同一会话的后续回引可在当前限额内恢复数组。合成书签用例
核对非空名称回引、有索引时的读取，以及缺少索引时的事务回滚。
ArcMap 10.1 与 QGIS 人工验收未进行。

属性集的 ObjectStream Variant 槽也可先遇到 SystemCore StrArray。
读取该类体时同步登记内容，使后续 IStringArray 槽能按键恢复数组。
合成测试串接 Variant 和有类型的槽，验证体长度限额及后继游标。
ArcMap 10.1 与 QGIS 人工验收未进行。

NetCDF 表名的工作空间槽请求 IWorkspaceName。共用数据集名称读取器
按 SystemCore 的查键规则，以已登记对象的类检查回引。合成旧 Map 测试
验证有效回引后继续读数组，遇到错误登记类时保留已有键并停在表名起点。
维度名称和索引长度不一致时，读取器保留两个独立数组的原始内容。

### NetCDF 表名的索引越界

复核 SystemCore.dll 10.1.0.3035（SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）
的 LongArray 虚表 `0x8007c368`。其 +0x18 槽为 `0x80042970`，
Ghidra 导出与 objdump 一致：下标越界返回 `0x80070057`，不写输出。
NetCDFDB.dll 同版本的表名 Load 在 `0x7ae2ff7f` 调该槽，随后的
`0x7ae2ff81..0x7ae2ffbe` 未检查返回值，继续循环。无法据此给未写
输出赋予确定值。合成旧 Map 测试确认空索引对象体和非空名称均原样
保留，并可继续读取下一条表名回引。
ArcMap 10.1 与 QGIS 人工验收未进行。

## 全流复核旧版 Envelope、Hyperlink、PropertySet 候选

用现有 CLI 的 inspect 和 extract 扫描参考样本、可读本地
10.1/10.8 介质及 SDK 工程。按整文件 SHA-256 去重后共 106
份文档，成功读取 2,004 条至少 22 字节的流。扫描以已知
CLSID 定位，暂以 `+20` 的 short 作带键对象版本候选。
Envelope 有 1,396 处 CLSID 匹配，本轮无 `+20` 为零的候选。
Hyperlink 未命中。PropertySet 有 640 处匹配，其中 601
处 `+20` 为零，全部在 ArcToolbox 流，且 `+16..+20`
均为 `01 00 01 00`。

在仓库参考样本的 `/Esri ArcToolbox` 偏移 `0x1da`，
从 `+16` 读为 short 1 和 LONG 1，随后能对位一个
终止符计入长度的属性名与 VT_BSTR 类型标签。这里的
`+20` 零落在 LONG 计数的高位，符合去掉四字节键后
的 v1 局部布局。其余候选只核对了相同前缀，未逐一建立
宿主对象边界。这批命中不能用作 PropertySet v0 的实档佐证。
三个 v0 实档待办保留，未进行 ArcMap 或 QGIS 人工验证。
