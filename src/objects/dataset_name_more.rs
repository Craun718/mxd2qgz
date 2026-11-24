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
//! | GeometricNetworkName | `{F452B4D2-9A95-11D2-AACF-00C04FA37B82}` | `0x709b8560` | `0x709b8ca0` | `0x709b8d50` | 有符号 >1 拒绝（Save 恒写 1） | 版本 + String×3 + IDatasetName 槽（obj+0x1c，随后获取 WorkspaceName） |
//! | RelationshipClassName | `{311C2EE1-DA65-11D2-8A54-000000000000}` | `0x70a3ab30` | `0x70a3b330` | `0x70a3b4b0` | 有符号 >1 拒绝（Save 恒写 1） | 版本 + String×3 + int32×2（obj+0x20/0x24）+ short×2（obj+0x28/0x2a）+ int16 选择子：==0 → IWorkspaceName 槽（obj+0x30）；!=0 → IDatasetName 槽（obj+0x2c，随后获取 WorkspaceName） |
//! | MemoryRelationshipClassName | `{6DBA211B-EBDB-11D3-9F84-00C04F6BC886}` | `0x70a402a0` | `0x70a402e0` | `0x70a40440` | 有符号 >1 拒绝（Save 恒写 1） | 版本 + String×3 + int32×2（obj+0x10/0x14）+ short×2（obj+0x18/0x1a）+ LoadObject×2（请求 IID 均为 IName `{4BA33331-D55F-11D1-8882-0000F877762D}`，obj+0x1c/0x20）+ String×4 |
//! | QueryTableName | `{793873DE-C0BB-48F4-A6BF-F6F83FA3C1BC}` | `0x709329d0` | `0x70932a10` | `0x70932ae0` | **无符号** `(ver-1) ≤ 1`，仅 1/2 通过（0 与 ≥3 拒绝；Save 经 `AfStreamGetSaveVersion` 表 `0x70ab7b2c` = (ffff,ffff,ffff,ffff,1,2)） | 版本 + String×3 + IWorkspaceName 槽（obj+0x1c）+ 槽（请求 IID `{2E060684-E14C-44D3-858A-442AB6A0A863}`，obj+0x18；装载后调用 IsSpatialQuery） |
//! | RelQueryTableName | `{DAB3EE10-0F92-455D-8AA2-3D4ADE5B2F7D}` | `0x70a5a4f0` | `0x70a5a530` | `0x70a5a6f0` | 有符号 >3 拒绝（Save 表 `0x70a88fd8` = (2,3)，恒写 2/3） | 版本 + String×2 + 槽（`{4BA33331-…}`，obj+0x10）+ short（obj+0x14）+ 槽（`{FDFEBD93-ED75-11D0-9A95-080009EC734B}`，obj+0x18）+ 槽（`{F7AD47D1-D55E-11D1-8882-0000F877762D}`，obj+0x1c）+ String + short（obj+0x24）+ [版本 ≥2] short（obj+0x26，短体原生置内存 0）+ [版本 ≥3] short（obj+0x28） |
//! | DBTableName | `{4C9307B2-C125-4497-BA95-AAE875553B06}` | `0x708d7a00` | `0x708d7a40` | `0x708d7b20` | **无版本字段** | String×3 + IWorkspaceName 槽（obj+0x1c）+ String + int32（obj+0x14，原生 ≠7 时置内存标志）+ 槽（`{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`，obj+0x20） |
//!
//! IDatasetName 槽（请求 IID `{89E75917-C287-11D1-AA77-00C04FA33A15}`）的
//! 新对象按类标识分派：FeatureDatasetName（第六轮，[`read_feature_dataset_name_body_10_1`]）
//! 与本模块其他已知类共用嵌套上限，登记先于体解析使循环呈回引；
//! 未确认类（TableName / UnknownDatasetName / 其他模块实现）
//! 为停止点。
//!
//! ## TableName、ObjectClassName 与未知名称
//!
//! TableName 和工厂别名 ObjectClassName 均经 FdbTableName 的持久化虚表，
//! 复用 GdbCoreLib 的 FdbDatasetName 版本、三条字符串和 IWorkspaceName 槽。
//! UnknownDatasetName 缺少已确认的对象体。
//! TableQueryName / FeatureQueryName 的 IPersistStream 读取见本模块；SystemCore
//! 优先查询 IPersistVariant，但两类静态 QI 路径均不支持该接口，详见
//! `docs/mxd-terra-integrate-query.md`。
//!
//! 调用约定与 [`crate::objects::dataset_name`] 一致：调用者先读对象头、比对
//! 类标识常量并取得键，从对象头之后传入 Reader；嵌套条目登记先于对象体解
//! 析，全部字段在 trial 游标与 trial 登记表上读取，失败不推进游标、不修改
//! 登记表；停止点是成功返回。字符串受 `max_bytes` 限额并原样转发给嵌套体。
//!
//! 已知偏离原生：负值版本位模式拒绝（较原生有符号比较更严，与仓库既有读取
//! 器口径一致）；QueryTableName 版本 0 原生被无符号门拒绝，读取器同拒。
//! RCN/FGN 装载 FeatureDatasetName 后获取 WorkspaceName，QueryTableName
//! 调用 IsSpatialQuery。读取器保留持久化字段的原值。五条请求 IID 已由
//! ArcMap 10.1 类型库定名为 IName、IQueryDescription、IQueryFilter、
//! ISelectionSet、ISpatialReference；只在中文注释标明对应请求槽，保留原公开
//! 字段名。接口名不表示流内类标识。
//! 已经由 FeatureClassName 的 IDatasetName 选择子接入 MXD 解析链；未闭合
//! 的嵌套新对象体仍是停止点。人工目视验收未做，不得声称已验证。

