# MXD 标注与后继层解析开放项复核

日期：2026-11-16。范围为 TODO 行 108、121、147–149、215–219、238、243–244、247、264、267–268。仅修改本报告及 `src/layers/feature_layer_tail.rs`、`src/layers/annotate_props.rs`。不改 TODO、README、模块接线或共享 robustness 清单。

## 取证基线

- ArcMap 目标身份沿用 [10.1 原生基线](arcmap-10.1-native-baseline.md)：Desktop `Setup1.cab` 提取的目标组件为 x86、版本资源 10.1.0.3035；FeatureLayer.dll 与 LabelPlacement.dll 文件 SHA-256 分别为 `2baf8706…98bb60`、`8415f63c…35ea3d`。本轮复核的 REA 输入哈希与文档登记一致。版本资源与哈希是文件身份，不是运行行为证据。
- 对 LabelPlacement.dll 的定点 REA 反编译 `0x785468c0`（LELP::Load）再次确认：版本 short 后依序读取 String、short、double、int32、short×3、int32、double×2、short、String×2、四个 LoadObject、double×2、int32×2；版本 >2 的 IGraphicsContainer 槽以及版本 ≥4/≥5/==6 的尾字段按门读取。REA 未加载可用 PDB 名称，故类名字符串搜索为空不能作为实现类不存在的负证据。该函数输出是 10.1.0.3035 文件上的静态反编译，不是目标程序运行观察。
- 对 FeatureLayer.dll 的定点 REA 复核 `FeatureLayer::Load 0x6f6e9940` 与 `Save 0x6f6eb1b0` 调用序。Load 在 ㊱ 门内直接调用 `CInternalLayerBase::ReadPreJoinDefExpressions`，之后第一个显式读取是 `+0x2c0` 的 4 字节；Save 对称地直接调用 `WritePreJoinDefExpressions`，之后第一个显式写出也是 `+0x2c0` 的 4 字节。现有 helper 反编译记录为 `[int32 count][count × (String + String)]`。调用点没有另外读写 6 字节。
- 样本 `/Maps` 字节（`temp/r6-maps.bin`）在 ㊱ 附近为：前置 short @`0x14ec`=`-1`；计数 @`0x14ee..0x14f2`=`0`；六个零字节 @`0x14f2..0x14f8`；`+0x2c0` int32 @`0x14f8..0x14fc`=`0`；㊲ short×3 @`0x14fc..0x1502`=`[-1,0,-1]`。解析器为到达样本后续字段而保留这六字节，但读写侧证据不能解释其来源。
- 目前工作区没有 ArcMap 10.1 运行观察；主代理执行的本轮基线 `cargo fmt --check`、`cargo test` 已通过，样本 `parse` 停在 `0x1622`。本轮未等待或重复全局测试。以下凡要求另一真实 MXD 或目标软件运行的项目均标出剩余缺口。

## TODO 逐项状态

