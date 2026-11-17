# 2026-11-18 MXD 解析并行复核

本轮开始时，TODO 的「MXD 解析」节有 62 个未勾选条目。六个 GPT-6 Luna 子代理按原行号分工，外加核查相邻的解析接线及原生证据前提。组内研究记录引用的行号是分派时的快照，以条目内容对应后续的 TODO 版本。

| 分组 | 原「MXD 解析」范围 | 研究记录 |
| --- | --- | --- |
| 版本 | 95、97–102、112–113、138–139 | [Version](mxd-luna-20261118-version.md) |
| 对象流 | 124–125、128–129、131、141、302 | [ObjectStream 与 Variant](mxd-luna-20261118-stream.md) |
| 地图 | 137、198、238、244–245、267、278、285 | [Map、装饰物与书签](mxd-luna-20261118-maps.md) |
| 图层 | 150、177–179、248–250、275、280、297、300–301、315–316 | [FeatureLayer、FDO 与标注](mxd-luna-20261118-layers.md) |
| 显示 | 234、240–241、257、288–294、296 | [颜色、符号、数字格式与 OLE](mxd-luna-20261118-display.md) |
| 数据与空间 | 242、259、266、268、274、298–299、303–305 | [数据源名称、空间参考与几何](mxd-luna-20261118-geo-data.md) |

## REA 补导

本机 npm 内容缓存可恢复 REA 6.9.0 CLI。使用独立工作区缓存、JDK 25 和 Ghidra 12.2；默认沙箱中导入后创建 Unix socket 返回 `Operation not permitted`，随后以获授权的本地进程完成定向补导。只读目标为安装介质 ArcMap.exe，PE x86，文件版本 10.1.0.3035、SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`，与[原生基线](arcmap-10.1-native-baseline.md)一致。

补导的 `0x47a740` 是 CMxDoc 的归档分发函数。原始指令检查归档参数 `+0x18` 的位 0，两个分支均调用基类归档方法；写、读路径分别在 `0x47a7c9`、`0x47a974` 调用 `0x466fb0`、`0x4764d0`，两处均取 CMxDoc `+0x7c` 作为参数。REA 证据 ID：`ev_62920ad6d48e79cafd1762c3bd760c05735fb7bfc8d71145d3c537148daa6808`，伪代码与证据在忽略目录 `temp/luna-20261118/rea-local/arcmap-47a740.json`。这为读写入口和参数来源提供静态复核；归档上游、旧档状态、版本映射和异常行为的运行结论仍需证据。

本机主要导出目录未包含 LabelPlacement.dll，本轮另补导 10.1.0.3035 的 LabelEngineLayerProperties Load `0x785468c0`（SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`）。可见三处 String 读取和后续 `LoadObject` 调用；版本门之后的第五处对象调用使用 `0x7857cb7c` 的 IID，与 [DEP 尾段记录](arcmap-10.1-dep-tail.md)相符。REA 证据 ID：`ev_39ef6ea29c366767b8d9cf30f6681206e06c8d44d63ed7e3ae31b5caa6d6edb6`，证据在 `temp/luna-20261118/rea-label/lelp.json`。该 Load 入口支持槽位边界复核；不能单凭该函数命名所有标量，或推断非空 IGraphicsContainer 的实际实现类。

