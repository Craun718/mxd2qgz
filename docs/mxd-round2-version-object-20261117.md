# 第二轮：版本与对象流（2026-11-17）

原行号以本轮分配为准。不改 TODO；未运行 ArcMap/QGIS。10.1 AfCore、SystemCore 文件本轮 SHA-256 分别为 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`、`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`；10.8 ArcMap.exe 为 `a9a963b9dffbf0cb346c67c98bce0880b7e64363410a788dccb9597cff3c75fe`，与既有身份一致。

## 10.8 Version 读侧新复核

对 `/tmp/ArcMap.exe.E97001B0_1E76_4BE8_B51F_269775FE0A80` 使用 `objdump -d -Mintel --start-address=0x47a700 --stop-address=0x47aa12`：`0x47a770` 打开 Version；`0x47a91c` 对 short 执行 MOVSX，`0x47a923` 比较 25、`0x47a926` JLE；`0x47a99a` 再 MOVSX，`0x47a9a1` 比较 12，`0x47a9a4` JL 跳过，`0x47a9b4` 压入长度 4。读入口属于 `0x47a250`，写入口为 `0x48a380`；[前次续查](mxd-version-luna-followup.md) 将读侧误归后者，本记录纠正。写入口 `0x48ab74` 将索引传给版本表函数、`0x48ab9e` 写二字节、`0x48ac31` 对索引 >=1 进入占位分支、`0x48aca1` 写四字节。这些指令不能确定最终回填值及流尾。

`src/version/version_stream.rs` 新增显式 `read_arcmap_header_10_8`，消费已证条件四字节并原样保留，其他尾部不消费；大于 25 拒绝，失败回滚。单元测试覆盖 -1、11、12、25、26 的有符号边界、尾部及全部截断。入口未接线到版本识别，不宣称支持 10.8 保存或互操作，也不模拟原生短读。

## 原行逐项与最小余项

ARC：须目标软件运行才能完成的验证；SAMPLE：缺真实文件，可用所述受控方法生成；STATIC：可续静态研究。多种路径并列不意味着必须同时具备才可继续。

| 原行 | 状态与余项 |
| --- | --- |
| 71 | 部分：两介质 134 项已知头后均空。STATIC：追踪 10.1 尾部消费者；SAMPLE：非空尾真实文件或不同保存路径生成的文件。 |
| 73 | 未闭合。ARC：10.1 保存并重开受控工程，比较 Version 字节和软件显示。 |
| 74 | 部分：保存路径收到索引。STATIC：追踪 SaveAs 命令、对话框和 put 虚槽；必要时 ARC 调试。 |
| 75 | 未闭合。STATIC：本地化资源及识别代码；ARC：至少两语言环境生成并打开同一工程。 |
| 76 | 未闭合。STATIC：追踪打开旧文档后版本属性赋值；ARC：在 10.1 观察打开前后属性，旧文件可用介质模板。 |
| 77 | 未闭合。ARC：10.1 各保存选项生成、重开文件并核对 short。 |
| 78 | 未闭合：索引是候选推导。ARC：观察原生属性索引并比较输出。 |
| 88 | 部分：本轮新增读侧及写侧指令证据。STATIC：追踪 `0x48a380` 后续 Seek/Write 回填来源和末尾；SAMPLE：10.8 自身保存文件可交叉验证。 |
| 89 | 未闭合：介质磁盘版本仅至 10.1。ARC：10.1 与 10.8 分别打开、另存、重开对方可生成版本；SAMPLE：各版本受控生成。 |
| 100 | 部分：10.1 `0x7ffcff70` 新键登记后调用类 Load，`0x7ffcf7a0` 切流不清表；对象体无通用长度。STATIC：逐类闭合 Load/Save；SAMPLE：含跨流引用的真实工程或受控保存。 |
| 101 | 部分：机制限核验的 10.1 实现。STATIC：逐版本复核 SetStream、表及键控路径；类体另行核对。 |
| 104 | 部分：10.1 Variant 9/13 经 LoadObject，空 GUID 子集已实现。STATIC：追具体 IPersistVariant 类体；SAMPLE：含非空对象工程，可通过 ArcObjects 构造属性并保存。 |
| 105 | 部分：入口是否进入键表取决于流 QI；包装接口传播及实例未闭合。STATIC：逐指令追包装器 QI 和底层身份；SAMPLE：嵌套对象与回引工程，或 ARC 跟踪 QI 与键序。 |
| 107 | 部分：10.1 `SaveObject 0x68a7b640` QI 命中走键控写槽，未命中进入 `SimpleSaveObject 0x68a7af70`。STATIC：闭合 Variant 包装递归及 10.8 对应链。 |
| 113 | 部分：PropertySet 键和值等字段已有归属；主审另接入 Map 书签后前导，见 [复核记录](mxd-round2-review-20261117.md)。STATIC：逐宿主及类追 Load/Save 和接口槽；SAMPLE：非空属性受控工程用于对位。 |
| 114 | 部分：10.1/10.8 局部 String 规则非全版本全字段通用。STATIC：历史版本 AfCore 和调用点；SAMPLE：对应版本的非 ASCII、空字符串文档，可由相应版本生成。 |
| 115 | 部分：对象属性依具体类与版本门变化。STATIC：逐版本逐类核对 Load/Save；SAMPLE：相应版本及类的真实或受控工程。 |
| 117 | 部分：10.1 `0x7ffd0950` 有替换分支，常规读路径先查表；Rust 重复登记不证明原生触发。STATIC：寻找可达调用者；ARC：驱动原生分支，观察替换及回引。 |
| 277 | 部分：10.1 PropertySet Load `0x7ffd3080` v0/v1 同路径，Save `0x7ffd3280` 写 v1；已有非空 v0 和截断合成测试。SAMPLE：真实 v0 流，可用具备旧版写出能力的软件或受控持久化生成；10.1 当前 Save 恒写 v1。 |

合成测试验证 Rust 游标和输入边界，不替代原生运行或应用内人工检查。
