//! Chamfer distance between densified polylines using nearest-neighbor search.
//!
//! Inputs are degrees; outputs are meters.

use rstar::{AABB, PointDistance, RTree, RTreeObject};

use crate::algorithms::{GeodesicAlgorithm, Spherical};
use crate::hausdorff::PolylineDirectedWitness;
use crate::polyline::{DensificationOptions, FlattenedPolyline, densify_multiline};
use crate::{BoundingBox, Distance, GeodistError, Point};

// Keep the O(n*m) fallback for small collections where index build overhead
// outweighs nearest-neighbor savings.
const MIN_INDEX_CANDIDATE_SIZE: usize = 32;
const MAX_NAIVE_CROSS_PRODUCT: usize = 4_000;
const WITNESS_TOLERANCE_M: f64 = 1e-12;

/// Reduction applied to per-origin nearest-neighbor distances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChamferReduction {
  #[default]
  Mean,
  Sum,
  Max,
}

/// Directed Chamfer result with an optional witness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChamferDirectedResult {
  distance: Distance,
  witness: Option<PolylineDirectedWitness>,
}

impl ChamferDirectedResult {
  /// Aggregated distance (mean/sum/max) from the origin polyline to the
  /// candidate polyline.
  pub const fn distance(&self) -> Distance {
    self.distance
  }

  /// Realizing witness when `reduction="max"`.
  pub const fn witness(&self) -> Option<PolylineDirectedWitness> {
    self.witness
  }
}

/// Symmetric Chamfer result containing both directions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChamferResult {
  distance: Distance,
  a_to_b: ChamferDirectedResult,
  b_to_a: ChamferDirectedResult,
}

impl ChamferResult {
  /// Aggregated Chamfer distance across both directions.
  pub const fn distance(&self) -> Distance {
    self.distance
  }

  /// Directed result from the first argument to the second.
  pub const fn a_to_b(&self) -> ChamferDirectedResult {
    self.a_to_b
  }

  /// Directed result from the second argument back to the first.
  pub const fn b_to_a(&self) -> ChamferDirectedResult {
    self.b_to_a
  }
}

/// Directed Chamfer distance between densified polylines.
pub fn chamfer_directed_polyline(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
) -> Result<ChamferDirectedResult, GeodistError> {
  chamfer_directed_polyline_with(&Spherical::default(), a, b, options, reduction)
}

/// Directed Chamfer distance with a custom geodesic algorithm.
pub fn chamfer_directed_polyline_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
) -> Result<ChamferDirectedResult, GeodistError> {
  chamfer_directed_polyline_internal(algorithm, a, b, options, reduction, None)
}

/// Symmetric Chamfer distance between densified polylines.
pub fn chamfer_polyline(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
) -> Result<ChamferResult, GeodistError> {
  chamfer_polyline_with(&Spherical::default(), a, b, options, reduction)
}

/// Symmetric Chamfer distance with a custom geodesic algorithm.
pub fn chamfer_polyline_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
) -> Result<ChamferResult, GeodistError> {
  chamfer_polyline_internal(algorithm, a, b, options, reduction, None)
}

/// Directed Chamfer distance between densified polylines after bounding box
/// clipping.
pub fn chamfer_directed_polyline_clipped(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
  bounding_box: BoundingBox,
) -> Result<ChamferDirectedResult, GeodistError> {
  chamfer_directed_polyline_internal(&Spherical::default(), a, b, options, reduction, Some(bounding_box))
}

/// Directed Chamfer distance with custom geodesic algorithm after bounding box
/// clipping.
pub fn chamfer_directed_polyline_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
  bounding_box: BoundingBox,
) -> Result<ChamferDirectedResult, GeodistError> {
  chamfer_directed_polyline_internal(algorithm, a, b, options, reduction, Some(bounding_box))
}

/// Symmetric Chamfer distance between densified polylines after bounding box
/// clipping.
pub fn chamfer_polyline_clipped(
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
  bounding_box: BoundingBox,
) -> Result<ChamferResult, GeodistError> {
  chamfer_polyline_internal(&Spherical::default(), a, b, options, reduction, Some(bounding_box))
}

