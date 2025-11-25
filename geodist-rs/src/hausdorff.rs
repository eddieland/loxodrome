//! Hausdorff distance between point sets using the geodesic kernel.
//!
//! Inputs are degrees; output is meters.

use rstar::{AABB, PointDistance, RTree, RTreeObject};

use crate::algorithms::{GeodesicAlgorithm, Spherical};
use crate::distance::{EcefPoint, geodetic_to_ecef};
use crate::polyline::{DensificationOptions, FlattenedPolyline, densify_multiline};
use crate::{BoundingBox, Distance, Ellipsoid, GeodistError, Point, Point3D};

// Keep the O(n*m) fallback for small collections where index build overhead
// outweighs nearest-neighbor savings.
const MIN_INDEX_CANDIDATE_SIZE: usize = 32;
const MAX_NAIVE_CROSS_PRODUCT: usize = 4_000;

/// Directed Hausdorff result including the realizing pair of points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HausdorffDirectedWitness {
  distance: Distance,
  origin_index: usize,
  candidate_index: usize,
}

impl HausdorffDirectedWitness {
  /// Directed Hausdorff distance in meters.
  pub const fn distance(&self) -> Distance {
    self.distance
  }

  /// Index of the origin point in the source iterable.
  ///
  /// The index refers to the caller-supplied order prior to any clipping.
  pub const fn origin_index(&self) -> usize {
    self.origin_index
  }

  /// Index of the nearest neighbor in the candidate iterable.
  ///
  /// The index refers to the caller-supplied order prior to any clipping.
  pub const fn candidate_index(&self) -> usize {
    self.candidate_index
  }

  fn from_raw(raw: DirectedHausdorffMeters) -> Result<Self, GeodistError> {
    let distance = Distance::from_meters(raw.meters)?;
    Ok(Self {
      distance,
      origin_index: raw.origin_index,
      candidate_index: raw.candidate_index,
    })
  }
}

/// Symmetric Hausdorff result containing both directed witnesses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HausdorffWitness {
  distance: Distance,
  a_to_b: HausdorffDirectedWitness,
  b_to_a: HausdorffDirectedWitness,
}

impl HausdorffWitness {
  /// Maximum distance across both directed evaluations in meters.
  pub const fn distance(&self) -> Distance {
    self.distance
  }

  /// Directed witness from the first argument to the second.
  pub const fn a_to_b(&self) -> HausdorffDirectedWitness {
    self.a_to_b
  }

  /// Directed witness from the second argument back to the first.
  pub const fn b_to_a(&self) -> HausdorffDirectedWitness {
    self.b_to_a
  }

  fn new(a_to_b: HausdorffDirectedWitness, b_to_a: HausdorffDirectedWitness) -> Result<Self, GeodistError> {
    let meters = a_to_b.distance().meters().max(b_to_a.distance().meters());
    let distance = Distance::from_meters(meters)?;

    Ok(Self {
      distance,
      a_to_b,
      b_to_a,
    })
  }
}

/// Directed Hausdorff witness across densified polylines.
///
/// Part/index fields reference the densified sample order within the supplied
/// MultiLineString inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolylineDirectedWitness {
  distance: Distance,
  source_part: usize,
  source_index: usize,
  target_part: usize,
  target_index: usize,
  source_coord: Point,
  target_coord: Point,
}

impl PolylineDirectedWitness {
  /// Directed Hausdorff distance in meters.
  pub const fn distance(&self) -> Distance {
    self.distance
  }

  /// Zero-based part index for the origin polyline.
  pub const fn source_part(&self) -> usize {
    self.source_part
  }

  /// Sample index within the origin part after densification.
  pub const fn source_index(&self) -> usize {
    self.source_index
  }

  /// Zero-based part index for the candidate polyline.
  pub const fn target_part(&self) -> usize {
    self.target_part
  }

  /// Sample index within the candidate part after densification.
  pub const fn target_index(&self) -> usize {
    self.target_index
  }

  /// Coordinate of the realizing origin sample.
  pub const fn source_coord(&self) -> Point {
    self.source_coord
  }

  /// Coordinate of the realizing candidate sample.
  pub const fn target_coord(&self) -> Point {
    self.target_coord
  }
}

/// Symmetric Hausdorff witness over densified polylines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolylineHausdorffWitness {
  distance: Distance,
  a_to_b: PolylineDirectedWitness,
  b_to_a: PolylineDirectedWitness,
}

impl PolylineHausdorffWitness {
  /// Maximum distance across both directed evaluations in meters.
  pub const fn distance(&self) -> Distance {
    self.distance
  }

  /// Directed witness from the first argument to the second.
  pub const fn a_to_b(&self) -> PolylineDirectedWitness {
    self.a_to_b
  }

