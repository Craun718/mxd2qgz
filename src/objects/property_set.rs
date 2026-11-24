//! ArcMap 10.1 PropertySet（esriSystem 属性集 coclass）对象体的严格读取器。
//!
//! 证据来源：`docs/arcmap-10.1-property-set.md`（本轮研究记录）。原生实现为
//! **SystemCore.dll 10.1.0.3035**（`temp/arcgis101-bin/` 既有基线，SHA-256
//! `f63cf949…e3ad1a`，pefile FileVersion/ProductVersion 10.1.0.3035；安装配
//! 置 `esri.clsid.xml` 声明 `{588E5A11-D09B-11D1-AA7C-00C04FA33A15}` progID
//! `PropertySet` 模块 SystemCore.dll）。
//!
//! ## 定位链（10.1 静态映像 VA，ImageBase `0x7ffb0000`）
//!
//! CLSID 字节常量 `0x8007190c` → GetClassID `0x7ffd3040`（把 `0x8007190c`
//! 常量原样写出 16 字节，esri.clsid 声明、GetClassID 写出与样本对象头三方
//! 互证）→ GetClassID 数据引用 `0x8007184c`（adjustor thunk）→ IPersistStream
//! 虚表 **`0x80071840`**（8 槽：+0x00/04/08 为 QI/AddRef/Release 的 adjustor
//! thunk，+0x0c = GetClassID thunk `0x7ffd5570`，+0x10 = IsDirty `0x80053770`
//! （读 this+4 字节标志），**+0x14 = Load thunk `0x7ffd5560` → 实体
//! `0x7ffd3080`**，+0x18 = Save thunk `0x7ffd55c0` → 实体 `0x7ffd3280`，
//! +0x1c = GetSizeMax thunk `0x7ffd5550` → `0x80056620`）。同族虚表复核：
//! PropertySet 的 RTTI `.?AVPropertySet@@`（TypeDescriptor `0x800a69a8`）派
//! 出 7 个子接口 COL，本虚表 [-1] = COL `0x8008381c`。
//!
//! ## Load `0x7ffd3080` 字段序（证据：确定的）
//!
//! 1. int16 版本：汇编 `cmp word [esp+0x14],1; jle ok; mov eax,0x80004005`
//!    ——**有符号**比较，>1 拒绝 `0x80004005`；负值位模式原生通过，本读取
//!    器一并拒绝（较原生更严，写侧 Save 恒写 1，负值无写侧来源）；
//! 2. int32 属性计数：原生 `0 < 计数` 才进循环（≤0 直接成功返回，负数按空
//!    集合），本读取器对负计数显式拒绝（较原生更严，同 element_collection
//!    的处理）；
//! 3. 循环计数次，每条目 = [键名 + 值 Variant]：
//!    - 键名：`String::Read`（AfCore String 布局：4 字节 cb 含 UTF-16LE 终
//!      止零 + 载荷，经 [`crate::stream::native_string::read_string_10_1`]）；
//!    - 值：`Variant::Read(stream, true)`（SystemCore 本地编译副本，签名
//!      `?Read@Variant@@QAEJPAUIStream@@_N@Z` 与 AfCore 导出 `0x68a7c6a0`
//!      同名同参；AfCore 副本 VARTYPE 表见下）。值存入条目对象 +0x8 槽。
//!    任一读取失败即中止返回，无尾随字段。
//!
//! ## Variant::Read（AfCore `0x68a7c6a0`，非数组分支）的 VARTYPE 表
//!
//! 2 字节小端 VARTYPE 前缀；`true` 实参跳过版本读后的定位调用。分支：
//! VT_EMPTY(0)/VT_NULL(1) 无载荷；VT_I2(2)/VT_BOOL(11) 2 字节；VT_I4(3)/
//! VT_ERROR(10) 4 字节；VT_R4(4) 4 字节；VT_R8(5)/VT_DATE(7) 8 字节；
//! VT_CY(6) 8 字节；VT_BSTR(8) 走 AfCore `String::Read`（cb 含终止零布局，
//! **不是** VariantStream `ReadString` 的 `n*2+2` 码元布局）；VT_I1(0x10)/
//! VT_UI1(0x11) 1 字节；VT_UI2(0x12) 2 字节；VT_UI4(0x13) 4 字节；其余
//! VARTYPE 返回 `0x80004005`。VT_DISPATCH(9)/VT_UNKNOWN(13) 走 AfCore
//! 普通 IStream 下 `LoadObject` 读取 16 字节类 GUID、无键；类 GUID 全零的
//! 空对象子集映射为 `PropertyValue10_1::NullObject`，非零类 GUID 显式不支持。
//! 若调用上下文已确认 ObjectStream，应使用独立 API
//! [`read_property_set_object_stream_10_1`]：它读取 GUID 和共享键，分派
//! PropertySet 及已确认的 SystemCore 数组对象。一维定宽和 BSTR Variant
//! 数组使用独立的内联布局。
//!
//! ## Save `0x7ffd3280`（写方向互证，证据：确定的）
//!
//! 版本 short 写静态常量 `DAT_8006d0e4`（内存字节 `01 00`，**恒写 1**）→
//! int32 计数（成员 this-0x10）→ 循环 [键 `String::Write` + 值
//! `Variant::Write(variant, stream, true)`]，与 Load 逐字段对称。
//!
//! ## 样本对位（辅助观察）
//!
//! 仓库参考样本 /Maps 流内 2 个 PropertySet 实例（均为 WorkspaceName 体
//! IPropertySet 槽的新对象），体逐字节吻合：
//! - 键 1547 @`0x147e5`：版本 1 + 计数 1 + 键 `"DATABASE"` + VT_BSTR 路径
//!   值（42 字节 cb）——体恰终结于外层 WorkspaceName 的工厂标志字节
//!   `01` @`0x14845`（工厂 CLSID = `{4C91D963-…}` RasterWorkspaceFactory）；
//! - 键 2027 @`0x219a0`：同构（路径 cb 62），终结于工厂标志 `01`
//!   @`0x21a14`（`{A06ADB96-…}` ShapefileWorkspaceFactory）。
//! 样本仅覆盖版本 1、计数 1、VT_BSTR 值；其余分支无样本字节佐证。
//!
//! ## 建议接入点（不在本轮改动）
//!
//! - `feature_layer::read_workspace_name_body_10_1` 的 IPropertySet 槽
//!   （`WorkspaceNameStop10_1::PropertySetNewObject` 停止点）：类标识命中本
//!   常量时改调 [`read_property_set_10_1`];
//! - `dataset_name` / `feature_layer` 的同名停止点传播链;
//! - 其他 WorkspaceName/连接属性宿主。
//!
//! 调用约定与事务性：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头、比对 [`PROPERTY_SET_CLASS_ID`] 确认类身份，从对象头之后的位
//! 置传入 Reader；本函数不重复读取对象头。全部字段在 trial 游标上读取，成
//! 功返回才提交；任何失败（截断、版本超限、负计数、限额、未知 VARTYPE、
//! 对象/数组分支）都不推进调用者游标。`max_bytes` 约束键名与 BSTR 值的声
//! 明长度，并在任何条目读取之前预检「计数 × 每条目最小 6 字节」。
//! 旧入口 `table` 参数仅为与同类读取器签名一致，登记表不被读取或修改；
//! ObjectStream 解析使用新 API。这是仅覆盖上述分支的严格安全子集，
//! 不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
//!
//! 未解决问题（详见证据文档「开放项」）：SystemCore 本地 `Variant::Read`
//! 编译副本与 AfCore 副本的逐 VARTYPE 对照仅覆盖签名与样本 VT_BSTR 分支；
//! 各值的业务语义（DATABASE/连接属性候选）未定性；版本 0 无样本佐证；
//! 全部结论为静态观察，未经目标软件运行验证。

use crate::layers::dep_tail::{
    TIME_CLASS_ID, TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID, TimeBody10_1, TimeExtentBody10_1,
    TimeReferenceBody10_1, TimeZoneInfoSlot10_1, read_time_body_10_1, read_time_extent_body_10_1,
    read_time_reference_body_10_1,
};
use crate::layers::feature_layer_tail::{
    NumberFormatSlot10_1, UID_CLASS_ID, UidBody10_1, read_number_format_body_for_class_10_1,
    read_uid_body_10_1,
};
use crate::layers::image_server_rules::{
    RASTER_COLORMAP, RASTER_HISTOGRAM, RASTER_INFO_CLASS_ID, RASTER_INFOS_CLASS_ID,
    RASTER_STATISTICS, RasterInfoVariantBody10_1, RasterTemplateInput10_1, RasterVariantBody10_1,
    function_max_version, read_input_body, read_raster_info_variant_body_10_1,
    read_raster_variant_body_10_1, supports_function_arguments,
};
use crate::layers::raster_dataset_name::{LONG_ARRAY_CLASS_ID, read_long_array_body};
use crate::layers::raster_renderer::{
    DOUBLE_ARRAY_CLASS_ID, STATS_HISTOGRAM_CLASS_ID, read_double_array_body,
};
use crate::layers::time_layer_track_extension::{
    TIME_WINDOW_CLASS_ID, TimeWindow10_1, read_time_window,
};
use crate::objects::color_ramp::{
    ALGORITHMIC_COLOR_RAMP_CLASS_ID, ColorRampSlot10_1, MULTI_PART_COLOR_RAMP_CLASS_ID,
    PRESET_COLOR_RAMP_CLASS_ID, RANDOM_COLOR_RAMP_CLASS_ID, nested_ramp_stop,
    read_color_ramp_body_for_class_10_1,
};
use crate::objects::display_color::{ColorBody10_1, read_color_body_for_class_10_1};
use crate::objects::display_symbol::{
    GraphicAttributeList10_1, is_graphic_attribute_object, read_graphic_attribute_list_10_1,
};
use crate::objects::envelope::{
    ENVELOPE_CLASS_ID, SpatialReferenceEntry10_1, read_spatial_reference_body_10_1,
};
use crate::objects::geometry::{
    GEOMETRY_BAG_CLASS_ID, GeometryBody10_1, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID,
    POINT_CLASS_ID, POLYGON_CLASS_ID, POLYLINE_CLASS_ID, read_keyed_geometry_body_10_1,
};
use crate::objects::string_array::{STR_ARRAY_CLASS_ID, read_string_array_body_10_1};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use crate::stream::variant_array::{ClassVariantArray10_1, read_class_variant_array_10_1};
use std::io::{self, ErrorKind};
use std::sync::Arc;

/// PropertySet::Load 接受的最大版本：汇编 `cmp word,1; jle` 有符号比较，>1
/// 拒绝；Save 恒写 1；样本两实例均为 1。
const MAX_BODY_VERSION: i16 = 1;

/// 每个属性条目的最小流内字节数：键名 String 最小 4 字节（cb=0 空串）+
/// 值 Variant 最小 2 字节（无载荷 VARTYPE）。用于计数 ×最小条目的限额预检。
const MIN_ENTRY_BYTES: usize = 6;
/// 递归 PropertySet 与数组共用一个深度限额。
const MAX_NESTED_VARIANT_OBJECTS: usize = 64;
pub(crate) const RASTER_SCALAR_CLASS_ID: [u8; 16] = [
    0x59, 0xbf, 0xb2, 0xa5, 0x76, 0x78, 0xac, 0x4b, 0xab, 0x34, 0x7c, 0x2f, 0x36, 0x84, 0xb9, 0x5f,
];
pub(crate) const RASTER_ARGUMENT_ARRAY_CLASS_ID: [u8; 16] = [
    0x90, 0xcb, 0x6d, 0x92, 0x33, 0xda, 0xa2, 0x45, 0x8a, 0x24, 0x90, 0x7b, 0x66, 0x36, 0x7a, 0xa9,
];
const VAR_ARRAY_CLASS_ID: [u8; 16] = [
    0x74, 0x04, 0x2f, 0x76, 0xa2, 0xec, 0x5b, 0x47, 0x99, 0xf4, 0x26, 0x67, 0x8d, 0x79, 0x43, 0x6e,
];
const VAR_STR_ARRAY_CLASS_ID: [u8; 16] = [
    0xe7, 0x08, 0x57, 0x48, 0xbd, 0x6e, 0x5f, 0x41, 0xaa, 0x56, 0x7c, 0xd1, 0x78, 0x2a, 0xec, 0xd1,
];
const PROPERTY_SET_ARRAY_CLASS_ID: [u8; 16] = [
    0x8a, 0xbd, 0x4b, 0xc9, 0x33, 0xec, 0x21, 0x49, 0x8e, 0xc3, 0x6a, 0xd4, 0xb3, 0x32, 0x32, 0xc3,
];
const VALUE_ARRAY_CLASS_ID: [u8; 16] = [
    0xb6, 0xbb, 0x78, 0x0d, 0x8b, 0xad, 0x25, 0x4f, 0xa9, 0xf5, 0x44, 0x36, 0xed, 0xf9, 0xd2, 0x17,
];

