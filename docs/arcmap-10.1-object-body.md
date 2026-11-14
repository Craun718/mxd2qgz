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

## 10.1 对象读取错误恢复分支（2026-11-14）

研究目标：闭合读取失败（短读、未知类、Load 返回错误 HRESULT）时
ObjectStream 与文档打开主读取器 `0x7b708d00` 的恢复/清理行为、引用表
回滚情况、错误向上传播路径及文档打开是否整体失败。证据为 REA 6.8.0 /
Ghidra 12.2 静态反编译与指令级观察，输入文件 SHA-256 复核为
PageLayout `ba9e71f4…8de9d69f`、SystemCore `f63cf949…3f7de3ad1a`
（与基线一致）；地址绑定 10.1.0.3035 静态映像。引用的 Evidence 记录：
`0x7b708d00` 主函数 `ev_8207418e58ebe1a23fa468dd67fc4ba3b2db786ad29e9c0311237bada0f29076`、
智能指针释放辅助 `0x7b6ee7a0` `ev_949814c5a8a9647f7c03f5e32ede1f6f82aa0a13803b9e4c3bbff1ceb5b06325`、
对象登记辅助 `0x7b708c20` `ev_b8252330ebbac93e57a0a13ab8523cdb7aea7ec94c876feb268eb30353c8c436`、
DrawingDefaults 加载器 `0x7b70e2e0` `ev_e6ad6fe3ad89d3166a62d4aafc0b96f9ba3ff3d2a9e07954ee37e11d26ec2294`、
存储打开包装 `0x7b7098b0` `ev_764272fcd08a7884816d93585495f32bb9a578be6fb1197bba939568aace9c7f`、
文档打开入口 `0x7b69f6b0` `ev_8dcb0e269943db8b880fa0a5e385288624a72b588ef8cbcfc5b0d819e16ac367`、
保存侧辅助读入 `0x7b70eb00` `ev_1ecdf3845e824bd97c6670bcfd7199f25de9539a7955f4798f1e2be7dba58bf9`、
缺省对象创建 `0x7b70e800` `ev_b986f467030a73affc3820f34a8d1c9e26f095fde46295d8d141247cc4bd56a6`、
`0x7b70ea70` `ev_fac19efd2fcd70d51f8f40376bebe54b363d1840e6cfa70cbe7312a3aeea93d3`、
EH 处理器字节与指令级证据 `ev_6bb004f2b30f7bbd67ec77ec48e0950464f5861682f78c3bf9a1c1562c56d44d`、
`ev_3ba37fdf3a16a7aa0ea8470c15da0862a294864eced777cb03bab4339413f529`（SystemCore 侧读空，属会话绑定差异，见下）、
`ev_88ec96d44bc199716095a0b8a35cc3833a4da710b90fc9b6c4c77e4824d232ca`、
`ev_d111784db07f33d89eaf013a7f08b055c6cac2be000ea00fb3e53f3bea2efde0`、
`ev_c2281db51b9b25466a84d36637e1b474eaf7b4daca699028fb72c245e84d23ec`、
`ev_3f17a20396e271e1cd8c804a2f04cf9fb0914bcbeccc07f53ede0a92a88190ce`、
`ev_40b32b35c5a30cbff91ea6de0d85bae3a4876f85392e353d878ca5ca7d131dc1`、
`ev_a3b3e5468a75af17bf9f5c29b15e0a30c42382c6dc5840d62c13aef67b49c3dd`、
调用方清单 `ev_b8fcc35b9f49ae70f04aac1df14d9dcf6c537ab52f19c159eca98ea3430b8e47`、
`ev_571779221cae097259e27cf1220f42d6f5c4bbf7090332b2afaeb5e2fbc06dfc`。
全部为静态结论，未运行 ArcMap 10.1，未人工验证。

### 主读取器 `0x7b708d00` 的 HRESULT 错误路径

