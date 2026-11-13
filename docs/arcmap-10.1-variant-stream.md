# ArcMap 10.1 VariantStream 读取布局复核

记录日期：2026-11-13。研究目标：在 10.1 原生实现（AfCore.dll）上复核
VariantStream 的按 `VARTYPE` 分发读取布局，逐项对照 Git 历史 `c17b324`
的 10.8 结论文档（`docs/variant-stream.md`，该文件现已被清理出工作区），
回答「10.8 严格读取子集规则是否适用于 10.1」。本文只记录静态反编译观察，
未运行 ArcMap，未在 ArcMap 10.1 或 QGIS 中人工验证任何读取或转换结果。

## 文件身份与 SHA-256 复核

2026-11-13 对本机研究文件执行 `sha256sum`，与
[基线文档](arcmap-10.1-native-baseline.md) 全部一致：

| 文件 | SHA-256 |
| --- | --- |
| AfCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF | `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17` |
| SystemCore.dll.017F52BF_20D0_487A_A583_436DA6406CFF | `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a` |
| PageLayout.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A | `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f` |
| ArcMap.exe.E97001B0_1E76_4BE8_B51F_269775FE0A80 | `1fb79eee6b442bd61c44dd3d0864ddfff06b1df882ddc59fb87380bb3ec5af97` |
| MxTools.dll.C13FAB27_D904_4317_AC52_8A308EFED288 | `d29677c8514e705c6585b1b5a27d4e5cf32f7326891ccfe6ed7beeb70ca51347` |

本轮分析全部针对 AfCore.dll 的上述摘要；地址是该静态映像的虚拟地址
（PE ImageBase `0x68a70000`，经 pefile 读取标头确认——普通 shell 工具观察，
非 REA 证据），不是运行时地址，也不是 MXD/OLE 流偏移。

## 工具与环境（偏离记录）

- 基线及此前 10.8 研究使用 REA 6.8.0 / Ghidra 12.1.4（分析配置摘要
  `e763201e…`）。本轮开始时 `/opt/ghidra` 已升级为 12.2，要求 JDK 25+，
  而 rea MCP 服务器注册环境固定 `JAVA_HOME=/usr/lib/jvm/java-21-openjdk`，
  Ghidra 提供者就绪检查失败（`unsupported_version`），MCP 深度分析不可用。
  本机另有 `java-25-openjdk`。已按 REA 恢复文档「继续使用 CLI」路径处理：
  所有本轮证据由 rea CLI 6.9.0（与服务器同一 npx 安装，`rea-agents@6.9.0`）
  在 `JAVA_HOME` 指向 JDK 25 的 shell 中逐命令生成，提供者为 Ghidra 12.2，
  分析配置摘要 `af4d30c9d76866092ce084421f113cb34393a9703542b46a5c290a493c77a8c4`。
  建议：将注册环境 `JAVA_HOME` 更新为 JDK 25 后重启 MCP 客户端，以恢复
  MCP 深度分析路径；该修改属用户配置，本轮未执行。
- 本轮使用 `temp/arcgis/afcore101-variant-snapshot.json` 作为本地分析快照
  （AfCore 10.1 摘要绑定）；每次 CLI 命令独立进程并复用该快照。主会话
  重启导致共享 MCP 会话中另一目标（PageLayout.dll）保持打开，为不干扰
  共享状态，本轮未在 MCP 侧重新打开目标。
- 证据包 `temp/arcgis/arcmap101-variant-101-evidence.json` 由本轮各命令的
  完整 Evidence 记录合并而成（合并脚本为 `temp/arcgis/afcore101-variant-work/build_bundle.py`，
  信封结构对齐既有规范包：`binding`/`entries`/`evidence_bundle`/`target`，
  每条命令附 `entries` 工作流条目，`query_id` 由真实命令数据确定性派生）。
  注意：`rea evidence-import`（6.9.0 CLI）对它返回 `wrong-kind`——该校验器
  只接受本版本 `export_evidence_bundle` 产物；既有 REA 6.8.0 会话导出的
  规范包（如 `arcmap101-save-version-table.json`）同样被拒。因此该文件
  未通过 canonical 校验，但其中 20 条记录均为本轮 CLI 命令的完整原始信封。
  两个 16 字节 GUID 常量的读取使用
  pefile 的 VA→文件偏移映射（`0x68c6bc2c`、`0x68c6bc4c`），标注为普通
  shell 工具观察，不在 REA 证据包内。

