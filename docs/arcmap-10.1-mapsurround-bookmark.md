# ArcMap 10.1 MapSurround 与空间书签非空循环元素体

记录日期：2026-11-14。对应 TODO「MXD 解析」节第 76 行开放项中的
「MapSurround/书签非空循环（样本计数 0，无字节佐证）」。全部为静态反编译
观察，未运行 ArcMap，未做任何人工目视验收。

**样本边界（先声明）**：仓库参考样本 `samples/测试底图.mxd` 的
MapSurround 循环计数（`/Maps:0x24f60`）与书签循环计数（`0x24f9c`）均为
0。本文全部结论**无样本字节佐证**；实现（接线轮后位于 `src/maps/map_surround.rs`）只能用
合成输入测试，不得声称样本验证。样本中能核对的只有循环结构本身（计数 0
即循环体为空），该部分与 `map_object::read_map_suffix_10_1` 现有骨架一致。

## 方法与文件身份

- Ghidra 桥接工具 MCP（Ghidra 12.2 提供者，用户级配置 JDK 25，本轮未发生 JAVA_HOME
  错误）。共享服务器与其他 agent 并发使用，多次出现打开目标被切走的情况，
  均以「open + 操作同批」重试恢复；结束已调用 `close_binary`。
- 本轮从 `/home/gxrs/下载/arcmap10.1/ar10.1/Desktop/Setup1.cab` 条目
  `Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 新提取至
  `temp/agent-map-surround/`：1242112 字节，SHA-256
  `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，
  pefile 复核 PE32 x86、ImageBase `0x793a0000`、VS_FIXEDFILEINFO/
  StringFileInfo `10.1.0.3035`（`FileDescription: Map DLL`、
  `CompanyName: Esri`），与既有研究轮记录一致。
