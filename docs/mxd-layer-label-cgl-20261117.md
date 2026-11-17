# 图层与标注续研及 CGL 子层接线（2026-11-17）

本记录对应分配时 TODO 原行号 124、151–152、220–224、248、252、269、272–273。原生静态证据为 ArcMap Desktop 10.1.0.3035，详见 `mxd-label-luna.md`、`mxd-layer-luna.md`、`mxd-parse-agent-map-layer-spatial-20261117.md`、`mxd-parse-agent-sample-evidence-data-20261117.md` 和 `mxd-parse-agent-sample-evidence-edge-branches-20261117.md`。并未运行 ArcMap。

## CGL

FeatureLayer.dll 的 `CompositeGraphicsLayer::Load` 0x6f6b2100 读子图层条目为 String、ILayer 对象、三个 short、两个 double、两个 int32、IElementCollection 对象及版本大于 1 的 double；见 `arcmap-10.1-map-object.md` CGL 子图层计数裁决及其反编译文本 `/tmp/r3/annolayer_work/cgl_load_6f6b2100.c`。接入仅分派已实现的 FeatureLayer；体为 TailCompleted 才继续条目尾段，体内停止保留键及已解析体，不消费未知部分。其他类（包括嵌套 CGL）停在对象头前，故本次没有新容器递归路径；循环引用由对象登记表识别。截断返回错误并回滚外层事务。

仓库真实参考样本在完整 Map 对象流登记上下文中按顺序消费 10 条（其中第 6 条回引键 1792），循环后停于 `/Maps:0x24f22`。这是读取器结果结合既有对象级走查的辅助验证，不是 ArcMap 执行验证。单独从 CGL 体起点使用空登记表并不重现完整上下文，只用于验证内部停止透传，不能据其位置判断真实样本边界。CGL 循环后尾段仍未读取；Map 后缀调用方也仍把基本图形层视为未完成，extent 未到达。主代理需决定 CGL 尾段读取、Map 后缀继续及模型图层树接线；不能仅将 TailNotExpanded 改为完成。

测试包括未知类回退、嵌套 CGL 不递归、回引、截断回滚、单独 CGL 的内部停止、完整参考样本的连续消费。它们分别是合成行为测试或静态文件读取；不是软件人工验收。

## 安装介质模板

主代理从 Setup1.cab 提取的 66 个原始模板及 `temp/arcmap101-media-streams/index.json` 可用；均有 /Maps。使用当前 CLI 对全部逐份解析：48 份显示到达 CGL 循环后未展开尾段，18 份显示在 CGL 元素组的新对象处停止。前者不是覆盖所有对象的证明，后者后续字段不可判断。例如 `ANSI_C_Landscape.mxd` SHA-256 `5e76a99d479caf5702b9f05bcbd899ca20a86d3662b74e51335b4d0fec28bbad`，其 /Version 文本 9.3、short 16，CGL 子层计数 0；`Africa.mxd` SHA-256 `ee6be21c409285644e81177a19878f5149cd2f3380b95f188fdfa5788d466674` 在 CGL 元素组未知对象处停止。本轮未证明这些模板包含指定 GroupLayer 或历史 FieldInfo/FDO 分支，不能以文件名、日期、GUID 扫描或未到达字段推断其不存在。应以版本目录选择候选并从已确认对象流边界逐条走查。

## 原 TODO 行状态与缺口

- 124：部分。10.1 GroupLayer Load `0x6f711270`、Save `0x6f7119b0` 的版本分支和版本表已静态闭合；尚无该类不同版本的对象体字节。需要包含 GroupLayer 的受控文件或运行保存记录。
- 151：阻塞。已有 AnnotationVBScriptEngine，其他表达式引擎的持久化实现类与体未定位；需要 GetClassID/虚表/Load 证据或可定位的真实非空对象。
- 152：阻塞。请求 IID 为 IGraphicsContainer，参考样本该槽为 16 字节零；需要非空对象和对应实现的 Load/Save。
- 220：部分。FDO 旧加载器 `0x6b197dc0` 字段序有静态证据；接口槽业务身份需要接口方法/类型库或运行观察。
- 221、222、223、224：部分。ALPC、LELP、ETM、FeatureIDSet 均有读取结构；LELP 样本字符串和 FeatureIDSet 整数并不足以证明公开业务属性。分别需要访问器与字段对应、受控属性变更，或非空 ID 实例及运行行为。
- 248：部分。FDO 0..3 已有静态字段序和读取器，参考样本未见旧体；需可定位的旧体真实字节或运行证据。
- 252：部分。FieldInfo ≤3 静态分支及合成测试已有，参考样本已走查实例为 v4；需 ≤3 真实体或旧档运行记录。
- 269：部分。UID、DEP 已取得 Load 证据；BOLP、HLEP 尚缺唯一归属 Load，整体不能闭合。需类标识、RTTI/虚表和函数体交叉确认。
- 272：部分。FeatureLayer 后继 0x1c..0x1f 门和读取器已静态实现；参考样本版本 34 不走这些门，需旧版对象体或运行取证。
- 273：部分。TimeData 四段静态布局、合成测试及参考样本四空槽已见，非空实现链未获真实对象；需启用时间属性的图层或运行记录。

可用性：安装介质和仓库样本可读，本机有 wine 和 QGIS 启动入口，但未发现可用的 ArcMap 10.1 安装或已连接的 Windows 目标；不能将 wine 存在等同于可运行 ArcMap。未在 ArcMap/QGIS 打开检查源工程或转换结果。

## 10.8 介质中的既有版本模板补查

