# 10.1 标注集合与表达式属性对位（2026-11-17）

后续已将 LELP 的四个未定成员对位到原生变换属性和标注类 ID，并确认两个旧 short
在 10.1 仅用于兼容消费，见[字段对位](mxd-10.1-label-transformation-fields.md)。

本次只补充成员归属；读取顺序和既有研究见 [FeatureLayer 尾段](arcmap-10.1-featurelayer-tail.md)和[标注续研](mxd-terra-next-label.md)。地址均为静态 PE VA，O 为完整对象起点、P 为传入 Load 的指针。没有启动 ArcMap 或 QGIS。未改 TODO，以下按缺口边界而非旧行号判断。

## 证据与换算

重新核对介质提取的 LabelPlacement.dll：PE32 x86、基址 `0x78530000`，FileVersion/ProductVersion `10.1.0.3035`，InternalName `LabelPlacement`，OriginalFilename `LabelPlacement.dll`，SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`，与既有 Ghidra 桥接工具 证据一致。esriCarto 10.1 TYPELIB SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。哈希确认字节身份，版本资源不是签名或运行证明。

复用本 DLL 的独立 Ghidra 桥接工具 CLI 尝试：此前同哈希 Ghidra 分析后桥接 Unix socket 报 `Operation not permitted`，见 [引擎研究](mxd-terra-depth-label.md)。本次使用 JDK 25、Ghidra 12.2、`GHIDRA_INSTALL_DIR=/opt/ghidra`，将既有工程复制到 `temp/terra-nextpass-expression/`，使用独占 TMPDIR、XDG 配置/缓存及 pyghidra 复核 Load、构造函数、QI、访问器，另查 PE 虚表和指令；未切换共享 Ghidra 桥接工具 目标。伪代码不是原始源码。隔离 Wine 沙箱内 socket 被拒；在隔离前缀中通过 Wine `LoadTypeLibEx` / `GetFuncDesc` / `GetNames` 导出类型库成员，工具进程退出码为 3，但目标五个类型及成员已写出，旧的同哈希导出可独立复核其中四个接口；本次新增的 IAnnotateLayerProperties 结果按 IID、槽位及 PE 实现交叉核对。64 位 Wine 的 `oVft` 除以 2 为目标 x86 槽字节偏移。忽略目录证据可能含本机路径，分享前须脱敏。

LELP 构造函数 `0x785458f0` 将 `IAnnotateLayerProperties`、`ILabelEngineLayerProperties`、`ILabelEngineLayerProperties2` 分别装在 O+`0x1c`、O+`0x20`、O+`0x24`；QI `0x78545c10` 分别匹配 `{01004142-0D1C-11D2-A26F-080009B6F22B}`、`{01004143-0D1C-11D2-A26F-080009B6F22B}`、`{59A8EEBA-8EF7-4143-B497-4EFCAF6A2405}` 并返回相应指针。构造函数采用 vbtable `0x7857cb2c`（`[-0x14,0xa0]`），持久化虚表为 O+`0xb4`；其 Load thunk `0x78547760` 减去 O+`0xb0` 的调整值，此构造路径写入 0，所以 P=O+`0xb4`。接口虚表分别为 `0x7857c98c`、`0x7857ca0c`；下表将访问器相对接口的存储地址与 `0x785468c0` 的 P 相对地址换算到同一 O。这里的接口属性名是静态绑定，不证明界面或运行效果。

## LELP：同一存储地址

| 读取器字段（Load 次序） | 类型库属性与访问器入口（put/get，省略 0x785 前缀） | 对象位置 O+ |
| --- | --- | --- |
| `expression` ② | Expression `463d0/463f0` | 3c |
| `short_74` ③ | IsExpressionSimple `46420/46440` | 40 |
| `double_54` ⑤ | Offset `46330/648f0` | 60 |
| `int32_60` ⑥ | LabelWhichFeatures `45fd0/45ff0` | 54 |
| `short_5c` ⑦ | CreateUnplacedElements `46020/46040` | 58 |
| `short_4c` ⑧ | FeatureLinked `34910/348e0` | 68 |
| `short_4a` ⑨ | UseOutput `45da0/45dc0` | 6a |
| `int32_80` ⑩ | SymbolID `46310/46560` | 34 |
| `double_34` ⑪ | AnnotationMinimumScale `460a0/46070` | 80 |
| `double_3c` ⑫ | AnnotationMaximumScale `60dd0/460c0` | 78 |
| `short_48` ⑬ | DisplayAnnotation `46140/46160` | 6c |
| `name` ⑭ | Class `45f80/45fa0` | 4c |
| `string_b` ⑮ | WhereClause `46190/461b0` | 50 |
| `int32_2c` ㉓ | Priority `46120/460f0` | 88 |
| `short_46` ㉕（版本 ≥4） | AddUnplacedToGraphicsContainer `46250/46270` | 6e |

String 访问器用 `String::operator=` / `MakeBSTR` 操作同槽；BOOL/枚举仍原样保留，不因属性名将非规范值归一化。`short_ver1` 和 `short_ver6` 读入局部变量，不能据此绑定成员。`double_24`、`double_1c`、`int32_14`、`int32_7c` 此次没有完成可靠的公开访问器对位，不命名；尤其不能把任何整数直接当作要素 OID。对象槽的行为、版本门和短读限制仍以现有代码及各自研究为准。

## ALPC：集合 ID、尾值及空条目

QI `0x785313d0` 将集合接口 `{1D5849F2-0D33-11D2-A26F-080009B6F22B}` 和集合接口 2 `{2B35E113-9F11-43D5-914F-5C7055DB37BC}` 分别返回 O+`0x18`、O+`0x1c`；构造函数 `0x785310b0` 同时安装对应虚表。vbtable `0x78579a1c` 为 `[-0x14,0x5c]`，持久化 thunk `0x78533f10` 对本构造路径得到 P=O+`0x70`。

集合 2 的类型库声明 `Add(Item, ID)`、`QueryItem(Index, Item, ID)`、`Properties(ID)`。`0x785318a0` 把接口 this+`0x44`（O+`0x60`）分配给新节点的 +4、递增该计数器并返回 ID；`0x785319a0` 按索引返回同一节点 +4。Load `0x78531be0` 对非空项将版本 ≥2 的流内 int32 写入节点 +4，随后以该值索引集合；版本 <2 则使用并递增 P-`0x10`。版本 ≥2 的尾 int32 写入 P-`0x10`，即同一 O+`0x60` 分配计数器。因此可将条目值称为**集合条目 ID**，尾值称为**下一次 Add 所用的 ID 计数器**；不等同于要素 OID，不承诺所有外部输入下 ID 唯一或单调。

**尚需修复和验证的结构差异**：Load 只有在 `LoadObject` 成功且结果非空时才读取该条目 ID；空对象则不读取 ID，也不插入节点。当前 Rust 循环对版本 ≥2 的空槽仍无条件读四字节并保留 Null 条目，可能误消费下一字段。此发现来自静态控制流，不证明实际保存器会产生空项；本轮未改变解析行为，不能把 ALPC 全版本/空项路径视为闭合。还需比对 Save、构造空项/截断输入并验证游标、登记回滚和后继字段。

## 可勾边界与剩余

- 可独立落账：上述 15 个 LELP 已读字段与 10.1 类型库属性的静态同槽绑定；ALPC 非空项 ID 及尾计数器的静态角色。源码仅更新这些已绑定字段的注释，保留现有公共字段名和原始值。
- 不可整体勾选现有 ALPC/LELP 业务语义条目：LELP 仍有未绑定字段、ALPC 空项行为与读取器存在差异；旧条目中“OID”等描述也不可从集合 ID 推出。
- 未进行 ArcMap 10.1 源工程和 QGIS 转换结果的人工检查；类型库枚举、静态反编译和 Rust 测试不替代这一步。受控修改表达式、类名、SQL、比例和标注选项后的保存/复读仍是后续运行验证。

## 本轮验证

`cargo fmt --check`、`cargo test` 和 `git diff --check` 均通过；Cargo 报全局缓存自动清理权限不足，但编译和测试成功。本轮只有注释及研究文档变更，没有新增解析行为或新增测试；现有测试不能验证本文新绑定的属性语义，也尚未覆盖上述 ALPC 空项差异。

主审按上述 Load 条件修复 ALPC：保留流内空槽用于定位，但仅非空条目消费 ID。构造一个空项后紧跟回引，验证其 ID、尾计数器、后继字节和逐字节截断回滚，见 `tests/terra_nextpass_expression.rs`。LELP 的未定字段和保存器是否会写出空项仍开放。

## parse 命令读取标注表达式

按上文 LabelPlacement.dll 10.1 的 `Expression` 访问器及 Load 字段对位，
`parse` 从 `/Maps` 的 FeatureLayer 进入 ALPC，再从 LELP 读取表达式原文。
仓库参考工程到达三条 LELP v7，键为 60、202、239。命令行回归逐条
检查三个已知表达式出现在 LELP 标注类诊断中，以区分同一图层上的
DisplayExpressionProperties。表达式求值、转换和 ArcMap/QGIS 人工检查仍未进行。

## 图层与标注类的显示开关

复核 FeatureLayer.dll 10.1.0.3035 x86（SHA-256
`2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`）。
用 Wine 的 `LoadTypeLibEx` 读取 10.1 Carto 类型库，得到 IGeoFeatureLayer
IID `E156D7E5-22AF-11D3-9F99-00C04F6BC78E`。DisplayAnnotation
访问器的 x86 虚表偏移为 `0x90/0x94`。原始指令中主 QI
`0x6f6e7a40` 将该 IID 映射到完整对象 `+0xd8`。
构造链 `0x6f6dc367` 安装虚表 `0x6f7acfec`，对应访问器
`0x6f6e52c0/0x6f6e52f0` 读写接口指针 `+0x160` 的 short，
即完整对象 `+0x238`。同一构造链在 `0x6f6dc353` 安装完整对象
`+0xd0` 的持久化虚表 `0x6f7acfb0`，其 Load/Save 分别在
`0x6f6e9ad4` 和 `0x6f6eb3ea` 按该接口指针 `+0x168` 读写
同一个 short。这对应读取器现有的 `short_168`。

命令行保留图层开关和 LELP 类开关各自的原始值。参考工程三个图层的
图层开关分别为 `0, 0, -1`，对应三个标注类均为 `-1`，回归分别断言
两级值。实际绘制还受图层可见性、比例尺等条件影响，ArcMap 与 QGIS
人工验收尚未进行。

## 标注字体

LELP 的 ITextSymbol 槽进入 DisplayCore TextSymbol Load
`0x6ebd0a60`，按[对象字段序](arcmap-10.1-element-bodies.md)
进入 IFont 槽。新 StdFont 对象复用
[oleaut32 持久化布局](arcmap-10.1-ole-stdfont.md)，输出名称原始字节、
字符集、字重、字号 CY 和样式位。名称编码由写侧系统 ANSI 代码页决定，
读取器保留原始字节。字体槽及文字符号槽的回引仍保留在解析对象中，
未将其解析成共享字体的属性。

参考 MXD 的 LELP 键 60、202、239 都到达 StdFont。三个名称均为字节
`cb ce cc e5`，字符集为 134，字重为 400，字号分别为 12、8、16 磅。
命令行回归逐条断言原始值。字体转换及 ArcMap/QGIS 人工验收尚未进行。

## 标准标注引擎的放置属性

LELP 的 Offset、Priority 由上文访问器对位确认。标准引擎使用的放置属性
对象及其三个子槽复用 [LabelPlacement.dll 10.1.0.3035 的 Load 证据](arcmap-10.1-featurelayer-tail.md#bolp-的原生版本门和标注对象宿主)。
`parse` 输出 LineLabelPosition 的原始字段和线、点优先级数组。参考工程中
三个 LELP 的 Offset 均为 0、Priority 分别为 1、2、3，三者均到达点和线的
优先级数组。回归验证三个类均在各自的诊断中输出放置属性。
LineLabelPosition 各标志的属性对位、Maplex 放置配置及 ArcMap/QGIS
人工验证仍待完成。

### Line Label Position 字段对位

使用 10.1 Carto 类型库和 LabelPlacement.dll 10.1.0.3035（SHA-256
`8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`）。
类型库导出的 ILineLabelPosition IID 首段为 `2442958B`，GetClassID
`0x7855afb0` 使用常量 `0x7857b02c`，CLSID 首段为 `2442958C`。

构造指令 `0x7855b037` 将接口表 `0x7857d07c` 安装至对象
`+0x18`。同一构造函数使用虚基表 `0x7857b274` 的
`+4 = 0x30`，把持久化表安装到对象 `+0x44`。Load thunk
`0x7855b7b0` 经对象调整值把该指针传给 `0x7855b4e0`。
两个接口相差 `0x2c`。据 PE 虚表和访问器指令，Load 的连续 short
`-0x26..-0x12` 对应 ILineLabelPosition 的 `+0x06..+0x1a`，
顺序为 Above、Below、OnTop、Left、Right、InLine、AtStart、AtEnd、
Parallel、Perpendicular、Horizontal。后续 double `-0x0c`
对应 Offset（getter `0x7855b4b0`，接口 `+0x20`）。
v2 尾 short `-0x28` 对应 ProduceCurvedLabels（getter
`0x7855b190`，接口 `+0x04`）。v0/v1 原始流没有尾 short。

命令行保留原始值，参考工程的三个标注类均有回归覆盖。Maplex 放置配置、
单位转换及 ArcMap/QGIS 人工验证仍待完成。

### Maplex 标注类的持久化入口

复用上节同一 SHA-256 的 LabelPlacement.dll 10.1.0.3035，从注册 CLSID、
MSVC RTTI 和 objdump 三处定位。MaplexLabelEngineLayerProperties
`20664808-0D1C-11D2-A26F-080009B6F22B` 的构造器在
`0x7856077a` 安装持久化虚表 `0x7857d844`，其 GetClassID、Load 和 Save
由调整器跳转到 `0x78560650`、`0x785615e0`、`0x785618f0`。
Load 在 `0x78561617` 拒绝有符号版本大于 11，Save 从 `0x7857972c` 写 v11。
该类具有自己的 Load 入口和版本序列。

| 条件 | Load 流序 | 指令范围 |
| --- | --- | --- |
| 固定前缀 | 版本 short、String、short；版本 ≤1 另读一个 short；double、LONG、三个 short、LONG、两个 double、short、两个 String | `0x7856160a..0x785616ea` |
| 固定对象段 | ITextSymbol；版本 <11 插入一个请求 BasicOverposterLayerProperties 对应接口的旧槽；表达式引擎、IEnvelope | `0x78561718..0x785617d0` |
| 固定标量段 | 两个 double、两个 LONG | `0x785617df..0x7856180f` |
| 版本化对象及标量 | ≥3 对象（请求 IID 常量 `0x7857cb7c`）；≥4 short；≥5 对象（请求 IID 常量 `0x7857d86c`）；仅 v6 一个 LONG；≥8 一个 LONG；仅 v9 一个 short | `0x78561818..0x785618c7` |

固定对象段的 IID 依次由 `0x7857cb5c`、`0x7857afdc`（旧分支）、
`0x7857a248`、`0x7857cb4c` 给出。旧对象读取到临时槽，紧接着释放；
即使忽略其业务属性也须消费对象体与共享键。v11 Save 在 `0x78561a43`
将 ≥3 的槽写为空标记，在 `0x78561a5d` 写成员 `-0x10`。
该成员对应归档对象的接口与 v5 起的读侧请求一致。

MaplexOverposterLayerProperties `20664808-41C9-11D1-880A-080009EC732A`
构造指令 `0x78563cde` 安装持久化虚表 `0x7857ec80`；GetClassID、
Load、Save 分别由槽调整器进入 `0x78566d00`、`0x78566d40`、
`0x785675b0`。该类与 Map 层级的 MaplexOverposterProperties 分属不同 CLSID。
后续仍须核对 MaplexOverposterLayerProperties 的持久化体及其嵌套子槽，
并从 ALPC 接入 Maplex 标注类。尚未以实档或 ArcMap 运行验证。

### Maplex 标注类接线

沿用上节 10.1 PE 证据。Maplex v3 起请求的 `0x7857cb7c` 与标准
LELP 的 IGraphicsContainer IID 相同，复用图形容器分派。Maplex 的
AnnotationClassID 访问器位于对象 `+0x24` 的虚表 `0x7857d7a4`，
getter/setter `0x7855be40/0x7855fda0` 访问接口 `+0x10`。
它与 v8 起的 Load 目的地址一致，均为完整对象 `+0x34`。

ALPC 接入两个标注类的共有字段。Maplex 放置属性槽可读空值或回引，
新对象暂在槽头停止，保留已读标注类并撤回该未知对象的登记。构造
v1/v6/v9/v11 验证尾段差异、逐前缀截断回滚及未知放置体停止。
参考工程仅验证既有标准标注路径。Maplex 放置对象体和目标软件人工验收仍开放。

### MaplexOverposterLayerProperties 的版本布局

在相同的 10.1.0.3035 LabelPlacement.dll 中，复核持久化虚表
`0x7857ec80` 的 +0x14 和 +0x18。调整器 `0x7856a6e0` 跳到
Load `0x78566d40`，调整器 `0x7856a5b0` 跳到 Save `0x785675b0`。
`0x78563ba0` 对应 IsDirty。Load 接受有符号版本 4–22。
Save 在 `0x785675db` 将版本表 `0x78579718`（前三项 -1、21、22）
与参数 3 传入版本选择助手，仍须复核目标文档版本与表项的具体映射。

下表按 10.1 Load 顺序列出流形态。short、LONG 和 double 分别为
2、4、8 字节；旧版混合宽度见表。
对象槽的地址后括号内为 IID，供按接口继续追到实现类。

| 版本条件 | 顺序 | 指令范围 |
| --- | --- | --- |
| 全部 | 3 short、4 LONG、2 short、double、LONG、double、LONG | `0x78566da3..0x78566f0a` |
| ≤11 | ReadString 形式的 BSTR，读取后释放 | `0x78566f17..0x78566f57` |
| 全部 | 对象槽（`261A4371-D9D5-11D2-A806-CC9F870BCD5A`）、3 short、double、short、3 double、short | `0x78566f71..0x7856706b` |
| 全部 | 对象槽（`20664808-3A22-12A1-FA09-0A56FF3C731A`）、short、double、3 short | `0x78567090..0x78567118` |
| ≥7 / <7 | 4 double；旧版改为 LONG、float、LONG、float，读取后接回公共字段 | `0x78567129..0x7856718b`、`0x7856754d..0x7856759f` |
| 全部 | short、3 LONG、2 short、double | `0x785671aa..0x7856724a` |
| ≥6 | short | `0x7856725b..0x7856726e` |
| ≥7 | short、double | `0x7856727b..0x785672ae` |
| 8–13 | 临时对象槽（`20664808-0A3A-1EAC-91D3-F3EF422A61A5`），加载后释放 | `0x785672bf..0x78567300` |
| ≥9 | short、对象槽（`20664808-1A42-1FA1-FCD1-0B532F3C8FFA`） | `0x78567302..0x7856734d` |
| ≥10 | 读入临时 LONG | `0x78567355..0x7856736b` |
| ≥12 | 对象槽（`20664808-77AF-65E1-FAEF-18E6F9EC732A`） | `0x78567398..0x785673b2` |
| ≥13 / ≥15 | LONG / AfCore String | `0x785673c1..0x785673f3` |
| ≥16、≥17、≥18、≥19 | 每个版本门各加一个 short | `0x785673f9..0x7856746c` |
| ≥20 / ≥21 | short、double / 两个 LONG | `0x78567479..0x785674e4` |
| ≥22 | IPropertySet 对象槽（`F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`） | `0x785674ed..0x78567505` |

Load 在 `0x78567519` 调用结束处理再返回。当前仅完成静态流序和
IID 对位，嵌套对象仍需逐类定位 Load，字段的业务语义也需访问器证据。
未在真实 Maplex 工程或 ArcMap 中验证。

### Maplex 放置属性读取器

沿上表将 v4–v22 的布局接入 Maplex 标注类，在已确认边界保留对象体
原始字节。首槽复用相同 IID 的 PointPlacementPriorities 读取器，v22
末槽复用 ObjectStream 的 PropertySet。两者的对象键进入 ALPC 共享引用表。
其余四槽可读取空值和回引；首次遇到未支持的新对象时停在子对象头，
保留此前已读字段与键。普通短读使调用回滚。

构造测试遍历 v4–v22，验证对象结束位置、点优先级与属性集键、截断及
未知子对象边界。ALPC v11 另测非空放置体后仍能读取标注类及集合条目 ID。
另外四槽的首次定义、业务字段命名、真实 Maplex 工程和目标软件验证仍开放。

### Maplex 标注堆叠属性

以同一 LabelPlacement.dll 10.1.0.3035 为依据。第二槽请求的 IID
位于 `0x7857d994`，MaplexLabelStackingProperties 的主 QI
`0x78562610` 返回对象 `+0x18`。MSVC RTTI 对位持久化虚表
`0x7857d96c`，GetClassID `0x78562350` 写出注册 CLSID
`20664808-41C9-11D1-840A-08ABC9ED731A`，Load 和 Save 的 thunk
分别跳到 `0x78562930` 和 `0x78562b30`。Load 读有符号 short，
拒绝大于 3，再读一个 LONG；v2 起读有符号计数，每项读取一个 BSTR
与三个 short（`0x785629e2..0x78562a97`）。非正计数跳过循环。
v3 另读三个 LONG（`0x78562ac0..0x78562af4`），Save 写 v3。
分隔符以内联字段排列，无独立对象头或键。

Maplex 放置对象第二槽现可读取该类的首次定义，透传外层键登记。
合成放置测试覆盖非空分隔符、原始标志、尾字段和父对象结束位置。
字段业务含义及 ArcMap/QGIS 人工验收仍待完成。

### Maplex 沿线偏移

同版 LabelPlacement.dll 的 MaplexOffsetAlongLineProperties 主 QI
`0x78563760` 对放置体第四槽 IID `0x7857db84` 返回对象 `+0x18`。
RTTI 与虚表 `0x7857db64` 对位 GetClassID `0x78563610`、
Load `0x785638d0` 和 Save `0x78563a90`。GetClassID 写出
`20664808-4FA1-C1D1-8C0A-08A2C9ED531A`，与登记表一致。

Load 拒绝有符号版本大于 2。v1 读 LONG 计数与若干内联
BSTR、double、double 记录（`0x7856392b..0x785639ce`），非正计数
跳过循环，旧记录在原生内不保留。v1 的字符串调用点 `0x7856396d`
使用 IAT `0x785791cc` 的 AfCore 自由函数 ReadString，前缀为
UTF-16 码元数，非空值另含终止零，与堆叠分隔符共用读取器。
v2 读取两个 LONG、两个 double、
LONG、short（`0x785639db..0x78563a4a`）；Save 写 v2。其余接受版本
无后继体。读取器保留上述原始值，沿宿主表登记对象键。
构造测试验证 v1 截断与 v2 非零字段及父对象边界，实际偏移单位及
ArcMap/QGIS 人工验收仍待核对。

### Maplex 旋转属性

同版 LabelPlacement.dll 的 MaplexRotationProperties 主 QI
`0x7856c7a0` 对第五槽 IID 常量 `0x7857ecd8` 返回对象 `+0x18`。
RTTI 定位持久化虚表 `0x7857f128`。GetClassID、Load 和 Save 的
调整器分别跳转到 `0x7856cb70`、`0x7856cbb0`、`0x7856cc70`。
CLSID 常量 `0x7857c6e4` 为 `20664808-BBA1-CCD2-8967-F453C9ED732A`，
与介质登记表一致。Load 拒绝有符号版本大于 2，之后依次读 short、
LONG、AfCore String 和 short；v2 增加一个 short
（`0x7856cbf8..0x7856cc4c`）。Save 在 `0x7856cc90` 引用的
版本常量为 2。

合成放置体覆盖 v1/v2 旋转对象、非空字符串、对象键与后继字段，
另测逐字节截断和超限版本。各标量的业务语义及真实工程验收仍待核对。

### Maplex 旧版放置体的临时 BSTR

同版 LabelPlacement.dll 的放置体 Load `0x78566d40` 在
`0x78566f17` 比较有符号版本与 11。旧版在
`0x78566f2a` 经 IAT `0x785791cc` 调 AfCore 自由函数
ReadString，读出临时 BSTR 并释放。字符串前缀按 UTF-16 码元
计数，非空字符串另含终止零。v15 起另一个字符串由
`0x785673f3` 调 AfCore String::Read，仍使用字节长度。
构造放置体验证单码元临时 BSTR 后的对象槽和截断。
真实旧版宿主及 ArcMap/QGIS 人工验收仍待核对。

### Maplex 放置属性的旧词典槽

同版 LabelPlacement.dll 的 MaplexDictionary 主 QI `0x7855bde0`
在 `0x7855bde7` 比较放置体 v8–v13 临时槽所请求的
`20664808-0A3A-1EAC-91D3-F3EF422A61A5`，匹配时返回对象
`+0x18`。RTTI 对位该类的持久化虚表 `0x7857d2a0`，复用已确认的
[MaplexDictionary Load 与 Save](mxd-terra-round9-old.md#maplex-标注引擎与地图级属性)。
原生 Load 在读取后释放临时指针，Rust 仍完整消费体并沿宿主表登记对象键。
构造测试覆盖 v8–v13 的首次定义和后续字段，其他版本验证不消费该槽。
ArcMap 和 QGIS 人工验收仍未进行。

### Maplex 放置属性诊断

`parse` 为 Maplex 标注类报告放置体版本、点优先级、堆叠分隔符、
沿线偏移 double 原始值和旋转配置字符串。可解析到外层回引时报告引用键。

现有介质提取物中，10.1 的 132 条 `/Maps`、`/PageLayout` 流与
10.8 的 136 条同名流均未命中 Maplex 标注类和放置体的两个 CLSID
字节模式。10.1 本地 63 份 MXD 与参考工程，共 64 份，`parse`
均正常退出，均未输出 Maplex 放置体诊断。退出状态不保证输入中的
所有对象已读取。Maplex 放置路径仍以构造测试和 10.1 原生静态取证为依据，
真实 Maplex 工程及 ArcMap/QGIS 人工验收尚待完成。

#### 沿线偏移的属性对位

10.1 Carto 类型库由 Wine `LoadTypeLibEx` 读出
IMaplexOffsetAlongLineProperties 的六组访问器。QI 返回的对象
`+0x18` 使用虚表 `0x7857db24`。构造指令 `0x78563673` 设置
虚基表 `0x7857b274`，`+4` 表项为 `0x30`；`0x785636b1`
据此在对象 `+0x44` 安装持久化表。两个接口相差 `0x2c`。

| v2 流字段 | 属性 | getter | 属性接口 / Load 偏移 |
| --- | --- | --- | --- |
| LONG 1 | PlacementMethod | `0x7856d840` | `+0x04 / -0x28` |
| LONG 2 | LabelAnchorPoint | `0x78547a40` | `+0x08 / -0x24` |
| double 1 | Distance | `0x785637e0` | `+0x10 / -0x1c` |
| double 2 | Tolerance | `0x78563830` | `+0x18 / -0x14` |
| LONG 3 | DistanceUnit | `0x78563880` | `+0x20 / -0x0c` |
| short | UseLineDirection | `0x785645e0` | `+0x24 / -0x08` |

读取器和命令行采用这些属性名，保留枚举和开关的原始数值。
单位枚举的语义映射、坐标与纸面距离换算以及目标软件人工验收仍开放。

#### 沿线偏移的距离单位

直接核对 10.1 Carto 类型库（SHA-256
`8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）。
用 `ITypeInfo::GetFuncDesc` 取得 IMaplexOffsetAlongLineProperties
的 DistanceUnit setter（memid `0x60010008`），沿用户定义参数的
`hreftype` 调用 `GetRefTypeInfo`，从 `GetVarDesc` 得到五个
VT_I4 常量。0 为 esriMaplexUnitMap，1 为 esriMaplexUnitMM，
2 为 esriMaplexUnitInch，3 为 esriMaplexUnitPoint，4 为
esriMaplexUnitPercentage。命令行同时输出原值和成员名，未知值原样保留。
测试覆盖五项和两侧未知值。比例基准、距离换算和 ArcMap/QGIS 人工验收仍开放。

