# MXD 解析：显示、元素与格式字段缺口复核

日期：2026-11-16。范围限于本轮分配的 TODO 行 203、205、209–210、220–225、
237、253–263、269。没有修改 TODO、README、模块接线或其他研究记录。

## 目标身份与证据边界

本轮显示类证据沿用现有研究记录中已核对的 ArcMap 10.1.0.3035 原生 DLL
身份：DisplayCore.dll SHA-256
`9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，
SystemCore.dll SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`；
元素体所用 LayerLib、CartoXProvider、GraphicElements 文件身份及地址见
[`arcmap-10.1-element-bodies.md`](arcmap-10.1-element-bodies.md)。相关对象的
字段序、版本门和读写行为来自 10.1 静态反编译。仓库参考 MXD 可提供少量
TextSymbol/StdFont/PropertySet 对位；许多显示对象、Hyperlink、九类 NumberFormat
均无实例。

本轮试图用 REA 对已核对摘要的 DisplayCore.dll 作独立函数分析。REA 当前没有
其他活动二进制会话；新鲜 Ghidra 导入在 `/tmp` tmpfs 空间不足时退出，未得到
函数证据。本轮不据此声称新增 REA 函数观察，也未清理任何研究资料。随后复用
仓库已有的 10.1 反编译与样本记录，并查阅 Esri ArcObjects 10.6/10.7 API 文档
作接口层旁证。较新 API 页面不能证明 10.1 的字段归属；只有 10.1 Load/Save
字段访问而缺少同一内存槽的 getter/setter 访问证据时，字段名仍标候选。

## TODO 行逐项结论

