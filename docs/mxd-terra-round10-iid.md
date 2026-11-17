# 10.1 保存私有接口：RTTI 定名与调用对位

接续 [QI 复核](mxd-terra-chain-iid.md) 和 [AfCore 保存分派](mxd-terra-round8-variant.md)。ArcMap 10.1.0.3035 的 AnnoLayer.dll、AfCore.dll 静态映像分别采用前轮记录的 SHA-256 `b487f3e0dc0e70dbc4b75f72eb6dcd8d5521eb422dbea6e3b2e201489a2d33a5` 和 `031858c414c0105b3b978c754f3d0f31ba1d98f15d44ff15b1589c9adcc47c17`。使用既有 REA AfCore 函数导出定位，再以 PE 虚表原始字节和 objdump 复核。

- AnnoLayer 构造 `0x6b1a184c` 将完整对象 +0x44 的虚表设为 `0x6b1ac8a4`。其 QI 命中 `{C9E7A782-3DDC-411F-8A66-908C579F123D}` 后返回相同接口视图。该表 +0x0c/+0x10 的实际地址分别为 `0x6b1a1d60`、`0x6b1a1d90`，两者均以 `ret 0x0c` 返回。
- 第一槽接收版本值和输出指针。空输出指针返回 E_POINTER；版本按有符号 32 位比较，小于 1 时向输出写 short 0，否则写 short 0xffff；正常返回 S_OK。第二槽只检查输出指针，正常时写 32 位空指针并返回 S_OK；该 AnnoLayer 实现不生成替代对象。
- AfCore `SimpleSaveObject` 调 `0x68a7c0f0` 查询同一 IID。命中后 `0x68a7b099..0x68a7b0b0` 调第一槽并判断 short 输出；非零继续普通保存路径，零则 `0x68a7b0bf..0x68a7b0e8` 调第二槽，并将输出指针作为对象递归传入 `SimpleSaveObject`。空替代对象命中函数的空对象写出分支。此分支发生于流支持 IDocumentVersion 且通过前置接口检查时；没有观察到实际 ArcMap 保存运行。该调用点按输出值分支，未将两次槽调用的 HRESULT 直接用作分支条件。
- 至此可确认该接口在保存路径中用于查询目标版本支持和取得替代对象；尚未找到公开类型库条目，但 C++ RTTI 确认名称为 `IDocumentVersionSupport`。此项与公开 `IDocumentVersionSupportGEN` 的角色相似，但参数及替代对象传递路径不同，不能合并两个 IID。其余实现、前置私有接口和完整委派仍待证实。

另一私有 IID `{4BF42172-9DCE-4E03-A974-6C161F94293A}` 在相同版本 MappingCore.dll（固定版本 10.1.0.3035，SHA-256 `ac6f99a6345bab8834bcd8824e50bf93c5ab153de1ce2889fe9e613f080921e7`）的原始地址为 `0x79a4a374`，从三处直接比较命中路径分别返回 QI 接收指针的 +0x38、+0x30、+0x38 并 AddRef（比较实参位于 `0x798de630`、`0x79957080`、`0x799fb2b2`）。对照 AfCore `0x68a7b069`，保存路径查询到该 IID 后释放临时接口，直接返回正数 `0x32`；不经后续普通写对象路径。尚未证明上层调用者如何处理该返回值。

RTTI 定名独立复核：通过该虚表前一指针读取 MSVC Complete Object Locator，再遍历类层级的 BaseClassDescriptor，检查非虚基（pdisp = -1）的成员偏移。按 QI 命中加数对位，忽略同偏移作为直接基接口的 IUnknown：

| IID | 虚表锚点 | 具体类 | 基接口偏移 | RTTI 类型名 |
| --- | --- | --- | ---: | --- |
| C9E7A782… | `0x6b1ac8a4[-1] → 0x6b1af270` | FDOGraphicsSublayer | +0x44 | `.?AUIDocumentVersionSupport@@`（BCD `0x6b1af3c4`） |
| 4BF42172… | `0x79a4a1f4` | DotDensityMarkerPlacement | +0x38 | `.?AUINoPersistInArcGISDocument@@`（BCD `0x79a6c00c`） |
| 4BF42172… | `0x79a64c54` | RepresentationPlacementOnVerticesWrapper | +0x30 | 同上（BCD `0x79a75734`） |
| 4BF42172… | `0x79a63458` | RepresentationEffectWaveWrapper | +0x38 | 同上（BCD `0x79a6c00c`） |

后三表锚点分别以该类的主 QI 入口 `0x798de560`、`0x79956fb0`、`0x799fb1b0` 开头，命中加数与 RTTI 描述符一致。两个名称来自原生 C++ 类型信息，不要求它们出现在公开 COM 类型库。静态定名不表示已在 ArcMap 中运行或证明完整委派链。
