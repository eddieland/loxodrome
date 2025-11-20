"""Geodesic typing primitives shared across geodist bindings.

Angles are expressed in degrees and distances in meters to match the Rust kernels.
"""

from __future__ import annotations

LatitudeDegrees = float
LongitudeDegrees = float
Meters = float
AltitudeMeters = float

# Geographic point represented as (latitude_degrees, longitude_degrees).
PointDegrees = tuple[LatitudeDegrees, LongitudeDegrees]
# Geographic point represented as (latitude_degrees, longitude_degrees, altitude_meters).
Point3DDegrees = tuple[LatitudeDegrees, LongitudeDegrees, AltitudeMeters]

# Bounding box encoded as (min_latitude, max_latitude, min_longitude, max_longitude).
BoundingBoxDegrees = tuple[LatitudeDegrees, LatitudeDegrees, LongitudeDegrees, LongitudeDegrees]

__all__ = (
    "AltitudeMeters",
    "BoundingBoxDegrees",
    "LatitudeDegrees",
    "LongitudeDegrees",
    "Meters",
    "PointDegrees",
    "Point3DDegrees",
)