- AfCore.dll 复用 `temp/arcgis101-bin/` 基线副本（SHA-256
  `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，
  10.1.0.3035）。地址均为 PE 静态映像 VA。
- 类名/模块：`esri.clsid.xml`（Setup1.cab `esri.clsid.ecfg` 解包，
  10675 条）；接口名：10.1 类型库 wine `LoadTypeLibEx` dump
  （`/tmp/mxd-arcgis101-guiddocs/wine/`，第五轮既有产物，本轮仅检索）。
- 辅助脚本（RTTI 遍历与虚表槽分类）在 `temp/agent-map-surround/`，
  以 `python3 -I` 运行、路径作参数传入。

## 一、循环结构确认（证据：确定的）

Map::Load `0x793ff4d0`（伪代码逐行复核，`/tmp/ms_map_load.c`）：

- **第 5 步（IMapSurround）**：`AfStreamRead(4)` 读 int32 计数 →
  `0 < count` 时循环，每元素**普通 `LoadObject`**（请求 IID
  `DAT_79481a84` = `{FE3115E9-ED6C-11D0-8683-0000F8751720}` IMapSurround），
  即 20 字节对象头（16 字节类标识 + 4 字节键）+ 对象体，**无 ExtraSafe
  `[A][B]` 8 字节前缀**；加载成功后经 `(this−0x40)` 子接口虚表 `+0x84`
  加入集合。
- **第 9 步（ISpatialBookmark）**：int32 计数 → 循环普通 `LoadObject`
  （请求 IID `DAT_794815a0` =
  `{EE53528B-41C9-11D1-880A-080009EC732A}` ISpatialBookmark）；每元素先
  QI（出参经 `extraout_ECX`，IID 为 `DAT_79488d18` =
  `{3036D35F-EDE5-11D0-87FE-080009EC732A}` IFeatureBookmark，第五轮已
  闭合）再 `FUN_7941c760` 入集。

Map::Save `0x79401850`（本轮反编译复核，与第四轮记录一致）：

- MapSurround 写侧：`AfStreamWrite(int32 *(this+0x130) 计数)` + 循环
  `SaveObject(*(*(this+0x12c) + i*4))`；
- 书签写侧：`AfStreamWrite(int32 *(this+0x174) 计数)` + 循环
  `SaveObject(*(*(this+0x170) + i*4))`；
- 均为普通 `SaveObject`，无 ExtraSafeWriteObject 前缀，与读侧对称。

**与 `map_object.rs` 的衔接点**（记录，未改该文件）：
`MapSuffixStop::MapSurroundBody { index, offset, class_id, key }` 与
`MapSuffixStop::BookmarkBody { … }` 的 `class_id` 即上述 `LoadObject` 的
流内类标识，按 CLSID 分派：书签循环 → `read_aoi_bookmark_10_1` /
`read_feature_bookmark_10_1`（整体消费）；MapSurround 循环 →
`read_legend_prefix_10_1`（前缀停止语义，与骨架其余停止点一致）。骨架的
「计数拒绝负数、计数 × 最小条目字节数预检」与本轮原生观察（原生负数跳过
循环）相容（骨架更严，已在骨架文档注明）。

## 二、coclass 清单（请求 IID 的实现类；证据强度逐行标注）

`FE3115E9`（IMapSurround）/`EE53528B`（ISpatialBookmark）实现类不在已
扫描 8 个 DLL 的数据段（第四轮结论）；本轮经 esri.clsid 全表 + Map.dll
RTTI/GetClassID/Load 槽取证得出：**两循环全部常见实现类都在 Map.dll**
（MapSurroundFrame 在 MapElements.dll，但它是宿主 MapSurround 的页面
元素，不是本循环成员）。

| 类名 | CLSID | 模块（ecfg） | 证据 | 强度 |
| --- | --- | --- | --- | --- |
| Legend | `7A3F91E4-B9E3-11D1-8756-0000F8751720` | Map.dll | RTTI `.?AVLegend@@`（TD `0x794b3d30`）；GetClassID `0x793d1330` 写出（`DAT_79485118`）；IPersistStream 虚表 `0x794859c0` | 确定的 |
| Scalebar | `7A3F91DB-B9E3-11D1-8756-0000F8751720` | Map.dll | RTTI `.?AVScalebar@@`（TD `0x794b4c7c`）；Load 槽 `0x79446040` 调共同前缀 | 确定的 |
| ScaleText | `7A3F91DC-B9E3-11D1-8756-0000F8751720` | Map.dll | RTTI `.?AVScaleText@@`（TD `0x794b4da0`） | 确定的（体未反编译） |
| MarkerNorthArrow | `7A3F91DD-B9E3-11D1-8756-0000F8751720` | Map.dll | RTTI `.?AVMarkerNorthArrow@@`；Load 槽 `0x79444b40`（读流特征命中） | 确定的（体未反编译） |
| MapTitle | `7A3F91DF-B9E3-11D1-8756-0000F8751720` | Map.dll | RTTI `.?AVMapTitle@@`；Load 槽 `0x794426b0`（读流特征命中） | 确定的（体未反编译） |
| LegendFormat | `7A3F91E5-B9E3-11D1-8756-0000F8751720` | Map.dll | RTTI `.?AVLegendFormat@@`（TD `0x794b3db0`） | 确定的（体未反编译；Legend 体内嵌套） |
| HollowScaleBar | `6589F143-F7F7-11D2-B872-00600802E603` | Map.dll | ecfg；Scalebar 族子类 | 候选的 |
| SingleDivisionScaleBar | `6589F146-F7F7-11D2-B872-00600802E603` | Map.dll | 同上 | 候选的 |
| AlternatingScaleBar | `6589F147-F7F7-11D2-B872-00600802E603` | Map.dll | 同上 | 候选的 |
| DoubleAlternatingScaleBar | `6589F148-F7F7-11D2-B872-00600802E603` | Map.dll | 同上 | 候选的 |
| AOIBookmark | `BBB1AE73-41E3-11D2-AE1E-080009EC732A` | Map.dll | RTTI `.?AVAOIBookmark@@`；GetClassID `0x793a2600` 写出（`DAT_79481590`）；Load/Save 反编译 | 确定的 |
| FeatureBookmark | `EC65B35B-4342-11D2-AE22-080009EC732A` | Map.dll | RTTI `.?AVFeatureBookmark@@`；GetClassID `0x793b5520` 写出（`DAT_79482c18`）；Load/Save 反编译 | 确定的 |
| MapSurroundFrame | `83FFCAE1-EDCA-11D0-8683-0000F8751720` | MapElements.dll | ecfg（布局侧框架元素，非本循环成员） | 候选的 |

类型库核对（esriCarto）：`ILegendFormat {52AA5BC1-CB21-11D0-87F9-080009EC732A}`、
`ILegendItem {52AA5BC0-CB21-11D0-87F9-080009EC732A}`、`IMap {E6BDAA75-…}`、
IMapSurround/ISpatialBookmark 同第五轮；`esriArcGISVersion` 枚举
83=0/90=1/92=2/93=3/10=4/101=5（Current=5）。

## 三、MapSurround 族共同前缀（证据：确定的）

装饰物族各类的 Load 先调共享前缀读取 `FUN_794413d0`（该函数共 9 个直接
调用者：`0x793a3bda`、`0x793c89d3`、`0x793da2f2`（Legend）、`0x7943b333`、
`0x7944272a`、`0x794460fd`（Scalebar）、`0x7944854a`、`0x79451171`、
`0x79457469`——族内各类再读各自尾段），经 MapSurround 基类 IPersistStream
虚表 `0x7948a510`（COL disp 0xdc）槽 `0x79442380`（Load thunk）/
`0x79442300`（Save thunk）到达。

Load `FUN_794413d0` 字段序（汇编复核 `0x7944147a`/`0x794414b5`）：

| # | 字段 | 门 | 备注 |
| --- | --- | --- | --- |
| 1 | int16 版本 | 无上限拒绝（原生） | 门为有符号 `≥2`/`≥3`；负值位模式原生通过 |
| 2 | String（名称 → this−0xa0） | 恒有 | AfCore String::Read |
| 3 | LoadObject（IMap `{E6BDAA75-…}`） | 恒有 | 关联地图；真实流中为对 Map 的键回引头（20 字节，无体）；空标记亦合法 |
| 4 | FixedAspectRatio（int16，this−0x8c） | 版本 ≥2 | 0 为假，非零为真；保留原始位模式 |
| 5 | WKSEnvelope（32 字节） | 版本 ≥3 | Load 读入栈后丢弃；Save 写 `this−0x84`，该包络由 FitToBounds 更新 |

Save `FUN_79441510` 对称：`AfStreamGetSaveVersion(docVer, 5, DAT_79480d58)`
→ String::Write → SaveObject(IMap) → int16（**恒写**，无版本门）→
[版本 >2] 32 字节。

**`AfStreamGetSaveVersion(docVer, N, table)` 语义本轮闭合**（AfCore
`0x68a847f0` 反编译）：`table == null || N == 0` 返回 −1；`docVer ≥ N`
返回 `table[N−1]`，否则返回 `table[docVer]`。即第二参数是表项数。

- 共同前缀版本表 `DAT_79480d58` = `[2,2,2,2,3]`（N=5）→ 10.1（docVer=5）
  恒写**版本 3**（含第 4/5 字段）；docVer 83..10 写 2（无 32 字节段）。
- Legend 版本表 `DAT_79480dc4` = `[3,4,4,4,4,5]`（N=6）→ 10.1 恒写 **5**。

### 固定纵横比（2026-11-18）

Map.dll 为 PE x86、文件版本 10.1.0.3035，SHA-256 为
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。
复用研究目录的 `Map/7943fd10.c`、`Map/794413d0.c`、`Map/79441510.c`、
`Map/79441940.c`、`Map/79441970.c` 与 `Map/79441130.c`，以 `objdump -d -Mintel`
复核以下静态地址：

- 介质 `esriCarto` 10.1 类型库（SHA-256
  `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）的
  TypeInfo 162 对应 `IBoundsProperties {7AD72C5D-EB8A-11D3-9314-00600802E603}`。
  直接读取 MSFT 类型库的名称表和成员记录（`memoffset=0xf0d38`）确认 x86
  `+0x0c/+0x10` 分别是 `FixedAspectRatio` 的 get/put；第三槽 `+0x14` 是
  `FixedSize` 的 get，另外一个属性。
