# MXD 图层目标项证据与独立读取器审计

记录日期：2026-11-16。目标版本 ArcMap 10.1.0.3035。本文依据本轮及仓库内已登记的
REA 静态反编译、参考样本逐字节对位和现有读取器；未运行 ArcMap 10.1，也未
在 ArcMap/QGIS 中人工打开验证。参考样本是辅助材料，样本缺项不能证明格式分支
不存在。

本轮范围对应 `TODO.md` 的第 121、169、172–174、243–246、284–285、287 行。
按用户要求不改 TODO；下文给出各项可否勾选的建议。

## 结论摘要

本节为续研前审计快照，FDO 尾段和子层样本分布以文末「续研」为准。

| TODO 行 | 当前证据与实现 | 建议状态 |
| --- | --- | --- |
| 121 GroupLayer 版本分支字节佐证 | 10.1 `GroupLayer::Load` 反编译字段序已闭合，仓库样本没有 GroupLayer；没有目标版本运行捕获或含组图层样本。 | 不勾选。静态字段规则不等于 MXD 字节佐证。 |
| 169 FeatureBookmark IName 实现类体 | 已知槽调用后 QI `IFeatureClass {D4803EE6}`，成功接入、失败软置空；IName `+0x14` 直调语义和流内实现类分布均未闭合。样本书签计数为 0。 | 不勾选。机制证据不构成实现类体/分布证据。 |
| 172 FDO 委派 FeatureLayer 后继 | 委派 FeatureLayer 已读取到 FieldInfo 循环；样本未知区从 `0x2c01` 开始。FDO 后续步骤 7–15 的字段/边界仍未定位。 | 不勾选。不得把 FeatureLayer 的闭合边界外推到 FDO。 |
| 173 FeatureQueryName / TableQueryName | GdbCore RTTI 与 GetIDType 写出的 CLSID 已定位，二类是 IPersistVariant 形态；VariantStream 体布局未反编译。 | 不勾选、不实现体读取器。 |
| 174 ObjectClassName RTTI | 既有 GdbCore 调查未找到实现类 RTTI。 | 不勾选；负结果仅限已检查的 GdbCore 范围。 |
| 243–244 FCN FeatureDatasetName / TopologyName 选择子 | FCN 选择子分派、FDN/TN 体读取器和 `feature_layer.rs` 调用接线均已存在。 | 不勾选：本类别要求样本或运行证据，非零分支仍无佐证。 |
| 245 FDO 旧加载器体 | 10.1 Load `0x6b197dc0` 的版本 0..=3 字段序已有静态证据，并有专属读取器 `src/layers/fdo_legacy.rs`。嵌套未知体按停止点保留。 | 不勾选：本类别要求样本或运行证据。 |
| 246 FDOGraphicsSublayer 体 | 10.1 Load `0x6b1a1b10` 字段与版本分支已有反编译证据，读取器 `read_fdo_graphics_sublayer_10_1` 位于 `src/layers/fdo_legacy.rs`。仓库样本无可用于验证的该对象体实例。 | 不勾选：本类别要求样本或运行证据。 |
| 284 FeatureLayer `+0x1f4` 语义 | 汇编将版本 `>=0xf` 的 IEnvelope 请求绑定到成员 `+0x1f4`；样本值表现为严格内含的视图窗口，可能是绘制/缓存范围。成员名及绘制路径尚无证据。 | 不勾选。样本数值特征只支持候选解释。 |
| 285 FeatureLayer `+0x1b4` 空值行为 | 汇编证实版本 `>8` 时读取 IEnvelope 并写入 `+0x1b4`；样本 11 个 FL 均为空标记，但原生空值对成员状态/后续行为的效果未取证。 | 不勾选。只观察到样本空槽。 |
| 287 Map `+0x298` coclass 身份 | Save 链把 Map `+0x298` 作为 IEnvelope 持有者，静态推定为 Geometry Envelope `{30707212}`；最终 coclass 身份尚未闭合。 | 不勾选。接口用途与具体 coclass 身份是不同层级结论。 |

## 分项原生证据和边界

### GroupLayer（121）

