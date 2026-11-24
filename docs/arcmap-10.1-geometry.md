# ArcMap 10.1 Geometry.dll 几何 coclass 对象体（Polyline 记录布局与同族骨架）

记录日期：2026-11-15。对应 TODO「MXD 解析」节开放项「TextElement 的
IGeometry 槽实现类（Polyline 等）体未闭合」。全部结论为**静态反编译/汇编核
对 + 样本字节对照**，未运行 ArcMap，未做人工目视验收；样本吻合仅作辅助观
察。不写本机绝对路径。

## 方法与环境

- **Geometry.dll**（Setup1.cab 条目 `Geometry.dll.F84366F9_C0B7_4267_A16E_...`，
  复制自既有提取物）：2,890,752 字节，SHA-256
  `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`，
  ImageBase `0x717e0000`，pefile 复核 FileVersion **10.1.0.3035**。节区：
  `.text 0x1000..0x22f15c`、`.rdata 0x230000`。
- 深度分析在本机 pyghidra（Ghidra 12.2 + JDK 25）完成；分析工程为临时目
  录内独立命名工程（未入库），Open 副本不共享锁。地址均为 PE 10.1.0.3035
  静态映像 VA。定位链的前半段（CLSID 常量 → 立即数引用 → movq GetClassID
  形态 → RTTI TD→COL→虚表锚定）以 pefile + 段字节扫描机械完成，Ghidra 反
  编译复核 Load/Save/导入器。Ghidra 桥接工具 MCP 未使用（并行研究约定）。
- 样本为第六轮导出的 /Maps 流（152,941 字节，流内偏移，见
  element-collection 文档）。注意：该流注记簇之后区域含数据源路径，本轮仅
  读取几何对象字节，未把该区域写入任何入库文件。
- 类名/接口名：esri.clsid 全表（Geometry.dll 模块）与 wine `LoadTypeLibEx`
  类型库 dump（esriGeometry，仅检索）。

## 定位链（证据：确定的）

esri.clsid 表声明 → CLSID 常量 → 代码引用 → RTTI TD→COL→虚表 →
GetClassID movq 复制 → Load/Save，逐环闭环：

| 环节 | 地址 | 证据 |
| --- | --- | --- |
| esri.clsid 声明 | 模块 Geometry.dll，progID `esriGeometry.Polyline` | 全表 XML |
| CLSID 常量 | `.rdata 0x71a1187c`（16 字节 `{30707210-52D5-11D0-A8F2-00608C85EDE5}`） | 字节扫描唯一命中 |
| RTTI TD | `0x71a6a6a4`，名称串 `.?AVPolyline@ESRI@@`（TD 基址 = 名称 −8） | 字节扫描 |
| COL | `0x71a3d128`（`[C]==0` 且 `[C+0xc]==TD`） | 字节核对 |
| 主虚表 | `0x71a228c4`（虚表 −4 处为 COL 指针），槽系 `[QI 0x7193f390, AddRef 0x719f1550, Release 0x718de060, 析构 0x71942f90, 类名 getter 0x7193f340（返回 UTF-16 "Polyline" @0x71a1ab30）, GetClassID 0x7193f350 @+0x14 非标准位, 空桩 0x71973ab0]` | 与 element-bodies 轮「七槽主虚表」族同构；GetClassID 位于 +0x14，按该判据**排除**为持久化虚表 |
| 持久化虚表 | `0x71a22bd4`：`[QI 0x719b9720, AddRef 0x719b9700, Release 0x719ce220, GetClassID 0x719402d0 @+0x0c, IsDirty 0x719d3220 @+0x10, Load 0x7192db20 @+0x14, Save 0x7192dcb0 @+0x18, GetSizeMax 0x7191c370 @+0x1c]` | GetClassID movq 复制常量 `0x71a1187c`（0x719402da 处 `f3 0f 7e 05` 形态）；IsDirty/GetSizeMax 与 Envelope 持久化虚表共享同一函数（spatial-reference 轮的「恒返 0」「返 0x22」互证）；构造器 `0x71943290` 把 `0x71a22bd4` 装到对象 `+0x2c` 槽（`0x719432c8`） |
| GetClassID | `0x719402d0`：读参数指针判空（`0x80004003`）后 `movq` 双 8 字节复制常量到出参，`xor eax,eax` 返回 S_OK | 反编译 9 行 |

排除的假象：`0x71a116b8` 一带的「九槽全 .text」扫描命中是常量紧邻虚表区
造成的形状误报；全家族复核统一采用「COL 锚定虚表 + +0x0c 槽函数体内含
CLSID 常量引用」双条件（见下同族表）。

## 包装字段序（Load `0x7192db20`，反编译逐行核对；证据：确定的）

载入者签名 `Load(this, IStream*)`，`this` 为持久化接口视角（对象基址
+0x2c）。AfCore 导入名经 IAT 解析（`?AfStreamRead@@YAJPAUIStream@@PAFKPAK@Z`
等）：

1. 先调 `this+0x30` 接口虚表 `+0x14` 槽（实参 `&this+0x30`）——集合清理
   （候选 RemoveAll，不读流；与 ElementCollection::Load 的 adjustor 清理同
   构）；
2. `AfStreamRead(stream, &版本, 2, NULL)`：int16 版本 short。返回值 < 0 失
   整体失败；**返回 1（S_FALSE）时原生走「空流」捷径**：调 `this−0x1c` 接
   口 `+0x20` 槽（实参 `&this−0x1c`）后原样返回 1（样本未覆盖，本读取器不
   实现）；`(short)版本 > 2` 拒绝——经 `FUN_719f1570(0x80004005, 0x1c4, 0,
   0, DAT_71a1135c)` 记录后返回，**有符号比较**（负值位模式原生通过，读取
   器一并拒绝，较原生更严）；
