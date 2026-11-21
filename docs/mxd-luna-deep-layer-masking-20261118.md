# LayerMasking 聚合 QI 与 Map 门控（2026-11-18）

本记录以 ArcMap Desktop 10.1.0.3035 x86 静态实现为目标；地址均为 PE 静态 VA。未启动 ArcMap/QGIS，结论不代表运行中 COM 注册状态或插件替换类。

## 身份与证据来源

| 模块 | 固定版本 / SHA-256 | 本轮关键入口 |
| --- | --- | --- |
| `Map.dll` | 10.1.0.3035 / `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` | `0x79403d50`、`0x7941a5b0`、`0x7941a620` |
| `AnnoLayer.dll` | 10.1.0.3035 / `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5` | FDO 构造 `0x6b18db50`、聚合创建 `0x6b18df98`、未知 IID 转发块 `0x6b18ef03..18` |
| `FeatureLayer.dll` | 10.1.0.3035 / `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` | 主 QI `0x6f6e7a40`、基类 QI `0x6f73c440`、通用基类查询 `0x6f73c0d0` |
| `AfCore.dll` | 10.1.0.3035 / `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` | `AfCreateInstance 0x68a77660`、激活分派 `0x68a777c0`、工厂缓存 `0x68a77b70` |

反编译优先复用 `/home/gxrs/文档/source/arcmap10.1/` 地址索引成果（AfCore、FeatureLayer）；AnnoLayer 使用仓库已有 10.1 研究成果。关键分支以本轮 `objdump -d -Mintel` 对原始 PE 复核，临时反汇编位于忽略目录 `temp/terra-round10-mask/`。摘要与版本资源只确认字节身份/版本声明，不证明签名或运行行为。

## FDOGraphicsLayer 的聚合对象身份

AnnoLayer 构造函数 `0x6b18db50` 在 `0x6b18df7c..df9d` 将外对象 `this+0x60` 作为 outer、聚合标志 `1`、请求 IID `{00000000-0000-0000-C000-000000000046}`（IUnknown），把输出写入 `this+0xb8`。类标识地址 `0x6b1ac030` 为 FeatureLayer coclass `{E663A651-8AAD-11D0-BEC7-00805F7C4268}`。随后构造代码直接从 `+0xb8` 的虚表调用 QI，取得 IPersistStream 到 `+0xbc`、ILayer 到 `+0xc4`、IFeatureLayer 到 `+0xc8`，与 AfCore `AfCreateInstance` 的参数序吻合。

AfCore `0x68a77660` 调 `0x68a777c0`；后者走 `CoCreateInstance` 或 `FactoryCache::CreateInstance`。`0x68a77b70` 的缓存缺失路径用 `CoGetClassObject` 取得工厂，再调用其 `+0x0c` 槽。FeatureLayer `DllGetClassObject 0x6f717160 → 0x6f73dd90` 按类表查 CLSID；初始化函数 `0x6f74ba70` 将 FeatureLayer CLSID 指针放在表项 `0x6f7d1338`，并在 `0x6f7d1340` 建立 `ClassFactory<FeatureLayer>`。该工厂 vtable `0x6f7b0294` 的 `+0x0c` 指向 `CreateInstance 0x6f719190`。

**返回指针到 QI 函数的指令级绑定：**`0x6f719190` 的聚合分支先以 `0x6f74eb7c` 比较请求 IID（该常量是 IID_IUnknown）；仅相等才继续。指令 `0x6f7191d7..1f4` 分配对象并调用 FeatureLayer 构造函数 `0x6f6db5e0`；`0x6f719216..21b` 将构造函数返回的 EAX 原样写入调用者 `ppv`。构造函数中 `0x6f6db953..958` 对对象首字写入主 vtable `0x6f7ace14`；该 vtable 首项在原始 `.rdata` 是 `0x6f6e7a40`。因此 AfCore 收到并由 AnnoLayer 存在 `+0xb8` 的 IID_IUnknown 指针，其 `QueryInterface` 槽确切绑定到 `0x6f6e7a40`，不是由 RTTI 猜测，也不依赖抽象的聚合约定推导。

`0x6f6e7a40` 是该返回对象的本地接口分派：未命中后进入 `0x6f73c440 → 0x6f73c0d0` 的 FeatureLayer 基类查询；这条链没有转调构造时保存的 outer QI。主 vtable 与委派子接口可由指令区分：构造时另设的 `+0xd0` vtable `0x6f7acfb0` 的 QI 首项是 thunk `0x6f709aa0`，它继续跳往 `0x6f6d8210` 的控制未知接口路径。故本次 `+0xb8` 指针走的是本地主 vtable QI `0x6f6e7a40`，而不是该子接口委派 thunk。

## ILayerMasking 与第二个门控 IID

- Map `0x79403d50` 遍历 `this+0x150` 数组、计数 `this+0x154`。每个非空条目先调用 `0x7941a5b0` 查询 `{F9D6AA98-5586-486F-8242-670AEDF7860E}`（IBasemapLayer）。原始指令 `0x79403de5..f6` 只有在该 QI 成功且 Map 的有符号 short 版本 `<40` 时跳过本图层；其余路径调用 `0x7941a620` 查询 `{536427DA-5354-491D-AE92-ABEBAA8BB05B}`（ILayerMasking），再把真实输出指针传给 `0x79403bc0`。因此这第二个 QI 是版本门控，Map 版本 `>=40` 时 IBasemapLayer 成败均不改变后续 ILayerMasking 探测。
- AnnoLayer FDO QI 对本类接口表未命中的请求，在 `0x6b18ef03..18` 转发到 `+0xb8` 的 QI。对照 FeatureLayer `0x6f6e7a40`：其主比较表没有 ILayerMasking 命中，末段转入 `0x6f73c440`；该函数仅处理自身基接口后调用 `0x6f73c0d0`，后者逐项查询其基础接口并以 `0x80004002` 结束未命中。目标 IID 因而在这个非委派查询路径返回不支持。FeatureLayer PE 的 `.rdata` 另有一次 ILayerMasking GUID 字节出现，但它不在这两段查询分派的比较路径中，不能单凭字节存在判为支持。
- 这闭合的是本机 FeatureLayer 实现经 FDO 聚合时 `ILayerMasking` 的静态负结果，也闭合 FDO 外层未知 IID 的转发接收者。不能推广为任意注册替代类的运行时 QI 结果。

同组已核实的类级能力表：GroupLayer `0x6f710211..240` 直接命中 ILayerMasking 并返回 `+0x70`；FeatureLayer 主/基类查询链不支持；RasterLayer 主/基类查询链以 `E_NOINTERFACE` 结束；FDOGraphicsSublayer 主/基类查询链不支持；FDOGraphicsLayer 对该 IID 转发到聚合 FeatureLayer，结果不支持。其他类保持未决。相应先前静态细节见 [第九轮](mxd-terra-round9-mask.md) 与 [第十轮](mxd-terra-round10-mask.md)。

### 可安全沿流继续的对象身份条件

- 对 FDOGraphicsLayer，只有在 Map 列表的新对象类标识互证为 `{34B2EF83-F4AC-11D1-A245-080009B6F22B}`、本机激活目标确为上述 10.1 AnnoLayer/FeatureLayer 二进制且内层 CLSID `{E663A651-8AAD-11D0-BEC7-00805F7C4268}` 由其 FeatureLayer 类工厂创建时，才可据本链将 ILayerMasking 判为不支持。此时 Map 若未被 `<40 + IBasemapLayer 成功` 门提前跳过，`0x79403bc0` 收到空接口指针而不消费遮罩流字节，能继续到下一逐图层项。Binary/ClassID 的吻合不覆盖注册替换或插件代理对象。
- 对 FeatureLayer、RasterLayer、FDOGraphicsSublayer 与 GroupLayer，同样要求具体对象类标识、10.1 实现身份与实际查询对象相符；回引必须在引用表中有已登记键，不能只凭回引头内重复的类 GUID 推断运行时对象身份。前四个已证类（GroupLayer 除外）查询失败时不读该层遮罩；GroupLayer 查询成功，才进入其遮罩集合。
- 对任意未知类，能力表给 `Unresolved`，不得猜测读或跳过。Map 版本 `<40` 时也只有在该实例的 IBasemapLayer QI 已证成功后才能采用早期跳过分支；Map 版本 `>=40` 时该 QI 结果不影响后续 ILayerMasking 查询。
- 即使已确认进入 LayerMasking 集合，内部新对象仍没有可由对象头推导的体长；当前读取器遇到未支持的新对象会事务性失败并回退，不能越过其未知体继续下一项。只有条目/目标均为空、已知回引，或新对象体由闭合的类读取器完整消费时，才可推进游标。

## 实现、测试与边界

`src/maps/layer_masking.rs` 新增两项明确的静态知识：`known_layer_masking_support_10_1` 仅对以上五类返回已知支持/不支持，对未知类返回 `Unresolved`；`layer_masking_probe_10_1` 按有符号 Map 版本和 IBasemapLayer 查询结果表达 `<40` 跳过门。现有读取入口仍要求调用方提供 ILayerMasking 是否存在；本轮不在 `map_object.rs` 猜测或接线运行时 QI。

