//! ArcMap 10.1 Geometry.dll 几何 coclass（esriGeometry）对象体的严格读取
//! 子集：Polyline（TextElement 的 IGeometry 槽样本实现类）。
//!
//! 证据来源：`docs/arcmap-10.1-geometry.md`（本轮研究记录）。定位链在
//! Geometry.dll 10.1.0.3035 内三方互证闭环（全部静态反编译/汇编核对，未运行
//! 目标软件）：esri.clsid.xml 声明 Geometry.dll 模块类 `Polyline`
//! `{30707210-52D5-11D0-A8F2-00608C85EDE5}` → `.rdata` 常量 `0x71a1187c` 与
//! 该 CLSID 字节逐一同 → `ESRI::Polyline` RTTI TypeDescriptor
//! `0x71a6a6a4`（名称串 `.?AVPolyline@ESRI@@`）→ COL `0x71a3d128` →
//! 持久化虚表 `0x71a22bd4`（COL 指针位于虚表前 4 字节；构造器 `0x71943290`
//! 把它装到对象 `+0x2c` 槽）→ GetClassID `0x719402d0`（`movq` 双 8 字节复制
//! 同一常量）→ Load `0x7192db20`、Save `0x7192dcb0`。虚表形状
//! `[QI/AddRef/Release/GetClassID@+0x0c/IsDirty@+0x10/Load@+0x14/Save@+0x18/
//! GetSizeMax@+0x1c]`，其中 IsDirty 与 Envelope（见
//! `docs/arcmap-10.1-spatial-reference.md`）共享同一恒返 0 函数。
//!
//! ## 包装字段序（Load `0x7192db20` 反编译逐行核对，确定的）
//!
//! 1. int16 版本 short：AfStreamRead 读 2 字节；`(short)版本 > 2` 拒绝
//!    `0x80004005`（经 `FUN_719f1570` 记录，有符号比较，负值位模式原生通
//!    过，本读取器一并拒绝，较原生更严）；AfStreamRead 返回 1（S_FALSE）
//!    时原生走「空流」捷径（调内嵌接口 +0x20 槽后原样返回），样本未覆盖，
//!    本读取器不实现；
//! 2. int32 声明长度：AfStreamRead 读 4 字节；
//! 3. 该长度的记录字节块（原生 malloc 后 AfStreamRead 读入，无长度上限，
//!    分配失败记 `0x8007000e`；本读取器以 `max_bytes` 设限，较原生更严）
//!    交给内嵌接口（对象 `+0x24` 槽，持久化 this−0x8 视角）`+0x38` 方法
//!    导入，导入标志恒 `0x100`；
//! 4. 版本 > 1：LoadObject 请求 IID `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`
//!    （ISpatialReference，与 Map::Load 第 4 步、Envelope 版本 3 槽同值），
//!    装入对象 `+0x10`（持久化 this−0x1c 视角）`+0x18` 槽。
//!
//! Save（`0x7192dcb0`，反编译核对）：`AfStreamGetDocumentVersion` → 写 2 字
//! 节**固定常量 `02 00`**（`.rdata 0x71a11e50`，不走版本表，10.1 恒写版本
//! 2）→ 导出记录（尺寸查询 + 填充两次调 `+0x10`/`+0x34` 槽，填充标志 =
//! docVer > 1 ? `0x400` : 0）→ 写 int32 长度 → 写记录字节 → SaveObject 空
//! 间参考。与 Load 版本 2 路径逐字段对称。
//!
//! ## 记录字节块内部布局（ShapePolyline::Parse `0x7197fc50` 反编译，确定的）
//!
//! 导入入口按对象类目构造 `ESRI::ShapePolyline` 栈对象（自身类型标记写 3；
//! Polygon 分支写 5）并以其虚表 `+0x94` 槽解析缓冲区，随后把解析结果移交
//! 几何内核（对象 `+0xc8` 内嵌指针的虚表 `+0x1cc` 槽，运行期装配，静态未定
//! 位）。导入标志经 `& 0xfffffeff` 清除 `0x100` 位后恒为 0，因此 Z/M/段描
//! 述等可选段在读路径上不可达。布局（小端）：
//!
//! | 偏移 | 字段 | 说明 |
//! | --- | --- | --- |
//! | 0 | int32 类型 | 内部 shape 类别；Polyline 样本 16/16 = 3，0 = 空几何 |
//! | 4 | 4×f64 包络 | `WKSEnvelope::PutCoords(XMin, YMin, XMax, YMax)` 调用序 |
//! | 36 | int32 路径数 | < 0 原生拒 `0x80040253` |
//! | 40 | int32 点数 | < 0 拒 `0x80040253`；== 1 拒 `0x80040221`（折线至少两点） |
//! | 44 | 路径数×int32 | 路径起始点序号数组（样本恒 `[0]`） |
//! | 44+4×路径数 | 点数×(f64,f64) | 二维点坐标，包络由写侧重算（读侧不校验） |
//!
//! 类型 0（空几何）记录仅 4 字节、无任何后续字段。原生解析不校验类型值、
//! 也不校验计数与声明长度的一致性（越界即未定义行为）；本读取器对两者显式
//! 校验并拒绝，较原生更严。可选段（Load 标志恒 0 故不可达，仅记录开放）：
//! 标志 `0x80000000` → Z 域 min/max 对 + 每点 Z；`0x40000000` → M 域对 +
//! 每点 M；`0x20000000` → 段描述数组（段类型 1 = Line、4 = CircularArc、
//! 5 = BezierCurve，仅描述条目，非流内对象）；`0x10000000` → 每点 4 字节附
//! 加数组。坐标 < −1e38 时原生钳位为 `DAT_71a66f20` 占位值，本读取器原样保
//! 留位模式不钳位（与 Envelope 读取器同约定）。
//!
//! ## 同族类（wrapper 骨架闭合，记录布局未开放）
//!
//! 同一 wrapper 骨架 `[int16 版本 ≤2][int32 长度][长度字节记录][版本 >1:
//! LoadObject ISpatialReference]` 由族内多类共享：Polygon 与 Polyline 共用
//! 同一 Load `0x7192db20` / Save `0x7192dcb0`；Point `0x7191c1a0`（栈缓冲
//! 版，声明长度 > 0x50 原生拒 `0x80004005`）与 Multipoint `0x718ff9c0`
//! （malloc 版）同构；Path `0x719089e0`、Ring `0x7195e110`、BezierCurve
//! `0x717fea60`、EllipticArc `0x7184cbb0` 同形（Save 槽为空桩 `0x71973ab0`
//! 的类身份证据较弱，见研究文档）。各类差异在记录字节块内部（内核解析器的
//! 形状分支），样本仅覆盖 Polyline；本读取器仅分派 Polyline，其余类返回
//! [`ErrorKind::Unsupported`]。
//!
//! ## 样本对位（辅助观察）
//!
//! 仓库参考样本 `/Maps` 流中 Polyline 对象头 16 处（键 1618、1855、1862、
//! …、2177），体全部为版本 2 + 80 字节记录（类型 3、路径数 1、点数 2、
//! 路径起始 `[0]`、2 个二维点）+ 16 字节全零空标记（版本 2 的 LoadObject
//! ISpatialReference 空引用）。逐字节走查脚本（temp 内，未入库）16/16 吻
//! 合；实例 1（键 1618 @`0x18724`）之后依次为 TextElement 尾段 `(0, -1)`
//! （与 TextElement::Save「恒写 −1」互证）、ElementCollection 每元素交错
//! 值 `(34, -1, 0)`、容器级注记块与下一个对象头。样本吻合为辅助观察，不
//! 构成对读取器正确性的验证。
//!
//! 调用约定与事务性：调用者已用
//! [`crate::storage::object_stream::read_header_10_1`] 读取对象头并按类标识
//! 分派，从对象头之后的位置传入 Reader；本模块不重复读取对象头。全部字段
//! 在 trial 游标与 trial 登记表上读取，完整成功才提交；任何失败不推进调用
//! 者游标、不修改登记表。`max_bytes` 约束记录声明长度（先检查后读取），并
//! 原样转发给嵌套投影坐标系对象体；`usize::MAX` 表示不额外设限。这是仅覆
//! 盖上述布局的严格安全子集：不代表完整原生 Load 行为，通过也不代表 MXD
//! 格式已验证。未接入 MXD 解析主链，人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见研究文档）：记录类型值的语义（内部 shape 类别与
//! esriGeometryType 的映射）仅候选；嵌套空间参考仅支持投影坐标系，其余类
//! 无样本与分派证据；版本 0/1、S_FALSE 捷径、可选段（Z/M/段描述/附加数
//! 组）无样本佐证；Point/Polygon/Multipoint/Path/Ring 等同族类的记录内部
//! 布局未开放；`ESRI::ShapePolyline::Parse` 对 < −1e38 坐标的钳位本读取器
//! 未实现（原样保留）；全部结论未运行目标软件验证。

