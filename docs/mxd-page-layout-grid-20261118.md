# Graticule、定位框与填充轮廓

本轮以 [10.1 页面元素接线](mxd-page-layout-surround-20261118.md)的四个实档停止点为起点，核对原生持久化调用和实档边界。涉及 10.1.0.3035 的 MapElements.dll（SHA-256 `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`），用 PE 原始 RTTI、虚表、导入和 `objdump -d -Mintel` 复核；接口 GUID 与 10.1 esriCarto、esriDisplay、esriSystem 类型库互证。地址为静态映像 VA。未进行 ArcMap 或 QGIS 人工验收。

| 结构 | 原生读侧依据 | 读出的内容 |
| --- | --- | --- |
| Graticule | GetClassID `0x79559100` 从 `0x7959b3dc` 取 CLSID；Load `0x79559140` 的 `0x79559171` 拒绝版本 >5 | 地图网格基体后，short、4 个 double、LONG；v2 加 short，v3 加两个 double、线符号、标记符号、double、short，v4 加 double，v5 加 short 和两个 double。部分 double 经同一通用读函数槽读 8 字节，按原始位保留。 |
| MapGrid | Graticule `0x79559191` 委派 `0x79576ec0`，后者版本上限 1 | 名称、线符号、标记符号、IGridLabel、线符号、double、short、线符号、double、4 个 short、IMapGridBorder。 |
| DMSGridLabel | GetClassID `0x795513c0` 取 `0x7959ac60`，Load `0x79551400` 版本上限 4 | LONG、3 个 ITextSymbol、2 个 short、ILatLonFormat、double；v2 加 10 字节，v3、v4 各再加一个 short。经纬度显示属性未从标量位推定。 |
| SimpleMapGridBorder | 持久化虚表 `0x7959dca0` 的 Load `0x7958ee10` 和 Save `0x7958ee90` | short 版本（上限 1）、一个 ILineSymbol，具体实现仍由流内 CLSID 决定。 |
| LocatorRectangle | GetClassID `0x7956ab00`，持久化虚表 `0x7959c0e8` 的 Load `0x7956ab40` | short 版本（上限 3）、IMapFrame、IBorder、IBackground、ISymbol、short；v2 加 IShadow，v3 加 short。作用于已登记的 MapFrame 回引；未把新 IMapFrame 的递归分支视作已闭合。 |

持久化体复用既有符号、颜色、字体、经纬度格式和边框读取器。未知新类、未知版本、计数或长度越界仍使整个有界页面元素回滚，不登记尚未解析的新键。Graticule 的工作区入口为 `src/maps/map_grid.rs`；LocatorRectangle 由 `src/maps/map_frame.rs` 在实际宿主的 ILocatorRectangle 循环内分派。

- **MilitaryPortrait.mxd**：首个曾失败的 Graticule 键 159 位于 `/PageLayout:0x10d4`，Graticule v5 委派 MapGrid v1，名称为 Grid，DMSGridLabel v4 为键 162；后续另一网格的边框键 192 位于 `0x19df`，其 ILineSymbol 为已知 CartographicLineSymbol。定位框键 211 在 `0x221c`，内嵌 IMapFrame 回引键 158。11 个页面元素读至 `0x288d`，保留其后 607 字节。MilitaryPortrait.mxt 同样读完 11 项，尾段从 `0x2937` 保留。
- **ConservationLandscape.mxd**：原停止点为 `/PageLayout:0x161e` 的 RectangleElement：MultiLayerFillSymbol 键 125 → SimpleFillSymbol 键 127 → MultiLayerLineSymbol 键 128（`0x178d`）。SimpleFillSymbol 的原生 ISymbol 请求和已有 MultiLayerLineSymbol 读写证据见 [DisplayCore 符号记录](arcmap-10.1-display-symbols-round2.md)。修正轮廓槽使用早期分派、找不到已支持类的问题，继续读取后续填充色和标量。该工程读完 9 项到 `0x18a5`，.mxt 也读完 9 项到 `0x1835`。同一变更让 SDK Portland 的第 14 个 FeatureLayer（`/Maps:0x6af1d`）读到全部四段 TimeData；旧测试只覆盖此前可读的 13 项。
- **10.1 介质版本候选分组**：66 份中，能进入页面元素循环的 48 份现全部读完集合（此前 44 份）。按 `/Version` 头的文本候选分组，9.2 为 3/3，9.3 为 34/34，10.0 为 11/11；这些工程都检测到至少一个页面地图框回引。候选文本为 10.1 的 18 份均因 /Maps 前置未知内容未进入页面元素循环，目前主要见于 Basemap 和 MapSurround 子体；未由这批样本证明 10.1 流类别的全覆盖。10.8 介质仅作回归对照，不据此确认 10.8 原生实现。

`cargo fmt --check`、`git diff --check` 与 `cargo test --no-fail-fast -- --include-ignored` 通过（887 项）。新增合成覆盖 Graticule v0–v5 门、DMS 标签、简单网格边框，以及逐字节截断后的游标和键表回滚。实档测试覆盖两条此前失败的宿主路径。从布局元素集合结束到 PageLayout 根对象结束的字段，以及其他网格、标签、边框实现仍待研究。
