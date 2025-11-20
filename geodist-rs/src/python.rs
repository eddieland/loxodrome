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
use pyo3::{PyErr, create_exception, wrap_pyfunction};

use crate::constants::EARTH_RADIUS_METERS;
use crate::{distance, hausdorff as hausdorff_kernel, types};

create_exception!(_geodist_rs, GeodistError, PyValueError);
create_exception!(_geodist_rs, InvalidLatitudeError, GeodistError);
create_exception!(_geodist_rs, InvalidLongitudeError, GeodistError);
create_exception!(_geodist_rs, InvalidAltitudeError, GeodistError);
create_exception!(_geodist_rs, InvalidDistanceError, GeodistError);
create_exception!(_geodist_rs, InvalidRadiusError, GeodistError);
create_exception!(_geodist_rs, InvalidEllipsoidError, GeodistError);
create_exception!(_geodist_rs, InvalidBoundingBoxError, GeodistError);
create_exception!(_geodist_rs, EmptyPointSetError, GeodistError);

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct HausdorffDirectedWitness {
  #[pyo3(get)]
  distance_m: f64,
  #[pyo3(get)]
  origin_index: usize,
  #[pyo3(get)]
  candidate_index: usize,
}

impl From<hausdorff_kernel::HausdorffDirectedWitness> for HausdorffDirectedWitness {
  fn from(value: hausdorff_kernel::HausdorffDirectedWitness) -> Self {
    Self {
      distance_m: value.distance().meters(),
      origin_index: value.origin_index(),
      candidate_index: value.candidate_index(),
    }
  }
}

#[pymethods]
impl HausdorffDirectedWitness {
  /// Return a tuple `(distance_m, origin_index, candidate_index)`.
  pub const fn to_tuple(&self) -> (f64, usize, usize) {
    (self.distance_m, self.origin_index, self.candidate_index)
  }

  fn __repr__(&self) -> String {
    format!(
      "HausdorffDirectedWitness(distance_m={}, origin_index={}, candidate_index={})",
      self.distance_m, self.origin_index, self.candidate_index
    )
  }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct HausdorffWitness {
  #[pyo3(get)]
  distance_m: f64,
  #[pyo3(get)]
  a_to_b: HausdorffDirectedWitness,
  #[pyo3(get)]
  b_to_a: HausdorffDirectedWitness,
}

impl From<hausdorff_kernel::HausdorffWitness> for HausdorffWitness {
  fn from(value: hausdorff_kernel::HausdorffWitness) -> Self {
    Self {
      distance_m: value.distance().meters(),
      a_to_b: value.a_to_b().into(),
      b_to_a: value.b_to_a().into(),
    }
  }
}

#[pymethods]
impl HausdorffWitness {
  /// Return a tuple `(distance_m, a_to_b, b_to_a)` where the latter two
  /// are witness tuples.
  pub const fn to_tuple(&self) -> (f64, (f64, usize, usize), (f64, usize, usize)) {
    (self.distance_m, self.a_to_b.to_tuple(), self.b_to_a.to_tuple())
  }

  fn __repr__(&self) -> String {
    let (dist, (a_dist, a_origin, a_candidate), (b_dist, b_origin, b_candidate)) = self.to_tuple();
    format!(
      "HausdorffWitness(distance_m={}, a_to_b=(distance_m={}, origin_index={}, candidate_index={}), \
       b_to_a=(distance_m={}, origin_index={}, candidate_index={}))",
      dist, a_dist, a_origin, a_candidate, b_dist, b_origin, b_candidate
    )
  }
}

/// Oblate ellipsoid expressed via semi-major/minor axes (meters).
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Ellipsoid {
  #[pyo3(get)]
  semi_major_axis_m: f64,
  #[pyo3(get)]
  semi_minor_axis_m: f64,
}

#[pymethods]
impl Ellipsoid {
  /// Create a new ellipsoid from semi-major/minor axes in meters.
  #[new]
  pub fn new(semi_major_axis_m: f64, semi_minor_axis_m: f64) -> PyResult<Self> {
    let ellipsoid = map_geodist_result(types::Ellipsoid::new(semi_major_axis_m, semi_minor_axis_m))?;
    Ok(Self {
      semi_major_axis_m: ellipsoid.semi_major_axis_m,
      semi_minor_axis_m: ellipsoid.semi_minor_axis_m,
    })
  }