  /// Directed witness from the second argument back to the first.
  pub const fn b_to_a(&self) -> PolylineDirectedWitness {
    self.b_to_a
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HausdorffStrategy {
  Naive,
  Indexed,
}

#[derive(Clone, Copy)]
struct Positioned<T> {
  point: T,
  index: usize,
}

#[derive(Clone, Copy)]
struct DirectedHausdorffMeters {
  meters: f64,
  origin_index: usize,
  candidate_index: usize,
}

/// Directed Hausdorff distance from set `a` to set `b` using the default
/// spherical geodesic, returning the realizing witness pair.
///
/// Inputs are in degrees. Returns the maximum, over all points in `a`, of the
/// minimum distance to any point in `b` alongside the origin/candidate indices.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when either slice is empty, or any
/// validation error surfaced by [`Point::validate`].
pub fn hausdorff_directed(a: &[Point], b: &[Point]) -> Result<HausdorffDirectedWitness, GeodistError> {
  hausdorff_directed_with(&Spherical::default(), a, b)
}

/// Directed Hausdorff distance using a custom geodesic algorithm.
///
/// Chooses between a naive O(n*m) search for small inputs and an R-tree based
/// nearest-neighbor lookup for larger sets to bound runtime while keeping
/// allocations modest.
///
/// # Errors
/// Propagates the same validation and empty-set errors as
/// [`hausdorff_directed`].
pub fn hausdorff_directed_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
) -> Result<HausdorffDirectedWitness, GeodistError> {
  ensure_non_empty(a)?;
  ensure_non_empty(b)?;
  validate_points(a)?;
  validate_points(b)?;

  let origins = position_points(a);
  let candidates = position_points(b);
  hausdorff_directed_positioned_with(algorithm, &origins, &candidates)
}

/// Symmetric Hausdorff distance between sets `a` and `b`.
///
/// Computes the directed Hausdorff distance in both directions with the
/// default spherical geodesic and returns the larger of the two along with
/// both witnesses.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when either slice is empty, or any
/// validation error surfaced by [`Point::validate`].
pub fn hausdorff(a: &[Point], b: &[Point]) -> Result<HausdorffWitness, GeodistError> {
  hausdorff_with(&Spherical::default(), a, b)
}

/// Symmetric Hausdorff distance using a custom geodesic algorithm.
///
/// Executes [`hausdorff_directed_with`] in both directions and returns the
/// dominant leg so asymmetric paths are respected.
///
/// # Errors
/// Propagates the same validation and empty-set errors as [`hausdorff`].
pub fn hausdorff_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
) -> Result<HausdorffWitness, GeodistError> {
  let forward = hausdorff_directed_with(algorithm, a, b)?;
  let reverse = hausdorff_directed_with(algorithm, b, a)?;
  HausdorffWitness::new(forward, reverse)
}

/// Directed Hausdorff distance from set `a` to set `b` using ECEF chord
/// distance, returning the realizing witness pair.
///
/// Inputs are 3D geographic points expressed in degrees/meters. Returns the
/// maximum, over all points in `a`, of the minimum straight-line distance to
/// any point in `b`.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when either slice is empty, or any
/// validation error surfaced by [`Point3D::validate`].
pub fn hausdorff_directed_3d(a: &[Point3D], b: &[Point3D]) -> Result<HausdorffDirectedWitness, GeodistError> {
  hausdorff_directed_3d_on_ellipsoid(Ellipsoid::wgs84(), a, b)
}

/// Directed 3D Hausdorff distance using a custom ellipsoid for ECEF conversion.
///
/// Validates inputs and picks an evaluation strategy (naive vs indexed)
/// matching the 2D implementation.
///
/// # Errors
/// Propagates the same validation and empty-set errors as
/// [`hausdorff_directed_3d`]. Returns [`GeodistError::InvalidEllipsoid`] when
/// ellipsoid axes are not ordered or finite.
pub fn hausdorff_directed_3d_on_ellipsoid(
  ellipsoid: Ellipsoid,
  a: &[Point3D],
  b: &[Point3D],
) -> Result<HausdorffDirectedWitness, GeodistError> {
  ensure_non_empty(a)?;
  ensure_non_empty(b)?;
  validate_points_3d(a)?;
  validate_points_3d(b)?;
  ellipsoid.validate()?;

  let positioned_a = position_points_3d(a);
  let positioned_b = position_points_3d(b);
  let ecef_a = to_ecef_points(&positioned_a, &ellipsoid)?;
  let ecef_b = to_ecef_points(&positioned_b, &ellipsoid)?;

  hausdorff_directed_3d_from_ecef(&ecef_a, &ecef_b)
}

/// Symmetric 3D Hausdorff distance between sets `a` and `b`.
///
/// Computes the directed Hausdorff distance in both directions using the
/// default ellipsoid and returns the larger witness plus both legs.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when either slice is empty, or any
/// validation error surfaced by [`Point3D::validate`].
pub fn hausdorff_3d(a: &[Point3D], b: &[Point3D]) -> Result<HausdorffWitness, GeodistError> {
  hausdorff_3d_on_ellipsoid(Ellipsoid::wgs84(), a, b)
}

/// Symmetric 3D Hausdorff distance using a custom ellipsoid.
///
/// Delegates to [`hausdorff_directed_3d_on_ellipsoid`] in each direction.
///
/// # Errors
/// Propagates the same validation and empty-set errors as
/// [`hausdorff_3d`].
pub fn hausdorff_3d_on_ellipsoid(
  ellipsoid: Ellipsoid,
  a: &[Point3D],
  b: &[Point3D],
) -> Result<HausdorffWitness, GeodistError> {
  let forward = hausdorff_directed_3d_on_ellipsoid(ellipsoid, a, b)?;
  let reverse = hausdorff_directed_3d_on_ellipsoid(ellipsoid, b, a)?;
  HausdorffWitness::new(forward, reverse)
}

/// Directed Hausdorff distance between densified polylines.
///
/// Accepts MultiLineString inputs (`Vec<Vec<Point>>`), densifies them using
/// `options`, and computes the directed Hausdorff witness between the emitted
/// samples.
pub fn hausdorff_directed_polyline(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
) -> Result<PolylineDirectedWitness, GeodistError> {
  hausdorff_directed_polyline_with(&Spherical::default(), a, b, options)
}

