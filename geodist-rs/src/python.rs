//! PyO3 module exposing geodist-rs functionality to Python.
//!
//! PyO3 compiles this crate into a CPython extension and wires Rust
//! functions into a Python module via the `#[pymodule]` entrypoint; see
//! https://pyo3.rs/latest/ for patterns and lifecycle details.
//!
//! Keep bindings in sync: any changes here must be mirrored in
//! `pygeodist/src/geodist/_geodist_rs.pyi` in the same commit.
#![allow(unsafe_op_in_unsafe_fn)]
use geographiclib_rs::{
  Geodesic as GeographicGeodesic, InverseGeodesic, PolygonArea as GeographicPolygonArea, Winding,
};
use pyo3::buffer::{PyBuffer, ReadOnlyCell};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3::{PyErr, create_exception, wrap_pyfunction};

use crate::constants::{EARTH_RADIUS_METERS, MAX_LAT_DEGREES, MAX_LON_DEGREES, MIN_LAT_DEGREES, MIN_LON_DEGREES};
use crate::{
  chamfer as chamfer_kernel, distance, hausdorff as hausdorff_kernel, polygon as polygon_kernel, polyline, types,
};

type RingTuple = Vec<(f64, f64)>;

create_exception!(_geodist_rs, GeodistError, PyValueError);
create_exception!(_geodist_rs, InvalidLatitudeError, GeodistError);
create_exception!(_geodist_rs, InvalidLongitudeError, GeodistError);
create_exception!(_geodist_rs, InvalidAltitudeError, GeodistError);
create_exception!(_geodist_rs, InvalidDistanceError, GeodistError);
create_exception!(_geodist_rs, InvalidRadiusError, GeodistError);
create_exception!(_geodist_rs, InvalidEllipsoidError, GeodistError);
create_exception!(_geodist_rs, InvalidBoundingBoxError, GeodistError);
create_exception!(_geodist_rs, EmptyPointSetError, GeodistError);
create_exception!(_geodist_rs, InvalidPolygonError, GeodistError);
create_exception!(_geodist_rs, InvalidGeometryError, GeodistError);

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

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct PolylineDirectedWitness {
  #[pyo3(get)]
  distance_m: f64,
  #[pyo3(get)]
  source_part: usize,
  #[pyo3(get)]
  source_index: usize,
  #[pyo3(get)]
  target_part: usize,
  #[pyo3(get)]
  target_index: usize,
  #[pyo3(get)]
  source_coord: Point,
  #[pyo3(get)]
  target_coord: Point,
}

impl From<hausdorff_kernel::PolylineDirectedWitness> for PolylineDirectedWitness {
  fn from(value: hausdorff_kernel::PolylineDirectedWitness) -> Self {
    Self {
      distance_m: value.distance().meters(),
      source_part: value.source_part(),
      source_index: value.source_index(),
      target_part: value.target_part(),
      target_index: value.target_index(),
      source_coord: Point {
        lat: value.source_coord().lat,
        lon: value.source_coord().lon,
      },
      target_coord: Point {
        lat: value.target_coord().lat,
        lon: value.target_coord().lon,
      },
    }
  }
}

