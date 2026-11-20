//! 跨图层与数据框共享的对象体严格读取器：空间参考、范围、元素集合、
//! 数据集名称分支、属性集、元素类体与 DisplayCore 颜色/简单符号族（接线轮
//! 自 src 根级移入，消除 objects→layers 依赖倒置）。各自对应独立的证据文档。

pub mod ams_dataset_name;
pub mod callout_background;
pub mod color_ramp;
pub mod dataset_name;
pub mod dataset_name_more;
pub mod display_color;
pub mod display_symbol;
pub mod domain;
pub mod element_body;
pub mod element_collection;
pub mod element_decor;
pub mod envelope;
pub mod field_defs;
pub mod fields;
pub mod geo_transformation;
pub mod geodata_xform;
pub mod geometry;
pub mod hyperlink;
pub mod multi_patch;
pub mod multi_patch_material;
pub mod number_format;
pub mod ole_font;
pub mod picture_fill_symbol;
pub mod property_set;
pub mod query_filter;
pub mod route_event_name;
pub mod spatial_reference;
pub mod spatial_reference_slots;
pub mod string_array;
pub mod symbol_collection;
pub mod variant_object_branch;
pub mod workspace_name;
