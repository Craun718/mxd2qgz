# MapFrame 对象体和 /PageLayout 接线

后续已接入装饰框、矩形和页面元素回引，见[布局续查](mxd-page-layout-surround-20261118.md)。本记录保留地图框接线当轮的检查结果。

以 10.1.0.3035 的原生实现为准。先复用[上轮 MapElements.dll 的 Ghidra 导出](mxd-map-tail-cross-stream-20261118.md#sdk-linearref-的跨流-mapframe)，用 `objdump` 复核版本门、导入跳转及委派实现。MapElements.dll SHA-256 为 `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`；LayerLib.dll（版本资源内部名 LayerUI）SHA-256 为 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`，均与既有介质身份记录相符。以下是静态映像 VA。

- MapFrame Load `0x79570a20` 从 IMap 槽读完之后，版本小于 4 在 `0x79570b1b..b40` 进入 LONG、double、IEnvelope 槽；小于 2 在 `0x79570c16` 再进 IGeometry、两个 short、IBorder 槽。两种旧分支的值保留原始位，未把其业务含义扩展到所有版本。
- `0x79570ce9/0x79570d83` 分别读两个 LONG 计数。其循环在 `0x79570d25/0x79570db4` 请求 IMapGrid 与 ILocatorRectangle；名称由磁盘常量地址 `0x7959c9b0/0x7959c190` 的 IID 与 esriCarto 10.1 类型库对位。最后 `0x79570de8` 通过导入槽 `0x7959926c` 调用 LayerLib 的 BaseFrameElement::Load。Load 唯独版本等于 3 时在 `0x79570df7` 之后再读 IMapFrame、LONG、double、LONG、IFeatureLayer。写侧存在版本转换分支，本轮没有把读侧门当作同版本对称写入承诺。
- LayerLib 导出表的 BaseFrameElement Load/Save 分别为 `0x7869b410/0x7869b530`。Load 先读版本（原生有符号上限 2），调用 `0x786d00d0`；该助手读独立版本（上限 3）、IGeometry 对象、版本 ≥2 的 LONG 和版本 ≥3 的 short。回到 BaseFrameElement，依序调用 `0x786b97f0` 读取 ElementProperties、载入 IBorder/IBackground，读一个 short，再在版本 ≥2 加载 IShadow。IIDs 取自 `0x7871f498/0x7871f4a8/0x7871f4c8`，用 esriCarto 10.1 类型库对位。LayerLib 导出的 get/put_DraftMode（`0x7869a720/0x7869a750`）访问接口视图 `+0x2c`；RTTI 虚表的该视图位于完整对象 `+0xc0`、持久化视图位于 `+0x0c`，故 `0xc0+0x2c=0x0c+0xe0`，对应同一 DraftMode 成员。原生有部分委派失败不马上退出；Rust 严格要求各体完整且不重现 COM 回调。
- 元素必须完整落在 ExtraSafe 声明范围内。空槽、已登记回引、已支持的 Geometry/Envelope 和 SymbolBorder/SymbolBackground/SymbolShadow 通用体可读；新 IMapGrid、ILocatorRectangle 或其他未知类没有外层子体长度，在元素开始处回滚整个事务。跨流入口还核对 IMap 回引指向已解析的根 Map，整张键表仍可能因另一个有界对象的未知内容而不完整。

SDK Java LinearRef 实档：元素包装位于 `/PageLayout:0x1e`，IMap 回引键 31 结束于 `0x50`。两个集合均为空，`0x74` 起为 BaseFrameElement v2，内联 Geometry 基类 v3 的 Polygon 键 418 含 UnknownCoordinateSystem 键 419。接续 ElementProperties v6（Type 是 `Data Frame`），再消费 SymbolBorder、SimpleLineSymbol 和 RgbColor 键 420–422，直到声明的元素终点 `0x27f`。该槽之后仍有 527 字节 PageLayout 尾段，不是 MapFrame 的剩余字节。工程的 /Maps 中还有未解析的有界图层体，未用此实档证明所有原生键都已登记。

`parse` 现在在 /Maps 根尾与根循环结束位置确认后把同一引用表用于 /PageLayout 根及受支持的 ExtraSafe 集合，报告地图框键、回引位置和根 Map 键。其余布局字段原样保留；未进入布局或遇到未知布局元素时仍报告未知流范围，不按 GUID 扫描恢复同步。对于 66 份 10.1 介质工程，48 份进入布局元素集合，共确认 36 个 MapFrame → Map 回引；14 份遇到暂未支持的 MapSurroundFrame。这些计数只描述当前顺序读取结果。

`cargo fmt --check`、`git diff --check` 与 `cargo test --no-fail-fast -- --include-ignored` 通过（881 项）。验证覆盖 v0–v4 读侧门、逐字节长度修正后的内层截断、未知网格不能登记新键及孤立 /PageLayout 无法解出实档回引；使用 LinearRef 从 `parse` 实际输出核对跨流位置。未做 ArcMap 10.1 或 QGIS 人工验收。完整 PageLayout 尾部、部分布局元素和跨流的其他类仍开放。

## IMap 槽的首次定义

MapElements.dll 10.1.0.3035（模块摘要见
[跨流研究](mxd-map-tail-cross-stream-20261118.md)）的 MapFrame Load
在 `0x79570aa2–0x79570ab0` 将 IMap IID 和当前流交给普通
LoadObject，调用方未限制其必须为回引。完整 MapFrame 读取器现
从同一流解析首次定义的 Map，要求 Map 后缀完整消费，并为跨容器
递归沿用已有深度上限。内嵌 Map 体、后继地图框字段及新对象键一并
提交，失败回滚。仅读取地图框前缀的接口仍要求空 IMap 或已登记回引。

以仓库参考工程已知边界内的完整 Map 构造布局元素，验证独立布局
可读取该内嵌 Map 并继续读完地图框。该布局为构造用例，ArcMap
10.1 和 QGIS 实档目视尚未执行。

独立布局的第二个地图框可回引首个地图框内定义的 Map。构造回归
验证该键跨元素保留，并核对诊断中的内嵌来源。另将首框末尾截断且
同步缩短外层声明长度，验证 Map 已读完后发生的地图框读取失败也会
撤销游标与内嵌对象键。

## 地图框语料复跑

以仓库参考档、介质提取件和本机 SDK 等样本中具有可读 `/Maps`
及 `/PageLayout` 的 286 个文件路径复跑当前读取器，包含 MSI
提取时带附加后缀的文件。同内容的不同路径未按字节去重。
地图框计数覆盖布局根元素、装饰框及递归组元素中的已解析体。

| 样本 | 路径数 | /Maps 到末尾 | /PageLayout 到末尾 | 地图框 | 首次定义内嵌 Map |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10.1 Desktop 安装介质 | 66 | 66 | 66 | 78 | 0 |
| 10.8 Desktop 安装介质 | 68 | 68 | 68 | 86 | 0 |
| 仓库、本机 SDK 等其他样本 | 152 | 150 | 150 | 171 | 0 |

两份未到达流尾的 SDK `mole_data` 仍在 GroupLayer 逐层遮罩处停止，
见[子层类证据](mxd-luna-deep-layer-masking-20261118.md#sdk-mole_data-的未识别子图层)。
已进入的布局没有地图框停点。此次样本仅验证现有路径的解析边界，
首次定义仍由构造回归覆盖。ArcMap/QGIS 人工核验未做。

### MapFrame v3 尾段的内联地图框

MapElements.dll 10.1.0.3035 的 MapFrame Load `0x79570a20`
在 v3 尾段通过普通 LoadObject 请求 IMapFrame。该槽与 LocatorRectangle
共用地图框分派、登记类检查和图形深度限额。构造数据验证首次定义的
地图框、其后的标量，以及误引根 Map 时整个外层地图框回滚。
真实内联地图框仍待取证。

### MapFrame v3 尾段的内联要素图层

MapElements.dll 10.1.0.3035 的 MapFrame Load `0x79570a20`
反编译中在三个标量后调用普通 LoadObject，请求 `0x7959c8b0`
的 IFeatureLayer IID。已接入普通 FeatureLayer 的首次定义及登记类回引，
只在图层完整消费后提交外层地图框。构造测试验证旧版图层体、
对象键及短读回滚。其他 IFeatureLayer 实现、真实宿主与 ArcMap
人工验证仍待补充。

### MapFrame 的有符号版本判断

MapElements.dll 10.1.0.3035（SHA-256
`67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`）
的 MapFrame Load 在 `0x79570a66 / 0x79570a6c` 拒绝有符号值大于 4。
`0x79570b1b / 0x79570b21` 和 `0x79570c16 / 0x79570c1c`
分别控制两个旧字段组。构造用例验证负版本包含两个旧字段组、保留原始版本
位模式，并与版本 5 的拒绝边界区分。真实负版本地图框尚待验证。

### MapFrame 的 IMap 回引

MapElements.dll 10.1.0.3035 的 MapFrame Load 在 `0x79570a9b..0x79570ab0`
调用普通 LoadObject 请求 IMap。同版 SystemCore 在 `0x7ffd0063`
按键找到登记实例后查询接口。读取器按登记类识别回引，已确认为 Map
但仅登记键的内联对象也能参与回引。构造测试验证回引头类标识不同
时仍能解析已登记的 Map，以及实际登记为其他类时回滚外层地图框。
目标软件人工验证仍待补充。

### 边框基类和矩形几何基类的版本门

LayerLib.dll 10.1.0.3035（SHA-256
`55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）
的 `0x7869b44c / 0x7869b451` 与 `0x786d0105 / 0x786d010a`
分别按有符号 short 拒绝 BaseFrameElement >2 和 RectShapeElement >3。
阴影门 `0x7869b4ec`、几何 LONG 门 `0x786d0163` 和 short 门
`0x786d0184` 同样采用有符号条件。构造地图框验证两级负版本
保留原始位模式、跳过后增字段并读完 DraftMode。真实负版本与原生打开仍待验证。
