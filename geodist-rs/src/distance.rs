//! Great-circle distance on a spherical Earth (WGS84 mean radius).
//!
//! Inputs are degrees; output is meters.

use crate::algorithms::{GeodesicAlgorithm, Spherical};
use crate::{Distance, Ellipsoid, GeodistError, Point, Point3D};

/// Distance plus forward and reverse bearings for a geodesic path.
///
/// Bearings are measured clockwise from north in degrees and normalized to
/// `[0, 360)`. The stored distance is a validated meter measurement that
/// matches the model used to compute the bearings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodesicSolution {
  distance: Distance,
  initial_bearing_degrees: f64,
  final_bearing_degrees: f64,
}

impl GeodesicSolution {
  /// Distance traveled along the geodesic in meters.
  pub fn distance(&self) -> Distance {
    self.distance
  }

  /// Initial bearing (forward azimuth) in degrees, normalized to `[0, 360)`.
  pub fn initial_bearing_degrees(&self) -> f64 {
    self.initial_bearing_degrees
  }

  /// Final bearing (reverse azimuth) in degrees, normalized to `[0, 360)`.
  pub fn final_bearing_degrees(&self) -> f64 {
    self.final_bearing_degrees
  }
}

/// Compute great-circle (geodesic) distance between two geographic points in
/// degrees using the default spherical algorithm.
///
/// Uses the WGS84 mean radius and validates inputs before calculating the
/// haversine arc length.
///
/// # Errors
///
/// Returns [`GeodistError`] when either coordinate is invalid or when the
/// resulting meter value cannot be represented as a [`Distance`].
pub fn geodesic_distance(p1: Point, p2: Point) -> Result<Distance, GeodistError> {
  geodesic_distance_with(&Spherical::default(), p1, p2)
}

/// Compute geodesic distance using a custom algorithm strategy.
///
/// Inputs are degrees. Validation and error semantics are defined by the
/// provided [`GeodesicAlgorithm`].
///
/// # Errors
///
/// Propagates the first [`GeodistError`] returned by `algorithm`.
pub fn geodesic_distance_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  p1: Point,
  p2: Point,
) -> Result<Distance, GeodistError> {
  algorithm.geodesic_distance(p1, p2)
}

/// Compute geodesic distance using a custom spherical radius.
///
/// Inputs are degrees; `radius_meters` must be finite and positive.
///
/// # Errors
///
/// Returns [`GeodistError::InvalidRadius`] for non-finite or non-positive
/// radii and propagates point validation or distance construction errors
/// surfaced while computing the geodesic.
pub fn geodesic_distance_on_radius(radius_meters: f64, p1: Point, p2: Point) -> Result<Distance, GeodistError> {
  let strategy = Spherical::with_radius(radius_meters)?;
  geodesic_distance_with(&strategy, p1, p2)
}

/// Compute geodesic distance using the mean radius of an ellipsoid.
///
/// Inputs are degrees; the ellipsoid is reduced to its mean radius before
/// computing the great-circle distance.
///
/// # Errors
///
/// Returns [`GeodistError::InvalidEllipsoid`] when axes are not valid or any
/// point validation or distance construction error encountered during
/// calculation.
pub fn geodesic_distance_on_ellipsoid(ellipsoid: Ellipsoid, p1: Point, p2: Point) -> Result<Distance, GeodistError> {
  let strategy = Spherical::from_ellipsoid(ellipsoid)?;
  geodesic_distance_with(&strategy, p1, p2)
}

/// Compute straight-line (ECEF chord) distance between two 3D geographic
/// points.
///
/// Uses the WGS84 ellipsoid and treats altitude as meters above the reference
/// ellipsoid surface. Inputs are degrees for latitude/longitude and meters for
/// altitude.
///
/// # Errors
///
/// Returns [`GeodistError`] when coordinates or altitude are invalid, or when
/// a valid [`Distance`] cannot be constructed.
pub fn geodesic_distance_3d(p1: Point3D, p2: Point3D) -> Result<Distance, GeodistError> {
  geodesic_distance_3d_on_ellipsoid(Ellipsoid::wgs84(), p1, p2)
}

