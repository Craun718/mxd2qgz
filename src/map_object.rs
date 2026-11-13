//! ArcMap 10.1 Map（数据框）对象体已确认前缀的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-map-object.md`（2026-11-13）。字段序来自
//! Map.dll 10.1.0.3035 的 `IPersistStream::Load`（RTTI `.?AVMap@@` →
//! IPersistStream 子接口虚表 `0x794874a4` 槽 +0x14 = `0x793ff4d0`），并与
//! 仓库参考样本 `samples/测试底图.mxd` 的 `/Maps` 导出逐字节对照吻合：
//! `/Maps:0x18` 版本 short=51、`0x1a` 名称「新建数据框」、`0x2a` 空名称
//! （`02 00 00 00 00 00`）、`0x30` CRS 对象头（键 30/0x1e）、`0x44` CRS 体
//! 版本 6 与 455 字节 WKT。
//!
//! 已确认前缀的字段序（Load `0x793ff4d0` 第 1-4 步；第 5 步起语义未闭合，
//! 一律不读，保留为未知尾部）：
//!
//! 1. int16 架构版本（>0x33/51 直接拒绝；样本值 51）；
//! 2. 字符串 → obj+0x160（数据框名称）；
//! 3. 字符串 → obj+0x164（第二名称，样本为空串）；
//! 4. 嵌套对象 ×1（空间参考，类身份由流内 16 字节对象头决定）：样本为
//!    ProjectedCoordinateSystem；16 零字节的空头按"无空间参考"处理；其他
//!    类标识显式拒绝（对象体无外部长度字段，无法跳过未知类）。
//!
//! 字符串布局已由样本字节闭合：AfCore String::Write 普通输出，4 字节长度
//! 前缀 = 含 UTF-16LE 终止零的字节数 n*2+2。样本「新建数据框」前缀 12
//! （5 字 ×2 + 终止零 2），空名称前缀 2（仅终止零，即文档强调的 6 字节
//! `02 00 00 00 00 00` 模式，勿误读为 int32 计数）。与
//! [`crate::native_string::read_string_10_1`] 的布局一致，名称读取据此
//! 接入该严格读取器。
//!
//! 调用约定：调用者先用 [`crate::object_stream::read_header_10_1`] 读取
//! Map 对象头并比对 [`MAP_CLASS_ID`]，从对象头之后的位置调用
//! [`read_map_prefix_10_1`]；本模块不重复读取对象头。
//!
//! 这是仅覆盖上述已确认字段的严格安全子集：要求完整输入、任何失败不推进
//! 调用者游标。Load 第 5 步起的计数序列、WKSEnvelope、用途未定性的嵌套
//! 对象及大量版本分支字段一律不读，保留在
//! [`MapPrefix10_1::remaining_bytes`] 中不解释，因此不代表完整 Map 体行
//! 为，未接入完整 MXD 解析，人工目视验收未做。
//!
//! 未解决问题（详见证据文档「未解决事项」）：Load 第 5/7/8/9 步嵌套序列
//! 的确切语义（哪个循环是图层集合）未逐对象定性；版本 short 比较的符号
//! 性未在文档闭合（本读取器按无符号解释，负值位模式一并拒绝，较有符号
//! 原生更严，与 spatial_reference 的版本处理一致）；嵌套空头是否消耗 4
//! 字节键未闭合（本读取器沿用 [`crate::object_stream`] 的 S_FALSE 约定，
//! 仅消耗 16 字节）。

use crate::native_string::read_string_10_1;
use crate::object_stream::{ObjectHeader, read_header_10_1};
use crate::reader::Reader;
use crate::spatial_reference::{
    PROJECTED_COORDINATE_SYSTEM_CLASS_ID, ProjectedCoordinateSystem10_1,
    read_projected_coordinate_system_10_1,
};
use std::io::{self, ErrorKind};

/// Map::Load 接受的最大架构版本：>0x33（51）直接拒绝；样本值 51。
const MAX_ARCHITECTURE_VERSION: u16 = 51;

/// Map 类标识（CLSID）的 16 字节内存序（Data1-3 小端、Data4 原序）。
///
/// GUID 字符串形式为 `{E6BDAA76-4D35-11D0-98BE-00805F7CED21}`。三重印证：
/// 安装配置 `esri.clsid.ecfg` 声明该 CLSID → 类名 Map（模块 Map.dll）；
/// 该 CLSID 的字节宿主在 Map.dll `0x79482adc`（GetClassID 引用）；Map.dll
/// RTTI 存在 `.?AVMap@@`。仓库参考样本 `/Maps:0x04..0x13` 首对象头的类
/// 标识与本常量逐字节相同。
pub const MAP_CLASS_ID: [u8; 16] = [
    0x76, 0xaa, 0xbd, 0xe6, 0x35, 0x4d, 0xd0, 0x11, 0x98, 0xbe, 0x00, 0x80, 0x5f, 0x7c, 0xed, 0x21,
];

