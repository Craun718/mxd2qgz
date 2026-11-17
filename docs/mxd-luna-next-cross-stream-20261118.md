# ObjectStream 跨流根链复核（2026-11-18）

## 范围与身份

本记录只审查 ArcMap 10.1 的 ObjectStream 在真实 MXD 根流间是否延续，以及将 `/Maps` 与 `/PageLayout` 分开解析时能直接观察到的状态差异。未改公共解析源码与 `TODO.md`，未运行 ArcMap；未在 ArcMap 或 QGIS 中人工打开样本。

原生材料取自 `/home/gxrs/文档/source/arcmap10.1/`。PageLayout.dll 为 10.1.0.3035，SHA-256 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`；SystemCore.dll 为 10.1.0.3035，SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。依据是已导出的函数伪代码与静态调用链，不是原始源码或运行时观测；10.1 研究导出边界见 [研究成果使用指南](arcmap-10.1-research-source.md)。

样本来自 ArcMap 10.1 安装介质 `Setup1.cab`，已提取的 CFB 文件位于忽略目录 `temp/arcmap101-media-mxd/`，对应流位于 `temp/arcmap101-media-streams/`。来源、载荷摘要和提取方法见[介质样本记录](mxd-install-media-samples-20261117.md)。介质来源说明文件出处；各样本实际 `/Version` 仍按流字节读取。

## 原生根链：同一实例切换底层流

PageLayout `0x7b708d00`（10.1 导出 `PageLayout/7b708d00.c`）在函数前段创建对象流局部对象 `local_18`。打开 `Mx Document` 后，它先连接版本流并读取根头/地图数量；在 `/Maps` 分支中，该对象流经虚表 `+0x38` 连接 Maps 流，循环按地图数量以 `+0x44` 加载对象；循环结束后读取 Maps 尾字段，再打开 `PageLayout` 存储，并对**同一 `local_18`** 再次调用 `+0x38` 连接 PageLayout 流，随后通过 `+0x44` 读取页面布局对象。对象流创建、Maps 循环、PageLayout 切换和读取顺序均在同一函数的控制流中。

SystemCore `0x7ffcf7a0` 是该对象流接口的 `SetStream` 实现。10.1 伪代码只替换对象的底层流指针并对新旧 IStream 做 AddRef/Release；不清读表或写表，也不重置键计数器。读侧 `0x7ffcff70` 在新键对象头后先登记，再调用该对象的 Load；回引依据是当前 ObjectStream 读表是否已有该键。相关读写细节及接口槽位见[对象体记录](arcmap-10.1-object-body.md)。因此静态实现证据支持：Maps 与 PageLayout 处在同一文档级 ObjectStream 引用会话中，后流可见前流登记的键。

这证明“同一引用会话”这一机制，不证明任意一份 MXD 必然含 Maps→PageLayout 的实际跨流回引。若 PageLayout 里出现此前未在本解析上下文登记的旧键，流内 20 字节头本身不能区分回引与新对象；使用独立空表会把它错分为新对象，并可能把回引键后的字节误当对象体。

## 真实介质根头对照

下表为流偏移 0 的原始根头；`/Maps` 开头另有 u32 地图数，故 Maps 根对象头位于偏移 4。Guid 按 Windows 磁盘字节顺序列出，键为紧随其后的 u32 little-endian。

| 10.1 介质样本 | `/Version` 文本 / 对象计数 | `/Maps` 地图数 / 根对象键 | `/PageLayout` 根对象键 | `/Maps` 长度 | `/PageLayout` 长度 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `Africa.mxd` | `10.1` / 637 | 1 / 38 | 536 | 50,452 | 8,636 |
| `CentralUSA.mxd` | `10.1` / 855 | 1 / 38 | 736 | 69,845 | 10,840 |
| `LandscapeClassic.mxd` | `10.0` / 149 | 1 / 29 | 84 | 5,008 | 4,969 |

`/Maps` 根 GUID 是 Map 类 `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`；`/PageLayout` 根 GUID 是 PageLayout 类 `{DD94D76E-836D-11D0-87EC-080009EC732A}`。两条流的根键并不按各自流重置：例如同一 Africa 文档根键为 38 和 536，CentralUSA 为 38 和 736，且 `/Version` 尾部记录的对象计数更大。这与原生同会话单调键分配相符，但只凭根头和总数不能证明全部中间键连续、逐一对应或不存在未解析对象。

另有已闭合的 CentralUSA `/PageLayout` 局部真实解析：从 PageLayout 根键 736 起，首个 TextElement 为键 737，其嵌套空间参考新键 743；随后 LineElement 为键 744，并回引键 743。按完整页面元素顺序共享登记表时第二项可解析；只从第二项新建空表时，键 743 被按新对象处理，体读取在截断处失败且游标/登记表回滚。细节及字节偏移见 `tests/round3_line_page_layout.rs` 中 `central_usa_first_two_elements_use_real_reference_definition`。这个差异是**页面流内部**的实测解析状态差异，不能冒充 Maps→PageLayout 的跨流回引实例。

## 独立测试与可观察边界

新增 `tests/luna_cross_stream_20261118.rs` 中被忽略的 `central_usa_maps_then_page_layout_preserves_one_reference_table`：从原始 CentralUSA CFB 读取 `/Maps` 与 `/PageLayout`，先在共享表登记 Maps 根 Map 键 38，再模拟 SetStream 换流并登记 PageLayout 根键 736；对照新建独立表读取 PageLayout 根，确认该表只有键 736、没有此前的键 38。测试只消费两根对象头，不声称已解析 Maps 对象体或遍历完整 MXD 根链；它将原生“同一实例换流”的状态要求落实为独立回归断言。

根头这一步的可观察差异是登记表状态，而不是根头分类结果：两个上下文都会把 PageLayout 根键 736 分类为新对象。现有证据没有给出 CentralUSA PageLayout 对键 38 的实际回引，因此测试不构造或声称这种样本事实。对真实跨流键的验证仍需从 `/Maps` 根体完整解析出对象表，再沿 `/PageLayout` 根链逐项读取和分类；未知对象体不可跳过，也不能通过 GUID/键扫描替代。

## 版本适用性与待闭合项

- 同实例 `SetStream`、读表不清空和读侧查表行为均直接来自已核验身份的 ArcMap 10.1.0.3035 SystemCore/PageLayout，适用于 10.1 目标的静态规则。
- 已有 10.8 文档曾记录同类对象流和跨流状态模型，但本轮没有复核 10.8 二进制。本轮不能把所有类体布局、Document 根链细节或此处具体静态地址外推到 10.8/其他版本。
- 真实介质只证明对应流中的字节布局和样本对象键；其安装介质来源与 `/Version` 版本字段都不等同于运行 ArcMap 保存验证。
- 尚未从现有解析器得到任一安装介质文件完整 Maps→PageLayout 对象登记表，也尚未找到经根链解析确认的跨这两条流的具体回引对象。未知体边界仍是该验证的主要缺口。
- ArcMap 运行取证按任务约束未执行；静态反编译和独立读取测试不替代目标软件打开与人工检查。
