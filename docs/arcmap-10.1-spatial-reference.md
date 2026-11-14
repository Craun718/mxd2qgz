# ArcMap 10.1 空间参考持久化（/Maps 投影文本）

记录日期：2026-11-13。本轮是静态 REA 研究与仓库参考样本的字节对照，未执行 ArcMap 或 QGIS；文中区分原生证据、样本辅助观察与未验证项。所有地址均为所属 DLL 静态映像虚拟地址（VA），绑定下述文件摘要。

## 来源与文件身份

五个基线 DLL（AfCore、SystemCore、PageLayout、MappingCore、MappingCoreLib）于本轮开工时用 `sha256sum` 重新复核，均与 [10.1 原生文件基线](arcmap-10.1-native-baseline.md)表格一致，复核位置为 `/tmp/mxd101-native/`。

本轮从 `Setup1.cab`（摘要见基线）新提取三个文件到 `/tmp/mxd101-geometry/`，提取后逐一核对：

| 模块 | Setup1.cab 条目后缀 | VS_FIXEDFILEINFO | SHA-256 |
| --- | --- | --- | --- |
| Geometry.dll | `F84366F9_C0B7_4267_A16E_D48D9E6DD261` | 10.1.0.3035 | `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47` |
| pe.dll | `D587F9C8_6324_4BA7_B1C5_A4C5201C6CC8` | 10.1.0.679 | `2c29c819ba6506eebd64834d562ad0664c62c3d3ed6a88bfc963df5121bee7b3` |
| pe.dll | `F84366F9_C0B7_4267_A16E_D48D9E6DD261` | 10.1.0.679 | `92f4e1354b2f9826cb204fec74800a40252c7f90cbf8443a323e4d433ad12dc7` |

三者均为 PE x86（Machine `0x14c`）。`Geometry.dll` 与基线五个 DLL 同为 10.1.0.3035；两个 `pe.dll` 条目报告 10.1.0.679，与 Desktop 3035 的组件版本映射未核对，属未解决事项。版本取自资源内 `VS_FIXEDFILEINFO` 及 ProductVersion 字符串，是资源声明，不是签名验证。原文件和提取物不纳入仓库。

## REA 入口与分析环境

本轮 REA MCP 服务器不可用：注册环境 `JAVA_HOME` 指向 JDK 21，而本机 Ghidra 12.2 要求 64 位 JDK 25+（提供者诊断报 `unsupported_version`）。研究经 `rea` CLI 6.9.0（npx 缓存入口）以 `JAVA_HOME=/usr/lib/jvm/java-25-openjdk` 完成，引擎为 Ghidra 12.2；bridge 声明核对版本为 12.1.4，12.2 为被接受的观察版本。字符串搜索与全量反编译的分析配置摘要为 `af4d30c9d76866092ce084421f113cb34393a9703542b46a5c290a493c77a8c4`；Geometry.dll 全量反编译 manifest 的 evidence ID 为 `ev_cc6d7cd18c2aa3d207835cb5238632dff09b480f062abf76ab2314c211ac4480`。Ghidra 类型信息引用 PDB `Geometry.pdb`（GUID `235bee7b-efff-44e5-bad9-9dde52dfc745`，RSDS），加载状态未知；`ClassFactory<ESRI::...>`、`ProjectedCoordinateSystem::vftable` 等名称来自该恢复的类型信息，不能视为已运行对象的证明。

证据包在忽略目录 `temp/arcgis/`：`arcmap101-spatialref-mappingcore-projcs-search.json`（`ev_70b20d890b74b3dd90aeec9bd63020877304d960e4517c4ed9fb068f8b6bb4c5`）、`arcmap101-spatialref-mappingcorelib-projcs-search.json`（`ev_fa6e9c81126ba89a66d1eb682035b43df0c959a1ba8b1c54dc7c236ab4c170cd`）、`arcmap101-spatialref-mappingcorelib-sr-search.json`（`ev_f53dfc98f07362202aa7634bf1244b7ff1ecaf093fc10677baaea09e0179f67b`）、`arcmap101-spatialref-geometry-decompile.json`、`arcmap101-spatialref-mappingcorelib-decompile.json`（该 manifest 文件为空，反编译产物 1811 个函数文件完好，见下条限制），以及关键函数伪代码副本 `arcmap101-spatialref-pseudocode/`。文件含本机路径，分享前须脱敏。字符串清单只覆盖 Ghidra 已定义字符串；字节级搜索（`search-bytes`）在 CLI 中不接受 `--snapshot`，本轮未取得字节搜索的 REA 证据，相关字节定位均为本地静态计算（python 脚本读文件），在文中明确标注。

## 原生证据：空间参考类与投影文本写出

以下函数证据均出自 Geometry.dll 上述摘要的全量反编译（伪代码副本在证据目录），地址仅适用于该 10.1.0.3035 文件。

