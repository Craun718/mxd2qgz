//! ArcMap 10.1 EditTemplate（要素模板）对象体的严格读取子集，及 AfCore
//! `Variant` 类方法序列化器（`?Read@Variant@@QAEJPAUIStream@@_N@Z`）的
//! 严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-legend-tails.md`（2026-11-15 本轮研究记录）。
//! 二进制证据为 FeatureLayer.dll 10.1.0.3035（temp/arcgis101-bin/ 既有基线）
//! 与 AfCore.dll 10.1.0.3035（同目录）的静态反编译。
//!
//! ## 类身份与持久化入口（证据：确定的）
//!
//! esri.clsid.ecfg 声明 CLSID `{377035B9-34C7-4A40-866D-E596519A1B06}`
//! （模块 FeatureLayer.dll，类名 EditTemplate）；实现类 RTTI TD
//! `0x6f7d24b8` 名 `.?AVCEditTemplate@@`（"EditTemplate" 字符串由
//! `0x6f6cc050` 返回）。QI（`0x6f6cc350`）把 IPersistStream
//! `{00000109-…}` 映射到 this+0x14，构造（`0x6f6cc0a0`）在该偏移安装虚表
//! `0x6f7ab8c0`（COL `0x6f7b5ce0`，TD 同上）：+0x0c = GetClassID
//! `0x6f6ce1a0` 写出上述 CLSID、+0x14 = Load `0x6f6ce1e0`、+0x18 = Save
//! `0x6f6ce3c0`。类名 identity 虚表 `0x6f7ab848` 的 GetClassID
//! `0x6f6cc060` 写出同一 CLSID（双方互证）。
//!
//! ## CEditTemplate::Load 字段序（FeatureLayer.dll `0x6f6ce1e0`）
//!
//! 1. int16 版本：汇编 `CMP word,2; JLE` 有符号（>2 拒绝 `0x80004005`；
//!    负值位模式原生通过，本读取器一并拒绝）。Save 侧
//!    `AfStreamGetSaveVersion(docVer, 5, DAT_6f77c02c)`，版本表
//!    `[0xFFFF,0xFFFF,0xFFFF,0xFFFF,2]`：docVer=83..92 时 Save 直接返回
//!    不写任何字节；10.0（docVer=4）与 10.1（docVer=5）恒写 **2**；
//! 2. String ×2（持久化接口 this+0x08/0x0c）：Name、Description；
//! 3. 16 字节原始 GUID（持久化接口 this+0x18）：Tool，原样保留；
//! 4. int32 计数 + 循环：每项 [String 键][`Variant::Read` 值]——键为
//!    字段名、值为该字段模板默认值（样本 12 项 = 字段默认值表候选）；
//!    原生负数/零计数跳过循环，本读取器拒绝负数（较原生更严）；
//! 5. 版本 ≥2：String（持久化接口 this+0x10，Tags）。Save 侧该项在
//!    迭代器耗尽后恒写（无版本门），与读侧版本 ≥2 门在 10.1 一致
//!    （Save 恒写版本 2）。
//!
//! ## AfCore `Variant::Read`（`0x68a7c6a0`，thiscall (this, stream, bool)）
//!
//! 与 `variant_stream`（VariantStream 全局 Read）**不同的独立序列化器**
//! （[`crate::stream::variant_stream`] 文档已记录其存在；本轮按本类调用
//! 点 bool=true 闭合其分支）：
//!
//! 1. 读 2 字节 VARTYPE（`IStream::Read` 直读）；bool=false 另有
//!    `Seek` 分支，本类调用点恒传 true，读取器不实现 false 路径；
//! 2. 按 VARTYPE（0x2000 = VT_ARRAY 标志，去掉标志后进同一张表）：
//!    | VT | 载荷 |
//!    | --- | --- |
//!    | 0 EMPTY / 1 NULL | 无（带 0x2000 标志时原生改道 BSTR 数组路径，语义未定，本读取器拒绝） |
//!    | 2 I2 / 11 BOOL | 2 字节（BOOL 原样保留位模式） |
//!    | 3 I4 / 10 ERROR | 4 字节 |
//!    | 4 R4 | 4 字节 |
//!    | 5 R8 / 7 DATE | 8 字节 |
//!    | 6 CY | 8 字节 |
//!    | 8 BSTR | AfCore `String::Read` 布局（u32 字节长度含终止零 + 载荷），**不是** `ReadString` 的 `n*2+2` 布局 |
//!    | 16 I1 / 17 UI1 | 1 字节 |
//!    | 18 UI2 | 2 字节 |
//!    | 19 UI4 | 4 字节 |
//!    | 9 DISPATCH / 13 UNKNOWN | 全局 `LoadObject`（16 字节类 GUID、零 GUID 语义，COM 上下文未闭合——**显式拒绝**，与 `variant_stream` 对象分支同纪律） |
//!    | 其他 | 原生返回 `0x80004005`，读取器报 InvalidData |
//! 3. 带 0x2000 标志的固定宽度类型：追加 int32 计数（原生负计数使
//!    SafeArrayCreate 失败 → `0x80004005`；读取器拒绝负数）+
//!    count × 元素字节数的**单次整块原始读取**；带标志的 BSTR：追加
//!    int32 计数 + 每元素 `ReadString`（**`n*2+2` 布局**，与第 2 条非
//!    数组 BSTR 布局不同）。
//!
//! ## 样本对位（辅助观察）
//!
//! 仓库参考样本 /Maps 流中本类 CLSID **10 命中**（新对象头，键 76、146、
//! 1605、1726、1784、1842、1925、2003、2075、2151）：键 1605 实例
//! （`/Maps:0x17f62`）体逐字段吻合——版本 2、String「JZD1」、空 String、
//! GUID `{2A8B3331-5238-4025-972E-452A69535B06}`、计数 12 的 [String 键
//! (JZDH/YSDM/ZDZHDM/JBLX/OBJECTID/ZZBZ/YZBZ/BSM/XZBZ/JZDLX/BDCDYBSM/
//! Ring)][Variant 值（BSTR 空 ×8、I4 0、R8 0 ×3）]、版本 ≥2 String「点」，
//! 体终结于其后 SimpleMarkerSymbol 新对象头。宿主槽位（其前的
//! `02 00 01 00 00 00` 尾段）属 FeatureLayer 后继字段链，本轮不定名。
//!
//! 调用约定：调用者已用 [`crate::storage::object_stream::read_header_10_1`]
//! 读取对象头、比对 [`EDIT_TEMPLATE_CLASS_ID`] 并取得键，从对象头之后的
//! 位置传入 Reader；本模块不重复读取对象头。任何失败不推进调用者游标、
//! 不修改登记表；`max_bytes` 约束字符串声明长度与循环「计数 × 最小条目
//! 字节数」预检（`usize::MAX` 不设限）。本读取器不递归解析其他对象体，
//! 不产生停止点（对象分支直接报不支持）。
//!
//! 访问器对位见 `docs/mxd-terra-host-label.md`。未解决问题：
//! [String,Variant] 键值对的重复键行为未逐项定性；VT_EMPTY/NULL 带
//! 数组标志的原生改道路径未展开；
//! 全部结论为静态观察与样本辅助对位，未经目标软件运行验证。