/// PropertySet 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{588E5A11-D09B-11D1-AA7C-00C04FA33A15}`。证据：
/// esri.clsid.xml 声明（模块 SystemCore.dll，progID `PropertySet`）；
/// SystemCore.dll GetClassID `0x7ffd3040` 写出 `.rdata` 常量 `0x8007190c`
/// 的同一 CLSID；仓库参考样本 /Maps 流 `0x147e5`/`0x219a0` 两处对象头的类
/// 标识与本常量逐字节相同（辅助观察）。
pub const PROPERTY_SET_CLASS_ID: [u8; 16] = [
    0x11, 0x5a, 0x8e, 0x58, 0x9b, 0xd0, 0xd1, 0x11, 0xaa, 0x7c, 0x00, 0xc0, 0x4f, 0xa3, 0x3a, 0x15,
];

/// PropertySetProperty 类标识（CLSID）的 16 字节内存序，供诊断对照。
///
/// GUID 字符串形式为 `{5B6D2B14-F64A-4457-84E0-617AE2130FF9}`（esri.clsid
/// 声明 SystemCore.dll）。esriSystem 的键值对 coclass；本模块不读取其对象
/// 体（样本未见实例，属性集流内编码为键名+值内联，未出现该类对象头）。
pub const PROPERTY_SET_PROPERTY_CLASS_ID: [u8; 16] = [
    0x14, 0x2b, 0x6d, 0x5b, 0x4a, 0xf6, 0x57, 0x44, 0x84, 0xe0, 0x61, 0x7a, 0xe2, 0x13, 0x0f, 0xf9,
];

/// PropertySet 值 Variant 的已确认分支（`Variant::Read`）。
///
/// 固定宽度值不转换成有符号数、浮点数或布尔语义，仅原样保留位模式；
/// 字符串经 AfCore String 布局解码。对象（9/13）分支接入已闭合的「类
/// GUID 全零 → 空对象」子集（`NullObject`，经
/// [`crate::objects::variant_object_branch`]），非零类 GUID 显式不支持。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue10_1 {
    /// VT_EMPTY(0)：无载荷。
    Empty,
    /// VT_NULL(1)：无载荷。
    Null,
    /// VT_DISPATCH(9) / VT_UNKNOWN(13)：类 GUID 全零的空对象（原生
    /// `LoadObject` 返回 S_OK 且出参保持 NULL，流内恰 16 字节零）。
    NullObject,
    /// VT_I2(2) / VT_BOOL(11)：2 字节原样位模式。
    Raw2([u8; 2]),
    /// VT_I4(3) / VT_ERROR(10) / VT_R4(4)：4 字节原样位模式。
    Raw4([u8; 4]),
    /// VT_R8(5) / VT_DATE(7) / VT_CY(6)：8 字节原样位模式。
    Raw8([u8; 8]),
    /// VT_BSTR(8)：AfCore String 布局（4 字节 cb 含 UTF-16LE 终止零）。
    String(String),
    /// VT_I1(0x10) / VT_UI1(0x11)：1 字节原样位模式。
    Raw1([u8; 1]),
    /// VT_UI2(0x12)：2 字节原样位模式。
    Ui2([u8; 2]),
    /// VT_UI4(0x13)：4 字节原样位模式。
    Ui4([u8; 4]),
    /// 已确认元素类型的一维数组。
    Array(ClassVariantArray10_1),
}

/// 单个属性条目：键名（AfCore String）+ 值 Variant。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyEntry10_1 {
    /// 键名（流内 String::Read → 条目对象 +0x0 槽；业务语义按内容候选，
    /// 样本键名 `"DATABASE"` 为工作空间连接属性）。
    pub key: String,
    /// 值 Variant（流内 Variant::Read → 条目对象 +0x8 槽）。
    pub value: PropertyValue10_1,
}

/// 10.1 PropertySet::Load（`0x7ffd3080`）可读取的对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct PropertySet10_1 {
    /// 2 字节版本 short 的无符号位模式。
    ///
    /// 原生有符号比较 >1 拒绝（`0x80004005`）、负值位模式原生通过；本报
    /// 取器接受 0..=1 并拒绝负值（较原生更严：写侧 Save 恒写 1，负值无写
    /// 侧来源）。版本 0 与 1 的字段序相同（Load 无版本分支，仅上限拒绝）。
    pub version: u16,
    /// int32 属性计数的无符号值。原生对 ≤0 计数不进循环直接成功；本读取
    /// 器对负计数显式拒绝（较原生更严），0 表示空属性集。
    pub count: u32,
    /// 已消费的属性条目（完整消费时长度 == 计数）。
    pub entries: Vec<PropertyEntry10_1>,
    /// 体之后的剩余字节数（体无外部长度；正常输入中该体恰在宿主对象的下
    /// 一字段处终结）。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 PropertySet 对象体的严格子集，失败时不推
/// 进游标。
///
/// 字段序、VARTYPE 表与证据见模块文档。VT_BSTR 的字符串载荷与键名共用
/// AfCore String 布局，`max_bytes` 约束两者声明长度；`usize::MAX` 表示不额
/// 外设限。此入口保留普通 IStream 的保守上下文，不把 `table` 当作
/// ObjectStream 会话。需要会话引用表时使用独立的
/// [`read_property_set_object_stream_10_1`]。
pub fn read_property_set_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PropertySet10_1> {
    let _ = table;
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("PropertySet 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();

    // 1. int16 版本：原生有符号 `cmp word,1; jle`，>1 拒绝；负值一并拒绝。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=1（Load 0x7ffd3080 \
             有符号比较 >1 拒绝 0x80004005；Save 0x7ffd3280 恒写 1；负值位模式原生通过，\
             本读取器一并拒绝，较原生更严）"
        )));
    }

    // 2. int32 计数：原生仅正数进入循环（≤0 直接成功）；负数显式拒绝。
    let count_offset = trial.position();
    let count_raw = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count_raw < 0 {
        return Err(invalid(format!(
            "属性计数 offset 0x{count_offset:x} 为负数（{count_raw}）：原生按 ≤0 不进循环直\
             接成功，负数语义未知，本读取器拒绝"
        )));
    }
    let count = count_raw as usize;

    // 3. 限额预检（在任何条目读取之前）：计数 × 每条目最小 6 字节。
    let min_total = count
        .checked_mul(MIN_ENTRY_BYTES)
        .ok_or_else(|| invalid("属性计数 × 最小条目字节数在 usize 上溢出".to_string()))?;
    if min_total > max_bytes {
        return Err(invalid(format!(
            "声明计数 {count_raw} × 最小条目字节数 {MIN_ENTRY_BYTES} 超过资源限额 {max_bytes}，\
             在任何条目读取之前拒绝"
        )));
    }

    // 4. 条目循环：[键名 String + 值 Variant]。
    let mut entries = Vec::new();
    for index in 0..count {
        let key = read_string_10_1(&mut trial, max_bytes)
            .map_err(|error| invalid(format!("条目 {index} 键名读取失败：{error}")))?;
        let value = read_variant_10_1(&mut trial, max_bytes)
            .map_err(|error| invalid(format!("条目 {index}（键 {key:?}）的值读取失败：{error}")))?;
        entries.push(PropertyEntry10_1 { key, value });
    }

    let result = PropertySet10_1 {
        version,
        count: count_raw as u32,
        entries,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}

/// 已确认 ObjectStream 宿主中 Variant 值的表示，含对象体与会话回引。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValueObjectStream10_1 {
    /// 内联标量或普通空对象，编码与 [`PropertyValue10_1`] 相同。
    Inline(PropertyValue10_1),
    /// 新对象且类为已证明可读取的 PropertySet。
    Object {
        class_id: [u8; 16],
        key: u32,
        body: Box<PropertySetObjectStream10_1>,
    },
    Domain {
        class_id: [u8; 16],
        key: u32,
        body: Box<crate::objects::domain::DomainBody10_1>,
    },
    GeometryDef(Box<EncodedVariantObject10_1<crate::objects::field_defs::GeometryDef10_1>>),
    RasterDef(Box<EncodedVariantObject10_1<crate::objects::field_defs::RasterDef10_1>>),
    Field(Box<EncodedVariantObject10_1<crate::objects::fields::FieldBody10_1>>),
    Fields(Box<EncodedVariantObject10_1<crate::objects::fields::FieldsBody10_1>>),
    FeatureIdSet {
        class_id: [u8; 16],
        body: Box<crate::layers::feature_id_set::FeatureIdSetBody10_1>,
    },
    QueryFilter(Box<EncodedVariantObject10_1<crate::objects::query_filter::QueryFilterBody10_1>>),
    FilterDefinitions(
        Box<EncodedVariantObject10_1<crate::objects::query_filter::FilterDefinitionsBody10_1>>,
    ),
    XmlFilterDef(Box<EncodedVariantObject10_1<crate::objects::query_filter::XmlFilterDef10_1>>),
    /// 有对象头及会话键的 SystemCore 数组。
    ArrayObject {
        class_id: [u8; 16],
        key: u32,
        version: u16,
        values: VariantArrayObjectValues10_1,
    },
    /// 已有独立 IPersistStream 对象体的 SystemCore UID。
    Uid(UidBody10_1),
    NumberFormat(Box<EncodedVariantObject10_1<NumberFormatSlot10_1>>),
    TimeLayerTrack(
        Box<
            EncodedVariantObject10_1<
                crate::layers::time_layer_track_extension::TimeLayerTrackExtension10_1,
            >,
        >,
    ),
    RasterRenderer(
        Box<EncodedVariantObject10_1<crate::layers::raster_renderer::RasterRendererPrefix10_1>>,
    ),
    GeodataXform(
        Box<EncodedVariantObject10_1<crate::objects::geodata_xform::GeodataXformBody10_1>>,
    ),
    RasterUniqueValueColors(
        Box<
            EncodedVariantObject10_1<
                crate::layers::raster_unique_value::RasterUniqueValueColors10_1,
            >,
        >,
    ),
    UniqueValues(
        Box<EncodedVariantObject10_1<crate::layers::raster_unique_value::UniqueValues10_1>>,
    ),
    Time(Box<EncodedVariantObject10_1<VariantTimeBody10_1>>),
    Geometry(Box<VariantGeometry10_1>),
    Color(Box<EncodedVariantObject10_1<ColorBody10_1>>),
    ColorRamp(Box<EncodedVariantObject10_1<ColorRampSlot10_1>>),
    GraphicAttributes(Box<EncodedVariantObject10_1<GraphicAttributeList10_1>>),
    SpatialReference(Box<EncodedVariantObject10_1<SpatialReferenceEntry10_1>>),
    RasterInfo(Box<EncodedVariantObject10_1<RasterInfoVariantBody10_1>>),
    Raster {
        class_id: [u8; 16],
        key: u32,
        body: Box<RasterVariantBody10_1>,
    },
    RasterScalar {
        key: u32,
        version: u16,
        value: Box<PropertyValueObjectStream10_1>,
    },
    RasterArgumentArray(Box<RasterArgumentArray10_1>),
    RasterArgumentInput(Box<EncodedVariantObject10_1<RasterTemplateInput10_1>>),
    /// 同一 ObjectStream 会话中已登记对象的回引，无后续对象体。
    BackReference {
        class_id: [u8; 16],
        key: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantTimeBody10_1 {
    Time(TimeBody10_1),
    Reference(TimeReferenceBody10_1),
    Extent(TimeExtentBody10_1),
    Window(TimeWindow10_1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterArgumentArray10_1 {
    pub key: u32,
    pub version: u16,
    pub strings: [String; 2],
    pub declared_count: i32,
    pub objects: Vec<PropertyValueObjectStream10_1>,
}

pub type VariantGeometry10_1 = EncodedVariantObject10_1<GeometryBody10_1>;

/// 保留对象体的原字节，以其编码比较坐标、统计量和 NaN 的位模式。
#[derive(Debug, Clone)]
pub struct EncodedVariantObject10_1<T> {
    pub class_id: [u8; 16],
    pub key: u32,
    bytes: Arc<[u8]>,
    body: T,
}

impl<T> EncodedVariantObject10_1<T> {
    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl<T> PartialEq for EncodedVariantObject10_1<T> {
    fn eq(&self, other: &Self) -> bool {
        self.class_id == other.class_id && self.key == other.key && self.bytes == other.bytes
    }
}

impl<T> Eq for EncodedVariantObject10_1<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantArrayObjectValues10_1 {
    Strings(Vec<String>),
    Longs(Vec<i32>),
    /// 保留 double 位模式，包括 NaN 的载荷位与有符号零。
    DoubleBits(Vec<u64>),
    Variants(Vec<PropertyValueObjectStream10_1>),
    PropertySets(Vec<PropertySetArrayEntry10_1>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertySetArrayEntry10_1 {
    pub entry: ObjectEntry10_1,
    /// 回引只有对象头。
    pub body: Option<PropertySetObjectStream10_1>,
}

/// ObjectStream 中单个 PropertySet 条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyEntryObjectStream10_1 {
    pub key: String,
    pub value: PropertyValueObjectStream10_1,
}

/// ObjectStream 上下文中的 PropertySet 对象体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertySetObjectStream10_1 {
    pub version: u16,
    pub count: u32,
    pub entries: Vec<PropertyEntryObjectStream10_1>,
    pub remaining_bytes: usize,
}

/// 显式按 ObjectStream 会话读取 PropertySet 体，并共享调用者的引用表。
///
/// 本 API 不探测普通 IStream 是否实现 IObjectStream：调用者必须已由外层
/// 上下文确认 ObjectStream。新对象按 CLSID 分派到已证的 PropertySet 和
/// SystemCore 数组体。游标和引用表均事务性提交。
pub fn read_property_set_object_stream_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PropertySetObjectStream10_1> {
    read_property_set_object_stream_nested_10_1(reader, max_bytes, table, 0)
}

fn read_property_set_object_stream_nested_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<PropertySetObjectStream10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("PropertySet ObjectStream 体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let mut trial_table = table.clone();
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出接受范围 0..=1"
        )));
    }
    let count_offset = trial.position();
    let count_raw = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count_raw < 0 {
        return Err(invalid(format!(
            "属性计数 offset 0x{count_offset:x} 为负数（{count_raw}）"
        )));
    }
    let count = count_raw as usize;
    let min_total = count
        .checked_mul(MIN_ENTRY_BYTES)
        .ok_or_else(|| invalid("计数乘最小条目长度溢出".into()))?;
    if min_total > max_bytes {
        return Err(invalid(format!(
            "计数 {count} 的最小条目长度超过资源限额 {max_bytes}"
        )));
    }
    let mut entries = Vec::new();
    for index in 0..count {
        let key = read_string_10_1(&mut trial, max_bytes)
            .map_err(|error| invalid(format!("条目 {index} 键名读取失败：{error}")))?;
        let value = read_variant_object_stream_value_10_1(
            &mut trial,
            max_bytes,
            &mut trial_table,
            depth,
        )
        .map_err(|error| invalid(format!("条目 {index}（键 {key:?}）值读取失败：{error}")))?;
        entries.push(PropertyEntryObjectStream10_1 { key, value });
    }
    let result = PropertySetObjectStream10_1 {
        version,
        count: count_raw as u32,
        entries,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 在已确认的 ObjectStream 宿主中读取一个 AfCore Variant::Read(true) 值。
pub(crate) fn read_variant_object_stream_value_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<PropertyValueObjectStream10_1> {
    read_variant_object_stream_layout_10_1(reader, max_bytes, table, depth, false)
}

/// 已确认的 ObjectStream 宿主中的 Variant::Read(false)，保留空对象、回引和已知对象体。
pub(crate) fn read_padded_variant_object_stream_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PropertyValueObjectStream10_1> {
    read_padded_variant_object_stream_nested_10_1(reader, max_bytes, table, 0)
}

pub(crate) fn read_padded_variant_object_stream_nested_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
) -> io::Result<PropertyValueObjectStream10_1> {
    read_variant_object_stream_layout_10_1(reader, max_bytes, table, depth, true)
}

fn read_variant_object_stream_layout_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    padded: bool,
) -> io::Result<PropertyValueObjectStream10_1> {
    let mut trial = reader.clone();
    let value_type = trial.read_u16_le()?;
    if value_type != 9 && value_type != 13 {
        return read_variant_layout_10_1(reader, max_bytes, padded)
            .map(PropertyValueObjectStream10_1::Inline);
    }
    if padded {
        // AfCore 先跨过保留区，再把原流直接传给 LoadObject。
        trial.read_bytes(14)?;
    }
    let mut trial_table = table.clone();
    let entry = read_object_entry_10_1(&mut trial, &mut trial_table)?;
    let value =
        read_variant_object_body_10_1(&mut trial, max_bytes, &mut trial_table, depth, entry)?;
    *reader = trial;
    *table = trial_table;
    Ok(value)
}

