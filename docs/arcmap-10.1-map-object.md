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

## 证据与复现

- 证据包：`temp/arcgis/arcmap101-map-object-evidence.json`（文件身份、关键地址、PageLayout 类表全 21 条、esri.clsid.xml 五模块清单、样本 GUID 归属 67 项、Map Load 虚表候选、元数据）。
- 关键反编译文本（会话临时目录 `/tmp/r3/`，未入库）：`map_load_real.c`（Map::Load 0x793ff4d0）、`grouplayer_load.c`、`featurelayer_load.c`、`init_decompile.c`（PageLayout 填表 0x7b71a350）、`f7b708d00.c`（/Maps 根入口）。
- 复现要点：`mxd2qgz extract temp/测试底图.mxd Maps /tmp/r3-maps.bin` 后按上文偏移核对；Ghidra 侧按"RTTI TypeDescriptor → COL(+12 引用 TD) → 虚表−4 引用 COL → IPersistStream 槽 +0x14"定位任意 Esri 类的 Load。

## 未解决事项

1. Map::Load 第 5 步计数序列（IID `DAT_79481a84`）与第 9 步循环（IID `DAT_794815a0`，加入集合）的元素类语义未逐对象定性；第 10 步嵌套头（ObjectStream +0x44）之后的对象身份、第 12 步两个计数循环与第 13 步版本分支字段的语义未展开。样本仅 1 个数据框，无法用字节区分第 5/9 步。
2. GroupLayer 三个 short（+0xa0/+0xa2/+0xa4）与三个 short（≥9 段 +0xf2/+0xf4 等）语义未定性；可见性结论锚定 FeatureLayer（+0x128）与 GroupLayer（+0xbc，版本分支明确）；参考样本无组图层，GroupLayer 版本分支无字节佐证。
3. FeatureLayer +0x144 的 16 字节原样 GUID 语义未定性；FeatureLayer 第 5 步之后的渲染/标注字段未展开。
4. `ExtraSafeReadObject`（GroupLayer ≥11 分支）与 `LoadObject` 的差异未展开；Map/图层各类的 Save 方向未按类分析。
5. MxTools.dll 263/264 条类表的逐条 CLSID 清单未从二进制恢复（全局构造填充，Ghidra 未恢复填表代码）；现以 esri.clsid.xml 声明为准，已用 PageLayout 21/21 全量比对与条目数 263↔264 佐证。
6. esri.clsid.xml 的"模块名"与实际工厂宿主存在差异（Map 声明 Map.dll 但 MappingCoreLib/PageLayout 等亦含相关字节）；清单使用时以"类身份 = CLSID+名称"为准，模块归属仅作线索。
7. `Mx Document` 新格式分支（`0x7b708d00` 前段）未逐行复核。
8. /Version 尾部计数 2267 与 /Maps 对象图对象数的对应关系未建立。
9. 全部结论为静态反编译观察；ArcMap 10.1 与 QGIS 的人工打开与目视核对均未进行，不声称已验证。
