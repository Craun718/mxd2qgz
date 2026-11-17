# 第三轮：Labels / FeatureLayer

原 ID 对应 `mxd-round2-needs-20261117.md` 第三个表的第一列。状态表示该原项整体，不表示所有局部研究无进展。除特别说明外均为静态证据，未运行 ArcMap 10.1 或 QGIS，也未核验转换视觉效果。

## 身份与方法

- FeatureLayer.dll：ArcMap 10.1.0.3035，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`，本轮 `sha256sum` 与 [原生版本记录](mxd-boundary-luna.md) 一致；其 PE 首选 ImageBase 为 `0x6f6a0000`。
- LabelPlacement.dll：SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`，本轮哈希与 [10.1 标注记录](mxd-label-luna.md) 一致。
- esriCarto 10.1 嵌入类型库 SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`；原 OLB 与提取关系见 [类型库证据](mxd-dataset-format-evidence-20261117.md#补充246-的真实模板-rust-试读和-225-枚举值)。用 Wine `LoadTypeLibEx` 和 `ITypeInfo::GetFuncDesc/GetNames` 读取；64 位 Wine 的 `oVft` 需除 2 才对应 32 位 DLL 的字节偏移。
- 独立 Ghidra 桥接工具 6.9.0 CLI / Ghidra 12.2：FeatureIDSet Load `0x6f6d97d0`，证据 `ev_b064555635a7126497ef1aee72d05ddcd6c2bbe809533f210aba129d3caca173`；Add `0x6f6d96b0`，证据 `ev_2b53d285e2ac1efd7a1bb7bd735a10a5d5b1b281f7b62219491628218cc0fa37`。原始输出只保留在 `temp/round3-labels/`。静态反编译不是运行轨迹。

## 新增访问器和样本证据

FeatureIDSet 的 RTTI `.?AVFeatureIDSet@@` 类型描述符为 `0x6f7d27bc`；COL `0x6f7b6798` 中子对象偏移 `0x10`，接口虚表 `0x6f7ac338`。同一个类的持久化虚表 `0x6f7ac360` 的 COL `0x6f7b6784` 子对象偏移 `0x14`，Load 槽为 `0x6f6d97d0`。类型库 `IFeatureIDSet {D5BB4B87-E0A1-11D2-9F4D-00C04F6BC78E}` 六个成员依次为 Contains、Count、Add、Remove、Clear、Members；Add 是第三个接口方法，对应 x86 虚表偏移 `0x14`，即 `0x6f6d96b0`。其以接口 this+8 调用 `0x6f6d9940`，Load 以持久化 this+4 调用同一函数；两者都指向完整对象 +0x18。故 Load 每个 int32 进入公开 FeatureIDSet 的同一集合。类型库成员名与同一集合不证明元素一定是数据源 OID，受控非空样本及调用来源仍有价值。

`ILabelEngineLayerProperties {01004143-0D1C-11D2-A26F-080009B6F22B}` 类型库公开 Symbol、SymbolID、Offset、BasicOverposterLayerProperties、Expression、IsExpressionSimple、ExpressionParser；`IHotLinkExpressionProperties {E720618B-99A9-46D7-AA6B-507898E1EA90}` 公开 Expression、IsExpressionSimple、ExpressionParser。`IAnnotateLayerPropertiesCollection` 公开 Add、Count、QueryItem、Clear、Remove、Sort；`IEditTemplateManager` 公开 Count、EditTemplate。仅有签名不能把持久化体中标量绑定到属性。

独立测试直接打开仓库 `samples/测试底图.mxd` 的 `/Maps`，验证 FeatureIDSet 键 75 处 `0xf63..0xf7d` 的完整对象头及版本 1、计数 0，HLEP 键 85 处 `0x15d8..0x1606` 的头、版本 1、空字符串、-1 和零块，并检查 HLEP 每个截断前缀不推进游标。HLEP 体长的解释仍是样本结构，不是 Load 已定位的证明；样本位置由既有宿主链定位，测试不宣称独立证明所有外层归属。

额外按 LabelPlacement.dll 的 MSVC RTTI 与 PE 虚表交叉检查：`BasicOverposterLayerProperties` 的类型描述符为 `0x7858dd20`，COL 显示多个子对象；其中 +0x10 虚表 `0x7857acc0` 的第四槽 `0x78547ba0` 跳转到 `0x78570dc0`。HLEP 类型描述符为 `0x7858dfa0`，其 +0x10 虚表 `0x7857b748` 第四槽也指向 `0x78570dc0`。因此以前对 HLEP 的“仅两个接口”描述不足以概括全部 RTTI 子对象；但 RTTI 子对象及共享函数本身未证明它们是 IPersistStream，也不能将该共享函数直接认作 Load。Ghidra 桥接工具 对 `0x78570dc0` 的伪代码（`ev_1ba1310741198fd5b7e152b02bc0b9dcbc9d2588d5b1b6ea2c681ecc80380bc4`）出现内存流、SaveObject、Seek 和 LoadObject 调用；但反编译报告超过最大重启次数并出现未恢复寄存器变量，暂不能据此确认槽语义或数据流。需要对 QI、接口类型和槽签名继续取证。

## 逐项裁决

| 原 ID | 状态 | 证据和缺口 |
| --- | --- | --- |
| 153 | PARTIAL | 既有 VBScript 引擎体有原生读取证据；10.1 类型库还登记 JScript、Python coclass。登记不证明持久化支持；缺各自 GetClassID、持久化虚表和 Load/Save 或有界不支持证据。 |
| 154 | PARTIAL | LELP 的 IID 已由既有记录更正为 IGraphicsContainer，参考样本该槽为空；尚缺非空实现类与其 Load/Save。见 [DEP 尾段记录](arcmap-10.1-dep-tail.md)。 |
| 155 | PARTIAL | TextSymbol 分派及既有 17 类已接入；其余 ISymbol 实现类未穷尽，仍需类身份、Load 和真实非空槽或有界排查证据。见 [DEP 尾段记录](arcmap-10.1-dep-tail.md)。 |
| 222 | PARTIAL | FDO 旧加载器 `0x6b197dc0` 的版本门和读取顺序已有静态证据；接口槽与业务身份尚缺 QI/访问器和类型库成员对应，旧体真实样本也未覆盖。 |
| 223 | PARTIAL | 已获 ALPC 公共接口成员列表；没有将每个流中标量和字符串对应至具体偏移及访问器，不能定名。 |
| 224 | PARTIAL | LELP Load `0x785468c0` 字段序和类型库 Expression 等公开方法已知；缺各访问器与 Load 成员偏移的逐项绑定，样本文字不足以证明其属性名。 |
| 225 | PARTIAL | ETM Load `0x6f6d0a80` 与 Count、EditTemplate 类型库成员已知；尚缺每个标量、字符串及尾字节的访问器映射。已有记录对尾字节样本值存在差异，未在本轮独立裁决。 |
| 226 | PARTIAL | Load 和 Add 通过同一函数操作同一集合，确认其为 FeatureIDSet 接口集合成员；缺非空真实实例和集合 ID 与具体数据源 OID 的对应证据。 |
| 272 | PARTIAL | UID、DEP 已有唯一 Load 证据，BOLP/HLEP 仍只有样本结构与局部旁证。HLEP 样本头和体边界增加直接回归；仍缺两类各自 GetClassID、所属 IPersistStream 虚表与 Load 的唯一闭环，不能外推可构造格式。见 [对象边界记录](mxd-parse-agent-sample-evidence-edge-branches-20261117.md)。 |
| 290 | PARTIAL | 既有 Load/Save 将版本 ≥0xf 的 IEnvelope 绑定 +0x1f4，并在数据源 QI 后传入另一个接口；已知类型库转储中两个 IID 未命名。仍缺完整类型库接口名、具体绘制路径或受控运行观察，不能确定正式字段语义。见 [空间研究](mxd-parse-agent-map-layer-spatial-20261117.md)。 |
| 291 | PARTIAL | 既有汇编显示版本 >8 的 IEnvelope 槽及样本 11 层为空；Save 对空成员调用 SimpleSaveObject 的空指针路径写 16 字节零。缺构造、setter 和空标记 Load 后的成员生命周期及运行行为，不能声称成员始终为空或没有其他副作用。见 [空间研究](mxd-parse-agent-map-layer-spatial-20261117.md)。 |

## 验证

- `cargo test --test round3_labels_sample`：2 项通过；`cargo test`：通过（部分既有测试按各自条件忽略）。
- `cargo fmt --check` 在共享工作区执行时先后因其他并行工作中的 `tests/round3_line_probe.rs`、`tests/round3_basemap.rs` 格式差异失败；没有修改该文件。本轮新增测试已单独格式化。
- 未打开 ArcMap 或 QGIS；以上均不构成目标软件人工验收。
