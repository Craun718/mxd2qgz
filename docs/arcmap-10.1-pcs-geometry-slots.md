# ArcMap 10.1 空间参考槽实现类：ISRPersistStream、PCS 标志/double 语义、GCS 与 Unknown 体、CircularArc 裁决

> [后续指令复核](mxd-terra-integrate-sr.md)更正了 PCS/GCS v2 的域串版本门、PCS 垂向单位段和 GCS 守卫位置。涉及这些细节时以后续记录为准。

记录日期：2026-11-16。对应 TODO「MXD 解析」开放项：闭合 ProjectedCoordinateSystem
的接口 IID；定性 PCS 标志字段语义；定性 PCS double 字段语义；ISpatialReference
槽实现类体（ProjectedCoordinateSystem 之外的 GeographicCoordinateSystem 与
UnknownCoordinateSystem）；Geometry CircularArc 持久化虚表定位。全部为**静态
反编译/汇编核对 + 类型库/GUID 检索 + 既有样本记录交叉验证**，未运行
ArcMap 10.1，未人工目视验收。地址均为 Geometry.dll 10.1.0.3035（sg.dll
10.1.0.679）静态映像 VA。不写本机绝对路径。

## 方法与环境

- **Geometry.dll**：沿用 `temp/arcgis101-bin/geometrydll/` 既有提取物，
  SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`、
  ImageBase `0x717e0000`、VS_FIXEDFILEINFO 10.1.0.3035，与既有各轮记录一致。
  深度分析在本机 pyghidra（Ghidra 12.2 + JDK 25）完成，复用既有分析工程副本
  （独立目录，Open 不共享锁）；其内嵌 PDB 残留符号（`ISRPersistStream::vftable`、
  `ClassFactory<ESRI::Arc>` 等）按既定纪律仅作命名辅助，身份判定均另行取证。
- **sg.dll**：本轮从介质 `Setup1.cab` 条目 `sg.dll.D587F9C8_6324_4BA7_B1C5_…`
  新提取，342,016 字节，SHA-256
  `ebbb758571da5619a3396b81057f97801f7107180658ced0837187da9e4e027c`，PE x86、
  ImageBase `0x7fab0000`、VS_FIXEDFILEINFO **10.1.0.679**（与两个 pe.dll 条目
  同版本族；与 Desktop 3035 的组件版本映射沿用既有未核对项）。sg.dll 以
  `SgCoordRef*` 等 229 个导出函数被 Geometry.dll 导入（既有 Save 证据）。
- **类型库/GUID**：wine `LoadTypeLibEx` 的 esriGeometry 类型库 dump（既有
  `/tmp` 会话产物，本轮仅检索）；`esri.clsid` 全表 XML（既有提取物，仅检索）。
- 样本交叉验证使用既有各轮记录的 `samples/测试底图.mxd` `/Maps` 首对象
  （PCS 键 30，体 `0x44..0x274`，560 字节）逐字节值，未重复导出。

## 任务 1（闭合）：PCS 持久化接口 = ISRPersistStream `{E88C7DB0-9E84-11D2-BD00-0000F875BCCE}`

旧记录开放项「接口虚表 `0x71a29ccc` 的接口 IID 与名称未确认（槽位序仅与
COM IPersistStream 吻合）」本轮按五环证据链闭合：

1. **类型库命名**：esriGeometry 类型库接口 #85 `ISRPersistStream
   E88C7DB0-9E84-11D2-BD00-0000F875BCCE` typekind=3（接口名与 GUID 绑定为
   类型库文件内静态数据）。
2. **IID 常量唯一**：`{E88C7DB0-…}` 的 16 字节内存序在 Geometry.dll 全镜像
   （.rdata + .data）唯一出现于 `.rdata 0x71a24b64`。
3. **QI 映射**：PCS 主（身份）虚表 `0x71a29748` 槽 0 = QI `0x719d05e0`。其
   汇编比较链依次 `MOV ECX,0x71a1135c`（= 标准 `IPersistStream`
   `{00000109-0000-0000-C000-000000000046}`）、`MOV ECX,0x71a1134c`（= 标准
   `IPersist` `{0000010C-0000-0000-C000-000000000046}`）、`MOV ECX,0x71a24b64`
   （ISRPersistStream，@`0x719d08e5`），三者匹配均跳 `0x719d0b0a`：该处理器
   `ADD EAX,0x48` 返回 **this+0x48**。即同一实现同时应答三个 IID。
4. **构造器安装**：PCS 构造器四个重载（`0x719da280`/`0x719da5f0`/`0x719da960`/
   `0x719dade0`）都在对象 +0x48 写 `0x71a29ccc`（如 `0x719da3eb`
   `MOV dword ptr [ESI+0x48],0x71a29ccc`）。构造器反编译带出 PDB 接口名序列：
   +0x10 IClone、+0x14 ICompareCoordinateSystems、+0x18 ISpatialReference3、
   +0x1c ISpatialReference4、+0x20 ISpatialReference2GEN、+0x24
   IProjectedCoordinateSystem5、+0x28 IProjectedCoordinateSystemImpl、+0x2c
   IProjectedCoordinateSystem4GEN、+0x30 IProjectedCoordinateSystemEdit、
   +0x34 IESRISpatialReference、+0x38 IESRISpatialReferenceGEN、+0x3c
   IESRISpatialReferenceGEN2、+0x40 IPRJSpatialReference、+0x44
   IPRJSpatialReferenceGEN、**+0x48 ISRPersistStream**、+0x4c IControlPrecision3、
   +0x50 IXMLSerialize、+0x54 IXMLVersionSupport、+0x58
   ISpatialReferenceTolerance、+0x5c ISpatialReferenceResolution、+0x60
   ISpatialReferenceAuthority、+0x64 IDocumentVersionSupportGEN（PDB 名仅作
   旁注；+0x48 槽的安装指令与 QI 映射为独立证据）。
5. **COL/RTTI 锚定**：虚表 `0x71a29ccc` 前一指针为 COL `0x71a43254`
   （`[C]==0`、`[C+0xc]`=TD `.?AVProjectedCoordinateSystem@@` @`0x71a6b8f4`），
   形式上归属该 C++ 类；全类 COL 枚举得 24 个虚表（GCS 23 个、Unknown 18 个、
   `ESRI::Arc` 16 个，见下文）。

**ISRPersistStream 虚表 `0x71a29ccc` 全 11 槽**（PCS）：

| 槽 | 函数 | 判定 |
| --- | --- | --- |
| 0/1/2 | `0x71936980`/`0x71807120`/`0x718918e0` | QI/AddRef/Release 的 +0x48 调整器 |
| 3 | `0x719d2b30` | GetClassID（与身份虚表共享；判空 `0x80004003` 后 movq 写 CLSID `0x71a13098`=`{2A626700-…}`） |
| 4 | `0x719d3220` | IsDirty（共享恒返 0） |
| 5 | `0x719d3230` | Load（既有证据，本轮反编译复核） |
| 6 | `0x719d3690` | Save（既有证据，本轮反编译复核） |
| 7 | `0x719d38d0` | GetSizeMax：恒写固定估计值 `0xc22`（3106；样本 v6 体实为 560 字节） |
| 8 | `0x719bf440` | 恒写 `0x38`（56 = 7×8，精度块字节量候选，语义未闭合） |
| 9 | `0x719d38f0` | 转调 `FUN_719bae60(0)`（读「无 2 字节标志」的精度块） |
| 10 | `0x719d3910` | 转调 `FUN_719bb150(0)`（写「无 2 字节标志」的精度块） |

GCS 的 ISRPersistStream 虚表 `0x71a26818` 同为 11 槽且 slot 3/4/7/8 与 PCS
完全同函数（GetClassID 换 GCS 版 `0x719bc190`、Load/Save 换 `0x719bec60`/
`0x719bf1d0`、slot 9/10 换 `0x719bf460`/`0x719bf480`）；Unknown 的持久化虚表
`0x71a2c320` 只有 **8 槽**（恰 IPersistStream 形，无 slot 8/9/10），且其主 QI
`0x719eabd0` 只比较标准 `{0000010C}`/`{00000109}`（`0x719ead49`/`0x719ead5d`）
命中后返回 **this+0x30**，**不**比较 ISRPersistStream——Unknown 实现的是普通
IPersistStream，不是 ISRPersistStream。

对象流侧衔接（沿用既有证据并补一句）：AfCore `LoadObject(IUnknown**, riid,
stream, …)` 以 `{4434D680}`(ISpatialReference) 为请求 IID 创建对象，读写本体
经上述持久化接口（PCS/GCS 以 IPersistStream/IPersist/ISRPersistStream 三个
IID 之一皆可达同一 +0x48/+0x2c 实现，Unknown 仅标准两 IID）。

## 任务 2（闭合）：PCS 标志字段（2 字节 u16）语义

- **写侧**（Save `0x719d3690` → `FUN_719bb150(1)`，反编译逐行）：先按
  SgCoordRef 标志位取值——bit2（`this+0x74` 句柄存在）非零时调
  `SgCoordRefGetM/GetZ/GetXY/GetPrecision` 填充栈上 7 个 double，然后
  `local_48 = SgCoordRefGetPrecision 结果 >> 6 & 1`，`AfStreamWrite(stream,
  &local_48, 2)` 写出 2 字节。即 **流内 u16 = 精度值的 bit6**。
- **精度值域**（sg.dll `SgCoordRefGetPrecision` `0x7faebc40` 反编译）：结构
  `+0x7c` bit1 置位 → 返回 **0**；否则返回 `0x20 + (bit0 ? 0x20 : 0)`，即
  **0x20（32）或 0x40（64）**。因此流内 u16 ∈ {1, 0}：**1 = 64 位（高）精度
  网格，0 = 32 位（低）精度或无精度网格**。
- **读侧**（`FUN_719bae60(1)` 反编译）：读入 2 字节后**不消费**（局部变量
  未再使用），Z/M 存在性改由第 4/第 6 个 double 是否 NaN 推导；首 XY double
  NaN 时整体清空（释放 SgCoordRef、标志位清零、返回）。例外：Unknown 的
  Load（`0x719ecd80` 版本 ≥2 内联分支）**消费**该 u16——非零 →
  `SgCoordRefSetPrecision(…, 0x40)`，零 → `0x20`，与写侧语义互证。
- 样本对照：首对象 u16 = `0x0001` ↔ 64 位高精度网格 ✓（ArcMap 10.x 新建
  数据框默认高精度）。

## 任务 3（闭合）：PCS 的 7 + 3 个 double 语义

**7 个精度 double**（`FUN_719bb150` 写序 = `FUN_719bae60` 读序，sg.dll
`SgCoordRefGetXY/GetZ/GetM`（`0x7faebd20`/`0x7faebea0`/`0x7faebad0`）逐字段
绑定）：

| 流序 | 字段 | sg 结构偏移 | 写侧来源 | 读侧去向 |
| --- | --- | --- | --- | --- |
| d0 | XY 假原点 X | +0x08 | `SgCoordRefGetXY` 出参 1 | `SgCoordRefSetXY` 实参 1（NaN → 全清） |
| d1 | XY 假原点 Y | +0x10 | 出参 2 | 实参 2 |
| d2 | XY 比例因子（scale） | +0x18 | 出参 3 | 实参 3（`SetXY` 要求 ≥1.0，否则 `0xfffff82d`） |
| d3 | Z 假原点 | +0x30 | `SgCoordRefGetZ` 出参 1（bit0=Z 已设时才取） | `SgCoordRefSetZ` 实参 1（NaN → 0 并按 NaN 定 bit0） |
| d4 | Z 比例因子 | +0x38 | 出参 2 | 实参 2 |
| d5 | M 假原点 | +0x50 | `SgCoordRefGetM` 出参 1（bit1） | `SgCoordRefSetM`（NaN → 0） |
| d6 | M 比例因子 | +0x58 | 出参 2 | 同上 |

命名依据：`SgCoordRefSetXY(handle, x, y, scale)`/`SetZ(handle, z, scale)`/
`SetM(handle, m, scale)` 的第三/二实参在 sg.dll 内要求 ≥1.0、写入后经
`SgsCalcCRound`/`SgsCalcHalfSU` 派生「快照单位」字段（+0x20/+0x28 等），即
ArcObjects 高精度网格的「假原点 + 比例因子」（分辨率 = 1/scale）；「假原点」
为按该机制命名的候选提法，pe/sg 内部如何从投影推导其数值未展开。样本值
XY(31876800.0, -10002100.0, 450265407.00157917)、Z(-100000.0, 10000.0)、
M(-100000.0, 10000.0)：Z/M 比例因子 10000 ↔ 分辨率 1e-4（esri 默认 Z/M
分辨率量级）；XY 比例因子 4.5e8 ↔ 分辨率约 2.22e-9。未设 Z/M 时的占位值
`DAT_71a66f20` 已读出：字节 `01 00 00 00 00 00 00 00` = **最小正非规格化数
4.94e-324**（与 AfCore WKSEnvelope 空占位同族），非 NaN——读侧会把该占位
当作「已设且值为占位」处理，属原生代际行为，读取器原样保留。

**3 个尾随 double** = **XY/Z/M 容差**（对象 +0x138/+0x140/+0x148）：

- ISpatialReferenceTolerance（this=对象+0x58）get/put XYTolerance 槽 5/6 读写
  +0x138、get/put ZTolerance 槽 9/10 读写 +0x140、get/put MTolerance 槽 13/14
  读写 +0x148（`0x719d2080`/`0x719d20b0`/`0x719d2200`/`0x719d2230`/
  `0x719d2330`/`0x719d2360` 反编译）；Save `0x719d3690` 直接写
  `param_1+0xf0/+0xf8/+0x100`（this=+0x48 视角即 +0x138/+0x140/+0x148）。
- PCS 默认容差（槽 3/7/11，`0x719d1f90`/`0x719d20d0`/`0x719d2250`）。
  XY 初值为 `0.001 / pe_unit_factor(pe_projcs_unit(projcs))`。Z 初值为 0.001，
  有 VCS 时通过 VCS vtable +0x28 取单位，再通过单位 vtable +0x20 取因子并除以该因子。
  M 初值恒为 0.001。相应网格组存在时，才用 `2.0 / scale` 提高该组容差的下限。
  XY/Z/M 的标志位分别为 `0x4/0x1/0x2`，Z/M 还检查 SgCoordRef 指针。
  参考样本的 XY 单位因子为 1，`2.0 / 450265407` 约为 4.4e-9，与流内 XY 容差
  0.001 一致。该样本的 Z/M 字段也都是 0.001。
- 旧版本恢复路径（版本 <6 的 Load）：按 SgCoordRef 标志位调 ISpatialReference
  Tolerance 的槽 4/8/12（`0x719d2010`/`0x719d2190`/`0x719d22c0`），把容差置为
  **2.0 / 对应 scale**（无对应组时报 `0x80040202`）——即旧容差 = 2 个网格
  单位（与 sg.dll `SgCoordRefSetDefault*ClusterTol` 导出族的「簇容差」概念
  对应）。版本 ≥5/≥6 直接读流（PCS ≥5、GCS/容差段 ≥6，见下文版本表）。

### 默认容差的跨模块来源

依据为 ArcMap 10.1 Desktop 介质的 PE32 模块及 `objdump -d -Mintel`。用导入表将
Geometry.dll 跳板对应到 pe.dll/sg.dll 的命名导出，再检查导出函数体。这里的地址
为模块首选加载地址下的虚拟地址。

- pe.dll 的 `pe_projcs_unit`（`0x7b8eada0`）与 `pe_geogcs_unit`（`0x7b8d7650`）
  提供单位对象。`pe_unit_factor`（`0x7b8f0830`）对符合对象签名和单位类型掩码
  `0x300` 的对象读取 +0x278 的 double，否则返回 1。
- GCS 的 XY 默认容差入口 `0x719bd7b0` 先取得角单位因子的倒数，再经
  `pe_geogcs_datum`、`pe_datum_spheroid` 取得椭球。从椭球提取半长轴 `a`
  和扁率 `f`，于 `0x719bd851` 向 `pe_geodesic_coordinate`（`0x7b8b3940`）
  传入 `a, f*(2-f), 0, 0, 0.001, π/2` 及两个输出参数。将结果转换为 GCS
  角单位并减去同单位的起点后，于 `0x719bd8bc` 调用 AfCore 的
  `WKSPoint::Norm(2)`。参数 2 是范数阶数。ArcMap 10.1 AfCore.dll 的命名导出
  `?Norm@WKSPoint@@QBENJ@Z`（`0x68aca870`）在该分支计算
  `sqrt(dx² + dy²)`。现有伪代码 `AfCore/functions/0x68aca870.c` 给出
  `_CIsqrt` 调用，平方和由 `0x68aca8f0–0x68aca8f6` 的 x87 指令复核。
  有 XY 网格时再用 `2.0 / XYscale` 提高下限。
- GCS 的 Z 默认容差入口 `0x719bd9a0` 将初值 0.001 写入接口 this+0x20。
  有 VCS 时也通过 VCS vtable +0x28 和单位 vtable +0x20 取因子，但在除法前
  调用 `_finite`（IAT `0x71a10958`），并检查因子严格大于 0（常量
  `0x71a2cb88`）。未通过检查时保留初值。PCS 对应实现 `0x719d20d0`
  在取得单位因子后直接执行除法（`0x719d2126`）。
- GCS 的 M 默认容差入口 `0x719bdb30` 将初值 0.001 写入接口 this+0x28。
  GCS 的 Z/M 都检查接口 this+0x6c 的 SgCoordRef 指针和 this+0x70 的对应
  标志位（Z 为 `0x1`，M 为 `0x2`）。网格组启用时，分别在
  `0x719bda44`/`0x719bdb62` 调用 sg 取比例因子，再用 `2.0 / scale`
  提高下限。
- sg.dll 的 `SgCoordRefGetXY`（`0x7faebd20`）、`SgCoordRefGetZ`
  （`0x7faebea0`）及 `SgCoordRefGetM`（`0x7faebad0`）分别把
  SgCoordRef +0x18、+0x38、+0x58 的 double 写到最后一个输出参数。
  这些值是 Geometry.dll 默认容差网格下限所用的除数。

AfCore.dll 的 SHA-256 为
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`。
本轮 pe.dll 的 SHA-256 为
`2c29c819ba6506eebd64834d562ad0664c62c3d3ed6a88bfc963df5121bee7b3`，
sg.dll 为 `ebbb758571da5619a3396b81057f97801f7107180658ced0837187da9e4e027c`。
尚待以原生构造案例测量自定义单位和各网格组启用时的数值。本轮未在 ArcMap 或 QGIS 中人工验证。

