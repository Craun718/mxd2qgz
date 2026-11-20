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
