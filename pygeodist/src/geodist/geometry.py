"""Rust-backed geometry wrappers exposed to Python callers."""

from __future__ import annotations

from collections.abc import Iterator
from math import isfinite

from . import _geodist_rs
from .errors import InvalidGeometryError
from .types import LatitudeDegrees, LongitudeDegrees, PointDegrees

__all__ = ("Point",)


class Point:
    """Immutable geographic point expressed in degrees."""

    __slots__ = ("_handle",)

    def __init__(
        self,
        latitude_degrees: LatitudeDegrees,
        longitude_degrees: LongitudeDegrees,
    ) -> None:
        latitude = _coerce_latitude(latitude_degrees)
        longitude = _coerce_longitude(longitude_degrees)
        self._handle = _geodist_rs.Point(latitude, longitude)

    @property
    def latitude_degrees(self) -> LatitudeDegrees:
        return float(self._handle.latitude_degrees)

    @property
    def longitude_degrees(self) -> LongitudeDegrees:
        return float(self._handle.longitude_degrees)

    def to_tuple(self) -> PointDegrees:
        """Return a tuple representation for interoperability."""
        return self._handle.to_tuple()

    def __iter__(self) -> Iterator[float]:
        yield from self.to_tuple()

    def __repr__(self) -> str:
        return (
            "Point("
            f"latitude_degrees={self.latitude_degrees}, "
            f"longitude_degrees={self.longitude_degrees}"
            ")"
        )

    def __eq__(self, other: object) -> bool:
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
        raise InvalidGeometryError(
            f"{name} {numeric_value!r} outside valid range [{min_value}, {max_value}]"
        )

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
