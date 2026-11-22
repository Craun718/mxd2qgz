# Version 组逐项复核（2026-11-18）

范围：当前 `TODO.md` 原行号 95–113、138–139，以及 21、22、28、35。本文只新增研究记录；没有改写 `TODO.md` 或其他代理正在编辑的文档。

## 逐条状态

| TODO 原行 | 状态 | 本轮判断与证据边界 |
|---:|---|---|
| 21 | 部分 | PageLayout `0x7b70a8e0` 的直接静态调用者是 `0x7b69f6b0`，其反编译显示在打开文档并取得一个版本比较结果后调用该函数；`calls.tsv` 还列出 9 个直接调用 `0x7b69f6b0` 的函数。调用者的上游及虚调用来源未闭合，故不能称为完整调用链。 |
| 22 | 可勾选 | 对 PageLayout 此局部读取器，格式已静态闭合为 `String::Read` 字节序列后读 2 字节有符号 short；输出先设为 5，short `< 0x12` 时改为 3。`String::Read` 的长度字段、流转发和短读边界另由 AfCore 10.1 证据闭合。此结论仅限 PageLayout 局部读取，不代表所有 `/Version` 消费者或全流边界。 |
| 28 | 已复核 | 后续原始指令确认 `String::Write` 用 32 位加倍和逻辑右移实现前缀掩码，AfStreamWrite 原样转发。[勘误](arcmap-10.1-native-baseline.md#101-afcore-string-流转发与写入调用者2026-11-14) |
| 35 | 部分 | 保存入口 `0x466fb0` 经 CMxDoc 虚槽 `+0x14c` 读取索引；Save A Copy 的静态分支把选择值经 CMxDoc `+0x148` 写槽到 AfDoc put `0x42b120`，构成静态来源链。对话框字段的准确语义、全部返回值/取消分支和运行时实际派发仍未验证。 |
| 95 | 部分 | 已知 10.1 文本、short，以及 short `> 11` 时的 4 字节对象表计数；介质模板扫描显示已核对的 134 个条目（102 种内容）在该头后无余字节。但这不能赋予可能存在的额外尾字节语义；目标软件写入路径和额外尾部消费者仍未穷尽。 |
| 96 | 可勾选 | 两套 10.1 安装介质模板记录已比较版本头之后的字节，134 条/102 种内容均无剩余字节，见 `docs/mxd-install-media-samples-20261117.md`。这是有限介质样本观察，不是尾部语义证明。 |
| 97 | 待验证 | 本轮未运行 ArcMap 10.1，也未人工保存/重开受控工程；Rust 合成测试不能替代此项。 |
| 98 | 部分 | 静态 Save A Copy 分支能追到 put 版本索引的接口槽，但对话框成员、版本列表过滤含义以及所有 UI 分支尚未绑定，实际软件派发未观察。 |
| 99 | 待验证 | 现有英文资源静态检查发现索引 3 资源候选异常；本机介质中没有可比较的本地化 `ResArcMap.dll` 资源，故其他语言的文本及识别影响未知。 |
| 100 | 部分 | 新 dossier 的 `0x47a740` 装载分支调用 `0x4764d0`，将其返回值写到文档 `+0x148`，没有在该函数直接写 `+0x14c`；构造初值为 Current=5。仍未穷尽间接写入者和其他加载路径，旧档打开后索引是否改变及再次保存结果仍需运行观察。 |
| 101 | 待验证 | 10.1 short/枚举映射有静态表证据，尚无 ArcMap 10.1 多版本保存并重读的运行验证。 |
| 102 | 待验证 | CLI 输出的索引是文本与 short 对照得到的候选值，不是额外的流内索引字段；尚未与软件中实际选择并保存的版本逐档比较。 |
| 103–109 | 不属本组 | 这些原行属于坐标系读取研究，不在 Version 组范围内，本轮未复核或更改。 |
| 110 | 可勾选 | 仓库已有 OLE `/Version` 等命名流读取及版本候选识别接线；见 `src/version/` 与 `tests/version_stream.rs`。这只说明仓库实现和合成输入契约，不表示目标软件验收。 |
| 111 | 可勾选 | 显式 10.8 局部 `/Version` 严格读取器已保留文本、signed short 位模式和剩余位置；现有正常与异常合成测试覆盖局部字段、截断和游标回滚。10.8 跨版本语义未因此闭合。 |
| 112 | 部分 | 已有 10.8 静态证据将条件 4 字节写回关联到对象写表条目数；额外尾部及失败/边界分支尚未闭合。不得直接套用 10.1 规则补全 10.8。 |
| 113 | 待验证 | 没有 10.1/10.8 同一受控工程的打开、保存和 `/Version` 字节对照运行结果。 |
| 138 | 部分 | 现有实现只对明确研究过的 10.1 和 10.8 字符串布局给出显式读取器；其他年代的字符串族、适用条件和调用面未闭合。 |
| 139 | 待验证 | 其他 MXD 版本对象属性规则仍需按目标原生版本逐类取得证据；当前 10.1 结论不可泛化。 |

## 10.1 PageLayout 静态复核

对象身份按本地反编译目录清单与仓库基线对应：PageLayout.dll，PE 资源版本 10.1.0.3035，SHA-256 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`；地址均为该静态映像地址。

- `PageLayout/7b70a8e0.c`：在输入状态允许时先写输出默认值 5；打开 OLE 命名流 `Version`，经流对象连接后调用 `String::Read`，再由底层 IStream 读取 2 字节到 short；按有符号比较 `< 0x12` 时输出改为 3。两次读取 HRESULT 在该调用者中未检查。随后它调用接口虚槽 `+0x28`，传入常数 `0x12`；仅从现有材料无法证明该槽的完整业务语义。
- `PageLayout/7b69f6b0.c` 与 `calls.tsv`：该函数直接调用 `0x7b70a8e0`；存在多处直接调用 `0x7b69f6b0` 的静态边。调用索引没有恢复更上游的 COM 虚调用，因此行 21 仍标部分。
- 10.1 AfCore `functions/0x68ab46f0.c`：`String::Read` 通过传入 IStream 的虚槽 `+0x0c` 先读 4 字节长度，再按长度分配并读数据；没有 cb 掩码，也没有完整读取量或终止零检查。Rust 解析器采用严格安全子集，不模拟该原生短读。
- 原始指令交叉核对：本机 `objdump -d -Mintel` 读取 PageLayout.dll `0x7b70a8e0`，确认输出默认值写 5、`Version` 读取代码位置、short 的 `< 0x12` 分支以及函数边界。反编译和机器码均是静态证据，不是运行取证。

## REA / pyghidra 尝试与限制

主代理另以获授权的本地 REA 成功生成 ArcMap.exe `0x47a740` dossier（Ghidra 12.2；输入 SHA-256 `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97`），保存在 `temp/luna-20261118/rea-local/arcmap-47a740.json`。它显示 `CDocument::Serialize` 覆写按 CArchive 方向分支：装载时调 `0x4764d0` 并把返回值写到 `+0x148`，保存时调 `0x466fb0`，其保存入口再从 `+0x14c` 取索引。该证据未显示 UI 虚派发，也不能证明旧档加载的间接副作用。

本轮还在默认沙箱下对 PageLayout `0x7b70a8e0` 请求 Ghidra function dossier。REA 导入并自动分析了 10.1 PageLayout.dll，但 Ghidra bridge 创建 Unix socket 时收到 `java.net.SocketException: Operation not permitted`，请求失败；没有把它记成成功 dossier。日志和临时项目位于 `temp/luna-20261118/workspaces/pagelayout-version/`，其中 `rea-function.log` 保留桥接失败详情。

随后使用本机 Ghidra 12.2 随附 PyGhidra。离线安装步骤将随附 wheel 安装进本轮专用 venv；一次 headless 启动因无 X11 失败。设置 `java.awt.headless=true` 后，后续系统执行器因全局 `/tmp` 仅剩 33 MiB、tmpfs 已达 100% 而无法启动（`No space left on device`），所以没有得到 PyGhidra 反编译输出。对应启动日志保存在上述临时工作区。此次补导未成功；本记录中的 10.1 结论来自已有完整 PageLayout/AfCore 导出、现成原始 DLL 和 objdump 静态复核。

本轮没有发现需要新增到 Rust 代码的已证实解析规则：现有 `version_stream` 已实现上述受支持的严格子集，且现有测试覆盖常规头部、条件计数、未知尾字节保留和截断失败。故本轮不改 `src/version`、`tests/version_stream.rs`，也未运行测试。没有运行 ArcMap 10.1/10.8 或进行 QGIS/ArcMap 人工验收；所有状态均保留相应运行验证缺口。

### PageLayout 分类后的版本属性调用

在 PageLayout.dll 10.1.0.3035（SHA-256
`ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`）中，
分类入口 `0x7b70a8e0` 的 `+0x28(0x12)` 调用接收者来自
`0x7b6a7ae0 → 0x7b6a8590`。后者在 `0x7b6a85e3` 将
`0x7b71e02c` 的 IID 传给 QI。原始字节解码为
`E8E5589D-215A-4322-BA68-E52F187363EE`，与 Metadata 路径
[已对位的接口](mxd-terra-round9-save.md) 相同。

- DocumentInfo 的接口虚表 `0x7b71ca9c` 在 +0x28 指向
  `0x7b692320`。该入口把传入 short 写到接口 this+0x60，
  即完整对象 +0x70，返回 S_OK。
- MapDocument 的同 IID 虚表 `0x7b71decc` 在 +0x28 指向
  `0x7b691050`，直接返回 E_NOTIMPL。

这里写入的实参固定为 18，与返回给分类调用者的 3 或 5 分开。
同版 PageLayout.dll 中，DocumentInfo 构造器在 `0x7b6911da` 初始化
完整对象 +0x70。旧式 Metadata 读取路径在 `0x7b692c74` 将此
short 符号扩展并与 7 比较。达到 7 且此前操作成功时，继续通过
`AfStreamRead` 读取两个 4 字节值（第一项写入 +0x4c，第二项写入
+0x3c）。对偶写路径 `0x7b692e00` 在 `0x7b692ed8` 也比较 7，
达到门槛时先写 +0x4c，再写一项 4 字节零值。
因此固定值 18 若保留到这些入口，会通过 Metadata 尾段的版本门。
分类调用的实际接收者及从分类到 Metadata 的值传递仍待核对。
本轮使用本地伪代码和 objdump，未在 ArcMap 或 QGIS 中人工验证。

### 旧文档分类后的 DocumentInfo 标志

在同一 PageLayout.dll 中，链 B 从上下文 +0x08 取接口，
于 `0x7b69f90f` 调用 +0x58 槽。实参由打开例程的局部输出
归一化为 0 或 -1（`0x7b69f8f5–0x7b69f8fc`）。
在使用 DocumentInfo 的路径上，初始化例程 `0x7b70aba0`
将上下文 +0x08 的地址传给创建助手 `0x7b70e560`，
后者创建 DocumentInfo 并查询 IDocumentInfo2。
DocumentInfo 的 QI 命中该 IID 时返回完整对象 +0x18
（`0x7b69149e`），构造器在此安装虚表 `0x7b71cb24`。

该表 +0x58 的指针位于 `0x7b71cb7c`，指向 `0x7b692640`。
该入口将实参解释为有符号 short，并把是否等于 -1 的 0/1
结果写入接口 this+0x34，即完整对象 +0x4c。这个成员正是
旧式 Metadata 尾部读写的首个 4 字节字段。以上绑定以调用时
上下文 +0x08 仍持有该 DocumentInfo 为条件，尚未在 ArcMap
或 QGIS 中运行验证。

该标志的输入可以追到主读取器的字节读取。`7b69f6b0.c` 中的
局部变量 `local_24` 通过 `0x7b7098b0` 原样传给
`0x7b708d00` 的第八个参数。按同版伪代码 `7b708d00.c`，
单流分支在 `Mx Document` 开头读完首个 4 字节版本值后，
将第二个 4 字节值写到该输出地址。分流分支在 `Maps` 的对象
循环后再读 4 字节，成功后写到同一输出地址，再打开 `PageLayout`。
因此旧版分类决定是否更新 DocumentInfo，两个文档布局各自的流内
值决定更新为 0 还是 1。上述路径以主读取器成功及后续调用分支命中为
前提，未在目标软件中执行验证。

10.1 esriCarto 类型库 TypeInfo 744（IID
`D3E5C54F-2A84-11D4-9FF2-00C04F6BC78E`）的 MSFT 函数记录将
+0x54/+0x58 定名为 `IDocumentInfo2.RelativePaths` 的 get/put。
读槽 `0x7b692610` 将完整对象 +0x4c 的非零值转换为
`VARIANT_TRUE`（-1）；写槽 `0x7b692640` 将 -1 存为 1，
其余输入存为 0。由此可将旧式 Metadata 尾部首个 4 字节成员
定名为相对路径标志。该结论来自本地类型库与 PE 交叉核对，
未在 ArcMap 或 QGIS 中运行验证。

旧式 Metadata 的第二个 4 字节字段在读侧由
`0x7b692ca4–0x7b692cb9` 写到完整对象 +0x3c。若读取失败，
`0x7b692cc5–0x7b692cd5` 调用 DocumentInfo 内部接口 +0x14；
该表 `0x7b71ca9c` 的对应槽是 `0x7b691550`，清理例程于
`0x7b6915e0` 将 +0x3c 置零。写侧 `0x7b692f06` 则将
临时值置零并写出 4 字节。现有证据可确认字段宽度、读入位置与
10.1 写出值，其早期业务含义仍未知。Rust 当前没有独立的 Metadata
读取入口，本轮没有为此引入未验证的布局选择。

### 旧式 Metadata 的字符串顺序

以 PageLayout.dll 10.1.0.3035 的旧式写函数 `0x7b692e00` 为准，
七次 AfCore String::Write 对应下表。属性名来自 esriCarto
TypeInfo 744 的 IDocumentInfo2 公开槽。表中读取入口是
DocumentInfo 虚表 `0x7b71cb24` 的 getter 经调整器到达的函数，
成员偏移按完整对象表示。

| 流顺序 | 成员 | 属性 | getter 函数 |
| ---: | ---: | --- | --- |
| 1 | +0x24 | DocumentTitle | `0x7b692340` |
| 2 | +0x28 | Author | `0x7b6923e0` |
| 3 | +0x2c | Comments | `0x7b692480` |
| 4 | +0x30 | Subject | `0x7b692390` |
| 5 | +0x34 | Category | `0x7b692430` |
| 6 | +0x38 | Keywords | `0x7b6924d0` |
| 7 | +0x40 | HyperlinkBase | `0x7b692520` |

该布局在七串之后才按完整对象 +0x70 的版本值判断是否写两个
4 字节尾字段。它只描述这个旧式入口，其他 Metadata 读写布局
与原生运行仍需独立验证。

同版旧式读取函数 `0x7b692ba0` 按上表顺序读取七串。
当读写入口的文档版本参数 <4 时，两者都通过 `0x7b693500`
选择布局。该函数从 DocumentInfo 完整对象 +0x50 的路径取扩展名，
通过 AfCore `FileSystemPath::GetExtension` 和
`String::CompareNoCase` 比较 `.3dd` 与 `.sxd`，匹配返回 0。
只要路径扩展名匹配，旧式分支就选择上述七串及可能的尾字段
（读 `0x7b692ba0`、写 `0x7b692e00`）。

其余扩展名在版本参数 <4 时走另一对旧式入口
（读 `0x7b692a70`、写 `0x7b692ce0`）。它们在前六串后
读取完整对象 +0x3c 的 4 字节值，或写出常数 1，再处理
HyperlinkBase 和 +0x44 的字符串。Credits 的 getter
`0x7b6925c0` 从接口 this+0x30 读取，即完整对象 +0x48；
流内最后一串实际取完整对象 +0x44，因此该末串暂保留成员偏移，
不能按公开 getter 定名为 Credits。两种选路依据来自同版伪代码、
PE 原始字符串和导入表，本轮未运行 ArcMap。

### 带版本头的 Metadata 对象体

DocumentInfo 的 `0x7b691650` 和 `0x7b691b50` 在外部文档版本
参数 ≥4 时使用带头布局。写入口的版本常量 `0x7b71c874` 为
`04 00`。以同版导出的两个函数及 PE 常量为证，v4 的顺序如下，
字段偏移均已从接口视角换算为完整对象偏移。

| 流顺序 | 宽度或编码 | 对象成员 |
| --- | --- | --- |
| 1 | 2 字节版本 | 写出 4 |
| 2 | 六个 AfCore 字符串 | +0x24 至 +0x38，名称同上表前六项 |
| 3 | 4 字节 | +0x3c |
| 4 | 四个 AfCore 字符串 | +0x40 HyperlinkBase、+0x44 未定名、+0x48 Credits、+0x50 Path |
| 5 | 三个 8 字节值 | +0x58、+0x60、+0x68 |
| 6 | 4 字节 | +0x4c RelativePaths |
| 7 | AfCore 字符串 | +0x74 |

读侧在体版本 <2 时，分别在 +0x48 和 +0x50 字符串之后
多读一个临时字符串。只有体版本 ≥3 才读 RelativePaths，≥4
才读取 +0x74 并进入随后的存储操作。+0x44 在带头与旧无头布局
中均出现，仍缺正式属性名。本轮未核对存储操作的额外流，也未在
ArcMap 中执行读写。

### Metadata v4 后的预览流

读取 v4 末串成功后，DocumentInfo 调用 `0x7b693260`，
将同一个 IStorage 传入。该函数首先尝试打开独立的
`Mx Document Preview` 流，成功取得流时先读 4 字节版本，
大于 1 则返回错误。若未取得该流，则尝试打开 `Preview`，
这个回退分支不预读版本。两条路径随后经 OLEAUT32 ordinal 418
请求 IID `7BF80980-BF32-101A-8BBB-00AA00300CAB`（IPicture），
输出存到 DocumentInfo 完整对象 +0x78。以上路径取自 ArcMap 10.1
PageLayout.dll 的同版伪代码及 PE 中 `0x7b71c72c` 的 IID。

写端的对应函数 `0x7b692f40` 处理已有独立预览流。根据
`0x7b69307a–0x7b69308b` 的指令，创建时实际传入
`0x7b71c8b4` 指向的 `Preview`，随后调用图片接口保存。
故此调用不扩展 Metadata 流的内联字段边界。本轮未执行
ArcMap 的预览加载或保存。

对 10.1 安装介质的 66 份 MXD/MXT 从 CFB 根独立打开
`/Metadata` 和 `/Preview`。66 份均有非空 Metadata 和独立
Preview，均无 `/Mx Document Preview`。其中 18 份的 Metadata
以 `04 00` 开头，按本节位宽调用 Rust Reader 和
`read_string_10_1`，全部恰好消费到该流末尾。其余 48 份未按 v4
布局解释。该实验确认了 v4 的实档边界和预览流分离，未验证图片内容
或 ArcMap 打开时实际使用的分支。

对同一批介质继续按 `0x7b691650` 的版本分支核对。剩余
48 条中，11 条以 `03 00` 开头，读完三项 8 字节字段与
RelativePaths 后恰好到达流尾。另 37 条的开头与第一个
AfCore 字符串的 4 字节长度一致。依照 `0x7b692a70` 的布局，
读六串、一个 4 字节值及后两串，也全部恰好到达流尾。
这 37 条的 4 字节字段为 0 或 1，分布为 34 条零、3 条一。
10.1 的对应旧式写入口写常数 1，介质中出现的零值仍需结合
文件写方来源解释。上述结果验证字节边界，尚未在 ArcMap 中
验证具体布局选择。