- 类注册构造 `0x71a09a70`（伪代码副本 `0x71a09a70.c`）逐项注册「GUID 数据地址 + UTF-16 类名 + `ClassFactory<ESRI::类名>`」。注册索引 `0x20` 的表项为：CLSID 数据 `0x71a13098`、类名 `L"ProjectedCoordinateSystem"`。`0x71a13098` 处 16 字节为 `00 67 62 2a d2 1d b2 11 bf 51 08 00 20 22 f5 73`，即 `{2A626700-1DD2-11B2-BF51-08002022F573}`（本地静态计算；该值与参考样本中的对象头 GUID 逐字节相同）。相邻注册项包括 `GeographicCoordinateSystem`、`Parameter`、`Projection`、`UnknownCoordinateSystem`、`NADCONTransformation`、`SpatialReferenceEnvironment` 等；完整清单见伪代码副本。
- 接口虚表 `0x71a29ccc`（11 槽，本地静态计算）：`slot[5]` = `0x719d3230`，`slot[6]` = `0x719d3690`，`slot[7]` = `0x719d38d0`（反编译为立即返回 `0xc22`）。槽位序与 COM `IPersistStream`（GetClassID/IsDirty/Load/Save/GetSizeMax）吻合，但该接口的 IID 与名称未经证据确认。
- `0x719d3690`（伪代码副本 `0x719d3690.c`）为空间参考的保存方法，写入顺序：
  1. 2 字节 short：`AfStreamGetSaveVersion(AfStreamGetDocumentVersion(stream), 4, &DAT_71a11dc8)`。`DAT_71a11dc8` 的 4 个小端 short 为 `[2, 3, 6, 6]`（本地静态计算）。按基线已证的 AfCore 超表尾取末项语义，文档版本 5（10.x）得到 6。
  2. 4 字节长度：栈上 ANSI 缓冲的 `strlen`，**不含终止零**。
  3. WKT 文本本身，纯 ASCII 字节，无终止零。文本来源：文档版本 <2 时 `pe_projcs_to_string`；≥2 时 `pe_coordsys_clone`（可带 `pe_vertcs_clone` 的垂向分量）组合 `pe_hvcoordsys_new`，再按一个未解析寄存器比较调用 `pe_hvcoordsys_to_string` 或 `pe_hvcoordsys_to_string_ext`（pe.dll 导入）。**该布局不是 AfCore `String::Write` 的 `n*2+2` UTF-16LE 布局。**
  4. `0x719bad00`（伪代码副本 `0x719bad00.c`）：连续写 4 个字符串，各为「4 字节长度 + ANSI 字节」，源为对象内 4 个 UTF-16 指针经导入序号函数 `Ordinal_149` 转码。
  5. 1 字节零。
  6. 文档版本非零时调用 `0x719bb150`（伪代码副本 `0x719bb150.c`）：参数 1 时先写 2 字节标志（`SgCoordRefGetPrecision` 结果 bit6），再写 7 个 double——`SgCoordRefGetXY` 3 个、`SgCoordRefGetZ` 2 个、`SgCoordRefGetM` 2 个，缺省值为 `DAT_71a66f20`；随后该保存方法再写 3 个 double（`this+0xf0/+0xf8/+0x100`）。`SgCoordRef*` 为 sg.dll 导入。
- `0x719d3230`（伪代码副本 `0x719d3230.c`）为对应读取方法：读 2 字节 short，大于 7 时经 `0x719f1570` 记录 `0x80004005`（第二参数 `0x1c4`）并退出；读 4 字节长度与该字节数的文本并在栈缓冲（反编译显示 3072 字节）上补终止零，反编译未见长度上限检查；保存版本 <6 时 `pe_projcs_from_string`，≥6 时 `pe_hvcoordsys_from_string` 并可选重建垂向分量；版本 >2 时读域字符串（`0x719babf0`）与 1 字节标志，非零时重建垂向域；版本 ≥4 时 `0x719bae60(1)` 读回标志与 7 个 double；版本 ≥5 时直接读 3 个 double 到 `this+0xf0..+0x104`，更旧版本从标志位恢复。函数末尾调用 `this-0x30` 接口虚表 `+0x74`，其归属未闭合。
- MappingCoreLib.dll：图层类 CLSID 候选 `{7EE9C496-D123-11D0-8383-080009B996CC}` 数据位于 VA `0x79bfccf0`，Map 类 CLSID 候选 `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}` 数据位于 VA `0x79bfce70`（均为本地静态计算；两个 GUID 同时见于 PageLayout/ArcMap.exe 等模块）。`0x79b85d00`（伪代码副本 `0x79b85d00.c`）以 `AfCreateInstance(&DAT_79bfccf0, ...)` 创建实例，随后 `OleRun` 与 `QueryInterface(&DAT_79bfd338)`；对象身份未闭合。字符串清单（`ev_f53dfc98f07362202aa7634bf1244b7ff1ecaf093fc10677baaea09e0179f67b`）含 PDB 残留装饰名 `?InstallWebMercator@@YAJPAUIMap@@PAPAUISpatialReference@@PAUtagVARIANT@@@Z` 与 `?UninstallWebMercator@@...`，支持「Map 级空间引用操作存在于该模块」的候选判断，但不构成调用链证明。
- 字符串搜索（`ev_70b20d89...`、`ev_fa6e9c81...`）确认 `PROJCS` 不在 MappingCore.dll 与 MappingCoreLib.dll 的已定义字符串清单中；`PROJCS`/`GEOGCS` 常量在 pe.dll 的两个条目内（本地二进制筛选），与 WKT 由 pe API 生成的反编译观察一致。

## 参考样本辅助观察（非目标软件执行验证）

样本为仓库内 `temp/测试底图.mxd`（10.1 保存）。`/Maps` 导出至本机 `/tmp/r1-maps.bin`，152941 字节，SHA-256 `9852c0e2e1ff2d0cafd61d5a9d6cef03327cd971331ef8583a9651d9e8129b49`。以下偏移均为该导出流的流内偏移，全部为本地静态字节核对。

