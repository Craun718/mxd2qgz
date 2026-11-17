# ArcMap 10.1 FCN 选择子与 FDO 旧加载器样本证据研究（2026-11-17）

本文是唯一新增研究输出，对应 `TODO.md` 第 246-248 行。本轮只做只读 REA 研究、
样本字节复核、一次性 `temp/` 分析和本文落档；未修改 Rust 源码、测试、
`README.md` 或 `TODO.md`。未运行 ArcMap 10.1，也未在 QGIS 中人工打开或目视
检查结果。现有读取器与并行未提交改动保持原样。

## 证据分级与边界

| 级别 | 本轮含义 | 结论可支持范围 |
| --- | --- | --- |
| A 样本证据 | 按已闭合对象边界走查 `/Maps` 字节 | 只证明该样本实例及其可达分支 |
| B 反编译证据 | ArcMap 10.1.0.3035 原生 DLL 的 REA 静态反编译 | 闭合读侧字段序与控制流，不证明运行状态 |
| C 读取器与合成测试 | 仓库实现或构造输入下的契约 | 不升格为 MXD 格式或目标软件验证 |
| D 运行取证 | ArcMap 10.1 保存、调试或受控 COM 观察 | 本轮未执行，仍为样本缺项的闭合路径 |

参考样本只是辅助证据；样本缺项不能证明分支不存在。10.1 原生实现是主要证
据，10.8 结论不移入本文。

## 文件身份

| 文件 | SHA-256 | 说明 |
| --- | --- | --- |
| `samples/测试底图.mxd` | `14772f8e3fedca8e1760bef77ab418011df4d8105f5159a05bc866bf603e3738` | 1,089,536 字节；仓库参考样本 |
| `temp/rea-agent-sample-20261117/maps.bin` | `fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe` | 152,941 字节；本轮 `extract /Maps` 导出，与 `temp/r6-maps.bin` 逐字节相同 |
| `temp/agent-dataset-name/GdbCore.dll.FA783766_EB8B_462D_8D00_E07AC33230EA` | `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4` | GdbCore.dll 10.1.0.3035 |
| `temp/agent-fdo-open/cab1/AnnoLayer.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` | `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5` | AnnoLayer.dll 10.1.0.3035 |

`GdbCore.dll` 与 `AnnoLayer.dll` 均为 x86 PE，静态映像地址分别绑定上述哈希。
`README.md` 已登记版本 `10.1.0.3035`；本轮未重新解包安装介质。

## 样本能否提供结构级佐证

本轮用仓库 `extract` 命令重新导出 `/Maps`，确认与既有导出同源；随后以
`temp/rea-agent-sample-20261117/scan_selectors.py` 按 `FeatureClassName`
CLSID 定位并按 FCN 字段序遍历，结果写入
`temp/rea-agent-sample-20261117/scan-result.json`。脚本只输出长度、偏移和
选择子，不输出任何字符串内容。首次脚本误用 Windows 文本 GUID 换端序，已在
结果中保留两目标零命中并更正为 MXD 对象头内存序。

FCN 对象头共 11 处：`0x2f6`、`0x431c`、`0x170b8`、`0x1881c`、`0x19ba4`、
`0x1b0a7`、`0x1c5a6`、`0x1e493`、`0x200ed`、`0x218bc`、`0x23552`。全部体
版本为 2，第 ⑦ 步 IDatasetName 选择子为 0。其中 10 处可穿过空 PropertySet
WorkspaceName 到达版本 >1 的第二选择子，第二选择子也全为 0；
FCN@`0x218bc` 在 WorkspaceName 的 PropertySet 新对象键 2027 处停止，其后的
第二选择子不可达。本轮已按现在已闭合的 PropertySet 走查确认该点是键 2027
的 PropertySet 体（版本 1、属性计数 1、VT_BSTR 值），体尾为 WorkspaceName
工厂标志 1 和 ShapefileWorkspaceFactory CLSID。

MXD 对象头内存序下，`{198846CF}` FeatureDatasetName 与 `{9FCA50EC}`
TopologyName 全流 0 命中。Windows 文本 GUID 换端序扫描也是 0 命中，但该扫
描只说明历史脚本易混淆两种字节序，不作为对象头判定依据。