- 所有失败分支汇入两个公共收尾块（伪代码标签 `LAB_7b708e32` 与
  `LAB_7b7096d3`）：按构造的逆序对全部局部智能指针调用 `0x7b6ee7a0`
  （非空则调虚表槽 2 Release 并清零，`ev_9498…`），随后返回。返回值
  两种：携带失败调用的 HRESULT（`LAB_7b708e32` 尾部 `return iVar5`），
  或固定 `0x80004005`（E_FAIL，`LAB_7b7096d3`，用于 `Mx Document`
  版本 >1、`PageLayout`/`View` 流打开失败等结构前置失败）。
  输出指针为空在函数最前返回 `0x80004003`（E_POINTER）。
- 失败收尾不触碰任何输出参数：它们在函数开头已被置零
  （`*param_3..*param_7 = 0`），只有在成功收尾（伪代码标签
  `LAB_7b7097a7`）才把局部文档/集合/类对象取出、AddRef 并写入输出。
  即**失败时调用方拿不到部分加载的文档对象**；已成功加载进局部
  Map 集合（`local_30`）的对象随收尾对集合与各局部智能指针的
  Release 一并释放（对集合 Release 会释放其持有的 Map 属静态推断，
  依赖集合实现，未逐类闭合）。
- `/Maps` 循环中途 `+0x44`（`0x7ffcff70`）失败：伪代码 `if (iVar5 < 0)
  goto LAB_7b7093fd` → 收尾，返回该 HRESULT。此前迭代已读出的对象
  已通过集合接口 `+0x14` 加入局部集合并经 `0x7b708c20` 登记
  （`ev_b825…`：经 `FUN_7b6f0510` 接口转移后调用对方虚表 `+0x1c`
  与 `+0xc`，再调 `+0xb4(0)`；该辅助自身失败路径未见对已登记项的
  撤销），这些对象不会回滚出集合，而是随整函数收尾释放。
- 数量读取、尾部 4 字节、`PageLayout`/`View` 流任一环节失败同样
  直接收尾返回，无重试、无部分提交。

### C++ 异常路径（SEH 展开栈）

- `0x7b708d00` 使用 MSVC 结构化异常展开：函数序言在
  `0x7b708d05` `PUSH 0x7b718088` 安装处理器
  （`ev_d111784…`、`ev_c2281db…`）；伪代码中的状态变量（Ghidra
  `local_8`，取值 0…0x16）在各局部构造/析构之间递增，是 /GS+C++
  EH 的状态跟踪。`0x7b718088` 处字节为
  `8b542408 8d420c 8b4a9c 33c8 e8(相对调用) b8 58b7727b e9(相对跳转)`
  （`ev_6bb004…`）：取帧指针、做 cookie 检查（`xor ecx,eax` 后调用）、
  再载入状态/作用域表指针 `0x7b72b758` 并跳转到 EH 分发
  ——与 MSVC `__ehhandler` 模式一致；具体辅助函数身份未闭合。
  语义推断：任何 C++ 异常（包括 SystemCore 查表登记 `0x7ffd0950`
  在 `malloc` 失败时抛出的异常，见上节）沿此处理器展开
  `0x7b708d00`，按状态表逆序析构已构造局部后异常继续上抛。
  此展开路径未做动态验证，属静态模式匹配推断。
- ObjectStream 实例本身持有在局部智能指针（Ghidra `local_18`）中，
  异常展开或 HRESULT 收尾都会经 `0x7b6ee7a0` 对其 Release，进而
  触发析构 `0x7ffcf200`（遍历两张对象表逐对象 Release）。结合上节
  「Load 失败不删除已登记键」，得出本节核心结论：
  **引用表不回滚——中途失败/异常后，表内残留在:ObjectStream 生存期内
  仍持有引用，直到 ObjectStream 整体析构才释放；没有按键的删除调用**。
  复用同一 ObjectStream 的后续流（`/PageLayout`、`View`）在失败后
  不会被本路径继续处理（函数已返回），但若上层捕获异常并重试打开，
  残留键是否会与新读取冲突未被任何代码路径验证——开放。

### 错误如何向上传播、文档打开是否整体失败