use crate::objects::envelope::SpatialReferenceEntry10_1;
use crate::objects::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, read_projected_coordinate_system_10_1,
};
use crate::storage::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// Polyline::Load（`0x7192db20`）接受的版本上限：`(short)版本 > 2` 拒绝
/// `0x80004005`（有符号）；Save 侧写固定常量 2，样本 16/16 为 2。
const MAX_POLYLINE_VERSION: i16 = 2;

/// 记录空几何布局的固定长度：仅 4 字节类型字段（`ShapePolyline::Parse`
/// 对类型 0 提前返回，`*consumed = 4`）。
const EMPTY_RECORD_LEN: usize = 4;

/// 记录非空布局的固定头部：4 字节类型 + 32 字节包络 + 2×int32 计数。
const FIXED_RECORD_HEADER: usize = 44;

/// Polyline 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{30707210-52D5-11D0-A8F2-00608C85EDE5}`。证据：
/// esri.clsid.xml 声明（模块 Geometry.dll，progID `esriGeometry.Polyline`）；
/// Geometry.dll `.rdata` 常量 `0x71a1187c` 与 GetClassID `0x719402d0` 的
/// movq 复制；仓库参考样本 /Maps 流 16 处对象头与本常量逐字节相同（辅助
/// 观察）。
pub const POLYLINE_CLASS_ID: [u8; 16] = [
    0x10, 0x72, 0x70, 0x30, 0xd5, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85, 0xed, 0xe5,
];

