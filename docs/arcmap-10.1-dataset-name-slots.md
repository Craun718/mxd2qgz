# ArcMap 10.1 IDatasetName 槽实现类体族（GeometricNetworkName / RelationshipClassName / MemoryRelationshipClassName / QueryTableName / RelQueryTableName / DBTableName）与不可持久化类（TableName / UnknownDatasetName / ObjectClassName / FeatureQueryName / TableQueryName）

> SDK Java LinearRef.mxd 的首个 StandaloneTable 从 Map 根顺序到达后，IName 槽在
> `/Maps:0x6a66` 出现 TableName CLSID `{06783DB1-E5EE-11D1-B0A2-0000F8780820}`、
> 新键 332，后面有对象体字节。见 [介质宿主复核](mxd-terra-round8-old.md)。
> [后续原生复核](mxd-terra-round8-tablename.md)已找到 GdbCore 虚表到 GdbCoreLib 基类 Load/Save 的跳板，
> 撤销 TableName“不可持久化”的推断；其他类的结论不由该实档外推。

记录日期：2026-11-16。本轮（并行轮 B 轮）闭合 TODO「MXD 解析」中
「IDatasetName 槽其余实现类」族。全部为静态取证（本机 pyghidra，Ghidra
12.2 + JDK 25，打开第六轮 GdbCore 工程缓存副本 `/tmp/gh-b-dsnames/`，逐锚
复核），未运行 ArcMap 10.1，未人工目视验收。仓库参考样本
`samples/测试底图.mxd` **全文件**按 16 字节 CLSID 内存序原样字节扫描，
本轮全部 11 个类 **0 命中**（对照组 WorkspaceName 72 命中、
FeatureClassName 23 命中，扫描方法有效），即**无样本字节佐证**；字段序全
部来自反编译，经 Save 侧逐字段对称互证；合成输入测试见
`src/objects/dataset_name_more.rs` 模块内 `#[cfg(test)]`。本文地址均为
GdbCore.dll 10.1.0.3035 静态映像 VA。无本机绝对路径。

## 文件身份（与前轮基线一致）