- 构造器 `0x7943fd96..0x7943fdb2` 在完整对象 `+0x24` 安装
  `IBoundsProperties` 虚表 `0x7948a49c`（RTTI 同一位移），并在
  `0x7943fe1a` 初始化完整对象的 `+0x50` 为 0。该虚表的属性设置器
  `0x79441970` 写 `(接口 this + 0x2c)`，即完整对象 `+0x50`。取值器
  `0x79441940` 转发到完整对象的内部虚表 `+0x48`，基类实现
  `0x793a1170` 取同一 short。持久化视图位于 `+0xdc`，其读写的
  `this−0x8c` 同样落在 `+0x50`。
- TypeInfo 178 把 `IMapSurround +0x20` 定名为 `QueryBounds`；该槽
  `0x79441130` 从接口视图 `+0x2c` 检查同一 short。`0x794411ce..0x794411d3`
  在非零时使用当前显示变换提供的包络路径，跳过普通边界重新计算。这是一个
  以零/非零区分的标志，禁止只识别 `0xffff` 或把它误作 FixedSize。

Rust 保留原始位模式，`fixed_aspect_ratio()` 在版本 0/1 返回 `None`，
版本 2/3 根据已存值取布尔值；此接口仅报告持久化属性，不重做 ArcMap 布局。
合成测试覆盖字段缺省、0 与非零。本轮未运行 ArcMap 或 QGIS。

