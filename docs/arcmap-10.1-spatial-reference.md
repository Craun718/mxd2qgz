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

## 与 10.8 记录的对照

Git 历史 `c17b324:docs/maps-analysis.md` 记录 10.8 历史样本 `/Maps:0x46..0x4a` 为 4 字节长度 455、`0x4a` 起 ASCII `PROJCS[`；本轮 10.1 参考样本在完全相同的偏移观察到相同值。两样本是不同文件，偏移一致只说明首 Map 对象体中空间参考子对象的序列化位置稳定，不能将 10.8 偏移迁移到其他 10.1 文件。本轮原生证据来自 10.1.0.3035 的 Geometry.dll，未与 10.8 DLL 做函数级对照。

## 未解决事项

- `0x71a29ccc` 虚表的接口 IID 与名称未确认（槽位序仅与 `IPersistStream` 吻合）；对象流如何从该接口分派读写未闭合。
- 保存方法中 `pe_hvcoordsys_to_string` 与 `_ext` 的选择条件基于未解析寄存器，条件来源未闭合；读取方法 `0x719babf0`、`0x719bae60`、`0x719ea140` 等域读写子函数未展开。
- 读取方法 4 字节长度未见缓冲上限检查（反编译视图），是否真实缺陷未核实；`Ordinal_149` 导入的归属模块未确认。
- 7 个域 double 与 3 个尾随 double 的语义未解释；`DAT_71a66f20` 缺省值未读取。
- Map 对象体与图层对象体触发空间参考子对象写出的调用点未定位；Map 级范围字段未定位。`{34B2EF83-F4AC-11D1-A245-080009B6F22B}`、`{D661941C-DA8A-11d3-9F60-00C04F6BDD7F}` 的类身份未确认；GEOGTRAN 文本的写出者未定位。
- 两个 `pe.dll` 条目（10.1.0.679）与 Desktop 10.1.0.3035 的组件版本关系未核对。
- 本轮未运行 ArcMap；10.1 源工程在 ArcMap 中与转换结果在 QGIS 中的人工目视检查均未进行。上述布局不构成转换能力，也不得声称已经验证。
