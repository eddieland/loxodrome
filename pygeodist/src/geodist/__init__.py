"""Python bindings for the geodist Rust library."""

from __future__ import annotations

from . import _geodist_rs
from .errors import (
    CRSValidationError,
    GeodistError,
    GeometryTypeError,
    InvalidGeometryError,
    KernelUnavailableError,
    VectorizationError,
)
from .geometry import Geometry, GeometryCollection, LineString, Point, Polygon
from .io import dumps_geojson, dumps_wkb, dumps_wkt, loads_geojson, loads_wkb, loads_wkt
from .ops import buffer, centroid, distance, equals, intersects, within
from .vectorized import distance_many, equals_many, intersects_many, within_many

EARTH_RADIUS_METERS = _geodist_rs.EARTH_RADIUS_METERS

__all__ = (
    "CRSValidationError",
    "EARTH_RADIUS_METERS",
    "GeodistError",
    "Geometry",
    "GeometryCollection",
    "GeometryTypeError",
    "InvalidGeometryError",
    "KernelUnavailableError",
    "LineString",
    "Point",
    "Polygon",
    "VectorizationError",
    "buffer",
    "centroid",
    "distance",
    "distance_many",
    "dumps_geojson",
    "dumps_wkb",
    "dumps_wkt",
    "equals",
    "equals_many",
    "intersects",
    "intersects_many",
    "loads_geojson",
    "loads_wkb",
    "loads_wkt",
    "within",
    "within_many",
)
