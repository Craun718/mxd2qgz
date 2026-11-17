# MXD 对象流复核（2026-11-17）

范围为用户指定的本轮工作副本 TODO 行号，而非第四轮报告中的原行号；只判断指定的未勾选项，不改 TODO。10.1 原生 VA 只适用于所核对的 10.1.0.3035 文件；静态观察、介质样本和合成测试均不等于 ArcMap 运行验证。对象和路径内容不在本文输出。

## 文件身份、环境与依据

本轮对 `temp/arcgis101-bin/` 的 AfCore.dll 和 SystemCore.dll 重算 SHA-256，分别为 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`、`f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与 [10.1 基线](arcmap-10.1-native-baseline.md)和[第四轮](mxd-round4-version-object-20261117.md)的身份及 10.1.0.3035 版本记录一致；本轮哈希确认文件未变，不独立证明产品版本或发行签名。已复读保留的 Ghidra（经桥接工具） 12.1.4 导出 `temp/pseudocode/AfCore/functions/0x68a7af70.c`、`0x68a7b490.c`、`0x68a7b530.c`；反编译不是原始代码，其中 `0x68a7b530` 明示删除不可达块，不能据此证明完整接口传播。本轮未发起新 Ghidra 桥接工具 会话：`/tmp` 剩余约 211 MiB，既有[取证记录](mxd-open-streams-20261116.md)记录 Ghidra 曾因 `/tmp` ENOSPC 失败；没有删除共享文件或重复该资源密集调用。

仓库参考样本为 `samples/测试底图.mxd`。已提取的两版介质流分别位于 `temp/arcmap101-media-streams/` 和 `temp/arcmap108-media-streams/`；既有[介质样本清单](mxd-install-media-samples-20261117.md)记录 66 与 68 个 MXD/MXT、102 份不同内容，磁盘版本覆盖 8.1、9.2、9.3、10.0、10.1，且各样本的 `/Version` 不代表对应版本对象体已逐字段闭合。本轮确认两目录及其流文件存在，未重新全量分析样本，也未用字节扫描宣称发现对象边界或 v0 体。

本机有 `wine`、`virsh`、`VBoxManage`，`temp/mxd101-run/bin` 为空；`virsh -c qemu:///session list --all` 因运行目录只读无法连接，`VBoxManage list runningvms` 提示 `/dev/vboxdrv` 不存在。未发现可确认运行且授权可用的 ArcMap 10.1/10.8；这不是对所有可能远程或本机环境的穷举。未运行 ArcMap 或 QGIS。

## 逐行结果

