"""Stateless geodesic operations backed by the Rust kernels."""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass

from . import _geodist_rs
from .geometry import BoundingBox, Ellipsoid, Point, Point3D
from .types import Meters

__all__ = (
    "GeodesicResult",
    "HausdorffDirectedWitness",
    "HausdorffWitness",
    "geodesic_distance",
    "geodesic_distance_on_ellipsoid",
    "geodesic_distance_3d",
    "geodesic_with_bearings",
    "geodesic_with_bearings_on_ellipsoid",
    "hausdorff_directed_3d",
    "hausdorff_directed",
    "hausdorff_3d",
    "hausdorff",
    "hausdorff_directed_clipped_3d",
    "hausdorff_directed_clipped",
    "hausdorff_clipped_3d",
    "hausdorff_clipped",
)


@dataclass(frozen=True, slots=True)
class GeodesicResult:
    """Result of a geodesic computation including distance and bearings."""

    distance_m: Meters
    initial_bearing_deg: float
    final_bearing_deg: float


@dataclass(frozen=True, slots=True)
class HausdorffDirectedWitness:
    """Directed Hausdorff witness containing the realizing pair indices."""

    distance_m: Meters
    origin_index: int
    candidate_index: int


@dataclass(frozen=True, slots=True)
class HausdorffWitness:
    """Symmetric Hausdorff witness with per-direction details."""

    distance_m: Meters
    a_to_b: HausdorffDirectedWitness
    b_to_a: HausdorffDirectedWitness


def geodesic_distance(origin: Point, destination: Point) -> Meters:
    """Compute the great-circle distance between two points in meters."""
    return float(_geodist_rs.geodesic_distance(origin._handle, destination._handle))


def geodesic_distance_on_ellipsoid(
    origin: Point,
    destination: Point,
    ellipsoid: Ellipsoid | None = None,
) -> Meters:
    """Compute the ellipsoidal geodesic distance between two points in meters."""
    model = ellipsoid or Ellipsoid.wgs84()
    return float(
        _geodist_rs.geodesic_distance_on_ellipsoid(
            origin._handle,
            destination._handle,
            ellipsoid=model._handle,
        )
    )


def geodesic_distance_3d(origin: Point3D, destination: Point3D) -> Meters:
    """Compute straight-line (ECEF chord) distance between two 3D points in meters."""
    return float(_geodist_rs.geodesic_distance_3d(origin._handle, destination._handle))


def geodesic_with_bearings(origin: Point, destination: Point) -> GeodesicResult:
    """Compute great-circle distance and bearings between two points."""
    solution = _geodist_rs.geodesic_with_bearings(origin._handle, destination._handle)

    return GeodesicResult(
        distance_m=float(solution.distance_m),
        initial_bearing_deg=float(solution.initial_bearing_deg),
        final_bearing_deg=float(solution.final_bearing_deg),
    )


def geodesic_with_bearings_on_ellipsoid(
    origin: Point,
    destination: Point,
    ellipsoid: Ellipsoid | None = None,
) -> GeodesicResult:
    """Compute ellipsoidal distance and bearings between two points."""
    model = ellipsoid or Ellipsoid.wgs84()
    solution = _geodist_rs.geodesic_with_bearings_on_ellipsoid(
        origin._handle,
        destination._handle,
        ellipsoid=model._handle,
    )

    return GeodesicResult(
        distance_m=float(solution.distance_m),
        initial_bearing_deg=float(solution.initial_bearing_deg),
        final_bearing_deg=float(solution.final_bearing_deg),
    )


