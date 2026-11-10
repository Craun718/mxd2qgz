//! Intermediate map representation. All source locations are stream-relative.

/// Location and bytes of an input field that has not been interpreted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownField {
    pub stream: String,
    pub offset: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub name: Option<String>,
    /// Data frames in source order.
    pub frames: Vec<DataFrame>,
    pub unknown_fields: Vec<UnknownField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
    pub name: String,
    /// Children in source order; no rendering order is assumed.
    pub children: Vec<LayerNode>,
    pub crs: Option<CoordinateSystem>,
    pub extent: Option<Extent>,
    pub unknown_fields: Vec<UnknownField>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayerNode {
    Group(LayerGroup),
    Layer(Layer),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerGroup {
    pub name: String,
    pub visible: bool,
    pub children: Vec<LayerNode>,
    pub unknown_fields: Vec<UnknownField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub source: Option<DataSource>,
    pub crs: Option<CoordinateSystem>,
    pub extent: Option<Extent>,
    pub unknown_fields: Vec<UnknownField>,
}

/// Source details as recorded; paths are not resolved or normalized here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSource {
    pub provider: Option<String>,
    pub path: Option<String>,
    pub connection: Option<String>,
    pub unknown_fields: Vec<UnknownField>,
}

/// Preserve the source definition without assuming an authority or syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinateSystem {
    pub definition: String,
    pub unknown_fields: Vec<UnknownField>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}