## 顺手闭合的旧记录开放项（对既有文档的更正，详见文末汇总）

- **域字符串编码**：Save `FUN_719bad00` 写出的是 BSTR 的**原样 UTF-16LE 字节**
  （长度来自 `Ordinal_149`，即 BSTR 字节长；Load `FUN_719babf0` 按宽字符消费、
  补宽 NUL 后转 BSTR）。旧记录「4 个 UTF-16 指针经 Ordinal_149 转码（ANSI）」
  为误读：无转码，流内就是 UTF-16LE 字节 + u32 字节长度。样本 4 串全空，与
  两种读法逐字节一致（长度 0），故旧记录当时未被样本否证。
- **垂向域段**：1 字节标志非零时，流内紧跟
  `[u16 段版本 ≤2][u32 字节长][等长单位 WKT 原样字节]`（`0x719ea140` 反编译：
  读出后 `pe_unit_from_string` 解析为垂向线性单位；随后仅栈上构造 LinearUnit
  与默认 `VERTCS["Unknown VCS from ArcInfo Workstation",…]` 垂向坐标系对象，
  **无更多流读取**）。而 10.1 Save 写该标志为**常量 0**（`0x719d3690`/
  `0x719bf1d0` 均无条件写 1 字节零）——垂向段只为非 10.1 写方存在。
