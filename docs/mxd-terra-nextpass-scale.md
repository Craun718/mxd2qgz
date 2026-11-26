# ScaleLine 原生持久化续查（2026-11-17）

## 身份与方法

只追踪真正的 `ScaleLine`、`SteppedScaleLine` 及其委派段；原先三个伪候选的更正见 [既有记录](mxd-terra-integrate-page.md)。地址为下述 PE 的虚拟地址。重算 `temp/agent-map-surround/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` SHA-256 为 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，PE32 x86、ImageBase `0x793a0000`；版本资源自述 Esri Map DLL、OriginalFilename `Map.DLL`、FileVersion/ProductVersion `10.1.0.3035`。注册包 `esri.clsid.ecfg` SHA-256 `170ae1f812bafc317ac98455eb7be844dd37f8d3a158c454cedf12f5046c3486`。文件自述身份与摘要不等于签名验证。

独立 Ghidra 桥接工具 6.9.0 CLI、JDK 25 和仓库独占 TMPDIR 启动 Ghidra 12.2；目标导入后桥接 Unix socket 的 bind 返回 `Operation not permitted`，未得到 Ghidra 桥接工具 伪代码（诊断保存在 `temp/terra-nextpass-scale/rea.json`）。按任务约定改用 `GHIDRA_INSTALL_DIR=/opt/ghidra` 的隔离 pyghidra，将已有项目**复制**到独占目录后打开副本，未修改共享目标或项目。反编译结果在 `temp/terra-nextpass-scale/0x*.c`；身份链另用 PE 字节及 objdump 核查。产物含本机路径，分享前须脱敏。以下是静态观察，并非原始源码或运行取证。

| 类 | 注册 CLSID | RTTI、持久化虚表 | Load / Save 实体 |
| --- | --- | --- | --- |
| ScaleLine | `{6589F140-F7F7-11D2-B872-00600802E603}` | TypeDescriptor `0x794b4d88`；COL `0x79497fe4`，位移 `0x2ac`；虚表 `0x7948c858` | `+0x14` → 跳板 `0x79453b80` → `0x794537f0`；`+0x18` → `0x79453aa0` → `0x79453870` |
| SteppedScaleLine | `{6589F141-F7F7-11D2-B872-00600802E603}` | TypeDescriptor `0x794b53a8`；COL `0x79499f24`，位移 `0x2b4`；虚表 `0x7948e530` | `+0x14` → `0x794685c0` → `0x794682d0`；`+0x18` → `0x79468550` → `0x79468360` |

跳板均先按接口指针前保存的动态位移和额外常数调整 `this`，不能直接把不同虚表的参数偏移等同。独立 identity 方法 `0x794530b0` / `0x79467a00` 分别复制 `0x79485098` / `0x794850a8` 的上述 CLSID，与注册包相符。持久化虚表的 `+0x0c` 则经调整跳到 `0x794510b0`，从实例取 16 字节，并非直接读取上述常量；静态数据不能单独证明实际执行时该实例字段为何值。`ScaleLineImpl` RTTI `0x794b4d6c` 是另一内部类型，不能据名字当成可独立持久化 coclass。

## 读写顺序和边界

两个具体类的 Load 首先读取自身 short 版本，有符号值大于 1 拒绝；负值通过该原生上限比较。Save 首 short 均为 1（`0x79480d18` / `0x79480cf0`）。随后两者顺序完全相同：

1. `0x79451120` 基类 Load；Save 为 `0x79451400`。它从 short 基类版本开始（有符号 >6 拒绝），再调 `0x794413d0` 共同前缀；之后的文本、标量、ITextSymbol×2 和条件 INumberFormat 等字段及版本门见 [已有逐字段记录](arcmap-10.1-legend-family.md)。写侧版本表和字段也在既有记录中，不在此重复。基类 `<2` 的额外 int32 是读侧旧分支，10.1 当前写侧不会写出该项。
2. `ScaleLineImpl` 委派段 Load `0x79453ef0`：short 版本（有符号 >1 拒绝），再 `LoadObject` 一个 ILineSymbol；其 IID 常量 `0x7948c8d0` 读取为 `{E6BDAA7D-4D35-11D0-98BE-00805F7CED21}`。Save `0x79453f70` 写版本 1 和该槽。**此段没有 double 或填充符号**，与已有四个 Scalebar 子类的中段不同。
3. 线段尾部 Load `0x79454ee0` / Save `0x79455030`：short 版本（有符号 >2 拒绝；Save 写 2）；版本 <2 先读 short 标志；然后两个 ILineSymbol 槽、8 字节×2、4 字节；版本 ≥2 再读 4 字节。版本 <2 时最后一个内存字段由标志计算为 0 或 5，**不从流中读**。Save 对应当前版本写两个槽、8 字节×2、4 字节×2。8 字节数值和标量的业务名尚未绑定，按原始位模式处理。
4. 仅 SteppedScaleLine 在上述尾段之后读取 8 字节（Load `0x79468340`，Save `0x794683d1`）。`0x79467a60` / `0x79467a70` 对相关实例位置作 double 读取和写入；访问器名称未闭合，不能据此命名其业务含义。Save 和 Load 对称，不依赖自身版本分支。

