"""Library-specific exceptions raised by the loxodrome Python layer."""

from __future__ import annotations

from . import _loxodrome_rs

GeodistError = _loxodrome_rs.GeodistError
InvalidLatitudeError = _loxodrome_rs.InvalidLatitudeError
InvalidLongitudeError = _loxodrome_rs.InvalidLongitudeError
InvalidAltitudeError = _loxodrome_rs.InvalidAltitudeError
InvalidDistanceError = _loxodrome_rs.InvalidDistanceError
InvalidRadiusError = _loxodrome_rs.InvalidRadiusError
InvalidEllipsoidError = _loxodrome_rs.InvalidEllipsoidError
InvalidBoundingBoxError = _loxodrome_rs.InvalidBoundingBoxError
EmptyPointSetError = _loxodrome_rs.EmptyPointSetError
InvalidGeometryError = _loxodrome_rs.InvalidGeometryError


__all__ = (
    "GeodistError",
    "InvalidGeometryError",
    "InvalidLatitudeError",
    "InvalidLongitudeError",
    "InvalidAltitudeError",
    "InvalidDistanceError",
    "InvalidRadiusError",
    "InvalidEllipsoidError",
    "InvalidBoundingBoxError",
    "EmptyPointSetError",
)