/// Directed Hausdorff distance between densified polylines using a custom
/// geodesic algorithm.
pub fn hausdorff_directed_polyline_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
) -> Result<PolylineDirectedWitness, GeodistError> {
  hausdorff_directed_polyline_internal(algorithm, a, b, options, None)
}

/// Symmetric Hausdorff distance between densified polylines.
///
/// Executes directed evaluation in both directions with the default spherical
/// geodesic and returns the dominant witness plus both legs.
pub fn hausdorff_polyline(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
) -> Result<PolylineHausdorffWitness, GeodistError> {
  hausdorff_polyline_with(&Spherical::default(), a, b, options)
}

/// Symmetric Hausdorff distance between densified polylines using a custom
/// geodesic algorithm.
pub fn hausdorff_polyline_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
) -> Result<PolylineHausdorffWitness, GeodistError> {
  hausdorff_polyline_internal(algorithm, a, b, options, None)
}

/// Directed Hausdorff distance after clipping both sets by a bounding box.
///
/// Points outside `bounding_box` are discarded prior to the directed distance
/// calculation.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when filtering removes all points
/// from either slice, or any validation error surfaced by [`Point::validate`].
pub fn hausdorff_directed_clipped(
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<HausdorffDirectedWitness, GeodistError> {
  hausdorff_directed_clipped_with(&Spherical::default(), a, b, bounding_box)
}

/// Directed Hausdorff distance with custom algorithm after bounding box filter.
///
/// Applies the same clipping semantics as [`hausdorff_directed_clipped`] and
/// delegates distance measurements to a supplied geodesic algorithm.
///
/// # Errors
/// Propagates the same validation and empty-set errors as
/// [`hausdorff_directed_clipped`].
pub fn hausdorff_directed_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<HausdorffDirectedWitness, GeodistError> {
  let filtered_a = filter_points(a, &bounding_box);
  let filtered_b = filter_points(b, &bounding_box);
  ensure_non_empty(&filtered_a)?;
  ensure_non_empty(&filtered_b)?;
  validate_positioned_points(&filtered_a)?;
  validate_positioned_points(&filtered_b)?;

  hausdorff_directed_positioned_with(algorithm, &filtered_a, &filtered_b)
}

/// Symmetric Hausdorff distance after clipping both sets by a bounding box.
///
/// Filters inputs using `bounding_box` and computes the symmetric Hausdorff
/// distance with the default spherical geodesic.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when filtering removes all points
/// from either slice, or any validation error surfaced by [`Point::validate`].
pub fn hausdorff_clipped(
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<HausdorffWitness, GeodistError> {
  hausdorff_clipped_with(&Spherical::default(), a, b, bounding_box)
}

/// Symmetric Hausdorff distance with custom algorithm after bounding box
/// filter.
///
/// Applies the same clipping semantics as [`hausdorff_clipped`] while allowing
/// custom geodesic strategies (e.g., different radii or ellipsoid handling).
///
/// # Errors
/// Propagates the same validation and empty-set errors as
/// [`hausdorff_clipped`].
pub fn hausdorff_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<HausdorffWitness, GeodistError> {
  let filtered_a = filter_points(a, &bounding_box);
  let filtered_b = filter_points(b, &bounding_box);
  ensure_non_empty(&filtered_a)?;
  ensure_non_empty(&filtered_b)?;
  validate_positioned_points(&filtered_a)?;
  validate_positioned_points(&filtered_b)?;

  hausdorff_positioned(algorithm, &filtered_a, &filtered_b)
}

/// Directed Hausdorff distance between densified polylines after bounding box
/// clipping.
pub fn hausdorff_directed_polyline_clipped(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: BoundingBox,
) -> Result<PolylineDirectedWitness, GeodistError> {
  hausdorff_directed_polyline_clipped_with(&Spherical::default(), a, b, options, bounding_box)
}

/// Directed polyline Hausdorff distance with custom geodesic algorithm after
/// bounding box clipping.
pub fn hausdorff_directed_polyline_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: BoundingBox,
) -> Result<PolylineDirectedWitness, GeodistError> {
  hausdorff_directed_polyline_internal(algorithm, a, b, options, Some(bounding_box))
}

/// Symmetric Hausdorff distance between densified polylines after clipping to
/// a bounding box.
pub fn hausdorff_polyline_clipped(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: BoundingBox,
) -> Result<PolylineHausdorffWitness, GeodistError> {
  hausdorff_polyline_clipped_with(&Spherical::default(), a, b, options, bounding_box)
}

/// Symmetric polyline Hausdorff distance with custom geodesic algorithm and
/// bounding box clipping.
pub fn hausdorff_polyline_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: BoundingBox,
) -> Result<PolylineHausdorffWitness, GeodistError> {
  hausdorff_polyline_internal(algorithm, a, b, options, Some(bounding_box))
}

/// Directed 3D Hausdorff distance after clipping points to a bounding box.
///
/// Bounding is performed on latitude/longitude; altitude does not participate
/// in the clipping filter.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when filtering removes all points
/// from either slice, or any validation error surfaced by
/// [`Point3D::validate`].
pub fn hausdorff_directed_clipped_3d(
  a: &[Point3D],
  b: &[Point3D],
  bounding_box: BoundingBox,
) -> Result<HausdorffDirectedWitness, GeodistError> {
  hausdorff_directed_clipped_3d_on_ellipsoid(Ellipsoid::wgs84(), a, b, bounding_box)
}