/// 记录字节块的解析结果（`ShapePolyline::Parse` 布局，Load 标志恒 0）。
#[derive(Debug, Clone, PartialEq)]
pub struct PolylineRecord10_1 {
    /// 记录首 4 字节类型，原样保留。
    ///
    /// 候选：内部 shape 类别（`ShapePolyline` 自身标记 3、`ShapePolygon` 5、
    /// 0 = 空几何），与 esriGeometryType 枚举的映射未闭合；样本 16/16 为 3。
    /// 原生不校验该值，本读取器同样不校验。
    pub kind: u32,
    /// 4 个小端 double，`WKSEnvelope::PutCoords(XMin, YMin, XMax, YMax)` 的
    /// 出参序（与 [`crate::objects::envelope`] 的字段序同源）。仅原样保留
    /// 位模式，不做 NaN、次序或范围判断（原生读侧不校验，样本 16/16 满足
    /// d0 ≤ d2、d1 ≤ d3 且各点在包络内）。
    pub envelope: [f64; 4],
    /// 路径起始点序号数组（路径数在流内先于点数声明；样本恒 `[0]`）。原生
    /// 负值拒 `0x80040253`，本读取器一并拒绝（u32 无负值位模式）。
    pub path_starts: Vec<u32>,
    /// 二维点坐标（点数 = 记录长度按布局推导；样本恒 2）。原生拒绝单点折
    /// 线（`0x80040221`），本读取器同拒；坐标原样保留位模式、不钳位。
    pub points: Vec<[f64; 2]>,
}

