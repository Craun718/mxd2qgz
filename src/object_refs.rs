//! 10.1 ObjectStream 对象条目分类与键登记状态机。
//!
//! 原生证据见 docs/arcmap-10.1-object-body.md「10.1 对象键分配与回引写侧闭合
//! （2026-11-14 第五轮）」：写侧键是 ObjectStream 对象 `+0x2c` 的 4 字节单调
//! 计数器，新对象写当前值作键、写后 +1；回引条目与新对象条目同构（16 字节
//! ClassID GUID + 4 字节既有键），无对象体、无任何专用标记；流内不能局部区分
//! 回引与新对象，唯一判据是键是否已在本 ObjectStream 会话（跨全部流，原生
//! `SetStream` 不清表不清计数）出现过；登记先于对象体持久化，因此循环引用
//! 天然表示为键回引，无需专门编码。
//!
//! 本模块是上述规则的安全读取子集：只做条目分类与键登记，不解析对象体、
//! 不加载 COM 对象、不校验类标识是否注册。在该编码下「缺失引用」无法与新
//! 对象区分——未见过键一律按新对象分类，这是有意保留的原生语义。全部为
//! 静态证据，未经目标软件运行验证。

use crate::object_stream::{ObjectHeader, read_header_10_1};
use crate::reader::Reader;
use std::collections::HashMap;
use std::io;

/// 跨流共享的已见键登记表（对应原生 ObjectStream 读表）。
///
/// 原生 `SetStream` 不清空该表，因此同一文档会话的全部流共用一个实例。
/// 键重复登记对应写侧键重复替换分支的读侧对偶：覆盖旧登记并保留替换次数。
#[derive(Debug, Clone, Default)]
pub struct RefTable10_1 {
    seen: HashMap<u32, u32>,
}

impl RefTable10_1 {
    /// 空登记表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 键是否已在本会话登记过；命中即回引。
    pub fn contains(&self, key: u32) -> bool {
        self.seen.contains_key(&key)
    }

    /// 登记键；返回 `true` 表示该键此前已存在（重复登记=替换）。
    ///
    /// 新对象必须在解析对象体之前登记：原生登记先于持久化，循环引用的
    /// 解析依赖此顺序。值字段保留替换次数（饱和计数，仅观测用途）。
    pub fn register(&mut self, key: u32) -> bool {
        match self.seen.get_mut(&key) {
            Some(count) => {
                *count = count.saturating_add(1);
                true
            }
            None => {
                self.seen.insert(key, 1);
                false
            }
        }
    }

    /// 当前已登记的不同键数。
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// 登记表是否为空。
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

/// 单个对象条目的分类结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectEntry10_1 {
    /// 16 字节全零类标识：空引用标记，无键、无对象体。
    Null,
    /// 键已在本会话出现过：回引条目，其后没有对象体。
    BackReference { class_id: [u8; 16], key: u32 },
    /// 未见过的新键：其后是未知长度的对象体。[`read_object_entry_10_1`]
    /// 返回前已把键登记进登记表（登记先于对象体解析）。
    NewObject { class_id: [u8; 16], key: u32 },
}

