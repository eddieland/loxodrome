//! PyO3 module exposing minimal bindings for smoke testing.
//!
//! PyO3 compiles this crate into a CPython extension and wires Rust
//! functions into a Python module via the `#[pymodule]` entrypoint; see
//! https://pyo3.rs/latest/ for patterns and lifecycle details.
//!
//! Keep bindings in sync: any changes here must be mirrored in
//! `pygeodist/src/geodist/_geodist_rs.pyi` in the same commit.
#![allow(unsafe_op_in_unsafe_fn)]
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3::wrap_pyfunction;

use crate::constants::EARTH_RADIUS_METERS;
use crate::{distance, hausdorff as hausdorff_kernel, types};

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
  lat: f64,
  /// Longitude in degrees east of the prime meridian. Negative values are west.
  #[pyo3(get)]
  lon: f64,
}

#[pymethods]
impl Point {
  /// Create a new geographic point.
  ///
  /// Arguments are expected in degrees and are stored as-is; callers should
  /// validate ranges in the Python layer.
  #[new]
  pub fn new(lat: f64, lon: f64) -> Self {
    Self { lat, lon }
  }

  /// Return a tuple representation for convenient unpacking.
  pub fn to_tuple(&self) -> (f64, f64) {
    (self.lat, self.lon)
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!("Point(lat={}, lon={})", self.lat, self.lon)
  }
}

/// Geographic point with altitude expressed in degrees + meters.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Point3D {
  /// Latitude in degrees north of the equator. Negative values are south.
  #[pyo3(get)]
  lat: f64,
  /// Longitude in degrees east of the prime meridian. Negative values are west.
  #[pyo3(get)]
  lon: f64,
  /// Altitude in meters relative to the reference ellipsoid.
  #[pyo3(get)]
  altitude_m: f64,
}

#[pymethods]
impl Point3D {
  /// Create a new geographic point with altitude.
  ///
  /// Arguments are expected in degrees for latitude/longitude and meters for
  /// altitude; callers should validate ranges in the Python layer.
  #[new]
  pub fn new(lat: f64, lon: f64, altitude_m: f64) -> Self {
    Self { lat, lon, altitude_m }
  }

  /// Return a tuple representation for convenient unpacking.
  pub fn to_tuple(&self) -> (f64, f64, f64) {
    (self.lat, self.lon, self.altitude_m)
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!(
      "Point3D(lat={}, lon={}, altitude_m={})",
      self.lat, self.lon, self.altitude_m
    )
  }
}

/// Distance + bearings solution for a geodesic path.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct GeodesicSolution {
  #[pyo3(get)]
  distance_meters: f64,
  #[pyo3(get)]
  initial_bearing_degrees: f64,
  #[pyo3(get)]
  final_bearing_degrees: f64,
}

impl From<distance::GeodesicSolution> for GeodesicSolution {
  fn from(value: distance::GeodesicSolution) -> Self {
    Self {
      distance_meters: value.distance().meters(),
      initial_bearing_degrees: value.initial_bearing_degrees(),
      final_bearing_degrees: value.final_bearing_degrees(),
    }
  }
}

#[pymethods]
impl GeodesicSolution {
  /// Return a tuple `(meters, initial_bearing_deg, final_bearing_deg)`.
  pub fn to_tuple(&self) -> (f64, f64, f64) {
    (
      self.distance_meters,
      self.initial_bearing_degrees,
      self.final_bearing_degrees,
    )
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!(
      "GeodesicSolution(distance_meters={}, initial_bearing_degrees={}, final_bearing_degrees={})",
      self.distance_meters, self.initial_bearing_degrees, self.final_bearing_degrees
    )
  }
}

/// Geographic bounding box used to clip point sets.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct BoundingBox {
  #[pyo3(get)]
  min_lat: f64,
  #[pyo3(get)]
  max_lat: f64,
  #[pyo3(get)]
  min_lon: f64,
  #[pyo3(get)]
  max_lon: f64,
}

