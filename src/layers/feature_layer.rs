//! ArcMap 10.1 FeatureLayer（要素图层）对象体已确认前缀的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md` 的「图层树、可见性与数据源连
//! 接」「FeatureLayer::Load 请求 IID 字节表」「FeatureLayer::Load 中后段补
//! 全」「MapSurround/书签循环接口身份…第五轮」「FDOGraphicsLayer 体闭合
//! …第六轮」各节（2026-11-13/14），以及 `docs/arcmap-10.1-spatial-reference.md`
//! 的「FeatureLayer::Save（0x6f6eb1b0，写侧新证据；归因仍开放）」节。字段序
//! 来自 FeatureLayer.dll 10.1.0.3035（SHA-256 `2baf8706…98bb60`）的
//! `IPersistStream::Load`（RTTI `.?AVFeatureLayer@@` → IPersistStream 子接口
//! 虚表 `0x6f7acfb0` 槽 +0x14 = `0x6f6e9940`）静态反编译；数据源
//! FeatureClassName / WorkspaceName 体字段序来自 GdbCore.dll 10.1.0.3035
//! （SHA-256 `ba5d8f5d…39ca1c4`，`FeatureClassName::Load 0x709b5080`、
//! `WorkspaceName::Load 0x70a27950`，见研究文档第六轮）。全部证据未运行目
//! 标软件。
//!
//! 已确认字段序（Load `0x6f6e9940` 头部段；上限版本 0x22=34，样本值 34）：
//!
//! 1. int16 架构版本（>34 拒绝；比较符号性未闭合，本读取器按无符号解释，
//!    负值位模式一并拒绝，与 map_object/spatial_reference 的版本处理一致）；
//! 2. AfCore String 名称 → obj+0x120；
//! 3. AfCore String 第二名称 → obj+0x124；
//! 4. int16 ×3 → obj+0x128 可见性 VARIANT_BOOL（样本 0xFFFF=TRUE，证据
//!    「确定的」）/ obj+0x12a / obj+0x12c（后两个语义未定，原样保留）；
//! 5. 嵌套 LoadObject（数据源连接，请求 IID
//!    `{4BA33331-D55F-11D1-8882-0000F877762D}`=IName，第五轮修正）：16 零
//!    字节 → 无数据源；键已见 → 回引（无对象体）；新对象且类标识为
//!    FeatureClassName → 读取其完整已确认体（见下）；新对象且类标识非
//!    FeatureClassName → 不支持（对象体无外部长度字段，无法跳过）；
//! 6. 嵌套 LoadObject（请求 IID `{40A9E884-5533-11D0-98BE-00805F7CED21}`=
//!    IFeatureRenderer，第五轮类型库考证）：空标记/回引 → 继续；新对象且
//!    类标识为 SimpleRenderer → 读取其闭合体（feature_layer_tail 模块：含
//!    SimpleMarkerSymbol/RgbColor/LegendGroup/LegendClass 嵌套链）；其他
//!    渲染器类 → 体未闭合，读取对象头后停止；
//! 7. 渲染器槽位为空标记/回引/完整消费的 SimpleRenderer 体时：原样 16 字节
//!    GUID → obj+0x144（非对象头）。其后字段（String、double ×2、short、
//!    `{3036D35D}` 计数循环等）在样本中存在与 RTM 3035 反编译不可调和的差
//!    异（开放项，见模块文档「未解决问题」），一律保留为剩余字节。
//!
//! ## FeatureClassName 体（GdbCore.dll `0x709b5080`，2026-11-14 第六轮闭合）
//!
//! 类身份：CLSID `{198846D0-…}` 由 GetClassID `0x709b3a40` 写出（与 Load 同
//! 虚表 `0x70ad02dc`，this-adjustor 0x20）；10.1 的 C++ 实现类为
//! `FdbFeatureClassName`（RTTI `.?AVFdbFeatureClassName@@`，经典类名仅存于
//! CLSID）。字段序：
//!
//! 1. int16 版本：汇编 `2 < (short)ver` 为**有符号**比较，>2 拒绝（样本 2）；
//! 2. String 名称 → obj+0x1c（样本与图层名同文）；
//! 3. String 第二名称 → obj+0x20（样本为空串，长度前缀 2——勿误读为 int32）；
//! 4. String 类型描述 → obj+0x24（样本「Shapefile 要素类」）；
//! 5. String 数据集名 → obj+0x28（样本「Shape」）；
//! 6. int32 → obj+0x30（样本 1）；int32 → obj+0x34（样本 1）；
//! 7. int16 选择子：==0 → LoadObject（请求 IID
//!    `{FADD975C-E36F-11D1-AA81-00C04FA33A15}`=IWorkspaceName）；!=0 →
//!    LoadObject（`{89E75917-C287-11D1-AA77-00C04FA33A15}`=IDatasetName）
//!    并对 obj+0x3c 做虚调用；
//! 8. 版本 >1：int16 第二选择子：!=0 → LoadObject（
//!    `{505B6FDD-9E6E-42F9-B705-53355B2F1DEF}`=ITopologyName）。
//!
//! 样本对位（`/Maps:0x30a..0x3ed`，辅助观察）：版本 2 @`0x30a`、名称
//! @`0x30c`、空串 @`0x330`、类型描述 @`0x336`、数据集名 @`0x356`、
//! int32 1 @`0x366`、int32 1 @`0x36a`、选择子 0 @`0x36e`、WorkspaceName
//! 对象头 @`0x370`、版本 >1 选择子 0 @`0x3eb`——恰终结于 SimpleRenderer
//! 对象头 @`0x3ed`（FeatureLayer::Load 第 6 步的渲染器槽位）。
//!
//! ## WorkspaceName 体（GdbCore.dll `0x70a27950`，第六轮闭合）
//!
//! 类身份：CLSID `{5A350011-…}` 由 GetClassID `0x70a27910` 写出（同虚表
//! `0x70ada890`，this-adjustor 0x18）。字段序：
//!
//! 1. int16 版本（>1 拒绝；样本 1）；
//! 2. String 路径 → obj+0x20（样本 `..\22.界址点`，敏感路径分享前脱敏）；
//! 3. String 第二路径 → obj+0x24（样本空串）；
//! 4. String 类型描述 → obj+0x28（样本「Shapefile Data」）；
//! 5. LoadObject（请求 IID `{F0BA6ABC-8E9F-11D0-B4AB-0000F8037368}`=
//!    IPropertySet）：样本为 16 字节零空标记；
//! 6. 1 字节标志（样本 1）；
//! 7. 标志非 0 时：原样 16 字节工作空间工厂 CLSID → 经
//!    AfCreateInstance（IID `{F173FC16-D63A-11D1-AA81-00C04FA33A15}`=
//!    IWorkspaceFactory）创建。样本 `{A06ADB96-D95C-11D1-AA81-…}`=
//!    ShapefileWorkspaceFactory（esri.clsid 声明）——**修正第五轮前记录**：
//!    该 16 字节（`/Maps:0x3d7`）不是 FeatureLayer 的 +0x144 GUID，+0x144
//!    GUID 位于渲染器槽位之后；
//! 8. int32 → obj+0x38（样本 0）。
//!
//! 样本对位（`/Maps:0x384..0x3eb`）逐字段吻合：版本 1、路径、空串、
//! 「Shapefile Data」、空标记、标志 1、工厂 CLSID、int32 0。
//!
//! 调用约定：调用者先用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`FEATURE_LAYER_CLASS_ID`]，从对象头之后的位置调用
//! [`read_feature_layer_prefix_10_1`]；本模块不重复读取对象头。嵌套条目用
//! [`crate::storage::object_refs::read_object_entry_10_1`] 分类：登记先于对象体解析
//! 是原生语义；为满足「失败不修改登记表」，分类与登记先发生在调用者登记表
//! 的副本上，全部成功后才整体提交。
//!
//! 样本核对（辅助观察，不作测试依赖）：`samples/测试底图.mxd` /Maps 流内首
//! 个 FeatureLayer——对象头 @`0x296` 类标识与本模块常量逐字节相同（键
//! 32）；体 @`0x2aa` 起与上述字段序逐字段吻合，数据源链
//! FCN(33)→WorkspaceName(34) 全部闭合到 `/Maps:0x3eb`，渲染器槽位
//! SimpleRenderer(35) @`0x3ed` 为停止点。样本吻合为辅助观察。
//!
//! 未解决问题（详见证据文档「未解决事项」）：
//! - FeatureLayer 的 +0x144 原样 16 字节 GUID 语义未定性；其后的字段在样
//!   本 11 层中与 RTM 3035 反编译存在不可调和差异（TODO「FeatureLayer
//!   Save 写侧新证据归因仍开放」：工作假设样本由某 10.1 服务包保存，
//!   `/Version` 文本不区分 SP，未验证）——本读取器读到 +0x144 GUID 后停
//!   止，其后字节全部保留，不得解释；
//! - FCN 体内 int32 ×2、WorkspaceName 体内 int32、各 String 的业务语义未
//!   定性（类型描述/数据集名为按内容候选），原样保留；
//! - IDatasetName / ITopologyName 槽的 FeatureDatasetName / TopologyName
//!   体已由 `dataset_name` 模块闭合并接入选择子分派（样本两选择子全 0，
//!   布局无字节佐证）；其他 IDatasetName 实现类（UnknownDatasetName 等）
//!   在对象头处停止；
//! - 可见性之外两个 short（obj+0x12a/+0x12c）语义未定，原样保留；
//! - FeatureLayer 版本比较的符号性未闭合（本读取器按无符号解释，较可能的
//!   有符号原生更严）；
//! - 回引条目不校验类标识：样本证据（如 `/Maps:0x2d2c` 对键 32 的回引头）
//!   表明回引头会重复被引对象的类标识，类标识原样保留、由调用者比对。