use crate::storage::object_refs::RefTable10_1;
use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// EditTemplate 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{377035B9-34C7-4A40-866D-E596519A1B06}`。证据：
/// esri.clsid.ecfg 声明（模块 FeatureLayer.dll，类名 EditTemplate）；
/// FeatureLayer.dll GetClassID `0x6f6cc060`/`0x6f6ce1a0` 写出同一 CLSID；
/// RTTI TD `0x6f7d24b8` 名 `.?AVCEditTemplate@@` 经 COL `0x6f7b5ce0` 与
/// IPersistStream 虚表 `0x6f7ab8c0` 相连。样本 /Maps 流 10 命中（见模块
/// 文档「样本对位」）。
pub const EDIT_TEMPLATE_CLASS_ID: [u8; 16] = [
    0xb9, 0x35, 0x70, 0x37, 0xc7, 0x34, 0x40, 0x4a, 0x86, 0x6d, 0xe5, 0x96, 0x51, 0x9a, 0x1b, 0x06,
];

/// CEditTemplate::Load 接受的最大版本：汇编有符号 `>2` 拒绝；10.1 Save
/// 恒写 2。
const MAX_BODY_VERSION: i16 = 2;

/// 循环条目的最小字节数：String（4 字节长度 + 2 字节终止零）+ Variant
/// （2 字节 VARTYPE + EMPTY 无载荷）。
const MIN_FIELD_ENTRY_BYTES: usize = 6 + 2;

/// VT_ARRAY 标志（与 VT 型号按位或出现）。
const VT_ARRAY_FLAG: u16 = 0x2000;

