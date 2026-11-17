# MXD 解析「样本无佐证分支」B 轮独立复核

记录日期：2026-11-17。范围是 `TODO.md`「MXD 解析」节「样本无佐证分支」下的
32 个未闭合叶子；本报告只新增本文件，未修改 `TODO.md`、`README.md`、
`src/`、`tests/` 或他人文档。没有把任何样本缺口升格为完成结论，也没有用
合成分支测试替代真实 MXD 或 ArcMap 运行证据。

## 证据边界与身份复核

- 目标版本为 ArcMap 10.1.0.3035。本轮复核到的可用原件与登记哈希一致：
  AfCore `031858c4…cc47c17`、SystemCore `f63cf949…7de3ad1a`、
  FeatureLayer `2baf8706…7298bb60`、Map `1c0675cc…34b1bb8`、Layer
  `fe52bd99…e9efed8`、LayerLib `55514f05…5f330e02`、Renderers
  `399c80c9…2b656274`、DisplayCore `9b3e61f9…926e4d2`、Geometry
  `c57d154f…828f0d47`、GdbCore `ba5d8f5d…39ca1c4`、AnnoLayer
  `b487f3e0…489a2d33a5`。`temp/arcgis/native/AfCore…`、`Map…`、
  `SystemCore…` 三份哈希均与登记不同，不能作为 10.1 原生证据；本报告不引
  用它们。
- 参考样本导出流 `samples/测试底图.mxd` → `/Maps`，SHA-256
  `fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe`，
  152941 字节，与仓库内 `temp/r6-maps.bin` 等既有导出逐字节同源。流内偏
  移均为该导出偏移。
- 本轮直接扫描 `fc0149f1…` 全流。MapSurround `7A3F91E4`、AOIBookmark
  `BBB1AE73`、FeatureBookmark `EC65B35B`、HorizontalBarLegendItem
  `2B65D211`、ILegendClassFormat 请求 IID `52AA5BC2`、MarkerElement
  `530FD712`、LineElement `8AB7FBE1`、Hyperlink CLSID `3036D35E`、
  StdPicture `0BE35204`、Time 族四 CLSID、FeatureDatasetName `198846CF`、
  TopologyName `9FCA50EC` 均 0 命中；除 Envelope `303607212` 类本身 25 处
  外，CharacterMarker `7914E600` 9 处、SimpleFillSymbol `7914E603` 301
  处也不是本组条目的目标形态。上述扫描是候选定位，不替代对象边界走查；
  对已有文档已作对象级走查的条目，不把本次粗扫写成新证据。
- 本轮一次 REA/Ghidra 打开 Map.dll 后，`binary_overview` 在临时工程导入
  时因 `/tmp` 设备无空间失败。该失败输出不能作为函数证据；下述所有结论
  仅复用可复查的既有 10.1 静态报告、Ghidra 导出、本地 Ghidra 工程和实现
  /测试。字符串扫描只是候选，未据此推断对象边界、数量或体长度。
- 本报告不新增独立 REA 函数分析结论；「本轮复核到的静态证据」指既有
  10.1 报告中已绑定上述哈希的反编译地址、Ghidra 导出文本或本地 Ghidra
  工程产物，以及可重跑的实现/测试。全部均为静态反编译或样本辅助观察；
  未运行 ArcMap 10.1，未在 QGIS 中人工目视验收。以下建议措辞不等于已闭
  合；闭合路径逐条写出。

## 逐项状态与建议

