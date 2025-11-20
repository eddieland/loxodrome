"""Optional Shapely interoperability helpers.

Imports are guarded so Shapely remains an opt-in dependency.
"""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable

from ..errors import InvalidGeometryError
from ..geometry import Point

__all__ = ("from_shapely", "to_shapely")


@runtime_checkable
class _PointLike(Protocol):
    x: float
    y: float
    has_z: bool


def _import_shapely_point() -> type[Any]:
    try:
        from shapely.geometry import Point as ShapelyPoint
    except ModuleNotFoundError as exc:
        raise ImportError(
            "Shapely is required for interop helpers; install the optional extra with "
            "`pip install pygeodist[shapely]` or add `shapely` to your environment."
        ) from exc
    return ShapelyPoint


def to_shapely(point: Point) -> Any:
    """Convert a geodist `Point` into a Shapely `Point`."""
    shapely_point = _import_shapely_point()
    if not isinstance(point, Point):
        raise TypeError(f"to_shapely expects a geodist Point, got {type(point).__name__}")

    # Shapely uses (x, y) == (longitude, latitude).
    latitude, longitude = point.to_tuple()
    return shapely_point(longitude, latitude)


def from_shapely(point: _PointLike) -> Point:
    """Convert a Shapely `Point` into a geodist `Point`."""
    shapely_point = _import_shapely_point()
    if not isinstance(point, shapely_point):
        raise TypeError(f"from_shapely expects shapely.geometry.Point, got {type(point).__name__}")

    if getattr(point, "has_z", False):
        raise InvalidGeometryError("3D Shapely points are not supported; drop the Z coordinate.")

    latitude: float = float(point.y)
    longitude: float = float(point.x)
    return Point(latitude, longitude)
