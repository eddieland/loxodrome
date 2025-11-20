"""Stateless geodesic operations backed by the Rust kernels."""

from __future__ import annotations

from . import _geodist_rs
from .errors import GeodistError
from .geometry import Point
from .types import Meters

__all__ = ("geodesic_distance",)


def _to_handle(point: Point) -> _geodist_rs.Point:
    if not isinstance(point, Point):
        raise TypeError(f"geodesic_distance expects Point arguments, got {type(point).__name__}")
    return point._handle


def geodesic_distance(origin: Point, destination: Point) -> Meters:
    """Compute the great-circle distance between two points in meters."""
    try:
        return float(_geodist_rs.geodesic_distance(_to_handle(origin), _to_handle(destination)))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc
