# ArcMap 10.1 /Maps 对象图与 Map 持久化结构

记录日期：2026-11-13。本轮为静态逆向研究，未运行 ArcMap，未做任何人工目视验收；所有结论标注证据强度，开放的仍开放。本文对应 TODO「MXD 解析」节中"解析数据框、图层树、图层顺序、图层可见性和数据源连接信息"的前置原生研究。

## 方法与环境

- 服务器与本机研究文件基线见 [arcmap-10.1-native-baseline.md](arcmap-10.1-native-baseline.md)；本轮开工前重新复核全部文件 SHA-256 与基线一致（见下表）。
- REA MCP 6.8.0 可绑定目标（`open_binary` PageLayout.dll 两次成功，SHA-256 校验一致），但其 Ghidra 深度分析提供商被拒绝：提供商要求 `JAVA_HOME` 指向 64 位完整 JDK 25+，而 REA 服务器进程继承 JAVA_HOME=JDK 21（错误码 `unsupported_version`，诊断 `java_command=/usr/lib/jvm/java-21-openjdk/bin/java`）。本轮按恢复指南未重试同一失败，改用本机直接驱动 Ghidra：**Ghidra 12.2**（`/opt/ghidra`，2026-11-11 构建）+ **pyghidra 3.2.0**（Ghidra 自带 wheel 离线安装的独立 venv）+ JDK 25.0.5，逐 DLL 全量自动分析，分析项目缓存于 `/tmp/r3/ghproj*`（会话临时目录）。
- 地址均为对应 PE 10.1.0.3035 静态映像虚拟地址，绑定下表文件摘要；不是运行时地址，也不是 MXD 流内偏移。流内偏移均显式标注 `/Maps:0x…`。
- 安装配置证据：`Setup1.cab` 条目 `esri.clsid.ecfg.6ADD9D9E_C501_4D1B_82A7_B406A96CFD61`（ZIP，SHA-256 `170ae1f812bafc317ac98455eb7be844dd37f8d3a158c454cedf12f5046c3486`）内含 `esri.clsid.xml`：全产品 635 个模块、10675 条 CLSID→progID→Module 映射。这是**安装配置声明**，单独不构成实现证据；其可信度由下文与二进制类表的全量比对支撑。
- REA 返回内容按数据对待；本轮目标内容中未出现指令性文本。共享服务器上有其他 agent 的 Ghidra JVM，未做任何进程级干预。
- 证据包：`temp/arcgis/arcmap101-map-object-evidence.json`（本轮自建，含文件身份、关键地址、类表、样本比对）；同目录 `arcmap101-map-object-rea-pagelayout.json`（同日更早 REA 会话对同一 PageLayout.dll 的证据，Ghidra 12.1.4）。两者含本机路径，分享前应脱敏。

## 文件身份复核（开工前 sha256sum 全部复算）

| 文件 | Setup1.cab 条目后缀 | SHA-256 | 与基线 |
| --- | --- | --- | --- |
| AfCore.dll | `017F52BF_…` | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` | 一致 |
| SystemCore.dll | `017F52BF_…` | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` | 一致 |
| PageLayout.dll | `6BF03936_…` | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` | 一致 |
| MappingCore.dll | `93A93CDE_…` | `ac6f99a6345bab8834bcd8824e50bf93c5ab153de1ce2889fe9e613f080921e7` | 一致 |
| MappingCoreLib.dll | `93A93CDE_…` | `c2bf81dcfcb65473ce35b953c4f4b76392a594085460c299326cc20bed59986a` | 一致 |
| MxTools.dll | `C13FAB27_…` | `d29677c8514e705c6585b1b5a27d4e5cf32f7326891ccfe6ed7beeb70ca51347` | 一致 |
| ArcMap.exe | `E97001B0_…` | `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97` | 一致 |

本轮新提取（同一 Setup1.cab，pefile 读 VS_FIXEDFILEINFO 均为 10.1.0.3035）：

| 文件 | SHA-256 |
| --- | --- |
| Map.dll | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` |
| FeatureLayer.dll | `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60` |
| Layer.dll | `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8` |
| RasterLayer.dll | `9262bc1a05d4ca4cf65613682d3ade93b70c0112cf1caf3e21ea1b168f359de6` |

## 类表机制（证据：确定的）

各 DLL 的 COM 类注册使用同一结构：

- 遍历器：`DllGetClassObject` 跳转的查找函数按 **0x34 字节步长** 线性扫表，条目 `[+0x00]` 为 CLSID（PageLayout 为指针、Map.dll 同为指针形态），命中且 `[+0x30]` 标志为 0 时调用 `[+0x08]` 工厂对象的首槽；未命中返回 `0x80040111`。
  - PageLayout：入口 `0x7b698800` → `0x7b702cf0`，表基址 `0x7b733860`。
  - Map.dll：入口 `0x793cabd0` → `0x7946c470`，表基址指针 `0x794b26f0`。
- 填表初始化把「CLSID 指针、UTF-16 名称、`ClassFactory<类名>::vftable` 工厂对象」写入条目；PageLayout 的填表函数为 `0x7b71a350`（21 条全部解析成功，工厂 RTTI 名称来自反编译中 Ghidra 恢复的 `ClassFactory<X>::vftable`）。

### PageLayout.dll 类表（证据：确定的）

二进制解析 21 条，与 esri.clsid.xml 中 `PageLayout.dll` 模块 21 类**逐条一致（CLSID+名称）**：

| CLSID | 类名 |
| --- | --- |
| dd94d76e-836d-11d0-87ec-080009ec732a | PageLayout |
| dd94d76f-836d-11d0-87ec-080009ec732a | Page |
| 0e66a40e-9087-11d3-9f50-00c04f6bc979 | PageEventsHelper |
| 31e081ac-cb02-11d1-876c-0000f8751720 | RulerSettings |
| 31e081ad-cb02-11d1-876c-0000f8751720 | SnapGrid |
| 56fd87f7-ddce-11d1-8778-0000f8751720 | GraphicSnapEnvironment |
| 31e081ae-cb02-11d1-876c-0000f8751720 | SnapGuides |
| 2fe5e68e-3da5-4bda-baf1-103c55115a1c | MapDocument |
| ddc2346d-62d0-4ea3-ab42-64a9557157c6 | LayerFile |
| 9384b302-16e4-4f27-a6a8-8c0c99f12579 | PageIndex |
| 5012efc9-6d2c-4f0e-8513-5349010cbce3 | DocumentInfo |
| f7ddd85a-3c8c-4646-bbbc-3c4943588c65 | DocumentPreview |
| fc27fab0-db88-11d1-8778-0000f8751720 | GridSnap |
| fc27fab1-db88-11d1-8778-0000f8751720 | GuideSnap |
| fc27fab2-db88-11d1-8778-0000f8751720 | MarginSnap |
| fc27fab3-db88-11d1-8778-0000f8751720 | RulerSnap |
| d20a292b-71aa-4d79-a3ea-74ce3a449887 | PageIndexEventsHelper |
| b6c6ed97-4064-4bbe-84a0-11dd03c697b6 | WebMapDocument |
| cabc70d2-78dc-4467-b0c6-38dade59d927 | WebMapLayerInfo |
| fab38c48-1b32-4088-9e60-e79b5c3eeaa5 | WebMapItemInfo |
| d65b6357-bfa6-4a7d-b448-6b7dbd11b751 | WebMapSlideInfo |

要点：**PageLayout 的类表里没有 Map 和任何图层类**；MapDocument/DocumentInfo 才在这里（与既有 10.8/10.1 研究一致）。

### /Maps 对象图相关类清单（CLSID→类名，含证据强度）

证据强度说明：`确定的` = esri.clsid.xml 声明 + 至少一项二进制证据（类表/RTTI/字节宿主/样本头）相互印证；`候选的` = 仅 esri.clsid.xml 或仅单一证据。