/// Compute straight-line (ECEF chord) distance between two 3D points using a
/// custom ellipsoid.
///
/// Inputs are degrees for latitude/longitude and meters for altitude. The
/// ellipsoid defines the reference axes for converting to ECEF.
///
/// # Errors
///
/// Returns [`GeodistError`] when point validation fails, the ellipsoid axes
/// are invalid, or when the resulting meter value is not representable as a
/// [`Distance`].
pub fn geodesic_distance_3d_on_ellipsoid(
  ellipsoid: Ellipsoid,
  p1: Point3D,
  p2: Point3D,
) -> Result<Distance, GeodistError> {
  let ecef1 = geodetic_to_ecef(p1, &ellipsoid)?;
  let ecef2 = geodetic_to_ecef(p2, &ellipsoid)?;

  let dx = ecef1.0 - ecef2.0;
  let dy = ecef1.1 - ecef2.1;
  let dz = ecef1.2 - ecef2.2;
  let meters = (dx * dx + dy * dy + dz * dz).sqrt();

  Distance::from_meters(meters)
}

/// Compute geodesic distances for many point pairs in a single call.
///
/// Accepts a slice of `(origin, destination)` tuples and returns a `Vec` of
/// meter distances in the same order. Validation is performed for every point;
/// the first invalid coordinate returns an error and short-circuits.
///
/// # Errors
///
/// Returns the first [`GeodistError`] encountered while validating or
/// computing a pair.
pub fn geodesic_distances(pairs: &[(Point, Point)]) -> Result<Vec<f64>, GeodistError> {
  geodesic_distances_with(&Spherical::default(), pairs)
}

/// Compute batch geodesic distances with a custom algorithm strategy.
///
/// Inputs are degrees; results are meter distances in the same order.
///
/// # Errors
///
/// Propagates the first [`GeodistError`] returned by `algorithm`.
pub fn geodesic_distances_with<A: GeodesicAlgorithm>(
  algorithm: &A,
  pairs: &[(Point, Point)],
) -> Result<Vec<f64>, GeodistError> {
  algorithm.geodesic_distances(pairs)
}

/// Compute distance and bearings using the default spherical model.
///
/// Returns the forward and reverse bearings in degrees alongside a validated
/// meter distance, using the WGS84 mean radius.
///
/// # Errors
///
/// Returns [`GeodistError`] when either point is invalid or if the derived
/// distance cannot be represented.
pub fn geodesic_with_bearings(p1: Point, p2: Point) -> Result<GeodesicSolution, GeodistError> {
  geodesic_with_bearings_on_radius(Spherical::default().radius_meters(), p1, p2)
}

/// Compute distance and bearings using a custom spherical radius.
///
/// Bearings are normalized to `[0, 360)` and measured clockwise from north.
///
/// # Errors
///
/// Returns [`GeodistError::InvalidRadius`] when `radius_meters` is NaN,
/// infinite, or non-positive, or any validation or distance construction error
/// while computing the geodesic.
pub fn geodesic_with_bearings_on_radius(
  radius_meters: f64,
  p1: Point,
  p2: Point,
) -> Result<GeodesicSolution, GeodistError> {
  let strategy = Spherical::with_radius(radius_meters)?;
  geodesic_with_bearings_inner(strategy.radius_meters(), p1, p2)
}

/// Compute distance and bearings using an ellipsoid's mean radius.
///
/// Bearings follow the same conventions as [`geodesic_with_bearings`].
///
/// # Errors
///
/// Returns [`GeodistError::InvalidEllipsoid`] when ellipsoid parameters are
/// not valid or any point validation or distance construction error encountered
/// while computing the geodesic.
pub fn geodesic_with_bearings_on_ellipsoid(
  ellipsoid: Ellipsoid,
  p1: Point,
  p2: Point,
) -> Result<GeodesicSolution, GeodistError> {
  let radius_meters = ellipsoid.mean_radius()?;
  geodesic_with_bearings_on_radius(radius_meters, p1, p2)
}

