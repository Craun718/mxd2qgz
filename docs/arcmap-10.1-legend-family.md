# ArcMap 10.1 Legend/MapSurround 族开放对象体：LegendFormat、Scalebar 族、ScaleText、MarkerNorthArrow、MapTitle、LinePatch/AreaPatch、LegendItem 其余实现类与 FeatureBookmark IName 槽

记录日期：2026-11-16。本轮闭合 TODO「MXD 解析」节「符号与 Legend 族」开放项中的
LegendFormat 体、ILegendItem 其余实现类体（HorizontalBarLegendItem/
NestedLegendItem/VerticalLegendItem）、Scalebar/ScaleText/MarkerNorthArrow/
MapTitle 体、四个 Scalebar 子类尾段、ILinePatch/IAreaPatch 实现类体
（LinePatch/AreaPatch），并对 FeatureBookmark 的 IName 槽接线机制补全（分布
仍未观察）。全部为静态反编译观察（本机 pyghidra，Ghidra 12.2 + JDK 25），
未运行 ArcMap 10.1，未做人工目视验收。实现见 `src/layers/patch.rs`、
`src/layers/legend_item_family.rs`、`src/maps/map_surround_family.rs`（模块
文档含同源字段表）。地址均为对应 PE 10.1.0.3035 静态映像 VA。

## 样本边界（先声明）

- 仓库参考样本 `samples/测试底图.mxd` 的 MapSurround 循环计数为 0
  （`/Maps:0x24f60`），本轮全部类的 CLSID 在全流（全部导出流）16 字节原样
  扫描 0 命中，全部布局结论**无样本字节佐证**，仅有反编译一手证据；实现
  只能用合成输入测试，不得声称样本验证。
- 共同前缀与两循环的既有结论（`docs/arcmap-10.1-mapsurround-bookmark.md`）
  本轮反编译复核一致，未发现该文档「字段序」层面的错误；其「定位链」层面
  的三处笔误见文末「对既有记录的修正」。

## 方法与文件身份

- 本机 pyghidra（`GHIDRA_INSTALL_DIR=/opt/ghidra` +
  `JAVA_HOME=/usr/lib/jvm/java-25-openjdk` + temp/pyghidra-venv，工程目录
  `/tmp/gh-c-legend/`，不与其他代理共享）：Map.dll 复用既有工程副本（拷贝
  自 `/tmp/r3/ghproj_map`，开工核对 SHA）、Layer.dll 复用
  `temp/agent-element-coll/ghproj` 副本、Renderers.dll 本轮新建工程并全量
  分析。辅助脚本（RTTI 定位 `rtti_locate.py`、GUID 读取 `read_guids.py`、
  分批反编译 `decompile_c1..c5.py`/`renderers_*.py`）在 worktree
  `temp/agent-c-legend/scripts/`，产物 `out/`。
- Renderers.dll 本轮从安装介质 `/home/gxrs/下载/arcmap10.1/ar10.1/Desktop/
  Setup1.cab` 条目 `Renderers.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`
  提取（cabextract，257024 字节，SHA-256
  `399c80c99a6c61de120c21b518d9b29069f2c78b20a705c07178aa3e2b656274`，与
  legend-tails 轮记录一致）；pefile 复核 PE32 x86（Machine 0x14c）、
  ImageBase `0x7cac0000`、VS_FIXEDFILEINFO 10.1.0.3035。
- 既有文件开工核对 SHA-256：Map.dll `1c0675cc…b1bb8`（temp/arcgis101-bin/
  mapdll/）、FeatureLayer.dll `2baf8706…98bb60`、Layer.dll
  `fe52bd99…9efed8`（temp/agent-fl-tail/）、LayerLib.dll `55514f05…
  30e02`（temp/agent-fl-tail/，本轮新增记录）。
- 类名/模块：`esri.clsid.ecfg`（`/tmp/r3/ecfg/x/esri.clsid.xml`，本轮以
  脚本按 `<Module>` 包裹重解析——注意 `/tmp/r3/clsid_table_101.json` 仅含
  5 个模块 499 条，是早期部分提取，不能作为全表使用）；接口 GUID 名称：
  10.1 类型库 wine `LoadTypeLibEx` dump（`/tmp/mxd-arcgis101-guiddocs/wine/
  dump2_*.txt`，本轮仅检索）。

## 对既有记录的修正（写本轮文档，不改旧文档）

`docs/arcmap-10.1-mapsurround-bookmark.md` 第二/三节的「定位链」三处笔误
（第五/六轮沿用），本轮反编译逐一证伪：

1. 「Scalebar … Load 槽 `0x79446040` 调共同前缀」——`0x79446040` 实为
   Scalebar identity 虚表 +0x14 槽的 **GetClassID**（反编译：写出
   `DAT_794850c8/794850d0` = `{7A3F91DB-…}`）；共同前缀调用点
   `0x794460fd` 在 Scalebar 真实 Load 实体 `0x794460b0` 内（Scalebar 的
   IPersistStream 虚表在其位移 0x1b4 的 COL 上，虚表 `0x7948b7ac`，Load
   thunk `0x79446ab0` → `0x794460b0`、Save thunk `0x79446a50` →
   `0x79446550`）。
2. 「MarkerNorthArrow … Load 槽 `0x79444b40`（读流特征命中）」——
   `0x79444b40` 同为 identity 虚表 +0x14 的 GetClassID；真实 Load 为
   `0x79444b80`（IPS 虚表 `0x7948b524` 位移 0x13c，Load thunk
   `0x79445200` → `0x79444b80`、Save thunk `0x79445170` → `0x79444ca0`），
   其中确有读流（`0x79444b40` 与 `0x79444b80` 相邻 0x40 字节，旧记录的
   「读流特征」扫描命中了邻近函数）。
