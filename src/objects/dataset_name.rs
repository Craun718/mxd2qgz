//! ArcMap 10.1 FeatureClassName 体 IDatasetName / ITopologyName 选择子分支
//! 新对象体（FeatureDatasetName / TopologyName）的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-dataset-name.md`（2026-11-14 本轮研究记录）与
//! `docs/arcmap-10.1-map-object.md` 第六轮（FeatureClassName 体闭合）。二进制
//! 证据为 GdbCore.dll 10.1.0.3035（SHA-256 `ba5d8f5d…39ca1c4`，提取路径
//! `temp/agent-dataset-name/`，身份经 pefile VS_FIXEDFILEINFO 与 SHA-256 复核）
//! 的静态反编译：定位沿用第六轮方法（CLSID 数据引用 → GetClassID → 同虚表
//! +0x14/+0x18 槽），虚表归属经 RTTI 完整对象定位符链确认。
//!
//! ## 背景：FeatureClassName 体（第六轮已闭合）的两个未闭合槽
//!
//! FCN 体（`FdbFeatureClassName::Load 0x709b5080`，版本 ≤2）第 ⑦ 步为
//! int16 选择子：==0 → LoadObject（请求 IID
//! `{FADD975C-E36F-11D1-AA81-00C04FA33A15}`=IWorkspaceName，类型库定名）；
//! !=0 → LoadObject（`{89E75917-C287-11D1-AA77-00C04FA33A15}`=IDatasetName）
//! 并对 obj+0x3c 的 IDatasetName 虚表 +0x20 调用 WorkspaceName getter。
//! 第 ⑧ 步（版本 >1）为第二选择子：
//! !=0 → LoadObject（`{505B6FDD-9E6E-42F9-B705-53355B2F1DEF}`=ITopologyName）。
//! 三个 IID 的文件字节本轮逐一复核（`DAT_70ab8554`/`DAT_70ab8574`/
//! `DAT_70ac9a70`）。选择子为 16 位值、原生只区分零与非零，无其他取值语义。
//!
//! ## FeatureDatasetName 体（本轮闭合）
//!
//! 类身份三重印证：esri.clsid.ecfg 声明 CLSID
//! `{198846CF-CA42-11D1-AA7C-00C04FA33A15}` → 类名 FeatureDatasetName（模块
//! GdbCore.dll）；GetClassID `0x709b62c0` 写出同一 CLSID；RTTI TD `0x70b34a90`
//! 名 `.?AVFdbFeatureDatasetName@@`（10.1 C++ 实现类，经 COL `0x70af5124` 与
//! IPersistStream 虚表 `0x70ad04ec` 相连；+0x0c=GetClassID、+0x14=Load
//! `0x709b7c90`、+0x18=Save `0x709b7d50`，槽序与 IPersistStream 一致）。
//! 字段序（Load/Save 逐字段对称，汇编核对）：
//!
//! 1. int16 版本：`cmp word,1; jle` **有符号**，>1 拒绝（0x80004005）；
//!    Save 恒写静态常量 1（`DAT_70aaac34`）；
//! 2. AfCore String ×4 → IPersistStream this+0x0c/+0x10/+0x14/+0x18；
//!    第 1、3 项为 IDatasetName 的 Name、Category，第 2 项为 IName.NameString；
//! 3. 1 字节标志 → IPersistStream this+0x1c，即完整对象 +0x48；
//!    `IFeatureDatasetNameImpl` 接口视图（完整对象 +0x20）的 getter/setter
//!    分别位于虚表 +0x14/+0x18，直接访问接口 this+0x28；getter 仅将字节
//!    `1` 映射为 `VARIANT_TRUE`，setter 仅将 `VARIANT_TRUE`（-1）写为 1，
//!    其他值写为 0。流读写不经过该规范化访问器，故解析仍保留原始字节；
//!    私有成员名、生产者及业务用途未定；
//! 4. LoadObject（请求 IID 与 FCN 选择子 ==0 分支同值 = IWorkspaceName）
//!    → IPersistStream this+0x20；条目经 [`crate::storage::object_refs::read_object_entry_10_1`]
//!    分类，新对象且类标识为 WorkspaceName 时读取其已闭合体（复用
//!    [`crate::objects::workspace_name::read_workspace_name_body_10_1`]，第六轮）。
//!
//! ## TopologyName 体（本轮闭合）
//!
//! 类身份三重印证：esri.clsid.ecfg 声明 CLSID
//! `{9FCA50EC-C413-440C-B453-49A591440096}` → 类名 TopologyName（模块
//! GdbCore.dll）；GetClassID `0x7095dee0` 写出同一 CLSID；RTTI TD `0x70b33680`
//! 名 `.?AVTopologyName@@`（COL `0x70af02fc`，IPersistStream 虚表
//! `0x70aca460`：+0x14=Load `0x7095e5c0`、+0x18=Save `0x7095e550`）。字段序
//! （Load/Save 对称）：
//!
//! 1. int16 版本：`cmp word,1; jg fail` **有符号**，>1 拒绝；Save 恒写静态
//!    常量 1（`DAT_70ab7aa0`）；
//! 2. AfCore String ×2 → IPersistStream this+0x0c/+0x10，分别对应
//!    IDatasetName 的 Name、Category；
//! 3. LoadObject（请求 IID 与 FCN 选择子 !=0 分支同值 = IDatasetName）
//!    → IPersistStream this+0x14，装载前先释放旧指针；装载成功后对槽内对象做虚表
//!    +0x20 调用 IDatasetName.WorkspaceName getter；该虚调用的具体副作用
//!    取决于槽内对象，不能一概断言不读流。条目分类同上；新对象按支持的类
//!    标识分派，未知类为停止点。
//!
//! ## 样本对位（辅助观察）
//!
//! `temp/r6-maps.bin`（152941 字节）全流扫描：FeatureDatasetName 与
//! TopologyName 的 CLSID 均 0 命中；11 个 FeatureClassName 体选择子全部为 0
//! （IWorkspaceName 分支），其中 10 个可完整遍历的版本 >1 第二选择子全部为
//! 0（FCN@0x218bc 的工作空间体内嵌真 PropertySet 新对象键 2027，遍历在其槽
//! 位停止，第二选择子不可达）。**两个新对象体在样本中均未出现，本模块全部
//! 布局结论无样本字节佐证，仅有反编译一手证据**；单元测试全部使用合成输入。
//!
//! 调用约定：与 [`crate::layers::feature_layer`] 一致——调用者先用
//! [`crate::storage::object_stream::read_header_10_1`] 读取对象头、比对类标识常量并
//! 取得键，从对象头之后的位置传入 Reader；本模块不重复读取对象头。嵌套条目
//! 登记 先于对象体解析是原生语义；为满足「失败不修改登记表」，分类与登记
//! 先发生在调用者登记表的副本上，全部成功后才整体提交。
//!
//! 这是仅覆盖上述已闭合字段的严格安全子集：长度/偏移/溢出经
//! [`crate::stream::reader::Reader`] 检查；有符号版本大于 1 时拒绝；未知选择子语义不存在（原生只区分零/非零）；首个
//! 未闭合字段（槽位新对象非预期类、WorkspaceName 体内 PropertySet 新对象）
//! 处停止并保留剩余；任何失败不推进调用者游标、不修改登记表。未接入 MXD
//! 解析（`feature_layer` 的 FCN 读取器在两个槽位的新对象处停止的既有行为
//! 未变，接线属后续共享文件改动），未运行目标软件，人工目视验收未做，
//! 不得声称已验证。
//!
//! 未解决问题（详见证据文档「开放项」）：
//! - FeatureDatasetName 的第 4 个 String 已确认由 `IFeatureDatasetNameImpl`
//!   虚表 +0x0c/+0x10 访问，接口 this+0x24（完整对象 +0x44）；标志的
//!   访问器位于 +0x14/+0x18、接口 this+0x28（完整对象 +0x48）。私有成员名、
//!   实际生产者/消费者及业务用途未定，流字段仍原样保留；
//! - IDatasetName 槽可能出现的其他 IDatasetName 实现类（UnknownDatasetName、
//!   GeometricNetworkName、FeatureClassName 自身等，esri.clsid 声明）未闭合，
//!   读取器在其对象头处停止；
//! - TopologyName 嵌套对象的 WorkspaceName getter 的动态副作用仍待验证；
//! - 版本 0 分支无字节佐证（原生接受，Save 恒写 1）；
//! - 全部结论为静态观察，未经目标软件运行验证。