## 四、AOIBookmark（空间书签，体完全闭合；证据：确定的）

虚表 `0x79481550`（COL `0x7948ffbc`，adjustor 0x14），GetClassID
`0x793a2600` 写出 `{BBB1AE73-…}`。

- Load `0x793a2cd0`：① int16 版本（`lVar4 < 0 || 1 < (short)v` 拒绝
  `0x80004005`，即有符号 0..=1，**负值原生即拒绝**）→ ② `LoadObject`
  （请求 IID `DAT_79481580` = `{1B077914-38C5-11D0-92D2-00805F7C28B0}`
  IEnvelope，位置包络）→ ③ `String::Read`（名称 → this+0x08）。到体尾。
- Save `0x793a2d60`：恒写版本 short 1（`DAT_79480eac` = `01 00`）→
  `SaveObject(this+4)` → `String::Write`。读写对称，体至此闭合。

## 五、FeatureBookmark（要素书签，体完全闭合；证据：确定的）

虚表 `0x79482be8`（COL `0x79490a88`，adjustor 0x14），GetClassID
`0x793b5520` 写出 `{EC65B35B-…}`。

- Load `0x793b5c50`：① int16 版本（有符号 0..=1）→ ② int32（→ this+0x04，
  语义未定）→ ③ `LoadObject`（请求 IID `DAT_79482c08` =
  `{4BA33331-D55F-11D1-8882-0000F877762D}` IName，名称对象；装载后对其
  虚表 `+0x14` 直调并经 `FUN_793acdc0` 接线到 this+0x08）→ ④
  `String::Read`（名称 → this+0x0c）。到体尾。
- Save `0x793b5d70`：恒写版本 short 1（`DAT_79480e94` = `01 00`）→
  int32 → `SaveObject`（经 `FUN_79466510` 取名称对象）→ `String::Write`。

## 六、Legend（图例，已确认前缀；证据：确定的）

