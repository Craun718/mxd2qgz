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
//! 并对 obj+0x3c 虚表 +0x20 做通知调用。第 ⑧ 步（版本 >1）为第二选择子：
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
//! 2. AfCore String ×4 → obj+0x0c/+0x10/+0x14/+0x18（语义未定性，按
//!    FCN 四字符串家族类推为候选：名称/第二名称/类型描述/数据集名）；
//! 3. 1 字节标志 → obj+0x1c（原生按 bool 存储，语义未定，原样保留）；
//! 4. LoadObject（请求 IID 与 FCN 选择子 ==0 分支同值 = IWorkspaceName）
//!    → obj+0x20；条目经 [`crate::storage::object_refs::read_object_entry_10_1`]
//!    分类，新对象且类标识为 WorkspaceName 时读取其已闭合体（复用
//!    [`crate::layers::feature_layer::read_workspace_name_body_10_1`]，第六轮）。
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
//! 2. AfCore String ×2 → obj+0x0c/+0x10（语义未定性，候选：名称/第二名称）；
//! 3. LoadObject（请求 IID 与 FCN 选择子 !=0 分支同值 = IDatasetName）
//!    → obj+0x14，装载前先释放旧指针；装载成功后对 obj+0x14 对象做虚表
//!    +0x20 调用（与 FCN 数据集分支 `this+0x3c` 虚表 +0x20 同一通知模式，
//!    不读流）。条目分类同上；新对象且类标识为 FeatureDatasetName 时读取
//!    其体（本模块），其余类为停止点。
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
//! [`crate::stream::reader::Reader`] 检查；版本超出 0..=1（含负值位模式，较原生
//! 有符号比较更严）拒绝；未知选择子语义不存在（原生只区分零/非零）；首个
//! 未闭合字段（槽位新对象非预期类、WorkspaceName 体内 PropertySet 新对象）
//! 处停止并保留剩余；任何失败不推进调用者游标、不修改登记表。未接入 MXD
//! 解析（`feature_layer` 的 FCN 读取器在两个槽位的新对象处停止的既有行为
//! 未变，接线属后续共享文件改动），未运行目标软件，人工目视验收未做，
//! 不得声称已验证。
//!
//! 未解决问题（详见证据文档「开放项」）：
//! - 四个/两个 String 的业务语义未定性（按家族类推为候选），标志字节语义
//!   未定，均原样保留；
//! - IDatasetName 槽可能出现的其他 IDatasetName 实现类（UnknownDatasetName、
//!   GeometricNetworkName、FeatureClassName 自身等，esri.clsid 声明）未闭合，
//!   读取器在其对象头处停止；
//! - TopologyName 体的虚表 +0x20 通知调用（与 FCN 分支同型）的接口语义未
//!   定名；
//! - 版本 0 分支无字节佐证（原生接受，Save 恒写 1）；
//! - 全部结论为静态观察，未经目标软件运行验证。

use crate::layers::feature_layer::{
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
    /// 第一个字符串（→ obj+0x0c；语义未定，候选：名称）。
    pub string_a: String,
    /// 第二个字符串（→ obj+0x10；语义未定，候选：第二名称）。
    pub string_b: String,
    /// 第三个字符串（→ obj+0x14；语义未定，候选：类型描述）。
    pub string_c: String,
    /// 第四个字符串（→ obj+0x18；语义未定，候选：数据集名）。
    pub string_d: String,
    /// 1 字节标志（→ obj+0x1c；原生按 bool 存储，语义未定，原样保留）。
    pub flag: u8,
    /// 第 4 步 IWorkspaceName 槽（请求 IID `{FADD975C-…}`）分类结果。
    pub workspace: WorkspaceNameSlot10_1,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<FeatureDatasetNameStop10_1>,
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
    /// 新对象但类标识不是 FeatureDatasetName（UnknownDatasetName、
    /// GeometricNetworkName、FeatureClassName 等其他 IDatasetName 实现类
    /// 未闭合）：停止点，游标回退到该对象头起点、该键不登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// TopologyName 体读取的停止原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyNameStop10_1 {
    /// IDatasetName 槽的新对象类标识不是 FeatureDatasetName（体未知）。
    DatasetNameUnsupported { class_id: [u8; 16], key: u32 },
    /// 槽内 FeatureDatasetName 体自身的停止点（如 WorkspaceName 体内
    /// PropertySet 新对象），原因透传。
    FeatureDataset(FeatureDatasetNameStop10_1),
}

/// TopologyName（GdbCore.dll Load `0x7095e5c0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct TopologyNameBody10_1 {
    /// 对象头键。解析成功时该键已按原生语义登记进调用者的登记表。
    pub key: u32,
    /// 体首 int16 版本。原生有符号比较 >1 拒绝（汇编证实）；Save 恒写 1。
    pub version: u16,
    /// 第一个字符串（→ obj+0x0c；语义未定，候选：名称）。
    pub string_a: String,
    /// 第二个字符串（→ obj+0x10；语义未定，候选：第二名称）。
    pub string_b: String,
    /// 第 3 步 IDatasetName 槽（请求 IID `{89E75917-…}`）分类结果。
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
    // 全部字段在 trial 游标与 trial 登记表上读取，完整成功（含停止点）才提交。
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("FeatureDatasetName 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };

    // 1. 版本 short：原生 `cmp word,1; jle` 有符号比较（>1 拒绝）。负值位
    //    模式按原生有符号解释会通过检查，此处一并拒绝（较原生更严，与
    //    FCN/WorkspaceName 读取器口径一致）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 FeatureDatasetName::Load 接受范围（>1 拒绝；负值位模式一并拒绝）"
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
    let workspace = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => WorkspaceNameSlot10_1::Null,
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

    let result = FeatureDatasetNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        string_c,
        string_d,
        flag,
        workspace,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
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

    // 1. 版本 short：原生 `cmp word,1; jg fail` 有符号比较（>1 拒绝）；负值
    //    位模式一并拒绝（较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 TopologyName::Load 接受范围（>1 拒绝；负值位模式一并拒绝）"
        )));
    }

    // 2. 两个字符串：语义未定（候选：名称/第二名称）。
    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;

    // 3. IDatasetName 槽（请求 IID 与 FCN 选择子 !=0 分支同值）。原生在此
    //    之前释放旧指针、装载成功后做虚表 +0x20 通知调用（不读流）。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let mut stop = None;
    let dataset = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => DatasetNameSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            DatasetNameSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == FEATURE_DATASET_NAME_CLASS_ID =>
        {
            let body =
                read_feature_dataset_name_body_10_1(&mut trial, max_bytes, &mut staged_table, key)?;
            if let Some(inner) = body.stop {
                stop = Some(TopologyNameStop10_1::FeatureDataset(inner));
            }
            DatasetNameSlot10_1::FeatureDataset(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 其他 IDatasetName 实现类未闭合：停止并回退到槽位起点。
            stop = Some(TopologyNameStop10_1::DatasetNameUnsupported { class_id, key });
            trial = reposition_to(reader, slot_offset);
            staged_table = saved_table;
            DatasetNameSlot10_1::Unsupported { class_id, key }
        }
    };

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
