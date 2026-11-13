# ArcMap 10.1 SystemCore 类工厂与构造闭合研究

记录日期：2026-11-13。研究目标：闭合 SystemCore.dll 中 ObjectStream
（CLSID `043731d0-a7cf-11d1-8bd1-080009ee4e41`）的 COM 类工厂链、实例
身份、对象写出器（写侧 GUID + 键的分配与写出顺序），以及
[对象体文档](arcmap-10.1-object-body.md) 遗留的三个静态问题，并与 10.8
文档（Git 历史 `c17b324`）对照。本文只记录静态反编译观察，未运行
ArcMap 10.1，未做任何人工验收；不据此声称 MXD 读取或转换能力已验证。

## 文件身份与环境

本轮仅分析一个文件并逐条复核：

- 目标 `SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`，
  `sha256sum` = `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，
  与 [基线文档](arcmap-10.1-native-baseline.md) 一致（ArcMap 10.1.0.3035）。
- 工具：REA 6.9.0 CLI（npm 缓存 `9b9809298fe40d2c`）+ Ghidra 12.2，
  全部经 CLI 以 `JAVA_HOME=/usr/lib/jvm/java-25-openjdk` 运行
  （MCP 注册的 JDK 21 会被 Ghidra 12.2 拒绝）。
- 下文地址全部是 PE 静态映像虚拟地址，绑定上述文件摘要，不是 MXD 偏移。
- 节区映射：`.text` 0x7ffb1000（raw 0x400）、`.rdata` 0x8006b000
  （raw 0xb9a00）、`.data` 0x800a4000（raw 0xf1c00）；`.rdata` 内锚定
  映射 `file_off = 0xCC1B8 + (va - 0x8007d7b8)`（由
  `rea address-to-file-offset` 权威给出，0x8007d7b8 → 836024）。
  Ghidra 对 `.data` 附近个别地址的字节读回与文件不符，凡涉字节事实均以
  `address-to-file-offset` + PE 文件直接解析裁决。

## 任务 1：类工厂闭合（DllGetClassObject → CreateInstance）

证据：`ev_a22ccfc754c6cbde19fb04f532f31706bfdc5a8e042b451bd522c6ecee786da3`
（DllGetClassObject）、`ev_600287947fb6889b421f82bc6884fdc79300c9114c3316f41198680cf1371b4d`
（同一内容的反编译）、`ev_bbee8b98b55b1ef9272d067da981bac33df6da37fe2c30129b3d216de8f02a94`
（0x80054220）、`ev_fa2e06a292df507fd15416945f46bac967fab90cf76e3b11b3576393a5e44176`
（注册序列指令）、`ev_69a57903b70aa8cf8f0a9ef9bbaf24fd220ee2ac7f9a9c4652229945c4e0b821`
（0x80070f80 xrefs）、`ev_639ed9398edd625ef51de702de97cc25423cb5772b6b03ed561b24132f47caca`
（CLSID 字节唯一命中）、`ev_a69be21fb26b46d7e143708356e92502e922ed22e27a04acbe23c72542a50168`
（类表静态字节）、`ev_7e2c3fc8c5620b6368d7d1a15590b7d0050dd10bfb98b898af7bb9838440d713`
（CreateInstance）、`ev_1f708b826b83fd844e6e6c303b98c112e61048e0f3773c12e70a7cad28c65496`
（CreateInstance 指令）。

- 导出 `DllGetClassObject`（0x800413b0）仅 `JMP 0x80054220`。
- 0x80054220 遍历 `.data` 内运行时填充的三张平行表：CLSID 指针数组
  `0x800a4280`、工厂节点数组 `0x800a4288`、标志数组 `0x800a42b0`，
  节距 0x34（0xd 个 dword，约 32 类）。命中后以
  `(*(code *)**(undefined4 **)槽)((槽值), riid, ppv)` 调用节点的第一个
  函数指针，即对工厂节点做 `QueryInterface`；未命中返回 `0x80040111`。
  静态文件中三张表全零（13312 字节实读，`ev_a69be21f...`），故表内容
  只能由初始化代码证明，不能由静态表字节推断类数量。
- 表由大型初始化函数 0x80066910（体 0x80066910–0x80069d9f）填充：
  每类 `malloc(0xc)` 节点 `{node[0]=平坦函数表指针, node[4]=1, node[8]=0}`
  逐槽写入。
- ObjectStream 为槽 #19，注册序列指令级证据（函数 0x80066910 体内）：
  - `0x800672d6` 把 CLSID 指针 `0x80070f80` 写入 CLSID 数组槽；
    `0x80070f80` 处 16 字节为 `043731d0-a7cf-11d1-8bd1-080009ee4e41`，
    全映像唯一命中（`ev_639ed939...`）；xref 仅
    `[0x7ffcef66, 0x7ffcf003, 0x800672d6]`（`ev_69a57903...`），
    前两处是旁路创建路径（见下）。
  - `0x800672e0` 写名称指针 `0x80070fc0`（`L"ObjectStream"`）。
  - `0x80067304` 写标志、`0x80067310` 写工厂节点。
- 工厂节点 `node[0] = 0x8007d7ac` 为平坦函数表（文件字节核实，经锚定
  映射解析）：+0x0 QI = `0x800472f0`、+0x4 AddRef = `0x80047620`、
  +0x8 Release = `0x80047f10`、+0xc CreateInstance = `0x80047930`、
  +0x10 LockServer = `0x80047150`、+0x14 = `0x80050f00`。
- 工厂 QI `0x800472f0`（`ev_2d907780...`）接受两个 IID：`0x8006bdec`
  （16 字节实读 `49f93cf6-cc4a-5d15-f7fa-6ffcfc388f86`，`ev_f9223e80...`，
  聚合校验用 IID_IUnknown 候选）与 `0x8006bdfc`；LockServer `0x80047150`
  操作全局计数 `0x800aa420`。
- `CreateInstance 0x80047930`（`ev_7e2c3fc8...` + `ev_1f708b82...`）：
  `unkOuter` 非 0 时校验 IID == `0x8006bdec`，否则 `CLASS_E_NOAGGREGATION`；
  非聚合路径 `malloc(0x80)` → 构造 `0x7ffcf030` → 对新对象 +0x10 接口做
  QI 输出；分配失败返回 `0x8007000e`（E_OUTOFMEMORY），另一失败分支经
  `CxxThrowException(0x8008ecdc)` 抛异常。链条至此闭合：
  `0x800413b0 → 0x80054220 → 表槽 #19 → QI 0x800472f0 → 0x80047930 → 0x7ffcf030`。
