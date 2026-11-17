# MXD 显示解析专项复核（Luna）

日期：2026-11-16。范围：TODO 第 205、207、212、222–227、239、255–265 行。目标基线 ArcMap 10.1。本文仅新增研究记录；现有解析器已覆盖这些对象体的已确认字节布局，本轮没有发现可在不改共享模块和接线的条件下独立新增的读取器，因此不新增 Rust 源文件。

## 证据来源和复核方式

- REA/Ghidra 12.1.4 直接打开 `temp/arcgis101-bin/SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`，SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。对 Percentage、Fraction、Rate、LatLon、Angle 的 10.1 Load 入口做函数分析，并按保留 Evidence 查看反编译伪代码。
- REA/Ghidra 直接打开 `temp/arcgis101-bin/featurelayerdll/FeatureLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，复核 Hyperlink Load `0x6f6d9170`。
- DisplayCore 符号与颜色、OLE StdFont/StdPicture 依据现有 10.1 记录交叉核对：`arcmap-10.1-display-symbols-round2.md`、`arcmap-10.1-display-slots-round3.md`、`arcmap-10.1-display-tails.md`、`arcmap-10.1-element-bodies.md`、`arcmap-10.1-hyperlink.md`、`arcmap-10.1-number-format.md`、`arcmap-10.1-ole-stdfont.md`。本轮没有在 REA 中打开 DisplayCore 或 oleaut32：工作区对应临时 DLL 路径已不可用；不把已有文档误称为本轮 REA 复核。
- REA 证据 ID：Percentage `ev_07a3b005bc406ec74cb2254c72002a3702aea3df2d621494643a588ef23fd8e8`；Fraction `ev_66ba676d208891089277363fc716ee05fc23dca4d1d95bc9ec5c63e8f72a7fff`；Rate `ev_7ff4bd10a068e83115cafb481e0b337dd5e17c09980eec47c6269aad40b9bed1`；LatLon `ev_99d22e8daa82fd3589fb90b6ae8c2b6fc0c58044a40f65cc2c7c28c291a8fc30`；Angle `ev_9bbd5ad7c90f070a8b4cdfa9d297937cba33312931d7aae26ca7350fdb61c45f`；Hyperlink `ev_85e8d04631002adf0ede539d9cce12c18a41557de3d503c19d7590031b580d3c`。REA 观察是静态反编译，不是 ArcMap 运行时观测。

## 分项结论

### 符号和颜色：TODO 205、207、212、239

- **205 颜色/符号标量业务语义：仍未闭合。** 现有 Load 字段、类族及颜色分量行为可以确认流布局，但如 RGB 颜色三个 double、末尾 byte、各种符号类 int32/double/byte 的业务命名仍有候选解释。没有从字段名或常见符号惯例升级成确定语义的依据；保留原始标量是正确做法。
- **207 三处 short 覆盖：可闭合为静态“读取即弃”。** round2 记录中的 ArrowMarker、CharacterMarker、PictureMarker 相关 short 被读入临时/成员槽，随后由 `Symbol::GetDefaultRotateWithTransformValue` 对应默认值覆盖；写侧写的是覆盖后的槽值。因而该 short 的持久化输入对最终内存字段无效。此结论不等于知道该流值在历史版本中的业务意义，也没有运行期验收。
- **212 TextSymbol 标量语义：仍未闭合。** 字体槽、颜色/符号槽和多个标量的结构位置已有实现；对齐、角度、边距等具体标量语义仍须由类型库成员或运行行为定位。StdFont 的字体属性字段另有 oleaut32 级别的映射，但不能据此给 TextSymbol 其余标量定名。
- **239 读后即弃观察：可闭合为反编译观察清单。** round3 已分别记录 RandomColorRamp 读后丢弃 int32、PictureFillSymbol 读后丢弃两个 double，以及 MultiPartColorRamp Load/Save 门不对称。它们能解释解析时为何保留/忽略这些字节，不能据此命名数值用途；Load 与 Save 的不对称也不代表样本具备相应版本体。

### Hyperlink：TODO 222–224、259–260

- Hyperlink `0x6f6d9170` 的 10.1 反编译确认版本 short（有符号上限 1）、int32、AfCore String、int32 的顺序；类型库把 `{1558D2D3}` 命名为 `esriHyperlinkType`，但没有在本轮取得枚举成员/数值映射。因此 **第 222 行仍未闭合**：第一个 int32 是该枚举槽的候选/强对应，不可写出值序。
- 第二个 int32 是独立槽 `this+0xc`，不应与前一个 `Type` 槽合并；但本轮可得资料不能命名第二槽。因此 **第 223 行“区分两个槽”可按结构证据闭合，语义仍未知**。
- String 位于 `this+0x8`，且读写接口路径与 Hyperlink 主体一致；它是链接目标候选。IHotlinkContainer/热链字段属于不同宿主接口/字段链，不能将 Hyperlink ③ 与宿主热链对象视为同一字段。因此 **第 224 行可按结构边界闭合**，但字符串业务取值类型（URL/路径等）仍未完全定性。
- FeatureLayer 样本 Hyperlink 计数为 0；第 259 版本 0 短体、第 260 空标记/回引形态没有本类样本证据。通用 ObjectStream 规则不能充当 Hyperlink 专属样本观察。

### NumberFormat：TODO 225–227、261–263

- REA 复核的 10.1 Load 形态与既有文档吻合：Percentage `0x7ffd11b0`、Fraction `0x7ffc68a0`、Rate `0x7ffd6a10`、LatLon `0x7ffca1e0`、Angle `0x7ffb33a0`。九类版本门和字段顺序已有实现/记录；类型名与字段布局仍不足以确认每个标量/字符串的显示业务含义。故 **225、226 不能勾选为语义定性完成**。
- **227 的静态布局结论可闭合，样本子项未闭合。** Percentage v2 及以上在原 short 所在位置继续读取 NumericFormat 核心（同槽升级）；Rate v2 及以上在 double 后读取 String，再在该 double 槽位置接续 NumericFormat 核心；LatLon/Angle 的 NumericFormat 核心位于各自不同对象偏移，是独立槽。REA 对应伪代码可见 Percentage 调 `0x7ffce4b0` 的版本条件，以及 LatLon/Angle 的不同基址参数。样本中九类变体没有足够实例覆盖这两种核心布局，所以 TODO 中的“样本 0 实例”验证不能完成。
- Angle `0x7ffb33a0` 在版本 >1 调用 NumericFormat Load 后没有检查返回值，之后仍按版本继续读取尾 short；现有 Rust 读取器把核心失败作为整体错误，较原生更严格。**第 263 行仍未闭合**：容错路径的原生流游标效应未在 ArcMap 运行时验证，不能仅按静态调用继续实现宽松恢复。
- TODO 第 261、262 行标记的是短体/样本形态核验。已有合成测试覆盖格式门与截断，但九类短体和升级核心均缺 MXD 样本；合成测试不能替代样本或目标软件验证。

### 样本与 OLE：TODO 255–258、264–265

- 仓库参考样本没有 MarkerElement、LineElement、两轮 Display 符号族和颜色族的对应样本实例；第 255–258 行不能凭合成测试勾选。
- StdPicture 在参考样本中为 0 实例。Load 按首 WORD 分派 BMP/JPEG/GIF 等解码分支，未知魔数返回错误；`StdPicture` 第二字 Save 写 0，但 Load 不校验，实际业务语义未知。未取得目标库版本的对照图片样本，故第 264 行未完成。
- StdFont 读取器已有 37 个样本实例逐字节吻合（已有记录：21 个 TextSymbol 字体槽、16 个 CharacterMarker 系字体槽），因此基本字体体布局已有样本级验证；但版本 2.40.4275 与操作系统所带 oleaut32 之间的跨版本稳定性假设尚未通过目标运行库对照，第 265 行仍未解决。

## TODO 勾选建议

以下为研究者建议；整合复核认为这些条目涉及业务语义或完整验证，现有证据尚不足以按原文勾选，均保持开放：

- 局部结构证据：第 207 行（三个 short 读取后覆盖）；第 223 行（结构槽位区分）；第 224 行（Hyperlink String 与宿主热链字段分离）；第 227 行（静态核心形态）；第 239 行（已列明的静态读即弃/门不对称观察）。
- 暂不勾选：第 205、212、222、225、226 行语义未定；第 227 行若要求“样本 0 实例”核验则其样本子项未完成；第 259–265 行仍缺对应版本/对象实例或目标运行库对照。第 255–258 行也缺真实对象样本。

其中第 223、224 行适合在 TODO 条目明确写成“字段槽位/宿主字段区分”时勾选；若条目要求业务语义定性，则应保持未勾选。第 227 行也应把静态形态确认与真实样本验证拆开记录。

## 代码与验证

本轮未修改既有源文件、测试文件或 TODO。相关读取器已存在：`src/objects/hyperlink.rs`、`src/objects/number_format.rs`、`src/objects/ole_font.rs`、`src/objects/display_color.rs`、`src/objects/display_symbol.rs`、`src/objects/element_body.rs`，以及 FeatureLayer 后继段中的 Hyperlink/NumberFormat 分派。因没有合法的新增专属代码边界，本轮不复制实现、不新增未接线的冗余解析器。

验证包括：REA 对 10.1 SystemCore.dll 和 FeatureLayer.dll 的静态打开与函数分析；读取仓库研究记录和现有实现。未运行测试（本轮没有代码改动），未在 ArcMap 10.1 或 QGIS 中人工检查；本轮结论不构成运行时或视觉验收。
