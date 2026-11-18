# ArcMap 10.1 Variant::Read/Write 与 PropertySet 值槽、对象分支、PropertySetProperty 槽位闭合

记录日期：2026-11-16。对应 TODO「MXD 解析」节 D 轮开放项（Variant::Read/Write
对象分支、bool=false Seek 分支、数组 0x2000 分支逐指令复核、PropertySet 对象
分支、PropertySetProperty 体、SystemCore 值序列化器对照、Variant::Write 对称
性、AddRef/Save thunk、双序列化器字段归属区分）。
全部结论为**静态反编译 + 字节常量 pefile 复核**，未运行 ArcMap，未做人工目视
验收；仓库样本仅含 VT_BSTR 值，数组/对象分支无样本字节佐证。地址均为 PE
10.1.0.3035 静态映像 VA（AfCore ImageBase `0x68a70000`、SystemCore
ImageBase `0x7ffb0000`）。

## 方法与环境

- AfCore.dll 10.1.0.3035（SHA-256 `031858c414c0105b3b978c754f3d0f31ba1d98f15d
  44ff15b1589c9adcc47c17`）与 SystemCore.dll 10.1.0.3035（`f63cf9494acc155df7
  fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`），均与
  [基线文档](arcmap-10.1-native-baseline.md) 一致，本轮开工前 `sha256sum`
  复核。
- 深度分析在本机 pyghidra（Ghidra 12.2 + JDK 25）完成；AfCore 复用既有缓存
  工程副本（打开前整目录复制到本轮私有目录，不共享锁），SystemCore 复用
  `temp/agent-fl-tail2/ghproj_sc/` 缓存工程的整目录副本。脚本与产物在本轮
  worktree `temp/agent-d-variant/`。Ghidra 桥接工具 MCP 未使用（并行轮约定）。
- IAT/导出序号、GUID 常量字节以 pefile 直接读取（普通 shell 工具观察）。
- 返回内容按数据对待；本轮目标内容中未出现指令性文本。

## 1. 值序列化器身份：SystemCore 按名导入 AfCore 导出，「本地副本」说法修正

前轮记录（`docs/arcmap-10.1-property-set.md`「值 Variant」节）称 SystemCore
的 PropertySet 值序列化器为「本地编译副本，导入表中无该符号」。**本轮推翻**：

- PropertySet Load 的值读取器 `0x7ffd3f00` / Save 值写出器 `0x7ffd3f90` 的
  序列化调用均为 IAT 间接调用，pefile 读得：
  - IAT `0x8006b2e0` = AfCore.dll **`?Read@Variant@@QAEJPAUIStream@@_N@Z`**
    （AfCore 导出序号 1771，RVA `0xc6a0`）；
  - IAT `0x8006b2e8` = AfCore.dll **`?Write@Variant@@QAEJPAUIStream@@_N@Z`**
    （序号 2059，RVA `0xcb50`）；
  - 同表另导入 `??0Variant@@`/`??1Variant@@`/`??IVariant@@`（复制构造、析
    构、operator&）与 OLEAUT32 `VariantInit`/`VariantCopy`。
- 即 `0x7ffd3f00` = `VariantInit` → **AfCore `Variant::Read(v, stream,
  true)`** → `Variant::operator&` → `VariantCopy(out, &v)` → `~Variant`；
  `0x7ffd3f90` = `Variant::Variant(&in)` → **AfCore `Variant::Write(v,
  stream, true)`** → `~Variant`。条目对象 +0x8 槽存的是普通 OLE VARIANT。
- 前轮未命中该符号的原因未复核（按序号/按名检索方式差异的猜测不予记录）；
  本轮以 pefile 导入表导出表双向对证。
- **推论**：AfCore `0x68a7c6a0/0x68a7cb50` 的逐分支分析就是 PropertySet 值
  编码的权威依据，不存在「两份实现是否一致」的对照问题（前轮开放项 1 就此
  闭合）。样本两实例 VT_BSTR 分支与此一致（辅助观察）。

## 2. `Variant::Read`（AfCore `0x68a7c6a0`，导出序号 1771）全貌

反编译 + 逐指令复核（本轮汇编为准，格式：`[VARTYPE:2]` 起步）：

