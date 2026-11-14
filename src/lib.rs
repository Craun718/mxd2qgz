//! Format-independent representation of a map and its unresolved input data.

pub mod composite_graphics;
pub mod envelope;
pub mod feature_layer;
pub mod group_layer;
pub mod layer_list;
pub mod map_object;
pub mod maps_stream;
pub mod model;
pub mod mxd_maps;
pub mod mxd_version;
pub mod native_string;
pub mod object_refs;
pub mod object_stream;
pub mod qgis;
pub mod reader;
pub mod source_path;
pub mod spatial_reference;
pub mod variant_stream;
pub mod version_stream;

#[cfg(feature = "python")]
mod python;