| CLSID | 类名 | ecfg 模块 | 二进制证据 | 强度 |
| --- | --- | --- | --- | --- |
| e6bdaa76-4d35-11d0-98be-00805f7ced21 | Map | Map.dll | Map.dll 内字节 0x79482adc（GetClassID 引用）；Map.dll RTTI `.?AVMap@@` 61 个子接口 COL；MappingCoreLib `0x79bfce70` 经 AfCreateInstance 懒创建 | 确定的 |
| edad6647-1810-11d1-86ae-0000f8751720 | GroupLayer | FeatureLayer.dll | FeatureLayer.dll RTTI `.?AVGroupLayer@@`（TD 0x6f7d3458），Load 见下 | 确定的 |
| e663a651-8aad-11d0-bec7-00805f7c4268 | FeatureLayer | FeatureLayer.dll | FeatureLayer.dll RTTI `.?AVFeatureLayer@@`（TD 0x6f7d2a34）；样本 /Maps 头 @0x296 键 32 | 确定的 |
| d02371c9-35f7-11d2-b1f2-00c04f8edeff | RasterLayer | RasterLayer.dll | RasterLayer.dll 字节命中（样本头 @0x4fc6 键 259，2 次） | 确定的 |
| 167c5ea2-af20-11d1-8817-080009ec732a | LegendGroup | Layer.dll | 样本候选 14 次；ecfg | 候选的 |
| 167c5ea3-af20-11d1-8817-080009ec732a | LegendClass | Layer.dll | 样本候选 269 次；ecfg | 候选的 |
| a2baae2d-969b-11d2-ae77-080009ec732a | FieldInfo | Layer.dll | 样本候选 204 次；ecfg | 候选的 |
| ce8f3972-e9be-11d1-a232-080009b6f22b | ElementCollection | Layer.dll | 样本候选 12 次；ecfg；Layer.dll 字节命中 | 确定的 |
| 34b2ef83-f4ac-11d1-a245-080009b6f22b | FDOGraphicsLayer | AnnoLayer.dll | 样本候选 @0x27a 键 31；ecfg | 候选的 |
| 2a626700-1dd2-11b2-bf51-08002022f573 | ProjectedCoordinateSystem | Geometry.dll | 样本 CRS 对象头 @0x30 键 30；ecfg；MappingCore/PageLayout 字节命中 | 确定的 |
| 198846d0-ca42-11d1-aa7c-00c04fa33a15 | FeatureClassName | GdbCore.dll | 样本 @0x2f6 键 33（FeatureLayer 体内嵌）；ecfg；GdbCore/GdbCoreLib/MxTools 字节命中 | 确定的 |
| 5a350011-e371-11d1-aa82-00c04fa33a15 | WorkspaceName | GdbCore.dll | 样本 @0x370 键 34；ecfg | 确定的 |
| a06adb96-d95c-11d1-aa81-00c04fa33a15 | ShapefileWorkspaceFactory | ShapefileFDB.dll | ecfg；AfCore/MappingCoreLib/MxTools 等字节命中（工厂，非持久化承载） | 候选的 |
| ee535289-41c9-11d1-880a-080009ec732a | BasicOverposterLayerProperties | LabelPlacement.dll | 样本候选 12 次；ecfg | 候选的 |
| 1d5849f3-0d33-11d2-a26f-080009b6f22b | AnnotateLayerPropertiesCollection | LabelPlacement.dll | 样本候选 12 次；ecfg | 候选的 |
| 588e5a11-d09b-11d1-aa7c-00c04fa33a15 | PropertySet | SystemCore.dll | 样本候选 2 次；ecfg；SystemCore 字节命中 | 确定的 |
| 7e4f4719-8e54-11d2-aad8-000000000000 | NumericFormat | SystemCore.dll | 样本候选 68 次；ecfg；多 DLL 字节命中 | 确定的 |
| 78ff7fa1-fb2f-11d1-94a2-080009eebecb | UID | SystemCore.dll | 样本候选 13 次；ecfg；Map/FeatureLayer/Layer 字节命中 | 确定的 |
| 7ee9c496-d123-11d0-8383-080009b996cc | RgbColor | DisplayCore.dll | 样本候选 667 次；ecfg | 确定的 |
| 7914e603-c892-11d0-8bb6-080009ee4e41 | SimpleFillSymbol | DisplayCore.dll | 样本候选 301 次；ecfg | 确定的 |
| 7914e5f9-c892-11d0-8bb6-080009ee4e41 | SimpleLineSymbol | DisplayCore.dll | 样本候选 282 次；ecfg | 确定的 |
| 7914e5fe-c892-11d0-8bb6-080009ee4e41 | SimpleMarkerSymbol | DisplayCore.dll | 样本候选 12 次；ecfg | 确定的 |
| b65a3e74-2993-11d1-9a43-0080c7ec5c96 | TextSymbol | DisplayCore.dll | 样本候选 28 次；ecfg | 确定的 |
| f3435801-5779-11d0-98bf-00805f7ced21 | SimpleRenderer | FeatureLayer.dll | 样本候选 13 次；ecfg | 确定的 |
| d5bb4b88-e0a1-11d2-9f4d-00c04f6bc78e | FeatureIDSet | FeatureLayer.dll | 样本候选 13 次；ecfg；FeatureLayer.dll 字节命中 | 确定的 |
| 9646bb83-9512-11d2-a2f6-080009b6f22b | CompositeGraphicsLayer | FeatureLayer.dll | 样本候选 1 次；ecfg | 候选的 |
| 0be35203-8f91-11ce-9de3-00aa004bb851 | （OLE StdFont，非 ArcObjects） | — | 样本候选 37 次；ecfg 无此条 | 确定的（OLE 标准） |

其余说明：MxTools.dll 的二进制类表由 C++ 全局对象数组构成（析构迭代器 `_eh_vector_destructor_iterator_(0x1029bd90, 0x34, 0x108, …)` 证明 **264 条、步长 0x34**），静态映像为零、由全局构造逐条填充，本轮未能完整恢复填表代码；esri.clsid.xml 声明 MxTools.dll 263 类，与 264（含空结束条）吻合，故 MxTools 的类名清单以 esri.clsid.xml 为准（`temp/arcgis/arcmap101-map-object-evidence.json` 存有五模块完整清单）。MappingCoreLib.dll 在 esri.clsid.xml 中为 0 类，但其代码经 `AfCreateInstance` 创建 Map（`0x79bab130`）与 FeatureLayer（`0x79babab0`）——即实际宿主与 ecfg 模块名（Map.dll/FeatureLayer.dll）存在"工厂宿主/声明模块"差异，使用时以二进制证据为准。

## 样本首对象比对（问题 2：命中）

仓库参考样本 `samples/测试底图.mxd`（/Version = 文本 "10.1"、short 19、计数 2267）导出 `/Maps` 152941 字节：

- `/Maps:0x00` 4 字节有符号数量 = **1**（一个数据框）。
- `/Maps:0x04..0x13` 首对象类标识 = **`e6bdaa76-4d35-11d0-98be-00805f7ced21`** → esri.clsid.xml 命中 **Map**（模块声明 Map.dll）；字节宿主证据显示该 CLSID 的工厂代码在 Map.dll（`0x79482adc` 被读取它的函数引用）。**首对象是 Map（数据框），命中确定。**
- `/Maps:0x14..0x17` 对象键 = **29**。
- 对象体 `/Maps:0x18` 起与 Map::Load 反编译逐字段吻合（见下节）；名称字段为「新建数据框」（中文版 ArcMap 默认数据框名）。

## Map 顶层持久化结构（问题 3：证据：确定的）

Map 的持久化读取函数定位链：Map.dll RTTI `.?AVMap@@`（TD `0x794b4250`）→ 61 个子接口 COL（`0x79493b7c` 起，步长 0x14）→ 各子接口虚表 → **IPersistStream 子接口虚表 `0x794874a4`，`+0x14` 槽 = Load `0x793ff4d0`**（9 KB 代码；同族虚表槽 `+0x0C` 应为 GetClassID，本轮未逐槽复核）。Load 读取顺序（伪代码顺序即流内顺序）：

| # | 字段（流内） | 证据（Map.dll） | 备注 |
| --- | --- | --- | --- |
| 1 | int16 架构版本 | `0x793ff4d0` 开头 `AfStreamRead(2)`，**>0x33（51）直接拒绝** | 样本值 51；这是每对象头部的"类版本"字段 |
| 2 | 字符串 → obj+0x160 | `String::Read` | 数据框名称（样本「新建数据框」） |
| 3 | 字符串 → obj+0x164 | `String::Read` | 第二名称字段（样本为空串：长度 2 仅终止符） |
| 4 | 嵌套对象 ×1 | `LoadObject(…)` 帮助函数（读 16 字节类标识 + 4 字节键，经 ObjectStream 创建/引用） | 样本：CRS `2a626700-…` 键 30；类身份来自流内对象头 |
| 5 | int32 计数 + 嵌套对象 ×N | `AfStreamRead(4)` 循环 `LoadObject`（请求 IID `fe3115e9-…`） | 计数类序列 |
| 6 | 32 字节 WKSEnvelope | `AfStreamRead(0x20)` | 原始字节，非对象 |
| 7 | int32 → 子接口 +0x38 / int32 → +0x40 | 各 `AfStreamRead(4)` | 用途未定性 |
| 8 | 嵌套对象 ×1 | `LoadObject`（请求 IID `1b077914-…`） | 用途未定性 |
| 9 | int32 计数 + 嵌套对象 ×N | 循环 `LoadObject`（请求 IID `ee53528b-…`），每对象加入集合（`FUN_7941c760`） | 图层序列最可能在此（未逐对象定性） |
| 10 | int16 → ObjectStream 子接口 +0x44 | `AfStreamRead(2)` 后调用 obj−0x18 处 ObjectStream 的 +0x44 | 嵌套对象头（10.8 同款） |
| 11 | int16 → obj+0x19c / double → obj+0x200 / int16 → obj+0x264 | `AfStreamRead(2/8/2)` | double 后随 `FUN_7940fd80(0, v)` |
| 12 | int32 计数 + 版本 < 0xB 才循环 `LoadObject`（`4ba33331-…`） | 旧版本分支 | 10.1 写 51，不进入 |
| 13 | …（后续为大量版本分支字段：short/double/String 等，含 `String::Read → obj+0x398` 等） | `0x793ff4d0` 全函数 | 未逐项展开 |

与样本字节对照：`/Maps:0x18` short=51 ✓；`0x1a` 名称「新建数据框」✓；`0x2a` 空名称（`02 00 00 00 00 00`）✓；`0x30` CRS 对象头 + 键 30 ✓；`0x44` CRS 体 short=6；`0x46` WKT 长度 455（不含终止符）+ WKT，与既有 10.8 文档记录的偏移完全一致。注意第 2 字段样本里的 `02 00 00 00` 曾可能被误读为"int32 计数"，实际是空字符串长度前缀——字段序以反编译为准。

嵌套 `LoadObject` 的第二参数为**请求接口 IID**（如 `34b2ef81-…`、`fe3115e9-…`），不是类 CLSID；嵌套对象的类身份完全由流内 16 字节对象头决定。这与 10.8 SystemCore 研究的"读头→查表→AfCreateInstance→持久化"模型一致。

### 字段序闭合补充（2026-11-13 第二轮：反编译产物逐段核对）

上一轮的反编译产物（会话临时目录 `/tmp/r3/`）按伪代码逐段复核，闭合了下表步骤的版本分支与请求 IID。证据同为 Map.dll/FeatureLayer.dll/Layer.dll 10.1.0.3035（SHA 见证据包），Ghidra 12.2 伪代码，地址为静态映像 VA。