1. `Clear(this)`（`0x68a7a1a0`）后 `IStream::Read(vt, 2, pcbRead=NULL)`，
   hr<0 透传；
2. **bool=false 的分支**（前轮任务 2）：`CMP byte [ebp+0xc],0; JNZ 跳过` →
   `Seek(stream, {LowPart=0xe, HighPart=0}, STREAM_SEEK_CUR(1), &新位置)`
   （汇编 `0x68a7c6fd` `MOV EAX,0xe`——Ghidra 反编译显示的 `0x100000000`
   是伪影）——**向前跳过 14 字节，返回值不检查**。语义是「完整头部」格式
   的保留字段，不是版本定位（前轮「版本读取后的定位调用」表述修正）；
3. 数组掩码：`vt & 0xdfff` 作为分发键（只清 0x2000 位，保留其他位）；数组
   标志判定用原始 vt；
4. switch（跳表 `0x68a7cafc`）非数组分支 VARTYPE 表与前轮记录一致：0/1 无
   载荷；2/11=2 字节；3/10/4=4 字节；5/7/6=8 字节；8=标量 BSTR；9/13=对象
   分支；0x10/0x11=1 字节；0x12=2 字节；0x13=4 字节；其余（含 >0x13）=
   `0x80004005`。定宽读取经 `AfStreamRead`（`0x68a84830`，纯转发
   `IStream::Read`，pcbRead=NULL，**短读不检查**）；
5. 标量 VT_BSTR（`0x68a7c90c`）：`String::Read`（`0x68ab46f0`）读入临时
   String 再 `SysAllocStringLen(data, *(header-4))` 存入 bstrVal。
   `String::Read` 布局（本轮反编译）：`[cb:4]`（malloc 尺寸）+ cb 字节载荷
   赋给 String；**cb==0 或 pcbRead==0 返回 `0x80004005`**；原生不检查终止
   零与偶数对齐（Rust 严格子集更严，兼容）；
6. default 返回 `0x80004005`；数组置位时的 default 返回 `0x80070057`
   （E_INVALIDARG，`0x68a7cae2`）。

### 数组分支（前轮任务 3，逐指令复核闭合；仍无样本）

分发后统一形态 `[计数:4][元素]`，计数用裸 `IStream::Read(4, pcbRead=NULL)`
读取（hr<0 透传；计数接收局部先清零）：

- `SafeArrayCreate`（OLEAUT32 序号 15，IAT `0x68c418f4`）以 `(掩码 vt,
  cDims=1, &bounds)` 创建，bounds = {cElements=计数, **lLbound=0**}（两条
  路径都有显式清零：数值路径 `0x68a7c80f`、BSTR 路径 `0x68a7c95c`）；创建
  失败（NULL，如计数过大或 VT_EMPTY/VT_NULL 建数组）→ `VariantClear` +
  返回 `0x80004005`；
- `SafeArrayAccessData`（序号 23）失败 → 同上返回 `0x80004005`；
- 定宽类型：**一次批量** `AfStreamRead(stream, pvData, cElements×元素宽,
  NULL)`（IMUL 32 位，溢出环绕；无短读检查），随后 `SafeArrayUnaccessData`
  （序号 24）；hr<0 → `SafeArrayDestroy`（序号 16）+ `VariantClear`；
- VT_BSTR 数组：**逐元素全局 `ReadString`**（`0x68a7b7a0`，`n*2+2` 布局，
  循环上限 `CMP i,计数; JC` 无符号；首个失败即断出），`SysAllocStringLen`
  为序号 4——即**数组内字符串布局与标量 BSTR 的 `String::Read` 不同**，两
  族布局在此分支汇合；
- `VT_EMPTY(0)|VT_ARRAY` / `VT_NULL(1)|VT_ARRAY` 复用 VT_BSTR 数组路径
  （`SafeArrayCreate(0/1,…)` 的成败取决于 OLEAUT32，NULL 则 `0x80004005`）。

### 对象分支 9/13（前轮任务 1/4，闭合语义、保留体边界开放）

