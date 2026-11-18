# ArcMap 10.1 标注变换属性与模板管理器尾标志（2026-11-18）

本轮用原生 x86 指令、虚表、RTTI 和 esriCarto 类型库交叉验证，下列地址是静态映像 VA。
LabelPlacement.dll、FeatureLayer.dll 和 EditTools.dll 的 PE 固定版本均为 10.1.0.3035。
LabelPlacement.dll SHA-256 为 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`；
FeatureLayer.dll 为 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`；
EditTools.dll 为 `7bc65f7d66d6e9535e3669225dec45fcad976e594915f4354f9080767b7a2313`。
esriCarto 10.1 类型库 SHA-256 为 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。
类型库名称与 x86 槽偏移从 MSFT TypeInfo 记录直接读取；新模块用 objdump 补齐，
既有 FeatureLayer 反编译成果以原生指令复核。

## LabelEngineLayerProperties（LELP）

沿用[已确认的 15 个属性](mxd-terra-nextpass-expression.md)及其中的指针换算：
完整对象 O，Load/Save 持久化指针 P = O+0xb4。构造函数 `0x785458f0`
为 O+0x28 安装虚表 `0x7857caac`。QI `0x78545ce9..0x78545d07` 对 IID
`3EB7EF02-DD8B-11D2-9F47-00C04F6BC6A5`（类型库第 353 项
`IAnnotateLayerTransformationProperties`）返回这个子对象，RTTI 基类描述符
`0x78582b54` 同样记录名称和偏移。类型库第 352 项
`ILabelEngineLayerProperties2` 由 QI 返回 O+0x24，虚表为 `0x7857ca54`。

| 字段 | Load / Save 取地址处 | O+ | 接口槽（get / put）及入口 |
| --- | --- | --- | --- |
| ⑳ `ScaleRatio` f64 | `0x78546ac8` / `0x78546d85`，P−0x24 | 0x90 | O+0x28 的 +0x1c / +0x20：`0x78564ce0` / `0x785613e0`，均访问 this+0x68 |
| ㉑ `ReferenceScale` f64 | `0x78546ade` / `0x78546d9f`，P−0x1c | 0x98 | O+0x28 的 +0x24 / +0x28：`0x7856fa00` / `0x78546760`，均访问 this+0x70 |
| ㉒ `Units` i32 | `0x78546afa` / `0x78546db9`，P−0x14 | 0xa0 | O+0x28 的 +0x14 / +0x18：`0x78546780` / `0x785467b0`，均访问 this+0x78 |
| ㉖ `AnnotationClassID` i32 | `0x78546b75` / `0x78546e11`，P−0x7c | 0x38 | O+0x24 的 +0x48 / +0x44：`0x78546560` / `0x78546310`，均访问 this+0x14 |

标注类 ID 自 LELP 版本 5 出现，与 O+0x34 的 SymbolID、ALPC 集合分配的条目
ID 及要素 OID 各自独立。同一短函数 `0x78546310` 可在不同接口基址上运行，
不能由函数地址相同推断两种 ID 共用成员。比例、单位、类 ID 均保留原始值。

两个没有公开属性对位的旧 short 在 10.1 Load 中都只作为兼容读取消费：
版本 ≤1 的 ④ 在 `0x78546938` 传栈地址，版本恰为 6 的 ㉗ 在
`0x78546b8e` 使用已不再需要的入参栈槽。两个临时值都不进入持久化成员，
原生后续逻辑仅检查读取状态。Save `0x78546bc0` 的表 `0x785797e4`
只有 `[4, 7]`，不会写出这两个旧槽。Rust 保留旧槽原始值供诊断；这里不为早期
写入者赋予未经验证的业务含义。

## EditTemplateManager（ETM）

体有版本、条目数、EditTemplate 对象槽循环及版本 ≥2 的一字节标志。ETM 本体不写
字符串；名称、描述、标签、工具 GUID 和字段默认值属于子对象 EditTemplate，
已有[独立的字段与访问器对位](mxd-terra-host-label.md#edittemplate)。

- FeatureLayer 的导出伪代码 `0x6f6d0670` 显示构造时清零 O+0x34。
  RTTI `0x6f7b6010` 绑定 O+0x14 的辅助接口为 `IEditTemplateManagerAdmin`，
  QI `0x6f6d0810` 对 IID `D06CB85E-F42C-4B1C-9807-B796A5CCF069`
  返回该接口。虚表 `0x6f7abb28` 的 +0x0c 和 +0x10 分别为增添和移除，
  合法增添或成功移除置 1；+0x14 `0x6f6d0a10` 对该字节返回规范化 BOOL。
  `0x6f6d0a80` 的旧档 Load 经同一 +0x0c 添加非空条目，所以 v0/v1
  流内缺席不代表原生对象标志恒为假。v2 的 Load 在条目循环后以流内字节覆盖。
- EditTools `0x10119dcf` 调用 EditorLib 导出 `FindEditTemplateManager`，
  经 `0x1011a360 → 0x1011a440` 在 `0x1011a4a3` 查询相同的管理 IID。
  `0x10119e13` 调用 +0x14 把标志返回到本地 short；`0x10119e19..0x10119e38`
  在非零时退出该图层的模板候选收集路径。零值继续检查图层、渲染和可编辑性；
  合格图层在 `0x1011a1a6` 入候选集合，组合层则在 `0x1011a203..0x1011a271`
  逐子层递归。非零因此阻止这一自动收集路径；零值本身不保证会生成模板。
  这是按已绑定消费者描述的行为，不假定该私有成员有公开属性名称。

没有运行 ArcMap 或 QGIS。这里闭合的是 10.1 持久化字段与已确认原生用途，
没有验证旧版保存器、实际编辑界面、外部实现类或转换效果。
