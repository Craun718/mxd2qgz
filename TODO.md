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
- [x] 读取 OLE 容器中的 `/Version`、`/Maps` 等数据流并识别 MXD 版本。
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
- [x] 解析长度前缀字符串、对象属性和对象引用，并验证 UTF-16LE、空字符串及非 ASCII 字符的处理。
  - [x] 用 REA 确认 10.8 AfCore String::Read/Write 的字节长度、终止零和不严格短读行为，区分原生行为与 Rust 安全子集。
  - [x] 实现显式 10.8 调用点的严格字符串读取器，覆盖空值、中文、代理对、非法编码、全部截断、资源限制和失败游标；未接入 MXD 字段解析。
  - [x] 接入已确认字段归属：Map 数据框名称与第二名称经 Map.dll Load 与样本字节闭合为 AfCore String 布局，接入 `native_string::read_string_10_1`（Map 前缀读取器）；对象属性其余字段与其他版本仍待闭合，Version 完整规则仍未知。
  - [x] 闭合 10.1 对象键分配与回引编码：键=ObjectStream `+0x2c` 4 字节单调计数器（新对象写后 +1、跨流共享键序）、回引=同构 20 字节头无体无专用标记、唯一判据=键是否已在本会话出现过、循环引用无需专门编码（登记先于持久化）、缺失引用与新对象不可区分（有意保留的原生语义）；实现 `object_refs` 条目分类器与登记表并接入 robustness 集中清单（13 入口），样本键 29 起点/`0x2d2c` 键 32 回引辅助对照吻合；委派链与替换分支触发场景仍开放，见 [研究记录](docs/arcmap-10.1-object-body.md#101-对象键分配与回引写侧闭合2026-11-14-第五轮)。
  - [x] 实现显式 10.1 FeatureLayer 对象体已确认前缀读取器（`feature_layer`：版本≤34、双名称、VARIANT_BOOL 可见性（非规范位模式原样保留）、数据源 FeatureClassName 前缀 = short+要素类名；FCN 中段与 `+0x144` 之后保留未知），经 `layer_list` 类分派接入对象属性解析与中间模型；样本首 FeatureLayer（键 32）前缀逐字段吻合（辅助观察），13 项单元测试。第六轮扩展：FCN 体（GdbCore.dll `0x709b5080`）与嵌套 WorkspaceName 体（`0x70a27950`）闭合，渲染器槽（IFeatureRenderer 新对象即停止，空/回引读 `+0x144` 原样 GUID 后停止）；第四轮「+0x144 GUID @0x3d6」系误读（实为 WS 体的 ShapefileWorkspaceFactory CLSID `{A06ADB96-…}`），真 `+0x144` GUID 在渲染器槽之后；样本 `0x30a..0x3ed` 逐字段吻合，见研究记录第六轮。
- [x] 解析数据框、图层树、图层顺序、图层可见性和数据源连接信息。
  - [x] 复核 10.1 反编译产物，闭合 Map::Load 各步骤的请求 IID 与版本分支、GroupLayer 完整字段序（版本≤12、计数循环、可见性 <2 默认 0xffff、≥11 ExtraSafeReadObject 分支）、FeatureLayer 头部段（版本≤34、+0x144 原样 16 字节 GUID）；第 5/9 步元素类语义、FeatureLayer 后段与 Save 方向仍开放，见 [研究记录](docs/arcmap-10.1-map-object.md)。
  - [x] 实现显式 10.1 Map 对象体已确认前缀读取器（版本≤51、双名称、嵌套空间参考含 PCS 分派与空引用标记），样本「新建数据框」前缀逐字节吻合；第 5 步起的字段保留为未知尾部，等待字段序与图层体研究闭合后再扩展。
  - [x] 闭合 10.1 Map 图层列表与 Save 写侧：读侧 count + 每项[16 位标志 1/2/3 + ILayer 对象（2/3 带 8 字节体积前缀）] + 4 字节活动图层索引，写侧 `FUN_794033a0`（标志 1=SaveObject、2/3=ExtraSafeWriteObject 按 docVersion，10.1 写 2）；样本 `0x274` 起 6 条目逐字节闭合（FDOGraphicsLayer 标志 1 内嵌 FeatureLayer 键 32、回引用条目 A=20 无体、活动索引 -1 @`0x16fb4`）；ExtraSafeRead/WriteObject 8 字节前缀语义、CompositeGraphicsLayer::Load（版本≤6、11 元素组、名称、子图层循环 String+ILayer）与 Map::Save 主体顺序同步闭合；FeatureLayer::Load 中后段（数据源 FCN+IName、+0x144 GUID={6ADB9601-…}、3036D35D 计数循环）字段序闭合，见 [研究记录](docs/arcmap-10.1-map-object.md#mapsave-方向extrasafe-前缀与图层列表样本闭合2026-11-14-第四轮)。
  - [x] 闭合 `FE3115E9`/`EE53528B` 两循环与键 1619..2101 簇归属：`FE3115E9`=IMapSurround（第 5 步地图装饰物集合循环）、`EE53528B`=ISpatialBookmark（第 9 步空间书签集合循环），同批经类型库定名约 20 个请求 IID（IHyperlink/IEnvelope/IStandaloneTable/IGeoTransformation/IFeatureRenderer/IName 等，修正第三轮两处候选）；键簇归属闭合为 CGL-1553 注记子图层条目（`[String 注记类名][FeatureLayer 完整体]`×9 + 1 个对键 1792 的回引头），修正第四轮「计数 50」误读，元素 coclass 经 esri.clsid 全表逐头归名；两循环样本元素定性仍需含书签/装饰物的样本，见 [研究记录](docs/arcmap-10.1-map-object.md#mapsurround书签循环接口身份注记子图层簇修正与样本尾部2026-11-14-第五轮)。
  - [x] 实现显式 10.1 图层列表读取器（`layer_list`：计数、标志 1/2/3、ExtraSafe `[A][B]` 前缀、B 原生同步与 A 界观察推定跳过、`object_refs` 分类、活动索引）与 Map 体后缀读取器（`map_object::read_map_suffix_10_1`：基本图形层槽位、MapSurround 循环、WKSEnvelope、两个 int32、IEnvelope 槽位、书签循环，到书签循环为止），接入 `parse` 与中间模型：条目顺序 → children（回引命中已解析键时复用同对象节点）、WKSEnvelope → extent。仓库样本在条目 1（FDOGraphicsLayer 标志 1，AnnoLayer.dll 未反编译）按原生语义停止（体无外部长度不可跳过）；条目 2..6 的 A 界经合成前导 6/6 辅助吻合——条目 4 A=3318 与活动索引 @`0x16fb6` 系研究文档笔误，以样本字节为准；跳过体键未登记对回引判定的影响已在模块文档说明（A/B 界保证对位不受影响）。第六轮更正：AnnoLayer.dll 已反编译，FDOGraphicsLayer 经类分派接入（新 `fdo_graphics_layer` 前缀读取器）并实现组图层子图层递归，样本条目 1 现解析 FDO 前缀与关联 FeatureLayer 体，停止点推进至关联 FeatureLayer 的渲染器槽位（SimpleRenderer 新对象 @`0x3ed`）。
  - [x] 实现显式 10.1 GroupLayer / CompositeGraphicsLayer 已确认前缀读取器（`group_layer`/`composite_graphics`，前缀停止语义：嵌套新对象/ExtensionHelper/未展开尾段处停止并保留剩余，失败事务性回滚登记表）；字段序按反编译一手证据（`/tmp/r3` 产物）修正研究文档表格三处出入（GroupLayer 嵌套对象门 `2 < ver`、CGL 第 6 步三个 int32、CGL 子图层条目尾段字段），样本 CGL 体 0x16fce 起逐段吻合（辅助观察）；样本无组图层，GroupLayer 版本分支无字节佐证，23 项单元测试。第六轮实现组图层子图层递归（<11 普通条目解析至前缀停止点、≥11 ExtraSafe 条目经类分派解析后按 A 界跳过，深度限额 64 为防栈溢出非原生限制），裁决 CGL 子图层计数位于 `0x17056`=10（`0x17052` 为版本>1 的 int32；10 条目 = 9 新 + 1 回引，键簇 1619..2101 逐条复核）。
  - [x] 第七轮闭合五组开放点并接入分派链：渲染器链第一环（`feature_layer_tail`：SimpleRenderer `{F3435801}` 体 + SimpleMarkerSymbol/RgbColor/LegendGroup/LegendClass，样本 `0x3ed..0x602` 逐字段吻合，接入渲染器槽分派）、FCN 的 IDatasetName/ITopologyName 新对象体（`dataset_name`：FeatureDatasetName `{198846CF}`/TopologyName `{9FCA50EC}`，样本 11 个 FCN 选择子全 0、无字节佐证，接入 FCN 选择子分派）、ElementCollection 体（`element_collection`：Layer.dll RTTI→COL→虚表，Load `0x7865a340` 版本 <4 三循环 / ==4 交错布局，样本 12 实例全版本 4，接入 FDO 第 5 步分派）、FDO 开放点（`fdo_legacy`：<4 旧加载器 20 字段序与元素循环类 FDOGraphicsSublayer `{4ABA3851}` 均无字节佐证；版本 4 探测因整流绝对位置依赖不实现；委派子对象闭合为聚合 FeatureLayer `{E663A651}` 机制解释）、MapSurround/书签循环类族（`map_surround`：AOI/Feature 书签体完全闭合、Legend 已确认前缀，样本计数 0 无字节佐证，接入 Map 后缀循环分派）；robustness 集中清单增至 28 个入口，见各轮研究记录（docs/arcmap-10.1-featurelayer-tail.md、-fdo-open-points.md、-element-collection.md、-dataset-name.md、-mapsurround-bookmark.md）。
  - [x] 并行五轮闭合 FL 后继段与尾段链族：FL::Load 后继段 10 步至 FieldInfo 循环（IFieldInfo `{A2BAAE2C}`；RTM 3035「不可调和差异」裁决为误读并撤销，样本 0x5b0..0xafe 逐字节零分歧，停止点 0x5c0→0xafe）+ FieldInfo/NumericFormat 体；DisplayCore 十类符号与颜色（SimpleLine/Fill、MarkerLine/CartographicLine/HashLine、Cmyk/Hsv/Hls/GrayColor 共享 Color 核心）；LegendClassFormat/HorizontalLegendItem（Renderers.dll 动态安装 IPersistStream 虚表）/EditTemplate（样本 10 处 CLSID 命中、键 1605 逐字段吻合）+ AfCore `Variant::Read` bool=true 分支；PropertySet（值序列化器=类方法 Variant::Read 而非 VariantStream，样本 2 实例）+ ElementProperties 前缀与 TextElement/TextSymbol/Marker/LineElement 体（TextElement 样本 16 实例对位，修正第五轮「内嵌 SimpleTextPath」误读为原样 GUID+空标记）；接线轮九处分派点接入、display 模块归入 objects 并消除 objects→layers 依赖倒置、修复 SMS 首色槽 key 阴影，robustness 集中清单现为 48 个入口，423 项测试，见各研究记录及其接线轮小节。
  - [x] 独立重构轮消除剩余两处 objects→layers 依赖倒置：WorkspaceName 体读取器自 `layers::feature_layer` 移入 `objects::workspace_name`（`dataset_name` 改引 objects 路径）、`class_id_hex` 自 `layers::layer_list` 移入 `storage`（`element_collection` 改调 storage 路径），Reader 新增 `reposition_to` 回退共用原语；423 项测试与样本停止点 0xafe 均不变。
  - [x] 第九轮闭合 FL 尾段 ALPC 环与后继段延续：ALPC（`{1D5849F3}` AnnotateLayerPropertiesCollection，LabelPlacement.dll `0x78531be0`）/LELP（`{01004145}`，`0x785468c0`）/EditTemplateManager/FeatureIDSet 四类体与后继段㉑-㊸剩余字段（含版本 0x1c..0x1f 分支与 TimeData 族四段）闭合至版本 >0x21 门 `short(+0x2ca)`；裁决 EditTemplate 宿主=后继段图层扩展段中的 EditTemplateManager（LayerLib.dll `ExtensionHelper::LoadExtensions` 装载），撤销第八轮「SaveObject(+0x218) 槽」假设；样本 `0xafe..0xb8e` 逐字节吻合，停止点 0xafe→0xb7a，见 [研究记录](docs/arcmap-10.1-featurelayer-tail.md)。
  - [x] 符号族第二轮闭合 DisplayCore 11 符号 coclass（MultiLayer Line/Marker/Fill、CharacterMarker、Picture Marker/Line、ArrowMarker、LineFill、MarkerFill、GradientFill、DotDensityFill）与 Template/LineDecoration/SimpleLineDecorationElement 三装饰 coclass；MarkerLine/CartographicLine/HashLine 的 ITemplate/ILineDecoration 槽改按类分派；TextSymbol 真虚表定位闭合（证据修正、读取器不变），见 [研究记录](docs/arcmap-10.1-display-symbols-round2.md)。
  - [x] Geometry.dll Polyline 对象体闭合（记录布局与同族骨架，16/16 样本对位，`src/objects/geometry.rs`），见 [研究记录](docs/arcmap-10.1-geometry.md)。
  - [x] Hyperlink 族体（coclass Hyperlink 与 FeatureHyperlink）与 INumberFormat 三变体（Percentage/Fraction/RateFormat）闭合；CGL 第 7 步嵌套对象按类标识分派 ElementCollection；全目标 476 项测试，见 [Hyperlink 记录](docs/arcmap-10.1-hyperlink.md)、[NumberFormat 记录](docs/arcmap-10.1-number-format.md) 与 [map-object 记录](docs/arcmap-10.1-map-object.md)。
  - [x] 整合接线轮九处接线（2026-11-15）：LELP ⑯ 槽接 TextSymbol 体（`annotate_props`，停止透传新变体）、SimpleRenderer/LegendClass/TextSymbol 三处 ISymbol 槽切 `read_symbol_slot_full_10_1` 全量分派（第一轮闭合类行为逐值不变）、TextElement IGeometry 槽接 `objects::geometry` 分派、FieldInfo INumberFormat 槽扩四类分派（NumericFormat 读取器去重为 objects 层单份）、FL 后继段 IHyperlink 循环接 Hyperlink 体、CGL 第 2/9 步嵌套对象按第 7 步同模式接 ElementCollection、`mxd_maps` 标注环逐条诊断、robustness 集中清单 48→74 入口；482 项测试全绿，样本停止点 0xb7a→0xd6c（+498 字节）。
  - [ ] 开放：样本停止点 0xd6c 的阻塞环——LELP ⑯ 槽接入 TextSymbol 体后，停止点落入 TextSymbol 体内 IFontDisp 槽的新对象 OLE StdFont `{0BE35203-8F91-11CE-9DE3-00AA004BB851}`（键 68 @`0xd6c`，OLE 标准类体未研究且无外部长度不可跳过），其后 StdFont 体、TextSymbol 版本门尾段、LELP 剩余槽与 ALPC 条目键/尾 int32、以及 0xf63..0xfe6 段（FIDSet/选择集/显示过滤器/扩展段头，走查脚本独立证实）在样本上暂不可达。其余开放项按类别逐项单列、每项职责单一，闭合路径注于类别，出处见各研究文档：
    - 体未闭合类族（逐个反编译闭合对象体或分支）：
      - [ ] BOLP（`{EE535289}`，LabelPlacement.dll）体——LELP 之后的标注实现类，实现函数未定位。
      - [ ] 标注表达式引擎族体（LELP 之后的同族条目，位于样本停止点后方）。
      - [ ] LELP ⑲/㉔ 槽 Envelope/IGraphicsContainer 实现类、㉖ ISymbol 槽其余实现类、㉘/㉙ Envelope/IUID 实现类、㊴/㊷ 表达式属性族与 TimeData 族槽实现类体。
      - [ ] Geometry 同族其余 9 类记录块内部布局（Point/Polygon/Multipoint/Path/Ring/BezierCurve/EllipticArc/Line/CircularArc；Path 等「Save 空桩」类的持久化虚表归属为候选）。
      - [ ] INumberFormat 其余同族体（CurrencyFormat `{7E4F471A}`、LatLonFormat `{7E4F471D}`、AngleFormat `{7E4F471E}`、ScientificFormat `{7E4F471F}`、CustomNumberFormat `{7E4F4722}`）。
      - [ ] DisplayCore 其余符号/文本族体（SimpleTextPath、Callout/背景族）及其槽实现类：IMarkerBackground（MultiLayerMarker v≥2）、IColorRamp 族（GradientFill）、IPicture（PictureMarker v9 类型 3 等）、IPictureFillSymbol（`{D842B082}`）、StdFont/StdPicture（IFontDisp/IPicture 槽常见实现，DisplayCore 之外）。
      - [ ] 元素体版本 ==7 装饰块与 ==9 尾加块及其 IBorder/IBackground/IShadow/ISpatialReference 槽实现类。
      - [ ] TextSymbol 版本门尾段（v>1/v≥3/v≥4 字段与 IFontDisp 槽 StdFont 体；StdFont 即当前样本停止点，无字节佐证）。
      - [ ] IDatasetName 槽其余实现类（UnknownDatasetName/GeometricNetworkName/RelationshipClassName/TableName 等）。
      - [ ] FeatureBookmark 的 IName 槽实现类（分布未观察，见 mapsurround 记录）。
      - [ ] FDOGraphicsLayer 版本 4 探测分支（三依赖未闭合，见「语义定性」类）。
    - 符号与 Legend 族（逐个反编译闭合）：
      - [ ] LegendFormat 体与 ILegendItem 实现类（HorizontalBarLegendItem 等）。
      - [ ] Scalebar/ScaleText/MarkerNorthArrow/MapTitle 及四个 Scalebar 子类尾段（未逐类反编译；共同前缀已证为其 Load 首段）。
      - [ ] ILinePatch/IAreaPatch 实现类体（读取器在新对象处停止，见 legend-tails 记录）。
    - Variant/PropertySet 分支（反编译或逐指令复核）：
      - [ ] `Variant::Read` 对象分支（VT_DISPATCH/UNKNOWN 的类 GUID 语义）。
      - [ ] `Variant::Read` bool=false 的 Seek 分支。
      - [ ] PropertySet 数组（0x2000）分支（反编译已见、未逐指令复核且无样本）。
      - [ ] PropertySet 对象（9/13）分支（依赖无键 LoadObject 的对象边界）。
      - [ ] PropertySetProperty `{5B6D2B14}` 体（未反编译）。
      - [ ] SystemCore 本地 `Variant::Read` 副本逐 VARTYPE 对照（仅签名、字符串分支与 Load 调用点已证）。
      - [ ] `Variant::Write`（`0x68a7cb50`）逐段反编译（现为按对称性推断）。
      - [ ] PropertySet AddRef/Save thunk 转跳目标相同的解释（不影响字段序，见 property-set 记录开放项 5）。
    - Save 写侧（反编译闭合写出链）：
      - [ ] FeatureLayer Save 归因。
      - [ ] FDOGraphicsLayer Save 委派写出。
      - [ ] FDOGraphicsSublayer Save 版本表逐条目展开。
      - [ ] EditTemplate 版本表前四档「不写任何字节」与无外部长度体的读侧交互（10.1 不可达）。
      - [ ] AfStreamGetSaveVersion 版本表内部映射（表在 AfCore 未提取；「docVersion 0 → 旧布局版本号」为观察推断，符号族第二轮印证表项为直接读数）。
    - 语义定性（类型库成员级命名或运行取证；读取器对未定性字段原样保留）：
      - [ ] 颜色与符号标量字段业务语义（DisplayCore 两轮各类，按内容候选标注）。
      - [ ] 符号族三处「装载后被 `GetDefaultRotateWithTransformValue` 覆盖」short 的「读取即弃」语义（ArrowMarker ⑫/CharacterMarker ⑭/PictureMarker v9 ⑨ 与历史路径 +0x70；流内布局已定）。
      - [ ] Legend 族标量字段业务语义（LegendClassFormat/HorizontalLegendItem/共同前缀各 short/int32）。
      - [ ] ElementProperties/TextSymbol 字段业务语义。
      - [ ] FDN/TN（FeatureDatasetName/TopologyName）字段业务语义。
      - [ ] MapSurround/书签共同前缀与 FDO 旧加载器接口槽业务语义。
      - [ ] ALPC/LELP/ETM/FeatureIDSet 标量与字符串业务语义（标注表达式、类名、OID 等；ETM 尾字节 0x00 未定）。
      - [ ] Hyperlink 字段业务语义（esriHyperlinkType 值序、② 与 Type、③ 与热链字段区分）。
      - [ ] NumberFormat 三变体标量与 String 业务语义（含 Percentage/Rate「同槽升级」候选）。
      - [ ] Geometry 记录类型值的枚举映射语义（内部 shape 类别 vs esriGeometryType）。
      - [ ] Geometry 内核移交链（+0xc8 内嵌指针虚表 +0x1cc 槽）运行期装配定位。
      - [ ] FDO 版本 4 探测三依赖与其「Map 架构版本 9」历史布局解读。
      - [ ] `{7453C805}`/`{DBCA59AC}`/`{97D5B914}` 等未定名 IID。
    - 样本无佐证分支（待真实样本或运行取证升级为证据）：
      - [ ] FCN 的 FeatureDatasetName/TopologyName 两选择子（样本 11 处全 0）。
      - [ ] FDO 旧加载器体与 FDOGraphicsSublayer 体。
      - [ ] MapSurround/书签循环非空形态（样本计数 0）。
      - [ ] FieldInfo ≤3 分支。
      - [ ] Legend 族样本形态（LegendClassFormat/HorizontalLegendItem/BaseLegendItem 及版本 0/1；10.1 写侧恒写 3）。
      - [ ] MarkerElement/LineElement 体（样本 0 实例）。
      - [ ] display 两轮闭合的符号/颜色族（样本 0 实例）。
      - [ ] Hyperlink 版本 0 短体与循环空标记/回引形态。
      - [ ] NumberFormat 短体形态与「同槽升级」。
      - [ ] Geometry 版本 0/1 与 S_FALSE 捷径分支。
      - [ ] 后继段版本 0x1c..0x1f 分支与 TimeData 族四段（样本走 ≥0x20 门）。
      - [ ] PropertySet/Envelope/FDN/TN 的版本 0 分支。
    - 实现约束（非缺口）：各读取器对负值版本位模式的拒绝均较原生更严（写侧不产负值版本；如需对齐原生行为，须先补运行取证）；Geometry <−1e38 坐标的原生钳位未实现（原样保留）、嵌套空间参考仅支持投影坐标系（GCS/Unknown 无证据）；ETM 尺寸声明与实际消费不一致为显式错误（原生不校验，注明较严）。
    已闭合清单见上方 [x] 条目与 [研究记录](docs/arcmap-10.1-map-object.md)。
- [x] 解析地图坐标系、图层坐标系和显示范围。
  - [x] 用 REA 闭合 Map::Load 空间参考分支：请求 IID `{4434D680-…}`（ISpatialReference 候选）、空引用零 GUID 标记、PCS 类分派由流内类标识决定；补全后继序列 `FUN_794035f0`（ver≥0x17 计数+short 1/2/3+`{34C20002-…}` 对象+尾部索引）与旧版 `FUN_794038a0`（ver 0x11 拒绝、双集合布局），见 [研究记录](docs/arcmap-10.1-map-object.md)。
  - [x] 用样本字节闭合范围载体：Envelope 对象 `{30707212-…}` 体 = short 3 + 4 double `[XMin,YMin,XMax,YMax]` + int32 0 + 内嵌空间参考（PCS 体）或对 Map SR 的键回引；图层坐标系宿主于图层体 Envelope 第 4 段（样本 25/25 一致），修正此前「紧邻 32 字节」定位，见 [空间参考记录](docs/arcmap-10.1-spatial-reference.md)。
  - [x] 确认地图显示范围为 Map::Load 第 5 步循环后的原样 32 字节 WKSEnvelope（4 double，非对象，写入 +0x230 接口 +0xe0），随后两个 int32（-0x40 接口 +0x38/+0x40）；字段序 [XMin,YMin,XMax,YMax] 与样本流内定位此后经写方向取证升级为证据，见下方写方向条目。
  - [x] 写方向取证闭合 Envelope/PCS Save 与显示范围：Envelope::Save 恒写版本 3（short 3 + 32 字节 WKSEnvelope + int32 + bit6/7 追加 Z/M + SR 子对象）、Load 按 1/2/3 分支（==3 才 LoadObject SR，IID 与 Map 第 4 步同值）；字段序 [XMin,YMin,XMax,YMax] 由 AfCore 访问器逐指令绑定证实；PCS Save 恒写 v6 与读侧分支对称；Map::Save 显示范围写出链闭合（Map+0x298 助手 +0xdc 槽 → 原样 32 字节），样本定位 `0x24f64` 升级为确定（前后括号完整：第 5 段 MapSurround 与书签循环计数确认=0，20 字节空标记=16 零+键 0，对象头类标识修正为 `{1C352F40}`、`{1C352F3F}` 为请求 IID），见 [研究记录](docs/arcmap-10.1-spatial-reference.md#envelopesave-wksenvelope-写方向与-map-显示范围闭合2026-11-14-写方向取证轮)。
  - [x] 实现显式 10.1 Envelope 对象体读取器（`envelope`：版本 1/2/3 分支（版本 0 证据未覆盖、显式拒绝并注明较原生更严）、bit6/bit7 Z/M double 对、嵌套 SR 空标记/键回引/内嵌 PCS），接入 Map 后缀 IEnvelope 槽位（整体消费后继续读书签循环）并纳入 robustness 清单；样本 25 实例全版本 3、键 74（回引 Map PCS）/键 80（内嵌 PCS 81）两形态辅助吻合；图层坐标系宿主（图层体 Envelope 第 4 段）的模型接入待后续接线轮处理（尾段链自第八轮起闭合至 FieldInfo 循环，第九轮续闭合 ALPC 环至 LELP 体内；样本停止点现为 LELP ⑯ ISymbol 槽 TextSymbol 新对象 @`0xb7a`，见上方开放项）。
  - [x] 闭合 Envelope 归因并消解「+0x144 后 7 字节」异常（归因取证轮）：旧锚点系双层误读——`0x3d6` 的 16 字节实为 WorkspaceName 体的 ShapefileWorkspaceFactory CLSID `{A06ADB96-…}`（真实位置 `0x3d7..0x3e7`，旧读法起点提前 1 字节得到畸形 GUID），「固定 7 字节」= 工厂 CLSID 末字节 + WS 尾随 int32 + FCN 版本>1 选择子；真 `+0x144` GUID 经四个图层（shapefile/FileGDB/注记形态）逐字段验证均为 16 零，RTM 3035 与样本完全可调和，「样本由 SP 构建」工作假设与 SP 介质对照需求一并撤销（本地仅 RTM 介质，两个 SP 命名文件经核对与 ArcGIS SP 无关）。三个 Envelope 归因闭合：键 74 = LabelPlacement.dll LELP Load `FUN_785468c0` 第 4 个 LoadObject（IEnvelope `{1B077914}`，宿主 LabelEngineLayerProperties 键 60）；键 80 = FeatureLayer::Load `0x6f6e9940` 版本≥0xf 槽成员 +0x1f4；键 151 = FDO 委派注记 FeatureLayer 体内同族槽；另证 FeatureLayer 版本>8 槽（+0x1b4）样本全部空标记、全样本 25 个 Envelope 宿主分布（LELP 槽 10 / FL ver≥0xf 槽 12 / Map 尾簇 2）。`{1C352F40}` = IlluminationProps（esri.clsid 全表查证，书签类候选撤销）；Map+0x298 助手升级为「IEnvelope 持有者（Map 显示范围包络对象）」。仍开放：+0x1f4 槽字段语义保留候选（「实际绘制/缓存范围」，原「全范围 vs 当前范围」二选一提法作废，缺类型库成员级命名与绘制路径取证）、+0x1b4 无值证据、Map+0x298 coclass 静态推定 Geometry Envelope `{30707212}` 未最终闭合、`{97D5B914}` 未定名，见 [研究记录](docs/arcmap-10.1-spatial-reference.md)。
- [x] 处理未知对象、未知版本、截断数据、非法长度及循环引用；未知对象/版本与截断、非法长度由各读取器原样保留或明确拒绝，循环引用经 `object_refs` 状态机覆盖。
  - [x] 建立跨读取器健壮性测试：全部严格读取器入口的逐前缀截断、失败游标不变与重读一致、超大长度/限额溢出边界、未知类标识与键原样保留、VARTYPE 全空间穷举及非递归有界完成。仅验证读取器自身契约，不代表 MXD 格式规则已验证。
  - [x] 对象引用编码闭合后覆盖缺失引用、重复标识及循环引用：`object_refs` 单元与集成测试覆盖 A→B→A 循环、自引用、跨流共享键序（SetStream 不清表）、重复登记=替换、空表不可区分回引与新对象（原生语义）、截断失败不改登记表；样本键 29/32 辅助对照吻合。仅验证分类器自身契约，不证明完整对象解析已验证。

## 中间模型

- [x] 定义地图、数据框、图层组和图层的数据结构，保存层级、顺序和可见性。
- [x] 定义数据源、坐标系和显示范围的数据结构，保留无法识别的字段及其位置。
- [ ] 将 MXD 解析结果转换为中间模型，并结合 REA 证据核对图层树和属性。
  - [x] 新增 `parse` 命令（`mxd_maps` 模块）接入显式 10.1 读取器：`/Maps` 根前缀（数量、首对象头、尾部原始值）、Map 对象体前缀（版本 ≤51、双名称、嵌套 PCS）与后缀（图层列表、基本图形层、MapSurround、WKSEnvelope、IEnvelope、书签循环）填充中间模型 `Map`/`DataFrame`：数据框名称/第二名称/WKT，图层条目顺序与体前缀（FeatureLayer 名称/可见性/数据源要素类名、GroupLayer 名称/可见性；回引复用同对象节点）→ children，WKSEnvelope → extent；`/Version` 识别候选并入输出。无论数量多少最多解析首个对象（对象循环边界未闭合），未解析体与停止点之后的字节全部保留为未知字段并输出诊断，PCS 域字符串/标志/double 不写入模型。仓库参考样本前缀逐项吻合仅为辅助观察；单元测试覆盖合成正常与异常输入（截断、负数量、非 Map/非 PCS 类、限额、空头、条目/槽位停止点），仅验证代码自身契约，未在 ArcMap 10.1 或 QGIS 中人工目视验证。

## QGZ 生成

- [ ] 实现 `convert` 命令，读取 MXD 并输出 QGZ。
- [ ] 将中间模型映射为 QGIS 工程、图层树、图层顺序和可见性。
  - [x] 构造模型的独立 QGS/QGZ 写入器：XML 转义、确定性标识、ZIP、防覆盖、并发发布和失败清理；单框层级、树序、组/叶勾选通过 XML 和 QGIS API 核对。
- [ ] 转换 Shapefile 数据源路径，处理相对路径、绝对路径和缺失文件。
  - [x] 构造模型支持 Windows/UNC/POSIX 词法路径重定位，断链保留并诊断；不将异平台路径当成本机路径。尚未从 MXD 解析连接。
- [ ] 写入工程及图层坐标系、地图显示范围。
  - [x] 构造模型支持明确 EPSG:4326/3857、不同地图/图层 CRS 和主画布范围；其他定义保留并诊断，未猜 EPSG。非法或溢出范围拒绝。
- [x] 构造模型路径已可生成 QGIS 可打开的 QGZ（独立写入器，本机 QGIS 4.2.3 实机加载验证，见「自动验证」节；人工目视仍未验证）；从 MXD 解析结果端到端生成仍开放，依赖 `convert` 接线。
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
- [x] 为二进制读取、对象引用、字符串解码和异常输入添加测试。
  - [x] 覆盖整数及浮点读取的全部截断长度、失败后重试、浮点特殊位模式和嵌套范围隔离；遍历全部 UTF-16 单码元及补充平面边界，核对非法代理项和非零偏移错误位置。
  - [x] 在确定对象引用编码后，覆盖缺失引用、重复标识及循环引用：编码已闭合（键回引按已见键集合判定），`object_refs` 覆盖缺失引用（未见过键即新对象，不可区分系原生语义）、重复标识（重复登记=替换）及循环引用（A→B→A、自引用、跨流共享键序），并接入 robustness 截断共性清单；仅验证分类器契约，不代表 MXD 对象解析已验证。
  - [x] 新增 feature_layer / envelope / group_layer / composite_graphics / layer_list / Map 后缀读取器的独立测试（13+13+13+10 项及 map_object/cli 扩展），并把 robustness 集中清单扩至 19 个入口：全前缀逐字节截断、失败游标与登记表事务性、版本上限（35/4/13/7）、图层列表计数与非法标志、活动索引与各停止点。仅验证读取器自身契约，不代表 MXD 格式规则已验证。
  - [x] 第七轮新增模块测试并扩充 robustness 集中清单至 28 个入口：feature_layer_tail（25 项：五类常量、样本形态全链、版本拒绝、各停止点游标与登记表事务性、截断扫描、限额）、map_surround（20 项）、fdo_legacy（20 项）、dataset_name（12 项）、element_collection（11 项）；集中清单新增 SimpleRenderer 体、ElementCollection 体、FeatureDatasetName/TopologyName 体、FDO 旧加载器前缀、AOI/Feature 书签与 Legend 前缀共 8 个入口的统一截断扫描。仅验证读取器自身契约，不代表 MXD 格式规则已验证。
- [x] 为构造模型的数据源路径重定位、坐标系处理和图层树映射添加测试（`tests/qgis.rs` 14 项通过 + 1 项 QGIS API 测试默认忽略、显式运行后通过）：词法路径重定位与断链诊断、歧义 URI/根逃逸拒绝、CRS 仅识别 EPSG:4326/3857 其余保留并诊断、树序/勾选/XML 往返与确定性标识、非法范围与溢出拒绝。仅覆盖输出层与构造模型自身契约；MXD 侧接入（`convert`）未实现，真实数据源路径与坐标系规则未在目标软件验证。
- [x] 并行四轮研究 + 整合接线轮新增模块测试并扩充 robustness 集中清单 48 → 74 个入口（ALPC/LELP/ETM/FeatureIDSet 四体、符号族第二轮 14 直读入口与全量分派、Polyline 体与 geometry 类分派、数字格式四类与 Hyperlink 体、CGL 三处嵌套分派）；全仓 482 项测试全绿（基线 476 +6），`cargo fmt --check` 干净；样本 `parse` 停止点 0xafe→0xb7a→0xd6c。仅验证读取器自身契约，不代表 MXD 格式规则已验证。

## 人工验收

- [ ] 在 ArcMap 10.1 中检查源工程，并与 QGIS 输出对比，记录实际软件版本、检查范围和结果；当前未验证。
- [ ] 在 ArcMap 中查看源工程，人工核对中间模型的图层树和属性与源工程一致。
- [ ] 在 QGIS 中打开转换结果，验证图层能够加载，并目视核对图层、坐标系和范围。
- [ ] 对比 ArcMap 与 QGIS 的渲染结果，逐项目视检查符号、标注和布局差异，记录软件版本及检查结果。