Map::Load `0x793ff4d0` 补充（步骤号接上表）：

| # | 流内字段 | 版本条件 | 证据（伪代码行语义） |
| --- | --- | --- | --- |
| 4 的 IID | 请求 IID `DAT_79481cc0` | 无条件 | 空间参考嵌套对象的请求接口（第 4 步 `LoadObject` 第二参） |
| 5 | 版本 0x16..0x1b 时先调 `FUN_79403ec0(obj-0x68, ver)`，随后**无条件** `AfStreamRead(4)` 计数 + 循环 `LoadObject`，请求 IID `DAT_79481a84` | 计数无条件读 | 第 5 步计数序列；0x16..0x1b 前置处理为对象状态重建，语义未定性 |
| 7 的目标 | 两个 4 字节值分别传入 `obj-0x40` 子接口虚表 `+0x38` / `+0x40` | 无条件 | 第 7 步的两个 int32 是子接口属性写入，非本地存储 |
| 8 的 IID | 请求 IID `DAT_79481580` | 无条件 | 第 8 步嵌套对象 |
| 9 的 IID | 请求 IID `DAT_794815a0`；每对象经 `FUN_7941c760` 加入集合 | 无条件 | 第 9 步循环为图层序列的最强候选（每对象"加载后加入集合"结构），类身份仍由流内对象头决定 |
| 10 | `AfStreamRead(2)` 后调用 `obj-0x18`（ObjectStream）虚表 `+0x44` | 无条件 | 第 10 步的 int16 先读，随后 +0x44 是"16 字节 GUID + 4 字节键"嵌套对象头读取（同根入口循环） |
| 13（部分） | ≥0x13..0x16：`LoadObject(DAT_79488470)`；>0x16：另一 `LoadObject`；≥0x18：short(+0x32a)+double(+0x320)+short(+0x328)；≥0x1a：9 个 long（+0x2a8..+0x2d4）；≥0x1b：`FUN_79403d50`；≥0x1c：short；≥0x1d：`String::Read→+0x398`；≥0x2b：short+`String::Read→+0x428`；>0x2b：`LoadObject`；≥0x2d/0x2f：short(+0x450/+0x454)；>0x2f：long；==0x31：long 计数+循环 `LoadObject`+short；≥0x23 else 分支：short(+0x1a0)+short+long(+0x1b0) | 逐项见左 | 第 13 步仍远未穷尽；≥0x33 分支存在（样本 51=0x33 走最终 else 前的长链） |
| 尾部 | 版本 <4：原始 `WKSPointZ` 0x18 字节 + double；≥4：`LoadObject(DAT_79487e60)`；随后版本分支默认值/short(+0x1ac)/short(+0x268)/long(+0x1a4/+0x1a8)/double 等；两个计数循环（<0xb：`LoadObject(DAT_79482c08)` 逐项 `+0x14` 虚调用；≥0xb：`LoadObject(DAT_79488300)` 逐项存 +0x20 接口）；函数尾 `LoadObject(DAT_79487fc0)` | 逐项见左 | 第 12 步两个计数循环即 <0xb / ≥0xb 双分支（上表第 12 行的展开） |

GroupLayer::Load `0x6f711270` 完整字段序（上一节第 149 行的展开；`0xc < ver` 拒绝 0x80004005，上限 12）：

| 字段（流内） | 版本条件 | 备注 |
| --- | --- | --- |
| int16 版本 | 恒有 | >12 拒绝 |
| String 名称 → obj+0x9c | 恒有 | |
| int16 ×3 → +0xa0/+0xa2/+0xa4 | 恒有 | 语义未定性 |
| double ×2 → +0xa8/+0xb0 | 恒有 | |
| int32 计数 + 循环子图层 `LoadObject`（IID `DAT_6f7aa4ec`） | 恒有；ver<11 走 `LoadObject`+`FUN_6f7021d0`，≥11 走 `ExtraSafeReadObject` | 内联序列；≥11 的 ExtraSafeReadObject 差异未展开 |
| int16 可见性 → +0xbc | <2 默认 0xffff，≥2 读取 | VARIANT_BOOL，与 FeatureLayer 同构 |
| 嵌套 `LoadObject`（IID `DAT_6f7aa4bc`） | ≥2 | |
| double → +0xc8 | ≥4 | |
| 扩展序列 `ExtensionHelper::LoadExtensions(+0x100)` | ≥5 | |
| String → +0xdc；==6 再读 short | ≥6 | |
| short → +0xd6 | ≥7 | |
| double ×2 → +0xe0/+0xe8 | ≥7 | |
| short ×3（local、+0xf2、+0xf4；首值传入 +0x78 接口 +0x24） | ≥9 | |
| 嵌套 `LoadObject`（IID `DAT_6f7aafb8`） | ≥11 | 结果存 +0x7c 接口 |
| LayerMasking 初始化 | 尾部 | 版本 <0xc 且 +0xf0 非零时有条件循环 |

FeatureLayer::Load `0x6f6e9940` 头部段补充（上限 0x22=34）：int16 版本 → String 名称(+0x120) → String 第二名称(+0x124) → int16 ×3（+0x128 可见性 VARIANT_BOOL / +0x12a / +0x12c）→ 嵌套 `LoadObject`（数据源连接）→ **原样 16 字节 GUID → obj+0x144**（`AfStreamRead(0x10)`，非对象头，未定性——候选为数据源扩展标识）。样本首 FeatureLayer（键 0x4f 起，版本 34）与此段逐字段吻合。

## /Maps 顶层入口（证据：确定的）

PageLayout `0x7b708d00`（基线已见其存在；本轮补 ASCII 流名证据 `Maps` @`0x7b7211b0`、`DrawingDefaults` 引用 `0x7b70e2e0`、`Mx Document` 宽字符 @`0x7b71dc20`）：先尝试 `Mx Document` 流（新格式分支，Ghidra 对其主体拆段，未逐行复核），失败后走：DrawingDefaults 处理 → 打开 `Maps` 流 → 连接 ObjectStream（子接口 +0x38）→ `AfStreamRead` 读 4 字节有符号数量 → 循环数量次调用 **ObjectStream +0x44**（16 字节类标识 + 4 字节键，查引用表/创建实例）→ 循环后再读 4 字节值（写入调用方输出）→ 同一 ObjectStream 依次转接 `PageLayout`、`View` 流。与 10.8 记录的 `0x10075370` 流程完全同构。

## 图层树、可见性与数据源连接（问题 4/5）

- **GroupLayer（FeatureLayer.dll）**：Load `0x6f711270`（IPersistStream 虚表 `0x6f7af244` +0x14）。顺序：int16 版本 → 字符串名称（obj+0x9c）→ int16 ×3（obj+0xa0/+0xa2/+0xa4）→ double ×2（obj+0xa8/0xb0）→ **int32 子图层计数 + 循环 `LoadObject`**（每个子图层是独立嵌套对象：自己的 16 字节类标识 + 键 + 对象体）→ 后续版本分支字段与 LayerMasking。即嵌套采用**内联序列 + 引用键**：首现内联写体，其后引用只写头。
- **FeatureLayer（FeatureLayer.dll）**：Load `0x6f6e9940`（虚表 `0x6f7acfb0` +0x14）。顺序：int16 版本 → 名称（obj+0x120）→ 第二名称（obj+0x124）→ int16 ×3（obj+0x128/+0x12a/+0x12c）→ `LoadObject`（数据源连接）→ 原样 16 字节 GUID（obj+0x144）→ 字符串/double ×2/int16 → 后续渲染/标注等大量字段（未逐项展开）。
- **可见性**：样本首个 FeatureLayer 体（`/Maps:0x2aa` 版本=34，名称「龙安镇人民政府工作底图用界址点」，第二名称「Shapefile 要素类」）在两名称后第一个 int16 = **0xFFFF @/Maps:0x2f0**，即 `obj+0x128`，VARIANT_BOOL 布尔——0xFFFF 是 VARIANT_TRUE，与"该图层可见"一致（证据：确定的，代码偏移 + 样本值互证）。GroupLayer 对应候选为名称后第一个 int16（obj+0xa0）；其余两个 short 语义未定性（候选的）。
- **数据源连接链**：FeatureLayer 体内嵌 **FeatureClassName**（`198846d0-…`，样本 @0x2f6 键 33，体首 short=2 后接要素类名字符串）→ 其后出现 **WorkspaceName**（`5a350011-…`，样本首现 @0x370 键 34），体内含工作空间描述字符串；样本可见 `..\22.界址点`（相对路径）与 `Shapefile Data`（工作空间类型描述）等字符串（敏感路径，公开前脱敏）。承载类归属：FeatureClassName/WorkspaceName 声明于 GdbCore.dll（esri.clsid.xml），ShapefileWorkspaceFactory 声明于 ShapefileFDB.dll（工厂，不随 MXD 持久化）。写入接口：与所有 /Maps 对象一致，经 ObjectStream 的普通持久化分支（对象体由各对象 IPersistStream::Save 写出，本轮未逐类分析 Save）。

## 坐标系与显示范围字段闭合（2026-11-14 第三轮）

对应 TODO「MXD 解析」第 67 行「解析地图坐标系、图层坐标系和显示范围」。证据来源：REA 6.9.0 / Ghidra 12.2 静态分析 Map.dll 与 FeatureLayer.dll（均 10.1.0.3035，SHA 见上文与本节文件身份），地址为静态映像 VA；样本字节定位用 `/tmp/r3-maps.bin`（SHA-256 `9852c0e2…8129b49`，偏移为 /Maps 流内偏移）。证据包：`temp/arcgis/arcmap101-map-cr-extent-evidence.json`（50 条，含本机路径，分享前脱敏）。两份 DLL 在本轮先后打开、由本轮关闭；未运行目标软件。未解决的问题见文末更新。