use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNamePrefix10_1, WorkspaceNameStop10_1,
    read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// FeatureDatasetName / TopologyName::Load 接受的最大版本：汇编
/// `cmp word,1; jle/jg` 有符号比较，>1 拒绝；两类的 Save 恒写 1。
const MAX_BODY_VERSION: i16 = 1;

/// CoverageFDB.dll 0x6e0c3320 返回的 CoverageName CLSID。
pub const COVERAGE_NAME_CLASS_ID: [u8; 16] = [
    0x72, 0x1c, 0x92, 0x1e, 0x2f, 0x12, 0xd3, 0x11, 0x9f, 0x31, 0, 0xc0, 0x4f, 0x79, 0x92, 0x7c,
];

/// FeatureDatasetName 类标识（CLSID）的 16 字节内存序（Data1-3 小端、
/// Data4 原序）。
///
/// GUID 字符串形式为 `{198846CF-CA42-11D1-AA7C-00C04FA33A15}`（Data1=
/// 0x198846CF → `cf 46 88 19`，Data2=0xCA42 → `42 ca`，Data3=0x11D1 →
/// `d1 11`，Data4 原序 `aa 7c 00 c0 4f a3 3a 15`；与 FeatureClassName
/// `{198846D0-…}` 相邻同族）。证据：安装配置 esri.clsid.ecfg 声明（模块
/// GdbCore.dll，类名 FeatureDatasetName）；GdbCore.dll GetClassID
/// `0x709b62c0` 写出同一 CLSID（IPersistStream 虚表 `0x70ad04ec` +0x0c）；
/// RTTI TD `0x70b34a90` 名 `.?AVFdbFeatureDatasetName@@` 经 COL `0x70af5124`
/// 与该虚表相连。仓库参考样本 /Maps 流 0 命中（该类新对象体在样本中未出现）。
pub const FEATURE_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0xcf, 0x46, 0x88, 0x19, 0x42, 0xca, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3, 0x3a, 0x15,
];

