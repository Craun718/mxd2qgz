# ArcMap 10.1 图例标量：类型库与访问器对位（2026-11-17）

接续 [页面续查](mxd-terra-next-page.md) 和 [Legend 尾段](arcmap-10.1-legend-tails.md)。此处所有地址是静态映像 VA；本轮没有运行 ArcMap，也没有在 ArcMap/QGIS 中打开或目视检查工程。原始 short 保留其 16 位位模式，不因接口类型为 VARIANT_BOOL 而限定文件只含 0 或 0xffff。

## 身份与方法

重算 SHA-256，并用 PE 固定版本资源复核均为 x86、10.1.0.3035：Map.dll `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`（ImageBase `0x793a0000`）、Layer.dll `fe52bd9988b66f9d6f906b014e5684f28fc20e9b97b53c90909fa3d44e9efed8`（`0x78650000`）、LayerLib.dll `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`（`0x78690000`）。版本资源是声明，哈希不证明签名。esriCarto TYPELIB SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`，头声明 SYS_WIN32、版本 10.1。

类型库通过隔离 Wine 前缀下 `LoadTypeLibEx` 静态读取；64 位 Wine 报告的 oVft 除以 2 才对应目标 x86 字节偏移。工具的宽字符串类型名乱码，按 IID 和正确输出的成员名识别，不能把导出过程当作 ArcMap 运行。独立 REA 6.9.0 CLI 绑定 Layer.dll 时，Ghidra 分析完成但 Unix socket 绑定返回 `Operation not permitted`，未切换共享 REA 目标；随后使用 JDK 25、Ghidra 12.2 / pyghidra，在 `temp/terra-depth-legend/` 独占项目、配置、缓存及 Java 临时目录反编译 Map 和 Layer 的 Load、QI、访问器，另用 objdump 检查机器指令。伪代码非原始源码。TLB 导出、脚本及汇编位于该忽略目录，分享前需检查本机路径。

## LegendClassFormat：两个 double

`ILegendClassFormat` IID `{52AA5BC2-CB21-11D0-87F9-080009EC732A}`；Layer.dll QI `0x786642d0` 命中 IID 地址 `0x7866f798` 返回对象 +0x10，虚表 `0x7866f86c`。IPersistStream 指针为对象 +0x14，Load `0x78664b20` 在版本 >1 时将两个 8 字节值读至该指针 +0x1c、+0x24。类型库 PatchWidth getter/put 的 x86 槽是 +0x1c/+0x20，PatchHeight 是 +0x24/+0x28；相应代码和 Load 位置换算如下：

| 流序 | getter / setter | 接口指针位移 | 持久化指针对应位移 | 属性 |
| --- | --- | --- | --- | --- |
| 第一个 double | `0x78664a60` / `0x78664a90` | +0x20 | +0x1c | PatchWidth |
| 第二个 double | `0x78664ac0` / `0x78664af0` | +0x28 | +0x24 | PatchHeight |

setter 对负值有检查，Load 直接读入，不据 setter 限制缩窄解析。版本 ≤1 无这两个流字段；不以默认内存值补造流值。符号和 patch 槽仍遵守原读取器的停止边界。

## HorizontalLegendItem：基类字段

本轮新核 LayerLib::Load `0x786a2b00` 汇编和 BaseLegendItem 的导出虚表及访问器；Horizontal 包装委派到该基类的既有定位见 [尾段记录](arcmap-10.1-legend-tails.md)。基类 IPersistStream 指针相对 BaseLegendItem 起点 +0x164；ILegendItem 虚表 `0x7871f8c4` 的 COL 位移 0，IDynamicLegendItem 虚表 `0x7871fb4c` 位移 +0x14。ILegendItem IID `{52AA5BC0-CB21-11D0-87F9-080009EC732A}`，IDynamicLegendItem IID `{815E4B50-152A-4B7B-B3E8-4778C59DC699}`；各属性均经 10.1 类型库槽序、导出虚表和对应 getter 指令与 Load 存储位置对位。

| 流字段（相对基类持久化指针） | 属性 | getter VA；接口指针相对位移 |
| --- | --- | --- |
| −0x12c / −0x12a / −0x128 / −0x126 | NewColumn / Columns / KeepTogether / ShowLayerName | `0x786a1e60` +0x38 / `0x786a1eb0` +0x3a / `0x786a1f00` +0x3c / `0x786a1f50` +0x3e |
| −0x120 int32 | GroupIndex | `0x786a2100` +0x44 |
| −0x11c | ShowHeading | `0x786a2150` +0x48 |
| −0x110 / −0x10e | ShowLabels / ShowDescriptions | `0x786a2370` +0x54 / `0x786a23c0` +0x56 |
| 版本 ≥2：−0xce / −0xcc | AutoVisibility / ShowCounts | 动态接口 `0x7869d230` +0x82 / `0x7869d1e0` +0x84 |
| 版本 ≥2：−0xc8 / −0xc4 | CountPrefix / CountSuffix | 动态接口 `0x7869d2d0` +0x88 / `0x7869d320` +0x8c |
| 版本 ≥2：−0xc0 | UseSimpleExtent | 动态接口 `0x7869d280` +0x90 |

例如 −0x12c + 0x164 = Base 起点 +0x38；动态接口 +0x14 +0x82 = Base 起点 +0x96 = 0x164−0xce。版本 <2 走原生默认分支，不能把默认值当流内字段。Horizontal 包装自己的 int32 尚无成员访问器对位，不推断名称；委派调用在具体派生类中的运行时安装、各标志实际呈现仍需目标运行验证。因本轮仅独占 `map_surround` 模块，`src/layers/legend_format.rs` 和 `src/layers/legend_item.rs` 未改，原数组和原始值仍可保留，以上属性映射供后续接线。

## Legend：字段对位

Legend QI `0x793d1f00` 对 IID `{15ABE7F6-49E8-4012-BE72-D14E5A755ACE}` 返回对象 +0xdc，虚表 `0x794857cc`；对 `{15ABE7F7-49E8-4012-BE72-D14E5A755ACE}` 返回 +0xe0，虚表 `0x7948585c`。持久化指针为对象 +0x1f4。类型库对应成员的 x86 槽序、虚表和 thunk 再与 Load `0x793da250` 对位：

| 流字段（相对持久化指针） | 属性或边界 | 访问器 |
| --- | --- | --- |
| −0xf0 / −0xee / −0xec | AutoAdd / AutoReorder / AutoVisibility | +0xdc 接口 thunk `0x793df1f0` / `0x793df130` / `0x793def50`，减 4 后分别跳 `0x793d73f0` / `0x793d7420` / `0x793d7470`，访问 +0x2c / +0x2e / +0x30 |
| −0xe8，版本 ≥3 | FlowRight 的内部反向标志 | +0xdc 槽 `0x793df060` → `0x793d74c0`，访问调整后 +0x34，getter 将内部值为零映射为真；原始 short 不是 FlowRight 的直接位模式 |
| −0xe6，版本 ≥4 | ScaleSymbols | +0xdc 接口 `0x793d8c20` 访问 +0x32 |
| −0x9c / −0x9a / −0x98 / −0x94，版本 ≥5 | FixedFrame / AutoColumns / AutoFit / MinFontSize | +0xe0 接口 `0x793d9190` / `0x793d9200` / `0x793d9280` / `0x793d92e0` 分别访问 +0x78 / +0x7a / +0x7c / +0x80；AutoColumns getter 还可能校正内存值 |

例：`0x1f4−0xf0 = 0xdc−4+0x2c = 0x104`。前缀的集合 short 仅证实写侧恒 `0xffff`，尚未证实业务名；int32 已由 Load 的循环和集合操作确认为条目计数。MapSurround 共同前缀自身的 short、旧版两个丢弃的 double 未定性。不能据静态 getter 宣称目标界面行为，也不能把样本或构造测试当作旧版 Legend v0/v1 的目标软件验证。

本轮只为 `src/maps/map_surround.rs` 已确认字段补充中文语义注释，不改变字段名、原始值、版本门和解析行为；未改 TODO。待完成：Horizontal 包装 int32、MapSurround 共同 short、Legend 集合 short 的成员级绑定，以及 ArcMap 10.1 和 QGIS 人工验收。

验证：`cargo fmt --check` 和完整 `cargo test` 通过；测试中本地安装介质依赖项按原设置忽略。Cargo 提示无权清理用户级全局缓存，但测试成功。测试只检查读取器契约，不是 ArcMap/QGIS 人工验收。
