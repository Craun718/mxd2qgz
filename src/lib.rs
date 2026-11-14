//! Format-independent representation of a map and its unresolved input data.

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
