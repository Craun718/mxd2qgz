# Layers 组复核（2026-11-18）

本记录只覆盖分配的 TODO 原行号。未修改 TODO.md、README.md、根模块入口及其他代理的研究文档。反编译、介质样本试读和合成测试分别记录；未把构造输入当作真实 MXD 字节证据。

## 原生证据与本轮补充

- **FeatureLayer.dll 10.1.0.3035**：文件摘要与本机基线一致。默认 REA/Ghidra 12.2 bridge 在沙箱中因 Unix socket `Operation not permitted` 失败；按要求转用独立 pyghidra 12.2/JDK 25 项目，临时目录、缓存和配置均位于 `temp/luna-20261118/rea/featurelayer/`。新反编译 `0x6f711270`（GroupLayer::Load）确认版本 `>12` 拒绝、`<11` 子项走普通 LoadObject、`≥11` 子项走 ExtraSafeReadObject；完整伪代码在 `pyghidra-retry.log`。同次 `0x6f6e9940` FeatureLayer::Load 伪代码已导出。
- **AnnoLayer.dll 10.1.0.3035**：SHA-256 `b487f3e0…a2d33a5`，ImageBase `0x6b180000`。同样在独立 pyghidra 项目复核 `0x6b197dc0`（旧加载器）和 `0x6b198420`（版本 4 探测）；导出在 `temp/luna-20261118/rea/annolayer/pyghidra-retry.log`。旧加载器确认版本 0–3 字段顺序及 SelectionSymbol/IColor 请求接口；版本 4 探测仍依赖流绝对位置与 IObjectStream 查询，未实现。
- **LabelPlacement.dll 10.1.0.3035**：用户授权的独立 REA 定向证据 `ev_39ef6ea29c366767b8d9cf30f6681206e06c8d44d63ed7e3ae31b5caa6d6edb6`，SHA-256 `8415f63c…35ea3d`，Ghidra 12.2，`0x785468c0`（LabelEngineLayerProperties::Load）。证据文件在 `temp/luna-20261118/rea-label/lelp.json`。它确认 LELP 版本上限 7 以及表达式、属性标志、Offset、比例尺、DisplayAnnotation、Class、WhereClause、Priority、GraphicsContainer 等流序；属性访问器成员对位以 `docs/mxd-terra-nextpass-expression.md` 的既有类型库/访问器交叉证据为准。伪代码本身不证明字段公开语义或运行效果。
- 其余模块/体地址沿用对应仓库研究记录：GroupLayer `0x6f711270`、FeatureLayer `0x6f6e9940`、FDO 旧加载器 `0x6b197dc0`、LELP `0x785468c0`；版本均为 ArcMap 10.1.0.3035 静态映像地址。RE A bridge 失败和 pyghidra 调试输出保留在本轮 `temp/luna-20261118/rea/` 子目录。没有启动 ArcMap 或 QGIS。

## 指定 TODO 行逐项状态