/// AfCore `Variant::Read` 值的严格读取结果。
///
/// 数值字段全部保留流内位模式（BOOL 为 VARIANT_BOOL 原样 16 位、CY 为
/// 8 字节整型位模式、DATE 为 8 字节 double 位模式），不做数值解释或
/// 归一化。
#[derive(Debug, Clone, PartialEq)]
pub enum VariantClassValue10_1 {
    /// VT_EMPTY（0）：无载荷。
    Empty,
    /// VT_NULL（1）：无载荷。
    Null,
    /// VT_I2（2）：2 字节。
    I2(i16),
    /// VT_I4（3）：4 字节。
    I4(i32),
    /// VT_R4（4）：4 字节 float。
    R4(f32),
    /// VT_R8（5）：8 字节 double。
    R8(f64),
    /// VT_CY（6）：8 字节（scaled integer 位模式）。
    Cy(i64),
    /// VT_DATE（7）：8 字节 double 位模式。
    Date(f64),
    /// VT_ERROR（10）：4 字节 HRESULT 位模式。
    Error(u32),
    /// VT_BOOL（11）：2 字节 VARIANT_BOOL 原样位模式（0xFFFF/0 之外视为
    /// 非规范位模式，原样保留）。
    Bool(u16),
    /// VT_I1（16）：1 字节。
    I1(i8),
    /// VT_UI1（17）：1 字节。
    Ui1(u8),
    /// VT_UI2（18）：2 字节。
    Ui2(u16),
    /// VT_UI4（19）：4 字节。
    Ui4(u32),
    /// VT_BSTR（8，非数组）：AfCore String::Read 布局。
    Bstr(String),
    /// VT_… | VT_ARRAY（0x2000）：int32 计数 + 元素序列。
    Array(ClassVariantArray10_1),
}

/// 数组变体的元素序列。
#[derive(Debug, Clone, PartialEq)]
pub enum ClassVariantArray10_1 {
    /// 固定宽度元素：去 0x2000 后的 VARTYPE、元素字节数与整块原始字节
    /// （原生为一次 `IStream::Read(count × 元素字节数)`）。
    Fixed {
        /// 去 0x2000 标志后的 VARTYPE。
        vartype: u16,
        /// 单元素字节数。
        element_bytes: usize,
        /// count × 元素字节数的原始字节。
        raw: Vec<u8>,
    },
    /// VT_BSTR|VT_ARRAY：每元素 `ReadString`（n*2+2 布局）。
    Strings(Vec<String>),
}

