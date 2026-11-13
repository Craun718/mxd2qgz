# ArcMap 10.1 对象体持久化分支与 /Maps 起点

记录日期：2026-11-13。研究目标：在 10.1 原生实现上复核对象头加载分支
`0x7ffcff70` 之后的普通/VariantStream 持久化分支、引用表生命周期、
键重复与错误分支，并闭合 `/Maps` 起点的读写证据。对照基线为 Git 历史
`c17b324` 的 10.8 文档（mapdocument-native.md、object-context-native.md、
maps-analysis.md）。本文只记录静态反编译观察，未运行 ArcMap，
未在 ArcMap 10.1 或 QGIS 中人工验证任何读取或转换结果。

## 文件身份与 SHA-256 复核

2026-11-13 重新对本机研究文件执行 `sha256sum`，与
[基线文档](arcmap-10.1-native-baseline.md) 全部一致：

| 文件 | SHA-256 |
| --- | --- |
| SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` |
| PageLayout.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` |
| ArcMap.exe.E97001B0_1E76_4BE8_B51F_269775FE0A80 | `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97` |
| AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` |
| MxTools.dll.C13FAB27_D904_4317_AC52_8A308EFED288 | `d29677c8514e705c6585b1b5a27d4e5cf32f7326891ccfe6ed7beeb70ca51347` |

## 对象头加载分支 0x7ffcff70（2026-11-13）

再次核对五个文件 SHA-256 均与上表一致。本轮 REA 6.9.0 CLI（npm 缓存
`9b9809298fe40d2c`）+ Ghidra 12.2 静态反编译；MCP 服务器注册的
`JAVA_HOME` 仍指向 JDK 21，Ghidra 12.2 拒绝启动，故本轮全部经 CLI 以
`JAVA_HOME=/usr/lib/jvm/java-25-openjdk` 运行，目标与摘要不变。首次
`decompile` 建立 Ghidra 项目耗时约 88 秒，后续调用复用。地址为
10.1.0.3035 静态映像虚拟地址。PDB（SystemCore.pdb，GUID
`98f4cce4-b371-4681-ad26-742de7b362bc`）加载状态未知；类名取自 Ghidra
类型信息或函数名常量（如 `VariantStream::vftable`），不表示运行验证。

### 读分支主体（`0x7ffcff70`，`ev_473e0fd34812ad4da5efb7d613d3ec1adc0cdba2ea6b104c040b8c00704547d1`）

参数为 `(this, riid, unkOuter→param_3, ppvOut→param_4)`。Ghidra 伪代码
将其恢复为 `void` 返回；实际返回值语义见下文汇编确认节
（E_POINTER/E_FAIL/S_FALSE/透传）。伪代码层次序列：

1. 输出指针为空或接口 `+0x10` 的底层流为空时提前返回
   （汇编分别返回 `0x80004003`/`0x80004005`，见下）。
2. `AfStreamRead`（底层流, GUID, 0x10, &实际量）读 16 字节 GUID。
3. `FUN_7ffb2e00(&DAT_8006be0c, &guid)` 为 16 字节相等比较
   （`ev_1260ac409008ad57876283a9ab9df024fa440d249b80ae3270c69052aafc7da5`）；
   相等（`DAT_8006be0c` 实测 16 字节全零）时以 S_FALSE 返回。
4. `AfStreamRead` 再读 4 字节键；`FUN_7ffd0a50(键, &已登记对象)` 查表。
5. 表命中：对已登记对象调用虚表槽 0，即 `QueryInterface(riid, ppvOut)`，
   不读对象体，不比较本次 GUID 与原登记类（与 10.8 `0x10070020` 命中分支一致）。
6. 未命中：`*(this+0x60)` 保存本次 GUID；`*(this+0x48)` 非空时经
   `StringFromCLSID`/`ProgIDFromCLSID`/注册表 `CLSID\{...}\InprocServer32`
   取模块名，`fwprintf("%ld\t- %s \t--  %-60s  \t--  %s\n", 键, CLSID, 模块, ProgID)`
   写调试日志。随后 `AfCreateInstance(&GUID, unkOuter, 0x17, riid, ppvOut)`
   创建实例，对实例调用虚表槽 1（AddRef）两次。
7. `FUN_7ffd0950(this+0x20, 键, 新对象)` 登记，发生在任何持久化调用之前
   （与 10.8 「对象体之前登记」一致）；随后 `FUN_7ffcefc0` 与 `FUN_7ffd0be0`
   两个接口查询分流持久化分支（见下节）。

### 查表与登记

- `0x7ffd0a50`（`ev_558253f23532424783381c06e4ea71db53443819fadee8e99ca4f94cb5e7565e`）
  哈希表查找：表结构首字段为桶数、第三字段为桶数组，键对桶数取模选链；
  链上按节点第四字段排序遍历，键相等时输出节点第二字段（对象指针）并返回 1，
  否则返回 0。与调用者的 `== 0 → 新对象` 分支一致。
- `0x7ffd0950`（`ev_a48d197d843b499184015a33112afd52dcd63386bff96991843ae4664116c8a8`）
  插入/更新：按同一排序规则遍历，找到键相等节点时**直接以新对象覆盖节点值并返回 1
  （替换分支），函数内不释放旧对象引用**；未命中则 `malloc(0x10)` 新节点
  （键存于第一、第四字段，对象存第二字段），桶计数加一并返回 0；
  分配失败抛 C++ 异常（`CxxThrowException`），不返回错误码。
  读分支调用点在新建对象之后，正常路径只走新增。
- 零 GUID 参考常量 `DAT_8006be0c` 实测 16 字节全零
  （`ev_204b01056cd6fc4b0d70418602e6cc3d7626d14a18573f563b83212bba3e0216`），
  证实读分支的提前返回即「全零 GUID」分支。

### ObjectStream 主接口虚表 `0x800710dc` 槽位（`ev_2d20e7253f852c8ef132a1e88165fb3652821c9fbff8bf552876982ba5388744`）

96 字节实读，24 槽（小端解析；`+0x0c`/`+0x44`/`+0x54`/`+0x5c` 与基线一致）：

| 偏移 | 目标 | 偏移 | 目标 |
| --- | --- | --- | --- |
| +0x00 | 0x7ffcfdb0 | +0x2c | 0x7ffcf6d0 |
| +0x04 | 0x80003340 | +0x30 | 0x7ffcf710 |
| +0x08 | 0x80000370 | +0x34 | 0x7ffcf740 |
| +0x0c | 0x7ffcf520 | +0x38 | 0x7ffcf7a0 |
| +0x10 | 0x7ffcf550 | +0x3c | 0x7ffcf770 |
| +0x14 | 0x7ffcf580 | +0x40 | 0x7ffcf7e0 |
| +0x18 | 0x7ffcf5c0 | **+0x44** | **0x7ffcff70**（对象头加载） |
| +0x1c | 0x7ffcf5f0 | +0x48 | 0x7ffd0370（保存分支候选） |
| +0x20 | 0x7ffcf630 | +0x4c | 0x7ffd0530 |
| +0x24 | 0x7ffcf660 | +0x50 | 0x7ffd0500 |
| +0x28 | 0x7ffcf690 | +0x54 | 0x7ffd0550（容量预估，基线已核） |
|  |  | +0x58 | 0x7ffd0590 |
|  |  | +0x5c | 0x7ffd05d0（计数，基线已核） |

基线已记录 PageLayout `0x7b70a8e0` 用接口 `+0x38` 连接 `Version` 流，
即 `0x7ffcf7a0` 是 SetStream 槽位候选；`+0x48` 与 10.8 保存分支
`0x100704e0` 槽位相同。两者反编译见下文。

### 构造与析构（补充基线）

- 构造 `0x7ffcf030`：Ghidra 类型名显示对象最终挂 4 个子接口——
  `+0x10` IObjectStream2（即 `0x800710dc`）、`+0x14` IDocumentVersion
  （`0x80071140`）、`+0x18` IObjectStreamDebug、`+0x1c`
  IObjectStreamRecorder（名称来自 Ghidra 虚表符号，非运行验证）。
  `+0x18` 的调试接口解释了读分支中 `this+0x48`（FILE*）与 `this+0x4c`
  的调试日志字段。
- 析构 `0x7ffcf200`：先重置全部子接口虚表，关闭 `this+0x58` 的 FILE*
  （调试日志），然后**遍历两张表**（计数在 `+0x24`/`+0x30`，桶数组在
  `+0x2c`/`+0x38`）逐节点对对象调用虚表槽 2（Release），最后销毁表与底层流。
  与 10.8 `0x1006f380` 行为一致；同样不能据此推断失败中途会回滚登记。

### 返回值与分支的汇编确认（`0x7ffcff70` 指令级证据）

`instructions` 导出（temp 内 `r2-sc-7ffcff70-asm.json`，Evidence 同上）逐条核对：

- 输出指针为空 → `MOV EAX, 0x80004003`（E_POINTER）；底层流为空 →
  `MOV EAX, 0x80004005`（E_FAIL）；两者均在写 `*ppvOut = 0` 之前/之后按序返回。
- 两次 `AfStreamRead` 经导入调用，只做 `TEST EAX,EAX; JL` 符号判断，
  **不比较读取量**；失败时 EAX 保留 HRESULT 直接透传返回。
- 零 GUID（与 `0x8006be0c` 16 字节相等）→ `MOV EAX, 1` 返回，
  即 S_FALSE（值为 1），与 10.8 `0x100700f1` 的语义一致。
- 表命中：`QI(已登记对象, riid, ppvOut)` 后跳 `XOR EAX,EAX` → S_OK，
  **覆盖 QI 自身的 HRESULT**，与 10.8 一致。
- 未命中：GUID 存 `this+0x60`、`this+0x68`（两条 MOVQ 共 16 字节）；
  `AfCreateInstance` 经调用 `[0x8006b070]`，参数 (GUID, unkOuter, 0x17, riid, ppvOut)，
  失败透传 HRESULT；新对象虚表槽 1（AddRef）连续调用两次；
  登记 `CALL 0x7ffd0950` 参数为 `(this+0x20 表, 键, 对象)`，此时尚未发生任何
  持久化调用。
- 伪代码中怪异表达式 `-(this != 0x10) & this` 的汇编为
  `LEA EAX,[EDI-0x10]; NEG EAX; SBB EAX,EAX; AND EAX,EDI`，
  即 `this != 0x10 ? this : 0` 的无分支实现；this 恒不等于 0x10，
  实际传参就是 ObjectStream 自身指针，两种持久化分支传参一致。
- VariantStream 分支（第二接口查询成功，接口存于局部 `+0x14`）：
  `malloc(0x1c)` → 经 `[0x8006b2d0]` 以 ObjectStream 为参构造 →
  `MOV dword ptr [obj], 0x80071220` 设接口虚表 → 接口指针位于对象 `+0x10` →
  对新建对象调用虚表 `+0x10`（槽 4），参数为该 VariantStream；失败时经槽 2
  Release 并透传 Load HRESULT。
- 普通分支（第二接口失败）：对第一接口调用虚表 `+0x14`（槽 5），
  参数为 ObjectStream 自身；失败透传 HRESULT。
- 成功尾部：`this+0x4c` 非零时调用 `0x7ffd0760`；随后三个局部智能指针析构
  （`0x7ffe29b0`）并返回 S_OK。持久化失败路径同样经这三个析构后返回，
  未见对已登记键的删除调用。

### 对象体由谁写、边界如何定（读分支证据）

汇编中两条持久化调用之前**没有任何长度或边界字段的读取**：
16 字节 GUID + 4 字节键读完后，立即 `CALL EAX` 进入类自身的
`Load(ObjectStream)`（普通分支，虚表 `+0x14`）或
`Load(VariantStream)`（VariantStream 分支，虚表 `+0x10`）。
即对象体由每个类自己的 Load/Save 方法读写，读完返回即到下一个对象头；
**没有外部长度字段**，未知类无法跳过。这与 10.8 结论一致
（「对象体没有可用的通用长度」）。

### 持久化分支的两个接口查询

- 第一查询 `FUN_7ffcefc0`：薄包装，置零输出后 `CALL FUN_800112a0`
  （QI 转移辅助，与 `FUN_7ffd0be0` 同构）；第二查询 `FUN_7ffd0be0`：
  查询结果为空时清理旧值并返回 `0x80004002`（E_NOINTERFACE），
  非空时把接口写入输出。
- 普通（IPersistStream）IID `00000109-0000-0000-c000-000000000046`：
  SystemCore 常量命中 `0x8006bdbc`、`0x80070fa0`。
- VariantStream IID `67a4d651-69e0-11d2-8500-0000f875b9c6`
  （IPersistVariant）：SystemCore 全映像唯一命中 `0x8007127c`
  （`ev_67f8614d571c19edbab1d25283becf6b6db81a55b52f62f9bf07036b6fc5ac71`），
  紧邻读分支写入的 VariantStream 接口虚表 `0x80071220`。
  VariantStream 包装对象 `0x1c` 字节、构造经 `[0x8006b2d0]`、
  接口指针位于对象 `+0x10`（汇编 `0x7ffd026f..0x7ffd0297`）。

### 保存侧辅助 `0x7ffd0370`（虚表 `+0x48`）

`ev_92996e64813442a104c193a4d1704853afac2c3947aecfd1058a1faa575a6767`：
接受调用者提供的对象——底层流为空直接返回；`param_2 != 0` 时先经
`FUN_800112a0` 对对象做 QI；取对象 ClassID（虚表 `+0xc`）；从底层流
读 16 字节 GUID：与 `DAT_8006be0c`（全零）相等即返回（结束），
与对象 ClassID 不等也返回（不匹配保护）；再读 4 字节键；最后对该
接口调用 `+0x14` 传 ObjectStream。语义是「按流中头校验后读入已有
对象」，对应 10.8 `0x100704e0`；不是通用重置入口，也不是对象写出器。

### 引用表生命周期（问题 2 复核）

- **SetStream `0x7ffcf7a0`（虚表 `+0x38`）**
  （`ev_132c3883e31f17f80acc2b363cb2958fbb3ee896b4b8403c856f2ccd9773642f`）：
  新旧流指针不同时替换 `this+0x10`，对新流 AddRef（槽 1）、旧流 Release
  （槽 2），返回 S_OK。**不清空两张对象表、不重置计数** ——与 10.8
  `0x1006f8c0` 一致：引用表跨 OLE 流切换保留（`/Maps`、`/PageLayout`、
  `View`、`DrawingDefaults` 共用同一 ObjectStream 实例时，键引用可跨流）。
- **登记时机**：读分支在 `AfCreateInstance` + AddRef×2 之后、任何
  Load 调用之前登记（汇编顺序 `0x7ffd0236 CALL 0x7ffd0950` →
  `0x7ffd0241 CALL 0x7ffcefc0` → Load），与 10.8 「对象体之前登记」
  一致，自引用/祖先引用静态可行。
- **键重复替换分支**：`0x7ffd0950` 存在显式替换（节点值直接覆盖，
  返回 1，函数内不释放旧对象引用）。读分支因先查表只在未命中时登记，
  正常读不会触发替换；写入侧若对同一对象重复保存（键已存在）则会走
  替换。旧对象引用是否在此泄漏/转移须由调用方核对——未闭合。
- **缺失引用 vs 新对象**：查表未命中一律按新对象创建处理
  （`0x7ffd0a50` 返回 0 → `AfCreateInstance`），没有独立的「坏引用」
  错误分支；与 10.8 一致。
- **错误分支**：见上文返回值表——E_POINTER/E_FAIL/S_FALSE(1)/
  透传 HRESULT/S_OK；持久化失败经三个局部智能指针析构后透传，
  **未见删除已登记键的调用**（同 10.8 的告诫：不是可重试保证）。
- **析构 `0x7ffcf200`**：遍历两张表逐对象 Release，关闭调试 FILE*，
  与 10.8 `0x1006f380` 一致。

## /Maps 起点：写入侧（ArcMap.exe `0x466fb0`）

保存入口 `0x466fb0`（`ev_35003540908ffc43202900f484f0e31ba2d1aacaae8e82ceff6255e65ec0a298`，
Ghidra 将其拆为多个反编译片段，本节引用的是同一函数体的伪代码）确认
10.1 写 `/Maps` 的完整顺序：

1. `CreateStream(IStorage, L"Maps", ...)` 创建流（同函数此前已创建
   `Version`、`Mx Document Version Info`、`Templates`、`DrawingDefaults` 等流）。
2. 把 ObjectStream 连接（SetStream）到 `/Maps` 流。
3. `AfStreamWrite(4 字节)` 写入 **Map 数量**，值取自文档对象字段
   `+0x128`（Map 指针数组在 `+0x124`）——数量语义即文档中的 Map
   （数据框）个数；参考样本实测为 1，与单数据框一致。
4. 循环数量次，对每个 Map 调 `SaveObject(map, ObjectStream, 1)`；
   首对象因此从 `/Maps:0x04` 开始。SaveObject 的内部实现（对
   ObjectStream 哪个槽位写 GUID + 键 + 委托对象体）本轮未闭合
   ——按符号名反编译返回空（Ghidra 名称解析失败），与 10.8
   「对象写出器未闭合」同状态；写侧顺序本身已由本函数的调用序列证实。
5. 循环结束后调用 ObjectStream `+0x54`（容量预估槽位）取值，
   写 **4 字节尾部标志**：返回 -1 时写 1，否则写 0。

与参考样本字节事实吻合：`/Maps:0x00` = `01 00 00 00`（1 个数据框）、
`0x04` 起 GUID + 键 `1d 00 00 00`（键 0x1d）、对象体随后；流大小
152941 字节。

## /Maps 起点：读取侧（PageLayout `0x7b708d00`）

`0x7b708d00`（`ev_8207418e58ebe1a23fa468dd67fc4ba3b2db786ad29e9c0311237bada0f29076`）
是 10.1 的文档打开主读取路径（基线所述「打开存储后调用的大型函数」），
结构对应 10.8 `0x10075370`：

1. 创建 ObjectStream（CLSID 常量 `0x7b71cf54`，即
   `043731d0-a7cf-11d1-8bd1-080009ee4e41`）。
2. 先尝试 **`Mx Document` 单流**（老格式）：OpenStream 成功时读 4 字节
   版本号，`> 1` 时清理并返回 `0x80004005`；版本 ≤1 时读 4 字节、
   SetStream（`+0x38`）、读 4 字节数量、按数量循环 `+0x44` 读对象。
3. `Mx Document` 打开失败才进入**分流格式**（10.1 参考样本属此格式）：
   依次处理 DrawingDefaults，然后 `OpenStream("Maps")`
   （IStorage `+0x10`，流名是 ANSI `"Maps"` 经辅助函数转 BSTR——
   因此全 DLL 无 UTF-16LE 独立 `Maps` 字面量，字节搜索只能命中
   `\ArcGIS\Web Maps\` 注册表路径）。
4. ObjectStream `SetStream`（接口 `+0x38`）连接 `/Maps` 流。
5. 从流读 4 字节有符号数量；**仅正数进入循环**（负数与 10.8 一样
   不进循环）。
6. 循环数量次：ObjectStream 接口 `+0x44`（即 `0x7ffcff70`）读对象，
   请求 IID 常量 `0x7b71c76c`（字节核对见下）；对象非空时经集合接口
   `+0x14` 加入 Map 集合并登记到输出。首个对象读取起点因此是
   `/Maps:0x04`，不靠字符串扫描。
7. 循环结束后**再读 4 字节**存入输出参数（与写侧的尾部标志对应：
   `-1 == 容量预估返回值` 的 0/1）。
8. 之后同样方式打开 `PageLayout`、`View` 等流并转接同一个
   ObjectStream（SetStream 复用同一实例——引用表跨流保留，见下节）。

`+0x44` 循环请求的 IID 常量 `0x7b71c76c` 实读 16 字节为
`00 00 00 00 00 00 00 00 c0 00 00 00 00 00 00 46`，即 **IID_IUnknown**；
10.8 读循环同样请求 IUnknown。

## 与 10.8 逐项对照

10.8 结论取自 Git 历史 `c17b324` 的 mapdocument-native.md、
object-context-native.md、maps-analysis.md。逐项对照
（10.1 地址 / 10.8 地址 / 结论）：

| 项目 | 10.8 | 10.1 | 对照 |
| --- | --- | --- | --- |
| 读分支入口 | `0x10070020` | `0x7ffcff70`（虚表 `+0x44`） | 结构一致 |
| 空输出返回 | E_POINTER | `0x80004003` | 相同 |
| 无底层流返回 | E_FAIL | `0x80004005` | 相同 |
| 全零 GUID | S_FALSE（`0x100700f1`） | `MOV EAX,1` = S_FALSE | 相同 |
| GUID/键读取检查 | 只看 HRESULT 符号 | 只看符号（`TEST/JL`） | 相同，均不查读取量 |
| 表命中 | QI 后 `XOR EAX` 覆盖为 S_OK | 相同（汇编 `0x7ffd0326`） | 相同 |
| 表未命中 | 创建实例并登记 | 相同（`AfCreateInstance` + `0x7ffd0950`） | 相同 |
| 登记时机 | 对象体之前 | 对象体之前（Load 前汇编序） | 相同 |
| 键重复替换 | 插入函数细节未记 | `0x7ffd0950` 显式替换、返回 1、不释放旧引用 | **10.1 新证据** |
| malloc 失败 | 未记 | `CxxThrowException` 抛异常 | **10.1 新证据** |
| 普通分支 | IPersistStream `Load`（`+0x14`，IID `00000109`） | 相同（IID 常量 `0x80070fa0`） | 相同 |
| VariantStream 分支 | 第二接口 `67a4d651`，`Load`（`+0x10`），0x1c 字节包装 | 相同（IID 常量 `0x8007127c` 唯一命中，接口虚表 `0x80071220`） | 相同 |
| 保存侧辅助（读入已有对象） | `0x100704e0`（`+0x48`） | `0x7ffd0370`（`+0x48`），GUID 校验 + 键 + `+0x14` | 相同 |
| SetStream 不重置引用表 | `0x1006f8c0` | `0x7ffcf7a0`（虚表 `+0x38`） | 相同 |
| 析构遍历两表 Release | `0x1006f380` | `0x7ffcf200` | 相同 |
| `/Maps` 分流读取 | Mx Document 失败→DrawingDefaults→Maps（`0x10075370`） | 相同顺序（`0x7b708d00`，含 Mx Document 版本 ≤1 老格式分支） | 相同 |
| `/Maps` 数量位置 | 读侧 `0x00` 4 字节 | 读侧相同 + **写侧确认**（ArcMap `0x466fb0` 先写数量再循环 SaveObject） | **10.1 写侧新证据** |
| 数量语义 | 开放 | 文档对象 `+0x128` 的 Map（数据框）数 | **10.1 新证据** |
| 负数量不进循环 | ✓ | ✓（`if (0 < local_50)`） | 相同 |
| 尾部 4 字节 | 读侧见值、来源开放 | 写侧确认：`(容量预估槽 +0x54 返回 == -1) ? 1 : 0` | **10.1 新证据** |
| 读循环请求 IID | IUnknown | IID_IUnknown（常量 `0x7b71c76c`） | 相同 |
| 对象体边界 | 无外部长度、未知类不可跳过 | 相同（两条 Load 调用前无任何长度读取） | 相同 |
| 对象写出器（GUID+键 谁写） | 未闭合 | 未闭合（ArcMap `SaveObject` 符号反编译为空） | 双版本开放 |
| 流名字符串形式 | 未记 | ANSI C 字面量经辅助函数转 BSTR（无 UTF-16 字面量） | **10.1 新观察** |
| 失败回滚 | 未见删除已登记键 | 相同 | 相同（均不保证可重试） |

## 跨版本适用范围（问题 4）

**有 10.1 原生证据（本轮，绑定 10.1.0.3035 文件摘要）**：读分支全部
行为（返回值、命中/未命中、双持久化分支、登记时机、IID、VariantStream
包装）、SetStream 不清表、查表/替换/析构、`/Maps` 写入顺序与数量
语义、`/Maps` 分流读取顺序与首对象起点、尾部 4 字节写侧来源。

**仍只有 10.8（本轮未在 10.1 复核）**：MapDocument 类工厂/CLSID 注册
与 Open 入口链（10.1 的对应类表闭合未做）；`Mx Document` 老格式的
完整字段语义（10.1 只确认存在该分支与版本 >1 报错）；DocumentInfo/
Metadata 连接细节；`String::Read/Write` 之外的对象体内部字段编码
（每个类的 Load/Save 属后续逐类研究）。

**双版本均开放**：对象写出器槽位（GUID+键的写出函数）、键重复时旧
对象引用的释放责任、失败中途登记残留的恢复行为、`/Version` short 与
对象流的进一步关系。

## 未解决事项

- ArcMap `SaveObject` 内部实现：Ghidra 对该符号反编译返回空，
  写出 GUID/键的具体槽位未闭合（读侧 `+0x44` 已闭合）。
- `FUN_7ffd0be0`（第二 QI 转移）所查询的接口对象来自哪次 QI 调用
  的返回寄存器，伪代码层次未完全闭合；IID 常量 `0x8007127c` 的引用
  点（哪个函数 push 它）未逐条列出——由「VariantStream 分支存在性 +
  常量唯一命中 + 紧邻 VariantStream 虚表」三者支撑，非直接指令级
  闭合。
- 替换分支（键重复）中旧对象引用的释放责任未核对调用方。
- `Mx Document` 老格式（版本 ≤1）的字段语义与版本 >1 返回
  `0x80004005` 的适用条件未展开。
- 读取循环对每次 `+0x44` 返回值的处理只见了 `JL` 符号判断的等价
  分支（`if (iVar5 < 0)`），读取量同样未比较。
- 全部结论为静态反编译观察；未运行 ArcMap 10.1，未在 QGIS 中做
  任何转换核对，人工目视验收未进行。

## 证据包

本轮原始 Evidence（CLI 导出的完整 JSON，含每条记录的 evidence_id、
digest、limitations）已汇总至忽略目录
`temp/arcgis/arcmap101-object-body-evidence.json`；各单条原始文件在
`temp/r2-*.json`（含本机绝对路径，分享前须脱敏）。