新增 `tests/luna_layer_masking_20261118.rs` 覆盖版本 39/40 门界、Basemap QI 成败、负 short 位模式、已核实的类能力及未知类保守返回。该专属测试 2 项通过，两个改动 Rust 文件的 `rustfmt --check` 通过。全量 `cargo test --offline` 有一项无关失败：`tests/composite_graphics.rs::complete_sample_context_reads_cgl_sublayers_without_skipping` 在第 973 行得到游标 152508、断言 152510；全仓 `cargo fmt --check` 还报告未改的 `tests/luna_map_tail_20261118.rs` 格式差异。未在 ArcMap/QGIS 中验证；测试只验证编码后的静态决策表，不证明运行时环境行为。

未重复分析 Map `0x79403d50` 伪代码中的 NULL 参数误判；本轮引用的是 Map 原始指令里实际传递的 QI 输出。未运行 REA CLI：现有地址导出加原始指令足以闭合本轮静态分派；若要再追注册工厂运行期替换或实际 COM 激活，应由主代理对 AnnoLayer `0x6b18df98`、`0x6b18ef03`，FeatureLayer `0x6f6e7a40` / `0x6f73c440` 运行 REA，并核验目标机器类注册。未输出 MXD 样本敏感属性。

## GroupLayer 子层遍历

沿用上表中的 Map.dll 和 FeatureLayer.dll 10.1.0.3035。对照地址索引中的 `79403bc0.c`、`6f711270.c`、`6f710540.c`、`6f710570.c`、`6f7105c0.c` 和 `6f701c60.c`，用两份原始 PE 的 `objdump -d -Mintel` 补齐反编译丢失的循环和参数。以下地址均为静态 VA。

| 模块及地址 | 观察 |
| --- | --- |
| Map.dll `0x79403bf9`、`0x79403c0e`、`0x79403c27`、`0x79403c6d` | 先读当前层遮罩，再查询 ICompositeLayer，从索引 0 开始枚举子层。 |
| Map.dll `0x79403c95`、`0x79403cad` | 对子层查询 ILayerMasking 后直接递归进入 `0x79403bc0`。递归路径没有 `0x79403d50` 的旧版顶层 Basemap 门。 |
| FeatureLayer.dll `0x6f70fe97..c0`、`0x6f710540`、`0x6f710570` | GroupLayer 的 ICompositeLayer 视图为 `this+0x30`；计数和索引读取对应同一子层数组。 |
| FeatureLayer.dll `0x6f7113e9..3fa`、`0x6f7105c0`、`0x6f701c60`、`0x6f7119b0` | Load 按流顺序添加成功读取的子层。Add 检查空指针及自引用或祖先循环，随后以索引 −1 追加；正常追加路径没有同级去重。Save 按索引递增写出该数组。 |

Rust 按上述顺序消费已完整识别的 GroupLayer 自身和子层遮罩，空槽无记录。回引须匹配先前确认的对象键和类，重复回引按各自位置再走一遍子树。畸形回引受递归深度上限保护。遇到未确认遮罩接口的子层、尚不能处理的子层体或遮罩截断，整个逐图层循环回退到循环起点。嵌套 BasemapLayer 可以在 Map v28–v39 中产生遮罩记录；其正式子层集合仍限于 PrepareResult 中完整确认的 FeatureLayer 回引。图层树中未识别的图层类接口、不能完整读取的新遮罩对象体及 ArcMap/QGIS 人工验收仍开放。

Map v28 的构造测试覆盖普通组图层子层项、嵌套 Basemap、重复子层回引、顶层回引、错类回引和截断。该测试验证对象停止点和记录顺序，尚缺原生生成的嵌套遮罩实档。

GroupLayer v11–v12 的 ExtraSafe 子层沿用已确认的外层边界。DimensionLayer、TopologyLayer、NetworkLayer 和 SchematicLayer 的类级 ILayerMasking 分派已确认不消费遮罩，因此完整组读取器可保留这些子层的未知体，并在遍历时登记对象键供回引使用。构造的 Map v28 覆盖 NetworkLayer 新子层、同组回引、顶层回引和未知类退回。子层体字段和原生嵌套实档尚未验证。

Map 诊断记录逐层循环失败的首个原因。有对象身份时，输出图层键及尚未确认的类，或输出该图层遮罩记录的底层读取错误。构造测试验证错类回引与遮罩截断各有独立诊断，并仍回退到同一循环起点。此项支持定位尚未解析的类和字段，原生绘制效果仍待人工验收。

## 遮罩集合中的新对象

LayerLib.dll 10.1.0.3035（SHA-256 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）的 `LayerMasking::LoadLayerMasking 0x786f1ff0` 已有 Ghidra 导出。复核原始指令及两处 IID 的 PE 字节后，`0x786f21b8` 将 `0x7871fc68` 传给外层 LoadObject，即 `{34C20002-4D3C-11D0-92D8-00805F7C28B0}`。同版本 `ESRI.ArcGIS.Carto.dll` 的 GuidAttribute 将该 IID 定名为 ILayer。`0x786f221c` 将 `0x7871e78c` 传给内层 LoadObject，即 `{00000000-0000-0000-C000-000000000046}`（IUnknown）。两者均走普通对象槽，头部没有外层体长。

两处新对象都可交给既有完整图层体分派器。对象键与嵌套对象键随遮罩集合一起提交。未知类或截断时集合回退，内层非图层的 IUnknown 对象仍需各类持久化证据。结果保留对象头、层级字符串、遮罩尾字段及两类槽中新出现的完整图层体。新图层体按存盘顺序与对象键关联，回引沿用原对象键。构造测试令外层和内层分别引入新图层，再回引先前内层对象，并逐字节截断检查游标和登记表回退。

## 回归结果

`cargo fmt --check`、`git diff --check` 通过。全量测试含忽略项共 1035 项通过。CLI 对仓库样例和 66 份 10.1 介质工程均读完 `/Maps` 与 `/PageLayout`。按实际对象键回查，10.1 介质的 18 份工程共读出 22 条 BasemapLayer 遮罩记录，集合条目数均为 0。68 份 10.8 介质补充对照中，60 份读到地图根尾，41 份读完布局。组层递归和集合内新对象由构造测试覆盖。ArcMap 和 QGIS 人工验收未做。

### 被遮罩图层、遮罩图层及层级字符串

10.1.0.3035 LayerLib.dll 的 ILayerMasking 虚表 `0x78722ac0` 将 `+0x20`、`+0x18`、`+0x14` 分别绑定到导出的 `get_MaskedLayers 0x786f1510`、`get_MaskingLayers 0x786f13c0`、`put_MaskingLayers 0x786f1100`。Save 从 `get_MaskedLayers` 返回的集合逐项取出外层对象，在 `0x786f1ba1..0x786f1bb2` 用它取得内层遮罩集合。Load 在 `0x786f231a..0x786f232b` 将外层 ILayer 和读出的内层 ISet 传给 `put_MaskingLayers`。

ILevelMasking 虚表 `0x78722aec` 的 `+0x10` 为 `get_LevelMasking 0x786f1690`。Save 在 `0x786f1ce1..0x786f1cfb` 传入外层与内层两个对象，随后用 `String::Write` 写出返回的层级字符串。解析结果保留空字符串与旧版无字段的区别，提供按原生语义命名的槽访问器。图层之间的实际绘制效果尚未做 ArcMap/QGIS 人工验收。

### LevelMasking 层级列表编码

同一 LayerLib.dll 导出的 `LayerMasking::ContainsLevel 0x786f38e0` 将字符串两端的空白去掉，`0x786f3932..0x786f395e` 对空串返回真。`0x786f395f..0x786f3963` 调用 AfCore String::Replace 将分号替换成逗号，`0x786f3986..0x786f39f9` 按逗号取非空项。`0x786f39ff..0x786f3a1d` 对无连字符的项比较单个层级，`0x786f3a25..0x786f3a81` 将连字符两侧转为整数，按闭区间匹配。AfCore 已有的 `String::AsLong 0x68ab5950` 伪代码表明数值转换下放给 `_wtol`。

解析器可解码常规十进制层级和闭区间，原始字符串始终保留。非规范数字、溢出等输入仍可作为原文读取，其 CRT 容错行为尚未做运行验证。构造测试覆盖逗号、分号、空串、单层级、闭区间和无法解码的项。

### 遮罩记录的版本门

复核同一 LayerLib.dll 的原始 x86 指令，`0x786f2041..0x786f204d` 用有符号 `jle` 判断版本上限 2，`0x786f224b..0x786f2255` 用有符号 `jle` 跳过目标名称。Map 自身和逐图层遮罩共用此原生入口。Rust 保留原始 short 位模式，允许负值沿旧版本分支解析，并拒绝正值 3 及以上。构造测试覆盖负值、目标名称边界和逐字节截断，参考工程改写版本测试覆盖 Map 宿主。尚未发现原生写出的负版本记录。

### 遮罩集合的计数分派

同一 LayerLib.dll 中，`0x786f2125..0x786f2130` 和 `0x786f21ea..0x786f21f5` 分别读取四字节外层、内层计数。`0x786f219d..0x786f21a3` 用有符号 `jle` 跳出外层循环，随后在 `0x786f23a4..0x786f23ab` 读取尾 short。`0x786f2208..0x786f220e` 用同一条件跳过内层循环，外层仍从 `0x786f2359..0x786f2363` 继续迭代。Rust 按该边界处理非正计数，Map 诊断保留外层原始计数，正计数仍受资源上限约束。覆写参考工程验证 Map 解析终点，构造记录验证内层跳过后继续读取下一外层项。原生写出的负计数记录与人工验收仍待验证。