/// TopologyName 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{9FCA50EC-C413-440C-B453-49A591440096}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll，类名 TopologyName）；GdbCore.dll
/// GetClassID `0x7095dee0` 写出同一 CLSID（IPersistStream 虚表 `0x70aca460`
/// +0x0c）；RTTI TD `0x70b33680` 名 `.?AVTopologyName@@` 经 COL `0x70af02fc`
/// 与该虚表相连。仓库参考样本 /Maps 流 0 命中。
pub const TOPOLOGY_NAME_CLASS_ID: [u8; 16] = [
    0xec, 0x50, 0xca, 0x9f, 0x13, 0xc4, 0x0c, 0x44, 0xb4, 0x53, 0x49, 0xa5, 0x91, 0x44, 0x00, 0x96,
];

/// FeatureDatasetName 体第 4 步 IWorkspaceName 槽位的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceNameSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。类标识原样保留、不校验（回引头重复
    /// 被引对象的类标识，见 [`crate::layers::feature_layer`] 模块文档）。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识为 WorkspaceName：已读取其闭合体（第六轮读取器）；
    /// 体内停止时携带原因。
    WorkspaceName(Box<WorkspaceNamePrefix10_1>),
    /// 新对象但类标识不是 WorkspaceName：体未知且无外部长度，停止点，
    /// 游标回退到该对象头起点、该键不登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// FeatureDatasetName 体读取的停止原因（体内在首个不可闭合处停止，非错误）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureDatasetNameStop10_1 {
    /// IWorkspaceName 槽的新对象类标识不是 WorkspaceName（体未知）。
    WorkspaceUnsupported { class_id: [u8; 16], key: u32 },
    /// WorkspaceName 体内 IPropertySet 槽的新对象体（字段序未闭合）：
    /// 其后的标志、工厂 CLSID 与 int32 不可达（第六轮已知停止点）。
    WorkspacePropertySetNewObject { class_id: [u8; 16], key: u32 },
}

