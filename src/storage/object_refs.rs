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

use crate::storage::object_stream::{ObjectHeader, read_header_10_1};
use crate::stream::reader::Reader;
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::Arc;

/// 从已确认的数据集名称体提取的路径与名称原文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetSource10_1 {
    pub name: Arc<str>,
    pub path: Option<Arc<str>>,
    pub band_name: Option<Arc<str>>,
    pub band_indices: Option<Arc<[i32]>>,
}

/// 跨流共享的已见键登记表（对应原生 ObjectStream 读表）。
///
/// 原生 `SetStream` 不清空该表，因此同一文档会话的全部流共用一个实例。
/// 本表保留已见键、已知类标识及显式重复登记次数，不保存 COM 对象指针。
/// 写侧插入器另有同键替换分支；正常读侧先查表，命中后直接作为回引返回，
/// 不会再次登记。因此这里的重复登记计数不能当作写侧替换行为的模拟。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RefTable10_1 {
    seen: HashMap<u32, (u32, Option<[u8; 16]>)>,
    map_keys: HashSet<u32>,
    page_element_keys: HashSet<u32>,
    workspace_paths: HashMap<u32, Arc<str>>,
    feature_dataset_paths: HashMap<u32, Arc<str>>,
    dataset_sources: HashMap<([u8; 16], u32), DatasetSource10_1>,
    raster_dataset_names_with_null_workspace: HashSet<u32>,
    raster_names_without_open_result: HashSet<u32>,
    table_names_with_null_workspace: HashSet<u32>,
    string_arrays: HashMap<u32, (u16, usize, Arc<[String]>)>,
    long_arrays: HashMap<u32, (u16, Arc<[i32]>)>,
    multi_patch_marker_strings: HashMap<u32, Arc<str>>,
    legend_group_counts: HashMap<u32, usize>,
    // 共享键表的试读副本也继承跨类图形对象的当前递归深度。
    graphics_nesting: u8,
    // 数据集名称和关联名称也可能经不同槽互相嵌套。
    dataset_name_nesting: u8,
    // 栅格信息可以经 Variant、工作空间或压缩流重新进入自身。
    raster_info_nesting: u8,
    raster_info_variant_depth: usize,
}

