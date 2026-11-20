# 第四轮 Map 与空间对象取证

日期：2026-11-17。表中编号为交接时 TODO 原行号；本组未编辑 TODO、README、公共模块接线或对象流通用文件。仅限 GIS 格式互操作。反编译为静态证据，不能替代运行观察及 ArcMap/QGIS 人工验证。

## 身份与本轮范围

重新对本地 Setup1.cab 提取物计算摘要并用 pefile 核对版本资源：Map.dll 为 PE x86、InternalName `Map`、OriginalFilename `Map.DLL`、FileVersion 10.1.0.3035，SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；Geometry.dll 为 PE x86、InternalName `Geometry`、OriginalFilename `Geometry.DLL`、FileVersion 10.1.0.3035，SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`。与既有身份记录一致；版本资源是文件自身声明，并非签名真实性或目标环境运行证明。

复用同摘要的既有 Ghidra 桥接工具 6.9.0/Ghidra 12.2 证据包 `temp/arcgis/arcmap101-map-cr-extent-evidence.json`，Map Load `0x793ff4d0` 的 Evidence `ev_069e1a6cf0430ee69dd90c7c7f562b775368daf04cca8a0fd201e4e59102951e`，Save `0x79401850` 的批量反编译 Evidence `ev_db66d1a2485171567327850f20933669ba520abf22d6fc735a0a09274cbe1a35`。本轮提取局部伪代码至本组忽略目录；没有声称重新反编译或运行目标进程。第三轮照明证据见 `mxd-round3-map-tail-20261117.md`，空间证据见 `mxd-round3-display-spatial-20261117.md`。

## 原 117：Map 后缀与独立表计数

Load 的持久化子接口相对偏移，照明对象之后对于本入口所支持的 Map 版本 23..=51 顺序读 `short +0x1a0`、一个临时 short、`int32 +0x1b0`、`short +0x1ac`、`short +0x268`，共 12 字节；Save 对应顺序写同槽及值为零的临时 short。Load 在版本 <35 时读后覆盖 `+0x1a0` 为 `0xffff`，所以读取器保留流中的原值，不能直接当作最终运行时属性。其后还有对象等分支；各字段业务名称仍未证实。新增 `read_post_illumination_fields_10_1`，仅由调用者在确认 Map 版本和照明槽边界后显式调用；版本超界或截断失败不推进游标。其独立于现有 Map 后缀接线，避免在共享中间模型尚未承载时丢弃这些原始字节。

参考 MXD 从 Map 根和实际共享键表连续解析，照明体止于 `/Maps:0x24fe8`；随后读取器消费上述 12 字节至 `0x24ff4`。此为辅助字节对位，非对属性含义和后续对象的验证。

**纠正既有标注**：书签后的第一个计数循环，版本 >=11 走 Map Load 的 `DAT_79488300`，其完整 IID `{FFC6B179-E3EC-11D3-A096-00C04F6BC626}` 已由既有 10.1 类型库记录为 `IStandaloneTable`；它是独立表集合，不是坐标变换集合。坐标变换请求 IID `{9273BB60-C611-11D2-BD08-0000F875BCCE}` (`IGeoTransformation`) 出现在照明和若干定长字段之后的另一个循环。见 `arcmap-10.1-map-object.md` 的 IID 表与本轮复核伪代码。现有字段名 `transformation_count` 为兼容而暂时保留，专属 Map 文件的诊断已改为独立表；共享 `mxd_maps.rs` 中的同名诊断及其测试仍需整合方修正。非零独立表尚未消费，真正的坐标变换体未解析。旧版 <11 走 IName，本轮新读取入口不覆盖该旧版。

## 原行状态与欠缺

| 原行 | 状态 | 证据与仍需工作 |
| --- | --- | --- |
| 117 | 部分 | 如上；定长读取入口和真实参考样本边界已推进；书签后三成员业务含义、非空独立表、真正的坐标变换及更多后缀仍开放。照明体是第三轮已实现，运行及语义不因此闭合。 |
| 238 | 阻塞 | `mxd-geometry-luna.md` 与 `arcmap-10.1-geometry.md` 静态记录 +0xc8 指针的虚表 +0x1cc 调用；尚无 ArcMap 10.1 进程中的装配、实际接收对象及调用栈。未以静态间接调用猜测运行身份。 |
| 243 | 阻塞 | `{97D5B914-FB22-11D1-94A2-080009EEBECB}` 在既有 79 份类型库枚举及 FeatureLayer.dll 有界字面搜索中未命中，见 `mxd-iid-research-20261116.md`；负结果不能定名。 |
| 244 | 部分 | 既有 FDO QI 未定名接口列于 `mxd-iid-research-20261116.md`；本组未对其余 IID 建立新的肯定身份。请整合方与负责相关对象组协调，245 的五 IID 归数据集组。 |
| 247 | 部分 | `arcmap-10.1-pcs-geometry-slots.md` 的持久化路径、读写顺序、slot 8 常量及 GCS 守卫为静态证据；域串、填充来源、slot 名、守卫业务含义和 pe/sg 数值推导仍待访问器及调用链。 |
| 269 | 阻塞 | Hyperlink v0 仅有按静态门构造的测试；未获得可证归属的真实旧档短体。 |
| 270 | 阻塞 | 循环空标记/回引沿通用对象流规则实现，但未找到能从图层根顺序到达此类实例的真实例证。 |
| 277 | 阻塞 | Geometry v0/1 已有静态分支及合成覆盖，仍缺真实版本字节或目标运行记录。 |
| 278 | 阻塞 | S_FALSE 是 IStream 返回状态，普通文件字节不能证明；需可控目标运行观察。 |
| 281 | 阻塞 | PropertySet v0 已有合成边界测试，未取得可从宿主解析链确认的真实旧档。 |
| 282 | 阻塞 | Envelope v0 读取器及合成边界存在，仍需真实旧档、已确认对象边界的字节佐证。 |

尝试现有介质与环境：两套提取目录共 66 个 10.1 介质文件、68 个 10.8 介质文件。以真实 `/Maps` 起点、Map 对象头及同一引用表运行当前根读取器：前者 58 个单 Map 根、40 个到达书签后前导并读到照明后 12 字节；后者分别为 56、11；这些已到达前导的计数均为零。其余因根数量或当前解析边界未进入统计；不是对各类、各旧版和整个介质的穷举，也不能证明目标实例不存在。10.8 介质可能含旧版本文件，不等于 10.8 写出。Wine 可执行文件存在，但仓库 `temp/mxd101-wine/drive_c` 文件清单没有发现 ArcMap.exe、Geometry.dll、SystemCore.dll 或 FeatureLayer.dll 的安装副本；未建立可用的 10.1 运行时，也未实施 COM 或界面运行实验。

## 验证与交接

`tests/round4_map_spatial.rs` 覆盖真实参考文件从根到该段的连续解析、原始位模式与末尾保留、版本 23/34/35/51、版本外拒绝和全部截断点回退；介质枚举测试手动 `--include-ignored` 执行，不作为每次默认测试的介质依赖。合成边界仅验证读取器，不替代真实旧档证据。静态 Ghidra 桥接工具 与样本对位不构成 ArcMap 10.1 或 QGIS 人工验收，本轮两者均未打开。

请整合方：若要在 `mxd_maps.rs` 接入该 12 字节入口，先定义中间模型/诊断如何保存全部原始值和剩余字节；随后修正 `transformation_count` 字段名及共享诊断、相关测试，避免现有 UI 把独立表误称为坐标变换。非零表和后续坐标变换需要对象类别及对象体证据后再消费。暂不改变共享接线。

主审接线已完成：字段名现为 `standalone_table_count`；照明后的原始字段进入解析结果和 CLI 诊断。见[第四轮复核](mxd-round4-review-20261117.md)。

## 书签后与照明后字段的接口定名

重新核验 x86 `Map.dll` `10.1.0.3035` 的版本资源和本页所列 SHA-256。由 COL 将 IPersistStream（`0x794874a4`）、IViewManager（`0x7948734c`）、IMap（`0x79486f14`）分别定位在完整对象 `+0x68`、`+0x58`、`+0x28`。用 10.1 介质 `esriCarto` 类型库中的 IMap（TypeInfo 179）和 IViewManager（TypeInfo 641）定位接口槽，再用 objdump 和 Map.dll 已有反编译稿对位读写成员。所用 `MSFT` 类型库主体 SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。

| 存盘字段 | 完整对象偏移 | 原生证据 |
| --- | --- | --- |
| 书签后 short `+0x19c` | `+0x204` | IViewManager.VerboseEvents（get/put `+0x38/+0x3c`），`0x793fc7c0 / 0x793fc7f0` 访问接口 `+0x1ac`。 |
| 书签后 double `+0x200` | `+0x268` | IMap.ReferenceScale（get/put `+0x54/+0x58`）。put `0x793efa90` 调 `0x7940fd80`，写入完整对象 `+0x268` 并同步显示变换。Load 在 `0x793ffbc1` 调同一助手。 |
| 书签后 short `+0x264` | `+0x2cc` | IMap.UseSymbolLevels（put/get `+0xd0/+0xd4`），`0x793ef850 / 0x793ef820` 访问接口 `+0x2a4`。Load 同时复制一份到持久化视图 `+0x266`。 |
| 照明后 short `+0x1a0` | `+0x208` | IViewManager.ConserveMemory（put/get `+0x14/+0x18`），`0x793fc650 / 0x793fc670` 访问接口 `+0x1b0`。Map 版本小于 35 时 Load 读后改写为 `0xffff`。 |
| 照明后的第二个 short | 栈上临时值 | Load 读取后继续，Save `0x79401dbe..0x79401dc9` 写零。 |
| 照明后 LONG `+0x1b0` | `+0x218` | IViewManager.OutputBandSize（put/get `+0x2c/+0x30`），`0x793fc760 / 0x793fc780` 访问接口 `+0x1c0`。 |
| 照明后 short `+0x1ac` | `+0x214` | IViewManager.DelayBackgroundDraw（get/put `+0x40/+0x44`），`0x793fc830 / 0x793fc810` 访问接口 `+0x1bc`。 |
| 照明后 short `+0x268` | `+0x2d0` | IMap.Expanded（put/get `+0xd8/+0xdc`），`0x793ef8a0 / 0x793ef870` 访问接口 `+0x2a8`。 |

原字段及其位模式继续保留，CLI 同时输出接口属性名与持久化偏移。参考 MXD 在 `0x24fa0` 的前导包含 ReferenceScale `5000`，在 `0x24fe8` 的定长段读到全部已定名字段。实档测试核对 CLI 所用诊断及下游边界，构造测试继续覆盖截断和存盘原值。未在 ArcMap 10.1 或 QGIS 中人工核对运行时显示效果。

## 照明后的非空 IGeometry 槽

用同一 SHA-256 的 Map.dll 10.1.0.3035 复核反编译和原始指令。
Load `0x793fffd0..0x793fffe1` 以持久化接口 `+0x1bc` 为目标，
请求 `0x79483174` 的 `1B07790D-38C5-11D0-92D2-00805F7C28B0`
（IGeometry）。LoadObject 成功后读 `+0x1c4` 的 IBorder。
Save `0x79401850` 的对应段按同序将两个成员交给 SaveObject。

对象槽共用 Map 的 ObjectStream 引用表。新对象调用已有 Point、
Multipoint、Polyline、Polygon 或 Envelope 的读取器，完整消费后读取边框。
支持这些类的已见键回引并保留空槽行为。未知类、截断或几何体未完整
消费时，游标和引用表停在几何槽前。构造用例在参考工程的 `0x24ff4`
几何槽替换 Point、Envelope 和 Point 回引，均走到 Map 末端。
真实参考工程该槽为空，非空宿主、其他几何类及 ArcMap/QGIS 人工检查
仍待取证。

最终验证：`cargo fmt --check` 与完整 `cargo test` 通过；本组介质测试显式运行 3 项通过（含 1 项默认忽略的介质枚举）。`git diff --check` 通过。并行代理仍可能继续修改共享工作区，整合时需复跑。

### GeometryBag 和 MultiPatch 的持久化入口

原生文件为 10.1.0.3035 x86 的 Geometry.dll。重新核对 SHA-256 为 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`。现有导出未覆盖 Geometry.dll，补用安装介质类注册、PE RTTI、虚表和 `objdump -d -Mintel` 原始指令。地址均为静态 VA。