| TODO 行 | 状态 | 本轮结论与证据 | 剩余缺口 |
| --- | --- | --- | --- |
| 203 | 部分 | `display_color.rs`、`display_symbol.rs` 已按 10.1 Load/Save 闭合多类对象字段序及部分嵌套槽；`display-tails.md` 与 `display-symbols-round2.md` 记有 RGB/CMYK/HSV/HLS、符号字段、样本覆盖情况。DisplayCore DLL 身份已核验。 | 标量字段缺少 10.1 成员 accessor 对同一对象偏移的读写对位；类名、变量取值或跨版本 API 属性不能单独定性。RGB/颜色分量和多类符号数值仍原样保留。无样本的类型不得记为样本验证。 |
| 205 | 部分（静态现象已闭合） | 三处 short 的 10.1 Load 分支在读入后将对应内存槽写成 `GetDefaultRotateWithTransformValue` 返回值；Save 写该默认槽值。函数与字节范围见 `display-symbols-round2.md` 的 ArrowMarkerSymbol `0x6eb3ec40`、CharacterMarkerSymbol `0x6eb9b150` 和 PictureMarkerSymbol `0x6eb7a650` 记录；实现字段仍保存流内 short，注释说明覆盖。 | 该静态读后覆盖事实确定；无运行时样本证明目标程序对所有输入均如此，也没有这三类的样本字节佐证。本项只能闭合为反编译级“Load 读入后覆盖”，不能称 ArcMap 运行验证。 |
| 209 | 部分 | LayerLib 10.1 `ElementProperties::Load` `0x786b97f0` 的字段序/落点见 `arcmap-10.1-element-bodies.md`：两 String、版本旧 int32、CustomProperty、short、double、尾 int32；读写函数静态字段访问可确认。10.1 Carto 类型库列有 `IElementProperties`、`IElementProperties2`、`IElementProperties3`；接口文档列出 Name、Type、CustomProperty、ReferenceScale 等成员。实现注释已将这些名称明确降为候选。 | 尚未核对 10.1 对应 getter/setter 的虚表槽及函数体，证明其读取/写入偏移分别对应 Load 落点。故 `name`/`type_description`/`custom_property`/`reference_scale` 仍是接口成员名候选，不勾为确定业务语义。样本 EP 版本 6 的值只作边界辅助。 |
| 210 | 部分 | TextSymbol 的 10.1 Load `0x6ebd0a60` 字段序与样本可达范围见 `arcmap-10.1-element-bodies.md`、`arcmap-10.1-display-symbols-round2.md`；样本 TextSymbol 及 StdFont 实例对位支持字段边界。10.7 `ITextSymbol` 文档公开 Color、Font、Angle、Size、Alignment、Text 等接口成员，但无法把它们对应到 10.1 的原始标量组。 | 缺少 10.1 getter/setter 槽、接收对象偏移及 Load/Save 落点的三向对位。`name_string`、多组 int/byte/double、GUID 和尾字段不得以 API 属性顺序猜名；保留原名/原值。 |
| 220 | 部分 | 10.1 esriCarto 类型库 `carto.txt` 确认 `esriHyperlinkType` 枚举（GUID `1558D2D3-8D39-11D1-B981-080009EE4E51`）存在；后续 Esri API 常量文档列值 Document=0、URL=1、Macro=2、Script=3。Hyperlink 本体字段序/Load `0x6f6d9170` 见 `arcmap-10.1-hyperlink.md`。 | 10.1 类型库枚举成员值尚未通过 10.1 原件导出；且 Load 第一个 int32 与枚举的直接绑定尚缺同槽 getter/setter 或调用链证据。因此值序与该字段绑定仍为候选，不改解析字段类型。 |
| 221 | 部分 | API 将 `IHyperlink.LinkType` 定义为 `esriHyperlinkType`；Hyperlink Load 有两个 int32（this+4、this+0xc），可把首个 int32 列为 LinkType 候选。 | 仅接口成员存在和类型吻合不足以证明 this+4 对应 LinkType；需 10.1 `get_LinkType`/`put_LinkType` 的虚表槽及内部内存偏移，且解释第二个 int32。 |
| 222 | 部分 | API 将 Hyperlink 的 `Link` 与 `IHotlinkContainer` 的 `HotlinkField`、`HotlinkType` 分别定义；后者管理图层级、字段型/宏型 hotlink。Hyperlink Load 的 String 位于 this+8，记作 Link 候选。 | 仍缺 10.1 Hyperlink `get_Link`/`put_Link` 对 this+8 的偏移证据，以及它与 FeatureLayer `IHotlinkContainer` 字段的真实读写关系。不能据“字符串像 URL”定性。 |
| 223 | 部分 | 10.7 `INumericFormat` 文档说明格式对象定义数字格式，并列出 AlignmentOption、AlignmentWidth、RoundingOption、RoundingValue、ShowPlusSign、UseSeparator、ZeroPad；官方类文档区分 Numeric/Percentage/Fraction/Rate/Currency/LatLon/Angle/Scientific/Custom 九类。当前九类读取器按 10.1 Load 字段序解析。 | 没有把 10.1 每个类的字段偏移与对应属性 getter/setter 对位。格式类名和 API 属性表不能替代此证据；字段继续保留原始值。 |
| 224 | 部分 | 10.1 静态字段序确认 CustomNumberFormat 与 RateFormat 等类含 String；NumberFormat 文档及读取器按原字符串保存。 | 字符串与 API 成员的精确关联、格式语言/单位语义及读写 accessor 偏移未闭合；名称仍为通用 `String`/raw。 |
| 225 | 部分（布局静态闭合） | `number_format.rs` 与 `arcmap-10.1-number-format.md` 已记录：Percentage/Rate 在版本门后把外壳标量槽升级为内嵌 NumericFormat 核心；LatLon/Angle 的 NumericFormat 核心位于独立槽。对应正常/短体/版本错误/截断已有合成覆盖。 | 仓库样本九类全族及两种槽形态均无实例。需目标 10.1 运行或真实 MXD 字节升级证据；不得称样本验证。本轮未重复运行全局测试。 |
| 237 | 部分（静态现象已闭合） | `display-slots-round3.md` 记录 RandomColorRamp Load 的 int32 读入后内存槽清零、PictureFillSymbol 两 double 读后即弃且 Save 写 0、MultiPartColorRamp Load/Save 版本门不对称。`color_ramp.rs` 与 `picture_fill_symbol.rs` 保留这些流字节。 | 均无对应类样本实例；运行中对象值未观察。需要真实样本或目标程序运行对位。不得从字段值推断业务名称。 |
| 253 | 部分 | MarkerElement Load `0x7504ea50` 与字段读取器已实现，前缀包含 ElementProperties、符号/点几何槽、版本尾段；10.1 原生类身份、字段序见 `arcmap-10.1-element-bodies.md`。 | 参考样本 0 个实例；尚无真实字节/目标软件验证。未知符号和剩余尾段仍按停止点保留。 |
| 254 | 部分 | LineElement Load `0x7504a100` 与字段读取器已实现，包括 ElementProperties、符号/几何槽及版本条件尾字段；字段序来自 10.1 静态反编译。 | 参考样本 0 个实例；尚无真实字节/目标软件验证。 |
| 255 | 部分 | 两轮显示符号族读取器已实现所闭合类及槽分派，依据 `display-symbols-round2.md`、`display-tails.md`、`display-slots-round3.md` 的 10.1 Load/Save 证据。 | 两轮闭合符号类在参考样本中 0 实例；只由合成输入验证，目标软件中的实际对象边界未核对。 |
| 256 | 部分 | `display_color.rs` 有五类已闭合颜色读取器，Load/Save 字段序和类身份见 `display-tails.md`。 | 颜色族样本实例不足以覆盖这些类与分支；标量业务语义未定，需 10.1 accessor 偏移对位或真实样本。 |
| 257 | 部分 | Hyperlink 读取器接受版本 0，字段序无版本分支；合成测试已有版本 0 完整体。 | 仓库样本 Hyperlink 循环计数为 0，无版本 0 字节；不能作为样本分支闭合。 |
| 258 | 部分 | Hyperlink 循环走 ObjectStream 通用对象头解析，可处理零 GUID 空槽与键回引；对象流基础语义有 10.1 共享实现证据。 | Hyperlink 无样本条目，不能证明该 Hyperlink 宿主循环两种形态均由 ArcMap 10.1 产生或接受；需要真实样本/运行取证。 |
| 259 | 部分 | `number_format.rs` 已实现九类体读取器与每类静态版本门；`arcmap-10.1-number-format.md` 记有短体字段序；可构造分支有合成正常/异常覆盖。 | 九类短体在参考样本均无实例，语义映射仍未闭合；不能把测试结果写成格式样本证据。 |
| 260 | 部分 | Percentage/Rate 同槽升级及 LatLon/Angle 独立槽结构已在 10.1 Load/Save 静态对照中分开实现；核心读取器测试覆盖构造形态。 | 样本 0 实例，缺少真实流的对象体边界证据。 |
| 261 | 阻塞（保守实现维持） | 10.1 AngleFormat Load 版本 ≥2 调 NumericFormat 核心但不检查其失败 HRESULT；当前严格读取器失败即回滚/报错，未仿原生继续读。静态反编译证据与差异已在 number format 记录中说明。 | 原生忽略失败后的游标/对象状态依赖可运行观察或精确指令级复核；REA 导入本轮因 ENOSPC 未完成。无证据前不放宽错误处理。 |
| 262 | 部分 | oleaut32 2.40.4275 静态代码显示 StdPicture 第二字 Save 写 0、Load 不校验；图像数据头会依魔数分流解码器。现解析器保留 `word2` 与图像 blob，不猜语义、不解码未知格式。 | 该 oleaut32 不是已确认的 ArcMap 10.1 OS 运行时 DLL；样本 StdPicture 0 实例。第二字语义和各解码器入口未闭合，需目标 OS 运行库及真实图像样本。 |
| 263 | 阻塞（跨版本验证） | 介质 oleaut32 2.40.4275 的 StdFont/StdPicture Load/Save 静态证据与 MXD 样本 37 个 StdFont 体逐字节吻合；运行环境版本信息见 `arcmap-10.1-ole-stdfont.md`。 | XP/ArcMap 10.1 实际使用的系统 oleaut32 5.1.x 未获取，未对两版本做函数/流输出对比。兼容性不能由旧介质 DLL 或一组 MXD 样本证明。 |
| 269 | 部分（实现与合成测试） | SystemCore.dll 10.1 Load `0x7ffd3080` 对版本 0 与 1 走同字段路径，Save 恒写 1；实现接受 0。新增单元测试覆盖版本 0 空集合、版本 2 拒绝回滚、版本 0 头截断回滚。`cargo test property_set` 本轮通过。 | 版本 0 无参考样本；只记录静态支持与合成测试，不勾真实样本佐证。 |