所有嵌套 `LoadObject` 都可能产生无外部长度的新对象体：解析器必须完成该体或在该对象头前回退游标及登记，不能扫描 GUID 或跳过未知长度。短读、非法版本、长度和键登记均需保持事务性。字段识别仅限 10.1.0.3035；尚无可控字节、ArcMap 执行或 QGIS 验收证据。

## 接线裁决与可勾边界

静态证据足以独立闭合**两具体类身份、Load/Save 跳板链及相互区别的持久化字段序**；可作为这些独立研究子项的勾选依据。业务属性命名、流内实例、目标软件运行验证不能随之勾选。

本轮没有接入解析器。既有 `read_scalebar_subclass_body_10_1` 的中段是 double 加填充符号，不能直接用于本类；它的尾段对版本 <2 仍无条件读取第二个 int32，与上面的原生旧分支不符，而且对未知符号未传播停止，不能安全复制其完整消费假设。正确接线还需要在 MapSurround 宿主循环分派新 CLSID，而当前分派位于 `src/maps/map_object.rs`，不在本轮独占的 `element_decor` / `map_surround` 范围内。应先实现可事务回退的基类及符号槽读取、单独修正旧尾段，再由拥有宿主文件的工作接线；不可因静态链闭合而声称 `parse` 已支持。上述问题是针对复用路径的代码审查结论，本轮未修改这些既有实现。

拟覆盖的构造验证：两类各自版本 1、Stepped 末尾 8 字节及哨兵边界；各版本门和尾段 0/1/2（旧版本不误读后继 4 字节）；空槽、已登记回引、未知新对象停止并回退登记；截断、超限版本及嵌套失败时游标和登记事务性。合成测试不能替代源工程与转换结果的人工核验。

本轮没有新增解析器或 ScaleLine 构造测试；对当时的整个工作区运行 `cargo fmt --check` 和 `cargo test` 均通过，这只能说明现有测试通过，不能证明上述新格式已被验证。

ArcMap 10.1 环境暂缺；未在 ArcMap 或 QGIS 打开工程，也未作图层、显示或布局目视核查。

## 专用读取入口（续轮）

上述“本轮没有接入解析器”保留为上一轮记录。本续轮在 `src/maps/map_surround_family.rs` 增加两个 CLSID 常量、`read_scale_line_10_1` / `read_stepped_scale_line_10_1` 和带类别参数的共享入口。调用者先读取对象头并登记键；入口从对象体读取，`stop == None` 表示已消费已确认的完整对象体。未知或未完成对象槽按整个槽回退、撤销该槽登记并返回槽名和偏移；任何错误回滚本次调用的游标和全部登记。关联 Map 新对象沿用共同前缀读取器的 Unsupported 错误。线符号新对象仅分派现有已知线符号类，回引沿用已有登记表的键判定，不能由该表验证接口类型。

旧尾段 0/1 不读取末尾 int32；原始标志由 `tail_legacy_flag` 保留，可按原生规则计算内存默认值。版本位模式限制为非负已确认子集，较原生有符号上限比较更严格；原始标量和 Stepped 附加值以字节保存。构造测试覆盖两类、基类 0..6、尾段 0..2、哨兵、回引、已闭合对象、未知及嵌套未闭合槽、逐字节截断和非法版本。没有修改既有四类读取器，也未接入 `map_object.rs` / `mxd_maps.rs`；宿主分派由后续集成完成。所有测试均为构造输入，仍需真实流及 ArcMap/QGIS 人工验证。

主审按两个 CLSID 接入 MapSurround 循环，构造两个连续新对象并验证读取后续非空显示范围的起点。实际 MXD 与目标软件验收仍未执行。

## IScaleBar 属性与旧尾段边界

复核上文标识的 Map.dll 10.1.0.3035，另按 esriCarto.tlb 10.1 的
MSFT TypeInfo 成员表读取 IScaleBar（6DFAE8E2-ECAA-11D0-8682-0000F8751720）
的 oVft。SteppedScaleLine 的 RTTI 将虚表 0x7948e2cc 放在完整对象 +0xd8，
持久化 Save 实体的 this 在 +0x268，传给基类 Save 0x79451400 的指针在
+0x26c。故基类成员相对 IScaleBar 的偏移等于其 Save 偏移加 0x194。
以下 getter 的取数指令由 objdump 复核。

| IScaleBar 属性 | oVft | getter | 基类 Save 成员 |
|---|---:|---:|---:|
| Divisions | 0x3c | 0x7944f9d0 | −0x174 |
| DivisionsBeforeZero | 0x44 | 0x7944fa50 | −0x172 |
| Subdivisions | 0x4c | 0x793a1070 | −0x170 |
| Units | 0x54 | 0x7944fb10 | −0x16c |
| UnitLabel | 0x5c | 0x794573b0 | −0x168 |
| UnitLabelPosition | 0x64 | 0x793b8800 | −0x164 |
| UnitLabelGap | 0x6c | 0x7944fc40 | −0x15c |
| LabelFrequency | 0x7c | 0x7944fe30 | −0x150 |
| LabelPosition | 0x8c | 0x79450080 | −0x148 |
| LabelGap | 0x94 | 0x794500f0 | −0x144 |
| ResizeHint | 0xb4 | 0x79450430 | −0x138 |

