# MXD 解析样本与反编译证据：样本无佐证元素族复核

记录日期：2026-11-17。范围是 `TODO.md` 第 250-261 行：MapSurround 循环非
空形态、书签循环非空形态、FieldInfo ≤3 分支、LegendClassFormat 样本形态、
HorizontalLegendItem 样本形态、BaseLegendItem 样本形态、Legend 族版本
0/1 形态、MarkerElement 体、LineElement 体、display 两轮闭合符号族样本形
态和 display 两轮闭合颜色族样本形态。本文是本轮唯一新增研究输出。

没有修改 Rust 源码、测试、`README.md` 或 `TODO.md`。没有运行 ArcMap
10.1，也没有在 QGIS 中人工打开或目视检查转换结果；本文不声称目标软件验
证。现有工作区已有未提交改动和并行文档，本文只引用、不改动。

## 证据分级

| 级别 | 含义 | 本文使用范围 |
| --- | --- | --- |
| A 样本证据 | 参考样本 `/Maps` 的对象边界走查或当前读取器级诊断 | 只证明该样本实例与可达分支 |
| B 反编译证据 | ArcMap 10.1.0.3035 原生 DLL 的 REA/Ghidra 静态分析 | 闭合字段序、版本门和读写侧控制流 |
| C 合成测试 | 读取器对构造输入的契约验证 | 不升格为 MXD 格式或目标软件验证 |
| D 运行取证 | ArcMap 保存、调试或受控 COM 观察 | 本轮未执行 |

样本证据只用于确认样本内的实例和空样本缺项，不能证明分支不存在；反编译
证据不等于运行时对象状态。A/B 两类证据在逐项表中分开标注。

## 文件身份与样本基线

| 文件 | SHA-256 | 长度 | 说明 |
| --- | --- | ---: | --- |
| `samples/测试底图.mxd` | `14772f8e3fedca8e1760bef77ab418011df4d8105f5159a05bc866bf603e3738` | 1,089,536 | 仓库参考样本 |
| `temp/agent-agent-sample-evidence-elements-20261117/maps.bin` | `fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe` | 152,941 | 本轮 `extract /Maps` 导出；与既有 `temp/r6-maps.bin` 逐字节同源 |

本轮只读解析样本成功到六条图层条目和活动图层索引 -1；数据框后缀停在
`/Maps:0x1706e` 的基本图形层停止点。输出副本见
`temp/agent-agent-sample-evidence-elements-20261117/parse-current.txt`。