## 10.1 函数定位（问题 1）

导出表覆盖 2393 条（`ev_23258b0ae46c0c2a6beb366cd4d6804fa74841ce2d8446fdf691024046e23899`，
coverage complete），10.8 文档所列函数在 10.1 均有同名导出对应物：

| 角色 | 10.1 导出名 | 地址 | 反编译证据 |
| --- | --- | --- | --- |
| 全局 `Read`（VARTYPE 分发） | `?Read@@YAJPAUtagVARIANT@@PAUIStream@@@Z` | `0x68a7b9a0` | `ev_76277b5162dc639a8eb9ff848ec1c061d4241e95aadce80d6518f29d081a484d` |
| 全局 `Write`（对称分发） | `?Write@@YAJPAUtagVARIANT@@PAUIStream@@@Z` | `0x68a7b860` | `ev_3382b308ab8e73058106cb5828cee087423796c8035060ac8b8eeb42963da645` |
| `ReadString` | `?ReadString@@YAJPAPA_WPAUIStream@@@Z` | `0x68a7b7a0` | `ev_93c727e2e9880eb65433aef4fb6c7ca9a9428246e24f9b060eedad408b434841` |
| `WriteString` | `?WriteString@@YAJPA_WPAUIStream@@@Z` | `0x68a7b800` | `ev_4939ab646a84f8c05138cb60ea37115f0b81679b25bfd8b37a8a7b6429131998` |
| `LoadObject` | `?LoadObject@@YAJPAPAUIUnknown@@ABU_GUID@@PAUIStream@@PAU1@@Z` | `0x68a7b6d0` | `ev_0dc3f12b4b4d77b1ee841297317586957511873e2dbc63180f4ba58e566f2dcd` |
| `SaveObject` | `?SaveObject@@YAJPAUIUnknown@@PAUIStream@@H@Z` | `0x68a7b640` | `ev_cb7a225e5d7568d1a40329ada0e4f1b5989368e091b4c76ef17b26ba8d950af7` |
| `VariantStream::Read` | `?Read@VariantStream@@UAGJPAUtagVARIANT@@@Z` | `0x68a7bc50` | `ev_e5c7d1300c5f0abdb7646cffe1ad17e28caddc36ca689472d3818767700c4b7a` |
| `VariantStream::Write` | `?Write@VariantStream@@UAGJUtagVARIANT@@@Z` | `0x68a7bc70` | `ev_c5ecfab784f7dce669452a6e4ff8e1108b6a457d282d9df34ffe761338e3cf12` |
| `VariantStream` 构造 | `??0VariantStream@@QAE@PAUIStream@@@Z` | `0x68a7a2c0` | `ev_243f7c093bc9f5563b2b17906f358b6173451fedf492b809e26931fa03b46c2e` |
| `get_DocumentVersion` | 同名导出 | `0x68a7bd00` | `ev_5e47a68682a336eeeb7951848a6a2b2ad3ee35727279e71c206c78f285629c1e` |
| `put_DocumentVersion` | 同名导出 | `0x68a7bc90` | `ev_dfce71506a045f73cc814838cce4b17ab38e40e9200f6eb79ad99e49e396cfe9` |

辅助函数（非导出）：LoadObject 核心 `0x68a7b490`
（`ev_57f936441fc419b11d0479538114da51a11f1c65eda658b9e5de76972fbf5723`；
汇编 `ev_3cde9aa82029bfe8a0cff12286d8390ed5378a2ea1b562fbda7e540501ce1639`）、
LoadObject 尾部包装 `0x68a7b530`
（`ev_d9deb06d9e5505f557d896f37bc57a2679ff6e4541eb65e48774a5bb6c41e8af`）、
16 字节比较 `0x68a775f0`
（`ev_86c015f9bb848f7867691ab360ce9b13be4661d841697b615241c87f2053c6c5`）、
内部接口查询 `0x68a7c520`
（`ev_8cf0977614e5fa60c3fcc8953830574615586492b615cec2b2e31a9993bb2f22`）。

结构观察：

- 构造函数将 `IVariantStream` 子对象放在基址 `+0x10`、`IDocumentVersion`
  子对象放在 `+0x14`、连接的 `IStream` 存于基址 `+0x18`。`VariantStream::Read/Write`
  经子对象调整后的 `this+8`（即基址 `+0x18`）取流，直接转发全局 `Read/Write`。