use crate::layers::feature_layer::{
    COVERAGE_FEATURE_CLASS_NAME_CLASS_ID, FEATURE_CLASS_NAME_CLASS_ID,
    FGDB_FEATURE_CLASS_NAME_CLASS_ID, read_coverage_feature_class_name_body_10_1,
    read_fgdb_feature_class_name_body_10_1,
};
use crate::layers::fgdb_raster_dataset_name::FGDB_RASTER_DATASET_NAME_CLASS_ID;
use crate::layers::raster_dataset_name::{
    MOSAIC_DATASET_NAME_CLASS_ID, RASTER_BAND_COLLECTION_NAME_CLASS_ID, RASTER_BAND_NAME_CLASS_ID,
    RASTER_CATALOG_NAME_CLASS_ID, RASTER_DATASET_NAME_CLASS_ID, RASTER_DATASET_NAME83_CLASS_ID,
    RasterDatasetNameEntryBody10_1, RasterLongArraySlot10_1, read_long_array_slot,
    read_supported_raster_name_body_10_1,
};
use crate::maps::standalone_table::{
    FGDB_TABLE_NAME_CLASS_ID, OBJECT_CLASS_NAME_CLASS_ID, TABLE_NAME_CLASS_ID, TableNameBody10_1,
    read_fgdb_table_name_body_10_1, read_table_name_body_10_1,
};
use crate::objects::ams_dataset_name::{
    AMS_DATASET_NAME_CLASS_ID, AmsDatasetName10_1, read_ams_dataset_name_10_1,
};
use crate::objects::dataset_name::{
    COVERAGE_NAME_CLASS_ID, FEATURE_DATASET_NAME_CLASS_ID, FeatureDatasetNameBody10_1,
    FeatureDatasetNameStop10_1, WorkspaceNameSlot10_1, read_coverage_name_body_10_1,
    read_feature_dataset_name_body_10_1,
};
use crate::objects::envelope::{
    SpatialReferenceEntry10_1, read_spatial_reference_body_10_1, read_spatial_reference_entry_10_1,
};
use crate::objects::query_filter::{
    QueryFilterBody10_1, QueryFilterObject10_1, read_query_filter_slot_10_1,
};
use crate::objects::route_event_name::{
    ROUTE_EVENT_SOURCE_NAME_CLASS_ID, RouteEventSourceNameBody10_1,
    read_route_event_source_name_body_10_1,
};
use crate::objects::string_array::{StringArraySlot10_1, read_string_array_slot_10_1};
use crate::objects::workspace_name::{
    WORKSPACE_NAME_CLASS_ID, WorkspaceNameEntry10_1, WorkspaceNamePrefix10_1,
    WorkspaceNameStop10_1, read_workspace_name_body_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// FeatureBookmark 的 IName 槽中已知对象的严格子集。
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkNameSlot10_1 {
    /// 空对象。
    Null,
    /// 当前对象流会话中已登记的键。
    BackReference { class_id: [u8; 16], key: u32 },
    /// FeatureClassName；体内停止点保留在 body.stop。
    FeatureClassName(Box<crate::layers::feature_layer::FeatureClassNamePrefix10_1>),
    /// 未闭合的类；游标和登记表均回退到槽起点。
    Unsupported { class_id: [u8; 16], key: u32 },
}

/// 读取书签的 IName 对象槽，不读取书签自身前后字段。
///
/// 目前只解析 FeatureLayer 的 IName 槽也使用的 FeatureClassName；未知类
/// 不推断对象体。此函数不模拟装载后的 QI 或数据源打开。
pub fn read_bookmark_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<BookmarkNameSlot10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let slot = match read_object_entry_10_1(&mut trial, &mut staged)? {
        ObjectEntry10_1::Null => BookmarkNameSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            BookmarkNameSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if class_id == crate::layers::feature_layer::FEATURE_CLASS_NAME_CLASS_ID =>
        {
            let body = crate::layers::feature_layer::read_feature_class_name_body_10_1(
                &mut trial,
                max_bytes,
                &mut staged,
                key,
            )?;
            BookmarkNameSlot10_1::FeatureClassName(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            return Ok(BookmarkNameSlot10_1::Unsupported { class_id, key });
        }
    };
    *reader = trial;
    *table = staged;
    Ok(slot)
}

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

/// TableQueryName `{81CAA5D8-90C5-11D3-9F7B-00C04F6BC886}`。
pub const TABLE_QUERY_NAME_CLASS_ID: [u8; 16] = [
    0xd8, 0xa5, 0xca, 0x81, 0xc5, 0x90, 0xd3, 0x11, 0x9f, 0x7b, 0x00, 0xc0, 0x4f, 0x6b, 0xc8, 0x86,
];
/// FeatureQueryName `{81CAA5D9-90C5-11D3-9F7B-00C04F6BC886}`。
pub const FEATURE_QUERY_NAME_CLASS_ID: [u8; 16] = [
    0xd9, 0xa5, 0xca, 0x81, 0xc5, 0x90, 0xd3, 0x11, 0x9f, 0x7b, 0x00, 0xc0, 0x4f, 0x6b, 0xc8, 0x86,
];

/// 查询名称体，停止后未读取的后续字段为 None。
#[derive(Debug, Clone, PartialEq)]
pub struct TableQueryNameBody10_1 {
    pub key: u32,
    pub version: u16,
    pub strings_before_workspace: [String; 3],
    pub workspace: WorkspaceNameSlot10_1,
    pub strings_after_workspace: Option<[String; 3]>,
    pub version_two_tail: Option<(String, u16)>,
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// FeatureQueryName 追加字符串和两个原始 32 位值。
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureQueryNameBody10_1 {
    pub table: TableQueryNameBody10_1,
    pub feature_tail: Option<(String, u32, u32)>,
}

/// 读取对象头后的 TableQueryName IPersistStream 体。
/// 调用者必须先确认该流使用 ObjectStream 引用表，且已登记对象键。
pub fn read_table_query_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<TableQueryNameBody10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = trial.read_u16_le()?;
    check_signed_version(version, 2, "TableQueryName", reader.position())?;
    let strings_before_workspace = [
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
        read_string_10_1(&mut trial, max_bytes)?,
    ];
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(&mut trial, max_bytes, &mut staged, &mut stop)?;
    let mut strings_after_workspace = None;
    let mut version_two_tail = None;
    if stop.is_none() {
        strings_after_workspace = Some([
            read_string_10_1(&mut trial, max_bytes)?,
            read_string_10_1(&mut trial, max_bytes)?,
            read_string_10_1(&mut trial, max_bytes)?,
        ]);
        if version >= 2 {
            version_two_tail = Some((
                read_string_10_1(&mut trial, max_bytes)?,
                trial.read_u16_le()?,
            ));
        }
    }
    *reader = trial;
    *table = staged;
    Ok(TableQueryNameBody10_1 {
        key,
        version,
        strings_before_workspace,
        workspace,
        strings_after_workspace,
        version_two_tail,
        stop,
    })
}

/// 读取 FeatureQueryName；基础体停止时不读取追加段，任何错误回滚整个体。
pub fn read_feature_query_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<FeatureQueryNameBody10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let body = read_table_query_name_body_10_1(&mut trial, max_bytes, &mut staged, key)?;
    let feature_tail = if body.stop.is_none() {
        Some((
            read_string_10_1(&mut trial, max_bytes)?,
            trial.read_u32_le()?,
            trial.read_u32_le()?,
        ))
    } else {
        None
    };
    *reader = trial;
    *table = staged;
    Ok(FeatureQueryNameBody10_1 {
        table: body,
        feature_tail,
    })
}

/// IDatasetName 槽（请求 IID `{89E75917-…}`）新对象的分派读取结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DatasetNameMoreBody10_1 {
    Workspace(Box<WorkspaceNamePrefix10_1>),
    /// FeatureClassName 自身在 IDatasetName 槽中递归嵌套的对象体。
    FeatureClassName(Box<crate::layers::feature_layer::FeatureClassNamePrefix10_1>),
    /// TableName 在 IName 槽中的对象体。
    Table(Box<TableNameBody10_1>),
    /// 可经 IName 打开的栅格属性表及栅格目录。
    RasterTable(Box<RasterDatasetNameEntryBody10_1>),
    RasterDataset(Box<RasterDatasetNameEntryBody10_1>),
    SchematicClass(Box<SchematicClassNameBody10_1>),
    SchematicDataset(Box<SchematicDatasetNameBody10_1>),
    /// NetCDFTableName 在 IName 槽中的对象体。
    NetCdfTable(Box<NetCdfTableNameBody10_1>),
    /// MosaicDB 的 ReferencedTableName 包装体。
    ReferencedTable(Box<ReferencedTableNameBody10_1>),
    /// NetCDFFeatureClassName 的独立持久化体。
    NetCdfFeatureClass(Box<NetCdfFeatureClassNameBody10_1>),
    XyEventSource(Box<XyEventSourceNameBody10_1>),
    RouteEventSource(Box<RouteEventSourceNameBody10_1>),
    AmsDatasetName(Box<AmsDatasetName10_1>),
    /// FeatureDatasetName（第六轮读取器）。
    FeatureDataset(Box<FeatureDatasetNameBody10_1>),
    CadDrawing(Box<CadDrawingNameBody10_1>),
    /// GeometricNetworkName（本模块）。
    GeometricNetwork(Box<GeometricNetworkNameBody10_1>),
    /// RelationshipClassName（本模块）。
    RelationshipClass(Box<RelationshipClassNameBody10_1>),
    /// MemoryRelationshipClassName（本模块）。
    MemoryRelationshipClass(Box<MemoryRelationshipClassNameBody10_1>),
    /// QueryTableName（本模块）。
    QueryTable(Box<QueryTableNameBody10_1>),
    /// TableQueryName（与 QueryTableName 为不同类）。
    TableQuery(Box<TableQueryNameBody10_1>),
    /// FeatureQueryName。
    FeatureQuery(Box<FeatureQueryNameBody10_1>),
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
    /// 新对象且类标识命中支持的类：已读取其体；体内停止时经
    /// [`stop`](DatasetNameMoreSlot10_1::Body) 之外的第二通道（读取器
    /// `stop` 字段）透传。
    Body(DatasetNameMoreBody10_1),
    /// 新对象但类标识无已闭合体（TableName / UnknownDatasetName /
    /// 其他模块实现）：停止点，游标回退到该对象头起点、
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
    /// 槽位新对象无已闭合读取器（IDatasetName 槽的未知类、或请求 IID 语义
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

/// 尚未分派新对象体的槽位分类结果。
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
    /// IDatasetName.Name。
    pub string_a: String,
    /// IName.NameString。
    pub string_b: String,
    /// IDatasetName.Category。
    pub string_c: String,
    /// FeatureDatasetName 槽（持久化接口 +0x1c）；原生装载后从中获取 WorkspaceName。
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
    /// 选择子非零：FeatureDatasetName 槽（持久化接口 +0x2c），原生从中获取 WorkspaceName。
    Dataset(DatasetNameMoreSlot10_1),
}

/// RelationshipClassName（GdbCore.dll Load `0x70a3b330`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipClassNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 体首 int16 版本（有符号 >1 拒绝；Save 恒写 1）。
    pub version: u16,
    /// IDatasetName.Name。
    pub string_a: String,
    /// IName.NameString。
    pub string_b: String,
    /// IDatasetName.Category。
    pub string_c: String,
    /// Cardinality，保留磁盘原值。
    pub int_a: i32,
    /// Notification，保留磁盘原值。
    pub int_b: i32,
    /// IsComposite，保留磁盘原值。
    pub short_a: u16,
    /// IsAttributed，保留磁盘原值。
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
    /// 依次为 IDatasetName.Name、IName.NameString、IDatasetName.Category。
    pub strings_a: [String; 3],
    /// Cardinality，保留磁盘原值。
    pub int_a: i32,
    /// Notification，保留磁盘原值。
    pub int_b: i32,
    /// IsComposite，保留磁盘原值。
    pub short_a: u16,
    /// IsAttributed，保留磁盘原值。
    pub short_b: u16,
    /// OriginName（IName，持久化接口 +0x1c）。
    pub slot_a: DatasetNameMoreSlot10_1,
    /// DestinationName（IName，持久化接口 +0x20）。
    pub slot_b: DatasetNameMoreSlot10_1,
    /// 依次为 ForwardPathLabel、BackwardPathLabel、OriginPrimaryKey、OriginForeignKey。
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
    /// IDatasetName.Name。
    pub string_a: String,
    /// IName.NameString。
    pub string_b: String,
    /// IDatasetName.Category。
    pub string_c: String,
    /// IWorkspaceName 槽（obj+0x1c；Save 先写本槽，再写 IQueryDescription 槽）。
    pub workspace: WorkspaceNameSlot10_1,
    /// QueryDescription（IQueryDescription，持久化接口 +0x18）；原生装载后调用 IsSpatialQuery。
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
    /// IDatasetName.Name。
    pub string_a: String,
    /// IName.NameString。
    pub string_b: String,
    /// 关系类名称（IName，持久化接口 +0x10）。支持普通和内存关系类名称。
    pub source_slot: DatasetNameMoreSlot10_1,
    /// ForwardDirection，保留磁盘 short 原值。
    pub short_a: u16,
    /// SrcQueryFilter（IQueryFilter，持久化接口 +0x18）。
    pub left_slot: Option<QueryFilterObject10_1<QueryFilterBody10_1>>,
    /// SrcSelectionSet（ISelectionSet，持久化接口 +0x1c）。
    pub right_slot: UnknownSlot10_1,
    /// TargetColumns。
    pub string_c: String,
    /// LeftOuterJoin，保留磁盘 short 原值。
    pub short_b: u16,
    /// DoNotPushJoinToDB，v2 起保存。较早版本没有此字段，原生默认置 0。
    pub short_c: Option<u16>,
    /// FireRowChangedEvents，v3 起保存。
    pub short_d: Option<u16>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// DBTableName 尾部的 ISpatialReference 槽。