| TODO 行 | 状态 | 本轮结论、证据与剩余缺口 |
| --- | --- | --- |
| 108 | 部分 | 本范围的标注属性、LELP、ALPC、ETM 与 FeatureLayer 后继字段已由现有读取器结构化保留；LELP::Load 的新鲜定点反编译支持上述字段次序。TODO 原项覆盖“Map 双名称之外对象属性”的更广范围，无法仅凭本标注子集宣告总体字段归属闭合。剩余：其他对象属性类的系统性字段归属。 |
| 121 | 阻塞（样本证据） | GroupLayer 版本 ≤12、版本 <11 普通子对象与 ≥11 ExtraSafe 分支已有 FeatureLayer.dll `0x6f711270` 静态 Load 证据，`group_layer` 已按这些分支解析；仓库参考样本没有 GroupLayer。图层列表中的 ExtraSafe `[A][B]` 对位只能验证包装格式，不能验证 GroupLayer 内部版本分支。剩余：含 GroupLayer 的真实 MXD 或 ArcMap 10.1 运行观察。 |
| 147 | 部分 | 10.1 esriCarto 类型库列出 AnnotationVBScriptEngine、AnnotationJScriptEngine、AnnotationPythonEngine；`annotate_props` 已解析样本中的 VBScript 引擎体。只有 coclass 登记不能证明 JScript/Python 在 MXD 中可持久化或体布局。剩余：逐一找到 10.1 IPersistStream 实现及其类身份、Load/Save，或证明其不持久化；不得把类型库登记当作读取器体证据。 |
| 148 | 部分 | LELP ㉔ 请求 IID 已确认是 esriCarto IGraphicsContainer `{D09411D1-4BB4-11D1-B6CC-080009B996CC}`；参考样本槽为 16 字节空标记。旧记录中另一 GUID 是转录错误。实现类候选尚未从可核文件中定出，样本不含对象体。剩余：定位 10.1 实现类和持久化体，或有覆盖范围明确的负证据。 |
| 149 | 部分 | LELP ㉖ 槽当前按 `read_symbol_slot_full_10_1` 可识别类分派，TextSymbol 分派也已接入；未知类安全停止。现有研究列出的 17 类闭合集没有建立“10.1 所有 ISymbol 实现”全集，本轮基线已完整消费该样本中的已支持 LELP 链，不能据此外推其他实现类。剩余：接口实现全集与本样本外的真实槽形态。 |
| 215 | 部分 | FDO 旧加载器的静态 `FUN_6b197dc0` 字段序及请求 IID 已记录：ILayer、ISpatialReference、IElementCollection2、IName、IFeatureRenderer、IColor。它们的结构角色可按请求接口说明，但 `ISpatialReference`/渲染器/颜色等旧分支内具体业务状态未从旧版本样本或运行取证定名。样本 FDO 版本 11 不走旧分支。 |
| 216 | 部分 | ALPC 静态 Load 结构为版本、计数、IAnnotateLayerProperties 条目（版本 ≥2 带条目 int32）及尾 int32；早期读取器曾在内层 LELP TextSymbol 槽停止；本轮基线已越过该点，现有样本走查确认 ALPC 尾 int32=1，但字段业务名称仍未闭合。表达式/类名/OID 等是字段候选，不能从字符串内容单独定名。剩余：接口属性映射或运行时行为证据。 |
| 217 | 部分 | LELP::Load 新鲜 REA 复核了字段与对象槽的精确顺序，样本提供 String `[JZDH]`、`默认` 和若干标量值；这些值不能唯一映射到公开属性。候选“表达式/类名/可见性”保持候选，不据此改字段名或业务模型。剩余：类型库属性访问器与成员偏移的静态对应，或运行取证。 |
| 218 | 部分 | ETM 宿主已由扩展段与 EditTemplate 新对象序列定位。样本 ETM 体版本 2、计数 1，末尾字节现有走查更正为 `0x01`（不是早期报告的 `0x00`）；本轮同步更正 `annotate_props.rs` 模块文档。该字节语义仍未知，ETM/模板其他标量与字符串的业务成员映射也未闭合。 |
| 219 | 部分 | FeatureIDSet 体为版本 short、int32 count、count 个 int32；类身份与 IFeatureIDSet 接口已由 CLSID/类型库确认，Save 以集合迭代器写出整数，故 OID 是强候选。样本 count=0，没有元素值佐证其范围、符号或实际 ID 语义。剩余：非空选择集样本或 ArcMap 运行观察。 |
| 238 | 部分（冲突定位已收窄） | 本轮重新反编译读/写调用点，确认 FeatureLayer::Load/Save 都在 PreJoin helper 后直接读/写 `+0x2c0`，调用点无六字节操作；helper 既有反编译也只含 count 与成对字符串。因此已排除“六字节是已知 PreJoin 字段/写出字段”。但样本确有 `0x14f2..0x14f8` 六个零字节，静态写侧不能解释它们如何进入样本。源码保留兼容字段名 `reserved`，注释限定为样本 count=0 下观察到的未知字节；不因命名或保留策略声称其为 PreJoin 字段。非空 count 的间隙位置也没有样本佐证。仍需含非空 PreJoin 的样本、完整写侧对象状态/生成过程，或目标运行取证来裁定来源；当前不能宣称字节来源闭合。 |
| 243 | 部分（真实样本阻塞） | 版本 0..=3 的 FDO 旧加载器读取器已实现，字段覆盖由 `fdo_legacy` 与 `tests/fdo_legacy.rs` 表明；仓库样本外层 FDO 版本为 11，不进入旧路径，合成测试不能替代目标软件验证。剩余：真实旧版文档或运行取证。 |
| 244 | 部分（真实样本阻塞） | FDOGraphicsSublayer 体读取器已实现，静态 Load/Save 和类身份有记录；该样本循环计数为 0，因而没有任何元素体字节可核。剩余：非空子层循环样本或 ArcMap 运行取证。 |
| 247 | 部分（真实样本阻塞） | FieldInfo 读取器实现版本 0..=4 分支；参考样本的 14 个 FieldInfo 全为版本 4，逐字节走查不能验证 ≤3。合成输入可以检查代码分支，但不能证明历史版本兼容行为。剩余：包含 ≤3 版 FieldInfo 的真实 MXD 或运行取证。 |
| 264 | 部分 | 样本现已给出 BOLP/UID/HLEP/DEP 链的结构级字节对位：UID/DEP/HLEP 头与已解析体落在明确偏移；BOLP 结构也可从样本序列定位。但 BOLP 的唯一 Load 归属仍未确认，UID 值的业务语义仍是候选。该条作为一组不能整体标完成。剩余：BOLP 的一手 Load 身份和 UID 属性对应证据。 |
| 267 | 部分（样本证据阻塞） | 版本 `0x1c..0x1f` 的标量与三个时间对象槽已有静态字段序并接入 `feature_layer_tail`；当前 10.1 样本 FL 版本为 34（`0x22`），只走 ≥`0x20` 分支。剩余：历史分支真实字节或运行观察。 |
| 268 | 部分（样本证据阻塞） | TimeData、TimeTableDefinition、TimeDimensionDefinition、TimeDataDisplay 与嵌套 TimeReference/TimeExtent/TimeZoneInfo 体已有静态读取器并接入；当前样本四段槽位全空，不能佐证非空实现类。剩余：含非空时间族对象的真实 MXD 或运行观察。 |

## 本轮改动与交接

- `src/layers/feature_layer_tail.rs`：保留公开字段名 `reserved` 以避免 API 破坏；更正注释，明确它是样本 count=0 路径上观察到的未知字节，不属于已知 PreJoin 读写格式。消费实现保持原样。
- `src/layers/annotate_props.rs`：把 ETM 尾字节样本值更正为 `0x01`，语义继续未知。
- 新增本研究记录。未修改 `TODO.md`，供主代理按逐项闭合规则决定更新状态。
- 本轮未新增或运行测试；保留公开字段名与消费实现，只有说明注释变化。主代理执行的全局基线测试结果继续有效，但本轮没有重新构建。目标软件验证未做。
- 主代理接线事项：没有新增模块接线。请保留 TODO 指定各项为未闭合或部分完成，直到报告所列真实样本/原生证据缺口补齐；第 238 行可记录“排除属于已知 PreJoin 写侧，但间隙来源未决”。