`0x68a7ca4d`：`LoadObject(out=this+8, IID 0x68c6bc2c, stream, NULL)`，hr
直接透传。AfCore 导出 `LoadObject`（`0x68a7b6d0`）结构（本轮反汇编定论）：

1. 入口 `0x68a7b6fe`-`0x68a7b730`：`FUN_68a7c4b0(stream)`——对流 QI
   IObjectStream（第四轮核对 IID 常量 `0x68c6dca8`；获取文档版本的另一查询
   使用 `0x68c6dcb8`）。**命中 → 走上下文对象虚表 +0x44**（即
   [基线文档](arcmap-10.1-native-baseline.md) 记录的带键 ObjectStream 读
   分支：16 字节 GUID + 零 GUID 比较 + 4 字节键 + 登记表）——前轮
   「10.8 文档所述 LoadObject 入口 QI 上下文分支在 10.1 未定位」就此闭合，
   位置在 `0x68a7b6fe`；**未命中（MXD 普通 OLE 存储流，本仓库读取模型）→
   普通路径** 核心 `0x68a7b490`；
2. 普通路径核心：`*out = NULL`；读 16 字节类 GUID（`cbRead < 0x10` →
   `0x80004005`）；与全零 GUID 常量 `0x68c6bc4c` 比较（`0x68a775f0`，相等
   返回 1、不等返回 0，本轮逐指令确认极性）——**相等（零 GUID）→ 直接返
   回 S_OK，出参保持 NULL；不等 → `AfCreateInstance(&类GUID, 空, 0x17,
   IID_IUnknown, out)`**。
   较早记录曾写反极性，第四轮已据比较函数及分支指令更正。
   GUID 常量字节本轮
   pefile 亲核：`0x68c6bc2c` = `00000000-…-c000-…0046`（IID_IUnknown）、
   `0x68c6bc4c` = 全零 16 字节；
3. 非零 GUID 实例化成功后，尾部包装 `0x68a7b530` 将流与对象关联（反编译
   含大量不可达块警告，内部细节未闭合）——对象体由对象自己的持久化方法继
   续消费，**体边界由类决定，静态无通用边界**。

**闭合子集（已实现）**：零 GUID 的空对象编码 `[9/13:2][16×00]` 共 18 字节，
读侧（两条路径）消费量相同、写侧 `SimpleSaveObject` 对空对象写同一编码
（见第 4 节），与流上下文无关——作为显式读取扩展落位于
`src/objects/variant_object_branch.rs`
（`read_variant_object_branch_10_1`，不改现有
`read_variant_10_1`；建议接入点见「接线建议」）。

## 3. `Variant::Write`（AfCore `0x68a7cb50`，导出序号 2059）逐段对称性（前轮任务 7）

反编译 + 汇编复核，与 Read **逐分支对称**，字段序推断成立：

1. `IStream::Write(vt, 2, NULL)`；bool=false 时**再写 14 字节全零**
   （`0x68a7cb99` `CMP byte [ebp+0xc],0; JNZ 跳过` → `0x68a7cba3-0x68a7cbdf`
   将 14 字节栈块清零后 `Write(&块, 0xe, NULL)`）——与 Read 的
   `Seek(+14, CUR)` 严格互证；bool=true 时无此块。**不存在版本字段**；
2. 标量 BSTR：`String::operator=(临时, bstrVal)` + `String::Write`；
   定宽类型：`AfStreamWrite`（转发）按 2/4/8/1/2/4 字节写出 this+8 载荷；
3. 数组：`SafeArrayAccessData` → 写 4 字节 cElements（取自
   `*(SAFEARRAY*)+0x10`）→ 定宽一次批量写 `cElements×宽` 字节 / VT_BSTR
   数组逐元素全局 `WriteString`（`0x68a7b800`，`n*2+2`，首字符为零且非空
   指针才写载荷）→ `UnaccessData`；AccessData 失败返回 `0x80004005`；
4. 9/13：`SaveObject(obj=[this+8], stream, 0)`（`0x68a7b640`）→
   `StrmUtil::SimpleSaveObject`（`0x68a7af70`）：**对象为空 → 写 16 字节
   全零 GUID**；普通路径（流无上下文）→ 经 `GetClassID`（虚表 +0x0c）写
   16 字节 CLSID、再 `IPersistStream::Save`（+0x18）写体（CLSID 是否写出
   由第 4 参控制，Variant 路径恒写）；带键路径走流上下文登记表；