- GdbCore.dll：`Setup1.cab` 条目 `GdbCore.dll.FA783766_…`，2836992 字节，
  SHA-256 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`
  （与 `docs/arcmap-10.1-dataset-name.md` 首节基线一致，本轮拷贝至本代理
  工作区后复核），ImageBase `0x708d0000`。
- 类名/模块声明：`esri.clsid.ecfg` 解码表（`esri.clsid.xml`，SHA-256
  `170ae1f8…`，第六轮产物，本轮复核）。GdbCore.dll 声明的 `*Name` 族共 15
  类，本轮覆盖关系：

| 类 | CLSID | 状态 |
| --- | --- | --- |
| FeatureClassName `{198846D0-…}` | 198846d0-ca42-11d1-aa7c-00c04fa33a15 | 已闭合（第六轮，host 在 feature_layer/dataset_name） |
| FeatureDatasetName `{198846CF-…}` | 198846cf-ca42-11d1-aa7c-00c04fa33a15 | 已闭合（第六轮） |
| TopologyName `{9FCA50EC-…}` | 9fca50ec-c413-440c-b453-49a591440096 | 已闭合（第六轮） |
| WorkspaceName `{5A350011-…}` | 5a350011-e371-11d1-aa82-00c04fa33a15 | 已闭合（第五/六轮） |
| **GeometricNetworkName** `{F452B4D2-9A95-11D2-AACF-00C04FA37B82}` | f452b4d2-9a95-11d2-aacf-00c04fa37b82 | **本轮闭合** |
| **RelationshipClassName** `{311C2EE1-DA65-11D2-8A54-000000000000}` | 311c2ee1-da65-11d2-8a54-000000000000 | **本轮闭合** |
| **MemoryRelationshipClassName** `{6DBA211B-EBDB-11D3-9F84-00C04F6BC886}` | 6dba211b-ebdb-11d3-9f84-00c04f6bc886 | **本轮闭合** |
| **QueryTableName** `{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}` | 793873de-c0bb-48f4-a6bf-f6f83fa3c1bc | **本轮闭合** |
| **RelQueryTableName** `{DAB3EE10-0F92-455D-8AA2-3D4ADE5B2F7D}` | dab3ee10-0f92-455d-8aa2-3d4ade5b2f7d | **本轮闭合** |
| **DBTableName** `{4C9307B2-C125-4497-BA95-AAE875553B06}` | 4c9307b2-c125-4497-ba95-aae875553b06 | **本轮闭合** |
| TableName `{06783DB1-E5EE-11D1-B0A2-0000F8780820}` | 06783db1-e5ee-11d1-b0a2-0000f8780820 | 本轮判明**无流持久化实现**（见「不可持久化」节） |
| UnknownDatasetName `{06783DB1-…082E}` | 06783db1-e5ee-11d1-b0a2-0000f878082e | 同上 |
| ObjectClassName `{905CEC64-0B2C-11D4-A060-00C04F6BDD84}` | 905cec64-0b2c-11d4-a060-00c04f6bdd84 | 同上 |
| FeatureQueryName `{81CAA5D9-90C5-11D3-9F7B-00C04F6BC886}` | 81caa5d9-90c5-11d3-9f7b-00c04f6bc886 | 持久化路径未闭合（IPersistVariant 形态，见下） |
| TableQueryName `{81CAA5D8-90C5-11D3-9F7B-00C04F6BC886}` | 81caa5d8-90c5-11d3-9f7b-00c04f6bc886 | 同上 |

## 定位方法与判据

1. CLSID 数据字节搜索（16 字节内存序，全镜像）→ 数据命中；
2. 数据命中的代码引用（Ghidra 引用管理器 + 4 字节立即数全镜像搜）；
3. RTTI TypeDescriptor（名 `.?AV<类名>@@`，名在 TD+8）→ COL（TD 值命中位
   置 −0xc，校验 [COL]==0 签名且 [COL+0xc]==TD）→ vftable（COL 指针命中位
   置 +4，即虚表 [-1] 槽）→ 虚表槽 +0x0c/+0x14/+0x18 = GetClassID/Load/Save
   候选；
4. GetClassID 反编译核对 CLSID 写出（立即数对或 .rdata MOVQ），Load/Save
   反编译核对 AfStreamRead/AfStreamWrite 流框架与逐字段对称。

方法备注（修正本轮早期尝试）：32 位 MSVC 的 RTTI BaseClassDescriptor 数组
（BCDD，`{TD, mdisp, pdisp, vdisp, attrs, CHD}` 24 字节）同样内嵌 TD 值，
「TD 命中位 −0xc 且 [位]==0」会产生**伪 COL**（如 `0x70af599c`），因此每个
候选虚表必须以 GetClassID 反编译写出 CLSID 收尾确认，不能只信 COL 链。
类工厂登记表（`0x70a807xx..0x70a808xx` 区域，数据引用 CLSID 地址）不构成
GetClassID 引用（第六轮 `0x70a7e630` 同族结论本轮再证）。

## 六类闭合（证据：确定的——反编译 + Save 对称 + GetClassID 写出 + RTTI 链）

| 类 | GetClassID（写出形态） | IPersistStream 虚表 | [-1] COL | RTTI TD | Load | Save |
| --- | --- | --- | --- | --- | --- | --- |
| GeometricNetworkName | `0x709b8560`（立即数对 `0x11d29a95f452b4d2`/`0x827ba34fc000cfaa`） | `0x70ad0648` | `0x70af54c4` | `0x70b34b48` `.?AVFdbGeometricNetworkName@@` | `0x709b8ca0` | `0x709b8d50` |
| RelationshipClassName | `0x70a3ab30`（`0x11d2da65311c2ee1`/`0x548a`） | `0x70adba24` | `0x70afdf00` | `0x70b36a2c` `.?AVFdbRelationshipClassName@@` | `0x70a3b330` | `0x70a3b4b0` |
| MemoryRelationshipClassName | `0x70a402a0`（`0x11d3ebdb6dba211b`/`0x86c86b4fc000849f`） | `0x70adbe8c` | `0x70afe4d0` | `0x70b36bd4` `.?AVMemoryRelationshipClassName@@` | `0x70a402e0` | `0x70a40440` |
| QueryTableName | `0x709329d0`（`0x48f4c0bb793873de`/`0xbcc1a33ff8f6bfa6`） | `0x70ac5128` | `0x70aeb9d8` | `0x70b32138` `.?AVQueryTableName@@` | `0x70932a10` | `0x70932ae0` |
| RelQueryTableName | `0x70a5a4f0`（`0x455d0f92dab3ee10`/`0x7d2f5bde4a3da28a`） | `0x70add10c` | `0x70affc5c` | `0x70b3717c` `.?AVRelQueryTableName@@` | `0x70a5a530` | `0x70a5a6f0` |
| DBTableName | `0x708d7a00`（.rdata `0x70ab8544`/`0x70ab854c` 双 MOVQ） | `0x70ab8480` | `0x70adec24` | `0x70b2d2b0` `.?AVDBTableName@@` | `0x708d7a40` | `0x708d7b20` |

注意：GdbCore 的 `IsDirty` 共享实现 `0x70a1a280`（`XOR EAX,EAX; RET 4`）与
`GetSizeMax` 共享 `0x709dbdf0`（`XOR EAX,EAX; RET 8`）只在部分类虚表出现
（FGN/QueryTN 的 +0x1c 为类自有函数 `0x7092c220`/`0x70932b90`），不能单靠
共享槽位筛选 IPersistStream 虚表，必须回到 GetClassID 反编译。

## 字段序（Load 反编译；Save 逐字段对称互证）

**GeometricNetworkName::Load `0x709b8ca0`**（版本门 `cmp word,1; jle` 有符
号，>1 拒绝 `0x80004005`；Save 恒写静态常量 `0x70aa1bb4` = short 1）：

| # | 字段 | 对象偏移 |
| --- | --- | --- |
| ① | int16 版本 | — |
| ② | AfCore String ×3 | this+0x10/0x14/0x18（Save 写侧 LEA 证实） |
| ③ | LoadObject（请求 IID `{89E75917-…}`=IDatasetName，`0x70ab8574`）→ obj+0x1c；装载成功后对槽对象做虚表 +0x20 调用（通知，不读流，与 FCN/TN 同型） | obj+0x1c（Save `SaveObject([obj+0x1c], 1)`） |

**RelationshipClassName::Load `0x70a3b330`**（版本门有符号 >1 拒绝；Save
恒写静态常量 `0x70a88fec` = short 1）：

| # | 字段 | 对象偏移 |
| --- | --- | --- |
| ① | int16 版本 | — |
| ② | AfCore String ×3 | this+0x14/0x18/0x1c |
| ③ | int32 | obj+0x20 |
| ④ | int32 | obj+0x24 |
| ⑤ | int16（AfStreamRead 短读长度==2 已检） | obj+0x28 |
| ⑥ | int16 | obj+0x2a |
| ⑦ | int16 选择子（读长==2 已检；原生只区分零/非零）：==0 → LoadObject（请求 IID `{FADD975C-…}`=IWorkspaceName `0x70ab8554`）→ obj+0x30；!=0 → LoadObject（IID IDatasetName `0x70ab8574`）→ obj+0x2c + 虚表 +0x20 通知。Save 侧选择子写 `(obj+0x2c != 0)` | obj+0x30 / obj+0x2c |

**MemoryRelationshipClassName::Load `0x70a402e0`**（版本门有符号 >1 拒绝；
Save 恒写静态常量 `0x70a88fe8` = short 1）：

| # | 字段 | 对象偏移 |
| --- | --- | --- |
| ① | int16 版本 | — |
| ② | AfCore String ×3 | — |
| ③ | int32 | obj+0x10 |
| ④ | int32 | obj+0x14 |
| ⑤ | int16 | obj+0x18 |
| ⑥ | int16 | obj+0x1a |
| ⑦ | LoadObject（请求 IID `{4BA33331-D55F-11D1-8882-0000F877762D}`，字节 `0x70ab84e4`，接口语义未定名） | obj+0x1c（Save `SaveObject([obj+0x1c],1)`） |
| ⑧ | LoadObject（同 IID） | obj+0x20 |
| ⑨ | AfCore String ×4 | — |

**QueryTableName::Load `0x70932a10`**（版本门**无符号**：汇编
`mov ecx,[ver]; dec ecx; cmp cx,1; ja fail`，即仅版本 ∈ {1,2} 通过，版本 0
被原生拒绝——与同族有符号门不同；Save 经 `AfStreamGetSaveVersion(文档版
本, 6, 0x70ab7b2c)` 查表写版本，表 = (0xffff, 0xffff, 0xffff, 0xffff, 1, 2)，
即真实文档版本写 1 或 2）：

| # | 字段 | 对象偏移 |
| --- | --- | --- |
| ① | int16 版本 | — |
| ② | AfCore String ×3 | — |
| ③ | LoadObject（IID IWorkspaceName）→ obj+0x1c | obj+0x1c（Save 先写本槽） |
| ④ | LoadObject（IID `{2E060684-E14C-44D3-858A-442AB6A0A863}`，字节 `0x70ac5158`，语义未定名）→ obj+0x18；装载后若非空做虚表 +0x40 查询（不读流），结果置内存 bool obj+0x14 | obj+0x18（Save 后写本槽） |

**RelQueryTableName::Load `0x70a5a530`**（版本门有符号 >3 拒绝；Save 经
`AfStreamGetSaveVersion(文档版本, 2, 0x70a88fd8)`，表 = (2, 3)，即真实文档
版本写 2 或 3，读侧 0/1 短体无写侧来源）：

| # | 字段 | 版本门 | 对象偏移 |
| --- | --- | --- | --- |
| ① | int16 版本 | 恒有 | — |
| ② | AfCore String ×2 | 恒有 | — |
| ③ | LoadObject（IID `{4BA33331-…}`）→ obj+0x10 | 恒有 | obj+0x10 |
| ④ | int16（AfStreamRead PAF 短型） | 恒有 | obj+0x14 |
| ⑤ | LoadObject（IID `{FDFEBD93-ED75-11D0-9A95-080009EC734B}`，字节 `0x70ac4f28`）→ obj+0x18 | 恒有 | obj+0x18 |
| ⑥ | LoadObject（IID `{F7AD47D1-D55E-11D1-8882-0000F877762D}`，字节 `0x70ac4d88`）→ obj+0x1c | 恒有 | obj+0x1c |
| ⑦ | AfCore String | 恒有 | — |
| ⑧ | int16 | 恒有 | obj+0x24 |
| ⑨ | int16 | 版本 ≥2（<2 原生置内存 0，流内无字段） | obj+0x26 |
| ⑩ | int16 | 版本 ≥3 | obj+0x28 |

**DBTableName::Load `0x708d7a40`**（**无版本字段**——Load 首字段即 String，
Save 首字段即 String::Write，逐字段对称；这是六类中唯一无版本者）：

| # | 字段 | 对象偏移 |
| --- | --- | --- |
| ① | AfCore String ×3 | — |
| ② | LoadObject（IID IWorkspaceName）→ obj+0x1c（装载前释放旧指针） | obj+0x1c |
| ③ | AfCore String | — |
| ④ | int32（原生 ≠7 时置内存标志 obj+0x19=1，语义未定） | obj+0x14 |
| ⑤ | LoadObject（IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`，字节 `0x70ab8504`，语义未定名）→ obj+0x20 | obj+0x20 |

