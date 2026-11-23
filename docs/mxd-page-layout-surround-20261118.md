# 装饰框、矩形和页面元素回引

下文保留本轮的取证边界。后续已读通 MilitaryPortrait 和 ConservationLandscape 的页面元素，见[网格与定位框复核](mxd-page-layout-grid-20261118.md)。

从已有 [MapFrame、BaseFrameElement 与 PageLayout 接线](mxd-map-frame-body-20261118.md)继续。本轮用原始二进制的 RTTI、虚表、导入表和 `objdump` 指令复核读写顺序，使用实档的有界元素循环验证；偏移均为未重定位映像 VA 或注明的数据流内位置。没有运行 ArcMap 或 QGIS。

| 10.1.0.3035 模块 | SHA-256 |
| --- | --- |
| MapElements.dll | `67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768` |
| GraphicElements.dll | `f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9` |
| Layer.dll | `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8` |
| LayerLib.dll | `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02` |

- **MapSurroundFrame**：MapElements RTTI 的持久化虚表 `0x7959cf98` 的 GetClassID/Load/Save 为 `0x7957b840/0x7957b870/0x7957bb20`。GetClassID 从 `0x7959bebc` 取 CLSID `{83FFCAE1-EDCA-11D0-8683-0000F8751720}`。Load 读 short 版本（上限 3），在 `0x7957b8f9/0x7957b95f` 请求 IMapSurround 和 IMapFrame。版本 <2 在 `0x7957b997` 请求 IGeometry，否则在 `0x7957ba14` 请求 IEnvelope。接着 `0x7957ba42` 经导入调用 LayerLib BaseFrameElement Load，最后版本 ≥3 再读两个 double（`0x7957ba70/0x7957ba87`）。两个 double 保留原始位，业务名称未确认。IIDs 来自 `0x7959d2fc/0x7959c180/0x7959c120/0x7959b5cc`，用 esriCarto 10.1 类型库核对。
- **内联和跨流对象**：装饰框中的新 IMapSurround 复用 Map 对象循环的类分派；如果遇到不完整前缀，整个布局元素回滚，而 Map 原有宿主仍可保留读过的前缀。IMapFrame 可以是新 MapFrame；其 IMap 回引须匹配已解析的根 Map 键。页面元素本身也可为空或回引，读取器要求该头恰好占满对应的 ExtraSafe 范围；否则保留整个条目，不据大小跳过未知体。
- **RectangleElement**：GraphicElements RTTI 持久化接口表 `0x750724f4` 的 GetClassID `0x75059b50` 从 `0x750710a8` 写出 `{3A9767C2-F253-11D0-83A4-080009B996CC}`，Load 为 `0x7505a400`。短版本上限 1；随后经 `0x7506e354/0x7506e350` 依次调用 LayerLib RectShapeElement 和 ElementProperties 的 Load，最后 `0x7505a484` 读取 IFillSymbol（常量 `0x7506ebcc`，esriDisplay TypeInfo 45）。RectShapeElement 即上一轮地图框共用的 LayerLib `0x786d00d0`。这里只接线到页面元素读取；未知填充符号或嵌套类仍使条目回滚。
- **FrameElement**：10.1 介质 `esri.clsid.xml` 将 `{E01BA2C5-24B2-11D3-B8AA-00600802E603}` 映射到 Layer.dll，原文件在 `0x7866e160` 有该 CLSID。其 RTTI 持久化虚表 `0x7866df48` 的 Load/Save 为 `0x7865b9c0/0x7865b9d0`，分别直接跳到导入 `0x7866d2a0/0x7866d29c` 的 LayerLib BaseFrameElement Load/Save，没有另加 FrameElement 自有持久化字段。

对 66 份 10.1 介质工程回归，48 份进入页面元素集合，其中完整读完集合的工程由 34 份增至 44 份；当前总共顺序读到 170 个页面元素，其中 28 个是装饰框。这些数字不含未解析的 PageLayout 尾段。Utilities.mxd 的 27 个元素全部读到 `/PageLayout:0x3669`，其中有五个 MapSurroundFrame 和共享地图框回引。三个内联 MapFrame 的键 165、175、183 在 `/PageLayout:0x66/0x491/0x7c8` 分别回引 `/Maps` 的 Map 键 29、85、124；末尾另保留 671 字节。合成用例覆盖装饰框 v0–v3 版本门、空槽和有界回引，及回引多出字节时拒绝消费条目。

尚未完成：MilitaryPortrait 的两个介质条目停在 Graticule 网格对象；ConservationLandscape 的两个条目停在矩形填充符号的子对象。另有 18 份工程因前置 /Maps 边界原因没进入布局；普通对象形式的旧布局元素、布局尾段和其他未知跨流对象体仍待闭合。原生读写分支及介质回归不能替代 ArcMap/QGIS 人工验收。

## 组元素中的装饰框