/// 从当前位置读取一个 AfCore `Variant::Read`（bool=true 路径）值，失败时
/// 不推进游标。
///
/// 对象分支（VT_DISPATCH/VT_UNKNOWN）与 EMPTY/NULL 带数组标志为显式拒绝
/// （[`ErrorKind::Unsupported`]，COM 上下文未闭合/语义未定，见模块文档）；
/// 未知 VARTYPE 报 [`ErrorKind::InvalidData`]（对应原生 0x80004005）。
/// `max_bytes` 约束字符串与数组载荷（`usize::MAX` 不设限）。不触及
/// 登记表（本序列化器不产生对象条目）。
pub fn read_variant_class_value_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<VariantClassValue10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Variant::Read 值 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();

    // 1. VARTYPE（2 字节）。
    let raw_vt = trial.read_u16_le()?;
    let is_array = raw_vt & VT_ARRAY_FLAG != 0;
    let vt = raw_vt & !VT_ARRAY_FLAG;

    // 2. 固定宽度宽度表（元素字节数）。
    let fixed_width: Option<usize> = match vt {
        2 | 11 => Some(2),
        3 | 4 | 10 | 19 => Some(4),
        5 | 6 | 7 => Some(8),
        16 | 17 => Some(1),
        18 => Some(2),
        _ => None,
    };

    if is_array {
        match vt {
            8 => {
                // BSTR 数组：int32 计数 + 每元素 ReadString（n*2+2）。
                let count_offset = trial.position();
                let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
                if count < 0 {
                    return Err(invalid(format!(
                        "BSTR 数组计数 offset 0x{count_offset:x}: 计数为负数（{count}）；\
                         原生负计数使 SafeArrayCreate 失败返回 0x80004005"
                    )));
                }
                let count_usize = usize::try_from(count).expect("计数已确认为非负");
                let min_total = count_usize
                    .checked_mul(6) // 每元素最小 4 字节 n + 2 字节载荷
                    .ok_or_else(|| invalid("BSTR 数组计数 × 最小元素字节数溢出".to_string()))?;
                if min_total > max_bytes {
                    return Err(invalid(format!(
                        "BSTR 数组计数 {count} 超过资源限额 {max_bytes}"
                    )));
                }
                let mut items = Vec::new();
                for _ in 0..count_usize {
                    items.push(read_variant_bstr_array_element_10_8(&mut trial, max_bytes)?);
                }
                let value = VariantClassValue10_1::Array(ClassVariantArray10_1::Strings(items));
                *reader = trial;
                return Ok(value);
            }
            // EMPTY/NULL 带数组标志：原生改道 BSTR 数组路径，语义未定，
            // 本读取器拒绝（较原生更严，无字节佐证）。
            0 | 1 => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Variant::Read 值 offset 0x{start:x}: VT_EMPTY/NULL 带 VT_ARRAY 标志 \
                         （原生改道 BSTR 数组路径，SafeArrayCreate(VT_EMPTY) 语义未定），不支持"
                    ),
                ));
            }
            // 对象类型不允许数组（原生 switch 无 9/13 的数组路径）。
            9 | 13 => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Variant::Read 值 offset 0x{start:x}: VT_DISPATCH/UNKNOWN（0x{raw_vt:04x}）\
                         为对象分支，原生经全局 LoadObject 走 16 字节类 GUID/零 GUID 的 COM \
                         上下文（非对象流对象头），未闭合，显式拒绝"
                    ),
                ));
            }
            w => {
                let element_bytes = match fixed_width {
                    Some(n) => n,
                    None => {
                        return Err(invalid(format!(
                            "数组 VARTYPE {w}（原始 0x{raw_vt:04x}）无对应载荷分支（原生 \
                             default → 0x80004005）"
                        )));
                    }
                };
                let count_offset = trial.position();
                let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
                if count < 0 {
                    return Err(invalid(format!(
                        "数组计数 offset 0x{count_offset:x}: 计数为负数（{count}）；原生负计数\
                         使 SafeArrayCreate 失败返回 0x80004005"
                    )));
                }
                let count_usize = usize::try_from(count).expect("计数已确认为非负");
                let total = count_usize
                    .checked_mul(element_bytes)
                    .ok_or_else(|| invalid("数组计数 × 元素字节数溢出".to_string()))?;
                if total > max_bytes {
                    return Err(invalid(format!(
                        "数组计数 {count} × 元素字节数 {element_bytes} 超过资源限额 {max_bytes}"
                    )));
                }
                let raw = trial.read_bytes(total)?.to_vec();
                let value = VariantClassValue10_1::Array(ClassVariantArray10_1::Fixed {
                    vartype: vt,
                    element_bytes,
                    raw,
                });
                *reader = trial;
                return Ok(value);
            }
        }
    }

    // 非数组分支。
    let value = match vt {
        0 => {
            // EMPTY：无载荷（原生仍完成收尾，返回 S_OK）。
            *reader = trial;
            return Ok(VariantClassValue10_1::Empty);
        }
        1 => {
            *reader = trial;
            return Ok(VariantClassValue10_1::Null);
        }
        8 => VariantClassValue10_1::Bstr(read_string_10_1(&mut trial, max_bytes)?),
        9 | 13 => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Variant::Read 值 offset 0x{start:x}: VT_DISPATCH/UNKNOWN（0x{raw_vt:04x}）\
                     为对象分支，原生经全局 LoadObject 走 16 字节类 GUID/零 GUID 的 COM 上下文\
                     （非对象流对象头），未闭合，显式拒绝"
                ),
            ));
        }
        w => {
            let width = match fixed_width {
                Some(n) => n,
                None => {
                    return Err(invalid(format!(
                        "VARTYPE {w}（原始 0x{raw_vt:04x}）无对应载荷分支（原生 default → \
                         0x80004005）"
                    )));
                }
            };
            let raw = trial.read_bytes(width)?.to_vec();
            match (vt, width) {
                (2, 2) => VariantClassValue10_1::I2(i16::from_le_bytes(raw.try_into().unwrap())),
                (3, 4) => VariantClassValue10_1::I4(i32::from_le_bytes(raw.try_into().unwrap())),
                (4, 4) => VariantClassValue10_1::R4(f32::from_le_bytes(raw.try_into().unwrap())),
                (5, 8) => VariantClassValue10_1::R8(f64::from_le_bytes(raw.try_into().unwrap())),
                (6, 8) => VariantClassValue10_1::Cy(i64::from_le_bytes(raw.try_into().unwrap())),
                (7, 8) => VariantClassValue10_1::Date(f64::from_le_bytes(raw.try_into().unwrap())),
                (10, 4) => {
                    VariantClassValue10_1::Error(u32::from_le_bytes(raw.try_into().unwrap()))
                }
                (11, 2) => VariantClassValue10_1::Bool(u16::from_le_bytes(raw.try_into().unwrap())),
                (16, 1) => VariantClassValue10_1::I1(raw[0] as i8),
                (17, 1) => VariantClassValue10_1::Ui1(raw[0]),
                (18, 2) => VariantClassValue10_1::Ui2(u16::from_le_bytes(raw.try_into().unwrap())),
                (19, 4) => VariantClassValue10_1::Ui4(u32::from_le_bytes(raw.try_into().unwrap())),
                _ => unreachable!("宽度表与分支一一对应"),
            }
        }
    };
    *reader = trial;
    Ok(value)
}