- 旁路：`0x7ffcef40`（`ev_bd040e13...`）内 `0x7ffcef66`、`0x7ffcf003`
  直接以该 CLSID 常量经 AfCreateInstance（导入表 `[0x8006b070]`，
  `dwClsContext=0x17`）创建，并 OleRun + QI `0x80070fb0`；这是模块内部
  自建 ObjectStream 的路径，对应 PageLayout 读入口所见行为
  （[对象体文档](arcmap-10.1-object-body.md)）。

## 任务 2：实例身份闭合

既有证据（对象体文档）：构造 `0x7ffcf030` 写 +0x10 = `0x800710dc`
（主接口 IObjectStream2）、+0x14 = `0x80071140`（IDocumentVersion）、
+0x54 = 5；PageLayout `0x7b70a8e0` 经接口 `+0x38`（SetStream `0x7ffcf7a0`）
连接 Version 流，落在同一实例。本轮补齐写出侧槽位行为（任务 3），从而
+0x10、+0x14 两条虚表与全部关键槽位（+0x38 SetStream、+0x44 读分支、
+0x40 写出器、+0x48 读入已有对象）均有 10.1 静态证据，实例身份闭合。
`String::Read`/`short` 直读等为对象体内部约定，不在本文件范围。

## 任务 3：对象写出器（虚表 `0x800710dc` 各槽位）

主虚表 96 字节 24 槽实读见对象体文档（`ev_2d20e725...`）。本轮反编译
其写出侧槽位并逐条核对汇编（`ev_712934fb...`、`ev_ddb585ca...`、
`ev_ed7946dd...`、`ev_9a4c0e62...`、`ev_369f436f...`、`ev_ead3efd7...`、
`ev_e21fca15...`、`ev_5cd2cb51...`、writer 指令导出）：

| 槽 | 目标 | 行为 |
| --- | --- | --- |
| +0x30 | `0x7ffcf710` | 转发底层流（this+0x10）同槽位；无流返回 `0x80004005` |
| +0x34 | `0x7ffcf740` | 转发底层流 +0x34 |
| +0x3c | `0x7ffcf770` | GetStream：输出 this+0x10 并 AddRef |
| **+0x40** | **`0x7ffcf7e0`** | **对象写出器**（详见下） |
| +0x4c | `0x7ffd0530` | String::operator=（this+0x40） |
| +0x50 | `0x7ffd0500` | String::MakeBSTR（this+0x40） |
| +0x58 | `0x7ffd0590` | `param_2==0 ? [this+0x14]*10 : [this+0x20]*10` |