/// 解析已登记的对象条目。调用者负责在失败时回滚对象头与子槽。
pub(crate) fn read_variant_object_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    entry: ObjectEntry10_1,
) -> io::Result<PropertyValueObjectStream10_1> {
    let depth = depth.max(table.raster_info_variant_depth());
    use crate::layers::feature_id_set::{
        FEATURE_ID_SET_CLASS_ID, FID_SET_CLASS_ID, read_feature_id_set_body_10_1,
        read_fid_set_body_10_1,
    };
    use crate::objects::domain::{
        BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID, CODED_VALUE_DOMAIN_CLASS_ID, RANGE_DOMAIN_CLASS_ID,
        read_domain_body_10_1,
    };
    use crate::objects::field_defs::{GEOMETRY_DEF_CLASS_ID, RASTER_DEF_CLASS_ID};
    use crate::objects::fields::{FIELD_CLASS_ID, FIELDS_CLASS_ID};
    use crate::objects::geodata_xform::{
        read_geodata_xform_body_10_1, supports_geodata_xform_body_10_1,
    };
    use crate::objects::query_filter::{
        FILTER_DEFS_CLASS_ID, QUERY_FILTER_CLASS_ID, SPATIAL_FILTER_CLASS_ID,
        XML_FILTER_DEF_CLASS_ID,
    };
    Ok(match entry {
        ObjectEntry10_1::Null => {
            PropertyValueObjectStream10_1::Inline(PropertyValue10_1::NullObject)
        }
        ObjectEntry10_1::BackReference { class_id, key } => {
            PropertyValueObjectStream10_1::BackReference {
                class_id: table.registered_class(key).unwrap_or(class_id),
                key,
            }
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if crate::layers::raster_renderer::supports_raster_renderer_body_10_1(class_id) =>
        {
            return read_variant_raster_renderer(reader, max_bytes, table, depth, class_id, key);
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if supports_geodata_xform_body_10_1(class_id) =>
        {
            check_variant_object_depth(depth)?;
            let mut wire = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body = table.with_raster_info_variant_depth(depth + 1, |table| {
                read_geodata_xform_body_10_1(&mut wire, max_bytes, table, class_id)
            })?;
            let bytes = Arc::from(reader.read_bytes(wire.position())?);
            PropertyValueObjectStream10_1::GeodataXform(Box::new(EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ crate::layers::raster_unique_value::RASTER_UNIQUE_VALUE_COLORS_CLASS_ID,
            key,
        } => return read_variant_raster_unique_colors(reader, max_bytes, class_id, key),
        ObjectEntry10_1::NewObject {
            class_id: class_id @ crate::layers::raster_unique_value::UNIQUE_VALUES_CLASS_ID,
            key,
        } => return read_variant_unique_values(reader, max_bytes, table, depth, class_id, key),
        ObjectEntry10_1::NewObject {
            class_id:
                class_id
                @ crate::layers::time_layer_track_extension::TIME_LAYER_TRACK_EXTENSION_CLASS_ID,
            key,
        } => return read_variant_time_layer_track(reader, max_bytes, table, class_id, key),
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (RANGE_DOMAIN_CLASS_ID
                | CODED_VALUE_DOMAIN_CLASS_ID
                | BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID),
            key,
        } => {
            check_variant_object_depth(depth)?;
            PropertyValueObjectStream10_1::Domain {
                class_id,
                key,
                body: Box::new(read_domain_body_10_1(
                    reader,
                    max_bytes,
                    table,
                    class_id,
                    depth + 1,
                )?),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (FIELD_CLASS_ID | FIELDS_CLASS_ID),
            key,
        } => return read_variant_field_object(reader, max_bytes, table, depth, class_id, key),
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (GEOMETRY_DEF_CLASS_ID | RASTER_DEF_CLASS_ID),
            key,
        } => return read_variant_field_definition(reader, max_bytes, table, class_id, key),
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (FEATURE_ID_SET_CLASS_ID | FID_SET_CLASS_ID),
            key,
        } => {
            let mut wire = reader
                .clone()
                .read_region(reader.remaining().min(max_bytes))?;
            let body = if class_id == FID_SET_CLASS_ID {
                read_fid_set_body_10_1(&mut wire, table, key)?
            } else {
                read_feature_id_set_body_10_1(&mut wire, table, key)?
            };
            reader.read_bytes(wire.position())?;
            PropertyValueObjectStream10_1::FeatureIdSet {
                class_id,
                body: Box::new(body),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (QUERY_FILTER_CLASS_ID | SPATIAL_FILTER_CLASS_ID),
            key,
        } => return read_variant_query_filter(reader, max_bytes, table, class_id, key),
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (FILTER_DEFS_CLASS_ID | XML_FILTER_DEF_CLASS_ID),
            key,
        } => return read_variant_filter_definition(reader, max_bytes, table, class_id, key),
        ObjectEntry10_1::NewObject { class_id, key } if class_id == PROPERTY_SET_CLASS_ID => {
            check_variant_object_depth(depth)?;
            let body =
                read_property_set_object_stream_nested_10_1(reader, max_bytes, table, depth + 1)?;
            PropertyValueObjectStream10_1::Object {
                class_id,
                key,
                body: Box::new(body),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: UID_CLASS_ID,
            key,
        } => PropertyValueObjectStream10_1::Uid(read_uid_body_10_1(reader, key)?),
        ObjectEntry10_1::NewObject {
            class_id: RASTER_SCALAR_CLASS_ID,
            key,
        } => {
            check_variant_object_depth(depth)?;
            let version = read_raster_argument_version(reader, "Scalar")?;
            let value =
                read_padded_variant_object_stream_nested_10_1(reader, max_bytes, table, depth + 1)?;
            PropertyValueObjectStream10_1::RasterScalar {
                key,
                version,
                value: Box::new(value),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id: RASTER_ARGUMENT_ARRAY_CLASS_ID,
            key,
        } => {
            check_variant_object_depth(depth)?;
            let mut body = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let version = read_raster_argument_version(&mut body, "ArgumentArray")?;
            let strings = [
                read_string_10_1(&mut body, max_bytes)?,
                read_string_10_1(&mut body, max_bytes)?,
            ];
            let declared_count = body.read_u32_le()? as i32;
            let count = declared_count.max(0) as usize;
            if count > body.remaining() / 16 {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "ArgumentArray 对象数超过剩余字节",
                ));
            }
            let mut objects = Vec::new();
            for _ in 0..count {
                let entry = read_object_entry_10_1(&mut body, table)?;
                objects.push(read_argument_array_item(
                    &mut body,
                    max_bytes,
                    table,
                    depth + 1,
                    entry,
                )?);
            }
            reader.read_bytes(body.position())?;
            PropertyValueObjectStream10_1::RasterArgumentArray(Box::new(RasterArgumentArray10_1 {
                key,
                version,
                strings,
                declared_count,
                objects,
            }))
        }
        ObjectEntry10_1::NewObject { class_id, key }
            if function_max_version(class_id).is_some()
                || supports_function_arguments(class_id) =>
        {
            read_variant_raster_input(reader, max_bytes, table, depth, class_id, key)?
        }
        ObjectEntry10_1::NewObject {
            class_id: class_id @ (RASTER_INFO_CLASS_ID | RASTER_INFOS_CLASS_ID),
            key,
        } => {
            check_variant_object_depth(depth)?;
            let mut wire = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body = table.with_raster_info_variant_depth(depth + 1, |table| {
                read_raster_info_variant_body_10_1(class_id, &mut wire, max_bytes, table)
            })?;
            let bytes = Arc::from(reader.read_bytes(wire.position())?);
            PropertyValueObjectStream10_1::RasterInfo(Box::new(EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (RASTER_STATISTICS
                | RASTER_HISTOGRAM
                | RASTER_COLORMAP
                | STATS_HISTOGRAM_CLASS_ID),
            key,
        } => {
            check_variant_object_depth(depth)?;
            let mut raster = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body =
                read_raster_variant_body_10_1(class_id, &mut raster, max_bytes, table, depth + 1)?;
            reader.read_bytes(raster.position())?;
            PropertyValueObjectStream10_1::Raster {
                class_id,
                key,
                body: Box::new(body),
            }
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (TIME_CLASS_ID
                | TIME_REFERENCE_CLASS_ID
                | TIME_EXTENT_CLASS_ID
                | TIME_WINDOW_CLASS_ID),
            key,
        } => {
            let mut wire = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body = match class_id {
                TIME_CLASS_ID => VariantTimeBody10_1::Time(read_time_body_10_1(
                    &mut wire, max_bytes, table, key,
                )?),
                TIME_REFERENCE_CLASS_ID => {
                    let body = read_time_reference_body_10_1(&mut wire, max_bytes, table, key)?;
                    if matches!(body.zone_info, TimeZoneInfoSlot10_1::Unsupported { .. }) {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "TimeReference 时区槽未闭合",
                        ));
                    }
                    VariantTimeBody10_1::Reference(body)
                }
                TIME_WINDOW_CLASS_ID => {
                    VariantTimeBody10_1::Window(read_time_window(&mut wire, max_bytes)?)
                }
                _ => {
                    let body = read_time_extent_body_10_1(&mut wire, max_bytes, table, key)?;
                    if body.stop.is_some() {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "TimeExtent 子槽未闭合",
                        ));
                    }
                    VariantTimeBody10_1::Extent(body)
                }
            };
            let bytes = Arc::from(reader.read_bytes(wire.position())?);
            PropertyValueObjectStream10_1::Time(Box::new(EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (POINT_CLASS_ID
                | MULTIPOINT_CLASS_ID
                | POLYLINE_CLASS_ID
                | POLYGON_CLASS_ID
                | ENVELOPE_CLASS_ID
                | GEOMETRY_BAG_CLASS_ID
                | MULTIPATCH_CLASS_ID),
            key,
        } => {
            let mut geometry = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body =
                read_keyed_geometry_body_10_1(&class_id, key, &mut geometry, max_bytes, table)?;
            let bytes = Arc::from(reader.read_bytes(geometry.position())?);
            PropertyValueObjectStream10_1::Geometry(Box::new(VariantGeometry10_1 {
                class_id,
                key,
                bytes,
                body,
            }))
        }
        ObjectEntry10_1::NewObject {
            class_id:
                class_id @ (RANDOM_COLOR_RAMP_CLASS_ID
                | MULTI_PART_COLOR_RAMP_CLASS_ID
                | PRESET_COLOR_RAMP_CLASS_ID
                | ALGORITHMIC_COLOR_RAMP_CLASS_ID),
            key,
        } => {
            check_variant_object_depth(depth)?;
            let mut wire = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body = read_color_ramp_body_for_class_10_1(
                &mut wire,
                max_bytes,
                table,
                (MAX_NESTED_VARIANT_OBJECTS - depth).min(16),
                class_id,
                key,
            )?
            .filter(|body| nested_ramp_stop(body).is_none())
            .ok_or_else(|| {
                io::Error::new(ErrorKind::Unsupported, "Variant 颜色梯度对象体未读完")
            })?;
            let bytes = Arc::from(reader.read_bytes(wire.position())?);
            PropertyValueObjectStream10_1::ColorRamp(Box::new(EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            }))
        }
        ObjectEntry10_1::NewObject { class_id, key } if is_graphic_attribute_object(class_id) => {
            check_variant_object_depth(depth)?;
            let mut wire = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            let body = read_graphic_attribute_list_10_1(&mut wire, max_bytes, table, depth + 1)?
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::Unsupported, "Variant 图形属性对象体未读完")
                })?;
            let bytes = Arc::from(reader.read_bytes(wire.position())?);
            PropertyValueObjectStream10_1::GraphicAttributes(Box::new(EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            }))
        }
        ObjectEntry10_1::NewObject { class_id, key } => {
            let mut wire = reader
                .clone()
                .read_region(max_bytes.min(reader.remaining()))?;
            if let Some(body) =
                read_color_body_for_class_10_1(class_id, &mut wire, max_bytes, table, key)?
            {
                let bytes = Arc::from(reader.read_bytes(wire.position())?);
                return Ok(PropertyValueObjectStream10_1::Color(Box::new(
                    EncodedVariantObject10_1 {
                        class_id,
                        key,
                        bytes,
                        body,
                    },
                )));
            }
            if let Some(body) = read_spatial_reference_body_10_1(&mut wire, max_bytes, &entry)? {
                let bytes = Arc::from(reader.read_bytes(wire.position())?);
                return Ok(PropertyValueObjectStream10_1::SpatialReference(Box::new(
                    EncodedVariantObject10_1 {
                        class_id,
                        key,
                        bytes,
                        body,
                    },
                )));
            }
            use VariantArrayObjectValues10_1::{DoubleBits, Longs, Strings};
            let (version, values) = match class_id {
                STR_ARRAY_CLASS_ID | VAR_STR_ARRAY_CLASS_ID => {
                    let start = reader.position();
                    let (version, values) = read_string_array_body_10_1(reader, max_bytes)?;
                    if class_id == STR_ARRAY_CLASS_ID {
                        table.mark_string_array(key, version, reader.position() - start, &values);
                    }
                    (version, Strings(values))
                }
                LONG_ARRAY_CLASS_ID => {
                    let (version, values) = read_long_array_body(reader, max_bytes)?;
                    table.mark_long_array(key, version, &values);
                    (version, Longs(values))
                }
                DOUBLE_ARRAY_CLASS_ID => {
                    let mut body = reader
                        .clone()
                        .read_region(reader.remaining().min(max_bytes))?;
                    let (version, values) = read_double_array_body(&mut body)?;
                    reader.read_bytes(body.position())?;
                    (
                        version,
                        DoubleBits(values.into_iter().map(f64::to_bits).collect()),
                    )
                }
                VAR_ARRAY_CLASS_ID | PROPERTY_SET_ARRAY_CLASS_ID => {
                    check_variant_object_depth(depth)?;
                    read_nested_array_body_10_1(reader, max_bytes, table, depth + 1, class_id)?
                }
                VALUE_ARRAY_CLASS_ID => {
                    // GdbCore 的具体 ValueArray 逐元素读写槽均返回 E_NOTIMPL。
                    let version = reader.read_u16_le()?;
                    if version > 1 {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            format!("未支持的 ValueArray 版本 {version}"),
                        ));
                    }
                    if reader.read_u32_le()? != 0 {
                        return Err(io::Error::new(
                            ErrorKind::Unsupported,
                            "GdbCore ValueArray 仅有空体的读写证据",
                        ));
                    }
                    (version, VariantArrayObjectValues10_1::Variants(Vec::new()))
                }
                _ => {
                    let mut wire = reader
                        .clone()
                        .read_region(reader.remaining().min(max_bytes))?;
                    if let Some(body) = read_number_format_body_for_class_10_1(
                        &mut wire, max_bytes, table, class_id, key,
                    )? {
                        let bytes = Arc::from(reader.read_bytes(wire.position())?);
                        return Ok(PropertyValueObjectStream10_1::NumberFormat(Box::new(
                            EncodedVariantObject10_1 {
                                class_id,
                                key,
                                bytes,
                                body,
                            },
                        )));
                    }
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        format!(
                            "Variant ObjectStream 类 GUID {class_id:02x?}、键 {key} 无已证对象体分派；边界由类决定"
                        ),
                    ));
                }
            };
            PropertyValueObjectStream10_1::ArrayObject {
                class_id,
                key,
                version,
                values,
            }
        }
    })
}