`FeatureLayer.dll` 10.1.0.3035 的 `GroupLayer::Load`（`0x6f711270`）反编译给出：
版本 short（有符号比较，`>12` 拒绝）、名称 String、三个 short、两个 double、
子图层计数和循环、版本门控可见性、IEnvelope、扩展和尾段。读到版本 `>=11`
时子图层循环经过 ExtraSafe `[A][B]` 前缀；读取器依据 B 同步，并以写侧观测到
的 A 总长度处理未识别体。第 5 步以下分支条件和读取器停止语义见
[`group_layer.rs`](../src/layers/group_layer.rs) 及
[`arcmap-10.1-map-object.md`](arcmap-10.1-map-object.md)。

这些是原生代码规则证据，不是版本分支的 MXD 字节佐证。仓库 `samples/测试底图.mxd`
没有 GroupLayer，现有样本无法确认版本 0–12 的实际写出形态。要关闭第 121 行，
需要取得含不同 GroupLayer 架构版本的 10.1 工程流字节，或由 10.1 可控工程保存
产生对应字节并记录版本、对象头和字段边界。单元测试的合成字节不足以替代该证据。

### FeatureBookmark IName（169）

`Map.dll` 10.1 `FeatureBookmark::Load`（`0x793b5c50`）字段为版本、int32、
`LoadObject(IName {4BA33331-…})`、名称 String。名称对象装载后有虚调用 `+0x14`，
继而尝试 QI `IFeatureClass {D4803EE6-…}`；成功时写入书签 `this+8`，失败时
置空继续。此接线机制记录于
[`arcmap-10.1-legend-family.md`](arcmap-10.1-legend-family.md#十featurebookmark-的-iname-槽机制补全分布仍未观察)。

尚缺的是 `+0x14` 调用的准确接口/语义，以及可被该槽保存的 IName coclass 集合和
对应对象体。现有 `read_feature_bookmark_10_1` 在新 IName 对象体处停止是正确的
保守行为；在实现类身份与体布局未确认前，不能用任意 `IName` 类猜测解析。

### FDOGraphicsLayer 委派尾段与查询名（172–174）

10.1 `FDOGraphicsLayer::Load` 主路径已确认会委派聚合 FeatureLayer 的
`IPersistStream::Load`，委派体读取到 FeatureLayer 后继字段和 FieldInfo 循环。
现有样本到 `0x2c01` 后进入未归属区域；不能根据 FeatureLayer 体结束便假定 FDO
结束。第 7–15 步仍需在 AnnoLayer 10.1 原生 Load/调用者中确认，逐项记录流读取、
版本门、成员偏移，并与 Save 方向交叉检查。证据入口见
[`arcmap-10.1-fdo-open-points.md`](arcmap-10.1-fdo-open-points.md) 和
[`arcmap-10.1-featurelayer-tail.md`](arcmap-10.1-featurelayer-tail.md)。

GdbCore 10.1 中 FeatureQueryName `{81CAA5D9-…}` 和 TableQueryName `{81CAA5D8-…}`
的 RTTI、主虚表及 `GetIDType` 写出的 CLSID 已确认；主虚表呈 IPersistVariant
形态。持久化体的 VariantStream 分支尚未反编译，不能将 `IPersistStream` 的对象体
框架套用到它们。见
[`arcmap-10.1-dataset-name-slots.md`](arcmap-10.1-dataset-name-slots.md#featurequeryname--tablequerynameipersistvariant-形态持久化路径未闭合)。
同一研究对 ObjectClassName 没有定位到实现类 RTTI；这只能作为 GdbCore 已检查
范围内的负结果，不证明其他模块不存在适配器。

### 查询名选择子与独立读取器（243–244）

GdbCore `FeatureClassName::Load`（`0x709b5080`）的字段序已由反编译、汇编及样本
对位复核：前置四个 String、两个 int32 后读取首选择子；0 请求
`IWorkspaceName`，非 0 请求 `IDatasetName`。版本 >1 后的第二选择子非 0 请求
`ITopologyName`。样本 11 个 FCN 的选择子均为 0，不能证明非零形态。完整规则见
[`arcmap-10.1-dataset-name.md`](arcmap-10.1-dataset-name.md)。

FeatureDatasetName、TopologyName 体的独立读取器已分别位于
`src/objects/dataset_name.rs` 与 `src/objects/dataset_name_more.rs`，并由
`src/layers/feature_layer.rs` 接入 FCN 分派：首选择子非零走 IDatasetName 统一
分派（含 FeatureDatasetName）；版本 >1 的第二选择子非零读取 TopologyName。
因此相关解析实现已存在。第 243、244 行位于样本无佐证类别，样本的 11 个 FCN
两个选择子均为 0，缺非零分支的真实样本或运行证据，暂不可勾选。

### FDO 旧加载器与子层（245–246）

旧加载器 `FUN_6b197dc0` 及 FDOGraphicsSublayer `0x6b1a1b10` 的字段规则均来自
10.1 AnnoLayer 原生反编译。读取器在独立模块
[`fdo_legacy.rs`](../src/layers/fdo_legacy.rs)：旧加载器支持已确认版本 0..=3
字段；子层读取器消费已确认体。两者均无仓库参考样本实例字节佐证，且旧加载器
内尚未闭合的嵌套新对象保持停止点。

第 245、246 行属于样本无佐证类别，应保持未勾选并等待相应对象样本或运行取证。

### FeatureLayer 字段和 Map 持有者（284–285、287）

FeatureLayer 两个 IEnvelope 槽均由 10.1 Load 汇编绑定：`+0x1b4` 在版本 `>8`
读取，`+0x1f4` 在版本 `>=0xf` 读取；Save 也持有对应成员。全样本 11 个
FeatureLayer 的 `+0x1b4` 槽为空标记，`+0x1f4` 承载 Envelope。后者样本范围
较小且处于视图窗口内，只能形成“绘制/缓存窗口”候选，不能定性字段语义。
逐图层字节分布和限制见
[`arcmap-10.1-spatial-reference.md`](arcmap-10.1-spatial-reference.md#字段语义候选保留提法更正)。

Map Save 链将 `+0x298` 作为 IEnvelope 对象访问器使用；接口类型不能单独证明
运行时 coclass。`Geometry Envelope {30707212-…}` 是基于现有证据的推定，最终身份
需要 Load/Save 的具体类分派、类工厂创建或受控对象流头标识来闭合。不得仅因
Envelope 对象通常对应此类就勾选第 287 行。

## 可复用的实现入口

本轮检查确认以下专属读取器已存在，不重复实现平行版本：

- GroupLayer 已确认前缀和子层递归：`src/layers/group_layer.rs`。
- FeatureBookmark 体读取器及其 IName 槽停止点：`src/maps/map_surround.rs`。
- FDO 旧加载器和 FDOGraphicsSublayer：`src/layers/fdo_legacy.rs`。
- FeatureDatasetName、TopologyName 体与 IDatasetName 槽分派：
  `src/objects/dataset_name.rs`、`src/objects/dataset_name_more.rs`。
- FeatureLayer 后继字段和 FieldInfo 循环：`src/layers/feature_layer_tail.rs`。

本轮没有新增源码文件：现有模块已经涵盖能依据闭合 10.1 字段序独立读取的
部分，FCN 选择子接线也已存在。新增一份未接入、重复解析同一布局的源码会造成
双重规则并带来漂移；真正缺少的是上文列出的原生字节证据和仍未定位的原生字段。

## 2026-11-16 续研：FDO 委派体之后的实际样本边界

### 本轮文件身份和 REA 证据

- 从 `Desktop/Setup1.cab` 条目 `AnnoLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`
  重新提取 PE32 i386 DLL，版本资源 10.1.0.3035，ImageBase `0x6b180000`，
  SHA-256 `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`，
  与既有原生基线相同。
- `DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF`，版本资源
  10.1.0.3035，SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。
  安装配置声明 `{0E5D8C66-8D91-11D3-9FCA-00C04F6BC6A5}` 属该 DLL 的
  SymbolCollection；本轮 `GetClassID` `0x6ebc8100` 独立写出相同 CLSID。
- REA 6.9.0 / Ghidra 12.1.4：AnnoLayer Load `0x6b1955a0` 的 Evidence ID
  `ev_e6e82e511ec1bc16df7bc1a09aa0ff97080d4654720afe95d78a7d9ccae02da0`；
  旧循环 `0x6b195a80` 为 `ev_a5a3a42e1ed79b0e8050361973e71bdbfb157d6b4d82c81d0004e010bb79d5ad`；
  Save `0x6b195b90` 为 `ev_44fed819f580613ff271c374c28161c0f6fff2ed109f566a72f0d2815d5ff743`。
- DisplayCore SymbolCollection Load `0x6ebc8160` 的 Evidence ID 为
  `ev_40a85b6c8d2db3b6a605b932aa737a97b390ba130c0938fcd97ee3cc87b6c32d`；
  Save `0x6ebc8410` 为 `ev_ca2f9c75614f6a214a13b7a90706a417ef70ec7031d569afefda9bd5697d2980`；
  GetClassID `0x6ebc8100` 为 `ev_289857599bbdc73f59dadf8d603a18ff5a18eec5a5e4812ab639c7269ce68131`。
  其中 RTTI `.?AVSymbolCollection@@` 的 COL `0x6ec2272c`（偏移 `0x44`）指向虚表
  `0x6ec15ce4`：`+0x0c` 为 GetClassID thunk `0x6ebc8c50`，`+0x14` 为 Load thunk
  `0x6ebc8d10`，`+0x18` 为 Save thunk `0x6ebc8cb0`。
- 完整 REA 会话证据导出到本机临时文件 `/tmp/luna-fdo-anno-evidence.json` 与
  `/tmp/luna-fdo-display-evidence.json`；其内容含本机路径，分享前应脱敏。

### 原生控制流与 SymbolCollection

`0x6b1955a0` 在委派 FeatureLayer 体成功之后按 FDO 架构版本决定流消费：版本
`<6` 设置默认值后结束；版本 `>=6` 读 int32、short；版本 6/7 再读旧循环
`[int32 计数][String、short、double、double]×计数`；接着读 int32、两个
IColor 槽和 ISymbolCollection 槽。版本 7–10 再读一个 String 与 short；
版本 `>=8` 读 short；版本 `>=9` 读 int32 计数并逐项加载内联
FDOGraphicsSublayer；版本 `>=10` 读 short、ILineSymbol、IMarkerSymbol、
IColor 三槽。该结论来自本轮 Load 反编译，Save 为版本 5/11 提供对照；循环
中 Ghidra 对接口间接调用的伪代码存在明显失真，元素身份还需依照已有构造、
虚表及汇编证据复核，不能单凭伪代码中的空指针表达式推断原生行为。

SymbolCollection Load 的可观察字段：short 版本（原生有符号 `>3` 拒绝）、
int32 计数、逐项 int32 标识和 `LoadObject` 符号；若标识此前不在容器中，
版本 `>1` 读取 String，版本 `==1` 根据序号生成名称；版本 `==2` 在循环后
额外读一个 int32，版本 `>1` 再读一个 int32。**重复标识分支不读取名称**
是反编译控制流观察，尚未以汇编或运行方式独立核对；原生对负数计数不迭代。
Save 对现代版本写标识、符号和名称，并在结尾写 int32；版本表及非空集合
尚未按汇编核实。编写读取器时应显式支持未知符号停止、保留已读对象、限额与
失败事务性，不能从空集合样本推导非空体字段语义。

### 样本字节重走：纠正此前“子层计数为零”

对 `samples/测试底图.mxd` 的 `/Maps` 已有导出 `temp/r6-maps.bin`（SHA-256
`fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe`）按照
上述原生字段序和现有 `read_fdo_graphics_sublayer_10_1` 的字段布局逐字节走查：

| 流内偏移 | 值及归属 |
| --- | --- |
| `0x2c01` | FDO int32 = 1；`0x2c05` short = `0xffff`；`0x2c07` int32 = 0。 |
| `0x2c0b`、`0x2c3e` | 两个 RgbColor 对象头，键 154、155；此前已有颜色体规则。 |
| `0x2c71` | SymbolCollection 对象头，CLSID `{0E5D8C66-…}`，键 156。 |
| `0x2c85..0x2c8f` | SymbolCollection 版本 3、计数 0、尾部 int32 0；依本轮 Load 字段序消费 10 字节。 |
| `0x2c8f`、`0x2c91` | FDO 版本 `>=8` short = 0；版本 `>=9` 子层计数 = **1**。 |
| `0x2c95..0x2ccd` | 一个内联子层：版本 2、int32 0、首 String 长度 6、short `0xffff`、两个零 double、第二 String 长度 2、两个零 double；字段消费恰至 `0x2ccd`。 |
| `0x2ccd`、`0x2ccf`、`0x2cdf` | FDO 版本 `>=10` short 0，随后两个空对象标记。 |
| `0x2cef..0x2d22` | RgbColor 对象头键 157 及 31 字节体；`0x2d22` 为下一图层条目标志。 |

这推翻了本文件此前“样本子层计数 0”和旧记录 `arcmap-10.1-map-object.md` 中
将 `0x2cc9` 定为计数位置的说法：`0x2cc9` 实际属于内联子层末尾 double。
这是依原生反编译规则和样本字节做的静态对位，尚未在 ArcMap 10.1 运行或人工
验证；不应把字符串扫描本身当成对象边界证据。本轮尚未编写或运行将读取器从
委派 FeatureLayer 一路接至 FDO 体尾的自动验证，故第 172 行仍不可勾选。第
243–246 行位于样本无佐证类别；此前建议勾选是错误的，当前维持未勾选，尤其
应先由实际读取器及引用登记表完整复核新发现的内联子层，再评估第 246 行。

### 可接线的后续改动和验证

1. 新建独立 FDO 尾段读取器，以已经完成的委派体末端为起点，接收 FDO 架构
   版本、共享 `Reader`、`RefTable10_1` 和字符串/循环限额。以试读游标和试用
   登记表实现事务性；仅在成功或明确停止时提交。
2. 新建 SymbolCollection 对象体读取器，逐字段复核重复标识分支，复用现有颜色、
   符号及 `read_fdo_graphics_sublayer_10_1`；未知类在对象头起点停止并撤销其
   引用登记，已解析嵌套体中的停止位置必须透传。
3. 后续允许修改共享文件时，在 `fdo_graphics_layer.rs` 的委派体状态为
   `TailCompleted` 分支调用尾段读取器，把结果和停止原因写入 FDO 模型；其他
   委派停止状态继续保留原停止点。不得将尚未消费的剩余字节标成完成。
4. 测试版本 5、6、7、8、9、10、11 的门控，非空子层、SymbolCollection
   空/回引/新对象/未知符号、重复标识和限额；覆盖逐前缀截断时游标与登记表
   不变。用仓库样本完整读到 `0x2d22`，并检查下一图层条目，才能声称这条
   样本路径已接线。完成 Rust 改动后依项目指南运行 `cargo fmt --check` 与
   `cargo test`，涉及 CLI 时按 README 验证；ArcMap/QGIS 人工验收另行记录。

## 未解决事项

1. GroupLayer 版本分支缺少真实 10.1 流字节佐证。
2. FeatureBookmark IName 的流内实现类分布及虚调用 `+0x14` 语义未知。
3. FDO 后续版本门和样本边界已作静态复核；独立尾段读取器和接线未实现，
   非空 SymbolCollection、异常路径及重复标识仍需验证。
4. FeatureQueryName/TableQueryName 的 VariantStream 体未知；ObjectClassName
   的 GdbCore 实现 RTTI 未定位。
5. FCN 选择子分派和 FDN/TN 体已接线，但参考样本未出现非零选择子/目标体实例。
6. FDO 旧加载器无样本字节佐证；本轮新发现一个样本内联子层的静态对位，
   尚未通过实际读取器及完整引用登记表复核；读取器测试只能验证代码契约。
7. FeatureLayer `+0x1f4` 语义、`+0x1b4` 空值效果、Map `+0x298` coclass
   身份均未闭合。
8. 所有结论均为静态研究和现有代码审计；未运行 ArcMap 10.1，未进行 QGIS
   结果人工目视检查。

## 2026-11-16 接线与验证

已按上述原生字段序接入 FDO 后续段和 SymbolCollection：未知符号及嵌套体
停止时保留停止位置，错误输入回滚游标与引用表。参考样本从委派体终点
`0x2c01` 消费到下一图层条目 `0x2d22`，包含一个内联 FDOGraphicsSublayer；
完整 `parse` 可读取六条图层条目和活动索引。两条栅格图层体仍按长度有界
保留为未知，数据框后缀在基本图形层的 `0x1706e` 停止，尚未读取显示范围。
新增版本门、对象引用、非空子层、截断和完整参考样本接线测试；上述样本观察
不替代 ArcMap 10.1 与 QGIS 中的人工验证。此前本文件中关于“尾段未接线”或
“无子层实例”的历史描述以本节及续研边界表为准。
