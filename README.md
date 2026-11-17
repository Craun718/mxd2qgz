# mxd2qgz

面向 ArcMap MXD 到 QGIS 工程转换的实验工具。目前处于 MXD 格式研究阶段。

## 适配目标

适配目标为 **ArcMap 10.1**，与旧版软件保持一致。后续 REA 研究、MXD 解析实现和源工程人工验收均以 10.1 为基准。已有 10.8 原生研究及显式 10.8 读取器保留原版本范围，需经 10.1 原生实现复核后才能用于目标版本；当前尚未验证 10.1 转换能力。

本机安装介质目录包含 Desktop 安装入口 `Desktop/setup.msi`，配套载荷为 `Desktop/Setup1.cab` 和 `Desktop/Setup2.cab`。已读取 `Desktop/Setup.ini`：`ProductName=ArcGIS 10.1 for Desktop`、`ProductVersion=10.1.3035`、`ProductLanguage=English`。这些是安装配置声明；尚未安装或进行 ArcMap 人工验证。安装介质仅作为本地研究来源，不纳入仓库。

已从 `Setup1.cab` 提取并核对五个原生 DLL 的自身版本、身份声明和摘要，并定位初步 REA 入口；细节和未解决事项见 [10.1 原生文件基线](docs/arcmap-10.1-native-baseline.md)。

Rust 库的 `native_string::read_string_10_1` 可读取 10.1 AfCore `String::Write` 普通输出的严格子集；调用者须先确认字段使用该布局。它经 `map_object::read_map_prefix_10_1` 的名称读取接入 `parse` 命令，不识别版本。

`object_stream::read_header_10_1` 可在调用者已确认的对象起点严格读取局部对象头，并保留未知类标识与键；它不定位对象、不解析对象体，已通过 `maps_stream` 与 `map_object` 接入 `parse` 命令的对象头读取。具体证据和短读限制见上述基线。

`version_stream::read_arcmap_header_10_1` 严格读取 ArcMap 10.1 的 Version 头部：文本、有符号 short 及条件出现的对象流表计数，并保留未识别的尾部；`write_arcmap_header_10_1` 根据明确提供的保存索引、文本和对象计数构造头部。它们不模拟原生短读，也不读取完整 MXD。原生证据及适用边界见上述基线。

`maps_stream::read_root_prefix_10_1` 可在调用者已确认的 10.1 分流 `/Maps` 起点严格读取 4 字节有符号数量、首个对象头和循环后的 4 字节尾部值（保留原始位模式），拒绝负数及超过限额的数量，尾部值之后的字节保留为未知尾部；它不识别版本、不解析对象体、不读取完整对象循环，已由 `mxd_maps::parse_maps_10_1` 接入 `parse` 命令。写侧与读侧证据及适用边界见 [10.1 对象体持久化分支与 /Maps 起点](docs/arcmap-10.1-object-body.md)。

`spatial_reference::read_projected_coordinate_system_10_1` 可在调用者已确认的 ProjectedCoordinateSystem 对象头之后按原生 Load 的版本分支严格读取对象体：保存版本、ASCII WKT、四个域字符串与垂向标志、精度标志与十个 double 均原样保留，版本大于 7 拒绝。它不定位对象、不解释 WKT 之外字段的语义，已由 `map_object::read_map_prefix_10_1` 的嵌套空间参考读取接入 `parse` 命令（WKT 写入中间模型，其余字段不写入）。证据及开放问题见 [10.1 空间参考持久化](docs/arcmap-10.1-spatial-reference.md)。

`map_object::read_map_prefix_10_1` 可在调用者已确认的 Map 对象头之后严格读取 10.1 Map 对象体的已确认前缀：版本 short（大于 51 拒绝）、两个 AfCore String 布局的名称字符串、嵌套空间参考对象（ProjectedCoordinateSystem 分派，全零 GUID 视为空引用标记）。证据及开放问题见 [10.1 Map 对象结构](docs/arcmap-10.1-map-object.md)。

