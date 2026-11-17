# Terra 后续宿主复核（2026-11-17）

接续 [67 项基线](mxd-terra-host-review.md)。六个 GPT‑6.1 Terra 子代理分工，主审复核和接线；关闭 ALPC 集合自身字段的静态定性一项，余 66 项。

| 范围 | 主审复核 |
| --- | --- |
| [Map 后缀](mxd-terra-nextpass-tail.md) | 接入版本字段、四个 Envelope 槽及 Map 自身空 LayerMasking 集合。参考样本停在逐图层遮罩入口 `0x253be`，余 431 字节；仍非 Map 体或根循环终点。 |
| [独立表](mxd-terra-nextpass-table.md) | 静态 QI 确认两个名称类；接入压缩序列、PreJoin、时间族等已确认部分及 Map 的逐项循环。仅整表读成才继续下一表；全部完成才继续照明槽。构造宿主验证完整表与回引、未知子对象停点和截断回滚。 |
| [比例尺线](mxd-terra-nextpass-scale.md) | 两类的实际原生持久化入口已定位，接入 ScaleLine 与 SteppedScaleLine；构造两个连续条目后能读到正确定界的显示范围。未定标量和真实宿主仍开放。 |
| [标注字段](mxd-terra-nextpass-expression.md) | LELP 的 15 个字段完成部分静态对位；ALPC 自身的 ID 和计数器静态定性。空项不读取 ID；新增构造回归覆盖紧随空项的回引与截断事务。 |
| [旧 FDO](mxd-terra-nextpass-fdo.md) | 更正旧加载器的槽接收者、多个虚表位移及字段 17/18 的独立调用关系；各成员业务作用与旧档仍待证。 |
| [VariantStream](mxd-terra-nextpass-prop.md) | 确认 PropertySet 候选的宿主和类方法值序列化范围；非空 VariantStream 对象及会话上下文证据不足，未扩展写法或读取。 |

ArcMap/QGIS 人工验收未做。按原生证据、真实参考流和构造测试分别记录，不将它们互相替代。

主审复跑 `cargo fmt --check`、`cargo test --offline`、`git diff --check`：777 项通过，11 项默认忽略。CLI 参考样本仍显示六条图层记录，未知 Map 尾部从 `0x253be` 保留 431 字节。
