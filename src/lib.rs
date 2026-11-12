//! Format-independent representation of a map and its unresolved input data.

pub mod model;
pub mod object_stream;
pub mod reader;

#[cfg(feature = "python")]
mod python;