#[pymethods]
impl PolylineDirectedWitness {
  fn __repr__(&self) -> String {
    format!(
      "PolylineDirectedWitness(distance_m={}, source_part={}, source_index={}, target_part={}, \
       target_index={}, source_coord=Point(lat={}, lon={}), target_coord=Point(lat={}, lon={}))",
      self.distance_m,
      self.source_part,
      self.source_index,
      self.target_part,
      self.target_index,
      self.source_coord.lat,
      self.source_coord.lon,
      self.target_coord.lat,
      self.target_coord.lon
    )
  }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct PolylineHausdorffWitness {
  #[pyo3(get)]
  distance_m: f64,
  #[pyo3(get)]
  a_to_b: PolylineDirectedWitness,
  #[pyo3(get)]
  b_to_a: PolylineDirectedWitness,
}

impl From<hausdorff_kernel::PolylineHausdorffWitness> for PolylineHausdorffWitness {
  fn from(value: hausdorff_kernel::PolylineHausdorffWitness) -> Self {
    Self {
      distance_m: value.distance().meters(),
      a_to_b: value.a_to_b().into(),
      b_to_a: value.b_to_a().into(),
    }
  }
}

#[pymethods]
impl PolylineHausdorffWitness {
  fn __repr__(&self) -> String {
    let dist = self.distance_m;
    let format_leg = |leg: &PolylineDirectedWitness| {
      format!(
        "(distance_m={}, source_part={}, source_index={}, target_part={}, target_index={}, source_coord=({}, {}), target_coord=({}, {}))",
        leg.distance_m,
        leg.source_part,
        leg.source_index,
        leg.target_part,
        leg.target_index,
        leg.source_coord.lat,
        leg.source_coord.lon,
        leg.target_coord.lat,
        leg.target_coord.lon
      )
    };
    format!(
      "PolylineHausdorffWitness(distance_m={}, a_to_b={}, b_to_a={})",
      dist,
      format_leg(&self.a_to_b),
      format_leg(&self.b_to_a)
    )
  }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct ChamferDirectedResult {
  #[pyo3(get)]
  distance_m: f64,
  #[pyo3(get)]
  witness: Option<PolylineDirectedWitness>,
}

impl From<chamfer_kernel::ChamferDirectedResult> for ChamferDirectedResult {
  fn from(value: chamfer_kernel::ChamferDirectedResult) -> Self {
    Self {
      distance_m: value.distance().meters(),
      witness: value.witness().map(PolylineDirectedWitness::from),
    }
  }
}

#[pymethods]
impl ChamferDirectedResult {
  fn __repr__(&self) -> String {
    match &self.witness {
      Some(witness) => format!(
        "ChamferDirectedResult(distance_m={}, witness={})",
        self.distance_m,
        witness.__repr__()
      ),
      None => format!("ChamferDirectedResult(distance_m={}, witness=None)", self.distance_m),
    }
  }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct ChamferResult {
  #[pyo3(get)]
  distance_m: f64,
  #[pyo3(get)]
  a_to_b: ChamferDirectedResult,
  #[pyo3(get)]
  b_to_a: ChamferDirectedResult,
}

impl From<chamfer_kernel::ChamferResult> for ChamferResult {
  fn from(value: chamfer_kernel::ChamferResult) -> Self {
    Self {
      distance_m: value.distance().meters(),
      a_to_b: value.a_to_b().into(),
      b_to_a: value.b_to_a().into(),
    }
  }
}

#[pymethods]
impl ChamferResult {
  fn __repr__(&self) -> String {
    format!(
      "ChamferResult(distance_m={}, a_to_b={}, b_to_a={})",
      self.distance_m,
      self.a_to_b.__repr__(),
      self.b_to_a.__repr__()
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

/// Polygon boundary wrapper.
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Polygon {
  inner: polygon_kernel::Polygon,
}

/// LineString/Polyline wrapper.
#[pyclass(name = "LineString", frozen)]
#[derive(Debug, Clone)]
pub struct Polyline {
  vertices: Vec<types::Point>,
}

/// Options controlling polyline densification prior to Hausdorff evaluation.
#[pyclass(name = "DensificationOptions", frozen)]
#[derive(Debug, Clone)]
pub struct PyDensificationOptions {
  #[pyo3(get)]
  max_segment_length_m: Option<f64>,
  #[pyo3(get)]
  max_segment_angle_deg: Option<f64>,
  #[pyo3(get)]
  sample_cap: usize,
}

#[pymethods]
impl Polygon {
  #[new]
  pub fn new(exterior: Vec<(f64, f64)>, holes: Option<Vec<Vec<(f64, f64)>>>) -> PyResult<Self> {
    let exterior_points = map_to_points_from_tuples(&exterior)?;
    let hole_points = match holes {
      Some(rings) => {
        let mut out = Vec::with_capacity(rings.len());
        for ring in rings {
          out.push(map_to_points_from_tuples(&ring)?);
        }
        out
      }
      None => Vec::new(),
    };

    let polygon = map_geodist_result(polygon_kernel::Polygon::new(exterior_points, hole_points))?;
    Ok(Self { inner: polygon })
  }

  fn __repr__(&self) -> String {
    "Polygon(...)".to_string()
  }

  /// Return `(exterior, holes)` where rings are lists of `(lat, lon)` tuples.
  pub fn to_tuple(&self) -> (RingTuple, Vec<RingTuple>) {
    let exterior = self.inner.exterior.iter().map(|p| (p.lat, p.lon)).collect();
    let holes = self
      .inner
      .holes
      .iter()
      .map(|ring| ring.iter().map(|p| (p.lat, p.lon)).collect())
      .collect();
    (exterior, holes)
  }
}

#[pymethods]
impl Polyline {
  #[new]
  pub fn new(vertices: Vec<(f64, f64)>) -> PyResult<Self> {
    let points = map_to_points_from_tuples(&vertices)?;
    let deduped = map_geodist_result(polyline::validate_polyline(&points, None))?;
    Ok(Self { vertices: deduped })
  }

  /// Return the vertices as `(lat, lon)` tuples.
  pub fn to_tuple(&self) -> Vec<(f64, f64)> {
    self.vertices.iter().map(|vertex| (vertex.lat, vertex.lon)).collect()
  }

  /// Densify the LineString into samples honoring spacing knobs and caps.
  #[pyo3(signature = (max_segment_length_m = Some(100.0), max_segment_angle_deg = Some(0.1), sample_cap = 50_000))]
  pub fn densify(
    &self,
    max_segment_length_m: Option<f64>,
    max_segment_angle_deg: Option<f64>,
    sample_cap: usize,
  ) -> PyResult<Vec<Point>> {
    let options = polyline::DensificationOptions {
      max_segment_length_m,
      max_segment_angle_deg,
      sample_cap,
    };
    let samples = map_geodist_result(polyline::densify_polyline(&self.vertices, options))?;

    Ok(
      samples
        .into_iter()
        .map(|vertex| Point {
          lat: vertex.lat,
          lon: vertex.lon,
        })
        .collect(),
    )
  }

  fn __repr__(&self) -> String {
    format!("LineString(num_vertices={})", self.vertices.len())
  }

  const fn __len__(&self) -> PyResult<usize> {
    Ok(self.vertices.len())
  }
}

#[pymethods]
impl PyDensificationOptions {
  #[new]
  #[pyo3(signature = (max_segment_length_m = Some(100.0), max_segment_angle_deg = Some(0.1), sample_cap = 50_000))]
  pub fn new(
    max_segment_length_m: Option<f64>,
    max_segment_angle_deg: Option<f64>,
    sample_cap: usize,
  ) -> PyResult<Self> {
    map_densification_options_parts(max_segment_length_m, max_segment_angle_deg, sample_cap)?;

    Ok(Self {
      max_segment_length_m,
      max_segment_angle_deg,
      sample_cap,
    })
  }

  /// Return a tuple `(max_segment_length_m, max_segment_angle_deg,
  /// sample_cap)`.
  pub const fn to_tuple(&self) -> (Option<f64>, Option<f64>, usize) {
    (self.max_segment_length_m, self.max_segment_angle_deg, self.sample_cap)
  }

  fn __repr__(&self) -> String {
    format!(
      "DensificationOptions(max_segment_length_m={}, max_segment_angle_deg={}, sample_cap={})",
      self
        .max_segment_length_m
        .map_or_else(|| "None".to_string(), |value| value.to_string()),
      self
        .max_segment_angle_deg
        .map_or_else(|| "None".to_string(), |value| value.to_string()),
      self.sample_cap
    )
  }
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

#[allow(unreachable_patterns)]
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
    types::GeodistError::InvalidRingOrientation { .. } => InvalidGeometryError::new_err(message),
    types::GeodistError::EmptyPointSet => EmptyPointSetError::new_err(message),
    types::GeodistError::MissingDensificationKnob
    | types::GeodistError::DegeneratePolyline { .. }
    | types::GeodistError::InvalidVertex { .. }
    | types::GeodistError::SampleCapExceeded { .. } => InvalidGeometryError::new_err(message),
    _ => GeodistError::new_err(message),
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

fn map_to_multiline(handles: &[Polyline]) -> Vec<Vec<types::Point>> {
  handles.iter().map(|line| line.vertices.clone()).collect()
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

fn map_to_points_from_tuples(coords: &[(f64, f64)]) -> PyResult<Vec<types::Point>> {
  coords
    .iter()
    .map(|(lat, lon)| map_geodist_result(types::Point::new(*lat, *lon)))
    .collect()
}

fn map_densification_options_parts(
  max_segment_length_m: Option<f64>,
  max_segment_angle_deg: Option<f64>,
  sample_cap: usize,
) -> PyResult<polyline::DensificationOptions> {
  if max_segment_length_m.is_none() && max_segment_angle_deg.is_none() {
    return Err(InvalidDistanceError::new_err(
      "polyline densification requires max_segment_length_m or max_segment_angle_deg",
    ));
  }

  Ok(polyline::DensificationOptions {
    max_segment_length_m,
    max_segment_angle_deg,
    sample_cap,
  })
}

fn map_densification_options(handle: Option<&PyDensificationOptions>) -> PyResult<polyline::DensificationOptions> {
  if let Some(options) = handle {
    return map_densification_options_parts(
      options.max_segment_length_m,
      options.max_segment_angle_deg,
      options.sample_cap,
    );
  }

  Ok(polyline::DensificationOptions::default())
}

fn map_boundary_densification_opts(
  max_segment_length_m: Option<f64>,
  max_segment_angle_deg: Option<f64>,
  sample_cap: usize,
) -> PyResult<polyline::DensificationOptions> {
  map_densification_options_parts(max_segment_length_m, max_segment_angle_deg, sample_cap)
}

fn map_chamfer_reduction(value: &str) -> PyResult<chamfer_kernel::ChamferReduction> {
  match value {
    "mean" => Ok(chamfer_kernel::ChamferReduction::Mean),
    "sum" => Ok(chamfer_kernel::ChamferReduction::Sum),
    "max" => Ok(chamfer_kernel::ChamferReduction::Max),
    other => Err(PyValueError::new_err(format!(
      "invalid reduction \"{other}\"; expected \"mean\", \"sum\", or \"max\""
    ))),
  }
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
#[pyo3(signature = (a, b, options = None))]
fn hausdorff_directed_polyline(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  options: Option<&PyDensificationOptions>,
) -> PyResult<PolylineDirectedWitness> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let densification_options = map_densification_options(options)?;

  hausdorff_kernel::hausdorff_directed_polyline(&parts_a, &parts_b, densification_options)
    .map(PolylineDirectedWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, options = None))]
fn hausdorff_polyline(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  options: Option<&PyDensificationOptions>,
) -> PyResult<PolylineHausdorffWitness> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let densification_options = map_densification_options(options)?;

  hausdorff_kernel::hausdorff_polyline(&parts_a, &parts_b, densification_options)
    .map(PolylineHausdorffWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, reduction = "mean", options = None))]
fn chamfer_directed_polyline(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  reduction: &str,
  options: Option<&PyDensificationOptions>,
) -> PyResult<ChamferDirectedResult> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let densification_options = map_densification_options(options)?;
  let reduction = map_chamfer_reduction(reduction)?;

  chamfer_kernel::chamfer_directed_polyline(&parts_a, &parts_b, densification_options, reduction)
    .map(ChamferDirectedResult::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, reduction = "mean", options = None))]
fn chamfer_polyline(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  reduction: &str,
  options: Option<&PyDensificationOptions>,
) -> PyResult<ChamferResult> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let densification_options = map_densification_options(options)?;
  let reduction = map_chamfer_reduction(reduction)?;