| 原生模块 | SHA-256 | 地址证据 |
| --- | --- | --- |
| Map.dll 10.1.0.3035 | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` | B 级 |
| Layer.dll 10.1.0.3035 | `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8` | 既有 B 级；本轮 REA 新分析失败 |
| LayerLib.dll 10.1.0.3035 | `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02` | 既有 B 级 |
| Renderers.dll 10.1.0.3035 | `399c80c99a6c61de120c21b518d9b29069f2c78b20a705c07178aa3e2b656274` | 既有 B 级 |
| GraphicElements.dll 10.1.0.3035 | `f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9` | 既有 B 级 |
| DisplayCore.dll 10.1.0.3035 | `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2` | 既有 B 级 |

本轮 REA/Ghidra 12.1.4 对 Map.dll 的新鲜分析证据：

| 函数 | 内容 | Evidence ID |
| --- | --- | --- |
| Map::Load `0x793ff4d0` | MapSurround 与书签循环结构入口 | `ev_ae0b81ded70ffdea2cb3eb83b1bc6e25ba50b741d0f4ef390c63dc3c1344e95e` |
| Map::Save `0x79401850` | 两循环写侧入口 | `ev_b00518d94d53fc5f2be0628257dd612a0e91d31eaadb7cbc917abfb7d7bcff6c` |
| MapSurround 共同前缀 Load `0x794413d0` | 版本/String/IMap/条件字段 | `ev_5ad07a423c6b9918b82757e294eeabb304a110df844b168bddea5d2151a97f85` |
| Legend::Load `0x793da250` | Legend 自身版本与条目循环 | `ev_33b7d165469d0e4dd23b60bcf46acc287f59caf90b934cd197889eb4f6dc80d4` |

上述 Evidence 的 subject 均为上表 Map.dll 哈希。Layer.dll 新建 REA 分析在
`/tmp` 99% 占用时以 `java.io.IOException: 设备上没有空间` 失败，未得到新
函数证据；失败尝试不作为证据。本文的 Layer/LayerLib/Renderers/Graphic
Elements/DisplayCore 结论均引用既有研究记录，那些记录绑定同一哈希或哈希
前缀相同的 10.1.0.3035 文件。

## 样本计数与目标类扫描

一次性只读脚本 `temp/agent-agent-sample-evidence-elements-20261117/scan
_reference_maps.py` 生成 `scan-reference-maps.json`。它只做固定偏移计数、
16 字节 GUID 原样计数和命中偏移记录；不做对象边界推断。关键结果：

| 检查 | 值 | 证据级别 |
| --- | --- | --- |
| MapSurround 循环计数 @`0x24f60` | 0 | A |
| 书签循环计数 @`0x24f9c` | 0 | A |
| Legend、LegendFormat、Scalebar、ScaleText、MarkerNorthArrow、MapTitle CLSID | 0 | A（候选扫描） |
| AOIBookmark、FeatureBookmark CLSID | 0 | A（候选扫描） |
| LegendClassFormat、HorizontalLegendItem、HorizontalBar/Nested/VerticalLegendItem CLSID | 0 | A（候选扫描） |
| MarkerElement、LineElement CLSID | 0 | A（候选扫描） |
| FieldInfo CLSID | 204 | A（候选计数；文档已对象级确认 14 个样本首图实例，其余在后续图层） |
| RgbColor CLSID | 667 | A（候选计数；对象级记录为首图 3 实例） |
| SimpleMarkerSymbol | 12 | A（候选计数；对象级记录首图 1 新对象与回引，后续有实例） |
| SimpleLineSymbol | 282 | A（候选计数；不是已闭合样本实例计数） |
| SimpleFillSymbol | 301 | A（候选计数；不是已闭合样本实例计数） |
| MultiLayerMarkerSymbol | 18 | A（候选计数；见下节新发现） |
| CharacterMarkerSymbol | 9 | A（候选计数；见下节新发现） |
| 第一轮其余 MarkerLine/CartographicLine/HashLine、第二轮其余符号与三装饰 CLSID | 0 | A（候选扫描） |
| Hsv/Hls/Gray/CmykColor CLSID | 0 | A（候选扫描） |

简单 CLSID 计数不能等同于实例数：嵌套、类名/接口引用或重复字节都可能造
成差异。只有明确写出对象级走查的计数才能升为 A 级实例证据。

## 本轮样本证据升级：display 两轮符号

既有 round2 文档写「第二轮 15 类样本 0 实例」；本轮固定偏移和 GUID 扫描发
现两类有明确成对对象头候选。当前 `parse` 在基本图形层之前停止，没有独立
读取这些段，因此这不是完整对象体解析。基于 ObjectStream 既有 A/B 级规则
（16 字节类标识 + 4 字节键 = 新对象或回引头，版本 short 紧随新对象体），
这些命中应从「全流零命中」降级改为「存在样本对象头候选，但体未对象级走
查」。

| 类 | 候选对象数 | 偏移与键 | 体首版本候选 | A 级判定 |
| --- | ---: | --- | --- | --- |
| MultiLayerMarkerSymbol `{7914E5FF}` | 18 | 9 对：键 1559/1623/1680/1738/1796/1879/1957/2029/2105 @ `0x171d9,0x17465` 等，每组前头 `0300`、后头 `0200` | 3 和 2 | 对象头候选成立；体未走查 |
| CharacterMarkerSymbol `{7914E600}` | 9 | 键 1565/1629/1686/1744/1802/1885/1963/2035/2111 @ `0x1730c` 等 | 4 | 对象头候选成立；体未走查 |

偏移 16 字节对齐样本：`0x171d9` 后键 1559、版本候选 3；`0x1730c` 后键
1565、版本候选 4。同一键在 MultiLayerMarker 中重复出现符合 ObjectStream
「前首现新对象 + 后回引头」模型：第二次头 `0x17465` 键 1559、版本字节
`0200` 实际属于后续嵌套符号体，而非第二个体首。这一层仍未验证。

第一轮 SimpleLine/SimpleFill 的大量候选计数也不能写成「闭合类已有多样样
本形态」。既有对象级记录只覆盖首图 SimpleMarkerSymbol、RgbColor 和
SimpleFillSymbol 嵌套；其余命中必须按对象边界逐层走查后才能按类计数升
级。本轮不新增 `Cargo` 试解析，避免在 `/tmp` 受限和未提交工作区上产生
可误读的断言。

## 逐项结论

下表「证据」列只列支撑该项的最小事实；「分支」列给出可构造边界或当前实
现行为；「开放问题」列仍需真实样本、旧档或运行取证的内容。所有静态反
编译结论都是 B 级，不能替代 D 级验证。

| # | TODO 项 | 结论 | 证据 | 分支 | 开放问题 |
| --- | --- | --- | --- | --- | --- |
| 1 | MapSurround 循环非空形态 | 循环结构 B 级闭合；样本计数 0，非空形态未验证 | Map::Load/Save 新鲜 B 级证据（上表）；样本 A 级计数 0 | 计数 int32、普通 LoadObject、无 ExtraSafe 前缀；正数进循环，负数原生跳过而读取器拒绝；Legend 及四个装饰族读取器按 CLSID 分派 | 非空样本的真实类分布、请求 IID 全部实现、共同前缀 32 字节语义；含装饰物的 10.1 样本或运行取证 |
| 2 | 书签循环非空形态 | 循环与 AOI/Feature 体 B 级闭合；样本计数 0 | 同上循环证据；样本 A 级计数 0；AOI/Feature 体字段序见既有 `arcmap-10.1-mapsurround-bookmark.md` | AOI v0/v1：IEnvelope + String；Feature v0/v1：int32 + IName + String；读取器对 IName 新对象停止 | FeatureBookmark int32、IName 实现分布、IName `+0x14` 调用语义；含书签样本或受控运行 |
| 3 | FieldInfo ≤3 分支 | 静态分支 B 级闭合；样本可达 FieldInfo 全 v4，≤3 无字节佐证 | Layer.dll FieldInfo Load/Save 与版本门为既有 B 级；样本 FieldInfo v4 走查为 A 级 | v≤1 不读 -0x14；v3 读 -0x18/-0x1c 但无 -0x8/-0x6；v4 全字段；v>4 与负值位模式拒绝 | ≤3 历史样本或旧档运行取证；FieldInfo/NumericFormat 标量语义 |
| 4 | LegendClassFormat 样本形态 | 字段序 B 级闭合；样本 0 实例 | Layer.dll Load/Save 既有 B 级；本轮 CLSID 扫描 0 命中 | v≤2，四个对象槽（ITextSymbol×2/ILinePatch/IAreaPatch），v>1 两 double；TextSymbol 新对象停止，Line/AreaPatch 命中闭合体 | 两 double、TextSymbol 槽实现分布、槽空/回引分布；真实 Legend 样本 |
| 5 | HorizontalLegendItem 样本形态 | 包装 + 基类布局 B 级闭合；样本 0 实例 | Renderers Load 链和 LayerLib BaseLegendItem 既有 B 级；本轮 CLSID 扫描 0 命中 | 条目版本 ≤1；基类版本 0..3，v≥2 前后缀五字段，v>2 INumberFormat；各对象槽未知类停止 | 真实样本中的 ILayer/符号/格式槽形态、标量语义；含图例样本 |
| 6 | BaseLegendItem 样本形态 | 基类字段序 B 级闭合；不是独立 coclass，无独立对象头形态 | LayerLib BaseLegendItem Load/Save 既有 B 级；样本无 Legend 条目 | 基类 v<2 写默认不读流；v≥2 读五字段；v>2 读 NumberFormat；原生无拒绝门而读取器只接受 0..3 | 旧版 v0/v1 的自然字节来源；与四实现类共存的真实分布 |
| 7 | Legend 族版本 0/1 形态 | Legend 与共同前缀旧版本分支 B 级/合成 C 级闭合；样本无 Legend 实例 | Map.dll Legend Load/Save 本轮 B 级复核；样本 CLSID 0 命中 | Legend 自身 0..5；共同前缀 0..3；v1 读两个遗留 double 且无版本门尾段；10.1 Save 恒写 Legend 5、共同前缀 3 | 真实 v0/v1 字节和历史写侧来源；共同前缀 v0/v1 的门差异样本 |
| 8 | MarkerElement 体 | 字段序 B 级闭合；样本 0 实例 | GraphicElements Load/Save 与类身份既有 B 级；本轮 CLSID 扫描 0 命中 | 版本 ≤3；EP 前缀、IMarkerSymbol、IPoint、v>1 int32、v>2 32 字节；符号/几何未知新类停止 | 符号槽实现分布、IPoint 体分派完整覆盖、样本字节和目标软件验证 |
| 9 | LineElement 体 | 字段序 B 级闭合；样本 0 实例 | GraphicElements Load/Save 与类身份既有 B 级；本轮 CLSID 扫描 0 命中 | 版本 ≤3；EP、ILineSymbol、IGeometry、v>1 int32、v>2 short；几何分派覆盖 Polyline/Polygon/Point/Multipoint，未知类停止 | 符号槽实现分布、旧版本自然样本、软件验收 |
| 10 | display 两轮闭合符号族样本形态 | 静态布局与分派 B/C 级闭合；样本覆盖较旧记录更复杂，未完成对象级统计 | 两轮 DisplayCore 记录为既有 B 级；本轮 A 级扫描发现 MultiLayerMarker 18/CharacterMarker 9 候选，其余目标 CLSID 0 命中 | 第一轮 6 类和第二轮 11+3 类均有读取器；符号槽分派 19 类；未知类安全停止；版本门逐类保留 | 所有候选头的对象级走查、SimpleLine/SimpleFill 计数归因、MultiLayerMarker v3/v2 与 CharacterMarker v4 体、标量语义 |
| 11 | display 两轮闭合颜色族样本形态 | 静态布局与五类分派 B/C 级闭合；样本首图确认 RgbColor，其余颜色族无对象级实例 | 共享 Color 核心与 CmykColor 读写为既有 B 级；本轮 RgbColor 扫描 667 候选，Hsv/Hls/Gray/Cmyk 0 命中 | RgbColor/共享核心 v≤1 双版本；Cmyk v0..4；分支/取补字节按原样保留；未知颜色类停止 | RgbColor 667 个候选的对象级归因；非 RgbColor 类样本；颜色标量语义 |

## 需要的样本与可构造边界

优先级由样本可达性决定；所有新样本保留原始 MXD 并记录 ArcMap 版本、保存
操作和 `/Version`，再用对象边界走查而非全文 GUID 粗扫升级证据。

1. 含装饰物与图例的 10.1 样本。生成方式：在 10.1 数据框中添加 Legend、
   Scalebar/ScaleText/MarkerNorthArrow/MapTitle、设置一个书签后保存。至少
   一组含图例条目，另一组含空图例条目。验证边界：MapSurround/书签计数、
   20 字节对象头、共同前缀版本 3、Legend 版本 5、条目版本 1、BaseLegendItem
   版本 3、LegendClassFormat 槽类型。
2. 含旧版本形态的样本。来源：ArcMap 8.x/9.x 保存的 MXD，或在受控环境中降
   版保存。重点覆盖 Legend v0/1、共同前缀 v0/1、BaseLegendItem v0/v1、
   FieldInfo ≤3、FeatureBookmark/AOIBookmark v0。不要用 10.1 写侧伪造旧
   版本并称为真实样本。
3. MarkerElement/LineElement 样本。在 CGL 或地图图形中插入点/线图形并保
   存；至少一个 EP 新对象和一组符号/几何回引；符号分别覆盖 IMarkerSymbol
   与 ILineSymbol 闭合类。
4. display 符号/颜色样本。为不同图层分别设置 MultiLayerMarker、Character
   Marker、SimpleLine、SimpleFill、Hsv/Hls/Gray/Cmyk 颜色后保存；保留当前
   参考样本中已出现的 9 组 MultiLayerMarker 和 9 个 CharacterMarker 头，
   用读取器试解析到明确停止点。
5. 受控 COM/运行取证。创建单对象并直接调用 `IPersistStream::Save`，记录
   字节、HRESULT、游标和对象状态；适合先闭合 FieldInfo、LegendClassFormat、
   HorizontalLegendItem、MarkerElement、LineElement 和颜色族，再引入完整
   ArcMap 场景。

## 本轮新增或辅助产物

| 路径 | 作用 |
| --- | --- |
| `docs/mxd-parse-agent-sample-evidence-elements-20261117.md` | 唯一新增研究文档 |
| `temp/agent-agent-sample-evidence-elements-20261117/maps.bin` | 本轮样本 `/Maps` 导出，与既有导出同源 |
| `temp/agent-agent-sample-evidence-elements-20261117/scan_reference_maps.py` | 一次性只读扫描脚本；曾误配 GUID 端序并在文中记录修正 |
| `temp/agent-agent-sample-evidence-elements-20261117/scan-reference-maps.json` | 循环计数、目标类候选计数与命中偏移 |
| `temp/agent-agent-sample-evidence-elements-20261117/parse-current.txt` | 当前只读 `parse` 诊断副本 |

定向验证：`cargo test --test map_surround --test legend_tails --test
feature_layer_tail --test element_body --test display_symbol --test
display_color` 通过 181 项（8/32/24/69/25/23）。Cargo 报告一条全局缓存清
理权限警告，但不影响构建或测试退出码。未做 ArcMap 10.1 与 QGIS 人工目视
验收。