/// FeatureDatasetName（GdbCore.dll Load `0x709b7c90`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureDatasetNameBody10_1 {
    /// 对象头键。解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 int16 版本。原生有符号比较 >1 拒绝（汇编证实）；Save 恒写 1。
    /// 样本无实例，接受域内位模式均可能。
    pub version: u16,
    /// 第一个字符串（IPersistStream this+0x0c；IDatasetName.Name）。
    pub string_a: String,
    /// 第二个字符串（IPersistStream this+0x10；IName.NameString）。
    pub string_b: String,
    /// 第三个字符串（IPersistStream this+0x14；IDatasetName.Category）。
    pub string_c: String,
    /// 要素类的类别描述（IPersistStream this+0x18，即完整对象 +0x44）。
    /// FileGDB 和 SdeFDB 通过 IFeatureDatasetNameImpl 的 +0x10 写工作空间对应的类别文本。
    pub string_d: String,
    /// 1 字节标志（IPersistStream this+0x1c，即完整对象 +0x48）。
    /// `IFeatureDatasetNameImpl` 接口视图位于完整对象 +0x20，其虚表 +0x14
    /// getter 将字节值恰为 1 映射为 `VARIANT_TRUE`，+0x18 setter 将输入
    /// `VARIANT_TRUE`（-1）映射为字节 1、其余输入映射为 0。Load/Save 直接
    /// 读写该字节，不经过访问器；因此保留原始值。枚举时真值将该名称作为单一要素类。
    pub flag: u8,
    /// 第 4 步 IWorkspaceName 槽（请求 IID `{FADD975C-…}`）分类结果。
    pub workspace: WorkspaceNameSlot10_1,
    /// 已确认的 WorkspaceName 回引路径。
    pub workspace_back_reference_path: Option<String>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FeatureDatasetNameStop10_1>,
    /// CoverageName 外层版本及原始 LONG。
    pub coverage_wrapper: Option<(u16, i32)>,
}

impl FeatureDatasetNameBody10_1 {
    /// 返回工作空间提供的要素类类别描述。原生文本可能随语言环境变化。
    pub fn feature_class_category(&self) -> &str {
        &self.string_d
    }

    /// 返回原生枚举器是否按此对象自身的名称枚举单一要素类。
    pub fn enumerates_as_single_feature_class(&self) -> bool {
        self.flag == 1
    }

    /// 返回已解析 WorkspaceName 体或已确认回引中的路径原文。
    ///
    /// 已知 WorkspaceName 即使在 PropertySet 子槽处停止，路径字段仍已先行
    /// 完整读取，因此仍可返回；调用方可同时检查 [`Self::stop`] 保留诊断。
    /// 返回 `None` 时，空、未解析回引和未知类等状态仍由 [`Self::workspace`] 原样
    /// 保留，调用方应检查该槽以输出相应诊断。路径不做规范化或本机存在性判断。
    pub fn workspace_path(&self) -> Option<&str> {
        match &self.workspace {
            WorkspaceNameSlot10_1::WorkspaceName(workspace) => Some(&workspace.path),
            WorkspaceNameSlot10_1::BackReference { .. } => {
                self.workspace_back_reference_path.as_deref()
            }
            WorkspaceNameSlot10_1::Null | WorkspaceNameSlot10_1::Unsupported { .. } => None,
        }
    }
}