三处 LoadObject 助手的调用形态与第六轮 FCN 槽一致：GdbCore 内部包装
`CALL 0x708ed690`（安全释放旧槽对象，ESI=槽地址）+ IAT `0x70a873ac`
`LoadObject(out, riid, pStream, flags)`（AfCore 导入，装饰名
`?LoadObject@@YAJPPAUIUnknown@@ABU_GUID@@PAUIStream@@PAU1@@Z`）。请求 IID
字节逐一读出核对（IWorkspaceName `0x70ab8554`/IDatasetName `0x70ab8574` 与
第六轮一致，新四 IID 见上表）。

## TableName / UnknownDatasetName / ObjectClassName：无流持久化实现（静态证据链）

三条相互独立的证据：

1. **无 GetClassID 写出**：三个 CLSID 的 16 字节内存序全镜像唯一命中各为其
   .rdata 数据本体（`0x70ac318c`/`0x70ac322c`/`0x70ac319c`）；作为 4 字节立
   即数出现在代码中的次数为 0（TableName 数据地址 `0x70ac318c` 的唯一"引用"
   `0x70a80854` 位于类工厂登记表数据区，非代码——该处反汇编为数据；Ghidra
   引用管理器亦无代码引用）。GdbCore 内**没有任何指令能写出这三个 CLSID**，
   `SaveObject`→GetClassID→写类头 的保存路径对它们不存在。
