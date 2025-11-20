//! Core types and validation helpers for geodist.
//!
//! - Angles are expressed in **degrees**.
//! - Distances are expressed in **meters**.
//! - Public constructors validate inputs and return [`GeodistError`] on
//!   failure.
//!
//! Layouts stay simple (`#[repr(C)]`) to ease future FFI bindings.

use std::fmt;

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
  /// Altitude must be finite (meters above/below reference ellipsoid surface).
  InvalidAltitude(f64),
  /// Distances must be finite and non-negative.
  InvalidDistance(f64),
  /// Radii must be finite and strictly positive.
  InvalidRadius(f64),
  /// Ellipsoid axes must be finite, positive, and ordered (semi-major >=
  /// semi-minor).
  InvalidEllipsoid { semi_major: f64, semi_minor: f64 },
  /// Bounding boxes must have ordered corners within valid latitude/longitude
  /// ranges.
  InvalidBoundingBox {
    min_latitude: f64,
    max_latitude: f64,
    min_longitude: f64,
    max_longitude: f64,
  },
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
      Self::InvalidAltitude(value) => write!(f, "invalid altitude {value}; expected finite meters"),
      Self::InvalidDistance(value) => write!(f, "invalid distance {value}; expected finite meters >= 0"),
      Self::InvalidRadius(value) => write!(f, "invalid radius {value}; expected finite meters > 0"),
      Self::InvalidEllipsoid { semi_major, semi_minor } => write!(
        f,
        "invalid ellipsoid axes a={semi_major}, b={semi_minor}; expected finite meters with a >= b > 0"
      ),
      Self::InvalidBoundingBox {
        min_latitude,
        max_latitude,
        min_longitude,
        max_longitude,
      } => write!(
        f,
        "invalid bounding box [{min_latitude}, {max_latitude}] x [{min_longitude}, {max_longitude}]; expected ordered finite degrees"
      ),
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

  /// Construct a point without performing validation.
  ///
  /// # Safety
  ///
  /// Caller must ensure `latitude` is within `[-90.0, 90.0]`, `longitude`
  /// within `[-180.0, 180.0]`, and both are finite. Breaking these assumptions
  /// can lead to incorrect downstream calculations.
  pub const unsafe fn new_unchecked(latitude: f64, longitude: f64) -> Self {
    Self { latitude, longitude }
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

/// Geographic position with altitude in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point3D {
  /// Latitude in degrees, expected in `[-90.0, 90.0]`.
  pub latitude: f64,
  /// Longitude in degrees, expected in `[-180.0, 180.0]`.
  pub longitude: f64,
  /// Altitude in meters relative to the reference ellipsoid; must be finite.
  pub altitude_meters: f64,
}

