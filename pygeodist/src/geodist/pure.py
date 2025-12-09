"""Pure-Python reference implementations of the geodist kernels.

These helpers intentionally avoid the Rust extension so we can benchmark the
performance delta between the compiled bindings and straightforward Python math.
They mirror the public signatures for the spherical distance and Hausdorff
helpers but make no attempt to be numerically identical to the Rust kernels;
they are correctness baselines and a slow-path teaching tool.
"""

from __future__ import annotations

import math
from collections.abc import Iterable, Sequence

from .errors import InvalidGeometryError
from .geometry import BoundingBox, Point, Point3D, _coerce_point_like
from .ops import GeodesicResult, HausdorffDirectedWitness, HausdorffWitness
from .types import Meters, Point as PointTuple, Point3D as Point3DTuple

# Keep in sync with the Rust constant for spherical calculations.
EARTH_RADIUS_METERS = 6_371_008.8

__all__ = (
    "geodesic_distance_sphere",
    "geodesic_with_bearings_sphere",
    "geodesic_distance_3d_chord",
    "hausdorff_directed_naive",
    "hausdorff_naive",
    "hausdorff_directed_clipped_naive",
    "hausdorff_clipped_naive",
)


def _point_from_any(value: Point | PointTuple) -> PointTuple:
    return _coerce_point_like(value)


def _point3d_from_any(value: Point3D | Point3DTuple) -> Point3DTuple:
    if isinstance(value, Point3D):
        return value.to_tuple()
    if isinstance(value, tuple) and len(value) == 3:
        lat, lon, altitude_m = value
        lat_tuple, lon_tuple = _coerce_point_like((lat, lon))
        try:
            altitude = float(altitude_m)
        except (TypeError, ValueError) as exc:  # pragma: no cover - validation parity
            raise InvalidGeometryError("altitude_m must be convertible to float") from exc
        if not math.isfinite(altitude):
            raise InvalidGeometryError(f"altitude_m must be finite, got {altitude!r}")
        return lat_tuple, lon_tuple, altitude
    raise InvalidGeometryError(f"expected Point3D or (lat, lon, altitude_m) tuple, got {type(value).__name__}")


def _haversine_distance(lat1_deg: float, lon1_deg: float, lat2_deg: float, lon2_deg: float) -> float:
    lat1_rad = math.radians(lat1_deg)
    lon1_rad = math.radians(lon1_deg)
    lat2_rad = math.radians(lat2_deg)
    lon2_rad = math.radians(lon2_deg)

    delta_lat = lat2_rad - lat1_rad
    delta_lon = lon2_rad - lon1_rad

    sin_lat = math.sin(delta_lat / 2.0)
    sin_lon = math.sin(delta_lon / 2.0)
    a = sin_lat * sin_lat + math.cos(lat1_rad) * math.cos(lat2_rad) * sin_lon * sin_lon
    central_angle = 2.0 * math.atan2(math.sqrt(a), math.sqrt(1.0 - a))
    return EARTH_RADIUS_METERS * central_angle


def _initial_bearing(lat1_deg: float, lon1_deg: float, lat2_deg: float, lon2_deg: float) -> float:
    lat1_rad = math.radians(lat1_deg)
    lat2_rad = math.radians(lat2_deg)
    delta_lon = math.radians(lon2_deg - lon1_deg)

    x = math.sin(delta_lon) * math.cos(lat2_rad)
    y = math.cos(lat1_rad) * math.sin(lat2_rad) - math.sin(lat1_rad) * math.cos(lat2_rad) * math.cos(delta_lon)
    bearing_deg = math.degrees(math.atan2(x, y)) % 360.0
    return bearing_deg


def geodesic_distance_sphere(origin: Point | PointTuple, destination: Point | PointTuple) -> Meters:
    """Spherical great-circle distance using a pure-Python haversine."""

    origin_lat, origin_lon = _point_from_any(origin)
    dest_lat, dest_lon = _point_from_any(destination)
    return _haversine_distance(origin_lat, origin_lon, dest_lat, dest_lon)


def geodesic_with_bearings_sphere(origin: Point | PointTuple, destination: Point | PointTuple) -> GeodesicResult:
    """Spherical distance and initial/final bearings using pure Python math."""

    origin_lat, origin_lon = _point_from_any(origin)
    dest_lat, dest_lon = _point_from_any(destination)
    distance_m = _haversine_distance(origin_lat, origin_lon, dest_lat, dest_lon)
    initial = _initial_bearing(origin_lat, origin_lon, dest_lat, dest_lon)
    final = _initial_bearing(dest_lat, dest_lon, origin_lat, origin_lon)
    return GeodesicResult(distance_m=distance_m, initial_bearing_deg=initial, final_bearing_deg=final)


