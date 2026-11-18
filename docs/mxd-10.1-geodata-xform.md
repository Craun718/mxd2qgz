# ArcMap 10.1 IGeodataXform 的三个二进制对象体

RasterLayer v13 与 ImageServerLayer v20 的请求 IID `{5E6CF525-4E74-419B-946F-7DA9BFDF815D}`
在 10.1 esriGeoDatabase 类型库中为 IGeodataXform。前者通过普通 LoadObject，
后者通过 Variant 对象分派；两者都以具体 CLSID 决定对象体读取器。

仓外现有导出中未找到 RasterDB.dll，使用 10.1 安装介质的 PE32 x86
RasterDB.dll 10.1.0.3035（ImageBase `0x7bfb0000`、SHA-256
`97207d5cc5307bfe126e030792b5e92c085668953a736a6875d35e72d7e86400`），
以 objdump 和本机 pyghidra 补充。主 QI 分别核对三个类，并确认 IPersist/IPersistStream
的 IID `0000010C/00000109` 走对象 +0x2c；这里与 XML 虚表不同。下面地址均为该映像 VA。

| 类 | 类型库/注册 CLSID | QI、IPersistStream 虚表 | GetClassID | Load / Save |
| --- | --- | --- | --- | --- |
| IdentityXform | `C4709A2E-299E-4609-9904-6C595319B30F` | `0x7bff1290` / `0x7c129a7c` | `0x7bff16d0` | `0x7bff1630` / `0x7bff15b0` |
| PolynomialXform | `B3CAA849-5523-4490-AA2D-780ACA41E1DD` | `0x7c034730` / `0x7c12eebc` | `0x7c036ff0` | `0x7c036a20` / `0x7c036680` |
| CompositeXform | `44923EBB-D988-4847-9B29-11AA8E6E132C` | `0x7bfbd4b0` / `0x7c124f94` | `0x7bfbd220` | `0x7bfbf580` / `0x7bfbf4a0` |

- Identity：int32 版本，≥2 时一个 ISpatialReference。Save 走 Get92SaveVersion，
  同版本使用 SaveObject；Load 对 ≤1 不读空间参考。读取器限制 0–2。
- Polynomial：int32 版本（≥3 拒绝），ISpatialReference，int32×3（后两项为
  系数数量与点数量），double×2，int32 标志×3（成员 +e4、+e8、+ec）。
  +ec 非零时先读两组二维点；+e4、+e8 非零时各读两组系数 double；
  +ec 非零时再各读两组点数量×3 的 double；版本 ≥2 最后读一条 AfCore String
  （Load 在 `0x7c036f5e` 比较版本，修正反编译将版本局部量误标为指针的结果）。
  对于 +ec 的最后两组数组，本实现按 Load 要求读取；Save 存在两个指针非零保护，
  未见实际缺写输出。超大、负数量与超界数组拒绝。
- Composite：int32 版本（>2 拒绝）、int32 数量、同一对象流的 IGeodataXform
  子槽循环；版本 ≥2 再读 short。读侧非正数量不进入循环。子变换有新对象时按上述
  三类递归分派，最大深度限制 16；回引不读取对象体。

Identity 和 Polynomial 的 Load 都请求 ISpatialReference `4434D680-…`
（`0x7c123c80`），与 Envelope 共享现有空间参考分派。普通槽整体使用同一
游标和对象表事务；Variant 宿主在提交前也会确认整个变换体。缺失、未知类或
截断不会泄漏子对象键。

参考工程 /Maps 与已抽取的 199 个 10.1 介质流文件没有这三个 CLSID 的字节命中；
正常、异常和两个图层宿主测试因此使用合成字节。覆盖多项式数组顺序、复合
子变换回引和深度、非法计数，以及任何截断时回退。尚未验证其他 IGeodataXform
实现类，也未做 ArcMap/QGIS 人工验收。
