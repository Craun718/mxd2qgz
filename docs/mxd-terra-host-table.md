# 10.1 Map 宿主的非空独立表（2026-11-17）

本轮只增加独立前缀读取器，不改 Map 接线或 TODO。结论为静态分析；没有 ArcMap 环境，未观察目标程序运行，也未在 ArcMap/QGIS 打开和目视验证转换结果。不能把合成测试当作运行或真实对象体证据。

## 身份、方法和宿主边界

重新计算 10.1 介质提取文件 `temp/arcgis101-bin/mapdll/Map.dll.6BF03936_8AC4_4632_8FB2_A6A34A19655A`：SHA-256 `1c0675cc3a99692686a2e9dd8aaa92a68a94b48e9419d8477e33bb57634b1bb8`，与 [Map 既有基线](arcmap-10.1-map-object.md)一致；PE32 x86，ImageBase `0x793a0000`，版本资源 FileVersion/ProductVersion `10.1.0.3035`，InternalName `Map`、OriginalFilename `Map.DLL`。版本资源和摘要是身份核对，不是签名认证。

独立 Ghidra 桥接工具 6.9.0 CLI / Ghidra 12.2 / JDK 25 在专用 `temp/terra-host-table/tmp` 生成 Load 的证据 `ev_d96d18622a3f8729b06c310974642ffaed1826085639c3e7949fa30bd6e7d888`，没有切换共享 Ghidra 桥接工具。另以 Ghidra 12.2 / JDK 25 隔离 pyghidra 打开复制的既有 Map 工程，定点反编译 Load、Save、首段委派及 Map Load，并用 PE 数据、objdump 关键指令交叉检查。Ghidra 桥接工具 及 Ghidra 对长函数均有重启次数或不可达块警告，不能把伪代码当原始源码；关键字段边界仍需后续复核。工作产物存于忽略目录 `temp/terra-host-table/`，可能含本机路径，分享前脱敏。最初因 `/tmp` 空间不足沙箱启动失败，独立 Ghidra 桥接工具 调用使用提权执行；JVM 和分析目录仍放在仓库内。

Map Load `0x793ff4d0` 在书签后读两个 short、一个 double、一个 short 和 int32 计数。Map 版本 `<11` 的逐项请求 IID `0x79482c08` 是 IName；版本 `>=11` 请求 `0x79488300` 是 IStandaloneTable `{FFC6B179-E3EC-11D3-A096-00C04F6BC626}`，经 `LoadObject` 逐项读取。旧分支不能当作 StandaloneTable 类体。此接口允许其他实现，不能把每项都假定为本类。参考 [先前介质根调查](mxd-terra-next-map.md)，到达该位置的介质工程均为零计数；未到达的工程不能据此推断。

## 类身份与读取序

10.1 类清单声明 Map.dll 的 `StandaloneTable` 为 `{EE7C5047-E3DB-11D3-A096-00C04F6BC626}`；RTTI `.?AVStandaloneTable@@` TD `0x794b5004` → COL `0x79498c60`（完整对象偏移 `0xc4`）→ 虚表 `0x7948d970`，槽 `+0x0c` GetClassID `0x794594e0` 引用同一 CLSID 数据 `0x794851a8`，`+0x14` Load `0x7945d630`，`+0x18` Save `0x7945e3c0`。均为静态映像 VA。实现类名、CLSID 和接口身份不同，接线时应先按对象头分类，再核实具体类及可用的会话引用表。

Load `0x7945d64e..0x7945d677` 先读 2 字节版本，`cmp ... 0xe; jg` 为**有符号**大于 14 时失败；本读取器拒绝负数位模式，是更严格的安全子集。Save 由 `AfStreamGetSaveVersion(docVersion, 6, 0x79480cf4)` 选版本；表的 12 原始字节为 `09 00 09 00 09 00 0a 00 0d 00 0e 00`，即 `[9,9,9,10,13,14]`。版本索引与实际另存路径的运行映射仍需对照验证。

可确认的流顺序：

1. 版本后调用 `0x7945d350`：其首个持久化动作是 `LoadObject` 请求 IName（`0x79482c08`）；对应 Save `0x7945e180` 对该成员调用 `SaveObject`。Load 调用点 `0x7945d68b..0x7945d690` 未见检查辅助函数返回值，不能推断原生以同一方式处理此处出错；本读取器对截断严格报错。
2. int32 计数；逐项 AfCore String + `LoadObject`（IID 数据 `0x7948dcd0`）；然后一个 AfCore String。Save 的对应顺序为 count、每项 String + SaveObject、String。字段业务名未绑定访问器，不据名称猜测。
3. 版本 >=2 再读 int32 计数和逐项 `LoadObject`（IID `0x7948dd50`）；版本 >=3 读取对象槽（IID `0x7948bb4c`）、int32 计数和编码后的 int32 序列。计数/展开的严格边界和真实业务语义未验证。
4. 版本 >=5 读 int32；版本 >=4 读 IName 对象；版本 >=6 读 String；版本 ==7 额外读一个临时 String；版本 >=9 读 String；版本 >=10 调用 `CInternalLayerBase::ReadPreJoinDefExpressions`。
5. 版本 >=11 读 String 和对象槽（IID `0x7948dee0`）；版本 11 或 12 再读 short 和两个对象槽（IID `0x7948dca0`、`0x7948de10`）。版本 >=12 调用四个 Time* Load；版本 >=14 调用 `ExtensionHelper::LoadExtensions`。具体类体、错误与消费边界仍须分别取证。

上述为静态字段顺序，长函数包含间接调用、委派和原生对象状态相关分支；动态路径、IID 的业务名、引用类型检查及实际版本间兼容性仍未闭合。

## 读取器与交接

`src/maps/standalone_table.rs` 提供独立 CLSID 与 `read_standalone_table_prefix_10_1`：调用方已经读取并核对外层新对象头、登记外层键之后，从类体起点调用。只处理首个 IName 槽为空或会话回引、以及后续命名槽为空或回引的已确认前缀；新对象头会作为停止点返回，游标保留在头前、该新键不登记。命名槽的 String 已读并保存在停止结果；共同前缀结束后始终停在后续字段前。会话表必须包含此前所有键，否则回引可能被判为新对象；读取器不证明回引对象支持请求 IID。非法版本、负计数、超限、短读和非法字符串报错且整体回滚；计数与字符串上限由调用者传入，并非原生上限。回滚/停止均不代表原生错误处理一致。

主代理可单独落账 **10.1 StandaloneTable 静态类身份、Load/Save 定位、版本表及首段字段序**；原有非空 IStandaloneTable 完整解析/转换条目仍不能勾选：真实非空表最先常含新 IName 对象，而本读取器在此停下，尚未完整消费该表或完成 Map 接线。还需按类分派嵌套 IName、属性对象、关系对象、压缩序列及时间/扩展体，再以受控 10.1 非空工程核对边界、回引和实际运行，并在 ArcMap/QGIS 人工检查。

主审接线：Map 的第一个独立表是已确认 CLSID 的新对象时调用前缀读取器。部分成功后停在该表未闭合处，截断或异常则回退到表对象头并撤销该键；不把其后字段误判为下一表或照明属性。宿主测试使用参考流的构造替换，非真实表实例。