- `get_DocumentVersion`（汇编 `ev_6becc0179ba8c62176b695c028c488111ae45b61716778ac1dc760cb4cd456e0`）
  对连接流 AddRef 后，用 `0x68a7c520` 以 `0x68c6dcb8` 处 IID（即基线已确认的
  `ecc43c55-0148-4ec1-bf87-b9a183c5dc98`）查询接口，取得后调用其虚表 `+0x10`
  写出版本，再 Release；内部流为空或查询失败时返回 `0x80004005`。这与基线
  `AfStreamGetDocumentVersion` 的转发描述同构。`put_DocumentVersion` 的
  反编译被 Ghidra 以不可达块警告拆分，尾部转发槽位未复核。

## VARTYPE 逐项对照（问题 2）

全局 `Read` 先经一次 `IStream::Read` 请求 2 字节小端 `VARTYPE`（`pcbRead=NULL`），
失败（hr<0）即返回 `0x80004005`；再按 switch 分发。全局 `Write` 对称。
下表「10.8 记录」引自 Git 历史 `c17b324:docs/variant-stream.md` 的字节观察表：

| VARTYPE（OLE 名） | 10.8 记录 | 10.1 Read | 10.1 Write | 一致 |
| --- | --- | --- | --- | --- |
| 0（VT_EMPTY） | 无载荷 | 无载荷（`lVar3=0`） | 无载荷 | 是 |
| 1（VT_NULL） | 无载荷 | 无载荷 | 无载荷 | 是 |
| 2（VT_I2） | 2 字节 | `cb=2` | 2 字节 | 是 |
| 4（VT_R4） | 4 字节 | `cb=4` | 4 字节 | 是 |
| 3（VT_I4） | 4 字节 | `cb=4` | 4 字节 | 是 |
| 5（VT_R8） | 8 字节 | `cb=8` | 8 字节 | 是 |
| 7（VT_DATE） | 8 字节 | `cb=8` | 8 字节 | 是 |
| 6（VT_CY） | 未记录（落入 E_FAIL） | default → `0x80004005` | default → `0x80004005` | 是（两版本均不支持） |
| 8（VT_BSTR） | 调用 `ReadString` | `ReadString(0x68a7b7a0)` | `WriteString(0x68a7b800)` | 是 |
| 9（VT_DISPATCH） | 调用 `LoadObject` | `LoadObject`，IID 参数 `DAT_68c6bc2c` | `SaveObject(obj, stream, 0)` | 是 |
| 13（VT_UNKNOWN） | 调用 `LoadObject` | 同上（与 9 共用调用点） | 同上 | 是 |
| 11（VT_BOOL） | 2 字节 | `cb=2` | 2 字节 | 是 |
| 17（0x11，VT_UI1） | 1 字节 | `cb=1` | 1 字节 | 是 |
| 其他 | 返回 `E_FAIL`（0x80004005） | default → `0x80004005` | default → `0x80004005` | 是 |

- 10.1 支持而 10.8 文档未记录的 VARTYPE：**VariantStream 分发内没有**。
  `VT_CY(6)`、`VT_ERROR(10)`、`VT_I1(16)`、`VT_UI2(18)`、`VT_UI4(19)`
  均落入 default/E_FAIL，与 10.8 记录一致。
- 固定宽度读取为单次 `IStream::Read(cb, pcbRead=NULL)`，hr<0 时返回
  `E_FAIL`；`pcbRead` 未检查，即原生接受 `S_FALSE` 短读（与 10.8 文档
  「不严格短读」记录一致）。
- 成功后原生把值写入输出 VARIANT：输出已为非 EMPTY 类型时经 OLEAUT32
  序号函数（`VariantChangeType` 候选）尝试转换；Rust 读取器不模拟该语义，
  两版本处理一致。
