//! ArcMap 10.1 空间参考槽的另两个实现类（GeographicCoordinateSystem 与
//! UnknownCoordinateSystem）对象体的严格读取子集。
//!
//! 证据来源：`docs/arcmap-10.1-pcs-geometry-slots.md`（2026-11-16）。布局证据
//! 为 Geometry.dll 10.1.0.3035 的 GeographicCoordinateSystem::Load `0x719bec60`
//! /Save `0x719bf1d0` 与 UnknownCoordinateSystem::Load `0x719ecd80`/Save
//! `0x719ed310` 反编译（持久化虚表分别经 COL→RTTI TD 锚定：GCS `0x71a26818`
//! @对象+0x2c、Unknown `0x71a2c320` @对象+0x30；两者 GetClassID 分别写出
//! CLSID 常量 `0x71a13078`/`0x71a130c8`）。仓库参考样本 `/Maps` 流内两类
//! **0 实例**（仅 ProjectedCoordinateSystem 出现），全部由合成输入测试覆盖，
//! 不得声称样本验证。
//!
//! 调用约定与 [`crate::objects::spatial_reference`] 一致：调用者先读对象头并
//! 比对类标识，从对象头之后调用；本模块不重复读对象头。
//!
//! 版本分支（GCS，原生 Load `0x719bec60`，10.1 Save 版本表 `[3,4,7,7]` 恒写 7）：
//!
//! - 全部版本：2 字节版本 + 4 字节 WKT 字节长（无终止零）+ 等长 ASCII WKT；
//! - 版本 >=2：4 组（4 字节字节长 + 原样字符串字节）+ 1 字节垂向标志；
//!   标志非零时紧跟垂向单位段（2 字节版本 ≤2 + 4 字节长 + 等长原样字节，
//!   原生 `0x719ea140` 读出后经 `pe_unit_from_string` 解析为垂向线性单位）；
//! - 版本 >2：2 个 double 地理范围（原生首项 NaN 时重建为 [-180, 180]，
//!   本读取器原样保留）；
//! - 版本 ==4：7 个 double（无 2 字节标志）；版本 >=5：2 字节标志 + 7 个
//!   double；其余版本无精度块；
//! - 版本 >=6：3 个 double（XY/Z/M 容差）；版本 <6 原生按标志位重建默认容差
//!   （不出现在流中，本读取器不消费）。
//!
//! 版本分支（Unknown，原生 Load `0x719ecd80`，10.1 Save 版本表前 3 项
//! `[1,1,3]` 恒写 3）：
//!
//! - 全部版本：2 字节版本 + 4 个 AfCore `String::Read` 布局字符串（成员
//!   +0x54/+0x4c/+0x4c/+0x50；原生把第 2、3 串写入同一成员、第 2 串实际被
//!   覆盖——读取器按流内 4 串原样保留并注记该原生行为）；
//! - 版本 ==1：7 个 double（无标志）；版本 >=2：2 字节标志 + 7 个 double
//!   （原生按标志推导精度 32/64：非零 → 64）；
//! - 版本 >=3：3 个 double（XY/Z/M 容差，原生 NaN 或 ≤0 时重置默认）。
//!
//! 这是仅覆盖上述分支的严格安全子集：要求完整输入、失败不推进游标；原生
//! 负版本位模式（有符号比较不拒）在本读取器一律拒绝，较原生更严。
//!
//! 未解决问题（详见证据文档）：两类在参考样本中 0 实例；域字符串内容语义、
//! Unknown 四串的业务语义未定；GCS 范围 double 的非默认值形态无样本。

use crate::stream::native_string::read_string_10_1;
use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// GeographicCoordinateSystem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{A6A87A80-1DD1-11B2-BF51-08002022F573}`。证据来自
/// Geometry.dll 10.1.0.3035：esriGeometry 类型库 coclass #96、类注册构造
/// `0x71a09a70` 的 `L"GeographicCoordinateSystem"` 表项（CLSID 数据
/// `0x71a13078`），与本模块按该地址扫描的字节一致；esri.clsid 全表亦声明
/// 同值。样本内无实例。
pub const GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID: [u8; 16] = [
    0x80, 0x7a, 0xa8, 0xa6, 0xd1, 0x1d, 0xb2, 0x11, 0xbf, 0x51, 0x08, 0x00, 0x20, 0x22, 0xf5, 0x73,
];

