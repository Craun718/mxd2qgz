# Terra 第九轮：MXD 解析复核（2026-11-17）

六个 GPT‑6.1 Terra 子代理并发接续 [第八轮的 65 个开放项](mxd-terra-round8-review.md)。研究以 ArcMap 10.1.0.3035 的已导出反编译代码、必要时新增静态取证和目标版本互操作程序集为依据；SDK 工程和仓库参考 MXD 用于核对可实际到达的对象边界。

| 工作 | 主审复核 |
| --- | --- |
| [独立表](mxd-terra-round9-table.md) | 复用 FieldInfo 读取器。SDK `LinearRef.mxd` 从 Map 宿主完整读取两张 v9 独立表；首表 24 个 FieldInfo 都是 v2。Map 继续到 `0x8407`；任意嵌套类及其他版本仍待证实。复跑了 v2 截断回滚、未知内嵌对象停止及 SDK 原始边界测试。 |
| [AfCore 版本支持接口](mxd-terra-round9-iid.md) | 核对 10.1 互操作程序集的 COM GuidAttribute 和方法序，确认 IID 为 IDocumentVersionSupportGEN；与 AfCore 保存阶段的支持查询/替换对象槽吻合。其余未知接口和对象委派链仍开放。 |
| [旧档加载](mxd-terra-round9-save.md) | 核对 CMxDoc 正常构造经 IDocumentInfo2 持有 DocumentInfo，再经另一接口进入 Metadata 读取。版本分类值选择元数据布局；没有足够证据判断旧档打开后保存索引是否改写。 |
| [逐图层遮罩](mxd-terra-round9-mask.md) | 证实 Map 先检查旧版 IBasemapLayer 跳过分支，再查询 ILayerMasking，有接口才读取并经 ICompositeLayer 递归。主审核实三个 IID 的类型库名称；参考样本各具体层的接口状态还未确定，仍保守停止。 |
| [文字符号](mxd-terra-round9-text.md) | 按 IMask 的读写槽确认 MaskStyle、MaskSize 和 MaskSymbol，核对 Color/ShadowColor 路径。只对完整体暴露遮罩属性，早停不返回默认值；其余未知字段保留。 |
| [要素书签](mxd-terra-round9-bookmark.md) | 单次对象头分派复用独立表的 TableName 读取器；部分体保持书签事务回退。主审核实 IName 第三槽为 Open，修正直接查询 IName 对象的旧说法。非空真实书签及具体类分布仍待取证。 |
| [旧版对象分支](mxd-terra-round9-old.md) | 8.1 工程从 Map 宿主顺序读到两个空旧图层列表。SDK 的两个 Envelope v0 局部候选更符合错位后的 v3，缺 ArcToolbox 宿主，不能当作实证。FieldInfo v0/v1、GroupLayer <11 与 Envelope v0 仅新增合成边界或截断测试，仍缺真实对象体。 |

本轮未满足任何原有开放项的全部闭合条件，[TODO 的 MXD 解析节](../TODO.md#mxd-解析)仍有 65 项。ArcMap 10.1 暂无可用环境，未执行 ArcMap 或 QGIS 打开、保存重开或目视验收。

主审跑过 `cargo fmt --check`、`git diff --check` 和暂存区差异检查；`cargo test --offline` 为 794 通过、17 默认忽略，另有 6 项依赖本地安装介质的测试单独跑过。按 README 的 `parse` 用法复跑参考 MXD：六条图层记录，`/Maps:0x253be` 起仍保留 431 字节。
