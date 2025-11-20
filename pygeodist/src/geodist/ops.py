"""Stateless geodesic operations backed by the Rust kernels."""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass

from . import _geodist_rs
from .errors import GeodistError
from .geometry import BoundingBox, Point, Point3D
from .types import Meters

__all__ = (
    "GeodesicResult",
    "geodesic_distance",
    "geodesic_distance_3d",
    "geodesic_with_bearings",
    "hausdorff_directed_3d",
    "hausdorff_directed",
    "hausdorff_3d",
    "hausdorff",
    "hausdorff_directed_clipped_3d",
    "hausdorff_directed_clipped",
    "hausdorff_clipped_3d",
    "hausdorff_clipped",
)


@dataclass(frozen=True)
class GeodesicResult:
    """Result of a geodesic computation including distance and bearings."""

    distance_meters: Meters
    initial_bearing_degrees: float
    final_bearing_degrees: float


def geodesic_distance(origin: Point, destination: Point) -> Meters:
    """Compute the great-circle distance between two points in meters."""
    try:
        return float(_geodist_rs.geodesic_distance(origin._handle, destination._handle))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def geodesic_distance_3d(origin: Point3D, destination: Point3D) -> Meters:
    """Compute straight-line (ECEF chord) distance between two 3D points in meters."""
    try:
        return float(_geodist_rs.geodesic_distance_3d(origin._handle, destination._handle))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def geodesic_with_bearings(origin: Point, destination: Point) -> GeodesicResult:
    """Compute great-circle distance and bearings between two points."""
    try:
        solution = _geodist_rs.geodesic_with_bearings(origin._handle, destination._handle)
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return GeodesicResult(
        distance_meters=float(solution.distance_meters),
        initial_bearing_degrees=float(solution.initial_bearing_degrees),
        final_bearing_degrees=float(solution.final_bearing_degrees),
    )


def hausdorff_directed(a: Iterable[Point], b: Iterable[Point]) -> Meters:
    """Directed Hausdorff distance from set `a` to set `b`."""
    try:
        return float(_geodist_rs.hausdorff_directed([it._handle for it in a], [it._handle for it in b]))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff(a: Iterable[Point], b: Iterable[Point]) -> Meters:
    """Symmetric Hausdorff distance between two point sets."""
    try:
        return float(_geodist_rs.hausdorff([it._handle for it in a], [it._handle for it in b]))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff_directed_clipped(
    a: Iterable[Point],
    b: Iterable[Point],
    bounding_box: BoundingBox,
) -> Meters:
    """Directed Hausdorff distance after clipping both sets to a bounding box."""
    try:
        return float(
            _geodist_rs.hausdorff_directed_clipped(
                [it._handle for it in a], [it._handle for it in b], bounding_box._handle
            )
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff_clipped(a: Iterable[Point], b: Iterable[Point], bounding_box: BoundingBox) -> Meters:
    """Symmetric Hausdorff distance after clipping both sets to a bounding box."""
    try:
        return float(
            _geodist_rs.hausdorff_clipped([it._handle for it in a], [it._handle for it in b], bounding_box._handle)
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff_directed_3d(a: Iterable[Point3D], b: Iterable[Point3D]) -> Meters:
    """Directed 3D Hausdorff distance using the ECEF chord metric."""
    try:
        return float(_geodist_rs.hausdorff_directed_3d([it._handle for it in a], [it._handle for it in b]))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff_3d(a: Iterable[Point3D], b: Iterable[Point3D]) -> Meters:
    """Symmetric 3D Hausdorff distance using the ECEF chord metric."""
    try:
        return float(_geodist_rs.hausdorff_3d([it._handle for it in a], [it._handle for it in b]))
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff_directed_clipped_3d(
    a: Iterable[Point3D],
    b: Iterable[Point3D],
    bounding_box: BoundingBox,
) -> Meters:
    """Directed 3D Hausdorff distance after clipping points by latitude/longitude."""
    try:
        return float(
            _geodist_rs.hausdorff_directed_clipped_3d(
                [it._handle for it in a],
                [it._handle for it in b],
                bounding_box._handle,
            )
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc


def hausdorff_clipped_3d(a: Iterable[Point3D], b: Iterable[Point3D], bounding_box: BoundingBox) -> Meters:
    """Symmetric 3D Hausdorff distance after clipping points by latitude/longitude."""
    try:
        return float(
            _geodist_rs.hausdorff_clipped_3d(
                [it._handle for it in a],
                [it._handle for it in b],
                bounding_box._handle,
            )
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc
