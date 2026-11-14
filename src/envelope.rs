//! ArcMap 10.1 Envelope（范围）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-spatial-reference.md`「Envelope::Save、
//! WKSEnvelope 写方向与 Map 显示范围闭合（2026-11-14 写方向取证轮）」（重点
//! 「Envelope::Save 与 Load 本体」「WKSEnvelope 字段序」，证据等级「确定的」）
//! 与 `docs/arcmap-10.1-map-object.md`「Envelope 对象：图层范围与图层坐标系
//! 的载体」。布局证据为 Geometry.dll 10.1.0.3035 的 Save `0x71869da0` 与
//! Load `0x71869b60` 反编译；类标识由 GetClassID `0x71864170` 写死。字段序
//! `[XMin, YMin, XMax, YMax]` 由写方向闭合：Save 把 AfCore 导出
//! `WKSEnvelope::QueryCoords`（序号 1722，`0x68abf530`）的四个出参按序连续
//! 写出，访问器 XMin→+0x00 / YMin→+0x08 / XMax→+0x10 / YMax→+0x18
//! （`0x68abe9d0` / `0x68abea70` / `0x68abea20` / `0x68abeac0`）逐指令绑定。
//!
//! 对象体布局（Load `0x71869b60` 的版本分支）：
//!
//! 1. 2 字节版本 short：>3 拒绝（原生 `FUN_719f1570` 记 `0x80004005`）；
//!    版本 1 只读 32 字节（无 int32、无空间参考段）；版本 2/3 再读 int32；
//! 2. 32 字节 = 4 个小端 double，字段序 `[XMin, YMin, XMax, YMax]`（写方向
//!    确定，见上）；
//! 3. int32 原样保留（RTTI 命名 IAttributeAwareness 候选，语义未定；样本
//!    25/25 恒 0），其低字节 bit6(0x40) → 追加 2 个 double（Z 域候选）、
//!    bit7(0x80) → 再追加 2 个 double（M 域候选），Z 在 M 前；
//! 4. 仅版本 3：LoadObject 空间参考子对象（请求 IID
//!    `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`，与 Map::Load 第 4 步同值）。
//!    条目经 [`crate::object_refs::read_object_entry_10_1`] 分类：全零类标识
//!    → 空引用；键已见 → 20 字节回引（无体）；新键且类标识为投影坐标系 →
//!    读嵌套 PCS 体；新键非投影坐标系 → 不支持（嵌套体无外部长度字段，
//!    无法跳过）。
//!
//! 写侧恒写版本 3 完整布局（Save 写静态常量 3，不走 AfStreamGetSaveVersion
//! 版本表）；读侧按 1/2/3 分支兼容旧版——与 PCS 读写结论同构。
//!
//! 调用约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取对象
//! 头并比对 [`ENVELOPE_CLASS_ID`] 确认类身份，从对象头之后的位置传入
//! Reader；本模块不重复读取对象头。图层坐标系宿主于图层体 Envelope 的第
//! 4 段嵌套空间参考（证据文档「Envelope 对象」节），字段归属由调用者负责。
//!
//! 样本字节核对（辅助观察，不作测试依赖）：`samples/测试底图.mxd` 的
//! `/Maps` 导出中 25 个 Envelope 全部版本 3、int32=0（无 Z/M 追加段）。键
//! 74 头 @`/Maps:0xedd`：体 = `03 00` + 4 double `[37358416.1434,
//! 2493488.6084, 37423696.7434, 2550958.6584]` + int32 0 + 对 Map PCS 键
//! 30 的 20 字节回引头；键 80 头 @`0x122b`：体尾内嵌新 PCS 键 81，PCS 体
//! 为版本 6 + 455 字节 WKT。样本吻合为辅助观察，不构成对读取器正确性的
//! 验证。
//!
//! 这是仅覆盖上述分支的严格安全子集：要求完整输入，任何失败不推进调用者
//! 游标、不修改登记表；未接入 MXD 解析，人工目视验收未做，不得声称已验证。
//!
//! 未解决问题（详见证据文档「未解决事项」）：int32 与 Z/M 追加段语义按
//! RTTI 与结构命名，未经目标软件行为验证；两个 Envelope 实例的字段语义
//! （数据全范围 vs 当前范围候选）与三个 Envelope（键 74/80/151）由哪个
//! LoadObject 装入仍开放；版本 0 的字段分支未在证据中闭合（证据仅覆盖
//! 版本 1 与版本 2/3 两组分支，本读取器显式拒绝，较原生「仅拒绝 >3」更
//! 严）；版本 short 比较的符号性未闭合（按无符号处理，负值位模式一并
//! 拒绝，与 map_object 的版本处理一致）；嵌套空间参考仅支持投影坐标系，
//! GeographicCoordinateSystem / UnknownCoordinateSystem 等其他空间参考类
//! 无样本与分派证据；回引与新对象在流内不可局部区分（沿用
//! [`crate::object_refs`] 的登记表语义）。

use crate::object_refs::{ObjectEntry10_1, RefTable10_1, read_object_entry_10_1};
use crate::reader::Reader;
use crate::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use std::io::{self, ErrorKind};

/// Envelope 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{30707212-52D5-11D0-A8F2-00608C85EDE5}`。证据：
/// Geometry.dll 10.1.0.3035 的 Envelope 持久化接口 GetClassID `0x71864170`
/// 写死该 CLSID（写方向取证轮，证据等级「确定的」）；esri.clsid.xml 声明
/// Geometry.dll 模块类名 `Envelope`。仓库参考样本 `/Maps` 中 25 个 Envelope
/// 对象头的类标识与本常量逐字节相同（辅助观察）。
pub const ENVELOPE_CLASS_ID: [u8; 16] = [
    0x12, 0x72, 0x70, 0x30, 0xd5, 0x52, 0xd0, 0x11, 0xa8, 0xf2, 0x00, 0x60, 0x8c, 0x85, 0xed, 0xe5,
];

/// Envelope 对象体第 4 段（仅版本 3）嵌套空间参考条目的分类结果。
///
/// 分类完全依赖调用者传入的登记表状态：回引与新对象条目在流内同构、不可
/// 局部区分，空表会把实际的回引字节按新对象分类（沿用
/// [`crate::object_refs`] 保留的原生语义）。Load 侧请求 IID
/// `{4434D680-F4DA-11D1-BCA6-0000F875BCCE}`（与 Map::Load 第 4 步同值），
/// 对象类身份由流内 16 字节类标识决定。
#[derive(Debug, Clone, PartialEq)]
pub enum SpatialReferenceEntry10_1 {
    /// 16 字节全零类标识：空引用标记，无键、无对象体（沿用对象头读取器的
    /// S_FALSE 约定，仅消耗 16 字节）。
    Null,
    /// 键已在本会话登记过：20 字节回引条目（16 字节类标识 + 4 字节既有键），
    /// 其后没有对象体。类标识原样保留；样本中最常见的回引目标是 Map 的
    /// PCS 键 30。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 未见过的新键且类标识为投影坐标系：已读取完整嵌套 PCS 对象体。
    ///
    /// 新键在对象体读取之前登记（登记先于持久化的读侧对偶），登记随本次
    /// 调用整体成功才提交到调用者的登记表；后续同键条目将命中回引。
    Projected {
        key: u32,
        body: ProjectedCoordinateSystem10_1,
    },
}

/// 10.1 Envelope::Load（`0x71869b60`）可读取的 Envelope 对象体。
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope10_1 {
    /// 2 字节版本 short 的无符号位模式。
    ///
    /// 原生对 >3 的版本经 `FUN_719f1570` 记 `0x80004005` 并退出；写侧恒写
    /// 静态常量 3，样本 25/25 为 3。比较的符号性未在证据闭合：按无符号解释
    /// 时 0x8000..0xffff 同样大于 3，若原生为有符号比较则负值位模式可能通过
    /// 原生检查，此处一并拒绝（较原生更严，与 map_object 的版本处理一致）。
    /// 版本 0 不属于证据覆盖的任何分支（版本 1 / 版本 2/3），本读取器显式
    /// 拒绝，较原生「仅拒绝 >3」更严。
    pub version: u16,
    /// WKSEnvelope 32 字节的 4 个小端 double，按写方向确定的字段序
    /// `[XMin, YMin, XMax, YMax]` 排列。
    ///
    /// 写方向证据：Save 把 `WKSEnvelope::QueryCoords` 四个出参连续写出，
    /// AfCore 访问器逐指令绑定 XMin→+0x00 / YMin→+0x08 / XMax→+0x10 /
    /// YMax→+0x18。仅原样保留位模式，不做 NaN、次序或范围判断（原生同样
    /// 不校验；样本 25 例 d0<d2、d1<d3 相容）。两个 Envelope 实例中该字段
    /// 的语义（数据全范围 vs 当前范围候选）开放，见模块文档。
    pub coords: [f64; 4],
    /// 版本 2/3 分支的 int32，按无符号位模式原样保留、不解释。
    ///
    /// 原生 Save 写 `this+0xc` 子接口虚表 +0xc 槽的返回值（RTTI 命名
    /// IAttributeAwareness 候选，语义未定）；Load 在其非零时调 `this+0xc`
    /// 接口 +0x10 槽。样本 25/25 恒 0。版本 1 无此字段（`None`）。
    pub attribute_flags: Option<u32>,
    /// Z 域候选 double 对：int32 低字节 bit6(0x40) 置位时追加，仅原样保留。
    ///
    /// 原生 Save 取 `[this+0x58]+8` 与 `+0x20` 两处写出；字段内 min/max
    /// 语义未验证，两值顺序即流内顺序。未置位、版本 1 或不存在 int32 时
    /// 为 `None`。样本 25/25 无 Z/M 追加段。
    pub z_domain: Option<[f64; 2]>,
    /// M 域候选 double 对：int32 低字节 bit7(0x80) 置位时再追加（Z 在 M
    /// 之前），仅原样保留。
    ///
    /// 原生 Save 取 `[this+0x58]+0` 与 `+0x18` 两处写出；语义未验证。未
    /// 置位、版本 1 或不存在 int32 时为 `None`。
    pub m_domain: Option<[f64; 2]>,
    /// 版本 3 分支第 4 段嵌套空间参考条目的分类结果；版本 1/2 为 `None`
    /// （原生不读该段）。
    ///
    /// `Some(Null)` 与 `None` 含义不同：前者是版本 3 流内 16 字节全零空引用
    /// 标记，后者是该版本分支根本没有空间参考段。图层坐标系宿主于此段。
    pub spatial_reference: Option<SpatialReferenceEntry10_1>,
}