结论：参考样本不能为 FCN 的 FeatureDatasetName 或 TopologyName 非零分支提
供结构级佐证；它继续支持既有 A 级事实「11 个 FCN 选择子全零」，并把其中的
PropertySet 停止点从局部未知升级为已闭合的样本形态。该升级不产生非零
FCN 选择子字节。

## TODO 第 246 行：FeatureDatasetName 选择子体

**状态：读侧布局为 B 级反编译证据闭合，现有读取器和合成测试已存在；样本 0
实例，非零选择子分支没有 A 级字节佐证，D 级运行取证未做。**

### FCN 选择子规则

本轮在 GdbCore.dll 上用 REA 复核 `FdbFeatureClassName::Load 0x709b5080`。
伪代码绑定 `ev_56a6db5abff391693494e637334f1d75e97b5c5585230298f094ee6ecb08f687`。
第 ⑦ 步是 int16 选择子：`==0` 请求 `DAT_70ab8554`
`{FADD975C-E36F-11D1-AA81-00C04FA33A15}` IWorkspaceName；`!=0` 请求
`DAT_70ab8574` `{89E75917-C287-11D1-AA77-00C04FA33A15}` IDatasetName，并对
`this+0x3c` 对象执行虚表 `+0x20` 通知调用，不消费流字节。版本 >1 时才读取
第 ⑧ 步 int16 选择子；非零请求 `DAT_70ac9a70`
`{505B6FDD-9E6E-42F9-B705-53355B2F1DEF}` ITopologyName。选择子只区分零与
非零；其他位模式没有独立错误分支。

### FeatureDatasetName 对象体

GdbCore 三方定位链为：esri.clsid.ecfg 声明 `{198846CF}` → GetClassID
`0x709b62c0` 写出同一值 → RTTI TD `0x70b34a90`
`.?AVFdbFeatureDatasetName@@`，经 COL `0x70af5124` 连接 IPersistStream 虚
表 `0x70ad04ec`；该虚表 `+0x14` 为 Load `0x709b7c90`，`+0x18` 为 Save
`0x709b7d50`。

本轮 REA 复核 `0x709b7c90`
（`ev_e435b925054f3073295a9aa947c324dea02ae9d554d4c7809713eb4507a0d80c`）
和 `0x709b7d50`
（`ev_c233c172793805287466fa71ffb249b6c275c56f22300231fc0f8c71f9fc5c6b`）。
体字段序为：

1. int16 版本。Load 有符号 `>1` 拒绝；Save 写静态常量 `DAT_70aaac34` short 1。
2. AfCore String ×4，分别到对象 `+0x0c/+0x10/+0x14/+0x18`。
3. 1 字节标志，到 `+0x1c`。
4. LoadObject，请求 IWorkspaceName，结果到 `+0x20`；Save 以文档版本 1
   `SaveObject` 该成员。

四个字符串和标志字节仍是候选语义，不强行定名。10.1 Save 恒写版本 1，版本
0 是 Load 门内兼容分支，不是本轮观察到的写侧产物。

### 现有读取器与测试

`src/objects/dataset_name.rs` 的
`read_feature_dataset_name_body_10_1` 按上述体布局解析；WorkspaceName 槽
复用 `read_workspace_name_body_10_1`，PropertySet 新对象通过其闭合读取器；
未知槽类回退到对象头并停止。`src/layers/feature_layer.rs` 已把 FCN 非零
选择子接到 IDatasetName 统一分派。`tests/dataset_name.rs` 覆盖 v0/v1、
版本越界、嵌套链、空标记/回引/未知类、截断事务性和限额；`tests/robustness.rs`
也有截断扫描。这些是 C 级契约，不能替代样本或运行取证。

### 不确定性

1. 参考样本 11 个 FCN 的 IDatasetName 选择子全零，没有 FDN CLSID 或对象体。
2. FDN 四字符串、标志字节和 WorkspaceName 槽的更多实现类分布未知；目前
   读取器对非闭合类保守停止。
