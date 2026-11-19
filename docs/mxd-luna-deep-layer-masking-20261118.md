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

Rust 按上述顺序消费已完整识别的 GroupLayer 自身和子层遮罩，空槽无记录。回引须匹配先前确认的对象键和类，重复回引按各自位置再走一遍子树。畸形回引受递归深度上限保护。遇到未知子层、未完整消费的子层或遮罩截断，整个逐图层循环回退到循环起点。嵌套 BasemapLayer 可以在 Map v28–v39 中产生遮罩记录；其正式子层集合仍限于 PrepareResult 中完整确认的 FeatureLayer 回引。图层树中未识别的图层类接口、不能完整读取的新遮罩对象体及 ArcMap/QGIS 人工验收仍开放。

Map v28 的构造测试覆盖普通组图层子层项、嵌套 Basemap、重复子层回引、顶层回引、错类回引和截断。该测试验证对象停止点和记录顺序，尚缺原生生成的嵌套遮罩实档。

## 遮罩集合中的新对象

LayerLib.dll 10.1.0.3035（SHA-256 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）的 `LayerMasking::LoadLayerMasking 0x786f1ff0` 已有 Ghidra 导出。复核原始指令及两处 IID 的 PE 字节后，`0x786f21b8` 将 `0x7871fc68` 传给外层 LoadObject，即 `{34C20002-4D3C-11D0-92D8-00805F7C28B0}`。同版本 `ESRI.ArcGIS.Carto.dll` 的 GuidAttribute 将该 IID 定名为 ILayer。`0x786f221c` 将 `0x7871e78c` 传给内层 LoadObject，即 `{00000000-0000-0000-C000-000000000046}`（IUnknown）。两者均走普通对象槽，头部没有外层体长。

两处新对象都可交给既有完整图层体分派器。对象键与嵌套对象键随遮罩集合一起提交。未知类或截断时集合回退，内层非图层的 IUnknown 对象仍需各类持久化证据。现有结果保留对象头、目标名称和遮罩尾字段。构造测试令外层和内层分别引入新图层，再回引先前内层对象，并逐字节截断检查游标和登记表回退。

## 回归结果

`cargo fmt --check`、`git diff --check` 通过。全量测试含忽略项共 1035 项通过。CLI 对仓库样例和 66 份 10.1 介质工程均读完 `/Maps` 与 `/PageLayout`。按实际对象键回查，10.1 介质的 18 份工程共读出 22 条 BasemapLayer 遮罩记录，集合条目数均为 0。68 份 10.8 介质补充对照中，60 份读到地图根尾，41 份读完布局。组层递归和集合内新对象由构造测试覆盖。ArcMap 和 QGIS 人工验收未做。
