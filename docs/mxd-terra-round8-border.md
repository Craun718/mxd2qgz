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
