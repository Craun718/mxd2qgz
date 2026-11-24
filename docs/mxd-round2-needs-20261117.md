# MXD 解析：第二轮后续清单

本轮开始时有 85 个未勾选条目，复核闭合 7 项。剩余 78 项按当前下一步分成 10 项目标运行、20 项真实样本、48 项继续静态研究。下面用本轮开始前的 TODO 行号标记，不是实时行号。两个条目可共用验证材料时仍分别验收。

## 必须运行 ArcMap 或目标 Windows

以下结论要求观察目标软件行为。第 271 项要求目标 Windows 的 OLE 互操作，其余使用 ArcMap 或同版本原生组件。

| 原行 | 缺口 | 验证动作 |
|---:|---|---|
| 73 | 在 ArcMap 10.1 中人工验证 `/Version` 头部读写（当前仅有合成边界测试）。 | 在 10.1 实际打开、另存、比对 `/Version`；保留前后字节和软件版本。 |
| 76 | 确认旧文档打开后版本索引是否改写。 | 在目标版本打开旧文档，比较打开前后索引及 Save/Save As 后的文件。 |
| 77 | 运行时验证 `/Version` 版本映射。 | 覆盖 8.3、9.0/9.1、9.2、9.3、10.0、10.1 的已知映射，核对识别和保存。 |
| 78 | 运行时验证 `/Version` 索引输出。 | 采集原生版本索引输出，对比 Rust 候选与结果。 |
| 89 | 验证 10.8 `/Version` 的跨版本兼容性。 | 让 10.8 打开不同保存版本，再验证 10.1 对共通目标版本的读取。 |
| 117 | 触发并验证重复键替换分支场景（替换结果与回引交互待取证）。 | 通过原生 ObjectStream 构造重复键替换，验证下一次回引命中对象身份。 |
| 234 | Geometry 内核移交链（+0xc8 内嵌指针虚表 +0x1cc 槽）运行期装配定位。 | 在 Geometry 装载时捕获内嵌指针的实际对象、虚表与装配来源。 |
| 269 | Angle 原生忽略核心失败的容错路径（读取器较严）。 | 在原生 Angle 装载中令核心受控失败，记录 HRESULT 和前后偏移。 |
| 271 | OLE 字体持久化跨 oleaut32 版本稳定性（介质 2.40.4275 vs 运行时 OS 版本）。 | 在目标 Windows/oleaut32 版本间双向保存和加载 StdFont，核对字体属性。 |
| 274 | Geometry S_FALSE 捷径分支。 | 构造能真实触发 Geometry S_FALSE 的对象流，捕获 HRESULT 与游标。 |

## 需要补真实样本

已有合成测试或静态布局的条目仍需对应版本、非空槽位或回引形态的真实字节。生成样本时记录产品版本、流和对象定位；样本只保存在本地。

| 原行 | 缺口 | 样本要求 |
|---:|---|---|
| 126 | 取得 GroupLayer 版本分支的字节佐证。 | 带 GroupLayer 的文档，分别覆盖旧普通子层列表和新 ExtraSafe 子层列表。 |
| 232 | 验证 Percentage/Rate「同槽升级」与 LatLon/Angle 独立槽核心形态（样本 0 实例）。 | 含 Percentage/Rate 嵌套数值格式、LatLon/Angle 非空核心的文档。 |
| 249 | 解析 FCN 的 TopologyName 选择子。 | 含拓扑数据且 FCN 的 ITopologyName 选择子实际写入非空新对象。 |
| 250 | FDO 旧加载器体。 | 由旧版 ArcMap 保存的 legacy FDOGraphicsLayer；保留产品版本和对象外层。 |
| 253 | 书签循环非空形态（样本计数 0）。 | 各保存一个带 AOI 书签、要素书签的文档；从 Map 根验证。 |
| 255 | FieldInfo v0–2 分支。 | 含旧 FieldInfo v0、v1、v2 的实际要素层；现有介质只确认 v3/v4 候选。 |
| 260 | 验证 Legend 主体 v0/v1 的旧版分支（独立于已实测的 HorizontalLegendItem/BaseLegendItem v1）。 | 各一份 Legend 主体为 v0、v1 的旧版文档。 |
| 263 | 补齐尚未实测的 DisplayCore 符号类；两套介质已有 MultiLayerMarkerSymbol/CharacterMarkerSymbol 体读证据 | 已知两类符号之外，按 DisplayCore 类清单补有对应非空对象槽的工程。 |
| 264 | 补齐尚未实测的 DisplayCore 颜色类。 | 按 DisplayCore 颜色类清单补仍未观察到的非空嵌套颜色对象。 |
| 265 | Hyperlink 版本 0 短体。 | 含 Hyperlink v0 的旧文档，核对短体与下一条边界。 |
| 266 | Hyperlink 循环空标记/回引形态。 | 真实 Hyperlink 集合中的空引用和同键回引两种实例。 |
| 267 | NumberFormat 短体形态（九类全族）。 | NumberFormat 九类的旧版本短体，可从旧版文档或受控原生持久化取得。 |
| 268 | NumberFormat「同槽升级」与 LatLon/Angle 独立槽核心形态。 | 覆盖 232 的同一批样本，额外保存对象边界用于校验体长和后继字段。 |
| 273 | Geometry 版本 0/1。 | Geometry v0/v1 的早期对象，不能用 Point/Envelope v0 替代。 |
| 275 | 后继段版本 0x1c..0x1f 分支。 | FeatureLayer 后继段落在 0x1c 至 0x1f 的旧文档。 |
| 276 | TimeData 族四段（样本走 ≥0x20 门）。 | 保存四类非空 TimeData 段的工程，同时记录 FeatureLayer 架构版本。 |
| 277 | PropertySet 版本 0 分支。 | 真实 PropertySet v0 对象流；10.1 保存路径写 v1。 |
| 278 | 取得 Envelope v0 的真实旧档字节佐证（读取器已按 10.1 Load 的 v0/v2 同布局实现并验证合成边界）。 | 真实 Envelope v0 对象流；10.1 保存路径写 v3。 |
| 279 | FeatureDatasetName 版本 0 分支。 | FeatureDatasetName v0 的旧文档；现有证据为 v1。 |
| 280 | TopologyName 版本 0 分支。 | TopologyName v0 的旧文档；可与 249 共享取样。 |