2. **无流框架 Load/Save**：TableName 的 C++ 族虚表块
   `0x70ad06f0..0x70ad0860`（14 个接口/C++ 虚表）与 UnknownDatasetName 块
   `0x70ad08c0..0x70ad0924`（4 个虚表）逐槽反编译，无任何含
   AfStreamRead/AfStreamWrite 调用的函数。TableName 构造函数 `0x709b94b0`
   （逐槽写虚表指针：+0/+4/+0x10/+0x14/+0x18/+0x2c/+0x30..0x4c）与删除析构
   `0x709b9480`（`TEST byte [esp+8],1; JZ; CALL 0x70a6376c` 堆释放）证实该
   块为 C++ 对象族；其 +0x14 "Load" 位为 `0x709dbdf0`（`XOR EAX,EAX; RET 8`
   空实现），+0x18 为 `0x708d1320`（空实现）。UnknownDatasetName 主虚表
   `0x70ad08c0` 槽 +0x14 `0x709ba730` 是构造函数（内部写 `0x70ad08c0`），非
   Load；另一虚表 `0x70ad0908` 槽 +0x0c `0x708d1620` 为 MakeBSTR 取值器
   （get_NameString 型），+0x14 `0x709baa40` 恒失败（`*out=-1` +
   `ChainError(0x80040315, {4BA33331-…}, NULL)`），+0x18 `0x709baa80` 返回
   BSTR "Unknown Dataset"（UTF-16 字面 `0x70ad089c`）——均为非持久化接口。
