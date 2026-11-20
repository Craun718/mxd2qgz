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
逐字节截断与总载荷限额。剩余元素类型的原生分派见[下节](#值域数组的读写分发表复核)。未在真实 MXD 中观察到这些数组，也未进行 ArcMap 运行验证。

## 值域数组的读写分发表复核

RangeDomain 与 CodedValueDomain 均经 AfCore.dll 10.1.0.3035 的
Variant(false)。本轮对照已核验映像（SHA-256
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`）
的读写跳表 `0x68a7cafc/0x68a7cea8`。两侧只清除
`VT_ARRAY (0x2000)` 再进行分派。

| 去除数组标志后的值 | 读写行为 |
| --- | --- |
| 2–8、10、11、16–19 | 已接入的一维定宽及 BSTR 元素布局 |
| 0、1 | 带数组标志时进入 SafeArray 路径，见下方 OLE 复核 |
| 9、13 | 直达单对象 LoadObject/SaveObject（`0x68a7ca4d/0x68a7ce1c`），没有数组计数或逐元素分派 |
| 12、14、15、大于 19 或还含其他标志位 | 进入缺省错误路径，数组返回 `E_INVALIDARG` |

介质分发件 Microsoft OLEAUT32.DLL 2.40.4275（SHA-256
`baeb2f7c1b8be56738d34e1d1ddf8e0eebd3a633215dc1575e14656be38b939d`）
的 SafeArrayCreate（序号 15，`0x6534393a`）先计算
`VARTYPE - 2`，以无符号比较拒绝高于 `0x22` 的结果，于
`0x65369184` 返回空指针。因此 0 和 1 均无法构造数组。该结论
对应安装介质携带的版本，尚无目标 Windows 运行取证。

Rust 对 0/1 与 9/13 数组分别给出明确错误。范围值域合成验证覆盖
未知类型、附加标志，以及第二个边界失败时回退游标和第一个值的新键。
未在真实 MXD 中发现这些数组分支。

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

### SystemCore 数组对象

AfCore 10.1.0.3035 的 `Variant::Read` 在 VARTYPE 9/13 把原始流交给
`LoadObject 0x68a7b6d0`，并在 `0x68a7ca4f` 传入 `0x68c6bc2c`
的 IUnknown IID。ObjectStream 路径根据类标识进行对象分派，
因此同一个对象头可以绑定 SystemCore 的已确认数组类，体中没有额外的
Variant VARTYPE。SystemCore.dll 10.1.0.3035 SHA-256 为
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。

| 类 | CLSID | 原生 Load / Save | 元素 |
| --- | --- | --- | --- |
| StrArray | `A7F92065-36CE-47B6-A463-4763DA947CC2` | `0x80042aa0 / 0x800422f0` | AfCore String，读取委派 `0x80040930` |
| LongArray | `98BFB808-E91F-11D2-9F81-00C04F8ECE27` | `0x80042aa0 / 0x800422f0` | LONG，读取委派 `0x800407c0` |
| DoubleArray | `60C06CA7-E09E-11D2-9F7B-00C04F8ECE27` | `0x800419b0 / 0x80041a80` | double，读取委派 `0x80040520` |

三个对象体均先读 short 版本和有符号 LONG 计数。Rust 复用原有三类数组
读取逻辑，保留对象头类标识、会话键以及 double 的原始位模式。内联
SAFEARRAY 仍由 VARTYPE 的数组标志决定。带保留区及不带保留区的
ObjectStream 宿主共享上述对象分派，未知类与对象数组仍显式停止。

构造数据在栅格类别中读取 StrArray 并从 UniqueValues 回引，在
CodedValueDomain 中读取非空 LongArray。DoubleArray 覆盖两种 Variant
外壳、负零、NaN 载荷、字节限额及键回滚。尚无实际 MXD 在上述 Variant
宿主中出现这些数组类的证据。

### SystemCore 嵌套数组

在同一份 SystemCore.dll 10.1.0.3035 上追踪 RTTI 至 IPersistStream
虚表，核对 GetClassID 的常量和逐元素的虚调用目标。类体均以
short 版本和有符号 LONG 计数开头，Save 写版本 1。

| 类 | CLSID | 持久化虚表 | Load / Save |
| --- | --- | --- | --- |
| VarArray | `762F0474-ECA2-475B-99F4-26678D79436E` | `0x8007c52c` | `0x80043990 / 0x80043a60` |
| VarStrArray | `485708E7-6EBD-415F-AA56-7CD1782AECD1` | `0x8007c794` | `0x80042aa0 / 0x800422f0` |
| PropertySetArray | `C94BBD8A-EC33-4921-8EC3-6AD4B33232C3` | `0x8007c630` | `0x800443c0 / 0x80044bd0` |

- VarArray 经逐项读取函数 `0x80040ed0` 在 `0x80040f2c` 压入 false，
  再调用 IAT `0x8006b2e0` 的 AfCore Variant::Read。写侧
  `0x80040e30` 同样用 false。因此即使外壳 Variant 无 14 字节保留区，
  每个 VarArray 元素仍按带保留区解析。
- VarStrArray 从完整对象虚表 `0x8007c744+0x28` 进入 `0x80040930`，
  复用 StrArray 的逐元素 AfCore String 读取。
- PropertySetArray 的逐项函数 `0x800448b0` 在 `0x8004490b` 通过
  LoadObject 载入对象，随后在 `0x80044923..0x8004492b` 请求 IPropertySet
  `F0BA6ABC-8E9F-11D0-B4AB-0000F8037368`。写侧 `0x80044120`
  经查询后在 `0x80044187` 调 SaveObject。Rust 读取非空 PropertySet 或
  同类回引，拒绝无法进入该 QI 的空槽。PropertySetPropertyArray 的
  类标识和 QI 均不同，尚无可用的非空逐元素体证据。

构造用例在 VarArray 中读取 PropertySetArray，新 PropertySet 内再读取
无保留区的 VarStrArray，随后分别验证属性集、字符串数组的同键回引。
嵌套数组沿用 PropertySet 的深度限额，未知类、截断和超限时回退对象值游标
及该值登记的全部键。目前未在 MXD 样本或 ArcMap 中确认这些嵌套数组的宿主。

### ValueArray 与 UID

GdbCore.dll x86 10.1.0.3035，SHA-256
`ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`。
注册代码 `0x70a81376..0x70a8139f` 将 `0D78BBB6-AD8B-4F25-A9F5-4436EDF9D217`
绑定到工厂虚表 `0x70ac248c`，工厂 `0x7090c710` 调构造 `0x70a12240`。
后者安装主虚表 `0x70ad89bc` 和完整对象 `+0x10` 处的持久化虚表
`0x70ad8a10`。QI 的 `0x70a1574d..0x70a157eb` 将 IPersistStream
与 IPersist 映射到该位置。GetClassID `0x70a15280` 从 `0x70ad8fec`
返回同一 CLSID。

Load/Save `0x70a152c0 / 0x70a15390` 按 short 版本与 LONG 计数开始，
正计数随后进主虚表的逐元素槽。普通 ValueArray 的读槽 `+0x28`
指向 `0x70a5aab0`，写槽 `+0x24` 指向 `0x70984a50`，两段指令
均返回 `0x80004001`。未对可能覆盖这些槽的派生类作布局推断。

按原址映射 GdbCore 和 AfCore 的原生指令，以 QEMU i386 执行对象体
Load/Save。驱动提供初始化的对象内存、内存 IStream 和固定堆缓冲。
空 Save 输出 `01 00 00 00 00 00`。Load v1 和改写版本字的 v0 都返回
`S_OK`，结束位置为 6，后随字节未消费。v2 在位置 2 返回
`0x80004005`。计数 1 的 Save 在写完 6 字节头后从实际主虚表槽返回
`0x80004001`。Rust 接受 v0/v1 且计数为零的对象体，保留外壳键。
该实验未启动 ArcMap 或触发完整文档保存。

Variant 分派同时复用 [SystemCore UID 对象体](mxd-boundary-luna.md)，
保留版本、标志、GUID 和最后一个 int32。构造 VarArray 验证 UID
后的第二个键头按回引消费，以及 ValueArray 超版本、非零计数、UID
回引截断时整个数组及其新键回滚。暂无含这些 Variant 对象值的实档宿主。
参考工程及两套介质的 134 份工程均读至 `/Maps` 与 `/PageLayout` 终点，
两条流均未命中 ValueArray CLSID。未做 ArcMap 或 QGIS 人工验收。

### SystemCore 的通用 Array

同版 SystemCore 的 `8F2B6061-AB00-11D2-87F4-0000F8751720` 由
`0x7ffb6040` 从 `0x8006d810` 取出。主 QI `0x7ffb6290` 的四个直接
IID 分支均无 IPersistStream，默认调用 `0x800533e0`，后者只匹配
IUnknown 和两个框架接口。完整对象 `+0x18` 的虚表 `0x8006d784`
在 `0x7ffb6540 / 0x7ffb65b0` 经外部序列化接口处理 BSTR、计数和
逐项对象。该入口无 MXD 二进制流读写依据，通用 Array 仍停在未知类分支。

### Geometry.dll 的有键几何对象

复核 PE32 10.1.0.3035 的 AfCore、SystemCore 和 Geometry.dll，摘要分别为
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`、
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` 和
`c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`。
AfCore `0x68a7ca4d` 将 VT_DISPATCH 和 VT_UNKNOWN 的对象值都传给原流的
LoadObject，请求 `0x68c6bc2c` 的 IUnknown。SystemCore
`0x7ffcff70` 在 `0x7ffd0236` 登记新键，再调用对象的持久化入口。
保留区只改变对象头在流中的位置。

Point、Multipoint、Polyline、Polygon、Envelope、GeometryBag 和 MultiPatch
已有各自的 GetClassID、Load 和 Save 证据，见
[几何实现清点](mxd-round4-map-spatial-20261117.md#101-的-igeometry-类体清点)。
Point、Multipoint、Polyline 与 Polygon 另有
[QEMU 原生 Load 记录](arcmap-10.1-geometry.md)。
共享 Variant 读取器调用同一几何体分派，保留类型化几何和实际消费的字节，
按原编码比较坐标位模式。GeometryBag 各成员及几何内的空间参考共用宿主键表。
裁切或非法体出错时，外层几何和所有暂存子键均回滚。

构造 RasterUniqueValueRenderer v4 验证类别中新 GeometryBag 的 Point
成员可在 UniqueValues 中回引，并核对后继统计值、颜色坡道和逐字节截断。
构造 CodedValueDomain 验证新 Point 的原始体、负零坐标和后继边界，
以及任一字段截断时的整个值域回滚。样本中这两类宿主的非空几何值仍待验证，
ArcMap/QGIS 人工验收尚未进行。
参考工程及两套安装介质共 135 份工程的 /Maps 和 /PageLayout 均读至末尾。

### Geometry.dll 的有键空间参考对象

对仓外提取的 PE32 x86 10.1.0.3035 原件复核版本资源和 SHA-256。
Geometry.dll 为 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`，
AfCore.dll 为 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，
GdbCore.dll 为 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`。
以 `objdump` 核对 GdbCore 的 RangeDomain::Save 在 `0x70935482` 和
`0x7093548e` 把两个边界值交给 AfCore Variant::Write(false)，
CodedValueDomain::Save 在 `0x708d5599` 经 IAT `0x70a873bc`
写出循环中的编码值。AfCore 的 VT_DISPATCH/VT_UNKNOWN 读写分支分别在
`0x68a7ca58` 和 `0x68a7ce26` 调用 LoadObject/SaveObject，
所以对象体由流内 CLSID 选择，并沿用原流的键表。

Geometry.dll 三类的持久化入口由虚表原字节核对。

| 类 | 持久化虚表 | GetClassID / Load / Save |
| --- | --- | --- |
| ProjectedCoordinateSystem | `0x71a29ccc` | `0x719d2b30` / `0x719d3230` / `0x719d3690` |
| GeographicCoordinateSystem | `0x71a26818` | `0x719bc190` / `0x719bec60` / `0x719bf1d0` |
| UnknownCoordinateSystem | `0x71a2c320` | `0x719eab80` / `0x719ecd80` / `0x719ed310` |

Rust 的 ObjectStream Variant 分派复用普通空间参考槽的三类读取器，
保留键、类型化对象体和原始字节。合成 RangeDomain 先读一个空间参考
Variant，再读一个内部回引该键的 Envelope Variant，覆盖三类、
结束位置、截断和未知类时的整域回滚。重新读取 134 份 10.1/10.8
介质工程和仓库参考工程，135 份的 `/Maps`、`/PageLayout` 均完整
消费，两种值域的 CLSID 字节均无命中。实际值域编辑、对象比较行为和
真实宿主字节仍待取证，未做 ArcMap/QGIS 人工验收。空间参考的字段和版本差异见
[各类持久化接口](arcmap-10.1-pcs-geometry-slots.md)。

### RasterDB 的统计、直方图与色表对象

复核 Desktop 10.1.0.3035 PE32 RasterDB.dll，SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`。
持久化虚表、读写入口和类标识互相对位：

| 类 | CLSID | IPersistStream 虚表 | GetClassID | Load / Save |
| --- | --- | --- | --- | --- |
| RasterStatistics | `8446EB83-3251-4F20-8B79-1D5421E39D4A` | `0x7c1346ec` | `0x7c0cb900` | `0x7c0cb940 / 0x7c0cba90` |
| RasterHistogram | `B01293EA-88D0-4E7D-AF82-A0B10BAC9B33` | `0x7c134184` | `0x7c0c82a0` | `0x7c0c82e0 / 0x7c0c8400` |
| RasterColormap | `92195722-5732-11D2-AAAD-00C04FA375FB` | `0x7c131fec` | `0x7c09b6e0` | `0x7c09cdb0 / 0x7c09cf00` |
| StatsHistogram | `BA3027C1-49CA-4788-8B5A-3A6B387DE78C` | `0x7c136850` | `0x7c100650` | `0x7c101910 / 0x7c101b30` |

前三类复用 [RasterInfo 的字段序](mxd-10.1-image-server-rules.md#rasterinfo-二进制对象体)。
版本比较分别在 `0x7c0cb96f`、`0x7c0c8312`、`0x7c09cdf1` 读取
有符号 short，Rust 保留原位模式。三类的内嵌 Variant 均使用 AfCore
的 `Variant::Read(false)`（RasterDB 导入槽 `0x7c1212a0`）与同一原流。
StatsHistogram 复用 [渲染器统计体](mxd-10.1-raster-renderer.md#rgb-的两个嵌套对象)
的 int32 版本和分版本桶数组。

两种已确认的有键 Variant 外壳均可分派这四个对象体。新键及内嵌键共用
宿主对象表，递归的色表、直方图、统计、属性集与 Variant 数组共用深度限制；
回引只消费对象头，失败不提交暂存状态。统计值按 double 位模式比较。
构造 RasterInfoFunction 和 MergeRasters 内层流验证四类对象嵌套、
后继字段、NaN 位模式和色表键回引。交替嵌套色表与 VarArray 的输入另行验证
深度限制、截断、超版本及回滚。实际宿主中的该类 Variant 值仍待实档或
ArcMap 运行取证，ArcMap/QGIS 人工验收尚未进行。
135 份对照工程的 /Maps 和 /PageLayout 均读至末尾。270 条流中仅一条命中
StatsHistogram 的 CLSID，前三类零命中。CLSID 扫描未用来判定 Variant 宿主。

## DisplayCore 颜色对象

带保留区的 Variant 对象分支读出 CLSID 和会话键后，颜色体沿用 IColor
槽的分派。DisplayCore.dll 10.1.0.3035 的 RgbColor、HsvColor、GrayColor
及 HlsColor 共用 Load 入口 0x6eb3d8b0 和核心 0x6eb35310，CmykColor
使用 0x6eb34850。各类身份及字段顺序见[颜色持久化取证](arcmap-10.1-display-tails.md)。
构造的 VT_UNKNOWN/RgbColor 覆盖逐字节截断回滚、键登记和后继字节边界。
参考工程已有独立颜色槽实例，颜色作为值域或栅格类别 Variant 的实际宿主尚未验证。

## DisplayCore 颜色渐变对象

复核 DisplayCore.dll 10.1.0.3035，SHA-256 为
`9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。
本地 Ghidra 导出中的四个 Load 为 Random `0x6eb3c190`、MultiPart
`0x6eb395b0`、Preset `0x6eb3ae20`、Algorithmic `0x6eb316c0`。
其 CLSID、Save 及普通对象体布局见 [四类渐变的原始取证](arcmap-10.1-display-slots-round3.md)。
AfCore 的普通 ObjectStream Variant 对象分支通过对象头分派持久化接口。
因此复用已有渐变体读取器。MultiPart 的渐变槽和后两类的颜色槽沿用
同一引用表；未知子类使整个 Variant 回滚。补齐 Preset 和 Algorithmic
向外层传播内部停止的路径，防止后继读取误把未知颜色对象头当作本对象尾段。

合成输入覆盖组合渐变内嵌算法渐变、颜色回引、非默认权重、后继字节定位、
逐字节截断和未知颜色回滚。未取得栅格统计或值域实际写出渐变 Variant 的
宿主字节，未在 ArcMap 10.1 或 QGIS 目视验证。

## SystemCore 数字格式对象

复用 [SystemCore.dll 10.1.0.3035 的九类 INumberFormat 持久化证据](arcmap-10.1-number-format.md#九类分派接入建议写给整合轮本不改-srclayers)。
例如 NumericFormat 的 Load/Save `0x7ffce4b0/0x7ffce590` 从标准
ObjectStream 对象头后处理版本及 22 字节字段。Variant 的 VT_UNKNOWN
和 VT_DISPATCH 沿用本页已对位的 AfCore LoadObject 路径，因此数字格式
对象可复用同一套 CLSID 分派与已有版本门。数字格式体按字节预算读取，
保留原始体和宿主键，后继回引不再读取对象体。

合成验证覆盖带保留区的 NumericFormat 新对象、同键回引、后继边界和
逐字节截断。九类的各自对象体沿用既有读取器及测试。栅格唯一值、
统计集合和值域中的数字格式实档实例，以及 ArcMap/QGIS 人工验证待补。

## 栅格函数与参数对象

标准 AfCore 带保留区和紧凑 Variant 的对象分支共用 ObjectStream，
因此可按 CLSID 复用已有原生证据的 IRasterFunction 和
IRasterFunctionArguments 读取器。类标识、持久化虚表和版本门见
[ArcMap 10.1 规则取证](mxd-10.1-image-server-rules.md)。
新分派保留原始对象体并累计跨属性集和栅格参数的嵌套深度。
合成输入覆盖 IdentityFunction 及 CachedRasterFunctionArguments
和 PropertySet 交替嵌套时的限额与回滚。实际栅格类别或值域中的
此类对象仍待实档取证，ArcMap/QGIS 目视验证待补。
