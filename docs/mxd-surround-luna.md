# ArcMap 10.1 Legend、MapSurround、书签与图面元素语义审查

记录日期：2026-11-16。对应 TODO MXD 解析 208–211、215–217、237、247–254 行。本文只记录本轮语义审查和样本边界；不修改 TODO，也不把既有读取器当作样本验证结果。

## 结论摘要

- 已有读取器覆盖 Legend 前缀/尾段、LegendClassFormat、HorizontalLegendItem/BaseLegendItem、MapSurround 共同前缀、AOIBookmark/FeatureBookmark，以及可识别的文本/Marker/Line 图面元素。解析结构已存在，故本轮没有新增重复源码或改动共享模块。
- Ghidra 桥接工具 对从 ArcMap 10.1 Setup1.cab 提取的 Map.dll 复核了 Legend、MapSurround 共同前缀、AOI 书签和 Feature 书签的 Load 实体。调用顺序与仓库已有布局记录相符；具体字段语义仍须按证据强度区分。
- 样本 `samples/测试底图.mxd` 的 MapSurround 和书签循环计数均为 0；Legend/装饰物及书签对象体没有样本实例。因此第 247–254 行相关的非空循环、LegendClassFormat、HorizontalLegendItem/BaseLegendItem 和 Legend 旧版本样本项均未被本轮实现或静态分析闭合。
- 图面元素的字符串字段语义可由接口名提出候选；多个整数、浮点及自定义属性仍按原始值保留。没有运行 ArcMap/QGIS，未作人工目视验证。

## 研究对象与可复核证据

