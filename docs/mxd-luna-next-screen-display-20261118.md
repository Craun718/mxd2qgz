# Map 遮罩后 IsFramed 的静态取证（2026-11-18）

研究对象为 ArcMap 10.1.0.3035 x86 的 Map.dll，SHA-256
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。
依据 10.1 Map 现有反编译、原始 PE 指令和同版 esriDisplay 类型库。地址均为
静态映像 VA。ArcMap 与 QGIS 未运行。

1. Map 构造 `0x793e2820` 初始安装另一组表，随后在 `0x793e2a49` 把完整
   对象的 `+0x28` 写为 `0x79486f14`，在 `0x793e2abb` 把 `+0x68` 写为
   `0x794874a4`（IPersistStream）。PE 原始表和 MSVC RTTI
   `.?AVMap@@` 的两个 COL 偏移均吻合。
2. Map Load `0x793ff4d0` 和 Save `0x79401850` 接收 IPersistStream，这两个
   函数的 `this-0x40` 因而正是完整对象的 `+0x28`。Load
   `0x794008b0..dd` 读两个字节，调用实际表的 `+0x60`（`0x793efaf0`）；
   Save `0x79402487..a8` 调用实际表的 `+0x5c`（`0x793efac0`），写两个
   字节。两个实体都从接口接收者 `+0x11c` 取内层接口，Load 调它的
   `+0xc8`，Save 调它的 `+0xc4`。内层接口实际在完整对象 `+0x144`。
3. 构造在 `0x793e2c70..7c` 把完整对象 `+0x144` 地址传入
   `0x794174f0`；该函数使用 AfCreateInstance 创建 ScreenDisplay
   CLSID `{E6BDB100-4D35-11D0-98BE-00805F7CED21}`，再 QI
   `{E6BDB004-4D35-11D0-98BE-00805F7CED21}` 并将输出存入原地址。
   CLSID 由 10.1 类表定名，IID 由 esriDisplay 类型库定名为 IScreenDisplay。
4. 10.1.0.3035 安装介质的同版 esriDisplay MSFT TLB（本地文件
   `temp/terra-follow-display/display.tlb`，SHA-256
   `6052021b77c05ddb6b37b5373a04ada586a6664a121733ddf0651297106229ab`）的
   typeinfo 39：函数 21 和 22 分别是 `0xc4` 的 property get（`VT_BOOL*`）
   和 `0xc8` 的 property put（`VT_BOOL`），共用 memid `0x60020015`
   与 NameTab 偏移 `0x1948`，直接读取名称为 `IsFramed`。由此闭合流字段的
   业务含义，撤销按外层槽位与 IMap 类型库相同数值对应 `MapSurround` 的猜测。

参考工程位于 `/Maps:0x253be` 的 short 原始值为 `0xffff`（VARIANT_TRUE）。
Rust 模型保持原始值、偏移和版本门；未改变样本游标。该定名不推断字符串或
之后的 short、字节属于同一属性，也不推断屏幕显示与 QGIS 布局转换等价。