## 本轮代码与检查

- 修改 `src/objects/element_body.rs`：ElementProperties 的四个接口属性映射保留为
  候选，注释明确缺少 10.1 getter/setter 偏移证据。
- 修改 `src/objects/ole_font.rs`：明确 StdPicture 第二字只观察到 Save 写 0、
  Load 不校验；语义仍未知。
- 修改 `src/objects/property_set.rs`：增加版本 0 正常和版本越界/截断异常的
  单元测试。
- 新增本研究报告。未改 TODO/README/`mod.rs`、robustness 清单、其他研究记录；
  无需新增模块接线。
- 初始全局基线 `cargo fmt --check` 与 `cargo test` 由主代理执行并通过；本轮运行
  `cargo test property_set`，PropertySet 新增 3 项单元测试及既有 FeatureLayer
  PropertySet 3 项集成测试均通过（其余匹配项为 0 tests）。命令有一条 Cargo
  全局缓存自动清理权限警告，但构建和测试退出码为 0。对本组 3 个 Rust 文件
  单独运行 `rustfmt --check --edition 2024` 与 `git diff --check` 均通过；其余
  全局验证留主代理统一执行。ArcMap 10.1 与 QGIS 人工目视验收均未做。

## 接口旁证（不是 MXD 字段归属证据）

10.1 本地类型库 dump 确认 `IElementProperties2`、`IHyperlink`、
`esriHyperlinkType` 等类型名/GUID；该 dump 没有接口 accessor 成员布局或枚举值。
官方后续版本 API 页可帮助识别接口概念，但只用于候选：

- [IElementProperties](https://desktop.arcgis.com/en/arcobjects/10.7/net/IElementProperties.htm)
  与 [IElementProperties2](https://desktop.arcgis.com/en/arcobjects/10.6/net/IElementProperties2.htm)
- [ITextSymbol](https://desktop.arcgis.com/en/arcobjects/10.7/net/ITextSymbol.htm)
- [INumericFormat](https://desktop.arcgis.com/en/arcobjects/10.7/net/INumericFormat.htm)
- [IHyperlink.LinkType](https://desktop.arcgis.com/en/arcobjects/10.7/net/IHyperlink_LinkType.htm)、
  [IHotlinkContainer](https://desktop.arcgis.com/en/arcobjects/10.7/net/IHotlinkContainer.htm)
- [ArcObjects 常量表中的 esriHyperlinkType](https://desktop.arcgis.com/en/arcobjects/latest/java/api/arcobjects/constant-values.html)

这些页面版本晚于 10.1；没有 10.1 accessor 函数体偏移或目标运行证据时，不将
其成员名、枚举值序或行为当作 10.1 MXD 流内字段的确定语义。
