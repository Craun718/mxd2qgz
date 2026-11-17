# stream 组续查：ObjectStream 分派与 Variant 键头

## 范围和证据身份

本记录只处理 TODO 133/130/131 中可由现有 ArcMap 10.1 原生文件闭合的窄项。目标二进制均为 10.1.0.3035：AfCore SHA-256 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`，SystemCore SHA-256 `f63cf9494acc155df7fa6ffcfc388f867b76ed005eac074f67161b3f7de3ad1a`。PyGhidra/Ghidra 伪代码用于定位，关键分支以 PE 原始指令复核；未运行 ArcMap 或 QGIS。

## 新闭合的保存分派事实

`IObjectStream` QI 并不发生在 `StrmUtil::SimpleSaveObject 0x68a7af70` 函数体内，而是在其上层 AfCore `SaveObject 0x68a7b640`。这是本轮对“SimpleSaveObject 的 ObjectStream QI 成功分支”这一表述的地址边界修正：

1. AfCore `0x68a7c4b0` 从 `[0x68c6dca8]` 取 IID `{18A45BA7-1266-11D1-86AD-0000F8751720}`，以流作 `this` 调 COM 虚表首槽 QI（字节指令 `call [eax]`）；输出接口仅在 HRESULT 非负时保留，失败则清空并 Release 旧输出。
2. `SaveObject 0x68a7b640` 的 `0x68a7b676` 调用该 helper。原始指令 `0x68a7b688..0x68a7b695` 检查输出接口：非空时从该接口虚表 `+0x40` 取函数并以原对象和查询接口为参数间接调用；空时才在 `0x68a7b697..0x68a7b6a4` 调 `SimpleSaveObject(obj, stream, clearDirty, 1)`。因此两条路径互斥。
3. SystemCore `ObjectStream` 主虚表 `0x800710dc` 的 `+0x40` 指向 `0x7ffcf7e0`。该函数原始指令成功路径先做 `0x7ffcf7e0` 内部的对象接口分派，随后以对象流自身为接收端走写对象逻辑；GUID、键/回引与对象体由此接收端负责。IID、地址和二进制身份与 [对象流指令记录](arcmap-10.1-object-body.md) 的静态基线一致。

读取侧对称证据：AfCore `LoadObject 0x68a7b6d0` 经 `0x68a7c4b0` 对输入流查询同一 IID；命中时调用返回接口的 `+0x44`，即 SystemCore `0x7ffcff70`。故本轮新增的可解析规则限于：**在调用方已确认 ObjectStream 上下文时，Variant 9/13 的非空 LoadObject 头是 16 字节类 GUID 加 4 字节键，并共享同一会话引用表；全零 GUID 仍是无键空标记**。新对象体无通用长度，仍须由具体类解析。

## 解析补丁

在 `src/objects/variant_object_branch.rs` 新增 `read_variant_object_branch_object_stream_10_1`：显式接收 `RefTable10_1`，先读 VARTYPE 9/13，再委托现有对象条目读取器分类空对象、新对象键头和回引。游标及引用表先在副本上推进，遇截断不提交状态。原 `read_variant_object_branch_10_1` 继续只接收普通流可证明的零 GUID 空对象；非零 GUID 仍返回 `Unsupported`。新入口仅消费头部，不表示已经实现具体对象体或应接入任何未经字段归属验证的 MXD 字段。

## 验证与边界

- `objdump -d -Mintel` 复核 AfCore `0x68a7b640`、QI helper `0x68a7c4b0`、SystemCore `0x7ffcf7e0`；IID 由 AfCore `0x68c6dca8` 的 16 字节原始数据按 Windows GUID 字节序解码。
- 合成单元用例覆盖 ObjectStream 新键登记、重复键回引、零 GUID 无键以及短键失败时游标/引用表不变；不证明 ArcMap 运行行为或真实 MXD 字段归属。
- 未闭合：10.1 目标进程实际 QI 结果、SystemCore `+0x40` 之后所有 COM 接口分支的完整运行行为、每个 Variant 持久化类的体布局、10.8 对应实现，以及 ArcMap/QGIS 人工验收。
