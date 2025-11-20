"""Minimal Python surface for the geodist Rust kernels.

Exports the constant, error types, and Rust-backed geometry wrappers. Keep this
module's public API aligned with the compiled extension. Optional Shapely
interop helpers live in `geodist.ext.shapely`.
"""

from __future__ import annotations

from typing import Final

from . import _geodist_rs
from .errors import GeodistError, InvalidGeometryError
from .geometry import Point

EARTH_RADIUS_METERS: Final[float] = float(_geodist_rs.EARTH_RADIUS_METERS)

__all__ = (
    "EARTH_RADIUS_METERS",
    "GeodistError",
    "InvalidGeometryError",
    "Point",
)
