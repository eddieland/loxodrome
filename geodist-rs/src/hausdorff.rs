//! Hausdorff distance between point sets using the spherical geodesic kernel.
//!
//! Inputs are degrees; output is meters.

use crate::{geodesic_distance, Distance, GeodistError, Point};

/// Directed Hausdorff distance from set `a` to set `b`.
///
/// Returns the maximum, over all points in `a`, of the minimum distance to any
/// point in `b`.
pub fn hausdorff_directed(a: &[Point], b: &[Point]) -> Result<Distance, GeodistError> {
  ensure_non_empty(a)?;
  ensure_non_empty(b)?;
  validate_points(a)?;
  validate_points(b)?;

  let mut max_min: f64 = 0.0;

  for origin in a {
    let mut min_distance: f64 = f64::INFINITY;
    for candidate in b {
      let meters = geodesic_distance(*origin, *candidate)?.meters();
      min_distance = min_distance.min(meters);
    }
    max_min = max_min.max(min_distance);
  }

  Distance::from_meters(max_min)
}

/// Symmetric Hausdorff distance between sets `a` and `b`.
pub fn hausdorff(a: &[Point], b: &[Point]) -> Result<Distance, GeodistError> {
  let forward = hausdorff_directed(a, b)?;
  let reverse = hausdorff_directed(b, a)?;
  let meters = forward.meters().max(reverse.meters());
  Distance::from_meters(meters)
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

#[cfg(test)]
mod tests {
  use super::*;

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
    assert!(matches!(
      hausdorff(&[point], &[]),
      Err(GeodistError::EmptyPointSet)
    ));
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
}