### 遮罩记录的 UseMasking

同版 LayerLib.dll 导出的 ILayerMasking 虚表 `0x78722ac0`，槽 `+0x0c`、`+0x10` 分别指向导出成员 `put_UseMasking 0x78698280`、`get_UseMasking 0x78698250`，二者直接读写同一两字节成员。Save `0x786f1f6f..0x786f1f89` 调用 getter 并写两字节；Load `0x786f239d..0x786f23ab` 读两字节，`0x786f2557..0x786f2564` 调用 setter。Map 和逐层记录的最后一项因此均可定名为 UseMasking。诊断保留原始 VARIANT_BOOL 位模式；构造记录覆盖非规范值。未在 ArcMap 中验证绘制行为。

## DimensionLayer、TopologyLayer 和 NetworkLayer

以 10.1.0.3035 Desktop 安装介质中的 PE32 文件为准。先用同版本 `esri.clsid.xml` 将 SDK 工程顶层有界槽中的 CLSID 对位到类名，再用 PE RTTI 的完整对象定位符绑定主虚表 QueryInterface 槽。复核过程中使用 FeatureLayer 的现有 Ghidra 导出，并用 `objdump -d -Mintel` 补查 Dimension 与 NetworkLayer 的指令。

| 类与 CLSID | 模块与 SHA-256 | QI 路径 |
|---|---|---|
| DimensionLayer `F1E27E32-0CA7-11D4-80D7-00C04F601565` | `Dimension.dll` `1a8a46c98c74ddf442481397cda68806b7a488fb12a3f68d806e5afc7e6eaecf` | 主虚表 `0x6e9f4c3c` 的 QI 为 `0x6e9e97d0`。外层在 `0x6e9e9adf..aec` 将未命中的 ILayerMasking 请求转发给 `this+0x54`。构造代码 `0x6e9e956b..9587` 以聚合标志 1 和 FeatureLayer CLSID `0x6e9f4fdc` 创建该成员，请求 IUnknown。其内部 QI 链按上文在 FeatureLayer 基类查询末尾返回 `E_NOINTERFACE`。 |
| TopologyLayer `DE98BAD5-135E-4B36-9653-57B5513FB7B2` | `FeatureLayer.dll` `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` | 主虚表 `0x6f7b1518` 的 QI 为 `0x6f7287a0`。Ghidra 导出内的 IID 比较均未包含 ILayerMasking，末尾直接调用 `0x6f73c0d0`，未命中在 `0x6f73c1ba` 路径返回 `E_NOINTERFACE`。同 DLL 内的遮罩 IID 引用 `0x6f710212` 属于 GroupLayer。 |
| NetworkLayer `D4F8E94B-5CF5-4F8F-8B4D-5B25AE4C0AF9` | `NetworkLayer.dll` `9a1f25263eca92124884deec893a44a7b51ca52f8a2d491c7fbdfad51b1af337` | 主虚表 `0x7b2a9a68` 的 QI 为 `0x7b28cc70`。未命中在 `0x7b28d24c` 调到 `0x7b29f850`，再经 `0x7b29f8a4` 调到 `0x7b29f080`，三个分派均只比较各自固定 IID，最终 `0x7b29f16c..172` 清空输出并返回 `E_NOINTERFACE`。 |

Map 读取器仅在图层列表已经为这些类确定顶层对象体范围后登记图层键，随后继续逐层遮罩遍历。仍保留图层原始体及其边界依据。SDK .NET 的 Montgomery 和 MontgomerySimple（Map v44）及 SDK Java 的 portland（Map v45）都复用原有 ExtraSafe `A - 20` 边界推定。三个工程此前停于阶段 38，现分别在 `0x1d7c4`、`0x9484`、`0x77373` 结束 Map 对象，随后均恰好剩下四字节 Maps 流尾值。构造用例验证顶层 NetworkLayer 和同键回引均不消费遮罩，并验证未知 CLSID 仍停在阶段 38。

两份 mole_data 中有界跳过的第三方子图层暂未找到同版本类实现。ArcMap 与 QGIS 的人工验收未执行。

## TemporalFeatureLayer 的聚合查询

10.1.0.3035 Desktop 的 `TrackingLayer.dll`（PE32，SHA-256 `06b59b5930c643be5244b20a8cc5f7576a16b9d6e5421ee327839e8230a169ab`）与同版本 `esri.clsid.xml` 均将 `3334CE40-1A1D-11D5-B7CF-00010265ADC5` 绑定到 TemporalFeatureLayer。DLL 中 `0x8086eb90` 的 IPersistStream::GetClassID 也返回该 CLSID。用 MSVC RTTI 定位 `CComObject<CoTemporalFeatureLayer>` 主虚表 `0x808a117c`，然后用 `objdump -d -Mintel` 复核：

- 主 QI `0x80884540` 调用 `0x808693f0` 扫描表 `0x808a0340`。表中没有 ILayerMasking 或 IBasemapLayer 的直接项。通配条目 `0x808a05c8` 调用 `0x808886e0`，将原 IID 和输出指针转交给 `this+0x2468` 保存的对象。此成员为空时返回 `E_NOINTERFACE`。外层聚合类的未知 IID 分支 `0x80884690..0x8088469d` 也进入同一张表。
- `0x80869ee7..0x80869f12` 调用 `CoCreateInstance`，以当前对象为 outer，使用上下文 3，请求 IUnknown，将返回值写入该成员。CLSID 常量 `0x8089f230` 是 FeatureLayer `E663A651-8AAD-11D0-BEC7-00805F7C4268`。同版本 FeatureLayer 工厂返回的非委派 QI 已在上文绑定到 `0x6f6e7a40`，该查询链对 ILayerMasking 无命中。
- DLL 内对 ILayerMasking 常量 `0x8089ece4` 的两处直接交叉引用 `0x808646b3` 和 `0x80864e58` 都查询 `this+0xec` 指向的另一个对象，与 Map 对 TemporalFeatureLayer 的 QI 分派属于不同调用路径。

读取器保留有界 TemporalFeatureLayer 的原始体，并允许该层及其回引通过逐层遮罩阶段。构造 Map v28 验证后续对象边界和未知类回退。原生 TemporalFeatureLayer 对象体、注册替代类与原生工程人工验收仍待验证。

## CompositeGraphicsLayer 的接口分派

复用 [图形容器取证](arcmap-10.1-dep-tail.md) 确认的 CLSID `9646BB83-9512-11D2-A2F6-080009B6F22B`、主 QI `0x6f6b0770` 及同一 10.1.0.3035 FeatureLayer.dll。用 `objdump -d -Mintel` 复核主 QI 的 21 个固定 IID，再沿 `0x6f6b0b0b → 0x6f73c440 → 0x6f73c0d0` 核对基类的四个固定 IID。各比较均不命中 ILayerMasking，末尾 `0x6f73c1bc..0x6f73c1c2` 清空输出并返回 `E_NOINTERFACE`。构造 Map v28 验证完整图层及其回引后可达流尾，内部未知对象仍在逐层遮罩阶段保留停点。未验证原生 MXD 中的对应图层宿主及 ArcMap/QGIS 人工验收。

## GraphicsLayer3D 的接口分派

复用 [三维图形容器取证](arcmap-10.1-dep-tail.md) 中的 GraphicsLayer3D CLSID `4C87A8F0-01E3-11D4-AB69-0008C73FD50C` 与主 QI `0x7d2a15b0`。复核 10.1.0.3035 Desktop 的 Scene.dll（PE32，SHA-256 `b41f37424a13b0257ce3c5c3258038a3640876aba48b33e05b332f7af3594006`），用 `objdump -d -Mintel` 追踪 QI 并读取比较常量指向的 16 字节 IID。主 QI 的 17 个固定候选均不含 ILayerMasking；未命中时 `0x7d2a18ce` 进入 `0x7d2c1950`，后者仅增加一个固定候选并在 `0x7d2c19a4` 转入 `0x7d2c2c50`。最终三个固定候选也不匹配遮罩接口，`0x7d2c2d3c..0x7d2c2d42` 清空输出并返回 `E_NOINTERFACE`。这些地址为静态 VA。

Map 现在可越过完整读取的 GraphicsLayer3D 及其回引。构造 Map v28 验证流尾位置；原生 MXD 中的对应宿主及 ArcMap/QGIS 人工验收仍待验证。

## GlobeGraphicsLayer 的接口分派

沿用三维图形容器取证确认的 CLSID `A415A073-CF06-4CDC-8664-439D0D5E7169`。复核 10.1.0.3035 Desktop 的 GlobeLayers.dll（PE32，SHA-256 `79346fc1fa0d25c01ac41ae2a7b0ee3014847c38f1661ff7d23cce75fe6fd1b8`）。用 `objdump -d -Mintel` 核对主 QI `0x72f51ad0` 的 21 个固定 IID，再读取未命中路径 `0x72f51e7e → 0x72f6c480` 和 `0x72f6c4d4 → 0x72f6d780` 的其余四个比较常量。所有常量均与 ILayerMasking 不同，`0x72f6d86c..0x72f6d872` 清空输出并返回 `E_NOINTERFACE`。构造 Map v28 验证完整图层及其回引之后的流尾。原生 MXD 宿主及 ArcMap/QGIS 人工验收仍待验证。