BarHeight 的 get/put（oVft 0xac/0xb0）进入 0x79450370/0x794503b0，
还原完整对象指针，再调用主虚表 0x7948e0b4 的 +0xd8/+0xdc。
在 SteppedScaleLine 两槽分别是 0x79467a60/0x79467a70，对完整对象
+0x2a8 的 double 读写。末尾 Save 成员 +0x268+0x40 正好对应该值。
Rust 保留现有字段名 stepped_double。尚未验证 ArcMap 界面显示单位。

四类填充比例尺共用的尾段读取已按 0x79454ee0 修正。版本 0/1
根据 short 标志恢复第二个 int32 成员（零为 0，非零为 5），只消费
第一个 int32。版本 2 两者均从流读取。构造输入覆盖后继字节、两个
旧版本和末字节截断。旧尾段的实档及目标软件目视仍未验证。

## IScaleMarks 尾段

esriCarto 10.1 的 IScaleMarks（6DFAE8E7-ECAA-11D0-8682-0000F8751720）
声明六对 get/put。Map.dll 的 SteppedScaleLine 在完整对象 +0x278
装配虚表 0x7948e4d4；Save 0x79468360 调用 0x79455030 时传入同一子对象
（+0x268+0x10）。按 get 槽与实际读成员指令可还原整个共享尾段。

| 属性 | get 槽 | 原生入口 | 成员 |
|---|---:|---:|---:|
| DivisionMarkSymbol | 0x0c | 0x79454b60 | +4 |
| SubdivisionMarkSymbol | 0x14 | 0x79454cd0 | +8 |
| DivisionMarkHeight | 0x1c | 0x793c8820 | +0x10 |
| SubdivisionMarkHeight | 0x24 | 0x7943b220 | +0x18 |
| MarkPosition | 0x2c | 0x794570a0 | +0x20 |
| MarkFrequency | 0x34 | 0x79457130 | +0x24 |

版本 0/1 由标志生成的 0 或 5 属于 MarkFrequency。两个线符号、
中段填充符号和基类文字符号均缺少独立体长。填充比例尺遇到未知类或
嵌套体停止时保留此前完整字段，游标停在所属符号头，放弃该槽内新键；
后继刻度与地图范围没有可读起点。构造测试检查了这三类停止位置。

解析结果现可通过 `scale_mark_properties()` 读取四个已对位的 IScaleMarks
标量，并通过 `stepped_bar_height()` 读取阶梯比例尺高度。原始字段仍保留。
构造测试覆盖旧版频率默认值、当前版本的有符号标量、阶梯高度位模式与后继
字节边界。实档以及 ArcMap 10.1 和 QGIS 目视验证待补。

## IScaleBar.Division 的两处持久化值