GraphicElements.dll 10.1.0.3035 的 GroupElement Load `0x75035ef0` 在子元素循环中使用普通 IElement 对象槽，见 [GroupElement 持久化顺序](mxd-page-layout-tail-20261118.md)。MapSurroundFrame 的类身份及生效的 Load 入口见上文。分组宿主现在使用同一个 MapSurroundFrame 读取器，并把布局起始偏移、跨流 Map 检查及对象键表传给子项。没有额外的 ExtraSafe 头。构造测试覆盖同一布局后续元素对组内装饰框的回引，以及子体截断时整组回退。

完成 v51 Map 段落文本接线后，10.8 补充对照中有八份工程在布局组内实际遇到该类。复用已核实的 10.1 对象体分派后，八份均到达布局终点，布局总数由 41/68 增至 49/68。66 份 10.1 介质工程和仓库样例仍到达布局终点。余下 19 份 10.8 对照工程的首个未支持类分别是 EmfPictureElement（9 份）、BmpPictureElement（9 份）、JpgPictureElement（1 份），由 10.1 介质类表确认类名，图片体未在本次读取。未做 ArcMap 或 QGIS 人工验收。

## 图片元素的普通对象体

使用 10.1 Desktop 介质的 x86 `RasterGraphicElements.dll`，版本资源为 `10.1.0.3035`，SHA-256 为 `a33a04b3a190074b1aa5291ab726825b9ab454914dafad81beeb7e595591e40d`。现有源码成果未导出此模块，本轮使用 PE 类注册、RTTI、虚表及 `objdump -d -Mintel` 对位。通用 PictureElement、Emf、Bmp、Jpg、Gif、Tif、Png、Jp2 八类的 `+0x9c` 持久化视图都由虚表 `+0x14` 的 thunk `0x7c5948c0` 进入 `0x7c593450`，`+0x18` 则从 `0x7c594b30` 到 Save `0x7c593780`。

| Load 原生地址 | 存盘字段 |
| --- | --- |
| `0x7c593474..0x7c5934e6` | short 版本，0–2 读 RectShapeElement 和 ElementProperties，3–9 读 BaseFrameElement。构造入口在 `0x7c591404` 将普通对象的另一路兼容标志置零。 |
| `0x7c5934e6..0x7c593569` | v2 起读嵌入 short。其值为 `0xffff`，或版本 ≤4 且 ElementProperties 名称为空时，以普通 LoadObject 读取 IPicture（`0x7c59e42c` 的 IID `{7BF80980-BF32-101A-8BBB-00AA00300CAB}`）。v0–3 再读旧 short。Save 在 v4 写 BaseFrameElement 前，将图片路径复制到该旧名称字段。 |
| `0x7c593569..0x7c5936c7` | v5 加三个 AfCore String；v6 加 32 字节 WKSEnvelope；v7 加 String；v8 加 short 和 String；v9 加 String。Save 版本表 `0x7c59e578` 为 `[4,4,4,4,5,9]`。 |

读取器复用几何框架、ElementProperties 及有长度的 StdPicture 体。仅在对象完整消费后提交游标和登记表。通用元素分派覆盖布局顶层、布局组内、图形子层和元素集合。图片按原始字节保留。类名仅说明元素类，例如 CMY_PlotterCalibrate 的 JpgPictureElement 内嵌 StdPicture 的有效载荷以 BMP 标识开头。外部图片路径只作为字符串读取。特殊旧兼容路径 `0x7c593180`、带外部状态的宿主、其他新 IPicture 类及无长度头的 StdPicture 尚未接入。

构造测试覆盖普通 v0–v9、旧名称门、八类分派、图片与元素回引、全部截断边界、过大声明和未知 IPicture 的回退。实档回归中，10.8 介质 19 份原停止工程各读出一项 v4 图片元素（EMF 9、BMP 9、JPG 1）。按根尾偏移加四字节和布局实际消费终点分别与流长度比较，10.8 的 68 份、10.1 的 66 份及仓库样例的两个流都到达精确终点。该检查覆盖对象流边界，未知的有界子对象仍可能保留。ArcMap 和 QGIS 人工验收未做。

MapElements.dll 10.1.0.3035 的 `0x7957b95f` 请求 IMapFrame，
SystemCore 同版在 `0x7ffd0063` 按键查询登记实例。装饰框按登记类
识别回引。构造用例将 IMapFrame 头指向此前登记的 MapSurroundFrame，
验证保留整个 ExtraSafe 元素及后续字节。Utilities 实档继续用于复核
内联地图框与跨流 Map 回引。ArcMap 人工验证尚未进行。

MapElements.dll 10.1.0.3035 的 objdump 指令确认版本上限与后缀均
按有符号 short 比较。`0x7957b8ac / 0x7957b8b1` 拒绝大于 3，
`0x7957b96b / 0x7957b97c` 在版本不小于 2 时进入 IEnvelope 分支，
`0x7957ba58 / 0x7957ba5c` 在版本小于 3 时跳过两个 double。
合成布局验证负值选择旧 IGeometry 槽、保留版本位模式以及后继元素。
真实负版本装饰框仍待 ArcMap 10.1 验证。