3. 「MapTitle … Load 槽 `0x794426b0`（读流特征命中）」——同为 GetClassID；
   真实 Load `0x794426f0`（IPS 虚表 `0x7948a76c` 位移 0xdc）。

共性结论（证据：确定的）：装饰物族各 coclass 的 identity 虚表槽序 =
QI(+0)/AddRef(+4)/Release(+8)/GetRuntimeClassName(+0xc，返回 `L"类名"`)/
析构(+0x10)/**GetClassID(+0x14)**/空实现(+0x18，共享 `0x793c6900`)；真正
的 IPersistStream 虚表在位移非 0 的 COL 上，槽函数为
`FUN_thunk(this − *(int*)(this−4))` 形态的 this 调整器，Load/Save thunk
尾调各实体。这与 Renderers.dll HorizontalLegendItem 的既有结论同构。

## 共同前缀调用者集合补全（证据：确定的）

`FUN_794413d0`（共同前缀 Load）的 9 个调用点所属函数本轮经 Ghidra
`getFunctionContaining` 全部定位：

| 实体 | 归属（本轮证据） |
| --- | --- |
| `0x793da250` | Legend（既有记录） |
| `0x794460b0` | Scalebar（本轮闭合） |
| `0x79457410` | ScaleText（本轮闭合） |
| `0x794426f0` | MapTitle（本轮闭合） |
| `0x793a3b90` | BaseNorthArrow（MarkerNorthArrow 的基类，本轮闭合） |
| `0x79451120` | 比例尺子类基类段（Hollow/SingleDivision/Alternating/DoubleAlternating 在 `this+4` 调它，本轮闭合） |
| `0x793c8990` | ScaleLine 族候选（本轮未定性；体内含 2 个 IEnvelope 槽 `DAT_79481580`） |
| `0x7943b2f0` | ScaleLine 族候选（未定性） |
| `0x79448500` | ScaleLine 族候选（未定性） |

即族内除 ScaleLine 系三个候选外全部闭合；ScaleLine/SteppedScaleLine/
ScaleLineImpl 不在本轮任务清单，留待后轮。

## 一、MapTitle（`{7A3F91DF-…}`，体完全闭合；证据：确定的）

GetClassID（identity 虚表 +0x14 `0x794426b0`）写出 CLSID；Load `0x794426f0` /
Save `0x79442740`（IPS 虚表 `0x7948a76c`，位移 0xdc）。

字段序：① int16 自身版本（**原生无拒绝门**，负值位模式通过；Save 恒写
静态常量 1，`DAT_79480d48` = `01 00`）→ ② 共同前缀（`FUN_794413d0(this)`，
同一 this——MapTitle 对象基即 MapSurround 基，无派生字段）。Save 对称：
写版本 1 → `FUN_79441510`（共同前缀 Save）。**MapTitle 持久化体 =
共同前缀，至此完结。**

Map.dll 10.1.0.3035 的 0x7944270e 读取自身 short 后只检查 HRESULT，
0x7944272a 直接调用共同前缀 Load。共同前缀在 0x7944147f 和
0x794414ba 分别将 2、3 与有符号 short 比较，使用 jg 跳过相应字段。
Rust 按此读取并保留两个版本的原始位模式。自身 short 是格式版本；
共同前缀的名称、关联地图、FixedAspectRatio 和 FitToBounds 写出的
WKSEnvelope 已分别对位，见[成员取证](mxd-map-tail-cross-stream-20261118.md)。
合成用例以紧邻的后继字节验证负版本及大于写侧版本的读侧边界。

## 二、Scalebar（`{7A3F91DB-…}`，体完全闭合；证据：确定的）

GetClassID `0x79446040`；Load `0x794460b0` / Save `0x79446550`。字段序
（Load/Save 逐段对称）：

| # | 字段 | 备注 |
| --- | --- | --- |
| 1 | int16 自身版本 | 有符号 `1 < v` 拒绝 `0x80004005`；Save 恒写 1（`DAT_79480d40` = `01 00`） |
| 2 | 共同前缀（5 字段） | `FUN_794413d0`；含其自身的版本 short（无拒绝门）/名称/IMap 回引/条件字段 |
| 3 | int32 ×3 → −0xdc/−0xd8/−0xd4 | 原样保留 |
| 4 | double → −0xcc | 原样保留 |
| 5 | int16 ×3 → −0xc4/−0xc2/−0xc0 | 原样保留 |
| 6 | 20×4 字节原样 → −0xbc..−0x70 | Ghidra 标注混合 long/ulong，按位模式保留 |
| 7 | LoadObject（请求 IID `{BEF6E002-A874-101A-8BBA-00AA00300CAB}` = **IFont**，OLE 标准）→ 槽 −0x6c | Save `SaveObject(−0x6c)` 对称；流内预期 OLE StdFont（`{0BE35203-…}`） |
| 8 | uint32 → −0x5c、int32 → −0x58 | 原样保留 |
| 9 | LoadObject（IFont）→ 槽 −0x54 | |
| 10 | uint32 → −0x44 | |
| 11 | String → −0x40 | 数字格式串候选 |
| 12 | int32 ×4 → −0x3c/−0x38/−0x34/−0x30 | |
| 13 | LoadObject（IFont）→ 槽 −0x2c | |
| 14 | uint32 → −0x1c | 体尾 |

三个 IFont 槽是比例尺标注字体候选（OLE StdFont 体的读取器在
`src/objects/ole_font`，样本 37 实例佐证过其布局）。

