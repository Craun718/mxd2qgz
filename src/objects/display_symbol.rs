//! ArcMap 10.1 DisplayCore.dll 符号族对象体的严格读取子集与符号槽位分派。
//!
//! 第一轮（`docs/arcmap-10.1-display-tails.md`）闭合 SimpleMarkerSymbol/
//! SimpleLineSymbol/SimpleFillSymbol/MarkerLineSymbol/CartographicLineSymbol/
//! HashLineSymbol 六类；第二轮（`docs/arcmap-10.1-display-symbols-round2.md`）
//! 闭合 MultiLayerLine/MultiLayerMarker/MultiLayerFill/CharacterMarker/
//! PictureMarker/PictureLine/ArrowMarker/LineFill/MarkerFill/GradientFill/
//! DotDensityFill 十一类符号与 Template/LineDecoration/SimpleLineDecorationElement
//! 三个装饰 coclass。
//!
//! 证据来源：上述两份研究记录。全部字段序来自 DisplayCore.dll 10.1.0.3035
//! （SHA-256 `9b3e61f9…a926e4d2`，复用前轮提取件 temp/agent-fl-tail/）的静态
//! 反编译；类身份经 RTTI 类型描述符 → 完整对象定位符（COL，判据 `[C]==0` 且
//! `[C+0xc]==TD`）→ 虚表 → GetClassID 立即数逐类确认（虚表/实体地址见各读取
//! 器文档），读写两侧对称闭合；接口定名经 esriDisplay 类型库 dump 复核
//! （ITemplate `{7914E5F4}`、ILineDecoration `{533D88F0}`、
//! ILineDecorationElement `{533D88F2}`、IMarkerBackground `{AE41E4F0}`、
//! IColorRamp `{BEB87091}`、IFontDisp/IPicture 为 OLE 标准接口）。
//!
//! **重要边界：仓库参考样本 `samples/测试底图.mxd` 只含 SimpleMarkerSymbol
//! 与 RgbColor 实例；本模块第二轮闭合的全部类无样本字节佐证**；只能用合成
//! 输入测试，不得声称样本验证。
//!
//! ## 第一轮六类字段序（详见 docs/arcmap-10.1-display-tails.md）
//!
//! - **SimpleLineSymbol**（IPersistStream 虚表 `0x6ec15498`，Load thunk
//!   `0x6ebbda20` → 实体 `0x6ebbced0`，Save `0x6ebbd9f0` → `0x6ebbcfc0`，
//!   GetClassID `0x6ebbd090`）：
//!   ① short 版本（>1 拒绝 `0x80004005`）→ ② LoadObject（IColor
//!   `{20CD40B0-…}`，`DAT_6ec0b4f4` → `this−0x2c`）→ ③ `FUN_6eb406f0`
//!   （前轮已证不读流，返回值装入 `this−0xc`，无流内字节）→ ④ double
//!   （`this−0x24`，宽度候选）→ ⑤ int32（`this−0x28`，样式候选）→
//!   ⑥⑦ int32 ×2（`this−0x7c`/`this−0x74`，语义未定）。
//! - **SimpleFillSymbol**（虚表 `0x6ec152ac`，Load `0x6ebbc650` →
//!   `0x6ebbb4e0`，Save `0x6ebbc6c0` → `0x6ebbb5e0`，GetClassID
//!   `0x6ebbb6a0`）：
//!   ① short 版本（>1 拒绝）→ ② LoadObject（ISymbol `{F3435802-…}`
//!   `DAT_6ec0ca2c` → Save 槽 `this−0x24`，轮廓线符号候选）→ ③ LoadObject
//!   （IColor `DAT_6ec0b4f4` → Save 槽 `this−0x28`，填充色候选）→
//!   ④ `FUN_6eb406f0`（不读流）→ ⑤⑥⑦ int32 ×3（`this−0x7c` 样式候选/
//!   `this−0x74`/`this−0xc`）。
//! - **MarkerLineSymbol**（虚表 `0x6ec0eca0`，Load `0x6eb636a0` →
//!   `0x6eb62810`，Save `0x6eb63630` → `0x6eb629e0`，GetClassID
//!   `0x6eb61410`，版本常量 `DAT_6ec0b2e0`=2）：
//!   ① short 版本（>2 拒绝）→ ② byte（`this−0x20d8`）→ ③ double
//!   （`this−0x20d4`）→ ④ LoadObject（IMarkerSymbol `{E6BDAA7C-…}`
//!   `DAT_6ec0ca3c`，Save 槽 `this−0x20e0`）→ ⑤ `FUN_6eb68fc0`（本内
//!   释放/重建辅助，反编译无 AfStreamRead，无流内字节）→ ⑥ LoadObject
//!   （ITemplate，Save 槽 `this−0x20cc`；第二轮起 Template 体闭合）→
//!   ⑦ LoadObject（ILineDecoration，Save 槽 `this−0x20c8`；同上）→
//!   ⑧⑨ int32 ×2（`this−0x2134`/`this−0x212c`）→ ⑩ byte（`this−0x20ac`）→
//!   ⑪ double（`this−0x20a4`）→ ⑫ 版本 ≥2：int32 ×2（`this−0x2094`/
//!   `this−0x2090`）+ double（`this−0x208c`）。Save 恒写版本 2（含 ⑫）。
//! - **CartographicLineSymbol**（虚表 `0x6ec11ca8`，Load `0x6eb99e80` →
//!   `0x6eb99140`，Save `0x6eb99e00` → `0x6eb99330`，GetClassID
//!   `0x6eb99100`，版本常量 `DAT_6ec0b27c`=1）：
//!   ① short 版本（>1 拒绝）→ ②③ int32 ×2（`this−0x10a8`/`−0x10a4`）→
//!   ④ double（`this−0x109c`）→ ⑤ byte（`this−0x1094`）→ ⑥ double
//!   （`this−0x108c`）→ ⑦ LoadObject（IColor → Save 槽 `this−0x10ac`）→
//!   ⑧ `FUN_6eb406f0`（不读流 → bool `this−0x104c`）→ ⑨ LoadObject
//!   （ITemplate → `this−0x1084`）→ ⑩ LoadObject（ILineDecoration →
//!   `this−0x107c`）→ ⑪⑫ int32 ×2（`this−0x10fc`/`−0x10f4`）→
//!   ⑬ byte（`this−0x1060`）→ ⑭⑮ double ×2（`this−0x105c`/`−0x1054`）。
//! - **HashLineSymbol**（虚表 `0x6ec0e0b8`，Load `0x6eb56f30` →
//!   `0x6eb554a0`，Save `0x6eb56e90` → `0x6eb556d0`，GetClassID
//!   `0x6eb53c30`，版本常量 `DAT_6ec0b2f4`=1）：
//!   ① short 版本（>1 拒绝）→ ② double（`this−0x2124`）→ ③④ int32 ×2
//!   （`this−0x2118`/`−0x2114`）→ ⑤ double（`this−0x2104`）→ ⑥ byte
//!   （`this−0x20fc`）→ ⑦ double（`this−0x20f4`）→ ⑧ LoadObject（ISymbol
//!   `{F3435802-…}`，Save 槽 `this−0x20e4`）→ ⑨ LoadObject（IColor，Save
//!   槽 `this−0x211c`）→ ⑩ `FUN_6eb406f0`（不读流 → bool `this−0x2094`）→
//!   ⑪ LoadObject（ITemplate → `this−0x20ec`）→ ⑫ LoadObject
//!   （ILineDecoration → `this−0x20e8`）→ ⑬⑭ int32 ×2（`this−0x217c`/
//!   `−0x2174`）→ ⑮ byte（`this−0x20a4`）→ ⑯⑰ double ×2（`this−0x209c`/
//!   `−0x210c`）。
//!
//! ## 第二轮各类字段序（详见 docs/arcmap-10.1-display-symbols-round2.md）
//!
//! 下列「helper」指 `FUN_6eb406f0`（Simple 族共享）或 `FUN_6eb68fc0`
//! （MultiLayer/DotDensity 层数组重建辅助），反编译确认无 AfStreamRead、
//! 无流内字节。`±count` 循环长度为流内 int32，读取器以 `max_bytes` 设循环
//! 上限（读取器自身限额，非原生限制；原生按流失败自然终止）。
//!
//! - **MultiLayerLineSymbol**（虚表 `0x6ec0f410`，Load `0x6eb68280`，Save
//!   `0x6eb68610`，GetClassID `0x6eb67350`，Save 版本表 `DAT_6ec0b2c4`，
//!   10.1 写 2）：short 版本（>2 拒绝）→ int32（−0xc4）→ int32（−0xbc）→
//!   count → count × LoadObject（ILineSymbol `{E6BDAA7D}`，Save 槽 −0x74
//!   数组）→ count × int32（Save 槽 −0x60 数组）→ count × int32（Save 槽
//!   −0x4c 数组）→ 版本 <2 无串段（内存补空串）；版本 ≥2 count × String
//!   （Save 槽 −0x38 向量）。
//! - **MultiLayerMarkerSymbol**（虚表 `0x6ec0f754`，Load `0x6eb6b010`，Save
//!   `0x6eb6b3d0`，GetClassID `0x6eb691c0`，Save 版本表 `DAT_6ec0b2b8`，
//!   10.1 写 3）：short 版本（>3 拒绝）→ int32（−0x10c）→ int32（−0x104）→
//!   double ×4（−0x5c/−0x44/−0x54/−0x4c）→ LoadObject（IColor → Save 槽
//!   −0x3c）→ int32（−0x34）→ double（−0x24）→ LoadObject（ISymbol → Save
//!   槽 −0x30）→ count → count × LoadObject（IMarkerSymbol `{E6BDAA7C}`，
//!   Save 槽 −0xbc 数组）→ count × int32（−0xa8）→ count × int32（−0x94）→
//!   版本 ≥2 LoadObject（IMarkerBackground `{AE41E4F0}` → Save 槽 −0xc）→
//!   版本 <3 无串段；版本 ≥3 count × String（Save 槽 −0x80 向量）。
//! - **MultiLayerFillSymbol**（虚表 `0x6ec0f0fc`，Load `0x6eb65af0`，Save
//!   `0x6eb65e60`，GetClassID `0x6eb64d00`，Save 版本表 `DAT_6ec0b2d0`，
//!   10.1 写 2）：short 版本（>2 拒绝）→ int32（−0xb4）→ int32（−0xac）→
//!   LoadObject（IColor → Save 槽 −0xc）→ count → count × LoadObject
//!   （IFillSymbol `{E6BDAA7E}`，Save 槽 −0x64 数组）→ count × int32
//!   （−0x50）→ count × int32（−0x3c）→ 版本 <2 无串段；版本 ≥2 count ×
//!   String（−0x28 向量）。
//! - **CharacterMarkerSymbol**（虚表 `0x6ec11fa0`，Load `0x6eb9b150`，Save
//!   `0x6eb9b5b0`，GetClassID `0x6eb9b9b0`，Save 版本表 `DAT_6ec0b270`，
//!   10.1 写 4）：short 版本（>4 拒绝）→ LoadObject（IColor → 槽 −0xc4）→
//!   helper（不读流）→ int32（−0xc0）→ double ×5（−0xb4/−0xac/−0xa4/−0x9c/
//!   −0x94/−0x8c 计六，见记录）→ 版本 <3 LoadObject（IFontDisp
//!   `{BEF6E002}` → Save 槽 −0xbc）→ int32（−0x124）→ int32（−0x11c）→
//!   版本 >1 short（−0x10c；装载后内存槽被
//!   `Symbol::GetDefaultRotateWithTransformValue` 覆盖，流内值读取即弃——
//!   反汇编核实 `0x6eb3edde..0x6eb3edf0` 同型）→ 版本 ≥3 String + int32 ×4
//!   + short ×2 + int32 ×2（字体描述符，Save 由 −0xbc 字体对象取值回写）→
//!   版本 >3 再 LoadObject（IFontDisp → 同 Save 槽 −0xbc 重写）。
//! - **PictureMarkerSymbol**（虚表 `0x6ec10738`，Load `0x6eb7a650`，Save
//!   `0x6eb7ab00`，GetClassID `0x6eb77da0`，Save 版本表 `DAT_6ec0b29c`，
//!   10.1 写 9）：short 版本（>9 拒绝）。版本 ≤8 走历史路径
//!   `FUN_6eb7d750`：版本 ≤6 时（版本 <4 LoadObject（IPicture）→）LoadObject
//!   （IPicture `{7BF80980}`）；版本 ≥7 String +（串空 → LoadObject(IPicture)
//!   /串非空 → double）→ LoadObject（IColor）×3 → double ×6 → int32 ×2 →
//!   版本 >1 byte → 版本 >2 short → 版本 >4 short ×3 → 版本 ==6 String +
//!   byte + double → 版本 >7（v8）int32 N +（N≠0 时 N 字节位图数据）。
//!   版本 ==9：int32 图像类型（1/2/3，越界拒绝 `0x80004005`）→ int32 →
//!   类型 1：String + double；类型 2：int32 N +（N≠0 时 N 字节，反汇编核实
//!   `0x6eb7a7b5`/`0x6eb7a7de` 尺寸来源）→ 类型 3：LoadObject（IPicture）；
//!   之后 LoadObject（IColor）×3 → double ×6 → int32 ×2 → byte → short →
//!   short ×3。
//! - **PictureLineSymbol**（虚表 `0x6ec10344`，Load `0x6eb77250`，Save
//!   `0x6eb77300`，GetClassID `0x6eb76160`，版本常量 `DAT_6ec0b2a8`=1）：
//!   short 版本（>1 拒绝）→ byte（−0x30）→ double（−0x2c）→ double
//!   （−0x24）→ LoadObject（IPictureFillSymbol `{D842B081}` → Save 槽
//!   −0x14；实现类 PictureFillSymbol 未闭合 = 停止点）。
//! - **ArrowMarkerSymbol**（虚表 `0x6ec0c9dc`，Load `0x6eb3ec40`，Save
//!   `0x6eb3ee10`，GetClassID `0x6eb3ef60`，版本常量 `DAT_6ec0b314`=2）：
//!   short 版本（>2 拒绝）→ LoadObject（IColor → 槽 −0x40）→ helper（不读
//!   流）→ double ×3（−0x8c/−0x7c/−0x6c）→ int32 ×3（−0x64/−0xdc/−0xd4）→
//!   double ×2（−0x2c/−0x24）→ 版本 ≥2 short（−0xc4；读取即弃，同上 quirk，
//!   Save 恒写槽内默认值）。
//! - **LineFillSymbol**（虚表 `0x6ec0e72c`，Load `0x6eb5e390`，Save
//!   `0x6eb5e4e0`，GetClassID `0x6eb5caf0`，版本常量 `DAT_6ec0b2e8`=1）：
//!   short 版本（>1 拒绝）→ LoadObject（IColor → 槽 −0x6c）→ LoadObject
//!   （ISymbol → Save 槽 −0x74）→ LoadObject（ISymbol → Save 槽 −0x70）→
//!   double ×3（−0x64/−0x5c/−0x54）→ int32 ×2（−0xc4/−0xbc）。
//! - **MarkerFillSymbol**（虚表 `0x6ec0e9fc`，Load `0x6eb606c0`，Save
//!   `0x6eb60850`，GetClassID `0x6eb5f3b0`，版本常量 `DAT_6ec0b2e4`=1）：
//!   short 版本（>1 拒绝）→ int32（−0x7c）→ double ×4（−0x74/−0x6c/−0x64/
//!   −0x5c）→ LoadObject（IColor → Save 槽 −0x34）→ LoadObject（ISymbol →
//!   Save 槽 −0x80）→ LoadObject（ISymbol → Save 槽 −0x30）→ int32 ×2
//!   （−0xd4/−0xcc）→ double（−0xc）。
//! - **GradientFillSymbol**（虚表 `0x6ec0dd5c`，Load `0x6eb52710`，Save
//!   `0x6eb52870`，GetClassID `0x6eb50b90`，版本常量 `DAT_6ec0b2f8`=1）：
//!   short 版本（>1 拒绝）→ LoadObject（IColorRamp `{BEB87091}` → 槽
//!   −0x4c；实现类（RandomColorRamp 等）未闭合 = 停止点）→ LoadObject
//!   （IColor → Save 槽 −0x34）→ LoadObject（ISymbol → Save 槽 −0x30）→
//!   double（−0x44）→ int32（−0x2c）→ double（−0x24）→ int32 ×3（−0xc/
//!   −0x9c/−0x94）。
//! - **DotDensityFillSymbol**（虚表 `0x6ec0da18`，Load `0x6eb4ded0`，Save
//!   `0x6eb4e180`，GetClassID `0x6eb4de90`，版本常量 `DAT_6ec0b2fc`=1）：
//!   short 版本（>1 拒绝）→ byte（−0x34）→ double（−0x3c）→ byte（−0x28）→
//!   LoadObject（IColor → Save 槽 −0x54）→ LoadObject（ISymbol → Save 槽
//!   −0x58）→ int32 ×3（−0xd4/−0xcc/−0x1c）→ double（−0x44）→ byte
//!   （−0x27）→ count → count × LoadObject（IMarkerSymbol，Save 槽 −0x7c
//!   数组；Save 侧倒序写出，Load 顺序消费）→ count × int32（−0x68）→
//!   LoadObject（IColor → Save 槽 −0x50）。
//! - **Template**（虚表 `0x6ec1114c`，Load `0x6eb87ad0`，Save `0x6eb87bb0`，
//!   GetClassID `0x6eb87aa0`，版本常量 `DAT_6ec0b280`=1）：short 版本
//!   （>1 拒绝）→ double（−0x94，Interval）→ count → count × 16 字节
//!   （2 double/标记：实段长度、空段长度；Save 槽 −0x88 数组）。
//! - **LineDecoration**（虚表 `0x6ec0e4e4`，Load `0x6eb5c640`，Save
//!   `0x6eb5c740`，GetClassID `0x6eb5c0a0`，版本常量 `DAT_6ec0b2ec`=1）：
//!   short 版本（>1 拒绝）→ count → count × LoadObject
//!   （ILineDecorationElement `{533D88F2}` → 集合 −0x1c；实现类
//!   SimpleLineDecorationElement 闭合，其余 = 停止点）。
//! - **SimpleLineDecorationElement**（虚表 `0x6ec10d54`，Load `0x6eb84120`，
//!   Save `0x6eb84270`，GetClassID `0x6eb840e0`，版本常量 `DAT_6ec0b290`=1）：
//!   short 版本（>1 拒绝）→ byte ×3（−0x18/−0x17/−0x16）→ short（−0x14）→
//!   LoadObject（ISymbol → Save 槽 −0x24；标记符号候选）→ count → count ×
//!   double（−0x40 集合，位置候选）。
//!
//! LoadObject 槽的写入目标多数被 Ghidra 反编译隐藏（`FUN_6eb32c80` 辅助），
//! 槽归属以 Save 侧 `SaveObject(this+槽位)` 序对称印证；请求 IID 直接读自
//! 反编译引用的数据地址并经类型库 dump 定名。
//!
//! ## 槽位分派（对象层共享入口）
//!
//! 本模块有两个按类分派的符号槽入口：
//!
//! - [`read_display_symbol_slot_10_1`]（第一轮闭合集：SimpleMarkerSymbol 与
//!   SimpleLine/SimpleFill/MarkerLine/CartographicLine/HashLine 五类）——
//!   既有宿主（SimpleRenderer/LegendClass 的 ISymbol 槽、TextSymbol 的
//!   ISymbol #2 槽等）的现有入口，接口保持稳定（其返回枚举与停止归约
//!   [`slot_stop_10_1`] 不变）。
//! - [`read_symbol_slot_full_10_1`]（全部闭合类）——第二轮起的对象层入口：
//!   第二轮闭合体的全部嵌套符号槽（MultiLayer 层槽、点密度标记数组、
//!   LineDecoration 元素符号槽、LineFill/MarkerFill 双符号槽等）经它分派；
//!   停止归约 [`slot_full_stop_10_1`]。既有宿主在整合轮切换到本入口后即得
//!   第二轮符号的递归解析（未知类仍以 `Unsupported` 停止，语义不变）。
//!
//! 第一轮闭合体的内部嵌套槽仍经第一轮入口解析（其闭合集以第一轮为界——
//! 例如经本入口嵌套的 SimpleFillSymbol，其轮廓槽只识别第一轮符号类）；
//! Template/LineDecoration 槽经本模块装饰槽分派 [`read_decoration_slot_10_1`]
//! 按类闭合。IFontDisp/IPicture/IColorRamp/IMarkerBackground/IPictureFillSymbol
//! 槽的实现类未闭合，仅分类空标记与回引，新对象 = 停止点。
//!
//! 符号可嵌套符号（如 SimpleFillSymbol 轮廓槽内常见线符号、MultiLayer 层
//! 槽），分派带深度参数：超过 [`MAX_SYMBOL_NESTING_DEPTH`] 报
//! `ErrorKind::InvalidInput`（防栈溢出的读取器自身限额，非原生限制；原生对
//! 嵌套深度无静态检查）。颜色槽经 [`crate::objects::display_color::
//! read_color_slot_10_1`] 分派（五类颜色全部闭合）。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头并比对类常量，从对象头之后的位置调用各 `read_*_body_10_1`；
//! 本模块不重复读取对象头。全部读取在 trial 游标与 trial 登记表副本上进行，
//! 完整成功（含停止点返回）才一并提交；任何错误既不推进游标、也不修改登记
//! 表。槽位新对象类未知时该槽位读取器内部回退到槽位起点（不保留该键），
//! 调用者以槽位 `Unsupported` 变体 / 本体 `stop` 字段识别并保留剩余字节。
//! 体内已闭合前缀的嵌套停止传播为本体停止原因（游标停在内层停止点，已
//! 闭合字段保留）。
//!
//! 未解决问题（详见两份研究记录开放项）：各标量字段的业务语义全部未定名；
//! `FUN_6eb406f0`/`FUN_6eb68fc0` 的副作用仅确认不读流；三处
//! 「装载后槽被 GetDefaultRotateWithTransformValue 覆盖」的 short（Arrow/
//! Character/PictureMarker）读侧丢弃语义为反汇编级观察（不影响流内字节）；
//! MultiLayer 各 int 数组与字符串的业务语义、字体描述符八字段与 IFontDisp
//! 接口属性的对应、Template 标记 16 字节的两 double 归属，均为候选；全部
//! 结论为静态观察，未经目标软件运行验证，样本无字节佐证。