/// Symmetric Chamfer distance with custom geodesic algorithm and clipping.
pub fn chamfer_polyline_clipped_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
  bounding_box: BoundingBox,
) -> Result<ChamferResult, GeodistError> {
  chamfer_polyline_internal(algorithm, a, b, options, reduction, Some(bounding_box))
}

fn chamfer_directed_polyline_internal<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
  bounding_box: Option<BoundingBox>,
) -> Result<ChamferDirectedResult, GeodistError> {
  let samples_a = densify_and_clip(a, options, bounding_box.as_ref())?;
  let samples_b = densify_and_clip(b, options, bounding_box.as_ref())?;
  chamfer_directed_from_samples(algorithm, &samples_a, &samples_b, reduction)
}

fn chamfer_polyline_internal<A: GeodesicAlgorithm>(
  algorithm: &A,
  a: &[Vec<Point>],
  b: &[Vec<Point>],
  options: DensificationOptions,
  reduction: ChamferReduction,
  bounding_box: Option<BoundingBox>,
) -> Result<ChamferResult, GeodistError> {
  let samples_a = densify_and_clip(a, options, bounding_box.as_ref())?;
  let samples_b = densify_and_clip(b, options, bounding_box.as_ref())?;

  let forward = chamfer_directed_from_samples(algorithm, &samples_a, &samples_b, reduction)?;
  let reverse = chamfer_directed_from_samples(algorithm, &samples_b, &samples_a, reduction)?;

  let meters = match reduction {
    ChamferReduction::Max => forward.distance().meters().max(reverse.distance().meters()),
    ChamferReduction::Mean | ChamferReduction::Sum => (forward.distance().meters() + reverse.distance().meters()) / 2.0,
  };
  let distance = Distance::from_meters(meters)?;

  Ok(ChamferResult {
    distance,
    a_to_b: forward,
    b_to_a: reverse,
  })
}

fn chamfer_directed_from_samples<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  reduction: ChamferReduction,
) -> Result<ChamferDirectedResult, GeodistError> {
  ensure_non_empty(origins)?;
  ensure_non_empty(candidates)?;

  let strategy = choose_strategy(origins.len(), candidates.len());
  match strategy {
    ChamferStrategy::Naive => chamfer_directed_naive(algorithm, origins, candidates, reduction),
    ChamferStrategy::Indexed => chamfer_directed_indexed(algorithm, origins, candidates, reduction),
  }
}

fn chamfer_directed_naive<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  reduction: ChamferReduction,
) -> Result<ChamferDirectedResult, GeodistError> {
  let mut sum = 0.0;
  let mut worst: Option<WitnessCandidate> = None;

  for (origin_index, origin) in origins.samples().iter().copied().enumerate() {
    let mut nearest: Option<(f64, usize)> = None;

    for (candidate_index, candidate) in candidates.samples().iter().copied().enumerate() {
      let meters = algorithm.geodesic_distance(origin, candidate)?.meters();

      if nearest.is_none_or(|(current, _)| meters < current)
        || (nearest.is_some_and(|(current, _)| (meters - current).abs() <= f64::EPSILON)
          && candidate_index < nearest.map(|(_, index)| index).unwrap_or(usize::MAX))
      {
        nearest = Some((meters, candidate_index));
      }
    }

    let (min_distance, candidate_index) = nearest.expect("candidate set validated as non-empty");
    sum += min_distance;

    if matches!(reduction, ChamferReduction::Max) {
      let candidate = WitnessCandidate {
        distance: min_distance,
        origin_index,
        candidate_index,
      };

      if prefer_witness(origins, candidates, worst.as_ref(), &candidate) {
        worst = Some(candidate);
      }
    }
  }

  finalize_directed_result(origins, candidates, reduction, sum, worst)
}

