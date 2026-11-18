# ArcMap 10.1 VariantStream 对象值与 ImageServerLayer 前缀（2026-11-18）

后续已接入四个 10.1 保存版本的对象尾段字段顺序和 ImageServerName 子对象，
见[尾段取证](mxd-10.1-image-server-layer-tail.md)。下文的停止边界是前缀轮结果。

本轮定位到实际读写 `IVariantStream` 对象值的 10.1 类，并接入有界图层列表。
固定版本均核对为 x86 PE 10.1.0.3035，地址为静态映像 VA。ImageServerLayer.dll
SHA-256 `0256f97057275fedb550c947b0f8333184427755a394ef09099d22349fe5bb14`，
用 objdump、重定位、RTTI 和类型库补查原生读写；AfCore.dll 与 SystemCore.dll
身份及摘要和[原包装研究](mxd-terra-follow-stream.md)一致。本轮复用已导出的
SystemCore Ghidra 伪代码，对接口安装、参数和跳转另核对原始指令。

## 具体对象体与宿主

ImageServerLayer 的 CLSID 为 `477D13E7-8D68-45B6-A7FD-2EF442BCCE95`。
esriCarto 10.1 TypeInfo 1637 的 coclass 与 ImageServerLayer.dll
`0x77feb34e` 交给 UID 的字节（`0x7802d360`）一致。完整对象
`+0x134` 的 IPersistVariant 有下列相互独立的对位：

- RTTI COL `0x78034634` 指回 ImageServerLayer，基类位移 `0x134`；
  构造指令 `0x77fe768a` 在同一偏移安装最终虚表 `0x7802f490`。
- QI `0x77fe8d29..0x77fe8d3f` 命中 `67A4D651-69E0-11D2-8500-0000F875B9C6`
  时返回 `+0x134`。虚表 `+0x0c/+0x10/+0x14` 分别为
  `0x77feb2f0`（类标识 UID）、`0x77feb390`（Load）、`0x77fece60`（Save）。
- Load 的 `0x77feb3ce` 经传入的 IVariantStream 槽 `+0x0c` 连续读取
  VARIANT。版本是 VT_I4，原生比较有符号值 `>22` 时拒绝。Save 在
  `0x77feceb0..0x77fecebe` 使用版本表 `0x7802caf0 = [-1,-1,8,13,18,22]`；
  解析器只接入其中四个非负保存版本，旧兼容读路径未接入。

接入的前缀（每项均带 VARTYPE）：

| 组 | 字节和访问器证据 |
| --- | --- |
| 普通属性 | 版本之后，按顺序为 BSTR×2、BOOL×4、R8×3、BSTR、R8×2、BOOL、(BOOL+I2)×3，保存调用范围 `0x77fecf6c..0x77fed3e3`。有三处 BOOL 在 Load 仅消费，由 Save 对位类型。第二个 BSTR 在 `0x77feb48f` 交 ILayer `+0x10`（Name put）；四个 BOOL 中第三个在 `0x77feb53b` 交同接口 `+0x30`（Visible put）。ILayer 子对象由 RTTI 定位在完整对象 `+0x78`，属性名来自 esriCarto TypeInfo 5。 |
| 图层扩展 | 保存 `0x77fed425` 输出 VT_I2 计数，`0x77fed46e` 在 ILayerExtensions `+0x10` 取每项，`0x77fed490` 保存 VT_UNKNOWN。Load `0x77feb86c` 区分版本小于 5 的单对象路径；版本 ≥5 的正计数逐项调用 `0x77feb8cf`，`0x77feb8f0` 转交 ILayerExtensions `+0x14`。Rust 新对象目前只接入已知的 EditTemplateManager；无依据的其他类不猜测体长。 |
| IEnvelope 与 ISpatialReference | Save 分别在 `0x77fed4e9`、`0x77fed573` 写 VT_UNKNOWN；两个属性间 `0x77fed527` 固定写 VT_UNKNOWN + 空指针，Load 在 `0x77feb958` 消费该值。非空值读后经 `0x78000c50`、`0x78000cc0` 查询 IEnvelope `1B077914-38C5-11D0-92D2-00805F7C28B0`、ISpatialReference `4434D680-F4DA-11D1-BCA6-0000F875BCCE`。复用 Envelope、PCS、GCS、UnknownCoordinateSystem 的 10.1 读取器，不混用 Variant::Read 语法。 |

## 两种对象头与引用表

依据 [AfCore LoadObject 分派](mxd-simple-object-dispatch-20261118.md)和
[ObjectStream 标准包装](mxd-terra-follow-stream.md)，用明确宿主选择字节布局：
普通底流为 `[VT][CLSID][类体]`；底流提供 IObjectStream 时为
`[VT][CLSID][u32 键][新对象类体]`。两种空对象均只有 `[VT][16 字节零]`；
后者已见键不带体，未见键则在调用类体前登记。不同 Reader 可以承接同一张
ObjectStream 会话表；类体解析失败不会把读到一半的键留给后续字段。

本轮也复核了另一个 10.1 包装 **SystemCore VariantStreamIO**：
esriSystem TypeInfo 25 声明 `IVariantStreamIO::Stream` 写槽 `+0x0c`，
类型库摘要 `0f3bc6496f2665cd6b03fa0071c0077f3b6fdab8f96c8b80f02beec36d7ad7a8`。
构造 `0x7fff9df1..0x7fff9dff` 安装公开 IVariantStream 虚表
`0x800746dc` 和 IVariantStreamIO 虚表 `0x800746f4`。
Stream 写槽 `0x7fff9fa0` 将调用者提供的 IStream 传给 AfCore
`VariantStream(IStream*)`（IAT `0x8006b2d0`），保存包装指针于完整对象
`+0x1c`。IVariantStream 的 `0x7fff9ff0/0x7fffa020` 把 Read/Write
转给该包装的 `+0x10` 子接口。因此它也不建立独立键表，最终由配置的底流
决定是否有键，而非由 VariantStreamIO 的名字决定。

## 验证边界

合成测试覆盖有界 ImageServerLayer、扩展新对象和同键回引、非空 Envelope、
Envelope 内普通对象条目与 Variant 空间参考值共用既有键、独立 Reader 回引、
普通流无键头、截断与未知对象回滚。未知尾部仅靠 ExtraSafe 边界保留。
扫描已提取 10.1 介质的 132 个 Maps/PageLayout 流没有找到该 CLSID；该搜索只用于
定位候选，不是格式验证。当前没有确认实际 MXD 宿主、真实键序、ImageServerLayer
剩余 Variant 值及所有原生实现类。没有运行 ArcMap 或 QGIS，未做人机验收。