use crate::layers::feature_layer_tail::{
    SIMPLE_RENDERER_CLASS_ID, SimpleRendererPrefix10_1, SimpleRendererStop10_1,
    read_simple_renderer_body_10_1,
};
use crate::objects::dataset_name::{
    FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1, FeatureDatasetNameStop10_1,
    TOPOLOGY_NAME_CLASS_ID, TopologyNameBody10_1, TopologyNameStop10_1,
    read_feature_dataset_name_body_10_1, read_topology_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// FeatureLayer::Load 接受的最大架构版本：>0x22（34）直接拒绝；样本值 34。
const MAX_ARCHITECTURE_VERSION: u16 = 34;

/// FeatureClassName::Load 接受的最大版本：汇编 `2 < (short)ver` 有符号比
/// 较，>2 拒绝；样本值 2。
const MAX_FCN_VERSION: i16 = 2;

/// WorkspaceName::Load 接受的最大版本：`1 < ver` 拒绝；样本值 1。
const MAX_WS_VERSION: i16 = 1;

/// FeatureLayer 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{E663A651-8AAD-11D0-BEC7-00805F7C4268}`（Data1=
/// 0xE663A651 → `51 a6 63 e6`，Data2=0x8AAD → `ad 8a`，Data3=0x11D0 →
/// `d0 11`，Data4 原序 `be c7 00 80 5f 7c 42 68`）。三重印证：安装配置
/// `esri.clsid.ecfg` 声明该 CLSID → 类名 FeatureLayer（模块
/// FeatureLayer.dll）；FeatureLayer.dll RTTI 存在 `.?AVFeatureLayer@@`（TD
/// `0x6f7d2a34`）；GetClassID（IPersistStream 虚表 `0x6f7acfb0` 槽 3 =
/// `0x6f6ec450`）写出同一 CLSID。仓库参考样本 `/Maps:0x296` 首个
/// FeatureLayer 对象头的类标识与本常量逐字节相同（键 32）。
pub const FEATURE_LAYER_CLASS_ID: [u8; 16] = [
    0x51, 0xa6, 0x63, 0xe6, 0xad, 0x8a, 0xd0, 0x11, 0xbe, 0xc7, 0x00, 0x80, 0x5f, 0x7c, 0x42, 0x68,
];

/// FeatureClassName 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4
/// 原序）。
///
/// GUID 字符串形式为 `{198846D0-CA42-11D1-AA7C-00C04FA33A15}`（Data1=
/// 0x198846D0 → `d0 46 88 19`，Data2=0xCA42 → `42 ca`，Data3=0x11D1 →
/// `d1 11`，Data4 原序 `aa 7c 00 c0 4f a3 3a 15`）。证据：安装配置
/// `esri.clsid.ecfg` 声明（模块 GdbCore.dll）；GdbCore.dll GetClassID
/// `0x709b3a40` 写出同一 CLSID（与 Load `0x709b5080` 同虚表，第六轮）；
/// 仓库参考样本 `/Maps:0x2f6`（FeatureLayer 体内嵌数据源对象头，键 33）
/// 与本常量逐字节相同。
pub const FEATURE_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0xd0, 0x46, 0x88, 0x19, 0x42, 0xca, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3, 0x3a, 0x15,
];

