# AfCore 对象读写委派链（2026-11-18）

ArcMap 10.1 的 AfCore 导出入口先查询流的 `IObjectStream`。命中接口时，由对象流处理类 GUID、共享对象键和对象体；未命中时走 AfCore 普通流路径，普通路径本身只处理 GUID 和对象体，不附加对象键。本记录闭合的是委派和错误传播。具体 COM 类如何读写对象体、真实 MXD 中哪些对象触发版本转换，是独立问题。

## 证据与复核方法

10.1 安装介质的 `AfCore.dll` 为 x86 PE，固定版本 10.1.0.3035，SHA-256 为 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，与[介质身份和导出清单](arcmap-10.1-research-source.md)一致。先读 AfCore 的 Ghidra 伪代码 `functions/0x68a7af70.c`、`0x68a7b490.c`、`0x68a7b530.c`、`0x68a7b640.c`、`0x68a7b6d0.c`。部分伪代码标记了删除分支，故以 `objdump -d -Mintel --start-address=0x68a7af70 --stop-address=0x68a7b790 <AfCore.dll>` 逐条复核，并检查 `0x68a7c0c0..0x68a7c530` 的 QI 辅助和其引用的 IID。接口名称用[原始 IID 字节](mxd-terra-round8-variant.md)、[托管互操作元数据](mxd-terra-round9-iid.md)和 [RTTI](mxd-terra-round10-iid.md)交叉核对。以下地址均为 PE 静态 VA。

## 导出入口

| 入口 | 流提供 IObjectStream（IID `18A45BA7-1266-11D1-86AD-0000F8751720`） | 流未提供 IObjectStream |
| --- | --- | --- |
| `SaveObject 0x68a7b640` | `0x68a7b693` 调虚表 `+0x40`，仅传对象；SystemCore 对象流写出器为 `0x7ffcf7e0` | `0x68a7b69f` 调 `SimpleSaveObject(obj, stream, clearDirty, 1)` |
| `LoadObject 0x68a7b6d0` | `0x68a7b72c` 调虚表 `+0x44`，传请求 IID、outer 和输出地址；SystemCore 对象流读者为 `0x7ffcff70` | `0x68a7b490` 读 GUID 并创建对象，随后按返回状态和输出决定是否调用 `0x68a7b530` |

两个导出均保留所选路径的 HRESULT。AfCore 取得接口后委派，键登记、回引和键冲突由 [SystemCore 对象流](arcmap-10.1-object-body.md)负责。普通流版本适配的递归直接回到 `SimpleSaveObject`，不会再次由导出入口选择对象流。

## 普通流写入：SimpleSaveObject 0x68a7af70

1. **空对象**：`0x68a7afc3..0x68a7afef` 写入 16 字节零 GUID，忽略 `writeHeader`，返回该次 `IStream::Write` 的 HRESULT。
2. **版本适配**：`0x68a7b004` 对流查 `IDocumentVersion`，命中才进入兼容检查；`0x68a7b026` 调其 `+0x10` 取得文档版本。若对象提供 `INoPersistInArcGISDocument`，直接返回正值 `0x32`，不写任何字节。
3. **私有版本接口**：`0x68a7b081` 查 `IDocumentVersionSupport`。其 `+0x0c(version, &supported)` 返回的 short 输出非零时转常规持久化；输出为零时，`+0x10(version, IUnknown**)` 取替代对象，`0x68a7b0e8` 直接递归保存替代对象。递归沿用原流、原 `clearDirty`，强制 `writeHeader=1`，返回递归的 HRESULT。
4. **通用版本接口**：仅在私有接口未取得时，`0x68a7b12b` 查 `IDocumentVersionSupportGEN`（IID `BB75DA81-8DFE-4122-A9C2-86F07C539AD5`）；同样先查支持标志，不支持才用 `ConvertToSupportedObject` 取得 VARIANT。`VT_UNKNOWN=13` 直接传对象；`VT_DISPATCH=9` 经 `QI(IUnknown)` 后传取得的指针；其他类型传空。根据分支在 `0x68a7b1a1` 或 `0x68a7b20d` 递归，参数与私有接口相同。兼容方法的调用点按输出做判定，没有单独传播这些方法的 HRESULT。
5. **常规持久化**：`0x68a7b280..` 分别查询 `IPersistVariant` 和 `IPersistStream`，Variant 优先。
   - IPersistVariant：`+0x0c` 先取得类标识对象，失败归一为 `E_FAIL`（`0x68a7b327`）。有头时经 `0x68a7c0c0 → 0x68a7c400` 对该对象查询 `IUIDAdmin`（IID `3B916CDD-E139-11D2-9F38-00C04F6BC61A`），接口 `+0x10` 以 GUID 输出缓冲区和空的第二输出指针取得写入的 16 字节标识；取标识和写头的 HRESULT 未检查。随后以原流构造 0x1c 字节 VariantStream，调用 `IPersistVariant::Save(+0x14)` 并返回其 HRESULT。未向该槽单独传 `clearDirty`。
   - IPersistStream：有头时调用 `GetClassID(+0x0c)`、写 16 字节 GUID，两步均未检查 HRESULT；`0x68a7b476` 调 `Save(+0x18)`，传原流和 `clearDirty`，返回其 HRESULT。两个持久化接口均未取得时返回 `E_FAIL`。