mod chart;
mod raster;
mod url_picture;

use chart::read_chart_symbol_body_10_1;
pub use chart::{
    BAR_CHART_SYMBOL_CLASS_ID, ChartSymbolBody10_1, ChartSymbolFields10_1,
    PIE_CHART_SYMBOL_CLASS_ID, STACKED_CHART_SYMBOL_CLASS_ID,
};

pub use raster::{
    COLOR_RAMP_SYMBOL_CLASS_ID, COLOR_SYMBOL_CLASS_ID, ColorRampSymbolBody10_1,
    ColorRampSymbolTail10_1, ColorSymbolBody10_1, RASTER_RGB_SYMBOL_CLASS_ID,
};
use raster::{read_color_ramp_symbol_body_10_1, read_color_symbol_body_10_1};

use url_picture::read_url_picture_marker_symbol_body_10_1;
pub use url_picture::{URL_PICTURE_MARKER_SYMBOL_CLASS_ID, UrlPictureMarkerSymbolBody10_1};

use crate::objects::callout_background::{CalloutSlot10_1, read_marker_background_slot_10_1};
use crate::objects::color_ramp::{ColorRampSlot10_1, read_color_ramp_slot_10_1};
use crate::objects::display_color::{ColorSlot10_1, read_color_slot_10_1};
use crate::objects::element_body::{
    TEXT_SYMBOL_CLASS_ID, TextSymbolBody10_1, read_text_symbol_body_10_1,
};
use crate::objects::ole_font::{
    STDFONT_CLASS_ID, STDPICTURE_CLASS_ID, StdFontBody10_1, StdPictureBody10_1,
    read_std_font_body_10_1, read_std_picture_body_10_1,
};
use crate::objects::picture_fill_symbol::{
    PICTURE_FILL_SYMBOL_CLASS_ID, PictureFillSymbolBody10_1, PictureFillSymbolSlot10_1,
    read_picture_fill_symbol_body_10_1, read_picture_fill_symbol_slot_10_1,
};
use crate::objects::property_set::{
    PropertyValueObjectStream10_1, read_padded_variant_object_stream_nested_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 线条/填充符号实体版本上限：`1 < ver` 有符号拒绝（四类共用；样本无实例）。
const MAX_SYMBOL_VERSION: i16 = 1;

/// SimpleMarkerSymbol::Load 接受的最大版本：`2 < (short)ver` 有符号拒绝；
/// 样本 2。证据：DisplayCore.dll 实体 Load `0x6ebbefa0`（第七轮记录）。
const MAX_SIMPLE_MARKER_VERSION: i16 = 2;

/// 符号槽位分派的最大嵌套深度（防栈溢出的读取器自身限额，非原生限制）。
pub const MAX_SYMBOL_NESTING_DEPTH: usize = 16;

/// SimpleMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FE-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid.ecfg 声明（模块 DisplayCore.dll）；DisplayCore.dll GetClassID
/// `0x6ebbf310` 写出；样本 `/Maps:0x403`（SimpleRenderer 体内 ISymbol 槽，
/// 键 36）与 LegendClass 体内的回引头逐字节相同（辅助观察）。
pub const SIMPLE_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfe, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MarkerLineSymbol 实体版本上限：`2 < ver` 有符号拒绝；Save 恒写 2。
const MAX_MARKER_LINE_VERSION: i16 = 2;

/// MultiLayerLineSymbol 实体版本上限：`2 < ver` 有符号拒绝；Save 版本表
/// `DAT_6ec0b2c4`（docVer 0..4 → 1,1,1,1,2），10.1 写 2。
const MAX_MULTI_LAYER_LINE_VERSION: i16 = 2;

/// MultiLayerMarkerSymbol 实体版本上限：`3 < ver` 有符号拒绝；Save 版本表
/// `DAT_6ec0b2b8`（docVer 0..4 → 2,2,2,2,3），10.1 写 3。
const MAX_MULTI_LAYER_MARKER_VERSION: i16 = 3;

/// MultiLayerFillSymbol 实体版本上限：`2 < ver` 有符号拒绝；Save 版本表
/// `DAT_6ec0b2d0`（docVer 0..4 → 1,1,1,1,2），10.1 写 2。
const MAX_MULTI_LAYER_FILL_VERSION: i16 = 2;

/// CharacterMarkerSymbol 实体版本上限：`4 < ver` 有符号拒绝；Save 版本表
/// `DAT_6ec0b270`（docVer 0..5 → …,4），10.1 写 4。
const MAX_CHARACTER_MARKER_VERSION: i16 = 4;

/// PictureMarkerSymbol 实体版本上限：`9 < ver` 有符号拒绝；Save 版本表
/// `DAT_6ec0b29c`（docVer 0..5 → 4,4,4,5,8,9），10.1 写 9。
const MAX_PICTURE_MARKER_VERSION: i16 = 9;

/// ArrowMarkerSymbol 实体版本上限：`2 < ver` 有符号拒绝；版本常量
/// `DAT_6ec0b314`=2（Save 恒写 2，含尾部 short）。
const MAX_ARROW_MARKER_VERSION: i16 = 2;

/// MultiLayerLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FA-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb67350` 写出（IPersistStream
/// 虚表 `0x6ec0f410` 槽 +0x0c）。样本中无实例，无字节佐证。
pub const MULTI_LAYER_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfa, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MultiLayerMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FF-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb691c0` 写出（虚表 `0x6ec0f754`）。样本中
/// 无实例，无字节佐证。
pub const MULTI_LAYER_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0xff, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MultiLayerFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E604-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb64d00` 写出（虚表 `0x6ec0f0fc`）。样本中
/// 无实例，无字节佐证。
pub const MULTI_LAYER_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x04, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// CharacterMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E600-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb9b9b0` 写出（虚表 `0x6ec11fa0`）。样本中
/// 无实例，无字节佐证。
pub const CHARACTER_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0x00, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// PictureMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E602-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb77da0` 写出（虚表 `0x6ec10738`）。样本中
/// 无实例，无字节佐证。
pub const PICTURE_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0x02, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// PictureLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{22C8C5A1-84FC-11D4-834D-0080C79F0371}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb76160` 写出（虚表 `0x6ec10344`）。样本中
/// 无实例，无字节佐证。
pub const PICTURE_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xa1, 0xc5, 0xc8, 0x22, 0xfc, 0x84, 0xd4, 0x11, 0x83, 0x4d, 0x00, 0x80, 0xc7, 0x9f, 0x03, 0x71,
];

/// ArrowMarkerSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{88539431-E06E-11D1-B277-0000F878229E}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb3ef60` 写出（虚表 `0x6ec0c9dc`）。样本中
/// 无实例，无字节佐证。
pub const ARROW_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0x31, 0x94, 0x53, 0x88, 0x6e, 0xe0, 0xd1, 0x11, 0xb2, 0x77, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];

/// LineFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E606-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb5caf0` 写出（虚表 `0x6ec0e72c`）。样本中
/// 无实例，无字节佐证。
pub const LINE_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x06, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MarkerFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E608-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb5f3b0` 写出（虚表 `0x6ec0e9fc`）。样本中
/// 无实例，无字节佐证。
pub const MARKER_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x08, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// GradientFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E609-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb50b90` 写出（虚表 `0x6ec0dd5c`）。样本中
/// 无实例，无字节佐证。
pub const GRADIENT_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x09, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// DotDensityFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{9A1EBA10-CDF9-11D3-81EB-0080C79F0371}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb4de90` 写出（虚表 `0x6ec0da18`）。样本中
/// 无实例，无字节佐证。
pub const DOT_DENSITY_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x10, 0xba, 0x1e, 0x9a, 0xf9, 0xcd, 0xd3, 0x11, 0x81, 0xeb, 0x00, 0x80, 0xc7, 0x9f, 0x03, 0x71,
];

/// Template 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{41093A71-CCE1-11D0-BFAA-0080C7E24280}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb87aa0` 写出（虚表
/// `0x6ec1114c`）；反编译立即数互证。请求接口 ITemplate `{7914E5F4}` 经
/// esriDisplay 类型库 dump 定名。样本中无实例，无字节佐证。
pub const TEMPLATE_CLASS_ID: [u8; 16] = [
    0x71, 0x3a, 0x09, 0x41, 0xe1, 0xcc, 0xd0, 0x11, 0xbf, 0xaa, 0x00, 0x80, 0xc7, 0xe2, 0x42, 0x80,
];

/// LineDecoration 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{533D88F5-0A1A-11D2-B27F-0000F878229E}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb5c0a0` 写出（虚表 `0x6ec0e4e4`）；反编译
/// 立即数互证。请求接口 ILineDecoration `{533D88F0}` 经类型库 dump 定名。
/// 样本中无实例，无字节佐证。
pub const LINE_DECORATION_CLASS_ID: [u8; 16] = [
    0xf5, 0x88, 0x3d, 0x53, 0x1a, 0x0a, 0xd2, 0x11, 0xb2, 0x7f, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];

/// SimpleLineDecorationElement 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{533D88F3-0A1A-11D2-B27F-0000F878229E}`。证据：
/// esri.clsid 声明；GetClassID `0x6eb840e0` 写出（虚表 `0x6ec10d54`）；反编译
/// 立即数互证。默认元素接口 ILineDecorationElement `{533D88F2}` 经类型库
/// dump 定名。样本中无实例，无字节佐证。
pub const SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID: [u8; 16] = [
    0xf3, 0x88, 0x3d, 0x53, 0x1a, 0x0a, 0xd2, 0x11, 0xb2, 0x7f, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];

/// SimpleLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5F9-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll，progID esriDisplay.SimpleLineSymbol）；
/// GetClassID `0x6ebbd090` 写出（IPersistStream 虚表 `0x6ec15498` 槽 +0x0c
/// thunk `0x6ebbd9a0`）。样本中无实例，无字节佐证。
pub const SIMPLE_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xf9, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// SimpleFillSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E603-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6ebbb6a0` 写出（虚表
/// `0x6ec152ac` 槽 +0x0c thunk `0x6ebbc660`）。样本中无实例，无字节佐证。
pub const SIMPLE_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x03, 0xe6, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// MarkerLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FD-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb61410` 写出（虚表
/// `0x6ec0eca0` 槽 +0x0c thunk `0x6eb636e0`）。样本中无实例，无字节佐证。
pub const MARKER_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfd, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// CartographicLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FB-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb99100` 写出（虚表
/// `0x6ec11ca8` 槽 +0x0c thunk `0x6eb99e90`）。样本中无实例，无字节佐证。
pub const CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfb, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// HashLineSymbol 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{7914E5FC-C892-11D0-8BB6-080009EE4E41}`。证据：
/// esri.clsid 声明（DisplayCore.dll）；GetClassID `0x6eb53c30` 写出（虚表
/// `0x6ec0e0b8` 槽 +0x0c thunk `0x6eb56f50`）。样本中无实例，无字节佐证。
pub const HASH_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0xfc, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// 请求 IID `{7914E5F4-C892-11D0-8BB6-080009EE4E41}` 的 16 字节内存序
/// （诊断对照用，不校验流内字节）。
///
/// MarkerLine/CartographicLine/HashLine 三类 Load 的中段 LoadObject 请求该
/// IID（反编译 `DAT_6ec0e0e8`）。接口名未定名；按符号属性面推断为 ITemplate
/// 候选（三类均有模板属性）。未经类型库定名，仅作对照。
pub const TEMPLATE_IID_CANDIDATE: [u8; 16] = [
    0xf4, 0xe5, 0x14, 0x79, 0x92, 0xc8, 0xd0, 0x11, 0x8b, 0xb6, 0x08, 0x00, 0x09, 0xee, 0x4e, 0x41,
];

/// 请求 IID `{533D88F0-0A1A-11D2-B27F-0000F878229E}` 的 16 字节内存序
/// （诊断对照用，不校验流内字节）。
///
/// 同三类 Load 的尾段 LoadObject 请求该 IID（反编译 `DAT_6ec0e0f8`）；esri
/// .clsid 中 LineDecoration coclass 为 `{533D88F5-0A1A-11D2-B27F-0000F878229E}`
/// （同族 Data1 相邻），接口名候选 ILineDecoration，未经类型库定名。
pub const LINE_DECORATION_IID_CANDIDATE: [u8; 16] = [
    0xf0, 0x88, 0x3d, 0x53, 0x1a, 0x0a, 0xd2, 0x11, 0xb2, 0x7f, 0x00, 0x00, 0xf8, 0x78, 0x22, 0x9e,
];

/// 读取 int16 版本并按原生有符号比较校验上限；负值位模式一并拒绝。
fn read_version_short(
    trial: &mut Reader<'_>,
    max: i16,
    what: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let version = read_signed_symbol_version(trial, max, what, entry_offset)?;
    if (version as i16) < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{what} 对象体 offset 0x{entry_offset:x}: 未确认的负版本位模式 {version}"),
        ));
    }
    Ok(version)
}

fn read_signed_symbol_version(
    trial: &mut Reader<'_>,
    max: i16,
    what: &str,
    entry_offset: usize,
) -> io::Result<u16> {
    let version = trial.read_u16_le()?;
    if (version as i16) > max {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{what} 对象体 offset 0x{entry_offset:x}: 版本 {version} 超过有符号上限 {max}"),
        ));
    }
    Ok(version)
}

/// 读取 4 字节小端 int32。
fn read_i32_le(trial: &mut Reader<'_>) -> io::Result<i32> {
    Ok(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()))
}

/// DisplayRepresentation.dll 的 SymbolMimic。
pub const SYMBOL_MIMIC_CLASS_ID: [u8; 16] = [
    0x86, 0x47, 0x28, 0xec, 0x94, 0xa3, 0xe9, 0x44, 0x94, 0xaa, 0xaa, 0x05, 0x83, 0x07, 0x47, 0xe2,
];

/// DisplayRepresentation.dll 的 RepresentationRule。
pub const REPRESENTATION_RULE_CLASS_ID: [u8; 16] = [
    0x0a, 0xe4, 0x79, 0x10, 0xc1, 0x83, 0x7b, 0x46, 0xb8, 0x28, 0xe5, 0xc0, 0x6e, 0xf5, 0xf6, 0xb7,
];

pub const BASIC_FILL_SYMBOL_CLASS_ID: [u8; 16] = [
    0x64, 0xc4, 0x38, 0xbc, 0x02, 0x8b, 0x0c, 0x4b, 0x81, 0xd5, 0x82, 0x37, 0x84, 0x7a, 0x46, 0xce,
];
pub const SOLID_COLOR_PATTERN_CLASS_ID: [u8; 16] = [
    0x05, 0x2a, 0x74, 0x29, 0x99, 0x59, 0xe7, 0x43, 0x9f, 0xa1, 0x53, 0x70, 0xc6, 0x84, 0xf4, 0x9e,
];

pub const LINE_PATTERN_CLASS_ID: [u8; 16] = [
    0x52, 0x0f, 0x4f, 0x44, 0xbf, 0xda, 0x39, 0x46, 0xb2, 0x39, 0x9a, 0x88, 0xb3, 0xc3, 0xf8, 0x30,
];
pub const GRADIENT_PATTERN_CLASS_ID: [u8; 16] = [
    0x2d, 0x1c, 0xf1, 0x00, 0xbb, 0xfc, 0x7f, 0x4b, 0x81, 0x35, 0xe7, 0xaf, 0x48, 0x1c, 0xe5, 0x92,
];

pub const BASIC_LINE_SYMBOL_CLASS_ID: [u8; 16] = [
    0x23, 0xb8, 0x8f, 0xed, 0xfa, 0x10, 0x8b, 0x46, 0x8d, 0xcd, 0x43, 0x27, 0xc0, 0xdf, 0x0b, 0x22,
];
pub const LINE_STROKE_CLASS_ID: [u8; 16] = [
    0x5c, 0x7f, 0x2a, 0xbd, 0x8b, 0x1d, 0x21, 0x42, 0xb3, 0xb0, 0x7f, 0x40, 0xb0, 0x30, 0x55, 0x03,
];

pub const BASIC_MARKER_SYMBOL_CLASS_ID: [u8; 16] = [
    0xa0, 0x73, 0xff, 0x1b, 0x34, 0xd1, 0x88, 0x4e, 0x8b, 0xc5, 0x78, 0x8b, 0xd3, 0x51, 0xb1, 0x2d,
];
pub const MARKER_PLACEMENT_ON_POINT_CLASS_ID: [u8; 16] = [
    0xff, 0x7b, 0x0c, 0x16, 0x37, 0x22, 0x67, 0x4a, 0xb1, 0x63, 0x89, 0xc2, 0x23, 0xdb, 0x0a, 0x1d,
];

/// 10.1 中使用 GraphicAttributeList 持久化的标记布置类。
pub const MARKER_PLACEMENT_CLASSES_10_1: [[u8; 16]; 11] = [
    MARKER_PLACEMENT_ON_POINT_CLASS_ID,
    // MarkerPlacementAlongLine
    [
        0x42, 0xe6, 0x55, 0x59, 0x3b, 0x62, 0x55, 0x4d, 0x86, 0xe9, 0x04, 0x42, 0xc6, 0x27, 0x29,
        0x42,
    ],
    // MarkerPlacementInsidePolygon
    [
        0x64, 0x67, 0x4f, 0xd9, 0x28, 0x5d, 0x6c, 0x4a, 0x90, 0xb6, 0x3f, 0x36, 0x5b, 0x6b, 0x05,
        0x39,
    ],
    // MarkerPlacementAtExtremities
    [
        0x56, 0x1d, 0xb8, 0x34, 0x7d, 0x88, 0x8f, 0x44, 0x9e, 0xe8, 0x4c, 0x3f, 0x52, 0x65, 0x6a,
        0x14,
    ],
    // MarkerPlacementPolygonCenter
    [
        0xf9, 0x7e, 0xda, 0xc2, 0xe1, 0x5a, 0x6b, 0x4d, 0x95, 0xbc, 0xad, 0x78, 0x77, 0x19, 0x62,
        0xe4,
    ],
    // MarkerPlacementDecoration
    [
        0xfe, 0x0f, 0xc2, 0xb6, 0x22, 0x88, 0xf7, 0x4c, 0xbc, 0x0d, 0x2c, 0xe7, 0xb2, 0x35, 0x3f,
        0x7b,
    ],
    // MarkerPlacementOnLine
    [
        0x1d, 0x5f, 0x3f, 0x7c, 0x9a, 0xcc, 0xb5, 0x45, 0x88, 0x84, 0xee, 0x3a, 0xea, 0xd9, 0xe7,
        0x4e,
    ],
    // MarkerPlacementVariableAlongLine
    [
        0x5c, 0x59, 0x83, 0xa0, 0xdd, 0x15, 0xd2, 0x4a, 0xa3, 0x0c, 0x90, 0xb7, 0x08, 0x7e, 0x5d,
        0xb6,
    ],
    // MarkerPlacementRandomAlongLine
    [
        0x96, 0x44, 0xdc, 0x2f, 0x2b, 0xed, 0xcc, 0x4e, 0xb5, 0x57, 0x05, 0x38, 0x73, 0xfd, 0x0c,
        0x03,
    ],
    // MarkerPlacementRandomInPolygon
    [
        0x2f, 0x68, 0x20, 0x6d, 0x9b, 0x1a, 0x87, 0x40, 0xbc, 0xc3, 0x0c, 0x9a, 0xa6, 0x62, 0xd5,
        0x2d,
    ],
    // MarkerPlacementOnVertices
    [
        0xce, 0xf9, 0x40, 0x09, 0x95, 0x4a, 0xd1, 0x46, 0xb3, 0xfa, 0xd3, 0xa1, 0x97, 0xee, 0xb1,
        0x9f,
    ],
];

