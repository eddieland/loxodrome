//! Pluggable algorithm abstractions for geodesic kernels.
//!
//! The default strategy uses a spherical great-circle model, but consumers can
//! supply their own implementations while reusing the higher-level APIs.

use crate::{Distance, GeodistError, Point};

mod geographiclib;
mod spherical;

pub use geographiclib::Geographiclib;
pub use spherical::Spherical;

/// Strategy for computing geodesic distance between two points.
///
/// Implementations take latitude/longitude in degrees and return meter
/// distances. This trait stays minimal to remain FFI-friendly and to keep
/// algorithm swaps lightweight. Implementors are expected to perform input
/// validation where necessary and surface [`GeodistError`] when coordinates or
/// intermediate values are invalid.
pub trait GeodesicAlgorithm {
  /// Compute geodesic distance between two points in degrees.
  ///
  /// Returns a [`Distance`] in meters using the strategy's chosen model.
  ///
  /// # Errors
  ///
  /// Implementations should return [`GeodistError`] when validation fails or
  /// when the calculation cannot be completed.
  fn geodesic_distance(&self, p1: Point, p2: Point) -> Result<Distance, GeodistError>;

  /// Compute distances for multiple point pairs using the same strategy.
  ///
  /// Accepts `(origin, destination)` tuples in degrees and returns a `Vec` of
  /// meters in the same order. The default implementation iterates sequentially
  /// and short-circuits on the first error.
  ///
  /// # Errors
  ///
  /// Propagates the first [`GeodistError`] reported by
  /// [`Self::geodesic_distance`].
  fn geodesic_distances(&self, pairs: &[(Point, Point)]) -> Result<Vec<f64>, GeodistError> {
    pairs
      .iter()
      .map(|(a, b)| self.geodesic_distance(*a, *b).map(|d| d.meters()))
      .collect()
  }
}