复核同一 Map.dll 10.1.0.3035 及 10.1 esriCarto MSFT 类型库
（SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`）。
从 IScaleBar 的 TypeInfo 成员记录读得 Division 的 get/put 槽
`0x34` / `0x38`。在 SteppedScaleLine 虚表 `0x7948e2cc` 中分别落到
`0x7944f920` / `0x7944f960`。

以 objdump 追踪 getter。它先读接口相对 `+0xa8` 的 double，
再以 `ucomisd` 与地址 `0x7948ed60` 的零比较。恰等于零时改读
`+0x18`。由上文确认的位移差 `0x194` 得到两个 Save 成员
`−0xec` 和 `−0x17c`。前者从基类 v4 起出现，对应 Rust 的
`tail_double` / `base_tail_double`；后者是共同前缀后的首个 double，
对应 `base_double_a`。原生 put 将设置值写入 `+0x18` 与另一个
非持久化成员 `+0xb0`，并把 `+0xa8` 的非零值清零。
这是 10.1 静态证据。v5/v6 两个 short 的语义、覆盖值的运行时来源以及
ArcMap/QGIS 的实档目视验证待补。

比例尺线和填充比例尺的解析结果现提供 `division()`。基类初始化段
`0x7944b7d6–0x7944b7f5` 把完整对象 +0x180 的覆盖成员清零，
为 v0–v3 未保存覆盖值时提供回退依据。构造测试覆盖旧版、零覆盖、
非零覆盖、原始 NaN 位模式以及 v4 起因前置槽未解析而缺少覆盖值的情形。

## 填充比例尺高度

用同一 Map.dll 10.1.0.3035 的 PE RTTI 核对主虚表，并用 objdump
沿 IScaleBar.BarHeight 的 getter `0x79450370` 核对派生虚调用。
Hollow、SingleDivision、Alternating、DoubleAlternating 的主虚表
依次为 `0x79483624`、`0x7948ce54`、`0x79480ee4`、`0x79482444`。
四张表的 `+0xd8/+0xdc` 均为 `0x793a1320/0x793b29c0`，直接读写
完整对象 `+0x270` 的 double。

四类持久化虚表所在子对象偏移均为 `+0x268`；Load 实体
`0x793a1be0/0x79458830` 将该指针原样传给中段读取器
`0x793b3bf0/0x79459000`，两者均在版本之后读八字节至子对象 `+8`。
因此流中该 double 正是 BarHeight。Rust 保留原字段并提供
`bar_height()`。合成输入验证字段与游标边界，ArcMap 和 QGIS
实档目视验证仍待补。

## 分数字符开关

esriCarto 10.1 的 IScaleBar2 IID
`D1658F8E-BE62-4A78-88D3-07170F09120C` 位于 Map.dll `0x7948c384`。
QueryInterface 在 `0x7944bc91` 比较该 IID，`0x7944bcac` 返回
入口 this +0xdc。用 HollowScaleBar 的 RTTI 对位完整对象 +0xdc
的虚表 `0x79483904`。类型库声明 UseFractionCharacters 的 get/put
在 +0x98/+0x9c，对应指令入口 `0x79450520/0x79450550`，读写接口
相对 +0xd6 的 short，即完整对象 +0x1b2，对应基类 Save 相对 −0xba。
`0x7944df7c` 将该 short 按零和非零分支，非零时检查刻度值的分数部分。
Rust 的访问器在流中存在该字段时返回布尔值，旧版保留字段缺省状态。

基类 v5 增加的 short 在完整对象 +0x1b0，`0x793a1210/0x793a1230`
分别直接写读它。基类初始化段 `0x7944b820/0x7944b835` 清零两字段。
用 Map.dll 10.1.0.3035 的 PE32 虚表和 RTTI 复核，SHA-256 为
`1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`。
两个入口在下列七张完整对象虚表中分别位于 +0x98 和 +0x9c，
各表前一指针所指的 RTTI Complete Object Locator 的偏移均为 0。

| 类型 | 完整对象虚表 |
| --- | --- |
| AlternatingScaleBar | `0x79480ee4` |
| DoubleAlternatingScaleBar | `0x79482444` |
| HollowScaleBar | `0x79483624` |
| ScaleBar | `0x7948bf54` |
| ScaleLine | `0x7948c3dc` |
| SingleDivisionScaleBar | `0x7948ce54` |
| SteppedScaleLine | `0x7948e0b4` |

这两个入口使用 ECX 传递 this，与上文 IScaleBar2 子对象虚表中
同偏移的 COM 访问器不同。Map.dll 直接调用扫描未找到这两个地址的
`call` 或 `jmp`。间接调用者和 v5 字段的业务名仍待追踪。
ArcMap/QGIS 实档目视验证待补。

### 10.1 的读取和降版保存分支

共用 Load `0x79451120` 在 `0x7945139e–0x794513c2` 仅对
对象保存版本 ≥5 读取首个 short，随后独立检查版本 ≥6 再读取第二个。
这段加载代码在旧对象版本跳过对应读取，也不在该分支重置成员。

共用 Save `0x79451400` 取得文档版本后，把 `0x79480d28` 的
六项保存版本表 `[3, 3, 3, 3, 4, 6]` 交给
`AfStreamGetSaveVersion`。随后在 `0x79451647` 比较文档版本
索引与 5，索引较小时直接结束，达到 5 时依次写两个 short
（`0x7945166b`、`0x79451682`）。因此这张表只产生 v3、v4、
v6，读取器接受的仅有首个 short 的 v5 布局仍有独立的读取分支。
该结论来自 Map.dll 10.1.0.3035 的汇编与 AfCore 导入名，
尚未原生运行降版保存。

### 介质文档实测的 v5 short

从 10.1 和 10.8 安装介质中的 MXD/MXT 打开 `/Maps`，按根计数和
对象键表调用 Rust Map 后缀读取器，统计可到达的 MapSurround 比例尺
对象体。只记录已经从对象体读取的字段，跳过无法到达的对象。

| 介质 | 解析类型 | 基类版本 | 原始值 | 数量 |
| --- | --- | ---: | --- | ---: |
| 10.1 | DoubleAlternating | 6 | `0x0000` | 19 |
| 10.8 | DoubleAlternating | 6 | `0x0000` | 19 |
| 10.8 | ScaleLine | 6 | `0xffff` | 16 |

还读到基类 v3/v4 的 DoubleAlternating、Hollow、SteppedScaleLine，
这些对象体没有该字段。此次遍历没有读到基类版本恰为 v5 的对象。
`0xffff` 证明该字段在实档中可以为非零，字段的业务含义仍需调用者
或原生运行证据。10.8 介质的观察仅用于提供取证线索。本轮未在
ArcMap 或 QGIS 中人工验证。

## 比例尺线数字格式槽

Map.dll 10.1.0.3035 的 ScaleLine 和 SteppedScaleLine 均调用基类
Load `0x79451120`。基类 v2 起以 `0x7948c354` 的 INumberFormat IID
请求 LoadObject，Save `0x79451400` 写出相同槽。解析器现复用
[SystemCore 的九种已确认持久化体](arcmap-10.1-number-format.md)，并保留
NumericFormat、空槽和回引在解析结果中的原有表示。未知新类停在槽头并
撤销其键。构造的 SteppedScaleLine 验证 FractionFormat 后仍能读取刻度
和高度，并消费到对象边界。实际工程中这类槽及 ArcMap/QGIS 目视仍待验证。

## 比例尺文本和填充比例尺数字格式槽

同版 Map.dll 的四类填充比例尺复用 `0x79451120/0x79451400` 的
基类数字格式槽。ScaleText v2 起也请求 `0x7948c354` 的 INumberFormat
IID，位于前后缀字符串之后，v3 格式串之前，见
[原生字段顺序](arcmap-10.1-legend-family.md)。
这两处接入同一 SystemCore 九类分派。现有 NumericFormat 字段仍保存数字
格式本类、空槽和回引，其他已知新对象由 `other_number_format` 保存。
构造测试检查 FractionFormat 后继续读取后继字段，并验证 ScaleText
后继格式串截断时回滚游标和新登记。ArcMap/QGIS 实档目视验证待补。

## 独立 Scalebar 的接口边界

Map.dll 10.1.0.3035（摘要同上）的独立 Scalebar RTTI TypeDescriptor
`0x794b4c7c`，主虚表 `0x7948b5d4`，持久化视图 `0x7948b7ac`
位于完整对象 +0x1b4。核对既有反编译和 objdump，主 QI
`0x79445ee0` 直接处理 IMapSurround 与 IPersistStream 相关 IID，
其余由 `0x794400b0 → 0x7946b940 → 0x7946c2d0` 比较。
逐个用 PE 原字节解码比较的 IID，链中没有 IScaleBar
`6DFAE8E2-ECAA-11D0-8682-0000F8751720` 或 IScaleBar2
`D1658F8E-BE62-4A78-88D3-07170F09120C`，末端在
`0x7946c3c2` 返回 E_NOINTERFACE。不能借用现代比例尺的接口偏移为
独立 Scalebar 的字段定名。

构造器 `0x79445b60` 为完整对象 +0xf8/+0xfc/+0x100/+0x104
分别写入 10、0、0、0，对应最近补读的持久化偏移
−0xbc、−0xb8、−0xb4、−0xb0。默认值本身不足以证明业务语义，
读取器继续保留原始位模式。10.1 介质 132 条、10.8 介质 136 条及
参考工程两条 Maps/PageLayout 流均未出现独立 Scalebar CLSID 字节。
绘制或转换侧的字段用途与 ArcMap/QGIS 实档目视验证仍待取证。

## 共用基类 v5 标量的接口访问

在同一 Map.dll 10.1.0.3035 中，基类 QI `0x7944017f` 比较
`0x7948a580` 的 IID `F81C8A13-AF7F-4B72-8B0C-D8C5F1736776`，
匹配时于 `0x7944019d` 返回完整对象 +0x1c。该 IID 在 72 份
Desktop MSFT 类型库中未找到声明。HollowScaleBar 的 +0x1c
接口表 `0x79483774` 在 +0x24/+0x28 指向 `0x79441ce0/0x79441d10`，
分别委派到完整对象主表 `0x79483624` 的 +0x98/+0x9c，
即直接写读 v5 short 的 `0x793a1210/0x793a1230`。写入后还调用
主表 +0x50，参数为 1、2。

独立 Scalebar 的 +0x1c 接口表 `0x7948b6e4` 共享包装入口，
但其主表 `0x7948b5d4` 的 +0x9c 指向 `0x793cabe0`，该入口
恒返回零。这一路径不能用来解释独立 Scalebar 的持久化标量。
MapElements.dll 的 `0x7957c3b9/0x7957c3d4` 经 `0x7957d020`
查询该 IID，随后 `0x7957c41c/0x7957c448` 调该接口的 +0x18。
此处传入调用方暂存槽的地址，尚未确认涉及的具体动态类型和业务含义。
Rust 继续保留原始 short；实档和目标软件人工验证待补。

## 独立 Scalebar 的绘制入口

复核同版 Map.dll 的主 QI `0x79445ee0`。它比较
`0x79481a84` 的 IMapSurround IID
`FE3115E9-ED6C-11D0-8683-0000F8751720`，并返回对象 +0x18，
对应构造器 `0x79445bc7` 安装的虚表 `0x7948b6ac`。
esriCarto 类型库声明 Draw 位于 +0x30，与表中的
`0x79446020` 对位。该入口只检查两个参数指针并返回 S_OK
或 E_INVALIDARG，未访问实例字段或绘图对象。QueryBounds
则经 `0x79439e20` 进入共用实现 `0x79441130`；需要重算时，
`0x794411e7` 调主表 +0x44。由此可区分公开的 Draw 与共用
边界计算路径，Draw 本身无法为旧字段定名。
字段语义、其他绘制路径和 ArcMap/QGIS 目视仍待确认。

### MapSurroundFrame 调用点的参数类型

MapElements.dll PE32 版本资源为 10.1.0.3035，SHA-256 为
`67a6cbb2dfd1fe2626d32a8fbb1835b38e26b9543833b52628a85e7330d98768`。
`0x7957c6a0` 位于 RTTI 为 MapSurroundFrame 的 +0x10 接口视图，
虚表 `0x7959cea8` 的 +0x14 槽。该入口在
`0x7957c701` 和 `0x7957c791` 从传入对象的接口取出两个
接口指针，随后在 `0x7957c7a3` 进入上述私有 IID 的调用辅助函数。
MapElements.dll 的 IID 常量 `0x7959d3cc` 与 Map.dll 的
`0x7948a580` 的 16 字节相同。

`0x7957c380` 清零四个 DWORD，前两个存 QI 结果。两次
+0x18 调用分别接收后两个 DWORD 的地址（`0x7957c417`、
`0x7957c446`）。清理函数 `0x7957c460` 将这两个 DWORD
的值传给 +0x1c（`0x7957c496–0x7957c4b4`），并对非零值
通过虚表 +0x08 调用 Release。这组指令显示调用方按接口对象
保存和使用这些值。

同版 Map.dll 的 HollowScaleBar 在此接口的 +0x18 使用
`0x79441ba0` 创建集合并写入输出指针，+0x1c 的 `0x79441c40`
接收集合指针。这两槽与框架调用的参数形态一致。v5 short 的
写读方法在 +0x24/+0x28。另核对 `0x7944b835`，共用初始化段
将完整对象 +0x1b0 清零。

### MapElementsUI 对私有接口的调用

从 10.1 安装介质提取的 MapElementsUI.dll 为 PE32，版本资源为
10.1.0.3035，SHA-256 为
`77bdc1f73f368e5ee33ea09e119373013d9fbfb955d7af3cc19093991607e290`。
以 objdump 核对，`0x10015423` 传给 QueryInterface 的
`0x10063420` 与 Map.dll 的私有 IID 相同。调用方 `0x10014ef0`
在取得接口后，于 `0x10014f7c` 调用槽 +0x10 保存初始值，
在 `0x10014fa3` 向槽 +0x14 传入 −1。配对的 `0x10014fc0`
在 `0x10015018` 取出保存值的低 16 位，于 `0x10015028`
写回 +0x14，随后释放接口。

HollowScaleBar 的同一接口表 `0x79483774` 中，+0x10/+0x14
指向 `0x793a1070/0x793a1030`，读写接口相对 +0x24 的
short，即完整对象 +0x40。接口 +0x20 指向 `0x793a1050`，
另写完整对象 +0xc8 的 DWORD。比例尺 v5 字段经 +0x24/+0x28
访问，UI 的这对调用访问的是 +0x40 的 short。

### 填充比例尺基类的负版本边界

Map.dll 10.1.0.3035 的基类 Load `0x79451120` 经 objdump 复核。
`0x79451151 / 0x79451156` 按有符号上限 6 判断版本，
`0x794512a4 / 0x794512a9` 对小于 2 的版本读取旧 LONG，
后续 `0x79451327` 起的版本比较也采用有符号条件。
构造的 SingleDivisionScaleBar 验证负版本保留原始位模式、读取旧 LONG，
跳过后增字段，并正确读取派生段。v5 short 的业务语义与真实负版本工程仍待验证。

### 线式比例尺的有符号版本门

Map.dll 10.1.0.3035 的 ScaleLine、SteppedScaleLine 和 ScaleLineImpl
分别在 `0x79453825`、`0x79468305`、`0x79453f25` 比较短版本，
随后由有符号 `jg` 拒绝大于 1 的值。共用基类见上一节。
线段尾部 `0x79454f15 / 0x79454f19` 接受有符号值 ≤2，
并以 `0x79454f28 / 0x79454f33` 的有符号比较确定旧标志槽。
构造测试覆盖两类线式比例尺的负版本、旧标志推导和后继字节。
真实负版本工程及 ArcMap 人工验证仍待补充。

### 填充比例尺派生段的有符号版本门

Map.dll 10.1.0.3035 中，填充比例尺的派生 Load
`0x793a1be0 / 0x79458830` 与中段 Load `0x793b3bf0 / 0x79459000`
均用有符号上限 1。尾段共用线式比例尺的 `0x79454ee0`，
因此负版本进入旧标志分支。构造用例用负值覆盖三处版本，
验证原始位模式、旧标志推导值和对象体终点。真实宿主尚未验证。

### 独立 Scalebar 的边界计算派发

沿同一 Map.dll 10.1.0.3035 的 QueryBounds 继续查主表
`0x7948b5d4`。表项由 PE 字节解码，目标函数用已有 Ghidra 导出
与 objdump 核对。共用边界函数 `0x794402b0` 是主表 +0x44。
它对独立 Scalebar 的后续派发如下。

| 主表槽 | 入口 | 对该实例的行为 |
| --- | --- | --- |
| +0x2c | `0x79441030` | 取 +0x28 辅助对象给出的两个尺寸，转换单位并建立初始范围 |
| +0x6c | `0x79445b20` | 立即返回 |
| +0x74、+0x90 | `0x79453e10` | 立即返回，指令为 `ret 8` |
| +0x78 | `0x794407b0` | 以 +0xb0/+0xb4 的内部图形列表计算范围，空范围时回退到 +0x58 的缓存 |
| +0x8c | `0x793a1190` | 固定返回 false |
| +0x94 | `0x79441e10` | 按范围左下角及锚点计算位置，再派发主表 +0x60 |

这里的 +0xb0/+0xb4 是相对完整对象的内部列表位置，和
持久化接口相对偏移 −0xb0/−0xb4 代表的标量是不同的成员。
此分支尚未为独立 Scalebar 的持久化标量提供可确认的业务名称。
基类 v5 short 的名称也仍待找到实际调用者或原生运行证据。
ArcMap 和 QGIS 目视验收仍待补充。

### 独立 Scalebar 的字体回引

Map.dll 10.1.0.3035 的 `0x794460b0` 在持久化接口相对
−0x6c、−0x54、−0x2c 依次调用普通 LoadObject，请求
`0x79480c2c` 的 IFont IID `{BEF6E002-A874-101A-8BBA-00AA00300CAB}`。
SystemCore.dll 同版本的 `0x7ffd0063–0x7ffd007d` 对键命中的实例
查询接口，并在 `0x7ffd0326` 返回 S_OK。Rust 的三个字体槽
优先返回键的已登记类，缺少登记类时保留回引头类标识。构造体让
后两个槽用不同 CLSID 回引首个 StdFont，验证登记类和末尾标量。
其他字体实现的接口支持及真实工程的字体回引待验证。
未做 ArcMap 或 QGIS 人工验证。

### 共用基类 v5 short 的偏移交叉复核

同一 Map.dll 的 `0x79441cf9` 通过主表 +0x98 写入
short 后，`0x79441d06` 调主表 +0x50。上述七种比例尺的
+0x50 均为 `0x79440970`。该入口先判断对象 +0x99、+0x40、
+0x42 的状态，再记录或派发传入的变更参数。共用通知路径只能
说明该 setter 会通知更新，尚不能用它给 short 定名。

另一处比例尺计算 `0x7944c670` 确实读取 short，但其
`0x7944c6a6` 使用的是完整对象 +0x19c。例如调用点
`0x7944d17b` 直接传递对象地址，没有额外的 +0x14 调整。
该值会传给计算辅助函数 `0x7944b400`，不能解释 +0x1b0。
同 DLL 中 `0x793fc650`、`0x793fc670` 的 +0x1b0
读写入口属于 Map 的 +0x58 接口，PE RTTI 对位表
`0x7948734c`。搜索相同成员偏移时应按虚表和接收者区分。
比例尺 v5 short 的业务含义仍待找到实际使用方或运行证据。


### MapSurroundFrame 的私有槽用于比较对象集合

重查同摘要 MapElements.dll 10.1.0.3035。调用 `0x7957c380`
后，MapSurroundFrame 在 `0x7957c7ad` 将暂存结构交给
`0x7957c530`。该函数区分两个集合指针均空、仅一侧为空及
均非空三条路径。非空时在 `0x7957c59a/0x7957c5a9` 调集合
虚表 +0x0c 并比较返回的计数。计数相等时按同一索引在
`0x7957c5de/0x7957c600` 调两边集合虚表 +0x10，
再比较返回对象，并于 `0x7957c630/0x7957c646` 释放对象。
调用结果在 `0x7957c7b2` 决定是否进入框架更新路径。

因此这条调用链期望私有接口 +0x18 填写对象集合指针，并实际
解引用该指针。HollowScaleBar 的同一接口在 +0x18 返回集合，符合这条路径的参数形态。
v5 short 的业务语义仍待验证。
本轮只补静态证据，未执行 ArcMap/QGIS 人工验收。

### v5 成员偏移的其他候选排除

对摘要已核对的 Map.dll 10.1.0.3035 复查直接访问
`[寄存器+0x1b0]` 的 WORD 指令。除比例尺已定位的访问器和
初始化外，`0x793ecdd9`、`0x793ece08` 与
`0x793f340b` 属于另一条 Data View 路径。
`0x793ecda0` 和 `0x793ecdf0` 分别读取和写入
`0x79486268` 的 UTF-16 注册表值名 `ShowScrollBarsDataView`，
并缓存 short。后者在 `0x793ece20–0x793ece33` 从接收者
`+0xf4` 取出被委派对象，将 short 传给其虚表 `+0xa8`。
现有反编译导出 `Map/793f3270.c` 中相应 Data View 调用也从
`+0xf4` 取对象；objdump `0x793f340b–0x793f3422` 确认其将
缓存的 short 传给同一槽。比例尺已核实的 v5 setter 接收完整
比例尺对象，并写入其 `+0x1b0` 后通知变更。两条路径的接收者
和用途有别。

另核对 10.1 提取目录中含私有 IID
`F81C8A13-AF7F-4B72-8B0C-D8C5F1736776` 的模块。
LayerLib.dll（SHA-256
`55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）
在 `0x786c0159–0x786c0187` 查询并保存接口。
GraphicElementsUI.dll（SHA-256
`de8e0a6f1875012420cdd5b5ffc5ffb525881d2167c8644c69b9b4d15e14e42d`）
在 `0x1002a093–0x1002a0e3` 执行同类查询和保存。
这些引用只对位到查询辅助入口。比例尺 v5 short 的实际业务用途
仍待运行取证，ArcMap/QGIS 人工验收未进行。


