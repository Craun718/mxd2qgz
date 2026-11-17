# TableQueryName / FeatureQueryName 集成取证（2026-11-17）

范围为标准 SystemCore ObjectStream 中这两类的静态分派和严格读取子集；字段布局及原始类身份链以[前次研究](mxd-terra-follow-names.md)为基础。未使用实机 ArcMap、未验证实际 MXD 宿主或转换结果。

## 原件与新证据

重新计算 GdbCore SHA-256 为 `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4`，SystemCore 为 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`，与既有 10.1.0.3035 PE 版本及身份核对一致；哈希本身不认证签名。此次重新读取两 PE 的 x86 原始指令和 GUID 数据；对独占副本用 pyghidra / Ghidra 12.2 和 JDK 25 定点反编译 GdbCore `0x708f4960`、`0x708f5700`、`0x70a60800`，结果在忽略目录 `temp/terra-integrate-query/probe.json`，不得不经脱敏就分享。上轮独立 Ghidra 桥接工具 CLI 的 Unix socket 失败记录见[前次研究](mxd-terra-follow-names.md)；本轮隔离进程未改变共享服务器状态。反编译是静态分析，不是运行证据。

## 新闭合的分派路径

SystemCore `0x7ffd023b..0x7ffd0269` 对新对象先经 `0x7ffcefc0 → 0x800112a0` 查询 `0x80070fa0`（IPersistStream），再经 `0x7ffd0be0` 查询 `0x8007127c`（IPersistVariant，内存字节 `51 d6 a4 67 e0 69 d2 11 85 00 00 00 f8 75 b9 c6`）。第二个查询有结果时，`0x7ffd026f..0x7ffd02a5` 包装流并调用该接口 +0x10；无结果时跳到普通接口 +0x14。因此**两者同时支持时，实际 Load 优先 Variant**，不能只根据第一次查询认定优先 Stream。

GdbCore Feature QI `0x708f5700` 先比较 `0x70ab8594`，不相等转 Table QI `0x708f4960`；后者比较 `0x70ab84e4`、`0x70ab8574`、`0x70a8840c`、`0x70a883fc`、`0x70aba854`、`0x70aba864`、`0x70aba834`，不相等委托 `0x70a60800`。后者只比较 `0x70a8841c`、`0x70a8846c`、`0x70ac8cd0`，其他 IID 在 `0x70a608ea` 置空并返回 E_NOINTERFACE。本轮读取这些地址上的原始 16 字节，均不同于上述 IPersistVariant IID；Feature 新增项亦不同。两个类的 IPersistStream IID `0x70a8840c` 命中并返回对象 +0x20。结合标准新对象路径可静态推断这两类走 IPersistStream Load。该结论以正常构造的具体对象及标准 ObjectStream 为边界，不等于证明其他包装、聚合或运行环境的所有行为。

## 读取器与边界

`dataset_name_more` 现在按两类 CLSID 分派：版本 0/1/2、三个前置字符串、workspace 子槽、三个后置字符串；版本 2 追加字符串和 16 位值，Feature 追加字符串及两个保留原始位模式的 32 位值。负版本按仓库严格口径拒绝，比原生有符号比较严格。Workspace 的空槽、已登记键和现有 WorkspaceName 读取器沿用既有约定；未知新对象在槽起点回退，不登记该键，不读取尾字段。上下文缺失的键无法从流内区分新对象与回引，所以也不能把未知键认作合法回引。嵌套 WorkspaceName 内部停止点向外传播。错误使用试探游标和暂存登记表回滚；停止是显式部分成功。公共入口调用者仍须提供完整 ObjectStream 会话引用表；不模拟 COM QI、setter、数据源打开及原生短读契约。

可独立勾选：10.1 静态确认这两类 QI 后备与 SystemCore 读取优先级；两类 IPersistStream 安全子集实现及合成边界测试。不可据此勾选真实宿主、实机互操作、业务字段语义、所有工作空间子类、或原 TODO 中所称的 IPersistVariant 体。ArcMap/QGIS 人工验收未进行。
