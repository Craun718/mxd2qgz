//! ArcMap 10.1 IDatasetName 槽实现类体族（GeometricNetworkName /
//! RelationshipClassName / MemoryRelationshipClassName / QueryTableName /
//! RelQueryTableName / DBTableName）的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-dataset-name-slots.md`（2026-11-16 本轮研究记
//! 录）。二进制证据为 GdbCore.dll 10.1.0.3035（SHA-256 `ba5d8f5d…39ca1c4`，
//! 提取路径 `temp/agent-b-dsnames/`，与第六轮基线一致）的静态反编译；类身份
//! 经 esri.clsid.ecfg 声明、GetClassID 立即数写出、RTTI TD→COL→虚表链三方
//! 互证。全部为静态观察，未运行 ArcMap 10.1。
//!
//! ## 类身份与体布局（反编译一手证据；样本全文件 16 字节 CLSID 扫描全部
//! 0 命中，**无样本字节佐证**）
//!
//! | 类 | CLSID | GetClassID | Load | Save | 版本门 | 体字段序 |
//! | --- | --- | --- | --- | --- | --- | --- |
//! | GeometricNetworkName | `{F452B4D2-9A95-11D2-AACF-00C04FA37B82}` | `0x709b8560` | `0x709b8ca0` | `0x709b8d50` | 有符号 >1 拒绝（Save 恒写 1） | 版本 + String×3 + IDatasetName 槽（obj+0x1c，装载后虚表 +0x20 通知） |
//! | RelationshipClassName | `{311C2EE1-DA65-11D2-8A54-000000000000}` | `0x70a3ab30` | `0x70a3b330` | `0x70a3b4b0` | 有符号 >1 拒绝（Save 恒写 1） | 版本 + String×3 + int32×2（obj+0x20/0x24）+ short×2（obj+0x28/0x2a）+ int16 选择子：==0 → IWorkspaceName 槽（obj+0x30）；!=0 → IDatasetName 槽（obj+0x2c，通知） |
//! | MemoryRelationshipClassName | `{6DBA211B-EBDB-11D3-9F84-00C04F6BC886}` | `0x70a402a0` | `0x70a402e0` | `0x70a40440` | 有符号 >1 拒绝（Save 恒写 1） | 版本 + String×3 + int32×2（obj+0x10/0x14）+ short×2（obj+0x18/0x1a）+ LoadObject×2（请求 IID 均为 IName `{4BA33331-D55F-11D1-8882-0000F877762D}`，obj+0x1c/0x20）+ String×4 |
//! | QueryTableName | `{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}` | `0x709329d0` | `0x70932a10` | `0x70932ae0` | **无符号** `(ver-1) ≤ 1`，仅 1/2 通过（0 与 ≥3 拒绝；Save 经 `AfStreamGetSaveVersion` 表 `0x70ab7b2c` = (ffff,ffff,ffff,ffff,1,2)） | 版本 + String×3 + IWorkspaceName 槽（obj+0x1c）+ 槽（请求 IID `{2E060684-E14C-44D3-858A-442AB6A0A863}`，obj+0x18；装载后虚表 +0x40 查询不读流） |
//! | RelQueryTableName | `{DAB3EE10-0F92-455D-8AA2-3D4ADE5B2F7D}` | `0x70a5a4f0` | `0x70a5a530` | `0x70a5a6f0` | 有符号 >3 拒绝（Save 表 `0x70a88fd8` = (2,3)，恒写 2/3） | 版本 + String×2 + 槽（`{4BA33331-…}`，obj+0x10）+ short（obj+0x14）+ 槽（`{FDFEBD93-ED75-11D0-9A95-080009EC734B}`，obj+0x18）+ 槽（`{F7AD47D1-D55E-11D1-8882-0000F877762D}`，obj+0x1c）+ String + short（obj+0x24）+ [版本 ≥2] short（obj+0x26，短体原生置内存 0）+ [版本 ≥3] short（obj+0x28） |
//! | DBTableName | `{4C9307B2-C125-4497-BA95-AAE875553B06}` | `0x708d7a00` | `0x708d7a40` | `0x708d7b20` | **无版本字段** | String×3 + IWorkspaceName 槽（obj+0x1c）+ String + int32（obj+0x14，原生 ≠7 时置内存标志）+ 槽（`{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`，obj+0x20） |
//!
//! IDatasetName 槽（请求 IID `{89E75917-C287-11D1-AA77-00C04FA33A15}`）的
//! 新对象按类标识分派：FeatureDatasetName（第六轮，[`read_feature_dataset_name_body_10_1`]）
//! 与本模块六类互相递归（深度受流长天然约束，登记先于体解析使循环呈回引）；
//! 其余类（TableName / UnknownDatasetName / FeatureClassName / 其他模块实现）
//! 为停止点。
//!
//! ## TableName / UnknownDatasetName / ObjectClassName：持久化体未闭合
//!
//! `TableName {06783DB1-E5EE-11D1-B0A2-0000F8780820}`、`UnknownDatasetName
//! {06783DB1-…082E}`、`ObjectClassName {905CEC64-0B2C-11D4-A060-00C04F6BDD84}`
//! 的具体持久化体尚未闭合。GdbCore.dll 10.1 的 ObjectClassName 注册代码
//! `0x70a808c5` 将 CLSID 与工厂虚表 `0x70ac2208` 绑定；工厂 RTTI 为
//! `ClassFactory<FdbTableName>`，工厂函数 `0x7090a240` 调用构造入口
//! `0x709b9330`，后者安装 RTTI 为 `FdbTableName` 的虚表 `0x70ad06f0`。
//! `0x709b94b0` 是析构入口，不是构造函数；该证据不证明所有持久化路径
//! 不存在。详见 `docs/mxd-round3-dataset-format-20261117.md`。
//! `FeatureQueryName {81CAA5D9-…}`/`TableQueryName
//! {81CAA5D8-…}` 的主虚表（`0x70aba700`/`0x70aba628`，槽 +0x14 GetIDType
//! 写出各自 CLSID）呈 IPersistVariant 形态，未找到 AfStream 框架的
//! IPersistStream Load/Save——持久化路径未闭合。读取器将上述类一律按
//! 「无已闭合体」停止点处理。
//!
//! 调用约定与 [`crate::objects::dataset_name`] 一致：调用者先读对象头、比对
//! 类标识常量并取得键，从对象头之后传入 Reader；嵌套条目登记先于对象体解
//! 析，全部字段在 trial 游标与 trial 登记表上读取，失败不推进游标、不修改
//! 登记表；停止点是成功返回。字符串受 `max_bytes` 限额并原样转发给嵌套体。
//!
//! 已知偏离原生：负值版本位模式拒绝（较原生有符号比较更严，与仓库既有读取
//! 器口径一致）；QueryTableName 版本 0 原生被无符号门拒绝，读取器同拒。
//! 通知调用（RCN/FGN 槽装载后的虚表 +0x20、QueryTN 的 +0x40）不消费流字
//! 节，读取器不模拟。六类标量/String 字段的业务语义未定性（字符串候选：名
//! 称/类别/连接信息等，按 FCN/WorkspaceName 家族类推），原样保留。五条请求
//! IID 已由 ArcMap 10.1 类型库定名为 IName、IQueryDescription、IQueryFilter、
//! ISelectionSet、ISpatialReference；只在中文注释标明对应请求槽，保留原公开
//! 字段名。接口名不表示流内类标识。
//! 已经由 FeatureClassName 的 IDatasetName 选择子接入 MXD 解析链；未闭合
//! 的嵌套新对象体仍是停止点。人工目视验收未做，不得声称已验证。

use crate::objects::dataset_name::{
    FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1, FeatureDatasetNameStop10_1,
    WorkspaceNameSlot10_1, read_feature_dataset_name_body_10_1,
};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNameStop10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// GeometricNetworkName 类标识（CLSID）的 16 字节内存序（Data1-3 小端、
/// Data4 原序）。
///
/// GUID 字符串形式 `{F452B4D2-9A95-11D2-AACF-00C04FA37B82}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll）；GdbCore.dll GetClassID
/// `0x709b8560` 以立即数对 `0x11d29a95f452b4d2`/`0x827ba34fc000cfaa` 写出
/// 同一 CLSID（IPersistStream 虚表 `0x70ad0648` 槽 +0x0c）；该虚表 [-1]
/// COL `0x70af54c4` 的 TD `0x70b34b48` 名 `.?AVFdbGeometricNetworkName@@`。
/// 仓库参考样本全文件扫描 0 命中（无样本字节佐证）。
pub const GEOMETRIC_NETWORK_NAME_CLASS_ID: [u8; 16] = [
    0xd2, 0xb4, 0x52, 0xf4, 0x95, 0x9a, 0xd2, 0x11, 0xaa, 0xcf, 0x00, 0xc0, 0x4f, 0xa3, 0x7b, 0x82,
];