/// 从对象头之后的位置读取 10.1 Envelope 对象体的严格子集。
///
/// 起点约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取
/// Envelope 对象头、比对 [`ENVELOPE_CLASS_ID`] 确认类身份，并从对象头之后
/// 的位置传入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：Envelope 本体各字段均为定宽读取、不直接受限额
/// 约束；限额原样转发给嵌套投影坐标系对象体（约束其 WKT 与域字符串长度，
/// 在任何读取与分配之前检查），`usize::MAX` 表示不额外设限。
///
/// `table` 是跨流共享的已见键登记表（对应原生 ObjectStream 读表）：
/// 嵌套空间参考条目经 [`crate::object_refs::read_object_entry_10_1`] 分类，
/// 新键在嵌套对象体读取之前登记到表的试读副本上，调用整体成功才一并提交；
/// 任何失败既不推进游标、也不修改调用者的登记表。
///
/// 嵌套条目为未见过的新键且类标识不是投影坐标系时返回
/// [`ErrorKind::Unsupported`]：嵌套对象体没有外部长度字段，本读取器无法
/// 跳过未知类继续解析，调用者不能把该流继续当作 Envelope 体消费。
///
/// 全部字段在 trial 游标与 trial 登记表上读取，完整成功才提交。错误位置
/// 相对于传入 Reader，调用方需另外记录流和对象身份。这是严格安全子集，
/// 不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_envelope_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
    table: &mut RefTable10_1,
) -> io::Result<Envelope10_1> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Envelope 对象体 offset 0x{start:x}: {reason}"),
        )
    };
    // trial 游标 + trial 登记表：对象条目分类器在返回前登记新键，而嵌套
    // PCS 体在其后才读取，失败必须整体回滚——否则半解析对象的键残留在
    // 登记表中，会把后续同键条目误判为回引。完整成功才把两者一并提交。
    let mut trial = reader.clone();
    let mut trial_table = table.clone();

    // 1. 2 字节版本 short：无符号解释（符号性未闭合，见结构体字段注释）。
    let version = trial.read_u16_le()?;
    if version > 3 {
        return Err(invalid(&format!(
            "版本 {version} 超出原生读取方法的接受范围（>3 记 0x80004005）"
        )));
    }
    if version == 0 {
        return Err(invalid(
            "版本 0 不属于证据覆盖的字段分支（版本 1 / 版本 2/3），无法安全读取后续字节",
        ));
    }

    // 2. 32 字节 WKSEnvelope：4 个小端 double 按写方向序，仅原样保留位模式。
    let mut coords = [0.0; 4];
    for slot in &mut coords {
        *slot = trial.read_f64_le()?;
    }

    // 3. 版本 2/3：int32 原样保留；低字节 bit6/bit7 追加 Z/M double 对。
    // int32 为 0 时两位必然为 0，与原生「非零才调接口再测位」的门等价。
    let attribute_flags = if version >= 2 {
        Some(trial.read_u32_le()?)
    } else {
        None
    };
    let z_domain = read_flagged_pair(&mut trial, attribute_flags, 0x40)?;
    let m_domain = read_flagged_pair(&mut trial, attribute_flags, 0x80)?;

    // 4. 仅版本 3：嵌套空间参考条目（类身份完全由流内对象头决定）。
    let spatial_reference = if version == 3 {
        let nested_offset = trial.position();
        match read_object_entry_10_1(&mut trial, &mut trial_table)? {
            ObjectEntry10_1::Null => Some(SpatialReferenceEntry10_1::Null),
            ObjectEntry10_1::BackReference { class_id, key } => {
                Some(SpatialReferenceEntry10_1::BackReference { class_id, key })
            }
            ObjectEntry10_1::NewObject { class_id, key }
                if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
            {
                // 投影坐标系对象体由专门读取器解析，max_bytes 原样转发。
                let body = read_projected_coordinate_system_10_1(&mut trial, max_bytes)?;
                Some(SpatialReferenceEntry10_1::Projected { key, body })
            }
            ObjectEntry10_1::NewObject { class_id, key } => {
                // 严格子集只认投影坐标系嵌套。嵌套对象体无外部长度字段，
                // 无法跳过未知类继续读取；显式报错，游标与登记表都不提交。
                let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Envelope 对象体嵌套对象 offset 0x{nested_offset:x}: 类标识 {id}（键 \
                         {key}）不是投影坐标系；本读取器仅支持投影坐标系嵌套，且对象体 \
                         无外部长度字段，无法跳过未知类继续读取"
                    ),
                ));
            }
        }
    } else {
        None
    };

    let result = Envelope10_1 {
        version,
        coords,
        attribute_flags,
        z_domain,
        m_domain,
        spatial_reference,
    };
    *reader = trial;
    *table = trial_table;
    Ok(result)
}

/// 读取 int32 标志位控制的追加 double 对（Z/M 域候选段）。
///
/// 位掩码只作用于低字节（bit6=0x40 / bit7=0x80），int32 高位字节不触发；
/// 标志缺省（版本 1）或未置位时不读任何字节。
fn read_flagged_pair(
    trial: &mut Reader<'_>,
    flags: Option<u32>,
    mask: u32,
) -> io::Result<Option<[f64; 2]>> {
    if !flags.is_some_and(|flags| flags & mask != 0) {
        return Ok(None);
    }
    let mut pair = [0.0; 2];
    for slot in &mut pair {
        *slot = trial.read_f64_le()?;
    }
    Ok(Some(pair))
}