- **WKT 生成条件**：`pe_hvcoordsys_to_string` 与 `_ext` 的选择 = 文档版本
  `< 3` 用前者、`>= 3` 用 `pe_hvcoordsys_to_string_ext(hv, 1)`（Save
  `0x719d3690` 中 `unaff_EBX` 即 `AfStreamGetDocumentVersion` 返回值）。旧
  记录「未解析寄存器」闭合。docVer < 2 时 WKT 走 `pe_projcs_to_string`（GCS
  为 `pe_geogcs_to_string`）不变。
- **Load 末尾钩子归属**：PCS Load 末行 `(*(this-0x30)+0x74)(this-0x30)`：
  this-0x30 = 对象 +0x18 = ISpatialReference3 虚表（`0x71a297ac`），+0x74 =
  该接口 29 号槽（语义候选：加载后刷新；GCS 同构，this-0x14 = +0x18）。
- **保存版本表**：PCS `DAT_71a11dc8` = [2,3,6,6]（既有）；GCS `DAT_71a11e1c`
  = **[3,4,7,7]**（10.1 恒写 7）；Unknown `DAT_71a11d68` 前 3 项 = **[1,1,3]**
  （10.1 恒写 3）。版本接受门：PCS ≤7、GCS ≤8、Unknown ≤3（原生均为有符号
  short 比较；读取器按无符号拒绝负位模式，较原生更严，与既有 PCS 读取器
  约定一致）。