## ImageServerLayer 的接口分派

10.1.0.3035 Desktop 的 `ImageServerLayer.dll`（PE32，SHA-256 `0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`）与同版本 `esri.clsid.xml` 提供类身份 `477D13E7-8D68-45B6-A7FD-2EF442BCCE95`。用 MSVC RTTI 的完整对象定位符找到 ImageServerLayer 主虚表 `0x7802eb64`，首项为 QI `0x77fe8380`。用 `objdump -d -Mintel` 提取该函数对 `0x77fe3df0` 的 56 次比较及对应 IID 常量，包含 `0x77fe89b2..0x77fe8c22` 条件分支中的候选。未命中经 `0x77fe8d55 → 0x78023390` 和 `0x780233e4 → 0x78022ff0` 比较另外四个固定 IID，均无 ILayerMasking 或 IBasemapLayer 命中，`0x780230e2` 返回 `E_NOINTERFACE`。

构造 Map v28 验证完整 ImageServerLayer v8 和回引之后的流尾，以及遇到未知 IName 后停于逐层遮罩入口。原生服务图层实档及 ArcMap/QGIS 人工验收仍待验证。

## WMSLayer 的接口分派

10.1.0.3035 Desktop 的 WMSLayer.dll（PE32，SHA-256 `d204a1efc758e6b721c356976f3be4384a8f23ce1ef085f3cccc4c4f53630da6`）在同版本类注册表中对应 `5B0DA8F6-5E43-40AE-9871-56BA33936F30`。MSVC RTTI 的零偏移完整对象定位符绑定 WMSLayer 主虚表 `0x80d8d0bc`，首项为 `0x80d6e000`。用 objdump 核对主 QI 的 21 个 IID 比较及未命中路径 `0x80d6e315 → 0x80d847b0`、`0x80d84804 → 0x80d83720` 中的另外四个固定候选。ILayerMasking 和 IBasemapLayer 均无命中，`0x80d83812` 返回 `E_NOINTERFACE`。

构造 Map v28 验证已确定边界的 WMSLayer 及其回引可通过逐图层遮罩，未知类仍在该阶段停止。66 份已提取的 10.1 介质 Maps 流中均未命中 WMSLayer CLSID 字节。WMSLayer 持久化体与 ArcMap/QGIS 人工验收仍待验证。

### WMS 子图层

同一 10.1.0.3035 模块及注册表将 WMSGroupLayer 绑定到 `F677BA62-7CA7-400A-9C59-62930A282CEB`，WMSMapLayer 绑定到 `E38A56C0-D5BD-4899-B089-C8ED4E38B77F`。零偏移 RTTI 定位符分别指向主虚表 `0x80d8c934` 和 `0x80d8da74`。用 objdump 复核首项 `0x80d66660` 和 `0x80d77830` 的 21 和 40 个固定 IID 比较，包含条件分支。两种遮罩探测 IID 均无命中，分别在 `0x80d669d7` 和 `0x80d77e94` 调用共用后备 `0x80d847b0`，沿上节证据返回 `E_NOINTERFACE`。构造 Map v28 验证两类有界图层及回引。原生持久化体和目标软件人工验收仍未验证。

## NALayer 的接口分派

10.1.0.3035 Desktop 的 NetworkAnalyst.dll（PE32，SHA-256 `8c4ebaee71aad63c25616aa806674018479829d97bc01c7ed3b8845813698532`）与同版本注册表将 NALayer 绑定到 `5FBD8571-A560-494F-9FAF-3EDCED5A7890`。MSVC RTTI 的零偏移定位符 `0x7afab62c` 将主虚表 `0x7af9f0cc` 绑定到 NALayer，首项为 `0x7af195c0`。用 objdump 复核该入口的 35 个固定 IID 以及 `0x7af19b1e → 0x7af82c80`、`0x7af82cd4 → 0x7af82280` 的四个后备 IID，均不匹配 ILayerMasking 或 IBasemapLayer。最终 `0x7af8236c..0x7af82372` 清空结果并返回 `E_NOINTERFACE`。

重跑 10.1 SDK .NET 和 Java 两份 SanFrancisco 工程，逐层遮罩解析越过六个有界 NALayer，首个未确认项从键 31 推进到键 4501 的 GroupLayer。构造 Map v28 覆盖 NALayer 有界体及回引，并验证未知类仍停止。NALayer 体字段、后续 GroupLayer 子树及 ArcMap/QGIS 人工验收仍待验证。

## ProcessLayer 的组图层委派及遮罩

用 `objdump -d -Mintel` 及 PE32 虚表复核 10.1.0.3035 的 `GpObjects.dll`，SHA-256 `c9da22687b40e070d7178538846f707c04d26474d2fa835acb01ceaa46d2f896`。同版本 `esri.clsid.xml`、原生 GetClassID `0x7477a220` 和 SDK 工程的顶层对象头均给出 ProcessLayer CLSID `D6E92E16-E656-4119-B8E1-88576F6833E6`。

| 位置 | 已确认的行为 |
|---|---|
| QI `0x7477a791..a7b3` 与 `0x7477ab7e..ab99` | IPersistStream 位于对象 `+0x3c`，虚表 `0x747bc9ac` 的 Load/Save 槽为 `0x7477b8d0`、`0x7477bb00`。 |
| Load `0x7477b8ee..b951` | 读取两字节版本，原生拒绝有符号值大于 1。随后以 IGroupLayer IID `EDAD6644-1810-11D1-86AE-0000F8751720` 调用普通 LoadObject，结果写入完整对象 `+0x90`（相对 IPersistStream 接口 `+0x54`）。 |
| Load `0x7477b97c..ba80`、`0x7477babb..bacd` | 创建单独的 MemoryBlobStream，读取四字节无符号体长和原始字节。非零体长时把字节交给该流，作为 MdProcess 持久化输入，随后从原流读两字节到完整对象 `+0x94`。Save 在 `0x7477bb20..bb43` 写版本 1 及组对象，并在 `0x7477bc2b..bc68` 写体长、进程字节和 short。 |
| QI `0x7477aa43..aa72`、`0x7477a6b9..a6e8` | 分别返回对象 `+0x78` 的 ILayerMasking 和 `+0x34` 的 ICompositeLayer。后者虚表 `0x747bc97c` 的子层计数与索引方法 `0x7477abc0`、`0x7477ac30` 均通过接口相对 `+0x5c`（完整对象 `+0x90`）取出 IGroupLayer，再查询其 ICompositeLayer。Map 对 ProcessLayer 读取自身记录后直接遍历委派组的子层。 |

SDK .NET 的 `USA_Data_ClipTools` 在 `/Maps:0x1a68` 定义 ProcessLayer v1（键 113）。其 GroupLayer v11（键 114）有一个完整 FeatureLayer 子层（键 115）。独立进程流长 14,513 字节，尾 short 为零。Map v30 的逐层遮罩在 `0xbf3d` 消费一条属于键 113 的 v1 空记录，组本身没有独立记录。Map 对象于 `0xbf4d` 结束，随后四字节流尾值为 1。构造体覆盖非空进程流、全部截断位置及超过限额的体长。

当前将 MdProcess 字节原样保留，支持 IGroupLayer 槽定义新 GroupLayer 的分支。委派槽回引、替代 IGroupLayer 类、非标准版本及进程运行时行为仍需取证。ArcMap 与 QGIS 的人工验收未执行。

### SanFrancisco 组图层的跨图层 UID 回引

两份 SDK 工程在 /Maps 同一偏移上复现。顶层 GroupLayer 键 4501 的体从 `0x72d5e` 开始，首个 ExtraSafe 子对象为 FeatureLayer v34、键 4502、体起点 `0x72dba`。其 UID 槽 `0x73b68` 的类和键 358 与较早的 `0x98e8` 完全一致。较早对象位于首个 NALayer 的有界体内，头之后可逐字段识别出 UID v1 的 23 字节体。跳过 NALayer 略过了该键的首次登记。较晚位置头后的 `02 00 00 00 00 00` 为 FeatureLayer `+0x28c` 的空字符串，与 `0x6f6e9940` 的读取顺序相符。此前推定的 UID v2 短体来自将回引后的字段当成对象版本。

另从 10.8 Desktop 安装介质提取 SystemCore.dll 10.8.0.12790，SHA-256 `48cbd921972d12156119d524f9c3923225fd30795e33c22c65c87db3a6c89aca`。RTTI 对位 UID 的 IPersistStream 表 `0x101035fc`，GetClassID `0x100a1740`、Load `0x100a1f90`、Save `0x100a1ff0`。读取顺序仍为 2、1、16、4 字节，写出版本 1，与 10.1 原生入口相符。

### NALayer 的有界对象体前缀

上节 10.1 NetworkAnalyst.dll 的 RTTI 虚表 `0x7af9f48c` 将 GetClassID、Load、Save 绑定到 `0x7af210d0 / 0x7af1ffc0 / 0x7af20be0`。GetClassID 返回 `0x7af9b1d8` 的 NALayer CLSID。Load 在 `0x7af2000d..0x7af2002b` 读取 short，以有符号比较拒绝大于 3 的版本，随后在 `0x7af20031..0x7af200b8` 读取 AfCore String、三个 short 和两个 double。Rust 在已确认外层边界的图层条目中保留这些字段，剩余体仍按原边界保存。两份 SanFrancisco 工程均从键 31 读取名称 Route，前缀止于 `0x19a`，逐层遮罩仍停在组层键 4501。嵌套对象体及其共享键、ArcMap/QGIS 人工验收仍待验证。