/// WorkspaceName 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{5A350011-E371-11D1-AA82-00C04FA33A15}`。证据：
/// `esri.clsid.ecfg` 声明（模块 GdbCore.dll）；GdbCore.dll GetClassID
/// `0x70a27910` 写出同一 CLSID（与 Load `0x70a27950` 同虚表，第六轮）；
/// 参考样本 `/Maps:0x370`（FeatureClassName 体内嵌 WorkspaceName 对象头，
/// 键 34）与本常量逐字节相同。
pub const WORKSPACE_NAME_CLASS_ID: [u8; 16] = [
    0x11, 0x00, 0x35, 0x5a, 0x71, 0xe3, 0xd1, 0x11, 0xaa, 0x82, 0x00, 0xc0, 0x4f, 0xa3, 0x3a, 0x15,
];

/// 可见性字段（obj+0x128）的 VARIANT_BOOL 解释辅助。
///
/// VARIANT_BOOL 的规范取值只有 VARIANT_TRUE（0xFFFF）与 VARIANT_FALSE
/// （0x0000）；规范之外的位模式不做「非零即真」假设（COM 惯例如此，但原生
/// 比较语义未取证），按非规范原样保留。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility10_1 {
    /// 0xFFFF = VARIANT_TRUE（样本值 @`/Maps:0x2f0`）。
    True,
    /// 0x0000 = VARIANT_FALSE。
    False,
    /// 规范取值之外的位模式，原样保留。
    NonCanonical(u16),
}

/// WorkspaceName 体读取的停止原因（体内在首个不可闭合处停止，非错误）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceNameStop10_1 {
    /// IPropertySet 槽位的新对象体（PropertySet 类字段序未闭合）：其后
    /// 的标志、工厂 CLSID 与 int32 不可达。
    PropertySetNewObject { class_id: [u8; 16], key: u32 },
}

/// WorkspaceName（GdbCore.dll `0x70a27950`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceNamePrefix10_1 {
    /// 对象头键（样本 34 @`/Maps:0x380`）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表（登记先于对象体解
    /// 析，见 [`crate::storage::object_refs`]）。
    pub key: u32,
    /// 体首 int16 版本（样本 1）；>1 原生拒绝。
    pub version: u16,
    /// 路径字符串（→ obj+0x20；样本 `..\22.界址点`，敏感路径分享前脱敏）。
    pub path: String,
    /// 第二路径字符串（→ obj+0x24；样本为空串）。
    pub secondary_path: String,
    /// 类型描述字符串（→ obj+0x28；样本「Shapefile Data」）。
    pub type_description: String,
    /// IPropertySet 槽位（请求 IID `{F0BA6ABC-…}`）分类；新对象 = 停止点。
    pub property_set: ObjectEntry10_1,
    /// 1 字节工厂标志（样本 1）。`None` = 在其之前已停止。
    pub factory_flag: Option<u8>,
    /// 标志非 0 时原样读取的 16 字节工作空间工厂 CLSID（样本
    /// `{A06ADB96-…}`=ShapefileWorkspaceFactory，经 AfCreateInstance 以
    /// IWorkspaceFactory `{F173FC16-…}` 创建）。`None` = 未到达或标志为 0。
    pub factory_class_id: Option<[u8; 16]>,
    /// 尾随 int32（→ obj+0x38；样本 0）。`None` = 在其之前已停止。
    pub trailing_int32: Option<i32>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<WorkspaceNameStop10_1>,
}