- `0x7b7098b0`（存储打开包装，`0x7b708d00` 唯一已解析直接调用方，
  `ev_5717…`）：打开 IStorage 成功后调用 `0x7b708d00`，其返回值原样
  上传；`StgOpenStorage` 失败也原样上传。函数自身的清理同样是对
  局部智能指针的 `0x7b6ee7a0`。
- `0x7b69f6b0`（文档打开入口，`0x7b7098b0` 的唯一已解析直接调用方，
  `ev_8dcb…`）：`FUN_7b7098b0` 返回负值时，五个输出智能指针全部
  Release 并置空，返回该错误；文档对象字段 `+0x38`（文档）、
  `+0x3c`、`+0x40`、`+0x44` 只在全部成功分支赋值。未知存储类型
  （`FUN_7b69f500` 返回非 0/1）与路径检查失败返回固定 `0x80004005`。
  即**任何读取失败都导致本次文档打开整体失败**：调用方既拿不到
  部分文档，也没有「跳过坏对象继续打开」的分支；恢复方式只能是
  重新打开。是否弹出用户级错误界面属上层（ArcMap.exe）路径，本轮
  未追踪——开放。
- DrawingDefaults 加载器 `0x7b70e2e0`（`ev_e6ad…`）：8 个
  `LoadObject` 串行执行，任一失败即把文档加载标志（`param_1+0x2c`
  的布尔字段）置 0 并返回；此前已成功的字段保留在文档对象上。
  该函数被 `0x7b708d00` 的分流格式分支调用，其失败同样使整体
  打开失败；字段残留只存在于即将被丢弃的文档对象上。

### 与 ObjectStream 侧失败行为合并（SystemCore 证据复核）

本节未改动上节 `0x7ffcff70` 的结论：GUID/键 `AfStreamRead` 失败
透传 HRESULT、`AfCreateInstance` 失败透传、Load 失败经三个局部
智能指针析构后透传、无登记键删除。本节新增的是其上层（PageLayout
主读取器）对相同 HRESULT 的处理：不重试、不部分提交、整体失败。
SystemCore 侧本轮另做了一次字节读取交叉核对（`ev_40b32b…` 返回
0 字节，`ev_3ba37f…` 同）：该次读空是 REA 会话绑定到
`/tmp/mxd-arcgis101-align2` 目录副本（SHA-256 与基线一致）导致的
会话路由差异，不是文件缺失；SystemCore 结论仍以上节
`temp/arcgis` 目标的证据为准。

### 与 10.8 对照

10.8 历史文档（`c17b324`）只记录了 ObjectStream 层「失败不回滚
登记键、不保证可重试」，未见主读取器层面的失败收尾记录。本轮在
10.1 新闭合的主读取器行为（公共收尾块、输出参数只成功才写、
SEH 状态表展开、入口层整体失败语义）在 10.8 侧没有逐地址对照
记录可比较，不能据此断言 10.8 相同或不同——双版本对照开放。

### 本轮闭合与仍开放

闭合（10.1 静态证据）：`0x7b708d00` 全部失败分支汇入两个公共
收尾块、输出参数仅成功时写、固定 E_FAIL/E_POINTER 返回值；
`0x7b7098b0`/`0x7b69f6b0` 的透传与整体失败语义；DrawingDefaults
加载器的失败标志；SEH 处理器安装点与状态表指针；引用表不回滚、
残留引用由 ObjectStream 析构统一释放。

仍开放：`0x7b708c20` 各虚表槽位（`+0x1c`/`+0xc`/`+0xb4`）语义；
EH 处理器调用的两个辅助函数身份与展开的动态验证；
`0x7b70ea70`（内部 `FUN_7b6c4490`/`FUN_7b70ec60`）的完整语义；
异常重试时 ObjectStream 残留键的行为；集合 Release 释放 Map 的
逐类闭合；10.8 侧主读取器失败路径的对应实现；ArcMap.exe 上层
对失败的呈现方式；全部结论未经运行验证。

## 10.1 对象键分配与回引写侧闭合（2026-11-14 第五轮）