impl Point3D {
  /// Construct a validated 3D point from latitude/longitude (degrees) and
  /// altitude (meters).
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError`] when any component is out of range or non-finite.
  pub fn new(latitude: f64, longitude: f64, altitude_meters: f64) -> Result<Self, GeodistError> {
    validate_latitude(latitude)?;
    validate_longitude(longitude)?;
    validate_altitude(altitude_meters)?;
    Ok(Self {
      latitude,
      longitude,
      altitude_meters,
    })
  }

  /// Construct a 3D point without performing validation.
  ///
  /// # Safety
  ///
  /// Caller must ensure latitude/longitude follow the same constraints as
  /// [`Point`] and altitude is finite. Violating these assumptions can lead
  /// to incorrect downstream calculations.
  pub const unsafe fn new_unchecked(latitude: f64, longitude: f64, altitude_meters: f64) -> Self {
    Self {
      latitude,
      longitude,
      altitude_meters,
    }
  }

  /// Validate the current point's coordinates and altitude.
  ///
  /// Use this to verify externally-constructed points (e.g., from FFI).
  pub fn validate(&self) -> Result<(), GeodistError> {
    validate_latitude(self.latitude)?;
    validate_longitude(self.longitude)?;
    validate_altitude(self.altitude_meters)?;
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

  /// Construct a distance without performing validation.
  ///
  /// # Safety
  ///
  /// Caller must ensure `meters` is finite and non-negative. Supplying invalid
  /// values may lead to incorrect calculations downstream.
  pub const unsafe fn from_meters_unchecked(meters: f64) -> Self {
    Self { meters }
  }
}

/// Oblate ellipsoid definition (semi-major/semi-minor axes).
///
/// Used to derive an equivalent mean radius for spherical approximations.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Ellipsoid {
  /// Semi-major axis (equatorial radius) in meters, must be >= semi-minor.
  pub semi_major_axis_meters: f64,
  /// Semi-minor axis (polar radius) in meters.
  pub semi_minor_axis_meters: f64,
}

impl Ellipsoid {
  /// Construct a validated ellipsoid.
  pub fn new(semi_major_axis_meters: f64, semi_minor_axis_meters: f64) -> Result<Self, GeodistError> {
    validate_ellipsoid(semi_major_axis_meters, semi_minor_axis_meters)?;
    Ok(Self {
      semi_major_axis_meters,
      semi_minor_axis_meters,
    })
  }

  /// WGS84 ellipsoid parameters in meters.
  pub fn wgs84() -> Self {
    Self {
      semi_major_axis_meters: crate::constants::WGS84_SEMI_MAJOR_METERS,
      semi_minor_axis_meters: crate::constants::WGS84_SEMI_MINOR_METERS,
    }
  }

  /// Mean radius derived from the ellipsoid (2a + b) / 3.
  pub fn mean_radius(&self) -> Result<f64, GeodistError> {
    validate_ellipsoid(self.semi_major_axis_meters, self.semi_minor_axis_meters)?;
    Ok((2.0 * self.semi_major_axis_meters + self.semi_minor_axis_meters) / 3.0)
  }

  /// Validate the ellipsoid axes.
  pub fn validate(&self) -> Result<(), GeodistError> {
    validate_ellipsoid(self.semi_major_axis_meters, self.semi_minor_axis_meters)
  }
}

/// Geographic bounding box used to filter point sets.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct BoundingBox {
  /// Minimum latitude in degrees.
  pub min_latitude: f64,
  /// Maximum latitude in degrees.
  pub max_latitude: f64,
  /// Minimum longitude in degrees.
  pub min_longitude: f64,
  /// Maximum longitude in degrees.
  pub max_longitude: f64,
}

impl BoundingBox {
  /// Construct a bounding box ensuring ordered corners inside valid ranges.
  pub fn new(
    min_latitude: f64,
    max_latitude: f64,
    min_longitude: f64,
    max_longitude: f64,
  ) -> Result<Self, GeodistError> {
    validate_latitude(min_latitude)?;
    validate_latitude(max_latitude)?;
    validate_longitude(min_longitude)?;
    validate_longitude(max_longitude)?;

    if min_latitude > max_latitude || min_longitude > max_longitude {
      return Err(GeodistError::InvalidBoundingBox {
        min_latitude,
        max_latitude,
        min_longitude,
        max_longitude,
      });
    }

    Ok(Self {
      min_latitude,
      max_latitude,
      min_longitude,
      max_longitude,
    })
  }

  /// Check whether a point lies inside the box (inclusive of edges).
  pub fn contains(&self, point: &Point) -> bool {
    point.latitude >= self.min_latitude
      && point.latitude <= self.max_latitude
      && point.longitude >= self.min_longitude
      && point.longitude <= self.max_longitude
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

/// Validate that altitude is finite (meters).
fn validate_altitude(value: f64) -> Result<(), GeodistError> {
  if !value.is_finite() {
    return Err(GeodistError::InvalidAltitude(value));
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

/// Validate a radius used for spherical approximations.
fn validate_radius(value: f64) -> Result<(), GeodistError> {
  if !value.is_finite() || value <= 0.0 {
    return Err(GeodistError::InvalidRadius(value));
  }
  Ok(())
}

/// Validate ellipsoid axes ordering and positivity.
fn validate_ellipsoid(semi_major: f64, semi_minor: f64) -> Result<(), GeodistError> {
  validate_radius(semi_major)?;
  validate_radius(semi_minor)?;
  if semi_major < semi_minor {
    return Err(GeodistError::InvalidEllipsoid { semi_major, semi_minor });
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
  fn point_new_unchecked_skips_validation() {
    let p = unsafe { Point::new_unchecked(120.0, 200.0) };
    assert_eq!(p.latitude, 120.0);
    assert_eq!(p.longitude, 200.0);
    assert!(p.validate().is_err());
  }

  #[test]
  fn point3d_accepts_finite_altitude() {
    let p = Point3D::new(10.0, 20.0, 500.0).unwrap();
    assert_eq!(p.altitude_meters, 500.0);
  }

  #[test]
  fn point3d_rejects_non_finite_altitude() {
    let result = Point3D::new(0.0, 0.0, f64::NAN);
    assert!(matches!(
        result,
        Err(GeodistError::InvalidAltitude(v)) if v.is_nan()
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

  #[test]
  fn distance_unchecked_skips_validation() {
    let d = unsafe { Distance::from_meters_unchecked(f64::NAN) };
    assert!(d.meters().is_nan());
  }

  #[test]
  fn ellipsoid_mean_radius_is_positive() {
    let ellipsoid = Ellipsoid::wgs84();
    let radius = ellipsoid.mean_radius().unwrap();
    assert!(radius > 6_300_000.0);
  }

  #[test]
  fn ellipsoid_rejects_inverted_axes() {
    let result = Ellipsoid::new(6_300_000.0, 7_000_000.0);
    assert!(matches!(
      result,
      Err(GeodistError::InvalidEllipsoid {
        semi_major: 6_300_000.0,
        semi_minor: 7_000_000.0
      })
    ));
  }

  #[test]
  fn bounding_box_accepts_ordered_ranges() {
    let bbox = BoundingBox::new(-1.0, 1.0, -2.0, 2.0).unwrap();
    let inside = Point::new(0.0, 0.0).unwrap();
    let outside = Point::new(10.0, 0.0).unwrap();
    assert!(bbox.contains(&inside));
    assert!(!bbox.contains(&outside));
  }

  #[test]
  fn bounding_box_rejects_unordered_ranges() {
    let result = BoundingBox::new(1.0, -1.0, 0.0, 1.0);
    assert!(matches!(
      result,
      Err(GeodistError::InvalidBoundingBox {
        min_latitude: 1.0,
        max_latitude: -1.0,
        min_longitude: 0.0,
        max_longitude: 1.0,
      })
    ));
  }
}