  /// WGS84 ellipsoid parameters in meters.
  #[staticmethod]
  pub const fn wgs84() -> Self {
    let ellipsoid = types::Ellipsoid::wgs84();
    Self {
      semi_major_axis_m: ellipsoid.semi_major_axis_m,
      semi_minor_axis_m: ellipsoid.semi_minor_axis_m,
    }
  }

  /// Return a tuple `(semi_major_axis_m, semi_minor_axis_m)`.
  pub const fn to_tuple(&self) -> (f64, f64) {
    (self.semi_major_axis_m, self.semi_minor_axis_m)
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!(
      "Ellipsoid(semi_major_axis_m={}, semi_minor_axis_m={})",
      self.semi_major_axis_m, self.semi_minor_axis_m
    )
  }
}

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
  pub const fn new(lat: f64, lon: f64) -> Self {
    Self { lat, lon }
  }

  /// Return a tuple representation for convenient unpacking.
  pub const fn to_tuple(&self) -> (f64, f64) {
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
  pub const fn new(lat: f64, lon: f64, altitude_m: f64) -> Self {
    Self { lat, lon, altitude_m }
  }

  /// Return a tuple representation for convenient unpacking.
  pub const fn to_tuple(&self) -> (f64, f64, f64) {
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
  distance_m: f64,
  #[pyo3(get)]
  initial_bearing_deg: f64,
  #[pyo3(get)]
  final_bearing_deg: f64,
}

impl From<distance::GeodesicSolution> for GeodesicSolution {
  fn from(value: distance::GeodesicSolution) -> Self {
    Self {
      distance_m: value.distance().meters(),
      initial_bearing_deg: value.initial_bearing_deg(),
      final_bearing_deg: value.final_bearing_deg(),
    }
  }
}

#[pymethods]
impl GeodesicSolution {
  /// Return a tuple `(meters, initial_bearing_deg, final_bearing_deg)`.
  pub const fn to_tuple(&self) -> (f64, f64, f64) {
    (self.distance_m, self.initial_bearing_deg, self.final_bearing_deg)
  }

  /// Human-friendly representation for debugging.
  fn __repr__(&self) -> String {
    format!(
      "GeodesicSolution(distance_m={}, initial_bearing_deg={}, final_bearing_deg={})",
      self.distance_m, self.initial_bearing_deg, self.final_bearing_deg
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
    let bbox = map_geodist_result(types::BoundingBox::new(min_lat, max_lat, min_lon, max_lon))?;

    Ok(Self {
      min_lat: bbox.min_lat,
      max_lat: bbox.max_lat,
      min_lon: bbox.min_lon,
      max_lon: bbox.max_lon,
    })
  }

  /// Return a tuple representation for convenient unpacking.
  pub const fn to_tuple(&self) -> (f64, f64, f64, f64) {
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

fn map_geodist_error(err: types::GeodistError) -> PyErr {
  let message = err.to_string();

  match err {
    types::GeodistError::InvalidLatitude(_) => InvalidLatitudeError::new_err(message),
    types::GeodistError::InvalidLongitude(_) => InvalidLongitudeError::new_err(message),
    types::GeodistError::InvalidAltitude(_) => InvalidAltitudeError::new_err(message),
    types::GeodistError::InvalidDistance(_) => InvalidDistanceError::new_err(message),
    types::GeodistError::InvalidRadius(_) => InvalidRadiusError::new_err(message),
    types::GeodistError::InvalidEllipsoid { .. } => InvalidEllipsoidError::new_err(message),
    types::GeodistError::InvalidBoundingBox { .. } => InvalidBoundingBoxError::new_err(message),
    types::GeodistError::EmptyPointSet => EmptyPointSetError::new_err(message),
  }
}

fn map_geodist_result<T>(result: Result<T, types::GeodistError>) -> PyResult<T> {
  result.map_err(map_geodist_error)
}

fn map_to_point(handle: &Point) -> PyResult<types::Point> {
  map_geodist_result(types::Point::new(handle.lat, handle.lon))
}

fn map_to_point3d(handle: &Point3D) -> PyResult<types::Point3D> {
  map_geodist_result(types::Point3D::new(handle.lat, handle.lon, handle.altitude_m))
}

fn map_to_points(handles: &[Point]) -> PyResult<Vec<types::Point>> {
  handles.iter().map(map_to_point).collect::<Result<Vec<_>, _>>()
}

fn map_to_points3d(handles: &[Point3D]) -> PyResult<Vec<types::Point3D>> {
  handles.iter().map(map_to_point3d).collect::<Result<Vec<_>, _>>()
}

fn map_to_bounding_box(handle: &BoundingBox) -> PyResult<types::BoundingBox> {
  map_geodist_result(types::BoundingBox::new(
    handle.min_lat,
    handle.max_lat,
    handle.min_lon,
    handle.max_lon,
  ))
}

fn map_to_ellipsoid(handle: &Ellipsoid) -> PyResult<types::Ellipsoid> {
  map_geodist_result(types::Ellipsoid::new(
    handle.semi_major_axis_m,
    handle.semi_minor_axis_m,
  ))
}

#[pyfunction]
fn geodesic_distance(p1: &Point, p2: &Point) -> PyResult<f64> {
  let origin = map_to_point(p1)?;
  let destination = map_to_point(p2)?;

  let distance = map_geodist_result(distance::geodesic_distance(origin, destination))?;

  Ok(distance.meters())
}

#[pyfunction]
fn geodesic_distance_on_ellipsoid(p1: &Point, p2: &Point, ellipsoid: &Ellipsoid) -> PyResult<f64> {
  let origin = map_to_point(p1)?;
  let destination = map_to_point(p2)?;
  let ellipsoid = map_to_ellipsoid(ellipsoid)?;

  distance::geodesic_distance_on_ellipsoid(ellipsoid, origin, destination)
    .map(|distance| distance.meters())
    .map_err(map_geodist_error)
}

#[pyfunction]
fn geodesic_with_bearings(p1: &Point, p2: &Point) -> PyResult<GeodesicSolution> {
  let origin = map_to_point(p1)?;
  let destination = map_to_point(p2)?;
  let solution = map_geodist_result(distance::geodesic_with_bearings(origin, destination))?;

  Ok(solution.into())
}

#[pyfunction]
fn geodesic_with_bearings_on_ellipsoid(p1: &Point, p2: &Point, ellipsoid: &Ellipsoid) -> PyResult<GeodesicSolution> {
  let origin = map_to_point(p1)?;
  let destination = map_to_point(p2)?;
  let ellipsoid = map_to_ellipsoid(ellipsoid)?;

  distance::geodesic_with_bearings_on_ellipsoid(ellipsoid, origin, destination)
    .map(GeodesicSolution::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn geodesic_distance_3d(p1: &Point3D, p2: &Point3D) -> PyResult<f64> {
  let origin = map_to_point3d(p1)?;
  let destination = map_to_point3d(p2)?;
  distance::geodesic_distance_3d(origin, destination)
    .map(|distance| distance.meters())
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_directed(a: Vec<Point>, b: Vec<Point>) -> PyResult<HausdorffDirectedWitness> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;

  hausdorff_kernel::hausdorff_directed(&points_a, &points_b)
    .map(HausdorffDirectedWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff(a: Vec<Point>, b: Vec<Point>) -> PyResult<HausdorffWitness> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;

  hausdorff_kernel::hausdorff(&points_a, &points_b)
    .map(HausdorffWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_directed_clipped(
  a: Vec<Point>,
  b: Vec<Point>,
  bounding_box: &BoundingBox,
) -> PyResult<HausdorffDirectedWitness> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;
  let bbox = map_to_bounding_box(bounding_box)?;

  hausdorff_kernel::hausdorff_directed_clipped(&points_a, &points_b, bbox)
    .map(HausdorffDirectedWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_clipped(a: Vec<Point>, b: Vec<Point>, bounding_box: &BoundingBox) -> PyResult<HausdorffWitness> {
  let points_a = map_to_points(&a)?;
  let points_b = map_to_points(&b)?;
  let bbox = map_to_bounding_box(bounding_box)?;

  hausdorff_kernel::hausdorff_clipped(&points_a, &points_b, bbox)
    .map(HausdorffWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_directed_3d(a: Vec<Point3D>, b: Vec<Point3D>) -> PyResult<HausdorffDirectedWitness> {
  let points_a = map_to_points3d(&a)?;
  let points_b = map_to_points3d(&b)?;

  hausdorff_kernel::hausdorff_directed_3d(&points_a, &points_b)
    .map(HausdorffDirectedWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_3d(a: Vec<Point3D>, b: Vec<Point3D>) -> PyResult<HausdorffWitness> {
  let points_a = map_to_points3d(&a)?;
  let points_b = map_to_points3d(&b)?;

  hausdorff_kernel::hausdorff_3d(&points_a, &points_b)
    .map(HausdorffWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_directed_clipped_3d(
  a: Vec<Point3D>,
  b: Vec<Point3D>,
  bounding_box: &BoundingBox,
) -> PyResult<HausdorffDirectedWitness> {
  let points_a = map_to_points3d(&a)?;
  let points_b = map_to_points3d(&b)?;
  let bbox = map_to_bounding_box(bounding_box)?;

  hausdorff_kernel::hausdorff_directed_clipped_3d(&points_a, &points_b, bbox)
    .map(HausdorffDirectedWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
fn hausdorff_clipped_3d(a: Vec<Point3D>, b: Vec<Point3D>, bounding_box: &BoundingBox) -> PyResult<HausdorffWitness> {
  let points_a = map_to_points3d(&a)?;
  let points_b = map_to_points3d(&b)?;
  let bbox = map_to_bounding_box(bounding_box)?;

  hausdorff_kernel::hausdorff_clipped_3d(&points_a, &points_b, bbox)
    .map(HausdorffWitness::from)
    .map_err(map_geodist_error)
}

#[pymodule]
fn _geodist_rs(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add("EARTH_RADIUS_METERS", EARTH_RADIUS_METERS)?;
  m.add("GeodistError", py.get_type::<GeodistError>())?;
  m.add("InvalidLatitudeError", py.get_type::<InvalidLatitudeError>())?;
  m.add("InvalidLongitudeError", py.get_type::<InvalidLongitudeError>())?;
  m.add("InvalidAltitudeError", py.get_type::<InvalidAltitudeError>())?;
  m.add("InvalidDistanceError", py.get_type::<InvalidDistanceError>())?;
  m.add("InvalidRadiusError", py.get_type::<InvalidRadiusError>())?;
  m.add("InvalidEllipsoidError", py.get_type::<InvalidEllipsoidError>())?;
  m.add("InvalidBoundingBoxError", py.get_type::<InvalidBoundingBoxError>())?;
  m.add("EmptyPointSetError", py.get_type::<EmptyPointSetError>())?;
  m.add_class::<Ellipsoid>()?;
  m.add_class::<Point>()?;
  m.add_class::<Point3D>()?;
  m.add_class::<GeodesicSolution>()?;
  m.add_class::<BoundingBox>()?;
  m.add_class::<HausdorffDirectedWitness>()?;
  m.add_class::<HausdorffWitness>()?;
  m.add_function(wrap_pyfunction!(geodesic_distance, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_on_ellipsoid, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_with_bearings, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_with_bearings_on_ellipsoid, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_clipped_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_clipped_3d, m)?)?;
  Ok(())
}