| 原行 | 状态 | 证据与结论 |
| --- | --- | --- |
| 150 | 待验证 | GroupLayer `<11` 普通 LoadObject 子层分支已由 `0x6f711270` 静态确认，读取器与合成测试已实现；10.1/SDK 介质实际遍历到的组图层仍是 v11/v12 ExtraSafe 分支。缺 `<11` 真实对象字节，不能升级为实档佐证。 |
| 177 | 部分 | LabelPlacement `0x785468c0` 的 LELP 表达式引擎槽已确认；VBScript/JScript/Python 的字段读取支持已有代码。JScript/Python 缺实际 LELP 宿主字节；其他尚未识别的引擎类仍未知。 |
| 178 | 部分 | REA 证实 LELP 的 GraphicsContainer 请求槽及其顺序；样本该槽为空。PageLayout 的静态持久化实现候选不能证明 LELP 曾保存它，且其完整体/其他实现类未闭合。 |
| 179 | 部分 | FeatureLayer 后继段的 `ISymbol` 请求槽已有静态位置，已知符号族已有分派；尚未穷尽其余实现类，且 LELP 自身相应序号是整数槽。 |
| 248 | 部分 | LELP 的 15 个已读字段已有类型库属性、访问器与存储偏移对位（表达式、Offset、LabelWhichFeatures、CreateUnplacedElements、FeatureLinked、UseOutput、SymbolID、AnnotationMinimum/MaximumScale、DisplayAnnotation、Class、WhereClause、Priority、AddUnplacedToGraphicsContainer）；其余字段仍未可靠命名。REA `ev_39ef…d6edb6` 的 Load 序与既有对位一致。 |
| 249 | 部分 | ETM 的 Count、EditTemplate 集合、版本 ≥2 尾标志已有静态成员对位；EditTemplate Name/Description/Tool GUID/DefaultValue/Tags 已闭合。ETM 尾标志正式属性名、完整生命周期及旧版本对象状态未定；样本尾字节 `0x01` 不是其业务含义证明。 |
| 250 | 部分 | FeatureIDSet Load 的整数项会进入其公开 ID 集合，计数和接口存储槽已对位；不能据此认定每项均为数据源 OID，仍缺非空真实对象和运行观察。 |
| 275 | 待验证 | 本轮把旧加载器已知版本 0–3 的 SelectionSymbol/IColor 新对象接到现有闭合符号、颜色体分派器；静态字段序由 AnnoLayer `0x6b197dc0` 确认。没有旧 FDO 版本对象实档；版本 4 探测不在安全子读取范围内，旧分支总体仍待实档/运行升级证据。 |
| 280 | 待验证 | 当前 10.1 介质/SDK 宿主链证据覆盖 FieldInfo v2/v3，没有 v0/v1 对象体；扫描到的类头命中不构成宿主与体边界证据。 |
| 297 | 部分 | BOLP、UID、HLEP、DEP 的流结构/样本区段已在 FeatureLayer 尾段研究与读取器中记录或解析；但这些体的 Load 未能逐一唯一定位，故不能满足“Load 反编译唯一定位”的证据升级条件。 |
| 300 | 待验证 | FeatureLayer 版本 `0x1c..0x1f` 分支已静态闭合并接入读取器（`0x6f6e9940` 后继段记录）；现有样本走 `≥0x20` 分支，缺该版本范围的实档字节。 |
| 301 | 待验证 | TimeData/TimeTable/TimeDimension/TimeDisplay 四体及版本门已有静态解析；样本同样走 `≥0x20`，没有该四段的宿主字节佐证。 |
| 315 | 待验证 | `+0x1f4` IEnvelope 槽位置/解析已接线；业务语义仍缺成员级命名及绘制路径证据。不能以字段候选名替代定性。 |
| 316 | 待验证 | `+0x1b4` 槽读前释放/清零及其无值静态路径已有记录；题目要求的无值运行行为仍需受控 ArcMap 运行观察。 |
| 363 | 部分 | LELP 表达式字符串与引擎槽已解析，现存读取器保留表达式原文；表达式语法/各引擎执行规则并未解析，且部分引擎缺宿主样本。 |
| 365 | 部分 | TextSymbol 的字体对象槽接入 OLE StdFont 读取器，字段体已解析；跨系统 oleaut32 版本稳定性仍开放，不能由静态读取器推出所有字体兼容。 |
| 367 | 部分 | LELP/BOLP/标注位置优先级等持久化字段已有分支读取；未命名字段、GraphicsContainer 实现及运行布局效果仍未闭合。 |
| 369 | 部分 | LELP 的 `DisplayAnnotation` 原始 BOOL 与图层可见性均有读取路径；完整标注显示语义及目标软件效果未验证，不能把原始非零值归一化为最终可见性结论。 |

## 实现与测试

`src/layers/fdo_legacy.rs` 保持原有公开 `renderer_slot` 与 `color_slot` 字段的 `Option<ObjectEntry10_1>` 类型和值，仅消费已支持的新符号/颜色对象体，不更改旧分类 API。未知新类仍保留原始对象条目分类并在槽头停止；已知符号内出现未知槽时传播嵌套停止并保持其游标位置。旧 FDO 的主版本 0–3 字段边界不变。

`tests/fdo_legacy.rs` 增加完整 SimpleMarkerSymbol/RgbColor 合成输入、未知类原始分类与回滚、嵌套停止偏移/对象表及截断回滚检查。专项测试 23 项通过。`cargo fmt --check` 因工作区其他文件的格式差异失败；本轮两个 Rust 文件单独通过 `rustfmt --check --edition 2024`。全量 `cargo test` 在未修改的 `tests/map_surround.rs::class_id_constants_match_documented_clsids` 失败，期望 GUID 字节顺序与当前常量不一致；本轮没有修改 `map_surround`。`git diff --check` 通过。以上测试验证解析器自身契约，不等同于旧档或 ArcMap 运行验证。

TODO 勾选状态由主代理在整合时处理；本记录不修改 TODO.md。