/// UnknownCoordinateSystem 类标识（CLSID）的 16 字节内存序。
///
/// GUID 字符串形式为 `{B286C06B-0879-11D2-AACA-00C04FA33C20}`。证据同上：
/// esriGeometry 类型库 coclass #130、注册表项 CLSID 数据 `0x71a130c8`、
/// esri.clsid 全表声明同值。样本内无实例。
pub const UNKNOWN_COORDINATE_SYSTEM_CLASS_ID: [u8; 16] = [
    0x6b, 0xc0, 0x86, 0xb2, 0x79, 0x08, 0xd2, 0x11, 0xaa, 0xca, 0x00, 0xc0, 0x4f, 0xa3, 0x3c, 0x20,
];

/// 版本 >=2 分支读取的域字符串块（与投影坐标系同构，原生子函数 `0x719babf0`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainBlock {
    /// 4 组「4 字节字节长 + 原样字节」的字符串原始字节（样本中无实例）。
    ///
    /// 原生按 UTF-16 宽字符消费（长度为偶数字节、末尾补宽终止零后转 BSTR），
    /// 本读取器不做编码校验、原样保留，与投影坐标系读取器的域串策略一致。
    /// 流内顺序为 Alias、Abbreviation、Remarks、Usage。
    pub strings: [Vec<u8>; 4],
    /// 1 字节垂向标志，原样保留、不校验。
    ///
    /// 10.1 写侧恒写 0（Save `0x719d3690`/`0x719bf1d0` 均写常量零）；原生
    /// Load 在该字节非零时按 [`VerticalUnitSegment`] 消费垂向单位段。
    pub vertical_flag: u8,
    /// 垂向标志非零时紧跟的垂向单位段；标志为零时为 `None`。
    pub vertical_unit: Option<VerticalUnitSegment>,
}

/// 垂向标志非零时的垂向单位段（原生子函数 `0x719ea140`，证据：反编译）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerticalUnitSegment {
    /// 2 字节段版本，原生大于 2 报 `0x80004005`。
    pub save_version: u16,
    /// 单位 WKT 文本的原样字节（4 字节字节长前缀不含终止零；原生经
    /// `pe_unit_from_string` 解析，生成侧为 ASCII，读取器不强制校验）。
    pub unit_wkt: Vec<u8>,
}

/// GCS/Unknown 版本 >=5（Unknown >=2）分支读取的标志与 7 个 double。
#[derive(Debug, Clone, PartialEq)]
pub struct PrecisionBlock {
    /// 2 字节标志，原样保留、不校验。
    ///
    /// 写侧为 `SgCoordRefGetPrecision` 结果 bit6（1 = 64 位高精度网格，0 =
    /// 32 位或无，sg.dll `0x7faebc40` 证据）；Unknown 读侧按非零推导精度
    /// 64/32，GCS/PCS 读侧不消费该值。
    pub flags: u16,
    /// 7 个小端 double，原样保留位模式。
    ///
    /// 原生按 `SgCoordRefGetXY/GetZ/GetM` 分组写出：XY 假原点 X/Y、XY
    /// 比例因子、Z 假原点、Z 比例因子、M 假原点、M 比例因子（sg.dll 结构
    /// 偏移 +0x08/+0x10/+0x18/+0x30/+0x38/+0x50/+0x58）；首 XY 值 NaN 表示
    /// 无精度网格（原生清空）。未设 Z/M 时写侧输出最小非规格化正数
    /// `DAT_71a66f20`（4.94e-324）占位。
    pub doubles: [f64; 7],
}