### NALayer 首个 IName 槽

NALayer Load 在 `0x7af200da` 调到 `0x7af23830`，后者在 `0x7af2389f..0x7af238a8` 请求 IName `4BA33331-D55F-11D1-8882-0000F877762D`。两份实档首例均为 NetworkDatasetName，键 32。10.1.0.3035 GdbNetwork.dll（PE32，SHA-256 `36b1fcd7ea5951aea6c53e03855993a2fcbb2b78c3e8b068642a890c44a0995c`）的持久化虚表 `0x71196ca0` 由 GetClassID `0x71044240` 核对为 `8167FE6A-E992-4ECB-B6AE-E5DFE4655247`。Load `0x71044ff0` 读取有符号版本上限 1，在 `0x71045030 / 0x7104503a` 读两个字符串，随后在 `0x71045084 / 0x710450ae` 读 IDatasetName 和 IWorkspaceName 槽。两个 IID 来自 `0x71196700 / 0x71196d80`。

本次复用 FeatureDatasetName 和 WorkspaceName 读取器。两个嵌套槽完整读取后才提交名称槽及其对象键。SanFrancisco 两份实档中键 31 的停止点由 `0x19a` 推进到 `0x370`。下一个对象和后续 UID 回引仍待接入，ArcMap/QGIS 人工验收未验证。

### NALayer 的上下文与工作空间前缀

继续沿 NetworkAnalyst.dll 10.1.0.3035 的 NALayer::Load `0x7af1ffc0` 和 Save `0x7af20be0` 对位，首个 IName 槽之后是普通 ObjectStream 对象槽，保存 `NAContext`。NAContext 的持久化接口虚表由主 QI `0x7af0f6c0` 的 IPersistStream IID `00000109-0000-0000-C000-000000000046` 分支返回对象 `+0x38`；GetClassID `0x7af0f350`、Load `0x7af12a10`、Save `0x7af132d0`。CLSID 为 `E322B95B-B689-4C57-997F-BDB221204738`，与 esri.clsid.xml 的 `NAContext` progID 一致。Load 先读有符号上限 0 的 short，再读 AfCore String 和首个工作空间对象。

两份 SanFrancisco 实档在 NAContext 头 `0x370`（键 35）、版本 0 和名称 `Route` 之后，均出现 NetworkAnalysisWorkspace 对象头 `0x396`（键 36）。NetworkAnalyst.dll 的类 QI `0x7af7d040`、IPersistStream 虚表 `0x7afa35a0` 将 GetClassID/Load/Save 定位为 `0x7af7ebe0 / 0x7af7ec40 / 0x7af7ed20`。Load 读取有符号上限 1 的 short 后，委派 GdbCoreLib.dll 10.1.0.3035（SHA-256 `10e0af85e3e3825647733f99d573c969d69ccaf47203365e51323fce07008baa`）的 `InMemoryWorkspaceBase::LoadWorkspace 0x70d31ca0`。该函数依次读取三个 AfCore String、一个 IPropertySet 对象，再读取四个字节的命名项计数。SaveWorkspace `0x70d32220` 按同一顺序写出。

实档工作空间版本 1 的三个字符串从 `0x3ac` 起，分别为 `Route`、`Route:{08932E9A-AAED-4D8A-842D-2B2B0AEC6080}` 和 `{08932E9A-AAED-4D8A-842D-2B2B0AEC6080}`。IPropertySet 对象头在 `0x46c`（键 37），其 ObjectStream 体在 `0x480` 开始，版本 1、计数 2，两个条目为 `CLSID` 和 `DATABASE`。Rust 现已读取这些已确认字段并事务性登记键 35、36、37；两份实档的首个 NALayer 体停点由 `0x370` 推进到命名项计数 `0x512`。

计数之后每项是 AfCore String 加一个 NAClass 对象。第一项键 `polylinebarriers` 后的 NAClass 对象头在 `0x53c`（键 38），体起点 `0x550`。NAClass::Load `0x7aeffcb0` 先调用 GdbCoreLib `InMemoryClassBase::Load 0x70d192a0`，后者读取版本、两个字符串、16 字节 GUID、对象槽、Fields/Field 槽、记录计数及记录体，再返回 NAClass 读取自身版本和后续对象槽。两份实档中的跨层 UID 键 358 位于这些 NAClass/记录嵌套体之后，当前仍由外层有界未知区保留。因此上下文与前缀已接好，但尚未取得该 UID 的首次登记；后续回引、ArcMap/QGIS 人工验收仍未验证。

### 网络分析工作空间命名项与 NAClass

继续使用上述 10.1.0.3035 NetworkAnalyst.dll 和 GdbCoreLib.dll。补导出的伪代码在忽略目录 `temp/network-na/`，并与原始指令核对。GdbCoreLib 的 LoadWorkspace `0x70d31ca0` 在属性集后读取计数，依次读取每项的 AfCore String 与对象槽。实档该槽出现 NAClass `CF7AF4E6-4FBA-4FB2-A417-92B074B88C90`。其 Load `0x7aeffcb0` 委派 InMemoryClassBase `0x70d192a0`，随后读取自身 short 与 NAClassDefinition 槽。后者的 Load `0x7af022f0` 读取 BSTR、IUID、IFields、两个整数边界、两个 short 与字段类型项。

InMemoryClassBase 按已解析的 Fields 确定记录布局，`0x70d20220` 按 OID、可选带长度几何及其余字段的类型标签消费字节。当前保留已识别的类和字段定义，以及原始记录计数。记录值用于确定边界，尚未建模；未知标签或未展开的字段对象会停读，避免猜测长度。构造测试验证非空命名项和 UID 登记，并逐字节截断检查回退。

两份 SanFrancisco SDK 工程的首个 NALayer 从 `0x512` 推进到 `0x73d9`，跨过工作空间命名项和后续 IName 槽。该处下一个对象是 NARouteSolver（键 246）。UID 键 358 的首次定义仍未读到，逐层遮罩停在 GroupLayer 键 4501。ArcMap/QGIS 人工验收仍未验证。

### NARouteSolver 的版本与属性名前缀

10.1.0.3035 NetworkAnalystSolvers.dll（PE32，SHA-256 `a88f590573b8115a05921d624258214f9ea73baa7eaa720d182b7b5c2798234c`）的 NARouteSolver 类标识是 `1A8AA208-3D4A-420F-B82C-F58194A72CB0`。主 QI `0x7b0fb850` 的基类入口 `0x7b122b10` 在 `0x7b122c39..0x7b122d13` 对 IPersistStream 返回对象 `+0x2c`。RTTI 对应虚表 `0x7b202cec`，Load 为 `0x7b0fcdf0`。补导出伪代码并以 objdump 核对，Load 先读有符号上限 3 的 short，再调用 `0x7b12ba00` 读取 BSTR 属性名及一个 short。这个 short 非零时额外读取一个集合，随后进入共用的下一集合。

NALayer 已沿 NAContext 的求解器槽读出两份 SanFrancisco SDK 工程的 NARouteSolver v3、属性名 TravelTime 和标志 `0xffff`，首个 NALayer 停点由 `0x73d9` 推进到 `0x740b`。集合、后续求解器字段及 UID 键 358 的首次定义仍待读取。ArcMap/QGIS 人工验收仍未验证。

### NARouteSolver 的内联字符串集合

同版 NetworkAnalystSolvers.dll 的求解器共用读取器 `0x7b12ba00` 在集合标志非零时，先构造并读取额外集合，随后无条件读取第二个集合。两次均经 `0x7b138cd0` 创建对象，其中 `0x7b138cf6..0x7b138cfb` 将 `0x7b1fd854` 的 CLSID 传给 AfCreateInstance。其值为 SystemCore StrArray `A7F92065-36CE-47B6-A463-4763DA947CC2`。两处均直接调用新对象的持久化 Load，因此流内只有对象体，复用 StrArray 对象体读取器。

两份 SanFrancisco SDK 工程的首个求解器均从 `0x740b` 读过空的额外集合和含五个属性名的第二个集合，到达 `0x74c7` 的 ExtraSafe 槽。构造测试覆盖标志两种形态和逐字节截断。后面的 ExtraSafe 槽、剩余求解器字段和目标软件人工验收仍待验证。

### NARouteSolver 的 ExtraSafe 字符串数组

同版 NetworkAnalystSolvers.dll 的 `0x7b12ba00` 在两处内联集合之后调用 ExtraSafeReadObject，`0x7b12bbc4` 传入 `0x7b1fd834` 的 IStringArray IID `206216C9-7253-4A4D-92DD-60329FD9D792`。两份 SanFrancisco SDK 工程在 `0x74c7` 的该槽均给出 A=80、B=0，内含 StrArray v1，键 247，值为 Oneway 和 RestrictedTurns。

读取器复用有键的 StrArray 槽，以 A 确认写出体范围，或在非零 B 时使用原生同步边界。只有解析结果与声明边界一致时才提交游标和对象键。构造测试覆盖新对象、B 同步回引、所有截断和边界过长。实档槽末端为 `0x751f`。后续求解器字段、UID 首次定义和人工验收仍待验证。

