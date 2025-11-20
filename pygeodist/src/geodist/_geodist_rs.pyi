"""Typing stub for the compiled geodist Rust extension.

Exports stay intentionally small while Rust-backed geometry wrappers are built.
Keep this stub in sync with `geodist-rs/src/python.rs`.
"""

from typing import Final

EARTH_RADIUS_METERS: Final[float]

class Point:
    latitude_degrees: float
    longitude_degrees: float

    def __init__(self, latitude_degrees: float, longitude_degrees: float) -> None: ...
    def to_tuple(self) -> tuple[float, float]: ...

def geodesic_distance(p1: Point, p2: Point) -> float: ...

__all__ = ["EARTH_RADIUS_METERS", "Point", "geodesic_distance"]

# Upcoming Rust-backed geometry handles will mirror the Rust structs once exposed:
# - Additional geometry containers will be added incrementally once the kernels are wired.