3. v0 只有静态接受和合成覆盖；10.1 Save 恒写 1，v0 历史写侧来源未证。
4. 未运行 ArcMap 10.1，未做 QGIS 人工检查。

## TODO 第 247 行：TopologyName 选择子体

**状态：读侧布局为 B 级反编译证据闭合，现有读取器和七类统一分派已存在；
样本 0 实例，非零第二选择子分支没有 A 级字节佐证，D 级运行取证未做。**

### FCN 第二选择子规则

FCN Load 在版本 >1 时读取第二选择子；非零请求 ITopologyName。样本 10 个
可达 FCN 的该选择子全零，因此 TN 分支不可由本样本到达。

### TopologyName 对象体

GdbCore 定位链为：esri.clsid.ecfg 声明 `{9FCA50EC}` → GetClassID
`0x7095dee0` → RTTI TD `0x70b33680` `.?AVTopologyName@@`，经 COL
`0x70af02fc` 连接 IPersistStream 虚表 `0x70aca460`；`+0x14` Load 是
`0x7095e5c0`，`+0x18` Save 是 `0x7095e550`。

本轮 REA 复核 `0x7095e5c0`
（`ev_65d12247d79fe9afb492ec129c5b7d1db4d68ef3c8660ac15fae7f5457ec3fe0`）
和 `0x7095e550`
（`ev_e58d71262768f6fcc28eabdab9f2d4e66cc418af7ffec06fb62261061fedd204`）。
体字段序为：

1. int16 版本。Load 有符号 `>1` 拒绝；Save 写静态常量 `DAT_70ab7aa0` short 1。
2. AfCore String ×2，分别到对象 `+0x0c/+0x10`。
3. 装载前释放旧 `+0x14`；LoadObject 请求 IDatasetName，结果到 `+0x14`。
   成功后对 `+0x14` 对象执行虚表 `+0x20` 通知调用，不读流。Save 以调用方
   `docVersion` 参数透传 `SaveObject`。

REA 入边分页对 FDN Load 返回 8 条、TN Load 返回 6 条均为函数体内条件跳转；
本会话没有直接给出虚表数据引用，因此本轮把“虚表 +0x14 指向对应 Load”作
为既有定位链复核结论，不以本轮入边页为唯一证明。这不影响伪代码函数体本身
的字段序结论。

### 现有读取器与测试

`src/objects/dataset_name.rs` 的 `read_topology_name_body_10_1` 解析上述
体；IDatasetName 槽经 `src/objects/dataset_name_more.rs` 统一分派。FN
FeatureDatasetName 臂递归，GeometricNetworkName、RelationshipClassName、
MemoryRelationshipClassName、QueryTableName、RelQueryTableName、DBTableName
六类也已有闭合读取器；其余实现类停止并回退。
`src/layers/feature_layer.rs` 已把 FCN 版本 >1 的非零第二选择子接到 TN 读
取器。`tests/dataset_name.rs` 覆盖 TN→FDN→WS、版本门、嵌套停止透传、空标
记/回引/未知类和事务性；`tests/robustness.rs` 提供统一截断扫描。

### 不确定性

1. 参考样本没有 TN CLSID，可达 FCN 的第二选择子全零。
2. TN 两字符串语义、槽后虚表 `+0x20` 通知的接口语义未定名。
3. IDatasetName 槽可能由其他模块实现类填充；读取器保守停止，全集未知。
4. v0 只有静态接受和合成覆盖；10.1 Save 恒写 1，v0 历史写侧来源未证。
5. 未运行 ArcMap 10.1，未做 QGIS 人工检查。

## TODO 第 248 行：FDO 旧加载器体

**状态：版本 0..=3 读侧字段序为 B 级反编译证据闭合，`fdo_legacy` 读取器和
合成测试已存在；参考样本 FDO 为版本 11，不进入该分支，因此没有 A 级字节
佐证，D 级运行取证未做。版本 4 探测已有静态机制证据但保持不实现。**

### 主 Load 分支