fn chamfer_directed_indexed<A: GeodesicAlgorithm>(
  algorithm: &A,
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  reduction: ChamferReduction,
) -> Result<ChamferDirectedResult, GeodistError> {
  let index = RTree::bulk_load(index_points(algorithm, candidates.samples()));

  let mut sum = 0.0;
  let mut worst: Option<WitnessCandidate> = None;

  for (origin_index, origin) in origins.samples().iter().copied().enumerate() {
    let query = [origin.lon, origin.lat];
    let mut nearest: Option<(f64, usize)> = None;

    for candidate in index.nearest_neighbor_iter(&query) {
      let meters = algorithm.geodesic_distance(origin, candidate.point)?.meters();

      if nearest.is_none_or(|(current, _)| meters < current)
        || (nearest.is_some_and(|(current, _)| (meters - current).abs() <= f64::EPSILON)
          && candidate.source_index < nearest.map(|(_, candidate_idx)| candidate_idx).unwrap_or(usize::MAX))
      {
        nearest = Some((meters, candidate.source_index));
      }

      if let Some((current_best, _)) = nearest
        && should_break_search(current_best, meters)
      {
        break;
      }
    }

    let (min_distance, candidate_index) = nearest.expect("candidate set validated as non-empty");
    sum += min_distance;

    if matches!(reduction, ChamferReduction::Max) {
      let candidate = WitnessCandidate {
        distance: min_distance,
        origin_index,
        candidate_index,
      };

      if prefer_witness(origins, candidates, worst.as_ref(), &candidate) {
        worst = Some(candidate);
      }
    }
  }

  finalize_directed_result(origins, candidates, reduction, sum, worst)
}

fn finalize_directed_result(
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  reduction: ChamferReduction,
  sum: f64,
  worst: Option<WitnessCandidate>,
) -> Result<ChamferDirectedResult, GeodistError> {
  let sample_count = origins.len() as f64;
  let (distance, witness) = match reduction {
    ChamferReduction::Mean => (Distance::from_meters(sum / sample_count)?, None),
    ChamferReduction::Sum => (Distance::from_meters(sum)?, None),
    ChamferReduction::Max => {
      let realizing = worst.expect("worst candidate recorded for max reduction");
      let witness = build_witness(origins, candidates, realizing)?;
      let distance = witness.distance();
      (distance, Some(witness))
    }
  };

  Ok(ChamferDirectedResult { distance, witness })
}

fn build_witness(
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  realizing: WitnessCandidate,
) -> Result<PolylineDirectedWitness, GeodistError> {
  let distance = Distance::from_meters(realizing.distance)?;
  PolylineDirectedWitness::from_indices(
    origins,
    candidates,
    distance,
    realizing.origin_index,
    realizing.candidate_index,
  )
}

fn witness_key(
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  candidate: &WitnessCandidate,
) -> (usize, usize, usize, usize) {
  let (source_part, source_index) = origins
    .part_and_index(candidate.origin_index)
    .expect("origin index generated from samples");
  let (target_part, target_index) = candidates
    .part_and_index(candidate.candidate_index)
    .expect("candidate index generated from samples");

  (source_part, source_index, target_part, target_index)
}

fn prefer_witness(
  origins: &FlattenedPolyline,
  candidates: &FlattenedPolyline,
  current: Option<&WitnessCandidate>,
  candidate: &WitnessCandidate,
) -> bool {
  match current {
    None => true,
    Some(existing) => {
      if candidate.distance - existing.distance > WITNESS_TOLERANCE_M {
        return true;
      }

      if (candidate.distance - existing.distance).abs() <= WITNESS_TOLERANCE_M {
        let existing_key = witness_key(origins, candidates, existing);
        let candidate_key = witness_key(origins, candidates, candidate);
        return candidate_key < existing_key;
      }

      false
    }
  }
}

fn choose_strategy(a_len: usize, b_len: usize) -> ChamferStrategy {
  if should_use_naive(a_len, b_len) {
    ChamferStrategy::Naive
  } else {
    ChamferStrategy::Indexed
  }
}