  chamfer_kernel::chamfer_polyline(&parts_a, &parts_b, densification_options, reduction)
    .map(ChamferResult::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, bounding_box, options = None))]
fn hausdorff_directed_polyline_clipped(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  bounding_box: &BoundingBox,
  options: Option<&PyDensificationOptions>,
) -> PyResult<PolylineDirectedWitness> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let bbox = map_to_bounding_box(bounding_box)?;
  let densification_options = map_densification_options(options)?;

  hausdorff_kernel::hausdorff_directed_polyline_clipped(&parts_a, &parts_b, densification_options, bbox)
    .map(PolylineDirectedWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, bounding_box, options = None))]
fn hausdorff_polyline_clipped(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  bounding_box: &BoundingBox,
  options: Option<&PyDensificationOptions>,
) -> PyResult<PolylineHausdorffWitness> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let bbox = map_to_bounding_box(bounding_box)?;
  let densification_options = map_densification_options(options)?;

  hausdorff_kernel::hausdorff_polyline_clipped(&parts_a, &parts_b, densification_options, bbox)
    .map(PolylineHausdorffWitness::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, bounding_box, reduction = "mean", options = None))]
fn chamfer_directed_polyline_clipped(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  bounding_box: &BoundingBox,
  reduction: &str,
  options: Option<&PyDensificationOptions>,
) -> PyResult<ChamferDirectedResult> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let bbox = map_to_bounding_box(bounding_box)?;
  let densification_options = map_densification_options(options)?;
  let reduction = map_chamfer_reduction(reduction)?;

  chamfer_kernel::chamfer_directed_polyline_clipped(&parts_a, &parts_b, densification_options, reduction, bbox)
    .map(ChamferDirectedResult::from)
    .map_err(map_geodist_error)
}

