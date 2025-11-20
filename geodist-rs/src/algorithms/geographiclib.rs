use std::fmt;

use geographiclib_rs::{Geodesic as GeographicGeodesic, InverseGeodesic};

use super::GeodesicAlgorithm;
use crate::{Distance, Ellipsoid, GeodistError, Point};

/// Ellipsoidal geodesic solver backed by `geographiclib-rs`.
///
/// This adapter keeps the public trait surface small while delegating the
/// heavy lifting to Karney's formulation provided by the external crate.
pub struct Geographiclib {
  geodesic: GeographicGeodesic,
}

impl Geographiclib {
  /// Construct a solver from an ellipsoid description.
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError::InvalidEllipsoid`] when the axes are not finite,
  /// positive, and ordered (semi-major >= semi-minor).
  pub fn from_ellipsoid(ellipsoid: Ellipsoid) -> Result<Self, GeodistError> {
    ellipsoid.validate()?;
    let flattening = 1.0 - (ellipsoid.semi_minor_axis_m / ellipsoid.semi_major_axis_m);
    let geodesic = GeographicGeodesic::new(ellipsoid.semi_major_axis_m, flattening);

    Ok(Self { geodesic })
  }

  fn distance_m(&self, p1: Point, p2: Point) -> Result<f64, GeodistError> {
    p1.validate()?;
    p2.validate()?;

    let meters = self.geodesic.inverse(p1.lat, p1.lon, p2.lat, p2.lon);
    Ok(meters)
  }

  /// Compute distance and bearings using the configured ellipsoid.
  ///
  /// Bearings are normalized to `[0, 360)` to match the spherical variants.
  ///
  /// # Errors
  ///
  /// Propagates validation and distance construction errors.
  pub fn geodesic_with_bearings(&self, p1: Point, p2: Point) -> Result<(Distance, f64, f64), GeodistError> {
    p1.validate()?;
    p2.validate()?;

    let (meters, azi1, azi2, _arc_degrees) = self.geodesic.inverse(p1.lat, p1.lon, p2.lat, p2.lon);
    let distance = Distance::from_meters(meters)?;

    if meters == 0.0 {
      return Ok((distance, 0.0, 0.0));
    }

    let initial = normalize_bearing(azi1);
    let final_bearing = normalize_bearing(azi2);

    Ok((distance, initial, final_bearing))
  }
}

impl GeodesicAlgorithm for Geographiclib {
  fn geodesic_distance(&self, p1: Point, p2: Point) -> Result<Distance, GeodistError> {
    let meters = self.distance_m(p1, p2)?;
    Distance::from_meters(meters)
  }
}

impl fmt::Debug for Geographiclib {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Geographiclib")
      .field("semi_major_axis_m", &self.geodesic.a)
      .field("flattening", &self.geodesic.f)
      .finish()
  }
}

fn normalize_bearing(mut degrees: f64) -> f64 {
  degrees %= 360.0;
  if degrees < 0.0 {
    degrees += 360.0;
  }
  degrees
}