本轮在 AnnoLayer.dll 上用 REA 复核 `FDOGraphicsLayer::Load 0x6b1955a0`
（分析父证据 `ev_6e406c01d8ba1f79f1388d77b431e4656a252f7812c3f394e356787eb5228fc4`）。
函数体范围为 `0x6b1955a0..0x6b195a74`。既有汇编记录为：有符号
`CMP AX,0xb; JG` 使版本 >11 拒绝；`CMP AX,3; JG` 使版本 0..=3（含负值位模
式）进入 `FUN_6b197dc0`；`0x6b1955f2..0x6b195607` 以 `(this-0x54, 流, 版本)`
调用旧加载器；版本 4 先进入探测 `FUN_6b198420`，真则旧加载器后读并丢弃一
个 String，假则主分支。本轮没有新增完整逐地址汇编页，主 Load 控制流引用既
有记录；地址范围与 REA 函数体边界一致。

### 旧加载器字段序

本轮 REA 复核 `FUN_6b197dc0`。分析父证据
`ev_723da649c48932dd7d88f46a06ca6f2d0d074e174064f6acb0ebd8a5d81cb995`；函数
体 `0x6b197dc0..0x6b19841a`，544 条汇编、44 个基本块；伪代码父证据
`ev_800231c24e93cd048412643f94cbda9c223d753b984a4c37facfba20075f706f`。
完整伪代码分页复核的关键控制流为：版本 `<2` 时跳过版本 ≥2 块，`2 < 版本`
时执行收尾 String 后返回。流内序为：

| # | 字段 | 目标 / 请求 IID | 版本门 |
| --- | --- | --- | --- |
| 1 | AfCore String | BSTR 后传入 `+0xc4` 接口 | 恒有 |
| 2 | int16 | `+0xc4` 接口 `+0x30` | 恒有 |
| 3 | int16 | `+0xc4` 接口 `+0x38` | 恒有 |
| 4 | double | 局部后传 `+0xc4` 接口 `+0x20` | 恒有 |
| 5 | double | 局部后传 `+0xc4` 接口 `+0x28` | 恒有 |
| 6 | int32 | 内层 `+0xe8` | 恒有 |
| 7 | int16 | 内层 `+0xe0` | 恒有 |
| 8 | LoadObject | ILayer `{34C20002-…}` | 恒有 |
| 9 | LoadObject | ISpatialReference `{4434D680-…}` | 恒有 |
| 10 | LoadObject | IElementCollection2 `{B2BFC403-…}` | 恒有 |
| 11 | `FUN_6b197be0` | LoadObject IName `{4BA33331-…}` 后 `ConvertToFullPath` | 恒有 |
| 12 | int16 | `+0xc8` 接口 `+0x70` | ≥2 |
| 13 | String | `+0x60` 接口 `+0x20` | ≥2 |
| 14 | String | `+0x60` 接口 `+0x20` | ≥2 |
| 15 | int32 | `+0x60` 接口 `+0x18` | ≥2 |
| 16 | LoadObject | IFeatureRenderer `{F3435802-…}` | ≥2 |
| 17 | int16 | `+0x60` 接口 `+0x28` | ≥2 |
| 18 | int16 | 与 17 组合写 `+0x60` 接口 `+0x28` | ≥2 |
| 19 | LoadObject | IColor `{20CD40B0-…}` | ≥2 |
| 20 | AfCore String | `+0xc8` 接口 `+0x20` | >2 |

`FUN_6b197be0` 本轮也复核：伪代码绑定
`ev_660710bfcd009c60ba6982a58cfb33e70f657fbfcaf2c76147eb93679f7520ac`，函
数体 `0x6b197be0..0x6b197dbe`。它先 LoadObject IName
（`DAT_6b1abcf0`），创建空 `String`，调用 LayerLib 导出
`ConvertToFullPath`，随后析构路径字符串并对 IName 做虚表 `+0x14` 调用；除
IName 对象头本身外不读流。

旧体在第 20 步收尾 String 后返回，没有主分支第 7 起的尾段。各接口指针槽的
业务语义仍只按请求 IID 表述，未定名成员。

### 版本 4 探测

