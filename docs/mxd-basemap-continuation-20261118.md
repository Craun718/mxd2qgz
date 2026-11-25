# Basemap 结果、逐图层遮罩与 10.1 候选跨流回引

本轮选用 ArcMap 10.1.0.3035 的 PE32 BasemapLayer.dll，SHA-256 `c8f4692970c5311e5d591f317c5e6c0379e9e6b9ad62a2383c71ecb6c8f64aad`，沿用[原生 Load/Save 对位](mxd-round2-basemap-20261117.md)及[结果容器取证](mxd-round3-basemap-20261117.md)。以下用既有反编译和 `objdump -d -Mintel` 复核实际指令；没有使用 10.8 规则补缺。

- MappingCoreLib.dll 的 PrepareResult Load `0x79b6fa20` 在版本 2 分支读普通对象列表。继续使用同一个键表，在已读完要素图层之后识别内嵌 Map；仅当其版本进入现有图层列表分支、列表为空、Map 尾部也完整闭合时才提交新键。图层列表非空时不递归进入 Basemap，尚不支持这一分支。
- BasemapLayer Load `0x6b646260` 在结果容器后按版本读 short、正式子图层计数和 ExtraSafe ILayer；`0x6b6464eb` 调用读对象，`0x6b646506` 通过集合虚表插入；版本 10 起读扩展、String 和两个 double，版本 11 再读 8 字节。Save `0x6b6465d0` 佐证子图层计数、逐项写出及尾部顺序。当前支持空槽和 A=20/B=0 的已登记子图层回引、空扩展集合；新增子图层体、非空扩展和其他 ExtraSafe 形式仍在原边界停止或明确回退。完整读取的 Basemap 必须与外层 ExtraSafe 边界一致。
- 通过 Basemap 的 QI `0x6b6434a3..0x6b6434d4` 比较 `0x6b6ac464` 中的 ILayerMasking IID，返回 `this+0x238`。同一 QI `0x6b64331b..0x6b64334c` 对 ICompositeLayer 返回 `this+0x21c`；虚表 `0x6b6ac190` 的计数及索引方法 `0x6b6436b0/0x6b6436f0` 枚举所选正式集合。Load 的集合槽 `+0x54` 进入 `0x6b64b0a0`，再到 `0x6b644000`；原生可包装被插入图层，包装类 QI 在 `0x6b64ad04` 走到 `0x6b69ebc0`，其默认路在 `0x6b69ecaa` 返回 E_NOINTERFACE，没有为 ILayerMasking 提供回退。结合 FeatureLayer 自身无该接口的[证据](mxd-luna-deep-layer-masking-20261118.md)，本轮只对 Map 版本至少 40、Basemap 体完整且正式集合仅由在结果中完整读出的 FeatureLayer 构成的情形，读取 Basemap 的遮罩段。其他复合类、未确认子图层及版本分支仍保守停止。
- 沿 Map.dll 原生已知调用点补齐三个未覆盖的槽形态：`0x7940005d..71` 的 ILayer 可读已登记 FeatureLayer 回引，`0x794000a7..bb` 的 IEnvelope 可读 Envelope 对象及其回引，`0x794005ed` 后的 IOverposterProperties 可读已登记 BasicOverposterProperties 回引。仍不解释未知实现类，也没有全局引用目标类型表。

Africa.mxd 从根 Map 键 38 连续解析：首个 PrepareResult 的 FeatureLayer 键 42 在 `0x145d` 结束，随后完整读取键 91 的内嵌 Map；六个要素键分别为 42、105、140、176、222、258，六条正式子图层都是回引。Basemap（键 40）的 v11 体完整消耗到 `/Maps:0x65fd`；图例版本 5 的三个条目读完。根 Map 在 `0xc395` 读取该 Basemap 的 v2 空遮罩集合，随后在 `0xc510` 闭合，紧接根流四字节尾部。切换到 `/PageLayout` 后读完 7 个元素；MapFrame 键 544 在 `0x59c` 回引根 Map 键 38。布局后续字节原样保留。

10.1 安装介质的 66 份已提取工程中，`/Version` 文本候选为 10.1 的 18 份有 10 份读完 `/Maps` 根并进入布局，其中 9 份读完整个元素列表。加此前 48 份，总计 58 份进入布局，57 份读完整个元素列表。CentralAmericaCaribbean 在 `/PageLayout:0x59a` 遇到 ParagraphTextElement（注册表 CLSID `{C84598B1-C4BE-4203-9132-ADA2BE57F30C}`）。CentralUSA、ConterminousUSA、NortheasternUSA、NorthwesternUSA、SouthernUSA、SouthwesternUSA、USA、USACounties 这 8 份仍停在 Basemap 内未闭合的 FeatureLayer，后续图例缺失相应回引。

验证：Africa 顺序引用及尾部、空结果/空子图层、全截断回退和版本越界测试。全量测试通过（887 项），包括本地介质的显式忽略用例；ArcMap 10.1 与 QGIS 人工打开及目视验收未执行。

### 准备结果中带图层的内嵌 Map

MappingCoreLib.dll 10.1.0.3035 的 PrepareResult Load
`0x79b6fa20` 通过普通 ObjectStream 读取 Map，新对象沿用其
自身 Load 的图层列表规则。Rust 复用正常 Map 读取器及已有的
图形嵌套深度限制，只有整张 Map 完成时才提交游标和对象表。
构造 Basemap 验证内嵌 Map v18 的旧列表含 GroupLayer 时，
两个新键均登记且可接续正式子层列表。带图层的真实宿主及
ArcMap/QGIS 人工验收仍待验证。