### Map::Load 空间参考分支复核与补全（证据：确定的）

Map::Load `0x793ff4d0` 第 4 步（嵌套空间参考对象）及其后继序列复核（接上文第二轮表）：

- 嵌套空间参考对象的请求 IID 数据在 Map.dll `DAT_79481cc0`，16 字节 `80d63444 daf4d111 bca60000 f875bcce` = `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`（`ev_38787b6f…`；该值即公开 ArcObjects 的 ISpatialReference IID，名称按 SDK 惯例标注，二进制只给字节）。对象类身份仍由流内 16 字节类标识决定（样本 `{2A626700-…}` PCS，键 30）；16 字节零类标识为空对象标记（SystemCore ObjectStream +0x44 既有证据，本节未重复取证）。PCS/GCS/Unknown 分派发生在类表创建层，Map.dll 本体不按 GUID 分派。
- 加载成功后 SR 对象指针存 `param_1+0x148`（对象内存，非流字段）；随后 `FUN_794035f0((void *)(param_1-0x68), pIVar10, ver)`：
  - **ver ≥ 0x17（样本 51 走此支）**：`AfStreamRead(4)` 计数 → 逐项 `AfStreamRead(2)` short → short==1 时 `LoadObject(DAT_79485a18)`，short==2/3 时 `ExtraSafeReadObject(DAT_79485a18)`（short 为其他值直接终止该轮）；循环后 `FUN_7940d240` + `AfStreamRead(4)` 尾部 int32（0≤idx<count 时 `FUN_794199f0`）。`DAT_79485a18` = `{34C20002-4D3C-11D0-92D8-00805F7C28B0}`（`ev_3cf51834…`）。样本对照：`/Maps:0x274` = `06 00 00 00`（计数 6）、`0x278` = `01 00`（short 1）、`0x27a` 起对象头 `{34B2EF83-F4AC-11D1-A245-080009B6F22B}` 键 31（类名候选 FDOGraphicsLayer，见上文类清单）。此支语义候选：图形/注记层集合 + 活动项索引，未逐对象定性。
  - **ver < 0x17**：`FUN_794038a0`（`ev_75a919e6…`）：ver==0x11 直接拒绝 `0x80004005`；int32 计数 + 循环（ver<0x11 用 `LoadObject(DAT_79485a18)`、≥0x11 用 `ExtraSafeReadObject(DAT_79485a18)`）→ `FUN_7940d240` → 第二个 int32 计数 + 循环 `LoadObject(DAT_79485a18)` → `FUN_7940d240` → 尾部 int32（≥-1 时 `FUN_794199f0`）。即旧版为「两个同类集合 + 尾部索引」。
- 其后继序列（补全第二轮表顺序）：`LoadObject(DAT_79487fe0)`，`DAT_79487fe0` = `{34B2EF81-F4AC-11D1-A245-080009B6F22B}`（`ev_064600bd…`，与 FDOGraphicsLayer CLSID 同族、末字节不同——请求 IID ≠ CLSID 的又一例）→ ver∈[0x16,0x1b] 时 `FUN_79403ec0` → int32 计数 + 循环 `LoadObject(DAT_79481a84)`（= `{FE3115E9-ED6C-11D0-8683-0000F8751720}`，`ev_ba5dfb60…`）→ **WKSEnvelope 32 字节原样读**（见下小节）→ int32 → `(param_1-0x40)` 子接口 +0x38 → int32 → +0x40 → `LoadObject(DAT_79481580)`（= `{1B077914-38C5-11D0-92D2-00805F7C28B0}`，`ev_fea0fc68…`）→ int32 计数 + 循环 `LoadObject(DAT_794815a0)`（= `{EE53528B-41C9-11D1-880A-080009EC732A}`，`ev_1f04d598…`），每项经 `FUN_7941c760` 以 `DAT_79488d18` 查询后加入集合；`DAT_79488d18` = `{3036D35F-EDE5-11D0-87FE-080009EC732A}`（`ev_583d538d…`；16 字节 `5fd33630 e5edd011 87fe0800 09ec732a`，2026-11-14 第四轮从文件字节复核，比 FeatureLayer.dll 计数循环 IID `3036D35D` 大 2）。据此第二轮表第 9 步「图层序列候选」的请求 IID 闭合：`{EE53528B…}`；第 5 步循环 IID 闭合：`{FE3115E9…}`。

Map::Load 其余分支的请求 IID 字节本轮一并解码（名称均按 SDK 惯例标注为候选）：`DAT_79482c08`=`{4BA33331-D55F-11D1-8882-0000F877762D}`（ver<0xb 循环，`ev_2099fb0c…`）、`DAT_79488300`=`{FFC6B179-E3EC-11D3-A096-00C04F6BC626}`（ver≥0xb 循环，`ev_f5fc03ef…`）、`DAT_79487e40`=`{9273BB60-C611-11D2-BD08-0000F875BCCE}`（ver 0xc/0xd 与 >0xd 循环，`ev_ea2dc20c…`）、`DAT_79487e60`=`{1C352F3F-298E-11D3-9F4F-00C04F6BC619}`（尾部 ver≥4 LoadObject，`ev_58c7c40f…`）、`DAT_79487fc0`=`{E4240307-DD41-11D2-B868-00600802E603}`（函数尾 LoadObject，`ev_4b165f9f…`）、`DAT_79487fd0`=`{E4240309-DD41-11D2-B868-00600802E603}`（>0xf 分支，与上者同族）、`DAT_79488470`=`{8C439003-14EC-11D2-A27E-080009B6F22B}`（ver 0x13..0x16 与 ≥0x13 分支）、`DAT_79488050`=`{E265C5B5-4513-4BB4-B280-27B03AAE5957}`（>0x16 分支）、`DAT_79488460`=`{E6BDAA75-4D35-11D0-98BE-00805F7CED21}`（≥0x22 分支；比 Map CLSID `E6BDAA76` 小 1，即 IMap 接口候选）、`DAT_79487fa0`=`{40A9E885-5533-11D0-98BE-00805F7CED21}`（≥0x22 分支）、`DAT_79480bbc`=`{00000000-0000-0000-C000-000000000046}`=IID_IUnknown（≥0x23 else 计数循环）。

### 地图显示范围（WKSEnvelope，证据：确定的）

- Map::Load 在第 5 步计数循环之后、两个 int32 之前，先在栈上 `WKSEnvelope::WKSEnvelope(local_a0)` 构造再 `AfStreamRead(pIVar10, local_a0, 0x20)` **原样读 32 字节**（非嵌套对象，无类标识）；经 `param_1+0x230` 子接口虚表 `+0xe0` 槽写入（`map_load_real.c` 第 175-179 行语义）。这是数据框的显示范围字段：4 个 double、36 字节流内位置固定，不随版本分支。
- 字段序：4 个 double 按 `[XMin, YMin, XMax, YMax]` 候选。直接字节证据来自同布局的 Envelope 对象（下节）：25 例全部满足第 1<第 3、第 2<第 4 值且呈「大-小-大-小」交替，若按 `[XMin,XMax,YMin,YMax]` 解读则自相矛盾；`WKSEnvelope` 类型名来自反编译类型推断，标注候选。
- 样本定位（候选的）：`/Maps:0x24f64` 起 4 double = `37392944.2840, 2520083.2851, 37394230.5290, 2520951.8351`，随后两个 int32 = `9, 9`，再后为对象头 `{1C352F3F-…}` 键 2178——与「包络 32 字节 → int32 → int32 → LoadObject」的结构吻合；但首个 FeatureLayer 体（键 32）之外的对象边界未逐段闭合，不能排除该四 double 属于某图层体。数据并集的整值包络 `37357978.0/2493328.0/37424022.0/2551672.0`（如 `/Maps:0x24306` 起，Envelope 键 2149 体内，共 4 处）是某对象持久化的「全图范围」，不是本字段。

### Envelope 对象：图层范围与图层坐标系的载体（证据：确定的）

- 类身份：`{30707212-52D5-11D0-A8F2-00608C85EDE5}`，esri.clsid.xml 声明 Geometry.dll 模块类名 `Envelope`（安装配置声明见上文；清单存于 `temp/arcgis/arcmap101-map-object-evidence.json`，样本 25 处命中，`temp/r3/sample_guid_resolved.json` 会话产物）。
- Envelope 对象体布局（样本 25/25 逐字节一致，Geometry.dll 本体本轮未反编译，布局为样本侧观察）：
  1. short 版本 = 3；
  2. 4 × double：`[XMin, YMin, XMax, YMax]`（8 字节非对齐紧排）；
  3. int32 = 0（样本恒零；语义未定性，候选 Z/M 域标志）；
  4. 嵌套空间参考对象：内嵌（16 字节类标识 + 4 字节键 + PCS 体，布局同上文 Geometry.dll 保存方法）**或** 20 字节回引用（16 字节类标识 + 4 字节既有键，不跟对象体）。