/// RelationshipClassName 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{311C2EE1-DA65-11D2-8A54-000000000000}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll）；GetClassID `0x70a3ab30` 以立即
/// 数对 `0x11d2da65311c2ee1`/`0x548a` 写出（虚表 `0x70adba24` 槽 +0x0c）；
/// [-1] COL `0x70afdf00` 的 TD `0x70b36a2c` 名 `.?AVFdbRelationshipClassName@@`。
/// 样本 0 命中。
pub const RELATIONSHIP_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0xe1, 0x2e, 0x1c, 0x31, 0x65, 0xda, 0xd2, 0x11, 0x8a, 0x54, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// MemoryRelationshipClassName 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{6DBA211B-EBDB-11D3-9F84-00C04F6BC886}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll）；GetClassID `0x70a402a0` 立即数
/// 对 `0x11d3ebdb6dba211b`/`0x86c86b4fc000849f`（虚表 `0x70adbe8c` 槽
/// +0x0c）；[-1] COL `0x70afe4d0` 的 TD `0x70b36bd4` 名
/// `.?AVMemoryRelationshipClassName@@`。样本 0 命中。
pub const MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0x1b, 0x21, 0xba, 0x6d, 0xdb, 0xeb, 0xd3, 0x11, 0x9f, 0x84, 0x00, 0xc0, 0x4f, 0x6b, 0xc8, 0x86,
];

/// QueryTableName 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll）；GetClassID `0x709329d0` 立即数
/// 对 `0x48f4c0bb793873de`/`0xbcc1a33ff8f6bfa6`（虚表 `0x70ac5128` 槽
/// +0x0c）；[-1] COL `0x70aeb9d8` 的 TD `0x70b32138` 名 `.?AVQueryTableName@@`。
/// 样本 0 命中。
pub const QUERY_TABLE_NAME_CLASS_ID: [u8; 16] = [
    0xde, 0x73, 0x38, 0x79, 0xbb, 0xc0, 0xf4, 0x48, 0xa6, 0xbf, 0xf6, 0xf8, 0x3f, 0xa3, 0xc1, 0xbc,
];

/// RelQueryTableName 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{DAB3EE10-0F92-455D-8AA2-3D4ADE5B2F7D}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll）；GetClassID `0x70a5a4f0` 立即数
/// 对 `0x455d0f92dab3ee10`/`0x7d2f5bde4a3da28a`（虚表 `0x70add10c` 槽
/// +0x0c）；[-1] COL `0x70affc5c` 的 TD `0x70b3717c` 名 `.?AVRelQueryTableName@@`。
/// 样本 0 命中。
pub const REL_QUERY_TABLE_NAME_CLASS_ID: [u8; 16] = [
    0x10, 0xee, 0xb3, 0xda, 0x92, 0x0f, 0x5d, 0x45, 0x8a, 0xa2, 0x3d, 0x4a, 0xde, 0x5b, 0x2f, 0x7d,
];

/// DBTableName 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式 `{4C9307B2-C125-4497-BA95-AAE875553B06}`。证据：
/// esri.clsid.ecfg 声明（模块 GdbCore.dll）；GetClassID `0x708d7a00` 从
/// .rdata `0x70ab8544`/`0x70ab854c` 以 MOVQ 写出 16 字节（虚表 `0x70ab8480`
/// 槽 +0x0c）；[-1] COL `0x70adec24` 的 TD `0x70b2d2b0` 名 `.?AVDBTableName@@`。
/// 样本 0 命中。
pub const DB_TABLE_NAME_CLASS_ID: [u8; 16] = [
    0xb2, 0x07, 0x93, 0x4c, 0x25, 0xc1, 0x97, 0x44, 0xba, 0x95, 0xaa, 0xe8, 0x75, 0x55, 0x3b, 0x06,
];

/// 请求 IID `{4BA33331-D55F-11D1-8882-0000F877762D}` 的 16 字节内存序
/// （MemoryRelationshipClassName 两槽与 RelQueryTableName 第一槽的
/// LoadObject 请求 IID 为 IName，字节读自 `0x70ab84e4`）。
pub const IID_4BA33331: [u8; 16] = [
    0x31, 0x33, 0xa3, 0x4b, 0x5f, 0xd5, 0xd1, 0x11, 0x88, 0x82, 0x00, 0x00, 0xf8, 0x77, 0x76, 0x2d,
];

/// 请求 IID `{2E060684-E14C-44D3-858A-442AB6A0A863}` 的 16 字节内存序
/// （QueryTableName 第二槽的 LoadObject 请求 IID 为 IQueryDescription，字节读自
/// `0x70ac5158`）。
pub const IID_2E060684: [u8; 16] = [
    0x84, 0x06, 0x06, 0x2e, 0x4c, 0xe1, 0xd3, 0x44, 0x85, 0x8a, 0x44, 0x2a, 0xb6, 0xa0, 0xa8, 0x63,
];

/// 请求 IID `{FDFEBD93-ED75-11D0-9A95-080009EC734B}` 的 16 字节内存序
/// （RelQueryTableName 第二槽的 LoadObject 请求 IID 为 IQueryFilter，字节读自
/// `0x70ac4f28`）。
pub const IID_FDFEBD93: [u8; 16] = [
    0x93, 0xbd, 0xfe, 0xfd, 0x75, 0xed, 0xd0, 0x11, 0x9a, 0x95, 0x08, 0x00, 0x09, 0xec, 0x73, 0x4b,
];

/// 请求 IID `{F7AD47D1-D55E-11D1-8882-0000F877762D}` 的 16 字节内存序
/// （RelQueryTableName 第三槽的 LoadObject 请求 IID 为 ISelectionSet，字节读自
/// `0x70ac4d88`）。
pub const IID_F7AD47D1: [u8; 16] = [
    0xd1, 0x47, 0xad, 0xf7, 0x5e, 0xd5, 0xd1, 0x11, 0x88, 0x82, 0x00, 0x00, 0xf8, 0x77, 0x76, 0x2d,
];

/// 请求 IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}` 的 16 字节内存序
/// （DBTableName 尾槽的 LoadObject 请求 IID 为 ISpatialReference，字节读自
/// `0x70ab8504`）。
pub const IID_4434D680: [u8; 16] = [
    0x80, 0xd6, 0x34, 0x44, 0xda, 0xf4, 0xd1, 0x11, 0xbc, 0xa6, 0x00, 0x00, 0xf8, 0x75, 0xbc, 0xce,
];

/// IDatasetName 槽（请求 IID `{89E75917-…}`）新对象的分派读取结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DatasetNameMoreBody10_1 {
    /// FeatureClassName 自身在 IDatasetName 槽中递归嵌套的对象体。
    FeatureClassName(Box<crate::layers::feature_layer::FeatureClassNamePrefix10_1>),
    /// FeatureDatasetName（第六轮读取器）。
    FeatureDataset(Box<FeatureDatasetNameBody10_1>),
    /// GeometricNetworkName（本模块）。
    GeometricNetwork(Box<GeometricNetworkNameBody10_1>),
    /// RelationshipClassName（本模块）。
    RelationshipClass(Box<RelationshipClassNameBody10_1>),
    /// MemoryRelationshipClassName（本模块）。
    MemoryRelationshipClass(Box<MemoryRelationshipClassNameBody10_1>),
    /// QueryTableName（本模块）。
    QueryTable(Box<QueryTableNameBody10_1>),
    /// RelQueryTableName（本模块）。
    RelQueryTable(Box<RelQueryTableNameBody10_1>),
    /// DBTableName（本模块）。
    DbTable(Box<DbTableNameBody10_1>),
}

