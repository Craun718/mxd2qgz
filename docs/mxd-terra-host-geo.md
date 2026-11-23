# ArcMap 10.1 坐标变换类工厂与共同对象体（2026-11-17）

接续 [Map 坐标变换记录](mxd-terra-chain-map.md)。本文只讨论静态证据和独立读取器；没有编辑 Map、TODO，也没有将新类接入宿主。

## 文件身份与取证方式

本轮重算 `temp/arcgis101-bin/geometrydll/Geometry.dll.F84366F9_C0B7_4267_A16E_D48D9E6DD261` SHA-256 为 `c57d154f207ee249adc69136574cf274771370ff93a15eb0be5682ce828f0d47`。PE machine `0x14c`、ImageBase `0x717e0000`，版本资源 FileVersion/ProductVersion `10.1.0.3035`、InternalName `Geometry`、OriginalFilename `Geometry.DLL`，与既有基线相符；版本自述与字节摘要不证明签名。以下地址均为静态 VA。

前轮独立 Ghidra 桥接工具 6.9.0 CLI 使用 JDK 25 时 Ghidra 12.2 的 Unix socket 绑定失败（`Operation not permitted`），见前述记录；本轮未切换共享 Ghidra 桥接工具。将前轮 Geometry 工程复制到 `temp/terra-host-geo/project`，以隔离 pyghidra/Ghidra 12.2、JDK 25 和仓库内独立临时/配置/缓存目录查询虚表及反编译，并以原 PE 的 objdump 指令核对构造。伪代码不是原始源码，私有分析产物在忽略目录，分享前需检查路径。

## 类工厂、构造和持久化槽

| 证据 | GeoTransformation | NADCONTransformation |
| --- | --- | --- |
| RTTI 类工厂虚表 | `0x71a1e06c` | `0x71a1dcfc` |
| 工厂 `+0x0c` 槽 | `0x718c7ff0` | `0x718c4c90` |
| 工厂创建路径构造 | `0x719c61e0` | `0x719c9020` |
| 持久化虚表 | `0x71a274f4` | `0x71a2894c` |
| GetClassID `+0x0c` | `0x719c40b0` | `0x719c4480` |
| Load `+0x14` | `0x719c5d40` | `0x719c5d40` |
| Save `+0x18` | `0x719c5ef0` | `0x719c5ef0` |

新反编译显示两个工厂各分配 `0x60` 字节并分别调用上表构造函数；GeoTransformation 构造 `0x719c626f` 明确将 `0x71a274f4` 写到对象 `+0x18`。新核对虚表指针，以及 `0x719c40b0` 写出的 16 字节，得到 GeoTransformation CLSID `{843C4D40-BD7E-4542-A126-24AEBA43AF20}`；NADCON 的 CLSID 为 `{D661941C-DA8A-11D3-9F60-00C04F6BDD7F}`。两个不同类的持久化 Load/Save 槽为同一函数，因而可以共享其对象体读取规则；这不等于证明其他变换类也共享。

共同 Load 顺序为 2 字节版本、4 字节长度、指定长度的原始文本字节；其处理细节与 Save 的 `pe_geogtran_to_string` 路径见 [前轮记录](mxd-terra-chain-map.md)。原生按有符号 short 比较版本是否大于 2，并有短读相关行为；本读取器只接受无符号 0..=2，限制长度不超过调用者限额且小于 3072，截断报错并回退游标，均属于有意更严格的策略。它不验证或转换变换文本，不执行坐标计算；不宣称原生对长度作相同校验。

## 独立接口和交接边界

`src/objects/geo_transformation.rs` 提供两个 CLSID 及 `read_geo_transformation_body_10_1`：仅接收已确认的两个类，调用者负责解析对象头、维护引用表和决定何时调用。读取失败不推进游标，成功只消费对象体；未知类、版本、超限和截断均报错。`src/objects/mod.rs` 暴露模块；现有 Map 的 NADCON 实现和调用均未改，主代理可按需要替换或接线，注意引用表及对象头事务性由宿主负责。