- **Save/Load 对称性补注**：PCS/GCS Save 对版本 2（docVer 0）仍**无条件**写
  4 域串 + 1 字节零，而 Load 对版本 ≤2 不读域串——写侧恒写、读侧按版本的
  不对称为原生行为（10.1 不产生版本 ≤3，实际无影响）。

## 任务 4（闭合）：ISpatialReference 槽的另两个实现类体

槽位事实（既有证据）：请求 IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`
（Map 第 4 步、Envelope 版本 3、Polyline 版本 2 等），实现类由流内 16 字节
类标识决定。样本 `/Maps` 中仅 PCS 有实例；本轮闭合 GCS 与 Unknown 两类体，
PCS 已由既有读取器闭合。

### GeographicCoordinateSystem（`{A6A87A80-1DD1-11B2-BF51-08002022F573}`）

定位链：esriGeometry coclass #96 + esri.clsid 声明；CLSID 常量 `0x71a13078`；
GetClassID `0x719bc190`（判空 → movq 写 `DAT_71a13078`，全镜像唯一）→ 双虚表
锚定：身份虚表 `0x71a265b0`（+0x14 槽）、持久化虚表 `0x71a26818`（+0x0c 槽），
两者 -4 处 COL（`0x71a40034`/`0x71a3ff80`）TD 均为 `.?AVGeographicCoordinateSystem@@`
@`0x71a6afe0`；构造器四个重载（`0x719c3060`/`0x719c3330`/`0x719c3610`/
`0x719c38b0`）在对象 **+0x2c** 安装 `0x71a26818`；QI `0x719bc1e0` 比较三个
持久化 IID 命中后 `ADD EAX,0x2c`（`0x719bc6b5` 处理器），与 PCS 同构。

字段序（Load `0x719bec60` 反编译逐行；Save `0x719bf1d0` 对称互证）：

| # | 字段 | 版本门 |
| --- | --- | --- |
| ① | int16 版本（>8 有符号拒绝 `0x80004005`） | 恒有 |
| ② | 4 字节 WKT 字节长 + 等长 ANSI WKT（3072 栈缓冲，原生无上限检查） | 恒有 |
| ③ | 清理旧状态（SgCoordRefFree、释放垂向对象；另有 `this+0x98`(对象+0xac) 非零时直接 `0x80004005`/0x1ca 的守卫，语义未定） | 恒有 |
| ④ | `pe_geogcs_from_string`（<7）或 `pe_hvcoordsys_from_string` + 可选垂向重建（≥7） | 恒有 |
| ⑤ | 4 组域串 + 1 字节垂向标志（非零 → 垂向单位段，见上节） | >2 |
| ⑥ | 2 个 double 地理范围 [min, max]（this+0x64/+0x6c；首项 NaN 时原生按 `180.0 ^ 0x8000000000000000` 与 ×2.0 重建为 [-180, 180]，掩码 `DAT_71a2cbb0` = `-0.0` 位模式已读出证实） | >2 |
| ⑦ | 精度块：版本 ==4 → 7 个 double（`0x719bae60(0)`，汇编 `PUSH 0x0` 证实）；版本 ≥5 → 2 字节标志 + 7 个 double（`PUSH 0x1` 证实）；版本 ≤3 无 | 见门 |
| ⑧ | 3 个 double 容差 [XY, Z, M] | ≥6（<6 时按标志位重建，无流字节） |

写侧版本表 [3,4,7,7]：文档版本 0 → 3（含 ⑤⑥、无 ⑦⑧）、1 → 4（含 ⑦ 无标志
形态）、≥2 → 7（全量）——分支与读侧逐条吻合。

### UnknownCoordinateSystem（`{B286C06B-0879-11D2-AACA-00C04FA33C20}`）

定位链：esriGeometry coclass #130 + esri.clsid 声明；CLSID 常量 `0x71a130c8`；
GetClassID `0x719eab80`（movq 写 `DAT_71a130c8`）→ 身份虚表 `0x71a2c0cc`
（+0x14 槽）、持久化虚表 `0x71a2c320`（+0x0c 槽），COL `0x71a4436c`/
`0x71a442a4` TD 均 `.?AVUnknownCoordinateSystem@@` @`0x71a6bb68`；默认构造器
`0x719edc60`、复制构造器 `0x719ede70`、析构入口 `0x719ee050`
均在对象 **+0x30** 安装 `0x71a2c320`；
主 QI `0x719eabd0` 比较标准 `{0000010C}`/`{00000109}` 命中 → `ADD EAX,0x30`
（`0x719eaf1e`）；持久化 QI 槽 `0x7193f2d0` 为薄调整器转发主 QI。

字段序（Load `0x719ecd80` 反编译逐行；Save `0x719ed310` 对称互证）：

| # | 字段 | 版本门 |
| --- | --- | --- |
| ① | int16 版本（>3 有符号拒绝） | 恒有 |
| ② | **4 个 AfCore `String::Read` 布局字符串**（UTF-16LE、cb=len×2+2 含终止零；非 PCS/GCS 的长度前缀裸字节形）。原生成员 +0x54/+0x4c/+0x4c/+0x50：**第 2、3 串写入同一成员**（Save 侧 `0x719ed38e`/`0x719ed3be` 两次读 `[ESI+0x4c]` 逐指令证实），即第 2 串被第 3 串覆盖——原生怪癖，读取器按流内 4 串原样保留 | 恒有 |
| ③ | 精度块：版本 ==1 → 7 个 double（`0x719bae60(0)`，汇编 `PUSH 0x0` 证实）；版本 ≥2 → 2 字节标志 + 7 个 double 内联读取（原生按标志非零推导精度 64/32，`SgCoordRefSetPrecision` 直接调用；首 XY double NaN → 清空） | 见门 |
| ④ | 3 个 double 容差 [XY, Z, M]（成员 +0x28/+0x30/+0x38；NaN 或 ≤0 时经持久化接口 this+0x10 的容差虚表槽 3/7/11 重置） | ≥3（<3 时从网格恢复，无流字节） |

容差恢复有两条路径。默认构造器在完整对象 +0x40 安装容差虚表
`0x71a2c39c`（`0x719edd8a`）。v3 的 Load 在容差为 NaN 或非正值时，
通过该虚表槽 3/7/11 调用 `0x719ebb20`/`0x719ebc20`/`0x719ebcf0`。
三个入口的初值均为 0.001，分别检查 XY/Z/M 标志 `0x4/0x1/0x2`，
随后经 sg 取比例因子，以 `2.0 / scale` 提高下限。Z/M 入口还检查
SgCoordRef 指针。三个入口均无单位换算。

v0–v2 的 Load 在 `0x719ecfbd` 直接跳到 `0x719ed2c1`，分别调用
`0x719bc0f0`/`0x719bc120`/`0x719bc150`。这些辅助函数经
`SgCoordRefGetXY/Z/M` 取比例因子，返回 `2.0 / scale`，没有 0.001
下限。两条路径均由 ArcMap 10.1 Geometry.dll 反汇编确认，尚未测量
旧版本无可用网格时的原生结果。Rust 读取测试覆盖 v3 的带负载 NaN、
负零和负数原样保留，并检查其后两个字节不被消费。

无 WKT、无垂向段、无范围段（Unknown 无投影概念）。写侧版本表 [1,1,3]：
文档版本 ≥2 恒写 3，与读侧 ≥3 的容差段吻合。

### 实现落位（本轮新模块）

`src/objects/spatial_reference_slots.rs`（新文件；`src/objects/mod.rs` 仅加
`pub mod spatial_reference_slots;` 一行）：

- `GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID` / `UNKNOWN_COORDINATE_SYSTEM_CLASS_ID`
  常量（16 字节内存序，与 CLSID 常量地址字节一致）。
- `read_geographic_coordinate_system_10_1` / `read_unknown_coordinate_system_10_1`：
  与既有 PCS 读取器同约定（调用者先读对象头比对类标识；trial 游标事务性，
  完整成功才提交；`max_bytes` 限额；WKT 强制 ASCII 较原生更严；负版本位模式
  拒绝较原生更严；域串与垂向单位段字节原样保留）。
- 垂向标志非零分支按原生消费垂向单位段（`VerticalUnitSegment`），不再像旧
  PCS 读取器那样把该分支留作未定字节。
- 测试 13 项（模块内 `#[cfg(test)]`）：两类 CLSID 逐字节、GCS 版本
  7/3/4/2 布局、垂向段消费与版本拒绝、Unknown 版本 3/1/0 布局、版本越界
  与全前缀截断事务性。**两类在参考样本中 0 实例，全部为合成输入覆盖，不得
  声称样本验证。**