#### 沿线偏移的放置方式和锚点

在同一 10.1 Carto 类型库上，沿 IMaplexOffsetAlongLineProperties
的 PlacementMethod 和 LabelAnchorPoint setter（memid
`0x60010000`、`0x60010002`）的用户定义参数调用
`GetRefTypeInfo` 和 `GetVarDesc`。PlacementMethod 的 VT_I4
值 0–4 依次为 esriMaplexBestPositionAlongLine、
esriMaplexBeforeStartOfLine、esriMaplexAlongLineFromStart、
esriMaplexAlongLineFromEnd、esriMaplexAfterEndOfLine。
LabelAnchorPoint 的值 0–2 依次为 esriMaplexCenterOfLabel、
esriMaplexNearestSideOfLabel、esriMaplexFurthestSideOfLabel。

`parse` 保留原始数值并输出已知成员名。构造测试覆盖已知放置方式和
未知锚点值。实际绘制及目标软件人工验收仍开放。

#### Maplex 旋转字段与角度类型

10.1 Carto 类型库的 IMaplexRotationProperties 五组属性访问器与同版
LabelPlacement.dll 的 QI、虚表和 Load 指令逐项对位。构造器在
`0x7856c659` 安装对象 `+0x18` 的接口表 `0x7857f0d0`，在
`0x7856c66a` 按虚基表 `0x7857d59c` 的 `+4 = 0x28` 安装
对象 `+0x3c` 的持久化接口。两接口相差 `0x24`。