/// 读取 `ReadString`（VariantStream/OLE 风格）布局的一个字符串元素：
/// 4 字节 UTF-16LE 码元数 `n`；`n == 0` 时无载荷（空串），否则读
/// `n*2+2` 字节并要求末尾 UTF-16LE 终止零。这是 `native_string`
/// 布局（n 为含终止零的字节长度）之外的另一种原生字符串布局，
/// 仅用于 [`ClassVariantArray10_1::Strings`] 元素。
///
/// `max_bytes` 约束 `n*2+2` 载荷（`usize::MAX` 不设限）；失败时不推进
/// 游标。
fn read_variant_bstr_array_element_10_8(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<String> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("ReadString 元素 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();
    let units = trial.read_u32_le()?;
    if units == 0 {
        *reader = trial;
        return Ok(String::new());
    }
    let payload_length = units
        .checked_mul(2)
        .and_then(|length| length.checked_add(2))
        .and_then(|length| usize::try_from(length).ok())
        .ok_or_else(|| invalid("码元数 × 2 + 2 溢出".to_string()))?;
    if payload_length > max_bytes {
        return Err(invalid(format!(
            "声明载荷 {payload_length} 字节超过资源限额 {max_bytes}"
        )));
    }
    let payload = trial.read_bytes(payload_length)?;
    if payload[payload_length - 2..] != [0, 0] {
        return Err(invalid("载荷末尾不是 UTF-16LE 终止零".to_string()));
    }
    let value = Reader::new(&payload[..payload_length - 2])
        .read_utf16_le(payload_length - 2)
        .map_err(|error| invalid(format!("非法 UTF-16LE 编码：{error}")))?;
    *reader = trial;
    Ok(value)
}

/// EditTemplate 字段默认值表的一项：[String 键][Variant 值]。
#[derive(Debug, Clone, PartialEq)]
pub struct EditTemplateField10_1 {
    /// 字段名（AfCore String::Read 布局；样本为要素类字段名）。
    pub key: String,
    /// 字段模板默认值。
    pub value: VariantClassValue10_1,
}

/// EditTemplate（FeatureLayer.dll CEditTemplate::Load `0x6f6ce1e0`）对象
/// 的已确认字段（体完整闭合：全部字段有读侧/写侧对称证据）。
#[derive(Debug, Clone, PartialEq)]
pub struct EditTemplateBody10_1 {
    /// 对象头键（由调用者在分类新对象时按「登记先于持久化」登记；本
    /// 读取器不产生嵌套对象条目）。
    pub key: u32,
    /// 体首 int16 版本的无符号位模式（有符号 >2 拒绝；10.1 Save 恒写 2）。
    pub version: u16,
    /// 第一 String：IEditTemplate::Name（持久化接口 this+0x08）。
    pub name: String,
    /// 第二 String：IEditTemplate::Description（持久化接口 this+0x0c）。
    pub string_b: String,
    /// 16 字节原始 GUID：IEditTemplate::Tool（持久化接口 this+0x18），原样保留。
    pub raw_guid: [u8; 16],
    /// 第 4 字段 int32 计数原始值（负数拒绝，较原生更严）。
    pub field_count: i32,
    /// 已消费的 [String 键][Variant 值] 项。
    pub fields: Vec<EditTemplateField10_1>,
    /// 版本 ≥2 的第三 String：IEditTemplate::Tags（持久化接口 this+0x10）；
    /// 版本 <2 时原生不读流，为 `None`。
    pub tail_string: Option<String>,
    /// 体之后的剩余字节数（体无外部长度；正常输入应为 0）。
    pub remaining_bytes: usize,
}

impl EditTemplateBody10_1 {
    /// 返回 IEditTemplate::Description 对应的持久化字符串。
    pub fn description(&self) -> &str {
        &self.string_b
    }

    /// 返回 IEditTemplate::Tool 的原始 GUID 字节，顺序与流一致。
    pub fn tool_guid_bytes(&self) -> &[u8; 16] {
        &self.raw_guid
    }

    /// 返回 IEditTemplate::Tags；旧版本未写入时为 `None`。
    pub fn tags(&self) -> Option<&str> {
        self.tail_string.as_deref()
    }
}

/// 从对象头之后的位置读取 10.1 EditTemplate 对象体，失败时不推进游标、
/// 不修改登记表。
///
/// 起点约定：调用者已读取对象头、比对 [`EDIT_TEMPLATE_CLASS_ID`] 并取得
/// 键 `key`。字段序见模块文档。`max_bytes` 约束三个字符串、循环计数
/// （计数 × 最小条目字节数）与数组载荷。循环内的 Variant 对象分支
/// （VT_DISPATCH/UNKNOWN）直接报不支持错误（整体失败、游标与登记表不动）；
/// 截断、版本超限、负计数同为错误。
pub fn read_edit_template_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
    key: u32,
) -> io::Result<EditTemplateBody10_1> {
    let mut trial = reader.clone();
    let entry_offset = trial.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("EditTemplate 对象体 offset 0x{entry_offset:x}: {reason}"),
        )
    };
    let staged_table = table.clone();

    // 1. 版本 short：有符号 >2 拒绝；负值一并拒绝（较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_BODY_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "架构版本 {version} 超出 EditTemplate::Load 接受范围（有符号 >2 拒绝 0x80004005；\
             Save 版本表 DAT_6f77c02c = [0xFFFF×4, 2]，10.0/10.1 恒写 2；负值位模式原生通过，\
             此处一并拒绝）"
        )));
    }

    // 2. 两个字符串。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let string_b = read_string_10_1(&mut trial, max_bytes)?;

    // 3. 16 字节原始 GUID（原样保留）。
    let raw_guid = trial.read_bytes(16)?.try_into().expect("固定 16 字节");

    // 4. int32 计数 + [String 键][Variant 值] 循环。原生负数/零计数跳过
    //    循环；本读取器拒绝负数（避免静默错位，与同族读取器口径一致）。
    let count_offset = trial.position();
    let field_count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if field_count < 0 {
        return Err(invalid(format!(
            "字段计数 offset 0x{count_offset:x}: 计数为负数（{field_count}）；原生读取仅正数\
             进入循环（负数跳过），本读取器拒绝以避免静默错位"
        )));
    }
    let count_usize = usize::try_from(field_count).expect("计数已确认为非负");
    let min_total = count_usize
        .checked_mul(MIN_FIELD_ENTRY_BYTES)
        .ok_or_else(|| invalid("字段计数 × 最小条目字节数溢出".to_string()))?;
    if min_total > max_bytes {
        return Err(invalid(format!(
            "字段计数 offset 0x{count_offset:x}: 声明计数 {field_count} × 最小条目字节数 \
             {MIN_FIELD_ENTRY_BYTES} 超过资源限额 {max_bytes}，在条目读取之前拒绝"
        )));
    }
    let mut fields = Vec::with_capacity(count_usize.min(4096));
    for _ in 0..count_usize {
        let key = read_string_10_1(&mut trial, max_bytes)?;
        let value = read_variant_class_value_10_1(&mut trial, max_bytes)?;
        fields.push(EditTemplateField10_1 { key, value });
    }

    // 5. 版本 ≥2：第三 String；版本 <2 原生不读流（Save 恒写，10.1 不产生
    //    版本 <2 的流）。
    let tail_string = if version >= 2 {
        Some(read_string_10_1(&mut trial, max_bytes)?)
    } else {
        None
    };

    let result = EditTemplateBody10_1 {
        key,
        version,
        name,
        string_b,
        raw_guid,
        field_count,
        fields,
        tail_string,
        remaining_bytes: trial.remaining(),
    };
    // 本体不产生嵌套对象条目（Variant 对象分支显式拒绝），staged_table
    // 与 table 相同；保留克隆提交形态以满足「失败不修改登记表」的统一
    // 口径。
    *table = staged_table;
    *reader = trial;
    Ok(result)
}