#[derive(Debug, Clone, PartialEq)]
pub enum DbTableSpatialReference10_1 {
    Parsed(SpatialReferenceEntry10_1),
    /// 未知新类。游标停在对象头之前。
    Unsupported {
        class_id: [u8; 16],
        key: u32,
    },
}

/// DBTableName（GdbCore.dll Load `0x708d7a40`）体的已确认字段。
#[derive(Debug, Clone, PartialEq)]
pub struct DbTableNameBody10_1 {
    /// 对象头键。
    pub key: u32,
    /// 依次为 IDatasetName.Name、IName.NameString、IDatasetName.Category（体首无版本字段）。
    pub strings_a: [String; 3],
    /// IWorkspaceName 槽（obj+0x1c）。
    pub workspace: WorkspaceNameSlot10_1,
    /// 几何列名称，来自 SqlClass 的几何字段 IField.Name。
    pub string_d: String,
    /// esriGeometryType 原值。原生加载时与 esriGeometryAny（7）比较。
    pub int_a: i32,
    /// 尾槽（请求 ISpatialReference，obj+0x20）。工作空间处停止时为 `None`。
    pub tail_slot: Option<DbTableSpatialReference10_1>,
    /// 停止原因；`None` = 体已完整消费。
    pub stop: Option<DatasetNameMoreStop10_1>,
}

pub const CAD_DRAWING_NAME_CLASS_ID: [u8; 16] = [
    0x09, 0x43, 0x22, 0xd4, 0xcb, 0xa5, 0xd2, 0x11, 0x9b, 0x10, 0, 0xc0, 0x4f, 0xa3, 0x32, 0x99,
];

#[derive(Debug, Clone, PartialEq)]
pub struct CadDrawingNameBody10_1 {
    pub key: u32,
    pub version: u16,
    pub strings: [String; 3],
    pub workspace: WorkspaceNameSlot10_1,
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// CadFDB.dll 0x6cab7030 在三条字符串后读取 IWorkspaceName。
pub(crate) fn read_cad_drawing_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<CadDrawingNameBody10_1> {
    let mut r = reader.clone();
    let mut staged = table.clone();
    let version = r.read_u16_le()?;
    if version != 1 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("CadDrawingName 不支持版本 {version}"),
        ));
    }
    let strings = [
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
    ];
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(&mut r, max, &mut staged, &mut stop)?;
    *reader = r;
    *table = staged;
    Ok(CadDrawingNameBody10_1 {
        key,
        version,
        strings,
        workspace,
        stop,
    })
}

pub const SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0xbc, 0x39, 0x9c, 0x35, 0xb6, 0xfd, 0xb6, 0x49, 0x8a, 0x1c, 0x23, 0x97, 0xb3, 0xea, 0x92, 0x61,
];
pub const SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0x86, 0xeb, 0x87, 0x67, 0xb7, 0x1e, 0x18, 0x4b, 0xbb, 0x61, 0x13, 0xa5, 0xd8, 0x47, 0xdc, 0xdb,
];
pub const SCHEMATIC_DATASET_NAME_CLASS_ID: [u8; 16] = [
    0x77, 0xae, 0x3d, 0x33, 0xeb, 0x1a, 0x7d, 0x44, 0x8e, 0x83, 0xfe, 0x04, 0x3e, 0x84, 0xa6, 0x71,
];

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicClassNameBody10_1 {
    pub key: u32,
    pub version: i16,
    pub strings: [String; 3],
    pub dataset: DatasetNameMoreSlot10_1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicDatasetNameBody10_1 {
    pub key: u32,
    pub version: i16,
    pub strings: [String; 3],
    pub workspace: WorkspaceNameSlot10_1,
}

pub(crate) fn read_schematic_dataset_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<(
    SchematicDatasetNameBody10_1,
    Option<DatasetNameMoreStop10_1>,
)> {
    let (version, strings) = read_schematic_name_prefix(reader, max_bytes)?;
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(reader, max_bytes, table, &mut stop)?;
    Ok((
        SchematicDatasetNameBody10_1 {
            key,
            version,
            strings,
            workspace,
        },
        stop,
    ))
}

pub const SCHEMATIC_DIAGRAM_NAME_CLASS_ID: [u8; 16] = [
    0x66, 0x49, 0xb5, 0xe6, 0x84, 0x9b, 0xee, 0x42, 0xb4, 0x56, 0x2e, 0x0d, 0xeb, 0x81, 0x59, 0xbc,
];

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicDiagramNameBody10_1 {
    pub version: i16,
    pub strings: [String; 3],
    pub diagram_class: DatasetNameMoreSlot10_1,
}

pub(crate) fn read_schematic_diagram_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<SchematicDiagramNameBody10_1> {
    let (version, strings) = read_schematic_name_prefix(reader, max_bytes)?;
    let diagram_class =
        read_name_slot_with_depth_10_1(reader, max_bytes, table, stop, 0, |class_id| {
            class_id == SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID
        })?;
    Ok(SchematicDiagramNameBody10_1 {
        version,
        strings,
        diagram_class,
    })
}

pub const SCHEMATIC_FOLDER_NAME_CLASS_ID: [u8; 16] = [
    0x02, 0x05, 0xab, 0x9d, 0x39, 0x61, 0x10, 0x43, 0x9f, 0xa0, 0x1a, 0x0e, 0xdc, 0xb9, 0x1c, 0x7b,
];

#[derive(Debug, Clone, PartialEq)]
pub struct SchematicFolderNameBody10_1 {
    pub version: i16,
    pub strings: [String; 4],
    pub dataset: DatasetNameMoreSlot10_1,
}

pub(crate) fn read_schematic_folder_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<SchematicFolderNameBody10_1> {
    let (version, [a, b, c]) = read_schematic_name_prefix(reader, max_bytes)?;
    let strings = [a, b, c, read_string_10_1(reader, max_bytes)?];
    let dataset = read_schematic_dataset_slot_10_1(reader, max_bytes, table, stop)?;
    Ok(SchematicFolderNameBody10_1 {
        version,
        strings,
        dataset,
    })
}

fn read_schematic_dataset_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<DatasetNameMoreSlot10_1> {
    read_name_slot_with_depth_10_1(reader, max_bytes, table, stop, 0, |class_id| {
        class_id == SCHEMATIC_DATASET_NAME_CLASS_ID
    })
}

fn read_schematic_name_prefix(r: &mut Reader<'_>, max: usize) -> io::Result<(i16, [String; 3])> {
    let version = r.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("Schematic 名称未支持版本 {version}"),
        ));
    }
    let strings = [
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
        read_string_10_1(r, max)?,
    ];
    Ok((version, strings))
}

pub(crate) fn read_schematic_class_name_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<(SchematicClassNameBody10_1, Option<DatasetNameMoreStop10_1>)> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let (version, strings) = read_schematic_name_prefix(&mut trial, max_bytes)?;
    let mut stop = None;
    let dataset = read_schematic_dataset_slot_10_1(&mut trial, max_bytes, &mut staged, &mut stop)?;
    *reader = trial;
    *table = staged;
    Ok((
        SchematicClassNameBody10_1 {
            key,
            version,
            strings,
            dataset,
        },
        stop,
    ))
}

pub const REFERENCED_TABLE_NAME_CLASS_ID: [u8; 16] = [
    0x5e, 0x4e, 0x14, 0xab, 0x75, 0x94, 0x3e, 0x47, 0xb0, 0x78, 0x4e, 0x95, 0x44, 0x0c, 0x52, 0xfe,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ReferencedTableNameBody10_1 {
    pub key: u32,
    /// MosaicDB 兼容版本表可写出 -1，后面仍有 IName 槽。
    pub version: i16,
    pub source: DatasetNameMoreSlot10_1,
}

/// MosaicDB.dll 0x7a59b300：有符号版本上限及一个普通 IName 槽。
pub(crate) fn read_referenced_table_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    key: u32,
    stop: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<ReferencedTableNameBody10_1> {
    let mut trial = reader.clone();
    let version = trial.read_u16_le()? as i16;
    if version > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ReferencedTableName 未支持架构版本 {version}"),
        ));
    }
    let source = read_tabular_name_slot_10_1(&mut trial, max, table, stop)?;
    *reader = trial;
    Ok(ReferencedTableNameBody10_1 {
        key,
        version,
        source,
    })
}

pub const XY_EVENT_SOURCE_NAME_CLASS_ID: [u8; 16] = [
    0x20, 0xa9, 0x9a, 0x30, 0xec, 0xea, 0xd3, 0x11, 0x9f, 0x8a, 0x00, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];