既有记录闭合了 `FUN_6b198420` 的静态机制：Seek 到流绝对位置 0，读 16 字节
GUID；分支 A 与 FDO 本类 `{34B2EF83-…}` 比对；分支 B 与 Map
`{E6BDAA76-4D35-11D0-98BE-00805F7CED21}` 比对，再按有无 int32 根前缀读取
对象 CLSID，比较 short 是否为 9。探测真则旧加载器加丢弃 String。其依赖包
括整流绝对位置访问、IObjectStream QI 的运行时游标效应和分支 B 的复位语义；
严格子读取器无法安全复现，现有代码对版本 4 显式拒绝。

### 现有读取器与测试

`src/layers/fdo_legacy.rs` 的
`read_fdo_graphics_layer_legacy_prefix_10_1` 实现版本 0..=3 已确认字段；
它只分派已闭合的 FeatureLayer、PCS/GCS/Unknown 空间参考和 FeatureClassName
槽，其余新对象在对象头回退停止。事务性、截断、限额和回滚由
`tests/fdo_legacy.rs` 的 20 项和 `tests/robustness.rs` 集中扫描覆盖。
`fdo_graphics_layer` 主读取器仍对版本 <5 停止，没有自动调用旧加载器；这是
保守接线，不是旧体字段序缺失。

### 不确定性

1. 参考样本 FDOGraphicsLayer 版本 11，旧加载器 20 个字段没有任何样本字节。
2. `+0xc4/+0xc8/+0x60` 接口成员、String 和标量业务语义未定名。
3. 版本 4 探测依赖运行时流对象状态与 QI 结果，本轮没有受控 IStream 或
   ArcMap 观察证据。
4. 旧加载器相关改动早于本轮已存在；引用的 20/6 项测试是既有产物记录。全
   量测试也与本轮无改动耦合，仍不能替代真实版本 0..=3 MXD 或运行取证。
5. 未运行 ArcMap 10.1，未做 QGIS 人工检查。

## 版本分支与写侧对照

| 分支 | 读侧门 | 写侧证据 | 样本覆盖 |
| --- | --- | --- | --- |
| FCN | 有符号 >2 拒绝 | 样本 11 个全部版本 2 | 首选择子 0；10/11 可达第二选择子为 0 |
| FDN | 有符号 >1 拒绝；v0/v1 同字段 | Save 写静态 short 1 | 0 实例 |
| TN | 有符号 >1 拒绝；v0/v1 同字段 | Save 写静态 short 1 | 0 实例 |
| FDO | >11 拒绝；≤3 旧加载器 | 已有主分支 Save 对照；旧加载器 Save 未逐指令展开 | FDO 11，仅主分支 |

因此 v0/v1 FDN/TN 的同字段序和 FDO ≤3 字段序只能是历史兼容读侧，不是本轮
参考样本或 10.1 Save 的直接产物。

## 样本、运行取证与构造输入方案

### 真实样本优先

1. FDN：取得含要素数据集内要素类或拓扑图层的 ArcMap 10.1 MXD，使 FCN 第 ⑦
   步选择子非零并出现 `{198846CF}` 对象。保留原始 MXD、ArcMap 版本、数据
   源类型和生成操作。
2. TN：取得含拓扑图层的 File/Enterprise Geodatabase MXD，使 FCN 版本 >1 的
   第二选择子非零并出现 `{9FCA50EC}` 对象。
3. FDO：取得保存时仍写 FDOGraphicsLayer 版本 0..=3 的旧 MXD，或先确认目标
   版本能否产生旧分支；版本 4 需 pre-8.3 档案或受控旧版运行。
4. 新样本必须做对象级边界走查，不能只用 CLSID 全文计数替代对象边界。

### 受控运行取证

1. 优先受控 COM：ArcObjects 或 ArcMap 进程内构造
   `FeatureDatasetName`/`TopologyName`，挂接 WorkspaceName/IDatasetName 后
   用包装 `IStream` 调 `IPersistStream::Save`。记录对象头、版本 short、每
   个 String 长度、标志字节、LoadObject 前后游标、HRESULT 和最终流长度。