研究目标：闭合 TODO 开放的「对象键的分配规则」「回引在流中的表示」「能否从流本身区分回引与新对象」
「循环引用编码」。证据来源分两路：AfCore.dll 经 REA 6.9.0 / Ghidra 12.2 共享会话分析（subject=
`temp/arcgis101-bin/AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF`，SHA-256 `031858c4…cc47c17`
开工前与基线一致；证据 `ev_c67bc79f…`（SaveObject 伪代码）、`ev_30b78320…`（SimpleSaveObject 伪代码）、
`ev_3ba50a88…`（SimpleSaveObject 全指令），证据包导出为
`temp/arcgis/arcmap101-objref-keys-evidence.json`（92 条，含既有轮次记录与本机路径，分享前脱敏）；
SystemCore.dll 与 ArcMap.exe 用本机 pyghidra（Ghidra 12.2 + JDK 25，复用项目 `/tmp/r3/ghproj_sc`、
新建 `/tmp/r5/ghproj_arcmap`，反编译/汇编产物在会话临时目录 `/tmp/r5/sc_work/`、
`/tmp/r5/arcmap_maps_region.txt`，未入库；SystemCore/ArcMap 输入与既有记录同摘要）。SystemCore 写出器
`0x7ffcf7e0` 的既有证据 `ev_712934fb…`（`temp/arcgis/arcmap101-systemcore-factory-followup.json`）本轮以
新鲜反编译复核，结论一致并补齐键写细节。样本字节核对用 `/tmp/r3-maps.bin`（SHA-256 `9852c0e2…8129b49`，
偏移为 /Maps 流内偏移）。全部为静态观察，未运行 ArcMap，未人工验证。

### AfCore 导出 SaveObject / LoadObject（证据：确定的）

- `SaveObject(IUnknown*, IStream*, int)` 导出 1844 @`0x68a7b640`：薄包装，SEH 安装后调
  `StrmUtil::SimpleSaveObject(param_1, param_2, param_3, 1)`（`ev_c67bc79f…`）。
- `LoadObject(IUnknown**, const GUID&, IStream*, IUnknown*)` 导出 1545 @`0x68a7b6d0`：调
  `0x68a7b490`（既有记录的「16 字节 GUID 读取+AfCreateInstance」内核辅助）后若输出非空再调
  `0x68a7b530`（本地反编译 `/tmp/r5/af_work/f68a7b6d0.c`；后一函数语义未展开，开放）。
- `StrmUtil::SimpleSaveObject(IUnknown* obj, IStream* stm, int fClearDirty, int writeHeader)`
  （`0x68a7af70`，`ev_30b78320…`/`ev_3ba50a88…`）通用保存器：`obj==0` 时直接
  `IStream::Write(stm, 16 字节零, 16)`（终止/空标记）；普通 IPersistStream 路径
  （汇编 `0x68a7b43f..0x68a7b476`）为 `GetClassID` → **`IStream::Write(stm, &clsid, 16, 0)`**
  （`writeHeader!=0` 时）→ **`pPersistStream->Save(stm, fClearDirty)`**（虚表 `+0x18`，
  `stm` 原样传入）——该分支**不写键**。IPersistVariant 路径按变体类型 0xd/9 递归并以
  0x1c 字节 VariantStream 包装分派。辅助 `0x68a7c1d0/0x7c240/0x7c0f0/0x7c160/0x7c320/0x7c390`
  为 fastcall QI 包装，所查 IID 与分支间的槽位链未逐条闭合（开放，见文末）。

### ObjectStream 写出器（虚表 `+0x40` = `0x7ffcf7e0`）逐分支闭合（证据：确定的）

以新鲜反编译复核 `ev_712934fb…`（`/tmp/r5/sc_work/f7ffcf7e0.c`），键写细节首次闭合：