**写出器 = `0x7ffcf7e0`**（虚表 +0x40；Ghidra 把同一函数体的尾部拆分
显示为 `0x7ffcfdb0`，属同一函数）。写出顺序（指令级核对）：

1. 底层流空 → `0x80004005`；对象参数空 → `AfStreamWrite` 写 16 字节
   零 GUID（常量 `0x8006be0c`/`0x8006be14`，与读侧零 GUID 常量一致）。
2. 否则 `FUN_7ffd0cc0` 按对象查引用表已有键：命中则只写 4 字节旧键
   （调试输出 `\tReference %ld`），不重写 GUID、不再登记。
3. 新对象：取对象 ClassID（对象虚表 +0xc）；`this+0x48` 调试字段非零时
   经 `StringFromCLSID`/`ProgIDFromCLSID`/`CRegKey` 查
   `CLSID\{...}\InprocServer32` 写调试日志；然后
   `AfStreamWrite(GUID, 16)` 写 GUID。
4. 键分配：`FUN_7ffd0850(this+0x14 表, 对象, &键)` 分配键 = 当前计数
   `[this+0x2c]`；对新对象 AddRef（对象虚表 +0x4）；`FUN_7ffdab10(表,
   对象, 键)` 插入；`AfStreamWrite(键, 4)` 写键；计数 `[this+0x2c]++`。
5. 持久化分流：`FUN_800112a0`（IPersistStream QI 转移辅助，IID 常量
   `0x80070fa0`）与 `FUN_7ffd0be0`（IPersistVariant QI，push
   `0x8007127c`）——VariantStream 分支创建 `malloc(0x1c)` 包装
   （构造经 `[0x8006b2d0]`，接口虚表 `0x80071220`），调对象虚表 +0x14
   （槽 5，Save）传包装；普通分支调对象虚表 +0x18（槽 6，Save），
   实参为 ObjectStream 自身（`-(this!=0x10)&this` 的无分支实现）与 1。

**与读分支 `0x7ffcff70` 镜像成立**：读侧 GUID→键→查表→未命中
创建/登记→Load；写侧 GUID→键→查表→命中只写旧键/未命中登记→Save。
读侧普通分支 Load 在虚表 +0x14、VariantStream Load 在接口 +0x10，
写侧对应 Save 在 +0x18 / +0x14——槽位差与 COM 接口布局一致，不矛盾。
这闭合了 10.8 与 10.1 双版本均开放的「对象写出器（GUID+键 谁写）」
在 10.1 侧的证据。

## 任务 4：对象体文档三个遗留项

### (a) `0x7ffcff70` 第二 QI 的接口实参来源

指令级证据（`ev_f7bd15fa...` 读分支全指令、`ev_c69155b9...`、
`ev_a9927398...`）：两次 QI 的接口实参均来自
`0x7ffd023b: MOV EAX,[EBX]`——即 `AfCreateInstance` 返回的新对象基址
+0 处的外层接口指针。第一查询经 `0x7ffcefc0`（薄包装，`CALL
0x800112a0` 转发 QI，IID `0x80070fa0` = IPersistStream）；随后
`0x7ffd0246: MOV EBX,[EBX]`、`0x7ffd025e: CALL 0x7ffd0be0`（EAX 仍为
同一指针），`FUN_7ffd0be0` 经 `*(*[EAX])`（虚表槽 0）做 QI，push
`0x8007127c`（IPersistVariant，`67a4d651-69e0-11d2-8500-0000f875b9c6`）。
另核实 QI 转移辅助 `0x800112a0` 全指令：IID 常量 `0x80070fa0`、
输出为空返回 `0x80004002`、失败释放旧智能指针，与既有伪代码结论一致。

### (b) `0x8007127c` 全部引用点

xref（`ev_0da52370...`）：全映像唯一引用 `0x7ffd0bf0`
（`FUN_7ffd0be0` 体内）。即 IPersistVariant IID 常量只被第二 QI 使用，
VariantStream 分支的常量支撑由「唯一引用」闭合。

### (c) `0x7ffd0950` 全部调用点与替换分支旧对象释放责任

