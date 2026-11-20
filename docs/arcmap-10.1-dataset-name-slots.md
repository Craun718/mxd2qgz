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
| ③ | LoadObject（请求 IID `{89E75917-…}`=IDatasetName，`0x70ab8574`）→ obj+0x1c；装载成功后对槽对象做虚表 +0x20 获取 WorkspaceName | obj+0x1c（Save `SaveObject([obj+0x1c], 1)`） |

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
| ⑦ | int16 选择子（读长==2 已检；原生只区分零/非零）：==0 → LoadObject（请求 IID `{FADD975C-…}`=IWorkspaceName `0x70ab8554`）→ obj+0x30；!=0 → LoadObject（IID IDatasetName `0x70ab8574`）→ obj+0x2c + 虚表 +0x20 获取 WorkspaceName。Save 侧选择子写 `(obj+0x2c != 0)` | obj+0x30 / obj+0x2c |

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
| ⑦ | LoadObject（请求 IID `{4BA33331-D55F-11D1-8882-0000F877762D}`，字节 `0x70ab84e4`，IName） | obj+0x1c（Save `SaveObject([obj+0x1c],1)`） |
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
| ④ | LoadObject（IID `{2E060684-E14C-44D3-858A-442AB6A0A863}`，字节 `0x70ac5158`，IQueryDescription）→ obj+0x18；装载后若非空则调用虚表 +0x40 的 IsSpatialQuery，结果置内存 bool obj+0x14 | obj+0x18（Save 后写本槽） |

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
| ③ | AfCore String，几何列名称 | obj+0x10 |
| ④ | int32，esriGeometryType。原生值异于 esriGeometryAny（7）时置内存标志 obj+0x19=1 | obj+0x14 |
| ⑤ | LoadObject（IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`，字节 `0x70ab8504`，ISpatialReference）→ obj+0x20 | obj+0x20 |

### DBTableName 尾部空间参考

用基线所列 GdbCore.dll 10.1.0.3035 原件重新核对 SHA-256，经
`objdump -d -Mintel` 确认 `0x708d7adc` 压入 ISpatialReference IID 常量
`0x70ab8504`，`0x708d7ae1` 取持久化接口相对偏移 `+0x20`，随后
`0x708d7aed` 将字段和当前流交给 LoadObject。Save 在 `0x708d7b8a`
取同一成员，并于 `0x708d7b91` 调用同一当前流上的 SaveObject。
这一路径使用普通对象头，和工作空间槽共享宿主登记表。

解析器复用已有 PCS、GCS 和 UnknownCoordinateSystem 分派。未知新类保留
对象头作为停止位置，已知类体截断时回滚整个 DBTableName 的游标和登记表。
合成 GCS v2 用例验证体边界及后续键回引。未取得非空尾槽实档，未在 ArcMap
和 QGIS 人工检查。

### 数据集名称字段与接口访问器

用同版 GdbCore.dll 复核下列字段。esriGeoDatabase.olb 来自 10.1 介质，
SHA-256 为 `09b741f2627633d36b67985e175632055fe6c5b8cd1000484acc3007f5917f9d`。
从其英语资源提取的 MSFT 类型库 SHA-256 为
`92634979daa3ed0008b36d152e1fdf39cba04944f6bc7bbb5ea4c786bb44bb9a`。
按 x86 类型库中记录的槽偏移读取接口属性名，并用 QI 的 IID 分支和
RTTI Complete Object Locator 偏移确定实际接口指针。下表偏移统一相对于
完整对象。首组字符串依次对位为 IDatasetName.Name、IName.NameString
及 IDatasetName.Category，其中 RelQueryTableName 首组仅有前两串。
字符串按原始值保留。

| 类 | IPersistStream / IDatasetName / IName 相对偏移 | 三串完整对象偏移 | 三个 getter（按流序） |
| --- | --- | --- | --- |
| GeometricNetworkName | `+0x1c / +0x14 / +0x10` | `+0x2c / +0x30 / +0x34` | `0x70a272f0 / 0x70a27580 / 0x70a27580` |
| RelationshipClassName | `+0x20 / +0x14 / +0x10` | `+0x34 / +0x38 / +0x3c` | `0x70a27580 / 0x708d7600 / 0x708d7600` |
| MemoryRelationshipClassName | `+0x20 / +0x14 / +0x10` | `+0x24 / +0x28 / +0x2c` | `0x7091b8c0 / 0x70a272f0 / 0x70a272f0` |
| QueryTableName | `+0x28 / +0x14 / +0x10` | `+0x30 / +0x34 / +0x2c` | `0x708d71f0 / 0x708d8750 / 0x70a272f0` |
| RelQueryTableName | `+0x20 / +0x14 / +0x10` | `+0x28 / +0x2c / —` | `0x709df5d0 / 0x708d71f0 / —` |
| DBTableName | `+0x2c / +0x14 / +0x10` | `+0x34 / +0x38 / +0x30` | `0x70a27580 / 0x708d7600 / 0x708d71f0` |

RelQueryTableName 首段只有两串，第三串在选择集槽之后，为下表的 TargetColumns。
GeometricNetworkName 的 IGeometricNetworkName 虚表 `0x70ad0630`
（完整对象 `+0x18`）的 getter `0x709b8b90` 读取完整对象 `+0x38`，
即持久化的 FeatureDatasetName 槽。RelationshipClassName 的
IRelationshipClassName 表 `0x70adb9dc`（`+0x18`）的 getter
`0x709fec90` 读取完整对象 `+0x4c`，对应选择子非零的 FeatureDatasetName。
零分支使用 IDatasetName.WorkspaceName，getter `0x70a49310` 读取完整对象
`+0x50`。上述两类读取 FeatureDatasetName 后对该对象调用虚表 `+0x20`
获取 WorkspaceName，分别存入本对象的工作空间成员。

| 类 / 接口虚表 | 流中剩余字段的顺序、完整对象偏移与 getter |
| --- | --- |
| RelationshipClassName / IRelationshipClassName `0x70adb9dc` | Cardinality `+0x40`（`0x708d1650`）、Notification `+0x44`（`0x70a44a70`）、IsComposite `+0x48`（`0x708de140`）、IsAttributed `+0x4a`（`0x70a3b200`）。 |
| MemoryRelationshipClassName / IRelationshipClassName `0x70adbe14`、IMemoryRelationshipClassName `0x70adbe4c` | Cardinality `+0x30`（`0x708fe3c0`）、Notification `+0x34`（`0x7093e310`）、IsComposite `+0x38`（`0x70a400d0`）、IsAttributed `+0x3a`（`0x70a40120`）；OriginName `+0x3c`（`0x709b8b90`）、DestinationName `+0x40`（`0x70a40150`）；ForwardPathLabel `+0x44`（`0x708d7600`）、BackwardPathLabel `+0x48`（`0x70a401e0`）、OriginPrimaryKey `+0x4c`（`0x709a9be0`）、OriginForeignKey `+0x50`（`0x70a40250`）。 |
| QueryTableName / IDatasetName `0x70ac5080`、IQueryTableName `0x70ac50b0` | WorkspaceName `+0x44`（`0x70a49440`）、QueryDescription `+0x40`（`0x70932360`）。Load 在 `0x70932ab6` 调用 IQueryDescription.`IsSpatialQuery`（+0x40），将布尔结果缓存于完整对象 `+0x3c`。IQueryTableName.Query getter `0x70932570` 优先调用该对象的 Query getter（+0x38），仅在无 QueryDescription 时使用完整对象 `+0x38` 的内存字符串。 |
| RelQueryTableName / IRelQueryTableName2 `0x70add0bc` | RelationshipClassName `+0x30`（`0x70a5a990`）、ForwardDirection `+0x34`（`0x70a5aa50`）、SrcQueryFilter `+0x38`（`0x70a5aa40`）、SrcSelectionSet `+0x3c`（`0x70a5a9b0`）、TargetColumns `+0x40`（`0x70a5a980`）、LeftOuterJoin `+0x44`（`0x70a5a9a0`）、v2 起 DoNotPushJoinToDB `+0x46`（`0x70a5a9e0`）、v3 起 FireRowChangedEvents `+0x48`（`0x70a5a4a0`）。前七个 getter 经跳板将 this 从完整对象 `+0x1c` 调整到 `+0x18`，FireRowChangedEvents 直接使用 `+0x1c`。 |
| DBTableName / IDatasetName `0x70ab83b4` | WorkspaceName `+0x48`（`0x709fec90`）。第四串 `+0x3c` 和 int32 `+0x40` 由 QI `0x708d7cc0` 映射到 `+0x18` 的接口 `{7561FC25-7E2F-4D96-8A4C-6F3973A3C21E}` 写入，见 setter `0x708d7020` 和 `0x708d7000`。字段含义见下一节。公开 IDatasetName.Type getter `0x708d71c0` 根据布尔缓存返回数据集类型 5 或 10。 |

这里确认的是字段对位。运行时打开数据源、未支持的嵌套类体，以及字段取值
在 ArcMap 中的实际效果仍需实档或运行取证。

### DBTableName 几何列名称与类型

同为 10.1.0.3035 的 SqlFDB.dll（SHA-256
`fb440a01e9435947892f0ad9cbb76aaf1cd7ca40c09e5cea0cdba11c23294b8b`）
在 `0x7fed3a29` 查询 DBTableName 的辅助接口。SqlClass 的函数
`0x7fedd890` 位于虚表 `0x7ff2eff4` 的 `+0x10`，其 RTTI COL
的接口偏移为 `+0x18`。这把赋值链中的接口相对偏移换算到完整对象：

- **几何列名称**。SqlClass 在 `0x7fee1bf8` 逐个取得 IField，
  `0x7fee1c94` 通过 `IField+0x14` 读取 Type。类型库 TypeInfo 44
  定义 7 为 esriFieldTypeGeometry。该分支在 `0x7fee1ccd` 通过
  `IField+0x0c` 读取 Name，并于 `0x7fee1d0f` 赋到 SqlClass
  完整对象 `+0xc4`（接口 `+0xac`）。`0x7fedd953` 将这个字符串
  传给辅助接口 `+0x10` 的 setter `0x708d7020`。GdbCore Save
  `0x708d7b68` 将对应成员写成第四个 String。
- **几何类型**。SqlClass 完整对象 `+0xd4`（接口 `+0xbc`）在
  `0x7fedd962` 传给辅助接口 `+0x14` 的 setter `0x708d7000`，
  GdbCore Save `0x708d7b7d` 将对应成员写成 4 字节整数。SqlFDB
  的 `0x7fee1ee2` 查询 `{439A0D51-3915-11D1-9CA7-0000F8780619}`，
  即 IGeometryDefEdit。随后 `0x7fee1efe` 取同一 SqlClass 成员传给
  该接口 `+0x2c` 的 GeometryType setter。对应类型库 TypeInfo 465
  定义了此槽。esriGeometry 10.1 类型库（SHA-256
  `78aaabd92926d1a42011a3e6f9100116666a4adfec2dde9165c1b0af37fd4f64`）
  TypeInfo 1 的枚举表定义 esriGeometryAny 为 7，和 GdbCore Load
  在 `0x708d7afa` 的比较一致。

上述结论绑定到读写字段的业务含义。辅助接口的正式成员名、其他数据源的赋值
分支和目标软件中的实际取值仍待取证。读取器继续原样保留字符串和整数。

三处 LoadObject 助手的调用形态与第六轮 FCN 槽一致：GdbCore 内部包装
`CALL 0x708ed690`（安全释放旧槽对象，ESI=槽地址）+ IAT `0x70a873ac`
`LoadObject(out, riid, pStream, flags)`（AfCore 导入，装饰名
`?LoadObject@@YAJPPAUIUnknown@@ABU_GUID@@PAUIStream@@PAU1@@Z`）。请求 IID
字节逐一读出核对（IWorkspaceName `0x70ab8554`/IDatasetName `0x70ab8574` 与
第六轮一致，新四 IID 见上表）。

### RelQueryTableName 的非空关系类名称

使用同一 GdbCore.dll 10.1.0.3035 原件的 `objdump -d -Mintel` 输出
复核。Load `0x70a5a530` 在 `0x70a5a5a8` 压入 IName IID 常量
`0x70ab84e4`，于 `0x70a5a5b3` 将槽（持久化接口 `+0x10`）和
宿主流传给 LoadObject。Save 在 `0x70a5a769` 取同一成员，
`0x70a5a776` 调用 SaveObject。RelationshipClassName 的 QI
`0x70a3ae00` 和 MemoryRelationshipClassName 的 QI `0x70a3fcc0`
都比较该 IID 并返回完整对象 `+0x10`。两类的 GetClassID 与
IPersistStream 读写入口见上表。

解析器复用两类已验证的对象体，并与宿主共享键表。未知新类停在对象头前；
已知类内部的停止原因逐层返回。数据集名称与关系名称交替嵌套时共用 32 层
新对象限额，回引不增加层数。构造测试覆盖两类体之后的连接字段、同键回引、
嵌套停止、逐字节截断及交替嵌套的限额回滚。

两套介质提取的 134 个 `/Maps` 流及仓库参考 MXD 文件未检出
RelQueryTableName 的 CLSID 内存序字节。此检索未取得实档对象体；
ISelectionSet 和其他可能的 IName 类仍需独立取证。
本轮未在 ArcMap 或 QGIS 人工检查。

### RelQueryTableName 的查询过滤器

使用本页所列 GdbCore.dll 10.1.0.3035 原件和同版 esriGeoDatabase 类型库，
以 `objdump -d -Mintel`、PE RTTI、原始 GUID 和导入地址交叉核对。
RelQueryTableName Load `0x70a5a5f1` 对第二对象槽请求 IQueryFilter
（`0x70ac4f28`）。以下类的 IPersistStream Load 和 Save 由虚表定位，
并以 GetClassID 返回常量确认类身份。QueryFilter、SpatialFilter、FilterDefs、
XMLFilterDef 的 QI 主入口分别为 `0x709fe890`、`0x70a00520`、
`0x708fd430`、`0x70a29af0`，沿基类链提供 IPersistStream，没有命中
标准 ObjectStream 优先查询的 IPersistVariant IID。

| 类 | CLSID | IPersistStream 虚表 | GetClassID / Load / Save |
| --- | --- | --- | --- |
| QueryFilter | `FDFEBD95-ED75-11D0-9A95-080009EC734B` | `0x70ad8018` | `0x709fecc0 / 0x709ff8b0 / 0x709ffaf0` |
| SpatialFilter | `FDFEBD96-ED75-11D0-9A95-080009EC734B` | `0x70ad8134` | `0x70a00630 / 0x70a007d0 / 0x70a00900` |
| FilterDefs | `3131151A-4B5C-4526-98BE-711746B03DF8` | `0x70abe040` | `0x708fd3f0 / 0x709011c0 / 0x708ff110` |
| XMLFilterDef | `47F975DB-77C0-4A8C-B8A0-8E9FE77A9795` | `0x70adabc4` | `0x70a29920 / 0x70a29fa0 / 0x70a2a020` |

- **QueryFilter** 先读 short 版本，再读三个 AfCore String（SubFields、
  WhereClause、第三串原样保留）、ISpatialReference、double SpatialResolution、
  IFIDSet。Save 表 `0x70a89008` 为 `[1,1,3,3,4]`。Load v1 在 IFIDSet
  后结束。其他版本还读自由函数 ReadString 编码的 PostfixClause。
  v2 在其后读取内联 int32 数量及 IFilterDef 对象循环（`0x709ffa41`），
  v0/v3/v4 读取一个 IFilterDefs 对象（`0x709ffaa7`）。v4 再读
  自由函数 ReadString 编码的 PrefixClause。
- **SpatialFilter** 先读自身的 short v0/v1，`0x70a00871` 直接调用
  QueryFilter 的 Load，含原有版本字段。随后读 SearchOrder、SpatialRel
  两个 int32，SpatialRelDescription String、IGeometry、GeometryField String，
  最后读取 FilterOwnsGeometry 的 short 原值（getter `0x709abef0`）。
  空间属性按 ISpatialFilter 虚表 `0x70ad81b4`
  和接口偏移 `+0x58` 对位，两个字符串 getter 分别为 `0x709ba9a0`
  和 `0x709df5d0`。现有几何体读取器支持 Point、Multipoint、
  Polyline、Polygon 和 Envelope。
- **FilterDefs** 读取 short v0/v1 和 int32 数量。循环中 `0x7090125c`
  经主虚表 `0x70abdfec` 的 `+0x2c` 到 `0x708fd250`，调用
  LoadObject，再对结果查询 IFilterDef。**XMLFilterDef** 读取 short
  v0/v1，再调用自由函数 ReadString 两次。此路径保持宿主键表，复用
  已有 ISpatialReference 和 FIDSet 读取器。

构造测试覆盖两类过滤器的版本体边界、空间几何、非空 FIDSet 和 FilterDefs、
XMLFilterDef 回引、逐字节截断回滚及未知定义类停止。新增解析器拒绝负数版本、
负数定义条目计数及长度不足。RelQueryTableName 真实宿主、其余过滤器实现和
ArcMap/QGIS 人工检查仍待取证。

### MemoryRelationshipClassName 的端点名称

同一 GdbCore.dll 10.1.0.3035 的 Load 在 `0x70a403d3` 和
`0x70a403f1` 依次把持久化接口的 `+0x1c`、`+0x20` 成员交给
LoadObject，请求相同的 IName IID `0x70ab84e4`。Save 对应调用位于
`0x70a40514` 与 `0x70a40528`。IMemoryRelationshipClassName 的
OriginName、DestinationName 对位见前表。

两个槽复用已核对的 FeatureClassName、FgdbFeatureClassName、TableName、
QueryTableName、FeatureQueryName、TableQueryName 和 DBTableName
对象体。新对象与后续回引使用同一宿主键表。构造用例经过外层
IDatasetName 入口读取 FileGDB 要素类名称、普通表名称及后四串，覆盖
逐字节截断回滚。未知类或内部未读完的成员继续报告停止点。尚无两槽非空的
真实旧档和目标软件运行证据。

### RelQueryTableName 的选择集槽

由 GdbCore 10.1 Load `0x70a5a613` 的 `0x70ac4d88` IID 数据和类型库
TypeInfo 59 确认，第三槽请求 ISelectionSet。Save 调用 `0x70a5a7b6`
将对象交给宿主流的 SaveObject。复核 GdbCore 的构造类、GetClassID 返回
值、QI 主入口和共享后备 `0x70a60800`，得到下列四类：

| 实现 | CLSID | ISelectionSet 的 QI 主入口 |
| --- | --- | --- |
| Selection | `10EC66F1-D954-11D1-89EA-006097AFF44E` | `0x70a1a600` |
| FileSelection | `EB75FC9A-485C-11D4-9F52-00C04F79927C` | `0x709e9130` |
| FIDSetSelection | `DA8B5B26-8BBD-41E8-A4AC-54C980F56119` | `0x709db270` |
| RelQueryTableSelectionSet | `6FC04EAF-F7B8-432F-B87C-C9D9B17E79BF` | `0x70a5ac80` |

四类均能返回 ISelectionSet。其 QI 分支和共享后备均没有命中
IPersistStream、IPersistVariant，或 SystemCore 写出器先查询的
`{C9E7A782-3DDC-411F-8A66-908C579F123D}`、
`{BB75DA81-8DFE-4122-A9C2-86F07C539AD5}`。RelQueryTableSelectionSet
在完整对象 `+0x1c` 的虚表 `0x70add21c` 属于请求
`{F87875E1-D8B4-11D1-89EA-006097AFF44E}` 的接口。
`0x70a5b460` 和 `0x70a5b4c0` 对所包对象查询同一 IID 后转调用
`+0x18/+0x1c`，两处都没有接收 IStream 参数。

SystemCore.dll 10.1.0.3035（SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）
的 ObjectStream 写出器 `0x7ffcf7e0` 在持久化接口均为空时，于
`0x7ffcfbd2` 跳到 `0x7ffcff1d`，以空对象入参递归调用同一写槽，
生成 16 个零字节。结论限定于这四个具体类在默认宿主写出路径的直接输入。
原生代码在持久化查询前还可经 `0x7ffd0760` 调用已配置的对象替换回调，
其他模块的 ISelectionSet 实现及替换输入仍待实档取证。读取器继续区分
空槽、回引和未知新类。这是静态取证，未在 ArcMap 或 QGIS 人工检查。

## TableName / UnknownDatasetName / ObjectClassName：早期持久化排查

2026-11-20 复核更新。TableName 与 ObjectClassName 已定位到
同一 FdbTableName 持久化表及 GdbCoreLib 读写实现。以下结论是早期排查
记录，新证据见[对象名分派续查](mxd-terra-round9-table.md#objectclassname-与栅格表名称)。

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

1. MemRCN 双槽的其他 IName 类、RelQueryTN 的其他 IQueryFilter 实现、
   选择集替换路径和其他 IName 类、
   以及 QueryTableName 的 QueryDescription
   槽仍需对其具体对象类补齐读取分派。
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
