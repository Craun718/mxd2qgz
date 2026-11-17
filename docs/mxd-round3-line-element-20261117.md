# 第三轮：PageLayout 与 LineElement

范围为 `mxd-round2-needs-20261117.md` 第三个表第一列的原 ID，非其他文档的行号。本轮没有运行 ArcMap 或 QGIS，静态反编译、介质字节和合成测试均不能替代人工验收。

## 262：页面入口与候选元素范围

复用 REA/Ghidra 12.1.4 输出 `temp/pseudocode/pagelayout/`：导出清单的目标 SHA-256 是 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`，本轮对 `temp/arcgis101-bin` 中的 PageLayout.dll 重新计算得到相同摘要，身份及版本 10.1.0.3035 见 `arcmap-10.1-native-baseline.md`。同名 `temp/arcgis/native` 文件摘要不同，本轮没有作为 10.1 证据。伪代码为反编译结果，不是原始源码。

PageLayout RTTI 的 `IPersistStream` 虚表 `0x7b71fa34` 中 GetClassID `0x7b6e4e00` 写 PageLayout CLSID，Load 槽是 `0x7b6e4e30`。Load 先读取无符号 short 版本（要求小于 16）、4 字节成员、4 字节有符号循环数；版本小于 7 调 `LoadObject`，否则调用 `ExtraSafeReadObject` 加载各元素。原生循环负数不进入；本轮新增读取器严格拒绝负数、过高版本和超限数量，失败回滚游标。未知成员仍保留原始位模式。

已有 `arcmap-10.1-object-body.md` 证明同一 ObjectStream 在 `/Maps`、`/PageLayout` 之间切换流，引用表不随切换清空。按上述前缀和 `src/layers/layer_list.rs` 记录的 **A 为对象头加体长度、B 为零时按 A 走位的观察推定**，两份本地 10.1 介质文件均可逐项检查候选范围：

| 文件 | 文件 SHA-256 | PageLayout 根键 / 版本 / 数量 | 零起元素序号 / 头偏移 / 键 | 候选对象范围 |
| --- | --- | --- | --- | --- |
| Utilities.mxd | `9c439b8d6624325b6b1a2e1199a7396772b6e5249c61f1076025e35323bd0eb6` | 163 / 14 / 27 | 11 / `0x20cf` / 254 | `0x20cf..0x21ec` |
| CentralUSA.mxd | `b3b594837ab29c9435c2214793d1dcc65d2985a73d26b6619d83704f99005bac` | 736 / 15 / 9 | 1 / `0xa0c` / 744 | `0xa0c..0xb29` |

两处前缀 `[A,B]=[285,0]`，已知 LineElement 局部探针消费范围均恰为 285 字节。Utilities 的 `/Version` 为 10.0/18 候选，CentralUSA 为 10.1/19 候选；介质版本与文档保存版本不可混同。Utilities 首项是 MapSurroundFrame 候选，CentralUSA 首项是 TextElement 候选。测试仅按范围检查对象头，**没有将跳过的对象键登记进引用表**；长度推定也不能替代逐个 Load，因此这是从页面根对目标位置的结构性定位，不是从原宿主完整解析到目标，254/744 也还不是已验证的首次登记及实际回引关系。

新增 `src/maps/page_layout_prefix.rs`，测试 `tests/round3_line_page_layout.rs` 覆盖版本门、限额、负数、每个截断及两个介质前缀/候选范围。该模块尚未声明：主代理如需对外使用，请在 `src/lib.rs` 的 `maps` 模块中添加 `pub mod page_layout_prefix;`，并在对象头和真实跨流引用上下文确定后接入；目前测试通过路径包含模块，不需要修改共享接线。未使用扫描结果构造引用表，也未修改 TextSymbol。

## 按原 ID 落档

| ID | 状态 | 证据与缺口 |
| --- | --- | --- |
| 262 | PARTIAL | 上述页面 Load、两份介质结构定位和现有 `tests/element_body.rs` 局部完整体；尚缺从 DrawingDefaults、Maps 经页面前置元素持续建立真实引用表，解析 Frame/Text 等前置对象，再逐项证实目标键与内部引用。 |
| 174 | PARTIAL | `arcmap-10.1-legend-family.md` 第十节及 `mxd-round2-surround-elements-20261117.md`：FeatureBookmark 请求 IName 并尝试取得 IFeatureClass；尚缺该槽具体实现类分布和真实非空书签对象体。 |
| 213 | PARTIAL | `arcmap-10.1-legend-tails.md` 第一节、`mxd-open-legend-20261116.md` 原 206：两个文本符号、线/面补丁和两个版本门 double 的读取顺序已有证据；double 和各符号的确切业务语义仍需 10.1 属性访问器与成员偏移互证。 |
| 214 | PARTIAL | 同文第二节及 `mxd-open-legend-20261116.md` 原 207：BaseLegendItem 各对象槽和字符串默认值已知；int32、short 和其余标量仍需 Renderers/LayerLib 属性访问器与 Load 偏移对位。 |
| 215 | PARTIAL | `arcmap-10.1-mapsurround-bookmark.md` 第六节：Legend 三 short、集合选择 short 和版本门尾字段可定位；各成员业务语义仍需接口访问器或受控运行互证。 |
| 216 | PARTIAL | `arcmap-10.1-element-bodies.md` 第二节：两个字符串、Variant、short、double、int32 成员及版本门已知；名称、类型和参考比例仍属候选，需 LayerLib 属性访问器绑定。 |
| 220 | PARTIAL | `arcmap-10.1-mapsurround-bookmark.md` 第三节：名称、IMap、short 和 32 字节组已定位；32 字节读入后丢弃，不能仅凭长度称为包络；需成员访问链确定语义。 |
| 221 | PARTIAL | `mxd-open-legend-20261116.md` 原 214：AOIBookmark 与 FeatureBookmark 不共享相同流内前缀；FeatureBookmark 的 int32 仍需属性访问器绑定。 |
| 242 | PARTIAL | `mxd-open-legend-20261116.md` 原 235 已复核三个指定调用者的反编译，故“未反编译”子缺口已有后续证据；三个 coclass 身份、IID 和字段业务名仍未闭合，装饰物族标量仍需逐类属性访问器对位。 |

## 验证与限制

`cargo fmt --check`、`cargo test`、`git diff --check` 通过；单独执行 `cargo test --test round3_line_page_layout -- --include-ignored` 覆盖本地介质。尚未在 ArcMap 10.1 或 QGIS 中打开工程及目视检查。原始伪代码、介质和诊断数据均只在忽略的临时目录；未提交。
