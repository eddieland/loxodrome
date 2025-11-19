//! Core types and validation helpers for geodist.
//!
//! - Angles are expressed in **degrees**.
//! - Distances are expressed in **meters**.
//! - Public constructors validate inputs and return [`GeodistError`] on
//!   failure.
//!
//! Layouts stay simple (`#[repr(C)]`) to ease future FFI bindings.

use std::fmt;

mod constants;
mod distance;
mod hausdorff;

pub use constants::EARTH_RADIUS_METERS;
pub use distance::geodesic_distance;
pub use hausdorff::{hausdorff, hausdorff_directed};

use crate::constants::{MAX_LAT_DEGREES, MAX_LON_DEGREES, MIN_LAT_DEGREES, MIN_LON_DEGREES};

/// Error type for invalid input or derived values.
///
/// The variants carry the offending value to simplify debugging and FFI error
/// mapping without additional allocation.
#[derive(Debug, Clone, PartialEq)]
pub enum GeodistError {
  /// Latitude must be within `[-90.0, 90.0]` degrees and finite.
  InvalidLatitude(f64),
  /// Longitude must be within `[-180.0, 180.0]` degrees and finite.
  InvalidLongitude(f64),
  /// Distances must be finite and non-negative.
  InvalidDistance(f64),
  /// Point sets must be non-empty for Hausdorff distance.
  EmptyPointSet,
}

impl fmt::Display for GeodistError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidLatitude(value) => {
        write!(f, "invalid latitude {value}; expected finite degrees in [-90, 90]")
      }
      Self::InvalidLongitude(value) => {
        write!(f, "invalid longitude {value}; expected finite degrees in [-180, 180]")
      }
      Self::InvalidDistance(value) => {
        write!(f, "invalid distance {value}; expected finite meters >= 0")
      }
      Self::EmptyPointSet => write!(f, "point sets must be non-empty"),
    }
  }
}

impl std::error::Error for GeodistError {}

/// Geographic position in degrees.
///
/// The struct uses `#[repr(C)]` to keep the layout predictable for future FFI
/// bindings. Construction validates latitude and longitude bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point {
  /// Latitude in degrees, expected in `[-90.0, 90.0]`.
  pub latitude: f64,
  /// Longitude in degrees, expected in `[-180.0, 180.0]`.
  pub longitude: f64,
}

impl Point {
  /// Construct a validated point from latitude/longitude in degrees.
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError::InvalidLatitude`] or
  /// [`GeodistError::InvalidLongitude`] when a coordinate is out of range or
  /// non-finite.
  pub fn new(latitude: f64, longitude: f64) -> Result<Self, GeodistError> {
    validate_latitude(latitude)?;
    validate_longitude(longitude)?;
    Ok(Self { latitude, longitude })
  }

  /// Validate the current point's coordinates.
  ///
  /// Use this when a point was constructed externally (e.g., via FFI) and
  /// should be checked before use.
  pub fn validate(&self) -> Result<(), GeodistError> {
    validate_latitude(self.latitude)?;
    validate_longitude(self.longitude)?;
    Ok(())
  }
}

/// Distance measurement in meters.
///
/// `Distance` is deliberately thin for FFI-friendliness and future extensions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Distance {
  /// Stored meter value (kept private to enforce invariants).
  meters: f64,
}

impl Distance {
  /// Construct a distance from meters, validating that the value is finite
  /// and non-negative.
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError::InvalidDistance`] when the value is NaN, infinite,
  /// or negative.
  pub fn from_meters(meters: f64) -> Result<Self, GeodistError> {
    validate_distance(meters)?;
    Ok(Self { meters })
  }

  /// Raw meter value.
  pub fn meters(&self) -> f64 {
    self.meters
  }
}

/// Validate that latitude is finite and inside `[-90, 90]` degrees.
fn validate_latitude(value: f64) -> Result<(), GeodistError> {
  if !value.is_finite() || !(MIN_LAT_DEGREES..=MAX_LAT_DEGREES).contains(&value) {
    return Err(GeodistError::InvalidLatitude(value));
  }
  Ok(())
}

/// Validate that longitude is finite and inside `[-180, 180]` degrees.
fn validate_longitude(value: f64) -> Result<(), GeodistError> {
  if !value.is_finite() || !(MIN_LON_DEGREES..=MAX_LON_DEGREES).contains(&value) {
    return Err(GeodistError::InvalidLongitude(value));
  }
  Ok(())
}

/// Validate that a distance is finite and non-negative.
fn validate_distance(value: f64) -> Result<(), GeodistError> {
  if !value.is_finite() || value < 0.0 {
    return Err(GeodistError::InvalidDistance(value));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn point_new_accepts_valid_bounds() {
    let p = Point::new(45.0, 120.0).unwrap();
    assert_eq!(p.latitude, 45.0);
    assert_eq!(p.longitude, 120.0);
  }

  #[test]
  fn point_new_rejects_invalid_latitude() {
    assert!(matches!(
      Point::new(100.0, 0.0),
      Err(GeodistError::InvalidLatitude(100.0))
    ));
    assert!(matches!(
        Point::new(f64::NAN, 0.0),
        Err(GeodistError::InvalidLatitude(v)) if v.is_nan()
    ));
  }

  #[test]
  fn point_new_rejects_invalid_longitude() {
    assert!(matches!(
      Point::new(0.0, 200.0),
      Err(GeodistError::InvalidLongitude(200.0))
    ));
    assert!(matches!(
        Point::new(0.0, f64::INFINITY),
        Err(GeodistError::InvalidLongitude(v)) if v.is_infinite()
    ));
  }

  #[test]
  fn distance_validation_accepts_non_negative_finite() {
    let d = Distance::from_meters(1.5).unwrap();
    assert_eq!(d.meters(), 1.5);
  }

  #[test]
  fn distance_validation_rejects_negative_or_non_finite() {
    assert!(matches!(
      Distance::from_meters(-1.0),
      Err(GeodistError::InvalidDistance(-1.0))
    ));
    assert!(matches!(
        Distance::from_meters(f64::NAN),
        Err(GeodistError::InvalidDistance(v)) if v.is_nan()
    ));
  }
}