## 任务 5（闭合）：CircularArc 不实现 IPersistStream（构造性不存在）

TODO 开放项「Geometry CircularArc 持久化虚表定位（其余五类已裁决不实现）」
本轮以与五类相同的方法闭合，结论为**该族的持久化虚表不存在**：

- **COM 类 ↔ C++ 类绑定**：esri.clsid 声明 `CircularArc
  {CD3A614E-BC4B-11D0-8303-0000F8034032}`（Geometry.dll 模块）。Geometry.dll
  `.data 0x71a65b60` 的类表条目为三个指针：→ CLSID 常量 `0x71a125e8`、→
  UTF-16 串 `L"CircularArc"`（`0x71a1ac68` 字节 `43 00 69 00 72 00 63 00 75
  00 6c 00 61 00 72 00 41 00 72 00 63 00 00 00` 已逐字节核对）、→
  `ClassFactory<ESRI::Arc>` 实例槽 `DAT_71a65b68`（注册构造 `0x71a09a70`
  开头装配，PDB 模板名）。即 **COM coclass CircularArc 由 C++ 类 `ESRI::Arc`
  实现**——这正是此前两轮 `.?AVCircularArc@ESRI@@`/`.?AVCircularArc@@` 两种
  RTTI 命名形态扫描均不命中的原因。