/// 10.1 GeographicCoordinateSystem 对象体；字段存在性见模块文档的版本分支。
#[derive(Debug, Clone, PartialEq)]
pub struct GeographicCoordinateSystem10_1 {
    /// 2 字节保存版本 short，按无符号位模式保留；原生接受 0..=8（有符号
    /// 比较 >8 报 `0x80004005`），本读取器拒绝其余位模式（负值一并拒绝）。
    pub save_version: u16,
    /// 纯 ASCII WKT 文本，无终止零；4 字节长度前缀同样不含终止零。
    pub wkt: String,
    /// 版本 >=2 分支的域字符串块；版本 <2 时为 `None`。
    pub domains: Option<DomainBlock>,
    /// 版本 >2 分支的 2 个地理范围 double `[min, max]`，原样保留
    /// （原生首项 NaN 时重建 [-180, 180]，本读取器不重建）。
    pub extent: Option<[f64; 2]>,
    /// 精度块：版本 ==4 为无标志 7 double（`flags` 恒 0），版本 >=5 为标志
    /// + 7 double；其余版本为 `None`。
    pub precision: Option<PrecisionBlock>,
    /// 版本 >=6 分支的 3 个容差 double `[XY, Z, M]`；版本 <6 为 `None`
    /// （原生按标志位重建默认容差，不出现在流中）。
    pub tolerances: Option<[f64; 3]>,
}

/// 10.1 UnknownCoordinateSystem 对象体；字段存在性见模块文档的版本分支。
#[derive(Debug, Clone, PartialEq)]
pub struct UnknownCoordinateSystem10_1 {
    /// 2 字节保存版本 short；原生接受 0..=3（`3 < ver` 有符号拒绝），本
    /// 读取器拒绝其余位模式。
    pub save_version: u16,
    /// 4 个 AfCore `String::Read` 布局字符串（流内顺序原样保留）。
    ///
    /// 10.1 写侧顺序为 Name、Abbreviation、Abbreviation、Remarks。
    /// 原生读取用第三串覆盖第二串，解析结果保留流内四项。
    pub strings: [String; 4],
    /// 精度块：版本 ==1 为无标志 7 double（`flags` 恒 0），版本 >=2 为标志
    /// + 7 double；版本 0 为 `None`。
    pub precision: Option<PrecisionBlock>,
    /// 版本 >=3 分支的 3 个容差 double `[XY, Z, M]`（原生 NaN 或 <=0 时按
    /// 默认容差重置，本读取器原样保留）。版本 <3 从原生网格恢复容差，流内无该字段，
    /// 本读取器返回 `None`。
    pub tolerances: Option<[f64; 3]>,
}

/// 从对象头之后的位置读取 10.1 GCS 对象体的严格子集，失败时不推进游标。
///
/// `max_bytes` 应用于 WKT、域字符串与垂向单位段的字节长（不含 4 字节前缀
/// 本身）；`usize::MAX` 表示不额外设限。
pub fn read_geographic_coordinate_system_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<GeographicCoordinateSystem10_1> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("地理坐标系对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();

    let save_version = trial.read_u16_le()?;
    if save_version > 8 {
        return Err(invalid(&format!(
            "保存版本 {save_version} 超出原生读取方法的接受范围（>8 报 0x80004005）"
        )));
    }
    let wkt = read_wkt_block(&mut trial, max_bytes, &invalid)?;

    let domains = if save_version >= 2 {
        Some(read_domain_block(&mut trial, max_bytes, &invalid)?)
    } else {
        None
    };

    let extent = if save_version > 2 {
        let min = trial.read_f64_le()?;
        let max = trial.read_f64_le()?;
        Some([min, max])
    } else {
        None
    };

    let precision = if save_version == 4 {
        let mut doubles = [0.0; 7];
        for slot in &mut doubles {
            *slot = trial.read_f64_le()?;
        }
        Some(PrecisionBlock { flags: 0, doubles })
    } else if save_version >= 5 {
        Some(read_precision_block(&mut trial)?)
    } else {
        None
    };

    let tolerances = if save_version >= 6 {
        Some(read_tolerances(&mut trial)?)
    } else {
        None
    };

    *reader = trial;
    Ok(GeographicCoordinateSystem10_1 {
        save_version,
        wkt,
        domains,
        extent,
        precision,
        tolerances,
    })
}