#[pymethods]
impl BoundingBox {
  /// Create a new bounding box from ordered corners.
  #[new]
  pub fn new(min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> PyResult<Self> {
    let bbox = types::BoundingBox::new(min_lat, max_lat, min_lon, max_lon)
      .map_err(|err| PyValueError::new_err(err.to_string()))?;

    Ok(Self {
      min_lat: bbox.min_lat,
      max_lat: bbox.max_lat,
      min_lon: bbox.min_lon,
      max_lon: bbox.max_lon,
    })
  }

  /// Return a tuple representation for convenient unpacking.
  pub fn to_tuple(&self) -> (f64, f64, f64, f64) {
    (self.min_lat, self.max_lat, self.min_lon, self.max_lon)
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!(
      "BoundingBox(min_lat={}, max_lat={}, min_lon={}, max_lon={})",
      self.min_lat, self.max_lat, self.min_lon, self.max_lon
    )
  }
}

fn map_to_point(handle: &Point) -> PyResult<types::Point> {
  types::Point::new(handle.lat, handle.lon).map_err(|err| PyValueError::new_err(err.to_string()))
}

fn map_to_point3d(handle: &Point3D) -> PyResult<types::Point3D> {
  types::Point3D::new(handle.lat, handle.lon, handle.altitude_m).map_err(|err| PyValueError::new_err(err.to_string()))
}

fn map_to_points(handles: &[Point]) -> PyResult<Vec<types::Point>> {
  handles.iter().map(map_to_point).collect::<Result<Vec<_>, _>>()
}

fn map_to_bounding_box(handle: &BoundingBox) -> PyResult<types::BoundingBox> {
  types::BoundingBox::new(handle.min_lat, handle.max_lat, handle.min_lon, handle.max_lon)
    .map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn geodesic_distance(p1: &Point, p2: &Point) -> PyResult<f64> {
  let origin = map_to_point(p1)?;
  let destination = map_to_point(p2)?;

  let distance =
    distance::geodesic_distance(origin, destination).map_err(|err| PyValueError::new_err(err.to_string()))?;

  Ok(distance.meters())
}

#[pyfunction]
fn geodesic_with_bearings(p1: &Point, p2: &Point) -> PyResult<GeodesicSolution> {
  let origin = map_to_point(p1)?;
  let destination = map_to_point(p2)?;
  let solution =
    distance::geodesic_with_bearings(origin, destination).map_err(|err| PyValueError::new_err(err.to_string()))?;

  Ok(solution.into())
}

#[pyfunction]
fn geodesic_distance_3d(p1: &Point3D, p2: &Point3D) -> PyResult<f64> {
  let origin = map_to_point3d(p1)?;
  let destination = map_to_point3d(p2)?;
  distance::geodesic_distance_3d(origin, destination)
    .map(|distance| distance.meters())
    .map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn hausdorff_directed(a: Vec<Point>, b: Vec<Point>) -> PyResult<f64> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;

  hausdorff_kernel::hausdorff_directed(&points_a, &points_b)
    .map(|distance| distance.meters())
    .map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn hausdorff(a: Vec<Point>, b: Vec<Point>) -> PyResult<f64> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;

  hausdorff_kernel::hausdorff(&points_a, &points_b)
    .map(|distance| distance.meters())
    .map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn hausdorff_directed_clipped(a: Vec<Point>, b: Vec<Point>, bounding_box: &BoundingBox) -> PyResult<f64> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;
  let bbox = map_to_bounding_box(bounding_box)?;

  hausdorff_kernel::hausdorff_directed_clipped(&points_a, &points_b, bbox)
    .map(|distance| distance.meters())
    .map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn hausdorff_clipped(a: Vec<Point>, b: Vec<Point>, bounding_box: &BoundingBox) -> PyResult<f64> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;
  let bbox = map_to_bounding_box(bounding_box)?;

  hausdorff_kernel::hausdorff_clipped(&points_a, &points_b, bbox)
    .map(|distance| distance.meters())
    .map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pymodule]
fn _geodist_rs(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add("EARTH_RADIUS_METERS", EARTH_RADIUS_METERS)?;
  m.add_class::<Point>()?;
  m.add_class::<Point3D>()?;
  m.add_class::<GeodesicSolution>()?;
  m.add_class::<BoundingBox>()?;
  m.add_function(wrap_pyfunction!(geodesic_distance, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_with_bearings, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_clipped, m)?)?;
  Ok(())
}