- **身份虚表**：`ESRI::Arc` RTTI TD `.?AVArc@ESRI@@` @`0x71a65460` → COL
  `0x71a2db1c` → 身份虚表 `0x71a1211c`（13 槽，形状与 Polyline 族「七槽主
  虚表」同构：QI `0x717ea180`、共享 AddRef/Release、析构、类名 getter、
  GetClassID @+0x14、空桩）。GetClassID `0x717e8d60`（movq 写 CLSID 的站点
  `0x717e8d73`，即旧记录记下的「movq 站点 0x717e8d73 一带」）在 .rdata 全镜像
  **仅此一处**引用，且位于 +0x14 非持久化位。
- **无持久化虚表**：+0x0c 位（GetClassID 持久化形）无任何命中 → 无
  ISRPersistStream/IPersistStream 形虚表。QI `0x717ea180` 的 21 个 IID 比较
  常量逐个解出（ICircularArc `{991F5C10-BC4C-11D0-8303-0000F8034032}`、
  IConstructCircularArc `{91B6F492-C70A-11D0-8307-0000F8034032}`、
  IConstructCircularArc2 `{F4EE06A7-7854-485D-9689-914BE1B4E900}` 等），**不
  含** `{0000010C}`/`{00000109}`/`{E88C7DB0-…}` 任一持久化 IID；回退
  `FUN_719f1450` 仅处理 IUnknown（自返）、IServiceProvider（ErrorInfoImpl
  拆件）与一个 ClassIDImpl 拆件（esriSystem IClassID `{9F9650F1-…}` 候选），
  亦无持久化接口。
- **构造器**：`ESRI::Arc` 构造器六个重载（`0x717f4960`/`0x717f4a60`/
  `0x717f4bd0`/`0x717f4cc0`/`0x717f4dc0`/`0x717f4ec0`；`0x71a1211c` 的全部
  6 个 .text 安装点）只装配 IClone/IGeometry 族与 IWkbSegment/IWin32Shape/
  ICircularArc/IConstructCircularArc2/ICurve3 虚表，**无任何持久化虚表安装
  点**（抽查两处反编译一致）。
- **结论**：CircularArc 与 Path/Ring/Line/BezierCurve/EllipticArc 同判——
  **不实现 IPersistStream，从不独立出现在流内**（样本全流 0 实例两轮一致）；
  段仅以 Polyline/Polygon 记录的段描述条目形态存在（标志 `0x20000000` 变体，
  Load 路径恒不可达）。上轮「未解决事项 4」的最后一项就此闭合，该族无遗留。

## 样本交叉验证（辅助观察，沿用既有记录值）

样本首 PCS（键 30，体 560 字节）按本轮语义复读：版本 6（表末项）✓；WKT
455 字节（EPSG:4525，单位米）✓；4 域串全空 ✓；垂向标志 0（Save 恒写）✓；
u16 标志 0x0001 = 64 位高精度 ✓；7 double = XY 假原点 (31876800, -10002100)、
XY 比例因子 450265407.00157917（分辨率约 2.2e-9）、Z (−100000, 10000)、
M (−100000, 10000)（分辨率 1e-4）✓；3 尾随 double 0.001×3 = 默认容差公式
max(0.001/单位因子, 2.0/scale) 的复现值 ✓。该交叉验证为静态一致性核对，
未运行目标软件。

## 接线建议（本轮不改共享文件，供整合轮）

1. ISpatialReference 槽分派（Map 第 4 步、Envelope 第 4 段、Polyline/Polygon/
   Point/Multipoint 版本 >1 段、元素体版本 ==9 尾加块等既有「内嵌 PCS 复用
   spatial_reference 读取器」的调用点）扩展三分派：类标识 = PCS → 既有
   `read_projected_coordinate_system_10_1`；GCS →
   `read_geographic_coordinate_system_10_1`；Unknown →
   `read_unknown_coordinate_system_10_1`；其余类维持既有 Unsupported/停止
   语义。GCS 的垂向标志非零分支已按原生消费，不产生新停止点。
2. `tests/robustness.rs` 集中清单建议补两个新入口（试探性输入 → 明确错误）。
3. PCS 读取器本体不动（本轮不改既有模块）；若整合轮愿意统一，可把垂向段
   消费回填进 PCS 读取器（证据同一段 `0x719ea140`，两类型共享）——属可选
   优化，不阻塞。
4. Unknown 的 4 串「第 2 串被覆盖」为原生怪癖，模型层若只保留 3 个成员须在
   注释注明舍弃的是第 2 串。

## 未解决事项（本轮更新）

1. 域字符串（4 串）与 Unknown 四串的业务语义未定（候选：非 WKT 的补充描述；
   两类填充来源均未定位——helper BStrings 创建后未见写侧赋值点）。
2. ISRPersistStream slot 8（恒返 0x38）的接口成员名与用途；slot 9/10 的成
   员名（行为已闭合：无标志精度块的读写便捷槽）。
3. GCS Load 的 `对象+0xac` 非零守卫（报 `0x80004005`/0x1ca）语义；GCS 范围
   double 的非默认值形态与维护路径（`FUN_719c2e00` 未展开）。
4. 「假原点/比例因子」为按机制命名的候选；pe/sg 从投影推导假原点与 scale
   的数值路径未展开（sg 结构内派生字段 +0x20/+0x28 的快照单位语义未核）。
5. sg.dll/pe.dll 版本 10.1.0.679 与 Desktop 10.1.0.3035 的组件版本映射未核对
   （沿用既有未解决项；本轮 sg.dll 证据绑定 0x7fab0000 映像摘要）。
6. ISpatialReferenceTolerance 槽 4/8/12（旧版恢复用的「2/scale」形态）的成
   员命名、槽 15（`0x719d1da0`）行为，未逐一展开（不影响流布局）。
7. 样本无 GCS/Unknown/CircularArc 实例；全部静态取证，未运行 ArcMap 10.1，
   未在 QGIS 中核对，不声称已验证。

## 对既有文档记录的更正与闭合汇总（本轮不改动旧文档）