/// 10.1 中使用 GraphicAttributeList 持久化的几何效果类。
pub const GEOMETRIC_EFFECT_CLASSES_10_1: [[u8; 16]; 22] = [
    // GeometricEffectRadial
    [
        0x0a, 0x4c, 0xd0, 0x18, 0xbf, 0x69, 0x0a, 0x49, 0x8d, 0x47, 0x37, 0xa7, 0x85, 0x48, 0x50,
        0x7a,
    ],
    // GeometricEffectDash
    [
        0x33, 0x52, 0x0e, 0x76, 0xe3, 0x1b, 0x73, 0x4c, 0x8c, 0x2d, 0x7c, 0x2d, 0xbf, 0xc0, 0x48,
        0x8a,
    ],
    // GeometricEffectAddControlPoints
    [
        0xb6, 0x6f, 0xf8, 0xf3, 0x32, 0xf7, 0x80, 0x45, 0x8e, 0xd9, 0xe9, 0x91, 0x12, 0x80, 0x7b,
        0x1b,
    ],
    // GeometricEffectOffset
    [
        0x99, 0x6d, 0x5e, 0x8d, 0x5f, 0x74, 0x4e, 0x42, 0x81, 0xab, 0xa2, 0xc3, 0xbf, 0xef, 0xbe,
        0xb7,
    ],
    // GeometricEffectBuffer
    [
        0xb7, 0x7a, 0x46, 0x8f, 0x02, 0x8c, 0xed, 0x42, 0x9e, 0x3b, 0x15, 0x21, 0x77, 0x30, 0x40,
        0x5a,
    ],
    // GeometricEffectReverse
    [
        0x7b, 0x91, 0xbc, 0x10, 0xa5, 0x1f, 0x20, 0x45, 0xa4, 0xfa, 0xd9, 0x79, 0xae, 0xd1, 0x53,
        0x06,
    ],
    // GeometricEffectCut
    [
        0xb9, 0x49, 0x18, 0xba, 0x40, 0x46, 0xb2, 0x42, 0xab, 0xcf, 0xd7, 0xf4, 0xcb, 0xf2, 0xf3,
        0xd1,
    ],
    // GeometricEffectArrow
    [
        0x47, 0x51, 0x19, 0x2b, 0xed, 0x95, 0x63, 0x48, 0xae, 0xbc, 0xb8, 0x76, 0xe1, 0xa4, 0x76,
        0x6c,
    ],
    // GeometricEffectJog
    [
        0xe5, 0xbc, 0x21, 0x95, 0xf1, 0xb2, 0x3a, 0x47, 0xa5, 0x33, 0xd8, 0xe9, 0x4f, 0x0b, 0x8b,
        0xed,
    ],
    // GeometricEffectDonut
    [
        0x37, 0x6e, 0x42, 0xa2, 0x61, 0xfb, 0x39, 0x43, 0x83, 0xf5, 0x70, 0xd3, 0xf8, 0x86, 0xec,
        0x3a,
    ],
    // GeometricEffectEnclosingPolygon
    [
        0xb3, 0x9f, 0x3b, 0xde, 0x10, 0xc7, 0x43, 0x4f, 0xbc, 0x7e, 0x35, 0xb5, 0xb3, 0xad, 0xac,
        0xb7,
    ],
    // GeometricEffectSimplify
    [
        0x84, 0xd1, 0x19, 0x3d, 0x20, 0xb8, 0xfb, 0x49, 0x95, 0x3e, 0x8f, 0xf9, 0x6c, 0xad, 0x5d,
        0x11,
    ],
    // GeometricEffectSmooth
    [
        0x3d, 0x65, 0x7d, 0xca, 0x35, 0xf0, 0x1d, 0x46, 0x93, 0xaa, 0xfb, 0xb2, 0xec, 0xa5, 0x65,
        0xef,
    ],
    // GeometricEffectRegularPolygon
    [
        0xee, 0x3a, 0x6a, 0xe1, 0x5c, 0xa3, 0x1f, 0x46, 0x9a, 0x2a, 0xbb, 0xb4, 0x1c, 0x0b, 0x77,
        0x7f,
    ],
    // GeometricEffectWave
    [
        0x97, 0x2a, 0x35, 0x74, 0xc6, 0x64, 0x5d, 0x47, 0x88, 0xa2, 0xa8, 0xe0, 0x52, 0x36, 0xba,
        0x07,
    ],
    // GeometricEffectTaperedPolygon
    [
        0x56, 0xbd, 0xa7, 0xb8, 0x56, 0xbb, 0xb7, 0x4b, 0x81, 0x55, 0x56, 0x74, 0x1d, 0x01, 0xd8,
        0xb5,
    ],
    // GeometricEffectMove
    [
        0x6e, 0xd7, 0x00, 0x8e, 0x54, 0x5a, 0x2d, 0x4c, 0x92, 0x18, 0xf7, 0xf5, 0xfc, 0x06, 0x9e,
        0xce,
    ],
    // GeometricEffectRotate
    [
        0xcf, 0xf3, 0xa1, 0xab, 0x07, 0x32, 0x89, 0x4c, 0x8c, 0xe3, 0x7a, 0x2a, 0x8f, 0xd8, 0xd2,
        0x21,
    ],
    // GeometricEffectScale
    [
        0x91, 0x3e, 0x79, 0xe9, 0x8a, 0x69, 0x45, 0x4c, 0x8f, 0x09, 0xb4, 0x36, 0x83, 0xe4, 0x9b,
        0x93,
    ],
    // GeometricEffectExtension
    [
        0x78, 0x74, 0xac, 0xf1, 0xeb, 0x67, 0x77, 0x4c, 0xad, 0xc4, 0x54, 0x91, 0xbb, 0x44, 0xf1,
        0x40,
    ],
    // GeometricEffectOffsetTangent
    [
        0xa2, 0x2e, 0x2e, 0xdc, 0xca, 0x2b, 0x4b, 0x4d, 0x95, 0xee, 0x51, 0xcf, 0x72, 0x43, 0x80,
        0xe8,
    ],
    // GeometricEffectSuppress
    [
        0xd7, 0x52, 0xa1, 0xb6, 0x82, 0xa8, 0x14, 0x40, 0x93, 0x80, 0xc9, 0x60, 0xe5, 0x24, 0x42,
        0x7e,
    ],
];

#[derive(Debug, Clone, PartialEq)]
pub struct GeometricEffectEntry10_1 {
    pub name: Option<String>,
    pub entry: ObjectEntry10_1,
    pub body: Option<GraphicAttributeList10_1>,
    pub rule: Option<Box<RepresentationRuleBody10_1>>,
    pub layer: Option<Box<RepresentationRuleLayer10_1>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicMarkerSymbol10_1 {
    pub version: u16,
    pub effect_list_version: u16,
    pub effect_count: i32,
    pub effects: Vec<GeometricEffectEntry10_1>,
    pub attributes: GraphicAttributeList10_1,
    pub placement_name: Option<String>,
    pub placement: ObjectEntry10_1,
    pub placement_body: Option<GraphicAttributeList10_1>,
    pub trailing_value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicLineSymbol10_1 {
    pub version: u16,
    pub effect_list_version: u16,
    pub effect_count: i32,
    pub effects: Vec<GeometricEffectEntry10_1>,
    pub stroke: ObjectEntry10_1,
    pub stroke_body: Option<GraphicAttributeList10_1>,
    pub trailing_value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicFillSymbol10_1 {
    pub version: u16,
    pub effect_list_version: u16,
    pub effect_count: i32,
    pub effects: Vec<GeometricEffectEntry10_1>,
    pub pattern: ObjectEntry10_1,
    pub pattern_body: Option<GraphicAttributeList10_1>,
    pub trailing_value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicAttributeList10_1 {
    pub version: u16,
    pub declared_count: i32,
    pub attributes: Vec<(i32, PropertyValueObjectStream10_1)>,
}

pub type SolidColorPattern10_1 = GraphicAttributeList10_1;

pub(crate) fn is_graphic_attribute_object(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        SOLID_COLOR_PATTERN_CLASS_ID
            | LINE_PATTERN_CLASS_ID
            | GRADIENT_PATTERN_CLASS_ID
            | LINE_STROKE_CLASS_ID
    ) || MARKER_PLACEMENT_CLASSES_10_1.contains(&class_id)
        || GEOMETRIC_EFFECT_CLASSES_10_1.contains(&class_id)
}

pub(crate) fn read_graphic_attribute_list_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    variant_depth: usize,
) -> io::Result<Option<GraphicAttributeList10_1>> {
    let version = read_signed_symbol_version(reader, 1, "GraphicAttributeList", reader.position())?;
    let declared_count = read_i32_le(reader)?;
    let count = declared_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / 20 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "GraphicAttributeList 属性计数超出剩余字节或限额",
        ));
    }
    let mut attributes = Vec::new();
    for _ in 0..count {
        let id = read_i32_le(reader)?;
        let value = match read_padded_variant_object_stream_nested_10_1(
            reader,
            max_bytes,
            refs,
            variant_depth,
        ) {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::Unsupported => return Ok(None),
            Err(error) => return Err(error),
        };
        attributes.push((id, value));
    }
    Ok(Some(GraphicAttributeList10_1 {
        version,
        declared_count,
        attributes,
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepresentationRuleLayer10_1 {
    pub entry: ObjectEntry10_1,
    pub fill: Option<BasicFillSymbol10_1>,
    pub line: Option<BasicLineSymbol10_1>,
    pub marker: Option<BasicMarkerSymbol10_1>,
}

fn read_extra_safe_symbol_entry_10_1<'a>(
    reader: &mut Reader<'a>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
) -> io::Result<(Reader<'a>, ObjectEntry10_1)> {
    let (a, b) = (reader.read_u32_le()?, reader.read_u32_le()?);
    let size = if b == 0 { a } else { b } as usize;
    if size > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ExtraSafe 属性对象长度超过限额",
        ));
    }
    let mut region = reader.read_region(size)?;
    let entry = if size == 0 {
        ObjectEntry10_1::Null
    } else {
        read_object_entry_10_1(&mut region, refs)?
    };
    Ok((region, entry))
}

fn finish_extra_safe_symbol_entry_10_1(region: &Reader<'_>) -> io::Result<()> {
    if region.remaining() != 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "对象体与 ExtraSafe 边界不符",
        ));
    }
    Ok(())
}

fn read_extra_safe_attributes_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    classes: &[[u8; 16]],
) -> io::Result<Option<(ObjectEntry10_1, Option<GraphicAttributeList10_1>)>> {
    let (mut region, entry) = read_extra_safe_symbol_entry_10_1(reader, max_bytes, refs)?;
    let body = match entry {
        ObjectEntry10_1::Null => None,
        ObjectEntry10_1::BackReference { class_id, .. } if classes.contains(&class_id) => None,
        ObjectEntry10_1::NewObject { class_id, .. } if depth > 1 && classes.contains(&class_id) => {
            let Some(body) = read_graphic_attribute_list_10_1(&mut region, max_bytes, refs, 0)?
            else {
                return Ok(None);
            };
            Some(body)
        }
        _ => return Ok(None),
    };
    finish_extra_safe_symbol_entry_10_1(&region)?;
    Ok(Some((entry, body)))
}

fn read_geometric_effects_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    version: u16,
    declared_count: i32,
) -> io::Result<Option<Vec<GeometricEffectEntry10_1>>> {
    let count = declared_count.max(0) as usize;
    let has_name = (version as i16) >= 2;
    let minimum_size = if has_name { 14 } else { 8 };
    if count > reader.remaining().min(max_bytes) / minimum_size {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "几何效果计数超出剩余字节或限额",
        ));
    }
    let mut effects = Vec::new();
    for _ in 0..count {
        let name = has_name
            .then(|| read_string_10_1(reader, max_bytes))
            .transpose()?;
        let (mut region, entry) = read_extra_safe_symbol_entry_10_1(reader, max_bytes, refs)?;
        let mut effect = GeometricEffectEntry10_1 {
            name,
            entry,
            body: None,
            rule: None,
            layer: None,
        };
        match entry {
            ObjectEntry10_1::Null => {}
            ObjectEntry10_1::BackReference { class_id, .. }
                if GEOMETRIC_EFFECT_CLASSES_10_1.contains(&class_id)
                    || class_id == REPRESENTATION_RULE_CLASS_ID
                    || is_representation_layer(class_id) => {}
            ObjectEntry10_1::NewObject { class_id, .. }
                if depth > 1 && GEOMETRIC_EFFECT_CLASSES_10_1.contains(&class_id) =>
            {
                let Some(body) = read_graphic_attribute_list_10_1(&mut region, max_bytes, refs, 0)?
                else {
                    return Ok(None);
                };
                effect.body = Some(body);
            }
            ObjectEntry10_1::NewObject {
                class_id: REPRESENTATION_RULE_CLASS_ID,
                ..
            } if depth > 1 => {
                let Some(rule) =
                    read_representation_rule_body_10_1(&mut region, max_bytes, refs, depth - 1)?
                else {
                    return Ok(None);
                };
                effect.rule = Some(Box::new(rule));
            }
            ObjectEntry10_1::NewObject { class_id, .. }
                if depth > 1 && is_representation_layer(class_id) =>
            {
                let Some(layer) = read_representation_layer_body_10_1(
                    &mut region,
                    max_bytes,
                    refs,
                    depth - 1,
                    entry,
                )?
                else {
                    return Ok(None);
                };
                effect.layer = Some(Box::new(layer));
            }
            _ => return Ok(None),
        }
        finish_extra_safe_symbol_entry_10_1(&region)?;
        effects.push(effect);
    }
    Ok(Some(effects))
}

fn read_basic_fill_symbol_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<Option<BasicFillSymbol10_1>> {
    let version = read_signed_symbol_version(reader, 1, "BasicFillSymbol", reader.position())?;
    let effect_list_version =
        read_signed_symbol_version(reader, 2, "BasicFillSymbol 效果列表", reader.position())?;
    let effect_count = read_i32_le(reader)?;
    let Some(effects) = read_geometric_effects_10_1(
        reader,
        max_bytes,
        refs,
        depth,
        effect_list_version,
        effect_count,
    )?
    else {
        return Ok(None);
    };
    let pattern = read_object_entry_10_1(reader, refs)?;
    let pattern_body = match pattern {
        ObjectEntry10_1::NewObject {
            class_id:
                SOLID_COLOR_PATTERN_CLASS_ID | LINE_PATTERN_CLASS_ID | GRADIENT_PATTERN_CLASS_ID,
            ..
        } if depth > 1 => {
            let Some(body) = read_graphic_attribute_list_10_1(reader, max_bytes, refs, 0)? else {
                return Ok(None);
            };
            Some(body)
        }
        ObjectEntry10_1::BackReference {
            class_id:
                SOLID_COLOR_PATTERN_CLASS_ID | LINE_PATTERN_CLASS_ID | GRADIENT_PATTERN_CLASS_ID,
            ..
        } => None,
        _ => return Ok(None),
    };
    Ok(Some(BasicFillSymbol10_1 {
        version,
        effect_list_version,
        effect_count,
        effects,
        pattern,
        pattern_body,
        trailing_value: read_i32_le(reader)?,
    }))
}

fn read_basic_line_symbol_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<Option<BasicLineSymbol10_1>> {
    let version = read_signed_symbol_version(reader, 1, "BasicLineSymbol", reader.position())?;
    let effect_list_version =
        read_signed_symbol_version(reader, 2, "BasicLineSymbol 效果列表", reader.position())?;
    let effect_count = read_i32_le(reader)?;
    let Some(effects) = read_geometric_effects_10_1(
        reader,
        max_bytes,
        refs,
        depth,
        effect_list_version,
        effect_count,
    )?
    else {
        return Ok(None);
    };
    let stroke = read_object_entry_10_1(reader, refs)?;
    let stroke_body = match stroke {
        ObjectEntry10_1::NewObject {
            class_id: LINE_STROKE_CLASS_ID,
            ..
        } if depth > 1 => {
            let Some(body) = read_graphic_attribute_list_10_1(reader, max_bytes, refs, 0)? else {
                return Ok(None);
            };
            Some(body)
        }
        ObjectEntry10_1::BackReference {
            class_id: LINE_STROKE_CLASS_ID,
            ..
        } => None,
        _ => return Ok(None),
    };
    Ok(Some(BasicLineSymbol10_1 {
        version,
        effect_list_version,
        effect_count,
        effects,
        stroke,
        stroke_body,
        trailing_value: read_i32_le(reader)?,
    }))
}

fn read_basic_marker_symbol_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<Option<BasicMarkerSymbol10_1>> {
    let version = read_signed_symbol_version(reader, 2, "BasicMarkerSymbol", reader.position())?;
    let effect_list_version =
        read_signed_symbol_version(reader, 2, "BasicMarkerSymbol 效果列表", reader.position())?;
    let effect_count = read_i32_le(reader)?;
    let Some(effects) = read_geometric_effects_10_1(
        reader,
        max_bytes,
        refs,
        depth,
        effect_list_version,
        effect_count,
    )?
    else {
        return Ok(None);
    };
    let Some(attributes) = read_graphic_attribute_list_10_1(reader, max_bytes, refs, 0)? else {
        return Ok(None);
    };
    let placement_name = ((version as i16) >= 2)
        .then(|| read_string_10_1(reader, max_bytes))
        .transpose()?;

    let Some((placement, placement_body)) = read_extra_safe_attributes_10_1(
        reader,
        max_bytes,
        refs,
        depth,
        &MARKER_PLACEMENT_CLASSES_10_1,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(BasicMarkerSymbol10_1 {
        version,
        effect_list_version,
        effect_count,
        effects,
        attributes,
        placement_name,
        placement,
        placement_body,
        trailing_value: read_i32_le(reader)?,
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepresentationRuleBody10_1 {
    pub version: u16,
    pub effect_list_version: u16,
    pub effect_count: i32,
    pub effects: Vec<GeometricEffectEntry10_1>,
    pub leading_value: i32,
    pub layer_count: i32,
    pub layers: Vec<RepresentationRuleLayer10_1>,
    pub override_count: i32,
    pub overrides: Vec<RepresentationRuleOverride10_1>,
    pub order_count: i32,
    pub order_ids: Vec<i32>,
    pub trailing_value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepresentationRuleOverride10_1 {
    pub ids: [i32; 2],
    pub value: String,
}

fn is_representation_layer(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        BASIC_FILL_SYMBOL_CLASS_ID | BASIC_LINE_SYMBOL_CLASS_ID | BASIC_MARKER_SYMBOL_CLASS_ID
    )
}

fn read_representation_layer_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
    entry: ObjectEntry10_1,
) -> io::Result<Option<RepresentationRuleLayer10_1>> {
    let (fill, line, marker) = match entry {
        ObjectEntry10_1::NewObject {
            class_id: BASIC_FILL_SYMBOL_CLASS_ID,
            ..
        } => {
            let Some(body) = read_basic_fill_symbol_10_1(reader, max_bytes, refs, depth)? else {
                return Ok(None);
            };
            (Some(body), None, None)
        }
        ObjectEntry10_1::NewObject {
            class_id: BASIC_LINE_SYMBOL_CLASS_ID,
            ..
        } => {
            let Some(body) = read_basic_line_symbol_10_1(reader, max_bytes, refs, depth)? else {
                return Ok(None);
            };
            (None, Some(body), None)
        }
        ObjectEntry10_1::NewObject {
            class_id: BASIC_MARKER_SYMBOL_CLASS_ID,
            ..
        } => {
            let Some(body) = read_basic_marker_symbol_10_1(reader, max_bytes, refs, depth)? else {
                return Ok(None);
            };
            (None, None, Some(body))
        }
        ObjectEntry10_1::BackReference {
            class_id:
                BASIC_FILL_SYMBOL_CLASS_ID | BASIC_LINE_SYMBOL_CLASS_ID | BASIC_MARKER_SYMBOL_CLASS_ID,
            ..
        } => (None, None, None),
        _ => return Ok(None),
    };
    Ok(Some(RepresentationRuleLayer10_1 {
        entry,
        fill,
        line,
        marker,
    }))
}

fn read_representation_rule_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    refs: &mut RefTable10_1,
    depth: usize,
) -> io::Result<Option<RepresentationRuleBody10_1>> {
    let version = read_signed_symbol_version(reader, 1, "RepresentationRule", reader.position())?;
    let effect_list_version =
        read_signed_symbol_version(reader, 2, "RepresentationRule 效果列表", reader.position())?;
    let effect_count = read_i32_le(reader)?;
    let Some(effects) = read_geometric_effects_10_1(
        reader,
        max_bytes,
        refs,
        depth,
        effect_list_version,
        effect_count,
    )?
    else {
        return Ok(None);
    };
    let leading_value = read_i32_le(reader)?;
    let layer_count = read_i32_le(reader)?;
    let count = layer_count.max(0) as usize;
    if count > reader.remaining().min(max_bytes) / 16 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RepresentationRule 符号层计数超出剩余字节或限额",
        ));
    }
    let mut layers = Vec::new();
    for _ in 0..count {
        let entry = read_object_entry_10_1(reader, refs)?;
        if depth <= 1 && matches!(entry, ObjectEntry10_1::NewObject { .. }) {
            return Ok(None);
        }
        let Some(layer) = read_representation_layer_body_10_1(
            reader,
            max_bytes,
            refs,
            depth.saturating_sub(1),
            entry,
        )?
        else {
            return Ok(None);
        };
        layers.push(layer);
    }
    let override_count = read_i32_le(reader)?;
    let overrides_len = override_count.max(0) as usize;
    if overrides_len > reader.remaining().min(max_bytes) / 14 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RepresentationRule 覆盖值计数超出剩余字节或限额",
        ));
    }
    let mut overrides = Vec::new();
    for _ in 0..overrides_len {
        overrides.push(RepresentationRuleOverride10_1 {
            ids: [read_i32_le(reader)?, read_i32_le(reader)?],
            value: read_string_10_1(reader, max_bytes)?,
        });
    }

    let order_count = read_i32_le(reader)?;
    let order_len = order_count.max(0) as u32;
    if order_len as usize > reader.remaining().min(max_bytes) / 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "RepresentationRule 次序计数超出剩余字节或限额",
        ));
    }
    let order_ids = read_i32_vec(reader, order_len)?;
    Ok(Some(RepresentationRuleBody10_1 {
        version,
        effect_list_version,
        effect_count,
        effects,
        leading_value,
        layer_count,
        layers,
        override_count,
        overrides,
        order_count,
        order_ids,
        trailing_value: read_i32_le(reader)?,
    }))
}

pub(crate) const MARKER_SYMBOL_CLASSES: &[[u8; 16]] = &[
    SIMPLE_MARKER_SYMBOL_CLASS_ID,
    MULTI_LAYER_MARKER_SYMBOL_CLASS_ID,
    CHARACTER_MARKER_SYMBOL_CLASS_ID,
    PICTURE_MARKER_SYMBOL_CLASS_ID,
    ARROW_MARKER_SYMBOL_CLASS_ID,
    URL_PICTURE_MARKER_SYMBOL_CLASS_ID,
];

pub(crate) const LINE_SYMBOL_CLASSES: &[[u8; 16]] = &[
    SIMPLE_LINE_SYMBOL_CLASS_ID,
    MARKER_LINE_SYMBOL_CLASS_ID,
    CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID,
    HASH_LINE_SYMBOL_CLASS_ID,
    MULTI_LAYER_LINE_SYMBOL_CLASS_ID,
    PICTURE_LINE_SYMBOL_CLASS_ID,
];

pub(crate) const FILL_SYMBOL_CLASSES: &[[u8; 16]] = &[
    SIMPLE_FILL_SYMBOL_CLASS_ID,
    MULTI_LAYER_FILL_SYMBOL_CLASS_ID,
    PICTURE_FILL_SYMBOL_CLASS_ID,
    COLOR_SYMBOL_CLASS_ID,
    COLOR_RAMP_SYMBOL_CLASS_ID,
    RASTER_RGB_SYMBOL_CLASS_ID,
    LINE_FILL_SYMBOL_CLASS_ID,
    MARKER_FILL_SYMBOL_CLASS_ID,
    GRADIENT_FILL_SYMBOL_CLASS_ID,
    DOT_DENSITY_FILL_SYMBOL_CLASS_ID,
];