IPersistStream 虚表 `0x794859c0`（COL `0x79493048`，**this 位移 0x1f4**；
注：位移 >0x100 的 COL 在早期脚本的过滤范围之外，本轮经 Load thunk
`0x793ded80` 反查补齐）。GetClassID `0x793dede0` → `0x793d1330` 写出
`{7A3F91E4-…}`；Load 槽 `0x793ded80` → `FUN_793da250`；Save 槽
`0x793df010` → `FUN_793da650`。

Load `FUN_793da250` 字段序（汇编逐门复核）：

| # | 字段 | 门 | 汇编/备注 |
| --- | --- | --- | --- |
| 1 | int16 Legend 版本 | `CMP word,5; JLE` 通过，>5 拒绝 `0x80004005`（有符号；负值原生通过） | `0x793da2ca` |
| 2 | 共同前缀（第三节 5 字段，`FUN_794413d0(this−0x118)`） | 恒有 | |
| 3 | LoadObject（ILegendFormat `{52AA5BC1-…}` → this−0xfc） | 恒有 | `DAT_79485a28` |
| 4 | String（this−0xf8，标题候选） | 恒有 | |
| 5 | double ×2（读后丢弃） | **版本 ==1** | `0x793da376`；写侧无对应（旧版遗留） |
| 6 | int16 ×3（−0xf0/−0xee/−0xec） | 恒有 | |
| 7 | int16（传入集合接口 +0xa8） | 恒有 | 写侧恒写 `0xFFFF` |
| 8 | int32 计数 + 循环 LoadObject（ILegendItem `{52AA5BC0-…}`） | 恒有 | 每项经集合接口 `+0xa4` 入集 |
| 9 | int16（−0xe8） | 版本 ≥3 | `0x794414e8` 同型门 `0x793da4e8` |
| 10 | int16（−0xe6） | 版本 ≥4 | `0x793da51b` |
| 11 | int16 ×3 + double（−0x9c..−0x94） | 版本 ≥5 | `0x793da545`；<5 原生写默认（含 4.0）不读流 |

Save `FUN_793da650` 对称（含恒写 `0xFFFF` short、int32 计数 + 循环
SaveObject 条目、恒写 −0xe8、docVersion≠0 写 −0xe6、docVersion≥5 写尾组；
汇编 `0x793da7f3`/`0x793da810`）。10.1 写侧版本 5 + docVersion 5 →
Load ver=5 读全部尾段，读写一致（ver/docVersion 桥接经版本表闭合，无
错位）。

## 七、实现（src/maps/map_surround.rs，仅合成输入测试）

- `read_map_surround_common_prefix_10_1`：共同前缀（版本仅接受 0..=3，
  较原生更严并注明；IMap 槽新对象报不支持——真实流恒为回引）。
- `read_aoi_bookmark_10_1` / `read_feature_bookmark_10_1`：两书签类完整
  体（嵌套 Envelope 新对象由 `envelope` 读取器整体消费；IName 新对象体
  不支持）。
- `read_legend_prefix_10_1`：Legend 已确认前缀（ILegendFormat 槽与
  ILegendItem 条目新对象体处停止并保留剩余，停止是成功返回；版本门
  0..=5，负值拒绝较原生更严）。
- 事务性：trial 游标 + trial 登记表，成功（含停止）才提交；20 项单元
  测试（`tests/map_surround.rs`）覆盖版本边界（1/5/6/0x8000）、门字段、
  空/回引/新对象三分支、逐字节截断、失败游标不变、登记表事务性、计数
  负数与限额、字符串限额、重读一致。未接入 `parse`/`map_object`（任务
  约束不改共享文件）。

## 八、开放项

1. **无样本字节佐证**（全篇结论的边界）：需含书签/装饰物的 MXD 样本或
   目标软件运行观察才能升级；两循环元素在真实流中的类分布未观察。
2. LegendFormat 体与 ILegendItem 实现类（HorizontalBarLegendItem 等）未
   反编译：Legend 读取器在相应新对象处停止；越过该点需这些类闭合。