| 旧记录（出处） | 本轮结论 |
| --- | --- |
| 「`0x71a29ccc` 接口 IID 与名称未确认」（spatial-reference 2026-11-13 未解决事项） | ISRPersistStream `{E88C7DB0-9E84-11D2-BD00-0000F875BCCE}`；同一实现应答标准 IPersist/IPersistStream（11 槽；Unknown 例外，见上） |
| 「7 个域 double 与 3 个尾随 double 语义未解释；`DAT_71a66f20` 缺省值未读取」（同上） | 7 = XY/Z/M 假原点+比例因子；3 = XY/Z/M 容差；`DAT_71a66f20` = 4.94e-324 占位 |
| 「`pe_hvcoordsys_to_string` 与 `_ext` 的选择条件基于未解析寄存器」（同上） | 文档版本 <3 / ≥3 |
| 「保存方法第 4 项源为 UTF-16 指针经 Ordinal_149 转码（隐含 ANSI）」（同上 2026-11-13 第 4 项） | 无转码：流内为 u32 字节长 + BSTR 原样 UTF-16LE 字节（样本全空故未暴露） |
| 「对象流如何从该接口分派读写未闭合」（同上） | QI 三 IID 同槽（PCS/GCS）；Unknown 仅标准两 IID |
| 「读取方法函数末尾 `this-0x30` 接口虚表 `+0x74` 归属未闭合」（同上） | ISpatialReference3 虚表 29 号槽（语义候选加载后刷新） |
| 「save 版本 <6 时 `pe_projcs_from_string`、域串 ANSI」（同上） | 版本门与编码如上更正；版本表补充 GCS [3,4,7,7]、Unknown [1,1,3] |
| TODO「MXD 解析」第 76/77/78/149/161 行 | 接口 IID、标志语义、double 语义、ISpatialReference 槽实现类体（GCS/Unknown 分支）、CircularArc 持久化定位均闭合；「接入中间模型」仍开放（见接线建议） |
| 「`.?AVCircularArc` RTTI 未命中，虚表未定位」（geometry 2026-11-15 第二轮未解决事项 4） | COM 类实现为 `ESRI::Arc`（RTTI `.?AVArc@ESRI@@`）；无持久化虚表，构造性不存在，事项闭合 |

## UnknownCoordinateSystem 的四条持久化字符串

用 objdump 复核同一份 x86 Geometry.dll 10.1.0.3035（SHA-256
`c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`）。
主 QI `0x719eac95` 比较 `0x71a24884` 的
`4DA0FE00-1DD2-11B2-BF49-08002022F573`，命中后跳至
`0x719eaf50`，返回完整对象的 `+0x18`。构造器 `0x719edd44`
给该子对象安装虚表 `0x71a2c134`。同版本 esriGeometry.olb 的
TypeInfo 4 将接口定名为 ISpatialReferenceInfo，并给出下表的槽位。

| 属性 | 槽位 | getter | 完整对象成员 | Save 中的流序号 |
| --- | --- | --- | --- | --- |
| Name | `+0x0c` | `0x719ec3d0` | `+0x84` | 1 |
| Alias | `+0x10` | `0x719ec400` | `+0x78` | 无 |
| Abbreviation | `+0x14` | `0x719ec450` | `+0x7c` | 2、3 |
| Remarks | `+0x18` | `0x719ec480` | `+0x80` | 4 |

构造器 `0x719eddeb–0x719ede24` 初始化 Alias、Abbreviation 和
Remarks 为空串，Name 为 `Unknown`。Save 的四处成员读取分别在
`0x719ed367`、`0x719ed38e`、`0x719ed3be`、`0x719ed3df`。
Load 按上述相同顺序写成员，以第三串覆盖第二串。解析器保留原始四串。
旧写方是否曾在第二串保存 Alias 仍待原生证据。下文记录 10.1 的 Assign 写入路径。ArcMap 10.1 人工验证尚未执行。


### 10.1 保存到旧文档版本

Geometry.dll 10.1.0.3035 的 Save `0x719ed310` 在 `0x719ed320`
调用 `AfStreamGetDocumentVersion`，随后将版本表 `0x71a11d68`
（三个 short，`[1, 1, 3]`）传给 `AfStreamGetSaveVersion`。
写完对象保存版本后，四串由 `0x719ed367–0x719ed3ea` 的共用路径
写出。此段只在写失败时提前返回，第二串和第三串始终取持久化接口
this+0x4c，即完整对象 +0x7c 的 Abbreviation。首次按文档版本显式分支
发生在第四串之后的 `0x719ed40b`，该分支选择精度与容差布局。

因此即使 10.1 将对象写成 v1，第二串仍为 Abbreviation。原生早期版本
自身的写方仍需用早期二进制或可区分 Alias 与 Abbreviation 的实档验证。
本轮为静态取证，未在 ArcMap 或 QGIS 中人工验证。

### 早期文档的第二串证据

对 10.8 安装介质附带的辅助文档，按 `/Version` 的版本文本选出
42 份 10.1 之前的 MXD/MXT。扫描各 CFB 流内的 UnknownCoordinateSystem
CLSID 磁盘序，跳过 CLSID 后的四字节对象键，筛选版本 0–3 且后接
四条长度有效、带终止零的 UTF-16LE 字符串的候选。

| 文档版本文本 | 文件数 | CLSID 命中 | 四串结构候选 |
| --- | ---: | ---: | ---: |
| 8.1 | 1 | 0 | 0 |
| 9.2 | 12 | 1451 | 15 |
| 9.3 | 18 | 256 | 62 |
| 10.0 | 11 | 128 | 22 |

99 个候选均为 v3，第二、第三串均为空。CLSID 命中包含回引，
候选计数仅描述通过局部筛选的字节位置。

随后用 Rust 读取器从这 42 份文件的 `/PageLayout` 根开始，
共享对象键表按声明顺序读取元素，遇到首个无法继续的元素即停止。
仅统计 TextElement 或 MarkerElement 的 Point 几何中读到的首次
UnknownCoordinateSystem 定义。42 份均读到布局前缀，其中 9.2 和 10.0
各实际读到 5 个定义，均为 v3，第二、第三串均为空。8.1、9.3 在这条
遍历路径上未读到定义，不能据此判断它们的全部布局对象。
两份 LetterPortrait 的首元素均从 `0x1e` 读到 `0x336`，
9.2 MXT 的未知坐标系键为 79，10.0 MXD 的键为 76。
对应文件 SHA-256 分别为
`d26a4c3c73b60188bdae35e7824deacc957c9fab68c73a7fd4fb70f23e878012`
和 `b5cb3056eaee2ee42c9c9f8e7e660ce9822400c932857308ef02b63172a1a3a3`。
版本文本描述文件声明的格式，无法确定实际写方版本。
这批样本没有出现能区分 Alias 与 Abbreviation 的第二、第三串，
早期写方的字段来源仍待取证。

### IClone::Assign 的字符串写入路径

