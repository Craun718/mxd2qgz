# ArcMap 10.1 FeatureClassName 体 IDatasetName / ITopologyName 选择子分支（FeatureDatasetName / TopologyName 新对象体）

记录日期：2026-11-14。对应 TODO「MXD 解析」节第 76 行开放项中的
「IDatasetName/ITopologyName 新对象体」。全部为静态逆向观察，未运行
ArcMap，未做人工目视验收；结论按证据强度标注，开放的仍开放。

## 任务与方法

- 背景：FeatureClassName（FCN）体（GdbCore.dll `FdbFeatureClassName::Load
  0x709b5080`）字段序已于第六轮闭合（见
  [arcmap-10.1-map-object.md](arcmap-10.1-map-object.md) 第六轮节），其中
  第 ⑦ 步 IWorkspaceName/IDatasetName 选择子与第 ⑧ 步（版本 >1）
  ITopologyName 选择子在选择子非 0 时以**新对象体**出现，第六轮未闭合，
  `feature_layer` 读取器在两处对象头即停止。
- 本轮方法：提取路径/证据先查第六轮产物（`/tmp/r3/gdbcore_work/`、
  `/tmp/r3/ghproj_gdbcore/` 项目缓存仍在），再按 AGENTS.md 要求自建目录
  重新提取并核对身份；类定位沿用第六轮方法（CLSID 数据引用 → GetClassID
  → 同虚表 +0x14/+0x18 槽），虚表归属经 RTTI 完整对象定位符（COL）链确认；
  字段序经 Load/Save 反编译与汇编逐指令核对；样本对位用
  `temp/r6-maps.bin` 全流扫描。
- 工具：REA MCP（Ghidra 12.2 提供者，JDK 25，用户级配置）+ 本机 pyghidra
  3.2.0（Ghidra 12.2 + JDK 25，复用第六轮项目缓存 `/tmp/r3/ghproj_gdbcore`）。
  REA 共享服务器本轮与其他研究代理（Map.dll 会话）多次互相抢占活动目标：
  本轮在 GdbCore 绑定状态下成功取得两条一手证据（`DAT_70ab8574` IID 字节
  读取、FDN Load `0x709b7c90` 反编译 ev_78efd183…、TN Load `0x7095e5c0`
  反编译 ev_4398b31a…），其余查询在目标被切换后改用本机 pyghidra（同一
  Ghidra 12.2 引擎、同一文件摘要），与第一轮研究记录的降级先例一致；结束
  时已调用 `close_binary` 释放服务器。

## 文件身份（本轮提取与复核）

| 文件 | 来源 | SHA-256 | 版本 |
| --- | --- | --- | --- |
| GdbCore.dll | `Setup1.cab` 条目 `GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`，提取至 `temp/agent-dataset-name/`（2836992 字节） | `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4` | pefile VS_FIXEDFILEINFO `10.1.0.3035`，CompanyName=Esri，FileDescription=GdbCore DLL，ImageBase `0x708d0000` |

SHA-256 与第六轮基线一致。地址均为该 PE 静态映像虚拟地址。

## 选择子语义复核（FCN 体 `0x709b5080`，证据：确定的）

第六轮字段序经本轮文件字节与 REA 读取复核：

- 第 ⑦ 步 int16 选择子：`==0` → LoadObject（IID `DAT_70ab8554` =
  `{FADD975C-E36F-11D1-AA81-00C04FA33A15}`）；`!=0` → LoadObject（IID
  `DAT_70ab8574` = `{89E75917-C287-11D1-AA77-00C04FA33A15}`）并对 obj+0x3c
  接口做虚表 +0x20 调用（通知，不读流）。选择子为 16 位值，原生只区分
  零与非零，无其他取值语义（无「未知选择子」错误分支）。
- 第 ⑧ 步（仅版本 >1）int16 第二选择子：`!=0` → LoadObject（IID
  `DAT_70ac9a70` = `{505B6FDD-9E6E-42F9-B705-53355B2F1DEF}`）。