/// IDatasetName 槽位的分类结果（本模块口径，含六类分派）。
#[derive(Debug, Clone, PartialEq)]
pub enum DatasetNameMoreSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象且类标识命中七类之一：已读取其体；体内停止时经
    /// [`stop`](DatasetNameMoreSlot10_1::Body) 之外的第二通道（读取器
    /// `stop` 字段）透传。
    Body(DatasetNameMoreBody10_1),
    /// 新对象但类标识无已闭合体（TableName / UnknownDatasetName /
    /// FeatureClassName / 其他模块实现）：停止点，游标回退到该对象头起点、
    /// 该键不登记。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 六类体读取的统一停止原因（体内在首个不可闭合处停止，非错误）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatasetNameMoreStop10_1 {
    /// 嵌套 FeatureClassName 自身的已确认停止点。
    FeatureClassNameStopped {
        class_id: [u8; 16],
        key: u32,
        reason: String,
    },
    /// 槽位新对象无已闭合读取器（IDatasetName 槽的非七类、或请求 IID 语义
    /// 未知的槽出现新对象）：游标已回退到该槽位对象头起点。
    SlotUnsupported { class_id: [u8; 16], key: u32 },
    /// WorkspaceName 体内 IPropertySet 槽新对象（经 IWorkspaceName 槽透传，
    /// 第六轮既有停止点）。
    WorkspacePropertySet { class_id: [u8; 16], key: u32 },
    /// 嵌套对象体自身的停止点（`class_id`/`key` 为该嵌套对象，`inner` 为其
    /// 体内停止原因；游标停在该嵌套读取器的停止点）。
    Nested {
        class_id: [u8; 16],
        key: u32,
        inner: Box<DatasetNameMoreStop10_1>,
    },
}

/// FCN 自身递归读取的安全上限（并非 ArcMap 格式限制）。
pub const MAX_FEATURE_CLASS_NAME_NESTING_DEPTH: usize = 16;

/// 具体对象体未知的槽位（MemoryRelationshipClassName 两槽、QueryTableName
/// 第二槽、RelQueryTableName 三槽、DBTableName 尾槽）的分类结果：新对象一律
/// 不分派（体未知），停止。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownSlot10_1 {
    /// 16 字节全零类标识：空引用。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新对象：体未知且无外部长度，停止点，游标回退到该对象头起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// GeometricNetworkName（GdbCore.dll Load `0x709b8ca0`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct GeometricNetworkNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 int16 版本（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// 第一个字符串（语义未定，候选：名称）。
    pub string_a: String,
    /// 第二个字符串（语义未定）。
    pub string_b: String,
    /// 第三个字符串（语义未定）。
    pub string_c: String,
    /// IDatasetName 槽（obj+0x1c；原生装载后对槽对象做虚表 +0x20 通知，不读流）。
    pub dataset: DatasetNameMoreSlot10_1,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// RelationshipClassName（GdbCore.dll Load `0x70a3b330`）选择子之后槽位的
/// 分类结果。
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipClassTail10_1 {
    /// 选择子 ==0：IWorkspaceName 槽（obj+0x30）。
    Workspace(WorkspaceNameSlot10_1),
    /// 选择子 !=0：IDatasetName 槽（obj+0x2c；装载后虚表 +0x20 通知）。
    Dataset(DatasetNameMoreSlot10_1),
}

/// RelationshipClassName（GdbCore.dll Load `0x70a3b330`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipClassNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 int16 版本（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// 第一个字符串（语义未定，候选：名称）。
    pub string_a: String,
    /// 第二个字符串（语义未定）。
    pub string_b: String,
    /// 第三个字符串（语义未定）。
    pub string_c: String,
    /// 第一个 int32（obj+0x20；语义未定）。
    pub int_a: i32,
    /// 第二个 int32（obj+0x24；语义未定）。
    pub int_b: i32,
    /// 第一个 short（obj+0x28；语义未定）。
    pub short_a: u16,
    /// 第二个 short（obj+0x2a；语义未定）。
    pub short_b: u16,
    /// 流内 int16 选择子（原生只区分零与非零，原样保留）。
    pub selector: u16,
    /// 选择子之后按分支读取的槽位。
    pub tail: RelationshipClassTail10_1,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// MemoryRelationshipClassName（GdbCore.dll Load `0x70a402e0`）体的已确认
/// 字段。
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRelationshipClassNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 int16 版本（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// 前三个字符串（语义未定，候选：名称/类别/描述）。
    pub strings_a: [String; 3],
    /// 第一个 int32（obj+0x10；语义未定）。
    pub int_a: i32,
    /// 第二个 int32（obj+0x14；语义未定）。
    pub int_b: i32,
    /// 第一个 short（obj+0x18；语义未定）。
    pub short_a: u16,
    /// 第二个 short（obj+0x1a；语义未定）。
    pub short_b: u16,
    /// 第一槽（请求 IID `{4BA33331-…}` = IName，obj+0x1c；新对象体未闭合）。
    pub slot_a: UnknownSlot10_1,
    /// 第二槽（同 IID = IName，obj+0x20；新对象体未闭合）。
    pub slot_b: UnknownSlot10_1,
    /// 后四个字符串（语义未定，候选：源/目标名称与类别）。
    pub strings_b: [String; 4],
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// QueryTableName（GdbCore.dll Load `0x70932a10`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct QueryTableNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 int16 版本；原生无符号 `(ver-1) ≤ 1`，仅 1/2 通过（0 拒绝，
    /// 与同族其他类的有符号门不同）。
    pub version: u16,
    /// 第一个字符串（语义未定，候选：名称）。
    pub string_a: String,
    /// 第二个字符串（语义未定）。
    pub string_b: String,
    /// 第三个字符串（语义未定）。
    pub string_c: String,
    /// IWorkspaceName 槽（obj+0x1c；Save 先写本槽，再写 IQueryDescription 槽）。
    pub workspace: WorkspaceNameSlot10_1,
    /// 第二槽（请求 IID `{2E060684-…}` = IQueryDescription，obj+0x18；新对象体未闭合；
    /// 原生装载后做虚表 +0x40 查询，不读流）。
    pub query_slot: UnknownSlot10_1,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// RelQueryTableName（GdbCore.dll Load `0x70a5a530`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RelQueryTableNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 int16 版本（有符号 >3 拒绝；Save 表 (2,3) 恒写 2/3）。
    pub version: u16,
    /// 第一个字符串（语义未定，候选：名称）。
    pub string_a: String,
    /// 第二个字符串（语义未定）。
    pub string_b: String,
    /// 第一槽（请求 IID `{4BA33331-…}` = IName，obj+0x10；新对象体未闭合）。
    pub source_slot: UnknownSlot10_1,
    /// short（obj+0x14；语义未定）。
    pub short_a: u16,
    /// 第二槽（请求 IID `{FDFEBD93-…}` = IQueryFilter，obj+0x18；新对象体未闭合）。
    pub left_slot: UnknownSlot10_1,
    /// 第三槽（请求 IID `{F7AD47D1-…}` = ISelectionSet，obj+0x1c；新对象体未闭合）。
    pub right_slot: UnknownSlot10_1,
    /// 第三个字符串（语义未定）。
    pub string_c: String,
    /// short（obj+0x24；语义未定）。
    pub short_b: u16,
    /// 版本 ≥2 的 short（obj+0x26；短体原生置内存 0，流内无字段 → `None`）。
    pub short_c: Option<u16>,
    /// 版本 ≥3 的 short（obj+0x28；短体流内无字段 → `None`）。
    pub short_d: Option<u16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// DBTableName（GdbCore.dll Load `0x708d7a40`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct DbTableNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 前三个字符串（语义未定，候选：目录/名称/类别；**本类无版本字段**）。
    pub strings_a: [String; 3],
    /// IWorkspaceName 槽（obj+0x1c）。
    pub workspace: WorkspaceNameSlot10_1,
    /// 第四个字符串（语义未定）。
    pub string_d: String,
    /// int32（obj+0x14；原生 ≠7 时置内存标志 obj+0x19，语义未定，原样保留）。
    pub int_a: i32,
    /// 尾槽（请求 IID `{4434D680-…}` = ISpatialReference，obj+0x20；新对象体未闭合）。
    pub tail_slot: UnknownSlot10_1,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// 把 IDatasetName 槽的新对象按类标识分派到已闭合读取器（FeatureDatasetName