复核 Map.dll 10.1.0.3035（SHA-256
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`）。
旧字段表从 −0xac 起算，漏列四项。Load 在
`0x794461e4/0x794461ff/0x7944621a/0x79446239` 取四个连续成员的地址，
Save 在 `0x7944667d/0x79446698/0x794466b3/0x794466d2` 对称写出。
读取器先保留这四个值，再读取原有 16 项。合成输入验证三个字体槽与
后续字段的边界，以及逐前缀截断回滚。字段业务语义和 ArcMap/QGIS
实档目视验证待补。

Load 的 `0x794460ea` 对自身版本做 16 位比较，`0x794460ef` 以
有符号 JG 拒绝大于 1 的值。随后立即读取共同前缀，余下字段不再
按自身版本分支。读取器据此保留负版本位模式并继续读取同一对象体。
构造测试覆盖有符号最小值、−1、0、1 及两个超上限值。负版本的真实
写出来源和目标软件目视验证仍待核对。

## 三、ScaleText（`{7A3F91DC-…}`，体完全闭合；证据：确定的）

GetClassID `0x79455970`；Load `0x79457410` / Save `0x794576a0`。

| # | 字段 | 版本门 | 备注 |
| --- | --- | --- | --- |
| 1 | int16 自身版本 | 有符号 `3 < v` 拒绝 | Save `AfStreamGetSaveVersion(docVer, 4, DAT_79480d08)`，表 `[2,2,2,3]` → 10.1 恒写 3 |
| 2 | 共同前缀 | 恒有 | `FUN_794413d0(this−0x48)` |
| 3 | String + LoadObject(ITextSymbol `{A80B5E91-…}`) | 仅版本 <2 | 旧形态（10.1 不可达）；经 `−0x38` 槽接口虚表 +0x18/+0x10 setter 装入 |
| 4 | int32 → −0x34；String → −0x30；int32 → −0x2c；int32 → −0x28 | 恒有 | |
| 5 | int32（读入即丢） | 仅版本 <2 | 10.1 不可达 |
| 5′ | LoadObject(**ITextElement** `{204034D2-F6EA-11D0-83AD-080009B996CC}`，类型库 esriCarto) → 槽 −0x38；String → −0x24；String → −0x20；LoadObject(**INumberFormat** `{7E4F470D-…}`) → 槽 −0x18 | 版本 ≥2 | Save 对称：`SaveObject(−0x38)`、`String::Write` ×2、`SaveObject(−0x18)` 恒写（10.1 版本 3 ≥2） |
| 6 | String → −0x1c（格式串候选） | 版本 ≥3 | <3 时原生写内存默认（`":"` 或经 0x153b 格式化），不读流 |

## 四、MarkerNorthArrow（`{7A3F91DD-…}`，体完全闭合；证据：确定的）

GetClassID `0x79444b40` 写出 `DAT_794850e8/794850f0` =
`{7A3F91DD-B9E3-11D1-8756-0000F8751720}`；Load `0x79444b80` /
Save `0x79444ca0`。两级版本（自身 + BaseNorthArrow 基类段）：

| # | 字段 | 版本门 | 备注 |
| --- | --- | --- | --- |
| 1 | int16 自身版本 | **无拒绝门**（负值通过；分支用） | Save 恒写 2（`DAT_79480d44` = `02 00`） |
| 2 | int16 BaseNorthArrow 版本 | 有符号 `4 < v` 拒绝 | Save `AfStreamGetSaveVersion(docVer, 6, DAT_79480ea0)`，表 `[3,3,3,3,3,4]` → 10.1 恒写 4 |
| 3 | 共同前缀 | 恒有 | `FUN_794413d0(this−0x40)` |
| 4 | double → −0x2c；double → −0x24 | 恒有 | 基类 |
| 5 | double → −0x1c | 基类版本 ≥2 | <2 内存清零不读流 |
| 6 | LoadObject(**IPoint** `{1B077915-38C5-11D0-92D2-00805F7C28B0}`，类型库 esriGeometry） | 基类版本 ≥3 | <3 内存释放不读流；位置点候选 |
| 7 | int32 → −0x34 | 基类版本 ≥4 | <4 内存清零 |
| 8 | int32（读入即丢）+ LoadObject(**IMarkerSymbol** `{E6BDAA7C-…}`) | 自身版本 <2 | 10.1 写侧不生成（自身恒写 2）；符号经虚表 setter 装入 −0x1c |
| 9 | LoadObject(**IMarkerElement** `{530FD711-EF0C-11D0-83A0-080009B996CC}`，类型库 esriCarto）→ 槽 −0x1c | 自身版本 ≥2 | Save `SaveObject(−0x1c)` 恒写；标记元素候选 |

### 指北针的属性

esriCarto 10.1 的 INorthArrow、INorthArrow2 成员表分别对位 Map.dll
的 0x7948b3ec 和 0x7948b44c。MarkerNorthArrow RTTI 定位这两个接口在
完整对象 +0xd8 和 +0xdc。基类 Save 0x793a3ce0 以完整对象 +0x11c
为基址，和两个接口分别相差 0x44、0x40。

| 基类流字段 | 属性 | 取证 |
|---|---|---|
| −0x2c | Size | INorthArrow 槽 0x40 的 setter 在 0x79444a75 将输入 double 缓存到接口 +0x18 |
| −0x24 | Angle | 槽 0x44 的 getter 0x793a39b0 读取 +0x20 |
| −0x1c | CalibrationAngle | 槽 0x48 的 getter 0x793a39e0 读取 +0x28 |
| −0x14 | ReferenceLocation | 槽 0x50 的 getter 0x793a3a60 读取 +0x30 的 IPoint |
| −0x34 | NorthArrowType | INorthArrow2 槽 0x5c 的 getter 0x793a3ad0 读取 +0x0c，对应 esriNorthArrowType |

Size 的 getter 0x794449b0 从标记元素请求符号，setter 同步缓存大小；
解析器保留保存时缓存的值。旧版标记符号前的 int32 在当前 10.1 读取路径
读入后丢弃，其历史写侧业务名仍未查明。上述对位来自静态指令，尚未在
ArcMap 面板中核对显示值。

## 五、四个 Scalebar 子类（体完全闭合；证据：确定的）

CLSID 与 GetClassID：HollowScaleBar `{6589F143-F7F7-11D2-B872-00600802E603}`
`0x793c5ea0`、SingleDivisionScaleBar `{6589F146-…}` `0x79457db0`、
AlternatingScaleBar `{6589F147-…}` `0x793a12b0`、DoubleAlternatingScaleBar
`{6589F148-…}` `0x793b2950`（均写出对应 CLSID，与 esri.clsid 声明互证）。

Load 实体：Hollow/Alternating/DoubleAlternating **共享** `0x793a1be0`
（Save：Hollow/Alt 共享 `0x793a1c60` 版本常量 `DAT_79480eb0`、DblAlt 独立
`0x793b3410` 版本常量 `DAT_79480e9c`——两者代码同构仅常量地址不同）；
SingleDivision 独立 `0x79458830`/Save `0x794588b0`（版本常量
`DAT_79480d04`）。四类流内布局同构，仅中段填充符号槽个数不同。

| # | 字段 | 版本门 | 备注 |
| --- | --- | --- | --- |
| 1 | int16 自身版本 | 有符号 `1 < v` 拒绝 | Save 恒写 1 |
| 2 | int16 基类段版本 | 有符号 `6 < v` 拒绝 | 基类 Load `0x79451120`；Save `AfStreamGetSaveVersion(docVer, 6, DAT_79480d28)`，表 `[3,3,3,3,4,6]` → 10.1 恒写 6 |
| 3 | 共同前缀 | 恒有 | `FUN_794413d0(this−0x190)` |
| 4 | double → −0x17c；int16 ×3 → −0x174/−0x172/−0x170；int32 → −0x16c；String → −0x168（UnitLabel）；int32 → −0x164；double → −0x15c | 恒有 | |
| 5 | LoadObject(ITextSymbol `{A80B5E91-…}`) → 槽 −0x154 | 恒有 | 标签符号候选 |
| 6 | int32 → −0x150 | 恒有 | |
| 7 | int32（读入即丢） | 基类版本 <2 | 10.1 不可达 |
| 8 | int32 → −0x148；double → −0x144 | 恒有 | |
| 9 | LoadObject(ITextSymbol) → 槽 −0x13c | 恒有 | |
| 10 | LoadObject(**INumberFormat** `{7E4F470D-…}`) → 槽 −0x14c | 基类版本 ≥2 | |
| 11 | int32 → −0x138 | 基类版本 ≥3 | |
| 12 | double → −0xec | 基类版本 ≥4 | Save 侧按 docVer≥4 写（10.1 写） |
| 13 | int16 → −0xbc | 基类版本 ≥5 | Save 按 docVer≥5 写 |
| 14 | int16 → −0xba | 基类版本 ≥6 | Save 恒写 |
| 15 | int16 中段版本；double → +8 | 有符号 `1 < v` 拒绝 | 中段 Load `0x793b3bf0`（Hollow/Alt/DblAlt 共享）或 `0x79459000`（SingleDivision）；Save 恒写 1 |
| 16 | LoadObject(**IFillSymbol** `{E6BDAA7E-…}`) ×2（SingleDivision ×1）→ 槽 +0x10/+0x14 | 恒有 | Save `SaveObject(+0x10)`、`SaveObject(+0x14)`（SingleDivision 仅 +0x10） |
| 17 | int16 尾段版本 | 有符号 `2 < v` 拒绝 | 共享尾段 Load `0x79454ee0`；Save 恒写 2（`DAT_79480d10` = `02 00`） |
| 18 | int16 标志 | 尾段版本 <2 | 旧档读取时由标志生成 +0x24（非 0 → 5，否则 0） |
| 19 | LoadObject(**ILineSymbol** `{E6BDAA7D-…}`) ×2 → 槽 +4/+8 | 恒有 | |
| 20 | double → +0x10；double → +0x18；int32 → +0x20；int32 → +0x24 | 前三项恒有，+0x24 仅版本 ≥2 从流读取 | |

## 六、LegendFormat（`{7A3F91E5-…}`，体完全闭合；证据：确定的）

GetClassID（identity 虚表 +0x14 `0x793dfd50`）写出 CLSID（`DAT_79485128/
79485130`）；真实 IPS 虚表为位移 0xb4 的 COL `0x79493538`（虚表
`0x794860a0`，Load thunk `0x793e1440` → 实体 `0x793e0c50`、Save thunk
`0x793e1630` → 实体 `0x793e0ec0`）。Legend 的 ILegendFormat 槽（Legend::Load
第 3 步）在流内遇到本类新对象时即进入此体。

| # | 字段 | 版本门 | 备注 |
| --- | --- | --- | --- |
| 1 | int16 版本 | 有符号 `2 < v` 拒绝 | Save `AfStreamGetSaveVersion(docVer, 6, DAT_79480db8)`，表 `[1,1,1,1,1,2]` → 10.1 恒写 2 |
| 2 | int16 → −0x8c | 恒有 | 原样保留 |
| 3 | LoadObject(ITextSymbol `{A80B5E91-…}`) → 槽 −0x88 | 恒有 | 标题符号候选 |
| 4 | int32 → −0x84 | 恒有 | |
| 5 | double ×11 → −0x7c..−0x2c | 恒有 | 原样保留（索引 n 对应 −0x7c + 8n） |
| 6 | LoadObject(**ILinePatch** `{BEB4EDB7-E3B4-11D2-B868-00600802E603}`）→ 槽 −0x14 | 恒有 | Save `SaveObject(−0x14)` |
| 7 | LoadObject(**IAreaPatch** `{BEB4EDB6-E3B4-11D2-B868-00600802E603}`）→ 槽 −0x10 | 恒有 | Save `SaveObject(−0x10)` |
| 8 | int16 → −0xc | 恒有 | |
| 9 | double ×2 → −0x24/−0x1c | 版本 ≥2 | <2 内存清零不读流；Save 按「保存版本 ≥2」写（10.1 一致） |

即 LegendFormat 体内嵌 ITextSymbol/ILinePatch/IAreaPatch 三个新对象槽——
Legend 读取器在其旧停止点（`LegendStop10_1::LegendFormatBody`）越过本类
体所需的全部字段序至此闭合。

### LegendFormat 的公开属性

对 Map.dll 10.1.0.3035 的 QueryInterface 0x793e0130 逐个读取 IID。
0x79485a28、0x79486108、0x79486118 分别与 esriCarto 10.1 类型库中的
ILegendFormat、ILegendFormat2、ILegendLayout 对应。返回的子对象在完整对象
+0x18、+0x1c、+0x20，RTTI 虚表分别是 0x79485f34、0x79485fcc、0x79486070。
按类型库 oVft 跟到 getter，用实际读成员指令和 Save 0x793e0ec0 对位如下。

| 流字段 | 属性（流内顺序） | getter 地址（同序） |
|---|---|---|
| short、对象、int32 | ShowTitle、TitleSymbol、TitlePosition | 793e02e0、793e0330、79456f90 |
| 连续 double 的前六项 | TitleGap、VerticalItemGap、HorizontalItemGap、LayerNameGap、GroupGap、HeadingGap | 793a39b0、793a39e0、793e0500、793e0550、793e0590、793e05d0 |
| 连续 double 的后五项 | TextGap、VerticalPatchGap、HorizontalPatchGap、DefaultPatchWidth、DefaultPatchHeight | 793e0610、793e0650、793e0690、793e06d0、793e0730 |
| 两个对象 | DefaultLinePatch、DefaultAreaPatch | 793e0790、793e0910 |
| 后段 short | ILegendLayout.ScaleGraphicsOnResize | 793e0290 |
| v2 两个 double | ILegendFormat2.LabelWidth、DescriptionWidth | 793e0b90、793e0bf0 |

例如 ScaleGraphicsOnResize 从第三个接口 +0x88 读取 short，即完整对象
+0xa8，对应 Save 指针 −0xc。两个宽度分别从第二个接口 +0x74/+0x7c 读取。
上述字段绑定属于原生静态证据。未知补丁几何的构造输入已验证停止在所属
补丁的对象头，读取器保留此前已完成的补丁键，截断返回错误时回滚本次调用。

## 七、LinePatch / AreaPatch（体完全闭合；证据：确定的）

两 coclass 声明于 Layer.dll（esri.clsid）；RTTI `.?AVLinePatch@@`/
`.?AVAreaPatch@@`。**两类共享同一对 IPersistStream 实现**（基类 Patch；
位移 0x48 的 IPS 虚表函数指针完全相同）：Load thunk `0x78662610` → 实体
`0x78666b70`、Save thunk `0x78662730` → 实体 `0x78666c40`。类的区别仅在
对象头 CLSID 与 GetClassID（LinePatch `0x7865c390` 写出 `{2066267F-…}`、
AreaPatch `0x7865c640` 写出 `{2066267E-…}`）。字段序：

| # | 字段 | 版本门 | 备注 |
| --- | --- | --- | --- |
| 1 | int16 版本 | 有符号 `3 < v` 拒绝 | Save `AfStreamGetSaveVersion(docVer, 3, DAT_7866d954)`，表 `[2,2,3]` → 10.1 恒写 3 |
| 2 | LoadObject(**IGeometry** `{1B07790D-38C5-11D0-92D2-00805F7C28B0}`，类型库 esriGeometry）→ 槽 −8 | 恒有 | 补丁形状；LinePatch 预期 Polyline、AreaPatch 预期 Polygon（类身份由流内对象头决定） |
| 3 | String → −0x10 | 恒有 | 补丁名称候选 |
| 4 | int16 → −0xc | 版本 ≥2 | |
| 5 | 装载后整理 `FUN_786664b0` | 版本 <3 | 不读流；10.1 恒写版本 3，分支不可达 |

### 两类 Patch 的属性

Layer.dll 10.1.0.3035 的 LinePatch 与 AreaPatch 在完整对象 +0x18
分别装配 0x7866e814 与 0x7866e738，两张表的 IPatch 成员实现一致。
esriCarto 10.1 类型库给出 IPatch 的 IID BEB4EDB5-E3B4-11D2-B868-00600802E603。
Name 的 get/put（槽 0x0c/0x10）在 0x78663450/0x78663480 使用接口 +0x1c
的 String。PreserveAspectRatio 的 get/put（0x14/0x18）在
0x7865b0a0/0x786667d0 读取和写入接口 +0x20 的 short。
槽 0x1c 的 0x78666af0 替换 +0x24 的 Geometry。三者分别对位
Save 0x78666c40 的 −0x10、−0xc、−8；v0/v1 缺少比例标志，
v2 起按原始 short 保存。实档界面属性值尚未人工核对。

## 八、ILegendItem 其余实现类（体前缀 + 尾段完全闭合；证据：确定的）

Renderers.dll 三类与 HorizontalLegendItem 同构：identity 虚表 +0x14 =
GetClassID，IPS 虚表由构造动态安装；Load 实体以 IAT `0x7cae9398`
（LayerLib `?Load@BaseLegendItem@@UAGJPAUIStream@@@Z`）的四条代码引用定位
（四实现类恰四条）。

| 类 | CLSID | GetClassID | Load 实体 | Save 实体（版本常量） | 基类嵌入偏移 |
| --- | --- | --- | --- | --- | --- |
| HorizontalBarLegendItem | `{2B65D211-C2C7-11D3-92F3-00600802E603}` | `0x7cad29d0` | `0x7cad3040` | `0x7cad30e0`（`DAT_7cae9764`=1） | −0x20 |
| NestedLegendItem | `{2B65D212-C2C7-11D3-92F3-00600802E603}` | `0x7cad7a10` | `0x7cad8fd0` | `0x7cad9100`（`DAT_7cae974c`=1） | −0x50 |
| VerticalLegendItem | `{A9401A48-4649-11D1-880B-080009EC732A}` | `0x7cae1550` | `0x7cae1a90` | `0x7cae1b00`（`DAT_7cae9738`=1） | −8 |

字段序（条目版本 short 均有符号 `1 < v` 拒绝；Save 恒写 1）：

- **HorizontalBarLegendItem**：条目版本 → BaseLegendItem 字段序 → int32
  （`−0x1c`）→ double（`−0x14`）→ double（`−0xc`）。
- **NestedLegendItem**：条目版本 → BaseLegendItem 字段序 → int32（`−0x2c`）
  → double（`−0x24`）→ LoadObject(**IFillSymbol** `{E6BDAA7E-…}`) →
  LoadObject(**ILineSymbol** `{E6BDAA7D-…}`) → int16 ×3（`−0x14/−0x12/−0x10`）
  → int32（`−0xc`）。
- **VerticalLegendItem**：条目版本 → int32（`−8`）→ BaseLegendItem 字段序。
  **流内布局与 HorizontalLegendItem（既有读取器）逐字节同序**（两者 C++
  基类槽同为 −8），区别仅在对象头类身份。

BaseLegendItem 字段序沿用 legend-tails 轮结论（`docs/arcmap-10.1-legend-tails.md`
第二节），本轮未发现字段序层面的出入。

### 三种图例项的派生属性

Renderers.dll 10.1.0.3035（SHA-256
`399c80c99a6c61de120c21b518d9b29069f2c78b20a705c07178aa3e2b656274`）
与 esriCarto 10.1 类型库对位如下。HorizontalBar 的 QI 0x7cad2e30
将 IHorizontalBarLegendItem 返回为完整对象 +0x178，将 IVerticalLegendItem
返回为 +0x17c。其 Arrangement getter 0x7cae15b0 读接口 +4，
等于 Save 指针 +0x19c 减 0x1c。AngleAbove、AngleBelow 的 getter
0x7cad2a50、0x7cad2ac0 分别读第一个接口 +0x10、+0x18。
Vertical 的 IVerticalLegendItem 虚表 0x7caec49c 复用同一 Arrangement
getter，接口在 +0x178，成员即 Save 指针 +0x184 减 8。

Nested 的 QI 0x7cad8030 将 INestedLegendItem 返回为 +0x178，
IHorizontalLegendItem 为 +0x17c。后者 getter 0x7cad7a70 读 +0x24，
对应 Save 指针 +0x1cc 减 0x2c，即首个 Arrangement。前者虚表
0x7caeb974 的 getter 与 Save 0x7cad9100 相比，依流内顺序如下。

| 成员 | 属性 | getter |
|---|---|---|
| −0x24 | LeaderOverhang | 7cad7b30 |
| −0x1c、−0x18 | OutlineSymbol、LeaderSymbol | 7cad7aa0、7cac8910 |
| −0x14、−0x12、−0x10 | AutoLayout、LabelEnds、ShowOutlines | 7cad7b80、7cad7bd0、7cad7c20 |
| −0xc | HorizontalAlignment | 7cad7c70 |

三类的外层版本均由有符号 short 比较后 jle 进入对象体，实际指令在
0x7cad3073、0x7cad9007、0x7cae1ac0。持久化字段的共享部分见
[BaseLegendItem 属性表](mxd-terra-depth-legend.md)。未做 ArcMap 图例显示验收。

## 九、请求 IID 定名补全（类型库 typekind=3，证据：确认（类型库））

| GUID | 名称 | 出现槽位 |
| --- | --- | --- |
| `{BEF6E002-A874-101A-8BBA-00AA00300CAB}` | IFont（OLE 标准，非 esri 类型库） | Scalebar 三个字体槽 |
| `{A80B5E91-7F9C-11D0-9410-080009EEBECB}` | ITextSymbol | LegendFormat 标题、比例尺基类段 ×2、ScaleText 旧段 |
| `{204034D2-F6EA-11D0-83AD-080009B996CC}` | ITextElement | ScaleText ≥2 主槽 |
| `{530FD711-EF0C-11D0-83A0-080009B996CC}` | IMarkerElement | MarkerNorthArrow ≥2 主槽 |
| `{E6BDAA7C-4D35-11D0-98BE-00805F7CED21}` | IMarkerSymbol | MarkerNorthArrow <2 旧槽（10.1 不可达） |
| `{1B077915-38C5-11D0-92D2-00805F7C28B0}` | IPoint | BaseNorthArrow ≥3 位置槽 |
| `{1B07790D-38C5-11D0-92D2-00805F7C28B0}` | IGeometry | Patch 形状槽 |
| `{BEB4EDB7-E3B4-11D2-B868-00600802E603}` | ILinePatch | LegendFormat 槽 −0x14 |
| `{BEB4EDB6-E3B4-11D2-B868-00600802E603}` | IAreaPatch | LegendFormat 槽 −0x10 |
| `{E6BDAA7E-4D35-11D0-98BE-00805F7CED21}` | IFillSymbol | 比例尺子类中段、NestedLegendItem |
| `{E6BDAA7D-4D35-11D0-98BE-00805F7CED21}` | ILineSymbol | 比例尺子类尾段 ×2、NestedLegendItem |
| `{7E4F470D-8E54-11D2-AAD8-000000000000}` | INumberFormat | ScaleText、比例尺基类段、（既有）HorizontalLegendItem |

## 十、FeatureBookmark 的 IName 槽（机制补全；分布仍未观察）

FeatureBookmark::Load `0x793b5c50` 的名称对象槽（第 ③ 步，请求 IID
`{4BA33331-…}` IName）本轮补全装载后接线：

- 装载成功后先对名称对象 **IName 视图虚表 +0x14 直调**（一参虚调用；槽
  5。按标准 IName 只有 get_NameString 一个方法，+0x14 已越过 IName 自身
  槽——该方法经由哪一真实接口/语义未展开，本轮反汇编现场取证未完成
  （共享工程锁冲突，脚本中止），开放）；
- 随后 `FUN_793acdc0` → `FUN_793ad1c0`：对名称对象 **QI
  `{D4803EE6-79F4-11D0-97FC-0080C7F79481}` = IFeatureClass**（类型库
  esriGeometry，typekind=3），QI 成功则把 IFeatureClass 指针写入
  `this+8`，失败则 `this+8` 置空（软失败，不报错）。
- 解读（候选的）：书签的名称对象需能开出 IFeatureClass（或 QI 软失败留
  空引用）；流内实现类分布仍无样本佐证（样本书签计数 0）。TODO 该项的
  「实现类体」在分布观察之前无法闭合——机制已按反编译记录于上，类分布与
  +0x14 直调语义维持开放。

## 实现（src/；新模块 + mod 声明行，未接分派链）

- `src/layers/patch.rs`（新）：`LINE_PATCH_CLASS_ID`/`AREA_PATCH_CLASS_ID`
  常量与共享体读取器 `read_patch_body_10_1`（严格事务子集；几何槽按
  `objects::geometry` 四类闭合几何分派，其余类停止回退）。
- `src/layers/legend_item_family.rs`（新）：三常量与三读取器
  （`read_horizontal_bar_legend_item_body_10_1`/
  `read_nested_legend_item_body_10_1`/`read_vertical_legend_item_body_10_1`）
  及本地副本 `read_base_legend_item_fields_10_1`（BaseLegendItem 字段序；
  副本原因与同步义务见模块文档）。与既有 `legend_item` 的差异：本副本的
  ITextSymbol/INumberFormat 槽对 TextSymbol/NumericFormat 整体消费（既有
  实现一律停止）——整合轮建议以本副本口径统一并消除副本。
- `src/maps/map_surround_family.rs`（新）：九常量与五类读取器
  （LegendFormat/Scalebar/ScaleText/MarkerNorthArrow/MapTitle + 参数化
  `read_scalebar_subclass_body_10_1`）。嵌套槽分派：TextSymbol→
  `objects::element_body`、StdFont→`objects::ole_font`、TextElement/
  MarkerElement→`objects::element_body`、Point→`objects::geometry`、
  LinePatch/AreaPatch→`layers::patch`、NumericFormat→
  `objects::number_format`（仅 NumericFormat，其余 INumberFormat 实现
  类停止）、ISymbol 族→`objects::display_symbol::read_symbol_slot_full_10_1`
  全量分派；共同前缀复用 `maps::map_surround::read_map_surround_common_prefix_10_1`。
- 事务性：全部读取器 trial 游标 + 暂存登记表，停止点为成功返回（游标回退
  到槽位起点、该键不保留），错误不推进调用者游标、不改登记表。
- 未知类新对象槽为停止点（无外部长度字段，不可跳过）——与仓库既有读取
  器约定一致。

## 测试（模块内 #[cfg(test)]；全部合成输入，样本 0 实例）

- `patch.rs` 6 项：完整体（内嵌 Polyline）、线/面共用布局、版本 >3 与负值
  位模式拒绝、未知几何类停止点游标回退、逐前缀截断事务性、版本 1/2 尾字段
  差异。
- `map_surround_family.rs` 16 项：九常量黄金字节表、MapTitle 完整体与版本
  拒绝、Scalebar 完整体（StdFont 槽 ×2 + 空标记）、字体槽未知类停止、
  ScaleText v3（NumericFormat 槽）与版本拒绝、MNA v2 与两级版本拒绝、
  LegendFormat v2（内嵌 LinePatch 体）与补丁槽停止、子类完身体
  （SingleDivision 1 填充 / Hollow 2 填充）与版本拒绝、Scalebar 逐前缀截断
  事务性。
- `legend_item_family.rs` 13 项：三常量、基类字段序 v3/v1、ILayer 未知类
  停止回退、类格式槽 LegendClassFormat 整体消费、基类版本拒绝、三类完
  整体、Vertical 与既有 HorizontalLegendItem 读取器同流同布局互证、截断
  事务性。
- 全部为合成输入（样本 0 实例），通过不代表 MXD 格式已验证。

## 建议整合接线点（写给整合轮；本轮不改共享文件）

1. `maps::map_object` 的 MapSurround 循环类分派（现仅 Legend/AOI/Feature
   书签）：按流内类标识追加 `SCALEBAR_CLASS_ID`/`SCALE_TEXT_CLASS_ID`/
   `MARKER_NORTH_ARROW_CLASS_ID`/`MAP_TITLE_CLASS_ID` 及四个子类常量 →
   `map_surround_family` 各读取器整体消费；体内停止以新变体透传。
2. `maps::map_surround::read_legend_prefix_10_1` 的 ILegendFormat 槽
   （`LegendStop10_1::LegendFormatBody`）：新对象类标识为
   `LEGEND_FORMAT_CLASS_ID` 时接 `read_legend_format_body_10_1`（体内停止
   透传），其余类维持停止。
3. `maps::map_surround` 的 ILegendItem 条目循环：新对象类标识为三常量时接
   `legend_item_family` 三读取器（Vertical 与 HorizontalLegendItem 同序，
   分派按 CLSID 区分）；体内停止透传。
4. `layers::legend_format::read_legend_class_format_body_10_1` 的
   ILinePatch/IAreaPatch 槽（现 `SlotUnsupported` 停止）：新对象类标识为
   `LINE_PATCH_CLASS_ID`/`AREA_PATCH_CLASS_ID` 时接 `patch` 读取器。
5. `layers::legend_item` 的槽分派对齐 `legend_item_family` 口径
   （ITextSymbol/INumberFormat 闭合类整体消费）并复用其 BaseLegendItem
   读取器消除副本。
6. robustness 集中清单补入三模块读取器入口（本轮按分工不触碰
   `tests/robustness.rs`）。

## 开放项

1. **样本 0 实例**（全篇边界）：本模块全部类在样本中无字节；真实流中的
   装饰物类分布、共同前缀 IMap 槽条目形态等需含装饰物/书签的样本或运行
   取证。
2. 全部标量字段的业务语义未定性（Scalebar 16 槽标量、三个 IFont、
   ScaleText 的 int32 组与两个字符串、MNA 的 double 组、比例尺基类段
   标量等；候选语义按类名与写侧槽位推断）。
3. 版本短体形态（各版本门以下的旧版分支：ScaleText <3、比例尺基类 <6、
   MNA 基类 <4 等）无字节佐证（10.1 写侧恒写满形态版本）。
4. ScaleLine 族三个共同前缀调用者（`0x793c8990`/`0x7943b2f0`/`0x79448500`）
   归属与体未反编译（不在本轮任务清单）。
5. FeatureBookmark IName 槽：+0x14 直调的接口与语义未展开（反汇编取证因
   共享工程锁冲突中止）；名称对象实现类分布未观察（样本 0 书签）——TODO
   「实现类体」项的闭合需此项先行。
6. 共同前缀自身的开放项（第 5 段 32 字节读入即丢的语义等）沿用
   mapsurround-bookmark 记录，本轮未新增证据。
7. 全部结论为静态反编译观察；ArcMap 10.1 与 QGIS 的人工打开与目视核对均
   未进行，不声称已验证。

### MarkerNorthArrow 版本分支复核

Map.dll 10.1.0.3035 的 `0x79444bd1 / 0x79444bd8` 以有符号
比较选择旧符号槽或标记元素槽。自身版本无单独上限。
BaseNorthArrow 在 `0x793a3bca / 0x793a3bcf` 按有符号上限 4 拒绝超限值。
构造测试验证负版本走旧符号槽、版本 3 走标记元素槽和基类版本 5 拒绝。
旧槽之前被弃读的 LONG 的历史用途及真实工程仍待取证。

MarkerNorthArrow 的旧标记分支在 `0x79444be5` 将 LONG 读入
栈局部 `[ebp-0xc]`，随后只检查此次读取的 HRESULT。
`0x79444c0e` 加载标记符号，`0x79444c33 / 0x79444c3d`
将另一栈槽 `[ebp-4]` 中的符号接口交给 setter。旧 LONG
未参与设置，解析结果保留它的原始值以便比对旧档。
10.1 保存端恒写自身版本 2，历史写侧用途及真实旧档仍待核对。

从本地 142 份按 SHA-256 去重的 OLE MXD/MXT 成功提取 280 条
`/Maps` 和 `/PageLayout` 流。MarkerNorthArrow 类标识在两类流各有
26 处命中。Maps 命中后的 +20 两字节均为 2，布局命中后的 +20 值
均为 55152。扫描值只用于筛选，不能当作对象版本。

10.1 Desktop 的 LandscapeClassic 实档通过 Map 根入口定位
`/Maps:0x885` 的 MarkerNorthArrow v2，确认无旧 LONG。
随后从双流入口读至布局流末尾，同键 MapSurroundFrame 插槽解析为
回引，因而该处 +20 两字节属于后续字段。早期写侧用途仍待取证，
ArcMap 10.1 人工验证未进行。


旧槽读取器按 `0x79444c0e` 请求的 IMarkerSymbol 约束分派。
对新对象先检查类，对回引优先检查键登记的类。构造测试验证
SimpleLineSymbol 新对象和头标识为 SimpleMarkerSymbol、登记类为
SimpleLineSymbol 的回引都会在该槽起点停止。旧 LONG 的历史写侧
用途及真实旧档仍待确认。


### 图形元素的符号回引

MarkerElement、LineElement、PolygonElement 等图形元素按各自
请求的 IMarkerSymbol、ILineSymbol 或 IFillSymbol 校验回引。
沿用分流 ObjectStream 的原生规则，SystemCore.dll 10.1.0.3035
在 `0x7ffd0063` 按键找到登记实例后查询请求接口。因此优先用登记类
检查符号族，成功时保留回引头原始类标识，失败时在符号槽起点停止。
构造测试覆盖三种符号槽的登记类与头类不一致的双向情况。
ArcMap/QGIS 人工验收未进行。