2. FDO 可用同样包装流直接驱动 FDOGraphicsLayer 保存，或打开含注记的旧档后
   断点 `0x6b1955a0` 和 `0x6b197dc0`；记录架构版本、分支地址、每次读流长度
   和成员写入偏移。
3. 版本 4 探测需要包装 `IStream::Seek`/`Read` 和 QI 结果，分别覆盖流首为
   FDO、Map、带计数根前缀三种形态，并记录每次 Seek 前后的绝对位置。
4. 若完整 ArcMap 保存不可控，可先构造 MXD 使 UI 建立 FDN/TN/FDO 状态，再在
   SaveObject 处断点截获序列化字节。

### 构造输入的边界

已闭合的反编译布局可以用合成流验证读取器事务性，但不能升格为 MXD 规则或
ArcMap 行为验证。未闭合的 IDatasetName/IWorkspaceName/IName/IColor 等实现
类不应按请求 IID 猜体；继续对象头停止，直到样本或运行证据闭合其布局。

## REA 执行状态与限制

本会话累计产生 70 条 Evidence。AnnoLayer 成功分析记录 `0x6b1955a0`、
`0x6b197dc0`、`0x6b197be0`；GdbCore 会话成功复核 FCN/FDN/TN Load、FDN/TN
Save 和入边页；相关地址均有正确目标哈希。本会话因文件名歧义未完成显式关
闭；后续 `binary_session` 仍报告目标绑定，但系统进程列表无 Ghidra/Cargo 下
游进程。
三次针对 GdbCore `0x709b5080`、`0x709b7c90`、`0x7095e5c0` 的
`procedure_references` 失败原因是当时活动目
标是 AnnoLayer，输出明确报告地址不在该目标且要求切换二进制，不作证据。
GdbCore 首次 `0x709b5080` 全函数分析因 `/tmp` tmpfs 100% 失败；随后重新打
开并成功取得同地址函数体和入边页。首次 Ghidra 过程输出不是函数证据，只记
录失败原因。

本轮本想复跑全量 `cargo fmt --check` 与 `cargo test`。`df` 显示 `/tmp` 16G
tmpfs 只剩 88M；`cargo test` 曾于 16:41 全绿，而本轮 08:40 的失败日志指出
`Setting up tmp... failed: no space left on device`。当前工作区有并行源码/
测试改动，本轮也没有修改它们，因此不引用该旧全量日志作为本轮验证。遵循
「Rust 代码改动后运行」的项目规则，本轮不改 Rust，故未清理 `/tmp` 或删除
他人资料来强行复跑；未重跑不构成本轮实现验证。

## 建议

维持 `TODO.md` 第 246-248 行未勾选，建议在措辞中区分两层：

- 第 246 行：FCN FeatureDatasetName 选择子分派与 FDN 体已由 10.1 反编译闭
  合并接线；参考样本 11 个 FCN 选择子全零、FDN 0 实例，需非零分支真实字节
  或受控运行取证。
- 第 247 行：FCN TopologyName 选择子分派与 TN 体已由 10.1 反编译闭合并接
  线；可达样本第二选择子全零、TN 0 实例，需非零分支真实字节或受控运行取
  证。
- 第 248 行：FDO 旧加载器 0..=3 字段序与 `fdo_legacy` 已闭合；参考样本
  FDO 11 不进入分支，需旧版字节或运行取证。版本 4 仍保持不实现。

既有文档若仍把 246-248 行写成「体未闭合」，应以后续保留源码状态为准修正
为「体静态闭合但样本/运行证据缺项」。

## 本轮与既有文件

本轮创建或辅助生成的路径：

- `docs/mxd-parse-agent-sample-evidence-data-20261117.md`
- `temp/rea-agent-sample-20261117/maps.bin`
- `temp/rea-agent-sample-20261117/scan_selectors.py`
- `temp/rea-agent-sample-20261117/scan-result.json`

未修改源码、测试、`README.md` 或 `TODO.md`。`scan-result.json` 不包含样本
字符串内容；本文也没有引用任何 MXD 内名称或路径。分享原始 MXD 或既有诊断
输出前仍需人工脱敏复核。