/// + 本模块六类；含相互递归）。失败向上传播；嵌套体停止透传为
/// [`DatasetNameMoreStop10_1::Nested`]。
fn read_dataset_name_object_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<(DatasetNameMoreBody10_1, Option<DatasetNameMoreStop10_1>)> {
    macro_rules! nested_stop {
        ($body:expr) => {
            $body
                .stop
                .take()
                .map(|inner| DatasetNameMoreStop10_1::Nested {
                    class_id,
                    key,
                    inner: Box::new(inner),
                })
        };
    }
    match class_id {
        FEATURE_DATASET_NAME_CLASS_ID => {
            let body = read_feature_dataset_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = body.stop.map(|inner| match inner {
                FeatureDatasetNameStop10_1::WorkspacePropertySetNewObject { class_id, key } => {
                    DatasetNameMoreStop10_1::WorkspacePropertySet { class_id, key }
                }
                FeatureDatasetNameStop10_1::WorkspaceUnsupported { class_id, key } => {
                    DatasetNameMoreStop10_1::SlotUnsupported { class_id, key }
                }
            });
            Ok((
                DatasetNameMoreBody10_1::FeatureDataset(Box::new(body)),
                stop,
            ))
        }
        GEOMETRIC_NETWORK_NAME_CLASS_ID => {
            let mut body = read_geometric_network_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((
                DatasetNameMoreBody10_1::GeometricNetwork(Box::new(body)),
                stop,
            ))
        }
        RELATIONSHIP_CLASS_NAME_CLASS_ID => {
            let mut body = read_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((
                DatasetNameMoreBody10_1::RelationshipClass(Box::new(body)),
                stop,
            ))
        }
        MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID => {
            let mut body =
                read_memory_relationship_class_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((
                DatasetNameMoreBody10_1::MemoryRelationshipClass(Box::new(body)),
                stop,
            ))
        }
        QUERY_TABLE_NAME_CLASS_ID => {
            let mut body = read_query_table_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::QueryTable(Box::new(body)), stop))
        }
        REL_QUERY_TABLE_NAME_CLASS_ID => {
            let mut body = read_rel_query_table_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::RelQueryTable(Box::new(body)), stop))
        }
        DB_TABLE_NAME_CLASS_ID => {
            let mut body = read_db_table_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::DbTable(Box::new(body)), stop))
        }
        _ => unreachable!("调用方先按类标识判定，此处必为七类之一"),
    }
}

/// 读取并分类一个 IDatasetName 槽（请求 IID `{89E75917-…}`）：空引用/回引
/// 原样返回；新对象命中七类时分派（嵌套体停止透传为 `Nested`）；其余类为
/// 停止点并回退到槽位起点。`stop_out` 带出停止原因（无停止时为 `None`）。
///
/// B 轮接线起为本模块与 `dataset_name` / `layers::feature_layer`（FCN ⑦
/// 选择子 !=0 槽）共用的统一分派入口（七类含 FeatureDatasetName，与第六轮
/// 读取器的 FeatureDatasetName 臂互斥不复读）。
pub fn read_dataset_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<DatasetNameMoreSlot10_1> {
    read_dataset_name_slot_with_depth_10_1(reader, max_bytes, table, stop_out, 0)
}

/// 带 FCN 递归深度预算的 IDatasetName 槽分派器。
pub(crate) fn read_dataset_name_slot_with_depth_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
    depth: usize,
) -> io::Result<DatasetNameMoreSlot10_1> {
    // 整体事务：入口克隆 trial 游标与暂存登记表，成功返回（含停止）才提交；
    // 任何失败（截断、嵌套体错误）与未知类回退路径都不推进调用者游标、
    // 不修改登记表。
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let result = (|| -> io::Result<DatasetNameMoreSlot10_1> {
        let saved_table = staged.clone();
        match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::Null => Ok(DatasetNameMoreSlot10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                Ok(DatasetNameMoreSlot10_1::BackReference { class_id, key })
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                let known = class_id == crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID
                    || class_id == FEATURE_DATASET_NAME_CLASS_ID
                    || class_id == GEOMETRIC_NETWORK_NAME_CLASS_ID
                    || class_id == RELATIONSHIP_CLASS_NAME_CLASS_ID
                    || class_id == MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID
                    || class_id == QUERY_TABLE_NAME_CLASS_ID
                    || class_id == REL_QUERY_TABLE_NAME_CLASS_ID
                    || class_id == DB_TABLE_NAME_CLASS_ID;
                if !known {
                    staged = saved_table;
                    trial = reader.clone();
                    *stop_out = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
                    return Ok(DatasetNameMoreSlot10_1::Unsupported { class_id, key });
                }
                if class_id == crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID {
                    if depth >= MAX_FEATURE_CLASS_NAME_NESTING_DEPTH {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "FeatureClassName 递归深度超过读取器限额 {MAX_FEATURE_CLASS_NAME_NESTING_DEPTH}（防栈溢出，非原生限制）"
                            ),
                        ));
                    }
                    let body =
                        crate::layers::feature_layer::read_feature_class_name_body_with_depth_10_1(
                            &mut trial,
                            max_bytes,
                            &mut staged,
                            key,
                            depth + 1,
                        )?;
                    *stop_out = body.stop.as_ref().map(|reason| {
                        DatasetNameMoreStop10_1::FeatureClassNameStopped {
                            class_id,
                            key,
                            reason: format!("{reason:?}"),
                        }
                    });
                    return Ok(DatasetNameMoreSlot10_1::Body(
                        DatasetNameMoreBody10_1::FeatureClassName(Box::new(body)),
                    ));
                }
                let (body, stop) = read_dataset_name_object_10_1(
                    &mut trial,
                    max_bytes,
                    &mut staged,
                    class_id,
                    key,
                )?;
                *stop_out = stop;
                Ok(DatasetNameMoreSlot10_1::Body(body))
            }
        }
    })()?;
    *reader = trial;
    *table = staged;
    Ok(result)
}

/// 读取并分类一个 IWorkspaceName 槽（请求 IID `{FADD975C-…}`）：新对象为
/// WorkspaceName 类时读取其闭合体（第六轮），体内 PropertySet 新对象透传为
/// [`DatasetNameMoreStop10_1::WorkspacePropertySet`]；其余类停止并回退。
fn read_workspace_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<WorkspaceNameSlot10_1> {
    let at_slot = reader.clone();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    match read_object_entry_10_1(reader, &mut staged_table)? {
        ObjectEntry10_1::Null => Ok(WorkspaceNameSlot10_1::Null),
        ObjectEntry10_1::BackReference { class_id, key } => {
            *table = staged_table;
            Ok(WorkspaceNameSlot10_1::BackReference { class_id, key })
        }
        ObjectEntry10_1::NewObject { class_id, key } if class_id == WORKSPACE_NAME_CLASS_ID => {
            let body = read_workspace_name_body_10_1(reader, max_bytes, &mut staged_table, key)?;
            if let Some(WorkspaceNameStop10_1::PropertySetNewObject { class_id, key }) = body.stop {
                *stop_out = Some(DatasetNameMoreStop10_1::WorkspacePropertySet { class_id, key });
            }
            *table = staged_table;
            Ok(WorkspaceNameSlot10_1::WorkspaceName(Box::new(body)))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *table = saved_table;
            *reader = at_slot;
            *stop_out = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
            Ok(WorkspaceNameSlot10_1::Unsupported { class_id, key })
        }
    }
}

/// 读取并分类一个具体对象体未知的槽：空引用/回引原样返回；新对象一律
/// 停止并回退到槽位起点。
fn read_unknown_slot_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<UnknownSlot10_1> {
    let at_slot = reader.clone();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    match read_object_entry_10_1(reader, &mut staged_table)? {
        ObjectEntry10_1::Null => Ok(UnknownSlot10_1::Null),
        ObjectEntry10_1::BackReference { class_id, key } => {
            *table = staged_table;
            Ok(UnknownSlot10_1::BackReference { class_id, key })
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *table = saved_table;
            *reader = at_slot;
            *stop_out = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
            Ok(UnknownSlot10_1::Unsupported { class_id, key })
        }
    }
}

/// 版本门（有符号比较）：`(version as i16) > max || 负值` 拒绝。
fn check_signed_version(version: u16, max: i16, context: &str, offset: usize) -> io::Result<()> {
    if (version as i16) > max || (version as i16) < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "{context} 对象体 offset 0x{offset:x}: 架构版本 {version} 超出 Load 接受范围（>{max} 拒绝；负值位模式一并拒绝）"
            ),
        ));
    }
    Ok(())
}

