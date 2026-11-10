use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// List OLE streams as (stream path, size in bytes) pairs.
#[pyfunction]
fn inspect(path: PathBuf) -> PyResult<Vec<(String, u64)>> {
    let document = cfb::open(path)?;
    Ok(document
        .walk()
        .filter(|entry| entry.is_stream())
        .map(|entry| (entry.path().to_string_lossy().into_owned(), entry.len()))
        .collect())
}

/// Read the complete contents of an OLE stream.
#[pyfunction]
fn read_stream<'py>(
    py: Python<'py>,
    path: PathBuf,
    stream: PathBuf,
) -> PyResult<Bound<'py, PyBytes>> {
    use std::io::Read;

    let mut document = cfb::open(path)?;
    let mut bytes = Vec::new();
    document.open_stream(stream)?.read_to_end(&mut bytes)?;
    Ok(PyBytes::new(py, &bytes))
}

#[pymodule]
fn mxd2qgz(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(inspect, module)?)?;
    module.add_function(wrap_pyfunction!(read_stream, module)?)?;
    Ok(())
}
