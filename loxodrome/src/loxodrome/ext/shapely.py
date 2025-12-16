"""Optional Shapely interoperability helpers.

Imports are guarded so Shapely remains an opt-in dependency.
"""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable

from ..errors import InvalidGeometryError
from ..geometry import BoundingBox, LineString, Point, Point3D

try:
    from shapely.geometry import LineString as ShapelyLineString
    from shapely.geometry import Point as ShapelyPoint
    from shapely.geometry import Polygon as ShapelyPolygon
    from shapely.geometry import box as shapely_box
except ModuleNotFoundError as exc:
    raise ImportError(
        "Shapely is required for interop helpers; install the optional extra with "
        "`pip install loxodrome[shapely]` or add `shapely` to your environment."
    ) from exc
__all__ = ("from_shapely", "to_shapely")


@runtime_checkable
class _PointLike(Protocol):
    x: float
    y: float
    has_z: bool

    @property
    def z(self) -> float: ...


@runtime_checkable
class _PolygonLike(Protocol):
    bounds: tuple[float, float, float, float]

    def equals(self, other: Any) -> bool: ...


def to_shapely(geometry: Point | Point3D | BoundingBox | LineString) -> Any:
    """Convert a loxodrome geometry into the matching Shapely shape."""
    if isinstance(geometry, Point):
        latitude, longitude = geometry.to_tuple()
        return ShapelyPoint(longitude, latitude)
    if isinstance(geometry, Point3D):
        latitude, longitude, altitude_m = geometry.to_tuple()
        return ShapelyPoint(longitude, latitude, altitude_m)
    if isinstance(geometry, BoundingBox):
        min_lat, max_lat, min_lon, max_lon = geometry.to_tuple()
        return shapely_box(min_lon, min_lat, max_lon, max_lat)
    if isinstance(geometry, LineString):
        coords = [(lon, lat) for lat, lon in geometry.to_tuple()]
        return ShapelyLineString(coords)

    raise TypeError(
        "to_shapely expects a loxodrome geometry type (Point, Point3D, LineString, BoundingBox), "
        f"got {type(geometry).__name__}",
    )


def from_shapely(geometry: _PointLike | _PolygonLike) -> Point | Point3D | BoundingBox | LineString:
    """Convert a Shapely geometry into a loxodrome geometry."""
    if isinstance(geometry, ShapelyPoint):
        latitude: float = float(geometry.y)
        longitude: float = float(geometry.x)
        if getattr(geometry, "has_z", False):
            altitude_m: float = float(geometry.z)
            return Point3D(latitude, longitude, altitude_m)
        return Point(latitude, longitude)

    if isinstance(geometry, ShapelyLineString):
        if getattr(geometry, "has_z", False):
            raise InvalidGeometryError("3D LineStrings are not supported; drop Z or flatten before converting.")
        coords = [(float(lat), float(lon)) for lon, lat in geometry.coords]
        return LineString(coords)

    if isinstance(geometry, ShapelyPolygon):
        min_lon, min_lat, max_lon, max_lat = geometry.bounds
        rectangle = shapely_box(min_lon, min_lat, max_lon, max_lat)
        if not geometry.equals(rectangle):
            raise InvalidGeometryError("Only axis-aligned rectangular polygons can be converted to BoundingBox.")
        return BoundingBox(float(min_lat), float(max_lat), float(min_lon), float(max_lon))

    raise TypeError(
        "from_shapely expects shapely.geometry.Point (2D/3D) or a rectangular shapely.geometry.Polygon, "
        f"got {type(geometry).__name__}",
    )