5. default（vt>0x13）：**非数组返回 `0x80004005`、数组返回 `0x80070057`**
   （汇编 `0x68a7ce60/0x68a7ce83` 定论；反编译的 `return lVar5` 是寄存器
   跟踪伪影）——与 Read 完全对称，写侧不会静默吞未知 VARTYPE。

## 4. PropertySetProperty `{5B6D2B14}`（前轮任务 5）

定位链（本轮全静态证据）：CLSID 字节常量 `0x8007195c`（pefile 文件命中唯一
字节序 `14 2b 6d 5b 4a f6 57 44 84 e0 61 7a e2 13 0f f9`）→ 引用三处：
GetClassID `0x7ffd2830`（写出该常量，空指针返回 `0x80004003`）、工厂/激活
辅助 `0x7ffd4ff0`（AfCreateInstance + OleRun + QI `0x800719cc`）、
DllGetClassObject 登记链 `0x80066910`。

RTTI `.?AVPropertySetProperty@@` 名称字符串 `0x800a6960`（pefile 文件命中），
TypeDescriptor 基址 `0x800a6958`，5 个 COL（TD 指针位于 COL+0xc）派出 5 个
子对象虚表，全部槽函数本轮反编译判别：

| 虚表基址 | COL | 槽位判别 |
| --- | --- | --- |
| `0x8007188c` | `0x80083750` | [QI, AddRef, Release(`InterlockedDecrement(this+3)`), **类名 L"PropertySetProperty"**（`0x7ffd2820`）, 析构, **GetClassID 在 +0x14 非标准位**（`0x7ffd2830`）, 空 nop] —— ESRI 内部族；注意 `0x80071894` 不是虚表基址，此前猜测的「+0x14=Load」槽值 `0x8008373c` 实为 COL 区数据 |
| `0x800718ac` | `0x8008373c` | 2 槽 `XOR EAX,EAX;RET` S_OK 桩（`0x800413c0`）×2 |
| `0x800718b8` | `0x80083728` | [QI `0x7ffcdb00`, AddRef `0x80043340`, Release `0x80040370`, **get_Name**（`0x800098a0`，取对象+0x8）, put_Name `0x7ffd4610`, get_Value `0x7ffd4650`, **put_Value(VARIANT)** `0x7ffd4680`] —— **IPropertySetProperty** |
| `0x800718d8` | `0x80083714` | [QI, AddRef, Release, **Serialize(IXMLSerializeData*)** `0x7ffd46a0`（出现 `VT_ARRAY|VT_BSTR` 不支持 SOAP serialization 字符串、XML ns esri:exref 常量）, Deserialize `0x7ffd4ad0`] —— **IXMLSerialize** |

**结论（闭合）**：SystemCore 内 PropertySetProperty **没有 IPersistStream
虚表**（无任何 `[GetClassID@+0x0c … Load@+0x14 Save@+0x18]` 形态）也**没有
IVariantStream**——不存在可反编译的流对象体序列化器。esriSystem 语义下它
是 PropertySet 条目的内存形态（Name/Value 持有者），MXD 内承载方式就是
PropertySet::Load 的内联「键名 String + 值 Variant」条目本身（样本 2/2 如
此，且 PropertySet::Load `0x7ffd3080` 体内无对象头读取）；对象头编码在其
他宿主（XML/SOAP 路径）走 IXMLSerialize，域外。前轮开放项 6 以「无体可反
编译」闭合；`PROPERTY_SET_PROPERTY_CLASS_ID` 常量保留为诊断对照。

## 5. PropertySet AddRef/Save thunk 同目标问题（前轮任务 8）：旧记录误读修正

IPersistStream 虚表 `0x80071840` 全槽本轮逐指令复核（与
`docs/arcmap-10.1-property-set.md` 虚表表一致），但两处 thunk 细节修正：

