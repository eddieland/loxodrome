//! Core types and validation helpers for loxodrome.
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
  /// Bounding boxes must have ordered latitudes within valid ranges. Longitudes
  /// may wrap the antimeridian (`min_lon > max_lon`).
  InvalidBoundingBox {
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
  },
  /// Polygon rings must follow orientation conventions (CCW exterior, CW
  /// holes).
  InvalidRingOrientation {
    part_index: Option<usize>,
    expected: RingOrientation,
    got: RingOrientation,
  },
  /// Point sets must be non-empty for Hausdorff distance.
  EmptyPointSet,
  /// Polyline inputs must provide at least one densification knob.
  MissingDensificationKnob,
  /// Polyline parts must contain at least two distinct vertices after
  /// collapsing duplicates.
  DegeneratePolyline { part_index: Option<usize> },
  /// Vertex validation failed with part/vertex index context.
  InvalidVertex {
    part_index: Option<usize>,
    vertex_index: usize,
    error: VertexValidationError,
  },
  /// Densification predicted more samples than the configured cap.
  SampleCapExceeded {
    expected: usize,
    cap: usize,
    part_index: Option<usize>,
  },
}

/// Vertex-level validation errors used by polyline validators.
#[derive(Debug, Clone, PartialEq)]
pub enum VertexValidationError {
  /// Latitude must lie within `[-90.0, 90.0]` and be finite.
  Latitude(f64),
  /// Longitude must lie within `[-180.0, 180.0]` and be finite.
  Longitude(f64),
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
        min_lat,
        max_lat,
        min_lon,
        max_lon,
      } => write!(
        f,
        "invalid bounding box [{min_lat}, {max_lat}] x [{min_lon}, {max_lon}]; expected finite degrees with min_lat <= max_lat and longitudes in [{MIN_LON_DEGREES}, {MAX_LON_DEGREES}]"
      ),
      Self::InvalidRingOrientation {
        part_index,
        expected,
        got,
      } => {
        let ring_label = match part_index {
          Some(index) => format!("ring {index}"),
          None => "exterior ring".to_string(),
        };
        write!(
          f,
          "{ring_label} has wrong orientation; expected {expected} but got {got}"
        )
      }
      Self::EmptyPointSet => write!(f, "point sets must be non-empty"),
      Self::MissingDensificationKnob => write!(
        f,
        "polyline densification requires at least one knob: set max_segment_length_m (default 100 m) and/or max_segment_angle_deg (default 0.1 deg)"
      ),
      Self::DegeneratePolyline { part_index } => match part_index {
        Some(index) => write!(f, "polyline part {index} must contain at least two distinct vertices"),
        None => write!(f, "polyline must contain at least two distinct vertices"),
      },
      Self::InvalidVertex {
        part_index,
        vertex_index,
        error,
      } => {
        let prefix = match part_index {
          Some(part) => format!("vertex {vertex_index} in part {part}"),
          None => format!("vertex {vertex_index}"),
        };

        match error {
          VertexValidationError::Latitude(value) => write!(
            f,
            "{prefix} has invalid latitude {value}; expected finite degrees in [{MIN_LAT_DEGREES}, {MAX_LAT_DEGREES}]"
          ),
          VertexValidationError::Longitude(value) => write!(
            f,
            "{prefix} has invalid longitude {value}; expected finite degrees in [{MIN_LON_DEGREES}, {MAX_LON_DEGREES}]"
          ),
        }
      }
      Self::SampleCapExceeded {
        expected,
        cap,
        part_index,
      } => match part_index {
        Some(index) => write!(
          f,
          "densification exceeds sample_cap={cap}; expected {expected} samples (part {index})"
        ),
        None => write!(f, "densification exceeds sample_cap={cap}; expected {expected} samples"),
      },
    }
  }
}

/// Ring orientation helpers for polygon validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingOrientation {
  Clockwise,
  CounterClockwise,
}

impl fmt::Display for RingOrientation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Clockwise => write!(f, "clockwise"),
      Self::CounterClockwise => write!(f, "counterclockwise"),
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
  pub lat: f64,
  /// Longitude in degrees, expected in `[-180.0, 180.0]`.
  pub lon: f64,
}