/// 从对象头之后的位置读取 10.1 Unknown 对象体的严格子集，失败时不推进游标。
pub fn read_unknown_coordinate_system_10_1(
    reader: &mut Reader<'_>,
    max_bytes: usize,
) -> io::Result<UnknownCoordinateSystem10_1> {
    let start = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("未知坐标系对象体 offset 0x{start:x}: {reason}"),
        )
    };
    let mut trial = reader.clone();

    let save_version = trial.read_u16_le()?;
    if save_version > 3 {
        return Err(invalid(&format!(
            "保存版本 {save_version} 超出原生读取方法的接受范围（>3 报 0x80004005）"
        )));
    }
    let mut strings = [String::new(), String::new(), String::new(), String::new()];
    for slot in &mut strings {
        *slot = read_string_10_1(&mut trial, max_bytes)
            .map_err(|err| invalid(&format!("字符串读取失败：{err}")))?;
    }

    let precision = if save_version == 1 {
        let mut doubles = [0.0; 7];
        for slot in &mut doubles {
            *slot = trial.read_f64_le()?;
        }
        Some(PrecisionBlock { flags: 0, doubles })
    } else if save_version >= 2 {
        Some(read_precision_block(&mut trial)?)
    } else {
        None
    };

    let tolerances = if save_version >= 3 {
        Some(read_tolerances(&mut trial)?)
    } else {
        None
    };

    *reader = trial;
    Ok(UnknownCoordinateSystem10_1 {
        save_version,
        strings,
        precision,
        tolerances,
    })
}

/// 读取「4 字节字节长 + 等长 ASCII 字节」的 WKT 块（无终止零）。
fn read_wkt_block(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    invalid: &dyn Fn(&str) -> io::Error,
) -> io::Result<String> {
    let length =
        usize::try_from(trial.read_u32_le()?).map_err(|_| invalid("WKT 长度不能由本平台表示"))?;
    if length > max_bytes {
        return Err(invalid("WKT 长度超过调用者资源限额"));
    }
    let wkt_bytes = trial.read_bytes(length)?;
    // WKT 由 pe API 生成、证据为纯 ASCII；原生按 C 字符串消费、不可能含
    // NUL。原生读取不校验内容，此处校验较原生更严。
    if let Some(offset) = wkt_bytes.iter().position(|&byte| byte == 0 || byte >= 0x80) {
        return Err(invalid(&format!(
            "WKT 含 NUL 或非 ASCII 字节（第 {offset} 字节）"
        )));
    }
    String::from_utf8(wkt_bytes.to_vec()).map_err(|_| invalid("WKT 不是合法 UTF-8"))
}

/// 读取版本 >2 分支的域字符串块：4 组字符串 + 1 字节垂向标志（+单位段）。
fn read_domain_block(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    invalid: &dyn Fn(&str) -> io::Error,
) -> io::Result<DomainBlock> {
    let mut strings: [Vec<u8>; 4] = Default::default();
    for slot in &mut strings {
        let length = usize::try_from(trial.read_u32_le()?)
            .map_err(|_| invalid("域字符串长度不能由本平台表示"))?;
        if length > max_bytes {
            return Err(invalid("域字符串长度超过调用者资源限额"));
        }
        *slot = trial.read_bytes(length)?.to_vec();
    }
    let vertical_flag = trial.read_bytes(1)?[0];
    let vertical_unit = if vertical_flag != 0 {
        Some(read_vertical_unit_segment(trial, max_bytes, invalid)?)
    } else {
        None
    };
    Ok(DomainBlock {
        strings,
        vertical_flag,
        vertical_unit,
    })
}

/// 读取垂向标志非零时的垂向单位段（原生 `0x719ea140`）。
pub(crate) fn read_vertical_unit_segment(
    trial: &mut Reader<'_>,
    max_bytes: usize,
    invalid: &dyn Fn(&str) -> io::Error,
) -> io::Result<VerticalUnitSegment> {
    let save_version = trial.read_u16_le()?;
    if save_version > 2 {
        return Err(invalid(&format!(
            "垂向单位段版本 {save_version} 超出原生接受范围（>2 报 0x80004005）"
        )));
    }
    let length = usize::try_from(trial.read_u32_le()?)
        .map_err(|_| invalid("垂向单位文本长度不能由本平台表示"))?;
    if length > max_bytes {
        return Err(invalid("垂向单位文本长度超过调用者资源限额"));
    }
    let unit_wkt = trial.read_bytes(length)?.to_vec();
    Ok(VerticalUnitSegment {
        save_version,
        unit_wkt,
    })
}