| 类 | 10.1 接口及读写证据 | Rust 读取范围 |
| --- | --- | --- |
| GeometryBag `{10B5F5C0-3781-11D2-BCC5-0000F875BCCE}` | 主 QI `0x7186f650` 支持 IGeometry，IPersistStream 比对 `0x7186f8d7` 后返回 `this+0x38`。对应虚表 `0x71a15fec` 的 Load/Save 为 `0x71873be0/0x71873dc0`。Load 依次读 v0–1、int32 计数、缓存 IGeometry（`0x71873c89`）、各成员 IGeometry（`0x71873d13`）、ISpatialReference（`0x71873cb7`）及一个标志字节（`0x71873d7e`）。Save 表 `0x71a11e94` 为 `[-1,1]`。 | 缓存及成员复用完整几何体分派，允许嵌套 GeometryBag；各槽共享键表。计数、嵌套深度受限，未知新对象使集合整体回退。 |
| MultiPatch `{F3C041C6-AE4D-11D2-9C93-00C04FB17838}` | 主 QI `0x718dd880` 匹配 IGeometry，IPersistStream 比对 `0x718ddaa5` 后返回 `this+0x18`。对应虚表 `0x71a1f490` 的 Load/Save 为 `0x718e4610/0x718e4960`。Load `0x718e4656..0x718e4661` 只接受 v1/v2，随后读 int32 长度及其字节区间；v1 在内存中处理前 12 字节兼容字段，v2 直接把记录交给导入器（`0x718e48ad`），再从原流读取 ISpatialReference（`0x718e48d1`）。Save 表 `0x71a11e5c` 为 `[1,2]`。 | 读取完整长度界定的记录及其后坐标系，保留面片、纹理与压缩记录原始字节。记录的内部解码和导入器语义仍开放。 |