def hausdorff_directed(a: Iterable[Point], b: Iterable[Point]) -> HausdorffDirectedWitness:
    """Directed Hausdorff distance and witness from set `a` to set `b`."""
    witness = _geodist_rs.hausdorff_directed(
        a=[it._handle for it in a],
        b=[it._handle for it in b],
    )

    return HausdorffDirectedWitness(
        distance_m=float(witness.distance_m),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff(a: Iterable[Point], b: Iterable[Point]) -> HausdorffWitness:
    """Symmetric Hausdorff distance and witnesses between two point sets."""
    witness = _geodist_rs.hausdorff(
        a=[it._handle for it in a],
        b=[it._handle for it in b],
    )

    return HausdorffWitness(
        distance_m=float(witness.distance_m),
        a_to_b=HausdorffDirectedWitness(
            distance_m=float(witness.a_to_b.distance_m),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_m=float(witness.b_to_a.distance_m),
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
    witness = _geodist_rs.hausdorff_directed_clipped(
        [it._handle for it in a],
        [it._handle for it in b],
        bounding_box._handle,
    )

    return HausdorffDirectedWitness(
        distance_m=float(witness.distance_m),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff_clipped(a: Iterable[Point], b: Iterable[Point], bounding_box: BoundingBox) -> HausdorffWitness:
    """Symmetric Hausdorff witness after clipping both sets to a bounding box."""
    witness = _geodist_rs.hausdorff_clipped(
        [it._handle for it in a],
        [it._handle for it in b],
        bounding_box._handle,
    )

    return HausdorffWitness(
        distance_m=float(witness.distance_m),
        a_to_b=HausdorffDirectedWitness(
            distance_m=float(witness.a_to_b.distance_m),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_m=float(witness.b_to_a.distance_m),
            origin_index=int(witness.b_to_a.origin_index),
            candidate_index=int(witness.b_to_a.candidate_index),
        ),
    )


def hausdorff_directed_3d(a: Iterable[Point3D], b: Iterable[Point3D]) -> HausdorffDirectedWitness:
    """Directed 3D Hausdorff witness using the ECEF chord metric."""
    witness = _geodist_rs.hausdorff_directed_3d(
        a=[it._handle for it in a],
        b=[it._handle for it in b],
    )

    return HausdorffDirectedWitness(
        distance_m=float(witness.distance_m),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff_3d(a: Iterable[Point3D], b: Iterable[Point3D]) -> HausdorffWitness:
    """Symmetric 3D Hausdorff witness using the ECEF chord metric."""
    witness = _geodist_rs.hausdorff_3d(
        a=[it._handle for it in a],
        b=[it._handle for it in b],
    )

    return HausdorffWitness(
        distance_m=float(witness.distance_m),
        a_to_b=HausdorffDirectedWitness(
            distance_m=float(witness.a_to_b.distance_m),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_m=float(witness.b_to_a.distance_m),
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
    witness = _geodist_rs.hausdorff_directed_clipped_3d(
        [it._handle for it in a],
        [it._handle for it in b],
        bounding_box._handle,
    )

    return HausdorffDirectedWitness(
        distance_m=float(witness.distance_m),
        origin_index=int(witness.origin_index),
        candidate_index=int(witness.candidate_index),
    )


def hausdorff_clipped_3d(a: Iterable[Point3D], b: Iterable[Point3D], bounding_box: BoundingBox) -> HausdorffWitness:
    """Symmetric 3D Hausdorff witness after clipping points by latitude/longitude."""
    witness = _geodist_rs.hausdorff_clipped_3d(
        [it._handle for it in a],
        [it._handle for it in b],
        bounding_box._handle,
    )

    return HausdorffWitness(
        distance_m=float(witness.distance_m),
        a_to_b=HausdorffDirectedWitness(
            distance_m=float(witness.a_to_b.distance_m),
            origin_index=int(witness.a_to_b.origin_index),
            candidate_index=int(witness.a_to_b.candidate_index),
        ),
        b_to_a=HausdorffDirectedWitness(
            distance_m=float(witness.b_to_a.distance_m),
            origin_index=int(witness.b_to_a.origin_index),
            candidate_index=int(witness.b_to_a.candidate_index),
        ),
    )
