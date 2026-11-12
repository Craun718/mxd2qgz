//! Format-independent representation of a map and its unresolved input data.

pub mod maps_stream;
pub mod model;
pub mod native_string;
pub mod object_stream;
pub mod qgis;
pub mod reader;
pub mod source_path;
pub mod version_stream;

#[cfg(feature = "python")]
mod python;