`layer_list::read_layer_list_10_1` 严格读取 Map 图层列表（Map.dll `FUN_794035f0`，架构版本 ≥0x17）：int32 计数、逐条目 int16 标志 1/2/3（其他值原生拒绝）、ExtraSafe 8 字节 `[A][B]` 前缀（`A==0&&B==0` 为空条目；`B≠0` 按原生 Seek 同步到前缀后+B，`B==0` 按「体长 = A−20」的写侧观察推定跳过）、条目经 `object_refs` 键登记表分类、循环后 int32 活动图层索引。新对象体按流内类标识分派给 FeatureLayer / GroupLayer / CompositeGraphicsLayer / FDOGraphicsLayer 的前缀读取器，其余类（如 RasterLayer）按有界未知保留；标志 1 新对象体无外部长度，各读取器在首个未闭合字段处停止并保留剩余（GroupLayer 子图层循环支持递归解析，深度限额 64 为防栈溢出，非原生限制）。`map_object::read_map_suffix_10_1` 继而读取后缀序列：基本图形层槽位（CompositeGraphicsLayer 前缀）、MapSurround 计数循环（Legend 新对象经 `map_surround` 前缀读取器解析，体完整消费时循环继续）、WKSEnvelope 32 字节显示范围（`[XMin,YMin,XMax,YMax]`，写方向确定）、两个 int32、IEnvelope 槽位（Envelope 读取器整体消费）、书签计数循环（AOIBookmark/FeatureBookmark 新对象体完全闭合，读取成功即循环继续；其余类或体内停止在该条目停止），到书签循环后读取已确认的属性前导（原始标量和坐标变换计数），从首个未解析字段开始保留未知尾部。A 前缀推定、空标记消费长度等开放项见上述文档。

`feature_layer::read_feature_layer_prefix_10_1` 严格读取 FeatureLayer 对象体的已确认前缀（FeatureLayer.dll `0x6f6e9940` 头部段）：版本 short（大于 34 拒绝）、双名称、VARIANT_BOOL 可见性（非规范位模式原样保留）、嵌套数据源（空标记 / 回引 / FeatureClassName 前缀：short + 要素类名）。FeatureClassName 体（GdbCore.dll `0x709b5080`：版本 short、四个 String、int32×2、IWorkspaceName/IDatasetName 选择子、版本>1 的 ITopologyName 选择子）与嵌套 WorkspaceName 体（`0x70a27950`：版本 short、路径与类型 String、IPropertySet 槽、工厂标志、原样工作空间工厂 CLSID、int32）已闭合；选择子分支以 FeatureDatasetName / TopologyName 新对象出现时经 `dataset_name` 读取其闭合体（样本未出现，无字节佐证），其他类在其对象头处停止；渲染器槽（IFeatureRenderer）的 SimpleRenderer 新对象经 `feature_layer_tail` 读取其闭合体（含 SimpleMarkerSymbol/RgbColor/LegendGroup/LegendClass 嵌套链，ISymbol/IColor 槽按类标识分派六类闭合符号与五类闭合颜色，未知类维持停止回退），其他渲染器类在其对象头处停止；SimpleRenderer 体完整消费或空/回引时读取 `+0x144` 原样 GUID，随后的后继字段段经 `feature_layer_tail::read_feature_layer_successor_10_1` 闭合解析（`+0x154` 字符串、double/short 标量、IHyperlink 循环、`{3036D35D}` 计数循环、IColor 槽、两 String、int32、FieldInfo 循环含 NumericFormat 体；样本 `0x5b0..0xafe` 逐字节吻合），在后继段 TailCompleted（标注属性环至 DEP 体均闭合解析，DEP 体完整消费——A 轮裁决其无尾段）处成功停止，其后字段保留为剩余字节。样本 `0x30a..0x3ed`（头部段至渲染器链起点）逐字段吻合（辅助观察）。