同一 ArcMap 10.1 Geometry.dll 的 IClone 查询比较点 `0x719eace5`
引用 IID `9BFF8AEB-E415-11D0-943C-080009EEBECB`，返回完整对象
+0x10。虚表 `0x71a2c0f8` 的槽 4 为 Assign（`0x719ebfb0`）。
该方法先查询源对象的持久化接口，再于 `0x719ec02f` 取源 CLSID，
并在 `0x719ec044` 和 UnknownCoordinateSystem 的 CLSID 比较。
通过后查询源 ISpatialReference（`0x719ec087`）。下面四次虚调用
通过输出指针把字符串写入目标成员，偏移已按完整对象换算。

| 调用点 | 源接口槽偏移 | 目标成员 | 属性 |
| --- | --- | --- | --- |
| `0x719ec0ff` | +0x10 | +0x78 | Alias |
| `0x719ec145` | +0x14 | +0x7c | Abbreviation |
| `0x719ec168` | +0x18 | +0x80 | Remarks |
| `0x719ec18b` | +0x0c | +0x84 | Name |

该路径可改写 Alias，但 10.1 Save 仍两次取 Abbreviation。另由
esriGeometry.olb TypeInfo 77 和主 QI 确认 IConstructUnknownCoordinateSystem
对应完整对象 +0x48，槽 3 为 `0x719eaf90`。这个入口从输入空间参考取
范围或使用常量范围，经 `0x7198f050` 取得对象后，再于 `0x719eb055`
委派给 Assign。该入口自身不接收字符串。下文核对其缓存来源。

`0x7198f050` 使用 `0x71a6d8bc` 起的五个缓存槽。空槽调用默认构造器
（`0x7198f135`），在 `0x7198f15a` 存入完整对象 +0x24 的接口。
`0x7198f227` 调用 `WKSEnvelope::Contains` 选择能容纳输入范围的槽，
未匹配时调整末槽的数值范围。创建时的虚调用 +0x58/+0x60/+0x68
分别经调整器跳到 `0x719ec750`、`0x719eca30`、`0x719ecbf0`。
缓存工厂按接口在 `0x7198f300` 返回命中项并增加引用计数。
调用者经 `0x719c3a70` 查询该对象的 IClone，再用其作为 Assign 的源。
因此在首次创建缓存项时，构造接口所用的字符串来自默认构造器，
即 Name 为 Unknown，其余三项为空。其他调用者能否改写已缓存对象的
字符串仍待运行验证。本轮未在 ArcMap 或 QGIS 中人工验证。

### 工厂、复制与介质对象的字符串

同一 Geometry.dll 中，UnknownCoordinateSystem 的类工厂 RTTI
虚表 `0x71a1dc38` 的 +0x0c 为 `0x718c4250`。
入口分别于 `0x718c42b4`、`0x718c4337` 调用默认构造器，
因此两条工厂路径都用 Name 为 Unknown、其余三项为空的初值。
`0x7198f135` 的缓存创建路径也调用默认构造器，随后的
`0x7198f182–0x7198f1c4` 在 +0x24 接口上设置数值参数。
完整对象 +0x10 的虚表 `0x71a2c0f8` 在 +0x0c 提供
`0x719ebf60`，调用复制构造器。它在
`0x719edfde–0x719ee028` 逐项复制 Alias、Abbreviation、
Remarks、Name。此前列出的 `0x719ee050` 则调用 BSTR 释放，
属于析构入口。

10.1 Desktop 介质的 CentralUSA.mxd（SHA-256
`b3b594837ab29c9435c2214793d1dcc65d2985a73d26b6619d83704f99005bac`）
的 /Version 为 10.1/19。由 /PageLayout 根和首个文本元素读取到
`0x96e` 的空间参考键 743，体从 `0x982` 起为 v3，
四串依次为 Unknown、空、空、空；后继折线回引同一键。
这是实档边界验证。两个空串无法区分旧写方是否使用 Alias，
也尚未核对属性编辑路径或在 ArcMap 10.1 和 QGIS 中人工验证。

### GCS 与 UnknownCoordinateSystem 的有符号版本门

Geometry.dll 10.1.0.3035（摘要同上）的 objdump 指令显示，
GCS 的 `0x719bece7/0x719becef`、UnknownCoordinateSystem 的
`0x719ecdaa/0x719ecdb0` 以有符号 short 限定上限 8 和 3。
GCS 的 `0x719beed2`、`0x719bef90`、`0x719bf02d`、
`0x719bf051`、`0x719bf07c` 按有符号版本选择后续段。
UnknownCoordinateSystem 在 `0x719ece82` 和 `0x719ed290`
按有符号版本跳过精度段，`0x719ecfb7` 所在版本门也跳过流内
容差。因此负版本依次消费 GCS 的 WKT 或 UnknownCoordinateSystem
的四串，而后续字段留给宿主。Rust 保留原始版本字，使用有符号
比较分派。构造输入覆盖负版本、零版本、两条不同的中间字符串，
以及对象末端和宿主后继字节的边界。
这项取证不能区分早期写方的 Alias 与 Abbreviation。真实负版本
工程及 ArcMap 人工验证仍待补充。

### PCS 的有符号版本门

同一 Geometry.dll 10.1.0.3035 的 PCS Load `0x719d3230`
以 `0x719d3291/0x719d3299` 的有符号比较限定上限 7。
`0x719d334c` 用有符号版本选择原生投影初始化分支，
`0x719d3442`、`0x719d34f1`、`0x719d3512` 和 `0x719d353a`
分别用有符号条件跳过域串、各精度形态及流内容差。
Rust 读取器保留负版本的原始位模式，从 WKT 结束位置交还宿主。
构造测试验证负版本、v0、v1 同样只消费 WKT，以及正值超限回滚。
真实负版本工程及 ArcMap 人工验证仍待补充。

### 共用垂向单位段的有符号版本门

同一 Geometry.dll 10.1.0.3035 在 `0x719ea178/0x719ea180`
对单位段 short 使用有符号上限 2。进入 `0x719ea1af` 后各接受
版本共用一条长度和文本读取路径。Rust 保留原始版本字并沿用长度及
资源限额检查。构造测试通过 GCS 和 PCS 宿主验证负版本，GCS
还覆盖 0–2、正值超限和后继字段的边界。真实负版本垂向单位段
及 ArcMap 人工验证仍待补充。
