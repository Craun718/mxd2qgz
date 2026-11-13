# ArcMap 10.1 文档打开、保存与 ObjectStream 调用链

记录日期：2026-11-14。研究目标：闭合 10.1 上「保存入口 → SaveObject 调用点」、
「MapDocument 类工厂与 Open 入口链」与「ArcMap 命令层 → PageLayout Open →
主读取器」的静态调用链，并对照 Git 历史 `c17b324` 的 10.8 文档
（mapdocument-native.md、object-context-native.md、maps-analysis.md）。
本文只记录静态反编译观察，未运行 ArcMap，未在 ArcMap 10.1 或 QGIS 中
人工验证任何读取或转换结果。

## 文件身份与 SHA-256 复核

2026-11-14 对本轮使用的三个研究文件执行 `sha256sum`，全部与
[基线文档](arcmap-10.1-native-baseline.md) 一致：

| 文件 | SHA-256 |
| --- | --- |
| ArcMap.exe.E97001B0_1E76_4BE8_B51F_269775FE0A80 | `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97` |
| PageLayout.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` |
| MxTools.dll.C13FAB27_D904_4317_AC52_8A308EFED288 | `d29677c8514e705c6585b1b5a27d4e5cf32f7326891ccfe6ed7beeb70ca51347` |

版本均为 10.1.0.3035（pefile 资源声明，见基线文档）。本轮 REA 6.9.0 CLI
（npm 缓存 `9b9809298fe40d2c`）+ Ghidra 12.2，`JAVA_HOME=/usr/lib/jvm/java-25-openjdk`；
MCP 注册的 `JAVA_HOME` 为 JDK 21 会被 Ghidra 12.2 拒绝，故全部经 CLI 运行。
地址为静态映像虚拟地址，绑定上表文件摘要，不是运行时地址。PDB 加载状态
未知；类名取自 Ghidra 类型信息或 Ghidra 恢复的符号名，不表示运行验证。
SystemCore.dll 侧结论（ObjectStream 虚表 `0x800710dc` 槽位、读分支
`0x7ffcff70`、SetStream `0x7ffcf7a0`、保存侧辅助 `0x7ffd0370`）只引用
[对象体文档](arcmap-10.1-object-body.md)，本轮未重复分析 SystemCore。

## 任务 1：保存入口 `0x466fb0` 的 SaveObject 调用点（ArcMap.exe）

- AfCore 导入：ArcMap.exe IAT 中 `?SaveObject@@YAJPAUIUnknown@@PAUIStream@@H@Z`
  位于 `0x541164`（hint 1843），`?LoadObject@@YAJPAPAUIUnknown@@ABU_GUID@@PAUIStream@@PAU1@@Z`
  位于 `0x541128`（hint 1544）；两者的修饰名以字符串形式存在于映像
  （`0x575890`/`0x575b16`）。即 SaveObject/LoadObject 是 AfCore.dll 的
  跨模块辅助导出，按符号名直接反编译 Ghidra 返回空（它们是导入而非内部函数）。
- 保存入口 `0x466fb0`（[对象体文档](arcmap-10.1-object-body.md) 已给出写
  `/Maps` 的顺序）内部共有 **10 处直接调用** SaveObject IAT：
  `0x468822`、`0x4689e9`、`0x468b6d`、`0x468989`、`0x469c9c`、`0x46a0fd`、
  `0x46a792`、`0x46aece`、`0x46bc5b`、`0x46c54a`（调用关系
  `ev_6ee6deb9013afeddad7d0b82442ae504898d6c91bd8335d6af582a95b31a4e6f`）。
- 首个调用点（对应 `/Maps` 第一个 Map）的指令序列
  （`ev_` 见证据包 `r2-am-ins-468822.json`）：
  `0x4687d1` 取 `[文档+0x1a0]`，`+0x180` 得 Map 对象指针；`0x4687f8`
  先对该 Map 经虚表槽 `+0x1c` 做一次虚调用；`0x468812` 压常量 `1`
  （SaveObject 第三参数）；`0x468822 CALL [0x541164]`；`TEST EAX; JGE`
  按符号判断成功。参数顺序可见为 `(Map 对象, ObjectStream, 1)`。
- **结论**：ArcMap 侧可见的只是调用签名与次数——GUID + 键的对象头写出
  发生在 AfCore `SaveObject` 内部（即被调方），10 处调用分别对应保存序列
  中的不同对象（Maps 循环、DrawingDefaults、模板等）。写侧顺序
  （数量 → 循环 SaveObject → 尾部 4 字节）由 [对象体文档](arcmap-10.1-object-body.md)
  的调用序列证实；GUID/键由谁经哪个虚表槽写出仍未闭合（AfCore 内部，
  与 10.8「对象写出器未闭合」同状态）。

## 任务 2：PageLayout MapDocument 类工厂与 Open 入口链

### DllGetClassObject 与运行期类表

- PageLayout `DllGetClassObject` `0x7b698800` 跳入 `FUN_7b702cf0`
  （`ev_b7724841...f4933`，完整 ID 见证据包）：遍历类表 `DAT_7b733860`
  （表项步长 0x34），经 `FUN_7b6937a0` 做 16 字节 GUID 比较，命中后调用
  工厂虚表槽 0；未命中返回 `0x80040111`（CLASS_E_CLASSNOTAVAILABLE）。
  静态映像中 `0x7b733860` 读出全零——类表在运行期由 `FUN_7b71a350`
  构建（`ev_7c200ea9...1df8b7`），共 0x16（22）项。
- `FUN_7b71a350` 的关键表项（名称为 Ghidra 恢复的字面量）：
  - 项 0：CLSID `dd94d76e-836d-11d0-87ec-080009ec732a`，`PageLayout`
  - 项 1：`dd94d76f-836d-11d0-87ec-080009ec732a`，`Page`
  - 项 2：`0e66a40e-9087-11d3-9f50-00c04f6bc979`，`PageEventsHelper`
  - 项 3：`31e081ac-cb02-11d1-876c-0000f8751720`，`RulerSettings`
  - 项 6：CLSID `2fe5e68e-3da5-4bda-baf1-103c55115a1c`（字节核对
    `0x7b71d978`），名称 `MapDocument`，工厂为 ClassFactory<MapDocument>
  - 项 0x10：`b6c6ed97-4064-4bbe-84a0-11dd03c697b6`，`WebMapDocument`
  - 析构 `FUN_7b71b730`

### MapDocument 工厂、构造与 QI

- `CreateInstance` `FUN_7b69a540`：聚合请求（riid == IID_IUnknown，
  常量 `DAT_7b71c76c`）返回 `0x80040110` 不支持；非聚合 `new(100)` 字节，
  调构造 `FUN_7b69ecb0`，再经 `FUN_7b69a665` 完成；分配失败 `0x8007000e`。
- 构造 `FUN_7b69ecb0`：基类虚表写入后，`+0x10` = IMapDocument 虚表，
  依次 `+0x14` IDocumentVersion、`+0x18` IDocumentInfo、`+0x1c`
  IDocumentInfo2、`+0x20` IDocInfoHelper、`+0x24` IDocumentPreview
  （名称来自 Ghidra 虚表符号）；`+0x28..+0x4d` 清零，`+0x60 = 5`。
- QI `FUN_7b69fe70`：IMapDocument IID（`0x7b71df7c`）→ 子对象 `+0x10`；
  另两个 IID → `+5`/`+6` 子接口。
- IMapDocument 虚表 `0x7b71dd7c` 共 16 槽已列出（证据包
  `pl-imapdoc-vtable.json`）。与本轮相关的槽位：
  - `+0x0c` = `0x7b6a0350`：按 `FUN_7b69f500` 分类扩展名——`.MXD`/`.MXT`
    → 0（同一路径）、`.PMF` → 1、`.LYR` → 2，其余（含 `.LPK`/`.MPK`）
    → -1；随后分别进入 `FUN_7b7086c0`（0）/`FUN_7b70f250`（1）/
    `FUN_7b7068d0`（2）。
  - `+0x30` = `0x7b6a01c0`：状态检查（失败 `0x8000000a`）后按扩展名分类
    调 `FUN_7b7081d0`（MXD 路径）等，输出布尔结果——「是否可打开」类语义。
  - `+0x38` = `0x7b6a05e0`：状态检查后返回对象 `+0x28` 的内部指针
    （AddRef），数据引用 `0x7b71ddb4`——对应文档打开后取文档对象。
  - `+0x24` = `0x7b6a1220`：大函数，按 `FUN_7b69f010` 状态并使用
    Variant 属性 `+0x28`，本轮只做局部反编译，未逐分支闭合。

### Open 链闭合：`IMapDocument::Open → … → 0x7b708d00`

MXD 路径（分类结果 0）的两条可静态闭合的链：

- 链 A（`+0x30`，0x7b6a01c0 → `FUN_7b7081d0` → `FUN_7b708080` →
  `0x7b708d00`）：`FUN_7b7081d0`（调用点 `0x7b6a021b`）做
  `StgOpenStorage`（或复用已有 IStorage）后调 `FUN_7b708080`；
  `FUN_7b708080`（调用点 `0x7b708128`）以 5 个智能指针包装调用
  **`0x7b708d00`**——即 [对象体文档](arcmap-10.1-object-body.md) 记录的
  文档打开主读取器（Mx Document 老格式 → DrawingDefaults → Maps →
  PageLayout/View）。
- 链 B（`+0x38`，0x7b6a05e0 → `FUN_7b69f6b0` → `0x7b7098b0` →
  `0x7b708d00`）：`FUN_7b69f6b0`（调用点 `0x7b69f814`；其静态调用者含
  `0x7b6a0680`、`0x7b6a07d0`、`0x7b6a3b60` 等多个接口实现）在分支中调用
  **`0x7b7098b0`**；`0x7b7098b0`（`ev_764272fc...ace9c7f`）是
  StgOpenStorage 包装，尾部（`0x7b7099eb`）直接调用 `0x7b708d00`。
  `0x7b7098b0` 的角色因此是「打开存储并转入主读取器」的前置包装，
  与链 A 的 `FUN_7b708080` 汇合于同一读取器。
- [基线文档](arcmap-10.1-native-baseline.md) 已记录的 Version 局部读取
  `0x7b70a8e0` 由 `0x7b69f6b0` 调用——即链 B 在进入主读取器前也做
  Version 前缀读取（输出 3/5 决定后续虚调用），与 10.8 后端
  `0x10076ef0` 的位置对应。
- `FUN_7b7086c0`（`+0x0c` 的 MXD 后端，`r2-pl-7086c0.json`）：StgOpenStorage
  → 按常量 `DAT_7b71df1c`/`DAT_7b71cf54`（ObjectStream CLSID
  `043731d0-a7cf-11d1-8bd1-080009ee4e41`）创建对象 → OpenStream
  `Mx Document`，失败则 OpenStream `Maps` → SetStream（`+0x38`）→
  `FUN_7b6a7370` → 另两个流（`FUN_7b707aa0`）→ 读 2 字节 short，
  `<3` 返回 1。这是「探测/预读」形态的后端，与链 A/B 的完整读取互补，
  具体分工未逐分支闭合。

## 任务 3：ArcMap 命令层到 PageLayout 的打开链

- OpenDocument 命令处理 `FUN_00456c20`：消息映射数据 `0x54a8b4` →
  包装 `FUN_00456c00` → `FUN_00456c20`；另一静态调用者 `0x4599b0`
  （调用点 `0x459d70`）。函数内 `.WebMap` 后缀分支走 `FUN_00406a40`，
  其余路径为 MFC 文档打开：`FUN_00410740`、`.mxd`/`.pmf` 后缀
  `CompareNoCase`、SetDocType 与 AfApp 锁
  （`r2-am-456c20.json`）。
- `.WebMap` 分支 `FUN_00406a40`（`ev_f1a9d8af...e41cfe`）：
  `AfCreateInstance(&DAT_005555c8, 0, 0x17, &DAT_005555f8, &局部)`
  → `OleRun` → `QI(&DAT_00555788)`。字节解码：
  `0x5555c8` = `2fe5e68e-3da5-4bda-baf1-103c55115a1c`——正是
  PageLayout 类表项 6 的 MapDocument CLSID；`0x555778`/`0x5555f8` =
  `dd94d76b-836d-11d0-87ec-080009ec732a`；`0x555788` = IID_IUnknown。
  **跨模块 COM 链闭合**：ArcMap 进程内以 CLSID 创建 PageLayout 的
  MapDocument 实例并运行。
- MFC 文档路径：文档类虚表 `0x54c294` 槽 `+0x08` = `FUN_0047a740`
  （CArchive 形态的 MFC Serialize/OnOpenDocument 结构）→ 调
  **`0x4764d0`**（[基线文档](arcmap-10.1-native-baseline.md) 的 Version
  头部读取入口；本轮确认 `0x47a740` 是其唯一静态调用者，
  `ev_efbe9ec0...b0174d`）。`0x4764d0` 内部约 15 处直接调用
  LoadObject IAT（`0x541128`），调用点
  `0x476ef8..0x47985d`，各前置不同 GUID 常量
  （`0x5554b8/c8/d8/e8`、`0x555568/578`、`0x555978`、`0x555aa8`、
  `0x555cf8`、`0x554e3c`、`0x554efc`、`0x554898`、`0x555e78` 等）；
  另调 `0x472ac0`（Templates）、`0x47a3f0`（构建号/版本判定）、
  错误辅助 `0x47a3cb`，尾部经 `0x4871b0` → `FUN_004798a8`。
- `FUN_004798a8`（`ev_8f901465...`，[对象体文档](arcmap-10.1-object-body.md)
  已引）：按数量循环 `LoadObject((IUnknown**)(局部), &DAT_00555e78,
  流, 0)`（`0x555e78` = `37c4c49c-66cc-4b39-ade8-15e2e270d8b1`），
  成功经 `FUN_004871b0` 处理，随后打开 `DataWindows` 流并 SetStream。
  即 ArcMap 侧的 LoadObject 循环承担 DataWindows 一类对象的重建，
  与 PageLayout 的 `0x7b708d00`（Maps/PageLayout/View）互补。
- MxTools 定位：`FUN_100e4380` 对文档对象字段 `+0x210/+0x214/+0x218/+0x220`
  做惰性初始化；字段 `+0x220 == 0` 时依次调 `FUN_100a8e30` 与
  **`FUN_100e6630`**（[基线文档](arcmap-10.1-native-baseline.md) 记录的
  DocumentInfo 读取函数；唯一静态调用者即 `0x100e4380`，调用点
  `0x100e43f8`，`ev_ec1bf500...702cc7`）。其位置是文档打开后的
  DocumentInfo/元数据惰性装配，不参与 Version 前缀读取。

**闭合结论**：ArcMap 打开 .mxd 的静态链为——命令层 `0x456c20` →
MFC 文档打开（`0x47a740` → `0x4764d0` Version 头 → LoadObject 系列 →
`0x4798a8` DataWindows 循环）＋（WebMap/COM 路径）`AfCreateInstance`
MapDocument CLSID → PageLayout `DllGetClassObject` → 类表项 6 →
ClassFactory → CreateInstance → 构造 → IMapDocument（`+0x10`）→
`+0x30`/`+0x38` 链 → `0x7b708d00` 主读取器。

## 与 10.8 对照

10.8 基线取自 `git show c17b324:docs/mapdocument-native.md`、
object-context-native.md、maps-analysis.md。逐项对照：

| 项目 | 10.8 | 10.1（本文） | 对照 |
| --- | --- | --- | --- |
| MapDocument CLSID | `2FE5E68E-3DA5-4BDA-BAF1-103C55115A1C`（esri.clsid.ecfg 注册清单） | 同一 CLSID 在 PageLayout 类表项 6（`0x7b71d978` 字节核对），且被 ArcMap `AfCreateInstance` 直接使用 | 相同；10.1 补上「注册清单之外的实现侧证据」 |
| 类表机制 | 全局初始化 `0x100015db`/`0x10001000` 关联 CLSID、名称与工厂（静态 .data） | 运行期由 `FUN_7b71a350` 构建表 `DAT_7b733860`（静态映像全零），22 项、步长 0x34 | **机制不同**：10.8 静态表，10.1 运行期构建；轮询结构一致 |
| DllGetClassObject | （未在该文档展开） | `0x7b698800` → `FUN_7b702cf0` 遍历类表，未命中 `0x80040111` | **10.1 新证据** |
| 工厂 CreateInstance | `0x1000cb00`：new(100) → 构造 `0x10010cc0` | `FUN_7b69a540`：new(100) → 构造 `FUN_7b69ecb0`；聚合请求拒绝 | 结构相同（同为 100 字节对象）；10.1 明确拒绝聚合 |
| IMapDocument 子对象位置 | 对象 `+0x10` | 对象 `+0x10` | 相同 |
| Open 虚表槽位 | IMapDocument 第 9 槽（`+0x24`，托管元数据方法顺序） | 10.1 IMapDocument 虚表 16 槽已列；`+0x24` = `0x7b6a1220`（大函数，未逐分支闭合）；MXD 完整打开链闭合在 `+0x30`/`+0x38` 两条 | **差异/开放**：10.1 侧 Open 槽归属未按托管元数据复核，本文以静态链闭合为准 |
| Open → 后端选择 | Open `0x10012ed0` 按 `0x1000f4d0` 分类，结果 0 → 后端 `0x10075190`/`0x10077160` | `FUN_7b69f500` 分类：MXD/MXT→0、PMF→1、LYR→2；0 → `FUN_7b7086c0`（预读形态）或链 A/B → `0x7b708d00` | 结构对应；10.1 给出完整扩展名表 |
| 主读取器 | `0x10075370`（Mx Document 失败 → DrawingDefaults → Maps） | `0x7b708d00`，同一顺序（[对象体文档](arcmap-10.1-object-body.md)） | 相同 |
| Version 局部读取 | 后端 `0x10076ef0`（`<0x12` → 3，初值 `0xb`） | `0x7b70a8e0`（`<0x12` → 3，初值 5；见[基线文档](arcmap-10.1-native-baseline.md)）；位于链 B 前置 | 位置对应，默认值不同（5 vs 0xb） |
| ArcMap 命令层 | 未研究（10.8 轮只到 PageLayout） | `0x456c20` 命令处理 + MFC `0x47a740` → `0x4764d0` → LoadObject×15 → `0x4798a8`；WebMap 分支 `AfCreateInstance` MapDocument | **10.1 新证据**：补齐 exe 侧调用链 |
| SaveObject 调用点 | 未闭合（对象写出器未闭合） | ArcMap `0x466fb0` 内 10 处 SaveObject IAT 调用；GUID/键写出仍在 AfCore 内部 | 10.1 补调用点清单，写出器仍双版本开放 |
| LoadObject 使用 | 未记 | `0x4764d0` 15 处（含各对象 GUID 常量）、`0x4798a8` 循环（GUID `37c4c49c-...`） | **10.1 新证据** |
| MxTools 角色 | 未记 | `FUN_100e4380` 惰性初始化中调 `0x100e6630`（DocumentInfo） | **10.1 新证据** |

## 未解决事项

- AfCore `SaveObject`/`LoadObject` 内部实现：GUID、键与对象体的写出/
  读取槽位未闭合（导入符号反编译为空，AfCore 侧本轮未分析）。
- `IMapDocument::Open` 槽位的最终归属：10.8 以托管元数据定为第 9 槽
  （`+0x24`）；10.1 侧 `+0x24` = `0x7b6a1220` 只做了局部反编译，
  未用 10.1 托管元数据（同 GUID、同方法顺序）复核槽位即断言等价。
  本文闭合的两条链（`+0x30`/`+0x38`）可能是 Open 之外的接口方法
  （如 CanOpen/文档获取），其接口名未逐一对回托管方法表。
- `FUN_7b7086c0`（预读形态后端）与链 A/B 的分工、调用条件未逐分支闭合。
- 类表 22 项中仅列出关键 6 项；`FUN_7b69f6b0` 的多个调用者未逐一归入
  具体接口方法。
- `0x7b6a1220`（`+0x24`）内 Variant 属性分支、`FUN_004798a8` 之后
  DataWindows 对象的具体类型、以及 LoadObject 各 GUID 常量对应的类名
  未逐一解码。
- 全部结论为静态反编译观察；未运行 ArcMap 10.1，未做任何 MXD 读取
  或转换的人工目视验收。

## 证据包

本轮原始 Evidence（CLI 导出，含 evidence_id、digest、limitations）与
r2/指令导出已汇总至忽略目录 `temp/arcgis/arcmap101-open-save-evidence.json`；
工作目录 `temp/arcgis/open-save-work/` 内含本机绝对路径，分享前须脱敏。
