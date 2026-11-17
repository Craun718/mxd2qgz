# TextSymbol 与 NumberFormat 属性续查（2026-11-17）

范围仅为 TextSymbol、NumberFormat 的属性定性。本轮新增类型库成员枚举及 NumericFormat 构造、QI、虚表与访问指令对照；没有修改解析器、公共入口或 TODO。静态证据不是 ArcMap 实际运行结果。

## 文件身份与方法

目标以 README 的 ArcGIS Desktop 10.1.3035 为准。重新计算 `SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF` SHA-256 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，`DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` 为 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`；与此前分别核对的 PE x86、FileVersion/ProductVersion 10.1.0.3035 一致，见 [数字格式续查](mxd-terra-next-format.md) 和 [显示续查](mxd-terra-next-display.md)。资源声明及摘要不等同签名认证。

esriSystem 的 TYPELIB 资源 SHA-256 `0f3bc6496f2665cd6b03fa0071c0077f3b6fdab8f96c8b80f02beec36d7ad7a8`，与 [10.1 介质身份记录](arcmap-10.1-native-baseline.md)一致；esriDisplay `.olb` 和提取资源的 SHA-256 分别为 `5f64ecc7511ca3a8174bab83a57f550d3703a5c96131262771ad1a2e78ad3660`、`6052021b77c05ddb6b37b5373a04ada586a6664a121733ddf0651297106229ab`，来自该记录所述 10.1 介质提取目录。借用已有 `memberdump`，在仓库专用 `temp/terra-follow-display/` 的独立 Wine 前缀和 TMPDIR 中通过 `LoadTypeLibEx` 读取类型库；这是 Wine 对静态元数据的解析，不是 ArcMap 运行。64 位 Wine 返回的 `oVft` 除以 2 与 x86 四字节虚表槽对位。工具进程输出成员后返回码 3，类型名称的宽字符打印乱码；下列成员名称经 UTF-8 转换正常输出，IID 则与已有类型库清单互证，不把退出码当成完整枚举成功的证明。原输出在该专用目录，分享前需检查本机路径。

原生实现部分使用既有 REA 6.9.0 / Ghidra 12.1.4 SystemCore 全量导出的 `functions/0x7ffce4b0.c`、`0x7ffcdb20.c`、各访问器伪代码及新执行的 PE 数据读取、objdump 指令复核；这些伪代码不是原始源码。未启动或更改共享 REA 服务器。默认沙箱中独立 Wine 因 Unix socket bind 被拒，随后使用独立授权进程完成枚举。

## NumericFormat 核心：七个流字段与访问器

类型库 `INumericFormat` IID `{7E4F4710-8E54-11D2-AAD8-000000000000}` 的七对属性按顺序为 RoundingOption、RoundingValue、AlignmentOption、AlignmentWidth、UseSeparator、ZeroPad、ShowPlusSign，每对均先 put 后 get。下面为新做的三方对位，适用此 10.1 实现：

- `0x7ffcdb20` 初始化 NumericFormat 时将 `0x80070e3c` 安装到对象基址 `O+0x1c`；`0x7ffcdc90` 比较该 IID 常量 `0x80070eec` 后返回 `O+0x1c`，所以该虚表是该类的 INumericFormat 接口。构造函数的虚基表 `0x80070ee4` 记录偏移 `0x34`，持久化接口位于 `P=O+0x14+0x34=O+0x48`；虚表 `0x80070ec4` 的 Load thunk `0x7ffcea50` 经 `P-[P-4]` 调用 `0x7ffce4b0`。在这个构造布局下 `[P-4]=0`，该 Load 的参数即 P；这项算术限定在本类该构造布局，不直接套用其他变体。
- 类型库的 x86 槽从 `+0x0c` 开始每对占 8 字节。逐槽读取 `0x80070e3c` 并对照访问器读写，再与 Load 参数相对偏移核对如下；偏移 O 与字段号均不包含版本 short。

| 流序 | 类型 | 业务属性（接口名） | 内存 O 偏移 | put / get 静态 VA |
| --- | --- | --- | --- | --- |
| 1 | int32 | RoundingOption | +0x2c | `0x7ffcdd80` / `0x7ffd3320` |
| 2 | int32 | RoundingValue | +0x30 | `0x7ffcdda0` / `0x7ffc1e10` |
| 3 | int32 | AlignmentOption | +0x34 | `0x7ffcddc0` / `0x7ffcc290` |
| 4 | int32 | AlignmentWidth | +0x38 | `0x7ffe2e90` / `0x7ffcc480` |
| 5 | int16 | UseSeparator | +0x3c | `0x7ffcdde0` / `0x7ffcde00` |
| 6 | int16 | ZeroPad | +0x3e | `0x7ffcde30` / `0x7ffcde50` |
| 7 | int16 | ShowPlusSign | +0x40 | `0x7ffcde80` / `0x7ffcdea0` |

Load 依序写 `P-0x1c/-0x18/-0x14/-0x10/-0xc/-0xa/-8`，代入 P 后恰为上表。另以指令复核：RoundingValue setter `0x7ffcdda3..0x7ffcddbd` 只接受无符号比较下 ≤15 的值；AlignmentWidth setter 伪代码检查非负。Load 直接写字段，**不能**把 setter 限制当作流解析约束，也不能据此断定所有读入值合法。三个布尔属性的流为原始 16 位，类型库的 BOOL 不授权丢弃非规范位模式。枚举值的完整意义、格式输出及各派生类的接口 this 调整仍须另外验证。

## TextSymbol 与其余数字格式：本阶段边界

本轮类型库枚举确认 `ITextSymbol` IID `{A80B5E91-7F9C-11D0-9410-080009EEBECB}`，属性包括 Font、Size、Color、VerticalAlignment、HorizontalAlignment 和 Angle；`IFormattedTextSymbol` 还包含 CharacterWidth、WordSpacing、Kerning、Leading、Direction、FlipAngle、Background、TypeSetting、FillSymbol 等。此为**接口成员清单**，没有完成 TextSymbol 的 QI → 子虚表 → 调整 thunk → 成员地址与 Load 的逐项对位，因此这些名称不能赋给 [TextSymbol 字段序](arcmap-10.1-element-bodies.md#textsymbolload-0x6ebd0a60-字段序证据确定的样本逐字节互证) 中相似类型或样本值。对象槽的请求 IID 仍仅证明所请求接口，未证明全部显示语义。

同样，类型库 `IRateFormat` `{7E4F4717-8E54-11D2-AAD8-000000000000}` 列出 RateFactor（R8）和 RateString（BSTR），`ICustomNumberFormat` `{7E4F4718-8E54-11D2-AAD8-000000000000}` 列出 FormatString（BSTR）。已有 Load 明确 Rate 的 double 和 String、Custom 的 String，但尚未完成它们的 QI、虚表与访问器成员地址对位；故暂不把接口名称直接绑定对应流字段。未研究其余派生类的各自字段和核心 this 调整。

## 可勾范围与验证

- 可作为独立闭合项：**NumericFormat 核心七个持久化字段的 10.1 静态属性名与访问器对位**。这不闭合「NumberFormat 九类标量」整项，也不闭合字符串或任何运行验证项。
- TextSymbol 全部标量语义、NumberFormat 其他类及 Rate/Custom 字符串继续开放。下一步应先分别从类型库 IID 跟进 QI 和虚表，处理 thunk/虚基调整，再将 getter/setter 实际成员地址与 Load/Save 对位；必要时借助独立 REA 或隔离 pyghidra 补证。对行为、显示效果及实际对象字节仍需目标软件运行或宿主路径明确的真实样本。
- 本轮仅新增研究文档，未改 Rust，不需要因此运行 Rust 测试；`git diff --check` 通过（该命令不检查未跟踪文档）。未在 ArcMap 10.1 或 QGIS 打开工程，软件版本、转换效果和人工目视验收均未验证。