/// FeatureClassName 体内 IWorkspaceName 槽位的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceNameEntry10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 WorkspaceName 对象（类标识经 [`WORKSPACE_NAME_CLASS_ID`] 校验）：
    /// 已解析体；体内在 IPropertySet 新对象处停止时 `stop` 携带原因。
    NewObject(Box<WorkspaceNamePrefix10_1>),
    /// 新对象但类标识不是 WorkspaceName（体未知、无外部长度）：停止点，
    /// 游标回退到该对象头起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// FeatureClassName 体内 IDatasetName 槽位（选择子 !=0，请求 IID
/// `{89E75917-…}`）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DatasetNameEntry10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 FeatureDatasetName 对象（类标识经 [`FEATURE_DATASET_NAME_CLASS_ID`]
    /// 校验）：已读取其闭合体（dataset_name 模块）；体内停止时经
    /// [`FeatureClassNameStop10_1::DatasetNameBodyStopped`] 携带原因。
    NewObject(Box<FeatureDatasetNameBody10_1>),
    /// 新对象但类标识不是 FeatureDatasetName（UnknownDatasetName、
    /// FeatureClassName 自身等其他 IDatasetName 实现类未闭合）：停止点，
    /// 游标回退到该对象头起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// FeatureClassName 体版本 >1 第二选择子非 0 的 ITopologyName 槽位（请求
/// IID `{505B6FDD-…}`）分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum TopologyNameEntry10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 TopologyName 对象（类标识经 [`TOPOLOGY_NAME_CLASS_ID`] 校验）：
    /// 已读取其闭合体；体内停止时经
    /// [`FeatureClassNameStop10_1::TopologyNameBodyStopped`] 携带原因。
    NewObject(Box<TopologyNameBody10_1>),
    /// 新对象但类标识不是 TopologyName：停止点，游标回退到该对象头起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// FeatureClassName 体第 7 步选择子的两个分支。
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureClassNameNested10_1 {
    /// 选择子 ==0：IWorkspaceName 槽（样本分支）。
    WorkspaceName(WorkspaceNameEntry10_1),
    /// 选择子 !=0：IDatasetName 槽（请求 IID `{89E75917-…}`）。
    DatasetName(DatasetNameEntry10_1),
}

/// FeatureClassName 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureClassNameStop10_1 {
    /// 选择子 !=0 分支的 IDatasetName 槽新对象类标识不是
    /// FeatureDatasetName：其后 的版本 >1 选择子不可达。
    DatasetNameNewObject { class_id: [u8; 16], key: u32 },
    /// 选择子 !=0 分支的 FeatureDatasetName 体自身的停止点（原因透传，
    /// 见 [`FeatureDatasetNameStop10_1`]）。
    DatasetNameBodyStopped(FeatureDatasetNameStop10_1),
    /// IWorkspaceName 槽的新对象类标识不是 WorkspaceName（体未知）。
    WorkspaceNameUnsupported { class_id: [u8; 16], key: u32 },
    /// WorkspaceName 体内 IPropertySet 槽的新对象体（见
    /// [`WorkspaceNameStop10_1`]）。
    WorkspacePropertySetNewObject { class_id: [u8; 16], key: u32 },
    /// 版本 >1 且第二选择子非 0：ITopologyName 槽（请求 IID
    /// `{505B6FDD-…}`）的新对象类标识不是 TopologyName。
    TopologyNameNewObject { class_id: [u8; 16], key: u32 },
    /// 版本 >1 且第二选择子非 0：ITopologyName 槽的 TopologyName 体自身的
    /// 停止点（原因透传，见 [`TopologyNameStop10_1`]）。
    TopologyNameBodyStopped(TopologyNameStop10_1),
}

/// FeatureClassName（GdbCore.dll `0x709b5080`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureClassNamePrefix10_1 {
    /// 对象头键（样本 33 @`/Maps:0x306`）。
    ///
    /// 解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 int16 版本（样本 2）；原生有符号比较 >2 拒绝（汇编证实）。
    pub version: u16,
    /// 要素类名（→ obj+0x1c；样本与图层名「龙安镇人民政府工作底图用界址
    /// 点」同文）。
    pub name: String,
    /// 第二名称（→ obj+0x20；样本为空串——长度前缀 2 的 6 字节模式，勿误
    /// 读为 int32 计数）。
    pub secondary_name: String,
    /// 类型描述（→ obj+0x24；样本「Shapefile 要素类」，语义按内容候选）。
    pub type_description: String,
    /// 数据集名（→ obj+0x28；样本「Shape」，语义按内容候选）。
    pub dataset_name: String,
    /// int32 → obj+0x30（样本 1），语义未定，原样保留。
    pub int32_a: i32,
    /// int32 → obj+0x34（样本 1），语义未定，原样保留。
    pub int32_b: i32,
    /// 第 7 步选择子（样本 0）。
    pub selector: u16,
    /// 选择子分支的嵌套槽位结果。
    pub nested: FeatureClassNameNested10_1,
    /// 版本 >1 的第二选择子。`None` = 版本 <=1，或因前置停止未读取。
    pub version1_selector: Option<u16>,
    /// 第二选择子非 0 时的 ITopologyName 槽分类；新对象非 TopologyName 类
    /// 或 TopologyName 体内停止 = 停止点。`None` = 未读取（版本 <=1、前置
    /// 停止或选择子为 0）。
    pub topology_slot: Option<TopologyNameEntry10_1>,
    /// 停止原因；`None` = 体已完整消费（样本形态：两选择子均为 0）。
    pub stop: Option<FeatureClassNameStop10_1>,
}