- **范围外发现（需与 VariantStream 区分）**：AfCore 另有类方法
  `Variant::Read`（`0x68a7c6a0`，`ev_0530bacf543a845d3f07bd1c81523984e985e742a725dbf944e6f310e869423f`）
  与 `Variant::Write`（`0x68a7cb50`，`ev_b119cd4c17b46c681b5edfd953d518e8846b88b9c7a085a26fab5bf49cabb81b`），
  是更丰富的独立序列化器：额外支持 `VT_CY(6)`=8 字节、`VT_ERROR(10)`=4 字节、
  `VT_I1(0x10)`=1 字节、`VT_UI2(0x12)`=2 字节、`VT_UI4(0x13)`=4 字节、
  `VT_ARRAY(0x2000)` 数组分支（4 字节计数 + SafeArray），且非数组 VT_BSTR
  走 AfCore `String::Read/Write`（字节长度含终止零，与 `ReadString` 的
  `n*2+2` 布局**不同**）。该组不是 10.8 文档所指的 VariantStream 规则，
  10.8 是否同样存在未核对；MXD 字段实际使用哪条路径尚未确认。

## 字符串布局（问题 3）

`ReadString`（`ev_93c727e2…`）：先读 4 字节小端 `n`（`pcbRead` 传入但未比较）；
`n != 0` 时以 OLEAUT32 序号 4（`SysAllocStringLen` 候选，按 `(0, n)` 调用）
分配 `n*2+2` 字节，再读 `n*2+2` 字节；hr<0 即返回该 hr，否则 S_OK。
`n == 0` 时无字符串载荷。`WriteString`（`ev_4939ab64…`）：以 OLEAUT32
序号 7（`SysStringLen` 候选）取 `n`，写 4 字节 `n`；
仅当字符串指针非空且首字符非零时写 `n*2+2` 字节。序号到 API 名的对应
是依据参数形态的推断，未逐一验证符号。

与 10.8 记录逐条一致：4 字节 `n` 是 UTF-16LE 内容码元数、末尾 UTF-16LE
终止零不计入 `n`、空字符串总长 6 字节（2 字节 VARTYPE=8 加 4 字节零长度）、
原生不检查终止零是否为 0、短读仅以 hr<0 判定。Rust 子集在完整读取、终止零
与 UTF-16 合法性上的附加严格性是安全设计，原生两版本都不保证。

## 对象分支（问题 4）

全局 `Read` 的 `9/13` 分支以固定 IID 参数调用 `LoadObject`；该 IID 位于
`0x68c6bc2c`，pefile 读得 16 字节 `00 00 00 00 00 00 00 00 c0 00 00 00 00 00 00 46`，
即 `IID_IUnknown`（`00000000-0000-0000-c000-000000000046`；普通 shell 工具观察）。

`LoadObject`（`ev_0dc3f12b…`）是 SEH 包装：核心 `0x68a7b490`（汇编
`ev_3cde9aa8…`）先经 `IStream` 虚表 `+0x0c` 读 16 字节类 GUID（`pcbRead`
有检查：hr<0 时透传该 hr，`cbRead < 0x10` 返回 `0x80004005`）；随后与
`0x68c6bc4c` 处 16 字节常量做比较（`0x68a775f0` 为 4×4 字节比较，
`ev_86c015f9…`）；常量经 pefile 读得为全零 GUID。**读取的类 GUID 等于
零 GUID 时**进入 `AfCreateInstance(&类GUID, 已有IUnknown, 0x17, IID_IUnknown, out)`
（`0x68a77660`）；**不等于零时返回 S_OK 且输出保持 NULL**。尾部包装
`0x68a7b530`（`ev_d9deb06d…`）会 `operator new` 一个 `VariantStream`
（构造 `0x68a7a2c0`）并与流关联；该函数反编译含大量不可达块警告，未闭合。

结论与 10.8 相同方向：对象分支的实际持久化依赖 COM 上下文（类 GUID、
`AfCreateInstance` 类厂查找、对象体序列化），**这 16 字节不是
`object_stream::read_header_10_1` 所表示的「16 字节 GUID + 4 字节键」对象头**，
不能把对象分支接入严格读取器。与 10.8 文档描述的差异：10.8 记录
「入口先对输入 IStream 做 QueryInterface 识别特定上下文，否则读 16 字节
类 GUID」；10.1 全局 Read 所用的 LoadObject 路径中未见该 QI 分支
（`0x68a775f0` 只是 16 字节比较），顺序与分支条件可能因版本或反编译拆分
而不同，未解决（见下）。

## 结论：10.8 严格读取子集规则在 10.1 是否成立（问题 5）