/// 槽位在体字段序中的类别（停止原因与诊断用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySlotKind10_1 {
    /// ISymbol 槽（SimpleFillSymbol 轮廓、HashLineSymbol 首槽、
    /// SimpleLineDecorationElement 符号槽等）。
    Symbol,
    /// IMarkerSymbol 槽（MarkerLineSymbol 首槽、DotDensityFillSymbol 点数组）。
    MarkerSymbol,
    /// IColor 槽（各符号的颜色槽）。
    Color,
    /// ITemplate 槽（MarkerLine/CartographicLine/HashLine）。
    Template,
    /// ILineDecoration 槽（同上三类）。
    LineDecoration,
    /// ILineSymbol 槽（MultiLayerLineSymbol 层数组）。
    LineSymbol,
    /// IFillSymbol 槽（MultiLayerFillSymbol 层数组）。
    FillSymbol,
    /// IColorRamp 槽（GradientFillSymbol；实现类未闭合，新对象 = 停止）。
    ColorRamp,
    /// IFontDisp 槽（CharacterMarkerSymbol 字体槽；实现类未闭合 = 停止）。
    Font,
    /// IMarkerBackground 槽（MultiLayerMarkerSymbol 版本 ≥2；实现类未闭合）。
    MarkerBackground,
    /// ITextBackground 槽（TextSymbol 的文字背景）。
    TextBackground,
    /// IPicture 槽（PictureMarkerSymbol 图像类型 3 / 历史路径；实现类未闭合）。
    Picture,
    /// IPictureFillSymbol 槽（PictureLineSymbol；实现类未闭合 = 停止）。
    PictureFillSymbol,
    /// ILineDecorationElement 槽（LineDecoration 元素数组）。
    LineDecorationElement,
}

/// 符号体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolStop10_1 {
    /// 本体内某槽位出现未闭合的新对象类（游标已由槽位读取器回退到该槽位
    /// 起点，槽位及其后的本体字段未消费）。
    Slot {
        kind: DisplaySlotKind10_1,
        class_id: [u8; 16],
        key: u32,
    },
    /// 嵌套符号体内出现停止（游标停在内层停止点；内层停止原因见嵌套体
    /// 自身的 `stop` 字段，逐层可查）。
    NestedIncomplete { class_id: [u8; 16], key: u32 },
}

/// 归约符号类槽位的停止：`Unsupported` → 本体槽位停止；嵌套体含 `stop` →
/// 嵌套未完整停止；其余无停止。`kind` 为调用者槽位的类别（进入停止原因）。
/// 公开给宿主（渲染器/图例类/元素体）在分派后透传停止原因。
pub fn slot_stop_10_1(
    slot: &DisplaySymbolSlot10_1,
    kind: DisplaySlotKind10_1,
) -> Option<SymbolStop10_1> {
    match slot {
        DisplaySymbolSlot10_1::Unsupported { class_id, key } => Some(SymbolStop10_1::Slot {
            kind,
            class_id: *class_id,
            key: *key,
        }),
        DisplaySymbolSlot10_1::SimpleMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::SimpleLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::SimpleFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::MarkerLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MARKER_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::CartographicLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        DisplaySymbolSlot10_1::HashLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: HASH_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        _ => None,
    }
}

/// 归约颜色槽位的停止：`Unsupported` → 本体槽位停止；其余无停止。
fn color_slot_stop(color: &ColorSlot10_1) -> Option<SymbolStop10_1> {
    match color {
        ColorSlot10_1::Unsupported { class_id, key } => Some(SymbolStop10_1::Slot {
            kind: DisplaySlotKind10_1::Color,
            class_id: *class_id,
            key: *key,
        }),
        _ => None,
    }
}

/// SimpleMarkerSymbol 体的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleMarkerSymbolStop10_1 {
    /// 首个 IColor 槽的新对象类标识不在五类闭合颜色集合（同族未知颜色类）：
    /// 游标回退到该槽位起点，其后的标量字段不可达。
    ColorUnsupported { class_id: [u8; 16], key: u32 },
    /// 第二个 IColor 槽（轮廓色）同类停止；首个颜色与标量段已消费。
    OutlineColorUnsupported { class_id: [u8; 16], key: u32 },
}

/// SimpleMarkerSymbol（DisplayCore.dll `0x6ebbefa0`，第七轮闭合）体的已确认
/// 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMarkerSymbolPrefix10_1 {
    /// 对象头键（样本 36）。
    pub key: u32,
    /// 体首 short 版本（样本 2）；有符号比较 >2 拒绝。
    pub version: u16,
    /// IColor 槽（主色）：空标记 / 回引 / 五类闭合颜色体 / 不支持停止。
    pub color: ColorSlot10_1,
    /// double → obj-0x6c（样本 6.0；Size 的存储值）。`None` = 首色槽停止。
    pub size: Option<f64>,
    /// ISimpleMarkerSymbol.Style（样本 0）。
    pub int32_a: Option<i32>,
    /// ISymbol.ROP2（样本 13）。
    pub int32_b: Option<i32>,
    /// MapLevel（样本 0）。
    pub int32_c: Option<i32>,
    /// 依次为 Angle、XOffset、YOffset 的存储值（样本均为 0）。
    pub doubles: Option<[f64; 3]>,
    /// Outline 的原始字节（样本 1）。
    pub byte_flag: Option<u8>,
    /// OutlineSize 的存储值（样本 1.0）。
    pub double_ratio: Option<f64>,
    /// 第二个 IColor 槽（轮廓色）。`None` = 在其之前已停止。
    pub outline_color: Option<ColorSlot10_1>,
    /// 版本 >1 的 short → obj-0xb4（样本 0xFFFF）。`None` = 版本 ≤1 或前置
    /// 停止未读取。
    pub rotate_short: Option<i16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SimpleMarkerSymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleMarkerSymbol 对象体，失败时不推进游
/// 标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_MARKER_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。两个 IColor 槽经 [`read_color_slot_10_1`] 分派（五类闭合
/// 颜色全部解析；接线轮起对象层共享，原实现位于 `feature_layer_tail`），
/// 新对象类标识不在闭合集合时是成功停止（停止原因携带类标识与键，游标回
/// 退到该槽位起点）；截断与版本超限为错误。
pub fn read_simple_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleMarkerSymbolPrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged_table = table.clone();

    // 1. 版本 short（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SIMPLE_MARKER_VERSION,
        "SimpleMarkerSymbol",
        entry_offset,
    )?;

    // 2. 首个 IColor 槽（主色）：五类闭合颜色经分派整体解析；未知颜色类
    //    由分派内部回退到槽位起点、登记表不保留该键，此处映射为本体停止。
    let (color, stop) = match read_color_slot_10_1(&mut trial, max_bytes, &mut staged_table)? {
        ColorSlot10_1::Unsupported { class_id, key } => (
            ColorSlot10_1::Null,
            Some(SimpleMarkerSymbolStop10_1::ColorUnsupported { class_id, key }),
        ),
        other => (other, None),
    };

    // 3-8. 标量段：double、int32 ×3、double ×3、byte、double（前置停止时不
    //       读取）。
    let mut size = None;
    let mut int32_a = None;
    let mut int32_b = None;
    let mut int32_c = None;
    let mut doubles = None;
    let mut byte_flag = None;
    let mut double_ratio = None;
    let mut outline_color = None;
    let mut rotate_short = None;
    let mut stop = stop;
    if stop.is_none() {
        size = Some(trial.read_f64_le()?);
        int32_a = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        int32_b = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        int32_c = Some(i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap()));
        let mut values = [0_f64; 3];
        for slot in values.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        doubles = Some(values);
        byte_flag = Some(trial.read_bytes(1)?[0]);
        double_ratio = Some(trial.read_f64_le()?);

        // 9. 第二个 IColor 槽（轮廓色）：同一分派；停止时首个颜色与标量段
        //    已消费并保留，游标停在该槽位起点。
        outline_color = Some(
            match read_color_slot_10_1(&mut trial, max_bytes, &mut staged_table)? {
                ColorSlot10_1::Unsupported { class_id, key } => {
                    stop =
                        Some(SimpleMarkerSymbolStop10_1::OutlineColorUnsupported { class_id, key });
                    ColorSlot10_1::Null
                }
                other => other,
            },
        );

        // 10. 版本 >1 的 short（前置停止时不读取）。
        if stop.is_none() && (version as i16) > 1 {
            rotate_short = Some(trial.read_u16_le()? as i16);
        }
    }

    let result = SimpleMarkerSymbolPrefix10_1 {
        key,
        version,
        color,
        size,
        int32_a,
        int32_b,
        int32_c,
        doubles,
        byte_flag,
        double_ratio,
        outline_color,
        rotate_short,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// SimpleLineSymbol（DisplayCore.dll `0x6ebbced0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// IColor 槽（请求 IID `{20CD40B0-…}`）：空标记 / 回引 / 五类闭合颜色体 /
    /// 不支持停止。
    pub color: ColorSlot10_1,
    /// double → this−0x24（Width 的持久化值）。`None` = 颜色槽停止。
    pub width: Option<f64>,
    /// ISimpleLineSymbol.Style（this−0x28）。`None` = 前置停止。
    pub style: Option<i32>,
    /// ISymbol.ROP2（this−0x7c）。`None` = 前置停止。
    pub int32_7c: Option<i32>,
    /// MapLevel（this−0x74）。`None` = 前置停止。
    pub int32_74: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_LINE_SYMBOL_CLASS_ID`] 并取
/// 得键 `key`。颜色槽为未闭合新对象时是成功停止（游标已回退到该槽位起点，
/// 其后标量字段不可达）；截断与版本超限为错误。
pub fn read_simple_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<SimpleLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "SimpleLineSymbol",
        entry_offset,
    )?;
    // ② IColor 槽（读取器内部对未知类回退）。
    let color = read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?;
    let stop = color_slot_stop(&color);
    // ③ `FUN_6eb406f0` 无流内字节（前轮证据），跳过。
    // ④-⑦ 标量段（前置停止时不读取）。
    let (width, style, int32_7c, int32_74) = if stop.is_none() {
        let width = trial.read_f64_le()?;
        let style = read_i32_le(&mut trial)?;
        let a = read_i32_le(&mut trial)?;
        let b = read_i32_le(&mut trial)?;
        (Some(width), Some(style), Some(a), Some(b))
    } else {
        (None, None, None, None)
    };

    *table = staged;
    *reader = trial;
    Ok(SimpleLineSymbolBody10_1 {
        key,
        version,
        color,
        width,
        style,
        int32_7c,
        int32_74,
        stop,
    })
}

/// SimpleFillSymbol（DisplayCore.dll `0x6ebbb4e0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// ISymbol 槽（请求 IID `{F3435802-…}`，轮廓线符号候选）。
    pub outline: SymbolSlotFull10_1,
    /// IColor 槽（填充色候选，Save 侧槽 `this−0x28`）。`None` = 轮廓槽停止。
    pub color: Option<ColorSlot10_1>,
    /// ISimpleFillSymbol.Style（this−0xc）。`None` = 前置停止。
    pub style: Option<i32>,
    /// MapLevel（this−0x74）。`None` = 前置停止。
    pub int32_74: Option<i32>,
    /// ISymbol.ROP2（this−0x7c）。
    pub int32_7c: Option<i32>,
    /// Style 的原始值（this−0xc），保留既有字段。`None` = 前置停止。
    pub int32_c: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 SimpleFillSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`SIMPLE_FILL_SYMBOL_CLASS_ID`] 并取
/// 得键 `key`。轮廓槽（ISymbol）经 [`read_symbol_slot_full_10_1`] 分派，
/// 未闭合新对象或嵌套体停止时其后的颜色槽与标量不可达（成功停止）。
pub fn read_simple_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SimpleFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "SimpleFillSymbol",
        entry_offset,
    )?;
    // ② ISymbol 槽（轮廓线符号候选）。
    let outline =
        read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut staged, depth.saturating_sub(1))?;
    let mut stop = slot_full_stop_10_1(&outline, DisplaySlotKind10_1::Symbol);
    // ③ IColor 槽 + ④-⑦ 标量段（前置停止时不读取）。
    let mut color = None;
    let mut int32_7c = None;
    let mut int32_74 = None;
    let mut int32_c = None;
    if stop.is_none() {
        color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
        if let Some(inner) = color_slot_stop(color.as_ref().unwrap()) {
            stop = Some(inner);
        } else {
            int32_7c = Some(read_i32_le(&mut trial)?);
            int32_74 = Some(read_i32_le(&mut trial)?);
            int32_c = Some(read_i32_le(&mut trial)?);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(SimpleFillSymbolBody10_1 {
        key,
        version,
        outline,
        color,
        style: int32_c,
        int32_7c,
        int32_74,
        int32_c,
        stop,
    })
}

/// MarkerLineSymbol（DisplayCore.dll `0x6eb62810`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝（Save 恒写 2）。
    pub version: u16,
    /// byte → this−0x20d8，Flip。
    pub byte_20d8: u8,
    /// double → this−0x20d4，Offset。
    pub double_20d4: f64,
    /// IMarkerSymbol 槽（请求 IID `{E6BDAA7C-…}`）。
    pub marker_symbol: SymbolSlotFull10_1,
    /// `{7914E5F4}` ITemplate 候选槽；新对象 coclass 体未闭合 = `None` 停止。
    pub template: Option<DecorationSlot10_1>,
    /// `{533D88F0}` ILineDecoration 候选槽；新对象 coclass 体未闭合 = `None`
    /// 停止。`None` 亦表示前置停止未读取。
    pub line_decoration: Option<DecorationSlot10_1>,
    /// int32 → this−0x2134，ROP2。`None` = 前置停止。
    pub int32_2134: Option<i32>,
    /// int32 → this−0x212c，MapLevel。`None` = 前置停止。
    pub int32_212c: Option<i32>,
    /// byte → this−0x20ac，DecorationOnTop。`None` = 前置停止。
    pub byte_20ac: Option<u8>,
    /// double → this−0x20a4，LineStartOffset。`None` = 前置停止。
    pub double_20a4: Option<f64>,
    /// 版本 ≥2 的 Cap（this−0x2094）。`None` = 版本 ≤1 或前置停止。
    pub int32_2094: Option<i32>,
    /// 版本 ≥2 的 Join（this−0x2090）。`None` = 版本 ≤1 或前置停止。
    pub int32_2090: Option<i32>,
    /// 版本 ≥2 的 MiterLimit（this−0x208c）。`None` = 版本 ≤1 或前置停止。
    pub double_208c: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MarkerLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`MARKER_LINE_SYMBOL_CLASS_ID`] 并取
/// 得键 `key`。ITemplate/ILineDecoration 候选槽的新对象（coclass 体未闭合）
/// 为成功停止（游标已回退到该槽位起点）；版本 ≥2 的尾段在前置停止时不读取。
pub fn read_marker_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MarkerLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >2 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_MARKER_LINE_VERSION,
        "MarkerLineSymbol",
        entry_offset,
    )?;
    // ②③ byte + double。
    let byte_20d8 = trial.read_bytes(1)?[0];
    let double_20d4 = trial.read_f64_le()?;
    // ④ IMarkerSymbol 槽（⑤ `FUN_6eb68fc0` 无流内字节，跳过）。
    let marker_symbol =
        read_symbol_slot_full_10_1(&mut trial, max_bytes, &mut staged, depth.saturating_sub(1))?;
    let mut stop = slot_full_stop_10_1(&marker_symbol, DisplaySlotKind10_1::MarkerSymbol);
    // ⑥⑦ ITemplate / ILineDecoration 候选槽（coclass 体未闭合）。
    let mut template = None;
    let mut line_decoration = None;
    if stop.is_none() {
        let (slot, inner) = read_decoration_slot_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth,
            DisplaySlotKind10_1::Template,
        )?;
        template = slot;
        if let Some(inner) = inner {
            stop = Some(inner);
        } else {
            let (slot, inner) = read_decoration_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth,
                DisplaySlotKind10_1::LineDecoration,
            )?;
            line_decoration = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    // ⑧-⑪ 标量段。
    let mut int32_2134 = None;
    let mut int32_212c = None;
    let mut byte_20ac = None;
    let mut double_20a4 = None;
    // ⑫ 版本 ≥2 尾段。
    let mut int32_2094 = None;
    let mut int32_2090 = None;
    let mut double_208c = None;
    if stop.is_none() {
        int32_2134 = Some(read_i32_le(&mut trial)?);
        int32_212c = Some(read_i32_le(&mut trial)?);
        byte_20ac = Some(trial.read_bytes(1)?[0]);
        double_20a4 = Some(trial.read_f64_le()?);
        if version as i16 >= 2 {
            int32_2094 = Some(read_i32_le(&mut trial)?);
            int32_2090 = Some(read_i32_le(&mut trial)?);
            double_208c = Some(trial.read_f64_le()?);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(MarkerLineSymbolBody10_1 {
        key,
        version,
        byte_20d8,
        double_20d4,
        marker_symbol,
        template,
        line_decoration,
        int32_2134,
        int32_212c,
        byte_20ac,
        double_20a4,
        int32_2094,
        int32_2090,
        double_208c,
        stop,
    })
}

/// CartographicLineSymbol（DisplayCore.dll `0x6eb99140`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct CartographicLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// int32 → this−0x10a8，Cap。
    pub int32_10a8: i32,
    /// int32 → this−0x10a4，Join。
    pub int32_10a4: i32,
    /// double → this−0x109c，Width。
    pub double_109c: f64,
    /// byte → this−0x1094，Flip。
    pub byte_1094: u8,
    /// double → this−0x108c，Offset。
    pub double_108c: f64,
    /// IColor 槽（Save 槽 this−0x10ac）。`None` = 前置停止（正常流不可能，
    /// 标量段先于颜色槽，保留 defensive 语义）。
    pub color: Option<ColorSlot10_1>,
    /// `{7914E5F4}` ITemplate 候选槽；新对象 = `None` 停止。
    pub template: Option<DecorationSlot10_1>,
    /// `{533D88F0}` ILineDecoration 候选槽；新对象 = `None` 停止。
    pub line_decoration: Option<DecorationSlot10_1>,
    /// int32 → this−0x10fc，ROP2。`None` = 前置停止。
    pub int32_10fc: Option<i32>,
    /// int32 → this−0x10f4，MapLevel。`None` = 前置停止。
    pub int32_10f4: Option<i32>,
    /// byte → this−0x1060，DecorationOnTop。`None` = 前置停止。
    pub byte_1060: Option<u8>,
    /// double → this−0x105c，LineStartOffset。`None` = 前置停止。
    pub double_105c: Option<f64>,
    /// double → this−0x1054，MiterLimit。`None` = 前置停止。
    pub double_1054: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 CartographicLineSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID`]
/// 并取得键 `key`。ITemplate/ILineDecoration 候选槽新对象为成功停止。
pub fn read_cartographic_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<CartographicLineSymbolBody10_1> {
    let _ = depth; // 颜色槽无嵌套下钻；符号类分派不需要深度
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "CartographicLineSymbol",
        entry_offset,
    )?;
    // ②-⑥ 标量段。
    let int32_10a8 = read_i32_le(&mut trial)?;
    let int32_10a4 = read_i32_le(&mut trial)?;
    let double_109c = trial.read_f64_le()?;
    let byte_1094 = trial.read_bytes(1)?[0];
    let double_108c = trial.read_f64_le()?;
    // ⑦ IColor 槽（⑧ `FUN_6eb406f0` 无流内字节，跳过）。
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    // ⑨⑩ ITemplate / ILineDecoration 候选槽。
    let mut template = None;
    let mut line_decoration = None;
    if stop.is_none() {
        let (slot, inner) = read_decoration_slot_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth,
            DisplaySlotKind10_1::Template,
        )?;
        template = slot;
        if let Some(inner) = inner {
            stop = Some(inner);
        } else {
            let (slot, inner) = read_decoration_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth,
                DisplaySlotKind10_1::LineDecoration,
            )?;
            line_decoration = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    // ⑪-⑮ 标量段（前置停止时不读取）。
    let mut int32_10fc = None;
    let mut int32_10f4 = None;
    let mut byte_1060 = None;
    let mut double_105c = None;
    let mut double_1054 = None;
    if stop.is_none() {
        int32_10fc = Some(read_i32_le(&mut trial)?);
        int32_10f4 = Some(read_i32_le(&mut trial)?);
        byte_1060 = Some(trial.read_bytes(1)?[0]);
        double_105c = Some(trial.read_f64_le()?);
        double_1054 = Some(trial.read_f64_le()?);
    }

    *table = staged;
    *reader = trial;
    Ok(CartographicLineSymbolBody10_1 {
        key,
        version,
        int32_10a8,
        int32_10a4,
        double_109c,
        byte_1094,
        double_108c,
        color,
        template,
        line_decoration,
        int32_10fc,
        int32_10f4,
        byte_1060,
        double_105c,
        double_1054,
        stop,
    })
}

/// HashLineSymbol（DisplayCore.dll `0x6eb554a0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct HashLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（Save 恒写 1）。
    pub version: u16,
    /// double → this−0x2124，Angle。
    pub double_2124: f64,
    /// int32 → this−0x2118，Cap。
    pub int32_2118: i32,
    /// int32 → this−0x2114，Join。
    pub int32_2114: i32,
    /// double → this−0x2104，Width。
    pub double_2104: f64,
    /// byte → this−0x20fc，Flip。
    pub byte_20fc: u8,
    /// double → this−0x20f4，Offset。
    pub double_20f4: f64,
    /// ISymbol 槽（Save 槽 this−0x20e4，语义候选：被刻线的线符号）。
    pub symbol: Option<SymbolSlotFull10_1>,
    /// IColor 槽（Save 槽 this−0x211c）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// `{7914E5F4}` ITemplate 候选槽；新对象 = `None` 停止。
    pub template: Option<DecorationSlot10_1>,
    /// `{533D88F0}` ILineDecoration 候选槽；新对象 = `None` 停止。
    pub line_decoration: Option<DecorationSlot10_1>,
    /// int32 → this−0x217c，ROP2。`None` = 前置停止。
    pub int32_217c: Option<i32>,
    /// int32 → this−0x2174，MapLevel。`None` = 前置停止。
    pub int32_2174: Option<i32>,
    /// byte → this−0x20a4，DecorationOnTop。`None` = 前置停止。
    pub byte_20a4: Option<u8>,
    /// double → this−0x209c，LineStartOffset。`None` = 前置停止。
    pub double_209c: Option<f64>,
    /// double → this−0x210c，MiterLimit。`None` = 前置停止。
    pub double_210c: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 HashLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`HASH_LINE_SYMBOL_CLASS_ID`] 并取得
