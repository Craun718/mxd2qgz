# PageLayout 对象尾段与 GroupElement（ArcMap 10.1）

本轮使用 PE32 x86、版本资源 10.1.0.3035 的 PageLayout.dll、GraphicElements.dll 和 CartoConverter.dll。PageLayout.dll 采用已有 Ghidra 导出并用 `objdump` 复核（SHA-256 `ba9e71f42e00d06a2311a1492fc3d1aeca544c96d9ebd3926140a11b8de9d69f`）；GraphicElements.dll（`f24451c8bf25cce7c1906ea67e5fa3ce351b8f9918c6ddcd2a40083151f77ae9`）和本轮从 Desktop/Setup1.cab 补提取的 CartoConverter.dll（`5982306cbe6564505a3c37b240e0ea75d5255c13b10865880938c2e051c1f264`）用注册表、RTTI、虚表、导入和原始指令交叉核对。下列地址是静态映像 VA。

| 顺序 | 10.1 原生依据 | 已接入 |
| --- | --- | --- |
| PageLayout 元素循环之后的五个对象 | PageLayout Load `0x7b6e4e30`（虚表 `0x7b71fa34`），Save `0x7b6e6a00`：按顺序请求 IPage、IRulerSettings、ISnapGrid、两次辅助线接口 | Page、RulerSettings、SnapGrid、SnapGuides 两槽，继续用当前引用表 |
| Page | CLSID `{DD94D76F-836D-11D0-87EC-080009EC732A}`；GetClassID `0x7b6aaea0`；Load thunk `0x7b6b11a0` 到 `0x7b6aaee0`，Save `0x7b6ab2f0` | v0..1 的纸张编号、方向、两个尺寸 double、范围、原始标量、颜色、边框和背景；仅保留存盘值，不模拟打印机环境导致的原生重算 |
| 标尺与捕捉 | RulerSettings Load `0x7b6f7530`，SnapGrid `0x7b6fac00`，SnapGuides `0x7b6fb590`；PageLayout 通过 `0x7b6ce300/0x7b6ce050` 获取 GraphicSnapEnvironment，再经 IPersistStream +0x14 直读 `0x7b6f9880` | 标尺间距、网格间距、辅助线、捕捉容差和两个 GUID 集合。直接委派无对象头；两个集合是裸 GUID，不能登记 ObjectStream 键 |
| 关联地图及视图标量 | PageLayout Load 后续 IMap 槽、short×4、一个 WKSEnvelope、四个 double；后者经 XMin/YMin/Width/Height 设置器写入布局范围；版本门依次为 2、3、4、5、6、8、9 | IMap 空槽或已确认的根 Map 回引；视图范围、版面位置及宽高、版本标量按流内值保留 |
| 版本 ≥10 布局扩展 | PageLayout 调 ExtensionHelper；条目为 int64 体长、CLSID、体，无 ObjectStream 对象键。CartoConverter 的 MapOptimizationLayoutExtension GetClassID `0x6ccc0760`，虚表 `0x6cd77b34` 的 Load `0x6ccc0de0` 调 MappingCoreLib::LoadResults，再按 v2..6 读字符串、标量及对象槽 | 空准备结果、空对象槽以及 v2..6 的其余字段，核对消耗量与声明体长一致；其他实现或非空结果/对象槽保留未解析 |
| 版本 ≥11 摘要、12/13 旧分支、13/15 元素数组 | Load `0x7b6e4e30` 后半段：ExtraSafe 槽、仅 v12/v13 的两个 double、v13 起的 LONG 和逐元素 LONG 列表，v15 继续用同一计数读取 IEnvelope 槽与 double | 当前 ExtraSafe 空槽仅接受 A=B=0 或 A=16/B=0 加空对象头；两个数组按原生顺序且计数必须与元素列表相合 |
| 布局对象之外的流尾 | 文档读函数 `0x7b708d00` 在 ObjectStream +0x44 返回后用当前 IStream +0x0c 再读 2 字节。非零走辅助 `0x7b70e800`、IPaper 和导出 IID `{2AB49820-9406-11D0-87EF-080009EC732A}` 的 IPrinter | 读取打印设置标记；零时该分支结束，非零时保留其后字节。不是 PageLayout 对象体字段 |

GraphicElements.dll 的 GroupElement CLSID `{803577D2-F8A3-11D0-83AF-080009B996CC}` 由 GetClassID `0x75035e80` 与持久化虚表 `0x7506f770` 对位。Load `0x75035ef0` 先读 v0..3、委派 ElementProperties Load、读取计数及普通 IElement 对象列表；v3 还读取 IGeometry（IID 地址 `0x7506fae4`），最后无条件委派 BaseFrameElement Load（IAT `0x7506e260`）。已支持当前布局宿主中的嵌套组、矩形和通用元素，设递归上限并在有界体失败时回退整个组，避免登记未确认的子键。

66 份 10.1 安装介质工程均从 `/Maps` 顺序读到 `/PageLayout` 并完全消费其流：布局架构版本为 v9 三份、v10 三十四份、v14 十一份、v15 十八份，亦与保存版本表 `0x7b71c840` 的 `[7,9,9,10,14,15]` 相符。所有 66 份的对象体之后均是零打印标记。SDK LinearRef 的对象体在 `/PageLayout:0x48c` 结束；仓库参考工程在 `0xb15` 的 ExtraSafe 包装内读出 GroupElement 键 2208、两个子元素及 Polygon 几何，七个布局元素之后继续读取 PageLayout 对象尾部至 `0x1c62`，打印标记至 `0x1c64`。参考工程的其他未解数据（如 `/Maps` 中的有界栅格素材）仍按原样保留。

合成测试覆盖 v0..15 的尾部版本门、逐字节截断回退、计数不一致、引用表一致性和非零打印设置内容保留。实现不覆盖未遇到的扩展类、非空摘要槽、普通对象形式的旧页面元素列表、非空打印设置体及完整文档中的其他流。未在 ArcMap 10.1 或 QGIS 中人工打开验证。