`group_layer::read_group_layer_prefix_10_1` 与 `composite_graphics::read_composite_graphics_layer_prefix_10_1` 分别严格读取 GroupLayer（版本 ≤12：名称、short×3、double×2、子图层循环（<11 普通 / ≥11 ExtraSafe）、版本条件的可见性与嵌套对象）与 CompositeGraphicsLayer（版本 ≤6：元素组循环、名称、标量组、子图层循环 `[String + ILayer]` 带条目尾段）的已确认前缀，在首个不可闭合字段处停止并保留剩余；字段序按反编译一手证据实现，修正了研究文档表格的三处出入（嵌套对象门 `2 < ver`、CGL 第 6 步三个 int32、子图层条目尾段）。子图层循环支持递归解析（<11 普通条目、≥11 ExtraSafe 条目经类分派与 A 界跳过）；CGL 子图层计数经样本裁决位于 `0x17056`（`0x17052` 为版本>1 的 int32）。

`fdo_graphics_layer::read_fdo_graphics_layer_prefix_10_1` 严格读取 FDOGraphicsLayer 对象体（AnnoLayer.dll `0x6b1955a0`，经 RTTI→COL→虚表 +0x14 定位，GetClassID 写出 `{34B2EF83-…}` 三方互证）：版本 short（有符号 >11 拒绝，<5 停止保留）、int32/int16 标量、关联要素图层槽（ILayer）、ElementCollection 槽（IElementCollection2：ElementCollection 新对象经 `element_collection` 整体消费，其他类停止）与无对象头的委派子对象体（this+0xbc 虚表 +0x14 直调，构造函数闭合为聚合 FeatureLayer `{E663A651}` 实例的 IPersistStream::Load，样本按 FeatureLayer 字段序逐字段吻合）；两个内嵌 FeatureLayer 体在其前缀停止点终结前，尾段（颜色/符号集合、元素循环）结构性不可达，保留为剩余。样本 `0x28e..0x2d22` 已确认字段逐字节吻合（辅助观察）。证据与开放项见 [10.1 Map 对象结构](docs/arcmap-10.1-map-object.md)。

`envelope::read_envelope_10_1` 严格读取 Envelope 对象体（Geometry.dll Save `0x71869da0` / Load `0x71869b60` 反编译）：版本 0/1/2/3 分支（v0 与 v2 同布局，尚无 v0 真实旧档佐证）、32 字节 `[XMin,YMin,XMax,YMax]`（写方向确定）、int32 标志低字节 bit6/bit7 追加的 Z/M double 对、版本 3 的嵌套空间参考（16 零空标记 / 键回引 / 内嵌 PCS 体）。

`feature_layer_tail::read_simple_renderer_body_10_1` 严格读取 10.1 SimpleRenderer 对象体的已确认前缀（FeatureLayer.dll `0x6f722b70`）：版本 ≤3、ISymbol 槽（SimpleMarkerSymbol，DisplayCore.dll：两个 IColor 槽只认 RgbColor）、ILegendGroup 槽（Layer.dll：LegendClass 循环，ISymbol 回引、ILegendClassFormat 新对象停止）、ILegendItem 新对象停止、版本 >2 的 RotationRenderer/SizeRenderer 尾段（LayerLib.dll）。样本 `0x3ed..0x602` 逐字段吻合（辅助观察）。证据与开放项见 [研究记录](docs/arcmap-10.1-featurelayer-tail.md)。

`element_collection::read_element_collection_10_1` 严格读取 10.1 ElementCollection 对象体（Layer.dll RTTI→COL→虚表定位，Load `0x7865a340`）：版本有符号 >4 拒绝；版本 <4 为三个顺序分离循环（元素 → int32 → 版本 ≥3 再 int32），版本 ==4 为逐元素交错 [IElement `{9B001319}` + int32 ×3]；新对象元素为唯一停止点（游标与登记回退）。样本 12 实例全版本 4，FDO 槽位键 86 的体恰 6 字节、终结于委派 FeatureLayer 起点（辅助观察）。见 [研究记录](docs/arcmap-10.1-element-collection.md)。

`dataset_name::read_feature_dataset_name_body_10_1` / `read_topology_name_body_10_1` 严格读取 FeatureClassName 体两选择子分支的新对象体（GdbCore.dll）：FeatureDatasetName `{198846CF}`（Load `0x709b7c90`：版本 ≤1 + String×4 + 1 字节标志 + IWorkspaceName 槽，WorkspaceName 体复用第六轮读取器）与 TopologyName `{9FCA50EC}`（Load `0x7095e5c0`：版本 ≤1 + String×2 + IDatasetName 槽，FeatureDatasetName 体递归）。参考样本 11 个 FCN 的选择子均为 0；安装介质模板 Africa 的非零选择子已用 Rust 局部读取到 FeatureDatasetName v1，TopologyName 分支仍无字节佐证。见 [研究记录](docs/mxd-dataset-format-evidence-20261117.md)。