1. **空对象**（参数 0）：写 **16 字节零** 到底流，返回（与读侧全零 GUID→S_FALSE 对偶）。
2. 取对象 ClassID 后 `AfStreamWrite(底流, ClassID, 16)` 写 16 字节 GUID。
3. `FUN_7ffd0850(对象指针, &键)` 查**写表**（键哈希＝指针 4 字节按 `*0x21` 混合；
   节点 `[0]`=对象、`[1]`=键、`[2]`=链、`[3]`=哈希）：命中（已有对象）→
   `AfStreamWrite(底流, &旧键, 4)`，**只写 4 字节键、不写体**，调试日志
   `"\tReference %ld\n"`（`this+0x48` 非零时）。
4. 未命中（新对象）→ `FUN_7ffdab10(ObjectStream+0x14, 对象指针, *(ObjectStream+0x2c))`
   插入写表（对象→当前键），`AfStreamWrite(底流, ObjectStream+0x2c, 4)` **写当前计数值作键**，
   然后 `*(ObjectStream+0x2c) += 1`（**先写后增**）；再分派对象体：
   `pPersistStream->Save(ObjectStream 自身, 1)`（虚表 `+0x18`）或 0x1c 字节
   VariantStream 包装（`0x80071220`，槽 `+0x10` 对偶读侧 Load）。调试日志
   `"%s \t-- %-60s \t-- %s\n"`（CLSID/InprocServer32/ProgID，`this+0x48` 非零时）。

### 键分配规则（闭合，一句话）

**键 = ObjectStream 对象 `+0x2c` 处的 4 字节单调计数器：新对象写入当前值作键、写后 +1；
计数器随 ObjectStream 实例生存，不按流重置（`SetStream 0x7ffcf7a0` 不清表不清计数，既有证据），
因此同一文档保存/打开会话内全部流共享一个键序。** 样本 /Maps 键序 29..2183（Map=29、CRS=30、
FDOGraphicsLayer=31、FeatureLayer=32 等）与 /Version 尾部计数 2267（见下）相互印证；构造器
反编译未见对 `+0x2c` 的显式清零，初值 0 属推断（开放）。

### 两张表与计数器布局修正（证据：确定的；修正既有记录）

- 写表基址 `ObjectStream+0x14`（`FUN_7ffdab10` 参数 `param_1+5`），元素数 `+0x18`
  （插入成功时 `param_1[1]++`）；读表基址 `+0x20`（读分支 `0x7ffd0950` 参数 `this+0x20`），
  元素数 `+0x24`。**`+0x2c` 是键计数器，不是桶数组**——修正本文 2026-11-13 节
  「析构遍历两张表（计数在 +0x24/+0x30，桶数组在 +0x2c/+0x38）」的记录：该记录出自
  Ghidra 局部名混叠，本轮以写出器与 `FUN_7ffd05d0` 的直接指令证据为准。
- `+0x5c`（`0x7ffd05d0`）= `GetCount(select, &out)`：`out==0 → 0x80004003`；
  `select==0 → *out = *(this+0x18)`（写表元素数），否则 `*out = *(this+0x24)`（读表元素数）。
  **ArcMap 保存序列末尾用 `+0x5c(0,&v)` 回填 /Version 的 4 字节计数**（既有记录），
  故 **/Version 尾部计数 = 本次保存会话写表登记的对象总数 = 键计数器终值** ——
  「/Version 尾部计数 2267 与 /Maps 对象图对象数的对应关系」由此闭合（逐对象核验未做，
  样本扫描因部分内部类 GUID 不在 esri.clsid 表只逐头命中部分键，见 open）。
- `+0x54`（`0x7ffd0550`）= `EnsureCapacity(this, select_word, count)`：
  `count/10`（0x66666667 magic 除法）为桶数下限、`max(17, …)`，`FUN_7ffd07b0` 重建桶数组
  并按旧链 rehash；`select==0` 选写表（`+0x14`）否则读表（`+0x20`）；**恒返回 0、
  不写任何输出参数**（汇编 `/tmp/r5/sc_work/asm_report.json`）。

### 回引的流内表示与「能否从流区分」（闭合）

