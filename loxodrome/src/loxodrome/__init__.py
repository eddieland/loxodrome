"""Minimal Python surface for the loxodrome Rust kernels.

Exports the constant, error types, and Rust-backed geometry wrappers. Keep this
module's public API aligned with the compiled extension. Optional Shapely
interop helpers live in `loxodrome.ext.shapely`.
"""

from __future__ import annotations

from typing import Final

from . import _loxodrome_rs
from .errors import (
    EmptyPointSetError,
    GeodistError,
    InvalidAltitudeError,
    InvalidBoundingBoxError,
    InvalidDistanceError,
    InvalidEllipsoidError,
    InvalidGeometryError,
    InvalidLatitudeError,
    InvalidLongitudeError,
    InvalidRadiusError,
)
from .geometry import BoundingBox, Ellipsoid, LineString, Point, Point3D, Polygon
from .ops import (
    GeodesicResult,
    HausdorffDirectedWitness,
    HausdorffWitness,
    geodesic_distance,
    geodesic_distance_3d,
    geodesic_distance_on_ellipsoid,
    geodesic_with_bearings,
    geodesic_with_bearings_on_ellipsoid,
    hausdorff,
    hausdorff_3d,
    hausdorff_clipped,
    hausdorff_clipped_3d,
    hausdorff_directed,
    hausdorff_directed_3d,
    hausdorff_directed_clipped,
    hausdorff_directed_clipped_3d,
    hausdorff_polygon_boundary,
)

EARTH_RADIUS_METERS: Final[float] = float(_loxodrome_rs.EARTH_RADIUS_METERS)

__all__ = (
    "EARTH_RADIUS_METERS",
    "GeodistError",
    "InvalidGeometryError",
    "InvalidLatitudeError",
    "InvalidLongitudeError",
    "InvalidAltitudeError",
    "InvalidDistanceError",
    "InvalidRadiusError",
    "InvalidEllipsoidError",
    "InvalidBoundingBoxError",
    "EmptyPointSetError",
    "BoundingBox",
    "Ellipsoid",
    "LineString",
    "Point",
    "Point3D",
    "Polygon",
    "GeodesicResult",
    "HausdorffDirectedWitness",
    "HausdorffWitness",
    "geodesic_distance",
    "geodesic_distance_on_ellipsoid",
    "geodesic_distance_3d",
    "geodesic_with_bearings",
    "geodesic_with_bearings_on_ellipsoid",
    "hausdorff",
    "hausdorff_3d",
    "hausdorff_clipped",
    "hausdorff_clipped_3d",
    "hausdorff_directed",
    "hausdorff_directed_3d",
    "hausdorff_directed_clipped",
    "hausdorff_directed_clipped_3d",
    "hausdorff_polygon_boundary",
)
