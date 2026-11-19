# 第九轮：独立表 FieldInfo 与 SDK 样本（2026-11-17）

## 身份与证据

本轮重新计算 SDK Java `LinearRef.mxd` SHA-256 为 `5eb1835d15610b6b56c853153360476400f6d55cf0693dfb72305ded3ce2715d`；文件 `/Version` 9.2/14、来源及提取见 [第八轮旧档记录](mxd-terra-round8-old.md)。保存版本不能等同 DLL 版本。

重新核对 Map.dll：PE32 x86、ImageBase `0x793a0000`、固定文件版本 10.1.0.3035、SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；Layer.dll：PE32 x86、ImageBase `0x78650000`、固定文件版本 10.1.0.3035、SHA-256 `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`。哈希和版本资源用于核对身份，不证明签名或运行行为。

此前独立 Ghidra 桥接工具 CLI / Ghidra 的 Map `0x7945d630` Load 伪代码在 String + 对象的计数循环中请求 `DAT_7948dcd0`，证据及限制见 [宿主记录](mxd-terra-host-table.md)。本轮对上述 Map.dll 直接按 VA 检查 16 字节，结果 `2c ae ba a2 9b 96 d2 11 ae 77 08 00 09 ec 73 2a`，即 IID `{A2BAAE2C-969B-11D2-AE77-080009EC732A}`，与既有 IFieldInfo 识别一致。Layer.dll FieldInfo CLSID `{A2BAAE2D-969B-11D2-AE77-080009EC732A}`、GetClassID、Load/Save、虚表、v0–4 读侧顺序及版本门的原生证据见 [FieldInfo 研究](arcmap-10.1-featurelayer-tail.md#fieldinfo-元素体layerdll-0x7865b520证据确定的)。本轮复用这些已保存 Ghidra 桥接工具 静态证据，并新增二进制 IID 字节交叉核查；未切换共享 Ghidra 桥接工具 目标，也未新增目标软件运行观察。IID 与 CLSID 不同；本轮没有通过运行时 QI 验证对象。

## 接线与样本

独立表命名循环仅对新对象的 FieldInfo CLSID 分派现有 FieldInfo 读取器，保存对象体；空槽和回引保留原有分类，并以对齐的可选体区分。非 FieldInfo 新对象仍在头前停止且不登记；FieldInfo 内部遇未知数字格式时保留外层键、已读体和内部停止原因，游标停在未知对象头前。短读、版本错误及限额导致整次调用回滚。这里的安全行为不代表模拟原生错误处理或验证回引的 QI。

按已有 `/Maps` 宿主读取链，首表头 `0x6a50`、键 331、版本 9。其首个命名对象头 `0x6bed`、键 334；对该已到达边界核对原始字节，类为 FieldInfo，体首 `02 00` 为版本 2。首表命名循环共读取 24 个 FieldInfo 新对象体，均为版本 2、均无内部停止，整个首表在现有读取器的支持范围内报告 Complete。对象键依次为 334、335、337、339、341、342、343、345、347、348、350、351、353、355、357、359、360、361、362、363、364、365、366、368。此前第八轮已有另一个宿主的 5 个 v2 实例，故这里是补充证据，不是首次发现 v2。

读取器继续到第二表头 `0x74ef`，其体版本 9，也报告 Complete；继续到 Map 已有停止点 `0x8407`（`AfterBorderTail { stage: 38 }`）。这些结论依赖现有读取器和共享引用表，上游部分未知体按既有有界策略跳过；并非独立原生仪器对每一个边界的观测，也不证明所有表、其业务数据或 Map 全体均已闭合。样本中数据源等字符串未在此记录，避免泄露路径。

## 验证与可勾范围

`tests/terra_round9_table.rs` 构造测试覆盖 FieldInfo v2 完整分派、逐字节截断时游标/引用表回滚、内部未知数字格式停止与键保留；独立忽略测试从 SDK 文件的 Map 宿主进入，断言两表及首个 FieldInfo 的类、键、版本原始字节。旧的 SDK 测试同步更新到新的停止点。执行 `cargo fmt --check`、`cargo test --offline` 均通过；第九轮 SDK 忽略测试及两个受影响的第八轮 SDK 忽略测试显式运行均通过。构造测试和读取器自验都不能取代目标软件验证。

- 可单独落账：Map 独立表命名槽的 10.1 请求 IID 字节与既有 IFieldInfo 证据相符；FieldInfo 在该槽的保守分派及上述 SDK 两表读取器结果；首表 24 个 v2 是已知 v2 的额外宿主证据。
- TODO 第 135 行是多缺口合项：SDK 首表停止于命名对象的旧描述可更新为本轮结果，但其他类和字段仍开放，整项不能勾选；第 278 行 v0/v1 仍缺实档，不能勾选。本轮依用户要求未修改 TODO。
- 非空独立表的任意对象实现、真实数据源打开、回引接口兼容、其他版本及完整转换未闭合。ArcMap 10.1 暂不可用，未在 ArcMap 或 QGIS 打开、目视检查图层、表、数据、坐标系、范围、样式或布局。

## 关系名称、序列颜色和后续对象

在同一份 Map.dll 10.1.0.3035 的 Ghidra 导出 `0x7945d630` 与
objdump 指令中复核后续 LoadObject；IID 字节以 PE 映像为准，接口名由
10.1 介质类型库对位。

| 版本条件 | 引用 IID 数据的指令 | 请求接口与读取器 |
| --- | --- | --- |
| 起始名称及 v4 起第二名称 | 辅助函数 `0x7945d350`，第二槽 `0x7945dbbe` | IName，共用旧 Map 表名分派，加入 FileGDB 要素类和查询表名称 |
| v2 起关系循环 | `0x7945d829` | IRelationshipClassName，接入类型库公开的 RelationshipClassName 和 MemoryRelationshipClassName |
| v3 起压缩序列之前 | `0x7945da0b` | IColor，类型库公开的 RgbColor、CmykColor、HsvColor、GrayColor、HlsColor 均已有体读取器 |
| v11 起尾槽 | `0x7945dd8b` | IDisplayExpressionProperties，接入 DisplayExpressionProperties 和已支持的引擎 |
| 仅 v11、v12，额外 short 后 | `0x7945dde1`、`0x7945de09` | IUID、IAnnotateLayerPropertiesCollection，复用各自体读取器 |

ALPC 请求的完整 IID 为 `1D5849F2-0D33-11D2-A26F-080009B6F22B`，
与 [Annotation.dll 接口实现](mxd-terra-nextpass-expression.md) 一致。
v14 在时间字段之后调用 `ExtensionHelper::LoadExtensions`，Save
`0x7945e3c0` 调用 `ExtensionHelper::SaveExtensions`。独立表复用
LayerLib 的旧式尺寸、CLSID 和类体分派，已知扩展的嵌套键共享宿主表。
未知关系、尾槽和扩展停在对应条目起点，保留此前读完的对象及引用键。

构造测试覆盖非空关系名、同键回引、序列前非空颜色、v11/v12 UID 与标注
集合、v14 时间轨道扩展的 LongArray 新键和后继未知扩展，以及截断时的
回滚。参考工程和两套安装介质的 134 份工程仍读到两条根流的终点，这批
工程的 Map 解析未报告独立表体。LinearRef 的两张 v9 独立表仍由 SDK
回归用例覆盖，新增非空组合尚缺 ArcMap 10.1 实档和人工验收。