- 样本对照：25 个 Envelope 的键为 74, 80, 151, 216, 258, 1603, 1609, 1667, 1672, 1724, 1730, 1782, 1788, 1840, 1846, 1923, 1929, 2001, 2007, 2073, 2079, 2149, 2155, 2182, 2183。其中 10 个内嵌 PCS 体（Envelope 键 80, 151, 216, 1609, 1672, 1730, 1788, 1846, 1929, 2079，其后紧随 10 个新 PCS 对象：键 81, 152, 217, 1610, 1673, 1731, 1789, 1847, 1930, 2080），其余 15 个（键 74, 258, 1603, 1667, 1724, 1782, 1840, 1923, 2001, 2007, 2073, 2149, 2155, 2182, 2183）的嵌套段为 20 字节回引用，目标均为 Map 的 PCS 对象（键 30）。
- 由此修正上文空间参考文档的定位观察：「空间参考对象头之前紧邻 32 字节恰为 4 个 double」应精确为：**范围 4 double 属于紧邻的 Envelope 对象体，其结构为 `03 00` + 32 字节包络 + `00 00 00 00` + 空间参考头/回引**，包络头（16 字节 `{30707212-…}` + 4 字节键）在四 double 之前 22 字节处。
- **图层坐标系字段的位置与读取规则（问题闭合）**：FeatureLayer 体不直接持久化坐标系字段；图层坐标系作为图层体内 **Envelope（范围）对象的第 4 段嵌套空间参考**持久化——内嵌 PCS 体（Geometry.dll 布局：short 版本 + WKT 长度 + ANSI WKT + 4 个域字符串 + 1 字节零 + 2 字节标志 + 7 double + 3 double）或对 Map 空间参考对象的键回引。样本归属：首 FeatureLayer（键 32，注释内层，体 `0x2aa..0x2d2c`）含 3 个 Envelope——键 74 @`0xedd`（回引 30）、键 80 @`0x122b`（内嵌 SR 81）、键 151 @`0x2872`（内嵌 SR 152）；FeatureLayer 键 158（体 `0x2d5e..0x42c6`）含 Envelope 216 @`0x3f44`（内嵌 SR 217）；FeatureLayer 键 218 体含 Envelope 258 @`0x4e74`（回引 30）。两个 Envelope 的语义（候选：数据全范围 vs 当前/显示范围）、同 CRS 时内嵌/回引的选择规则、以及每层 Envelope 数量差异（1..3 个）的成因未闭合。

### FeatureLayer::Load 请求 IID 字节表（证据：确定的；接口名未确认）

FeatureLayer.dll（SHA-256 `2baf8706…98bb60`）`0x6f6e9940` 反编译中全部 `LoadObject`/集合循环的 IID 数据（`/tmp/r3/featurelayer_load.c` + 本轮 `read_bytes`）：

| 数据地址 | IID 字节解码 | 反编译位置 |
| --- | --- | --- |
| `DAT_6f7a98cc` | `{40A9E884-5533-11D0-98BE-00805F7CED21}` | 数据源连接之后的首个 LoadObject |
| `DAT_6f7ac2a4` | `{3036D35D-EDE5-11D0-87FE-080009EC732A}` | 计数循环（与 Map 图层集合 QI `{3036D35F…}` 同族、末字节差 2，见 2026-11-14 第四轮修正） |
| `DAT_6f7aaf98` | `{20CD40B0-8F3D-11D0-8590-0800091A2A72}` | 计数循环后的 LoadObject |
| `DAT_6f7ab098` | `{A2BAAE2C-969B-11D2-AE77-080009EC732A}` | 计数循环（比 FieldInfo CLSID `A2BAAE2D` 小 1） |
| `DAT_6f7adda4` | `{1D5849F2-0D33-11D1-A26F-080009B6F22B}` | LoadObject（比 AnnotateLayerPropertiesCollection CLSID `1D5849F3` 小 1） |
| `DAT_6f7ac380` | `{D5BB4B87-E0A1-11D2-9F4D-00C04F6BC78E}` | LoadObject（比 FeatureIDSet CLSID `D5BB4B88` 小 1） |
| `DAT_6f7aafb8` | `{19F85377-1387-11D3-B89C-00600802E603}` | ver≥2 LoadObject |
| `DAT_6f7aaf28` | `{4BA33331-D55F-11D1-8882-0000F877762D}` | ver≥4 LoadObject（与 Map ver<0xb 循环同值） |
| `DAT_6f7adc24` | `{311C2EE2-DA65-11D2-8A54-000000000000}` | ver>6 计数循环 |
| `DAT_6f7aa4bc` | `{1B077914-38C5-11D0-92D2-00805F7C28B0}` | ver≥8 与 ver≥0xe 各一次（与 Map 步骤 8/≥0x22 分支同值） |
| `DAT_6f7a97dc` | `{F3435802-5799-11D0-98BF-00805F7CED21}` | ver≥0xd LoadObject（比 SimpleRenderer CLSID `F3435801` 大 1） |
| `DAT_6f7ada24` | `{1714D59B-FB22-11D1-94A2-080009EEBECB}` | ver≥0x11 LoadObject |
| `DAT_6f7ab7a8` | `{A62B9EC2-5C52-4B98-B116-5EA341D4AF6D}` | ver≥0x19 LoadObject |
| `DAT_6f7adab4` | `{F6CBA4A3-BC48-47BC-9F15-2A561B9E6C71}` | ver≥0x1d LoadObject |
| `DAT_6f7adad4` | `{BD724B95-018F-4367-9883-9560D92293A7}` | ver≥0x1e LoadObject |
| `DAT_6f7adde4` | `{E720618B-99A9-46D7-AA6B-507898E1EA90}` | ver==0x1f 与 ver>0x20 LoadObject |

「CLSID±1」的 IID 候选关系仅作线索（esri 类库常见「CLSID n、主接口 n±1」布局），接口身份未确认。FeatureLayer::Load 反编译的头部段与中后段之间，样本中段（渲染器/字段信息/标注集合等对象序列）与反编译字段的逐字节对位未完成；两个 Envelope 对象由哪个 LoadObject 装入未闭合（开放问题）。

## Map::Save 方向、ExtraSafe 前缀与图层列表样本闭合（2026-11-14 第四轮）

