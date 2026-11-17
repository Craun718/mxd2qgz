# 第二轮：符号与 OLE 局部对象验证

本记录的行号对应交接时的 TODO 原行号，不修改共享 TODO。读取了总复核和符号分报告；以 README 的 ArcGIS Desktop 10.1.3035 安装声明为目标。复算介质 x86 oleaut32 2.40.4275 SHA-256 为 `baeb2f7c1b8be56738d34e1d1ddf8e0eebd3a633215dc1575e14656be38b939d`，它不是已确认的目标系统运行库。以下没有新增原生反编译或目标软件运行。

## 本轮可复现推进

新增 `tests/round2_symbol_ole_media.rs`，显式运行 `cargo test --test round2_symbol_ole_media -- --ignored`：2 项通过，默认忽略，不要求安装介质参与默认测试。它将上轮临时探针变为受断言约束的回归检查，并新增实际图片字节的截断、长度限额和游标事务性验证：

- 10.1 介质 Africa `/Maps:0x526..0x887`：局部 MultiLayerMarkerSymbol 键 47、版本 3、两层均为 CharacterMarker；10.8 介质内磁盘版本 9.3 的 africa.mxt `/Maps:0x47e..0x743`：键 45、版本 2、两层同类。均以全新引用表试读且无停止原因；并未连通至宿主对象。这检验了两个不同版本的真实局部嵌套边界，不证明全部符号类或图层结构。
- 同一 9.3 模板 `/PageLayout:0x922` 核对 StdPicture CLSID 和键 537，体 `0x936..0x37554` 消费至精确末尾；第二字 0，数据长 224278，BMP 文件头长度一致。对头内和末尾共 9 个截断位置验证 UnexpectedEof、游标归零，限额 224277 验证 InvalidData、游标归零。样本来源和 SHA-256 见 `mxd-symbol-ole-20261117-followup.md` 的 10.8 介质段；流内宿主尚未定位，图片未解码或在 ArcMap 中运行。

本轮未修改生产解析器；上述测试不能确定字段语义或跨运行库兼容性。`cargo test` 通过；`cargo fmt --check` 因并行已有改动 `src/layers/composite_graphics.rs`、`src/version/version_stream.rs` 的格式差异失败，本轮测试文件已单独 rustfmt。未修改这些文件。未在 ArcMap 10.1 或 QGIS 人工验收。

## 原行逐项状态及最小余项

类别：STATIC 是可继续通过现有二进制、类型库及静态调用链研究；SAMPLE 指缺少特定文件字节；ARC 指需要目标软件或其实际运行库的运行观察。一个条目可有多类余项，不能把静态未做视作必须运行。

| 原行 | 状态与证据 | 最小余项 |
| --- | --- | --- |
| 155 | 部分：TextSymbol 已分派；ISymbol 实现族未穷尽，见 `arcmap-10.1-dep-tail.md`。 | STATIC：对未覆盖模块的 coclass、QI 与 IPersistStream Load 做身份和体定位。 |
| 210 | 部分：已有字段序和成员偏移，业务语义仍未全面定名。 | STATIC：将 10.1 类型库属性访问器与各类成员偏移逐个绑定；无法静态区分时 ARC：单属性变化和存取观察。 |
| 212 | 部分：`arcmap-10.1-display-symbols-round2.md` 的 ArrowMarker `0x6eb3edde..0x6eb3edf0` 等证实读取后默认值覆盖，历史值业务意义未闭合。 | STATIC：追踪默认函数及该成员的属性访问器；必要时 ARC：改变旋转相关属性后存取比对。 |
| 217 | 部分：TextSymbol Load `0x6ebd0a60` 及字体槽布局已有证据，标量未逐一定名。 | STATIC：查 10.1 属性 getter/setter 到成员槽的映射；不足时 ARC：单属性更改并保存。 |
| 244 | 部分：RandomColorRamp Load `0x6eb3c190`、PictureFill 两 double 覆盖及 MultiPart 版本门见 `arcmap-10.1-display-slots-round3.md`；MultiPart 写表和门在已观察档位对应，不足以解释历史意图。 | STATIC：追踪相关槽写入及调用者；无法静态定性的丢弃值需 ARC：控制属性与加载前后值。 |
| 263 | 部分：本轮两版本多层标记及两个字符标记局部测试通过，不能外推全族。 | STATIC：对介质其余候选进行宿主链与嵌套结构定位；SAMPLE：对仍无实例的类，在 ArcMap 10.1 创建每类符号并保存独立工程；ARC：打开工程检查显示。 |
| 264 | 部分：上述两个多层标记含 RGB，之前局部解析与真实边界已记录；颜色族未全覆盖。 | STATIC：继续搜索并解析现有介质候选；SAMPLE：对未覆盖的 HSV/HLS/Gray/CMYK 分别用 10.1 创建受控颜色工程；ARC：检查实际显示。 |
| 270 | 部分：真实 BMP 局部对象及边界、截断测试成立；仅有第二字为 0 的单例。2.40.4275 读头 `0x653a0191`、写头 `0x653a1071` 及分派见 `arcmap-10.1-ole-stdfont.md`，未定第二字语义。 | STATIC：追踪完整读头循环、各魔数分支及调用者；SAMPLE：准备含 JPEG/GIF/图元文件的持久化对象，可用 ArcMap 10.1 或对应 Windows OLE API 受控保存；ARC：验证实际加载分派和显示。 |
| 271 | 部分：既有参考样本及介质 2.40 字段一致，不构成跨版本保证。 | SAMPLE：取得目标 Windows 版本的真实 oleaut32；STATIC：核对身份并比较 Load/Save；ARC：在各目标 Windows 运行库上用相同字体属性保存与互读，ArcMap 端检查显示。 |