/// 键 `key`。ISymbol 槽经 [`read_symbol_slot_full_10_1`] 分派（未知类或
/// 嵌套体停止时其后的槽位与标量不可达）；ITemplate/ILineDecoration 候选槽
/// 新对象为成功停止。
pub fn read_hash_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<HashLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    // ① short 版本（有符号 >1 拒绝）。
    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "HashLineSymbol",
        entry_offset,
    )?;
    // ②-⑦ 标量段。
    let double_2124 = trial.read_f64_le()?;
    let int32_2118 = read_i32_le(&mut trial)?;
    let int32_2114 = read_i32_le(&mut trial)?;
    let double_2104 = trial.read_f64_le()?;
    let byte_20fc = trial.read_bytes(1)?[0];
    let double_20f4 = trial.read_f64_le()?;
    // ⑧ ISymbol 槽。
    let symbol = Some(read_symbol_slot_full_10_1(
        &mut trial,
        max_bytes,
        &mut staged,
        depth.saturating_sub(1),
    )?);
    let mut stop = slot_full_stop_10_1(symbol.as_ref().unwrap(), DisplaySlotKind10_1::Symbol);
    // ⑨ IColor 槽（⑩ `FUN_6eb406f0` 无流内字节，跳过）。
    let mut color = None;
    if stop.is_none() {
        color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
        if let Some(inner) = color_slot_stop(color.as_ref().unwrap()) {
            stop = Some(inner);
        }
    }
    // ⑪⑫ ITemplate / ILineDecoration 候选槽。
    let mut template = None;
    let mut line_decoration = None;
    if stop.is_none() {
        let (slot, inner) = read_decoration_slot_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth,
            DisplaySlotKind10_1::Template,
        )?;
        template = slot;
        if let Some(inner) = inner {
            stop = Some(inner);
        } else {
            let (slot, inner) = read_decoration_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth,
                DisplaySlotKind10_1::LineDecoration,
            )?;
            line_decoration = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    // ⑬-⑰ 标量段（前置停止时不读取）。
    let mut int32_217c = None;
    let mut int32_2174 = None;
    let mut byte_20a4 = None;
    let mut double_209c = None;
    let mut double_210c = None;
    if stop.is_none() {
        int32_217c = Some(read_i32_le(&mut trial)?);
        int32_2174 = Some(read_i32_le(&mut trial)?);
        byte_20a4 = Some(trial.read_bytes(1)?[0]);
        double_209c = Some(trial.read_f64_le()?);
        double_210c = Some(trial.read_f64_le()?);
    }

    *table = staged;
    *reader = trial;
    Ok(HashLineSymbolBody10_1 {
        key,
        version,
        double_2124,
        int32_2118,
        int32_2114,
        double_2104,
        byte_20fc,
        double_20f4,
        symbol,
        color,
        template,
        line_decoration,
        int32_217c,
        int32_2174,
        byte_20a4,
        double_209c,
        double_210c,
        stop,
    })
}

/// ISymbol/IMarkerSymbol 类槽位条目的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DisplaySymbolSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 SimpleMarkerSymbol 对象（前轮闭合；体内停止经其 `stop` 传播）。
    SimpleMarkerSymbol(Box<SimpleMarkerSymbolPrefix10_1>),
    /// 新 SimpleLineSymbol 对象。
    SimpleLineSymbol(Box<SimpleLineSymbolBody10_1>),
    /// 新 SimpleFillSymbol 对象。
    SimpleFillSymbol(Box<SimpleFillSymbolBody10_1>),
    /// 新 MarkerLineSymbol 对象。
    MarkerLineSymbol(Box<MarkerLineSymbolBody10_1>),
    /// 新 CartographicLineSymbol 对象。
    CartographicLineSymbol(Box<CartographicLineSymbolBody10_1>),
    /// 新 HashLineSymbol 对象。
    HashLineSymbol(Box<HashLineSymbolBody10_1>),
    /// 新对象但类标识不在闭合集合：游标已回退到该槽位起点、登记表未保留
    /// 该键（本读取器内部完成回退，调用者据此停止）。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 从当前位置读取一个符号类槽位对象条目并按类标识分派。
///
/// 支持的闭合类：SimpleMarkerSymbol（前轮）、SimpleLineSymbol/
/// SimpleFillSymbol/MarkerLineSymbol/CartographicLineSymbol/HashLineSymbol
/// （第一轮）与 Template/LineDecoration/MultiLayer 三族/CharacterMarker/
/// PictureMarker/PictureLine/ArrowMarker/LineFill/MarkerFill/GradientFill/
/// DotDensityFill（第二轮）。新对象类未知时回退到槽位起点并返回
/// [`DisplaySymbolSlot10_1::Unsupported`]。`depth` 为剩余允许嵌套层数：为 0
/// 时报 `ErrorKind::InvalidInput`（防栈溢出的读取器自身限额，非原生限制）。
/// 全部过程在 trial 游标与 trial 登记表上进行，任何失败不推进调用者游标、
/// 不修改登记表。
pub fn read_display_symbol_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<DisplaySymbolSlot10_1> {
    let origin = reader.clone();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let result = match entry {
        ObjectEntry10_1::Null => DisplaySymbolSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            DisplaySymbolSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 深度限额在需要解析新对象体时才生效（空标记与回引不消耗深度）。
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "符号槽位 offset 0x{:x}: 嵌套深度超过读取器限额 \
                         {MAX_SYMBOL_NESTING_DEPTH}（防栈溢出，非原生限制）",
                        origin.position()
                    ),
                ));
            }
            let slot = if class_id == SIMPLE_MARKER_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::SimpleMarkerSymbol(Box::new(
                    read_simple_marker_symbol_body_10_1(&mut trial, max_bytes, &mut staged, key)?,
                ))
            } else if class_id == SIMPLE_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::SimpleLineSymbol(Box::new(
                    read_simple_line_symbol_body_10_1(&mut trial, max_bytes, &mut staged, key)?,
                ))
            } else if class_id == SIMPLE_FILL_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::SimpleFillSymbol(Box::new(
                    read_simple_fill_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == MARKER_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::MarkerLineSymbol(Box::new(
                    read_marker_line_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::CartographicLineSymbol(Box::new(
                    read_cartographic_line_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == HASH_LINE_SYMBOL_CLASS_ID {
                DisplaySymbolSlot10_1::HashLineSymbol(Box::new(read_hash_line_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else {
                // 未知符号类：回退到槽位起点，不保留该键。
                *reader = origin;
                return Ok(DisplaySymbolSlot10_1::Unsupported { class_id, key });
            };
            slot
        }
    };
    *reader = trial;
    *table = staged;
    Ok(result)
}

/// 全闭合符号类槽位条目（第二轮起的对象层共享入口）。
///
/// 相比 [`DisplaySymbolSlot10_1`]（第一轮闭合集，供 SimpleRenderer/
/// LegendClass/TextSymbol 等既有宿主保持接口稳定）扩展第二轮闭合类：嵌套
/// 的新类符号（MultiLayer 层槽、点密度标记数组、装饰元素符号槽等）在本
/// 入口下整体解析。未知符号类回退到槽位起点并返回
/// [`SymbolSlotFull10_1::Unsupported`]。既有宿主在整合轮切换到本入口即可
/// 获得第二轮符号的递归解析。
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolSlotFull10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference {
        class_id: [u8; 16],
        key: u32,
    },
    /// 新 SimpleMarkerSymbol 对象（第一轮）。
    SimpleMarkerSymbol(Box<SimpleMarkerSymbolPrefix10_1>),
    /// 新 SimpleLineSymbol 对象（第一轮）。
    SimpleLineSymbol(Box<SimpleLineSymbolBody10_1>),
    /// 新 SimpleFillSymbol 对象（第一轮；轮廓槽经第一轮入口解析）。
    SimpleFillSymbol(Box<SimpleFillSymbolBody10_1>),
    /// 新 MarkerLineSymbol 对象（第一轮）。
    MarkerLineSymbol(Box<MarkerLineSymbolBody10_1>),
    /// 新 CartographicLineSymbol 对象（第一轮）。
    CartographicLineSymbol(Box<CartographicLineSymbolBody10_1>),
    /// 新 HashLineSymbol 对象（第一轮）。
    HashLineSymbol(Box<HashLineSymbolBody10_1>),
    /// 新 Template 对象（第二轮）。
    Template(Box<TemplateBody10_1>),
    /// 新 LineDecoration 对象（第二轮）。
    LineDecoration(Box<LineDecorationBody10_1>),
    /// 新 MultiLayerLineSymbol 对象。
    MultiLayerLineSymbol(Box<MultiLayerLineSymbolBody10_1>),
    /// 新 MultiLayerMarkerSymbol 对象。
    MultiLayerMarkerSymbol(Box<MultiLayerMarkerSymbolBody10_1>),
    /// 新 MultiLayerFillSymbol 对象。
    MultiLayerFillSymbol(Box<MultiLayerFillSymbolBody10_1>),
    /// 新 CharacterMarkerSymbol 对象。
    CharacterMarkerSymbol(Box<CharacterMarkerSymbolBody10_1>),
    /// 新 PictureMarkerSymbol 对象。
    PictureMarkerSymbol(Box<PictureMarkerSymbolBody10_1>),
    /// WMSLayer.dll 的 URL 图像标记符号。
    UrlPictureMarkerSymbol(Box<UrlPictureMarkerSymbolBody10_1>),
    /// SymbolMimic 及其表示规则槽。
    SymbolMimic {
        key: u32,
        version: u16,
        leading_value: i32,
        rule: ObjectEntry10_1,
        rule_body: Option<RepresentationRuleBody10_1>,
        trailing_value: i32,
    },
    /// 新 PictureLineSymbol 对象。
    PictureLineSymbol(Box<PictureLineSymbolBody10_1>),
    /// 新 PictureFillSymbol 对象。
    PictureFillSymbol(Box<PictureFillSymbolBody10_1>),
    /// 新 ArrowMarkerSymbol 对象。
    ArrowMarkerSymbol(Box<ArrowMarkerSymbolBody10_1>),
    /// 新 LineFillSymbol 对象。
    LineFillSymbol(Box<LineFillSymbolBody10_1>),
    /// 新 MarkerFillSymbol 对象。
    MarkerFillSymbol(Box<MarkerFillSymbolBody10_1>),
    /// 新 GradientFillSymbol 对象。
    GradientFillSymbol(Box<GradientFillSymbolBody10_1>),
    /// 新 DotDensityFillSymbol 对象。
    DotDensityFillSymbol(Box<DotDensityFillSymbolBody10_1>),
    /// RasterLayer.dll 的颜色、渐变和 RGB 符号。
    ColorSymbol(Box<ColorSymbolBody10_1>),
    ColorRampSymbol(Box<ColorRampSymbolBody10_1>),
    RasterRgbSymbol(Box<ColorSymbolBody10_1>),
    /// 统计图符号及其完整嵌套对象。
    ChartSymbol(Box<ChartSymbolBody10_1>),
    /// 新 TextSymbol 对象（A 轮接线补齐：LELP ⑯ 槽分派已含 TextSymbol，
    /// 本入口与该清单对齐；证据 docs/arcmap-10.1-dep-tail.md 第五节——
    /// ㉖ 槽出现 TextSymbol 新对象时不再按未知类停止回退）。TextSymbol 体
    /// 本身不带键，`key` 为槽位对象头键（嵌套停止归约用）。
    TextSymbol {
        body: Box<TextSymbolBody10_1>,
        key: u32,
    },
    /// 新对象但类标识不在闭合集合：游标已回退到该槽位起点、登记表未保留
    /// 该键（读取器内部完成回退，调用者据此停止）。
    Unsupported {
        class_id: [u8; 16],
        key: u32,
    },
}

/// 归约全闭合符号类槽位的停止（语义同 [`slot_stop_10_1`]，覆盖第二轮闭合
/// 类）：`Unsupported` → 本体槽位停止；嵌套体含 `stop` → 嵌套未完整停止；
/// 其余无停止。`kind` 为调用者槽位的类别。
pub fn slot_full_stop_10_1(
    slot: &SymbolSlotFull10_1,
    kind: DisplaySlotKind10_1,
) -> Option<SymbolStop10_1> {
    match slot {
        SymbolSlotFull10_1::Unsupported { class_id, key } => Some(SymbolStop10_1::Slot {
            kind,
            class_id: *class_id,
            key: *key,
        }),
        SymbolSlotFull10_1::SimpleMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::SimpleLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::SimpleFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: SIMPLE_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::MarkerLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MARKER_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::CartographicLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::HashLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: HASH_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::Template(_) => None,
        SymbolSlotFull10_1::LineDecoration(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: LINE_DECORATION_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::MultiLayerLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MULTI_LAYER_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::MultiLayerMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MULTI_LAYER_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::MultiLayerFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MULTI_LAYER_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::CharacterMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: CHARACTER_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::PictureMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: PICTURE_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::UrlPictureMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: URL_PICTURE_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::PictureLineSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: PICTURE_LINE_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::PictureFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: PICTURE_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::ArrowMarkerSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: ARROW_MARKER_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::LineFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: LINE_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::MarkerFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: MARKER_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::GradientFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: GRADIENT_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::DotDensityFillSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: DOT_DENSITY_FILL_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::ColorSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: COLOR_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::ColorRampSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: COLOR_RAMP_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::RasterRgbSymbol(body) => {
            body.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: RASTER_RGB_SYMBOL_CLASS_ID,
                key: body.key,
            })
        }
        SymbolSlotFull10_1::ChartSymbol(_) | SymbolSlotFull10_1::SymbolMimic { .. } => None,
        SymbolSlotFull10_1::TextSymbol { body, key } => {
            body.stop.clone().map(|_| SymbolStop10_1::NestedIncomplete {
                class_id: TEXT_SYMBOL_CLASS_ID,
                key: *key,
            })
        }
        SymbolSlotFull10_1::Null | SymbolSlotFull10_1::BackReference { .. } => None,
    }
}

/// 从当前位置读取一个符号类槽位对象条目并按类标识分派到全部闭合类
/// （第一轮六类 + SimpleMarkerSymbol + 第二轮十四类）。
///
/// 调用约定与事务性同 [`read_display_symbol_slot_10_1`]：空标记与回引原样
/// 返回；新对象类未知时回退到槽位起点并返回
/// [`SymbolSlotFull10_1::Unsupported`]；`depth` 为剩余允许嵌套层数（0 时报
/// `ErrorKind::InvalidInput`，防栈溢出的读取器自身限额，非原生限制）。
/// 全部过程在 trial 游标与 trial 登记表上进行，任何失败不推进调用者游标、
/// 不修改登记表。
///
/// 第一轮闭合体（SimpleFillSymbol/MarkerLine/CartographicLine/HashLine）的
/// 内部嵌套槽仍经第一轮入口 [`read_display_symbol_slot_10_1`] 解析（其闭
/// 合集以第一轮为界）；第二轮闭合体的嵌套槽经本入口解析。
pub fn read_symbol_slot_full_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<SymbolSlotFull10_1> {
    let origin = reader.clone();
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
    let result = match entry {
        ObjectEntry10_1::Null => SymbolSlotFull10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            SymbolSlotFull10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "符号槽位 offset 0x{:x}: 嵌套深度超过读取器限额 \
                         {MAX_SYMBOL_NESTING_DEPTH}（防栈溢出，非原生限制）",
                        origin.position()
                    ),
                ));
            }
            let slot = if class_id == SIMPLE_MARKER_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::SimpleMarkerSymbol(Box::new(
                    read_simple_marker_symbol_body_10_1(&mut trial, max_bytes, &mut staged, key)?,
                ))
            } else if class_id == SIMPLE_LINE_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::SimpleLineSymbol(Box::new(read_simple_line_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?))
            } else if class_id == SIMPLE_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::SimpleFillSymbol(Box::new(read_simple_fill_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else if class_id == MARKER_LINE_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::MarkerLineSymbol(Box::new(read_marker_line_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else if class_id == CARTOGRAPHIC_LINE_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::CartographicLineSymbol(Box::new(
                    read_cartographic_line_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == HASH_LINE_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::HashLineSymbol(Box::new(read_hash_line_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else if class_id == TEMPLATE_CLASS_ID {
                SymbolSlotFull10_1::Template(Box::new(read_template_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?))
            } else if class_id == LINE_DECORATION_CLASS_ID {
                SymbolSlotFull10_1::LineDecoration(Box::new(read_line_decoration_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth - 1,
                )?))
            } else if class_id == MULTI_LAYER_LINE_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::MultiLayerLineSymbol(Box::new(
                    read_multi_layer_line_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == MULTI_LAYER_MARKER_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::MultiLayerMarkerSymbol(Box::new(
                    read_multi_layer_marker_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == MULTI_LAYER_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::MultiLayerFillSymbol(Box::new(
                    read_multi_layer_fill_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == CHARACTER_MARKER_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::CharacterMarkerSymbol(Box::new(
                    read_character_marker_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                    )?,
                ))
            } else if class_id == PICTURE_MARKER_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::PictureMarkerSymbol(Box::new(
                    read_picture_marker_symbol_body_10_1(&mut trial, max_bytes, &mut staged, key)?,
                ))
            } else if class_id == URL_PICTURE_MARKER_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::UrlPictureMarkerSymbol(Box::new(
                    read_url_picture_marker_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth - 1,
                    )?,
                ))
            } else if class_id == PICTURE_LINE_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::PictureLineSymbol(Box::new(
                    read_picture_line_symbol_body_with_depth(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth - 1,
                    )?,
                ))
            } else if class_id == PICTURE_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::PictureFillSymbol(Box::new(read_picture_fill_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth - 1,
                )?))
            } else if class_id == ARROW_MARKER_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::ArrowMarkerSymbol(Box::new(read_arrow_marker_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?))
            } else if class_id == LINE_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::LineFillSymbol(Box::new(read_line_fill_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else if class_id == MARKER_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::MarkerFillSymbol(Box::new(read_marker_fill_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth,
                )?))
            } else if class_id == GRADIENT_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::GradientFillSymbol(Box::new(
                    read_gradient_fill_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == DOT_DENSITY_FILL_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::DotDensityFillSymbol(Box::new(
                    read_dot_density_fill_symbol_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged,
                        key,
                        depth,
                    )?,
                ))
            } else if class_id == COLOR_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::ColorSymbol(Box::new(read_color_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?))
            } else if class_id == SYMBOL_MIMIC_CLASS_ID {
                let version =
                    read_signed_symbol_version(&mut trial, 1, "SymbolMimic", origin.position())?;
                let leading_value = read_i32_le(&mut trial)?;
                let rule = read_object_entry_10_1(&mut trial, &mut staged)?;
                let rule_body = match rule {
                    ObjectEntry10_1::Null
                    | ObjectEntry10_1::BackReference {
                        class_id: REPRESENTATION_RULE_CLASS_ID,
                        ..
                    } => None,
                    ObjectEntry10_1::NewObject {
                        class_id: REPRESENTATION_RULE_CLASS_ID,
                        ..
                    } if depth > 1 => {
                        let Some(body) = read_representation_rule_body_10_1(
                            &mut trial,
                            max_bytes,
                            &mut staged,
                            depth - 1,
                        )?
                        else {
                            return Ok(SymbolSlotFull10_1::Unsupported { class_id, key });
                        };
                        Some(body)
                    }
                    _ => return Ok(SymbolSlotFull10_1::Unsupported { class_id, key }),
                };
                SymbolSlotFull10_1::SymbolMimic {
                    key,
                    version,
                    leading_value,
                    rule,
                    rule_body,
                    trailing_value: read_i32_le(&mut trial)?,
                }
            } else if class_id == COLOR_RAMP_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::ColorRampSymbol(Box::new(read_color_ramp_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                    depth - 1,
                )?))
            } else if class_id == RASTER_RGB_SYMBOL_CLASS_ID {
                SymbolSlotFull10_1::RasterRgbSymbol(Box::new(read_color_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    key,
                )?))
            } else if matches!(
                class_id,
                BAR_CHART_SYMBOL_CLASS_ID
                    | PIE_CHART_SYMBOL_CLASS_ID
                    | STACKED_CHART_SYMBOL_CLASS_ID
            ) {
                match read_chart_symbol_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    depth - 1,
                    class_id,
                    key,
                ) {
                    Ok(body) => SymbolSlotFull10_1::ChartSymbol(Box::new(body)),
                    Err(error) if error.kind() == ErrorKind::Unsupported => {
                        return Ok(SymbolSlotFull10_1::Unsupported { class_id, key });
                    }
                    Err(error) => return Err(error),
                }
            } else if class_id == TEXT_SYMBOL_CLASS_ID {
                // A 轮接线补齐：TextSymbol 体闭合（element_body 模块，LELP ⑯
                // 槽分派同款），本入口不再按未知类停止回退。键随槽位携带
                // （TextSymbol 体自身无键字段）。
                let body = read_text_symbol_body_10_1(&mut trial, max_bytes, &mut staged)?;
                SymbolSlotFull10_1::TextSymbol {
                    body: Box::new(body),
                    key,
                }
            } else {
                // 未知符号类：回退到槽位起点，不保留该键。
                *reader = origin;
                return Ok(SymbolSlotFull10_1::Unsupported { class_id, key });
            };
            slot
        }
    };
    *reader = trial;
    *table = staged;
    Ok(result)
}

/// ITemplate / ILineDecoration 槽位条目的分类结果（第二轮起按类分派）。
#[derive(Debug, Clone, PartialEq)]
pub enum DecorationSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 Template 对象（体闭合；`0x6eb87ad0`）。
    Template(Box<TemplateBody10_1>),
    /// 新 LineDecoration 对象（体闭合；`0x6eb5c640`；体内元素槽停止经其
    /// `stop` 传播为嵌套未完整）。
    LineDecoration(Box<LineDecorationBody10_1>),
}

