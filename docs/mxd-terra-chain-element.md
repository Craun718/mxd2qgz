# ElementProperties 与 TextSymbol 字段对位（2026-11-17）

本轮以 [ElementProperties 字段序](arcmap-10.1-element-bodies.md)及 [TextSymbol 既有对位](mxd-terra-integrate-text.md)为起点，新增 PE 导出、虚表和 x86 指令静态对照。地址为下列 10.1 文件的静态 VA，结论是存储对位，不是运行观察。原有公开字段名与读取行为不变；未知值和停止点仍原样保留。

## 目标身份与方法

重新计算 `temp/agent-fl-tail/LayerLib.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` SHA-256 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`，以及 `DisplayCore.dll.33A02325_2485_4623_B2CE_71B5736B51DF` SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`，均与既有元素体记录一致。两者 PE Machine 均为 `0x14c`，资源 FileVersion/ProductVersion 为 `10.1.0.3035`。注意前者 CAB 条目名为 LayerLib.dll，但其版本资源的 InternalName/OriginalFilename 实际为 `LayerUI`/`LayerUI.DLL`；后者资源为 `DisplayCore`/`DisplayCore.dll`。这些是文件字节和资源声明，不证明签名真实性或安装映射；地址仅绑定对应摘要。

本轮用 pefile 读取导出、虚表和常量，以 objdump 读取指令，并复用先前 10.1 esriDisplay 类型库 dump；不存在源码不足而需新建反编译会话的情况，没有连接、切换或重置共享 REA。类型库经 Wine 解析及其输出限制见 [前次记录](mxd-terra-follow-display.md)，下述 TextSymbol 槽号以该 dump 的 `oVft/2` 为 x86 偏移。

## ElementProperties

设默认构造对象起点为 `O`，IPersistStream 接口指针为 `P`。导出构造函数 `0x786b95a0` 在 `O`、`O+4`、`O+8`、`O+0xc` 分别安装导出虚表 `IElementProperties` `0x7871f0b0`、`IElementProperties2` `0x7871f0ec`、`IElementProperties3` `0x7871f12c`、`IPersistStream` `0x7871f178`；后者 +0x14 为直接导出的 Load `0x786b97f0`，所以该构造布局中 `P=O+0xc`。对应的接口 2/3 转发 thunk 对参数分别减 4/8；与直接接口方法的接收基址不可混同。Load 以 `P` 为基址，直接读入的偏移与访问器相减如下：

| 现有字段 | Load 落点 | 属性访问器及其基址 | 对位 |
| --- | --- | --- | --- |
| `name` | `P+8 = O+0x14` | get `0x786981c0`、put `0x786981f0`，`O+0x14` | Name |
| `type_description` | `P+0xc = O+0x18` | get `0x786b97a0`、put `0x78698220`，`O+0x18` | Type |
| `custom_property` | `P+0x14 = O+0x20` | get `0x786982e0`、put `0x786b97d0`，`O+0x20` | CustomProperty |
| `extra_short` | `P+0x10 = O+0x1c` | get `0x78698250`、put `0x78698280`，`O+0x1c` | AutoTransform 原始 16 位 |
| `reference_scale` | `P+0x2c = O+0x38` | IElementProperties2 get `0x78697ec0`、put `0x78697ef0`，接口指针 `O+4` 再加 `0x34` | ReferenceScale |
| `tail_int32` | `P+0x34 = O+0x40` | IElementProperties3 get `0x78697f50`、put `0x78697f30`，接口指针 `O+8` 再加 `0x38` | AnchorPoint 原始 32 位 |

前三对及 AutoTransform 为具名导出且直接位于接口 1 虚表；接口 2 的 ReferenceScale 为其虚表 +0x30/+0x34，接口 3 的 AnchorPoint 为 +0x38/+0x3c。Load `0x786b9838/0x786b983e/0x786b985c/0x786b98d6/0x786b98ef/0x786b9929` 分别提供上述存储位置；版本门见已有记录。旧版读后丢弃的 int32 不在此表。`ReferenceScale` setter 有非负检查，而 Load 直接读入：不将 setter 的约束强加给流。AutoTransform 的原始 short 不归一化；AnchorPoint 的枚举值意义和显示行为未在此闭合。

## TextSymbol

沿用 [前次记录](mxd-terra-integrate-text.md)经 QI、构造函数和 Load thunk 建立的本类关系：接口 `I=O+0x98`，持久化 Load 接收基址 `P=I+0x4bc`，接口虚表 `0x6ec1643c`。本轮读取类型库对应的 getter 槽、查看其实际指令并与 Load 读入位置比较：

| 现有字段 | 访问器与实际存储 | Load 落点及结论 |
| --- | --- | --- |
| `color2` | ShadowColor getter 槽 +0x50 → `0x6ebcad90`，取 `I+0x13c` 后委派 | `P-0x380`，第二 IColor 槽（`0x6ebd0c65`） |
| `text_path` | Background getter 槽 +0xb0 → `0x6ebcb230`，取 `I+0x198` | `P-0x324`，第一个对象槽 `0x6ebd0e3b` 请求 ITextPath；同一存储的接口属性名为 Background，不能由名字推断具体实现类 |
| `symbol1` | FillSymbol getter 槽 +0xc0 → `0x6ebcb450`，对 `I+0x1a0` 的对象 QueryInterface | `P-0x31c`，第二个对象槽 `0x6ebd0e84` 请求 ISymbol；getter 会查询接口，不等于所有 ISymbol 都是有效 FillSymbol |
| `name_string` | Text getter 槽 +0x44 → `0x6ebcc1d0`，取 `I+0x284` | `P-0x238`，String `0x6ebd0ea2` |
| `font` | Font getter 槽 +0x0c → `0x6ebcb700`，取 `I+0x40` 并委派 | `P-0x47c`，IFontDisp 槽 `0x6ebd0f60` |

Color getter 槽 +0x1c → `0x6ebcb5a0`：当 `I+0x1a0` 有对象时，通过该对象的接口取色；否则把 `I+0x3c=P-0x480` 的对象交给辅助虚调用。因此第一 IColor 槽确实参与 Color 获取，但不能断言 Color 始终等于该槽。对 getter 的辅助调用、引用计数和运行中对象转换尚未完整追踪。`text_path` 和 `symbol1` 原有字段名不变，以上对位不改变槽位读取、类型分派或停止策略。

## 可勾范围与开放项

可单独闭合：**ElementProperties 的 Name、Type、CustomProperty、AutoTransform、ReferenceScale、AnchorPoint 存储与 10.1 Load 静态对位**；**TextSymbol 的 ShadowColor、Background、FillSymbol、Text、Font 存储与 10.1 Load 静态对位**。Color 仅闭合上述条件依赖关系，不能勾全部颜色行为。TextSymbol 其他未映射标量、版本尾字段、枚举值、辅助调用和非空对象的实际行为继续开放。既有 17 项标量对位见前次记录，不重复计为本轮新增。字段偏移对位不能代替真实样本、ArcMap 10.1 与 QGIS 的打开和目视验证；本轮没有真实运行环境，均未执行。
