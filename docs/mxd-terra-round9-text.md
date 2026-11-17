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
