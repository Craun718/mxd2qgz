# Display 对象组补充（2026-11-18）

本记录对应分派时 TODO 原行 234、240–241、257、288–296（不含 297），并检查 329、357、359、361。按要求未修改 TODO、README、公共模块声明或共享 dispatcher；本轮新建本记录。结论优先引用 ArcMap 10.1.0.3035 安装介质静态研究和本地对象流分析。静态字段序、类型库和合成测试不等同于 ArcMap/QGIS 运行或目视验收。

## 分析工具与证据边界

- 目标版本沿用 ArcMap Desktop 10.1.0.3035。DisplayCore、FeatureLayer、SystemCore 和 LayerLib 的文件身份、SHA-256、加载基址与方法定位见既有 [元素体记录](arcmap-10.1-element-bodies.md)、[数字格式记录](arcmap-10.1-number-format.md)、[Hyperlink 记录](arcmap-10.1-hyperlink.md)及 [本地研究成果指南](arcmap-10.1-research-source.md)。本轮未重新提取或宣称核验运行中的 ArcMap。
- 可复用的关键函数地址：DisplayCore `TextSymbol::Load 0x6ebd0a60`、共享颜色核心 `0x6eb35310`、RgbColor `0x6eb3d8b0`、CmykColor `0x6eb34850`；LayerLib `ElementProperties::Load 0x786b97f0`；FeatureLayer `FeatureHyperlink::Load 0x6f6d9170`；RasterLayer 主 QI `0x7c73aaa0`、IPersistStream `Load 0x7c73bd40`。地址均是 10.1 静态映像 VA，不是运行时观测。
- 按用户补充尝试本地 REA 6.9.0：`doctor` 识别本机 Ghidra 12.2 与 Java 25；定向 `decompile` 对既有 DisplayCore DLL 调用无输出、长时间无响应，手动中断，stdout/stderr 保存在 `temp/luna-20261118/display/rea-textsymbol.*`。随后从 Ghidra 安装目录离线安装 PyGhidra 3.2.0 wheel 到本轮临时目录；启动脚本因缺少 JPype1 依赖失败（日志 `temp/luna-20261118/display/pyghidra-error.log`），本地 pip 缓存未找到该依赖。本轮没有成功通过 REA bridge 或 PyGhidra 新增反编译结果，也没有运行取证。
- 因此下文引用的原生证据均来自仓库既有研究记录及已存在反编译导出；本轮代码只修正已由类型库与原生接口槽对位确认的 Hyperlink 字段名，不把工具失败写成新证据。

## 指定原行状态

