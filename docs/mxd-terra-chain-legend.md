# ArcMap 10.1 Legend 集合 short 与 Horizontal 包装字段续查

2026-11-17，接续 [图例标量对位](mxd-terra-depth-legend.md)和 [Legend 尾段](arcmap-10.1-legend-tails.md)。本轮为静态分析，未运行 ArcMap 或 QGIS，也未人工打开和检查工程。

## 文件身份与方法

本轮重算 `temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 的 SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`；PE machine `0x14c`、ImageBase `0x793a0000`，固定版本资源为 `10.1.0.3035`，与前轮身份一致。这是文件自述和字节身份，不证明签名或运行状态。下列地址为该 PE 的静态映像 VA。

前轮独立 Ghidra 桥接工具 CLI 的 Ghidra 桥接 Unix socket 因 `Operation not permitted` 失败，见前轮记录。本轮没有切换共享 Ghidra 桥接工具 会话：使用 JDK 25、Ghidra 12.2 / pyghidra，复制前轮 Map 工程到独占 `temp/terra-chain-legend/`，设置仓库内独立 Java 临时目录、配置及缓存，重新反编译 Load、Save 和集合清理方法；同时从本轮重验的 PE 读取虚表、用 objdump 核查调用指令。Ghidra 伪代码不是原始源码；临时产物可能含本机路径，分享前须检查脱敏。

## Legend 集合前 short：读后未使用

- Load `0x793da250` 中 `0x793da427..0x793da436` 将 2 字节读入栈临时位置，检查读取结果；`0x793da441..0x793da458` 从对象起点虚表的 `+0xa8` 取函数，仅压入对象指针后调用，没有压入该 short。之后读取 4 字节条目计数。此处更正前轮及代码中“传入集合接口”的表述。
- 该类起点虚表 `0x7948556c` 的 `+0xa8` 项为 `0x793d79d0`。该方法逐项调用接口 `+8`，按需释放集合缓冲并清零指针、计数及容量：静态上是清理原有条目。这个调用与 short 的值无关；短读处理仍取决于 `AfStreamRead` 的返回值。
- Save `0x793da650` 的 `0x793da74e..0x793da760` 将栈上 `0xffffffff` 的低 2 字节写出，再写集合计数和对象。故对该 10.1 实现可描述为**读后未使用、写出固定 `0xffff` 的保留字段**；没有证据为历史版本文件赋予业务名或要求输入必须为该常量。读取器继续保留所有 16 位位模式，不改变解析行为或公开字段名称。

## HorizontalLegendItem 包装 int32：流向相关的内部布局模式

续查时从用户重新提取的 `temp/terra-chain-renderers/Renderers.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` 重算 SHA-256 为 `399c80c99a6c61de120c21b518d9b29069f2c78b20a705c07178aa3e2b656274`；PE x86、ImageBase `0x7cac0000`，固定版本及 FileVersion/ProductVersion 均为 `10.1.0.3035`，InternalName 为 `Renderers`，OriginalFilename 为 `Renderers.dll`。这是文件自述及字节身份，不是签名或运行验证。使用 JDK 25、Ghidra 12.2 / pyghidra 在该目录新建独立工程、配置、缓存和 TMPDIR，定点反编译并以 PE 虚表、IID 和 objdump 指令交叉核对；未操作共享 Ghidra 桥接工具。原始结果位于忽略目录，分享前须检查本机路径。

令构造 `0x7cad4c70` 的对象指针为 O。配置表 `0x7caec4d4` 的第二项为 `0x170`，构造在 `O+0x14+0x170 = O+0x184` 安装持久化虚表，并在 `O+0x180` 写入调整值 0。Load thunk `0x7cad5d30` 减去该调整值后，Load `0x7cad50e0` 读取相对持久化指针 `−8` 的 int32，故字段位于 **O+0x17c**；构造 `0x7cad4d6a` 在此写 0。此换算也更正 [尾段记录](arcmap-10.1-legend-tails.md) 对此构造布局持久化指针写作 `this+0x174` 的表述：该值不能直接当作相对 O 的持久化指针，读取字段时应计入配置表所在的 `+0x14`。Save `0x7cad5150` 从同一位置回写。

- QI `0x7cad4f10` 对 IID `{7D23F970-D5A6-4C1C-8618-3CC7B5839EFA}` 返回 O+0x1c；该接口虚表 `0x7caeb06c` 的 `+0x84` 为 `0x7cad5b50`。前轮 esriCarto 10.1 类型库中相同 IID 的 `ChangeFlowDirection(RightToLeft)` 槽位为 64 位导出偏移 264，折算 x86 为 `+0x84`。
- `0x7cad5b50` 把参数的低 16 位按非零与否转换，分别向接口指针 `+0x160` 写 5 或 0；`O+0x1c+0x160 = O+0x17c`。随后它调用 LayerLib 导出的 `BaseLegendItem::ChangeFlowDirection`，仍传入原始 short 参数。另一个继承接口的虚表 `+0x84` 经 thunk `0x7cad5d80` 减 4 后进入同一方法。
- 对象 O+0x18 所安装的内部虚表 `0x7caeaf64` 中 `+0xf8/+0xfc` 分别为 `0x7cad5360/0x7cad51b0`。两函数以 O+0x18 为 `this`，分别在 `0x7cad56ca` 和 `0x7cad52aa` 读取 `this+0x164 = O+0x17c`，前者对 0..5 作跳转表分派，后者对 2、4 单独选择计算分支；控制流和浮点坐标运算表明该字段参与图例项布局计算。

