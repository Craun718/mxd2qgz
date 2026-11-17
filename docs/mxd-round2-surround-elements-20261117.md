# 第二轮：装饰、书签和元素

范围按分派时 TODO 原行号；不修改 TODO。现有反编译、文件身份与上一轮探针见 `mxd-surround-media-evidence-20261117.md` 和 `arcmap-10.1-legend-family.md`。以下没有 ArcMap/QGIS 人工验收。

## 本轮推进

FeatureLayer.dll 10.1.0.3035 的 CGL::Load `0x6f6b2100` 在元素组请求 IElement（`{9B001319}`）；GraphicElements.dll 的 TextElement、MarkerElement、LineElement 对象体已分别在 `arcmap-10.1-element-bodies.md` 定位。本轮将这三个精确 CLSID 接到该组，嵌套未知对象停止时保留已登记的元素键、元素体与内层停止原因；错误回滚游标及登记表。测试 `cgl_elements_round2` 覆盖三类空槽完整体、每一截断前缀和未知符号停止。

10.1 介质 Africa `/Version` 为 10.1/19 候选，文件 SHA-256 与上一轮记录一致。以 `cargo run -- parse temp/arcmap101-media-mxd/Africa.mxd` 从 Map 根读取，CGL 对象头 `0x6601`，其内 TextElement 头 `0x661f`；现 CGL 完整消费，进入 MapSurround 计数 3、首个 Legend 对象头 `0x9683`。在第一个 HorizontalLegendItem（头 `0x9a69`）的 ILayer 槽 `0x9a85` 停止，该槽类为 FeatureLayer，键 42。前方图层列表的对象体在 `0x220` 起有界保留未解析，故不能证明键 42 已注册：上一轮预登记键探针不能代替真实引用表。继续需要共享图层/Map 接线重建真实键状态，不得在此假定回引。完整输出在忽略的 `temp/round2-africa-parse.txt`，可能含路径。

既有 CGL 元素组还将 ElementCollection 当 IElement 分派，尚未证实其实现 IElement；本轮未改旧分支，须静态核查 QI/类型库后决定是否移除。第 7 步及子图层尾段请求 IElementCollection，与该疑问分开。

主审补充：另用 LandscapeClassic 和旧版 Africa 模板从 Map 根验证了 252、256–258、261；下表保留子代理最初定位的文件和停止点。见 [主审记录](mxd-round2-review-20261117.md)。

## 原行号状态

ARC：必须目标软件运行；SAMPLE：缺真实文件，附可控生成途径；STATIC：现有材料仍可静态研究或读取器接线，不断言必须运行或补样本。

| 原行 | 状态及最小余项 |
| --- | --- |
| 174 | 部分；STATIC：定位 FeatureBookmark 的 IName 具体实现类与体；SAMPLE：含要素书签的文档，可在 ArcMap 10.1 受控建立要素书签并保存。 |
| 213 | 部分；STATIC：LegendClassFormat getter/setter 与 Load 偏移对照。 |
| 214 | 部分；STATIC：HorizontalLegendItem 属性接口与对象偏移对照。 |
| 215 | 部分；STATIC：Legend 属性 getter/setter 与前缀 short/int32 对照。 |
| 216 | 部分；STATIC：ElementProperties 的属性访问函数与偏移互证。 |
| 220 | 部分；STATIC：MapSurround 前缀 short 和 32 字节块的成员访问链。 |
| 221 | 部分；STATIC：书签接口访问函数与 FeatureBookmark int32 偏移对照。 |
| 242 | 部分；STATIC：ScaleLine 三个调用者及各装饰类属性访问函数，逐字段与 Load/Save 偏移互证。 |
| 252 | 部分；Africa 从 Map 根读到计数 3 与第一个 Legend，后因 ILayer 键状态未知停止；STATIC：恢复前方图层对象登记表后连续读取。 |
| 253 | 未闭合；SAMPLE：含 AOI/Feature 书签的真实文档；可在 ArcMap 10.1 分别创建并保存。 |
| 256 | 部分；Africa 局部预登记探针消费版本 2 体；STATIC：重建引用表后从 Map 连贯读到 `0x9dab`。 |
| 257 | 部分；Africa 从根到首条对象头 `0x9a69`，在 ILayer 停止；STATIC：恢复键 42 并完整读取。 |
| 258 | 部分；同 257，基类版本 3 位于首条中；STATIC：重建真实键状态并走完。 |
| 260 | 部分；STATIC：继续旧版本分支静态复核；SAMPLE：含 Legend v0/v1 的旧版文档，或用相应旧版 ArcMap 生成并保存。 |
| 261 | 部分；已有 Africa `0xb53b..0xb6c3` 局部实例和本轮三类 CGL 分派；STATIC：恢复真实键状态并从外层走至 MarkerNorthArrow。 |
| 262 | 部分；已有 Utilities `0x20cf..0x21ec` 局部实例；STATIC：从 PageLayout 外层与真实引用表走读，亦可继续核对 10.1 CentralUSA 实例。 |

ARC 验收适用于最终转换结果：在 ArcMap 10.1 与 QGIS 打开同一工程检查元素及显示；本轮未运行，以上静态和样本读取均不替代该验收。