### 用构造指令校准私有虚表首址

Map.dll 10.1.0.3035 的 HollowScaleBar 构造入口 `0x793c5ef0`
在 `0x793c5f58` 把虚表 `0x79483774` 写到完整对象 +0x1c，
与 QI `0x7944019d` 返回的接口地址对应。独立 Scalebar 在
`0x79445bce` 安装 `0x7948b6e4`。较早记录把两个表的首个
业务方法当作表首地址，导致各槽偏移少算三个 IUnknown 槽。
以上已按构造指令订正。v5 short 的加载和保存位置未受影响，
字段的业务语义仍待取证。

### 比例尺克隆和赋值的持久化路径

Map.dll 10.1.0.3035 的 HollowScaleBar 构造
`0x793c5f4a` 将 `0x7948371c` 安装到完整对象 +0x10，
对应 QI `0x7944bd47–0x7944bd65` 返回的 IClone。
虚表的 Clone/Assign 槽 +0x0c/+0x10 分别为
`0x79441600/0x79441670`。对照同版反编译导出
`Map/7946b9f0.c`、`Map/7946baf0.c`，Clone 调用前者，
经 CreateMemoryStream、SaveObject、Seek 和请求 IClone 的
LoadObject 构造副本。Assign 在同类分支调用后者，将源对象
写入内存流，核对类标识并跳过键后调用目标的持久化 Load。

