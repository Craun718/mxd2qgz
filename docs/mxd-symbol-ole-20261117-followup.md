# 符号、颜色与 OLE 续查（2026-11-17）

范围为本轮交接时 TODO 原行 153、208、210、215、242、260–261、267–268；不修改共享待办。结论区分既有静态证据、参考样本和合成测试。本轮复用已有打包证据和研究记录，未重新进行 REA 反编译。

## 修正 StdPicture 标识

`arcmap-10.1-ole-stdfont.md` 记录介质 oleaut32 2.40.4275 `0x65346d68` 的 StdPicture CLSID 为 `{0BE35204-8F91-11CE-9DE3-00AA004BB851}`。GUID 内存序应以 `04 52 e3 0b` 开始，而 `STDPICTURE_CLASS_ID` 曾误写为 `04 0b e3 0b`；两个现有 IPicture 槽以该常量分派，故正确 CLSID 的新对象会被当作未知类。已修正常量，并让 PictureMarker v9 类型 3 合成测试直接写入独立的固定 GUID 字节。此前测试引用同一常量，无法检出该错误。参考样本 StdPicture 为零命中，因此此修正没有真实样本或原生运行验证。

## 逐项状态

- 153 部分：`arcmap-10.1-dep-tail.md` 第五节记录 ISymbol 引用和未穷尽实现者；TextSymbol 已接入，仍需逐类核对其他模块、类身份及持久化体，不能因有限负搜索认定穷尽。
- 208 部分：`arcmap-10.1-display-symbols-round2.md` 和 `arcmap-10.1-display-tails.md` 给出布局及候选，尚缺类型库访问器到成员槽的关联或受控运行观察，不能将候选字段定名为事实。
- 210 静态读取即弃行为已闭合：`arcmap-10.1-display-symbols-round2.md` ArrowMarker `0x6eb3edde..0x6eb3edf0`、CharacterMarker 及 PictureMarker 分支记录读取后默认值覆盖。历史流值的业务含义及运行行为仍未知；是否将该单项按静态行为闭合由主代理决定。
- 215 部分：TextSymbol 字体槽与字段布局已有 `arcmap-10.1-element-bodies.md` 等静态记录；其余标量仍需类型库访问器或受控属性变更，缺完整语义映射。
- 242 部分：`arcmap-10.1-display-slots-round3.md` 记录 RandomColorRamp Load `0x6eb3c190` 的 int32 清零、PictureFill 的两个 double 被覆盖，及 MultiPart Load 版本≥2、Save docVer≥4 的门。MultiPart 的写侧版本表 `[1,1,1,1,2]` 与门在这些档位相互对应；不能据此推断未知历史版本作者意图。丢弃字段用途仍未知。
- 260 部分：`mxd-parse-agent-sample-evidence-elements-20261117.md` 记录首图 SimpleMarker/SimpleFill 对象级证据，MultiLayerMarker 与 CharacterMarker 仅有对象头候选；其余目标类须取得对应真实符号工程、逐嵌套对象走查并在 ArcMap 验收。
- 261 部分：同一记录确认首图 RgbColor 对象级证据，HSV/HLS/Gray/CMYK 缺对象级实例；需对应真实颜色工程及逐对象走查。
- 267 部分：`mxd-parse-agent-sample-evidence-edge-branches-20261117.md` 记录介质库头部和魔数分派；word2 业务语义、各解码器入口及真实图片对象尚缺。需含多种图片格式的 10.1 工程和受控 IPicture 保存/加载实验。
- 268 部分：同一记录给出 StdFont 样本 37 处与介质库布局对位；尚缺 ArcMap 实际运行 OS 的 oleaut32 与版本间对照或受控运行，不能证明跨版本稳定性。

## 可用性与验证