#[pyfunction]
#[pyo3(signature = (a, b, bounding_box, reduction = "mean", options = None))]
fn chamfer_polyline_clipped(
  a: Vec<Polyline>,
  b: Vec<Polyline>,
  bounding_box: &BoundingBox,
  reduction: &str,
  options: Option<&PyDensificationOptions>,
) -> PyResult<ChamferResult> {
  let parts_a = map_to_multiline(&a);
  let parts_b = map_to_multiline(&b);
  let bbox = map_to_bounding_box(bounding_box)?;
  let densification_options = map_densification_options(options)?;
  let reduction = map_chamfer_reduction(reduction)?;

  chamfer_kernel::chamfer_polyline_clipped(&parts_a, &parts_b, densification_options, reduction, bbox)
    .map(ChamferResult::from)
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

#[pyfunction]
fn hausdorff_polygon_boundary(
  a: &Polygon,
  b: &Polygon,
  max_segment_length_m: Option<f64>,
  max_segment_angle_deg: Option<f64>,
  sample_cap: usize,
) -> PyResult<f64> {
  let options = map_boundary_densification_opts(max_segment_length_m, max_segment_angle_deg, sample_cap)?;
  polygon_kernel::hausdorff_boundary(&a.inner, &b.inner, options)
    .map(|witness| witness.distance().meters())
    .map_err(map_geodist_error)
}

#[derive(Debug, Clone)]
struct LatLonBuffers {
  lat: Vec<f64>,
  lon: Vec<f64>,
}

fn extract_float_buffer<'py, T>(py: Python<'py>, obj: &Bound<'py, PyAny>, name: &str) -> PyResult<Option<Vec<f64>>>
where
  T: pyo3::buffer::Element + Copy + Into<f64>,
{
  if let Ok(buffer) = PyBuffer::<T>::get(obj) {
    if buffer.dimensions() > 1 || !buffer.is_c_contiguous() {
      return Err(PyValueError::new_err(format!(
        "{name} must be a contiguous 1-D buffer, got {} dimensions",
        buffer.dimensions()
      )));
    }

    if let Some(slice) = buffer.as_slice(py) {
      let values: Vec<f64> = slice.iter().map(ReadOnlyCell::get).map(Into::into).collect();
      return Ok(Some(values));
    }

    return Err(PyValueError::new_err(format!("{name} must expose a readable buffer")));
  }

  Ok(None)
}

/// Extract a vector of f64 values from a Python object, attempting to read
/// from a buffer first, then falling back to sequence extraction.
///
/// Arguments:
/// - `py`: Python GIL token.
/// - `obj`: Python object to extract from.
/// - `name`: Name for error reporting.
///
/// Returns:
/// - `Ok(Vec<f64>)` if extraction is successful.
/// - `Err(PyErr)` if extraction fails.
fn extract_f64_vector(py: Python<'_>, obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<f64>> {
  if let Some(values) = extract_float_buffer::<f64>(py, obj, name)? {
    return Ok(values);
  }

  if let Some(values) = extract_float_buffer::<f32>(py, obj, name)? {
    return Ok(values);
  }

  obj.extract::<Vec<f64>>()
}

/// Validate latitude/longitude ranges for a single point.
///
/// Latitude must be in [-90.0, 90.0] and longitude in [-180.0, 180.0]. These
/// invariants are enforced to ensure that geographic computations behave
/// correctly.
///
/// Arguments:
/// - `lat`: Latitude in degrees.
/// - `lon`: Longitude in degrees.
/// - `index`: Index of the point in the input array for error reporting.
///
/// Returns:
/// - `Ok(())` if the latitude and longitude are valid.
/// - `Err(PyErr)` if the latitude or longitude are invalid.
fn validate_lat_lon(lat: f64, lon: f64, index: usize) -> PyResult<()> {
  if !lat.is_finite() {
    return Err(InvalidGeometryError::new_err(format!(
      "index {index}: latitude must be finite, got {lat}"
    )));
  }

  if !(MIN_LAT_DEGREES..=MAX_LAT_DEGREES).contains(&lat) {
    return Err(InvalidGeometryError::new_err(format!(
      "index {index}: latitude {lat} outside [{MIN_LAT_DEGREES}, {MAX_LAT_DEGREES}]"
    )));
  }

  if !lon.is_finite() {
    return Err(InvalidGeometryError::new_err(format!(
      "index {index}: longitude must be finite, got {lon}"
    )));
  }

  if !(MIN_LON_DEGREES..=MAX_LON_DEGREES).contains(&lon) {
    return Err(InvalidGeometryError::new_err(format!(
      "index {index}: longitude {lon} outside [{MIN_LON_DEGREES}, {MAX_LON_DEGREES}]"
    )));
  }

  Ok(())
}

/// Given latitude and longitude objects from Python, extract their values
/// into vectors and validate them.
///
/// Arguments:
/// - `py`: Python GIL token.
/// - `lat_obj`: Python object representing latitudes.
/// - `lon_obj`: Python object representing longitudes.
/// - `name`: Base name for error reporting.
///
/// Returns:
/// - `Ok(LatLonBuffers)` containing the extracted latitude and longitude
///   vectors if successful.
/// - `Err(PyErr)` if extraction or validation fails.
fn load_lat_lon_buffers(
  py: Python<'_>,
  lat_obj: &Bound<'_, PyAny>,
  lon_obj: &Bound<'_, PyAny>,
  name: &str,
) -> PyResult<LatLonBuffers> {
  let lat = extract_f64_vector(py, lat_obj, &format!("{name}.lat"))?;
  let lon = extract_f64_vector(py, lon_obj, &format!("{name}.lon"))?;

  if lat.len() != lon.len() {
    return Err(PyValueError::new_err(format!(
      "{name}.lat and {name}.lon must have equal length, got {} and {}",
      lat.len(),
      lon.len()
    )));
  }

  for (index, (&lat_value, &lon_value)) in lat.iter().zip(&lon).enumerate() {
    validate_lat_lon(lat_value, lon_value, index)?;
  }

  Ok(LatLonBuffers { lat, lon })
}

fn ellipsoid_axes(ellipsoid: Option<&Ellipsoid>) -> PyResult<Option<(f64, f64)>> {
  if let Some(model) = ellipsoid {
    let out = map_to_ellipsoid(model)?;
    return Ok(Some((out.semi_major_axis_m, out.semi_minor_axis_m)));
  }

  Ok(None)
}

const fn default_ellipsoid_axes() -> (f64, f64) {
  let default = types::Ellipsoid::wgs84();
  (default.semi_major_axis_m, default.semi_minor_axis_m)
}

#[pyfunction]
fn geodesic_distance_batch(
  py: Python<'_>,
  origins_lat: &Bound<'_, PyAny>,
  origins_lon: &Bound<'_, PyAny>,
  destinations_lat: &Bound<'_, PyAny>,
  destinations_lon: &Bound<'_, PyAny>,
  ellipsoid: Option<&Ellipsoid>,
) -> PyResult<Vec<f64>> {
  let origins = load_lat_lon_buffers(py, origins_lat, origins_lon, "origins")?;
  let destinations = load_lat_lon_buffers(py, destinations_lat, destinations_lon, "destinations")?;

  if origins.lat.len() != destinations.lat.len() {
    return Err(PyValueError::new_err(format!(
      "origins and destinations must share length, got {} and {}",
      origins.lat.len(),
      destinations.lat.len()
    )));
  }

  let count = origins.lat.len();
  if count == 0 {
    return Ok(Vec::new());
  }

  let ellipsoid_axes = ellipsoid_axes(ellipsoid)?;

  #[allow(deprecated)]
  let distances = py.allow_threads(|| -> PyResult<Vec<f64>> {
    match ellipsoid_axes {
      Some((semi_major, semi_minor)) => {
        let flattening = 1.0 - (semi_minor / semi_major);
        let geodesic = GeographicGeodesic::new(semi_major, flattening);
        let mut out = Vec::with_capacity(count);
        for idx in 0..count {
          let meters = geodesic.inverse(
            origins.lat[idx],
            origins.lon[idx],
            destinations.lat[idx],
            destinations.lon[idx],
          );
          out.push(meters);
        }
        Ok(out)
      }
      None => {
        let mut out = Vec::with_capacity(count);
        for idx in 0..count {
          let (meters, _, _) = distance::spherical_distance_and_bearings(
            origins.lat[idx],
            origins.lon[idx],
            destinations.lat[idx],
            destinations.lon[idx],
          );
          out.push(meters);
        }
        Ok(out)
      }
    }
  })?;

  Ok(distances)
}

#[pyfunction]
fn geodesic_with_bearings_batch(
  py: Python<'_>,
  origins_lat: &Bound<'_, PyAny>,
  origins_lon: &Bound<'_, PyAny>,
  destinations_lat: &Bound<'_, PyAny>,
  destinations_lon: &Bound<'_, PyAny>,
  ellipsoid: Option<&Ellipsoid>,
) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
  let origins = load_lat_lon_buffers(py, origins_lat, origins_lon, "origins")?;
  let destinations = load_lat_lon_buffers(py, destinations_lat, destinations_lon, "destinations")?;

  if origins.lat.len() != destinations.lat.len() {
    return Err(PyValueError::new_err(format!(
      "origins and destinations must share length, got {} and {}",
      origins.lat.len(),
      destinations.lat.len()
    )));
  }

  let count = origins.lat.len();
  if count == 0 {
    return Ok((Vec::new(), Vec::new(), Vec::new()));
  }

  let ellipsoid_axes = ellipsoid_axes(ellipsoid)?;

  #[allow(deprecated)]
  let (distances, initials, finals) = py.allow_threads(|| -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
    match ellipsoid_axes {
      Some((semi_major, semi_minor)) => {
        let flattening = 1.0 - (semi_minor / semi_major);
        let geodesic = GeographicGeodesic::new(semi_major, flattening);

        let mut distances = Vec::with_capacity(count);
        let mut initials = Vec::with_capacity(count);
        let mut finals = Vec::with_capacity(count);

        for idx in 0..count {
          let (meters, initial, final_bearing, _) = geodesic.inverse(
            origins.lat[idx],
            origins.lon[idx],
            destinations.lat[idx],
            destinations.lon[idx],
          );
          distances.push(meters);
          initials.push(distance::normalize_bearing(initial));
          finals.push(distance::normalize_bearing(final_bearing));
        }

        Ok((distances, initials, finals))
      }
      None => {
        let mut distances = Vec::with_capacity(count);
        let mut initials = Vec::with_capacity(count);
        let mut finals = Vec::with_capacity(count);

        for idx in 0..count {
          let (meters, initial, final_bearing) = distance::spherical_distance_and_bearings(
            origins.lat[idx],
            origins.lon[idx],
            destinations.lat[idx],
            destinations.lon[idx],
          );
          distances.push(meters);
          initials.push(initial);
          finals.push(final_bearing);
        }

        Ok((distances, initials, finals))
      }
    }
  })?;

  Ok((distances, initials, finals))
}