- 10.1 安装介质：README 所述本机 ArcMap 10.1 Desktop Setup1.cab。Ghidra 桥接工具 从中提取 `Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，PE32 x86，ImageBase `0x793a0000`，SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。哈希与既有 10.1 研究记录所用 Map.dll 一致；介质声明版本为 10.1.3035。
- Ghidra（经桥接工具） 12.1.4 shipped-artifact Evidence：Legend `FUN_793da250`，ID `ev_8cf0eadc86bac4a1b9a6c5f8029fc6fd22774cf21036cdb83a488dc18dcbffbd`；MapSurround 共同前缀 `FUN_794413d0`，ID `ev_6ad9125ebcac64ae34f7bd7d7da65ab32cb3a73cc725d1bb9413385324c6ca42`；AOIBookmark `FUN_793a2cd0`，ID `ev_cf736ced07110b759d4b690bc925b05db816291be7989ce02be0ffeb63eb1e08`；FeatureBookmark `FUN_793b5c50`，ID `ev_76f42705e7e64509d77abc6d39cf75ca6faa75f2b239998d1703eda1b6ceda07`。
- 方法是对上述 Load 入口作静态反编译并与仓库既有 Save/类型库/类身份研究互证。Ghidra 伪代码不是原始源码；对未命名字段的业务解释不因此自动成立。LegendClassFormat、HorizontalLegendItem 与元素字段的详细布局，本轮复用了仓库 10.1 既有研究记录，没有声称本轮重新逐个调用 Ghidra 桥接工具。

## Legend 字段语义

### Legend 共同前缀与自身字段

Legend Load 先读自身版本，再调用 MapSurround 共同前缀，随后读 `ILegendFormat` 槽、标题 String、版本 1 遗留双 double、三个 short、集合接口上的 short、`ILegendItem` 计数与条目循环，以及版本门控尾段。

| 字段 | 当前判断 | 证据边界 |
| --- | --- | --- |
| 共同前缀 String | MapSurround 名称字段 | Load 写入对象名称成员；业务角色也与公开 MapSurround 名称成员吻合。 |
| 共同前缀 IMap 对象槽 | 关联地图 | 请求接口为 IMap；样本/既有记录显示通常是 Map 键回引。 |
| 共同前缀版本 ≥2 的 short | 未定性 | 仅确认存储位置和版本门；不能从 short 类型推定布局标志。 |
| 共同前缀版本 ≥3 的 32 字节 WKSEnvelope | 未定性/读后丢弃 | Load 读入临时区域，未发现写入对象成员；Save 却从对象内存恒写相应区域。不能直接命名为显示范围或位置。 |
| Legend `ILegendFormat` 槽 | Legend 格式对象 | 接口 IID 与类型库名称相符；对象体语义由其具体实现类决定。 |
| Legend 标题 String | 标题候选 | Load 写入标题附近成员，但本轮没有属性 getter/setter 与该偏移的独立闭环。 |
| 版本 1 双 double | 旧版兼容字段，读后丢弃 | 仅版本 1 读、Save 无对称字段；不能断言其具体单位/用途。 |
| 三个 short、集合前 short、尾部 short/int16×3+double | 未定性 | 已有体布局和版本门；未从公开属性或运行数据确定其业务含义。集合前 short 的 Save 恒写 `0xffff`。 |
| 条目循环 | 图例条目集合 | Load 请求 `ILegendItem` 并逐项加入 Legend 集合接口；成员身份确定，排列/布局字段另论。 |

Legend 写侧 10.1 版本恒为 5，MapSurround 共同前缀写侧恒为 3。以上仅证明该版本保存时的结构，不证明较早版本实例在样本中出现。

### LegendClassFormat

Layer.dll 体前缀含两个 `ITextSymbol`、`ILinePatch`、`IAreaPatch`，版本大于 1 时另有两个 double。按槽位和接口身份，两个文本符号分别作为“标签符号”和“描述符号”是有根据的角色候选；四个接口成员本身可确定其对象类别。两个 double 目前只有字段宽度、顺序和版本门证据，语义未定。写侧恒写版本 2。样本没有此类对象。

### HorizontalLegendItem / BaseLegendItem

BaseLegendItem 含关联 `ILayer` 槽、两处 `ITextSymbol` 槽、`ILegendClassFormat` 槽、`INumberFormat` 槽，以及多个 short/int32 和版本门控的两个 String、两个 short。接口槽的类别可以确认；标签符号角色仍是由同族属性/位置推得的候选。版本 ≥2 的两个 String 按默认值 `" ("`、`")"`，可称为显示前后缀候选；int32、四个 short、后续 short 组等没有成员级命名依据，继续原样保留。样本没有 HorizontalLegendItem 或 BaseLegendItem 实例，不能据此确认真实项目中的值域和效果。

## MapSurround 与书签

### MapSurround 共同前缀

Ghidra 桥接工具 复核 `FUN_794413d0`：int16 版本、String、`LoadObject` 请求 IID `IMap`、版本大于 1 时 short、版本大于 2 时 32 字节 WKSEnvelope。后者由原生函数读入栈临时区；Load 体没有将其写入对象的迹象。既有 Save 反编译显示恒写对象内另一个位置对应的 32 字节，因此该项读写映射/初始化路径仍未解释。

| 字段 | 结论 |
| --- | --- |
| String | MapSurround 名称，业务角色确定。 |
| IMap 槽 | 所属/关联地图，接口类别确定。 |
| 版本 ≥2 short | 未定性，按原始位模式保留。 |
| 版本 ≥3 32 字节 envelope | 结构上按 WKSEnvelope 读取，业务语义未定；不能因类型名称就断言为装饰物边界或显示范围。 |

### 书签

- AOIBookmark：Ghidra 桥接工具 证实版本 short、`IEnvelope` 对象槽、String。IEnvelope 的接口类别确定；它作为空间书签范围是由类身份与接口组成共同支持的语义。String 是书签名称候选，仍建议用公开成员或运行值确认其精确属性名。
- FeatureBookmark：Ghidra 桥接工具 证实版本 short、int32、`IName` 槽、String。`IName` 对象槽表示要素/数据名称相关对象，但它的具体实现类与其 `+0x14` 虚调用语义未由本轮闭合。int32 未定性；末尾 String 为书签名称候选。
- 两类保存侧恒写版本 1；Load 允许 0..=1。该版本兼容范围不代表样本具有任一书签对象。

## 图面元素业务语义与样本状态

ElementProperties 现有读取器按流字段保存版本、两个 String、自定义属性、short、double 和 int32。`IElementProperties2` 接口公开成员名为 Name、Type、CustomProperty、ReferenceScale，可用作对应槽位的语义候选；目前尚无 getter/setter 接收偏移与 Load 落点的独立互证，不能提升为确定结论。版本小于 2 的遗留 int32 明确是读后丢弃字段。

TextElement 已确认包含文本 String、`ITextSymbol`、几何对象和额外 int32。文本内容和符号/几何接口类别可确定；额外 int32 虽在内存布局注释中按“布尔”候选描述，样本值 1 和 32 位宽不足以证明布尔语义。MarkerElement、LineElement 的专属体在仓库读取器中已有结构化解析，但本参考样本 MarkerElement 与 LineElement 均为 0 实例，不能把可解析性作为样本确认。

| TODO 样本项 | 本仓库样本所见 | 状态 |
| --- | --- | --- |
| MapSurround 非空循环 | 计数 0 | 未验证循环条目/对象体 |
| 书签非空循环 | 计数 0 | 未验证循环条目/对象体 |
| LegendClassFormat | 0 实例 | 未验证字段值与版本形态 |
| HorizontalLegendItem | 0 实例 | 未验证字段值与版本形态 |
| BaseLegendItem | 0 实例 | 未验证字段值与版本形态 |
| MarkerElement / LineElement | 各 0 实例 | 未验证元素体 |
| Legend 版本 0/1 | 未发现对应 Legend 实例（MapSurround 计数为 0） | 未验证旧版本形态 |

## 可独立完成项与未解决项

本轮重新确认目标 DLL 身份、用 Ghidra 桥接工具 复核四个 Load 字段序，并基于接口类型和读写操作收紧语义边界。仓库已有实现覆盖对应布局，新增并行读取器会造成字段定义和分派重复，因此没有创建源码文件。

仍开放：Legend 各 short/int32 的成员级语义；LegendClassFormat 双 double；HorizontalLegendItem/BaseLegendItem 标量；MapSurround 共同前缀 short 与 32 字节段的真实用途；FeatureBookmark int32、IName 实现分布及调用语义；ElementProperties 候选成员映射；TextElement 额外 int32；所有指定的非空样本形态和 ArcMap/QGIS 人工打开检查。应由真实非空 MXD、10.1 类型库属性实现偏移互证或受控运行取证继续闭合。