| 工作副本行 | 证据与已有实现 | 可勾选性与剩余阻碍 |
| --- | --- | --- |
| 106 跨流对象体 | [对象体记录](arcmap-10.1-object-body.md)与[对象流续研](mxd-parse-agent-object-stream-20261117.md)指出 SetStream 复用对象表，命中键不读体，未命中先登记再交具体类 Load，通常无外部通用体长。 | **否**。机制不等于所有类及其完整体闭合；未知类不可猜长度；需逐类证据与真实对象体边界。现有 Map 等读取器及跨流接线与其他代理工作交叉，不直接修改。 |
| 107 跨流版本适用性 | 10.1 静态 SetStream 不清表；[第三轮](mxd-round3-version-object-20261117.md)及第四轮对 10.8 同机制有局部静态观察。 | **否**。局部机制相似不证明各类体及 8.x/9.x 等版本兼容；需要各版本原生实现与文件对照。 |
| 108 已勾选背景 | 10.1 SetStream 与登记、析构和接口证据见上述对象体记录。 | 已勾选，不属于本次未完成项；不代表 106、107 完成。 |
| 109 已勾选背景 | 10.8 VariantStream 定宽及字符串局部证据见 [VariantStream 记录](arcmap-10.1-variant-stream.md)的版本对照。 | 已勾选，不代表后续对象体与引用表完成。 |
| 110 VariantStream 对象体 | 10.1 的 VT 9/13 走 LoadObject；是否带键取决于传入流 QI 到 IObjectStream。已暂存的 `read_value_10_1` 支持标量及零 GUID 空对象，非零对象明确拒绝。 | **否**。具体对象类负责各自的体，非零类的完整体和真实文件字段归属尚缺；不能据空对象或合成数据外推。 |
| 111 VariantStream 引用表 | ObjectStream 的 IPersistVariant 分支创建包装流；[第四轮](mxd-round4-version-object-20261117.md)确认 LoadObject 的 QI 命中走 +0x44，而嵌套包装的 QI 传播仍需逐层证明。 | **否**。[另一份对象流续研](mxd-parse-agent-object-stream-20261117.md)主张嵌套包装可 QI 到同一表，但未给出足以独立核查全部包装 QueryInterface 槽及对象身份的链条；需汇编和运行/真实文件对照，也需明确 10.8 范围。 |
| 112 已勾选背景 | 已有显式 10.8 Variant 前缀及错误边界。 | 已勾选，不视为 10.1 非空对象或真实文档验证。 |
| 113 SimpleSaveObject 委派 | 第四轮确认 10.1 `SaveObject` 在 QI IObjectStream 成功时走 +0x40，否则走 `SimpleSaveObject`；`LoadObject` 对应 +0x44 或普通流。既有 Ghidra 桥接工具 `0x68a7af70` 显示普通及递归/包装路径；`0x68a7b530` 伪代码删除多个块。 | **否**。续研对 10.1 写入口与 QI 有更强结论，但包装递归各分支的参数、失败与读取对应关系尚未完整逐指令核对，10.8 亦未闭合；不能把写入口局部闭合升级为整个对象读写委派链完成。 |
| 120 其他版本字符串 | 10.1、显式 10.8 有局部字符串路径；介质包括更早磁盘版本。 | **否**。仅凭 `/Version` 和字符串扫描不能证明历史版本各字段编码；需各版本二进制、具体调用点及样本。 |
| 121 其他版本对象属性 | 10.1 类体有局部分支与对象版本门；介质包含旧版头部。 | **否**。对象字段按类和版本变化，历史磁盘版本不证明其各类属性规则；需逐版本逐类 Load/Save 与实档。 |
| 122 已勾选背景 | 10.1 键序与回引的局部静态证据和分类器已有记录。 | 已勾选，不代表重复登记运行场景已观察。 |
| 123 重复键替换 | 10.1 `0x7ffd0950` 有替换节点值分支；正常读取先查表；参见[第四轮](mxd-round4-version-object-20261117.md)。 | **否**。尚未触发可归因的原生重复登记并观察替换后回引，Rust 合成表操作不能替代所要求运行取证。 |
| 283 PropertySet v0 | SystemCore `PropertySet::Load 0x7ffd3080` 的版本上限比较和无其他版本分支，见 [PropertySet 记录](arcmap-10.1-property-set.md)；现有读取器已接受 v0，[既有测试](../tests/property_set_v0_independent.rs)覆盖非空与全部截断前缀。 | **否**。静态布局及合成边界已覆盖，但尚无结构级归属的真实 v0 旧档或运行佐证；Save 恒写 v1。不能将任意零字节扫描命中当作 v0 对象。 |

对于普通流零 GUID 的具体行为，[第四轮主审记录](mxd-round4-review-20261117.md)更正为零 GUID 返回空、非零才创建；[对象流续研](mxd-parse-agent-object-stream-20261117.md)开头仍残留与其自身后文相反的表述。应以指令级复核为准，不利用存在冲突的文字作进一步推断。本文不改其他代理的研究文档。

## 复核命令与测试

以下命令在仓库根目录执行；原始介质流、导出及诊断可能含敏感路径，不应未经脱敏分享：

```sh
sha256sum temp/arcgis101-bin/AfCore.dll.* temp/arcgis101-bin/SystemCore.dll.*
sed -n '95,123p;275,283p' TODO.md
rg -n 'SetStream|SimpleSaveObject|引用表|版本 0' docs/mxd-round4-version-object-20261117.md docs/arcmap-10.1-property-set.md docs/mxd-parse-agent-object-stream-20261117.md
cargo test --test property_set_v0_independent --test round4_version_object
df -h /tmp .
virsh -c qemu:///session list --all
VBoxManage list runningvms
```

本轮运行上述两个测试目标，分别 1 项与 2 项通过。Cargo 提示无法清理用户全局缓存（权限拒绝），不影响本次测试结果；未修改 Rust 代码，未重复全量 Rust 测试。ArcMap 10.1、10.8 与 QGIS 人工打开、保存、重开和目视核对均未进行。要闭合 123 和 283，分别需要可运行的原生重复登记场景及可结构定位的真实 v0 旧档或运行产物；其余项按上表补齐版本、对象和字段证据。