可独立落账：**GeoTransformation 类工厂、构造、CLSID 和持久化虚表对位；该类与 NADCON 共用同一 10.1 Load/Save 的静态证据；两类严格对象体读取器及合成边界测试。**不能勾选全部 GeoTransformation 类、宿主接线、真实 GeoTransformation 样本、文本语义、坐标运算或完整 Map 尾部。ArcMap 10.1 环境暂缺，未运行目标软件或在 ArcMap/QGIS 中打开并目视验收；静态证据及合成测试不能替代运行验证。

验证：`cargo test --offline --test terra_host_geo` 的 3 项通过，覆盖两类、版本、原始字节、尾随字节、空值、长度上界、未知类及各截断位置的游标回退；`cargo test --offline` 通过（默认忽略项未运行），本轮新增 Rust 文件的 `rustfmt --check` 和 `git diff --check` 通过。全仓 `cargo fmt --check` 未通过，差异在并行修改的 `tests/terra_integrate_map.rs`，本轮未改该文件。测试均使用仓库内专用 TMPDIR。

主审接线：Map 坐标变换循环改用本模块，接受两类新对象及已见键回引。主体按原始字节保留，另以构造宿主测试验证第二类不早停；样例实档只观察到 NADCON。

## Geometry 类型库与派生变换

