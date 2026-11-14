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
//! `LoadObject`（16 字节类 GUID、无键、非 ObjectStream 编码）——对象分支
//! 边界未闭合，本读取器显式不支持。数组标志 0x2000（含字符串数组与批量
//! 定宽数组）同样未闭合，显式不支持。
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
//! `table` 参数仅为与同类读取器签名一致（本体内没有 ObjectStream 编码的
//! 嵌套条目，登记表不被读取或修改）。这是仅覆盖上述分支的严格安全子集，
//! 不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
//!
//! 未解决问题（详见证据文档「开放项」）：SystemCore 本地 `Variant::Read`
//! 编译副本与 AfCore 副本的逐 VARTYPE 对照仅覆盖签名与样本 VT_BSTR 分支；
//! 各值的业务语义（DATABASE/连接属性候选）未定性；版本 0 无样本佐证；
//! 全部结论为静态观察，未经目标软件运行验证。

use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// PropertySet::Load 接受的最大版本：汇编 `cmp word,1; jle` 有符号比较，>1
/// 拒绝；Save 恒写 1；样本两实例均为 1。
const MAX_BODY_VERSION: i16 = 1;

/// 每个属性条目的最小流内字节数：键名 String 最小 4 字节（cb=0 空串）+
/// 值 Variant 最小 2 字节（无载荷 VARTYPE）。用于计数 ×最小条目的限额预检。
const MIN_ENTRY_BYTES: usize = 6;

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

/// PropertySet 值 Variant 的已确认分支（`Variant::Read` 非数组部分）。
///
/// 固定宽度值不转换成有符号数、浮点数或布尔语义，仅原样保留位模式；
/// 字符串经 AfCore String 布局解码。对象（9/13）与数组（0x2000 标志）分
/// 支显式不支持，不产生本枚举值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue10_1 {
    /// VT_EMPTY(0)：无载荷。
    Empty,
    /// VT_NULL(1)：无载荷。
    Null,
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
/// 外设限。`table` 参数不使用（本体没有 ObjectStream 编码的嵌套条目；对象
/// 分支 9/13 与数组分支在读取前显式拒绝），仅为与同类读取器签名一致。
pub fn read_property_set_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut crate::storage::object_refs::RefTable10_1,
) -> io::Result<PropertySet10_1> {
    let _ = table; // 无嵌套 ObjectStream 条目；见模块文档。
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

/// 读取一个 `Variant::Read`（AfCore `0x68a7c6a0`，非数组分支）值，失败时
/// 不推进游标。
///
/// VARTYPE 表见模块文档；对象分支（9/13）与数组标志（0x2000）显式不支持；
/// 未知 VARTYPE 与原生 default 分支一致返回错误。字符串为 AfCore String
/// 布局（`String::Read`，4 字节 cb 含 UTF-16LE 终止零）。
pub fn read_variant_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
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
    if value_type & 0x2000 != 0 {
        return Err(io::Error::new(
            ErrorKind::Unsupported,
            format!(
                "Variant 值 offset 0x{start:x}: 数组标志 0x2000（VARTYPE 0x{value_type:04x}）\
                 的 SafeArray 分支未闭合，本读取器不支持"
            ),
        ));
    }
    let value = match value_type {
        0 => PropertyValue10_1::Empty,
        1 => PropertyValue10_1::Null,
        2 | 11 => PropertyValue10_1::Raw2(trial.read_bytes(2)?.try_into().unwrap()),
        3 | 10 | 4 => PropertyValue10_1::Raw4(trial.read_bytes(4)?.try_into().unwrap()),
        5 | 7 | 6 => PropertyValue10_1::Raw8(trial.read_bytes(8)?.try_into().unwrap()),
        8 => PropertyValue10_1::String(read_string_10_1(&mut trial, max_bytes)?),
        9 | 13 => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Variant 值 offset 0x{start:x}: VARTYPE {value_type}（对象分支）走 AfCore \
                     LoadObject 的无键类 GUID 编码，对象边界未闭合，本读取器不支持"
                ),
            ));
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