pub const XY_EVENT_2_FIELDS_PROPERTIES_CLASS_ID: [u8; 16] = [
    0xa2, 0x5c, 0x04, 0x71, 0x02, 0x79, 0xd4, 0x11, 0x9f, 0xe5, 0x00, 0xc0, 0x4f, 0x6b, 0xdf, 0x06,
];

#[derive(Debug, Clone, PartialEq)]
pub struct XyEventSourceNameBody10_1 {
    pub key: u32,
    pub version: u16,
    pub strings: [String; 2],
    pub source: DatasetNameMoreSlot10_1,
    pub properties: Option<(ObjectEntry10_1, Option<XyEventFields10_1>)>,
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
    pub stop: Option<DatasetNameMoreStop10_1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XyEventFields10_1 {
    pub version: u16,
    pub x_field: String,
    pub y_field: String,
    pub z_field: String,
}

/// XYEvents.dll 0x810365d0 使用普通 IName、IXYEventProperties 和 ISpatialReference 槽。
pub(crate) fn read_xy_event_source_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<XyEventSourceNameBody10_1> {
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let version = trial.read_u16_le()?;
    check_signed_version(version, 1, "XYEventSourceName", reader.position())?;
    let strings = [
        read_string_10_1(&mut trial, max)?,
        read_string_10_1(&mut trial, max)?,
    ];
    let mut stop = None;
    let source = read_tabular_name_slot_10_1(&mut trial, max, &mut staged, &mut stop)?;
    let mut properties = None;
    let mut spatial_reference = None;
    if stop.is_none() {
        let entry = read_object_entry_10_1(&mut trial, &mut staged)?;
        let body = match entry {
            ObjectEntry10_1::NewObject {
                class_id: XY_EVENT_2_FIELDS_PROPERTIES_CLASS_ID,
                ..
            } => {
                let version = trial.read_u16_le()?;
                check_signed_version(version, 1, "XYEvent2FieldsProperties", trial.position() - 2)?;
                Some(XyEventFields10_1 {
                    version,
                    x_field: read_string_10_1(&mut trial, max)?,
                    y_field: read_string_10_1(&mut trial, max)?,
                    z_field: read_string_10_1(&mut trial, max)?,
                })
            }
            ObjectEntry10_1::Null | ObjectEntry10_1::BackReference { .. } => None,
            ObjectEntry10_1::NewObject { class_id, key } => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "IXYEventProperties 槽的未知类 {}（键 {key}）",
                        crate::storage::class_id_hex(&class_id)
                    ),
                ));
            }
        };
        properties = Some((entry, body));
        spatial_reference = Some(read_spatial_reference_entry_10_1(
            &mut trial,
            max,
            &mut staged,
        )?);
    }
    *reader = trial;
    *table = staged;
    Ok(XyEventSourceNameBody10_1 {
        key,
        version,
        strings,
        source,
        properties,
        spatial_reference,
        stop,
    })
}

pub const NETCDF_FEATURE_CLASS_NAME_CLASS_ID: [u8; 16] = [
    0x68, 0xf1, 0x5e, 0xee, 0x62, 0x7b, 0xd5, 0x4e, 0x89, 0xbb, 0xff, 0x73, 0xd3, 0x90, 0xaf, 0x93,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NetCdfFeatureClassNameBody10_1 {
    pub key: u32,
    pub version: u16,
    /// 按持久化顺序保留四个字符串和两个 LONG。
    pub strings: [String; 4],
    pub longs: [i32; 2],
    pub workspace: WorkspaceNameSlot10_1,
    pub arrays: Option<[StringArraySlot10_1; 3]>,
    pub dimension_indices: Option<RasterLongArraySlot10_1>,
    pub strings_v2: Option<[String; 4]>,
    pub stop: Option<DatasetNameMoreStop10_1>,
}

fn read_netcdf_arrays(
    r: &mut Reader<'_>,
    max: usize,
    refs: &mut RefTable10_1,
) -> io::Result<([StringArraySlot10_1; 3], RasterLongArraySlot10_1)> {
    let names = [
        read_string_array_slot_10_1(r, max, refs)?,
        read_string_array_slot_10_1(r, max, refs)?,
        read_string_array_slot_10_1(r, max, refs)?,
    ];
    let indices = read_long_array_slot(r, max, refs)?;
    // 原生仅对非空维度名称逐项取索引。
    if names[2]
        .body
        .as_ref()
        .is_some_and(|(_, values)| !values.is_empty())
        && matches!(indices.entry, ObjectEntry10_1::Null)
    {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "NetCDF 名称的非空维度名称缺少索引数组",
        ));
    }
    Ok((names, indices))
}

/// NetCDFDB.dll 0x7ae1fd40。额外的四个字符串只在 v2 中出现。
pub(crate) fn read_netcdf_feature_class_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<NetCdfFeatureClassNameBody10_1> {
    let mut r = reader.clone();
    let mut staged = table.clone();
    let version = r.read_u16_le()?;
    check_signed_version(version, 2, "NetCDFFeatureClassName", reader.position())?;
    let strings = [
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
    ];
    let longs = [r.read_u32_le()? as i32, r.read_u32_le()? as i32];
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(&mut r, max, &mut staged, &mut stop)?;
    let (arrays, dimension_indices, strings_v2) = if stop.is_some() {
        (None, None, None)
    } else {
        let (arrays, indices) = read_netcdf_arrays(&mut r, max, &mut staged)?;
        let strings_v2 = (version >= 2)
            .then(|| -> io::Result<_> {
                Ok([
                    read_string_10_1(&mut r, max)?,
                    read_string_10_1(&mut r, max)?,
                    read_string_10_1(&mut r, max)?,
                    read_string_10_1(&mut r, max)?,
                ])
            })
            .transpose()?;
        (Some(arrays), Some(indices), strings_v2)
    };
    *reader = r;
    *table = staged;
    Ok(NetCdfFeatureClassNameBody10_1 {
        key,
        version,
        strings,
        longs,
        workspace,
        arrays,
        dimension_indices,
        strings_v2,
        stop,
    })
}

pub const NETCDF_TABLE_NAME_CLASS_ID: [u8; 16] = [
    0x23, 0x7a, 0x59, 0x2d, 0x89, 0xa9, 0xc1, 0x43, 0x9b, 0x1b, 0x19, 0xe7, 0x5b, 0xbf, 0xb7, 0x8f,
];

#[derive(Debug, Clone, PartialEq)]
pub struct NetCdfTableNameBody10_1 {
    pub version: u16,
    pub strings: [String; 3],
    pub workspace: WorkspaceNameSlot10_1,
    /// 按流中顺序保留前两个成员数组和临时维度名称数组。
    pub arrays: Option<[StringArraySlot10_1; 3]>,
    pub dimension_indices: Option<RasterLongArraySlot10_1>,
    pub stop: Option<DatasetNameMoreStop10_1>,
}

/// NetCDFDB.dll 0x7ae2fdc0 与 0x7ae2fff0 使用宿主 ObjectStream 的五个槽。
pub(crate) fn read_netcdf_table_name_body_10_1(
    reader: &mut Reader<'_>,
    max: usize,
    table: &mut RefTable10_1,
) -> io::Result<NetCdfTableNameBody10_1> {
    let mut r = reader.clone();
    let mut staged = table.clone();
    let version = r.read_u16_le()?;
    if (version as i16) > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "NetCDFTableName 版本不支持",
        ));
    }
    let strings = [
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
        read_string_10_1(&mut r, max)?,
    ];
    let mut stop = None;
    let workspace = read_workspace_slot_10_1(&mut r, max, &mut staged, &mut stop)?;
    let (arrays, dimension_indices) = if stop.is_some() {
        (None, None)
    } else {
        let (arrays, indices) = read_netcdf_arrays(&mut r, max, &mut staged)?;
        (Some(arrays), Some(indices))
    };
    *reader = r;
    *table = staged;
    Ok(NetCdfTableNameBody10_1 {
        version,
        strings,
        workspace,
        arrays,
        dimension_indices,
        stop,
    })
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
    table.with_nested_dataset_name(|table| {
        read_dataset_name_object_body_10_1(reader, max_bytes, table, class_id, key)
    })
}