/// 从对象头之后的位置读取 10.1 GeometricNetworkName 对象体，失败时不推进
/// 游标、不修改登记表。
///
/// 起点约定与限额口径见模块文档。IDatasetName 槽的新对象命中七类时分派；
/// 其余类为停止点（游标回退到槽位起点）。
pub fn read_geometric_network_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<GeometricNetworkNameBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    let version = trial.read_u16_le()?;
    check_signed_version(version, 1, "GeometricNetworkName", entry_offset)?;

    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;
    let string_c = read_string_10_1(&mut trial, max_bytes)?;

    let mut staged_table = table.clone();
    let mut stop = None;
    let dataset = read_dataset_name_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;

    let result = GeometricNetworkNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        string_c,
        dataset,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 RelationshipClassName 对象体，失败时不推进
/// 游标、不修改登记表。选择子 ==0 走 IWorkspaceName 槽、!=0 走 IDatasetName
/// 槽（原生对 !=0 槽装载成功后做虚表 +0x20 通知，不读流）。
pub fn read_relationship_class_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RelationshipClassNameBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    let version = trial.read_u16_le()?;
    check_signed_version(version, 1, "RelationshipClassName", entry_offset)?;

    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;
    let string_c = read_string_10_1(&mut trial, max_bytes)?;

    let int_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().expect("4 字节"));
    let int_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().expect("4 字节"));
    let short_a = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));
    let short_b = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));

    // int16 选择子：原生检查读出长度为 2 后按零/非零分支。
    let selector = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));

    let mut staged_table = table.clone();
    let mut stop = None;
    let tail = if selector == 0 {
        let workspace =
            read_workspace_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;
        RelationshipClassTail10_1::Workspace(workspace)
    } else {
        let dataset =
            read_dataset_name_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;
        RelationshipClassTail10_1::Dataset(dataset)
    };

    let result = RelationshipClassNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        string_c,
        int_a,
        int_b,
        short_a,
        short_b,
        selector,
        tail,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 MemoryRelationshipClassName 对象体，失败时
/// 不推进游标、不修改登记表。两个 LoadObject 槽的请求 IID
/// `{4BA33331-…}` 为 IName，新对象体未知时停止。
pub fn read_memory_relationship_class_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<MemoryRelationshipClassNameBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    let version = trial.read_u16_le()?;
    check_signed_version(version, 1, "MemoryRelationshipClassName", entry_offset)?;

    let strings_a = [
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
    ];

    let int_a = i32::from_le_bytes(trial.read_bytes(4)?.try_into().expect("4 字节"));
    let int_b = i32::from_le_bytes(trial.read_bytes(4)?.try_into().expect("4 字节"));
    let short_a = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));
    let short_b = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));

    let mut staged_table = table.clone();
    let mut stop = None;
    let slot_a = read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?;
    // 槽位停止后其体不可达（未知对象无外部长度），后续字段按占位填充。
    let (slot_b, strings_b) = if stop.is_some() {
        (
            UnknownSlot10_1::Null,
            [String::new(), String::new(), String::new(), String::new()],
        )
    } else {
        let slot_b = read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?;
        let strings_b = if stop.is_some() {
            [String::new(), String::new(), String::new(), String::new()]
        } else {
            [
                read_string_10_1(&mut trial, max_bytes)?,
                read_string_10_1(&mut trial, max_bytes)?,
                read_string_10_1(&mut trial, max_bytes)?,
                read_string_10_1(&mut trial, max_bytes)?,
            ]
        };
        (slot_b, strings_b)
    };

    let result = MemoryRelationshipClassNameBody10_1 {
        key,
        version,
        strings_a,
        int_a,
        int_b,
        short_a,
        short_b,
        slot_a,
        slot_b,
        strings_b,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 QueryTableName 对象体，失败时不推进游标、
/// 不修改登记表。版本门为原生无符号 `(ver-1) ≤ 1`：仅 1/2 通过，0 被原生
/// 拒绝（读取器同拒），负值位模式按无符号解释可能通过大值分支、此处一并
/// 拒绝（较原生更严，与仓库口径一致）。
pub fn read_query_table_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<QueryTableNameBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    let version = trial.read_u16_le()?;
    // 原生汇编：`mov ecx,[ver]; dec ecx; cmp cx,1; ja fail`（无符号）。
    let minus_one = version.wrapping_sub(1);
    if minus_one > 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "QueryTableName 对象体 offset 0x{entry_offset:x}: 架构版本 {version} 超出 Load 接受范围（原生无符号 (ver-1)≤1，仅 1/2 通过）"
            ),
        ));
    }

    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;
    let string_c = read_string_10_1(&mut trial, max_bytes)?;

    let mut staged_table = table.clone();
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;
    let query_slot = if stop.is_none() {
        read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?
    } else {
        // 工作空间槽已停止：其后的第二槽不可达，体到此为止。
        UnknownSlot10_1::Null
    };

    let result = QueryTableNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        string_c,
        workspace,
        query_slot,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 RelQueryTableName 对象体，失败时不推进游标、
