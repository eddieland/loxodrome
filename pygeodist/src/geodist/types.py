"""Geodesic typing primitives shared across geodist bindings.

Angles are expressed in degrees and distances in meters to match the Rust kernels.
"""

from __future__ import annotations

from typing import TypeAlias

Latitude: TypeAlias = float
Longitude: TypeAlias = float
Meters: TypeAlias = float
AltitudeM = float

# Geographic point represented as (lat, lon) in degrees.
Point: TypeAlias = tuple[Latitude, Longitude]
# Geographic point represented as (lat, lon, altitude_m) in degrees/meters.
Point3D: TypeAlias = tuple[Latitude, Longitude, AltitudeM]

# Bounding box encoded as (min_lat, max_lat, min_lon, max_lon), degrees.
BoundingBox: TypeAlias = tuple[Latitude, Latitude, Longitude, Longitude]

__all__ = (
    "AltitudeM",
    "Latitude",
    "Longitude",
    "Meters",
    "Point",
    "Point3D",
    "BoundingBox",
)