/// 读取 ITemplate / ILineDecoration 槽（MarkerLine/CartographicLine/HashLine
/// 三类共用的槽位）：空标记与回引可消费；新对象按类分派——Template/
/// LineDecoration 体已闭合则解析其体，其余实现类回退到槽位起点、不保留该
/// 键，以停止原因返回。
///
/// 返回 `(槽位, 停止)`：停止非 `None` 时槽位为 `None`（新对象的类标识与键
/// 在停止原因中）。`staged` 为调用者（体读取器）的 trial 登记表；回退时
/// 一并恢复，任何失败不修改登记表。`depth` 供嵌套体使用（Template 体无嵌
/// 套对象；LineDecoration → 元素 → 符号槽逐层下钻）。
fn read_decoration_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<DecorationSlot10_1>, Option<SymbolStop10_1>)> {
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(DecorationSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(DecorationSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            if depth == 0 {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "装饰槽位 offset 0x{:x}: 嵌套深度超过读取器限额 \
                         {MAX_SYMBOL_NESTING_DEPTH}（防栈溢出，非原生限制）",
                        origin.position()
                    ),
                ));
            }
            if class_id == TEMPLATE_CLASS_ID {
                let body = read_template_body_10_1(&mut forward, max_bytes, staged, key)?;
                *trial = forward;
                Ok((Some(DecorationSlot10_1::Template(Box::new(body))), None))
            } else if class_id == LINE_DECORATION_CLASS_ID {
                let body = read_line_decoration_body_10_1(
                    &mut forward,
                    max_bytes,
                    staged,
                    key,
                    depth - 1,
                )?;
                let slot = DecorationSlot10_1::LineDecoration(Box::new(body));
                // 体内容嵌套停止（元素槽未知类）以嵌套未完整原因传播；游标
                // 停在内层停止点（forward 已推进）。
                let stop = match &slot {
                    DecorationSlot10_1::LineDecoration(b) => {
                        b.stop.map(|_| SymbolStop10_1::NestedIncomplete {
                            class_id: LINE_DECORATION_CLASS_ID,
                            key,
                        })
                    }
                    _ => None,
                };
                *trial = forward;
                Ok((Some(slot), stop))
            } else {
                // 未知装饰实现类：回退游标并丢弃该键登记。
                *trial = origin;
                *staged = saved;
                Ok((
                    None,
                    Some(SymbolStop10_1::Slot {
                        kind,
                        class_id,
                        key,
                    }),
                ))
            }
        }
    }
}

/// IFontDisp 类槽位条目（StdFont 分派形态；CharacterMarkerSymbol 字体槽 v≤3
/// / v>3 重写门共用）。
///
/// 接线轮起 StdFont 新对象（`{0BE35203-…}`，OLE oleaut32 系统类）经
/// [`read_std_font_body_10_1`] 整体解析（体不含嵌套对象条目、无停止点；体
/// 自身错误上抛即宿主符号体整体失败，与 TextSymbol 字体槽语义一致）；其余
/// 实现类新对象维持停止回退（游标停在槽位起点、键不登记）。证据：
/// docs/arcmap-10.1-ole-stdfont.md「接线建议」2（样本 16 处「ESRI Default
/// Marker」StdFont 位于本槽语境）。
#[derive(Debug, Clone, PartialEq)]
pub enum StdFontSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 StdFont 对象（类标识经 [`STDFONT_CLASS_ID`] 校验）：已解析体。
    StdFont(Box<StdFontBody10_1>),
}

/// 读取一个 IFontDisp 类槽位（StdFont 分派形态），返回 `(槽位, 停止)`。
///
/// StdFont 新对象整体解析；其余实现类新对象 = 停止点（回退语义与
/// [`read_unclosed_slot_10_1`] 相同）。
fn read_std_font_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<StdFontSlot10_1>, Option<SymbolStop10_1>)> {
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(StdFontSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((Some(StdFontSlot10_1::BackReference { class_id, key }), None))
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == STDFONT_CLASS_ID => {
            // 键已由条目分类按「登记先于持久化」语义登记进 staged；体无停
            // 止点，完整成功才提交——失败上抛时本函数不回写 trial/staged，
            // 由宿主体读取器的外层 trial 语义保证整体不提交。max_bytes 约
            // 束 StdFont 名字声明长度（`read_std_font_body_10_1` 契约）。
            let body = read_std_font_body_10_1(&mut forward, max_bytes)?;
            *trial = forward;
            Ok((Some(StdFontSlot10_1::StdFont(Box::new(body))), None))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *trial = origin;
            *staged = saved;
            Ok((
                None,
                Some(SymbolStop10_1::Slot {
                    kind,
                    class_id,
                    key,
                }),
            ))
        }
    }
}

/// IPicture 类槽位条目（StdPicture 分派形态；PictureMarkerSymbol 四处
/// IPicture 槽共用）。
///
/// 接线轮起 StdPicture 新对象（`{0BE35204-…}`，OLE oleaut32 系统类）经
/// [`read_std_picture_body_10_1`] 整体解析（体无停止点）；其余实现类新对
/// 象维持停止回退。证据：docs/arcmap-10.1-ole-stdfont.md「接线建议」1
/// （样本 0 实例，接线契约由合成测试固定）。
#[derive(Debug, Clone, PartialEq)]
pub enum StdPictureSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 StdPicture 对象（类标识经 [`STDPICTURE_CLASS_ID`] 校验）：已解析
    /// 体（图像数据原样保留，不解释格式）。
    StdPicture(Box<StdPictureBody10_1>),
}

/// 读取一个 IPicture 类槽位（StdPicture 分派形态），返回 `(槽位, 停止)`。
fn read_std_picture_slot_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    kind: DisplaySlotKind10_1,
) -> io::Result<(Option<StdPictureSlot10_1>, Option<SymbolStop10_1>)> {
    let origin = trial.clone();
    let saved = staged.clone();
    let mut forward = trial.clone();
    let entry = read_object_entry_10_1(&mut forward, staged)?;
    match entry {
        ObjectEntry10_1::Null => {
            *trial = forward;
            Ok((Some(StdPictureSlot10_1::Null), None))
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            *trial = forward;
            Ok((
                Some(StdPictureSlot10_1::BackReference { class_id, key }),
                None,
            ))
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == STDPICTURE_CLASS_ID => {
            // 键登记与提交语义同 read_std_font_slot_10_1；max_bytes 约束
            // StdPicture 图像数据声明长度。
            let body = read_std_picture_body_10_1(&mut forward, max_bytes)?;
            *trial = forward;
            Ok((Some(StdPictureSlot10_1::StdPicture(Box::new(body))), None))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *trial = origin;
            *staged = saved;
            Ok((
                None,
                Some(SymbolStop10_1::Slot {
                    kind,
                    class_id,
                    key,
                }),
            ))
        }
    }
}

/// 读取一个符号槽并按调用者槽位的类别归约停止原因（`Unsupported` → 本类
/// 槽位停止；嵌套体停止 → 嵌套未完整）。供 MultiLayer 层数组、LineFill/
/// MarkerFill 的多符号槽等循环使用，经全闭合类入口
/// [`read_symbol_slot_full_10_1`] 分派。
fn read_symbol_slot_with_kind_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    staged: &mut RefTable10_1,
    depth: usize,
    kind: DisplaySlotKind10_1,
) -> io::Result<(SymbolSlotFull10_1, Option<SymbolStop10_1>)> {
    let slot = read_symbol_slot_full_10_1(trial, max_bytes, staged, depth)?;
    let stop = slot_full_stop_10_1(&slot, kind);
    Ok((slot, stop))
}

/// 读取流内 int32 循环长度并以 `max_bytes` 设上限（读取器自身限额，非原生
/// 限制；原生按流失败自然终止，限额同时约束预分配）。负值位模式一并拒绝
/// （较原生更严，长度非负）。
fn read_loop_count_10_1(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    what: &str,
    entry_offset: usize,
) -> io::Result<u32> {
    let raw = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if raw < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{what} 对象体 offset 0x{entry_offset:x}: 循环长度 {raw} 为负值位模式"),
        ));
    }
    let count = raw as u32;
    if count as usize > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{what} 对象体 offset 0x{entry_offset:x}: 循环长度 {count} 超过 \
                 读取器 max_bytes 限额 {max_bytes}（读取器自身限额，非原生限制）"
            ),
        ));
    }
    Ok(count)
}

/// 读取 count × int32（循环长度已由 [`read_loop_count_10_1`] 验证）。
fn read_i32_vec(trial: &mut Reader<'_>, count: u32) -> io::Result<Vec<i32>> {
    let mut values = Vec::with_capacity((count as usize).min(4096));
    for _ in 0..count {
        values.push(read_i32_le(trial)?);
    }
    Ok(values)
}

/// 读取 count × String（循环长度已验证；每串声明长度受 `max_bytes` 约束）。
fn read_string_vec(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    count: u32,
) -> io::Result<Vec<String>> {
    let mut values = Vec::with_capacity((count as usize).min(4096));
    for _ in 0..count {
        values.push(read_string_10_1(trial, max_bytes)?);
    }
    Ok(values)
}

/// 读取 PictureMarker 的位图块尺寸（int32；负值与超过 `max_bytes` 的尺寸
/// 拒绝——读取器自身限额，非原生限制）。
fn read_blob_size(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    entry_offset: usize,
) -> io::Result<u32> {
    let raw = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if raw < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "PictureMarkerSymbol 对象体 offset 0x{entry_offset:x}: 位图尺寸 {raw} 为负值位模式"
            ),
        ));
    }
    let size = raw as u32;
    if size as usize > max_bytes {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "PictureMarkerSymbol 对象体 offset 0x{entry_offset:x}: 位图尺寸 {size} \
                 超过读取器 max_bytes 限额 {max_bytes}（读取器自身限额，非原生限制）"
            ),
        ));
    }
    Ok(size)
}

/// 读取位图块数据：size==0 时无流内字节（`None`）；size>0 时消费 size 字节
/// 原样保留（截断报错）。
fn read_blob_data(trial: &mut Reader<'_>, size: u32) -> io::Result<Option<Vec<u8>>> {
    if size == 0 {
        return Ok(None);
    }
    let data = trial.read_bytes(size as usize)?.to_vec();
    Ok(Some(data))
}

/// Template（DisplayCore.dll `0x6eb87ad0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b280`=1）。
    pub version: u16,
    /// double → this−0x94（ITemplate::Interval）。
    pub interval: f64,
    /// 流内 int32 循环长度（PatternElementCount）。
    pub mark_count: u32,
    /// count × 16 字节（每个图案元素依次保存实段长度 mark、空段长度 Gap；
    /// Save 槽 this−0x88 数组）。
    pub marks: Vec<[f64; 2]>,
}

/// 从对象头之后的位置读取 10.1 Template 对象体，失败时不推进游标、不修改
/// 登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`TEMPLATE_CLASS_ID`] 并取得键
/// `key`。体全部为标量，无停止点：要么完整消费，要么失败（截断 / 版本
/// 超限 / 负或超限计数）。
pub fn read_template_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TemplateBody10_1> {
    let _ = table; // Template 体无嵌套对象，登记表仅在签名上保持一致
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    let version = read_version_short(&mut trial, MAX_SYMBOL_VERSION, "Template", entry_offset)?;
    let interval = trial.read_f64_le()?;
    let mark_count = read_loop_count_10_1(&mut trial, max_bytes, "Template", entry_offset)?;
    let mut marks = Vec::with_capacity((mark_count as usize).min(4096));
    for _ in 0..mark_count {
        let mark = trial.read_f64_le()?;
        let gap = trial.read_f64_le()?;
        marks.push([mark, gap]);
    }

    *reader = trial;
    Ok(TemplateBody10_1 {
        key,
        version,
        interval,
        mark_count,
        marks,
    })
}

/// LineDecoration 元素数组中的单条目分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum LineDecorationElement10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 SimpleLineDecorationElement 对象（闭合体；体内停止经其 `stop`
    /// 传播）。
    Element(Box<SimpleLineDecorationElementBody10_1>),
    /// 新对象但类标识不是 SimpleLineDecorationElement：游标已回退到该条目
    /// 起点、登记表未保留该键。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// LineDecoration（DisplayCore.dll `0x6eb5c640`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LineDecorationBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b2ec`=1）。
    pub version: u16,
    /// 流内 int32 循环长度（元素数候选）。
    pub element_count: u32,
    /// count × LoadObject（ILineDecorationElement `{533D88F2-…}` → 集合
    /// this−0x1c）：仅 SimpleLineDecorationElement 体闭合，其余类 = 停止
    /// （游标停在停止条目起点，已解析条目保留）。
    pub elements: Vec<LineDecorationElement10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 LineDecoration 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LINE_DECORATION_CLASS_ID`] 并取得
/// 键 `key`。元素条目新对象且类标识为 SimpleLineDecorationElement 时递归
/// 解析其体（符号槽经 [`read_display_symbol_slot_10_1`] 分派）；其余新类
/// 是成功停止（游标停在该条目起点，其后条目不可达）。`depth` 为嵌套限额。
pub fn read_line_decoration_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<LineDecorationBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "LineDecoration",
        entry_offset,
    )?;
    let element_count =
        read_loop_count_10_1(&mut trial, max_bytes, "LineDecoration", entry_offset)?;
    let mut elements = Vec::with_capacity((element_count as usize).min(4096));
    let mut stop = None;
    for _ in 0..element_count {
        let origin = trial.clone();
        let saved = staged.clone();
        let mut forward = trial.clone();
        let entry = read_object_entry_10_1(&mut forward, &mut staged)?;
        let item = match entry {
            ObjectEntry10_1::Null => LineDecorationElement10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                LineDecorationElement10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                if class_id == SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID {
                    if depth == 0 {
                        return Err(io::Error::new(
                            ErrorKind::InvalidInput,
                            format!(
                                "LineDecoration 元素槽 offset 0x{:x}: 嵌套深度超过 \
                                 读取器限额 {MAX_SYMBOL_NESTING_DEPTH}（防栈溢出，非原生限制）",
                                origin.position()
                            ),
                        ));
                    }
                    let body = read_simple_line_decoration_element_body_10_1(
                        &mut forward,
                        max_bytes,
                        &mut staged,
                        key,
                        depth - 1,
                    )?;
                    let item = LineDecorationElement10_1::Element(Box::new(body));
                    // 体内容嵌套停止（其符号槽）→ 本层停止传播，循环中止。
                    if let LineDecorationElement10_1::Element(b) = &item {
                        if b.stop.is_some() {
                            stop = Some(SymbolStop10_1::NestedIncomplete {
                                class_id: SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID,
                                key,
                            });
                        }
                    }
                    item
                } else {
                    // 其余 ILineDecorationElement 实现类体未闭合：回退该条目
                    // 并停止。
                    forward = origin;
                    staged = saved;
                    stop = Some(SymbolStop10_1::Slot {
                        kind: DisplaySlotKind10_1::LineDecorationElement,
                        class_id,
                        key,
                    });
                    LineDecorationElement10_1::Unsupported { class_id, key }
                }
            }
        };
        elements.push(item);
        trial = forward;
        if stop.is_some() {
            break;
        }
    }

    *table = staged;
    *reader = trial;
    Ok(LineDecorationBody10_1 {
        key,
        version,
        element_count,
        elements,
        stop,
    })
}

/// SimpleLineDecorationElement（DisplayCore.dll `0x6eb84120`）体的已确认
/// 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleLineDecorationElementBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b290`=1）。
    pub version: u16,
    /// byte → this−0x18，Rotate 原始值。
    pub byte_18: u8,
    /// byte → this−0x17，FlipFirst 原始值。
    pub byte_17: u8,
    /// byte → this−0x16，FlipAll 原始值。
    pub byte_16: u8,
    /// short → this−0x14，PositionAsRatio 原始值。
    pub short_14: i16,
    /// ISymbol 槽（Save 槽 this−0x24；MarkerSymbol）经
    /// [`read_symbol_slot_full_10_1`] 分派。
    pub symbol: Option<SymbolSlotFull10_1>,
    /// 流内 int32 循环长度（PositionCount）。
    pub position_count: u32,
    /// count × double（→ this−0x40 集合，Position）。`None` = 符号槽停止。
    pub positions: Option<Vec<f64>>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

impl SimpleLineDecorationElementBody10_1 {
    /// 是否旋转标记以跟随线方向。
    pub fn rotate(&self) -> bool {
        self.byte_18 != 0
    }

    /// 是否翻转首个标记。
    pub fn flip_first(&self) -> bool {
        self.byte_17 != 0
    }

    /// 是否翻转所有标记。
    pub fn flip_all(&self) -> bool {
        self.byte_16 != 0
    }

    /// 是否将位置解释为沿线比例。
    pub fn position_as_ratio(&self) -> bool {
        self.short_14 != 0
    }
}

/// 从对象头之后的位置读取 10.1 SimpleLineDecorationElement 对象体，失败时
/// 不推进游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`SIMPLE_LINE_DECORATION_ELEMENT_CLASS_ID`] 并取得键 `key`。符号槽未知
/// 类或嵌套体停止时其后的位置数组不可达（成功停止，游标停在停止点）。
/// `depth` 为嵌套限额。
pub fn read_simple_line_decoration_element_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<SimpleLineDecorationElementBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "SimpleLineDecorationElement",
        entry_offset,
    )?;
    let byte_18 = trial.read_bytes(1)?[0];
    let byte_17 = trial.read_bytes(1)?[0];
    let byte_16 = trial.read_bytes(1)?[0];
    let short_14 = trial.read_u16_le()? as i16;
    // ISymbol 槽（经符号类分派；⑤ 请求 IID `{F3435802}`）。
    let (symbol, stop) = read_symbol_slot_with_kind_10_1(
        &mut trial,
        max_bytes,
        &mut staged,
        depth,
        DisplaySlotKind10_1::Symbol,
    )?;
    let mut position_count = 0;
    let mut positions = None;
    if stop.is_none() {
        position_count = read_loop_count_10_1(
            &mut trial,
            max_bytes,
            "SimpleLineDecorationElement",
            entry_offset,
        )?;
        let mut values = Vec::with_capacity((position_count as usize).min(4096));
        for _ in 0..position_count {
            values.push(trial.read_f64_le()?);
        }
        positions = Some(values);
    }

    *table = staged;
    *reader = trial;
    Ok(SimpleLineDecorationElementBody10_1 {
        key,
        version,
        byte_18,
        byte_17,
        byte_16,
        short_14,
        symbol: Some(symbol),
        position_count,
        positions,
        stop,
    })
}

/// MultiLayerLineSymbol（DisplayCore.dll `0x6eb68280`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MultiLayerLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝（Save 版本表 `DAT_6ec0b2c4`，10.1 写 2）。
    pub version: u16,
    /// int32 → this−0xc4，语义未定。
    pub int32_c4: i32,
    /// int32 → this−0xbc，语义未定。
    pub int32_bc: i32,
    /// 流内 int32 循环长度（层数候选）。
    pub layer_count: u32,
    /// count × LoadObject（ILineSymbol `{E6BDAA7D-…}`，Save 槽 this−0x74
    /// 数组）经 [`read_display_symbol_slot_10_1`] 分派。未知层符号类 = 停止
    /// （游标停在该层条目起点，已解析层保留，其后数组/串段不可达）。
    pub layers: Vec<SymbolSlotFull10_1>,
    /// count × int32（Save 槽 this−0x60 数组，语义未定）。`None` = 前置停止。
    pub ints_60: Option<Vec<i32>>,
    /// count × int32（Save 槽 this−0x4c 数组，语义未定）。`None` = 前置停止。
    pub ints_4c: Option<Vec<i32>>,
    /// 版本 ≥2 的 count × String（Save 槽 this−0x38 向量，语义未定）；
    /// 版本 <2 无流内串段（原生内存补空串）。`None` = 前置停止。
    pub strings: Option<Vec<String>>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MultiLayerLineSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`MULTI_LAYER_LINE_SYMBOL_CLASS_ID`]
/// 并取得键 `key`。层槽新对象未知类或嵌套体停止时为成功停止（游标停在该
/// 层条目起点）。版本 <2 的串段在流内不存在（原生补空串），读取器如实不
/// 消费字节。`depth` 为嵌套限额。
pub fn read_multi_layer_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MultiLayerLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_MULTI_LAYER_LINE_VERSION,
        "MultiLayerLineSymbol",
        entry_offset,
    )?;
    let int32_c4 = read_i32_le(&mut trial)?;
    let int32_bc = read_i32_le(&mut trial)?;
    let layer_count =
        read_loop_count_10_1(&mut trial, max_bytes, "MultiLayerLineSymbol", entry_offset)?;
    let mut layers = Vec::with_capacity((layer_count as usize).min(4096));
    let mut stop = None;
    for _ in 0..layer_count {
        let (slot, inner) = read_symbol_slot_with_kind_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth.saturating_sub(1),
            DisplaySlotKind10_1::LineSymbol,
        )?;
        layers.push(slot);
        if let Some(inner) = inner {
            stop = Some(inner);
            break;
        }
    }
    let mut ints_60 = None;
    let mut ints_4c = None;
    let mut strings = None;
    if stop.is_none() {
        ints_60 = Some(read_i32_vec(&mut trial, layer_count)?);
        ints_4c = Some(read_i32_vec(&mut trial, layer_count)?);
        if version as i16 >= 2 {
            strings = Some(read_string_vec(&mut trial, max_bytes, layer_count)?);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(MultiLayerLineSymbolBody10_1 {
        key,
        version,
        int32_c4,
        int32_bc,
        layer_count,
        layers,
        ints_60,
        ints_4c,
        strings,
        stop,
    })
}

/// MultiLayerMarkerSymbol（DisplayCore.dll `0x6eb6b010`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MultiLayerMarkerSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>3 原生拒绝（Save 版本表 `DAT_6ec0b2b8`，10.1 写 3）。
    pub version: u16,
    /// int32 → this−0x10c，语义未定。
    pub int32_10c: i32,
    /// int32 → this−0x104，语义未定。
    pub int32_104: i32,
    /// double → this−0x5c，语义未定（尺寸候选）。
    pub double_5c: f64,
    /// double → this−0x44，语义未定。
    pub double_44: f64,
    /// double → this−0x54，语义未定。
    pub double_54: f64,
    /// double → this−0x4c，语义未定。
    pub double_4c: f64,
    /// IColor 槽（Save 槽 this−0x3c）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// int32 → this−0x34，语义未定。`None` = 前置停止。
    pub int32_34: Option<i32>,
    /// double → this−0x24，语义未定。`None` = 前置停止。
    pub double_24: Option<f64>,
    /// ISymbol 槽（Save 槽 this−0x30，语义候选）。`None` = 前置停止。
    pub symbol: Option<SymbolSlotFull10_1>,
    /// 流内 int32 循环长度（层数候选）。
    pub layer_count: u32,
    /// count × LoadObject（IMarkerSymbol `{E6BDAA7C-…}`，Save 槽 this−0xbc
    /// 数组）。未知层符号类 = 停止。
    pub layers: Vec<SymbolSlotFull10_1>,
    /// count × int32（Save 槽 this−0xa8 数组）。`None` = 前置停止。
    pub ints_a8: Option<Vec<i32>>,
    /// count × int32（Save 槽 this−0x94 数组）。`None` = 前置停止。
    pub ints_94: Option<Vec<i32>>,
    /// 版本 ≥2 的 IMarkerBackground 槽（请求 IID `{AE41E4F0-DDE7-11D3-…}`，
    /// Save 槽 this−0xc；实现类未闭合 = 停止）。`None` = 版本 <2 或前置停止。
    /// IMarkerBackground 槽（G 轮接线起经 callout_background 槽分派：
    /// LineCallout 新对象整体消费；未知实现类映射为停止原因，
    /// 字段保持 `None`——`None` 同时表示前置停止未达，见 `stop`）。
    pub marker_background: Option<CalloutSlot10_1>,
    /// 版本 ≥3 的 count × String（Save 槽 this−0x7c 向量）；版本 <3 无流内
    /// 串段。`None` = 前置停止。
    pub strings: Option<Vec<String>>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MultiLayerMarkerSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`MULTI_LAYER_MARKER_SYMBOL_CLASS_ID`] 并取得键 `key`。版本 ≥2 的