README 声明介质 ArcGIS Desktop 10.1.3035；此前 `mxd-parse-agent-sample-evidence-edge-branches-20261117.md` 记录提取的 x86 oleaut32 版本 2.40.4275，本轮复算 SHA-256 `baeb2f7c1b8be56738d34e1d1ddf8e0eebd3a633215dc1575e14656be38b939d` 一致。该 DLL 是安装介质分发件，不是已确认的目标 OS 运行库。参考样本存在；本机有 Wine 和 QGIS 启动命令，Wine 前缀搜索未发现 ArcMap.exe，syswow64 oleaut32 是 Wine DLL；未发现可用 ArcMap 10.1 或目标 Windows 运行库，因此未进行 ArcMap/QGIS 人工验收。未对 Wine 行为冒充 Windows 证据。

`cargo fmt --check` 和 `cargo test` 通过，含固定 GUID 的 PictureMarker 合成输入；全套有一个忽略的测试。Cargo 全局缓存自动清理因权限失败发出警告，不影响测试。合成测试验证分派和解析器契约，不是原生运行证据。

## 接线交接

IPicture 分派已分别引用修正后的共享常量（`display_symbol.rs` 与 `picture_fill_symbol.rs`），无需改模块总分派。请主代理将本记录的单项状态和 GUID 修正纳入共享 TODO/汇总；不要把 260/261 全族样本、267/268 跨库结论记作已验证。

## 安装介质原始模板补充（2026-11-17）

主代理已将 66 份介质模板和流导出至 `temp/arcmap101-media-mxd/`、`temp/arcmap101-media-streams/`（不纳入提交）。`index.json` 根据 `/Version` 报告 9.2/14 共 3 份、9.3/16 共 34 份、10.0/18 共 11 份、10.1/19 共 18 份，均有 `/Maps`、`/PageLayout`；这些是流头识别结果，不是运行验收。此前“缺真实符号工程”的说法应改为：现有原始模板覆盖了一部分符号，其他类和受控属性变化仍缺。

### Africa 的局部嵌套结构

`Africa.mxd` 文件 SHA-256 `ee6be21c409285644e81177a19878f5149cd2f3380b95f188fdfa5788d466674`；`/Version` 文本 10.1、short 19、对象数 637；`/Maps` SHA-256 `0d967ec22e59f6a8162c70df9614547cd636197fcbe3fa7d30b954c6e342401f`。现有 parse 自根部到图层条目，`0x220` 起的图层体保留未知；以下 `0x526` 为 GUID 搜索定位后，按 10.1 符号读取器带独立引用表进行的**局部试解析**，不是从根部连通的证明：

- `0x526..0x887`：MultiLayerMarkerSymbol，键 47，版本 3；内部 color 为键 48 的 RgbColor。内嵌 `0x5a3..0x655` 的 SimpleFillSymbol 键 49、版本 1，轮廓为 SimpleLineSymbol 键 50、颜色键 51，填充颜色键 52。
- 多层循环计数 2：`0x659..0x75a` 的 CharacterMarkerSymbol 键 53、版本 4，颜色键 54；其 IFont 槽的 StdFont 头位于 `0x728`、键 55。第二个字符标记 `0x75a` 键 56，颜色键 57，StdFont 头 `0x829`、键 58。两字体体均版本 1、charset 0、weight 400，字号低字分别 52500 和 67500，字体名字节为 `ESRI Default Marker`。多层对象尾部按读取器恰终止在 `0x887`。
- 另一独立候选 `0x110c..0x11bf` 可按 SimpleMarkerSymbol 键 85、版本 2 消费，内部 RgbColor 键 86、87。`0x11bf` 后内容归属未确认。

这些连续嵌套体、计数及下一项边界彼此吻合，提供对 260、261、268 的新增**局部真实字节佐证**；没有上层图层体和完整引用上下文，因此不声称这些局部候选已从根对象验证，也不能确定最后游标是宿主结构边界。RGB 中保留的三个 double 可出现负数等值，不据颜色类名猜测分量语义。对象内容与名称仅作为此模板的观察，不推广至全部实现类或保存版本。临时探针为 `temp/symbol_media_probe.rs`，依赖当前构建的库，不提交。