因此可将其定性为**受图例流向变更影响、参与布局计算的内部模式值**。证据支持该方法写 0/5，不足以给 0..5 各值建立完整名称，也不能把文件中的其他 int32 值当成布尔值或拒绝它们；不据 0/5 推断最终界面呈现。当前读取器保留原值，未改变解析行为。后续可追踪其他潜在写入路径和各分支的图形布局，并在 ArcMap 10.1 中以可控图例核验。

## 可闭合范围及验证边界

可独立闭合 **Legend 集合前 short 在已核对的 10.1 Load/Save 中的处理语义**，以及 **Horizontal 包装 int32 与流向和布局计算的关联**。若条目要求完整的各模式值业务含义，则 Horizontal 的标量业务语义仍不能整体勾选；Legend 共同前缀亦仍有未知字段。未修改 TODO。

本轮修改 `map_surround` 的三处注释以修正调用关系，保留现有字段和行为。`cargo fmt --check`、完整 `cargo test` 和 `git diff --check` 通过；Cargo 无权清理用户级全局缓存，但未影响测试。合成测试及静态分析不能替代 ArcMap 10.1 和 QGIS 中的人工验证。

Renderers 续查仅更新本记录，没有修改 Rust 代码或新增解析规则；检查文件身份、反编译结果及上述 PE 指令与虚表，不以静态分析宣称目标软件运行结果。

## 续查：Arrangement 枚举已定名（2026-11-18）

上一节的「仅能定性为内部模式」是当时未对到属性和枚举的阶段结论。
横排与纵排包装 int32 都是 `Arrangement`。其值决定图例样块、标签和描述的顺序。
以下以 10.1 静态文件为依据，未在 ArcMap 或 QGIS 中检查视觉效果。

1. **类型库定名。** 介质 `esriCarto` 10.1 TYPELIB 的 SHA-256 为
   `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。
   直接解读 MSFT 的 TypeInfo 418/419，分别是 `IHorizontalLegendItem`
   （`8A7A1786-79B2-11D2-87D5-0000F8751720`）和 `IVerticalLegendItem`
   （同前缀、首字段末位为 7）。两者 `+0x0c` 是 `get_Arrangement`，
   `+0x10` 是 `put_Arrangement`。参数类型描述 `0xf50` 指向 TypeInfo 395
   （类型引用 `0x9a4c`）的 `esriLegendItemArrangement`，getter 使用其指针类型
   `0xf58`。枚举成员记录起于流内 `0x105374`，数值取自成员记录中的常量。
2. **持久化字段对位。** 复核上一节所列 Renderers.dll 的 SHA-256、x86 架构与
   10.1.0.3035 版本。原始指令中，横排 QI `0x7cad4f10` 命中属性接口 IID 后
   在 `0x7cad4f35` 返回完整对象 O+`0x178`；构造 `0x7cad4d45` 向该处装
   `0x7caeb22c` 虚表。纵排 QI `0x7cae18c0` 同样返回 O+`0x178`，构造
   `0x7cae16f5` 装 `0x7caec49c`。两表的 get/put 共用
   `0x7cae15b0/0x7cae1590`，分别读取和写入接口再加 4，即 O+`0x17c`。
   两类的 Load/Save 使用持久化视图 O+`0x184` 减 8，与同一成员对位。
3. **分支复核。** `objdump -d -Mintel` 横排入口 `0x7cad5360` 在
   `0x7cad56dd` 按原始值跳转到下表地址。纵排 `0x7cae1f47` 对同一枚举
   使用自己的坐标计算表。任意不在 0..5 范围内的 int32 仍能原样持久化；
   横排计算对此走 `0x7cad588b` 的缺省路径，不能据此把未知字节当成 0。

| 原值 | 10.1 枚举常量 | 顺序 | 横排分支 |
| --- | --- | --- | --- |
| 0 | `esriPatchLabelDescription` | 样块、标签、描述 | `0x7cad56e4` |
| 1 | `esriPatchDescriptionLabel` | 样块、描述、标签 | `0x7cad57b6` |
| 2 | `esriLabelPatchDescription` | 标签、样块、描述 | `0x7cad571c` |
| 3 | `esriLabelDescriptionPatch` | 标签、描述、样块 | `0x7cad575b` |
| 4 | `esriDescriptionPatchLabel` | 描述、样块、标签 | `0x7cad57ee` |
| 5 | `esriDescriptionLabelPatch` | 描述、标签、样块 | `0x7cad5833` |

横排 `ILegendItem2.ChangeFlowDirection` 仍按上一节的证据把枚举改写为 0 或 5。
宽度计算 `0x7cad51b0` 单列 2 和 4，是样块位于两段文本之间的两种排列。
类型库还独立解释了为什么 0 与 5 互为反序。Rust 对两类公开同一枚举的读取接口，
原字段保留全部位模式；合成输入覆盖六个枚举值和未知值。
