//! Spherical great-circle implementation using the WGS84 mean radius.
//!
//! Inputs are degrees; output is meters.

use super::GeodesicAlgorithm;
use crate::{Distance, EARTH_RADIUS_METERS, Ellipsoid, GeodistError, Point};

/// Baseline spherical algorithm using a haversine great-circle model.
///
/// The default configuration uses the WGS84 mean radius and validates inputs
/// before computing a meter distance.
#[derive(Debug, Clone, Copy)]
pub struct Spherical {
  radius_meters: f64,
}

impl Default for Spherical {
  fn default() -> Self {
    Self {
      radius_meters: EARTH_RADIUS_METERS,
    }
  }
}

impl Spherical {
  /// Construct a spherical strategy with a custom radius.
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError::InvalidRadius`] if the provided radius is NaN,
  /// infinite, or non-positive.
  pub fn with_radius(radius_meters: f64) -> Result<Self, GeodistError> {
    if !radius_meters.is_finite() || radius_meters <= 0.0 {
      return Err(GeodistError::InvalidRadius(radius_meters));
    }
    Ok(Self { radius_meters })
  }

  /// Construct a spherical strategy using the mean radius of an ellipsoid.
  ///
  /// # Errors
  ///
  /// Propagates [`GeodistError::InvalidEllipsoid`] if the ellipsoid axes are
  /// not finite, positive, and ordered.
  pub fn from_ellipsoid(ellipsoid: Ellipsoid) -> Result<Self, GeodistError> {
    let radius_meters = ellipsoid.mean_radius()?;
    Self::with_radius(radius_meters)
  }

  /// Radius used by the strategy (meters).
  pub const fn radius_meters(&self) -> f64 {
    self.radius_meters
  }
}

impl GeodesicAlgorithm for Spherical {
  fn geodesic_distance(&self, p1: Point, p2: Point) -> Result<Distance, GeodistError> {
    spherical_distance(self.radius_meters, p1, p2)
  }
}

/// Compute spherical great-circle distance for a single pair.
///
/// Inputs are degrees and validated before calculating the haversine arc
/// length. Returns a [`Distance`] in meters based on the provided radius.
///
/// # Errors
///
/// Returns [`GeodistError`] if either point is invalid or if the meter value
/// cannot be expressed as a valid [`Distance`].
pub fn spherical_distance(radius_meters: f64, p1: Point, p2: Point) -> Result<Distance, GeodistError> {
  p1.validate()?;
  p2.validate()?;

  let lat1 = p1.lat.to_radians();
  let lat2 = p2.lat.to_radians();
  let delta_lat = (p2.lat - p1.lat).to_radians();
  let delta_lon = (p2.lon - p1.lon).to_radians();

  let sin_lat = (delta_lat / 2.0).sin();
  let sin_lon = (delta_lon / 2.0).sin();

  let a = sin_lat * sin_lat + lat1.cos() * lat2.cos() * sin_lon * sin_lon;
  // Clamp to guard against minor floating error that could push `a` outside
  // [0, 1] and cause NaNs.
  let normalized_a = a.clamp(0.0, 1.0);
  let c = 2.0 * normalized_a.sqrt().atan2((1.0 - normalized_a).sqrt());

  let meters = radius_meters * c;
  Distance::from_meters(meters)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn computes_expected_distance() {
    let origin = Point::new(0.0, 0.0).unwrap();
    let east = Point::new(0.0, 1.0).unwrap();

    let meters = Spherical::default().geodesic_distance(origin, east).unwrap().meters();
    let expected = 111_195.080_233_532_9;
    assert!((meters - expected).abs() < 1e-6);
  }

  #[test]
  fn propagates_validation_errors() {
    let invalid = Point { lat: 200.0, lon: 0.0 };
    let valid = Point::new(0.0, 0.0).unwrap();
    let result = Spherical::default().geodesic_distance(invalid, valid);
    assert!(matches!(result, Err(GeodistError::InvalidLatitude(200.0))));
  }

  #[test]
  fn accepts_custom_radius() {
    let origin = Point::new(0.0, 0.0).unwrap();
    let east = Point::new(0.0, 1.0).unwrap();
    let strategy = Spherical::with_radius(EARTH_RADIUS_METERS * 2.0).unwrap();

    let doubled = strategy.geodesic_distance(origin, east).unwrap().meters();
    let baseline = Spherical::default().geodesic_distance(origin, east).unwrap().meters();

    assert!((doubled - baseline * 2.0).abs() < 1e-6);
  }

  #[test]
  fn constructs_from_ellipsoid() {
    let ellipsoid = Ellipsoid::wgs84();
    let strategy = Spherical::from_ellipsoid(ellipsoid).unwrap();
    assert!(strategy.radius_meters() > 6_300_000.0);
  }
}
