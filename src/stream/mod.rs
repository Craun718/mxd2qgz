//! 与 MXD 字段编码无关的二进制读取原语。
//!
//! 这些读取器不定位 MXD 对象，也不确认字段归属；调用点、版本及字段解释
//! 必须由调用者确认。

pub mod native_string;
pub mod reader;
pub mod variant_array;
pub mod variant_stream;
