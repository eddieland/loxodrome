//! Hausdorff distance between point sets using the geodesic kernel.
//!
//! Inputs are degrees; output is meters.

use rstar::{AABB, PointDistance, RTree, RTreeObject};

use crate::algorithms::{GeodesicAlgorithm, Spherical};
use crate::{BoundingBox, Distance, GeodistError, Point};

// Keep the O(n*m) fallback for small collections where index build overhead
// outweighs nearest-neighbor savings.
const MIN_INDEX_CANDIDATE_SIZE: usize = 32;
const MAX_NAIVE_CROSS_PRODUCT: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HausdorffStrategy {
  Naive,
  Indexed,
}

/// Directed Hausdorff distance from set `a` to set `b`.
///
/// Returns the maximum, over all points in `a`, of the minimum distance to any
/// point in `b`.
pub fn hausdorff_directed(a: &[Point], b: &[Point]) -> Result<Distance, GeodistError> {
  hausdorff_directed_with(&Spherical::default(), a, b)
}

/// Directed Hausdorff distance using a custom geodesic algorithm.
pub fn hausdorff_directed_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
) -> Result<Distance, GeodistError> {
  ensure_non_empty(a)?;
  ensure_non_empty(b)?;
  validate_points(a)?;
  validate_points(b)?;

  let strategy = choose_strategy(a.len(), b.len());
  let meters = match strategy {
    HausdorffStrategy::Naive => hausdorff_directed_naive(algorithm, a, b)?,
    HausdorffStrategy::Indexed => hausdorff_directed_indexed(algorithm, a, b)?,
  };

  Distance::from_meters(meters)
}

/// Symmetric Hausdorff distance between sets `a` and `b`.
pub fn hausdorff(a: &[Point], b: &[Point]) -> Result<Distance, GeodistError> {
  hausdorff_with(&Spherical::default(), a, b)
}

/// Symmetric Hausdorff distance using a custom geodesic algorithm.
pub fn hausdorff_with<A: GeodesicAlgorithm>(algorithm: &A, a: &[Point], b: &[Point]) -> Result<Distance, GeodistError> {
  let forward = hausdorff_directed_with(algorithm, a, b)?;
  let reverse = hausdorff_directed_with(algorithm, b, a)?;
  let meters = forward.meters().max(reverse.meters());
  Distance::from_meters(meters)
}

/// Directed Hausdorff distance after clipping both sets by a bounding box.
pub fn hausdorff_directed_clipped(
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<Distance, GeodistError> {
  hausdorff_directed_clipped_with(&Spherical::default(), a, b, bounding_box)
}

/// Directed Hausdorff distance with custom algorithm after bounding box filter.
pub fn hausdorff_directed_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<Distance, GeodistError> {
  let filtered_a = filter_points(a, &bounding_box);
  let filtered_b = filter_points(b, &bounding_box);
  hausdorff_directed_with(algorithm, &filtered_a, &filtered_b)
}

/// Symmetric Hausdorff distance after clipping both sets by a bounding box.
pub fn hausdorff_clipped(a: &[Point], b: &[Point], bounding_box: BoundingBox) -> Result<Distance, GeodistError> {
  hausdorff_clipped_with(&Spherical::default(), a, b, bounding_box)
}

/// Symmetric Hausdorff distance with custom algorithm after bounding box
/// filter.
pub fn hausdorff_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Point],
  b: &[Point],
  bounding_box: BoundingBox,
) -> Result<Distance, GeodistError> {
  let filtered_a = filter_points(a, &bounding_box);
  let filtered_b = filter_points(b, &bounding_box);
  hausdorff_with(algorithm, &filtered_a, &filtered_b)
}

fn ensure_non_empty(points: &[Point]) -> Result<(), GeodistError> {
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

fn filter_points(points: &[Point], bounding_box: &BoundingBox) -> Vec<Point> {
  points
    .iter()
    .copied()
    .filter(|point| bounding_box.contains(point))
    .collect()
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

fn hausdorff_directed_naive<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &[Point],
  candidates: &[Point],
) -> Result<f64, GeodistError> {
  let mut max_min: f64 = 0.0;

  for origin in origins {
    let mut min_distance: f64 = f64::INFINITY;
    for candidate in candidates {
      let meters = algorithm.geodesic_distance(*origin, *candidate)?.meters();
      min_distance = min_distance.min(meters);
    }
    max_min = max_min.max(min_distance);
  }

  Ok(max_min)
}

fn hausdorff_directed_indexed<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &[Point],
  candidates: &[Point],
) -> Result<f64, GeodistError> {
  let index = RTree::bulk_load(index_points(algorithm, candidates));
  let mut max_min: f64 = 0.0;

  for origin in origins {
    let query = [origin.longitude, origin.latitude];
    let nearest = index
      .nearest_neighbor(&query)
      .expect("candidate set validated as non-empty");
    let meters = algorithm.geodesic_distance(*origin, nearest.point)?.meters();
    max_min = max_min.max(meters);
  }

  Ok(max_min)
}

fn index_points<'a, A: GeodesicAlgorithm>(algorithm: &'a A, points: &[Point]) -> Vec<IndexedPoint<'a, A>> {
  points
    .iter()
    .copied()
    .map(|point| IndexedPoint { algorithm, point })
    .collect()
}

#[derive(Clone, Copy)]
struct IndexedPoint<'a, A> {
  algorithm: &'a A,
  point: Point,
}

impl<'a, A> RTreeObject for IndexedPoint<'a, A> {
  type Envelope = AABB<[f64; 2]>;

  fn envelope(&self) -> Self::Envelope {
    AABB::from_point([self.point.longitude, self.point.latitude])
  }
}

impl<'a, A: GeodesicAlgorithm> PointDistance for IndexedPoint<'a, A> {
  fn distance_2(&self, point: &[f64; 2]) -> f64 {
    let query = Point {
      latitude: point[1],
      longitude: point[0],
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::geodesic_distance;

  #[test]
  fn identical_sets_have_zero_distance() {
    let a = [Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()];
    let b = a;

    let d = hausdorff(&a, &b).unwrap();
    assert_eq!(d.meters(), 0.0);
  }

  #[test]
  fn asymmetric_directed_distance() {
    let a = [Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 2.0).unwrap()];
    let b = [Point::new(0.0, 0.0).unwrap()];

    let directed = hausdorff_directed(&a, &b).unwrap().meters();
    let expected = geodesic_distance(a[1], b[0]).unwrap().meters();
    assert!((directed - expected).abs() < 1e-9);

    let symmetric = hausdorff(&a, &b).unwrap().meters();
    assert_eq!(symmetric, directed);
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
    let bad_point = Point {
      latitude: 95.0,
      longitude: 0.0,
    };
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
    assert_eq!(distance.meters(), 0.0);
  }

  #[test]
  fn uses_index_for_large_sets() {
    let a: Vec<Point> = (0..70).map(|i| Point::new(0.0, i as f64 * 0.1).unwrap()).collect();
    let b: Vec<Point> = (0..70)
      .map(|i| Point::new(0.0, i as f64 * 0.1 + 0.05).unwrap())
      .collect();

    let distance = hausdorff_directed(&a, &b).unwrap().meters();
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
    assert_eq!(distance.meters(), 0.0);
  }

  #[test]
  fn clipped_variants_error_when_filter_removes_all_points() {
    let point = Point::new(10.0, 10.0).unwrap();
    let bounding_box = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let result = hausdorff_clipped(&[point], &[point], bounding_box);
    assert!(matches!(result, Err(GeodistError::EmptyPointSet)));
  }
}