/// 读取 2 字节标志 + 7 个 double 的精度块（原生 `0x719bae60(1)` 主路径）。
fn read_precision_block(trial: &mut Reader<'_>) -> io::Result<PrecisionBlock> {
    let flags = trial.read_u16_le()?;
    let mut doubles = [0.0; 7];
    for slot in &mut doubles {
        *slot = trial.read_f64_le()?;
    }
    Ok(PrecisionBlock { flags, doubles })
}

/// 读取 3 个容差 double `[XY, Z, M]`。
fn read_tolerances(trial: &mut Reader<'_>) -> io::Result<[f64; 3]> {
    let mut tolerances = [0.0; 3];
    for slot in &mut tolerances {
        *slot = trial.read_f64_le()?;
    }
    Ok(tolerances)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 按内存序断言类标识常量（esriGeometry 类型库与注册表项字节）。
    #[test]
    fn class_id_constants_match_source_bytes() {
        assert_eq!(
            GEOGRAPHIC_COORDINATE_SYSTEM_CLASS_ID,
            [
                0x80, 0x7a, 0xa8, 0xa6, 0xd1, 0x1d, 0xb2, 0x11, 0xbf, 0x51, 0x08, 0x00, 0x20, 0x22,
                0xf5, 0x73
            ]
        );
        assert_eq!(
            UNKNOWN_COORDINATE_SYSTEM_CLASS_ID,
            [
                0x6b, 0xc0, 0x86, 0xb2, 0x79, 0x08, 0xd2, 0x11, 0xaa, 0xca, 0x00, 0xc0, 0x4f, 0xa3,
                0x3c, 0x20
            ]
        );
    }

    fn push_wkt(bytes: &mut Vec<u8>, wkt: &str) {
        bytes.extend_from_slice(&(wkt.len() as u32).to_le_bytes());
        bytes.extend_from_slice(wkt.as_bytes());
    }

    fn push_domains(bytes: &mut Vec<u8>, vertical_flag: u8) {
        for _ in 0..4 {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        bytes.push(vertical_flag);
    }

    fn push_precision(bytes: &mut Vec<u8>, flags: u16) {
        bytes.extend_from_slice(&flags.to_le_bytes());
        for value in [1.0f64, 2.0, 3.0, -100000.0, 10000.0, -100000.0, 10000.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }

    #[test]
    fn gcs_version7_full_layout_roundtrip() {
        // 10.1 写侧恒写版本 7（版本表 [3,4,7,7] 末项）：WKT + 域串 + 垂向 0
        // + 范围 + 标志 + 7 double + 3 容差。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&7u16.to_le_bytes());
        push_wkt(&mut bytes, r#"GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984"]]"#);
        push_domains(&mut bytes, 0);
        bytes.extend_from_slice(&(-180.0f64).to_le_bytes());
        bytes.extend_from_slice(&180.0f64.to_le_bytes());
        push_precision(&mut bytes, 1);
        for value in [0.001f64, 0.001, 0.001] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let mut reader = Reader::new(&bytes);
        let body = read_geographic_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(body.save_version, 7);
        assert_eq!(body.wkt, r#"GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984"]]"#);
        let domains = body.domains.unwrap();
        assert_eq!(domains.vertical_flag, 0);
        assert!(domains.strings.iter().all(Vec::is_empty));
        assert!(domains.vertical_unit.is_none());
        assert_eq!(body.extent, Some([-180.0, 180.0]));
        let precision = body.precision.unwrap();
        assert_eq!(precision.flags, 1);
        assert_eq!(precision.doubles[3], -100000.0);
        assert_eq!(body.tolerances, Some([0.001, 0.001, 0.001]));
    }

    #[test]
    fn gcs_version3_has_domains_and_extent_but_no_precision() {
        // 版本表 [3,4,7,7] 首项：文档版本 0 写版本 3——域串 + 范围，无精度块。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&3u16.to_le_bytes());
        push_wkt(&mut bytes, "GEOGCS[\"X\"]");
        push_domains(&mut bytes, 0);
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&1.0f64.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let body = read_geographic_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert!(body.domains.is_some());
        assert_eq!(body.extent, Some([0.0, 1.0]));
        assert!(body.precision.is_none());
        assert!(body.tolerances.is_none());
    }

    #[test]
    fn gcs_version4_precision_has_no_flags_short() {
        // 版本 ==4 走 0x719bae60(0)：7 个 double、无 2 字节标志。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&4u16.to_le_bytes());
        push_wkt(&mut bytes, "GEOGCS[\"X\"]");
        push_domains(&mut bytes, 0);
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&1.0f64.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 56]);
        let mut reader = Reader::new(&bytes);
        let body = read_geographic_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(body.precision.unwrap().flags, 0);
        assert!(body.tolerances.is_none());
    }

    #[test]
    fn gcs_low_versions_read_wkt_only() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u16.to_le_bytes());
        push_wkt(&mut bytes, "GEOGCS[\"X\"]");
        let mut reader = Reader::new(&bytes);
        let body = read_geographic_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert!(body.domains.is_none());
        assert!(body.extent.is_none());
        assert!(body.precision.is_none());
    }

    #[test]
    fn gcs_vertical_flag_consumes_unit_segment() {
        // 垂向标志非零：紧跟 [u16 ≤2][u32 长度][原样字节]（0x719ea140）。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&7u16.to_le_bytes());
        push_wkt(&mut bytes, "GEOGCS[\"X\"]");
        for _ in 0..4 {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        bytes.push(1);
        bytes.extend_from_slice(&2u16.to_le_bytes());
        let unit = b"LINEAR_UNIT[\"Meter\",1.0]";
        bytes.extend_from_slice(&(unit.len() as u32).to_le_bytes());
        bytes.extend_from_slice(unit);
        bytes.extend_from_slice(&0.0f64.to_le_bytes());
        bytes.extend_from_slice(&1.0f64.to_le_bytes());
        push_precision(&mut bytes, 1);
        bytes.extend_from_slice(&[0u8; 24]);
        let mut reader = Reader::new(&bytes);
        let body = read_geographic_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        let segment = body.domains.unwrap().vertical_unit.unwrap();
        assert_eq!(segment.save_version, 2);
        assert_eq!(segment.unit_wkt, unit.to_vec());
    }

    #[test]
    fn gcs_rejects_bad_versions_and_wkt() {
        let wkt_bytes = 11u32.to_le_bytes();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&9u16.to_le_bytes());
        bytes.extend_from_slice(&wkt_bytes);
        bytes.extend_from_slice(b"GEOGCS[\"X\"]");
        let mut reader = Reader::new(&bytes);
        let before = reader.position();
        assert!(read_geographic_coordinate_system_10_1(&mut reader, 4096).is_err());
        assert_eq!(reader.position(), before);

        // 负版本位模式：原生有符号比较不拒，读取器拒绝（较原生更严）。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xffffu16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        assert!(read_geographic_coordinate_system_10_1(&mut reader, 4096).is_err());

        // WKT 非 ASCII。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&7u16.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&[0xc4, 0xe3]);
        let mut reader = Reader::new(&bytes);
        assert!(read_geographic_coordinate_system_10_1(&mut reader, 4096).is_err());
    }

    #[test]
    fn gcs_vertical_segment_version_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&7u16.to_le_bytes());
        push_wkt(&mut bytes, "GEOGCS[\"X\"]");
        for _ in 0..4 {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        bytes.push(1);
        bytes.extend_from_slice(&3u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let before = reader.position();
        assert!(read_geographic_coordinate_system_10_1(&mut reader, 4096).is_err());
        assert_eq!(reader.position(), before);
    }

    #[test]
    fn gcs_truncation_leaves_cursor_unchanged() {
        let mut full = Vec::new();
        full.extend_from_slice(&7u16.to_le_bytes());
        push_wkt(&mut full, "GEOGCS[\"X\"]");
        push_domains(&mut full, 0);
        full.extend_from_slice(&0.0f64.to_le_bytes());
        full.extend_from_slice(&1.0f64.to_le_bytes());
        push_precision(&mut full, 1);
        full.extend_from_slice(&[0u8; 24]);
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let before = reader.position();
            assert!(
                read_geographic_coordinate_system_10_1(&mut reader, 4096).is_err(),
                "截断长度 {cut} 应失败"
            );
            assert_eq!(reader.position(), before, "截断长度 {cut} 游标应不变");
        }
    }

    fn push_unk_string(bytes: &mut Vec<u8>, value: &str) {
        // AfCore String::Write 布局：cb = 字节数(含 UTF-16 终止零)。
        let mut utf16: Vec<u8> = value.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        utf16.extend_from_slice(&[0, 0]);
        bytes.extend_from_slice(&(utf16.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&utf16);
    }

    #[test]
    fn unknown_version3_full_layout_roundtrip() {
        // 10.1 写侧恒写版本 3：4 串 + 标志 + 7 double + 3 容差。
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&3u16.to_le_bytes());
        push_unk_string(&mut bytes, "Unknown");
        push_unk_string(&mut bytes, "");
        push_unk_string(&mut bytes, "");
        push_unk_string(&mut bytes, "");
        push_precision(&mut bytes, 1);
        for value in [0.001f64, 0.001, 0.001] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let mut reader = Reader::new(&bytes);
        let body = read_unknown_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(body.strings[0], "Unknown");
        assert!(body.strings[1..].iter().all(String::is_empty));
        assert_eq!(body.precision.unwrap().flags, 1);
        assert_eq!(body.tolerances, Some([0.001, 0.001, 0.001]));
    }

    #[test]
    fn unknown_version3_preserves_serialized_tolerance_bits() {
        let mut bytes = 3u16.to_le_bytes().to_vec();
        for name in ["Unknown", "", "", ""] {
            push_unk_string(&mut bytes, name);
        }
        push_precision(&mut bytes, 1);
        // ArcMap 加载时会重置这三个值。读取器应保留磁盘上的位模式。
        let bits = [
            0x7ff8_0000_0000_0042_u64,
            (-0.0_f64).to_bits(),
            (-2.0_f64).to_bits(),
        ];
        for value in bits {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let end = bytes.len();
        bytes.extend_from_slice(&[0xab, 0xcd]);

        let mut reader = Reader::new(&bytes);
        let body = read_unknown_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(body.tolerances.unwrap().map(f64::to_bits), bits);
        assert_eq!(reader.position(), end);
    }

    #[test]
    fn unknown_version1_precision_has_no_flags_short() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u16.to_le_bytes());
        for name in ["A", "B", "C", "D"] {
            push_unk_string(&mut bytes, name);
        }
        bytes.extend_from_slice(&[0u8; 56]);
        let mut reader = Reader::new(&bytes);
        let body = read_unknown_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        let precision = body.precision.unwrap();
        assert_eq!(precision.flags, 0);
        assert!(body.tolerances.is_none());
    }

    #[test]
    fn unknown_version0_reads_strings_only() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0u16.to_le_bytes());
        for name in ["A", "B", "C", "D"] {
            push_unk_string(&mut bytes, name);
        }
        let mut reader = Reader::new(&bytes);
        let body = read_unknown_coordinate_system_10_1(&mut reader, 4096).unwrap();
        assert_eq!(reader.position(), bytes.len());
        assert_eq!(
            body.strings,
            ["A".to_string(), "B".into(), "C".into(), "D".into()]
        );
        assert!(body.precision.is_none());
        assert!(body.tolerances.is_none());
    }

    #[test]
    fn unknown_rejects_version_above_3_and_truncation() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&4u16.to_le_bytes());
        let mut reader = Reader::new(&bytes);
        let before = reader.position();
        assert!(read_unknown_coordinate_system_10_1(&mut reader, 4096).is_err());
        assert_eq!(reader.position(), before);

        let mut full = Vec::new();
        full.extend_from_slice(&3u16.to_le_bytes());
        for name in ["A", "B", "C", "D"] {
            push_unk_string(&mut full, name);
        }
        push_precision(&mut full, 0);
        full.extend_from_slice(&[0u8; 24]);
        for cut in 0..full.len() {
            let mut reader = Reader::new(&full[..cut]);
            let before = reader.position();
            assert!(
                read_unknown_coordinate_system_10_1(&mut reader, 4096).is_err(),
                "截断长度 {cut} 应失败"
            );
            assert_eq!(reader.position(), before, "截断长度 {cut} 游标应不变");
        }
    }
}