- `0x00` 的 4 字节小端值为 1；`0x04..0x14` 为对象头 GUID `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`（与上节 Map CLSID 候选一致），`0x14..0x18` 为对象键 `0x1d`；Map 对象体自 `0x18` 起。流清单中不存在独立的坐标系统数据流。
- `PROJCS` 的 ASCII 命中共 11 处（UTF-16LE 命中为零），内容全部相同：`PROJCS["CGCS2000_3_Degree_GK_Zone_37",...AUTHORITY["EPSG",4525]]`，长度恒为 455 字节。每处紧跟对象头 `{2A626700-1DD2-11B2-BF51-08002022F573}`（16 字节）+ 4 字节对象键，键依次为 `0x1e, 0x51, 0x98, 0xd9, 0x64a, 0x689, 0x6c3, 0x6fd, 0x737, 0x78a, 0x820`（首个在 Map 对象体内 `0x30` 处，其余 10 处分别嵌在各图层对象体内）。样本字节与保存方法布局逐项吻合：
  - `06 00`：保存版本 short（= 保存方法第 1 项，文档版本 5 → 表末项 6）；
  - `c7 01 00 00`：455 = WKT 字节数，不含终止零（第 2、3 项；WKT 后无终止零）；
  - 16 字节零：4 个长度为零的域字符串（第 4 项）；
  - `00`：1 字节零（第 5 项）；`01 00`：2 字节标志（第 6 项）；
  - 7 个 double（第 6 项）：`31876800.0, -10002100.0, 450265407.00157917, -100000.0, 10000.0, -100000.0, 10000.0`，字段语义未解释；
  - 3 个 double（第 6 项尾）：均为 `0.001`。
- 首个空间参考对象结束于 `0x274`，随后出现对象头 `{34B2EF83-F4AC-11D1-A245-080009B6F22B}` + 键 `0x1f`；该 GUID 不在基线五个 DLL 与 Geometry.dll 中，类身份未确认。
- 10 处图层对象体中，每处空间参考对象头之前紧邻 32 字节，恰为 4 个 double，量级约 3.739e7（东向）与 2.52e6（北向），符合该投影坐标系的显示范围候选；这是「范围 4 double 紧邻空间参考对象头之前」的定位观察，写出到该布局的原生调用点未闭合。Map 对象体的范围字段未在本样本定位。
- `0x25074` 起有一段 `01 00` + 4 字节长度 447 + ASCII 文本 `GEOGTRAN["NAD_1927_To_NAD_1983_NADCON",GEOGCS[...NAD_1927...],GEOGCS[...NAD_1983...],METHOD["NADCON"],...]`；该文本前后成对出现 16 字节 `{D661941C-DA8A-11d3-9F60-00C04F6BDD7F}`（身份未确认）。
- 流末尾在零填充后以 4 字节 `01 00 00 00` 结束，与 10.8 研究记录的「对象循环结束后再读一个 4 字节值」结构位置一致。

## 坐标系字段的宿主与 Map 侧分支闭合（2026-11-14）

对应 TODO「MXD 解析」第 67 行。证据来源：REA 6.9.0 / Ghidra 12.2 静态分析 Map.dll（SHA-256 `1c0675cc…34b1bb8`）与 FeatureLayer.dll（`2baf8706…298bb60`，均 10.1.0.3035），样本 `/tmp/r3-maps.bin`（SHA-256 `9852c0e2…8129b49`）字节对照；偏移为 /Maps 流内偏移。证据包 `temp/arcgis/arcmap101-map-cr-extent-evidence.json`（含本机路径，分享前脱敏）。未运行目标软件；详细字段序见 [Map 对象体文档](arcmap-10.1-map-object.md) 2026-11-14 节。

- **Map 对象体的空间参考读取分支补全**：Map::Load（`0x793ff4d0`）读取嵌套空间参考对象时请求 IID 数据 `DAT_79481cc0` = `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`（公开 ArcObjects 的 ISpatialReference IID；名称按 SDK 惯例标注）。对象类身份由流内类标识决定（PCS `{2A626700-…}` 等），Map.dll 不自行按 GUID 分派；16 字节零类标识 = 空对象标记（既有 SystemCore 证据）。PCS 内部布局（short 保存版本 + WKT 长度 + ANSI WKT + 4 域字符串 + 1 字节零 + 2 字节标志 + 7 double + 3 double）维持上节证据，本轮以样本首对象（`0x44..0x274`，560 字节）逐字节复核一致。
- **对上节定位观察的精确化**：上节「每处空间参考对象头之前紧邻 32 字节，恰为 4 个 double」应修正为：该 32 字节属于紧邻的 **Envelope 对象**（`{30707212-52D5-11D0-A8F2-00608C85EDE5}`，esri.clsid.xml 类名 `Envelope`，Geometry.dll）体的一段，完整结构为 `short 3` + 4 double `[XMin, YMin, XMax, YMax]` + `int32 0` + 空间参考嵌套头/回引；Envelope 头（16 字节类标识 + 4 字节键）在四 double 前 22 字节。样本 25 个 Envelope（键 74..2183）逐字节一致；上节列出的 11 处 PCS 对象头全部位于某 Envelope 体内或紧随其后。
- **图层坐标系的宿主（本轮闭合）**：FeatureLayer 体不直接持久化坐标系字段；图层坐标系 = 图层体内 Envelope（范围）对象第 4 段的嵌套空间参考——内嵌 PCS 体或 20 字节回引用（16 字节类标识 + 4 字节既有键，ObjectStream 引用语义，样本中最常见目标是 Map 的 PCS 键 30）。同名 CRS 时的内嵌/回引选择规则未闭合（样本中回引与内嵌并存）。
- **写出方向（未变，维持开放）**：Envelope 与 PCS 的 Save 本体本轮未反编译；上节 Geometry.dll PCS 保存方法（`0x719d3690`）证据仍适用于 PCS 体布局。「地图对象体与图层对象体触发空间参考子对象写出的调用点未定位」仍开放，但宿主结构已明确（Map: 第 4 步嵌套对象；图层: Envelope 体内嵌/回引）。
- **Map 显示范围**：Map::Load 在第 5 步计数循环后原样读 32 字节 WKSEnvelope（4 double，非对象），写入 `+0x230` 子接口 `+0xe0` 槽；随后两个 int32 写入 `-0x40` 子接口 `+0x38`/`+0x40`。字段序 `[XMin, YMin, XMax, YMax]` 为候选（依据 Envelope 同布局的 25 例排序证据）。样本定位候选 `0x24f64`，未闭合。
- 上节未解决事项中「`{34B2EF83-F4AC-11D1-A245-080009B6F22B}` 类身份未确认」更新：esri.clsid.xml 声明其为 AnnoLayer.dll 的 FDOGraphicsLayer（维持候选的），样本内出现 1 次（`0x27a`，键 31，Map 对象体内图形/注记层集合首项）。