3. **虚表块无 IPersistStream 形状**：两类的全部 RTTI COL→虚表中，无任何虚
   表同时具备「GetClassID 写出本类 CLSID + AfStream 框架 Load/Save」。

结论（推断，证据如上）：**TableName/UnknownDatasetName/ObjectClassName 在
10.1 GdbCore 内不能经 IPersistStream 持久化**；若 MXD 对象流中出现其 CLSID
新对象，原生装载路径（对象工厂创建后按流框架 Load）无法完成。读取器对这
三类维持「无已闭合体」停止点，与原生失败路径对应。开放点：三类的
ATL/Unknown 双阶段构造（TableName 构造尾部以 `.?AVUnknown@@` TD
`0x70b290ac` 的虚表 `0x70add5b8`/`0x70add5d8` 覆盖 +0/+4，并携 TD
`0x70b37968` 调注册）表明其 COM 形态经聚合/tear-off 暴露，若其他 DLL 提供
持久化适配则上述「不可持久化」结论需按 DLL 范围限定（esri.clsid 声明模块
为 GdbCore）。

## FeatureQueryName / TableQueryName：IPersistVariant 形态，持久化路径未闭合

两类的 RTTI（`.?AVFdbFeatureQueryName@@` TD `0x70b2d7f8` /
`.?AVFdbTableQueryName@@` TD `0x70b2d81c`）→ 主虚表（`0x70aba700` COL
`0x70ae03e8` / `0x70aba628` COL `0x70ae052c`）槽 +0x14 为 GetIDType 型写出：

- FQN `0x708f5760`：写 `{81CAA5D9-90C5-11D3-9F7B-00C04F6BC886}`；
- TQN `0x708f46c0`：写 `{81CAA5D8-90C5-11D3-9F7B-00C04F6BC886}`。

主虚表布局 `{QI, AddRef, Release, get_X(返回 .rdata 宽串指针), 删除析构,
GetIDType, 空实现}` 与 TableName 家族的 IPersistVariant 形态同构（GetIDType
是 `IPersistVariant::GetIDType(CLSID*)` 的特征写出），未见 AfStream 框架的
IPersistStream Load/Save。**IVariantStream 分支的体布局未反编译**（Save
候选 `0x708f4ab0` 体量大、框架未核），本轮不实现读取器，仅记录。同族
ObjectClassName 连 RTTI 类名也未找到（`IObjectClassNameImpl` 接口 TD 存在，
实现类 TD 缺失）。

## 实现（src/objects/dataset_name_more.rs，本轮新模块）

- 六个类标识常量（`GEOMETRIC_NETWORK_NAME_CLASS_ID` 等）与五个请求 IID
  常量（`IID_4BA33331`/`IID_2E060684`/`IID_FDFEBD93`/`IID_F7AD47D1`/
  `IID_4434D680`，请求 IID 语义未定名，仅作字节记录）；