| 原行 | 状态 | 证据与边界 |
| --- | --- | --- |
| 234 | 部分 | `display_color.rs` 已读取 RgbColor、HsvColor、GrayColor、HlsColor、CmykColor；符号族也已有多个 10.1 体读取器。字段序及颜色值/分支有原生证据，但颜色与符号全部标量对公开属性的业务语义仍未完成；未知字段继续保留原值。见 [显示尾段研究](arcmap-10.1-display-tails.md)、[颜色续查](mxd-terra-host-color.md)。 |
| 240 | 部分 | LayerLib `0x786b97f0` 的 ElementProperties 前缀已解析；类型描述字符串等语义仍有候选字段，完整属性语义没有闭合。见 [元素体研究](arcmap-10.1-element-bodies.md)。 |
| 241 | 部分 | DisplayCore `0x6ebd0a60` 的 TextSymbol 字段序已解析；`mxd-terra-integrate-text.md` 静态对位了 VerticalAlignment、HorizontalAlignment、RightToLeft、Angle、若干间距/位置/方向字段与访问器。仍有字段、槽和 setter 行为未定，结构中保留原始组字段。 |
| 257 | 待验证 | Percentage、Rate、LatLon、Angle 的嵌套 NumericFormat 核心已有 10.1 静态 Load/Save 证据和读取器；本项要求的运行验证及真实对象边界仍缺。见 [NumberFormat 原生分析](arcmap-10.1-number-format.md)和 [续查记录](mxd-terra-next-format.md)。 |
| 288 | 部分 | DisplayCore 符号读取器已覆盖多个常见类；现有介质模板有 MultiLayerMarkerSymbol、CharacterMarkerSymbol 的局部体证据。未覆盖的符号实现类及其真实宿主边界仍需逐类取得实档，不能由 CLSID 命中推断完成。见 [符号/OLE 介质记录](mxd-round2-symbol-ole-20261117.md)。 |
| 289 | 部分 | 颜色读取器覆盖上述五类，但仓库参考样本中只有 RgbColor 实例；其他颜色类未从已确认宿主路径取得完整实档体边界。见 [颜色体实现说明](../src/objects/display_color.rs)及 [第四轮记录](mxd-round4-display-20261117.md)。 |
| 290 | 待验证 | Hyperlink 读取器按静态版本门接受版本 0，但 10.1 写侧恒写版本 1；尚无历史 v0 MXD 体字节。`tests/hyperlink.rs` 的 v0 用例是合成输入。 |
| 291 | 待验证 | ObjectStream 通用空标记和回引已有实现；Hyperlink 宿主循环没有非空实档来确认空/回引形态及其具体位置。 |
| 292 | 部分 | 九类 NumberFormat 的读取器按已知版本门处理短体；单元测试/合成输入不能证明真实历史体边界。九类真实短体仍待样本或受控旧版本写侧。 |
| 293 | 待验证 | 四种内嵌 NumericFormat 核心布局由 SystemCore 10.1 静态反编译确认，但没有真实对象流体边界。 |
| 294 | 部分 | AngleFormat `0x7ffb33a0` 对内嵌核心 Load 返回值不作检查，当前读取器按严格子集将核心失败作为整体失败；原生失败后游标和后续字段行为没有运行取证，不能安全地把两者等同。见 [数字格式记录](arcmap-10.1-number-format.md)。 |
| 295 | 部分 | 介质 OLEAUT32 2.40.4275.1 的 StdPicture 保存第二字为零、该版本 Load 读取后不使用，并按载荷首字分派解码路径；目标 Windows 实际 oleaut32 的解码/运行行为和跨版本兼容未验证。见 [OLE 持久化记录](arcmap-10.1-ole-stdfont.md)。 |
| 296 | 待验证 | 现有依据为介质 OLEAUT32 2.40.4275.1 的静态 Load/Save 与仓库样本 37 处 StdFont 字节吻合；目标操作系统运行时 oleaut32 的版本、跨版本互读和受控写出尚未核实。 |

## RasterLayer 前缀增量

RasterLayer.dll 文件身份从本机 10.1 介质清单复核：SHA-256 `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6`；`objdump -x` 显示 PE32/i386、ImageBase `0x7c710000`。静态定位链如下：

1. 主 QI `0x7c73aaa0` 将 `IID_IPersistStream`（.rdata `0x7c76778c`）比对成功路径返回 `object+0x9c`；指令在 `0x7c73abc3` 开始该 IID 比较，成功路径 `0x7c73b352..0x7c73b357` 加 `0x9c`。
2. IPersistStream RTTI 子表 COL 指针位于 `0x7c76c2ac`，指向 COL `0x7c774088`；COL 的 offset 字段为 `0x9c`，与 QI 返回位置相同。子表 `0x7c76c2b0` 的 `+0x14` 槽内容为 `0x7c73bd40`，与函数序言及其参数形态相符。
3. `0x7c73bd40` 对传入流读取四字节版本，`0x7c73bdf0..0x7c73bdf5` 的 `cmp [esp+0x3c],0x11; jg 0x7c73c211` 是有符号上限判断。顺序落下后，`0x7c73bdfb..0x7c73be08` 直接以 `lea 0x88(%ebx),%ecx` 调 AfCore `String::Read`；到首串字段为止，没有负值早退。随后才检查 String::Read 的 HRESULT 并进入第二条 String（`0x7c73be12..0x7c73be21`）。故 `i32` 负值通过本模块实现的版本门并继续解析首串；此结论只覆盖至首串，不外推后续版本分支。
4. 对象构造函数 `0x7c7367c0` 在对象基址 `+0x49*4 = +0x124` 调 `String::String`。结合 QI 返回的 IPersistStream 子对象偏移 `+0x9c`，Load 的 `this+0x88` 正好落在同一对象成员 `+0x124`，可复核首串目标是该 String 成员。当前导出里未找到能把这个成员绑定为 RasterLayer 的 `ILayer::Name` getter 的直接证据：唯一 `get_Name` 命名导出 `0x7c760cc4` 属于 `BasicRasterDataset` 跳板，不能据此赋予 RasterLayer 语义；第二串是 Load 栈上临时 String，也没有被认定为 getter 属性。因此实现继续使用 `first_string` 原值名称，不猜首/次串的业务含义。