`fdo_legacy::read_fdo_graphics_layer_legacy_prefix_10_1` 覆盖 10.1 FDOGraphicsLayer 版本 0..=3 旧加载器的已确认字段（无样本字节佐证）；版本 4 探测依赖整流访问与运行时游标效应，主读取器维持停止。`read_fdo_graphics_sublayer_10_1` 读取版本 ≥9 元素循环的 FDOGraphicsSublayer 体（`{4ABA3851}`）；参考样本中有一个内联实例，见 [FDO 续研记录](docs/mxd-layer-luna.md)。

`map_surround::read_aoi_bookmark_10_1` / `read_feature_bookmark_10_1` / `read_legend_prefix_10_1` 严格读取 Map 后缀两循环非空时的元素体（Map.dll 10.1.0.3035 反编译）：AOIBookmark（版本 ≤1 + IEnvelope + 名称）与 FeatureBookmark（版本 ≤1 + int32 + IName 条目 + 名称）体完全闭合，Legend 为已确认前缀（共同前缀 + ILegendFormat/ILegendItem 新对象停止）。样本两循环计数均为 0，全部结论无字节佐证；循环经 `map_object::read_map_suffix_10_1` 按类标识分派。见 [研究记录](docs/arcmap-10.1-mapsurround-bookmark.md)。

`mxd_maps::parse_maps_10_1` 将已确认的 10.1 读取器接入中间模型：数据框名称、第二名称、空间参考 WKT 与 PCS 原始字段、按条目顺序构造的图层节点及到达时的显示范围。图层 CRS 从 `+0x1f4` Envelope 提取，Map CRS 回引同时匹配键和类标识；未知内容按流内偏移保留。对象循环仍未闭合，最多解析首个 Map。参考样本首条 FDOGraphicsLayer 的关联、委派 FeatureLayer 与 FDO 后续字段已消费，包含一个内联子层；解析继续到六个图层条目与活动索引，其中两个栅格体按有界未知内容保留。基本图形层的 10 个子层及循环后尾段已消费，读到 `/Maps:0x24f64` 的显示范围；书签后属性前导从 `0x24fa0` 开始，参考文件的 IlluminationProps 从 `0x24fb2` 开始，未知尾部从 `0x24fe8` 开始。Map 架构版本小于 23 时只读取前缀，旧图层列表及其后数据保留为未知。证据见 [FDO 续研记录](docs/mxd-layer-luna.md) 和 [图层及标注续研](docs/mxd-layer-label-cgl-20261117.md)。另见 [第二轮复核](docs/mxd-round2-review-20261117.md)的实样验证与待办分类。结果未在 ArcMap 10.1 或 QGIS 中人工目视验证。

`maps::page_layout_prefix` 提供显式 10.1 PageLayout 元素列表前缀和有界元素体读取器；已从介质 `CentralUSA.mxd` 的页面根对象顺序读取线元素和先前定义的空间参考回引。BasemapLayer 已接入有界图层入口：介质 `Africa.mxd` 的首个图例项能回引基图内实际登记的图层键，未支持的剩余体按边界保留。见 [第三轮复核](docs/mxd-round3-review-20261117.md)。

## 使用

需要 Rust 工具链。运行：

```sh
cargo run -- inspect 'map.mxd'
cargo run -- strings 'map.mxd'
cargo run -- version 'map.mxd'
cargo run -- parse 'map.mxd'
cargo run -- extract 'map.mxd' /Maps maps.bin
cargo run -- compare 'before.mxd' 'after.mxd' /Maps
cargo run -- hex 'map.mxd' /Maps 0x860 96
```