#[inline(never)]
fn read_variant_raster_renderer(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    check_variant_object_depth(depth)?;
    let mut wire = reader
        .clone()
        .read_region(max_bytes.min(reader.remaining()))?;
    let body = table.with_raster_info_variant_depth(depth + 1, |table| {
        crate::layers::raster_renderer::read_raster_renderer_body_10_1(
            &mut wire, max_bytes, table, class_id,
        )
    })?;
    if !body.is_complete() {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            "Variant 栅格渲染器对象体未闭合",
        ));
    }
    let bytes = Arc::from(reader.read_bytes(wire.position())?);
    Ok(PropertyValueObjectStream10_1::RasterRenderer(Box::new(
        EncodedVariantObject10_1 {
            class_id,
            key,
            bytes,
            body,
        },
    )))
}

fn read_variant_raster_input(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    check_variant_object_depth(depth)?;
    let mut wire = reader
        .clone()
        .read_region(max_bytes.min(reader.remaining()))?;
    let body = table.with_raster_info_variant_depth(depth + 1, |refs| {
        read_input_body(
            &mut wire,
            max_bytes,
            refs,
            class_id,
            key,
            (MAX_NESTED_VARIANT_OBJECTS - depth).min(16),
        )
    })?;
    let bytes = Arc::from(reader.read_bytes(wire.position())?);
    Ok(PropertyValueObjectStream10_1::RasterArgumentInput(
        Box::new(EncodedVariantObject10_1 {
            class_id,
            key,
            bytes,
            body,
        }),
    ))
}

#[inline(never)]
fn read_variant_raster_unique_colors(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    let body = crate::layers::raster_unique_value::read_legacy_colors_body(&mut wire, max_bytes)?;
    let bytes = Arc::from(reader.read_bytes(wire.position())?);
    Ok(PropertyValueObjectStream10_1::RasterUniqueValueColors(
        Box::new(EncodedVariantObject10_1 {
            class_id,
            key,
            bytes,
            body,
        }),
    ))
}

#[inline(never)]
fn read_variant_unique_values(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    check_variant_object_depth(depth)?;
    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    let body = crate::layers::raster_unique_value::read_unique_values_body(
        &mut wire,
        max_bytes,
        table,
        depth + 1,
    )?;
    let bytes = Arc::from(reader.read_bytes(wire.position())?);
    Ok(PropertyValueObjectStream10_1::UniqueValues(Box::new(
        EncodedVariantObject10_1 {
            class_id,
            key,
            bytes,
            body,
        },
    )))
}

#[inline(never)]
fn read_variant_filter_definition(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    use crate::objects::query_filter::{
        FILTER_DEFS_CLASS_ID, read_filter_defs_body_10_1, read_xml_filter_def_body_10_1,
    };

    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    if class_id == FILTER_DEFS_CLASS_ID {
        let mut stop = None;
        let body = read_filter_defs_body_10_1(&mut wire, max_bytes, table, &mut stop)?;
        if stop.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Variant 过滤定义集合含未支持对象",
            ));
        }
        let bytes = Arc::from(reader.read_bytes(wire.position())?);
        Ok(PropertyValueObjectStream10_1::FilterDefinitions(Box::new(
            EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            },
        )))
    } else {
        let body = read_xml_filter_def_body_10_1(&mut wire, max_bytes)?;
        let bytes = Arc::from(reader.read_bytes(wire.position())?);
        Ok(PropertyValueObjectStream10_1::XmlFilterDef(Box::new(
            EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            },
        )))
    }
}

#[inline(never)]
fn read_variant_time_layer_track(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    let body = crate::layers::time_layer_track_extension::read_time_layer_track_body_10_1(
        &mut wire, max_bytes, table,
    )?;
    let bytes = Arc::from(reader.read_bytes(wire.position())?);
    Ok(PropertyValueObjectStream10_1::TimeLayerTrack(Box::new(
        EncodedVariantObject10_1 {
            class_id,
            key,
            bytes,
            body,
        },
    )))
}

#[inline(never)]
fn read_variant_query_filter(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    let mut stop = None;
    let body = crate::objects::query_filter::read_query_filter_body_10_1(
        &mut wire, max_bytes, table, class_id, &mut stop,
    )?;
    let body = body.filter(|_| stop.is_none()).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Unsupported, "Variant 查询过滤器含未支持对象")
    })?;
    let bytes = Arc::from(reader.read_bytes(wire.position())?);
    Ok(PropertyValueObjectStream10_1::QueryFilter(Box::new(
        EncodedVariantObject10_1 {
            class_id,
            key,
            bytes,
            body,
        },
    )))
}

// 定义体只在对应分支分配栈空间，避免扩大共用递归分派的栈帧。
#[inline(never)]
fn read_variant_field_definition(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    use crate::objects::field_defs::{GEOMETRY_DEF_CLASS_ID, read_geometry_def, read_raster_def};

    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    if class_id == GEOMETRY_DEF_CLASS_ID {
        let body = read_geometry_def(&mut wire, max_bytes, table)?;
        let bytes = Arc::from(reader.read_bytes(wire.position())?);
        Ok(PropertyValueObjectStream10_1::GeometryDef(Box::new(
            EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            },
        )))
    } else {
        let body = read_raster_def(&mut wire, max_bytes, table)?;
        let bytes = Arc::from(reader.read_bytes(wire.position())?);
        Ok(PropertyValueObjectStream10_1::RasterDef(Box::new(
            EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            },
        )))
    }
}

#[inline(never)]
fn read_variant_field_object(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
    key: u32,
) -> io::Result<PropertyValueObjectStream10_1> {
    use crate::objects::fields::{
        FIELD_CLASS_ID, read_field_body_10_1, read_fields_body_nested_10_1,
    };

    check_variant_object_depth(depth)?;
    let mut wire = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    if class_id == FIELD_CLASS_ID {
        let body = read_field_body_10_1(&mut wire, max_bytes, table, depth + 1)?;
        let bytes = Arc::from(reader.read_bytes(wire.position())?);
        Ok(PropertyValueObjectStream10_1::Field(Box::new(
            EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            },
        )))
    } else {
        let body = read_fields_body_nested_10_1(&mut wire, max_bytes, table, depth + 1)?;
        let bytes = Arc::from(reader.read_bytes(wire.position())?);
        Ok(PropertyValueObjectStream10_1::Fields(Box::new(
            EncodedVariantObject10_1 {
                class_id,
                key,
                bytes,
                body,
            },
        )))
    }
}

fn read_argument_array_item(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    entry: ObjectEntry10_1,
) -> io::Result<PropertyValueObjectStream10_1> {
    let ObjectEntry10_1::NewObject { class_id, key } = entry else {
        return read_variant_object_body_10_1(reader, max_bytes, table, depth, entry);
    };
    let mut trial = reader.clone();
    let mut staged = table.clone();
    let value =
        match read_variant_object_body_10_1(&mut trial, max_bytes, &mut staged, depth, entry) {
            Ok(value) => value,
            Err(error) if error.kind() == ErrorKind::Unsupported => {
                trial = reader.clone();
                staged = table.clone();
                read_variant_raster_input(&mut trial, max_bytes, &mut staged, depth, class_id, key)?
            }
            Err(error) => return Err(error),
        };
    *reader = trial;
    *table = staged;
    Ok(value)
}

fn read_raster_argument_version(reader: &mut Reader<'_>, name: &str) -> io::Result<u16> {
    let version = reader.read_u16_le()?;
    if version as i16 > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("{name} 未支持版本 {version}"),
        ));
    }
    Ok(version)
}

fn check_variant_object_depth(depth: usize) -> io::Result<()> {
    if depth >= MAX_NESTED_VARIANT_OBJECTS {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Variant ObjectStream 对象嵌套深度超过 {MAX_NESTED_VARIANT_OBJECTS}"),
        ));
    }
    Ok(())
}

