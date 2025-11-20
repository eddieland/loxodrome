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
    "HausdorffDirectedWitness",
    "HausdorffWitness",
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


@dataclass(frozen=True)
class HausdorffDirectedWitness:
    """Directed Hausdorff witness containing the realizing pair indices."""

    distance_meters: Meters
    origin_index: int
    candidate_index: int


@dataclass(frozen=True)
class HausdorffWitness:
    """Symmetric Hausdorff witness with per-direction details."""

    distance_meters: Meters
    a_to_b: HausdorffDirectedWitness
    b_to_a: HausdorffDirectedWitness


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


def hausdorff_directed(a: Iterable[Point], b: Iterable[Point]) -> HausdorffDirectedWitness:
    """Directed Hausdorff distance and witness from set `a` to set `b`."""
    try:
        witness = _geodist_rs.hausdorff_directed([it._handle for it in a], [it._handle for it in b])
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffDirectedWitness(
        distance_meters=float(witness.distance_meters),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff(a: Iterable[Point], b: Iterable[Point]) -> HausdorffWitness:
    """Symmetric Hausdorff distance and witnesses between two point sets."""
    try:
        witness = _geodist_rs.hausdorff([it._handle for it in a], [it._handle for it in b])
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffWitness(
        distance_meters=float(witness.distance_meters),
        a_to_b=HausdorffDirectedWitness(
            distance_meters=float(witness.a_to_b.distance_meters),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_meters=float(witness.b_to_a.distance_meters),
            origin_index=int(witness.b_to_a.origin_index),
            candidate_index=int(witness.b_to_a.candidate_index),
        ),
    )


def hausdorff_directed_clipped(
    a: Iterable[Point],
    b: Iterable[Point],
    bounding_box: BoundingBox,
) -> HausdorffDirectedWitness:
    """Directed Hausdorff witness after clipping both sets to a bounding box."""
    try:
        witness = _geodist_rs.hausdorff_directed_clipped(
            [it._handle for it in a],
            [it._handle for it in b],
            bounding_box._handle,
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffDirectedWitness(
        distance_meters=float(witness.distance_meters),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff_clipped(a: Iterable[Point], b: Iterable[Point], bounding_box: BoundingBox) -> HausdorffWitness:
    """Symmetric Hausdorff witness after clipping both sets to a bounding box."""
    try:
        witness = _geodist_rs.hausdorff_clipped(
            [it._handle for it in a],
            [it._handle for it in b],
            bounding_box._handle,
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffWitness(
        distance_meters=float(witness.distance_meters),
        a_to_b=HausdorffDirectedWitness(
            distance_meters=float(witness.a_to_b.distance_meters),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_meters=float(witness.b_to_a.distance_meters),
            origin_index=int(witness.b_to_a.origin_index),
            candidate_index=int(witness.b_to_a.candidate_index),
        ),
    )


def hausdorff_directed_3d(a: Iterable[Point3D], b: Iterable[Point3D]) -> HausdorffDirectedWitness:
    """Directed 3D Hausdorff witness using the ECEF chord metric."""
    try:
        witness = _geodist_rs.hausdorff_directed_3d([it._handle for it in a], [it._handle for it in b])
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffDirectedWitness(
        distance_meters=float(witness.distance_meters),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff_3d(a: Iterable[Point3D], b: Iterable[Point3D]) -> HausdorffWitness:
    """Symmetric 3D Hausdorff witness using the ECEF chord metric."""
    try:
        witness = _geodist_rs.hausdorff_3d([it._handle for it in a], [it._handle for it in b])
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffWitness(
        distance_meters=float(witness.distance_meters),
        a_to_b=HausdorffDirectedWitness(
            distance_meters=float(witness.a_to_b.distance_meters),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_meters=float(witness.b_to_a.distance_meters),
            origin_index=int(witness.b_to_a.origin_index),
            candidate_index=int(witness.b_to_a.candidate_index),
        ),
    )


def hausdorff_directed_clipped_3d(
    a: Iterable[Point3D],
    b: Iterable[Point3D],
    bounding_box: BoundingBox,
) -> HausdorffDirectedWitness:
    """Directed 3D Hausdorff witness after clipping points by latitude/longitude."""
    try:
        witness = _geodist_rs.hausdorff_directed_clipped_3d(
            [it._handle for it in a],
            [it._handle for it in b],
            bounding_box._handle,
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffDirectedWitness(
        distance_meters=float(witness.distance_meters),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff_clipped_3d(a: Iterable[Point3D], b: Iterable[Point3D], bounding_box: BoundingBox) -> HausdorffWitness:
    """Symmetric 3D Hausdorff witness after clipping points by latitude/longitude."""
    try:
        witness = _geodist_rs.hausdorff_clipped_3d(
            [it._handle for it in a],
            [it._handle for it in b],
            bounding_box._handle,
        )
    except ValueError as exc:
        raise GeodistError(str(exc)) from exc

    return HausdorffWitness(
        distance_meters=float(witness.distance_meters),
        a_to_b=HausdorffDirectedWitness(
            distance_meters=float(witness.a_to_b.distance_meters),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_meters=float(witness.b_to_a.distance_meters),
            origin_index=int(witness.b_to_a.origin_index),
            candidate_index=int(witness.b_to_a.candidate_index),
        ),
    )