/// 不修改登记表。三个槽请求 IName、IQueryFilter、ISelectionSet，新对象体
/// 未闭合时停止。
pub fn read_rel_query_table_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<RelQueryTableNameBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    let version = trial.read_u16_le()?;
    check_signed_version(version, 3, "RelQueryTableName", entry_offset)?;

    let string_a = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;

    let mut staged_table = table.clone();
    let mut stop = None;
    let source_slot = read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?;
    // 槽位停止后其体不可达（未知对象无外部长度），后续字段按占位填充。
    let (short_a, left_slot, right_slot, string_c, short_b, short_c, short_d) = if stop.is_some() {
        (
            0,
            UnknownSlot10_1::Null,
            UnknownSlot10_1::Null,
            String::new(),
            0,
            None,
            None,
        )
    } else {
        let short_a = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));
        let left_slot = read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?;
        let right_slot = if stop.is_some() {
            UnknownSlot10_1::Null
        } else {
            read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?
        };
        let string_c = if stop.is_some() {
            String::new()
        } else {
            read_string_10_1(&mut trial, max_bytes)?
        };
        let short_b = if stop.is_some() {
            0
        } else {
            u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"))
        };
        // 版本 ≥2：short（obj+0x26）；短体原生置内存 0（流内无字段）。
        // 槽位停止后流不可达，均为 None（占位）。
        let short_c = if version >= 2 && stop.is_none() {
            Some(u16::from_le_bytes(
                trial.read_bytes(2)?.try_into().expect("2 字节"),
            ))
        } else {
            None
        };
        // 版本 ≥3：short（obj+0x28）。
        let short_d = if version >= 3 && stop.is_none() {
            Some(u16::from_le_bytes(
                trial.read_bytes(2)?.try_into().expect("2 字节"),
            ))
        } else {
            None
        };
        (
            short_a, left_slot, right_slot, string_c, short_b, short_c, short_d,
        )
    };

    let result = RelQueryTableNameBody10_1 {
        key,
        version,
        string_a,
        string_b,
        source_slot,
        short_a,
        left_slot,
        right_slot,
        string_c,
        short_b,
        short_c,
        short_d,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

/// 从对象头之后的位置读取 10.1 DBTableName 对象体，失败时不推进游标、不
/// 修改登记表。**本类体无版本字段**（Load/Save 反编译逐字段对称证实）。
pub fn read_db_table_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<DbTableNameBody10_1> {
    let mut trial = reader.clone();

    let strings_a = [
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
    ];

    let mut staged_table = table.clone();
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;
    let string_d = if stop.is_none() {
        read_string_10_1(&mut trial, max_bytes)?
    } else {
        String::new()
    };
    let int_a = if stop.is_none() {
        i32::from_le_bytes(trial.read_bytes(4)?.try_into().expect("4 字节"))
    } else {
        0
    };
    let tail_slot = if stop.is_none() {
        read_unknown_slot_10_1(&mut trial, &mut staged_table, &mut stop)?
    } else {
        UnknownSlot10_1::Null
    };

    let result = DbTableNameBody10_1 {
        key,
        strings_a,
        workspace,
        string_d,
        int_a,
        tail_slot,
        stop,
    };
    *table = staged_table;
    *reader = trial;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AfCore String::Write 编码：u32 字节长度（含 2 字节终止零）+ UTF-16LE。
    fn str_bytes(s: &str) -> Vec<u8> {
        let mut units: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
        units.extend_from_slice(&[0, 0]);
        let mut out = (units.len() as u32).to_le_bytes().to_vec();
        out.extend_from_slice(&units);
        out
    }

    /// 对象条目：16 字节类标识 + u32 键。
    fn entry(class: &[u8; 16], key: u32) -> Vec<u8> {
        let mut v = class.to_vec();
        v.extend_from_slice(&key.to_le_bytes());
        v
    }

    /// 完整 WorkspaceName 体（第六轮布局）：版本 1 + 三空串 + IPropertySet
    /// 空标记 + 标志 1 + 工厂 CLSID + 尾随 int32。
    fn workspace_body() -> Vec<u8> {
        let mut v = 1u16.to_le_bytes().to_vec();
        v.extend(str_bytes(""));
        v.extend(str_bytes(""));
        v.extend(str_bytes(""));
        v.extend_from_slice(&[0u8; 16]);
        v.push(1);
        v.extend_from_slice(&[0xa5; 16]);
        v.extend_from_slice(&0i32.to_le_bytes());
        v
    }

    /// 完整 RelationshipClassName 体（选择子分支由调用者给尾段）。
    fn rcn_body(selector: u16, tail: &[u8]) -> Vec<u8> {
        let mut v = 1u16.to_le_bytes().to_vec();
        v.extend(str_bytes("源类"));
        v.extend(str_bytes("目标类"));
        v.extend(str_bytes(""));
        v.extend_from_slice(&7i32.to_le_bytes());
        v.extend_from_slice(&(-9i32).to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&2u16.to_le_bytes());
        v.extend_from_slice(&selector.to_le_bytes());
        v.extend_from_slice(tail);
        v
    }

    /// 完整 GeometricNetworkName 体（IDatasetName 槽尾段由调用者给）。
    fn fgn_body(tail: &[u8]) -> Vec<u8> {
        let mut v = 1u16.to_le_bytes().to_vec();
        v.extend(str_bytes("管网"));
        v.extend(str_bytes(""));
        v.extend(str_bytes("几何网络"));
        v.extend_from_slice(tail);
        v
    }

    fn fcn_body(nested: &[u8]) -> Vec<u8> {
        let mut v = 1u16.to_le_bytes().to_vec();
        for _ in 0..4 {
            v.extend(str_bytes(""));
        }
        v.extend_from_slice(&0i32.to_le_bytes());
        v.extend_from_slice(&0i32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(nested);
        v
    }

    #[test]
    fn fcn_dataset_name_slot_recursively_dispatches_feature_class_name() {
        let inner_entry = entry(
            &crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID,
            42,
        );
        let mut bytes = entry(
            &crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID,
            41,
        );
        bytes.extend(fcn_body(&inner_entry));
        bytes.extend(fcn_body(&[0u8; 16]));

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let mut stop = None;
        let result = read_dataset_name_slot_10_1(&mut reader, 64, &mut table, &mut stop).unwrap();
        let DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::FeatureClassName(outer)) =
            result
        else {
            panic!("顶层 FCN 应完成类分派");
        };
        assert_eq!(outer.key, 41);
        assert!(matches!(
            outer.nested,
            crate::layers::feature_layer::FeatureClassNameNested10_1::DatasetName(
                crate::layers::feature_layer::DatasetNameEntry10_1::NewObject(inner)
            ) if matches!(*inner, DatasetNameMoreBody10_1::FeatureClassName(_))
        ));
        assert!(table.contains(41));
        assert!(table.contains(42));
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(stop, None);
    }

    #[test]
    fn fcn_recursion_depth_limit_fails_transactionally() {
        let bytes = entry(
            &crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID,
            51,
        );
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let mut stop = None;
        let err = read_dataset_name_slot_with_depth_10_1(
            &mut reader,
            64,
            &mut table,
            &mut stop,
            MAX_FEATURE_CLASS_NAME_NESTING_DEPTH,
        )
        .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert!(err.to_string().contains("递归深度超过读取器限额"));
        assert_eq!(reader.position(), 0);
        assert!(!table.contains(51));
        assert_eq!(stop, None);
    }

    #[test]
    fn fgn_reads_full_body_and_gates_version_signed() {
        let mut table = RefTable10_1::new();
        table.register(3);
        let bytes = fgn_body(&[0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let body = read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 3).unwrap();
        assert_eq!(body.key, 3);
        assert_eq!(body.version, 1);
        assert_eq!(body.string_a, "管网");
        assert_eq!(body.string_c, "几何网络");
        assert_eq!(body.dataset, DatasetNameMoreSlot10_1::Null);
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());

        // 版本 0：有符号门内通过。
        let mut zero = 0u16.to_le_bytes().to_vec();
        zero.extend(str_bytes(""));
        zero.extend(str_bytes(""));
        zero.extend(str_bytes(""));
        zero.extend_from_slice(&[0u8; 16]);
        let mut reader = Reader::new(&zero);
        let mut table = RefTable10_1::new();
        table.register(1);
        let body = read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 1).unwrap();
        assert_eq!(body.version, 0);

        // 版本 2 与负值位模式 0xFFFF：拒绝且不推进游标、不动登记表。
        for ver in [2u16, 0xFFFF] {
            let mut bad = ver.to_le_bytes().to_vec();
            bad.extend(str_bytes(""));
            let mut reader = Reader::new(&bad);
            let mut table = RefTable10_1::new();
            table.register(1);
            assert!(read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 1).is_err());
            assert_eq!(reader.position(), 0);
            assert!(table.contains(1));
            assert_eq!(table.len(), 1);
        }
    }

    #[test]
    fn rcn_selector_chooses_slot_branch_and_keeps_scalars() {
        // 选择子 !=0：IDatasetName 槽（此处空引用）。
        let mut table = RefTable10_1::new();
        table.register(11);
        let bytes = rcn_body(1, &[0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let body = read_relationship_class_name_body_10_1(&mut reader, 64, &mut table, 11).unwrap();
        assert_eq!(body.selector, 1);
        assert!(matches!(
            body.tail,
            RelationshipClassTail10_1::Dataset(DatasetNameMoreSlot10_1::Null)
        ));
        assert_eq!(body.int_a, 7);
        assert_eq!(body.int_b, -9);
        assert_eq!(body.short_b, 2);
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());

        // 选择子 ==0：IWorkspaceName 槽。
        let bytes = rcn_body(0, &[0u8; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(12);
        let body = read_relationship_class_name_body_10_1(&mut reader, 64, &mut table, 12).unwrap();
        assert_eq!(body.selector, 0);
        assert!(matches!(
            body.tail,
            RelationshipClassTail10_1::Workspace(WorkspaceNameSlot10_1::Null)
        ));
    }

    #[test]
    fn rcn_workspace_slot_reads_workspace_body() {
        let mut table = RefTable10_1::new();
        table.register(21);
        let tail = entry(&WORKSPACE_NAME_CLASS_ID, 22);
        let mut bytes = rcn_body(0, &tail);
        bytes.extend(workspace_body());
        let mut reader = Reader::new(&bytes);
        let body = read_relationship_class_name_body_10_1(&mut reader, 64, &mut table, 21).unwrap();
        assert_eq!(body.stop, None);
        assert!(table.contains(22));
        match body.tail {
            RelationshipClassTail10_1::Workspace(WorkspaceNameSlot10_1::WorkspaceName(ws)) => {
                assert_eq!(ws.key, 22);
                assert_eq!(ws.factory_class_id, Some([0xa5; 16]));
                assert_eq!(ws.trailing_int32, Some(0));
            }
            other => panic!("期望 WorkspaceName 槽，实际 {other:?}"),
        }
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn idataset_slot_dispatches_nested_bodies() {
        // FGN 的 IDatasetName 槽放 RCN 新对象：嵌套读取，键均登记。
        let mut table = RefTable10_1::new();
        table.register(31);
        let tail = entry(&RELATIONSHIP_CLASS_NAME_CLASS_ID, 32);
        let mut bytes = fgn_body(&tail);
        bytes.extend(rcn_body(1, &[0u8; 16]));
        let mut reader = Reader::new(&bytes);
        let body = read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 31).unwrap();
        assert!(table.contains(32));
        match body.dataset {
            DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RelationshipClass(rc)) => {
                assert_eq!(rc.key, 32);
                assert_eq!(rc.selector, 1);
            }
            other => panic!("期望嵌套 RelationshipClass，实际 {other:?}"),
        }
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn nested_stop_propagates_as_nested_variant() {
        // 嵌套停止透传：FGN → RCN → 未知类槽 → Nested{SlotUnsupported}。
        let tail = entry(&RELATIONSHIP_CLASS_NAME_CLASS_ID, 34);
        let mut bytes = fgn_body(&tail);
        bytes.extend(rcn_body(1, &entry(&[0x77; 16], 35)));
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(33);
        let body = read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 33).unwrap();
        // 登记先于体解析:嵌套体的键已登记不回滚(与 dataset_name 读取器口径
        // 一致);未知槽条目的新键(35)不登记。
        assert!(table.contains(34));
        assert!(!table.contains(35));
        match body.stop {
            Some(DatasetNameMoreStop10_1::Nested {
                class_id,
                key,
                inner,
            }) => {
                assert_eq!(class_id, RELATIONSHIP_CLASS_NAME_CLASS_ID);
                assert_eq!(key, 34);
                assert_eq!(
                    *inner,
                    DatasetNameMoreStop10_1::SlotUnsupported {
                        class_id: [0x77; 16],
                        key: 35
                    }
                );
            }
            other => panic!("期望 Nested 停止，实际 {other:?}"),
        }
        // 游标停在未知对象头起点（= 全部已消费前缀之后、未知条目之前）。
        assert_eq!(reader.position(), bytes.len() - 20);
    }

    #[test]
    fn idataset_slot_unknown_class_rolls_back() {
        let mut table = RefTable10_1::new();
        table.register(41);
        let tail = entry(&[0x99; 16], 42);
        let bytes = fgn_body(&tail);
        let mut reader = Reader::new(&bytes);
        let body = read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 41).unwrap();
        assert_eq!(
            body.dataset,
            DatasetNameMoreSlot10_1::Unsupported {
                class_id: [0x99; 16],
                key: 42
            }
        );
        assert_eq!(
            body.stop,
            Some(DatasetNameMoreStop10_1::SlotUnsupported {
                class_id: [0x99; 16],
                key: 42
            })
        );
        assert!(!table.contains(42));
        assert_eq!(reader.position(), bytes.len() - tail.len());
    }

    #[test]
    fn slot_back_reference_with_pre_registered_key() {
        let tail = entry(&GEOMETRIC_NETWORK_NAME_CLASS_ID, 51);
        let bytes = fgn_body(&tail);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(52);
        table.register(51);
        let body = read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, 52).unwrap();
        assert_eq!(
            body.dataset,
            DatasetNameMoreSlot10_1::BackReference {
                class_id: GEOMETRIC_NETWORK_NAME_CLASS_ID,
                key: 51
            }
        );
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn memrcn_reads_scalars_and_stops_at_unknown_slot() {
        let mut table = RefTable10_1::new();
        table.register(61);
        let mut bytes = 1u16.to_le_bytes().to_vec();
        bytes.extend(str_bytes("内存关系"));
        bytes.extend(str_bytes(""));
        bytes.extend(str_bytes("类"));
        bytes.extend_from_slice(&1i32.to_le_bytes());
        bytes.extend_from_slice(&2i32.to_le_bytes());
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend_from_slice(&4u16.to_le_bytes());
        let unknown = entry(&[0x66; 16], 62);
        bytes.extend_from_slice(&unknown);
        bytes.extend(str_bytes("不应被消费"));

        let mut reader = Reader::new(&bytes);
        let body =
            read_memory_relationship_class_name_body_10_1(&mut reader, 64, &mut table, 61).unwrap();
        assert_eq!(body.strings_a[0], "内存关系");
        assert_eq!(body.short_b, 4);
        assert_eq!(
            body.slot_a,
            UnknownSlot10_1::Unsupported {
                class_id: [0x66; 16],
                key: 62
            }
        );
        // 停止后 slot_b/strings_b 为占位。
        assert_eq!(body.slot_b, UnknownSlot10_1::Null);
        assert!(body.strings_b.iter().all(|s| s.is_empty()));
        assert!(!table.contains(62));
        let consumed_before_slot =
            2 + str_bytes("内存关系").len() + str_bytes("").len() + str_bytes("类").len() + 12;
        assert_eq!(reader.position(), consumed_before_slot);
    }

    #[test]
    fn querytn_version_gate_is_unsigned_range() {
        let base = |ver: u16| {
            let mut v = ver.to_le_bytes().to_vec();
            v.extend(str_bytes("查询表"));
            v.extend(str_bytes(""));
            v.extend(str_bytes(""));
            v.extend_from_slice(&[0u8; 16]); // workspace 槽空引用
            v.extend_from_slice(&[0u8; 16]); // query 槽空引用
            v
        };
        for ver in [1u16, 2] {
            let bytes = base(ver);
            let mut reader = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            table.register(71);
            let body = read_query_table_name_body_10_1(&mut reader, 64, &mut table, 71).unwrap();
            assert_eq!(body.version, ver);
            assert_eq!(reader.position(), bytes.len());
        }
        for ver in [0u16, 3, 0xFFFF] {
            let bytes = base(ver);
            let mut reader = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            table.register(71);
            assert!(read_query_table_name_body_10_1(&mut reader, 64, &mut table, 71).is_err());
            assert_eq!(reader.position(), 0);
        }
    }

    #[test]
    fn relquerytn_version_branches() {
        // 版本 1：短体（无 0x26/0x28 short），其后缀字节不被消费。
        let mut v1 = 1u16.to_le_bytes().to_vec();
        v1.extend(str_bytes("rel"));
        v1.extend(str_bytes(""));
        v1.extend_from_slice(&[0u8; 16]);
        v1.extend_from_slice(&5u16.to_le_bytes());
        v1.extend_from_slice(&[0u8; 16]);
        v1.extend_from_slice(&[0u8; 16]);
        v1.extend(str_bytes("过滤"));
        v1.extend_from_slice(&6u16.to_le_bytes());
        let mut bytes = v1.clone();
        bytes.extend_from_slice(&[0xAA, 0xBB]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(81);
        let body = read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 81).unwrap();
        assert_eq!(body.short_a, 5);
        assert_eq!(body.short_c, None);
        assert_eq!(body.short_d, None);
        assert_eq!(body.string_c, "过滤");
        assert_eq!(reader.position(), v1.len());

        // 版本 2：读 0x26，不读 0x28。
        let mut v2 = 2u16.to_le_bytes().to_vec();
        v2.extend(str_bytes(""));
        v2.extend(str_bytes(""));
        v2.extend_from_slice(&[0u8; 16]);
        v2.extend_from_slice(&0u16.to_le_bytes());
        v2.extend_from_slice(&[0u8; 16]);
        v2.extend_from_slice(&[0u8; 16]);
        v2.extend(str_bytes(""));
        v2.extend_from_slice(&0u16.to_le_bytes());
        v2.extend_from_slice(&7u16.to_le_bytes());
        let mut reader = Reader::new(&v2);
        let mut table = RefTable10_1::new();
        table.register(82);
        let body = read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 82).unwrap();
        assert_eq!(body.short_c, Some(7));
        assert_eq!(body.short_d, None);
        assert_eq!(reader.position(), v2.len());

        // 版本 3：两个尾部 short 全读。
        let mut v3 = 3u16.to_le_bytes().to_vec();
        v3.extend(str_bytes(""));
        v3.extend(str_bytes(""));
        v3.extend_from_slice(&[0u8; 16]);
        v3.extend_from_slice(&0u16.to_le_bytes());
        v3.extend_from_slice(&[0u8; 16]);
        v3.extend_from_slice(&[0u8; 16]);
        v3.extend(str_bytes(""));
        v3.extend_from_slice(&0u16.to_le_bytes());
        v3.extend_from_slice(&7u16.to_le_bytes());
        v3.extend_from_slice(&8u16.to_le_bytes());
        let mut reader = Reader::new(&v3);
        let mut table = RefTable10_1::new();
        table.register(84);
        let body = read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 84).unwrap();
        assert_eq!(body.short_c, Some(7));
        assert_eq!(body.short_d, Some(8));
        assert_eq!(reader.position(), v3.len());

        // 版本 4：拒绝。
        let mut v4 = 4u16.to_le_bytes().to_vec();
        v4.extend(str_bytes(""));
        v4.extend(str_bytes(""));
        let mut reader = Reader::new(&v4);
        let mut table = RefTable10_1::new();
        table.register(83);
        assert!(read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 83).is_err());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn dbtn_has_no_version_field() {
        // 体首即字符串长度：前 4 字节按 String 解析而非版本。
        let mut bytes = str_bytes("目录");
        bytes.extend(str_bytes("表名"));
        bytes.extend(str_bytes("类别"));
        bytes.extend_from_slice(&entry(&WORKSPACE_NAME_CLASS_ID, 91));
        bytes.extend(workspace_body());
        bytes.extend(str_bytes("第四串"));
        bytes.extend_from_slice(&7i32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 16]); // 尾槽空引用

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(90);
        let body = read_db_table_name_body_10_1(&mut reader, 64, &mut table, 90).unwrap();
        assert_eq!(body.strings_a[0], "目录");
        assert_eq!(body.strings_a[1], "表名");
        assert_eq!(body.int_a, 7);
        assert!(table.contains(91));
        match body.workspace {
            WorkspaceNameSlot10_1::WorkspaceName(ws) => assert_eq!(ws.key, 91),
            other => panic!("期望 WorkspaceName 槽，实际 {other:?}"),
        }
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn workspace_stop_blocks_following_fields() {
        // DBTN 的工作空间槽放未知类新对象：体在其处停止，后续字段占位。
        let mut bytes = str_bytes("");
        bytes.extend(str_bytes(""));
        bytes.extend(str_bytes(""));
        let unknown = entry(&[0x55; 16], 92);
        bytes.extend_from_slice(&unknown);
        bytes.extend(str_bytes("不可达"));

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(93);
        let body = read_db_table_name_body_10_1(&mut reader, 64, &mut table, 93).unwrap();
        assert_eq!(
            body.stop,
            Some(DatasetNameMoreStop10_1::SlotUnsupported {
                class_id: [0x55; 16],
                key: 92
            })
        );
        assert_eq!(
            body.workspace,
            WorkspaceNameSlot10_1::Unsupported {
                class_id: [0x55; 16],
                key: 92
            }
        );
        assert_eq!(body.string_d, "");
        assert_eq!(body.tail_slot, UnknownSlot10_1::Null);
        assert!(!table.contains(92));
    }

    #[test]
    fn max_bytes_limit_applies_to_strings() {
        let mut bytes = 1u16.to_le_bytes().to_vec();
        bytes.extend(str_bytes("一段超过限额的字符串"));
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        table.register(95);
        // 5 个 UTF-16 单元 10 字节 + 终止零 2 字节 = 12；限额 8 拒绝。
        assert!(read_geometric_network_name_body_10_1(&mut reader, 8, &mut table, 95).is_err());
        assert_eq!(reader.position(), 0);
        assert!(table.contains(95));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn truncated_bodies_are_transactional_across_all_readers() {
        // 每个读取器对合法满体的全部前缀截断：错误、游标 0、登记表不变。
        // dbtn 用例的嵌套 WorkspaceName 体读取成功会登记键 999，故放宽到 2。
        let cases: Vec<(String, Vec<u8>, u32)> = vec![
            ("fgn".into(), fgn_body(&[0u8; 16]), 101),
            ("rcn".into(), rcn_body(1, &[0u8; 16]), 102),
            (
                "memrcn".into(),
                {
                    let mut v = 1u16.to_le_bytes().to_vec();
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend_from_slice(&0i32.to_le_bytes());
                    v.extend_from_slice(&0i32.to_le_bytes());
                    v.extend_from_slice(&0u16.to_le_bytes());
                    v.extend_from_slice(&0u16.to_le_bytes());
                    v.extend_from_slice(&[0u8; 16]);
                    v.extend_from_slice(&[0u8; 16]);
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v
                },
                103,
            ),
            (
                "querytn".into(),
                {
                    let mut v = 1u16.to_le_bytes().to_vec();
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend_from_slice(&[0u8; 16]);
                    v.extend_from_slice(&[0u8; 16]);
                    v
                },
                104,
            ),
            (
                "relquerytn".into(),
                {
                    let mut v = 3u16.to_le_bytes().to_vec();
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend_from_slice(&[0u8; 16]);
                    v.extend_from_slice(&0u16.to_le_bytes());
                    v.extend_from_slice(&[0u8; 16]);
                    v.extend_from_slice(&[0u8; 16]);
                    v.extend(str_bytes(""));
                    v.extend_from_slice(&0u16.to_le_bytes());
                    v.extend_from_slice(&0u16.to_le_bytes());
                    v.extend_from_slice(&0u16.to_le_bytes());
                    v
                },
                105,
            ),
            (
                "dbtn".into(),
                {
                    let mut v = str_bytes("");
                    v.extend(str_bytes(""));
                    v.extend(str_bytes(""));
                    v.extend_from_slice(&entry(&WORKSPACE_NAME_CLASS_ID, 999));
                    v.extend(workspace_body());
                    v.extend(str_bytes(""));
                    v.extend_from_slice(&0i32.to_le_bytes());
                    v.extend_from_slice(&[0u8; 16]);
                    v
                },
                106,
            ),
        ];
        for (name, bytes, key) in cases {
            for end in 0..bytes.len() {
                let mut reader = Reader::new(&bytes[..end]);
                let mut table = RefTable10_1::new();
                table.register(key);
                let result: io::Result<()> = match name.as_str() {
                    "fgn" => {
                        read_geometric_network_name_body_10_1(&mut reader, 64, &mut table, key)
                            .map(|_| ())
                    }
                    "rcn" => {
                        read_relationship_class_name_body_10_1(&mut reader, 64, &mut table, key)
                            .map(|_| ())
                    }
                    "memrcn" => read_memory_relationship_class_name_body_10_1(
                        &mut reader,
                        64,
                        &mut table,
                        key,
                    )
                    .map(|_| ()),
                    "querytn" => read_query_table_name_body_10_1(&mut reader, 64, &mut table, key)
                        .map(|_| ()),
                    "relquerytn" => {
                        read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, key)
                            .map(|_| ())
                    }
                    _ => read_db_table_name_body_10_1(&mut reader, 64, &mut table, key).map(|_| ()),
                };
                if result.is_ok() {
                    // 工作空间槽嵌套体在极端截断处可能合法停止而非错误。
                    continue;
                }
                assert_eq!(reader.position(), 0, "{name} 截断 {end} 应保持游标");
                assert!(table.contains(key), "{name} 截断 {end} 应保留原登记");
                assert!(
                    table.len() <= 2,
                    "{name} 截断 {end} 登记数异常: {}",
                    table.len()
                );
            }
        }
    }

    #[test]
    fn class_id_constants_match_esri_clsid_declarations() {
        // esri.clsid.ecfg 声明（模块 GdbCore.dll）↔ GetClassID 立即数写出；
        // 此处核对内存序字节与 GUID 字符串互证。
        fn hex(s: &[u8]) -> String {
            s.iter().map(|x| format!("{x:02x}")).collect()
        }
        fn guid(b: &[u8; 16]) -> String {
            format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{}-{}",
                b[3],
                b[2],
                b[1],
                b[0],
                b[5],
                b[4],
                b[7],
                b[6],
                hex(&b[8..10]),
                hex(&b[10..16])
            )
        }
        assert_eq!(
            guid(&GEOMETRIC_NETWORK_NAME_CLASS_ID),
            "f452b4d2-9a95-11d2-aacf-00c04fa37b82"
        );
        assert_eq!(
            guid(&RELATIONSHIP_CLASS_NAME_CLASS_ID),
            "311c2ee1-da65-11d2-8a54-000000000000"
        );
        assert_eq!(
            guid(&MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID),
            "6dba211b-ebdb-11d3-9f84-00c04f6bc886"
        );
        assert_eq!(
            guid(&QUERY_TABLE_NAME_CLASS_ID),
            "793873de-c0bb-48f4-a6bf-f6f83fa3c1bc"
        );
        assert_eq!(
            guid(&REL_QUERY_TABLE_NAME_CLASS_ID),
            "dab3ee10-0f92-455d-8aa2-3d4ade5b2f7d"
        );
        assert_eq!(
            guid(&DB_TABLE_NAME_CLASS_ID),
            "4c9307b2-c125-4497-ba95-aae875553b06"
        );
        assert_eq!(guid(&IID_4BA33331), "4ba33331-d55f-11d1-8882-0000f877762d");
        assert_eq!(guid(&IID_2E060684), "2e060684-e14c-44d3-858a-442ab6a0a863");
        assert_eq!(guid(&IID_FDFEBD93), "fdfebd93-ed75-11d0-9a95-080009ec734b");
        assert_eq!(guid(&IID_F7AD47D1), "f7ad47d1-d55e-11d1-8882-0000f877762d");
        assert_eq!(guid(&IID_4434D680), "4434d680-f4da-11d1-bca6-0000f875bcce");
    }
}
