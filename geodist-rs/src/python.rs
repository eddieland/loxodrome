//! PyO3 module exposing minimal bindings for smoke testing.
//!
//! PyO3 compiles this crate into a CPython extension and wires Rust
//! functions into a Python module via the `#[pymodule]` entrypoint; see
//! https://pyo3.rs/latest/ for patterns and lifecycle details.
//!
//! Keep bindings in sync: any changes here must be mirrored in
//! `pygeodist/src/geodist/_geodist_rs.pyi` in the same commit.
#![allow(unsafe_op_in_unsafe_fn)]
use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::constants::EARTH_RADIUS_METERS;

/// Geographic point expressed in degrees.
///
/// The struct is intentionally minimal and opaque to Python callers;
/// higher-level validation happens in the Python wrapper to keep this layer
/// thin.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Point {
  /// Latitude in degrees north of the equator. Negative values are south.
  #[pyo3(get)]
  latitude_degrees: f64,
  /// Longitude in degrees east of the prime meridian. Negative values are west.
  #[pyo3(get)]
  longitude_degrees: f64,
}

#[pymethods]
impl Point {
  /// Create a new geographic point.
  ///
  /// Arguments are expected in degrees and are stored as-is; callers should
  /// validate ranges in the Python layer.
  #[new]
  pub fn new(latitude_degrees: f64, longitude_degrees: f64) -> Self {
    Self {
      latitude_degrees,
      longitude_degrees,
    }
  }

  /// Return a tuple representation for convenient unpacking.
  pub fn to_tuple(&self) -> (f64, f64) {
    (self.latitude_degrees, self.longitude_degrees)
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!(
      "Point(latitude_degrees={}, longitude_degrees={})",
      self.latitude_degrees, self.longitude_degrees
    )
  }
}

#[pymodule]
fn _geodist_rs(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add("EARTH_RADIUS_METERS", EARTH_RADIUS_METERS)?;
  m.add_class::<Point>()?;
  Ok(())
}