### NARouteSolver 的层级列表与属性集

同版 NetworkAnalystSolvers.dll 的共用读取器 `0x7b12ba00` 在 ExtraSafe 槽后读取 short、LONG、BSTR 和声明计数。原始指令 `0x7b12bccc`、`0x7b12bd9d` 分别核对两个列表计数，`0x7b12bd57`、`0x7b12bdf2` 核对各列表从 1 开始的连续索引。每项为索引与一个四字节值，最后一项也消费该值。随后 `0x7b12be36` 用 `0x7b200d08` 的 IID 请求 IPropertySet。

两份 SanFrancisco SDK 工程在 `0x751f` 读到属性名 HierarchyMultiNet 和两份三项列表，再读完键 248 的 PropertySet，到达 `0x76d1`。构造数据验证重复计数、索引和截断回退。剩余版本字段、后续对象键及 ArcMap/QGIS 人工验收仍待验证。

### NARouteSolver 的版本尾段

NetworkAnalystSolvers.dll 10.1.0.3035 的共用读取器 `0x7b12ba00` 在属性集后读取 short，v1 起再读一个 short，v2 起读取参数数组槽，v3 再读 double 和 LONG。外层读取器 `0x7b0fcdf0` 在版本非零时另外读取 double 与 short。反编译记录见 `temp/network-na/solver-7b12ba00.c` 与 `temp/network-na/solver-7b0fcdf0.c`。

同版注册表给出参数数组 CLSID `61939E6B-8011-40C5-B672-73EA1E026FEC`。持久化表 `0x7b1fe700` 的 Load `0x7b09f110` 读取 short 版本及 LONG 计数，计数为正进入逐项读取。两份 SanFrancisco SDK 工程均从 `0x76d1` 读取键 249 的空数组，经 v3 字段到求解器末端 `0x7705`。构造测试覆盖 v0 至 v3、逐字节截断和非空数组回退。非空数组、后续 NAContext 槽和 ArcMap/QGIS 人工验收待验证。

### NAContext 求解器后的对象槽

NetworkAnalyst.dll 10.1.0.3035（SHA-256 `8c4ebaee71aad63c25616aa806674018479829d97bc01c7ed3b8845813698532`）的 NAContext::Load `0x7af12a10` 在求解器之后连续调用两次 LoadObject，请求 IID 常量分别位于 `0x7af9e7bc` 和 `0x7af9e71c`。DLL 的 .rdata 显示后者为 IPropertySet（`F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`）。对应 Save `0x7af132d0` 的两个槽依次取自对象偏移 `+0x14`（特定分支写空）和 `+0x24`。

两份 SanFrancisco SDK 工程均从 `0x7705` 读过 16 字节空槽，再从 `0x7715` 读过键 250 的空 PropertySet，停在 `0x772f`。该处新键 251 的 CLSID `4319ADE1-D6E5-4818-8380-8EE389D148EA` 在 10.1 的 esri.clsid.xml 注册为 NALocator。首槽中新对象的持久化体、NALocator 及后续槽仍待读取。构造测试验证属性集截断时保留已确认前缀并回滚新键。ArcMap/QGIS 人工验收未验证。

### NALocator 的代理记录前缀

同版 NetworkAnalyst.dll 的 GetClassID `0x7af31e50` 读取 CLSID 常量 `0x7af9b1f8`，通过调整 thunk `0x7af365c0` 与持久化虚表对位。该虚表的 Load 槽经 `0x7af365b0` 跳到 `0x7af31e90`，Save 经 `0x7af36540` 跳到 `0x7af322d0`。Ghidra 补充导出见 `temp/network-na/7af31e90.c`。Load 依次读取有符号上限 3 的 short、两个 double、LONG、short、ExtraSafe 空间参考槽（接口常量 `0x7afa0378`）与 LONG 代理计数。正数计数每项调用一次 ExtraSafeReadObject（接口常量 `0x7afa0388`）。代理循环后 v2 以上追加 double，v3 再读 short，v1 的单位换算走内存路径。

两份 SanFrancisco SDK 工程从 `0x772f` 读取键 251、v3 定位器、带 16 字节空头的空间参考槽及计数 2，到达首个代理封装 `0x7777`。该封装 A=56、B=0，内层 CLSID `22B4C9AB-32FA-4686-8F5E-8A2A370B3350` 与 10.1 注册表的 NALocatorFeatureAgent 一致。构造测试覆盖逐字节截断和键回滚。新空间参考对象体、代理循环、版本尾字段及 ArcMap/QGIS 人工验收仍待验证。

### NALocator 的要素代理与版本尾段

同版 NetworkAnalyst.dll 的要素代理持久化虚表 `0x7afa0764` 中 GetClassID `0x7af394c0` 引用 CLSID 常量 `0x7af9b208`，Load/Save 为 `0x7af39500 / 0x7af39760`。Ghidra 补充导出核对为 short 版本（有符号上限 2）、LONG、short、LONG、AfCore BSTR，v2 再追加一个 AfCore BSTR。代理封装在声明边界内完整消费才提交键；若代理循环或随后定位器版本尾段截断，则回到第一个代理封装之前并撤销该段新键。

两份 SanFrancisco SDK 工程读取了键 252 和 253 的 v2 要素代理及 v3 定位器尾段，到达 `0x781b` 的 NamedSet 对象头。定位器空间参考的新对象、其他代理类和 NAContext 后续槽仍待读取。ArcMap/QGIS 人工验收尚未进行。

### NAContext 的 NamedSet 集合

同版 NetworkAnalyst.dll 的 NAContext::Load `0x7af12a10` 在定位器后两次请求位于 `0x7af9e76c` 的接口。NamedSet（CLSID `F6B4FE84-7A02-4969-9E72-11B4F8377155`）的持久化虚表 `0x7afa0ce4` 将读写定位到 `0x7af43920 / 0x7af43ba0`。Ghidra 导出显示 Load 读取 short（有符号上限 1）、LONG 计数，每项依次为 AfCore String 与请求 IUnknown 的普通对象槽（IID 常量 `0x7af98bac`）。读取器复用 NAClass 分派及共享回引表，条目截断或遇到未支持的新对象时回滚整个集合。

两份 SanFrancisco SDK 工程在 `0x781b` 的首个集合键 254 中读出五个 NAClass 回引，到达 `0x791b` 的第二个集合。第二个集合键 255 的首项对象键 256 注册为 NAStreetDirectionsAgent，CLSID `C21BC9A1-B784-48F1-8FE7-DF7F12F77D72`。同版 Directions.dll（SHA-256 `5b06cec21feb2bcbe72d304f214058a1ce6902abc2391116f1c696f77857945f`）在 `0x6eadc130` 保存该 CLSID，代码引用位于 `0x6ea2d4c7` 和 `0x6ead53f7`。方向代理对象体及 ArcMap/QGIS 人工验收仍待验证。

### 第二个 NamedSet 的方向代理

同版 Directions.dll 的街道方向代理持久化虚表 `0x6eadc0d4` 指向 Load/Save `0x6ea2d510 / 0x6ea2d770`，紧凑街道方向代理虚表 `0x6eadb3ac` 指向 `0x6ea224a0 / 0x6ea226b0`。Ghidra 导出确认两者都使用有符号 LONG 版本。v0 起读取 LONG 和 AfCore String，街道方向类在字符串前后各多读一个 LONG。v1 起追加一个 AfCore String，v2 起追加 LONG 和 AfCore String。读取器限制到已核对的 v2，复用 NamedSet 的对象表事务。

两份 SanFrancisco SDK 工程均读完第二个集合键 255 及方向代理键 256、257，停在 `0x7a73`。其后出现数值 5 及键 258 的 FeatureLayer，对应上层持久化边界仍待确认。构造测试覆盖两种代理及逐字节截断时的集合回滚。ArcMap/QGIS 人工验收尚未进行。

### NALayer 的首个子图层循环

同版 NetworkAnalyst.dll 的 NALayer::Load `0x7af1ffc0` 在上下文返回后读取有符号 LONG，正数逐次调用普通 LoadObject，IID 常量 `0x7af9f5e0` 为 ILayer（`34C20002-4D3C-11D0-92D8-00805F7C28B0`）。Save `0x7af20be0` 从数组 `+0xc` 写出经过过滤的计数和对象。循环没有单项长度，FeatureLayer 必须达到自身的完整停止点，才能提交对象键并进入下一项。

两份 SanFrancisco SDK 工程均在 `0x7a73` 读出计数 5。首项 FeatureLayer 键 258 从 `0x7a77` 开始，读取到名为 Stops 的子图层中键 261 的渲染器槽。10.1 注册表将该类 `7CF9F6E3-376A-49B2-BAEE-C8300798AECE` 定名为 NAStopRenderer。首项未完时游标和键退回项首，外层保留剩余字节。构造测试覆盖已完成的回引、未支持的新对象和短计数区。NAStopRenderer 对象体、后续子图层与 ArcMap/QGIS 人工验收仍待验证。

### NAStopRenderer 的图例与分类数组

