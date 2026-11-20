# TextSymbol 持久化属性续查（2026-11-17）

本轮从 [既有访问器对位](mxd-terra-integrate-text.md) 和 [颜色等槽对位](mxd-terra-chain-element.md) 出发，只研究 ArcMap 10.1 TextSymbol；静态地址不代表运行观察。重新计算 `temp/agent-fl-tail/DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，PE machine `0x14c`，版本资源 InternalName/OriginalFilename 为 `DisplayCore`/`DisplayCore.dll`，FileVersion/ProductVersion 均为 `10.1.0.3035`；与既有记录和 README 介质版本声明一致，不能由此推断签名真实性。类型库为此前从同介质提取的 esriDisplay 资源，重新计算 SHA-256 `6052021b77c05ddb6b37b5373a04ada586a6664a121733ddf0651297106229ab`，`winedump` 报头版本 10.1、SYS_WIN32。

以独立 Ghidra 桥接工具 6.9.0 CLI、Ghidra 12.2、JDK 25、仓库独占 TMPDIR 对同一 DLL 的 `0x6ebcc310` 发起反编译；run `d4e712aa-6de4-4a61-87de-68b5e4e86ab3` 的 Ghidra 桥接 Unix socket 因 `Operation not permitted` 失败，无本轮 Ghidra 桥接工具 函数 Evidence。随后将已有 DisplayCore 项目复制到本轮目录，以隔离 pyghidra、`/opt/ghidra`、JDK 25 定点反编译 Load 与相关访问器，得到新的静态伪代码；未更改共享目标。以下由原 PE 字节、objdump 指令、类型库 `winedump` 和新伪代码交叉核对；伪代码仅作为辅助，不是源码或运行结果，间接调用以指令为准。临时诊断可能包含本机路径，分享前须脱敏。

## IMask 三项新增对位

`O` 是构造对象首地址，`P=O+0x554` 为既有构造布局下 Load 接收指针。QI `0x6ebcaa79` 比较 PE 地址 `0x6ec0f7c4` 的 IID `{6A7EF984-6924-11D2-980D-0080C7E04196}`，匹配返回 `M=O+0xa0`；构造 `0x6ebc9eea` 在 M 安装虚表 `0x6ec16588`。同哈希类型库 GUID 条目 244 指向 TypeInfo 210，名称表标识 IMask，函数记录按 x86 偏移 +0x0c/+0x10、+0x14/+0x18、+0x1c/+0x20 分别为 MaskStyle、MaskSize、MaskSymbol 的 get/put。故不是按字段值或接口顺序猜测。

| 流内字段 | 实际虚表 get / put | 存储及 Load 对位 | 结论 |
| --- | --- | --- | --- |
| `int_4` | `0x6ebcc310` / `0x6ebcc2d0` | `M+0x2a4 = P-0x210`，Load `0x6ebd0ef3` 读 4 字节 | MaskStyle 原始枚举位模式；不限制数值 |
| `double_6` | `0x6ebcc380` / `0x6ebcc340` | `M+0x2b0 = P-0x204`，Load `0x6ebd0f17` 读 8 字节 | MaskSize 原始 double；不将接口行为强加给流 |
| `symbol2` | `0x6ebcc400` / `0x6ebcc3b0` | `M+0x2a8 = P-0x20c`，Load `0x6ebd0f3c` 请求 ISymbol | MaskSymbol 底层对象槽；getter 对非空对象还执行接口查询，流槽不等同于接口调用结果 |

`TextSymbolBody10_1` 增加同名小写方法读取这三个原始结果，既有公开字段和读取器不变；为避免早停时把未到达字段的初始化值误当成流值，只有 `stop=None` 才返回 `Some`。这也有意保守地屏蔽了部分早停前已读到的属性；对象槽的空引用仍是 `Some(Null)`。这不模拟 COM getter、对象 QueryInterface、显示结果或 setter 副作用。

## 颜色：补充 setter 与条件路径

两槽的静态角色已有上轮对位，本轮重读两侧而不把它们视作独立可直接渲染颜色。设 `I=O+0x98`，`P=I+0x4bc`：

- Color：Load `0x6ebd0ae2` 的 `P-0x480=I+0x3c` 为第一 IColor 存储；getter `0x6ebcb5a0` 在 `I+0x1a0`（FillSymbol 底层存储）为空时把该槽传入 `I-0x80` 虚表 +0x40 辅助调用；非空时经 `0x6eb44b60` 和所得对象虚表 +0x0c 取色。setter `0x6ebcb4d0` 清理并通过辅助调用重设第一槽，若 FillSymbol 底层非空还经同一辅助函数和所得接口 +0x10 传色。这里不认定辅助函数的所有转换和引用语义。
- ShadowColor：Load `0x6ebd0c65` 处第二 IColor 与 `P-0x380=I+0x13c` 对位；getter `0x6ebcad90` 把它交给同一 +0x40 辅助虚调用，setter `0x6ebcadc0` 清理该槽后经 +0x2c 设置。两种 getter 均可能涉及辅助行为，不把槽值直接等同完整 COM 返回值。

可闭合两颜色的**持久化槽与 getter/setter 存储及 Color 条件分支静态对位**；不能闭合辅助实现的所有失败、对象转换或 ArcMap 实际显示行为。

## 其他原始数值与版本门

- `doubles_1[1]` / `[2]`：Load `0x6ebd0bee` / `0x6ebd0c0a` 分别向 `P-0x44c` / `P-0x444` 读 double。本轮未找到具名访问器的充分对位，保留未知；没有依据零样本值命名。
- `tail_short_1`：版本 ≥2 时 Load `0x6ebd0fbd` 读 `P-0x4e4=O+0x70`。构造 `0x6ebca42b..0x6ebca463` 从进程全局配置候选值生成 0 或 0xffff 写入该位置；这不证明属性名称，也不限制版本 ≥2 的流内原始值。
- `tail_short_2`：版本 ≥4 时 Load `0x6ebd1070` 读 `P-0x384`；版本 <4 时 `0x6ebd10a9..0x6ebd10da` 从字体槽经虚表 +0x4c 取得句柄，调用导入 `GetObjectW`，比较结果结构中的一个 word 与 `0x40` 后写入 0 或 0xffff。只证明旧版本的静态回填依赖字体及该结构字段，未独立闭合字段的业务含义或所有失败路径；读取器按流版本保留 `None`，不模拟运行时回填。
- `ints_1[0]`、`bytes_1[0]`、`ints_2` 和其余 GUID 的业务含义、剩余两个 double、两个 short 的具名属性及对象动态行为仍未完成。既有 17 项标量、Text/Font/Background/FillSymbol 对位见前述记录，不重复算成本轮新增。

## 验证与可勾范围

可独立勾：IMask 的 MaskStyle、MaskSize、MaskSymbol **三个存储对位**；Color/ShadowColor **槽及访问器存储对位与上述条件分支**。尾部 short 仅补充旧版本回填与初始化证据，尚不能勾业务语义；TextSymbol 全部持久化属性业务语义仍未闭合。`tests/element_body.rs` 对完整体的三个方法及未知符号停止时保守 `None` 进行断言；`cargo fmt --check`、`cargo test` 通过（依赖安装介质的测试按原配置忽略）。Cargo 报全局缓存自动清理权限不足警告，不影响测试退出状态。ArcMap 10.1 暂不可用，未在 ArcMap 或 QGIS 中打开工程、未验证显示和转换效果。

## 文字背景、填充符号和版本尾段复核

2026-11-20 从既有 10.1 DisplayCore 和 DisplayLib 伪代码起步，复核
Desktop 原件的 x86 指令及 10.1 esriDisplay 类型库。DisplayCore.dll 的
SHA-256 与本页开头一致。DisplayLib.dll 的版本资源为 10.1.0.3035，
SHA-256 为 `e16c4c2610f20896005cf7ae56965dd60a61530b27daa3d566682f6ac11982c6`。
以下 `O` 为 TextSymbol 完整对象地址，Load 与 Save 的接口地址为
`O+0x554`。

- Load `0x6ebd0e1b` 将字段 9 的类标识交给 `0x6eb49c00`。
  后者激活类并请求 ITextPath，IID 位于 `0x6ec0d4fc`。Save
  `0x6ebd068a..0x6ebd06a8` 取该对象的持久化类标识。此字段无对象键。
- Load `0x6ebd0e36` 请求的 `8FEB6611-2A0D-11D1-9A44-0080C7EC5C96`
  经同版类型库确认为 ITextBackground，消费字段 10 的带键对象。类型库列出
  BalloonCallout、LineCallout、SimpleLineCallout 和 MarkerTextBackground
  四个实现。其持久化体已有 [原生对位](arcmap-10.1-display-slots-round3.md#二callout文字背景族证据强度确定样本-0-实例)。
  Rust 复用四类解析器，空值与回引按普通对象流处理。
- 紧随的 ISymbol 对象槽进入 FillSymbol 的底层存储。Rust 复用已有符号分派，
  两个符号槽与背景共用会话键。经符号或背景递归再次进入 TextSymbol 时，使用
  会话上的图形嵌套限额。保留原有空值与回引公开字段，新解析的对象由
  `background` 和 `fill_symbol` 提供。

| 流内字段 | 字段身份和取证 |
| --- | --- |
| `ints_1[0]` | BreakCharacter。构造在 `O+0x9c` 安装 ISimpleTextSymbol 表 `0x6ec1650c`，get/put `0x6ebcb810/0x6ebcb7e0` 访问视图 +0x40，等于 `O+0xdc`；`0x6ebcda10` 把该值交给 DisplayLib 的 ConvertBreaksToReturns。 |
| `bytes_1[0]` | Clip。该表 get/put `0x6ebcb960/0x6ebcb990` 访问视图 +0x50，等于 `O+0xec`。 |
| `doubles_1[1..=2]` | 绘制 X/Y 偏移输入。 `0x6ebcc901..0x6ebcc94a` 将 `O+0x108/+0x110` 分别乘设备比例，Y 分量翻转符号，舍入并保存到 `O+0x118/+0x11c`。源字段单位及接口名称待核。 |
| `ints_2[0]` | ROP2。DisplayCore 在 `O+0x18` 构造 DisplayLib 的 Symbol 基类。其导出 get/put `0x6ed95300/0x6ed952d0` 访问基类 +0x40，等于 Load 的 `O+0x58`。 |
| `tail_short_1` | RotateSymbolsOnConvert 设置。构造 `0x6ebca42b..0x6ebca463` 缓存 HKCU 的 `SOFTWARE\ESRI\ArcMap\Settings\Annotation` 下同名整数（缺省 1），将非零转成 0xffff 写到 `O+0x70`。v2 起由存盘 short 覆盖。 |
| `tail_path_clsid` | ITextParser 类标识。v3 起，`0x6ebd1004..0x6ebd104f` 对零 GUID 清除成员，对非零值调用 `0x6ebe22d0` 请求 ITextParser（IID 常量 `0x6ec16868`），成员与 ITextParserSupport getter `0x6ebcc4b0` 同址。v0–v2 无该 GUID，Load 使用 SimpleTextParser 的 CLSID。 |
| `tail_short_2` | CJKCharactersRotation。构造 `0x6ebc9f4e` 在 `O+0xc8` 安装 ICharacterOrientation 表 `0x6ec166a4`，put/get `0x6ebdee00/0x6ebdee20` 访问 +0x108，与 Load 的 `O+0x1d0` 同址。v4 直接读取。 |

v0–v3 的旧版回填在 `0x6ebd10a9..0x6ebd10da`。对字体句柄调用
GetObjectW，缓冲区长 0x5c。比较字位于缓冲区 +0x1c，
由 LOGFONTW 布局可确定为 lfFaceName 的第一个 UTF-16 码元。
等于 `@` 时写入 0xffff，否则写入 0。Rust 仅返回流中存在的原始 short，
没有用存盘字体名模拟 Windows GDI 的字体映射。
[微软 LOGFONTW 定义](https://learn.microsoft.com/en-us/windows/desktop/api/wingdi/ns-wingdi-logfontw)

`ints_2[1]` 与 IMapLevel.MapLevel 对位，见下节。其他运行行为和真实背景、填充符号宿主仍需实档或人工验证。

合成 TextElement 覆盖四种文字背景、非空 FillSymbol、MaskSymbol 回引、
后继文本字段、未知背景在对象头回退及外层截断回滚。全量含忽略项 1103
个 Rust 测试通过。参考工程和 134 份安装介质工程的 /Maps 和 /PageLayout
均读至流尾，流中未扫描到这四个背景 CLSID。ArcMap 10.1 与 QGIS
人工验收尚未进行。

### 文字偏移的源单位

沿用上节同哈希 DisplayCore 与 DisplayLib。DisplayCore
`0x6ebcc510` 使用 `O+0x18` 的 Symbol 基接口调用
DisplayLib 的 SetupDC `0x6ed94f70`，并在 `0x6ebcc520`
还原到完整 TextSymbol 指针，进入绘制准备 `0x6ebcc570`。
SetupDC 调 `0x6ed94500` 获取设备分辨率，先查询
IDisplayTransformation `{E6BDB000-4D35-11D0-98BE-00805F7CED21}`，
分辨率不可用时走 `0x6ed96aa0` 的 GDI 分辨率路径，其常规分支
`0x6ed96b7c` 调 GetDeviceCaps(LOGPIXELSY)。

`0x6ed94fd3` 将分辨率除以 72.0（`0x6edb9500`），写到
Symbol +0x30，即 TextSymbol +0x48。显示变换有效时
`0x6ed95084..0x6ed95093` 还按当前显示与参考比例缩放。
`0x6ebcc901..0x6ebcc963` 乘该比率、翻转 Y 分量并舍入。这把
两个持久化 double 的输入单位对位到排版点（1/72 英寸）。
`offsets_in_points()` 返回原始 X/Y 值。合成测试分别检查非零正 X、
负 Y 和读取结束位置。

DisplayLib 的构造、复制构造和赋值分别在 `0x6ed94d10`、
`0x6ed94680`、`0x6ed94860` 对 Symbol +0x48 初始化为 0 或原样复制。
SetupDC 和 ResetDCW 的旧 ROP2 缓存经 `0x6ed94ff0`
对位到 Symbol +0x44。软件人工验收仍待进行。

### Symbol 基类成员与旧版本短尾

复核本页同 SHA-256、版本 10.1.0.3035 的 x86 DisplayCore.dll 和
DisplayLib.dll。TextSymbol 的持久化 Load 在 `0x6ebd0c48`、
Save 在 `0x6ebd04f0` 取 `O+0x554-0x4f4` 的地址，均读写一个
int32。该地址为完整对象 `O+0x60`，即 Symbol 基类 `+0x48`。
旧记录将访问器的 `+0x3c` 误读为 Symbol 基类偏移。实际访问器从 IMapLevel 接口指针起算，证据链如下。

- DisplayCore `0x6ebc9dfb..0x6ebc9e04` 传 `O+0x18` 给导入的 Symbol 构造函数。DisplayLib 构造指令 `0x6ed94d7f` 在 Symbol `+0x0c` 安装导出的 IMapLevel 虚表 `0x6edb7c2c`。
- 同版 esriDisplay 类型库的 TypeInfo 255 将 IID `CEF72580-C1D9-11D2-9888-0080C7E04196` 定名为 IMapLevel，`+0x0c/+0x10` 分别为 MapLevel 的 get/put。TextSymbol 在 `0x6ebca9af` 比较该 IID（PE 常量 `0x6ec0ca5c`），命中在 `0x6ebca9cd..0x6ebca9d0` 返回 `O+0x24`。
- TextSymbol 构造指令 `0x6ebc9ecf` 在 `O+0x24` 安装类自己的虚表 `0x6ec16424`。表中 `+0x0c/+0x10` 指向 `0x6eb61450/0x6eb99f40`，两处均将原接口参数直接交给导入的 `Symbol::get_MapLevel` / `put_MapLevel`，导入槽分别为 `0x6ec0a644/0x6ec0a648`。
- DisplayLib getter `0x6ed954c6` 和 setter `0x6ed9548e` 读写接口相对 `+0x3c` 的 int32。因此该地址为 `O+0x24+0x3c=O+0x60`，也就是 Symbol 基类 `+0x48`，与 Load/Save 的存盘地址一致。

Rust 保留原始字段 `ints_2[1]`，新增 `map_level()` 读取已完整消费对象的存盘值。合成用例覆盖非零有符号值及早停行为。ArcMap 和 QGIS 的人工验收未执行。

TextSymbol::Load 在 `0x6ebd0ada..0x6ebd0ae0` 用有符号
`cmp/jg` 判定版本上限 4，尾部三处
`0x6ebd0fb0`、`0x6ebd0fda`、`0x6ebd1061` 也按有符号值判断。
Rust 保留原始 u16 位模式，用 i16 选择布局。合成输入覆盖 v0–v4、
-1 和最小负 short，检查三个可选字段和后继字节；v5 仍在对象起点回退。
负版本的实际写入来源及 ArcMap/QGIS 人工验收待补。
