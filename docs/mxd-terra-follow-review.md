# Terra 定点续查复核（2026-11-17）

接续 [75 项逐项复核](mxd-terra-next-review.md)，六个 GPT‑6.1 Terra 子代理并行处理目前能取得新证据的路径。本轮关闭 NumericFormat 核心七个属性对位这一独立子范围；完整「九类标量」及其他未满足条件的条目仍开放，现为 74 项。

| 方向 | 复核结果与下一步 |
| --- | --- |
| [对象流](mxd-terra-follow-stream.md) | 标准 VariantStream 经保存的底流调用全局读写后使用原引用表；包装自身未转发 IObjectStream QI。更正早期 IUnknown 误标。还缺非空实际对象、其他包装及递归错误路径。 |
| [GdbCore 查询名称](mxd-terra-follow-names.md) | 两类 QI 和次级虚表确认标准 IPersistStream 归属，四次写调用确认为 String::Write。继续验证 ObjectStream 选择的实际接口、流上下文与嵌套槽，再接入读取器。 |
| [FeatureLayer 包络槽](mxd-terra-follow-feature.md) | 两槽在 LoadObject 前释放已有引用并清零。后者的两级接口查询和参数已定位，仍缺正式接口方法归因和运行时生命周期。 |
| [历史介质](mxd-terra-follow-media.md) | 逐文件进入 134 个首 Map。10.8 介质中磁盘版本为 9.3 的 18 份模板，读到 368 个完整 FieldInfo 和 18 个 Map 后缀 Envelope，均为 v3；未取得要求的低版本体。不能将未到达的对象视为不存在。 |
| [数字格式与文本符号](mxd-terra-follow-display.md) | NumericFormat 七个核心字段均与类型库属性和访问器对位；代码注释保留原始值契约。仍缺其余派生类、字符串与 TextSymbol 的成员对位。 |
| [版本保存链](mxd-terra-follow-version.md) | 10.1 指定对话框分支接到 AfDoc put；10.8 回填点和后继独立流得到局部确认。完整命令分发、旧档打开后的索引和跨版本行为未验证。 |

主审同步更正了旧对象流、查询名称和 FeatureLayer 笔记中的归因。已通过 `cargo fmt --check`、`cargo test --offline`（727 通过，10 默认忽略）、`git diff --check`。另显式运行六个测试目标内依赖本机介质的 9 个用例，均通过。测试与介质分析未运行 ArcMap。用户确认目前没有可用 ArcMap 环境；[实机取证步骤](mxd-arcmap-runtime-capture.md)可在环境就绪后执行。