两个类已接入 Map 照明后 IGeometry 槽及共享几何分派。构造测试覆盖递归集合、跨槽回引、两版面片记录之后的坐标系、坏计数、过深嵌套、截断及其后的 Map 边框与根尾。扫描两套介质的 134 个 `/Maps` 和 134 个 `/PageLayout` 流均未命中这两个 CLSID 的字节候选，因此本轮没有为两类取得实档对象佐证。ArcMap 与 QGIS 人工验收未做。

本轮全量测试含忽略项 1043 项通过，格式及差异检查通过。CLI 回归覆盖仓库样例、66 份 10.1 介质工程和 68 份 10.8 补充对照，其地图根尾和布局对象终点均仍可读取。

### MultiPatch 面片记录和压缩封套

对 Geometry.dll 与 SystemCore.dll 10.1.0.3035 原始 PE 补做定点 Ghidra 反编译和 `objdump` 指令复核。Geometry.dll 摘要见上节，SystemCore.dll SHA-256 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。以下地址是静态 VA。

- `0x718e48ad → 0x71906090 → 0x718e0ee0` 将有长度记录交给几何导入器。`0x718e1085..0x718e11eb` 依次读取 XY 包络、面片数和点数、面片首点索引、面片类型字、XY 坐标、Z 范围和逐点 Z。类型 `0x1f/0x20` 在有剩余字节时读取 M 范围和逐点 M（`0x718e17a0..0x718e1822`）。类型字和 M 存盘值保留原样。
- 旧 M 段的导入器从范围两端探测编码标志（`0x718e1835..0x718e188d`）。`0x71887b70` 在 M 位模式的高字检查法线位 `0x00100000`，把两个 9 位分量按常量 `255.0` 还原，再用符号位恢复 Z。越过单位圆时将 XY 归一化，Z 为零。`0x71887d60` 检查纹理位 `0x00200000`，把低字两个 16 位分量按 `分量 / 21000.0 − 1.0` 还原，纹理维数为 2，逐面片起点沿用几何索引。保留原始 M，并按导入器规则对缺少对应标志的顶点补零。
- 通用面片 `0x36` 的后续数组由各自的 LONG 数量控制（`0x718e120c..0x718e13ec`）。顺序为 M、点 ID、float32×3 法线、纹理坐标、材质。非空纹理还读维数和逐面片纹理起点。非空材质在 v2 包含版本、`数量+1` 个相对偏移及最后一个偏移所界定的原始数据。v1 转换 `0x718fd9e0` 从 `0x718feb50` 计算的偏移表位置回退四字节，再重建表和写入版本，证明旧记录该位置直接是偏移表。实现据此对两版材质分别保留精确的字节范围。
- `0x718e46e5..0x718e4770` 处理 v1 的旧 `0x38` 或「压缩长度等于记录剩余长度」封套，`0x718e0fb1..0x718e103e` 处理导入器的低字节 `0x38` 或压缩位 `0x00800000` 封套。两处的字段顺序为类型字、解压长度、压缩长度、压缩字节。Geometry.dll 构造段 `0x718fb027..0x718fb036 → 0x718fc3f0` 创建 SystemCore 的 Compression 类 `{523C0660-35CC-40FF-B0DF-AA400F598752}`，请求 `{FEE4D81C-25D9-4389-A20C-16821EC90719}`。SystemCore QI `0x7ffba220` 返回 `this+0x10`，该视图的 `+0x18` 为 `0x7ffba3f0`，在 `0x7ffba418` 调用 zlib `uncompress`。Rust 使用仓库已有的 flate2，并与合并栅格读取复用精确长度校验。