/// TopologyName 体第 3 步 IDatasetName 槽位的分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DatasetNameSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识为 FeatureDatasetName：已读取其体（本模块）；体内
    /// 停止时携带原因。
    FeatureDataset(Box<FeatureDatasetNameBody10_1>),
    /// 新对象且类标识为 B 轮闭合的六类之一（GeometricNetworkName/
    /// RelationshipClassName/MemoryRelationshipClassName/QueryTableName/
    /// RelQueryTableName/DBTableName）：已读取其体
    /// （[`crate::objects::dataset_name_more`]，B 轮接线）；体内停止经
    /// [`TopologyNameStop10_1`] 透传。
    More(Box<crate::objects::dataset_name_more::DatasetNameMoreBody10_1>),
    /// 新对象但类标识无已闭合读取器（TableName / UnknownDatasetName /
    /// FeatureClassName 自身等）：停止点，游标回退到该对象头起点、该键不
    /// 登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// TopologyName 体读取的停止原因。
#[derive(Debug, Clone, PartialEq)]
pub enum TopologyNameStop10_1 {
    /// IDatasetName 槽的新对象类标识无已闭合读取器（体未知）。
    DatasetNameUnsupported { class_id: [u8; 16], key: u32 },
    /// 槽内 FeatureDatasetName 体自身的停止点（如 WorkspaceName 体内
    /// PropertySet 新对象），原因透传。
    FeatureDataset(FeatureDatasetNameStop10_1),
    /// 槽内 B 轮六类体自身的停止点（原因透传，见
    /// [`crate::objects::dataset_name_more::DatasetNameMoreStop10_1`]）：
    /// `SlotUnsupported`/`WorkspacePropertySet` 游标停在对应槽位起点或其
    /// 停止点，`Nested` 游标停在该嵌套读取器的停止点。
    MoreNested(crate::objects::dataset_name_more::DatasetNameMoreStop10_1),
}

/// TopologyName（GdbCore.dll Load `0x7095e5c0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TopologyNameBody10_1 {
    /// 对象头键。解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 int16 版本。原生有符号比较 >1 拒绝（汇编证实）；Save 恒写 1。
    pub version: u16,
    /// 第一个字符串（IPersistStream this+0x0c；IDatasetName.Name）。
    pub string_a: String,
    /// 第二个字符串（IPersistStream this+0x10；IDatasetName.Category）。
    pub string_b: String,
    /// 第 3 步 IDatasetName 槽，亦对应 ITopologyName.FeatureDatasetName。
    pub dataset: DatasetNameSlot10_1,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<TopologyNameStop10_1>,
}

/// 从对象头之后的位置读取 10.1 FeatureDatasetName 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对象
/// 头、比对 [`FEATURE_DATASET_NAME_CLASS_ID`] 并取得键 `key`，从对象头之后的
/// 位置传入 Reader。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：应用于四个字符串的声明长度（与
/// [`crate::layers::feature_layer::read_feature_layer_prefix_10_1`] 同口径），并原样
/// 转发给嵌套 WorkspaceName 体；`usize::MAX` 表示不额外设限。
///
/// IWorkspaceName 槽的新对象非 WorkspaceName 类、或 WorkspaceName 体内
/// PropertySet 新对象，均为成功返回（停止原因
/// [`FeatureDatasetNameStop10_1`]，前者游标回退到槽位起点，后者游标停在
/// WorkspaceName 读取器的停止点）；截断、版本超限、字符串超限为错误，游标
/// 与登记表不动。
pub fn read_feature_dataset_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FeatureDatasetNameBody10_1> {
    let body = read_feature_dataset_name_payload(reader, max_bytes, table, key)?;
    if body.stop.is_none()
        && let Some(path) = body.workspace_path()
    {
        table.mark_feature_dataset_path(key, path);
    }
    Ok(body)
}