## 现有材料还可继续静态研究

> 此表保留第二轮分派编号；第三轮的闭合条目及剩余状态见[第三轮复核](mxd-round3-review-20261117.md)。

这些项仍有未追完的原生调用、访问器或现成文件读取路径；不列为必须先运行或补样本。需要的证据和已排除的候选见[主审记录](mxd-round2-review-20261117.md)所链接的专题记录。

| 原行 | 缺口 |
|---:|---|
| 71 | 定性 10.1 `/Version` 头部未知尾部的字节语义。 |
| 74 | 闭合 SaveAs UI→put 间接路径（版本属性写入）。 |
| 75 | 核对本地化语言包文本对版本文本/识别的影响。 |
| 88 | 闭合 10.8 `/Version` 的写入规则。 |
| 100 | 闭合跨流复用 ObjectStream 的对象体。 |
| 101 | 确定跨流复用 ObjectStream 的版本适用性。 |
| 104 | 闭合 VariantStream 对象分支的对象体。 |
| 105 | 闭合 VariantStream 对象分支的引用表。 |
| 107 | 闭合 SimpleSaveObject 委派链（对象读写委派路径）。 |
| 113 | 闭合 Map 双名称之外的未定对象属性，包括书签后已读取的三个成员的业务用途、坐标变换及照明属性等后续对象，见 [本轮取证](docs/mxd-round2-review-20261117.md)。 |
| 114 | 闭合其他 MXD 版本的字符串规则。 |
| 115 | 闭合其他 MXD 版本的对象属性规则。 |
| 153 | 其余标注表达式引擎实现类体（样本为 AnnotationVBScriptEngine，已闭合）。 |
| 154 | LELP ㉔ 槽 IGraphicsContainer 实现类体（部分闭合：请求 IID 更正为 esriCarto IGraphicsContainer `{D09411D1-4BB4-…}`，旧记录第二 GUID 系笔误；样本槽位=空标记；实现者候选在 afcore 等未反编译模块，负证据落档，见 [研究记录](docs/arcmap-10.1-dep-tail.md)）。 |
| 155 | LELP ㉖ 槽 ISymbol 17 闭合类之外的实现类体（负证据清单落档；`read_symbol_slot_full_10_1` 缺 TextSymbol 分派的缺口已由整合轮补齐，见 [研究记录](docs/arcmap-10.1-dep-tail.md)）。 |
| 174 | FeatureBookmark 的 IName 槽实现类体（部分闭合：装载后对名称对象 QI IFeatureClass `{D4803EE6}`，成功写 this+8 失败软置空；实现类分布仍未观察——样本 0 书签，见 legend-family 记录第十节）。 |
| 178 | FeatureQueryName/TableQueryName 的 IVariantStream 体（GdbCore，IPersistVariant 形态，主虚表 +0x14 GetIDType 写出各自 CLSID）。 |
| 179 | ObjectClassName 实现类 RTTI 定位（GdbCore 静态未找到）。 |
| 210 | 颜色与符号标量字段业务语义（DisplayCore 两轮各类，按内容候选标注）。 |
| 212 | 定性三处 short「装载后被 `GetDefaultRotateWithTransformValue` 覆盖」的「读取即弃」语义。 |
| 213 | LegendClassFormat 标量字段业务语义。 |
| 214 | HorizontalLegendItem 标量字段业务语义。 |
| 215 | Legend 共同前缀各 short/int32 字段业务语义。 |
| 216 | ElementProperties 字段业务语义。 |
| 217 | TextSymbol 字段业务语义。 |
| 218 | FeatureDatasetName 字段业务语义。 |
| 219 | TopologyName 字段业务语义。 |
| 220 | MapSurround 共同前缀字段业务语义。 |
| 221 | 书签共同前缀字段业务语义。 |
| 222 | FDO 旧加载器接口槽业务语义。 |
| 223 | ALPC 标量与字符串业务语义（标注表达式、类名、OID 等）。 |
| 224 | LELP 标量与字符串业务语义（标注表达式、类名、OID 等）。 |
| 225 | ETM 标量与字符串业务语义（标注表达式、类名、OID 等；尾字节样本 0x01）。 |
| 226 | FeatureIDSet 标量与字符串业务语义（标注表达式、类名、OID 等）。 |
| 230 | 定性 NumberFormat 九类的标量字段语义。 |
| 231 | 定性 NumberFormat 九类的 String 字段语义。 |
| 233 | Geometry 记录类型值的枚举映射语义（内部 shape 类别 vs esriGeometryType）。 |
| 239 | `{97D5B914}` 未定名 IID。 |
| 240 | 其余未定名 IID（上述之外）。 |
| 241 | 定性 B 轮 IDatasetName 六类标量/String 字段与五条请求 IID 业务语义（`{4BA33331}`/`{2E060684}`/`{FDFEBD93}`/`{F7AD47D1}`/`{4434D680}`，OleDB 尾段族类型库定名未做），见 [研究记录](docs/arcmap-10.1-dataset-name-slots.md)。 |
| 242 | 定性 C 轮装饰物族（Scalebar 族/MarkerNorthArrow/MapTitle/LegendFormat/LegendItem 三类/Patch 两类）标量字段业务语义；ScaleLine 族三个共同前缀调用者（`0x793c8990`/`0x7943b2f0`/`0x79448500`）未反编译。 |
| 243 | 定性 F 轮 PCS/GCS/Unknown 域串与 Unknown 四串业务语义及填充来源；ISRPersistStream slot8（恒 0x38）成员名；GCS `对象+0xac` 非零守卫语义；容差公式 pe/sg 数值推导链，见 [研究记录](docs/arcmap-10.1-pcs-geometry-slots.md)。 |
| 244 | 定性 G 轮 RandomColorRamp 读即弃 int32、PictureFillSymbol 读后即弃 double×2、MultiPart 读写门不对称等反编译级观察（与 round2 三处 short quirk 同族），见 [研究记录](docs/arcmap-10.1-display-slots-round3.md)。 |
| 262 | LineElement 体（样本 0 实例）。 |
| 270 | 定性 StdPicture 第二字及解码器分派；已验证介质 BMP 体第二字为 0，见 [样本记录](docs/mxd-round2-symbol-ole-20261117.md)。 |
| 272 | BOLP/UID/HLEP/DEP 前缀的样本结构级证据升级（Load 反编译唯一定位）。 |
| 290 | 定性 FeatureLayer `+0x1f4` 槽字段语义（候选「实际绘制/缓存范围」，原「全范围 vs 当前范围」二选一提法作废，缺类型库成员级命名与绘制路径取证）。 |
| 291 | 取证 `+0x1b4` 槽（版本>8）无值行为。 |

## v0 旧对象介质筛查

续查在仓库参考样本、历史提取目录以及本机 ArcMap 10.1 介质研究目录中，
按 SHA-256 去重了 106 份 MXD/MXT。用项目的 `extract` 命令分别提取
`/Maps` 和 `/PageLayout`，每种流各成功 104 份、提取失败 2 份。
只能对成功提取的对象流给出以下筛查结果。

| 类 | CLSID 字节命中（后接至少 6 字节） | 按 CLSID + 键之后的 short 筛出 v0 |
| --- | ---: | ---: |
| Envelope | 1396 | 0 |
| Hyperlink | 0 | 0 |
| PropertySet | 4 | 0 |

扫描将 CLSID 后的 4 字节视作候选对象键，之后的 2 字节视作候选版本。
命中数包含回引及可能的误命中，只用于筛选待按宿主解析的候选。
该方法未覆盖省略普通对象头的内联体或其他子流，也未经 ArcMap/QGIS
人工验收。三项真实旧档待办保持开放。