复核 10.1.0.3035 的 Geometry.dll 与同介质 esriGeometry.olb（见
[类型库摘要](mxd-10.1-geodata-xform.md#coordinatexform-操作集)）。
沿类型库 RefTab 和接口继承可得到 18 个 ITransformation coclass。
其中 14 个的 RTTI IPersistStream 表均引用 Load `0x719c5d40`，
包含上文三类和下表 11 类。逐项将 GetClassID 读取的映像常量与介质注册表
及类型库三方对位。Save 虽有若干覆盖，逐项核对的 Ghidra 导出保持
「short 版本、int32 长度、原始文本」的流布局。

| 类 | IPersistStream 表 | GetClassID | Save |
| --- | --- | --- | --- |
| GeocentricTranslation | `0x71a2768c` | `0x719c4110` | `0x719c5ef0` |
| PositionVectorTransformation | `0x71a27a4c` | `0x719c41e0` | `0x719c5ef0` |
| MolodenskyTransformation | `0x71a27c2c` | `0x719c4240` | `0x719c5ef0` |
| MolodenskyBadekasTransformation | `0x71a27e0c` | `0x719c4330` | `0x719c7fd0` |
| NullTransformation | `0x71a28004` | `0x719c45d0` | `0x719ca000` |
| UnitChangeTransformation | `0x71a281dc` | `0x719c4570` | `0x719ca600` |
| Geographic2DOffsetTransformation | `0x71a283b4` | `0x719c4630` | `0x719cac70` |
| AbridgedMolodenskyTransformation | `0x71a28594` | `0x719c42b0` | `0x719c5ef0` |
| LongitudeRotationTransformation | `0x71a28774` | `0x719c4510` | `0x719c5ef0` |
| HARNTransformation | `0x71a28ae4` | `0x719c4390` | `0x719c5ef0` |
| NTv2Transformation | `0x71a28c7c` | `0x719c43f0` | `0x719c5ef0` |

CompositeGeoTransformation `{5847FB82-DBAE-11D3-9F60-00C04F6BDD7F}`
的 IPersistStream 表在 `0x71a28e18`。GetClassID `0x719c46a0`，
Load `0x719cb0e0`，Save `0x719cb300`。补充反编译后，以原始指令复核
short 版本、short 名称字节数、UTF-16LE 名称、short 数量，以及逐项
「short 方向、16 字节 CLSID、对象体」。名称的 Load 在内存追加终止零，
AfCore 的宽字符重载 `0x68a84830` / `0x68a84850` 直接转发字节数给 IStream。
Load `0x719cb25b` 按 CLSID 创建 IGeoTransformation，
`0x719cb2af` 直接调 IPersistStream::Load，组合成员省略对象流键。

Geometry 类型库剩余的 AffineTransformation2D、AffineTransformation3D 和
ProjectiveTransformation2D 的持久化表 `0x71a117e0`、`0x71a11c80`、
`0x71a23588` 中，Load 和 Save 均指向 `0x718ad650`（E_NOTIMPL）。
另查 Adjustment.dll 的 PiecewiseTransformation，QI `0x1000b490` 识别
ITransformation，委派 `0x100177e0`。两段 QI 均无二进制持久化接口。
其他模块的实现类仍需逐项核对。

Map 列表和 GeometricXform 的对象分支共用读取器。组合名称和成员计数受限，
组合嵌套最多 16 层，解析失败回到调用前的对象槽。合成测试覆盖 Map 无键
组合成员与外壳回引、GeometricXform 对象后的 double、组合截断和深度限额。
参考工程和 66 个介质 /Maps 流均未命中新接入的 12 个 CLSID，未做 ArcMap
或 QGIS 人工验收。

## GEOCON 与 NADCON5 的注册和工厂表

10.1 介质的 `esri.clsid.ecfg` 将 GEOCONTransformation
（`66D01872-722C-47D0-AAFD-C83CB3612023`）和 NADCON5Transformation
（`96B3AF79-0300-4EF5-8AEA-3724C17BD0B3`）注册到 Geometry.dll。
复核本机研究目录同一 SHA-256 的 Geometry.dll 10.1.0.3035。导出
DllGetClassObject `0x718bd8f0` 转到 `0x719f17f0`，从
`0x71a65b60` 以 `0x34` 字节步长逐项读取 CLSID 指针，
使用 `0x717e4ed0` 比较完整 16 字节，指针为空即结束并返回
`CLASS_E_CLASSNOTAVAILABLE`。

按 PE 中的首项、初始化指令 `0x71a09b00..0x71a0be56` 以及
`0x71a0becc` 写入的空终止槽重建 78 个 CLSID。表中可对位
CoordinateFrameTransformation、NADCONTransformation、
HARNTransformation 和 CompositeGeoTransformation。GEOCONTransformation、
NADCON5Transformation 以及同模块注册的 HVDatumTransformation、
HVCompositeDatumTransformation 均无对应表项。参考 MXD 及两套介质的
134 个 `/Maps` 流也未命中 GEOCON 或 NADCON5 的 CLSID。
静态结论限于该映像的工厂表初始化及导出路径，其他运行期工厂和二进制
对象体仍缺证据，读取器继续保留未知。

### 10.1 安装介质的接口实现清点

以 IID `9273BB60-C611-11D2-BD08-0000F875BCCE` 为起点，扫描同一套 10.1 Desktop 介质中的 72 个可读 MSFT 类型库，跨库合并 GUID，按接口继承和 coclass RefTab 枚举实现。得到 14 个公开 coclass，均位于 esriGeometry，且均已接入共同文本体或组合变换体。另有只在原生工厂中定位的通用 `GeoTransformation`，也已接入。AffineTransformation2D、AffineTransformation3D、ProjectiveTransformation2D 是一般 `ITransformation` 的实现，Map 的请求 IID 为 `IGeoTransformation`。

对同一提取目录的 1203 个 PE 文件搜索两个未匹配注册项的 CLSID 原始字节及 ASCII 类名。`GEOCONTransformation` 和 `NADCON5Transformation` 均无命中，与 Geometry.dll 导出工厂及类型库的结果一致。安装配置中的注册声明仍在。此结果覆盖已提取的介质映像和公开类清单，不能排除补丁、额外组件、运行时注册或应用自有的私有变换类。未知对象仍停在其头部，由后续证据决定如何接入。

继续核对本机 10.1 安装介质的 CAB 条目。Desktop `Setup1.cab` 对
`geocon`/`nadcon`/`datum`/`transformation` 的命中只有
`geographic_transformations.pdf`、`gdal_datum.csv`、`gt_datum.csv`
和已接入的 `RasterTransformationOp.dll`；`Setup2.cab` 无相关条目。
DataInterop `Setup1.cab` 的命中是 FME 的 `geoconcept.db`、
`geoconcept.dll`、备份格式和 datum 字典，不包含 ArcGIS 的
GEOCONTransformation 或 NADCON5Transformation 实现。

因此，在目标 ArcGIS 10.1 Desktop 安装介质范围内，这两个 CLSID 只有
注册声明，没有可定位的持久化类实现；解析器继续把它们作为未知对象体
停止。补丁、独立扩展或第三方运行期注册若提供真实实现，仍需以其二进制
和对象流宿主另立证据后接入。