3. Scalebar/ScaleText/MarkerNorthArrow/MapTitle 及四个 Scalebar 子类的
   尾段未逐类反编译（共同前缀已由调用者集合证实为其 Load 首段）；本轮
   仅列清单。
4. 本轮未定性的共同前缀字段已由上节与[拟合包络复核](mxd-map-tail-cross-stream-20261118.md#书签与装饰物)定名；Legend 自身标量需按类体核对。
5. FeatureBookmark 名称对象（IName）的实现类分布未观察；其 `+0x14`
   直调与接线（`FUN_793acdc0`）的语义未展开。
6. 共同前缀原生无版本上限、Legend 原生接受负版本（有符号 JLE）：读取器
   的更严拒绝（>3 / >5 / 负值）是实现约束而非原生规则。
7. Map::Load 第 5 步每元素入集用的 `(this−0x40) +0x84` 槽与第 9 步
   `FUN_7941c760` 的集合语义未逐项定性（不影响流内布局）。
8. 全部结论为静态观察；ArcMap 10.1 与 QGIS 的人工打开与目视核对均未
   进行，不声称已验证。

## 九、接线补记（2026-11-15）

第七节「未接入 `parse`/`map_object`」已作废：第七轮接线已把本模块接入
`map_object::read_map_suffix_10_1` 的 MapSurround/书签两循环——循环条目按
流内类标识分派（AOI/Feature 书签体经本模块读取器整体解析，Legend 走
`read_legend_prefix_10_1`）；同日接线轮再在 Legend 条目循环按 CLSID 分派
HorizontalLegendItem 体（`legend_format` 读取器，新增
`LegendItemBodyIncomplete` 停止变体透传嵌套停止）。样本两循环计数均为 0，
样本层面仍无字节佐证；开放项 2/3/5 状态不变（开放项 5 的 IName 槽已列入
TODO 体未闭合类族清单）。模块测试现为 22 项（接线新增 2 项）。

## Legend 主体 v0/v1 原生读取

2026-11-19 核对研究目录中 Map.dll 10.1.0.3035（PE32，SHA-256
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`）。
按原址映射 Legend Load `0x793da250`、MapSurround 共同前缀
`0x794413d0`、子对象指针清理助手，以及 AfCore 10.1 的
`String::Read` 和 `AfStreamRead`，在 QEMU i386 中运行。IStream、
LoadObject、内存分配、图例集合和地图关联虚表由测试提供，状态守卫
`0x793d1470 / 0x793d14e0` 及读取完成后的布局更新 `0x793d3a70`
以不读流的替身返回。这覆盖持久化字段的消费和条目派发，
图例布局刷新未纳入试验。

构造两个主体，均含 MapSurround v3（名称、空 IMap、FixedAspectRatio、
32 字节包络）、空 ILegendFormat、标题 `图例`、四个可区分的 short、
计数 1 和一个 ILegendItem 回引。v1 在标题后写入 `-9.5` 和
`123.25`，原生从 `0x793da38b`、`0x793da3a7` 读取到同一临时位置。
共同前缀和图例在两个版本中都调用设定关联地图、清空条目和追加一个
条目的虚表，各消费 120、136 字节。哨兵仍在对象结束位置，
v5 才写入流的尾字段由原生在这两个版本中设为 `0 / 0xffff / 0xffff / 4.0`。

Rust 使用相同对象体与已登记的回引键，复核标题、两个 double 的版本门、
四个 short、回引起点以及结尾。两个主体共 256 个截断前缀均报错，
游标与键表保持不动。测试文件在忽略目录 `temp/mxd-legend-v0/`，
两条带长度记录的原始输出 SHA-256 为
`fd29a7fce5772897e7058e3a9e50810f8ad2aff33cae844d1d5d9e80e66649b9`。
尚未找到真实旧 MXD 的 Legend 主体 v0/v1，也未进行 ArcMap/QGIS 人工验收。