fn read_nested_array_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    depth: usize,
    class_id: [u8; 16],
) -> io::Result<(u16, VariantArrayObjectValues10_1)> {
    let mut body = reader
        .clone()
        .read_region(reader.remaining().min(max_bytes))?;
    let version = body.read_u16_le()?;
    if version > 1 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("未支持的 Variant 数组对象版本 {version}"),
        ));
    }
    let count = (body.read_u32_le()? as i32).max(0) as usize;
    let min_item_bytes = if class_id == VAR_ARRAY_CLASS_ID {
        16
    } else {
        20
    };
    if count > body.remaining() / min_item_bytes {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Variant 数组对象计数超出可读范围",
        ));
    }
    let values = if class_id == VAR_ARRAY_CLASS_ID {
        // VarArray 的逐项委派固定调用 Variant::Read(false)。
        let values = (0..count)
            .map(|_| {
                read_variant_object_stream_layout_10_1(&mut body, max_bytes, table, depth, true)
            })
            .collect::<io::Result<_>>()?;
        VariantArrayObjectValues10_1::Variants(values)
    } else {
        let mut values = Vec::new();
        for _ in 0..count {
            let entry = read_object_entry_10_1(&mut body, table)?;
            let item = match entry {
                ObjectEntry10_1::NewObject {
                    class_id: PROPERTY_SET_CLASS_ID,
                    ..
                } => {
                    check_variant_object_depth(depth)?;
                    Some(read_property_set_object_stream_nested_10_1(
                        &mut body,
                        max_bytes,
                        table,
                        depth + 1,
                    )?)
                }
                ObjectEntry10_1::BackReference {
                    class_id: PROPERTY_SET_CLASS_ID,
                    ..
                } => None,
                _ => {
                    return Err(io::Error::new(
                        ErrorKind::Unsupported,
                        "PropertySetArray 元素必须是可读取的非空 PropertySet",
                    ));
                }
            };
            values.push(PropertySetArrayEntry10_1 { entry, body: item });
        }
        VariantArrayObjectValues10_1::PropertySets(values)
    };
    reader.read_bytes(body.position())?;
    Ok((version, values))
}

/// 读取一个 `Variant::Read`（AfCore `0x68a7c6a0`）值，失败时
/// 不推进游标。
///
/// VARTYPE 表见模块文档；对象分支（9/13）接入「类 GUID 全零 → 空对象」
/// 已闭合子集（非零类 GUID 仍显式拒绝），支持定宽与 BSTR 数组；
/// 未知 VARTYPE 与原生 default 分支一致返回错误。标量字符串为 AfCore String
/// 布局（`String::Read`，4 字节 cb 含 UTF-16LE 终止零）。
pub fn read_variant_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<PropertyValue10_1> {
    read_variant_layout_10_1(reader, max_bytes, false)
}

