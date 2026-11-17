# 第三轮：Version、ObjectStream 与 Map 属性（2026-11-17）

编号为 `mxd-round2-needs-20261117.md` 第三个表的原 ID；原有未提交内容是基线。证据为静态分析，非 ArcMap/QGIS 人工验收。本报告只涵盖分配范围，113 限三个 Map 成员的业务用途。

## 逐项账目

| ID | 状态 | 已证子链与仍缺证据 |
| --- | --- | --- |
| 71 | PARTIAL | 10.1 Version 文本、short 与条件 4 字节已由读写路径对位；后续额外尾字节的语义、消费者及非空实例未闭合。 |
| 74 | PARTIAL | 保存实现消费版本索引；SaveAs UI 到 `put_DocumentVersion` 的调用链未闭合。 |
| 75 | PARTIAL | 10.1 英文资源已核对；其他语言资源、查找回退和读取识别的影响尚缺证据。 |
| 88 | PARTIAL | 10.8 写入文本、short 和条件占位已有局部指令证据；回填值、完整写尾尚未闭合。 |
| 100 | PARTIAL | 10.1 SetStream 不清对象表，普通对象按类持久化；跨流引用对象体无通用长度，须按具体类闭合。 |
| 101 | PARTIAL | 跨流复用机制仅限已核对的 10.1 实现；其他版本须各自验证。 |
| 104 | PARTIAL | 10.1 Variant 9/13 走 LoadObject；空对象分支已有生产读取器。非空对象体由各类持久化实现决定，未普遍闭合。 |
| 105 | PARTIAL | 流是否支持 IObjectStream 决定键控或简化分支；嵌套包装中接口传播、引用表身份与回引尚需确认。 |
| 107 | PARTIAL | 10.1 SaveObject 按流 QI 委派至键控保存或 SimpleSaveObject；普通持久化对象路径已见，Variant 包装递归与 10.8 链尚未闭合。 |
| 113 | PARTIAL | Map Load/Save 已对位三个成员 +0x19c（short）、+0x200（double）、+0x264（short）；各自业务含义尚待静态调用者和接口槽确证。坐标变换和照明属性由主代理另行处理。 |
| 114 | PARTIAL | 10.1/10.8 已有局部字符串读写证据，不能推广到其他版本或全部字段；须核对对应版本与调用点。 |
| 115 | PARTIAL | 对象属性由具体类和版本门决定；须逐版本逐类验证，不能由某个版本的实现泛化。 |

现有依据见 [第二轮版本与对象流](mxd-round2-version-object-20261117.md)、[10.1 原生基线](arcmap-10.1-native-baseline.md)、[Variant 对象分支](arcmap-10.1-variant-propertyset-slots.md)、[Map 复核](mxd-round2-review-20261117.md)。
