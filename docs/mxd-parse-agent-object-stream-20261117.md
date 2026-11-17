# MXD 解析对象流与版本规则研究（2026-11-17）

本轮只做只读 REA 研究和本记录整理；没有修改 Rust 源码、测试、`README.md` 或 `TODO.md`。

## 范围与证据基线

目标对应 `TODO.md` MXD 解析节 2026-11-17 工作副本指定行：98-99、102-103、105、111-113、115。100、101、104、106、114 是当前已勾选背景项，只在必要时作为交叉证据引用。

10.1 证据继续绑定既有基线：

| 文件 | SHA-256 |
| --- | --- |
| AfCore.dll | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` |
| SystemCore.dll | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` |
| PageLayout.dll | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` |
| Map.dll | `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8` |
| ArcMap.exe | `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97` |

既有材料另有 10.8 安装包 `temp/arcgis/unpacked/Desktop/SetupFiles/Data1.cab` 提取文件身份；本轮只复核这些记录，没有对这些文件完成新的反编译：

| 文件 | SHA-256 |
| --- | --- |
| AfCore.dll | `560be16e0a99b9de4599f8241c421bddf0dcc8b26509f584074dea53b0ac4aae` |
| SystemCore.dll | `48cbd921972d12156119d524f9c3923225fd30795e33c22c65c87db3a6c89aca` |
| ArcMap.exe | `a9a963b9dffbf0cb346c67c98bce0880b7e64363410a788dccb9597cff3c75fe` |
| PageLayout.dll | `6d432cb4ee65fc3f73f20c2270db1f10e0292935dfec466fe74e8922ff2413ee` |

本轮用 REA/Ghidra 12.1.4 对上述 10.1 AfCore.dll、SystemCore.dll 完成新的函数级静态取证；没有运行 ArcMap 或 QGIS。为避免覆盖并行研究，未把本轮 REA 会话导出为新的证据包；文中保留 MCP 返回的证据 ID 和关键地址，供后续会话核对。

本轮关键函数：AfCore `LoadObject`、`SaveObject`、`SimpleSaveObject`、QI 包装与尾部包装；SystemCore `SetStream`、对象读头、对象写出器、读/写表查找与替换函数。既有记录之外的修正：`LoadObject` 普通分支只实例化零 GUID 空对象，非零 GUID 保持空输出并返回成功；这与此前把 `0x68a7b490` 泛化为“读 GUID 后创建实例”的表述不同。SimpleSaveObject 的 QI IID 也已从指令常量还原。

## 逐项结论

### 98：闭合跨流复用 ObjectStream 的对象体

**结论：机制闭合（10.1 静态），完整对象体族谱未闭合。**

已有 10.1 静态证据确认的是机制，不是全部对象体：

- ObjectStream 通过 `SetStream` 切换 OLE 流，且不清空读/写对象表，也不重置键计数器；因此键序和回引跨 `/Maps`、`/PageLayout`、`View`、`DrawingDefaults` 等流共享。证据见 [arcmap-10.1-object-body.md](arcmap-10.1-object-body.md) 180-201 行与 [mxd-open-streams-20261116.md](mxd-open-streams-20261116.md)。
- 对象头后没有通用体长；命中键只 QI，不读体；未命中键先登记，再交由具体类自己的 `Load(ObjectStream)` 或包装后的 `Load(VariantStream)` 消费。证据见 [arcmap-10.1-object-body.md](arcmap-10.1-object-body.md) 143-151 行。
- 本轮复核 `SetStream 0x7ffcf7a0`：替换 `ObjectStream+0x10`，只对新流 AddRef、旧流 Release，不触碰读表、写表或键计数器。
- 本轮复核读头 `0x7ffcff70`：读 16 字节 GUID + 4 字节键，零 GUID 返回 S_FALSE；键命中时 QI 到请求 IID，不读对象体；未命中时创建实例、在 `0x7ffd0950` 登记后调用 `IPersistStream::Load`（虚表 +0x14）或 `IPersistVariant::Load`（包装流，虚表 +0x10）。因此“跨流复用”的闭合必要条件不是通用体长，而是逐类闭合 Load/Save 完整字段序；未知类不能跳过。

开放问题：

1. 10.1 已接入读取器只覆盖严格前缀或部分体，完整 Map、FeatureLayer、FDOGraphicsLayer、CompositeGraphicsLayer、MapSurround、Bookmark 等家族仍未全量闭合。
2. 需要逐类给出“读取返回即下一对象头”的完整边界，并用含相应对象的真实 10.1 样本或受控保存流复核。
3. 静态证据不能替代 ArcMap 10.1 打开和 QGIS 转换后的人工目视验收。

### 99：确定跨流复用 ObjectStream 的版本适用性

**结论：10.1 机制部分闭合；版本外推未闭合。**

10.1 的机制证据明确：SetStream 不清表、读/写表跨流保留、对象体由类自身持久化方法决定。相关结论可直接用于 ArcMap 10.1.0.3035 静态目标。

10.8 侧既有研究记录了同构线索，但本轮没有重新打开或反编译 10.8 原生文件；文中 10.8 哈希只表示已提取文件的候选身份，不构成新的函数级证据。

开放问题：

1. 需要在 10.8 SystemCore 上重新定位等价的 SetStream、对象读分支、对象写分支、读/写表和键计数器，确认与 10.1 是否字段级同构。
2. 类级对象体规则不因容器机制相同而自动跨版本成立；每个版本仍需单独取证。
3. 8.x、9.x 等更早 MXD 目标尚无本仓已核验的原生文件结论，不能外推。

### 102：闭合 VariantStream 对象分支的对象体

**结论：机制闭合（10.1 静态）；逐类对象体未闭合。**

已有证据表明：

- AfCore 全局 `VariantStream::Read` 的 `VT_DISPATCH(9)`/`VT_UNKNOWN(13)` 分支调用导出 `LoadObject`。本轮闭合导出 `LoadObject 0x68a7b6d0` 的优先分支：`0x68a7c4b0` 对传入流 QI `IObjectStream {18A45BA7-1266-11D1-86AD-0000F8751720}`；命中时直接调用上下文接口虚表 +0x44，即 SystemCore ObjectStream 带键对象读头；未命中才进入 `0x68a7b490`。`0x68a7b490` 读 16 字节类 GUID 并与全零 GUID 比较：非零返回 S_OK 且输出保持 NULL；只有零 GUID 经 `AfCreateInstance` 创建已知空对象包装。该逻辑修正了旧记录“非零也创建实例”的表述。
- 因此非 ObjectStream 普通流上的 9/13 条目不能按 20 字节键控头解读，也不能按“读非零 GUID 后实例化类体”解读；只有确认 QI 命中 ObjectStream 后，才能回到 SystemCore 读头路径。
- SystemCore ObjectStream 读分支在对象支持 `IPersistVariant` 时也会创建 0x1c 字节 VariantStream 包装，再调用对象自己的 `Load`。证据见 [arcmap-10.1-object-body.md](arcmap-10.1-object-body.md) 132-136 行。
- 因此对象分支没有通用长度，未知类体不能安全跳过。

开放问题：

1. 需要逐类闭合支持 `IPersistVariant` 的对象体布局；`Variant::Read/Write` 的对象分支与 VariantStream 分支不能混用同一读取规则。
2. 需要含非空 `IPersistVariant` 实现类（例如 TableQueryName 族）的真实或受控 10.1 MXD，验证包装流上的键复用与体边界。
3. 10.8 的 VariantStream 对象分支体规则未在本轮重新取证。

### 103：闭合 VariantStream 对象分支的引用表

**结论：入口与引用表关系闭合（10.1 静态）；真实 IPersistVariant 场景未运行验证。**

可确认的三条路径：

1. ObjectStream 读头未命中且对象 QI `IPersistVariant {67A4D651-69E0-11D2-8500-0000F875B9C6}` 命中时，用 0x1c 字节包装流包装 ObjectStream，再调用其 `Load(+0x10)`。包装流构造把底层流存到基址 +0x18，`Read(+0x10)` 转发全局 `Read`；因此对象若再调用 `LoadObject`，新包装流仍会 QI 到 `IObjectStream` 并回到同一 ObjectStream 读表。
2. 普通流上的 `LoadObject` 先用 `IObjectStream` 判别上下文；命中走 +0x44 带键路径并复用读表，未命中走零 GUID 空对象分支。
3. AfCore 写侧 `LoadObject` 的尾部包装只处理 QI `IID_IUnknown` 与 `IDocumentVersion` 两条辅助路径，不是引用表入口。

仍未闭合的是完整接口传播链：

1. 不能仅凭 `VT_DISPATCH`/`VT_UNKNOWN` 断定具体 `IPersistVariant` 实现类在体内又通过 ObjectStream 写键；这取决于该对象自身 Load/Save。
2. 10.8 的包装流 `QueryInterface` 行为未在本轮重新取证。
3. 需要含 `IPersistVariant` 对象的真实 MXD 或运行取证，确认实际键序与体边界。

### 105：闭合 SimpleSaveObject 委派链

**结论：10.1 写入口和 QI 身份已指令级闭合；SystemCore 键控委托链整体闭合，普通持久化语义同时闭合。**

已有 10.1 证据：

- `SaveObject 0x68a7b640` 先用 `0x68a7c4b0` 对流 QI `IObjectStream`。命中时取返回接口并调用虚表 +0x40，即 SystemCore `0x7ffcf7e0` 对象写出器；未命中才调用 `SimpleSaveObject 0x68a7af70`，`writeHeader` 恒为 1。
- 本轮从指令常量还原 QI IID：`0x68a7c1d0`=IID_IUnknown；`0x68a7c240`=`IPersistStream`；`0x68a7c0f0`=`IPersistVariant`；`0x68a7c160`=`IVariantStream`；`0x68a7c320`=`IDocumentVersion`；`0x68a7c390`=`IVariantStreamIO`。
- 普通路径：`IPersistStream` 命中且流 QI `IID_IUnknown` 成功时，先 `GetClassID`；如果 CLSID 为零则只写 16 字节零空标记；否则 `writeHeader=1` 写 16 字节 CLSID，再 `IPersistStream::Save`。该路径不写 ObjectStream 键。
- ObjectStream 委托路径与 SimpleSaveObject 互斥：`SaveObject` 只在未 QI 到 `IObjectStream` 时进入 SimpleSaveObject。SystemCore 写出器本身写 CLSID、按对象指针查写表、命中只写 4 字节旧键；未命中先插入写表，再写当前键并递增 +0x2c，随后按 `IPersistStream(+0x18)` 或 `IPersistVariant(+0x10)` 分派体。

未闭合点：

1. `SimpleSaveObject` 中 `IVariantStream`/`IVariantStreamIO` 对象的具体递归规则已可按顶层分支描述，但各 `IPersistVariant` 实现类的体布局不在本函数闭合范围内。
2. 10.8 侧对应委托链未重新取证。

### 111：闭合 Map 双名称之外对象属性字段归属

**结论：部分证据，未闭合。**

当前已确认的 Map 体结构：

1. 版本 short；
2. 数据框名称；
3. 第二名称；
4. 嵌套空间参考；
5. 图层列表；
6. 基本图形层；
7. MapSurround 计数循环；
8. WKSEnvelope 显示范围；
9. 两个 int32；
10. IEnvelope 槽位；
11. 书签计数循环；
12. 其后仍有未归属字段。

证据来自 [arcmap-10.1-map-object.md](arcmap-10.1-map-object.md) 和 [src/maps/map_object.rs](../src/maps/map_object.rs) 的字段注释；实现明确在 `AfterBookmarks` 停止，其后保留未知尾部。

开放问题：

1. 书签循环之后的字段语义未闭合；需要 Map.dll 类型库成员级证据，或含非零 MapSurround/书签计数和尾部字段的受控样本。
2. 参考样本当前在基本图形层体内停止，尚未走到全部 Map 后缀，因此样本不能证明尾部字段完整规则。
3. MapSurround 共同前缀、AOI/Feature 书签与 Legend 已有静态布局；但参考样本相关循环计数为 0，非空实例的字节佐证与若干标量语义仍开放。

### 112：闭合其他 MXD 版本的字符串规则

**结论：部分证据，未闭合。**

10.1 AfCore `String::Read/Write` 已闭合：Read 读 4 字节字节长度，只检查 HRESULT 非负和 `pcbRead != 0`，不比较实际读取量；Write 写 `len*2+2` 的 32 位 `& 0x7fffffff` 长度，再写同量字节。Rust 的 `read_string_10_1` 是严格安全子集，不是原生等价。

10.8 已有同一 AfCore String 布局的严格读取器，但版本流层面另有 ArcMap.exe 观察证据：String + signed short + 条件 4 字节 + `CopyTo`；写入回填、流尾和兼容行为未闭合。证据见 [mxd-version-luna-followup.md](mxd-version-luna-followup.md) 与 [mxd-open-version-20261116.md](mxd-open-version-20261116.md)。

开放问题：

1. 8.x、9.x 等更早版本的字符串读取/写出规则缺少逐版本原生二进制证据。
2. 10.8 的 4 字节尾部、复制目标和流尾边界需要汇编级数据流追踪。
3. 语言资源对版本文本和识别候选的影响仍需多语言受控样本。
4. 需要同批文档在 10.1/10.8/更早目标中打开、保存、复读，才能验证跨版本兼容性；静态规则差异不是兼容性结论。

### 113：闭合其他 MXD 版本的对象属性规则

**结论：部分证据，未闭合。**

对象属性规则不是单一全局格式，而是每个对象类自己的 Load/Save 加版本门。10.1 已闭合的类只覆盖部分对象家族；10.8 有局部版本研究，但没有与 10.1 逐类对齐；更早版本缺少原生证据。

开放问题：

1. 需要按版本逐类取回 `IPersistStream`/`IPersistVariant` 的 Load/Save，而不是假设同一类在所有版本共享字段序。
2. 每个版本还需要受控样本或真实工程流，验证版本门实际写出的字节边界。
3. 不能用 10.1 读取器外推旧版本；也不能把 10.8 局部结论当成通用规则。

### 115：重复键替换分支场景及回引交互取证方案

**结论：未闭合；本轮只能给出取证计划。**

已有静态事实：

- 本轮复核读表插入/更新函数 `0x7ffd0950`：按 `node->hash < key` 找到边界，键相等时直接覆盖 `node+0x04` 的对象指针并返回 1；函数不释放旧对象。读头先调用 `0x7ffd0a50` 查键，只有未命中才插入，因此普通读路径不会自然触发替换。
- 写侧按对象指针查写表 `0x7ffd0850`；命中直接写旧键，不调用替换。`0x7ffdab10` 存在替换路径，但正常 SystemCore 写出器路径未显式构造该场景。
- 正常读路径先查表，只在未命中时创建并登记，因此普通 MXD 读取不会自然触发替换。
- 写侧按对象指针查写表，命中时复用既有键；重复插入同键需要绕过“对象指针已存在”的正常路径。

取证方案：

1. 读侧场景：构造同一 ObjectStream 会话中同一键出现两次的受控流。可利用 SetStream 不清表的性质，把同一键放入不同 OLE 流，使第二次读取在查表时命中。观测点是第二次读取是否进入替换分支、旧对象是否保留、后续同键回引解析到哪个对象。
2. 写侧场景：在受控运行中让同一逻辑键被再次插入，或直接驱动插入/更新函数观测返回值 1 的路径。需要记录替换前后键表状态和对象引用计数。
3. 回引交互：在替换后继续写入或读取同键回引头，确认回引解析目标、类标识校验和旧对象引用释放责任。
4. 取证工具：优先用 ArcMap 10.1 运行时 + 调试器/日志观测；若无运行环境，需获得包含异常重复键的真实或受控 MXD 字节，并用原生代码路径反推其生成方式。合成 Rust 登记表测试只能验证实现契约，不能证明原生场景。

当前只读工作区和缺少 ArcMap 10.1 运行环境，不能执行上述运行取证；因此本条不闭合。

## 本轮未做与验证边界

- 未运行 ArcMap 10.1、10.8 或 QGIS。
- 未新增 Rust 代码、测试或解析器接线。
- 未更新 `TODO.md`。
- 未更新既有研究文档或导出新的 REA 证据包；文中 REA Evidence ID 只可在对应 MCP 会话核对，文件级可复核证据包括输入 SHA-256、反编译产物索引与既有证据包。
- 10.1 结论是静态反编译，不是目标软件运行观察；Map、图层、空间参考与转换结果的人工验收仍缺。
- 10.8 文件本轮仅完成提取与 SHA-256 核验，不构成新的函数级证据。