fn read_variant_layout_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    padded: bool,
) -> io::Result<PropertyValue10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Variant 值 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let value_type = trial.read_u16_le()?;
    if padded {
        // 0x68a7c6fd：Read(false) 从 VARTYPE 之后跳过十四字节。
        trial.read_bytes(14)?;
    }
    if value_type & 0x2000 != 0 {
        let array = read_class_variant_array_10_1(&mut trial, value_type & !0x2000, max_bytes)?;
        *reader = trial;
        return Ok(PropertyValue10_1::Array(array));
    }
    let value = match value_type {
        0 => PropertyValue10_1::Empty,
        1 => PropertyValue10_1::Null,
        2 | 11 => PropertyValue10_1::Raw2(trial.read_bytes(2)?.try_into().unwrap()),
        3 | 10 | 4 => PropertyValue10_1::Raw4(trial.read_bytes(4)?.try_into().unwrap()),
        5 | 7 | 6 => PropertyValue10_1::Raw8(trial.read_bytes(8)?.try_into().unwrap()),
        8 => PropertyValue10_1::String(read_string_10_1(&mut trial, max_bytes)?),
        9 | 13 => {
            if padded {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "带保留区的 Variant 对象值尚未支持",
                ));
            }
            // 该公开入口保留普通 IStream 的上下文语义，不推断其实现 IObjectStream。
            let mut branch = reader.clone();
            let value =
                match crate::objects::variant_object_branch::read_variant_object_branch_10_1(
                    &mut branch,
                )? {
                    crate::objects::variant_object_branch::VariantObjectBranch10_1::Null => {
                        PropertyValue10_1::NullObject
                    }
                };
            *reader = branch;
            return Ok(value);
        }
        0x10 | 0x11 => PropertyValue10_1::Raw1(trial.read_bytes(1)?.try_into().unwrap()),
        0x12 => PropertyValue10_1::Ui2(trial.read_bytes(2)?.try_into().unwrap()),
        0x13 => PropertyValue10_1::Ui4(trial.read_bytes(4)?.try_into().unwrap()),
        _ => {
            return Err(invalid(format!(
                "VARTYPE {value_type} 超出 Variant::Read 已确认分支（原生 default 返回 \
                 0x80004005）"
            )));
        }
    };
    *reader = trial;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(bytes: &mut Vec<u8>, class_id: [u8; 16], key: u32) {
        bytes.extend(class_id);
        bytes.extend(key.to_le_bytes());
    }

    #[test]
    fn shared_variant_object_dispatch_covers_domain_and_raster_value_classes() {
        use crate::layers::dep_tail::{
            TIME_CLASS_ID, TIME_EXTENT_CLASS_ID, TIME_REFERENCE_CLASS_ID,
        };
        use crate::layers::feature_id_set::{FEATURE_ID_SET_CLASS_ID, FID_SET_CLASS_ID};
        use crate::layers::raster_classify_renderer::RASTER_CLASSIFY_RENDERER_CLASS_ID;
        use crate::layers::raster_dataset_name::LONG_ARRAY_CLASS_ID;
        use crate::layers::raster_renderer::{
            DOUBLE_ARRAY_CLASS_ID, RASTER_COLORMAP_RENDERER_CLASS_ID, RASTER_RGB_RENDERER_CLASS_ID,
            SIMPLE_RASTER_RENDERER_CLASS_ID, STATS_HISTOGRAM_CLASS_ID,
        };
        use crate::layers::raster_stretch_renderer::RASTER_STRETCH_RENDERER_CLASS_ID;
        use crate::layers::raster_unique_value::{
            RASTER_UNIQUE_VALUE_COLORS_CLASS_ID, RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID,
            UNIQUE_VALUES_CLASS_ID,
        };
        use crate::layers::time_layer_track_extension::TIME_WINDOW_CLASS_ID;
        use crate::objects::display_color::{
            CMYK_COLOR_CLASS_ID, GRAY_COLOR_CLASS_ID, HLS_COLOR_CLASS_ID, HSV_COLOR_CLASS_ID,
            RGB_COLOR_CLASS_ID,
        };
        use crate::objects::domain::{
            BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID, CODED_VALUE_DOMAIN_CLASS_ID,
            RANGE_DOMAIN_CLASS_ID,
        };
        use crate::objects::envelope::ENVELOPE_CLASS_ID;
        use crate::objects::field_defs::{GEOMETRY_DEF_CLASS_ID, RASTER_DEF_CLASS_ID};
        use crate::objects::fields::{FIELD_CLASS_ID, FIELDS_CLASS_ID};
        use crate::objects::geodata_xform::{
            ADJUST_XFORM_CLASS_ID, COMPOSITE_XFORM_CLASS_ID, COORDINATE_XFORM_CLASS_ID,
            CSM_XFORM_CLASS_ID, FORMOSAT2_XFORM_CLASS_ID, FRAME_XFORM_CLASS_ID,
            GCS_SHIFT_XFORM_CLASS_ID, GEOMETRIC_XFORM_CLASS_ID, IDENTITY_XFORM_CLASS_ID,
            LSR_XFORM_CLASS_ID, PICTOMETRY_XFORM_CLASS_ID, POLYNOMIAL_XFORM_CLASS_ID,
            PROJECTIVE_XFORM_CLASS_ID, RPC_XFORM_CLASS_ID, SPLINE_XFORM_CLASS_ID,
            SPOT5_XFORM_CLASS_ID,
        };
        use crate::objects::geometry::{
            GEOMETRY_BAG_CLASS_ID, MULTIPATCH_CLASS_ID, MULTIPOINT_CLASS_ID, POINT_CLASS_ID,
            POLYGON_CLASS_ID, POLYLINE_CLASS_ID,
        };
        use crate::objects::number_format::{
            ANGLE_FORMAT_CLASS_ID, CURRENCY_FORMAT_CLASS_ID, CUSTOM_NUMBER_FORMAT_CLASS_ID,
            FRACTION_FORMAT_CLASS_ID, LATLON_FORMAT_CLASS_ID, NUMERIC_FORMAT_CLASS_ID,
            PERCENTAGE_FORMAT_CLASS_ID, RATE_FORMAT_CLASS_ID, SCIENTIFIC_FORMAT_CLASS_ID,
        };
        use crate::objects::query_filter::{
            FILTER_DEFS_CLASS_ID, QUERY_FILTER_CLASS_ID, SPATIAL_FILTER_CLASS_ID,
            XML_FILTER_DEF_CLASS_ID,
        };
        use crate::objects::spatial_reference::PROJECTED_COORDINATE_SYSTEM_CLASS_ID;
        use crate::objects::spatial_reference_slots::{
            GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID, UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
        };

        let classes = [
            RASTER_UNIQUE_VALUE_COLORS_CLASS_ID,
            UNIQUE_VALUES_CLASS_ID,
            RANGE_DOMAIN_CLASS_ID,
            CODED_VALUE_DOMAIN_CLASS_ID,
            BIT_MASK_CODED_VALUE_DOMAIN_CLASS_ID,
            FIELD_CLASS_ID,
            FIELDS_CLASS_ID,
            GEOMETRY_DEF_CLASS_ID,
            RASTER_DEF_CLASS_ID,
            FEATURE_ID_SET_CLASS_ID,
            FID_SET_CLASS_ID,
            QUERY_FILTER_CLASS_ID,
            SPATIAL_FILTER_CLASS_ID,
            FILTER_DEFS_CLASS_ID,
            XML_FILTER_DEF_CLASS_ID,
            PROPERTY_SET_CLASS_ID,
            UID_CLASS_ID,
            RASTER_SCALAR_CLASS_ID,
            RASTER_ARGUMENT_ARRAY_CLASS_ID,
            crate::layers::image_server_rules::RASTER_INFO_CLASS_ID,
            crate::layers::image_server_rules::RASTER_INFOS_CLASS_ID,
            crate::layers::image_server_rules::RASTER_STATISTICS,
            crate::layers::image_server_rules::RASTER_HISTOGRAM,
            crate::layers::image_server_rules::RASTER_COLORMAP,
            STATS_HISTOGRAM_CLASS_ID,
            TIME_CLASS_ID,
            TIME_REFERENCE_CLASS_ID,
            TIME_EXTENT_CLASS_ID,
            TIME_WINDOW_CLASS_ID,
            POINT_CLASS_ID,
            MULTIPOINT_CLASS_ID,
            POLYLINE_CLASS_ID,
            POLYGON_CLASS_ID,
            ENVELOPE_CLASS_ID,
            GEOMETRY_BAG_CLASS_ID,
            MULTIPATCH_CLASS_ID,
            crate::objects::color_ramp::RANDOM_COLOR_RAMP_CLASS_ID,
            crate::objects::color_ramp::MULTI_PART_COLOR_RAMP_CLASS_ID,
            crate::objects::color_ramp::PRESET_COLOR_RAMP_CLASS_ID,
            crate::objects::color_ramp::ALGORITHMIC_COLOR_RAMP_CLASS_ID,
            STR_ARRAY_CLASS_ID,
            LONG_ARRAY_CLASS_ID,
            DOUBLE_ARRAY_CLASS_ID,
            VAR_ARRAY_CLASS_ID,
            VAR_STR_ARRAY_CLASS_ID,
            PROPERTY_SET_ARRAY_CLASS_ID,
            VALUE_ARRAY_CLASS_ID,
            NUMERIC_FORMAT_CLASS_ID,
            PERCENTAGE_FORMAT_CLASS_ID,
            FRACTION_FORMAT_CLASS_ID,
            RATE_FORMAT_CLASS_ID,
            CURRENCY_FORMAT_CLASS_ID,
            LATLON_FORMAT_CLASS_ID,
            ANGLE_FORMAT_CLASS_ID,
            SCIENTIFIC_FORMAT_CLASS_ID,
            CUSTOM_NUMBER_FORMAT_CLASS_ID,
            RGB_COLOR_CLASS_ID,
            CMYK_COLOR_CLASS_ID,
            HSV_COLOR_CLASS_ID,
            GRAY_COLOR_CLASS_ID,
            HLS_COLOR_CLASS_ID,
            PROJECTED_COORDINATE_SYSTEM_CLASS_ID,
            GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID,
            UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
            IDENTITY_XFORM_CLASS_ID,
            POLYNOMIAL_XFORM_CLASS_ID,
            COMPOSITE_XFORM_CLASS_ID,
            ADJUST_XFORM_CLASS_ID,
            crate::objects::geodata_xform::APPROXIMATION_XFORM_CLASS_ID,
            GCS_SHIFT_XFORM_CLASS_ID,
            COORDINATE_XFORM_CLASS_ID,
            RPC_XFORM_CLASS_ID,
            GEOMETRIC_XFORM_CLASS_ID,
            LSR_XFORM_CLASS_ID,
            PICTOMETRY_XFORM_CLASS_ID,
            SPLINE_XFORM_CLASS_ID,
            FRAME_XFORM_CLASS_ID,
            FORMOSAT2_XFORM_CLASS_ID,
            SPOT5_XFORM_CLASS_ID,
            CSM_XFORM_CLASS_ID,
            PROJECTIVE_XFORM_CLASS_ID,
            RASTER_COLORMAP_RENDERER_CLASS_ID,
            RASTER_RGB_RENDERER_CLASS_ID,
            SIMPLE_RASTER_RENDERER_CLASS_ID,
            RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID,
            RASTER_CLASSIFY_RENDERER_CLASS_ID,
            RASTER_STRETCH_RENDERER_CLASS_ID,
            crate::layers::raster_renderer::DISCRETE_COLOR_RENDERER_CLASS_ID,
            crate::layers::raster_renderer::SAI_RENDERER_CLASS_ID,
        ];

        for class_id in classes {
            let entry = ObjectEntry10_1::NewObject { class_id, key: 1 };
            let result = read_variant_object_body_10_1(
                &mut Reader::new(&[]),
                64,
                &mut RefTable10_1::new(),
                0,
                entry,
            );
            if let Err(error) = result {
                assert!(
                    !error.to_string().contains("无已证对象体分派"),
                    "Variant 类 {} 未进入专属分派：{error}",
                    crate::storage::class_id_hex(&class_id),
                );
            }
        }
    }

    #[test]
    fn variant_reads_raster_function_and_limits_recursive_parameters() {
        use crate::layers::image_server_rules::{
            CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID, IDENTITY_FUNCTION_CLASS_ID,
        };

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, IDENTITY_FUNCTION_CLASS_ID, 42);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(7_i32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let PropertyValueObjectStream10_1::RasterArgumentInput(value) =
            read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut table).unwrap()
        else {
            panic!("应读取 Variant 中的栅格函数");
        };
        assert!(
            matches!(value.body(), RasterTemplateInput10_1::Function(body) if body.base_long == 7)
        );
        assert!(table.contains(42));
        assert_eq!(reader.remaining(), 0);

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        for key in 0..=MAX_NESTED_VARIANT_OBJECTS as u32 {
            object(
                &mut bytes,
                CACHED_RASTER_FUNCTION_ARGUMENTS_CLASS_ID,
                key * 2,
            );
            bytes.extend(1_u16.to_le_bytes());
            object(&mut bytes, PROPERTY_SET_CLASS_ID, key * 2 + 1);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend([2, 0, 0, 0, 0, 0]); // 空属性名。
            bytes.extend(13_u16.to_le_bytes()); // 紧凑 Variant 对象值。
        }
        bytes.extend([0; 16]);
        let mut reader = Reader::new(&bytes);
        let mut table = RefTable10_1::new();
        let error =
            read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut table).unwrap_err();
        assert!(error.to_string().contains("嵌套深度"));
        assert_eq!(reader.position(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn variant_number_format_preserves_body_and_session_key() {
        use crate::objects::number_format::NUMERIC_FORMAT_CLASS_ID;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, NUMERIC_FORMAT_CLASS_ID, 12);
        let header_end = bytes.len();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 22]);
        let body_end = bytes.len();
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, NUMERIC_FORMAT_CLASS_ID, 12);

        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        let parsed = read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut table).unwrap();
        let PropertyValueObjectStream10_1::NumberFormat(value) = parsed else {
            panic!("应读取数字格式对象");
        };
        assert_eq!(&*value.bytes, &bytes[header_end..body_end]);
        assert!(matches!(value.body, NumberFormatSlot10_1::NumericFormat(_)));
        assert_eq!(reader.position(), body_end);
        assert_eq!(
            read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut table).unwrap(),
            PropertyValueObjectStream10_1::BackReference {
                class_id: NUMERIC_FORMAT_CLASS_ID,
                key: 12,
            }
        );
        assert_eq!(reader.remaining(), 0);

        for end in header_end..body_end {
            let mut reader = Reader::new(&bytes[..end]);
            let mut table = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut table).is_err());
            assert_eq!(reader.position(), 0);
            assert!(!table.contains(12));
        }
    }

    fn variant_array(bytes: &mut Vec<u8>, key: u32, count: u32) {
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(bytes, VAR_ARRAY_CLASS_ID, key);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(count.to_le_bytes());
    }

    // RasterInfo v1 的首批对象槽及固定标量，直到带保留区的 Variant。
    fn raster_info_prefix(bytes: &mut Vec<u8>) {
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 4 * 16 + 2 * 8 + 2 * 4]);
    }

    #[test]
    fn property_set_raster_infos_mixes_new_members_and_backreferences() {
        const EMPTY_STRING: [u8; 6] = [2, 0, 0, 0, 0, 0];
        let mut bytes = 1_u16.to_le_bytes().to_vec();
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(EMPTY_STRING);
        bytes.extend(13_u16.to_le_bytes());
        object(&mut bytes, RASTER_ARGUMENT_ARRAY_CLASS_ID, 10);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(EMPTY_STRING.repeat(2));
        bytes.extend(2_i32.to_le_bytes());
        object(&mut bytes, RASTER_INFOS_CLASS_ID, 11);
        let collection_start = bytes.len();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        object(&mut bytes, RASTER_INFO_CLASS_ID, 12);
        raster_info_prefix(&mut bytes);
        bytes.extend([0; 16 + 4 * 4]); // 空 Variant 及后继 LONG。
        bytes.extend(7_u16.to_le_bytes());
        object(&mut bytes, RASTER_INFO_CLASS_ID, 12);
        let collection_end = bytes.len();
        object(&mut bytes, RASTER_INFO_CLASS_ID, 13);
        raster_info_prefix(&mut bytes);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, RASTER_INFOS_CLASS_ID, 11);
        bytes.extend([0; 4 * 4 + 2]);
        bytes.extend(EMPTY_STRING);
        bytes.extend(9_u16.to_le_bytes());
        object(&mut bytes, RASTER_INFO_CLASS_ID, 13);
        let end = bytes.len();
        bytes.push(0xa5);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_property_set_object_stream_10_1(&mut reader, 4096, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::RasterArgumentArray(array) = &parsed.entries[0].value
        else {
            panic!("应读取参数数组");
        };
        let PropertyValueObjectStream10_1::RasterInfo(collection) = &array.objects[0] else {
            panic!("应读取栅格信息集合");
        };
        let RasterInfoVariantBody10_1::Infos(infos) = collection.body() else {
            panic!("应读取 RasterInfos");
        };
        assert_eq!(collection.bytes(), &bytes[collection_start..collection_end]);
        assert_eq!(infos.entries[0].body.as_ref().unwrap().flags, 7);
        assert!(matches!(
            infos.entries[1].entry,
            ObjectEntry10_1::BackReference { key: 12, .. }
        ));
        let PropertyValueObjectStream10_1::RasterInfo(second) = &array.objects[1] else {
            panic!("应读取独立栅格信息");
        };
        let RasterInfoVariantBody10_1::Info(info) = second.body() else {
            panic!("应读取 RasterInfo");
        };
        assert_eq!(
            info.value,
            PropertyValueObjectStream10_1::BackReference {
                class_id: RASTER_INFOS_CLASS_ID,
                key: 11
            }
        );
        assert_eq!(
            parsed.entries[1].value,
            PropertyValueObjectStream10_1::BackReference {
                class_id: RASTER_INFO_CLASS_ID,
                key: 13
            }
        );
        assert_eq!(reader.position(), end);
        assert!((10..=13).all(|key| refs.contains(key)));

        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            let mut refs = RefTable10_1::new();
            assert!(read_property_set_object_stream_10_1(&mut reader, 4096, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn raster_info_reentry_inherits_variant_depth_and_preserves_rollback() {
        let mut bytes = Vec::new();
        for key in 0..(MAX_NESTED_VARIANT_OBJECTS - 1) as u32 {
            variant_array(&mut bytes, key, 1);
        }
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, RASTER_INFO_CLASS_ID, 100);
        raster_info_prefix(&mut bytes);
        let value_offset = bytes.len();
        variant_array(&mut bytes, 101, 0);
        bytes.extend([0; 4 * 4 + 2]);

        let mut refs = RefTable10_1::new();
        refs.register(102);
        let original = refs.clone();
        let mut reader = Reader::new(&bytes);
        let error =
            read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).unwrap_err();
        assert!(error.to_string().contains("嵌套深度"));
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);

        bytes.truncate(value_offset);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, VAR_ARRAY_CLASS_ID, 0);
        bytes.extend([0; 4 * 4 + 2]);
        let mut reader = Reader::new(&bytes);
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).is_ok());
        assert_eq!(reader.remaining(), 0);
        assert!(refs.contains(100));
        assert!(!refs.contains(101));
    }

    #[test]
    fn variant_unique_renderer_propagates_depth_and_child_keys() {
        use crate::layers::raster_renderer::RasterRendererPrefix10_1;
        use crate::layers::raster_unique_value::RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID;
        use crate::objects::geodata_xform::IDENTITY_XFORM_CLASS_ID;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, RASTER_UNIQUE_VALUE_RENDERER_CLASS_ID, 11);
        bytes.extend(3_i32.to_le_bytes());
        for _ in 0..3 {
            bytes.extend([2, 0, 0, 0, 0, 0]);
        }
        bytes.extend([0; 2 + 16]); // short 及空符号。
        bytes.extend([2, 0, 0, 0, 0, 0]); // 空标签。
        bytes.extend(0_i32.to_le_bytes()); // 空图例组。
        for field in [1_i32, 77, 2] {
            bytes.extend(field.to_le_bytes());
        }
        for new in [true, false] {
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0; 14]);
            object(&mut bytes, IDENTITY_XFORM_CLASS_ID, 12);
            if new {
                bytes.extend(1_i32.to_le_bytes());
            }
        }
        bytes.extend([0; 2 + 16]); // short 及空 IUniqueValues。
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_nested_10_1(
            &mut reader,
            4096,
            &mut refs,
            MAX_NESTED_VARIANT_OBJECTS - 2,
        )
        .unwrap();
        let PropertyValueObjectStream10_1::RasterRenderer(renderer) = value else {
            panic!("缺少栅格渲染器");
        };
        let RasterRendererPrefix10_1::UniqueValue(body) = renderer.body() else {
            panic!("缺少唯一值渲染器");
        };
        assert!(matches!(
            &body.classes[0].values[..],
            [PropertyValueObjectStream10_1::GeodataXform(value),
             PropertyValueObjectStream10_1::BackReference {
                class_id: IDENTITY_XFORM_CLASS_ID, key: 12
             }] if value.key == 12
        ));
        assert_eq!(reader.position(), end);
        assert_eq!(refs.raster_info_variant_depth(), 0);
        assert!(refs.contains(11) && refs.contains(12));

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error = read_padded_variant_object_stream_nested_10_1(
            &mut reader,
            4096,
            &mut refs,
            MAX_NESTED_VARIANT_OBJECTS - 1,
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("深度"));
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_raster_renderer_rejects_partial_colormap_bodies() {
        use crate::layers::raster_renderer::{
            RASTER_COLORMAP_RENDERER_CLASS_ID, RasterRendererPrefix10_1,
        };

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, RASTER_COLORMAP_RENDERER_CLASS_ID, 11);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 8]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        bytes.extend(29_i32.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        assert!(matches!(
            value,
            PropertyValueObjectStream10_1::RasterRenderer(value)
                if matches!(value.body(), RasterRendererPrefix10_1::Colormap { tail: Some(tail), .. }
                    if tail.colormap_values == [29])
        ));
        assert_eq!(reader.position(), end);
        assert_eq!(
            refs.registered_class(11),
            Some(RASTER_COLORMAP_RENDERER_CLASS_ID)
        );

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_adjust_transform_reuses_geodata_dispatch() {
        use crate::objects::geodata_xform::{
            ADJUST_XFORM_CLASS_ID, GeodataXformBody10_1, IDENTITY_XFORM_CLASS_ID,
        };

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, ADJUST_XFORM_CLASS_ID, 11);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(7_u16.to_le_bytes());
        bytes.extend(23_i32.to_le_bytes());
        bytes.extend([0; 16]);
        object(&mut bytes, IDENTITY_XFORM_CLASS_ID, 12);
        bytes.extend(1_i32.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        assert!(matches!(
            value,
            PropertyValueObjectStream10_1::GeodataXform(value) if matches!(value.body(),
                GeodataXformBody10_1::Adjust { short_c8: 7, long_a8: 23, inner, .. }
                    if inner.body.is_some())
        ));
        assert_eq!(reader.position(), end);
        assert_eq!(refs.registered_class(12), Some(IDENTITY_XFORM_CLASS_ID));

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_composite_transform_shares_child_keys() {
        use crate::objects::geodata_xform::{
            COMPOSITE_XFORM_CLASS_ID, GeodataXformBody10_1, IDENTITY_XFORM_CLASS_ID,
        };

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, COMPOSITE_XFORM_CLASS_ID, 11);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(2_i32.to_le_bytes());
        object(&mut bytes, IDENTITY_XFORM_CLASS_ID, 12);
        bytes.extend(1_i32.to_le_bytes());
        object(&mut bytes, IDENTITY_XFORM_CLASS_ID, 12);
        bytes.extend(7_u16.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::GeodataXform(value) = value else {
            panic!("缺少地理变换对象");
        };
        let GeodataXformBody10_1::Composite {
            children, short_34, ..
        } = value.body()
        else {
            panic!("缺少组合变换");
        };
        assert!(children[0].body.is_some());
        assert_eq!(
            children[1].entry,
            ObjectEntry10_1::BackReference {
                class_id: IDENTITY_XFORM_CLASS_ID,
                key: 12
            }
        );
        assert_eq!(*short_34, Some(7));
        assert_eq!(reader.position(), end);
        assert!(refs.contains(11) && refs.contains(12));

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_polynomial_transform_reads_coefficients_before_next_field() {
        use crate::objects::geodata_xform::{GeodataXformBody10_1, POLYNOMIAL_XFORM_CLASS_ID};

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, POLYNOMIAL_XFORM_CLASS_ID, 11);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend([0; 16]);
        for field in [7_i32, 1, 0] {
            bytes.extend(field.to_le_bytes());
        }
        bytes.extend([0; 16]);
        for flag in [1_i32, 0, 0] {
            bytes.extend(flag.to_le_bytes());
        }
        bytes.extend(2.5_f64.to_le_bytes());
        bytes.extend(3.5_f64.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::GeodataXform(value) = value else {
            panic!("缺少地理变换对象");
        };
        let GeodataXformBody10_1::Polynomial(body) = value.body() else {
            panic!("缺少多项式变换");
        };
        assert_eq!(
            body.coefficients_e4.as_ref().unwrap(),
            &[vec![2.5], vec![3.5]]
        );
        assert_eq!(reader.position(), end);
        assert_eq!(refs.registered_class(11), Some(POLYNOMIAL_XFORM_CLASS_ID));

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_identity_transform_keeps_its_body_boundary() {
        use crate::objects::geodata_xform::{GeodataXformBody10_1, IDENTITY_XFORM_CLASS_ID};

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, IDENTITY_XFORM_CLASS_ID, 11);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend([0; 16]); // 空空间参考。
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        assert!(matches!(
            value,
            PropertyValueObjectStream10_1::GeodataXform(value)
                if value.key == 11 && matches!(value.body(),
                    GeodataXformBody10_1::Identity { version: 2, reference: Some(_) })
        ));
        assert_eq!(reader.position(), end);
        assert_eq!(refs.registered_class(11), Some(IDENTITY_XFORM_CLASS_ID));

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_stats_histogram_legacy_versions_end_after_fixed_buckets() {
        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, STATS_HISTOGRAM_CLASS_ID, 11);
        let version_offset = bytes.len();
        bytes.extend([0; 4 + 48 + 256 * 8]);
        let end = bytes.len();
        bytes.push(0xaa);

        for version in [i32::MIN, -1, 0, 1] {
            bytes[version_offset..version_offset + 4].copy_from_slice(&version.to_le_bytes());
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value =
                read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut refs).unwrap();
            assert!(matches!(
                value,
                PropertyValueObjectStream10_1::Raster { body, .. }
                    if matches!(body.as_ref(), RasterVariantBody10_1::StatsHistogram(histogram)
                        if histogram.version == version && histogram.bins.len() == 256
                            && histogram.double_a.is_none())
            ));
            assert_eq!(reader.position(), end);
            assert!(refs.contains(11));
        }

        bytes[version_offset..version_offset + 4].copy_from_slice(&6_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 4096, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn variant_stats_histogram_commits_only_complete_body() {
        use crate::layers::raster_renderer::STATS_HISTOGRAM_CLASS_ID;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, STATS_HISTOGRAM_CLASS_ID, 11);
        bytes.extend(3_i32.to_le_bytes());
        bytes.extend([0; 48]);
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(5.0_f64.to_le_bytes());
        bytes.extend(2.0_f64.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::Raster {
            class_id,
            key,
            body,
        } = value
        else {
            panic!("缺少栅格 Variant 对象");
        };
        assert_eq!(class_id, STATS_HISTOGRAM_CLASS_ID);
        assert_eq!(key, 11);
        let RasterVariantBody10_1::StatsHistogram(histogram) = *body else {
            panic!("缺少统计直方图");
        };
        assert_eq!(histogram.bins, [5.0]);
        assert_eq!(histogram.double_a, Some(2.0));
        assert_eq!(reader.position(), end);
        assert_eq!(refs.registered_class(11), Some(STATS_HISTOGRAM_CLASS_ID));

        let mut reader = Reader::new(&bytes[..end - 1]);
        let mut refs = RefTable10_1::new();
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
        assert!(!refs.contains(11));
    }

    #[test]
    fn variant_unique_values_preserves_nonpositive_counts() {
        use crate::layers::raster_unique_value::UNIQUE_VALUES_CLASS_ID;

        for count in [i32::MIN, -1, 0] {
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            bytes.extend([0; 14]);
            object(&mut bytes, UNIQUE_VALUES_CLASS_ID, 11);
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(count.to_le_bytes());
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let result =
                read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
            assert!(
                matches!(result, PropertyValueObjectStream10_1::UniqueValues(value)
                if value.body().declared_count == count && value.body().values.is_empty())
            );
            assert_eq!(reader.read_bytes(1).unwrap(), &[0xaa]);
            assert!(refs.contains(11));
        }
    }

    #[test]
    fn variant_unique_values_reads_members_and_keeps_depth_budget() {
        use crate::layers::raster_unique_value::UNIQUE_VALUES_CLASS_ID;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, UNIQUE_VALUES_CLASS_ID, 11);
        bytes.extend(2_i32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, UNIQUE_VALUES_CLASS_ID, 12);
        bytes.extend((-1_i32).to_le_bytes());
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(17_i32.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xaa);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 256, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::UniqueValues(value) = value else {
            panic!("缺少唯一值集合");
        };
        assert!(matches!(value.body().values.as_slice(),
            [(PropertyValueObjectStream10_1::UniqueValues(child), 17)] if child.key == 12 && child.body().version == -1));
        assert_eq!(value.body().version, 2);
        assert!(refs.contains(11) && refs.contains(12));
        assert_eq!(reader.position(), end);
        for (bytes, depth) in [
            (&bytes[..end - 1], 0),
            (&bytes[..], MAX_NESTED_VARIANT_OBJECTS - 1),
        ] {
            let mut reader = Reader::new(bytes);
            let mut refs = RefTable10_1::new();
            assert!(
                read_padded_variant_object_stream_nested_10_1(&mut reader, 256, &mut refs, depth)
                    .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn variant_reads_fid_set_classes_and_keeps_their_distinct_id_rules() {
        use crate::layers::feature_id_set::{FEATURE_ID_SET_CLASS_ID, FID_SET_CLASS_ID};

        for (class_id, expected) in [
            (FEATURE_ID_SET_CLASS_ID, &[-2, 7][..]),
            (FID_SET_CLASS_ID, &[7][..]),
        ] {
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            bytes.extend([0; 14]);
            object(&mut bytes, class_id, 18);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(3_i32.to_le_bytes());
            for id in [-2_i32, 7, 7] {
                bytes.extend(id.to_le_bytes());
            }
            let end = bytes.len();
            bytes.push(0xaa);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value =
                read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
            assert!(
                matches!(value, PropertyValueObjectStream10_1::FeatureIdSet {
                class_id: actual, body,
            } if actual == class_id && body.key == 18 && body.count == 3 && body.ids == expected)
            );
            assert_eq!(reader.position(), end);
            assert!(refs.contains(18));

            let mut reader = Reader::new(&bytes[..end - 1]);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn field_variants_share_the_depth_budget_with_default_values() {
        use crate::objects::fields::{FIELD_CLASS_ID, FIELDS_CLASS_ID};

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, FIELD_CLASS_ID, 11);
        bytes.extend([0; 6]);
        for (index, flag) in [true, true, false, true].into_iter().enumerate() {
            bytes.extend([2, 0, 0, 0, 0, 0]);
            if flag {
                bytes.push(index as u8);
            }
        }
        bytes.extend(13_u16.to_le_bytes());
        object(&mut bytes, FIELDS_CLASS_ID, 12);
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend([0; 16 + 5 + 16 + 16]);
        let end = bytes.len();
        bytes.push(0xaa);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let parsed = read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::Field(field) = parsed else {
            panic!("缺少字段对象");
        };
        assert_eq!(field.key, 11);
        assert!(
            matches!(&field.body().default, PropertyValueObjectStream10_1::Fields(fields)
            if fields.key == 12 && fields.body().fields.is_empty())
        );
        assert!(refs.contains(11) && refs.contains(12));
        assert_eq!(reader.position(), end);

        for (input, depth) in [
            (&bytes[..end - 1], 0),
            (&bytes[..], MAX_NESTED_VARIANT_OBJECTS - 1),
        ] {
            let mut reader = Reader::new(input);
            let mut refs = RefTable10_1::new();
            assert!(
                read_padded_variant_object_stream_nested_10_1(&mut reader, 1024, &mut refs, depth)
                    .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn property_set_str_array_can_be_resolved_from_a_typed_slot() {
        use crate::objects::string_array::read_string_array_slot_10_1;

        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, STR_ARRAY_CLASS_ID, 4);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([4, 0, 0, 0, b'x', 0, 0, 0]);
        let boundary = bytes.len();
        object(&mut bytes, STR_ARRAY_CLASS_ID, 4);
        let mut refs = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap();
        assert_eq!(reader.position(), boundary);
        let mut limited = reader.clone();
        assert!(
            read_string_array_slot_10_1(&mut limited, 13, &mut refs)
                .unwrap()
                .body
                .is_none()
        );
        let array = read_string_array_slot_10_1(&mut reader, 14, &mut refs).unwrap();
        assert_eq!(array.body, Some((1, vec!["x".to_owned()])));
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn var_array_reads_padded_items_and_shares_keys_with_property_set_array() {
        let mut bytes = Vec::new();
        variant_array(&mut bytes, 1, 3);
        bytes.extend(9_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, PROPERTY_SET_ARRAY_CLASS_ID, 2);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        object(&mut bytes, PROPERTY_SET_CLASS_ID, 3);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([2, 0, 0, 0, 0, 0]); // 空属性名。
        bytes.extend(13_u16.to_le_bytes()); // PropertySet 属性内的 Variant 无保留区。
        object(&mut bytes, VAR_STR_ARRAY_CLASS_ID, 4);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend([4, 0, 0, 0, b'x', 0, 0, 0]);
        let property_ref_offset = bytes.len();
        object(&mut bytes, PROPERTY_SET_CLASS_ID, 3);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, VAR_STR_ARRAY_CLASS_ID, 4);
        bytes.extend(3_u16.to_le_bytes());
        bytes.extend([0; 14]);
        bytes.extend(123_i32.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xbb);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::ArrayObject {
            key: 1,
            values: VariantArrayObjectValues10_1::Variants(values),
            ..
        } = value
        else {
            panic!("缺少 VarArray");
        };
        let [
            PropertyValueObjectStream10_1::ArrayObject {
                key: 2,
                values: VariantArrayObjectValues10_1::PropertySets(sets),
                ..
            },
            PropertyValueObjectStream10_1::BackReference {
                class_id: VAR_STR_ARRAY_CLASS_ID,
                key: 4,
            },
            PropertyValueObjectStream10_1::Inline(PropertyValue10_1::Raw4(raw)),
        ] = values.as_slice()
        else {
            panic!("VarArray 元素与边界不符");
        };
        assert_eq!(*raw, 123_i32.to_le_bytes());
        assert_eq!(sets.len(), 2);
        assert_eq!(
            sets[0].body.as_ref().unwrap().entries[0].value,
            PropertyValueObjectStream10_1::ArrayObject {
                class_id: VAR_STR_ARRAY_CLASS_ID,
                key: 4,
                version: 1,
                values: VariantArrayObjectValues10_1::Strings(vec!["x".into()]),
            }
        );
        assert_eq!(
            sets[1],
            PropertySetArrayEntry10_1 {
                entry: ObjectEntry10_1::BackReference {
                    class_id: PROPERTY_SET_CLASS_ID,
                    key: 3
                },
                body: None,
            }
        );
        assert_eq!(refs.len(), 4);
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xbb]);

        let mut invalid_item = bytes.clone();
        invalid_item[property_ref_offset..property_ref_offset + 16].fill(0);
        for invalid in [&invalid_item[..], &bytes[..end - 1]] {
            let mut reader = Reader::new(invalid);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn variant_arrays_keep_the_depth_limit_when_reading_recursive_items() {
        let mut bytes = Vec::new();
        for key in 0..MAX_NESTED_VARIANT_OBJECTS as u32 {
            variant_array(&mut bytes, key, 1);
        }
        variant_array(&mut bytes, MAX_NESTED_VARIANT_OBJECTS as u32, 0);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error =
            read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).unwrap_err();
        assert!(error.to_string().contains("嵌套深度"));
        assert_eq!(reader.position(), 0);
        assert!(refs.is_empty());
    }

    #[test]
    fn raster_variant_and_var_array_share_depth_limit_and_roll_back() {
        let mut bytes = Vec::new();
        for key in 0..MAX_NESTED_VARIANT_OBJECTS as u32 {
            if key % 2 == 0 {
                bytes.extend(13_u16.to_le_bytes());
                bytes.extend([0; 14]);
                object(&mut bytes, RASTER_COLORMAP, key);
                bytes.extend((-1_i16).to_le_bytes());
            } else {
                variant_array(&mut bytes, key, 1);
            }
        }
        let limit_offset = bytes.len();
        variant_array(&mut bytes, MAX_NESTED_VARIANT_OBJECTS as u32, 0);
        let mut refs = RefTable10_1::new();
        refs.register(101);
        let original = refs.clone();
        let mut reader = Reader::new(&bytes);
        let error =
            read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).unwrap_err();
        assert!(error.to_string().contains("嵌套深度"));
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);

        // 达到限额后仍可消费已登记的回引。
        bytes.truncate(limit_offset);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, RASTER_COLORMAP, 0);
        for cut in [limit_offset - 1, bytes.len() - 1] {
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, original);
        }
        let mut reader = Reader::new(&bytes);
        assert!(read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).is_ok());
        assert_eq!(reader.remaining(), 0);
        assert_eq!(refs.len(), MAX_NESTED_VARIANT_OBJECTS + 1);

        bytes[36..38].copy_from_slice(&3_u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = original.clone();
        let error =
            read_padded_variant_object_stream_10_1(&mut reader, 8192, &mut refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);
    }

    #[test]
    fn graphic_attribute_variant_shares_keys_and_limits_recursion() {
        use crate::objects::display_symbol::LINE_STROKE_CLASS_ID;

        let mut bytes = Vec::new();
        for key in 1..=2_u32 {
            bytes.extend(13_u16.to_le_bytes());
            bytes.extend([0; 14]);
            object(&mut bytes, LINE_STROKE_CLASS_ID, key);
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(u32::from(key == 1).to_le_bytes());
            if key == 1 {
                bytes.extend(5_i32.to_le_bytes());
            }
        }
        let end = bytes.len();
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, LINE_STROKE_CLASS_ID, 2);
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let PropertyValueObjectStream10_1::GraphicAttributes(outer) =
            read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap()
        else {
            panic!("未读出图形属性对象");
        };
        let (id, value) = &outer.body().attributes[0];
        assert_eq!(*id, 5);
        assert!(
            matches!(value, PropertyValueObjectStream10_1::GraphicAttributes(value) if value.key == 2)
        );
        assert_eq!(reader.position(), end);
        assert_eq!(
            read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap(),
            PropertyValueObjectStream10_1::BackReference {
                class_id: LINE_STROKE_CLASS_ID,
                key: 2
            }
        );

        for end in 0..end {
            let mut reader = Reader::new(&bytes[..end]);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 0);
        }

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        assert!(
            read_padded_variant_object_stream_nested_10_1(
                &mut reader,
                1024,
                &mut refs,
                MAX_NESTED_VARIANT_OBJECTS - 1,
            )
            .is_err()
        );
        assert_eq!(reader.position(), 0);
        assert_eq!(refs.len(), 0);
    }

    #[test]
    fn color_variant_preserves_boundary_and_rolls_back_truncation() {
        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(
            &mut bytes,
            crate::objects::display_color::RGB_COLOR_CLASS_ID,
            1,
        );
        let body = [1, 0, 1, 0, 1, 4, 3, 2, 1, 0, 255];
        bytes.extend(body);
        let mut refs = RefTable10_1::new();
        for end in 0..bytes.len() {
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs.len(), 0);
        }
        bytes.push(0xac);
        let mut reader = Reader::new(&bytes);
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::Color(color) = value else {
            panic!("应为颜色对象");
        };
        assert_eq!(color.bytes(), body);
        assert_eq!(reader.remaining(), 1);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn color_ramp_variant_preserves_nested_keys_and_rejects_unknown_color() {
        let mut bytes = 13_u16.to_le_bytes().to_vec();
        bytes.extend([0; 14]);
        object(&mut bytes, MULTI_PART_COLOR_RAMP_CLASS_ID, 1);
        let body_start = bytes.len();
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        object(&mut bytes, ALGORITHMIC_COLOR_RAMP_CLASS_ID, 2);
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend([0; 8]);
        let color_start = bytes.len();
        object(
            &mut bytes,
            crate::objects::display_color::RGB_COLOR_CLASS_ID,
            3,
        );
        bytes.extend([1, 0, 1, 0, 1, 4, 3, 2, 1, 0, 255]);
        object(
            &mut bytes,
            crate::objects::display_color::RGB_COLOR_CLASS_ID,
            3,
        );
        bytes.extend(1.25_f64.to_le_bytes());
        let end = bytes.len();
        bytes.push(0xac);

        let mut refs = RefTable10_1::new();
        refs.register(9);
        let original = refs.clone();
        for cut in 0..end {
            let mut reader = Reader::new(&bytes[..cut]);
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert_eq!(refs, original);
        }
        let mut unknown = bytes.clone();
        unknown[color_start..color_start + 16].fill(0xee);
        let mut reader = Reader::new(&unknown);
        let err = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Unsupported);
        assert_eq!(reader.position(), 0);
        assert_eq!(refs, original);

        let mut reader = Reader::new(&bytes);
        let value = read_padded_variant_object_stream_10_1(&mut reader, 128, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::ColorRamp(ramp) = value else {
            panic!("应为颜色梯度对象");
        };
        assert_eq!(ramp.bytes(), &bytes[body_start..end]);
        let ColorRampSlot10_1::MultiPart(body) = ramp.body() else {
            panic!("应为组合渐变");
        };
        assert_eq!(
            body.tail_doubles.as_deref(),
            Some(&[1.25_f64.to_bits()][..])
        );
        assert_eq!(reader.position(), end);
        assert_eq!(refs.len(), 4);
    }

    #[test]
    fn variant_value_array_and_uid_have_distinct_body_boundaries() {
        let mut bytes = Vec::new();
        variant_array(&mut bytes, 1, 3);
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, VALUE_ARRAY_CLASS_ID, 2);
        let version_offset = bytes.len();
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, UID_CLASS_ID, 3);
        bytes.extend(1_u16.to_le_bytes());
        bytes.push(1);
        bytes.extend([0x4d; 16]);
        bytes.extend((-3_i32).to_le_bytes());
        bytes.extend(13_u16.to_le_bytes());
        bytes.extend([0; 14]);
        object(&mut bytes, UID_CLASS_ID, 3);
        let end = bytes.len();
        bytes.push(0xac);

        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let value = read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).unwrap();
        let PropertyValueObjectStream10_1::ArrayObject {
            values: VariantArrayObjectValues10_1::Variants(values),
            ..
        } = value
        else {
            panic!("缺少 VarArray");
        };
        assert_eq!(
            values,
            [
                PropertyValueObjectStream10_1::ArrayObject {
                    class_id: VALUE_ARRAY_CLASS_ID,
                    key: 2,
                    version: 1,
                    values: VariantArrayObjectValues10_1::Variants(Vec::new()),
                },
                PropertyValueObjectStream10_1::Uid(UidBody10_1 {
                    key: 3,
                    version: 1,
                    flag: 1,
                    guid: [0x4d; 16],
                    value_18: -3,
                }),
                PropertyValueObjectStream10_1::BackReference {
                    class_id: UID_CLASS_ID,
                    key: 3,
                },
            ],
        );
        assert_eq!(refs.len(), 3);
        assert_eq!(reader.position(), end);
        assert_eq!(reader.read_bytes(1).unwrap(), [0xac]);

        let mut nonempty = bytes.clone();
        nonempty[version_offset + 2] = 1;
        let mut invalid_version = bytes.clone();
        invalid_version[version_offset] = 2;
        for invalid in [&nonempty[..], &invalid_version, &bytes[..end - 1]] {
            let mut reader = Reader::new(invalid);
            let mut refs = RefTable10_1::new();
            assert!(read_padded_variant_object_stream_10_1(&mut reader, 1024, &mut refs).is_err());
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn variant_time_extent_preserves_nested_keys_and_rolls_back_unknown_slot() {
        for padded in [false, true] {
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            if padded {
                bytes.extend([0; 14]);
            }
            bytes.extend(TIME_EXTENT_CLASS_ID);
            bytes.extend(41_u32.to_le_bytes());
            let body_offset = bytes.len();
            bytes.extend(2_u16.to_le_bytes());
            bytes.extend(TIME_CLASS_ID);
            bytes.extend(42_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(1_i32.to_le_bytes());
            bytes.extend(27.0_f64.to_le_bytes());
            bytes.extend(TIME_CLASS_ID);
            bytes.extend(42_u32.to_le_bytes());
            bytes.extend(TIME_REFERENCE_CLASS_ID);
            bytes.extend(43_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            let zone_offset = bytes.len();
            bytes.extend([0; 16]);
            bytes.extend([2, 0, 3, 0]);
            bytes.push(1);
            let end = bytes.len();
            bytes.push(0xee);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let PropertyValueObjectStream10_1::Time(value) =
                read_variant_object_stream_layout_10_1(&mut reader, 4096, &mut refs, 0, padded)
                    .unwrap()
            else {
                panic!("TimeExtent 未分派");
            };
            let VariantTimeBody10_1::Extent(body) = value.body else {
                panic!("缺少 TimeExtent 体");
            };
            assert_eq!(body.tail_byte, Some(1));
            assert_eq!(&*value.bytes, &bytes[body_offset..end]);
            assert!(refs.contains(41) && refs.contains(42) && refs.contains(43));
            assert_eq!(reader.read_bytes(1).unwrap(), &[0xee]);

            for end in 0..end {
                let mut reader = Reader::new(&bytes[..end]);
                let mut refs = RefTable10_1::new();
                assert!(
                    read_variant_object_stream_layout_10_1(&mut reader, 4096, &mut refs, 0, padded)
                        .is_err()
                );
                assert_eq!(reader.position(), 0);
                assert!(refs.is_empty());
            }
            bytes[zone_offset..zone_offset + 16].fill(0x7d);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            assert!(
                read_variant_object_stream_layout_10_1(&mut reader, 4096, &mut refs, 0, padded)
                    .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(refs.is_empty());
        }
    }

    #[test]
    fn object_stream_double_array_keeps_bits_and_atomic_boundary() {
        let bits = [(-0.0_f64).to_bits(), 0x7ff8_0000_0000_0012];
        for padded in [false, true] {
            let mut bytes = 13_u16.to_le_bytes().to_vec();
            if padded {
                bytes.extend([0xa1; 14]);
            }
            bytes.extend(DOUBLE_ARRAY_CLASS_ID);
            bytes.extend(42_u32.to_le_bytes());
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(2_i32.to_le_bytes());
            for value in bits {
                bytes.extend(value.to_le_bytes());
            }
            let end = bytes.len();
            bytes.push(0x2b);
            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            let value =
                read_variant_object_stream_layout_10_1(&mut reader, 22, &mut refs, 0, padded)
                    .unwrap();
            assert_eq!(
                value,
                PropertyValueObjectStream10_1::ArrayObject {
                    class_id: DOUBLE_ARRAY_CLASS_ID,
                    key: 42,
                    version: 1,
                    values: VariantArrayObjectValues10_1::DoubleBits(bits.to_vec()),
                }
            );
            assert_eq!(reader.position(), end);
            assert_eq!(reader.read_bytes(1).unwrap(), [0x2b]);
            assert!(refs.contains(42));

            let mut reader = Reader::new(&bytes);
            let mut refs = RefTable10_1::new();
            assert!(
                read_variant_object_stream_layout_10_1(&mut reader, 21, &mut refs, 0, padded)
                    .is_err()
            );
            assert_eq!(reader.position(), 0);
            assert!(!refs.contains(42));
        }
    }

    #[test]
    fn property_set_version_zero_accepts_empty_set() {
        // Load 对版本 0 与 1 使用同一字段序；0 计数不进入属性循环。
        let mut bytes = 0_u16.to_le_bytes().to_vec();
        bytes.extend(0_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let body = read_property_set_10_1(&mut reader, 64, &mut refs).unwrap();
        assert_eq!(body.version, 0);
        assert_eq!(body.count, 0);
        assert!(body.entries.is_empty());
        assert_eq!(reader.position(), bytes.len());
    }

    #[test]
    fn property_set_version_above_one_rejects_without_advancing() {
        let mut bytes = 2_u16.to_le_bytes().to_vec();
        bytes.extend(0_i32.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        let error = read_property_set_10_1(&mut reader, 64, &mut refs).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn truncated_property_set_version_zero_rejects_without_advancing() {
        let bytes = 0_u16.to_le_bytes();
        let mut reader = Reader::new(&bytes);
        let mut refs = RefTable10_1::new();
        assert!(read_property_set_10_1(&mut reader, 64, &mut refs).is_err());
        assert_eq!(reader.position(), 0);
    }
}
