//! ArcMap 10.1 投影坐标系（ProjectedCoordinateSystem）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-spatial-reference.md`（2026-11-13）。布局证据
//! 为 Geometry.dll 10.1.0.3035 的保存方法 `0x719d3690` 与读取方法 `0x719d3230`
//! 反编译，并与仓库参考样本 `temp/测试底图.mxd` 的 `/Maps` 导出逐字节对照
//! 吻合（版本 6：WKT 455 字节、4 个空域字符串、垂向标志 0x00、标志 0x0001、
//! 7+3 个 double）。
//!
//! 调用约定：调用者先用 [`crate::object_stream::read_header_10_1`] 读取对象
//! 头，比对 [`PROJECTED_COORDINATE_SYSTEM_CLASS_ID`] 确认类标识后，从对象头
//! 之后的位置调用读取器；本模块不重复读取对象头。
//!
//! 字段存在性随 2 字节保存版本 short 分支，与原生 Load（`0x719d3230`）一致：
//!
//! - 全部版本：2 字节版本 + 4 字节 WKT 长度（不含终止零）+ 等长 ASCII WKT
//!   字节（无终止零）；
//! - 版本 >2：4 组（4 字节长度 + ANSI 域字符串）+ 1 字节垂向域标志；
//! - 版本 >=4：2 字节标志 + 7 个 double；
//! - 版本 >=5：再读 3 个 double；
//! - 版本 >7：原生经 `0x719f1570` 记录 `0x80004005` 并退出，本读取器拒绝。
//!
//! 这是仅覆盖上述分支的严格安全子集：要求完整输入、失败不推进游标，不代
//! 表完整原生行为，未接入 MXD 解析，人工验收未做。
//!
//! 未解决问题（详见证据文档）：接口 IID 未确认（`0x71a29ccc` 虚表槽位序仅
//! 与 `IPersistStream` 吻合）；2 字节标志与 10 个 double 的语义未确认；保存
//! 方法第 4-6 项的版本条件与读取分支的对应不对称，完整布局仅由样本版本 6
//! 证实；域字符串子函数（`0x719babf0` 等）未展开；原生把 WKT 读入 3072 字
//! 节栈缓冲且反编译未见长度上限检查（潜在原生缺陷），本读取器保留
//! `max_bytes` 资源限额。

use crate::reader::Reader;
use std::io::{self, ErrorKind};

/// ProjectedCoordinateSystem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{2A626700-1DD2-11B2-BF51-08002022F573}`。证据来自
/// ArcMap 10.1 Geometry.dll（10.1.0.3035）的类注册构造 `0x71a09a70`：索引
/// `0x20` 的表项注册类名 `L"ProjectedCoordinateSystem"` 与 CLSID 数据地址
/// `0x71a13098`；该地址的 16 字节与仓库参考样本 `/Maps` 流中 PCS 对象头的
/// 类标识逐字节相同。CLSID 本身是 COM 标识，其分配来源未进一步考证。
pub const PROJECTED_COORDINATE_SYSTEM_CLASS_ID: [u8; 16] = [
    0x00, 0x67, 0x62, 0x2a, 0xd2, 0x1d, 0xb2, 0x11, 0xbf, 0x51, 0x08, 0x00, 0x20, 0x22, 0xf5, 0x73,
];

/// 版本 >2 分支读取的域字符串块（对应原生子函数 `0x719babf0` 候选，未展开）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainBlock {
    /// 4 个长度前缀 ANSI 域字符串的原始字节（样本中全部为空，语义未确认）。
    ///
    /// 不做 WKT 式的 ASCII 校验并保留 NUL 与 >=0x80 字节：域字符串是
    /// `Ordinal_149` 从 UTF-16 转码的 ANSI 字节，内容随代码页（如 GBK 的
    /// 双字节区域），非 ASCII 字节可能是原生合法输出；而 WKT 由 pe API 生
    /// 成、证据记载为纯 ASCII。两者验证策略的差异源于生成路径的证据强度。
    pub strings: [Vec<u8>; 4],
    /// 1 字节垂向域标志，原样保留、不校验。
    ///
    /// 保存侧恒写 0（样本 0x00）；原生 Load 在该字节非零时重建垂向域。
    /// 语义细节未确认。
    pub vertical_flag: u8,
}

/// 版本 >=4 分支读取的标志与 7 个 double（对应原生 `0x719bae60(1)`）。
#[derive(Debug, Clone, PartialEq)]
pub struct PrecisionBlock {
    /// 2 字节标志，原样保留、不校验。
    ///
    /// 保存侧为 `SgCoordRefGetPrecision` 结果 bit6（样本 0x0001），完整语义
    /// 未确认。
    pub flags: u16,
    /// 7 个小端 double，仅原样保留位模式，不解释语义、不做 NaN 或范围判断。
    ///
    /// 原生按 `SgCoordRefGetXY`(3)/`GetZ`(2)/`GetM`(2) 分组写出，缺省值
    /// `DAT_71a66f20` 未读取；样本值为 31876800.0、-10002100.0、
    /// 450265407.00157917、-100000.0、10000.0、-100000.0、10000.0。
    pub doubles: [f64; 7],
}