结果同时保留原始记录和结构化面片。已确认记录中的负数、溢出、超出区间的数组、非法面片索引及压缩长度不符会报错并回退整个对象。未确认的形态仍保留原始字节，沿声明边界接续坐标系。

构造用例覆盖 v1/v2 的旧与通用记录、压缩封套、两版材质表、旧 M 法线和纹理、空间参考回引、坏索引、截断、压缩损坏和超限回退。两套介质前轮未找到 MultiPatch 持久化对象，因此新增字段由静态证据和构造测试覆盖，ArcMap/QGIS 人工验收未做。

#### 旧面片标识和材质命令

对同一份 Geometry.dll 10.1.0.3035 x86 补做 Ghidra 定点反编译，并用 PE 指令和跳转表复核。

- `0x718fa360` 的 `0x718fa4d0/0x718fa4c0` 分派表使 `0x29/0x2a` 经过兼容警告 `0x718fa3ee` 后进入 `0x718fa41b`。导入器 `0x718e0ee0` 保留原类型字，两种记录共享通用面片的包络、索引、XY/Z 和计数控管的后续数组。`0x718e11fa..0x718e120c` 仅把 `0x1f/0x20` 导向旧 M 分支。`0x718fd9e0` 的 v1 旧材质转换由 `0x718fdd10` 的标志控制，该标志仅为低字节 `0x36` 置位，所以 `0x29/0x2a` 在 v1 持久化对象中仍读材质版本字。
- 带版本的材质命令由 `0x718fed40` 和 `0x718fedb4` 分派表划界。`IGeneralMultiPatchInfo` 虚表 `0x71a1f914` 的 `+0x50/+0x54/+0x58` 分别进入 `0x718e85e0`、`0x718e8640`、`0x718e86b0`，确认命令 `1` 是三个 RGB 字节，`3` 是一个透明度百分比字节，`6` 以操作码存在表示背面剔除。命令 `5` 的有符号 LONG 是提供纹理的材质索引，`0x718feaa7..0x718feab5` 据此在目标材质的区间查找命令 `2`，`0x718ebca0..0x718ebcbc` 在合并材质时重定位该索引。
- 带版本的内嵌纹理命令 `2` 在操作码后依次存一字节每像素字节数、两个有符号 short 宽高、有符号 LONG 载荷长度、DWORD 编码号与载荷。`0x718fe020` 的纹理转换及 `0x718fea30` 的读取入口共同确认其布局。旧转换 `0x718fd9e0` 与长度表 `0x718fdccc` 确认，无版本的旧命令 `2` 以宽 × 高 × 每像素字节数定长，旧命令 `7` 另存 LONG 长度。转换器给这两种旧纹理分别补编码号 `1`、`3`，并在 `0x718fdc81..0x718fdc90` 把旧命令 `8` 改写为 `5`，保留其材质索引。
- `_3DSymbolsLib.dll` 10.1.0.3035 x86（SHA-256 `9aa0b75fe16ea2ad4b01b924616bbc2c0737890421a7e0bb733b53e906344566`）的命名导出将另外三个操作码对位到属性。下列 getter 均调用 `MultiPatchBuffMgr::FindMaterialType 0x68192690`，写侧对应 `MultiPatchBuffMgr::Init 0x68191670`。

