# SDK Java 旧档的 TimeData 四段宿主复核（2026-11-18）

## 范围与结论

本轮针对 TODO 306 的 TimeData 族四段，使用已提取的 ArcMap 10.1 SDK Java 工程，以 `/Maps` 根、Map、图层列表到 FeatureLayer 的顺序宿主链复核。新增独立测试，不改 parser、模块声明、TODO 或 README。

SDK Java `portland.mxd` 的 `/Version` 局部读取为文本 `10.0`、short `18`；提取文件 SHA-256 为 `2002fc5ebf4f32098bccbd7d6e7c5cbdf9fe2c50d91c7947d9ee5765c802c60d`。它属于 10.1 SDK Java CAB（CAB 身份及摘要见 [介质旧档记录](mxd-terra-round8-old.md)），不据此推断该工程由 ArcMap 10.1 保存。

从 `/Maps` 的首个 Map 对象开始，读取器沿图层列表顺序确认其中 13 个 FeatureLayer 新对象及完整后继段（入口偏移：`0x2da, 0x1c450, 0x386ed, 0x3a7b6, 0x3b38c, 0x4d32a, 0x4e1ac, 0x6a00b, 0x6af1d, 0x6e402, 0x6f3ed, 0x700d9, 0x713af`）。这 13 个图层均为架构版本 34；每个后继段都完整读到 `TailCompleted`，四个 TimeData 段均存在、均为版本 1：

- TimeData：ITimeReference 槽为空标记；
- TimeTableDefinition：四个字符串为空，ITimeExtent 槽为空标记；
- TimeDimensionDefinition：两个字符串为空；
- TimeDataDisplay：版本 1，标量组完整读取，两个 int32 原值分别为 4 和 8。

因此，证据支持将 TODO 306 中「四段经真实旧档宿主完整到达」作为可独立闭合的范围。它**不**证明非空 TimeReference / TimeExtent 的体布局或嵌套引用，也不代表 ArcMap 运行验证；若原条目要求非空时间属性对象，仍应保持该更窄范围开放。未从文件提取或记录任何图层名称、数据路径或其他样本属性。

## 10.1 原生门与指令复核

实现目标为 ArcMap 10.1.0.3035。原生文件身份及静态函数位置沿用 [DEP 与 TimeData 研究](arcmap-10.1-dep-tail.md)：FeatureLayer.dll `FeatureLayer::Load` `0x6f6e9940`；LayerLib.dll 四个 Load 分别为 TimeData `0x7870d8b0`、TimeTableDefinition `0x7870df80`、TimeDimensionDefinition `0x7870e250`、TimeDataDisplay `0x7870f6c0`。字段顺序与各类版本界已在该记录中按 Load/Save 反编译列出，本轮不重做字段语义推定。

对本机提取的 FeatureLayer.dll 用 `objdump -d` 复核：`0x6f6e997e` 将 FeatureLayer 架构版本与 `0x22`（34）比较并拒绝更高值；`0x6f6eae6a` 比较版本与 `0x1f`，`jle 0x6f6eaf2e` 跳过四个时间段，较高版本顺序调用 `this-0x68/-0x80/-0x58/-0x48` 的四个接口槽（`0x6f6eae74..0x6f6eaec0`），随后在 `0x6f6eaecc` 比较版本与 `0x20`，较高版本继续到 HLEP 槽。层体架构版本 34 因而经过四段门。对 LayerLib `0x7870d8b0` 的指令复核也确认首 short 版本上限 2、随后 short 与对象槽，版本大于 1 才读尾 short。静态指令界和模块伪代码互相核对；地址均为 10.1.0.3035 静态映像 VA。未调用 REA。

## 测试与边界

新增 `tests/luna_timedata_host_20261118.rs`：先解析 `/Version`，再从真实 `/Maps` 根读取首个 Map；断言 FeatureLayer 对象类别、13 个宿主入口偏移、版本 34、四段版本及终点，并验证两个嵌套对象槽为空标记。测试没有扫描 GUID 来建立实例身份。

主工作树上的专属测试通过（1 项）。所有分析临时目录位于仓库 `temp/luna-timedata-20261118/`。

没有在 ArcMap 10.1 或 QGIS 打开工程，也未做人工目视验证。真实字节复核说明现有读取器能按静态 10.1 规则解释该旧档宿主链，不代表目标应用成功读取。


### 四个时间段的有符号版本门

复核 LayerLib.dll 10.1.0.3035（SHA-256
`55514f054c9e4827d773cf1a9b9a83e7c41c483f73758965244625165f330e02`）。
四个 Load 分别在 `0x7870d8e7`、`0x7870dfb4`、`0x7870e284`、
`0x7870f6f8` 使用有符号上限跳转。TimeData 在 `0x7870d92a` 仅当
有符号版本大于 1 时读取尾 short。构造连续四段验证负版本原始位模式、
TimeData 省略尾 short 及后继边界。尚无负版本实档或 ArcMap/QGIS 人工验证。
