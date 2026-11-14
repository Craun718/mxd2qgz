# MXD 到 QGZ 转换 TODO

本项目采用零样本 REA 逆向，以目标软件的实现及运行行为为研究依据；可控样本如可获得，仅作为辅助证据。功能和转换结果由人工在 ArcMap、QGIS 中目视检查。

当前适配目标为 **ArcMap 10.1**，与旧版软件一致。安装介质及配置声明见 [README 适配目标](README.md#适配目标)。下列已完成的 10.8 研究与读取器仍仅适用于原标注版本，不表示已支持 10.1。

## 10.1 目标对齐（优先）

- [x] 明确 ArcMap 10.1 为适配基准，记录本地安装介质及 `Setup.ini` 声明的 `10.1.3035`。
- [x] 从 10.1 安装介质提取原生研究文件，记录文件版本、身份、摘要及来源；定位 PageLayout、SystemCore、AfCore、MappingCore 和 MappingCoreLib，见 [10.1 原生文件基线](docs/arcmap-10.1-native-baseline.md)。CAB 条目已复核，MSI 组件和安装目录映射仍未知。
- [x] 用 REA 定位类工厂、版本辅助函数和 PageLayout 字符串引用入口；尚未证明 10.1 完整调用链或读取规则。
- [x] 使用 REA 追踪 10.1 文档打开、Version 读写及 ObjectStream 持久化调用链，逐项复核现有 10.8 结论，记录相同点、差异和未知项；各链已闭合至可记录的静态边界，遗留开放项（MFC 命令层上游分发、`+0x14c` 版本索引运行时来源、激活表工厂指针写入者、`558c4e63` 类名、EH 展开动态验证等）见各子项与研究记录，全部未经 ArcMap 10.1 运行验证。
  - [x] 核对 10.1 PageLayout 的 Version 局部读取、一个静态调用者及独立构建号流；与 10.8 对照发现默认输出差异，完整调用链和格式仍未闭合，见 [原生文件基线](docs/arcmap-10.1-native-baseline.md#101-version-局部读取和构建号2026-11-13)。
  - [x] 核对 10.1 AfCore 的 String 读写导出及其局部短读行为，区分另一组 ReadString/WriteString；Version 流对象转发和写入调用者仍待确认。
  - [x] 闭合文档打开与保存调用点：MapDocument 类工厂链（CLSID `2fe5e68e-3da5-4bda-baf1-103c55115a1c`，运行期构建类表）与两条静态打开链汇合于主读取器 `0x7b708d00`；ArcMap 命令层经同一 CLSID 跨模块创建实例，MFC 路径串接 Version 头读取 `0x4764d0`；SaveObject/LoadObject 为 AfCore 跨模块导出，对象头写出仍在 AfCore 内部未闭合，见 [研究记录](docs/arcmap-10.1-document-open-save.md)。
  - [x] 实现显式 10.1 AfCore String 普通写出布局的严格读取子集，并覆盖构造输入和异常；未连接 MXD 字段，原生短读与磁盘流转发未验证。
  - [x] 闭合 AfCore String 读写的流转发与短读边界：两函数直接走调用方 IStream 虚表（读 +0x0c、写 +0x10），短读不检查完成量与终止零；确认两族导出格式不可混用，10.1 未见 10.8 记录的 cb 掩码（待复核），见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-afcore-string-流转发与写入调用者2026-11-14)。
  - [x] 闭合 Version 流对象身份：CLSID `043731d0` 经 AfCore `AfCreateInstance`→FactoryCache/CoCreateInstance 创建，+0x38 为底层流连接槽位；复核 `0x7b708d00` 主干（Mx Document/Maps/PageLayout 存储及四个接口 GUID），与 [文档打开链](docs/arcmap-10.1-document-open-save.md) 汇合，见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-version-流对象身份与文档打开链2026-11-14)。各 GUID 语义、对象错误恢复与 Version 写入路径的闭合见后续条目。
  - [x] 闭合各 GUID 语义（类型库 `LoadTypeLibEx` 解析 + 主干行为双证据，纠正此前手工解析的 IVariantStreamIO/IPageLayout 误判）：`043731d0` 为 esriSystem coclass **ObjectStream**（主接口 IObjectStream `18a45ba7`），主干四接口为 IDocumentVersion/IPageLayout/IActiveView 与打印设置分支 IPaper/IPrinter，另证 Array 收集容器与未登记候选 `558c4e63`（对外提供 IPaper 的内部组件类，类名未闭合）；仅闭合 IID 语义，各对象体字段布局不在其内，见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-version-流对象接口-guid-语义考证2026-11-14)。
  - [x] 闭合 Version 写入路径与触发链：写入器 `0x466fb0` 的唯一直接调用者为 MFC `CDocument::Serialize` 覆写 `0x47a740`，Save/SaveAs 共用同一路径，读写经同一函数按 CArchive 方向对称分发；上游 MFC 命令层分发、`+0x14c` 版本索引运行时来源与 4 字节回填取值仍开放，见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-version-写入路径与写入调用者2026-11-14)。
  - [x] 闭合对象读取错误恢复：主读取器 `0x7b708d00` 全部失败分支汇入收尾块（逆序释放、输出参数仅成功时写），引用表不回滚、残留键由 ObjectStream 析构统一释放，任何读取失败使文档打开整体失败、无跳过坏对象分支；EH 展开为静态推断，见 [研究记录](docs/arcmap-10.1-object-body.md#101-对象读取错误恢复分支2026-11-14)。
- [x] 10.1 SystemCore 已定位接口 GUID 的查询和匹配分支；继续闭合类工厂、构造后虚表和读流转发。
  - [x] 确认 ObjectStream 构造后虚表、局部读流转发、内存版本属性和对象头加载分支；类表映射、Version 流实例身份和对象体仍未闭合，见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-objectstream-构造转发和局部对象头2026-11-13)。
  - [x] 实现显式 10.1 严格对象头读取器并测试截断、未知类和键；不接入 MXD，不模拟原生短读。
  - [x] 闭合 DllGetClassObject 类表遍历（静态工厂槽全零、运行时登记框架）、ObjectStream 虚表读转发槽位及 +0x40 对象写出器（GUID→查表→键→IPersistStream/VariantStream 分派，与读分支对称）；类描述表的运行时消费者已闭合于 SystemCore 内部（激活表 13 处访问者：查表分发、DllCanUnloadNow 探测、CRT 清理；并区分 0x800a4290 组件类别登记表与 esriDllRegisterServer 的分工），激活表工厂指针的运行时写入者仍开放，见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-systemcore-类描述表运行时消费者2026-11-14)。
