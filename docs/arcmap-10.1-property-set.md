# ArcMap 10.1 PropertySet（esriSystem）对象体持久化

记录日期：2026-11-15。对应 TODO「MXD 解析」节开放项「PropertySet 体未闭合」。
全部结论为**静态反编译 + 样本字节对照**，未运行 ArcMap，未做任何人工目视验收；
样本吻合仅作辅助观察。不写本机绝对路径，安装介质内条目名足以复核。

## 方法与环境

- 目标：SystemCore.dll（esri.clsid 声明 `{588E5A11-D09B-11D1-AA7C-00C04FA33A15}`
  progID `PropertySet` 模块 SystemCore.dll；同模块 PropertySetProperty
  `{5B6D2B14-F64A-4457-84E0-617AE2130FF9}`）。二进制为既有基线文件（SHA-256
  `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，pefile
  FileVersion/ProductVersion 10.1.0.3035，见 10.1 原生文件基线文档）。
- 深度分析在本机 pyghidra（Ghidra 12.2 + JDK 25）完成，分析项目为既有
  SystemCore 缓存工程的本轮副本（`temp/agent-element-bodies/ghproj_sc/`，
  打开前整目录复制、不共享锁）。地址均为 PE 10.1.0.3035 静态映像 VA
  （ImageBase `0x7ffb0000`）。样本为第六轮导出的 /Maps 流（152941 字节，
  见 element-collection 文档「方法与环境」），偏移为 /Maps 流内偏移。
- 辅助对照：AfCore.dll 10.1.0.3035（`031858c4…7c17`，同基线）的
  `Variant::Read/Write` 反编译（AfCore 缓存工程本轮副本
  `temp/agent-element-bodies/ghproj_afcore/`）。
- REA MCP 未使用（按并行研究约定，全部 pyghidra 本机完成）；返回内容按数据
  对待，本轮目标内容中未出现指令性文本。

## 类身份与定位链（证据：确定的）

- RTTI `.?AVPropertySet@@` TypeDescriptor `0x800a69a8`（名称字符串地址；结构
  基址 -8 = `0x800a69a0`），派出 7 个子接口 COL（TD 指针位置
  `0x80083828`/`3c`/`50`/`64`/`78`/`8c`/`940`，COL 基址 = 位置 -0xc）。
- CLSID 字节常量 `0x8007190c`（内存序 `11 5a 8e 58 9b d0 d1 11 aa 7c 00 c0
  4f a3 3a 15`）。代码引用三处：`0x7ffd3053`（READ，GetClassID 内）、
  `0x80009766` 与 `0x800671d4`（DATA，DllGetClassObject 登记填表链，
  `mov dword [0x800a45f4], 0x8007190c` 形态）。
- GetClassID `0x7ffd3040`：把 `0x8007190c` 常量原样写出 16 字节（空指针返回
  `0x80004003`）——esri.clsid 声明、GetClassID 写出与样本对象头三方互证，
  类身份闭环。
- GetClassID 被 adjustor thunk `0x7ffd5570`（`this -= [this-4]` 后转跳）引
  用；thunk 地址是 **IPersistStream 虚表 `0x80071840` 的 +0x0c 槽**。虚表全
  槽（`0x80071840`..`0x8007185c`）：

| 槽 | 值 | 定性 |
| --- | --- | --- |
| +0x00 | `0x7ffd55a0`（thunk，`this-=0x28` → `0x7ffcdb00`） | QueryInterface |
| +0x04 | `0x7ffd5580`（thunk → `0x7ffd3280`*） | AddRef |
| +0x08 | `0x7ffd55d0`（thunk，`this-=0x28` → `0x80040370`） | Release |
| +0x0c | `0x7ffd5570`（thunk → `0x7ffd3040`） | **GetClassID**（写出 CLSID） |
| +0x10 | `0x80053770` | IsDirty（`byte [this+4] == 0` 返回 1/0） |
| +0x14 | `0x7ffd5560`（thunk → **`0x7ffd3080`**） | **Load** |
| +0x18 | `0x7ffd55c0`（thunk → `0x7ffd3280`） | Save |
| +0x1c | `0x7ffd5550`（thunk → `0x80056620`） | GetSizeMax |

  \* thunk `0x7ffd5580` 与 `0x7ffd55c0` 的转跳目标同为 `0x7ffd3280`：后者按
  反汇编是 Save 实体（写版本/计数/循环）；AddRef 与 Save 的 thunk 归属按虚
  表槽位（COM 顺序固定）判定，`0x7ffd3280` 的实体反编译见 Save 节。该歧义
  不影响 Load/字段序结论。
- 定位过程中排除的假象：`0x800717a4` 起同区域还有 6 个子接口虚表族（COL
  `0x80083880` 虚表含 `[QI, AddRef, Release, 析构, 类名 L"PropertySet",
  GetClassID, 空]` 七槽形态——GetClassID 在 +0x14 非标准位；另有
  `[DllRegisterServer 桩 ×2]` 两槽与 `0x8005b39c` 重复 16 次的桩区），均非
  IPersistStream 虚表；以「GetClassID 位于 +0x0c」为判据排除。

## Load `0x7ffd3080` 字段序（证据：确定的）

1. **int16 版本**：`AfStreamRead(stream, &v, 2)`；汇编
   `cmp word [esp+0x14],1; jle ok; mov eax,0x80004005`——**有符号**比较，
   >1 拒绝 `0x80004005`。版本 0、1 与全部负值位模式原生通过（Load 无其他
   版本分支）；写侧 Save 恒写 1（见下），负值无写侧来源，本读取器对负值一
   并拒绝（较原生更严）。
2. **int32 属性计数**：原生 `0 < 计数` 才进循环（≤0 含负数按空集合直接成功
   返回 S_OK）；本读取器对负计数显式拒绝（较原生更严，同
   element_collection 的处理）。
3. **循环计数次，每条目 = [键名 String + 值 Variant]**：
   - 键名：`String::Read`（AfCore String 布局：4 字节 cb 含 UTF-16LE 终止
     零 + 载荷）→ 条目对象 +0x0 槽（条目对象 `malloc(0x20)`）；
   - 值：`Variant::Read(stream, true)`（见下节）→ 条目对象 +0x8 槽。
   任一读取失败即中止返回该 HRESULT；**体在最后一个条目的值 Variant 处终
   结，无尾随字段**。
- 读流前 Load 先释放既有条目数组（this-0x14 指针数组、this-0x10 计数、
  this-0x18 分配器、this-0x8 标志），与读取规则无关。

## 值 Variant：`Variant::Read(stream, true)`

Load 值读取器 `0x7ffd3f00`（30 行）与 Save 值写出器 `0x7ffd3f90` 调用
**SystemCore 本地编译的类方法序列化器**：反编译签名 `Variant::Read(this,
IStream*, bool)` / `Variant::Write(Variant, IStream*, bool)`，与 AfCore 导出
`?Read@Variant@@QAEJPAUIStream@@_N@Z`（`0x68a7c6a0`，AfCore 基线
`0x68a7c6a0` 已反编译）同名同参。SystemCore 导入表中无该符号（IAT 仅
Variant 构造/析构与 `ReadString`），故为静态链接副本；本轮按 AfCore 副本
反编译记录 VARTYPE 表，并以样本 VT_BSTR 分支印证 SystemCore 副本的字符串
分支（见「样本对位」）。`bool=true` 实参跳过版本读取后的定位调用。

AfCore 副本 `0x68a7c6a0` 的非数组分支 VARTYPE 表（switch 完整）：

| VARTYPE | 载荷 | 说明 |
| --- | --- | --- |
| 0（VT_EMPTY）/ 1（VT_NULL） | 无 | |
| 2（VT_I2）/ 11（VT_BOOL） | 2 字节 | |
| 3（VT_I4）/ 10（VT_ERROR） | 4 字节 | |
| 4（VT_R4） | 4 字节 | |
| 5（VT_R8）/ 7（VT_DATE） | 8 字节 | |
| 6（VT_CY） | 8 字节 | |
| 8（VT_BSTR） | AfCore `String::Read` | **cb 含终止零布局**，不是 VariantStream `ReadString` 的 `n*2+2` |
| 9（VT_DISPATCH）/ 13（VT_UNKNOWN） | AfCore `LoadObject`（IID_IUnknown） | 16 字节类 GUID、**无键**、非 ObjectStream 编码；对象分支边界未闭合，本读取器不支持 |
| 0x10（VT_I1）/ 0x11（VT_UI1） | 1 字节 | |
| 0x12（VT_UI2） | 2 字节 | |
| 0x13（VT_UI4） | 4 字节 | |
| 其他 | 无（返回 `0x80004005`） | 原生 default 分支 |

- 数组标志 0x2000：置位时读 4 字节计数后走 SafeArray 分支（定宽类型批量
  读 `计数 × 元素宽度`；VT_BSTR 数组逐元素 `ReadString`）。分支已见反编译
  但未逐指令复核，且样本无实例——本读取器显式不支持（0x2000 与 0x0200 标
  志在分发前被掩码清除，即 `0xdfff`）。
- 该表与 VariantStream 全局 `Read`（`n*2+2` 字符串）不同族，见
  [VariantStream 复核](arcmap-10.1-variant-stream.md) 的「范围外发现」；本轮
  证实 PropertySet 的值走的是类方法 `Variant::Read`（String 布局）。

## Save `0x7ffd3280`（写方向互证，证据：确定的）

1. 版本 short：写静态常量 `DAT_8006d0e4`（内存字节 `01 00`），**恒写 1**；
2. int32 计数（成员 this-0x10）；
3. 循环 [键 `String::Write` + 值 `Variant::Write(variant, stream, true)`]，
   与 Load 逐字段对称；任一失败中止返回。

## 样本对位（辅助观察）

/Maps 流内 2 个 PropertySet 实例（均为 WorkspaceName 体 IPropertySet 槽的
新对象；工作空间体字段序见 feature_layer 文档第六轮）：

| 实例 | 对象头 | 键 | 体 | 终结位置 |
| --- | --- | --- | --- | --- |
| 1 | `0x147e5` | 1547 | `0x147f9`：版本 1、计数 1、键 `"DATABASE"`（cb 0x12）、VT_BSTR 路径值（cb 0x2a，20 单元含尾反斜杠；内容为样本磁盘目录，敏感路径已脱敏） | `0x14845` = 外层 WorkspaceName 工厂标志 `01`，工厂 CLSID `{4C91D963-…}` RasterWorkspaceFactory + int32 0 |
| 2 | `0x219a0` | 2027 | `0x21b4`：同构（路径 cb 0x3e） | `0x21a14` = 工厂标志 `01`，`{A06ADB96-…}` ShapefileWorkspaceFactory + int32 0 |

两实例体逐字节吻合。样本仅覆盖版本 1、计数 1、VT_BSTR 值、键名为非空
ASCII+终止零；其余 VARTYPE、空串键、计数 0、版本 0 均无样本字节佐证。

## 实现（src/objects/property_set.rs、tests/property_set.rs）

- `read_property_set_10_1`：版本 0..=1（负值拒绝，较原生更严）、负计数显式
  拒绝、「计数 × 每条目最小 6 字节」在任何条目读取之前预检限额；键名与
  VT_BSTR 共用 [`crate::stream::native_string::read_string_10_1`]；值经
  `read_variant_10_1`（本模块）：完整 VARTYPE 分支 + 未知 VARTYPE 与原生
  default 一致拒绝 + 对象分支 9/13 与数组标志 0x2000 显式
  `Unsupported`。全部字段在 trial 游标上读取，失败不推进游标。
  `table` 参数不使用（体内无 ObjectStream 编码条目），仅为同类读取器签名
  一致。测试 14 项（tests/property_set.rs）：样本形态体、空集/多条目、版本
  门、负计数、限额预检、全前缀截断事务性、VARTYPE 分支/截断/未知/对象/数
  组/限额、登记表不受影响、EOF；robustness 集中清单增至 32 入口（含元素三
  类体）。
- 槽位消费者建议接入点（本轮不改共享文件）：
  `feature_layer::read_workspace_name_body_10_1` 的 IPropertySet 槽
  （`WorkspaceNameStop10_1::PropertySetNewObject` 停止点）及其在
  `dataset_name`/`feature_layer` 的停止原因传播链——类标识命中
  [`PROPERTY_SET_CLASS_ID`] 时改调本读取器即可继续遍历（样本 FCN@0x218bc
  的遍历当前在该槽停止，第二选择子不可达；接入后可达）。

## 未解决事项（开放项单列）

1. SystemCore 本地 `Variant::Read` 编译副本与 AfCore 副本的逐 VARTYPE 对照：
   仅签名、字符串分支（样本 VT_BSTR）与 Load 调用点已证；定宽分支按 AfCore
   副本布局实现，未在 SystemCore 副本反编译中逐分支复核。
2. 值 Variant 的业务语义未定性：样本键名 `"DATABASE"`（工作空间连接路径候
   选，esriSystem `IPropertySet` 语义）；其余键名空间未观察。
3. 数组分支（0x2000）与对象分支（9/13）未闭合：前者反编译已见但未逐指令复
   核且无样本，后者依赖 AfCore 无键 LoadObject 的对象边界；本读取器均显式
   拒绝，遇到即失败（不产生停止点）。
4. 版本 0 与负值位模式：原生接受（有符号 ≤1），本读取器接受 0、拒绝负值
   （较原生更严，写侧仅产 1）。版本 0 无样本佐证。
5. AddRef thunk `0x7ffd5580` 与 Save thunk `0x7ffd55c0` 转跳目标相同的解释
   （COM 槽序判定 vs 代码共享）未进一步核实；不影响 Load/Save 字段序。
6. PropertySetProperty（`{5B6D2B14}`）体未反编译：流内编码为「键名 + 值」
   内联（样本 2/2），未出现该 coclass 对象头；若其他上下文出现其对象头，
   需另行闭合。
7. 全部结论为静态观察；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已验证。