impl RefTable10_1 {
    /// 空登记表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 图形容器可经图层和标注属性互相嵌套；限制新对象层数，回引不占用深度。
    pub(crate) fn with_nested_graphics<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        const MAX_GRAPHICS_NESTING: u8 = 8;
        let depth = self.graphics_nesting;
        if depth >= MAX_GRAPHICS_NESTING {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("图形容器嵌套深度超过读取器限额 {MAX_GRAPHICS_NESTING}"),
            ));
        }
        self.graphics_nesting = depth + 1;
        let result = read(self);
        self.graphics_nesting = depth;
        result
    }

    pub(crate) fn with_nested_dataset_name<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        const MAX_DATASET_NAME_NESTING: u8 = 32;
        let depth = self.dataset_name_nesting;
        if depth >= MAX_DATASET_NAME_NESTING {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("数据集名称嵌套深度超过读取器限额 {MAX_DATASET_NAME_NESTING}"),
            ));
        }
        self.dataset_name_nesting = depth + 1;
        let result = read(self);
        self.dataset_name_nesting = depth;
        result
    }

    pub(crate) fn with_nested_raster_info<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        const MAX_RASTER_INFO_NESTING: u8 = 16;
        let depth = self.raster_info_nesting;
        if depth >= MAX_RASTER_INFO_NESTING {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "栅格信息对象跨流嵌套超过限额",
            ));
        }
        self.raster_info_nesting = depth + 1;
        let result = read(self);
        self.raster_info_nesting = depth;
        result
    }

    /// 合并栅格的压缩子流重新分配键，继续受外层栅格信息对象的嵌套限额约束。
    pub(crate) fn isolated_raster_stream(&self) -> Self {
        Self {
            raster_info_nesting: self.raster_info_nesting,
            raster_info_variant_depth: self.raster_info_variant_depth,
            ..Self::default()
        }
    }

    pub(crate) fn raster_info_variant_depth(&self) -> usize {
        self.raster_info_variant_depth
    }

    /// 普通对象、子流或新属性集再次读取 Variant 时，继续计算外层 Variant 的深度。
    pub(crate) fn with_raster_info_variant_depth<T>(
        &mut self,
        depth: usize,
        read: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let old_depth = self.raster_info_variant_depth;
        self.raster_info_variant_depth = depth;
        let result = read(self);
        self.raster_info_variant_depth = old_depth;
        result
    }

    /// 标记经真实对象头确认的 Map，供嵌套布局校验 IMap 回引。
    pub(crate) fn mark_map(&mut self, key: u32) {
        debug_assert!(self.contains(key));
        self.map_keys.insert(key);
    }

    pub(crate) fn contains_map(&self, key: u32) -> bool {
        self.map_keys.contains(&key)
    }

    /// 记录已从 PageLayout 元素槽成功读取的对象键。
    pub(crate) fn mark_page_element(&mut self, key: u32) {
        debug_assert!(self.contains(key));
        self.page_element_keys.insert(key);
    }

    pub(crate) fn contains_page_element(&self, key: u32) -> bool {
        self.page_element_keys.contains(&key)
    }

    /// 只记录完整读取且通过对象头登记的图例组。
    pub(crate) fn mark_legend_group(&mut self, key: u32, count: usize) {
        if self.contains(key) {
            self.legend_group_counts.insert(key, count);
        }
    }

    pub(crate) fn legend_group_count(&self, key: u32) -> Option<usize> {
        self.legend_group_counts.get(&key).copied()
    }

    /// 只缓存完整读取且已登记的 WorkspaceName 路径。
    pub(crate) fn mark_workspace_path(&mut self, key: u32, path: &str) {
        if self.contains(key) {
            self.workspace_paths.insert(key, Arc::from(path));
        }
    }

    pub(crate) fn workspace_path(&self, key: u32) -> Option<&str> {
        self.workspace_paths.get(&key).map(AsRef::as_ref)
    }

    pub(crate) fn mark_feature_dataset_path(&mut self, key: u32, path: &str) {
        if self.contains(key) {
            self.feature_dataset_paths.insert(key, Arc::from(path));
        }
    }

    pub(crate) fn feature_dataset_path(&self, key: u32) -> Option<&str> {
        self.feature_dataset_paths.get(&key).map(AsRef::as_ref)
    }

    pub(crate) fn mark_raster_dataset_name_with_null_workspace(&mut self, key: u32) {
        self.raster_dataset_names_with_null_workspace.insert(key);
    }

    pub(crate) fn raster_dataset_name_has_null_workspace(&self, key: u32) -> bool {
        self.raster_dataset_names_with_null_workspace.contains(&key)
    }

    pub(crate) fn mark_raster_name_without_open_result(&mut self, key: u32) {
        self.raster_names_without_open_result.insert(key);
    }

    pub(crate) fn raster_name_opens_without_result(&self, key: u32) -> bool {
        self.raster_names_without_open_result.contains(&key)
    }

    pub(crate) fn mark_table_name_with_null_workspace(&mut self, key: u32) {
        self.table_names_with_null_workspace.insert(key);
    }

    pub(crate) fn table_name_has_null_workspace(&self, key: u32) -> bool {
        self.table_names_with_null_workspace.contains(&key)
    }

    pub(crate) fn mark_dataset_source(
        &mut self,
        class_id: [u8; 16],
        key: u32,
        name: &str,
        path: Option<&str>,
        band_name: Option<&str>,
    ) {
        if self.contains(key) {
            self.dataset_sources.insert(
                (class_id, key),
                DatasetSource10_1 {
                    name: Arc::from(name),
                    path: path.map(Arc::from),
                    band_name: band_name.map(Arc::from),
                    band_indices: None,
                },
            );
        }
    }

    pub(crate) fn mark_band_collection_source(
        &mut self,
        class_id: [u8; 16],
        key: u32,
        parent: Option<DatasetSource10_1>,
        indices: Option<&[i32]>,
    ) {
        if self.contains(key) {
            self.dataset_sources.insert(
                (class_id, key),
                DatasetSource10_1 {
                    name: parent
                        .as_ref()
                        .map(|source| source.name.clone())
                        .unwrap_or_default(),
                    path: parent.and_then(|source| source.path),
                    band_name: None,
                    band_indices: indices.map(Arc::from),
                },
            );
        }
    }

    pub(crate) fn dataset_source(
        &self,
        class_id: [u8; 16],
        key: u32,
    ) -> Option<&DatasetSource10_1> {
        self.dataset_sources.get(&(class_id, key))
    }

    pub(crate) fn mark_string_array(
        &mut self,
        key: u32,
        version: u16,
        bytes: usize,
        values: &[String],
    ) {
        self.string_arrays
            .insert(key, (version, bytes, Arc::from(values)));
    }

    pub(crate) fn string_array(&self, key: u32, max: usize) -> Option<(u16, Vec<String>)> {
        self.string_arrays
            .get(&key)
            .filter(|(_, bytes, _)| *bytes <= max)
            .map(|(version, _, values)| (*version, values.to_vec()))
    }

    pub(crate) fn mark_long_array(&mut self, key: u32, version: u16, values: &[i32]) {
        if self.contains(key) {
            self.long_arrays.insert(key, (version, Arc::from(values)));
        }
    }

    pub(crate) fn long_array(&self, key: u32) -> Option<(u16, &[i32])> {
        self.long_arrays
            .get(&key)
            .map(|(version, values)| (*version, values.as_ref()))
    }

    pub(crate) fn remember_multi_patch_marker_string(&mut self, key: u32, value: String) {
        self.multi_patch_marker_strings
            .insert(key, Arc::from(value));
    }

    pub(crate) fn multi_patch_marker_string(&self, key: u32) -> Option<&str> {
        self.multi_patch_marker_strings.get(&key).map(AsRef::as_ref)
    }

    /// 键是否已在本会话登记过；命中即回引。
    pub fn contains(&self, key: u32) -> bool {
        self.seen.contains_key(&key)
    }

    /// 登记键；返回 `true` 表示该键此前已登记。
    ///
    /// 新对象必须在解析对象体之前登记：原生登记先于持久化，循环引用的
    /// 解析依赖此顺序。重复次数是本读取器的观测值，不代表原生对象被替换；
    /// 正常条目读取遇到已见键时会返回回引，不调用本方法。
    pub fn register(&mut self, key: u32) -> bool {
        match self.seen.get_mut(&key) {
            Some((count, _)) => {
                *count = count.saturating_add(1);
                true
            }
            None => {
                self.seen.insert(key, (1, None));
                false
            }
        }
    }

    /// 读取该键首次登记的类标识。仅登记了键时没有类信息。
    pub(crate) fn registered_class(&self, key: u32) -> Option<[u8; 16]> {
        self.seen.get(&key).and_then(|(_, class_id)| *class_id)
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
                table.seen.insert(key, (1, Some(class_id)));
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