/// 数据源嵌套条目（请求 IID `{4BA33331-…}`=IName）的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DataSource10_1 {
    /// 16 字节全零类标识：空引用标记，无键、无对象体。
    Null,
    /// 新 FeatureClassName 对象：已确认体（版本、四字符串、int32 ×2、
    /// 选择子与嵌套 WorkspaceName 链、版本 >1 选择子）。
    FeatureClassName(Box<FeatureClassNamePrefix10_1>),
    /// 键已在本会话登记过的回引条目：其后没有对象体。
    ///
    /// 类标识原样保留、不校验是否为
    /// [`FEATURE_CLASS_NAME_CLASS_ID`]：回引头会重复被引对象的类标识（样本
    /// 证据见模块文档），调用者可自行比对；回引无体、不影响后续字段对位，
    /// 本读取器不据此拒绝。
    BackReference { class_id: [u8; 16], key: u32 },
}

/// FeatureLayer 体渲染器槽位（请求 IID `{40A9E884-…}`=IFeatureRenderer）
/// 的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum RendererSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 SimpleRenderer 对象（类标识经 [`SIMPLE_RENDERER_CLASS_ID`] 校验）：
    /// 已读取其闭合体（feature_layer_tail 模块：SimpleMarkerSymbol、
    /// RgbColor、LegendGroup、LegendClass 嵌套链）；体内停止时经
    /// [`FeatureLayerStop10_1::RendererBody`] 携带原因。
    SimpleRenderer(Box<SimpleRendererPrefix10_1>),
    /// 新对象但类标识不是 SimpleRenderer（其他渲染器类体未闭合）：停止点，
    /// 游标回退到该对象头起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// FeatureLayer 体读取的停止原因（本读取器为「已确认前缀」语义：始终在最
/// 深可闭合点停止并返回成功）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureLayerStop10_1 {
    /// 数据源 FeatureClassName 体内（含 WorkspaceName 体）的停止点。
    DataSource(FeatureClassNameStop10_1),
    /// 数据源之后的首个嵌套对象（请求 IID `{40A9E884-…}`=IFeatureRenderer）
    /// 为新对象且类标识不是 SimpleRenderer（其他渲染器类体未闭合）：读取
    /// 对象头后停止。
    RendererNewObject { class_id: [u8; 16], key: u32 },
    /// 渲染器槽的 SimpleRenderer 体自身的停止点（原因透传，见
    /// [`SimpleRendererStop10_1`]）：游标停在其停止点。
    RendererBody(SimpleRendererStop10_1),
    /// +0x144 原样 16 字节 GUID 之后：其后字段在样本中与 RTM 3035 反编译
    /// 不可调和（归因开放，见模块文档「未解决问题」），全部保留为剩余字节。
    AfterDataSourceGuid,
}

/// 10.1 FeatureLayer::Load（`0x6f6e9940`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureLayerPrefix10_1 {
    /// 2 字节架构版本 short 的无符号位模式，>34 的位模式一律拒绝。
    ///
    /// 证据：Load `0x6f6e9940` 开头读取后对 >0x22（34）直接拒绝，样本值 34
    /// （`/Maps:0x2aa`）。反编译比较的符号性未闭合：按无符号解释接受
    /// 0..=34；若原生为有符号比较，负值位模式（0x8000..）可能通过原生检
    /// 查，此处一并拒绝（较原生更严，与 map_object/spatial_reference 的版
    /// 本处理一致）。
    pub version: u16,
    /// 图层名称（Load 第 2 步，写入 obj+0x120；样本「龙安镇人民政府工作底
    /// 图用界址点」）。
    pub name: String,
    /// 第二名称（Load 第 3 步，写入 obj+0x124；样本「Shapefile 要素类」）。
    pub secondary_name: String,
    /// obj+0x128 可见性的原始 u16 位模式（VARIANT_BOOL；样本 0xFFFF=TRUE）。
    ///
    /// 可见性结论为「确定的」（代码偏移 + 样本 0xFFFF @`/Maps:0x2f0` 互
    /// 证）；解释辅助见 [`Self::visibility`] 与 [`Visibility10_1`]，规范之
    /// 外的位模式原样保留。
    pub visibility_raw: u16,
    /// obj+0x12a 的 short，语义未定，按无符号位模式原样保留（样本 0x0000）。
    pub unknown_short_a: u16,
    /// obj+0x12c 的 short，语义未定，按无符号位模式原样保留（样本 0x0000）。
    pub unknown_short_b: u16,
    /// Load 第 5 步嵌套数据源条目的分类结果（请求 IID=IName，见
    /// [`DataSource10_1`]）。
    pub data_source: DataSource10_1,
    /// 数据源之后的首个嵌套对象槽（请求 IID `{40A9E884-…}`=
    /// IFeatureRenderer）分类。`None` = 因数据源体内停止未读取。
    pub renderer: Option<RendererSlot10_1>,
    /// 渲染器槽位为空标记/回引/完整消费的 SimpleRenderer 体时读取的
    /// +0x144 原样 16 字节 GUID。`None` = 未到达。
    pub guid_after_renderer: Option<[u8; 16]>,
    /// 停止原因（恒有：本读取器始终在最深可闭合点停止）。
    pub stop: FeatureLayerStop10_1,
    /// 停止点之后剩余的字节数。
    pub remaining_bytes: usize,
}

