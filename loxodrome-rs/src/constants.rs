//! Shared constants for geographic kernels and validation.
//!
//! Values are kept in one place to preserve consistency across calculations
//! and future bindings.

/// Mean radius of Earth in meters (WGS84).
pub const EARTH_RADIUS_METERS: f64 = 6_371_008.8;
/// WGS84 semi-major axis in meters.
pub const WGS84_SEMI_MAJOR_METERS: f64 = 6_378_137.0;
/// WGS84 semi-minor axis in meters.
pub const WGS84_SEMI_MINOR_METERS: f64 = 6_356_752.314_245;

/// Minimum allowed latitude in degrees.
pub const MIN_LAT_DEGREES: f64 = -90.0;
/// Maximum allowed latitude in degrees.
pub const MAX_LAT_DEGREES: f64 = 90.0;
/// Minimum allowed longitude in degrees.
pub const MIN_LON_DEGREES: f64 = -180.0;
/// Maximum allowed longitude in degrees.
pub const MAX_LON_DEGREES: f64 = 180.0;