- **AddRef thunk `0x7ffd5580`**：`MOV ECX,[ESP+4]; SUB ECX,[ECX-4];
  **SUB ECX,0x28**; MOV [ESP+4],ECX; **JMP 0x80043340**` ——目标是共享
  AddRef 实现 `0x80043340`（该实现同时出现在 PropertySetProperty 虚表 C
  的 +0x04），**不是 `0x7ffd3280`**。前轮「AddRef 与 Save 转跳目标相同」
  是把相邻 thunk 的转跳目标张冠李戴，该开放项以「目标本不相同」闭合；
- **Save thunk `0x7ffd55c0`**：`MOV ECX,[ESP+4]; SUB ECX,[ECX-4];
  MOV [ESP+4],ECX; JMP 0x7ffd3280` ——仅按 `[this-4]`（vcall 序号）调整，
  无 -0x28；QI/AddRef/Release thunk 为 `[this-4]` 调整后再减 0x28 的两级
  adjustor 形态，Load/Save/GetClassID/GetSizeMax thunk 仅 `[this-4]`。
  虚表槽位归属（COM 顺序）不受影响，Load/Save 字段序结论不变。

## 6. `Variant::Read/Write` 与 VariantStream 的字段归属区分（前轮任务 9）

两族序列化器的判别式（全部闭合，静态证据见
[VariantStream 复核](arcmap-10.1-variant-stream.md) 与本文）：

| 判别项 | 类方法 `Variant::Read/Write`（AfCore `0x68a7c6a0`/`0x68a7cb50`） | 全局 `Read/Write`（VariantStream 用，`0x68a7b9a0`/`0x68a7b860`） |
| --- | --- | --- |
| 头部 | 仅 `[vt:2]`；bool=false 时另有 14 零字节（Read Seek / Write 写零对称） | 仅 `[vt:2]` |
| VT_CY(6)/VT_ERROR(10)/VT_I1(0x10)/VT_UI2(0x12)/VT_UI4(0x13) | 支持（8/4/1/2/4 字节） | 不支持（default → `0x80004005`） |
| VT_BSTR(8) 标量 | AfCore `String::Read/Write`：**cb 含终止零**（cb==0 拒绝） | 全局 `ReadString/WriteString`：**n*2+2**（n==0 无载荷） |
| VT_BSTR 数组元素 | **同全局 `ReadString/WriteString`（n*2+2）** | —— |
| 数组 0x2000 | 支持（`[计数:4]`+SafeArrayCreate/AccessData 批量/逐元素；default→`0x80070057`） | 不存在 |
| 9/13 对象 | `LoadObject(out, IID_IUnknown, stream, NULL)` / `SaveObject(obj, stream, 0)` | `LoadObject`（IID_IUnknown）/ `SaveObject(obj, stream, 0)`——同函数 |
| 短读检查 | pcbRead 均为 NULL，不检查 | `pcbRead` 传入但不比较 |

**调用点归属（已证）**：PropertySet 键名与值 = 类方法族（Load
`0x7ffd3080` → `String::Read`/AfCore `Variant::Read`，IAT 证据见第 1 节）；
VariantStream 的 `Read/Write` 成员（`0x68a7bc50`/`0x68a7bc70`）= 全局族。
**未证**：MXD 其他字段各自走哪族——按 AGENTS 纪律这属各字段读取器的归属
判定，不因本轮「判别式闭合」而外推；在调用点身份确认前，两族读取器继续
不得混用（现有 `stream::variant_stream` 与
`objects::property_set::read_variant_10_1` 的分立维持不变）。

## 实现（本轮代码产物）

- `src/objects/variant_object_branch.rs`（新模块，`src/objects/mod.rs` 仅
  加一行声明）：`read_variant_object_branch_10_1`——仅接受 VARTYPE 9/13 且
  类 GUID 全零（`[vt:2][16×00]` 共 18 字节）返回
  `VariantObjectBranch10_1::Null`；非零 GUID 显式 `Unsupported`（错误信息
  携带 16 字节原样字节序）；trial 游标事务性。测试 7 项（vt 9/13 正常、
  非零 GUID 拒绝与游标不动、非对象分支 VARTYPE 分派错误、截断各前缀、
  EOF 事务性），全部为可构造输入，不代表原生运行验证。