- [x] 依据 10.1 原生证据实现相应读取规则及正常、异常输入测试；经核对再复用现有实现，保持版本范围显式。
  - [x] 复核 VariantStream：10.8 严格子集的全部规则在 10.1 逐项成立（VARTYPE 表、字符串 n*2+2、对象分支不支持）；另发现独立的 Variant::Read/Write 类方法序列化器（字符串含终止零、支持更多 VARTYPE），字段归属未闭合前不得混用，见 [研究记录](docs/arcmap-10.1-variant-stream.md)。
  - [x] 按上述 10.1 证据实现 `/Maps` 根前缀读取器与 ProjectedCoordinateSystem 对象体读取器：独立实现、入口版本显式，负数与超限额拒绝、逐前缀截断与失败游标纳入健壮性覆盖，尾部值保留原始位模式；不解析对象体、未接入 MXD 解析；仓库参考样本逐字节吻合仅为辅助观察（详见「MXD 解析」节对应条目）。
- [x] 建立 35 步提交路线，明确原生证据、实现测试与人工验收的区别；核对基线 24 项测试及格式检查通过。

## MXD 解析

- [x] 添加固定版本的 ForestRight 研究子模块，核对托管 DLL 身份并定位原生运行库研究入口。

- [x] 使用 REA 分析版本读取逻辑，验证 `/Version` 中 `10.1` 候选及其余字段；磁盘版本映射闭合为 esriArcGISVersion 枚举（83/90/92/93/10/101 → short 10/13/14/16/18/19），样本文本 `10.1`+short 19 与索引 5（Current）吻合；运行时验证仍缺。
  - [x] 调查 10.8 MappingCore/MappingCoreLib 版本入口，沿 AfCore 核对版本辅助函数及局部保存调用；补记参考样本身份和版本流字节，这些函数尚未关联到 OLE `/Version`。
  - [x] 由 MapDocument 类/接口 GUID 定位原生 Open/GetVersionInfo，追踪实际版本流读写；核对长度、编码、终止符、尾部及错误分支，验证与 `10.1` 候选的关系。
    - [x] 从 10.8 注册清单定位 PageLayout.dll，闭合类工厂、QI、最终虚表及 Open/GetVersionInfo；区分 Build Number 与 Version 流。
    - [x] 定位 Version 局部读取及有符号 short 比较，记录未检查 HRESULT 的调用分支；尚未确认字符串编码、尾部或版本映射。
    - [x] 追踪 10.8 Save/SaveAs、`Mx Document Version Info` 写入器和主文档 ObjectStream 保存调用；`/Version` 写入者、short 与尾部来源仍未知。
    - [x] 核对 AfCore 版本 IID 的实现者及完整 Version 读写规则：属性来源闭合到 CMxDoc→AfDoc 内存字段（构造即 `MOV [edx+0x14c],5`，初值=Current），4 字节回填=对象流写表键计数（样本 2267 即该语义候选），读侧 3/5 分支=枚举 93/Current；剩余开放项（SaveAs UI→put 间接路径、本地化语言包文本、旧文档打开后索引是否改写）见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-磁盘版本映射属性来源与回填取值2026-11-14)。
      - [x] 解码版本 IID 并闭合至 SystemCore ObjectStream 的内存属性虚表；属性来源和磁盘版本映射仍未知。
      - [x] 在 10.1 复核 AfCore 查询的 IID 和 SystemCore 五个实现类的构造、虚表及属性访问，排查 PageLayout 的 String::Write 候选；证据与范围见 [研究记录](docs/arcmap-10.1-native-baseline.md#101-版本接口实现及保存分支复核2026-11-13)。
      - [x] 从 10.1 ArcMap.exe 定位 Version 写入与读取、六项 short 映射及条件计数字段；实现严格头部读写并覆盖合成边界，保留未知尾部和人工验证缺口，见 [后续研究](docs/arcmap-10.1-native-baseline.md#101-arcmap-version-头部读取与写入2026-11-13)。
- [x] 使用 REA 分析坐标系读取逻辑，验证 `/Maps` 中投影文本候选的长度和作用范围。
  - [x] 10.1 Geometry.dll 证实投影文本是 ProjectedCoordinateSystem 子对象的原始字节持久化（保存版本 short、4 字节长度+ASCII WKT、域字符串、标志与 double），不是 String::Write 布局；Map 级与图层级皆有，样本字节逐项吻合；读侧按版本分支取字段。接口 IID、标志与 double 语义仍开放，见 [研究记录](docs/arcmap-10.1-spatial-reference.md)。
  - [x] 实现显式 10.1 ProjectedCoordinateSystem 对象体读取器：按原生 Load 版本分支取字段（>2 域串、≥4 标志+7 double、≥5 再 3 double、>7 拒绝），WKT 强制 ASCII，域串与 double 原样保留；未接入 MXD 解析。
- [ ] 读取 OLE 容器中的 `/Version`、`/Maps` 等数据流并识别 MXD 版本。
  - [x] 实现显式 10.8 `/Version` 前缀的严格读取器，保留候选文本、有符号 short 位模式和尾部位置；不识别版本，写入规则和跨版本兼容性仍未知。
  - [x] 实现基于 10.1 头部规则（文本+short+条件计数）的版本识别候选和 `version` 命令；未知文本/short 保留为未识别候选，10.1 严格读取失败时回退显式 10.8 布局。静态映射（esriArcGISVersion 枚举+资源 ID 表）已闭合并接入命令：`version` 新增候选保存索引输出，文本与 short 须命中同一索引，索引 3（9.3）因英文资源文本异常不可由文本命中、未知文本不映射为 9.3；运行时或软件验证仍缺，用法见 README。
- [x] 使用 REA 分析对象的序列化和读取逻辑，确定对象边界、类型标识和引用关系；记录证据及适用版本。对象边界（各类自己的 Load/Save、无外部长度）、类型标识（16 字节 ClassID，写侧取 GetClassID 与请求 IID 无关）与引用关系（键回引编码）均已闭合；剩余开放项（SimpleSaveObject 委派链、键计数器构造初值等）见各记录。
  - [x] 核对 10.1 托管 IObjectStream 方法顺序，记录 LoadObject 和版本属性的 x86 槽位候选及跨版本限制。
  - [x] 在 Ghidra 12.1.4 中核对 10.8 接口 GUID、构造后的实际虚表及 IUnknown 转发；定位读流入口。
  - [x] 对照 10.8 原生读写分支，确认局部空头、类标识和键，提供显式版本的对象头读取器及合成边界测试；尚未接入 `/Maps`。
  - [x] 实现显式 10.8 `/Maps` 数量和首对象头的前缀读取器，拒绝异常数量并保留未知尾部；不解析对象体，不应用于 10.1。
  - [x] 追踪持久化接口、替换分支和错误分支，核对对象体边界、`/Maps` 起点及跨版本适用范围。
    - [x] 10.1 汇编级复核：对象体由各类自己的 Load/Save 读写、无外部长度字段（未知类不可跳过）；登记先于持久化、键重复替换分支存在、SetStream 不清表；`/Maps` 写侧（数量=数据框数、循环后 4 字节尾部值）与读侧闭合，与 10.8 结构一致，见 [研究记录](docs/arcmap-10.1-object-body.md)。
    - [x] 实现显式 10.1 `/Maps` 根前缀读取器（数量、首对象头、尾部原始值），覆盖全前缀截断、失败游标与限额；不解析对象体，未接入 MXD 解析。
    - [x] 从 PageLayout.dll 实际文档读取链确认分流格式的 `/Maps:0x00` 数量及 `0x04` 首对象入口；记录 Mx Document 优先分支和跨流复用 ObjectStream。对象体、引用表生命周期和版本适用性仍未闭环。
    - [x] 确认 SetStream 不重置引用表、对象体之前登记、析构释放、普通/VariantStream 分支的接口 GUID 及局部 HRESULT；缺失引用与新对象不能只靠键区分。完整对象体、替换、失败恢复及版本适用性仍未知。
  - [x] 用 REA 确认 10.8 VariantStream 已知 VARTYPE 的固定宽度分支和字符串 `n * 2 + 2` 布局；对象分支、对象体和引用表仍未闭环。
  - [x] 实现显式 10.8 Variant 前缀读取器，覆盖固定宽度位模式、字符串边界、失败游标和显式对象分支错误；未接入 MXD 解析。
- [ ] 解析长度前缀字符串、对象属性和对象引用，并验证 UTF-16LE、空字符串及非 ASCII 字符的处理。
  - [x] 用 REA 确认 10.8 AfCore String::Read/Write 的字节长度、终止零和不严格短读行为，区分原生行为与 Rust 安全子集。
  - [x] 实现显式 10.8 调用点的严格字符串读取器，覆盖空值、中文、代理对、非法编码、全部截断、资源限制和失败游标；未接入 MXD 字段解析。
  - [x] 接入已确认字段归属：Map 数据框名称与第二名称经 Map.dll Load 与样本字节闭合为 AfCore String 布局，接入 `native_string::read_string_10_1`（Map 前缀读取器）；对象属性其余字段与其他版本仍待闭合，Version 完整规则仍未知。
  - [x] 闭合 10.1 对象键分配与回引编码：键=ObjectStream `+0x2c` 4 字节单调计数器（新对象写后 +1、跨流共享键序）、回引=同构 20 字节头无体无专用标记、唯一判据=键是否已在本会话出现过、循环引用无需专门编码（登记先于持久化）、缺失引用与新对象不可区分（有意保留的原生语义）；实现 `object_refs` 条目分类器与登记表并接入 robustness 集中清单（13 入口），样本键 29 起点/`0x2d2c` 键 32 回引辅助对照吻合；委派链与替换分支触发场景仍开放，见 [研究记录](docs/arcmap-10.1-object-body.md#101-对象键分配与回引写侧闭合2026-11-14-第五轮)。
- [ ] 解析数据框、图层树、图层顺序、图层可见性和数据源连接信息。
  - [x] 复核 10.1 反编译产物，闭合 Map::Load 各步骤的请求 IID 与版本分支、GroupLayer 完整字段序（版本≤12、计数循环、可见性 <2 默认 0xffff、≥11 ExtraSafeReadObject 分支）、FeatureLayer 头部段（版本≤34、+0x144 原样 16 字节 GUID）；第 5/9 步元素类语义、FeatureLayer 后段与 Save 方向仍开放，见 [研究记录](docs/arcmap-10.1-map-object.md)。
  - [x] 实现显式 10.1 Map 对象体已确认前缀读取器（版本≤51、双名称、嵌套空间参考含 PCS 分派与空引用标记），样本「新建数据框」前缀逐字节吻合；第 5 步起的字段保留为未知尾部，等待字段序与图层体研究闭合后再扩展。
  - [x] 闭合 10.1 Map 图层列表与 Save 写侧：读侧 count + 每项[16 位标志 1/2/3 + ILayer 对象（2/3 带 8 字节体积前缀）] + 4 字节活动图层索引，写侧 `FUN_794033a0`（标志 1=SaveObject、2/3=ExtraSafeWriteObject 按 docVersion，10.1 写 2）；样本 `0x274` 起 6 条目逐字节闭合（FDOGraphicsLayer 标志 1 内嵌 FeatureLayer 键 32、回引用条目 A=20 无体、活动索引 -1 @`0x16fb4`）；ExtraSafeRead/WriteObject 8 字节前缀语义、CompositeGraphicsLayer::Load（版本≤6、11 元素组、名称、子图层循环 String+ILayer）与 Map::Save 主体顺序同步闭合；FeatureLayer::Load 中后段（数据源 FCN+IName、+0x144 GUID={6ADB9601-…}、3036D35D 计数循环）字段序闭合，见 [研究记录](docs/arcmap-10.1-map-object.md#mapsave-方向extrasafe-前缀与图层列表样本闭合2026-11-14-第四轮)。
  - [x] 闭合 `FE3115E9`/`EE53528B` 两循环与键 1619..2101 簇归属：`FE3115E9`=IMapSurround（第 5 步地图装饰物集合循环）、`EE53528B`=ISpatialBookmark（第 9 步空间书签集合循环），同批经类型库定名约 20 个请求 IID（IHyperlink/IEnvelope/IStandaloneTable/IGeoTransformation/IFeatureRenderer/IName 等，修正第三轮两处候选）；键簇归属闭合为 CGL-1553 注记子图层条目（[String 注记类名][FeatureLayer 完整体]×9 + 1 个对键 1792 的回引头），修正第四轮「计数 50」误读，元素 coclass 经 esri.clsid 全表逐头归名；两循环样本元素定性仍需含书签/装饰物的样本，见 [研究记录](docs/arcmap-10.1-map-object.md#mapsurround书签循环接口身份注记子图层簇修正与样本尾部2026-11-14-第五轮)。
- [ ] 解析地图坐标系、图层坐标系和显示范围。
  - [x] 用 REA 闭合 Map::Load 空间参考分支：请求 IID `{4434D680-…}`（ISpatialReference 候选）、空引用零 GUID 标记、PCS 类分派由流内类标识决定；补全后继序列 `FUN_794035f0`（ver≥0x17 计数+short 1/2/3+`{34C20002-…}` 对象+尾部索引）与旧版 `FUN_794038a0`（ver 0x11 拒绝、双集合布局），见 [研究记录](docs/arcmap-10.1-map-object.md)。
  - [x] 用样本字节闭合范围载体：Envelope 对象 `{30707212-…}` 体 = short 3 + 4 double `[XMin,YMin,XMax,YMax]` + int32 0 + 内嵌空间参考（PCS 体）或对 Map SR 的键回引；图层坐标系宿主于图层体 Envelope 第 4 段（样本 25/25 一致），修正此前「紧邻 32 字节」定位，见 [空间参考记录](docs/arcmap-10.1-spatial-reference.md)。
  - [x] 确认地图显示范围为 Map::Load 第 5 步循环后的原样 32 字节 WKSEnvelope（4 double，非对象，写入 +0x230 接口 +0xe0），随后两个 int32（-0x40 接口 +0x38/+0x40）；字段序 [XMin,YMin,XMax,YMax] 与样本流内定位此后经写方向取证升级为证据，见下方写方向条目。
  - [x] 写方向取证闭合 Envelope/PCS Save 与显示范围：Envelope::Save 恒写版本 3（short 3 + 32 字节 WKSEnvelope + int32 + bit6/7 追加 Z/M + SR 子对象）、Load 按 1/2/3 分支（==3 才 LoadObject SR，IID 与 Map 第 4 步同值）；字段序 [XMin,YMin,XMax,YMax] 由 AfCore 访问器逐指令绑定证实；PCS Save 恒写 v6 与读侧分支对称；Map::Save 显示范围写出链闭合（Map+0x298 助手 +0xdc 槽 → 原样 32 字节），样本定位 `0x24f64` 升级为确定（前后括号完整：第 5 段 MapSurround 与书签循环计数确认=0，20 字节空标记=16 零+键 0，对象头类标识修正为 `{1C352F40}`、`{1C352F3F}` 为请求 IID），见 [研究记录](docs/arcmap-10.1-spatial-reference.md#envelopesave-wksenvelope-写方向与-map-显示范围闭合2026-11-14-写方向取证轮)。
  - [ ] Envelope 两个实例的字段语义（数据全范围 vs 当前范围候选）、三个 Envelope（键 74/80/151）的 LoadObject 逐条归因仍开放：样本 11 层 +0x144 GUID 后固定 7 字节与 RTM 3035 Save/Load 反编译不可调和（工作假设：样本由某 10.1 服务包构建保存，`/Version` 文本不区分 SP，未验证），需 SP 介质对照样本或运行时取证；另开放 Map +0x298 助手类身份与 `{1C352F40}` 类名，见 [研究记录](docs/arcmap-10.1-spatial-reference.md#featuresave-写侧新证据归因仍开放)。
- [x] 处理未知对象、未知版本、截断数据、非法长度及循环引用；未知对象/版本与截断、非法长度由各读取器原样保留或明确拒绝，循环引用经 `object_refs` 状态机覆盖。
  - [x] 建立跨读取器健壮性测试：全部严格读取器入口的逐前缀截断、失败游标不变与重读一致、超大长度/限额溢出边界、未知类标识与键原样保留、VARTYPE 全空间穷举及非递归有界完成。仅验证读取器自身契约，不代表 MXD 格式规则已验证。
  - [x] 对象引用编码闭合后覆盖缺失引用、重复标识及循环引用：`object_refs` 单元与集成测试覆盖 A→B→A 循环、自引用、跨流共享键序（SetStream 不清表）、重复登记=替换、空表不可区分回引与新对象（原生语义）、截断失败不改登记表；样本键 29/32 辅助对照吻合。仅验证分类器自身契约，不证明完整对象解析已验证。

## 中间模型

- [x] 定义地图、数据框、图层组和图层的数据结构，保存层级、顺序和可见性。
- [x] 定义数据源、坐标系和显示范围的数据结构，保留无法识别的字段及其位置。
- [ ] 将 MXD 解析结果转换为中间模型，并结合 REA 证据核对图层树和属性；软件中的人工检查见下方人工验收。
  - [x] 新增 `parse` 命令（`mxd_maps` 模块）接入显式 10.1 读取器：`/Maps` 根前缀（数量、首对象头、尾部原始值）与 Map 对象体已确认前缀（版本 ≤51、双名称、嵌套 PCS）填充中间模型 `Map`/`DataFrame`（数据框名称、第二名称、WKT 定义）；无论数量多少最多解析首个对象——对象循环边界未闭合，其余数据框、图层、范围与已确认前缀之后的字节全部保留为未知字段并输出诊断，PCS 域字符串/标志/double 不写入模型。仓库参考样本前缀逐项吻合仅为辅助观察；单元测试覆盖合成正常与异常输入（截断、负数量、非 Map/非 PCS 类、限额、空头），仅验证代码自身契约，未在 ArcMap 10.1 或 QGIS 中人工目视验证。

## QGZ 生成

- [ ] 实现 `convert` 命令，读取 MXD 并输出 QGZ。
- [ ] 将中间模型映射为 QGIS 工程、图层树、图层顺序和可见性。
  - [x] 构造模型的独立 QGS/QGZ 写入器：XML 转义、确定性标识、ZIP、防覆盖、并发发布和失败清理；单框层级、树序、组/叶勾选通过 XML 和 QGIS API 核对。
- [ ] 转换 Shapefile 数据源路径，处理相对路径、绝对路径和缺失文件。
  - [x] 构造模型支持 Windows/UNC/POSIX 词法路径重定位，断链保留并诊断；不将异平台路径当成本机路径。尚未从 MXD 解析连接。
- [ ] 写入工程及图层坐标系、地图显示范围。
  - [x] 构造模型支持明确 EPSG:4326/3857、不同地图/图层 CRS 和主画布范围；其他定义保留并诊断，未猜 EPSG。非法或溢出范围拒绝。
- [ ] 生成可由 QGIS 打开的 QGZ，并验证图层能够加载。
- [ ] 支持图层组、多数据框及其他数据源的映射。

## 样式与布局

- [ ] 解析并转换常用矢量符号和渲染方式。
- [ ] 解析并转换栅格渲染方式。
- [ ] 解析并转换标注表达式、字体、位置和可见性。
- [ ] 解析并转换布局中的地图、图例、比例尺、指北针和文本。
- [ ] 对无法转换或只能近似转换的属性输出对象位置和具体原因。

## 自动验证

- [x] 独立输出层新增 14 项自动测试；总计 43 项通过，1 项 QGIS API 测试默认忽略，显式运行后通过。本机 QGIS 4.2.3 加载构造 QGS/QGZ：2 层有效、1 层断链保留；人工视觉仍未验证。

- [x] UTF-16LE 候选扫描保留合法代理对；覆盖两种对齐、孤立代理项、奇数字节尾和空输入。这仅验证扫描器，不代表 MXD 字符串规则已确定。

- [x] 覆盖通用 UTF-16LE 读取的中文、代理对、非法编码、显式字节长度及失败游标不变；覆盖受限子范围读取。此项不代表 MXD 字符串规则已经验证。

- [x] 覆盖 OLE 数据流读取以及字节范围的边界和溢出检查。
- [x] 覆盖 10.8 VariantStream 固定宽度、空/中文/代理对/嵌入 NUL 字符串、截断、限额、长度溢出、未知 VARTYPE 和对象分支；仅静态证据与合成测试，未做软件人工验证。
- [x] 使用合成 OLE 容器端到端验证五个诊断命令、中文文件名、导出防覆盖、缺失流、非法参数、越界范围及截断容器。此项不验证 MXD 格式或转换结果。
- [x] 覆盖显式 10.1 `/Maps` 根前缀读取器与 10.1 ProjectedCoordinateSystem 对象体读取器：全前缀逐字节截断、失败游标不变与重读一致、限额与超大长度、版本分支与拒绝边界、原始位模式保留；robustness 集中清单增至 11 个读取器入口。仅验证读取器自身契约，不代表 MXD 格式规则已验证。
- [x] 覆盖 10.1 Map 对象体前缀读取器：版本边界、双名称顺序与空名称 6 字节模式、空引用标记、未知嵌套类拒绝、逐字节截断扫描、限额边界与嵌套 PCS 限额转发；robustness 集中清单增至 12 个入口。仅验证读取器自身契约，不代表 MXD 格式规则已验证。
- [x] 覆盖 `parse` 命令接线（`mxd_maps`）：模型填充（双名称、WKT）、数量 0/正数、全零空头、非 Map/非 PCS 类拒绝、已确认前缀内逐字节截断整体失败与尾部区域截断缩短、负数量与限额拒绝，以及合成 OLE 容器端到端（含 /Version 缺失诊断与负数量错误）。仅验证接线自身契约，不代表 MXD 格式规则已验证，未在目标软件中人工验证。
- [ ] 为二进制读取、对象引用、字符串解码和异常输入添加测试。
  - [x] 覆盖整数及浮点读取的全部截断长度、失败后重试、浮点特殊位模式和嵌套范围隔离；遍历全部 UTF-16 单码元及补充平面边界，核对非法代理项和非零偏移错误位置。
  - [x] 在确定对象引用编码后，覆盖缺失引用、重复标识及循环引用：编码已闭合（键回引按已见键集合判定），`object_refs` 覆盖缺失引用（未见过键即新对象，不可区分系原生语义）、重复标识（重复登记=替换）及循环引用（A→B→A、自引用、跨流共享键序），并接入 robustness 截断共性清单；仅验证分类器契约，不代表 MXD 对象解析已验证。
- [ ] 为数据源路径解析、坐标系和图层树映射添加测试。

## 人工验收

- [ ] 在 ArcMap 10.1 中检查源工程，并与 QGIS 输出对比，记录实际软件版本、检查范围和结果；当前未验证。
- [ ] 在 ArcMap 中查看源工程，在 QGIS 中打开转换结果，目视核对图层、坐标系、范围和数据加载结果，并核对中间模型的图层树和属性。
- [ ] 对比 ArcMap 与 QGIS 的渲染结果，逐项目视检查符号、标注和布局差异，记录软件版本及检查结果。
