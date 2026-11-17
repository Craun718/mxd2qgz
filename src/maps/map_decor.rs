//! Map 的 IBorder 与 IBackground 已知实现类的事务式读取。

use crate::objects::element_decor::{
    DecorKind10_1, DecorSlot10_1, SYMBOL_BACKGROUND_CLASS_ID, SYMBOL_BORDER_CLASS_ID,
    read_decor_slot_10_1,
};
use crate::storage::object_refs::RefTable10_1;
use crate::stream::reader::Reader;
use std::io;

/// 在对象体完整且槽类匹配时提交游标与引用表；未知类或未闭合的内嵌对象保留原位。
pub fn read_map_decor_slot_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
    max_bytes: usize,
    kind: DecorKind10_1,
) -> io::Result<Option<DecorSlot10_1>> {
    let mut candidate = reader.clone();
    let mut staged = table.clone();
    let (slot, stop) = read_decor_slot_10_1(&mut candidate, max_bytes, &mut staged, 16, kind)?;
    let accepted = match (&slot, kind) {
        (Some(DecorSlot10_1::Null), _) => true,
        (Some(DecorSlot10_1::Border(_)), DecorKind10_1::Border)
        | (Some(DecorSlot10_1::Background(_)), DecorKind10_1::Background) => true,
        (Some(DecorSlot10_1::BackReference { class_id, .. }), DecorKind10_1::Border) => {
            *class_id == SYMBOL_BORDER_CLASS_ID
        }
        (Some(DecorSlot10_1::BackReference { class_id, .. }), DecorKind10_1::Background) => {
            *class_id == SYMBOL_BACKGROUND_CLASS_ID
        }
        _ => false,
    };
    if accepted && stop.is_none() {
        *reader = candidate;
        *table = staged;
        Ok(slot)
    } else {
        Ok(None)
    }
}