同版 NetworkAnalyst.dll 的 NAStopRenderer GetClassID `0x7af688c0` 在持久化虚表 `0x7afa2870` 对应 Load/Save `0x7af6a360 / 0x7af6a5e0`。Ghidra 导出显示 Load 先探测 16 字节：LegendGroup CLSID 表示旧版并回退到对象头，零 GUID 表示随后有版本 short。三个普通槽依次请求 ILegendGroup、IColor、ITextSymbol。版本至少 2 时，按图例组分类数读取整组可见性 short，再读整组索引 LONG。带分类数组的图例组回引需要恢复原组计数，当前仍待支持。

两份 SanFrancisco SDK 工程均读完键 261 的 v2 渲染器及首个 Route NALayer 的五个子图层，停点从 `0x7a77` 推进到 `0xed73`，该 NALayer 尚余 94 字节。此前遮罩组层的 UID 回引停点随对象键登记推进到了另一个未完整的子层类。构造测试覆盖分类数组和逐字节截断回滚。ArcMap/QGIS 人工验收尚未进行。

### NALayer 的版本尾段

同版 NetworkAnalyst.dll 的 Load/Save `0x7af1ffc0 / 0x7af20be0` 在首组子图层之后读写 short、普通对象槽（IID 常量 `0x7af9f570`）、double、ExtensionHelper 序列、AfCore String、short 和两个 double。v1 起追加普通对象计数循环，v2 追加 short，v3 追加两个位集。位集读侧 `0x7af24900` 消费四个 LONG 和一个字节。该字节非零时按第四个 LONG 再读取对应数量的四字节字，原始指令 `0x7af249cf` 和 `0x7af24a30` 附近可核对此分支。

两份 SanFrancisco SDK 工程均从 `0xed73` 完整读完 Route NALayer 尾部 94 字节，结束于 `0xedd1`，包含空扩展序列和空第二组子图层。构造测试覆盖内联位集字及逐字节截断。尾部普通槽的新对象、非空扩展序列和其他网络求解器仍有缺口。ArcMap/QGIS 人工验收尚未进行。

### NAStopRenderer 的图例组回引

NetworkAnalyst.dll 10.1.0.3035 的 Load `0x7af6a360` 和 Save `0x7af6a5e0` 在回引与新对象路径汇合后，均调用已取得图例组的虚表 `+0x24` 获得分类数。v2 的可见性和索引数组由该计数定界，回引后没有额外计数字段。读取器随 ObjectStream 键表保存已完整解析的 LegendGroup 分类数。内联图例组没有独立键，不登记此元数据。如果只见对象键而缺少完整定义，则保留渲染器停止边界。

构造测试使用一次新图例组和随后同键回引，核对两次得到相同的数组，并验证缺少分类数时回滚。ArcMap/QGIS 人工验收尚未进行。

### 最近设施求解器

NetworkAnalystSolvers.dll 10.1.0.3035 的 GetClassID `0x7b0c4960` 引用 `0x7b1fed34`，与 10.1 注册表中 NAClosestFacilitySolver 的 CLSID `771F83D4-C698-4836-8222-C0B689A85827` 一致。原始指令及持久化表中的 `0x7b200c68` 将 Load 定位至 `0x7b0c6120`。该函数限制有符号版本上限为 4，依照与路径求解器相同的规则把外层版本映射到 `0x7b12ba00` 的共用版本。只有外层 v4 在共用体之后再读 double 和 LONG。

两份 SanFrancisco SDK 工程从 `0x17a0f` 的键 876 读取 v3 求解器，随后均完整消费 Closest Facility 图层声明区间。构造测试检验 v3 与 v4 的边界和 v4 截断回滚。其他网络求解器及 ArcMap/QGIS 人工验收仍待完成。

### 服务区求解器

NetworkAnalystSolvers.dll 10.1.0.3035 的 GetClassID `0x7b111e10` 引用 CLSID 常量 `0x7b1fed24`，与注册表 NAServiceAreaSolver 一致。持久化表 Load 槽 `0x7b203408` 指向 `0x7b114870`。原始指令显示外层有符号上限为 4，调用共用读取器 `0x7b12ba00`。外层版本非零时追加三个 short、double、LONG，v2 起再读 short，v4 再读 double 和 LONG。

两份 SanFrancisco SDK 工程均读取 `0x27886` 的 v3 求解器键 1514，并完整消费 Service Area 图层声明区间。构造测试覆盖 v3/v4 差异和 v4 截断回滚。其他求解器与 ArcMap/QGIS 人工验收仍待完成。

### OD 成本矩阵求解器

NetworkAnalystSolvers.dll 10.1.0.3035 的持久化 Load 槽 `0x7b202420` 指向 `0x7b0f2eb0`。对象 CLSID `865BE173-5D20-4F11-9016-FF27BCA167E2` 与注册表中的 NAODCostMatrixSolver 一致。原始指令显示有符号版本上限为 5。`0x7b0f2f2c–0x7b0f2f58` 将外层 v2/v3 都映射到共用 v2，v4/v5 使用共用 v3。外层 v3 起在共用体后追加 LONG 和 short，v5 再追加 double 和 LONG。

两份 SanFrancisco SDK 工程均读取 `0x3662f` 的 v4 求解器键 2094，并完整消费 OD Cost Matrix 图层声明区间。构造测试覆盖 v2 至 v5 及逐字节截断回滚。其他求解器与 ArcMap/QGIS 人工验收仍待完成。

### 车辆路径求解器

NetworkAnalystSolvers.dll 10.1.0.3035 的 NAVRPSolver CLSID `52B62278-735F-4DA1-8A4E-2E6F4E3BCD60` 位于 `0x7b1fed64`，持久化 Load 槽 `0x7b206ed0` 指向 `0x7b14b230`。原始指令和 Ghidra 导出确认版本有符号上限为 2，外层 v0/v1 使用共用 v2，v2 使用共用 v3。共用体后读取 LONG、short、两个 LONG、double、LONG、四个 double、LONG。外层版本非零时追加 ExtraSafe 对象槽，接口常量位于 `0x7b207044`。

两份 SanFrancisco SDK 工程均完整读取 `0x5061c` 的 v2 求解器键 3018 和 ExtraSafe 内的 NAStreetDirectionsAgent 键 3021，随后继续读取 NAContext 和子图层，遇到 `0x5e692` 的 StandaloneTable（`EE7C5047-E3DB-11D3-A096-00C04F6BC626`）。构造测试覆盖 v0 至 v2 和逐字节截断回滚。剩余子图层及 ArcMap/QGIS 人工验收仍待完成。

### NALayer 的独立表集合

同版 NetworkAnalyst.dll 的 NALayer::Load `0x7af1ffc0` 对 v1 起的第二个计数循环请求 `0x7af9f760` 的 IID，即 `FFC6B179-E3EC-11D3-A096-00C04F6BC626`（IStandaloneTable）。Save `0x7af20be0` 写出数组 `+0x20` 的普通对象。读取器复用 Map 独立表的对象体读取规则，并要求表体完整后才进入下一项。

两份 SanFrancisco SDK 工程的车辆路径图层先前因整段回滚而报在尾部起点 `0x5e692`，首个 StandaloneTable 对象头实际位于 `0x5e6cc`。接入后两份工程均读完四张独立表及随后的位集，到达图层声明边界。构造测试覆盖独立表回引、空槽和逐字节截断回滚。ArcMap/QGIS 人工验收尚未进行。

### 位置分配求解器

同版 NetworkAnalystSolvers.dll 的持久化槽 `0x7b201b30` 指向 Load `0x7b0d2cb0`，10.1 注册表将 CLSID `659CC65B-D656-4F32-87F2-FD0D3058DDF9` 定名为 NALocationAllocationSolver。Ghidra 补充导出及原始指令确认有符号上限 4，外层 v0/v1 使用共用 v2，v2 起使用共用 v3。随后读取四个 LONG、三个 double 和 LONG。外层 v3 新增 double 和 LONG，v4 再新增 double。

两份 SanFrancisco SDK 工程均从 `0x68bb2` 读取 v2 求解器键 4003，直至完整消费 Location-Allocation 图层的声明区间。构造测试覆盖 v0 至 v4 和逐字节截断回滚。遮罩组图层键 4501 的未完整子层及 ArcMap/QGIS 人工验收仍待处理。

### SanFrancisco UID 回引复验

在分级渲染器与网络分析图层的解析接入后，以当前命令行对 .NET 和 Java 两份 SanFrancisco SDK 工程复验。六个顶层 NALayer 和键 4501 的 Analysis Inputs 组均完整消费其声明区间。这验证了前述 UID 键 358 的共享引用路径，逐图层遮罩已越过该组。两份工程现在都在底图层键 5146 停止。底图读取器报告 `/Maps:0xd2107` 的新建 GroupLayer 键 5148，当前正式子层分派尚不能处理。该底图缺口单列于 TODO。ArcMap/QGIS 人工验收尚未进行。

### SanFrancisco 底图正式子层

BasemapLayer.dll 10.1.0.3035 的 Load `0x6b646260` 在 `0x6b6464eb` 对正式子层调用 ExtraSafeReadObject 并请求 ILayer。读取器以封装边界复用完整 GroupLayer 和 FeatureLayer 读侧，只在精确消费范围后登记子层及其嵌套键。构造样例验证多余字节触发游标和键回滚。

两份 SanFrancisco SDK 工程均读取键 5148 的 GroupLayer 和后续 FeatureLayer，底图对象在 `/Maps:0xdb52d` 到达声明边界。逐层遮罩已进入底图分派，目前在尝试读取键 5148 的遮罩时停止。底图对遮罩遍历暴露的子层枚举仍需核对。ArcMap/QGIS 人工验收尚未进行。

