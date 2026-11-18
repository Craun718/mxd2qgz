# SimpleSaveObject 委派链复核（2026-11-18）

> 历史记录。这里的 GEN 接口名、IPersistVariant 与 VT 分支的归属、普通流读侧和 TODO 状态已被[后续原始指令复核](mxd-simple-object-dispatch-20261118.md)取代。

## 结论

ArcMap 10.1.0.3035 的**静态保存委派骨架**可由 AfCore 与 SystemCore 原生指令闭合到分支级；原 TODO「SimpleSaveObject 委派链」仍不能作为“真实宿主/读写全链已完成”勾选。闭合的是入口选择、接口 IID、可见递归参数及对象流写出端的职责边界。没有闭合的是参考 MXD 中某个具体非空对象实际命中哪条分支、所有 IPersistVariant 类的真实宿主和体边界，以及 AfCore 读包装器的完整传播与失败语义。未运行 ArcMap，符合本轮不使用 ArcMap 的约束。

简化后的保存分派是：

```text
AfCore SaveObject(stream)
  ├─ QI IObjectStream 命中 → ObjectStream::WriteObject (+0x40)
  │    └─ SystemCore 写 CLSID/键；新对象时委派类体
  └─ 未命中 → SimpleSaveObject(obj, stream, clearDirty, writeHeader=1)
       ├─ 对象声明 INoPersistInArcGISDocument → 直接返回 0x32
       ├─ 流支持 IDocumentVersion 且对象有 IDocumentVersionSupport
       │    └─ 版本可保存则继续；否则取替代值，按其 Variant 类型递归/写空对象
       ├─ 对象有 AfCore IVariantStreamIO → 按该接口产生 VARIANT，再递归处理对象/空对象
       ├─ 对象有 IPersistVariant → GetIDType/类标识，再经 VariantStream 适配后写 VARIANT 体
       └─ 对象有 IPersistStream → 可选写 16 字节 CLSID，再对原流调用 Save
```

图中“支持/取替代”与具体接口槽位含义来自 AfCore 私有 IID 复核；“参考 MXD 中确实存在并触发”不由静态分派代码推出。SystemCore 的 `+0x40` 路径和 AfCore 的普通流回退是互斥入口，不能把 SimpleSaveObject 内任一 QI 或递归调用称作 ObjectStream 槽本身。

## 二进制身份与可复现步骤

