# Map 后续字段与根循环边界（2026-11-17）

续接 [变换出口](mxd-terra-host-tail.md)，不重复既有字段。地址是静态 PE 虚拟地址；静态分析、构造数据和参考样本均不是 ArcMap 运行取证。

## 身份与证据

重算 `temp/arcgis101-bin/mapdll/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`：SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，PE machine `0x14c`，资源 FileVersion / ProductVersion `10.1.0.3035`、InternalName `Map`、OriginalFilename `Map.DLL`。这些是文件身份和版本声明，不是签名验证。`temp/r6-maps.bin` SHA-256 `fc0149f1cdbd4abb893610c0b442382ea0cc4c191c434173113db26b01aeacfe`。复用隔离 Ghidra 12.2 导出 `temp/terra-depth-maptail/0x793ff4d0.c`、`0x79401850.c` 和 `/tmp/r3/r4_map_funcs/f79403d50.c`；未修改共享目标。

## 本轮 Load/Save 对位

Map::Load `0x793ff4d0` 在版本 >22 的 IOverposterProperties 槽后：版本 >=24 依次读取 `+0x32a` short、`+0x320` double、`+0x328` short；版本 >=26 依次读取两个 IEnvelope、`+0x2b8` double、两个 IEnvelope、`+0x2d8` double；版本 >=27 再读 `+0x2a8/+0x2ac/+0x2b0/+0x2b4/+0x2c8/+0x2cc/+0x2d0/+0x2d4` 八个 4 字节成员。普通新版 Save `0x79401850` 依次写相同字段，四个对象分别源于 `+0x2a4/+0x2a0/+0x2c4/+0x2c0`。此对位不证明所有旧档写侧版本策略。字段业务语义尚未确定，double 保留原始位。

读取器逐字段提交；四槽支持空引用、Envelope 新对象和该类键回引，其他类或体截断回退槽起点并撤销该槽内登记。引用表只保存键，无法独立验证回引目标的类。参考样本从根连续读取：`0x252be..0x252ca` 为三个标量，`0x252ca`、`0x252da` 为空槽，`0x252ea` 为 double，`0x252f2`、`0x25340` 分别为 Envelope 新对象键 2182、2183，`0x2538e` 为 double，`0x25396..0x253b6` 为八个成员。

版本 >=28 下一调用是 Map `0x79403d50`：先调用 LayerLib 的 `LayerMasking::LoadLayerMasking`，成功后根据 Map 中图层数量循环调用 `0x79403bc0`。`0x79403bc0` 的非空接口分支再次调用同一 Load；写侧 `0x794031b0` 先保存 Map 自身，再按图层循环。不能由数据框图层条目个数直接断言每个接口均非空或按固定长度跳过。其后 Map Load 还有多个版本门、字符串、对象及计数循环，直到正常返回才可认为 Map 对象体结束；本轮没有接入根循环或声称 Map 完成。

该调用的导入宿主 `temp/agent-fl-tail/LayerLib.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A` SHA-256 `55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`，PE x86，FileVersion / ProductVersion `10.1.0.3035`，资源自报 InternalName `LayerUI`、OriginalFilename `LayerUI.DLL`，导出名明确包括 `LayerMasking::LoadLayerMasking` 和 `SaveLayerMasking`，地址分别 `0x786f1ff0`、`0x786f1a30`；不能将文件名当作资源名。独立 Ghidra 桥接工具 6.9.0 CLI 采用 JDK 25 和专用 TMPDIR，run `bb9c18e1-2127-4351-be75-5e5d66c9d4f9` 导入后 Unix socket 报 `Operation not permitted`，未返回反编译；随后改用隔离 pyghidra 和 `/opt/ghidra`，不切换共享目标。

隔离 pyghidra 反编译 `0x786f1ff0/0x786f1a30`：非空接口的 Load 依次读 short 版本（有符号大于 2 拒绝）、int32 外层计数、每项对象与 int32 内层计数、内层各对象（版本 >1 还各带 String），最后读 short；Save 对应写出。两份反编译均提示重启上限，控制流及接口副作用仍有不确定；此处仅实现版本 0..=2、外层计数 0 的安全子集，并逐字段保留原始值。负版本位模式、负数及超限计数比原生更严格地停止；非零计数在首项起点停止。参考样本 `0x253b6` 版本 2、`0x253b8` 计数 0、`0x253bc` short `0xffff`，在 `0x253be` 停于逐图层段，余 431 字节。不能将 `0x253be` 解释为 Map 或根循环尾部。

## 根循环安全边界

10.1 分流读侧 PageLayout `0x7b708d00` 和写侧 ArcMap `0x466fb0` 的既有证据见 [对象体记录](arcmap-10.1-object-body.md)：数量后逐个完整处理对象，循环结束后才读或写 4 字节尾值。因此根前缀读取器在正数量首对象头之后试读的 4 字节并非已确认根尾值，可能是对象体或后续对象的一部分。首对象为空时，`mxd_maps` 原先丢掉了试读的四字节；现从空头结束处完整保留未知数据。数量为零时才直接解释其后四字节为循环尾值。当前 API 仍保留历史 `tail_raw` 原始预读字段，正数量不解释；多 Map 和首项空引用的完整根循环仍未实现。

## 可勾范围与未闭合项

- 可单列勾选：10.1 Map Load 版本 24/26/27 这三组字段的静态顺序及普通新版 Save 对位；四个 IEnvelope 槽对空引用、已支持 Envelope 新对象和回引的读取与安全回退；Map 自身 LayerMasking 空集合前缀读取；首对象空引用后的未知字节保留修正。
- 不可整体勾选：完整 Map 后缀、LayerMasking、非 Envelope 类、业务语义、多 Map 根循环、根尾部实际位置、历史保存版本以及目标软件验收。若 TODO 现有项合并上述范围，保持开放；本轮未修改 TODO。

## 验证

新增 `tests/terra_nextpass_tail.rs`：覆盖字段所有截断点、两个样本 Envelope 对象每个截断点与引用表回退、未知类停止、四空槽、原始浮点位以及 LayerMasking 空集合、非空计数、非法版本和截断；既有样本断言同步至新停止点。测试和临时输出在仓库专用 TMPDIR 下执行，可能含本机路径。ArcMap 10.1 环境暂缺，ArcMap 和 QGIS 均未打开源工程或结果进行人工目视验收。

最终 `cargo fmt --check`、定向测试和 `git diff --check` 通过；完整 `cargo test --offline` 首次在运行时找不到共享构建目录中的一个测试可执行文件而中断，复跑通过（777 通过、11 默认忽略）。README 的 `cargo run --offline -- parse 'samples/测试底图.mxd'` 输出未知尾部 `0x253be` 起 431 字节。