- 三个 IID 的类型库名称（`esriGeoDatabase.olb` dump，wine `LoadTypeLibEx`
  产物，typekind=3）：`FADD975C…`=**IWorkspaceName**、`89E75917…`=
  **IDatasetName**、`505B6FDD…`=**ITopologyName**。

新对象的类身份由流内 16 字节对象头决定（ObjectStream 查表创建），请求
IID 只约束接口。两分支的预期 coclass 由本轮闭合如下。

## FeatureDatasetName 体（证据：确定的；样本无字节佐证）

类定名（三方互证，与 FCN/WorkspaceName 第六轮方法同构）：

| 证据 | 地址/值 |
| --- | --- |
| esri.clsid.ecfg 声明 | CLSID `{198846CF-CA42-11D1-AA7C-00C04FA33A15}` → FeatureDatasetName（模块 GdbCore.dll；与 FeatureClassName `{198846D0-…}` 相邻同族） |
| GetClassID 写出 | `0x709b62c0`：`*out = {198846CF-…}`（两条 8 字节立即值，反编译核对） |
| RTTI | TD `0x70b34a90` 名 `.?AVFdbFeatureDatasetName@@`（10.1 C++ 实现类），经 COL `0x70af5124`（col_sig=0）与 IPersistStream 虚表 `0x70ad04ec` 相连 |

IPersistStream 虚表 `0x70ad04ec` 槽序：+0x0c GetClassID `0x709b62c0`、
+0x10 IsDirty `0x70a1a280`（跨类共享）、**+0x14 Load `0x709b7c90`**、
**+0x18 Save `0x709b7d50`**、+0x1c GetSizeMax `0x709dbdf0`。

Load `0x709b7c90` 字段序（Save `0x709b7d50` 逐字段对称；汇编逐指令核对）：

| # | 流内字段 | 版本门 | 证据 |
| --- | --- | --- | --- |
| 1 | int16 版本 | `cmp word ptr [ebp+0xc],1; jle` **有符号**，>1 拒绝（0x80004005） | Save 恒写静态常量 `DAT_70aaac34` = short 1（`01 00`） |
| 2 | AfCore String → obj+0x0c | | `String::Read` 间接调用 ×4（偏移 0x0c/0x10/0x14/0x18） |
| 3 | AfCore String → obj+0x10 | | |
| 4 | AfCore String → obj+0x14 | | |
| 5 | AfCore String → obj+0x18 | | |
| 6 | 1 字节标志 → obj+0x1c | | `push 1`（长度 1）；原生按 bool 存储 |
| 7 | 嵌套对象（IID `{FADD975C-…}`=IWorkspaceName，与 FCN 选择子 ==0 分支同值）→ obj+0x20 | | `push 0x70ab8554`；Save 读 `[edi+0x20]` 调 SaveObject(…, 1)，读写对称 |

无版本分支（仅版本 0/1 两态，Save 恒写 1）。四个 String 与标志字节的
业务语义未定性（按 FCN 四字符串家族类推：名称/第二名称/类型描述/数据集名
均为候选）。

## TopologyName 体（证据：确定的；样本无字节佐证）

类定名（三方互证）：

| 证据 | 地址/值 |
| --- | --- |
| esri.clsid.ecfg 声明 | CLSID `{9FCA50EC-C413-440C-B453-49A591440096}` → TopologyName（模块 GdbCore.dll） |
| GetClassID 写出 | `0x7095dee0`：`*out = {9FCA50EC-…}` |
| RTTI | TD `0x70b33680` 名 `.?AVTopologyName@@`，经 COL `0x70af02fc` 与 IPersistStream 虚表 `0x70aca460` 相连 |

IPersistStream 虚表 `0x70aca460`：+0x0c GetClassID `0x7095dee0`、+0x10
IsDirty `0x70a1a280`、**+0x14 Load `0x7095e5c0`**、**+0x18 Save
`0x7095e550`**。

Load `0x7095e5c0` 字段序（Save `0x7095e550` 对称；汇编核对）：