def geodesic_distance_3d_chord(origin: Point3D | Point3DTuple, destination: Point3D | Point3DTuple) -> Meters:
    """Straight-line ECEF chord distance using a spherical Earth approximation."""

    origin_lat, origin_lon, origin_alt = _point3d_from_any(origin)
    dest_lat, dest_lon, dest_alt = _point3d_from_any(destination)

    origin_radius = EARTH_RADIUS_METERS + origin_alt
    dest_radius = EARTH_RADIUS_METERS + dest_alt

    origin_lat_rad = math.radians(origin_lat)
    origin_lon_rad = math.radians(origin_lon)
    dest_lat_rad = math.radians(dest_lat)
    dest_lon_rad = math.radians(dest_lon)

    x1 = origin_radius * math.cos(origin_lat_rad) * math.cos(origin_lon_rad)
    y1 = origin_radius * math.cos(origin_lat_rad) * math.sin(origin_lon_rad)
    z1 = origin_radius * math.sin(origin_lat_rad)

    x2 = dest_radius * math.cos(dest_lat_rad) * math.cos(dest_lon_rad)
    y2 = dest_radius * math.cos(dest_lat_rad) * math.sin(dest_lon_rad)
    z2 = dest_radius * math.sin(dest_lat_rad)

    return math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2 + (z2 - z1) ** 2)


def _directed_hausdorff(
    a: Sequence[Point | PointTuple],
    b: Sequence[Point | PointTuple],
    *,
    allow_empty: bool = False,
) -> HausdorffDirectedWitness:
    if not a:
        raise InvalidGeometryError("set a must contain at least one point")
    if not b:
        if allow_empty:
            return HausdorffDirectedWitness(distance_m=0.0, origin_index=-1, candidate_index=-1)
        raise InvalidGeometryError("set b must contain at least one point")

    best_distance = -math.inf
    best_origin = -1
    best_candidate = -1

    for origin_index, origin in enumerate(a):
        origin_lat, origin_lon = _point_from_any(origin)
        nearest = math.inf
        nearest_index = -1
        for candidate_index, candidate in enumerate(b):
            candidate_lat, candidate_lon = _point_from_any(candidate)
            distance = _haversine_distance(origin_lat, origin_lon, candidate_lat, candidate_lon)
            if distance < nearest:
                nearest = distance
                nearest_index = candidate_index
        if nearest > best_distance:
            best_distance = nearest
            best_origin = origin_index
            best_candidate = nearest_index

    return HausdorffDirectedWitness(
        distance_m=float(best_distance), origin_index=int(best_origin), candidate_index=int(best_candidate)
    )


def hausdorff_directed_naive(a: Iterable[Point | PointTuple], b: Iterable[Point | PointTuple]) -> HausdorffDirectedWitness:
    """Directed Hausdorff distance using a naive O(n*m) search in Python."""

    points_a = list(a)
    points_b = list(b)
    return _directed_hausdorff(points_a, points_b)


def hausdorff_naive(a: Iterable[Point | PointTuple], b: Iterable[Point | PointTuple]) -> HausdorffWitness:
    """Symmetric Hausdorff distance using the naive Python kernels."""

    points_a = list(a)
    points_b = list(b)
    forward = _directed_hausdorff(points_a, points_b)
    reverse = _directed_hausdorff(points_b, points_a)
    distance_m = forward.distance_m if forward.distance_m >= reverse.distance_m else reverse.distance_m
    return HausdorffWitness(distance_m=distance_m, a_to_b=forward, b_to_a=reverse)


def _clip_points(points: Sequence[Point | PointTuple], bounding_box: BoundingBox) -> list[PointTuple]:
    min_lat, max_lat, min_lon, max_lon = bounding_box.to_tuple()
    clipped = [
        (lat, lon)
        for lat, lon in (_point_from_any(point) for point in points)
        if min_lat <= lat <= max_lat and min_lon <= lon <= max_lon
    ]
    if not clipped:
        raise InvalidGeometryError("all points were clipped by the bounding box")
    return clipped


def hausdorff_directed_clipped_naive(
    a: Iterable[Point | PointTuple], b: Iterable[Point | PointTuple], bounding_box: BoundingBox
) -> HausdorffDirectedWitness:
    """Directed Hausdorff after clipping to a bounding box using pure Python."""

    clipped_a = _clip_points(list(a), bounding_box)
    clipped_b = _clip_points(list(b), bounding_box)
    return _directed_hausdorff(clipped_a, clipped_b)


def hausdorff_clipped_naive(
    a: Iterable[Point | PointTuple], b: Iterable[Point | PointTuple], bounding_box: BoundingBox
) -> HausdorffWitness:
    """Symmetric Hausdorff after clipping to a bounding box using pure Python."""

    clipped_a = _clip_points(list(a), bounding_box)
    clipped_b = _clip_points(list(b), bounding_box)
    forward = _directed_hausdorff(clipped_a, clipped_b)
    reverse = _directed_hausdorff(clipped_b, clipped_a)
    distance_m = forward.distance_m if forward.distance_m >= reverse.distance_m else reverse.distance_m
    return HausdorffWitness(distance_m=distance_m, a_to_b=forward, b_to_a=reverse)