xref（`ev_f447101f...`）：全映像唯一调用点 `0x7ffd0236`（读分支登记，
参数 `(this+0x20 表, 键, 对象)`）。读分支先经 `0x7ffd0a50` 查表、只在
未命中时登记，故正常路径只走新增；替换分支（`0x7ffd09b5`：
`MOV [ESI+4],ECX` 覆盖节点值、返回 1、函数内不 Release 旧对象）在本
模块内没有可达调用方——写侧新对象插入用的是 `FUN_7ffdab10`，键重复
时走写出器的「已有键只写旧键」分支，不经插入。因此「替换分支旧对象
引用的释放责任」在 10.1 SystemCore 模块范围内闭合为：**无可达调用点，
该分支为死代码；不存在写侧重复登记路径**。其他模块是否调用本导出
不在本文件范围（单一文件约束）。

## 与 10.8 对照

10.8 材料取自 Git 历史 `c17b324` 的 `object-context-native.md`
（全文快照在 `temp/arcgis/sc-factory-work/oc-10.8.md`）与
`mapdocument-native.md` 类工厂节。逐项对照（10.8 / 10.1）：

| 项目 | 10.8 | 10.1 | 结论 |
| --- | --- | --- | --- |
| 读分支入口 | `0x10070020` | `0x7ffcff70` | 结构一致（对象体文档已记） |
| 对象写出器 | **未闭合** | **`0x7ffcf7e0` 闭合**（顺序见任务 3） | **本轮新证据** |
| 零 GUID 写出分支 | 未记 | 对象参数空写 16 字节零 GUID | **10.1 新证据** |
| 键分配来源 | 未记 | 计数 `[this+0x2c]` 递增分配 | **10.1 新证据** |
| 命中已有键 | 未记 | 只写 4 字节旧键，不重写 GUID | **10.1 新证据** |
| 类工厂 | `vtable 0x100897f4` 型 ClassFactory（CLSID `0x10089a84`，init `0x10001000`，CI `0x1000cb00`→ctor `0x10010cc0`，100 字节） | `.data` 注册表（0x800a4280 三张平行表）+ 平坦表 `0x8007d7ac` + CI `0x80047930`→ctor `0x7ffcf030`（0x80 字节） | 结构不同：10.8 是静态虚表型工厂，10.1 是 DllMain 阶段运行时注册表；分发逻辑同为 QI 分发 |
| 读入口 SetStream/析构/+0x48 | `0x1006f8c0`/`0x1006f380`/`0x100704e0` | `0x7ffcf7a0`/`0x7ffcf200`/`0x7ffd0370` | 一致（对象体文档已记） |
| 键重复替换分支 | 细节未记 | 死代码（任务 4c） | **10.1 新结论** |
| VariantStream IID 引用 | 多处命中未逐条闭合 | 全映像唯一引用 `0x7ffd0bf0` | **10.1 闭合** |

10.8 侧写侧顺序（「先写数量再循环 SaveObject」）仍只在 10.1
ArcMap.exe `0x466fb0` 有证据；SaveObject 内部（即写出器）在 10.1
由 SystemCore `0x7ffcf7e0` 闭合，10.8 对应函数本轮未分析（单一文件
约束），不能推定两版本同实现。

## 未解决事项

- `0x80054220` 未命中码 `0x80040111`（CLASS_E_CLASSNOTAVAILABLE）
  的完整适用条件（如许可校验分支）未逐条展开。
- 注册表第三张表（`0x800a42b0` 标志列）及名称列的运行时用途未逐一
  枚举；32 类的完整清单只取了 ObjectStream 槽，未逐类核对。
- 写出器中调试注册表路径（`StringFromCLSID`/`CRegKey`）的运行时
  开销与失败行为未验证；`FUN_7ffd0cc0` 查表与 `0x7ffd0a50` 是否同一
  结构仅由参数形态推断，未逐指令比对。
- `0x7ffcef40` 旁路创建路径（AfCreateInstance + OleRun）的调用时机
  （哪个入口在何时自建 ObjectStream）未向上追到调用方。
- `FUN_7ffdab10`（写侧插入）与 `0x7ffd0950`（读侧插入）的内部差异
  （如是否同样无失败回滚）未逐条比对。
- 全部结论为静态反编译观察；未运行 ArcMap 10.1，未做任何人工目视
  验收，不据此声称读取或转换能力已验证。

## 证据包

本轮原始 Evidence（CLI 导出的 JSON，含 evidence_id、digest、
limitations）在忽略目录 `temp/arcgis/sc-factory-work/`（66 个原始
文件）；汇总索引 `temp/arcgis/arcmap101-sc-factory-evidence.json`。
两处均含本机绝对路径，分享前须脱敏，且该目录不入库。
