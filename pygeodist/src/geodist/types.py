"""Geodesic typing primitives shared across geodist bindings.

Angles are expressed in degrees and distances in meters to match the Rust kernels.
"""

from __future__ import annotations

Latitude = float
Longitude = float
Meters = float
AltitudeM = float

# Geographic point represented as (lat, lon) in degrees.
Point = tuple[Latitude, Longitude]
# Geographic point represented as (lat, lon, altitude_m) in degrees/meters.
Point3D = tuple[Latitude, Longitude, AltitudeM]

# Bounding box encoded as (min_lat, max_lat, min_lon, max_lon), degrees.
BoundingBox = tuple[Latitude, Latitude, Longitude, Longitude]

__all__ = (
    "AltitudeM",
    "Latitude",
    "Longitude",
    "Meters",
    "Point",
    "Point3D",
    "BoundingBox",
)