/// 10.1 Map::Load（`0x793ff4d0`）已确认前缀的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MapPrefix10_1 {
    /// 2 字节架构版本 short 的无符号位模式，>51 的位模式一律拒绝。
    ///
    /// 证据：Load `0x793ff4d0` 开头 `AfStreamRead(2)` 后对 >0x33（51）直接
    /// 拒绝，样本值 51。反编译比较的符号性未闭合：按无符号解释接受 0..=51；
    /// 若原生为有符号比较，负值位模式（0x8000..）可能通过原生检查，此处
    /// 一并拒绝（较原生更严，与 spatial_reference 的版本处理一致）。
    pub version: u16,
    /// 数据框名称（Load 第 2 步，写入 obj+0x160；样本「新建数据框」）。
    pub name: String,
    /// 第二名称字段（Load 第 3 步，写入 obj+0x164；样本为空串）。
    pub secondary_name: String,
    /// Load 第 4 步的嵌套空间参考对象体。
    ///
    /// 16 零字节空头 → `None`（沿用对象头的 S_FALSE 约定，仅消耗 16 字节、
    /// 不读键）；类标识等于 PCS 常量 → 已解析对象体；其他类标识 →
    /// [`read_map_prefix_10_1`] 直接报错。对象头键值原样丢弃，未保留。
    pub spatial_reference: Option<ProjectedCoordinateSystem10_1>,
    /// 已确认前缀之后剩余的字节数。
    ///
    /// 对应 Load 第 5 步起的未闭合字段（计数序列、WKSEnvelope、用途未定性
    /// 的嵌套对象、大量版本分支字段等），本读取器不读取、不解释。
    pub remaining_bytes: usize,
}

/// 从对象头之后的位置读取 10.1 Map 对象体的已确认前缀，失败时不推进游标。
///
/// 起点约定：调用者已用 [`crate::object_stream::read_header_10_1`] 读取
/// Map 对象头、比对 [`MAP_CLASS_ID`] 确认类身份，并从对象头之后的位置传
/// 入 Reader；本函数不重复读取对象头。字段序与证据见模块文档。
///
/// `max_bytes` 是资源限额：应用于两个名称字符串的声明长度（含终止零、
/// 不含 4 字节前缀），并原样转发给嵌套投影坐标系对象体（约束 WKT 与域字
/// 符串长度）；限额在任何读取与分配之前检查，`usize::MAX` 表示不额外设
/// 限，此时超长流按缺短读失败。
///
/// 嵌套空间参考对象的类身份由流内对象头决定：全零 16 字节为空头
/// （`spatial_reference = None`），类标识等于
/// [`PROJECTED_COORDINATE_SYSTEM_CLASS_ID`] 时用
/// [`read_projected_coordinate_system_10_1`] 读取对象体；其他类标识返回
/// [`ErrorKind::Unsupported`]——嵌套对象体没有外部长度字段，本读取器无法
/// 跳过未知类继续解析，调用者不能把该流继续当作 Map 前缀消费。
///
/// 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进调用
/// 者游标。错误位置相对于传入 Reader，调用方需另外记录流和对象身份。这是
/// 严格安全子集，不代表完整原生 Load 行为，通过也不代表 MXD 格式已验证。
pub fn read_map_prefix_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<MapPrefix10_1> {
    // 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进原游标。
    let mut trial = reader.clone();
    let entry_offset = trial.position();

    // 1. 2 字节架构版本：无符号解释，>51 直接拒绝（原生 >0x33 拒绝）。
    let version = trial.read_u16_le()?;
    if version > MAX_ARCHITECTURE_VERSION {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "Map 对象体 offset 0x{entry_offset:x}: 架构版本 {version} 超出 \
                 Map::Load 接受范围（>0x33/51 直接拒绝）"
            ),
        ));
    }

    // 2/3. 两个名称字符串：AfCore String 布局（样本字节闭合，见模块文档），
    // 各自受 max_bytes 限额约束。
    let name = read_string_10_1(&mut trial, max_bytes)?;
    let secondary_name = read_string_10_1(&mut trial, max_bytes)?;

    // 4. 嵌套空间参考对象：先读对象头（类身份完全由流内对象头决定）。
    let nested_offset = trial.position();
    let spatial_reference = match read_header_10_1(&mut trial)? {
        // 全零 16 字节：沿用对象头读取器的 S_FALSE 约定，仅消耗对象头本身、
        // 不读键；语义上表示该 Map 没有空间参考对象。
        ObjectHeader::Null => None,
        ObjectHeader::Object { class_id, key: _ }
            if class_id == PROJECTED_COORDINATE_SYSTEM_CLASS_ID =>
        {
            // PCS 对象体由专门读取器解析，max_bytes 原样转发。
            Some(read_projected_coordinate_system_10_1(
                &mut trial, max_bytes,
            )?)
        }
        ObjectHeader::Object { class_id, key } => {
            // 严格子集只认投影坐标系嵌套。嵌套对象体无外部长度字段，无法
            // 跳过未知类继续读取；显式报错，原游标不推进。
            let id: String = class_id.iter().map(|byte| format!("{byte:02x}")).collect();
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Map 对象体嵌套对象 offset 0x{nested_offset:x}: 类标识 {id}（键 \
                     {key}）不是投影坐标系；本读取器仅支持投影坐标系嵌套，且对象体 \
                     无外部长度字段，无法跳过未知类继续读取"
                ),
            ));
        }
    };

    let result = MapPrefix10_1 {
        version,
        name,
        secondary_name,
        spatial_reference,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}
