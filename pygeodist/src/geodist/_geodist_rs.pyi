"""Typing stub for the compiled geodist Rust extension.

Exports stay intentionally small while Rust-backed geometry wrappers are built.
Keep this stub in sync with `geodist-rs/src/python.rs`.
"""

from typing import Final

EARTH_RADIUS_METERS: Final[float]

class Point:
    lat_deg: float
    lon_deg: float

    def __init__(self, lat_deg: float, lon_deg: float) -> None: ...
    def to_tuple(self) -> tuple[float, float]: ...

class Point3D:
    lat_deg: float
    lon_deg: float
    altitude_m: float

    def __init__(self, lat_deg: float, lon_deg: float, altitude_m: float) -> None: ...
    def to_tuple(self) -> tuple[float, float, float]: ...

class GeodesicSolution:
    distance_meters: float
    initial_bearing_degrees: float
    final_bearing_degrees: float

    def to_tuple(self) -> tuple[float, float, float]: ...

class BoundingBox:
    min_lat_deg: float
    max_lat_deg: float
    min_lon_deg: float
    max_lon_deg: float

    def __init__(
        self,
        min_lat_deg: float,
        max_lat_deg: float,
        min_lon_deg: float,
        max_lon_deg: float,
    ) -> None: ...
    def to_tuple(self) -> tuple[float, float, float, float]: ...

def geodesic_distance(p1: Point, p2: Point) -> float: ...
def geodesic_with_bearings(p1: Point, p2: Point) -> GeodesicSolution: ...
def geodesic_distance_3d(p1: Point3D, p2: Point3D) -> float: ...
def hausdorff_directed(a: list[Point], b: list[Point]) -> float: ...
def hausdorff(a: list[Point], b: list[Point]) -> float: ...
def hausdorff_directed_clipped(a: list[Point], b: list[Point], bounding_box: BoundingBox) -> float: ...
def hausdorff_clipped(a: list[Point], b: list[Point], bounding_box: BoundingBox) -> float: ...

__all__ = [
    "EARTH_RADIUS_METERS",
    "Point",
    "Point3D",
    "GeodesicSolution",
    "BoundingBox",
    "geodesic_distance",
    "geodesic_distance_3d",
    "geodesic_with_bearings",
    "hausdorff_directed",
    "hausdorff",
    "hausdorff_directed_clipped",
    "hausdorff_clipped",
]

# Upcoming Rust-backed geometry handles will mirror the Rust structs once exposed:
# - Additional geometry containers will be added incrementally once the kernels are wired.