| 操作码 | 命名 getter 与字段 |
| --- | --- |
| 4 | `GetMaterialShininess 0x68192090` 取一个无符号字节。写点 `0x68191aa5`。 |
| 9 | `GetMaterialEdgeColor 0x68192130 / 0x68192170` 取三个 RGB 字节，`GetMaterialEdgeDisplay 0x68192110` 按该命令是否存在返回布尔值。写点 `0x68191ad4`。 |
| 10 | `GetMaterialEdgeWidth 0x68192260` 取一个无符号字节，缺省返回 1。写点 `0x68191ae0`。 |

旧命令 `3/4` 的长度表与通用查找器均给出总长两字节，旧转换器的复制分支 `0x718fdbf3..0x718fdc06` 却前进三字节。受控执行结果见下节。

`MultiPatchMaterials10_1::commands` 逐个解析索引表划出的材质区间，未知命令和截断会返回带相对偏移的错误，原始载荷以及后续材质均可继续读取。内嵌纹理保留已划界的图像字节。构造测试从几何对象入口覆盖两个旧标识、两类材质表、纹理索引及损坏命令后的独立材质区间。ArcMap/QGIS 人工验收未做。

#### 旧材质转换器的宽度复核

将上述 Geometry.dll 10.1.0.3035 的原始指令及跳转表按静态地址装入一个 32 位测试程序，在 QEMU i386 中运行 `0x718fd9e0`、其元数据初始化与偏移提取函数，以及旧纹理查找器 `0x718fec60`。测试用固定缓冲区和逐字节复制替代分配器及 memcpy。转换输入为可控的 `0x36` 三角面片记录，材质数量及偏移表由原生前置函数读取。