| 属性 | getter | 属性接口 / Load 偏移 |
| --- | --- | --- |
| Enable | `0x7856c830` | `+0x08 / -0x1c` |
| RotationType | `0x7856c880` | `+0x0c / -0x18` |
| RotationField | `0x7856c8b0` | `+0x10 / -0x14` |
| PerpendicularToAngle | `0x7855b370` | `+0x14 / -0x10` |
| AlignLabelToAngle（v2） | `0x7855b3c0` | `+0x16 / -0x0e` |

沿 RotationType setter 的类型库引用（memid `0x60010002`）得到
VT_I4 值 0–3，依次为 esriRotateLabelGeographic、esriRotateLabelArithmetic、
esriRotateLabelRadians、esriRotateLabelAV3。`parse` 以属性名输出，
保留原始枚举值和开关值。构造测试覆盖四个成员及未知值。实际角度
计算和 ArcMap/QGIS 人工验收仍开放。

#### 堆叠分隔符和行限制的属性对位

10.1 Carto 类型库的 IMaplexLabelStackingProperties 与同版
LabelPlacement.dll 的接口表 `0x7857d92c` 对位。RTTI 和构造指令
`0x78562449..0x78562452` 确认属性接口与持久化接口分别在完整对象
`+0x18` 和 `+0x80`。Load 首个 LONG（持久化接口 `-0x64`）
对应 StackJustification（getter `0x7856d840`，属性接口 `+0x04`）。

