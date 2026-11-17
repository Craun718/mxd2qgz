# ArcMap 10.1 派生数字格式属性对位（2026-11-17）

本轮新增接口到持久化成员的静态对位；NumericFormat 七字段既有结果见 [前轮](mxd-terra-follow-display.md)，流顺序和版本门见 [续查](mxd-terra-next-format.md)。原始类型及结构字段名保持不变；BOOL、枚举和数值保留流内原始值。静态结论不等于目标软件运行观察。

## 身份和方法

对 `Setup1.cab` 提取的 `SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF` 重新计算 SHA-256：`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`；PE32 x86，版本资源 FileVersion、ProductVersion 为 `10.1.0.3035`，InternalName `SystemCore`、OriginalFilename `SystemCore.dll`。这是身份声明及字节摘要，不是签名认证。类型库使用前轮同一 esriSystem TYPELIB（SHA-256 `0f3bc6496f2665cd6b03fa0071c0077f3b6fdab8f96c8b80f02beec36d7ad7a8`）。

用 REA 6.9.0/Ghidra 12.1.4 既有全量反编译定位各构造、QI，再新读原 PE 的指令、虚表、IID 和虚基表；伪代码不是源代码。用独立 Wine 前缀在 `temp/terra-integrate-numeric/` 读取类型库，64 位 Wine 给出的槽偏移除二换算为 x86；类型库输出是静态元数据，不是 ArcMap 运行。普通沙箱 Unix socket 被拒，隔离进程在授权环境运行；未更改共享 REA 服务。表格的 O 是构造函数接收的对象基址，P 是该构造布局的 IPersistStream 指针。各持久化虚表的 Load thunk 先把传入 P 减 `[P-4]`；这些构造函数在 P-4 写入的值为零，因此下表 P 可直接用于对应 Load 的负偏移。此结论不外推到其他类或构造布局。

## 属性及内存对位

下表接口名称按类型库虚表顺序、原生 QI 对应 IID、构造函数安装的最终虚表和访问器指令逐项对位；不是根据同类型或名称推测。put/get 地址为映像 VA。`P-偏移` 是 Load 写入地址，括号为对象基址地址。

| 类 / QI、接口指针、最终虚表 | P | 流字段 → 属性；访问器 put/get |
| --- | --- | --- |
| Rate：`0x7ffd67f0` 比较 IID `{7E4F4717-8E54-11D2-AAD8-000000000000}`，返回 O+44；`0x80071d74` | O+5c | P-14=O+48 → RateFactor，`0x7ffd6860/0x7ffd6890`；P-c=O+50 → RateString，`0x7ffd68c0/0x7ffd68e0` |
| Custom：`0x7ffbbfb0` 比较 IID `{7E4F4718-8E54-11D2-AAD8-000000000000}`，返回 O+1c；`0x8006e3a0` | O+2c | P-8=O+24 → FormatString，`0x7ffbc070/0x7ffbc090` |
| Percentage：`0x7ffd0f10` 比较 IID `{7E4F4711-8E54-11D2-AAD8-000000000000}`，返回 O+44；`0x800713ec` | O+54 | P-c=O+48 → AdjustPercentage，`0x7ffd0f80/0x7ffd0fa0` |
| Fraction：`0x7ffc6320` 比较 IID `{7E4F4713-8E54-11D2-AAD8-000000000000}`，返回 O+1c；`0x8006fc04` | O+34 | P-10=O+24 → FractionOption，`0x80008b60/0x80008b30`；P-c=O+28 → FractionFactor，`0x7ffc63e0/0x7ffd7720` |
| Scientific：`0x7ffda030` 比较 IID `{D4F5C355-76B8-11D3-9FC3-00C04F6BC78E}`，返回 O+1c；`0x80072514` | O+2c | P-8=O+24 → DecimalPlaces，`0x7ffda0f0/0x80008b30` |
| Angle：`0x7ffb30b0` 比较 IID `{7E4F4716-8E54-11D2-AAD8-000000000000}`，返回 O+44；`0x8006d344` | O+58 | P-c=O+4c → AngleInDegrees，`0x7ffb3150/0x7ffb3170`；P-a=O+4e → DisplayDegrees，`0x7ffb31a0/0x7ffcc8e0` |
| LatLon：`0x7ffc94f0` 比较 IID `{7E4F4714-8E54-11D2-AAD8-000000000000}`，返回 O+44；`0x800701bc` | O+80 | P-2c=O+54 → ShowDirections，`0x7ffc95e0/0x7ffc9600`；P-2a=O+56 → IsLatitude，`0x7ffc9630/无`；P-c=O+74 → ShowZeroMinutes，`0x7ffc97e0/0x7ffd7a00`；P-a=O+76 → ShowZeroSeconds，`0x7ffc9800/0x7ffc9820` |

LatLon 的 ILatLonFormat2 有 IsLatitude getter，但 QI 返回 O+48，虚表 `0x800701ec` 该槽 `0x7ffc9650` 读接口指针+0xe，即同一 O+56；不能把它当成 O+44 虚表的 getter。LatLon v4 的 P-8=O+78 暂无完成的属性对位，继续原样保留。Currency 的 Load 仅版本字段，不能推断其未持久化状态。

Rate 虚基表 `0x80071db4` 的第二项为 0x48，构造函数 `0x7ffd6630` 安装持久化虚表 O+0x14+0x48；Load thunk `0x7ffd6b80` 到 `0x7ffd6a10`。RateFactor 的 setter 拒绝非正值；Load 直接读 double，不能把 setter 限制作为流校验。Custom 虚基表 `0x8006e3ec` 第二项为 0x18，构造 `0x7ffbbe10`，Load thunk `0x7ffbc6a0` 到 `0x7ffbc4d0`。其字符串属性读写与 Load 均使用同一 O+24 String 成员，不表明任意格式串有效。

其余虚基表第二项依次为 Percentage `0x80071424` = 0x40、Fraction `0x8006fc58` = 0x20、Scientific `0x8006e3ec` = 0x18、Angle `0x8006d39c` = 0x44、LatLon `0x80070270` = 0x6c；持久化接口均从 O+0x14 加该项计算。Scientific setter 限制非负，而 Load 直接读 int32，故不对流施加该限制。内嵌 Numeric 核心保持独立读取，以上对位不推断其在所有版本中的运行状态。

## 可勾范围和未闭合

可独立勾选：Rate 的 RateFactor/RateString、Custom 的 FormatString、Percentage 的 AdjustPercentage、Fraction 的 FractionOption/FractionFactor、Scientific 的 DecimalPlaces、Angle 的 AngleInDegrees/DisplayDegrees，以及 LatLon 上述四个属性的 **10.1 静态持久化成员对位**。LatLon v4 最后一个 short、各派生类完整运行行为及目标软件验收仍开放，不能以此勾选覆盖全部字段或运行验证的大项。未修改 TODO。

未在 ArcMap 10.1 或 QGIS 中打开工程，因无已授权 ArcMap 环境，真实对象、显示效果和转换结果均未进行人工验证；类型库解析与反汇编不替代运行取证。

验证：`cargo fmt --check`、完整 `cargo test`、`cargo test --test number_format`（19 项）和指定源码的 `git diff --check` 通过。Cargo 提示无权自动清理全局缓存，未影响测试。此次仅补充字段注释，没有改变读取行为；构造输入测试不证明真实 ArcMap 保存结果。