| 受控输入（十六进制） | 两种操作码的观测结果 |
| --- | --- |
| 同一材质内 `03 28 06` 或 `04 09 06` | 预扫描按二字节到达 `06`。复制阶段用同一次复制带走 `06`，写出版本 3，偏移表为 `[0,3]`。 |
| 前一材质只有 `03 28` 或 `04 09`，后一材质为 `06` | 输入偏移表 `[0,2,3]`。输出为 `[0,3,4]`，`06` 同时出现在第一段末尾和第二段开头。 |
| 同一材质内上述两字节命令之后为 `01 0b 16 21` | 复制阶段把 `01` 一并取走，随后从 `0b` 分派并提早返回，最终偏移保持为 0。 |
| 直接查找跟在上述两字节命令后的旧纹理命令 `02` | `0x718fec60` 对两种操作码均返回材质起点加 2。 |

因此磁盘读取按命令总长两字节和材质偏移表划界。Rust 回归覆盖同一材质内后继颜色命令与跨材质边界。该实验覆盖指定原生函数的指令行为，ArcMap 打开实际旧工程的效果尚待验收。

### 10.1 的 IGeometry 类体清点

沿同套 10.1 Desktop 介质 72 个 MSFT 类型库的 RefTab 和跨库接口继承清点，从 Map 请求的 `IGeometry` IID 共找到 19 个公开 coclass。其中 Point、Multipoint、Polyline、Polygon、Envelope、GeometryBag 和 MultiPatch 的二进制持久化体已接入。其余类按 PE 中的主 QueryInterface、直接比较的 IID 和委派进行核对。目标持久化接口为 IPersistStream `{00000109-0000-0000-C000-000000000046}` 和 IPersistVariant `{67A4D651-69E0-11D2-8500-0000F875B9C6}`。

| 类 | 10.1 PE 主 QueryInterface |
| --- | --- |
| Path、Ring | Geometry.dll `0x71908a30`、`0x7195e480` |
| Line、BezierCurve、EllipticArc | Geometry.dll `0x718d4fc0`、`0x717feb60`、`0x7184cc90` |
| CircularArc | Geometry.dll `0x717ea180`，原生类名为 `ESRI::Arc` |
| Sphere、Ray | Geometry.dll `0x7198ada0`、`0x7194bdc0` |
| TriangleFan、TriangleStrip、Triangles | Geometry.dll `0x7198e270`、`0x7198dcf0`、`0x7198e7f0` |
| MapPoint | Mensuration.dll `0x7a19ad20`，向 `0x7a1bc600` 委派 |

Geometry.dll 的这些 QI 路径都没有返回上述两个持久化接口。共享回退 `0x719f1450` 只比较 IUnknown 及两个内部接口。CircularArc 由表首 `0x71a65b60` 的 CLSID `0x71a125e8` 进入类工厂虚表 `0x71a1d850`，创建路径 `0x718c0b70 → 0x717f4960`，对应 `ESRI::Arc` 的 RTTI 和主表 `0x71a1211c`。旧记录按 `CircularArc` 寻找 RTTI 未命中，现已通过工厂和类标识对位。Mensuration.dll 为 x86 10.1.0.3035，SHA-256 `c7d1ad745174d43c01463950f9e2337cc7adbb5d8d8b0722c905184f3cc00f1a`。MapPoint 的主 QI 直接受理其公开接口、有条件的 IPoint 和 IUnknown，委派只受理 IUnknown 和两个内部接口，也没有二进制持久化入口。

在 10.1 标准 ObjectStream 中，两种持久化接口均不可用时，保存侧写空 GUID，见 [SystemCore 写侧](mxd-10.1-geodata-xform.md#coordinatexform-操作集)。因此这些类不增加 Map 的独立二进制对象体读取分支。外部组件或新样本里的未知非空类仍保留在对象头。本节为 10.1 安装介质公开类的静态清点，未做目标软件运行取证。