/// Directed 3D Hausdorff distance on a custom ellipsoid after clipping points.
///
/// Applies the same bounding semantics as [`hausdorff_directed_clipped_3d`] but
/// allows callers to select the ellipsoid used for the ECEF projection.
///
/// # Errors
/// Propagates validation and empty-set errors from
/// [`hausdorff_directed_3d_on_ellipsoid`] and reports
/// [`GeodistError::InvalidEllipsoid`] when axes are malformed.
pub fn hausdorff_directed_clipped_3d_on_ellipsoid(
  ellipsoid: Ellipsoid,
  a: &[Point3D],
  b: &[Point3D],
  bounding_box: BoundingBox,
) -> Result<HausdorffDirectedWitness, GeodistError> {
  let filtered_a = filter_points_3d(a, &bounding_box);
  let filtered_b = filter_points_3d(b, &bounding_box);
  ensure_non_empty(&filtered_a)?;
  ensure_non_empty(&filtered_b)?;
  validate_positioned_points_3d(&filtered_a)?;
  validate_positioned_points_3d(&filtered_b)?;
  ellipsoid.validate()?;

  let ecef_a = to_ecef_points(&filtered_a, &ellipsoid)?;
  let ecef_b = to_ecef_points(&filtered_b, &ellipsoid)?;
  hausdorff_directed_3d_from_ecef(&ecef_a, &ecef_b)
}

/// Symmetric 3D Hausdorff distance after bounding box clipping.
///
/// Filters both sets with `bounding_box` on latitude/longitude and returns the
/// dominant directed distance using the default ellipsoid.
///
/// # Errors
/// Returns [`GeodistError::EmptyPointSet`] when filtering removes all points
/// from either slice, or any validation error surfaced by
/// [`Point3D::validate`].
pub fn hausdorff_clipped_3d(
  a: &[Point3D],
  b: &[Point3D],
  bounding_box: BoundingBox,
) -> Result<HausdorffWitness, GeodistError> {
  hausdorff_clipped_3d_on_ellipsoid(Ellipsoid::wgs84(), a, b, bounding_box)
}

/// Symmetric 3D Hausdorff distance on a custom ellipsoid after bounding.
///
/// Delegates to [`hausdorff_directed_clipped_3d_on_ellipsoid`] in both
/// directions, honoring the provided ellipsoid for ECEF conversion.
///
/// # Errors
/// Propagates the same validation, empty-set, and ellipsoid errors as
/// [`hausdorff_directed_clipped_3d_on_ellipsoid`].
pub fn hausdorff_clipped_3d_on_ellipsoid(
  ellipsoid: Ellipsoid,
  a: &[Point3D],
  b: &[Point3D],
  bounding_box: BoundingBox,
) -> Result<HausdorffWitness, GeodistError> {
  let filtered_a = filter_points_3d(a, &bounding_box);
  let filtered_b = filter_points_3d(b, &bounding_box);
  ensure_non_empty(&filtered_a)?;
  ensure_non_empty(&filtered_b)?;
  validate_positioned_points_3d(&filtered_a)?;
  validate_positioned_points_3d(&filtered_b)?;
  ellipsoid.validate()?;

  let ecef_a = to_ecef_points(&filtered_a, &ellipsoid)?;
  let ecef_b = to_ecef_points(&filtered_b, &ellipsoid)?;

  let forward = hausdorff_directed_3d_from_ecef(&ecef_a, &ecef_b)?;
  let reverse = hausdorff_directed_3d_from_ecef(&ecef_b, &ecef_a)?;
  HausdorffWitness::new(forward, reverse)
}

fn hausdorff_directed_polyline_internal<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: Option<BoundingBox>,
) -> Result<PolylineDirectedWitness, GeodistError> {
  let samples_a = densify_and_clip(a, options, bounding_box.as_ref())?;
  let samples_b = densify_and_clip(b, options, bounding_box.as_ref())?;
  hausdorff_directed_polyline_from_samples(algorithm, &samples_a, &samples_b)
}

fn hausdorff_polyline_internal<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: Option<BoundingBox>,
) -> Result<PolylineHausdorffWitness, GeodistError> {
  let samples_a = densify_and_clip(a, options, bounding_box.as_ref())?;
  let samples_b = densify_and_clip(b, options, bounding_box.as_ref())?;
  hausdorff_polyline_from_samples(algorithm, &samples_a, &samples_b)
}

const fn ensure_non_empty<T>(points: &[T]) -> Result<(), GeodistError> {
  if points.is_empty() {
    return Err(GeodistError::EmptyPointSet);
  }
  Ok(())
}

fn validate_points(points: &[Point]) -> Result<(), GeodistError> {
  for point in points {
    point.validate()?;
  }
  Ok(())
}

fn validate_points_3d(points: &[Point3D]) -> Result<(), GeodistError> {
  for point in points {
    point.validate()?;
  }
  Ok(())
}

fn validate_positioned_points(points: &[Positioned<Point>]) -> Result<(), GeodistError> {
  for positioned in points {
    positioned.point.validate()?;
  }
  Ok(())
}

fn validate_positioned_points_3d(points: &[Positioned<Point3D>]) -> Result<(), GeodistError> {
  for positioned in points {
    positioned.point.validate()?;
  }
  Ok(())
}

fn position_points(points: &[Point]) -> Vec<Positioned<Point>> {
  points
    .iter()
    .copied()
    .enumerate()
    .map(|(index, point)| Positioned { point, index })
    .collect()
}

fn position_points_3d(points: &[Point3D]) -> Vec<Positioned<Point3D>> {
  points
    .iter()
    .copied()
    .enumerate()
    .map(|(index, point)| Positioned { point, index })
    .collect()
}

