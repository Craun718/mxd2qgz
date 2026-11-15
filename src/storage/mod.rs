//! 对象存储编码：对象流头部的局部读取规则与对象键引用表状态机。

pub mod object_refs;
pub mod object_stream;

/// 类标识的 32 字符小写十六进制串（诊断用；Windows 磁盘字节顺序原样转换）。
///
/// 原在 `layers::layer_list`（接线轮移入 storage 消除 objects→layers 依赖倒
/// 置；`layer_list` 保留 `pub use` 重导出）。
pub fn class_id_hex(class_id: &[u8; 16]) -> String {
    class_id.iter().map(|byte| format!("{byte:02x}")).collect()
}
