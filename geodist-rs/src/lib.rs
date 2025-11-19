//! Public surface for geodist kernels and types.

#[cfg(feature = "python")]
mod python;

mod algorithms;
mod constants;
mod distance;
mod hausdorff;
mod types;

pub use algorithms::{GeodesicAlgorithm, Spherical};
pub use constants::EARTH_RADIUS_METERS;
pub use distance::{
  GeodesicSolution, geodesic_distance, geodesic_distance_on_ellipsoid, geodesic_distance_on_radius, geodesic_distances,
  geodesic_with_bearings, geodesic_with_bearings_on_ellipsoid, geodesic_with_bearings_on_radius,
};
pub use hausdorff::{hausdorff, hausdorff_clipped, hausdorff_directed, hausdorff_directed_clipped};
pub use types::{BoundingBox, Distance, Ellipsoid, GeodistError, Point};