/// 读取一个对象条目并按已见键集合分类；失败时不推进游标、不修改登记表。
///
/// 调用者须已确认对象起点及 10.1 版本。类标识按 Windows 磁盘字节顺序原样
/// 保留。新对象条目在返回前登记，调用者随后自行解析未知长度的对象体；
/// 回引条目之后没有对象体。同一键的回引判定只依赖登记表状态：空表会把
/// 实际的回引字节分类为 `NewObject`（流内无法局部区分，与原生一致）。
pub fn read_object_entry_10_1(
    reader: &mut Reader<'_>,
    table: &mut RefTable10_1,
) -> io::Result<ObjectEntry10_1> {
    let header = read_header_10_1(reader)?;
    match header {
        ObjectHeader::Null => Ok(ObjectEntry10_1::Null),
        ObjectHeader::Object { class_id, key } => {
            if table.contains(key) {
                Ok(ObjectEntry10_1::BackReference { class_id, key })
            } else {
                table.register(key);
                Ok(ObjectEntry10_1::NewObject { class_id, key })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_bytes(class_id_byte: u8, key: u32) -> Vec<u8> {
        let mut bytes = vec![class_id_byte; 16];
        bytes.extend(key.to_le_bytes());
        bytes
    }

    #[test]
    fn classifies_null_new_and_back_reference_entries() {
        let mut table = RefTable10_1::new();
        // 空引用标记：16 字节零，无键。
        let mut reader = Reader::new(&[0_u8; 16]);
        assert_eq!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::Null
        );
        assert!(table.is_empty());
        // 新键：分类为新对象，返回前已登记。
        let bytes = entry_bytes(0xa5, 29);
        let mut reader = Reader::new(&bytes);
        let entry = read_object_entry_10_1(&mut reader, &mut table).unwrap();
        assert_eq!(
            entry,
            ObjectEntry10_1::NewObject {
                class_id: [0xa5; 16],
                key: 29
            }
        );
        assert_eq!(reader.position(), 20);
        assert!(table.contains(29));
        // 同键再现：回引，不重复登记。
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::BackReference {
                class_id: [0xa5; 16],
                key: 29
            }
        );
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn empty_table_cannot_distinguish_a_back_reference_from_a_new_object() {
        // 流内无法局部区分回引与新对象：同一回引字节在空表下按新对象分类。
        let bytes = entry_bytes(0xa5, 20);
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::NewObject {
                class_id: [0xa5; 16],
                key: 20
            }
        );
    }

    #[test]
    fn cyclic_bodies_resolve_without_special_encoding() {
        // 合成循环：对象 A（键 1）的体内引用对象 B（键 2），B 的体内又引用 A。
        // 登记先于对象体解析（读取器返回前登记），祖先引用自然命中回引。
        let body_of_a = {
            let mut bytes = entry_bytes(0x11, 1); // A 头
            bytes.extend(entry_bytes(0x22, 2)); // A 体内：B 头（新）
            bytes.extend(entry_bytes(0x11, 1)); // B 体内：对 A 的回引
            bytes
        };
        let mut table = RefTable10_1::new();
        let mut reader = Reader::new(&body_of_a);
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::NewObject { key: 1, .. }
        ));
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::NewObject { key: 2, .. }
        ));
        assert_eq!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::BackReference {
                class_id: [0x11; 16],
                key: 1
            }
        );
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn self_reference_and_repeated_registration_behave_as_documented() {
        // 自引用：同一对象体内的自身回引。
        let mut table = RefTable10_1::new();
        let bytes = entry_bytes(0x33, 5);
        let mut reader = Reader::new(&bytes);
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::NewObject { key: 5, .. }
        ));
        let mut reader = Reader::new(&bytes);
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::BackReference { key: 5, .. }
        ));
        // 重复登记=替换：显式 register 返回 true，不影响回引判定。
        assert!(table.register(5));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn registration_survives_across_independent_readers_like_setstream() {
        // 原生 SetStream 不清表：同一会话换流后，此前登记的键仍是回引。
        let mut table = RefTable10_1::new();
        let bytes = entry_bytes(0x44, 2139);
        let mut reader = Reader::new(&bytes);
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::NewObject { key: 2139, .. }
        ));
        // 新"流"：新 Reader，共用登记表。
        let mut reader = Reader::new(&bytes);
        assert!(matches!(
            read_object_entry_10_1(&mut reader, &mut table).unwrap(),
            ObjectEntry10_1::BackReference { key: 2139, .. }
        ));
    }

    #[test]
    fn truncated_or_short_input_leaves_table_and_cursor_unchanged() {
        let mut table = RefTable10_1::new();
        table.register(7);
        let bytes = entry_bytes(0xa5, 8);
        for end in 0..bytes.len() {
            let mut table = table.clone();
            let mut reader = Reader::new(&bytes[..end]);
            assert!(read_object_entry_10_1(&mut reader, &mut table).is_err());
            assert_eq!(reader.position(), 0);
            // 失败路径不得修改登记表。
            assert_eq!(table.len(), 1);
            assert!(table.contains(7));
            assert!(!table.contains(8));
        }
    }
}
