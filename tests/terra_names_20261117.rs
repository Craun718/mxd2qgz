//! 对已知严格读取策略的合成输入检查，不代表 ArcMap 运行验证。

use mxd2qgz::objects::number_format::read_angle_format_body_10_1;
use mxd2qgz::storage::object_refs::RefTable10_1;
use mxd2qgz::stream::reader::Reader;

#[test]
fn angle_rejects_invalid_core_even_when_tail_is_available() {
    // Angle v3 的 NumericFormat 版本超限，后面仍有可读的 Angle 尾字段。
    // 原生忽略核心错误；严格读取器应整体回滚，不能误把尾部当作成功解析。
    let bytes = [3, 0, 7, 0, 2, 0, 9, 0];
    let mut reader = Reader::new(&bytes);
    let mut table = RefTable10_1::new();
    table.register(42);

    assert!(read_angle_format_body_10_1(&mut reader, &mut table, 99).is_err());
    assert_eq!(reader.position(), 0);
    assert_eq!(table.len(), 1);
    assert!(table.contains(42));
}
