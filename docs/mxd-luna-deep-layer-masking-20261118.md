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