- 六个体读取器 `read_geometric_network_name_body_10_1`/
  `read_relationship_class_name_body_10_1`/
  `read_memory_relationship_class_name_body_10_1`/
  `read_query_table_name_body_10_1`/`read_rel_query_table_name_body_10_1`/
  `read_db_table_name_body_10_1`：严格事务子集（trial 游标 + trial 登记表，
  失败不推进调用者游标、不修改登记表；停止点是成功返回），口径与
  `dataset_name` 读取器一致；
- IDatasetName 槽统一分派 `DatasetNameMoreSlot10_1`（Null/BackReference/
  Body/Unsupported）：新对象命中 FeatureDatasetName（复用第六轮
  `read_feature_dataset_name_body_10_1`）或本模块六类时递归读取（深度受流
  长天然约束；登记先于体解析使循环呈回引），其余类（TableName/
  UnknownDatasetName/FeatureClassName/其他模块实现）停止并回退到槽位起点、
  键不登记；嵌套体停止透传为 `DatasetNameMoreStop10_1::Nested`（键保留，
  与 dataset_name 读取器「透传不回滚」口径一致）；
- IWorkspaceName 槽（RCN ==0 分支、QueryTN ③、DBTN ②）复用
  `WorkspaceNameSlot10_1` 分类与第六轮 WorkspaceName 读取器，体内
  PropertySet 新对象透传为 `WorkspacePropertySet`；
- 请求 IID 语义未知的槽（MemRCN 两槽、QueryTN ④、RelQueryTN 三槽、DBTN
  尾槽）以 `UnknownSlot10_1` 分类：新对象一律停止回退（原生这些槽可承载
  任意类对象，无法安全分派）。
- 与原生的已知偏离（均在读取器文档注明）：负值版本位模式拒绝（较原生有符
  号门更严，QueryTableName 的无符号门对 0xFFFF 原生同样拒绝故无偏离）；槽
  位通知调用（RCN/FGN 虚表 +0x20、QueryTN 虚表 +0x40）不消费流字节，读取
  器不模拟。

## 测试（模块内 #[cfg(test)]，15 项）

FGN 满体+版本门（0 通过/2、0xFFFF 拒绝/事务性）、RCN 选择子两分支+标量保
留、RCN 工作空间槽嵌套 WorkspaceName 体、IDatasetName 槽嵌套 RCN 体、嵌套
停止 Nested 透传（键保留、未知键不登记、游标停在未知对象头）、未知类槽回
退（键不登记、游标回槽位起点）、回引槽、MemRCN 标量+未知槽停止后占位、
QueryTableName 无符号门（0/3/0xFFFF 拒绝）、RelQueryTableName 版本分支
（1 短体不消费后缀/2 读 0x26/3 全读/4 拒绝）、DBTableName 无版本（体首按
String 解析）+工作空间槽嵌套、工作空间槽停止阻断后续字段、max_bytes 限额、
六读取器逐前缀截断事务性、类标识常量与 esri.clsid 声明 GUID 互证（含五
IID）。全部合成输入（样本 0 实例），通过不代表 MXD 格式已验证。

## 未解决事项

1. 六类全部标量/String 字段与五条请求 IID 的业务语义未定性（名称/类别/连
   接信息/源目标名称等均为按 FCN/WorkspaceName 家族类推的候选），原样保
   留；MemRCN 双槽与 RelQueryTN 三槽的对象类语义未定名（IID `0x70ab84e4`
   等 GUID 尾段呈 OleDB `{…8882-0000F877762D}`/`{…080009EC734B}` 族，类型
   库定名未做）。
2. 六类全部无样本字节佐证（参考样本全文件 0 命中）；版本 0 分支（FGN/RCN/
   MemRCN/RelQueryTN 门内）与 QueryTableName 版本 1/2 之外的短体形态无写侧
   来源（Save 恒写 1 或 2/3）。
3. QueryTableName/RelQueryTableName 的 `AfStreamGetSaveVersion` 表
   （(ffff×4,1,2) / (2,3)）的索引/封顶语义未反编译（number-format 轮同类
   开放点）。