研究对象为安装介质提取的 10.1.0.3035 x86 映像。按 `binaries/manifest.json` 与本轮 `sha256sum` 复核：AfCore SHA-256 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`；SystemCore SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。哈希确认与研究基线文件字节相同，不代表签名或运行验证。

仓库外反编译资料 `/home/gxrs/文档/source/arcmap10.1/AfCore/functions/0x68a7af70.c`、`0x68a7b640.c`、`0x68a7b530.c` 和 `SystemCore/7ffcf7e0.c` 用于函数定位；AfCore 伪代码来自 Ghidra 12.2，不是原始源码。指令与 IID 用原 DLL 复核，可在仓库根目录重现：

```sh
sha256sum /home/gxrs/文档/source/arcmap10.1/binaries/{AfCore,SystemCore}.dll.017F52BF_20D0_487A_A583_436DA6406CFF
objdump -D -Mintel --start-address=0x68a7af70 --stop-address=0x68a7b480 /home/gxrs/文档/source/arcmap10.1/binaries/AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF
objdump -D -Mintel --start-address=0x68a7c0c0 --stop-address=0x68a7c3f5 /home/gxrs/文档/source/arcmap10.1/binaries/AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF
objdump -s -j .rdata --start-address=0x68c6dc78 --stop-address=0x68c6dce8 /home/gxrs/文档/source/arcmap10.1/binaries/AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF
```

已有完整 `SimpleSaveObject` 指令转储也保存在 `temp/luna-20261118/pyghidra/simple-save.dis`；该文件头标注的临时二进制副本与上列介质提取映像哈希相同。地址均为静态映像 VA。

## AfCore 写侧调用与状态

### 入口选择与空对象

- AfCore `SaveObject 0x68a7b640` 先以输入流查询 IID `{18A45BA7-1266-11D1-86AD-0000F8751720}`（IObjectStream）。指令 `0x68a7b688..0x68a7b695`：接口非空则调用其虚表 `+0x40`，以待存对象和查询接口作为实参；接口为空则 `0x68a7b697..0x68a7b6a4` 调 `SimpleSaveObject(obj, stream, clearDirty, 1)`。AfCore QI 包装器是 `0x68a7c4b0`。
- `SimpleSaveObject 0x68a7af70` 对空对象直接向传入流写 16 个零字节并返回；该处不写 4 字节 ObjectStream 键。空对象分支的 16 字节写入量由 `0x68a7afc3..0x68a7afed` 的指令确认。
- 对非空对象，`writeHeader`（第四参数）决定是否写类标识/对象头；普通 IPersistStream 路径只在该标记非零时写 16 字节类 GUID，然后把**同一流指针**传到对象 `Save`。这一普通流分支自身不生成 ObjectStream 键。

### 特殊对象及版本兼容

- `0x68a7c240` 固定查询 IID `{4BF42172-9DCE-4E03-A974-6C161F94293A}`。RTTI 复核名称为 `INoPersistInArcGISDocument`。SimpleSaveObject 在文档版本适配检查前查询对象；命中后释放接口并返回正值 `0x32`（`0x68a7b036..0x68a7b06e`）。这个状态被当前调用者如何解释、具体 MXD 是否含命中对象，本轮未观察。
- `0x68a7c1d0` 查询流上的 `{ECC43C55-0148-4EC1-BF87-B9A183C5DC98}`（IDocumentVersion）。只有查询成功才继续版本支持检查。
- `0x68a7c0f0` 查询对象上的 `{C9E7A782-3DDC-411F-8A66-908C579F123D}`。AfCore 对 `+0x0c` 方法返回的 short 作判断；为 0 时再调用 `+0x10` 取得替代值并递归保存。RTTI 及虚表交叉核对将该接口定名 `IDocumentVersionSupport`。递归固定令 `writeHeader=1`，流与第三参数沿用；替代值为 VT_EMPTY/VT_NULL 类状态时走空对象写出，否则传返回对象继续处理。分支指令位于 `0x68a7b099..0x68a7b0e8`。
- `IDocumentVersionSupport` 的已确认具体实现宿主至少包括 AnnoLayer.dll `FDOGraphicsSublayer`，对象接口偏移 `+0x44`；其 `0x6b1a1d60/+0x10` 两方法实现与 AfCore 调用槽吻合。它是**实现类证据**，不是参考 MXD 对象实例或实际命中该替代路径的证据。替代值按 Variant 类型分流，VT=13 递归保存其对象，VT=9 先处理对象值后走空对象递归，其他类型也走空对象递归；不据此给替代 VARIANT 赋业务含义。依据见 [私有接口 RTTI 记录](mxd-terra-round10-iid.md)。

### AfCore Variant 接口与两种持久化

- `0x68a7c160` 固定查询 IID `{81DA75BB-FE8D-2241-A9C2-86F07C539AD5}`。目前现有 10.1 资料只够把它作为 AfCore 内部 Variant/流适配接口记录；本轮没有找到可独立核实的 RTTI 名称，因此不补造接口名。调用点检验其 `+0x0c` 输出，再用 `+0x10` 取得 VARIANT；VT=13 递归保存其对象，VT=9 对象值先经另一辅助查询处理后递归，其他类型递归空对象。分支在 `0x68a7b141..0x68a7b24a`，各递归调用仍强制头标记为 1。该机制是对象实现的 AfCore 值适配，不等同于 COM `IPersistVariant`。
- `0x68a7c320` 查询标准 IID `{67A4D651-69E0-11D2-8500-0000F875B9C6}`（IPersistVariant）；`0x68a7c390` 查询 `{00000109-0000-0000-C000-000000000046}`（IPersistStream）。两者均为显式 QI，普通 IPersistStream 是回退/独立实现分支。
- IPersistVariant 路径先取得 ID 类型/类 GUID，再通过 VariantStream 适配向对象接口提供 Variant 值；指令显示对结果按 VT=13、VT=9 和其他类型分别递归到 `SimpleSaveObject`，同时在需要时写类标识。新 VariantStream 为 0x1c 字节对象，构造调用 `0x68a7a2c0`，传入原流；错误分支清理暂存接口/Variant/流，并在可见创建或查询失败处返回错误。IPersistStream 路径则按可选头后直接调用 `Save`。其字段写入由各类自身实现。
- 已确认的 IPersistVariant 类候选包括 GdbCore 的 FeatureQueryName/TableQueryName 家族；对具体主接口及 `GetIDType`/`IPersistStream` 的既有 QI 研究说明不同实现可能各自选择持久化接口。尚未把这些类绑定到一个实际 MXD 字段、非空 Variant 值或确定的 ObjectStream/普通流调用上下文；见 [VariantStream 宿主记录](mxd-terra-round8-variant.md)。因此这里不将类候选说成“真实 MXD 宿主”。

## SystemCore 接收端与对象体归属

SystemCore ObjectStream 主接口虚表 `0x800710dc` 的 `+0x40` 为 `0x7ffcf7e0`。该写出器对输入对象作持久化接口选择：Variant 持久化值经过 ObjectStream/VariantStream 包装；普通持久化对象转入对象自己的 Save。新对象路径由 ObjectStream 写 16 字节类 GUID 与 4 字节引用键并登记对象，再委派类体；已有对象键走引用路径，不重复写类体。空对象写零 GUID 标记。写对象体没有通用长度字段，结束位置由具体类 Save 决定。对应读入口 `0x7ffcff70` 读 GUID+键、登记新实例后调用该类的 Load；已见键回引不读体。写/读端职责及无通用体长度见 [SystemCore 对象流基线](arcmap-10.1-object-body.md)。

这个对应关系说明了两种不同层次：`SaveObject → IObjectStream +0x40` 是**对象头和对象表协议入口**；回退到 `SimpleSaveObject` 后的 `IPersistStream::Save` 是**对象体写入调用**。SimpleSaveObject 中为版本兼容、Variant 适配而发生的递归，是 AfCore 层内部转换，不经过 AfCore 导出 `SaveObject` 再次选择 IObjectStream 槽；因此不能从递归本身推出对象重新登记新键。

## 读侧、实际宿主与未闭合边界

AfCore `LoadObject 0x68a7b6d0` 对流查询同一 IObjectStream IID，命中则调用接口 `+0x44` 到 SystemCore `0x7ffcff70`，未命中才转普通对象加载。AfCore `VariantStream::Read 0x68a7bc50` 将包装对象保存的底流交给全局 Variant Read；VT 9/13 进入 LoadObject。AfCore 的 `0x68a7b530` 本身是短 wrapper，反编译表示它从上下文取对象并调用虚表 `+0x14`；这不足以单独标注具体读取类、读回写侧的每个替代/变体状态，现有 Ghidra 伪代码还报告删除不可达块。读侧的包装和接口传播尚未由本轮逐分支的原始指令重建。

“真实宿主”拆为三个层次以免混称：

1. **接口实现宿主已知**：例如 FDOGraphicsSublayer 实现 `IDocumentVersionSupport`；GdbCore 中有 IPersistVariant 形态候选。
2. **保存器可达的接口调用已知**：SimpleSaveObject 的 IID、槽、递归、头标记及 SystemCore 写对象流接收端可由 10.1 静态映像复现。
3. **实档实例宿主/实际分支未证**：没有确认参考 MXD 的对象表中某项实现了上述私有接口，亦无非空 IPersistVariant 对象的实档字节与 ArcMap 运行追踪，无法证明谁实际使用 VariantStream、替代对象递归或 `0x32` 跳过路径。

此外本轮不追 10.8 对应实现，不闭合系统层之外的所有 COM 聚合/插件宿主，不验证各类 Save/Load 体，不运行 ArcMap/QGIS。当前 `TODO.md` SimpleSaveObject 条目应维持未勾；本报告可作为 10.1 静态分派结论和后续真实宿主研究的边界记录。