`inspect` 列出 OLE 容器的数据流和字节数；`strings` 扫描 `/Maps` 中两种对齐方式的 UTF-16LE 候选字符串，并输出相对于该流的十六进制偏移。扫描不是对象解析，可能存在乱码、截断或误报，不能据此判断图层数量、完整路径或坐标系。`extract` 导出指定数据流，不会覆盖已有文件。`hex` 查看指定流的字节范围，偏移和长度支持十进制或 `0x` 十六进制，范围超出流时返回错误。`compare` 比较两个文件中指定流的相同偏移，输出不同字节的半开区间及流长度；插入或删除字节会使后续数据错位，差异区间不表示对象边界。`version` 读取 `/Version` 并按 ArcMap 10.1 头部规则与已知文本、short 集合输出识别候选；仅在文本与 short 精确配对时给出候选保存索引（即 esriArcGISVersion 枚举值）。`9.3`/16 可识别为索引 3，由原生 short 表和[安装介质样本](docs/mxd-install-media-samples-20261117.md)共同支持。10.1 规则不满足时改报 10.8 局部布局候选；全部结果都是候选，映射未经运行时确认，未在目标软件中验证。`parse` 读取 `/Maps` 并按显式 10.1 已确认前缀与后缀填充中间模型：输出 `/Version` 识别候选、数据框数量候选、首个数据框的名称/第二名称/CRS WKT、图层列表条目（标志、前缀、分类、键、体前缀）与活动图层索引、显示范围（到达时）、未知字节的流内偏移与长度及逐条诊断；对象循环未闭合，最多解析首个 Map。参考样本可读取六条图层记录、活动索引和首个数据框的显示范围；未支持的有界图层体和未解析的 Map 后缀保留为未知。解析失败返回明确错误。结果未在目标软件中验证，不能据此判断完整图层树或转换能力。
研究以 REA 对目标软件实现和运行行为的分析为主；可控对照样本仅作辅助证据。记录流内偏移和脱敏的方法见研究记录中的“对照样本采集流程”。

## Python 接口

需要 Python 3.8 或更高版本及 Rust 工具链。在项目根目录安装：

```sh
python -m pip install .
```

```python
from pathlib import Path
import mxd2qgz

path = Path("map.mxd")
streams = mxd2qgz.inspect(path)  # [(流路径, 字节数), ...]
maps = mxd2qgz.read_stream(path, "/Maps")  # bytes
```

接口只读取 OLE 数据流，不解析图层或生成 QGIS 工程。文件或流不存在时会抛出 Python 的 `OSError` 子类。构建可分发的 wheel 可运行 `maturin build --release`。

## Rust 构造模型输出

Rust 库提供 `qgis::to_qgs` 和 `qgis::write_project`，可从调用者构造的
`model::Map` 生成 QGS/QGZ，支持单数据框、分组、顺序、可见性、Shapefile 路径、
EPSG:4326/3857 与主画布范围。输出默认不覆盖已有文件，断链保留并诊断；
多数据框和不支持的数据源明确报错。`mxd_maps::parse_maps_10_1` 可从 10.1
`/Maps` 已确认前缀填充同一模型（见上文），但两者之间没有 `convert` 命令
连接，Python 接口不变。API 加载通过，人工目视验收未完成。

## 后续工作

1. 使用 REA 追踪原生文档打开、版本和对象持久化实现，确定对象边界、类型及版本差异；可控样本可辅助核对。
   AfCore.dll 10.1.0.3035 的函数级反编译产物已导出到 `temp/rea-decompile-20261117/AfCore/`，并在本机研究目录保留副本；覆盖、哈希与限制见 [对象体记录](docs/arcmap-10.1-object-body.md)。
2. 解析地图、图层树、数据源、坐标系与显示范围，建立有明确未知字段的中间模型。
3. 使用 Rust 直接写入 QGS XML 和 QGZ ZIP，QGIS 用于输出基准和验收；逐步覆盖符号、标注和布局，报告不支持的对象与断链数据源。
4. 对照 ArcMap 原始工程和 QGIS 输出，验证数据、图层顺序及视觉效果。

MXD 通常引用外部空间数据；仅有 MXD 时，数据源未必可访问。原始路径可能包含敏感信息，分享诊断结果前请检查。