fn filter_points(points: &[Point], bounding_box: &BoundingBox) -> Vec<Positioned<Point>> {
  points
    .iter()
    .copied()
    .enumerate()
    .filter(|(_, point)| bounding_box.contains(point))
    .map(|(index, point)| Positioned { point, index })
    .collect()
}

fn filter_points_3d(points: &[Point3D], bounding_box: &BoundingBox) -> Vec<Positioned<Point3D>> {
  points
    .iter()
    .copied()
    .enumerate()
    .filter(|(_, point)| bounding_box.contains_3d(point))
    .map(|(index, point)| Positioned { point, index })
    .collect()
}

fn densify_and_clip(
  parts: &[Vec<Point>],
  options: DensificationOptions,
  bounding_box: Option<&BoundingBox>,
) -> Result<FlattenedPolyline, GeodistError> {
  let densified = densify_multiline(parts, options)?;

  match bounding_box {
    Some(bbox) => densified.clip(bbox),
    None => Ok(densified),
  }
}

fn to_ecef_points(
  points: &[Positioned<Point3D>],
  ellipsoid: &Ellipsoid,
) -> Result<Vec<Positioned<EcefPoint>>, GeodistError> {
  points
    .iter()
    .map(|positioned| {
      geodetic_to_ecef(positioned.point, ellipsoid).map(|ecef| Positioned {
        point: ecef,
        index: positioned.index,
      })
    })
    .collect()
}

fn hausdorff_directed_polyline_from_samples<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
) -> Result<PolylineDirectedWitness, GeodistError> {
  let witness = hausdorff_directed_with(algorithm, origins.samples(), candidates.samples())?;
  let (source_part, source_index) = origins.part_and_index(witness.origin_index())?;
  let (target_part, target_index) = candidates.part_and_index(witness.candidate_index())?;

  Ok(PolylineDirectedWitness {
    distance: witness.distance(),
    source_part,
    source_index,
    target_part,
    target_index,
    source_coord: origins.samples()[witness.origin_index()],
    target_coord: candidates.samples()[witness.candidate_index()],
  })
}

fn hausdorff_polyline_from_samples<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &FlattenedPolyline,
  b: &FlattenedPolyline,
) -> Result<PolylineHausdorffWitness, GeodistError> {
  let forward = hausdorff_directed_polyline_from_samples(algorithm, a, b)?;
  let reverse = hausdorff_directed_polyline_from_samples(algorithm, b, a)?;
  let meters = forward.distance().meters().max(reverse.distance().meters());
  let distance = Distance::from_meters(meters)?;

  Ok(PolylineHausdorffWitness {
    distance,
    a_to_b: forward,
    b_to_a: reverse,
  })
}

fn choose_strategy(a_len: usize, b_len: usize) -> HausdorffStrategy {
  if should_use_naive(a_len, b_len) {
    HausdorffStrategy::Naive
  } else {
    HausdorffStrategy::Indexed
  }
}

fn should_use_naive(a_len: usize, b_len: usize) -> bool {
  let min_size = a_len.min(b_len);
  let cross_product = a_len.saturating_mul(b_len);
  min_size < MIN_INDEX_CANDIDATE_SIZE || cross_product <= MAX_NAIVE_CROSS_PRODUCT
}

fn prefer_candidate(current: &Option<DirectedHausdorffMeters>, witness: &DirectedHausdorffMeters) -> bool {
  match current {
    None => true,
    Some(existing) => {
      witness.meters < existing.meters
        || ((witness.meters - existing.meters).abs() <= f64::EPSILON
          && witness.candidate_index < existing.candidate_index)
    }
  }
}

fn should_break_search(current: &DirectedHausdorffMeters, candidate_distance: f64) -> bool {
  candidate_distance - current.meters > f64::EPSILON
}

fn hausdorff_directed_positioned_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &[Positioned<Point>],
  candidates: &[Positioned<Point>],
) -> Result<HausdorffDirectedWitness, GeodistError> {
  ensure_non_empty(origins)?;
  ensure_non_empty(candidates)?;

  let strategy = choose_strategy(origins.len(), candidates.len());
  let raw = match strategy {
    HausdorffStrategy::Naive => hausdorff_directed_naive(algorithm, origins, candidates)?,
    HausdorffStrategy::Indexed => hausdorff_directed_indexed(algorithm, origins, candidates)?,
  };

  HausdorffDirectedWitness::from_raw(raw)
}

fn hausdorff_positioned<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Positioned<Point>],
  b: &[Positioned<Point>],
) -> Result<HausdorffWitness, GeodistError> {
  let forward = hausdorff_directed_positioned_with(algorithm, a, b)?;
  let reverse = hausdorff_directed_positioned_with(algorithm, b, a)?;
  HausdorffWitness::new(forward, reverse)
}

fn hausdorff_directed_3d_from_ecef(
  origins: &[Positioned<EcefPoint>],
  candidates: &[Positioned<EcefPoint>],
) -> Result<HausdorffDirectedWitness, GeodistError> {
  ensure_non_empty(origins)?;
  ensure_non_empty(candidates)?;

  let strategy = choose_strategy(origins.len(), candidates.len());
  let raw = match strategy {
    HausdorffStrategy::Naive => hausdorff_directed_3d_naive(origins, candidates),
    HausdorffStrategy::Indexed => hausdorff_directed_3d_indexed(origins, candidates),
  };

  HausdorffDirectedWitness::from_raw(raw)
}

