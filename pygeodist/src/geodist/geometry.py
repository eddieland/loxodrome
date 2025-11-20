"""Rust-backed geometry wrappers exposed to Python callers."""

from __future__ import annotations

from collections.abc import Iterator
from math import isfinite

from . import _geodist_rs
from .errors import InvalidGeometryError
from .types import BoundingBoxDegrees, LatitudeDegrees, LongitudeDegrees, PointDegrees

__all__ = (
    "Point",
    "BoundingBox",
)


class Point:
    """Immutable geographic point expressed in degrees."""

    __slots__ = ("_handle",)

    def __init__(
        self,
        latitude_degrees: LatitudeDegrees,
        longitude_degrees: LongitudeDegrees,
    ) -> None:
        """Initialize a Point from latitude and longitude in degrees."""
        latitude = _coerce_latitude(latitude_degrees)
        longitude = _coerce_longitude(longitude_degrees)
        self._handle = _geodist_rs.Point(latitude, longitude)

    @property
    def latitude_degrees(self) -> LatitudeDegrees:
        """Return the latitude in degrees."""
        return float(self._handle.latitude_degrees)

    @property
    def longitude_degrees(self) -> LongitudeDegrees:
        """Return the longitude in degrees."""
        return float(self._handle.longitude_degrees)

    def to_tuple(self) -> PointDegrees:
        """Return a tuple representation for interoperability."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        """Iterate over the latitude and longitude in degrees."""
        yield from self.to_tuple()

    def __repr__(self) -> str:
        """Return a string representation of the Point."""
        return f"Point(latitude_degrees={self.latitude_degrees}, longitude_degrees={self.longitude_degrees})"

    def __eq__(self, other: object) -> bool:
        """Check equality with another Point."""
        if not isinstance(other, Point):
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


def _coerce_latitude(latitude_degrees: float) -> LatitudeDegrees:
    return _coerce_coordinate(
        latitude_degrees,
        min_value=_LATITUDE_MIN_DEGREES,
        max_value=_LATITUDE_MAX_DEGREES,
        name="latitude_degrees",
    )


def _coerce_longitude(longitude_degrees: float) -> LongitudeDegrees:
    return _coerce_coordinate(
        longitude_degrees,
        min_value=_LONGITUDE_MIN_DEGREES,
        max_value=_LONGITUDE_MAX_DEGREES,
        name="longitude_degrees",
    )


class BoundingBox:
    """Immutable geographic bounding box expressed in degrees."""

    __slots__ = ("_handle",)

    def __init__(
        self,
        min_latitude_degrees: LatitudeDegrees,
        max_latitude_degrees: LatitudeDegrees,
        min_longitude_degrees: LongitudeDegrees,
        max_longitude_degrees: LongitudeDegrees,
    ) -> None:
        """Initialize a BoundingBox from min/max latitude and longitude in degrees."""
        min_latitude = _coerce_latitude(min_latitude_degrees)
        max_latitude = _coerce_latitude(max_latitude_degrees)
        min_longitude = _coerce_longitude(min_longitude_degrees)
        max_longitude = _coerce_longitude(max_longitude_degrees)

        if min_latitude > max_latitude:
            raise InvalidGeometryError(
                f"min_latitude_degrees must not exceed max_latitude_degrees: {min_latitude} > {max_latitude}"
            )
        if min_longitude > max_longitude:
            raise InvalidGeometryError(
                f"min_longitude_degrees must not exceed max_longitude_degrees: {min_longitude} > {max_longitude}"
            )

        self._handle = _geodist_rs.BoundingBox(
            min_latitude,
            max_latitude,
            min_longitude,
            max_longitude,
        )

    @property
    def min_latitude_degrees(self) -> LatitudeDegrees:
        """Return the minimum latitude in degrees."""
        return float(self._handle.min_latitude_degrees)

    @property
    def max_latitude_degrees(self) -> LatitudeDegrees:
        """Return the maximum latitude in degrees."""
        return float(self._handle.max_latitude_degrees)

    @property
    def min_longitude_degrees(self) -> LongitudeDegrees:
        """Return the minimum longitude in degrees."""
        return float(self._handle.min_longitude_degrees)

    @property
    def max_longitude_degrees(self) -> LongitudeDegrees:
        """Return the maximum longitude in degrees."""
        return float(self._handle.max_longitude_degrees)

    def to_tuple(self) -> BoundingBoxDegrees:
        """Return the bounding box as a tuple of degrees."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        """Iterate over the bounding box coordinates in degrees."""
        yield from self.to_tuple()

    def __repr__(self) -> str:
        """Return a string representation of the BoundingBox."""
        return (
            "BoundingBox("
            f"min_latitude_degrees={self.min_latitude_degrees}, "
            f"max_latitude_degrees={self.max_latitude_degrees}, "
            f"min_longitude_degrees={self.min_longitude_degrees}, "
            f"max_longitude_degrees={self.max_longitude_degrees}"
            ")"
        )
