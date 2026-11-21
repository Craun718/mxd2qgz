# ArcMap 10.1 ClassBreaksRenderer 对象体

本文记录 `ClassBreaksRenderer` 在 ArcMap 10.1 中的持久化布局、版本门、嵌套对象分派和当前实现边界。结论以安装介质中的 10.1 原生命令为主；仓库参考样本只用于确认现有解析链未被破坏，未替代目标软件人工验收。

## 原生文件

| 模块 | 文件与摘要 | 入口 |
| --- | --- | --- |
| FeatureLayer.dll | `FeatureLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` | Load `0x6f6a7190`，Save `0x6f6a7870` |
| LayerLib.dll | `LayerLib.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`，SHA-256 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02` | DataExclusion Load `0x786e4880`，Save `0x786e4930` |

反编译产物：

- `/home/gxrs/文档/source/arcmap10.1/FeatureLayer/6f6a7190.c`
- `/home/gxrs/文档/source/arcmap10.1/FeatureLayer/6f6a7870.c`
- `temp/terra-class-breaks/DataExclusion-Load-0x786e4880.c`
- `temp/terra-class-breaks/DataExclusion-Save-0x786e4930.c`

## 类标识与接口标识

FeatureLayer 的渲染器槽以 `LoadObject` 请求 `IFeatureRenderer`：

```text
{40A9E884-5533-11D0-98BE-00805F7CED21}
```

流中的对象头不保存该请求接口 IID，而保存具体 coclass 的 ClassID。`ClassBreaksRenderer` 的 coclass CLSID 为：

```text
{AE5F7EA2-8B48-11D0-8356-080009B996CC}
```

两者不可互换：前者说明宿主希望得到什么接口，后者说明流中的新对象应实例化为哪个实现类。`esri.clsid` 的 `ClassBreaksRenderer` 声明与 FeatureLayer.dll 的同名持久化入口相互印证。

本对象体内的直接嵌套槽仍由 `LoadObject` 读取，按各自的请求接口和实际 coclass CLSID 分派。已接入的图例项类为：

| coclass | CLSID |
| --- | --- |
| LegendGroup | `{167C5EA2-AF20-11D1-8817-080009EC732A}` |
| HorizontalLegendItem | `{A9401A47-4649-11D1-880B-080009EC732A}` |
| HorizontalBarLegendItem | `{2B65D211-C2C7-11D3-92F3-00600802E603}` |
| NestedLegendItem | `{2B65D212-C2C7-11D3-92F3-00600802E603}` |
| VerticalLegendItem | `{A9401A48-4649-11D1-880B-080009EC732A}` |

## 读取布局

`FeatureLayer.dll` 的 Load 首先读无符号 short。原生判定接受 1 至 8，超出范围即失败并进入统一收尾。其余字段按下列顺序读取：

1. `version: u16`，仅接受 1 至 8。
2. `field: String`。
3. `secondary_field: String`。
4. `count: i32`，随后先读 `count` 个 `f64`，再读第二组 `count` 个 `f64`。
5. 版本大于等于 2 时读 `minimum: f64`；版本 1 不读流，保留原生内存初值的原始位模式 `0xff_ee42_d130_773b_76`。
6. LegendGroup 对象槽。
7. LegendItem 对象槽。
8. 版本 1 读旧式 `String`；版本大于等于 2 先读 1 字节存在标记，非零时再读 16 字节 GUID。
9. `u16`。
10. NumberFormat 对象槽。
11. 版本小于 5 时读旧式 `String`。
12. 读 `label: String`；版本小于 5 时再读 `u16`。
13. 读两个 `u16` 和两个 `i32`。
14. 版本大于 4 时读 `LegacyRotation` 版本。原生只接受 0 和 1；随后读 `String` 与 `i32`。版本小于等于 4 时没有独立版本字段，但仍读这两个字段。
15. 读一个 `f64`。
16. FillSymbol 对象槽。
17. 版本大于 2 时读一个 `i32` 和一个 `f64`。
18. 版本大于 3 时读一个 `i32`。
19. 版本大于等于 5 时读 DataExclusion。
20. 版本大于等于 6 时依次读 RotationRenderer 与 SizeRenderer 的持久化尾段。
21. 版本大于 6 时读 ColorRamp 对象槽。
22. 版本大于 7 时读最终 Symbol 对象槽，成功读取槽后再读一个 `u16`。

版本 8 的合成完整对象可把这些允许为空的嵌套槽全部写成 Null，并读到最终 short。读取器仍会保留尚未定性的标量为原始类型，避免为了命名而猜测业务含义。

### DataExclusion

`DataExclusion` 是 ClassBreaksRenderer 体内的直接结构，不是带独立对象头的 `LoadObject`。LayerLib.dll 的 Load/Save 对位如下：

1. `version: u16`，原生拒绝大于 1 的值；读取器同时拒绝按有符号解释为负的位模式。
2. LegendGroup 对象槽。
3. `description: String`。
4. `u16`，当前实现保留为 `show_exclusion_class` 原始值。

DataExclusion 的字段名和开关语义尚未由类型库或消费者行为完整定性；这不影响按上述边界继续消费后续对象。

## 嵌套对象和停止点

渲染器只按本层已确认的 coclass 读取嵌套体：

- LegendGroup 与 NumberFormat 复用各自已有读取器。
- LegendItem 支持 Horizontal、HorizontalBar、Nested、Vertical 四类，并透传其基类或符号槽内部的停止原因。
- FillSymbol、ColorRamp 和最终 Symbol 复用共享对象槽分派。
- DataExclusion 内另有一个 LegendGroup 槽。

遇到未知 coclass 时，不把“不支持”误报为截断。外层读取器成功返回带 `class_id` 和 `key` 的停止原因，调用方游标停在未支持槽的对象头起点，未支持对象的键也不登记；渲染器自身的新对象键已在进入体读取前登记。若已知嵌套体的前缀内再次遇到未知类或截断，则保留该嵌套体的已读前缀和内部停止原因，不再读取渲染器后继字段。

顶层读取采用事务式暂存：版本非法、数量为负或溢出、数组/字符串/标量截断时，调用方游标和引用表均回退到对象体起点。只有成功读到某个明确的内部停止点，或完整消费版本 8 尾段时才提交已读前缀。

## 与现有接入的关系

`src/layers/class_breaks_renderer.rs` 实现本记录的对象体；`src/layers/feature_layer.rs` 在 `IFeatureRenderer` 槽中按 `ClassBreaksRenderer` coclass CLSID 分派，并将内部停止原因经 `FeatureLayerStop10_1::ClassBreaksRendererBody` 透传。FeatureLayer 接线测试同时覆盖完整后继段和体内停止不越读后继段两种情况。

## 验证边界

- `cargo fmt --check` 通过。
- `cargo test` 通过，包括 ClassBreaksRenderer 的版本分支、完整 v8、Null/回引、四类图例项、DataExclusion、共享符号和色带停止点、截断与事务性回退，以及 FeatureLayer 两处接线测试。
- `cargo run -- parse 'samples/测试底图.mxd'` 成功，参考样本输出仍读至 `/Maps:0x25569` 与 `/PageLayout:0x1c62`。
- 尚未在 ArcMap 10.1 中打开含非空 ClassBreaksRenderer 的工程做对象级目视核对，也尚未在 QGIS 中核对转换结果；因此不能把自动测试视为目标软件验收。