QuerySeparator `0x78562700` 按类型库参数顺序返回 Visible、
SplitForced、SplitAfter，访问属性接口 `+0x20/+0x34/+0x48`
的数组指针。Load 的三个 short 按同一顺序加入对应数组
（`0x78562a2d..0x78562a97`）。v3 尾三个 LONG 对应
MaximumNumberOfLines、MinimumNumberOfCharsPerLine、
MaximumNumberOfCharsPerLine。getter `0x78564a00/0x78562890/0x785628e0`
读取属性接口 `+0x58/+0x5c/+0x60`，与 Load 的
`-0x10/-0x0c/-0x08` 一致。

读取器和诊断采用属性名，保留开关和对齐枚举原值。非零构造输入覆盖
三个标志和三个行限制。实际堆叠行为及目标软件人工验收仍待核对。

#### 堆叠对齐枚举

读取 10.1 Carto 类型库中 IMaplexLabelStackingProperties 的
StackJustification setter（memid `0x60010000`），沿参数的
`hreftype` 调用 `GetRefTypeInfo` 与 `GetVarDesc`。
VT_I4 值 0–4 依次为 esriMaplexChooseBestJustification、
esriMaplexConstrainJustificationLeftOrRight、
esriMaplexConstrainJustificationLeft、
esriMaplexConstrainJustificationRight、
esriMaplexConstrainJustificationCenter。
命令行输出原始值和已知成员名。构造测试覆盖全部成员和未知值，
实际绘制与 ArcMap/QGIS 人工验收仍待验证。