/// IMarkerBackground 槽新对象（实现类未闭合）为成功停止（其后的串段不可
/// 达）；层槽停止传播。`depth` 为嵌套限额。
pub fn read_multi_layer_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MultiLayerMarkerSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_MULTI_LAYER_MARKER_VERSION,
        "MultiLayerMarkerSymbol",
        entry_offset,
    )?;
    let int32_10c = read_i32_le(&mut trial)?;
    let int32_104 = read_i32_le(&mut trial)?;
    let double_5c = trial.read_f64_le()?;
    let double_44 = trial.read_f64_le()?;
    let double_54 = trial.read_f64_le()?;
    let double_4c = trial.read_f64_le()?;
    // ⑧ IColor 槽（请求 IID `{20CD40B0}`）。
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    let mut int32_34 = None;
    let mut double_24 = None;
    let mut symbol = None;
    let mut layer_count = 0;
    let mut layers = Vec::new();
    let mut ints_a8 = None;
    let mut ints_94 = None;
    let mut marker_background = None;
    let mut strings = None;
    if stop.is_none() {
        int32_34 = Some(read_i32_le(&mut trial)?);
        double_24 = Some(trial.read_f64_le()?);
        // ⑪ ISymbol 槽（请求 IID `{F3435802}`）。
        let (slot, inner) = read_symbol_slot_with_kind_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth.saturating_sub(1),
            DisplaySlotKind10_1::Symbol,
        )?;
        symbol = Some(slot);
        stop = inner;
    }
    if stop.is_none() {
        layer_count = read_loop_count_10_1(
            &mut trial,
            max_bytes,
            "MultiLayerMarkerSymbol",
            entry_offset,
        )?;
        for _ in 0..layer_count {
            let (slot, inner) = read_symbol_slot_with_kind_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth.saturating_sub(1),
                DisplaySlotKind10_1::MarkerSymbol,
            )?;
            layers.push(slot);
            if let Some(inner) = inner {
                stop = Some(inner);
                break;
            }
        }
    }
    if stop.is_none() {
        ints_a8 = Some(read_i32_vec(&mut trial, layer_count)?);
        ints_94 = Some(read_i32_vec(&mut trial, layer_count)?);
        if version as i16 >= 2 {
            // G 轮接线：IMarkerBackground 槽接 callout_background 槽分派
            //（LineCallout；未知实现类回退并映射为停止原因，
            // 停止语义不变）。
            let (slot, inner) = read_marker_background_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth.saturating_sub(1),
            )?;
            marker_background = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    if stop.is_none() && version as i16 >= 3 {
        strings = Some(read_string_vec(&mut trial, max_bytes, layer_count)?);
    }

    *table = staged;
    *reader = trial;
    Ok(MultiLayerMarkerSymbolBody10_1 {
        key,
        version,
        int32_10c,
        int32_104,
        double_5c,
        double_44,
        double_54,
        double_4c,
        color,
        int32_34,
        double_24,
        symbol,
        layer_count,
        layers,
        ints_a8,
        ints_94,
        marker_background,
        strings,
        stop,
    })
}

/// MultiLayerFillSymbol（DisplayCore.dll `0x6eb65af0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MultiLayerFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝（Save 版本表 `DAT_6ec0b2d0`，10.1 写 2）。
    pub version: u16,
    /// int32 → this−0xb4，语义未定。
    pub int32_b4: i32,
    /// int32 → this−0xac，语义未定。
    pub int32_ac: i32,
    /// IColor 槽（Save 槽 this−0xc）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// 流内 int32 循环长度（层数候选）。
    pub layer_count: u32,
    /// count × LoadObject（IFillSymbol `{E6BDAA7E-…}`，Save 槽 this−0x64
    /// 数组）经 [`read_display_symbol_slot_10_1`] 分派。未知层符号类 = 停止。
    pub layers: Vec<SymbolSlotFull10_1>,
    /// count × int32（Save 槽 this−0x50 数组）。`None` = 前置停止。
    pub ints_50: Option<Vec<i32>>,
    /// count × int32（Save 槽 this−0x3c 数组）。`None` = 前置停止。
    pub ints_3c: Option<Vec<i32>>,
    /// 版本 ≥2 的 count × String（Save 槽 this−0x28 向量）；版本 <2 无流内
    /// 串段。`None` = 前置停止。
    pub strings: Option<Vec<String>>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MultiLayerFillSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`MULTI_LAYER_FILL_SYMBOL_CLASS_ID`] 并取得键 `key`。层槽未知类或嵌套体
/// 停止时为成功停止。`depth` 为嵌套限额。
pub fn read_multi_layer_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MultiLayerFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_MULTI_LAYER_FILL_VERSION,
        "MultiLayerFillSymbol",
        entry_offset,
    )?;
    let int32_b4 = read_i32_le(&mut trial)?;
    let int32_ac = read_i32_le(&mut trial)?;
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    let mut layer_count = 0;
    let mut layers = Vec::new();
    let mut ints_50 = None;
    let mut ints_3c = None;
    let mut strings = None;
    if stop.is_none() {
        layer_count =
            read_loop_count_10_1(&mut trial, max_bytes, "MultiLayerFillSymbol", entry_offset)?;
        for _ in 0..layer_count {
            let (slot, inner) = read_symbol_slot_with_kind_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth.saturating_sub(1),
                DisplaySlotKind10_1::FillSymbol,
            )?;
            layers.push(slot);
            if let Some(inner) = inner {
                stop = Some(inner);
                break;
            }
        }
    }
    if stop.is_none() {
        ints_50 = Some(read_i32_vec(&mut trial, layer_count)?);
        ints_3c = Some(read_i32_vec(&mut trial, layer_count)?);
        if version as i16 >= 2 {
            strings = Some(read_string_vec(&mut trial, max_bytes, layer_count)?);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(MultiLayerFillSymbolBody10_1 {
        key,
        version,
        int32_b4,
        int32_ac,
        color,
        layer_count,
        layers,
        ints_50,
        ints_3c,
        strings,
        stop,
    })
}

/// CharacterMarkerSymbol（DisplayCore.dll `0x6eb9b150`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterMarkerSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>4 原生拒绝（Save 版本表 `DAT_6ec0b270`，10.1 写 4）。
    pub version: u16,
    /// IColor 槽（→ this−0xc4）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// int32 → this−0xc0，CharacterIndex。
    pub int32_c0: i32,
    /// double → this−0xb4，Angle。
    pub double_b4: f64,
    /// double → this−0xac，Size。
    pub double_ac: f64,
    /// double → this−0xa4，XOffset。
    pub double_a4: f64,
    /// double → this−0x9c，YOffset。
    pub double_9c: f64,
    /// double → this−0x94，XScale。
    pub double_94: f64,
    /// double → this−0x8c，YScale。
    pub double_8c: f64,
    /// 版本 <3 的 IFont 槽（请求 IID `{BEF6E002-A874-101A-8BBA-…}`，
    /// Save 槽 this−0xbc；接线轮起 StdFont 分派，其余实现类 = 停止）。
    /// `None` = 版本 ≥3 或前置停止。
    pub font_slot_v3: Option<StdFontSlot10_1>,
    /// int32 → this−0x124，ROP2。`None` = 前置停止。
    pub int32_124: Option<i32>,
    /// int32 → this−0x11c，MapLevel。`None` = 前置停止。
    pub int32_11c: Option<i32>,
    /// 版本 >1 的 RotateWithTransform 原始 short → this−0x10c（读取后原生内存槽被
    /// `Symbol::GetDefaultRotateWithTransformValue` 覆盖，流内值读取即弃；
    /// 反汇编同 ArrowMarker）。`None` = 版本 ≤1 或前置停止。
    pub short_10c: Option<i16>,
    /// 版本 ≥3 的 String（字体名称）。`None` = 版本 <3 或前置停止。
    pub font_name: Option<String>,
    /// 版本 ≥3 的 int32 ×4（依次为 Bold、Italic、Underline、Strikethrough）。`None` = 版本 <3 或前置停止。
    pub font_ints_a: Option<[i32; 4]>,
    /// 版本 ≥3 的 short ×2（依次为 Weight、Charset）。
    /// `None` = 版本 <3 或前置停止。
    pub font_shorts: Option<[i16; 2]>,
    /// 版本 ≥3 的 int32 ×2（Size 的 CY 高、低 32 位）。
    /// `None` = 版本 <3 或前置停止。
    pub font_ints_b: Option<[i32; 2]>,
    /// 版本 >3 的第二 IFont 槽（Save 恒写同槽 this−0xbc 重写；接线轮起
    /// StdFont 分派，其余实现类 = 停止）。`None` = 版本 ≤3 或前置停止。
    pub font_slot_v5: Option<StdFontSlot10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

impl CharacterMarkerSymbolBody10_1 {
    /// 按磁盘上的高、低字顺序还原 IFont::Size，单位为万分之一磅。
    pub fn font_size_cy(&self) -> Option<i64> {
        self.font_ints_b
            .map(|[high, low]| (i64::from(high) << 32) | i64::from(low as u32))
    }
}

/// 从对象头之后的位置读取 10.1 CharacterMarkerSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`CHARACTER_MARKER_SYMBOL_CLASS_ID`] 并取得键 `key`。颜色槽 / 字体槽
/// （版本 <3 或 >3）新对象未知类为成功停止。版本分支（<3 字体对象 /
/// ≥3 字体描述符 / >3 字体对象重写）按反编译如实覆盖。
pub fn read_character_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<CharacterMarkerSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_CHARACTER_MARKER_VERSION,
        "CharacterMarkerSymbol",
        entry_offset,
    )?;
    // ② IColor 槽（请求 IID `{20CD40B0}` → this−0xc4）。
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    // ③ helper 不读流；④-⑨ 标量段（颜色槽停止时不可达）。
    let mut int32_c0 = 0;
    let mut double_b4 = 0.0;
    let mut double_ac = 0.0;
    let mut double_a4 = 0.0;
    let mut double_9c = 0.0;
    let mut double_94 = 0.0;
    let mut double_8c = 0.0;
    let mut font_slot_v3 = None;
    let mut int32_124 = None;
    let mut int32_11c = None;
    let mut short_10c = None;
    let mut font_name = None;
    let mut font_ints_a = None;
    let mut font_shorts = None;
    let mut font_ints_b = None;
    let mut font_slot_v5 = None;
    if stop.is_none() {
        int32_c0 = read_i32_le(&mut trial)?;
        double_b4 = trial.read_f64_le()?;
        double_ac = trial.read_f64_le()?;
        double_a4 = trial.read_f64_le()?;
        double_9c = trial.read_f64_le()?;
        double_94 = trial.read_f64_le()?;
        double_8c = trial.read_f64_le()?;
        // ⑩ 版本 <3 的 IFontDisp 槽（接线轮起 StdFont 分派，其余类停止）。
        if (version as i16) < 3 {
            let (slot, inner) = read_std_font_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                DisplaySlotKind10_1::Font,
            )?;
            font_slot_v3 = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }
    if stop.is_none() {
        int32_124 = Some(read_i32_le(&mut trial)?);
        int32_11c = Some(read_i32_le(&mut trial)?);
        if version as i16 > 1 {
            // short 读取后原生覆盖内存槽（GetDefaultRotateWithTransformValue），
            // 流内位模式原样保留。
            short_10c = Some(trial.read_u16_le()? as i16);
        }
        if version as i16 >= 3 {
            font_name = Some(read_string_10_1(&mut trial, max_bytes)?);
            let mut a = [0_i32; 4];
            for slot in a.iter_mut() {
                *slot = read_i32_le(&mut trial)?;
            }
            font_ints_a = Some(a);
            let s1 = trial.read_u16_le()? as i16;
            let s2 = trial.read_u16_le()? as i16;
            font_shorts = Some([s1, s2]);
            let b1 = read_i32_le(&mut trial)?;
            let b2 = read_i32_le(&mut trial)?;
            font_ints_b = Some([b1, b2]);
        }
        if version as i16 > 3 {
            let (slot, inner) = read_std_font_slot_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                DisplaySlotKind10_1::Font,
            )?;
            font_slot_v5 = slot;
            if let Some(inner) = inner {
                stop = Some(inner);
            }
        }
    }

    *table = staged;
    *reader = trial;
    Ok(CharacterMarkerSymbolBody10_1 {
        key,
        version,
        color,
        int32_c0,
        double_b4,
        double_ac,
        double_a4,
        double_9c,
        double_94,
        double_8c,
        font_slot_v3,
        int32_124,
        int32_11c,
        short_10c,
        font_name,
        font_ints_a,
        font_shorts,
        font_ints_b,
        font_slot_v5,
        stop,
    })
}

/// PictureMarkerSymbol 图像块的版本 9 分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum PictureImagePayload10_1 {
    /// 类型 1：String（图像路径候选）+ double（→ this−0xac）。
    Path { path: String, double_value: f64 },
    /// 类型 2：int32 N（全局内存块尺寸候选）；N≠0 时其后 N 字节原始数据
    /// （反汇编核实 `0x6eb7a7b5`/`0x6eb7a7de`：尺寸即首 int32，条件读取）。
    Blob { size: u32, data: Option<Vec<u8>> },
    /// 类型 3：LoadObject（IPicture `{7BF80980-BF32-101A-8BBB-…}` → 槽
    /// this−0x88；接线轮起 StdPicture 分派，其余实现类 = 停止）。
    Picture(StdPictureSlot10_1),
}

/// PictureMarkerSymbol 图像块的版本 ≤8 历史路径分类结果。
#[derive(Debug, Clone, PartialEq)]
pub struct PictureImageLegacy10_1 {
    /// 版本 <4 的第一个 IPicture 槽（请求 IID `{7BF80980}`；接线轮起
    /// StdPicture 分派）。`None` = 版本 ≥4、槽位停止或未达。
    pub picture_a: Option<StdPictureSlot10_1>,
    /// 版本 ≤6 的第二个 IPicture 槽；版本 ≥7 且路径串为空时同样读此槽
    /// （StdPicture 分派）。`None` = 版本 ≥7 且路径非空、槽位停止或未达。
    pub picture_b: Option<StdPictureSlot10_1>,
    /// 版本 ≥7 的 String（图像路径候选）；版本 ≤6 为空。`None` = 前置停止。
    pub path: Option<String>,
    /// 版本 ≥7 且路径非空时的 double（→ +0xf8）。`None` = 前置停止或路径空。
    pub path_double: Option<f64>,
    /// 版本 ==6 的第二个 String。`None` = 版本 ≠6 或前置停止。
    pub string6: Option<String>,
    /// 版本 ==6 的 byte。`None` = 版本 ≠6 或前置停止。
    pub byte6: Option<u8>,
    /// 版本 ==6 的 double（→ +0xf8）。`None` = 版本 ≠6 或前置停止。
    pub double6: Option<f64>,
    /// 版本 >7（v8）的 int32 N（位图数据尺寸候选）。`None` = 版本 ≤7 或
    /// 前置停止。
    pub blob_size: Option<u32>,
    /// 版本 8 的 N 字节原始数据（N≠0 时存在）。`None` = N==0、版本 ≤7 或
    /// 前置停止。
    pub blob_data: Option<Vec<u8>>,
}

/// PictureMarkerSymbol 图像块（版本分支收拢）。
#[derive(Debug, Clone, PartialEq)]
pub enum PictureImage10_1 {
    /// 版本 9 的图像块：int32 图像类型（1/2/3，越界拒绝 `0x80004005`）→
    /// int32（→ this−0x10）→ 类型分支载荷。载荷为 `None` = 类型 3 的
    /// IPicture 槽停止（游标停在槽位起点）。
    V9 {
        image_type: u32,
        /// 原生图像编码标记，保留原始值。
        aux_int32: i32,
        payload: Option<PictureImagePayload10_1>,
    },
    /// 版本 ≤8 的历史路径（各版本门如实覆盖）。
    Legacy(Box<PictureImageLegacy10_1>),
}

/// PictureMarkerSymbol（DisplayCore.dll `0x6eb7a650`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct PictureMarkerSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>9 原生拒绝（Save 版本表 `DAT_6ec0b29c`，10.1 写 9）。
    pub version: u16,
    /// 图像块（版本 9 分支 / 版本 ≤8 历史路径）。体内槽位停止时保留已读
    /// 前缀（`stop` 携带原因）。
    pub image: PictureImage10_1,
    /// IColor 槽 ×3，依次为 Color、BackgroundColor、BitmapTransparencyColor。
    /// 第 i 项为 `None` = 在该槽停止或未达。
    pub colors: [Option<ColorSlot10_1>; 3],
    /// 六个 double，依次为 Angle、Size、XOffset、YOffset、XScale、YScale。
    /// `None` = 前置停止。
    pub doubles: Option<[f64; 6]>,
    /// 两个 int32，依次为 ROP2、MapLevel。`None` = 前置停止。
    pub int32s: Option<[i32; 2]>,
    /// SwapForeGroundBackGroundColor，保留原始 byte。`None` = 前置停止。
    pub byte_48: Option<u8>,
    /// RotateWithTransform 原始 short → this−0x134（读取后原生内存槽被
    /// `Symbol::GetDefaultRotateWithTransformValue` 覆盖，流内值读取即弃）。
    /// `None` = 前置停止。
    pub short_134: Option<i16>,
    /// 透明度、亮度、对比度的原始 short。旧版本无此段或前置停止时为 None。
    pub shorts: Option<[i16; 3]>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 PictureMarkerSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`PICTURE_MARKER_SYMBOL_CLASS_ID`] 并取得键 `key`。版本 ≤8 走历史路径
