"""Geodesic typing primitives shared across geodist bindings.

Angles are expressed in degrees and distances in meters to match the Rust kernels.
"""

from __future__ import annotations

LatitudeDeg = float
LongitudeDeg = float
Meters = float
AltitudeM = float

# Geographic point represented as (lat_deg, lon_deg).
PointDeg = tuple[LatitudeDeg, LongitudeDeg]
# Geographic point represented as (lat_deg, lon_deg, altitude_m).
Point3DDeg = tuple[LatitudeDeg, LongitudeDeg, AltitudeM]

# Bounding box encoded as (min_lat_deg, max_lat_deg, min_lon_deg, max_lon_deg).
BoundingBoxDeg = tuple[LatitudeDeg, LatitudeDeg, LongitudeDeg, LongitudeDeg]

__all__ = (
    "AltitudeM",
    "LatitudeDeg",
    "LongitudeDeg",
    "Meters",
    "PointDeg",
    "Point3DDeg",
    "BoundingBoxDeg",
)