### 候选扫描的范围和限制

仅对这 66 份导出的 `/Maps` 与 `/PageLayout` 扫描固定 GUID：StdPicture 正确类标识 0 个命中；StdFont 1100 次、RgbColor 3581 次、TextSymbol 1047 次、MultiLayerMarker 44 次、CharacterMarker 58 次。重复对象引用和原始二进制碰撞均可能出现，数字仅供定位，不能当对象个数或样本覆盖证明；其他流未搜索。由于 `/Maps` 和 `/PageLayout` 都存在，本轮无需借 `Mx Document` 推断流归属。267 的真实图片对象仍未取得；268 的跨系统 oleaut32 对照仍缺。208、210、215、242 也没有因此取得字段语义或运行观察；153 的实现者穷尽问题不变。

`cargo test` 未因本轮只新增研究记录而重跑；局部探针运行成功，既不是目标软件运行，也不是新增合成测试。ArcMap 10.1 和 QGIS 人工验收仍未执行。主代理可将上述局部样本证据加入 260/261 的进度说明，尚不能勾选全族或 267/268。

## 10.8 安装介质模板的定向对照

`temp/arcmap108-media-streams/index.json` 记录 68 份模板，磁盘 `/Version` 识别为 10.1/19（26）、9.3/16（18）、9.2/14（12）、10.0/18（11）、8.1/10（1），没有 10.8；介质来源不能替代磁盘版本证据。

- 此介质 `africa.mxd.2319C344_B117_4BE0_8B98_93D6E4E94502` 与上节 10.1 介质 `Africa.mxd` 全文件 SHA-256 相同，`/Maps` 内容也逐字节相同，不算独立样本。
- `africa.mxt.2319C344_B117_4BE0_8B98_93D6E4E94502` 文件 SHA-256 `39180b9d95cf1ade7fad3d4314b5157b9a58cfeced98ee54decb877659e8e884`，`/Version` 是 9.3/16；`/Maps` SHA-256 `9568135816f004803db2c81a4f3d0af8e4e7d6b856cd583b7243924e89e43b2e`。`0x47e..0x743` 独立引用表局部试解析为 MultiLayerMarker 版本 2、键 45，内嵌 SimpleFill/Line、RGB 和两个 CharacterMarker 版本 2；两个字符标记的 StdFont 位于旧版字体槽，保留相同的字号低字 52500、67500。与上节 10.1/19 模板的 MultiLayerMarker v3、CharacterMarker v4 形成不同磁盘版本局部对照；上层对象尚未连通，不能由此证明整个工程解析正确。
- 定向搜索 StdPicture 时，`africa.mxt` 的 `/PageLayout` 在 `0x922` 有正确 CLSID，后接键 537；按 StdPicture 体读取器从 `0x936` 到 `0x37554` 消费，word2=0、图像长度 224278，数据以 `BM` 开始。BMP 文件头声明大小亦为 224278，尺寸 242×308、24 位；图像数据 SHA-256 `d4c91a5b74ed997c81bb9c0ef4c9b75819b25207e0ee9445fad5e9ffc89bf179`。`/PageLayout` SHA-256 `34cbc046e966da7ffe1cd1ceed3d51deb0a95568a9dff8a44b4639a184545ae8`。这是与结构、长度吻合的真实局部 StdPicture **候选**，宿主对象槽及读取器之后字段尚未从上层确认，不能当作完整流级解析或图片解码器运行证据；单例 word2=0 不足以确定其语义。该 BMP 候选推进 267，但不足以闭合。

临时探针 `temp/symbol_media_108_probe.rs` 使用现有读取器执行，未修改解析代码或重跑全套测试；上述均非 ArcMap/QGIS 人工验收。153、208、210、215、242 的原有结论不变，260/261 仍不能按全族闭合，268 仍缺实际 Windows 运行库跨版本比较。