/// （`FUN_6eb7d750`，各版本门如实覆盖），版本 ==9 走图像类型分支；IPicture
/// 槽实现类未闭合 = 停止点，颜色槽未知类 = 停止。位图 blob 尺寸以
/// `max_bytes` 设上限（读取器自身限额，非原生限制）。
pub fn read_picture_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<PictureMarkerSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("PictureMarkerSymbol 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    let version = read_signed_symbol_version(
        &mut trial,
        MAX_PICTURE_MARKER_VERSION,
        "PictureMarkerSymbol",
        entry_offset,
    )?;
    let signed = version as i16;
    let mut stop = None;

    // 图像块：版本 ≤8 历史路径 / 版本 9 分支。槽位停止记入 `stop`，共享
    // 尾段与附属段跳过（游标停在停止点）。
    let mut image = if version == 9 {
        let image_type_raw = read_i32_le(&mut trial)?;
        if ![1, 2, 3].contains(&image_type_raw) {
            return Err(invalid(format!(
                "图像类型 {image_type_raw} 不在 1/2/3（原生拒绝 0x80004005）"
            )));
        }
        let aux_int32 = read_i32_le(&mut trial)?;
        let payload = match image_type_raw {
            1 => {
                let path = read_string_10_1(&mut trial, max_bytes)?;
                let double_value = trial.read_f64_le()?;
                Some(PictureImagePayload10_1::Path { path, double_value })
            }
            2 => {
                let size = read_blob_size(&mut trial, max_bytes, entry_offset)?;
                let data = read_blob_data(&mut trial, size)?;
                Some(PictureImagePayload10_1::Blob { size, data })
            }
            _ => {
                let (slot, inner) = read_std_picture_slot_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    DisplaySlotKind10_1::Picture,
                )?;
                if let Some(inner) = inner {
                    stop = Some(inner);
                }
                slot.map(PictureImagePayload10_1::Picture)
            }
        };
        PictureImage10_1::V9 {
            image_type: image_type_raw as u32,
            aux_int32,
            payload,
        }
    } else {
        let mut legacy = PictureImageLegacy10_1 {
            picture_a: None,
            picture_b: None,
            path: None,
            path_double: None,
            string6: None,
            byte6: None,
            double6: None,
            blob_size: None,
            blob_data: None,
        };
        if signed <= 6 {
            // 版本 <4：第一个 IPicture 槽。
            if signed < 4 {
                let (slot, inner) = read_std_picture_slot_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    DisplaySlotKind10_1::Picture,
                )?;
                if let Some(inner) = inner {
                    stop = Some(inner);
                }
                legacy.picture_a = slot;
            }
            if stop.is_none() {
                let (slot, inner) = read_std_picture_slot_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    DisplaySlotKind10_1::Picture,
                )?;
                if let Some(inner) = inner {
                    stop = Some(inner);
                }
                legacy.picture_b = slot;
            }
        } else {
            // 版本 ≥7：String +（串空 → IPicture 槽 / 串非空 → double）。
            let path = read_string_10_1(&mut trial, max_bytes)?;
            legacy.path = Some(path.clone());
            if path.is_empty() {
                let (slot, inner) = read_std_picture_slot_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    DisplaySlotKind10_1::Picture,
                )?;
                if let Some(inner) = inner {
                    stop = Some(inner);
                }
                legacy.picture_b = slot;
            } else {
                legacy.path_double = Some(trial.read_f64_le()?);
            }
        }
        PictureImage10_1::Legacy(Box::new(legacy))
    };

    // 共享尾段：IColor ×3 → double ×6 → int32 ×2 → byte → short → short ×3
    //（图像槽已停止时全部跳过）。
    let mut colors = [None, None, None];
    for color_slot in colors.iter_mut() {
        if stop.is_some() {
            break;
        }
        let slot = read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?;
        let inner = color_slot_stop(&slot);
        *color_slot = Some(slot);
        if let Some(inner) = inner {
            stop = Some(inner);
        }
    }
    let mut doubles = None;
    let mut int32s = None;
    let mut byte_48 = None;
    let mut short_134 = None;
    let mut shorts = None;
    if stop.is_none() {
        let mut values = [0_f64; 6];
        for slot in values.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        doubles = Some(values);
        let a = read_i32_le(&mut trial)?;
        let b = read_i32_le(&mut trial)?;
        int32s = Some([a, b]);
        // 尾段版本门：版本 9 恒有；历史路径 byte >1、short >2、short ×3 >4。
        if signed > 1 {
            byte_48 = Some(trial.read_bytes(1)?[0]);
        }
        if signed > 2 {
            short_134 = Some(trial.read_u16_le()? as i16);
        }
        if signed > 4 {
            let s1 = trial.read_u16_le()? as i16;
            let s2 = trial.read_u16_le()? as i16;
            let s3 = trial.read_u16_le()? as i16;
            shorts = Some([s1, s2, s3]);
        }
    }

    // 版本 ≤8 的图像附属段（历史路径，位于共享尾段之后）：版本 ==6 的
    // String + byte + double；版本 >7（v8）的 int32 N +（N≠0 时 N 字节）。
    if stop.is_none() && signed <= 8 {
        if let PictureImage10_1::Legacy(legacy) = &mut image {
            if version == 6 {
                legacy.string6 = Some(read_string_10_1(&mut trial, max_bytes)?);
                legacy.byte6 = Some(trial.read_bytes(1)?[0]);
                legacy.double6 = Some(trial.read_f64_le()?);
            }
            if signed > 7 {
                let size = read_blob_size(&mut trial, max_bytes, entry_offset)?;
                legacy.blob_size = Some(size);
                legacy.blob_data = read_blob_data(&mut trial, size)?;
            }
        }
    }

    *table = staged;
    *reader = trial;
    Ok(PictureMarkerSymbolBody10_1 {
        key,
        version,
        image,
        colors,
        doubles,
        int32s,
        byte_48,
        short_134,
        shorts,
        stop,
    })
}

/// PictureLineSymbol（DisplayCore.dll `0x6eb77250`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct PictureLineSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b2a8`=1）。
    pub version: u16,
    /// byte → this−0x30，Rotate 原始位模式。
    pub byte_30: u8,
    /// double → this−0x2c，Offset 的流内值。
    pub double_2c: f64,
    /// double → this−0x24，Width 的流内值。
    pub double_24: f64,
    /// IPictureFillSymbol 槽（this−0x14）。未知对象体停在该槽，详见 stop。
    pub picture_fill_symbol: Option<PictureFillSymbolSlot10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 PictureLineSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`PICTURE_LINE_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。未知图像填充符号实现为成功停止。
pub fn read_picture_line_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<PictureLineSymbolBody10_1> {
    read_picture_line_symbol_body_with_depth(
        reader,
        max_bytes,
        table,
        key,
        MAX_SYMBOL_NESTING_DEPTH,
    )
}

fn read_picture_line_symbol_body_with_depth(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<PictureLineSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_signed_symbol_version(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "PictureLineSymbol",
        entry_offset,
    )?;
    let byte_30 = trial.read_bytes(1)?[0];
    let double_2c = trial.read_f64_le()?;
    let double_24 = trial.read_f64_le()?;
    let (slot, stop) =
        read_picture_fill_symbol_slot_10_1(&mut trial, max_bytes, &mut staged, depth)?;

    *table = staged;
    *reader = trial;
    Ok(PictureLineSymbolBody10_1 {
        key,
        version,
        byte_30,
        double_2c,
        double_24,
        picture_fill_symbol: slot,
        stop,
    })
}

/// ArrowMarkerSymbol（DisplayCore.dll `0x6eb3ec40`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowMarkerSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>2 原生拒绝（版本常量 `DAT_6ec0b314`=2）。
    pub version: u16,
    /// IColor 槽（→ this−0x40）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// double → this−0x8c，Length 的流内值。
    pub double_8c: f64,
    /// double → this−0x7c，Width 的流内值。
    pub double_7c: f64,
    /// double → this−0x6c，Angle 的流内值。
    pub double_6c: f64,
    /// int32 → this−0x64，Style。
    pub int32_64: i32,
    /// int32 → this−0xdc，ROP2。
    pub int32_dc: i32,
    /// int32 → this−0xd4，MapLevel。
    pub int32_d4: i32,
    /// double → this−0x2c，XOffset 的流内值。
    pub double_2c: f64,
    /// double → this−0x24，YOffset 的流内值。
    pub double_24: f64,
    /// 版本 ≥2 的 RotateWithTransform 原始 short → this−0xc4。读取后原生内存槽被
    /// `Symbol::GetDefaultRotateWithTransformValue` 覆盖（反汇编
    /// `0x6eb3edde..0x6eb3edf0`：读入 → 无条件覆写，流内值读取即弃），
    /// Save 恒写版本 2 并写该时刻的槽值。读取器保留流内位模式。`None` =
    /// 版本 ≤1 或前置停止。
    pub short_c4: Option<i16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 ArrowMarkerSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`ARROW_MARKER_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。颜色槽新对象未知类为成功停止（标量段不可达）。
pub fn read_arrow_marker_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<ArrowMarkerSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_signed_symbol_version(
        &mut trial,
        MAX_ARROW_MARKER_VERSION,
        "ArrowMarkerSymbol",
        entry_offset,
    )?;
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let stop = color_slot_stop(color.as_ref().unwrap());
    // ③ helper 不读流；④-⑪ 标量段。
    let mut double_8c = 0.0;
    let mut double_7c = 0.0;
    let mut double_6c = 0.0;
    let mut int32_64 = 0;
    let mut int32_dc = 0;
    let mut int32_d4 = 0;
    let mut double_2c = 0.0;
    let mut double_24 = 0.0;
    let mut short_c4 = None;
    if stop.is_none() {
        double_8c = trial.read_f64_le()?;
        double_7c = trial.read_f64_le()?;
        double_6c = trial.read_f64_le()?;
        int32_64 = read_i32_le(&mut trial)?;
        int32_dc = read_i32_le(&mut trial)?;
        int32_d4 = read_i32_le(&mut trial)?;
        double_2c = trial.read_f64_le()?;
        double_24 = trial.read_f64_le()?;
        if version as i16 >= 2 {
            short_c4 = Some(trial.read_u16_le()? as i16);
        }
    }

    *table = staged;
    *reader = trial;
    Ok(ArrowMarkerSymbolBody10_1 {
        key,
        version,
        color,
        double_8c,
        double_7c,
        double_6c,
        int32_64,
        int32_dc,
        int32_d4,
        double_2c,
        double_24,
        short_c4,
        stop,
    })
}

/// LineFillSymbol（DisplayCore.dll `0x6eb5e390`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct LineFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b2e8`=1）。
    pub version: u16,
    /// IColor 槽（→ this−0x6c，独立颜色槽，公开 Color 从线符号取值）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// ISymbol 槽 #1（Save 槽 this−0x74，LineSymbol）。
    /// `None` = 前置停止。
    pub symbol_a: Option<SymbolSlotFull10_1>,
    /// ISymbol 槽 #2（Save 槽 this−0x70，IFillSymbol::Outline）。
    /// `None` = 前置停止。
    pub symbol_b: Option<SymbolSlotFull10_1>,
    /// double ×3（→ this−0x64/−0x5c/−0x54，依次为 Angle、Offset、Separation）。
    /// `None` = 前置停止。
    pub doubles: Option<[f64; 3]>,
    /// int32 → this−0xc4，ROP2。`None` = 前置停止。
    pub int32_c4: Option<i32>,
    /// int32 → this−0xbc，MapLevel。`None` = 前置停止。
    pub int32_bc: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 LineFillSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`LINE_FILL_SYMBOL_CLASS_ID`] 并取得
/// 键 `key`。两个 ISymbol 槽与颜色槽未知类或嵌套体停止时为成功停止。
/// `depth` 为嵌套限额。
pub fn read_line_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<LineFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "LineFillSymbol",
        entry_offset,
    )?;
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    let mut symbol_a = None;
    let mut symbol_b = None;
    let mut doubles = None;
    let mut int32_c4 = None;
    let mut int32_bc = None;
    for field in 0..2usize {
        if stop.is_some() {
            break;
        }
        let (slot, inner) = read_symbol_slot_with_kind_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth.saturating_sub(1),
            DisplaySlotKind10_1::Symbol,
        )?;
        if field == 0 {
            symbol_a = Some(slot);
        } else {
            symbol_b = Some(slot);
        }
        stop = inner;
    }
    if stop.is_none() {
        let mut values = [0_f64; 3];
        for slot in values.iter_mut() {
            *slot = trial.read_f64_le()?;
        }
        doubles = Some(values);
        int32_c4 = Some(read_i32_le(&mut trial)?);
        int32_bc = Some(read_i32_le(&mut trial)?);
    }

    *table = staged;
    *reader = trial;
    Ok(LineFillSymbolBody10_1 {
        key,
        version,
        color,
        symbol_a,
        symbol_b,
        doubles,
        int32_c4,
        int32_bc,
        stop,
    })
}

/// MarkerFillSymbol（DisplayCore.dll `0x6eb606c0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b2e4`=1）。
    pub version: u16,
    /// int32 → this−0x7c，Style。
    pub int32_7c: i32,
    /// double → this−0x74，XOffset。
    pub double_74: f64,
    /// double → this−0x6c，YOffset。
    pub double_6c: f64,
    /// double → this−0x64，XSeparation。
    pub double_64: f64,
    /// double → this−0x5c，YSeparation。
    pub double_5c: f64,
    /// IColor 槽（Save 槽 this−0x34，独立颜色槽，公开 Color 从标记符号取值）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// ISymbol 槽 #1（Save 槽 this−0x80，MarkerSymbol）。`None` =
    /// 前置停止。
    pub symbol_a: Option<SymbolSlotFull10_1>,
    /// ISymbol 槽 #2（Save 槽 this−0x30，IFillSymbol::Outline）。`None` =
    /// 前置停止。
    pub symbol_b: Option<SymbolSlotFull10_1>,
    /// int32 ×2（→ this−0xd4/−0xcc，依次为 ROP2、MapLevel）。`None` = 前置停止。
    pub int32s: Option<[i32; 2]>,
    /// double → this−0xc，GridAngle。`None` = 前置停止。
    pub double_c: Option<f64>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 MarkerFillSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`MARKER_FILL_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。颜色/双符号槽未知类或嵌套体停止时为成功停止。`depth` 为
/// 嵌套限额。
pub fn read_marker_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<MarkerFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "MarkerFillSymbol",
        entry_offset,
    )?;
    let int32_7c = read_i32_le(&mut trial)?;
    let double_74 = trial.read_f64_le()?;
    let double_6c = trial.read_f64_le()?;
    let double_64 = trial.read_f64_le()?;
    let double_5c = trial.read_f64_le()?;
    let color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color.as_ref().unwrap());
    let mut symbol_a = None;
    let mut symbol_b = None;
    let mut int32s = None;
    let mut double_c = None;
    for field in 0..2usize {
        if stop.is_some() {
            break;
        }
        let (slot, inner) = read_symbol_slot_with_kind_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth.saturating_sub(1),
            DisplaySlotKind10_1::Symbol,
        )?;
        if field == 0 {
            symbol_a = Some(slot);
        } else {
            symbol_b = Some(slot);
        }
        stop = inner;
    }
    if stop.is_none() {
        let a = read_i32_le(&mut trial)?;
        let b = read_i32_le(&mut trial)?;
        int32s = Some([a, b]);
        double_c = Some(trial.read_f64_le()?);
    }

    *table = staged;
    *reader = trial;
    Ok(MarkerFillSymbolBody10_1 {
        key,
        version,
        int32_7c,
        double_74,
        double_6c,
        double_64,
        double_5c,
        color,
        symbol_a,
        symbol_b,
        int32s,
        double_c,
        stop,
    })
}

/// GradientFillSymbol（DisplayCore.dll `0x6eb52710`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct GradientFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b2f8`=1）。
    pub version: u16,
    /// IColorRamp 槽（请求 IID `{BEB87091-C0B4-11D0-8379-…}` → this−0x4c；
    /// G 轮接线起经 color_ramp 槽分派：四类闭合色带整体消费；未知实现类
    /// 映射为停止原因，字段保持 `None`——`None` 同时表示前置停止未达，
    /// 见 `stop`）。
    pub color_ramp: Option<ColorRampSlot10_1>,
    /// IColor 槽（Save 槽 this−0x34，IFillSymbol::Color）。`None` = 前置停止。
    pub color: Option<ColorSlot10_1>,
    /// ISymbol 槽（Save 槽 this−0x30，IFillSymbol::Outline）。`None` = 前置停止。
    pub symbol: Option<SymbolSlotFull10_1>,
    /// double → this−0x44，GradientPercentage。`None` = 前置停止。
    pub double_44: Option<f64>,
    /// int32 → this−0x2c，IntervalCount。`None` = 前置停止。
    pub int32_2c: Option<i32>,
    /// double → this−0x24，GradientAngle。`None` = 前置停止。
    pub double_24: Option<f64>,
    /// int32 ×3（→ this−0xc/−0x9c/−0x94，依次为 Style、ROP2、MapLevel）。`None` = 前置停止。
    pub int32s: Option<[i32; 3]>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

/// 从对象头之后的位置读取 10.1 GradientFillSymbol 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`GRADIENT_FILL_SYMBOL_CLASS_ID`] 并
/// 取得键 `key`。色带槽（实现类族未闭合）/颜色槽/符号槽新对象未知类为成功
/// 停止。`depth` 为嵌套限额。
pub fn read_gradient_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<GradientFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "GradientFillSymbol",
        entry_offset,
    )?;
    // ② IColorRamp 槽（G 轮接线：四类闭合色带整体消费；未知实现类回退并
    //    映射为停止原因，停止语义不变）。
    let (ramp_slot, ramp_stop) = read_color_ramp_slot_10_1(
        &mut trial,
        max_bytes,
        &mut staged,
        depth.saturating_sub(1),
        entry_offset,
    )?;
    let color_ramp = ramp_slot;
    let mut stop = ramp_stop;
    let mut color = None;
    let mut symbol = None;
    let mut double_44 = None;
    let mut int32_2c = None;
    let mut double_24 = None;
    let mut int32s = None;
    if stop.is_none() {
        color = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
        stop = color_slot_stop(color.as_ref().unwrap());
    }
    if stop.is_none() {
        let (slot, inner) = read_symbol_slot_with_kind_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth.saturating_sub(1),
            DisplaySlotKind10_1::Symbol,
        )?;
        symbol = Some(slot);
        stop = inner;
    }
    if stop.is_none() {
        double_44 = Some(trial.read_f64_le()?);
        int32_2c = Some(read_i32_le(&mut trial)?);
        double_24 = Some(trial.read_f64_le()?);
        let a = read_i32_le(&mut trial)?;
        let b = read_i32_le(&mut trial)?;
        let c = read_i32_le(&mut trial)?;
        int32s = Some([a, b, c]);
    }

    *table = staged;
    *reader = trial;
    Ok(GradientFillSymbolBody10_1 {
        key,
        version,
        color_ramp,
        color,
        symbol,
        double_44,
        int32_2c,
        double_24,
        int32s,
        stop,
    })
}

/// DotDensityFillSymbol（DisplayCore.dll `0x6eb4ded0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct DotDensityFillSymbolBody10_1 {
    /// 对象头键（样本中无实例）。
    pub key: u32,
    /// 体首 short 版本；>1 原生拒绝（版本常量 `DAT_6ec0b2fc`=1）。
    pub version: u16,
    /// byte → this−0x34（FixedPlacement）。
    pub byte_34: u8,
    /// double → this−0x3c，DotSpacing。
    pub double_3c: f64,
    /// byte → this−0x28（UseMasking 原始字节）。
    pub byte_28: u8,
    /// IColor 槽 #1（Save 槽 this−0x54，独立颜色槽，尚未确认属性名）。`None` = 前置停止。
    pub color_a: Option<ColorSlot10_1>,
    /// ISymbol 槽（Save 槽 this−0x58，IFillSymbol::Outline）。`None` = 前置停止。
    pub symbol: Option<SymbolSlotFull10_1>,
    /// int32 → this−0xd4，ROP2。
    pub int32_d4: i32,
    /// int32 → this−0xcc，MapLevel。
    pub int32_cc: i32,
    /// int32 → this−0x1c，RandomSeed。
    pub int32_1c: i32,
    /// double → this−0x44，DotSize。
    pub double_44: f64,
    /// byte → this−0x27（ExcludeMask 原始字节，仅在标量段已读取时有效）。
    pub byte_27: u8,
    /// 流内 int32 循环长度（点标记数候选；Save 侧标记倒序写出、顺序读取）。
    pub marker_count: u32,
    /// count × LoadObject（IMarkerSymbol `{E6BDAA7C-…}`，Save 槽 this−0x7c
    /// 数组）经 [`read_display_symbol_slot_10_1`] 分派。未知标记符号类 = 停止。
    pub markers: Vec<SymbolSlotFull10_1>,
    /// count × int32（Save 槽 this−0x68 数组，各标记的 DotCount）。`None` = 前置停止。
    pub values: Option<Vec<i32>>,
    /// IColor 槽 #2（Save 槽 this−0x50，BackgroundColor）。
    /// `None` = 前置停止。
    pub color_b: Option<ColorSlot10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<SymbolStop10_1>,
}

impl DotDensityFillSymbolBody10_1 {
    /// 是否启用点密度掩膜。
    pub fn use_masking(&self) -> bool {
        self.byte_28 != 0
    }

    /// 是否排除掩膜覆盖区域。前置对象槽停止时还未读取该标志。
    pub fn exclude_mask(&self) -> Option<bool> {
        self.symbol
            .as_ref()
            .filter(|symbol| slot_full_stop_10_1(symbol, DisplaySlotKind10_1::Symbol).is_none())
            .map(|_| self.byte_27 != 0)
    }
}

/// 从对象头之后的位置读取 10.1 DotDensityFillSymbol 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对
/// [`DOT_DENSITY_FILL_SYMBOL_CLASS_ID`] 并取得键 `key`。颜色/符号槽未知类
/// 或标记数组内嵌套停止时为成功停止（标记数组游标停在停止条目起点，其
/// 后值数组与尾颜色不可达）。`depth` 为嵌套限额。
pub fn read_dot_density_fill_symbol_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
    depth: usize,
) -> io::Result<DotDensityFillSymbolBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let mut staged = table.clone();

    let version = read_version_short(
        &mut trial,
        MAX_SYMBOL_VERSION,
        "DotDensityFillSymbol",
        entry_offset,
    )?;
    let byte_34 = trial.read_bytes(1)?[0];
    let double_3c = trial.read_f64_le()?;
    let byte_28 = trial.read_bytes(1)?[0];
    let color_a = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
    let mut stop = color_slot_stop(color_a.as_ref().unwrap());
    let mut symbol = None;
    let mut int32_d4 = 0;
    let mut int32_cc = 0;
    let mut int32_1c = 0;
    let mut double_44 = 0.0;
    let mut byte_27 = 0;
    let mut marker_count = 0;
    let mut markers = Vec::new();
    let mut values = None;
    let mut color_b = None;
    if stop.is_none() {
        let (slot, inner) = read_symbol_slot_with_kind_10_1(
            &mut trial,
            max_bytes,
            &mut staged,
            depth.saturating_sub(1),
            DisplaySlotKind10_1::Symbol,
        )?;
        symbol = Some(slot);
        stop = inner;
    }
    if stop.is_none() {
        // ⑦-⑪ 标量段（→ this−0xd4/−0xcc/−0x1c/−0x44/−0x27）。
        int32_d4 = read_i32_le(&mut trial)?;
        int32_cc = read_i32_le(&mut trial)?;
        int32_1c = read_i32_le(&mut trial)?;
        double_44 = trial.read_f64_le()?;
        byte_27 = trial.read_bytes(1)?[0];
        marker_count =
            read_loop_count_10_1(&mut trial, max_bytes, "DotDensityFillSymbol", entry_offset)?;
        for _ in 0..marker_count {
            let (slot, inner) = read_symbol_slot_with_kind_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                depth.saturating_sub(1),
                DisplaySlotKind10_1::MarkerSymbol,
            )?;
            markers.push(slot);
            if let Some(inner) = inner {
                stop = Some(inner);
                break;
            }
        }
    }
    if stop.is_none() {
        values = Some(read_i32_vec(&mut trial, marker_count)?);
        color_b = Some(read_color_slot_10_1(&mut trial, max_bytes, &mut staged)?);
        stop = color_slot_stop(color_b.as_ref().unwrap());
    }

    *table = staged;
    *reader = trial;
    Ok(DotDensityFillSymbolBody10_1 {
        key,
        version,
        byte_34,
        double_3c,
        byte_28,
        color_a,
        symbol,
        int32_d4,
        int32_cc,
        int32_1c,
        double_44,
        byte_27,
        marker_count,
        markers,
        values,
        color_b,
        stop,
    })
}