3. `AfStreamRead(stream, &声明长度, 4, &实读)`：int32 声明长度；
4. `malloc(声明长度)`：NULL 时经 `FUN_719f1570(0x8007000e, 0x10f, …)` 记
   录失败（原生**无长度上限**，本读取器以 max_bytes 设限，较原生更严）；
5. `AfStreamRead(stream, 缓冲, 声明长度, &实读)`：整块读入；
6. `this−0x8` 接口（对象 `+0x24` 槽，虚表 `0x71a22b7c`）`+0x38` 槽：
   `Import(this−0x8, 标志 0x100, &实读, 缓冲)` —— 记录字节块导入；
7. 释放缓冲（`FUN_718b2ea0`）；
8. **版本 > 1**：`FUN_718ae2a0` 读侧装载 + AfCore `LoadObject(IUnknown**,
   const GUID&, IStream*, IUnknown*)`，请求 IID `DAT_71a15a10` =
   `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`（ISpatialReference；与 Map::Load
   第 4 步、Envelope 版本 3 槽同值），结果存 `this−0x1c` 接口 `+0x18` 槽；
   装载失败时该槽写空并清理后失败返回。

Save（`0x7192dcb0`，反编译核对）：`AfStreamGetDocumentVersion` →
`AfStreamWrite(stream, 常量 0x71a11e50, 2, NULL)` 写 **2 字节固定 `02 00`**
（不走 AfStreamGetSaveVersion 版本表，10.1 恒写版本 2）→ `this−0x8` 接口
`+0x10` 槽查询导出尺寸 → `malloc` → `+0x34` 槽填充（标志 = docVer > 1 ?
`0x400` : 0）→ `AfStreamWrite` 写 int32 长度 → 写记录字节 → 释放 →
SaveObject 空间参考。与 Load 版本 2 路径逐字段对称。

## 记录字节块内部布局（ShapePolyline::Parse `0x7197fc50`；证据：确定的）