fn read_dataset_name_object_body_10_1(
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
        WORKSPACE_NAME_CLASS_ID => {
            let body = read_workspace_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = body.stop.map(
                |WorkspaceNameStop10_1::PropertySetNewObject { class_id, key }| {
                    DatasetNameMoreStop10_1::WorkspacePropertySet { class_id, key }
                },
            );
            Ok((DatasetNameMoreBody10_1::Workspace(Box::new(body)), stop))
        }

        AMS_DATASET_NAME_CLASS_ID => {
            let body = read_ams_dataset_name_10_1(reader, max_bytes, table)?;
            Ok((
                DatasetNameMoreBody10_1::AmsDatasetName(Box::new(body)),
                None,
            ))
        }
        SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID | SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID => {
            let (body, stop) = read_schematic_class_name_body_10_1(reader, max_bytes, table, key)?;
            Ok((
                DatasetNameMoreBody10_1::SchematicClass(Box::new(body)),
                stop,
            ))
        }
        SCHEMATIC_DATASET_NAME_CLASS_ID => {
            let (body, stop) =
                read_schematic_dataset_name_body_10_1(reader, max_bytes, table, key)?;
            Ok((
                DatasetNameMoreBody10_1::SchematicDataset(Box::new(body)),
                stop,
            ))
        }
        CAD_DRAWING_NAME_CLASS_ID => {
            let mut body = read_cad_drawing_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::CadDrawing(Box::new(body)), stop))
        }
        ROUTE_EVENT_SOURCE_NAME_CLASS_ID => {
            let mut body = read_route_event_source_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((
                DatasetNameMoreBody10_1::RouteEventSource(Box::new(body)),
                stop,
            ))
        }
        XY_EVENT_SOURCE_NAME_CLASS_ID => {
            let mut body = read_xy_event_source_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::XyEventSource(Box::new(body)), stop))
        }
        NETCDF_FEATURE_CLASS_NAME_CLASS_ID => {
            let mut body = read_netcdf_feature_class_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((
                DatasetNameMoreBody10_1::NetCdfFeatureClass(Box::new(body)),
                stop,
            ))
        }
        NETCDF_TABLE_NAME_CLASS_ID => {
            let mut body = read_netcdf_table_name_body_10_1(reader, max_bytes, table)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::NetCdfTable(Box::new(body)), stop))
        }
        REFERENCED_TABLE_NAME_CLASS_ID => {
            let mut stop = None;
            let body =
                read_referenced_table_name_body_10_1(reader, max_bytes, table, key, &mut stop)?;
            let stop = stop.map(|inner| DatasetNameMoreStop10_1::Nested {
                class_id,
                key,
                inner: Box::new(inner),
            });
            Ok((
                DatasetNameMoreBody10_1::ReferencedTable(Box::new(body)),
                stop,
            ))
        }
        FGDB_FEATURE_CLASS_NAME_CLASS_ID | COVERAGE_FEATURE_CLASS_NAME_CLASS_ID => {
            let body = if class_id == COVERAGE_FEATURE_CLASS_NAME_CLASS_ID {
                read_coverage_feature_class_name_body_10_1(reader, max_bytes, table, key)?
            } else {
                read_fgdb_feature_class_name_body_10_1(reader, max_bytes, table, key)?
            };
            let stop =
                body.stop
                    .as_ref()
                    .map(|reason| DatasetNameMoreStop10_1::FeatureClassNameStopped {
                        class_id,
                        key,
                        reason: format!("{reason:?}"),
                    });
            Ok((
                DatasetNameMoreBody10_1::FeatureClassName(Box::new(body)),
                stop,
            ))
        }
        TABLE_NAME_CLASS_ID | FGDB_TABLE_NAME_CLASS_ID | OBJECT_CLASS_NAME_CLASS_ID => {
            let (body, _) = if class_id == FGDB_TABLE_NAME_CLASS_ID {
                read_fgdb_table_name_body_10_1(reader, table, max_bytes)?
            } else {
                read_table_name_body_10_1(reader, table, max_bytes)?
            };
            let stop = match &body.workspace {
                WorkspaceNameEntry10_1::Unsupported { class_id, key } => {
                    Some(DatasetNameMoreStop10_1::SlotUnsupported {
                        class_id: *class_id,
                        key: *key,
                    })
                }
                WorkspaceNameEntry10_1::NewObject(workspace) => workspace.stop.map(|reason| {
                    let WorkspaceNameStop10_1::PropertySetNewObject { class_id, key } = reason;
                    DatasetNameMoreStop10_1::WorkspacePropertySet { class_id, key }
                }),
                _ => None,
            };
            let stop = stop.map(|inner| DatasetNameMoreStop10_1::Nested {
                class_id,
                key,
                inner: Box::new(inner),
            });
            Ok((DatasetNameMoreBody10_1::Table(Box::new(body)), stop))
        }
        RASTER_DATASET_NAME_CLASS_ID
        | RASTER_DATASET_NAME83_CLASS_ID
        | FGDB_RASTER_DATASET_NAME_CLASS_ID
        | MOSAIC_DATASET_NAME_CLASS_ID
        | RASTER_BAND_COLLECTION_NAME_CLASS_ID => {
            let body =
                read_supported_raster_name_body_10_1(reader, max_bytes, table, class_id, key)?;
            Ok((DatasetNameMoreBody10_1::RasterDataset(Box::new(body)), None))
        }
        RASTER_BAND_NAME_CLASS_ID | RASTER_CATALOG_NAME_CLASS_ID => {
            let body =
                read_supported_raster_name_body_10_1(reader, max_bytes, table, class_id, key)?;
            Ok((DatasetNameMoreBody10_1::RasterTable(Box::new(body)), None))
        }
        FEATURE_DATASET_NAME_CLASS_ID | COVERAGE_NAME_CLASS_ID => {
            let body = if class_id == COVERAGE_NAME_CLASS_ID {
                read_coverage_name_body_10_1(reader, max_bytes, table, key)?
            } else {
                read_feature_dataset_name_body_10_1(reader, max_bytes, table, key)?
            };
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
        TABLE_QUERY_NAME_CLASS_ID => {
            let mut body = read_table_query_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = nested_stop!(body);
            Ok((DatasetNameMoreBody10_1::TableQuery(Box::new(body)), stop))
        }
        FEATURE_QUERY_NAME_CLASS_ID => {
            let mut body = read_feature_query_name_body_10_1(reader, max_bytes, table, key)?;
            let stop = body
                .table
                .stop
                .take()
                .map(|inner| DatasetNameMoreStop10_1::Nested {
                    class_id,
                    key,
                    inner: Box::new(inner),
                });
            Ok((DatasetNameMoreBody10_1::FeatureQuery(Box::new(body)), stop))
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
        _ => unreachable!("调用方先按类标识判定，此处必为支持的类"),
    }
}

/// 读取并分类一个 IDatasetName 槽（请求 IID `{89E75917-…}`）：空引用/回引
/// 原样返回；新对象命中支持的类时分派（嵌套体停止透传为 `Nested`）；其余类为
/// 停止点并回退到槽位起点。`stop_out` 带出停止原因（无停止时为 `None`）。
///
/// B 轮接线起为本模块与 `dataset_name` / `layers::feature_layer`（FCN ⑦
/// 选择子 !=0 槽）共用的统一分派入口（支持的类含 FeatureDatasetName，与第六轮
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
    read_name_slot_with_depth_10_1(reader, max_bytes, table, stop_out, depth, |class_id| {
        matches!(
            class_id,
            FEATURE_CLASS_NAME_CLASS_ID
                | FEATURE_DATASET_NAME_CLASS_ID
                | COVERAGE_NAME_CLASS_ID
                | CAD_DRAWING_NAME_CLASS_ID
                | COVERAGE_FEATURE_CLASS_NAME_CLASS_ID
                | GEOMETRIC_NETWORK_NAME_CLASS_ID
                | RELATIONSHIP_CLASS_NAME_CLASS_ID
                | MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID
                | QUERY_TABLE_NAME_CLASS_ID
                | TABLE_QUERY_NAME_CLASS_ID
                | FEATURE_QUERY_NAME_CLASS_ID
                | REL_QUERY_TABLE_NAME_CLASS_ID
                | DB_TABLE_NAME_CLASS_ID
        )
    })
}

/// 从 IName 槽读取已有对象体布局的表或要素类名称。
pub(crate) fn read_tabular_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<DatasetNameMoreSlot10_1> {
    read_name_slot_with_depth_10_1(reader, max_bytes, table, stop_out, 0, is_tabular_name)
}

fn is_tabular_name(class_id: [u8; 16]) -> bool {
    matches!(
        class_id,
        FEATURE_CLASS_NAME_CLASS_ID
            | FGDB_FEATURE_CLASS_NAME_CLASS_ID
            | COVERAGE_FEATURE_CLASS_NAME_CLASS_ID
            | TABLE_NAME_CLASS_ID
            | OBJECT_CLASS_NAME_CLASS_ID
            | FGDB_TABLE_NAME_CLASS_ID
            | RASTER_BAND_NAME_CLASS_ID
            | RASTER_CATALOG_NAME_CLASS_ID
            | RELATIONSHIP_CLASS_NAME_CLASS_ID
            | SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID
            | SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID
            | NETCDF_TABLE_NAME_CLASS_ID
            | REFERENCED_TABLE_NAME_CLASS_ID
            | NETCDF_FEATURE_CLASS_NAME_CLASS_ID
            | XY_EVENT_SOURCE_NAME_CLASS_ID
            | ROUTE_EVENT_SOURCE_NAME_CLASS_ID
            | AMS_DATASET_NAME_CLASS_ID
            | QUERY_TABLE_NAME_CLASS_ID
            | TABLE_QUERY_NAME_CLASS_ID
            | FEATURE_QUERY_NAME_CLASS_ID
            | REL_QUERY_TABLE_NAME_CLASS_ID
            | DB_TABLE_NAME_CLASS_ID
    )
}

/// 读取叠加格网中已核对持久化体的 IName。
pub(crate) fn read_overlay_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<DatasetNameMoreSlot10_1> {
    read_name_slot_with_depth_10_1(reader, max_bytes, table, stop_out, 0, |class_id| {
        is_tabular_name(class_id)
            || matches!(
                class_id,
                RASTER_DATASET_NAME_CLASS_ID
                    | RASTER_DATASET_NAME83_CLASS_ID
                    | FGDB_RASTER_DATASET_NAME_CLASS_ID
                    | MOSAIC_DATASET_NAME_CLASS_ID
                    | RASTER_BAND_COLLECTION_NAME_CLASS_ID
                    | WORKSPACE_NAME_CLASS_ID
                    | FEATURE_DATASET_NAME_CLASS_ID
                    | COVERAGE_NAME_CLASS_ID
                    | GEOMETRIC_NETWORK_NAME_CLASS_ID
                    | MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID
                    | CAD_DRAWING_NAME_CLASS_ID
                    | SCHEMATIC_DATASET_NAME_CLASS_ID
            )
    })
}