| # | 流内字段 | 版本门 | 证据 |
| --- | --- | --- | --- |
| 1 | int16 版本 | `cmp word ptr [ebp+0xc],1; jg fail` **有符号**，>1 拒绝（0x80004005；AfStreamRead 失败同值） | Save 恒写静态常量 `DAT_70ab7aa0` = short 1 |
| 2 | AfCore String → obj+0x0c | | `String::Read` ×2 |
| 3 | AfCore String → obj+0x10 | | |
| 4 | 嵌套对象（IID `{89E75917-…}`=IDatasetName，与 FCN 选择子 !=0 分支同值）→ obj+0x14 | | 装载前释放旧 +0x14（Release）；`push 0x70ab8574`；装载成功后对 obj+0x14 对象做虚表 +0x20 调用——与 FCN 数据集分支 obj+0x3c 虚表 +0x20 为**同一通知模式**（不读流，接口语义未定名）；Save 读 `[edi+0x14]` 调 SaveObject(…, docVersion 参数透传) |

两个 String 的业务语义未定性（候选：名称/第二名称）。

## 附带观察（GdbCore 其他 CLSID 引用者，避免后续误判）

- `0x70944800` 同时引用 FeatureDatasetName 与 TopologyName 两个 CLSID（及
  更多类名数据），为跨类 CLSID→名称查找类辅助函数，非持久化入口。
- `0x70a7e630`（虚表族 `0x70a88170`）同为两 CLSID 的引用者，属类表工厂
  （`ClassFactory<>`）对象族，非持久化入口。
- TopologyName CLSID 另有两个引用函数 `0x7095f770`（第六轮工作目录曾误
  记为候选归属）与 `0x7095b990`，其所在虚表（`0x70abdee8`/`0x70aca28c`）
  +0x14/+0x18 槽指向 `0x7093e060`/`0x70983200`，非 IPersistStream 布局，
  归属其他接口（候选 TopologyWorkspaceExtension 等，未展开）。

## 样本对位（辅助观察）

`temp/r6-maps.bin`（152941 字节，第六轮导出）全流扫描（脚本
`temp/agent-dataset-name/scripts/scan_sample2.py`）：

- FeatureDatasetName CLSID 内存序：**0 命中**；TopologyName CLSID：**0 命中**。
  两分支的新对象体在仓库参考样本中均未出现。
- 11 个 FCN 对象头（键 33/219/1556/1620/1677/1735/1793/1876/1954/2025/
  2102）：第 ⑦ 步选择子**全部为 0**（IWorkspaceName 分支）；其中 10 个可
  完整遍历（工作空间体内 IPropertySet 为 16 字节空标记、工厂标志 1、
  尾随 int32 0）的 FCN 版本 >1 第二选择子**全部为 0**。
- 例外：FCN@`0x218bc`（键 2025）的工作空间体内 IPropertySet 槽为**真新
  对象**（键 2027，类标识非空）——即样本中存在一处 WorkspaceName 体内
  PropertySet 新对象（PropertySet 体未闭合，`feature_layer` 既有停止点），
  其后字段（含该 FCN 的第二选择子）从流上不可达。此为独立观察，不改变
  两分支「样本未出现」的结论。

由此：本轮两个新对象体的字段序**仅有反编译一手证据，无样本字节佐证**；
`dataset_name` 读取器的全部单元测试使用合成输入。

## 实现接线（src/dataset_name.rs、tests/dataset_name.rs）

- 新模块 `dataset_name`（`lib.rs` 已声明，本轮未改共享文件）：
  - `FEATURE_DATASET_NAME_CLASS_ID` / `TOPOLOGY_NAME_CLASS_ID` 常量（证据
    见上）；
  - `read_feature_dataset_name_body_10_1`：版本门（有符号 >1 拒绝，负值
    位模式一并拒绝、较原生更严）、四字符串（max_bytes 限额）、1 字节标志
    原样保留、IWorkspaceName 槽分类——新 WorkspaceName 对象复用
    `feature_layer::read_workspace_name_body_10_1`（第六轮），体内
    PropertySet 新对象停止透传，非 WorkspaceName 类新对象停止并回退到槽位
    起点；
  - `read_topology_name_body_10_1`：版本门同上、两字符串、IDatasetName 槽
    分类——新 FeatureDatasetName 对象递归读取（体内停止透传），其余类停止
    并回退；
  - 事务性：全部字段在 trial 游标与 trial 登记表上读取，失败不推进游标、
    不修改登记表；停止点是成功返回。