fn hausdorff_directed_naive<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &[Positioned<Point>],
  candidates: &[Positioned<Point>],
) -> Result<DirectedHausdorffMeters, GeodistError> {
  let mut best: Option<DirectedHausdorffMeters> = None;

  for origin in origins {
    let mut nearest: Option<(f64, usize)> = None;
    for candidate in candidates {
      let meters = algorithm.geodesic_distance(origin.point, candidate.point)?.meters();
      if nearest.is_none_or(|(current, _)| meters < current) {
        nearest = Some((meters, candidate.index));
      }
    }

    let (min_distance, nearest_index) = nearest.expect("candidate set validated as non-empty");
    let witness = DirectedHausdorffMeters {
      meters: min_distance,
      origin_index: origin.index,
      candidate_index: nearest_index,
    };

    if best.is_none_or(|current| witness.meters > current.meters) {
      best = Some(witness);
    }
  }

  best.ok_or(GeodistError::EmptyPointSet)
}

fn hausdorff_directed_indexed<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &[Positioned<Point>],
  candidates: &[Positioned<Point>],
) -> Result<DirectedHausdorffMeters, GeodistError> {
  let index = RTree::bulk_load(index_points(algorithm, candidates));
  let mut best: Option<DirectedHausdorffMeters> = None;

  for origin in origins {
    let query = [origin.point.lon, origin.point.lat];
    let mut nearest: Option<DirectedHausdorffMeters> = None;

    for candidate in index.nearest_neighbor_iter(&query) {
      let meters = algorithm.geodesic_distance(origin.point, candidate.point)?.meters();
      let witness = DirectedHausdorffMeters {
        meters,
        origin_index: origin.index,
        candidate_index: candidate.source_index,
      };

      if prefer_candidate(&nearest, &witness) {
        nearest = Some(witness);
      }

      if let Some(current) = nearest
        && should_break_search(&current, meters)
      {
        break;
      }
    }

    let origin_witness = nearest.expect("candidate set validated as non-empty");

    if best.is_none_or(|current| origin_witness.meters > current.meters) {
      best = Some(origin_witness);
    }
  }

  best.ok_or(GeodistError::EmptyPointSet)
}

/// Directed 3D Hausdorff distance using a naive O(n*m) search.
///
/// Iterates over every origin/candidate pair and returns the maximum of the
/// per-origin nearest-neighbor distances measured in ECEF meters.
fn hausdorff_directed_3d_naive(
  origins: &[Positioned<EcefPoint>],
  candidates: &[Positioned<EcefPoint>],
) -> DirectedHausdorffMeters {
  let mut best: Option<DirectedHausdorffMeters> = None;

  for origin in origins {
    let mut nearest: Option<(f64, usize)> = None;
    for candidate in candidates {
      let meters = origin.point.distance_to(candidate.point);
      if nearest.is_none_or(|(current, _)| meters < current) {
        nearest = Some((meters, candidate.index));
      }
    }

    let (min_distance, nearest_index) = nearest.expect("candidate set validated as non-empty");
    let witness = DirectedHausdorffMeters {
      meters: min_distance,
      origin_index: origin.index,
      candidate_index: nearest_index,
    };

    if best.is_none_or(|current| witness.meters > current.meters) {
      best = Some(witness);
    }
  }

  best.expect("origin set validated as non-empty")
}

/// Directed 3D Hausdorff distance using an R-tree for nearest-neighbor lookup.
///
/// Builds an index over the candidate set and queries the closest point for
/// each origin, computing distances in ECEF meters.
fn hausdorff_directed_3d_indexed(
  origins: &[Positioned<EcefPoint>],
  candidates: &[Positioned<EcefPoint>],
) -> DirectedHausdorffMeters {
  let index = RTree::bulk_load(index_ecef_points(candidates));
  let mut best: Option<DirectedHausdorffMeters> = None;

  for origin in origins {
    let query = [origin.point.x, origin.point.y, origin.point.z];
    let mut nearest: Option<DirectedHausdorffMeters> = None;

    for candidate in index.nearest_neighbor_iter(&query) {
      let meters = origin.point.distance_to(candidate.point);
      let witness = DirectedHausdorffMeters {
        meters,
        origin_index: origin.index,
        candidate_index: candidate.source_index,
      };

      if prefer_candidate(&nearest, &witness) {
        nearest = Some(witness);
      }

      if let Some(current) = nearest
        && should_break_search(&current, meters)
      {
        break;
      }
    }

    let origin_witness = nearest.expect("candidate set validated as non-empty");

    if best.is_none_or(|current| origin_witness.meters > current.meters) {
      best = Some(origin_witness);
    }
  }

  best.expect("origin set validated as non-empty")
}

fn index_points<'a, A: GeodesicAlgorithm>(algorithm: &'a A, points: &[Positioned<Point>]) -> Vec<IndexedPoint<'a, A>> {
  points
    .iter()
    .copied()
    .map(|positioned| IndexedPoint {
      algorithm,
      point: positioned.point,
      source_index: positioned.index,
    })
    .collect()
}

fn index_ecef_points(points: &[Positioned<EcefPoint>]) -> Vec<IndexedEcefPoint> {
  points
    .iter()
    .copied()
    .map(|positioned| IndexedEcefPoint {
      point: positioned.point,
      source_index: positioned.index,
    })
    .collect()
}

#[derive(Clone, Copy)]
struct IndexedPoint<'a, A> {
  algorithm: &'a A,
  point: Point,
  source_index: usize,
}

