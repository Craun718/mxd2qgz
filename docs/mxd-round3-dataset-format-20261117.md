# 第三轮：数据集与数字格式（2026-11-17）

编号为 `mxd-round2-needs-20261117.md` 第三个表第一列的原 ID。本轮没有执行 ArcMap 或 QGIS，静态定位不代表运行验证。没有修改变体对象分支。

## 文件身份和方法

重新计算提取原件 SHA-256，并用 PE 版本资源核对 FileVersion 均为 10.1.0.3035：GdbCore `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`；DisplayCore `9b3e61f9ca553f14a442338dfc1c24ac51c1eea6dd9a4509282466a6a926e4d2`；SystemCore `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。地址为目标 DLL 的 PE 虚拟地址。原 Ghidra 桥接工具 伪代码证据、类型库来源和既有样本范围分别见 [数据集槽](arcmap-10.1-dataset-name-slots.md)、[变体流](mxd-table-query-name-variant-20261116.md)、[数字格式](arcmap-10.1-number-format.md) 与 [数据源 IID](mxd-dataset-format-evidence-20261117.md)。

本轮独立启动 Ghidra 桥接工具 6.8.0 CLI，不切换共享 MCP 目标。定点反编译 `0x708f4ab0` 返回 `provider_unavailable`：当前 `/opt/ghidra` 是 12.2，CLI 指定支持 12.1.x；诊断的 Java 21 也低于要求的 JDK 25（机器另有 JDK 25）。未更改全局配置或其他代理的会话。原始错误保存在 `temp/round3-dataset/`。以下 ObjectClassName 的新证据来自原 DLL 的 PE 字节和 objdump 指令，而非声称本轮新取得 Ghidra 桥接工具 伪代码；需用兼容版本的独立 Ghidra 桥接工具 会话进一步复核。

## ObjectClassName 实现定位

1. `0x70ac319c` 的 16 字节为 ObjectClassName CLSID `{905CEC64-0B2C-11D4-A060-00C04F6BDD84}`。初始化指令 `0x70a808c5` 将其地址写入注册对象 `0x70b2a648`，随后的 `0x70a808cf` 写入 UTF-16 名称 `ObjectClassName` 地址 `0x70abdb64`，`0x70a808e5` 将新分配的工厂对象虚表设为 `0x70ac2208`。
2. 虚表 `[-1]` 指向 COL `0x70ae32f0`，其 TypeDescriptor `0x70b2f294` 为 `.?AV?$ClassFactory@VFdbTableName@@@@`。虚表 `+0x0c` 指向 `0x7090a240`；该函数分配 `0x80` 字节，在 `0x7090a2a4` 调用 `0x709b9330`。另一个条件分支仍须逐条检查，不能据此声称全部工厂行为已闭合。
3. `0x709b9330` 安装主虚表 `0x70ad06f0` 等；该虚表的 COL `0x70af5728` 指向 TypeDescriptor `0x70b34bb4`，名称为 `.?AVFdbTableName@@`。故至少上述 ObjectClassName 工厂创建路径实际进入 GdbCore 的 FdbTableName 实现。`0x709b9480` 调用 `0x709b94b0` 并按标志释放，后者是析构而非旧研究中所称构造入口。

这解释了独立搜索 `ObjectClassName` 类 RTTI 的失败，且修正了早期把 `0x70a808cb` 称为数据表引用的判断：它位于初始化指令的立即数内部。该类的全部接口、聚合方式、GetIDType 及持久化路径尚未由此闭合；不能把未找到 IPersistStream 写入等同于不支持任何持久化形式。源码注释已纠正，读取器仍在未知体处停止。

## 原编号状态

| ID | 状态 | 证据和缺口 |
| --- | --- | --- |
| 178 | PARTIAL | 已有 Ghidra 桥接工具 证据 `ev_692562cbf1e7d3f7180662b9c83de903e10293297ab9531dd5970cd2e49162d8` 覆盖 TableQueryName 候选 `0x708f5070`/`0x708f5280`；版本、字符串和对象调用的局部次序见既有变体流记录。虚表归属、写侧 `Write_exref`、间接调用和 FeatureQueryName 完整读写链未闭合；需兼容 Ghidra（经桥接工具） 的独立会话补伪代码。不能实现猜测布局。 |
| 179 | CLOSED | ObjectClassName 的注册、工厂 RTTI、构造和实际 FdbTableName 类型链见上；本项只关闭实现位置定位，不关闭其持久化或运行行为。 |
| 218 | PARTIAL | GdbCore `0x709b7c90` 的四 String、标志和 Workspace 槽已有 Load/Save 静态布局及介质局部链；仍缺属性访问器与对象偏移的匹配，不能定名字段。 |
| 219 | PARTIAL | GdbCore `0x7095e5c0` 的两 String 和 IDatasetName 槽已有静态布局；仍缺访问器与各字段的绑定及实际拓扑对象。 |
| 230 | PARTIAL | 九类短体的标量布局及版本门已有 SystemCore 静态证据和合成测试；尚未完成逐类类型库属性、访问器及持久化成员偏移的交叉绑定。 |
| 231 | PARTIAL | RateFormat 和 CustomNumberFormat 等字符串读写入口已有静态证据；其业务属性名称尚未完成访问器绑定，不按类型或名字推测。 |
| 240 | PARTIAL | 既有类型库证据已将 ITemplate `{7914E5F4-C892-11D0-8BB6-080009EE4E41}`、ILineDecoration `{533D88F0-0A1A-11D2-B27F-0000F878229E}` 等具体 IID 定名；未枚举并交叉验证所有其余未定名引用，不能宣称穷尽。 |
| 241 | PARTIAL | 类型库与已定位 LoadObject 槽对应：`{4BA33331-D55F-11D1-8882-0000F877762D}` IName；`{2E060684-E14C-44D3-858A-442AB6A0A863}` IQueryDescription；`{FDFEBD93-ED75-11D0-9A95-080009EC734B}` IQueryFilter；`{F7AD47D1-D55E-11D1-8882-0000F877762D}` ISelectionSet；`{4434D680-F4DA-11D1-BCA6-0000F875BCCE}` ISpatialReference。源码中相互矛盾的“未定名”注释已更新；六类标量与字符串仍须逐槽对照访问器，接口名不代表具体持久化类。 |

本轮只修正源码注释，没有改变解析行为或新增猜测性的解析器。未知嵌套体仍按现有规则停止；目标软件运行、人工检查及实际转换验收均未进行。

## 检查

`cargo test` 通过，依赖本机介质的测试按既有设置忽略。`cargo fmt --check` 未通过，报告并行工作中的 `tests/round3_line_page_layout.rs` 格式差异；本轮未修改该文件。`git diff --check -- src/objects/dataset_name_more.rs` 通过。Cargo 另报告无法清理全局缓存的权限警告，不影响本次测试结果。