impl Point {
  /// Construct a validated point from latitude/longitude in degrees.
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError::InvalidLatitude`] or
  /// [`GeodistError::InvalidLongitude`] when a coordinate is out of range or
  /// non-finite.
  pub fn new(lat: f64, lon: f64) -> Result<Self, GeodistError> {
    validate_latitude(lat)?;
    validate_longitude(lon)?;
    Ok(Self { lat, lon })
  }

  /// Construct a point without performing validation.
  ///
  /// Caller is responsible for ensuring `lat` is in `[-90.0, 90.0]`,
  /// `lon` is in `[-180.0, 180.0]`, and both are finite. Invalid inputs
  /// skip validation and may yield incorrect downstream calculations.
  pub const fn new_unchecked(lat: f64, lon: f64) -> Self {
    Self { lat, lon }
  }

  /// Validate the current point's coordinates.
  ///
  /// Use this when a point was constructed externally (e.g., via FFI) and
  /// should be checked before use.
  pub fn validate(&self) -> Result<(), GeodistError> {
    validate_latitude(self.lat)?;
    validate_longitude(self.lon)?;
    Ok(())
  }
}

/// Geographic position with altitude in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Point3D {
  /// Latitude in degrees, expected in `[-90.0, 90.0]`.
  pub lat: f64,
  /// Longitude in degrees, expected in `[-180.0, 180.0]`.
  pub lon: f64,
  /// Altitude in meters relative to the reference ellipsoid; must be finite.
  pub altitude_m: f64,
}

impl Point3D {
  /// Construct a validated 3D point from latitude/longitude (degrees) and
  /// altitude (meters).
  ///
  /// # Errors
  ///
  /// Returns [`GeodistError`] when any component is out of range or non-finite.
  pub fn new(lat: f64, lon: f64, altitude_m: f64) -> Result<Self, GeodistError> {
    validate_latitude(lat)?;
    validate_longitude(lon)?;
    validate_altitude(altitude_m)?;
    Ok(Self { lat, lon, altitude_m })
  }

  /// Construct a 3D point without performing validation.
  ///
  /// Caller must ensure latitude/longitude follow the same constraints as
  /// [`Point`] and altitude is finite. Invalid inputs skip validation and may
  /// lead to incorrect downstream calculations.
  pub const fn new_unchecked(lat: f64, lon: f64, altitude_m: f64) -> Self {
    Self { lat, lon, altitude_m }
  }

  /// Validate the current point's coordinates and altitude.
  ///
  /// Use this to verify externally-constructed points (e.g., from FFI).
  pub fn validate(&self) -> Result<(), GeodistError> {
    validate_latitude(self.lat)?;
    validate_longitude(self.lon)?;
    validate_altitude(self.altitude_m)?;
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
  pub const fn meters(&self) -> f64 {
    self.meters
  }

  /// Construct a distance without performing validation.
  ///
  /// # Safety
  ///
  /// Caller must ensure `meters` is finite and non-negative. Supplying invalid
  /// values may lead to incorrect calculations downstream.
  pub const fn from_meters_unchecked(meters: f64) -> Self {
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
  pub semi_major_axis_m: f64,
  /// Semi-minor axis (polar radius) in meters.
  pub semi_minor_axis_m: f64,
}

impl Ellipsoid {
  /// Construct a validated ellipsoid.
  pub fn new(semi_major_axis_m: f64, semi_minor_axis_m: f64) -> Result<Self, GeodistError> {
    validate_ellipsoid(semi_major_axis_m, semi_minor_axis_m)?;
    Ok(Self {
      semi_major_axis_m,
      semi_minor_axis_m,
    })
  }

  /// WGS84 ellipsoid parameters in meters.
  pub const fn wgs84() -> Self {
    Self {
      semi_major_axis_m: crate::constants::WGS84_SEMI_MAJOR_METERS,
      semi_minor_axis_m: crate::constants::WGS84_SEMI_MINOR_METERS,
    }
  }

  /// Mean radius derived from the ellipsoid (2a + b) / 3.
  pub fn mean_radius(&self) -> Result<f64, GeodistError> {
    validate_ellipsoid(self.semi_major_axis_m, self.semi_minor_axis_m)?;
    Ok((2.0 * self.semi_major_axis_m + self.semi_minor_axis_m) / 3.0)
  }

  /// Validate the ellipsoid axes.
  pub fn validate(&self) -> Result<(), GeodistError> {
    validate_ellipsoid(self.semi_major_axis_m, self.semi_minor_axis_m)
  }
}

/// Geographic bounding box used to filter point sets.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct BoundingBox {
  /// Minimum latitude in degrees.
  pub min_lat: f64,
  /// Maximum latitude in degrees.
  pub max_lat: f64,
  /// Minimum longitude in degrees.
  pub min_lon: f64,
  /// Maximum longitude in degrees.
  pub max_lon: f64,
}

impl BoundingBox {
  /// Construct a bounding box ensuring latitudes are ordered inside valid
  /// ranges.
  pub fn new(min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Result<Self, GeodistError> {
    validate_latitude(min_lat)?;
    validate_latitude(max_lat)?;
    validate_longitude(min_lon)?;
    validate_longitude(max_lon)?;

    if min_lat > max_lat {
      return Err(GeodistError::InvalidBoundingBox {
        min_lat,
        max_lat,
        min_lon,
        max_lon,
      });
    }

    Ok(Self {
      min_lat,
      max_lat,
      min_lon,
      max_lon,
    })
  }

  /// Check whether a point lies inside the box (inclusive of edges).
  pub fn contains(&self, point: &Point) -> bool {
    self.contains_lat(point.lat) && self.contains_lon(point.lon)
  }

  /// Check whether a 3D point lies inside the box using latitude/longitude
  /// only.
  pub fn contains_3d(&self, point: &Point3D) -> bool {
    self.contains_lat(point.lat) && self.contains_lon(point.lon)
  }

  /// Return true when the bounding box crosses the antimeridian.
  pub const fn wraps_antimeridian(&self) -> bool {
    self.min_lon > self.max_lon
  }

  fn contains_lat(&self, lat: f64) -> bool {
    lat >= self.min_lat && lat <= self.max_lat
  }

  fn contains_lon(&self, lon: f64) -> bool {
    if !self.wraps_antimeridian() {
      lon >= self.min_lon && lon <= self.max_lon
    } else {
      lon >= self.min_lon || lon <= self.max_lon
    }
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
const fn validate_altitude(value: f64) -> Result<(), GeodistError> {
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
    assert_eq!(p.lat, 45.0);
    assert_eq!(p.lon, 120.0);
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
    let p = Point::new_unchecked(120.0, 200.0);
    assert_eq!(p.lat, 120.0);
    assert_eq!(p.lon, 200.0);
    assert!(p.validate().is_err());
  }

  #[test]
  fn point3d_accepts_finite_altitude() {
    let p = Point3D::new(10.0, 20.0, 500.0).unwrap();
    assert_eq!(p.altitude_m, 500.0);
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
    let d = Distance::from_meters_unchecked(f64::NAN);
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
  fn bounding_box_accepts_antimeridian_wrap() {
    let bbox = BoundingBox::new(-10.0, 10.0, 170.0, -170.0).unwrap();
    assert!(bbox.wraps_antimeridian());
    assert!(bbox.contains(&Point::new(0.0, 175.0).unwrap()));
    assert!(bbox.contains(&Point::new(0.0, -175.0).unwrap()));
    assert!(!bbox.contains(&Point::new(0.0, -90.0).unwrap()));
  }

  #[test]
  fn bounding_box_checks_point3d_coordinates() {
    let bbox = BoundingBox::new(-1.0, 1.0, -2.0, 2.0).unwrap();
    let inside = Point3D::new(0.0, 0.0, 250.0).unwrap();
    let outside = Point3D::new(10.0, 0.0, 0.0).unwrap();

    assert!(bbox.contains_3d(&inside));
    assert!(!bbox.contains_3d(&outside));
  }

  #[test]
  fn bounding_box_rejects_unordered_ranges() {
    let result = BoundingBox::new(1.0, -1.0, 0.0, 1.0);
    assert!(matches!(
      result,
      Err(GeodistError::InvalidBoundingBox {
        min_lat: 1.0,
        max_lat: -1.0,
        min_lon: 0.0,
        max_lon: 1.0,
      })
    ));
  }
}