fn should_use_naive(a_len: usize, b_len: usize) -> bool {
  let min_size = a_len.min(b_len);
  let cross_product = a_len.saturating_mul(b_len);
  min_size < MIN_INDEX_CANDIDATE_SIZE || cross_product <= MAX_NAIVE_CROSS_PRODUCT
}

fn should_break_search(current_best: f64, candidate_distance: f64) -> bool {
  candidate_distance - current_best > f64::EPSILON
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

const fn ensure_non_empty(polyline: &FlattenedPolyline) -> Result<(), GeodistError> {
  if polyline.is_empty() {
    return Err(GeodistError::EmptyPointSet);
  }
  Ok(())
}

#[derive(Debug, Clone, Copy)]
struct WitnessCandidate {
  distance: f64,
  origin_index: usize,
  candidate_index: usize,
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

fn index_points<'a, A: GeodesicAlgorithm>(algorithm: &'a A, points: &[Point]) -> Vec<IndexedPoint<'a, A>> {
  points
    .iter()
    .copied()
    .enumerate()
    .map(|(index, point)| IndexedPoint {
      algorithm,
      point,
      source_index: index,
    })
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChamferStrategy {
  Naive,
  Indexed,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::geodesic_distance;

  #[test]
  fn chamfer_mean_matches_expected_average() {
    let line_a = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()]];
    let line_b = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 2.0).unwrap()]];
    let options = DensificationOptions {
      max_segment_length_m: Some(500_000.0),
      max_segment_angle_deg: None,
      sample_cap: 10_000,
    };

    let directed = chamfer_directed_polyline(&line_a, &line_b, options, ChamferReduction::Mean).unwrap();
    let expected =
      Distance::from_meters(geodesic_distance(line_a[0][1], line_b[0][0]).unwrap().meters() / 2.0).unwrap();

    assert!((directed.distance().meters() - expected.meters()).abs() < 1.0);
    assert!(directed.witness().is_none());
  }

  #[test]
  fn chamfer_sum_averages_directions() {
    let line_a = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()]];
    let line_b = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 2.0).unwrap()]];
    let options = DensificationOptions {
      max_segment_length_m: Some(500_000.0),
      max_segment_angle_deg: None,
      sample_cap: 10_000,
    };

    let symmetric = chamfer_polyline(&line_a, &line_b, options, ChamferReduction::Sum).unwrap();
    assert!(symmetric.distance().meters() > 0.0);
    assert!(symmetric.a_to_b().witness().is_none());
    assert!(symmetric.b_to_a().witness().is_none());
    assert!(symmetric.distance().meters() >= symmetric.a_to_b().distance().meters());
  }

  #[test]
  fn chamfer_max_emits_witness_with_tie_breaks() {
    let line_a = vec![vec![
      Point::new(0.0, 0.0).unwrap(),
      Point::new(0.0, 0.5).unwrap(),
      Point::new(0.0, 1.0).unwrap(),
    ]];
    let line_b = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 0.1).unwrap()]];
    let options = DensificationOptions {
      max_segment_length_m: Some(1_000_000.0),
      max_segment_angle_deg: None,
      sample_cap: 10_000,
    };

    let directed = chamfer_directed_polyline(&line_a, &line_b, options, ChamferReduction::Max).unwrap();
    let witness = directed.witness().expect("witness emitted for max reduction");
    assert_eq!(witness.source_part(), 0);
    assert_eq!(witness.source_index(), 2);
    assert_eq!(witness.target_index(), 1);
    assert_eq!(directed.distance(), witness.distance());
  }

  #[test]
  fn chamfer_clipped_errors_when_empty() {
    let line_a = vec![vec![Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()]];
    let line_b = vec![vec![Point::new(10.0, 0.0).unwrap(), Point::new(10.0, 1.0).unwrap()]];
    let options = DensificationOptions::default();
    let bbox = BoundingBox::new(-1.0, 1.0, -1.0, 1.0).unwrap();

    let result = chamfer_directed_polyline_clipped(&line_a, &line_b, options, ChamferReduction::Mean, bbox);
    assert!(matches!(result, Err(GeodistError::EmptyPointSet)));
  }
}