对应 TODO「解析数据框、图层树、图层顺序、图层可见性和数据源连接信息」：把第三轮标为候选的 `FUN_794035f0` 集合升级为样本逐条目闭合，并闭合 Save 写侧与 `ExtraSafeReadObject`/`ExtraSafeWriteObject` 前缀语义。证据：REA 6.9.0 / Ghidra 12.2 复用会话（Map.dll，SHA-256 `1c0675cc…`，会话开工前已由其他轮打开，本轮直接复用、未关闭）；AfCore.dll 与 AnnoLayer.dll 用本机 pyghidra（Ghidra 12.2 + JDK 25，`/tmp/r3/venv`，项目 `/tmp/r3/ghproj_afcore`、`/tmp/r3/ghproj_anno`）。本轮新提取 AnnoLayer.dll（`Setup1.cab` 条目 `6BF03936_…`，版本 10.1.0.3035，SHA-256 `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5`）与 LabelPlacement.dll（10.1.0.3035，SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`），仅做字节扫描与 RTTI 定位，未反编译其函数体。样本偏移为 `/tmp/r3-maps.bin` 流内偏移。全部为静态观察，未运行目标软件。

### Map.dll ImageBase 与 IAT（证据：确定的）

Map.dll PE ImageBase = `0x793a0000`（pefile 复核），AfCore 导入经 IAT 解析：`0x79480030`=`AfStreamRead(PAJ)`、`0x79480040`=`SaveObject(IUnknown*, IStream*, int)`、`0x79480048`=`LoadObject`、`0x794801e4`=`ExtraSafeReadObject`、`0x794801e8`=`ExtraSafeWriteObject`（AfCore 导出表序号 1170/1171：读 `0x68a7be60`、写 `0x68a7bd70`）。第三轮已载的 `DAT_79480da4`=1、`DAT_79480da8`=2（short 静态值，文件字节复核）。

### ExtraSafeReadObject / ExtraSafeWriteObject（证据：确定的；闭合未解决事项 4）

AfCore `ExtraSafeReadObject(0x68a7be60)`：先 `IStream::Read` **8 字节前缀 `[A:int32][B:int32]`**；`A==0 && B==0` 时返回 1（调用方按"空条目"跳过）；否则 `LoadObject`（16 字节类标识 + 4 字节键 + 对象体，与普通嵌套对象完全同布局），之后 `Seek(0, SEEK_CUR)` 取实际终点，仅当 `B != 0` 时强同步（Seek 回 `前缀后位置 + B`）。即前缀不改变对象体布局：**B=0 表示信任 LoadObject 自封闭**，B≠0 是"预期体积"恢复路径。流内布局与 GroupLayer ≥0xb 分支观察一致。

AfCore `ExtraSafeWriteObject(0x68a7bd70)`：`Seek(0,SEEK_CUR)` 记起点 → 写 8 字节零占位 → `SaveObject`（16 字节 GUID + 键 + 未登记对象的体）→ 记终点 → 回写占位为 `[A][B]`，样本中观察到 `[A=对象总体积（头 20 字节+体）][B=0]`（占位补丁的逐字段语义以反编译为准，样本只给出观察值）。

### Map::Save 主体顺序（0x79401850，证据：确定的）

Map.dll `FUN_79401850` 为 Map 的 Save 主体（写侧，与 Load 逐段对称）：AfStreamGetDocumentVersion/AfStreamGetSaveVersion 写 2 字节版本 → `String::Write(+0x160)`、`String::Write(+0x164)` → `SaveObject(+0x148)`（空间参考）→ **图层列表写侧**（docVersion==0 走 `FUN_7940cd80` 旧双集合 + 索引；否则 `FUN_794033a0`）→ `SaveObject(+0xf8)`（基本图形层对象）→ `int32(+0x130) 计数 + 循环 SaveObject(+0x12c 数组)` → WKSEnvelope 32 字节（取自 `param_1+0x230` 子接口 +0xdc）→ 两个 int32（子接口 +0x34/+0x3c）→ `SaveObject(+0x168)` → `int32(+0x174) + 循环 SaveObject(+0x170 数组)` → 两个 `0xFFFF` short → 之后与 Load 的版本分支字段逐段对应（+0x19c/+0x200/+0x264 等）。Map::Save 尾段（transformations 循环之后的字段）未逐项展开（开放）。

图层列表写侧 `FUN_794033a0`：写 `*(+0x154)` 图层数组计数 → 每图层写 **2 字节标志** 再写对象：走 `SaveObject` 的条目标志取静态值 `DAT_79480da4`=1；走 `ExtraSafeWriteObject` 的条目标志在 docVersion<4 时写 2、否则写 3（10.1 的 docVersion=3 → 样本全部为 2，与观察一致）；图层指针为空时标志取 `DAT_79480da8`=2。哪一状态使图层走标志 1 的 SaveObject 路径未逐分支定性（候选：每轮先行调用的 `FUN_794199f0` 相关状态）。循环后写 4 字节活动图层索引（`local_2c` 初值 -1，匹配 `*+0x164` 或 `FUN_79419150` 判定时记录当前序号）。`FUN_7940cd80`（docVersion==0 旧路径）：按 `+0x150/+0x154` 数组写 `ExtraSafeWriteObject` 循环 → 第二个 int32 计数 + `SaveObject` 循环 → 尾部 4 字节索引。

读侧 `FUN_794035f0`（第三轮已载）补全：图层对象经 `FUN_79410040` 加入 Map——该函数对入参图层查询一个"子图层枚举"接口（虚表 +0xc 取计数、+0x10 按 index 取项）并**递归加入子图层**，再经 `FUN_7940fe80`/`FUN_79410140` 完成其余登记（槽位语义未逐项定性）；数组 `+0x150`、计数 `+0x154`，尾部 int32 由 `FUN_794199f0` 设置活动图层。标志为其他值时读侧返回 `0x80004005`（汇编 `0x794036b2`：`CMP AX,1/2/3`，其余 `MOV EAX,0x80004005`）。

### 样本图层列表逐条目闭合（证据：确定的）

`/tmp/r3-maps.bin` 按上述结构完整解析（流内偏移）：

| 位置 | 字节/结构 | 内容 |
| --- | --- | --- |
| `0x274` | `06 00 00 00` | 图层计数 = 6 |
| `0x278` | 标志 `01 00` | 条目 1：`LoadObject` → `0x27a` FDOGraphicsLayer `{34B2EF83-…}` 键 31，体 `0x28e..0x2d22`；体 = `0b 00`（版本 11）+ `03 00 00 00` + `ff ff` + **内嵌 FeatureLayer 对象**（`0x296` 键 32，体 `0x2aa..0x2d22`）——注记层把其关联要素层作为体内嵌套对象持久化（候选的：FDOGraphicsLayer::Load 本体在 AnnoLayer.dll，本轮未反编译，字段语义按字节观察） |
| `0x2d22` | 标志 `02 00` + 前缀 `[14 00 00 00][00 00 00 00]` | 条目 2：`ExtraSafeReadObject` → `0x2d2c` FeatureLayer CLSID + 键 32 **回引用（A=20=仅头，无体）**，命中键 32 既有对象 |
| `0x2d40` | 标志 `02 00` + 前缀 `[72 15 00 00][0]` | 条目 3：FeatureLayer 键 158，体 `0x2d5e..0x42bc`（A=5490=头 20+体 5470） |
| `0x42bc` | 标志 `02 00` + 前缀 `[f6 0c 00 00][0]` | 条目 4：FeatureLayer 键 218，体 `0x42da..0x4fbc`（A=3324） |
| `0x4fbc` | 标志 `02 00` + 前缀 `[dd f5 00 00][0]` | 条目 5：RasterLayer `{D02371C9-…}` 键 259，体约 `0x4fd6..0x145a3`（A=62941） |
| `0x145a3` | 标志 `02 00` + 前缀 `[09 2a 00 00][0]` | 条目 6：RasterLayer 键 1544，体 `0x145c1..0x16fb4`（A=10761） |
| `0x16fb4` | `ff ff ff ff` | 活动图层索引 = -1（无活动图层） |
| `0x16fba` | CompositeGraphicsLayer `{9646BB83-…}` 键 1553 | `LoadObject(DAT_79487fe0={34B2EF81-…})` 装入的 **+0xf8 基本图形层**，体 `0x16fce..` |

条目体积 A 与下一标志位置的差值逐条吻合（A = 16 字节 CLSID + 4 字节键 + 体），条目 2 的 A=20 且无体直接证明"回引用条目只写头"。由此第三轮的"图形/注记层集合候选"升级：**该结构即 Map 的图层列表（含绘制顺序）+ 活动图层索引**，图层树由「GroupLayer 子图层循环 + 注记层内嵌要素层 + 基本图形层子图层循环」三者构成。

### CompositeGraphicsLayer::Load（0x6f6b2100，证据：确定的）

FeatureLayer.dll `0x6f6b2100`（RTTI `.?AVCompositeGraphicsLayer@@` → COL `0x6f7b4624` → 虚表 `0x6f7aa318` +0x14；`ExtraSafeReadObject` 的 FeatureLayer.dll 唯一调用点 `0x6f7113da` 属 GroupLayer::Load，反证此函数走普通 `LoadObject`）：short 版本（>6 拒绝）→ **int32 元素组计数 + 每组[int32 元素计数 + 循环 `LoadObject(DAT_6f7aa4fc)`]**（元素对象入组，`DAT_6f7aa4fc` 字节未解码）→ `String::Read(+0x20)`（图形层名称）→ short ×3（+0x34/+0x36/+0x38）→ double ×2（+0x24/+0x2c）→ long/int32 → `LoadObject(DAT_6f7aa58c)` → 版本>1 时 double + int32 → **int32 子图层计数 + 每项[String::Read + `LoadObject(DAT_6f7aa4ec={34C20002-…} ILayer)`]**（子图层树）→ 尾段未逐项展开。

样本对照（`0x16fce` 起）：版本 6 ✓；元素组计数 11、11 组元素计数全 0（44 字节零）✓；名称 `<默认> `（长度 10，0x17000）✓；后续 short/double 与函数序一致；子图层计数 = 1 @`0x17052`，子图层 = String「MZD1 」（`0x1705e`）+ FeatureLayer 键 1555 @`0x1706e`（名称「MZD1 」，即注记组对应的要素层）。其后 FeatureLayer 键 1619..2101（`0x187d2..0x23508`）的归属（候选：键 1555 体内 `DAT_6f7ac2a4={3036D35D-…}` 计数循环，计数候选 50 @`0x187ce`）未闭合（开放）。

### fe3115e9 / ee53528b 两循环的跨 DLL 成员扫描（证据：确定的；语义仍开放）

对 Map/FeatureLayer/Layer/RasterLayer/AnnoLayer/LabelPlacement/GdbCore/AfCore 八个 10.1.0.3035 文件做 16 字节原样扫描：`{FE3115E9-…}` 仅存在于 Map.dll（2 处，即 `DAT_79481a84` 及另一处）、`{EE53528B-…}` 仅存在于 Map.dll（1 处 = `DAT_794815a0`）、`{3036D35F-…}` 仅存在于 Map.dll（2 处）。这些请求 IID 的实现类不在上述 8 个模块的数据段中（LabelPlacement.dll 含 `ee535289` 1 处，不含 `ee53528b`），按「接口常量归属实现 DLL」的惯例，两循环的元素类语义仍开放；样本中两循环的流内定位（CGL 体尾之后、WKSEnvelope 之前与之后）未逐字节闭合，计数是否为 0 未验证。

### FeatureLayer::Load 中后段补全（证据：确定的；接第三轮 IID 表）

`0x6f6e9940` 字段序（接头部段）：数据源段 = `FUN_6f6e94e0`（内含 `LoadObject(DAT_6f7aaf28={4BA33331-…} IDatasetName 候选)` 装入 FeatureClassName 至 `+0x290` 并对名称做 **`ConvertToFullPath` 相对路径展开**）→ `LoadObject(DAT_6f7a98cc={40A9E884-…} IName 候选)` 装入工作空间名称对象 → **原样 16 字节 GUID → +0x144** → `String::Read(+0x154)` → double +0x158、double +0x160 → short +0x168 → **int32 计数 + 循环 `LoadObject(DAT_6f7ac2a4={3036D35D-…})`** → `LoadObject(DAT_6f7aaf98={20CD40B0-…})` → String +0x1a8、String +0x184、long +0x188 → 后续为渲染器/标注/字段信息等大量版本分支（`featurelayer_load.c` 162 行以后，未逐项展开）。

样本首 FeatureLayer（键 32）对位：FeatureClassName 体 `0x30a..0x370` = `02 00` + String（长度 32，`0x30e`，字节与图层名同文）+ 0x330..0x370 字段；WorkspaceName 体 `0x384..0x3d6` = `01 00`（版本 1）+ String「..\22.界址点」（长度 20，`0x386`，敏感路径公开前脱敏）+ int32 `2`（`0x39e`）+ String「Shapefile Data」（长度 30，`0x3a4`）+ 16 字节零；**+0x144 原样 GUID = `{6ADB9601-5CA0-11D1-AA81-00C04FA33A15}`（`0x3d6`）**——与 WorkspaceName CLSID `5A350011-…AA82` 同族（`aa81` 对 `aa82`），候选为数据源类型/工厂标识，语义未定性；其后 `15 00 00 00`（`0x3e6`）与 SimpleRenderer 键 35 @`0x3ed`（体首版本 3，内嵌 SimpleMarkerSymbol 键 36 @`0x403`）衔接。GdbCore 反编译 `0x709a9c90`（short≤3 + int32 + String + String + int32 + int32 + 版本分支 String ×2）与 709a9000（short≤2 + int32 长度 + 原样字节块 + `LoadObject(DAT_70ac9080)` + ver>1 String/int32）分别候选为 WorkspaceName/DatasetName 家族的 Load，但字段序与样本 WorkspaceName 体（ver=1 无首个 int32）不完全对位，归属开放。

## 证据与复现

- 证据包：`temp/arcgis/arcmap101-map-object-evidence.json`（文件身份、关键地址、PageLayout 类表全 21 条、esri.clsid.xml 五模块清单、样本 GUID 归属 67 项、Map Load 虚表候选、元数据）；2026-11-14 第三轮新增 `temp/arcgis/arcmap101-map-cr-extent-evidence.json`（Map.dll/FeatureLayer.dll 请求 IID 字节、FUN_794035f0/794038a0 反编译、50 条记录）。
- 关键反编译文本（会话临时目录 `/tmp/r3/`，未入库）：`map_load_real.c`（Map::Load 0x793ff4d0）、`map_load_fresh.c`（同函数 2026-11-14 复核副本）、`grouplayer_load.c`、`featurelayer_load.c`、`init_decompile.c`（PageLayout 填表 0x7b71a350）、`f7b708d00.c`（/Maps 根入口）、`r4_map_funcs/f794035f0.c`（图层列表读侧）、`annolayer_work/cgl_load_6f6b2100.c`（CompositeGraphicsLayer::Load）、`afcore_extrasafe/f68a7be60.c`、`f68a7bd70.c`（ExtraSafeRead/WriteObject）。
- 复现要点：`mxd2qgz extract temp/测试底图.mxd Maps /tmp/r3-maps.bin` 后按上文偏移核对；Ghidra 侧按"RTTI TypeDescriptor → COL(+12 引用 TD) → 虚表−4 引用 COL → IPersistStream 槽 +0x14"定位任意 Esri 类的 Load。

## 未解决事项

1. Map::Load 第 5 步计数序列（IID `{FE3115E9…}`，2026-11-14 解码）与第 9 步循环（IID `{EE53528B…}`，集合 QI `{3036D35F…}`，2026-11-14 第四轮字节修正）的元素类语义未逐对象定性；第 10 步嵌套头（ObjectStream +0x44）之后的对象身份、第 12 步两个计数循环与第 13 步版本分支字段的语义未展开。样本仅 1 个数据框，无法用字节区分第 5/9 步。
2. GroupLayer 三个 short（+0xa0/+0xa2/+0xa4）与三个 short（≥9 段 +0xf2/+0xf4 等）语义未定性；可见性结论锚定 FeatureLayer（+0x128）与 GroupLayer（+0xbc，版本分支明确）；参考样本无组图层，GroupLayer 版本分支无字节佐证。
3. FeatureLayer +0x144 的 16 字节原样 GUID 语义未定性；FeatureLayer 第 5 步之后的渲染/标注字段未展开。
4. `ExtraSafeReadObject`（GroupLayer ≥11 分支）与 `LoadObject` 的差异未展开；Map/图层各类的 Save 方向未按类分析。（2026-11-14 第四轮更新：两者差异已闭合——ExtraSafeReadObject 多读 8 字节 `[A][B]` 前缀、对象体布局不变；Map::Save 主体顺序与图层列表写侧已闭合，见第四轮小节；各图层类自己的 Save 未按类展开。）
5. MxTools.dll 263/264 条类表的逐条 CLSID 清单未从二进制恢复（全局构造填充，Ghidra 未恢复填表代码）；现以 esri.clsid.xml 声明为准，已用 PageLayout 21/21 全量比对与条目数 263↔264 佐证。
6. esri.clsid.xml 的"模块名"与实际工厂宿主存在差异（Map 声明 Map.dll 但 MappingCoreLib/PageLayout 等亦含相关字节）；清单使用时以"类身份 = CLSID+名称"为准，模块归属仅作线索。
7. `Mx Document` 新格式分支（`0x7b708d00` 前段）未逐行复核。
8. /Version 尾部计数 2267 与 /Maps 对象图对象数的对应关系未建立。
9. 全部结论为静态反编译观察；ArcMap 10.1 与 QGIS 的人工打开与目视核对均未进行，不声称已验证。
10. （2026-11-14）`FUN_794035f0`/`FUN_794038a0` 两个图形/注记层集合的元素类语义、short 1/2/3 分支语义与尾部索引含义未定性；`DAT_79485a18`=`{34C20002-…}`、`DAT_79487fe0`=`{34B2EF81-…}`、`DAT_79481580`=`{1B077914-…}` 等请求 IID 的接口身份未确认。
11. （2026-11-14）WKSEnvelope 32 字节（显示范围）在样本中的流内定位为候选（`0x24f64`），首个 FeatureLayer 体之外的对象边界未逐段闭合；`WKSEnvelope` 4 double 的 `[XMin,YMin,XMax,YMax]` 序依据 Envelope 同布局证据，未从 Map.dll 写出方向直接取证。
12. （2026-11-14）Envelope 对象体布局为样本侧观察（25/25 一致），Geometry.dll 的 Envelope Load/Save 本体未反编译；第 4 段后的 int32 0 语义未定性。两个 Envelope 由 FeatureLayer::Load 的哪个 LoadObject 装入未闭合；FeatureLayer 头部段（反编译）与样本中段（渲染器/标注对象序列）的逐字节对位未完成。
13. （2026-11-14 第四轮）`{FE3115E9-…}`/`{EE53528B-…}` 两循环（及集合 QI `{3036D35F-…}`）的实现类不在已扫描的 8 个 10.1 DLL 数据段中，元素类语义开放；两循环在样本中的流内定位与计数（疑似 0）未逐字节闭合。FeatureLayer 键 1619..2101 簇的归属（候选：键 1555 体内 `3036D35D` 计数循环，计数候选 50 @`0x187ce`）未闭合。
14. （2026-11-14 第四轮）图层列表写侧标志 1（SaveObject 路径）的触发状态未逐分支定性；`FUN_79410040` 子图层枚举接口与 `FUN_7940fe80/79410140` 登记的槽位语义未逐项定性；FDOGraphicsLayer::Load 本体（AnnoLayer.dll）未反编译，其体内字段（`0b 00 / 03 00 00 00 / ff ff`）语义按字节观察。Map::Save 尾段（transformations 循环之后）未逐项展开。`+0x144` GUID `{6ADB9601-…}` 语义未定性。

## MapSurround/书签循环接口身份、注记子图层簇修正与样本尾部（2026-11-14 第五轮）

对应 TODO 开放的「`FE3115E9`/`EE53528B` 两循环与键 1619..2101 簇归属」及 /Maps 尾部值语义。
证据分三路：① 10.1 类型库名称解析沿用「Version 流对象接口 GUID 语义考证」节的 wine
`LoadTypeLibEx` dump（`/tmp/mxd-arcgis101-guiddocs/wine/dump*.txt`，全部 79 个 .olb，本轮未重新
解析，仅检索既有输出）；② 完整类表 `esri.clsid.xml`（`/tmp/r3/ecfg/x/`，10608 条 progID）用于
样本字节逐头归名；③ 本机 pyghidra（Ghidra 12.2 + JDK 25）对 ArcMap.exe 保存函数写点的局部
反汇编（`/tmp/r5/arcmap_maps_region.txt`，项目 `/tmp/r5/ghproj_arcmap`，未入库）。样本
`/tmp/r3-maps.bin`（SHA-256 `9852c0e2…8129b49`）偏移为 /Maps 流内偏移。全部为静态观察，
未运行 ArcMap，未人工验证。类型库名称↔GUID 绑定为文件内静态数据（与上轮同置信度：确认），
「接口在某字段中的语义角色」为反编译+字节互证，逐条标注。

### 开放接口身份一次闭合（类型库 typekind=3，证据：确认（类型库）；字段角色：反编译+样本）

| GUID | 类型库名称 | 在字段序中的角色（标注） |
| --- | --- | --- |
| `FE3115E9-ED6C-11D0-8683-0000F8751720` | **IMapSurround** | Map::Load 第 5 步循环请求 IID：地图装饰物（图例/比例尺/指北针等）集合；Save 侧 `+0x130/+0x12c` 数组（第四轮）即 MapSurround 列表 |
| `EE53528B-41C9-11D1-880A-080009EC732A` | **ISpatialBookmark** | 第 9 步循环请求 IID：空间书签集合（`IMapBookmarks`=`EE53528F`） |
| `3036D35F-EDE5-11D0-87FE-080009EC732A` | **IFeatureBookmark** | 第 9 步逐项 QI/入集（`FUN_7941c760`+`DAT_79488d18`）的目标接口 |
| `3036D35D-EDE5-11D0-87FE-080009EC732A` | **IHyperlink** | FeatureLayer::Load 计数循环（第三轮 IID 表 `DAT_6f7ac2a4`） |
| `1B077914-38C5-11D0-92D2-00805F7C28B0` | **IEnvelope**（非 IGeometry） | Map::Load 第 8 步与 FeatureLayer ver≥8/≥0xe 的 LoadObject；样本内对应 Envelope 类对象头 |
| `FFC6B179-E3EC-11D3-A096-00C04F6BC626` | **IStandaloneTable** | 第 12 步 ≥0xb 计数循环（`DAT_79488300`）：独立表集合（`ITableCollection`=`7D931EAB`） |
| `9273BB60-C611-11D2-BD08-0000F875BCCE` | **IGeoTransformation** | 第 13 步链 0xc/0xd/>0xd 循环（`DAT_79487e40`）：坐标转换对象 |
| `1C352F3F-298E-11D3-9F4F-00C04F6BC619` | **IIlluminationProps** | 尾部 ver≥4 LoadObject（`DAT_79487e60`）；样本流内对象 coclass 为 `{1C352F40-298E-11D3-9F4F-00C04F6BC619}`（esri.clsid 名 IlluminationProps），键 2178 @`0x24fb2` |
| `8C439003-14EC-11D2-A27E-080009B6F22B` | **IAnnotateMap** | ver 0x13..0x16 与 ≥0x13 LoadObject（`DAT_79488470`）：标注引擎对象 |
| `4BA33331-D55F-11D1-8882-0000F877762D` | **IName** | ver<0xb 计数循环（`DAT_79482c08`）与 FeatureLayer 数据源段（`DAT_6f7aaf28`），修正第三轮「IDatasetName 候选」 |
| `40A9E884-5533-11D0-98BE-00805F7CED21` | **IFeatureRenderer** | FeatureLayer 数据源段后 LoadObject（`DAT_6f7a98cc`），修正「IName 候选」 |
| `40A9E885-5533-11D0-98BE-00805F7CED21` | **IFeatureLayer** | Map ≥0x22 分支（`DAT_79487fa0`） |
| `E4240307-DD41-11D2-B868-00600802E603` | **IBorder** | 函数尾 LoadObject（`DAT_79487fc0`） |
| `E4240309-DD41-11D2-B868-00600802E603` | **IBackground** | >0xf 分支（`DAT_79487fd0`） |
| `20CD40B0-8F3D-11D0-8590-0800091A2A72` | **IColor** | FeatureLayer 计数循环后 LoadObject（`DAT_6f7aaf98`） |
| `311C2EE2-DA65-11D2-8A54-000000000000` | **IRelationshipClassName** | FeatureLayer ver>6 计数循环（`DAT_6f7adc24`） |
| `19F85377-1387-11D3-B89C-00600802E603` | **IDisplayFilter** | FeatureLayer ver≥2 |
| `A62B9EC2-5C52-4B98-B116-5EA341D4AF6D` | **IDisplayExpressionProperties** | FeatureLayer ver≥0x19；样本内实现类 DisplayExpressionProperties（esri.clsid 名） |
| `F6CBA4A3-BC48-47BC-9F15-2A561B9E6C71` | **ITimeReference** | FeatureLayer ver≥0x1d |
| `BD724B95-018F-4367-9883-9560D92293A7` | **ITimeExtent** | FeatureLayer ver≥0x1e |
| `E720618B-99A9-46D7-AA6B-507898E1EA90` | **IHotLinkExpressionProperties** | FeatureLayer ver==0x1f 与 >0x20 |
| `34C20002-4D3C-11D0-92D8-00805F7C28B0` | **ILayer** | `DAT_79485a18`/CGL 子图层循环，候选转确认 |

第 5 步＝MapSurround 集合循环、第 9 步＝空间书签集合循环（含 IFeatureBookmark 特化入集）
由此闭合到接口层；**两循环元素的具体 coclass 由流内 16 字节类标识决定，样本未见非空实例**
（样本定位仍开放，见下），元素类语义的最终定性需含书签/装饰物的样本。

### 键 1619..2101 簇归属修正（证据：确定的；修正第四轮两处误读）

- **「计数候选 50 @`0x187ce`」系误读**：该 4 字节是注记类名字符串 `"JZD1 注记 2"` 的 UTF-16
  字符 `'2'`（`32 00`）+ 终止符（`00 00`），非循环计数。
- 簇实为 **CGL-1553（`0x16fba`）子图层循环的注记类条目**，条目模式＝
  `[String 注记类名][FeatureLayer 完整体]`：字符串形如 `"JZD1 注记 2"`…`"JZD1 注记 10"`
  （String::Read 布局，cb=20/22 等可变），紧跟 FeatureLayer 对象头。观测 9 个 FeatureLayer 头：
  键 1555（`0x1706e`，首条目，名 `"JZD1"`）、1619（`0x187d2`）、1676、1734、1792、1875、1953、
  2024、2101（`0x23508`），键距 57–58；另有一个对键 1792 的 **FeatureLayer 回引头** @`0x1dcba`
  （20 字节无体，位于某元素体内，属写侧「同对象复引」实例）。
- 每单元体构成（逐头归名，esri.clsid.xml 全表）：FeatureClassName+WorkspaceName（数据源）、
  SimpleRenderer+符号族（MultiLayerMarkerSymbol `7914E5FF`、CharacterMarkerSymbol `7914E600`、
  SimpleFill/SimpleLine/SimpleMarker、RgbColor）、OLE StdFont、LegendGroup+LegendClass（内含对
  既有符号键的回引）、FieldInfo 数组（含 NumericFormat 子对象）、
  AnnotateLayerPropertiesCollection→LabelEngineLayerProperties→TextSymbol→BasicOverposterLayerProperties
  →LineLabelPosition→LineLabelPlacementPriorities→PointPlacementPriorities→
  AnnotationVBScriptEngine→Envelope→FeatureIDSet→EditTemplate（`{B9357037-34C7-…}`，
  esri.clsid 名 EditTemplate，体内 SimpleMarkerSymbol/RgbColor）→Envelope（对 Map PCS 键 30 的
  回引或内嵌 PCS）→UID→DisplayExpressionProperties（或 ElementCollection `{CE8F3972}`）→
  ElementCollection 元素集合（TextElement+TextSymbol+SimpleTextPath 布局字节+Polyline 引线，
  Polyline=`{30707210-…}`，与 Envelope `{30707212-…}` 相邻异类， Polyline 体含点坐标 double 对，
  内含对 PCS 键 30 的回引）。
- 簇后段（`0x24fb2` 起）：IlluminationProps（`{1C352F40}`，键 2178）→ NADCONTransformation
  （键 2179 + 对 2179 的回引）→ BasicOverposterProperties（键 2180）→ RgbColor（2181）→
  Envelope（2182，PCS 回引 30）→ Envelope（2183，PCS 回引 30）→ 纯标量区（`0x25396..0x25569`，
  无任何对象头）→ 尾 4 字节。**这些对象与 Map::Load 后继步骤（第 5 步 MapSurround 计数、
  WKSEnvelope、第 8 步 IEnvelope、第 9 步书签等）的逐字节对位仍未闭合**；候选包括 CGL 体尾
  （`DAT_6f7aa48c`/`DAT_6f7aa4ec` 装载）与 Map::Load 第 8 步（Envelope-2182/2183 之一为
  IEnvelope 候选）。样本仅 1 个数据框、无书签/装饰物，第 5/9 步计数（候选 0）未验证。

### /Maps 尾部 4 字节（写侧指令级闭合）

ArcMap.exe 保存函数 `0x4699cd..0x4699f1`：`LEA EDX,[EBP-0x14]; PUSH EDX; PUSH EAX(流对象);
call [vtbl+0x54]` → `MOVSX ECX, word [EBP-0x14]` → `CMP ECX,-1` → `SETZ DL` →
`AfStreamWrite(流, &flag, 4)`（int 变体重载 IAT `0x5411d0`）。SystemCore `+0x54`
（`0x7ffd0550`）实际为 `EnsureCapacity(this, select_word, count)`：恒返 0、不写输出参数
（汇编 `/tmp/r5/sc_work/asm_report.json`）；故 `MOVSX` 读到的 `-1` 来自栈残留（静态推断）。
读侧消费已闭合：打开入口把 /Version 的 4 字节计数传 `+0x54(-1, v)` 作读表容量提示
（`max(17, v/10)` 桶），/Maps 尾部 4 字节同构。样本 /Maps 尾 4 字节 @`0x25569` = `01 00 00 00`。
详见 [对象体记录](arcmap-10.1-object-body.md#maps-尾部-4-字节指令级闭合对既有结论的精化)。

### 本轮方法与身份补充

- 本轮新提取 `Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 至
  `temp/arcgis101-bin/mapdll/`：1242112 字节，SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`
  （与第四轮记录一致），pefile 读 PE x86、ImageBase `0x793a0000`、VS_FIXEDFILEINFO
  `10.1.0.3035`。本轮 Map.dll 本体未新增反编译（两循环接口身份来自类型库、循环代码沿用
  `/tmp/r3/map_load_real.c` 既有产物）；后续 Map::Save 尾段展开可直接使用该文件。

## 未解决事项（第五轮新增）

1. 两循环在样本中的流内定位与计数：CGL 体尾、`0x25340` 后纯标量区（~467 字节）与
   Map::Load 第 5–13 步的对位未闭合；第 10 步嵌套对象（ObjectStream `+0x44`）在样本尾部
   未定位（未见 16 字节零头或类头）。
2. 第 5 步（MapSurround）与第 9 步（书签）元素的 coclass 集合：需含书签/装饰物的 MXD 样本
   或写方向取证；样本计数疑似 0 未验证。
3. CGL-1553 子图层条目的尾字段（`short×3+double×2+int×2+LoadObject+double`，CGL::Load
   `0x6f6b2100` 已有）与单元 FL 体结束边界的逐字节对位；子图层计数的准确值与流内位置
   （`0x17052` 值 1 与 `0x17056` 值 10 二解，未裁决）。
4. `0x24fb2` 起尾段对象（IlluminationProps/NADCON/BasicOverposter/Envelope×2）的宿主
   （CGL 尾 vs Map::Load 步骤）未定性；`DAT_6f7aa48c`/`DAT_6f7aa4ec` 请求 IID 的类型库名称
   未解码。
5. TextSymbol 体内 4 处 `{SimpleTextPath 布局}`（键位恒 0，无键）的持久化机制（非 ObjectStream
   键路径的内嵌对象候选）未展开。
6. 全部结论为静态观察；ArcMap 10.1 与 QGIS 的人工打开与目视核对均未进行，不声称已验证。