fn read_feature_dataset_name_payload(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FeatureDatasetNameBody10_1> {
    // 全部字段在 trial 游标与 trial 登记表上读取，完整成功（含停止点）才提交。
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FeatureDatasetName 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 原生用有符号 short 比较版本上限 1。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION {
        return Err(invalid(format!(
            "架构版本 {version} 超出 FeatureDatasetName::Load 接受范围（有符号 >1 拒绝）"
        )));
    }

    // 2. 四个字符串：语义未定（候选：名称/第二名称/类型描述/数据集名）。
    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;
    let string_c = read_string_10_1(&mut trial, max_bytes)?;
    let string_d = read_string_10_1(&mut trial, max_bytes)?;

    // 3. 1 字节标志（原生 bool，语义未定，原样保留）。
    let flag = trial.read_bytes(1)?[0];

    // 4. IWorkspaceName 槽（请求 IID 与 FCN 选择子 ==0 分支同值）。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let mut stop = None;
    let mut entry = read_object_entry_10_1(&mut trial, &mut staged_table)?;
    if let ObjectEntry10_1::BackReference { class_id, key } = &mut entry
        && let Some(registered) = staged_table.registered_class(*key)
    {
        *class_id = registered;
    }
    let workspace = match entry {
        ObjectEntry10_1::Null => WorkspaceNameSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key }
            if staged_table.registered_class(key).is_some()
                && class_id != WORKSPACE_NAME_CLASS_ID =>
        {
            stop = Some(FeatureDatasetNameStop10_1::WorkspaceUnsupported { class_id, key });
            trial = reposition_to(reader, slot_offset);
            WorkspaceNameSlot10_1::Unsupported { class_id, key }
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            WorkspaceNameSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == WORKSPACE_NAME_CLASS_ID => {
            // 新 WorkspaceName 对象：读取其闭合体（第六轮读取器）。体内的
            // PropertySet 新对象停止透传为本体的停止原因；其余错误向上传播。
            let body =
                read_workspace_name_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
            if let Some(WorkspaceNameStop10_1::PropertySetNewObject {
                class_id: prop_class,
                key: prop_key,
            }) = body.stop
            {
                stop = Some(FeatureDatasetNameStop10_1::WorkspacePropertySetNewObject {
                    class_id: prop_class,
                    key: prop_key,
                });
            }
            WorkspaceNameSlot10_1::WorkspaceName(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 非 WorkspaceName 类的新对象：体未知且无外部长度，停止并回退到
            // 槽位起点（回退停止不保留该键，与 FCN 读取器口径一致）。
            stop = Some(FeatureDatasetNameStop10_1::WorkspaceUnsupported { class_id, key });
            trial = reposition_to(reader, slot_offset);
            staged_table = saved_table;
            WorkspaceNameSlot10_1::Unsupported { class_id, key }
        }
    };

    let workspace_back_reference_path = match &workspace {
        WorkspaceNameSlot10_1::BackReference { class_id, key }
            if *class_id == WORKSPACE_NAME_CLASS_ID =>
        {
            staged_table.workspace_path(*key).map(str::to_owned)
        }
        _ => None,
    };
    let result = FeatureDatasetNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        string_c,
        string_d,
        flag,
        workspace,
        workspace_back_reference_path,
        stop,
        coverage_wrapper: None,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// CoverageFDB.dll 0x6e0c3360 读取版本和 LONG 后委派给内联 FeatureDatasetName。
pub(crate) fn read_coverage_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FeatureDatasetNameBody10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "CoverageName 版本不支持",
        ));
    }
    let long = trial.read_u32_le()? as i32;
    let mut body = read_feature_dataset_name_payload(&mut trial, max_bytes, table, key)?;
    body.coverage_wrapper = Some((version, long));
    *reader = trial;
    Ok(body)
}

