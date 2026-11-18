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
