"""Minimal Python surface for the geodist Rust kernels.

Exports the constant, error types, and Rust-backed geometry wrappers. Keep this
module's public API aligned with the compiled extension. Optional Shapely
interop helpers live in `geodist.ext.shapely`.
"""

from __future__ import annotations

from typing import Final

from . import _geodist_rs
from .errors import GeodistError, InvalidGeometryError
from .geometry import BoundingBox, Point, Point3D
from .ops import (
    GeodesicResult,
    geodesic_distance,
    geodesic_distance_3d,
    geodesic_with_bearings,
    hausdorff,
    hausdorff_clipped,
    hausdorff_directed,
    hausdorff_directed_clipped,
)

EARTH_RADIUS_METERS: Final[float] = float(_geodist_rs.EARTH_RADIUS_METERS)

__all__ = (
    "EARTH_RADIUS_METERS",
    "GeodistError",
    "InvalidGeometryError",
    "BoundingBox",
    "Point",
    "Point3D",
    "GeodesicResult",
    "geodesic_distance",
    "geodesic_distance_3d",
    "geodesic_with_bearings",
    "hausdorff",
    "hausdorff_clipped",
    "hausdorff_directed",
    "hausdorff_directed_clipped",
)