两条路径因此都经过比例尺的版本化持久化代码。基类保存版本为
v6 时会随对象体写读 v5 short；旧文档索引分支省略此字段。
本轮核对了调用链，内存流初始化后的实际文档版本及 Clone/Assign
的运行结果仍待验证。该证据也未定性 v5 short 的业务用途。

#### 克隆内存流的默认保存版本

Map.dll `0x7946ba13–0x7946ba21` 和 `0x7946bb2c–0x7946bb38`
均以空内存句柄和标志 1 调用 AfCore::CreateMemoryStream。
同版 AfCore.dll（SHA-256
`031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`）
在 `0x68a7bb2a` 调 CreateStreamOnHGlobal，`0x68a7bb52`
创建 SystemCore 的 ObjectStream，`0x68a7bb68` 以
IObjectStream +0x38 绑定底层流。

SystemCore.dll 10.1.0.3035（SHA-256
`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）
在构造指令 `0x7ffcf16c` 将完整对象 +0x54 初始化为 5。
SetStream `0x7ffcf7a0` 只更新底层流引用。
IDocumentVersion getter `0x7ffd0630` 从接口相对 +0x40
返回该值。Clone 和 Assign 的这两条内存流路径在 SaveObject
之前没有设置其他版本。按静态调用链，比例尺基类 Save 使用
文档索引 5，写出 v6，携带 v5 short 和 v6 short。
ArcMap 实际运行及 v5 short 的业务用途仍待验证。

### v5 short 控制自定义锚点偏移

在已核对摘要的 LayerLib.dll 10.1.0.3035 中，导出
`AnchorPointCustomOffset(IGraphicsContainer*, IElement*)` 的入口为
`0x786bbb10`。它取得被包围对象，然后经 `0x786bbbd4` 调用
前述 `0x786c0120`，查询私有 IID `F81C8A13-AF7F-4B72-8B0C-D8C5F1736776`。
成功时 `0x786bbbf5` 调该接口的 `+0x28`，在 `0x786bbbf7`
按返回的 short 是否为零选择路径。零值跳过偏移计算。
非零值继续获得几何对象，在 `0x786bbd2b` 计算坐标差，
最后将两个 double 写入返回的 WKSPoint。

对具备已确认 v5 访问器的比例尺，`+0x28` 通过
`0x79441d10 → 0x793a1230` 读取完整对象 `+0x1b0`。
这将 v5 字段对位为是否计算自定义锚点偏移的开关。
线式和填充比例尺读取器保留原始 short，并提供按零值判断的访问器。
独立 Scalebar 的相同接口槽仍按自身虚表固定返回零。
构造测试覆盖字段缺省和非零值。ArcMap/QGIS 目视验收待验证。

### 独立 Scalebar 类工厂与固定尺寸

在同摘要 Map.dll 中，复核已有 `Map/793cb880.c` 和对应指令。
对 `79445b60` 构造器的直接调用位于同一工厂 `0x793cb880`
的聚合及非聚合路径。两者均分配 0x1bc 字节。非聚合路径在
`0x793cb99f` 查询请求接口并在查询失败时销毁对象，聚合路径
只接受 IUnknown 请求。这两条路径均未将旧字段转写为现代比例尺
成员。回查提取的 Desktop DLL 集合，独立 Scalebar 的原始 CLSID
仅在同一 Map.dll 命中，故跨模块追踪需继续沿接口调用进行。

独立 Scalebar 的完整对象主表 `0x7948b5d4` 在
`+0x1c`、`+0x20` 分别进入 `0x79445fe0`、`0x79446000`，
向输出参数固定写入 200 和 60。`0x79446040` 也只从
`0x794850c8/0x794850d0` 写出常量。从这些访问器无法反推
持久化 double 或前三个 LONG 的名称。接下来需要找到读取旧成员
的兼容路径或检查更早版本的原生实现。当前字段语义待定，
ArcMap/QGIS 人工验收尚未执行。