`temp/arcmap108-media-mxd` 的 68 份均可由当前 CLI 返回报告：31 份到达 CGL 循环后尾段、36 份在 CGL 元素组未知对象处停止、1 份为其他停止。这里 10.8 指安装介质来源，不是磁盘版本；以 `temp/arcmap108-media-streams/index.json` 的 `/Version` 为准。本轮不将报告通用说明中的“GroupLayer/FDOGraphicsLayer”文字当作对象实例。

定点检查 `southernusa.mxt.756AEF33_7F48_48CE_804C_228E110C42CC`：解析器到达第 1 个 FeatureLayer 的 FieldInfo 循环，报告计数 11、已消费 11；此诊断未列出 FieldInfo 各对象版本，不能据此闭合第 252 行。该 FeatureLayer 随后在标注 TextSymbol 嵌套符号停止，部分其他有界图层解析失败或保留未知。要验证 FieldInfo ≤3，需从这条已到达对象流的读取结果输出各对象版本及边界后再核对原始字节；不能直接把循环计数视为旧版本。其余项本轮没有新增可闭合的对象级证据。

## CGL 尾段与 FieldInfo v3 补充（后续轮）

此前停点 `0x24f22` 是子层循环终点，不是 CGL 体终点。ArcMap 10.1.0.3035 FeatureLayer.dll（SHA-256 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`）的 Load `0x6f6b2100` 尾段依次读 int32、ILayer 对象、版本 >2 的对象、版本 >=4 的 String、版本 ==4 的 short、版本 >=5 的两个 double，随后返回。Load 伪代码在 `/tmp/r3/annolayer_work/cgl_load_6f6b2100.c`；虚表相邻槽 `0x6f7aa308/0x6f7aa30c` 指向 Load/Save。Save `0x6f6b2620` 的 Ghidra（经桥接工具） 证据 `ev_b96a845b02442287ae05d724a7fd78cac279534e21a26b6770d42b98d8d42072` 显示末段同序写 int32、两个对象和字符串、双精度数；写侧使用文档版本门，不能将其直接外推为所有历史读侧架构版本。旧文档中的虚表地址与该相邻槽地址存在差异，以本轮 PE 字节及函数关联为准。

参考样本 `/Maps`（完整登记上下文）：`0x24f22..26` int32 1；`0x24f26..36` 第一个对象空标记；`0x24f36..4a` 第二个对象为键 30 回引；`0x24f4a..50` 空 String；`0x24f50..60` 两个零 double。CGL 在 `0x24f60` 结束。下游 Map 后缀实际到达 `0x24f64` WKSEnvelope，最终保留未知尾部起点 `0x24fa0`。这是静态代码与样本对位，不是目标软件运行验证。未知新对象体无外部长度，读取器在相应对象头回退并返回 TailNewObject；完整体才返回 Completed。

10.8 安装介质模板 `southernusa.mxt.756AEF33_7F48_48CE_804C_228E110C42CC` 原始文件 SHA-256 `60bc00446ebd5b6de9a38d78c3a45072c41f0e272a17c722b081eb0b21a15389`，其 `/Version` 文本 `9.3`、short 16。经完整 Map 图层列表解析到首个 FeatureLayer（键 40）的 FieldInfo 循环，11 个新 FieldInfo 体均完整、版本 3。对应 `/Maps` 对象头及键分别为 `0x931`/59、`0x987`/60、`0x9df`/61、`0xa3f`/62、`0xaa1`/63、`0xb05`/64、`0xb69`/65、`0xbc1`/66、`0xc13`/67、`0xc6f`/68、`0xce7`/70；每个对象头后版本字节均为 `03 00`。这些是解析器按宿主循环到达后对原始头和版本字节复核，并非单凭 GUID 扫描。第 252 行因此获得 v3 真实字节佐证；v0–2 仍无该样本证据，是否将该条拆分或闭合由主代理按条目要求决定。未在 ArcMap 人工打开模板。

## CGL 尾部空间参考槽（补线）

Load `0x6f6b2100` 在 CGL 架构版本 >2 时，以 `DAT_6f7aa48c` 请求第二尾槽；该常量经类型库定名为 ISpatialReference `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`（`arcmap-10.1-map-object.md` 请求 IID 定名表；原记录“≥2”应以 Load 的 `2 < version` 为准）。本轮仅对第二槽新 PCS、GCS、Unknown 分别调用已有 Geometry 10.1 对象体严格读取器，记录对象键；第一槽及其他类仍在头部保守停止。版本超限、截断等错误整体回滚；不将 10.1 读取规则泛化为任意历史软件实现。

10.1 安装介质 `ANSI_C_Landscape.mxd`（SHA-256 `5e76a99d479caf5702b9f05bcbd899ca20a86d3662b74e51335b4d0fec28bbad`，/Version 文本 9.3、short 16）从 Map 根到达该槽：`/Maps:0xe0` 的新 GCS 对象键 32，体首 `0xf4` 为版本 7，属于 10.1 GCS 读取器支持的范围，随后为 GEOGCS WKT。接线后 CGL 完整消费，Map WKSEnvelope 位于 `0x246`，后续未知区自 `0x282` 起。参考样本第二槽仍为键 30 回引，范围位于 `0x24f64`、未知区自 `0x24fa0` 起。均为静态文件及解析器观察，未运行 ArcMap/QGIS。

合成测试覆盖 PCS/GCS/Unknown 版本 0 形态、未知类停止、读取器超版本及截断事务回滚；真实模板提供 GCS 版本 7 的辅助对位，不提供 PCS/Unknown 新对象的该槽样本证据。
