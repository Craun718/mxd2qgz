# Map 非空边框与背景（10.1，第八轮）

2026-11-17。针对 Map 的两个接口槽补充类兼容证据并接线；原有对象体字段和版本分支已在 [装饰类研究](arcmap-10.1-display-slots-round3.md) 记录，本轮不重复宣称发现。均为静态证据，非 ArcMap 运行结果。

## 文件身份与新增证据

重新读取本地安装介质提取物的 PE 资源及 SHA-256：两文件均为 x86（machine `0x14c`），FileVersion / ProductVersion 均为 `10.1.0.3035`。

| 文件 | InternalName / OriginalFilename | SHA-256 |
| --- | --- | --- |
| Map.dll | Map / Map.DLL | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` |
| GraphicElements.dll | GraphicElements / GraphicElements.DLL | `f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9` |

资源声明和摘要标识分析文件，不证明签名真实性。Map 的 Load/Save 调用顺序及接口 IID 沿用 [照明后槽](mxd-terra-integrate-map.md) 和 [变换循环出口](mxd-terra-host-tail.md)。本轮读取 GraphicElements PE 中 IID 原始字节并反汇编引用指令，将 Map 请求的接口与旧取证中已由 RTTI、类标识和持久化虚表定位的具体类连接：

- `0x75072840` 的 16 字节为 IBorder `{E4240307-DD41-11D2-B868-00600802E603}`；SymbolBorder 查询分支 `0x7505dbf0` 在 `0x7505dc0b` 引用该常量并调用比较函数，匹配后跳到 `0x7505dcc0`，返回对象 `+0x18` 接口子对象并调用其引用计数槽。
- `0x75072850` 为 IBackground `{E4240309-DD41-11D2-B868-00600802E603}`；SymbolBackground 查询分支 `0x7505bce0` 在 `0x7505bcfb` 引用该常量，匹配后跳到 `0x7505bdb0`，同样返回 `+0x18` 子对象。两个分支地址分别位于 `0x75072b0c` 和 `0x75072968` 虚表首槽，其宿主类身份由旧 RTTI/COL 记录 `temp/terra-integrate-page/decor-rtti.txt` 关联。

以上是新增的接口兼容静态证据；具体类 CLSID、Load/Save 地址和字段序依赖旧取证，不表示 Map 保存时必定使用这两类。既有 Ghidra（经桥接工具） 结果与本轮 PE 字节和 objdump 已足以支持本次接入，因此没有新建 Ghidra 桥接工具 会话或切换共享目标。

## 实现及边界

`maps::map_decor` 使用既有 `element_decor` 读取 SymbolBorder 和 SymbolBackground，包装候选游标及引用表；仅当整个对象及嵌套符号均已读取完毕才提交。空槽、新对象以及类标识匹配的既有键回引可继续；未知类、未知嵌套体、错误版本或截断停在宿主槽起点，并撤销宿主及内嵌键的临时登记。原引用表只记录键，不记录实际目标类；回引仅能检查流中类标识，不能证明该键原对象的类或接口兼容性。嵌套符号支持范围仍受既有符号分派器约束，并未遍历全部 ILineSymbol / IFillSymbol 实现，也未另行证明每种分派符号的接口兼容性。

Map 的照明后边框槽及变换循环后的背景槽均接入此包装；已消费的对象体保存于后缀结构并用于诊断。保持原空槽字段以兼容已有调用者。Map 版本范围仍为既有读取器支持的 23..=51；未知 Map 几何或前置对象仍可能使两个槽不可达。其他 IBorder/IBackground 实现（包括其他模块的边框类）、样本覆盖和样式转换不在本轮闭合范围。

## 可勾范围和验证

可独立记录：10.1 静态接口查询证据确认 SymbolBorder / SymbolBackground 分别是 Map 两个接口槽可接受的典型类；Map 对这两类及已有支持的嵌套符号实现事务式读取和接续。当前 TODO 将边框与其他多个未闭合缺口合并，不能整体勾选；非空类族并未穷尽。本轮未修改 TODO。

构造测试覆盖两个典型类、每个截断位置、未知回引类、未知嵌套类和登记回退，另在参考样本的两个空槽上替换成构造对象，验证从根读取后到达后续 BasicOverposterProperties；这是合成输入，非实际非空样本。`cargo fmt --check`、`cargo test --offline`（783 通过、12 忽略）和 `git diff --check` 通过。ArcMap 10.1 运行环境暂缺，未在 ArcMap 或 QGIS 中人工打开和目视核对源工程、样式及转换结果。

## 10.1 安装介质类型库补查

扫描 72 份去重的安装介质 `.olb` 文件（10,392 条类型定义），查询 IBorder 和 IBackground 的 coclass 接口声明，同时沿类型库记录的接口继承关系查找后继接口。IFillSymbol 等嵌套槽沿用各自的解析范围。公开的 coclass 结果是 IBorder 的 SymbolBorder 和 IBackground 的 SymbolBackground。此结果覆盖类型库声明，未覆盖未公开的原生实现或第三方 COM 类。

`MapElements.dll` 10.1 的 CalibratedMapGridBorder 查询入口 `0x79544ec0` 匹配 IMapGridBorder `{AC81ECF9-9EE4-11D2-AADF-000000000000}` 和 ICalibratedMapGridBorder `{6CA416AF-E160-11D2-9F4E-00C04F6BC78E}`。SimpleMapGridBorder 同样用于 MapGrid 对象的 IMapGridBorder 槽。两类的持久化入口与布局宿主见[网格复核](mxd-page-layout-grid-20261118.md#measuredgridindexgrid-与-calibratedmapgridborder)。

### 原生实现与消费者复核

对同套 10.1 Desktop 介质的 1203 个 PE 搜索两个完整 IID，再对常量虚地址在 `.text`、`.rdata`、`.data` 中查引用。逐个反汇编 13 个 DLL 内的 90 处 `.text` 命中。除 GraphicElements.dll 的两个已知实现查询外，其余引用是查询已有对象、LoadObject、样式选择器或样式类别的参数。例如 CartoXProvider 的 `0x6ce520b2` 和 `0x6ce520e8` 使用导入 `0x6ced5024` 的 AfCore LoadObject，GraphicElementsUI 的 `0x1000ba33` 和 `0x1000ba56` 将 IID 传给 `afUILib::CStyleButton` 构造器。StyleGalleryClasses 的 `0x7ff62779` 和 `0x7ff62bd9` 将 IID 作为样式类别筛选参数。候选 DLL 的 `.rdata` 与 `.data` 未发现指向这两个 IID 的绝对地址表项。以上与公开类型库的清单一致，支持在所查介质范围内以 SymbolBorder 和 SymbolBackground 作为两个槽的实现类。运行时额外注册的类仍按现有未知对象规则保留。

### 背景的 PictureFillSymbol 槽

核对 DisplayCore.dll x86 10.1.0.3035，SHA-256 `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`。PictureFillSymbol 主虚表 `0x6ec0fc04` 的 QueryInterface `0x6eb6fc50` 在 `0x6eb6fc6b` 比较 `0x6ec0ce8c` 的 IFillSymbol IID，匹配后走 `0x6eb6ff5c`。背景 `0x7505c510` 以同一 IID 读取填充符号。已有 `PictureFillSymbol` 专用体读取器接入通用符号分派，Map 背景和其他使用该分派的填充符号槽可读取该类。

复核旧读取函数 `0x6eb74aa0`。颜色和轮廓之后的 `0x6eb74d86..0x6eb74e61` 读取七个成员 double 和两个临时 double。`0x6eb74ea0` 仅在 v2 起读取字节字段，`0x6eb74ec4` 仅在 v4 起读取三个 short，`0x6eb74f26` 在 v7 读取尾随位图长度和数据。Rust 保留位图原字节。未知 IPicture 在自身槽停止，避免将它重新解释为后面的颜色对象。构造用例从 `/Maps` 根读完带内嵌位图的背景以及后继标注属性，并核对各旧版本尾部哨兵。尚无对应非空背景的实档，未做 ArcMap/QGIS 人工验收。