fn geodesic_with_bearings_inner(radius_meters: f64, p1: Point, p2: Point) -> Result<GeodesicSolution, GeodistError> {
  p1.validate()?;
  p2.validate()?;

  let lat1 = p1.lat.to_radians();
  let lat2 = p2.lat.to_radians();
  let delta_lat = (p2.lat - p1.lat).to_radians();
  let delta_lon = (p2.lon - p1.lon).to_radians();

  let sin_lat = (delta_lat / 2.0).sin();
  let sin_lon = (delta_lon / 2.0).sin();

  let a = sin_lat * sin_lat + lat1.cos() * lat2.cos() * sin_lon * sin_lon;
  let normalized_a = a.clamp(0.0, 1.0);
  let c = 2.0 * normalized_a.sqrt().atan2((1.0 - normalized_a).sqrt());

  let meters = radius_meters * c;
  let distance = Distance::from_meters(meters)?;

  if meters == 0.0 {
    return Ok(GeodesicSolution {
      distance,
      initial_bearing_degrees: 0.0,
      final_bearing_degrees: 0.0,
    });
  }

  let initial = initial_bearing_from_radians(lat1, lat2, delta_lon);
  let reverse = initial_bearing_from_radians(lat2, lat1, -delta_lon);
  let final_bearing = normalize_bearing(reverse + 180.0);

  Ok(GeodesicSolution {
    distance,
    initial_bearing_degrees: initial,
    final_bearing_degrees: final_bearing,
  })
}

fn initial_bearing_from_radians(lat1: f64, lat2: f64, delta_lon: f64) -> f64 {
  let y = delta_lon.sin() * lat2.cos();
  let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();
  normalize_bearing(y.atan2(x).to_degrees())
}

fn normalize_bearing(mut degrees: f64) -> f64 {
  degrees %= 360.0;
  if degrees < 0.0 {
    degrees += 360.0;
  }
  degrees
}