/// 10.1 读取方法可加载的投影坐标系对象体；字段存在性见模块文档的版本分支。
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedCoordinateSystem10_1 {
    /// 2 字节保存版本 short，按无符号位模式保留。
    ///
    /// 原生读取方法对大于 7 的版本报 `0x80004005`；保存侧版本表为
    /// `[2,3,6,6]`（文档版本 5，即 10.x，写出表末项 6，样本为 6）。本读取
    /// 器接受 0..=7 并拒绝其余位模式：0x8000.. 在无符号解释下同样大于 7，
    /// 若原生为有符号比较则此处更严（负值无保存表证据）。
    pub save_version: u16,
    /// 纯 ASCII WKT 文本，无终止零；4 字节长度前缀同样不含终止零。
    ///
    /// 原生读取不校验内容并把文本读入 3072 字节栈缓冲、反编译未见长度上限
    /// 检查（潜在原生缺陷）；本读取器强制 ASCII 且拒绝 NUL，较原生更严。
    pub wkt: String,
    /// 版本 >2 分支的域字符串块；版本 <=2 时为 `None`（原生 Load 同样不读）。
    pub domains: Option<DomainBlock>,
    /// 版本 >=4 分支的标志与 7 个 double；与 `domains` 同现（>2 且 >=4）。
    pub precision: Option<PrecisionBlock>,
    /// 版本 >=5 分支再读的 3 个小端 double，仅原样保留（样本均为 0.001）。
    ///
    /// 原生直接写入 `this+0xf0..+0x104`；更旧版本从标志位恢复，不出现在流中。
    /// `precision` 为 `Some` 时本字段可能仍为 `None`（版本 4）。
    pub trailing_doubles: Option<[f64; 3]>,
}

/// 从对象头之后的位置读取 10.1 PCS 对象体的严格子集，失败时不推进游标。
///
/// `max_bytes` 是应用于 WKT 长度与每个域字符串长度的资源上限（不含 4 字节
/// 前缀本身），在任何读取与分配之前检查；`usize::MAX` 表示不额外设限，此时
/// 超长流按缺短读失败。版本分支见模块文档；错误位置相对于传入 Reader，调
/// 用方需另外记录流和对象身份。
pub fn read_projected_coordinate_system_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<ProjectedCoordinateSystem10_1> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("投影坐标系对象体 offset 0x{start:x}: {reason}"),
        )
    };
    // 全部字段在 trial 游标上读取，完整成功才提交；任何失败路径都不推进原游标。
    let mut trial = reader.clone();

    // 1. 2 字节保存版本 short：无符号解释，拒绝 >7（原生记 0x80004005）。
    let save_version = trial.read_u16_le()?;
    if save_version > 7 {
        return Err(invalid(&format!(
            "保存版本 {save_version} 超出原生读取方法的接受范围（>7 报 0x80004005）"
        )));
    }

    // 2. WKT：4 字节长度（不含终止零）+ 等长 ASCII 字节（无终止零）。
    let wkt_length =
        usize::try_from(trial.read_u32_le()?).map_err(|_| invalid("WKT 长度不能由本平台表示"))?;
    if wkt_length > max_bytes {
        return Err(invalid("WKT 长度超过调用者资源限额"));
    }
    let wkt_bytes = trial.read_bytes(wkt_length)?;
    // WKT 证据记载为纯 ASCII，且原生按 C 字符串消费、不可能含 NUL；原生
    // 读取本身不校验内容，此处校验较原生更严。
    if let Some(offset) = wkt_bytes.iter().position(|&byte| byte == 0 || byte >= 0x80) {
        return Err(invalid(&format!(
            "WKT 含 NUL 或非 ASCII 字节（第 {offset} 字节）"
        )));
    }
    let wkt = String::from_utf8(wkt_bytes.to_vec()).map_err(|_| invalid("WKT 不是合法 UTF-8"))?;

    // 3. 版本 >2：4 组（4 字节长度 + ANSI 域字符串）+ 1 字节垂向域标志。
    let domains = if save_version > 2 {
        let mut strings: [Vec<u8>; 4] = Default::default();
        for slot in &mut strings {
            let length = usize::try_from(trial.read_u32_le()?)
                .map_err(|_| invalid("域字符串长度不能由本平台表示"))?;
            if length > max_bytes {
                return Err(invalid("域字符串长度超过调用者资源限额"));
            }
            // 原始字节保留：内容随 ANSI 代码页，不做 WKT 式校验（见字段注释）。
            *slot = trial.read_bytes(length)?.to_vec();
        }
        let vertical_flag = trial.read_bytes(1)?[0];
        Some(DomainBlock {
            strings,
            vertical_flag,
        })
    } else {
        None
    };

    // 4. 版本 >=4：2 字节标志 + 7 个 double。
    let precision = if save_version >= 4 {
        let flags = trial.read_u16_le()?;
        let mut doubles = [0.0; 7];
        for slot in &mut doubles {
            // 10 个 double 一律按小端 f64 原样保留，不解释、不判 NaN 或范围。
            *slot = trial.read_f64_le()?;
        }
        Some(PrecisionBlock { flags, doubles })
    } else {
        None
    };

    // 5. 版本 >=5：再读 3 个 double。
    let trailing_doubles = if save_version >= 5 {
        let mut doubles = [0.0; 3];
        for slot in &mut doubles {
            *slot = trial.read_f64_le()?;
        }
        Some(doubles)
    } else {
        None
    };

    *reader = trial;
    Ok(ProjectedCoordinateSystem10_1 {
        save_version,
        wkt,
        domains,
        precision,
        trailing_doubles,
    })
}