#[pyfunction]
fn geodesic_distance_to_many(
  py: Python<'_>,
  origin_lat: f64,
  origin_lon: f64,
  destinations_lat: &Bound<'_, PyAny>,
  destinations_lon: &Bound<'_, PyAny>,
  ellipsoid: Option<&Ellipsoid>,
) -> PyResult<Vec<f64>> {
  validate_lat_lon(origin_lat, origin_lon, 0)?;
  let destinations = load_lat_lon_buffers(py, destinations_lat, destinations_lon, "destinations")?;
  let count = destinations.lat.len();

  if count == 0 {
    return Ok(Vec::new());
  }

  let ellipsoid_axes = ellipsoid_axes(ellipsoid)?;
  let distances = py.detach(|| -> PyResult<Vec<f64>> {
    match ellipsoid_axes {
      Some((semi_major, semi_minor)) => {
        let flattening = 1.0 - (semi_minor / semi_major);
        let geodesic = GeographicGeodesic::new(semi_major, flattening);
        let mut out = Vec::with_capacity(count);

        for idx in 0..count {
          let meters = geodesic.inverse(origin_lat, origin_lon, destinations.lat[idx], destinations.lon[idx]);
          out.push(meters);
        }

        Ok(out)
      }
      None => {
        let mut out = Vec::with_capacity(count);
        for idx in 0..count {
          let (meters, _, _) = distance::spherical_distance_and_bearings(
            origin_lat,
            origin_lon,
            destinations.lat[idx],
            destinations.lon[idx],
          );
          out.push(meters);
        }
        Ok(out)
      }
    }
  })?;

  Ok(distances)
}