fn geodetic_to_ecef(point: Point3D, ellipsoid: &Ellipsoid) -> Result<(f64, f64, f64), GeodistError> {
  point.validate()?;
  ellipsoid.validate()?;

  let a = ellipsoid.semi_major_axis_m;
  let b = ellipsoid.semi_minor_axis_m;
  let eccentricity_squared = 1.0 - (b * b) / (a * a);

  let lat = point.lat.to_radians();
  let lon = point.lon.to_radians();
  let sin_lat = lat.sin();
  let cos_lat = lat.cos();
  let sin_lon = lon.sin();
  let cos_lon = lon.cos();

  let surface_normal_radius = a / (1.0 - eccentricity_squared * sin_lat * sin_lat).sqrt();
  let altitude = point.altitude_m;

  let x = (surface_normal_radius + altitude) * cos_lat * cos_lon;
  let y = (surface_normal_radius + altitude) * cos_lat * sin_lon;
  let z = ((1.0 - eccentricity_squared) * surface_normal_radius + altitude) * sin_lat;

  Ok((x, y, z))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn computes_equatorial_degree() {
    let origin = Point::new(0.0, 0.0).unwrap();
    let east = Point::new(0.0, 1.0).unwrap();

    let meters = geodesic_distance(origin, east).unwrap().meters();
    let expected = 111_195.080_233_532_9;
    assert!((meters - expected).abs() < 1e-6);
  }

  #[test]
  fn handles_polar_antipodal_case() {
    let north_pole = Point::new(90.0, 0.0).unwrap();
    let south_pole = Point::new(-90.0, 0.0).unwrap();

    let meters = geodesic_distance(north_pole, south_pole).unwrap().meters();
    let expected = 20_015_114.442_035_925;
    assert!((meters - expected).abs() < 1e-6);
  }

  #[test]
  fn computes_long_range_path() {
    let new_york = Point::new(40.7128, -74.0060).unwrap();
    let london = Point::new(51.5074, -0.1278).unwrap();

    let meters = geodesic_distance(new_york, london).unwrap().meters();
    let expected = 5_570_229.873_656_523;
    assert!((meters - expected).abs() < 1e-6);
  }

  #[test]
  fn identical_points_are_zero() {
    let point = Point::new(10.0, 20.0).unwrap();
    let meters = geodesic_distance(point, point).unwrap().meters();
    assert_eq!(meters, 0.0);
  }

  #[test]
  fn computes_batch_distances_in_order() {
    let pairs = [
      (Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()),
      (Point::new(0.0, 0.0).unwrap(), Point::new(1.0, 0.0).unwrap()),
    ];

    let results = geodesic_distances(&pairs).unwrap();
    assert_eq!(results.len(), 2);

    let expected_first = geodesic_distance(pairs[0].0, pairs[0].1).unwrap().meters();
    let expected_second = geodesic_distance(pairs[1].0, pairs[1].1).unwrap().meters();

    assert!((results[0] - expected_first).abs() < 1e-9);
    assert!((results[1] - expected_second).abs() < 1e-9);
  }

  #[test]
  fn propagates_validation_error() {
    let valid = Point::new(0.0, 0.0).unwrap();
    let invalid = Point { lat: 95.0, lon: 0.0 };
    let pairs = [(valid, valid), (invalid, valid)];

    let result = geodesic_distances(&pairs);
    assert!(matches!(result, Err(GeodistError::InvalidLatitude(95.0))));
  }

  #[test]
  fn supports_custom_algorithm_for_single_distance() {
    struct FakeAlgorithm;

    impl GeodesicAlgorithm for FakeAlgorithm {
      fn geodesic_distance(&self, _p1: Point, _p2: Point) -> Result<Distance, GeodistError> {
        Distance::from_meters(42.0)
      }
    }

    let origin = Point::new(0.0, 0.0).unwrap();
    let destination = Point::new(1.0, 1.0).unwrap();

    let meters = geodesic_distance_with(&FakeAlgorithm, origin, destination)
      .unwrap()
      .meters();
    assert_eq!(meters, 42.0);
  }

  #[test]
  fn supports_custom_algorithm_for_batch() {
    struct ConstantAlgorithm;

    impl GeodesicAlgorithm for ConstantAlgorithm {
      fn geodesic_distance(&self, _p1: Point, _p2: Point) -> Result<Distance, GeodistError> {
        Distance::from_meters(1.5)
      }
    }

    let points = [
      (Point::new(0.0, 0.0).unwrap(), Point::new(0.0, 1.0).unwrap()),
      (Point::new(10.0, 10.0).unwrap(), Point::new(10.0, 11.0).unwrap()),
    ];

    let results = geodesic_distances_with(&ConstantAlgorithm, &points).unwrap();
    assert_eq!(results, vec![1.5, 1.5]);
  }

  #[test]
  fn computes_distance_on_custom_radius() {
    let origin = Point::new(0.0, 0.0).unwrap();
    let east = Point::new(0.0, 1.0).unwrap();

    let baseline = geodesic_distance(origin, east).unwrap().meters();
    let doubled = geodesic_distance_on_radius(10_000_000.0, origin, east)
      .unwrap()
      .meters();

    assert!(doubled > baseline);
  }

  #[test]
  fn computes_distance_on_ellipsoid_mean_radius() {
    let origin = Point::new(0.0, 0.0).unwrap();
    let east = Point::new(0.0, 1.0).unwrap();

    let ellipsoid = Ellipsoid::wgs84();
    let distance = geodesic_distance_on_ellipsoid(ellipsoid, origin, east).unwrap();

    assert!(distance.meters() > 100_000.0);
  }

  #[test]
  fn computes_bearings() {
    let origin = Point::new(0.0, 0.0).unwrap();
    let east = Point::new(0.0, 1.0).unwrap();

    let result = geodesic_with_bearings(origin, east).unwrap();
    assert!((result.initial_bearing_degrees() - 90.0).abs() < 1e-6);
    assert!((result.final_bearing_degrees() - 90.0).abs() < 1e-6);
  }

  #[test]
  fn computes_chord_distance_on_equator() {
    let origin = Point3D::new(0.0, 0.0, 0.0).unwrap();
    let east = Point3D::new(0.0, 1.0, 0.0).unwrap();

    let meters = geodesic_distance_3d(origin, east).unwrap().meters();
    let expected = 2.0 * Ellipsoid::wgs84().semi_major_axis_m * (0.5_f64.to_radians().sin());
    assert!((meters - expected).abs() < 1e-6);
  }

  #[test]
  fn altitude_only_distance_matches_vertical_delta() {
    let ground = Point3D::new(0.0, 0.0, 0.0).unwrap();
    let elevated = Point3D::new(0.0, 0.0, 1_000.0).unwrap();

    let meters = geodesic_distance_3d(ground, elevated).unwrap().meters();
    assert!((meters - 1_000.0).abs() < 1e-9);
  }

  #[test]
  fn geodesic_distance_3d_rejects_invalid_altitude() {
    let nan_point = Point3D {
      lat: 0.0,
      lon: 0.0,
      altitude_m: f64::NAN,
    };
    let valid = Point3D::new(0.0, 0.0, 0.0).unwrap();

    let result = geodesic_distance_3d(nan_point, valid);
    assert!(matches!(result, Err(GeodistError::InvalidAltitude(v)) if v.is_nan()));
  }
}