impl<'a, A> RTreeObject for IndexedPoint<'a, A> {
  type Envelope = AABB<[f64; 2]>;

  fn envelope(&self) -> Self::Envelope {
    AABB::from_point([self.point.lon, self.point.lat])
  }
}

impl<'a, A: GeodesicAlgorithm> PointDistance for IndexedPoint<'a, A> {
  fn distance_2(&self, point: &[f64; 2]) -> f64 {
    let query = Point {
      lat: point[1],
      lon: point[0],
    };

    match self.algorithm.geodesic_distance(self.point, query) {
      Ok(distance) => {
        let meters = distance.meters();
        meters * meters
      }
      Err(_) => f64::INFINITY,
    }
  }
}

#[derive(Clone, Copy)]
struct IndexedEcefPoint {
  point: EcefPoint,
  source_index: usize,
}

impl RTreeObject for IndexedEcefPoint {
  type Envelope = AABB<[f64; 3]>;

  fn envelope(&self) -> Self::Envelope {
    AABB::from_point([self.point.x, self.point.y, self.point.z])
  }
}

impl PointDistance for IndexedEcefPoint {
  fn distance_2(&self, point: &[f64; 3]) -> f64 {
    let query = EcefPoint::new(point[0], point[1], point[2]);
    self.point.squared_distance_to(query)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{geodesic_distance, geodesic_distance_3d};

  #[test]
  fn identical_sets_have_zero_distance() {
    let a = [Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()];
    let b = a;

    let d = hausdorff(&a, &b).unwrap();
    assert_eq!(d.distance().meters(), 0.0);
  }

  #[test]
  fn asymmetric_directed_distance() {
    let a = [Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 2.0).unwrap()];
    let b = [Point::new(0.0, 0.0).unwrap()];

    let directed = hausdorff_directed(&a, &b).unwrap().distance().meters();
    let expected = geodesic_distance(a[1], b[0]).unwrap().meters();
    assert!((directed - expected).abs() < 1e-9);

    let symmetric = hausdorff(&a, &b).unwrap().distance().meters();
    assert_eq!(symmetric, directed);
  }

  #[test]
  fn directed_witness_reports_indices() {
    let far = Point::new(0.0, 2.0).unwrap();
    let near = Point::new(0.0, 0.5).unwrap();
    let candidate = Point::new(0.0, 0.0).unwrap();

    let witness = hausdorff_directed(&[far, near], &[candidate]).unwrap();
    assert_eq!(witness.origin_index(), 0);
    assert_eq!(witness.candidate_index(), 0);
  }

  #[test]
  fn symmetric_witness_captures_both_directions() {
    let a = [Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()];
    let b = [Point::new(0.0, 0.0).unwrap()];

    let witness = hausdorff(&a, &b).unwrap();
    assert_eq!(witness.a_to_b().origin_index(), 1);
    assert_eq!(witness.a_to_b().candidate_index(), 0);
    assert_eq!(witness.b_to_a().origin_index(), 0);
    assert_eq!(witness.b_to_a().candidate_index(), 0);
    assert_eq!(witness.distance().meters(), witness.a_to_b().distance().meters());
  }

  #[test]
  fn rejects_empty_inputs() {
    let point = Point::new(0.0, 0.0).unwrap();
    assert!(matches!(
      hausdorff_directed(&[], &[point]),
      Err(GeodistError::EmptyPointSet)
    ));
    assert!(matches!(hausdorff(&[point], &[]), Err(GeodistError::EmptyPointSet)));
  }

  #[test]
  fn propagates_validation_error() {
    let bad_point = Point { lat: 95.0, lon: 0.0 };
    let good_point = Point::new(0.0, 0.0).unwrap();

    let result = hausdorff_directed(&[bad_point], &[good_point]);
    assert!(matches!(result, Err(GeodistError::InvalidLatitude(95.0))));
  }

  #[test]
  fn accepts_custom_algorithm_strategy() {
    struct ZeroAlgorithm;

    impl GeodesicAlgorithm for ZeroAlgorithm {
      fn geodesic_distance(&self, _p1: Point, _p2: Point) -> Result<Distance, GeodistError> {
        Distance::from_meters(0.0)
      }
    }

    let a = [Point::new(0.0, 0.0).unwrap()];
    let b = [Point::new(10.0, 10.0).unwrap()];

    let distance = hausdorff_with(&ZeroAlgorithm, &a, &b).unwrap();
    assert_eq!(distance.distance().meters(), 0.0);
  }

  #[test]
  fn uses_index_for_large_sets() {
    let a: Vec<Point> = (0..70).map(|i| Point::new(0.0, i as f64 * 0.1).unwrap()).collect();
    let b: Vec<Point> = (0..70)
      .map(|i| Point::new(0.0, i as f64 * 0.1 + 0.05).unwrap())
      .collect();

    let distance = hausdorff_directed(&a, &b).unwrap().distance().meters();
    let expected = geodesic_distance(a[0], b[0]).unwrap().meters();

    assert!((distance - expected).abs() < 1e-6);
  }

  #[test]
  fn strategy_prefers_naive_for_small_inputs() {
    assert!(should_use_naive(10, 100));
    assert!(should_use_naive(60, 60));
    assert!(!should_use_naive(70, 70));
  }

  #[test]
  fn clips_points_before_distance() {
    let inside = Point::new(0.0, 0.0).unwrap();
    let outside = Point::new(50.0, 50.0).unwrap();
    let bounding_box = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let distance = hausdorff_clipped(&[inside, outside], &[inside], bounding_box).unwrap();
    assert_eq!(distance.distance().meters(), 0.0);
  }