/// 从对象头之后的位置读取 10.1 TopologyName 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`TOPOLOGY_NAME_CLASS_ID`] 并取得键
/// `key`。IDatasetName 槽的新对象非 FeatureDatasetName 类为成功返回的停止点
/// （游标回退到槽位起点）；嵌套 FeatureDatasetName 体内的停止透传为
/// [`TopologyNameStop10_1::FeatureDataset`]（游标停在其停止点）。
pub fn read_topology_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TopologyNameBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("TopologyName 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 版本 short：原生 `cmp word,1; jg fail` 有符号比较（>1 拒绝）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION {
        return Err(invalid(format!(
            "架构版本 {version} 超出 TopologyName::Load 接受范围（有符号 >1 拒绝）"
        )));
    }

    // 2. 两个字符串：语义未定（候选：名称/第二名称）。
    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;

    // 3. IDatasetName 槽（请求 IID 与 FCN 选择子 !=0 分支同值）。原生在此
    //    之前释放旧指针、装载成功后做虚表 +0x20 通知调用（不读流）。B 轮
    //    起复用 [`crate::objects::dataset_name_more`] 的七类统一分派：
    //    FeatureDatasetName 走本模块读取器（既有臂），六类走 B 轮读取器，
    //    停止透传映射见各变体文档。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let mut stop = None;
    let mut more_stop = None;
    let dataset = {
        let slot = crate::objects::dataset_name_more::read_dataset_name_slot_10_1(
            &mut trial,
            max_bytes,
            &mut staged_table,
            &mut more_stop,
        )?;
        match slot {
            crate::objects::dataset_name_more::DatasetNameMoreSlot10_1::Null => {
                DatasetNameSlot10_1::Null
            }
            crate::objects::dataset_name_more::DatasetNameMoreSlot10_1::BackReference {
                class_id,
                key,
            } => DatasetNameSlot10_1::BackReference { class_id, key },
            crate::objects::dataset_name_more::DatasetNameMoreSlot10_1::Body(body) => {
                // 七类统一分派：FeatureDatasetName 变体映射回本模块既有臂
                //（体读取器同源；其体内 WorkspaceName→PropertySet 停止按旧
                // 口径映射为 `FeatureDataset(WorkspacePropertySetNewObject)`，
                // 与 B 轮接线建议的「复用 FeatureDataset 变体表达」一致），
                // 其余六类进 `More`，停止经 MoreNested 透传。
                match body {
                    crate::objects::dataset_name_more::DatasetNameMoreBody10_1::FeatureDataset(
                        fd,
                    ) => {
                        if let Some(inner) = fd.stop.clone() {
                            stop = Some(TopologyNameStop10_1::FeatureDataset(inner));
                        }
                        // 统一分派对 FDN 体只透传 WorkspacePropertySet 一类停
                        // 止（其余 FDN 体内停止已在体读取器的 stop 字段）；
                        // 此处消费掉 more_stop 以免落入 MoreNested。
                        more_stop = None;
                        DatasetNameSlot10_1::FeatureDataset(fd)
                    }
                    other => DatasetNameSlot10_1::More(Box::new(other)),
                }
            }
            crate::objects::dataset_name_more::DatasetNameMoreSlot10_1::Unsupported {
                class_id,
                key,
            } => {
                // 槽位未知类：既有停止变体（B 轮映射 SlotUnsupported →
                // DatasetNameUnsupported），分派带出的 stop 一并作废。
                more_stop = None;
                stop = Some(TopologyNameStop10_1::DatasetNameUnsupported { class_id, key });
                trial = reposition_to(reader, slot_offset);
                staged_table = saved_table;
                DatasetNameSlot10_1::Unsupported { class_id, key }
            }
        }
    };
    if let Some(inner) = more_stop {
        // 六类体内的停止（SlotUnsupported/Nested/WorkspacePropertySet）：
        // 游标已由分派读取器定位（槽位起点或内层停止点），透传给宿主。
        stop = Some(TopologyNameStop10_1::MoreNested(inner));
    }

    let result = TopologyNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        dataset,
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