/// Extract a monotonic offset vector from Python buffers or sequences.
///
/// Supports `usize`/`i64` contiguous buffers to avoid copies, falling back
/// to sequence extraction. Negative values are rejected for signed buffers.
fn extract_offsets(py: Python<'_>, obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<usize>> {
  if let Ok(buffer) = PyBuffer::<usize>::get(obj) {
    if buffer.dimensions() > 1 || !buffer.is_c_contiguous() {
      return Err(PyValueError::new_err(format!(
        "{name} must be a contiguous 1-D integer buffer, got {} dimensions",
        buffer.dimensions()
      )));
    }

    if let Some(slice) = buffer.as_slice(py) {
      return Ok(slice.iter().map(ReadOnlyCell::get).collect());
    }

    return Err(PyValueError::new_err(format!("{name} must expose a readable buffer")));
  }

  if let Ok(buffer) = PyBuffer::<i64>::get(obj) {
    if buffer.dimensions() > 1 || !buffer.is_c_contiguous() {
      return Err(PyValueError::new_err(format!(
        "{name} must be a contiguous 1-D integer buffer, got {} dimensions",
        buffer.dimensions()
      )));
    }

    if let Some(slice) = buffer.as_slice(py) {
      let mut out = Vec::with_capacity(slice.len());
      for value in slice {
        let value = value.get();
        if value < 0 {
          return Err(InvalidGeometryError::new_err(format!(
            "{name} must be non-negative, got {value}"
          )));
        }
        out.push(value as usize);
      }
      return Ok(out);
    }

    return Err(PyValueError::new_err(format!("{name} must expose a readable buffer")));
  }

  obj.extract::<Vec<usize>>()
}

