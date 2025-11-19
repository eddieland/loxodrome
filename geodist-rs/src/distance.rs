//! Great-circle distance on a spherical Earth (WGS84 mean radius).
//!
//! Inputs are degrees; output is meters.

use crate::constants::EARTH_RADIUS_METERS;
use crate::{Distance, GeodistError, Point};

/// Compute great-circle (geodesic) distance between two geographic points in
/// degrees.
///
/// Returns a validated [`Distance`] in meters. Inputs are validated before
/// calculations.
pub fn geodesic_distance(p1: Point, p2: Point) -> Result<Distance, GeodistError> {
  p1.validate()?;
  p2.validate()?;

  let lat1 = p1.latitude.to_radians();
  let lat2 = p2.latitude.to_radians();
  let delta_lat = (p2.latitude - p1.latitude).to_radians();
  let delta_lon = (p2.longitude - p1.longitude).to_radians();

  let sin_lat = (delta_lat / 2.0).sin();
  let sin_lon = (delta_lon / 2.0).sin();

  let a = sin_lat * sin_lat + lat1.cos() * lat2.cos() * sin_lon * sin_lon;
  // Clamp to guard against minor floating error that could push `a` outside
  // [0, 1] and cause NaNs.
  let normalized_a = a.clamp(0.0, 1.0);
  let c = 2.0 * normalized_a.sqrt().atan2((1.0 - normalized_a).sqrt());

  let meters = EARTH_RADIUS_METERS * c;
  Distance::from_meters(meters)
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
}