/// 10.1 Polyline 对象体（`IPersistStream::Load` 版本 2 布局）。
#[derive(Debug, Clone, PartialEq)]
pub struct PolylineBody10_1 {
    /// 2 字节版本 short 的无符号位模式。
    ///
    /// 原生有符号 `(short)版本 > 2` 拒绝 `0x80004005`；负值位模式原生通过
    /// （有符号比较），本读取器一并拒绝，较原生更严。Save 侧写固定常量
    /// 2；版本 0/1 原生按「记录 + 无空间参考」处理（无样本，合成测试覆盖）。
    pub version: u16,
    /// 记录字节块的解析结果。类型 0 空几何时 [`PolylineRecord10_1`] 各集合
    /// 字段为空。
    pub record: PolylineRecord10_1,
    /// 版本 > 1 的嵌套空间参考条目；版本 0/1 无该段（`None`）。
    ///
    /// `Some([`SpatialReferenceEntry10_1::Null`])` 是版本 2 流内 16 字节全
    /// 零空引用标记（样本 16/16），`None` 表示该版本分支没有此段。
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
}

/// 按调用者已读取的对象头类标识分派几何体读取。
///
/// `class_id` 必须是调用方从对象头取得的 16 字节类标识；起点为对象头之后的
/// 第一字节。当前仅闭合 Polyline；同族其他类（Point/Polygon/Multipoint/
/// Path/Ring/BezierCurve/EllipticArc/CircularArc）返回
/// [`ErrorKind::Unsupported`]（wrapper 骨架已闭合但记录内部布局无样本佐证，
/// 见模块文档与研究文档），游标不推进、登记表不修改。
pub fn read_geometry_body_of_class_10_1(
    class_id: &[u8; 16],
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PolylineBody10_1> {
    if *class_id == POLYLINE_CLASS_ID {
        read_polyline_body_10_1(reader, max_bytes, table)
    } else {
        let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
        Err(io::Error::new(
            ErrorKind::Unsupported,
            format!("几何类 {id} 不在本读取器的闭合清单（仅 Polyline）；同族类记录内部布局未开放"),
        ))
    }
}

/// 从对象头之后的位置读取 10.1 Polyline 对象体的严格子集。
///
/// 字段序与证据见模块文档。`max_bytes` 约束记录声明长度（在任何读取与分配
/// 之前检查）并原样转发给嵌套投影坐标系对象体；原生无此限额（malloc 不设
/// 上限），本读取器设限较原生更严。任何失败不推进游标、不修改登记表；错
/// 误位置相对于传入 Reader，调用方需另外记录流和对象身份。
///
/// 声明长度与布局常量不一致、点数为 1、路径数或点数为负位模式时返回
/// [`ErrorKind::InvalidData`]（原生或拒绝同型输入、或存在越界未定义行为，
/// 本读取器一律显式拒绝）。这是严格安全子集，通过不代表 MXD 格式已验证。
pub fn read_polyline_body_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<PolylineBody10_1> {
    let start = reader.position();
    let invalid = |reason: String| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Polyline 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    // trial 游标 + trial 登记表：嵌套空间参考条目在其对象体之前登记新键，
    // 失败必须整体回滚（与 envelope 读取器同约定）。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. int16 版本 short：原生有符号 >2 拒绝；负值一并拒绝（较原生更严）。
    let version = trial.read_u16_le()?;
    if (version as i16) > MAX_POLYLINE_VERSION || (version as i16) < 0 {
        return Err(invalid(format!(
            "版本位模式 {version}（0x{version:04x}）超出原生接受范围 0..=2（Load 0x7192db20 \
             有符号比较 >2 拒绝 0x80004005；负值位模式原生通过，本读取器一并拒绝）"
        )));
    }

    // 2. int32 声明长度；先限额后读取（原生 malloc 无上限，本读取器设限）。
    let declared = trial.read_u32_le()?;
    let record_len = declared as usize;
    if record_len > max_bytes {
        return Err(invalid(format!(
            "记录声明长度 {record_len} 超过资源限额 {max_bytes}（原生无长度上限，本读取器 \
             设限较原生更严）"
        )));
    }

    // 3. 记录字节块：在有界子窗口上解析（长度受步骤 2 限额约束，read_region
    //    失败即截断错误）。布局见模块文档；原生不校验计数与长度的一致性，
    //    本读取器显式校验（较原生更严）。
    let record_offset = trial.position();
    let record = {
        let mut window = trial
            .read_region(record_len)
            .map_err(|error| invalid(format!("记录声明长度 {record_len} 超出剩余字节：{error}")))?;
        read_polyline_record_10_1(&mut window, record_len).map_err(|reason| invalid(reason))?
    };
    debug_assert_eq!(trial.position(), record_offset + record_len);

    // 4. 版本 > 1：嵌套空间参考条目（类身份完全由流内对象头决定）。
    let spatial_reference = if (version as i16) > 1 {
        let nested_offset = trial.position();
        match read_object_entry_10_1(&mut trial, &mut trial_table)? {
            ObjectEntry10_1::Null => Some(SpatialReferenceEntry10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                Some(SpatialReferenceEntry10_1::BackReference { class_id, key })
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
            {
                let body = read_projected_coordinate_system_10_1(&mut trial, max_bytes)?;
                Some(SpatialReferenceEntry10_1::Projected { key, body })
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 严格子集只认投影坐标系嵌套（与 envelope 读取器同语义）：
                // 嵌套对象体无外部长度字段，无法跳过未知类继续读取；显式
                // Unsupported，游标与登记表都不提交。
                let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Polyline 对象体嵌套空间参考 offset 0x{nested_offset:x}: 类标识 {id}\
                         （键 {key}）不是投影坐标系；本读取器仅支持投影坐标系嵌套，且对象体\
                         无外部长度字段，无法跳过未知类继续读取"
                    ),
                ));
            }
        }
    } else {
        None
    };

    let result = PolylineBody10_1 {
        version,
        record,
        spatial_reference,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 在有界窗口上解析记录字节块（`ShapePolyline::Parse` 标志 0 布局）。
///
/// `record_len` 为包装声明的长度；空几何（类型 0）要求恰为 4 字节，非空布
/// 局要求 `44 + 路径数×4 + 点数×16` 严格相等。返回 `Err(String)` 表示布局
/// 违规（由调用方包装为 InvalidData）。
fn read_polyline_record_10_1(
    window: &mut Reader<'_>,
    record_len: usize,
) -> Result<PolylineRecord10_1, String> {
    if record_len == EMPTY_RECORD_LEN {
        let kind = window.read_u32_le().map_err(|error| error.to_string())?;
        if kind != 0 {
            return Err(format!(
                "空几何记录（长度 4）的类型位模式 {kind} 非 0；原生对类型 0 之外的值会继续\
                 按完整布局解析（越界未定义行为），本读取器拒绝"
            ));
        }
        return Ok(PolylineRecord10_1 {
            kind,
            envelope: [0.0; 4],
            path_starts: Vec::new(),
            points: Vec::new(),
        });
    }
    if record_len < FIXED_RECORD_HEADER {
        return Err(format!(
            "记录声明长度 {record_len} 不足以容纳非空布局头部 44 字节"
        ));
    }
    let kind = window.read_u32_le().map_err(|error| error.to_string())?;
    let mut envelope = [0.0; 4];
    for slot in &mut envelope {
        *slot = window.read_f64_le().map_err(|error| error.to_string())?;
    }
    let path_count = window.read_u32_le().map_err(|error| error.to_string())?;
    let point_count = window.read_u32_le().map_err(|error| error.to_string())?;
    let expected = FIXED_RECORD_HEADER + path_count as usize * 4 + point_count as usize * 16;
    if expected != record_len {
        return Err(format!(
            "记录长度 {record_len} 与布局常量（44 + 路径数 {path_count}×4 + 点数 \
             {point_count}×16 = {expected}）不一致；原生不校验此一致性（越界未定义行为），\
             本读取器拒绝"
        ));
    }
    if point_count == 1 {
        return Err(format!(
            "点数 1：原生 ShapePolyline::Parse 拒绝单点折线（0x80040221，折线至少两点），\
             本读取器同拒"
        ));
    }
    let mut path_starts = Vec::new();
    for _ in 0..path_count {
        path_starts.push(window.read_u32_le().map_err(|error| error.to_string())?);
    }
    let mut points = Vec::new();
    for _ in 0..point_count {
        let x = window.read_f64_le().map_err(|error| error.to_string())?;
        let y = window.read_f64_le().map_err(|error| error.to_string())?;
        points.push([x, y]);
    }
    debug_assert_eq!(window.remaining(), 0);
    Ok(PolylineRecord10_1 {
        kind,
        envelope,
        path_starts,
        points,
    })
}
