use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::constants::EARTH_RADIUS_METERS;

/// PyO3 module exposing minimal bindings for smoke testing.
#[pymodule]
fn _geodist_rs(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("EARTH_RADIUS_METERS", EARTH_RADIUS_METERS)?;
    Ok(())
}
