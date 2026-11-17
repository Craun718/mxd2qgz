# FeatureIDSet 与编辑模板宿主的静态接口对位（2026-11-17）

本轮接续 [Terra 标注续研](mxd-terra-next-label.md) 和 [FeatureIDSet 前次取证](mxd-round3-labels-featurelayer-20261117.md)。本记录的地址是 ArcMap 10.1 静态映像 VA；类型库、反编译和构造测试均不代表 ArcMap 的运行行为，也不代替 QGIS 人工验收。

## 身份和方法

重新计算介质提取的 FeatureLayer.dll SHA-256 为 `2baf870615bb75b6f30ccea173ba4303fdd0b1a6571e11b9197c5fcf7298bb60`；PE Machine `0x14c`、FileVersion `10.1.0.3035`、InternalName `FeatureLayer`、OriginalFilename `FeatureLayer.DLL`。esriCarto 嵌入类型库 SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`、类型库版本 10.1。摘要确认字节身份，版本资源是文件声明，均不是签名或运行证明。

同一 DLL 的独立 REA 6.9.0 CLI / Ghidra 12.2 在[既有独立取证](mxd-terra-chain-fl.md)中因桥接 Unix socket `Operation not permitted` 失败，未产生函数 Evidence；本轮未改共享 REA，使用 JDK 25、Ghidra 12.2 / pyghidra，在仓库 `temp/terra-host-label/` 复制工程并重新反编译 Load、Save、QI 和访问器，以 PE 字节及 objdump 复核。JVM 临时目录、缓存和工程均位于该目录。类型库在复制的独立 Wine 前缀下通过 `LoadTypeLibEx`、`GetFuncDesc`、`GetNames` 枚举；沙箱内 Wine socket 被拒绝，随后在授权环境中完成枚举。这不是运行 ArcMap。64 位 Wine 输出的 `oVft` 除以 2 对应 x86 目标字节偏移。临时输出可能含本机路径，分享前须脱敏。

## FeatureIDSet

接口 IID `{D5BB4B87-E0A1-11D2-9F4D-00C04F6BC78E}`，RTTI 和虚表 `0x6f7ac338` 的接口子对象位于完整对象 +0x10，持久化接口位于 +0x14。类型库按次序声明 Contains、Count、Add、Remove、Clear、Members；对应函数分别是 `0x6f6d9640`、`0x6f6d9680`、`0x6f6d96b0`、`0x6f6d96d0`、`0x6f6d96f0`、`0x6f6d9710`。Contains 和 Add 的 ID 参数为 I4；Members 构造 `VT_ARRAY|VT_I4`，空集合可返回空数组指针。

| 持久化字段 | 原生读写及接口对位 | 边界 |
| --- | --- | --- |
| int16 版本 | Load `0x6f6d97d0` 有符号大于 1 拒绝；Save `0x6f6d9880` 写 1 | Rust 另拒绝负版本 |
| int32 计数 | Load 以正数为循环次数；Save 读取持久化 this+8；Count `0x6f6d9680` 读取接口 this+0xc，同为完整对象 +0x1c | 输入计数是流项数；重复插入后的集合大小不能仅凭输入计数保证 |
| 逐项 int32 | Load 将每项交给 `0x6f6d9940`；Add 以接口 this+8、Load 以持久化 this+4 指向同一完整对象 +0x18 集合；Save 遍历该集合写 int32 | 属于接口集合输入成员；不能据此证明均是数据源 OID。Rust 保留流顺序、负数和重复项，不模拟原生集合去重或遍历顺序 |

参考样本该对象 count=0；尚缺非空受控实例、实际数据源对应及运行观察。流项 `ids` 是原始持久化输入，不应直接等同于运行时集合的完整语义。

## EditTemplateManager（ETM）

QI `0x6f6d0810` 把 `IEditTemplateManager` IID `{000F7E24-8470-40C7-B0C1-8D2E8C33A619}` 映射到完整对象 +0x10，虚表 `0x6f7abb10`；IPersistStream 映射 +0x18，虚表 `0x6f7abb44`。因而相同成员的接口偏移必须先换算完整对象偏移。Count `0x6f7177f0` 读取接口 this+0x18，EditTemplate `0x6f6d0910` 根据接口 this+0x14 的数组及 +0x18 的数量取对象，两者分别对位 Save `0x6f6d0bc0` 的持久化 this+0xc、+0x10。Load `0x6f6d0a80` 读取版本、流计数后逐项请求 `IEditTemplate` 并调用另一个子对象的添加方法。

版本 ≥2 的末字节在 Load/Save 均为持久化 this+0x1c，即完整对象 +0x34。另一个 QI IID `{D06CB85E-F42C-4B1C-9807-B796A5CCF069}` 映射完整对象 +0x14：其虚表 `0x6f7abb28` 的 `0x6f6d0960` 增加、`0x6f6d09a0` 移除模板时置接口 this+0x20 为 1，`0x6f6d0a10` 将该字节的零/非零转换为 0/`0xffff` 返回。这证明它是与增删相关的持久化布尔标志；该 IID 未在本次 esriCarto 类型库枚举中找到，正式接口及属性名、其他置位或清除条件仍未确认，不能仅据此断言为完整 dirty 状态。版本 ≤1 无尾字节；不能将 Rust 的 `None` 当作原生对象的默认值。参考样本尾字节为 `0x01`，早期少数记录的 `0x00` 不适用于该样本。

## EditTemplate

QI `0x6f6cc350` 把 `IEditTemplate` IID `{0952F6C1-5E9D-4E2E-BC99-ED55BCC4C6CE}` 映射到完整对象 +0x10，接口虚表 `0x6f7ab874`；IPersistStream 位于 +0x14，虚表 `0x6f7ab8c0`。以下偏移以各自接口 this 为基准：

| 流内次序 | Load `0x6f6ce1e0` / Save `0x6f6ce3c0` | 类型库及访问器 | 结论 |
| --- | --- | --- | --- |
| int16 版本 | 栈局部量；有符号大于 2 拒绝 | 非公开属性 | 格式版本；Rust 另拒绝负版本 |
| 第一 String | 持久化 this+0x08 | Name getter `0x6f6cc420`，接口 this+0x0c | Name |
| 第二 String | 持久化 this+0x0c | Description getter `0x6f6cc450`、setter `0x6f6cc480`，接口 this+0x10 | Description |
| 16 字节 | 持久化 this+0x18 | Tool getter `0x6f6cc740`、setter `0x6f6cc770`，接口 this+0x1c | Tool 的原始 GUID；此前数据源标识猜测应撤回 |
| int32 与 [String, Variant] 循环 | 持久化 this+0x2c 容器、Save this+0x30 数量 | DefaultValue `0x6f6cc7b0` 遍历接口 this+0x30 的同一容器；SetDefaultValue `0x6f6cc910`、ClearDefaultValues `0x6f6cc8f0` | 字段默认值表的持久化内容；重复键行为和对象型 Variant 仍未闭合 |
| 版本 ≥2 的 String | 持久化 this+0x10；Load 汇编 `0x6f6ce39c` 显示实际地址，反编译的间接调用未还原参数 | Tags getter `0x6f6ce150`、setter `0x6f6ce180`，接口 this+0x14 | Tags；此前几何类型提示猜测应撤回 |

类型库还列出 Layer、SetDefaultValues 和 CreateCount；这些是接口成员，不应凭其存在推断本持久化序列另有对应流字段。参考样本中的字符串和 GUID 只能辅助定位，不证明各属性的运行含义或用户界面显示。

## 实现与可勾边界

`src/layers/edit_template.rs` 为既有字段增加 Description、Tool GUID 原始字节、Tags 的只读命名访问器；不改字段布局或破坏旧调用。`src/layers/annotate_props.rs` 增加 ETM 标志零/非零解释，原字节保留；缺失和提前停止返回 `None`。`src/layers/feature_id_set.rs` 明确区分流项与运行时集合。`tests/terra_host_label.rs` 覆盖版本门、非规范非零标志、原始 GUID、重复与负 ID 及截断回滚，均为构造输入。

可单独闭合的静态子项：EditTemplate 的 Name、Description、Tool、Tags 和默认值容器与持久化读写的接口对位；ETM Count、EditTemplate 数组和持久化增删相关标志的成员对位；FeatureIDSet 流整数与公开集合的静态对位。它们不等同于更宽泛的运行验收或 OID 身份条目闭合。ETM 标志正式名称、完整生命周期和旧版本对象状态、FeatureIDSet 实际数据源 OID 对应和非空行为，仍需更完整接口来源或受控运行；不修改 TODO。

## 验证

`cargo fmt --check` 和 `git diff --check` 通过。`cargo test --test terra_host_label --test legend_tails --test feature_layer_tail --test round3_labels_sample` 共 99 项通过；其中参考样本测试仅验证文件解析，不是目标软件运行。全量 `cargo test` 在库测试 `objects::color_ramp::tests::multi_part_old_version_defaults_weights_without_consuming_tail` 失败（断言期望 `0xa5`，实际读取 `0x00`），当时该范围外 `src/objects/color_ramp.rs` 有未提交修改，本轮未改动它；因此全量测试未通过。

ArcMap 环境暂缺，本轮未在 ArcMap 或 QGIS 打开工程，未验证图层、显示与转换结果；构造测试及静态证据不能替代目标软件运行和人工检查。