impl FeatureLayerPrefix10_1 {
    /// 按 VARIANT_BOOL 语义解释可见性原始位模式。
    ///
    /// 0xFFFF → [`Visibility10_1::True`]、0x0000 →
    /// [`Visibility10_1::False`]、其余位模式 → [`Visibility10_1::NonCanonical`]
    /// 原样保留（见枚举文档）。
    pub fn visibility(&self) -> Visibility10_1 {
        match self.visibility_raw {
            0xFFFF => Visibility10_1::True,
            0x0000 => Visibility10_1::False,
            other => Visibility10_1::NonCanonical(other),
        }
    }
}

/// 从对象头之后的位置读取 10.1 WorkspaceName 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对
/// 象头、比对 [`WORKSPACE_NAME_CLASS_ID`] 并取得键 `key`，从对象头之后的位
/// 置传入 Reader。字段序与证据见模块文档。在 IPropertySet 新对象处停止是
/// 成功返回（停止原因 [`WorkspaceNameStop10_1`]）；截断、版本超限、字符串
/// 超限为错误。
pub fn read_workspace_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<WorkspaceNamePrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("WorkspaceName 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 版本 short:原生 `1 < ver` 拒绝。汇编符号性未逐条复核,按有符号
    //    解释;负值位模式一并拒绝(负值版本无业务意义,较原生更严)。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_WS_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 WorkspaceName::Load 接受范围（>1 拒绝；负值位模式一并拒绝）"
        )));
    }

    // 2-4. 三个字符串:路径、第二路径、类型描述,各受 max_bytes 限额约束。
    let path = read_string_10_1(&mut trial, max_bytes)?;
    let secondary_path = read_string_10_1(&mut trial, max_bytes)?;
    let type_description = read_string_10_1(&mut trial, max_bytes)?;

    // 5. IPropertySet 槽位(请求 IID {F0BA6ABC-…});新对象体未闭合 → 停止。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let property_set = read_object_entry_10_1(&mut trial, &mut staged_table)?;
    let (factory_flag, factory_class_id, trailing_int32, stop) = match property_set {
        ObjectEntry10_1::NewObject { class_id, key } => {
            let stop = WorkspaceNameStop10_1::PropertySetNewObject { class_id, key };
            (None, None, None, Some(stop))
        }
        ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => {
            // 6. 1 字节工厂标志。
            let flag = trial.read_bytes(1)?[0];
            // 7. 标志非 0:原样 16 字节工作空间工厂 CLSID(原生经
            //    AfCreateInstance 创建,读取器不创建实例,原样保留)。
            let factory_class_id = if flag != 0 {
                Some(trial.read_bytes(16)?.try_into().expect("固定 16 字节"))
            } else {
                None
            };
            // 8. 尾随 int32。
            let trailing = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
            (Some(flag), factory_class_id, Some(trailing), None)
        }
    };
    if stop.is_some() {
        // 在 IPropertySet 新对象处停止:该对象体未闭合、无法跳过,游标与登
        // 记表回退到槽位起点(回退到对象头起点的停止不保留该键——与
        // GroupLayer/CGL 读取器的回退语义一致)。
        trial = reposition_to(reader, slot_offset);
        staged_table = saved_table;
    }

    let result = WorkspaceNamePrefix10_1 {
        key,
        version,
        path,
        secondary_path,
        type_description,
        property_set,
        factory_flag,
        factory_class_id,
        trailing_int32,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 把游标回退到入口窗口内的绝对偏移（Reader 无随机定位原语，用入口克隆
/// 前向读取实现；目标必在已读范围内）。
fn reposition_to<'a>(origin: &Reader<'a>, target: usize) -> Reader<'a> {
    let mut positioned = origin.clone();
    positioned
        .read_bytes(target - origin.position())
        .expect("回退目标在停止前的已读位置，必在窗口内");
    positioned
}

