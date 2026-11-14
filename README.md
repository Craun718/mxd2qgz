# mxd2qgz

面向 ArcMap MXD 到 QGIS 工程转换的实验工具。目前处于 MXD 格式研究阶段。

## 适配目标

适配目标为 **ArcMap 10.1**，与旧版软件保持一致。后续 REA 研究、MXD 解析实现和源工程人工验收均以 10.1 为基准。已有 10.8 原生研究及显式 10.8 读取器保留原版本范围，需经 10.1 原生实现复核后才能用于目标版本；当前尚未验证 10.1 转换能力。

本机安装介质目录为 `/home/gxrs/下载/arcmap10.1/ar10.1`，Desktop 安装入口为 `Desktop/setup.msi`，配套载荷为 `Desktop/Setup1.cab` 和 `Desktop/Setup2.cab`。已读取 `Desktop/Setup.ini`：`ProductName=ArcGIS 10.1 for Desktop`、`ProductVersion=10.1.3035`、`ProductLanguage=English`。这些是安装配置声明；尚未安装或进行 ArcMap 人工验证。安装介质仅作为本地研究来源，不纳入仓库。

已从 `Setup1.cab` 提取并核对五个原生 DLL 的自身版本、身份声明和摘要，并定位初步 REA 入口；细节和未解决事项见 [10.1 原生文件基线](docs/arcmap-10.1-native-baseline.md)。

Rust 库的 `native_string::read_string_10_1` 可读取 10.1 AfCore `String::Write` 普通输出的严格子集；调用者须先确认字段使用该布局。它经 `map_object::read_map_prefix_10_1` 的名称读取接入 `parse` 命令，不识别版本。

`object_stream::read_header_10_1` 可在调用者已确认的对象起点严格读取局部对象头，并保留未知类标识与键；它不定位对象、不解析对象体，已通过 `maps_stream` 与 `map_object` 接入 `parse` 命令的对象头读取。具体证据和短读限制见上述基线。

`version_stream::read_arcmap_header_10_1` 严格读取 ArcMap 10.1 的 Version 头部：文本、有符号 short 及条件出现的对象流表计数，并保留未识别的尾部；`write_arcmap_header_10_1` 根据明确提供的保存索引、文本和对象计数构造头部。它们不模拟原生短读，也不读取完整 MXD。原生证据及适用边界见上述基线。

`maps_stream::read_root_prefix_10_1` 可在调用者已确认的 10.1 分流 `/Maps` 起点严格读取 4 字节有符号数量、首个对象头和循环后的 4 字节尾部值（保留原始位模式），拒绝负数及超过限额的数量，尾部值之后的字节保留为未知尾部；它不识别版本、不解析对象体、不读取完整对象循环，已由 `mxd_maps::parse_maps_10_1` 接入 `parse` 命令。写侧与读侧证据及适用边界见 [10.1 对象体持久化分支与 /Maps 起点](docs/arcmap-10.1-object-body.md)。

`spatial_reference::read_projected_coordinate_system_10_1` 可在调用者已确认的 ProjectedCoordinateSystem 对象头之后按原生 Load 的版本分支严格读取对象体：保存版本、ASCII WKT、四个域字符串与垂向标志、精度标志与十个 double 均原样保留，版本大于 7 拒绝。它不定位对象、不解释 WKT 之外字段的语义，已由 `map_object::read_map_prefix_10_1` 的嵌套空间参考读取接入 `parse` 命令（WKT 写入中间模型，其余字段不写入）。证据及开放问题见 [10.1 空间参考持久化](docs/arcmap-10.1-spatial-reference.md)。

`map_object::read_map_prefix_10_1` 可在调用者已确认的 Map 对象头之后严格读取 10.1 Map 对象体的已确认前缀：版本 short（大于 51 拒绝）、两个 AfCore String 布局的名称字符串、嵌套空间参考对象（ProjectedCoordinateSystem 分派，全零 GUID 视为空引用标记）；第 5 步起的字段保留为未知尾部。它不读取图层循环，已由 `mxd_maps::parse_maps_10_1` 接入 `parse` 命令。证据及开放问题见 [10.1 Map 对象结构](docs/arcmap-10.1-map-object.md)。

`mxd_maps::parse_maps_10_1` 把上述显式 10.1 读取器接入中间模型：数据框名称与第二名称、嵌套 PCS 的 ASCII WKT 写入 `model::Map`/`model::DataFrame` 的已确认字段；对象循环边界未闭合，无论数量多少最多解析首个 Map 对象，其余数据框、图层、显示范围、工程名称与已确认前缀之后的字节全部保留为未知字段并在诊断中说明；PCS 域字符串、标志与 double 已解析但语义未确认，不写入模型。解析结果是候选观察，未在 ArcMap 10.1 或 QGIS 中人工目视验证。

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

`inspect` 列出 OLE 容器的数据流和字节数；`strings` 扫描 `/Maps` 中两种对齐方式的 UTF-16LE 候选字符串，并输出相对于该流的十六进制偏移。扫描不是对象解析，可能存在乱码、截断或误报，不能据此判断图层数量、完整路径或坐标系。`extract` 导出指定数据流，不会覆盖已有文件。`hex` 查看指定流的字节范围，偏移和长度支持十进制或 `0x` 十六进制，范围超出流时返回错误。`compare` 比较两个文件中指定流的相同偏移，输出不同字节的半开区间及流长度；插入或删除字节会使后续数据错位，差异区间不表示对象边界。`version` 读取 `/Version` 并按 ArcMap 10.1 头部规则与已知文本、short 集合输出识别候选，并按静态闭合的「文本↔保存索引↔short」映射（见 10.1 原生基线 2026-11-14 节）给出候选保存索引（即 esriArcGISVersion 枚举值）；文本与 short 须命中同一索引才给出候选，索引 3（9.3）因资源文本异常无法由文本命中，未知文本不映射为 9.3。10.1 规则不满足时改报 10.8 局部布局候选；全部结果都是候选，映射未经运行时确认，未在目标软件中验证。`parse` 读取 `/Maps` 并按显式 10.1 已确认前缀填充中间模型：输出数据框数量候选、首个数据框的名称/第二名称/CRS WKT、未知字节的流内偏移与长度及逐条诊断；对象循环未闭合，最多解析首个对象，解析失败（负数量、非 Map 首对象、截断等）返回明确错误而不输出部分模型。解析结果未在目标软件中验证，不能据此判断图层树或转换能力。
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
2. 解析地图、图层树、数据源、坐标系与显示范围，建立有明确未知字段的中间模型。
3. 使用 Rust 直接写入 QGS XML 和 QGZ ZIP，QGIS 用于输出基准和验收；逐步覆盖符号、标注和布局，报告不支持的对象与断链数据源。
4. 对照 ArcMap 原始工程和 QGIS 输出，验证数据、图层顺序及视觉效果。

MXD 通常引用外部空间数据；仅有 MXD 时，数据源未必可访问。原始路径可能包含敏感信息，分享诊断结果前请检查。