`this−0x8` 接口 `+0x38` 槽（`0x7192c950`）按对象的形状模式选择导入路径。
默认模式直接调用堆上 `ESRI::ShapePolyline` 或 `ESRI::ShapePolygon` 的
`+0x94` 槽（`0x7197fc50`）。编辑模式下，Load 传入的标志 `0x100` 使导入入口
先构造对应的栈上 Shape，解析成功后通过编辑对象的 `+0x1cc` 槽移交结果。
两条路径都会在调用 Parse 前清除 `0x100` 位，因此 **Parse 收到的标志为 0**。
具体的对象偏移和构造路径见[形状对象装配](#polylinepolygon-的形状对象装配)。
布局（小端，`param_4` 为缓冲基址，`int*` 索引）：

| 偏移 | 字段 | 解析行为 |
| --- | --- | --- |
| 0 | int32 类型 | 存入 shape 对象 +0x14；**0 = 空几何**：清空、类型回填 3（Polygon 分支 5）、`*consumed = 4`、返回 S_OK（记录仅 4 字节）；其他值不校验（存什么算什么） |
| 4 | 4×f64 包络 | `WKSEnvelope::PutCoords(env, [4],[12],[20],[28])` = (XMin, YMin, XMax, YMax)，与 Envelope::Save 字段序同源 |
| 36 | int32 路径数 A | < 0 返回 `0x80040253` |
| 40 | int32 点数 B | < 0 返回 `0x80040253`；**B == 1 返回 `0x80040221`**（折线至少两点） |
| 44 | A×int32 | 路径起始点序号数组 |
| 44+4A | B×(f64,f64) | 二维点坐标（16 字节/点） |

长度自洽性：`44 + 4A + 16B` 恰为记录声明长度（样本 16/16：44+4+32=80）。
原生**不校验**该一致性（计数越界即未定义行为），读取器显式校验拒绝，较原
生更严。可选段（读路径标志恒 0 不可达，仅反编译记录）：标志 `0x80000000` →
Z 域 min/max 对（0x10 字节）+ 每点 Z（8B）；`0x40000000` → M 域对 + 每点
M；`0x20000000` → 段描述数组（`int32 计数` + 每条目 `int32 类型`（1 = Line、
4 = CircularArc、5 = BezierCurve）+ 段参数，按对象槽 `+0x14`/`+0x1ec`/`+0xe4`
分派）；`0x10000000` → 每点 4 字节附加数组。 memcpy 重指针算术
（`starts A×4 → points B×16 → Z 16+8B → M 16+8B`）与逐字段读取相互印证。
坐标 < −1e38 时原生钳位为 `DAT_71a66f20` 占位值（读侧仅在标志置位分支执
行），读取器原样保留位模式不钳位。

**Path/segment 子对象结论**：Polyline 记录是长度前缀的内联紧凑格式，流内
**没有** Path/Line/CircularArc/BezierCurve 独立对象头（全流 CLSID 扫描 0
实例）；段仅以「段描述数组条目」形态存在于标志 `0x20000000` 变体中，而该
标志在 Load 路径恒不可达。Path/segment 的 IPersistStream 体在样本中无事可
做，按「样本实际出现」为零不闭合（见下同族表）。

## 同族类（wrapper 骨架闭合；记录内部布局未开放）

同一 wrapper 骨架由族内多类共享（定位链同法，条件：COL 锚定虚表 +
`+0x0c` 槽函数体内含本类 CLSID 常量引用）：

| 类 | CLSID（esri.clsid 声明 Geometry.dll） | 持久化虚表 | GetClassID | Load | Save | IsDirty | 证据强度 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Polyline | `{30707210-52D5-11D0-A8F2-00608C85EDE5}` | `0x71a22bd4` | `0x719402d0` | `0x7192db20` | `0x7192dcb0` | `0x719d3220`（共享恒 0） | **确定**（反编译 + 样本 16/16） |
| Polygon | `{00A5CB42-52DA-11D0-A8F2-00608C85EDE5}` | `0x71a220d4` | `0x7193b170` | **同 Polyline `0x7192db20`** | **同 `0x7192dcb0`** | `0x719d3220` | **确定**（Load/Save 与 Polyline 同函数；记录布局无样本） |
| Point | `{00A5CB41-52DA-11D0-A8F2-00608C85EDE5}` | `0x71a21010` | `0x71915540` | `0x7191c1a0`（栈缓冲变体：声明长度 > 0x50 直接返回 `0x80004005`；导入槽 `+0x18` 非 `+0x38`） | `0x7191c2b0` | `0x719d3220` | 确定（反编译） |
| Multipoint | `{00A5CB40-52DA-11D0-A8F2-00608C85EDE5}` | `0x71a2010c` | `0x718ff700` | `0x718ff9c0`（malloc 变体，导入槽 `+0x18`） | `0x718ffb50` | `0x719d3220` | 确定（反编译） |
| Path | `{30707211-52D5-11D0-A8F2-00608C85EDE5}` | `0x71a203bc`（候选） | `0x719089d0` | `0x719089e0` | 空桩 `0x71973ab0` | `0x71909270`（非共享） | 候选（Save 空桩 + IsDirty 非共享标记，类身份证据弱） |
| Ring | `{B9BC6BCE-7891-11D0-82F4-0000F8034032}` | `0x71a2394c`（候选） | `0x7195e100` | `0x7195e110` | 空桩 | 非共享 | 候选 |
| BezierCurve | `{C8970BDC-B15B-11D1-A2D3-0000F8775FBF}` | `0x71a12944`（候选） | `0x717fea50` | `0x717fea60` | 空桩 | `0x718551c0`（与 EllipticArc 共享） | 候选 |
| EllipticArc | `{E54925BB-16D9-11D2-AA7A-00C04FA37BA4}` | `0x71a14c14`（候选） | `0x7184cba0` | `0x7184cbb0` | 空桩 | `0x718551c0` | 候选 |
| Line | `{6F4E63E0-5AB4-11D0-A8F6-00608C85EDE5}` | `0x71a1ea10`（候选） | `0x718d4de0` | `0x718d4df0` | 空桩 | `0x718da300` | 候选 |
| CircularArc | `{CD3A614E-BC4B-11D0-8303-0000F8034032}` | 未定位 | movq 站点 `0x717e8d73` 一带 | — | — | — | RTTI 名称串未在两种命名形态中命中，未继续 |

「Save 空桩」类的解读（候选）：段/路径类从不独立持久化（嵌在 Polyline/
Polygon 记录内），其 IPersistStream::Save 可能从不被调用，留空桩合理——但
这一解释未验证。各族记录字节块内部差异（Point 无路径数组、Polygon 带环
方向等）属内核解析器形状分支，样本仅覆盖 Polyline，故本读取器仅分派
Polyline，其余类返回 Unsupported。

## 样本对位（辅助观察）

/Maps 全流 Geometry.dll 模块 CLSID 扫描：**Polyline 16 实例**（键 1618、
1855、1862、1868、1874、1940、1946、1952、2017、2023、2088、2094、2100、
2165、2171、2177；对象头 @`0x18724`、`0x1dc0c`、`0x1ded9`、`0x1e13a`、
`0x1e39b`、`0x1fb2b`、`0x1fd90`、`0x1fff5`、`0x2155f`、`0x217c4`、
`0x22f8e`、`0x231f3`、`0x23458`、`0x249c2`、`0x24c27`、`0x24e8c`）、
Envelope 25 实例（已由 spatial-reference/envelope 轮闭合）、其余几何类 0
实例。

16/16 实例逐字节走查（走查脚本在 temp，未入库）全部吻合：版本 2、声明长
度 80、记录 = 类型 3 + 包络（水平引线：Y0=Y1，包络四角即两点）+ 路径数 1
+ 点数 2 + 路径起始 `[0]` + 2 个二维点，其后 16 字节全零空标记（版本 2
LoadObject ISpatialReference 空引用）。

实例 1（键 1618 @`0x18724`）之后的流内锚点：

| 偏移 | 内容 | 互证 |
| --- | --- | --- |
| `0x1879e` | int32 0、int32 −1 | TextElement::Load 尾段（字段 7 版本 >2、字段 8 版本 >3）；−1 与 TextElement::Save「恒写 −1」一致 |
| `0x187a6` | int32 ×3 = (34, −1, 0) | ElementCollection 版本 4 每元素交错值（各实例该值不同，−1/0 恒定） |
| `0x187b2` | f64 5393.0 + String `cb=20` `"JZD1 注记 2"` | 容器级注记类块（实例 2 @`0x1dc9a` 同构：5000.0 + `"JZD1 注记 6"`）；归属容器层，非本读取器范围 |
| `0x187d2` | FeatureLayer `{E663A651-8AAD-11D0-BEC7-00805F7C4268}` 键 1619 对象头 | 下一容器条目（esri.clsid 声明 progID `FeatureLayer`） |

第二实例链（TextElement 键 1850 @`0x1da39`，Polyline 键 1855
@`0x1dc0c`）同构互证；Polyline 记录解析值与 envelope 边界完全一致（点的
包络）。

## 实现（src/objects/geometry.rs、tests/geometry.rs）

- `read_geometry_body_of_class_10_1(class_id, reader, max_bytes, table)`：
  类分派入口（调用者已读对象头并登记新键）；仅 Polyline 闭合，其余几何类
  返回 Unsupported（游标不推进、登记表不修改）。
- `read_polyline_body_10_1`：版本 ≤2（有符号 >2 拒绝 `0x80004005`，负值一
  并拒绝较原生更严）→ int32 声明长度（先限额 `max_bytes` 后读取，原生无
  上限较原生更严）→ 记录块在 `read_region` 有界子窗口上解析（截断显式报
  错）→ 版本 >1 的 LoadObject 空间参考条目（Null/回引/新键投影坐标系复用
  `spatial_reference` 读取器；其余类 Unsupported——嵌套体无外部长度字段，
  与 envelope 读取器同语义）。
- `read_polyline_record_10_1`：类型 0 空几何（恰 4 字节，类型非 0 拒绝较
  原生更严）或 `44 + 4×路径数 + 16×点数` 严格闭合（不一致拒绝较原生更
  严）；点数 1 拒绝（原生 `0x80040221` 同拒）；路径起始与点坐标原样保留
  位模式。
- 事务性：trial 游标 + trial 登记表，完整成功才提交；空间参考新键登记先
  于嵌套体解析（登记先于持久化的读侧对偶），失败整体回滚。
- 测试（13 项）：类标识常量逐字节、样本键 1618 记录块字节形态镜像（期望
  值从同一 fixture 解码保证按位相等）、合成记录往返、空几何、版本 1 无空
  间参考段、回引/新键 PCS 嵌套、版本门与负值位模式、max_bytes 限额、全前
  缀截断事务性、长度/计数一致性、未知嵌套类回滚、分派闭合与未开放类拒
  绝。

## 接线建议（本轮不改共享文件，供整合轮）

1. `src/objects/element_body.rs` 的 `read_text_element_body_10_1` 第 6 字段
   「IGeometry 文本几何槽」现为 `read_classified_slot` 仅分类（新对象 = 停
   止点，样本 16 实例全部停在此）。建议改走
   `objects::geometry::read_geometry_body_of_class_10_1` 分派（形态参照同
   文件的 `read_color_slot`/`read_symbol_slot` 辅助函数）：Polyline 整体解
   析（无停止点，记录定长、空间参考段可闭合）；其他几何类维持既有停止语
   义（`ElementBodyStop10_1::NestedObjectNotClosed`，游标回退到槽对象头起
   点、键不登记）。
2. MarkerElement 的 IPoint 槽与 LineElement 的 IGeometry 槽样本 0 实例，
   建议维持仅分类，待同族记录布局闭合后再接入同一分派。
3. Polyline 嵌套空间参考为非投影坐标系新对象时按 envelope 读取器同语义返
   Unsupported 硬错误（整元素体失败、游标登记表均不提交）而非元素停止
   点——与既有 envelope 嵌套语义一致，整合轮无需特判。
4. 停止点类型无需新增：Polyline 体读取要么完整成功、要么错误上抛，无样本
   内停止点。

## Polyline/Polygon 的形状对象装配

证据来自 ArcMap 10.1 的 `Geometry.dll`（10.1.0.3035，SHA-256
`c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`）。
以下地址为 PE 首选加载地址。使用 `objdump -d -Mintel` 检查调用和赋值，
从 PE 虚表前的 RTTI Complete Object Locator 核对接口偏移及类名。

- Polyline/Polygon 持久化接口位于完整对象 `+0x2c`（虚表 `0x71a22bd4` /
  `0x71a220d4`）。Load `0x7192db20` 在 `0x7192dc1e` 调用 `this−8`
  接口的 `+0x38` 槽。导入接口位于完整对象 `+0x24`（虚表
  `0x71a22b7c` / `0x71a2207c`），其槽指向 `0x7192c950`。因此导入入口的
  `[this+0x9c]` 是完整对象 `+0xc0` 的形状指针，而 `[this+0xa4]` 是
  完整对象 `+0xc8` 的单字节模式标志。
- 默认构造在 `0x71942f2e` / `0x7193e882` 保存形状指针，在
  `0x71942f68` / `0x7193e8be` 将标志清零。两类形状的虚表
  `0x71a245e4` / `0x71a244ac` 分别对应 RTTI `ESRI::ShapePolyline` /
  `ESRI::ShapePolygon`。标志为零时导入跳到 `0x7192cbbb`，直接经形状
  虚表 `+0x94` 调用 Parse。
- 编辑转换在 `0x719429a7` / `0x7193e1ec` 换入由 `0x7184b9a0` /
  `0x718450e0` 构造的编辑形状，在 `0x719429bb` / `0x7193e2c8` 将标志
  置一。编辑形状的虚表 `0x71a1455c` / `0x71a13eac` 分别对应 RTTI
  `ESRI::EditPolyline` / `ESRI::EditPolygon`。标志置一且导入标志含
  `0x100` 时，栈上形状经 `+0x94` 解析成功后在 `0x7192cac3` 移交。
  两类编辑对象的 `+0x1cc` 槽均为 `0x7183c0c0`，接收者通过
  `ECX` 传入，形状地址通过栈传入。
- 编辑模式下若导入标志缺 `0x100`，`0x7192cafe` 分支会释放原形状，
  在 `0x7192cb8f` 换入默认形状并清除模式标志，随后直接 Parse。该分支
  使用不同的导入参数，MXD 持久化 Load 的调用点固定传入 `0x100`。

以上为静态对位。尚未在 ArcMap 进程中记录参考工程的形状模式、实际接收者
或调用栈。

## 未解决事项（开放项单列）

1. 记录类型值的语义：内部 shape 类别（ShapePolyline 标记 3、ShapePolygon
   5、0 空几何）与 esriGeometryType 枚举（类型库存在该枚举但 dump 未含成
   员值）的映射仅候选；读取器原样保留不校验（与原生一致）。
2. 版本 0/1、AfStreamRead S_FALSE 捷径（返回 1 的「空流」分支）无样本佐
   证；边界与 S_FALSE 已有[原生函数运行取证](#四类几何的-v0v1-和-s_false)。
3. 可选段（Z/M/段描述/每点附加数组）在 Load 路径恒不可达（标志恒 0），仅
   反编译记录；Save 侧标志 `0x400`（docVer > 1）的导出差异未展开。
4. Point/Polygon/Multipoint/Path/Ring/BezierCurve/EllipticArc/Line/
   CircularArc 的记录字节块内部布局未开放；Path 等「Save 空桩」类的持久
   化虚表归属为候选。
5. [形状对象装配](#polylinepolygon-的形状对象装配)已静态定位默认和编辑两条路径。
   ArcMap 10.1 进程中实际命中的路径、调用栈和内核校验行为待运行取证。
6. < −1e38 坐标的原生钳位（`DAT_71a66f20`）未在读取器中实现（原样保留）。
7. 嵌套空间参考仅支持投影坐标系；GeographicCoordinateSystem、
   UnknownCoordinateSystem 等无样本与分派证据。
8. 全部结论为静态观察；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已验
   证。

## 接线轮落位（2026-11-15）

「接线建议」1 已落位：`element_body::read_text_element_body_10_1` 第 6 字段
IGeometry 槽改经 `read_geometry_body_of_class_10_1` 类分派（新
`GeometrySlot10_1` 槽枚举，Polyline 体整体解析入 `Polyline` 变体；其余几何
类维持 `NestedObjectNotClosed` 停止回退）。Polyline 体内空间参考未知类按建
议 3 保持硬错误上抛（分派前先比对 Polyline 类标识，不特判）。建议 2 照办：
MarkerElement IPoint 与 LineElement IGeometry 槽维持仅分类（样本 0 实例）。
样本 16 实例位于 CGL 图形集合（流内偏移在本轮样本停止点之后），接线不改变
当前样本停止点；`read_polyline_body_10_1` 与类分派入口已入 robustness 集中
清单。

## 第二轮落位：Polygon/Point/Multipoint 记录布局闭合与 Path 族「持久化不存在」结论（2026-11-15）

本轮在上一轮工程副本（同一临时目录内独立命名工程，Open 副本不共享锁）
继续取证；全部结论为**静态反编译/汇编核对 + 机械字节扫描**，未运行
ArcMap。样本 0 实例的类全部由合成测试覆盖，不声称样本验证。不写本机绝对
路径。

### 类身份复核（esri.clsid 表）

- `30707211-52D5-11D0-A8F2-00608C85EDE5` = **Path**（progID `Path`，非
  Polygon）；Polygon/Point/Multipoint = `00A5CB42/41/40-52DA-11D0-…`。上轮
  同族表与 esri.clsid 声明一致，本轮以全表 XML 复核维持。
- CLSID 常量独立字节扫描（不经 Ghidra，直读 PE 节表换算 VA）：Polyline
  `0x71a1187c`、Multipoint `0x71a1186c`、Polygon `0x71a125f8`、Point
  `0x71a13a78`、Path `0x71a131d0`，各唯一命中。
- RTTI TD 名称串（独立字节扫描）：`.?AVPolygon@ESRI@@`、`.?AVPoint@ESRI@@`、
  `.?AVMultipoint@ESRI@@`、`.?AVPath@ESRI@@`、`.?AVRing@ESRI@@`、
  `.?AVLine@ESRI@@`、`.?AVBezierCurve@ESRI@@`、`.?AVEllipticArc@ESRI@@` 均
  唯一命中并锚定对应虚表；`.?AVPolyBase@ESRI@@`（共享基类，见下）唯一命中；
  **`.?AVCircularArc@ESRI@@` 与 `.?AVCircularArc@@` 两种命名形态均未命中**
  （维持「虚表未定位」）。

### Polygon：与 Polyline 完全同构（证据：确定的）

- Polygon 持久化虚表 `0x71a220d4` 的 Load/Save 槽 = `0x7192db20` /
  `0x7192dcb0`（与 Polyline 同函数，上轮已证，本轮虚表槽直读复核）。
- **解析器为同一函数**：`ESRI::ShapePolyline::vftable`（基址 `0x71a245e4`）
  与 `ESRI::ShapePolygon::vftable`（基址 `0x71a244ac`）的 `+0x94` 槽直读均
  为 `0x7197fc50`。导入入口 `0x7192c950` 的编辑模式按几何类目（4 =
  Polygon）构造 `ShapePolygon` 栈对象（标记 5）。解析逻辑与 Polyline 分支
  共用同一代码：
  字段布局、空几何（4 字节、回填标记按分支取 3/5）、负计数拒
  `0x80040253`、**点数 1 拒 `0x80040221`（无条件尾部检查，不分区分类别）**
  完全一致。
- 上轮「Polygon 带环方向等差异属内核解析器形状分支」的候选：本轮反编译
  证实**记录布局层面无环方向处理**——多环即「环数 A + 环起始数组（每环首
  点序号）+ 点序」，与折线路径同构；环方向和闭合在内核中的处理仍待核对。
- 包装字段序（版本 ≤2、声明长度、版本 >1 的 LoadObject ISpatialReference）
  与 Save 的固定版本常量 `02 00` 全部与 Polyline 相同（同一函数）。

### Point：记录布局（Load `0x7191c1a0` → 导入 `0x7191bd30`；反编译逐行核对，
确定的；样本 0 实例）

- wrapper 变体：版本门同 Polyline（有符号 >2 拒 `0x80004005`；S_FALSE 空
  流捷径同样存在、读取器不实现）；**声明长度 > `0x50` 直接拒
  `0x80004005`**（80 字节栈缓冲，无符号比较）；内嵌接口在对象 `+0x40` 槽
  （持久化 this−0x8 视角，虚表 `0x71a20fbc`），导入 = `+0x18` 槽
  `0x71906090`（薄转发，补写标志 0）→ `+0x38` 槽 `0x7191bd30`。版本 >1 的
  LoadObject ISpatialReference 段与 Polyline 相同（IID `DAT_71a15a10`）。
- Save `0x7191c2b0`：版本常量 `0x71a11e54` 字节 `02 00`（本轮直读），导出
  对称（`+0x10` 查尺寸、`+0x24` 填充、写长度与记录、SaveObject 空间参考）。
- 记录布局（小端；坐标 < −1e38 原生钳位 `DAT_71a66f20`，读取器原样保留；
  无包络、无路径概念，X/Y 直接读自记录）：

| 类型字（全字匹配） | 声明长度 | 字段 |
| --- | --- | --- |
| 恰 0 | 4 | 空点（原生 `*consumed = 4`） |
| 恰 1 | 20 | f64 X（偏移 4）+ f64 Y（偏移 12） |
| 恰 9 | 28 | + f64 Z（偏移 20） |
| 恰 0xb | 36 | + f64 Z + f64 M（偏移 28） |
| 恰 0x15 | 28 | + f64 M（偏移 20） |
| 低字节 0x34 且 `& 0x2f000000` == 0 | 20 + 8·Z位 + 8·M位 + 4·ID位 | X + Y + 可选 Z（`0x80000000`）+ 可选 M（`0x40000000`）+ 可选 ID（`0x10000000`，u32） |

- 掩码说明：`0x2f000000` = 0x01000000/0x02000000/0x04000000/0x08000000/
  0x20000000，**不含** `0x10000000`——ID 段可达（初判误读为不可达，复核反
  编译后更正）。恰值 1/9/0xb/0x15 带高位即落默认分支（低字节 ≠ 0x34 拒）。
- 读取器较原生更严的三点：声明长度与布局常量严格相等（原生允许死空间尾
  随）、空点恰 4 字节、其余类型字同拒 `0x8004020a`。

### Multipoint：记录布局（Load `0x718ff9c0` → 导入 `0x71906590`；反编译逐
行核对，确定的；样本 0 实例）

- wrapper 变体（malloc 版，原生无长度上限）：内嵌接口在对象 `+0x3c` 槽
  （持久化 this−0x20 视角，虚表 `0x71a1ffa4`），导入 = `+0x18` 槽
  `0x71906090` → `+0x38` 槽 `0x71906590`。版本门与 LoadObject 段同
  Polyline；Save `0x718ffb50` 版本常量 `0x71a11e58` 字节 `02 00`。
- 记录布局按**类型字低字节** switch（高标志位按全字判别）：

| 低字节 | 声明长度 | 字段 |
| --- | --- | --- |
| 0 | 4 | 空点集（原生完全不读内容、consumed 不回写；读取器要求恰 4 较原生更严） |
| 1/9/0xb/0x15/0x34 | 20 + 8·Z + 8·M + 4·ID | f64 X（偏移 4）+ f64 Y（偏移 12）；Z 段 ⇔ 恰为 9/0xb 或 `0x80000000` 位；M 段 ⇔ 恰为 0x15/0xb 或 `0x40000000` 位；ID（u32）⇔ `0x10000000` 位；判别按全字（如 `0x40000009` = 有 M 无 Z） |
| 8/0x12/0x14/0x1c/0x35 | 40 + 16N + (16+8N)·Z + (16+8N)·M + 4N·ID | 4×f64 包络（偏移 4）+ int32 点数 N（偏移 36）+ N×(f64,f64)（偏移 40）+ Z 段（16 字节 min/max 对 + 每点 f64）⇔ 恰为 0x14/0x12 或 `0x80000000` 位 + M 段 ⇔ 恰为 0x1c/0x12 或 `0x40000000` 位 + 每点 u32 ID ⇔ `0x10000000` 位；段序 Z → M → ID |

- 单点型包络为写侧退化形 `(x, y, x, y)`（`WKSEnvelope::PutCoords` 四参同
  值）；多点型为完整 min/max。其余低字节原生拒 `0x8004020a`。
- 读取器较原生更严的三点：声明长度严格相等（空点集恰 4）；多点型点数负位
  模式拒绝（原生静默跳过建点继续读尾随段，无显式错误）；长度算术用 u64
  防溢出。

### Path/Ring/Line/BezierCurve/EllipticArc：流内持久化不存在（结论，非缺口；
本轮更正上轮表）

- 上轮表的「Load」列（`0x719089e0` 等）反编译为 **GetClassID 形态函数**
  （判空 `0x80004003` → movq 双 8 字节复制本类 CLSID 常量 → S_OK，不读
  流）；「Save 空桩 `0x71973ab0`」为其所在**类身份虚表**尾槽。该虚表形状
  `[QI、共享 AddRef 0x719f1550、共享 Release 0x718de060、类名 getter
  （L"Path"/L"Ring"/L"Line"…）@+0x0c、私有槽 @+0x10、GetClassID movq
  @+0x14、空桩 @+0x18、…]` 与 Polyline 主虚表 `0x71a228c4` 同族（非持久
  化虚表）。
- 全镜像（.rdata + .data）4 字节扫描：五类 movq-GetClassID 各仅出现 1 次
  （位置即类身份虚表 +0x14 槽）——不存在持久化形状虚表（无第二处以 movq
  居 +0x0c 位）。
- 构造器逐指令核对（Path `0x719091f0` 一族 5 处、Ring `0x71961d70`、Line
  `0x718da350`）：只安装类身份 + 能力接口虚表（两阶段覆写），**无任何持久
  化虚表安装点**。Polyline/Polygon 共享的基类持久化虚表 `0x71a2173c`（COL
  → TD `.?AVPolyBase@ESRI@@`，GetClassID = purecall `0x719f36fe`，Load/
  Save = `0x7192db20`/`0x7192dcb0`）从未被这三族构造器安装——「两类共用
  Load/Save」的解释即派生类覆写 GetClassID 而继承 PolyBase 持久化实现。
- 结论：五类**不实现 IPersistStream**（构造性不存在），从不独立出现在流内
  （样本 0 实例两轮一致）；段仅以标志 `0x20000000` 变体的描述条目存在，而
  该标志 Load 路径恒不可达。上轮「Save 空桩」解读作废，按本节更正；未解决
  事项 4 相应闭合（CircularArc 维持未定位）。

### 实现落位（src/objects/geometry.rs、tests/geometry.rs）

- 新入口：`read_geometry_body_10_1(class_id, …) -> GeometryBody10_1`（四类
  枚举 `Polyline/Polygon/Point/Multipoint`）；`read_polygon_body_10_1`、
  `read_point_body_10_1`、`read_multipoint_body_10_1` 显式 10.1 入口；常量
  `POLYGON_CLASS_ID`/`POINT_CLASS_ID`/`MULTIPOINT_CLASS_ID`。
- 兼容入口 `read_geometry_body_of_class_10_1`（返回具体
  `PolylineBody10_1`）**签名与行为不变**：`element_body.rs` 的 IGeometry 槽
  接线点按 Polyline 类标识调用并取具体类型，签名变更属整合轮；其余类错误
  文案改为提示类型化入口。`PolylineRecord10_1` 变为共享 `ShapeRecord10_1`
  的类型别名（既有调用不受影响）。
- 共享组件：版本门（`MAX_GEOMETRY_VERSION` ≤ 2 有符号，负位模式一并拒）、
  声明长度门（`max_bytes` 先限额后读取）、嵌套空间参考段（Null/回引/新键
  投影坐标系，其余 Unsupported 整体回滚——四类同语义）；Point 另有 `0x50`
  wrapper 门（同原生 `0x80004005`）。
- 测试（tests/geometry.rs 13 → 24 项）：三个新 CLSID 常量逐字节、Polygon
  多环（环数 2 + 起始 `[0, 3]` + 5 点）往返、空几何与共享门（单点/长度/版
  本）、兼容入口仍仅路由 Polyline、Point 类型字表（8 例含 ID 段）、空点与
  拒绝（死空间、未知类型、掩码位、`0x50` 门）、Point 全前缀截断事务性、
  Multipoint 单点型 4 例（含 `0x40000009` 全字判别与 ID 位）、多点型 4 例
  （类型 8、`0x14` +Z、`0x12` +Z+M、`0x10000008` +ID）、空点集与拒绝（低字
  节 0 死空间、未知低字节、负点数、长度不一致）、类型化分派四类路由与未知
  类回滚。
- `cargo fmt --check`、`cargo test`（493 项）全绿；既有 13 项 Polyline 测
  试与 robustness/element_body 集中清单不改动、不回退。

### 样本复核（全流重扫）

/Maps 流（152,941 字节）以独立脚本重扫 12 个几何类 CLSID：Polyline 16 实
例（16 处对象头偏移与上轮表逐一相同）、Envelope 25 实例、Polygon/Point/
Multipoint/Path/Ring/Line/BezierCurve/EllipticArc/CircularArc/MultiPatch 均
**0 实例**。本轮新增的三个读取器（Polygon/Point/Multipoint）无字节佐证，
全部由合成测试覆盖，不得声称样本验证。

### 接线建议（本轮不改共享文件，供整合轮）

1. `element_body.rs` 的 `read_geometry_slot`（TextElement 第 6 字段）当前在
   分派前先比对 `POLYLINE_CLASS_ID`。切换为类型化入口
   `read_geometry_body_10_1`：`GeometryBody10_1` 四变体分别入
   `GeometrySlot10_1`（建议新增 `Polygon`/`Point`/`Multipoint` 槽变体），
   未知类维持 `NestedObjectNotClosed` 停止回退语义。
2. MarkerElement 的 IPoint 槽与 LineElement 的 IGeometry 槽样本 0 实例；若
   接入 Point 分派，注意其体无停止点、错误上抛语义与 Polyline 一致。
3. `tests/robustness.rs` 集中清单建议补 `read_polygon_body_10_1`/
   `read_point_body_10_1`/`read_multipoint_body_10_1`/
   `read_geometry_body_10_1` 四入口（试探性输入 → 明确错误）。

### 未解决事项（本轮更新，承接上轮清单）

1. 记录类型值语义（shape 类别 ↔ esriGeometryType 映射；Point/Multipoint 的
   0x34/0x35 等标记）仍为反编译观察候选——新增三类的类型字同样原样保留不
   校验（与原生一致）。
2. 四类的版本 0/1、S_FALSE 均已补入[原生函数运行取证](#四类几何的-v0v1-和-s_false)，仍缺实档。
3. 折线/多边形可选段（Z/M/段描述/附加数组）Load 路径恒不可达；Point/
   Multipoint 类型字高标志位的 Z/M/ID 段已按反编译闭合（合成测试覆盖），
   亦无样本佐证。
4. （原第 4 项部分闭合）Point/Polygon/Multipoint 记录布局已闭合；Path/
   Ring/Line/BezierCurve/EllipticArc 的流内持久化按「构造性不存在」闭合；
   CircularArc 虚表与 RTTI 名称维持未定位。
5. 几何内核移交链（环方向、起始合法性校验等内核语义）静态未定位。
6. < −1e38 坐标钳位未实现（原样保留位模式）——四类同。
7. 嵌套空间参考仅支持投影坐标系——四类同。
8. Polygon/Point/Multipoint 布局证据为静态反编译（确定的），样本 0 实例：
   合成测试覆盖；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已验证。

## 接线轮落位（整合接线轮，2026-11-15）

「接线建议」1、2 已落位：

- **TextElement IGeometry 槽四类分派**（建议 1）：`element_body.rs`
  `read_geometry_slot` 自 Polyline 类标识比对改经
  `read_geometry_body_10_1` 类型化分派，`GeometrySlot10_1` 新增
  `Polygon`/`Point`/`Multipoint` 三变体；闭合四类之外的几何类维持
  `NestedObjectNotClosed` 停止回退（游标回槽对象头起点、键不登记——回退
  语义在槽位层按类集合守卫，不依赖分派入口的 `Unsupported` 错误路径）。
  四类体自身错误上抛（含体内空间参考未知类）按建议 3 保持不变。
- **LineElement IGeometry 槽**（建议 2 延伸）：与 TextElement 共用同一
  `read_geometry_slot`/`GeometrySlot10_1`（同请求 IID，样本 0 实例）。
- **MarkerElement IPoint 槽**（建议 2 裁决：接入）：Point 布局已闭合，接
  `read_point_body_10_1` 单类分派（新 `PointSlot10_1` 槽枚举；体无停止点、
  错误上抛语义与 Polyline 一致）；其余实现类停止回退。样本 0 实例，接线
  契约由合成测试固定。
- **robustness**（建议 3）：`read_polygon_body_10_1`/`read_point_body_10_1`
  /`read_multipoint_body_10_1`/`read_geometry_body_10_1` 四入口并入
  `tests/robustness.rs` 集中清单。
- 样本重走：停止点维持 `0x155a`；16 处 Polyline 实例位于 CGL 图形集合
  （流内偏移在当前样本停止点之后），接线不改变样本行为。`cargo fmt
  --check` 与全量 `cargo test` 绿；未声称运行验证。

## 四类几何的 v0/v1 和 S_FALSE

2026-11-19 从研究目录取 Geometry.dll 10.1.0.3035，校验 SHA-256 与本文方法节
一致。在 QEMU i386 中按原地址执行 Point Load `0x7191c1a0`、Multipoint
Load `0x718ff9c0`、Polyline/Polygon 共用 Load `0x7192db20`。真实
SystemCore.dll ObjectStream Read `0x7ffcf520`（10.1.0.3035，SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）
及 AfCore `AfStreamRead` 转发器 `0x68a84830` 夹在对象与受控 IStream 之间，
后者按剩余字节数返回完整读或短读。内存分配、记录导入接口和几何内核接口
由测试提供。记录导入替身逐字节核对传入缓冲和长度，并验证折线、多边形
的导入标志为 `0x100`，不执行图形内核的导入逻辑。

v0/v1 两组各有 Point XY、三点 Multipoint、单路径 Polyline 及单环 Polygon。
原生各完成一次版本读取、一次长度读取和一次记录读取，四类都在记录末尾
返回 S_OK。Rust 读取同一输入，核对类型字、XY、包络、路径起点及对象后的
哨兵，四类均没有空间参考槽。对应的两组完整体长度均为
26、94、86、102 字节。616 个逐字节截断前缀均报错，保持游标及已有键。

复核 RTTI 与 10.1 类型库的 `IGeometry::SetEmpty`。四个类在完整对象
`+0x10` 的 IGeometry 视图通过虚表 `+0x20` 分派，具体包括 Point
`0x719158b0`、Multipoint `0x718ffd90`、Polyline/Polygon 共用的
`0x71922e10`。测试在版本字段的零字节和一字节两种短读中运行这些
原生函数。两种情况均在唯一一次 Read 后进入 SetEmpty，保持短读实际
消费量并返回 S_FALSE。未请求长度、分配缓冲或读空间参考。Point 写入
内部空状态，Multipoint 在已走过 Load 前置清理后再次调用集合清理，
折线和多边形清理集合后转入几何内核清空槽，测试确认该槽被调用。

字节解析入口把不完整的版本字段报告为截断，保持对象游标和引用表。
完整的空几何仍可由正常记录中的空类型字表达。受控运行与对照脚本在
`temp/mxd-geometry-v0/`。八条带类型和长度的构造记录输出 SHA-256 为
`6e1151fbe2b283a72b6e8040a1e9eaae272762fe552999b77e8762a0dc8c693c`。
未找到对应的真实旧 MXD，ArcMap/QGIS 人工验收尚未进行。

### 图形元素 IPoint 回引

MarkerElement 的几何槽和 Text3DElement 的锚点槽请求 IPoint。分流
ObjectStream 的共同回引路径在 SystemCore.dll 10.1.0.3035
的 `0x7ffd0063` 按登记实例查询接口。Rust 在共享 IPoint 槽检查
登记类，登记信息缺失时回退到流内类标识。构造测试验证流内头写
Point 而实际键登记为 Polyline 时停在几何槽起点，保留原有登记。
ArcMap/QGIS 人工验收仍待执行。