- 测试（12 项）：完整链 TN→FDN→WS、空标记/回引/未知类槽位、两处停止点
  的原因与游标锚点、版本边界（0/1 通过，2 与 0xFFFF 拒绝）、逐前缀截断
  事务性、限额转发、非规范标志位原样保留。
- **未接线**：`feature_layer` 的 FCN 读取器在两槽位新对象处停止的既有
  行为不变（该文件为共享文件，本轮不修改）；把
  `FeatureClassNameStop10_1::DatasetNameNewObject` /
  `TopologyNameNewObject` 停止点接到本模块读取器属后续共享文件改动，
  见开放项。

## 开放项

1. 两分支样本形态未获得：需含要素数据集内要素类或拓扑图层的 MXD 样本，
   或写方向运行取证；当前布局结论仅有静态反编译证据。
2. FDN 四字符串与标志字节、TN 两字符串的业务语义未定性（候选命名按
   FCN/WS 家族类推）；TN 槽装载后的虚表 +0x20 通知调用的接口语义未定名
   （与 FCN obj+0x3c 同型）。
3. IDatasetName 槽可能出现的其他实现类未闭合：esri.clsid 声明的
   UnknownDatasetName `{06783DB1-…2E}`、GeometricNetworkName
   `{F452B4D2-…}`、RelationshipClassName `{311C2EE1-…}`、TableName
   `{06783DB1-…20}` 及 FeatureClassName 自身（若基类 FdbDatasetName 实现
   IDatasetName，派生类均可能出现在该槽）；读取器在非 FeatureDatasetName
   类对象头处停止。
4. 版本 0 分支无字节佐证（原生门为有符号 ≤1，接受 0 与负值位模式；Save
   恒写 1）；读取器按 0..=1 接受、负值拒绝（较原生更严）。
5. 接线开放：FCN 读取器两槽位停止点 → 本模块读取器的分派（需修改共享
   文件 `feature_layer.rs` 或经 `mxd_maps` 层包装）；接入后 `parse` 命令
   对含要素数据集/拓扑样本的解析深度才会受益。
6. 本轮附带观察（FCN@0x218bc 工作空间体内 PropertySet 新对象键 2027）属
   `feature_layer` 既有停止点语义，PropertySet 体字段序仍开放（第六轮
   开放项，未因本轮改变）。
7. 全部结论为静态观察；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已
   验证。

## 证据与复现

- 二进制：`temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA`
  （身份见首节表）。
- REA 证据（GdbCore 绑定状态下）：IID 字节 `DAT_70ab8574`（ev_1c153cbb…）、
  FDN Load 反编译（ev_78efd183…，父证据 ev_4f33e987…）、TN Load 反编译
  （ev_4398b31a…）。
- 本机 pyghidra 产物（脚本在 `temp/agent-dataset-name/scripts/`，输出 JSON
  同目录）：`locate_clsids2.py`（CLSID 数据命中与引用函数、RTTI 名枚举）、
  `resolve_vtables.py`/`decomp_candidates.py`（虚表槽与反编译）、
  `verify_details.py`（保存版本常量、COL 链、版本门汇编头）、`full_asm.py`
  （四函数全量汇编）、`thunks.py`（副虚表排除）、`scan_sample2.py`
  （样本对位）。项目缓存 `/tmp/r3/ghproj_gdbcore`（会话临时目录，未入库）。
- 原始 MXD 样本与诊断输出可能包含敏感路径；本记录中的合成路径与样本
  字符串已检查，公开分享前仍需复核脱敏。
