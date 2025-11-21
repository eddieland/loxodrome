"""Rust-backed geometry wrappers exposed to Python callers."""

from __future__ import annotations

from collections.abc import Iterator
from math import isfinite
from typing import Sequence

from . import _geodist_rs
from .errors import InvalidGeometryError
from .types import AltitudeM, Latitude, Longitude
from .types import BoundingBox as BoundingBoxTuple
from .types import Point as PointTuple
from .types import Point3D as Point3DTuple

__all__ = (
    "Ellipsoid",
    "Point",
    "Point3D",
    "BoundingBox",
    "Polygon",
)


class Ellipsoid:
    """Immutable ellipsoid definition expressed in meters."""

    __slots__ = ("_handle",)

    def __init__(self, semi_major_axis_m: float, semi_minor_axis_m: float) -> None:
        """Initialize an ellipsoid from semi-major/minor axes in meters."""
        _validate_axis(semi_minor_axis_m, name="semi_minor_axis_m")
        _validate_axis(semi_major_axis_m, name="semi_major_axis_m")

        if semi_major_axis_m < semi_minor_axis_m:
            raise InvalidGeometryError(
                f"semi_major_axis_m must be >= semi_minor_axis_m: {semi_major_axis_m!r} < {semi_minor_axis_m!r}"
            )

        self._handle = _geodist_rs.Ellipsoid(semi_major_axis_m, semi_minor_axis_m)

    @classmethod
    def _from_handle(cls, handle: _geodist_rs.Ellipsoid) -> "Ellipsoid":
        instance = cls.__new__(cls)
        instance._handle = handle
        return instance

    @classmethod
    def wgs84(cls) -> "Ellipsoid":
        """Return the WGS84 reference ellipsoid."""
        return cls._from_handle(_geodist_rs.Ellipsoid.wgs84())

    @property
    def semi_major_axis_m(self) -> float:
        """Semi-major axis in meters."""
        return float(self._handle.semi_major_axis_m)

    @property
    def semi_minor_axis_m(self) -> float:
        """Semi-minor axis in meters."""
        return float(self._handle.semi_minor_axis_m)

    def to_tuple(self) -> tuple[float, float]:
        """Return a tuple representation `(semi_major_axis_m, semi_minor_axis_m)`."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        """Iterate over the semi-major and semi-minor axes."""
        yield from self.to_tuple()

    def __repr__(self) -> str:
        """Return a string representation of the ellipsoid."""
        return f"Ellipsoid(semi_major_axis_m={self.semi_major_axis_m}, semi_minor_axis_m={self.semi_minor_axis_m})"

    def __eq__(self, other: object) -> bool:
        """Check equality with another Ellipsoid."""
        if not isinstance(other, Ellipsoid):
            return NotImplemented
        return self.to_tuple() == other.to_tuple()


class Point:
    """Immutable geographic point expressed in degrees."""

    __slots__ = ("_handle",)

    def __init__(self, lat: Latitude, lon: Longitude) -> None:
        """Initialize a Point from latitude and longitude in degrees."""
        latitude = _coerce_latitude(lat)
        longitude = _coerce_longitude(lon)
        self._handle = _geodist_rs.Point(latitude, longitude)

    @property
    def lat(self) -> Latitude:
        """Return the latitude in degrees."""
        return float(self._handle.lat)

    @property
    def lon(self) -> Longitude:
        """Return the longitude in degrees."""
        return float(self._handle.lon)

    def to_tuple(self) -> PointTuple:
        """Return a tuple representation for interoperability."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        """Iterate over the latitude and longitude in degrees."""
        yield from self.to_tuple()

    def __repr__(self) -> str:
        """Return a string representation of the Point."""
        return f"Point(lat={self.lat}, lon={self.lon})"

    def __eq__(self, other: object) -> bool:
        """Check equality with another Point."""
        if not isinstance(other, Point):
            return NotImplemented
        return self.to_tuple() == other.to_tuple()


class Point3D:
    """Immutable geographic point with altitude."""

    __slots__ = ("_handle",)

    def __init__(
        self,
        lat: Latitude,
        lon: Longitude,
        altitude_m: AltitudeM,
    ) -> None:
        """Initialize a 3D point from latitude/longitude in degrees and altitude in meters."""
        latitude = _coerce_latitude(lat)
        longitude = _coerce_longitude(lon)
        altitude = _coerce_altitude(altitude_m)
        self._handle = _geodist_rs.Point3D(latitude, longitude, altitude)

    @property
    def lat(self) -> Latitude:
        """Return the latitude in degrees."""
        return float(self._handle.lat)

    @property
    def lon(self) -> Longitude:
        """Return the longitude in degrees."""
        return float(self._handle.lon)

    @property
    def altitude_m(self) -> AltitudeM:
        """Return the altitude in meters."""
        return float(self._handle.altitude_m)

    def to_tuple(self) -> Point3DTuple:
        """Return a tuple representation for interoperability."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        """Iterate over the latitude, longitude, and altitude."""
        yield from self.to_tuple()

    def __repr__(self) -> str:
        """Return a string representation of the 3D point."""
        return f"Point3D(lat={self.lat}, lon={self.lon}, altitude_m={self.altitude_m})"

    def __eq__(self, other: object) -> bool:
        """Check equality with another Point3D."""
        if not isinstance(other, Point3D):
            return NotImplemented
        return self.to_tuple() == other.to_tuple()


_LATITUDE_MIN_DEGREES = -90.0
_LATITUDE_MAX_DEGREES = 90.0
_LONGITUDE_MIN_DEGREES = -180.0
_LONGITUDE_MAX_DEGREES = 180.0


def _validate_axis(value: float, *, name: str) -> None:
    """Convert an axis length into a finite positive float."""
    if not isfinite(value):
        raise InvalidGeometryError(f"{name} must be finite: {value!r}")

    if value <= 0.0:
        raise InvalidGeometryError(f"{name} must be greater than 0 meters: {value!r}")


def _coerce_coordinate(
    value: float,
    *,
    min_value: float,
    max_value: float,
    name: str,
) -> float:
    """Convert an input into a finite float within the allowed bounds."""
    if isinstance(value, bool):
        raise InvalidGeometryError(f"{name} must be a float, not bool: {value!r}")

    try:
        numeric_value = float(value)
    except (TypeError, ValueError) as exc:
        raise InvalidGeometryError(f"{name} must be convertible to float: {value!r}") from exc

    if not isfinite(numeric_value):
        raise InvalidGeometryError(f"{name} must be finite: {numeric_value!r}")

    if numeric_value < min_value or numeric_value > max_value:
        raise InvalidGeometryError(f"{name} {numeric_value!r} outside valid range [{min_value}, {max_value}]")

    return numeric_value


def _coerce_latitude(lat: float) -> Latitude:
    return _coerce_coordinate(
        lat,
        min_value=_LATITUDE_MIN_DEGREES,
        max_value=_LATITUDE_MAX_DEGREES,
        name="lat",
    )


def _coerce_longitude(lon: float) -> Longitude:
    return _coerce_coordinate(
        lon,
        min_value=_LONGITUDE_MIN_DEGREES,
        max_value=_LONGITUDE_MAX_DEGREES,
        name="lon",
    )


def _coerce_altitude(altitude_m: float) -> AltitudeM:
    if isinstance(altitude_m, bool):
        raise InvalidGeometryError(f"altitude_m must be a float, not bool: {altitude_m!r}")

    try:
        numeric_value = float(altitude_m)
    except (TypeError, ValueError) as exc:
        raise InvalidGeometryError(f"altitude_m must be convertible to float: {altitude_m!r}") from exc

    if not isfinite(numeric_value):
        raise InvalidGeometryError(f"altitude_m must be finite: {numeric_value!r}")

    return numeric_value


def _coerce_point_like(value: Point | PointTuple) -> PointTuple:
    if isinstance(value, Point):
        return value.to_tuple()
    if isinstance(value, tuple) and len(value) == 2:
        lat, lon = value
        return (_coerce_latitude(lat), _coerce_longitude(lon))
    raise InvalidGeometryError(f"expected Point or (lat, lon) tuple, got {type(value).__name__}")


class BoundingBox:
    """Immutable geographic bounding box expressed in degrees."""

    __slots__ = ("_handle",)

    def __init__(
        self,
        min_lat: Latitude,
        max_lat: Latitude,
        min_lon: Longitude,
        max_lon: Longitude,
    ) -> None:
        """Initialize a BoundingBox from min/max latitude and longitude in degrees."""
        min_latitude = _coerce_latitude(min_lat)
        max_latitude = _coerce_latitude(max_lat)
        min_longitude = _coerce_longitude(min_lon)
        max_longitude = _coerce_longitude(max_lon)

        if min_latitude > max_latitude:
            raise InvalidGeometryError(f"min_lat must not exceed max_lat: {min_latitude} > {max_latitude}")

        self._handle = _geodist_rs.BoundingBox(
            min_latitude,
            max_latitude,
            min_longitude,
            max_longitude,
        )

    @property
    def min_lat(self) -> Latitude:
        """Return the minimum latitude in degrees."""
        return float(self._handle.min_lat)

    @property
    def max_lat(self) -> Latitude:
        """Return the maximum latitude in degrees."""
        return float(self._handle.max_lat)

    @property
    def min_lon(self) -> Longitude:
        """Return the minimum longitude in degrees."""
        return float(self._handle.min_lon)

    @property
    def max_lon(self) -> Longitude:
        """Return the maximum longitude in degrees."""
        return float(self._handle.max_lon)

    def to_tuple(self) -> BoundingBoxTuple:
        """Return the bounding box as a tuple of degrees."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        """Iterate over the bounding box coordinates in degrees."""
        yield from self.to_tuple()

    def __repr__(self) -> str:
        """Return a string representation of the BoundingBox."""
        return (
            "BoundingBox("
            f"min_lat={self.min_lat}, "
            f"max_lat={self.max_lat}, "
            f"min_lon={self.min_lon}, "
            f"max_lon={self.max_lon}"
            ")"
        )

    def __eq__(self, other: object) -> bool:
        """Check equality with another BoundingBox."""
        if not isinstance(other, BoundingBox):
            return NotImplemented
        return self.to_tuple() == other.to_tuple()


class Polygon:
    """Immutable polygon boundary consisting of an exterior ring and optional holes."""

    __slots__ = ("_handle",)

    def __init__(
        self,
        exterior: Sequence[Point | PointTuple],
        holes: Sequence[Sequence[Point | PointTuple]] | None = None,
    ) -> None:
        """Initialize a polygon boundary with CCW exterior and optional CW holes."""
        exterior_ring = [_coerce_point_like(vertex) for vertex in exterior]
        hole_rings = [[_coerce_point_like(vertex) for vertex in ring] for ring in holes or []]
        self._handle = _geodist_rs.Polygon(exterior_ring, hole_rings)

    def __repr__(self) -> str:
        """Return a concise representation for debugging."""
        return f"Polygon(num_holes={len(self.to_tuple()[1])})"

    def to_tuple(self) -> tuple[list[PointTuple], list[list[PointTuple]]]:
        """Return a tuple of exterior and holes for inspection."""
        exterior, holes = self._handle.to_tuple()
        return list(exterior), [list(ring) for ring in holes]

    def area_m2(self) -> float:
        """Return the spherical area of the polygon in square meters."""
        return float(self._handle.area_m2())