回引条目 = 与首现对象完全同构的 **20 字节前缀（16 字节 GUID + 4 字节既有键），无对象体、
无任何专用标记**（写侧第 3 步 + 样本 `0x2d2c` 条目 A=20 无体既有证据 + 样本符号回引
`0x17465`/`0x18bd1`/`0x24977`（键 2139）多例）。**流本身不能只靠局部字节区分回引与新对象：
唯一判据是键是否已在本 ObjectStream 会话（跨全部流）出现过**——读者 `0x7ffcff70` 查表命中→
仅 QI 不读体、未命中→按新对象读体。循环引用因此无需专门编码：先登记后持久化（既有证据）
使祖先/自身引用天然表示为键回引；Rust 侧解析器必须维护「已见键集合」状态机。
新对象与回引写出的 GUID 均为对象 ClassID，与请求 IID 无关（写侧取 `GetClassID`）。

### SimpleSaveObject 与写出器的委派关系（开放）

顶层对象经 AfCore `SaveObject` 保存时键必然被写出（样本首对象 GUID 后即键 29），而
SimpleSaveObject 的普通分支不写键——委派必经其对 ObjectStream 目标流的专用分支
（`FUN_68a7c1d0` QI 成功后的 `0x68a7b01b..` 路径），但该路径各 QI 的 IID 与最终到达
`+0x40` 的槽位链未逐条闭合；普通分支与 ObjectStream 场景的互斥关系属静态推断。

### /Maps 尾部 4 字节（指令级闭合；对既有结论的精化）

ArcMap.exe 保存函数写点 `0x4699cd..0x4699f1` + `0x469a0a`（本机 pyghidra 局部反汇编
`/tmp/r5/arcmap_maps_region.txt`；`0x4699a2` 取文档对象 `+0x190` 的流对象、`0x4699e0`
`call [vtbl+0x54]`）：调用后 `MOVSX ECX, word ptr [EBP-0x14]` → `CMP ECX,-1` → `SETZ DL` →
`AfStreamWrite(流, &flag, 4)`。即 **尾部 4 字节 = `((int16)输出 == -1) ? 1 : 0`**。但 10.1
SystemCore 的 `+0x54` 槽（`0x7ffd0550`）是 EnsureCapacity：**恒返回 0、不写任何输出参数**，
`[EBP-0x14]` 在写点前无任何写入指令——故该 `-1` 实际来自栈残留（静态推断，开放：不排除
`+0x190` 处是其他包装对象其 `+0x54` 语义为写输出的 getter）。读侧对偶已闭合：打开入口
`0x4764d0` 把 /Version 的 4 字节计数传给 `+0x54(-1, v)` = `EnsureCapacity(读表, max(17, v/10))`
——**/Maps（与 /Version）尾部 4 字节的消费者语义 = ObjectStream 读表容量提示**；样本 /Maps
尾 4 字节 @`0x25569` = `01 00 00 00`，即提示 17 桶。写侧 10.1 实际只会产出 0 或 1。

## 未解决事项（第五轮新增）

- SimpleSaveObject 对 ObjectStream 目标流的委派链：`FUN_68a7c1d0` 等 6 个 fastcall QI 包装
  所查 IID、ObjectStream 分支（`0x68a7b01b..`）各槽位语义、到 `+0x40` 的确切到达路径。
- `+0x2c` 键计数器的构造初始化（反编译未见显式清零，初值 0 为推断）。
- `+0x190` 处保存侧流对象的类身份与其 `+0x54` 槽在运行时的实际行为（栈残留 -1 的来源）。
- `LoadObject` 导出的第二辅助 `0x68a7b530` 语义；`FUN_7ffd0850` 的表指针来源寄存器（ESI）
  在写者内的设置指令未逐条核对（伪代码层级）。
- 写表按对象指针哈希：同一对象跨 SetStream 后保存必然命中（键回引），跨文档不适用；
  替换分支（`FUN_7ffdab10` 返回 1）在写侧的触发场景未核。
- 全部结论为静态观察；ArcMap 10.1 与 QGIS 的人工打开与目视核对均未进行，不声称已验证。