新增 `src/layers/raster_layer.rs`，公开接口为 `RasterLayerPrefix10_1 { version: i32, first_string: String }` 与 `read_raster_layer_prefix_10_1(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<RasterLayerPrefix10_1>`。成功后游标停在第二条 String 前，不猜第一条 String 的属性名、不读取后续字段；对象体无通用长度，本结果不是完整 RasterLayer 解析。读取错误与超限均事务性回滚。模块尚未在 `mod.rs`、`layer_list` 或模型接线。

## MXD 侧解析适配检查

| 原行 | 状态 | 现有实现与缺口 |
| --- | --- | --- |
| 357 | 部分 | `src/objects/display_symbol.rs` 含多个点/线/面符号体读取器，并有对象层符号槽分派；符号类覆盖不完整，未知类仍停止回退。QGIS 符号转换是另一个范围。 |
| 359 | 部分 | FeatureLayer 的 SimpleRenderer 已在 `feature_layer_tail.rs` 解析，并由 `feature_layer.rs` 接入；“等”所含的其他 Renderer 体尚未闭合。 |
| 329 | 部分 | 新增 RasterLayer 对象体前缀读取器，覆盖版本和首条 String；后续体仍未解析，且尚未接到图层列表或 parse 模型。 |
| 361 | 部分 | 已建立 RasterLayer 头部读取前缀；渲染器字段仍在其后未解析区域，暂无栅格 Renderer 类/体读取器。 |

## 本轮代码与接口

此前对 Hyperlink 字段的临时改名已撤销，源文件与测试维持原有公开字段名和兼容注释。本轮只新增 RasterLayer 前缀读取器；共享 dispatcher 不在本轮修改范围。

## 验证与未验证项

- RasterLayer 五个单元测试通过，覆盖正常前缀/停止边界、负版本原生门行为、版本上限、逐前缀截断回滚及首字段字符串限额；版本上限用例先绑定字节数组再借用，避免临时值生命周期问题。该模块在测试时已由共享工作区现有 `mod.rs` 接线纳入编译，本轮未改该文件。
- `rustfmt --check src/layers/raster_layer.rs` 通过。仓库级 `cargo fmt --check` 未通过，报告了共享工作区其他文件（例如 `src/maps/mxd_maps.rs`、`tests/map_object.rs`）的格式差异；这些文件不在本轮修改范围。
- 全量 `cargo test` 中 RasterLayer 五项通过，后续在既有 `tests/map_surround.rs::class_id_constants_match_documented_clsids` 失败，左右 GUID 字节序不符；本轮未改该测试或对应对象。按用户要求配置本工作区 `TMPDIR`、`XDG_CACHE_HOME` 和 `JAVA_HOME` 后，定向 RasterLayer 测试再次通过。
- 未在 ArcMap 10.1 或 QGIS 中打开 MXD/QGZ；以上实档/运行类 TODO 不因本轮代码或合成测试而闭合。
