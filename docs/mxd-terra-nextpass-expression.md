# 10.1 标注集合与表达式属性对位（2026-11-17）

后续已将 LELP 的四个未定成员对位到原生变换属性和标注类 ID，并确认两个旧 short
在 10.1 仅用于兼容消费，见[字段对位](mxd-10.1-label-transformation-fields.md)。

本次只补充成员归属；读取顺序和既有研究见 [FeatureLayer 尾段](arcmap-10.1-featurelayer-tail.md)和[标注续研](mxd-terra-next-label.md)。地址均为静态 PE VA，O 为完整对象起点、P 为传入 Load 的指针。没有启动 ArcMap 或 QGIS。未改 TODO，以下按缺口边界而非旧行号判断。

## 证据与换算

重新核对介质提取的 LabelPlacement.dll：PE32 x86、基址 `0x78530000`，FileVersion/ProductVersion `10.1.0.3035`，InternalName `LabelPlacement`，OriginalFilename `LabelPlacement.dll`，SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`，与既有 Ghidra 桥接工具 证据一致。esriCarto 10.1 TYPELIB SHA-256 `8a70f4094c85b966ce0b0129ad7fa3d9562269c73f87e4f70cbc78c75775b578`。哈希确认字节身份，版本资源不是签名或运行证明。

复用本 DLL 的独立 Ghidra 桥接工具 CLI 尝试：此前同哈希 Ghidra 分析后桥接 Unix socket 报 `Operation not permitted`，见 [引擎研究](mxd-terra-depth-label.md)。本次使用 JDK 25、Ghidra 12.2、`GHIDRA_INSTALL_DIR=/opt/ghidra`，将既有工程复制到 `temp/terra-nextpass-expression/`，使用独占 TMPDIR、XDG 配置/缓存及 pyghidra 复核 Load、构造函数、QI、访问器，另查 PE 虚表和指令；未切换共享 Ghidra 桥接工具 目标。伪代码不是原始源码。隔离 Wine 沙箱内 socket 被拒；在隔离前缀中通过 Wine `LoadTypeLibEx` / `GetFuncDesc` / `GetNames` 导出类型库成员，工具进程退出码为 3，但目标五个类型及成员已写出，旧的同哈希导出可独立复核其中四个接口；本次新增的 IAnnotateLayerProperties 结果按 IID、槽位及 PE 实现交叉核对。64 位 Wine 的 `oVft` 除以 2 为目标 x86 槽字节偏移。忽略目录证据可能含本机路径，分享前须脱敏。

LELP 构造函数 `0x785458f0` 将 `IAnnotateLayerProperties`、`ILabelEngineLayerProperties`、`ILabelEngineLayerProperties2` 分别装在 O+`0x1c`、O+`0x20`、O+`0x24`；QI `0x78545c10` 分别匹配 `{01004142-0D1C-11D2-A26F-080009B6F22B}`、`{01004143-0D1C-11D2-A26F-080009B6F22B}`、`{59A8EEBA-8EF7-4143-B497-4EFCAF6A2405}` 并返回相应指针。构造函数采用 vbtable `0x7857cb2c`（`[-0x14,0xa0]`），持久化虚表为 O+`0xb4`；其 Load thunk `0x78547760` 减去 O+`0xb0` 的调整值，此构造路径写入 0，所以 P=O+`0xb4`。接口虚表分别为 `0x7857c98c`、`0x7857ca0c`；下表将访问器相对接口的存储地址与 `0x785468c0` 的 P 相对地址换算到同一 O。这里的接口属性名是静态绑定，不证明界面或运行效果。

## LELP：同一存储地址

| 读取器字段（Load 次序） | 类型库属性与访问器入口（put/get，省略 0x785 前缀） | 对象位置 O+ |
| --- | --- | --- |
| `expression` ② | Expression `463d0/463f0` | 3c |
| `short_74` ③ | IsExpressionSimple `46420/46440` | 40 |
| `double_54` ⑤ | Offset `46330/648f0` | 60 |
| `int32_60` ⑥ | LabelWhichFeatures `45fd0/45ff0` | 54 |
| `short_5c` ⑦ | CreateUnplacedElements `46020/46040` | 58 |
| `short_4c` ⑧ | FeatureLinked `34910/348e0` | 68 |
| `short_4a` ⑨ | UseOutput `45da0/45dc0` | 6a |
| `int32_80` ⑩ | SymbolID `46310/46560` | 34 |
| `double_34` ⑪ | AnnotationMinimumScale `460a0/46070` | 80 |
| `double_3c` ⑫ | AnnotationMaximumScale `60dd0/460c0` | 78 |
| `short_48` ⑬ | DisplayAnnotation `46140/46160` | 6c |
| `name` ⑭ | Class `45f80/45fa0` | 4c |
| `string_b` ⑮ | WhereClause `46190/461b0` | 50 |
| `int32_2c` ㉓ | Priority `46120/460f0` | 88 |
| `short_46` ㉕（版本 ≥4） | AddUnplacedToGraphicsContainer `46250/46270` | 6e |

String 访问器用 `String::operator=` / `MakeBSTR` 操作同槽；BOOL/枚举仍原样保留，不因属性名将非规范值归一化。`short_ver1` 和 `short_ver6` 读入局部变量，不能据此绑定成员。`double_24`、`double_1c`、`int32_14`、`int32_7c` 此次没有完成可靠的公开访问器对位，不命名；尤其不能把任何整数直接当作要素 OID。对象槽的行为、版本门和短读限制仍以现有代码及各自研究为准。

## ALPC：集合 ID、尾值及空条目

QI `0x785313d0` 将集合接口 `{1D5849F2-0D33-11D2-A26F-080009B6F22B}` 和集合接口 2 `{2B35E113-9F11-43D5-914F-5C7055DB37BC}` 分别返回 O+`0x18`、O+`0x1c`；构造函数 `0x785310b0` 同时安装对应虚表。vbtable `0x78579a1c` 为 `[-0x14,0x5c]`，持久化 thunk `0x78533f10` 对本构造路径得到 P=O+`0x70`。

集合 2 的类型库声明 `Add(Item, ID)`、`QueryItem(Index, Item, ID)`、`Properties(ID)`。`0x785318a0` 把接口 this+`0x44`（O+`0x60`）分配给新节点的 +4、递增该计数器并返回 ID；`0x785319a0` 按索引返回同一节点 +4。Load `0x78531be0` 对非空项将版本 ≥2 的流内 int32 写入节点 +4，随后以该值索引集合；版本 <2 则使用并递增 P-`0x10`。版本 ≥2 的尾 int32 写入 P-`0x10`，即同一 O+`0x60` 分配计数器。因此可将条目值称为**集合条目 ID**，尾值称为**下一次 Add 所用的 ID 计数器**；不等同于要素 OID，不承诺所有外部输入下 ID 唯一或单调。

**尚需修复和验证的结构差异**：Load 只有在 `LoadObject` 成功且结果非空时才读取该条目 ID；空对象则不读取 ID，也不插入节点。当前 Rust 循环对版本 ≥2 的空槽仍无条件读四字节并保留 Null 条目，可能误消费下一字段。此发现来自静态控制流，不证明实际保存器会产生空项；本轮未改变解析行为，不能把 ALPC 全版本/空项路径视为闭合。还需比对 Save、构造空项/截断输入并验证游标、登记回滚和后继字段。

## 可勾边界与剩余

- 可独立落账：上述 15 个 LELP 已读字段与 10.1 类型库属性的静态同槽绑定；ALPC 非空项 ID 及尾计数器的静态角色。源码仅更新这些已绑定字段的注释，保留现有公共字段名和原始值。
- 不可整体勾选现有 ALPC/LELP 业务语义条目：LELP 仍有未绑定字段、ALPC 空项行为与读取器存在差异；旧条目中“OID”等描述也不可从集合 ID 推出。
- 未进行 ArcMap 10.1 源工程和 QGIS 转换结果的人工检查；类型库枚举、静态反编译和 Rust 测试不替代这一步。受控修改表达式、类名、SQL、比例和标注选项后的保存/复读仍是后续运行验证。

## 本轮验证

`cargo fmt --check`、`cargo test` 和 `git diff --check` 均通过；Cargo 报全局缓存自动清理权限不足，但编译和测试成功。本轮只有注释及研究文档变更，没有新增解析行为或新增测试；现有测试不能验证本文新绑定的属性语义，也尚未覆盖上述 ALPC 空项差异。

主审按上述 Load 条件修复 ALPC：保留流内空槽用于定位，但仅非空条目消费 ID。构造一个空项后紧跟回引，验证其 ID、尾计数器、后继字节和逐字节截断回滚，见 `tests/terra_nextpass_expression.rs`。LELP 的未定字段和保存器是否会写出空项仍开放。