### 底图正式子层的遮罩分派

BasemapLayer.dll 10.1.0.3035 的 `0x6b6436b0 / 0x6b6436f0` 从当前集合返回计数和图层包装接口。Load 通过集合插入路径 `0x6b64b0a0 → 0x6b644000` 装入正式子层。包装类 QI 的后备 `0x6b64ad04 → 0x6b69ebc0` 只处理固定 IID，对 ILayerMasking 在 `0x6b69ecaa` 返回 E_NOINTERFACE。Map.dll `0x79403bc0` 收到空遮罩接口后直接返回，因此底图内包装的组不消费独立遮罩记录。读取器对已完整确认的组和要素子层应用此规则，对其他正式子层保持停止。

两份 SanFrancisco SDK 工程均越过键 5148 的旧错位，读取两条逐图层遮罩记录并进入后续 Map 版本段。两份目前共同停在阶段 40，未知尾部起于 `/Maps:0xdbc13`，余 86 字节。ArcMap/QGIS 人工验收尚未进行。

### NetworkLayer 的数据集名称

同版 NetworkLayer.dll 的持久化虚表 `0x7b2a9ed4` 将 Load/Save 定位到 `0x7b2903d0 / 0x7b2908a0`。Ghidra 补充导出与原始指令确认 Load 读取有符号上限 0 的 short、AfCore String，再经 `0x7b291140` 以 `0x7b2a9fa4` 的 IName IID 调用 LoadObject。读取器复用 NALayer 的 NetworkDatasetName 分派。构造测试覆盖名称回引和逐字节截断。

两份 SanFrancisco SDK 工程的键 5053 从 `/Maps:0xd05b4` 开始读到名称 Streets_ND，消费网络名称对象及嵌套键后停于 `0xd07a6`。Map 根尾与 PageLayout 对象和打印尾部均已读取。NetworkLayer 后续对象和 ArcMap/QGIS 人工验收仍待进行。

### NetworkLayer 的显示属性与渲染器计数

NetworkLayer.dll 10.1.0.3035 的 Load `0x7b2903d0` 在名称之后请求 `0x7b2a9fe4` 的 ISpatialReference 和 `0x7b2aa004` 的 IDisplayFilter。其后由 Ghidra 导出和 Save `0x7b2908a0` 核对 short、String、short、两个 double、两个 short、double、ExtensionHelper 序列、String、两个 double，以及 LONG 渲染器计数。读取器复用现有空间参考、显示过滤器和扩展分派，截断时回滚这个字段段落的游标和键。

两份 SanFrancisco SDK 工程从 `/Maps:0xd07a6` 读取 GCS 键 30 回引和空过滤器，读出渲染器计数 5，停于 `0xd080e` 的首项 short。随后对象键 5057 对应 NetworkDirtyAreaRenderer。渲染器循环和 ArcMap/QGIS 人工验收尚待处理。

### NetworkLayer 的脏区渲染器

NetworkLayer.dll 10.1.0.3035 的 Load `0x7b2903d0` 在计数之后逐项读取 short、普通对象槽（IID 位于 `0x7b2a7a20`）及 16 字节字段。NetworkDirtyAreaRenderer GetClassID `0x7b284810` 返回 `0x7b2a7a60` 的 CLSID，持久化虚表 `0x7b2a8d64` 的 Load/Save 为 `0x7b285590 / 0x7b285490`。Ghidra 补充导出确认体内只有有符号上限 1 的 short 和 ILegendGroup 槽。完整读取一项后提交它及嵌套对象键。

两份 SanFrancisco SDK 工程均从 `0xd080e` 读完脏区渲染器键 5057，停于 `0xd0954` 的第二项，其渲染器键 5064。构造测试覆盖项尾及逐字节截断回滚。其余网络渲染器与 ArcMap/QGIS 人工验收仍待处理。

### 简单网络渲染器

NetworkLayer.dll 10.1.0.3035 的 SimpleNetworkRenderer GetClassID `0x7b29e810` 引用 `0x7b2a7a50` 的 CLSID。持久化表 `0x7b2aaab4` 的 Load/Save 为 `0x7b29e770 / 0x7b29e650`。Load 读取版本上限 2 的 short，经共用体 `0x7b298990` 读取版本上限 3 的 short、String、两个 LONG、String 和三个 ISymbol 槽。共用 v2 起有三个 short，v3 起有来源计数及逐项 String、字节、String。外层最后读取 ILegendGroup 槽。Ghidra 补充导出与共用 Save `0x7b298550` 可复核。

两份 SanFrancisco SDK 工程均读过键 5064 的简单网络渲染器，解析推进至 `/Maps:0xd0d78` 的下一未支持项，键 5079。构造测试覆盖来源列表、项尾与逐字节截断回滚。ArcMap/QGIS 人工验收尚未进行。

### 网络路况渲染器

NetworkLayer.dll 10.1.0.3035 的 NetworkTrafficRenderer 持久化槽指向 Load/Save `0x7b29b380 / 0x7b29b070`。Ghidra 补充导出和原始指令确认版本上限 2，读共用体 `0x7b298990` 后读 String 和 ILegendGroup，并按图例虚表 `+0x24` 返回的分类数读取 double。v2 追加两 short、IDoubleArray、short、IDoubleArray、两 short 和 IColor。构造测试用已登记的图例组回引定界分级数组并核对 v2 槽位边界。

两份 SanFrancisco SDK 工程均从 `/Maps:0xd0d78` 继续读完声明的五个条目，NetworkLayer 完整抵达外层声明边界。ArcMap/QGIS 人工验收尚未进行。

### 非空网络属性参数数组

NetworkAnalystSolvers.dll 10.1.0.3035 的数组 Load `0x7b09f110` 经虚表 `0x7b1fe6b8` 的项读取函数 `0x7b09ec60` 调用普通 LoadObject，再查询参数值接口，故新对象及回引用同一对象键表。参数值 CLSID `F8ED3FA8-0B8B-4A56-A5B1-22973B3290C9` 的持久化虚表 `0x7b200510` 经 `0x7b0befe0` 到 Load `0x7b0bd1c0`。原始指令和导入表确认它读取有符号上限 0 的 short、两个 AfCore BSTR、LONG，最后在 `0x7b0bd255` 调用 Variant::Read(false)。读取器复用该填充式 Variant 分派，逐项读取后随求解器尾段一起提交键。

构造测试覆盖新参数、同键回引和逐字节截断。两份 SanFrancisco SDK 工程用于回归现有空数组路径。非空数组尚缺真实样例和 ArcMap/QGIS 人工验收。

### NALocator 的空间参考对象

NetworkAnalyst.dll 10.1.0.3035 的 NALocator::Load `0x7af31e90` 使用 `0x7afa0378` 的 IID `4434D680-F4DA-11D1-BCA6-0000F875BCCE` 请求 ISpatialReference。该 ExtraSafe 槽复用普通对象流的 PCS、GCS、UnknownCoordinateSystem 及回引分派，只有完整消费封装长度才提交新键。构造测试覆盖新 UnknownCoordinateSystem 及声明边界多出一字节时的回滚。两份 SanFrancisco 工程的此槽均为空，非空形态尚无真实宿主与人工验收。

### NALayer 尾部的 IEnvelope

NetworkAnalyst.dll 10.1.0.3035 的 Load `0x7af1ffc0` 在首组子图层及其后 short 之后，以普通 LoadObject 请求 `0x7af9f570` 的 IID `1B077914-38C5-11D0-92D2-00805F7C28B0`（IEnvelope），与 Save `0x7af20be0` 的对象槽对位。读取器复用 Envelope 体及空间参考分派，尾段完整后提交新键。构造测试覆盖非空范围和逐字节截断回滚。两份 SanFrancisco SDK 工程用于现有路径的回归验证。非空槽尚缺真实工程及 ArcMap/QGIS 人工验收。

### NALayer 尾部的扩展序列

NetworkAnalyst.dll 10.1.0.3035 的 Load `0x7af1ffc0` 在 IEnvelope 槽与 double 后调用 LayerLib 的 ExtensionHelper::LoadExtensions，Save `0x7af20be0` 在对应位置调用 SaveExtensions。复用同版 LayerLib `0x786c22e0` 对应的读取器，序列为 LONG 计数、每项八字节长度、CLSID 和体。已支持的扩展按声明边界解析，未知非空类保持尾段停止。构造测试覆盖 PropertySet 扩展、后续字符串边界及长度不一致时的回滚；非空 NALayer 扩展仍缺真实工程及 ArcMap/QGIS 人工验收。

### 定位器的位置字段代理

NetworkAnalyst.dll 10.1.0.3035 的持久化虚表 `0x7afa0bb0` 经跳板 `0x7af430b0 / 0x7af43150` 调到 Load/Save `0x7af423b0 / 0x7af425a0`。GetClassID `0x7af42370` 返回注册表中 NALocatorLocationFieldsAgent 的 CLSID。Ghidra 补充导出确认版本 short 的有符号上限为 2，随后四个 AfCore BSTR，v2 再追加一个。对象通过 NALocator 的已有 ExtraSafe 槽读取并校验完整边界。构造测试覆盖 v1/v2、后继字节和截断回滚；真实宿主与 ArcMap/QGIS 人工验收仍待验证。
