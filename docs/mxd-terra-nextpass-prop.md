# Variant 对象分支：PropertySet 宿主与具体类边界（2026-11-17）

本轮选择有明确持久化证据的 **PropertySet `{588E5A11-D09B-11D1-AA7C-00C04FA33A15}`**，追踪 WorkspaceName 内的 PropertySet 宿主和其中的值；不修改 TODO。已有类体、对象流及两族 Variant 的详细分析分别见 [PropertySet](arcmap-10.1-property-set.md)、[序列化器与槽位修正](arcmap-10.1-variant-propertyset-slots.md)、[标准包装数据流](mxd-terra-follow-stream.md)，此处只记录本次判断所需的连接与缺口。

## 身份和验证方法

对仓库忽略目录中提取物重新计算 SHA-256，PE 版本资源的 FileVersion、ProductVersion 均为 `10.1.0.3035`，OriginalFilename 与模块名相符；哈希与既有基线一致。地址为这些 PE 的静态映像 VA，不代表运行地址或签名认证。

| 模块 | SHA-256 | ImageBase |
| --- | --- | --- |
| AfCore.dll | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` | `0x68a70000` |
| SystemCore.dll | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` | `0x7ffb0000` |
| GdbCore.dll | `ba5d8f5d2a729a414bc11735a90454aca04cfda980538c96d9d69301c39ca1c4` | `0x708d0000` |

重读原生证据和仓库实现，另用 `objdump` 核对 SystemCore `0x7ffd3f3a..0x7ffd3f47` 与 `0x7ffd3fc6..0x7ffd3fd3`：两处压入参数 1 后分别通过 IAT `0x8006b2e0`、`0x8006b2e8` 调用；导入身份见上述槽位修正记录。此前独立 REA CLI 因 Unix socket 权限失败的具体记录见[查询名称续查](mxd-terra-follow-names.md)。本轮使用 JDK 25、`GHIDRA_INSTALL_DIR=/opt/ghidra`、仓库独占临时目录及复制的工程尝试隔离 pyghidra：首次未找到目标函数，重新分析未完成而中止，**没有取得可用新反编译证据**。未改变共享 REA 目标；临时输出可能含本机路径，分享前须脱敏。

## 本轮判断

1. `WorkspaceName::Load` 的 IPropertySet **对象槽**有类身份和持久化证据；PropertySet 的 `GetClassID`、IPersistStream Load/Save 与体布局已有明确定位，参考样本该槽观察到实例。这只证明它可在该宿主的对象槽出现，**不证明它也作为某个 VT 9/13 的值出现**。
2. PropertySet 体的值由 `Variant::Read/Write(stream, true)` 处理；上述两条 IAT 指令再次确认包装调用方向。它是类方法族，不是 `IVariantStream::Read` 的全局族。两者 9/13 同调 `LoadObject` 是局部共同点，并不能把已确认的 PropertySet 值宿主改说成已确认的 VariantStream 宿主。当前参考样本两实例观察的是字符串值，没有可据以绑定非空对象的值。
3. 标准 ObjectStream 创建 `IPersistVariant` 包装后，`VariantStream::Read` 把保存的底流交给全局 Read，VT 9/13 经底流的 `LoadObject` 可能回到同一会话对象表；普通流则不能假定有键。具体上下文、回引和对象身份不能从 VARTYPE 或非零 CLSID 单独判断。`variant_object_branch` 现在没有上下文与会话表参数，`property_set` 的值调用也没有传入表；直接按 CLSID 接上 `read_property_set_10_1` 会在带键场景把键当作对象体开头，也无法正确处理回引和递归。该体读取器目前对于非空嵌套对象仍停止，不能声称完成任意 PropertySet 对象体。
4. 另一个已有候选 TableQueryName/FeatureQueryName 已由 [10.1 QI 与分派链](mxd-terra-integrate-query.md)纠正为标准路径的 IPersistStream，不能用作已确认的 IPersistVariant 宿主。PropertySetProperty 是内联条目持有者，也不能替代实际流对象。

## 可勾边界与下一步

本轮能单独记账的范围是：**10.1 PropertySet 具体类、WorkspaceName 对象槽与 PropertySet 值序列化器的归属边界已复核**；这些不证明非空 VariantStream 对象正向读取。TODO 当前“VariantStream 对象分支的对象体”和“引用表”两项均不可勾；PropertySet v0 的真实旧档验证亦不因本轮改变。没有修改 `variant_object_branch` 或其他 Rust 文件。

下一步应先锁定具体 10.1 类的 `IPersistVariant` QI、Load/Save 和实际宿主，或证明某个已知值字段的非空具体类及其流上下文；追踪新对象和回引在完整会话表中的身份与登记、类体结束位置和错误回滚。再设计显式流上下文、资源限额及递归处理，覆盖 VT 9/13、新对象、回引、未知类、截断、错误后游标与表、后继字段边界等可构造测试；合成测试不能代替原生证据。

本轮 `cargo test variant_object_branch`：6 项通过；其余测试被过滤，未执行全量测试。Cargo 尝试清理全局缓存时收到权限警告，未影响测试。未运行 ArcMap 或 QGIS，未进行人工验收；静态推论与样本观察不等于目标软件验证。
