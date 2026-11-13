//! Desktop 10.8 和 10.1 分流格式 `/Maps` 的严格前缀读取。
//!
//! 两个版本的原生读取链结构一致：4 字节有符号数量在流起点，首个对象入口在
//! 数量之后。10.1 证据来自 `docs/arcmap-10.1-object-body.md`：写侧 ArcMap.exe
//! `0x466fb0` 先写 4 字节 Map（数据框）数量再循环保存对象，循环后写 4 字节
//! 尾部值（ObjectStream 容量预估返回 -1 时写 1，否则写 0）；读侧 PageLayout
//! `0x7b708d00` 读 4 字节有符号数量后仅正数进入对象循环（对象头经
//! ObjectStream 虚表 `+0x44`，即 `0x7ffcff70`），循环结束后再读 4 字节尾部值。
//!
//! 两个读取器都是严格安全子集：要求完整字节，短读返回错误且不推进游标，
//! 与原生「只看 HRESULT 符号、不比较读取量」的短读行为不同；不识别对象体，
//! 不读取完整对象循环，因此不代表 MXD 格式已完全验证；人工目视验收未做。
//! 本模块不能推断地图数量之外的完整图层树；每个读取器仅适用于调用者已
//! 确认的对应版本。

use crate::object_stream::{ObjectHeader, read_header_10_1, read_header_10_8};
use crate::reader::Reader;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapsRootPrefix10_8 {
    /// `/Maps` 起点的原始有符号数量；仅正数时原生代码进入对象循环。
    pub count: i32,
    /// 正数量下的第一个对象头；空头保持后续字段未读取。
    pub first_header: Option<ObjectHeader>,
    /// 首个对象头之后未读取的字节数；对象体边界未知。
    pub remaining_bytes: usize,
}

/// 从 `/Maps` 起点读取 10.8 数量和首对象头，失败时不推进游标。
///
/// `max_count` 限制进入对象循环前申请解析工作的上限。负数量在原生分支中
/// 不进入循环，但这里显式拒绝，避免把未知异常当作空集合。
pub fn read_root_prefix_10_8(
    reader: &mut Reader<'_>,
    max_count: u32,
) -> io::Result<MapsRootPrefix10_8> {
    let mut trial = reader.clone();
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 {
        return Err(io::Error::new(ErrorKind::InvalidData, "/Maps 数量为负数"));
    }
    let limit = i32::try_from(max_count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "/Maps 限额不能由本平台表示"))?;
    if count > limit {
        return Err(io::Error::new(ErrorKind::InvalidData, "/Maps 数量超过限额"));
    }
    let first_header = if count == 0 {
        None
    } else {
        Some(read_header_10_8(&mut trial)?)
    };
    let result = MapsRootPrefix10_8 {
        count,
        first_header,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapsRootPrefix10_1 {
    /// `/Maps` 起点的原始有符号数量；写侧取自文档对象 +0x128 的 Map（数据框）
    /// 个数（ArcMap `0x466fb0`），读侧仅正数进入对象循环（PageLayout
    /// `0x7b708d00` 的 `if (0 < local_50)`）。
    pub count: i32,
    /// 正数量下的第一个对象头；空头保持后续字段未读取。
    pub first_header: Option<ObjectHeader>,
    /// 对象循环之后的 4 字节尾部值的原始位模式。写侧来源：ObjectStream 容量
    /// 预估槽（虚表 `+0x54`）返回 -1 时写 1，否则写 0（ArcMap `0x466fb0`）；
    /// 0/1 语义属证据性解读，本读取器按读侧行为保留全部位模式，不做解释。
    pub tail_raw: u32,
    /// 尾部值之后未读取的字节数；对象体无外部长度字段，边界未知，不解释。
    pub remaining_bytes: usize,
}

/// 从调用者已确认的 10.1 分流 `/Maps` 起点读取数量、首对象头与尾部值，
/// 失败时不推进游标。
///
/// 仅适用于调用者已确认的 10.1 分流 `/Maps` 起点：写侧 ArcMap `0x466fb0`
/// 先写 4 字节 Map 数量再循环保存对象，读侧 PageLayout `0x7b708d00` 在
/// `Mx Document` 老格式失败后经 ObjectStream `+0x44` 从数量之后的 `0x04`
/// 读取首个对象头，循环结束后再读 4 字节尾部值；证据见
/// `docs/arcmap-10.1-object-body.md`。首对象头经 [`read_header_10_1`] 严格
/// 读取：全零 GUID 消耗 16 字节（对应原生 S_FALSE），否则保留原始 GUID 与
/// 4 字节键；未知类标识与键原样保留。尾部值无论数量是否为正都读取（读侧
/// 在循环结束后无条件再读），原始位模式保留在 [`MapsRootPrefix10_1::tail_raw`]。
///
/// 本读取器不识别版本、不解析对象体、不读取完整对象循环、不接入 MXD 解析、
/// 也不模拟原生短读；通过不代表 MXD 格式已完全验证。
///
/// `max_count` 限制进入对象循环前申请解析工作的上限，与 10.8 读取器保持
/// 相同的严格程度。负数量在原生分支中不进入循环（写侧该值非负），这里
/// 显式拒绝并在错误信息中说明原生语义，避免把未知异常当作空集合。
pub fn read_root_prefix_10_1(
    reader: &mut Reader<'_>,
    max_count: u32,
) -> io::Result<MapsRootPrefix10_1> {
    let mut trial = reader.clone();
    let count = i32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    if count < 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "10.1 /Maps 数量为负数：原生读取仅正数进入对象循环，负数语义未知",
        ));
    }
    let limit = i32::try_from(max_count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "10.1 /Maps 限额不能由本平台表示"))?;
    if count > limit {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "10.1 /Maps 数量超过限额",
        ));
    }
    let first_header = if count == 0 {
        None
    } else {
        Some(read_header_10_1(&mut trial)?)
    };
    // 读侧在对象循环结束后无条件再读 4 字节尾部值（PageLayout `0x7b708d00`），
    // 因此数量为 0（空循环）时同样读取；原始位模式保留，0/1 解释不在此处。
    let tail_raw = u32::from_le_bytes(trial.read_bytes(4)?.try_into().unwrap());
    let result = MapsRootPrefix10_1 {
        count,
        first_header,
        tail_raw,
        remaining_bytes: trial.remaining(),
    };
    *reader = trial;
    Ok(result)
}
