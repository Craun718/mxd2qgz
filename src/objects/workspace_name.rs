//! ArcMap 10.1 WorkspaceName（工作空间名称，GdbCore.dll `0x70a27950`）对象
//! 体已确认字段的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md` 的「FDOGraphicsLayer 体闭合…第六轮」
//! 节（Load `0x70a27950` 反编译与样本逐字节对位）。本模块自
//! `src/layers/feature_layer.rs` 移入 objects 层（消除 objects→layers 依赖倒
//! 置；`feature_layer` 保留 `pub use` 重导出，外部调用与测试不受影响）。全部
//! 证据未运行目标软件。
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
//! 未解决问题：尾随 int32 与各 String 的业务语义未定性（按内容候选），原样
//! 保留；IPropertySet 槽位其余实现类体未闭合（新对象为停止点）；全部结论为
//! 静态观察，未经目标软件运行验证。

use crate::objects::property_set::{
    PROPERTY_SET_CLASS_ID, PropertySetObjectStream10_1, read_property_set_object_stream_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// WorkspaceName::Load 接受的最大版本：`1 < ver` 拒绝；样本值 1。
const MAX_WS_VERSION: i16 = 1;

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

/// WorkspaceName 体读取的停止原因（体内在首个不可闭合处停止，非错误）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceNameStop10_1 {
    /// IPropertySet 槽位的新对象类标识非 PropertySet（其余 IPropertySet 实现
    /// 类体未闭合）：其后的标志、工厂 CLSID 与 int32 不可达。
    PropertySetNewObject { class_id: [u8; 16], key: u32 },
}

/// WorkspaceName 体内 IPropertySet 槽位（请求 IID `{F0BA6ABC-…}`）的分类
/// 结果（已接入 [`read_property_set_object_stream_10_1`]）。
#[derive(Debug, Clone, PartialEq)]
pub enum PropertySetSlot10_1 {
    /// 16 字节全零类标识：空引用，无键、无对象体。
    Null,
    /// 键已见：回引条目，无对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 新 PropertySet 对象（类标识经 [`PROPERTY_SET_CLASS_ID`] 校验）：已读
    /// 取其闭合体（版本、计数、[键名 String + 值 Variant] 循环）。
    PropertySet(Box<PropertySetObjectStream10_1>),
    /// 新对象但类标识非 PropertySet（其余实现类体未闭合）：停止点，游标回
    /// 退到该对象头起点、该键不保留（见
    /// [`WorkspaceNameStop10_1::PropertySetNewObject`]）。
    Unsupported { class_id: [u8; 16], key: u32 },
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
    /// IPropertySet 槽位（请求 IID `{F0BA6ABC-…}`）分类；新对象为 PropertySet
    /// 类时读取其闭合体，其余类为停止点。
    pub property_set: PropertySetSlot10_1,
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

/// 从对象头之后的位置读取 10.1 WorkspaceName 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`] 读取对
/// 象头、比对 [`WORKSPACE_NAME_CLASS_ID`] 并取得键 `key`，从对象头之后的位
/// 置传入 Reader。字段序与证据见模块文档。IPropertySet 槽新对象且类标识为
/// PropertySet 时读取其闭合体（样本两实例即该形态）；其余类新
/// 对象处停止是成功返回（停止原因 [`WorkspaceNameStop10_1`]）；截断、版本
/// 超限、字符串超限为错误。
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

    // 5. IPropertySet 槽位(请求 IID {F0BA6ABC-…}):新对象且类标识为
    //    PropertySet 时读取其闭合体(样本两实例为该形态);其余
    //    类体未闭合 → 停止并回退。
    let slot_offset = trial.position();
    let saved_table = table.clone();
    let mut staged_table = table.clone();
    let mut stop = None;
    let property_set = match read_object_entry_10_1(&mut trial, &mut staged_table)? {
        ObjectEntry10_1::Null => PropertySetSlot10_1::Null,
        ObjectEntry10_1::BackReference { class_id, key } => {
            PropertySetSlot10_1::BackReference { class_id, key }
        }
        ObjectEntry10_1::NewObject { class_id, key: _ } if class_id == PROPERTY_SET_CLASS_ID => {
            // 接线:PropertySet 体(版本 + 计数 + [键名 String + 值 Variant]
            // 循环)整体消费;失败不推进游标、不修改登记表(读取器自身事务性)。
            // WorkspaceName 已在 ObjectStream 会话中解码对象头。原生把同一
            // IStream 传给 PropertySet::Load，嵌套值必须共用会话键表。
            let body =
                read_property_set_object_stream_10_1(&mut trial, max_bytes, &mut staged_table)?;
            PropertySetSlot10_1::PropertySet(Box::new(body))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            // 其余 IPropertySet 实现类:该对象体未闭合、无法跳过,游标与登记
            // 表回退到槽位起点(回退到对象头起点的停止不保留该键——与
            // GroupLayer/CGL 读取器的回退语义一致)。
            stop = Some(WorkspaceNameStop10_1::PropertySetNewObject { class_id, key });
            trial = reader.reposition_to(slot_offset);
            staged_table = saved_table;
            PropertySetSlot10_1::Unsupported { class_id, key }
        }
    };
    let (factory_flag, factory_class_id, trailing_int32) = if stop.is_none() {
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
        (Some(flag), factory_class_id, Some(trailing))
    } else {
        (None, None, None)
    };

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