4. TableName/UnknownDatasetName/ObjectClassName 的「不可持久化」是 GdbCore
   静态证据上的推断；ATL 聚合形态（见上）下若存在其他模块的持久化适配（例
   如经 IPersistVariant 的 VariantStream 框架），需另行取证。FQN/TQN 的
   IVariantStream 体布局未反编译；ObjectClassName 实现类 RTTI 未找到。
5. TableName 构造函数注册用的 TD `0x70b37968`（≠FdbTableName TD
   `0x70b34bb4`）归属未解析（候选：ATL CComObject 或第二实现类）。
6. 全部结论为静态取证；未运行 ArcMap 10.1，未人工目视验收，不声称已验证。

## 建议整合接线点（写给整合轮；本轮未改任何分派点）

1. **`src/objects/dataset_name.rs`（TopologyName ③ IDatasetName 槽，`read_topology_name_body_10_1` 的 `ObjectEntry10_1::NewObject` match）**：
   现仅 `FEATURE_DATASET_NAME_CLASS_ID` 臂 + `Unsupported` 兜底。扩位表：
   新增六臂，类标识常量 → 本模块读取器（`Body(DatasetNameMoreBody10_1::…)`
   直接作 `DatasetNameSlot10_1` 扩展变体或映射），`stop` 透传映射：
   - `DatasetNameMoreStop10_1::SlotUnsupported` → 既有 `TopologyNameStop10_1::DatasetNameUnsupported`；
   - `WorkspacePropertySet` → 新增变体（或复用 `FeatureDataset` 变体经
     `FeatureDatasetNameStop10_1::WorkspacePropertySetNewObject` 表达）；
   - `Nested` → 需为 `TopologyNameStop10_1` 新增透传变体。
2. **`src/layers/feature_layer.rs`（FCN ⑦ 选择子 !=0 的 IDatasetName 槽，
   `FeatureClassNameStop10_1::DatasetNameNewObject` 兜底处）**：同上扩位；
   本模块的 `DatasetNameMoreSlot10_1`/`DatasetNameMoreBody10_1`/
   `DatasetNameMoreStop10_1` 可整体作为该槽的分类/体/停止类型（七类已含
   FeatureDatasetName）。
3. **FCN 自身 `{198846D0}`**：IDatasetName 槽内嵌 FeatureClassName（递归
   嵌套要素类）未分派——FCN 读取器在 layers 层，objects 层不反向依赖；整
   合轮可在 feature_layer 的槽分派处接入（本模块 `Unsupported` 分支口径不
   变）。
4. **robustness 集中清单**（tests/robustness.rs，整合轮统一补）：六读取器
   入口（`read_geometric_network_name_body_10_1` 等六个名字）。
5. 样本回归：样本 11 个 FCN 选择子全 0 且十类 CLSID 全文 0 命中（本轮扫描
   复核），接线不改变样本行为，`parse samples/测试底图.mxd` 停止点不变。

## 证据与复现

- 二进制：worktree `temp/agent-b-dsnames/GdbCore.dll`（SHA 与第六轮基线一
  致）。
- 本机 pyghidra 产物（脚本与输出 JSON 在 worktree `temp/agent-b-dsnames/`）：
  `scripts/locate_clsids.py`（CLSID 字节命中+RTTI 名枚举）、
  `scripts/resolve_impl.py`（TD→COL→vftable 链与虚表槽）、
  `scripts/decomp_impl.py`（六类 GetClassID/Load/Save 反编译）、
  `scripts/evidence2.py`（GUID/版本常量字节、虚表区原始 dword、疑点汇编）、
  `scripts/evidence3.py`（立即数搜捕+UDN 槽反编译+六字节链）、
  `scripts/evidence4.py`（TableName 构造函数虚表写入证据）、
  `scripts/evidence5.py`/`evidence6.py`（FQN/TQN/OCN 定位与槽）、
  `scripts/check_loadobj.py`（LoadObject 调用形态）、
  `scripts/verify_cols.py`（六虚表 [-1] COL/TD 核对）、
  `scripts/scan_sample.py`（样本全文件 CLSID 扫描）。
- Ghidra 工程副本 `/tmp/gh-b-dsnames/ghproj_gdbcore`（第六轮缓存的拷贝，
  会话临时目录，未入库）。
- 原始 MXD 样本与诊断输出可能包含敏感路径；本记录中的合成字符串已检查，
  公开分享前仍需复核脱敏。