/// 从对象头之后的位置读取 10.1 FeatureClassName 对象体，失败时不推进游
/// 标、不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`FEATURE_CLASS_NAME_CLASS_ID`] 并取
/// 得键 `key`。选择子分支中 IDatasetName/ITopologyName 槽的新对象、
/// WorkspaceName 体内的停止均为成功返回（停止原因
/// [`FeatureClassNameStop10_1`]，其中 WorkspaceName 停止时游标停在其停止
/// 点）；`stop == None` 表示体已完整消费（样本形态）。
pub fn read_feature_class_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FeatureClassNamePrefix10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FeatureClassName 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 版本 short:原生有符号比较(汇编 `2 < (short)ver`)。负值位模式一并
    //    拒绝(负值版本无业务意义,较原生更严)。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_FCN_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 FeatureClassName::Load 接受范围（>2 拒绝；负值位模式一并拒绝）"
        )));
    }

    // 2-5. 四个字符串:名称、第二名称、类型描述、数据集名。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let secondary_name = read_string_10_1(&mut trial, max_bytes)?;
    let type_description = read_string_10_1(&mut trial, max_bytes)?;
    let dataset_name = read_string_10_1(&mut trial, max_bytes)?;

    // 6-7. int32 ×2 与选择子。
    let int32_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let int32_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let selector = trial.read_u16_le()?;

    // 选择子分支。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let mut stop = None;
    let nested = if selector == 0 {
        // IWorkspaceName 槽(样本分支)。
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        match entry {
            ObjectEntry10_1::NewObject { class_id, key } if class_id == WORKSPACE_NAME_CLASS_ID => {
                let ws =
                    read_workspace_name_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
                if let Some(WorkspaceNameStop10_1::PropertySetNewObject {
                    class_id: prop_class,
                    key: prop_key,
                }) = ws.stop
                {
                    stop = Some(FeatureClassNameStop10_1::WorkspacePropertySetNewObject {
                        class_id: prop_class,
                        key: prop_key,
                    });
                }
                FeatureClassNameNested10_1::WorkspaceName(WorkspaceNameEntry10_1::NewObject(
                    Box::new(ws),
                ))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 非 WorkspaceName 类的新对象:体未知且无外部长度,停止并回退
                // 到槽位起点。
                stop = Some(FeatureClassNameStop10_1::WorkspaceNameUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = saved_table.clone();
                FeatureClassNameNested10_1::WorkspaceName(WorkspaceNameEntry10_1::Unsupported {
                    class_id,
                    key,
                })
            }
            other => FeatureClassNameNested10_1::WorkspaceName(match other {
                ObjectEntry10_1::Null => WorkspaceNameEntry10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    WorkspaceNameEntry10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { .. } => unreachable!("上方已匹配新对象分支"),
            }),
        }
    } else {
        // IDatasetName 槽：新 FeatureDatasetName 对象读取其闭合体（dataset_name
        // 模块）；其他类停止并回退到槽位起点。
        let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
        let slot = match entry {
            ObjectEntry10_1::Null => DatasetNameEntry10_1::Null,
            ObjectEntry10_1::BackReference { class_id, key } => {
                DatasetNameEntry10_1::BackReference { class_id, key }
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == FEATURE_DATASET_NAME_CLASS_ID =>
            {
                let body = read_feature_dataset_name_body_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged_table,
                    key,
                )?;
                if let Some(inner) = body.stop {
                    stop = Some(FeatureClassNameStop10_1::DatasetNameBodyStopped(inner));
                }
                DatasetNameEntry10_1::NewObject(Box::new(body))
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 非 FeatureDatasetName 类：体未知且无外部长度，停止并回退。
                stop = Some(FeatureClassNameStop10_1::DatasetNameNewObject { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = saved_table.clone();
                DatasetNameEntry10_1::Unsupported { class_id, key }
            }
        };
        FeatureClassNameNested10_1::DatasetName(slot)
    };

    // 8. 版本 >1 的第二选择子(前置停止时不读取)。
    let mut version1_selector = None;
    let mut topology_slot = None;
    if stop.is_none() && (version as i16) > 1 {
        version1_selector = Some(trial.read_u16_le()?);
        if version1_selector != Some(0) {
            let topo_offset = trial.position();
            // 此前已消费的对象(如 WorkspaceName 体)的登记不受本槽位回退影响。
            let pre_topology_table = staged_table.clone();
            let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
            let slot = match entry {
                ObjectEntry10_1::Null => TopologyNameEntry10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    TopologyNameEntry10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == TOPOLOGY_NAME_CLASS_ID =>
                {
                    let body = read_topology_name_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged_table,
                        key,
                    )?;
                    if let Some(inner) = body.stop {
                        stop = Some(FeatureClassNameStop10_1::TopologyNameBodyStopped(inner));
                    }
                    TopologyNameEntry10_1::NewObject(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    // 非 TopologyName 类：体未知且无外部长度，停止并回退。
                    stop = Some(FeatureClassNameStop10_1::TopologyNameNewObject { class_id, key });
                    trial = reposition_to(reader, topo_offset);
                    staged_table = pre_topology_table;
                    TopologyNameEntry10_1::Unsupported { class_id, key }
                }
            };
            topology_slot = Some(slot);
        }
    }

    let result = FeatureClassNamePrefix10_1 {
        key,
        version,
        name,
        secondary_name,
        type_description,
        dataset_name,
        int32_a,
        int32_b,
        selector,
        nested,
        version1_selector,
        topology_slot,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 FeatureLayer 对象体的已确认前缀，失败时不
/// 推进游标、不修改登记表。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`FEATURE_LAYER_CLASS_ID`] 确认类身份，并从对象头之后的位置传
/// 入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：应用于全部字符串（名称、FCN/WorkspaceName 体内
/// 各字符串）的声明长度（含终止零、不含 4 字节前缀），在任何读取与分配之
/// 前检查；`usize::MAX` 表示不额外设限，此时超长流按短读失败。
///
/// 数据源条目经 [`crate::storage::object_refs::read_object_entry_10_1`] 分类并登记：
/// 16 零字节为空引用；键已见为回引（无体）；新对象且类标识为
/// FeatureClassName 时读取其完整已确认体（体内的嵌套停止传播为本读取器的
/// [`FeatureLayerStop10_1::DataSource`] 成功停止）；其他类标识返回
/// [`ErrorKind::Unsupported`]——嵌套对象体没有外部长度字段，本读取器无法
/// 跳过未知类继续解析，调用者不能把该流继续当作 FeatureLayer 前缀消费。
/// 分类依赖登记表状态：空表会把实际的回引分类为新对象（流内无法局部区分，
/// 与原生一致）。登记先于对象体解析是原生语义；为保证「失败不修改登记
/// 表」，登记先发生在调用者登记表的副本上，全部成功后才整体提交。
///
/// 全部字段在 trial 游标上读取，完整成功才提交（停止点也是成功）；任何失
/// 败路径都不推进调用者游标。错误位置相对于传入 Reader，调用方需另外记录
/// 流和对象身份。这是严格安全子集，不代表完整原生 Load 行为，通过也不代表
/// MXD 格式已验证。
pub fn read_feature_layer_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<FeatureLayerPrefix10_1> {
    // 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进原游标。
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 2 字节架构版本：无符号解释，>34 直接拒绝（原生 >0x22 拒绝）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "FeatureLayer 对象体 offset 0x{entry_offset:x}: 架构版本 {version} 超出 \
                 FeatureLayer::Load 接受范围（>0x22/34 直接拒绝）"
            ),
        ));
    }

    // 2/3. 两个名称字符串：AfCore String 布局（证据见模块文档），各自受
    // max_bytes 限额约束。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let secondary_name = read_string_10_1(&mut trial, max_bytes)?;

    // 4. int16 ×3：+0x128 可见性（VARIANT_BOOL，解释辅助见 visibility()）、
    // +0x12a/+0x12c 语义未定，均按原始位模式保留、不校验。
    let visibility_raw = trial.read_u16_le()?;
    let unknown_short_a = trial.read_u16_le()?;
    let unknown_short_b = trial.read_u16_le()?;

    // 5. 嵌套数据源条目：object_refs 的分类状态机（Null/回引/新对象）。
    // 登记先于对象体解析是原生语义；为满足「失败不修改登记表」，分类与
    // 登记先发生在调用者登记表的副本上，全部成功后才整体提交。
    let nested_offset = trial.position();
    let mut staged_table = table.clone();
    let mut fcn_stop = None;
    let data_source = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => DataSource10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            DataSource10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == FEATURE_CLASS_NAME_CLASS_ID => {
            // FeatureClassName 完整已确认体(第六轮闭合)。体内停止传播为本
            // 读取器的 DataSource 停止;截断/版本超限等错误向上传播。
            let fcn =
                read_feature_class_name_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
            fcn_stop = fcn.stop;
            DataSource10_1::FeatureClassName(Box::new(fcn))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 严格子集只认 FeatureClassName 数据源。新对象体没有外部长度字
            // 段，无法跳过未知类继续读取；显式报错，原游标与登记表不动
            // （键虽已被分类状态机登记进副本，副本随失败丢弃）。
            let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "FeatureLayer 数据源对象 offset 0x{nested_offset:x}: 类标识 {id}（键 \
                     {key}）不是 FeatureClassName；本读取器仅支持 FeatureClassName 数据源，\
                     且对象体无外部长度字段，无法跳过未知类继续读取"
                ),
            ));
        }
    };

    // 6. 渲染器槽位（请求 IID {40A9E884-…}=IFeatureRenderer）：仅当数据源
    //    已完整消费（含 FCN 体内无停止）时到达。新 SimpleRenderer 对象读取
    //    其闭合体；其他渲染器类在对象头处停止。
    let mut renderer = None;
    let mut guid_after_renderer = None;
    let stop = match fcn_stop {
        Some(stop) => FeatureLayerStop10_1::DataSource(stop),
        None => {
            let slot_offset = trial.position();
            let saved_table = staged_table.clone();
            let entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
            let mut renderer_stop = None;
            let slot = match entry {
                ObjectEntry10_1::Null => RendererSlot10_1::Null,
                ObjectEntry10_1::BackReference { class_id, key } => {
                    RendererSlot10_1::BackReference { class_id, key }
                }
                ObjectEntry10_1::NewObject { class_id, key }
                    if class_id == SIMPLE_RENDERER_CLASS_ID =>
                {
                    let body = read_simple_renderer_body_10_1(
                        &mut trial,
                        max_bytes,
                        &mut staged_table,
                        key,
                    )?;
                    if let Some(inner) = body.stop {
                        renderer_stop = Some(FeatureLayerStop10_1::RendererBody(inner));
                    }
                    RendererSlot10_1::SimpleRenderer(Box::new(body))
                }
                ObjectEntry10_1::NewObject { class_id, key } => {
                    // 其他渲染器类：体未知且无外部长度，回退到槽位起点（不
                    // 保留该键），其后字段不可达。
                    staged_table = saved_table;
                    trial = reposition_to(reader, slot_offset);
                    renderer_stop = Some(FeatureLayerStop10_1::RendererNewObject { class_id, key });
                    RendererSlot10_1::Unsupported { class_id, key }
                }
            };
            renderer = Some(slot);
            match renderer_stop {
                Some(stop) => stop,
                None => {
                    // 7. +0x144 原样 16 字节 GUID（非对象头）。其后字段与样本
                    //    存在不可调和差异（开放项），读到 GUID 即停止。
                    guid_after_renderer =
                        Some(trial.read_bytes(16)?.try_into().expect("固定 16 字节"));
                    FeatureLayerStop10_1::AfterDataSourceGuid
                }
            }
        }
    };

    let result = FeatureLayerPrefix10_1 {
        version,
        name,
        secondary_name,
        visibility_raw,
        unknown_short_a,
        unknown_short_b,
        data_source,
        renderer,
        guid_after_renderer,
        stop,
        remaining_bytes: trial.remaining(),
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}
