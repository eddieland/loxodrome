//! Public surface for geodist kernels and types.

// Enable Clippy lints.
#![warn(
    // --- Panic & unwrap safety ---
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::todo,
    clippy::unimplemented,

    // --- Result / Option correctness ---
    clippy::map_err_ignore,
    clippy::bind_instead_of_map,
    clippy::map_flatten,

    // --- Logging & debugging hygiene ---
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::dbg_macro,

    // --- Correctness & readability ---
    clippy::inefficient_to_string,
    clippy::shadow_unrelated,
    clippy::match_bool,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::useless_conversion,
    clippy::undocumented_unsafe_blocks,

    // --- API Hygiene ---
    clippy::missing_const_for_fn,
    clippy::redundant_pub_crate,
    clippy::pub_underscore_fields,
    clippy::pub_without_shorthand
)]

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
  GeodesicSolution, geodesic_distance, geodesic_distance_3d, geodesic_distance_3d_on_ellipsoid,
  geodesic_distance_on_ellipsoid, geodesic_distance_on_radius, geodesic_distances, geodesic_with_bearings,
  geodesic_with_bearings_on_ellipsoid, geodesic_with_bearings_on_radius,
};
pub use hausdorff::{
  HausdorffDirectedWitness, HausdorffWitness, hausdorff, hausdorff_3d, hausdorff_3d_on_ellipsoid, hausdorff_clipped,
  hausdorff_clipped_3d, hausdorff_clipped_3d_on_ellipsoid, hausdorff_directed, hausdorff_directed_3d,
  hausdorff_directed_3d_on_ellipsoid, hausdorff_directed_clipped, hausdorff_directed_clipped_3d,
  hausdorff_directed_clipped_3d_on_ellipsoid,
};
pub use types::{BoundingBox, Distance, Ellipsoid, GeodistError, Point, Point3D};
