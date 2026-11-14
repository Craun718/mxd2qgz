//! Format-independent representation of a map and its unresolved input data.

pub mod layers;
pub mod maps;
pub mod model;
pub mod objects;
pub mod output;
pub mod storage;
pub mod stream;
pub mod version;

#[cfg(feature = "python")]
mod python;