| # | TODO 条目 | 状态 | 核心证据与当前实现 | 建议措辞或闭合路径 |
| --- | --- | --- | --- | --- |
| 1 | FDO 旧加载器体 | 仍开放（静态体已闭合，样本 0） | AnnoLayer 10.1 主 Load 分支：`0x6b1955de/e6` 有符号 >11 拒绝、`0x6b1955ec/f0` ≤3 走 `FUN_6b197dc0`；`0x6b1955f2/07` 传 `(this-0x54,版本)`；旧加载器 20 字段序收尾即完整。见 `docs/arcmap-10.1-fdo-open-points.md` 开放点 1；`src/layers/fdo_legacy.rs` 有版本 0..=3 读取器。 | 维持未勾选：改写为「FDO 旧加载器体（静态 0..=3 字段序与 `fdo_legacy` 读取器已闭合；需版本 ≤3 真实 MXD 或运行取证升级）」。 |
| 2 | MapSurround 循环非空形态 | 仍开放 | Map::Load/Save `0x793ff4d0`/`0x79401850` 的 int32 计数 + 普通 Load/SaveObject、无 ExtraSafe 前缀已闭合；本流 `0x24f60` 计数为 0。分派与 MapSurround 体见 `docs/arcmap-10.1-mapsurround-bookmark.md`、`legend-family.md` 与 `src/maps/map_surround*`。 | 维持未勾选：「MapSurround 循环非空形态（静态读写结构闭合；样本计数 0，需非空真实样本或运行取证）」。 |
| 3 | 书签循环非空形态 | 仍开放 | 书签循环同为 int32 + 普通 LoadObject；本流 `0x24f9c` 计数为 0。AOI `0x793a2cd0` 与 Feature `0x793b5c50` 体静态闭合；Feature 的 IName 实现分布仍开放。 | 维持未勾选：「书签循环非空形态（循环与 AOI/Feature 体静态闭合；样本计数 0，且 Feature IName 实现分布未观察）」。 |
| 4 | FieldInfo ≤3 分支 | 仍开放（静态分支闭合） | Layer.dll FieldInfo Load `0x7865b520` / Save `0x7865b660`；版本门 `4 < short`，v3 无 ⑦、v≤2 无 ⑥⑦。样本 14 个 FieldInfo 全为 v4；`tests/feature_layer_tail.rs::field_info_reads_null_number_format_and_version_branches` 覆盖 v1/v3 合成分支。 | 维持未勾选：「FieldInfo ≤3 分支（静态与合成分支闭合；参考样本全 v4，需 ≤3 真实 MXD 或运行取证）」。 |
| 5 | LegendClassFormat 样本形态 | 仍开放 | Layer.dll `0x78664b20`/`0x78664c40`：版本 ≤2、四个对象槽、v>1 两 double；`src/layers/legend_format.rs` 读取器已接线。本流目标 CLSID 扫描 0 命中。 | 维持未勾选：「LegendClassFormat 样本形态（静态字段序闭合；样本 0 实例，需真实对象或运行取证）」。 |
| 6 | HorizontalLegendItem 样本形态 | 仍开放 | Renderers `0x7cad3040` 条目版本 ≤1 后调 LayerLib BaseLegendItem；`0x7cad5d30`/`0x7cad50e0` 包装链闭合。样本 0 实例；`src/layers/legend_item.rs` 已接线。 | 维持未勾选：「HorizontalLegendItem 样本形态（静态布局闭合；样本 0 实例，需真实对象或运行取证）」。 |
| 7 | BaseLegendItem 样本形态 | 仍开放 | LayerLib `0x786a2b00`/`0x786a2df0` 字段序与版本表 `[1,1,1,1,1,3]` 闭合；`read_base_legend_item_fields_10_1` 为四实现类共用。BaseLegendItem 不是独立 coclass，无独立对象头形态；基类 v0/v1 布局仍无样本。 | 维持未勾选：「BaseLegendItem 样本形态（静态基类字段序闭合；其不是流内独立对象，v0/v1 布局需真实条目或运行取证）」。 |
| 8 | Legend 族版本 0/1 形态 | 仍开放 | Map.dll Legend `0x793da250`/`0x793da650`：自身版本 ≤5、共同前缀、v1 两 legacy double；`src/maps/map_surround.rs` 有版本 0..=5 与 v1 分支合测。写侧 10.1 恒写 5，旧形态无写侧来源；共同前缀 v0/1 分支合测。 | 维持未勾选：「解析 Legend 族版本 0/1 形态（静态读侧分支与合成覆盖闭合；10.1 写侧不产，需旧版真实 MXD 或运行取证）」。 |
| 9 | MarkerElement 体 | 仍开放（静态体闭合） | GraphicElements `0x7504ea50`/Save `0x7504eb70`：版本 ≤3、EP 前缀、IMarkerSymbol/IPoint、v>1 int32、v>2 32 字节；`src/objects/element_body.rs::read_marker_element_body_10_1` 已接 Point 分派。全流 CLSID 扫描 0 命中。 | 维持未勾选：「MarkerElement 体（静态字段序与 Point 槽分派闭合；样本 0 实例，且符号槽实现类分布未观察）」。 |
| 10 | LineElement 体 | 仍开放（静态体闭合） | GraphicElements `0x7504a100`/Save `0x7504a230`：版本 ≤3、EP 前缀、ILineSymbol/IGeometry、v>1 int32、v>2 short；`read_line_element_body_10_1` 已接四类几何分派。全流 CLSID 扫描 0 命中。 | 维持未勾选：「LineElement 体（静态字段序与几何槽分派闭合；样本 0 实例，且符号槽实现类分布未观察）」。 |
| 11 | display 两轮闭合的符号族 | 仍开放 | 第一轮样本仅 SimpleMarkerSymbol；第一轮另五类无实例。第二轮 11 符号 + 三装饰样本 0 实例；`display_symbol.rs` 分派 19 类，TextSymbol 已补。本轮粗扫 MultiLayer/Fill/Arrow/Picture 等目标 CLSID 均 0 命中。 | 维持未勾选：「display 两轮闭合的符号族（静态 Load/Save 与分派闭合；除 SimpleMarkerSymbol 外参考样本 0 实例，需真实对象或运行取证）」。 |
| 12 | display 两轮闭合的颜色族 | 仍开放 | DisplayCore 共享 Color 核心 `0x6eb3d8b0`/`0x6eb35310` 与 Cmyk v0..=4 闭合；样本只有 RgbColor，Hsv/Hls/Gray/Cmyk 无实例。`display_color.rs` 五类读取器已接线。 | 维持未勾选：「display 两轮闭合的颜色族（静态字段序闭合；样本实例不足以覆盖非 RgbColor 类与旧分支，需真实对象或运行取证）」。 |
| 13 | Hyperlink 版本 0 短体 | 仍开放（静态可读） | FeatureLayer.dll Hyperlink `0x6f6d9170`/Save `0x6f6d9210`：版本 ≤1、short+int32+String+int32；`tests/hyperlink.rs` 有 v0 合成体。样本循环计数 0、CLSID 0 命中。 | 维持未勾选：「Hyperlink 版本 0 短体（静态布局与合成覆盖闭合；样本 0 实例，需真实对象或运行取证）」。 |
| 14 | Hyperlink 循环空标记/回引形态 | 仍开放 | ObjectStream 空标记/回引语义与 Hyperlink 循环实现闭合；本类专属样本条目为 0，不能把通用对象流规则升格为本类字节形态。 | 维持未勾选：「Hyperlink 循环空标记/回引形态（通用机制闭合；本宿主循环计数 0，需真实 Hyperlink 条目或运行取证）」。 |
| 15 | NumberFormat 短体形态（九类全族） | 仍开放（静态与合成闭合） | SystemCore 九类 Load/Save 版本门与短体布局已记录并实现；样本仅 NumericFormat v1，其余八类全流 0 命中。合成测试在 `tests/number_format.rs` 与槽级分派测试。 | 维持未勾选：「NumberFormat 短体形态（九类全族静态布局与合成覆盖闭合；参考样本全族无对应实例，需真实 MXD 或运行取证）」。 |
| 16 | NumberFormat 同槽升级与 LatLon/Angle 独立槽核心形态 | 仍开放 | Percentage `0x7ffd11b0` 同槽 v≥2、Rate `0x7ffd6a10` 同槽 v>1；LatLon `0x7ffca1e0`、Angle `0x7ffb33a0` 独立槽；Save 表保证 10.1 恒写满形态。九类样本 0。 | 维持未勾选：「NumberFormat 同槽升级与 LatLon/Angle 独立槽核心形态（静态读写对照闭合；样本 0 实例，需真实 MXD 或运行取证）」。 |
| 17 | Angle 原生容错路径 | 仍开放 | Angle Load `0x7ffb33a0` v≥2 调 NumericFormat 核心、无 TEST/JL 检查后继续读尾 short；严格读取器改为整体失败并注明较原生更严。失败后的游标与对象状态需指令级或运行取证。 | 维持未勾选：「Angle 原生忽略核心失败的容错路径（反编译事实闭合；游标/对象状态和真实截断流未取证，读取器保持较严）」。 |
| 18 | StdPicture 第二字语义与解码器分派 | 仍开放 | oleaut32 2.40.4275 Ghidra 工程 `/tmp/mxd-olestd-ghidra-proj/oleaut32` 与 SaveHeader/Load 链证据显示第二字 Save 写 0、Load 不校验；解码器按魔数分派但入口未逐一闭合。本流 StdPicture 0 命中；oleaut32 本体原件当前不在本地路径。 | 维持未勾选：「StdPicture 第二字语义与解码器分派（2.40.4275 静态头/分派观察闭合；第二字语义、各解码器入口和真实图像样本未闭合）」。 |
| 19 | OLE 字体持久化跨 oleaut32 版本稳定性 | 仍开放 | 介质 2.40.4275 的 StdFont/StdPicture 读写与参考样本 37/37 StdFont 对位闭合；XP/ArcMap 10.1 实际 OS oleaut32 5.1.x 未获取，无法函数/流对比。 | 维持未勾选：「OLE 字体持久化跨 oleaut32 版本稳定性（需 XP/实际 OS 运行库本体或受控运行取证；37 处 StdFont 样本只支撑目标环境写侧字节）」。 |
| 20 | BOLP/UID/HLEP/DEP 前缀样本结构级证据升级 | 部分升级后仍开放 | BOLP 样本结构键 69 @0xda3、体 0xdb7..0xec5；UID `0x80074574` 虚表 + `0x7fff9320/0x7fff9380` 原生 Load/Save、样本键 82 @0x14a9；DEP `0x6f6cbdd0` 全体闭合、样本 0x151a；HLEP 仍只有样本结构 @0x15d8，一手 Load 未定位。 | 不能整体勾选；按对象拆行：「BOLP 样本结构闭合但 Load 未唯一定位」「UID 样本与 10.1 Load/Save 对位闭合，值语义开放」「DEP 静态全体闭合」「HLEP 前缀样本结构仍缺一手 Load」。 |
| 21 | Geometry 版本 0/1 | 仍开放（静态与合成闭合） | Geometry 四类 Load 门有符号 ≤2；PolyBase/Polygon 共用 `0x7192db20`，v≤1 跳过空间参考。`tests/geometry.rs` 有 v1 空几何与跳段；v0 仅受同一门约束且无样本。 | 维持未勾选：「Geometry 版本 0/1（静态门与合成分支闭合；参考样本四类均 0 实例，需真实 MXD 或运行取证）」。 |
| 22 | Geometry S_FALSE 捷径分支 | 未实现、仍开放 | Geometry Load `0x7192db20` 在 AfStreamRead 返回 1 时调用内嵌接口并原样返回；严格读取器不实现。无目标运行观察或样本证据。 | 维持未勾选：「Geometry S_FALSE 捷径分支（需目标运行观察或能捕获该返回态的受控样本；不要仅按静态调用实现宽松恢复）」。 |
| 23 | Geometry 后继段版本 0x1c..0x1f 分支 | 仍开放 | FeatureLayer 后继段版本 `0x1b<v<0x20` 的 short+4 String+标量与三时间槽字段序闭合；本流 FL 版本 34 走 ≥0x20。`feature_layer_tail.rs` 已按门接入，槽未知类停止。 | 维持未勾选：「后继段版本 0x1c..0x1f 分支（静态与门控实现闭合；样本不可达，需旧档真实 MXD 或运行取证）」。 |
| 24 | TimeData 族四段 | 仍开放（静态闭合，样本全空） | FL 后继段 ㊶四段与 SystemCore Time 族读写闭合；本流 `0x156e..0x15d7` 四槽全空。`dep_tail.rs::read_time_data_family_10_1` 与样本形态合成测试闭合。 | 维持未勾选：「TimeData 族四段（静态字段序与空槽形态闭合；非空实现类对象无样本，需启用时间图层或运行取证）」。 |
| 25 | PropertySet 版本 0 分支 | 仍开放（静态与合成闭合） | SystemCore PropertySet Load `0x7ffd3080` 有符号 ≤1、v0/v1 同路径；Save 恒写 1。本流两个实例均 v1；`src/objects/property_set.rs` 覆盖 v0 空集合、v2 拒绝和截断。 | 维持未勾选：「PropertySet 版本 0 分支（静态同路径与合成覆盖闭合；参考样本全 v1，需真实 v0 MXD 或运行取证）」。 |
| 26 | Envelope 版本 0 分支 | 未闭合 | Geometry Envelope Save `0x71869da0` 恒写 3，Load `0x71869b60` 只见 v1/2/3 分支；v0 无静态接受路径、无字节实例。`src/objects/envelope.rs` 明确拒绝 v0。 | 维持未勾选：「Envelope 版本 0 分支（现有静态证据不支持 v0 接受；除非取得旧版字节或反编译新证据，不应实现 v0）」。 |
| 27 | FeatureDatasetName 版本 0 分支 | 仍开放（静态与合成闭合） | GdbCore FDN `0x709b7c90` 有符号 ≤1、v0/v1 同字段序；Save 恒写 1。本流 CLSID 0 命中；`tests/dataset_name.rs` v0 通过、v2/负值拒绝。 | 维持未勾选：「FeatureDatasetName 版本 0 分支（静态与合成覆盖闭合；样本 0 实例，需真实 v0 MXD 或运行取证）」。 |
| 28 | TopologyName 版本 0 分支 | 仍开放（静态与合成闭合） | GdbCore TN `0x7095e5c0` 有符号 ≤1、v0/v1 同字段序；Save 恒写 1。本流 CLSID 0 命中；同一测试组覆盖 v0 通过与越界拒绝。 | 维持未勾选：「TopologyName 版本 0 分支（静态与合成覆盖闭合；样本 0 实例，需真实 v0 MXD 或运行取证）」。 |

## 汇总

- 证据已闭合但「样本/运行取证」缺口仍在的有：1-10、11-17、20 的 BOLP/
  UID/DEP 部分、21、23-25、27-28。这些条目可以把措辞拆成「静态闭合」与
  「样本无佐证」两层，但不应因合成分支测试通过而勾选。
- 明确未实现或证据不足的是：22（S_FALSE）、26（Envelope v0）、18-19 的
  OLE 运行库与真实图像缺口、20 的 HLEP/BOLP Load 归属。
- 本报告未发现可把上述 32 项任一「样本无佐证分支」直接标 `[x]` 的新证
  据；`FDOGraphicsSublayer` 已由未提交工作另行闭合，不在本轮列表内。
