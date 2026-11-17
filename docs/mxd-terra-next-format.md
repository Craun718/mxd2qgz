# 数字格式与 Hyperlink 续查（2026-11-17）

范围：本轮分派的 TODO 当前行 236–238、271–275。未修改 TODO、公共入口或已有 `hyperlink.rs` 改动。没有 ArcMap/QGIS 人工验收，以下均非运行验证。

## 原生证据与更正

本机 `temp/arcgis101-bin/SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF` SHA-256 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与 [既有 10.1.0.3035 身份核对](arcmap-10.1-number-format.md)及 `temp/pseudocode/SystemCore/decompile-index.json` 的 target SHA 一致。复核 Ghidra（经桥接工具） 12.1.4 导出 `functions/0x7ffd11b0.c`、`0x7ffd6a10.c`、`0x7ffce4b0.c`、`0x7ffca1e0.c`、`0x7ffb33a0.c`，并用 objdump 对同一 PE 的 x86 指令作交叉核对（静态 VA）。

**更正旧记录的“同槽升级/覆盖”解释**：以各 Load 接收的第一个栈参数为 P。Percentage `0x7ffd11ea` 令 `esi=P-0xc`，把该地址交给 AfStreamRead 写 short，`0x7ffd120b–120d` 又将 esi 作为 NumericFormat Load 的第一个参数。Rate `0x7ffd6a47` 令 `edi=P-0x14`，将其作为 double 的写入地址，`0x7ffd6a6e–70` 又将 edi 作为 NumericFormat Load 的第一个参数。但是 NumericFormat `0x7ffce4e6–f1` 将收到的第一个参数再减 `0x1c` 才交给第一个字段的 AfStreamRead；其伪代码中后续字段偏移依次为 -0x18、-0x14、-0x10、-0xc、-0xa、-8。因而核心字段位于调用参数之前（分别为 P-0x28..P-0x13、P-0x30..P-0x1b），不覆盖外层 short 或 double。上界为包含式。这里确认的是对指令及反编译中调用参数和读入地址的静态算术，不推断对象类型或内存生命周期。流内两个字段仍是顺序出现，现有读取器保留两者，读取逻辑无需因此更改。原 [数字格式记录](arcmap-10.1-number-format.md) 的“同槽升级”和“覆盖”结论不应再引用，待主代理整合修订该共享文档及其他引用。

LatLon 调用传 `P-0x38`、Angle 传 `P-0x10`，见上述导出；这些只是传入指针，不能直接当成第一个核心字段的写入地址。各类布局见原研究，尚需真实对象或目标运行行为确认。Angle 的调用后无返回值检查属于静态观察；本轮没有制造原生兼容容错分支。

## 条目状态

| 行 | 状态 | 结果与剩余证据 |
| --- | --- | --- |
| 236 | 推进 | 核心成员实际写入范围得到更正；九类标量仍缺类型库成员与实现访问器逐槽绑定。 |
| 237 | 未闭合 | Rate/Custom 的 String 顺序有 10.1 读写证据，但尚无访问器绑定，不能用类名猜业务语义。 |
| 238 | 推进 | 静态证据否定 Percentage/Rate 所谓覆盖；LatLon/Angle 传入指针和不同版本门已复核，缺真实对象或运行行为。 |
| 271 | 阻塞 | Hyperlink v0 与 v1 静态门及构造测试具备，缺可从宿主顺序验证的真实 v0。 |
| 272 | 阻塞 | 通用空/回引和读取器已有构造测试，缺真实 Hyperlink 宿主循环的空或回引记录。 |
| 273 | 阻塞 | 九类短分支已有静态结构及构造测试，缺可确认边界的真实旧档或目标运行产物。 |
| 274 | 推进 | 同 238；传参指针相同不构成覆盖证据；无真实对象或运行验证。 |
| 275 | 推进 | 复核并运行现有核心版本非法且尾字段存在的构造测试，证明读取器严格拒绝并回滚；原生失败后的真实流位置与运行行为未验证。 |

先前介质筛查范围及局限见 [介质记录](mxd-dataset-format-evidence-20261117.md) 和 [第四轮记录](mxd-round4-names-format-20261117.md)；候选 GUID 的零命中不是所有文件或流均无实例的证明。Hyperlink 字段接口绑定见 [第二轮](mxd-round2-dataset-format-20261117.md)。

## 检查

`cargo fmt --check` 和完整 `cargo test` 均通过。定向运行 NumberFormat、Hyperlink 与旧 Legend 测试分别 19、6、2 项通过；并行工作中已有同一 Angle 失败构造测试，本轮不重复保留。构造输入并非 ArcMap 运行结果；保留已存在的 `src/objects/hyperlink.rs` 改动。
