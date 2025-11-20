"""Rust-backed geometry wrappers exposed to Python callers."""

from __future__ import annotations

from collections.abc import Iterator
from math import isfinite

from . import _geodist_rs
from .errors import InvalidGeometryError
from .types import AltitudeM, Latitude, Longitude
from .types import BoundingBox as BoundingBoxTuple
from .types import Point as PointTuple
from .types import Point3D as Point3DTuple

__all__ = (
    "Point",
    "Point3D",
    "BoundingBox",
)


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
        if min_longitude > max_longitude:
            raise InvalidGeometryError(f"min_lon must not exceed max_lon: {min_longitude} > {max_longitude}")

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
