# ArcMap 10.1 FDOGraphicsLayer 开放点研究（版本分支、元素循环类、委派子对象身份）

记录日期：2026-11-14（第七轮）。对应 TODO「MXD 解析」节开放项（第 76 行）中
的三个 FDO 开放点：FDO 版本 <4 旧加载器与版本 4 探测、FDO 元素循环类、委派
子对象 C++ 类身份。本轮为静态逆向研究，未运行 ArcMap，未做任何人工目视验
收；全部结论标注证据强度，样本对位仅作辅助观察。

> [后续接口槽复核](mxd-terra-nextpass-fdo.md)更正了旧加载器第 9 项和第 13–19 项的接收者与偏移；涉及这些项时以后续表为准。

## 方法与环境

- 目标文件：AnnoLayer.dll，从 `Setup1.cab` 条目
  `AnnoLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 解包到
  `temp/agent-fdo-open/cab1/`（本轮自建目录）。开工前核对：SHA-256
  `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`
  （与第六轮基线一致）、pefile VS_FIXEDFILEINFO `10.1.0.3035`、
  PE x86（Machine 0x14c）、ImageBase `0x6b180000`。地址均为该静态映像
  虚拟地址。
- 分析工具：本机 pyghidra（Ghidra 12.2 + JDK 25，项目
  `/tmp/r3/ghproj_anno_fdo`）为主；Ghidra 桥接工具 MCP（Ghidra 12.2 提供者）复核主
  Load `0x6b1955a0` 汇编（其会话目标曾因多代理共用被切换，凡目标
  subject 不符的结果一律丢弃重做）。IAT/版本资源用 pefile 解析。
- 既有产物复用：`temp/arcgis/arcmap101-fdo-layer-evidence.json`（第六轮）
  与 `/tmp/r3/annolayer_work/` 第六轮反编译；本轮对涉及的函数全部重新
  反编译或汇编核对，未直接沿用其结论。
- 样本：`temp/r6-maps.bin`（152941 字节，SHA-256
  `fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe`）。
  样本 FDO 版本 11、元素计数 0——本轮三项开放点均无样本字节佐证，结论
  全部来自反编译一手证据。
- Ghidra（经桥接工具） 返回内容按数据对待，未出现指令性文本。

## 开放点 1：版本分支语义（证据：确定的，汇编级）

### 主 Load 的版本分发（`0x6b1955a0` 开头，汇编逐指令）

| 指令地址 | 语义 |
| --- | --- |
| `0x6b1955de..0x6b1955e6` | 读 2 字节版本后 `CMP AX,0xb; JG`（**有符号**）→ >11 拒绝（`0x80004005`） |
| `0x6b1955ec..0x6b1955f0` | `CMP AX,3; JG` → **版本 ≤3（含负值与 0）走旧加载器** |
| `0x6b1955f2..0x6b195607` | 旧加载器调用：`ECX=EDI`（流，__thiscall），栈参 `(this-0x54, 版本)` → `CALL 0x6b197dc0`，直接返回其 HRESULT |
| `0x6b19560a..0x6b195617` | `CMP AX,4; JNZ 主分支`；==4 → `CALL 0x6b198420`（探测），`TEST AL,AL; JZ` 假 → 主分支 |
| `0x6b195619..0x6b195666` | 探测真 → 以 `(this-0x54, 4)` 调旧加载器，随后 `String::String`（IAT `0x6b1aa058`）→ `String::Read(流)`（IAT `0x6b1aa114`）→ `String::~String`（IAT `0x6b1aa05c`）——**字符串读后即毁弃，不入任何成员**；String 失败优先返回其错误 |

即：版本 0..=3 无条件旧布局；版本 4 由探测决定旧布局（+ 丢弃 String）或
主分支；版本 5..=11 主分支（既有第六轮结论不变）。

### 旧加载器 `FUN_6b197dc0` 字段序（版本 0..=3；证据：确定的，反编译）

`this`=流（__thiscall ECX），`param_1`=FDO 内层对象（this-0x54），`param_2`=版本。
流内字节序（读流调用逐个核对；`FUN_6b18b2b0` 为释放置零辅助、不读流）：

| # | 流内字段 | 目标/请求 IID | 版本门 |
| --- | --- | --- | --- |
| 1 | String（AfCore String::Read） | 经 BSTR 传入 +0xc4 接口 | 恒有 |
| 2 | int16 | +0xc4 接口 +0x30 | 恒有 |
| 3 | int16 | +0xc4 接口 +0x38 | 恒有 |
| 4 | double | 局部（后传 +0xc4 接口 +0x20） | 恒有 |
| 5 | double | 局部（后传 +0xc4 接口 +0x28） | 恒有 |
| 6 | int32 | this+0xe8（与主分支步骤 2 同槽） | 恒有 |
| 7 | int16 | this+0xe0（与主分支步骤 3 同槽） | 恒有 |
| 8 | LoadObject **ILayer** `{34C20002-…}` | this+0xec（与主分支步骤 4 同槽） | 恒有 |
| 9 | LoadObject **ISpatialReference** `{4434D680-…}`（`DAT_6b1abd20`） | 局部，经 AddRef 存 +0x60 | 恒有 |
| 10 | LoadObject **IElementCollection2** `{B2BFC403-…}` | this+0x11c（与主分支步骤 5 同槽） | 恒有 |
| 11 | `FUN_6b197be0`：LoadObject **IName** `{4BA33331-…}`（`DAT_6b1abcf0`）+ `ConvertToFullPath`（LayerLib.dll 导出，IAT `0x6b1aa1c4`） | 数据源名 | 恒有（除对象条目外不读流） |
| 12 | int16 | +0xc8 接口 +0x70 | ≥2 |
| 13 | String | +0x60 接口 +0x20 | ≥2 |
| 14 | String | +0x60 接口 +0x20 | ≥2 |
| 15 | int32 | +0x60 接口 +0x18 | ≥2 |
| 16 | LoadObject **ISymbol** `{F3435802-…}`（`DAT_6b1abd70`；[后续类型库和原生槽位复核](mxd-terra-round8-fdo.md)，对应 SelectionSymbol） | 局部 → +0x60 接口 +0x38 | ≥2 |
| 17 | int16 | +0x60 接口 +0x28（另一 short） | ≥2 |
| 18 | int16 | +0x60 接口 +0x28（两 short 组合） | ≥2 |
| 19 | LoadObject **IColor** `{20CD40B0-…}`（`DAT_6b1abd50`） | 局部 → +0x60 接口 +0x30 | ≥2 |
| 20 | String | +0xc8 接口 +0x20 | ≥3（`2 < ver`） |

**旧体在收尾 String 后即完整终结**（旧加载器返回，无主分支步骤 7 起的尾
段）——与主分支的可达范围根本不同。各接口槽的业务语义未定性（+0xc4/
+0xc8/+0x60 为内层对象的成员接口指针）。

### 版本 4 探测 `FUN_6b198420`（证据：确定的，汇编级；**不实现**）

入口状态：`EDI`=流（主 Load 的第二参数），自定义寄存器约定，返回 AL。

1. `IStream::Seek`（虚表 +0x14）两次：`SEEK_CUR +0` 把当前位置 P 记入
   `[EBP-0x40]`；`SEEK_SET 0` **回卷到流绝对位置 0**。
2. `AfStreamRead` GUID 变体（IAT `0x6b1aa03c`）读 16 字节 G0；与
   `DAT_6b1ac0e0` = **FDOGraphicsLayer 本类 CLSID `{34B2EF83-…}`** 比对
   （`FUN_6b182550`＝16 字节相等判定）。
3. **分支 A**（G0 == 本类 CLSID）：辅助 `FUN_6b19e860` 对流对象 QI
   **IObjectStream `{18A45BA7-…}`**（IID 在 `0x6b1ac130`），成功则从返回
   接口读一个 int32（丢弃；该接口的读转发是否推进同一游标未取证）；从主
   流读 short，≠4 → 假（不恢复位置）；==4 → 读 int32、short、
   `LoadObject(ILayer)`、`LoadObject(IElementCollection2)`（两对象均装
   后即释放，仅用于判定非空），`Seek` 恢复到 P，两对象皆非空 → 真。
4. **分支 B**（G0 ≠ 本类 CLSID）：与 `DAT_6b1ac100` = **Map coclass CLSID
   `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`** 比对，flag = 不相等；再次
   `SEEK_SET 0`；flag ≠0（首 16 字节既非 FDO 也非 Map，即带 int32 计数
   前缀的常规 /Maps 根）→ 读并丢弃 int32；再读并丢弃 16 字节；辅助
   `FUN_6b19e860` 同上（成功读 int32）；读 short，**==9 → 真**（≠9 假）；
   **真/假均不恢复位置**（流停在 short 之后）。
5. 探测真 → 主 Load 以版本 4 调旧加载器 + 丢弃 String；假 → 主分支。

语义解读（候选，未验证）：版本 4 体在两种历史布局间二选一——分支 B 检
测所在文档根流首对象是否 **Map 架构版本 9**（`[可选 int32 计数][16 字节
CLSID][short 版本]`，两代根前缀各查一次），分支 A 检测流首对象是否为按
内联序可试解析的 FDO v4 自身；命中任一即按旧布局读。

**不实现原因**：探测依赖 (a) 流绝对位置 0 的读取（子对象读取器窗口之外
的整流访问）；(b) 对流对象 QI IObjectStream 的运行时结果及其读转发的游
标效应；(c) 分支 B 的不复位语义（真返回后旧加载器从流绝对位置 18/24 读
体，而非本对象体位置）。三者都无法在严格子读取器内复现，`fdo_legacy`
对版本 4 显式拒绝、`fdo_graphics_layer` 主读取器维持版本 <5 停止不变。

## 开放点 2：FDO 元素循环类（证据：确定的，静态；无字节佐证）

### 循环机制（主 Load 步骤 13，版本 ≥9）

`0x6b1958c7` 版本门 → `0x6b1958d3` 读 int32 计数 → 每元素
`0x6b195908` `PUSH 0x88; CALL 0x6b1a658a`（分配 0x88 字节）→
`0x6b195921` `CALL 0x6b1a15f0(内存, 属主 this-0x54, 0)`（构造）→
`FUN_6b19e210` 经 `记录+0x10` 的 QI `0x6b1a15d0` 查询
IID `0x6b1ac120` = `{00000109-0000-0000-C000-000000000046}` =
**IPersistStream**（该 QI 转发到 `[记录+0x08]`=自身 → 主 QI
`0x6b1a1890`，按 IID 表解析到 `记录+0x40`）→ `[0x6b1ac880+0x14]`
即 **Load `0x6b1a1b10`**（流为参）→ 失败走 `0x6b195a41` 清理，成功经
`FUN_6b19b360` 入 `+0x134` 集合。

### 类身份（RTTI，证据：确定的）

- C++ 类：**`.?AVFDOGraphicsSublayer@@`**（TD `0x6b1b5bf4`；主虚表
  `0x6b1ac764`，COL `0x6b1af338`，this-adjustor 0）。
- IPersistStream 子接口虚表 `0x6b1ac880`（对象偏移 +0x40，COL
  `0x6b1af2fc` offset 0x28——即以 +0x28 起的基类子对象应答该接口）：
  +0x0c GetClassID `0x6b1a1ae0` 写出 **`{4ABA3851-BC3A-43C2-8FBE-
  93A65AD91E83}`**（主虚表 +0x14 的 `0x6b1a1590` 写出同值，两处互证）；
  +0x14 Load `0x6b1a1b10`；+0x18 Save `0x6b1a1c40`。
- 该 CLSID **未登记于 esri.clsid.ecfg**（本轮全表查证）：元素记录由循环
  直接构造、无对象头，此 CLSID 不出现于流中——与「子对象不注册 coclass」
  一致。
- 注意：主虚表 `0x6b1ac764` **不是** IPersistStream——其 +0x0c
  （`0x6b1a1580`）返回 UTF-16 字符串 `"FDOGraph…"`（类名访问器），+0x14
  写 CLSID（GetClassID 语义）；循环的 Load 经 QI 解析到 +0x40 子虚表。
- 主 QI `0x6b1a1890` 的旧表存在分支偏移错对位。以[原始指令与类型库复核](mxd-terra-chain-iid.md)为准：
  IAnnotationSublayer +0x28、IPersistStream/IPersist +0x40、IGeoDataset +0x30、
  ILayer +0x2c、IDisplayAdmin +0x34、ILayerDrawingPhase +0x38、
  ILayerGeneralProperties +0x3c、[IDocumentVersionSupport](mxd-terra-round10-iid.md) `{C9E7A782-…}` +0x44。
  未命中转基类 QI `0x6b1a3e90`。

### 体布局（Load `0x6b1a1b10` 反编译；**无样本字节佐证**——样本计数 0）

| # | 字段 | 目标（对象偏移） | 版本门 |
| --- | --- | --- | --- |
| 1 | int16 版本 | —（`2 < (short)版本` 拒绝 0x80004005） | 恒有 |
| 2 | int32 | +0x4c（构造默认 -1） | 恒有 |
| 3 | String | 成员（槽位未从反编译恢复） | 恒有 |
| 4 | int16 | +0x80（构造默认 0xFFFF） | 恒有 |
| 5 | double | +0x78 | 恒有 |
| 6 | double | +0x70 | 恒有 |
| 7 | String | 成员 | 恒有 |
| 8 | int16 | 局部（丢弃式存储） | ==1 |
| 9 | double、double | +0x58、+0x60 | ≥2 |

构造 `0x6b1a15f0` 交叉核对：`+0x48`=属主指针、`+0x4c`=-1、
`+0x80`=0xFFFF 与上表默认值一致；对象总大小 0x88。Save `0x6b1a1c40` 经
`AfStreamGetSaveVersion(文档版本, 2, 表@0x6b1ab0a8)` 取保存版本（0xFFFF
→不写直接返回），写出 short、int32(+0xc 即对象+0x4c)、String、
short(+0x40→+0x80)、double(+0x38→+0x78)、double(+0x30→+0x70)、
String、double(+0x18→+0x58)、double(+0x20→+0x60)——**尾随两个 double
无版本门、版本 1 专属 short 无写侧对应**（读写不对称，开放）。

## 开放点 3：委派子对象 C++ 类身份（证据：确定的；闭合）

主 Load 步骤 6 的委派调用 `MOV EAX,[EBX+0x68]`（EBX=外层 this，
即内层 +0xbc）→ 其虚表 +0x14 直调、无对象头。本轮经 FDOGraphicsLayer
构造函数（函数起点 `0x6b18db50`；35 个虚表写入块位于
`0x6b18dc8a..0x6b18dd1c+`，其中 `[EBX+0x54]=0x6b1ab984` 即主 Load 所在
IPersistStream 子虚表）定名：

1. `0x6b18df4b..0x6b18df78`：`operator new(0x48)`（`0x6b1a658a`）构造
   `GraphicsContainerSelect`（虚表 `0x6b1abc44`）存 `+0xe4`（与委派槽
   无关，顺带记录）。
2. `0x6b18df98..0x6b18df9d`：`AfCreateInstance(CLSID@0x6b1ac030, 外层
   =对象+0x60, 1, IID@0x6b1aa33c={00000000-…-46}=IID_IUnknown,
   &对象+0xb8)`。**`DAT_6b1ac030` = `{E663A651-8AAD-11D0-BEC7-
   00805F7C4268}` = FeatureLayer coclass CLSID**（esri.clsid 声明 +
   既有类清单二进制证据，第四轮已定名）——即 FDO 构造时以自身 +0x60 为
   聚合外层创建一个 **FeatureLayer 实例**，IUnknown 存 +0xb8。
3. 随后对该实例 QI（`0x6b18dfac` 起）：**IPersistStream `{00000109-…}`
   → +0xbc**（即主 Load 步骤 6 的委派槽）、**ILayer `{34C20002-…}` →
   +0xc4**、**IFeatureLayer `{40A9E885-…}` → +0xc8**、
   `{7453C805-9D64-44A6-BF2F-73E477DCE18F}`（未定名）→ +0xc0。

**结论：委派子对象是聚合创建的 FeatureLayer（coclass
`{E663A651-…}`，C++ 实现类为 FeatureLayer.dll 的 `FeatureLayer`
（RTTI `.?AVFeatureLayer@@`，运行期工厂宿主可能不同，见
[Map 对象记录](arcmap-10.1-map-object.md) 的工厂宿主差异说明）；其体 =
该实例的 IPersistStream::Load（FeatureLayer.dll `0x6f6e9940`，第六轮已
按 GetClassID 锚定）——第六轮「委派体按 FeatureLayer 字段序逐字段吻
合」的样本辅助观察由此获得机制解释。无对象头的原因：FDO 保存侧直接委
派该实例的 IPersistStream::Save 内联写出，不经 ObjectStream 登记（Save
侧委派本轮未逐指令复核，按对称性推定，开放）。**

## 实现（见 src/、tests/）

- 新模块 `fdo_legacy`（独立入口，不改 `fdo_graphics_layer` 与 `lib.rs`
  之外任何共享文件）：
  - `read_fdo_graphics_layer_legacy_prefix_10_1`：旧加载器（版本 0..=3）
    已确认字段读取器。ILayer 槽与主分支同构（新 FeatureLayer 前缀停止/
    其他类回退/空标记与回引继续）；ISpatialReference 槽分派 PCS 完整体
    （其余类回退停止）；IElementCollection2/IFeatureRenderer/IColor 槽
    新对象一律回退停止（体未闭合）；IName 槽分派 FeatureClassName 完整
    体（体内停止透传）。事务性（trial 游标/登记表，失败不推进）；截断
    向上传播；`stop=None` 表示旧体完整消费。
  - `read_fdo_graphics_sublayer_10_1`：元素体读取器（版本 ≤2；无对象
    头、无键、无 LoadObject）。
  - 常量 `FDO_GRAPHICS_SUBLAYER_CLASS_ID`（GetClassID 两处写出互证；不
    用于流内比对）。
- 版本 4 探测不实现（原因见上）；两个读取器的字段序均无样本字节佐证，
  合成测试（`tests/fdo_legacy.rs`，20 项）覆盖版本门、全槽位分派、逐字
  节截断、登记表事务性与限额转发。
- 接线说明：`fdo_graphics_layer::read_fdo_graphics_layer_prefix_10_1`
  对版本 <5 仍停止（`VersionBranchNotExpanded`）；旧加载器入口供调用者
  在确认版本 ≤3 时显式使用，未接入 `parse` 命令（与 TODO 另一开放项
  「对象循环边界」的集成一并考虑）。

## 未解决事项

1. 版本 4 探测的三个依赖（流绝对起点访问、IObjectStream QI 的游标效应、
   分支 B 不复位）未在本轮闭合；其「Map 架构版本 9 / 流首 FDO 内联序」
   的历史布局解读为候选，需旧版本样本或运行时取证。
2. 旧加载器各接口槽（+0xc4/+0xc8/+0x60）与 FDOGraphicsSublayer 两个
   String 成员的具体业务语义未定性；FDOGraphicsSublayer Save 的版本表
   `0x6b1ab0a8` 逐条目展开未做（写侧总写尾随 double 的不对称已记录）。
3. FDOGraphicsLayer Save 侧的委派写出（聚合 FeatureLayer 内联）未逐指
   令复核；`{7453C805-…}`、`{DBCA59AC-…}` 等未定名 IID 未做类型库考
   证。
4. 两个新读取器仅验证代码自身契约（合成输入），未在目标软件验证；样
   本（版本 11、计数 0）对本轮三项开放点均无字节佐证，全部结论为静态
   反编译证据。