SystemCore.dll 10.1.0.3035（SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`）把该 IID 放在 `0x800745c4`。UID 的 QI（`0x7fff8dd0`）命中后返回完整对象 `+0x14`；对应虚表 `0x8007455c` 的 COL `0x80085578`、基类描述 `0x80085614` 将同一位移定名为 `IUIDAdmin`。取值槽 `0x7fff92c0` 的第二实参是可空输出指针，不是索引；传空时只取 16 字节 GUID。

两种版本适配接口独立于持久化接口。GEN 的 VT 分派属于替代对象转换；直接 IPersistVariant 保存只取类标识、构造流适配器并调用对象。

## 普通流读取：0x68a7b490 → 0x68a7b530

- `0x68a7b490` 清空输出，用 `IStream::Read(+0x0c)` 读 16 字节。负 HRESULT 原样返回；非负但不足 16 字节时返回 `E_FAIL`。全零 GUID 不创建对象，返回 `S_OK`（`0x68a7b4f5`；与 SystemCore 对象流的空对象 `S_FALSE` 不同）。非零 GUID 调 `AfCreateInstance 0x68a77660`，传标志 `0x17`、outer、请求 IID 及输出地址。
- 导出包装保持读取或工厂的负 HRESULT；非负且输出为空则返回 `S_OK`，非空则返回 `0x68a7b530` 选中的对象 Load 状态。包装在对象 Load 失败时没有再清空调用者的输出。
- `0x68a7b530` 对空对象输入返回 `E_INVALIDARG`。非空时查询 IPersistVariant 和 IPersistStream；Variant 优先，调用 `Load(+0x10)` 并传 VariantStream 接口，否则调用 `IPersistStream::Load(+0x14)` 并传原流。与保存侧相同，VariantStream 的构造函数是 `0x68a7a2c0`，向 Load/Save 传构造结果的 `+0x10` 接口子对象；分配或构造返回空时，该调用点传空接口，未构造新的 `E_OUTOFMEMORY`。读侧直接分派时假定至少能取得其中一种持久化接口，`0x68a7b630` 未检查 IPersistStream 指针。这些是原生行为观察，Rust 读取器不应照搬未检查的指针路径。

## 10.8 对照

另核对 x86 AfCore 10.8.0.12790（`winedump` 文件与产品版本；SHA-256 `560be16e0a99b9de4599f8241c421bddf0dcc8b26509f584074dea53b0ac4aae`）的 `SimpleSaveObject 0x10057930`、`SaveObject 0x10057d70`、`LoadObject 0x10057e10`。IObjectStream 的 IID 在 `0x1026c8dc`，入口仍分别调 `+0x40/+0x44`；普通流匹配相同的私有和 GEN 版本接口、两个递归分支及 Variant 优先的持久化选择。10.8 的普通读取已内联到导出，`0x10057e94` 清空输出、`0x10057eab` 读 GUID、`0x10057ef5` 调工厂、`0x10057f87/0x10058000` 调所选 Load。这里只比较分派，不以 10.8 的指令或异常行为补足 10.1。

未修改 Rust 读取器；未运行 ArcMap/QGIS。各 IPersistVariant 类的体布局、实际调用者对 `0x32` 的处理仍未归属，不影响上述选择路径。
