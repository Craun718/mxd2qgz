//! AfCore Variant::Read 的一维 SafeArray 载荷。

use crate::stream::reader::Reader;
use std::io::{self, ErrorKind};

/// 数组变体的元素序列。
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// 调用者已经消费 VARTYPE 及适用的保留字节，传入已去掉 VT_ARRAY 的类型。
pub(crate) fn read_class_variant_array_10_1(
    reader: &mut Reader<'_>,
    vartype: u16,
    max_bytes: usize,
) -> io::Result<ClassVariantArray10_1> {
    let offset = reader.position();
    let invalid = |reason: &str| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Variant 数组 offset 0x{offset:x}: {reason}"),
        )
    };
    let width = match vartype {
        2 | 11 | 18 => 2,
        3 | 4 | 10 | 19 => 4,
        5..=7 => 8,
        16 | 17 => 1,
        8 => 4, // BSTR 元素至少占码元计数；空串无正文。
        0 | 1 => {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("VARTYPE {vartype} 无有效的 SafeArray 元素布局"),
            ));
        }
        9 | 13 => {
            // AfCore 清除数组标志后直接进入单对象 LoadObject，流内没有元素区。
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                format!("VARTYPE {vartype} 的数组没有逐对象持久化布局"),
            ));
        }
        _ => return Err(invalid("数组元素类型未知")),
    };
    let mut trial = reader.clone();
    let count =
        usize::try_from(trial.read_u32_le()? as i32).map_err(|_| invalid("数组计数为负"))?;
    let min_total = count
        .checked_mul(width)
        .ok_or_else(|| invalid("数组字节数溢出"))?;
    if min_total > max_bytes {
        return Err(invalid("数组载荷超过资源限额"));
    }
    if min_total > trial.remaining() {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Variant 数组超出可读范围",
        ));
    }
    let result = if vartype == 8 {
        let start = trial.position();
        let mut items = Vec::new();
        for _ in 0..count {
            let budget = max_bytes.saturating_sub(trial.position() - start);
            if budget < 4 {
                return Err(invalid("BSTR 数组超过资源限额"));
            }
            items.push(read_bstr_array_element_10_1(&mut trial, budget - 4)?);
        }
        ClassVariantArray10_1::Strings(items)
    } else {
        ClassVariantArray10_1::Fixed {
            vartype,
            element_bytes: width,
            raw: trial.read_bytes(min_total)?.to_vec(),
        }
    };
    *reader = trial;
    Ok(result)
}

/// 读取 `ReadString`（VariantStream/OLE 风格）布局的一个字符串元素：
/// 4 字节 UTF-16LE 码元数 `n`；`n == 0` 时无载荷（空串），否则读
/// `n*2+2` 字节并要求末尾 UTF-16LE 终止零。这是 `native_string`
/// 布局（n 为含终止零的字节长度）之外的另一种原生字符串布局，
/// 仅用于 [`ClassVariantArray10_1::Strings`] 元素。
///
/// `max_bytes` 约束 `n*2+2` 载荷（`usize::MAX` 不设限）；失败时不推进
/// 游标。
fn read_bstr_array_element_10_1(reader: &mut Reader<'_>, max_bytes: usize) -> io::Result<String> {
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