  #[test]
  fn clipped_variants_error_when_filter_removes_all_points() {
    let point = Point::new(10.0, 10.0).unwrap();
    let bounding_box = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let result = hausdorff_clipped(&[point], &[point], bounding_box);
    assert!(matches!(result, Err(GeodistError::EmptyPointSet)));
  }

  #[test]
  fn clipped_directed_preserves_original_indices() {
    let inside = Point::new(0.0, 0.0).unwrap(); // index 0
    let outside = Point::new(10.0, 10.0).unwrap(); // index 1
    let bounding_box = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let witness = hausdorff_directed_clipped(&[inside, outside], &[inside], bounding_box).unwrap();
    assert_eq!(witness.origin_index(), 0);
    assert_eq!(witness.candidate_index(), 0);
  }

  #[test]
  fn identical_3d_sets_have_zero_distance() {
    let a = [
      Point3D::new(0.0, 0.0, 0.0).unwrap(),
      Point3D::new(0.0, 1.0, 10.0).unwrap(),
    ];
    let b = a;

    let distance = hausdorff_3d(&a, &b).unwrap();
    assert_eq!(distance.distance().meters(), 0.0);
  }

  #[test]
  fn directed_3d_distance_reflects_altitude_delta() {
    let ground = Point3D::new(0.0, 0.0, 0.0).unwrap();
    let elevated = Point3D::new(0.0, 0.0, 500.0).unwrap();

    let directed = hausdorff_directed_3d(&[ground], &[elevated]).unwrap();
    assert!((directed.distance().meters() - 500.0).abs() < 1e-9);
  }

  #[test]
  fn clipped_variants_filter_3d_points() {
    let inside = Point3D::new(0.0, 0.0, 100.0).unwrap();
    let outside = Point3D::new(10.0, 0.0, 0.0).unwrap();
    let bounding_box = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let distance = hausdorff_clipped_3d(&[inside, outside], &[inside], bounding_box).unwrap();
    assert_eq!(distance.distance().meters(), 0.0);
  }

  #[test]
  fn uses_index_for_large_sets_3d() {
    let a: Vec<Point3D> = (0..70)
      .map(|i| Point3D::new(0.0, i as f64 * 0.1, 0.0).unwrap())
      .collect();
    let b: Vec<Point3D> = (0..70)
      .map(|i| Point3D::new(0.0, i as f64 * 0.1 + 0.05, 0.0).unwrap())
      .collect();

    let distance = hausdorff_directed_3d(&a, &b).unwrap().distance().meters();
    let expected = geodesic_distance_3d(a[0], b[0]).unwrap().meters();

    assert!((distance - expected).abs() < 1e-6);
  }

  #[test]
  fn directed_3d_witness_reports_indices() {
    let far = Point3D::new(0.0, 0.0, 100.0).unwrap();
    let near = Point3D::new(0.0, 0.0, 10.0).unwrap();
    let candidate = Point3D::new(0.0, 0.0, 0.0).unwrap();

    let witness = hausdorff_directed_3d(&[far, near], &[candidate]).unwrap();
    assert_eq!(witness.origin_index(), 0);
    assert_eq!(witness.candidate_index(), 0);
    assert!((witness.distance().meters() - 100.0).abs() < 1e-9);
  }

  #[test]
  fn indexed_search_breaks_ties_by_candidate_order() {
    let origins: Vec<Point> = (0..35).map(|_| Point::new(0.0, 0.0).unwrap()).collect();
    let mut candidates = vec![Point::new(0.0, 1.0).unwrap(), Point::new(0.0, -1.0).unwrap()];
    candidates.extend(
      (0..198)
        .map(|i| Point::new(10.0, i as f64 * 0.1).unwrap())
        .collect::<Vec<Point>>(),
    );

    let witness = hausdorff_directed(&origins, &candidates).unwrap();
    assert_eq!(witness.candidate_index(), 0);
    assert_eq!(witness.origin_index(), 0);
  }

  #[test]
  fn polyline_directed_maps_parts_and_indices() {
    let options = DensificationOptions {
      max_segment_length_m: Some(5_000_000.0),
      max_segment_angle_deg: None,
      sample_cap: 100,
    };

    let a = vec![
      vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 10.0).unwrap()],
      vec![Point::new(5.0, 0.0).unwrap(), Point::new(5.0, 10.0).unwrap()],
    ];
    let b = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 10.0).unwrap()]];

    let witness = hausdorff_directed_polyline(&a, &b, options).unwrap();

    assert_eq!(witness.source_part(), 1);
    assert_eq!(witness.source_index(), 0);
    assert_eq!(witness.target_part(), 0);
    assert_eq!(witness.target_index(), 0);
    assert_eq!(witness.source_coord(), Point::new(5.0, 0.0).unwrap());
    assert_eq!(witness.target_coord(), Point::new(0.0, 0.0).unwrap());
  }

  #[test]
  fn polyline_clipped_errors_when_no_samples_survive() {
    let options = DensificationOptions::default();
    let a = vec![vec![Point::new(10.0, 10.0).unwrap(), Point::new(10.0, 10.5).unwrap()]];
    let bbox = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let result = hausdorff_polyline_clipped(&a, &a, options, bbox);
    assert!(matches!(result, Err(GeodistError::EmptyPointSet)));
  }

  #[test]
  fn strategy_cutover_respects_cross_product_threshold() {
    assert!(should_use_naive(32, 125));
    assert!(!should_use_naive(32, 126));
  }
}