**成立**——`variant_stream` 的 10.8 严格读取子集所依据的全部读取规则在
10.1.0.3035 的 AfCore 静态实现中逐项复核一致；本轮未发现任何不成立项。
依据以上证据，`read_value_10_8` 的规则表可直接判定适用于 10.1 读取目标，
但实现复用时应按 TODO 要求保持版本范围显式（例如以类型别名/调用点命名
明确声明「经 10.1 复核后复用 10.8 实现」），而不是把 10.8 名称当作通用规则。

逐条状态：

| 规则 | 状态 | 依据 |
| --- | --- | --- |
| 2 字节小端 VARTYPE 前缀 | 成立 | `ev_76277b51…`（Read）、`ev_3382b308…`（Write 对称） |
| 0/1 无载荷 | 成立 | 同上 |
| 2、11 → 2 字节 | 成立 | 同上 |
| 3、4 → 4 字节 | 成立 | 同上 |
| 5、7 → 8 字节 | 成立 | 同上 |
| 6、10、16、18、19 等未知 → E_FAIL | 成立 | 同上（default → `0x80004005`） |
| 17 → 1 字节 | 成立 | 同上 |
| 8 → 4 字节 `n` + `n*2+2` 载荷，`n==0` 无载荷，空串共 6 字节 | 成立 | `ev_93c727e2…`、`ev_4939ab64…` |
| 末尾 UTF-16LE 终止零由写入侧保证、原生不校验 | 成立 | 同上 |
| 9/13 对象分支显式不支持（COM 上下文未闭环） | 成立 | `ev_0dc3f12b…`、`ev_57f93644…`、`ev_3cde9aa8…` |
| 原生短读不严格（pcbRead 不检查），失败游标恢复是读取器事务设计 | 成立 | 同上及 `ev_93c727e2…` |
| 不模拟输出 VARIANT 类型转换 | 不适用（两版本一致，Rust 均不模拟） | `ev_76277b51…` |

未覆盖项（不阻塞上述结论，但限制其外推）：

- `Variant::Read/Write`（`0x68a7c6a0`/`0x68a7cb50`）的扩展 VARTYPE 集、
  SafeArray 分支及其 `String::Read` 字符串布局在 10.8 的对应物未核对；
  MXD 各字段实际经由哪条序列化路径未确认。
- LoadObject 的类 GUID 语义（零 GUID、`0x17` 参数）、对象体边界与引用
  登记未闭合；尾部包装 `0x68a7b530` 的 VariantStream 关联未复核。
- 10.8 文档所述 LoadObject 入口 QI 上下文分支在 10.1 的对应物未定位。

## 适用范围声明

- 本文地址与结论仅绑定上表五个 10.1.0.3035 文件（以 SHA-256 为准），
  其中核心结论绑定 AfCore.dll 摘要 `031858c4…`；不外推到其他版本。
- 本轮分析工具为 rea CLI 6.9.0 / Ghidra 12.2 / JDK 25（分析配置摘要
  `af4d30c9…`），与基线及 10.8 研究的 REA 6.8.0 / Ghidra 12.1.4 不同；
  伪代码与汇编均为静态反编译产物，PDB 加载状态未知，间接调用与寄存器
  参数可能不完整。
- 这是静态复核，不是运行验证：未运行 ArcMap，未做人工目视检查，未声称
  任何 MXD 字段或软件兼容性已验证。
- 上表 `0x68c6bc2c`、`0x68c6bc4c` 两处 16 字节为 pefile 文件读取观察，
  不是 REA 证据记录。

## 未解决事项

- LoadObject 的 QI 上下文分支（10.8 文档描述）在 10.1 的对应位置；
  `AfCreateInstance` 类厂查找、`0x17` 参数与类 GUID 语义。
- 对象体持久化边界、引用表及 `0x68a7b530` 尾部包装的 VariantStream 关联。
- `Variant::Read/Write` 扩展序列化器的版本对照及其在 MXD 中的使用位置。
- `put_DocumentVersion` 尾部转发槽位（反编译拆分，未按汇编复核）。
- MXD 字段到 VariantStream/Variant 序列化路径的归属，及其与 OLE 流的
  边界；需结合其他对象的持久化分支研究闭合。
- 原生运行时对短读、嵌入 NUL 及异常输入的实际行为（静态只能证明分支
  存在与否，不能证明全部路径）；目标软件运行与人工目视检查均未进行。