## Envelope::Save、WKSEnvelope 写方向与 Map 显示范围闭合（2026-11-14 写方向取证轮）

对应 TODO「MXD 解析」第 75 行。本轮走写方向：在 Geometry.dll 反编译 Envelope 的 IPersistStream Save/Load 本体，AfCore 反编译 WKSEnvelope 结构访问器，Map.dll 反编译 Map::Save 的显示范围段，与样本 /Maps 字节对照闭合流内定位。证据包 `temp/arcgis/arcmap101-envelope-save-evidence.json`（含本机路径，分享前脱敏）。全部为静态取证，未运行目标软件。

### 文件身份

Geometry.dll 本轮从 `Setup1.cab` 重新提取到 `temp/arcgis101-bin/geometrydll/`，SHA-256 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`、PE VS_FIXEDFILEINFO 10.1.0.3035、ImageBase 0x717e0000，与 2026-11-13 轮记录一致；其全量反编译产物（2026-11-13 轮,13392 个函数文件）按同一 SHA-256 与 analysis digest 复用。Map.dll（`1c0675cc…4b1bb8`）与 FeatureLayer.dll（`2baf8706…98bb60`）从同一 cab 提取核对，均为 10.1.0.3035。样本换用 `samples/测试底图.mxd`（提交 5393274 脱敏版，/Maps SHA-256 `fc0149f1…acfe`，152941 字节）：流长与原 `temp/测试底图.mxd` 导出（`9852c0e2…29b49`）相同、清洗仅替换路径字符串内容，本文所有流内偏移在两份导出中逐字节一致（渲染器等个别字节内容除外，见下）。

### Envelope::Save 与 Load 本体（证据：确定的）

类身份：Envelope CoClass 构造 `0x7186a640` 装配 26 个内嵌接口虚表（IGeometry5/IEnvelope3/IEnvelope4/IEnvelopeGEN/IArea/…/IBGLShape），WKSEnvelope 结构内嵌于 CoClass+0x78、空间参考指针 CoClass+0xA4、Z/M 域源对象指针 CoClass+0x98；RTTI TD `0x71a67570`（`.?AVEnvelope@ESRI@@`）共 26 个 COL。持久化接口为构造第 0x10 槽（字节 0x40，反编译未命名）的虚表 `0x71a1587c`，其 GetClassID 槽 `0x71864170` 写死 CLSID 字节 `{30707212-52D5-11D0-A8F2-00608C85EDE5}`，IsDirty `0x719d3220` 恒返 0，GetSizeMax `0x71869ee0` 返 0x22。

- **Save `0x71869da0`**（反编译+汇编逐条核对）写入顺序：
  1. short 版本 = **静态常量 3**（`DAT_71a11ea4`，不走 AfStreamGetSaveVersion 表）；
  2. 32 字节：栈上调 AfCore 导出 `WKSEnvelope::QueryCoords`（thiscall，this=CoClass+0x78），四个出参按序连续 `AfStreamWrite(…,0x20)`；
  3. int32：`this+0xc` 子接口虚表 +0xc 槽返回值（RTTI 命名 IAttributeAwareness 候选，语义未定）；
  4. 该 int32 低字节 bit6(0x40) → 追加 2 个 double（取 `[this+0x58]+8` 与 `+0x20`，Z 域候选）；bit7(0x80) → 追加 2 个 double（取 `[this+0x58]+0` 与 `+0x18`，M 域候选）；
  5. `SaveObject(this+0x64, stream, fClearDirty)` 写出空间参考子对象/回引。
- **Load `0x71869b60`**：short 版本 >3 拒绝（`FUN_719f1570(0x80004005,0x1c4,…)`）；版本 ==1 只读 32 字节（无 int32、无 SR）；版本 2/3 读 32 字节 + int32，int32 非零时调 `this+0xc` 接口 +0x10 槽，低字节 bit6/bit7 各读 0x10 字节 Z/M；**版本 ==3 才 LoadObject**，请求 IID `DAT_71a15a10` = `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`（与 Map::Load 第 4 步同值）。
- 写侧恒写版本 3 完整布局，读侧按 1/2/3 分支——与既有 PCS 结论同构。样本 25 个 Envelope 全部版本 3、int32=0（无 Z/M 追加段），与写侧分支一致。

### WKSEnvelope 字段序（证据：确定的，写方向）

AfCore 访问器导出逐指令绑定：`?XMin@WKSEnvelope` (0x68abe9d0) 读 +0x00、YMin (0x68abea70) 读 +0x08、XMax (0x68abea20) 读 +0x10、YMax (0x68abeac0) 读 +0x18；`WKSEnvelope::QueryCoords` 导出 1722 (0x68abf530) 的四个出参即 +0x00/+0x08/+0x10/+0x18（NaN 时四个出参均写 `DAT_68ccb760` = 4.94e-324）。Envelope::Save 将 QueryCoords 出参连续写出，故 **流内 32 字节序 = [XMin, YMin, XMax, YMax]**，样本 25 例 d0<d2、d1<d3 相容。第三轮该字段序的「候选」升级为写方向证据。

### Map 显示范围（WKSEnvelope）写出与样本定位（闭合）

- Map::Save `0x79401850`：栈上构造 WKSEnvelope → 调 `*(Map+0x298)` 子接口虚表 +0xdc 槽填充该结构 → `AfStreamWrite(stream,&wks,0x20)` 原样 32 字节（Map::Save 的 this 为 Map+0x68 的 IPersistStream 子接口，+0x230 即 Map 基址 +0x298；IPersistStream 虚表 `0x794874a4` 由构造 `0x793e2820` 写入 Map+0x68 证实）。字段序经上述 WKSEnvelope 结构证据确定。
- Map::Load 对称：`AfStreamRead`（IAT 0x794801fc，导入签名即 `PAUIStream@@PAUWKSEnvelope@@KPAK`）读 0x20 字节入栈结构后调 +0xe0 槽写入同一子对象；该子对象还在 +0x18 槽接收空间参考指针。`Map+0x298` 由构造置零、析构 `FUN_793e3a80` 释放、`FUN_793f3aa0` 置换、`FUN_793e1820/793ef8c0` 读取、`FUN_793fdfd0→793fdcc0` 惰性创建（创建路径对 `{1B077914-38C5-11D0-92D2-00805F7C28B0}` 做 QI，同类对象亦以 +0xdc 槽取 WKSEnvelope 出参并用 XMin/XMax/YMin/YMax/PutCoords/Width/Height 运算）。**该助手类的类身份静态未闭合**（开放）。
- **样本流内定位闭合（原候选 `0x24f64` 升级为确定）**：`/Maps:0x24f64` 4 double = `37392944.2840, 2520083.2851, 37394230.5290, 2520951.8351`，前后结构逐字节吻合 Map::Save 尾段：`0x24f60` int32=0（第 5 段 `{FE3115E9}` 循环计数，第四轮开放项确认=0）→ WKSEnvelope 32B → int32=9、int32=9 → `0x24f8c..0x24f9f` 20 字节零（SaveObject(+0x168) 空标记 = 16 零类标识 + 键 0）→ `0x24f9c` int32=0（`{EE53528B}` 循环计数，同样确认=0）→ `0x24fa0` `ff ff` → short=0 → double=5537.0（+0x200）→ short=0 → 对象头 `{1C352F40-298E-11D3-9F4F-00C04F6BC619}` 键 2178。第四轮「对象头 {1C352F3F} 键 2178」应修正为类标识 **{1C352F40}**（其 +1 关系：请求 IID `{1C352F3F}` = DAT_79487e60）。该对象体内含键 2182/2183 两个 Envelope（`0x252f2`/`0x25340`），值与显示范围完全相同——空间书签类候选，类名未确认。

### PCS Save 方向（复核，无新证据）

`0x719d3690` 写侧恒输出 v6 完整布局（版本 short、WKT、4 域串、1 字节零、标志+7 double、3 double，均 2026-11-13 证据），读侧 `0x719d3230` 按 1/2/4/5/6 分支兼容旧版本——读写对称成立，本轮未发现新分歧。

### FeatureLayer::Save（0x6f6eb1b0，写侧新证据；归因仍开放）

虚表 `0x6f7acfb0` 槽[6]（槽[3] GetClassID `0x6f6ec450` 写 `{E663A651-8AAD-11D0-BEC7-00805F7C4268}` 证实类归属）。SaveObject 成员序列（本机汇编/伪代码核对，全清单见证据包）：`+0x140`（Load 侧 `{40A9E884}` 结果，样本=WorkspaceName 键 34）→ 16 字节原样 GUID `+0x144` → `String(+0x154)/double(+0x158)/double(+0x160)/short(+0x168)` → `int32(+0x178)+循环 SaveObject(+0x174)` → `SaveObject(+0x1b8)`（Load 侧 `{20CD40B0}` asm 0x6f6e9b5c 存 +0x1b8）→ `String(+0x1a8)/String(+0x184)+int32(+0x188)` → `(+0x1d0) 计数+[String::Write(名)+SaveObject]` → `SaveObject(+0x1cc)`（= ALPC，Load `{1D5849F2}`，样本键 59 @0xafe）→ `short(+0x1bc)` → `SaveObject(+0x1dc)`（= FeatureIDSet 家族，Load `{D5BB4B87}`，样本键 75 @0xf63）→ 选择集 ID 数组 → `SaveObject(+0x218)` → … → `SaveObject(-0x84)`（Load ≥0x1e `{BD724B95}`→-0x7c，两函数 this 差 8）→ `SaveObject(+0x2f4)`（Load >0x20 `{E720618B}`）→ `short(+0x2ca)` → 末个 SaveObject。数据源段 `FUN_6f6e94e0`（Load）/`FUN_6f6eaf70`（Save）承载 FeatureClassName（其体内嵌 WorkspaceName）。

**归因开放项（更新第三轮第 12 条）**：样本 11 个 FeatureLayer 体在 +0x144 GUID（`{6ADB9601-…}`）之后均为固定 7 字节 `15 00 00 00 00 00 00` 且紧跟 SimpleRenderer 头（键 32 @0x3ed）；而 RTM 3035 反编译的 Save 在 GUID 后写 `String::Write(+0x154)`（AfCore `String::Write` 0x68ab4680 已反编译，cb=len\*2+2 恒为偶）+ 两个 double + short（≥22 字节），Load 侧同为无条件读——与样本不可调和。旁证：渲染器在样本体内位置（+0x143）与 Load 的 >0xd 版本门位置亦不符。工作假设为样本由 10.1 某服务包构建保存（/Version 串不区分 SP），未验证；三个 Envelope（键 74/80/151）由哪个 LoadObject 装入、两个 Envelope 的字段语义（数据全范围 vs 当前范围候选）因此仍开放。Envelope 对象体本身的读写布局（Geometry.dll 本轮闭合）不受此影响。

> **[2026-11-14 归因轮更正：本节「不可调和」异常已消解，撤销。]** 旧锚点的 16 字节是
> WorkspaceName 体第⑦步的工作空间工厂 CLSID `{A06ADB96-…}`（ShapefileWorkspaceFactory，
> 位于 `0x3d7..0x3e7`，旧读法起点提前 1 字节才得到畸形的 `{6ADB9601-5CA0-D1D9-…}`）；
> 「7 字节」= 工厂 CLSID 末字节 + WS 尾随 int32 + FCN 版本>1 选择子；SimpleRenderer 紧随其
> 后是 FL::Load 数据源段后的**渲染器槽**（`{40A9E884}`=IFeatureRenderer），真正的 +0x144
> 原样 GUID 在渲染器体之后且样本中为 **16 零字节**，其后字段与 RTM 反编译逐字节一致。
> SP 工作假设不再必要。逐字段证据见
> [下文归因轮](#envelope-归因闭合0x144-异常消解与-map-显示范围助手身份2026-11-14-归因取证轮)。

### 未解决事项（本轮新增/更新）

- Map `+0x298` 显示范围助手对象的类身份未闭合；`{1C352F40}` 键 2178 对象类名未确认（书签类候选）。
- Envelope int32（attribute-awareness 候选）与 Z/M 追加段语义按结构命名，未经目标软件行为验证。
- FeatureLayer +0x154 字段 7 字节异常、渲染器位置矛盾、Envelope 的 LoadObject 归因如上，均开放。
- 本轮仍未运行 ArcMap/QGIS，人工目视检查未进行；不得声称已验证。

> **[2026-11-14 归因轮更正：]** 上条中「+0x154 字段 7 字节异常、渲染器位置矛盾」已消解（见
> 上节更正与下文归因轮）；`{1C352F40}` 类名经 esri.clsid.xml 确认为 **IlluminationProps**
> （照明属性，esriCarto 族；请求 IID `{1C352F3F}` 在 esriCarto 类型库命中 IIlluminationProps），
> 「书签类候选」撤销——样本无书签，键 2178 对象是 Map 尾部的照明属性对象。Map `+0x298`
> 助手身份升级为「IEnvelope 持有者」（见下文归因轮）。

## Envelope 归因闭合、+0x144 异常消解与 Map 显示范围助手身份（2026-11-14 归因取证轮）

对应 TODO「MXD 解析」第 83 行（Envelope 两实例字段语义、键 74/80/151 的 LoadObject
逐条归因、+0x144 GUID 后 7 字节与 RTM 3035 不可调和、Map `+0x298` 助手身份、
`{1C352F40}` 类名）。证据：静态 REA（pyghidra 3.2.0 + Ghidra 12.2 + JDK 25，本地）+
样本字节逐字段对照 + 类名/IID 查证（`esri.clsid.ecfg` 解出的 `esri.clsid.xml` 全表
10675 类、`/tmp/mxd-arcgis101-guiddocs/wine/` 的 esri\*.olb 类型库 dump）。证据包
`temp/arcgis/arcmap101-envelope-attribution-evidence.json`（含本机路径，分享前脱敏）。
全部为静态取证，未运行目标软件，未在 ArcMap/QGIS 中人工检查。

### 文件身份（本轮新增/复核）

| 文件 | 来源与摘要 | 版本 |
| --- | --- | --- |
| LabelPlacement.dll | 本轮从 `Setup1.cab` 条目 `LabelPlacement.dll.6BF03936_…` 新提取（此前各轮未登记），SHA-256 `8415f63c1889e269ba6ae585538f601130c20cef02e4556e875a52217d35ea3d`，ImageBase `0x78530000` | 10.1.0.3035 |
| FeatureLayer.dll | 沿用 `temp/arcgis101-bin/featurelayerdll/` 既有提取物，SHA-256 `2baf8706…98bb60` | 10.1.0.3035 |
| GdbCore.dll | 沿用第六轮登记（SHA-256 `ba5d8f5d…39ca1c4`）；本轮独立反编译复核与第六轮结论一致 | 10.1.0.3035 |

样本为 `samples/测试底图.mxd` 的 `/Maps` 导出 `temp/ea-round/maps.bin`（152941 字节，
SHA-256 `fc0149f1…eacfe`，与写方向轮 sanitized 导出同源同摘要），偏移为流内偏移。

### +0x144「7 字节不可调和」异常消解（结论：异常为误读，撤销）

旧异常由两层误读叠加（第五轮已把 `{40A9E884}` 更正为 IFeatureRenderer，但写字轮的
字节走查未回传该更正）：

1. **WorkspaceName 体边界**：WSN::Load `0x70a27950`（GdbCore，GetClassID `0x70a27910`
   写 `{5A350011}`）字段序 = short 版本(≤1) → String(+0x20) → String(+0x24) →
   String(+0x28) → LoadObject(IPropertySet 族 `{F0BA6ABC}`，样本 16 零空标记) →
   1 字节标志(样本 1) → 标志非 0 时**原样 16 字节工作空间工厂 CLSID** → int32(+0x38)。
   样本 `0x3d7..0x3e7` 即该工厂 CLSID = `{A06ADB96-D95C-11D1-AA81-00C04FA33A15}`
   （esri.clsid.xml：**ShapefileWorkspaceFactory**）；旧读法把起点定在 `0x3d6`（提前
   1 字节），得到 Data3=0xD1D9 的畸形 `{6ADB9601-5CA0-D1D9-…}`，并把工厂 CLSID 末字节
   `15`、WS 尾随 int32 0、FCN 版本>1 选择子 short 0（`0x3e7..0x3ed`）拼成「固定 7 字节」。
   FCN::Load `0x709b5080` 与第六轮结论一致（三字符串中第二为空串、int32×2、选择子、
   版本>1 第二选择子），体界 `0x30a..0x3ed` 逐字段吻合。
2. **渲染器槽位置**：FL::Load `0x6f6e9940` 数据源段后的 `LoadObject(DAT_6f7a98cc=
   {40A9E884}=IFeatureRenderer)` 槽就是样本 SimpleRenderer（键 35 @`0x3ed`，体 =
   `0x6f724cf0→0x6f722b70`：short 版本(≤3) → ISymbol `{F3435802-5779}` → ILegendGroup
   `{167C5E9F}` → ILegendItem `{52AA5BC0}` → …）。渲染器位置与 >0xd 版本门「矛盾」
   系把 FL 本体另一 ISymbol 槽（选择符号，ver>0xd）当成了渲染器。

**真 +0x144 锚点字节核对（四个图层体，逐字段）**：渲染器体之后依次为 +0x144 原样
16 字节 GUID（**样本全部为 16 零字节**）→ String(+0x154)（字段名样字符串）→
double(+0x158)/double(+0x160) → short(+0x168) → IHyperlink 计数 → IColor 槽
（`{20CD40B0}`，样本 16 零空标记）→ String(+0x1a8)/String(+0x184)（样本空串）→
int32(+0x188) → FieldInfo 计数 + 循环 [String + LoadObject(IFieldInfo `{A2BAAE2C}`)]：

| 图层 | +0x144 GUID @ | String(+0x154) | double×2 | FieldInfo 计数 @（实际条目） |
| --- | --- | --- | --- | --- |
| 层 32（shapefile） | `0x5b0`（全零） | `BSM` | 0.0/0.0 | `0x602`=14（14 条 ✓） |
| 层 158（FileGDB） | `0x303b`（全零） | `ZDDM` | 0.0/0.0 | `0x308f`=29（29 条 ✓） |
| 层 218（shapefile） | `0x45e9`（全零） | `DJZQDM` | 0.0/0.0 | `0x4641`=8（8 条 ✓） |
| 簇层 1555（注记） | `0x17525`（全零） | `BSM` | 0.0/0.0 | `0x17577`=14（14 条 ✓） |

计数与流内实际 FieldInfo 对象数（剔除体内嵌 NumericFormat）逐一吻合。**样本与
RTM 3035 反编译完全可调和，「样本由某 10.1 服务包构建」工作假设不再必要，撤销**
（无法追溯历史保存环境，但已无任何未解分歧支撑它）。遗留：+0x144 GUID 非零形态
未在样本出现，语义仍开放。

### FeatureLayer 的两个 IEnvelope 槽位（汇编绑定）

FL::Load `0x6f6e9940` 有且仅有两处以 `{1B077914-38C5-11D0-92D2-00805F7C28B0}`
（esriGeometry 类型库命中 **IEnvelope**）为请求 IID 的 LoadObject：

| 槽 | 版本门（汇编） | 成员（汇编） | Save 侧对应 |
| --- | --- | --- | --- |
| Envelope #1 | `>8`（`0x6f6ea569`: CMP/JLE） | `+0x1b4`（`0x6f6ea579`: LEA ESI,[EBX+0x1b4]） | SaveObject(+0x1b4) |
| Envelope #2 | `≥0xf`（`0x6f6ea845`: CMP/JL） | `+0x1f4`（`0x6f6ea8ae`: LEA ESI,[EBX+0x1f4]） | SaveObject(+0x1f4) |

样本 11 个 FeatureLayer 体（版本均 0x22）：#1 槽全部为 16 零空标记（对应区间无
Envelope 对象头，扫描级观察）；#2 槽承载实际 Envelope。

### LabelEngineLayerProperties 的第三个 Envelope 装载点（LabelPlacement.dll）

LELP（CLSID `{01004145-…}`）IPersistStream 虚表 `0x7857cb0c` 槽5 Load `0x78547760`
→ `FUN_785468c0`，字段序：short 版本(≤7) → 标量/String 组 → LoadObject(ITextSymbol
`{A80B5E91}`) → LoadObject(IBasicOverposterLayerProperties `{3141F2FB}`) →
LoadObject(IAnnotationVBScriptEngine `{AA157206}`) → **LoadObject(IEnvelope
DAT_7857cb4c=`{1B077914}`)** → double×2/long×2 → [版本>2：LoadObject(IGraphicsContainer
`{D09411D1}`)] → 版本尾段。即注记/标注类属性对象自带一个 Envelope 槽。

### 三个 Envelope 的逐条归因（闭合）

| Envelope | @偏移 | 装载者（证据绑定） | 宿主 | 值特征 |
| --- | --- | --- | --- | --- |
| 键 74 | `0xedd` | LELP::Load `FUN_785468c0` 第 4 个 LoadObject（IEnvelope） | LELP 键 60（FL 32 的 ALPC 槽 `+0x1cc` 内，ALPC 键 59） | 界址点数据全集范围（约 65km，候选：注记类范围） |
| 键 80 | `0x122b` | FL::Load `0x6f6e9940` Envelope #2（ver≥0xf，成员 `+0x1f4`） | FeatureLayer 32 本体 | 严格内含于 Map 显示范围的紧窗口；XMin 与层 158 值相同到毫米级（视图派生） |
| 键 151 | `0x2872` | FDO 第 6 步委派注记 FeatureLayer 体（`0x1622..`）内同族 ver≥0xf 槽（前置 `[short][int32]` 模式与 FL #2 槽一致） | FDOGraphicsLayer 键 31 的委派体（体自身 Load 函数未定位，成员偏移未定） | ≈ Map 显示范围 |

全样本 25 个 Envelope 的宿主分布：LELP 槽 10 个（74、1603、1667、1724、1782、
1840、1923、2001、2073、2149）；FL #2 槽 12 个（80、151、216、258 及簇 9 体的第二
Envelope 1609/1672/1730/1788/1846/1929/2007/2079/2155）；Map 尾簇 2 个（2182/2183，
IlluminationProps 2178 之后的 LELP 族结构内，值恰为 Map 显示范围——Map 自身第 8 步
IEnvelope 槽（SaveObject(+0x168)）在样本为空标记，不承载它们）。

### 字段语义（候选保留，提法更正）

原「数据全范围 vs 当前范围」二选一提法作废——两槽值呈「视图相交范围 ↔ 数据范围」
混合分布：

- FL `+0x1b4`（ver>8）：样本全空标记，无值证据，语义开放。
- FL `+0x1f4`（ver≥0xf）：层 32/158/216 与簇体第二 Envelope 多为紧贴显示窗口的范围
  （层 32 XMin 37392987.7829 与层 158 的 37392987.7828 相同到毫米级 ⇒ 视图派生而非
  数据派生），但层 218 的 258 为 59km×52km 宽数据范围。候选：图层实际绘制/缓存范围
  （显示窗口与数据相交；未绘制层回退宽范围）。缺成员命名证据（类型库成员级 dump）
  与绘制路径调用取证，保持候选。
- LELP Envelope 槽：簇体首 Envelope 多为共享数据范围（1667=1724=1840=2073 同值；
  1782/1923/2001/2149 同整值范围），层 32 的 74 为全集范围，簇层 1 的 1603 反而为
  1.4km 小范围。候选：注记类范围，同上未验证。

### Map +0x298 显示范围助手身份（升级：接口闭合）

Map 基址 `+0x298` 助手的惰性创建链（`FUN_793fdfd0→793fdcc0`，既有证据）内以
`{1B077914}` 做 QI；本轮经 esriGeometry 类型库确认该 IID = **IEnvelope** ⇒ 该成员是
**IEnvelope 持有者**（Map 的显示范围包络对象），其 `+0xdc`（取 WKSEnvelope 出参）/
`+0xe0`（写回）/`+0x18`（写空间参考）槽即 IEnvelope 方法族（+0xdc/+0xe0 相邻 get/put
对）。coclass 归属静态仍未闭合：最合理候选为 Geometry.dll Envelope CoClass
`{30707212}`（样本同簇流内对象正是该类），但 QI 源对象与 vtable 匹配未取证，保留开放。

### SP 介质检查（结果：无）

`/home/gxrs/下载/arcmap10.1/ar10.1/` 仅有 RTM DVD；`*sp1*` 命中的
`DotNet35/dotnetfx35sp1.exe` 为 .NET Framework 3.5 SP1，LicenseManager solaris/linux
的 `kb_patchid.txt` 为 FlexNet 工具文件，均与 ArcGIS 10.1 服务包无关。随异常消解，
SP 介质对照需求一并撤销。

### 本轮未解决事项

- +0x144 原样 GUID 语义（样本恒全零，非零形态未见过）；FCN/WS 各 String 业务语义。
- FL `+0x1b4` 槽语义（样本无值）；`+0x1f4`/LELP 槽成员命名（需类型库成员级 dump
  或绘制路径取证）。
- 委派注记 FeatureLayer 体（FDO 第 6 步）自身 Load 函数与成员偏移；渲染器体尾段
  （RotationRenderer/SizeRenderer）与 EditTemplate 族逐类闭合。
- `{97D5B914-FB22-11D1-94A2-080009EEBECB}`（FL 版本>0x10 槽）未在既有 olb dump 命中。
- Map 尾簇（IlluminationProps 之后的 LELP 族与 2182/2183 宿主链）逐字节对位。
- 全部静态取证，未人工验证。

## 与 10.8 记录的对照

Git 历史 `c17b324:docs/maps-analysis.md` 记录 10.8 历史样本 `/Maps:0x46..0x4a` 为 4 字节长度 455、`0x4a` 起 ASCII `PROJCS[`；本轮 10.1 参考样本在完全相同的偏移观察到相同值。两样本是不同文件，偏移一致只说明首 Map 对象体中空间参考子对象的序列化位置稳定，不能将 10.8 偏移迁移到其他 10.1 文件。本轮原生证据来自 10.1.0.3035 的 Geometry.dll，未与 10.8 DLL 做函数级对照。

## 未解决事项

- `0x71a29ccc` 虚表的接口 IID 与名称未确认（槽位序仅与 `IPersistStream` 吻合）；对象流如何从该接口分派读写未闭合。
- 保存方法中 `pe_hvcoordsys_to_string` 与 `_ext` 的选择条件基于未解析寄存器，条件来源未闭合；读取方法 `0x719babf0`、`0x719bae60`、`0x719ea140` 等域读写子函数未展开。
- 读取方法 4 字节长度未见缓冲上限检查（反编译视图），是否真实缺陷未核实；`Ordinal_149` 导入的归属模块未确认。
- 7 个域 double 与 3 个尾随 double 的语义未解释；`DAT_71a66f20` 缺省值未读取。
- Map 对象体与图层对象体触发空间参考子对象写出的调用点未定位；Map 级范围字段未定位。`{34B2EF83-F4AC-11D1-A245-080009B6F22B}`、`{D661941C-DA8A-11d3-9F60-00C04F6BDD7F}` 的类身份未确认；GEOGTRAN 文本的写出者未定位。[更新：Map 级范围字段已由写方向轮闭合（`0x24f64`）；`{34B2EF83}` 经第六轮与 AnnoLayer GetClassID `0x6b195e60` 证实为 FDOGraphicsLayer；`{D661941C}` 仍开放。]
- 两个 `pe.dll` 条目（10.1.0.679）与 Desktop 10.1.0.3035 的组件版本关系未核对。
- 本轮未运行 ArcMap；10.1 源工程在 ArcMap 中与转换结果在 QGIS 中的人工目视检查均未进行。上述布局不构成转换能力，也不得声称已经验证。