/// Validate that offsets are non-empty, start at zero, monotonic, and end
/// at the expected sentinel value. Returns `InvalidGeometryError` on
/// malformed inputs to mirror Python-facing error types.
fn validate_offsets(offsets: &[usize], name: &str, expected_final: usize) -> PyResult<()> {
  if offsets.is_empty() {
    return Err(InvalidGeometryError::new_err(format!(
      "{name} must contain at least one offset"
    )));
  }

  if offsets[0] != 0 {
    return Err(InvalidGeometryError::new_err(format!(
      "{name} must start at 0, got {}",
      offsets[0]
    )));
  }

  for window in offsets.windows(2) {
    if window[1] < window[0] {
      return Err(InvalidGeometryError::new_err(format!(
        "{name} must be monotonically increasing"
      )));
    }
  }

  if let Some(last) = offsets.last()
    && *last != expected_final
  {
    return Err(InvalidGeometryError::new_err(format!(
      "{name} must end at {expected_final}, got {last}"
    )));
  }

  Ok(())
}

/// Extract a `Vec<(lat, lon)>` from a Python buffer or sequence.
///
/// Accepts any C-contiguous float buffer (f64 or f32) with at least two
/// dimensions where the trailing dimension has length 2 or greater, or falls
/// back to `extract`ing a list of tuples. Raises a `ValueError` when the
/// buffer is not readable/contiguous or when the shape is incompatible.
fn extract_ring_coords(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Vec<(f64, f64)>> {
  if let Ok(buffer) = PyBuffer::<f64>::get(obj) {
    if !buffer.is_c_contiguous() {
      return Err(PyValueError::new_err("coords must be contiguous"));
    }

    let dimensions = buffer.dimensions();
    if dimensions < 2 {
      return Err(PyValueError::new_err(
        "coords must be at least 2-D with trailing dimension 2 or 3",
      ));
    }
    let shape = buffer.shape();
    let last_dim = *shape.last().unwrap_or(&0);

    if last_dim < 2 {
      return Err(PyValueError::new_err(
        "coords trailing dimension must be at least length 2",
      ));
    }

    let rows = buffer.item_count() / last_dim;
    let slice = buffer
      .as_slice(py)
      .ok_or_else(|| PyValueError::new_err("coords must be a readable, contiguous buffer"))?;

    let mut coords = Vec::with_capacity(rows);
    for row in 0..rows {
      let base = row * last_dim;
      let lat = slice[base].get();
      let lon = slice[base + 1].get();
      coords.push((lat, lon));
    }
    return Ok(coords);
  }

  if let Ok(buffer) = PyBuffer::<f32>::get(obj) {
    if !buffer.is_c_contiguous() {
      return Err(PyValueError::new_err("coords must be contiguous"));
    }

    let dimensions = buffer.dimensions();
    if dimensions < 2 {
      return Err(PyValueError::new_err(
        "coords must be at least 2-D with trailing dimension 2 or 3",
      ));
    }
    let shape = buffer.shape();
    let last_dim = *shape.last().unwrap_or(&0);

    if last_dim < 2 {
      return Err(PyValueError::new_err(
        "coords trailing dimension must be at least length 2",
      ));
    }

    let rows = buffer.item_count() / last_dim;
    let slice = buffer
      .as_slice(py)
      .ok_or_else(|| PyValueError::new_err("coords must be a readable, contiguous buffer"))?;

    let mut coords = Vec::with_capacity(rows);
    for row in 0..rows {
      let base = row * last_dim;
      let lat = slice[base].get() as f64;
      let lon = slice[base + 1].get() as f64;
      coords.push((lat, lon));
    }
    return Ok(coords);
  }

  obj.extract::<Vec<(f64, f64)>>()
}

/// Compute area of a ring slice using a GeographicLib accumulator.
///
/// Arguments define a half-open interval within `coords` that represent a
/// single ring, along with indices used for error reporting. Returns the
/// absolute value of the signed area. Errors when offsets are invalid, rings
/// are too short, or any vertex fails latitude/longitude validation.
fn compute_ring_area(
  geodesic: &GeographicGeodesic,
  coords: &[(f64, f64)],
  start: usize,
  end: usize,
  polygon_index: usize,
  ring_index: usize,
) -> PyResult<f64> {
  if end < start || end > coords.len() {
    return Err(InvalidGeometryError::new_err(format!(
      "polygon {polygon_index} ring {ring_index} has invalid offsets {start}..{end}"
    )));
  }

  let count = end - start;
  if count < 3 {
    return Err(InvalidGeometryError::new_err(format!(
      "polygon {polygon_index} ring {ring_index} must include at least 3 vertices"
    )));
  }

  let mut area = GeographicPolygonArea::new(geodesic, Winding::CounterClockwise);
  for (vertex_index, (lat, lon)) in coords[start..end].iter().copied().enumerate() {
    validate_lat_lon(lat, lon, start + vertex_index)?;
    area.add_point(lat, lon);
  }

  let (_perimeter, ring_area, _num) = area.compute(false);
  Ok(ring_area.abs())
}

#[pyfunction]
fn polygon_area_batch(
  py: Python<'_>,
  coords: &Bound<'_, PyAny>,
  ring_offsets: &Bound<'_, PyAny>,
  polygon_offsets: &Bound<'_, PyAny>,
  ellipsoid: Option<&Ellipsoid>,
) -> PyResult<Vec<f64>> {
  let coords = extract_ring_coords(py, coords)?;
  let ring_offsets = extract_offsets(py, ring_offsets, "ring_offsets")?;
  let polygon_offsets = extract_offsets(py, polygon_offsets, "polygon_offsets")?;

  validate_offsets(&ring_offsets, "ring_offsets", coords.len())?;
  validate_offsets(
    &polygon_offsets,
    "polygon_offsets",
    ring_offsets.len().saturating_sub(1),
  )?;

  let (semi_major, semi_minor) = ellipsoid_axes(ellipsoid)?.unwrap_or_else(default_ellipsoid_axes);
  let flattening = 1.0 - (semi_minor / semi_major);
  let geodesic = GeographicGeodesic::new(semi_major, flattening);

  let mut areas = Vec::with_capacity(polygon_offsets.len().saturating_sub(1));

  for (polygon_index, window) in polygon_offsets.windows(2).enumerate() {
    let ring_start = window[0];
    let ring_end = window[1];

    if ring_end < ring_start || ring_end > ring_offsets.len().saturating_sub(1) {
      return Err(InvalidGeometryError::new_err(format!(
        "polygon {polygon_index} has invalid ring offsets {ring_start}..{ring_end}"
      )));
    }

    if ring_start == ring_end {
      areas.push(0.0);
      continue;
    }

    let exterior_area = compute_ring_area(
      &geodesic,
      &coords,
      ring_offsets[ring_start],
      ring_offsets[ring_start + 1],
      polygon_index,
      ring_start,
    )?;

    let mut holes_area = 0.0;
    for ring_index in (ring_start + 1)..ring_end {
      let ring_area = compute_ring_area(
        &geodesic,
        &coords,
        ring_offsets[ring_index],
        ring_offsets[ring_index + 1],
        polygon_index,
        ring_index,
      )?;
      holes_area += ring_area;
    }

    let mut net = exterior_area - holes_area;
    if net < 0.0 {
      net = 0.0;
    }
    areas.push(net);
  }

  Ok(areas)
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
  m.add("InvalidPolygonError", py.get_type::<InvalidPolygonError>())?;
  m.add("InvalidGeometryError", py.get_type::<InvalidGeometryError>())?;
  m.add_class::<Ellipsoid>()?;
  m.add_class::<Point>()?;
  m.add_class::<Point3D>()?;
  m.add_class::<Polygon>()?;
  m.add_class::<Polyline>()?;
  m.add_class::<PyDensificationOptions>()?;
  m.add_class::<GeodesicSolution>()?;
  m.add_class::<BoundingBox>()?;
  m.add_class::<HausdorffDirectedWitness>()?;
  m.add_class::<HausdorffWitness>()?;
  m.add_class::<PolylineDirectedWitness>()?;
  m.add_class::<PolylineHausdorffWitness>()?;
  m.add_class::<ChamferDirectedResult>()?;
  m.add_class::<ChamferResult>()?;
  m.add_function(wrap_pyfunction!(geodesic_distance, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_on_ellipsoid, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_with_bearings, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_with_bearings_on_ellipsoid, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_polyline, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_polyline, m)?)?;
  m.add_function(wrap_pyfunction!(chamfer_directed_polyline, m)?)?;
  m.add_function(wrap_pyfunction!(chamfer_polyline, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_clipped_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_clipped_3d, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_directed_polyline_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_polyline_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(chamfer_directed_polyline_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(chamfer_polyline_clipped, m)?)?;
  m.add_function(wrap_pyfunction!(hausdorff_polygon_boundary, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_batch, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_with_bearings_batch, m)?)?;
  m.add_function(wrap_pyfunction!(geodesic_distance_to_many, m)?)?;
  m.add_function(wrap_pyfunction!(polygon_area_batch, m)?)?;
  Ok(())
}
