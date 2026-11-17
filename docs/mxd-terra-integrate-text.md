# TextSymbol 属性与 Load 偏移续研（2026-11-17）

本轮对 [前次开放项](mxd-terra-follow-display.md) 做新的访问器与 Load 对位；只记录本轮新增证据。地址仅适用于下述文件，结论均为静态观察，不是 ArcMap 运行结果。

## 身份、接口与 this 换算

重新对 `temp/agent-fl-tail/DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` 计算 SHA-256 为 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，PE32 x86；本轮 pefile 读取资源声明 `DisplayCore.dll`、Esri ArcGIS、FileVersion/ProductVersion `10.1.0.3035`。该摘要与 [10.1 显示续查](mxd-terra-follow-display.md)一致，文件身份声明不构成签名认证。接口成员及虚表序来自同一 10.1 介质的 esriDisplay 类型库，详见该记录；x64 枚举的 oVft 除以 2 得到 x86 偏移。

本轮读取 PE 虚表和 objdump x86 指令，复核已有 REA/Ghidra 定位的 Load `0x6ebd0a60`；局部反汇编留在忽略目录 `temp/terra-integrate-text/`。没有连接或改变共享 REA 服务。下述 `O` 是构造函数 `0x6ebc9da0` 的对象基址，`I` 为接口指针，`P` 为 Load 实体的 this：

- QI `0x6ebca980` 比较 `IFormattedTextSymbol` IID 常量 `0x6ec15a64` 与 `ITextSymbol` IID 常量 `0x6ec0ce9c`，匹配均跳到 `0x6ebcad5a` 返回 `O+0x98`；构造代码 `0x6ebc9ed6` 正在该位置安装虚表 `0x6ec1643c`。两接口共用前缀；因此 `I=O+0x98`，类型库的 getter/put 槽次可对到该虚表，而不是依据相似值猜测。
- 构造代码 `0x6ebc9dd4` 安装 vbtable `0x6ec16708`，其第二项为 `0x540`；`0x6ebc9f6c` 在 `O+0x14+0x540=O+0x554` 放置 IPersistStream 虚表 `0x6ec166e8`，并在该指针前一字设置 `0x540-0x540=0`。Load thunk `0x6ebe5bd0` 从虚表指针减去前一字后跳到实体，因此此构造布局下 `P=O+0x554=I+0x4bc`。这是该构造实例的静态关系，不泛化到别的类。
- 例如 `0x6ebcae10` 读取 `I+0x140`，等于 `P-0x37c`，与 Load 的首个字段 8 double 同址。getter、setter 的地址来自虚表实际条目；setter 可能调用辅助例程或维护其他状态，不能将其取值限制套给直接读取流的 Load。

## 字段对位

以下原始字段名对应 `TextSymbolBody10_1`，偏移相对 Load 实体 `P`。序号的 put/get 顺序以类型库 `inv` 和虚表为准，表中列 getter / setter；BOOL 接口不意味着流内 byte 可以只保留 0 或 1。

| 字段 | 接口属性 | Load 偏移 | getter / setter |
| --- | --- | --- | --- |
| `ints_1[1]` | VerticalAlignment | -0x470 | `0x6ebcb750` / `0x6ebcb720` |
| `ints_1[2]` | HorizontalAlignment | -0x474 | `0x6ebcb7b0` / `0x6ebcb780` |
| `bytes_1[1]` | RightToLeft | -0x467 | `0x6ebcb9c0` / `0x6ebcb9f0` |
| `doubles_1[0]` | Angle | -0x45c | `0x6ebcb930` / `0x6ebcb900` |
| `group_8a.0[0]` | ShadowXOffset | -0x37c | `0x6ebcae10` / `0x6ebcae40` |
| `group_8a.0[1]` | ShadowYOffset | -0x374 | `0x6ebcae70` / `0x6ebcaea0` |
| `group_8a.1[0]` | Position | -0x364 | `0x6ebcaed0` / `0x6ebcaf00` |
| `group_8a.1[1]` | Case | -0x360 | `0x6ebcaf30` / `0x6ebcaf60` |
| `doubles_2[0]` | CharacterSpacing | -0x35c | `0x6ebcaf90` / `0x6ebcafc0` |
| `doubles_2[1]` | CharacterWidth | -0x354 | `0x6ebcaff0` / `0x6ebcb020` |
| `doubles_2[2]` | WordSpacing | -0x34c | `0x6ebcb050` / `0x6ebcb080` |
| `byte_2` | Kerning | -0x344 | `0x6ebcb0b0` / `0x6ebcb0e0` |
| `double_3` | Leading | -0x33c | `0x6ebcb110` / `0x6ebcb140` |
| `int_3` | Direction | -0x330 | `0x6ebcb170` / `0x6ebcb1a0` |
| `double_4` | FlipAngle | -0x32c | `0x6ebcb1d0` / `0x6ebcb200` |
| `byte_3` | TypeSetting | -0x320 | `0x6ebcb390` / `0x6ebcb3c0` |
| `double_5` | Size 相关存储 | -0x1ec | `0x6ebcb8d0` / `0x6ebcb840` |

Size getter 从 `I+0x2d0=P-0x1ec` 读取，但 Load 读入后还进行虚调用；该字段不是对最终显示字号行为的运行验证。RightToLeft getter 对 `I+0x55` 作 byte 零扩展，其 setter 的行为不等于流字节的限制。原始数组及公开字段名不变，只在注释标记已有证据；截断、停止时的默认值契约亦未改变。

## 可勾项与仍开放

可单独勾选：**TextSymbol 的 VerticalAlignment、HorizontalAlignment、Angle、RightToLeft、Size 存储及 12 个格式化标量字段的 10.1 访问器与 Load 偏移静态对位**。这不等于 TextSymbol 全部属性闭合。字体、Color、ShadowColor、Background、FillSymbol、Text 与其他标量、枚举含义、setter 间接调用的完整行为及版本门字段仍需单独取证；运行效果和 QGIS 转换没有因此得到验证。

未在 ArcMap 10.1 或 QGIS 中打开工程：没有授权的 ArcMap 实机环境，静态取证不能替代运行或人工目视验收。
