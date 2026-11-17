# PropertySet VariantStream 对象体与会话引用

日期：2026-11-18。范围：在 PropertySet 的 Variant 值中区分普通 IStream 与已确认的 ObjectStream；使用 ArcMap 10.1 原生静态分析，只分派有证据的对象体。尚未运行 ArcMap，也未进行 ArcMap/QGIS 人工验收。

## 10.1 文件身份

研究对象是 `binaries/manifest.json` 中列出的 AfCore.dll（SHA-256 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`）和 SystemCore.dll（SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`），两者文件身份及 10.1.0.3035 版本基线见 `docs/arcmap-10.1-variant-propertyset-slots.md`。地址均为静态映像 VA。反编译 C 是 Ghidra 伪代码，不是原始源码。

## 调用流与引用上下文

1. SystemCore `PropertySet::Load` 实体 `0x7ffd3080` 读取版本、条目计数，再循环读取键及值。它在 `0x7ffd3f00` 调 `Variant::Read(local_variant, stream, true)`；该调用点没有替换或包装传入的流。可复核：`/home/gxrs/文档/source/arcmap10.1/SystemCore/7ffd3080.c`、`7ffd3f00.c`。
2. SystemCore IAT 将 `Variant::Read(IStream*, bool)` 解析到 AfCore 导出。AfCore `Variant::Read` `0x68a7c6a0` 的 VARTYPE 9/13 分支（约 `0x68a7ca4d`）调用 `LoadObject` `0x68a7b6d0`。可复核：`/home/gxrs/文档/source/arcmap10.1/AfCore/functions/0x68a7c6a0.c`、`0x68a7b6d0.c`；SystemCore IAT 关系也记录于 `docs/arcmap-10.1-variant-propertyset-slots.md` 第 1 节。
3. AfCore `LoadObject` `0x68a7b6d0` 先调 `0x68a7c4b0` 查询传入 IStream 的 `IObjectStream`。普通流未命中时 `0x68a7b490` 读 16 字节 GUID，非零才 `AfCreateInstance`，不读键；ObjectStream 命中时转到该接口的 `LoadObject` 槽（vtable `+0x44`），由 SystemCore 对象流读取 GUID+u32 键并查读表。可复核：AfCore `0x68a7b6d0.c`、`0x68a7b490.c`，SystemCore `/home/gxrs/文档/source/arcmap10.1/SystemCore/7ffcff70.c` 及 `docs/arcmap-10.1-object-body.md` 的对象流读取分支。
4. SystemCore 对象流在新键未命中时先登记，再通过对象的 `IPersistStream::Load(ObjectStream)` 读取体；已见键仅作为回引，不消费体。`SetStream` `0x7ffcf7a0` 只替换当前底层流、不清读表，因此同一 ObjectStream 会话跨流共享键。依据：`docs/arcmap-10.1-object-body.md`、`docs/mxd-open-streams-20261116.md` 与相应 SystemCore 伪代码。

所以 PropertySet::Load 的 Variant 值必须依传入流的实际接口上下文解释。VARTYPE 9/13 本身不能证明对象头有键；原有 `read_property_set_10_1` 保留普通 IStream 的保守契约。新增 `read_property_set_object_stream_10_1` 由上层显式调用，仅适用于已经确认的 ObjectStream，并通过 `read_variant_object_branch_object_stream_10_1` 复用 VARTYPE、对象头和共享键分类逻辑。

## 有证据的对象体分派

选择 PropertySet 自身作为已闭合的新对象体，不按类名推定：

- `esri.clsid.xml` 将 `{588E5A11-D09B-11D1-AA7C-00C04FA33A15}` 标为 SystemCore `PropertySet`；原生 `GetClassID` `0x7ffd3040` 写出相同 GUID。
- SystemCore IPersistStream 虚表 `0x80071840` 的 Load 槽 `+0x14` 经 thunk `0x7ffd5560` 到实体 `0x7ffd3080`。该实体的对象体规则已独立复核并由 `read_property_set_10_1` 实现：版本、计数、重复的键名+Variant。
- ObjectStream 新对象头中只有类 GUID 与键都已读取，且通过 CLSID 精确匹配时，专用 API 才递归消费 PropertySet 体。成功后返回嵌套结果并保留同一键表；回引只返回键，不消费体。空 GUID 是 Null 标记，不带键。
- 其他非空 CLSID 明确拒绝，不尝试跳过或猜体长。新对象头先登记这一行为与原生顺序一致；Rust 解析失败时由 trial 游标和 trial 引用表一起回滚。
- 递归深度上限只在确认为新 PropertySet 对象后检查；因此在上限处仍可读取空值或已登记键的回引。

主审将新 API 接到 `WorkspaceName` 中经 ObjectStream 头和 CLSID 核验的新 PropertySet 对象。普通 IStream 入口仍独立；其他调用方须先确认自身持有 ObjectStream 上下文。对非 PropertySet 对象类的分派、原生运行行为及真实 MXD 中嵌套 PropertySet 的出现频率仍未验证。

## 变更与测试范围

- `src/objects/property_set.rs`：保留现有普通流函数和类型；增加 ObjectStream 专用返回类型/API，仅对 PropertySet CLSID 递归解析。
- `tests/property_set.rs`：覆盖新对象体解析、跨独立 Reader 共享键回引、未知类失败时游标与引用表回滚、普通流不误读会话键，以及既有普通流测试。
- `src/objects/workspace_name.rs`：复用同一引用表解析新 PropertySet；FeatureLayer 的已有测试同时核对内联标量仍能顺序读到后续字段。所有测试是构造输入测试，不是 ArcMap 运行验证。