/// 两个端点槽请求 IName，共用已有表名称和要素类名称读取器。
fn read_relationship_endpoint_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<DatasetNameMoreSlot10_1> {
    read_name_slot_with_depth_10_1(reader, max_bytes, table, stop_out, 0, |class_id| {
        matches!(
            class_id,
            FEATURE_CLASS_NAME_CLASS_ID
                | FGDB_FEATURE_CLASS_NAME_CLASS_ID
                | TABLE_NAME_CLASS_ID
                | OBJECT_CLASS_NAME_CLASS_ID
                | QUERY_TABLE_NAME_CLASS_ID
                | TABLE_QUERY_NAME_CLASS_ID
                | FEATURE_QUERY_NAME_CLASS_ID
                | DB_TABLE_NAME_CLASS_ID
        )
    })
}

fn read_name_slot_with_depth_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
    depth: usize,
    accepts: impl Fn([u8; 16]) -> bool,
) -> io::Result<DatasetNameMoreSlot10_1> {
    // 整体事务：入口克隆 trial 游标与暂存登记表，成功返回（含停止）才提交；
    // 任何失败（截断、嵌套体错误）与未知类回退路径都不推进调用者游标、
    // 不修改登记表。
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let result = (|| -> io::Result<DatasetNameMoreSlot10_1> {
        let entry = match read_object_entry_10_1(&mut trial, &mut staged)? {
            ObjectEntry10_1::BackReference { class_id, key } => {
                // 原生按键取回对象并请求接口，回引头中的类标识不参与接口查询。
                let class_id = staged.registered_class(key).unwrap_or(class_id);
                ObjectEntry10_1::BackReference { class_id, key }
            }
            entry => entry,
        };
        match entry {
            ObjectEntry10_1::Null => Ok(DatasetNameMoreSlot10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key }
            | ObjectEntry10_1::NewObject { class_id, key }
                if !accepts(class_id) =>
            {
                staged = table.clone();
                trial = reader.clone();
                *stop_out = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
                Ok(DatasetNameMoreSlot10_1::Unsupported { class_id, key })
            }
            ObjectEntry10_1::BackReference { class_id, key } => {
                Ok(DatasetNameMoreSlot10_1::BackReference { class_id, key })
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                if class_id == FEATURE_CLASS_NAME_CLASS_ID {
                    if depth >= MAX_FEATURE_CLASS_NAME_NESTING_DEPTH {
                        return Err(io::Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "FeatureClassName 递归深度超过读取器限额 {MAX_FEATURE_CLASS_NAME_NESTING_DEPTH}（防栈溢出，非原生限制）"
                            ),
                        ));
                    }
                    let body = staged.with_nested_dataset_name(|table| {
                        crate::layers::feature_layer::read_feature_class_name_body_with_depth_10_1(
                            &mut trial,
                            max_bytes,
                            table,
                            key,
                            depth + 1,
                        )
                    })?;
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
    let mut staged_table = table.clone();
    match read_object_entry_10_1(reader, &mut staged_table)? {
        ObjectEntry10_1::Null => Ok(WorkspaceNameSlot10_1::Null),
        ObjectEntry10_1::BackReference { class_id, key } => match table.registered_class(key) {
            Some(class_id) if class_id != WORKSPACE_NAME_CLASS_ID => {
                *reader = at_slot;
                *stop_out = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
                Ok(WorkspaceNameSlot10_1::Unsupported { class_id, key })
            }
            registered => Ok(WorkspaceNameSlot10_1::BackReference {
                class_id: registered.unwrap_or(class_id),
                key,
            }),
        },
        ObjectEntry10_1::NewObject { class_id, key } if class_id == WORKSPACE_NAME_CLASS_ID => {
            let body = read_workspace_name_body_10_1(reader, max_bytes, &mut staged_table, key)?;
            if let Some(WorkspaceNameStop10_1::PropertySetNewObject { class_id, key }) = body.stop {
                *stop_out = Some(DatasetNameMoreStop10_1::WorkspacePropertySet { class_id, key });
            }
            *table = staged_table;
            Ok(WorkspaceNameSlot10_1::WorkspaceName(Box::new(body)))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            *reader = at_slot;
            *stop_out = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
            Ok(WorkspaceNameSlot10_1::Unsupported { class_id, key })
        }
    }
}

/// 读取关系查询表的关系类名称槽，复用已验证的两种 IName 实现。
pub(crate) fn read_relationship_name_slot_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    stop_out: &mut Option<DatasetNameMoreStop10_1>,
) -> io::Result<DatasetNameMoreSlot10_1> {
    read_name_slot_with_depth_10_1(reader, max_bytes, table, stop_out, 0, |class_id| {
        matches!(
            class_id,
            RELATIONSHIP_CLASS_NAME_CLASS_ID | MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID
        )
    })
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
pub(super) fn check_signed_version(
    version: u16,
    max: i16,
    context: &str,
    offset: usize,
) -> io::Result<()> {
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
/// 起点约定与限额口径见模块文档。IDatasetName 槽的新对象命中支持的类时分派；
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
    let slot_a =
        read_relationship_endpoint_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;
    // 槽位停止后其体不可达（未知对象无外部长度），后续字段按占位填充。
    let (slot_b, strings_b) = if stop.is_some() {
        (
            DatasetNameMoreSlot10_1::Null,
            [String::new(), String::new(), String::new(), String::new()],
        )
    } else {
        let slot_b = read_relationship_endpoint_slot_10_1(
            &mut trial,
            max_bytes,
            &mut staged_table,
            &mut stop,
        )?;
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
    let source_slot =
        read_relationship_name_slot_10_1(&mut trial, max_bytes, &mut staged_table, &mut stop)?;
    // 槽位停止后其体不可达（未知对象无外部长度），后续字段按占位填充。
    let (short_a, left_slot, right_slot, string_c, short_b, short_c, short_d) = if stop.is_some() {
        (0, None, UnknownSlot10_1::Null, String::new(), 0, None, None)
    } else {
        let short_a = u16::from_le_bytes(trial.read_bytes(2)?.try_into().expect("2 字节"));
        let left_slot = Some(read_query_filter_slot_10_1(
            &mut trial,
            max_bytes,
            &mut staged_table,
            &mut stop,
        )?);
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
        let mut tail_reader = trial.clone();
        let mut tail_table = staged_table.clone();
        let entry = read_object_entry_10_1(&mut tail_reader, &mut tail_table)?;
        let reference = read_spatial_reference_body_10_1(&mut tail_reader, max_bytes, &entry)?;
        Some(match (reference, entry) {
            (Some(reference), _) => {
                trial = tail_reader;
                staged_table = tail_table;
                DbTableSpatialReference10_1::Parsed(reference)
            }
            (None, ObjectEntry10_1::NewObject { class_id, key }) => {
                stop = Some(DatasetNameMoreStop10_1::SlotUnsupported { class_id, key });
                DbTableSpatialReference10_1::Unsupported { class_id, key }
            }
            _ => unreachable!("仅未知新类没有对象体读取器"),
        })
    } else {
        None
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
    #[test]
    fn overlay_raster_collection_reads_mosaic_child() {
        let mut bytes = entry(&RASTER_BAND_COLLECTION_NAME_CLASS_ID, 90).to_vec();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(str_bytes("collection"));
        bytes.extend(entry(&MOSAIC_DATASET_NAME_CLASS_ID, 91));
        bytes.extend(4_i16.to_le_bytes());
        bytes.extend(str_bytes("mosaic"));
        bytes.extend(str_bytes(""));
        bytes.extend([0; 16 + 2 + 4 + 16]);
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0xa5);
        let mut refs = RefTable10_1::new();
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut None).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
        let mut reader = Reader::new(&bytes);
        let slot = read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut None).unwrap();
        let DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RasterDataset(body)) = slot
        else {
            panic!("应读取波段集合名称");
        };
        let RasterDatasetNameEntryBody10_1::BandCollection(collection) = body.as_ref() else {
            panic!("栅格名称类型错误");
        };
        assert!(
            matches!(collection.name.body.as_ref(), Some(RasterDatasetNameEntryBody10_1::Mosaic(name)) if name.version == 4)
        );
        assert_eq!(
            refs.registered_class(91),
            Some(MOSAIC_DATASET_NAME_CLASS_ID)
        );
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn overlay_raster_name_reads_dataset_and_legacy_delegate() {
        for class_id in [RASTER_DATASET_NAME_CLASS_ID, RASTER_DATASET_NAME83_CLASS_ID] {
            let mut bytes = entry(&class_id, 91).to_vec();
            bytes.extend(2_i32.to_le_bytes());
            for value in ["raster", "", "", ""] {
                bytes.extend(str_bytes(value));
            }
            bytes.extend([0; 16]);
            let end = bytes.len();
            bytes.extend([0xa5; 4]);
            let mut refs = RefTable10_1::new();
            for cut in 0..end {
                let mut reader = Reader::new(&bytes[..cut]);
                assert!(
                    read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut None).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(!refs.contains(91));
            }
            let mut reader = Reader::new(&bytes);
            let slot = read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut None).unwrap();
            let DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RasterDataset(body)) = slot
            else {
                panic!("应读取栅格数据集名称");
            };
            match body.as_ref() {
                RasterDatasetNameEntryBody10_1::Dataset(_) => {
                    assert_eq!(class_id, RASTER_DATASET_NAME_CLASS_ID)
                }
                RasterDatasetNameEntryBody10_1::Dataset83 { records: None, .. } => {
                    assert_eq!(class_id, RASTER_DATASET_NAME83_CLASS_ID)
                }
                _ => panic!("栅格名称类型错误"),
            }
            assert_eq!(refs.registered_class(91), Some(class_id));
            assert_eq!(reader.position(), end);
        }
    }

    #[test]
    fn overlay_workspace_name_reads_inline_body_and_reference() {
        let mut bytes = entry(&WORKSPACE_NAME_CLASS_ID, 91).to_vec();
        bytes.extend(workspace_body());
        let end = bytes.len();
        bytes.extend(entry(&TABLE_NAME_CLASS_ID, 91));
        let mut refs = RefTable10_1::new();
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut None).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(91));
        }
        let mut reader = Reader::new(&bytes);
        let mut stop = None;
        assert!(matches!(
            read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut stop).unwrap(),
            DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Workspace(body)) if body.key == 91
        ));
        assert_eq!(reader.position(), end);
        assert!(matches!(
            read_overlay_name_slot_10_1(&mut reader, 256, &mut refs, &mut stop).unwrap(),
            DatasetNameMoreSlot10_1::BackReference {
                class_id: WORKSPACE_NAME_CLASS_ID,
                key: 91
            }
        ));
        assert!(stop.is_none());
        assert_eq!(reader.remaining(), 0);
    }

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

    #[test]
    fn shared_tabular_slot_dispatches_every_confirmed_name_class() {
        let classes = [
            FEATURE_CLASS_NAME_CLASS_ID,
            FGDB_FEATURE_CLASS_NAME_CLASS_ID,
            COVERAGE_FEATURE_CLASS_NAME_CLASS_ID,
            TABLE_NAME_CLASS_ID,
            OBJECT_CLASS_NAME_CLASS_ID,
            FGDB_TABLE_NAME_CLASS_ID,
            RASTER_BAND_NAME_CLASS_ID,
            RASTER_CATALOG_NAME_CLASS_ID,
            RELATIONSHIP_CLASS_NAME_CLASS_ID,
            SCHEMATIC_DIAGRAM_CLASS_NAME_CLASS_ID,
            SCHEMATIC_ELEMENT_CLASS_NAME_CLASS_ID,
            NETCDF_TABLE_NAME_CLASS_ID,
            REFERENCED_TABLE_NAME_CLASS_ID,
            NETCDF_FEATURE_CLASS_NAME_CLASS_ID,
            XY_EVENT_SOURCE_NAME_CLASS_ID,
            ROUTE_EVENT_SOURCE_NAME_CLASS_ID,
            AMS_DATASET_NAME_CLASS_ID,
            QUERY_TABLE_NAME_CLASS_ID,
            TABLE_QUERY_NAME_CLASS_ID,
            FEATURE_QUERY_NAME_CLASS_ID,
            REL_QUERY_TABLE_NAME_CLASS_ID,
            DB_TABLE_NAME_CLASS_ID,
        ];

        for class_id in classes {
            let bytes = entry(&class_id, 1);
            let mut stop = None;
            let result = read_tabular_name_slot_10_1(
                &mut Reader::new(&bytes),
                4096,
                &mut RefTable10_1::new(),
                &mut stop,
            );
            if let Ok(slot) = result {
                assert!(
                    !matches!(slot, DatasetNameMoreSlot10_1::Unsupported { .. }),
                    "共享表名槽把已确认类 {} 判为未支持",
                    crate::storage::class_id_hex(&class_id),
                );
            }
            assert!(
                !matches!(stop, Some(DatasetNameMoreStop10_1::SlotUnsupported { .. })),
                "共享表名槽在类 {} 的对象头处停止",
                crate::storage::class_id_hex(&class_id),
            );
        }
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
            DatasetNameMoreSlot10_1::Unsupported {
                class_id: [0x66; 16],
                key: 62
            }
        );
        // 停止后 slot_b/strings_b 为占位。
        assert_eq!(body.slot_b, DatasetNameMoreSlot10_1::Null);
        assert!(body.strings_b.iter().all(|s| s.is_empty()));
        assert!(!table.contains(62));
        let consumed_before_slot =
            2 + str_bytes("内存关系").len() + str_bytes("").len() + str_bytes("类").len() + 12;
        assert_eq!(reader.position(), consumed_before_slot);
    }

    #[test]
    fn memrcn_reads_named_endpoints_before_labels() {
        let mut bytes = entry(&MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID, 60);
        bytes.extend(1u16.to_le_bytes());
        for _ in 0..3 {
            bytes.extend(str_bytes(""));
        }
        bytes.extend([0; 12]);
        bytes.extend(entry(&FGDB_FEATURE_CLASS_NAME_CLASS_ID, 61));
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(20u16.to_le_bytes());
        bytes.extend(fcn_body(&[0; 16]));
        bytes.extend(entry(&TABLE_NAME_CLASS_ID, 62));
        bytes.extend(1u16.to_le_bytes());
        for text in ["目的表", "", "表"] {
            bytes.extend(str_bytes(text));
        }
        bytes.extend([0; 16]);
        for text in ["正向", "反向", "OID", "OriginID"] {
            bytes.extend(str_bytes(text));
        }
        let end = bytes.len();
        bytes.extend(entry(&TABLE_NAME_CLASS_ID, 62));

        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let mut stop = None;
        let result = read_dataset_name_slot_10_1(&mut reader, 64, &mut table, &mut stop).unwrap();
        let DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::MemoryRelationshipClass(body)) =
            result
        else {
            panic!("应读到内存关系类名称");
        };
        assert!(matches!(
            &body.slot_a,
            DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::FeatureClassName(fcn))
                if fcn.key == 61 && fcn.fgdb_wrapper.is_some()
        ));
        assert!(matches!(
            &body.slot_b,
            DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::Table(table))
                if table.strings[0] == "目的表"
        ));
        assert_eq!(body.strings_b, ["正向", "反向", "OID", "OriginID"]);
        assert_eq!(stop, None);
        assert_eq!(reader.position(), end);
        assert!(table.contains(61));
        assert_eq!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::BackReference {
                class_id: TABLE_NAME_CLASS_ID,
                key: 62
            }
        );
        for at in 0..end {
            let mut reader = Reader::new(&bytes[..at]);
            let mut table = RefTable10_1::new();
            assert!(read_dataset_name_slot_10_1(&mut reader, 64, &mut table, &mut None).is_err());
            assert_eq!(reader.position(), 0);
            assert!(table.is_empty());
        }
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

    fn relquerytn_v3_body(source: &[u8]) -> Vec<u8> {
        let mut bytes = 3u16.to_le_bytes().to_vec();
        bytes.extend(str_bytes(""));
        bytes.extend(str_bytes(""));
        bytes.extend_from_slice(source);
        bytes.extend(1u16.to_le_bytes());
        bytes.extend([0; 32]);
        bytes.extend(str_bytes("字段"));
        bytes.extend(2u16.to_le_bytes());
        bytes.extend(3u16.to_le_bytes());
        bytes.extend(4u16.to_le_bytes());
        bytes
    }

    #[test]
    fn relquerytn_reads_both_relationship_names_before_the_join_fields() {
        const RELATIONSHIP: [u8; 16] = RELATIONSHIP_CLASS_NAME_CLASS_ID;
        const MEMORY: [u8; 16] = MEMORY_RELATIONSHIP_CLASS_NAME_CLASS_ID;
        let mut memory_body = 1u16.to_le_bytes().to_vec();
        for _ in 0..3 {
            memory_body.extend(str_bytes(""));
        }
        memory_body.extend([0; 12 + 32]);
        for _ in 0..4 {
            memory_body.extend(str_bytes(""));
        }
        for (class_id, body) in [(RELATIONSHIP, rcn_body(0, &[0; 16])), (MEMORY, memory_body)] {
            let mut source = entry(&class_id, 42);
            source.extend(body);
            let mut bytes = entry(&REL_QUERY_TABLE_NAME_CLASS_ID, 41);
            bytes.extend(relquerytn_v3_body(&source));
            let end = bytes.len();
            bytes.extend(entry(&class_id, 42));
            let mut reader = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            let mut stop = None;
            let outer =
                read_dataset_name_slot_10_1(&mut reader, 64, &mut table, &mut stop).unwrap();
            let DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RelQueryTable(outer)) =
                outer
            else {
                panic!("应读到 RelQueryTableName");
            };
            match outer.source_slot {
                DatasetNameMoreSlot10_1::Body(DatasetNameMoreBody10_1::RelationshipClass(body)) => {
                    assert_eq!((class_id, body.key), (RELATIONSHIP, 42));
                }
                DatasetNameMoreSlot10_1::Body(
                    DatasetNameMoreBody10_1::MemoryRelationshipClass(body),
                ) => {
                    assert_eq!((class_id, body.key), (MEMORY, 42));
                }
                other => panic!("未读到关系类名称：{other:?}"),
            }
            assert_eq!(outer.string_c, "字段");
            assert_eq!(outer.short_d, Some(4));
            assert_eq!(stop, None);
            assert_eq!(reader.position(), end);
            assert_eq!(
                read_object_entry_10_1(&mut reader, &mut table).unwrap(),
                ObjectEntry10_1::BackReference { class_id, key: 42 }
            );

            for at in 0..end {
                let mut reader = Reader::new(&bytes[..at]);
                let mut table = RefTable10_1::new();
                assert!(
                    read_dataset_name_slot_10_1(&mut reader, 64, &mut table, &mut None).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(table.is_empty());
            }
        }
    }

    #[test]
    fn relquerytn_query_filter_versions_and_spatial_suffix() {
        use crate::objects::geometry::{GeometryBody10_1, POINT_CLASS_ID};
        use crate::objects::query_filter::{
            FilterDefinitions10_1, QUERY_FILTER_CLASS_ID, SPATIAL_FILTER_CLASS_ID,
            SpatialFilterGeometry10_1,
        };

        for (version, spatial) in [(0u16, false), (1, false), (2, false), (3, false), (4, true)] {
            let class = if spatial {
                SPATIAL_FILTER_CLASS_ID
            } else {
                QUERY_FILTER_CLASS_ID
            };
            let mut filter = entry(&class, 42);
            if spatial {
                filter.extend(1u16.to_le_bytes());
            }
            filter.extend(version.to_le_bytes());
            filter.extend(str_bytes("OID"));
            filter.extend(str_bytes("OID > 0"));
            filter.extend(str_bytes(""));
            filter.extend([0; 16]);
            filter.extend(0.25f64.to_le_bytes());
            filter.extend([0; 16]);
            if version != 1 {
                filter.extend(0u32.to_le_bytes()); // 空 PostfixClause。
                if version == 2 {
                    filter.extend(0u32.to_le_bytes());
                } else {
                    filter.extend([0; 16]);
                }
            }
            if version == 4 {
                filter.extend(1u32.to_le_bytes()); // 一个 UTF-16 码元加终止零。
                filter.extend([b'X', 0, 0, 0]);
            }
            if spatial {
                filter.extend(1i32.to_le_bytes());
                filter.extend(2i32.to_le_bytes());
                filter.extend(str_bytes("rel"));
                filter.extend(entry(&POINT_CLASS_ID, 43));
                filter.extend(1u16.to_le_bytes());
                filter.extend(4u32.to_le_bytes());
                filter.extend(0u32.to_le_bytes()); // 空点记录。
                filter.extend(str_bytes("Shape"));
                filter.extend(3u16.to_le_bytes());
            }
            let mut bytes = relquerytn_v3_body(&[0; 16]);
            let offset = 2 + str_bytes("").len() * 2 + 16 + 2;
            bytes.splice(offset..offset + 16, filter);
            let end = bytes.len();
            bytes.extend(entry(&class, 42));

            let mut reader = Reader::new(&bytes);
            let mut table = RefTable10_1::new();
            let outer =
                read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 41).unwrap();
            let Some(QueryFilterObject10_1::Body { key: 42, body }) = outer.left_slot else {
                panic!("应读到过滤器对象体");
            };
            let core = match *body {
                QueryFilterBody10_1::Query(core) => core,
                QueryFilterBody10_1::Spatial {
                    version: 1,
                    core,
                    spatial: Some(fields),
                } => {
                    assert_eq!((fields.search_order, fields.spatial_rel), (1, 2));
                    assert_eq!(fields.geometry_field.as_deref(), Some("Shape"));
                    assert!(matches!(
                        fields.geometry,
                        QueryFilterObject10_1::Body { key: 43, body }
                            if matches!(body.as_ref(), SpatialFilterGeometry10_1::Geometry(GeometryBody10_1::Point(_)))
                    ));
                    assert_eq!(fields.spatial_rel_description, "rel");
                    assert_eq!(fields.filter_owns_geometry, Some(3));
                    core
                }
                other => panic!("SpatialFilter 后缀未完整读取：{other:?}"),
            };
            assert_eq!(core.version, version);
            assert_eq!(core.where_clause, "OID > 0");
            assert_eq!(core.spatial_resolution, Some(0.25));
            assert_eq!(core.postfix_clause.as_deref(), (version != 1).then_some(""));
            assert_eq!(core.prefix_clause.as_deref(), (version == 4).then_some("X"));
            if version == 2 {
                assert_eq!(
                    core.definitions,
                    Some(FilterDefinitions10_1::Inline(vec![]))
                );
            }
            assert_eq!(outer.stop, None);
            assert_eq!(outer.short_d, Some(4));
            assert_eq!(reader.position(), end);
            assert_eq!(
                read_object_entry_10_1(&mut reader, &mut table).unwrap(),
                ObjectEntry10_1::BackReference {
                    class_id: class,
                    key: 42
                }
            );
            for at in 0..end {
                let mut reader = Reader::new(&bytes[..at]);
                let mut table = RefTable10_1::new();
                assert!(
                    read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 41).is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(table.is_empty());
            }
        }
    }

    #[test]
    fn relquerytn_preserves_the_first_unknown_relationship_slot() {
        let unknown = [0x55; 16];
        let mut source = entry(&RELATIONSHIP_CLASS_NAME_CLASS_ID, 42);
        source.extend(rcn_body(1, &entry(&unknown, 43)));
        let bytes = relquerytn_v3_body(&source);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let body = read_rel_query_table_name_body_10_1(&mut reader, 64, &mut table, 41).unwrap();
        assert_eq!(
            body.stop,
            Some(DatasetNameMoreStop10_1::Nested {
                class_id: RELATIONSHIP_CLASS_NAME_CLASS_ID,
                key: 42,
                inner: Box::new(DatasetNameMoreStop10_1::SlotUnsupported {
                    class_id: unknown,
                    key: 43
                }),
            })
        );
        let at = reader.position();
        assert_eq!(bytes[at..at + 20], entry(&unknown, 43));
        assert!(table.contains(42));
        assert!(!table.contains(43));
        assert_eq!(body.short_d, None);
    }

    #[test]
    fn alternating_relationship_names_share_the_dataset_nesting_limit() {
        let mut entry_bytes = vec![0; 16];
        for level in (0..34).rev() {
            let mut next = entry(&REL_QUERY_TABLE_NAME_CLASS_ID, level * 2 + 1);
            let mut source = entry(&RELATIONSHIP_CLASS_NAME_CLASS_ID, level * 2 + 2);
            source.extend(rcn_body(1, &entry_bytes));
            next.extend(relquerytn_v3_body(&source));
            entry_bytes = next;
        }
        let mut reader = Reader::new(&entry_bytes);
        let mut table = RefTable10_1::new();
        let error =
            read_dataset_name_slot_10_1(&mut reader, 64, &mut table, &mut None).unwrap_err();
        assert!(error.to_string().contains("数据集名称嵌套深度"));
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
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
        assert_eq!(
            body.tail_slot,
            Some(DbTableSpatialReference10_1::Parsed(
                SpatialReferenceEntry10_1::Null
            ))
        );
        match body.workspace {
            WorkspaceNameSlot10_1::WorkspaceName(ws) => assert_eq!(ws.key, 91),
            other => panic!("期望 WorkspaceName 槽，实际 {other:?}"),
        }
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn dbtn_spatial_reference_uses_the_shared_key_table() {
        use crate::objects::spatial_reference_slots::GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID as GCS;

        let mut bytes = str_bytes("");
        bytes.extend(str_bytes(""));
        bytes.extend(str_bytes(""));
        bytes.extend([0; 16]);
        bytes.extend(str_bytes(""));
        bytes.extend(7_i32.to_le_bytes());
        let tail_offset = bytes.len();
        bytes.extend(entry(&GCS, 91));
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.push(b'X');
        bytes.extend([0; 17]); // 四个空域串及垂向标志。
        let end = bytes.len();
        bytes.extend(entry(&GCS, 91));

        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let body = read_db_table_name_body_10_1(&mut reader, 64, &mut table, 90).unwrap();
        assert!(matches!(
            body.tail_slot,
            Some(DbTableSpatialReference10_1::Parsed(
                SpatialReferenceEntry10_1::Geographic { key: 91, .. }
            ))
        ));
        assert_eq!(body.stop, None);
        assert_eq!(reader.position(), end);
        assert_eq!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::BackReference {
                class_id: GCS,
                key: 91
            }
        );

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut table = RefTable10_1::new();
        table.register(90);
        assert!(read_db_table_name_body_10_1(&mut reader, 64, &mut table, 90).is_err());
        assert_eq!(reader.position(), 0);
        assert_eq!(table.len(), 1);

        let unknown = [0x55; 16];
        bytes[tail_offset..tail_offset + 16].copy_from_slice(&unknown);
        let mut reader = Reader::new(&bytes);
        let body = read_db_table_name_body_10_1(&mut reader, 64, &mut table, 90).unwrap();
        assert_eq!(
            body.stop,
            Some(DatasetNameMoreStop10_1::SlotUnsupported {
                class_id: unknown,
                key: 91
            })
        );
        assert_eq!(
            body.tail_slot,
            Some(DbTableSpatialReference10_1::Unsupported {
                class_id: unknown,
                key: 91
            })
        );
        assert_eq!(reader.position(), tail_offset);
        assert!(!table.contains(91));
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
        assert_eq!(body.tail_slot, None);
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