顺带以 [MSI 四表映射](arcmap-10.1-native-baseline.md#msi-组件和默认安装目录2026-11-18) 复核了 15 个研究模块的 CAB 成员、组件和默认目标目录，未安装或运行 ArcMap。

## 验证基线

改动前 `cargo fmt --check` 通过；`cargo test` 通过 796 项，另外 17 项默认跳过，显式运行均通过。运行本机解析器对仓库参考工程和 63 个 10.1 安装介质工程执行 `parse`，64 次均返回成功并报告一个数据框；返回成功可能包含有界未知内容或提前停止。参考工程在 `/Maps:0x253be` 留下 431 字节的 Map 尾部，另有两个未解析栅格体。此处记录的是解析器回归基线，不表示 ArcMap 或 QGIS 人工验收。

## 主审核与整合结果

- **书签**：按 Map.dll 10.1.0.3035 的 `.rdata` 原始字节核对 `0x79481590`（AOI）与 `0x79482c18`（Feature）。同步修正陈旧的类标识测试期望，并用正常体及截断体验证宿主停止行为。18 份 `/Version` 文本为 9.3 的介质工程共 133 个 AOIBookmark，均完整读过书签循环；完整 FeatureBookmark 的其它 IName 类仍开放。
- **栅格**：复核 10.1 RasterLayer 的 IPersistStream 虚表及 `0x7c73bd40` 的有符号版本门和首个 String。仅在 ExtraSafe 条目界内试读，不把首串写成模型名称；测试证明长度声明不能让试读借用活动索引。参考工程两个 v17 对象分别读到 `/Maps:0x4ff8` 与 `0x145fb`，仍保留未读体，活动索引保持 -1，Map 尾部仍从 `0x253be` 保留 431 字节。
- **旧 FDO**：在 SelectionSymbol 与 IColor 槽接入已有对象体分派；检查了原分类字段兼容、未知类槽头与引用表回退、嵌套停止及截断。旧 FDO 真实实例与 v4 探测仍无充分证据。
- **数据源**：在模型接线复用 FeatureDatasetName 的 WorkspaceName 路径读取。合成 FeatureLayer 验证 FCN 非零首选择子从对象体进入模型，安装介质 Africa 佐证局部名称链范围；不推断未知类型或回引的路径。

集成后 `cargo fmt --check`、`cargo test -- --include-ignored`（827 项全部通过）、`git diff --check` 通过。重跑 `parse`：仓库参考工程 1 份、10.1 介质 66 份、另一套介质 68 份，全部成功返回；部分工程仍在未支持的对象处停止。没有在 ArcMap 或 QGIS 中打开工程做人工核对。

「MXD 解析」还剩 61 项，完整条目见 [TODO](../TODO.md#mxd-解析)。后续主要缺口是 Version 尾部及版本状态、跨流和 Variant 对象体、Map 末尾与后续数据框、未接入的图层及渲染对象、未定字段语义，以及旧版本对象体或受控运行证据。本轮仅处理静态或现有样本可验证的范围；需要 ArcMap 10.1 实机的条目保留待办。

## 后续并行与主审核

工作树内六名 Luna 分别复核了 [Map 尾部](mxd-luna-followup-maps-20261118.md)、[栅格](mxd-luna-followup-display-20261118.md)、[对象流](mxd-luna-followup-stream-20261118.md)、[FeatureIDSet](mxd-luna-followup-feature-id-set-20261118.md)、[旧 FieldInfo](mxd-luna-followup-fieldinfo-old-gate-20261118.md) 及 [MapSurround/Legend](mxd-luna-followup-surround-legend-20261118.md)。复核后撤销了依据 Ghidra 伪代码中恒空接口推进 Map 尾部的改动：原始指令传的是逐图层查询的输出，仍需闭合 FDO 聚合查询。

主审核用 10.1 esriCarto 的 ILayer 类型库绑定栅格名称和可见性，接入中间模型。两条栅格都读出首个 IName 头，普通 RasterDatasetName 连同内嵌 WorkspaceName 读到 `0x14862`；FileGDB 栅格的 IName 体仍停在 `0x501e`。补导 10.1 RasterDB/FgdbRasterDB 的加载入口（[普通名称体](mxd-luna-followup-rasterdatasetname-20261118.md)，FileGDB `0x6f934910` REA ID `ev_ef97e89c48dcddba404ff51f19cd0b8174f8d0d3638e6651b71fb0bbd1446464`，先委派 IPersistStream 读取，再进入 RasterIO 标签段）；另补导 [10.8 DocumentInfo 保存](mxd-luna-followup-version-20261118.md)，分清 Metadata 与 /Version 写入。FieldInfo v0/v1 的新增合成测试不等于取得旧实档。

最终整合 `cargo fmt --check`、`git diff --check`、`cargo test -- --include-ignored` 通过（840 项）。参考工程和两批介质 134 份均完成 `parse`；18 份非空循环仍完整消费 133 个 AOIBookmark。参考工程仍保留 `0x253be` 起的 Map 尾部。新增已确证的子项已单独落账，原「MXD 解析」仍有 61 个开放缺口；ArcMap 与 QGIS 人工验收未做。


## 第二次续查与主审

六名 GPT-6 Luna 并行推进剩余可静态或样本验证的工作，以下为主审后的结论：

- **Map**：Map.dll `0x79403d50` 逐层查询 IBasemapLayer 和 ILayerMasking。[原生复核](mxd-luna-deep-layer-masking-20261118.md) 由工厂和构造指令证明 FDOGraphicsLayer 转发到 FeatureLayer 的实际 QI 入口。主审保留 LayerLib `0x786f1ff0` 消费的 Map 自身遮罩尾 short，再仅对所有图层均证实为无逐层遮罩数据的列表继续读。参考样本回引实际指向 FDO 内已读完的关联 FeatureLayer。版本 29 的[接口 short](mxd-luna-deep-map-tail-20261118.md)、30 的 `+0x398` String、31 的 `+0x3a0` short、32 的 `+0x15d` 字节按 Map 原生门读取，停点由 `/Maps:0x253be` 推进至 `0x253c9`；后两个门由 Map.dll `0x79400908..14` 和 `0x79400939..45` 指令复核。SDK LinearRef 也到达新段。GroupLayer 的逐层集合、未验证类、未验证回引、后续 Map 集合与第二个数据框仍保留。
- **PropertySet**：[原生调用链](mxd-luna-deep-property-20261118.md) 确认 `Variant::Read` 根据传入流的接口选择 ObjectStream 会话。主审接入 WorkspaceName 经 CLSID 分派的 PropertySet；递归只接受已证的 PropertySet 新体，回引共享键表，失败同时回滚读指针和键表。嵌套对象实样与独立的 VariantStream 类族仍待验证。
- **TimeData**：[SDK Java 旧档](mxd-luna-deep-timedata-20261118.md) 从 `/Maps` 根顺序读出 13 个宿主版本 34 的 FeatureLayer，四段版本均为 1，分别完整消费至尾端；其中两个嵌套时间对象槽为空。这一条原待办已闭合。
- **名称和版本**：[FeatureDatasetName](mxd-luna-deep-names-20261118.md) 第四字符串和标志字节已静态绑定到访问器，业务名称仍保持未知。REA 补导 AfUI 的 [Document Info 读写](mxd-luna-deep-version-info-20261118.md)；主审分别核对了 `Document Info`、`Metadata`、`Templates` 与保存版本索引的归属，真实打开后索引状态仍需实机。

- **FileGDB 栅格名称**：[RasterIO 取证](mxd-luna-deep-fgdb-raster-20261118.md) 以 10.1 FgdbRasterDB 工厂指令绑定委派的普通 RasterDatasetName，再用 RasterCoreLib 的 REA 和原始指令复核标签头、跳读和结束。主审接入有界 RasterLayer；参考工程顺序消费委派 WorkspaceName 至 `0x513b`，再读到结束记录后的 `0x51d9`。非空 RasterIO 对象、额外记录和 RasterLayer 自身后续字段仍保留。

主审验证：`cargo fmt --check`、`git diff --check` 通过；`cargo test --no-fail-fast -- --include-ignored` 通过 861 项，未跳过测试。CLI `parse` 覆盖参考工程、66 份 10.1 介质工程和 68 份 10.8 介质工程，135 次均正常返回；18 个非空书签循环仍完整消费共 133 项。CLI 成功可能包含保留的未知内容。参考工程首个 Map 从 `0x253c9` 保留 420 字节。按 `TODO.md` 的「MXD 解析」章节统计，开放项 61 → 60；此轮未做 ArcMap 或 QGIS 人工验收。