- `docs/arcmap-10.1-property-set.md` 等既有文档未改动（对旧记录的修正见本
  文第 1/2/4/5 节）。

## 接线建议（不在本轮改动，交整合轮）

- `src/objects/property_set.rs` `read_variant_10_1` 的 `9 | 13` 分支：改调
  `crate::objects::variant_object_branch::read_variant_object_branch_10_1`，
  `Null` 映射为 `PropertyValue10_1` 的新变体（如
  `PropertyValue10_1::NullObject`）；现有
  `PropertyValue10_1`/`read_variant_10_1` 其余行为不动。
- 本轮当时未接入数组；后续实现范围见[单维数组接入](#单维数组接入)。
- `tests/robustness.rs` 集中清单如需登记新入口，由整合轮统一处理。

## 未解决事项

1. 非零类 GUID 对象分支的**对象体边界**：`AfCreateInstance`（`0x68a77660`，
   `0x17` 参数语义）与尾部包装 `0x68a7b530`（流-对象关联、`VariantStream`
   包装）内部未闭合；需要 COM 运行上下文，严格读取器永久显式拒绝。
2. LoadObject/SaveObject 带键上下文路径（流 QI `ecc43c55…` 命中时）在 MXD
   中的实际触发条件：MXD 各宿主传入 PropertySet::Load 的流是否携带该上下
   文未运行取证；零 GUID 空对象子集与该问题无关（两路径编码相同）。
3. 数组分支与 9/13 非零 GUID 分支均无 MXD 样本字节佐证；SafeArrayCreate
   对 VT_EMPTY/VT_NULL 与超大计数的具体失败行为依赖 OLEAUT32 运行时。
4. 版本 0 与负值位模式（PropertySet 体）、各值的业务语义（DATABASE/连接属
   性候选）等既有开放项维持不变（见 property-set 文档开放项 2/4）。
5. MXD 字段级序列化路径归属（第 6 节「未证」行）：未逐字段确认前不得以本
   文判别式外推。
6. 全部结论为静态观察；未运行 ArcMap 10.1，未在 QGIS 中核对，不声称已
   验证。

## 单维数组接入

依据上述 AfCore.dll 10.1.0.3035 `Variant::Read`（`0x68a7c6a0`）和
`Variant::Write`（`0x68a7cb50`），PropertySet、Field 默认值、值域、
MosaicRule 排序值和 EditTemplate 共用 [数组读取器](../src/stream/variant_array.rs)。
去除 `0x2000` 标志后，已证固定宽度元素保留 VARTYPE 与原始位模式；
BSTR 元素采用全局 ReadString：零长度只占 4 字节，非零长度为码元数，
另读末尾终止零。带 14 字节保留区与不带保留区的 Variant 仍由各宿主指定。

读取器拒绝负计数、未知元素和超限载荷，并在截断时保留原游标。
合成测试覆盖编码值域及 Field 默认值的对象边界、混合空与非空的 BSTR 数组、
逐字节截断与总载荷限额。VT_EMPTY/VT_NULL 数组的 SafeArrayCreate 行为、对象
数组及其他未知类型仍未实现；未在真实 MXD 中观察到这些数组，也未进行 ArcMap
运行验证。

## 带保留区的 ObjectStream 对象值

AfCore.dll 10.1.0.3035 的 Load 在 `0x68a7c6eb..0x68a7c709` 根据实参跳过
14 字节，然后才分派 VARTYPE。9 / 13 共享 `0x68a7ca4d`，把原 IStream
直接传入 `LoadObject 0x68a7b6d0`；保留区不影响底流的 IObjectStream 查询。
Write 在 `0x68a7cb99` 后对应写 14 个零字节，然后同样分派对象值。

栅格唯一值类别、UniqueValues、RangeDomain、CodedValueDomain 和 MosaicRule
的已确认 ObjectStream 宿主复用同一读取器。空对象不登记键；回引及新
PropertySet 共享调用方的对象键表，内部 PropertySet 条目保留其不带保留区的
自身布局。未知新类和对象数组继续显式停止。合成验证覆盖类别与统计集合之间
的回引、值域回滚及 MosaicRule 后继字段；未做 ArcMap 运行验证。
