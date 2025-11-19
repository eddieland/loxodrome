"""Shared typing primitives for the geodist Python bindings."""

from __future__ import annotations

from pathlib import Path
from typing import Mapping, Sequence, Sized

# Coordinate representations follow Shapely/pygeos conventions to ease interop.
Coordinate = tuple[float, float]
CoordinateSequence = Sequence[Coordinate]

# Opaque handle returned by the compiled Rust extension; must at least be Sized.
GeometryHandle = Sized

# Coordinate reference systems may be stored as EPSG integers or authority strings.
CRSLike = int | str | None

# Parsed GeoJSON objects are represented as mappings to keep them serializable.
GeoJSONLike = Mapping[str, object]

# Paths accepted by IO helpers.
PathLike = str | Path

__all__ = (
    "Coordinate",
    "CoordinateSequence",
    "CRSLike",
    "GeoJSONLike",
    "GeometryHandle",
    "PathLike",
)
